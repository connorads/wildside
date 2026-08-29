// === Game timing ===

// ASM Load_SegaScreen `loc_12E6C` burns 11 VBlanks, then `loc_12E96` burns
// another 129 VBlanks before writing `Game_Mode=$0004` at `loc_12EDE`.
// Oracle cold-boot traces also include the hidden startup shell before the
// logo object reaches those loops, so the first visible intro frame is 173.
pub(crate) const SEGA_SCREEN_FRAMES: usize = 174;
// ASM Mode_Standard -> Execute_Objects loc_E12 draws one random longword
// before dispatching the newly allocated loc_12E64 logo controller. Oracle
// traces expose the controller on frame 32 and the completed draw on frame 33.
pub(crate) const SEGA_SCREEN_FIRST_OBJECT_PASS_ELAPSED: usize = 33;
// ASM Load_IntroSequence1 `sub_1B8F4 -> sub_1C204 -> loc_1B924` uses the
// shared 17-frame fade helper followed by the `$12B` inclusive DBF wait.
// Oracle cold-boot probes keep mode `$0004` through frame 533 and enter
// intro sequence 2 on frame 534.
pub(crate) const INTRO_FIRST_SEGMENT_FRAMES: usize = 361;
/// First pre-step elapsed value where sub_1B8F4 has reached loc_1B924 and
/// polls sub_1C246. Oracle cold boot consumes the frame-77 Start edge here.
pub(crate) const INTRO_FIRST_SEGMENT_DIRECT_TITLE_FRAMES: usize = 100;
// ASM Load_IntroSequence2 `loc_1B9D0 -> loc_1B9F4 -> loc_1BA02` uses the
// same fade/text shell plus the `$149` inclusive DBF loop. Oracle cold-boot
// probes keep mode `$0018` through frame 982 and enter intro sequence 3 on
// frame 983.
pub(crate) const INTRO_SECOND_SEGMENT_FRAMES: usize = 449;
// ASM Load_IntroSequence3 `sub_1B93E -> sub_1C204 -> loc_1B96E` uses the
// shared fade helper, then the `$EF` inclusive DBF loop. Oracle cold-boot
// probes keep mode `$001C` through frame 1281 and enter intro sequence 4 on
// frame 1282.
pub(crate) const INTRO_THIRD_SEGMENT_FRAMES: usize = 299;
// ASM Load_IntroSequence4 `sub_1B988 -> sub_1C204 -> loc_1B9B6` exposes the
// camera hand-off after the same 37-frame setup used by timeout restarts,
// then waits through the shared fade and `$21B` inclusive DBF loop. Oracle
// cold-boot probes keep mode `$0020` through frame 1875 and enter intro
// sequence 5 on frame 1876.
pub(crate) const INTRO_FOURTH_SEGMENT_FRAMES: usize = 594;
// ASM Load_IntroSequence5 `loc_1BA28 -> sub_1C204 -> loc_1BA5C` uses the
// shared fade shell before the `$1BC` inclusive DBF loop. Cold-boot oracle
// probes keep mode `$0024` through frame 2395 and enter intro sequence 6
// (`Game_Mode=$0028`) on frame 2396.
pub(crate) const INTRO_FIFTH_SEGMENT_FRAMES: usize = 520;
/// ASM loc_1BA28 (kid.asm) reaches the live IntroSequence5 polling window only
/// after the hidden setup has exposed the first loc_1BCD0 gfx slots. Oracle
/// probes show cold Start taps before global frame 1933 are ignored, while a
/// frame-1933 tap publishes `Ctrl_1_Pressed=$80` on 1934 before loc_1C264.
pub(crate) const INTRO_SEQUENCE_5_START_POLL_ELAPSED_FRAMES: usize = 57;
/// Pre-step local elapsed at which timeout-restart IntroSequence5's
/// `loc_1BA28` workers wake inside a new outer
/// `Mode_Standard -> Execute_Objects loc_E12` pass. The same boundary exposes
/// the first loc_1BCD0 GFX slots and arms loc_1BA60's Start polling.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_FIFTH_RNG_FIRST_LIVE_ELAPSED: usize =
    INTRO_SEQUENCE_5_START_POLL_ELAPSED_FRAMES;
/// ASM Load_IntroSequence5 allocates loc_1BCD0 alongside loc_1BA28. On the
/// later Start-skip title-return lineage, oracle exposes loc_1BCD0's first gfx
/// slots - and therefore the enclosing loc_E12 object pass - one local frame
/// before the plain timeout-restart lineage.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_FIFTH_START_SKIP_RNG_FIRST_LIVE_ELAPSED: usize =
    DEMO_AUTO_TIMEOUT_INTRO_FIFTH_RNG_FIRST_LIVE_ELAPSED - 1;
/// Start-skip-lineage local elapsed whose `loc_1BC36` object pass chains
/// `sub_1C034(d4=4)` directly into `sub_1C0EE`. The combined decompression and
/// VRAM upload crosses the next VBlank, so oracle presents one frame without
/// returning to `Mode_Standard -> Execute_Objects loc_E12` for another draw.
/// The boundary follows the worker start plus its `$A0,$90,$0B,$0B`
/// hibernations; published-route oracle probes pin it at post-step elapsed 385.
pub(crate) const INTRO_SEQUENCE_5_START_SKIP_PANEL_UPLOAD_STALL_PRE_STEP_ELAPSED: usize =
    INTRO_SEQUENCE_5_START_POLL_ELAPSED_FRAMES + 1 + 0xA0 + 0x90 + 0x0B + 0x0B;
/// The ordinary timeout-restart IntroSequence5 lineage enters the same
/// `loc_1BC36 -> sub_1C034(d4=4) -> sub_1C0EE` presentation stall one local
/// frame later. Published frames 20758/20759 pin the live/frozen boundary.
pub(crate) const INTRO_SEQUENCE_5_PANEL_UPLOAD_STALL_PRE_STEP_ELAPSED: usize =
    INTRO_SEQUENCE_5_START_SKIP_PANEL_UPLOAD_STALL_PRE_STEP_ELAPSED + 1;
/// ASM loc_1BA5C writes `Game_Mode=$28`, then loc_1BA90 hibernates and
/// subtracts `$20` from the `$100` palette-fade word until it becomes
/// negative. Those nine inclusive resumes still return through the existing
/// Mode_Standard `Execute_Objects loc_E12` pass before loc_1BAAC enters
/// `loc_6E2 -> Load_IntroSequence6`.
pub(crate) const INTRO_SEQUENCE_6_ENTRY_FADE_OUTER_PASSES_LOC_1BA90: usize = 9;
/// Post-step local frame on which `Load_IntroSequence6` has returned after
/// allocating `loc_1BAB0`, `loc_1B37C`, and `sub_1B43A`, allowing the next
/// `Mode_Standard -> Execute_Objects loc_E12` pass to draw. Published-route
/// oracle probes resume on global frame 20927, local frame 33.
pub(crate) const INTRO_SEQUENCE_6_POST_LOADER_RNG_FIRST_LIVE_FRAME: usize = 33;
/// The first timeout-restart Load_IntroSequence6 worker shell presents two
/// frames earlier than the cold-boot reconstruction clock: oracle exposes
/// sub_1B43A's first sub_1BF24 mapping on local frame 33 instead of 35.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_EARLY_WORKER_PRESENTATION_ADVANCE_FRAMES: usize =
    2;
