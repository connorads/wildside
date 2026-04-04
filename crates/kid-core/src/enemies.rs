use super::*;

// Private items from lib.rs that don't come through the glob import.
use super::{
    ACTIVATION_WINDOW_PADDING, ACTIVATION_WINDOW_X_LEAD, ACTIVATION_WINDOW_Y_LEAD, CameraState,
    EnemyProjectile, FLYER_RETENTION_X_PADDING, FLYER_RETENTION_Y_PADDING, MAX_PROJECTILES,
};

// MiniHoppingSkull hopper constants recovered from the enemy module.
pub(crate) const MINI_SKULL_JUMP_X_SPEED_FP: i32 = 0x0001_8000;
pub(crate) const MINI_SKULL_JUMP_Y_SPEED_FP: i32 = -0x0003_8000;
pub(crate) const MINI_SKULL_GRAVITY_FP: i32 = 0x0000_1200;
pub(crate) const MINI_SKULL_GROUND_DELAY_FRAMES: u8 = 21; // ASM Mini_Hopping_Skull.asm:99,141 — dbf #$14 = 21 iterations

// Big Hopping Skull: continuous bouncer from Big_Hopping_Skull.asm.
// FP16.16 velocities: x_vel = ±$10000 (1px/frame), y_vel init = $FFFE0000 (-2px/frame).
// Gravity: $FA0/frame. Deflects off all surfaces with no ground delay.
pub(crate) const BIG_SKULL_INIT_X_SPEED: i32 = 0x0001_0000; // ASM: move.w #1,x_vel (high word)
pub(crate) const BIG_SKULL_INIT_Y_SPEED: i32 = -0x0002_0000; // ASM: move.w #$FFFE,y_vel (high word)
pub(crate) const BIG_SKULL_GRAVITY: i32 = 0x0000_0FA0; // ASM: addi.l #$FA0,y_vel

pub(crate) struct HopperConstants {
    pub(crate) jump_x_speed: i32,
    pub(crate) jump_y_speed: i32,
    pub(crate) gravity: i32,
    pub(crate) ground_delay: u8,
}

pub(crate) fn hopper_constants(_family: EnemyFamily) -> HopperConstants {
    HopperConstants {
        jump_x_speed: MINI_SKULL_JUMP_X_SPEED_FP,
        jump_y_speed: MINI_SKULL_JUMP_Y_SPEED_FP,
        gravity: MINI_SKULL_GRAVITY_FP,
        ground_delay: MINI_SKULL_GROUND_DELAY_FRAMES,
    }
}
// ── Boss multi-head constants (ASM kid.asm:40177–40700) ──
// Each boss spawns 3 head objects at offsets from the layout spawn position.
// Offsets are pixel-level (word), applied to the layout x_pos/y_pos.
/// Sentinel: not a boss head.
pub(crate) const BOSS_HEAD_NONE: u8 = 0xFF;

/// Shishkaboss head Y offsets (X is same for all 3).
pub(crate) const SHISHKABOSS_HEAD_Y_OFFSETS: [i16; 3] = [0, 0x48, 0x90]; // 0, +72, +144
pub(crate) const SHISHKABOSS_HEAD_X_OFFSETS: [i16; 3] = [0, 0, 0];
/// Shishkaboss per-head HP: head1=$14(20), head2=$1E(30), head3=$1E(30).
pub(crate) const SHISHKABOSS_HEAD_HP: [u8; 3] = [20, 30, 30];

/// BoomerangBosses head X offsets (Y is same for all 3).
pub(crate) const BOOMERANG_HEAD_X_OFFSETS: [i16; 3] = [0, -0xC8, -0x1C2]; // 0, -200, -450
pub(crate) const BOOMERANG_HEAD_Y_OFFSETS: [i16; 3] = [0, 0, 0];
/// BoomerangBosses per-head HP: $1E(30) each.
pub(crate) const BOOMERANG_HEAD_HP: [u8; 3] = [30, 30, 30];

/// BagelBrothers head offsets.
pub(crate) const BAGEL_HEAD_X_OFFSETS: [i16; 3] = [0, -0x64, 0]; // 0, -100, 0
pub(crate) const BAGEL_HEAD_Y_OFFSETS: [i16; 3] = [0, -0x96, -0x12C]; // 0, -150, -300
/// BagelBrothers per-head HP: $1E(30) each.
pub(crate) const BAGEL_HEAD_HP: [u8; 3] = [30, 30, 30];

/// Returns true if this enemy family uses multi-head boss spawning.
pub(crate) fn is_multi_head_boss(family: EnemyFamily) -> bool {
    matches!(
        family,
        EnemyFamily::Shishkaboss | EnemyFamily::BoomerangBosses | EnemyFamily::BagelBrothers
    )
}

/// Get per-head HP and position offsets for a multi-head boss.
pub(crate) fn boss_head_config(family: EnemyFamily) -> ([u8; 3], [i16; 3], [i16; 3]) {
    match family {
        EnemyFamily::Shishkaboss => (
            SHISHKABOSS_HEAD_HP,
            SHISHKABOSS_HEAD_X_OFFSETS,
            SHISHKABOSS_HEAD_Y_OFFSETS,
        ),
        EnemyFamily::BoomerangBosses => (
            BOOMERANG_HEAD_HP,
            BOOMERANG_HEAD_X_OFFSETS,
            BOOMERANG_HEAD_Y_OFFSETS,
        ),
        EnemyFamily::BagelBrothers => (BAGEL_HEAD_HP, BAGEL_HEAD_X_OFFSETS, BAGEL_HEAD_Y_OFFSETS),
        _ => ([1, 1, 1], [0, 0, 0], [0, 0, 0]),
    }
}

// Enemy half-width for terrain probes (8px, matching the 16px sprite).
pub(crate) const ENEMY_PROBE_HALF_WIDTH: i32 = 8;
// Enemy height for terrain probes (16px).
pub(crate) const ENEMY_PROBE_HEIGHT: i32 = 16;

/// Compute pacer walk speed from the enemy family and HP.
///
/// The ASM init routines all follow the same pattern: start at $8000 and
/// add $10000 for each HP tier above the family's minimum.
pub(crate) fn pacer_walk_speed(family: EnemyFamily, hit_points: u8) -> i32 {
    let (base_hp, base_speed, speed_step) = match family {
        // Goat: HP = enemy_level + 3 (min 3). Base speed $8000, step $10000.
        // Player-tracking callback (Goat.asm loc_3CC18) wired in step dispatch.
        EnemyFamily::Goat => (3u8, 0x0000_8000i32, 0x0001_0000i32),
        // Armadillo: HP = enemy_level + 1 (min 1). Base speed $8000, step $10000.
        EnemyFamily::Armadillo => (1, 0x0000_8000, 0x0001_0000),
        // Crab: HP = enemy_level + 2 (min 2). Base speed $8000, step $10000.
        EnemyFamily::Crab => (2, 0x0000_8000, 0x0001_0000),
        // Scorpion: HP = enemy_level + 3 (min 3). Fixed speed $18000.
        EnemyFamily::Scorpion => (3, 0x0001_8000, 0),
        // Driller: HP = enemy_level + 2 (min 2). Base speed $8000, step $8000.
        EnemyFamily::Driller => (2, 0x0000_8000, 0x0000_8000),
        // Tank: HP = enemy_level + 2 (min 2). Base speed $8000, step $8000.
        // ASM: Tank.asm lines 52-64, Tank_Shooting.asm lines 53-65.
        EnemyFamily::Tank => (2, 0x0000_8000, 0x0000_8000),
        // Fire Demon: HP = enemy_level + 3 (min 3). Base speed $8000, step $8000.
        EnemyFamily::FireDemon => (3, 0x0000_8000, 0x0000_8000),
        // Robot: HP = enemy_level + 2 (min 2). Base speed $8000, step $8000.
        EnemyFamily::Robot => (2, 0x0000_8000, 0x0000_8000),
        // Orca: fixed 4 HP. Fixed speed $8000.
        EnemyFamily::Orca => (4, 0x0000_8000, 0),
        // Ninja: HP = enemy_level + 2 (min 2). Base speed $8000, step $8000.
        EnemyFamily::Ninja => (2, 0x0000_8000, 0x0000_8000),
        // Lion: HP = enemy_level + 6 (min 6). Binary speed check: $28000 at HP==8,
        // $18000 otherwise. ASM: Lion.asm lines 32-41, `cmpi.b #8`.
        // Using step=0 and handling HP==8 via lion_charge_speed instead.
        EnemyFamily::Lion => return lion_charge_speed(hit_points),
        // DragonWalking: HP = enemy_level + 1 (min 1). Speed $8000 at HP≤2, $18000 at HP=3.
        // ASM: Dragon.asm lines 64-74 — only HP==3 sets $10000 base, then always +$8000.
        EnemyFamily::DragonWalking => (2, 0x0000_8000, 0x0001_0000),
        // Hand: HP = 1. Speed from table: $3000/$4800/$6000 by HP.
        EnemyFamily::Hand => (1, 0x0000_3000, 0x0000_1800),
        // SpinningTwins: walk speed $18000 after landing phase.
        EnemyFamily::SpinningTwins => (1, 0x0001_8000, 0),
        // TarMonster: walk speed $18000 after emerging.
        EnemyFamily::TarMonster => (2, 0x0001_8000, 0),
        _ => (1, 0x0001_0000, 0),
    };
    let tiers = hit_points.saturating_sub(base_hp) as i32;
    base_speed + tiers * speed_step
}

/// Tank fire interval in frames (from Tank_Shooting.asm: $20 = 32 frames).
pub(crate) const TANK_FIRE_INTERVAL: u16 = 0x20;
/// Tank bullet speed in FP16.16 (3 px/frame → $30000).
pub(crate) const TANK_BULLET_SPEED: i32 = 0x3_0000;
/// Tank bullet lifetime in frames (enough to cross a screen width).
pub(crate) const TANK_BULLET_LIFETIME: u16 = 120;

/// Scorpion tracking range in pixels (Scorpion.asm line 158: $AA = 170).
/// Outside this range, speed resets to base $18000.
const SCORPION_TRACK_RANGE: i64 = 0xAA;
/// Scorpion boosted speed when within tracking range and enemy_level > 1 (HP > 3).
/// From Scorpion.asm line 164: $38000.
const SCORPION_BOOST_SPEED: i32 = 0x3_8000;
/// Scorpion base speed (always $18000, from Scorpion.asm init line 30).
const SCORPION_BASE_SPEED: i32 = 0x1_8000;

/// Archer fire interval indexed by enemy_level (HP 1/2/3).
/// From Archer.asm `arrow_accuracy`: $1E=30, $14=20, $0A=10 frames.
pub(crate) const ARCHER_FIRE_TIMER: [u16; 3] = [0x1E, 0x14, 0x0A];

/// Arrow velocity table from Archer.asm `byte_3437A`.
/// 28 entries: (x_direction_sign, sprite_id, x_vel_byte, y_vel_byte).
/// Velocity = signed_byte * 0x1000 (ext.w → swap → asr.l #4).
pub(crate) const ARROW_ANGLE_TABLE: [(i8, i8); 28] = [
    (0x20, 0),   // 0: right
    (0x20, -8),  // 1: right + slight up
    (0x1C, -16), // 2
    (0x18, -24), // 3
    (0x10, -28), // 4
    (8, -32),    // 5
    (0, -32),    // 6: straight up
    (0, -32),    // 7: straight up (mirrored)
    (-8, -32),   // 8
    (-16, -28),  // 9
    (-24, -24),  // 10
    (-28, -16),  // 11
    (-32, -8),   // 12
    (-32, 0),    // 13: left
    (-32, 0),    // 14: left (dup)
    (-32, 8),    // 15
    (-28, 16),   // 16
    (-24, 24),   // 17
    (-16, 28),   // 18
    (-8, 32),    // 19
    (0, 32),     // 20: straight down
    (0, 32),     // 21: straight down (mirrored)
    (8, 32),     // 22
    (16, 28),    // 23
    (24, 24),    // 24
    (28, 16),    // 25
    (32, 8),     // 26
    (32, 0),     // 27: right (wrap)
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActivationWindow {
    pub(crate) x_min: u32,
    pub(crate) x_max: u32,
    pub(crate) y_min: u32,
    pub(crate) y_max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyFamily {
    Diamond,
    Dragon,        // Flying dragon ($0C): choreographed flight pattern
    DragonWalking, // Walking dragon ($0D): stationary with breath attack
    Cloud,
    Sphere,
    BigHoppingSkull,
    EmoRock,
    Archer,
    Fireball,
    FireDemon,
    Robot,
    TarMonster,
    Goat,
    Armadillo,
    Tank,
    Crab,
    Orca,
    #[allow(clippy::upper_case_acronyms)]
    UFO,
    Ninja,
    Lion,
    Scorpion,
    SpinningTwins,
    Drips,
    Hand,
    Tornado,
    Driller,
    MiniHoppingSkull,
    HeadyMetal,
    Shishkaboss,
    BoomerangBosses,
    BagelBrothers,
    Unknown(u8),
}

/// Dragon Flying state machine (Dragon_Flying.asm).
/// ASM uses a script interpreter with 3 variant flight tables selected by skin byte.
/// Commands: Hover, Drift, DriftWithFire, FacePlayer, Descent, ReturnToAnchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DragonFlyingState {
    /// Not a Dragon Flying enemy (default for all other families).
    #[default]
    None,
    /// Hovering in place with zero x velocity (ASM cmd 0, loc_34804).
    Hovering,
    /// Accelerating drift (ASM cmd 1, loc_34776). $800 start, $200/frame, $18000 cap.
    Drifting,
    /// Same as Drift but checks fire-breath cone each frame (ASM cmd 5, loc_34774).
    DriftingWithFire,
    /// Descent attack sub-script: fixed velocity segments from unk_34554.
    Descending,
    /// Rising back to anchor Y with deceleration (ASM cmd 6, loc_34B44).
    ReturningToAnchor,
}

impl DragonFlyingState {
    /// Stationary phases where the dragon is hovering — suppress walk animation.
    /// `None` is excluded: non-Dragon enemies use their own stationary logic.
    pub(crate) fn is_stationary(self) -> bool {
        matches!(self, Self::Hovering)
    }
}

/// Ninja stealth/attack state machine (Ninja.asm).
/// Stealth = pacing invisibly. JumpAttack = airborne lunge at player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum NinjaState {
    /// Walking invisibly, checking detection cone each frame.
    #[default]
    Stealth,
    /// Airborne jump-attack toward the player.
    JumpAttack,
}

/// Orca lunge state machine (Orca.asm).
/// Paces normally, detects nearby player, launches a ballistic lunge arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum OrcaState {
    /// Walking patrol with proximity detection.
    #[default]
    Walking,
    /// Airborne ballistic lunge toward the player.
    Lunging,
}

/// Robot phase state machine (Robot.asm).
/// Paces at HP-dependent speed, periodically shoots, then charges fast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum RobotState {
    /// Walking patrol with periodic shooting.
    #[default]
    Walking,
    /// Charging at high speed after jump counter expires.
    Charging,
}

/// Lion charge state machine (Lion.asm).
/// Paces normally, then charges toward player and fires a fast projectile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum LionState {
    /// Walking patrol, checking for player in range.
    #[default]
    Walking,
    /// Charging toward the player at high speed.
    Charging,
}

/// Hand patrol/attack state machine (Hand.asm).
/// Walks until player enters forward detection range, then jump-attacks.
/// On landing near player, grabs and follows player for 10 frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum HandState {
    /// Walking on ground, checking for player in detection cone.
    #[default]
    Walking,
    /// Airborne jump toward the player.
    Jumping,
    /// Grabbing: hand follows player position for 10 frames (Hand.asm loc_3292E).
    Grabbing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TarMonsterState {
    /// Hidden underground, waiting to emerge.
    #[default]
    Submerged,
    /// Rising from the floor (animation frames, 24f).
    Emerging,
    /// Visible and vulnerable above ground.
    Visible,
    /// Attack wind-up + glob shot + wind-down (8+8+8 = 24f). ASM Op 5.
    Attacking,
    /// Sinking back into the floor (24f).
    Submerging,
}

/// SpinningTwins: orbits first, then drops into walking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SpinningTwinsPhase {
    /// Orbiting at spawn position with visual oscillation.
    #[default]
    Orbiting,
    /// Dropped to ground, walking as a pacer.
    Walking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct SharedWalkerTerrainTrace {
    pub(crate) next_x_px: Option<i32>,
    pub(crate) wall: Option<terrain::WallProbe>,
    pub(crate) center_floor: Option<terrain::FloorProbe>,
    pub(crate) left_floor: Option<terrain::FloorProbe>,
    pub(crate) right_floor: Option<terrain::FloorProbe>,
}

/// Persistent per-enemy state that survives across frames.
/// Enemies are activated when they enter the camera window and
/// deactivated when they leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActiveEnemy {
    /// Index into the enemy layout segment list, or 0 for legacy flat layouts.
    pub(crate) segment_index: usize,
    /// Index into the level's enemy layout objects list.
    pub(crate) layout_index: usize,
    /// Raw layout `enemy_level(a3)` copied from `hit_points_plus_one`.
    pub(crate) enemy_level: u8,
    pub(crate) family: EnemyFamily,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    /// Per-axis desired velocity for acceleration-driven flyers.
    pub(crate) target_x_velocity: i32,
    pub(crate) target_y_velocity: i32,
    /// Per-frame acceleration (UFO, Tornado)
    pub(crate) x_acceleration: i32,
    pub(crate) y_acceleration: i32,
    pub(crate) facing: Direction,
    pub(crate) on_ground: bool,
    pub(crate) hit_points: u8,
    pub(crate) grounded_frames: u8,
    pub(crate) damage_flash_frames: u8,
    /// Palette variant (0, 1, or 2) from the enemy type slot.
    pub(crate) palette_variant: u8,
    /// Generic countdown timer for phased enemies (EmoRock hover wait, etc.)
    pub(crate) wait_timer: u16,
    pub(crate) dragon_phase: DragonFlyingState,
    pub(crate) dragon_phase_frames: u16,
    /// Dragon Flying: script variant (0, 1, 2) from layout skin byte.
    pub(crate) dragon_script_variant: u8,
    /// Dragon Flying: current command index within the script.
    pub(crate) dragon_script_cmd: u8,
    /// Dragon Flying: anchor Y position (FP16.16) — initial spawn Y to return to.
    pub(crate) dragon_anchor_y: u32,
    /// Dragon Flying: descent sub-script entry index.
    pub(crate) dragon_descent_index: u8,
    /// Dragon Flying: frames remaining in current descent entry.
    pub(crate) dragon_descent_frames: u8,
    pub(crate) ninja_state: NinjaState,
    pub(crate) hand_state: HandState,
    pub(crate) lion_state: LionState,
    pub(crate) robot_state: RobotState,
    pub(crate) orca_state: OrcaState,
    /// Orca: wall-pinned flag ($5C(a5)), suppresses X accel during lunge.
    pub(crate) wall_pinned: bool,
    pub(crate) tar_monster_state: TarMonsterState,
    pub(crate) spinning_twins_phase: SpinningTwinsPhase,
    pub(crate) spinning_twins_orbit_index: u8,
    /// UFO beam: segments remaining to spawn (0=not firing). ASM $48(a5).
    pub(crate) ufo_beam_segments: u8,
    /// UFO beam: frames until next segment. ASM $4A(a5).
    pub(crate) ufo_beam_interval: u8,
    /// Boss head index (0, 1, 2) for multi-head bosses. BOSS_HEAD_NONE if not a boss head.
    pub(crate) boss_head_index: u8,
    /// Scorpion: tracking-lock flag ($4E(a5)). Suppresses re-facing when locked onto player.
    pub(crate) scorpion_tracking_lock: bool,
    /// ASM UFO.asm `$4E(a5)`: doubles per-axis acceleration for UFO/Tornado tiers 1-2.
    pub(crate) flight_double_acceleration: bool,
    /// ASM UFO.asm `$4F(a5)`: doubles target speed for UFO/Tornado tier 2.
    pub(crate) flight_double_terminal_speed: bool,
    /// Walk speed override from tracking callback ($50(a5)). When non-zero, step_pacer
    /// uses this instead of pacer_walk_speed(). Set by Scorpion tracking (Scorpion.asm loc_3DD7E).
    pub(crate) walk_speed_override: i32,
    /// Crab: frozen state flag ($6E(a5)). ASM loc_3B104 sets this when player
    /// within 64px and facing the crab; loc_3B170 clears it when distance >224px.
    pub(crate) crab_frozen: bool,
    /// Shared Dragon/Lion grounded-state word ($42(a3)).
    pub(crate) shared_walker_state: i16,
    /// Shared Dragon/Lion patrol cadence countdown ($54(a5)).
    pub(crate) shared_walker_patrol_timer: u16,
    /// Dragon alternates the next patrol cadence reload between $32 and $12C via $6E(a5).
    pub(crate) shared_walker_patrol_short_next: bool,
    /// Frontier-only trace label for the last shared-walker branch taken.
    pub(crate) shared_walker_debug_branch: Option<&'static str>,
    /// Signed horizontal distance to Kid when the grounded shared walker last evaluated him.
    pub(crate) shared_walker_dx_to_kid_px: Option<i16>,
    /// Frontier-only trace of the last terrain probes used by the shared walker.
    pub(crate) shared_walker_terrain_trace: SharedWalkerTerrainTrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyArchetype {
    Bouncer,
    Collectible,
    Flyer,
    Hopper,
    Hoverer,
    Pacer,
    Stationary,
    Tracker,
    Inert,
}

pub(crate) const DRAGON_SPAWN_X_OFFSET_FP: u32 = 0x001A_8000;
/// Dragon families spawn 1px below layout Y. ASM-traced offset.
const DRAGON_SPAWN_Y_OFFSET_PX: u32 = 1;
/// Walking dragons also use the 1px Y offset.
const DRAGON_WALKING_SPAWN_Y_OFFSET_PX: u32 = 1;

// --- Dragon Flying script interpreter (Dragon_Flying.asm) ---

/// Drift initial speed: $800 (subpixel FP16.16). ASM loc_34776.
const DRAGON_DRIFT_INITIAL_SPEED: i32 = 0x800;
/// Drift acceleration per frame: $200. ASM loc_3478C.
const DRAGON_DRIFT_ACCEL: i32 = 0x200;
/// Drift speed cap: $18000. ASM loc_3478C.
const DRAGON_DRIFT_SPEED_CAP: i32 = 0x1_8000;
/// Return-to-anchor deceleration per frame: $400. ASM loc_34B4C.
const DRAGON_RETURN_DECEL: i32 = 0x400;
/// Return-to-anchor intermediate Y velocity cap: $FFFF1000. ASM loc_34B4C.
const DRAGON_RETURN_VEL_CAP_SOFT: i32 = -0xF000_i32; // $FFFF1000 as signed
/// Return-to-anchor hard Y velocity cap: $FFFF8000. ASM loc_34B70.
const DRAGON_RETURN_VEL_CAP_HARD: i32 = -0x8000_i32; // $FFFF8000 as signed
/// Face-player initial x velocity: $8000. ASM loc_3473A.
const DRAGON_FACE_PLAYER_SPEED: i32 = 0x8000;

/// Script command for Dragon Flying flight scripts.
#[derive(Debug, Clone, Copy)]
pub(crate) enum DragonFlyingCmd {
    /// Hover in place for N frames (ASM cmd 0).
    Hover(u16),
    /// Accelerating drift for N frames (ASM cmd 1).
    Drift(u16),
    /// Drift + fire-breath cone check for N frames (ASM cmd 5).
    DriftWithFire(u16),
    /// Face player and set initial drift speed (ASM cmd 3). Immediate.
    FacePlayer,
    /// Descent attack sub-script (ASM cmd 4).
    Descent,
    /// Return to anchor Y position (ASM cmd 6).
    ReturnToAnchor,
}

/// Variant 0 script (unk_34710): hover → descent → drift → hover → drift → return.
const DRAGON_SCRIPT_0: &[DragonFlyingCmd] = &[
    DragonFlyingCmd::Hover(0x70), // 112 frames
    DragonFlyingCmd::Descent,
    DragonFlyingCmd::Drift(0x50), // 80 frames
    DragonFlyingCmd::Hover(0x32), // 50 frames
    DragonFlyingCmd::Drift(0x78), // 120 frames
    DragonFlyingCmd::ReturnToAnchor,
];

/// Variant 1 script (unk_34706): aggressive fire-heavy patrol.
const DRAGON_SCRIPT_1: &[DragonFlyingCmd] = &[
    DragonFlyingCmd::DriftWithFire(0x28), // 40 frames
    DragonFlyingCmd::DriftWithFire(0x1E), // 30 frames
    DragonFlyingCmd::DriftWithFire(0x32), // 50 frames
    DragonFlyingCmd::ReturnToAnchor,
];

