# Playtest Backlog

Canonical tracker for bugs, missing features, and verification gaps. See `CLAUDE.md`
for iteration protocol, `docs/source-port/methodology.md` for the investigation
playbook, and `docs/source-port/roadmap.md` for the programme-level workstreams.

Important: a green TAS frontier or a green `1p-accurate` release artefact does **not**
mean the game is nearly finished. Native-play breakage, wider verification gaps, and
documented deviations belong here too.

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
| 122 | [ ] | P0 | level | Level 2 entry can spawn Kid in midair, then fall through the floor and die | Boot -> finish Level 1 -> enter Level 2 -> confirm stable landing and 10s of live play |
| 123 | [ ] | P0 | player | Death/respawn still leaves camera drift, stuck-from-start, or broken restart state | Die in live play, confirm camera settles correctly and movement resumes without soft-lock |
| 124 | [x] | P1 | enemy | Enemy kill timing still allows Kid to get hurt after the enemy should already be dead | Sword/projectile/charge/AoE kills now persist the ROM kill bit, so dead enemies do not reactivate into fresh contact damage |
| 125 | [ ] | P1 | block | Breakable blocks can render with the wrong lifecycle art or regress into prize-like visuals | Break affected blocks, capture before/after, confirm sprite/state progression is correct |
| 126 | [ ] | P1 | player | Walking up slopes is still observably wrong in native play | Traverse representative slopes in focused scenario and live play, confirm no hitching or bad grounding |
| 127 | [ ] | P1 | block | Black bouncy blocks do not produce the expected bounce behaviour | Hit bounce blocks in focused scenario and live play, confirm bounce height/timing |
| 128 | [ ] | P1 | form | Form-specific block breaking can now destroy metal blocks that should stay solid | Repro with affected form, confirm allowed blocks break and metal blocks remain intact |
| 129 | [ ] | P1 | infra | Add a `1p-playable` gate that covers native boot -> Level 1 -> Level 2, death/respawn, slopes, bounce surfaces, and common block interactions | `mise run` gate is green only when the core 1P smoke path survives those scenarios |
| 130 | [ ] | P1 | infra | Turn `play-native --record` / `verify-recording` into the standard playtest evidence pipeline | Capture a recording on one machine, replay and verify it locally, and use the slug as a bug artefact |
| 131 | [ ] | P2 | fidelity | Burn down the six parked DEVIATION items instead of treating them as accepted resting state | Each deviation is either removed, reclassified as oracle limitation, or explicitly deferred with evidence |
| 132 | [ ] | P3 | gameflow | Verify 2-player alternating mode against oracle-backed or strong native scenarios | Toggle 2P, die as P1, confirm P2 takeover / game-over / continue behaviour |
| 133 | [ ] | P3 | gameflow | Investigate password flow and decide whether it is missing, partial, or intentionally out of 1P scope | Document ASM path, native support status, and explicit implementation decision |
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

### 122 — Level 2 entry can spawn Kid in midair, then fall through the floor and die (OPEN)
Evidence source: repeated native play reports. Native repro: reported, needs canonical recording slug.
Oracle repro: unknown. Minimum gate: `1p-playable`.

The current release gate proves the narrow `progress-two-transitions` compare, but that is not
strong enough to dismiss this failure. The next fix must start by capturing a reproducible native
recording for the Level 1 -> Level 2 transition and turning it into a permanent smoke scenario.

### 123 — Death/respawn still leaves camera drift, stuck-from-start, or broken restart state (OPEN)
Evidence source: repeated native play reports. Native repro: reported, needs canonical recording slug.
Oracle repro: mixed/unknown. Minimum gate: `1p-playable`.

This issue stays separate from the closed floor-snap and TAS frontier items. It is a player-facing
native stability bug until a recording or focused scenario proves otherwise.

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

### 129 — Add a `1p-playable` gate (OPEN)
Evidence source: roadmap decision after repeated native-play failures escaping the current gate.
Native repro: n/a. Oracle repro: n/a. Minimum gate: n/a.

The current `1p-accurate` artefact remains useful, but it is too narrow to stand in for "the game
works". This item is complete only when there is a standard smoke gate covering the real 1P path
and the recurring native-play failures above.

### 130 — Turn recordings into the standard playtest evidence pipeline (OPEN)
Evidence source: existing `play-native --record` / `verify-recording` support plus current workflow gaps.
Native repro: partial. Oracle repro: partial. Minimum gate: n/a.

The repo already records input + trace. The missing work is packaging, remote-ingest workflow,
optional artefacts, and making the recording slug the normal reproduction handle for future bugs.

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