/// `loc_1B37C` is allocated beside `sub_1B43A`, but its first timeout-restart
/// presentation is four frames ahead of the cold reconstruction clock. Oracle
/// VRAM keeps every `sub_1B41C` upload on a three-frame boundary from published
/// frames 20942 through 21089; using the mapping worker's two-frame correction
/// leaves the lightning strip two frames late.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_EARLY_LIGHTNING_PRESENTATION_ADVANCE_FRAMES:
    usize = 4;
/// Cold boot and the later timeout revisit present `loc_1B37C/sub_1B41C` one
/// object pass ahead of the trace-facing title clock. Oracle FIFO traces place
/// the first active-display loop upload at local presentation elapsed 82 on
/// both lineages; retaining the raw elapsed selects the preceding flash frame.
pub(crate) const INTRO_SEQUENCE_6_LIGHTNING_PRESENTATION_ADVANCE_FRAMES: usize = 1;
/// The later early-title-restore revisit reaches `Load_IntroSequence6` with a
/// three-frame delayed worker shell: oracle keeps the complete RNG ring fixed
/// through local frames 33..35, exposes the three allocations on frame 35,
/// then resumes `Mode_Standard -> Execute_Objects loc_E12` on frame 36. Apply
/// the same offset to the later `loc_1B45A` upload stalls because they belong
/// to those newly allocated workers, not to the preceding `loc_1BA90` fade.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_WORKER_DELAY_FRAMES: usize = 3;
/// Post-step local frame consumed by loc_1B45A's first one-frame hibernate
/// after its 64-word palette blackout. The enclosing loc_E12 pass has already
/// drawn on the preceding frame, so the oracle RNG ring stays fixed here.
pub(crate) const INTRO_SEQUENCE_6_PALETTE_BLACKOUT_STALL_FRAME_LOC_1B470: usize = 224;
/// Post-step local frames during which loc_1B45A's `sub_1B532` tilemap rebuild
/// and `Load_TitleArt` decompression/uploads keep the current object pass
/// stretched across VBlanks. Oracle resumes the ordinary outer draw on 237.
pub(crate) const INTRO_SEQUENCE_6_TITLE_ART_STALL_FIRST_FRAME_LOC_1B494: usize = 226;
pub(crate) const INTRO_SEQUENCE_6_TITLE_ART_STALL_LAST_FRAME_LOAD_TITLE_ART: usize = 236;
pub(crate) const INTRO_SEGMENT_FRAMES: usize = 80;
pub(crate) const TITLE_TO_LEVEL_FRAMES: usize = 60;
/// Demo/title-return lineages still use the short local title attract lock
/// before the rebuilt title pose becomes ready.
pub(crate) const TITLE_ATTRACT_LOCK_FRAMES: usize = 58;
/// Oracle frame probes for the cold IntroSequence6 Start-skip route keep the
/// rebuilt title shell hidden from the frame-2634 `sub_1C246` handoff until
/// frame 2720, when the standard title text/Kid workers are visible.
pub(crate) const INTRO_SEQUENCE_6_START_SKIP_TITLE_READY_FRAMES: usize = 86;
/// Early cold IntroSequence6 Start taps consumed during loc_1BAB0 enter title
/// before the hidden title rebuild has advanced as far as the later frame-2634
/// route. A frame-2400 Start reaches `$2C` on frame 2402, and oracle first
/// exposes the standard title pose on frame 2552.
pub(crate) const INTRO_SEQUENCE_6_EARLY_START_SKIP_TITLE_READY_FRAMES: usize = 150;
/// The same cold IntroSequence6 Start-skip route draws the visible
/// `Load_TitleScreen` VDP shell before the trace-facing title pose is exposed.
/// Oracle frame 2700 is visually complete while state still reports
/// `title_screen_ready=false`; frame 2720 is the later trace-ready point.
pub(crate) const INTRO_SEQUENCE_6_START_SKIP_TITLE_VDP_READY_FRAMES: usize = 66;
/// ASM loc_1BAB0 -> sub_1C246 can jump to `Load_TitleScreen` during
/// IntroSequence6's hidden wait. Oracle frame probes for Start at 2400 blackout
/// by title-local frame 19, then expose the rebuilt title VDP on frame 58
/// before trace-facing title readiness.
pub(crate) const INTRO_SEQUENCE_6_EARLY_START_SKIP_TITLE_VDP_READY_FRAMES: usize =
    TITLE_ATTRACT_LOCK_FRAMES;