/// Variant 2 script (unk_346FA): long fire patrols with retargeting.
const DRAGON_SCRIPT_2: &[DragonFlyingCmd] = &[
    DragonFlyingCmd::DriftWithFire(0x70), // 112 frames
    DragonFlyingCmd::DriftWithFire(0x70), // 112 frames
    DragonFlyingCmd::FacePlayer,
    DragonFlyingCmd::DriftWithFire(0x50), // 80 frames
    DragonFlyingCmd::ReturnToAnchor,
];

pub(crate) fn dragon_script(variant: u8) -> &'static [DragonFlyingCmd] {
    match variant {
        0 => DRAGON_SCRIPT_0,
        1 => DRAGON_SCRIPT_1,
        _ => DRAGON_SCRIPT_2,
    }
}

/// Descent sub-script entry: fixed velocity for N frames (unk_34554).
#[derive(Debug, Clone, Copy)]
enum DragonDiveAction {
    /// Move at (x_vel, y_vel) pixels/frame for N frames. x_vel is for facing-left;
    /// negated when facing right (ASM loc_348CE).
    Move { x_vel: i8, y_vel: i8, frames: u8 },
    /// Spawn fire-breath projectile (ASM $E19F sentinel).
    SpawnFire,
    /// Flip facing direction (ASM $DD48 sentinel).
    FlipDirection,
    /// Restart main flight script (ASM $D8F1 sentinel).
    RestartScript,
}

/// Descent attack animation script (unk_34554, kid.asm:37771).
/// U-shaped dive: descend → level out → ascend → flip → continue ascending → restart.
const DRAGON_DIVE_SCRIPT: &[DragonDiveAction] = &[
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 1,
        frames: 12,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 2,
        frames: 9,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 3,
        frames: 6,
    },
    DragonDiveAction::SpawnFire,
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: 4,
        frames: 6,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 3,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 2,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 1,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: 0,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: -1,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: -2,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -2,
        y_vel: -3,
        frames: 3,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -4,
        frames: 6,
    },
    DragonDiveAction::FlipDirection,
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -3,
        frames: 6,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -2,
        frames: 9,
    },
    DragonDiveAction::Move {
        x_vel: -1,
        y_vel: -1,
        frames: 12,
    },
    DragonDiveAction::RestartScript,
];

#[derive(Debug, Clone, Copy)]
pub(crate) struct EnemyLayoutView<'a> {
    pub(crate) segment_index: usize,
    pub(crate) enemy_type_slots: &'a [u16],
    pub(crate) objects: &'a [EnemyLayoutObject],
}

impl ActivationWindow {
    pub(crate) fn from_camera(camera: CameraState) -> Self {
        let camera_x_px = camera.x_pos >> 16;
        let camera_y_px = camera.y_pos >> 16;
        Self {
            x_min: camera_x_px.saturating_sub(ACTIVATION_WINDOW_PADDING),
            x_max: camera_x_px.saturating_add(ACTIVATION_WINDOW_X_LEAD),
            y_min: camera_y_px.saturating_sub(ACTIVATION_WINDOW_PADDING),
            y_max: camera_y_px.saturating_add(ACTIVATION_WINDOW_Y_LEAD),
        }
    }

    pub(crate) fn contains(&self, x_pos: u32, y_pos: u32) -> bool {
        (self.x_min..=self.x_max).contains(&x_pos) && (self.y_min..=self.y_max).contains(&y_pos)
    }

    pub(crate) fn contains_fixed_signed_with_margin(
        &self,
        x_pos: u32,
        y_pos: u32,
        x_margin: u32,
        y_margin: u32,
    ) -> bool {
        let x_pos = (x_pos as i32) >> 16;
        let y_pos = (y_pos as i32) >> 16;
        let x_min = (self.x_min as i32).saturating_sub(x_margin as i32);
        let x_max = (self.x_max as i32).saturating_add(x_margin as i32);
        let y_min = (self.y_min as i32).saturating_sub(y_margin as i32);
        let y_max = (self.y_max as i32).saturating_add(y_margin as i32);
        (x_min..=x_max).contains(&x_pos) && (y_min..=y_max).contains(&y_pos)
    }
}

impl ActiveEnemy {
    /// Whether this enemy can be damaged (stomped, hit by projectile, etc.)
    pub(crate) fn is_vulnerable(&self) -> bool {
        if self.hit_points == 0 {
            return false;
        }
        if matches!(self.family, EnemyFamily::TarMonster) {
            return matches!(
                self.tar_monster_state,
                TarMonsterState::Visible | TarMonsterState::Attacking
            );
        }
        true
    }

    pub(crate) fn snapshot(&self) -> NativeObjectSnapshot {
        let shared_walker_state =
            matches!(self.family, EnemyFamily::DragonWalking | EnemyFamily::Lion)
                .then_some(self.shared_walker_state);
        let shared_walker_patrol_timer =
            matches!(self.family, EnemyFamily::DragonWalking | EnemyFamily::Lion)
                .then_some(self.shared_walker_patrol_timer);
        let shared_walker_patrol_short_next = matches!(self.family, EnemyFamily::DragonWalking)
            .then_some(self.shared_walker_patrol_short_next);
        let walk_phase =
            matches!(self.family, EnemyFamily::DragonWalking).then_some(self.dragon_script_cmd);
        let wait_timer =
            matches!(self.family, EnemyFamily::DragonWalking).then_some(self.wait_timer);
        let mut debug_fields = std::collections::BTreeMap::new();
        debug_fields.insert("segment_index".into(), self.segment_index.to_string());
        if matches!(self.family, EnemyFamily::DragonWalking) {
            debug_fields.insert("grounded_frames".into(), self.grounded_frames.to_string());
            if let Some(branch) = self.shared_walker_debug_branch {
                debug_fields.insert("shared_walker_branch".into(), branch.into());
            }
            if let Some(dx_to_kid_px) = self.shared_walker_dx_to_kid_px {
                debug_fields.insert(
                    "shared_walker_dx_to_kid_px".into(),
                    dx_to_kid_px.to_string(),
                );
            }
            if let Some(next_x_px) = self.shared_walker_terrain_trace.next_x_px {
                debug_fields.insert(
                    "shared_walker_probe_next_x_px".into(),
                    next_x_px.to_string(),
                );
            }
            if let Some(probe) = self.shared_walker_terrain_trace.center_floor {
                debug_fields.insert(
                    "shared_walker_probe_center_floor".into(),
                    format_shared_walker_floor_probe(probe),
                );
            }
            if let Some(probe) = self.shared_walker_terrain_trace.left_floor {
                debug_fields.insert(
                    "shared_walker_probe_left_floor".into(),
                    format_shared_walker_floor_probe(probe),
                );
            }
            if let Some(probe) = self.shared_walker_terrain_trace.right_floor {
                debug_fields.insert(
                    "shared_walker_probe_right_floor".into(),
                    format_shared_walker_floor_probe(probe),
                );
            }
            if let Some(probe) = self.shared_walker_terrain_trace.wall {
                debug_fields.insert(
                    "shared_walker_probe_wall".into(),
                    format_shared_walker_wall_probe(probe),
                );
            }
        }
        NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy {
                object_type: self.family.object_type(),
            },
            x_pos: self.x_pos,
            y_pos: self.y_pos,
            layout_index: Some(self.layout_index),
            x_velocity: Some(self.x_velocity),
            y_velocity: Some(self.y_velocity),
            facing: Some(self.facing),
            on_ground: Some(self.on_ground),
            hit_points: Some(self.hit_points),
            damage_flash: self.damage_flash_frames > 0,
            palette_variant: self.palette_variant,
            stationary: self.dragon_phase.is_stationary(),
            shared_walker_state,
            shared_walker_patrol_timer,
            shared_walker_patrol_short_next,
            walk_phase,
            wait_timer,
            debug_fields,
        }
    }
}

fn format_shared_walker_floor_probe(probe: terrain::FloorProbe) -> String {
    format!(
        "ground={}/y={}/cell={},{}",
        u8::from(probe.on_ground),
        probe.floor_y_px,
        probe.cell_x,
        probe.cell_y
    )
}

fn format_shared_walker_wall_probe(probe: terrain::WallProbe) -> String {
    format!(
        "hit={}/x={}/cell={},{}",
        u8::from(probe.hit),
        probe.wall_x_px,
        probe.cell_x,
        probe.cell_y
    )
}

impl EnemyFamily {
    pub(crate) fn archetype(self) -> EnemyArchetype {
        match self {
            Self::Diamond => EnemyArchetype::Collectible,
            Self::Dragon | Self::Cloud => EnemyArchetype::Flyer,
            Self::DragonWalking => EnemyArchetype::Pacer, // paces + breath attack
            Self::Sphere | Self::Fireball => EnemyArchetype::Bouncer,
            Self::UFO | Self::Tornado => EnemyArchetype::Flyer,
            Self::MiniHoppingSkull => EnemyArchetype::Hopper,
            Self::BigHoppingSkull => EnemyArchetype::Bouncer, // continuous bouncer, handled in step_enemy
            Self::Drips => EnemyArchetype::Stationary,        // ceiling dropper
            Self::Goat
            | Self::Armadillo
            | Self::Tank
            | Self::Crab
            | Self::Scorpion
            | Self::Driller
            | Self::Orca
            | Self::Hand
            | Self::Robot
            | Self::Lion
            | Self::FireDemon => EnemyArchetype::Pacer,
            Self::Ninja => EnemyArchetype::Pacer, // uses Pacer in stealth, but step_active_enemies dispatches to step_ninja
            Self::EmoRock => EnemyArchetype::Hoverer,
            Self::Archer => EnemyArchetype::Stationary,
            Self::SpinningTwins => EnemyArchetype::Pacer, // orbit phase dispatched to step_spinning_twins
            Self::TarMonster => EnemyArchetype::Stationary, // emerge/submerge cycle
            Self::HeadyMetal | Self::Shishkaboss | Self::BoomerangBosses | Self::BagelBrothers => {
                EnemyArchetype::Tracker
            }
            Self::Unknown(_) => EnemyArchetype::Inert,
        }
    }

    pub(crate) fn from_layout_object(enemy_type_slots: &[u16], object: &EnemyLayoutObject) -> Self {
        match resolved_enemy_id(enemy_type_slots, object.object_type) {
            Some(0x01) => Self::Diamond,
            Some(0x00) => Self::FireDemon,
            Some(0x03) => Self::Robot,
            Some(0x04) => Self::Armadillo,
            Some(0x05) => Self::TarMonster,
            Some(0x09) => Self::Crab,
            Some(0x0A) => Self::Tank,
            Some(0x0C) => Self::Dragon,
            Some(0x0D) => Self::DragonWalking,
            Some(0x06) => Self::Sphere,
            Some(0x07) => Self::Archer,
            Some(0x08) => Self::Orca,
            Some(0x0E) => Self::Cloud,
            Some(0x0F) => Self::UFO,
            Some(0x10) => Self::Goat,
            Some(0x11) => Self::Ninja,
            Some(0x12) => Self::Lion,
            Some(0x13) => Self::Scorpion,
            Some(0x14) => Self::SpinningTwins,
            Some(0x16) => Self::Drips,
            Some(0x17) => Self::Hand,
            Some(0x18) => Self::Tornado,
            Some(0x1A) => Self::Driller,
            Some(0x19) => Self::Fireball,
            Some(0x1B) => Self::EmoRock,
            Some(0x1C) => Self::MiniHoppingSkull,
            Some(0x1D) => Self::BigHoppingSkull,
            Some(0x20) => Self::HeadyMetal,
            Some(0x22) => Self::Shishkaboss,
            Some(0x23) => Self::BoomerangBosses,
            Some(0x24) => Self::BagelBrothers,
            Some(enemy_id) => Self::Unknown(enemy_id),
            None => Self::Unknown(object.object_type),
        }
    }

    pub(crate) fn object_type(self) -> u8 {
        match self {
            Self::FireDemon => 0x00,
            Self::Diamond => 0x01,
            Self::Robot => 0x03,
            Self::Armadillo => 0x04,
            Self::TarMonster => 0x05,
            Self::Sphere => 0x06,
            Self::Archer => 0x07,
            Self::Orca => 0x08,
            Self::Crab => 0x09,
            Self::Tank => 0x0A,
            Self::Dragon => 0x0C,
            Self::DragonWalking => 0x0D,
            Self::Cloud => 0x0E,
            Self::UFO => 0x0F,
            Self::Goat => 0x10,
            Self::Ninja => 0x11,
            Self::Lion => 0x12,
            Self::Scorpion => 0x13,
            Self::SpinningTwins => 0x14,
            Self::Drips => 0x16,
            Self::Hand => 0x17,
            Self::Tornado => 0x18,
            Self::Fireball => 0x19,
            Self::Driller => 0x1A,
            Self::EmoRock => 0x1B,
            Self::MiniHoppingSkull => 0x1C,
            Self::BigHoppingSkull => 0x1D,
            Self::HeadyMetal => 0x20,
            Self::Shishkaboss => 0x22,
            Self::BoomerangBosses => 0x23,
            Self::BagelBrothers => 0x24,
            Self::Unknown(enemy_id) => enemy_id,
        }
    }

    #[cfg(test)]
    fn snapshot(
        self,
        object: &EnemyLayoutObject,
        window: ActivationWindow,
    ) -> Option<NativeObjectSnapshot> {
        let object_x = u32::from(object.x);
        let object_y = u32::from(object.y);
        if !window.contains(object_x, object_y) {
            return None;
        }
        Some(NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy {
                object_type: self.object_type(),
            },
            x_pos: object_x << 16,
            y_pos: object_y << 16,
            layout_index: None,
            x_velocity: None,
            y_velocity: None,
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
        })
    }
}

#[cfg(test)]
pub(crate) fn build_enemy_roster(
    layout: &EnemyLayout,
    window: ActivationWindow,
) -> Vec<NativeObjectSnapshot> {
    let view = enemy_layout_view(layout, window.x_min, window.y_min);
    view.objects
        .iter()
        .filter_map(|object| {
            EnemyFamily::from_layout_object(view.enemy_type_slots, object).snapshot(object, window)
        })
        .collect()
}

