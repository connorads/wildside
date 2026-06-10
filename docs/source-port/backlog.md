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
- **Next**: #260 live slime/metal-block route (the Drill-skin headbutt blocker is now
  gameplay-CRC clean on `.cache/debug/drips-route-candidates/orig_l480_530_b528.json`
  through frame 570, so do not re-open block type C/`sub_BB54` from that route; still
  need a route that reaches the reported live slime/metal-block contact. Current clean
  near-miss: `.cache/debug/issue260-tail-grid/l524_b530_540.json` passes
  gameplay CRC and enemy checkpoints through frame 560 and reaches `(456,543)` only as
  close as healthy `dx=0 dy=-16` at frame 540. The right-side raw `dx=+13 dy=-1`
  candidates are HP-zero/LevelTransition routes; healthy closest remains `(584,543)`
  `dx=0 dy=-48`, so do not mine them as proof. Do not reuse
  `.cache/debug/issue260-route-probes/floor-explicit-left-drips-close-617.json` as live
  metal-block evidence. Fresh 2026-06-10 trace reconfirms that exact route is already
  Drips-phase shifted before contact because it uses `settle_oracle_warp=true`: native #08
  seeds at frame 13 with wait `$43` while oracle exposes `loc_33424 d0=$5d` at frame 15.
  The no-settle copy starts #07/#08 clean but is not a valid late metal-block proof either
  because the oracle side has lost the comparable Drips slots by frame 590. Next useful #260
  work is a genuinely refreshed route/bootstrap that starts clean and stays oracle-comparable,
  not Tank/Drips hitbox/block patches from these stale route files; #285 crawler art);
  #277 left-jump input drop reconfirmed after the loc_7464 candidate fix; the
  existing natural BLW2 route is not the repro because its frame-207 L+B jump
  stays CRC-clean and honours held-left through loc_7664 -> loc_A426 -> sub_B084;
  next pass needs a direct human-equivalent left-wall/stair jump repro;
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
| 284 | [?] | P0 | block/prize | 2026-06-10 playtest: later-level prize blocks award diamonds instead of their… | Fixed in b0e2a098; generated RAM-dump parser now rejects foreground-queue spill records masquerading as Diamond prizes. Human playtest still required. |
| 290 | [?] | P1 | trace/objects | Full any-percent TAS diverges at checkpoint frame_7800 (demo_play): native objects=9/4 vs oracle 10/9 | Fixed candidate: Dragon.asm loc_3CE90/loc_3CF90 breath child object-only count; full published TAS passes all 37759 frames by CRC. Human validation still required to close. |
| 260 | [~] | P0 | block/enemy | 2026-06-07 playtest: Level-2 slime/metal-block interaction breaks block and emits red projectile left | Drill-skin block no-op is now gameplay-CRC clean on the current close route; still needs a fresh route that reaches the reported live slime/metal-block contact. |
| 277 | [ ] | P1 | player/input | 2026-06-09 playtest: holding left into a wall then jumping drops the left input… | Reconfirmed 2026-06-10 after the loc_7464 Left+Right candidate. Existing BLW2 natural L+B route is CRC-clean and honours held-left; direct human-equivalent left-wall/stair jump repro still needed. |
| 276 | [?] | P1 | hud | 2026-06-09 playtest: player hit-point HUD does not decrement when hit (shows 4, take a hit, still shows 4) | Fixed candidate: oracle confirms same-frame HUD delay is original Mode_Level/sub_BDC0 timing; native render latch now preserves previous HP pips for the hit frame. Human validation still required. |
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
| 228 | [?] | P1 | player | Wire ASM-faithful helmet/form/Diamond-Power helpers into the live path; extends #206 beyond the DP invuln stub | Helmet/form/Diamond-Power fidelity audit 2026-05-31… |
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
| 253 | [ ] | P3 | gameflow | Highwater Pass 2 (level 03): native score runs +40 ahead of the oracle (real… | Found while oracle-investigating #250: `verify-tas --scenario… |
| 254 | [?] | P1 | intro/render | 2026-06-07 playtest: intro ninja animation has black outline artefacts | Evidence source: human playtest. Symptom: ninja animation in the intro has… |
| 255 | [x] | P1 | menu/input | 2026-06-07 playtest: level select accepts… | archived |
| 256 | [ ] | P1 | render/enemy | 2026-06-07 playtest: third-level Rhino/Berzerker renders in front of foreground tiles | Evidence source: human playtest. Symptom: Rhino/Berzerker on the third level… |
| 257 | [?] | P0 | gameflow/death | 2026-06-07 playtest: dying goes black and only recovers when Start is pressed | Evidence source: human playtest. Symptom: dying still goes black and does not… |
| 258 | [?] | P0 | player/slope | 2026-06-07 playtest: walking left/right/up slopes regressed and Kid/forms get stuck | 2026-06-07 fixed base-Kid Level 00 natural slope route: `verify-tas --scenario… |
| 259 | [?] | P1 | enemy/render | 2026-06-07 playtest: tanks still appear stuck in the floor | Current Level 02 enemy-clean route shows native/oracle Tank placement matched; low appearance is foreground/terrain occlusion. Needs human validation / death visibility handled via #167/#252. |
| 261 | [x] | P1 | block/projectile | 2026-06-07 playtest: fourth-level growing blocks still shoot spikes | archived |
| 262 | [?] | P1 | level/render | 2026-06-07 playtest: levels 5/6/7 tilesets do not match character placement | Evidence source: human playtest. Cross-ref #238/#251 terrain decoder and… |
| 263 | [?] | P1 | player/block | 2026-06-07 playtest: Samurai cannot break blocks below him with jump+sword | Evidence source: human playtest. Cross-ref Red Stealth/Samurai attack and… |
| 264 | [ ] | P1 | audio/perf | 2026-06-07 playtest: music slows down when hitting blocks and collecting diamonds | Evidence source: human playtest. Cross-ref #003/#160/#216 audio rows. `play-native` audio diagnostics now include frame-loop overrun counters; next repro capture should classify CPU/frame budget vs GEMS/mixer timing. |
| 265 | [ ] | P2 | audio/gameflow | 2026-06-07 playtest: EOL countdown sound does not finish | Evidence source: human playtest. Cross-ref #253 collapsed EOL modelling and… |
| 266 | [ ] | P1 | render/level | 2026-06-07 playtest: death-wall level does not render left-side drills | Evidence source: human playtest. Exact level ID/name uncertain. Native repro:… |
| 267 | [x] | P1 | gameflow/eol | 2026-06-07 playtest: EOL flag only activates when jumped on, should activate on floor contact | 2026-06-10 verified fixed: Highwater floor-contact probe is authoritative-clean through the flag contact window once synthetic probe-camera drift is ignored; later frame-111 object-count drift is separate score/transition verifier debt. |
| 268 | [~] | P1 | form/animation | 2026-06-08 latest-pushed playtest: Iron Knight/Heavy Man wall-climb animation is wrong | Evidence source: human playtest. Symptom: first helmet form ("Benny Manthe… |
| 269 | [~] | P1 | form/animation | 2026-06-08 latest-pushed playtest: Samurai held-right into slope glitches animation | Evidence source: human playtest. Symptom: running Samurai into a slope while… |
| 270 | [ ] | P1 | player/wall | 2026-06-08 latest-pushed playtest: Level 2 well left-wall hug launches Samurai/Kid upward | Evidence source: human playtest. Symptom: down the Level 2 well, hugging the… |
| 271 | [x] | P0 | player/enemy | 2026-06-08 latest-pushed playtest: jumping… | archived |
| 272 | [ ] | P1 | audio | 2026-06-08 latest-pushed playtest: Samurai air attack, helmet pickup voice, and bouncy sounds play slow/low | Evidence source: human playtest. Cross-ref #003/#160/#216/#264. Native repro:… |
| 273 | [?] | P1 | block/prize | 2026-06-08 latest-pushed playtest: later-level helmet prize blocks may not be mapped/extracted | Evidence source: human playtest. Symptom: all helmet prize blocks in later… |
| 274 | [ ] | P1 | demo | 2026-06-09 playtest: attract-mode intro demo — Kid grabs the Cyclops/Eyeclops… | Watch attract demo; confirm Kid grabs Cyclops and spins (no prize-box diamond)… |
| 275 | [ ] | P1 | render/hud | 2026-06-09 playtest: collected diamond does not fly up to the top-right HUD diamond counter | Collect a diamond, confirm it animates to the top-right counter RECONFIRMED… |
| 278 | [ ] | P1 | block/collision | 2026-06-09 playtest: breakable floor blocks (level 3, below the Heavy Man… | Break the three floor blocks, confirm Kid stands on stone, not falls through |
| 279 | [ ] | P1 | enemy/block | 2026-06-09 playtest: Rhino/Berzerker cannot destroy the light/breakable blocks it should be able to (level 3) | Charge Rhino into breakable blocks, confirm destruction |
| 280 | [ ] | P2 | render/background | 2026-06-09 playtest: parallax sky loads in late (level 3 top), stray dark-green… | Reach top of level 3 / death-wall level, confirm sky/clouds render on time… |
| 281 | [ ] | P1 | block | 2026-06-09 playtest: push-up blocks rise silently when jumped on from above and… | Hit block from below, confirm it rises with SFX; landing on top should not pass through |
| 282 | [ ] | P3 | infra/debug | Level-skip (Next) should show the level intro/name card so the level name is visible for debugging | Press Next, confirm level name card displays before gameplay RECONFIRMED… |
| 283 | [ ] | P1 | gameflow/death | 2026-06-09 playtest: death-wall level respawn drops Kid back at the spot he died, not the level/checkpoint start | Die on death-wall level, confirm respawn at level/flag start |
| 285 | [?] | P2 | render/enemy | 2026-06-10 playtest: DripsSurfaceCrawler renders as a tiny blob instead of the proper floor-slime crawl animation | Renderer now selects ASM `stru_33FA6` startup and `stru_33F14` moving crawler frames… |
| 286 | [~] | P1 | enemy/lifecycle | 2026-06-10 playtest: second stomp on RockTank crushes it visually but the tank stays alive/active | Isolated two-stomp Tank death path passes; old #271 stomp-gate window no longer reproduces a Tank stomp bounce; needs fresh live route/render proof |
| 287 | [ ] | P2 | audio | 2026-06-10 playtest: Samurai attack plays several overlapping attack sounds instead of one per swing | Evidence source: human playtest. Distinct from #272 (pitch); this is repeated… |
| 288 | [ ] | P2 | intro/render | 2026-06-10 playtest: intro shows the jacket sprite in the bottom-left before… | Evidence source: human playtest (long-standing). Two related intro-sequence… |
| 289 | [ ] | P1 | level/spawn | 2026-06-10 playtest: hell-theme levels start Kid in the wrong place and the map layout looks wrong | Evidence source: human playtest. Check header/start-position decode and layout… |
