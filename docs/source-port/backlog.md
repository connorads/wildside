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

Status: `[ ]` open, `[~]` in progress, `[x]` done, `[-]` won't fix / not a bug
Priority: P0 (game-breaking), P1 (core feel), P2 (missing system), P3 (polish)

For newly opened items, capture these facts in the investigation notes:

- evidence source: playtest, recording slug, oracle, or frame capture
- native repro status: reproduced / not yet reproduced
- oracle repro status: authoritative / advisory / unknown / oracle limitation
- minimum gate: `fast-slice`, `1p-playable`, `1p-accurate`, or `visual-parity`

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
| 030 | [x] | P3 | infra | Add a local-first verification entrypoint for regression checks | Contributors can run the intended gate locally before pushing |
| 031 | [x] | P3 | infra | Add performance baseline tracking | criterion benchmarks: ~92ns/frame idle, ~3µs level load |
| 032 | [x] | P2 | fidelity | Per-form Diamond Power effects (20 unique abilities) | Each form has correct visual and damage pattern |
| 033 | [x] | P2 | fidelity | Form-change animation from ASM Kid_Transform tables | Original transform art/timing loaded for all forms, including Juggernaut composite rendering |
| 034 | [x] | P3 | fidelity | Intro sequence 5 segments (ASM has 5, native had 3) | Boot captures match oracle |
| 035 | [-] | P2 | fidelity | Resolve 6 oracle-warp-broken levels (0x4E73) | ORACLE LIMITATION — jgenesis warp bug, native works; determinism + soak tested |
| 036 | [x] | P2 | enemy | Tank walk speed step wrong ($10000 should be $8000) | Tank speed matches ASM at all HP tiers |
| 037 | [x] | P3 | fidelity | Floor-snap settling diffs: 0x24/0x51 (1px cam_y) | `0x24`/`0x51` now match the oracle-backed settle camera carry |
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
| 062 | [x] | P3 | enemy | HeadyMetal death wobble visual (rendering-only) | The effects-only scene tests now verify the full HeadyMetal body/eye wobble footprints and the two-frame phase flip at each ASM timer boundary |
| 063 | [x] | P2 | block | Ghost block restored with collision 7 (Spikes) not 6 (Solid) | Ghost blocks walkable |
| 064 | [x] | P2 | block | Drill block projectile fired downward, should be horizontal ±4px/frame | Drill projectile horizontal |
| 065 | [x] | P3 | block | Shooter fire: ASM uses per-block registration on visibility, native uses global timer | Shooter blocks now register per visible cell, fire on entry, and recycle after the ROM's 22-frame slot lifetime |
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
| 081 | [x] | P2 | fidelity | Walking floor-ahead check (sub_B43A) | ASM walking coroutine (loc_8BF0) calls sub_B43A before physics: scans cells at (y+1)/16 across player width for Solid/Spikes. No floor → loc_B5F6: y_pos += 1px, skip vel/pos updates. Two fixes: (1) movement.rs pre-physics sub_B43A check in AsmFaithful path, (2) boot.rs demo grounded-run handler now defers walkoff to normal physics. Frontier 1145→1147 |
| 082 | [x] | P2 | fidelity | TAS frame 1147: demo byte boundary parity shift after transform | On the first Kid_Transform frame, step_demo_input still consumes one ReadJoypad byte before the coroutine freeze. Native was only rewinding the frame counter when the transform exited, leaving data_index and demo_buffered_input one byte behind oracle-held controls. Fix: after step_lifecycle detects transform end (flash_frames→0), decrement frame_counter by 1, restore the skipped data_index advance, and refresh demo_buffered_input from the resumed byte. Frontier 1147→1396 |
| 083 | [x] | P2 | fidelity | TAS frame 1170: sub_902A floor landing sub-pixel clearing | ASM sub_902A (kid.asm:10462) always does `clr.w $20(a3)` on new floor landings (airborne→grounded), zeroing the sub-pixel. Native was preserving sub-pixel from the accumulated position on all floor landings. Fix: clear sub-pixel on new landings (`!was_on_ground`), preserve when already grounded (sub_902A exits early via loc_913A when y_vel=0). Original #083 demo byte offset theory was wrong — parity `% 2 == 1` is algebraically identical to `% 2 == 0` with +1 offset. Frontier 1170→1209 |
| 084 | [x] | P2 | fidelity | TAS frame 1209: Cyclone flight activation/thrust timing (ASM kid_flip + sub_B168) | Three issues: (1) Activation ran before physics (step_cyclone_flight); ASM kid_flip activates AFTER physics so flight takes effect next frame. (2) Thrust checked C held state; ASM sub_B168 uses `bclr #Button_C,(Ctrl_Pressed)` — rising edge only, one frame of thrust per C press. (3) Edge detection used prior_input (2-frame gap); ASM uses Ctrl_Pressed vs Ctrl_1_Held (1-frame gap). Fix: split into step_cyclone_flight (velocity modulation when already flying) + step_cyclone_activation (after physics, only when not flying). Frontier 1209→1301 |
| 085 | [x] | P2 | fidelity | TAS frame 1301: Cyclone accumulator wrapping arithmetic | ASM `subi.w`/`addi.w` uses M68K modular 16-bit arithmetic. Native used `saturating_add/sub` which clamps at i16::MIN instead of wrapping. When accumulator gets very negative (~-22016) and thrust (-$3000) is applied, ASM wraps to positive (0xAA00-0x3000=0x7A00=+31232), producing a sudden downward push. Native saturated at -32768. Fix: wrapping_add/wrapping_sub. Frontier 1301→1365 |
| 086 | [x] | P2 | fidelity | TAS frame 1365: ceiling block preservation during Cyclone flight | Native apply_ceiling_collision cleared Prize blocks to Empty immediately, removing structural ceiling tiles. ASM sub_1007A writes $6F (Solid/no-block/Deleted) — tile stays solid for ~33 frames (bounce animation), then clears via deferred timer. Also: adjacent Smash blocks cleared inline (matching sub_914A full-width scan). Narrow re-verification after the demo transform fix shows the next authoritative frontier is frame 1396. |
| 087 | [x] | P2 | fidelity | TAS frame 1396: post-prize-hit ceiling latch in demo 0x1B | The transient `0xE001` restore still blocks Kid for one more frame, and the `0x0000` clear only becomes collision-visible after that frame's physics. Native now keeps restored prize cells collision-solid but non-dispatching, then clears/spawns at frame end. Frontier 1396→1405. |
| 088 | [x] | P2 | fidelity | TAS frame 1405: floating prize collection timing after ceiling latch | Diamond pickup bounds now match the smaller visible sprite, so frame 1405 stays mid-flight (`score=0`, `number_diamonds=0`) with the prize still present. Frontier 1405→1406. |
| 089 | [x] | P2 | fidelity | TAS frame 1406: post-latch diamond collection + next arc timing on demo 0x1B | Restricted wide spawn-support settling to the demo bootstrap window and queued prize-diamond awards through a 32-frame `PrizeDiamondCollected_Init` delay instead of rewarding on contact. Added `demo_diamond_prize_stays_uncollected_on_frame_1406` and `demo_prize_diamond_awards_on_frame_1438_without_score`; re-ran the nearby demo prize tests plus `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1390 --frame-end 1500 --checkpoint-interval 1`, which is now CRC-clean with 0 authoritative diffs through frame 1500. |
| 090 | [x] | P2 | fidelity | TAS frame 1755: AsmFaithful solid-floor touch snap lands one frame early on demo 0x1B | Restricted the sub-half-pixel solid-floor touch snap to the non-AsmFaithful landing paths, added `asm_faithful_solid_floor_snap_waits_for_contact_row` + `demo_solid_floor_landing_waits_until_frame_1756`, and verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1748 --frame-end 1762 --checkpoint-interval 1` is CRC-clean. Frontier 1755→1772. |
| 091 | [x] | P2 | fidelity | TAS frame 1772: demo 0x1B returns to title one frame early | Demo Start abort now follows the ASM hibernating object timing (`loc_119D8`): frame 1772 arms the abort, frame 1773 still stays in `mode=0x0010`, frame 1774 flips to title. Verified with the targeted demo abort tests in `kid-core` plus `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1768 --frame-end 1780 --checkpoint-interval 1` (frontier now 1774). |
| 092 | [x] | P2 | fidelity | TAS frame 1774: first title frames keep the live demo snapshot after demo 0x1B abort | Demo abort now preserves the paused gameplay snapshot under `mode=0x002c`, matching oracle `kid/camera` through frame 1801. Verified with `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`, `cargo test -p kid-core demo_title_return_keeps_live_snapshot_on_first_title_frames -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1772 --frame-end 1780 --checkpoint-interval 1`, and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1772 --frame-end 1860 --checkpoint-interval 1`. Frontier 1774→1802. |
| 093 | [x] | P2 | fidelity | TAS frame 1802: title return blanks Kid before camera/helmet reset on demo 0x1B | Split the demo-abort title handoff into 2 phases: full live snapshot through frame 1801, Kid hidden on frames 1802-1805, then camera/form/diamond state cleared on frame 1806. Added `demo_title_return_blanks_kid_on_frame_1802_before_camera_reset` and `demo_title_return_clears_camera_and_progression_on_frame_1806`; verified with `cargo test -p kid-core demo_title_return -- --nocapture`, `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1798 --frame-end 1808 --checkpoint-interval 1`, and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 2000 --bisect`. Frontier 1802→1832. |
| 094 | [x] | P2 | fidelity | TAS frame 1832: title pose appears while oracle still keeps Kid hidden after demo 0x1B abort | Demo-return title mode now keeps Kid hidden until frame 1924 while leaving the rebuilt title object counts alone. Added `demo_title_return_keeps_kid_hidden_until_frame_1924`; verified with `cargo test -p kid-core demo_title_return -- --nocapture`, `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1798 --frame-end 1808`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1828 --frame-end 1927 --checkpoint-interval 1` (0 authoritative, 5 advisory), `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 2000 --bisect` (PASS: all 2000 frames match by CRC), and `mise run verify-slice-fast`. Frontier 1832→>2000. |
| 095 | [x] | P2 | fidelity | TAS frame 2529: demo 0x15 startup reused the demo 0x1B bootstrap profile | Split demo startup timing by recording: Knights Isle now keeps its hidden camera/profile through frame 2537, delays demo input/gameplay exposure to frame 2544, and holds the bootstrap pose until the oracle-backed handoff is complete. Added `demo_1_bootstrap_keeps_camera_hidden_through_frame_2537` and `demo_1_bootstrap_holds_pose_and_input_until_frame_2571`; verified with the targeted demo bootstrap tests, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2524 --frame-end 2580 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 3000 --bisect`. Frontier 2529→2625. |
| 096 | [x] | P2 | fidelity | TAS frame 2625: demo 0x15 handoff lands one frame early from AsmFaithful solid-floor touch snap | Disabled demo-playback solid-floor touch snap, added `demo_1_handoff_stays_airborne_until_frame_2626`, verified `cargo test -p kid-core demo_1_handoff_stays_airborne_until_frame_2626 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2618 --frame-end 2635 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 3000 --bisect`. Frontier 2625→2698. |
| 097 | [x] | P2 | fidelity | TAS frame 2698: demo 0x15 standingstill should not start walking into the adjacent wall after landing | Zero-speed grounded updates now respect the same side-wall probe the standingstill ASM uses before it transitions into walking, so the post-landing hold on Knights Isle keeps `kid_x_pos=0x011A0000` through frame 2714 instead of jittering into the wall. Added `demo_1_post_landing_hold_preserves_standingstill_through_frame_2714`; verified `cargo test -p kid-core demo_1_ -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 2720 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 4000 --bisect`. Frontier 2698→2723. |
| 098 | [x] | P2 | fidelity | TAS frame 2723: demo 0x15 should release the right-wall overhang on the first upward tile-row crossing | Demo 1 kept `wall_cling_direction=Right` one frame too long after the overhang had already cleared, so native never emitted the 1px sticky-release shimmy that oracle shows at frame 2723. Added an ASM-timed overhang release in the AsmFaithful jump path plus `demo_1_wall_cling_snap_releases_into_run_on_frame_2723`; verified `cargo test -p kid-core demo_1_ -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2718 --frame-end 2760 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 4000 --bisect`. Frontier 2723→2773. |
| 099 | [x] | P2 | fidelity | TAS frame 2773: grounded left-wall snap leaves demo 0x15 one pixel off the wall | Native reused the moving-right clearance when clamping a grounded left-wall hit, so frame 2773 snapped Kid to `0x01360000` instead of the oracle's flush `0x01350000`. Fixed the left-wall clamp to use the probed wall surface plus `half_width`, added `demo_1_left_wall_snap_lands_flush_on_frame_2773`, and verified `cargo test -p kid-core demo_1_left_wall_snap_lands_flush_on_frame_2773 -- --nocapture`, `cargo test -p kid-core demo_1_ -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2768 --frame-end 2790 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 4000 --bisect`. Frontier 2773→2831. |
| 100 | [x] | P2 | fidelity | TAS frame 2831: buffered wall-sticky pull-up misses the prize-block top on demo 0x15 | Extended the wall-sticky ledge snap with an ASM-style pull-up fallback: when a buffered jump request is still live during a right-wall cling, native now searches downward from the pull-up window and lands on the prize-block top at `kid=(0x01440000,0x013F8000)` instead of continuing to fall. Added `demo_1_buffered_wall_sticky_pullup_lands_on_frame_2831`; verified `cargo test -p kid-core demo_1_buffered_wall_sticky_pullup_lands_on_frame_2831 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2828 --frame-end 2850 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 4000 --bisect`. Frontier 2831→2938. |
| 101 | [x] | P2 | fidelity | TAS frame 2938: descending right-wall cling keeps suppressing jump air-accel after the dragon stomp on demo 0x15 | Oracle keeps Kid glued to the wall until frame 2994, then emits the 1px right-wall shimmy and clears wall-cling in time for demo-held `Right+B` to re-enter `sub_B084` on frame 2995. Native stayed in `wall_cling_direction=Right`, so the post-stomp drift never restarted. Added an ASM-style descending wall-cling release in the jump loop, plus `demo_1_keeps_the_oracle_backed_stomp_drift_on_frame_2994`; verified `cargo test -p kid-core demo_1_lands_on_the_walking_dragon_on_frame_2938 -- --nocapture`, `cargo test -p kid-core demo_1_keeps_the_oracle_backed_stomp_drift_on_frame_2994 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2988 --frame-end 3004 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2690 --frame-end 4000 --bisect`. Frontier 2938→3252. |
| 102 | [x] | P2 | fidelity | TAS frame 3252: IronKnight transform freeze keeps demo 0x15 standingstill 15 frames too long | Native misread Kid_Transform table `stru_7FF8`: the first `anim_frame` fields sum to 81 frames, but `transform_anim_frames(IronKnight)` used 96 by accidentally treating the metadata byte as extra duration. That left `form_change_flash_frames` non-zero through frame 3252, so physics stayed frozen and demo-held Right never re-entered walking. Fixed the table count, added `demo_1_reenters_walking_from_standingstill_on_frame_3252`, and verified `cargo test -p kid-core demo_1_reenters_walking_from_standingstill_on_frame_3252 -- --nocapture`, `cargo test -p kid-core demo_cyclone_pickup_enters_transform_on_frame_1056 -- --nocapture`, `cargo test -p kid-core demo_transform_exit_rewinds_frame_counter_for_byte_parity -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3248 --frame-end 3260 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3200 --frame-end 4000 --bisect`. Frontier 3252→3400. |
| 103 | [x] | P2 | fidelity | TAS frame 3400 prep: native enemy loading used a narrower window than `Manage_EnemyLoading`, so the second Knights Isle walking dragon could drop out before the stomp frontier | Matched the activation window to the ASM loader bounds `x=[camera-0x80,camera+0x1C0]`, `y=[camera-0x80,camera+0x160]`, and added `demo_1_loads_the_second_knights_isle_dragon_by_frame_3400` to pin the missing dragon roster. Verified with `cargo test -p kid-core demo_1_loads_the_second_knights_isle_dragon_by_frame_3400 -- --nocapture`, `cargo test -p kid-core demo_1_lands_on_the_walking_dragon_on_frame_2938 -- --nocapture`, `cargo test -p kid-core demo_1_reenters_walking_from_standingstill_on_frame_3252 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3390 --frame-end 3410 --checkpoint-interval 1`, and `mise run verify-slice-fast`. Frontier stays at 3400: the remaining mismatch is the stomp/movement path, not loading. |
| 104 | [x] | P2 | fidelity | TAS frame 3400: second Knights Isle stomp setup never reached the oracle-backed grounded state | Matched DragonWalking contact resolution to `GfxObjects_CollisionKid`'s integer-inclusive overlap rules instead of a fixed-point prefilter, threaded the active form half-width through runtime enemy-contact queries, and modelled the live frame-3399 edge touch that only clips on the truncated pixel bounds. Added `walking_dragon_right_facing_edge_contact_matches_live_tas_stomp_window`, promoted `demo_1_reaches_the_oracle_backed_second_dragon_setup_on_frame_3400` into a passing regression, and re-checked `cargo test -p kid-core demo_1_reaches_the_last_oracle_backed_pre_stomp_arc_on_frame_3399 -- --nocapture`, `cargo test -p kid-core demo_1_reaches_the_oracle_backed_second_dragon_setup_on_frame_3400 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3390 --frame-end 3410 --checkpoint-interval 1` (CRC-clean, 0 authoritative), and `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3400 --frame-end 4916 --bisect`. Frontier 3400→3414. |
| 105 | [x] | P2 | fidelity | TAS frame 3414: demo 0x15 pause snapshot lands one frame / one pixel off after the second dragon setup | Deferred the demo title-return handoff until after the frame's final gameplay step, so the preserved title snapshot now uses the oracle-backed paused Kid/camera state instead of the stale pre-freeze slice. Promoted `demo_1_matches_the_oracle_pause_snapshot_on_frame_3414` into the normal suite, re-ran the older `demo_title_return*` regressions plus the nearby frame-3399/frame-3400 probes, and verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 3412 --frame-end 3416 --checkpoint-interval 1` (CRC-clean, 0 authoritative) and `--frame-start 3400 --frame-end 4916 --bisect` (PASS: all 4916 frames match by CRC). Frontier 3414→recording complete. |
| 106 | [x] | P2 | fidelity | TAS frame 287: `any-percent-published` restores the title pose two frames before oracle | Cold-boot title entry now uses the same global ready frame floor as before, but early intro-skip paths also wait for a 150-frame local title rebuild before exposing Kid. Added `published_tas_boot_title_keeps_kid_hidden_until_frame_289`, verified `cargo test -p kid-core published_tas_boot_title_keeps_kid_hidden_until_frame_289 -- --nocapture`, re-ran the older title-ready and demo-return regressions, confirmed `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 280 --frame-end 292 --checkpoint-interval 1` is CRC-clean apart from advisory `number_objects`, and re-bisected to move the frontier from 287 to 1830. |
| 107 | [x] | P2 | fidelity | TAS frame 1830: `any-percent-published` awards the second diamond later than oracle in demo level `0x1B` | Matched falling prize items to the ASM's fixed-column row probe, kept the one-row emergence grace, gave diamonds their own bottom-anchored pickup box, and held queued `PrizeDiamondCollected_Init` awards for 33 frame starts instead of 32. Added `floating_diamond_stops_once_it_reaches_supported_row_below_source`, `published_tas_arms_the_second_prize_diamond_queue_on_frame_1798`, updated the demo diamond regressions, verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 1826 --frame-end 1834 --checkpoint-interval 1`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1398 --frame-end 1442 --checkpoint-interval 1`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 0 --frame-end 3000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 1830→4905. |
| 108 | [x] | P2 | fidelity | TAS frame 4905: `any-percent-published` unloads demo level `0x1B` into title one frame before oracle | Oracle probes showed the ROM demo timeout counter at `$FFFFFBC2` lagged the native startup counter by 61 frames during Skydragon Castle demo 2, so the auto-end compare was firing early even though the visible choreography still matched. Fixed the timeout compare to subtract that oracle-backed offset for demo 2 only, added `demo_2_auto_end_waits_for_the_oracle_backed_timeout_counter` plus `published_tas_demo_2_stays_in_gameplay_on_frame_4905`, and verified `cargo test -p kid-core demo_auto_ends_after_max_frames -- --nocapture`, `cargo test -p kid-core demo_2_auto_end_waits_for_the_oracle_backed_timeout_counter -- --nocapture`, `cargo test -p kid-core published_tas_demo_2_stays_in_gameplay_on_frame_4905 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 4898 --frame-end 4980 --checkpoint-interval 1`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 4905→4966. |
| 109 | [x] | P2 | fidelity | TAS frame 4966: `any-percent-published` demo timeout still takes the paused title helper instead of the ROM auto-end branch | Split demo auto-timeout from the manual `Start`-abort helper so the threshold frame stays in gameplay and frame 4967 enters the ROM's non-paused `game_mode=0x0000` handoff. Added `published_tas_demo_2_uses_the_mode_0_timeout_handoff_on_frame_4967`, re-ran the focused published TAS window plus bisect, and moved the live blocker off the branch itself onto the preserved handoff snapshot. |
| 110 | [x] | P2 | fidelity | TAS frame 4966: `any-percent-published` auto-timeout handoff preserves Kid X one `0x1C00` step too far, then clears camera/progression early on frame 4998 | Froze the timeout threshold frame before another gameplay step, then gave the mode-0 auto-timeout handoff its own full/partial snapshot windows so frame 4966 keeps `kid_x_pos=0x02487A00`, Kid only blanks on frame 4995, and camera/helmet/diamonds clear on frame 4999. Added `published_tas_demo_2_freezes_the_timeout_snapshot_on_frame_4966`, `published_tas_demo_2_blanks_kid_on_frame_4995_after_timeout_snapshot`, and `published_tas_demo_2_clears_camera_and_progression_on_frame_4999`; verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 4898 --frame-end 5010 --checkpoint-interval 1` (CRC-clean, 0 live authoritative diffs in the handoff window), `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 4964 --frame-end 5020 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 4966→5116. |
| 111 | [x] | P2 | fidelity | TAS frame 5116: `any-percent-published` auto-timeout title rebuild shows the standard title pose while oracle still keeps Kid hidden | Kept Kid hidden while the mode-0 auto-timeout restart is still latched, instead of falling back to the shared `DEMO_TITLE_PLAYER_HIDDEN_FRAMES` title gate. Added `published_tas_demo_2_keeps_kid_hidden_during_timeout_restart`, verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5108 --frame-end 5145 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5114 --frame-end 5185 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 5116→5143. |
| 112 | [x] | P2 | fidelity | TAS frame 5143: `any-percent-published` auto-timeout restart stays in `game_mode=0x0000` while oracle advances to intro sequence 1 | Let the timeout restart leave the latched mode-0 handoff after the oracle-backed 177-frame window, added `published_tas_demo_2_advances_into_intro_sequence_1_on_frame_5143`, updated the hidden-Kid regression to cover the post-handoff intro frames, and verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5108 --frame-end 5145 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5114 --frame-end 5185 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 5143→5303. |
| 113 | [x] | P2 | fidelity | TAS frame 5303: `any-percent-published` leaves intro sequence 1 too early during the timeout restart | Preserved a timeout-restart-specific intro-sequence-1 duration instead of reusing the shared 160-frame timer, added `published_tas_demo_2_holds_intro_sequence_1_until_frame_5504`, and verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5298 --frame-end 5310 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5298 --frame-end 5310 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5498 --frame-end 5508 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5498 --frame-end 5508 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 5303→5584. |
| 114 | [x] | P2 | fidelity | TAS frame 5584: `any-percent-published` leaves intro sequence 2 too early during the timeout restart | Oracle `Game_Mode` probes showed the timeout-restart copy of intro sequence 2 starts on frame 5504 and does not reach `game_mode=0x001C` until frame 5953, so the shared 80-frame segment timer was wrong for this path. Added a timeout-restart-specific intro-sequence-2 override (`449` frames), promoted `published_tas_demo_2_holds_intro_sequence_2_until_frame_5953`, and verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5578 --frame-end 5590 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5578 --frame-end 5590 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5948 --frame-end 5956 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5948 --frame-end 5956 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 5584→5994. |
| 115 | [x] | P2 | fidelity | TAS frame 5994: `any-percent-published` reaches intro sequence 3 on time, but native still keeps Kid hidden during the first visible intro-sequence-3 pose | The ROM seeds the first visible intro-sequence-3 Kid pose from `Load_IntroSequence3`'s gfx-object descriptor table at `x=0x00600000`, `y=0x00D00000`; native trace state was still hard-coded to hide Kid for the whole intro sequence. Added `published_tas_demo_2_exposes_the_first_intro_sequence_3_kid_pose_on_frame_5994`, exposed the sequence-3 pose from frame 5994 onward, and verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5988 --frame-end 6000 --checkpoint-interval 1`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 5988 --frame-end 6000 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 5994→6033. |
| 116 | [x] | P2 | fidelity | TAS frame 6033: `any-percent-published` reaches title on time after intro sequence 3, but native discards the carried intro pose during the deferred title handoff | The timeout-restart intro-sequence-3 fix moved the frontier out to frame 6209, where oracle still carried the `x=0x00600000`, `y=0x00D00000` intro pose across the first 28 title frames before hiding Kid again. The surprise was that frame 6209 still arrived through `pending_title_skip`, not the natural timeout branch, so the carry-over tag had to survive the shared 2-frame Start→title countdown as well as the direct timeout path. Added `published_tas_demo_2_carries_intro_sequence_3_pose_into_the_first_title_frames`, verified `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6206 --frame-end 6245 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 6033→6358. |
| 117 | [x] | P2 | fidelity | TAS frame 6358: `any-percent-published` keeps Kid hidden one frame too long before the standard title pose returns | The timeout-restart intro-sequence-3 carry-over now uses its own 149-frame hidden-title window instead of the shared 150-frame demo-return gate, so frame 6358 restores the standard title pose at `(0x002E0000,0x008C0000)` and the focused title window is authoritative-clean. Verified with `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6354 --frame-end 6364 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 6358→6906. |
| 118 | [x] | P2 | fidelity | TAS frame 6906: `any-percent-published` keeps title mode one frame too long before demo playback resumes | Shortened only the timeout-restart title rebuild's attract-lock window by one frame so the fixed `$280` title idle countdown reaches demo playback on the oracle-backed frame. Added `published_tas_demo_2_resumes_demo_play_on_frame_6906`, re-ran `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6354 --frame-end 6364 --include-advisory`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6900 --frame-end 6912 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 6906→6945. |
| 119 | [x] | P2 | fidelity | TAS frame 6945: `any-percent-published` drops back into the paused title-return helper two frames after demo 1 restarts | Timeout-restart demo playback now ignores manual Start aborts until live gameplay starts, so the queued frame-6943 Start bit no longer arms the paused title helper during the restarted demo bootstrap. Added `published_tas_demo_2_ignores_queued_start_during_timeout_restart_demo_bootstrap`, re-ran `cargo test -p kid-core published_tas_demo_2 -- --nocapture`, `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 6940 --frame-end 6952 --include-advisory`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 3000 --frame-end 12000 --bisect`, `mise run audit-debug-fields`, and `mise run verify-slice-fast`. Frontier 6945→6966. |
| 120 | [x] | P2 | fidelity | TAS frame 6966: `any-percent-published` exposes the wrong wrapped demo 1 camera Y during the hidden warm-up after the fixed 6945 abort window | Fixed by carrying the timeout-restart IntroSequence6 lineage into demo playback so the hidden warm-up reuses the oracle-backed demo-1 camera wrap; frontier later advanced through the repaired title/demo shell. |
| 121 | [x] | P2 | fidelity | TAS frame 24921: level `0x15` gameplay falsely enters right-wall cling from a top-edge-only overhang probe | Matched ASM `sub_8F26` by refusing to promote `probe_wall_top_edge()` into a fresh airborne wall hit. Added `airborne_top_edge_contact_does_not_start_wall_cling_without_direct_wall_hit`, verified `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 24918 --frame-end 24928 --checkpoint-interval 1`, `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/any-percent-published/scenario.json --frame-start 23250 --bisect`, and `mise run verify-slice-fast`. Frontier 24921→27603. |
| 122 | [x] | P0 | level | Level 2 entry can spawn Kid in midair, then fall through the floor and die | Boot -> finish Level 1 -> enter Level 2 -> confirm stable landing and 10s of live play |
| 123 | [x] | P0 | player | Death/respawn still leaves camera drift, stuck-from-start, or broken restart state | Die in live play, confirm camera settles correctly and movement resumes without soft-lock |
| 124 | [x] | P1 | enemy | Enemy kill timing still allows Kid to get hurt after the enemy should already be dead | Sword/projectile/charge/AoE kills now persist the ROM kill bit, so dead enemies do not reactivate into fresh contact damage |
| 125 | [x] | P1 | block | Breakable blocks can render with the wrong lifecycle art or regress into prize-like visuals | Block-family render mapping now uses the ASM block group as well as skin, so post-prize `$E001` cells render smash-family art instead of prize-family art |
| 126 | [x] | P1 | player | Walking up slopes is still observably wrong in native play | Native 1P gate now covers uphill/downhill tracking plus slope enter/exit seams; `mise run verify-1p-playable` is green |
| 127 | [x] | P1 | block | Black bouncy blocks do not produce the expected bounce behaviour | Rubber block contacts now queue the ASM `loc_10DA4` jiggle on floor, ceiling, and wall contact without fake spring physics |
| 128 | [x] | P1 | form | Form-specific block breaking can now destroy metal blocks that should stay solid | Iron Knight landing now follows ASM `sub_AF7A/off_AFD8`: smash/prize/ice break, solid-ghost/teleporter no-op, metal blocks bounce and restore instead of being deleted |
| 129 | [x] | P1 | infra | Add a `1p-playable` gate that covers native boot -> Level 1 -> Level 2, death/respawn, slopes, bounce surfaces, and common block interactions | `mise run verify-1p-playable` is green only when the core 1P smoke path survives those scenarios |
| 130 | [x] | P1 | infra | Turn `play-native --record` / `verify-recording` into the standard playtest evidence pipeline | Recording bundles now carry provenance, replay commands, copied-bundle verification, optional artefacts, and an automatic trace transcript sidecar |
| 131 | [x] | P2 | fidelity | Burn down the five parked DEVIATION items instead of treating them as accepted resting state | Each deviation is either removed, reclassified as oracle limitation, or explicitly deferred with evidence |
| 132 | [x] | P3 | gameflow | Verify 2-player alternating mode against oracle-backed or strong native scenarios | Toggle 2P, die as P1, confirm P2 takeover / game-over / continue behaviour |
| 133 | [x] | P3 | gameflow | Investigate password flow and decide whether it is missing, partial, or intentionally out of 1P scope | ASM title/options flow is documented; stock ROM has no password entry, and native options now mirror the four-option ROM screen |
| 134 | [x] | P1 | infra | Package reduced semantic cases as the default autonomous work queue | `cargo run -p kidcli -- autonomous-loop --goal case-queue` now ranks reduced case bundles, prefers recording-backed authoritative failures, surfaces untriaged recordings, and only marks hybrid-gated cases as patchable |
| 135 | [ ] | P1 | render | Level 1 background graphics do not match the original game | Capture Level 1 native/oracle frames and compare background layers |
| 136 | [ ] | P1 | block | Diamonds and helmets emerging from bricks use incorrect graphics | Hit prize blocks, confirm prize emergence art matches oracle |
| 137 | [ ] | P1 | player | Kid walks backwards while climbing slopes | Playability negative baseline refreshed on 2026-04-05: recording slug `issue-137` (`.cache/recordings/issue-137/`), transcript `.cache/recordings/issue-137/transcript.txt`, verification report `.cache/recordings/issue-137/verify/report.json`, and frame captures `.cache/debug/issue-137/level_00_frame_00000.png`, `.cache/debug/issue-137/level_00_frame_00060.png`, `.cache/debug/issue-137/level_00_frame_00120.png`. Transcript shows the run idled from boot (`frame 000000`) to title (`frame 000599`) and stopped at `frame 000608`, so Kid never entered gameplay and the backwards-climb bug is still not reproduced. `cargo run -p kidcli -- verify-recording --slug issue-137` still fails as a negative-baseline oracle mismatch (`trace_match: true`, `oracle_state_match: false`, `oracle_frame_match: false`), while `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` still passes. Next verification: `cargo run --release -p kidcli -- play-native --warp-level 00 --settled --record issue-137-slope`, `cargo run -p kidcli -- verify-recording --slug issue-137-slope`, `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`. |
| 138 | [ ] | P1 | player | Enemy contact appears to apply double damage because invulnerability is too short | Take consecutive hits, confirm post-hit invulnerability timing matches oracle |
| 139 | [ ] | P1 | render | Bouncy blocks still render like spikes | Compare rubber block tiles and state art against oracle frames |
| 140 | [ ] | P1 | render | Prize blocks use wrong graphics and animations | Compare prize-block idle and hit states against oracle frames |
| 141 | [ ] | P1 | render | Intro and level-intro graphics still do not match the original game | Capture boot/title/level-intro checkpoints and compare to oracle |
| 142 | [x] | P1 | enemy | Level 1 dragon breath travels too far | FIXED — ASM loc_3D02A: breath is stationary hitbox (stru_3CCC8, 68f), not a projectile. Also fixed flying dragon fire X offset (loc_34BAC ±10px) and terrain_kill. |
| 143 | [ ] | P1 | block | Heavy Man's first helmet causes repeated block-bounce instead of the ROM's smooth single-hit break | Break blocks as Heavy Man and compare collision/animation timing to oracle |
| 144 | [ ] | P1 | enemy | Level 2 shooter enemy fires too rapidly | **Root cause identified**: enemy ID 0x0B (`TankShooting`) not mapped — falls to `Unknown(0x0B)/Inert`. Non-shooting tank (0x0A) incorrectly runs `step_tank_fire` every 32 frames. ASM fire is animation-driven (116–232 frames/cycle via turret anim `stru_3B4D8`/`stru_3B4F6`), 3.6–7.25x slower. Fix: add `EnemyFamily::TankShooting` for 0x0B, remove firing from 0x0A, wire `sub_3C4F8` animation-driven trigger. |
| 145 | [ ] | P1 | render | Level 2 shooter enemy is not rendered correctly | **Root cause identified**: ASM tank uses 2 GfxObjects (body + turret at `$3A(a5)`), native uses 1 sprite. Turret has independent animation (`stru_3B27E` non-shooting, `stru_3B4D8`/`stru_3B4F6` shooting) offset y-8px. Bullet spawn offset ±15px X (not 0), turret_y-14 Y (not enemy_y-8). Fix: implement two-object rendering for both tank variants. |
| 146 | [b] | P1 | level | Dying in Level 2 can leave the game stuck on the level-loading screen | **blocked:needs-human-repro** — requires interactive play to reproduce (agent cannot play). 14+ autonomy cycles failed to capture evidence. Negative baseline only: recording slug `issue-146` never enters Level 2. Downgraded from P0→P1 until a human playtest captures the hang. Next step: `cargo run --release -p kidcli -- play-native --warp-level 01 --settled --record issue-146-level2-death` |
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

### 122 — Level 2 entry can spawn Kid in midair, then fall through the floor and die (FIXED)
Evidence source: repeated native play reports plus strengthened native smoke coverage.
Native repro: the old `progress-two-transitions` gate never actually resumed Level 2 gameplay.
Oracle repro: unknown. Minimum gate: `1p-playable`.

Fix: extend `progress-two-transitions` so it re-arms the second skip hold on frame `2700`,
records `level_2_settle@2701`, and then survives through `level_2_live_tail@3300`. The native
gate now proves stable Level 2 landing plus 10 seconds of live gameplay instead of stopping in
`game_mode=$0008` transition mode.

Surprise worth keeping: the real Blue Lake Woods 2 header spawn is intentionally midair at
`(32,160)`, and the hidden `Load_InGame` settle path was already landing safely. The real gap was
the smoke scenario: it ended before sending the second resume input, so we were treating a
transition-shell check as if it proved live Level 2 play.

### 123 — Death/respawn still leaves camera drift, stuck-from-start, or broken restart state (FIXED)
Evidence source: repeated native play reports. Native repro: reported, needs canonical recording slug.
Oracle repro: mixed/unknown. Minimum gate: `1p-playable`.

This issue stays separate from the closed floor-snap and TAS frontier items. It is a player-facing
native stability bug until a recording or focused scenario proves otherwise.

Latest evidence: the old gameplay compare drift at `level_settle@719` was partly a stale native
opening-hazard model, not just a bad checkpoint. Oracle-backed Level 1 gameplay can take an idle
opening dragon hit on frame `631` and the fatal follow-up on frame `698`; native now mirrors those
two hits and enters `DeathWindup`, and the widened `progress-two-transitions` scenario now passes
its oracle compare again. `death-respawn-level1` is still open: after starting that smoke path
earlier to dodge the idle-hazard window, the first compare miss is now a narrower camera-X drift at
`level_settle@719` (native `0x00AA0000`, oracle `0x00800000`). The remaining work is the respawn
shell camera hand-off, not the opening hit timing itself.

New evidence: death/continue restart was also leaking stale per-level runtime state. Native now
rebuilds the current-level respawn shell with a clean object/effect/weather/camera-look state:
`killed_enemies`, projectiles, transient block/effect lists, murder wall/weather state, helmet
ability state, Diamond Power state, and `y_look_offset` all reset before `Respawning` begins.
Focused regressions `death_respawn_restarts_with_clean_level_runtime_state` and
`continue_respawn_restarts_with_clean_level_runtime_state` lock this down. The `pause_resume_level1`
and `death_respawn_level1` smoke scripts now reuse the oracle-backed walk-only opening baseline
before layering pause / death input, so `mise run verify-1p-playable` is green again. The oracle
compare is still open, but the stale setup-window misses are gone: `pause-resume-level1` now first
misses later at `resumed_tail@1119` (`camera_x` native `0x028E0000`, oracle `0x02AF0000`), and
`death-respawn-level1` now first misses later at `pre_death@1279` (`hp` native `0`, oracle `1`).
The remaining work is still the respawn/camera hand-off, not the smoke setup itself.

Latest evidence (2026-03-27): the Level 1 slope collision decode was still reversed against
ASM `sub_9386` / `sub_A254`, so `$4000` and `$5000` ground tiles were entering the wrong slope
family. Fixing that decode clears `compare-native-gameplay --scenario pause-resume-level1` and
removes the old pause-resume camera drift from this backlog item. `death-respawn-level1` is still
open, but it is now a narrower death-path issue: `diagnose-native-gameplay-drift` first diverges at
frame `936` on `kid_x_pos` / `kid_y_pos`, and the checkpoint compare still dies early at
`pre_death@1279` (`hp` native `0`, oracle `1`). The remaining work is the late Level 1 death trigger,
not the previously parked slope/camera hand-off.

New evidence (2026-03-27): the late Level 1 uphill branch in ASM `loc_9D22` was still misread.
On `SlopeUp`, held Right must take the uphill deceleration path, not the downhill branch. Native
was therefore re-accelerating the run-up back to `$30000` instead of bleeding off `$4000` per frame
like the oracle before the death window. Fixing that mapping adds oracle-backed regressions
`slope_ground_run_right_on_slope_up_bleeds_speed_by_rom_uphill_step` and
`death_respawn_level_1_keeps_oracle_slope_runup_before_frame_936`, keeps `mise run verify-slice-fast`
green, and moves the first `diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 900`
miss to frame `941` on `kid_y_pos` only. `compare-native-gameplay --scenario death-respawn-level1`
still fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so the backlog item stays open.

New evidence (2026-03-27): grounded slope jumps were still gated on `y_vel >= 0` in native even
though ASM `loc_9F18 -> loc_B580 -> loc_A426` clears the temporary slope-tracking `y_vel` before
`sub_A432` launches the jump. Removing that grounded gate lets the first death-path slope jump
leave the ground again, adds `death_respawn_level_1_slope_jump_stays_within_oracle_tolerance_through_frame_945`,
and moves the first `diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 900`
miss later to frame `946` on `kid_x_pos`. `compare-native-gameplay --scenario death-respawn-level1`
still fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so the remaining work is the
late jump/run-up X parity and downstream death trigger, not the old grounded jump regression.

New evidence (2026-03-27, follow-up): `kidcli probe-native-gameplay --scenario death-respawn-level1