/// ASM loc_1BA60 -> Load_IntroSequence6 -> loc_1BAB0 can enter title through
/// sub_1C246 while the stale loc_1BCD0 slot-2 pose is still visible. Oracle
/// frame probes for a cold frame-2400 Start keep that pose through frame 2429
/// and clear it on 2430, i.e. 29 title-local elapsed frames.
pub(crate) const INTRO_SEQUENCE_6_START_SKIP_TITLE_INTRO_SEQUENCE_5_POSE_CARRY_FRAMES: usize = 29;
/// The same `$28` -> `$2C` Start-skip lineage keeps the old hidden workers
/// count-visible after the stale slot-2 pose has cleared. Oracle frame probes
/// keep four objects through frame 2433 and clear the count on 2434.
pub(crate) const INTRO_SEQUENCE_6_START_SKIP_TITLE_INTRO_SEQUENCE_5_COUNT_CARRY_FRAMES: usize = 33;
/// On the no-input cold IntroSequence6 path, oracle frame 2640 has already
/// passed `sub_1B43A`'s loc_1B45A palette-restore tail and restored
/// `Load_TitleArt`/`sub_1B532`, even though trace-facing title readiness
/// remains false until the later `loc_1B5CE` title-pose point.
pub(crate) const INTRO_SEQUENCE_6_NATURAL_TITLE_VDP_REBUILD_FRAMES: usize = 244;
/// ASM loc_1B45A resumes the outer object pass on local frame 237 after
/// `Load_TitleArt`, having allocated `loc_1B6AC` and `loc_1B5CE`. Both wait
/// `$1E` frames before their live reveal loops, so their first moving title
/// presentation begins on local frame 267 while Game_Mode remains `$28`.
pub(crate) const INTRO_SEQUENCE_6_NATURAL_TITLE_WORKER_START_FRAMES: usize = 267;
/// The natural no-input IntroSequence6 path has moved from the intermediate
/// `loc_1B45A` hidden-shell restore to the normal title VDP shell by oracle
/// frame 2700, while trace-facing title readiness remains false.
pub(crate) const INTRO_SEQUENCE_6_NATURAL_TITLE_VDP_READY_FRAMES: usize = 304;
/// Stock title-card levels ignore the early intro-skip Start taps, then let
/// the frame-178 press arm Obj_TitleMenu before the later frame-360 press
/// selects 1P and flips into the hidden level bootstrap on the next frame.
pub(crate) const TITLE_SCREEN_MENU_ARM_GLOBAL_FRAME: usize = 178;
/// ASM Obj_TitleMenu `Title_InputLoop` writes the next `Game_Mode` on the
/// frame after Start is latched; native queues the selection on the press
/// frame, then consumes it on the next outer frame.
pub(crate) const TITLE_TO_LEVEL_SELECTION_DELAY_FRAMES: usize = 1;
/// ASM Obj_TitleMenu `loc_1BBD6` jumps through `loc_6E2` into Load_TitleCard,
/// but the resident title/menu VDP remains visible while the hidden
/// Load_TitleCard/`loc_1AE2E` gate warms up. Oracle frame probes for the cold
/// Start route `2400,2700,2720` carry the menu shell through frame 2759 and
/// expose the title-card shell on frame 2760.
pub(crate) const TITLE_MENU_TO_TITLE_CARD_VDP_CARRY_FRAMES: usize = 38;
/// ASM Obj_TitleMenu `loc_1BB52` builds the interactive menu cursor before
/// entering `Title_InputLoop`; oracle TAS traces keep the attract countdown
/// inactive for this short setup window after the first Start press arms the
/// menu.
pub(crate) const TITLE_MENU_ACTIVATION_DELAY_FRAMES: usize = 15;
/// ASM Obj_TitleMenu `loc_1BB52` links the cursor GfxObject two object passes
/// after the Start edge, before the full menu/VDP activation delay finishes.
/// Raw published-route WRAM shows `$FFFF2578` joining on frame 28764 after the
/// frame-28762 Start press.
pub(crate) const TITLE_MENU_CURSOR_GFX_ALLOCATION_DELAY_FRAMES_LOC_1BB52: usize = 2;
/// ASM Load_IntroSequence6 allocates the `loc_1BAB0`, `loc_1B37C`, and
/// `sub_1B43A` workers together. Although `loc_1BAB0` has a `$B8` DBF wait,
/// oracle frame probes for the cold IntroSequence6 Start-skip route show the
/// Start latch is not consumed by `sub_1C246` until cold frame 2634, 237 local
/// frames after Game_Mode first becomes `$28` at frame 2396.
/// TODO(fidelity): this is the NATURAL/cold lineage value only; the extra 52
/// frames over the `loc_1BAB0` `$B8` DBF pre-roll have not been traced to a
/// specific ASM wait yet.
pub(crate) const INTRO_SEQUENCE_6_TITLE_ATTRACT_LOCK_FRAMES: usize = 237;
/// ASM Load_IntroSequence6 `loc_1BAB0` seeds `d0=$B8` then `dbf d0,loc_1BAB4`,
/// so the mode-$28 timeout-restart title rebuild hibernates for 185 frames
/// before `loc_1BAEA` even seeds the shared `$280` title idle loop. The
/// published any-percent TAS timeout-restart window (frames 21765..) verified
/// this lineage against the oracle's IntroSequence6 -> DemoPlay hand-off;
/// applying the cold-route 237 here delays the demo flip by 52 frames.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_6_TITLE_ATTRACT_LOCK_FRAMES: usize = 185;
/// ASM Obj_TitleMenu `loc_1BAEA -> loc_1BAEE` seeds `d0=$280`, then hibernates
/// once before each inclusive `dbf` pass.
pub(crate) const TITLE_IDLE_DBF_FRAMES_LOC_1BAEA: u16 = 0x0280 + 1;
/// ASM `loc_1B37C -> loc_1B3BC` runs the hidden title-shell lightning's central
/// `d1=$2C` DBF loop for 45 inclusive passes on the IntroSequence6 rebuild.
pub(crate) const INTRO_SEQUENCE_6_LIGHTNING_LOOP_FRAMES_LOC_1B3BC: u16 = 0x002C + 1;
/// The timeout-restart IntroSequence6 title rebuild reaches the demo hand-off
/// after the shared title-menu idle plus the ROM lightning loop that runs
/// alongside the hidden title shell.
pub(crate) const INTRO_SEQUENCE_6_TITLE_IDLE_FRAMES: u16 =
    TITLE_IDLE_DBF_FRAMES_LOC_1BAEA + INTRO_SEQUENCE_6_LIGHTNING_LOOP_FRAMES_LOC_1B3BC;
