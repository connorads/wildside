// === Game timing ===

pub(crate) const SEGA_SCREEN_FRAMES: usize = 120;
pub(crate) const INTRO_FIRST_SEGMENT_FRAMES: usize = 160;
pub(crate) const INTRO_FIRST_SEGMENT_DIRECT_TITLE_FRAMES: usize = 100;
pub(crate) const INTRO_SEGMENT_FRAMES: usize = 80;
pub(crate) const TITLE_TO_LEVEL_FRAMES: usize = 60;
/// Demo/title-return lineages still use the short local title attract lock
/// before the rebuilt title pose becomes ready.
pub(crate) const TITLE_ATTRACT_LOCK_FRAMES: usize = 58;
/// ASM Obj_TitleMenu `Title_InputLoop` writes the next `Game_Mode` on the
/// frame after Start is latched; native queues the selection on the press
/// frame, then consumes it on the next outer frame.
pub(crate) const TITLE_TO_LEVEL_SELECTION_DELAY_FRAMES: usize = 0;
/// ASM Load_IntroSequence6 `loc_1BAB0` seeds `d0=$B8` then `dbf d0,loc_1BAB4`,
/// so the mode-$28 timeout-restart title rebuild hibernates for 185 frames
/// before `loc_1BAEA` even seeds the shared `$280` title idle loop.
pub(crate) const INTRO_SEQUENCE_6_TITLE_ATTRACT_LOCK_FRAMES: usize = 185;
/// TODO(fidelity): ASM `Load_IntroSequence6 -> loc_1BAB0 -> loc_1BAEA`
/// clearly owns the later `Game_Mode=$10` flip, but the published TAS oracle
/// keeps the timeout-restart mode-$28 rebuild alive 45 frames beyond the raw
/// `$280` DBF window. Keep the oracle-backed 686-frame hold here until the
/// exact pre-`loc_1BB02` gate is traced from the ROM.
pub(crate) const INTRO_SEQUENCE_6_TITLE_IDLE_FRAMES: u16 = 0x02AE;
pub(crate) const OPENING_GROUNDED_CAMERA_Y_FP: u32 = 0x0073_0000;
// The opening Dragon hit lands earlier under the walk-only gameplay script than
// under the run-held trace used by the focused timing regression.
// Oracle boot-to-level traces use an early intro-skip sequence, then one
// Start press to arm the title menu and a later Start press to select 1P.
pub(crate) const BOOT_TO_LEVEL_START_FRAMES: [usize; 7] = [33, 77, 135, 137, 178, 360, 460];
pub(crate) const TITLE_KID_X_POS: u32 = 0x002E_0000;
pub(crate) const TITLE_KID_Y_POS: u32 = 0x008C_0000;
// Oracle-backed published TAS traces expose the first intro-sequence-3 Kid
// pose on frame 5994 after a 41-frame hidden lead-in. The visible coordinates
// now come from the ROM's rotating Load_IntroSequence3 roster in
// `movement.rs`, not from a single shared hard-coded pose.
pub(crate) const INTRO_SEQUENCE_3_KID_VISIBLE_ELAPSED_FRAMES: usize = 41;
// Oracle-backed demo-abort traces split the return-to-title handoff into two
// phases: Kid disappears first, then the preserved camera/form state drops a
// few frames later when the title objects fully take over.
pub(crate) const DEMO_TITLE_FULL_SNAPSHOT_FRAMES: usize = 28;
pub(crate) const DEMO_TITLE_PARTIAL_SNAPSHOT_FRAMES: usize = 32;
// The ROM auto-timeout branch keeps the frozen gameplay snapshot alive one
// title frame longer than the manual Start-abort helper before Kid blanks,
// then clears camera/progression one frame later as well.
pub(crate) const DEMO_AUTO_TIMEOUT_FULL_SNAPSHOT_FRAMES: usize = 29;
pub(crate) const DEMO_AUTO_TIMEOUT_PARTIAL_SNAPSHOT_FRAMES: usize = 33;
// ASM loc_1B5CE has already restored the standard title pose before the later
// timeout-restart demo starts. When that lineage eventually returns to title,
// oracle ends the generic hidden rebuild window two frames earlier than the
// other demo-timeout paths and starts exposing the standard title pose on the
// last two local title frames of that window.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_SEQUENCE_6_EARLY_TITLE_RESTORE_FRAMES: usize = 2;
// Oracle-backed published TAS traces keep the timeout restart in the latched
// mode-0 Sega handoff through local frame 176, then switch to intro sequence 1
// on local frame 177 (trace frame 5143).
pub(crate) const DEMO_AUTO_TIMEOUT_MODE_0_FRAMES: usize = 177;
// Oracle-backed published TAS traces keep the timeout-restart copy of intro
// sequence 1 alive through frame 5503 and only expose `game_mode=0x0018` on
// frame 5504, so this restart path cannot reuse the cold-boot 160-frame timer.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_FIRST_SEGMENT_FRAMES: usize = 361;
// Oracle-backed Game_Mode probes keep the timeout-restart copy of intro
// sequence 2 alive through frame 5952 and only expose `game_mode=0x001C` on
// frame 5953, so this restart path cannot reuse the shared 80-frame timer.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_SECOND_SEGMENT_FRAMES: usize = 449;
// ASM sub_1B93E / loc_1B96A keep the timeout-restart copy of intro sequence 3
// polling the Start-gated sub_1C246 return until the natural timeout hands off
// to `Game_Mode=$20`. Oracle-backed published TAS traces keep the second visit
// alive through frame 9354 and only expose intro sequence 4 on frame 9355, so
// the native boot-state model needs a 299-frame local timer here rather than
// the earlier 256-frame approximation.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_THIRD_SEGMENT_FRAMES: usize = 299;
// ASM loc_1B96A flips `Game_Mode` to intro sequence 4 immediately, but the
// previous intro-sequence-3 roster objects remain visible until the intro-4
// setup fully takes over. Oracle-backed timeout-restart traces keep the second
// roster pose alive through frames 9355..9381 and only clear it on 9382.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_4_POSE_CARRY_FRAMES: usize = 27;
// ASM Load_IntroSequence4 only allocates the `sub_1B988` object; the first
// visible `move.w #4,(Camera_Y_pos).w` does not surface until frame 9392 in
// the timeout-restart published TAS window, after a short zero-camera gap.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES: usize = 37;
// ASM sub_1C204 loops on `Hibernate_Object_1Frame`, `addi.w #$10,
// ($FFFFF876).w`, `cmpi.w #$100,($FFFFF876).w`, so the fade helper idles for
// 17 frames before it finally dispatches `$44(a5)` to loc_1B9B2.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_4_FADE_FRAMES: usize = 17;
// ASM sub_1B988 / sub_1C204 / loc_1B9B2 do not start the
// `move.w #$21B,d0` / `dbf` countdown until the delayed `sub_1B988` worker
// reaches the end of the 17-frame fade helper. On the timeout-restart path
// oracle keeps the carried intro-sequence-3 pose alive for 27 frames,
// exposes the `Camera_Y_pos=4` hand-off on frame 9392, idles inside
// `sub_1C204`, then holds `Game_Mode=$20` for the full `0x21C`-frame
// `loc_1B9B2` loop after that dispatch. Native therefore needs the camera
// hand-off delay, the `sub_1C204` fade delay, and the `loc_1B9B2` loop
// length; counting only from the visible camera hand-off exits 17 frames too
// early.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_FOURTH_SEGMENT_FRAMES: usize =
    TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES
        + TIMEOUT_RESTART_INTRO_SEQUENCE_4_FADE_FRAMES
        + 0x021C;