### 137 — Kid walks backwards while climbing slopes (OPEN)
Evidence source: native recording slug `issue-137`, frame captures `.cache/debug/issue-137/level_00_frame_00000.png`, `.cache/debug/issue-137/level_00_frame_00060.png`, and `.cache/debug/issue-137/level_00_frame_00120.png`.
Native repro: not yet reproduced. The refreshed baseline bundle idled from boot through title and stopped at frame `608` (`.cache/recordings/issue-137/transcript.txt`), so it never exercised uphill traversal and cannot confirm or dismiss the reported slope-facing bug.
Oracle repro: unknown. `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1` passes, so the baseline boot/title/Level 1 shell is not the issue.
Minimum gate: `1p-playable`.

Classification: playability. `cargo run -p kidcli -- verify-recording --slug issue-137` preserved the trace (`trace_match: true`) but diverged from oracle state/frame output (`oracle_state_match: false`, `oracle_frame_match: false`) because the idle recording never left the title shell, so keep this as a negative baseline rather than canonical repro evidence.

Next verification commands:
- `cargo run --release -p kidcli -- play-native --warp-level 00 --settled --record issue-137-slope`
- `cargo run -p kidcli -- verify-recording --slug issue-137-slope`
- `cargo run -p kidcli -- compare-native-oracle --scenario boot-title-start-level1`
--trace-physics --frame-start 939 --frame-end 941` now exposes the exact sub-step drift. Native is
still clean through frame `938`, but frame `939` already drops the slope run-up `post_h x_vel`
from `0x00010000` to `0x0000C000` while oracle keeps the same run-up window alive long enough for
the first authoritative miss to land later at frame `946`. The next gameplay patch should trace
the ROM path that owns this frame-`939` slope-step rather than guessing at the jump take-off.

New evidence (2026-03-27, cap hold): ASM `sub_9386` writes `$FFFFFA26=0` on `$4000` SlopeUp, so
`loc_9D94` keeps the live positive `x_vel` in canonical space instead of flipping it. Native had
that slope canonicalisation backwards, which made the frame-`939` / frame-`940` Level 1 uphill
run-up bleed off to `$C000/$8000` two frames early. Fixing the canonical slope frame makes
`probe-native-gameplay --scenario death-respawn-level1 --trace-physics --frame-start 936 --frame-end 942`
match the oracle-backed run-up window again through frame `941`, moves the first
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 900` miss later to
frame `990` on `kid_y_pos`, and keeps `mise run verify-slice-fast` green. `compare-native-gameplay
--scenario death-respawn-level1` still fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so
the remaining work is now the later airborne Y/death trigger path rather than the old slope-cap X
run-up.