/// Plain cold-boot `Load_IntroSequence6 -> loc_1BAB0 -> loc_1BAEA` reaches
/// `loc_1BB02` on global frame 3270. Native starts counting the shared title
/// idle only after the trace-facing title arm point, so the effective cold
/// threshold is shorter than the later timeout-restart rebuild.
pub(crate) const COLD_INTRO_SEQUENCE_6_TITLE_IDLE_FRAMES: u16 = 637;
pub(crate) const OPENING_GROUNDED_CAMERA_Y_FP: u32 = 0x0073_0000;
// The opening Dragon hit lands earlier under the walk-only gameplay script than
// under the run-held trace used by the focused timing regression.
// Oracle boot-to-level traces use an early intro-skip sequence, then one
// Start press to arm the title menu and a later Start press to select 1P.
pub(crate) const BOOT_TO_LEVEL_START_FRAMES: [usize; 7] = [33, 77, 135, 137, 178, 360, 460];
pub(crate) const COLD_BOOT_TITLE_CARD_LAST_HIDDEN_FRAME_LOC_1AE2E: usize = 719;
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
/// ASM loc_119D8 -> Pal_FadeOut loc_28C6 presents the resident gameplay image
/// for 19 local title frames: four `$0100` presentation boundaries followed
/// by the 15 visible descending `$10` blend steps before VRAM is cleared.
pub(crate) const DEMO_TITLE_PALETTE_FADE_SCENE_FRAMES_LOC_28C6: usize = 19;
/// ASM Pal_FadeOut -> loc_6E2 clears VRAM after the fade. Oracle keeps the
/// `$082A` background colour visible through local frame 56 and presents the
/// first rebuilt title image on local frame 57.
pub(crate) const DEMO_TITLE_SOLID_FILL_END_ELAPSED_LOC_6E2: usize = 57;
/// ASM ReadJoypad loc_C0A returns a timed-out demo through Pal_FadeOut and
/// mode-0 Load_SegaScreen. Oracle retains the `$082A` VRAM-clear fill through
/// local frame 36 and presents the Sega VDP on local frame 37.
pub(crate) const DEMO_AUTO_TIMEOUT_SOLID_FILL_END_ELAPSED_LOC_6E2: usize = 37;
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
/// Post-step local frames whose mode-0 Sega loader reaches an outer
/// `Execute_Objects` pass. ASM Load_SegaScreen loc_12E64/loc_12E6C keeps the
/// controller inside its own VBlank loop between these passes; published TAS
/// WRAM probes expose the matching `Get_RandomNumber_long` draws at global
/// frames 4967 and 5003.
pub(crate) const DEMO_AUTO_TIMEOUT_MODE_0_RNG_LONG_FRAMES: [usize; 2] = [1, 37];
// ASM loc_1B924 hibernates intro sequence 1 for the full `#$12B+1` DBF loop
// before it hands off to `Game_Mode=$18`. Oracle-backed published TAS traces
// keep the timeout-restart copy of intro sequence 1 alive through frame 5503
// and only expose `game_mode=0x0018` on frame 5504, so this restart path
// cannot reuse the cold-boot 160-frame timer.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_FIRST_SEGMENT_FRAMES: usize = 361;
// ASM Load_IntroSequence2 allocates an object with `move.l #loc_1B9D0,4(a0)`.
// loc_1B9D0 stores `#loc_1C7A0,4(a0)` (render helper), `#IntroText2,$44(a0)`,
// `#loc_1B9F4,$44(a5)` (continuation), then calls `sub_1C204` (17-frame fade).
// loc_1B9F4 hibernates for `#$3C` (60) frames, then sets `d0=#$149` (329) for
// the `dbf` loop. loc_1BA02 runs `Hibernate_Object_1Frame`, `sub_1C1A2` twice,
// `sub_1C246`, and `sub_1C180` per iteration. The `dbf d0,loc_1BA02` executes
// 330 times (d0: $149→0). After the loop, `loc_1BA02+14` sets `Game_Mode=$1C`.
// Oracle-backed Game_Mode probes keep the timeout-restart copy of intro
// sequence 2 alive through frame 5952 and only expose `game_mode=0x001C` on
// frame 5953, so this restart path cannot reuse the shared 80-frame timer.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_SECOND_SEGMENT_FRAMES: usize = 449;
/// Pre-step local elapsed at which the timeout-restart IntroSequence2 loader
/// has finished its hidden `loc_1B9D0 -> sub_1C204` setup and the outer
/// `Mode_Standard -> Execute_Objects loc_E12` pass resumes. Oracle object/RNG
/// probes expose the render worker on local frame 41 and the first draw on the
/// following step (published frame 5546).
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_SECOND_RNG_FIRST_LIVE_ELAPSED: usize = 41;
// ASM loc_1B96E keeps the timeout-restart copy of intro sequence 3 polling
// the Start-gated sub_1C246 return until the natural timeout hand-off reaches
// `Game_Mode=$20`. Oracle-backed published TAS traces keep the second visit
// alive through frame 9354 and only expose intro sequence 4 on frame 9355, so
// the native boot-state model needs a 299-frame local timer here rather than
// the earlier 256-frame approximation.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_THIRD_SEGMENT_FRAMES: usize = 299;
/// Pre-step local elapsed at which timeout-restart IntroSequence3 has finished
/// `sub_1B93E -> sub_1C204` and resumes the outer `Execute_Objects loc_E12`
/// draw. Oracle exposes the first live pass on published frame 5995.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_THIRD_RNG_FIRST_LIVE_ELAPSED: usize = 41;
/// Post-step title elapsed at which the timeout-restart IntroSequence3
/// `loc_1C264 -> loc_6E2` hidden rebuild rejoins Obj_TitleMenu's outer object
/// passes. Oracle freezes published frames 6210..6264 and resumes on 6265.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_3_TITLE_RNG_FIRST_LIVE_ELAPSED: usize = 56;
/// Post-step title elapsed at which the timeout-restart IntroSequence1 early
/// restore has finished `sub_1C246 -> loc_1C264 -> loc_6E2 -> Load_TitleScreen`.
/// Oracle keeps the complete ring fixed on published frames 34221..34274 and
/// resumes `Mode_Standard -> Execute_Objects loc_E12` on frame 34275.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_1_EARLY_TITLE_RNG_FIRST_LIVE_ELAPSED: usize = 55;
/// Post-step title elapsed at which IntroSequence2's `loc_1B9D0 -> loc_1BA02`
/// worker returns through `sub_1C246 -> loc_1C264 -> Load_TitleScreen`. Oracle
/// keeps the complete ring fixed through published frame 36877 (elapsed 56)
/// and resumes `Mode_Standard -> Execute_Objects loc_E12` at elapsed 57.
pub(crate) const INTRO_SEQUENCE_2_TITLE_RNG_FIRST_LIVE_ELAPSED: usize = 57;
/// Post-step title elapsed at which the later IntroSequence4 Start return has
/// finished `loc_1C264 -> loc_6E2 -> Load_TitleScreen`. Oracle keeps the ring
/// fixed on published frames 32008..32062, then the three allocated title
/// objects rejoin `Mode_Standard -> Execute_Objects loc_E12` on frame 32063.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_4_TITLE_RNG_FIRST_LIVE_ELAPSED: usize = 56;
/// Post-step title elapsed at which the timeout-restart IntroSequence5
/// `sub_1C246 -> loc_1C264 -> loc_6E2` rebuild rejoins Obj_TitleMenu's outer
/// object passes. Oracle freezes published frames 10291..10344 and resumes on
/// 10345, which is local title elapsed 55.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_5_TITLE_RNG_FIRST_LIVE_ELAPSED: usize = 55;
/// Post-step title elapsed at which the later Start-skip IntroSequence5
/// `sub_1C246 -> loc_1C264 -> loc_6E2` rebuild rejoins Obj_TitleMenu's outer
/// object passes. Its carried intro-return delay keeps published frame 16705
/// in Load_TitleScreen; `Execute_Objects loc_E12` resumes on frame 16706.
pub(crate) const START_SKIP_TIMEOUT_RESTART_INTRO_SEQUENCE_5_TITLE_RNG_FIRST_LIVE_ELAPSED: usize =
    56;
// ASM loc_1B96A flips `Game_Mode` to intro sequence 4 immediately, but the
// previous intro-sequence-3 roster objects remain visible until the intro-4
// setup fully takes over. Oracle clears the visible Kid pose after 27 traced
// frames on both the cold and timeout-restart lineages.
pub(crate) const INTRO_SEQUENCE_4_POSE_CARRY_FRAMES: usize = 27;
/// ASM loc_1B96A -> loc_6E2 -> Load_IntroSequence4/sub_1B988. The later
/// timeout-restart lineage enters through a different outer object-pass phase:
/// oracle keeps roster 1's Object/GfxObject counts alive on published frames
/// 9355..9385 and clears them on 9386, at local elapsed 31, even though the Kid
/// pose fields clear four frames earlier.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_4_ROSTER_COUNT_CARRY_FRAMES: usize = 31;
// ASM Load_IntroSequence4 only allocates the `sub_1B988` object; the first
// visible `move.w #4,(Camera_Y_pos).w` does not surface until frame 9392 in
// the timeout-restart published TAS window, after a short zero-camera gap.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES: usize = 37;
/// Pre-step local elapsed at which timeout-restart IntroSequence4's
/// hibernating `sub_1B988` worker wakes inside a new outer
/// `Mode_Standard -> Execute_Objects loc_E12` pass. The trace produced after
/// that pass has elapsed 37 and `Camera_Y_pos=4`, so the scheduler burns from
/// pre-step elapsed 36 through the final `loc_1B9B6` pass.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_FOURTH_RNG_FIRST_LIVE_ELAPSED: usize =
    TIMEOUT_RESTART_INTRO_SEQUENCE_4_CAMERA_HANDOFF_FRAMES - 1;
// ASM loc_1C210 (kid.asm:31089): shared sub_1C204 fills all 64 words of
// `Palette_Buffer_3` from `$FFFFFBCC` before the fade-loop timing below.
// Native's timeout-restart model keeps this path as timing-visible state and
// does not materialise the intermediate CRAM staging buffer.
// ASM loc_1C21E (kid.asm:31096): shared sub_1C204 loops on
// `Hibernate_Object_1Frame`, `sub_1C246`, `addi.w #$10,($FFFFF876).w`,
// `cmpi.w #$100,($FFFFF876).w`, so the fade helper idles for 17 frames before
// it finally dispatches `$44(a5)` to loc_1B9B2.
// ASM loc_1C240 (kid.asm:31110): the strict `>$100` exit loads the continuation
// callback from object offset `$44(a5)` and jumps there after the 17th hibernate
// frame, so native counts the shared fade as a complete timing window before
// the caller-specific intro worker resumes.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_4_FADE_FRAMES: usize = 17;
// ASM loc_1C234 (kid.asm:31103): before each non-final shared fade pass,
// `moveq #-1,d0` writes `$FFFFFFFF` to `$FFFFF888` and `$FFFFF88C`, then
// branches back to loc_1C21E. Native folds those shadow-palette writes into
// the same 17-frame timing window because the timeout-restart model does not
// keep the intermediate palette masks.
// ASM sub_1B988 / sub_1C204 / loc_1B9B6 do not start the
// `move.w #$21B,d0` / `dbf` countdown until the delayed `sub_1B988` worker
// reaches the end of the 17-frame fade helper. On the timeout-restart path
// oracle keeps the carried intro-sequence-3 pose alive for 27 frames,
// exposes the `Camera_Y_pos=4` hand-off on frame 9392, idles inside
// `sub_1C204`, then holds `Game_Mode=$20` for the full `0x21C`-frame
// `loc_1B9B6` loop after that dispatch. Native therefore needs the camera
// hand-off delay, the `sub_1C204` fade delay, and the `loc_1B9B6` loop
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
// ASM loc_1BA28 -> sub_1C204 presents the already-loaded alley three outer
// traces after loc_1BCD0's first GfxObject pass on timeout-restart visits.
// Oracle frames 10009/10010 pin the plain elapsed 60/61 boundary; the later
// Start-skip lineage shifts the enclosing object pass, and therefore the
// display boundary, one frame earlier at frames 16230/16231.
pub(crate) const TIMEOUT_RESTART_INTRO_SEQUENCE_5_VDP_VISIBLE_ELAPSED_LOC_1BA28: usize =
    TIMEOUT_RESTART_INTRO_SEQUENCE_5_SLOT_2_VISIBLE_ELAPSED_FRAMES + 3;
