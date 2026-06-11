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

- **Focus**: top of the table = human playtest priority (order in table IS the queue).
- **Last session** (2026-06-10): #290 TAS frame_7800 object-count divergence proposed fixed.
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
- **Next**: #260 live slime/metal-block route (the Drill-skin headbutt blocker is now
  gameplay-CRC clean on `.cache/debug/drips-route-candidates/orig_l480_530_b528.json`
  through frame 570, so do not re-open block type C/`sub_BB54` from that route; still
  need a route that reaches the reported live slime/metal-block contact. Current clean
  near-miss: `.cache/debug/issue260-tail-grid/l524_b530_540.json` passes
  gameplay CRC and enemy checkpoints through frame 560 and reaches `(456,543)` only as
  close as healthy `dx=0 dy=-16` at frame 540. Named route
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
  native has not. The immediate `530..600` B/B+Left/B+Right climb continuations
  also do not improve the healthy score. Do not reuse this `480..520 B+Left`
  setup seed as proof. A fresh post-300 right-run/brake axis over
  `issue-260-drips-drop-contact-probe-current/scenario.json` (`300..400/430/450`
  clears, `400..520` right windows, `500..555 B+Right`, and early left brakes)
  also failed: the best raw right-target proximity was `dx=+3 dy=-8` around
  frame 456, but Kid was already unhealthy or fell into LevelTransition; every
  healthy closest stayed at the old early `dx=0 dy=-48 @128`. Do not retry this
  post-300 right-run/brake family without a different vertical setup. On the comparable left-tail seed
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
  bootstrap/route family changes.
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
  negative held-left x velocity, and frames 260..310 pass gameplay CRC. Next
  pass needs a direct human-equivalent lower left-wall/stair jump repro. A
  2026-06-11 lower-left-wall scratch route (`lower-left-wall-b-current`) is
  also not the repro: native keeps the held-left wall/jump path, while the
  first oracle split is a Drips controller/object-count mismatch at frame 103;
  #291 stale #249 gate assertion re-derived (tail sample is airborne after walking off support).
- **Gates**: verify-slice-fast green. Full published TAS
  `verify-tas --scenario tests/data/tas/any-percent-published/scenario.json
  --frame-start 0 --checkpoint-interval 1 --bisect --frontier-kind gameplay`
  passes all 37759 frames by CRC.

This block replaces the old next-agent.md handoff: rewrite it at the end of every
loop session.

For newly opened items, capture these facts in the investigation notes:

- evidence source: playtest, recording slug, oracle, or frame capture
- native repro status: reproduced / not yet reproduced
- oracle repro status: authoritative / advisory / unknown / oracle limitation
- minimum gate: `fast-slice`, `1p-playable`, `1p-accurate`, or `visual-parity`

> **Evidence lives in [`backlog-archive.md`](./backlog-archive.md).**
> This file is the table index only. Per-issue investigation notes, follow-ups,
> rejected hypotheses, and dated playtest history are cold storage — grep the
> archive by `#NNN` (e.g. `grep -n '#238' backlog-archive.md`) for an issue's
> full history. When closing an issue, flip its status here and append closing
> evidence to the archive.

| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 284 | [ ] | P0 | block/prize | 2026-06-10 playtest: later-level prize blocks award diamonds instead of their… | Reopened 2026-06-11: latest playtest still sees attract/demo prize block award a Diamond instead of Cyclops/Eyeclops/spin form. Current published TAS and cold idle attract-loop captures expose Cyclone/Iron Knight, then later real Diamonds; generated block-list/live-worker tests pass with cache assets and `capture-scenario-frames` now renders the live runtime grid. A direct Level `0x14` Devil's Marsh 2 Cyclone block route also spawns content id `3` (Cyclone), not Diamond; its synthetic oracle warp diverges at frame 0 on object count, so it is only a native rejected repro. Remaining repro needs exact live SDL path or a different later-level prize route. |
| 290 | [?] | P1 | trace/objects | Full any-percent TAS diverges at checkpoint frame_7800 (demo_play): native objects=9/4 vs oracle 10/9 | Fixed candidate: Dragon.asm loc_3CE90/loc_3CF90 breath child object-only count; full published TAS passes all 37759 frames by CRC. Human validation still required to close. |
| 260 | [~] | P0 | block/enemy | 2026-06-07 playtest: Level-2 slime/metal-block interaction breaks block and emits red projectile left | Reconfirmed 2026-06-11: Drips now creates a crawler, but the drip source appears to be a prize block. Raw Level 01 cells at row 32 are prize/drill/prize around both Drips anchors, matching current render; existing route probes still need a route/fresh repro reaching centre `EC2D` live contact. Current `478..528 Left` and b-jump climb-tail route families are exhausted: best healthy result remains `dx=+3 dy=-20` near `(584,543)`, still below the row-32 block line. |
| 277 | [ ] | P1 | player/input | 2026-06-09 playtest: holding left into a wall then jumping drops the left input… | Reconfirmed 2026-06-10 after the loc_7464 Left+Right candidate. Existing BLW2 natural L+B route is CRC-clean and honours held-left; direct human-equivalent left-wall/stair jump repro still needed. |
| 276 | [ ] | P1 | hud | 2026-06-09 playtest: player hit-point HUD does not decrement when hit (shows 4, take a hit, still shows 4) | Reclassified 2026-06-11: Iron Knight's first hit is ROM-faithful state 5 -> 4 but visually unchanged because `sub_BDC0` hides the odd fifth HP slot; regression covers this. Still open for repro where a later hit or non-Iron-Knight form should visibly decrement but does not. |
| 292 | [?] | P1 | level/timer | Fixed candidate: native timer now applies `Init_SpriteAttr_HUD`'s +1 second seed and keeps `loc_C0CC` completion-time math on the ROM `$B4` base; clock prizes also apply the separate `loc_FF92` +1s carry before `loc_FFDE` +3 minutes/cap | Verify with playtest: fresh level should show the ROM-normalised 3:00 after first tick, timer death no longer ~60 frames early, and clock prizes net a clean +3:00 after the compensating tick |
| 293 | [?] | P1 | menu/shell | Fixed candidate 2026-06-11: live pause now runs ASM `Game_Paused_Loop`/`Game_Paused_ChkStart` cursor and Start branches instead of a boolean-only toggle; Restart Round spends a life, restarts the level, clears helmet/extra HP, and restores Kid HP; last-life Give Up enters the native dying path via `loc_6C0A` semantics | Focused runtime and pure pause-menu tests pass. Human live validation still required: Pause, Down, Start with >1 life should spend one life and restart; on last life should play the bummer/dying/game-over path. |
| 294 | [x] | P1 | combat | Murder-wall contact now follows ASM `sub_5E02 loc_5E34 -> off_B640 -> Crushed_to_Death`: contact enters DeathWindup directly, bypasses HP decrement, and ignores damage invulnerability | Verified by `murder_wall_contact_crushes_helmeted_kid_loc_5e34_b640`, `murder_wall_crush_bypasses_damage_invulnerability_loc_5e34_b640`, Level 0x08 respawn regression, and `cargo test -q -p kid-core murder_wall -- --nocapture` |
| 295 | [x] | P1 | terrain | Spike routing now follows ASM `loc_91A6`/`loc_92B6`: floor and ceiling spike contacts latch spike damage and route through normal `Check_for_recent_damge -> loc_B700` HP/helmet/invulnerability handling instead of plain ceiling or instant death | Verified by floor spike damage, ceiling spike damage, and `loc_91A6` regression groups: `cargo test -q -p kid-core spike_contact -- --nocapture`, `cargo test -q -p kid-core ceiling_spike -- --nocapture`, `cargo test -q -p kid-core loc_91a6 -- --nocapture` |
| 296 | [x] | P1 | projectile | Maniaxe axes now follow ASM `loc_8ABC`: live lifetime-zero axe projectiles still deliver `colid_hurt` to enemies and are deleted on enemy contact | Verified by `live_maniaxe_axe_lifetime_zero_damages_and_deletes_loc_8abc` and the Maniaxe axe regression group: `cargo test -q -p kid-core maniaxe_axe -- --nocapture` |
| 297 | [x] | P1 | block | Ice-block smash paths now route through ASM `IceBlock_SmashAndLoadBullets`: `off_BB80[2]` ceiling headbutts delete the ice cell and spawn ice bullets, and Red Stealth `off_B058[2]` floor impacts load the red bullet pair before rebound | Verified by `cargo test -q -p kid-core ceiling_ice -- --nocapture` and `cargo test -q -p kid-core red_stealth_ice -- --nocapture` |
| 298 | [?] | P1 | block | Fixed candidate: telepad warp now requires `sub_B43A loc_B522`'s `$5A` grounded countdown; first-contact landing and raw feet-position lookups no longer bypass the timer | Human playtest: stand on telepad, warp only at ~frame 90; quick walk-over should not warp |
| 299 | [?] | P1 | level | Fixed candidate: ending trigger now follows ASM `loc_D7F4`'s direct `Current_LevelID == Final_LevelID ($33)` comparison instead of treating the last MapOrder entry as final | Human/playtest: completing logical `$33` enters ending; any later MapOrder tail level continues through normal Teleport/title-card flow |
| 300 | [?] | P1 | level | Fixed candidate: teleporter destination now follows ASM `loc_BCD6` by reverse-scanning MapOrder and storing the logical level index, not the raw map byte | Human/playtest: inter-level teleporter where map id differs from logical index should load the ROM destination; same-level teleport still repositions only |
| 301 | [ ] | P1 | enemy/boss | Umbrella: bosses-fidelity-audit.md — Shishkaboss HP, boss-level flag suppression, HeadyMetal `loc_37582` velocity/hover, `sub_37332` eye-launch physics, boss-head `sfx_Boss_dies`, live multi-head fire Y-window, live `sub_39846` fixed boss-head movement, live `sub_3A292` collision/HP/flash/groan, live `sub_397E4` boss-head projectile spawn, `loc_3A0D2` arrow move-then-stick lifecycle, and `loc_3A1EA`/`loc_3A24C` boomerang acceleration/countdown lifecycle sub-gaps fixed; remaining high gaps include `loc_39EBE` status exits, animation-gated `loc_37F30` attack FSM, and HeadyMetal composite/freeze/fire handlers/lifecycle | Per-gap rows in bosses-fidelity-audit.md; continue with one coherent boss-engine slice at a time |
| 302 | [?] | P2 | block/prize | Fixed candidate: disguised `$E101` prize blocks now take the ROM reveal-only first hit (`loc_101D8` writes `$E102`; prize needs a second hit) | Headbutt `$E101`: first hit converts to visible `$E102` with no prize; second hit pays |
| 303 | [?] | P2 | menu | Fixed candidate: CONTINUE? spends a continue only on Start and keeps the ROM `loc_D3AA` second Start window / 239-frame timeout tail | Tap A/B/C on CONTINUE?: countdown continues, nothing spent; no-input title-return waits through the ROM tail; Start during second window spends the continue |
| 304 | [x] | P2 | player/input | Fixed: live input now applies `Permute_ABCButtons` before the held/pressed latch and Fast Action now inverts `Ctrl_A_Held` in gameplay input, preserving the demo-mode skip | Verified with `cargo test -q -p kid-core input_ -- --nocapture` and `mise run verify-slice-fast` |
| 305 | [ ] | P2 | verify | Add interactive-lane oracle scenarios for demo-skipped paths (options remap, Fast Action, pause menu): ROM disables these in demo mode so existing TAS/demo traces structurally cannot cover them; fresh DynamicScenarios navigating title-options-gameplay drive identical input paths in native and oracle | Scenario suite exists and is oracle-compared in a gate; covers #293/#304 fixes |
| 306 | [ ] | P2 | platform | Umbrella: platforms-fidelity-audit.md — 32 of 40 gaps still open after the 2026-06-10 staleness check (least-worked May audit) | Per-gap rows in doc; verdicts in fidelity-audit-staleness-2026-06-10.md |
| 307 | [ ] | P2 | level | Umbrella: level-loading-fidelity-audit.md — 34 of 55 still open (+11 partial) after the staleness check | Per-gap rows in doc; staleness report has current file:line evidence |
| 308 | [ ] | P2 | combat | Umbrella: combat-fidelity-audit.md — 21 gaps incl. per-sprite-frame `CollisionSize_Index` hitboxes vs invented fixed 8x16 contact box, colid word delivery, Berzerker reflect bit (#294 promoted separately) | Per-gap rows in doc |
| 309 | [ ] | P2 | block | Umbrella: blocks-fidelity-audit.md — 27 gaps (#297/#298/#302 promoted separately) | Per-gap rows in doc |
| 310 | [ ] | P2 | terrain | Umbrella: terrain-fidelity-audit.md — 18 gaps (#295 promoted separately) | Per-gap rows in doc |
| 311 | [ ] | P2 | projectile | Umbrella: projectiles-fidelity-audit.md — 16 gaps incl. Juggernaut skull terrain probe and invented below-foot airborne hitbox (#296 promoted separately) | Per-gap rows in doc |
| 312 | [ ] | P2 | level | Umbrella: level-flow-fidelity-audit.md — remaining gaps incl. final-level routing, teleporter MapOrder translation, and timer semantics (#299/#300/#321 promoted separately) | Per-gap rows in doc |
| 313 | [ ] | P2 | menu/shell | Umbrella: shell-fidelity-audit.md — remaining gaps (#292/#293/#303/#304 promoted separately); `Allow_Pausing` is a fixed candidate, pause-render/game-over hold details remain open | Per-gap rows in doc |
| 314 | [ ] | P2 | score | Umbrella: scoring-economy-fidelity-audit.md — 9 gaps incl. Hundred-K-Trip timer armed but never decremented (warp can never fire) and one-time-prize anti-farming unwired (1-ups re-farmable) | Per-gap rows in doc |
| 315 | [ ] | P2 | enemy | Umbrella: enemy-fidelity-audit.md — 25 of 67 still open plus 9 partial remainders (e.g. Big Hopping Skull patrol bounds, Ninja knockdown branch) after the staleness check | Staleness report lists each remainder with current file:line |
| 316 | [ ] | P2 | form | Umbrella: helmets-fidelity-audit.md — 25 of 38 still open after the staleness check | Per-gap rows in doc; staleness report verdicts |
| 317 | [ ] | P3 | weather | Umbrella: weather-fidelity-audit.md — 16 of 33 still open after the staleness check | Per-gap rows in doc |
| 318 | [ ] | P3 | player | Umbrella: movement-fidelity-audit.md — 8 of 25 still open after the staleness check | Per-gap rows in doc |
| 319 | [ ] | P3 | audio | Umbrella: audio-triggers-fidelity-audit.md — 20 trigger gaps; loop already wired Ankh/evanescent/ice-block prize SFX (89280c99..9973af5c) | Per-gap rows in doc; tick off as wired |
| 320 | [ ] | P3 | camera | Umbrella: camera-fidelity-audit.md — 6 gaps, none high | Per-gap rows in doc |
| 321 | [?] | P1 | level | Fixed candidate: death respawn now clears `Extra_hitpoint_slots` on the ROM `Death`/`lose_life` path before restoring base The Kid HP | Die after earning extra HP slots; respawn/next helmet pickup must use base+0 slots, not stale DP slots |
| 291 | [x] | P1 | player | NOT A BUG: BLW2 no-jump left-pit wall gate was asserting the tail frame after the route walked off support; contact/hold frames stay out of loc_ABA4 ledge snap and do not inject upward velocity | cargo test -p kidcli gameplay_regression_issue249_blw2_left_pit_wall_no_jump_does_not_auto_launch_juggernaut -- --nocapture |
| 001 | [x] | P0 | player | Player gets stuck after first damage | Take hit, confirm movement resumes |
| 002 | [x] | P0 | enemy | Enemies don't die when stomped | Stomp enemy on level 0x00, confirm death |
| 003 | [?] | P1 | audio | No music or sound effects playing | Blocked on exact human/real-device repro; current deterministic cold-intro and title/options live-SDL routes produce BGM |
| 004 | [x] | P1 | render | Colour palette wrong (floor/tree colours reversed) | Capture frame, compare to oracle |
| 005 | [-] | P1 | render | No parallax background | NOT A BUG — parallax working correctly |
| 006 | [x] | P1 | enemy | Dragon moonwalking (sprite direction vs movement) | Watch dragon, confirm facing matches walk dir |
| 007 | [x] | P1 | camera | Screen shakes/moves weirdly when hit | Take damage, confirm camera stays stable |
| 008 | [x] | P1 | render | Unwanted dark overlay effect when hit | Take damage, confirm no full-screen tint |
| 009 | [x] | P2 | menu | No menu — spawns directly into game | Launch, confirm title screen appears |
| 010 | [x] | P2 | enemy | No enemy attack patterns (fire breath, arrows) | Reach archer/dragon, confirm projectiles |
| 011 | [x] | P2 | block | Non-P bricks look half-broken (wrong sprite state) | Resolved by palette fix (#004) |
| 012 | [x] | P2 | block | No particle effects when blocks break | Hit block, confirm debris particles |
| 013 | [x] | P2 | block | P blocks don't convert to regular blocks after hit | archived |
| 014 | [x] | P2 | block | Diamonds/items don't visually emerge from blocks | Hit prize block, confirm item sprite appears |
| 015 | [x] | P2 | player | Can't wall-grab (A+A to grab block edges) | Jump at wall edge, confirm grab |
| 016 | [x] | P2 | level | No level intro screen showing level name | Enter level, confirm name card |
| 017 | [x] | P2 | form | Masks/helmets not visible when they appear | Hit helmet block, confirm sprite |
| 018 | [x] | P2 | form | No form-change animation when equipping helmet | Placeholder flash; real animation pending |
| 019 | [x] | P2 | form | Player doesn't transform at all | Collect helmet, confirm form changes |
| 020 | [x] | P3 | render | No water/shimmer background effect | Load water level, confirm shimmer |
| 021 | [x] | P3 | menu | Sound Test does nothing (cursor==1 unhandled) | Enter options, select Sound Test |
| 022 | [x] | P3 | render | Duplicate Game Over overlay rendering | Game over screen renders once |
| 023 | [x] | P2 | render | Floating item sprites use placeholder rectangles | Replace with recognisable pixel-art icons |
| 024 | [x] | P1 | enemy | UFO beam child object system not implemented | UFO fires beam segments downward |
| 025 | [x] | P2 | enemy | Fire Demon death trail not implemented | Kill Fire Demon, confirm fire trail |
| 026 | [x] | P3 | menu | Demo playback — ROM has Demo_InputData1/2 at kid.asm:33373 | Title screen auto-plays demo |
| 027 | [x] | P2 | level | Multi-screen block list cross-level references | Graceful fallback (ba36f63) |
| 028 | [x] | P3 | testing | Adopt `insta` snapshot testing for NativeTraceFrame | Snapshots detect state drift between commits |
| 029 | [x] | P3 | testing | Adopt `proptest` for physics pure functions | Speed tables, slopes, gravity hold across input range |
| 030 | [x] | P3 | infra | Add a local-first verification entrypoint… | archived |
| 031 | [x] | P3 | infra | Add performance baseline tracking | criterion benchmarks: ~92ns/frame idle, ~3µs level load |
| 032 | [x] | P2 | fidelity | Per-form Diamond Power effects (20 unique abilities) | Each form has correct visual and damage pattern |
| 033 | [x] | P2 | fidelity | Form-change animation from ASM Kid_Transform tables | archived |
| 034 | [x] | P3 | fidelity | Intro sequence 5 segments (ASM has 5, native had 3) | Boot captures match oracle |
| 035 | [-] | P2 | fidelity | Resolve 6 oracle-warp-broken levels (0x4E73) | archived |
| 036 | [x] | P2 | enemy | Tank walk speed step wrong ($10000 should be $8000) | Tank speed matches ASM at all HP tiers |
| 037 | [x] | P3 | fidelity | Floor-snap settling diffs: 0x24/0x51 (1px cam_y) | archived |
| 038 | [-] | P3 | fidelity | Camera Y settling 1px diffs on 0x02, 0x24, 0x51 | MERGED with #037 — same root cause |
| 039 | [-] | P3 | fidelity | 4th speed table entry is dead data in ROM | NOT A BUG — unreachable slot |
| 040 | [x] | P2 | enemy | Lion constants wrong (detect range, cooldown, projectile) | Lion matches ASM line-by-line |
| 041 | [x] | P2 | enemy | Hand jump velocity fixed, should be randomised per-tier | Hand matches ASM jump tables |
| 042 | [x] | P2 | enemy | Tornado acceleration $300 should be $C00,… | Tornado matches ASM init table |
| 043 | [x] | P2 | enemy | Dragon Flying is structural placeholder (3 variant scripts) | Dragon Flying matches ASM flight scripts |
| 044 | [x] | P3 | enemy | Lion projectile wall-bounce and per-enemy homing accel | Projectile bounces off terrain |
| 045 | [x] | P3 | enemy | Hand player-grab mechanic missing entirely | Hand grabs and drags player |
| 046 | [x] | P3 | enemy | Tar Monster timing/visibility/glob shot wrong | Tar Monster matches ASM bytecode |
| 047 | [x] | P3 | enemy | Drips random timing and surface-crawl spawning | Drips matches ASM drop timing |
| 048 | [x] | P3 | enemy | Spinning Twins orbit-to-walk should be collision-triggered | Transition on player contact |
| 049 | [-] | P3 | fidelity | Level 0x29 has 5px X position diff… | MERGED with #037 — same root cause |
| 050 | [-] | P3 | fidelity | 4 levels show idle_tail position diffs from… | archived |
| 051 | [x] | P2 | enemy | Dragon HP base 2 should be 1 (ASM Dragon.asm:65 addq.w #1) | Dragon HP matches ASM |
| 052 | [x] | P2 | enemy | EmoRock velocity cap logic wrong ($60000… | EmoRock drop matches ASM |
| 053 | [x] | P2 | enemy | Robot charge permanent until wall, not timed exit to walking | Robot charge matches ASM |
| 054 | [x] | P3 | enemy | MiniHoppingSkull ground delay 21 not 32 (dbf #$14) | Skull delay matches ASM |
| 055 | [x] | P3 | enemy | BagelBrothers projectile speed 3 not 4, lifetime 50 not 45 | Projectile matches ASM |
| 056 | [x] | P3 | enemy | BigHoppingSkull is continuous bouncer, not discrete hopper | archived |
| 057 | [x] | P3 | enemy | Goat player-tracking callback missing (turns… | Goat tracks player |
| 058 | [x] | P3 | enemy | Cloud 60-frame post-bounce idle period missing | Cloud idles after bounce |
| 059 | [x] | P3 | enemy | Fireball ceiling bounce uses Sphere formula,… | Fireball bounce matches ASM |
| 060 | [x] | P2 | enemy | Boss multi-head system… | 3 heads per boss with ASM offsets + per-head HP |
| 061 | [x] | P3 | enemy | Orca missing ceiling bounce, slope handling,… | archived |
| 062 | [x] | P3 | enemy | HeadyMetal death wobble visual (rendering-only) | archived |
| 063 | [x] | P2 | block | Ghost block restored with collision 7 (Spikes) not 6 (Solid) | Ghost blocks walkable |
| 064 | [x] | P2 | block | Drill block projectile fired downward,… | Drill projectile horizontal |
| 065 | [?] | P3 | block | Shooter fire: ASM uses per-block registration on visibility, native uses global timer | Shooter blocks now register per visible cell, fire on entry, and recycle after… |
| 066 | [x] | P3 | block | SoftPlatform bounce moves platform object | archived |
| 067 | [x] | P3 | block | Lift block uses accel physics; ASM uses coroutine animation | Lift matches ASM: 2px/frame×8 up, erase, 1px/frame×16 down |
| 068 | [x] | P3 | block | Shifting block grid swap is instant… | archived |
| 069 | [x] | P2 | fidelity | TAS boot slice desync: native consumed early… | archived |
| 070 | [x] | P1 | fidelity | TAS frame 956: post-ledge-snap movement state suppression | archived |
| 071 | [-] | P3 | infra | AsmFaithful Phase 2: floor collision… | DONE — resolved in #080 |
| 072 | [-] | P3 | infra | BlastEm second oracle for cross-validation | archived |
| 073 | [-] | P3 | infra | Instruction-level M68K trace from jgenesis | archived |
| 074 | [x] | P1 | fidelity | TAS frame 986: sub-pixel X drift from false wall probe | Tile-boundary guard (sub_8F26), frontier now 987 |
| 075 | [x] | P1 | fidelity | TAS frame 987: wall-cling h_accel… | archived |
| 076 | [x] | P1 | fidelity | TAS frame 988: spurious Cyclone form change… | archived |
| 077 | [x] | P1 | fidelity | TAS frame 1056: prize item physics +… | archived |
| 078 | [x] | P1 | fidelity | TAS frame 1129: demo input freeze during… | archived |
| 079 | [x] | P1 | fidelity | TAS frame 1144: per-form collision… | archived |
| 080 | [x] | P1 | fidelity | AsmFaithful Phase 2: sub_902A before sub_8F26 position split | archived |
| 081 | [x] | P2 | fidelity | Walking floor-ahead check (sub_B43A) | archived |
| 082 | [x] | P2 | fidelity | TAS frame 1147: demo byte boundary parity… | archived |
| 083 | [x] | P2 | fidelity | TAS frame 1170: sub_902A floor landing sub-pixel clearing | archived |
| 084 | [x] | P2 | fidelity | TAS frame 1209: Cyclone flight… | archived |
| 085 | [x] | P2 | fidelity | TAS frame 1301: Cyclone accumulator wrapping arithmetic | archived |
| 086 | [x] | P2 | fidelity | TAS frame 1365: ceiling block preservation… | archived |
| 087 | [x] | P2 | fidelity | TAS frame 1396: post-prize-hit ceiling latch in demo 0x1B | archived |
| 088 | [x] | P2 | fidelity | TAS frame 1405: floating prize collection… | archived |
| 089 | [x] | P2 | fidelity | TAS frame 1406: post-latch diamond… | archived |
| 090 | [x] | P2 | fidelity | TAS frame 1755: AsmFaithful solid-floor… | archived |
| 091 | [x] | P2 | fidelity | TAS frame 1772: demo 0x1B returns to title one frame early | archived |
| 092 | [x] | P2 | fidelity | TAS frame 1774: first title frames keep the… | archived |
| 093 | [x] | P2 | fidelity | TAS frame 1802: title return blanks Kid… | archived |
| 094 | [x] | P2 | fidelity | TAS frame 1832: title pose appears while… | archived |
| 095 | [x] | P2 | fidelity | TAS frame 2529: demo 0x15 startup reused the… | archived |
| 096 | [x] | P2 | fidelity | TAS frame 2625: demo 0x15 handoff lands one… | archived |
| 097 | [x] | P2 | fidelity | TAS frame 2698: demo 0x15 standingstill… | archived |
| 098 | [x] | P2 | fidelity | TAS frame 2723: demo 0x15 should release the… | archived |
| 099 | [x] | P2 | fidelity | TAS frame 2773: grounded left-wall snap… | archived |
| 100 | [x] | P2 | fidelity | TAS frame 2831: buffered wall-sticky pull-up… | archived |
| 101 | [x] | P2 | fidelity | TAS frame 2938: descending right-wall cling… | archived |
| 102 | [x] | P2 | fidelity | TAS frame 3252: IronKnight transform freeze… | archived |
| 103 | [x] | P2 | fidelity | TAS frame 3400 prep: native enemy loading… | archived |
| 104 | [x] | P2 | fidelity | TAS frame 3400: second Knights Isle stomp… | archived |
| 105 | [x] | P2 | fidelity | TAS frame 3414: demo 0x15 pause snapshot… | archived |
| 106 | [x] | P2 | fidelity | TAS frame 287: `any-percent-published`… | archived |
| 107 | [x] | P2 | fidelity | TAS frame 1830: `any-percent-published`… | archived |
| 108 | [x] | P2 | fidelity | TAS frame 4905: `any-percent-published`… | archived |
| 109 | [x] | P2 | fidelity | TAS frame 4966: `any-percent-published` demo… | archived |
| 110 | [x] | P2 | fidelity | TAS frame 4966: `any-percent-published`… | archived |
| 111 | [x] | P2 | fidelity | TAS frame 5116: `any-percent-published`… | archived |
| 112 | [x] | P2 | fidelity | TAS frame 5143: `any-percent-published`… | archived |
| 113 | [x] | P2 | fidelity | TAS frame 5303: `any-percent-published`… | archived |
| 114 | [x] | P2 | fidelity | TAS frame 5584: `any-percent-published`… | archived |
| 115 | [x] | P2 | fidelity | TAS frame 5994: `any-percent-published`… | archived |
| 116 | [x] | P2 | fidelity | TAS frame 6033: `any-percent-published`… | archived |
| 117 | [x] | P2 | fidelity | TAS frame 6358: `any-percent-published`… | archived |
| 118 | [x] | P2 | fidelity | TAS frame 6906: `any-percent-published`… | archived |
| 119 | [x] | P2 | fidelity | TAS frame 6945: `any-percent-published`… | archived |
| 120 | [x] | P2 | fidelity | TAS frame 6966: `any-percent-published`… | archived |
| 121 | [x] | P2 | fidelity | TAS frame 24921: level `0x15` gameplay… | archived |
| 122 | [x] | P0 | level | Level 2 entry can spawn Kid in midair, then… | archived |
| 123 | [x] | P0 | player | Death/respawn still leaves camera drift,… | archived |
| 124 | [x] | P1 | enemy | Enemy kill timing still allows Kid to get… | archived |
| 125 | [x] | P1 | block | Breakable blocks can render with the wrong… | archived |
| 126 | [?] | P1 | player | Walking up slopes is still observably wrong in native play | Kid Level 00 natural route is CRC-clean; Red Stealth Level 01 route is blocked… |
| 127 | [x] | P1 | block | Black bouncy blocks do not produce the… | archived |
| 128 | [x] | P1 | form | Form-specific block breaking can now destroy… | archived |
| 129 | [x] | P1 | infra | Add a `1p-playable` gate that covers native… | archived |
| 130 | [x] | P1 | infra | Turn `play-native --record` /… | archived |
| 131 | [x] | P2 | fidelity | Burn down the five parked DEVIATION items… | archived |
| 132 | [x] | P3 | gameflow | Verify 2-player alternating mode against… | archived |
| 133 | [x] | P3 | gameflow | Investigate password flow and decide whether… | archived |
| 134 | [x] | P1 | infra | Package reduced semantic cases as the… | archived |
| 135 | [x] | P1 | render | Level 1 background graphics do not match the original game | archived |
| 136 | [x] | P1 | block | Diamonds and helmets emerging from bricks… | archived |
| 137 | [x] | P1 | player | Kid walks backwards while climbing slopes | archived |
| 138 | [x] | P1 | player | Enemy contact appears to apply double damage… | archived |
| 139 | [x] | P1 | render | Bouncy blocks still render like spikes | archived |
| 140 | [x] | P1 | render | Prize blocks use wrong graphics and animations | archived |
| 141 | [?] | P1 | render | Intro and level-intro graphics still do not match the original game | Reopened as evidence-needed after the 2026-06-02 playtest report. Fresh… |
| 142 | [x] | P1 | enemy | Level 1 dragon breath travels too far | archived |
| 143 | [x] | P1 | block | Heavy Man's first helmet causes repeated… | archived |
| 144 | [?] | P1 | enemy | Level 2 shooter enemy fires too rapidly | Evidence-needed after the latest live report. Current `main` still passes the… |
| 145 | [x] | P1 | render | FIXED 2026-06-06: the full-tank turret (base… | archived |
| 146 | [x] | P1 | level | Dying in Level 2 can leave the game stuck on… | archived |
| 147 | [?] | P1 | player | FIXED 2026-06-04 (oracle-verified): culprit was recent commit 708e663c… | Evidence-needed after latest form-specific playtest reports. The original Kid… |
| 148 | [x] | P2 | player | Kid death animation does not match the original | archived |
| 149 | [x] | P2 | block | Hit blocks do not shake before breaking | archived |
| 150 | [x] | P2 | render | Kid blinks invisible after taking damage… | archived |
| 151 | [x] | P2 | render | Red Stealth (samurai) helmet graphics look… | archived |
| 152 | [x] | P1 | player | Red Stealth (samurai) cannot walk up… | archived |
| 153 | [x] | P2 | player | Red Stealth sword attack while jumping… | archived |
| 154 | [x] | P1 | render | Bouncy blocks invisible at the bottom of the… | archived |
| 155 | [?] | P2 | enemy | Slimes (Drips family?) hang at wrong attach point and shoot red projectile downward incorrectly | Landed in commits `152d2d1b` (DripsDrop projectile renders green-tinted instead… |
| 156 | [x] | P3 | render | Block destruction still uses a small… | archived |
| 157 | [?] | P3 | render | Level-complete bonus overlay uses typed text instead of EOL sprite workers | Fixed 2026-05-19: `kid-render` now loads… |
| 158 | [x] | P1 | player | Berzerker/rhino head charge does not hit… | archived |
| 159 | [x] | P2 | debug | `n`/`p` debug level changes show a fake… | archived |
| 160 | [?] | P0 | audio | Native play still has little or no real music/SFX | User playtest 2026-05-25 reports the game is basically silent. Root causes… |
| 161 | [?] | P1 | render | Intro sequence is visually incomplete and misses scenes/Kid animation | User playtest 2026-05-25 reports the introduction still lacks the real scene… |
| 162 | [x] | P1 | render | Level-card graphics are wrong even when the… | archived |
| 163 | [x] | P1 | render | HUD diamond and special-prize icons render… | archived |
| 164 | [x] | P1 | render | Falling helmet prize sprites still look like approximations | archived |
| 165 | [x] | P1 | player | Wall-grab pull-up lacks the ROM rollover/flip animation | archived |
| 166 | [x] | P1 | render | Enemy sprites have transparent holes / wrong masks | archived |
| 167 | [?] | P1 | enemy | Enemy deaths disappear instead of showing ROM death/body animation | User playtest 2026-05-25: enemies often just vanish when killed. Partial: Mini… |
| 168 | [x] | P1 | player | Berzerker/rhino side charge does not break side blocks | archived |
| 169 | [?] | P1 | enemy | Level 2 Drips/slime placement and drop/attack behaviour still looks wrong | Fixed 2026-05-25: first concrete mismatch was rendering, not the whole Drips… |
| 170 | [x] | P2 | gameflow | Death/continue restart still shows fake level loader | archived |
| 171 | [x] | P1 | render | HUD hitpoint/life pips miss their grey second tile | archived |
| 172 | [x] | P1 | render | HUD diamond icon should animate | archived |
| 173 | [x] | P1 | block | Broken bricks should emit shard debris | archived |
| 174 | [x] | P1 | block | Prize blocks should show spawn explosion above the block | archived |
| 175 | [x] | P1 | gameflow | Winning the level sequence is incomplete | archived |
| 176 | [?] | P1 | enemy | Wire ASM-faithful enemy helpers into the live step/combat dispatch (systemic root cause) | Enemy fidelity audit 2026-05-29 (`docs/source-port/enemy-fidelity-audit.md`)… |
| 177 | [?] | P1 | enemy | Diamond modelled as a Collectible pickup, not a hostile HP enemy (stub) | Partial 2026-05-30: live Diamond layout objects are no longer… |
| 178 | [?] | P1 | enemy | Armadillo: damage recoil window, Berzerker curl-launch, child fall-then-die, walk-speed cap not wired | Partial 2026-05-29: charging Berzerker side contact with no positive `$42` now… |
| 179 | [?] | P1 | enemy | Big_Hopping_Skull: patrol corridor, relaunch, floor-stop, per-hit SFX, respawn marker not wired | Partial 2026-05-30: live flat-floor contact no longer negates accumulated… |
| 180 | [x] | P1 | enemy | Crab: directional front-immune vulnerability… | archived |
| 181 | [x] | P1 | enemy | Driller: probe offset wrong, drills every… | archived |
| 182 | [?] | P1 | enemy | Emo_Rock: invented ground-walk, no disappear/respawn re-attack, Berzerker launch + death routing missing | Partial 2026-05-29: settled side contact with a charging Berzerker now follows… |
| 183 | [?] | P1 | enemy | Fire_Demon: walking fire-trail emission, Berzerker launch, hurt recovery anim not driven | Partial 2026-05-29: charging Berzerker side contact now follows ASM `loc_3A5E4`… |
| 184 | [?] | P1 | enemy | Goat: Berzerker launch, collision-type dispatch, recoil shove + hurt recoil missing | Partial 2026-05-29: charging Berzerker side contact with no positive `$42` now… |
| 185 | [?] | P1 | enemy | Hand: grab is cosmetic; collision dispatch tables and grab gating not wired | Partial 2026-05-30: live grab countdown now follows ASM `loc_3292E` by sampling… |
| 186 | [?] | P1 | enemy | Lion: hurt recoil, gravity ledge-leap charge, roar windup + engagement bands missing | Partial 2026-05-30: live Lion timer expiry now enters explicit `Roaring` and… |
| 187 | [?] | P1 | enemy | Mini_Hopping_Skull: generic step_hopper replaces the ASM hop state machine | Partial 2026-05-29: charging Berzerker side contact now follows ASM `loc_3F262`… |
| 188 | [?] | P1 | enemy | Ninja: attack cadence, block/guard state machine, Berzerker launch, knockdown + airborne branches missing | Partial 2026-05-29: charging Berzerker side contact now follows ASM… |
| 189 | [?] | P1 | enemy | Orca: in-lunge contact dispatch, ceiling $44 fatal-fall, windup phase, side re-lunge missing | Partial 2026-05-29: ordinary charging Berzerker side contact now enters the… |
| 190 | [x] | P1 | enemy | Robot: charge entered every shot, no aim… | archived |
| 191 | [x] | P1 | enemy | Scorpion: sting deals no damage;… | archived |
| 192 | [x] | P1 | enemy | Spinning_Twins: orbit never splits into two… | archived |
| 193 | [x] | P1 | enemy | Tank: body-vs-mouth vulnerability,… | archived |
| 194 | [x] | P1 | enemy | Tar_Monster: 5-projectile burst fires on the… | archived |
| 195 | [x] | P1 | enemy | Tornado: Cyclone-above kill branch, ceiling… | archived |
| 196 | [x] | P1 | enemy | UFO: bounce table, chase steering, motion… | archived |
| 197 | [x] | P2 | enemy | Archer: fire cadence ~4-5x too fast; shot… | archived |
| 198 | [x] | P2 | enemy | Cloud: lightning projectile has a fixed… | archived |
| 199 | [x] | P2 | enemy | Dragon (walking): breath flame deleted on… | archived |
| 200 | [x] | P2 | enemy | Dragon_Flying: descent fireball lacks… | archived |
| 201 | [x] | P2 | enemy | Fireball: fire-trail child objects never… | archived |
| 202 | [x] | P2 | enemy | Tank_Shooting: bullets pass through walls;… | archived |
| 203 | [?] | P1 | architecture | Dead-helper census: ASM-faithful unit-tested helpers exist crate-wide but the… | Codebase-wide follow-up to the enemy audit. 2,972 `allow(dead_code)`-gated… |
| 204 | [x] | P1 | player | REOPENED 2026-06-03 (Samurai/Red Stealth… | archived |
| 205 | [?] | P1 | infra | Object/enemy-active oracle scenario harness — prerequisite for verifying dead-helper wiring (#206-#208) | The idle sweep exercises no boss / Diamond-Power-weapon / platform /… |
| 206 | [?] | P1 | player | Diamond Power weapons collapsed to an invulnerability stub; full weapon set modelled but unwired | Current `step_diamond_power` is no longer just invulnerability: rain is wired;… |
| 207 | [?] | P1 | architecture | Generic collision primitives unwired (engine-wide); relates to #139/#140/#143 | Current 2026-06-02 status: evidence-needed, not closed. ASM `Mode_Level` runs… |
| 208 | [?] | P1 | player | All platform physics unwired (collision / carry / landing / overlap-resolution) | Current 2026-06-02 status: partially corrected, still evidence-needed. The thin… |
| 209 | [x] | P3 | architecture | Hygiene: strip stale `cfg_attr(not(test),… | archived |
| 210 | [?] | P0 | gameflow/render | 2026-05-30 playtest: intro/title/menu path still visually corrupt and can re-enter demo loop over Start/menu | Evidence source: human playtest after pushed commits. Symptoms: Wild Side kids… |
| 211 | [?] | P1 | player/render | 2026-05-30 playtest: live slope traversal still shows wrong-facing/diagonal animation despite green slope helper tests | Evidence source: human playtest after pushed commits. Symptoms: The Kid appears… |
| 212 | [x] | P1 | player/combat | 2026-05-30 playtest: Helmet/IronKnight seems… | archived |
| 213 | [?] | P1 | enemy/block | 2026-05-30 playtest: Level 01 Drips/slime and metal-block interaction still wrong | Evidence source: human playtest after pushed commits. Symptoms: Drips appears… |
| 214 | [?] | P1 | block/enemy | 2026-05-30 playtest: Highwater shooter blocks still have live-path projectile/barrier regressions | Evidence source: human playtest after pushed commits. Symptoms: top shooter… |
| 215 | [b] | P2 | render | 2026-05-30 playtest: water/foreground priority layering wrong in Level 02 | INVESTIGATED 2026-06-03, BLOCKED:needs-repro. The native sprite-vs-plane… |
| 216 | [?] | P2 | audio | 2026-05-30 playtest: SFX timing/level-complete sound still lag or cut off | Evidence source: human playtest after pushed commits. Symptoms: block break and… |
| 217 | [?] | P1 | player | Wire ASM-faithful player move-state helpers into the live step (systemic root cause, movement layer) | Player-movement fidelity audit 2026-05-30… |
| 218 | [?] | P1 | player | Ground speed tables inverted: ice plays grippy, rubber plays slippery (ROM cross-wire undone) | `sub_8600` writes terrain index 1 for ice blocks / 2 for rubber; selector… |
| 219 | [x] | P2 | player | `KidGrabbedByHand` RAM flag never surfaced… | archived |
| 220 | [x] | P2 | player | Crawl stand-up & entry gating incomplete… | archived |
| 221 | [x] | P2 | player | Airborne Skycutter uses generic jump-air… | archived |
| 222 | [x] | P2 | player | Up-/Down-held does not drive uphill/downhill… | archived |
| 223 | [x] | P2 | player | Iron Knight climb edge behaviours (wall-end… | archived |
| 224 | [x] | P2 | architecture | Compound `$14/$18` slope-corner collision… | archived |
| 225 | [x] | P2 | player | Remove/gate invented movement state changes… | archived |
| 226 | [x] | P3 | player | Minor input/audio parity: L+R resolver… | archived |
| 227 | [?] | P1 | platform | Wire ASM-faithful platform helpers into the live path (systemic root cause, platform layer); details #208 | Platform fidelity audit 2026-05-31… |
| 228 | [?] | P1 | player | Wire ASM-faithful helmet/form/Diamond-Power helpers into the live path; extends #206 beyond the DP invuln stub | Reconfirmed 2026-06-11 play-native: Cyclops/Eyeclops/spin-form level skip still does not work as a playable form route. Keep broad helmet/form/Diamond-Power wiring open; use a natural or state-aligned form route before changing live helpers. |
| 229 | [?] | P1 | level | Level-loading fidelity: `Load_Level` tail modelled but dead; live load… | Advanced 2026-06-02: native `level_bounds()` now matches ASM `Load_LevelData… |
| 230 | [?] | P1 | weather | Weather/environment fidelity: storm machine, hail worker, and geyser state… | Advanced 2026-06-02: live storm+hail spawn now respects ASM `sub_61CA`'s five… |
| 231 | [x] | P1 | platform | Platform placement x/y may omit the ROM `asl… | archived |
| 232 | [x] | P2 | block | Lift platform spawns with y-velocity 0… | archived |
| 233 | [x] | P2 | player | Maniaxe axe turnaround reverses… | archived |
| 234 | [x] | P2 | player | ExtraHitPoint Diamond Power bypasses the… | archived |
| 235 | [x] | P2 | level | Non-oracle level-load fallback feeds the raw… | archived |
| 236 | [x] | P2 | player | Maniaxe axe flight uses invented finite… | archived |
| 237 | [x] | P2 | player | Maniaxe axe terrain delete is broader than… | archived |
| 238 | [?] | P0 | level | Transition after Highwater Pass can load the next level into instant death —… | FIXED 2026-06-01: native `StartingLevel` title-card hand-off now follows ASM… |
| 239 | [-] | P1 | enemy | 2026-06-03 playtest: Level 0 walking dragon… | archived |
| 240 | [-] | P1 | enemy | 2026-06-03 playtest: Level 0 dragon breathes… | archived |
| 241 | [b] | P1 | hud | 2026-06-03 playtest: lives HUD miscounts — helmet + 6 lives shown, take a hit (no real death), lives drop to 4 | NEEDS HUMAN REPRO 2026-06-03. ASM cross-check (recon): native lives logic is… |
| 242 | [?] | P2 | audio | 2026-06-03 playtest: bouncy/rubber tiles play the brick-break SFX instead of a bounce sound | FIXED 2026-06-03 (commit 2650c7df): the springy "bouncy" tiles are Metal blocks… |
| 243 | [?] | P2 | enemy | 2026-06-03 playtest: "black-eye" enemy flickers (appears/disappears) and fires… | RESOLVED 2026-06-03. Enemy IDENTIFIED = Tar Monster (obj 0x05), in Highwater… |
| 244 | [-] | P2 | enemy | 2026-06-03 playtest: duplicated whole tanks… | archived |
| 245 | [x] | P1 | gameflow | 2026-06-03 playtest: dying can leave the… | archived |
| 246 | [?] | P1 | level | 2026-06-03 playtest: intermittent "failed to load visuals" error when skipping between levels | PLAYER PATH FIXED 2026-06-04: the native PLAYER (kidcli) had its OWN… |
| 247 | [x] | P2 | level | Theme collision table trimmed: high skin… | archived |
| 248 | [?] | P0 | player | 2026-06-04 playtest: death → black/frozen screen for NON-dragon-graze deaths… | #245 fixed the Level-1 grounded-dragon-graze death-respawn TIMING (byte-matches… |
| 249 | [?] | P1 | player | 2026-06-04 playtest: touching a wall auto-launches Kid to the top of the level (no C press) — recurring, causes deaths | In the Level-2 well, hitting the LEFT wall makes Kid "jump straight up to the… |
| 250 | [x] | P1 | block | 2026-06-04 playtest: growing/evanescent… | archived |
| 251 | [x] | P1 | level | 2026-06-04 playtest: skull-theme levels… | archived |
| 252 | [?] | P2 | enemy | 2026-06-04 playtest: Level-2 tank renders doubled / "half in the floor" and its death is invisible | Reconfirms #145 (RockTank turret vertical offset) and adds: the tank looks like… |
| 253 | [-] | P3 | gameflow | Highwater Pass 2 flag-transition score/object-count gap is collapsed EOL worker modelling, not a current gameplay bug | Archive #253: Kid/game mode/level-complete match at onset; extra oracle objects are EOL/flag workers and only bonus timing differs. Reopen only if final EOL bonus total is wrong or a non-flag mid-level route reproduces it. |
| 254 | [?] | P1 | intro/render | 2026-06-07 playtest: intro ninja animation has black outline artefacts | Evidence source: human playtest. Symptom: ninja animation in the intro has… |
| 255 | [x] | P1 | menu/input | 2026-06-07 playtest: level select accepts… | archived |
| 256 | [ ] | P1 | render/enemy | 2026-06-07 playtest: third-level Rhino/Berzerker renders in front of foreground tiles | Evidence source: human playtest. Symptom: Rhino/Berzerker on the third level… |
| 257 | [?] | P0 | gameflow/death | 2026-06-07 playtest: dying goes black and only recovers when Start is pressed | Reconfirmed 2026-06-11 play-native: continue overlay eventually appears, but can start off-screen/late after falling to death. The ASM `$1C` hold candidate is insufficient; capture the live death/continue slide-in route before more changes. |
| 258 | [?] | P0 | player/slope | 2026-06-07 playtest: walking left/right/up slopes regressed and Kid/forms get stuck | 2026-06-07 fixed base-Kid Level 00 natural slope route: `verify-tas --scenario… |
| 259 | [?] | P1 | enemy/render | 2026-06-07 playtest: tanks still appear stuck in the floor | Current Level 02 enemy-clean route shows native/oracle Tank placement matched; low appearance is foreground/terrain occlusion. Needs human validation / death visibility handled via #167/#252. |
| 261 | [x] | P1 | block/projectile | 2026-06-07 playtest: fourth-level growing blocks still shoot spikes | archived |
| 262 | [?] | P1 | level/render | 2026-06-07 playtest: levels 5/6/7 tilesets do not match character placement | Evidence source: human playtest. Cross-ref #238/#251 terrain decoder and… |
| 263 | [?] | P1 | player/block | 2026-06-07 playtest: Samurai cannot break blocks below him with jump+sword | Evidence source: human playtest. Cross-ref Red Stealth/Samurai attack and… |
| 264 | [?] | P1 | audio/perf | 2026-06-07 playtest: music slows down when hitting blocks and collecting diamonds | Reconfirmed 2026-06-11 play-native: diamonds/block breaks and Samurai/helmet sounds still appear to slow or play low. Settled waveform verifier passes one block/diamond route, so the remaining bug is live playback/mixer timing or a different route, not the old silent-start verifier miss. |
| 265 | [ ] | P2 | audio/gameflow | 2026-06-07 playtest: EOL countdown sound does not finish | Reopened 2026-06-11 play-native: even after ScoreBoard/StopMusic queue-clear/same-frame suppression candidates, the score-countdown ding still audibly bleeds into the next loading/title-card screen until Start/new-level advance. Needs live audio-events/waveform capture on the exact EOL route. |
| 266 | [?] | P1 | render/level | 2026-06-07 playtest: death-wall level does not render left-side drills | Reconfirmed 2026-06-11 play-native: death wall is still not visible to the human route, despite focused Level 0x08 capture candidates showing fallback/ROM-art drill sprites. Keep open for exact live route capture and timing/layout validation. |
| 267 | [x] | P1 | gameflow/eol | 2026-06-07 playtest: EOL flag only activates when jumped on, should activate on floor contact | 2026-06-10 verified fixed: Highwater floor-contact probe is authoritative-clean through the flag contact window once synthetic probe-camera drift is ignored; later frame-111 object-count drift is separate score/transition verifier debt. |
| 268 | [?] | P1 | form/animation | 2026-06-08 latest-pushed playtest: Iron Knight/Heavy Man wall-climb animation is wrong | Reconfirmed 2026-06-11 in latest pushed playtest: first helmet / Iron Knight animation still looks wrong around wall/climb handling. Existing renderer/core probes have not produced a clean human-equivalent route; `issue260-drips-iron-knight-climb-probe` seeds Iron Knight but never enters `iron_knight_climbing`, so keep this open until a route/frame capture isolates the wrong animation descriptor. |
| 269 | [~] | P1 | form/animation | 2026-06-08 latest-pushed playtest: Samurai held-right into slope glitches animation | Reconfirmed 2026-06-11: Samurai movement/animation still looks slightly strange around the reported route. Existing slope probes cover some CRC-clean cases but not the live visual symptom; next pass needs a captured route with frame crops before changing Red Stealth/Samurai animation state. |
| 270 | [?] | P1 | player/wall | 2026-06-08 latest-pushed playtest: Level 2 well left-wall hug launches Samurai/Kid upward | Fixed candidate: `1c06f5d8` clears the natural Red Stealth BLW2 left-wall frame-266 divergence; route is gameplay CRC-clean through frame 520. Human validation still required. |
| 271 | [x] | P0 | player/enemy | 2026-06-08 latest-pushed playtest: jumping… | archived |
| 272 | [ ] | P1 | audio | 2026-06-08 latest-pushed playtest: Samurai air attack, helmet pickup voice, and bouncy sounds play slow/low | Reconfirmed 2026-06-11: Samurai noises still sound slowed/low. Static `verify-audio-output` passes title/level-entry but fails gameplay-extending routes with native onset/peak drift, so keep dynamic state-gated audio probes for #272. |
| 273 | [?] | P1 | block/prize | 2026-06-08 latest-pushed playtest: later-level helmet prize blocks may not be mapped/extracted | Evidence source: human playtest. Symptom: all helmet prize blocks in later… |
| 274 | [ ] | P1 | demo | 2026-06-09 playtest: attract-mode intro demo — Kid grabs the Cyclops/Eyeclops… | Reconfirmed 2026-06-11 by human playtest, but native trace now proves the published demo exposes Cyclone content before later real Diamond prizes. Next step is visual/frame capture to identify whether the visible report is a later Diamond event or renderer/route mismatch. Cross-ref reopened #284. |
| 275 | [?] | P1 | render/hud | 2026-06-09 playtest: collected diamond does not fly up to the top-right HUD diamond counter | Fixed candidate: native now mirrors ASM `PrizeDiamondCollected_Init` / `Increase_Diamonds` by keeping a collected-diamond HUD-flight gfx worker from pickup screen offset to HUD x=$126/y=$30 before the delayed count award. Human visual validation still required. |
| 278 | [x] | P1 | block/collision | 2026-06-09 playtest: breakable floor blocks (level 3, below the Heavy Man… | Fixed 2026-06-11: floor prize-block impacts now route through ASM `loc_1002E`/`sub_1007A`, keeping the struck cell `$6Fxx` solid during the wobble instead of erasing Kid's support. |
| 279 | [?] | P1 | enemy/block | 2026-06-09 playtest: Rhino/Berzerker cannot destroy the light/breakable blocks it should be able to (level 3) | Fixed candidate 2026-06-11: Berzerker side block dispatch now follows ASM `loc_92DE`/`off_933E` for Prize and Ice blocks, including `Block_LoadSmashBits` and `IceBlock_SmashAndLoadBullets`. Human level-3 route still needs validation. |
| 280 | [?] | P2 | render/background | 2026-06-09 playtest: parallax sky loads in late (level 3 top), stray dark-green… | Reconfirmed 2026-06-11 play-native: death-wall/hell route still shows green background artefacts. The Hills `$08` Plane-B wrap candidate fixes one captured frame-151 sky hole, but broader live-route background/layout parity remains open. |
| 281 | [?] | P1 | block | 2026-06-09 playtest: push-up blocks rise silently when jumped on from above and… | Fixed candidate 2026-06-11: Shifting blocks now activate only from ASM `sub_BB54`/`off_BB80[7]` headbutts, play `sfx_Shifting_block`, reserve `$6F` above, then commit `$E710` and erase the source in `sub_113C0` order. Human route validation still required. |
| 282 | [ ] | P3 | infra/debug | Level-skip (Next) should show the level intro/name card so the level name is visible for debugging | Reopened 2026-06-11 play-native: `N`/`P` still appears to skip straight into gameplay. Latest run also warned debug warp target `0x0D` lacked generated level layout/native terrain decode, so next pass must distinguish missing extracted assets from the live hotkey/title-card path. |
| 283 | [?] | P1 | gameflow/death | 2026-06-09 playtest: death-wall level respawn drops Kid back at the spot he died, not the level/checkpoint start | Fixed candidate 2026-06-11: murder-wall contact now follows ASM `sub_5E02`/`Crushed_to_Death` instead of ordinary HP damage, and Level 0x08 death-wall respawn is covered by `murder_wall_death_respawns_at_level_start_loc_5e34_b8de`. Human death-wall route validation still required. |
| 285 | [x] | P2 | render/enemy | 2026-06-10 playtest: DripsSurfaceCrawler renders as a tiny blob instead of the proper floor-slime crawl animation | Fixed 2026-06-10: native now distinguishes ASM `Drips.asm loc_338F8`'s spawned `Tar_Monster.asm loc_325B6` helper from the original crawler's `loc_3392C` finish shell. The helper still renders the Tar `$0C1` fragment, while the crawler finish renders Drips `stru_33F06` (`$0C5/$0C9/$0D2`) and the existing startup/moving crawler render paths remain `stru_33FA6`/`stru_33F14`. Verified with focused core lifecycle tests, renderer regression, `cargo fmt --check`, and `cargo check -q -p kidcli`. |
| 286 | [~] | P1 | enemy/lifecycle | 2026-06-10 playtest: second stomp on RockTank crushes it visually but the tank stays alive/active | Isolated two-stomp Tank death path passes; old #271 stomp-gate window no longer reproduces a Tank stomp bounce. Recheck 2026-06-11: `tas-diff` on the current #271 sharp RockTank route frames 200..260 is enemy-frontier clean (60 match / 0 diverge), so this still needs a fresh live/render route that actually shows the second-stomp lifecycle bug before changing Tank HP/death logic. |
| 287 | [ ] | P2 | audio | 2026-06-10 playtest: Samurai attack plays several overlapping attack sounds instead of one per swing | Reopened 2026-06-11: latest playtest still hears too many Samurai attack noises. Current grounded repeated-C baseline emits exactly one `$23` per completed swing and passes native-vs-oracle waveform; remaining repro needs the exact live/airborne/target-contact route. |
| 288 | [ ] | P2 | intro/render | 2026-06-10 playtest: intro shows the jacket sprite in the bottom-left before… | Reconfirmed 2026-06-11: intro still shows Kid/jacket below the prize block before the throw. Earlier IntroSequence6 frame windows are pixel-green vs oracle, and cold frames 2220/2280/2340/2400 do not reproduce it, so next evidence must be the exact live route/frame range rather than more changes to the already-green cold checkpoints. |
| 289 | [~] | P1 | level/spawn | 2026-06-10 playtest: hell-theme levels start Kid in the wrong place and the map layout looks wrong | Partial 2026-06-11: Level 0x06 / Under Skull Mountain 3 settled recording repro is fixed for frames 24..40 (`tas-diff` now 16/16 match). Native now ages the direct settled-warp hidden object scheduler through the Dragon Flying path (`Dragon_Flying.asm loc_34710` -> `loc_348A0` -> `loc_34BAC`) and counts the descent-fire projectile as an object-only worker. Broader playtest symptom remains open for other hell-theme wrong-start/graphics cases. |