// ASM Load_IntroSequence5 only allocates the `loc_1BA28` / `loc_1BCD0`
// workers; unlike Load_IntroSequence6 and GfxObjects_Collision_BottomBoundary4
// it does not call `sub_1B222` or otherwise rewrite `Camera_Y_pos`. Oracle
// timeout-restart traces therefore keep the carried `sub_1B988` camera only
// for the first 31 intro-sequence-5 frames, through frame 9979, and clear it
// on frame 9980.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_5_CAMERA_CARRY_FRAMES: usize = 31;
// ASM sub_1C246 returns the later timeout-restart intro-sequence-4 revisit to
// `Game_Mode=$2C`, but title rebuild traces only preserve sub_1B988's
// `Camera_Y_pos=4` hand-off for the first 32 local title frames. Oracle clears
// the carried camera on frame 32039, so keep the title-screen carry bounded to
// that short settle window rather than the full title rebuild.
pub(crate) const TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_4_CAMERA_CARRY_FRAMES: usize = 32;
// The later intro-sequence-4 Start return still reaches title through
// `sub_1C246 -> Game_Mode=$2C`, but oracle-backed published TAS traces expose
// the rebuilt title pose one local title frame earlier than the generic
// `enter_title_screen_from_intro_return` settle window.
pub(crate) const TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_4_EARLY_READY_FRAMES: usize = 1;
// ASM Obj_TitleMenu `loc_1BAEA` / `loc_1BB02` count down the final title idle
// loop after the later intro-sequence-4 pose is already visible. Because that
// lineage restores `loc_1B5CE` one frame early, oracle also reaches the
// `$2C -> $10` demo hand-off one frame earlier than the shared title arm.
pub(crate) const TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_4_EARLY_ATTRACT_LOCK_FRAMES: usize = 1;
// ASM loc_1BCD0 writes the third allocated title gfx slot with
// `move.w #0,$1A(a1)` / `move.w #$E0,$1E(a1)`. Oracle probes of the verifier's
// fixed slot-2 WRAM reads ($FF24FA/$FF24FE) keep that slot zero through frame
// 10006, then expose the loc_1BCD0 slot-2 coordinates on frame 10007 while
// intro sequence 5 is still active. Start-skip intro-return lineage handles
// its one-frame-earlier revisit separately in `trace.rs`.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_5_SLOT_2_VISIBLE_ELAPSED_FRAMES: usize = 58;
pub(crate) const INTRO_SEQUENCE_5_SLOT_2_X_POS: u32 = 0x0000_0000;
pub(crate) const INTRO_SEQUENCE_5_SLOT_2_Y_POS: u32 = 0x00E0_0000;
// ASM loc_1BA28 dispatches to the shared fade helper sub_1C204 before
// loc_1BA5C starts its long intro-sequence-5 timeout. Oracle traces of the
// published TAS later timeout-restart revisit keep `Game_Mode=$24` alive
// through frame 20893 and only reach loc_1BA5C's `move.w #$28,(Game_Mode).w`
// hand-off on 20894, so the older `17 + 0x1BD` approximation was 59 frames
// short on the real timeout path.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_FRAMES: usize = 17 + 0x01BD + 59;
// The later Start-skip intro-return lineage revisits the timeout-restart copy
// of intro sequence 5 one shared sub_1C204 fade window earlier than the plain
// timeout path. Oracle keeps `Game_Mode=$24` alive through frame 16649 and
// only lets the frame-16648 Start press arm sub_1C246's shared 2-frame title
// return, so native must not expire this lineage before one extra fade window
// elapses.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_START_SKIP_LINEAGE_EXTRA_FRAMES: usize =
    TIMEOUT_RESTART_INTRO_SEQUENCE_4_FADE_FRAMES;