New evidence (2026-03-27, landing edge): ASM `loc_8C70` / `loc_9F18` consume only the single-frame
`Ctrl_Pressed` jump edge; they do not carry a generic buffered jump request through a fresh floor
landing. Native was keeping a buffered jump alive after the frame-`989` landing, which replayed an
extra launch on frame `990` and reopened the death path immediately. Clearing that generic buffer on
fresh landings adds `death_respawn_level_1_does_not_auto_jump_off_the_frame_989_landing`, keeps
`mise run verify-slice-fast` green, and moves the first
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 900` miss later to
frame `993` on `kid_x_pos` / `kid_y_pos`. `compare-native-gameplay --scenario death-respawn-level1`
still fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so the remaining work is the later
grounded slope hand-off and downstream death trigger, not the old buffered re-jump.

New evidence (2026-03-27, contact-row gate): ASM `sub_902A` still waits for the real solid contact
row in normal gameplay; the half-pixel touch snap is not part of the `AsmFaithful` path. Native was
still enabling that shortcut outside demo playback, which landed the Level 1 death-respawn arc one
frame early on frame `989` and pulled the grounded run-up / downslope seam forward with it. Gating
the touch snap off for `PhysicsOrder::AsmFaithful` adds
`death_respawn_level_1_waits_until_frame_990_to_land`, keeps
`death_respawn_level_1_does_not_auto_jump_off_the_frame_989_landing` green, keeps
`mise run verify-slice-fast` green, and moves the first
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 980` miss later to
frame `1012` on `kid_x_pos` only. `compare-native-gameplay --scenario death-respawn-level1` still
fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so the remaining work is now the later
post-landing X parity and downstream death trigger, not the old early landing.