/// Activate enemies that enter the camera window and deactivate those that leave.
/// Newly activated enemies start at their layout position.
#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(crate) fn activate_enemies(
    active: &mut Vec<ActiveEnemy>,
    layout: &EnemyLayout,
    window: ActivationWindow,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    player_x: u32,
    killed: &std::collections::HashSet<(usize, usize)>,
) {
    let mut rng = RomRandomState::default();
    activate_enemies_with_rng(
        active,
        layout,
        window,
        camera_x_px,
        camera_y_px,
        level_bounds,
        player_x,
        0,
        killed,
        &mut rng,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn activate_enemies_with_rng(
    active: &mut Vec<ActiveEnemy>,
    layout: &EnemyLayout,
    window: ActivationWindow,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    player_x: u32,
    blank_top_rows: u16,
    killed: &std::collections::HashSet<(usize, usize)>,
    rng: &mut RomRandomState,
) {
    let view = enemy_layout_view(layout, camera_x_px, camera_y_px);
    // Most enemies despawn on spawn-point exit. Flyers keep their live object
    // around longer; the Level 1 Dragon continues drifting after its layout
    // anchor scrolls behind the camera.
    let active_enemy_count = active.len();
    active.retain(|enemy| {
        enemy_should_remain_active(
            enemy,
            &view,
            window,
            camera_x_px,
            camera_y_px,
            level_bounds,
            active_enemy_count,
        )
    });

    // Activate new enemies that entered the window.
    for (index, object) in view.objects.iter().enumerate() {
        let object_x = u32::from(object.x);
        let object_y = u32::from(object.y);
        if !window.contains(object_x, object_y) {
            continue;
        }
        if active
            .iter()
            .any(|e| e.segment_index == view.segment_index && e.layout_index == index)
        {
            continue;
        }
        // Don't respawn killed enemies (ASM: per-segment killed bitfield).
        if killed.contains(&(view.segment_index, index)) {
            continue;
        }
        let family = EnemyFamily::from_layout_object(view.enemy_type_slots, object);
        // ASM init routines copy the layout word at offset +2 straight into
        // enemy_level(a3). Dragon Flying is the exception: it reuses the same
        // layout word as a script variant selector instead of HP scaling.
        let enemy_level = object.hit_points_plus_one as u8;
        let spawn_y = spawn_y_pos_sub_36FF4(family, object_y, blank_top_rows);
        // Dragon Flying: variant from layout, anchor Y from spawn position (ASM $44(a5)).
        let dragon_variant = if matches!(family, EnemyFamily::Dragon) {
            object.hit_points_plus_one as u8
        } else {
            0
        };

        // Multi-head bosses spawn 3 head objects per layout entry.
        let head_count = if is_multi_head_boss(family) { 3u8 } else { 1 };
        let (head_hp_table, head_x_offsets, head_y_offsets) = if is_multi_head_boss(family) {
            boss_head_config(family)
        } else {
            ([0u8; 3], [0i16; 3], [0i16; 3])
        };

        for head_idx in 0..head_count {
            let hit_points = if is_multi_head_boss(family) {
                head_hp_table[head_idx as usize]
            } else {
                spawn_hit_points(family, enemy_level)
            };
            let base_x = spawn_x_pos(family, object_x);
            let base_y = spawn_y;
            let head_x = if is_multi_head_boss(family) {
                let offset_px = head_x_offsets[head_idx as usize];
                (base_x as i64 + (i64::from(offset_px) << 16)).clamp(0, i64::from(u32::MAX)) as u32
            } else {
                base_x
            };
            let head_y = if is_multi_head_boss(family) {
                let offset_px = head_y_offsets[head_idx as usize];
                (base_y as i64 + (i64::from(offset_px) << 16)).clamp(0, i64::from(u32::MAX)) as u32
            } else {
                base_y
            };

            let mut enemy = ActiveEnemy {
                segment_index: view.segment_index,
                layout_index: index,
                enemy_level,
                family,
                x_pos: head_x,
                y_pos: head_y,
                x_velocity: spawn_x_velocity(family, enemy_level),
                y_velocity: spawn_y_velocity(family),
                target_x_velocity: 0,
                target_y_velocity: 0,
                x_acceleration: spawn_x_acceleration(family),
                y_acceleration: spawn_y_acceleration(family),
                facing: spawn_facing(family, head_x, player_x),
                on_ground: matches!(family.archetype(), EnemyArchetype::Pacer),
                hit_points,
                grounded_frames: if matches!(family, EnemyFamily::DragonWalking) {
                    1
                } else {
                    0
                },
                damage_flash_frames: 0,
                palette_variant: resolved_enemy_palette(view.enemy_type_slots, object.object_type),
                wait_timer: spawn_wait_timer(family, enemy_level),
                dragon_phase: spawn_dragon_phase(family),
                dragon_phase_frames: 0,
                dragon_script_variant: dragon_variant,
                dragon_script_cmd: 0,
                dragon_anchor_y: head_y,
                dragon_descent_index: 0,
                dragon_descent_frames: 0,
                ninja_state: NinjaState::Stealth,
                hand_state: HandState::Walking,
                lion_state: LionState::Walking,
                robot_state: RobotState::Walking,
                orca_state: OrcaState::Walking,
                wall_pinned: false,
                tar_monster_state: TarMonsterState::default(),
                spinning_twins_phase: SpinningTwinsPhase::default(),
                spinning_twins_orbit_index: 0,
                ufo_beam_segments: 0,
                ufo_beam_interval: 0,
                boss_head_index: if is_multi_head_boss(family) {
                    head_idx
                } else {
                    BOSS_HEAD_NONE
                },
                scorpion_tracking_lock: false,
                crab_frozen: false,
                flight_double_acceleration: matches!(
                    family,
                    EnemyFamily::UFO | EnemyFamily::Tornado
                ) && enemy_level >= 1,
                flight_double_terminal_speed: matches!(
                    family,
                    EnemyFamily::UFO | EnemyFamily::Tornado
                ) && enemy_level >= 2,
                walk_speed_override: 0,
                shared_walker_state: spawn_shared_walker_state(family),
                shared_walker_patrol_timer: spawn_shared_walker_patrol_timer(family),
                shared_walker_patrol_short_next: spawn_shared_walker_patrol_short_next(family),
                shared_walker_debug_branch: None,
                shared_walker_dx_to_kid_px: None,
                shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
            };
            if matches!(family, EnemyFamily::Tornado) {
                reload_tornado_pattern_with_rng(&mut enemy, rng);
            }
            active.push(enemy);
        }
    }
}

/// Step all active enemies: apply movement, gravity, and terrain collision.
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn step_active_enemies(
    active: &mut [ActiveEnemy],
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    player_facing: Option<Direction>,
    player_x_velocity: Option<i32>,
    player_form: PlayerForm,
    projectiles: Option<&mut Vec<EnemyProjectile>>,
    drilled_blocks: &mut Vec<(u16, u16)>,
) {
    let mut rng = RomRandomState::default();
    step_active_enemies_with_rng(
        active,
        block_grid,
        player_x_pos,
        player_y_pos,
        None,
        player_facing,
        player_x_velocity,
        player_form,
        projectiles,
        &mut rng,
        drilled_blocks,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn step_active_enemies_with_rng(
    active: &mut [ActiveEnemy],
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    camera_x_px: Option<u32>,
    player_facing: Option<Direction>,
    player_x_velocity: Option<i32>,
    player_form: PlayerForm,
    mut projectiles: Option<&mut Vec<EnemyProjectile>>,
    rng: &mut RomRandomState,
    drilled_blocks: &mut Vec<(u16, u16)>,
) {
    for enemy in active.iter_mut() {
        enemy.damage_flash_frames = enemy.damage_flash_frames.saturating_sub(1);
        match enemy.family.archetype() {
            EnemyArchetype::Collectible | EnemyArchetype::Inert => {}
            EnemyArchetype::Bouncer => {
                if matches!(enemy.family, EnemyFamily::BigHoppingSkull) {
                    step_big_hopping_skull(enemy, block_grid);
                } else {
                    step_bouncer_with_rng(enemy, block_grid, camera_x_px, rng);
                }
            }
            EnemyArchetype::Flyer => {
                if matches!(enemy.family, EnemyFamily::Dragon) {
                    step_dragon(
                        enemy,
                        block_grid,
                        player_x_pos,
                        player_y_pos,
                        &mut projectiles,
                    );
                } else {
                    step_flyer_with_rng(enemy, block_grid, rng);
                    if matches!(enemy.family, EnemyFamily::UFO)
                        && let Some(ref mut projs) = projectiles
                    {
                        step_ufo_beam(enemy, projs, player_x_pos, player_y_pos);
                    }
                }
            }
            EnemyArchetype::Hopper => step_hopper(enemy, block_grid, player_x_pos),
            EnemyArchetype::Hoverer => step_emo_rock(enemy, block_grid, player_x_pos, player_y_pos),
            EnemyArchetype::Pacer => {
                if matches!(enemy.family, EnemyFamily::Ninja) {
                    step_ninja(enemy, block_grid, player_x_pos, player_y_pos, player_form);
                } else if matches!(enemy.family, EnemyFamily::Hand) {
                    step_hand(enemy, block_grid, player_x_pos, player_y_pos);
                } else if matches!(enemy.family, EnemyFamily::Lion) {
                    step_lion(enemy, block_grid, player_x_pos, &mut projectiles);
                } else if matches!(enemy.family, EnemyFamily::Robot) {
                    step_robot_with_rng(enemy, block_grid, player_x_pos, &mut projectiles, rng);
                } else if matches!(enemy.family, EnemyFamily::Orca) {
                    step_orca(enemy, block_grid, player_x_pos);
                } else if matches!(enemy.family, EnemyFamily::SpinningTwins) {
                    step_spinning_twins(enemy, block_grid, player_x_pos, player_y_pos);
                } else {
                    if matches!(enemy.family, EnemyFamily::DragonWalking) {
                        if let Some(ref mut projs) = projectiles {
                            step_dragon_walking(enemy, block_grid, projs, player_x_pos);
                        } else {
                            step_dragon_walking(enemy, block_grid, &mut Vec::new(), player_x_pos);
                        }
                    } else if matches!(enemy.family, EnemyFamily::Driller) {
                        // Driller.asm loc_3BE78: drills through Smash/Ice blocks while pacing.
                        let cells = step_driller(enemy, block_grid);
                        drilled_blocks.extend(cells);
                    } else {
                        // Crab: player-proximity freeze callback (Crab.asm loc_3B0BE-loc_3B174).
                        // Check freeze before stepping; if frozen, skip movement.
                        if matches!(enemy.family, EnemyFamily::Crab) {
                            step_crab_freeze(enemy, player_x_pos, player_facing, player_x_velocity);
                            if enemy.crab_frozen {
                                continue; // skip pacer walk while frozen
                            }
                        }
                        step_pacer(enemy, block_grid);
                        // Goat: player-tracking callback (Goat.asm loc_3CC18).
                        // If player is >32px away, turn to face them.
                        if matches!(enemy.family, EnemyFamily::Goat)
                            && let Some(px) = player_x_pos
                        {
                            let dx = (px as i64 - enemy.x_pos as i64) >> 16;
                            if dx.abs() > 0x20 {
                                enemy.facing = if dx > 0 {
                                    Direction::Right
                                } else {
                                    Direction::Left
                                };
                            }
                        }
                        // Scorpion: tracking callback (Scorpion.asm loc_3DD7E).
                        // Within 170px: face toward player (unless tracking-locked),
                        // boost speed to $38000 if enemy_level > 1 (HP > 3).
                        // Outside 170px: reset speed to base $18000, clear tracking lock
                        // if player is airborne.
                        if matches!(enemy.family, EnemyFamily::Scorpion) {
                            step_scorpion_tracking(enemy, player_x_pos, player_y_pos);
                        }
                        // Tank fires horizontal bullets at intervals
                        if matches!(enemy.family, EnemyFamily::Tank)
                            && let Some(ref mut projs) = projectiles
                        {
                            step_tank_fire(enemy, projs);
                        }
                    }
                }
            }
            EnemyArchetype::Stationary => {
                if matches!(enemy.family, EnemyFamily::TarMonster) {
                    if let Some(ref mut projs) = projectiles {
                        step_tar_monster(enemy, projs);
                    } else {
                        step_tar_monster(enemy, &mut Vec::new());
                    }
                } else if matches!(enemy.family, EnemyFamily::Drips) {
                    if let Some(projs) = projectiles.as_deref_mut() {
                        step_drips(enemy, projs);
                    }
                } else if let Some(ref mut projs) = projectiles {
                    step_archer(enemy, projs, player_x_pos, player_y_pos);
                }
            }
            EnemyArchetype::Tracker => {
                step_tracker(enemy, player_x_pos, player_y_pos);
                // Bosses periodically spawn projectiles while tracking
                if let Some(ref mut projs) = projectiles {
                    step_boss_fire(enemy, projs, player_x_pos, player_y_pos);
                }
            }
        }
    }
}

pub(crate) fn step_flyer_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    rng: &mut RomRandomState,
) {
    // Cloud: horizontal flight with wall reversal
    if matches!(enemy.family, EnemyFamily::Cloud) {
        step_cloud(enemy, block_grid);
        return;
    }

    // UFO: acceleration-based drifting with periodic direction changes
    if matches!(enemy.family, EnemyFamily::UFO) {
        step_ufo(enemy, block_grid);
        return;
    }

    // Tornado: acceleration-based bouncing patrol with wall/floor reversal
    if matches!(enemy.family, EnemyFamily::Tornado) {
        step_tornado_with_rng(enemy, block_grid, rng);
        return;
    }

    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
}

/// Cloud flies horizontally, reversing direction on wall contact.
/// ASM `Cloud.asm loc_35BDE -> loc_35C54`: `move.w #$3C,d3` is written after
/// wall reversal, but the active loop never reads `d3`, so Cloud resumes
/// flying immediately on the next frame instead of idling.
pub(crate) fn step_cloud(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

    // Wall probe: reverse on wall contact (Cloud.asm loc_35BDE)
    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;
        let moving_right = enemy.x_velocity >= 0;
        let wall = terrain::probe_wall(
            grid,
            cx,
            cy,
            ENEMY_PROBE_HALF_WIDTH,
            ENEMY_PROBE_HEIGHT,
            moving_right,
        );
        if wall.hit {
            enemy.x_velocity = -enemy.x_velocity;
            enemy.facing = opposite_direction(enemy.facing);
        }
    }
}

/// UFO: acceleration-based drifting, reverses direction periodically.
/// Uses `wait_timer` as a countdown; every 60 frames reverses acceleration.
/// Acceleration ±$400 per frame, terminal velocity ±$18000.
pub(crate) const UFO_ACCELERATION: i32 = 0x400;
pub(crate) const UFO_TERMINAL_VELOCITY: i32 = 0x18000;
pub(crate) const UFO_PATTERN_FRAMES: u16 = 60;

pub(crate) fn step_ufo(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    // Apply acceleration to velocity (clamped to terminal)
    enemy.x_velocity = (enemy.x_velocity + enemy.x_acceleration)
        .clamp(-UFO_TERMINAL_VELOCITY, UFO_TERMINAL_VELOCITY);
    enemy.y_velocity = (enemy.y_velocity + enemy.y_acceleration)
        .clamp(-UFO_TERMINAL_VELOCITY, UFO_TERMINAL_VELOCITY);

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    // Periodic direction change
    if enemy.wait_timer == 0 {
        enemy.wait_timer = UFO_PATTERN_FRAMES;
        enemy.x_acceleration = -enemy.x_acceleration;
        enemy.y_acceleration = -enemy.y_acceleration;
    } else {
        enemy.wait_timer -= 1;
    }

    // Wall/floor reversal
    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;
        let moving_right = enemy.x_velocity >= 0;
        let wall = terrain::probe_wall(
            grid,
            cx,
            cy,
            ENEMY_PROBE_HALF_WIDTH,
            ENEMY_PROBE_HEIGHT,
            moving_right,
        );
        if wall.hit {
            enemy.x_velocity = -enemy.x_velocity;
            enemy.x_acceleration = -enemy.x_acceleration;
            enemy.facing = opposite_direction(enemy.facing);
        }
    }
}

/// UFO beam: spawns downward beam segments at the UFO's position.
/// ASM: `ChkShoot` fires when player is directly below (±2px), `ChkLoadBeam` fires
/// randomly (~4/256 per frame). Spawns 4 segments at 5-frame intervals.
const UFO_BEAM_SEGMENT_COUNT: u8 = 4;
const UFO_BEAM_INTERVAL_FRAMES: u8 = 5;
/// Beam segment y_velocity: $30000 from ASM.
const UFO_BEAM_Y_VELOCITY: i32 = 0x3_0000;

pub(crate) fn step_ufo_beam(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x: Option<u32>,
    player_y: Option<u32>,
) {
    // Active beam sequence: spawn segments at intervals
    if enemy.ufo_beam_segments > 0 {
        if enemy.ufo_beam_interval == 0 {
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: (enemy.y_pos as i64 + (10 << 16)) as u32,
                x_velocity: 0,
                y_velocity: UFO_BEAM_Y_VELOCITY,
                lifetime: 60,
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
            enemy.ufo_beam_segments -= 1;
            enemy.ufo_beam_interval = UFO_BEAM_INTERVAL_FRAMES;
        } else {
            enemy.ufo_beam_interval -= 1;
        }
        return;
    }

    // Check if player is directly below — targeted shoot (ASM ChkShoot)
    if let (Some(px), Some(py)) = (player_x, player_y) {
        let ex = enemy.x_pos >> 16;
        let ey = enemy.y_pos >> 16;
        let kx = px >> 16;
        let ky = py >> 16;
        // Player must be below UFO and within ±2px horizontally
        if ky > ey && kx.abs_diff(ex) <= 2 {
            enemy.ufo_beam_segments = UFO_BEAM_SEGMENT_COUNT;
            enemy.ufo_beam_interval = 0;
            // Freeze UFO during beam sequence (ASM clears velocities)
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            return;
        }
    }

    // Random beam: ~4/256 chance per frame (ASM ChkLoadBeam)
    // Use a deterministic pseudo-random from enemy position + wait_timer
    let pseudo_rand = (enemy
        .x_pos
        .wrapping_add(enemy.wait_timer as u32)
        .wrapping_mul(2654435761))
        >> 24;
    if pseudo_rand < 4 {
        enemy.ufo_beam_segments = UFO_BEAM_SEGMENT_COUNT;
        enemy.ufo_beam_interval = 0;
    }
}

/// Tornado: acceleration-based flight. ASM UFO.asm loc_35326/unk_35344/loc_35444
/// shares `Enemy_HandleAcceleration` with UFO. Terminal velocity from the table
/// entries is typically $18000. Cycle timer from `unk_3588C`: 60/40/30 by enemy_level.
pub(crate) const TORNADO_ACCELERATION: i32 = 0xC00;
/// Per-level cycle timers: level 0 = 60, level 1 = 40, level 2 = 30 (unk_3588C).
pub(crate) const TORNADO_PATTERN_FRAMES: [u16; 3] = [60, 40, 30];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TornadoPattern {
    pub(crate) target_x_velocity: i32,
    pub(crate) target_y_velocity: i32,
    pub(crate) x_acceleration: i32,
    pub(crate) y_acceleration: i32,
}

// ASM UFO.asm unk_35344: 16 entries of {target_x_vel, target_y_vel, x_accel, y_accel}.
pub(crate) const TORNADO_PATTERNS: [TornadoPattern; 16] = [
    TornadoPattern {
        target_x_velocity: 0x0000_0000,
        target_y_velocity: -0x0002_0000,
        x_acceleration: 0x0000_0000,
        y_acceleration: -0x0000_1000,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_0000,
        target_y_velocity: -0x0002_0000,
        x_acceleration: 0x0000_0800,
        y_acceleration: -0x0000_1000,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: 0x0000_0C00,
        y_acceleration: -0x0000_0C00,
    },
    TornadoPattern {
        target_x_velocity: 0x0002_0000,
        target_y_velocity: -0x0001_0000,
        x_acceleration: 0x0000_1000,
        y_acceleration: -0x0000_0800,
    },
    TornadoPattern {
        target_x_velocity: 0x0002_0000,
        target_y_velocity: 0x0000_0000,
        x_acceleration: 0x0000_1000,
        y_acceleration: 0x0000_0000,
    },
    TornadoPattern {
        target_x_velocity: 0x0002_0000,
        target_y_velocity: 0x0001_0000,
        x_acceleration: 0x0000_1000,
        y_acceleration: 0x0000_0800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: 0x0000_0C00,
        y_acceleration: 0x0000_0C00,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_0000,
        target_y_velocity: 0x0002_0000,
        x_acceleration: 0x0000_0800,
        y_acceleration: 0x0000_1000,
    },
    TornadoPattern {
        target_x_velocity: 0x0000_0000,
        target_y_velocity: 0x0002_0000,
        x_acceleration: 0x0000_0000,
        y_acceleration: 0x0000_1000,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_0000,
        target_y_velocity: 0x0002_0000,
        x_acceleration: -0x0000_0800,
        y_acceleration: 0x0000_1000,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: -0x0000_0C00,
        y_acceleration: 0x0000_0C00,
    },
    TornadoPattern {
        target_x_velocity: -0x0002_0000,
        target_y_velocity: 0x0001_0000,
        x_acceleration: -0x0000_1000,
        y_acceleration: 0x0000_0800,
    },
    TornadoPattern {
        target_x_velocity: -0x0002_0000,
        target_y_velocity: 0x0000_0000,
        x_acceleration: -0x0000_1000,
        y_acceleration: 0x0000_0000,
    },
    TornadoPattern {
        target_x_velocity: -0x0002_0000,
        target_y_velocity: -0x0001_0000,
        x_acceleration: -0x0000_1000,
        y_acceleration: -0x0000_0800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: -0x0000_0C00,
        y_acceleration: -0x0000_0C00,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_0000,
        target_y_velocity: -0x0002_0000,
        x_acceleration: -0x0000_0800,
        y_acceleration: -0x0000_1000,
    },
];

pub(crate) fn tornado_pattern_index_with_rng(
    enemy: &ActiveEnemy,
    rng: &mut RomRandomState,
) -> usize {
    // ASM UFO.asm loc_35326: `Get_RandomNumber_byte`; `eor.b x_pos(a3),d7`; `andi #$F`.
    let random_byte = rng.next_byte();
    let x_low_byte = ((enemy.x_pos >> 16) & 0xFF) as u8;
    usize::from((random_byte ^ x_low_byte) & 0x0F)
}

fn acceleration_for_target(
    current_velocity: i32,
    target_velocity: i32,
    base_acceleration: i32,
) -> i32 {
    if base_acceleration == 0 {
        // ASM UFO.asm loc_3545A/loc_35496: zero accel entry means "$C00 toward zero".
        return if current_velocity < 0 {
            TORNADO_ACCELERATION
        } else {
            -TORNADO_ACCELERATION
        };
    }

    let mut acceleration = base_acceleration;
    if current_velocity >= 0 {
        if target_velocity >= 0 && target_velocity < current_velocity {
            acceleration = -acceleration;
        }
    } else if target_velocity >= 0 || target_velocity > current_velocity {
        acceleration = -acceleration;
    }
    acceleration
}

fn apply_acceleration_axis(current_velocity: i32, target_velocity: i32, acceleration: i32) -> i32 {
    let next_velocity = current_velocity.saturating_add(acceleration);
    if acceleration >= 0 {
        next_velocity.min(target_velocity)
    } else {
        next_velocity.max(target_velocity)
    }
}

pub(crate) fn reload_tornado_pattern_with_rng(enemy: &mut ActiveEnemy, rng: &mut RomRandomState) {
    // ASM UFO.asm loc_35326 -> loc_35444: choose a table row, then derive the
    // signed per-axis acceleration that drives Enemy_HandleAcceleration.
    // ASM UFO.asm loc_34E10 then doubles both acceleration (`$4E(a5)`) and
    // desired speed (`$4F(a5)`) for higher Tornado/UFO layout tiers.
    let pattern = TORNADO_PATTERNS[tornado_pattern_index_with_rng(enemy, rng)];
    let target_x_velocity = if enemy.flight_double_terminal_speed {
        pattern.target_x_velocity.saturating_mul(2)
    } else {
        pattern.target_x_velocity
    };
    let target_y_velocity = if enemy.flight_double_terminal_speed {
        pattern.target_y_velocity.saturating_mul(2)
    } else {
        pattern.target_y_velocity
    };
    let x_acceleration =
        acceleration_for_target(enemy.x_velocity, target_x_velocity, pattern.x_acceleration);
    let y_acceleration =
        acceleration_for_target(enemy.y_velocity, target_y_velocity, pattern.y_acceleration);
    enemy.target_x_velocity = target_x_velocity;
    enemy.target_y_velocity = target_y_velocity;
    enemy.x_acceleration = if enemy.flight_double_acceleration {
        x_acceleration.saturating_mul(2)
    } else {
        x_acceleration
    };
    enemy.y_acceleration = if enemy.flight_double_acceleration {
        y_acceleration.saturating_mul(2)
    } else {
        y_acceleration
    };
    enemy.facing = if target_x_velocity < 0 {
        Direction::Left
    } else {
        Direction::Right
    };
}

#[cfg(test)]
pub(crate) fn step_tornado(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    let mut rng = RomRandomState::default();
    step_tornado_with_rng(enemy, block_grid, &mut rng);
}

pub(crate) fn step_tornado_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    rng: &mut RomRandomState,
) {
    // ASM UFO.asm Enemy_HandleAcceleration: move toward desired axis speeds.
    enemy.x_velocity = apply_acceleration_axis(
        enemy.x_velocity,
        enemy.target_x_velocity,
        enemy.x_acceleration,
    );
    enemy.y_velocity = apply_acceleration_axis(
        enemy.y_velocity,
        enemy.target_y_velocity,
        enemy.y_acceleration,
    );

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    // ASM UFO.asm loc_35326 + unk_3588C: reload a new pattern on each sub-cycle.
    if enemy.wait_timer == 0 {
        let level = usize::from(enemy.enemy_level).min(2);
        enemy.wait_timer = TORNADO_PATTERN_FRAMES[level];
        reload_tornado_pattern_with_rng(enemy, rng);
    } else {
        enemy.wait_timer -= 1;
    }

    // Wall and floor reversal
    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;
        let moving_right = enemy.x_velocity >= 0;
        let wall = terrain::probe_wall(
            grid,
            cx,
            cy,
            ENEMY_PROBE_HALF_WIDTH,
            ENEMY_PROBE_HEIGHT,
            moving_right,
        );
        if wall.hit {
            enemy.x_velocity = -enemy.x_velocity;
            enemy.x_acceleration = -enemy.x_acceleration;
            enemy.facing = opposite_direction(enemy.facing);
        }
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground && enemy.y_velocity > 0 {
            enemy.y_velocity = -enemy.y_velocity;
            enemy.y_acceleration = -enemy.y_acceleration;
        }
    }
}

/// Sphere bounces off walls/floors/ceilings with gravity.
/// Sphere.asm: gravity +$800/frame, wall→negate x_vel, floor→negate y_vel,
/// ceiling→negate y_vel and halve.
pub(crate) const SPHERE_GRAVITY: i32 = 0x800;

/// Fireball max upward velocity indexed by live HP
/// (Fireball.asm `Enemy19_Fireball_Init`: `move.w 2(a0),d7; asl.w #3,d7` into `unk_32BA6`).
/// HP 1/2/3 use the second/third/final table entries because spawn HP already includes the +1 base.
pub(crate) const FIREBALL_UPWARD_CAP_BY_HP: [i32; 3] = [-0x4_0000, -0x5_0000, -0x5_0000];

#[cfg(test)]
pub(crate) fn fireball_bounce_perturbation(enemy: &ActiveEnemy) -> i32 {
    let mut rng = RomRandomState::default();
    let camera_x_px = enemy.x_pos >> 16;
    fireball_bounce_perturbation_with_rng(&mut rng, camera_x_px)
}

pub(crate) fn fireball_bounce_perturbation_with_rng(
    rng: &mut RomRandomState,
    camera_x_px: u32,
) -> i32 {
    // ASM Enemy19_Fireball_BounceFloorCeiling: `Get_RandomNumber_byte`,
    // `eor.b (Camera_X_pos).w,d7`, `ext.w d7`, `asl.w #7,d7`.
    let random_byte = rng.next_byte();
    let camera_x_low_byte = (camera_x_px & 0xFF) as u8;
    i32::from((random_byte ^ camera_x_low_byte) as i8) << 7
}

#[cfg(test)]
pub(crate) fn fireball_bounce_y_velocity(enemy: &ActiveEnemy) -> i32 {
    let mut rng = RomRandomState::default();
    let camera_x_px = enemy.x_pos >> 16;
    fireball_bounce_y_velocity_with_rng(enemy, &mut rng, camera_x_px)
}

pub(crate) fn fireball_bounce_y_velocity_with_rng(
    enemy: &ActiveEnemy,
    rng: &mut RomRandomState,
    camera_x_px: u32,
) -> i32 {
    let perturbed = enemy
        .y_velocity
        .saturating_add(fireball_bounce_perturbation_with_rng(rng, camera_x_px));
    let mut rebound = perturbed.saturating_neg();
    if rebound < 0 {
        let hp_index = enemy.hit_points.saturating_sub(1).min(2) as usize;
        let cap = FIREBALL_UPWARD_CAP_BY_HP[hp_index];
        if rebound < cap {
            rebound = cap;
        }
    }
    rebound
}

pub(crate) fn bouncer_gravity(enemy: &ActiveEnemy) -> i32 {
    match enemy.family {
        // Fireball.asm `Enemy19_Fireball_Init` indexes `unk_32BA6` with the raw
        // layout `hit_points_plus_one`, so live HP 1/2/3 maps to $2000/$3000/$3000.
        EnemyFamily::Fireball => {
            const FIREBALL_GRAVITY_BY_HP: [i32; 3] = [0x2000, 0x3000, 0x3000];
            let hp_index = enemy.hit_points.saturating_sub(1) as usize;
            FIREBALL_GRAVITY_BY_HP[hp_index.min(2)]
        }
        _ => SPHERE_GRAVITY,
    }
}

/// Big Hopping Skull: continuous bouncer from Big_Hopping_Skull.asm.
/// Gravity $FA0/frame, deflects off all surfaces. No ground delay.
/// Floor: negate y_vel. Wall: backup 2x, negate x_vel, clear y_vel.
/// Ceiling: negate y_vel. Slope up: swap+negate both. Slope down: swap only.
pub(crate) fn step_big_hopping_skull(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) {
    // Apply gravity (ASM loc_36540: addi.l #$FA0,y_vel)
    enemy.y_velocity = enemy.y_velocity.saturating_add(BIG_SKULL_GRAVITY);

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;

        // Wall collision (ASM loc_366EC: backup 2x, negate x_vel, clear y_vel)
        let moving_right = enemy.x_velocity >= 0;
        let wall = terrain::probe_wall(
            grid,
            cx,
            cy,
            ENEMY_PROBE_HALF_WIDTH,
            ENEMY_PROBE_HEIGHT,
            moving_right,
        );
        if wall.hit {
            // Backup position by 2x x_velocity
            let backup = 2 * i64::from(enemy.x_velocity);
            enemy.x_pos = (enemy.x_pos as i64 - backup).max(0) as u32;
            enemy.y_velocity = 0;
            enemy.x_velocity = -enemy.x_velocity;
            enemy.facing = opposite_direction(enemy.facing);
        }

        // Floor collision (ASM loc_36580: triggers kid-hit state, but for physics
        // the restart at loc_366D0 resets y_vel to $FFFE0000. We negate as bounce.)
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground && enemy.y_velocity > 0 {
            let floor_y_fp = (floor.floor_y_px as u32) << 16;
            if enemy.y_pos >= floor_y_fp {
                enemy.y_pos = floor_y_fp;
                // Floor bounce: negate y_velocity
                enemy.y_velocity = -enemy.y_velocity;
            }

            // Slope deflection: check if floor cell is a slope
            if floor.cell.collision == terrain::CollisionKind::SlopeUp {
                // ASM loc_36706: swap and negate both
                let old_x = enemy.x_velocity;
                let old_y = enemy.y_velocity;
                enemy.y_velocity = -old_x;
                enemy.x_velocity = -old_y;
                enemy.facing = opposite_direction(enemy.facing);
            } else if floor.cell.collision == terrain::CollisionKind::SlopeDown {
                // ASM loc_36720: swap without negate
                let old_x = enemy.x_velocity;
                let old_y = enemy.y_velocity;
                enemy.y_velocity = old_x;
                enemy.x_velocity = old_y;
                enemy.facing = opposite_direction(enemy.facing);
            }
        }

        // Ceiling bounce (ASM loc_366E6: neg.l y_vel)
        let ceil = terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
        if ceil.hit && enemy.y_velocity < 0 {
            enemy.y_velocity = -enemy.y_velocity;
        }
    }
}

#[cfg(test)]
pub(crate) fn step_bouncer(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    let mut rng = RomRandomState::default();
    let camera_x_px = enemy.x_pos >> 16;
    step_bouncer_with_rng(enemy, block_grid, Some(camera_x_px), &mut rng);
}

pub(crate) fn step_bouncer_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    camera_x_px: Option<u32>,
    rng: &mut RomRandomState,
) {
    // Apply gravity
    enemy.y_velocity = enemy.y_velocity.saturating_add(bouncer_gravity(enemy));

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;

        // Wall bounce
        let moving_right = enemy.x_velocity >= 0;
        let wall = terrain::probe_wall(
            grid,
            cx,
            cy,
            ENEMY_PROBE_HALF_WIDTH,
            ENEMY_PROBE_HEIGHT,
            moving_right,
        );
        if wall.hit {
            enemy.x_velocity = -enemy.x_velocity;
            enemy.facing = opposite_direction(enemy.facing);
        }

        // Floor bounce
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground && enemy.y_velocity > 0 {
            // Check for slope deflection first (ASM Sphere.asm loc_36012/loc_36032,
            // Fireball.asm loc_32D06/loc_32D46, Big_Hopping_Skull.asm loc_36706/loc_36720).
            // Slopes take precedence over floor bounce.
            if floor.cell.collision == terrain::CollisionKind::SlopeUp {
                // ASM: swap velocities, negate both
                let old_x = enemy.x_velocity;
                let old_y = enemy.y_velocity;
                enemy.y_velocity = -old_x;
                enemy.x_velocity = -old_y;
                enemy.facing = opposite_direction(enemy.facing);
            } else if floor.cell.collision == terrain::CollisionKind::SlopeDown {
                // ASM: swap velocities without negation
                let old_x = enemy.x_velocity;
                let old_y = enemy.y_velocity;
                enemy.y_velocity = old_x;
                enemy.x_velocity = old_y;
                enemy.facing = opposite_direction(enemy.facing);
            } else {
                // Regular floor bounce
                if matches!(enemy.family, EnemyFamily::Fireball) {
                    enemy.y_velocity =
                        fireball_bounce_y_velocity_with_rng(enemy, rng, camera_x_px.unwrap_or(0));
                } else {
                    enemy.y_velocity = -enemy.y_velocity;
                }
                // Snap out of floor
                let floor_y_fp = (floor.floor_y_px as u32) << 16;
                if enemy.y_pos > floor_y_fp {
                    enemy.y_pos = floor_y_fp;
                }
            }
        }

        // Ceiling bounce
        let ceil = terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
        if ceil.hit && enemy.y_velocity < 0 {
            if matches!(enemy.family, EnemyFamily::Fireball) {
                // ASM Enemy19_Fireball_BounceFloorCeiling reuses the same
                // perturb-and-negate path for both floor and ceiling contact.
                enemy.y_velocity =
                    fireball_bounce_y_velocity_with_rng(enemy, rng, camera_x_px.unwrap_or(0));
            } else {
                // Sphere.asm: negate and halve
                enemy.y_velocity = -(enemy.y_velocity >> 1);
            }
        }
    }
}