// ASM Load_IntroSequence6 no longer allocates loc_1BA28 / loc_1BCD0; it
// switches to the new `loc_1BAB0` / `loc_1B37C` / `sub_1B43A` title-shell
// workers immediately, so the carried slot-2 pose in the verifier's fixed
// WRAM read is just stale state. Oracle-backed published TAS traces keep that
// stale pose alive through frame 20911 and clear it on 20912, i.e. after 18
// local mode-$28 frames.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_SLOT_2_POSE_CARRY_FRAMES: usize = 18;
// ASM Load_IntroSequence6 seeds `($FFFFA656).w` with `$FFD00000`, then
// `sub_1B222` / `GfxObjects_Collision_BottomBoundary4` keep rewriting
// `Camera_Y_pos` from that hidden title-camera accumulator instead of the
// generic cleared title camera. Oracle-backed timeout-restart traces first
// expose the wrapped `0xFFF0_0000` row on the local mode-$28 frame 32.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_WRAP_ELAPSED_FRAMES: usize = 32;
// Oracle-backed probes of the later title-restore revisit show the same
// `Load_IntroSequence6 -> sub_1B222` seed eventually reappears, but only
// after the cleared camera survives through local mode-$28 frame 34. Frame
// 28101 is the first later-revisit frame where `($FFFFA656).w` and
// `Camera_Y_pos` both become `$FFD00000`.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_SEEDED_CAMERA_ELAPSED_FRAMES: usize = 35;
// ASM sub_1B43A eventually allocates GfxObjects_Collision_BottomBoundary4,
// which waits through the title-shell animation and then applies
// `sub.l d1,($FFFFA656).w; bsr.w sub_1B222` for 16 frames. Starting from the
// wrapped `$FFF0_0000` accumulator, oracle ramps the hidden camera upward by
// one pixel per frame beginning on local mode-$28 frame 248.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_ELAPSED_FRAMES: usize = 248;
// The later title-restore revisit keeps the delayed `$FFD00000` seed flat for
// three extra local mode-$28 frames before that same worker starts. Because
// `($FFFFA656).w >> 4` becomes `$FFFD0000`, the later ramp moves by +3 pixels
// per frame from local mode-$28 frame 251.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_CAMERA_RAMP_ELAPSED_FRAMES: usize = 251;
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_CAMERA_RAMP_FRAMES: usize = 16;
// ASM loc_1B5CE is the first timeout-restart mode-$28 worker that writes the
// standard title pose directly (`move.w #$2E,x_pos(a3); move.w #$8C,y_pos(a3)`).
// Oracle-backed traces keep Kid hidden through local mode-$28 frame 329 and
// only expose that pose on frame 330, while `Game_Mode` still remains `$28`.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_TITLE_POSE_VISIBLE_ELAPSED_FRAMES: usize = 330;
// On the later title-restore revisit, the same `loc_1B5CE` pose does not
// surface until three local mode-$28 frames after the earlier timeout-restart
// lineage. Oracle-backed traces keep Kid hidden through frame 332 and only
// expose the standard title pose on frame 333.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_TITLE_POSE_VISIBLE_ELAPSED_FRAMES: usize =
    333;
// ASM Obj_TitleMenu `loc_1BAEA` / `loc_1BAEE` still own the later
// `Game_Mode=$28 -> $10` transition after the title pose is already visible.
// Oracle-backed published TAS traces keep the later title-restore revisit in
// mode `$28` through frames 29402..29404 and only let `loc_1BB02` fire on
// frame 29405, i.e. three local mode-$28 frames after the earlier lineage.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_DEMO_HANDOFF_EXTRA_FRAMES: u16 = 3;
// ASM sub_1C246 writes `Game_Mode=$2C` and returns immediately, so the
// loc_1BCD0 slot-2 pose survives into title mode until the later title rebuild
// overwrites that WRAM slot 28 frames later.
pub(crate) const TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_POSE_CARRY_FRAMES: usize = 28;
// After the carried intro-sequence-5 slot-2 pose clears, oracle keeps title
// slot 2 blank through frame 10437 and only restores the standard title pose
// on 10438.
pub(crate) const TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_HIDDEN_FRAMES: usize = 148;
// The later Start-skip intro-return lineage revisits the same
// `sub_1C246 -> Game_Mode=$2C` title hand-off, but oracle WRAM probes keep the
// standard title pose hidden through frame 16798 and only restore it on 16799.
// Model that extra rebuild frame separately instead of shifting the plain
// timeout-restart path that already matches frame 10438.
pub(crate) const TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_START_SKIP_LINEAGE_EXTRA_HIDDEN_FRAMES:
    usize = 1;
// After the partial handoff, the title objects rebuild for much longer before
// the standard title pose returns. Oracle TAS traces keep Kid hidden through
// local title frame 149 and only expose the pose again on frame 150.
pub(crate) const DEMO_TITLE_PLAYER_HIDDEN_FRAMES: usize = 150;
// The timeout-restart intro-sequence-3 carry-over keeps the intro pose alive
// for the first 28 title frames, so the follow-up hidden rebuild window ends
// one frame earlier than the older demo-abort path.
pub(crate) const TIMEOUT_RESTART_TITLE_PLAYER_HIDDEN_FRAMES: usize = 149;
// Oracle-backed TAS traces show the title screen reaches a "ready" state only
// once the overall boot timeline has progressed far enough. Synthetic
// bootstrap scripts reach title much later and are ready immediately on entry,
// while power-on TAS recordings can enter title earlier and remain inert until
// frame 287.
pub(crate) const TITLE_SCREEN_READY_GLOBAL_FRAME: usize = 287;
// Early intro-skip traces do not make the title pose visible immediately on
// entry. The title rebuild still needs 150 local frames before Kid appears,
// even when the absolute boot-ready frame has already nearly been reached.
pub(crate) const TITLE_SCREEN_EARLY_ENTRY_LOCAL_READY_FRAMES: usize = 150;
pub(crate) const OPENING_GROUNDED_KID_Y_POS: u32 = 0x010F_0000;
// === Enemy contact ===

// Default enemy contact box: 16x16 pixels, centred on position.
pub(crate) const ENEMY_CONTACT_HALF_WIDTH: i64 = 4;
// Oracle-backed Level 1 traces show MiniHoppingSkull side contact is narrower
// than the full 16px sprite; otherwise the late skull arc in death-respawn
// registers a false graze hit.
pub(crate) const MINI_HOPPING_SKULL_CONTACT_HALF_WIDTH: i64 = 1;
pub(crate) const ENEMY_CONTACT_HALF_HEIGHT: i64 = 8;
pub(crate) const ENEMY_CONTACT_MIN_OVERLAP_FP: i64 = 0x0001_0000;

// === Player death/respawn timing ===