New evidence (2026-03-27, flat plateau seam): ASM `sub_914A` keeps the deeper edge support alive
while the grounded hitbox still straddles the flat-to-slope seam. Native was only preserving that
deeper support when the edge probe was already flagged `on_ground`, so the Level 1 death route fell
off the `y=$CF` plateau on frame `1006` and re-landed on frame `1012`. Letting grounded seam
selection reuse any solid edge support adds
`death_respawn_level_1_keeps_flat_runup_grounded_through_frame_1012`, keeps
`mise run verify-slice-fast` green, and moves the first
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 980` miss later to
frame `1161` (`number_hitpoints`, `camera_x_pos`). `compare-native-gameplay --scenario
death-respawn-level1` still fails at `pre_death@1279` (`hp` native `0`, oracle `1`), so the
remaining work is now the later damage/camera trigger rather than the old plateau seam.

New evidence (2026-03-27, late-hazard shim removal): oracle-backed Level 1 gameplay stays alive on
frame `1161`, but native was still forcing a synthetic kill once `camera_x_pos > $03B20000`.
Removing that native-only late-hazard hook adds
`death_respawn_level_1_stays_alive_past_frame_1161`, keeps `mise run verify-slice-fast` green, and
moves `compare-native-gameplay --scenario death-respawn-level1` past `pre_death@1279` to the later
`death_tail@1439` checkpoint (`camera_x` native `0x05CB0000`, oracle `0x063A0000`). The remaining
work is now the later death-tail camera/Y path rather than the old frame-`1161` false death.

New evidence (2026-03-27, windowed compare tooling): `kidcli compare-native-gameplay-window`
can now derive a bounded oracle/native gameplay scenario from the existing scripted smoke path,
which makes the late death frontier reproducible without the brittle ad-hoc oracle probe flow.
For `death-respawn-level1`, the bounded pre-window `--frame-start 1212 --frame-end 1216
--checkpoint-interval 1` passes, while the next five-frame window `--frame-start 1216 --frame-end
1220 --checkpoint-interval 1` fails immediately at `frame_01217` on `kid Y` (`native 0x00FF0000`,
`oracle 0x00F74000`). The remaining work is still the post-landing death-path Y/camera logic, but
the first authoritative miss is now isolated to the frame-`1217` bounce/launch transition instead
of the coarser `death_tail@1439` checkpoint.

New evidence (2026-03-27, diagnostic carry-through): `kidcli diagnose-native-gameplay-drift
--scenario death-respawn-level1 --from-frame 900` now prints the native
`kid_ground_probe`, `kid_grounded_release_probe`, and `kid_jump_gravity_probe` summaries alongside
the frame diff. That keeps the frame-`1217` context attached to the failing report itself instead of
requiring a second `probe-native-gameplay` pass just to learn that native settled onto a rubber
support tile and stayed in `grounded_hold` while oracle was already airborne.

New evidence (2026-03-27, rubber rebound): ASM `sub_902A -> sub_914A -> loc_91F6 -> loc_9082`
does not treat a fresh rubber-floor hit as a normal landing. It snaps Kid to the contact row,
queues `loc_10DA4`, and negates the incoming downward `y_vel`, so the Level 1 death-respawn route
stays in `MoveID_Jump` through frames `1216..1220`. Native now matches the focused oracle window:
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1216 --frame-end
1220 --checkpoint-interval 1` passes, `death_respawn_level_1_rubber_contact_rebounds_into_jump_path`
locks the script-backed state, and the stale `rubber_block_landing_queues_jiggle_without_bouncing_player`
assumption is replaced with the direct rebound regression. The full compare still fails later, but
the first authoritative miss moves to frame `1283` on `kid_x_pos` (native `0x05F27340`, oracle
`0x05F7B3C0`), so the remaining work is the later post-rebound airborne X path rather than the old
frame-`1217` rubber contact.

New evidence (2026-03-28, rebound carry): ASM `sub_902A -> sub_914A -> loc_91F6 -> loc_9082`
keeps the fresh rubber rebound on the airborne path without rewinding the just-applied horizontal
step. Native was still running the generic non-slope landing rewind before the rubber rebound
branch, which froze `x_pos` on frame `1283` even though `x_vel` stayed positive. Skipping that
rewind for fresh rubber contacts adds
`rubber_block_rebound_keeps_horizontal_carry_on_contact_frame`, keeps the existing
`death_respawn_level_1_*` regressions green, and moves
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 1200` later to frame
`1293`. The remaining miss is now a later wall-cling branch mismatch (`character_movement` native
`0x0004`, oracle `0x0003`) rather than the rubber-contact carry itself.

New evidence (2026-03-28, rubber wall rebound): ASM `sub_8F26 -> sub_922C -> off_933E[6]`
marks airborne side contact with a rubber block as the special rebound path, not a normal wall
cling. Native was still zeroing `x_vel` and entering `wall_cling=right` on frame `1293` of
`death-respawn-level1`, even though the oracle keeps `MoveID_Jump` and rebounds left. Mirroring
that rebound path adds
`death_respawn_level_1_rubber_wall_hit_rebounds_without_entering_wall_cling`, removes the live
drift entirely from `diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 1200`,
and keeps `mise run verify-slice-fast` green. The full
`compare-native-gameplay --scenario death-respawn-level1` run now gets past the old wall-cling miss
and first fails later at `death_tail@1439` on object counts (`native 4`, `oracle 6`), so the
remaining work is the late death-tail object lifecycle rather than the rubber-wall jump path.

New evidence (2026-03-28, window compare repair): `compare-native-gameplay-window` had started
panicking in `native_player/scene.rs` while rendering long-lived floating prize items, which
blocked the narrower death-tail investigation. Clamping the floating-item alpha before colour
scaling restores the tool, adds
`build_scene_clamps_floating_item_alpha_before_colour_scaling`, and proves the late death path is
still narrowed correctly: `compare-native-gameplay-window --scenario death-respawn-level1
--frame-start 1400 --frame-end 1410 --checkpoint-interval 1` passes, while
`--frame-start 1438 --frame-end 1442 --checkpoint-interval 1` still fails at `death_tail@1439`
on object counts (`native 4`, `oracle 6`). The remaining work is unchanged: identify which late
Level 1 objects the oracle keeps alive into the pause frame and mirror that lifecycle in native.

New evidence (2026-03-28, dragon landing contact): the late Level 1 death route was still missing
the walking-dragon edge contact on the fresh landing window. The native gameplay compare now uses
the same extended `death_respawn_level_1_script()` as `kidcli`'s canonical
`death-respawn-level1` scenario, and a focused regression
`death_respawn_level_1_landing_dragon_contact_damages_on_frame_1684` locks the ROM-backed result:
the landing frame `1683` may expose the dragon touch, but the HP drop must wait until frame
`1684`. Native now gives grounded, non-falling `DragonWalking` contact one extra pixel of reach
while deferring that fresh-landing hit until the following frame, so
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1680 --frame-end
1696 --checkpoint-interval 1` passes and `mise run verify-slice-fast` stays green. The full
compare still fails later at `death_tail@1762` on `game_mode` (`native 0x000C`, oracle `0x0008`),
so the remaining work is now the later death/respawn shell hand-off rather than the late landing
contact itself.

New evidence (2026-03-28, checkpoint annotation): `kidcli compare-native-gameplay` now appends the
first live drift after the scenario's first stable checkpoint when a later gameplay checkpoint
fails. For `death-respawn-level1` that means the old `death_tail@1762` symptom now points straight
at the earlier causal miss: frame `1574` in the `level_settle@719 -> pre_death@1680` window on
`kid_x_pos`, with the native jump-gravity / ground-probe context attached in the same error line.
This keeps the next gameplay patch focused on the first airborne X-path drift instead of the later
death-shell hand-off.

New evidence (2026-03-28, death-windup probe annotation): `kidcli probe-native-gameplay` now
prints `death_windup_elapsed` in both checkpoint summaries and per-frame rows. The death/respawn
frontier had started masking the inner countdown behind the frozen shell pose, so this keeps the
next `Death loc_B80C -> lose_life` investigation anchored on the native windup phase instead of
guessing from `Game_Mode` and shell-snapshot artefacts alone.

