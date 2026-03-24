mod blocks;
mod boot;
mod combat;
pub(crate) mod constants;
mod demo;
pub(crate) mod enemies;
mod helmets;
mod level;
mod movement;
pub mod physics;
mod platforms;
mod projectiles;
mod scenario;
pub mod terrain;
mod trace;
pub(crate) mod types;
mod weather;

use constants::*;
use enemies::*;
use kid_data::{
    AssetManifest, EnemyLayout, EnemyLayoutObject, PlatformLayout, PlatformPlacement,
    resource_level_record,
};
use physics::{Direction, HorizontalInput, LevelBounds};
use platforms::*;
pub use scenario::{
    ScriptRunner, run_script, run_script_with_physics_order, run_script_with_trace,
    run_script_with_warp,
};
pub const BOOT_TO_LEVEL_START_FRAMES: [usize; 7] = constants::BOOT_TO_LEVEL_START_FRAMES;
use std::fs;
#[cfg(test)]
pub(crate) use trace::opening_level_one_object_counts;
pub use types::*;

#[derive(Debug, Clone)]
pub struct NativeRuntime {
    assets: AssetManifest,
    frame_index: usize,
    boot_state: BootState,
    prior_input: FrameInput,
    previous_input: FrameInput,
    start_press_count: usize,
    current_level: LevelId,
    level_start_frame: Option<usize>,
    progression: PlayerProgression,
    helmet: HelmetState,
    paused: bool,
    pause_toggle_pending: bool,
    pending_level_skip: Option<LevelId>,
    pending_level_skip_resume: bool,
    player_lifecycle: PlayerLifecycle,
    opening_slice: Option<OpeningSliceState>,
    death_camera_lock: Option<CameraState>,
    level_skip_transition: Option<LevelSkipTransitionState>,
    level_skip_resume_hold: bool,
    /// When true, skip the entry animation on the current level (used after
    /// level-skip warp — the original game has no entry animation).
    skip_entry_animation: bool,
    active_enemy_roster: Vec<NativeObjectSnapshot>,
    active_enemies: Vec<ActiveEnemy>,
    active_platforms: Vec<ActivePlatform>,
    active_platform_roster: Vec<NativeObjectSnapshot>,
    platform_script_table: Option<kid_data::PlatformScriptTable>,
    block_grid: Option<terrain::BlockGrid>,
    block_list: Option<terrain::BlockList>,
    block_grid_source: BlockGridSource,
    /// Player has beaten the final level — show victory/ending.
    game_complete: bool,
    block_state: BlockInteractionState,
    projectiles: Vec<EnemyProjectile>,
    weather: WeatherState,
    murder_wall: Option<MurderWallState>,
    /// Metal block bounce animations in progress.
    block_bounces: Vec<BlockBounceAnim>,
    /// Block destruction fragment effects.
    block_destroy_effects: Vec<BlockDestroyEffect>,
    /// Floating prize items rising from blocks.
    floating_items: Vec<FloatingItem>,
    /// Prize diamonds award their count later via PrizeDiamondCollected_Init,
    /// not on the touch frame.
    pending_prize_diamond_awards: Vec<u8>,
    /// Enemy death particle effects.
    enemy_death_effects: Vec<EnemyDeathEffect>,
    /// Set each frame if a stomp occurred.
    stomped_this_frame: bool,
    /// `kidabove` collisions resolve on the next frame, not immediately on the
    /// overlap frame.
    pending_stomp_bounce: Option<PendingStompBounce>,
    /// Count of blocks hit by headbutt this frame.
    blocks_hit_this_frame: u8,
    /// Sound test play pending (consumed each frame).
    sound_test_play_pending: bool,
    level_timer: LevelTimer,
    /// Camera Y look-ahead accumulator ($FFFFFB58): +1 per frame while down held
    /// and grounded, -8 per frame on release, clamped to [0, 30].
    y_look_offset: i16,
    /// Set to true when player takes damage during current level (for no-hit bonus).
    level_damage_taken: bool,
    /// Set to true when player collects any prize from a block (disqualifies no-prize bonus).
    level_prize_collected: bool,
    /// Number of clock pickups collected during current level (for completion time calc).
    clocks_collected: u16,
    /// Title screen phase timer: counts down to demo playback in both the
    /// pre-menu attract phase and the interactive title menu. The interactive
    /// loop resets this on Up/Down navigation, matching the ASM.
    title_idle_frames: u16,
    /// Title screen local elapsed frames, used for diagnostics and demo timing.
    title_screen_elapsed_frames: usize,
    /// First global boot frame where the title pose becomes visible in trace output.
    title_screen_ready_frame: usize,
    /// Last global boot frame of the initial title attract lock.
    title_menu_arm_frame: usize,
    /// False during the initial title attract wait where only Start matters.
    /// True once the Start press arms the real interactive title menu.
    title_menu_active: bool,
    /// Demo playback reads inputs after game logic (ASM ReadJoypad path), so
    /// the byte fetched this frame becomes the controller state for the next
    /// frame.
    demo_buffered_input: Option<FrameInput>,
    /// Demo Start abort uses a hibernating object coroutine (`loc_119D8`):
    /// the live Start press arms the return, then title only becomes visible
    /// two frames later when that object wakes and flips Game_Mode.
    pending_demo_title_return: u8,
    /// Once the wake-up lands, preserve the current frame's updated gameplay
    /// slice under title mode instead of freezing the previous frame.
    demo_title_return_after_frame: bool,
    /// Demo auto-timeout runs in ReadJoypad after the current gameplay frame,
    /// so the hand-off should not become visible until the next native step.
    demo_auto_timeout_return_pending: bool,
    /// When the ROM timeout counter trips, the current trace frame already
    /// exposes the frozen gameplay snapshot rather than another live physics
    /// step. Skip the remaining per-frame gameplay work once for that frame.
    demo_auto_timeout_freeze_gameplay_frame: bool,
    /// The ROM auto-timeout path clears Game_Mode to 0 instead of taking the
    /// paused title helper used by manual Start aborts.
    demo_auto_timeout_handoff_active: bool,
    /// Timeout restart hands the generic intro state machine a different
    /// sequence-1 duration than cold boot.
    demo_auto_timeout_intro_sequence_1_frames_override: Option<usize>,
    /// Timeout restart also keeps intro sequence 2 alive much longer than the
    /// shared post-sequence-1 attract timer.
    demo_auto_timeout_intro_sequence_2_frames_override: Option<usize>,
    /// Timeout restart keeps intro sequence 3 alive far longer than cold boot.
    /// Its direct title return is still Start-driven; the natural timeout
    /// path continues through the later intro sequences.
    demo_auto_timeout_intro_sequence_3_frames_override: Option<usize>,
    /// ASM sub_1B988 / loc_1B9B2 hibernate intro sequence 4 for `#$21B+1`
    /// frames before exposing intro sequence 5 on the timeout-restart path.
    demo_auto_timeout_intro_sequence_4_frames_override: Option<usize>,
    /// ASM loc_1BA28 / sub_1C204 / loc_1BA5C keep timeout-restart intro
    /// sequence 5 alive far longer than the shared 80-frame cold-boot timer.
    demo_auto_timeout_intro_sequence_5_frames_override: Option<usize>,
    /// A Start-driven timeout-restart intro-sequence-3 title jump can still
    /// pass through the shared deferred Start→title skip countdown before
    /// title mode becomes visible, so the later handoff must remember to keep
    /// the intro pose.
    timeout_restart_title_intro_sequence_3_return_pending: bool,
    /// ASM sub_1C246 can also jump straight from timeout-restart intro
    /// sequence 5 into title mode after the shared 2-frame Start deferral.
    timeout_restart_title_intro_sequence_5_return_pending: bool,
    /// Demo abort returns to title without rebuilding Kid/camera RAM
    /// immediately; oracle traces keep exposing the live gameplay snapshot
    /// under title mode until later title objects take over.
    preserve_demo_title_snapshot: bool,
    /// The timeout-restart intro-sequence-3 return reaches title mode before
    /// the standard title pose appears; oracle traces keep the last intro pose
    /// alive for the first title frames, then hide Kid again.
    timeout_restart_title_intro_sequence_3_pose: bool,
    /// ASM sub_1C246 enters title mode before the title objects rebuild, so
    /// the timeout-restart intro-sequence-5 slot-2 pose survives briefly into
    /// title mode before Kid blanks again.
    timeout_restart_title_intro_sequence_5_pose: bool,
    /// ASM loc_1B96A switches to `Game_Mode=$20` before Load_IntroSequence4 /
    /// sub_1B988 clear the old intro-sequence-3 roster, so timeout-restart
    /// intro sequence 4 briefly keeps the previous pose visible. The same
    /// latch also carries sub_1B988's `Camera_Y_pos=4` hand-off through the
    /// later intro-sequence-5 window until title setup takes over.
    timeout_restart_intro_sequence_4_pose_carry: bool,
    /// After the live snapshot is fully cleared, oracle traces still keep Kid
    /// hidden for a longer title-object rebuild window on the demo-return path.
    hide_demo_return_title_player: bool,
    /// Timeout-restart demo playback ignores queued manual Start aborts until
    /// the restarted attract demo reaches its live gameplay hand-off.
    ignore_demo_start_abort_until_gameplay: bool,
    /// The next demo start comes from a Start-skip intro return rather than
    /// the cold-boot attract path.
    next_demo_uses_start_skip_intro_return_camera_wrap: bool,
    /// Active demo bootstrap should expose the Start-skip intro-return hidden
    /// camera wrap instead of the cold-boot row.
    demo_uses_start_skip_intro_return_camera_wrap: bool,
    /// ASM sub_1B8F4 / loc_1B920 can re-enter the title shell via
    /// sub_1C246's deferred Start skip. Oracle later exposes that demo's
    /// timeout-restart `loc_1BCD0` slot-2 pose one frame earlier than the
    /// plain timeout path, so keep that lineage alive through the demo's
    /// later auto-timeout return.
    demo_timeout_restart_uses_intro_return_slot_2_early_visibility: bool,
    /// Next Load_IntroSequence3 roster index from ROM byte `$FFFFFC83`.
    next_intro_sequence_3_roster_index: usize,
    /// Active/last Load_IntroSequence3 roster used by trace-visible intro pose.
    active_intro_sequence_3_roster_index: usize,
    /// Alternates between demo 1 and demo 2 (ASM $FFFFFBC8 toggle).
    demo_toggle: bool,
    /// Tracks whether demo playback is currently inside Kid_Transform so the
    /// post-transform demo counter parity can be restored on the exit frame.
    demo_transform_active: bool,
    /// Two-player alternating mode enabled.
    two_player: bool,
    /// Current player index (0=P1, 1=P2). Mirrors ASM $FFFFFC39.
    current_player_index: u8,
    /// Per-player persistent state slots.
    player_slots: [PlayerSlot; 2],
    /// Physics sub-step ordering (default: GravityFirst = current native order).
    physics_order: PhysicsOrder,
    /// When true, populate `PhysicsTrace` on each frame for sub-step debugging.
    trace_physics: bool,
    /// Debug label for the gameplay branch that last controlled Kid this frame.
    last_kid_frame_motion_branch: KidFrameMotionBranch,
    /// Debug label for the first post-motion system that moved or consumed Kid.
    last_kid_frame_post_motion_mutator: KidFramePostMotionMutator,
    /// Debug label for the prior frame's first post-motion system that touched Kid.
    previous_kid_frame_post_motion_mutator: KidFramePostMotionMutator,
    /// Debug snapshot of Kid before any pre-physics mutator runs this frame.
    last_kid_frame_pre_physics_start: Option<PlayerState>,
    /// Debug snapshot of Kid immediately before `step_with_order(...)`.
    last_kid_frame_pre_physics_before_physics: Option<PlayerState>,
    /// Ordered list of pre-physics handlers that changed Kid state this frame.
    last_kid_frame_pre_physics_mutators: Vec<KidFramePrePhysicsMutator>,
    /// Debug snapshot of native carry-style state before pre-physics consumption,
    /// including the closest native equivalent to ROM's pre-sub_B168
    /// Character_CheckCollision / Jump_On_Enemy handoff.
    last_kid_frame_collision_handoff_probe: Option<KidCollisionHandoffProbe>,
    /// Debug snapshot of stomp eligibility immediately after physics and before
    /// any post-motion consumer can move Kid again.
    last_kid_frame_same_frame_stomp_probe: Option<KidSameFrameStompProbe>,
    /// Debug snapshot of the Iron Knight wall-attach gate at the pre-physics
    /// consumer boundary.
    last_kid_frame_iron_knight_attach_probe: Option<KidIronKnightAttachProbe>,
    /// Debug snapshot of the shared kid_flip branch-selection fan-out after the
    /// post-sub_AF10 wall scan.
    last_kid_frame_kid_flip_branch_probe: Option<KidFlipBranchProbe>,
    /// Debug snapshot of the shared post-kid_flip follow-on stages inside the
    /// native loc_AA22 mirror.
    last_kid_frame_kid_flip_follow_on_probe: Option<KidFlipFollowOnProbe>,
    /// Debug snapshot of the gate inputs and branch taken by the native
    /// `apply_jump_and_gravity_vel_only(...)` call.
    last_kid_frame_jump_gravity_probe: Option<KidJumpGravityProbe>,
    /// Debug snapshot of the AsmFaithful grounded-release scan before normal physics.
    last_kid_frame_grounded_release_probe: Option<KidGroundedReleaseProbe>,
}