// ASM `Death` (kid.asm:14714) drives a fixed tumble arc, but exits as soon as
// the object falls out of the locked camera window rather than after a long
// fixed hold. Keep a generous cap here and let the runtime end the arc early.
pub(crate) const PLAYER_DEATH_WINDUP_FRAMES: usize = 159;
// Oracle death traces switch from the hidden death shell into the respawn
// bootstrap after roughly half a second, not the older 120-frame placeholder.
pub(crate) const PLAYER_DEATH_HIDE_FRAMES: usize = 32;
// Oracle-backed `lose_life` / `Teleport` traces keep the frozen post-Death
// shell pose visible only for the first 21 hidden frames before Kid's WRAM
// coordinates clear while the respawn shell still holds camera/object state.
pub(crate) const PLAYER_DEATH_HIDDEN_SHELL_VISIBLE_FRAMES: usize = 21;
// Oracle-backed `lose_life -> Teleport -> loc_6E2` traces keep the frozen
// shell camera for five extra hidden frames after Kid's WRAM pose clears, then
// zero Camera_X/Y before the later respawn bootstrap takes over.
pub(crate) const PLAYER_DEATH_HIDDEN_CAMERA_VISIBLE_FRAMES: usize = 26;
// Oracle-backed death/respawn traces keep Kid on the hidden respawn settle path
// far longer than the old 428-frame placeholder: the canonical
// `death-respawn-level1` window compare still reports the mode-8 bootstrap hold
// pose through frame 9000. Keep the shared respawn shell alive through that
// lower bound; the later visible `Load_InGame` return is still a separate
// frontier.
pub(crate) const PLAYER_DEATH_RESPAWN_SETTLE_FRAMES: usize = 7_210;
// ASM `lose_life -> Teleport -> loc_6E2` keeps Level 1 on a hidden mode-8
// bootstrap before `Load_InGame` later restores the grounded respawn pose.
// Oracle-backed traces slide from `(x=$160,y=$44)` to the usual hold pose
// `(x=$40,y=$68)` over 65 frames, then idle there until the visible settle
// resumes.
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_ANIMATION_FRAMES: usize = 65;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_INITIAL_X_FP: u32 = 0x0160_0000;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_INITIAL_Y_FP: u32 = 0x0044_0000;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_X_FP: u32 = 0x0040_0000;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_HOLD_Y_FP: u32 = 0x0068_0000;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_X_STEP_FP: i32 = -0x0004_8000;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_Y_STEP_FP: i32 = 0x0000_9000;
// ASM loc_1AE2E -> loc_6E2 -> Load_InGame on the early title-start lineage
// keeps Level 1 hidden for 51 local frames, briefly preloads Camera_Y on the
// `$00E0_0000` and wrapped `$FF00_0000` rows, then exposes the respawn slice
// on local frame 60 before the settled `$0073_0000` camera arrives.
pub(crate) const EARLY_START_LOAD_INGAME_CAMERA_PRELOAD_FRAME: usize = 51;
pub(crate) const EARLY_START_LOAD_INGAME_CAMERA_WRAP_FRAME: usize = 52;
pub(crate) const EARLY_START_LOAD_INGAME_VISIBLE_FRAME: usize = 60;
pub(crate) const EARLY_START_LOAD_INGAME_HANDOFF_FRAMES: usize = 122;
// ASM `loc_1AE2E -> loc_6E2` leaves Level 1's early title-start lineage on a
// hidden mode-$0008 bootstrap that slides from slot 2's seeded `loc_1AC70`
// pose toward the same `(x=$40,y=$68)` hold used by the later hidden bootstrap
// shell before the visible `Load_InGame` settle frames begin.
pub(crate) const EARLY_START_LEVEL_ONE_BOOTSTRAP_INITIAL_X_FP: u32 = 0x0128_0000;
pub(crate) const EARLY_START_LEVEL_ONE_BOOTSTRAP_INITIAL_Y_FP: u32 = 0x0100_0000;
pub(crate) const EARLY_START_LEVEL_ONE_BOOTSTRAP_X_STEP_FP: i32 = -0x0003_A000;
pub(crate) const EARLY_START_LEVEL_ONE_BOOTSTRAP_Y_STEP_FP: i32 = -0x0002_6000;
pub(crate) const EARLY_START_LEVEL_ONE_BOOTSTRAP_ANIMATION_FRAMES: usize = 8;
pub(crate) const EARLY_START_LEVEL_ONE_BOOTSTRAP_HOLD_X_FP: u32 = 0x010E_A000;
pub(crate) const EARLY_START_LEVEL_ONE_BOOTSTRAP_HOLD_Y_FP: u32 = 0x00EF_6000;
// ASM Load_TitleCard allocates loc_1AC70 immediately on the early title-start
// lineage too, but the later Start arrives much sooner than on the cold boot
// path. Oracle therefore exposes slot 2's seeded loc_1AC70 pose on frame 172,
// i.e. 34 local frames after mode-$0008 begins on frame 138.
pub(crate) const EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME: usize = 34;
// ASM Obj_TitleMenu `Title_InputLoop` jumps straight to `loc_6E2` after
// writing `Game_Mode=$0008`. Oracle-backed early-start traces keep the last
// cursor pose visible for 23 traced mode-8 frames (frames 138..160), then
// clear the player WRAM view on frame 161.
pub(crate) const EARLY_START_TITLE_MENU_CURSOR_POSE_CARRY_FRAMES: usize = 23;
// ASM `loc_6E2` clears the rest of the title shell later than the cursor pose:
// the 4-object/3-gfx shell persists for 28 traced mode-8 frames
// (frames 138..165) before the hidden bootstrap drops to 0/0 on frame 166.
pub(crate) const EARLY_START_TITLE_MENU_SHELL_CARRY_FRAMES: usize = 28;
// After the title shell clears, oracle-backed early-start traces keep the
// mode-$0008 bootstrap fully empty for two frames (166..167), then expose the
// first hidden preload workers from frame 168 until the later camera preload.
pub(crate) const EARLY_START_LOAD_INGAME_PRELOAD_OBJECTS_FRAME: usize = 30;
// ASM `Make_SpriteFromGfxObject` checks the death sprite against the normal
// `$160/-$80` Y visibility window using the death frame's vertical extent, not
// a generic "viewport + 160px" threshold. `kid_death.bin` carries height
// `$1B`, so the hidden death shell does not latch until that full footprint
// falls below the camera window.
pub(crate) const PLAYER_DEATH_SPRITE_Y_EXTENT_PX: i32 = 0x1B;
pub(crate) const SPRITE_VISIBILITY_BOTTOM_MARGIN_PX: i32 = 0x80;
pub(crate) const SPRITE_VISIBILITY_TOP_MARGIN_PX: i32 = 0x80;
pub(crate) const SPRITE_VISIBILITY_RIGHT_MARGIN_PX: i32 = 0x40;
pub(crate) const SPRITE_VISIBILITY_LEFT_MARGIN_PX: i32 = 0x40;
/// ASM: Game Over display holds for 0x1A3 (419) frames.
pub(crate) const PLAYER_GAME_OVER_HOLD_FRAMES: usize = 419;
/// ASM: Continue countdown: 28 initial pause + 291 countdown = 319 frames.
pub(crate) const CONTINUE_COUNTDOWN_FRAMES: usize = 319;
pub(crate) const PLAYER_LEVEL_ADVANCE_RESPAWN_SETTLE_FRAMES: usize = 401;
// ASM: Just_received_damage flag cleared by flicker object (loc_BC54):
// outer loop dbf d3 with d3=$A (11 iterations), 6 frames per iteration = 66 frames.
pub(crate) const PLAYER_DAMAGE_INVULNERABLE_FRAMES: usize = 66;