pub(crate) const INTRO_SEQUENCE_5_SLOT_2_X_POS: u32 = 0x0000_0000;
pub(crate) const INTRO_SEQUENCE_5_SLOT_2_Y_POS: u32 = 0x00E0_0000;
// ASM loc_1BA28 dispatches to the shared fade helper sub_1C204 before
// loc_1BA5C starts its long intro-sequence-5 timeout. Oracle traces of the
// published TAS later timeout-restart revisit keep `Game_Mode=$24` alive
// through frame 20893 and only reach loc_1BA5C's `move.w #$28,(Game_Mode).w`
// hand-off on 20894.
// ASM loc_1BA28 also allocates a worker at loc_1BC36 (kid.asm:30516) for
// tile decompression via repeated sub_1C034 calls and j_Hibernate_Object
// yields. After the tile uploads complete, loc_1BCCA (kid.asm:30562) spins
// forever on `j_Hibernate_Object_1Frame` while the fade runs.
// ASM loc_1BCD0 (kid.asm:30567) allocates title screen gfx slots and
// spawns particle objects with routine loc_1BE3C (kid.asm:30667) which
// loads sprite frames from the unk_1BE18/unk_1BE24/unk_1BE30 tables.
// ASM loc_1BA5C: `move.w #$1BC,d0` seeds the countdown. ASM loc_1BA60 runs
// `jsr (j_Hibernate_Object_1Frame).w; bsr.w sub_1C246; dbf d0,loc_1BA60`
// for 0x1BC+1 = 0x1BD = 445 iterations before the `move.w #$28,(Game_Mode).w`
// transition to intro sequence 6. The DBF loop includes the zero slot.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_FRAMES: usize = 17 + 0x01BD + 59;
// ASM loc_1BA60: after the 445-iteration hibernate loop finishes, the
// routine clears the intro-sequence flag at `$FFFFFBCE`, then ASM loc_1BA82
// fills Palette_Buffer_3 with 64 words of `$AAA` for the fade-to-black.
// The palette fill uses `moveq #$3F,d0` (63) and `dbf d0,loc_1BA82`,
// writing $AAA to each of the 64 palette entries.
#[allow(dead_code)]
pub(crate) const INTRO_SEQUENCE_5_FADE_PALETTE_FILL_WORDS: usize = 64;
// ASM loc_1BA90 / loc_1BAA0 together implement a fade counter decrement loop.
// After loc_1BA82 sets `$FFFFF876 = $100` (256), loc_1BAA0 seeds d0=$FF and
// writes -1 to both `$FFFFF888` and `$FFFFF88C`. Each loc_1BA90 frame then
// runs `Hibernate_Object_1Frame; sub_1C246; subi.w #$20,($FFFFF876).w`,
// decrementing the fade counter by $20 (32) per frame until it goes negative
// (`bmi.s loc_1BAAC`), at which point `j_loc_6E2` returns the object.
// The fade runs 9 iterations: $100→$E0→$C0→$A0→$80→$60→$40→$20→$00, then
// the next subtraction yields $FFE0 and bmi branches to exit.
#[allow(dead_code)]
pub(crate) const INTRO_SEQUENCE_5_FADE_COUNTER_ITERATIONS: usize = 9;
// The later Start-skip intro-return lineage revisits the timeout-restart copy
// of intro sequence 5 one shared sub_1C204 fade window earlier than the plain
// timeout path. Oracle keeps `Game_Mode=$24` alive through frame 16649 and
// only lets the frame-16648 Start press arm sub_1C246's shared 2-frame title
// return, so native must not expire this lineage before one extra fade window
// elapses.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_START_SKIP_LINEAGE_EXTRA_FRAMES: usize =
    TIMEOUT_RESTART_INTRO_SEQUENCE_4_FADE_FRAMES;
// ASM loc_1BCD0 hibernates for $1A0 frames after allocating its five pose
// GfxObjects. On the wake-up pass it allocates four replacement Objects and
// deletes itself plus its two owned GfxObjects; the following loc_E12 pass
// lets loc_1BE3C/loc_1BE88 allocate four replacement GfxObjects. The normal
// timeout-restart lineage first exposes that wake-up at local elapsed 475;
// the early-visibility lineage reaches it one frame earlier.
pub(crate) const DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_START_SKIP_PARTICLE_COUNT_FRAME: usize =
    TIMEOUT_RESTART_INTRO_SEQUENCE_5_SLOT_2_VISIBLE_ELAPSED_FRAMES + 0x01A1;
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
// ASM sub_1C246 -> loc_1C264 changes mode without unlinking loc_1BCD0's five
// GfxObjects. Oracle WRAM keeps the stale three-Object/five-GfxObject lists
// through title-local frame 31; loc_6E2 clears both lists on frame 32.
pub(crate) const TIMEOUT_RESTART_TITLE_INTRO_SEQUENCE_5_COUNT_CARRY_FRAMES: usize = 32;
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
pub(crate) const MINI_HOPPING_SKULL_MIN_CONTACT_OVERLAP_FP: i64 = 0x0001_0000;

// === Player death/respawn timing ===

