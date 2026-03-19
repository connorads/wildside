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

/// Persistent per-enemy state that survives across frames.
/// Enemies are activated when they enter the camera window and
/// deactivated when they leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActiveEnemy {
    /// Index into the enemy layout segment list, or 0 for legacy flat layouts.
    pub(crate) segment_index: usize,
    /// Index into the level's enemy layout objects list.
    pub(crate) layout_index: usize,
    pub(crate) family: EnemyFamily,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
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
    /// Walk speed override from tracking callback ($50(a5)). When non-zero, step_pacer
    /// uses this instead of pacer_walk_speed(). Set by Scorpion tracking (Scorpion.asm loc_3DD7E).
    pub(crate) walk_speed_override: i32,
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
pub(crate) const DRAGON_SPAWN_Y_OFFSET_PX: u32 = 1;
pub(crate) const DRAGON_WALKING_SPAWN_Y_OFFSET_PX: u32 = 1;

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
        if matches!(self.family, EnemyFamily::TarMonster) {
            return matches!(
                self.tar_monster_state,
                TarMonsterState::Visible | TarMonsterState::Attacking
            );
        }
        true
    }

    pub(crate) fn snapshot(&self) -> NativeObjectSnapshot {
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
        }
    }
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
pub(crate) fn activate_enemies(
    active: &mut Vec<ActiveEnemy>,
    layout: &EnemyLayout,
    window: ActivationWindow,
    camera_x_px: u32,
    camera_y_px: u32,
    player_x: u32,
) {
    let view = enemy_layout_view(layout, camera_x_px, camera_y_px);
    // Most enemies despawn on spawn-point exit. Flyers keep their live object
    // around longer; the Level 1 Dragon continues drifting after its layout
    // anchor scrolls behind the camera.
    active.retain(|enemy| enemy_should_remain_active(enemy, &view, window));

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
        let family = EnemyFamily::from_layout_object(view.enemy_type_slots, object);
        // Dragon Flying uses hit_points_plus_one as script variant selector (ASM $46(a5)),
        // not as HP. All other families treat it as enemy_level for HP scaling.
        let enemy_level = object.hit_points_plus_one.saturating_sub(1) as u8;
        let spawn_y = spawn_y_pos(family, object_y);
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

            active.push(ActiveEnemy {
                segment_index: view.segment_index,
                layout_index: index,
                family,
                x_pos: head_x,
                y_pos: head_y,
                x_velocity: spawn_x_velocity(family, enemy_level),
                y_velocity: spawn_y_velocity(family),
                x_acceleration: spawn_x_acceleration(family),
                y_acceleration: spawn_y_acceleration(family),
                facing: spawn_facing(family, head_x, player_x),
                on_ground: matches!(family.archetype(), EnemyArchetype::Pacer),
                hit_points,
                grounded_frames: 0,
                damage_flash_frames: 0,
                palette_variant: resolved_enemy_palette(view.enemy_type_slots, object.object_type),
                wait_timer: spawn_wait_timer(family),
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
                walk_speed_override: 0,
            });
        }
    }
}

