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
pub(crate) const MINI_SKULL_INITIAL_Y_SPEED_FP: i32 = 0x0002_8000;
pub(crate) const MINI_SKULL_LEVEL_INITIAL_Y_SPEED_BONUS_FP: i32 = 0x0000_1000;
pub(crate) const MINI_SKULL_GROUND_DELAY_FRAMES: u8 = 21; // ASM Mini_Hopping_Skull.asm:99,141 — dbf #$14 = 21 iterations

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionInitVisibleState {
    pub(crate) status_flag_13: bool,
    pub(crate) object_meta: u16,
    pub(crate) palette_line: u8,
    pub(crate) current_hp: u16,
    pub(crate) facing: Direction,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_init_visible_state_loc_3df92(
    enemy_level: u16,
    lion_x_word: u16,
    kid_x_word: u16,
) -> LionInitVisibleState {
    // ASM Lion.asm loc_3DF92: after the signed spawn-facing compare, mark the
    // object visible via `$13(a3)`, seed object_meta `$12`, initialise art,
    // force palette line 2, set object_meta bit 7, and store `enemy_level+6`
    // into current_hp.
    let facing = if (kid_x_word as i16) < (lion_x_word as i16) {
        Direction::Left
    } else {
        Direction::Right
    };

    LionInitVisibleState {
        status_flag_13: true,
        object_meta: 0x0012 | 0x0080,
        palette_line: 2,
        current_hp: enemy_level.wrapping_add(6),
        facing,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionInitMotionProfile {
    pub(crate) walk_speed: i32,
    pub(crate) charge_speed: i32,
    pub(crate) animation_script: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_init_motion_profile_loc_3dfd8(current_hp: u16) -> LionInitMotionProfile {
    // ASM Lion.asm loc_3DFD8 is reached by `cmpi.w #8,d7; bne.s`: every HP
    // value except exactly 8 uses `$18000` for both `$50(a5)` and `$5E(a5)`
    // and installs `stru_3DEDA`.
    if current_hp == 8 {
        LionInitMotionProfile {
            walk_speed: 0x0002_8000,
            charge_speed: 0x0002_8000,
            animation_script: "stru_3DEF4",
        }
    } else {
        LionInitMotionProfile {
            walk_speed: 0x0001_8000,
            charge_speed: 0x0001_8000,
            animation_script: "stru_3DEDA",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionInitRuntimeFields {
    pub(crate) ufo_beam_interval: u16,
    pub(crate) ufo_beam_segments: u16,
    pub(crate) tracking_callback: &'static str,
    pub(crate) animation_script: &'static str,
    pub(crate) shared_walker_state: i16,
    pub(crate) charge_timer: u16,
    pub(crate) collision_routine: &'static str,
    pub(crate) random_phase_byte: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_init_runtime_fields_loc_3dff0(
    motion_profile: LionInitMotionProfile,
    random_byte: u8,
) -> LionInitRuntimeFields {
    // ASM Lion.asm loc_3DFF0: common tail after the HP-specific motion branch.
    // It writes `$4A/$48/$6A`, initialises the selected animation, seeds
    // `$42(a3)` and `$54(a5)`, points a0 at the Lion collision routine, then
    // stores the RNG byte in `$71(a5)` with the original zero-byte remap to `$4B`.
    LionInitRuntimeFields {
        ufo_beam_interval: 0x002F,
        ufo_beam_segments: 0x0014,
        tracking_callback: "LionTrackingCallback",
        animation_script: motion_profile.animation_script,
        shared_walker_state: 0x000A,
        charge_timer: 0x003C,
        collision_routine: "LionCollisionRoutine",
        random_phase_byte: if random_byte == 0 { 0x4B } else { random_byte },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionFacingDispatch {
    LeftPacing,
    RightPacing,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_init_facing_dispatch_loc_3e030(facing: Direction) -> LionFacingDispatch {
    // ASM Lion.asm loc_3E030: `tst.b x_direction(a3); beq.w loc_3E546;
    // bra.w loc_3E392`. For Lion art, clear x_direction is the left-facing
    // pacing loop and set x_direction is the right-facing pacing loop.
    match facing {
        Direction::Left => LionFacingDispatch::LeftPacing,
        Direction::Right => LionFacingDispatch::RightPacing,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionTrackingGate {
    Hold,
    TrackKid,
    ClearLatchAndTrackKid,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_tracking_gate_loc_3e054(
    lion_x_word: u16,
    kid_x_word: u16,
    range_latch_set: bool,
) -> LionTrackingGate {
    let lion_x = i16::from_ne_bytes(lion_x_word.to_ne_bytes());
    let kid_x = i16::from_ne_bytes(kid_x_word.to_ne_bytes());
    let delta = lion_x.wrapping_sub(kid_x);
    // ASM Lion.asm loc_3E054: after the close [-$3F,+$40] band has been
    // rejected, clear `$4E(a5)` only outside the wider [-$9F,+$9F] band.
    // While `$4E(a5)` is clear, fall straight through to the facing update.
    if !range_latch_set {
        return LionTrackingGate::TrackKid;
    }

    if (-0x9F..=0x9F).contains(&delta) {
        LionTrackingGate::Hold
    } else {
        LionTrackingGate::ClearLatchAndTrackKid
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct LionLatchClearResult {
    pub(crate) range_latch_set: bool,
    pub(crate) facing: Direction,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_clear_tracking_latch_loc_3e066(
    lion_x_word: u16,
    kid_x_word: u16,
) -> LionLatchClearResult {
    let lion_x = i16::from_ne_bytes(lion_x_word.to_ne_bytes());
    let kid_x = i16::from_ne_bytes(kid_x_word.to_ne_bytes());
    let delta = lion_x.wrapping_sub(kid_x);
    // ASM Lion.asm loc_3E066: `sf $4E(a5)` clears the tracking latch before
    // the same signed `d4 = lion_x - kid_x` value selects the Lion's facing.
    LionLatchClearResult {
        range_latch_set: false,
        facing: if delta < 0 {
            Direction::Right
        } else {
            Direction::Left
        },
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_track_kid_nonnegative_delta_facing_loc_3e06a(
    lion_x_word: u16,
    kid_x_word: u16,
) -> Direction {
    let lion_x = i16::from_ne_bytes(lion_x_word.to_ne_bytes());
    let kid_x = i16::from_ne_bytes(kid_x_word.to_ne_bytes());
    let delta = lion_x.wrapping_sub(kid_x);
    // ASM Lion.asm loc_3E06A: `tst.w d4`; negative values branch away,
    // otherwise the fallthrough executes `sf x_direction(a3)`.
    // The fallthrough path handles zero and positive deltas by clearing
    // x_direction, which is the Lion's left-facing orientation.
    if delta < 0 {
        Direction::Right
    } else {
        Direction::Left
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_track_kid_negative_delta_facing_loc_3e074(
    lion_x_word: u16,
    kid_x_word: u16,
) -> Direction {
    let lion_x = i16::from_ne_bytes(lion_x_word.to_ne_bytes());
    let kid_x = i16::from_ne_bytes(kid_x_word.to_ne_bytes());
    let delta = lion_x.wrapping_sub(kid_x);
    // ASM Lion.asm loc_3E074: the negative `d4 = lion_x - kid_x` branch
    // executes `st x_direction(a3)`, selecting the right-facing orientation.
    if delta < 0 {
        Direction::Right
    } else {
        Direction::Left
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullAirSprite {
    Rising,
    Falling,
}

#[cfg_attr(not(test), allow(dead_code))]
const MINI_SKULL_COLID_RIGHTWALL: u16 = 0x0004;
const MINI_SKULL_COLID_LEFTWALL: u16 = 0x0008;
const MINI_SKULL_COLID_FLOOR: u16 = 0x000C;
const MINI_SKULL_COLID_CEILING: u16 = 0x0010;
const MINI_SKULL_COLID_SLOPEUP: u16 = 0x0014;
const MINI_SKULL_COLID_HURT: u16 = 0x001C;
const MINI_SKULL_COLID_GROUND_KIDABOVE: u16 = 0x001C;
const MINI_SKULL_COLID_GROUND_KIDLEFT: u16 = 0x0020;
const MINI_SKULL_COLID_GROUND_KIDRIGHT: u16 = 0x0024;
const MINI_SKULL_COLID_GROUND_KIDBELOW: u16 = 0x0028;
const MINI_SKULL_COLID_GROUND_HURT: u16 = 0x002C;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullCollisionDispatch {
    AirborneLoop,
    WallBounce,
    FloorLanding,
    CeilingHit,
    HurtOrWeapon,
    SlopeOrOtherTerrain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullTerrainContactRoute {
    NotSlopeUp,
    SlopeUpNonPositiveXVelocity,
    SlopeUpPositiveXVelocity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct MiniHoppingSkullTerrainContact {
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) route: MiniHoppingSkullTerrainContactRoute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullGroundWaitSprite {
    SlopeUpRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct MiniHoppingSkullGroundWaitSetup {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) x_direction: u8,
    pub(crate) sprite: MiniHoppingSkullGroundWaitSprite,
    pub(crate) dbf_counter: u16,
    pub(crate) wait_frames: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullGroundWaitCollisionRoute {
    ContinueWait,
    DamageSkull,
    QueueScreamWorker,
    BerzerkerChargeKillsSkull,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullDamageSequence {
    SkullDestroyed,
    PlayHitAnimation {
        remaining_hits: u16,
        animation: &'static str,
        hibernate_until_finished: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum MiniHoppingSkullGroundWaitTail {
    ContinueWait {
        next_dbf_counter: u16,
    },
    SwitchToLeftGroundWaitSetup,
    BeginRightHopWindup {
        sprite: &'static str,
        dbf_counter: u16,
        wait_frames: u8,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct MiniHoppingSkullScreamWorkerAllocation {
    pub(crate) allocation_selector: u16,
    pub(crate) routine: &'static str,
    pub(crate) delay_frames: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct MiniHoppingSkullRelaunchTail {
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) next_loop: &'static str,
}

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
        // Armadillo.asm loc_3C756/loc_3C766: `enemy_level + 1` yields HP 1/2/3+,
        // then the walk-speed slot starts at `$8000` and gains `$10000` at each tier.
        EnemyFamily::Armadillo => (1, 0x0000_8000, 0x0001_0000),
        // Crab: HP = enemy_level + 2 (min 2). Base speed $8000, step $10000.
        EnemyFamily::Crab => (2, 0x0000_8000, 0x0001_0000),
        // Scorpion: HP = enemy_level + 3 (min 3). Fixed speed $18000.
        EnemyFamily::Scorpion => (3, 0x0001_8000, 0),
        // Driller: HP = enemy_level + 2 (min 2). Base speed $8000, step $8000.
        EnemyFamily::Driller => (2, 0x0000_8000, 0x0000_8000),
        // Tank: HP = enemy_level + 2 (min 2). Base speed $8000, step $8000.
        // ASM Tank.asm loc_3B36A/loc_3B372: HP 2 gets one `$8000` add,
        // HP 3 gets two, HP 4+ gets three.
        EnemyFamily::Tank | EnemyFamily::TankShooting => (2, 0x0000_8000, 0x0000_8000),
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
        // Diamond.asm loc_3AD16/loc_3AD4C: `addq.w #2,d7` yields HP 2/3+, and
        // the <=3 HP path walks at $10000 then $18000.
        EnemyFamily::Diamond => (2, 0x0001_0000, 0x0000_8000),
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

/// Tank bullet speed in FP16.16 (3 px/frame → $30000).
pub(crate) const TANK_BULLET_SPEED: i32 = 0x3_0000;
/// Tank bullet lifetime in frames (enough to cross a screen width).
pub(crate) const TANK_BULLET_LIFETIME: u16 = 120;
/// ASM Tank_Shooting.asm `stru_3B4D8`: one shot every 116 frames at HP 4.
pub(crate) const TANK_SHOOTING_FAST_CYCLE_FRAMES: u16 = 116;
/// ASM Tank_Shooting.asm `stru_3B4F6`: one shot every 232 frames at HP 2-3.
pub(crate) const TANK_SHOOTING_SLOW_CYCLE_FRAMES: u16 = 232;
/// ASM GfxObject_Animate + `stru_3B4D8`/`stru_3B4F6`: `loc_3C50E` fires one
/// frame after the turret enters `LnkTo_unk_C7F10`, which is 74 frames after init.
pub(crate) const TANK_SHOOTING_FIRST_SHOT_DELAY_FRAMES: u16 = 74;
/// ASM Tank_Shooting.asm loc_3B75A/loc_3B77E: spawn at ±15px from tank centre.
pub(crate) const TANK_BULLET_SPAWN_X_OFFSET_PX: i32 = 15;
/// ASM Tank_Shooting.asm loc_3B786/loc_3B78E: projectile Y = turret_y - (8 + 6) px.
pub(crate) const TANK_BULLET_SPAWN_Y_OFFSET_PX: i32 = 14;

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
    TankShooting,
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
    /// Tornado reuses this as the next `loc_359F2` pattern index from `unk_35A0C`.
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
            | Self::TankShooting
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
            Some(0x0B) => Self::TankShooting,
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
            Self::TankShooting => 0x0B,
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
    player_y: u32,
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
        player_y,
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
    player_y: u32,
    blank_top_rows: u16,
    killed: &std::collections::HashSet<(usize, usize)>,
    rng: &mut RomRandomState,
) {
    let view = enemy_layout_view(layout, camera_x_px, camera_y_px);
    // Most enemies despawn on spawn-point exit. Flyers keep their live object
    // around longer; the Level 1 Dragon continues drifting after its layout
    // anchor scrolls behind the camera.
    // ASM loc_3C46E (kid.asm:43965-44020): the shared deactivation routine
    // entered via loc_3C436 trampoline. Reads object_meta, dispatches
    // SpinningTwins to loc_3BC56, Driller/Tank to loc_3C494 (drive sound
    // stop with refcount decrement and optional PlaySound2), then falls
    // through to loc_3C4C0 which decrements Number_of_Enemy, updates
    // EnemyStatus_Table from $42(a5), and calls j_Delete_CurrentObject.
    //
    // loc_3C4A2 (kid.asm:43987): $19(a3)==0 path — tests $6A(a5);
    // if nonzero skips sound (bne.s loc_3C4C0), else falls to loc_3C4A8.
    // loc_3C4A8 (kid.asm:43991): subi.b #1,(a4,d7.w) refcount decrement;
    // plays j_PlaySound2 with byte_3C46C[d6] when counter reaches zero.
    // loc_3C4E6 (kid.asm:44013): positive $42(a5) slot writes $2168 into
    // EnemyStatus_Table[d0] as the respawn marker.
    //
    // Native handles deactivation by dropping the enemy from the active
    // list; sound cleanup is managed separately by the sound system.
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
        let family = EnemyFamily::from_layout_object(view.enemy_type_slots, object);
        if matches!(family, EnemyFamily::MiniHoppingSkull)
            && !mini_hopping_skull_spawn_gate_loc_3e9d0(object_x, object_y, player_x, player_y)
        {
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
                y_velocity: spawn_y_velocity(family, enemy_level),
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
                wait_timer: spawn_wait_timer(family, enemy_level, player_x, rng),
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
                // ASM Tornado.asm seeds a movement pattern before entering the
                // per-frame loop, so activate the first script entry here.
                reload_tornado_pattern_script(&mut enemy);
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
                } else if matches!(enemy.family, EnemyFamily::DragonWalking) {
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
                    // ASM Tank_Shooting.asm loc_3C50E/loc_3B730: only the
                    // shooting tank variant emits bullets, and only on the
                    // turret animation gate.
                    if matches!(enemy.family, EnemyFamily::TankShooting)
                        && let Some(ref mut projs) = projectiles
                    {
                        step_tank_shooting_fire(enemy, projs);
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

// ASM Tornado.asm `unk_35A0C`: 16-entry motion script, followed by a `$00000001`
// sentinel that wraps back to entry 0 in `loc_359F2`.
pub(crate) const TORNADO_PATTERN_SCRIPT: [TornadoPattern; 16] = [
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: 0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: 0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: -0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: -0x0000_1800,
    },
    TornadoPattern {
        target_x_velocity: -0x0001_8000,
        target_y_velocity: 0x0001_8000,
        x_acceleration: -0x0000_1800,
        y_acceleration: 0x0000_1800,
    },
];

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

pub(crate) fn reload_tornado_pattern_script(enemy: &mut ActiveEnemy) {
    // ASM Tornado.asm loc_359F2 / unk_35A0C: advance through the 16-entry
    // motion script and wrap when the following longword sentinel equals 1.
    let index = usize::from(enemy.dragon_script_cmd) % TORNADO_PATTERN_SCRIPT.len();
    let pattern = TORNADO_PATTERN_SCRIPT[index];
    enemy.dragon_script_cmd = ((index + 1) % TORNADO_PATTERN_SCRIPT.len()) as u8;
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
    _rng: &mut RomRandomState,
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

    // ASM Tornado.asm loc_359F2 / unk_3588C: reload the next script entry on
    // each sub-cycle and reset the per-level countdown.
    if enemy.wait_timer == 0 {
        let level = usize::from(enemy.enemy_level).min(2);
        enemy.wait_timer = TORNADO_PATTERN_FRAMES[level];
        reload_tornado_pattern_script(enemy);
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
            if !matches!(enemy.family, EnemyFamily::Fireball | EnemyFamily::Sphere) {
                // ASM Sphere.asm loc_36148 negates `x_vel(a3)` and immediately
                // resumes the loop. The sphere family never flips x_direction on
                // wall contact, so keep facing unchanged here.
                enemy.facing = opposite_direction(enemy.facing);
            }
        }

        // Floor bounce
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground && enemy.y_velocity > 0 {
            // Check for slope deflection first (ASM Sphere.asm loc_36012/loc_36032,
            // Fireball.asm loc_32D06/loc_32D46, Big_Hopping_Skull.asm loc_36706/loc_36720).
            // Slopes take precedence over floor bounce.
            if floor.cell.collision == terrain::CollisionKind::SlopeUp {
                let old_x = enemy.x_velocity;
                let old_y = enemy.y_velocity;
                if matches!(enemy.family, EnemyFamily::Sphere) {
                    // ASM Sphere.asm loc_3614E mirrors Enemy06_Sphere_SlopeUp:
                    // negate both source velocities, back the object out by one
                    // frame along that vector, then swap into x/y velocity.
                    enemy.x_pos = (enemy.x_pos as i64 - i64::from(old_x)).max(0) as u32;
                    enemy.y_pos = (enemy.y_pos as i64 - i64::from(old_y)).max(0) as u32;
                    enemy.y_velocity = -old_x;
                    enemy.x_velocity = -old_y;
                } else {
                    // ASM: swap velocities, negate both
                    enemy.y_velocity = -old_x;
                    enemy.x_velocity = -old_y;
                    enemy.facing = opposite_direction(enemy.facing);
                }
            } else if floor.cell.collision == terrain::CollisionKind::SlopeDown {
                let old_x = enemy.x_velocity;
                let old_y = enemy.y_velocity;
                if matches!(enemy.family, EnemyFamily::Sphere) {
                    // ASM Sphere.asm loc_36164 mirrors Enemy06_Sphere_SlopeDown:
                    // back out by the pre-collision motion, then swap the live
                    // x/y velocities without changing facing.
                    enemy.x_pos = (enemy.x_pos as i64 - i64::from(old_x)).max(0) as u32;
                    enemy.y_pos = (enemy.y_pos as i64 - i64::from(old_y)).max(0) as u32;
                    enemy.y_velocity = old_x;
                    enemy.x_velocity = old_y;
                } else {
                    // ASM: swap velocities without negation
                    enemy.y_velocity = old_x;
                    enemy.x_velocity = old_y;
                    enemy.facing = opposite_direction(enemy.facing);
                }
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
                // Spawn fire-breath projectile during descent.
                // ASM loc_34BAC: x_pos += $FFF6 (-10px, negated if facing right),
                // y_pos += $A (+10px), x_vel from dragon, y_vel = 3<<16.
                // No frame-based lifetime — dies off-screen ($19) or on terrain.
                if let Some(projs) = projectiles {
                    let x_offset: i32 = match enemy.facing {
                        Direction::Right => 10,
                        Direction::Left => -10,
                    };
                    projs.push(EnemyProjectile {
                        x_pos: (enemy.x_pos as i64 + ((i64::from(x_offset)) << 16)) as u32,
                        y_pos: (enemy.y_pos as i64 + (10 << 16)) as u32,
                        // ASM Dragon_Flying loc_348A0: copy the dragon's live
                        // x_vel into the projectile and force y_vel to +3px/frame.
                        x_velocity: enemy.x_velocity,
                        y_velocity: 3 << 16,
                        // ASM: no lifetime timer; projectile persists until off-screen
                        // or terrain contact. Use large lifetime + terrain_kill.
                        lifetime: 600,
                        gravity: 0,
                        friendly: false,
                        homing_accel: 0,
                        speed_cap: 0,
                        wall_bounce: false,
                        terrain_kill: true,
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
        EnemyArchetype::Collectible if matches!(enemy.family, EnemyFamily::Diamond) => {
            shared_enemy_should_remain_active_sub_3C3CE(
                enemy.x_pos,
                enemy.y_pos,
                level_bounds,
                active_enemy_count.saturating_add(3) as u16,
                camera_x_px,
                camera_y_px,
            )
        }
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
        EnemyArchetype::Pacer
            if matches!(
                enemy.family,
                EnemyFamily::Crab
                    | EnemyFamily::Driller
                    | EnemyFamily::FireDemon
                    | EnemyFamily::Robot
                    | EnemyFamily::SpinningTwins
                    | EnemyFamily::Tank
                    | EnemyFamily::TankShooting
            ) =>
        {
            shared_enemy_should_remain_active_sub_3C3CE(
                enemy.x_pos,
                enemy.y_pos,
                level_bounds,
                active_enemy_count.saturating_add(3) as u16,
                camera_x_px,
                camera_y_px,
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

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shared_enemy_should_remain_active_sub_3C3CE(
    x_pos: u32,
    y_pos: u32,
    level_bounds: physics::LevelBounds,
    number_objects: u16,
    camera_x_px: u32,
    camera_y_px: u32,
) -> bool {
    // ASM loc_3C044 / loc_3C0F0 (kid.asm:43505-43508): after the walker
    // hibernate frame, `bsr.w sub_3C3CE` checks whether the enemy should
    // remain active before dispatching to the next state via `jmp (a0)`.
    // ASM loc_3C194 (kid.asm:43653-43655): same hibernate + sub_3C3CE in the
    // leftward walking loop, but falls through to loc_3C19C (body collision
    // scan) instead of dispatching via `jmp (a0)`.
    // ASM sub_3C3CE (kid.asm:43901-43955): the shared Diamond/FireDemon/
    // walker retention path first rejects live positions outside
    // [-$20, level+$20] on both axes before applying the camera-relative
    // medium-load or heavy-load culling boxes.
    // ASM loc_3C436 (kid.asm:43936-43937): `bra.w loc_3C46E` — all four
    // level-bounds checks and both camera-relative culling boxes branch
    // here when the enemy falls outside the retention zone. loc_3C436 is
    // a trampoline to loc_3C46E (the full deactivation routine).
    let x_pos = (x_pos as i32) >> 16;
    let y_pos = (y_pos as i32) >> 16;
    let level_x_max = i32::from(level_bounds.width_pixels) + 0x20;
    let level_y_max = i32::from(level_bounds.height_pixels) + 0x20;
    if x_pos <= -0x20 || y_pos <= -0x20 || x_pos > level_x_max || y_pos > level_y_max {
        return false;
    }
    if number_objects <= 0x000A {
        return true;
    }

    let relative_x = x_pos - camera_x_px as i32;
    let relative_y = y_pos - camera_y_px as i32;
    let (left_margin, right_margin, top_margin, bottom_margin) = if number_objects <= 0x0014 {
        // ASM loc_3C43A: medium load uses [-$1A4,+$2E4]x[-$1A4,+$284].
        (0x1A4, 0x2E4, 0x1A4, 0x284)
    } else {
        // ASM sub_3C3CE fallthrough: heavy load tightens to
        // [-$104,+$244]x[-$104,+$1E4].
        (0x104, 0x244, 0x104, 0x1E4)
    };
    relative_x >= -left_margin
        && relative_x <= right_margin
        && relative_y >= -top_margin
        && relative_y <= bottom_margin
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

fn mini_hopping_skull_spawn_gate_loc_3e9d0(
    object_x_px: u32,
    object_y_px: u32,
    player_x_pos: u32,
    player_y_pos: u32,
) -> bool {
    // ASM Mini_Hopping_Skull.asm loc_3E9D0: hibernate until
    // `kid_x-layout_x` is in [-$50,$50] and `kid_y-layout_y` is in [0,$A0].
    let player_x_px = (player_x_pos as i32) >> 16;
    let player_y_px = (player_y_pos as i32) >> 16;
    let dx = player_x_px - object_x_px as i32;
    let dy = player_y_px - object_y_px as i32;
    (-0x50..=0x50).contains(&dx) && (0..=0xA0).contains(&dy)
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

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_air_sprite_after_loc_3ea88(
    is_animated: bool,
    y_velocity: i32,
) -> Option<MiniHoppingSkullAirSprite> {
    if is_animated {
        return None;
    }

    if y_velocity < 0 {
        Some(MiniHoppingSkullAirSprite::Rising)
    } else {
        // ASM Mini_Hopping_Skull.asm loc_3EAAC: non-negative `y_vel(a3)`
        // selects `LnkTo_unk_C8088` after the animated-state guard is clear.
        Some(MiniHoppingSkullAirSprite::Falling)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_collision_dispatch_loc_3eab2(
    collision_type: u16,
) -> MiniHoppingSkullCollisionDispatch {
    // ASM Mini_Hopping_Skull.asm loc_3EAB2 reads `collision_type(a3)` into d4
    // and dispatches by compare order, not by a dense table.
    if collision_type == 0 {
        MiniHoppingSkullCollisionDispatch::AirborneLoop
    } else if collision_type >= MINI_SKULL_COLID_HURT {
        MiniHoppingSkullCollisionDispatch::HurtOrWeapon
    } else if collision_type <= MINI_SKULL_COLID_LEFTWALL {
        MiniHoppingSkullCollisionDispatch::WallBounce
    } else if collision_type == MINI_SKULL_COLID_FLOOR {
        MiniHoppingSkullCollisionDispatch::FloorLanding
    } else if collision_type == MINI_SKULL_COLID_CEILING {
        MiniHoppingSkullCollisionDispatch::CeilingHit
    } else {
        MiniHoppingSkullCollisionDispatch::SlopeOrOtherTerrain
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_terrain_contact_loc_3ead8(
    collision_type: u16,
    x_velocity: i32,
) -> MiniHoppingSkullTerrainContact {
    let route = if collision_type != MINI_SKULL_COLID_SLOPEUP {
        MiniHoppingSkullTerrainContactRoute::NotSlopeUp
    } else if x_velocity <= 0 {
        MiniHoppingSkullTerrainContactRoute::SlopeUpNonPositiveXVelocity
    } else {
        MiniHoppingSkullTerrainContactRoute::SlopeUpPositiveXVelocity
    };

    // ASM Mini_Hopping_Skull.asm loc_3EAD8: terrain contact clears
    // `is_moved(a3)`, `is_animated(a3)`, and `has_level_collision(a3)`
    // before the slope-up-specific velocity gate.
    MiniHoppingSkullTerrainContact {
        is_moved: false,
        is_animated: false,
        has_level_collision: false,
        route,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_ground_wait_setup_loc_3eb00() -> MiniHoppingSkullGroundWaitSetup {
    // ASM Mini_Hopping_Skull.asm loc_3EB00: after positive-X slope-up contact,
    // clear both velocities, `st x_direction(a3)`, select `LnkTo_unk_C8030`,
    // and seed d1 with `$14`; the following DBF loop runs 21 hibernated frames.
    MiniHoppingSkullGroundWaitSetup {
        x_velocity: 0,
        y_velocity: 0,
        x_direction: 0xFF,
        sprite: MiniHoppingSkullGroundWaitSprite::SlopeUpRight,
        dbf_counter: 0x0014,
        wait_frames: 21,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_ground_wait_collision_loc_3eb0e(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EB0E: each grounded wait frame clears
    // `collision_type(a3)`, hibernates one frame, polls `sub_3F29A`, then
    // dispatches by ordered comparisons against Kid/hurt collision IDs.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            mini_hopping_skull_side_contact_loc_3eb56(berzerker_charging)
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_side_contact_loc_3eb56(
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EB56: side contact tests the global
    // `Berzerker_charging` byte; clear branches back to loc_3EB40's contact
    // damage worker, non-clear jumps to loc_3F262 to kill the skull.
    if berzerker_charging {
        MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
    } else {
        MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_right_hop_side_contact_loc_3ebda(
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EBDA: the right-hop windup side-contact
    // gate repeats loc_3EB56's Berzerker check; clear falls through to
    // the contact worker, set jumps to loc_3F262 to kill the skull.
    if berzerker_charging {
        MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
    } else {
        MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_damage_sequence_loc_3eb60(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3EB60: `subi.w #1,$44(a3)` mutates the
    // live hit counter before the zero test; non-zero replays `stru_3E9C2`
    // and hibernates until that animation finishes.
    let remaining_hits = hit_counter.wrapping_sub(1);
    if remaining_hits == 0 {
        MiniHoppingSkullDamageSequence::SkullDestroyed
    } else {
        MiniHoppingSkullDamageSequence::PlayHitAnimation {
            remaining_hits,
            animation: "stru_3E9C2",
            hibernate_until_finished: true,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_right_hop_damage_sequence_loc_3ebe4(
    hit_counter: u16,
) -> MiniHoppingSkullDamageSequence {
    // ASM Mini_Hopping_Skull.asm loc_3EBE4: the right-hop windup damage path
    // repeats loc_3EB60's word decrement of `$44(a3)`; zero branches to
    // loc_3F176, otherwise `stru_3E9C2` plays before the wait loop resumes.
    mini_hopping_skull_damage_sequence_loc_3eb60(hit_counter)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_ground_wait_tail_loc_3eb78(
    dbf_counter: u16,
    kid_x_word: u16,
    skull_x_word: u16,
) -> MiniHoppingSkullGroundWaitTail {
    // ASM Mini_Hopping_Skull.asm loc_3EB78: `dbf d1` consumes the inclusive
    // wait counter first. Only the `$FFFF` fall-through compares Kid X against
    // skull X as signed words; Kid to the right starts the short right-hop windup.
    let next_dbf_counter = dbf_counter.wrapping_sub(1);
    if next_dbf_counter != 0xFFFF {
        return MiniHoppingSkullGroundWaitTail::ContinueWait { next_dbf_counter };
    }

    if (kid_x_word as i16) <= (skull_x_word as i16) {
        MiniHoppingSkullGroundWaitTail::SwitchToLeftGroundWaitSetup
    } else {
        MiniHoppingSkullGroundWaitTail::BeginRightHopWindup {
            sprite: "LnkTo_unk_C8038",
            dbf_counter: 0x000A,
            wait_frames: 11,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EB92: each right-hop windup frame clears
    // `collision_type(a3)`, hibernates one frame, polls `sub_3F29A`, then
    // dispatches by the same ordered Kid/hurt collision comparisons as the
    // grounded wait path.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            mini_hopping_skull_right_hop_side_contact_loc_3ebda(berzerker_charging)
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
    collision_type: u16,
    berzerker_charging: bool,
) -> MiniHoppingSkullGroundWaitCollisionRoute {
    // ASM Mini_Hopping_Skull.asm loc_3EC48: the left-facing grounded wait loop
    // clears `collision_type(a3)`, hibernates one frame, polls `sub_3F29A`,
    // then dispatches by ordered Kid/hurt collision comparisons.
    match collision_type {
        0 => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
        MINI_SKULL_COLID_GROUND_KIDABOVE | MINI_SKULL_COLID_GROUND_HURT => {
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
        }
        MINI_SKULL_COLID_GROUND_KIDLEFT | MINI_SKULL_COLID_GROUND_KIDRIGHT => {
            mini_hopping_skull_side_contact_loc_3eb56(berzerker_charging)
        }
        MINI_SKULL_COLID_GROUND_KIDBELOW => {
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
        }
        _ => MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_scream_worker_allocation_loc_3eb40(
    route: MiniHoppingSkullGroundWaitCollisionRoute,
) -> Option<MiniHoppingSkullScreamWorkerAllocation> {
    if route != MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker {
        return None;
    }

    // ASM Mini_Hopping_Skull.asm loc_3EB40: exchange a0/a4, allocate with
    // selector `$A000`, store `sub_3F3F0` at object slot +4, then restore a0/a4.
    // `sub_3F3F0` hibernates for `$F` frames before playing the scream SFX.
    Some(MiniHoppingSkullScreamWorkerAllocation {
        allocation_selector: 0xA000,
        routine: "sub_3F3F0",
        delay_frames: 15,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_right_hop_scream_worker_allocation_loc_3ebc4(
    route: MiniHoppingSkullGroundWaitCollisionRoute,
) -> Option<MiniHoppingSkullScreamWorkerAllocation> {
    if route != MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker {
        return None;
    }

    // ASM Mini_Hopping_Skull.asm loc_3EBC4: the right-hop windup contact path
    // mirrors the grounded contact worker allocation: swap a0/a4, allocate with
    // selector `$A000`, install `sub_3F3F0` at slot +4, then restore a0/a4.
    Some(MiniHoppingSkullScreamWorkerAllocation {
        allocation_selector: 0xA000,
        routine: "sub_3F3F0",
        delay_frames: 15,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn mini_hopping_skull_right_hop_relaunch_tail_loc_3ec26() -> MiniHoppingSkullRelaunchTail
{
    // ASM Mini_Hopping_Skull.asm loc_3EC26: after loc_3EBFC has seeded the
    // rightward hop velocities, this shared tail sets `is_moved(a3)` and
    // `has_level_collision(a3)`, then branches back to the airborne loop.
    MiniHoppingSkullRelaunchTail {
        is_moved: true,
        has_level_collision: true,
        next_loop: "loc_3EA88",
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
        // ASM Mini_Hopping_Skull.asm loc_3EBFC/loc_3ED36/loc_3EE7C/loc_3EFB6:
        // enemy_level >= 1 (HP > 1) adds `$4000` to X speed and `$10000` extra
        // upward Y speed for either facing.
        if matches!(enemy.family, EnemyFamily::MiniHoppingSkull) && enemy.hit_points > 1 {
            jx += 0x4000;
            jy -= 0x1_0000;
        }
        enemy.x_velocity = horizontal_velocity(enemy.facing, jx);
        enemy.y_velocity = jy;
        enemy.on_ground = false;
    }

    // ASM Mini_Hopping_Skull.asm loc_3EA88: after the hibernated frame and
    // retention check, `add.l d3,y_vel(a3)` applies `$1200` gravity before the
    // collision-type dispatch.
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
        // ASM Mini_Hopping_Skull.asm loc_3F1AA flips x_direction and negates
        // the live X velocity; it does not reset to the base hop speed.
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
        enemy.x_velocity = -enemy.x_velocity;
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
    // ASM loc_3C1DE (kid.asm:43684-43686): no-collision X path in the leftward
    // walker loop — move.l d2,x_pos(a3) stores the predicted position when the
    // displacement doesn't cross a tile boundary (bcc.s) or d6==0 (no floor
    // status). The rightward mirror is loc_3C0C6. Native stores the predicted
    // x_pos unconditionally then corrects via probe_wall if a collision is found.
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
        // ASM loc_3C026 (rightward walkers) / loc_3C0D2 (leftward walkers):
        // clr.l x_vel(a3); clr.l y_vel(a3); st/sf x_direction(a3) — stop
        // and face the opposite direction after wall collision. The ASM also
        // bumps a byte counter at $5A(a5) by 4 and deactivates (loc_3C464)
        // if the counter wraps to zero (i.e. after 64 turnarounds). Native
        // handles deactivation via the retain/cull check each frame instead.
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

    // ASM loc_3C04E (kid.asm:43511-43542) / loc_3C0FA (kid.asm:43585-43609):
    // rightward/leftward walker terrain dispatch after sub_3C4F8 + sub_36972/
    // sub_36A58. The ASM computes displacement, aligns to 16px grid, and checks
    // d7 flags (bit 0=floor edge, bit 1=wall, bit 2=hurt) plus d6 (floor
    // status) to decide: continue walking (loc_3C17A/loc_3C266), snap position
    // (loc_3C0C6/loc_3C172), or turnaround (loc_3C0A6→loc_3C0D2 /
    // loc_3C152→loc_3C026). Native collapses these into probe_floor.
    //
    // ASM loc_3C19C (kid.asm:43657-43660): secondary body collision scan entry
    // in the leftward walking loop — `bsr.w sub_36B3C; tst.w d5; bne.w
    // loc_3C464`. After loc_3C194 hibernate + remain-active, sub_36B3C scans
    // the walker's body rectangle for overlapping collision cells; if d5
    // (scan result) is nonzero, the walker deactivates. Native covers this
    // via the probe_wall check above.
    //
    // ASM loc_3C0A6 (kid.asm:43545-43549): btst #0,d7 — floor edge bit set →
    // st $6E(a5), bra loc_3C0D2 turnaround. Native: !probe_floor().on_ground.
    //
    // ASM loc_3C0B4 (kid.asm:43552-43558): no floor edge bit, but d6 >= 0
    // (no solid floor) and bit 1 not set (no wall) → st $6E(a5), bra loc_3C0D2
    // turnaround. This catches the "cliff with no wall to cling to" case.
    // Native: also covered by the probe_floor edge check below.
    //
    // ASM loc_3C1E4 (kid.asm:43688-43696): Y-axis movement entry in the
    // leftward walker loop — loads y_pos, adds y_vel, computes pixel
    // displacement, branches to loc_3C220 when >= 0 (moving down). Native
    // doesn't apply Y velocity to walkers; probe_floor keeps them grounded.
    //
    // ASM loc_3C220 (kid.asm:43713-43725): downward Y movement floor check —
    // sub_36C6A scans for floor; if found, snaps y_pos and branches to
    // loc_3C026 turnaround. If no floor, stores predicted y_pos at loc_3C242
    // and gravity continues. Native: probe_floor edge detection reverses the
    // walker at cliff edges, preventing the fall state entirely.
    //
    // ASM loc_3C276 (kid.asm:43751-43754): falling walker loop top —
    // tst.w $3A(a5); beq.s loc_3C280; bsr.w sub_3BF72. Identical attachment
    // check to loc_3C18A/loc_3C03A but in the falling walker loop (entered
    // via loc_3C266 when a walker walks off an edge). When $3A(a5) is nonzero
    // (child present), sub_3BF72 syncs the child; when zero, skips to
    // loc_3C280. Native: walkers never enter the fall state because
    // probe_floor reverses them at cliff edges.
    //
    // ASM loc_3C280 (kid.asm:43756-43758): falling walker hibernate +
    // remain-active — jsr (j_Hibernate_Object_1Frame).w; bsr.w sub_3C3CE.
    // Same hibernate + remain-active as loc_3C194/loc_3C044 but in the
    // falling walker loop. Falls through to loc_3C288. Native: collapsed
    // into the per-frame step_pacer cycle; walkers don't fall.
    //
    // ASM loc_3C288 (kid.asm:43760-43763): falling walker body collision
    // scan — bsr.w sub_36BD6; tst.w d5; bne.w loc_3C464 deactivation. If
    // no collision, performs X movement with wall checks (loc_3C2CA snap),
    // Y movement with floor/ceiling checks and gravity (addi.l #$2000,
    // capped at $80000). Native: walkers are always grounded; the falling
    // loop's body scan + gravity are unreachable.
    //
    // ASM loc_3C2D0 (kid.asm:43791-43813): falling walker Y-axis movement
    // entry — move.l y_pos(a3),d0; add.l y_vel(a3),d1; sub.w d0,d1;
    // bge.s loc_3C30C. When displacement < 0 (moving upward), checks
    // ceiling via sub_36CB8; if ceiling found, snaps y_pos and clears
    // y_vel. When displacement >= 0, falls through to loc_3C30C (floor
    // check). Native: unreachable — walkers never enter fall state.
    //
    // ASM loc_3C30C (kid.asm:43816-43828): falling walker downward Y floor
    // check — sub_36C6A floor scan; if found, snaps y_pos and branches to
    // loc_3C0D2 (leftward turnaround). If no floor, falls through to
    // loc_3C32E. Native: unreachable — walkers reverse at cliff edges
    // before falling, so the in-air floor search never runs.
    //
    // ASM loc_3C32E (kid.asm:43831-43832): falling walker no-Y-collision
    // path — move.l d2,y_pos(a3) stores predicted position. Falls through
    // to loc_3C332 gravity (addi.l #$2000, capped $80000), then loops
    // back to loc_3C276. Native: unreachable — probe_floor cliff-edge
    // reversal prevents the falling walker loop entirely.
    //
    // ASM loc_3C34A (kid.asm:43841-43843): falling walker gravity cap
    // convergence — move.l d0,y_vel(a3); bra.w loc_3C276. Stores the
    // (potentially capped) y_vel and loops back to the falling walker
    // loop top. Native: unreachable — walkers never enter fall state.
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

/// Ninja detection cone: player must be 0x20–0x50 pixels above and in the
/// half-open X range [-0x10, +0x10) (Ninja.asm loc_3D706).
pub(crate) const NINJA_DETECT_Y_MIN: i32 = 0x20;
pub(crate) const NINJA_DETECT_Y_MAX: i32 = 0x50;
pub(crate) const NINJA_DETECT_X_RANGE: i32 = 0x10;

fn ninja_detects_player_while_stealthed(dx_px: i64, dy_px: i64) -> bool {
    // ASM loc_3D706: attack only when kid_y - ninja_y is in [-$50, -$20] and
    // kid_x - ninja_x is in [-$10, +$10). The right edge is exclusive because
    // `cmpi.w #$10,d4` falls through only on `d4 < $10`.
    dy_px >= NINJA_DETECT_Y_MIN as i64
        && dy_px <= NINJA_DETECT_Y_MAX as i64
        && dx_px >= -(NINJA_DETECT_X_RANGE as i64)
        && dx_px < NINJA_DETECT_X_RANGE as i64
}

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

            // Player must be 0x20–0x50 px above ninja and in [-0x10, +0x10).
            if ninja_detects_player_while_stealthed(dx, dy) {
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

/// Lion charge speed: $28000 for HP==8, $18000 otherwise (Lion.asm loc_3DFD8).
/// Also used as walk speed — ASM sets both $50(a5) and $5E(a5) from same value.
pub(crate) fn lion_charge_speed(hit_points: u8) -> i32 {
    if hit_points == 8 { 0x2_8000 } else { 0x1_8000 }
}

/// Lion charge timer: starts at $3C (60 frames), counts down while pacing.
/// Seeded by Lion.asm loc_3DFF0.
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
//   Attack: 27f (9+9+9 from `moveq #8,d0` + `dbf` in loc_3231C/loc_32348/loc_32360),
//   fires the horizontal glob shot after the first 9-frame wind-up
//   Submerge animation: 24f (stru_3274C, reverse of emerge)
/// Initial submerged wait before first emergence (Tar_Monster.asm unk_32264: 4 + 90).
pub(crate) const TAR_MONSTER_SUBMERGED_FRAMES: u16 = 94;
/// Emerge animation duration (6 frames × 4 ticks = 24 frames).
pub(crate) const TAR_MONSTER_EMERGE_FRAMES: u16 = 24;
/// Visible hold before attack sequence ($32 = 50 frames from bytecode).
pub(crate) const TAR_MONSTER_VISIBLE_FRAMES: u16 = 50;
/// Attack sequence duration. ASM loc_32310/loc_3231C/loc_32348/loc_32360:
/// three `moveq #8,d0` + `dbf` loops, so each phase lasts 9 frames.
pub(crate) const TAR_MONSTER_ATTACK_FRAMES: u16 = 27;
/// Zero-based elapsed frame index for the glob shot. ASM loc_3231C fires after
/// the first 9-frame `dbf` loop, which is elapsed frame 8 in the native countdown.
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
                // ASM loc_3231C/loc_32348: spawn the glob after the first `dbf` loop.
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
/// ASM loc_3D02A: breath is a stationary GfxObject, not a travelling projectile.
/// Animation stru_3CCC8 lasts 68 frames (8+10+10+10+10+10+10), after which
/// $18(a3) signals completion. No velocity is applied in the ROM.
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
            if enemy.shared_walker_state == SHARED_WALKER_STATE_PATROL {
                enemy.shared_walker_debug_branch = Some("airborne_land_patrol");
            } else {
                enemy.shared_walker_state = SHARED_WALKER_STATE_COOLDOWN;
                enemy.shared_walker_debug_branch = Some("airborne_land_cooldown");
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
            // ASM loc_3D02A: breath is a stationary animated hitbox at
            // the dragon's mouth (±28px). No velocity — the GfxObject
            // stays in place for the animation duration (stru_3CCC8, 68f).
            let x_offset = match enemy.facing {
                Direction::Right => DRAGON_WALKING_BREATH_OFFSET_PX,
                Direction::Left => -DRAGON_WALKING_BREATH_OFFSET_PX,
            };
            projectiles.push(EnemyProjectile {
                x_pos: (enemy.x_pos as i64 + ((i64::from(x_offset)) << 16)) as u32,
                y_pos: enemy.y_pos,
                x_velocity: 0,
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
// Random dormant delay: Drips.asm loc_3340E uses `(RNG >> 2)`,
// then XORs the low byte with Kid's integer X low byte.
// Forming animation: ~35f. Total interval ≈ 67f average.
// Drop child starts with zero vertical speed, then gains $2000/frame gravity
// until it hits the floor. ASM loc_33424 clears `$2A(a1)` before the fall and
// loc_3345C adds `#$2000` every frame.
// Surface-crawl spawning (max 5 crawlers, x_vel=$3000) not implemented —
// would require multi-object child spawning system.
/// Drop interval: forming animation (~35f) + average random delay (~32f) = ~67f.
pub(crate) const DRIPS_DROP_INTERVAL: u16 = 67;
/// ASM loc_33424 clears the drop child's y-velocity before the fall begins.
pub(crate) const DRIPS_DROP_INITIAL_Y_SPEED: i32 = 0;
/// Gravity accumulation: ASM loc_3345C `addi.l #$2000,$2A(a1)`.
pub(crate) const DRIPS_DROP_GRAVITY: i32 = 0x2000;
/// Native fallback lifetime for the falling drop. The ROM deletes on terrain
/// contact after the splash sequence; native keeps a generous timer in case the
/// projectile never reaches solid terrain.
pub(crate) const DRIPS_DROP_LIFETIME: u16 = 120;
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const DRIPS_COLID_CEILING: u16 = 0x0010;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsCeilingGateLoc333F6 {
    pub(crate) waiting_for_ceiling: bool,
    pub(crate) collision_type: u16,
    pub(crate) y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_ceiling_gate_loc_333_f6(
    collision_type: u16,
    y_velocity: i32,
) -> DripsCeilingGateLoc333F6 {
    // ASM Drips.asm loc_333F6: loop until `collision_type(a3)` is
    // `colid_ceiling`, then clear `collision_type(a3)` and `y_vel(a3)`.
    if collision_type == DRIPS_COLID_CEILING {
        DripsCeilingGateLoc333F6 {
            waiting_for_ceiling: false,
            collision_type: 0,
            y_velocity: 0,
        }
    } else {
        DripsCeilingGateLoc333F6 {
            waiting_for_ceiling: true,
            collision_type,
            y_velocity,
        }
    }
}

pub(crate) fn drips_wait_timer_with_rng_loc_3340e(rng: &mut RomRandomState, kid_x_pos: u32) -> u16 {
    let x_pos_bytes = kid_x_pos.to_be_bytes();
    let kid_x_word = u16::from_be_bytes([x_pos_bytes[0], x_pos_bytes[1]]);
    drips_wait_timer_loc_3340e(rng.next_byte(), kid_x_word)
}

pub(crate) fn drips_wait_timer_loc_3340e(random_byte: u8, kid_x_word: u16) -> u16 {
    // ASM Drips.asm loc_3340E: `andi.w #$FF,d7; asr.w #2,d7; move.w d7,d0`,
    // then `move.w x_pos(a4),d7; eor.b d7,d0`.
    let shifted = u16::from(random_byte) >> 2;
    let shifted_bytes = shifted.to_be_bytes();
    let kid_x_low_byte = kid_x_word.to_be_bytes()[1];
    u16::from_be_bytes([shifted_bytes[0], shifted_bytes[1] ^ kid_x_low_byte])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsSplashResetLoc33488 {
    pub(crate) drop_is_moved: bool,
    pub(crate) drop_status_flag: bool,
    pub(crate) drop_collision_type: u16,
    pub(crate) drop_y_pos: u32,
    pub(crate) surface_child_status_flag: bool,
    pub(crate) surface_child_x_pos: u32,
    pub(crate) surface_child_y_pos: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_splash_reset_loc_33488(
    drop_x_pos: u32,
    drop_y_pos: u32,
) -> DripsSplashResetLoc33488 {
    // ASM Drips.asm loc_33488: clear `$14(a1)` and `collision_type(a1)`,
    // add `$A` to the integer Y word, then copy the drop position into `a2`.
    let adjusted_drop_y_pos = drop_y_pos.wrapping_add(0x000A_0000);
    DripsSplashResetLoc33488 {
        drop_is_moved: false,
        drop_status_flag: false,
        drop_collision_type: 0,
        drop_y_pos: adjusted_drop_y_pos,
        surface_child_status_flag: true,
        surface_child_x_pos: drop_x_pos,
        surface_child_y_pos: adjusted_drop_y_pos,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_table_reset_loc_33506() -> (usize, u16) {
    // ASM Drips.asm loc_33506: when `off_33520` reaches its negative sentinel,
    // reload `a0` from the start of the table and clear `d7` before the surface
    // child sprite offset is written.
    (0, 0)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_surface_sprite_write_loc_3350e(selected_sprite_offset: u16) -> u16 {
    // ASM Drips.asm loc_3350E: write the selected `d7` word to
    // `addroffset_sprite(a2)` for the surface child before returning to the
    // dormant delay loop.
    selected_sprite_offset
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsIdleLoopLoc33516 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checks_in_range: bool,
    pub(crate) loops_forever: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_idle_loop_loc_33516() -> DripsIdleLoopLoc33516 {
    // ASM Drips.asm loc_33516: call one-frame hibernate, run
    // Object_CheckInRange, then branch back to the same idle loop.
    DripsIdleLoopLoc33516 {
        hibernates_one_frame: true,
        checks_in_range: true,
        loops_forever: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsVelocityDeltasLoc33526 {
    pub(crate) x_delta: i32,
    pub(crate) y_delta: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_velocity_deltas_loc_33526(
    child_byte_44: u8,
    child_byte_45: u8,
    x_velocity: i32,
    y_velocity: i32,
) -> DripsVelocityDeltasLoc33526 {
    // ASM Drips.asm loc_33526: clear d7/d6, only copy `x_vel(a3)` and
    // `y_vel(a3)` when `$44(a5)` is non-zero, then double both deltas when
    // `$45(a5)` is also non-zero.
    if child_byte_44 == 0 {
        DripsVelocityDeltasLoc33526 {
            x_delta: 0,
            y_delta: 0,
        }
    } else if child_byte_45 == 0 {
        DripsVelocityDeltasLoc33526 {
            x_delta: x_velocity,
            y_delta: y_velocity,
        }
    } else {
        DripsVelocityDeltasLoc33526 {
            x_delta: x_velocity.wrapping_mul(2),
            y_delta: y_velocity.wrapping_mul(2),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsVelocityApplyLoc33546 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_velocity_apply_loc_33546(
    x_velocity: i32,
    y_velocity: i32,
    x_delta: i32,
    y_delta: i32,
) -> DripsVelocityApplyLoc33546 {
    // ASM Drips.asm loc_33546: add d7/d6 back into the object's x/y
    // velocities as 32-bit values, preserving 68000 wrapping semantics.
    DripsVelocityApplyLoc33546 {
        x_velocity: x_velocity.wrapping_add(x_delta),
        y_velocity: y_velocity.wrapping_add(y_delta),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DripsCollisionDispatchLoc33550 {
    pub(crate) collision_type: u16,
    pub(crate) contact_flag: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn drips_collision_dispatch_loc_33550(
    collision_type: u16,
    berzerker_charging: bool,
) -> DripsCollisionDispatchLoc33550 {
    // ASM Drips.asm loc_33550: zero collision returns d5=0 unchanged; negative
    // collision clears the word and returns d5=1. Positive collision clears the
    // word before dispatching collision IDs from `$1C` upward.
    if collision_type == 0 {
        return DripsCollisionDispatchLoc33550 {
            collision_type,
            contact_flag: 0,
        };
    }
    if (collision_type & 0x8000) != 0 {
        return DripsCollisionDispatchLoc33550 {
            collision_type: 0,
            contact_flag: 1,
        };
    }

    let adjusted = collision_type.wrapping_sub(0x001C);
    if (adjusted & 0x8000) != 0 {
        return DripsCollisionDispatchLoc33550 {
            collision_type: 0,
            contact_flag: 0,
        };
    }

    let contact_flag = match collision_type {
        0x001C | 0x002C => 1,
        0x0020 | 0x0024 if berzerker_charging => 1,
        _ => 0,
    };
    DripsCollisionDispatchLoc33550 {
        collision_type: 0,
        contact_flag,
    }
}

/// Step a Drips enemy: stationary ceiling hanger that periodically drops projectiles.
/// ASM: Drips.asm — forming animation → drop at 10px/frame → splash → reform → repeat.
/// Native approximation uses a physics projectile for the drop.
pub(crate) fn step_drips(enemy: &mut ActiveEnemy, projectiles: &mut Vec<EnemyProjectile>) {
    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }
    // ASM loc_33424/loc_3345C: the drop starts from rest and accelerates
    // downward under `$2000` gravity until it reaches the floor.
    projectiles.push(EnemyProjectile {
        x_pos: enemy.x_pos,
        y_pos: enemy.y_pos,
        x_velocity: 0,
        y_velocity: DRIPS_DROP_INITIAL_Y_SPEED,
        lifetime: DRIPS_DROP_LIFETIME,
        gravity: DRIPS_DROP_GRAVITY,
        friendly: false,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
        terrain_kill: true,
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
            if enemy.on_ground {
                step_pacer(enemy, block_grid);
            } else {
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

fn tank_shooting_cycle_frames(hit_points: u8) -> u16 {
    if hit_points >= 4 {
        TANK_SHOOTING_FAST_CYCLE_FRAMES
    } else {
        TANK_SHOOTING_SLOW_CYCLE_FRAMES
    }
}

/// Tank_Shooting.asm loc_3C50E only spawns `loc_3B730` when the turret child
/// is on `LnkTo_unk_C7F10` with `animation_timer == 5`.
fn tank_shooting_spawn_worker_now_sub_3c4f8() -> bool {
    shared_enemy_robot_helper_sub_3C4F8(SharedEnemyRobotHelperInputSub3C4F8 {
        helper_kind: SharedEnemyRobotHelperKindSub3C4F8::RockTankShooting,
        parent_mapping: ROCK_TANK_SHOOTING_MAPPING_SUB_3C4F8,
        parent_state_word_32: 5,
        child_state_word_40: 0,
        child_counter_word_4e: 0,
        child_timer_word_54: 0,
        parent_x_pos: 0,
        walker_x_pos: 0,
        kid_x_pos: 0,
        parent_x_direction: 0,
        parent_flag_15: false,
        random_word: 0,
    })
    .spawned_loc_3_b730_worker
}

/// ASM Tank_Shooting.asm loc_3C50E/loc_3B730: fire only when the turret
/// animation reaches `LnkTo_unk_C7F10` with `animation_timer == 5`.
pub(crate) fn step_tank_shooting_fire(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
) {
    if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }
    enemy.wait_timer = tank_shooting_cycle_frames(enemy.hit_points).saturating_sub(1);

    if projectiles.len() >= MAX_PROJECTILES {
        return;
    }
    if !tank_shooting_spawn_worker_now_sub_3c4f8() {
        return;
    }

    let x_velocity = horizontal_velocity(enemy.facing, TANK_BULLET_SPEED);
    let x_offset = horizontal_velocity(enemy.facing, TANK_BULLET_SPAWN_X_OFFSET_PX << 16);
    projectiles.push(EnemyProjectile {
        // ASM loc_3B75A/loc_3B786: start from the turret child and offset to
        // the muzzle instead of the body centre.
        x_pos: enemy.x_pos.wrapping_add_signed(x_offset),
        y_pos: enemy
            .y_pos
            .wrapping_sub((TANK_BULLET_SPAWN_Y_OFFSET_PX as u32) << 16),
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
        // ASM Armadillo.asm loc_3C718, Dragon.asm loc_3CD70, and Tank.asm loc_3B2E4:
        // face TOWARD the
        // player at spawn.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn mini_hopping_skull_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::MiniHoppingSkull,
            x_pos: 32 << 16,
            y_pos: 32 << 16,
            x_velocity: 0,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: true,
            hit_points: 1,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            dragon_phase: DragonFlyingState::None,
            dragon_phase_frames: 0,
            dragon_script_variant: 0,
            dragon_script_cmd: 0,
            dragon_anchor_y: 0,
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
            boss_head_index: BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
            shared_walker_patrol_short_next: false,
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        }
    }

    fn mini_hopping_skull_test_grid() -> terrain::BlockGrid {
        let data = vec![0u8; 8 * 4 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        for col in 0..8 {
            grid.set_cell_at(col, 2, terrain::Cell::from_be_bytes(0x60, 0x00));
        }
        grid
    }

    fn tar_monster_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::TarMonster,
            hit_points: 2,
            facing: Direction::Right,
            tar_monster_state: TarMonsterState::Attacking,
            wait_timer: TAR_MONSTER_ATTACK_FRAMES,
            ..mini_hopping_skull_test_enemy()
        }
    }

    fn fireball_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Fireball,
            hit_points: 1,
            facing: Direction::Right,
            ..mini_hopping_skull_test_enemy()
        }
    }

    fn sphere_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Sphere,
            hit_points: 1,
            facing: Direction::Right,
            x_velocity: 0x0001_0000,
            ..mini_hopping_skull_test_enemy()
        }
    }

    fn ninja_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Ninja,
            hit_points: 2,
            facing: Direction::Left,
            ..mini_hopping_skull_test_enemy()
        }
    }

    #[test]
    fn diamond_hp_and_walk_speed_follow_loc_3ad16_and_loc_3ad4c() {
        // ASM Diamond.asm loc_3AD16 copies `enemy_level(a3)` to d7, adds 2 HP,
        // then loc_3AD4C applies the <=3 HP walk-speed tiers.
        let hp2 = spawn_hit_points(EnemyFamily::Diamond, 0);
        let hp3 = spawn_hit_points(EnemyFamily::Diamond, 1);

        assert_eq!(hp2, 2);
        assert_eq!(hp3, 3);

        assert_eq!(pacer_walk_speed(EnemyFamily::Diamond, hp2), 0x0001_0000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Diamond, hp3), 0x0001_8000);
    }

    #[test]
    fn tank_spawn_faces_toward_player_per_loc_3b2e4() {
        assert_eq!(
            spawn_facing(EnemyFamily::Tank, 100 << 16, 80 << 16),
            Direction::Left
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Tank, 100 << 16, 100 << 16),
            Direction::Right
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Tank, 100 << 16, 120 << 16),
            Direction::Right
        );
    }

    #[test]
    fn tank_hp_and_walk_speed_follow_loc_3b36a_and_loc_3b372() {
        // ASM Tank.asm loc_3B36A/loc_3B372: enemy_level is copied to d7, then
        // addq.w #2,d7 produces HP 2/3/4. The walk-speed longword receives one,
        // two, or three `$8000` increments at those tiers.
        let hp2 = spawn_hit_points(EnemyFamily::Tank, 0);
        let hp3 = spawn_hit_points(EnemyFamily::Tank, 1);
        let hp4 = spawn_hit_points(EnemyFamily::Tank, 2);

        assert_eq!(hp2, 2);
        assert_eq!(hp3, 3);
        assert_eq!(hp4, 4);

        assert_eq!(pacer_walk_speed(EnemyFamily::Tank, hp2), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Tank, hp3), 0x0001_0000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Tank, hp4), 0x0001_8000);
        assert_eq!(
            pacer_walk_speed(EnemyFamily::TankShooting, hp2),
            0x0000_8000
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::TankShooting, hp3),
            0x0001_0000
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::TankShooting, hp4),
            0x0001_8000
        );
    }

    #[test]
    fn tank_shooting_layout_object_maps_enemy_id_0b() {
        let object = kid_data::EnemyLayoutObject {
            object_type: 0x0B,
            flags: 0,
            hit_points_plus_one: 1,
            x: 0x0120,
            y: 0x0088,
        };

        assert_eq!(
            EnemyFamily::from_layout_object(&[0x000A, 0x000B], &object),
            EnemyFamily::TankShooting,
            "ASM kid.asm enemyloaddata table maps object id `$0B` to `Enemy0B_RockTank_shooting_Init`"
        );
    }

    #[test]
    fn tank_shooting_fire_uses_loc_3c50e_cadence_and_loc_3b730_offsets() {
        let mut enemy = ActiveEnemy {
            family: EnemyFamily::TankShooting,
            hit_points: 4,
            facing: Direction::Right,
            x_pos: 100 << 16,
            y_pos: 80 << 16,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();

        step_tank_shooting_fire(&mut enemy, &mut projectiles);

        assert_eq!(projectiles.len(), 1);
        assert_eq!(
            projectiles[0].x_pos,
            (115_u32) << 16,
            "ASM Tank_Shooting.asm loc_3B75A adds `$000F` when facing right"
        );
        assert_eq!(
            projectiles[0].y_pos,
            (66_u32) << 16,
            "ASM Tank_Shooting.asm loc_3B786/loc_3B78E fires from turret_y-14"
        );
        assert_eq!(projectiles[0].x_velocity, 0x0003_0000);
        assert_eq!(projectiles[0].y_velocity, 0);
        assert_eq!(
            enemy.wait_timer,
            TANK_SHOOTING_FAST_CYCLE_FRAMES - 1,
            "ASM `stru_3B4D8` loops every 116 frames, so the countdown reload is 115 after a shot"
        );
    }

    #[test]
    fn tank_shooting_spawn_wait_timer_matches_first_loc_3c50e_fire_gate() {
        let mut rng = RomRandomState::default();

        assert_eq!(
            spawn_wait_timer(EnemyFamily::Tank, 0, 0, &mut rng),
            0,
            "Tank.asm never reaches the shooter helper"
        );
        assert_eq!(
            spawn_wait_timer(EnemyFamily::TankShooting, 0, 0, &mut rng),
            TANK_SHOOTING_FIRST_SHOT_DELAY_FRAMES - 1,
            "ASM GfxObject_Animate reaches `LnkTo_unk_C7F10`/timer 5 after 74 frames, so the countdown seed is 73"
        );
    }

    #[test]
    fn armadillo_spawn_faces_toward_player_per_loc_3c718() {
        assert_eq!(
            spawn_facing(EnemyFamily::Armadillo, 100 << 16, 80 << 16),
            Direction::Left
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Armadillo, 100 << 16, 100 << 16),
            Direction::Right
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Armadillo, 100 << 16, 120 << 16),
            Direction::Right
        );
    }

    #[test]
    fn armadillo_hp_and_walk_speed_follow_loc_3c756_and_loc_3c766() {
        // ASM Armadillo.asm loc_3C756/loc_3C766: enemy_level is copied to d7,
        // then `addq.w #1,d7` produces HP 1/2/3. The walk-speed slot receives
        // `$8000` base plus one `$10000` increment per tier.
        let hp1 = spawn_hit_points(EnemyFamily::Armadillo, 0);
        let hp2 = spawn_hit_points(EnemyFamily::Armadillo, 1);
        let hp3 = spawn_hit_points(EnemyFamily::Armadillo, 2);

        assert_eq!(hp1, 1);
        assert_eq!(hp2, 2);
        assert_eq!(hp3, 3);

        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, hp1), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, hp2), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, hp3), 0x0002_8000);
    }

    #[test]
    fn armadillo_initial_patrol_timer_matches_loc_3c7f8() {
        let mut rng = RomRandomState::default();
        assert_eq!(
            spawn_wait_timer(EnemyFamily::Armadillo, 0, 0, &mut rng),
            0x012C
        );
        assert_eq!(
            spawn_wait_timer(EnemyFamily::Armadillo, 2, 0, &mut rng),
            0x012C
        );
    }

    #[test]
    fn emo_rock_spawn_wait_timer_matches_loc_361f4_rng_mix() {
        let mut rng = RomRandomState::default();

        assert_eq!(
            emo_rock_wait_timer_with_rng(&mut rng, 0x0123_0000),
            0x009F,
            "ASM Emo_Rock.asm loc_361F4 seeds d0 with `$3C`, XORs the first RNG byte with Kid X low byte `$23`, clears bit 7, then adds the result"
        );
    }

    #[test]
    fn emo_rock_spawn_wait_timer_stays_within_loc_361f4_masked_range() {
        let mut rng = RomRandomState::default();
        let delay = emo_rock_wait_timer_with_rng(&mut rng, 0);

        assert!(
            (0x003C..=0x00BB).contains(&delay),
            "ASM Emo_Rock.asm loc_361F4 clears bit 7 before adding the `$3C` base delay, so the hover seed must stay within 60..187 frames"
        );
    }

    #[test]
    fn drips_ceiling_gate_matches_loc_333f6() {
        assert_eq!(
            drips_ceiling_gate_loc_333_f6(0, -0x0001_0000),
            DripsCeilingGateLoc333F6 {
                waiting_for_ceiling: true,
                collision_type: 0,
                y_velocity: -0x0001_0000,
            },
            "ASM Drips.asm loc_333F6 branches back while `collision_type(a3)` is not `colid_ceiling`"
        );
        assert_eq!(
            drips_ceiling_gate_loc_333_f6(DRIPS_COLID_CEILING, -0x0001_0000),
            DripsCeilingGateLoc333F6 {
                waiting_for_ceiling: false,
                collision_type: 0,
                y_velocity: 0,
            },
            "ASM Drips.asm loc_333F6 clears `collision_type(a3)` and `y_vel(a3)` after ceiling contact"
        );
    }

    #[test]
    fn drips_wait_timer_matches_loc_3340e_rng_and_kid_x_mix() {
        assert_eq!(
            drips_wait_timer_loc_3340e(0xFC, 0x01C0),
            0x00FF,
            "ASM Drips.asm loc_3340E shifts the RNG byte right by two, then XORs only d0's low byte with Kid X"
        );

        let mut rng = RomRandomState::default();
        assert_eq!(
            spawn_wait_timer(EnemyFamily::Drips, 0, 0x0123_0000, &mut rng),
            0x0013,
            "ASM Drips.asm loc_3340E mixes the first ROM RNG byte `$C0` with Kid integer X low byte `$23`"
        );
    }

    #[test]
    fn drips_splash_reset_matches_loc_33488_child_writeback() {
        assert_eq!(
            drips_splash_reset_loc_33488(0x0123_4000, 0x00F6_8000),
            DripsSplashResetLoc33488 {
                drop_is_moved: false,
                drop_status_flag: false,
                drop_collision_type: 0,
                drop_y_pos: 0x0100_8000,
                surface_child_status_flag: true,
                surface_child_x_pos: 0x0123_4000,
                surface_child_y_pos: 0x0100_8000,
            },
            "ASM Drips.asm loc_33488 clears the falling child, adds 10px to its Y word, and copies the adjusted x/y to the surface child"
        );
    }

    #[test]
    fn drips_surface_table_reset_matches_loc_33506() {
        assert_eq!(
            drips_surface_table_reset_loc_33506(),
            (0, 0),
            "ASM Drips.asm loc_33506 reloads `a0` with `off_33520` and clears d7 before the surface child sprite write"
        );
    }

    #[test]
    fn drips_surface_sprite_write_matches_loc_3350e() {
        assert_eq!(
            drips_surface_sprite_write_loc_3350e(0x1234),
            0x1234,
            "ASM Drips.asm loc_3350E stores d7 unchanged into `addroffset_sprite(a2)`"
        );
        assert_eq!(
            drips_surface_sprite_write_loc_3350e(0),
            0,
            "ASM Drips.asm loc_3350E also writes the zero sprite word produced by the table reset path"
        );
    }

    #[test]
    fn drips_idle_loop_matches_loc_33516() {
        assert_eq!(
            drips_idle_loop_loc_33516(),
            DripsIdleLoopLoc33516 {
                hibernates_one_frame: true,
                checks_in_range: true,
                loops_forever: true,
            },
            "ASM Drips.asm loc_33516 hibernates for one frame, checks object range, then branches back to itself"
        );
    }

    #[test]
    fn drips_velocity_deltas_match_loc_33526_child_state_gates() {
        assert_eq!(
            drips_velocity_deltas_loc_33526(0, 0xFF, 0x0000_3000, -0x0000_2000),
            DripsVelocityDeltasLoc33526 {
                x_delta: 0,
                y_delta: 0,
            },
            "ASM Drips.asm loc_33526 leaves d7/d6 clear when `$44(a5)` is zero, even if `$45(a5)` is set"
        );
        assert_eq!(
            drips_velocity_deltas_loc_33526(1, 0, 0x0000_3000, -0x0000_2000),
            DripsVelocityDeltasLoc33526 {
                x_delta: 0x0000_3000,
                y_delta: -0x0000_2000,
            },
            "ASM Drips.asm loc_33526 copies x/y velocity once when `$44(a5)` is set and `$45(a5)` is clear"
        );
        assert_eq!(
            drips_velocity_deltas_loc_33526(1, 1, 0x0000_3000, -0x0000_2000),
            DripsVelocityDeltasLoc33526 {
                x_delta: 0x0000_6000,
                y_delta: -0x0000_4000,
            },
            "ASM Drips.asm loc_33526 doubles d7/d6 with `add.l` when both child state bytes are set"
        );
    }

    #[test]
    fn drips_velocity_apply_matches_loc_33546_wrapping_adds() {
        assert_eq!(
            drips_velocity_apply_loc_33546(0x0000_3000, -0x0000_2000, 0x0000_6000, -0x0000_4000,),
            DripsVelocityApplyLoc33546 {
                x_velocity: 0x0000_9000,
                y_velocity: -0x0000_6000,
            },
            "ASM Drips.asm loc_33546 adds d7/d6 directly into x_vel/y_vel"
        );
        assert_eq!(
            drips_velocity_apply_loc_33546(i32::MAX, i32::MIN, 1, -1),
            DripsVelocityApplyLoc33546 {
                x_velocity: i32::MIN,
                y_velocity: i32::MAX,
            },
            "ASM Drips.asm loc_33546 uses 32-bit `add.l`, so overflow wraps instead of saturating"
        );
    }

    #[test]
    fn drips_collision_dispatch_matches_loc_33550_entry_paths() {
        assert_eq!(
            drips_collision_dispatch_loc_33550(0, true),
            DripsCollisionDispatchLoc33550 {
                collision_type: 0,
                contact_flag: 0,
            },
            "ASM Drips.asm loc_33550 returns d5=0 for zero collision without needing the dispatch table"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0xFFFF, false),
            DripsCollisionDispatchLoc33550 {
                collision_type: 0,
                contact_flag: 1,
            },
            "ASM Drips.asm loc_33550 treats a negative collision word as contact and clears `collision_type(a3)`"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x001A, false),
            DripsCollisionDispatchLoc33550 {
                collision_type: 0,
                contact_flag: 0,
            },
            "ASM Drips.asm loc_33550 clears positive collision words before rejecting values below `$1C`"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x001C, false).contact_flag,
            1,
            "ASM Drips.asm loc_33550 maps `$1C` to d5=1 contact"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x0020, false).contact_flag,
            0,
            "ASM Drips.asm loc_33550 maps `$20` through the Berzerker-charging gate"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x0024, true).contact_flag,
            1,
            "ASM Drips.asm loc_33550 accepts `$24` when `Berzerker_charging` is set"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x0028, true).contact_flag,
            0,
            "ASM Drips.asm loc_33550 dispatches `$28` to the no-contact return"
        );
        assert_eq!(
            drips_collision_dispatch_loc_33550(0x002C, false).contact_flag,
            1,
            "ASM Drips.asm loc_33550 maps `$2C` to d5=1 contact"
        );
    }

    #[test]
    fn lion_init_visible_state_matches_loc_3df92() {
        assert_eq!(
            lion_init_visible_state_loc_3df92(2, 0x0200, 0x0210),
            LionInitVisibleState {
                status_flag_13: true,
                object_meta: 0x0092,
                palette_line: 2,
                current_hp: 8,
                facing: Direction::Right,
            },
            "ASM Lion.asm loc_3DF92 should set `$13`, object_meta `$12|$80`, palette line 2, HP enemy_level+6, and keep the prior `st x_direction` result when Kid is right"
        );
        assert_eq!(
            lion_init_visible_state_loc_3df92(0, 0x0200, 0x01F0).facing,
            Direction::Left,
            "ASM Lion.asm loc_3DF92 should preserve zero x_direction when the signed compare finds Kid left of Lion"
        );
        assert_eq!(
            lion_init_visible_state_loc_3df92(0, 0x7FFF, 0x8000).facing,
            Direction::Left,
            "ASM Lion.asm loc_3DF92 inherits the signed word `blt` spawn-facing compare"
        );
    }

    #[test]
    fn lion_init_motion_profile_matches_loc_3dfd8_binary_hp_branch() {
        assert_eq!(
            lion_init_motion_profile_loc_3dfd8(7),
            LionInitMotionProfile {
                walk_speed: 0x0001_8000,
                charge_speed: 0x0001_8000,
                animation_script: "stru_3DEDA",
            },
            "ASM Lion.asm loc_3DFD8 should use `$18000` and `stru_3DEDA` when current_hp is not exactly 8"
        );
        assert_eq!(
            lion_init_motion_profile_loc_3dfd8(8),
            LionInitMotionProfile {
                walk_speed: 0x0002_8000,
                charge_speed: 0x0002_8000,
                animation_script: "stru_3DEF4",
            },
            "ASM Lion.asm skips loc_3DFD8 only for current_hp exactly 8"
        );
        assert_eq!(
            lion_charge_speed(9),
            0x0001_8000,
            "ASM Lion.asm loc_3DFD8 is selected by `bne`, so HP above 8 is not a fast tier"
        );
    }

    #[test]
    fn lion_init_runtime_fields_match_loc_3dff0_common_tail() {
        let fast_profile = lion_init_motion_profile_loc_3dfd8(8);
        assert_eq!(
            lion_init_runtime_fields_loc_3dff0(fast_profile, 0x2A),
            LionInitRuntimeFields {
                ufo_beam_interval: 0x002F,
                ufo_beam_segments: 0x0014,
                tracking_callback: "LionTrackingCallback",
                animation_script: "stru_3DEF4",
                shared_walker_state: 0x000A,
                charge_timer: 0x003C,
                collision_routine: "LionCollisionRoutine",
                random_phase_byte: 0x2A,
            },
            "ASM Lion.asm loc_3DFF0 should seed the common runtime fields after the HP-specific animation script is selected"
        );

        let regular_profile = lion_init_motion_profile_loc_3dfd8(7);
        assert_eq!(
            lion_init_runtime_fields_loc_3dff0(regular_profile, 0),
            LionInitRuntimeFields {
                ufo_beam_interval: 0x002F,
                ufo_beam_segments: 0x0014,
                tracking_callback: "LionTrackingCallback",
                animation_script: "stru_3DEDA",
                shared_walker_state: 0x000A,
                charge_timer: LION_CHARGE_TIMER_INIT,
                collision_routine: "LionCollisionRoutine",
                random_phase_byte: 0x4B,
            },
            "ASM Lion.asm loc_3DFF0 remaps RNG byte 0 to `$4B` after storing the selected animation"
        );
    }

    #[test]
    fn lion_init_facing_dispatch_matches_loc_3e030() {
        assert_eq!(
            lion_init_facing_dispatch_loc_3e030(Direction::Left),
            LionFacingDispatch::LeftPacing,
            "ASM Lion.asm loc_3E030 should send clear `x_direction(a3)` to loc_3E546"
        );
        assert_eq!(
            lion_init_facing_dispatch_loc_3e030(Direction::Right),
            LionFacingDispatch::RightPacing,
            "ASM Lion.asm loc_3E030 should send set `x_direction(a3)` to loc_3E392"
        );
    }

    #[test]
    fn lion_tracking_gate_matches_loc_3e054_latched_outer_band() {
        assert_eq!(
            lion_tracking_gate_loc_3e054(0x0100 + 0x0041, 0x0100, true),
            LionTrackingGate::Hold,
            "ASM Lion.asm loc_3E054 should keep current facing while `$4E(a5)` is set and d4 is below +$A0"
        );
        assert_eq!(
            lion_tracking_gate_loc_3e054(0x0100 - 0x0040, 0x0100, true),
            LionTrackingGate::Hold,
            "ASM Lion.asm loc_3E054 should keep current facing while `$4E(a5)` is set and d4 is above -$A0"
        );
    }

    #[test]
    fn lion_tracking_gate_matches_loc_3e054_clear_latch_bypass() {
        assert_eq!(
            lion_tracking_gate_loc_3e054(0x0100 + 0x0041, 0x0100, false),
            LionTrackingGate::TrackKid,
            "ASM Lion.asm loc_3E054 should fall through to the facing update immediately when `$4E(a5)` is clear"
        );
    }

    #[test]
    fn lion_clear_tracking_latch_matches_loc_3e066() {
        assert_eq!(
            lion_clear_tracking_latch_loc_3e066(0x0200, 0x0300),
            LionLatchClearResult {
                range_latch_set: false,
                facing: Direction::Right,
            },
            "ASM Lion.asm loc_3E066 should clear `$4E(a5)` and keep negative d4 facing toward Kid on the right"
        );
        assert_eq!(
            lion_clear_tracking_latch_loc_3e066(0x0300, 0x0200),
            LionLatchClearResult {
                range_latch_set: false,
                facing: Direction::Left,
            },
            "ASM Lion.asm loc_3E066 should clear `$4E(a5)` and keep non-negative d4 facing toward Kid on the left"
        );
    }

    #[test]
    fn lion_track_kid_facing_matches_loc_3e06a_nonnegative_delta() {
        assert_eq!(
            lion_track_kid_nonnegative_delta_facing_loc_3e06a(0x0300, 0x0200),
            Direction::Left,
            "ASM Lion.asm loc_3E06A should clear x_direction when signed d4 = lion_x - kid_x is positive"
        );
        assert_eq!(
            lion_track_kid_nonnegative_delta_facing_loc_3e06a(0x0200, 0x0200),
            Direction::Left,
            "ASM Lion.asm loc_3E06A should also clear x_direction when signed d4 is zero"
        );
    }

    #[test]
    fn lion_track_kid_facing_matches_loc_3e074_negative_delta() {
        assert_eq!(
            lion_track_kid_negative_delta_facing_loc_3e074(0x0200, 0x0300),
            Direction::Right,
            "ASM Lion.asm loc_3E074 should store x_direction when signed d4 = lion_x - kid_x is negative"
        );
        assert_eq!(
            lion_track_kid_negative_delta_facing_loc_3e074(0x7FFF, 0x8000),
            Direction::Right,
            "ASM Lion.asm loc_3E074 inherits 68k signed-word comparison semantics across the sign boundary"
        );
    }

    #[test]
    fn tank_child_layout_matches_init_offsets_from_loc_3b372() {
        // ASM Tank.asm loc_3B372 copies the parent x/y into the child sprite,
        // then `subi.w #8,$1E(a1)` lifts the child 8px above the body.
        let layout = shared_enemy_attachment_sub_3BF72(SharedEnemyAttachmentInputSub3BF72 {
            family: EnemyFamily::Tank,
            parent_x_pos: 0x0120,
            parent_y_pos: 0x0088,
            parent_x_direction: 0xFF,
            child_y_offset: 8,
            robot_parent_state: 0,
            robot_child_mapping: 0,
            child_x_direction: 0,
        });

        assert_eq!(
            layout.primary,
            SharedEnemyAttachmentChildSub3BF72 {
                x_pos: 0x0120,
                y_pos: 0x0080,
                x_velocity: 0,
                x_direction: Some(0xFF),
            }
        );
        assert_eq!(layout.secondary, None);
    }

    #[test]
    fn mini_hopping_skull_spawn_gate_matches_loc_3e9d0_edges() {
        // ASM Mini_Hopping_Skull.asm loc_3E9D0 keeps hibernating outside
        // Kid-relative X [-$50,$50] and Y [0,$A0] from the layout point.
        assert!(mini_hopping_skull_spawn_gate_loc_3e9d0(
            200,
            100,
            120 << 16,
            100 << 16
        ));
        assert!(mini_hopping_skull_spawn_gate_loc_3e9d0(
            200,
            100,
            280 << 16,
            260 << 16
        ));
        assert!(!mini_hopping_skull_spawn_gate_loc_3e9d0(
            200,
            100,
            119 << 16,
            100 << 16
        ));
        assert!(!mini_hopping_skull_spawn_gate_loc_3e9d0(
            200,
            100,
            200 << 16,
            99 << 16
        ));
        assert!(!mini_hopping_skull_spawn_gate_loc_3e9d0(
            200,
            100,
            200 << 16,
            261 << 16
        ));
    }

    #[test]
    fn mini_hopping_skull_initial_y_velocity_matches_loc_3ea48() {
        assert_eq!(
            spawn_y_velocity(EnemyFamily::MiniHoppingSkull, 0),
            0x0002_8000,
            "ASM Mini_Hopping_Skull.asm loc_3EA48 seeds base y_vel with `$28000`"
        );
        assert_eq!(
            spawn_y_velocity(EnemyFamily::MiniHoppingSkull, 1),
            0x0002_9000,
            "ASM Mini_Hopping_Skull.asm loc_3EA48 adds `$1000` when enemy_level is non-zero"
        );
        assert_eq!(
            spawn_y_velocity(EnemyFamily::MiniHoppingSkull, 2),
            0x0002_9000,
            "ASM Mini_Hopping_Skull.asm loc_3EA48 tests non-zero enemy_level, not a tiered table"
        );
    }

    #[test]
    fn mini_hopping_skull_first_air_tick_applies_loc_3ea88_gravity() {
        let data = vec![0u8; 8 * 4 * 2];
        let grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        let mut enemy = ActiveEnemy {
            on_ground: false,
            y_velocity: spawn_y_velocity(EnemyFamily::MiniHoppingSkull, 0),
            ..mini_hopping_skull_test_enemy()
        };

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.y_velocity, 0x0002_9200,
            "ASM Mini_Hopping_Skull.asm loc_3EA88 adds `$1200` gravity to the `$28000` loc_3EA48 seed"
        );
        assert_eq!(
            enemy.y_pos,
            (32 << 16) + 0x0002_9200,
            "ASM Mini_Hopping_Skull.asm loc_3EA88 applies the gravity-adjusted velocity during the airborne frame"
        );
    }

    #[test]
    fn mini_hopping_skull_air_sprite_matches_loc_3eaac_falling_branch() {
        assert_eq!(
            mini_hopping_skull_air_sprite_after_loc_3ea88(false, -1),
            Some(MiniHoppingSkullAirSprite::Rising),
            "ASM Mini_Hopping_Skull.asm branches around loc_3EAAC while `y_vel(a3)` is negative"
        );
        assert_eq!(
            mini_hopping_skull_air_sprite_after_loc_3ea88(false, 0),
            Some(MiniHoppingSkullAirSprite::Falling),
            "ASM Mini_Hopping_Skull.asm loc_3EAAC selects the falling sprite once `y_vel(a3)` is non-negative"
        );
        assert_eq!(
            mini_hopping_skull_air_sprite_after_loc_3ea88(true, 0),
            None,
            "ASM Mini_Hopping_Skull.asm checks `is_animated(a3)` before the loc_3EAAC sprite branch"
        );
    }

    #[test]
    fn mini_hopping_skull_collision_dispatch_matches_loc_3eab2_compare_order() {
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(0),
            MiniHoppingSkullCollisionDispatch::AirborneLoop,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 branches back to loc_3EA88 when `collision_type(a3)` is zero"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_RIGHTWALL),
            MiniHoppingSkullCollisionDispatch::WallBounce,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 sends right-wall collision to loc_3F1AA"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_LEFTWALL),
            MiniHoppingSkullCollisionDispatch::WallBounce,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 sends left-wall collision to loc_3F1AA"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_FLOOR),
            MiniHoppingSkullCollisionDispatch::FloorLanding,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 sends floor collision to loc_3EFEC"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_CEILING),
            MiniHoppingSkullCollisionDispatch::CeilingHit,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 sends ceiling collision to loc_3F1A2"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_HURT),
            MiniHoppingSkullCollisionDispatch::HurtOrWeapon,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 sends hurt-or-above collision to loc_3F1B8 before terrain checks"
        );
        assert_eq!(
            mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_SLOPEUP),
            MiniHoppingSkullCollisionDispatch::SlopeOrOtherTerrain,
            "ASM Mini_Hopping_Skull.asm loc_3EAB2 falls through to loc_3EAD8 for slope-up collision"
        );
    }

    #[test]
    fn mini_hopping_skull_terrain_contact_matches_loc_3ead8_flag_clear_and_slope_gate() {
        assert_eq!(
            mini_hopping_skull_terrain_contact_loc_3ead8(MINI_SKULL_COLID_CEILING, 0),
            MiniHoppingSkullTerrainContact {
                is_moved: false,
                is_animated: false,
                has_level_collision: false,
                route: MiniHoppingSkullTerrainContactRoute::NotSlopeUp,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EAD8 clears live movement flags before branching away from non-slope-up terrain"
        );
        assert_eq!(
            mini_hopping_skull_terrain_contact_loc_3ead8(MINI_SKULL_COLID_SLOPEUP, 0),
            MiniHoppingSkullTerrainContact {
                is_moved: false,
                is_animated: false,
                has_level_collision: false,
                route: MiniHoppingSkullTerrainContactRoute::SlopeUpNonPositiveXVelocity,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EAD8/sub_3F366 sends zero or negative `x_vel(a3)` to loc_3EC32"
        );
        assert_eq!(
            mini_hopping_skull_terrain_contact_loc_3ead8(MINI_SKULL_COLID_SLOPEUP, 1),
            MiniHoppingSkullTerrainContact {
                is_moved: false,
                is_animated: false,
                has_level_collision: false,
                route: MiniHoppingSkullTerrainContactRoute::SlopeUpPositiveXVelocity,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EAD8 clears both velocity components after slope-up contact with positive `x_vel(a3)`"
        );
    }

    #[test]
    fn mini_hopping_skull_ground_wait_setup_matches_loc_3eb00() {
        let setup = mini_hopping_skull_ground_wait_setup_loc_3eb00();

        assert_eq!(
            setup,
            MiniHoppingSkullGroundWaitSetup {
                x_velocity: 0,
                y_velocity: 0,
                x_direction: 0xFF,
                sprite: MiniHoppingSkullGroundWaitSprite::SlopeUpRight,
                dbf_counter: 0x0014,
                wait_frames: 21,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EB00 should face right, choose the slope-up wait sprite, and seed the `$14` DBF counter"
        );
    }

    #[test]
    fn mini_hopping_skull_ground_wait_collision_matches_loc_3eb0e_compare_order() {
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(0, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E should continue the DBF wait when `collision_type(a3)` stays zero"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E sends `colid_kidabove` to the HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(MINI_SKULL_COLID_GROUND_HURT, false),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E sends `colid_hurt` to the same HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E treats non-charging side contact as contact damage"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E routes charging Berzerker side contact to loc_3F262"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E sends `colid_kidbelow` to contact damage before considering Berzerker charge"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_collision_loc_3eb0e(MINI_SKULL_COLID_SLOPEUP, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EB0E ignores unrecognised collision IDs during the grounded wait loop"
        );
    }

    #[test]
    fn mini_hopping_skull_side_contact_matches_loc_3eb56_berzerker_gate() {
        assert_eq!(
            mini_hopping_skull_side_contact_loc_3eb56(false),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EB56 should branch to loc_3EB40 when `Berzerker_charging` is clear"
        );
        assert_eq!(
            mini_hopping_skull_side_contact_loc_3eb56(true),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB56 should jump to loc_3F262 while `Berzerker_charging` is set"
        );
    }

    #[test]
    fn mini_hopping_skull_right_hop_side_contact_matches_loc_3ebda_berzerker_gate() {
        assert_eq!(
            mini_hopping_skull_right_hop_side_contact_loc_3ebda(false),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EBDA should fall through to the contact worker when `Berzerker_charging` is clear"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_side_contact_loc_3ebda(true),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EBDA should jump to loc_3F262 while `Berzerker_charging` is set"
        );
    }

    #[test]
    fn mini_hopping_skull_damage_sequence_matches_loc_3eb60_counter_update() {
        assert_eq!(
            mini_hopping_skull_damage_sequence_loc_3eb60(1),
            MiniHoppingSkullDamageSequence::SkullDestroyed,
            "ASM Mini_Hopping_Skull.asm loc_3EB60 should branch to the skull death routine when `$44(a3)` becomes zero after decrement"
        );
        assert_eq!(
            mini_hopping_skull_damage_sequence_loc_3eb60(2),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 1,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EB60 should replay `stru_3E9C2` and hibernate until it finishes when hits remain"
        );
        assert_eq!(
            mini_hopping_skull_damage_sequence_loc_3eb60(0),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 0xFFFF,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EB60 uses word wrapping for the decrement before the zero branch"
        );
    }

    #[test]
    fn mini_hopping_skull_right_hop_damage_sequence_matches_loc_3ebe4_counter_update() {
        assert_eq!(
            mini_hopping_skull_right_hop_damage_sequence_loc_3ebe4(1),
            MiniHoppingSkullDamageSequence::SkullDestroyed,
            "ASM Mini_Hopping_Skull.asm loc_3EBE4 should branch to loc_3F176 when `$44(a3)` reaches zero"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_damage_sequence_loc_3ebe4(2),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 1,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EBE4 should replay `stru_3E9C2` and hibernate until it finishes when hits remain"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_damage_sequence_loc_3ebe4(0),
            MiniHoppingSkullDamageSequence::PlayHitAnimation {
                remaining_hits: 0xFFFF,
                animation: "stru_3E9C2",
                hibernate_until_finished: true,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EBE4 uses word wrapping for the hit-counter decrement"
        );
    }

    #[test]
    fn mini_hopping_skull_ground_wait_tail_matches_loc_3eb78_dbf_and_x_compare() {
        assert_eq!(
            mini_hopping_skull_ground_wait_tail_loc_3eb78(0x0014, 40, 32),
            MiniHoppingSkullGroundWaitTail::ContinueWait {
                next_dbf_counter: 0x0013,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EB78 should loop while `dbf d1` leaves a non-$FFFF counter"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_tail_loc_3eb78(0x0000, 32, 32),
            MiniHoppingSkullGroundWaitTail::SwitchToLeftGroundWaitSetup,
            "ASM Mini_Hopping_Skull.asm loc_3EB78 should switch grounded setup when Kid X is equal to skull X"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_tail_loc_3eb78(0x0000, 31, 32),
            MiniHoppingSkullGroundWaitTail::SwitchToLeftGroundWaitSetup,
            "ASM Mini_Hopping_Skull.asm loc_3EB78 should use the signed `ble` path when Kid X is left of the skull"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_tail_loc_3eb78(0x0000, 33, 32),
            MiniHoppingSkullGroundWaitTail::BeginRightHopWindup {
                sprite: "LnkTo_unk_C8038",
                dbf_counter: 0x000A,
                wait_frames: 11,
            },
            "ASM Mini_Hopping_Skull.asm loc_3EB78 should choose the right-hop windup sprite and `$A` DBF seed when Kid X is greater"
        );
        assert_eq!(
            mini_hopping_skull_ground_wait_tail_loc_3eb78(0x0000, 0x8000, 0x7FFF),
            MiniHoppingSkullGroundWaitTail::SwitchToLeftGroundWaitSetup,
            "ASM Mini_Hopping_Skull.asm loc_3EB78 uses signed word comparison, so `$8000` is less than `$7FFF`"
        );
    }

    #[test]
    fn mini_hopping_skull_right_hop_windup_collision_matches_loc_3eb92_compare_order() {
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(0, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 should continue the DBF windup when `collision_type(a3)` stays zero"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 sends `colid_kidabove` to the HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_GROUND_HURT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 sends `colid_hurt` to the same HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 treats non-charging side contact as contact damage"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 routes charging Berzerker side contact to loc_3F262"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 sends `colid_kidbelow` to contact damage before considering Berzerker charge"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_windup_collision_loc_3eb92(
                MINI_SKULL_COLID_SLOPEUP,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EB92 ignores unrecognised collision IDs during the right-hop windup loop"
        );
    }

    #[test]
    fn mini_hopping_skull_left_ground_wait_collision_matches_loc_3ec48_compare_order() {
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(0, false),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 should continue the DBF wait when `collision_type(a3)` stays zero"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_GROUND_KIDABOVE,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 sends `colid_kidabove` to loc_3EC9A's HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_GROUND_HURT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 sends `colid_hurt` to the same HP decrement branch"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_GROUND_KIDLEFT,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 treats non-charging side contact as contact damage"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_GROUND_KIDRIGHT,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 routes charging Berzerker side contact to loc_3F262"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_GROUND_KIDBELOW,
                true
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 sends `colid_kidbelow` to contact damage before considering Berzerker charge"
        );
        assert_eq!(
            mini_hopping_skull_left_ground_wait_collision_loc_3ec48(
                MINI_SKULL_COLID_SLOPEUP,
                false
            ),
            MiniHoppingSkullGroundWaitCollisionRoute::ContinueWait,
            "ASM Mini_Hopping_Skull.asm loc_3EC48 ignores unrecognised collision IDs during the left grounded wait loop"
        );
    }

    #[test]
    fn mini_hopping_skull_scream_worker_allocation_matches_loc_3eb40() {
        assert_eq!(
            mini_hopping_skull_scream_worker_allocation_loc_3eb40(
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            ),
            Some(MiniHoppingSkullScreamWorkerAllocation {
                allocation_selector: 0xA000,
                routine: "sub_3F3F0",
                delay_frames: 15,
            }),
            "ASM Mini_Hopping_Skull.asm loc_3EB40 should allocate a helper object for `sub_3F3F0`"
        );
        assert_eq!(
            mini_hopping_skull_scream_worker_allocation_loc_3eb40(
                MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EB40 is skipped when the collision decrements skull HP"
        );
        assert_eq!(
            mini_hopping_skull_scream_worker_allocation_loc_3eb40(
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EB40 is skipped by the charging-Berzerker side-contact branch"
        );
    }

    #[test]
    fn mini_hopping_skull_right_hop_scream_worker_allocation_matches_loc_3ebc4() {
        assert_eq!(
            mini_hopping_skull_right_hop_scream_worker_allocation_loc_3ebc4(
                MiniHoppingSkullGroundWaitCollisionRoute::QueueScreamWorker
            ),
            Some(MiniHoppingSkullScreamWorkerAllocation {
                allocation_selector: 0xA000,
                routine: "sub_3F3F0",
                delay_frames: 15,
            }),
            "ASM Mini_Hopping_Skull.asm loc_3EBC4 should allocate the `sub_3F3F0` contact worker with selector `$A000`"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_scream_worker_allocation_loc_3ebc4(
                MiniHoppingSkullGroundWaitCollisionRoute::DamageSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EBC4 is skipped when the windup collision decrements skull HP"
        );
        assert_eq!(
            mini_hopping_skull_right_hop_scream_worker_allocation_loc_3ebc4(
                MiniHoppingSkullGroundWaitCollisionRoute::BerzerkerChargeKillsSkull
            ),
            None,
            "ASM Mini_Hopping_Skull.asm loc_3EBC4 is skipped by the charging-Berzerker side-contact branch"
        );
    }

    #[test]
    fn mini_hopping_skull_right_hop_relaunch_tail_matches_loc_3ec26_flags() {
        assert_eq!(
            mini_hopping_skull_right_hop_relaunch_tail_loc_3ec26(),
            MiniHoppingSkullRelaunchTail {
                is_moved: true,
                has_level_collision: true,
                next_loop: "loc_3EA88",
            },
            "ASM Mini_Hopping_Skull.asm loc_3EC26 should re-enable movement and level collision before returning to loc_3EA88"
        );
    }

    #[test]
    fn mini_hopping_skull_hp_tier_boost_matches_loc_3ebfc() {
        let grid = mini_hopping_skull_test_grid();
        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.hit_points = 2;

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.x_velocity, 0x0001_C000,
            "ASM Mini_Hopping_Skull.asm loc_3EBFC adds `$4000` to the base `$18000` hop speed for levelled skulls"
        );
        assert_eq!(
            enemy.y_velocity, -0x0004_6E00,
            "ASM Mini_Hopping_Skull.asm loc_3EBFC applies the extra `$10000` upward boost before the shared `$1200` gravity tick"
        );
    }

    #[test]
    fn mini_hopping_skull_wall_bounce_preserves_boosted_speed_per_loc_3f1aa() {
        let mut grid = mini_hopping_skull_test_grid();
        grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.on_ground = false;
        enemy.facing = Direction::Right;
        enemy.x_pos = 23 << 16;
        enemy.y_pos = 24 << 16;
        enemy.x_velocity = 0x0001_C000;
        enemy.y_velocity = 0;

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(enemy.facing, Direction::Left);
        assert_eq!(
            enemy.x_velocity, -0x0001_C000,
            "ASM Mini_Hopping_Skull.asm loc_3F1AA negates the live X velocity, so wall hits must preserve boosted magnitude"
        );
    }

    #[test]
    fn fireball_wall_bounce_keeps_facing_through_loc_32c60_dispatch() {
        let mut grid = mini_hopping_skull_test_grid();
        grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = fireball_test_enemy();
        enemy.on_ground = false;
        enemy.facing = Direction::Right;
        enemy.x_pos = 23 << 16;
        enemy.y_pos = 24 << 16;
        enemy.x_velocity = 0x0001_8000;
        enemy.y_velocity = 0;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Fireball.asm loc_32C60/loc_32C86 dispatches the wall-collision handler, and Enemy19_Fireball_BounceWall only negates x_vel without touching x_direction"
        );
        assert_eq!(enemy.x_velocity, -0x0001_8000);
    }

    #[test]
    fn sphere_wall_bounce_keeps_facing_per_loc_36148() {
        let mut grid = mini_hopping_skull_test_grid();
        grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.on_ground = false;
        enemy.facing = Direction::Right;
        enemy.x_pos = 23 << 16;
        enemy.y_pos = 24 << 16;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Sphere.asm loc_36148 only negates `x_vel(a3)`, so wall hits must not rewrite x_direction"
        );
        assert_eq!(enemy.x_velocity, -0x0001_0000);
    }

    #[test]
    fn sphere_slope_up_backs_out_motion_per_loc_3614e() {
        let data = vec![0u8; 10 * 10 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 10, 10);
        grid.set_cell_at(5, 5, terrain::Cell::from_be_bytes(0x40, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.facing = Direction::Right;
        enemy.x_pos = (5 * 16 + 8) << 16;
        enemy.y_pos = 85 << 16;
        enemy.x_velocity = 0x0002_0000;
        enemy.y_velocity = 0x0002_0000;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_pos,
            (5 * 16 + 8) << 16,
            "ASM Sphere.asm loc_3614E subtracts the pre-collision x motion before swapping velocities"
        );
        assert_eq!(
            enemy.y_pos,
            85 << 16,
            "ASM Sphere.asm loc_3614E also subtracts the pre-collision y motion after gravity is applied"
        );
        assert_eq!(enemy.x_velocity, -(0x0002_0000 + SPHERE_GRAVITY));
        assert_eq!(enemy.y_velocity, -0x0002_0000);
        assert_eq!(enemy.facing, Direction::Right);
    }

    #[test]
    fn sphere_slope_down_backs_out_motion_per_loc_36164() {
        let data = vec![0u8; 10 * 10 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 10, 10);
        grid.set_cell_at(5, 5, terrain::Cell::from_be_bytes(0x50, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.facing = Direction::Right;
        enemy.x_pos = (5 * 16 + 8) << 16;
        enemy.y_pos = 89 << 16;
        enemy.x_velocity = 0x0002_0000;
        enemy.y_velocity = 0x0001_0000;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_pos,
            (5 * 16 + 8) << 16,
            "ASM Sphere.asm loc_36164 backs the sphere out by the original x motion before keeping the swapped vector"
        );
        assert_eq!(
            enemy.y_pos,
            89 << 16,
            "ASM Sphere.asm loc_36164 also backs out the gravity-adjusted y step before resuming"
        );
        assert_eq!(enemy.x_velocity, 0x0001_0000 + SPHERE_GRAVITY);
        assert_eq!(enemy.y_velocity, 0x0002_0000);
        assert_eq!(enemy.facing, Direction::Right);
    }

    #[test]
    fn tar_monster_attack_waits_nine_frames_before_firing_per_loc_32310() {
        let mut enemy = tar_monster_test_enemy();
        let mut projectiles = Vec::new();

        for frame in 0..8 {
            step_tar_monster(&mut enemy, &mut projectiles);
            assert!(
                projectiles.is_empty(),
                "ASM Tar_Monster.asm loc_3231C uses `moveq #8,d0` + `dbf`, so frame {} is still part of the 9-frame wind-up",
                frame + 1
            );
        }

        step_tar_monster(&mut enemy, &mut projectiles);

        assert_eq!(projectiles.len(), 1);
    }

    #[test]
    fn tar_monster_attack_runs_twenty_seven_frames_per_loc_3231c() {
        let mut enemy = tar_monster_test_enemy();
        let mut projectiles = Vec::new();

        for _ in 0..27 {
            step_tar_monster(&mut enemy, &mut projectiles);
        }

        assert_eq!(enemy.tar_monster_state, TarMonsterState::Attacking);
        assert_eq!(enemy.wait_timer, 0);

        step_tar_monster(&mut enemy, &mut projectiles);

        assert_eq!(enemy.tar_monster_state, TarMonsterState::Submerging);
    }

    #[test]
    fn ninja_stealth_detection_excludes_positive_0x10_edge_per_loc_3d706() {
        let mut enemy = ninja_test_enemy();
        let player_x = (enemy.x_pos as i64 + ((NINJA_DETECT_X_RANGE as i64) << 16)) as u32;
        let player_y = (enemy.y_pos as i64 - ((NINJA_DETECT_Y_MIN as i64) << 16)) as u32;

        step_ninja(
            &mut enemy,
            None,
            Some(player_x),
            Some(player_y),
            PlayerForm::TheKid,
        );

        assert_eq!(enemy.ninja_state, NinjaState::Stealth);
        assert_eq!(
            enemy.x_velocity, 0,
            "ASM Ninja.asm loc_3D706 only enters the jump branch when kid_x - ninja_x < $10; the +$10 edge must stay in stealth"
        );
    }
}

pub(crate) fn spawn_y_velocity(family: EnemyFamily, enemy_level: u8) -> i32 {
    match family {
        // Big_Hopping_Skull.asm: move.w #$FFFE,y_vel → $FFFE0000 FP16.16
        EnemyFamily::BigHoppingSkull => BIG_SKULL_INIT_Y_SPEED,
        EnemyFamily::MiniHoppingSkull => {
            // ASM Mini_Hopping_Skull.asm loc_3EA48 seeds y_vel with `$28000`;
            // non-zero enemy_level at `$40(a3)` adds `$1000`.
            MINI_SKULL_INITIAL_Y_SPEED_FP
                + if enemy_level != 0 {
                    MINI_SKULL_LEVEL_INITIAL_Y_SPEED_BONUS_FP
                } else {
                    0
                }
        }
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
        EnemyFamily::Tank | EnemyFamily::TankShooting => 2,
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
        EnemyFamily::Diamond => 2, // Diamond.asm loc_3AD16: `addq.w #2,d7`
        // Mini Hopping Skull still uses raw level as HP with a minimum of 1.
        _ => 0,
    };
    (enemy_level + base).max(1)
}

pub(crate) fn emo_rock_wait_timer_with_rng(rng: &mut RomRandomState, player_x: u32) -> u16 {
    // ASM Emo_Rock.asm loc_361F4: seed d0 with `$3C`, XOR the RNG byte with
    // Kid's X low byte, clear bit 7, then add the masked offset.
    let random_byte = rng.next_byte();
    let kid_x_low_byte = ((player_x >> 16) & 0xFF) as u8;
    u16::from((random_byte ^ kid_x_low_byte) & 0x7F) + 0x003C
}

pub(crate) fn spawn_wait_timer(
    family: EnemyFamily,
    enemy_level: u8,
    player_x: u32,
    rng: &mut RomRandomState,
) -> u16 {
    match family {
        EnemyFamily::EmoRock => emo_rock_wait_timer_with_rng(rng, player_x),
        // ASM Tank_Shooting.asm `stru_3B4D8`/`stru_3B4F6` reaches the
        // `loc_3C50E` fire gate 74 frames after init. Plain Tank.asm never fires.
        EnemyFamily::Tank => 0,
        EnemyFamily::TankShooting => TANK_SHOOTING_FIRST_SHOT_DELAY_FRAMES.saturating_sub(1),
        // Archer: fire first arrow after one full interval (HP1 rate).
        EnemyFamily::Archer => ARCHER_FIRE_TIMER[0],
        // Lion: charge timer counts down from $3C (60 frames) before first charge.
        EnemyFamily::Lion => LION_CHARGE_TIMER_INIT,
        // Robot: ASM init $54(a5) low word = 1 (fires on 2nd frame).
        EnemyFamily::Robot => 1,
        // TarMonster: starts submerged, waits before first emergence.
        EnemyFamily::TarMonster => TAR_MONSTER_SUBMERGED_FRAMES,
        // ASM Drips.asm loc_3340E seeds the dormant delay from RNG and Kid X.
        EnemyFamily::Drips => drips_wait_timer_with_rng_loc_3340e(rng, player_x),
        // DragonWalking: first breath after cooldown.
        EnemyFamily::DragonWalking => DRAGON_WALKING_BREATH_INITIAL,
        // Armadillo.asm loc_3C7F8 seeds `$54(a5)` with `$12C` before entering
        // the shared grounded walker loop.
        EnemyFamily::Armadillo => 0x012C,
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
pub(crate) struct EnemyArtPaletteEntryLoc36D16 {
    pub(crate) enemy_id: u16,
    pub(crate) palette_variant: u8,
    pub(crate) enemy_load_index_offset: u16,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_art_palette_entry_loc_36D16(
    raw_slot: u16,
) -> Option<EnemyArtPaletteEntryLoc36D16> {
    // ASM loc_36D16 (kid.asm:38603): consume one enemy-layout header word,
    // treat `$FFFF` as a blank art slot, mask the low 12 bits for
    // `EnemyLoad_Index`, and rotate the top two bits down for the palette
    // variant before choosing the palette pointer.
    if raw_slot == 0xFFFF {
        return None;
    }

    let enemy_id = raw_slot & 0x0FFF;
    Some(EnemyArtPaletteEntryLoc36D16 {
        enemy_id,
        palette_variant: u8::try_from(raw_slot.rotate_left(2) & 0x0003)
            .expect("palette variant is masked to two bits"),
        enemy_load_index_offset: enemy_id << 3,
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_palette_variant_addend_loc_36D4E(palette_variant: u8) -> u16 {
    // ASM loc_36D4E (kid.asm:38623): variant 1 branches directly here for a
    // single `addi.w #4,d2`; variants 2 and 3 run the preceding add first,
    // then fall through here for the second `+4`.
    match palette_variant & 0x03 {
        0 => 0,
        1 => 4,
        _ => 8,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_palette_data_index_offset_loc_36D52(
    enemy_load_palette_offset: u16,
    palette_variant: u8,
) -> u16 {
    // ASM loc_36D52 (kid.asm:38626): after the variant-specific `d2` addend,
    // `move.l (Data_Index,d2.w),a0` dereferences the chosen palette pointer.
    enemy_load_palette_offset.wrapping_add(enemy_palette_variant_addend_loc_36D4E(palette_variant))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyArtDecompressRouteLoc36E24 {
    ForestRam,
    ForestTarMonsterRam,
    Vram,
    SpecialVram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyArtLoadTailLoc36E3E {
    pub(crate) next_slot_index: u16,
    pub(crate) finished: bool,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_art_decompress_route_loc_36E24(
    slot_index: u16,
    background_is_forest: bool,
    family: EnemyFamily,
    use_special_vram_palette: bool,
) -> EnemyArtDecompressRouteLoc36E24 {
    // ASM loc_36E24 (kid.asm:38718): only the first enemy slot on Forest
    // backgrounds reaches `j_DecompressToRAM`; TarMonster swaps `a3` to the
    // alternate forest permutation immediately before this shared call.
    if slot_index == 0 && background_is_forest {
        if matches!(family, EnemyFamily::TarMonster) {
            EnemyArtDecompressRouteLoc36E24::ForestTarMonsterRam
        } else {
            EnemyArtDecompressRouteLoc36E24::ForestRam
        }
    } else if use_special_vram_palette {
        EnemyArtDecompressRouteLoc36E24::SpecialVram
    } else {
        // ASM loc_36E38 (kid.asm:38730): when neither the Forest RAM path nor
        // the special VRAM decompressor applies, call `j_DecompressToVRAM`.
        EnemyArtDecompressRouteLoc36E24::Vram
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_art_load_tail_loc_36E3E(slot_index: u16) -> EnemyArtLoadTailLoc36E3E {
    // ASM loc_36E3E/loc_36E42 (kid.asm:38733-38739): restore the saved
    // `d7/a1`, increment the restored slot index, and loop back to loc_36D16
    // until the post-increment value reaches three.
    let next_slot_index = slot_index.wrapping_add(1);
    EnemyArtLoadTailLoc36E3E {
        next_slot_index,
        finished: next_slot_index == 3,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyArtSlotAssignmentSub36E84 {
    pub(crate) slot_index: u16,
    pub(crate) palette_line: u8,
    pub(crate) vram_tile: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyGfxSlotLinkLoc32188 {
    pub(crate) next_free_gfx_slot: u32,
    pub(crate) allocated_slot_next: u32,
    pub(crate) owner_gfx_head: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyGfxSlotInitLoc3219E {
    pub(crate) active_word_offset: u16,
    pub(crate) active_word_value: u16,
    pub(crate) next_number_gfx_objects: u16,
    pub(crate) a1_slot_address: u32,
    pub(crate) clear_start_offset: u16,
    pub(crate) cleared_byte_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SharedEnemyAttachmentChildSub3BF72 {
    pub(crate) x_pos: i16,
    pub(crate) y_pos: i16,
    pub(crate) x_velocity: i32,
    pub(crate) x_direction: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SharedEnemyAttachmentLayoutSub3BF72 {
    pub(crate) primary: SharedEnemyAttachmentChildSub3BF72,
    pub(crate) secondary: Option<SharedEnemyAttachmentChildSub3BF72>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SharedEnemyAttachmentInputSub3BF72 {
    pub(crate) family: EnemyFamily,
    pub(crate) parent_x_pos: i16,
    pub(crate) parent_y_pos: i16,
    pub(crate) parent_x_direction: u8,
    pub(crate) child_y_offset: i16,
    pub(crate) robot_parent_state: u16,
    pub(crate) robot_child_mapping: u16,
    pub(crate) child_x_direction: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SharedEnemyDriveSoundEffectSub3C352 {
    DrillMoving,
    TankDriving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SharedEnemyDriveSoundCallSub3C352 {
    PlaySound(SharedEnemyDriveSoundEffectSub3C352),
    PlaySound2(SharedEnemyDriveSoundEffectSub3C352),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SharedEnemyDriveSoundResultSub3C352 {
    pub(crate) next_previous_motion_active: bool,
    pub(crate) next_refcount: u8,
    pub(crate) sound_call: Option<SharedEnemyDriveSoundCallSub3C352>,
}

// ASM loc_3C03A / loc_3C0E6 (kid.asm:43500-43503): after velocity clear +
// direction set, `tst.w $3A(a5)` checks whether the enemy has a child
// attachment pointer; if nonzero, `bsr.w sub_3BF72` syncs the child's
// position/direction to the parent before the hibernate frame at loc_3C044.
// ASM loc_3C18A (kid.asm:43648-43651): identical attachment check in the
// leftward walking loop — reached via loc_3C25E after gravity update,
// runs once per frame before hibernate at loc_3C194.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shared_enemy_attachment_sub_3BF72(
    input: SharedEnemyAttachmentInputSub3BF72,
) -> SharedEnemyAttachmentLayoutSub3BF72 {
    let SharedEnemyAttachmentInputSub3BF72 {
        family,
        parent_x_pos,
        parent_y_pos,
        parent_x_direction,
        child_y_offset,
        robot_parent_state,
        robot_child_mapping,
        child_x_direction,
    } = input;

    if matches!(family, EnemyFamily::Driller) {
        // ASM sub_3BF72 / loc_3BFD4 (kid.asm:43464-43487): Driller keeps a
        // paired child layout centred on the parent, with asymmetric 23/24px
        // X offsets and opposing `$8000` tread velocities.
        // ASM loc_3C000: nonzero x_direction → subi.w #$17,x_pos(a4); addi.w #$18,$1A(a2)
        // ASM zero direction: subi.w #$18,x_pos(a4); addi.w #$17,$1A(a2); bra.s loc_3C00C
        let (primary_x, secondary_x) = if parent_x_direction == 0 {
            (parent_x_pos - 0x18, parent_x_pos + 0x17)
        } else {
            // ASM loc_3C000: right-facing Driller offsets
            (parent_x_pos - 0x17, parent_x_pos + 0x18)
        };
        // ASM loc_3C00C: convergence — copy y_pos to both objects, set
        // x_vel=$FFFF8000 for primary (a4) and x_vel=$8000 for secondary (a2)
        return SharedEnemyAttachmentLayoutSub3BF72 {
            primary: SharedEnemyAttachmentChildSub3BF72 {
                x_pos: primary_x,
                y_pos: parent_y_pos,
                x_velocity: -0x8000,
                x_direction: None,
            },
            secondary: Some(SharedEnemyAttachmentChildSub3BF72 {
                x_pos: secondary_x,
                y_pos: parent_y_pos,
                x_velocity: 0x8000,
                x_direction: None,
            }),
        };
    }

    // ASM sub_3BF72 (kid.asm:43425-43443): shared child sprite copies the
    // parent's world position, then subtracts the child's `$46(a4)` Y offset.
    // ASM loc_3BFA0: move.b x_direction(a3),$16(a4) — default direction copy
    let mut primary_x_pos = parent_x_pos;
    let primary_y_pos = parent_y_pos - child_y_offset;
    let mut primary_x_direction = parent_x_direction;

    if matches!(family, EnemyFamily::Robot)
        // ASM loc_3BFA8 only enters the offset-preserving Robot path once the
        // child leaves the special `$800` mapping and `$40(a3)` is non-zero.
        && robot_child_mapping != 0x0800
        && robot_parent_state != 0
        && child_x_direction != parent_x_direction
    {
        // ASM loc_3BFBE/loc_3BFCC preserves the old child flip while nudging
        // the child ±4px so the arm does not pop when the parent turns.
        primary_x_direction = child_x_direction;
        if child_x_direction == 0 {
            primary_x_pos += 4;
        } else {
            primary_x_pos -= 4;
        }
    }

    SharedEnemyAttachmentLayoutSub3BF72 {
        primary: SharedEnemyAttachmentChildSub3BF72 {
            x_pos: primary_x_pos,
            y_pos: primary_y_pos,
            x_velocity: 0,
            x_direction: Some(primary_x_direction),
        },
        secondary: None,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shared_enemy_drive_sound_sub_3C352(
    family: EnemyFamily,
    motion_active: bool,
    previous_motion_active: bool,
    refcount: u8,
) -> SharedEnemyDriveSoundResultSub3C352 {
    let Some(effect) = (match family {
        // ASM sub_3C352 (kid.asm:43849-43880): `d6=0` selects
        // `byte_3C46C[0]=sfx_Drill_moving`.
        EnemyFamily::Driller => Some(SharedEnemyDriveSoundEffectSub3C352::DrillMoving),
        // ASM sub_3C352: `d6=1` selects `byte_3C46C[1]=sfx_Tank_driving` for
        // both RockTank variants.
        EnemyFamily::Tank | EnemyFamily::TankShooting => {
            Some(SharedEnemyDriveSoundEffectSub3C352::TankDriving)
        }
        _ => None,
    }) else {
        return SharedEnemyDriveSoundResultSub3C352 {
            next_previous_motion_active: previous_motion_active,
            next_refcount: refcount,
            sound_call: None,
        };
    };

    // ASM loc_3C372 (kid.asm:43861-43872): motion_active ($19(a3)) is
    // nonzero — stopped→moving edge. Checks previous_motion_active
    // ($6A(a5)); if already active, skip (loc_3C3C6). Otherwise
    // subi.b #1 on shared refcount at $FFFFFA34+d7; when counter
    // reaches zero, plays sound via j_PlaySound2 using byte_3C46C[d6].
    if motion_active {
        if previous_motion_active {
            return SharedEnemyDriveSoundResultSub3C352 {
                next_previous_motion_active: true,
                next_refcount: refcount,
                sound_call: None,
            };
        }

        let next_refcount = refcount.wrapping_sub(1);
        return SharedEnemyDriveSoundResultSub3C352 {
            next_previous_motion_active: true,
            next_refcount,
            sound_call: (next_refcount == 0)
                .then_some(SharedEnemyDriveSoundCallSub3C352::PlaySound2(effect)),
        };
    }

    // ASM loc_3C39C (kid.asm:43875-43889): motion_active ($19(a3)) is
    // zero — moving→stopped edge. Checks previous_motion_active
    // ($6A(a5)); if already stopped, skip (loc_3C3C6). Otherwise
    // plays sound via j_PlaySound using byte_3C46C[d6] when shared
    // refcount at $FFFFFA34+d7 is zero, then addi.b #1 increments it.
    if !previous_motion_active {
        return SharedEnemyDriveSoundResultSub3C352 {
            next_previous_motion_active: false,
            next_refcount: refcount,
            sound_call: None,
        };
    }

    // ASM loc_3C3BE (kid.asm:43886-43889): convergence point after the
    // optional j_PlaySound — `move.l d5,a4; addi.b #1,(a4,d7.w)` restores
    // the $FFFFFA34 base and increments the shared refcount unconditionally.
    // The sound plays only when refcount was zero (loc_3C39C tst.b check);
    // the increment always executes regardless.
    SharedEnemyDriveSoundResultSub3C352 {
        next_previous_motion_active: false,
        next_refcount: refcount.wrapping_add(1),
        sound_call: (refcount == 0).then_some(SharedEnemyDriveSoundCallSub3C352::PlaySound(effect)),
    }
}

/// Result of the deactivation-time drive sound cleanup at loc_3C494.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeactivationDriveSoundResultLoc3C494 {
    pub(crate) next_refcount: u8,
    pub(crate) sound_call: Option<SharedEnemyDriveSoundCallSub3C352>,
}

/// ASM loc_3C494 (kid.asm:43979-43997): deactivation-time drive sound cleanup
/// for Driller/Tank enemies.  Reached from loc_3C46E's object_meta dispatch
/// when d7 matches objid_Driller or objid_RockTank/objid_RockTank_shooting.
///
/// The routine checks two flags — `$19(a3)` (motion_active) and `$6A(a5)`
/// (previous_motion_active) — to decide whether to decrement the shared drive
/// sound refcount at `$FFFFFA34+d7`.
///
/// - loc_3C4A2 (kid.asm:43987): `$19(a3)==0` path — tests `$6A(a5)`;
///   if nonzero skips sound cleanup (`bne.s loc_3C4C0`), otherwise falls
///   through to loc_3C4A8.
/// - loc_3C4A8 (kid.asm:43991): shared refcount decrement path —
///   `lea ($FFFFFA34).w,a4; subi.b #1,(a4,d7.w)`.  When counter reaches
///   zero, plays `j_PlaySound2` with `byte_3C46C[d6]` (Drill or Tank sfx).
///
/// In both `$19` states the gate is the same: `$6A != 0` → skip sound.
/// So the deactivation sound cleanup runs iff `$6A(a5) == 0`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn deactivation_drive_sound_cleanup_loc_3C494(
    family: EnemyFamily,
    motion_active: bool,
    previous_motion_active: bool,
    refcount: u8,
) -> DeactivationDriveSoundResultLoc3C494 {
    let Some(effect) = (match family {
        EnemyFamily::Driller => Some(SharedEnemyDriveSoundEffectSub3C352::DrillMoving),
        EnemyFamily::Tank | EnemyFamily::TankShooting => {
            Some(SharedEnemyDriveSoundEffectSub3C352::TankDriving)
        }
        _ => None,
    }) else {
        return DeactivationDriveSoundResultLoc3C494 {
            next_refcount: refcount,
            sound_call: None,
        };
    };

    // ASM loc_3C494: tst.b $19(a3); beq.s loc_3C4A2
    // Both paths check $6A(a5) and skip if nonzero.
    if motion_active {
        // $19 != 0: tst.b $6A(a5); bne.s loc_3C4C0; bra.s loc_3C4A8
        if previous_motion_active {
            return DeactivationDriveSoundResultLoc3C494 {
                next_refcount: refcount,
                sound_call: None,
            };
        }
    } else {
        // ASM loc_3C4A2: $19 == 0: tst.b $6A(a5); bne.s loc_3C4C0
        if previous_motion_active {
            return DeactivationDriveSoundResultLoc3C494 {
                next_refcount: refcount,
                sound_call: None,
            };
        }
    }

    // ASM loc_3C4A8: lea ($FFFFFA34).w,a4; subi.b #1,(a4,d7.w); bne.s loc_3C4C0
    let next_refcount = refcount.wrapping_sub(1);
    DeactivationDriveSoundResultLoc3C494 {
        next_refcount,
        sound_call: (next_refcount == 0)
            .then_some(SharedEnemyDriveSoundCallSub3C352::PlaySound2(effect)),
    }
}

/// Result of the deactivation status table update at loc_3C4C0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeactivationStatusActionLoc3C4C0 {
    /// loc_3C4E6: bit 7 clear (positive slot) — write `$2168` to
    /// `EnemyStatus_Table[d0]`.
    WriteRespawnMarker { index: usize },
    /// Bit 7 set + bit 6 set — subtract `$400` from `EnemyStatus_Table[d0 & $3F]`.
    SubtractSpawnCounter { index: usize },
    /// Bit 7 set + bit 6 clear — no table update, just delete.
    DeleteOnly,
}

/// ASM loc_3C4C0 (kid.asm:43999-44020): deactivation status table update.
/// After decrementing `Number_of_Enemy`, reads `$42(a5)` and dispatches:
///
/// - `bpl.s loc_3C4E6`: bit 7 clear → positive slot.
///   loc_3C4E6 (kid.asm:44013): `add.w d0,d0; lea (EnemyStatus_Table).w,a0;
///   move.w #$2168,d7; move.w d7,(a0,d0.w)` writes the respawn marker.
///
/// - Bit 7 set + bit 6 set → `andi.w #$3F,d0; add.w d0,d0;
///   subi.w #$400,(a0,d0.w)` decrements the spawn counter.
///
/// - Bit 7 set + bit 6 clear → falls straight to loc_3C4F4 (delete).
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn deactivation_status_update_loc_3C4C0(
    object_status_slot: u8,
) -> DeactivationStatusActionLoc3C4C0 {
    // ASM: moveq #0,d0; move.b $42(a5),d0; bpl.s loc_3C4E6
    if object_status_slot & 0x80 == 0 {
        // ASM loc_3C4E6: add.w d0,d0 → word index; move.w #$2168,(a0,d0.w)
        return DeactivationStatusActionLoc3C4C0::WriteRespawnMarker {
            index: usize::from(object_status_slot),
        };
    }
    // ASM: btst #6,d0; beq.s loc_3C4F4
    if object_status_slot & 0x40 == 0 {
        return DeactivationStatusActionLoc3C4C0::DeleteOnly;
    }
    // ASM: andi.w #$3F,d0; add.w d0,d0; subi.w #$400,(a0,d0.w)
    DeactivationStatusActionLoc3C4C0::SubtractSpawnCounter {
        index: usize::from(object_status_slot & 0x3F),
    }
}

pub(crate) const ROCK_TANK_SHOOTING_MAPPING_SUB_3C4F8: u16 = 0x7F10;
pub(crate) const ROBOT_IDLE_MAPPING_SUB_3C4F8: u16 = 0x82F0;
pub(crate) const ROBOT_ATTACK_MAPPING_A_SUB_3C4F8: u16 = 0x8338;
pub(crate) const ROBOT_ATTACK_MAPPING_B_SUB_3C4F8: u16 = 0x8340;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum SharedEnemyRobotHelperKindSub3C4F8 {
    RockTankShooting,
    Robot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SharedEnemyRobotHelperInputSub3C4F8 {
    pub(crate) helper_kind: SharedEnemyRobotHelperKindSub3C4F8,
    pub(crate) parent_mapping: u16,
    pub(crate) parent_state_word_32: u16,
    pub(crate) child_state_word_40: u16,
    pub(crate) child_counter_word_4e: u16,
    pub(crate) child_timer_word_54: u16,
    /// ASM loc_3C56A: `x_pos(a4)` — the turret child's X position.
    pub(crate) parent_x_pos: u16,
    /// ASM loc_3C5FC: `x_pos(a3)` — the walker's own X position, used when
    /// `$40(a3)==2` selects the alternate direction-check path.
    pub(crate) walker_x_pos: u16,
    pub(crate) kid_x_pos: u16,
    pub(crate) parent_x_direction: u8,
    pub(crate) parent_flag_15: bool,
    pub(crate) random_word: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SharedEnemyRobotHelperResultSub3C4F8 {
    pub(crate) spawned_loc_3_b730_worker: bool,
    pub(crate) next_parent_mapping: u16,
    pub(crate) next_parent_x_direction: u8,
    pub(crate) next_parent_flag_15: bool,
    pub(crate) next_child_counter_word_4e: u16,
    pub(crate) next_child_timer_word_54: u16,
    pub(crate) initialised_parent_animation_stru_3_a716: bool,
    pub(crate) branch_to_robot_shoot: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shared_enemy_robot_helper_sub_3C4F8(
    input: SharedEnemyRobotHelperInputSub3C4F8,
) -> SharedEnemyRobotHelperResultSub3C4F8 {
    let mut result = SharedEnemyRobotHelperResultSub3C4F8 {
        spawned_loc_3_b730_worker: false,
        next_parent_mapping: input.parent_mapping,
        next_parent_x_direction: input.parent_x_direction,
        next_parent_flag_15: input.parent_flag_15,
        next_child_counter_word_4e: input.child_counter_word_4e,
        next_child_timer_word_54: input.child_timer_word_54,
        initialised_parent_animation_stru_3_a716: false,
        branch_to_robot_shoot: false,
    };

    match input.helper_kind {
        SharedEnemyRobotHelperKindSub3C4F8::RockTankShooting => {
            // ASM sub_3C4F8 loc_3C50E only allocates `loc_3B730` when the
            // RockTank parent still uses `LnkTo_unk_C7F10` and `$32(a4)=5`.
            if input.parent_mapping == ROCK_TANK_SHOOTING_MAPPING_SUB_3C4F8
                && input.parent_state_word_32 == 5
            {
                result.spawned_loc_3_b730_worker = true;
            }
            result
        }
        SharedEnemyRobotHelperKindSub3C4F8::Robot => {
            if input.child_counter_word_4e == 0 {
                // ASM loc_3C538 only runs the Robot helper from the idle
                // parent shell `LnkTo_unk_C82F0` with `$32(a4)=9`.
                if input.parent_mapping != ROBOT_IDLE_MAPPING_SUB_3C4F8
                    || input.parent_state_word_32 != 9
                {
                    return result;
                }
                // ASM loc_3C554 (kid.asm:44066-44072): cmpi.w #2,$40(a3);
                // beq.w loc_3C5FC — $40==2 skips the random gate entirely
                // and uses x_pos(a3) for direction check. Otherwise
                // j_Get_RandomNumber; cmpi.w #$A,d7; ble loc_3C56A — only
                // proceed when random <= $A.
                if input.child_state_word_40 != 2 && input.random_word > 0x000A {
                    return result;
                }
                if input.child_state_word_40 < 1 {
                    result.branch_to_robot_shoot = true;
                    return result;
                }

                // ASM loc_3C56A (kid.asm:44075-44084): uses x_pos(a4) for
                // the direction check. loc_3C5FC (kid.asm:44138-44157):
                // uses x_pos(a3) — the walker's own position — when
                // $40(a3)==2.
                let x_for_check = if input.child_state_word_40 == 2 {
                    input.walker_x_pos // ASM loc_3C5FC: x_pos(a3)
                } else {
                    input.parent_x_pos // ASM loc_3C56A: x_pos(a4)
                };

                // ASM loc_3C588 (kid.asm:44087-44089): bmi.s path — when
                // x_pos < Kid_x (entity is left of Kid), tst.b $16(a4);
                // bne.s loc_3C5EE — facing right already means facing Kid,
                // so branch to Robot_Shoot. The non-bmi path (entity
                // right of or at Kid) mirrors: beq.s loc_3C5EE when
                // $16(a4)==0 (facing left, towards Kid).
                // ASM loc_3C612 (kid.asm:44148-44150): identical check in
                // the loc_3C5FC ($40==2) path — bmi.s to loc_3C612: tst.b
                // $16(a4); bne.s loc_3C5EE when turret left of Kid and
                // already facing right (towards Kid).
                let entity_left_of_kid = (x_for_check as i16) < (input.kid_x_pos as i16);
                let parent_faces_right = input.parent_x_direction != 0;
                let facing_matches_kid = if entity_left_of_kid {
                    parent_faces_right
                } else {
                    !parent_faces_right
                };
                if facing_matches_kid {
                    result.branch_to_robot_shoot = true;
                    return result;
                }

                // ASM loc_3C618 (kid.asm:44152-44157): arms the helper
                // phase — identical to loc_3C58E: st $4E(a5), sf $15(a4),
                // timer=9, mapping=C8338, then bra.w loc_3C5A2. Reached
                // from the loc_3C5FC ($40==2) path when the turret is not
                // already facing Kid.
                result.next_child_counter_word_4e |= 0xFF00;
                result.next_parent_flag_15 = false;
                result.next_child_timer_word_54 = 9;
                result.next_parent_mapping = ROBOT_ATTACK_MAPPING_A_SUB_3C4F8;
            }

            // ASM loc_3C5A2 decrements `$54(a5)` before inspecting the low byte
            // of `$4E(a5)`. The helper therefore burns one frame immediately
            // when loc_3C5FC falls through into the countdown path.
            if result.next_child_timer_word_54 > 0 {
                result.next_child_timer_word_54 -= 1;
            }
            if result.next_child_timer_word_54 != 0 {
                return result;
            }

            let phase = (result.next_child_counter_word_4e & 0x00FF).wrapping_add(1);
            // ASM loc_3C5D8 (kid.asm:44116-44126): phase > 2 completes the
            // turn sequence — st $15(a4), clr.w $4E(a5), init stru_3A716 on
            // parent, then falls through to loc_3C5EE which pops the return
            // address and bra.w Enemy03_Robot_Shoot.
            if phase > 2 {
                result.next_parent_flag_15 = true;
                result.next_child_counter_word_4e = 0;
                result.initialised_parent_animation_stru_3_a716 = true;
                result.branch_to_robot_shoot = true;
                return result;
            }

            if phase == 2 {
                result.next_parent_x_direction ^= 0xFF;
            }
            // ASM loc_3C5C4 (kid.asm:44108-44113): or.w d7,$4E(a5) stores
            // the incremented phase; add.w d7,d7 doubles it as a word
            // offset into off_3C5F6 → $22(a4). Timer reset to 9.
            // off_3C5F6: [LnkTo_unk_C8338, LnkTo_unk_C8340, LnkTo_unk_C8338]
            //   phase 1 → offset 2 → C8340 (ROBOT_ATTACK_MAPPING_B)
            //   phase 2 → offset 4 → C8338 (ROBOT_ATTACK_MAPPING_A)
            result.next_child_counter_word_4e |= phase;
            result.next_parent_mapping = match phase {
                1 => ROBOT_ATTACK_MAPPING_B_SUB_3C4F8,
                2 => ROBOT_ATTACK_MAPPING_A_SUB_3C4F8,
                _ => unreachable!(),
            };
            result.next_child_timer_word_54 = 9;
            result
        }
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_art_slot_assignment_sub_36E84(
    enemy_type_slots: &[u16],
    enemy_id: u16,
) -> EnemyArtSlotAssignmentSub36E84 {
    // ASM sub_36E84 / loc_32146 / loc_3215E / loc_3216C / loc_3217C
    // (kid.asm:37582-37609) mirrors the enemy art slot search used by
    // loc_36EA6 (kid.asm:38819): seed the slot index at zero, compare the
    // masked 12-bit enemy id against the first two header slots, then use the
    // selected index to write `palette_line(a3)`, `$3E(a3)`, and
    // `vram_tile(a3)` from the ROM art tables.
    const ENEMY_ART_PALETTE_LINES: [u8; 4] = [1, 2, 2, 0];
    const ENEMY_ART_VRAM_TILE_ADDRESSES: [u16; 3] = [0x02FB, 0x0409, 0x0517];

    let enemy_id = enemy_id & 0x0FFF;
    let slot_index = if enemy_type_slots
        .first()
        .is_some_and(|slot| slot & 0x0FFF == enemy_id)
    {
        0
    } else if enemy_type_slots
        // ASM loc_3215E (kid.asm:37596-37602): read `4(a0)`, mask it with
        // `$0FFF`, compare it to `d0`, and allocate slot 1 when it matches;
        // otherwise increment `d1` again so loc_3216C falls back to slot 2.
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

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_gfx_slot_link_loc_32188(
    free_slot_address: u32,
    next_free_slot_address: u32,
    owner_gfx_head: u32,
) -> EnemyGfxSlotLinkLoc32188 {
    // ASM loc_32188 (kid.asm:37597-37603): save `a0`, pop the first node from
    // `Addr_NextFreeGfxObjectSlot`, copy the owner's current `0(a3)` link into
    // the allocated slot's `0(a0)`, then install that slot as the new
    // owner-head link at `0(a3)`.
    EnemyGfxSlotLinkLoc32188 {
        next_free_gfx_slot: next_free_slot_address,
        allocated_slot_next: owner_gfx_head,
        owner_gfx_head: free_slot_address,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn enemy_gfx_slot_init_loc_3219E(
    allocated_slot_address: u32,
    number_gfx_objects: u16,
) -> EnemyGfxSlotInitLoc3219E {
    // ASM loc_3219E (kid.asm:37604-37608) writes `#1` to `$32(a0)`,
    // increments `Number_GfxObjects`, copies the allocated slot address into
    // `a1`, then advances `a0` to `4(a0)` so loc_321B2 can clear `$48` bytes
    // across the rest of the gfx slot body.
    EnemyGfxSlotInitLoc3219E {
        active_word_offset: 0x0032,
        active_word_value: 0x0001,
        next_number_gfx_objects: number_gfx_objects.wrapping_add(1),
        a1_slot_address: allocated_slot_address,
        clear_start_offset: 0x0004,
        cleared_byte_count: 0x0048,
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
            self.block_state.reset_shooter_state_sub_F06A();
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
        let blank_top_rows = self.level_header(self.current_level).map_or(0, |header| {
            terrain::blank_top_rows_from_height_flag_bits_sub_140F6(header.height_flag_bits)
        });
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
                    slice.player.y_pos,
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