// === Collision / activation ===

pub(crate) const FLAG_HIT_HALF_SIZE: u32 = 8; // 16x16 hit region → 8px either side of centre
pub(crate) const LEVEL_COMPLETE_PAUSE_FRAMES: usize = 120; // brief pause before advancing
// ASM Manage_EnemyLoading (kid.asm loc_368A8): spawn window is
// x=[camera-0x80, camera+0x1C0], y=[camera-0x80, camera+0x160].
pub(crate) const ACTIVATION_WINDOW_X_LEAD: u32 = 0x1C0;
pub(crate) const ACTIVATION_WINDOW_Y_LEAD: u32 = 0x160;
pub(crate) const ACTIVATION_WINDOW_PADDING: u32 = 0x80;
pub(crate) const FLYER_RETENTION_X_PADDING: u32 = 512;
pub(crate) const FLYER_RETENTION_Y_PADDING: u32 = 96;
pub(crate) const PLATFORM_PRELOAD_X_PADDING: u32 = 64;

// === Player hitbox ===

// Player collision hitbox for "The Kid" (form 0).
// From ASM hitbox data: dc.w -$5, $A, -$1F, $1F
// x_offset=-5, x_size=10, y_offset=-31, y_size=31
// Position is at feet, so hitbox extends half_width either side and height above.
pub(crate) const PLAYER_HALF_WIDTH: i32 = 5;
pub(crate) const PLAYER_HEIGHT: i32 = 31;
pub(crate) const GROUND_SNAP_TOLERANCE_FP: u32 = 0x0006_0000;
pub(crate) const JUMP_BUFFER_FRAMES: u8 = 6;
/// Post-ledge-snap pull-up animation: 7 anim frames × 3 VBlanks = 21 frames.
/// ASM: stru_8BB4 at kid.asm:10014, driven by Hibernate_UntilAnimFinished.
pub(crate) const LEDGE_SNAP_PULLUP_FRAMES: u8 = 21;
pub(crate) const SLOPE_AIR_SNAP_TOLERANCE_FP: u32 = 0x0003_0000;

// === Slope physics ===

pub(crate) const KID_SLOPE_MAX_SPEED_FP: i32 = 0x0004_0000;
pub(crate) const KID_SLOPE_BRAKE_FP: i32 = 0x0000_4000;
pub(crate) const KID_SLOPE_NEUTRAL_DECEL_FP: i32 = 0x0000_2000;
pub(crate) const KID_SLOPE_NEUTRAL_TARGET_FP: i32 = -0x0000_8000;
pub(crate) const KID_SLOPE_WALK_UPHILL_CAP_FP: i32 = 0x0000_8000;
pub(crate) const KID_SLOPE_WALK_UPHILL_ACCEL_FP: i32 = 0x0000_1000;
pub(crate) const KID_SLOPE_WALK_DOWNHILL_CAP_FP: i32 = -0x0001_8000;
pub(crate) const KID_SLOPE_WALK_DOWNHILL_ACCEL_FP: i32 = 0x0000_2000;
pub(crate) const KID_SLOPE_RUN_UPHILL_CAP_FP: i32 = 0x0001_0000;
pub(crate) const KID_SLOPE_RUN_UPHILL_ACCEL_FP: i32 = 0x0000_2000;
pub(crate) const KID_SLOPE_RUN_DOWNHILL_CAP_FP: i32 = -0x0003_0000;
pub(crate) const KID_SLOPE_RUN_DOWNHILL_ACCEL_FP: i32 = 0x0000_4000;
pub(crate) const MICROMAX_SLOPE_MAX_SPEED_FP: i32 = 0x0002_0000;
pub(crate) const MICROMAX_SLOPE_BRAKE_FP: i32 = 0x0000_4000;
pub(crate) const MICROMAX_SLOPE_NEUTRAL_DECEL_FP: i32 = 0x0000_1000;
pub(crate) const MICROMAX_SLOPE_NEUTRAL_TARGET_FP: i32 = -0x0000_4000;
pub(crate) const MICROMAX_SLOPE_WALK_UPHILL_CAP_FP: i32 = 0x0000_4000;
pub(crate) const MICROMAX_SLOPE_WALK_UPHILL_ACCEL_FP: i32 = 0x0000_0800;
pub(crate) const MICROMAX_SLOPE_WALK_DOWNHILL_CAP_FP: i32 = -0x0000_C000;
pub(crate) const MICROMAX_SLOPE_WALK_DOWNHILL_ACCEL_FP: i32 = 0x0000_1000;
pub(crate) const MICROMAX_SLOPE_RUN_UPHILL_CAP_FP: i32 = 0x0000_8000;
pub(crate) const MICROMAX_SLOPE_RUN_UPHILL_ACCEL_FP: i32 = 0x0000_1000;
pub(crate) const MICROMAX_SLOPE_RUN_DOWNHILL_CAP_FP: i32 = -0x0001_8000;
pub(crate) const MICROMAX_SLOPE_RUN_DOWNHILL_ACCEL_FP: i32 = 0x0000_2000;
pub(crate) const SPECIAL_FORM_SLOPE_STEP_FP: i32 = 0x0000_1000;

// === Combat ===