#[derive(Debug, Clone, Copy)]
struct PendingStompBounce {
    anchor_x_pos: u32,
    anchor_y_pos: u32,
}

impl NativeRuntime {
    pub fn new(assets: AssetManifest) -> Self {
        let mut runtime = Self {
            assets,
            frame_index: 0,
            boot_state: BootState::SegaScreen { elapsed: 0 },
            prior_input: FrameInput::default(),
            previous_input: FrameInput::default(),
            start_press_count: 0,
            current_level: LevelId(0),
            level_start_frame: None,
            progression: PlayerProgression::default(),
            helmet: HelmetState::default(),
            paused: false,
            pause_toggle_pending: false,
            pending_level_skip: None,
            pending_level_skip_resume: false,
            player_lifecycle: PlayerLifecycle::Alive,
            opening_slice: None,
            death_camera_lock: None,
            level_skip_transition: None,
            level_skip_resume_hold: false,
            skip_entry_animation: false,
            active_enemy_roster: Vec::new(),
            active_enemies: Vec::new(),
            active_platforms: Vec::new(),
            active_platform_roster: Vec::new(),
            platform_script_table: None,
            block_grid: None,
            block_list: None,
            block_grid_source: BlockGridSource::None,
            game_complete: false,
            block_state: BlockInteractionState::default(),
            projectiles: Vec::new(),
            weather: WeatherState::default(),
            murder_wall: None,
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
            floating_items: Vec::new(),
            pending_prize_diamond_awards: Vec::new(),
            enemy_death_effects: Vec::new(),
            stomped_this_frame: false,
            pending_stomp_bounce: None,
            blocks_hit_this_frame: 0,
            sound_test_play_pending: false,
            level_timer: LevelTimer::new(),
            y_look_offset: 0,
            level_damage_taken: false,
            level_prize_collected: false,
            clocks_collected: 0,
            title_idle_frames: 0,
            title_screen_elapsed_frames: 0,
            title_screen_ready_frame: TITLE_SCREEN_READY_GLOBAL_FRAME,
            title_menu_arm_frame: 0,
            title_menu_active: false,
            demo_buffered_input: None,
            pending_demo_title_return: 0,
            demo_title_return_after_frame: false,
            demo_auto_timeout_return_pending: false,
            demo_auto_timeout_freeze_gameplay_frame: false,
            demo_auto_timeout_handoff_active: false,
            demo_auto_timeout_intro_sequence_1_frames_override: None,
            demo_auto_timeout_intro_sequence_2_frames_override: None,
            demo_auto_timeout_intro_sequence_3_frames_override: None,
            demo_auto_timeout_intro_sequence_4_frames_override: None,
            demo_auto_timeout_intro_sequence_5_frames_override: None,
            timeout_restart_title_intro_sequence_3_return_pending: false,
            timeout_restart_title_intro_sequence_5_return_pending: false,
            preserve_demo_title_snapshot: false,
            timeout_restart_title_intro_sequence_3_pose: false,
            timeout_restart_title_intro_sequence_5_pose: false,
            timeout_restart_intro_sequence_4_pose_carry: false,
            hide_demo_return_title_player: false,
            ignore_demo_start_abort_until_gameplay: false,
            next_demo_uses_start_skip_intro_return_camera_wrap: false,
            demo_uses_start_skip_intro_return_camera_wrap: false,
            demo_timeout_restart_uses_intro_return_slot_2_early_visibility: false,
            next_intro_sequence_3_roster_index: 0,
            active_intro_sequence_3_roster_index: 0,
            demo_toggle: false,
            demo_transform_active: false,
            two_player: false,
            current_player_index: 0,
            player_slots: [PlayerSlot::default(), PlayerSlot::default()],
            physics_order: PhysicsOrder::default(),
            trace_physics: false,
            last_kid_frame_motion_branch: KidFrameMotionBranch::default(),
            last_kid_frame_post_motion_mutator: KidFramePostMotionMutator::default(),
            previous_kid_frame_post_motion_mutator: KidFramePostMotionMutator::default(),
            last_kid_frame_pre_physics_start: None,
            last_kid_frame_pre_physics_before_physics: None,
            last_kid_frame_pre_physics_mutators: Vec::new(),
            last_kid_frame_collision_handoff_probe: None,
            last_kid_frame_same_frame_stomp_probe: None,
            last_kid_frame_iron_knight_attach_probe: None,
            last_kid_frame_kid_flip_branch_probe: None,
            last_kid_frame_kid_flip_follow_on_probe: None,
            last_kid_frame_jump_gravity_probe: None,
            last_kid_frame_grounded_release_probe: None,
        };
        runtime.reload_block_grid_for_level(LevelId(0));
        runtime.load_platform_script_table();
        runtime
    }

