use crate::physics::{Direction, LevelBounds};
use crate::terrain;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

/// Controls the order of physics sub-steps within each frame.
/// Used for data-driven comparison of native vs ASM execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PhysicsOrder {
    /// Legacy native order: gravity (update_vertical_motion) before collisions.
    GravityFirst,
    /// Experimental: position update first, then collisions, then gravity.
    PositionFirst,
    /// ASM-faithful separation matching the exact sub-step order:
    #[default]
    /// 1. update_horizontal_velocity_only()  — sub_B084: x_vel only
    /// 2. apply_jump_and_gravity_vel_only()  — sub_B168: y_vel only
    /// 3. apply_y_velocity_to_position()     — y_pos += y_vel
    /// 4. ceiling + floor collision           — sub_902A (old x_pos, new y_pos)
    /// 5. apply_x_velocity_to_position()     — x_pos += x_vel
    /// 6. wall collision                      — sub_8F26 (new x_pos)
    AsmFaithful,
}

/// Sub-frame physics trace: captures intermediate values at each sub-step
/// within `step_with_order` for transparent debugging of physics divergences.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PhysicsTrace {
    // Pre/post horizontal motion
    pub pre_horizontal: (u32, i32),
    pub post_horizontal: (u32, i32),
    pub horizontal_step_applied: bool,

    // Pre/post vertical motion
    pub pre_vertical: (u32, i32),
    pub post_vertical: (u32, i32),

    // Gravity specifics
    pub pre_gravity_y_vel: i32,
    pub post_gravity_y_vel: i32,
    pub gravity_applied: bool,
    pub ascending_and_b_held: bool,

    // Collision results
    pub wall_hit: bool,
    pub wall_snap_x: Option<u32>,
    pub ceiling_hit: bool,
    pub floor_hit: bool,
    pub floor_y_px: Option<i32>,
    pub landing_path: Option<LandingPath>,
    pub on_ground_before: bool,
    pub on_ground_after: bool,

    // Jump
    pub jump_initiated: bool,
    pub jump_y_vel: Option<i32>,

    // Final state
    pub final_pos: (u32, u32),
    pub final_vel: (i32, i32),
}