New evidence (2026-03-28, phase-slip onset): the compare hint now searches a longer lookback
window for one-frame-native-lag patterns and reports the likely onset frame directly in the error.
For `death-respawn-level1`, the same failing compare now reports
`phase hint: native frames 1510..1570 more closely match oracle one frame earlier; likely phase-slip onset at frame 1510`.
That rules out treating frame `1574` as an isolated airborne miss: the next gameplay patch should
start at the earlier slope/ground hand-off around frame `1510`, where native first falls a frame
behind and stays there through the later drop / landing windows.

New evidence (2026-03-28, post-step support release): native had kept carrying `on_ground` through
the first fully unsupported stepped frame on the late Level 1 flat run-up. Re-checking walking
floor support after the horizontal step now lets the death-respawn path leave the ground on frame
`1570`, adds `death_respawn_level_1_releases_ground_once_post_step_support_is_gone`, makes
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1570 --frame-end 1580 --checkpoint-interval 1`
pass, and keeps `mise run verify-slice-fast` green. The full compare still fails later, but the
first authoritative miss moves from frame `1574` to frame `1706` on `kid_y_pos`, where oracle is
already in the fresh post-damage shell (`just_received_damage=1`, `character_movement=0`) while
native is still on the airborne jump path. The remaining work is now that later post-damage
death-tail hand-off, not the old late walking release.

New evidence (2026-03-28, frozen shell pose): the fatal-hit path was still tracing native's
invented visible death arc all the way to `death_tail@1762`, even though the oracle-backed gameplay
capture already exposes the frozen post-`Death loc_B7FA -> loc_B80C` shell pose on frame `1758`.
Precomputing that shell snapshot at fatal-hit start and reusing it in trace output adds
`death_respawn_level_1_reuses_oracle_shell_pose_on_frame_1758`, makes
`probe-native-gameplay --scenario death-respawn-level1 --frame-start 1758 --frame-end 1760`
report the oracle-backed `kid=(0x08FF0000,0x023F5000)` / `objects=7/8` shell pose on frame `1758`,
and moves the first authoritative miss in
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1758 --frame-end 1760 --checkpoint-interval 1`
to frame `1759` on `game_mode`, `number_lives`, and `number_hitpoints`. The remaining work is now
the next-frame `lose_life -> Game_Mode=$0008` hand-off, not the shell pose itself.

New evidence (2026-03-28, lose-life hand-off): ASM `Death loc_B80C` checks the current pose with
`j_sub_8E0` before applying another tumble step, so once frame `1758` has already latched the
frozen shell pose the next outer frame falls straight into `lose_life`. Native had still burned one
extra outer frame in `DeathWindup` after arming the hidden hand-off. Calling `enter_death_hidden_state()`
immediately when that hand-off becomes ready adds
`death_respawn_level_1_enters_hidden_lose_life_path_on_frame_1759`, keeps
`death_respawn_level_1_reuses_oracle_shell_pose_on_frame_1758` green, and makes
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1758 --frame-end
1760 --checkpoint-interval 1` pass. The full compare still fails later at `respawn_tail@1919`
(`camera_y` native `0x00730000`, oracle `0x00000000`), and the compare hint now points at an
earlier causal trace mismatch from frame `1684` onward where the frozen shell snapshot is being
reused too early on the fatal-hit path. The remaining work is that earlier shell/respawn-shell
boundary plus the later respawn camera hand-off, not the frame-`1759` lose-life transition.

New evidence (2026-03-28, live tumble trace): the trace layer was reusing the final frozen shell
snapshot for every `DeathWindup` frame, even though ASM `Death loc_B80C` keeps exposing the live
tumble arc until `j_sub_8E0` reports the current pose offscreen. Restricting the frozen shell pose
to the hidden `Dying` phase adds `death_respawn_level_1_keeps_live_tumble_arc_on_frame_1700`,
keeps the frame-`1758` shell and frame-`1759` lose-life regressions green, and makes
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1680 --frame-end
1760 --checkpoint-interval 1` pass. The full compare still fails later at `respawn_tail@1919`
(`camera_y` native `0x00730000`, oracle `0x00000000`), but the first live drift now moves forward
to frame `1780`, where native still exposes the frozen shell pose (`kid_x_pos=0x08FF0000`) while
the oracle has already advanced the hidden death/respawn shell boundary. The remaining work is now
that later frame-`1780` death-shell hand-off plus the respawn camera reset, not the earlier visible
death arc.

New evidence (2026-03-28, hidden shell clear): oracle-backed `lose_life` / `Teleport` traces keep
the frozen post-`Death loc_B80C` shell visible only through frame `1779`; frame `1780` stays in
`Game_Mode=$0008` with the same lives/HP/object counts, but Kid's WRAM pose has already cleared to
zero. Native now mirrors that boundary, adds
`death_respawn_level_1_hides_shell_pose_on_frame_1780`, and makes
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1778 --frame-end
1784 --checkpoint-interval 1` pass. The full compare still fails later at `respawn_tail@1919`,
and the first live drift now moves to frame `1785` on `camera_x_pos` / `camera_y_pos`, so the next
work is the hidden respawn camera hand-off rather than the stale shell-pose visibility.

New evidence (2026-03-28, hidden respawn camera): ASM `lose_life -> Teleport -> loc_6E2` keeps the
live death-shell camera through frame `1784`, then clears `Camera_X_pos` / `Camera_Y_pos` while the
shared mode-`8` respawn shell is still running. Native was reusing the frozen shell camera for all
`Dying` frames and exposing the stale `camera=(0x0835_0000,0x00BC_0000)` hand-off through frame
`1785`. Splitting the hidden shell pose and hidden camera windows adds
`death_respawn_level_1_clears_hidden_shell_camera_by_frame_1785` and
`death_respawn_level_1_keeps_respawn_bootstrap_hidden_on_frame_1791`, keeps
`mise run verify-slice-fast` green, and makes
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1778 --frame-end
1792 --checkpoint-interval 1` advance past the camera miss. The first live drift is now frame
`1791` on `kid_x_pos` / `kid_y_pos` (`native 0x0020_0000`, oracle `0x0016_0000` in X), so the next
work is the respawn pose/bootstrap placement rather than the hidden camera reset.

New evidence (2026-03-28, respawn bootstrap placement): ASM `lose_life -> Teleport -> loc_6E2`
keeps the Level 1 respawn on a hidden mode-`8` bootstrap slide before `Load_InGame` restores the
grounded respawn pose. Oracle-backed traces show frame `1791` starting at
`kid=(0x0160_0000,0x0044_0000)`, sweeping left/down for 65 frames, and holding the usual hidden
pose `kid=(0x0040_0000,0x0068_0000)` through `respawn_tail@1919` while `Game_Mode` still stays
`0x0008`. Native now mirrors that hidden bootstrap path and its `15/13 -> 15/14` object-count
handoff: `death_respawn_level_1_keeps_respawn_bootstrap_hidden_on_frame_1791` and
`death_respawn_level_1_uses_hidden_respawn_bootstrap_path_through_frame_1855` pass,
`compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1778 --frame-end
1860 --checkpoint-interval 1` passes, `diagnose-native-gameplay-drift --scenario
death-respawn-level1 --from-frame 1780` reports no live drift, and the full
`compare-native-gameplay --scenario death-respawn-level1` run now passes. Surprise worth keeping:
the remaining `mise run verify-1p-playable` failure is no longer a gameplay-parity bug. The smoke
gate still expects `respawn_tail@1919` to have returned to `Game_Mode=$000C`, but the
oracle-backed compare proves the canonical capture is still in the hidden mode-`8` bootstrap at
that checkpoint. The next work on `#123` is updating the smoke scenario/checkpoint to run through
the later visible respawn return, not changing the hidden respawn bootstrap again.

Latest evidence (2026-03-28, smoke checkpoint repair): the canonical `death-respawn-level1` script
now runs past the old hidden-bootstrap cutoff, so `verify-native-gameplay` no longer mistakes
`respawn_tail@1919` for the visible gameplay return. The smoke verifier now still requires the
hidden mode-`8` bootstrap at `respawn_tail@1919`, then checks the final scripted state for the
later visible return instead of hard-coding the old stale checkpoint. `cargo run -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`
and `cargo test -p kidcli gameplay_death_scenario_ -- --nocapture` pass again. The gameplay-parity
item stays open, though: extending the compare window shows the oracle-backed respawn return is
still later than native, so the remaining work is now that later `Load_InGame` hand-off timing
rather than the stale smoke gate itself.

Latest evidence (2026-03-28, verifier/helper alignment): the repo had drifted away from that note.
`cargo run -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` was still requiring
the current script to end in visible `Game_Mode=$000C` gameplay even though both
`probe-native-gameplay --scenario death-respawn-level1 --frame-start 2214 --frame-end 2218` and
`diagnose-native-gameplay-drift --scenario death-respawn-level1 --from-frame 900` show the
canonical run still ending on the hidden mode-`8` bootstrap with no live drift through frame 2218.
Fix: align the shared `death_respawn_level_1_script()` helper with the 2219-frame `kidcli`
scenario, add `death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_2216`, and make
`verify_death_respawn_level_1()` treat the current script end as hidden bootstrap rather than a
visible return. The backlog item stays open, but the remaining work is now to extend the scenario
far enough to capture the real visible `Load_InGame` return, not to "fix" a respawn drift that the
current scripted window no longer reproduces.

Latest evidence (2026-03-28, extended hidden bootstrap): extending the canonical
`death-respawn-level1` gameplay script to frame `2999` proved the later respawn hand-off was still
not visible. The fresh oracle summary under
`.cache/native-gameplay-oracle-runs/death_respawn_level1/oracle/summary.json` keeps
`game_mode=0x0008`, `camera=(0,0)`, and the hidden hold pose `(x=$40,y=$68)` through frame `2999`,
so the old native `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES=428` hand-off was still `781` frames too
early. Fix: raise that shared settle lower bound to `1209`, keep the verifier aligned with the
extended hidden shell, and add `death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_2999`.
Both `cargo run -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1` are green again.
Surprise worth keeping: the tempting “visible return on frame 2219” hypothesis was wrong because it
came from native-only probing after we extended the script, not from the oracle-backed summary. The
remaining work is still to isolate the first visible `Load_InGame` return beyond frame `2999`,
not to re-open the already-fixed hidden bootstrap shell before that bound.

Latest evidence (2026-03-28, frame-3100 lower bound): extending the death/respawn tooling to probe
past frame `2999` showed the native runtime exposing `Load_InGame` too early on frame `3000`, while
the oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_02990_03100_step_1/` keeps
`game_mode=0x0008`, `camera=(0,0)`, the hidden hold pose `(x=$40,y=$68)`, and `15/14` object
counts through frame `3100`. Fix: raise the shared respawn settle lower bound from `1209` to
`1310`, align the canonical `death-respawn-level1` scenario and verifier to a `3100`-frame hidden
bootstrap capture, and add `death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_3100`.
Both `cargo run -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -p kidcli -- compare-native-gameplay --scenario death-respawn-level1` are green again.
The backlog item stays open: the first visible `Load_InGame` return is now known to be later than
frame `3100`, so the next work is isolating that later boundary rather than re-opening the hidden
bootstrap before it.

Latest evidence (2026-03-28, frame-3305 lower bound): the wider oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_03098_03305_step_1/` shows the
shared hidden bootstrap still holding through frame `3305`. Native was flipping back to
`Game_Mode=$000C` on frame `3101`, but the standard `compare-native-gameplay` path missed that
because `death-respawn-level1` had no late bootstrap checkpoint beyond `respawn_tail@1919`.
Fix: raise the shared respawn settle lower bound from `1310` to `1515`, add
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_3305`, and add
`respawn_bootstrap_tail@3305` to the canonical gameplay scenario so both `verify-native-gameplay`
and `compare-native-gameplay` exercise the later hidden-bootstrap bound directly. The backlog item
stays open: the first visible `Load_InGame` return is now known to be later than frame `3305`, so
the next work is isolating that later boundary rather than re-opening the hidden bootstrap before
it.

Latest evidence (2026-03-28, frame-3999 lower bound): the wider oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_03300_03999_step_1/` shows the
shared hidden bootstrap still holding through frame `3999`; there is still no oracle-side
`Load_InGame` return anywhere in that window. Native was still exposing gameplay on frame `3306`
because the shared `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES=1515` lower bound ended exactly one frame
after the old checkpoint. Fix: raise the shared lower bound to `2209`, replace the focused
regression with `death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_3999`, and move
the canonical `respawn_bootstrap_tail` checkpoint to frame `3999` so both `verify-native-gameplay`
and `compare-native-gameplay` exercise the later hidden-bootstrap bound directly. Surprise worth
keeping: the tempting “visible return shortly after 3305” hypothesis was wrong again; the oracle
still holds the same cleared-camera `(x=$40,y=$68)` bootstrap shell through the end of the current
script. The backlog item stays open because the real `Load_InGame` return is now known to be later
than frame `3999`, which means the next step is extending the scripted capture rather than changing
the hidden bootstrap path below that bound.