    fn load_platform_script_table(&mut self) {
        let scripts_path = self.assets.source_root.join("level/platform_scripts.asm");
        if let Ok(contents) = std::fs::read_to_string(&scripts_path) {
            self.platform_script_table = Some(kid_data::parse_platform_scripts(&contents));
        }
    }

    pub fn with_generated_level_layout(assets: AssetManifest) -> Self {
        Self::new(assets)
    }

    /// Override the physics sub-step ordering (for sweep comparison experiments).
    pub fn set_physics_order(&mut self, order: PhysicsOrder) {
        self.physics_order = order;
    }

    /// Enable sub-frame physics tracing for TAS debugging.
    pub fn set_trace_physics(&mut self, enabled: bool) {
        self.trace_physics = enabled;
    }

    /// Skip the source port's opening entry animation (camera pan + player
    /// float). The original ROM has no entry animation — set this to true
    /// for ROM-faithful play.
    pub fn set_skip_entry_animation(&mut self, skip: bool) {
        self.skip_entry_animation = skip;
    }

    /// Set the block collision grid for the current level.
    /// Call after constructing the runtime, before stepping frames.
    pub fn set_block_grid(&mut self, grid: terrain::BlockGrid) {
        self.block_grid = Some(grid);
        self.block_grid_source = BlockGridSource::Manual;
    }

    pub fn block_grid(&self) -> Option<&terrain::BlockGrid> {
        self.block_grid.as_ref()
    }