/// Dragon Flying script interpreter (Dragon_Flying.asm).
/// Reads commands from one of 3 variant scripts, executing each per-frame.
/// Commands: Hover (idle), Drift (accelerate), DriftWithFire (drift + cone check),
/// FacePlayer (retarget), Descent (dive sub-script), ReturnToAnchor (rise to spawn Y).
pub(crate) fn step_dragon(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    match enemy.dragon_phase {
        DragonFlyingState::None => {}
        DragonFlyingState::Hovering => {
            // Hover: zero velocity, count down frames. ASM loc_34804.
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            enemy.dragon_phase_frames = enemy.dragon_phase_frames.saturating_sub(1);
            if enemy.dragon_phase_frames == 0 {
                dragon_advance_script(enemy, player_x_pos, projectiles);
            }
            dragon_apply_velocity(enemy);
        }
        DragonFlyingState::Drifting | DragonFlyingState::DriftingWithFire => {
            // Drift: accelerate x_velocity toward cap, count down frames.
            // ASM loc_3478C: add/sub $200 per frame, cap at ±$18000.
            if enemy.x_velocity > 0 {
                enemy.x_velocity =
                    (enemy.x_velocity + DRAGON_DRIFT_ACCEL).min(DRAGON_DRIFT_SPEED_CAP);
            } else if enemy.x_velocity < 0 {
                enemy.x_velocity =
                    (enemy.x_velocity - DRAGON_DRIFT_ACCEL).max(-DRAGON_DRIFT_SPEED_CAP);
            }
            // Fire-breath cone check during DriftWithFire (ASM loc_34836).
            if enemy.dragon_phase == DragonFlyingState::DriftingWithFire
                && dragon_fire_breath_cone(enemy, player_x_pos, player_y_pos)
            {
                // Cone hit: enter descent attack.
                dragon_enter_descent(enemy);
                dragon_apply_velocity(enemy);
                return;
            }
            enemy.dragon_phase_frames = enemy.dragon_phase_frames.saturating_sub(1);
            if enemy.dragon_phase_frames == 0 {
                dragon_advance_script(enemy, player_x_pos, projectiles);
            }
            dragon_apply_velocity(enemy);
            // Wall collision during Drift: reverse direction on terrain hit.
            // ASM Dragon_Flying.asm loc_347D6: collision_type check after position update.
            // If terrain collision (not hurt/death), flip direction and negate x_velocity.
            if let Some(grid) = block_grid {
                let cx = (enemy.x_pos >> 16) as i32;
                let cy = (enemy.y_pos >> 16) as i32;
                let moving_right = enemy.x_velocity >= 0;
                let wall = terrain::probe_wall(
                    grid,
                    cx,
                    cy,
                    ENEMY_PROBE_HALF_WIDTH,
                    ENEMY_PROBE_HEIGHT,
                    moving_right,
                );
                if wall.hit {
                    enemy.x_velocity = -enemy.x_velocity;
                    enemy.facing = match enemy.facing {
                        Direction::Right => Direction::Left,
                        Direction::Left => Direction::Right,
                    };
                }
            }
        }
        DragonFlyingState::Descending => {
            step_dragon_descent(enemy, projectiles);
            dragon_apply_velocity(enemy);
        }
        DragonFlyingState::ReturningToAnchor => {
            // Return to anchor: decelerate upward. ASM loc_34B4C.
            enemy.x_velocity = 0;
            let anchor_y = enemy.dragon_anchor_y as i64;
            let current_y = enemy.y_pos as i64;
            if current_y <= anchor_y {
                // Reached anchor: restart script. ASM bra loc_346C4.
                dragon_restart_script(enemy);
            } else {
                // Decelerate upward: subtract $400 per frame from y_vel.
                enemy.y_velocity -= DRAGON_RETURN_DECEL;
                if enemy.y_velocity < DRAGON_RETURN_VEL_CAP_SOFT {
                    // Soft cap exceeded: use hard cap. ASM loc_34B70.
                    enemy.y_velocity = DRAGON_RETURN_VEL_CAP_HARD;
                }
            }
            dragon_apply_velocity(enemy);
        }
    }
}

/// Apply velocity to position (FP16.16 arithmetic).
fn dragon_apply_velocity(enemy: &mut ActiveEnemy) {
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
}

/// Advance to the next command in the dragon's flight script.
/// Immediate commands (FacePlayer, Reverse) execute inline and continue advancing.
fn dragon_advance_script(
    enemy: &mut ActiveEnemy,
    player_x_pos: Option<u32>,
    _projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    let script = dragon_script(enemy.dragon_script_variant);
    loop {
        enemy.dragon_script_cmd += 1;
        if enemy.dragon_script_cmd as usize >= script.len() {
            // Script end: restart. ASM $FFFF sentinel → loc_346C0.
            dragon_restart_script(enemy);
            return;
        }
        match script[enemy.dragon_script_cmd as usize] {
            DragonFlyingCmd::Hover(frames) => {
                enemy.dragon_phase = DragonFlyingState::Hovering;
                enemy.dragon_phase_frames = frames;
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                return;
            }
            DragonFlyingCmd::Drift(frames) => {
                dragon_begin_drift(enemy, frames, false);
                return;
            }
            DragonFlyingCmd::DriftWithFire(frames) => {
                dragon_begin_drift(enemy, frames, true);
                return;
            }
            DragonFlyingCmd::FacePlayer => {
                // Immediate: face player and set initial drift speed. ASM loc_3473A.
                if let Some(px) = player_x_pos {
                    if enemy.x_pos > px {
                        enemy.facing = Direction::Left;
                        enemy.x_velocity = -DRAGON_FACE_PLAYER_SPEED;
                    } else {
                        enemy.facing = Direction::Right;
                        enemy.x_velocity = DRAGON_FACE_PLAYER_SPEED;
                    }
                }
                // Continue to next command immediately (no frame consumed).
            }
            DragonFlyingCmd::Descent => {
                dragon_enter_descent(enemy);
                return;
            }
            DragonFlyingCmd::ReturnToAnchor => {
                enemy.dragon_phase = DragonFlyingState::ReturningToAnchor;
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                return;
            }
        }
    }
}

/// Begin a drift command: set initial speed and direction. ASM loc_34776.
fn dragon_begin_drift(enemy: &mut ActiveEnemy, frames: u16, with_fire: bool) {
    enemy.dragon_phase = if with_fire {
        DragonFlyingState::DriftingWithFire
    } else {
        DragonFlyingState::Drifting
    };
    enemy.dragon_phase_frames = frames;
    // Initial drift speed: $800, direction-dependent. ASM loc_34776.
    enemy.x_velocity = match enemy.facing {
        Direction::Right => DRAGON_DRIFT_INITIAL_SPEED,
        Direction::Left => -DRAGON_DRIFT_INITIAL_SPEED,
    };
}

/// Restart the flight script from the beginning. ASM loc_346C0 → loc_346C4.
fn dragon_restart_script(enemy: &mut ActiveEnemy) {
    enemy.dragon_script_cmd = 0;
    enemy.x_velocity = 0;
    enemy.y_velocity = 0;
    let script = dragon_script(enemy.dragon_script_variant);
    match script[0] {
        DragonFlyingCmd::Hover(frames) => {
            enemy.dragon_phase = DragonFlyingState::Hovering;
            enemy.dragon_phase_frames = frames;
        }
        DragonFlyingCmd::Drift(frames) => dragon_begin_drift(enemy, frames, false),
        DragonFlyingCmd::DriftWithFire(frames) => dragon_begin_drift(enemy, frames, true),
        _ => {
            enemy.dragon_phase = DragonFlyingState::Hovering;
            enemy.dragon_phase_frames = 1;
        }
    }
}

/// Enter the descent attack sub-script. ASM loc_3487E.
fn dragon_enter_descent(enemy: &mut ActiveEnemy) {
    enemy.dragon_phase = DragonFlyingState::Descending;
    enemy.dragon_descent_index = 0;
    enemy.dragon_descent_frames = 0;
    // First entry will be processed immediately in step_dragon_descent.
}

/// Step through the descent attack sub-script (unk_34554).
/// Each entry sets fixed x/y velocities for N frames. Special entries
/// spawn fire, flip direction, or restart the main script.
fn step_dragon_descent(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    loop {
        if enemy.dragon_descent_index as usize >= DRAGON_DIVE_SCRIPT.len() {
            // Shouldn't happen — script ends with RestartScript.
            dragon_restart_script(enemy);
            return;
        }
        match DRAGON_DIVE_SCRIPT[enemy.dragon_descent_index as usize] {
            DragonDiveAction::Move {
                x_vel,
                y_vel,
                frames,
            } => {
                if enemy.dragon_descent_frames == 0 {
                    // Start this entry: set velocities (pixel/frame → FP16.16).
                    let xv = i32::from(x_vel) << 16;
                    let yv = i32::from(y_vel) << 16;
                    // x_vel is for facing-left; negate when facing right. ASM loc_348CE.
                    enemy.x_velocity = if enemy.facing == Direction::Right {
                        -xv
                    } else {
                        xv
                    };
                    enemy.y_velocity = yv;
                    enemy.dragon_descent_frames = frames;
                }
                enemy.dragon_descent_frames -= 1;
                if enemy.dragon_descent_frames == 0 {
                    enemy.dragon_descent_index += 1;
                }
                return; // Consume one frame.
            }
            DragonDiveAction::SpawnFire => {
                // Spawn fire-breath projectile during descent. ASM $E19F.
                if let Some(projs) = projectiles {
                    projs.push(EnemyProjectile {
                        x_pos: enemy.x_pos,
                        y_pos: (enemy.y_pos as i64 + (10 << 16)) as u32,
                        // ASM Dragon_Flying loc_348A0: copy the dragon's live
                        // x_vel into the projectile and force y_vel to +3px/frame.
                        x_velocity: enemy.x_velocity,
                        y_velocity: 3 << 16,
                        lifetime: DRAGON_WALKING_BREATH_LIFETIME,
                        gravity: 0,
                        friendly: false,
                        homing_accel: 0,
                        speed_cap: 0,
                        wall_bounce: false,
                        ..Default::default()
                    });
                }
                enemy.dragon_descent_index += 1;
                // Immediate — continue to next entry without consuming a frame.
            }
            DragonDiveAction::FlipDirection => {
                // Flip facing direction. ASM $DD48.
                enemy.facing = match enemy.facing {
                    Direction::Left => Direction::Right,
                    Direction::Right => Direction::Left,
                };
                enemy.dragon_descent_index += 1;
                // Immediate — continue.
            }
            DragonDiveAction::RestartScript => {
                // Descent complete: restart main flight script. ASM $D8F1.
                dragon_restart_script(enemy);
                return;
            }
        }
    }
}

/// Fire-breath cone check (ASM loc_34836).
/// Returns true if the player is within the dragon's breath cone.
/// Cone geometry: player must be below dragon (Y >= dragon Y), and within
/// a triangular region defined by the relative position and facing direction.
/// The cone is 3:1 aspect (3× lateral distance = vertical threshold).
fn dragon_fire_breath_cone(
    enemy: &ActiveEnemy,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) -> bool {
    let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
        return false;
    };
    let dragon_x = (enemy.x_pos >> 16) as i32;
    let dragon_y = (enemy.y_pos >> 16) as i32;
    let player_x = (px >> 16) as i32;
    let player_y = (py >> 16) as i32;

    // Player must be at or below dragon. ASM: cmp.w y_pos(a4),y_pos(a3) / blt return.
    if player_y < dragon_y {
        return false;
    }

    // Compute lateral distance and cone threshold.
    // ASM: d7 = dragon_x - player_x, then d7 = d7*3 (or d7*2+abs for facing).
    // The exact ASM: d7=dx, d6=dx, d7=d7+d7+d6=3*dx.
    // If facing right: d7 = d7 - d6 then neg → d7 = -(3*dx - dx) = -2*dx.
    // Wait, re-reading: d7=dx, d6=dx, d7+=d7 (d7=2*dx), d7+=d6 (d7=3*dx).
    // If facing right (x_direction set): sub d6 from d7 → d7=2*dx, then neg → -2*dx.
    let dx = dragon_x - player_x;
    let d7 = if enemy.facing == Direction::Right {
        -(dx * 2)
    } else {
        dx * 3
    };

    // Cone Y range: [d7 + dragon_y - 8, d7 + dragon_y].
    // Player Y must be within this range. ASM loc_34860-34878.
    let cone_y = d7 + dragon_y;
    player_y >= (cone_y - 8) && player_y <= cone_y
}

pub(crate) fn enemy_should_remain_active(
    enemy: &ActiveEnemy,
    view: &EnemyLayoutView<'_>,
    window: ActivationWindow,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    active_enemy_count: usize,
) -> bool {
    if enemy.segment_index != view.segment_index {
        return false;
    }
    let Some(object) = view.objects.get(enemy.layout_index) else {
        return false;
    };

    match enemy.family.archetype() {
        // Bosses are always retained — they're confined to their arena.
        EnemyArchetype::Tracker => true,
        EnemyArchetype::Flyer => window.contains_fixed_signed_with_margin(
            enemy.x_pos,
            enemy.y_pos,
            FLYER_RETENTION_X_PADDING,
            FLYER_RETENTION_Y_PADDING,
        ),
        EnemyArchetype::Pacer
            if matches!(enemy.family, EnemyFamily::DragonWalking | EnemyFamily::Lion) =>
        {
            large_pacer_should_remain_active(
                enemy,
                camera_x_px,
                camera_y_px,
                level_bounds,
                active_enemy_count,
            )
        }
        EnemyArchetype::Hopper if matches!(enemy.family, EnemyFamily::MiniHoppingSkull) => {
            mini_hopping_skull_should_remain_active(
                enemy,
                camera_x_px,
                camera_y_px,
                level_bounds,
                active_enemy_count,
            )
        }
        _ => window.contains(u32::from(object.x), u32::from(object.y)),
    }
}

fn large_pacer_should_remain_active(
    enemy: &ActiveEnemy,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    active_enemy_count: usize,
) -> bool {
    // ASM Dragon.asm loc_374E8 and Lion.asm sub_3E8BA both cull the live
    // object from its current world position rather than the layout anchor.
    let x_pos = (enemy.x_pos as i32) >> 16;
    let y_pos = (enemy.y_pos as i32) >> 16;
    let level_x_max = i32::from(level_bounds.width_pixels) + 0x20;
    let level_y_max = i32::from(level_bounds.height_pixels) + 0x20;
    if x_pos <= -0x20 || y_pos <= -0x20 || x_pos > level_x_max || y_pos > level_y_max {
        return false;
    }
    if active_enemy_count <= 10 {
        return true;
    }

    // ASM Dragon.asm loc_37552 / Lion.asm sub_3E8BA:
    // 11..20 active objects use the wider [-$1A4,+$2E4]x[-$1A4,+$284] box,
    // 21+ active objects tighten to [-$104,+$244]x[-$104,+$1E4].
    let (left_margin, right_margin, top_margin, bottom_margin) = if active_enemy_count <= 20 {
        (0x1A4, 0x2E4, 0x1A4, 0x284)
    } else {
        (0x104, 0x244, 0x104, 0x1E4)
    };
    let relative_x = x_pos - camera_x_px as i32;
    let relative_y = y_pos - camera_y_px as i32;
    relative_x >= -left_margin
        && relative_x <= right_margin
        && relative_y >= -top_margin
        && relative_y <= bottom_margin
}

fn mini_hopping_skull_should_remain_active(
    enemy: &ActiveEnemy,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    active_enemy_count: usize,
) -> bool {
    // ASM Mini_Hopping_Skull.asm sub_3F29A culls from the skull's live
    // object position, not its layout anchor. The routine first rejects
    // positions outside [-$20, level+$20] on both axes.
    let x_pos = (enemy.x_pos as i32) >> 16;
    let y_pos = (enemy.y_pos as i32) >> 16;
    let level_x_max = i32::from(level_bounds.width_pixels) + 0x20;
    let level_y_max = i32::from(level_bounds.height_pixels) + 0x20;
    if x_pos <= -0x20 || y_pos <= -0x20 || x_pos > level_x_max || y_pos > level_y_max {
        return false;
    }
    if active_enemy_count <= 10 {
        return true;
    }

    // ASM Mini_Hopping_Skull.asm sub_3F29A:
    // 11..20 active objects use [-$1A4,+$2E4]x[-$1A4,+$284] relative to camera.
    // 21+ active objects tighten to [-$40,+$180]x[-$40,+$120].
    let (left_margin, right_margin, top_margin, bottom_margin) = if active_enemy_count <= 20 {
        (0x1A4, 0x2E4, 0x1A4, 0x284)
    } else {
        (0x40, 0x180, 0x40, 0x120)
    };
    let relative_x = x_pos - camera_x_px as i32;
    let relative_y = y_pos - camera_y_px as i32;
    relative_x >= -left_margin
        && relative_x <= right_margin
        && relative_y >= -top_margin
        && relative_y <= bottom_margin
}

fn mini_hopping_skull_snap_slopeup_x(grid: &terrain::BlockGrid, x_px: i32, y_px: i32) -> u32 {
    let row = terrain::pixel_to_cell(y_px).max(0) as u16;
    let start_col = terrain::pixel_to_cell(x_px).max(0) as u16;

    for col in start_col..grid.width_cells {
        let cell = grid.cell_at(col, row);
        if cell.collision == terrain::CollisionKind::SlopeUp {
            // ASM Mini_Hopping_Skull.asm sub_3F366: `addi.w #$10,d6` before the
            // collision check means the snap uses the matched slope cell's right
            // edge, then subtracts the low nibble of the current Y position.
            let snapped_x_px = ((i32::from(col) + 1) * 16) - (y_px & 0x0F);
            return (snapped_x_px.max(0) as u32) << 16;
        }
    }

    (x_px.max(0) as u32) << 16
}

fn mini_hopping_skull_snap_slopedown_x(grid: &terrain::BlockGrid, x_px: i32, y_px: i32) -> u32 {
    let row = terrain::pixel_to_cell(y_px).max(0) as u16;
    let start_col = terrain::pixel_to_cell(x_px).max(0) + 1;

    for col in (0..=start_col).rev() {
        let cell = grid.cell_at(col as u16, row);
        if cell.collision == terrain::CollisionKind::SlopeDown {
            // ASM Mini_Hopping_Skull.asm sub_3F3A6: `addi.w #2,a4` advances to
            // the next cell on the right before each loop subtracts 16px, so
            // the final snap uses the matched slope cell's left edge plus the
            // current Y low nibble.
            let snapped_x_px = (col * 16) + (y_px & 0x0F);
            return (snapped_x_px.max(0) as u32) << 16;
        }
    }

    (x_px.max(0) as u32) << 16
}

pub(crate) fn step_hopper(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
) {
    let Some(grid) = block_grid else {
        return;
    };
    let hc = hopper_constants(enemy.family);

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    if enemy.on_ground {
        let mut facing =
            player_relative_direction(enemy.x_pos, player_x_pos).unwrap_or(enemy.facing);
        if hopper_should_reverse(facing, grid, cx, cy) {
            facing = opposite_direction(facing);
        }
        enemy.facing = facing;
        if enemy.grounded_frames > 0 {
            enemy.grounded_frames -= 1;
            enemy.x_velocity = 0;
            return;
        }
        let mut jx = hc.jump_x_speed;
        let mut jy = hc.jump_y_speed;
        // Mini_Hopping_Skull.asm lines 191-194: enemy_level ≥ 1 (HP > 1) boosts hop.
        if matches!(enemy.family, EnemyFamily::MiniHoppingSkull) && enemy.hit_points > 1 {
            jx += 0x4000;
            jy -= 0x1_0000;
        }
        enemy.x_velocity = horizontal_velocity(enemy.facing, jx);
        enemy.y_velocity = jy;
        enemy.on_ground = false;
    }

    enemy.y_velocity = (enemy.y_velocity + hc.gravity).min(physics::MAX_FALL_SPEED);
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let moving_right = enemy.x_velocity >= 0;
    let wall = terrain::probe_wall(
        grid,
        cx,
        cy,
        ENEMY_PROBE_HALF_WIDTH,
        ENEMY_PROBE_HEIGHT,
        moving_right,
    );
    if wall.hit {
        reverse_enemy_direction(enemy);
        let wall_fp = (wall.wall_x_px as u32) << 16;
        let edge_fp = (ENEMY_PROBE_HALF_WIDTH as u32) << 16;
        if moving_right {
            if enemy.x_pos + edge_fp > wall_fp {
                enemy.x_pos = wall_fp.saturating_sub(edge_fp);
            }
        } else if enemy.x_pos < wall_fp + edge_fp {
            enemy.x_pos = wall_fp + edge_fp;
        }
        enemy.x_velocity = horizontal_velocity(enemy.facing, hc.jump_x_speed);
    }

    let floor = terrain::probe_floor(grid, cx, cy);
    if floor.on_ground {
        let floor_y_fp = (floor.floor_y_px as u32) << 16;
        if enemy.y_velocity >= 0 && enemy.y_pos >= floor_y_fp {
            if floor.cell.collision == terrain::CollisionKind::SlopeUp {
                enemy.x_pos = mini_hopping_skull_snap_slopeup_x(
                    grid,
                    (enemy.x_pos >> 16) as i32,
                    (enemy.y_pos >> 16) as i32,
                );
            } else if floor.cell.collision == terrain::CollisionKind::SlopeDown {
                enemy.x_pos = mini_hopping_skull_snap_slopedown_x(
                    grid,
                    (enemy.x_pos >> 16) as i32,
                    (enemy.y_pos >> 16) as i32,
                );
            }
            enemy.y_pos = floor_y_fp;
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            enemy.on_ground = true;
            enemy.grounded_frames = hc.ground_delay;
        }
    } else {
        enemy.on_ground = false;
    }
}

pub(crate) fn hopper_should_reverse(
    facing: Direction,
    grid: &terrain::BlockGrid,
    cx: i32,
    cy: i32,
) -> bool {
    let moving_right = matches!(facing, Direction::Right);
    let wall = terrain::probe_wall(
        grid,
        cx,
        cy,
        ENEMY_PROBE_HALF_WIDTH,
        ENEMY_PROBE_HEIGHT,
        moving_right,
    );
    if wall.hit {
        return true;
    }
    // Reverse at platform edge: look ahead for floor support
    let look_ahead_x = if moving_right {
        cx + ENEMY_PROBE_HALF_WIDTH
    } else {
        cx - ENEMY_PROBE_HALF_WIDTH
    };
    !terrain::probe_floor(grid, look_ahead_x, cy).on_ground
}

/// Step a Pacer-archetype enemy: walk at a fixed speed, reverse on wall or
/// floor edge. Matches the shared walker subroutines in the disassembly
/// (loc_3E392 / loc_3E546 / loc_3E560 / loc_3E3AC).
pub(crate) fn step_pacer(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    let Some(grid) = block_grid else {
        return;
    };

    let walk_speed = if enemy.walk_speed_override != 0 {
        enemy.walk_speed_override
    } else {
        pacer_walk_speed(enemy.family, enemy.hit_points)
    };
    enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
    enemy.y_velocity = 0;

    // Apply horizontal movement.
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let moving_right = enemy.x_velocity >= 0;

    // Wall probe: reverse if we hit a wall.
    let wall = terrain::probe_wall(
        grid,
        cx,
        cy,
        ENEMY_PROBE_HALF_WIDTH,
        ENEMY_PROBE_HEIGHT,
        moving_right,
    );
    if wall.hit {
        // Push out of wall.
        let wall_fp = (wall.wall_x_px as u32) << 16;
        let edge_fp = (ENEMY_PROBE_HALF_WIDTH as u32) << 16;
        if moving_right {
            if enemy.x_pos + edge_fp > wall_fp {
                enemy.x_pos = wall_fp.saturating_sub(edge_fp);
            }
        } else if enemy.x_pos < wall_fp + edge_fp {
            enemy.x_pos = wall_fp + edge_fp;
        }
        reverse_enemy_direction(enemy);
        enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
        return;
    }

    // Edge probe: reverse if there's no floor ahead.
    let look_ahead_x = if moving_right {
        cx + ENEMY_PROBE_HALF_WIDTH
    } else {
        cx - ENEMY_PROBE_HALF_WIDTH
    };
    if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
        reverse_enemy_direction(enemy);
        enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
    }
}

