# Playtest Backlog

Canonical tracker for bugs and missing features. See `CLAUDE.md` for iteration protocol and `docs/source-port/methodology.md` for the investigation playbook.

Status: `[ ]` open, `[~]` in progress, `[x]` done, `[-]` won't fix / not a bug
Priority: P0 (game-breaking), P1 (core feel), P2 (missing system), P3 (polish)

| # | Status | Pri | System | Summary | Verify |
|-----|--------|-----|--------|---------|--------|
| 001 | [x] | P0 | player | Player gets stuck after first damage | Take hit, confirm movement resumes |
| 002 | [x] | P0 | enemy | Enemies don't die when stomped | Stomp enemy on level 0x00, confirm death |
| 003 | [x] | P1 | audio | No music or sound effects playing | Load level, confirm BGM and SFX |
| 004 | [x] | P1 | render | Colour palette wrong (floor/tree colours reversed) | Capture frame, compare to oracle |
| 005 | [-] | P1 | render | No parallax background | NOT A BUG — parallax working correctly |
| 006 | [x] | P1 | enemy | Dragon moonwalking (sprite direction vs movement) | Watch dragon, confirm facing matches walk dir |
| 007 | [x] | P1 | camera | Screen shakes/moves weirdly when hit | Take damage, confirm camera stays stable |
| 008 | [x] | P1 | render | Unwanted dark overlay effect when hit | Take damage, confirm no full-screen tint |
| 009 | [x] | P2 | menu | No menu — spawns directly into game | Launch, confirm title screen appears |
| 010 | [x] | P2 | enemy | No enemy attack patterns (fire breath, arrows) | Reach archer/dragon, confirm projectiles |
| 011 | [x] | P2 | block | Non-P bricks look half-broken (wrong sprite state) | Resolved by palette fix (#004) |
| 012 | [x] | P2 | block | No particle effects when blocks break | Hit block, confirm debris particles |
| 013 | [-] | P2 | block | P blocks don't convert to regular blocks after hit | NOT A BUG — prize blocks vanish correctly |
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
| 030 | [x] | P3 | infra | Add GitHub Actions CI running `verify-slice-fast` | Tests run automatically on every push |
| 031 | [x] | P3 | infra | Add performance baseline tracking | criterion benchmarks: ~92ns/frame idle, ~3µs level load |
| 032 | [x] | P2 | fidelity | Per-form Diamond Power effects (20 unique abilities) | Each form has correct visual and damage pattern |
| 033 | [x] | P2 | fidelity | Form-change animation from ASM Kid_Transform tables | Original transform art/timing loaded for all forms, including Juggernaut composite rendering |
| 034 | [x] | P3 | fidelity | Intro sequence 5 segments (ASM has 5, native had 3) | Boot captures match oracle |
| 035 | [-] | P2 | fidelity | Resolve 6 oracle-warp-broken levels (0x4E73) | ORACLE LIMITATION — jgenesis warp bug, native works; determinism + soak tested |
| 036 | [x] | P2 | enemy | Tank walk speed step wrong ($10000 should be $8000) | Tank speed matches ASM at all HP tiers |
| 037 | [~] | P3 | fidelity | Floor-snap settling diffs: 0x52 (~2px cam_y, ~2.25px kid_y), 0x29 (+5px X), 0x24/0x51 (1px cam_y) | Needs full physics step during init |
| 038 | [-] | P3 | fidelity | Camera Y settling 1px diffs on 0x02, 0x24, 0x51 | MERGED with #037 — same root cause |
| 039 | [-] | P3 | fidelity | 4th speed table entry is dead data in ROM | NOT A BUG — unreachable slot |
| 040 | [x] | P2 | enemy | Lion constants wrong (detect range, cooldown, projectile) | Lion matches ASM line-by-line |
| 041 | [x] | P2 | enemy | Hand jump velocity fixed, should be randomised per-tier | Hand matches ASM jump tables |
| 042 | [x] | P2 | enemy | Tornado acceleration $300 should be $C00, level-dependent timer | Tornado matches ASM init table |
| 043 | [x] | P2 | enemy | Dragon Flying is structural placeholder (3 variant scripts) | Dragon Flying matches ASM flight scripts |
| 044 | [x] | P3 | enemy | Lion projectile wall-bounce and per-enemy homing accel | Projectile bounces off terrain |
| 045 | [x] | P3 | enemy | Hand player-grab mechanic missing entirely | Hand grabs and drags player |
| 046 | [x] | P3 | enemy | Tar Monster timing/visibility/glob shot wrong | Tar Monster matches ASM bytecode |
| 047 | [x] | P3 | enemy | Drips random timing and surface-crawl spawning | Drips matches ASM drop timing |
| 048 | [x] | P3 | enemy | Spinning Twins orbit-to-walk should be collision-triggered | Transition on player contact |
| 049 | [-] | P3 | fidelity | Level 0x29 has 5px X position diff (slope-slide during gravity settle) | MERGED with #037 — same root cause |
| 050 | [-] | P3 | fidelity | 4 levels show idle_tail position diffs from enemy/timing divergence | EXPECTED — advisory: oracle warp timing + enemy interaction accumulation |
| 051 | [x] | P2 | enemy | Dragon HP base 2 should be 1 (ASM Dragon.asm:65 addq.w #1) | Dragon HP matches ASM |
| 052 | [x] | P2 | enemy | EmoRock velocity cap logic wrong ($60000 threshold → $40000 reset) | EmoRock drop matches ASM |
| 053 | [x] | P2 | enemy | Robot charge permanent until wall, not timed exit to walking | Robot charge matches ASM |
| 054 | [x] | P3 | enemy | MiniHoppingSkull ground delay 21 not 32 (dbf #$14) | Skull delay matches ASM |
| 055 | [x] | P3 | enemy | BagelBrothers projectile speed 3 not 4, lifetime 50 not 45 | Projectile matches ASM |
| 056 | [x] | P3 | enemy | BigHoppingSkull is continuous bouncer, not discrete hopper | Continuous bouncer from ASM: $FA0 gravity, wall/floor/ceiling/slope deflection |
| 057 | [x] | P3 | enemy | Goat player-tracking callback missing (turns toward player >32px) | Goat tracks player |
| 058 | [x] | P3 | enemy | Cloud 60-frame post-bounce idle period missing | Cloud idles after bounce |
| 059 | [x] | P3 | enemy | Fireball ceiling bounce uses Sphere formula, should be random+cap | Fireball bounce matches ASM |
| 060 | [x] | P2 | enemy | Boss multi-head system (Shishkaboss/BoomerangBosses/BagelBrothers) | 3 heads per boss with ASM offsets + per-head HP |
| 061 | [x] | P3 | enemy | Orca missing ceiling bounce, slope handling, X accel during lunge | All 3 from ASM: ceiling bounce, slope re-lunge, X accel $200/frame |
| 062 | [~] | P3 | enemy | HeadyMetal death wobble visual (rendering-only) | ASM-timed wobble death effect wired; still needs oracle-backed visual verification |
| 063 | [x] | P2 | block | Ghost block restored with collision 7 (Spikes) not 6 (Solid) | Ghost blocks walkable |
| 064 | [x] | P2 | block | Drill block projectile fired downward, should be horizontal ±4px/frame | Drill projectile horizontal |
| 065 | [-] | P3 | block | Shooter fire: ASM uses per-block registration on visibility, native uses global timer | DEVIATION — structural: ASM EvanescentBlock_Trigger per-block, native global timer |
| 066 | [-] | P3 | block | Mushroom bounce uses direct player velocity; ASM bounces platform object | DEVIATION — structural: ASM uses SoftPlatform objects, native bounces player inline |
| 067 | [x] | P3 | block | Lift block uses accel physics; ASM uses coroutine animation | Lift matches ASM: 2px/frame×8 up, erase, 1px/frame×16 down |
| 068 | [-] | P3 | block | Shifting block grid swap is instant (correct); visual uses sub_113C0 sprite | DEVIATION — visual only: ASM animates sprite with shared lift coroutine |
| 069 | [x] | P2 | fidelity | TAS boot slice desync: native consumed early boot/title Start presses on the wrong frames | First 250-frame boot slices of 3 imported TAS recordings show 0 authoritative diffs |
| 070 | [x] | P1 | fidelity | TAS frame 956: post-ledge-snap movement state suppression | 21-frame pull-up animation cooldown (stru_8BB4), frontier now 986 |
| 074 | [x] | P1 | fidelity | TAS frame 986: sub-pixel X drift from false wall probe | Tile-boundary guard (sub_8F26), frontier now 987 |
| 075 | [x] | P1 | fidelity | TAS frame 987: wall-cling h_accel suppression (loc_AA22) + tile-boundary guard refinement | wall_cling_direction field gates h_accel; removed distance>0 workaround; frontier now 988 |
| 076 | [x] | P1 | fidelity | TAS frame 988: spurious Cyclone form change + ceiling bounce velocity | Deferred prize collection + zero ceiling y_vel; frontier now 1056 |
| 077 | [x] | P1 | fidelity | TAS frame 1056: prize item physics + transform freeze + wall fixup | ASM-faithful FloatingItem, per-form half-width, sub_7B30 wall adjustment, physics suppression during Kid_Transform; frontier now 1129 |
| 078 | [x] | P1 | fidelity | TAS frame 1129: demo input freeze during transform + left-accel vel=0 | Demo data pointer freeze during Kid_Transform (loc_C22 not called in coroutine); apply_holding_left vel=0 uses strong_brake not run_accel (bpl semantics); frontier now 1144 |
| 079 | [x] | P1 | fidelity | TAS frame 1144: per-form collision half-width (FA78/unk_B408) | ASM sub_914A uses FA78 (per-form from unk_B408) for floor/wall/ceiling probes; native used fixed PLAYER_HALF_WIDTH=5. Cyclone (hw=7) walked off ledge 1 frame early due to narrower floor probe. Threaded form.half_width() through all collision functions; frontier now 1145 |
| 080 | [x] | P1 | fidelity | AsmFaithful Phase 2: sub_902A before sub_8F26 position split | ASM floor collision (sub_902A) runs before horizontal (sub_8F26), seeing old x_pos. Split apply_velocity_to_position into y-then-x with collision between. Made AsmFaithful the default PhysicsOrder. Remaining frame 1145 divergence: walking coroutine floor-ahead check (sub_ADB8) |
| 081 | [~] | P2 | fidelity | Walking floor-ahead check (sub_ADB8) | ASM walking coroutine checks floor at post-sub_8F26 x via sub_ADB8. On walkoff: reverts x_pos/x_vel, transitions to jump, skips 1 frame of h-physics. Native has no equivalent — needs sub_ADB8 probing logic matched |
| 071 | [-] | P3 | infra | AsmFaithful Phase 2: floor collision self-applies y_pos (sub_902A) | DONE — resolved in #080 |
| 072 | [-] | P3 | infra | BlastEm second oracle for cross-validation | DEFERRED — jgenesis sufficient, GDB stub approach documented in previous plan |
| 073 | [-] | P3 | infra | Instruction-level M68K trace from jgenesis | DEFERRED — oracle-probe memory watches are lighter; env var approach documented |

## Investigation notes

### 001 — Player stuck after damage (FIXED)
Root cause: `apply_damage()` did not clear `left_wall_sticky` / `right_wall_sticky`.
If the player was wall-grabbed when hit, they stayed pinned to the wall during
invulnerability frames. Fix: clear both wall-sticky fields in `apply_damage()`.

### 002 — Enemies don't die when stomped (FIXED)
Root cause: stomp detection used strict `previous_bottom <= stomp_e_top` position check,
requiring the player's feet to be precisely above the enemy's top edge on the prior frame.
The ASM uses a velocity-direction cross-product: `y_overlap × |rel_x_vel| < x_overlap × |rel_y_vel|`
to determine approach direction. Reimplemented to match ASM — stomps are now detected based
on vertical approach direction, not prior-frame position history.

### 003 — Audio (FIXED)
GEMS engine is fully wired: BGM per theme, SFX for jump/stomp/damage/death/form/block.
Audio init now reports specific SDL2 errors with troubleshooting hints (SDL_AUDIODRIVER,
audio support). Triggers BGM on first InGame entry. Pipeline verified correct.

### 004 — Palette (FIXED)
Root cause: theme-specific palette permutation tables were not applied. The ASM art
decompression remaps palette indices per theme (Forest, City, Mountain have unique tables).
Fix: load 16-byte permutation tables and reorder palette entries during level visual loading.

### 005 — Parallax (NOT A BUG)
Parallax IS working. bgscroll data loads correctly, `background_scroll_x_px()` applies
camera-relative rates. Effect is subtle (e.g. Forest: 26% camera speed) matching original.

### 006 — Dragon moonwalking (FIXED)
Root cause: `select_enemy_frame` used global `trace.frame_index` for ALL enemies,
cycling walk frames even when the dragon is in a stationary phase (perch/parked).
Fix: added `stationary` field to NativeObjectSnapshot, populated from
`DragonMotionPhase::is_stationary()`. Stationary enemies show frame 0 (idle).

### 010 — Enemy attacks (MOSTLY DONE)
11 enemy families + 4 bosses + 3 helmets + 3 environmental sources have projectile support.
Only gaps: UFO beam (child object system, spawns beam segments) and Fire Demon fire trails
(child objects spawned on death). Both require a child-object spawning system not yet built.
All simple projectile-based attacks are implemented.

### 011 — Bricks half-broken (FIXED)
Resolved by palette permutation fix (#004). Verified via capture-frames comparison —
block tiles render with correct colours matching original game.

### 013 — P-blocks (NOT A BUG)
Prize blocks vanish when hit (cell cleared to empty). This is correct per original behaviour.

### 015 — Wall grab (FIXED)
Added `left_wall_sticky` field and mirrored right-wall logic in `apply_wall_collision()`.
Both directions now support airborne wall-sticky with overhang top-edge retention.

### 017/019 — Helmets and form transform (FIXED)
`apply_prize_content` → `queue_form_change` → `resolve_pending_form_change` chain is wired
and verified. Forms change player hitbox, speed table, and abilities. FloatingItem spawns
on prize hit with content-type-specific colours. Unit tests confirm IronKnight and
Juggernaut transforms trigger correctly with proper hitpoint assignment.

### 018 — Form-change animation (PLACEHOLDER + RESEARCH)
Alternating white/yellow flash overlay for 8 frames after form transform.
ASM traced: `Kid_Transform` (kid.asm:8369-8607) uses per-helmet sprite+palette
cycling tables (stru_7EFA..stru_8064). Full implementation needs ROM sprite tile
extraction for the spinning animation frames.

### 020 — Water shimmer (DONE)
Real ASM ripple table from `BackgroundScroll_ForestWaterRippleData` (kid.asm:34310).
64-byte signed lookup, max 8px amplitude, phase = `Time_Frames >> 2 & 0x3F`.

### 021 — Sound Test (FIXED)
Options menu cursor==1 now enters Sound Test screen. Left/Right ±1, Up/Down ±16,
A/C plays track via GEMS engine, B returns to options.

### 022 — Duplicate Game Over overlay (FIXED)
Scene rendering had two `if trace.game_over` blocks (lines 1023 and 1091).
Removed the duplicate.

### 036 — Tank walk speed step (FIXED)
Tank.asm lines 52-64 use a 3-tier fall-through chain: HP=2→$8000, HP=3→$10000,
HP=4→$18000. This gives step=$8000. Was incorrectly $10000 in `pacer_walk_speed()`.
All other pacer families verified correct: Goat/Armadillo/Crab ($10000), Driller ($8000),
Scorpion (fixed $18000), FireDemon/Robot/Ninja ($8000).

### 037 — Floor-snap misses terrain (PARTIALLY FIXED)
Gravity simulation in `level_respawn_slice_with_floor_snap` fixed most levels but a small tail remains:
- **Level 0x52**: native `camera_y=0x00B30000`, oracle `0x00B50000` (+2px);
  native `kid_y=0x014F0000`, oracle `0x01514000` (+2.25px). The wide spawn-support
  fallback still snaps onto support outside the narrow hitbox on frame 1.
- **Level 0x29**: native `camera_x=0x02040000`, oracle `0x02090000` (+5px);
  native `kid_x=0x02A00000`, oracle `0x02A50000` (+5px). Slope-slide during gravity settling
  still accumulates X drift the simplified init path doesn't reproduce.
- **Levels 0x24, 0x51**: 1px `camera_y` dead-zone rounding differences remain.
- **Level 0x02**: now clean apart from the usual advisory `number_lives` field.
Fix: replace the remaining spawn-support shortcut with an init path that reproduces the
real frame-step collision/camera convergence, not a widened probe. Current diffs are all
classified advisory/oracle-limited in the focused sweeps, but they are still fidelity debt.

### 038 — Camera Y settling dead-zone (FIXED)
8 levels showed +1 to +7px camera_y diffs at level_settle. The asymmetric dead-zone
in `update_camera_axis` (`(target-camera+4)>>3` rounds to 0 within ±4px) meant the
final camera position depended on approach direction. Floor-snap placed the player
instantly at the floor, so the camera approached from a different direction than the
oracle (which tracks the player frame-by-frame during real gravity fall).
Fix: simulate gravity fall + camera follow in `level_respawn_slice_with_floor_snap` —
replay the player falling from header_y to floor_y under gravity, updating the camera
each step. This matches the oracle's convergence path. Fixed all 8 levels.

### 039 — 4th speed table entry (NOT A BUG)
Normal terrain table (unk_9C0A) has 4 entries of 7 longwords each ($70 bytes total),
but Ice (unk_9C7A) and Rubber (unk_9CCE) only have 3 entries ($54 bytes each).
ASM selection at loc_9AFC checks Micromax (+$1C) and Juggernaut (+$38) independently.
The 4th entry at offset +$54 is only reachable if BOTH checks matched simultaneously,
which can't happen (one helmet at a time). Dead data — likely a cut form or dev remnant.
Values: $20000/$4000/$2800/$4000/$30000/$3800/$6000 (similar to Juggernaut but higher accel).

### 049 — Level 0x29 X position diff (MERGED WITH #037)
5px consistent X diff caused by slope-sliding during gravity settling. Same root cause
as #037 — native simplified gravity simulation doesn't include full physics step.
Fix tracked in #037.

### 050 — Idle-tail position diffs on 4 levels (INVESTIGATING)
Levels 0x00, 0x45, 0x48, 0x61 show wildly different positions at idle_tail (frame ~359).
- 0x00: game_mode native=InGame(0x0C) oracle=LevelTransition(0x08) — oracle's warp
  to level 0 is slower, still in level-name-card at frame 359
- 0x45, 0x61: native positions zeroed out (player fell off map or died)
- 0x48: large position divergence (different death timing)
Root cause: combination of oracle warp timing differences and enemy interaction timing.
These are not simple constant fixes — they require either matching oracle warp timing
exactly or increasing settle frames in scenarios.

### 069 — TAS boot slice desync (FIXED)
Imported BK2 recordings exposed a boot-only fidelity gap that the per-level warp sweep
could not see. Native was exposing gameplay-looking state on frame 0, title readiness
was modelled with a global six-Start shortcut, and the first intro segment accepted
Start too early. Fixes:
- cold boot trace now keeps gameplay-visible state zeroed on frame 0
- title readiness is phase-local instead of keyed off cumulative Start count
- intro segment 1 has an oracle-backed early-Start dead zone plus direct-to-title skip
- TAS reports now include boot-phase diagnostics and checked-in 250-frame boot-slice fixtures

Verification:
- `kid_chameleon_short_demo` first 250 frames: 0 authoritative diffs
- `kc_any_percent_2024` first 250 frames: 0 authoritative diffs
- `any-percent-published` first 250 frames: 0 authoritative diffs
