mod blocks;
mod boot;
mod combat;
pub(crate) mod constants;
mod demo;
pub(crate) mod enemies;
mod gfx_animation;
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
pub mod vdp;
mod weather;

use combat::{PendingSoundEffect, QueuedSoundEffect};
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
use std::collections::HashSet;
use std::fs;
#[cfg(test)]
pub(crate) use trace::opening_level_one_object_counts;
pub use types::*;

#[derive(Debug, Clone)]
pub struct NativeRuntime {
    assets: AssetManifest,
    frame_index: usize,
    boot_state: BootState,
    trace_input: FrameInput,
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
    starting_level_title_workers: Vec<LevelEntryTitleWorker>,
    starting_level_title_rng: Option<RomRandomState>,
    death_camera_lock: Option<CameraState>,
    level_skip_transition: Option<LevelSkipTransitionState>,
    death_shell_snapshot: Option<LevelSkipTransitionState>,
    level_skip_resume_hold: bool,
    active_enemy_roster: Vec<NativeObjectSnapshot>,
    active_enemies: Vec<ActiveEnemy>,
    /// Enemies killed during this level — (segment_index, layout_index).
    /// Prevents respawn when their spawn point re-enters the activation window.
    /// ASM: per-segment killed bitfield at $FFFFF66A.
    killed_enemies: HashSet<(usize, usize)>,
    /// Blocks drilled by Driller enemies this frame — (col, row).
    /// Driller.asm loc_3BE78: checks sprite timer, destroys Smash/Ice blocks ahead.
    drilled_blocks: Vec<(u16, u16)>,
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
    /// ASM Mini_Hopping_Skull.asm sub_3F3F0 spawns a transient worker that
    /// hibernates 15 frames, then optionally plays the scream SFX before
    /// deleting itself.
    pending_sound_effects: Vec<PendingSoundEffect>,
    /// Internal log used by focused sound-worker tests.
    played_sound_effects: Vec<QueuedSoundEffect>,
    /// Enemy death particle effects.
    enemy_death_effects: Vec<EnemyDeathEffect>,
    /// Set each frame if a stomp occurred.
    stomped_this_frame: bool,
    /// `kidabove` collisions resolve on the next frame, not immediately on the
    /// overlap frame.
    pending_stomp_bounce: Option<PendingStompBounce>,
    /// ASM Init_RNG / Get_RandomNumber_byte global gameplay RNG state.
    rom_random: RomRandomState,
    /// Count of blocks hit by headbutt this frame.
    blocks_hit_this_frame: u8,
    /// Sound test play pending (consumed each frame).
    sound_test_play_pending: bool,
    /// Cached Sega screen art and palette data (loaded once, reused each frame).
    sega_screen_data: Option<vdp::SegaScreenData>,
    /// Cached intro sequence art/tilemap/palette for sub_1B850 segments.
    intro_sub_1b850_data: Option<vdp::IntroSub1B850Data>,
    /// Cached title screen art/tilemap/palette for sub_1B2A4.
    title_screen_data: Option<vdp::TitleScreenData>,
    /// Software VDP state for non-gameplay screens (Sega logo, intro, title).
    /// Populated each frame during boot modes so the VDP renderer can produce
    /// accurate frames; `None` during gameplay where the pre-extracted asset
    /// pipeline handles rendering.
    vdp_state: Option<vdp::VdpState>,
    /// ASM sub_914 / sub_924 nest counter at `$FFFFF805`: active Z80 stop
    /// requests. Zero means the Z80 bus may run; non-zero keeps it halted.
    #[allow(dead_code)]
    z80_stop_request_depth: u8,
    /// ASM sub_914 / sub_924 writes SR directly before stopping/starting the
    /// Z80 so later gameplay code can observe the current interrupt mask.
    #[allow(dead_code)]
    z80_guard_sr: u16,
    /// Native mirror of the Z80 run state toggled by ASM `j_Stop_z80` /
    /// `j_Start_z80`.
    #[allow(dead_code)]
    z80_running: bool,
    /// ASM OptionScreen_Input `Option_2PInput`: false = one controller, true = two.
    ///
    /// ASM ReadJoypad (kid.asm:1332): routes Ctrl_Held/Ctrl_Pressed to
    /// per-player variables depending on this flag and Current_player:
    ///   - loc_B8C (kid.asm:1342): default path (1P mode or Current_player==0)
    ///     stores `Ctrl_Held → Ctrl_1_Held`, `Ctrl_Pressed → Ctrl_1_Pressed`.
    ///   - 2P + Current_player!=0: stores to Ctrl_2_Held/Ctrl_2_Pressed.
    ///
    /// Native: single FrameInput consumed directly — no per-player routing.
    options_two_player_two_controllers: bool,
    /// ASM OptionScreen_Input `Options_Suboption_Controls`: 0..=5.
    options_controls_scheme: u8,
    /// ASM OptionScreen_Input `Option_SpeedButton`: false = normal action, true = fast.
    options_speed_button_fast: bool,
    /// ASM sub_1CC88 (kid.asm:31808): option-menu worker byte toggled by
    /// `not.b ($FFFFFB5C).w` before applying the symmetric `$FFFFFB5E` offset.
    options_menu_panel_toggle_byte: u8,
    /// ASM loc_1CB2C / sub_1CC88: left option-menu worker word at `$24(a0)`.
    options_menu_left_panel_x: i16,
    /// ASM loc_1CB2C / sub_1CC88: right option-menu worker word at `$24(a1)`.
    options_menu_right_panel_x: i16,
    /// ASM loc_1CB2C / sub_1CC88: shared signed offset seeded from
    /// `($FFFFFB5E).w`.
    options_menu_panel_delta: i16,
    level_timer: LevelTimer,
    /// Camera Y look-ahead accumulator ($FFFFFB58): +1 per frame while down held
    /// and grounded, -8 per frame on release, clamped to [0, 30].
    y_look_offset: i16,
    /// ASM $FFFFFA2E: Kid's y_pos from the previous camera-follow invocation.
    /// Used by loc_726C to detect upward motion near the level ceiling.
    camera_last_kid_y: i16,
    /// Set to true when player takes damage during current level (for no-hit bonus).
    level_damage_taken: bool,
    /// Exact per-frame damage latch, mirroring ROM `Just_received_damage`.
    just_received_damage: bool,
    /// Set to true when player collects any prize from a block (disqualifies no-prize bonus).
    level_prize_collected: bool,
    /// Number of clock pickups collected during current level (for completion time calc).
    clocks_collected: u16,
    /// ASM Prize10000Points_Init loc_10AC2 / main-loop loc_7452 share the
    /// `$FFFFFB4C` countdown that starts the Hundred K Trip event in level 4.
    hundred_k_trip_timer: u16,
    /// Title screen phase timer: counts down to demo playback in both the
    /// pre-menu attract phase and the interactive title menu. The interactive
    /// loop resets this on Up/Down navigation, matching the ASM.
    title_idle_frames: u16,
    /// Title screen local elapsed frames, used for diagnostics and demo timing.
    title_screen_elapsed_frames: usize,
    /// Cold-boot intro-return title rebuild uses Obj_TitleMenu's hidden shell
    /// staging before the later title pose becomes ready.
    title_screen_uses_intro_return_hidden_shell: bool,
    /// First global boot frame where the title pose becomes visible in trace output.
    title_screen_ready_frame: usize,
    /// Last global boot frame of the initial title attract lock.
    title_menu_arm_frame: usize,
    /// False during the initial title attract wait where only Start matters.
    /// True once the Start press arms the real interactive title menu.
    title_menu_active: bool,
    /// True on the exact frame the title menu gets armed; the cursor sprite
    /// only becomes visible on the following traced frame.
    title_menu_cursor_pending: bool,
    /// Pending title-menu selection (0=1P, 1=2P, 2=options) applied on the
    /// following frame to match Obj_TitleMenu's deferred hand-off.
    title_menu_selection_pending: Option<u8>,
    /// ASM Obj_TitleMenu `Title_InputLoop` hibernates once before the Start
    /// edge flips into `Game_Mode=$0008`, so visible level entry lands one
    /// traced frame after the pending selection wakes.
    title_menu_selection_delay_frames: usize,
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
    /// ASM sub_1B988 / loc_1B9B6 hibernate intro sequence 4 for `#$21B+1`
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
    /// ASM sub_1C246 can also jump straight from the carried intro-sequence-4
    /// camera hand-off into title after the shared 2-frame deferral.
    timeout_restart_title_intro_sequence_4_return_pending: bool,
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
    /// The later intro-sequence-4 Start return reaches title mode while the
    /// carried `sub_1B988` camera hand-off is still latched.
    timeout_restart_title_intro_sequence_4_camera_carry: bool,
    /// The later Start-skip intro-return lineage reaches the same
    /// `sub_1C246 -> Game_Mode=$2C` hand-off, but oracle keeps the rebuilt
    /// title pose hidden for one extra frame before the standard title pose
    /// returns.
    timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay: bool,
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
    /// The next demo start comes from the generic `loc_119D8 -> loc_6E2`
    /// demo-title return rather than the cold-boot attract path.
    next_demo_uses_demo_title_return_camera_wrap: bool,
    /// Active demo bootstrap should expose the demo-title-return hidden camera
    /// wrap instead of the cold-boot row.
    demo_uses_demo_title_return_camera_wrap: bool,
    /// Load_IntroSequence6 can also flow into Obj_TitleMenu `loc_1BB02 ->
    /// Load_DemoPlay -> Load_InGame`; oracle keeps that timeout-restart
    /// lineage's demo-1 hidden wrap distinct from cold boot and the earlier
    /// Start-skip intro return.
    demo_uses_intro_sequence_6_hidden_camera_wrap: bool,
    /// A later attract demo can also start from a title shell that already had
    /// loc_1B5CE's standard pose restored, either on the timeout-restart
    /// IntroSequence6 lineage or the later intro-sequence-4 title return.
    /// When that demo returns to title, oracle re-exposes the pose two frames
    /// earlier than the generic demo-title rebuild.
    demo_uses_intro_sequence_6_early_title_restore: bool,
    /// A later demo auto-timeout can re-enter intro sequence 1 after the
    /// timeout-restart shell already restored loc_1B5CE's standard title pose.
    /// If Start skips that intro-1 revisit back to title, oracle exposes the
    /// rebuilt title pose two local frames earlier than the generic
    /// intro-return settle window.
    timeout_restart_intro_sequence_1_uses_early_title_restore: bool,
    /// ASM Obj_TitleMenu `loc_1BB02 -> Load_DemoPlay -> Load_InGame` keeps a
    /// distinct demo-2 preload row only on the later IntroSequence6 revisit
    /// that still carries the old `loc_1BCD0` slot-2 pose but no longer uses
    /// the earlier hidden-camera wrap.
    demo_uses_intro_sequence_6_later_demo_2_preload_row: bool,
    /// ASM loc_1B5CE has already restored the standard title pose before a
    /// later timeout-restart demo starts. When that lineage returns to title,
    /// oracle exposes the pose two frames earlier than the generic demo-title
    /// rebuild window before the usual hide/rebuild choreography finishes.
    demo_title_return_uses_intro_sequence_6_early_title_restore: bool,
    /// ASM sub_1B8F4 / loc_1B920 can re-enter the title shell via
    /// sub_1C246's deferred Start skip. Oracle later exposes that demo's
    /// timeout-restart `loc_1BCD0` slot-2 pose one frame earlier than the
    /// plain timeout path, so keep that lineage alive through the demo's
    /// later auto-timeout return.
    demo_timeout_restart_uses_intro_return_slot_2_early_visibility: bool,
    /// Only the earlier Start-skip intro-return lineage keeps intro sequence 5
    /// alive for one extra shared fade window before `loc_1BA5C` reaches its
    /// `move.w #$28,(Game_Mode).w` hand-off. The later intro-sequence-6 title
    /// restore still inherits the one-frame-earlier slot-2 visibility, but it
    /// must not reuse this extra intro-sequence-5 timeout delay.
    demo_timeout_restart_uses_start_skip_intro_sequence_5_delay: bool,
    /// A later attract demo that starts after IntroSequence6 has already
    /// restored the standard title pose reaches the next timeout-restart
    /// `loc_1BA5C -> Game_Mode=$28` edge one frame earlier than the earlier
    /// Start-skip revisit, even though both lineages share the same early
    /// `loc_1BCD0` slot-2 visibility.
    demo_timeout_restart_uses_intro_sequence_6_early_intro_sequence_5_handoff: bool,
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

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoundDriverWriteSubE1330 {
    pub(crate) queued_bytes: [u8; 3],
    pub(crate) preserves_a0: bool,
    pub(crate) preserves_d0: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoundDriverWriteSubE1334 {
    pub(crate) queued_bytes: [u8; 2],
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoundDriverWriteSubE1338 {
    pub(crate) queued_bytes: [u8; 2],
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoundDriverWriteSubE133C {
    pub(crate) queued_bytes: [u8; 0],
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SoundDriverUploadSubE13DA {
    pub(crate) source_relative_path: &'static str,
    pub(crate) destination_start: u32,
    pub(crate) destination_end_exclusive: u32,
    pub(crate) driver_bytes: Vec<u8>,
    pub(crate) zero_fill_len: usize,
    pub(crate) requests_z80_reset: bool,
    pub(crate) stops_z80: bool,
    pub(crate) preserves_a1: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoundDriverResetSubE1416 {
    pub(crate) requests_z80_reset: bool,
    pub(crate) reset_assert_delay_iterations: u32,
    pub(crate) restarts_z80_bus: bool,
    pub(crate) releases_z80_reset: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoundDriverBeginWriteSubE143A {
    pub(crate) command_head_address: u32,
    pub(crate) ring_buffer_address: u32,
    pub(crate) write_index: u8,
    pub(crate) interrupt_mask: u16,
    pub(crate) requests_z80_bus: bool,
    pub(crate) waits_for_bus_grant: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoundDriverWriteCommandSubE1482 {
    pub(crate) queued_bytes: [u8; 2],
    pub(crate) next_write_index: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoundDriverWriteByteSubE148E {
    pub(crate) queued_byte: u8,
    pub(crate) next_write_index: u8,
    pub(crate) committed_write_index: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoundDriverWriteLongLowByteSubE149C {
    pub(crate) begin_write: SoundDriverBeginWriteSubE143A,
    pub(crate) write: SoundDriverWriteByteSubE148E,
    pub(crate) releases_z80_bus: bool,
    pub(crate) restores_previous_sr: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoundDriverWritePointerSubE14AC {
    pub(crate) begin_write: SoundDriverBeginWriteSubE143A,
    pub(crate) writes: [SoundDriverWriteByteSubE148E; 3],
    pub(crate) releases_z80_bus: bool,
    pub(crate) restores_previous_sr: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoundDriverWriteCommandAndByte {
    pub(crate) begin_write: SoundDriverBeginWriteSubE143A,
    pub(crate) command_write: SoundDriverWriteCommandSubE1482,
    pub(crate) payload_write: SoundDriverWriteByteSubE148E,
    pub(crate) releases_z80_bus: bool,
    pub(crate) restores_previous_sr: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlaneClearPassSetupLoc1698 {
    pub(crate) target_addr: u32,
    pub(crate) dbf_seed: u16,
    pub(crate) next_source_longword: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlaneClearWordFillLoc16B6 {
    pub(crate) target_addr: u32,
    pub(crate) next_addr: u32,
    pub(crate) words: Vec<u16>,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_write_sub_E1330(sound_id: u8) -> SoundDriverWriteSubE1330 {
    // ASM sub_E1330 -> sub_E1546 -> sub_E1552 (kid.asm:49481, 49806):
    // preserve `a0`/`d0`, then queue `FF 05 <sound-id>` into the GEMS ring.
    SoundDriverWriteSubE1330 {
        queued_bytes: [0xFF, 0x05, sound_id],
        preserves_a0: true,
        preserves_d0: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_write_sub_E1334() -> SoundDriverWriteSubE1334 {
    // ASM sub_E1334 -> sub_E155A (kid.asm:49490, 49829): queue `FF 0C`
    // into the GEMS ring with no extra payload byte.
    SoundDriverWriteSubE1334 {
        queued_bytes: [0xFF, 0x0C],
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_write_sub_E1338() -> SoundDriverWriteSubE1338 {
    // ASM sub_E1338 -> sub_E1568 (kid.asm:49499, 49840): queue `FF 0D`
    // into the GEMS ring with no extra payload byte.
    SoundDriverWriteSubE1338 {
        queued_bytes: [0xFF, 0x0D],
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_write_sub_E133C() -> SoundDriverWriteSubE133C {
    // ASM sub_E133C -> loc_E1576 (kid.asm:49508, 49847): enter the shared
    // Z80 write critical section, then return without queueing any command.
    SoundDriverWriteSubE133C { queued_bytes: [] }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn plane_clear_word_fill_loc_16B6(
    target_addr: u32,
    source_word: u16,
    dbf_seed: u16,
) -> PlaneClearWordFillLoc16B6 {
    // ASM loc_16B6 (kid.asm:2811): `move.w (a6),(a0)+` followed by `dbf d0`
    // copies the same source word `d0+1` times and advances `a0` by two bytes
    // per iteration.
    let repeat_count = usize::from(dbf_seed).wrapping_add(1);
    PlaneClearWordFillLoc16B6 {
        target_addr,
        next_addr: target_addr.wrapping_add((repeat_count as u32) * 2),
        words: vec![source_word; repeat_count],
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn plane_clear_word_fill_loc_16C8(
    target_addr: u32,
    source_word: u16,
    dbf_seed: u16,
) -> PlaneClearWordFillLoc16B6 {
    // ASM loc_16C8 (kid.asm:2817): the second clear pass uses the same
    // `move.w (a6),(a0)+` / `dbf d0` loop body as loc_16B6, so it emits
    // `d0+1` copies of the current source word and advances `a0` by two bytes
    // per iteration.
    plane_clear_word_fill_loc_16B6(target_addr, source_word, dbf_seed)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn plane_clear_word_fill_loc_16E0(
    target_addr: u32,
    source_word: u16,
    dbf_seed: u16,
) -> PlaneClearWordFillLoc16B6 {
    // ASM loc_16E0 (kid.asm:2824): the final clear pass reuses the same
    // word-fill loop body, then falls through to `j_sub_924` after the
    // inclusive `dbf` count reaches `-1`.
    plane_clear_word_fill_loc_16B6(target_addr, source_word, dbf_seed)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_upload_sub_E13DA(
    manifest: &AssetManifest,
) -> std::io::Result<SoundDriverUploadSubE13DA> {
    // ASM sub_E13DA (kid.asm:49599): assert Z80 reset, stop the Z80 bus,
    // copy `sound/GEMS_Sounddriver.bin` into `$A00000`, then zero-fill the
    // remainder of `$A00000..$A02000` before restoring `a1`.
    let source_relative_path = "sound/GEMS_Sounddriver.bin";
    let driver_bytes = fs::read(manifest.source_root.join(source_relative_path))?;
    let z80_ram_len = 0x2000usize;
    let zero_fill_len = z80_ram_len.saturating_sub(driver_bytes.len());
    Ok(SoundDriverUploadSubE13DA {
        source_relative_path,
        destination_start: 0xA00000,
        destination_end_exclusive: 0xA02000,
        driver_bytes,
        zero_fill_len,
        requests_z80_reset: true,
        stops_z80: true,
        preserves_a1: true,
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_reset_sub_E1416() -> SoundDriverResetSubE1416 {
    // ASM sub_E1416 (kid.asm:49626): assert Z80 reset, burn the fixed
    // 15-iteration delay loop, restart the bus, then release reset.
    SoundDriverResetSubE1416 {
        requests_z80_reset: true,
        reset_assert_delay_iterations: 15,
        restarts_z80_bus: true,
        releases_z80_reset: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_begin_write_sub_E143A(
    command_head_byte: u8,
) -> SoundDriverBeginWriteSubE143A {
    // ASM sub_E143A (kid.asm:49642): raise the interrupt mask to `$0700`,
    // request the Z80 bus, spin until bit 0 clears, then expose the command
    // head byte from `$A00036` plus the shared GEMS ring base at `$A01A40`.
    SoundDriverBeginWriteSubE143A {
        command_head_address: 0xA00036,
        ring_buffer_address: 0xA01A40,
        write_index: command_head_byte,
        interrupt_mask: 0x0700,
        requests_z80_bus: true,
        waits_for_bus_grant: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_write_command_sub_E1482(
    write_index: u8,
    command_byte: u8,
) -> SoundDriverWriteCommandSubE1482 {
    // ASM sub_E1482 (kid.asm:49675): write the leading `FF`, advance the
    // 6-bit ring index, then fall through into the shared byte writer so the
    // command byte lands immediately afterwards and the head byte commits.
    SoundDriverWriteCommandSubE1482 {
        queued_bytes: [0xFF, command_byte],
        next_write_index: write_index.wrapping_add(2) & 0x3F,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_write_byte_sub_E148E(
    write_index: u8,
    command_byte: u8,
) -> SoundDriverWriteByteSubE148E {
    // ASM sub_E148E (kid.asm:49685): write `d0` into `(a1,d1.w)`, advance the
    // 6-bit GEMS ring index by one, then commit the updated head byte through
    // `a0`.
    let next_write_index = write_index.wrapping_add(1) & 0x3F;
    SoundDriverWriteByteSubE148E {
        queued_byte: command_byte,
        next_write_index,
        committed_write_index: next_write_index,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_write_long_low_byte_sub_E149C(
    command_head_byte: u8,
    value: u32,
) -> SoundDriverWriteLongLowByteSubE149C {
    // ASM sub_E149C (kid.asm:49697): enter the shared Z80/GEMS write section,
    // queue only the low byte from `8(a6)` through sub_E148E, then drop the
    // bus request and restore the caller SR in loc_E1470.
    SoundDriverWriteLongLowByteSubE149C {
        begin_write: sound_driver_begin_write_sub_E143A(command_head_byte),
        write: sound_driver_write_byte_sub_E148E(command_head_byte, value as u8),
        releases_z80_bus: true,
        restores_previous_sr: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_write_pointer_sub_E14AC(
    command_head_byte: u8,
    value: u32,
) -> SoundDriverWritePointerSubE14AC {
    // ASM sub_E14AC (kid.asm:49711): enter the shared GEMS write section, then
    // queue the low three bytes from `8(a6)` by writing `d0`, arithmetic
    // shifting right eight bits, and repeating sub_E148E three times before
    // releasing the Z80 bus in loc_E1470.
    let first = sound_driver_write_byte_sub_E148E(command_head_byte, value as u8);
    let second = sound_driver_write_byte_sub_E148E(first.committed_write_index, (value >> 8) as u8);
    let third =
        sound_driver_write_byte_sub_E148E(second.committed_write_index, (value >> 16) as u8);
    SoundDriverWritePointerSubE14AC {
        begin_write: sound_driver_begin_write_sub_E143A(command_head_byte),
        writes: [first, second, third],
        releases_z80_bus: true,
        restores_previous_sr: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_write_sub_E151C(
    command_head_byte: u8,
    sound_id: u8,
) -> SoundDriverWriteCommandAndByte {
    // ASM sub_E151C (kid.asm:49766): enter the shared Z80/GEMS write section,
    // queue `FF 10` through sub_E1482, then queue the low byte from `8(a6)`
    // through sub_E148E before releasing the bus in loc_E1470.
    let command_write = sound_driver_write_command_sub_E1482(command_head_byte, 0x10);
    SoundDriverWriteCommandAndByte {
        begin_write: sound_driver_begin_write_sub_E143A(command_head_byte),
        command_write,
        payload_write: sound_driver_write_byte_sub_E148E(command_write.next_write_index, sound_id),
        releases_z80_bus: true,
        restores_previous_sr: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sound_driver_write_sub_E153E(
    command_head_byte: u8,
    sound_id: u8,
) -> SoundDriverWriteCommandAndByte {
    // ASM sub_E153E (kid.asm:49796): enter the shared Z80/GEMS write section,
    // queue `FF 12` through sub_E1482, then queue the low byte from `8(a6)`
    // through sub_E148E before releasing the bus in loc_E1470.
    let command_write = sound_driver_write_command_sub_E1482(command_head_byte, 0x12);
    SoundDriverWriteCommandAndByte {
        begin_write: sound_driver_begin_write_sub_E143A(command_head_byte),
        command_write,
        payload_write: sound_driver_write_byte_sub_E148E(command_write.next_write_index, sound_id),
        releases_z80_bus: true,
        restores_previous_sr: true,
    }
}

#[derive(Debug, Clone, Copy)]
struct PendingStompBounce {
    anchor_x_pos: u32,
    anchor_y_pos: u32,
}

impl NativeRuntime {
    pub fn new(assets: AssetManifest) -> Self {
        let sega_screen_data = vdp::load_sega_screen_data(&assets).ok();
        let intro_sub_1b850_data = vdp::load_intro_sub_1b850_data(&assets).ok();
        let title_screen_data = vdp::load_title_screen_data(&assets).ok();
        let mut runtime = Self {
            assets,
            frame_index: 0,
            boot_state: BootState::SegaScreen { elapsed: 0 },
            trace_input: FrameInput::default(),
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
            starting_level_title_workers: Vec::new(),
            starting_level_title_rng: None,
            death_camera_lock: None,
            level_skip_transition: None,
            death_shell_snapshot: None,
            level_skip_resume_hold: false,
            active_enemy_roster: Vec::new(),
            active_enemies: Vec::new(),
            killed_enemies: HashSet::new(),
            drilled_blocks: Vec::new(),
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
            pending_sound_effects: Vec::new(),
            played_sound_effects: Vec::new(),
            enemy_death_effects: Vec::new(),
            stomped_this_frame: false,
            pending_stomp_bounce: None,
            rom_random: RomRandomState::default(),
            blocks_hit_this_frame: 0,
            sound_test_play_pending: false,
            sega_screen_data,
            intro_sub_1b850_data,
            title_screen_data,
            vdp_state: None,
            z80_stop_request_depth: 0,
            z80_guard_sr: 0x2500,
            z80_running: true,
            options_two_player_two_controllers: false,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
            options_menu_panel_toggle_byte: 0,
            options_menu_left_panel_x: 0x0030,
            options_menu_right_panel_x: 0x0110,
            options_menu_panel_delta: 0x0118,
            level_timer: LevelTimer::new(),
            y_look_offset: 0,
            camera_last_kid_y: 0,
            level_damage_taken: false,
            just_received_damage: false,
            level_prize_collected: false,
            clocks_collected: 0,
            hundred_k_trip_timer: 0,
            title_idle_frames: 0,
            title_screen_elapsed_frames: 0,
            title_screen_uses_intro_return_hidden_shell: false,
            title_screen_ready_frame: TITLE_SCREEN_READY_GLOBAL_FRAME,
            title_menu_arm_frame: 0,
            title_menu_active: false,
            title_menu_cursor_pending: false,
            title_menu_selection_pending: None,
            title_menu_selection_delay_frames: 0,
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
            timeout_restart_title_intro_sequence_4_return_pending: false,
            timeout_restart_title_intro_sequence_5_return_pending: false,
            preserve_demo_title_snapshot: false,
            timeout_restart_title_intro_sequence_3_pose: false,
            timeout_restart_title_intro_sequence_4_camera_carry: false,
            timeout_restart_title_intro_sequence_5_pose: false,
            timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay: false,
            timeout_restart_intro_sequence_4_pose_carry: false,
            hide_demo_return_title_player: false,
            ignore_demo_start_abort_until_gameplay: false,
            next_demo_uses_start_skip_intro_return_camera_wrap: false,
            demo_uses_start_skip_intro_return_camera_wrap: false,
            next_demo_uses_demo_title_return_camera_wrap: false,
            demo_uses_demo_title_return_camera_wrap: false,
            demo_uses_intro_sequence_6_hidden_camera_wrap: false,
            demo_uses_intro_sequence_6_early_title_restore: false,
            timeout_restart_intro_sequence_1_uses_early_title_restore: false,
            demo_uses_intro_sequence_6_later_demo_2_preload_row: false,
            demo_title_return_uses_intro_sequence_6_early_title_restore: false,
            demo_timeout_restart_uses_intro_return_slot_2_early_visibility: false,
            demo_timeout_restart_uses_start_skip_intro_sequence_5_delay: false,
            demo_timeout_restart_uses_intro_sequence_6_early_intro_sequence_5_handoff: false,
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
        runtime.update_vdp_state();
        runtime
    }

    #[allow(non_snake_case)]
    #[allow(dead_code)]
    fn stop_z80_guard_sub_914(&mut self) {
        // ASM sub_914 (kid.asm:1027): increment `$FFFFF805`, raise SR to
        // `$2700`, then call `j_Stop_z80` unconditionally. Nested callers keep
        // stacking the byte counter even though the bus is already halted.
        self.z80_stop_request_depth = self.z80_stop_request_depth.wrapping_add(1);
        self.z80_guard_sr = 0x2700;
        self.z80_running = false;
    }

    #[allow(non_snake_case)]
    #[allow(dead_code)]
    fn prepare_plane_clear_pass_loc_1698(&mut self) -> PlaneClearPassSetupLoc1698 {
        self.stop_z80_guard_sub_914();
        // ASM loc_1698 (kid.asm:2803): after `j_sub_914`, the ROM targets
        // `$FE0000`, seeds `d0` with `#$3F` for the first 64-word `dbf` fill
        // pass, and stores `#$20` into `4(a6)` before entering `loc_16B6`.
        PlaneClearPassSetupLoc1698 {
            target_addr: 0x00FE_0000,
            dbf_seed: 0x003F,
            next_source_longword: 0x0000_0020,
        }
    }

    #[allow(dead_code)]
    fn start_z80_guard_sub_924(&mut self) {
        // ASM sub_924 (kid.asm:1038): decrement the shared byte, branch on a
        // signed `bgt` if nested stops remain, otherwise clear the byte,
        // restore SR=`$2500`, and call `j_Start_z80`.
        let remaining_depth = self.z80_stop_request_depth.wrapping_sub(1);
        self.z80_stop_request_depth = remaining_depth;
        if (remaining_depth as i8) > 0 {
            return;
        }
        self.z80_stop_request_depth = 0;
        self.z80_guard_sr = 0x2500;
        self.z80_running = true;
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

    /// Set the block collision grid for the current level.
    /// Call after constructing the runtime, before stepping frames.
    pub fn set_block_grid(&mut self, grid: terrain::BlockGrid) {
        self.block_grid = Some(grid);
        self.block_grid_source = BlockGridSource::Manual;
    }

    /// Rebuild VDP state based on the current boot phase.
    /// Called once per frame after `step_boot_state`.
    fn update_vdp_state(&mut self) {
        self.vdp_state = match &self.boot_state {
            BootState::SegaScreen { elapsed } => self
                .sega_screen_data
                .as_ref()
                .map(|data| vdp::build_sega_screen_vdp(data, *elapsed)),
            BootState::IntroSequence { segment, .. } => self
                .intro_sub_1b850_data
                .as_ref()
                .map(|data| vdp::build_intro_sequence_vdp(data, *segment)),
            BootState::IntroSequence6 { .. } | BootState::TitleScreen { .. } => self
                .title_screen_data
                .as_ref()
                .map(vdp::build_title_screen_vdp),
            _ => None,
        };
    }

    pub fn block_grid(&self) -> Option<&terrain::BlockGrid> {
        self.block_grid.as_ref()
    }

    /// Returns the current VDP state for non-gameplay screen rendering.
    /// `Some` during Sega screen, intro sequences, and title screen;
    /// `None` during gameplay where the pre-extracted asset pipeline renders.
    pub fn vdp_state(&self) -> Option<&vdp::VdpState> {
        self.vdp_state.as_ref()
    }

    pub fn step_frame(&mut self, input: FrameInput) -> NativeTraceFrame {
        let raw_input = input;
        self.trace_input = input;
        self.just_received_damage = false;
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
            self.trace_input = raw_input;
            return self.trace_frame();
        };
        self.trace_input = input;
        self.maybe_begin_pending_level_skip();
        self.maybe_begin_pending_level_skip_resume();
        self.maybe_finish_level_skip_resume();
        let freeze_demo_auto_timeout_snapshot = self.demo_auto_timeout_freeze_gameplay_frame;
        // ASM loc_C4E (kid.asm:1404): ReadJoypad edge detection —
        // `eor.b d1,d0; and.b (Ctrl_1).w,d0; or.b d0,(Ctrl_1_Pressed).w`
        // computes newly-pressed bits, then copies held state:
        // `move.b (Ctrl_1).w,(Ctrl_1_Held).w`. Native equivalent: compare
        // current `input` fields against `self.previous_input` for rising
        // edges, then store `self.previous_input = input` at frame end.
        let start_pressed = input.start && !self.previous_input.start;
        let apply_pending_pause_toggle = self.pause_toggle_pending;
        self.pause_toggle_pending = false;
        self.handle_start_press(input, start_pressed);
        let frame_started_in_sega_screen = matches!(self.boot_state, BootState::SegaScreen { .. });
        let frame_started_in_title_screen =
            matches!(self.boot_state, BootState::TitleScreen { .. });
        let frame_started_in_level_entry =
            matches!(self.boot_state, BootState::StartingLevel { .. });
        self.step_boot_state(input, start_pressed);
        self.update_vdp_state();
        if frame_started_in_sega_screen
            && matches!(self.boot_state, BootState::IntroSequence { .. })
        {
            // ASM Sega/logo boot still runs through Mode_Standard on the frame
            // that flips into `Game_Mode=$0004`, so Execute_Objects burns one
            // `Get_RandomNumber_long` before IntroSequence1 becomes visible on
            // the traced frame.
            let _ = self.rom_random.next_long();
        }
        if frame_started_in_title_screen && self.title_screen_rng_long_burn_active() {
            // ASM TitleScreen still runs through Mode_Standard
            // (kid.asm:854-894), but oracle keeps `RNG_Offset` frozen through
            // the hidden title rebuild window after the early `$002C`
            // hand-off. Only burn `Get_RandomNumber_long` outside that
            // oracle-backed freeze range.
            let _ = self.rom_random.next_long();
        }
        if frame_started_in_level_entry
            && let BootState::StartingLevel {
                elapsed_frames,
                uses_load_ingame_bootstrap,
                ..
            } = self.boot_state
        {
            let cold_boot_titlecard_long_burn =
                    // Oracle WRAM watches keep the cold-boot mode-$0008 shell
                    // RNG-idle through the hidden carry window, then expose a
                    // single pre-visible `Execute_Objects` burn on the frame
                    // immediately before the loc_1AB26 / loc_1AD48 seed
                    // batches wake.
                    elapsed_frames
                        == crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME
                            .saturating_sub(4)
                        // The burn resumes on the first visible frame when
                        // Execute_Objects first runs the loc_1AB26/loc_1AD48
                        // workers, and continues every frame after.
                        || elapsed_frames
                            >= crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME;
            if uses_load_ingame_bootstrap || cold_boot_titlecard_long_burn {
                // ASM loc_6E2 -> Load_TitleCard keeps the cold-boot
                // mode-$0008 title-card shell dormant through an early
                // hidden pre-roll. The burn resumes once the
                // loc_1AB26/loc_1AD48 workers are live within
                // Execute_Objects — frame 43 (letter batch) and 44
                // (deferred number) both burn, matching oracle delta of
                // 38 bytes (4 long + 34 worker bytes).
                let _ = self.rom_random.next_long();
            }
        }
        if let BootState::StartingLevel {
            elapsed_frames,
            uses_load_ingame_bootstrap,
            load_ingame_bootstrap_started,
            ..
        } = self.boot_state
        {
            if uses_load_ingame_bootstrap
                && !load_ingame_bootstrap_started
                // ASM Obj_TitleMenu Title_InputLoop/loc_1BB52 still owns the
                // carried early-start shell for this window. Keep a separate
                // loc_1AC70 seed stream that only begins burning RNG once the
                // later Load_TitleCard worker lineage is active.
                && elapsed_frames >= EARLY_START_TITLE_MENU_SHELL_CARRY_FRAMES
                && let Some(title_rng) = self.starting_level_title_rng.as_mut()
            {
                let _ = title_rng.next_long();
            }
            self.maybe_seed_starting_level_title_workers(elapsed_frames);
        }
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
            self.step_pending_sound_effects();
            // ASM loc_892 (kid.asm:956): Mode_Level rendering tail join point.
            // When $FFFFFB49 (paused), skip j_DiamondPower_CompileSprites and
            // Make_SpritesFromPlatforms but still run j_sub_30194 (HUD sprites),
            // j_loc_3038A (scroll data transfer / Time_Frames), sub_14C0
            // (palette fade). All three tail calls are visual/VDP — not modelled.
            // Diamond Power uses raw input (Start+A held) since gameplay_input delays A by 1 frame
            if matches!(self.player_lifecycle, PlayerLifecycle::Alive)
                && gameplay_slice_active
                && !self.paused
            {
                self.step_diamond_power(input);
            }
            if gameplay_slice_active {
                // ASM loc_1AE2E only gates the later `Game_Mode=$000C`
                // transition; the hidden mode-$0008 level-entry shell still
                // runs the gameplay/object slice before that visible Start.
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
                    .map_or(demo::DEMO_GAMEPLAY_START_FRAME, |use_demo_2| {
                        self.demo_gameplay_start_frame(use_demo_2)
                    });
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
        // ASM loc_C4E (kid.asm:1410): `move.b (Ctrl_1).w,(Ctrl_1_Held).w` —
        // latch current held state for next-frame edge detection. Also
        // loc_C9C: route Ctrl_1_Held/Pressed → Ctrl_Held/Pressed (1P default).
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
        // sub_AF10 (kid.asm:13750): kid_flip wall scan — called at
        // kid_flip entry (`bsr.w sub_AF10; beq.w loc_AD34`). Scans the
        // player's leading-edge column for wall tiles across the full
        // body height.
        //   loc_AF2E (kid.asm:13760): helmet-size merge point. d5=$F
        //   for Micromax/Juggernaut (16px body), $1F for others (32px).
        //   `sub.w d5,d6` → d6 = top of body. `asr.w #4` both → row
        //   indices. `sub.w d6,d7` → row count. Load row pointer from
        //   $4A04, add FA78+1 column offset (negated if FA72≠0).
        //   loc_AF52 (kid.asm:13774): column resolve — `add.w x_pos,d6;
        //   asr.w #4,d6; add.w d6,d6; add.w d6,a4` → tile pointer.
        //   loc_AF5C (kid.asm:13780): scan loop — `move.w (a4),d6;
        //   andi.w #$7000,d6; cmpi.w #$6000,d6; bge loc_AF76` — if
        //   wall ($6000+) → hit. Else `add.w Level_width_tiles,a4;
        //   dbf d7,loc_AF5C` — step down one row, repeat.
        //   loc_AF76 (kid.asm:13791): `moveq #1,d7; rts` — wall found.
        //   Fall-through: `moveq #0,d7; rts` — no wall in column.
        // Native: `probe_wall` (3-point body scan) covers the same
        // column. The 3-point probe at y-8, y-h/2, y-h+8 samples every
        // tile row for 32px bodies; for Micromax (16px) the points
        // collapse to fewer distinct rows but still cover the column.
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
                // ASM loc_AC42 (kid.asm:13474): non-Kid B-press in kid_flip.
                // Micromax: y_vel=$FFFC0000, x_vel=±$FFFE0000, flip direction,
                // init stru_8B74, bra loc_A6F8. Same velocities as loc_A6AE
                // (platform-grab detach).
                PlayerForm::Micromax => KidFlipBranchSelection::BPressedMicromax,
                // ASM loc_AC88 (kid.asm:13495): non-Kid/non-Micromax B-press
                // in kid_flip. `tst.b ($FFFFFA6A); beq loc_AD2C` — if on
                // Skycutter board: floor probe (sub_B43A) or y near top →
                // loc_ACA4 (`move.l #$30000,y_vel` — board dive). Otherwise
                // fall through to loc_AD2C (no action). In practice only
                // Skycutter can have $FA6A set, so other helmets always skip.
                // Native: step_skycutter_flipboard B-press handles the dive.
                _ if player_before.on_ground => KidFlipBranchSelection::BPressedGroundedLaunch,
                _ => KidFlipBranchSelection::BPressedFallthrough,
            }
        } else if c_pressed {
            // ASM loc_ACB0 (kid.asm:13510): kid_flip C-press dispatch.
            // Cyclone → st (Cyclone_flying). Maniaxe → loc_ACCC: axe throw.
            // Skycutter → loc_AD00/loc_AD26: flipboard deploy. Others → skip.
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
            velocity_branch: if player.on_ground && jump_pressed {
                KidGroundedReleaseVelocityBranch::JumpPressed
            } else if player.on_ground && player.jump_buffer_frames > 0 {
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
        if self.block_grid.is_none() {
            self.progression.form_width_delta = 0;
            return;
        }

        if let Some(aligned_x) =
            self.resolve_transform_overlap_sub_7b30(slice, self.progression.current_form)
        {
            slice.player.x_pos = (aligned_x as u32) << 16;
        }

        self.progression.form_width_delta = 0;
    }

    fn resolve_transform_overlap_sub_7b30(
        &self,
        slice: &OpeningSliceState,
        target_form: PlayerForm,
    ) -> Option<i32> {
        let grid = self.block_grid.as_ref()?;
        let x_px = (slice.player.x_pos >> 16) as i32;
        let foot_y_px = (slice.player.y_pos >> 16) as i32;
        let half_width = target_form.half_width();
        // ASM loc_757E (kid.asm:7691–7698): `moveq #$10,d6` then adds $10
        // unless Current_Helmet_Available is Micromax (9) or Juggernaut (5).
        // d6=$10 → short form (1 cell tall), d6=$20 → tall form (2 cells).
        // sub_7B30 converts: d6==$10 → d5=0 rows, else d5=1 row.
        let probe_height = target_form.ceiling_probe_height() + 1;
        let mut rows_up = if probe_height == 0x10 { 0 } else { 1 };
        // ASM loc_7B48 (kid.asm:8241): `move.w y_pos(a3),d4; andi.w #$F,d4;
        // cmpi.w #$F,d4; beq.w loc_7B5A; addq.w #1,d5` — if foot pixel is
        // NOT tile-aligned (y_pos & $F != $F), add one extra row to scan.
        if foot_y_px & 0xF != 0xF {
            rows_up += 1;
        }

        // ASM loc_7B5A (kid.asm:8248): `move.w x_pos(a3),d6; sub.w d7,d6;
        // bpl.w loc_7B74` — compute left_edge = x_pos - half_width. If
        // negative (off-screen left), snap x to half_width and check right.
        let left_edge = x_px - half_width;
        if left_edge < 0 {
            let cleared_x = half_width;
            let right_edge = cleared_x + half_width;
            // ASM loc_7BB6 (kid.asm:8287): `move.w 2(sp),d7; bsr.w sub_7BEA;
            // beq.s loc_7B9E` — shared right-column probe entry. Reached from
            // both the negative-left-edge snap (loc_7B5A) and the left-blocked
            // snap-right (loc_7BA4). Probes right_edge column upward; if clear
            // → loc_7B9E (success). If blocked → loc_7BC0 (restore & fail).
            return if terrain::transform_overlap_column_blocked_sub_7bea(
                grid, right_edge, foot_y_px, rows_up,
            ) {
                // ASM loc_7BC0 (kid.asm:8292): `moveq #1,d7; addq.w #4,sp;
                // move.l d0,x_pos(a3); move.l d1,y_pos(a3); rts` — both
                // snaps failed, restore original position and return blocked.
                None
            } else {
                Some(cleared_x)
            };
        }

        // ASM loc_7B74 (kid.asm:8259): `move.w d6,d4; move.w 2(sp),d7;
        // bsr.w sub_7BEA; bne.w loc_7BA4` — left edge non-negative; probe
        // left column for walls. If blocked, jump to loc_7BA4 (snap right).
        if terrain::transform_overlap_column_blocked_sub_7bea(grid, left_edge, foot_y_px, rows_up) {
            // ASM loc_7BA4 (kid.asm:8279): `andi.w #$FFF0,d4; addi.w #$10,d4;
            // add.w (sp),d4; move.w d4,x_pos(a3)` — snap x_pos right to the
            // next tile boundary + half_width, then check the right column.
            let cleared_x = (left_edge & !0xF) + 16 + half_width;
            let right_edge = cleared_x + half_width;
            // Falls through to loc_7BB6 (kid.asm:8287) — right-column probe
            // after snap. loc_7BC0 if still blocked.
            return if terrain::transform_overlap_column_blocked_sub_7bea(
                grid, right_edge, foot_y_px, rows_up,
            ) {
                None
            } else {
                Some(cleared_x)
            };
        }

        // ASM loc_7B74 continued (kid.asm:8264–8271): left clear — probe right
        // column. `cmp.w (Level_width_pixels).w,d6; bge.w loc_7BCE` — if right
        // edge >= level width, snap left. Otherwise sub_7BEA on right column.
        let right_edge = x_px + half_width;
        if right_edge < grid.width_pixels()
            && !terrain::transform_overlap_column_blocked_sub_7bea(
                grid, right_edge, foot_y_px, rows_up,
            )
        {
            // ASM loc_7B9E (kid.asm:8273): `moveq #0,d7; addq.w #4,sp; rts`
            // — both columns clear, return success with position unchanged.
            return Some(x_px);
        }

        // ASM loc_7BCE (kid.asm:8300): right column blocked or off-level —
        // snap x_pos left: `(right_edge & $FFF0) - 1 - half_width`.
        let cleared_x = (right_edge & !0xF) - 1 - half_width;
        if cleared_x < 0 {
            return None;
        }
        let left_edge = cleared_x - half_width;
        // ASM loc_7BCE cont. (kid.asm:8307–8310): `bsr.w sub_7BEA; beq.s loc_7B9E;
        // bra.s loc_7BC0` — if left probe after snap-left also blocked → loc_7BC0.
        if terrain::transform_overlap_column_blocked_sub_7bea(grid, left_edge, foot_y_px, rows_up) {
            None
        } else {
            Some(cleared_x)
        }
    }

    fn handle_pending_form_change_sub_7acc(&mut self, slice: &mut OpeningSliceState) -> bool {
        let Some(target_form) = self.progression.pending_form else {
            return false;
        };

        // ASM sub_7ACC (kid.asm:8178): when `Check_Helmet_Change` is set, try
        // sub_7B30's space-clear using the target helmet hitbox. If it fails,
        // Kid drops the queued helmet, while any already-helmeted form dies.
        // ASM loc_8252 (kid.asm:9013): in crawling state (loc_8218), after
        // sub_7ACC returns NE, Micromax(9) and Juggernaut(5) exit crawling
        // via `bclr #Button_B; bra.w loc_8E16`; other forms continue
        // crawling. Native: no separate crawling state, so transforms
        // resolve uniformly here.
        //
        // ASM loc_7AD6 (kid.asm:8185): `cmpi.w #6,($FFFFFA56).w; beq.w loc_7B12`
        // — if Character_Movement == 6 (uphill-downhill/slope), skip the
        // sub_7B30 overlap check entirely and jump straight to Kid_Transform.
        // TODO(fidelity): native always runs sub_7B30; slope bypass not modelled.
        //
        // ASM loc_7B0A (kid.asm:8202): short-form helmets (Micromax=9,
        // Juggernaut=5) keep d6=$10; others get d6=$20 via `addi.w #$10,d6`.
        // Native: `ceiling_probe_height()` returns 15 vs 31, producing the
        // same rows_up values in `resolve_transform_overlap_sub_7b30`.
        if let Some(aligned_x) = self
            .block_grid
            .as_ref()
            .and_then(|_| self.resolve_transform_overlap_sub_7b30(slice, target_form))
        {
            // ASM loc_7B12 (kid.asm:8206): `bra.w Kid_Transform` — sub_7B30
            // returned 0 (space clear), proceed with the helmet transform.
            slice.player.x_pos = (aligned_x as u32) << 16;
            self.resolve_pending_form_change();
            self.apply_form_width_overlap_fixup(slice);
            return false;
        }

        if self.block_grid.is_none() {
            self.resolve_pending_form_change();
            self.apply_form_width_overlap_fixup(slice);
            return false;
        }

        // ASM loc_7B1A (kid.asm:8213): sub_7B30 returned non-zero (blocked).
        // `tst.w (Current_Helmet).w` — if helmeted, `bra.w Death` (crush).
        // ASM loc_7B28 (kid.asm:8220): if unhelmeted Kid, `sf (Check_Helmet_Change).w`
        // — cancel the pending transform silently and return d7=0.
        self.progression.pending_form = None;
        if self.progression.current_form == PlayerForm::TheKid {
            return false;
        }

        self.progression.number_hitpoints = 0;
        self.opening_slice = Some(*slice);
        self.begin_death_windup();
        true
    }

    fn refresh_opening_slice(&mut self, input: FrameInput, jump_pressed: bool) {
        if !self.step_lifecycle(input) {
            self.demo_transform_active = false;
            return;
        }
        self.update_demo_transform_parity();

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
        slice.rubber_block_hit = None;
        slice.smash_block_hit = None;
        slice.shooter_block_hit = None;
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
        // ASM sub_114C0 / sub_114F4 are the same style of coroutine: shooter
        // headbutt tiles update before physics on their wake frames.
        self.step_shooter_headbutt_workers();
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
        // ASM loc_8CB6 (kid.asm:10100): walking-state C-press → Maniaxe axe throw.
        // ASM loc_8CF8 (kid.asm:10117): walking-state C-press → Red Stealth sword.
        // ASM loc_8C26 (kid.asm:10058): walking-state sub_7428 + Eyeclops hard beam.
        // Native runs all three every frame, not only in the walking state.
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

            // ASM loc_726C/loc_727E (kid.asm:7377/7385): Y-axis look-offset
            // pull-back adjustments run once per frame inside sub_71E4, before
            // the loc_728C offset application.  Applied here so the adjusted
            // y_look_offset is used by whichever physics branch calls
            // update_camera_follow below.
            {
                let kid_y = (slice.player.y_pos >> 16) as i16;
                let cam_y = (slice.camera.y_pos >> 16) as i16;
                let max_y = i32::from(slice.level_bounds.height_pixels) - physics::VIEWPORT_HEIGHT;
                let target_y = (i32::from(kid_y) - physics::CAMERA_PLAYER_OFFSET)
                    .max(0)
                    .min(max_y);
                self.y_look_offset += physics::camera_y_ceiling_look_adjust_loc_726c(
                    target_y,
                    kid_y,
                    self.camera_last_kid_y,
                );
                self.y_look_offset +=
                    physics::camera_y_viewport_top_look_adjust_loc_727e(kid_y, cam_y);
                // ASM loc_75D4 (kid.asm:7712–7714):
                //   move.w x_pos(a3),($FFFFFA2C).w
                //   move.w y_pos(a3),($FFFFFA2E).w
                // Stores Kid's current position for camera follow reference.
                // Native only models the Y component ($FFFFFA2E); the X
                // counterpart ($FFFFFA2C) is not consumed by any camera logic.
                self.camera_last_kid_y = kid_y;
            }

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
                // ASM sub_902A lands only after the contact row is reached. The
                // legacy touch-snap shortcut is useful for non-AsmFaithful
                // orders, but AsmFaithful must preserve the remaining
                // sub-half-pixel gap until the real contact row or gameplay and
                // demo arcs land one frame early.
                let allow_asm_faithful_solid_touch_snap =
                    self.physics_order != PhysicsOrder::AsmFaithful;
                self.begin_kid_grounded_release_probe(
                    slice.player,
                    input,
                    jump_pressed,
                    custom_gravity,
                );
                slice.step_with_order(
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
            // ASM sub_AF7A: IronKnight breaks floor blocks on landing.
            // Called from loc_A94E (kid.asm:13239) in the jump-loop landing
            // sequence: `bsr.w sub_AF7A; bne.w loc_A762`. Runs after
            // sub_B000 (Red Stealth floor impact) and before sub_DB22.
            // Checks after physics so we know the player just touched a floor cell.
            // Not-IK or velocity < $48000 (without prior flag) → loc_AFD0
            // (kid.asm:13830): `moveq #0,d7; rts` — no break, return 0.
            if self.progression.current_form == PlayerForm::IronKnight
                && slice.player.on_ground
                && pre_physics_y_velocity > 0
            {
                let px = (slice.player.x_pos >> 16) as i32;
                let py = (slice.player.y_pos >> 16) as i32;
                let hw = self.progression.current_form.half_width();
                if let Some(ref mut grid) = self.block_grid {
                    let probe = terrain::probe_floor_hitbox(grid, px, py, hw);
                    // ASM loc_AF96 (kid.asm:13808): active break path — `st
                    // (Iron_Knight_block_breaker).w`, save regs, `bsr.w
                    // sub_BBE2` to probe for a block, then dispatch through
                    // off_AFD8 jump table.  On hit: `move.l #$FFFF8000,
                    // y_vel(a3); moveq #1,d7; rts`.  No block or block ID
                    // > 5 → loc_AFCA (kid.asm:13826): restore regs, fall
                    // through to loc_AFD0 (`moveq #0,d7; rts`).
                    if probe.on_ground && probe.cell.has_block {
                        match probe.cell.block_type {
                            // ASM loc_AFF0 (kid.asm:13854): off_AFD8 block
                            // type 0 (Smash) handler — `moveq #2,d6; jmp
                            // (j_Block_LoadSmashBits).l`.
                            terrain::BlockType::Smash => {
                                grid.set_cell_at(
                                    probe.cell_x,
                                    probe.cell_y,
                                    terrain::Cell::from_be_bytes(0x00, 0x00),
                                );
                                self.spawn_smash_block_effect_sub_10eda(
                                    probe.cell_x * 16,
                                    probe.cell_y * 16,
                                    Self::BLOCK_DESTROY_DIRECTION_ABOVE,
                                );
                                slice.player.y_velocity = -0x8000;
                                slice.player.on_ground = false;
                            }
                            terrain::BlockType::Prize => {
                                grid.set_cell_at(
                                    probe.cell_x,
                                    probe.cell_y,
                                    terrain::Cell::from_be_bytes(0x00, 0x00),
                                );
                                self.block_destroy_effects.push(BlockDestroyEffect {
                                    x_px: probe.cell_x * 16,
                                    y_px: probe.cell_y * 16,
                                    remaining: 8,
                                });
                                slice.player.y_velocity = -0x8000;
                                slice.player.on_ground = false;
                            }
                            terrain::BlockType::Ice => {
                                grid.set_cell_at(
                                    probe.cell_x,
                                    probe.cell_y,
                                    terrain::Cell::from_be_bytes(0x00, 0x00),
                                );
                                self.spawn_ice_block_effect_sub_10f9c(
                                    probe.cell_x * 16,
                                    probe.cell_y * 16,
                                    Self::BLOCK_DESTROY_DIRECTION_ABOVE,
                                );
                                slice.player.y_velocity = -0x8000;
                                slice.player.on_ground = false;
                            }
                            // ASM off_AFD8 -> loc_110D0/sub_11120: metal
                            // blocks bounce, play the steel hit effect, then
                            // restore $E50B instead of being deleted.
                            terrain::BlockType::Metal => {
                                slice.metal_block_hit = Some((probe.cell_x, probe.cell_y));
                                slice.player.y_velocity = -0x8000;
                                slice.player.on_ground = false;
                            }
                            // ASM off_AFD8 -> sub_AFD4: SolidGhost and
                            // Teleporter blocks are not broken by this path.
                            terrain::BlockType::SolidGhost | terrain::BlockType::Teleporter => {}
                            _ => {}
                        }
                    }
                }
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
        let spawn_support_snapped = demo_bootstrap_support_active
            && self
                .block_grid
                .as_ref()
                .is_some_and(|grid| self.try_settle_on_spawn_support(&mut slice, grid));
        let demo_bootstrap_floor_snapped = self.try_snap_demo_bootstrap_floor(&mut slice);
        if spawn_support_snapped || demo_bootstrap_floor_snapped {
            slice.update_camera_follow(self.y_look_offset);
        }

        // ASM loc_8218 (kid.asm:8997): crawling state entry — sets
        // `Character_Movement = MoveID_Crawling`, calls sub_71E4 (camera
        // follow), hibernates 1 frame, sets half_width = $E (14px), runs
        // Character_CheckCollision, saves x/y to $FFFFFA2C/$FFFFFA2E, then
        // sub_7ACC (helmet change check). Native: the crawling state is not
        // a separate movement ID — down-held+grounded is handled inline.
        //
        // ASM loc_825C (kid.asm:9018): crawl continuation — if down still
        // held (`tst.b (Ctrl_Down_Held)`), branch to loc_82D8 (crawl loop).
        // If released, `subi.w #8,($FFFFFB58)` decrements the look offset,
        // clamped to 0, then falls through to loc_8270 (stand-up check).
        //
        // ASM loc_8C12 (kid.asm:10051): walking-state version —
        // `tst.b (Ctrl_Down_Held); bne.s loc_8C26` skips decrement when down
        // held; `subi.w #8,($FFFFFB58); bge.s loc_8C26` decrements, clamp to 0.
        //
        // ASM loc_7650 (kid.asm:7741–7746): same offset decay in other
        // movement states. Native merges both paths here.
        if input.down && slice.player.on_ground {
            self.y_look_offset = (self.y_look_offset + 1).min(30);
        } else if self.y_look_offset > 0 {
            self.y_look_offset = (self.y_look_offset - 8).max(0);
        }

        // ASM loc_8270 (kid.asm:9025): stand-up from crawl — `bsr.w
        // sub_DAA6` checks for platform overhead (returns NE → can't stand),
        // `moveq #-$11,d7; bsr.w sub_B43A` floor probe, Skycutter gate,
        // then `bsr.w sub_81B4` wall escape. Native: physics handles floor
        // probe implicitly; sub_81B4 is crawl_wall_escape_sub_81B4;
        // sub_DAA6 is ceiling_probe_height (platform-overhead not yet
        // modelled as a separate stand-up gate — TODO(fidelity)).
        //
        // ASM loc_8298 (kid.asm:9038): when sub_B43A finds floor (EQ),
        // `bsr.w sub_B41C` restores the standing half_width from unk_B408
        // (e.g. TheKid=5, Skycutter=10). Falls through to loc_829C which
        // clears Button_B and dispatches: Skycutter → walking (sub_942A →
        // loc_8BF0), others → loc_82BA.
        // Native: collision half_width always comes from
        // `PlayerForm::half_width()`, so the sub_B41C restore is implicit.
        //
        // ASM loc_82BA (kid.asm:9050): non-Skycutter path after stand-up —
        // `tst.l x_vel(a3)`: if non-zero, `bsr.w sub_78E8` then `bra.w
        // loc_75D4` (standing loop); if zero → loc_82CA.
        // Native: `refresh_grounded_helmet_state_sub_78E8` runs every
        // grounded frame (below), covering both x_vel paths.
        //
        // ASM loc_82CA (kid.asm:9057): zero-velocity idle entry from crawl
        // stand-up — `clr.w ($FFFFF8F0)` (walk accumulator), `bclr
        // #Button_B,(Ctrl_Pressed)`, `bra.w loc_8E3E`. Visual-only: the
        // walk accumulator and sprite dispatch at loc_8E3E are not modelled.
        //
        // ASM loc_8C70 (kid.asm:10079): walking-state crawl transition —
        // `bclr #Button_B,(Ctrl_Pressed); bne.w loc_A426` (jump dispatch),
        // then `tst.b (Ctrl_Down_Held); beq.s loc_8CB6` (skip if down not
        // held), `tst.b (KidGrabbedByHand); bne.w loc_8CB6` (skip if hand-
        // grabbed), `addq.w #1,($FFFFFB58)`, helmet != Micromax(9) and
        // != Juggernaut(5), `bsr.w sub_7A10` (overhead clearance), then
        // MoveID_Crawling → loc_8218. Native: down-held + on_ground +
        // form check + sub_7A10 alignment merged here.
        if input.down
            && slice.player.on_ground
            && !matches!(
                self.progression.current_form,
                PlayerForm::Micromax | PlayerForm::Juggernaut
            )
            && let Some(grid) = self.block_grid.as_ref()
        {
            let down_hold_alignment = movement::align_player_for_down_hold_sub_7A10(
                grid,
                slice.player.x_pos,
                slice.player.y_pos,
                slice.level_bounds.width_pixels,
            );
            slice.player.x_pos = down_hold_alignment.x_pos;
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

        // ASM loc_787E (kid.asm:7924): when the standing-still platform block
        // (loc_7828) clears Addr_PlatformStandingOn and x_vel is zero, the ROM
        // calls sub_78E8 then returns to loc_75D4.
        // ASM loc_78C6 (kid.asm:7957): when Addr_PlatformStandingOn is null at
        // loc_7828, the ROM zeroes x_vel/y_vel, runs sub_B43A floor check:
        //   bmi → loc_BC80 (no floor, death/fall state)
        //   beq → loc_B5F6 (no solid/spikes cell, drop 1px)
        //   positive → sub_78E8 + loop back to loc_75D4 (standing-still).
        // Native: grounded flag already set; velocity is zero from the movement
        // system; sub_78E8 runs unconditionally below.
        // ASM loc_789C (kid.asm:7937): when sub_902A finds floor and sub_8ED0
        // confirms platform support, the ROM loads $FFFFFA98 into d0 (dead —
        // immediately overwritten by sub_78E8's `move.w (Current_Helmet),d0`),
        // calls sub_78E8, and returns to loc_75D4.
        // Native runs sub_78E8 unconditionally each grounded frame, covering
        // the "platform gone" (loc_787E/loc_78C6) and "platform supported"
        // (loc_789C) paths.
        if slice.player.on_ground {
            self.refresh_grounded_helmet_state_sub_78E8();
        }

        // ASM loc_7FC (kid.asm:915): `jsr (j_sub_F7E0).l; bsr.w sub_6034`
        // �� Mode_Level mid-loop calls shooter slot tick (sub_F730 via the
        // j_sub_F7E0 trampoline) then dispatches level special effects
        // (storm/hail weather via sub_6034). Both are skipped when
        // $FFFFFB49 is set (paused); native gates on gameplay_slice_active.
        //
        // ASM loc_806 (kid.asm:919): `jsr (j_sub_F096).l` — shooter block
        // scan. Detects new shooter blocks entering Kid's ±3-col / -5..+2-row
        // window when Kid crosses a block boundary, registers them for the
        // 22-frame fire-and-expire cycle driven by sub_F730. Native merges
        // both scan (sub_F096) and tick (sub_F730) into step_shooter_blocks.
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
            // Visual: shake the prize block (same as Metal block bounce).
            // ASM sub_1007A bounce coroutine wiggles the block for ~31 frames.
            self.block_bounces.push(BlockBounceAnim {
                cell_x: col,
                cell_y: row,
                remaining: 8,
            });
        }

        self.process_block_effects(&mut slice);
        self.collect_floating_items(&mut slice);
        // ASM loc_75D4 (kid.asm:7715): `bsr.w sub_7ACC` — check pending
        // helmet transform after collision processing.  In native, this runs
        // for all movement states (the ROM calls it from each state handler).
        if self.handle_pending_form_change_sub_7acc(&mut slice) {
            return;
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