/// Driller probe half-width for block drilling (Driller.asm line 27: move.w #$C,$48(a5)).
/// ASM stores 12px ($C) in the Driller sprite's collision half-width field.
pub(crate) const DRILLER_PROBE_HALF_WIDTH: i32 = 12;

/// Step a Driller enemy: paces like normal, but drills through Smash and Ice blocks.
/// ASM loc_3BE78: when sprite timer matches direction (8 for left, 4 for right),
/// checks cell at (sprite_x ± offset, sprite_y - 16) for Smash/Ice type and destroys it.
/// Returns list of (col, row) cells to destroy this frame.
pub(crate) fn step_driller(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) -> Vec<(u16, u16)> {
    let mut drilled = Vec::new();
    let Some(grid) = block_grid else {
        return drilled;
    };

    // Pace first
    step_pacer(enemy, block_grid);

    // Now check for drillable blocks in the walking direction.
    // ASM loc_3BE8E: get sprite position, check timer, probe cell 16px above.
    // Simplified: drill every frame when walking into Smash/Ice blocks.
    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let probe_x = if enemy.facing == Direction::Right {
        cx + DRILLER_PROBE_HALF_WIDTH
    } else {
        cx - DRILLER_PROBE_HALF_WIDTH
    };
    // ASM sub_3BEF2: Y probe is 16px above sprite (subi.w #$10,d7).
    let probe_y = cy - 16;

    let col = terrain::pixel_to_cell(probe_x).max(0) as u16;
    let row = terrain::pixel_to_cell(probe_y).max(0) as u16;
    let cell = grid.cell_at(col, row);

    // ASM loc_3BEF2: check block type. $100 = Smash, $200 = Ice.
    // Any block with collision Solid + has_block + block_type Smash/Ice.
    if cell.has_block
        && cell.collision == terrain::CollisionKind::Solid
        && (cell.block_type == terrain::BlockType::Smash
            || cell.block_type == terrain::BlockType::Ice)
    {
        drilled.push((col, row));
    }

    drilled
}

/// Ninja detection cone: player must be 0x20–0x50 pixels above and ±0x10 pixels
/// horizontally (Ninja.asm lines 148-161).
pub(crate) const NINJA_DETECT_Y_MIN: i32 = 0x20;
pub(crate) const NINJA_DETECT_Y_MAX: i32 = 0x50;
pub(crate) const NINJA_DETECT_X_RANGE: i32 = 0x10;

/// Ninja jump-attack velocities (Ninja.asm lines 384-392).
pub(crate) const NINJA_JUMP_X_SPEED_LEFT: i32 = 0x1_8000;
pub(crate) const NINJA_JUMP_X_SPEED_RIGHT: i32 = 0x1_0000;
pub(crate) const NINJA_JUMP_Y_SPEED: i32 = -0x4_8000;
pub(crate) const NINJA_GRAVITY: i32 = 0x2800;

/// Step a Ninja enemy: stealth-paces until player enters detection cone,
/// then faces player and jump-attacks. Returns to stealth on landing.
pub(crate) fn step_ninja(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    player_form: PlayerForm,
) {
    match enemy.ninja_state {
        NinjaState::Stealth => {
            // Pace like normal
            step_pacer(enemy, block_grid);

            // Check detection cone (skip if player wears Red Stealth)
            if player_form == PlayerForm::RedStealth {
                return;
            }
            let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
                return;
            };
            let dx = (px as i64 - enemy.x_pos as i64) >> 16;
            let dy = (enemy.y_pos as i64 - py as i64) >> 16;

            // Player must be 0x20–0x50 px above ninja and within ±0x10 px horizontally
            if dy >= NINJA_DETECT_Y_MIN as i64
                && dy <= NINJA_DETECT_Y_MAX as i64
                && dx.abs() <= NINJA_DETECT_X_RANGE as i64
            {
                // Face the player
                enemy.facing = if dx >= 0 {
                    Direction::Right
                } else {
                    Direction::Left
                };
                // Launch jump attack
                enemy.ninja_state = NinjaState::JumpAttack;
                enemy.x_velocity = match enemy.facing {
                    Direction::Left => -NINJA_JUMP_X_SPEED_LEFT,
                    Direction::Right => NINJA_JUMP_X_SPEED_RIGHT,
                };
                enemy.y_velocity = NINJA_JUMP_Y_SPEED;
                enemy.on_ground = false;
            }
        }
        NinjaState::JumpAttack => {
            // Apply gravity
            enemy.y_velocity = (enemy.y_velocity + NINJA_GRAVITY).min(physics::MAX_FALL_SPEED);

            // Move
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
            enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

            let Some(grid) = block_grid else {
                return;
            };
            let cx = (enemy.x_pos >> 16) as i32;
            let cy = (enemy.y_pos >> 16) as i32;

            // Wall bounce: reverse x on wall hit
            let moving_right = enemy.x_velocity >= 0;
            let wall = terrain::probe_wall(
                grid,
                cx,
                cy,
                ENEMY_PROBE_HALF_WIDTH,
                ENEMY_PROBE_HEIGHT,
                moving_right,
            );
            if wall.hit {
                enemy.x_velocity = -enemy.x_velocity;
                enemy.facing = opposite_direction(enemy.facing);
            }

            // Floor landing: return to stealth
            if enemy.y_velocity > 0 {
                let floor = terrain::probe_floor(grid, cx, cy);
                if floor.on_ground {
                    let floor_y_fp = (floor.floor_y_px as u32) << 16;
                    if enemy.y_pos >= floor_y_fp {
                        enemy.y_pos = floor_y_fp;
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.on_ground = true;
                        enemy.ninja_state = NinjaState::Stealth;
                    }
                }
            }
        }
    }
}

/// Hand detection ranges by HP tier (Hand.asm unk_3276A table).
/// HP 1 = 64px, HP 2 = 80px, HP 3+ = 100px.
pub(crate) const HAND_DETECT_RANGE: [i32; 3] = [0x40, 0x50, 0x64];

/// Hand walk velocities by HP tier (Hand.asm unk_3276A table).
pub(crate) const HAND_WALK_SPEED: [i32; 3] = [0x3000, 0x4800, 0x6000];

/// Hand jump-attack gravity (Hand.asm line 130: $4000 per frame).
pub(crate) const HAND_GRAVITY: i32 = 0x4000;

/// Hand jump velocity: randomised per-jump from ASM (Hand.asm lines 128-147).
/// Y: -(tier + random(0..3)) × $10000, X: ±(tier + random(0..3)) × $10000.
/// Random component is `(random_byte XOR kid_pos) & 3`.
/// We approximate with a frame-based pseudo-random in the range.
pub(crate) fn hand_jump_speed(tier: usize, frame_counter: u32) -> (i32, i32) {
    // Approximate the ASM's `random_byte XOR kid_pos & 3` pattern
    let rand_y = ((frame_counter.wrapping_mul(7)) & 3) as i32;
    let rand_x = ((frame_counter.wrapping_mul(13).wrapping_add(5)) & 3) as i32;
    let t = (tier + 1) as i32; // tier 0 = HP 1 → multiplier 1
    let y_speed = -(t + rand_y) * 0x1_0000;
    let x_speed = (t + rand_x) * 0x1_0000;
    (x_speed, y_speed)
}

/// Hand max fall speed: $180000 (Hand.asm lines 728-730), NOT the shared MAX_FALL_SPEED.
pub(crate) const HAND_MAX_FALL_SPEED: i32 = 0x18_0000;

/// Hand grab duration: 10 frames (Hand.asm loc_32906: moveq #$A,d3).
pub(crate) const HAND_GRAB_DURATION: u16 = 10;

/// Step a Hand enemy through its walk/detect/jump state machine.
/// Walking: paces with edge/wall detection. Detects player in 45-degree cone.
/// Jumping: randomised arc toward player with gravity, lands and resumes walking.
/// ASM: Hand.asm — full state machine with cone detection, random jump, gravity cap.
pub(crate) fn step_hand(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    let tier = (enemy.hit_points.saturating_sub(1) as usize).min(2);

    match enemy.hand_state {
        HandState::Walking => {
            // Walk like a pacer with Hand-specific speed
            let walk_speed = HAND_WALK_SPEED[tier];
            if let Some(grid) = block_grid {
                enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
                enemy.y_velocity = 0;
                enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

                let cx = (enemy.x_pos >> 16) as i32;
                let cy = (enemy.y_pos >> 16) as i32;
                let moving_right = enemy.x_velocity >= 0;

                let wall = terrain::probe_wall(
                    grid,
                    cx,
                    cy,
                    ENEMY_PROBE_HALF_WIDTH,
                    ENEMY_PROBE_HEIGHT,
                    moving_right,
                );
                if wall.hit {
                    let wall_fp = (wall.wall_x_px as u32) << 16;
                    let edge_fp = (ENEMY_PROBE_HALF_WIDTH as u32) << 16;
                    if moving_right {
                        if enemy.x_pos + edge_fp > wall_fp {
                            enemy.x_pos = wall_fp.saturating_sub(edge_fp);
                        }
                    } else if enemy.x_pos < wall_fp + edge_fp {
                        enemy.x_pos = wall_fp + edge_fp;
                    }
                    reverse_enemy_direction(enemy);
                    return;
                }

                let look_ahead_x = if moving_right {
                    cx + ENEMY_PROBE_HALF_WIDTH
                } else {
                    cx - ENEMY_PROBE_HALF_WIDTH
                };
                if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
                    reverse_enemy_direction(enemy);
                }
            }

            // Player detection: forward cone + 16px vertical + 45-degree angle
            let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
                return;
            };
            let dx = (px as i64 - enemy.x_pos as i64) >> 16;
            let dy = (enemy.y_pos as i64 - py as i64) >> 16;

            // Player must be 0-16px above Hand
            if !(0..=16).contains(&dy) {
                return;
            }

            let detect_range = i64::from(HAND_DETECT_RANGE[tier]);
            let in_range = match enemy.facing {
                Direction::Right => dx > 0 && dx <= detect_range,
                Direction::Left => dx < 0 && (-dx) <= detect_range,
            };

            // 45-degree cone constraint: |dy| <= |dx| (Hand.asm loc_3284C lines 99-114)
            let in_cone = in_range && dy.abs() <= dx.abs();

            if in_cone {
                // Jump toward player with randomised velocity (Hand.asm lines 128-147)
                enemy.hand_state = HandState::Jumping;
                enemy.on_ground = false;
                let (x_speed, y_speed) = hand_jump_speed(tier, enemy.wait_timer as u32);
                enemy.x_velocity = if dx >= 0 { x_speed } else { -x_speed };
                enemy.y_velocity = y_speed;
            }
        }
        HandState::Jumping => {
            // Apply gravity, capped at Hand-specific $180000 (not shared MAX_FALL_SPEED)
            enemy.y_velocity = (enemy.y_velocity + HAND_GRAVITY).min(HAND_MAX_FALL_SPEED);

            // Move
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
            enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

            let Some(grid) = block_grid else {
                return;
            };
            let cx = (enemy.x_pos >> 16) as i32;
            let cy = (enemy.y_pos >> 16) as i32;

            // Wall reversal
            let moving_right = enemy.x_velocity >= 0;
            let wall = terrain::probe_wall(
                grid,
                cx,
                cy,
                ENEMY_PROBE_HALF_WIDTH,
                ENEMY_PROBE_HEIGHT,
                moving_right,
            );
            if wall.hit {
                enemy.x_velocity = 0;
            }

            // Floor landing: check for grab, then return to walking
            if enemy.y_velocity > 0 {
                let floor = terrain::probe_floor(grid, cx, cy);
                if floor.on_ground {
                    let floor_y_fp = (floor.floor_y_px as u32) << 16;
                    if enemy.y_pos >= floor_y_fp {
                        enemy.y_pos = floor_y_fp;
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.on_ground = true;

                        // On landing, check if player is within grab range
                        // (Hand.asm loc_3284C — same range check as jump trigger)
                        let grab = if let (Some(px), Some(py)) = (player_x_pos, player_y_pos) {
                            let dx = ((px as i64 - enemy.x_pos as i64) >> 16).abs();
                            let dy = ((enemy.y_pos as i64 - py as i64) >> 16).abs();
                            dx <= i64::from(HAND_DETECT_RANGE[tier]) && dy <= 16
                        } else {
                            false
                        };

                        if grab {
                            // Enter grab: follow player for 10 frames (Hand.asm loc_32906)
                            enemy.hand_state = HandState::Grabbing;
                            enemy.wait_timer = HAND_GRAB_DURATION;
                        } else {
                            enemy.hand_state = HandState::Walking;
                        }
                    }
                }
            }
        }
        HandState::Grabbing => {
            // Hand follows player position for HAND_GRAB_DURATION frames.
            // Hand.asm loc_3292E: hand snaps to kid's X±2, Y-4 each frame.
            if let (Some(px), Some(py)) = (player_x_pos, player_y_pos) {
                let x_off: i64 = match enemy.facing {
                    Direction::Right => 2 << 16,
                    Direction::Left => -(2i64 << 16),
                };
                enemy.x_pos = (px as i64 + x_off) as u32;
                enemy.y_pos = (py as i64 - (4i64 << 16)) as u32;
            }
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;

            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
            } else {
                enemy.hand_state = HandState::Walking;
            }
        }
    }
}

/// Lion charge speed: $28000 for HP==8, $18000 otherwise (Lion.asm lines 36-48).
/// Also used as walk speed — ASM sets both $50(a5) and $5E(a5) from same value.
pub(crate) fn lion_charge_speed(hit_points: u8) -> i32 {
    if hit_points >= 8 { 0x2_8000 } else { 0x1_8000 }
}

/// Lion charge timer: starts at $3C (60 frames), counts down while pacing.
/// ASM loc_3E078: on expiry, unconditionally starts roar → charge regardless
/// of Kid distance.  Reload is $B4 (180) via LION_COOLDOWN_AFTER_FIRE.
pub(crate) const LION_CHARGE_TIMER_INIT: u16 = 60;

/// Lion post-projectile cooldown: $B4 (180 frames) before next charge after
/// firing a projectile (Lion.asm loc_3E096).
pub(crate) const LION_COOLDOWN_AFTER_FIRE: u16 = 180;

/// Lion charge overshoot distance for termination: $70 (112px).
/// ASM does not use a fixed duration — charge ends when Lion overshoots
/// the player by >112px (loc_3E450/3E604).
pub(crate) const LION_CHARGE_OVERSHOOT: i32 = 0x70;

/// Lion projectile speed (Lion.asm line 165: $60000).
pub(crate) const LION_PROJECTILE_SPEED: i32 = 0x6_0000;

// Lion homing accel is $2000/frame (Lion.asm loc_3E1C2) — set on projectile.
// Wall-bounce: Lion.asm loc_3E260 — reverses velocity on terrain collision.
// Speed cap: ±$60000 on both axes (Lion.asm loc_3E1C2).

/// Lion projectile spawn X offset: 10px ahead in facing direction.
pub(crate) const LION_PROJECTILE_X_OFFSET: i32 = 10;

/// Lion projectile spawn Y offset: 24px above Lion's Y position ($18).
pub(crate) const LION_PROJECTILE_Y_OFFSET: i32 = 24;

fn lion_active_projectile_count(enemy: &ActiveEnemy) -> u8 {
    enemy.ufo_beam_segments
}

fn increment_lion_active_projectile_count(enemy: &mut ActiveEnemy) {
    enemy.ufo_beam_segments = enemy.ufo_beam_segments.saturating_add(1);
}

/// Step a Lion enemy: paces normally, then charges toward player and fires a
/// homing projectile. Charge ends on wall/edge or player overshoot >112px.
/// ASM: Lion.asm — full state machine with detection, charge, and cooldown.
pub(crate) fn step_lion(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    match enemy.lion_state {
        LionState::Walking => {
            // Pace like normal
            step_pacer(enemy, block_grid);

            // Count down charge timer.  ASM loc_3E078: timer always ticks and
            // the charge fires unconditionally on expiry (no Kid distance gate).
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
                return;
            }

            // ASM loc_3E078: after reloading $54(a5) to $B4, a live projectile
            // count in $5B(a5) suppresses the next roar/fire cycle.
            if lion_active_projectile_count(enemy) > 0 {
                enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                return;
            }

            // Timer expired — face the player and start charging.
            // ASM loc_3E0D6/loc_3E0DA: after roar, always turns toward Kid.
            let Some(px) = player_x_pos else {
                // No player info (shouldn't happen in practice) — reload.
                enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                return;
            };
            let dx = (px as i64 - enemy.x_pos as i64) >> 16;

            // Face the player and start charging
            enemy.facing = if dx >= 0 {
                Direction::Right
            } else {
                Direction::Left
            };
            enemy.lion_state = LionState::Charging;

            // Fire a homing projectile toward the player (Lion.asm loc_3E12A).
            // Spawns 10px ahead in X, 24px above in Y.
            if let Some(projs) = projectiles {
                let speed = LION_PROJECTILE_SPEED;
                let (xv, x_off) = match enemy.facing {
                    Direction::Right => (speed, LION_PROJECTILE_X_OFFSET),
                    Direction::Left => (-speed, -LION_PROJECTILE_X_OFFSET),
                };
                let proj_x = (enemy.x_pos as i64 + ((x_off as i64) << 16)) as u32;
                let proj_y =
                    (enemy.y_pos as i64 - ((LION_PROJECTILE_Y_OFFSET as i64) << 16)) as u32;
                projs.push(EnemyProjectile {
                    x_pos: proj_x,
                    y_pos: proj_y,
                    x_velocity: xv,
                    y_velocity: 0,
                    lifetime: 120,
                    gravity: 0,
                    friendly: false,
                    homing_accel: 0x2000,             // Lion.asm loc_3E1C2
                    speed_cap: LION_PROJECTILE_SPEED, // ±$60000 cap (Lion.asm)
                    wall_bounce: true,                // Lion.asm loc_3E260
                    owner: Some(EnemyProjectileOwner {
                        segment_index: enemy.segment_index,
                        layout_index: enemy.layout_index,
                        token: enemy.ufo_beam_interval,
                    }),
                    ..Default::default()
                });
                increment_lion_active_projectile_count(enemy);
            }

            // Post-fire cooldown is longer: $B4 (180 frames) at loc_3E096.
            // The cooldown is applied when returning to walk after charge.
        }
        LionState::Charging => {
            // Move at charge speed
            let speed = lion_charge_speed(enemy.hit_points);
            enemy.x_velocity = horizontal_velocity(enemy.facing, speed);
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

            // Wall check
            if let Some(grid) = block_grid {
                let cx = (enemy.x_pos >> 16) as i32;
                let cy = (enemy.y_pos >> 16) as i32;
                let moving_right = enemy.x_velocity >= 0;
                let wall = terrain::probe_wall(
                    grid,
                    cx,
                    cy,
                    ENEMY_PROBE_HALF_WIDTH,
                    ENEMY_PROBE_HEIGHT,
                    moving_right,
                );
                if wall.hit {
                    reverse_enemy_direction(enemy);
                    enemy.lion_state = LionState::Walking;
                    enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                    return;
                }

                // Edge check
                let look_ahead_x = if moving_right {
                    cx + ENEMY_PROBE_HALF_WIDTH
                } else {
                    cx - ENEMY_PROBE_HALF_WIDTH
                };
                if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
                    reverse_enemy_direction(enemy);
                    enemy.lion_state = LionState::Walking;
                    enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                    return;
                }
            }

            // Overshoot check: if Lion has passed player by >112px, stop charging
            // (Lion.asm loc_3E450/3E604 — no fixed duration timer).
            if let Some(px) = player_x_pos {
                let dx = match enemy.facing {
                    Direction::Right => (enemy.x_pos as i64 - px as i64) >> 16,
                    Direction::Left => (px as i64 - enemy.x_pos as i64) >> 16,
                };
                if dx > i64::from(LION_CHARGE_OVERSHOOT) {
                    enemy.lion_state = LionState::Walking;
                    enemy.wait_timer = LION_COOLDOWN_AFTER_FIRE;
                }
            }
        }
    }
}

/// Robot walk speed: HP 2 = $8000, HP 3+ = $10000 (Robot.asm lines 28-36).
pub(crate) fn robot_walk_speed(hit_points: u8) -> i32 {
    if hit_points >= 3 { 0x1_0000 } else { 0x8000 }
}

/// Robot charge speed: always $28000 (Robot.asm line 157).
pub(crate) const ROBOT_CHARGE_SPEED: i32 = 0x2_8000;

/// Robot shoot interval: ASM Robot.asm init sets $4A(a5) = $20 = 32 frames.
pub(crate) const ROBOT_SHOOT_INTERVAL: u16 = 32;

/// Robot charge direction-flip timer base: ASM Robot.asm sets $54(a5) = $F = 15.
/// Reloads to (random_byte >> 1) + $F (range 15-142) on each flip.
/// The charge is permanent until wall hit — no timed exit.
pub(crate) const ROBOT_CHARGE_FLIP_BASE: u16 = 15;

pub(crate) fn robot_charge_flip_reload_with_rng(rng: &mut RomRandomState) -> u16 {
    // ASM loc_3A9F0: `Get_RandomNumber_byte`, `lsr.w #1,d7`, `addi.w #$F,d7`.
    let random_byte = rng.next_byte();
    u16::from(random_byte >> 1) + ROBOT_CHARGE_FLIP_BASE
}

/// Robot projectile speed: ~5 px/frame horizontal (Robot.asm).
pub(crate) const ROBOT_PROJECTILE_SPEED: i32 = 0x5_0000;

/// Step a Robot enemy: paces at HP-dependent speed, shoots at intervals,
/// then enters a fast charge phase.
#[cfg(test)]
pub(crate) fn step_robot(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
) {
    let mut rng = RomRandomState::default();
    step_robot_with_rng(enemy, block_grid, player_x_pos, projectiles, &mut rng);
}

pub(crate) fn step_robot_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
    rng: &mut RomRandomState,
) {
    match enemy.robot_state {
        RobotState::Walking => {
            let walk_speed = robot_walk_speed(enemy.hit_points);
            // Walk like a pacer with robot-specific speed
            if let Some(grid) = block_grid {
                enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
                enemy.y_velocity = 0;
                enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

                let cx = (enemy.x_pos >> 16) as i32;
                let cy = (enemy.y_pos >> 16) as i32;
                let moving_right = enemy.x_velocity >= 0;

                let wall = terrain::probe_wall(
                    grid,
                    cx,
                    cy,
                    ENEMY_PROBE_HALF_WIDTH,
                    ENEMY_PROBE_HEIGHT,
                    moving_right,
                );
                if wall.hit {
                    let wall_fp = (wall.wall_x_px as u32) << 16;
                    let edge_fp = (ENEMY_PROBE_HALF_WIDTH as u32) << 16;
                    if moving_right {
                        if enemy.x_pos + edge_fp > wall_fp {
                            enemy.x_pos = wall_fp.saturating_sub(edge_fp);
                        }
                    } else if enemy.x_pos < wall_fp + edge_fp {
                        enemy.x_pos = wall_fp + edge_fp;
                    }
                    reverse_enemy_direction(enemy);
                    return;
                }

                let look_ahead_x = if moving_right {
                    cx + ENEMY_PROBE_HALF_WIDTH
                } else {
                    cx - ENEMY_PROBE_HALF_WIDTH
                };
                if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
                    reverse_enemy_direction(enemy);
                }
            }

            // Shoot timer
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
            } else {
                // Face player if available
                if let Some(px) = player_x_pos {
                    let dx = px as i64 - enemy.x_pos as i64;
                    enemy.facing = if dx >= 0 {
                        Direction::Right
                    } else {
                        Direction::Left
                    };
                }

                // Fire projectile
                if let Some(projs) = projectiles {
                    let speed = ROBOT_PROJECTILE_SPEED;
                    let xv = match enemy.facing {
                        Direction::Right => speed,
                        Direction::Left => -speed,
                    };
                    projs.push(EnemyProjectile {
                        x_pos: enemy.x_pos,
                        y_pos: enemy.y_pos,
                        x_velocity: xv,
                        y_velocity: 0,
                        lifetime: 60,
                        gravity: 0,
                        friendly: false,
                        homing_accel: 0,
                        speed_cap: 0,
                        wall_bounce: false,
                        ..Default::default()
                    });
                }

                // Enter charge mode — ASM: permanent until wall hit
                enemy.robot_state = RobotState::Charging;
                enemy.wait_timer = ROBOT_CHARGE_FLIP_BASE;
            }
        }
        RobotState::Charging => {
            // ASM Robot.asm: charge is permanent until wall hit.
            // Direction-flip timer fires periodically; Robot stays charging.
            enemy.x_velocity = horizontal_velocity(enemy.facing, ROBOT_CHARGE_SPEED);
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

            if let Some(grid) = block_grid {
                let cx = (enemy.x_pos >> 16) as i32;
                let cy = (enemy.y_pos >> 16) as i32;
                let moving_right = enemy.x_velocity >= 0;
                let wall = terrain::probe_wall(
                    grid,
                    cx,
                    cy,
                    ENEMY_PROBE_HALF_WIDTH,
                    ENEMY_PROBE_HEIGHT,
                    moving_right,
                );
                if wall.hit {
                    // Wall hit ends charge → return to walking
                    reverse_enemy_direction(enemy);
                    enemy.robot_state = RobotState::Walking;
                    enemy.wait_timer = ROBOT_SHOOT_INTERVAL;
                    return;
                }
                let look_ahead_x = if moving_right {
                    cx + ENEMY_PROBE_HALF_WIDTH
                } else {
                    cx - ENEMY_PROBE_HALF_WIDTH
                };
                if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
                    // Edge → also exit charge
                    reverse_enemy_direction(enemy);
                    enemy.robot_state = RobotState::Walking;
                    enemy.wait_timer = ROBOT_SHOOT_INTERVAL;
                    return;
                }
            }

            // Direction-flip timer: flip direction, stay charging
            // ASM reload: (random_byte >> 1) + $F (range 15-142)
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
            } else {
                reverse_enemy_direction(enemy);
                enemy.wait_timer = robot_charge_flip_reload_with_rng(rng);
            }
        }
    }
}