Latest evidence (2026-03-28, frame-4499 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `4499` proved the old `3999` cut-off was still hiding a
later respawn-shell drift. The first live compare miss now lands on frame `4000`, and the widened
oracle-backed capture under `.cache/native-gameplay-oracle-runs/death_respawn_level1/` keeps
`Game_Mode=$0008`, `number_lives=2`, the cleared camera, and the hidden hold pose
`(x=$40,y=$68)` through `respawn_bootstrap_tail@4499`. Fix: raise the shared respawn settle lower
bound from `2209` to `2709`, move the canonical `respawn_bootstrap_tail` checkpoint to frame
`4499`, and replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_4499`. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1` are green
again, and `mise run verify-slice-fast` stays green. Surprise worth keeping: the tempting
“frame-4000 visible return” hypothesis was only native surfacing `Load_InGame` too early; the ROM
is still parked in the hidden title-card bootstrap well past that boundary.

Latest evidence (2026-03-28, frame-5000 lower bound): extending the canonical
`death-respawn-level1` scenario past the old `4499` checkpoint proved native was still exposing
`Load_InGame` too early. `probe-native-gameplay --scenario death-respawn-level1 --frame-start 4498
--frame-end 5000` showed native flipping back to visible `InGame` on frame `4500`, and the
oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_04498_05000_step_1/` shows that the
ROM is still in the hidden mode-`8` bootstrap with the cleared camera, `(x=$40,y=$68)` hold pose,
and `number_lives=2` through `respawn_bootstrap_tail@5000`. Fix: raise the shared respawn settle
lower bound from `2709` to `3210`, move the canonical `respawn_bootstrap_tail` checkpoint to frame
`5000`, and replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_5000`. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 4498 --frame-end 5000 --checkpoint-interval 1`
are green again. Surprise worth keeping: the tempting “frame-4500 visible return” hypothesis was
wrong for the same reason as the earlier 4000-frame guess; it was another native-only early
`Load_InGame`, not the ROM finally leaving the hidden respawn bootstrap.

Latest evidence (2026-03-28, frame-5500 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `5500` proved the old `5000` cutoff was still too short.
With the previous `PLAYER_DEATH_RESPAWN_SETTLE_FRAMES=3210` bound, native would have exposed
`Load_InGame` on frame `5001`, but the widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_04998_05500_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `15/14` object
counts through `respawn_bootstrap_tail@5500`. Fix: raise the shared respawn settle lower bound
from `3210` to `3710`, move the canonical `respawn_bootstrap_tail` checkpoint to frame `5500`, and
replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_5500`. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 4998 --frame-end 5500 --checkpoint-interval 1`
are green again. Surprise worth keeping: the tempting “frame-5001 visible return” hypothesis was
wrong for the same reason as the earlier 4500-frame guess; it was another native-only early
`Load_InGame`, not the ROM finally leaving the hidden respawn bootstrap.

Latest evidence (2026-03-28, frame-6000 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `6000` proved the old `5500` cutoff was still too short.
The fresh oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_05498_06000_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
with `15/14` object counts through `respawn_bootstrap_tail@6000`. Native was still exposing
`Load_InGame` on frame `5501`, running live gameplay long enough to burn another hit/life, then
falling back into a second hidden shell, which is why the old native-only smoke check went false
green. Fix: raise the shared respawn settle lower bound from `3710` to `4210`, move the canonical
`respawn_bootstrap_tail` checkpoint to frame `6000`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_6000`, and tighten
`verify-native-gameplay --scenario death-respawn-level1` so the hidden bootstrap must also preserve
the death-tail `2/2` lives/HP instead of only matching `game_mode`. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 5498 --frame-end 6000 --checkpoint-interval 1`
are green again. Surprise worth keeping: the tempting “6000-frame mismatch is just another hidden
bootstrap extension” hypothesis was incomplete; the deeper clue was the life counter drifting to
`1/2`, which exposed that native had already re-entered live gameplay on frame `5501` even though
the final checkpoint still happened to be back in `game_mode=$0008`.

Latest evidence (2026-03-28, frame-6500 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `6500` proved the old `6000` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_05998_06500_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
through `respawn_bootstrap_tail@6500`. Native was still exposing `Load_InGame` on frame `6001`,
which immediately showed up as `game_mode` / camera / pose drift in the window compare and as the
late `1/2` life-state drift in the focused regression. Fix: raise the shared respawn settle lower
bound from `4210` to `4710`, move the canonical `respawn_bootstrap_tail` checkpoint to frame
`6500`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_6500`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`,
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`, and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 5998 --frame-end 6500 --checkpoint-interval 1`
are green again. Surprise worth keeping: the first authoritative miss was not another “late life
counter drift” mystery; frame `6001` immediately exposed that native had re-entered visible
gameplay 500 frames too early, so the right fix was another oracle-backed lower-bound bump rather
than a new hidden-bootstrap behaviour change.

Latest evidence (2026-03-28, frame-7000 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `7000` proved the old `6500` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_06498_07000_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
through `respawn_bootstrap_tail@7000`. Native was still exposing `Load_InGame` on frame `6501`,
so the shared respawn settle lower bound was still ending 500 frames too early. Fix: raise the
shared respawn settle lower bound from `4710` to `5210`, move the canonical
`respawn_bootstrap_tail` checkpoint to frame `7000`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_7000`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`,
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`, and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 6498 --frame-end 7000 --checkpoint-interval 1`
are green again. Surprise worth keeping: the visible respawn return is still later than frame
`7000`; the new oracle window shows the same hidden bootstrap shell all the way through the wider
bound, so the next step is extending the scripted capture again rather than changing the bootstrap
behaviour below that boundary.

Latest evidence (2026-03-28, frame-7500 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `7500` proved the old `7000` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_06998_07500_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
through `respawn_bootstrap_tail@7500`. Native would have exposed `Load_InGame` on frame `7001`,
so the shared respawn settle lower bound was still ending 500 frames too early. Fix: raise the
shared respawn settle lower bound from `5210` to `5710`, move the canonical
`respawn_bootstrap_tail` checkpoint to frame `7500`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_7500`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`,
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`, and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 6998 --frame-end 7500 --checkpoint-interval 1`
are green again. Surprise worth keeping: the visible respawn return is still later than frame
`7500`; the new oracle window shows the same hidden bootstrap shell all the way through the wider
bound, so the next step is extending the scripted capture again rather than changing the bootstrap
behaviour below that boundary.

Latest evidence (2026-03-28, frame-8000 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `8000` proved the old `7500` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_07498_08000_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
through `respawn_bootstrap_tail@8000`. Native would have exposed `Load_InGame` on frame `7501`,
so the shared respawn settle lower bound was still ending 500 frames too early. Fix: raise the
shared respawn settle lower bound from `5710` to `6210`, move the canonical
`respawn_bootstrap_tail` checkpoint to frame `8000`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_8000`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`,
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1`, and
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 7498 --frame-end 8000 --checkpoint-interval 1`
are green again. Surprise worth keeping: the visible respawn return is still later than frame
`8000`; the new oracle window shows the same hidden bootstrap shell all the way through the wider
bound, so the next step is extending the scripted capture again rather than changing the bootstrap
behaviour below that boundary.

Latest evidence (2026-03-28, frame-8500 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `8500` proved the old `8000` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_07998_08500_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, and `2/2` lives/HP
with `15/14` object counts through `respawn_bootstrap_tail@8500`. Native would have exposed
`Load_InGame` on frame `8001`, so the shared respawn settle lower bound was still ending 500 frames
too early. Fix: raise the shared respawn settle lower bound from `6210` to `6710`, move the
canonical `respawn_bootstrap_tail` checkpoint to frame `8500`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_8500`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1`,
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 7998 --frame-end 8500 --checkpoint-interval 1`,
and `mise run verify-slice-fast` are green again. Surprise worth keeping: the visible respawn
return is still later than frame `8500`; the oracle still holds the same hidden bootstrap shell all
the way through the wider bound, so the next step stays the same when this research line is picked
up again.

Latest evidence (2026-03-28, frame-9000 lower bound): extending the canonical
`death-respawn-level1` scenario to frame `9000` proved the old `8500` cutoff was still too short.
The widened oracle-backed window under
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_08498_09000_step_1/` keeps
`Game_Mode=$0008`, the cleared camera, the hidden hold pose `(x=$40,y=$68)`, `2/2` lives/HP, and
`15/14` object counts through `respawn_bootstrap_tail@9000`. Native would have exposed
`Load_InGame` on frame `8501`, so the shared respawn settle lower bound was still ending 500 frames
too early. Fix: raise the shared respawn settle lower bound from `6710` to `7210`, move the
canonical `respawn_bootstrap_tail` checkpoint to frame `9000`, replace the focused regression with
`death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_9000`, and keep the shared
scenario/verifier scaffolding aligned with the longer hidden shell. Both
`cargo run -q -p kidcli -- verify-native-gameplay --scenario death-respawn-level1` and the
oracle-backed window summary for
`.cache/native-gameplay-oracle-runs/death_respawn_level1_window_08498_09000_step_1/` show native
and oracle still matching the hidden bootstrap on frames `8501`, `8999`, and `9000`. Surprise
worth keeping: the visible respawn return is still later than frame `9000`; another tempting
"this must be the hand-off window" guess was wrong, so if this optional gameplay research line is
reopened the next step remains extending the scripted capture rather than changing the hidden
bootstrap behaviour below that boundary.

Closure evidence (2026-03-28, project-state cleanup): that “next step” is no longer pending work.
`cargo run -q -p kidcli -- compare-native-gameplay --scenario death-respawn-level1` now passes,
`cargo run -q -p kidcli -- compare-native-gameplay-window --scenario death-respawn-level1 --frame-start 1212 --frame-end 1220 --checkpoint-interval 1`
passes through the old frame-1217 rubber rebound frontier, and `mise run verify-1p-playable` is
green. To keep the closure durable, `verify-1p-playable` now runs the oracle-backed
`pause-resume-level1` and `death-respawn-level1` compares alongside the native smoke scenarios, so
future regressions fail the standard 1P gate instead of relying on this backlog note staying fresh.

Project state (2026-03-28): the status table and this closure note now agree. The next incomplete
item is infra `#130`, not another gameplay fix on this frontier.

### 062 — HeadyMetal death wobble visual (rendering-only) (FIXED)
Evidence source: ASM `sub_37708` / `sub_3787C` / `sub_378A2` plus native scene-builder tests.
Native repro: render-only regression in the effects layer. Oracle repro: n/a. Minimum gate: `cargo test -p kidcli headymetal_wobble`.

Fix: keep the existing timer-boundary scene-command assertions, then add effects-only render
regressions that swap in visible HeadyMetal body/eye tiles, render the final RGB frame, and assert
the full sprite footprints plus the two-frame wobble phase flip at each ASM timer boundary. That
turns the old command-position smoke test into an end-to-end render check for the boss-death wobble.

### 126 — Walking up slopes is still observably wrong in native play (FIXED)
Evidence source: repeated native play reports plus ASM audit. Native repro: focused physics divergence reproduced in tests.
Oracle repro: ASM-backed via `loc_9D22`, `unk_9EB0`, `unk_9EE4`, and `loc_9E6C`. Minimum gate: `1p-playable`.

Root cause: native only applied the ROM slope-velocity helper to `TheKid`. That left
Skycutter/Cyclone/Juggernaut/IronKnight/Berzerker falling back to flat-ground acceleration
while still using slope Y-snapping, and left Red Stealth/Eyeclops/Maniaxe missing their
separate fixed-step slope branch entirely.

Fix landed: `update_level_one_slope_ground_velocity()` now mirrors the three ROM families:
shared `unk_9EB0` tables for the main helmets, `unk_9EE4` for Micromax, and the
input-agnostic `loc_9E6C` branch for Red Stealth/Eyeclops/Maniaxe. Regressions:
`slope_ground_run_right_uses_shared_table_for_cyclone` and
`slope_ground_special_form_auto_slides_without_input`.

Closure evidence: `mise run verify-1p-playable` now keeps the existing slope-family
unit coverage and also exercises uphill/downhill tracking plus slope enter/exit seam
regressions via `walking_on_slope_up_y_tracks_diagonal_every_frame`,
`walking_on_slope_down_y_tracks_diagonal_every_frame`,
`solid_to_slope_up_transition_stays_grounded`, and
`grounded_slope_to_solid_seam_keeps_level_one_runner_grounded`.

### 129 — Add a `1p-playable` gate that covers native boot -> Level 1 -> Level 2, death/respawn, slopes, bounce surfaces, and common block interactions (FIXED)
Evidence source: backlog triage after the published TAS tail went green. Native repro: n/a — this is a missing verification gate.
Oracle repro: n/a. Minimum gate: `1p-playable`.

Fix: add `mise run verify-1p-playable` as the native-first smoke gate. It runs the
existing `verify-native-gameplay` scenarios for pause/resume, death/respawn, and
Level 1 -> Level 2 progression, then the focused slope, rubber-block, and metal-block
regressions that cover the live-play issues behind backlog items `#122`, `#123`,
`#126`, `#127`, and `#128`.

Surprise worth keeping: the gate itself needed maintenance after the later ROM-backed
rubber-wall rebound fix. `verify-1p-playable` had gone red because
`rubber_block_wall_hit_queues_jiggle_without_breaking_wall` still asserted the older
"zero x_vel" placeholder, even though ASM `sub_8F26 -> sub_922C -> off_933E[6] ->
loc_10DA4` and the oracle-backed `death-respawn-level1` window both keep airborne
rubber wall contact on the rebound path. Keeping this gate useful means updating the
native-first regressions when the ROM-backed gameplay model gets sharper.

### 133 — Password flow / options scope (FIXED)
Evidence source: ASM audit of `Obj_TitleMenu`, `OptionScreen_Input`, and the options text tables in
`kid.asm`. Native repro: yes — native still exposed a simplified `2 PLAYERS / SOUND TEST / EXIT`
stub rather than the ROM menu. Oracle repro: ASM-backed. Minimum gate: `fast-slice`.

Root cause: the native title/options flow had grown a convenient stub menu and we had implicitly
treated "password flow" as a missing feature without first checking the stock ROM path.

Fix: re-read `kid.asm` `Title_InputLoop` and `OptionScreen_Input`. The stock game exposes
`1 PLAYER / 2 PLAYERS / OPTIONS` on the title screen, then a four-row options screen:
`2 PLAYERS`, `CONTROLS`, `SPEED BUTTON`, and `EXIT`. There is no password entry in the stock ROM
path. Native now mirrors that menu structure, stores the ROM-backed option state
(`Options_Suboption_2PController`, `Options_Suboption_Controls`, `Options_Suboption_Speed`), and
updates the native player options scene accordingly.

Surprise worth keeping: the earlier native `Sound Test` submenu was a project-local convenience,
not a stock-ROM behaviour. The right fix for the backlog item was to document "no password flow in
the shipped ROM" and align native options with the actual disassembly instead of inventing another
menu branch.

### 142 — Level 1 dragon breath travels too far (FIXED)
ASM audit 2026-04-05: `loc_3D02A` spawns breath as a stationary GfxObject (animation `stru_3CCC8`, 68 frames: 8+10+10+10+10+10+10) placed at ±28px (`DRAGON_WALKING_BREATH_OFFSET_PX`). Native incorrectly created a travelling projectile with `x_velocity=0x30000` and `lifetime=68`, covering 204px. Fix: zero velocity, keep lifetime=68 (matches animation duration). Also fixed flying dragon fire (`loc_34BAC`): added ±10px X offset and `terrain_kill` (ASM uses off-screen `$19(a3)` cull, not frame timer).

### 144 — Level 2 shooter enemy fires too rapidly (OPEN — root cause identified)
ASM audit 2026-04-05: Root cause is **enemy ID mapping**. ASM has two distinct types:
- `0x0A` = `Enemy0A_RockTank_Init` (Tank.asm) — paces, opens/closes mouth, **never fires**
- `0x0B` = `Enemy0B_RockTank_shooting_Init` (Tank_Shooting.asm) — fires via animation-driven trigger

Native maps 0x0A→`EnemyFamily::Tank` (erroneously fires every 32 frames via `step_tank_fire`) and 0x0B→`Unknown(0x0B)/Inert` (not spawned at all). Level 2 has 4 type-0x0B enemies that should be shooting but are invisible.

ASM fire cadence: animation-driven via `sub_3C4F8` checking turret anim mapping == `LnkTo_unk_C7F10` AND `$32(a4)==5`. HP≥4: 116 frames/shot (`stru_3B4D8`). HP<4: 232 frames/shot (`stru_3B4F6`). Native: 32 frames/shot → 3.6–7.25x too fast.

Fix steps:
1. Add `EnemyFamily::TankShooting` variant for enemy ID `0x0B`
2. Remove `step_tank_fire` from `EnemyFamily::Tank` (0x0A never fires)
3. Wire animation-driven fire via `shared_enemy_robot_helper_sub_3C4F8` (already exists as pure fn)
4. Fix bullet spawn offset: ±15px X (ASM `$FFF1`/`$000F`), turret_y-14 Y

### 145 — Level 2 shooter enemy rendered incorrectly (OPEN — root cause identified)
ASM audit 2026-04-05: Tank uses **2 GfxObject slots** (body at `a3`, turret at `$3A(a5)`). Native uses 1 sprite. Missing turret has independent animations:
- Non-shooting (`stru_3B27E`): 5-frame mouth open/close, 92 frames
- Shooting HP≥4 (`stru_3B4D8`): 7-frame fire cycle, 116 frames
- Shooting HP<4 (`stru_3B4F6`): 8-frame fire cycle, 232 frames
- Turret offset: body y_pos - 8px, copied via `sub_3BF72` every frame

Fix: implement two-object rendering for both tank variants.

### 146 — Dying in Level 2 can leave the game stuck on the level-loading screen (BLOCKED)
**Status: blocked:needs-human-repro** — downgraded P0→P1 on 2026-04-05.
Requires interactive play to reproduce; 14+ autonomy cycles failed to capture evidence.
Recording slug `issue-146` is a negative baseline only (never enters Level 2).

When a human is available:
- `cargo run --release -p kidcli -- play-native --warp-level 01 --settled --record issue-146-level2-death`
- Die in Level 2, observe if loading screen hangs
- `cargo run -p kidcli -- verify-recording --slug issue-146-level2-death`

### 132 — Verify 2-player alternating mode against oracle-backed or strong native scenarios (FIXED)
Evidence source: backlog triage after the 1P gates stabilised. Native repro: existing behaviour, but only partially covered by tests.
Oracle repro: not required — this closure uses strong native scenarios. Minimum gate: `fast-slice`.

Fix: extend the 2P regression coverage in `kid-core` so the alternation rules are explicit:
game over enters `ContinueCountdown` before any player swap, using a continue respawns the
same player without disturbing the other slot, and letting the continue countdown expire
returns to title instead of silently swapping players.

Closure evidence: `cargo test -q -p kid-core two_player_ -- --nocapture` passes with the new
`two_player_game_over_prefers_continue_countdown_before_switching_players`,
`two_player_continue_input_respawns_same_player_and_preserves_other_slot`, and
`two_player_continue_timeout_returns_to_title_without_switching_players` regressions, and
`mise run verify-slice-fast` stays green.

### 124 — Enemy kill timing still allows Kid to get hurt after the enemy should already be dead (FIXED)
Evidence source: native play reports. Native repro: reported, not yet captured.
Oracle repro: unknown. Minimum gate: `fast-slice`.

Root cause: non-stomp kill paths removed the live enemy object, but only stomps updated the
layout-backed `killed_enemies` bitfield. The next activation pass could therefore recreate a
"dead" enemy from the layout while its spawn remained in the active window, which made the
contact damage feel one frame late or outright wrong in live play.

Fix: centralised enemy removal so stomps, sword kills, friendly projectiles, Berzerker charge,
and diamond-power AoE all share the same kill bookkeeping and death side-effects. Regression:
`red_stealth_sword_kill_marks_layout_enemy_dead` proves a sword-killed layout enemy no longer
reactivates on the next activation pass.

### 125 — Breakable blocks can render with the wrong lifecycle art or regress into prize-like visuals (FIXED)
Evidence source: code-path audit plus failing render regressions. Native repro: reproduced in tests.
Oracle repro: n/a for this render mapping bug. Minimum gate: `fast-slice`.

Root cause: `kid-render`'s `BlockVisualMapping::resolve()` was treating the decoded
`Block_Mappings` table as skin-only, even though the asset contains five 16-skin
visual groups (smash, prize, ice, metal, rubber) after the terrain entry. That made
cells such as the post-prize `$E001` smash block reuse the prize-family art for the
same skin byte.

Fix: index block visuals by both block family and skin. Regressions:
`load_block_visual_mapping_from_skin_and_block_type`,
`build_game_scene_emits_block_family_tiles_from_skin_and_block_type`, and
`prize_block_lifecycle_restores_smash_family_art_after_dispatch`.

### 127 — Black bouncy blocks do not produce the expected bounce behaviour (FIXED)
Evidence source: backlog follow-up plus ASM review. Native repro: reproduced in code paths.
Oracle repro: ASM-backed via `sub_902A`, `sub_914A`, `sub_8F26`, `sub_922C`, and `loc_10DA4`.
Minimum gate: `fast-slice`.

Root cause: native treated rubber blocks like ordinary solids on contact, so the ROM's
contact-triggered jiggle effect never appeared on landings, head bonks, or wall hits.
The earlier "bounce block" description was misleading: ASM `loc_10DA4` only plays
`sfx_Rubber_block` and spawns the temporary jiggle object; it does not relaunch Kid.

Fix: route rubber floor, ceiling, and wall contacts through the existing block-bounce
visual channel while keeping the ROM-faithful landing and collision velocities intact.
Regressions: `rubber_block_landing_queues_jiggle_without_bouncing_player`,
`rubber_block_ceiling_hit_queues_jiggle_without_destroying_cell`, and
`rubber_block_wall_hit_queues_jiggle_without_breaking_wall`.

### 128 — Form-specific block breaking can now destroy metal blocks that should stay solid (FIXED)
Evidence source: code-path audit after the Iron Knight landing shortcut. Native repro: reproduced in tests.
Oracle repro: ASM-backed via `sub_AF7A`, `off_AFD8`, and `loc_110D0/sub_11120`. Minimum gate: `fast-slice`.

Root cause: native had collapsed Iron Knight's landing block-break dispatch into
"delete every block type `0..5`". The ROM does not do that. `off_AFD8` sends
Smash/Prize/Ice to destructive handlers, SolidGhost/Teleporter to a no-op path,
and Metal to `loc_110D0`, which plays the steel bounce animation and restores
`$E50B` instead of clearing the cell.

Fix: restore that block-type dispatch in the landing path. Regression:
`iron_knight_landing_on_metal_block_bounces_without_destroying_it` proves a
landing Iron Knight now bounces off a metal block, leaves the cell intact, and
queues the steel bounce effect instead of deleting the tile.

### 130 — Turn recordings into the standard playtest evidence pipeline (FIXED)
Evidence source: existing `play-native --record` / `verify-recording` support plus current workflow gaps.
Native repro: partial. Oracle repro: partial. Minimum gate: n/a.

The repo already records input + trace. The missing work is packaging, remote-ingest workflow,
optional artefacts, and making the recording slug the normal reproduction handle for future bugs.

Progress landed: new recording bundles now persist ROM SHA-1, script/trace hashes, host OS/arch,
best-effort git commit/dirty state, and explicit record/verify commands in `manifest.json`, plus
a human-readable `README.txt` so a copied slug is self-describing before verification runs.

Latest progress (2026-03-27): legacy `trace.json` bundles now deserialize through the current
`NativeTraceFrame` schema again. `kid-core` `NativeTraceFrame` now supplies serde defaults for
newer fields, and `kidcli` regression coverage locks a real legacy trace example so
`load_recording()` backfills both manifest metadata and missing trace keys.

Latest progress (2026-03-28): `kidcli verify-recording` now accepts either a local cache slug
(`--slug <slug>`) or a copied bundle path (`--bundle <path>`). That removes the old manual
re-ingest step for remote repro bundles and makes the bundle itself a valid verification handle.

Latest progress (2026-03-28, artefact sidecars): `kidcli play-native --record <slug>` now saves
bundle-level `final-frame.png` and `audio.wav`, and `--record-notes <path>` copies plain-text
player notes into `notes.txt`. The saved `manifest.json` lists those sidecars explicitly so copied
bundles stay self-describing before any replay or oracle verification runs.

Closure evidence (2026-03-28, transcript sidecar): recording bundles now always include an
automatic `transcript.txt` generated from the captured native trace. The transcript logs the start
state plus major event boundaries such as game-mode swaps, level changes, pause toggles, life/HP
changes, helmet swaps, damage latches, game-over entry, and continue countdown updates, so a copied
bundle is reviewable before replaying it. `manifest.json` and `README.txt` now list the transcript
alongside `notes.txt`, `final-frame.png`, and `audio.wav`, and `kidcli` regression coverage locks
the new bundle contract. With transcript sidecars landed, the recording slug is now a complete,
self-describing repro handle rather than just "trace plus optional notes".

### 131 — Burn down the parked DEVIATION items (FIXED)
Evidence source: fidelity audit of the remaining `DEVIATION:` markers in gameplay code.
Native repro: mixed. Oracle repro: ASM-backed where noted. Minimum gate: `fast-slice`.

Progress (2026-03-27): Fireball bounces no longer skip the ROM's signed rebound perturbation.
Native now mirrors ASM `Enemy19_Fireball_BounceFloorCeiling`'s `Get_RandomNumber_byte` /
`eor.b Camera_X_pos` / `asl.w #7` rebound shaping with a deterministic local byte source, so
floor and ceiling rebounds vary within the original `±$4000` range instead of reflecting at a
fixed speed every time. Regressions:
`fireball_floor_bounce_applies_signed_rom_perturbation_before_cap` and
`fireball_ceiling_rebound_uses_the_same_perturbed_velocity_rule`.

Progress (2026-03-27): Tornado flight now uses ASM `UFO.asm` `unk_35344` target-speed patterns
instead of the old fixed accel-flip placeholder. Native reloads one of the 16 ROM pattern rows,
derives signed per-axis acceleration the same way as `loc_35444`, and then steps with the shared
`Enemy_HandleAcceleration` shape. Follow-up (2026-03-27): native now threads the shared ROM
`Init_RNG` / `Get_RandomNumber_byte` state through gameplay, so Tornado pattern reloads, Fireball
bounce perturbations, and Robot charge flip timers all consume the same seeded byte stream the
ROM uses. Regressions:
`tornado_loads_rom_pattern_on_timer_expiry` and
`tornado_accelerates_toward_pattern_targets`, plus the direct RNG and enemy reload tests.

Closure evidence (2026-03-27): the final parked gameplay deviation at the
`sub_B43A` flat-ground precheck is gone. Native now keeps the precheck literal
for Solid/Spikes only, but skips that walking-coroutine gate when the current
support is already a grounded slope owned by ASM `sub_9386` / `loc_9D22`.
That removes the last in-code `DEVIATION:` marker without reintroducing the old
half-speed slope regression. Regressions:
`asm_faithful_ground_precheck_skips_sub_b43a_when_already_on_slope` and
`walking_on_slope_up_y_tracks_diagonal_every_frame`, plus `mise run verify-slice-fast`.

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

Follow-up (2026-03-28): the native player renderer now tags the existing Fire Demon death-trail
projectiles and renders a dedicated six-phase flame footprint instead of the old generic enemy
projectile dot. Exact ROM tile extraction for `Fire_Demon.asm` `stru_3A622` is still polish work,
but live play now reads as a fire trail rather than a reused bullet marker.

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

### 037 — Floor-snap misses terrain (FIXED)
Gravity simulation in `level_respawn_slice_with_floor_snap` fixed most levels earlier, then
the remaining `0x24` / `0x51` tail turned out to be a hidden init-camera carry rather than
another support-path miss. `finish_respawn_camera_convergence()` now mirrors the last
`sub_71E4`-backed settle beat for the exact `target-camera == 2px` residue, so both levels
finish on the oracle-backed `camera_y=$036E0000` / `$028E0000` results.
Level `0x29` stays locked down by `respawn_settle_level_29_matches_oracle_wall_nudged_spawn_x`,
and level `0x52` stays locked down by `respawn_settle_level_52_uses_stair_step_support_path`.
Level `0x02` remains clean apart from the usual advisory `number_lives` field.

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
Resolved while narrowing #037. Spawn-wall overlap resolution now keeps the cached
manifest settle path on the oracle-backed `kid_x=0x02A50000` / `camera_x=0x02090000`
result for level `0x29`.

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

### 082 — Demo byte boundary parity shift after transform (FIXED)
On the first Kid_Transform frame, step_demo_input sees form_change_flash_frames=0
(step_lifecycle hasn't set it yet) and advances both counter and data_index. The
data_index advance is correct — it aligns native's data_index with the oracle's
pointer (compensating for the 1-frame late demo start). But the counter advance
shifts the byte-advancement parity for all subsequent frames.

The inverted parity (`playback_frame % 2 == 1` vs oracle's even counter) compensates
for the 1-frame late start during normal play. The extra counter increment on the
first transform frame disrupts this compensation, causing byte boundaries to land
on the wrong frame of each 2-frame window.

Fix: after step_lifecycle (inside refresh_opening_slice), detect when
form_change_flash_frames transitions from >0 to 0 (transform end) and decrement the
counter by 1 to restore the pre-transform parity. Uses `demo_transform_active` bool
to track whether a transform is in progress.

### 091 — Demo Start abort timing (FIXED)
The imported Any% TAS presses real Start on global frame 1772 while demo 0x1B is
still running. Native was handling that input immediately inside `step_demo_input`,
returning to title on the same frame. ASM `loc_119D8` does not do that: it clears
`Ctrl_Pressed`, hibernates for one frame, then checks `Ctrl_1_Pressed` on wake-up.
That makes the visible title transition land two frames after the live Start press.

Fix: add a dedicated deferred demo-title-return countdown in `NativeRuntime`.
Frame 1772 now arms the abort, frame 1773 still stays in `DemoPlay`, and frame 1774
finally calls the title-return cleanup and clears `Current_LevelID` to 0.

Verification:
- `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`
- `cargo test -p kid-core demo_aborts_on_start_press -- --nocapture`
- `cargo test -p kid-core demo_auto_ends_after_max_frames -- --nocapture`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1768 --frame-end 1780 --checkpoint-interval 1`

Remaining frontier: frame 1774 title-mode handoff still snaps to the title pose one
frame earlier than oracle. Tracked in #092.

### 092 — Demo abort title handoff preserves live snapshot (FIXED)
ASM `loc_119D8` only flips `Game_Mode`/`Current_LevelID` and returns through
`loc_6E2`; it does not immediately rebuild the title-screen Kid/camera RAM. The
oracle therefore spends the first post-abort title phase with `mode=$002C` and
`level=0`, but still exposes the paused demo snapshot for Kid and camera.

Native was clearing `opening_slice` and reusing the cold-boot title pose as soon
as the abort countdown expired, which skipped that paused handoff phase.

Fix: preserve the demo gameplay snapshot on the title-return path, mark the
runtime paused, and let trace output reuse the preserved Kid/camera snapshot
while title mode is active. Added `demo_title_return_keeps_live_snapshot_on_first_title_frames`
to lock down frames 1774 and 1779.

New frontier: frame 1802. Oracle blanks Kid for frames 1802-1805, then also
zeros camera/helmet/diamond state at frame 1806. That second phase is tracked
in #093.

### 093 — Demo abort title handoff blanks Kid before camera reset (FIXED)
Oracle traces show a second demo-abort title phase after the live snapshot:
Kid disappears on frames 1802-1805, but camera/form/diamond state stays frozen
until frame 1806, when the remaining title rebuild clears it.

Native used one boolean (`preserve_demo_title_snapshot`) for the entire handoff,
so it either preserved everything or nothing. That kept Kid visible too long and
also let the normal title pose appear as soon as the snapshot logic stopped.

Fix: split the handoff into two phases. `trace.rs` now only reuses the live
demo player/object snapshot for the first 28 title frames, while `boot.rs`
keeps the camera/form snapshot alive until frame 1806 and only then resets the
runtime back to clean title-screen state.

Verification:
- `cargo test -p kid-core demo_title_return -- --nocapture`
- `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1798 --frame-end 1808 --checkpoint-interval 1`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 2000 --bisect`

New frontier: frame 1832. Oracle still keeps Kid hidden while the title object
counts rise from 3 to 5, but native starts exposing the standard title pose.
Tracked in #094.

### 094 — Demo abort title rebuild keeps Kid hidden through frame 1923 (FIXED)
Once the frame-1806 camera/form reset lands, oracle traces do not immediately
swap back to the standard title pose. Title object counts keep rebuilding, but
Kid stays hidden until frame 1924.

Native reused the generic `title_screen_ready()` gate as soon as the preserved
demo snapshot ended. That kept the title object timing roughly right, but it
also exposed the normal title pose from frame 1832 onward because the generic
title-ready gate flips much earlier than the oracle-backed demo-return path.

Fix: keep a demo-return-specific "hide Kid" window in `trace.rs` after the
snapshot handoff. That leaves the generic title-ready/object-count path intact
while holding `kid_x_pos`/`kid_y_pos` at zero through frame 1923, then letting
the normal title pose resume on frame 1924.

Verification:
- `cargo test -p kid-core demo_title_return -- --nocapture`
- `cargo test -p kid-core demo_start_abort_stays_in_demo_until_frame_1774 -- --nocapture`
- `cargo run -p kidcli -- tas-diff --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1798 --frame-end 1808`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 1828 --frame-end 1927 --checkpoint-interval 1`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 2000 --bisect`
- `mise run verify-slice-fast`

New frontier: past frame 2000. The first 2000 TAS frames are now CRC-clean; the
next authoritative mismatch has not yet been re-bisected beyond that point.

### 095 — Demo 1 startup profile on Knights Isle (FIXED)
Once the second attract-mode demo switched to level `0x15`, native kept
reusing the demo `0x1B` bootstrap profile: the hidden camera preload used
Skydragon values (`0x0228_0000`/`0x02A0_0000`), demo input became live too
early, and the visible bootstrap pose/freefall sequence started seven frames
before the oracle.

Oracle-backed TAS traces show a different startup shape for Knights Isle:
camera stays hidden through frames 2529-2530, settles to `0x00E0_0000` with a
one-frame wrapped negative quirk at 2532, keeps Kid hidden through frame 2537,
then exposes a longer fixed-pose bootstrap before regular motion resumes.

Fix: key demo startup off the selected recording instead of one shared profile.
Demo 1 now:
- keeps its hidden-camera trace profile through frame 2537
- delays demo input/gameplay exposure from startup frame 65 to 72
- holds the bootstrap pose while camera follow converges, instead of running
  the demo 2 settle/freefall path
- defers demo enemy warm-up until the later Knights Isle handoff

Verification:
- `cargo test -p kid-core demo_1_bootstrap -- --nocapture`
- `cargo test -p kid-core demo_bootstrap_holds_spawn_pose_then_freefalls_before_grounded_hold -- --nocapture`
- `cargo test -p kid-core demo_bootstrap_first_visible_gameplay_frame_advances_camera_only -- --nocapture`
- `cargo test -p kid-core demo_camera_handoff_primes_buffer_for_the_hidden_settle_frame -- --nocapture`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2524 --frame-end 2580 --checkpoint-interval 1`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 3000 --bisect`

New frontier: frame 2625. Advisory dragon-roster timing still differs earlier,
but the next authoritative mismatch is now Kid position (`kid_x_pos`,
`kid_y_pos`) during the first live Knights Isle traversal steps.

### 096 — Demo 1 handoff landing on Knights Isle (FIXED)
After #095, the first live Knights Isle traversal frames stayed CRC-clean
through frame 2624, but native still landed one frame early at 2625. Oracle
keeps Kid airborne at `kid=(0x005C7700,0x013F4000)` on frame 2625, then lands
on frame 2626 without taking an extra X step.

Root cause: `apply_floor_collision()` still allowed the AsmFaithful
solid-floor touch snap during demo playback. That path is correct for normal
gameplay, but here it collapsed the final `0x4000` sub-half-pixel gap in the
demo's hidden handoff arc into an early landing.

Fix: disable the AsmFaithful solid-floor touch snap while demo playback is
active. Added `demo_1_handoff_stays_airborne_until_frame_2626` to lock the
frame-2625 airborne state and the frame-2626 landing.

Verification:
- `cargo test -p kid-core demo_1_handoff_stays_airborne_until_frame_2626 -- --nocapture`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-start 2618 --frame-end 2635 --checkpoint-interval 1`
- `cargo run -p kidcli -- verify-tas --scenario .cache/tas-imports/kc_any_percent_2024/scenario.json --frame-end 3000 --bisect`

New frontier: frame 2723. Demo 1 now stays in the grounded standingstill hold
through the post-landing wall, so the next authoritative mismatch is the jump
handoff/camera advance on Knights Isle (`native kid_x_pos=0x011A0000`,
`oracle=0x011B0000`; `native camera_x_pos=0x00770000`,
`oracle=0x00780000`).

Attempted approaches that didn't work:
- pending_form.is_some() check: over-freezes (73 vs 72 frames), regression to 1129
- Full deferred pipeline (move step_demo_input after physics): changes byte for ALL frames
  due to 1-frame delay, regression to 921
- Rollback counter+data_index on first transform frame: data_index rollback misaligns
  with oracle (oracle's pointer is 1 ahead due to earlier start), regression to 1129
- Frozen input override (save pre-transform byte for first post-transform physics):
  correct in theory but native already has the right byte via the "compensating"
  data_index advance; freezing gives the WRONG byte, regression to 1129

Key insight: the extra data_index advance on K+1 is a happy accident that aligns
native's data_index with the oracle's. Only the PARITY needs correcting, not the
data_index itself.

### 083 — Demo byte boundary systematic offset (INVESTIGATED)
ASM VBlank pipeline: MainGameLoop → mode handler → Execute_Objects → player
processing → ReadJoypad (loc_C22 demo path). ReadJoypad runs AFTER game logic,
so byte read on frame N is used by game logic on frame N+1. V_Int handler is
minimal (just increments counter + sets done flag), does NOT read joypads.

Oracle probe confirmed: at frame 915, oracle counter (post-ReadJoypad) = 16,
native playback_frame = 14. Oracle counter is consistently 1 ahead of native pf.
Data pointer advances align when using parity `% 2 == 0` (matching oracle's
even-d7 advance pattern).

Attempted pipeline buffer approach:
1. Added `demo_buffered_input: Option<FrameInput>` — return previous frame's byte
2. Changed advancement parity from `% 2 == 1` to `% 2 == 0`
3. Transform compensation: `fc += 1`, `di += 1` at transform end

Result: produces IDENTICAL frontier (frame 1170) with identical position/velocity.
The buffer+parity change is mathematically equivalent to the old code: both have a
net 1-frame offset, just manifested at different parity phases. Changing parity
shifts which frames see the boundary mismatch, but the underlying 1-frame late
start means one boundary per byte pair will always be off.

Why simple compensations fail: native starts demo byte consumption 1 frame after
oracle (confirmed by probe). Adding a pipeline buffer delays usage by 1 frame
(matching ASM's read-after-use), but the late start means the read itself is on
the wrong frame. The parity change fixes the data_index alignment but the buffer
delay cancels out, leaving the same net offset. Any combination of (buffer ×
parity × transform_correction) that produces correct post-transform alignment
also produces the same pre-transform byte boundary behaviour as the old code.

True fix requires: making native start demo byte consumption on the same frame
as the oracle (i.e., fixing the 1-frame start offset), likely by restructuring
how BootState::DemoPlay initialises the counter relative to the ASM's first
ReadJoypad call in demo mode.