/// Which ASM floor landing path was taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LandingPath {
    /// ASM loc_AAF8: `move.w d7,y_pos(a3)` — keeps sub-pixel.
    PreserveSubPixel,
    /// ASM loc_AB22: `clr.w $20(a3)` — zeroes sub-pixel.
    ZeroSubPixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ActiveProjectileKind {
    #[default]
    Generic,
    FireDemonDeathTrail,
    /// Walking Dragon breath child: ASM `Dragon.asm loc_3D02A` initialises
    /// `stru_3CCC8` as a stationary hostile GfxObject at the dragon mouth.
    DragonWalkingBreath,
    /// Drips slime drop: ASM `Drips.asm loc_33424`/`loc_3345C` spawn a falling
    /// child object that accelerates under `$2000` gravity.
    DripsDrop,
    /// Drips surface crawler: ASM `Drips.asm loc_335D8` allocates a hostile
    /// child after selected splash cycles.
    DripsSurfaceCrawler,
    /// Shooter block bullet: ASM `Process_ShooterObjects` (kid.asm:19394)
    /// emits a hostile 1x1 sprite from the shooter-object slot.
    ShooterBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ActiveProjectileSnapshot {
    pub x_pos: u32,
    pub y_pos: u32,
    pub friendly: bool,
    #[serde(default)]
    pub remaining_frames: u16,
    #[serde(default)]
    pub kind: ActiveProjectileKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireDemonTrailChild {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) remaining_frames: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KidFrameMotionBranch {
    #[default]
    None,
    DemoAutoTimeoutFrozen,
    DemoBootstrapPoseHold,
    DemoBootstrapCameraHandoff,
    DemoBootstrapInitialSettle,
    DemoBootstrapFreefall,
    DemoBootstrapGroundedHold,
    DemoBootstrapGroundedRun,
    IronKnightAttachHibernate,
    IronKnightClimbExitHibernate,
    IronKnightClimbResume,
    TransformHibernating,
    NormalPhysics,
}

impl KidFrameMotionBranch {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::DemoAutoTimeoutFrozen => "demo_auto_timeout_frozen",
            Self::DemoBootstrapPoseHold => "demo_bootstrap_pose_hold",
            Self::DemoBootstrapCameraHandoff => "demo_bootstrap_camera_handoff",
            Self::DemoBootstrapInitialSettle => "demo_bootstrap_initial_settle",
            Self::DemoBootstrapFreefall => "demo_bootstrap_freefall",
            Self::DemoBootstrapGroundedHold => "demo_bootstrap_grounded_hold",
            Self::DemoBootstrapGroundedRun => "demo_bootstrap_grounded_run",
            Self::IronKnightAttachHibernate => "iron_knight_attach_hibernate",
            Self::IronKnightClimbExitHibernate => "iron_knight_climb_exit_hibernate",
            Self::IronKnightClimbResume => "iron_knight_climb_resume",
            Self::TransformHibernating => "transform_hibernating",
            Self::NormalPhysics => "normal_physics",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum IronKnightClimbStep {
    #[default]
    None,
    AttachHibernate,
    ExitHibernate,
    ResumeClimb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KidFramePostMotionMutator {
    #[default]
    None,
    PlatformCollision,
    LiftPlatform,
    LevelTimerDeath,
    PitDeath,
    SpikeDeath,
    ProjectileDamage,
    MurderWallDamage,
    Teleporter,
    FlagTouch,
    StompBounce,
    ContactDamage,
    CycloneTornadoBounce,
}

impl KidFramePostMotionMutator {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PlatformCollision => "platform_collision",
            Self::LiftPlatform => "lift_platform",
            Self::LevelTimerDeath => "level_timer_death",
            Self::PitDeath => "pit_death",
            Self::SpikeDeath => "spike_death",
            Self::ProjectileDamage => "projectile_damage",
            Self::MurderWallDamage => "murder_wall_damage",
            Self::Teleporter => "teleporter",
            Self::FlagTouch => "flag_touch",
            Self::StompBounce => "stomp_bounce",
            Self::ContactDamage => "contact_damage",
            Self::CycloneTornadoBounce => "cyclone_tornado_bounce",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KidFramePrePhysicsMutator {
    #[default]
    None,
    FormWidthOverlapFixup,
    PendingStompBounce,
    SkycutterFlipboard,
    CycloneFlight,
    IronKnightClimb,
    MicromaxStick,
}

impl KidFramePrePhysicsMutator {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::FormWidthOverlapFixup => "form_width_overlap_fixup",
            Self::PendingStompBounce => "pending_stomp_bounce",
            Self::SkycutterFlipboard => "skycutter_flipboard",
            Self::CycloneFlight => "cyclone_flight",
            Self::IronKnightClimb => "iron_knight_climb",
            Self::MicromaxStick => "micromax_stick",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct KidGroundedReleaseProbe {
    pub(crate) on_ground_before: bool,
    pub(crate) held_direction: Option<Direction>,
    pub(crate) jump_pressed: bool,
    pub(crate) jump_buffer_frames: u8,
    pub(crate) custom_gravity: bool,
    pub(crate) wall_cling_direction: Option<Direction>,
    pub(crate) velocity_branch: KidGroundedReleaseVelocityBranch,
    pub(crate) probe_row: Option<u16>,
    pub(crate) left_col: Option<u16>,
    pub(crate) right_col: Option<u16>,
    pub(crate) support_col: Option<u16>,
    pub(crate) support_collision: Option<terrain::CollisionKind>,
    pub(crate) support_block_type: Option<terrain::BlockType>,
    pub(crate) release_triggered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KidWallClingProbeDirectionSource {
    WallCling,
    Held,
    Facing,
    #[default]
    None,
}

impl KidWallClingProbeDirectionSource {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::WallCling => "wall_cling",
            Self::Held => "held",
            Self::Facing => "facing",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KidWallClingReleaseBranch {
    SubAdb8,
    SubAe6e,
    PressAwayExit,
    #[default]
    NoWallClingState,
}

impl KidWallClingReleaseBranch {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::SubAdb8 => "sub_ADB8",
            Self::SubAe6e => "sub_AE6E",
            Self::PressAwayExit => "press_away_exit",
            Self::NoWallClingState => "no_wall_cling_state",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KidWallClingReleaseGate {
    Ready,
    #[default]
    WallClingMissing,
    StickyInactive,
    OnGround,
    Rising,
    NotAscending,
    HeldDirectionMismatch,
    NeedsTileCrossing,
    ForwardWallBlocked,
    TopEdgeBlocked,
}

impl KidWallClingReleaseGate {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::WallClingMissing => "wall_cling_missing",
            Self::StickyInactive => "sticky_inactive",
            Self::OnGround => "on_ground",
            Self::Rising => "rising",
            Self::NotAscending => "not_ascending",
            Self::HeldDirectionMismatch => "held_direction_mismatch",
            Self::NeedsTileCrossing => "needs_tile_crossing",
            Self::ForwardWallBlocked => "forward_wall_blocked",
            Self::TopEdgeBlocked => "top_edge_blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct KidWallClingReleaseProbe {
    pub(crate) held_direction: Option<Direction>,
    pub(crate) probe_direction: Option<Direction>,
    pub(crate) probe_direction_source: KidWallClingProbeDirectionSource,
    pub(crate) wall_cling_direction: Option<Direction>,
    pub(crate) right_wall_sticky: bool,
    pub(crate) left_wall_sticky: bool,
    pub(crate) on_ground: bool,
    pub(crate) y_velocity: i32,
    pub(crate) pre_vertical_y_px: i32,
    pub(crate) post_vertical_y_px: i32,
    pub(crate) branch: KidWallClingReleaseBranch,
    pub(crate) sub_adb8_gate: KidWallClingReleaseGate,
    pub(crate) sub_ae6e_gate: KidWallClingReleaseGate,
    pub(crate) forward_wall_probe: Option<terrain::WallProbe>,
    pub(crate) top_edge_probe: Option<terrain::WallProbe>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidCollisionHandoffProbe {
    pub(crate) player_lifecycle: PlayerLifecycle,
    pub(crate) previous_post_motion_mutator: KidFramePostMotionMutator,
    pub(crate) pending_stomp_bounce: bool,
    pub(crate) pending_form: Option<PlayerForm>,
    pub(crate) pending_level_skip: Option<LevelId>,
    pub(crate) form_change_flash_frames: u8,
    pub(crate) on_ground_before: bool,
    pub(crate) jump_buffer_frames: u8,
    pub(crate) wall_cling_direction: Option<Direction>,
    pub(crate) stomp_candidate: bool,
    pub(crate) stomp_enemy_object_type: Option<u8>,
    pub(crate) stomp_enemy_x_pos: Option<u32>,
    pub(crate) stomp_enemy_y_pos: Option<u32>,
    pub(crate) anchor_x_pos: u32,
    pub(crate) anchor_y_pos: u32,
}

impl Default for KidCollisionHandoffProbe {
    fn default() -> Self {
        Self {
            player_lifecycle: PlayerLifecycle::Alive,
            previous_post_motion_mutator: KidFramePostMotionMutator::None,
            pending_stomp_bounce: false,
            pending_form: None,
            pending_level_skip: None,
            form_change_flash_frames: 0,
            on_ground_before: false,
            jump_buffer_frames: 0,
            wall_cling_direction: None,
            stomp_candidate: false,
            stomp_enemy_object_type: None,
            stomp_enemy_x_pos: None,
            stomp_enemy_y_pos: None,
            anchor_x_pos: 0,
            anchor_y_pos: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct KidSameFrameStompProbe {
    pub(crate) stomp_candidate: bool,
    pub(crate) enemy_object_type: Option<u8>,
    pub(crate) enemy_x_pos: Option<u32>,
    pub(crate) enemy_y_pos: Option<u32>,
    pub(crate) pre_physics_y_velocity: i32,
    pub(crate) post_physics_y_velocity: i32,
    pub(crate) anchor_x_pos: u32,
    pub(crate) anchor_y_pos: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct KidIronKnightAttachProbe {
    pub(crate) c_pressed: bool,
    pub(crate) on_ground_before: bool,
    pub(crate) facing: Option<Direction>,
    pub(crate) climbing_before: bool,
    pub(crate) climbing_after: bool,
    pub(crate) y_velocity_before: i32,
    pub(crate) y_velocity_after: i32,
    pub(crate) wall_probe: Option<terrain::WallProbe>,
    pub(crate) candidate: bool,
    pub(crate) attach_triggered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KidFlipBranchSelection {
    #[default]
    NoWallPath,
    NoButtonBranch,
    BPressedTheKid,
    BPressedMicromax,
    BPressedGroundedLaunch,
    BPressedFallthrough,
    CPressedCyclone,
    CPressedManiaxe,
    CPressedSkycutter,
    CPressedFallthrough,
}

impl KidFlipBranchSelection {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::NoWallPath => "no_wall_path",
            Self::NoButtonBranch => "no_button_branch",
            Self::BPressedTheKid => "b_pressed_the_kid",
            Self::BPressedMicromax => "b_pressed_micromax",
            Self::BPressedGroundedLaunch => "b_pressed_grounded_launch",
            Self::BPressedFallthrough => "b_pressed_fallthrough",
            Self::CPressedCyclone => "c_pressed_cyclone",
            Self::CPressedManiaxe => "c_pressed_maniaxe",
            Self::CPressedSkycutter => "c_pressed_skycutter",
            Self::CPressedFallthrough => "c_pressed_fallthrough",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct KidFlipBranchProbe {
    pub(crate) wall_scan_hit: bool,
    pub(crate) b_pressed: bool,
    pub(crate) c_pressed: bool,
    pub(crate) helmet_id: u16,
    pub(crate) facing: Option<Direction>,
    pub(crate) on_ground_before: bool,
    pub(crate) on_ground_after: bool,
    pub(crate) branch: KidFlipBranchSelection,
    pub(crate) wall_probe: Option<terrain::WallProbe>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct KidFlipFollowOnStageState {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) on_ground: bool,
    pub(crate) wall_cling_direction: Option<Direction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct KidFlipFollowOnStages {
    pub(crate) entry: KidFlipFollowOnStageState,
    pub(crate) after_press_away: KidFlipFollowOnStageState,
    pub(crate) after_jump_gravity: KidFlipFollowOnStageState,
    pub(crate) after_floor: KidFlipFollowOnStageState,
    pub(crate) after_wall: KidFlipFollowOnStageState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct KidFlipFollowOnProbe {
    pub(crate) branch: KidFlipBranchSelection,
    pub(crate) motion_branch: KidFrameMotionBranch,
    pub(crate) post_motion_mutator: KidFramePostMotionMutator,
    pub(crate) stages: KidFlipFollowOnStages,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KidJumpGravityBranch {
    JumpPressed,
    JumpBuffered,
    AirborneJustJumpedHold,
    AirborneCustomGravity,
    AirborneHeldJumpGravity,
    AirborneDefaultGravity,
    #[default]
    GroundedHold,
}

impl KidJumpGravityBranch {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::JumpPressed => "jump_pressed",
            Self::JumpBuffered => "jump_buffered",
            Self::AirborneJustJumpedHold => "airborne_just_jumped_hold",
            Self::AirborneCustomGravity => "airborne_custom_gravity",
            Self::AirborneHeldJumpGravity => "airborne_held_jump_gravity",
            Self::AirborneDefaultGravity => "airborne_default_gravity",
            Self::GroundedHold => "grounded_hold",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct KidJumpGravityProbe {
    pub(crate) branch: KidJumpGravityBranch,
    pub(crate) jump_pressed: bool,
    pub(crate) input_b_held: bool,
    pub(crate) custom_gravity: bool,
    pub(crate) on_ground_before: bool,
    pub(crate) on_ground_after: bool,
    pub(crate) just_jumped_before: bool,
    pub(crate) just_jumped_after: bool,
    pub(crate) jump_buffer_frames_before: u8,
    pub(crate) jump_buffer_frames_after: u8,
    pub(crate) y_velocity_before: i32,
    pub(crate) y_velocity_after: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KidGroundedReleaseVelocityBranch {
    JumpPressed,
    JumpBuffered,
    AirborneCustomGravity,
    AirborneGravity,
    #[default]
    GroundedHold,
}

impl KidGroundedReleaseVelocityBranch {
    pub(crate) fn as_debug_label(self) -> &'static str {
        match self {
            Self::JumpPressed => "jump_pressed",
            Self::JumpBuffered => "jump_buffered",
            Self::AirborneCustomGravity => "airborne_custom_gravity",
            Self::AirborneGravity => "airborne_gravity",
            Self::GroundedHold => "grounded_hold",
        }
    }
}

pub const HEADYMETAL_DEATH_EFFECT_FRAMES: u16 = 0x01C2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnProbeInspection {
    pub kid_x_pos: u32,
    pub kid_y_pos: u32,
    pub hitbox_probe: terrain::FloorProbe,
    pub span_probe: terrain::FloorProbe,
}

/// Block interaction state: evanescent, ghost, shooter, and lift blocks.
#[derive(Debug, Clone)]
pub(crate) struct BlockInteractionState {
    pub(crate) evanescent_blocks: Vec<EvanescentEntry>,
    pub(crate) ghost_groups: Vec<GhostGroup>,
    pub(crate) lift_platforms: Vec<LiftPlatform>,
    /// ASM loc_EB72 -> LoadBulletsFromEvanescentBlock: visible shooter blocks
    /// register once, fire immediately, then keep their slot for 22 frames.
    pub(crate) shooter_blocks: Vec<ShooterBlockEntry>,
    /// ASM loc_11430 / sub_1149C / sub_114C0 / sub_114F4: shooter blocks
    /// hit from below queue transient deleted-tile workers for the struck cell
    /// and any empty support cells above it.
    pub(crate) shooter_headbutt_workers: Vec<ShooterHeadbuttWorker>,
    /// ASM sub_F06A resets `$FFFFF8BE/$FFFFF8C0` to -1 so sub_F096 can detect
    /// the next shooter scan line entering Kid's local block window.
    pub(crate) shooter_last_scanned_col: i16,
    pub(crate) shooter_last_scanned_row: i16,
    /// Prize blocks headbutted from below: ASM sub_1007A owns the full
    /// lifecycle — keep the cell Solid (0x6F), run the bounce coroutine,
    /// then clear the tile and spawn the prize object.
    pub(crate) deferred_ceiling_clears: Vec<DeferredCeilingClear>,
    /// ASM sub_10DE4: rubber-block contact toggles the same cell between two
    /// skins with 2/1/1/1/2/2/3/3 frame hibernate gaps.
    pub(crate) rubber_block_animations: Vec<RubberBlockAnimation>,
}

impl Default for BlockInteractionState {
    fn default() -> Self {
        Self {
            evanescent_blocks: Vec::new(),
            ghost_groups: Vec::new(),
            lift_platforms: Vec::new(),
            shooter_blocks: Vec::new(),
            shooter_headbutt_workers: Vec::new(),
            shooter_last_scanned_col: -1,
            shooter_last_scanned_row: -1,
            deferred_ceiling_clears: Vec::new(),
            rubber_block_animations: Vec::new(),
        }
    }
}

impl BlockInteractionState {
    #[allow(non_snake_case)]
    pub(crate) fn reset_shooter_state_sub_F06A(&mut self) {
        self.shooter_blocks.clear();
        self.shooter_headbutt_workers.clear();
        self.shooter_last_scanned_col = -1;
        self.shooter_last_scanned_row = -1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterBlockEntry {
    pub(crate) col: u16,
    pub(crate) row: u16,
    pub(crate) directions: u8,
    pub(crate) supported_by_neighbour: bool,
    pub(crate) age: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShooterHeadbuttWorkerKind {
    Center,
    Support,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterHeadbuttWorker {
    pub(crate) col: u16,
    pub(crate) row: u16,
    pub(crate) age: u8,
    pub(crate) kind: ShooterHeadbuttWorkerKind,
}

/// Weather system state for the current level.
#[derive(Debug, Clone, Default)]
pub(crate) struct WeatherState {
    /// Weather type (0=none, 1=lava, 2=storm, 3=storm+hail).
    pub(crate) weather_type: u8,
    /// Hail system: frames until next hail spawn (0=spawn this frame).
    pub(crate) hail_spawn_timer: u16,
    /// Hail system: whether hail is currently active (toggles on $258-frame period).
    pub(crate) hail_active: bool,
    /// Hail system: frames until next active/inactive toggle.
    pub(crate) hail_period_timer: u16,
    /// Active lava geysers for lava weather levels.
    pub(crate) lava_geysers: Vec<LavaGeyser>,
    /// Active small fireballs from the lava special-effect worker.
    pub(crate) lava_fireballs: Vec<LavaFireball>,
}

/// Per-helmet ability state, reset on form change or death.
#[derive(Debug, Clone, Default)]
pub(crate) struct HelmetState {
    /// Skycutter flipboard deploy counter (0=inactive, 1..31=animating).
    pub(crate) skycutter_flipboard: u8,
    pub(crate) skycutter_on_board: bool,
    pub(crate) cyclone_flying: bool,
    pub(crate) cyclone_y_accel: i16,
    pub(crate) red_stealth_sword_timer: u8,
    /// ASM loc_B350/loc_B380 (kid.asm:14220-14248) per-frame counter for the
    /// Red Stealth airborne attack-down slash. Increments by `$10` each frame
    /// while airborne+fast-falling+C-held, caps at `$7F`. Selects one of four
    /// sprite phases via `(counter>>5)<<2` indexing `LnkTo_unk_B40DE`. The
    /// ROM stores this in the same word as `red_stealth_sword_timer` but
    /// disambiguates via state transitions (sub_DB22 clears on jumps);
    /// native keeps a dedicated field so the ground swing timer's
    /// 12→0 semantics never overlap with this counter's 0→$7F semantics.
    pub(crate) red_stealth_airborne_slash_counter: u8,
    pub(crate) eyeclops_beam_active: bool,
    pub(crate) eyeclops_beam_counter: u16,
    pub(crate) eyeclops_beam_hard: bool,
    pub(crate) maniaxe_axe_active: bool,
    pub(crate) maniaxe_turnaround: u8,
    pub(crate) juggernaut_skull_delay_frames: u8,
    pub(crate) iron_knight_climbing: bool,
    pub(crate) iron_knight_climb_accel: i16,
    pub(crate) iron_knight_climb_pose_accum: u16,
    pub(crate) berzerker_charge_accum: u8,
    pub(crate) berzerker_charging: bool,
    pub(crate) berzerker_charge_pose: bool,
    pub(crate) micromax_stuck: bool,
}

impl HelmetState {
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Player progression: form, lives, HP, diamonds, score, and combat state.
#[derive(Debug, Clone)]
pub(crate) struct PlayerProgression {
    pub(crate) current_form: PlayerForm,
    pub(crate) pending_form: Option<PlayerForm>,
    pub(crate) number_lives: u16,
    pub(crate) number_continues: u8,
    pub(crate) number_hitpoints: u16,
    pub(crate) extra_hitpoint_slots: u16,
    pub(crate) invulnerable_frames: usize,
    /// Visual flash countdown after form change (0=inactive).
    pub(crate) form_change_flash_frames: u8,
    pub(crate) diamond_count: u16,
    pub(crate) score: u32,
    /// Pending half-width change from form transform.
    /// ASM: during Kid_Transform, the new half-width is set via sub_B41C,
    /// and the Hibernate frame resolves any wall overlap. Non-zero = need fixup.
    pub(crate) form_width_delta: i32,
    /// Diamond Power active: grants invulnerability and clears nearby enemies.
    pub(crate) diamond_power_frames: u16,
    /// Which Diamond Power type is currently active (determines behaviour).
    pub(crate) diamond_power_type: super::helmets::DiamondPowerType,
}

impl Default for PlayerProgression {
    fn default() -> Self {
        Self {
            current_form: PlayerForm::TheKid,
            pending_form: None,
            number_lives: 3,
            number_continues: 3,
            number_hitpoints: PlayerForm::TheKid.max_hitpoints(),
            extra_hitpoint_slots: 0,
            invulnerable_frames: 0,
            form_change_flash_frames: 0,
            diamond_count: 0,
            score: 0,
            diamond_power_frames: 0,
            diamond_power_type: super::helmets::DiamondPowerType::None,
            form_width_delta: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    SegaScreen,
    IntroSequence1,
    IntroSequence2,
    IntroSequence3,
    IntroSequence4,
    IntroSequence5,
    IntroSequence6,
    TitleScreen,
    OptionsMenu,
    LevelTransition,
    InGame,
    /// ASM game mode 0x10 — title screen demo playback.
    DemoPlay,
    /// ASM game mode 0x30 — end credits sequence after beating the final level.
    EndingSequence,
}

impl GameMode {
    pub fn as_disassembly_value(self) -> u16 {
        match self {
            Self::SegaScreen => 0x0000,
            Self::IntroSequence1 => 0x0004,
            Self::IntroSequence2 => 0x0018,
            Self::IntroSequence3 => 0x001C,
            Self::IntroSequence4 => 0x0020,
            Self::IntroSequence5 => 0x0024,
            Self::IntroSequence6 => 0x0028,
            Self::TitleScreen => 0x002C,
            Self::OptionsMenu => 0x0014,
            Self::LevelTransition => 0x0008,
            Self::InGame => 0x000C,
            Self::DemoPlay => 0x0010,
            Self::EndingSequence => 0x0030,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerForm {
    TheKid,
    Skycutter,
    Cyclone,
    RedStealth,
    Eyeclops,
    Juggernaut,
    IronKnight,
    Berzerker,
    Maniaxe,
    Micromax,
}

impl PlayerForm {
    pub fn helmet_id(self) -> u16 {
        match self {
            Self::TheKid => 0,
            Self::Skycutter => 1,
            Self::Cyclone => 2,
            Self::RedStealth => 3,
            Self::Eyeclops => 4,
            Self::Juggernaut => 5,
            Self::IronKnight => 6,
            Self::Berzerker => 7,
            Self::Maniaxe => 8,
            Self::Micromax => 9,
        }
    }

    /// Per-form collision half-width from ASM unk_B408 table (kid.asm:14307).
    /// Called via sub_B41C at level start (loc_757E, kid.asm:7689) and during
    /// helmet transform (sub_7ACC, kid.asm:8188).
    /// Used by sub_8F26 (wall collision) and GfxObjects_Collision.
    ///
    /// Also used as the crawling floor-scan half-width override at
    /// ASM loc_B482 (kid.asm:14366): `moveq #7,d1` (default), TheKid→5,
    /// ASM loc_B494 (kid.asm:14374): Skycutter→10 (`moveq #$A,d1`).
    pub(crate) fn half_width(self) -> i32 {
        match self {
            Self::TheKid => 5,
            Self::Skycutter => 10,
            Self::Juggernaut => 23,
            Self::Micromax => 4,
            Self::Cyclone
            | Self::RedStealth
            | Self::Eyeclops
            | Self::IronKnight
            | Self::Berzerker
            | Self::Maniaxe => 7,
        }
    }

    /// Ceiling-collision height from ASM sub_902A/sub_DAA6.
    /// Most forms use a 31px head probe; Juggernaut and Micromax use 15px.
    pub(crate) fn ceiling_probe_height(self) -> i32 {
        match self {
            Self::Juggernaut | Self::Micromax => 15,
            _ => 31,
        }
    }

    pub fn max_hitpoints(self) -> u16 {
        match self {
            Self::TheKid => 2,
            Self::IronKnight => 5,
            Self::Skycutter
            | Self::Cyclone
            | Self::RedStealth
            | Self::Eyeclops
            | Self::Juggernaut
            | Self::Berzerker
            | Self::Maniaxe
            | Self::Micromax => 3,
        }
    }

    /// Select the ground speed table for this form and terrain type.
    /// ASM loc_9AE0 (kid.asm:11673): sub_9A0A enters the generic ground
    /// table path by loading the normal table, then swapping in ice/rubber
    /// tables before applying Micromax/Juggernaut form offsets.
    /// ASM loc_9AF2 (kid.asm:11680): terrain type 1 replaces the current
    /// table pointer with the rubber table (`unk_9CCE`).
    /// ASM loc_9AFC (kid.asm:11685): Micromax advances the selected terrain
    /// table pointer by `$1C`, one `SpeedTable` entry.
    /// ASM loc_9B08 (kid.asm:11690): Juggernaut advances the selected terrain
    /// table pointer by `$38`, two `SpeedTable` entries.
    /// Three form groups (Kid/Micromax/Juggernaut) × three terrain types (Normal/Ice/Rubber).
    pub(crate) fn ground_speed_table(
        self,
        terrain_type: TerrainType,
    ) -> &'static crate::physics::SpeedTable {
        match (self, terrain_type) {
            (Self::Micromax, TerrainType::Normal) => &crate::physics::NORMAL_MICROMAX,
            (Self::Micromax, TerrainType::Ice) => &crate::physics::ICE_MICROMAX,
            (Self::Micromax, TerrainType::Rubber) => &crate::physics::RUBBER_MICROMAX,
            (Self::Juggernaut, TerrainType::Normal) => &crate::physics::NORMAL_JUGGERNAUT,
            (Self::Juggernaut, TerrainType::Ice) => &crate::physics::ICE_JUGGERNAUT,
            (Self::Juggernaut, TerrainType::Rubber) => &crate::physics::RUBBER_JUGGERNAUT,
            (_, TerrainType::Normal) => &crate::physics::NORMAL_KID,
            (_, TerrainType::Ice) => &crate::physics::ICE_KID,
            (_, TerrainType::Rubber) => &crate::physics::RUBBER_KID,
        }
    }

    pub fn from_helmet_id(helmet_id: u16) -> Option<Self> {
        match helmet_id {
            0 => Some(Self::TheKid),
            1 => Some(Self::Skycutter),
            2 => Some(Self::Cyclone),
            3 => Some(Self::RedStealth),
            4 => Some(Self::Eyeclops),
            5 => Some(Self::Juggernaut),
            6 => Some(Self::IronKnight),
            7 => Some(Self::Berzerker),
            8 => Some(Self::Maniaxe),
            9 => Some(Self::Micromax),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FrameInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub c: bool,
    pub start: bool,
}

impl FrameInput {
    /// Decode a raw controller byte into individual button flags.
    ///
    /// ASM loc_7464 (kid.asm:7593–7609): the ROM unpacks `Ctrl_Held` into
    /// seven consecutive flag bytes (`Ctrl_A_Held` .. `Ctrl_Up_Held`) via
    /// repeated `add.b d0,d0; scs (a0)+` — each left-shift moves the next
    /// bit into carry, and `scs` sets the target byte to $FF when carry is
    /// set.  The first shift discards bit 7 (Start, handled separately by
    /// loc_74B0's `bclr`).  The final `sne (a0)` sets `Ctrl_Up_Held` from
    /// the Z flag of the last shift result.
    ///
    /// Native uses direct bit-mask tests instead of the serial shift chain,
    /// producing the same boolean result per button.
    pub fn from_mask(mask: u8) -> Self {
        Self {
            up: mask & (1 << 0) != 0,
            down: mask & (1 << 1) != 0,
            left: mask & (1 << 2) != 0,
            right: mask & (1 << 3) != 0,
            b: mask & (1 << 4) != 0,
            c: mask & (1 << 5) != 0,
            a: mask & (1 << 6) != 0,
            start: mask & (1 << 7) != 0,
        }
    }

    /// ASM loc_7464 (kid.asm:7610–7614): after the shift chain, the ROM
    /// checks `tst.b (Ctrl_Left_Held)` + `tst.b (Ctrl_Right_Held)` — when
    /// both are held simultaneously, `sf (Ctrl_Right_Held)` clears right so
    /// left wins.  Returns a new `FrameInput` with the conflict resolved.
    #[must_use]
    pub fn with_left_right_conflict_resolved(self) -> Self {
        if self.left && self.right {
            Self {
                right: false,
                ..self
            }
        } else {
            self
        }
    }

    pub fn to_mask(&self) -> u8 {
        let mut mask = 0u8;
        if self.up {
            mask |= 1 << 0;
        }
        if self.down {
            mask |= 1 << 1;
        }
        if self.left {
            mask |= 1 << 2;
        }
        if self.right {
            mask |= 1 << 3;
        }
        if self.b {
            mask |= 1 << 4;
        }
        if self.c {
            mask |= 1 << 5;
        }
        if self.a {
            mask |= 1 << 6;
        }
        if self.start {
            mask |= 1 << 7;
        }
        mask
    }

    /// ASM loc_749C (kid.asm:7616–7621): when `Demo_Mode_flag` is clear and
    /// `Options_Suboption_Speed` is non-zero (fast mode), the ROM inverts
    /// `Ctrl_A_Held` with `not.b`.  This makes the player run by default and
    /// hold A to walk — a speed option convenience toggle.
    ///
    /// Returns a new `FrameInput` with the A button inverted.
    #[must_use]
    pub fn with_speed_option_a_inverted(self) -> Self {
        Self { a: !self.a, ..self }
    }

    /// ASM Permute_ABCButtons (sub_CCE, kid.asm:1458): remaps A/B/C buttons
    /// based on the player's control scheme setting (`Options_Suboption_Controls`).
    /// Scheme 0 = default (no permutation, early return at `return_D7C`).
    /// Schemes 1–5 are dispatched via a `jmp loc_CE8(pc,d7.w)` branch table
    /// (ASM loc_CE8, kid.asm:1468) — five `bra.s` entries, one per scheme.
    ///
    /// Each scheme extracts A/B/C bits from `d0` via `bclr`, accumulates the
    /// remapped bits into `d6`, then merges with `or.b d6,d0` at `loc_D7A`.
    /// The routine also gates on `Demo_Mode_flag` — in demo mode it returns
    /// immediately (native: caller gates on demo mode).
    ///
    /// Bit layout (from `from_mask`): bit 4 = B, bit 5 = C, bit 6 = A.
    ///
    /// | Scheme | ASM entry | Mapping              |
    /// |--------|-----------|----------------------|
    /// | 1      | loc_CF2   | B↔C (bit4↔bit5)     |
    /// | 2      | loc_D0A   | A↔B (bit6↔bit4)     |
    /// | 3      | loc_D22   | A→B, B→C, C→A       |
    /// | 4      | loc_D44   | A→C, B→A, C→B       |
    /// | 5      | loc_D66   | A↔C (bit6↔bit5)     |
    ///
    /// Returns `self` unchanged for scheme 0 or any out-of-range value.
    #[must_use]
    pub fn with_abc_buttons_permuted(self, scheme: u8) -> Self {
        match scheme {
            // ASM loc_CF2 (kid.asm:1480): `bclr #4,d0 → bset #5,d6` then
            // loc_CFC (kid.asm:1485): `bclr #5,d0 → bset #4,d6` — swap B↔C.
            1 => Self {
                b: self.c,
                c: self.b,
                ..self
            },
            // ASM loc_D0A (kid.asm:1492): `bclr #6,d0; bset #4,d6` maps A→B.
            // ASM loc_D14 (kid.asm:1497): `bclr #4,d0; bset #6,d6` maps B→A.
            2 => Self {
                a: self.b,
                b: self.a,
                ..self
            },
            // ASM loc_D22 (kid.asm:1504): A→B, then loc_D2C (kid.asm:1509)
            // maps B→C before loc_D36 maps C→A (3-way rotate).
            3 => Self {
                a: self.c,
                b: self.a,
                c: self.b,
                ..self
            },
            // ASM loc_D44 (kid.asm:1521): `bclr #6,d0; bset #5,d6` maps A→C.
            // ASM loc_D4E (kid.asm:1526): `bclr #4,d0; bset #6,d6` maps B→A.
            // ASM loc_D58 (kid.asm:1531): `bclr #5,d0; bset #4,d6` maps C→B.
            4 => Self {
                a: self.b,
                b: self.c,
                c: self.a,
                ..self
            },
            // ASM loc_D66 (kid.asm:1538): `bclr #6,d0; bset #5,d6` maps A→C.
            // ASM loc_D70 (kid.asm:1543): `bclr #5,d0; bset #6,d6` maps C→A.
            5 => Self {
                a: self.c,
                c: self.a,
                ..self
            },
            // Scheme 0 or invalid — no permutation (ASM: `beq.w return_D7C`).
            _ => self,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NativeTraceFrame {
    pub frame_index: usize,
    pub game_mode: GameMode,
    pub game_mode_value: u16,
    pub start_press_count: usize,
    pub boot_phase: Option<String>,
    pub boot_phase_elapsed: Option<usize>,
    pub title_screen_ready: bool,
    pub current_player: u8,
    pub current_level: LevelId,
    pub player_1_level_id: LevelId,
    pub current_level_name: Option<String>,
    pub player_start_x: Option<u16>,
    pub player_start_y: Option<u16>,
    pub flag_x: Option<u16>,
    pub flag_y: Option<u16>,
    pub level_enemy_object_count: Option<u8>,
    pub active_enemy_object_count: usize,
    pub active_platform_object_count: usize,
    pub number_lives: u16,
    pub number_continues: u8,
    pub number_hitpoints: u16,
    /// ROM `Extra_hitpoint_slots` mirror. `Make_SpriteAttr_HUD` adds this to
    /// the helmet base HP before drawing the grey empty HUD pips.
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub extra_hitpoint_slots: u16,
    pub current_helmet_id: u16,
    /// ROM `Red_Stealth_sword_swing` mirror for native-player rendering.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub red_stealth_sword_timer: u8,
    /// ROM `Red_Stealth_sword_swing` (airborne path) — ASM `sub_B270`
    /// loc_B350/loc_B380 counter (0→$7F) that selects one of four downward
    /// slash sprite phases via `(counter>>5)`. Separate from the ground
    /// `red_stealth_sword_timer` so the renderer can distinguish.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub red_stealth_airborne_slash_counter: u8,
    pub paused: bool,
    pub kid_x_pos: Option<u32>,
    pub kid_y_pos: Option<u32>,
    pub kid_x_velocity: Option<i32>,
    pub kid_y_velocity: Option<i32>,
    /// ROM `Character_Movement` mirror (`$FFFFFA56`).
    pub character_movement: Option<u16>,
    pub kid_facing: Option<Direction>,
    pub kid_on_ground: Option<bool>,
    /// ROM `$FFFFF8F0`: visual walk/crawl/slope animation phase.
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub walk_visual_accumulator: u16,
    /// ROM `Berzerker_charging` (`$FFFFFA6E`): set by ASM loc_9564 after the
    /// Berzerker/Cyclone walk accumulator carries.
    #[serde(default)]
    pub berzerker_charging: bool,
    /// Render latch for the sprite written by ASM loc_9564 before later
    /// collision code can clear `Berzerker_charging` at loc_9326.
    #[serde(default)]
    pub berzerker_charge_pose: bool,
    /// Remaining frames in ASM `stru_8BB4` wall-grab pull-up animation.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub ledge_snap_cooldown: u8,
    /// Slope direction the player is standing on: 0 = flat, 1 = SlopeUp, -1 = SlopeDown.
    /// ASM sub_9386 / $FFFFFA26: used by renderer for per-column sprite tilt.
    #[serde(default)]
    pub kid_on_slope: i8,
    pub active_objects: Vec<NativeObjectSnapshot>,
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
    pub number_objects: u16,
    pub number_gfx_objects: u16,
    pub rng_offset: Option<u16>,
    pub level_complete: bool,
    /// Elapsed frames in the current collapsed end-of-level sequence.
    /// Used by native rendering to replay ASM `loc_D770` score-counter
    /// visuals without changing the already-awarded progression score.
    #[serde(default)]
    pub level_complete_elapsed_frames: usize,
    /// Elapsed frames since ASM `loc_D468` allocated the EOL score rows.
    #[serde(default)]
    pub level_complete_score_rows_elapsed_frames: usize,
    /// Elapsed frames inside ASM `loc_D770` score-counter drain.
    #[serde(default)]
    pub level_complete_score_counter_elapsed_frames: usize,
    /// ASM loc_D17E copied score value preserved for EOL score rendering.
    #[serde(default)]
    pub level_complete_initial_score: u32,
    /// Non-zero once ASM `loc_D468` score-screen row setup has been reached.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub level_complete_score_rows_started: u8,
    /// ASM loc_C0CC computed `Level_completion_time`, capped at `$FF`.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub level_completion_time: u8,
    /// Non-zero when ASM `LevelSkip_Cheat` remains armed for the EOL digit path.
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub level_skip_cheat_enabled: u8,
    pub game_over: bool,
    /// ASM loc_D25E no-continues / pre-continue GAME OVER visual elapsed frames.
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub game_over_elapsed_frames: usize,
    /// Continue countdown remaining frames (None if not in continue screen).
    pub continue_countdown: Option<usize>,
    /// ASM Continue_Screen visual elapsed frames (0 outside continue screen).
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub continue_countdown_elapsed_frames: usize,
    pub game_complete: bool,
    /// Ending credits sequence elapsed frames (None if not in ending).
    pub ending_elapsed: Option<usize>,
    pub options_cursor: Option<u8>,
    pub sound_test_track: Option<u8>,
    /// Cumulative GEMS sound ids queued through ASM-backed native core paths.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub played_sound_effect_ids: Vec<u8>,
    pub two_player: bool,
    pub options_two_player_two_controllers: bool,
    pub options_controls_scheme: u8,
    pub options_speed_button_fast: bool,
    pub invulnerable_frames: usize,
    /// Exact per-frame damage latch, mirroring ROM `Just_received_damage`.
    pub just_received_damage: bool,
    pub diamond_power_active: bool,
    pub diamond_count: u16,
    pub score: u32,
    pub level_timer_seconds: u16,
    pub level_damage_taken: bool,
    /// Whether the player collected the level's prize block. Determines
    /// whether the ASM loc_D5D2 NO-PRIZE bonus is awarded at level
    /// complete (5000 points when this is `false`).
    #[serde(default)]
    pub level_prize_collected: bool,
    /// Whether `apply_level_completion_bonuses` awarded the ASM loc_D660
    /// speed bonus (10,000 points) for the current level-complete
    /// sequence. The renderer uses this to decide whether to show the
    /// SPEED BONUS line in the level-complete overlay; the previous
    /// overlay used a hand-tuned `elapsed < 60` threshold that did not
    /// match the engine's per-level `SPEED_BONUS_TABLE` condition.
    #[serde(default)]
    pub level_speed_bonus_awarded: bool,
    /// Amount added by `apply_level_completion_bonuses` for the ASM
    /// loc_D60E path bonus (`PathBonus_Values[level] × 1000`). Zero
    /// when the level has no path bonus.
    #[serde(default)]
    pub level_path_bonus_amount: u32,
    pub murder_wall_x: Option<i64>,
    /// Active projectiles: (x_pos, y_pos, friendly).
    pub active_projectiles: Vec<(u32, u32, bool)>,
    /// Active projectile details for the native player renderer. This stays
    /// additive so older recordings that only serialised `active_projectiles`
    /// still load cleanly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_projectile_details: Vec<ActiveProjectileSnapshot>,
    /// Active lava geyser eruptions: (x_px, y_top_px, height_px) for rendering.
    pub active_geysers: Vec<(u16, u16, u16)>,
    /// Metal block bounce animations: (cell_x, cell_y, remaining_frames, y_offset_px).
    #[serde(default, deserialize_with = "deserialize_block_bounces")]
    pub block_bounces: Vec<(u16, u16, u8, i16)>,
    /// Block destruction fragment effects: (x_px, y_px, remaining_frames, direction).
    #[serde(default, deserialize_with = "deserialize_block_destroy_effects")]
    pub block_destroy_effects: Vec<(u16, u16, u8, u16)>,
    /// Falling block debris shards from ASM `sub_2A4C`: (x_px, y_px).
    #[serde(default)]
    pub block_debris_pieces: Vec<(i16, i16)>,
    /// Prize-block spawn flash effects: (x_px, y_px, remaining_frames).
    #[serde(default)]
    pub prize_block_spawn_flashes: Vec<(u16, u16, u8)>,
    /// Floating prize items rising from blocks: (x_px, y_px, remaining_frames, content_id).
    pub floating_items: Vec<(u16, u16, u8, u8)>,
    /// Enemy death particle effects: (x_pos_fp, y_pos_fp, remaining_frames).
    pub enemy_death_effects: Vec<(u32, u32, u16)>,
    /// Death windup elapsed frames (0=not dying). Used by renderer for tumble animation.
    #[serde(default)]
    pub death_windup_elapsed: u16,
    /// Form-change visual flash countdown (0=no flash).
    pub form_change_flash_frames: u8,
    /// True if a stomp occurred this frame (for SFX).
    pub stomped_this_frame: bool,
    /// Number of blocks hit by headbutt this frame (for SFX).
    pub blocks_hit_this_frame: u8,
    /// Sound test: play was requested this frame (A/C pressed).
    pub sound_test_play_pending: bool,
    /// Frontier-specific frame debug fields. Start exploratory here; promote
    /// recurring decision-making fields into typed members.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub gameplay_debug_fields: BTreeMap<String, String>,
    /// Sub-frame physics trace (populated when `trace_physics` is enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physics_trace: Option<PhysicsTrace>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BlockBounceTraceEntry {
    Current(BlockBounceTraceTuple),
    Legacy((u16, u16, u8)),
}

type BlockBounceTraceTuple = (u16, u16, u8, i16);

fn deserialize_block_bounces<'de, D>(
    deserializer: D,
) -> Result<Vec<BlockBounceTraceTuple>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<BlockBounceTraceEntry>::deserialize(deserializer).map(|entries| {
        entries
            .into_iter()
            .map(|entry| match entry {
                BlockBounceTraceEntry::Current(entry) => entry,
                BlockBounceTraceEntry::Legacy((cell_x, cell_y, remaining)) => {
                    (cell_x, cell_y, remaining, 0)
                }
            })
            .collect()
    })
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BlockDestroyTraceEntry {
    Current(BlockDestroyTraceTuple),
    Legacy((u16, u16, u8)),
}

type BlockDestroyTraceTuple = (u16, u16, u8, u16);

fn deserialize_block_destroy_effects<'de, D>(
    deserializer: D,
) -> Result<Vec<BlockDestroyTraceTuple>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<BlockDestroyTraceEntry>::deserialize(deserializer).map(|entries| {
        entries
            .into_iter()
            .map(|entry| match entry {
                BlockDestroyTraceEntry::Current(entry) => entry,
                // Legacy native-player recordings predate the ASM `sub_292E`
                // direction byte. Use the upward branch (`d6=2`) because the
                // earliest traced effects came from floor-smash paths.
                BlockDestroyTraceEntry::Legacy((x_px, y_px, remaining)) => {
                    (x_px, y_px, remaining, 2)
                }
            })
            .collect()
    })
}

impl Default for NativeTraceFrame {
    fn default() -> Self {
        Self {
            frame_index: 0,
            game_mode: GameMode::SegaScreen,
            game_mode_value: GameMode::SegaScreen.as_disassembly_value(),
            start_press_count: 0,
            boot_phase: None,
            boot_phase_elapsed: None,
            title_screen_ready: false,
            current_player: 0,
            current_level: LevelId(0),
            player_1_level_id: LevelId(0),
            current_level_name: None,
            player_start_x: None,
            player_start_y: None,
            flag_x: None,
            flag_y: None,
            level_enemy_object_count: None,
            active_enemy_object_count: 0,
            active_platform_object_count: 0,
            number_lives: 0,
            number_continues: 0,
            number_hitpoints: 0,
            extra_hitpoint_slots: 0,
            current_helmet_id: 0,
            red_stealth_sword_timer: 0,
            red_stealth_airborne_slash_counter: 0,
            paused: false,
            kid_x_pos: None,
            kid_y_pos: None,
            kid_x_velocity: None,
            kid_y_velocity: None,
            character_movement: None,
            kid_facing: None,
            kid_on_ground: None,
            walk_visual_accumulator: 0,
            berzerker_charging: false,
            berzerker_charge_pose: false,
            ledge_snap_cooldown: 0,
            kid_on_slope: 0,
            active_objects: Vec::new(),
            camera_x_pos: 0,
            camera_y_pos: 0,
            number_objects: 0,
            number_gfx_objects: 0,
            rng_offset: None,
            level_complete: false,
            level_complete_elapsed_frames: 0,
            level_complete_score_rows_elapsed_frames: 0,
            level_complete_score_counter_elapsed_frames: 0,
            level_complete_initial_score: 0,
            level_complete_score_rows_started: 0,
            level_completion_time: 0,
            level_skip_cheat_enabled: 0,
            game_over: false,
            game_over_elapsed_frames: 0,
            continue_countdown: None,
            continue_countdown_elapsed_frames: 0,
            game_complete: false,
            ending_elapsed: None,
            options_cursor: None,
            sound_test_track: None,
            played_sound_effect_ids: Vec::new(),
            two_player: false,
            options_two_player_two_controllers: false,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
            invulnerable_frames: 0,
            just_received_damage: false,
            diamond_power_active: false,
            diamond_count: 0,
            score: 0,
            level_timer_seconds: 0,
            level_damage_taken: false,
            level_prize_collected: false,
            level_speed_bonus_awarded: false,
            level_path_bonus_amount: 0,
            murder_wall_x: None,
            active_projectiles: Vec::new(),
            active_projectile_details: Vec::new(),
            active_geysers: Vec::new(),
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
            block_debris_pieces: Vec::new(),
            prize_block_spawn_flashes: Vec::new(),
            floating_items: Vec::new(),
            enemy_death_effects: Vec::new(),
            death_windup_elapsed: 0,
            form_change_flash_frames: 0,
            stomped_this_frame: false,
            blocks_hit_this_frame: 0,
            sound_test_play_pending: false,
            gameplay_debug_fields: BTreeMap::new(),
            physics_trace: None,
        }
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero_u8(value: &u8) -> bool {
    *value == 0
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero_u16(value: &u16) -> bool {
    *value == 0
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero_usize(value: &usize) -> bool {
    *value == 0
}

impl NativeTraceFrame {
    /// CRC32 of authoritative fields — used for fast divergence detection.
    /// Excludes advisory fields (number_objects, number_hitpoints, number_lives, murder_wall_x).
    pub fn authoritative_crc32(&self) -> u32 {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&self.game_mode_value.to_le_bytes());
        hasher.update(&(self.current_level.0 as u16).to_le_bytes());
        hasher.update(&self.camera_x_pos.to_le_bytes());
        hasher.update(&self.camera_y_pos.to_le_bytes());
        hasher.update(&self.kid_x_pos.unwrap_or(0).to_le_bytes());
        hasher.update(&self.kid_y_pos.unwrap_or(0).to_le_bytes());
        // Velocity excluded: capture-timing differs between native (post-frame)
        // and oracle (VBlank), making velocity a lagging indicator. Positions
        // are the authoritative ground truth for gameplay parity.
        hasher.update(&self.current_helmet_id.to_le_bytes());
        hasher.update(&self.score.to_le_bytes());
        hasher.update(&self.diamond_count.to_le_bytes());
        hasher.update(&[self.diamond_power_active as u8]);
        hasher.update(&self.current_player.to_le_bytes());
        // player_1_level_id excluded: demo-mode timing artefact (native sets it
        // when demo level loads, oracle sets it at a different VBlank boundary).
        hasher.finalize()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeObjectSnapshot {
    pub kind: NativeObjectKind,
    pub x_pos: u32,
    pub y_pos: u32,
    pub layout_index: Option<usize>,
    pub x_velocity: Option<i32>,
    pub y_velocity: Option<i32>,
    pub facing: Option<Direction>,
    pub on_ground: Option<bool>,
    pub hit_points: Option<u8>,
    pub damage_flash: bool,
    /// Palette variant (0, 1, or 2) from the level's enemy type slot.
    #[serde(default)]
    pub palette_variant: u8,
    /// True when the enemy is in a stationary phase (e.g. dragon perching).
    #[serde(default)]
    pub stationary: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_walker_state: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_walker_patrol_timer: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_walker_patrol_short_next: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub walk_phase: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_timer: Option<u16>,
    /// Frontier-specific object debug fields. Start exploratory here; promote
    /// recurring decision-making fields into typed members.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub debug_fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeObjectKind {
    Player,
    Enemy { object_type: u8 },
    Platform { kind: u8, subtype: u8 },
}

#[derive(Debug, Clone)]
pub(crate) struct BlockBounceAnim {
    pub(crate) cell_x: u16,
    pub(crate) cell_y: u16,
    pub(crate) remaining: u8,
    pub(crate) kind: BlockBounceKind,
    /// Total duration when the bounce was spawned. Drives the wobble lookup
    /// per ASM `sub_1007A` (31 frames for prize blocks, shorter arc for
    /// metal/rubber until their ASM coroutines are individually pinned).
    pub(crate) initial_duration: u8,
    /// Prize content for ASM `sub_1007A` bounces. Diamond prize objects use
    /// `sf is_moved(a3)` in `PrizeDiamond_Init`, making their scheduler timing
    /// differ from helmet/shared prize initialisers that set `is_moved`.
    pub(crate) prize_content: Option<terrain::PrizeContent>,
    /// Current y-pixel offset for visual rendering. Negative = rising (block
    /// moves up after a head-bonk). Maintained by `process_block_effects`.
    pub(crate) y_offset_px: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockBounceKind {
    Metal,
    Prize,
    Shifting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeBlockWorkerTail {
    pub(crate) cell_x: u16,
    pub(crate) cell_y: u16,
    pub(crate) content: Option<terrain::PrizeContent>,
    pub(crate) remaining: u8,
}

#[derive(Debug, Clone)]
pub(crate) struct PrizeBlockSpawnFlash {
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) remaining: u8,
}

#[derive(Debug, Clone)]
pub(crate) struct BlockDestroyEffect {
    pub(crate) x_px: u16,
    pub(crate) y_px: u16,
    pub(crate) remaining: u8,
    /// ASM `sub_292E` (kid.asm:4881) selects debris velocity tables from
    /// `d6`: 0=below, 1=left, 2=above, otherwise right/default.
    pub(crate) direction: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockDebrisPieceSub2A4C {
    pub(crate) x_pos_fp: i32,
    pub(crate) y_pos_fp: i32,
    pub(crate) x_velocity_fp: i32,
    pub(crate) y_velocity_fp: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockDebrisSwarmSub2A4C {
    pub(crate) pieces: [BlockDebrisPieceSub2A4C; 5],
}

/// A floating prize item spawned from a prize block hit.
/// ASM: sub_1007A runs the block-bounce coroutine, then spawns
/// Prize_Init -> PrizeDiamond_Init (kid.asm:21604-21679).
/// Item spawns at block center, falls downward with accumulating velocity,
/// collected on GfxObjects_Collision contact with player.
#[derive(Debug, Clone)]
pub(crate) struct FloatingItem {
    /// FP16.16 X position (initialized to block_center_x << 16).
    pub(crate) x_pos: u32,
    /// FP16.16 Y position (initialized to block_y << 16).
    pub(crate) y_pos: i64,
    /// FP16.16 Y velocity (accumulates by +$1388 per frame = downward).
    pub(crate) y_velocity: i32,
    /// Frames remaining. ASM PrizeDiamond_Init uses `$01F4`, while the shared
    /// sub_10C78/sub_10CB0 falling-prize initialisers use `$0320`.
    pub(crate) remaining: u16,
    /// Delay frames before the item becomes active after object creation.
    /// Oracle-backed TAS probing for demo 0x1B shows the spawned prize object
    /// is already active on its first native frame, so this remains `0`.
    pub(crate) spawn_delay: u16,
    /// Source column from the level layout address used by the original
    /// object. Prize items consult this fixed column while falling.
    pub(crate) level_col: u16,
    /// Current row pointer used by the ASM prize/helmet fall routines.
    pub(crate) level_row: u16,
    /// ASM keeps a one-row grace flag so the prize can emerge from its source
    /// tile before later solid rows stop the fall.
    pub(crate) first_row_grace: bool,
    pub(crate) content: terrain::PrizeContent,
    /// Whether the player has collected this item by touching it.
    pub(crate) collected: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct EnemyDeathEffect {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) remaining: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RespawnSettleFrame {
    pub step_index: usize,
    pub kid_x_pos: u32,
    pub kid_y_pos: u32,
    pub kid_x_velocity: i32,
    pub kid_y_velocity: i32,
    pub kid_on_ground: bool,
    pub kid_on_spikes: bool,
    pub camera_x_pos: u32,
    pub camera_y_pos: u32,
    pub floor_y_pos: u32,
    pub wide_support_applied: bool,
}

/// A lift block that has been activated and is now a moving platform.
/// ASM coroutine (sub_113C0): rise 2px/frame × 8, erase block, fall 1px/frame × 16, delete.
#[derive(Debug, Clone)]
pub(crate) struct LiftPlatform {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) y_velocity: i32,
    /// Frames since activation.
    pub(crate) age: u16,
    /// ASM sub_FCA6 keeps the previous integer `6(a3)` word in `d0` so it can
    /// count only the frames where the platform actually moves to a new pixel.
    pub(crate) last_integer_y: u16,
    /// ASM sub_FCA6 increments `d1` only when `6(a3)` changes.
    pub(crate) moved_y_steps: u16,
    /// Width in pixels (one block = 16px).
    pub(crate) width_px: u16,
    /// Grid column of the source block (for erasing after rise phase).
    pub(crate) source_col: u16,
    /// Grid row of the source block.
    pub(crate) source_row: u16,
    /// Whether the source block has been erased from the grid.
    pub(crate) block_erased: bool,
}

/// A lava geyser that erupts periodically from a fixed position.
#[derive(Debug, Clone)]
pub(crate) struct LavaGeyser {
    /// Fixed X position (pixel coordinates, not FP16.16).
    pub(crate) x_px: u16,
    /// Base Y position (bottom of eruption, pixel coordinates).
    pub(crate) y_px: u16,
    /// Eruption height in pixels.
    pub(crate) height_px: u16,
    /// Phase: 0=waiting, 1=erupting upward, 2=holding, 3=collapsing.
    pub(crate) phase: u8,
    /// Frames remaining in current phase.
    pub(crate) timer: u16,
    /// Current eruption column height in pixels (for collision/rendering).
    pub(crate) current_height: u16,
}

/// A scripted lava fireball emitted by the USM2 lava special-effect worker.
#[derive(Debug, Clone)]
pub(crate) struct LavaFireball {
    /// Fixed X position (pixel coordinates).
    pub(crate) x_px: u16,
    /// Baseline Y position used by the ROM script.
    pub(crate) base_y_px: u16,
    /// Current script baseline Y position.
    pub(crate) current_y_px: u16,
    /// Current motion-script segment index.
    pub(crate) segment_index: u8,
    /// Frames spent in the current segment.
    pub(crate) segment_frame: u8,
    /// Frames until the next loop starts.
    pub(crate) cooldown_timer: u16,
}

/// Per-player persistent state for 2P alternating mode.
/// Mirrors the ASM Player_1_*/Player_2_* RAM slots ($FFFFFC54-$FFFFFC80).
#[derive(Debug, Clone)]
pub(crate) struct PlayerSlot {
    pub(crate) lives: u16,
    pub(crate) hitpoints: u16,
    pub(crate) diamonds: u16,
    pub(crate) level_id: LevelId,
    pub(crate) form: PlayerForm,
    pub(crate) continues: u8,
    pub(crate) score: u32,
}

impl Default for PlayerSlot {
    fn default() -> Self {
        // ASM loc_65C (kid.asm:793-819): new-game init for both player slots.
        //   move.w #3,(Player_1_Lives).w        → lives = 3
        //   clr.l  (Player_1_Score).w           → score = 0
        //   clr.w  (Player_1_Helmet).w          → form = TheKid (0)
        //   move.w #2,(Player_1_Hitpoints).w    → hitpoints = 2
        //   move.w #3,(Player_1_Continues).w    → continues = 3
        //   move.w #$FFFF,(Player_1_OneTimePrizes).w
        //   (identical block for Player_2_*)
        // level_id comes from MapHeader_BaseAddress which is 0 at cold boot.
        Self {
            lives: 3,
            hitpoints: PlayerForm::TheKid.max_hitpoints(),
            diamonds: 0,
            level_id: LevelId(0),
            form: PlayerForm::TheKid,
            continues: 3,
            score: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BootState {
    SegaScreen {
        elapsed: usize,
    },
    IntroSequence {
        segment: usize,
        elapsed: usize,
        /// Countdown for deferred Start→title skip. The original game's
        /// intro objects read Ctrl_1_Pressed (set by VBlank interrupt),
        /// giving a 2-frame delay between button press and visible state
        /// change: 1 frame for VBlank interrupt → game code, 1 frame for
        /// jmp loc_6E2 → Pal_FadeOut → next FrameRendered.
        pending_title_skip: u8,
    },
    IntroSequence6 {
        cursor: u8,
        /// ASM loc_1BA5C switches to `Game_Mode=$28`, then Load_IntroSequence6
        /// rebuilds the title shell while the old loc_1BCD0 slot-2 pose still
        /// survives in WRAM. This flag tracks that later timeout-restart
        /// lineage so trace output keeps exposing the carried pose until a
        /// later frontier models the overwrite precisely.
        carry_intro_sequence_5_slot_2_pose: bool,
        /// ASM Load_IntroSequence6 also seeds `($FFFFA656).w` and calls
        /// `sub_1B222`, but only the earlier timeout-restart lineage reuses
        /// the wrapped hidden camera row during the early mode-$28 rebuild.
        /// The later title-restore revisit still carries the stale loc_1BCD0
        /// pose without inheriting that hidden camera wrap.
        carry_timeout_restart_hidden_camera_wrap: bool,
    },
    TitleScreen {
        cursor: u8,
    },
    OptionsMenu {
        cursor: u8,
    },
    #[allow(dead_code)]
    SoundTest {
        track: u8,
    },
    StartingLevel {
        start_latched: bool,
        elapsed_frames: usize,
        /// ASM loc_1AE2E can bypass the visible title-worker shell and jump
        /// straight into `loc_6E2 -> Load_InGame` from the early title-start
        /// lineage. Keep that hidden preload path distinct from the ordinary
        /// title-card worker lineage.
        uses_load_ingame_bootstrap: bool,
        /// ASM loc_1AE2E only starts `loc_6E2 -> Load_InGame` after the later
        /// Start edge, so the early title-start lineage must keep the visible
        /// loc_1AC70 worker shell alive until that second press is consumed.
        load_ingame_bootstrap_started: bool,
    },
    LevelSkipHold,
    InGame,
    /// Demo playback: auto-plays a recorded input sequence on a demo level.
    /// `frame_counter` counts VBlanks (0..1010), `data_index` indexes into the
    /// demo byte array, advancing every 2 frames.
    DemoPlay {
        frame_counter: u16,
        data_index: usize,
        /// false = Demo_InputData1 (Knights Isle), true = Demo_InputData2 (Skydragon Castle)
        use_demo_2: bool,
    },
    /// Post-final-level ending sequence: fade out, credits screens, return to title.
    EndingSequence {
        elapsed: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockGridSource {
    None,
    Manual,
    Generated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSkipHoldPose {
    pub(crate) kid_x_pos: u32,
    pub(crate) kid_y_pos: u32,
    pub(crate) camera_x_pos: u32,
    pub(crate) camera_y_pos: u32,
    pub(crate) number_objects: u16,
    pub(crate) number_gfx_objects: u16,
}

#[derive(Debug, Clone)]
pub(crate) struct LevelSkipTransitionState {
    pub(crate) elapsed_frames: usize,
    pub(crate) frozen_player: PlayerState,
    pub(crate) frozen_camera: CameraState,
    pub(crate) frozen_active_objects: Vec<NativeObjectSnapshot>,
    pub(crate) frozen_number_objects: u16,
    pub(crate) frozen_number_gfx_objects: u16,
}

#[derive(Debug, Clone)]
pub(crate) struct LevelCompleteGfxSnapshot {
    pub(crate) frozen_active_objects: Vec<NativeObjectSnapshot>,
    pub(crate) frozen_number_objects: u16,
    pub(crate) frozen_number_gfx_objects: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelSkipAnimationPath {
    pub(crate) initial_x_pos: i32,
    pub(crate) initial_y_pos: i32,
    pub(crate) x_step_fp: i32,
    pub(crate) y_step_fp: i32,
    pub(crate) first_animation_number_objects: u16,
    pub(crate) first_animation_number_gfx_objects: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct LevelLayoutMetadata {
    pub(crate) width_blocks: u16,
    pub(crate) height_blocks: u16,
    pub(crate) data_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlayerLifecycle {
    Alive,
    DeathWindup {
        remaining_frames: usize,
    },
    Dying {
        remaining_frames: usize,
    },
    Respawning {
        remaining_frames: usize,
    },
    LevelComplete {
        remaining_frames: usize,
        total_frames: usize,
    },
    /// All lives lost — "GAME OVER" display, then continue countdown if available.
    GameOver {
        remaining_frames: usize,
    },
    /// "CONTINUE?" countdown (only enters if continues > 0). Button press uses continue.
    /// ASM: 28 frames initial pause + 291 frames countdown = 319 total.
    ContinueCountdown {
        remaining_frames: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DeathWindupPath {
    #[default]
    Tumble,
    PitFall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlayerState {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) facing: Direction,
    pub(crate) on_ground: bool,
    pub(crate) on_spikes: bool,
    pub(crate) just_jumped: bool,
    pub(crate) jump_buffer_frames: u8,
    pub(crate) right_wall_sticky: bool,
    pub(crate) left_wall_sticky: bool,
    /// Post-ledge-snap pull-up animation cooldown: suppresses all physics for N
    /// frames. ASM: stru_8BB4 = 7 anim frames × 3 VBlanks = 21 frames, driven
    /// by Hibernate_UntilAnimFinished (kid.asm:1982).
    pub(crate) ledge_snap_cooldown: u8,
    /// ASM $FFFFFA72: wall side during wall-cling. Suppresses h_accel (sub_B084)
    /// while active. Set on airborne wall hit (loc_A8DE), cleared on landing,
    /// pressing away, or damage. Some(Right) = wall on right ($FFFFFA72=0),
    /// Some(Left) = wall on left ($FFFFFA72=0xFF).
    pub(crate) wall_cling_direction: Option<Direction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LevelEntryTitleWorker {
    pub(crate) gfx_slot_index: usize,
    pub(crate) glyph: u8,
    pub(crate) activation_frame: usize,
    pub(crate) target_x_pos: u32,
    pub(crate) target_y_pos: u32,
    pub(crate) motion: Option<LevelEntryTitleWorkerMotion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LevelEntryTitleWorkerMotion {
    pub(crate) start_x_pos: u32,
    pub(crate) start_y_pos: u32,
    pub(crate) x_step_fp: i32,
    pub(crate) y_step_fp: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct CameraState {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
}

/// ASM Init_RNG / Get_RandomNumber_byte (kid.asm:1106-1204).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RomRandomState {
    ram: [u8; Self::RAM_LENGTH],
    index: usize,
}

impl RomRandomState {
    // ASM Init_RNG / Get_RandomNumber_byte (kid.asm:1106-1204) treats
    // `RNG_RAM_End` as one-past-the-ring and copies `RNG_RAM_Length=$37`
    // bytes, so the live ring is 55 bytes and the first generated byte reads
    // slot 1 rather than slot 0.
    const RAM_LENGTH: usize = 55;
    const TAP_OFFSET: usize = 31;
    const SEED: [u8; Self::RAM_LENGTH] = [
        0x89, 0x72, 0x2D, 0x8D, 0x66, 0x4F, 0x80, 0x62, 0xCA, 0x5D, 0x30, 0x30, 0x9D, 0x9E, 0x21,
        0xB8, 0x93, 0x77, 0x7F, 0xE4, 0x2B, 0xBE, 0x8D, 0x9E, 0x56, 0xAA, 0xDD, 0xC2, 0xA8, 0x10,
        0xBF, 0x08, 0xB2, 0x9B, 0x8A, 0xCF, 0xAC, 0x64, 0x59, 0x0E, 0x18, 0x4B, 0xC4, 0xF4, 0x89,
        0x6C, 0x50, 0xFD, 0x99, 0x5F, 0x92, 0xD8, 0xD0, 0x90, 0x68,
    ];

    pub(crate) fn next_byte(&mut self) -> u8 {
        self.index = (self.index + 1) % Self::RAM_LENGTH;
        let tap_index = (self.index + Self::TAP_OFFSET) % Self::RAM_LENGTH;
        let next = self.ram[self.index].wrapping_sub(self.ram[tap_index]);
        self.ram[self.index] = next;
        next
    }

    pub(crate) fn next_word(&mut self) -> u16 {
        let high = self.next_byte();
        let low = self.next_byte();
        u16::from_be_bytes([high, low])
    }

    pub(crate) fn next_long(&mut self) -> u32 {
        let high = self.next_word();
        let low = self.next_word();
        u32::from_be_bytes([(high >> 8) as u8, high as u8, (low >> 8) as u8, low as u8])
    }

    pub(crate) fn offset_word(&self) -> u16 {
        ((self.index as i32) - (Self::RAM_LENGTH as i32)) as i16 as u16
    }
}

impl Default for RomRandomState {
    fn default() -> Self {
        Self {
            ram: Self::SEED,
            index: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelStartPose {
    pub(crate) kid_x_pos: u32,
    pub(crate) kid_y_pos: u32,
    pub(crate) camera_x_pos: u32,
    pub(crate) camera_y_pos: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TerrainType {
    #[default]
    Normal,
    Ice,
    Rubber,
}

impl TerrainType {
    pub(crate) fn from_block_type(bt: terrain::BlockType) -> Self {
        match bt {
            terrain::BlockType::Ice => Self::Ice,
            terrain::BlockType::Rubber => Self::Rubber,
            _ => Self::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TerrainState {
    pub(crate) floor_y_pos: u32,
    pub(crate) floor_terrain_type: TerrainType,
}

/// Movement script execution state for a platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformScriptState {
    /// Relative offset into the shared script table (key into `PlatformScriptTable`).
    pub(crate) script_offset: u16,
    /// Current command index within the script.
    pub(crate) command_index: usize,
    /// Remaining frames for the current command.
    pub(crate) remaining_duration: u16,
}

/// An active platform with collision dimensions and movement state.
/// Positions use the same FP16.16 format as player/enemy positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActivePlatform {
    /// Source placement index in the level platform layout.
    pub(crate) layout_index: Option<usize>,
    /// Platform type nibble from the `ptfm` macro.
    pub(crate) kind: u8,
    /// Platform subtype/art variant from the `ptfm` macro.
    pub(crate) subtype: u8,
    /// FP16.16 X position (left edge of the platform).
    pub(crate) x_pos: u32,
    /// FP16.16 Y position (top edge of the platform).
    pub(crate) y_pos: u32,
    /// Platform width in pixels (from h_variant: `h * 16 + 15`).
    pub(crate) width_px: u16,
    /// Platform height in pixels (from v_variant: `v * 16 + 15`).
    pub(crate) height_px: u16,
    /// FP16.16 X velocity (from movement script).
    pub(crate) x_velocity: i32,
    /// FP16.16 Y velocity (from movement script).
    pub(crate) y_velocity: i32,
    /// Movement script state (None for stationary platforms).
    pub(crate) script: Option<PlatformScriptState>,
    /// Special-platform coroutine state for negative ROM platform types.
    pub(crate) special: Option<PlatformSpecialState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformSpecialState {
    Soft(SoftPlatformState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoftPlatformState {
    pub(crate) original_y_pos: u32,
    pub(crate) phase: SoftPlatformPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SoftPlatformPhase {
    Wait,
    Bounce,
    Settled,
}

impl ActivePlatform {
    pub(crate) fn from_placement(
        placement: &kid_data::PlatformPlacement,
        script_table: Option<&kid_data::PlatformScriptTable>,
    ) -> Self {
        // ASM loc_2822 (kid.asm:4748): once `Manage_PlatformLoading` has
        // finished packing the placement into `$18/$1C(a3)`, the label itself
        // only selects a VRAM art word from `PlatformArtAddr_Table`. The
        // native gameplay port does not model that render-only word; it keeps
        // the collision-relevant width/height derived from the typed layout.
        // ASM loc_2836 (kid.asm:4762): non-negative platform types stay on the
        // scripted path and store `PlatformScript_BaseAddress + $A(a4)` into
        // `$12(a3)`, while negative types branch to `loc_2876` as special
        // stationary platforms. The native port mirrors that split as
        // `relative_offset: Some(_)` versus `None`.
        let script = placement
            .script
            .relative_offset
            .and_then(|offset| {
                let table = script_table?;
                let script_data = table.scripts.get(&offset)?;
                let first_cmd = script_data.commands.first()?;
                Some((offset, first_cmd))
            })
            .map(|(offset, first_cmd)| PlatformScriptState {
                script_offset: offset,
                command_index: 0,
                remaining_duration: first_cmd.duration,
            });
        let y_pos = u32::from(placement.y) << 16;
        let special = if placement.kind == 8 && placement.script.raw.trim() == "0" {
            Some(PlatformSpecialState::Soft(SoftPlatformState {
                original_y_pos: y_pos,
                phase: SoftPlatformPhase::Wait,
            }))
        } else {
            None
        };

        Self {
            layout_index: None,
            kind: placement.kind,
            subtype: placement.subtype,
            x_pos: u32::from(placement.x) << 16,
            y_pos,
            width_px: u16::from(placement.h_variant) * 16 + 15,
            height_px: u16::from(placement.v_variant) * 16 + 15,
            x_velocity: 0,
            y_velocity: 0,
            script,
            special,
        }
    }

    pub(crate) fn from_placement_with_index(
        layout_index: usize,
        placement: &kid_data::PlatformPlacement,
        script_table: Option<&kid_data::PlatformScriptTable>,
    ) -> Self {
        Self {
            layout_index: Some(layout_index),
            ..Self::from_placement(placement, script_table)
        }
    }

    /// Advance one frame of movement script execution.
    pub(crate) fn step_script(&mut self, script_table: &kid_data::PlatformScriptTable) {
        let Some(ref mut state) = self.script else {
            return;
        };
        let Some(script) = script_table.scripts.get(&state.script_offset) else {
            return;
        };
        if script.commands.is_empty() {
            return;
        }

        let cmd = &script.commands[state.command_index];
        self.x_velocity = cmd.x_velocity;
        self.y_velocity = cmd.y_velocity;

        // ASM loc_23F2 (kid.asm:4269): add the current `$A/$E(a2)` velocities
        // into `2/6(a2)` before decrementing the command timer.
        self.x_pos = (i64::from(self.x_pos) + i64::from(self.x_velocity))
            .clamp(0, i64::from(u32::MAX)) as u32;
        self.y_pos = (i64::from(self.y_pos) + i64::from(self.y_velocity))
            .clamp(0, i64::from(u32::MAX)) as u32;

        // ASM loc_2422 (kid.asm:4290): when the timer expires, immediately load
        // the next duration and velocities into platform RAM for the following
        // frame, even though this frame already moved with the old command.
        state.remaining_duration = state.remaining_duration.saturating_sub(1);
        if state.remaining_duration == 0 {
            state.command_index += 1;
            if state.command_index >= script.commands.len() {
                // ASM loc_243E (kid.asm:4302) follows the `$FFFF` sentinel's
                // loop-back pointer, which the native typed script table
                // represents by rewinding to command 0 in the current script.
                state.command_index = 0;
            }
            let next_cmd = &script.commands[state.command_index];
            state.remaining_duration = next_cmd.duration;
            self.x_velocity = next_cmd.x_velocity;
            self.y_velocity = next_cmd.y_velocity;
        }
    }

    pub(crate) fn snapshot(&self) -> NativeObjectSnapshot {
        NativeObjectSnapshot {
            kind: NativeObjectKind::Platform {
                kind: self.kind,
                subtype: self.subtype,
            },
            x_pos: self.x_pos,
            y_pos: self.y_pos,
            layout_index: self.layout_index,
            x_velocity: Some(self.x_velocity),
            y_velocity: Some(self.y_velocity),
            facing: None,
            on_ground: None,
            hit_points: None,
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        }
    }
}

/// Countdown timer for the level. Ticks down once per 60 frames.
/// When it reaches zero the player dies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LevelTimer {
    /// Remaining whole seconds (3:00 = 180).
    pub(crate) seconds: u16,
    /// Frames remaining until the next second tick (counts down from 60).
    pub(crate) sub_frames: u16,
}

impl LevelTimer {
    pub(crate) fn new() -> Self {
        Self {
            seconds: super::LEVEL_TIMER_START_SECONDS,
            // ASM sets Time_SubSeconds to 1 — first frame immediately
            // decrements a second and reloads to 60.
            sub_frames: 1,
        }
    }

    /// Tick one frame. Returns true when the timer has just expired (0:00).
    pub(crate) fn tick(&mut self) -> bool {
        if self.seconds == 0 && self.sub_frames == 0 {
            return true; // already expired
        }
        self.sub_frames = self.sub_frames.saturating_sub(1);
        if self.sub_frames == 0 {
            if self.seconds > 0 {
                self.seconds -= 1;
                self.sub_frames = super::LEVEL_TIMER_FRAMES_PER_SECOND;
                false
            } else {
                true // just hit 0:00
            }
        } else {
            false
        }
    }

    pub(crate) fn add_clock_bonus(&mut self) {
        // ASM loc_FFDE (kid.asm:21281): add 3 to `Time_Minutes`; if the
        // result reaches 10 or higher, clamp to 10 and clear both seconds
        // digits.  The native timer stores total remaining seconds, so this is
        // equivalent to adding 180 seconds and capping at exactly 10:00.
        let new_seconds = self.seconds + super::LEVEL_TIMER_CLOCK_BONUS_SECONDS;
        if new_seconds >= super::LEVEL_TIMER_MAX_SECONDS {
            self.seconds = super::LEVEL_TIMER_MAX_SECONDS;
        } else {
            self.seconds = new_seconds;
        }
        // ASM loc_FFFC (kid.asm:21295) runs immediately before the clock
        // collect tail at loc_10002 and always writes `#1` to
        // `Time_SubSeconds`, even when the timer does not cap at 10:00.
        self.sub_frames = 1;
    }
}

/// Tracks a triggered evanescent block that is counting down to removal.
/// The block stays solid during the countdown; once timer reaches the
/// removal threshold it is cleared from the block grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvanescentEntry {
    pub(crate) col: u16,
    pub(crate) row: u16,
    /// Timer value (starts negative from skin offset, incremented each frame).
    pub(crate) timer: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RubberBlockAnimation {
    pub(crate) col: u16,
    pub(crate) row: u16,
    pub(crate) base_skin: u8,
    pub(crate) phase: u8,
    pub(crate) frames_until_next_write: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeferredPrizeBlockPhase {
    Deleted,
    Restored,
    PendingClear,
    PendingClearPrizeSpawned,
    FreshSmash,
}

/// ASM sub_1007A prize-block lifecycle: after a headbutt the block stays Solid
/// (0x6F), then briefly restores the original prize cell, then writes the
/// collision-solid Smash cell `$E001` and creates the prize object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DeferredCeilingClear {
    pub(crate) col: u16,
    pub(crate) row: u16,
    pub(crate) content: terrain::PrizeContent,
    pub(crate) restore_cell: terrain::Cell,
    pub(crate) deleted_frames_remaining: u16,
    pub(crate) phase: DeferredPrizeBlockPhase,
}

/// Runtime state for a ghost block group that cycles solid/vanished.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GhostGroup {
    pub(crate) col: u16,
    pub(crate) row: u16,
    pub(crate) count: u8,
    pub(crate) vertical: bool,
    pub(crate) solid_time: u16,
    pub(crate) vanished_time: u16,
    pub(crate) timer: u16,
    pub(crate) solid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyProjectileOwner {
    pub(crate) segment_index: usize,
    pub(crate) layout_index: usize,
    pub(crate) token: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum EnemyProjectileKind {
    #[default]
    Generic,
    DragonWalkingBreath,
    DripsDrop,
    /// Hostile Drips surface crawler child from ASM `Drips.asm loc_335D8`.
    DripsSurfaceCrawler,
    Hail,
    TankBullet,
    ShooterBlock,
    ManiaxeAxe,
    DiamondPowerRain,
    FireDemonDeathTrail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DripsSurfaceCrawlerState {
    #[default]
    HorizontalLoc33650,
    VerticalLoc336D0,
    VerticalLoc33730,
    VerticalLoc337A0,
    IdleLoc33A7C,
    DiagonalLoc33810,
    DiagonalLoc33892,
}

/// Projectile fired by enemies and player helmet attacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct EnemyProjectile {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    /// Remaining lifetime in frames (destroyed when zero).
    pub(crate) lifetime: u16,
    /// True for player-fired projectiles (Eyeclops beam, Maniaxe axe, Juggernaut skull) — these
    /// damage enemies, not the player.
    pub(crate) friendly: bool,
    /// Per-frame gravity added to y_velocity (for arcing projectiles like Juggernaut skull).
    pub(crate) gravity: i32,
    /// Homing acceleration per axis per frame. 0 = not homing.
    /// HeadyMetal: $7D0, Lion: $2000 (Lion.asm loc_3E1C2).
    pub(crate) homing_accel: i32,
    /// Speed cap for homing projectiles (per-axis). 0 = no cap.
    /// Lion: $60000 (Lion.asm loc_3E1C2).
    pub(crate) speed_cap: i32,
    /// Bounces off terrain walls/floors/ceilings (Lion.asm loc_3E260).
    pub(crate) wall_bounce: bool,
    /// Re-aim interval in frames (Archer arrows). 0 = no re-aiming.
    /// ASM loc_342D6: arrow re-aims every `arrow_accuracy` frames toward player.
    pub(crate) reaim_interval: u16,
    /// Countdown to next re-aim (decremented each frame).
    pub(crate) reaim_countdown: u16,
    /// Current angle index (0..27) for ±1 circular re-aim adjustment.
    /// ASM loc_34308/loc_34320: adjusts by ±1 per re-aim cycle toward target.
    pub(crate) current_angle: u8,
    /// Destroy on terrain contact instead of timer expiry.
    /// ASM loc_342D6: Archer arrows check `$19(a3)` / `collision_type(a3)`.
    /// ASM loc_8ACC: Maniaxe axes branch to their terrain-hit delete path.
    pub(crate) terrain_kill: bool,
    /// Drips.asm loc_33488/off_33520: true only for the splash that reaches
    /// the negative table sentinel and may allocate a loc_335D8 crawler.
    pub(crate) drips_spawn_crawler_on_splash: bool,
    /// Owning enemy identity for projectiles whose lifecycle feeds back into
    /// the spawner state. ASM Lion.asm sub_3E2CE / loc_3E1B4.
    pub(crate) owner: Option<EnemyProjectileOwner>,
    pub(crate) kind: EnemyProjectileKind,
    /// DiamondPower_Main_SlashingRain `$3E(a0)`: zero for non-tracking rain;
    /// incremented by loc_3FF6A after Tracking Rain writes `$26/$2A` velocity.
    pub(crate) diamond_power_rain_tracking_word_3e: u16,
    /// DiamondPower_Main_SlashingRain `$24(a0)`: diamond sprite size used by
    /// loc_3FF90 when centring and culling the compiled rain sprite.
    pub(crate) diamond_power_rain_size_word_24: u16,
    /// Drips surface-crawler movement coroutine label. ASM turn scripts under
    /// `loc_33AF6` dispatch through `off_33ADA`; native persists the subset of
    /// labels wired into the simplified projectile carrier.
    pub(crate) drips_surface_crawler_state: DripsSurfaceCrawlerState,
    /// Drips surface-crawler `x_direction(a3)` byte. `loc_33AF6` toggles this
    /// with `not.b` on script flip bytes, and labels such as `loc_336D0` branch
    /// on whether the byte is zero.
    pub(crate) drips_surface_crawler_x_direction_flipped: bool,
    /// Drips.asm loc_335D8 copies parent state byte `$44(a5)` into the surface
    /// crawler; loc_33526 uses it to decide whether the startup tail reapplies
    /// the current velocity.
    pub(crate) drips_child_byte_44: u8,
    /// Drips.asm loc_335D8 copies parent state byte `$45(a5)` into the surface
    /// crawler; loc_33526 doubles the startup velocity delta when this is set.
    pub(crate) drips_child_byte_45: u8,
}

/// DiamondPower_Main_SlashingRain DP object-list slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DiamondPowerRainSlot {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    /// ASM DiamondPower_Main_SlashingRain `$3E(a0)`: zero before Tracking Rain
    /// acquires a target; incremented at loc_3FF6A after `$26/$2A` are written.
    pub(crate) tracking_word_3e: u16,
    /// ASM loc_3FF90 uses `$24(a0)` to centre and cull the compiled diamond.
    pub(crate) size_word_24: u16,
}

/// Murder wall: an advancing wall that kills the player on contact.
/// Accelerates from zero to max_speed at $100/frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MurderWallState {
    /// Current X position in fixed-point 16.16.
    pub(crate) x_pos: i64,
    /// Current speed in fixed-point 16.16.
    pub(crate) speed: i32,
    /// Terminal speed in fixed-point 16.16.
    pub(crate) max_speed: i32,
    /// True if wall moves right-to-left (reversed).
    pub(crate) reversed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct PlatformLoaderState {
    pub(crate) next_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectSystemState {
    pub(crate) platform_loader: PlatformLoaderState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpeningSliceState {
    pub(crate) player: PlayerState,
    pub(crate) camera: CameraState,
    pub(crate) terrain: TerrainState,
    pub(crate) level_bounds: LevelBounds,
    pub(crate) objects: ObjectSystemState,
    /// ASM `$FFFFF8F0`: visual walk/crawl/slope animation phase.
    pub(crate) walk_visual_accumulator: u16,
    /// Prize block headbutted this frame — (content, x_px, y_px) or None.
    pub(crate) prize_block_hit: Option<(terrain::PrizeContent, u16, u16)>,
    /// Lift block activated this frame — (x_px, y_px) of block top-left in pixels.
    pub(crate) lift_block_hit: Option<(u16, u16)>,
    /// Drill block headbutted this frame — (x_fp, y_fp) centre of spawned projectile.
    pub(crate) drill_projectile_hit: Option<(u32, u32)>,
    /// Metal block bounced this frame — (cell_x, cell_y).
    pub(crate) metal_block_hit: Option<(u16, u16)>,
    /// Rubber block contacted this frame — (cell_x, cell_y).
    pub(crate) rubber_block_hit: Option<(u16, u16)>,
    /// Smash block destroyed this frame — (x_px, y_px).
    pub(crate) smash_block_hit: Option<(u16, u16)>,
    /// Shooter block headbutted this frame — (cell_x, cell_y).
    pub(crate) shooter_block_hit: Option<(u16, u16)>,
    /// Teleporter block contacted this frame — (cell_x, cell_y).
    pub(crate) teleporter_contact: Option<(u16, u16)>,
    /// ASM `Telepad_timer` ($FFFFFA84): countdown before a telepad floor activates.
    pub(crate) telepad_timer: u16,
    /// Prize block ceiling hit: queue the sub_1007A lifecycle for this cell.
    pub(crate) ceiling_prize_clear: Option<(u16, u16, terrain::Cell)>,
    /// Pre-gravity y_vel snapshot for floor landing sub-pixel logic.
    /// ASM tests y_vel before sub_B168 (gravity); native applies gravity first.
    pub(crate) pre_gravity_y_vel: i32,
    /// Sub-frame physics trace (populated when `trace_physics` is enabled).
    pub(crate) physics_trace: Option<PhysicsTrace>,
    /// Focused wall-cling live-release probe captured at the loc_AA9C mirror.
    pub(crate) wall_cling_release_probe: Option<KidWallClingReleaseProbe>,
    /// Focused shared post-kid_flip follow-on stages inside the native loc_AA22 mirror.
    pub(crate) kid_flip_follow_on_stages: Option<KidFlipFollowOnStages>,
    /// Focused gate snapshot around the native `apply_jump_and_gravity_vel_only(...)` call.
    pub(crate) kid_jump_gravity_probe: Option<KidJumpGravityProbe>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_input_mask_round_trip_all_values() {
        for mask in 0..=u8::MAX {
            let input = FrameInput::from_mask(mask);
            assert_eq!(
                input.to_mask(),
                mask,
                "round-trip failed for mask 0x{mask:02X}"
            );
        }
    }

    #[test]
    fn frame_input_to_mask_specific_buttons() {
        let input = FrameInput {
            up: true,
            down: false,
            left: true,
            right: false,
            b: false,
            c: true,
            a: false,
            start: true,
        };
        assert_eq!(input.to_mask(), 0b1010_0101);
    }

    #[test]
    fn frame_input_to_mask_empty() {
        assert_eq!(FrameInput::default().to_mask(), 0);
    }

    #[test]
    fn native_trace_frame_reads_legacy_three_field_block_bounce_entries() {
        let json = r#"{
            "block_bounces": [[8, 12, 8]]
        }"#;

        let frame: NativeTraceFrame = serde_json::from_str(json).expect("legacy trace frame");

        assert_eq!(
            frame.block_bounces,
            vec![(8, 12, 8, 0)],
            "legacy recording traces predate ASM sub_1007A y_offset_px; load them as zero-offset"
        );
    }

    #[test]
    fn native_trace_frame_reads_current_four_field_block_bounce_entries() {
        let json = r#"{
            "block_bounces": [[8, 12, 8, -2]]
        }"#;

        let frame: NativeTraceFrame = serde_json::from_str(json).expect("current trace frame");

        assert_eq!(frame.block_bounces, vec![(8, 12, 8, -2)]);
    }

    #[test]
    fn native_trace_frame_reads_legacy_three_field_block_destroy_entries() {
        let json = r#"{
            "block_destroy_effects": [[80, 52, 2]]
        }"#;

        let frame: NativeTraceFrame = serde_json::from_str(json).expect("legacy trace frame");

        assert_eq!(
            frame.block_destroy_effects,
            vec![(80, 52, 2, 2)],
            "legacy traces predate ASM sub_292E direction; load as the upward floor-smash branch"
        );
    }

    #[test]
    fn native_trace_frame_reads_current_four_field_block_destroy_entries() {
        let json = r#"{
            "block_destroy_effects": [[80, 52, 2, 1]]
        }"#;

        let frame: NativeTraceFrame = serde_json::from_str(json).expect("current trace frame");

        assert_eq!(frame.block_destroy_effects, vec![(80, 52, 2, 1)]);
    }
}