/// Orca proximity detection range (Orca.asm loc_3D2DE: ~$30 px).
pub(crate) const ORCA_DETECT_RANGE: i32 = 0x30;

/// Orca lunge velocities (Orca.asm).
pub(crate) const ORCA_LUNGE_X_SPEED: i32 = 0x3_0000;
pub(crate) const ORCA_LUNGE_Y_SPEED: i32 = -0x2_8000;

/// Orca lunge gravity (Orca.asm line loc_3D3B4: $1800 per frame).
pub(crate) const ORCA_LUNGE_GRAVITY: i32 = 0x1800;

/// Orca lunge X acceleration (Orca.asm loc_3D3A2: $200 per frame in facing dir).
pub(crate) const ORCA_LUNGE_X_ACCEL: i32 = 0x200;

/// Orca lunge cooldown (Orca.asm: $12C = 300 frames between lunges).
pub(crate) const ORCA_LUNGE_COOLDOWN: u16 = 300;

/// Step an Orca enemy: paces at $8000, detects nearby player, lunges in a
/// ballistic arc, lands and resumes walking with a cooldown.
pub(crate) fn step_orca(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
) {
    match enemy.orca_state {
        OrcaState::Walking => {
            step_pacer(enemy, block_grid);

            // Cooldown between lunges
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
                return;
            }

            // Proximity detection
            let Some(px) = player_x_pos else {
                return;
            };
            let dx = (px as i64 - enemy.x_pos as i64) >> 16;
            if dx.abs() > i64::from(ORCA_DETECT_RANGE) {
                return;
            }

            // Launch lunge
            enemy.orca_state = OrcaState::Lunging;
            enemy.on_ground = false;
            enemy.facing = if dx >= 0 {
                Direction::Right
            } else {
                Direction::Left
            };
            enemy.x_velocity = match enemy.facing {
                Direction::Right => ORCA_LUNGE_X_SPEED,
                Direction::Left => -ORCA_LUNGE_X_SPEED,
            };
            enemy.y_velocity = ORCA_LUNGE_Y_SPEED;
        }
        OrcaState::Lunging => {
            // Gravity (Orca.asm loc_3D3B4: $1800 per frame)
            enemy.y_velocity = (enemy.y_velocity + ORCA_LUNGE_GRAVITY).min(physics::MAX_FALL_SPEED);

            // X acceleration during lunge (Orca.asm loc_3D3A2: $200 per frame in
            // facing direction, unless wall-pinned via $5C flag).
            if !enemy.wall_pinned {
                let accel = match enemy.facing {
                    Direction::Right => ORCA_LUNGE_X_ACCEL,
                    Direction::Left => -ORCA_LUNGE_X_ACCEL,
                };
                enemy.x_velocity += accel;
            }

            // Move
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
            enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

            let Some(grid) = block_grid else {
                return;
            };
            let cx = (enemy.x_pos >> 16) as i32;
            let cy = (enemy.y_pos >> 16) as i32;

            // Slope collision → flip direction and re-lunge (Orca.asm loc_3D3DE)
            let floor = terrain::probe_floor(grid, cx, cy);
            if floor.on_ground
                && (floor.cell.collision == terrain::CollisionKind::SlopeUp
                    || floor.cell.collision == terrain::CollisionKind::SlopeDown)
            {
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                enemy.facing = opposite_direction(enemy.facing);
                enemy.wall_pinned = false;
                // Re-lunge in opposite direction
                enemy.x_velocity = match enemy.facing {
                    Direction::Right => ORCA_LUNGE_X_SPEED,
                    Direction::Left => -ORCA_LUNGE_X_SPEED,
                };
                enemy.y_velocity = ORCA_LUNGE_Y_SPEED;
                return;
            }

            // Wall bounce — ASM Orca.asm loc_3D40C: zero x_vel, set wall-pinned flag.
            let moving_right = enemy.x_velocity >= 0;
            let wall = terrain::probe_wall(
                grid,
                cx,
                cy,
                ENEMY_PROBE_HALF_WIDTH,
                ENEMY_PROBE_HEIGHT,
                moving_right,
            );
            if wall.hit {
                enemy.x_velocity = 0;
                enemy.wall_pinned = true;
                enemy.facing = opposite_direction(enemy.facing);
            }

            // Ceiling bounce — ASM Orca.asm loc_3D47A: y_vel = $18000
            if enemy.y_velocity < 0 {
                let ceil = terrain::probe_ceiling(
                    grid,
                    cx,
                    cy,
                    ENEMY_PROBE_HALF_WIDTH,
                    ENEMY_PROBE_HEIGHT,
                );
                if ceil.hit {
                    enemy.y_velocity = 0x1_8000; // bounce down
                }
            }

            // Floor landing (Orca.asm loc_3D420)
            if enemy.y_velocity > 0 {
                let floor = terrain::probe_floor(grid, cx, cy);
                if floor.on_ground
                    && floor.cell.collision != terrain::CollisionKind::SlopeUp
                    && floor.cell.collision != terrain::CollisionKind::SlopeDown
                {
                    let floor_y_fp = (floor.floor_y_px as u32) << 16;
                    if enemy.y_pos >= floor_y_fp {
                        enemy.y_pos = floor_y_fp;
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.on_ground = true;
                        enemy.wall_pinned = false;
                        enemy.orca_state = OrcaState::Walking;
                        enemy.wait_timer = ORCA_LUNGE_COOLDOWN;
                    }
                }
            }
        }
    }
}

// --- TarMonster constants (Tar_Monster.asm) ---
// Bytecode-driven cycle: submerged → emerge → visible → attack → submerge → loop.
// Timings from ASM script tables (unk_32264, unk_3226C):
//   Initial idle: 4f then 90f ($5A)
//   Emerge animation: 24f (stru_3272E, 6 sprites × 4fps)
//   Visible hold: 50f ($32) before attack
//   Attack: 24f (8+8+8 wind-up/fire/wind-down), fires horizontal glob shot
//   Submerge animation: 24f (stru_3274C, reverse of emerge)
/// Initial submerged wait before first emergence (Tar_Monster.asm unk_32264: 4 + 90).
pub(crate) const TAR_MONSTER_SUBMERGED_FRAMES: u16 = 94;
/// Emerge animation duration (6 frames × 4 ticks = 24 frames).
pub(crate) const TAR_MONSTER_EMERGE_FRAMES: u16 = 24;
/// Visible hold before attack sequence ($32 = 50 frames from bytecode).
pub(crate) const TAR_MONSTER_VISIBLE_FRAMES: u16 = 50;
/// Attack sequence: 8f wind-up + 8f fire + 8f wind-down = 24f.
pub(crate) const TAR_MONSTER_ATTACK_FRAMES: u16 = 24;
/// Frame within attack at which glob shot fires (after 8f wind-up).
pub(crate) const TAR_MONSTER_ATTACK_FIRE_FRAME: u16 = 8;
/// Submerge animation duration (24 frames, reverse of emerge).
pub(crate) const TAR_MONSTER_SUBMERGE_FRAMES: u16 = 24;
/// Horizontal glob shot speed: ±4px/frame (Tar_Monster.asm line 316: $FFFC = -4).
pub(crate) const TAR_MONSTER_GLOB_SPEED_X: i32 = 4 << 16;
/// Glob gravity: $1000/frame (Tar_Monster.asm line 412).
pub(crate) const TAR_MONSTER_GLOB_GRAVITY: i32 = 0x1000;

/// Step a TarMonster through its bytecode-driven emerge/attack/submerge cycle.
/// ASM: Tar_Monster.asm, jump table off_32290.
/// Submerged → Emerging → Visible → Attacking → Submerging → Submerged (loop).
/// While Submerged, the enemy is invulnerable and invisible (handled by collision/render).
pub(crate) fn step_tar_monster(enemy: &mut ActiveEnemy, projectiles: &mut Vec<EnemyProjectile>) {
    if enemy.wait_timer > 0 {
        // Check if we need to fire during attack sequence
        if enemy.tar_monster_state == TarMonsterState::Attacking {
            let elapsed = TAR_MONSTER_ATTACK_FRAMES - enemy.wait_timer;
            if elapsed == TAR_MONSTER_ATTACK_FIRE_FRAME {
                // Fire horizontal glob shot (Tar_Monster.asm Op 5 line 175)
                let xv = match enemy.facing {
                    Direction::Right => TAR_MONSTER_GLOB_SPEED_X,
                    Direction::Left => -TAR_MONSTER_GLOB_SPEED_X,
                };
                let spawn_y = (enemy.y_pos as i64 - (0x12 << 16)) as u32; // -18px Y offset
                let spawn_x = if matches!(enemy.facing, Direction::Left) {
                    (enemy.x_pos as i64 - (0x19 << 16)) as u32 // -25px X offset
                } else {
                    (enemy.x_pos as i64 + (0x19 << 16)) as u32 // +25px (mirrored)
                };
                projectiles.push(EnemyProjectile {
                    x_pos: spawn_x,
                    y_pos: spawn_y,
                    x_velocity: xv,
                    y_velocity: 0,
                    lifetime: 120,
                    gravity: TAR_MONSTER_GLOB_GRAVITY,
                    friendly: false,
                    homing_accel: 0,
                    speed_cap: 0,
                    wall_bounce: false,
                    ..Default::default()
                });
            }
        }
        enemy.wait_timer -= 1;
        return;
    }
    match enemy.tar_monster_state {
        TarMonsterState::Submerged => {
            enemy.tar_monster_state = TarMonsterState::Emerging;
            enemy.wait_timer = TAR_MONSTER_EMERGE_FRAMES;
        }
        TarMonsterState::Emerging => {
            // Flip direction on emerge (ASM Op 3: not.b x_direction)
            enemy.facing = opposite_direction(enemy.facing);
            enemy.tar_monster_state = TarMonsterState::Visible;
            enemy.wait_timer = TAR_MONSTER_VISIBLE_FRAMES;
        }
        TarMonsterState::Visible => {
            enemy.tar_monster_state = TarMonsterState::Attacking;
            enemy.wait_timer = TAR_MONSTER_ATTACK_FRAMES;
        }
        TarMonsterState::Attacking => {
            enemy.tar_monster_state = TarMonsterState::Submerging;
            enemy.wait_timer = TAR_MONSTER_SUBMERGE_FRAMES;
        }
        TarMonsterState::Submerging => {
            enemy.tar_monster_state = TarMonsterState::Submerged;
            enemy.wait_timer = TAR_MONSTER_SUBMERGED_FRAMES;
        }
    }
}

/// Spawn 5 tar blob projectiles in a spread pattern when a TarMonster is damaged.
pub(crate) fn spawn_tar_monster_retaliation(
    projectiles: &mut Vec<EnemyProjectile>,
    x: u32,
    y: u32,
) {
    let blobs: [(i32, i32); 5] = [
        (0x1_0000, -0x1_0000),  // up-right
        (-0x1_0000, -0x1_0000), // up-left
        (0x0_8000, -0x2_0000),  // steep up-right
        (-0x0_8000, -0x2_0000), // steep up-left
        (0x0_2000, -0x1_C000),  // gentle arc
    ];
    for (xv, yv) in blobs {
        projectiles.push(EnemyProjectile {
            x_pos: x,
            y_pos: y,
            x_velocity: xv,
            y_velocity: yv,
            lifetime: 60,
            gravity: 0x1000,
            friendly: false,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });
    }
}

/// Fire Demon death trail: spawn 3 fire fragments arcing outward from death position.
pub(crate) fn spawn_fire_demon_death_trail(projectiles: &mut Vec<EnemyProjectile>, x: u32, y: u32) {
    // TODO(fidelity): visual — fire trail uses generic projectile rendering.
    // ASM uses fire fragment VRAM tiles. Needs ROM sprite extraction for visual parity.
    let fragments: [(i32, i32); 3] = [
        (0x1_8000, -0x2_0000),  // right arc
        (-0x1_8000, -0x2_0000), // left arc
        (0, -0x2_8000),         // straight up
    ];
    for (xv, yv) in fragments {
        projectiles.push(EnemyProjectile {
            x_pos: x,
            y_pos: y,
            x_velocity: xv,
            y_velocity: yv,
            lifetime: 45,
            gravity: 0x1800,
            friendly: false,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });
    }
}

// --- DragonWalking constants ---
// From ASM Dragon.asm: initial cooldown $12C (300f, line 95), reset after breath $1F4 (500f, line 194).
pub(crate) const DRAGON_WALKING_BREATH_INITIAL: u16 = 300;
pub(crate) const DRAGON_WALKING_BREATH_RESET: u16 = 500;
pub(crate) const DRAGON_WALKING_BREATH_SPEED: i32 = 0x3_0000;
pub(crate) const DRAGON_WALKING_BREATH_LIFETIME: u16 = 68;
pub(crate) const DRAGON_WALKING_BREATH_HOLD_FRAMES: u16 = 68;
pub(crate) const DRAGON_WALKING_RECOVER_FRAMES: u16 = 0;
pub(crate) const DRAGON_WALKING_BREATH_OFFSET_PX: i32 = 0x1C;
pub(crate) const DRAGON_WALKING_GRAVITY: i32 = 0x4000;
pub(crate) const SHARED_WALKER_STATE_PATROL: i16 = 10;
pub(crate) const SHARED_WALKER_STATE_FACE_RIGHT: i16 = 1;
pub(crate) const SHARED_WALKER_STATE_FACE_LEFT: i16 = 2;
const SHARED_WALKER_STATE_COOLDOWN: i16 = -120;
const SHARED_WALKER_DETECT_RANGE: i32 = 0x60;
const SHARED_WALKER_RELEASE_RANGE: i32 = 0x70;
const DRAGON_WALKING_PATROL_SHORT_FRAMES: u16 = 0x32;
const DRAGON_WALKING_PATROL_LONG_FRAMES: u16 = 0x12C;
const DRAGON_WALKING_PHASE_WALK: u8 = 0;
const DRAGON_WALKING_PHASE_BREATH: u8 = 1;
const DRAGON_WALKING_PHASE_RECOVER: u8 = 2;

enum DragonWalkingWalkStep {
    Walked,
    Idled,
    StartedAirborne,
}

pub(crate) fn spawn_shared_walker_state(family: EnemyFamily) -> i16 {
    match family {
        EnemyFamily::DragonWalking | EnemyFamily::Lion => SHARED_WALKER_STATE_PATROL,
        _ => 0,
    }
}

pub(crate) fn spawn_shared_walker_patrol_timer(family: EnemyFamily) -> u16 {
    match family {
        EnemyFamily::DragonWalking => DRAGON_WALKING_PATROL_LONG_FRAMES,
        EnemyFamily::Lion => LION_CHARGE_TIMER_INIT,
        _ => 0,
    }
}

pub(crate) fn spawn_shared_walker_patrol_short_next(family: EnemyFamily) -> bool {
    matches!(family, EnemyFamily::DragonWalking)
}

fn shared_walker_turnaround(enemy: &mut ActiveEnemy) {
    reverse_enemy_direction(enemy);
    if enemy.shared_walker_state > 0 && enemy.shared_walker_state != SHARED_WALKER_STATE_PATROL {
        enemy.shared_walker_state = SHARED_WALKER_STATE_COOLDOWN;
    }
}

fn shared_walker_should_tick_patrol(enemy: &ActiveEnemy) -> bool {
    enemy.shared_walker_state <= 0 || enemy.shared_walker_state == SHARED_WALKER_STATE_PATROL
}