// ASM `Death` (kid.asm:14714) drives a fixed tumble arc, but exits as soon as
// the object falls out of the locked camera window rather than after a long
// fixed hold. Keep a generous cap here and let the runtime end the arc early.
pub(crate) const PLAYER_DEATH_WINDUP_FRAMES: usize = 159;
// Oracle death traces switch from the hidden death shell into the respawn
// bootstrap 32 frames after lose_life. The death-respawn Level 1 oracle window
// `death_respawn_level1_window_01700_01855_step_1` fires lose_life on frame
// 1759 and exposes the respawn bootstrap pose on frame 1791 (32 frames later).
// The earlier 40-frame value was an 8-frame over-compensation that masked a
// one-frame-late death-arc launch (#245); once the death lands on the oracle's
// frame (1684, not 1685) the hidden window is 32 frames, not 40.
pub(crate) const PLAYER_DEATH_HIDE_FRAMES: usize = 32;
// Oracle-backed `lose_life` / `Teleport` traces keep the frozen post-Death
// shell pose visible until 13 frames remain (Level 1 oracle clears Kid's WRAM
// pose on frame 1780 = lose_life + 21), then Kid's coordinates clear while the
// transition camera remains latched. `HIDE - SHELL = 13`, the pose-clear point
// the loc_b8de Level 2 probe also pins (remaining 14 visible -> 13 cleared).
pub(crate) const PLAYER_DEATH_HIDDEN_SHELL_VISIBLE_FRAMES: usize = 19;
// Oracle-backed `lose_life -> Teleport -> loc_6E2` traces keep the frozen
// shell camera until 9 frames remain (Level 1 oracle zeroes Camera_X/Y on frame
// 1785 = lose_life + 26), then clears it before the respawn bootstrap takes
// over. `HIDE - CAMERA = 9`, matching the loc_b8de Level 2 probe (remaining 10
// visible -> 9 cleared).
pub(crate) const PLAYER_DEATH_HIDDEN_CAMERA_VISIBLE_FRAMES: usize = 23;
// Oracle-backed death/respawn traces keep Kid on the hidden respawn settle path
// far longer than the old 428-frame placeholder: the canonical
// `death-respawn-level1` window compare still reports the mode-8 bootstrap hold
// pose through frame 26000. Keep the shared respawn shell alive through that
// lower bound on the real `GameplayScenario` route; the later visible
// `Load_InGame` return is still a separate frontier.
pub(crate) const PLAYER_DEATH_RESPAWN_SETTLE_FRAMES: usize = 24_210;
// Settled oracle probe `issue-248-death-start-resume-settled` keeps
// `Game_Mode=$0008` on the first Start-edge frame and returns to live
// `Game_Mode=$000C` on the following frame. Trace frames are sampled after the
// native step, so latch the edge for one local frame before the live hand-off.
pub(crate) const PLAYER_DEATH_RESPAWN_START_EDGE_HANDOFF_FRAMES: u8 = 1;
// ASM `loc_1AE2E -> loc_6E2 -> Load_InGame` exposes `Game_Mode=$000C` before
// the hidden respawn/bootstrap WRAM view has collapsed to the normal level
// slice. Keep the post-Start hand-off in live mode while the existing
// bootstrap trace pose owns Kid/camera/object counts.
pub(crate) const PLAYER_DEATH_RESPAWN_LIVE_HANDOFF_FRAMES: u8 =
    EARLY_START_LOAD_INGAME_VISIBLE_FRAME as u8;
// ASM `loc_1AE2E -> loc_6E2 -> Load_InGame` on the normal title-card path
// writes live `Game_Mode=$000C` before the old title-card object slots and the
// fresh level bootstrap finish cycling through WRAM. The Fast Action route
// oracle keeps that carry visible through local frames 0..93, then settles on
// the normal Level 1 gameplay slice on local frame 94.
pub(crate) const TITLE_CARD_LIVE_HANDOFF_FRAMES_LOC_1AE2E: u8 = 94;
/// ASM loc_1AE2E -> loc_6E2 keeps Load_InGame inside its blocking loader for
/// 58 traced frames after Game_Mode becomes $000C. Mode_Level resumes on 59.
pub(crate) const TITLE_CARD_LOAD_INGAME_HIBERNATE_FRAMES_LOC_1AE2E: u8 = 58;
pub(crate) const TITLE_CARD_LIVE_HANDOFF_TITLE_WORKER_FRAMES_LOC_1AE2E: usize = 27;
pub(crate) const TITLE_CARD_LIVE_HANDOFF_ZEROED_TITLE_WORKER_FRAMES_LOC_1AE2E: usize = 31;
pub(crate) const TITLE_CARD_LIVE_HANDOFF_LOAD_INGAME_VISIBLE_FRAME_LOC_1AE2E: usize = 69;
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
// ASM `lose_life -> Teleport -> loc_6E2` follows the same hidden mode-8
// bootstrap on internal LevelId(1), but the upjump/Drips oracle route starts
// from the wrapped `$FFE0` row and slides to the shared hold pose with smaller
// per-frame deltas.
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_TWO_BOOTSTRAP_INITIAL_X_FP: u32 = 0x0088_0000;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_TWO_BOOTSTRAP_INITIAL_Y_FP: u32 = 0xffe0_0000;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_TWO_BOOTSTRAP_X_STEP_FP: i32 = -0x0001_2000;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_TWO_BOOTSTRAP_Y_STEP_FP: i32 = 0x0002_2000;
// The direct lower-floor Drips jump probe reaches the same ASM
// `lose_life -> Teleport -> loc_B8DE -> loc_6E2` hidden mode-8 hand-off with
// a different seeded loc_1AC70 pose. Keep this as a route-specific trace
// profile until the full LoadGameModeData/loc_1AC70 worker lineage is native.
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_TWO_DIRECT_DRIPS_INITIAL_X_FP: u32 = 0x0130_0000;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_TWO_DIRECT_DRIPS_INITIAL_Y_FP: u32 = 0x0100_0000;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_TWO_DIRECT_DRIPS_X_STEP_FP: i32 = -0x0003_C000;
pub(crate) const PLAYER_DEATH_RESPAWN_LEVEL_TWO_DIRECT_DRIPS_Y_STEP_FP: i32 = -0x0002_6000;
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
// ASM Make_SpriteFromGfxObject loc_11EE compares screen X directly with
// `$0160`: 320 visible pixels plus a 32px right-side allocation margin.
pub(crate) const SPRITE_VISIBILITY_RIGHT_MARGIN_PX: i32 = 0x20;
pub(crate) const SPRITE_VISIBILITY_LEFT_MARGIN_PX: i32 = 0x40;
/// ASM loc_D2C4/loc_D2CE: no-continue GAME OVER waits through a `$3C`
/// text-fall hibernate before the `$1A3` Start-skippable title-return loop.
pub(crate) const PLAYER_GAME_OVER_PRE_HIBERNATE_FRAMES: usize = 0x3C;
pub(crate) const PLAYER_GAME_OVER_START_SKIP_HOLD_FRAMES: usize = 0x1A3;
pub(crate) const PLAYER_GAME_OVER_HOLD_FRAMES: usize =
    PLAYER_GAME_OVER_PRE_HIBERNATE_FRAMES + PLAYER_GAME_OVER_START_SKIP_HOLD_FRAMES;
/// ASM loc_D0C0 plus loc_D15E..loc_D17E: with continues remaining, GAME OVER
/// runs the shared score/fade setup and branches to Continue_Screen at loc_D25E
/// without the no-continues `$3C + $1A3` title-return hold.
pub(crate) const PLAYER_GAME_OVER_CONTINUE_HOLD_FRAMES: usize = 10;
/// ASM: Continue countdown: loc_D31E initial pause, loc_D350 first Start
/// window, then loc_D3AA post-fade Start window before returning to title.
pub(crate) const CONTINUE_INITIAL_HOLD_FRAMES: usize = 28;
pub(crate) const CONTINUE_FIRST_START_WINDOW_FRAMES: usize = 291;
pub(crate) const CONTINUE_SECOND_START_WINDOW_FRAMES: usize = 239;
pub(crate) const CONTINUE_FIRST_PHASE_FRAMES: usize =
    CONTINUE_INITIAL_HOLD_FRAMES + CONTINUE_FIRST_START_WINDOW_FRAMES;
pub(crate) const CONTINUE_COUNTDOWN_FRAMES: usize =
    CONTINUE_FIRST_PHASE_FRAMES + CONTINUE_SECOND_START_WINDOW_FRAMES;