// Stomp bounce: ~60% of a standstill jump (Kid jump_height 0x42, factor 0x100).
// Full jump y_vel = -(0x42 * 0x100) << 4 = -0x420_000. 60% ≈ -0x270_000.
// ASM Jump_On_Enemy restores Kid to the prior frame's anchor and starts the
// next frame with a fixed -4 px bounce (`move.l #$FFFC0000,d7`).
pub(crate) const STOMP_BOUNCE_VELOCITY: i32 = -0x0004_0000;
// Frames an enemy flashes white after taking a non-lethal stomp.
pub(crate) const ENEMY_DAMAGE_FLASH_FRAMES: u8 = 8;
// DragonWalking hibernates after a surviving stomp instead of resuming patrol
// on the next frame. The active TAS frontier keeps layout #02 parked from frame
// 2939 through 2962 before the right-facing patrol resumes on 2963.
pub(crate) const DRAGON_WALKING_NONLETHAL_STOMP_HIBERNATE_FRAMES: u8 = 24;

// === Evanescent blocks ===

// Evanescent blocks: solid until the player stands on them, then fade and vanish.
// Timer starts at (skin - 28); incremented each frame; block removed at timer ≥ 22.
pub(crate) const EVANESCENT_SKIN_OFFSET: i16 = 0x1C; // 28
pub(crate) const EVANESCENT_REMOVAL_THRESHOLD: i16 = 0x16; // 22
pub(crate) const EVANESCENT_MAX_ENTRIES: usize = 19; // ASM limit $13

// === Level timer ===

// Level timer: every level starts at 3:00 and counts down 1 second per 60 frames.
// Time-up = death. Clocks add 3 minutes (capped at 10:00).
pub(crate) const LEVEL_TIMER_START_SECONDS: u16 = 180; // 3:00
pub(crate) const LEVEL_TIMER_MAX_SECONDS: u16 = 600; // 10:00
pub(crate) const LEVEL_TIMER_CLOCK_BONUS_SECONDS: u16 = 180; // +3:00
pub(crate) const LEVEL_TIMER_FRAMES_PER_SECOND: u16 = 60;

// === Scoring ===

pub(crate) const SCORE_CAP: u32 = 9_999_999;
/// ASM: FirstElsewhere_LevelID = $49. Path/speed bonuses only for levels < this.
pub(crate) const FIRST_ELSEWHERE_LEVEL_ID: u8 = 0x49;
/// ASM: extra life awarded every 50,000 points.
pub(crate) const SCORE_EXTRA_LIFE_INTERVAL: u32 = 50_000;

/// Per-level speed bonus thresholds in seconds (ASM: SpeedBonus_Values).
/// If level completion time (elapsed seconds) <= threshold, award 10,000 pts.
/// 0 means speed bonus is impossible for that level.
#[rustfmt::skip]
pub(crate) const SPEED_BONUS_TABLE: [u8; 76] = [
    20, 22, 60, 27, 19, 26, 91, 17, 54, 37, 23, 28, 35, 26, 30, 28, // 0x00-0x0F
    75, 42, 84,171,125, 48, 60, 64,103, 92, 54, 28, 32, 43,108, 60, // 0x10-0x1F
    71, 27,174, 96, 50, 72, 60, 82, 41, 19,220,  0,  0, 50,  0, 60, // 0x20-0x2F
     0, 68,125, 80, 28,  0, 15,  0,  0,  0,  0, 87,  0, 98,  0,  0, // 0x30-0x3F
     0, 95,  0, 28, 30,  0,  0,  0,  0,  0,  0,  0,                  // 0x40-0x4B
];

/// Per-level path bonus multipliers × 1000 (ASM: PathBonus_Values).
/// Non-zero only for later levels. Multiplied by 1000 to get point value.
#[rustfmt::skip]
pub(crate) const PATH_BONUS_TABLE: [u8; 76] = [
     0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 0x00-0x0F
     0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 0x10-0x1F
     0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, 25,  0, 25, // 0x20-0x2F
     0, 25, 25, 25, 10,  0, 10,  0,  0,  0,  0, 20,  0, 50,  0,  0, // 0x30-0x3F
     0, 25,  0, 25, 25,  0,  0,  0,  0,  0,  0,  0,                  // 0x40-0x4B
];

// === Lava geysers ===

pub(crate) const LAVA_GEYSER_HOLD_FRAMES: u16 = 30;
pub(crate) const LAVA_GEYSER_DORMANT_FRAMES: u16 = 60;
pub(crate) const LAVA_GEYSER_ERUPT_SPEED: u16 = 4; // pixels per frame

// === Ending sequence ===

/// Number of frames for the ending sequence phases (ASM End_Credits, kid.asm:16920).
pub(crate) const ENDING_PAUSE_FRAMES: usize = 40; // Sleep 0x28
pub(crate) const ENDING_MUSIC_DELAY_FRAMES: usize = 200; // Sleep 0xC8
pub(crate) const ENDING_FADE_OUT_FRAMES: usize = 256; // fade counter 0x100
/// Each credits screen: 112 frames scroll + 200 frames display.
pub(crate) const CREDITS_SCROLL_FRAMES: usize = 112;
pub(crate) const CREDITS_DISPLAY_FRAMES: usize = 200;
pub(crate) const CREDITS_SCREEN_COUNT: usize = 7; // congratulations + 6 credits screens
/// Total credits duration per screen.
pub(crate) const CREDITS_SCREEN_TOTAL: usize = CREDITS_SCROLL_FRAMES + CREDITS_DISPLAY_FRAMES;
// === Projectiles ===

/// Maximum active projectiles (original limits to 2 arrows, we allow a few more
/// for Tank bullets sharing the pool).
pub(crate) const MAX_PROJECTILES: usize = 4;
/// Projectile hitbox half-width for player collision (pixels).
pub(crate) const PROJECTILE_HITBOX_HALF: i32 = 6;

// === Skycutter ===