fn shared_walker_tick_patrol(enemy: &mut ActiveEnemy) -> bool {
    if !shared_walker_should_tick_patrol(enemy) || enemy.shared_walker_patrol_timer == 0 {
        return false;
    }

    enemy.shared_walker_patrol_timer -= 1;
    if enemy.shared_walker_patrol_timer != 0 {
        return false;
    }

    enemy.shared_walker_patrol_timer = match enemy.family {
        EnemyFamily::DragonWalking => {
            let next = if enemy.shared_walker_patrol_short_next {
                DRAGON_WALKING_PATROL_SHORT_FRAMES
            } else {
                DRAGON_WALKING_PATROL_LONG_FRAMES
            };
            enemy.shared_walker_patrol_short_next = !enemy.shared_walker_patrol_short_next;
            next
        }
        EnemyFamily::Lion => LION_CHARGE_TIMER_INIT,
        _ => 0,
    };
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DragonWalkingFloorSupport {
    selected: terrain::FloorProbe,
    center: terrain::FloorProbe,
    left: terrain::FloorProbe,
    right: terrain::FloorProbe,
}

fn dragon_walking_floor_support(
    grid: &terrain::BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
) -> DragonWalkingFloorSupport {
    let center = terrain::probe_floor(grid, center_x_px, foot_y_px);
    let left = terrain::probe_floor(grid, center_x_px - ENEMY_PROBE_HALF_WIDTH, foot_y_px);
    let right = terrain::probe_floor(grid, center_x_px + ENEMY_PROBE_HALF_WIDTH, foot_y_px);
    DragonWalkingFloorSupport {
        selected: terrain::probe_floor_hitbox(grid, center_x_px, foot_y_px, ENEMY_PROBE_HALF_WIDTH),
        center,
        left,
        right,
    }
}

fn dragon_walking_probe_wall(
    grid: &terrain::BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    moving_right: bool,
) -> terrain::WallProbe {
    let wall_center_x = if moving_right {
        center_x_px
    } else {
        center_x_px - 1
    };
    let edge_x = if moving_right {
        wall_center_x + ENEMY_PROBE_HALF_WIDTH + 1
    } else {
        wall_center_x - ENEMY_PROBE_HALF_WIDTH
    };

    // ORIGINAL_QUIRK: the shared Dragon/Lion grounded walker helpers
    // (`sub_36972`/`sub_36A58`) treat hurt tiles as blocking side contacts for
    // the turnaround path. The frame-3126 TAS frontier depends on a spike
    // shoulder counting as a wall.
    let lower_body_y = foot_y_px - 8;
    let upper_body_y = foot_y_px - ENEMY_PROBE_HEIGHT + 8;
    let probe_ys = [
        lower_body_y,
        foot_y_px - ENEMY_PROBE_HEIGHT / 2,
        upper_body_y,
    ];

    for &py in &probe_ys {
        let probe = terrain::probe_at(grid, edge_x, py);
        if matches!(
            probe.cell.collision,
            terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
        ) {
            let wall_x = if moving_right {
                i32::from(probe.cell_x) * 16
            } else {
                (i32::from(probe.cell_x) + 1) * 16
            };
            return terrain::WallProbe {
                hit: true,
                wall_x_px: wall_x,
                cell: probe.cell,
                cell_x: probe.cell_x,
                cell_y: probe.cell_y,
            };
        }
    }

    terrain::WallProbe {
        hit: false,
        wall_x_px: 0,
        cell: terrain::Cell::from_be_bytes(0, 0),
        cell_x: 0,
        cell_y: 0,
    }
}

fn dragon_walking_wall_snap_x(wall_x_px: i32, moving_right: bool) -> u32 {
    let wall_fp = (wall_x_px as u32) << 16;
    let edge_fp = (ENEMY_PROBE_HALF_WIDTH as u32) << 16;
    if moving_right {
        wall_fp.saturating_sub(edge_fp)
    } else {
        wall_fp + edge_fp + (1 << 16)
    }
}

fn dragon_walking_blocked(enemy: &ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) -> bool {
    let Some(grid) = block_grid else {
        return false;
    };
    let walk_speed = pacer_walk_speed(enemy.family, enemy.hit_points);
    let x_velocity = horizontal_velocity(enemy.facing, walk_speed);
    let moving_right = x_velocity >= 0;
    let next_x = (enemy.x_pos as i64 + i64::from(x_velocity)) as u32;
    let cx = (next_x >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let wall = dragon_walking_probe_wall(grid, cx, cy, moving_right);
    if wall.hit {
        return true;
    }
    !dragon_walking_floor_support(grid, cx, cy)
        .selected
        .on_ground
}

fn step_dragon_walking_walk(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
) -> DragonWalkingWalkStep {
    // ASM routes the positive shared-walker states through loc_3E4CE/loc_3E50A,
    // which swap $50(a5) to the stationary $4C(a5) slot before returning to the
    // grounded walker loop. DragonWalking leaves $4C(a5) at zero, so the alert
    // branch holds position until Kid leaves the wider release window.
    let walk_speed = if enemy.shared_walker_state > 0
        && enemy.shared_walker_state != SHARED_WALKER_STATE_PATROL
    {
        0
    } else {
        pacer_walk_speed(enemy.family, enemy.hit_points)
    };
    let walk_delta = horizontal_velocity(enemy.facing, walk_speed);
    let moving_right = walk_delta >= 0;

    if shared_walker_tick_patrol(enemy) {
        reverse_enemy_direction(enemy);
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        enemy.shared_walker_debug_branch = Some("patrol_timer_turn");
        return DragonWalkingWalkStep::Idled;
    }

    let current_y_px = (enemy.y_pos >> 16) as i32;

    if let Some(grid) = block_grid {
        let next_x = (enemy.x_pos as i64 + i64::from(walk_delta)) as u32;
        let cx = (next_x >> 16) as i32;
        let wall = dragon_walking_probe_wall(grid, cx, current_y_px, moving_right);
        let floor_support = dragon_walking_floor_support(grid, cx, current_y_px);
        enemy.shared_walker_terrain_trace = SharedWalkerTerrainTrace {
            next_x_px: Some(cx),
            wall: Some(wall),
            center_floor: Some(floor_support.center),
            left_floor: Some(floor_support.left),
            right_floor: Some(floor_support.right),
        };
        if wall.hit {
            enemy.x_pos = dragon_walking_wall_snap_x(wall.wall_x_px, moving_right);
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            shared_walker_turnaround(enemy);
            enemy.shared_walker_debug_branch = Some("patrol_wall_turn");
            return DragonWalkingWalkStep::Idled;
        }

        let floor = floor_support.selected;
        if !floor.on_ground {
            enemy.on_ground = false;
            enemy.x_velocity = walk_delta;
            enemy.y_velocity = 0;
            enemy.y_pos = enemy.y_pos.saturating_add(1 << 16);
            enemy.shared_walker_debug_branch = Some("patrol_airborne_start");
            return DragonWalkingWalkStep::StartedAirborne;
        }

        enemy.y_pos = (floor.floor_y_px as u32) << 16;
        enemy.on_ground = true;
    }

    if enemy.shared_walker_state < 0 {
        enemy.shared_walker_state += 1;
    } else if enemy.shared_walker_state == 0 {
        if let Some(player_x_pos) = player_x_pos {
            let dx = ((player_x_pos as i64 - enemy.x_pos as i64) >> 16) as i32;
            enemy.shared_walker_dx_to_kid_px =
                Some(dx.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);
            if dx.abs() <= SHARED_WALKER_DETECT_RANGE {
                if dx >= 0 {
                    enemy.facing = Direction::Right;
                    enemy.shared_walker_state = SHARED_WALKER_STATE_FACE_RIGHT;
                } else {
                    enemy.facing = Direction::Left;
                    enemy.shared_walker_state = SHARED_WALKER_STATE_FACE_LEFT;
                }
                enemy.x_velocity = 0;
                enemy.y_velocity = 0;
                enemy.shared_walker_debug_branch = Some("calm_face_kid");
                return DragonWalkingWalkStep::Idled;
            }
        }
    } else if enemy.shared_walker_state != SHARED_WALKER_STATE_PATROL
        && let Some(player_x_pos) = player_x_pos
    {
        let dx = ((player_x_pos as i64 - enemy.x_pos as i64) >> 16) as i32;
        enemy.shared_walker_dx_to_kid_px =
            Some(dx.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);
        if dx.abs() > SHARED_WALKER_RELEASE_RANGE {
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            shared_walker_turnaround(enemy);
            enemy.shared_walker_debug_branch = Some("alert_release_turn");
            return DragonWalkingWalkStep::Idled;
        }
    }

    enemy.x_pos = (enemy.x_pos as i64 + i64::from(walk_delta)) as u32;
    enemy.x_velocity = 0;
    enemy.y_velocity = 0;
    enemy.shared_walker_debug_branch = Some(
        if enemy.shared_walker_state > 0 && enemy.shared_walker_state != SHARED_WALKER_STATE_PATROL
        {
            "alert_hold"
        } else if enemy.shared_walker_state < 0 {
            "cooldown_walk"
        } else {
            "patrol_walk"
        },
    );
    DragonWalkingWalkStep::Walked
}

fn step_dragon_walking_airborne(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    let Some(grid) = block_grid else {
        enemy.y_velocity = (enemy.y_velocity + DRAGON_WALKING_GRAVITY).min(physics::MAX_FALL_SPEED);
        enemy.shared_walker_debug_branch = Some("airborne_fall");
        return;
    };

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let moving_right = enemy.x_velocity >= 0;
    let wall = dragon_walking_probe_wall(grid, cx, cy, moving_right);
    let floor = terrain::probe_floor(grid, cx, cy);
    enemy.shared_walker_terrain_trace = SharedWalkerTerrainTrace {
        next_x_px: Some(cx),
        wall: Some(wall),
        center_floor: Some(floor),
        left_floor: None,
        right_floor: None,
    };
    if wall.hit {
        enemy.x_pos = dragon_walking_wall_snap_x(wall.wall_x_px, moving_right);
        enemy.x_velocity = 0;
        enemy.shared_walker_debug_branch = Some("airborne_wall_snap");
    }

    if floor.on_ground {
        let floor_y_fp = (floor.floor_y_px as u32) << 16;
        if enemy.y_velocity >= 0 && enemy.y_pos >= floor_y_fp {
            enemy.y_pos = floor_y_fp;
            enemy.y_velocity = 0;
            enemy.x_velocity = 0;
            enemy.on_ground = true;
            if enemy.shared_walker_state != SHARED_WALKER_STATE_PATROL {
                enemy.shared_walker_state = SHARED_WALKER_STATE_COOLDOWN;
                enemy.shared_walker_debug_branch = Some("airborne_land_cooldown");
            } else {
                enemy.shared_walker_debug_branch = Some("airborne_land_patrol");
            }
            return;
        }
    }

    enemy.y_velocity = (enemy.y_velocity + DRAGON_WALKING_GRAVITY).min(physics::MAX_FALL_SPEED);
    enemy.shared_walker_debug_branch = Some("airborne_fall");
}
/// Fire breath for a walking dragon (called after step_pacer handles movement).
pub(crate) fn step_dragon_walking(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x_pos: Option<u32>,
) {
    enemy.shared_walker_debug_branch = None;
    enemy.shared_walker_dx_to_kid_px = None;
    enemy.shared_walker_terrain_trace = SharedWalkerTerrainTrace::default();

    let dragon_hurt_hibernate_armed = enemy.damage_flash_frames == ENEMY_DAMAGE_FLASH_FRAMES;
    if enemy.grounded_frames > 0 && !dragon_hurt_hibernate_armed {
        enemy.grounded_frames -= 1;
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        enemy.shared_walker_debug_branch = Some("grounded_hibernate");
        return;
    }

    if enemy.dragon_script_cmd == DRAGON_WALKING_PHASE_WALK && enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        if enemy.wait_timer == 0 {
            let x_vel = match enemy.facing {
                Direction::Right => DRAGON_WALKING_BREATH_SPEED,
                Direction::Left => -DRAGON_WALKING_BREATH_SPEED,
            };
            let x_offset = match enemy.facing {
                Direction::Right => DRAGON_WALKING_BREATH_OFFSET_PX,
                Direction::Left => -DRAGON_WALKING_BREATH_OFFSET_PX,
            };
            projectiles.push(EnemyProjectile {
                x_pos: (enemy.x_pos as i64 + ((i64::from(x_offset)) << 16)) as u32,
                y_pos: enemy.y_pos,
                x_velocity: x_vel,
                y_velocity: 0,
                lifetime: DRAGON_WALKING_BREATH_LIFETIME,
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
            enemy.wait_timer = DRAGON_WALKING_BREATH_RESET;
            enemy.dragon_script_cmd = DRAGON_WALKING_PHASE_BREATH;
            enemy.dragon_phase_frames = DRAGON_WALKING_BREATH_HOLD_FRAMES;
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            return;
        }
    }

    if !enemy.on_ground {
        step_dragon_walking_airborne(enemy, block_grid);
        return;
    }

    let blocked = if enemy.dragon_script_cmd == DRAGON_WALKING_PHASE_WALK {
        match step_dragon_walking_walk(enemy, block_grid, player_x_pos) {
            DragonWalkingWalkStep::Walked => false,
            DragonWalkingWalkStep::Idled => false,
            DragonWalkingWalkStep::StartedAirborne => {
                step_dragon_walking_airborne(enemy, block_grid);
                return;
            }
        }
    } else {
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        dragon_walking_blocked(enemy, block_grid)
    };

    if enemy.dragon_script_cmd == DRAGON_WALKING_PHASE_WALK && blocked {
        reverse_enemy_direction(enemy);
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        return;
    }

    match enemy.dragon_script_cmd {
        DRAGON_WALKING_PHASE_BREATH => {
            if enemy.dragon_phase_frames > 0 {
                enemy.dragon_phase_frames -= 1;
            } else {
                enemy.dragon_script_cmd = DRAGON_WALKING_PHASE_RECOVER;
                enemy.dragon_phase_frames = DRAGON_WALKING_RECOVER_FRAMES;
            }
        }
        DRAGON_WALKING_PHASE_RECOVER => {
            if enemy.dragon_phase_frames > 0 {
                enemy.dragon_phase_frames -= 1;
            } else {
                enemy.dragon_script_cmd = DRAGON_WALKING_PHASE_WALK;
            }
        }
        _ => {}
    }
}

// --- Drips constants (Drips.asm) ---
// ASM Drips is a multi-object enemy: ceiling attachment, forming animation,
// animation-driven drop, splash, and surface-crawl spawning.
// Native approximation: periodic projectile drop with ASM-matched timing.
// Random dormant delay: (RNG >> 2) XOR kid_x ≈ 0-63f average ~32f.
// Forming animation: ~35f. Total interval ≈ 67f average.
// Drop: 10px/frame for 12 frames in ASM (animation-driven, not physics).
// Native uses physics projectile with gravity $2000 for similar trajectory.
// Surface-crawl spawning (max 5 crawlers, x_vel=$3000) not implemented —
// would require multi-object child spawning system.
/// Drop interval: forming animation (~35f) + average random delay (~32f) = ~67f.
pub(crate) const DRIPS_DROP_INTERVAL: u16 = 67;
/// Drop Y speed: ASM drops at 10px/frame (line 106: addi.w #$A,y_pos).
/// Native uses initial velocity + gravity to approximate the trajectory.
pub(crate) const DRIPS_DROP_Y_SPEED: i32 = 0xA_0000; // 10px/frame in FP16.16
/// Gravity accumulation: $2000/frame (Drips.asm line 93).
pub(crate) const DRIPS_DROP_GRAVITY: i32 = 0x2000;

/// Step a Drips enemy: stationary ceiling hanger that periodically drops projectiles.
/// ASM: Drips.asm — forming animation → drop at 10px/frame → splash → reform → repeat.
/// Native approximation uses a physics projectile for the drop.
pub(crate) fn step_drips(enemy: &mut ActiveEnemy, projectiles: &mut Vec<EnemyProjectile>) {
    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }
    // Drop a projectile straight down (Drips.asm line 106: 10px/frame)
    projectiles.push(EnemyProjectile {
        x_pos: enemy.x_pos,
        y_pos: enemy.y_pos,
        x_velocity: 0,
        y_velocity: DRIPS_DROP_Y_SPEED,
        lifetime: 30, // 12-frame ASM drop + splash ≈ 30f total
        gravity: DRIPS_DROP_GRAVITY,
        friendly: false,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
        ..Default::default()
    });
    enemy.wait_timer = DRIPS_DROP_INTERVAL;
}

// SpinningTwins: 8-entry orbit X offset table from ASM (word_3B9A0).
// Applied relative to spawn position during orbit phase.
pub(crate) const SPINNING_TWINS_ORBIT_OFFSETS: [i32; 8] =
    [-0x10, -0x0A, 0x05, 0x11, 0x10, 0x0A, -0x05, -0x11];
/// Orbit-to-walk transition: collision-triggered. ASM loc_3B904 checks
/// collision_type each frame — negative, $2C, or $1C all trigger the drop.
/// In native, we check player hitbox overlap with the orbiting enemy.
/// Hitbox half-width for orbit collision check (approximate from sprite size).
const SPINNING_TWINS_ORBIT_HITBOX_HALF: i32 = 16;
// Gravity applied each frame once the twins drop. ASM: $2800 FP.
pub(crate) const SPINNING_TWINS_GRAVITY: i32 = 0x2800;
// Launch Y velocity when transitioning from orbit. ASM: -$10000.
pub(crate) const SPINNING_TWINS_DROP_Y_VEL: i32 = -0x1_0000;
// Initial X velocity on drop: ASM sets $FFFC8000 (left) or $38000 (right).
pub(crate) const SPINNING_TWINS_DROP_X_VEL_LEFT: i32 = -0x3_8000;
pub(crate) const SPINNING_TWINS_DROP_X_VEL_RIGHT: i32 = 0x3_8000;

/// SpinningTwins: orbit at spawn position, then drop and walk as pacer.
/// Orbit-to-walk transition is collision-triggered (player touching the orbiting form).
/// ASM: loc_3B904 checks collision_type each frame.
pub(crate) fn step_spinning_twins(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    match enemy.spinning_twins_phase {
        SpinningTwinsPhase::Orbiting => {
            // Advance orbit index each frame
            enemy.spinning_twins_orbit_index =
                (enemy.spinning_twins_orbit_index + 1) % SPINNING_TWINS_ORBIT_OFFSETS.len() as u8;
            let offset = SPINNING_TWINS_ORBIT_OFFSETS[enemy.spinning_twins_orbit_index as usize];
            // Apply X oscillation as velocity (visual only, resets each frame)
            enemy.x_velocity = offset << 16;
            enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
            enemy.x_velocity = 0;

            // Check player collision — triggers orbit-to-walk transition.
            // ASM loc_3B904: collision_type negative/$2C/$1C all trigger drop.
            let player_contact = if let (Some(px), Some(py)) = (player_x_pos, player_y_pos) {
                let ex = (enemy.x_pos >> 16) as i32;
                let ey = (enemy.y_pos >> 16) as i32;
                let px_i = (px >> 16) as i32;
                let py_i = (py >> 16) as i32;
                (ex - px_i).abs() < SPINNING_TWINS_ORBIT_HITBOX_HALF
                    && (ey - py_i).abs() < SPINNING_TWINS_ORBIT_HITBOX_HALF + 8
            } else {
                false
            };

            if player_contact {
                // Drop to walking phase with initial X impulse (ASM loc_3BA74)
                enemy.spinning_twins_phase = SpinningTwinsPhase::Walking;
                enemy.y_velocity = SPINNING_TWINS_DROP_Y_VEL;
                enemy.x_velocity = match enemy.facing {
                    Direction::Left => SPINNING_TWINS_DROP_X_VEL_LEFT,
                    Direction::Right => SPINNING_TWINS_DROP_X_VEL_RIGHT,
                };
                enemy.on_ground = false;
            }
        }
        SpinningTwinsPhase::Walking => {
            // Apply gravity while airborne until grounded, then walk as pacer
            if !enemy.on_ground {
                enemy.y_velocity += SPINNING_TWINS_GRAVITY;
                enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
                // Check floor
                if let Some(grid) = block_grid {
                    let x_px = (enemy.x_pos >> 16) as i32;
                    let y_px = (enemy.y_pos >> 16) as i32;
                    let probe = terrain::probe_floor(grid, x_px, y_px);
                    if probe.on_ground {
                        enemy.y_pos = (probe.floor_y_px as u32) << 16;
                        enemy.y_velocity = 0;
                        enemy.on_ground = true;
                    }
                }
            } else {
                step_pacer(enemy, block_grid);
            }
        }
    }
}

/// EmoRock hover height above player, indexed by enemy_level (derived from HP).
/// Level 0: 80px, Level 1: 60px, Level 2: 40px.
pub(crate) const EMO_ROCK_HOVER_HEIGHT: [i32; 3] = [0x50, 0x3C, 0x28];

/// EmoRock gravity acceleration (applied each frame while dropping).
/// Level 0: $4000, Level 1: $6000, Level 2: $8000.
pub(crate) const EMO_ROCK_GRAVITY: [i32; 3] = [0x4000, 0x6000, 0x8000];

/// EmoRock terminal velocity threshold/cap while dropping.
/// ASM Emo_Rock.asm:72-74: if y_vel > $60000, reset to $40000.
pub(crate) const EMO_ROCK_VELOCITY_THRESHOLD: i32 = 0x6_0000;
pub(crate) const EMO_ROCK_VELOCITY_CAP: i32 = 0x4_0000;

/// EmoRock walk speed after landing.
pub(crate) const EMO_ROCK_WALK_SPEED: i32 = 0x8000;

/// Step an EmoRock enemy through its state machine:
/// - wait_timer > 0: hover above player, count down
/// - wait_timer == 0 && !on_ground: drop with gravity
/// - on_ground: walk like a pacer
pub(crate) fn step_emo_rock(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    if enemy.on_ground {
        // Walking phase: pacer-like ground movement facing player.
        if let Some(dir) = player_relative_direction(enemy.x_pos, player_x_pos) {
            enemy.facing = dir;
        }
        let Some(grid) = block_grid else { return };
        enemy.x_velocity = horizontal_velocity(enemy.facing, EMO_ROCK_WALK_SPEED);
        enemy.y_velocity = 0;
        enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;
        let moving_right = enemy.x_velocity >= 0;

        let wall = terrain::probe_wall(
            grid,
            cx,
            cy,
            ENEMY_PROBE_HALF_WIDTH,
            ENEMY_PROBE_HEIGHT,
            moving_right,
        );
        if wall.hit {
            reverse_enemy_direction(enemy);
        }
        return;
    }

    if enemy.wait_timer > 0 {
        // Hovering phase: track player position, count down.
        let level_index = enemy.hit_points.saturating_sub(1).min(2) as usize;
        let hover_height_fp = (EMO_ROCK_HOVER_HEIGHT[level_index] as u32) << 16;
        if let Some(px) = player_x_pos {
            enemy.x_pos = px;
        }
        if let Some(py) = player_y_pos {
            enemy.y_pos = py.saturating_sub(hover_height_fp);
        }
        enemy.wait_timer -= 1;
        return;
    }

    // Dropping phase: apply gravity, check floor.
    let level_index = enemy.hit_points.saturating_sub(1).min(2) as usize;
    let gravity = EMO_ROCK_GRAVITY[level_index];
    let new_yv = enemy.y_velocity + gravity;
    // ASM Emo_Rock.asm:72-74: if exceeds $60000, snap to $40000
    enemy.y_velocity = if new_yv > EMO_ROCK_VELOCITY_THRESHOLD {
        EMO_ROCK_VELOCITY_CAP
    } else {
        new_yv
    };
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground {
            // Land: snap to floor, enter walking state.
            enemy.y_pos = (floor.floor_y_px as u32) << 16;
            enemy.y_velocity = 0;
            enemy.on_ground = true;
            if let Some(dir) = player_relative_direction(enemy.x_pos, player_x_pos) {
                enemy.facing = dir;
            }
        }
    }
}

pub(crate) fn reverse_enemy_direction(enemy: &mut ActiveEnemy) {
    enemy.facing = opposite_direction(enemy.facing);
}

pub(crate) fn opposite_direction(facing: Direction) -> Direction {
    match facing {
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    }
}

pub(crate) fn player_relative_direction(
    enemy_x_pos: u32,
    player_x_pos: Option<u32>,
) -> Option<Direction> {
    let player_x_pos = player_x_pos?;
    Some(if player_x_pos > enemy_x_pos {
        Direction::Right
    } else {
        Direction::Left
    })
}

/// Compute arrow angle index (0..27) from archer to player.
/// Faithfully mirrors Archer.asm `loc_34458`: quadrant dispatch + unk_34404 ratio lookup.
pub(crate) fn archer_angle_to_player(
    archer_x: u32,
    archer_y: u32,
    player_x: u32,
    player_y: u32,
) -> usize {
    let dx = (player_x >> 16) as i32 - (archer_x >> 16) as i32;
    // Archer aims 16px above player centre (subi.w #$10,d7 in ASM).
    let dy = ((player_y >> 16) as i32 - 0x10) - (archer_y >> 16) as i32;

    // Ratio lookup table from unk_34404 (84 entries). Maps (abs_dy*8/abs_dx) to sub-angle 0-5.
    const RATIO_TABLE: [u8; 84] = [
        0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    ];

    if dx == 0 {
        // ORIGINAL_QUIRK: ASM loc_34530 sets d7=0 then tests d7 (not d4/dy), so
        // bmi never fires and this always returns angle 21 (straight down).
        return 0x15;
    }

    let abs_dx = dx.unsigned_abs();
    let abs_dy = dy.unsigned_abs();
    let ratio = (abs_dy * 8) / abs_dx;

    // ASM loc_34546: ratio >= $3F → nearly vertical, return 6 (up) or 21 (down).
    if ratio >= 0x3F {
        return if dy < 0 { 6 } else { 0x15 };
    }

    let sub = RATIO_TABLE[(ratio as usize).min(RATIO_TABLE.len() - 1)] as i32;

    match (dx > 0, dy >= 0) {
        // Quadrant 0: right+up (d5=0)
        (true, false) => sub as usize,
        // Quadrant 1: left+up (d5=1) → 0xD - sub
        (false, false) => (0x0D - sub) as usize,
        // Quadrant 2: left+down (d5=2) → 0xE + sub
        (false, true) => (0x0E + sub) as usize,
        // Quadrant 3: right+down (d5=3) → 0x1B - sub
        (true, true) => (0x1B - sub) as usize,
    }
}

/// Archer arrow spawn X offset (Archer.asm Shooting_Horizontal/Vertical 3rd frame: $FFF8 = -8).
const ARCHER_ARROW_X_OFFSET: i32 = -8;
/// Archer arrow spawn Y offset for horizontal shooting (Archer.asm: $FFEF = -17).
const ARCHER_ARROW_Y_OFFSET_HORIZONTAL: i32 = -17;
/// Archer arrow spawn Y offset for vertical shooting (Archer.asm: $FFE8 = -24).
const ARCHER_ARROW_Y_OFFSET_VERTICAL: i32 = -24;

/// Adjust arrow angle by ±1 toward target on a 28-value circular ring.
/// ASM loc_34308/loc_34320: if shortest arc ≤ 14 steps, move direct; otherwise wrap.
pub(crate) fn archer_adjust_angle(current: u8, target: u8) -> u8 {
    if current == target {
        return current;
    }
    let delta = target as i16 - current as i16; // -27..27
    let threshold: i16 = if target > current { 14 } else { -14 };
    if delta >= threshold {
        // Decrement path (ASM loc_34320)
        if current == 0 { 27 } else { current - 1 }
    } else {
        // Increment path (ASM loc_34308 fall-through)
        if current >= 27 { 0 } else { current + 1 }
    }
}

/// Convert an angle index (0..27) to (x_velocity, y_velocity) via ARROW_ANGLE_TABLE.
/// ASM Enemy07_ArcherArrow_AngleToSpeedAndAnim: byte * 0x1000.
pub(crate) fn arrow_velocity_from_angle(angle: usize) -> (i32, i32) {
    let (x_byte, y_byte) = ARROW_ANGLE_TABLE[angle.min(ARROW_ANGLE_TABLE.len() - 1)];
    ((x_byte as i32) * 0x1000, (y_byte as i32) * 0x1000)
}

/// Step Archer: stationary enemy that faces player and fires arrows at intervals.
/// ASM: arrows persist until terrain/object collision, re-aiming every `arrow_accuracy`
/// frames by ±1 angle step toward the player (loc_342D6 loop).
pub(crate) fn step_archer(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    // Face towards player.
    if let Some(dir) = player_relative_direction(enemy.x_pos, player_x_pos) {
        enemy.facing = dir;
    }

    // Count down fire timer.
    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }

    // Reset timer.
    let level = enemy.hit_points.saturating_sub(1).min(2) as usize;
    enemy.wait_timer = ARCHER_FIRE_TIMER[level];

    // Need player position to compute angle.
    let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
        return;
    };

    // ASM loc_34248: `cmpi.w #2,(Number_of_Arrows).w; bge.w loc_3408C`
    if projectiles.len() >= MAX_PROJECTILES {
        return;
    }

    // Compute initial angle and velocity.
    let angle = archer_angle_to_player(enemy.x_pos, enemy.y_pos, px, py);
    let (x_velocity, y_velocity) = arrow_velocity_from_angle(angle);
    let reaim = ARCHER_FIRE_TIMER[level];

    // Spawn offset from Archer.asm 3rd animation frame.
    // Vertical shooting (angle 6..=13): Y=-24. Horizontal: Y=-17.
    // X offset negated when facing right (ASM loc_34216).
    let y_offset = if (6..=13).contains(&angle) {
        ARCHER_ARROW_Y_OFFSET_VERTICAL
    } else {
        ARCHER_ARROW_Y_OFFSET_HORIZONTAL
    };
    let x_offset = if enemy.facing == Direction::Right {
        -ARCHER_ARROW_X_OFFSET
    } else {
        ARCHER_ARROW_X_OFFSET
    };
    let spawn_x = (enemy.x_pos as i64 + ((x_offset as i64) << 16)) as u32;
    let spawn_y = (enemy.y_pos as i64 + ((y_offset as i64) << 16)) as u32;

    projectiles.push(EnemyProjectile {
        x_pos: spawn_x,
        y_pos: spawn_y,
        x_velocity,
        y_velocity,
        // ASM arrows live until terrain collision, not a fixed timer.
        // Use large lifetime as fallback; terrain_kill handles normal removal.
        lifetime: u16::MAX,
        friendly: false,
        gravity: 0,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
        reaim_interval: reaim,
        reaim_countdown: reaim,
        current_angle: angle as u8,
        terrain_kill: true,
        owner: None,
    });
}

/// Scorpion tracking callback (Scorpion.asm loc_3DD7E).
///
/// Within 170px ($AA): face toward player (unless tracking-locked $4E),
/// boost speed to $38000 if enemy_level > 1 (HP > 3).
/// Outside 170px: reset speed to $18000, clear tracking lock if player airborne.
pub(crate) fn step_scorpion_tracking(
    enemy: &mut ActiveEnemy,
    player_x_pos: Option<u32>,
    _player_y_pos: Option<u32>,
) {
    let Some(px) = player_x_pos else { return };
    let dx = (enemy.x_pos as i64 - px as i64) >> 16; // scorpion_x - player_x

    // Outside tracking range: reset speed, optionally clear tracking lock.
    // ASM loc_3DE7E: $50(a5) = $18000; if player airborne, clear $4E.
    if dx.abs() > SCORPION_TRACK_RANGE {
        enemy.walk_speed_override = SCORPION_BASE_SPEED;
        // ASM checks player y_vel ($26(a4)) to clear tracking lock.
        // We approximate: clear lock when out of range (conservative).
        enemy.scorpion_tracking_lock = false;
        return;
    }

    // Within tracking range: face toward player unless tracking-locked.
    // ASM loc_3DD7E lines 146-155: if $4E clear, set x_direction toward player.
    if !enemy.scorpion_tracking_lock {
        if dx > 0 {
            enemy.facing = Direction::Left; // player is to the left
        } else if dx < 0 {
            enemy.facing = Direction::Right; // player is to the right
        }
    }

    // ASM line 162-164: if enemy_level > 1 (HP >= 5 since HP = level + 3), boost speed.
    if enemy.hit_points >= 5 {
        enemy.walk_speed_override = SCORPION_BOOST_SPEED;
    }
}