    pub fn step_frame(&mut self, input: FrameInput) -> NativeTraceFrame {
        let raw_input = input;
        self.previous_kid_frame_post_motion_mutator = self.last_kid_frame_post_motion_mutator;
        self.last_kid_frame_motion_branch = KidFrameMotionBranch::None;
        self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::None;
        self.last_kid_frame_pre_physics_start = None;
        self.last_kid_frame_pre_physics_before_physics = None;
        self.last_kid_frame_pre_physics_mutators.clear();
        self.last_kid_frame_collision_handoff_probe = None;
        self.last_kid_frame_same_frame_stomp_probe = None;
        self.last_kid_frame_iron_knight_attach_probe = None;
        self.last_kid_frame_kid_flip_branch_probe = None;
        self.last_kid_frame_kid_flip_follow_on_probe = None;
        self.last_kid_frame_jump_gravity_probe = None;
        self.last_kid_frame_grounded_release_probe = None;
        let replay_demo_1_timeout_handoff_frame = matches!(
            self.boot_state,
            BootState::DemoPlay {
                use_demo_2: false,
                ..
            }
        ) && self.demo_auto_timeout_return_pending
            && self.demo_buffered_input.is_some();
        if replay_demo_1_timeout_handoff_frame {
            self.demo_auto_timeout_return_pending = false;
        } else if self.demo_auto_timeout_return_pending {
            self.demo_auto_timeout_return_pending = false;
            self.finish_demo_auto_timeout_return();
        }
        let Some(input) = self.step_demo_input(input, replay_demo_1_timeout_handoff_frame) else {
            return self.trace_frame();
        };
        self.maybe_begin_pending_level_skip();
        self.maybe_begin_pending_level_skip_resume();
        self.maybe_finish_level_skip_resume();
        let freeze_demo_auto_timeout_snapshot = self.demo_auto_timeout_freeze_gameplay_frame;
        let start_pressed = input.start && !self.previous_input.start;
        let apply_pending_pause_toggle = self.pause_toggle_pending;
        self.pause_toggle_pending = false;
        self.handle_start_press(input, start_pressed);
        self.step_boot_state(input, start_pressed);
        if let BootState::DemoPlay { use_demo_2, .. } = self.boot_state
            && self.demo_gameplay_active()
            && self.opening_slice.is_none()
        {
            self.begin_demo_gameplay(NativeRuntime::demo_level_id(use_demo_2));
        }

        let gameplay_slice_active = matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame
        ) || (self.demo_gameplay_active()
            && self.opening_slice.is_some());
        let preserve_demo_title_snapshot_frame =
            matches!(self.boot_state, BootState::TitleScreen { .. })
                && self.preserve_demo_title_snapshot;
        if !freeze_demo_auto_timeout_snapshot && !preserve_demo_title_snapshot_frame {
            self.refresh_active_roster();
            self.step_pending_prize_diamond_awards();
            // Diamond Power uses raw input (Start+A held) since gameplay_input delays A by 1 frame
            if matches!(self.player_lifecycle, PlayerLifecycle::Alive)
                && gameplay_slice_active
                && !self.paused
            {
                self.step_diamond_power(input);
            }
            if gameplay_slice_active {
                let demo_live_input = matches!(self.boot_state, BootState::DemoPlay { .. })
                    && self
                        .demo_startup_frame()
                        .is_some_and(|frame| frame > demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME);
                let gameplay_input = if demo_live_input {
                    input
                } else {
                    self.gameplay_input(input)
                };
                let jump_pressed = if demo_live_input {
                    input.b && !self.previous_input.b
                } else {
                    self.gameplay_jump_pressed()
                };
                let demo_use_demo_2 = self.demo_uses_second_recording();
                let demo_gameplay_start = demo_use_demo_2
                    .map(demo::gameplay_start_frame)
                    .unwrap_or(demo::DEMO_GAMEPLAY_START_FRAME);
                match self.demo_startup_frame() {
                    Some(frame)
                        if demo_use_demo_2 == Some(false)
                            && (demo_gameplay_start..=demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME)
                                .contains(&frame) =>
                    {
                        self.last_kid_frame_motion_branch =
                            KidFrameMotionBranch::DemoBootstrapPoseHold;
                        self.apply_demo_1_bootstrap_pose_hold();
                    }
                    Some(frame) if frame == demo_gameplay_start => {
                        self.last_kid_frame_motion_branch =
                            KidFrameMotionBranch::DemoBootstrapCameraHandoff;
                        self.apply_demo_bootstrap_camera_handoff();
                    }
                    Some(frame) if frame == demo::DEMO_BOOTSTRAP_INIT_FRAME => {
                        self.last_kid_frame_motion_branch =
                            KidFrameMotionBranch::DemoBootstrapInitialSettle;
                        self.apply_demo_bootstrap_initial_settle();
                    }
                    Some(frame)
                        if frame > demo::DEMO_BOOTSTRAP_INIT_FRAME
                            && frame < demo::DEMO_BOOTSTRAP_GROUNDED_FRAME =>
                    {
                        self.last_kid_frame_motion_branch =
                            KidFrameMotionBranch::DemoBootstrapFreefall;
                        self.apply_demo_bootstrap_freefall(frame);
                    }
                    Some(frame)
                        if (demo::DEMO_BOOTSTRAP_GROUNDED_FRAME
                            ..=demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME)
                            .contains(&frame) =>
                    {
                        self.last_kid_frame_motion_branch =
                            KidFrameMotionBranch::DemoBootstrapGroundedHold;
                        self.apply_demo_bootstrap_grounded_hold();
                    }
                    _ => {
                        if self.try_apply_demo_bootstrap_grounded_run(gameplay_input, jump_pressed)
                        {
                            self.last_kid_frame_motion_branch =
                                KidFrameMotionBranch::DemoBootstrapGroundedRun;
                        } else {
                            self.refresh_opening_slice(gameplay_input, jump_pressed);
                        }
                    }
                }
            }
        } else {
            self.last_kid_frame_motion_branch = KidFrameMotionBranch::DemoAutoTimeoutFrozen;
        }
        if apply_pending_pause_toggle {
            self.paused = !self.paused;
        }
        if self.demo_title_return_after_frame {
            self.demo_title_return_after_frame = false;
            self.finish_demo_title_return(raw_input);
        }
        if replay_demo_1_timeout_handoff_frame {
            self.finish_demo_auto_timeout_return();
        }