/// Step all active enemies: apply movement, gravity, and terrain collision.
pub(crate) fn step_active_enemies(
    active: &mut [ActiveEnemy],
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    player_form: PlayerForm,
    mut projectiles: Option<&mut Vec<EnemyProjectile>>,
) {
    for enemy in active.iter_mut() {
        enemy.damage_flash_frames = enemy.damage_flash_frames.saturating_sub(1);
        match enemy.family.archetype() {
            EnemyArchetype::Collectible | EnemyArchetype::Inert => {}
            EnemyArchetype::Bouncer => {
                if matches!(enemy.family, EnemyFamily::BigHoppingSkull) {
                    step_big_hopping_skull(enemy, block_grid);
                } else {
                    step_bouncer(enemy, block_grid);
                }
            }
            EnemyArchetype::Flyer => {
                if matches!(enemy.family, EnemyFamily::Dragon) {
                    step_dragon(enemy, player_x_pos, player_y_pos, &mut projectiles);
                } else {
                    step_flyer(enemy, block_grid);
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
                    step_robot(enemy, block_grid, player_x_pos, &mut projectiles);
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
                    } else {
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

pub(crate) fn step_flyer(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
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
        step_tornado(enemy, block_grid);
        return;
    }

    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
}

/// Cloud post-bounce idle frames: ASM Cloud.asm sets d3=$3C (60) after wall reversal.
pub(crate) const CLOUD_POST_BOUNCE_IDLE: u8 = 60;

/// Cloud flies horizontally, reversing direction on wall contact.
/// After hitting a wall, idles for 60 frames before resuming (Cloud.asm).
pub(crate) fn step_cloud(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    // Post-bounce idle: count down grounded_frames as idle timer
    if enemy.grounded_frames > 0 {
        enemy.grounded_frames -= 1;
        return;
    }

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
            enemy.grounded_frames = CLOUD_POST_BOUNCE_IDLE;
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

/// Tornado: acceleration-based flight. ASM: shared `Enemy_HandleAcceleration`
/// with UFO. Default acceleration from `loc_35444` is $C00 when random table
/// entry is 0. Terminal velocity from table entries: $18000 is common.
/// Cycle timer from `unk_3588C`: 60/40/30 by enemy_level.
// DEVIATION: ASM picks from 16 random patterns (unk_35344) each sub-cycle
// using (random_byte XOR x_pos) & 0xF. Each pattern sets target_x_vel,
// target_y_vel, x_accel, y_accel. Native uses fixed acceleration reversal.
// RNG-dependent — would require full RNG synchronisation to match oracle.
pub(crate) const TORNADO_ACCELERATION: i32 = 0xC00;
pub(crate) const TORNADO_TERMINAL_VELOCITY: i32 = 0x1_8000;
/// Per-level cycle timers: level 0 = 60, level 1 = 40, level 2 = 30 (unk_3588C).
pub(crate) const TORNADO_PATTERN_FRAMES: [u16; 3] = [60, 40, 30];

pub(crate) fn step_tornado(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    // Apply acceleration
    enemy.x_velocity = (enemy.x_velocity + enemy.x_acceleration)
        .clamp(-TORNADO_TERMINAL_VELOCITY, TORNADO_TERMINAL_VELOCITY);
    enemy.y_velocity = (enemy.y_velocity + enemy.y_acceleration)
        .clamp(-TORNADO_TERMINAL_VELOCITY, TORNADO_TERMINAL_VELOCITY);

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    // Periodic direction change — cycle timer from unk_3588C per enemy_level
    if enemy.wait_timer == 0 {
        let level = (enemy.hit_points.saturating_sub(1) as usize).min(2);
        enemy.wait_timer = TORNADO_PATTERN_FRAMES[level];
        enemy.x_acceleration = -enemy.x_acceleration;
        enemy.y_acceleration = -enemy.y_acceleration;
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

/// Fireball max upward velocity per HP tier (unk_32BA6 second column).
/// Level 0: -$30000, Level 1: -$40000, Level 2: -$50000.
pub(crate) const FIREBALL_UPWARD_CAP: [i32; 3] = [-0x3_0000, -0x4_0000, -0x5_0000];

pub(crate) fn bouncer_gravity(enemy: &ActiveEnemy) -> i32 {
    match enemy.family {
        // Fireball.asm: gravity indexed by enemy_level ($1000/$2000/$3000)
        EnemyFamily::Fireball => {
            const FIREBALL_GRAVITY: [i32; 3] = [0x1000, 0x2000, 0x3000];
            let level = enemy.hit_points.saturating_sub(1) as usize;
            FIREBALL_GRAVITY[level.min(2)]
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

pub(crate) fn step_bouncer(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
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
            if matches!(enemy.family, EnemyFamily::Fireball) {
                // Fireball.asm BounceFloorCeiling: negate + random perturbation + cap
                // DEVIATION: ASM adds ±$4000 random perturbation (RNG-dependent)
                let new_yv = -enemy.y_velocity;
                let tier = enemy.hit_points.saturating_sub(1).min(2) as usize;
                let cap = FIREBALL_UPWARD_CAP[tier];
                enemy.y_velocity = if new_yv < cap { cap } else { new_yv };
            } else {
                enemy.y_velocity = -enemy.y_velocity;
            }
            // Snap out of floor
            let floor_y_fp = (floor.floor_y_px as u32) << 16;
            if enemy.y_pos > floor_y_fp {
                enemy.y_pos = floor_y_fp;
            }
        }

        // Ceiling bounce
        let ceil = terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
        if ceil.hit && enemy.y_velocity < 0 {
            if matches!(enemy.family, EnemyFamily::Fireball) {
                // Fireball.asm BounceFloorCeiling: same formula for floor and ceiling
                // DEVIATION: ASM adds ±$4000 random perturbation (RNG-dependent)
                let new_yv = -enemy.y_velocity;
                let tier = enemy.hit_points.saturating_sub(1).min(2) as usize;
                let cap = FIREBALL_UPWARD_CAP[tier];
                enemy.y_velocity = if new_yv < cap { cap } else { new_yv };
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
                    let breath_speed = match enemy.facing {
                        Direction::Right => DRAGON_WALKING_BREATH_SPEED,
                        Direction::Left => -DRAGON_WALKING_BREATH_SPEED,
                    };
                    projs.push(EnemyProjectile {
                        x_pos: enemy.x_pos,
                        y_pos: (enemy.y_pos as i64 + (10 << 16)) as u32,
                        x_velocity: breath_speed,
                        y_velocity: 0,
                        lifetime: DRAGON_WALKING_BREATH_LIFETIME,
                        gravity: 0,
                        friendly: false,
                        homing_accel: 0,
                        speed_cap: 0,
                        wall_bounce: false,
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
        _ => window.contains(u32::from(object.x), u32::from(object.y)),
    }
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

/// Lion charge detection range — primary trigger is $60 (96px) at loc_3E450/3E604.
/// $A0 (160px) is used for turnaround hysteresis, not charge trigger.
pub(crate) const LION_DETECT_RANGE: i32 = 0x60;

/// Lion charge speed: $28000 for HP==8, $18000 otherwise (Lion.asm lines 36-48).
/// Also used as walk speed — ASM sets both $50(a5) and $5E(a5) from same value.
pub(crate) fn lion_charge_speed(hit_points: u8) -> i32 {
    if hit_points >= 8 { 0x2_8000 } else { 0x1_8000 }
}

/// Lion charge timer: starts at $3C (60 frames), counts down while pacing.
/// When timer expires and player is in range, Lion charges.
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

            // Count down charge timer
            if enemy.wait_timer > 0 {
                enemy.wait_timer -= 1;
                return;
            }

            // Timer expired — check if player is in range ($60 = 96px)
            let Some(px) = player_x_pos else {
                enemy.wait_timer = LION_CHARGE_TIMER_INIT;
                return;
            };
            let dx = (px as i64 - enemy.x_pos as i64) >> 16;
            if dx.abs() > i64::from(LION_DETECT_RANGE) {
                enemy.wait_timer = LION_CHARGE_TIMER_INIT;
                return;
            }

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
                });
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

/// Robot projectile speed: ~5 px/frame horizontal (Robot.asm).
pub(crate) const ROBOT_PROJECTILE_SPEED: i32 = 0x5_0000;

/// Step a Robot enemy: paces at HP-dependent speed, shoots at intervals,
/// then enters a fast charge phase.
pub(crate) fn step_robot(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    projectiles: &mut Option<&mut Vec<EnemyProjectile>>,
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
                // DEVIATION: ASM reload uses (random_byte >> 1) + $F (range 15-142).
                // Native uses fixed base — RNG-dependent, cosmetic timing difference.
                enemy.wait_timer = ROBOT_CHARGE_FLIP_BASE;
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
        });
    }
}

// --- DragonWalking constants ---
// From ASM Dragon.asm: initial cooldown $12C (300f, line 95), reset after breath $1F4 (500f, line 194).
pub(crate) const DRAGON_WALKING_BREATH_INITIAL: u16 = 300;
pub(crate) const DRAGON_WALKING_BREATH_RESET: u16 = 500;
pub(crate) const DRAGON_WALKING_BREATH_SPEED: i32 = 0x3_0000;
pub(crate) const DRAGON_WALKING_BREATH_LIFETIME: u16 = 68;
pub(crate) const DRAGON_WALKING_BREATH_HOLD_FRAMES: u16 = 70;
pub(crate) const DRAGON_WALKING_RECOVER_FRAMES: u16 = 5;
pub(crate) const DRAGON_WALKING_BREATH_OFFSET_PX: i32 = 0x1C;
const DRAGON_WALKING_PHASE_WALK: u8 = 0;
const DRAGON_WALKING_PHASE_BREATH: u8 = 1;
const DRAGON_WALKING_PHASE_RECOVER: u8 = 2;

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
    let look_ahead_x = if moving_right {
        cx + ENEMY_PROBE_HALF_WIDTH
    } else {
        cx - ENEMY_PROBE_HALF_WIDTH
    };
    !terrain::probe_floor(grid, look_ahead_x, cy).on_ground
}

fn step_dragon_walking_walk(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) -> bool {
    let Some(grid) = block_grid else {
        let walk_speed = pacer_walk_speed(enemy.family, enemy.hit_points);
        enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
        enemy.y_velocity = 0;
        enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
        return false;
    };

    let walk_speed = pacer_walk_speed(enemy.family, enemy.hit_points);
    let x_velocity = horizontal_velocity(enemy.facing, walk_speed);
    let moving_right = x_velocity >= 0;
    let next_x = (enemy.x_pos as i64 + i64::from(x_velocity)) as u32;
    let cx = (next_x >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
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
        enemy.x_pos = if moving_right {
            wall_fp.saturating_sub(edge_fp)
        } else {
            wall_fp + edge_fp
        };
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        return true;
    }

    let look_ahead_x = if moving_right {
        cx + ENEMY_PROBE_HALF_WIDTH
    } else {
        cx - ENEMY_PROBE_HALF_WIDTH
    };
    if !terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        return true;
    }

    enemy.x_pos = next_x;
    enemy.x_velocity = x_velocity;
    enemy.y_velocity = 0;
    false
}

/// Fire breath for a walking dragon (called after step_pacer handles movement).
pub(crate) fn step_dragon_walking(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x_pos: Option<u32>,
) {
    let blocked = if enemy.dragon_script_cmd == DRAGON_WALKING_PHASE_WALK {
        step_dragon_walking_walk(enemy, block_grid)
    } else {
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        dragon_walking_blocked(enemy, block_grid)
    };

    match enemy.dragon_script_cmd {
        DRAGON_WALKING_PHASE_BREATH => {
            if enemy.dragon_phase_frames > 0 {
                enemy.dragon_phase_frames -= 1;
                return;
            }
            enemy.dragon_script_cmd = DRAGON_WALKING_PHASE_RECOVER;
            enemy.dragon_phase_frames = DRAGON_WALKING_RECOVER_FRAMES;
            return;
        }
        DRAGON_WALKING_PHASE_RECOVER => {
            if enemy.dragon_phase_frames > 0 {
                enemy.dragon_phase_frames -= 1;
                return;
            }
            if let Some(facing) = player_relative_direction(enemy.x_pos, player_x_pos) {
                enemy.facing = facing;
            } else if blocked {
                reverse_enemy_direction(enemy);
            }
            enemy.dragon_script_cmd = DRAGON_WALKING_PHASE_WALK;
        }
        _ => {}
    }

    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }

    // Fire breath in facing direction
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
    });
    enemy.wait_timer = DRAGON_WALKING_BREATH_RESET;
    enemy.dragon_script_cmd = DRAGON_WALKING_PHASE_BREATH;
    enemy.dragon_phase_frames = DRAGON_WALKING_BREATH_HOLD_FRAMES;
    enemy.x_velocity = 0;
    enemy.y_velocity = 0;
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
        // Directly above or below.
        return if dy < 0 { 6 } else { 0x15 };
    }

    let abs_dx = dx.unsigned_abs();
    let abs_dy = dy.unsigned_abs();
    let ratio = ((abs_dy * 8) / abs_dx).min(0x3E) as usize;
    let sub = RATIO_TABLE[ratio.min(RATIO_TABLE.len() - 1)] as i32;

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

/// Step Archer: stationary enemy that faces player and fires arrows at intervals.
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

    // Don't exceed projectile limit.
    if projectiles.len() >= MAX_PROJECTILES {
        return;
    }

    // Compute angle and spawn arrow.
    let angle = archer_angle_to_player(enemy.x_pos, enemy.y_pos, px, py);
    let (x_byte, y_byte) = ARROW_ANGLE_TABLE[angle.min(ARROW_ANGLE_TABLE.len() - 1)];
    let x_velocity = (x_byte as i32) * 0x1000;
    let y_velocity = (y_byte as i32) * 0x1000;

    projectiles.push(EnemyProjectile {
        x_pos: enemy.x_pos,
        y_pos: enemy.y_pos,
        x_velocity,
        y_velocity,
        lifetime: ARCHER_FIRE_TIMER[level],
        friendly: false,
        gravity: 0,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
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

pub(crate) fn spawn_y_pos(family: EnemyFamily, object_y: u32) -> u32 {
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
        // All other families face toward the player at spawn
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

pub(crate) fn spawn_wait_timer(family: EnemyFamily) -> u16 {
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
        // Tornado: first cycle timer (level 0 = 60 frames).
        EnemyFamily::Tornado => TORNADO_PATTERN_FRAMES[0],
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

impl NativeRuntime {
    pub(crate) fn refresh_active_roster(&mut self) {
        if self.paused {
            return;
        }
        self.active_enemy_roster.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        if self.level_skip_resume_hold {
            self.active_enemies.clear();
            self.block_state.evanescent_blocks.clear();
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
        let enemy_layout = self.enemy_layout(self.current_level).cloned();
        let platform_layout = self.platform_layout(self.current_level).cloned();
        if let Some(enemy_layout) = enemy_layout.as_ref() {
            activate_enemies(
                &mut self.active_enemies,
                enemy_layout,
                window,
                camera_x_px,
                camera_y_px,
                slice.player.x_pos,
            );
            step_active_enemies(
                &mut self.active_enemies,
                self.block_grid.as_ref(),
                Some(slice.player.x_pos),
                Some(slice.player.y_pos),
                self.progression.current_form,
                Some(&mut self.projectiles),
            );
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
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
