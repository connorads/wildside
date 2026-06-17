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
  Left/B+Left` brake axis unchanged. 2026-06-13 follow-up: `kidcli` now accepts
  `--target-cell col,row` for single and batch probe scoring, so #260 route
  search can rank actual 16x16 centre-`EC2D` block-cell contact instead of a
  guessed point. On `issue_260_drips_drop_contact_probe_01`, scoped to
  `--frame-start 520`, the current route remains 32px below left cell `28,32`
  and 157px away from right cell `36,32`; a small baseline/late-B/late-BR batch
  all tied at the same 32px left-cell miss. Do not re-run those unchanged late
  variants as metal-block proof. 2026-06-13 follow-up: `kidcli` also accepts
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
| 284 | [?] | P0 | block/prize | 2026-06-10 playtest: later-level prize blocks award diamonds instead of their… | Reopened 2026-06-11: latest playtest still sees attract/demo prize block award a Diamond instead of Cyclops/Eyeclops/spin form. Published TAS, deterministic cold idle, and generated block-list/live-worker tests expose configured helmets before later real Diamonds. 2026-06-14 refresh: the old dummy-SDL bundle is stale on current HEAD (`verify-recording --slug issue284-live-dummy-attract-4500 --native-only` drifts at frame 3491), and fresh dummy-SDL live captures stopped before demo play (3260-frame `current-3900`; 1749-frame `current-4500`, ending in `IntroSequence4` with no floating items). Current deterministic cold-idle oracle window is still clean: `verify-tas` frames 3448..3495 passes, `tas-diff` reports 47 match / 0 diverge, and refreshed trace `.cache/recordings/issue-284-cold-idle-trace-current-20260614/` shows the first DemoPlay level `0x1B` floating item at frames 3456..3490 as content id `3` (Cyclone), followed by separate real Diamond content id `0` windows from frame 3832 after helmet id 2. 2026-06-14 display refresh with `capture-scenario-frames`/`capture-oracle-scenario-frames` at 3488/3491/3494 confirms native and oracle both visibly draw the purple Cyclone prize at frame 3488; exact AE is `0/419/143`, and a player-area crop narrows frame 3491 to the first transform render while frame 3494's player crop is pixel-perfect. Native/oracle state and camera also match at 3491 (`current_helmet_id=2`, Kid/camera positions equal), and ASM `Kid_Transform loc_7CD0`/`stru_7F30` confirms this is the first Cyclone transform frame, so the remaining delta is transform rendering/timing, not a Diamond prize substitution. 2026-06-14 follow-up re-read ASM `Prize_Init`/`loc_10342` and confirmed the ROM dispatches prize content by exact `Level_Layout` low-word scan, matching native `prize_block_by_layout_addr_loc_10342`; cache-backed `generated_visible_helmet_prizes_spawn_configured_content_from_live_worker` and `generated_level20_reported_prize_clusters_use_layout_addr_not_cluster_base_loc_10342` both pass with `KID_ENABLE_CACHE_BACKED_TESTS=1`. Fixed candidate 2026-06-15: manual/live preflight now treats `<level>.blocklist.bin` as required alongside `<level>.bin` and `<level>.json`, so old partial generated caches cannot enter later levels with terrain present but no prize list and fall back to Diamond. 2026-06-15 latest-playtest follow-up added the exact Level `0x1B` left-ledge Cyclone prize `(col=4,row=36,layout=$C81A)` to the cache-backed live-worker audit; it passes through `start_prize_block_wobble_sub_1007A` and spawns content id `3`, not Diamond. 2026-06-15 retest on local generated cache reconfirmed both cache-backed audits: `KID_ENABLE_CACHE_BACKED_TESTS=1 cargo test -q -p kid-core generated_visible_helmet_prizes_spawn_configured_content_from_live_worker -- --nocapture` and `KID_ENABLE_CACHE_BACKED_TESTS=1 cargo test -q -p kid-core generated_level20_reported_prize_clusters_use_layout_addr_not_cluster_base_loc_10342 -- --nocapture` pass. Fixed candidate 2026-06-15: generated/live runtime loading is now all-or-nothing for `Level_Layout` plus `<level>.blocklist.bin`, so a partial cache cannot keep terrain loaded while sending `Prize_Init loc_10342` through the fixture-only Diamond fallback. Re-test the reported later Cyclone route from a freshly launched build; if it still emits content id `0`, capture the exact live route/frame because current HEAD should no longer run generated levels with missing prize metadata. |
| 290 | [?] | P1 | trace/objects | Full any-percent TAS diverges at checkpoint frame_7800 (demo_play): native objects=9/4 vs oracle 10/9 | Fixed candidate: Dragon.asm loc_3CE90/loc_3CF90 breath child object-only count; 2026-06-13 full published TAS passes all 37759 frames by CRC. 2026-06-14 state classifier now treats frames 7840..7866 `number_objects` as Dragon.asm loc_3D086 breath-child sampling oracle limitation when native has the projectile and oracle still exposes the child Object_RAM slot; no authoritative divergence remains in that window. Keep open for human-validation policy on full report wording. |
| 260 | [~] | P0 | block/enemy | 2026-06-07 playtest: Level-2 slime/metal-block interaction breaks block and emits red projectile left | Reconfirmed 2026-06-11: Drips now creates a crawler. The current Level 01 Drips source visually being a `P` block is oracle-matching on the known route (native/oracle frame 535 both show `P` blocks), so do not patch that visual without a different human route. 2026-06-14 fixed sub-case: native no longer rewrites BLW2 live cell `0xCE8A` `$EC2D` into `$E102`, so `.cache/debug/issue260-brakehop-probe/oracle-frame-scenario.json` now verifies with no authoritative divergences through frames `0..700`; this removes the false centre-cell Diamond branch only. Existing route probes still need a route/fresh repro reaching the reported live slime/metal-block contact. Current `478..528 Left`, b-jump climb-tail, b-jump second-crawler window, early-arc rebuild, early-jump extension, simple C/A/Up/Down perturbations, `450..479 Right,480..535 B+Right`, and the `480..540 B` lift overlay are exhausted; best clean route is still a Kid/block-line near-miss (`dx=0 dy=-1 @490` against row-32 cells), with the closest DripsSurfaceCrawler still `dx=+102 dy=+1 @512`. 2026-06-14 follow-up on `.cache/debug/issue260-tail-grid/l524_b530_540.json`: batch probes adding `left+b` over `390..510` keep the Kid/row-32 near-miss exact (`dx=0 dy=-1`) but only improve the crawler-to-cell distance to `dx=-29 dy=-65 @558` for cell `33,32`; no live slime/metal-block contact, and this left+jump window is another route dead end. 2026-06-14 late route-batch follow-up rejects the brakehop pre-drop (`pd1..pd10`) and p11 pre-300 rebuild (`rb1..rb10`) axes: no row-32 destroy, crawler/drop remain separated from centre `EC2D`; see archive before retrying. 2026-06-15 pre-380 coast/late-B grids from the DLB row-32 bounce base are also rejected: best `c300_360_b370` still has Kid `dy=-33` and no row-32 `Bounce`/`Destroy`. 2026-06-15 post-570 `issue260_gr_pulse600` tail batch rejected the right/right+jump correction around the closest drop window: best `r610_650`/`br610_650` only bounce cell `37,32 @610`, with no DripsDrop cell conjunction or destroy, while overlays that preserve the drop at cell `36,32 @640` put Kid over 100 px away. 2026-06-16 recheck: corrected DLB baseline is enemy-frontier clean through `450..530`, has `Bounce cell 37,32 @511`, no `Destroy`, and `lower_170_300` is rejected because its better drop/cell score loses the bounce and kills Kid. 2026-06-16 corrected-overlay batch on `.cache/recordings/issue-260-gr-pulse600-current/scenario.json` clears before applying early/mid route changes: `clear300_lower_route` aligns healthy Kid and `DripsDrop` vertically at row 32 but remains `dx=-28 @510` with no block event, while `c300_r300_430_b500` improves Kid to `cell 37,32 dx=0 dy=-1` and `Bounce cell 37,32 @610` but leaves the closest `DripsDrop` `dy=-9..-12` earlier and the crawler `dy=-65`; no candidate produced row-32 `Destroy`. The new `--target-projectile-direction shooter-block:left` probe also reports no leftward shooter shot and no ShooterBlock near cell `36,32` on `issue260_gr_pulse600`. 2026-06-16 ASM check: `sub_F096` only fires visible `$C00` shooter slots when `loc_F248` marks the direction word negative because the matching neighbour skin byte is non-zero; raw `$EC2D` has direction mask zero, and unsupported `$EC2F`/synthetic slots now register without spawning red shots. 2026-06-16 late-pulse/phase follow-up after visible-slot bullet removal rejects `.cache/recordings/issue-260-gr-pulse600-pulse-left600/scenario.json`: native reaches `cell 37,32 dx=0 dy=-1 @610` and bounces it, but DripsDrop remains vertically separated (`kid_dy=-33`, `drop_dy=-12`, manhattan 60 @570), no Destroy occurs, and enemy-frontier oracle bisects to HP divergence at frame 616. 2026-06-16 corrected hypothesis: raw `$EC2D` is not itself a viable leftward-shot source; `loc_F1DE` queues zero direction bits and `loc_F28C` leaves that slot non-visible/non-projectile. Next search needs a different live bootstrap that actually produces the reported red-projectile/type-C interaction, or a fresh live recording, not another local Drips row-32 overlay or raw-`$EC2D` left-shot route. |
| 277 | [?] | P1 | player/input | 2026-06-09 playtest: holding left into a wall then jumping drops the left input… | Fixed candidates 2026-06-13: gameplay input now preserves held-left/held-right on delayed jump edges, and `de869f33` mirrors ASM `sub_902A -> loc_A984 -> loc_A99E` for leftward leading-edge SlopeDown recontact instead of blocking it behind the row-boundary guard. Regressions `gameplay_input_preserves_held_left_on_delayed_jump_edge_sub_a432_sub_b084` and `level00_airborne_leading_edge_slope_down_left_recontact_enters_loc_9d22_same_frame` cover the two confirmed native gaps. Verified with focused tests and `mise run verify-slice-fast`. Human validation still needed on the exact live stair/wall jump route. |
| 276 | [x] | P1 | hud | NOT A BUG: Iron Knight's first hit appears `4 -> 4` because ASM `sub_BDC0` hides the odd fifth HP slot; current native capture and oracle state parity confirm the first-hit display is ROM-faithful | Current capture `.cache/debug/issue276-current-native/` frames 0..3 shows the reported first-hit pips; `tas-diff` on `.cache/debug/issue276-oracle/oracle-frame-scenario.json` frames 0..5 is 5/5 match; focused HUD tests cover one-frame latch and later visible decrement |
| 292 | [?] | P1 | level/timer | Fixed candidate: native timer now applies `Init_SpriteAttr_HUD`'s +1 second seed and keeps `loc_C0CC` completion-time math on the ROM `$B4` base; clock prizes also apply the separate `loc_FF92` +1s carry before `loc_FFDE` +3 minutes/cap | Verify with playtest: fresh level should show the ROM-normalised 3:00 after first tick, timer death no longer ~60 frames early, and clock prizes net a clean +3:00 after the compensating tick |
| 293 | [?] | P1 | menu/shell | Fixed candidate 2026-06-11: live pause now runs ASM `Game_Paused_Loop`/`Game_Paused_ChkStart` cursor and Start branches instead of a boolean-only toggle; Restart Round spends a life, restarts the level, clears helmet/extra HP, and restores Kid HP; last-life Give Up enters the native dying path via `loc_6C0A` semantics | Focused runtime and pure pause-menu tests pass. Human live validation still required: Pause, Down, Start with >1 life should spend one life and restart; on last life should play the bummer/dying/game-over path. |
| 294 | [x] | P1 | combat | Murder-wall contact now follows ASM `sub_5E02 loc_5E34 -> off_B640 -> Crushed_to_Death`: contact enters DeathWindup directly, bypasses HP decrement, and ignores damage invulnerability | Verified by `murder_wall_contact_crushes_helmeted_kid_loc_5e34_b640`, `murder_wall_crush_bypasses_damage_invulnerability_loc_5e34_b640`, Level 0x08 respawn regression, and `cargo test -q -p kid-core murder_wall -- --nocapture` |
| 295 | [x] | P1 | terrain | Spike routing now follows ASM `loc_91A6`/`loc_92B6`: floor, ceiling, and side-wall spike contacts latch spike damage and route through normal `Check_for_recent_damge -> loc_B700` HP/helmet/invulnerability handling instead of plain ceiling or instant death | Verified by floor spike damage, ceiling spike damage, side-wall spike classification, and `loc_91A6` regression groups: `cargo test -q -p kid-core spike_contact -- --nocapture`, `cargo test -q -p kid-core ceiling_spike -- --nocapture`, `cargo test -q -p kid-core loc_91a6 -- --nocapture`, `cargo test -q -p kid-core loc_92b6_side_wall_spike_sets_on_spikes_and_clamps_x -- --nocapture` |
| 296 | [x] | P1 | projectile | Maniaxe axes now follow ASM `loc_8ABC`: live lifetime-zero axe projectiles still deliver `colid_hurt` to enemies and are deleted on enemy contact | Verified by `live_maniaxe_axe_lifetime_zero_damages_and_deletes_loc_8abc` and the Maniaxe axe regression group: `cargo test -q -p kid-core maniaxe_axe -- --nocapture` |
| 297 | [x] | P1 | block | Ice-block smash paths now route through ASM `IceBlock_SmashAndLoadBullets`: `off_BB80[2]` ceiling headbutts delete the ice cell and spawn ice bullets, and Red Stealth `off_B058[2]` floor impacts load the red bullet pair before rebound | Verified by `cargo test -q -p kid-core ceiling_ice -- --nocapture` and `cargo test -q -p kid-core red_stealth_ice -- --nocapture` |
| 298 | [?] | P1 | block | Fixed candidate: telepad warp now requires `sub_B43A loc_B522`'s `$5A` grounded countdown; first-contact landing and raw feet-position lookups no longer bypass the timer | Reverified 2026-06-13 with cache-enabled generated assets: BLW2 telepad cell `(156,36)` resolves through the generated block-list entry and warps to tile `(28,9)` after the 90-frame ROM countdown, with `sfx_Teleport` queued. Human playtest: stand on telepad, warp only at ~frame 90; quick walk-over should not warp. |
| 299 | [?] | P1 | level | Fixed candidate: ending trigger now follows ASM `loc_D7F4`'s direct `Current_LevelID == Final_LevelID ($33)` comparison instead of treating the last MapOrder entry as final | Human/playtest: completing logical `$33` enters ending; any later MapOrder tail level continues through normal Teleport/title-card flow |
| 300 | [?] | P1 | level | Fixed candidate: teleporter destination now follows ASM `loc_BCD6` by reverse-scanning MapOrder and storing the logical level index, not the raw map byte | Human/playtest: inter-level teleporter where map id differs from logical index should load the ROM destination; same-level teleport still repositions only |
| 301 | [ ] | P1 | enemy/boss | Umbrella: bosses-fidelity-audit.md — Shishkaboss HP, boss-level flag suppression, HeadyMetal `loc_37582` velocity/hover, HeadyMetal `$FFFFFB4F` freeze branch/count for live `sub_3729E` workers, `sub_37332` eye-launch physics, HeadyMetal `sub_37708` phase `$19..=$24` pop/attack spawn, `sub_379E8` rendered palette pulse, boss-head `sfx_Boss_dies` at the `loc_39EE2` pre-spin boundary, live multi-head fire Y-window, live `sub_39846` fixed boss-head movement, live `sub_3A292` collision/HP/flash/groan, live `sub_397E4` boss-head projectile spawn, live `stru_39FC6` wind-up/`$18(a3)` projectile gate, live activation-facing latch equivalent to `$FFFFFB86/87/88`, live post-projectile `stru_39FD8` gate, lethal wait-loop `loc_39EBE` status exits, `loc_3A0D2` arrow move-then-stick lifecycle, `loc_3A1EA`/`loc_3A24C` boomerang acceleration/countdown lifecycle, final HeadyMetal `sub_378C8` traced split-piece rendering, live-breakup `sub_37ABC` body-piece refresh, live-load `Enemy20_HeadyMetal_Init loc_39247` worker-table persistence, live `$FB72` `sub_3787C/sub_378A2` final-scatter timer/x-velocity, and live `sub_393C8/sub_39420` boss-eye satellite trace/render sub-gaps fixed/partial; remaining high gaps include boss-head death visuals and HeadyMetal full live composite worker lifecycle before breakup | Per-gap rows in bosses-fidelity-audit.md; continue with one coherent boss-engine slice at a time |
| 302 | [?] | P2 | block/prize | Fixed candidate: disguised `$E101` prize blocks now take the ROM reveal-only first hit (`loc_101D8` writes `$E102`; prize needs a second hit) | Headbutt `$E101`: first hit converts to visible `$E102` with no prize; second hit pays |
| 303 | [x] | P2 | menu | Fixed: CONTINUE? spends a continue only on Start and keeps the ROM `loc_D3AA` second Start window / 239-frame timeout tail | Verified by `loc_d350_`, `loc_d3aa_`, `use_continue_clears_extra_hitpoint_slots_loc_d3c4`, and `continue_countdown` focused tests |
| 304 | [x] | P2 | player/input | Fixed: live input now applies `Permute_ABCButtons` before the held/pressed latch and Fast Action now inverts `Ctrl_A_Held` in gameplay input, preserving the demo-mode skip | Verified with `cargo test -q -p kid-core input_ -- --nocapture` and `mise run verify-slice-fast` |
| 305 | [x] | P2 | verify | Add interactive-lane oracle scenarios for demo-skipped paths (options remap, Fast Action, pause menu): ROM disables these in demo mode so existing TAS/demo traces structurally cannot cover them; fresh DynamicScenarios navigating title-options-gameplay drive identical input paths in native and oracle | Fast Action lane is checked in at `tests/data/dynamic-scenarios/issue-305-fast-action-gameplay-probe.json`: title Down/Down at 500/530, Start Options at 600, options Down/Down/Right at 820/850/880 sets `Options_Suboption_Speed=$FF`, Down+Start at 1200/1230 returns to title, Start at 1300 enters level card, and title-card Start at 2600 enters gameplay. Verified with `verify-tas --frame-start 2600 --frame-end 2805 --checkpoint-interval 1 --frontier-kind gameplay --bisect` (all 2805 frames CRC-match) and advisory `tas-diff` 2600..2805 (205 match / 0 diverge); promoted to `tests/tas_frontier.json` 2026-06-15. Options-remap lane is checked in at `tests/data/dynamic-scenarios/issue-305-options-remap-a-jump-probe.json`: after entering Options it moves to Controls at 820, presses Right at 850/880 to select scheme 2 (`Permute_ABCButtons` A<->B), exits through Speed/Exit at 910/940/970, starts gameplay through the proven title-card timings, and presses raw A at 2640..2646. Verified with `verify-tas --frame-start 2600 --frame-end 2695 --checkpoint-interval 1 --frontier-kind gameplay --bisect` (all 2695 frames CRC-match) and advisory `tas-diff` 2638..2655 (17 match / 0 diverge); do not extend this lane past 2695 without addressing the known camera-Y drift that starts at 2696; promoted to `tests/tas_frontier.json` 2026-06-15. Pause Restart Round life-spend slice is checked in at `tests/data/dynamic-scenarios/issue-305-pause-restart-round-life-spend-probe.json`: after proven title-card timings, Start pauses at 2801/2802, Down selects `Pause_Option=1` at 2805, and Start on Restart Round spends one life by 2813. Verified with `verify-tas --frame-start 2800 --frame-end 2814 --checkpoint-interval 1 --frontier-kind gameplay --bisect` (all 2814 frames CRC-match); promoted to `tests/tas_frontier.json` 2026-06-15 after the TAS verifier classified the residual CRC-clean `game_paused`/`character_movement` latch skew as oracle export limitation when all gameplay-critical state matches. Fresh oracle WRAM watch overturned the earlier blocker: oracle does consume the paused-loop Start and drops lives 3->2; the remaining pause lane work is the later reload-shell timing mismatch, where `tas-diff` 2836..2848 shows native already unpaused/running gameplay while oracle is still in `Game_Paused_Loop`/`loc_6E2` restore, first CRC drift at 2841. Last-life Give Up lane is checked in at `tests/data/dynamic-scenarios/issue-305-pause-give-up-last-life-probe.json`: `initial_lives=1`, Start pauses, Down selects Give Up, and Start follows the ROM two-frame `loc_6B62` -> `loc_6C0A` sequence. Verified 2026-06-15 with `verify-tas --frame-start 0 --frame-end 40 --checkpoint-interval 1 --frontier-kind gameplay --bisect` (all 40 frames CRC-match); promoted to `tests/tas_frontier.json` 2026-06-15 after the TAS verifier classified the residual CRC-clean `character_movement` latch skew as oracle export limitation when all gameplay-critical state matches. |
| 306 | [ ] | P2 | platform | Umbrella: platforms-fidelity-audit.md — 32 of 40 gaps still open after the 2026-06-10 staleness check (least-worked May audit) | Per-gap rows in doc; verdicts in fidelity-audit-staleness-2026-06-10.md. 2026-06-15 fixed candidate: airborne `Platforms_CheckCollision` no longer lands via the native 6px terrain snap fallback after ASM `loc_25A0` AABB rejection; covered by `platform_collision_requires_aabb_overlap_not_snap_tolerance_loc_25a0` and `platform_collision_airborne_path_ignores_snap_tolerance_loc_25a0`. 2026-06-15 fixed candidate: lift-block headbutts now run the ROM `sub_FB3E` pre-spawn bob before allocating the `sub_FCA6` lift platform and elevator sound, covered by `process_block_effects_lift_spawns_platform` and `lift_block_headbutt_delays_platform_until_sub_fb3e_finishes`. 2026-06-16 follow-up: live `PlatformLoaderState` now carries the `Initialize_Platforms/sub_231C` slot model and runtime platform rosters follow `Addr_FirstPlatformSlot` reuse order, covered by `platform_loader_state_snapshots_follow_reused_slot_chain_loc_257e` and `runtime_platform_roster_follows_loader_slot_chain_loc_257e`. |
| 307 | [ ] | P2 | level | Umbrella: level-loading-fidelity-audit.md — 34 of 55 still open (+11 partial) after the staleness check | Per-gap rows in doc; staleness report has current file:line evidence. 2026-06-15 fixed candidate: live block debris now uses the ROM six-slot `sub_28FC`/`sub_292E` scratch cursor and overwrites slot 0 on the seventh same-frame debris spawn, covered by `live_block_debris_uses_six_slot_scratch_ring_sub_292e`. 2026-06-15 fixed candidate: live `sub_2A4C` debris cull now kills the scratch slot on tail-fragment relative x `$1EF` while preserving the camera-relative lead-fragment cull, covered by `live_block_debris_tail_fragment_culls_slot_sub_2a4c` and `highwater_block_debris_culls_against_camera_not_world_origin_sub_2a4c`; renderer `$8228` coverage remains in the focused `kidcli` block-debris tests. 2026-06-15 fixed candidate: `begin_level_entry` now records the ASM `loc_11FD2` level-start BGM id in core `played_bgm_ids`, including the murder-wall override before foreground-theme lookup, and native-player now prefers that core-queued id when present; covered by `begin_level_entry_queues_level_bgm_from_loc_11fd2`, `begin_level_entry_queues_murderwall_bgm_before_theme_lookup_loc_11fd2`, `play_native_level_bgm_prefers_core_queued_loc_11fd2_id`, `loc_11fd2`, and `kidcli bgm` tests. 2026-06-15 partial fixed candidate: live level entry now spawns and ticks the Island `loc_1202A` shore-animation object state when `loc_11FB0` is reached, while respecting the Cave/lava foreground branch at `loc_11F96`; covered by `begin_level_entry_spawns_island_shore_cycle_loc_11fb0`, `begin_level_entry_cave_foreground_skips_island_shore_cycle_loc_11f96`, and `step_island_shore_cycle_ticks_loc_12098_12110_12188`. 2026-06-15 partial fixed candidate: live level entry now also spawns/ticks the lava/water `RotatePalette_Lava_Water` countdown state for `loc_11F86` Mountain and `loc_11F96` Cave and mutates runtime `Palette_Buffer+$E..+$14`, covered by `begin_level_entry_spawns_lava_water_rotator_for_cave_loc_11f96_1223a`, `begin_level_entry_spawns_lava_water_rotator_for_mountain_loc_11f86_1223a`, `step_lava_water_rotator_reloads_after_loc_1223c_rotation`, `step_lava_water_rotator_mutates_live_palette_buffer_loc_1223c`, `trace_exposes_lava_water_palette_words_loc_1223c`, and `highwater_foreground_palette_rotates_lava_water_cram_words_loc_1223c`. 2026-06-16 partial fixed candidate: live level entry and debug-skip hold now seed generated theme FG/BG palette words into `Palette_Buffer` with ASM `loc_11DB0`/`loc_11E16` offsets, and the lava/water rotate start was corrected from byte-offset-as-index `$0E` to word index `$07`; covered by `begin_level_entry_seeds_theme_palette_buffer_loc_11db0_11e16`. Island shore DMA is now traced via `island_shore_dma_uploads_loc_1202a` and `island_shore_vram_frames_loc_1202a`; renderer consumption of that stream and hardcoded override palette word data remain open. 2026-06-16 rejection: `debug_hotkey_cave_back_to_blue_lake_live_background_uses_target_visuals_loc_6e2` proves the exact cave -> Blue Lake Woods hotkey title-card + live handoff uses target background samples, not stale cave visuals; `debug_hotkey_cave_to_highwater_live_background_uses_target_visuals_loc_6e2` also proves the exact cave -> Highwater hotkey handoff reaches live gameplay with the Highwater render bundle and nonblack target background samples. The reported cave-eyes/black background needs a different live skip route or frame. |
| 308 | [ ] | P2 | combat | Umbrella: combat-fidelity-audit.md — murder-wall crush, Berzerker `loc_B692`, death/hurt SFX, Skycutter FA6A stomp bounce, projectile i-frame consumption, projectile top-contact bounce routing, hostile-projectile FA75 latch, stomp bit14 contact damage, FireDemon-walking/Diamond-walking/Robot/Armadillo/TarMonster-live/Sphere/Archer-body-zero/DragonFlying-parent/Cloud-parent-live/Fireball-main/EmoRock-live/Hand-body-zero/Orca-active/Crab/UFO/Tornado/Goat/Ninja-active/Lion-parent/Scorpion-walking/SpinningTwins-live/Driller-parent-drills/BigHoppingSkull-live contact boxes, MiniHoppingSkull scream contact, PitFall death arc ordering, and `loc_1F08` pass-0 hit sparks fixed; remaining gaps include more per-sprite-frame `CollisionSize_Index` hitboxes vs invented fixed 8x16 contact box, generic colid word delivery, and remaining projectile/contact routing edge cases | Per-gap rows in doc |
| 309 | [ ] | P2 | block | Umbrella: blocks-fidelity-audit.md — 27 gaps (#297/#298/#302 promoted separately) | Per-gap rows in doc |
| 310 | [ ] | P2 | terrain | Umbrella: terrain-fidelity-audit.md — 18 gaps (#295 promoted separately) | Per-gap rows in doc |
| 311 | [x] | P2 | projectile | Umbrella: projectiles-fidelity-audit.md — fixed after Lion, Maniaxe, skull, burst-gravity, Lion owner-token, Red Stealth airborne hitbox, and EyeClops hidden-block `$12C` foreground update worker fixes | Per-gap rows in doc |
| 312 | [ ] | P2 | level | Umbrella: level-flow-fidelity-audit.md — remaining gaps incl. final-level routing, teleporter MapOrder translation, and timer semantics (#299/#300/#321 promoted separately) | Per-gap rows in doc |
| 313 | [ ] | P2 | menu/shell | Umbrella: shell-fidelity-audit.md — remaining gaps (#292/#293/#303/#304 promoted separately); `Allow_Pausing` is a fixed candidate, pause-render/game-over hold details remain open | Per-gap rows in doc |
| 314 | [ ] | P2 | score | Umbrella: scoring-economy-fidelity-audit.md — remaining gaps incl. one-time-prize anti-farming unwired (1-ups re-farmable) plus score/life edge cases; Hundred-K-Trip timer/teleport and life 9-vs-99 cap are fixed candidates | Per-gap rows in doc |
| 315 | [ ] | P2 | enemy | Umbrella: enemy-fidelity-audit.md — 25 of 67 still open plus 9 partial remainders (e.g. Big Hopping Skull patrol bounds, Ninja knockdown branch) after the staleness check | Staleness report lists each remainder with current file:line |
| 316 | [ ] | P2 | form | Umbrella: helmets-fidelity-audit.md — 25 of 38 still open after the staleness check | Per-gap rows in doc; staleness report verdicts |
| 317 | [ ] | P3 | weather | Umbrella: weather-fidelity-audit.md — weather gaps still open after the staleness check | Per-gap rows in doc. 2026-06-15 fixed candidate: live lava geysers no longer use the invented 4px ramp or raw 30-frame hold; `step_lava_geysers` now follows ASM `loc_1484E`/`loc_1496C`/`loc_1498C` row timing and init converts encoded row counts to pixel heights. 2026-06-15 partial fixed candidate: level entry now spawns/ticks Island shore and lava/water rotator object state, and lava/water mutates runtime `Palette_Buffer+$E..+$14`. 2026-06-16 fixed candidates: live lava weather now seeds `Pal_7B8AC` into `Palette_Buffer+$20`, ticks `Animation_Geyser loc_14B2A`, rotates runtime `Palette_Buffer+$28..+$2E`, native-player loc_14BE8 sprites consume those live trace words, traces the `ArtComp_983D2_Lava` -> `$5F60` upload, and stores the storm saved-palette pointer `Pal_7B85C`; remaining script-walk, storm art/pattern/fade consumption, shore-frame-source, and VRAM-backend gaps stay open. |
| 318 | [ ] | P3 | player | Umbrella: movement-fidelity-audit.md — 8 of 25 still open after the staleness check | Per-gap rows in doc |
| 319 | [ ] | P3 | audio | Umbrella: audio-triggers-fidelity-audit.md — remaining low/stub trigger gaps after Ankh/evanescent/ice-block/no-time and later enemy/BGM follow-ups; timeout `$51`/`$60` path is already live-tested | Per-gap rows in doc; tick off as wired |
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
| 177 | [x] | P1 | enemy | Diamond modelled as a Collectible pickup, not a hostile HP enemy (stub) | Fixed 2026-06-14: live Diamond layout objects now stay in the enemy path instead of prize collection. Rendering follows `Enemy01_Diamond_Init` / `stru_3ACA8`; overlap damages Kid through Character_CheckCollision; charging Berzerker side contact follows `loc_3AF24` and launches Diamond upward without HP damage; generic `colid_hurt` from Maniaxe/projectile routes now decrements Diamond HP and enters the Diamond death tail at `loc_3AEE6`. Covered by `diamond_enemy_animates_while_stationary_stru_3aca8`, `diamond_enemy_is_not_collected_on_touch_loc_3ad16`, `berzerker_charge_launches_diamond_instead_of_damaging_loc_3af24`, `maniaxe_axe_colid_hurt_damages_diamond_loc_3aec4`, and `maniaxe_axe_colid_hurt_starts_diamond_death_tail_loc_3aee6`. |
| 178 | [x] | P1 | enemy | Armadillo: damage recoil window, Berzerker curl-launch, child fall-then-die, walk-speed cap not wired | Fixed 2026-06-14: charging Berzerker side contact with no positive `$42` launches through `loc_3C898/loc_3C8CC`; `colid_hurt` honours `loc_3C848` recoil immunity, seeds `$42=$FF88`, and restores capped walk speed; `$5C(a5)` spawned children now route through `loc_3C93C/loc_3C954` fall-settle before `loc_3C886` death animation. Covered by `berzerker_charge_launches_armadillo_instead_of_damaging_loc_3c898`, `maniaxe_axe_colid_hurt_respects_armadillo_recoil_window_loc_3c848`, `maniaxe_axe_colid_hurt_seeds_armadillo_recoil_loc_3c848`, `armadillo_spawned_child_terminal_damage_falls_before_death_loc_3c87e`, `armadillo_spawned_child_floor_contact_starts_death_tail_loc_3c954`, and `armadillo_hp_and_walk_speed_follow_loc_3c756_and_loc_3c766`. |
| 179 | [x] | P1 | enemy | Big_Hopping_Skull: patrol corridor, relaunch, floor-stop, per-hit SFX, respawn marker not wired | Fixed 2026-06-14: live flat-floor contact follows `loc_36580` by saving d0=x_vel, clearing motion through `stru_367AE`/`stru_367B4`, then relaunching via `loc_366D0`; relaunch now applies segment patrol bounds from `loc_364A0`/`loc_36770`; nonfatal damage hibernates through `stru_367CC` and queues `sfx_Big_Hopping_Skull_groan`/`sfx_Voice_die` at `loc_365F2`/`loc_36616`; terminal `colid_hurt` queues `sfx_Big_Hopping_Skull_dies=$26` at `loc_3664E`; finished death animation writes the positive `$2168` respawn marker at `loc_366A4`. Covered by `big_hopping_skull_live_floor_contact_stops_like_loc_36580`, `big_hopping_skull_live_floor_contact_timer_relaunches_like_loc_366d0`, `big_hopping_skull_live_relaunch_applies_patrol_bounds_like_loc_36770`, `big_hopping_skull_nonfatal_damage_enters_stru_367cc_hibernate`, `big_hopping_skull_hit_animation_queues_stru_367cc_sounds_like_loc_365f2`, `maniaxe_axe_colid_hurt_queues_big_hopping_skull_death_sfx_loc_3664e`, and `big_hopping_skull_death_waits_for_stru_367fa_animation_before_delete`. |
| 180 | [x] | P1 | enemy | Crab: directional front-immune vulnerability… | archived |
| 181 | [x] | P1 | enemy | Driller: probe offset wrong, drills every… | archived |
| 182 | [x] | P1 | enemy | Emo_Rock: invented ground-walk, no disappear/respawn re-attack, Berzerker launch + death routing missing | Fixed 2026-06-14: death now queues Prize_block and applies the `$2168` status tail after `stru_36468`; landed `stru_363FE` no longer enters generic ground-walk, queues `sfx_Emo_Rock_disappears=$25`, hibernates through 16-frame `stru_36456`, then restarts `loc_361F4` hover timing; grounded pending collisions route through `loc_362B6` so hurt/above enter `loc_36346`, charging Berzerker side contacts launch through `loc_362EE`, and ordinary side contacts keep waiting. Covered by `emo_rock_lands_and_enters_settled_wait`, `emo_rock_settled_wait_does_not_walk_like_pacer_loc_362b6`, `emo_rock_settled_wait_disappears_and_restarts_hover_loc_36320`, `emo_rock_death_waits_for_stru_36468_animation_before_delete`, `emo_rock_settled_side_contact_waits_when_berzerker_is_not_charging_loc_362b6`, `emo_rock_settled_side_contact_launches_for_charging_berzerker_loc_362ee`, and `emo_rock_settled_hurt_collision_enters_break_death_loc_362b6_36346`. |
| 183 | [x] | P1 | enemy | Fire_Demon: walking fire-trail emission, Berzerker launch, hurt recovery anim not driven | Fixed 2026-06-14: pending collision routing follows ASM `loc_3A586`: nonfatal `colid_hurt` decrements HP and hibernates through `loc_3A5BE`, terminal hurt enters `loc_3A5A6`, ordinary side contact resumes `loc_3A5D4`, and charging Berzerker side contact enters `loc_3A5E4`/`loc_3A610` without HP damage. Walking `loc_3A542` now allocates typed `loc_3A63C` trails from the saved walk cadence, uses raw level 2's three-frame `stru_3A334`, respects the seven-trail cap at `loc_3A564`, and applies the immediate support bailout before the child is observable. Covered by `fire_demon_*` focused tests including pending collision, walking cadence, trail allocation, cap, and unsupported-floor cases. |
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
| 218 | [x] | P1 | player | FIXED 2026-06-13: Ground speed tables inverted: ice plays grippy, rubber plays slippery (ROM cross-wire undone) | Commit "Cross-wire ice rubber ground speed tables": `PlayerForm::ground_speed_table` now preserves ASM `sub_8600` + `loc_9AE0`/`loc_9AF2` cross-wire centrally, and the old movement-local rubber hand-off was removed. Regression coverage: `speed_table_`, `level00_frame_731_flat_rubber_span_uses_refreshed_code_two_table_loc_8e16`, `ground_speed_table_selects_by_form_and_terrain`, and `mise run verify-slice-fast`. |
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
| 256 | [ ] | P1 | render/enemy | 2026-06-07 playtest: third-level Rhino/Berzerker renders in front of foreground tiles | Evidence source: human playtest. Current Highwater audit route plus B+Right, C/special-action, and jump/reset overlays all still stop at the wall about 75 px left of the waterfall-front target. 2026-06-15 bounded recheck over the current route reconfirmed the same cap: best healthy candidates still reach only `(840,1548)` / `dx=+75` from target `(915,1548)` (`br_680_1160`, `down_reset`, `jump_pulse`). The best C-family candidate also has a transient native/oracle wall-cling divergence at frames 832..833, so it is not clean visual-priority evidence. 2026-06-15 follow-up regenerated `.cache/recordings/issue-256-highwater-water-priority-current/` and reran the B/C/right route family; the best current route is still `760..1240 B+Right` (same result for `B+C+Right`) at `(840,1548)`, full health. Focused trace frames 1268..1290 show native is wall-clinging against solid column `x=848`, cells `53,89..99` (`kid_wall_cling_release_probe branch=sub_ADB8/sub_ADB8=forward_wall_blocked`), and native/oracle captures at `.cache/debug/issue256-current-best/{native,oracle}/` frame 1284 both show Berzerker left of the waterfall foreground, not inside the reported occlusion window. Do not patch sprite priority from this route; it is a valid route-search rejection only. 2026-06-14 rejected synthetic waterfall probe at `(915,1548)`/`(915,1535)`: native shows the intended foreground window but immediately climbs through stacked solid cells and hits spikes while oracle holds the patched object/camera, so it is not valid priority evidence. Renderer invariant now covered by `player_sprite_priority_stays_low_except_flagpole_override_loc_c070`; ordinary player/Berzerker frames stay low priority and only ASM `CharacterCollision_TouchFlagpole loc_C070` forces first-frame high priority. `priority_foreground_terrain_masks_later_low_priority_sprites_loc_3034a` now covers actual `DrawCommand::Sprite` rejection, and ignored cache-backed `highwater_priority_foreground_masks_low_priority_sprites_loc_3034a` proves generated Highwater priority cells can mask low-priority sprites. Still needs the actual live occlusion window or a different route seed into the foreground mask. |
| 257 | [?] | P0 | gameflow/death | 2026-06-07 playtest: dying goes black and only recovers when Start is pressed | Reconfirmed 2026-06-11 play-native: continue overlay eventually appears, but can start off-screen/late after falling to death. The ASM `$1C` hold candidate is insufficient; capture the live death/continue slide-in route before more changes. |
| 258 | [?] | P0 | player/slope | 2026-06-07 playtest: walking left/right/up slopes regressed and Kid/forms get stuck | 2026-06-07 fixed base-Kid Level 00 natural slope route: `verify-tas --scenario… |
| 259 | [?] | P1 | enemy/render | 2026-06-07 playtest: tanks still appear stuck in the floor | Current Level 02 enemy-clean route shows native/oracle Tank placement matched; low appearance is foreground/terrain occlusion. Needs human validation / death visibility handled via #167/#252. |
| 261 | [x] | P1 | block/projectile | 2026-06-07 playtest: fourth-level growing blocks still shoot spikes | archived |
| 262 | [?] | P1 | level/render | 2026-06-07 playtest: levels 5/6/7 tilesets do not match character placement | Evidence source: human playtest. Cross-ref #238/#251 terrain decoder and… |
| 263 | [?] | P1 | player/block | 2026-06-07 playtest: Samurai cannot break blocks below him with jump+sword | Evidence source: human playtest. Cross-ref Red Stealth/Samurai attack and… |
| 264 | [?] | P1 | audio/perf | 2026-06-07 playtest: music slows down when hitting blocks and collecting diamonds | Fixed candidate 2026-06-13: live `play-native` caps the SDL queue at two generated frames and keeps the queue paused until that two-frame target is primed, including after BGM flushes, reducing start/restart underrun risk without reintroducing the older three-frame SFX lag on pickups/blocks. 2026-06-16 follow-up: current-frame SFX commands may burst one extra generated frame past the two-frame cap; otherwise a queued GEMS/fallback cue is mixed, dropped because the queue is full, and never reaches SDL during stacked pickup/block bursts. Regression: `play_native_audio_queue_cap_is_measured_in_generated_frames` covers byte accounting, the pause-until-primed threshold, and the one-frame SFX preservation limit; `mise run verify-slice-fast` passes. Human validation on a real audio device is still required. |
| 265 | [?] | P2 | audio/gameflow | 2026-06-07 playtest: EOL countdown sound does not finish | Fixed candidate 2026-06-15: `StopMusic` now clears fallback SFX tones as well as the SDL/GEMS queue, so fallback playback cannot carry the `loc_D7F4` score-counter tail into the next title-card shell when GEMS data is unavailable or the fallback mixer is active. Covered by `play_native_stopmusic_clears_fallback_sfx_for_eol_handoff`; `cargo test -q -p kidcli play_native_stopmusic -- --nocapture`, `cargo test -q -p kidcli play_native_ -- --nocapture`, and `mise run verify-slice-fast` pass. Human validation still needed on the live EOL route. |
| 266 | [?] | P1 | render/level | 2026-06-07 playtest: death-wall level does not render left-side drills | Reconfirmed 2026-06-11 play-native: death wall is still not visible to the human route, despite focused Level 0x08 capture candidates showing fallback/ROM-art drill sprites. Keep open for exact live route capture and timing/layout validation. |
| 267 | [x] | P1 | gameflow/eol | 2026-06-07 playtest: EOL flag only activates when jumped on, should activate on floor contact | 2026-06-10 verified fixed: Highwater floor-contact probe is authoritative-clean through the flag contact window once synthetic probe-camera drift is ignored; later frame-111 object-count drift is separate score/transition verifier debt. |
| 268 | [?] | P1 | form/animation | 2026-06-08 latest-pushed playtest: Iron Knight/Heavy Man wall-climb animation is wrong | Reconfirmed 2026-06-11 in latest pushed playtest: first helmet / Iron Knight animation still looks wrong around wall/climb handling. Existing renderer/core probes have not produced a clean human-equivalent route; `issue260-drips-iron-knight-climb-probe` seeds Iron Knight but never enters `iron_knight_climbing`, so keep this open until a route/frame capture isolates the wrong animation descriptor. |
| 269 | [?] | P1 | form/animation | 2026-06-08 latest-pushed playtest: Samurai held-right into slope glitches animation | Fixed candidate 2026-06-12: live Red Stealth/Samurai Level 00 route isolated frames 554..561 as `kid_y_vel` drift, not renderer table selection. Native was clearing the downhill SlopeUp y_vel mirror on the grounded `loc_9F18` tail after a solid-edge probe; ASM `loc_A13A -> loc_9D22` keeps it mirrored while slope support remains. Verified `tas-diff --scenario .cache/debug/issue269-redstealth-oracle-current/oracle-frame-scenario.json --frame-start 554 --frame-end 561 --frontier-kind gameplay --include-advisory --only-divergent` now reports 7 match / 0 diverge, and `verify-tas` frames 500..572 is CRC-clean. Human visual validation still useful. |
| 270 | [?] | P1 | player/wall | 2026-06-08 latest-pushed playtest: Level 2 well left-wall hug launches Samurai/Kid upward | Fixed candidate: `1c06f5d8` clears the natural Red Stealth BLW2 left-wall frame-266 divergence; route is gameplay CRC-clean through frame 520. Human validation still required. |
| 271 | [x] | P0 | player/enemy | 2026-06-08 latest-pushed playtest: jumping… | archived |
| 272 | [ ] | P1 | audio | 2026-06-08 latest-pushed playtest: Samurai air attack, helmet pickup voice, and bouncy sounds play slow/low | Reconfirmed 2026-06-11: Samurai noises still sound slowed/low. Static `verify-audio-output` passes title/level-entry but fails gameplay-extending routes with native onset/peak drift, so keep dynamic state-gated audio probes for #272. 2026-06-14 rerun showed `walk-right3s` oracle checkpoints remain in `LevelTransition` (`0x0008`), not `InGame` (`0x000C`); `verify-audio-output` now rejects non-boot static scenarios and directs gameplay SFX work to `verify-native-audio-output-file`. 2026-06-12 route search rejected the natural Level 0x20 spawn-to-lower-cluster family: `red-stealth-natural-lower-base` variants can approach the lower cluster, but best `0..25 Right,22..60 B` is only a no-transform waveform-control route (`current_helmet_id=0`, HP 2 throughout, no `$2E` cue). A follow-up direct upper-cluster batch from `x=128,y=248` also failed: best `0..60 Right,20..45 B,60..100 Left,90..125 B` remains 31 px below the upper prize target and never transforms. A second direct-velocity upper batch over `.cache/debug/issue272-audio/red-stealth-prize-direct.json` tried B-hold, side-swing, late-B, and double-pulse overlays against row-8 targets `(160,128)/(160,160)/(176,128)`; best live candidate was `late_b40_120`, still 32 px right and 73 px below the closest target at frame 106, with no transform evidence. 2026-06-14 isolated Samurai cue probes passed native-vs-oracle audio: `red-stealth-c-direct` emits `PlaySound(35)` for `sfx_Red_Stealth_sword_swinging`, and scratch `red-stealth-air-attack-direct` matches ASM `loc_B350` by emitting `PlaySound(13)` at frame 9 for `sfx_Red_Stealth_attack_down`; both pass `verify-native-audio-output-file`. Same-day `cargo test -q -p kidcli play_native_ -- --nocapture` also passes the clock-prize duplicate and core queued-SFX native-player tests. 2026-06-14 fallback fix: when GEMS data is unavailable, `SfxEngine` now uses SDL's obtained output sample rate, so 48 kHz devices no longer stretch 44.1 kHz fallback tones slow/low. Follow-up same day: fallback SFX now replays exact queued core/GEMS IDs before heuristics, so diamond/clock cues fire on ASM contact frames (`loc_2510C`, `PrizeClock_Init`) and metal/rubber/transform cues do not fall back to late or generic tones. Follow-up `2f723ab8`: live `play-native --record` now generates GEMS/fallback audio at SDL's obtained queue rate instead of fixed 44.1 kHz and writes `audio.wav` with the matching sample rate, removing another 48 kHz slow/low capture path. Verified `cargo test -q -p kidcli fallback_sfx_ -- --nocapture`, `cargo test -q -p kidcli play_native_ -- --nocapture`, `cargo test -q -p kidcli recording -- --nocapture`, and `mise run verify-slice-fast`. 2026-06-14 cached direct route `issue272-red-stealth-lower-direct-x96-b20-45` does reach native Red Stealth pickup at frame 125 (`current_helmet_id=3`, `form_change_flash_frames=55`, `PlaySound(46)` = `sfx_Red_Stealth_transform`), but reject it as a verifier route: `verify-tas --frame-start 100 --frame-end 140 --frontier-kind gameplay --bisect` diverges immediately at frame 100 (`number_objects` 3 vs 4), and `verify-native-audio-output-file` also fails before the pickup window (`transform_window@100` hitpoints 1 vs 2). 2026-06-15 follow-up: no-GEMS fallback cues for Red Stealth/Samurai are now distinct, louder shapes instead of a shared weak 659 Hz blip: `loc_B350` attack-down, `sub_868E` sword/hit-target, and `loc_7CD0` transform each queue separate fallback tones while preserving the ASM sound IDs. Covered by `fallback_sfx_gives_red_stealth_cues_distinct_audible_shapes_loc_b350_868e_7cd0`. 2026-06-15 Heavy/Iron Knight metal-block ding retest: ASM `sub_AF7A -> loc_110D0` already reaches typed `QueuedSoundEffect::MetalBlock` (`$12`) from the top-hit metal path, and native-player replays it without the generic `$0D` crunch; `cargo test -q -p kid-core metal_block -- --nocapture` and `cargo test -q -p kidcli metal_block -- --nocapture` pass. Do not retread direct RedStealth C/air-attack, clock-prize duplicate, fallback/live-recording sample-rate, queued-ID replay, metal-block `$12` replay, `issue272-red-stealth-lower-direct-x96-b20-45`, or broad `play_native_` cue-event handlers; next route should focus on real-device validation or reach a RedStealth pickup while staying oracle-aligned through item spawn. |
| 273 | [?] | P1 | block/prize | 2026-06-08 latest-pushed playtest: later-level helmet prize blocks may not be mapped/extracted | Evidence source: human playtest. Symptom: all helmet prize blocks in later… |
| 274 | [ ] | P1 | demo | 2026-06-09 playtest: attract-mode intro demo — Kid grabs the Cyclops/Eyeclops… | Reconfirmed 2026-06-11 by human playtest, but native trace now proves the published demo exposes Cyclone content before later real Diamond prizes. 2026-06-12 current native/oracle frame refresh on the cold-idle route shows the early prize frames 3488/3491/3494 match as Cyclone/spin-form, while frames 8385/8388/8391 match as a later real Diamond in a different demo segment. Dummy-SDL attract bundle `.cache/recordings/issue284-live-dummy-attract-4500` also verifies trace/state/frames and shows the same separation: first prize window is Cyclone/spin-form, later prize window is a real Diamond. 2026-06-14 current-HEAD refresh confirmed frame 3488 pixel-perfect, frame 3491 isolated to first-transform render timing after `Kid_Transform loc_7CD0`, and frame 3494 player/prize crop pixel-perfect. Follow-up latching of the `sub_44DC loc_45EE` HUD diamond page removes the frame-3491 HUD residual (`AE 39 -> 0`). Follow-up replay of `Make_SpriteAttr_HUD loc_BDC0` / `loc_BE82` transform HP-pip suppression keeps frame 3492 at `AE 39` and drops frames 3493/3494 from `AE 143` to `AE 39`. Follow-up transform-hibernate HUD diamond latch keeps the visible `unk_461C` page one art slot behind the post-update trace while `Kid_Transform loc_7CD0` hibernates, making cold-idle frames 3491..3494 all pixel-perfect (`AE 0/0/0/0`). Next step remains an exact live SDL route/frame capture if the visible report is still wrong. Cross-ref reopened #284. |
| 275 | [?] | P1 | render/hud | 2026-06-09 playtest: collected diamond does not fly up to the top-right HUD diamond counter | Fixed candidate: native now mirrors ASM `PrizeDiamondCollected_Init` / `Increase_Diamonds` by keeping a collected-diamond HUD-flight gfx worker from pickup screen offset to HUD x=$126/y=$30 before the delayed count award. Human visual validation still required. |
| 278 | [x] | P1 | block/collision | 2026-06-09 playtest: breakable floor blocks (level 3, below the Heavy Man… | Fixed 2026-06-11: floor prize-block impacts now route through ASM `loc_1002E`/`sub_1007A`, keeping the struck cell `$6Fxx` solid during the wobble instead of erasing Kid's support. |
| 279 | [?] | P1 | enemy/block | 2026-06-09 playtest: Rhino/Berzerker cannot destroy the light/breakable blocks it should be able to (level 3) | Fixed candidate 2026-06-11: Berzerker side block dispatch now follows ASM `loc_92DE`/`off_933E` for Prize and Ice blocks, including `Block_LoadSmashBits` and `IceBlock_SmashAndLoadBullets`. Human level-3 route still needs validation. |
| 280 | [?] | P2 | render/background | 2026-06-09 playtest: parallax sky loads in late (level 3 top), stray dark-green… | Reconfirmed 2026-06-11 play-native: death-wall/hell route still shows green background artefacts. The Hills `$08` Plane-B wrap candidate fixes one captured frame-151 sky hole, but broader live-route background/layout parity remains open. |
| 281 | [?] | P1 | block | 2026-06-09 playtest: push-up blocks rise silently when jumped on from above and… | Fixed candidate 2026-06-11: Shifting blocks now activate only from ASM `sub_BB54`/`off_BB80[7]` headbutts, play `sfx_Shifting_block`, reserve `$6F` above, then commit `$E710` and erase the source in `sub_113C0` order. Human route validation still required. |
| 282 | [x] | P3 | infra/debug | Level-skip (Next) should show the level intro/name card so the level name is visible for debugging | archived |
| 283 | [?] | P1 | gameflow/death | 2026-06-09 playtest: death-wall level respawn drops Kid back at the spot he died, not the level/checkpoint start | Fixed candidate 2026-06-11: murder-wall contact now follows ASM `sub_5E02`/`Crushed_to_Death` instead of ordinary HP damage, and Level 0x08 death-wall respawn is covered by `murder_wall_death_respawns_at_level_start_loc_5e34_b8de`. Human death-wall route validation still required. |
| 285 | [x] | P2 | render/enemy | 2026-06-10 playtest: DripsSurfaceCrawler renders as a tiny blob instead of the proper floor-slime crawl animation | Fixed 2026-06-10: native now distinguishes ASM `Drips.asm loc_338F8`'s spawned `Tar_Monster.asm loc_325B6` helper from the original crawler's `loc_3392C` finish shell. The helper still renders the Tar `$0C1` fragment, while the crawler finish renders Drips `stru_33F06` (`$0C5/$0C9/$0D2`) and the existing startup/moving crawler render paths remain `stru_33FA6`/`stru_33F14`. Verified with focused core lifecycle tests, renderer regression, `cargo fmt --check`, and `cargo check -q -p kidcli`. |
| 286 | [b] | P1 | enemy/lifecycle | 2026-06-10 playtest: second stomp on RockTank crushes it visually but the tank stays alive/active | Needs fresh live/render repro before more Tank HP/death changes. Current evidence argues against patching: isolated `second_stomp_plain_rock_tank_enters_death_animation_loc_3b464_3b480` covers the ASM `Tank.asm loc_3B464`/`loc_3B480` second-stomp death path, and the old #271 stomp-gate route no longer reproduces a Tank stomp bounce. Recheck 2026-06-11: `tas-diff` on the current #271 sharp RockTank route frames 200..260 is enemy-frontier clean (60 match / 0 diverge). |
| 287 | [?] | P2 | audio | 2026-06-10 playtest: Samurai attack plays several overlapping attack sounds instead of one per swing | Fixed candidate 2026-06-15: direct Red Stealth airborne attack route `.cache/debug/issue272-audio/red-stealth-air-attack-direct.json` reproduced the over-trigger as grounded frames 26..46 repeatedly replaying `sfx_Red_Stealth_attack_down` (`$13`) while C stayed held. Native now gates the split `sub_B270` airborne slash helper out on grounded frames, matching the ROM call path, and regenerated recording `issue272-red-stealth-air-attack-direct-after-ground-gate` queues one `$13` at frame 10 only. Regression `red_stealth_grounded_c_hold_does_not_restart_airborne_attack_down_sound`; nearby `red_stealth_airborne` tests pass. Keep `[?]` until human retests the live Samurai route because pitch/mix issues remain under #272/#264. |
| 288 | [?] | P2 | intro/render | 2026-06-10 playtest: intro shows the jacket sprite in the bottom-left before… | Reopened 2026-06-13 from latest playtest after `1fb6744a`: intro still shows the jacket/Kid sprite at bottom-left before the throw. Rechecked 2026-06-14 on `602f276f`: `issue288-live-intro5-check` passes trace/state/frame verification, and the longer `issue284-live-dummy-attract-4500` route has zero-pixel diffs for frames 1860..2460, covering the old frame-1934 leak window. That longer route now drifts later at frame 3491 on native trace replay, so the live symptom still needs an exact route/frame or different timing path before more invasive IntroSequence5/6 rendering changes. 2026-06-15 current-HEAD visual retest: the old `issue288-live-intro5-check` recording bundle is stale at trace frame 0, but replaying its DynamicScenario and capturing native/oracle frames `1860,1920,1934,1980,2040,2400,2460` gives exact ImageMagick AE `0` for every frame. Fresh dummy-SDL capture `issue288-live-headless-current` exposed a separate first-frame handoff miss: native frame 1876 blanked when ASM `sub_1B988 -> Game_Mode=$24 -> j_loc_6E2` oracle still displayed the final IntroSequence4 VDP for one frame before `Load_IntroSequence5`/`loc_1BA28`'s hidden shell. Fixed candidate 2026-06-15 carries the previous VDP only for IntroSequence5 elapsed 0, while frames 1920/1934 remain black and frame 1980 remains visible. Verification: `cold_boot_intro_sequence_5_first_frame_carries_intro_4_vdp`, `intro_sequence_5` focused tests, and native/oracle ImageMagick AE `0` at frames 1860/1876/1920/1934/1980 including bottom-left crop. Original reported live jacket leak still needs pushed-build retest or exact live frame if it appears on a different route. |
| 289 | [~] | P1 | level/spawn | 2026-06-10 playtest: hell-theme levels start Kid in the wrong place and the map layout looks wrong | Partial 2026-06-13: Level 0x06 / Under Skull Mountain 3 remains clean, and Level 0x07 / Isle of the Lion Lord no longer pre-ages the Lion through the 24-frame hidden Flying Dragon tail. Native now applies that extra settled-warp scheduler tail only when a Flying Dragon is active (`Dragon_Flying.asm loc_348A0` -> `loc_34BAC`), while `Lion.asm loc_3E078`/`loc_3E09E` stays in the oracle-backed roar setup at frame 24 without pre-damaging Kid. Verified with Level 0x07 Lion regressions and `tas-diff` frames 24..120 for issue289 levels 0x05, 0x06, and 0x0D (all 96/96 match). Follow-up classified Level 0x07 Lion projectile/body-contact `Number_Objects` drift as an oracle export limitation: CRC-clean frames expose the Lion child object slot at `Lion.asm loc_3E196`/`loc_3E2A8` while oracle `Number_Objects` is one lower. 2026-06-14 fixed subcase: generated `Level_Layout` fixtures and block-list rows now apply ASM `sub_140F6` / `loc_141EE` blank-top-row expansion when `Number_blank_top_rows` is set, so flagged generated levels no longer mix post-`Load_LevelData` bounds with pre-shift terrain/block-list coordinates. Covered by `generated_hell_level_layout_applies_blank_top_rows_sub_140f6`. 2026-06-14 Level 0x0D settled idle route remains gameplay-clean for frames 24..120; the central frame-120 visual mismatch traced to #177 live Diamond animation, not level spawn/layout. 2026-06-14 visual subcase: Level 0x06 Flying Dragon descent fire now renders ASM `Dragon_Flying.asm loc_34BAC` child frames `unk_C75CE`/`unk_C75D6`/`unk_C75DE` instead of the 6x6 placeholder; gameplay slice remains 96/96 match, frame 60 MAE improves 329.475 -> 320.495 while strict AE is 1155 -> 1166, and frame 179 stays 1129 AE / 269.74 MAE, leaving a dragon pose/flame residual. 2026-06-14 visual/state subcase: Flying Dragon descent `$D8F1` now follows ASM `Dragon_Flying.asm loc_3488A`/`loc_346C4` by resuming the current main-script `a0` instead of reloading variant 0 from `$48(a5)`; Level 0x06 gameplay remains 96/96 match for frames 24..120, frame 60 stays 1166 AE / 320.495 MAE, and frame 179 improves 1129 AE / 269.74 MAE -> 750 AE / 165.3 MAE. Covered by `dragon_flying_descent_tail_resumes_next_script_command_loc_3488a_d8f1` and `dragon_flying_descent_tail_preserves_player_facing_for_next_command_loc_3488a_d8f1`. 2026-06-14 renderer subcase: live Flying Dragon sprites now follow ASM `Dragon_Flying.asm stru_34600` (`C75EE/C75F6/C75FE/C7606` repeats, with `C760E` only at the scripted slots) instead of the old even five-frame fallback loop. Focused `flying_dragon_` renderer tests pass, Level 0x06 gameplay remains 96/96 clean, and fresh `.cache/debug/issue289-level06-stru34600-scenario/` frame 179 shows the Dragon pose much closer to oracle. 2026-06-14 renderer follow-up: hibernated/zero-velocity Flying Dragons also keep the same `stru_34600` animation instead of falling through the generic stationary frame-0 shortcut; `flying_dragon_` tests and Level 0x06 frames 24..120 remain clean, and `.cache/debug/issue289-level06-stationary-stru34600-scenario/` frame 179 now visually matches the oracle Dragon pose. Rejected 2026-06-14: adding raw `off_345F6` descent body frame `unk_C7626` for native frame 60 worsened strict AE 1114 -> 1259 against oracle; follow-up found `unk_C7626` is the anonymous Data_Index row skipped by `off_345F6`. 2026-06-14 renderer follow-up: native now selects Flying Dragon dive body frames from ASM `Dragon_Flying.asm loc_348CE` / `off_345F6` velocity bands, including frame-60 `-2,-3` -> `LnkTo_unk_C762E`. Focused `flying_dragon_` tests, `cargo check -q -p kidcli`, and `mise run verify-slice-fast` pass; `verify-recording --slug issue-289-level06-idle-current` remains state-clean and improves frame 60 to MAE 1.0588 / 947 changed pixels, while frame 179 remains MAE 0.2112 / 157 changed pixels. Rejected 2026-06-14: flipping descent-fire rendering from the child x-velocity sign, despite ASM `loc_34BAC` copying parent `x_direction`, worsened frame 60 to MAE 1.4537 / 1091 changed pixels; current residual is not a simple child H-flip issue. Remaining residual is small flame/background/render-priority mismatch. Broader hell-theme wrong-start/graphics work remains open. |

Issue 289 rejected 2026-06-14: changing Flying Dragon dive descriptor origin or chunk order is unsupported by ASM `Make_SpriteFromGfxObject` (`loc_129E`/`sub_13C0`). Frame-60 `C762E` correctly draws at `(x - xcenter, y - height + ycenter) = (56,87)` and emits a 32px chunk plus 8px tail from tile `$1D9`; pursue the remaining flame/background/render-priority residual instead.
Issue 289 fixed subcase 2026-06-15: live `play-native` level-change asset fallback now re-enters ASM `loc_6E2`/`Load_TitleCard` for the resolved playable target instead of raw `enter_level_playtest`, so missing generated assets for the requested level cannot expose the replacement level's spawn/fall before the loader shell. Covered by `live_level_change_resolved_target_reenters_rom_title_card` and `mise run verify-slice-fast`.

Issue 289 fixed subcase 2026-06-15: title-card predicates and starting-level workers now resolve title metadata through `MapOrder_Index`/resource records before falling back to the logical level record, matching ASM `Load_TitleCard`/`loc_6E2`. This prevents aliased levels such as Under Skull Mountain 3 (`0x06 -> resource 0x02`) from taking the no-title-card start path and exposing an incorrect live spawn shell. Covered by `aliased_level_entry_uses_resource_title_card_loc_6e2`, `generated_title_card_check_uses_map_order_resource_record`, `cargo test -q -p kid-core title_card -- --nocapture`, `cargo test -q -p kidcli debug_hotkey_level_skip_ -- --nocapture`, and `cargo test -q -p kidcli live_level_change_ -- --nocapture`.

Issue 289 fixed subcase 2026-06-15: direct `play-native` level-change asset swaps now rebuild the hidden opening slice against the freshly loaded target block grid while the ASM `loc_74E0`/`loc_1AE2E` title-card shell owns the transition. This closes the gap where the visible grid refreshed but the first live spawn could still preserve a stale no-floor/old-level settle pose and appear to fall from the top or into a void after debug skipping. Covered by `live_level_change_direct_target_refreshes_runtime_block_grid_and_title_card`, `cargo test -q -p kidcli live_level_change_ -- --nocapture`, and `cargo test -q -p kidcli debug_hotkey_level_skip_ -- --nocapture`.

Issue 289 note 2026-06-14: fixed the Flying Dragon descent-fire second cleanup sprite. ASM `Dragon_Flying.asm loc_34C4A` renders `LnkTo_unk_C75DE` for the first three-frame cleanup loop, then switches to `LnkTo_unk_C75E6` for the second; native previously reused `unk_C75DE` for both states. Regression coverage now includes `CleanupSecondLoc34C4A -> unk_C75E6`. Refreshed Level 0x06 idle sample frames 60/179 are unchanged by this subcase (947 AE / 272.105 MAE at frame 60; 157 AE / 54.2707 MAE at frame 179), so the remaining sampled residual is still elsewhere in the flame/background/render-priority path.

Issue 289 rejected 2026-06-14: the remaining frame-60 Dragon residual is not bad Dragon art, palette, or the `C762E` descriptor. Oracle WRAM/GfxObject probes show the parent at `addroffset_sprite=$05AC`, `vram_tile=$0409`, `palette_line=2`, and the child fire at `$0580`; decoded native `Enemy0C_Dragon.bin` tile `$1D9` matches oracle VRAM tile `$5E2` byte-for-byte (`01 32 20 00 ... 70 00 82 42`), and oracle CRAM line 2 matches `Dragon_1` after Mega Drive colour-bit masking. Oracle VDP pixel samples see the frame-60 Dragon sprite at `(57,87)`/`(74,87)`, but the captured oracle PNG still shows background there until frame 61, while native already draws the sprite in frame 60. A capture-side experiment rendering the previous synthetic VDP snapshot for full-frame `capture-scenario-frames` left frame 60 unchanged at 947 AE, so do not retry the simple one-frame VDP latch patch without first tracing the capture/oracle frame numbering boundary.

Issue 289 note 2026-06-14: refreshed `verify-recording --slug issue-289-level06-idle-current` with native pixel provenance. The Level 0x06 route remains state-clean and strict pixels still fail only as expected (`frame_00060` 947 changed pixels / MAE 1.0587751, `frame_00179` 157 changed pixels / MAE 0.21117002). Frame-60 Dragon-body sample pixels such as `(56,88)` and `(58,87)` are low-priority `enemies` layer writes over black/red background, not a high-priority mask error; fire sample pixels such as `(81,130)` and `(94,128)` are low-priority `effects` layer writes shifted against black. Do not retry broad sprite-priority, `C762E` descriptor-origin, Dragon art/palette, or fire H-flip patches; next useful pass should trace the oracle/native frame-boundary or VDP capture timing around `Dragon_Flying.asm loc_348CE` / `loc_34BAC`.

Issue 289 partial 2026-06-15: frame-boundary cross-compare showed native frame 60 matched oracle frame 61 at 75 changed pixels, while same-index compare was 947 changed pixels. ASM `Mode_Level` starts at `kid.asm:901` by transferring the already-built Sprite_Table before `Execute_Objects` and `GfxObjects_MoveAndAnimate`, then rebuilds the next table near the end of the frame with `Make_SpritesFromGfxObjects`; native-player now latches prior-frame non-player GfxObject sprite sources while preserving current Kid state. `verify-recording --slug issue-289-level06-idle-current` remains state-clean and now reduces frame 60 to 75 changed pixels / MAE 0.051353235; frame 179 remains 157 changed pixels / MAE 0.21117002. The Dragon/body/fire residual is cleared; remaining strict pixels are right-edge/HUD/edge-effect sized, not the Dragon descent path.

Issue 289 rejected 2026-06-15: normal gameplay must not apply an extra one-slot `unk_461C` HUD diamond DMA lag beyond the existing previous-trace latch. Although `Mode_Level` calls `Make_SpriteAttr_HUD/sub_BDC0` before `sub_44DC loc_45EE`, changing `render_trace_with_vdp_latched_state` to map the previous HUD diamond offset through `previous_hud_diamond_frame_offset_loc_45ee` worsened `verify-recording --slug issue-289-level06-idle-current` frame 60 from 75 changed pixels / MAE 0.051353235 to 114 changed pixels / MAE 0.07773437 and broke the Circle of Doom HUD-diamond VRAM art expectation. Keep the existing normal previous-trace HUD diamond latch; remaining pixels are still right-edge terrain/HUD-sized, with frame 179 adding a small digit/row cluster.
Issue 289 verifier note 2026-06-16: `verify-recording --slug issue-289-level06-idle-current` now normalises additive core level-BGM and `loc_1223C` lava/water diagnostics for legacy recordings. The route no longer fails at frame 0 trace-schema drift; current report is `trace_match=true`, `oracle_state_match=true`, `oracle_frame_match=false`, with frame 60 still 75 changed pixels / MAE 0.051353235. Treat the remaining work as visual pixels only, not gameplay state drift.