        let trace = self.trace_frame();
        self.demo_auto_timeout_freeze_gameplay_frame = false;
        self.prior_input = self.previous_input;
        self.previous_input = input;
        if matches!(self.boot_state, BootState::LevelSkipHold)
            && let Some(transition) = self.level_skip_transition.as_mut()
        {
            transition.elapsed_frames += 1;
        }
        self.frame_index += 1;
        trace
    }

    fn begin_kid_pre_physics_probe(&mut self, player: PlayerState) {
        self.last_kid_frame_pre_physics_start = Some(player);
        self.last_kid_frame_pre_physics_before_physics = Some(player);
        self.last_kid_frame_pre_physics_mutators.clear();
    }

    fn begin_kid_collision_handoff_probe(&mut self, slice: &OpeningSliceState) {
        let player = slice.player;
        // ROM loc_AA22 runs Character_CheckCollision immediately before sub_B168,
        // and Jump_On_Enemy can already restore the saved anchor and inject the
        // bounce velocity there. Native does not have the same collision payload,
        // so record the closest pre-physics stomp/collision carry state here.
        let stomp_candidate =
            self.first_stomp_contact_candidate(slice, player.y_velocity, player.y_pos);
        self.last_kid_frame_collision_handoff_probe = Some(KidCollisionHandoffProbe {
            player_lifecycle: self.player_lifecycle,
            previous_post_motion_mutator: self.previous_kid_frame_post_motion_mutator,
            pending_stomp_bounce: self.pending_stomp_bounce.is_some(),
            pending_form: self.progression.pending_form,
            pending_level_skip: self.pending_level_skip,
            form_change_flash_frames: self.progression.form_change_flash_frames,
            on_ground_before: player.on_ground,
            jump_buffer_frames: player.jump_buffer_frames,
            wall_cling_direction: player.wall_cling_direction,
            stomp_candidate: stomp_candidate.is_some(),
            stomp_enemy_object_type: stomp_candidate.and_then(|enemy| match enemy.kind {
                NativeObjectKind::Enemy { object_type } => Some(object_type),
                _ => None,
            }),
            stomp_enemy_x_pos: stomp_candidate.map(|enemy| enemy.x_pos),
            stomp_enemy_y_pos: stomp_candidate.map(|enemy| enemy.y_pos),
            anchor_x_pos: player.x_pos,
            anchor_y_pos: player.y_pos,
        });
    }

    fn note_kid_same_frame_stomp_probe(
        &mut self,
        slice: &OpeningSliceState,
        pre_physics_y_velocity: i32,
        pre_physics_y_pos: u32,
        frame_start_x_pos: u32,
        frame_start_y_pos: u32,
    ) {
        let candidate =
            self.first_stomp_contact_candidate(slice, pre_physics_y_velocity, pre_physics_y_pos);
        self.last_kid_frame_same_frame_stomp_probe = Some(KidSameFrameStompProbe {
            stomp_candidate: candidate.is_some(),
            enemy_object_type: candidate.and_then(|enemy| match enemy.kind {
                NativeObjectKind::Enemy { object_type } => Some(object_type),
                _ => None,
            }),
            enemy_x_pos: candidate.map(|enemy| enemy.x_pos),
            enemy_y_pos: candidate.map(|enemy| enemy.y_pos),
            pre_physics_y_velocity,
            post_physics_y_velocity: slice.player.y_velocity,
            anchor_x_pos: frame_start_x_pos,
            anchor_y_pos: frame_start_y_pos,
        });
    }

    fn note_kid_iron_knight_attach_probe(
        &mut self,
        input: FrameInput,
        player_before: PlayerState,
        climbing_before: bool,
        player_after: PlayerState,
    ) {
        if self.progression.current_form != PlayerForm::IronKnight {
            self.last_kid_frame_iron_knight_attach_probe = None;
            return;
        }

        let c_pressed = input.c && !self.previous_input.c;
        let wall_probe = self.block_grid.as_ref().map(|grid| {
            let x_px = (player_before.x_pos >> 16) as i32;
            let y_px = (player_before.y_pos >> 16) as i32;
            let facing_right = player_before.facing == Direction::Right;

            if climbing_before {
                terrain::probe_iron_knight_climb_resume_wall(
                    grid,
                    x_px,
                    y_px,
                    self.progression.current_form.half_width(),
                    facing_right,
                )
            } else if player_before.on_ground {
                terrain::probe_wall_column(
                    grid,
                    x_px,
                    y_px,
                    self.progression.current_form.half_width(),
                    PLAYER_HEIGHT,
                    facing_right,
                )
            } else {
                terrain::probe_wall(grid, x_px, y_px, 5, 31, facing_right)
            }
        });
        let candidate = c_pressed && !climbing_before && wall_probe.is_some_and(|probe| probe.hit);
        let climbing_after = self.helmet.iron_knight_climbing;
        let attach_triggered = !climbing_before
            && climbing_after
            && player_after.y_velocity == IRON_KNIGHT_ATTACH_Y_VEL;

        self.last_kid_frame_iron_knight_attach_probe = Some(KidIronKnightAttachProbe {
            c_pressed,
            on_ground_before: player_before.on_ground,
            facing: Some(player_before.facing),
            climbing_before,
            climbing_after,
            y_velocity_before: player_before.y_velocity,
            y_velocity_after: player_after.y_velocity,
            wall_probe,
            candidate,
            attach_triggered,
        });
    }

    fn note_kid_flip_branch_probe(
        &mut self,
        input: FrameInput,
        player_before: PlayerState,
        player_after: PlayerState,
        slice: &OpeningSliceState,
    ) {
        let wall_probe = self.block_grid.as_ref().map(|grid| {
            terrain::probe_wall(
                grid,
                (player_before.x_pos >> 16) as i32,
                (player_before.y_pos >> 16) as i32,
                self.progression.current_form.half_width(),
                PLAYER_HEIGHT,
                player_before.facing == Direction::Right,
            )
        });
        let wall_scan_hit = wall_probe.is_some_and(|probe| probe.hit);
        let b_pressed = input.b && !self.previous_input.b;
        let c_pressed = input.c && !self.previous_input.c;
        let branch = if !wall_scan_hit {
            KidFlipBranchSelection::NoWallPath
        } else if b_pressed {
            match self.progression.current_form {
                PlayerForm::TheKid => KidFlipBranchSelection::BPressedTheKid,
                PlayerForm::Micromax => KidFlipBranchSelection::BPressedMicromax,
                _ if player_before.on_ground => KidFlipBranchSelection::BPressedGroundedLaunch,
                _ => KidFlipBranchSelection::BPressedFallthrough,
            }
        } else if c_pressed {
            match self.progression.current_form {
                PlayerForm::Cyclone => KidFlipBranchSelection::CPressedCyclone,
                PlayerForm::Maniaxe => KidFlipBranchSelection::CPressedManiaxe,
                PlayerForm::Skycutter => KidFlipBranchSelection::CPressedSkycutter,
                _ => KidFlipBranchSelection::CPressedFallthrough,
            }
        } else {
            KidFlipBranchSelection::NoButtonBranch
        };

        self.last_kid_frame_kid_flip_branch_probe = Some(KidFlipBranchProbe {
            wall_scan_hit,
            b_pressed,
            c_pressed,
            helmet_id: self.progression.current_form.helmet_id(),
            facing: Some(player_before.facing),
            on_ground_before: player_before.on_ground,
            on_ground_after: player_after.on_ground,
            branch,
            wall_probe,
        });
        self.last_kid_frame_kid_flip_follow_on_probe =
            slice
                .kid_flip_follow_on_stages
                .map(|stages| KidFlipFollowOnProbe {
                    branch,
                    motion_branch: self.last_kid_frame_motion_branch,
                    post_motion_mutator: self.last_kid_frame_post_motion_mutator,
                    stages,
                });
    }

    fn kid_pre_physics_probe_state_changed(before: PlayerState, after: PlayerState) -> bool {
        before.x_pos != after.x_pos
            || before.y_pos != after.y_pos
            || before.x_velocity != after.x_velocity
            || before.y_velocity != after.y_velocity
            || before.on_ground != after.on_ground
    }

    fn note_kid_pre_physics_mutator(
        &mut self,
        mutator: KidFramePrePhysicsMutator,
        before: PlayerState,
        after: PlayerState,
    ) {
        self.last_kid_frame_pre_physics_before_physics = Some(after);
        if Self::kid_pre_physics_probe_state_changed(before, after) {
            self.last_kid_frame_pre_physics_mutators.push(mutator);
        }
    }

    fn begin_kid_grounded_release_probe(
        &mut self,
        player: PlayerState,
        input: FrameInput,
        jump_pressed: bool,
        custom_gravity: bool,
    ) {
        if self.physics_order != PhysicsOrder::AsmFaithful {
            self.last_kid_frame_grounded_release_probe = None;
            return;
        }

        let held_direction = match (input.left, input.right) {
            (true, false) => Some(Direction::Left),
            (false, true) => Some(Direction::Right),
            _ => None,
        };
        let mut probe = KidGroundedReleaseProbe {
            on_ground_before: player.on_ground,
            held_direction,
            jump_pressed,
            jump_buffer_frames: player.jump_buffer_frames,
            custom_gravity,
            wall_cling_direction: player.wall_cling_direction,
            velocity_branch: if player.on_ground && player.y_velocity >= 0 && jump_pressed {
                KidGroundedReleaseVelocityBranch::JumpPressed
            } else if player.on_ground && player.y_velocity >= 0 && player.jump_buffer_frames > 0 {
                KidGroundedReleaseVelocityBranch::JumpBuffered
            } else if !player.on_ground && custom_gravity {
                KidGroundedReleaseVelocityBranch::AirborneCustomGravity
            } else if !player.on_ground {
                KidGroundedReleaseVelocityBranch::AirborneGravity
            } else {
                KidGroundedReleaseVelocityBranch::GroundedHold
            },
            ..KidGroundedReleaseProbe::default()
        };

        if !player.on_ground {
            self.last_kid_frame_grounded_release_probe = Some(probe);
            return;
        }

        let Some(grid) = self.block_grid.as_ref() else {
            self.last_kid_frame_grounded_release_probe = Some(probe);
            return;
        };

        let collision_hw = self.progression.current_form.half_width();
        let x_px = (player.x_pos >> 16) as i16 as i32;
        let y_px = (player.y_pos >> 16) as i16 as i32;
        let probe_row = ((y_px + 1) >> 4).max(0) as u16;
        let left_col = ((x_px - collision_hw) >> 4).max(0) as u16;
        let right_col = ((x_px + collision_hw) >> 4)
            .max(0)
            .min(grid.width_cells as i32 - 1) as u16;

        probe.probe_row = Some(probe_row);
        probe.left_col = Some(left_col);
        probe.right_col = Some(right_col);

        for col in left_col..=right_col {
            let cell = grid.cell_at(col, probe_row);
            if matches!(
                cell.collision,
                terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
            ) {
                probe.support_col = Some(col);
                probe.support_collision = Some(cell.collision);
                probe.support_block_type = Some(cell.block_type);
                break;
            }
        }

        probe.release_triggered = probe.support_col.is_none();
        self.last_kid_frame_grounded_release_probe = Some(probe);
    }

    fn level_elapsed_frames(&self) -> Option<usize> {
        self.level_start_frame
            .map(|frame| self.frame_index.saturating_sub(frame))
    }

    fn gameplay_input(&self, input: FrameInput) -> FrameInput {
        FrameInput {
            up: self.previous_input.up,
            down: self.previous_input.down,
            left: self.previous_input.left,
            right: self.previous_input.right,
            a: self.previous_input.a,
            // Gameplay motion consumes the previous controller state, including jump hold.
            b: self.previous_input.b,
            c: input.c,
            start: input.start,
        }
    }

    fn gameplay_jump_pressed(&self) -> bool {
        self.previous_input.b && !self.prior_input.b
    }

    fn update_demo_transform_parity(&mut self) {
        let transform_active = self.progression.form_change_flash_frames > 0;
        if let BootState::DemoPlay {
            frame_counter,
            data_index,
            use_demo_2,
            ..
        } = &mut self.boot_state
        {
            if self.demo_transform_active && !transform_active {
                // During Kid_Transform, step_demo_input freezes frame_counter.
                // When the coroutine exits, native would otherwise resume one
                // frame too far ahead, flipping the 2-frame byte parity for the
                // rest of the demo playback. The first transform frame still
                // consumes one ReadJoypad byte before the coroutine freeze, so
                // restore that skipped byte advance when the transform exits.
                *frame_counter = frame_counter.saturating_sub(1);
                *data_index += 1;
                let byte = demo::demo_byte(*use_demo_2, *data_index);
                self.demo_buffered_input = Some(demo::decode_demo_byte(byte));
            }
            self.demo_transform_active = transform_active;
        } else {
            self.demo_transform_active = false;
        }
    }

    pub(crate) fn gameplay_hibernating_for_transform(&self) -> bool {
        self.progression.form_change_flash_frames > 0
    }

    fn apply_form_width_overlap_fixup(&mut self, slice: &mut OpeningSliceState) {
        if self.progression.form_width_delta == 0 {
            return;
        }
        let Some(grid) = self.block_grid.as_ref() else {
            self.progression.form_width_delta = 0;
            return;
        };

        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let new_half = self.progression.current_form.half_width();

        // Probe left edge (x - half_width)
        let left_edge = x_px - new_half;
        let left_probe = terrain::probe_wall(grid, x_px, y_px, new_half, PLAYER_HEIGHT, false);
        if left_probe.hit {
            // Push right: align to tile boundary + half_width
            let aligned = (left_edge & !0xF) + 16 + new_half;
            slice.player.x_pos = (aligned as u32) << 16;
        } else {
            // Probe right edge (x + half_width)
            let right_edge = x_px + new_half;
            let right_probe = terrain::probe_wall(grid, x_px, y_px, new_half, PLAYER_HEIGHT, true);
            if right_probe.hit {
                // Push left: ASM formula (right_edge & 0xFFF0) - 1 - half_width
                let aligned = (right_edge & !0xF) - 1 - new_half;
                slice.player.x_pos = (aligned as u32) << 16;
            }
        }

        self.progression.form_width_delta = 0;
    }

    fn refresh_opening_slice(&mut self, input: FrameInput, jump_pressed: bool) {
        if !self.step_lifecycle(input) {
            self.demo_transform_active = false;
            return;
        }
        self.update_demo_transform_parity();

        let level_elapsed = self.level_elapsed_frames().unwrap_or(0);
        // When skip_entry_animation is set (after level-skip warp), pretend
        // the level has been running long enough to bypass the entry animation.
        // The original game has no entry animation — the player appears
        // instantly at the spawn point.
        let effective_elapsed = if self.skip_entry_animation || self.demo_gameplay_active() {
            level_elapsed + OPENING_SCROLL_START_FRAME
        } else {
            level_elapsed
        };
        let bounds = self.level_bounds(self.current_level);
        let mut slice = self.opening_slice.unwrap_or_else(|| {
            if matches!(self.boot_state, BootState::DemoPlay { .. })
                && self.current_level != LevelId(0)
            {
                self.level_respawn_slice(self.current_level)
            } else {
                OpeningSliceState::new(bounds)
            }
        });
        // ASM sub_7B30 (kid.asm:8230): before Kid_Transform, checks if the
        // new helmet's wider hitbox overlaps walls and immediately pushes the
        // player clear. This can come from a transform resolved last frame or
        // from a floating helmet prize collected later in this frame.
        self.stomped_this_frame = false;
        self.begin_kid_collision_handoff_probe(&slice);
        self.begin_kid_pre_physics_probe(slice.player);
        let player_before_form_width_overlap_fixup = slice.player;
        self.apply_form_width_overlap_fixup(&mut slice);
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::FormWidthOverlapFixup,
            player_before_form_width_overlap_fixup,
            slice.player,
        );
        let player_before_pending_stomp_bounce = slice.player;
        self.apply_pending_stomp_bounce(&mut slice);
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::PendingStompBounce,
            player_before_pending_stomp_bounce,
            slice.player,
        );
        slice.prize_block_hit = None;
        slice.lift_block_hit = None;
        slice.drill_projectile_hit = None;
        slice.metal_block_hit = None;
        slice.smash_block_hit = None;
        slice.teleporter_contact = None;
        slice.ceiling_prize_clear = None;
        self.blocks_hit_this_frame = 0;
        self.sound_test_play_pending = false;
        // ASM Kid_Transform replaces the normal frame loop with a coroutine
        // that hibernates gameplay objects. Keep the previous slice/roster
        // stable until the transform flash countdown finishes.
        if self.gameplay_hibernating_for_transform() {
            self.last_kid_frame_motion_branch = KidFrameMotionBranch::TransformHibernating;
            self.opening_slice = Some(slice);
            return;
        }
        // Tick visual effects
        self.step_visual_effects();
        // Prize blocks headbutted from below update their queued sub_1007A
        // lifecycle before movement, so the clear frame is collision-visible.
        self.step_deferred_ceiling_clears();
        // Step helmet abilities BEFORE physics
        let player_before_skycutter_flipboard = slice.player;
        self.step_skycutter_flipboard(&mut slice, input);
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::SkycutterFlipboard,
            player_before_skycutter_flipboard,
            slice.player,
        );
        let player_before_cyclone_flight = slice.player;
        self.step_cyclone_flight(&mut slice, input);
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::CycloneFlight,
            player_before_cyclone_flight,
            slice.player,
        );
        let player_before_iron_knight_climb = slice.player;
        let iron_knight_climbing_before = self.helmet.iron_knight_climbing;
        let iron_knight_climb_step = self.step_iron_knight_climb(&mut slice, input);
        self.note_kid_iron_knight_attach_probe(
            input,
            player_before_iron_knight_climb,
            iron_knight_climbing_before,
            slice.player,
        );
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::IronKnightClimb,
            player_before_iron_knight_climb,
            slice.player,
        );
        self.step_red_stealth_sword(input);
        self.step_eyeclops_beam(&slice, input);
        self.step_maniaxe_axe(&slice, input);
        self.step_berzerker_charge(&slice);
        self.step_juggernaut_skull(&slice, input);
        let player_before_micromax_stick = slice.player;
        self.step_micromax_stick(&mut slice, input);
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::MicromaxStick,
            player_before_micromax_stick,
            slice.player,
        );
        let frame_start_x_pos = slice.player.x_pos;
        let frame_start_y_pos = slice.player.y_pos;

        // Capture pre-physics velocity for stomp detection: after physics,
        // floor collision zeroes y_velocity so the stomp check would always fail.
        let pre_physics_y_velocity = slice.player.y_velocity;
        let pre_physics_y_pos = slice.player.y_pos;

        // Initialise physics trace for this frame if tracing is enabled.
        slice.physics_trace = if self.trace_physics {
            Some(PhysicsTrace::default())
        } else {
            None
        };

        // ASM Kid_Transform runs as a coroutine: during the transform animation,
        // the player object is in the transform state and normal movement/physics
        // don't run. Suppress physics while form_change_flash_frames > 0.
        if self.progression.form_change_flash_frames == 0 {
            let deferred_prize_ceiling_cells: Vec<(u16, u16, DeferredPrizeBlockPhase)> = self
                .block_state
                .deferred_ceiling_clears
                .iter()
                .filter(|entry| entry.phase != DeferredPrizeBlockPhase::Deleted)
                .map(|entry| (entry.col, entry.row, entry.phase))
                .collect();
            // Helmet abilities that override normal gravity
            let custom_gravity = self.helmet.skycutter_on_board
                || self.helmet.cyclone_flying
                || self.helmet.iron_knight_climbing
                || self.helmet.micromax_stuck;
            if iron_knight_climb_step == IronKnightClimbStep::AttachHibernate {
                // ORIGINAL_QUIRK: `loc_A276` seeds wall-climb state, then
                // `Hibernate_Object_1Frame` stops Kid immediately. The attach
                // frame preserves position while the new upward velocity becomes
                // visible in object state.
                self.last_kid_frame_motion_branch = KidFrameMotionBranch::IronKnightAttachHibernate;
                if self.trace_physics {
                    slice.physics_trace = Some(PhysicsTrace {
                        pre_horizontal: (slice.player.x_pos, slice.player.x_velocity),
                        post_horizontal: (slice.player.x_pos, slice.player.x_velocity),
                        horizontal_step_applied: false,
                        pre_vertical: (slice.player.y_pos, slice.player.y_velocity),
                        post_vertical: (slice.player.y_pos, slice.player.y_velocity),
                        pre_gravity_y_vel: slice.player.y_velocity,
                        post_gravity_y_vel: slice.player.y_velocity,
                        gravity_applied: false,
                        on_ground_before: slice.player.on_ground,
                        on_ground_after: slice.player.on_ground,
                        final_pos: (slice.player.x_pos, slice.player.y_pos),
                        final_vel: (slice.player.x_velocity, slice.player.y_velocity),
                        ..PhysicsTrace::default()
                    });
                }
            } else if iron_knight_climb_step == IronKnightClimbStep::ExitHibernate {
                // ORIGINAL_QUIRK: `loc_A2E2`/`loc_A2F6` flip facing and jump
                // into `loc_A6F8`, which hibernates the release frame before
                // normal jump physics consumes the preserved climb velocity on
                // the next frame. `loc_A6F8` still runs `sub_71E4`, so camera
                // follow advances once on the hibernated release frame.
                self.last_kid_frame_motion_branch =
                    KidFrameMotionBranch::IronKnightClimbExitHibernate;
                if self.trace_physics {
                    slice.physics_trace = Some(PhysicsTrace {
                        pre_horizontal: (slice.player.x_pos, slice.player.x_velocity),
                        post_horizontal: (slice.player.x_pos, slice.player.x_velocity),
                        horizontal_step_applied: false,
                        pre_vertical: (slice.player.y_pos, slice.player.y_velocity),
                        post_vertical: (slice.player.y_pos, slice.player.y_velocity),
                        pre_gravity_y_vel: slice.player.y_velocity,
                        post_gravity_y_vel: slice.player.y_velocity,
                        gravity_applied: false,
                        on_ground_before: slice.player.on_ground,
                        on_ground_after: slice.player.on_ground,
                        final_pos: (slice.player.x_pos, slice.player.y_pos),
                        final_vel: (slice.player.x_velocity, slice.player.y_velocity),
                        ..PhysicsTrace::default()
                    });
                }
                slice.update_camera_follow(self.y_look_offset);
            } else if iron_knight_climb_step == IronKnightClimbStep::ResumeClimb {
                self.last_kid_frame_motion_branch = KidFrameMotionBranch::IronKnightClimbResume;
                let allow_asm_faithful_solid_touch_snap = !self.demo_gameplay_active();
                self.begin_kid_grounded_release_probe(
                    slice.player,
                    input,
                    jump_pressed,
                    custom_gravity,
                );
                slice.step_iron_knight_climb_resume(
                    input,
                    jump_pressed,
                    self.progression.current_form,
                    self.block_grid.as_mut(),
                    self.block_list.as_ref(),
                    self.y_look_offset,
                    allow_asm_faithful_solid_touch_snap,
                    &deferred_prize_ceiling_cells,
                );
                self.last_kid_frame_jump_gravity_probe = slice.kid_jump_gravity_probe;
            } else {
                self.last_kid_frame_motion_branch = KidFrameMotionBranch::NormalPhysics;
                // Demo playback keeps a hidden shadow arc with one last airborne
                // sub-half-pixel gap before landing. Letting AsmFaithful collapse
                // that gap lands frame 2625 of demo 1 one frame early.
                let allow_asm_faithful_solid_touch_snap = !self.demo_gameplay_active();
                self.begin_kid_grounded_release_probe(
                    slice.player,
                    input,
                    jump_pressed,
                    custom_gravity,
                );
                slice.step_with_order(
                    effective_elapsed,
                    input,
                    jump_pressed,
                    self.progression.current_form,
                    self.block_grid.as_mut(),
                    self.block_list.as_ref(),
                    custom_gravity,
                    self.y_look_offset,
                    self.physics_order,
                    allow_asm_faithful_solid_touch_snap,
                    &deferred_prize_ceiling_cells,
                );
                self.last_kid_frame_jump_gravity_probe = slice.kid_jump_gravity_probe;
            }
            self.note_kid_same_frame_stomp_probe(
                &slice,
                pre_physics_y_velocity,
                pre_physics_y_pos,
                frame_start_x_pos,
                frame_start_y_pos,
            );
            // ASM kid_flip (loc_ACB0): Cyclone flight activation from C press
            // runs AFTER physics so flight takes effect next frame.
            self.step_cyclone_activation(&slice, input);
            self.note_kid_flip_branch_probe(
                input,
                player_before_iron_knight_climb,
                slice.player,
                &slice,
            );
        }
        self.finish_deferred_ceiling_clears();
        // Wide spawn-support settling is a bootstrap/respawn helper, not
        // general demo gameplay logic. Keeping it live after the hidden demo
        // settle finishes yanks Kid down onto nearby support during later arcs,
        // which in turn makes the frame-1406 diamond collectible too early.
        let demo_bootstrap_support_active = self
            .demo_startup_frame()
            .is_some_and(|frame| frame <= demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME);
        let spawn_support_snapped = (self.skip_entry_animation || demo_bootstrap_support_active)
            && self
                .block_grid
                .as_ref()
                .is_some_and(|grid| self.try_settle_on_spawn_support(&mut slice, grid));
        let demo_bootstrap_floor_snapped = self.try_snap_demo_bootstrap_floor(&mut slice);
        if spawn_support_snapped || demo_bootstrap_floor_snapped {
            slice.update_camera_follow(self.y_look_offset);
        }

        // Y look-ahead accumulator ($FFFFFB58): +1/frame while grounded+down,
        // -8/frame on release, clamped to [0, 30].
        if input.down && slice.player.on_ground {
            self.y_look_offset = (self.y_look_offset + 1).min(30);
        } else if self.y_look_offset > 0 {
            self.y_look_offset = (self.y_look_offset - 8).max(0);
        }

        // Berzerker charge: break Smash blocks on wall contact
        if self.helmet.berzerker_charging {
            self.berzerker_break_wall_block(&slice);
        }

        // Step platform movement scripts
        if let Some(ref script_table) = self.platform_script_table {
            for platform in &mut self.active_platforms {
                platform.step_script(script_table);
            }
        }

        // Platform collision: player can stand on top of platforms
        let player_before_platform_collision = slice.player;
        apply_platform_collision(&mut slice, &self.active_platforms);
        if self.last_kid_frame_post_motion_mutator == KidFramePostMotionMutator::None
            && slice.player != player_before_platform_collision
        {
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::PlatformCollision;
        }

        // Evanescent blocks: trigger when player stands on one, tick timers,
        // and clear expired blocks from the grid.
        self.step_evanescent_blocks(&slice);

        // Ghost blocks: cycle between solid and vanished states
        self.step_ghost_blocks();

        // Shifting blocks: move block up one cell when player stands on it
        self.apply_shifting_blocks(&slice);

        // Lift platforms: step movement and collision
        let player_before_lift_platforms = slice.player;
        self.step_lift_platforms(&mut slice);
        if self.last_kid_frame_post_motion_mutator == KidFramePostMotionMutator::None
            && slice.player != player_before_lift_platforms
        {
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::LiftPlatform;
        }

        // Shooter blocks: fire projectiles periodically
        self.step_shooter_blocks(&slice);

        // Weather: hail spawns falling projectiles, lava geysers erupt
        self.step_weather(&slice);
        self.step_lava_geysers(&slice);

        // Queue the full prize-block lifecycle from a headbutt. The tile stays
        // Solid (0x6F) until the coroutine completes, then the same state
        // record clears the tile and spawns the prize object.
        if let Some((col, row, restore_cell)) = slice.ceiling_prize_clear
            && let Some((content, _, _)) = slice.prize_block_hit
        {
            self.block_state
                .deferred_ceiling_clears
                .push(DeferredCeilingClear {
                    col,
                    row,
                    content,
                    restore_cell,
                    deleted_frames_remaining: Self::PRIZE_BLOCK_DELETED_FRAMES,
                    phase: DeferredPrizeBlockPhase::Deleted,
                });
        }

        self.process_block_effects(&mut slice);
        self.collect_floating_items(&mut slice);
        if self.progression.pending_form.is_some() {
            // Oracle demo 0x1B enters Kid_Transform on the same frame that the
            // floating Cyclone prize is touched, so the traced helmet id and the
            // 2px wall-separation push are already visible at frame end.
            self.resolve_pending_form_change();
            self.apply_form_width_overlap_fixup(&mut slice);
        }
        if self.resolve_frame_hazards(
            &mut slice,
            input,
            pre_physics_y_velocity,
            pre_physics_y_pos,
            frame_start_x_pos,
            frame_start_y_pos,
        ) {
            return;
        }
        self.opening_slice = Some(slice);
    }
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod tests;