/// Skycutter flipboard counter threshold: animation complete at 32.
pub(crate) const SKYCUTTER_FLIPBOARD_TIMEOUT: u8 = 0x20;
/// Counter value where the player toggles on/off the board.
pub(crate) const SKYCUTTER_MOUNT_FRAME: u8 = 0x11;
/// Counter increment per frame.
pub(crate) const SKYCUTTER_COUNTER_STEP: u8 = 2;
/// Gravity while on the board (B held): lighter descent.
pub(crate) const SKYCUTTER_GRAVITY_B_HELD: i32 = 0x2000;
/// Gravity while on the board (B not held): normal descent.
pub(crate) const SKYCUTTER_GRAVITY_NO_B: i32 = 0x4000;
/// Upward bounce velocity when landing while on the board.
pub(crate) const SKYCUTTER_LANDING_BOUNCE: i32 = -0x3_0000;
/// Max upward velocity while on the board ($FFF88000 signed = -0x78000).
pub(crate) const SKYCUTTER_MAX_UPWARD_VEL: i32 = -0x7_8000;

// === RedStealth ===

/// RedStealth sword: C press spawns a melee hitbox 28px ahead, 10px up
/// from the player. 8 frames startup, 3 frames active. One swing at a time.
pub(crate) const RED_STEALTH_STARTUP_FRAMES: u8 = 8;
pub(crate) const RED_STEALTH_ACTIVE_FRAMES: u8 = 3;
pub(crate) const RED_STEALTH_REACH_X: i32 = 28;
pub(crate) const RED_STEALTH_OFFSET_Y: i32 = -10;
pub(crate) const RED_STEALTH_HIT_HALF: i32 = 8;

// === Maniaxe ===

/// Maniaxe thrown axe: travels 12 frames outward at ±5 px/frame then
/// reverses for 12 frames. Damages enemies on contact, breaks on walls.
pub(crate) const MANIAXE_STARTUP_FRAMES: u8 = 6;
pub(crate) const MANIAXE_BASE_SPEED: i32 = 5;
pub(crate) const MANIAXE_TRAVEL_FRAMES: u8 = 12;

// === Eyeclops ===

/// Eyeclops beam: C press fires a horizontal beam projectile.
/// Normal beam (C button) has no diamond cost; hard beam (A+B held) costs 2.
pub(crate) const EYECLOPS_BEAM_SPEED: i32 = 0x8_0000; // 8 px/frame
pub(crate) const EYECLOPS_BEAM_LIFETIME: u16 = 60;

// === Juggernaut ===

/// Juggernaut skull shot: C fires a skull projectile with gravity.
pub(crate) const JUGGERNAUT_SKULL_X_SPEED: i32 = 0x4_0000; // 4 px/frame
pub(crate) const JUGGERNAUT_SKULL_GRAVITY: i32 = 0x1000; // $1000/frame gravity on y_vel
pub(crate) const JUGGERNAUT_SKULL_LIFETIME: u16 = 120;
pub(crate) const JUGGERNAUT_MAX_SKULLS: usize = 8;
pub(crate) const JUGGERNAUT_SKULL_STARTUP_FRAMES: u8 = 3;
pub(crate) const JUGGERNAUT_SKULL_OFFSET_X: i32 = 26;
pub(crate) const JUGGERNAUT_SKULL_OFFSET_Y: i32 = -8;
pub(crate) const JUGGERNAUT_SKULL_BOUNCES: u8 = 4;

// === Berzerker ===

/// Berzerker charge: accumulator increments by $10 each frame when velocity >= 1.0 FP.
/// Charging activates when accumulator overflows past $FF.
pub(crate) const BERZERKER_CHARGE_VEL_THRESHOLD: i32 = 0x1_0000;
pub(crate) const BERZERKER_CHARGE_ACCUM_STEP: u8 = 0x10;

// === Cyclone ===

/// Cyclone flight: acceleration-based vertical flight.
/// Per-frame gravity added to acceleration accumulator.
pub(crate) const CYCLONE_ACCEL_GRAVITY: i16 = 0x400;
/// Upward thrust subtracted from accumulator when C is held.
pub(crate) const CYCLONE_ACCEL_THRUST: i16 = 0x3000;
/// Minimum acceleration (max upward thrust) — $8001 as signed i16 = -32767.
pub(crate) const CYCLONE_ACCEL_MIN: i16 = -0x7FFF;
/// Maximum descent acceleration when C not held.
pub(crate) const CYCLONE_ACCEL_MAX_DESCENT: i16 = 0x1200;
/// Initial acceleration on entering flight (set on jump).
pub(crate) const CYCLONE_ACCEL_INITIAL: i16 = 0x2000;
/// Max downward velocity during Cyclone flight.
pub(crate) const CYCLONE_MAX_FALL_VEL: i32 = 0x2_8000;
/// Max upward velocity during Cyclone flight.
pub(crate) const CYCLONE_MAX_RISE_VEL: i32 = -0x3_0000;

// === IronKnight ===

/// IronKnight wall climb: C press subtracts this from Y acceleration.
pub(crate) const IRON_KNIGHT_CLIMB_BOOST: i16 = 0x1800;
/// IronKnight wall climb: natural per-frame drift toward sliding down.
pub(crate) const IRON_KNIGHT_SLIDE_DRIFT: i16 = 0x80;
/// IronKnight wall climb: max slide-down acceleration (no C pressed).
pub(crate) const IRON_KNIGHT_MAX_SLIDE_ACCEL: i16 = 0x1800;
/// IronKnight wall climb: min acceleration (fast climbing upward).
pub(crate) const IRON_KNIGHT_MIN_ACCEL: i16 = -0x0800; // $F800 as signed
/// IronKnight wall climb: initial Y velocity on attach.
pub(crate) const IRON_KNIGHT_ATTACH_Y_VEL: i32 = -0x4000;
/// IronKnight wall climb: max upward velocity.
pub(crate) const IRON_KNIGHT_MAX_CLIMB_VEL: i32 = -0x1_8000;
/// IronKnight wall climb: max downward velocity (falling off).
pub(crate) const IRON_KNIGHT_MAX_FALL_VEL: i32 = 0x4_0000;

// === Micromax ===

/// Micromax hitbox: half-size character (16-pixel grid alignment per ASM).
pub(crate) const MICROMAX_HALF_W: i32 = 3;
pub(crate) const MICROMAX_H: i32 = 15;
/// Micromax stuck gravity: $600 per frame (much lower than normal $4000).
pub(crate) const MICROMAX_STUCK_GRAVITY: i32 = 0x600;
/// Micromax max fall speed before peeling off surface.
pub(crate) const MICROMAX_STUCK_MAX_FALL: i32 = 0x1_0000;
/// Micromax detach Y velocity: -4.0 FP (strong upward kick).
pub(crate) const MICROMAX_DETACH_Y_VEL: i32 = -0x4_0000;
/// Micromax detach X speed: 2.0 FP (kick away from surface).
pub(crate) const MICROMAX_DETACH_X_SPEED: i32 = 0x2_0000;