pub(crate) const PLAYER_LEVEL_ADVANCE_RESPAWN_SETTLE_FRAMES: usize = 401;
// ASM loc_BC34 (kid.asm:15181): damage-flash thread spawned by sub_7196 at
// priority $FFFC.  loc_BC40 waits for Just_received_damage, then loc_BC54
// seeds d3=$A and loc_BC56 through loc_BC6E run the flash cycle: sub_80BC
// (white) + 2-frame hold (loc_BC5C, dbf d2 with d2=1) + sub_80D0 (helmet
// palette) + 4-frame hold (loc_BC6E, dbf d2 with d2=3), outer loop dbf d3
// with d3=$A (11 iterations).
// 11 × (2 + 4) = 66 frames, then `sf Just_received_damage` and back to loc_BC40.
// Native models the gameplay effect (invulnerability window) with this constant;
// the palette cycling is visual-only.
pub(crate) const PLAYER_DAMAGE_INVULNERABLE_FRAMES: usize = 66;

// === Collision / activation ===

// ASM Manage_EnemyLoading (kid.asm loc_368A8): spawn window is
// x=[camera-0x80, camera+0x1C0], y=[camera-0x80, camera+0x160].
pub(crate) const ACTIVATION_WINDOW_X_LEAD: u32 = 0x1C0;
pub(crate) const ACTIVATION_WINDOW_Y_LEAD: u32 = 0x160;
pub(crate) const ACTIVATION_WINDOW_PADDING: u32 = 0x80;
pub(crate) const FLYER_RETENTION_X_PADDING: u32 = 512;
pub(crate) const FLYER_RETENTION_Y_PADDING: u32 = 96;

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
/// ASM Dragon.asm `loc_3CEBE -> stru_3CCE6`: a surviving walking Dragon
/// holds `unk_C755E` for `$19` animation ticks before restoring `$62(a5)`.
pub(crate) const DRAGON_WALKING_HURT_ANIMATION_FRAMES_STRU_3CCE6: u8 = 0x19;
// DragonWalking hibernates after a surviving stomp instead of resuming patrol
// on the next frame. The active TAS frontier keeps layout #02 parked from frame
// 2939 through 2962 before the right-facing patrol resumes on 2963.
pub(crate) const DRAGON_WALKING_NONLETHAL_STOMP_HIBERNATE_FRAMES: u8 = 24;

// === Evanescent blocks ===

// Evanescent blocks: solid until the player stands on them, then fade and vanish.
// Timer starts at (skin - 28); incremented each frame; block removed at timer ≥ 22.
pub(crate) const EVANESCENT_SKIN_OFFSET: i16 = 0x1C; // 28
pub(crate) const EVANESCENT_REMOVAL_THRESHOLD: i16 = 0x16; // 22
pub(crate) const EVANESCENT_COLLISION_OVERLAY_TIMER: i16 = 0x10; // 16
#[cfg(test)]
pub(crate) const EVANESCENT_MAX_ENTRIES: usize = 19; // ASM limit $13

// === Level timer ===

// Level timer: Init_Timer_and_Bonus_Flags seeds 3:00, then Init_SpriteAttr_HUD
// adds one low seconds digit and seeds Time_SubSeconds to 1 before gameplay.
// Time-up = death. Clocks add 3 minutes (capped at 10:00).
pub(crate) const LEVEL_TIMER_START_SECONDS: u16 = 181; // 3:01 pre-decrement
pub(crate) const LEVEL_TIMER_COMPLETION_BASE_SECONDS: u16 = 180; // ASM loc_C0CC addi #$B4
pub(crate) const LEVEL_TIMER_MAX_SECONDS: u16 = 600; // 10:00
pub(crate) const LEVEL_TIMER_CLOCK_BONUS_SECONDS: u16 = 180; // +3:00
pub(crate) const LEVEL_TIMER_CLOCK_COLLECT_SECONDS: u16 = 181; // loc_FF92 addq + loc_FFDE
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

pub(crate) const LAVA_GEYSER_ROW_HEIGHT_PX: u16 = 16;
/// `loc_14874` presentations from `$48(a5)=2`: DBF visits 2, 1, then 0.
pub(crate) const LAVA_GEYSER_RISE_FRAMES_PER_ROW: u16 = 3;
pub(crate) const LAVA_GEYSER_HOLD_CYCLES: u16 = 30;
/// `loc_14874` presentations from `$48(a5)=6`: DBF visits 6 through 0.
pub(crate) const LAVA_GEYSER_HOLD_FRAMES_PER_CYCLE: u16 = 7;
/// `loc_14874` presentations from collapse `$48(a5)=1`.
pub(crate) const LAVA_GEYSER_COLLAPSE_FRAMES_PER_ROW: u16 = 2;
pub(crate) const LAVA_GEYSER_PHASE_DORMANT_LOC_14772: u8 = 0;
pub(crate) const LAVA_GEYSER_PHASE_RISING_LOC_1484E: u8 = 1;
pub(crate) const LAVA_GEYSER_PHASE_HOLDING_LOC_1496C: u8 = 2;
pub(crate) const LAVA_GEYSER_PHASE_COLLAPSING_LOC_1498C: u8 = 3;
pub(crate) const LAVA_GEYSER_PHASE_STARTUP_SCRIPT_LOC_14788: u8 = 4;
pub(crate) const LAVA_GEYSER_PHASE_DISABLED_LOC_1483A: u8 = 5;
pub(crate) const LAVA_GEYSER_PHASE_REPEAT_SCRIPT_LOC_1479C: u8 = 6;
pub(crate) const LAVA_GEYSER_STARTUP_SCRIPT_VISIBLE_STEPS_LOC_146B6: u16 = 5;
pub(crate) const LAVA_GEYSER_REPEAT_SCRIPT_FRAMES_PER_TEMPLATE_LOC_146A0: u16 = 3;
pub(crate) const LAVA_GEYSER_REPEAT_SCRIPT_VISIBLE_STEPS_LOC_146A0: u16 =
    5 * LAVA_GEYSER_REPEAT_SCRIPT_FRAMES_PER_TEMPLATE_LOC_146A0;

// === Ending sequence ===

/// Number of frames for the ending sequence phases (ASM End_Credits, kid.asm:16920).
pub(crate) const ENDING_PAUSE_FRAMES: usize = 40; // Sleep 0x28
pub(crate) const ENDING_MUSIC_DELAY_FRAMES: usize = 200; // Sleep 0xC8
pub(crate) const ENDING_FADE_OUT_FRAMES: usize = 256; // fade counter 0x100

// GEMS BGM ids from upstream constants.asm. These are traced from core when an
// ASM-backed path reaches a `j_PlaySound` music command.
pub(crate) const BGM_ENDING_LOC_D824: u8 = 0x05;
pub(crate) const BGM_SCORE_BOARD_LOC_C238: u8 = 0x74;
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
pub(crate) const DIAMOND_POWER_RAIN_MAX_OBJECTS: usize = 6;
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
/// from the player. 8 frames startup, 4 frames active. One swing at a time.
/// ASM sub_868E (kid.asm:9522): `move.w #8,-(sp); jsr Hibernate_Object` = 8 startup.
/// ASM loc_86D8 (kid.asm:9546): `moveq #3,d0` + `dbf d0,loc_86D8` = 4 active iterations.
pub(crate) const RED_STEALTH_STARTUP_FRAMES: u8 = 8;
pub(crate) const RED_STEALTH_ACTIVE_FRAMES: u8 = 4;
pub(crate) const RED_STEALTH_REACH_X: i32 = 28;
pub(crate) const RED_STEALTH_OFFSET_Y: i32 = -10;
// === Maniaxe ===