/// Crab freeze distance constants from ASM.
/// ASM loc_3B0E4: cmpi.w #$40,d7 (freeze threshold = 64px).
const CRAB_FREEZE_DISTANCE: i64 = 0x40;
/// ASM loc_3B142: cmpi.w #$E0,d7 (unfreeze threshold = 224px).
const CRAB_UNFREEZE_DISTANCE: i64 = 0xE0;

/// Crab player-proximity freeze callback (Crab.asm loc_3B0BE-loc_3B174).
///
/// When player is within 64px horizontally AND facing the crab:
/// - Freeze the crab (stop animation, set freeze sprite, zero velocity)
///
/// Unfreeze when:
/// - Distance > 224px, OR
/// - Within 64px but player is moving away from the crab
///
/// The freeze uses the player's facing direction and velocity to determine
/// whether to freeze or unfreeze.
pub(crate) fn step_crab_freeze(
    enemy: &mut ActiveEnemy,
    player_x_pos: Option<u32>,
    player_facing: Option<Direction>,
    player_x_velocity: Option<i32>,
) {
    let Some(px) = player_x_pos else { return };
    let Some(pfacing) = player_facing else { return };

    let dx = (enemy.x_pos as i64 - px as i64) >> 16; // crab_x - player_x
    let abs_dx = dx.abs();

    // ASM loc_3B142-loc_3B170: unfreeze when distance > 224px
    if abs_dx > CRAB_UNFREEZE_DISTANCE {
        enemy.crab_frozen = false;
        return;
    }

    // ASM loc_3B0E4-loc_3B11E: check freeze condition (within 64px)
    if abs_dx <= CRAB_FREEZE_DISTANCE {
        // ASM loc_3B0E4: player within 64px
        // Check if player is facing the crab:
        // - If player faces right (Direction::Right in native, $FF in ASM),
        //   crab must also face right for freeze
        // - If player faces left (Direction::Left in native, 0 in ASM),
        //   crab must also face left for freeze
        let player_faces_crab = match (pfacing, enemy.facing) {
            (Direction::Right, Direction::Right) => dx < 0, // player to left of crab, facing right toward it
            (Direction::Left, Direction::Left) => dx > 0, // player to right of crab, facing left toward it
            _ => false,
        };

        if player_faces_crab {
            // ASM loc_3B104: freeze the crab
            // sf is_animated(a3) - stop animation
            // bset #7,object_meta(a3) - set freeze bit
            // move.l #0,$50(a5) - zero x_velocity
            enemy.crab_frozen = true;
            enemy.x_velocity = 0;
            return;
        }

        // ASM loc_3B142-loc_3B16C: within 64px, check if player moving toward crab
        // If player moving away, unfreeze
        if let Some(p_vel) = player_x_velocity {
            // Player to left of crab (dx > 0), moving right (p_vel > 0): moving toward -> keep frozen
            // Player to left of crab (dx > 0), not moving right: moving away -> unfreeze
            // Player to right of crab (dx < 0), moving left (p_vel < 0): moving toward -> keep frozen
            // Player to right of crab (dx < 0), not moving left: moving away -> unfreeze
            let player_moving_toward_crab = if dx > 0 {
                p_vel > 0 // player to left, moving right toward crab
            } else {
                p_vel < 0 // player to right, moving left toward crab
            };
            if !player_moving_toward_crab {
                enemy.crab_frozen = false;
            }
        }
    }
    // ASM loc_3B11E: outside 64px but within 224px, keep current state
    // (already frozen stays frozen, already unfrozen stays unfrozen)
}

/// Tank fires a horizontal bullet at intervals while pacing.
pub(crate) fn step_tank_fire(enemy: &mut ActiveEnemy, projectiles: &mut Vec<EnemyProjectile>) {
    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }
    enemy.wait_timer = TANK_FIRE_INTERVAL;

    if projectiles.len() >= MAX_PROJECTILES {
        return;
    }

    let x_velocity = horizontal_velocity(enemy.facing, TANK_BULLET_SPEED);
    projectiles.push(EnemyProjectile {
        x_pos: enemy.x_pos,
        y_pos: enemy.y_pos.wrapping_sub(8 << 16), // spawn above tank body
        x_velocity,
        y_velocity: 0,
        lifetime: TANK_BULLET_LIFETIME,
        friendly: false,
        gravity: 0,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
        ..Default::default()
    });
}

/// Boss tracking movement: chase the player with fixed velocity components.
/// HeadyMetal ASM: x_vel ±$8000 towards player, y_vel $4800 down / -$B000 up,
/// targeting 40px above player. Other bosses use the same pattern at different speeds.
pub(crate) fn step_tracker(
    enemy: &mut ActiveEnemy,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    let (Some(px), Some(py)) = (player_x_pos, player_y_pos) else {
        return;
    };

    let ex = enemy.x_pos as i64;
    let ey = enemy.y_pos as i64;
    let target_y = (py as i64).saturating_sub(0x28 << 16); // 40px above player

    // Horizontal: chase player at $8000 (0.5 px/frame)
    // HeadyMetal ASM: X velocity forced to 0 during first 4 hit phases ($3E < 4).
    // Phase = 37 - current_hp, so phase < 4 means hp > 33.
    let x_chase_speed: i32 =
        if matches!(enemy.family, EnemyFamily::HeadyMetal) && enemy.hit_points > 33 {
            0 // Early phases: vertical-only tracking
        } else {
            0x8000
        };
    if (px as i64) > ex {
        enemy.x_velocity = x_chase_speed;
        enemy.facing = Direction::Right;
    } else {
        enemy.x_velocity = -x_chase_speed;
        enemy.facing = Direction::Left;
    }

    // Vertical: faster upward (-$B000) than downward ($4800)
    if target_y < ey {
        enemy.y_velocity = -0xB000;
    } else {
        enemy.y_velocity = 0x4800;
    }

    // Apply velocities
    enemy.x_pos = (ex + i64::from(enemy.x_velocity)).clamp(0, i64::from(u32::MAX)) as u32;
    enemy.y_pos = (ey + i64::from(enemy.y_velocity)).clamp(0, i64::from(u32::MAX)) as u32;
}

/// Boss periodic projectile fire interval (frames).
pub(crate) const BOSS_FIRE_INTERVAL: u16 = 180; // ~3 seconds
/// Boss skewer speed (Shishkaboss downward projectiles).
pub(crate) const BOSS_SKEWER_SPEED: i32 = 0x2_0000;

/// Bosses periodically fire projectiles at the player.
pub(crate) fn step_boss_fire(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }
    // HeadyMetal ASM: fire rate is animation-driven, faster in later phases.
    // Phase = 37 - hp. Phases < 6 (hp > 31): slower animation → ~120 frame interval.
    // Phases >= 6 (hp <= 31): rapid-fire animation → ~30 frame interval.
    let interval = if matches!(enemy.family, EnemyFamily::HeadyMetal) {
        if enemy.hit_points > 31 { 120 } else { 30 }
    } else {
        BOSS_FIRE_INTERVAL
    };
    enemy.wait_timer = interval;

    if projectiles.len() >= MAX_PROJECTILES {
        return;
    }

    match enemy.family {
        EnemyFamily::HeadyMetal => {
            // Spawn homing eye projectile with gravity
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                x_velocity: 0,
                y_velocity: 0,
                lifetime: 120,
                gravity: 0x2710,
                friendly: false,
                homing_accel: 0x7D0, // HeadyMetal ASM
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }
        EnemyFamily::Shishkaboss => {
            // Downward skewer from above
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                x_velocity: 0,
                y_velocity: BOSS_SKEWER_SPEED,
                lifetime: 90,
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }
        EnemyFamily::BoomerangBosses => {
            // Boomerang arc: fast horizontal, slight upward
            let dir = if let Some(px) = player_x_pos {
                if px >= enemy.x_pos { 1i32 } else { -1 }
            } else {
                match enemy.facing {
                    Direction::Right => 1,
                    Direction::Left => -1,
                }
            };
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                x_velocity: dir * 0x3_0000,
                y_velocity: -0x1_0000,
                lifetime: 60,
                gravity: 0x1000,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }
        EnemyFamily::BagelBrothers => {
            // Fast horizontal dash projectile
            let (Some(px), Some(_py)) = (player_x_pos, player_y_pos) else {
                return;
            };
            let dir = if px >= enemy.x_pos { 1i32 } else { -1 };
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                x_velocity: dir * 0x3_0000, // ASM: 3 px/frame (word)
                y_velocity: 0,
                lifetime: 50, // ASM: 50 frames moving phase
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }
        _ => {}
    }
}

pub(crate) fn spawn_x_pos(family: EnemyFamily, object_x: u32) -> u32 {
    match family {
        EnemyFamily::Dragon => (object_x << 16).saturating_add(DRAGON_SPAWN_X_OFFSET_FP),
        _ => object_x << 16,
    }
}

#[allow(dead_code)]
pub(crate) fn spawn_y_pos(family: EnemyFamily, object_y: u32) -> u32 {
    spawn_y_pos_sub_36FF4(family, object_y, 0)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_spawn_y_adjustment_sub_36FF4(blank_top_rows: u16) -> u32 {
    // ASM sub_36FF4 (kid.asm:38882): load `Number_blank_top_rows`, shift left
    // by 4, and add that pixel offset to `y_pos(a3)`.
    u32::from(blank_top_rows) << 4
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn spawn_y_pos_sub_36FF4(
    family: EnemyFamily,
    object_y: u32,
    blank_top_rows: u16,
) -> u32 {
    let object_y = object_y.saturating_add(enemy_spawn_y_adjustment_sub_36FF4(blank_top_rows));
    match family {
        EnemyFamily::Dragon => object_y.saturating_add(DRAGON_SPAWN_Y_OFFSET_PX) << 16,
        EnemyFamily::DragonWalking => {
            object_y.saturating_add(DRAGON_WALKING_SPAWN_Y_OFFSET_PX) << 16
        }
        _ => object_y << 16,
    }
}

pub(crate) fn spawn_x_velocity(family: EnemyFamily, enemy_level: u8) -> i32 {
    match family {
        EnemyFamily::Cloud => {
            // Cloud.asm dword_35B38: speeds indexed by enemy_level
            const CLOUD_SPEEDS: [i32; 3] = [-0x3000, -0x5000, -0x7000];
            CLOUD_SPEEDS[usize::from(enemy_level).min(CLOUD_SPEEDS.len() - 1)]
        }
        // Sphere.asm: fixed x_vel = $10000
        EnemyFamily::Sphere => 0x10000,
        // Fireball.asm: x_vel = max_upwards >> 1, indexed by enemy_level
        EnemyFamily::Fireball => {
            const FIREBALL_MAX_UPWARD: [i32; 3] = [-0x30000, -0x40000, -0x50000];
            FIREBALL_MAX_UPWARD[usize::from(enemy_level).min(2)] >> 1
        }
        // Big_Hopping_Skull.asm: move.w #1,x_vel → $10000 FP16.16
        EnemyFamily::BigHoppingSkull => BIG_SKULL_INIT_X_SPEED,
        _ => 0,
    }
}

pub(crate) fn spawn_facing(family: EnemyFamily, enemy_x: u32, player_x: u32) -> Direction {
    match family {
        // Hand always starts facing right (Hand.asm init)
        EnemyFamily::Hand => Direction::Right,
        // Drips always start facing right (Drip.asm init)
        EnemyFamily::Drips => Direction::Right,
        // ASM Dragon.asm loc_3CD70: face TOWARD the player at spawn.
        // `move.w x_pos(a4),d4; cmp.w x_pos(a3),d4; blt.s loc_3CD82; st x_direction(a3)`
        // x_direction=0 → LEFT (default VRAM orientation), x_direction=$FF → RIGHT (flipped).
        // kid.x >= dragon.x → st x_direction → $FF → faces RIGHT (toward kid on the right).
        // kid.x < dragon.x → skip → x_direction stays 0 → faces LEFT (toward kid on the left).
        _ => {
            if player_x < enemy_x {
                Direction::Left
            } else {
                Direction::Right
            }
        }
    }
}

pub(crate) fn spawn_y_velocity(family: EnemyFamily) -> i32 {
    match family {
        // Big_Hopping_Skull.asm: move.w #$FFFE,y_vel → $FFFE0000 FP16.16
        EnemyFamily::BigHoppingSkull => BIG_SKULL_INIT_Y_SPEED,
        _ => 0,
    }
}

pub(crate) fn spawn_x_acceleration(family: EnemyFamily) -> i32 {
    match family {
        EnemyFamily::UFO => UFO_ACCELERATION,
        EnemyFamily::Tornado => TORNADO_ACCELERATION,
        _ => 0,
    }
}

pub(crate) fn spawn_y_acceleration(family: EnemyFamily) -> i32 {
    match family {
        EnemyFamily::UFO => -UFO_ACCELERATION,
        EnemyFamily::Tornado => -TORNADO_ACCELERATION,
        _ => 0,
    }
}

/// Compute initial HP from the enemy family and the raw `enemy_level` field
/// in the layout. Each family's ASM init adds a base offset to `enemy_level`
/// to produce the actual HP value.
pub(crate) fn spawn_hit_points(family: EnemyFamily, enemy_level: u8) -> u8 {
    let base = match family {
        EnemyFamily::Goat => 3,
        EnemyFamily::Armadillo => 1,
        EnemyFamily::Crab => 2,
        EnemyFamily::Tank => 2,
        EnemyFamily::Scorpion => 3,
        EnemyFamily::Driller => 2,
        EnemyFamily::Cloud => 1,
        EnemyFamily::Sphere => 1,
        EnemyFamily::BigHoppingSkull => 2,
        EnemyFamily::EmoRock => 1,
        EnemyFamily::Archer => 1,
        EnemyFamily::Fireball => 1,
        // Tier 3-4 families: approximate base HP from ASM init routines
        EnemyFamily::FireDemon => 3,  // HP 3-5
        EnemyFamily::Robot => 2,      // HP 2-3
        EnemyFamily::TarMonster => 2, // HP {2,4,6}
        EnemyFamily::Orca => 4,       // fixed 4 HP
        EnemyFamily::UFO => 1,
        EnemyFamily::Ninja => 2, // HP 2-3
        EnemyFamily::Lion => 6,  // HP 6-8
        EnemyFamily::SpinningTwins => 1,
        EnemyFamily::Drips => 1,
        EnemyFamily::Hand => 1,
        EnemyFamily::Tornado => 1,
        EnemyFamily::DragonWalking => 1, // HP 1-3 (ASM Dragon.asm:65 addq.w #1)
        EnemyFamily::HeadyMetal => 37,   // ASM: $25 = 37 hits
        // Multi-head bosses: HP is set per-head in activate_enemies via boss_head_config.
        // These values are fallback defaults (head 1 HP).
        EnemyFamily::Shishkaboss => 20,
        EnemyFamily::BoomerangBosses => 30,
        EnemyFamily::BagelBrothers => 30,
        // Existing families: Diamond and MiniHoppingSkull use raw level as
        // HP with a minimum of 1 (legacy behaviour preserved).
        _ => 0,
    };
    (enemy_level + base).max(1)
}

pub(crate) fn spawn_wait_timer(family: EnemyFamily, enemy_level: u8) -> u16 {
    match family {
        // EmoRock: base $3C (60) frames hover wait before first drop attempt.
        // Original adds a random offset; we use the fixed base.
        EnemyFamily::EmoRock => 60,
        // Tank: fire first bullet after one full interval.
        EnemyFamily::Tank => TANK_FIRE_INTERVAL,
        // Archer: fire first arrow after one full interval (HP1 rate).
        EnemyFamily::Archer => ARCHER_FIRE_TIMER[0],
        // Lion: charge timer counts down from $3C (60 frames) before first charge.
        EnemyFamily::Lion => LION_CHARGE_TIMER_INIT,
        // Robot: ASM init $54(a5) low word = 1 (fires on 2nd frame).
        EnemyFamily::Robot => 1,
        // TarMonster: starts submerged, waits before first emergence.
        EnemyFamily::TarMonster => TAR_MONSTER_SUBMERGED_FRAMES,
        // Drips: first drop after one interval.
        EnemyFamily::Drips => DRIPS_DROP_INTERVAL,
        // DragonWalking: first breath after cooldown.
        EnemyFamily::DragonWalking => DRAGON_WALKING_BREATH_INITIAL,
        // ASM UFO.asm `unk_3588C`: 60/40/30 by raw layout level.
        EnemyFamily::Tornado => TORNADO_PATTERN_FRAMES[usize::from(enemy_level).min(2)],
        // SpinningTwins: orbit-to-walk is collision-triggered, no timer needed.
        EnemyFamily::SpinningTwins => 0,
        // Bosses: fire first projectile after initial delay.
        EnemyFamily::HeadyMetal
        | EnemyFamily::Shishkaboss
        | EnemyFamily::BoomerangBosses
        | EnemyFamily::BagelBrothers => BOSS_FIRE_INTERVAL,
        _ => 0,
    }
}

pub(crate) fn spawn_dragon_phase(family: EnemyFamily) -> DragonFlyingState {
    match family {
        EnemyFamily::Dragon => DragonFlyingState::Hovering,
        _ => DragonFlyingState::None,
    }
}

pub(crate) fn horizontal_velocity(facing: Direction, speed: i32) -> i32 {
    match facing {
        Direction::Left => -speed,
        Direction::Right => speed,
    }
}

pub(crate) fn enemy_layout_view(
    layout: &EnemyLayout,
    camera_x_px: u32,
    camera_y_px: u32,
) -> EnemyLayoutView<'_> {
    if layout.segments.is_empty() {
        return EnemyLayoutView {
            segment_index: 0,
            enemy_type_slots: &layout.enemy_type_slots,
            objects: &layout.objects,
        };
    }

    let position = if layout.axis_uses_y_position {
        camera_y_px.min(u32::from(u16::MAX)) as u16
    } else {
        camera_x_px.min(u32::from(u16::MAX)) as u16
    };
    let mut selected_index = 0usize;
    for (index, segment) in layout.segments.iter().enumerate() {
        if position >= segment.lower_bound {
            selected_index = index;
        }
        if position < segment.upper_bound {
            break;
        }
    }
    let segment = &layout.segments[selected_index];
    EnemyLayoutView {
        segment_index: selected_index,
        enemy_type_slots: &segment.enemy_type_slots,
        objects: &segment.objects,
    }
}

pub(crate) fn resolved_enemy_id(enemy_type_slots: &[u16], object_type: u8) -> Option<u8> {
    if object_type & 0x80 == 0 {
        return Some(object_type);
    }
    let slot_index = usize::from(object_type & 0x7F);
    let raw_slot = *enemy_type_slots.get(slot_index)?;
    if raw_slot == 0xFFFF {
        return None;
    }
    u8::try_from(raw_slot & 0x3FFF).ok()
}

/// Extract the palette variant (0, 1, or 2) from the enemy type slot.
pub(crate) fn resolved_enemy_palette(enemy_type_slots: &[u16], object_type: u8) -> u8 {
    if object_type & 0x80 == 0 {
        return 0;
    }
    let slot_index = usize::from(object_type & 0x7F);
    let Some(&raw_slot) = enemy_type_slots.get(slot_index) else {
        return 0;
    };
    ((raw_slot >> 14) & 3) as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyArtSlotAssignmentSub36E84 {
    pub(crate) slot_index: u16,
    pub(crate) palette_line: u8,
    pub(crate) vram_tile: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_art_slot_assignment_sub_36E84(
    enemy_type_slots: &[u16],
    enemy_id: u16,
) -> EnemyArtSlotAssignmentSub36E84 {
    // ASM sub_36E84 (kid.asm:38805): compare the 12-bit enemy id against the
    // first two header slots and otherwise fall back to slot 2, then map that
    // slot to `EnemyArt_PaletteLines` / `EnemyArt_VRAMTileAddresses`.
    const ENEMY_ART_PALETTE_LINES: [u8; 4] = [1, 2, 2, 0];
    const ENEMY_ART_VRAM_TILE_ADDRESSES: [u16; 3] = [0x02FB, 0x0409, 0x0517];

    let enemy_id = enemy_id & 0x0FFF;
    let slot_index = if enemy_type_slots
        .first()
        .is_some_and(|slot| slot & 0x0FFF == enemy_id)
    {
        0
    } else if enemy_type_slots
        .get(1)
        .is_some_and(|slot| slot & 0x0FFF == enemy_id)
    {
        1
    } else {
        2
    };

    EnemyArtSlotAssignmentSub36E84 {
        slot_index,
        palette_line: ENEMY_ART_PALETTE_LINES[usize::from(slot_index)],
        vram_tile: ENEMY_ART_VRAM_TILE_ADDRESSES[usize::from(slot_index)],
    }
}

impl NativeRuntime {
    fn enemy_loading_enabled(&self) -> bool {
        !matches!(self.boot_state, BootState::DemoPlay { .. })
            || (self.current_level == LevelId(demo::DEMO_1_LEVEL)
                && self
                    .demo_startup_frame()
                    .zip(self.demo_uses_second_recording())
                    .is_some_and(|(frame, use_demo_2)| {
                        frame >= demo::enemy_warmup_frame(use_demo_2)
                    }))
    }

    pub(crate) fn refresh_active_roster(&mut self) {
        if self.paused {
            return;
        }
        if self.gameplay_hibernating_for_transform() {
            return;
        }
        self.active_enemy_roster.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        if self.level_skip_resume_hold {
            self.active_enemies.clear();
            self.block_state.evanescent_blocks.clear();
            self.block_state.shooter_blocks.clear();
            self.block_state.shooter_headbutt_workers.clear();
            self.block_state.ghost_groups.clear();
            return;
        }
        if !matches!(
            self.boot_state,
            BootState::InGame | BootState::DemoPlay { .. }
        ) {
            return;
        }

        let Some(slice) = self.opening_slice else {
            return;
        };
        if !matches!(
            self.player_lifecycle,
            PlayerLifecycle::Alive
                | PlayerLifecycle::DeathWindup { .. }
                | PlayerLifecycle::Respawning { .. }
                | PlayerLifecycle::LevelComplete { .. }
        ) {
            return;
        }
        let window = ActivationWindow::from_camera(slice.camera);
        let camera_x_px = slice.camera.x_pos >> 16;
        let camera_y_px = slice.camera.y_pos >> 16;
        let level_bounds = self.level_bounds(self.current_level);
        let enemy_layout = self.enemy_layout(self.current_level).cloned();
        let platform_layout = self.platform_layout(self.current_level).cloned();
        let blank_top_rows = self
            .level_header(self.current_level)
            .map(|header| {
                terrain::blank_top_rows_from_height_flag_bits_sub_140F6(header.height_flag_bits)
            })
            .unwrap_or(0);
        if let Some(enemy_layout) = enemy_layout.as_ref() {
            if self.enemy_loading_enabled() {
                activate_enemies_with_rng(
                    &mut self.active_enemies,
                    enemy_layout,
                    window,
                    camera_x_px,
                    camera_y_px,
                    level_bounds,
                    slice.player.x_pos,
                    blank_top_rows,
                    &self.killed_enemies,
                    &mut self.rom_random,
                );
                step_active_enemies_with_rng(
                    &mut self.active_enemies,
                    self.block_grid.as_ref(),
                    Some(slice.player.x_pos),
                    Some(slice.player.y_pos),
                    Some(camera_x_px),
                    Some(slice.player.facing),
                    Some(slice.player.x_velocity),
                    self.progression.current_form,
                    Some(&mut self.projectiles),
                    &mut self.rom_random,
                    &mut self.drilled_blocks,
                );
                // Apply drilled blocks to the block grid after enemy step.
                // Driller.asm loc_3BE78 destroys Smash/Ice blocks during pacing.
                if let Some(ref mut grid) = self.block_grid {
                    for (col, row) in self.drilled_blocks.drain(..) {
                        grid.set_cell_at(col, row, terrain::Cell::from_be_bytes(0, 0));
                    }
                }
                self.drilled_blocks.clear();
                self.active_enemy_roster = self
                    .active_enemies
                    .iter()
                    .map(ActiveEnemy::snapshot)
                    .collect();
            } else {
                // ORIGINAL_QUIRK: during title-screen demo playback, the game
                // suppresses enemy loading for every level except Knights Isle.
                self.active_enemies.clear();
                self.active_enemy_roster.clear();
            }
        }
        if let Some(platform_layout) = platform_layout.as_ref() {
            let mut objects = slice.objects;
            objects
                .platform_loader
                .advance_to_window(platform_layout, window);
            self.active_platform_roster = objects
                .platform_loader
                .active_platforms(platform_layout, window);
            let script_table = self.platform_script_table.as_ref();
            self.active_platforms = platform_layout
                .placements
                .iter()
                .take(objects.platform_loader.next_index)
                .filter(|p| platform_is_active(p, window))
                .map(|p| ActivePlatform::from_placement(p, script_table))
                .collect();
            self.opening_slice = Some(OpeningSliceState { objects, ..slice });
        }
    }
}