// === Murder wall ===

/// Murder wall acceleration per frame (kid.asm:7456 `addi.l #$100`).
pub(crate) const MURDER_WALL_ACCELERATION: i32 = 0x0100; // ASM-verified
/// Default max speed (kid.asm:26889 `move.l #$20000`).
pub(crate) const MURDER_WALL_DEFAULT_MAX_SPEED: i32 = 0x2_0000; // ASM-verified
/// Slower max speed for Hills of the Warrior 1 (kid.asm:26892 `move.l #$18000`).
pub(crate) const MURDER_WALL_SLOW_MAX_SPEED: i32 = 0x1_8000; // ASM-verified
/// Initial offset from camera (kid.asm:26898 `addi.w #$30`).
pub(crate) const MURDER_WALL_CAMERA_OFFSET_PX: i32 = 0x30; // ASM-verified

// === Level skip ===

pub(crate) const LEVEL_SKIP_FREEZE_VISIBLE_FRAMES: usize = 21;
pub(crate) const LEVEL_SKIP_HIDE_PLAYER_FRAMES: usize = 5;
pub(crate) const LEVEL_SKIP_BLANK_FRAMES: usize = 2;
pub(crate) const LEVEL_SKIP_SMALL_PRELOAD_FRAMES: usize = 3;
pub(crate) const LEVEL_SKIP_LARGE_PRELOAD_FRAMES: usize = 1;
pub(crate) const LEVEL_SKIP_ANIMATION_FRAMES: usize = 64;
pub(crate) const LEVEL_SKIP_ANIMATION_X_STEP_FP: u32 = 0x0004_8000;
pub(crate) const LEVEL_SKIP_ANIMATION_Y_STEP_FP: u32 = 0x0001_5000;
pub(crate) const LEVEL_SKIP_RESUME_VISIBLE_FRAMES: usize = LEVEL_SKIP_FREEZE_VISIBLE_FRAMES;
pub(crate) const LEVEL_SKIP_RESUME_HIDE_PLAYER_FRAMES: usize = LEVEL_SKIP_HIDE_PLAYER_FRAMES;
pub(crate) const LEVEL_SKIP_RESUME_PRELOAD_FRAMES: usize = 1;
pub(crate) const LEVEL_SKIP_RESUME_BLANK_FRAMES: usize = TITLE_TO_LEVEL_FRAMES
    - LEVEL_SKIP_RESUME_VISIBLE_FRAMES
    - LEVEL_SKIP_RESUME_HIDE_PLAYER_FRAMES
    - LEVEL_SKIP_RESUME_PRELOAD_FRAMES;
pub(crate) const LEVEL_SKIP_RESUME_ENTRY_FRAMES: usize = 20;
pub(crate) const LEVEL_SKIP_RESUME_TOTAL_FRAMES: usize =
    TITLE_TO_LEVEL_FRAMES + LEVEL_SKIP_RESUME_ENTRY_FRAMES;
pub(crate) const LEVEL_ONE_SKIP_RESUME_ENTRY_PLAYER_Y_FP: [u32; LEVEL_SKIP_RESUME_ENTRY_FRAMES] = [
    0x009F_0000,
    0x00A0_0000,
    0x00A0_4000,
    0x00A0_C000,
    0x00A1_8000,
    0x00A2_8000,
    0x00A3_C000,
    0x00A5_4000,
    0x00A7_0000,
    0x00A9_0000,
    0x00AB_4000,
    0x00AD_C000,
    0x00B0_8000,
    0x00B3_8000,
    0x00B6_C000,
    0x00BA_4000,
    0x00BE_0000,
    0x00BF_0000,
    0x00BF_0000,
    0x00BF_0000,
];
pub(crate) const LEVEL_ONE_SKIP_RESUME_ENTRY_CAMERA_Y_FP: [u32; LEVEL_SKIP_RESUME_ENTRY_FRAMES] = [
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0001_0000,
    0x0002_0000,
    0x0003_0000,
    0x0004_0000,
    0x0005_0000,
    0x0006_0000,
    0x0008_0000,
    0x000A_0000,
    0x000C_0000,
    0x000E_0000,
    0x0010_0000,
    0x0012_0000,
    0x0014_0000,
];

// === Sine/Cosine lookup table ===

/// ASM word_402B0 (kid.asm:45677): 65-entry sine table for angles 0-90 degrees.
/// Entry i contains sin(π/2 × i/64) × $4000, clamped at $FFFF for i >= 65.
/// ASM sub_40236 (kid.asm:45620) uses this table for angle-to-sin/cos conversion.
#[allow(dead_code)]
pub(crate) const SINE_TABLE_WORD_402B0: [u16; 65] = [
    0x0000, 0x0192, 0x0324, 0x04B5, 0x0646, 0x07D5, 0x0964, 0x0AF1, 0x0C7C, 0x0E06, 0x0F8D, 0x1112,
    0x1294, 0x1413, 0x158F, 0x1707, 0x187D, 0x19EF, 0x1B5D, 0x1CC7, 0x1E2B, 0x1F8B, 0x20E7, 0x223C,
    0x238E, 0x24DA, 0x2620, 0x2760, 0x2899, 0x29CD, 0x2AFB, 0x2C21, 0x2D40, 0x2E5A, 0x2F6C, 0x3076,
    0x317A, 0x3276, 0x336A, 0x3452, 0x3538, 0x3614, 0x36E4, 0x37B0, 0x3872, 0x392C, 0x39DC, 0x3A82,
    0x3B20, 0x3BB8, 0x3C42, 0x3CC6, 0x3D40, 0x3DAE, 0x3E16, 0x3E72, 0x3EC4, 0x3F0E, 0x3F4E, 0x3F84,
    0x3FB0, 0x3FD2, 0x3FEC, 0x3FFA, 0x3FFE,
];