/// Maniaxe thrown axe: travels 12 frames outward at ±5 px/frame then
/// reverses for 12 frames. Damages enemies on contact, breaks on walls.
pub(crate) const MANIAXE_BASE_SPEED: i32 = 5;
pub(crate) const MANIAXE_TRAVEL_FRAMES: u8 = 12;
/// ASM `stru_8B4E`/`stru_8B60`: Maniaxe's throw animation releases
/// `Maniaxe_throwing_axe` through `sub_78E8 loc_794C` after the pose finishes.
pub(crate) const MANIAXE_THROW_LATCH_FRAMES_LOC_794C: u8 = 17;
/// ASM ManiaxeAxe_Init uses `stru_8B86`; active frames map through
/// `LnkTo_unk_A5924`/`LnkTo_unk_A59AA` to `CollisionSize_Index` entry
/// `unk_318B2`: left=-7, width=14, top=-14, height=14.
pub(crate) const MANIAXE_AXE_ENEMY_HIT_HALF_WIDTH: i32 = 7;
pub(crate) const MANIAXE_AXE_ENEMY_HIT_HALF_HEIGHT: i32 = 7;

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
/// Micromax grab gravity accumulator: ASM loc_A606 adds $800 per frame.
pub(crate) const MICROMAX_STUCK_GRAVITY: i32 = 0x800;
/// Micromax max fall speed before peeling off surface.
pub(crate) const MICROMAX_STUCK_MAX_FALL: i32 = 0x1_0000;
/// Micromax detach Y velocity: -4.0 FP (strong upward kick).
/// Shared by platform-grab detach (loc_A6AE, kid.asm:13013) and kid_flip
/// wall-cling B-press (loc_AC42, kid.asm:13474): `move.l #$FFFC0000,y_vel`.
pub(crate) const MICROMAX_DETACH_Y_VEL: i32 = -0x4_0000;
/// Micromax detach X speed: 2.0 FP (kick away from surface).
/// loc_A6AE + loc_AC42: `move.l #$FFFE0000,d7; tst.b ($FA72); beq skip;
/// neg.l d7` — direction depends on which side the wall is on.
/// ASM loc_AC64 (kid.asm:13483): store the selected `d7` into x_vel before
/// clearing the climb latch, setting the jump latch, and flipping x_direction.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intro_and_continue_timing_constants_pin_inclusive_asm_counts() {
        assert_eq!(
            DEMO_AUTO_TIMEOUT_INTRO_FIFTH_START_SKIP_RNG_FIRST_LIVE_ELAPSED, 56,
            "ASM loc_1BA28 Start-skip lineage exposes the first gfx pass one frame early"
        );
        assert_eq!(
            INTRO_SEQUENCE_5_START_SKIP_PANEL_UPLOAD_STALL_PRE_STEP_ELAPSED, 384,
            "ASM loc_1BC36 chains the $A0,$90,$0B,$0B hibernations after worker start"
        );
        assert_eq!(
            INTRO_SEQUENCE_5_PANEL_UPLOAD_STALL_PRE_STEP_ELAPSED, 385,
            "ordinary timeout-restart lineage reaches the loc_1BC36 upload stall one frame later"
        );
        assert_eq!(
            DEMO_AUTO_TIMEOUT_INTRO_FOURTH_RNG_FIRST_LIVE_ELAPSED, 36,
            "ASM sub_1B988 wakes in the pass before the elapsed-37 camera handoff trace"
        );
        assert_eq!(
            TIMEOUT_RESTART_INTRO_SEQUENCE_5_VDP_VISIBLE_ELAPSED_LOC_1BA28, 61,
            "ASM loc_1BA28 presents the alley three traces after loc_1BCD0's first gfx pass"
        );
        assert_eq!(
            TITLE_IDLE_DBF_FRAMES_LOC_1BAEA, 0x0281,
            "ASM Obj_TitleMenu loc_1BAEA seeds d0=$280; dbf is inclusive"
        );
        assert_eq!(
            INTRO_SEQUENCE_6_LIGHTNING_LOOP_FRAMES_LOC_1B3BC, 0x002D,
            "ASM loc_1B3BC seeds d1=$2C; dbf is inclusive"
        );
        assert_eq!(
            DEMO_AUTO_TIMEOUT_INTRO_FOURTH_SEGMENT_FRAMES, 594,
            "timeout-restart IntroSequence4 combines camera handoff, sub_1C204 fade, and loc_1B9B6 loop"
        );
        assert_eq!(
            DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_FRAMES, 521,
            "ASM loc_1BA5C hibernates through 0x1BC+1 iterations plus lineage padding"
        );
        assert_eq!(
            DEMO_AUTO_TIMEOUT_INTRO_FIFTH_SEGMENT_START_SKIP_PARTICLE_COUNT_FRAME, 475,
            "ASM loc_1BCD0 wakes after the inclusive $1A0 hibernation count"
        );
        assert_eq!(
            CONTINUE_FIRST_PHASE_FRAMES, 319,
            "ASM Continue path combines loc_D31E initial hold and loc_D350 first Start window"
        );
        assert_eq!(
            CONTINUE_COUNTDOWN_FRAMES, 558,
            "ASM Continue path includes the loc_D3AA post-fade Start window"
        );
        assert_eq!(
            CREDITS_SCREEN_TOTAL, 312,
            "credits screens keep the 112-frame scroll plus 200-frame display cadence"
        );
        assert_eq!(
            LEVEL_SKIP_RESUME_TOTAL_FRAMES, 80,
            "ASM level-skip resume keeps the title-to-level shell plus the 20-frame entry arc"
        );
    }

    #[test]
    fn respawn_and_form_motion_constants_pin_direction_signs() {
        assert_eq!(
            PLAYER_DEATH_RESPAWN_LEVEL_ONE_BOOTSTRAP_X_STEP_FP, -0x0004_8000,
            "oracle-backed Level 1 death bootstrap slides left toward the hold pose"
        );
        assert_eq!(
            EARLY_START_LEVEL_ONE_BOOTSTRAP_X_STEP_FP, -0x0003_A000,
            "ASM loc_1AE2E early title-start bootstrap slides left"
        );
        assert_eq!(
            EARLY_START_LEVEL_ONE_BOOTSTRAP_Y_STEP_FP, -0x0002_6000,
            "ASM loc_1AE2E early title-start bootstrap rises toward the hold pose"
        );
        assert_eq!(
            KID_SLOPE_RUN_DOWNHILL_CAP_FP, -0x0003_0000,
            "Kid right-facing downhill run cap is negative in the slope table model"
        );
        assert_eq!(
            MICROMAX_SLOPE_NEUTRAL_TARGET_FP, -0x0000_4000,
            "Micromax neutral slope drift targets the downhill negative velocity"
        );
        assert_eq!(
            MICROMAX_SLOPE_WALK_DOWNHILL_CAP_FP, -0x0000_C000,
            "Micromax walk-downhill cap remains negative"
        );
        assert_eq!(
            MICROMAX_SLOPE_RUN_DOWNHILL_CAP_FP, -0x0001_8000,
            "Micromax run-downhill cap remains negative"
        );
        assert_eq!(
            SKYCUTTER_LANDING_BOUNCE, -0x0003_0000,
            "ASM Skycutter landing bounce launches upward"
        );
    }
}
