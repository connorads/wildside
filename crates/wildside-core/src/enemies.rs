use super::*;

// Private items from lib.rs that don't come through the glob import.
use super::{
    ACTIVATION_WINDOW_PADDING, ACTIVATION_WINDOW_X_LEAD, ACTIVATION_WINDOW_Y_LEAD, CameraState,
    EnemyProjectile, FLYER_RETENTION_X_PADDING, FLYER_RETENTION_Y_PADDING, MAX_PROJECTILES,
};
use crate::combat::{
    QueuedSoundEffect, boss_death_burst_init_sub_370D2, boss_eye_counter_orbit_placement_sub_39420,
    boss_eye_orbit_placement_sub_393C8, boss_head_death_anim_sequence_loc_39EE2,
    boss_head_death_spin_loop_loc_39F7A, boss_head_movement_step_for_family_sub_39846,
    boss_head_projectile_spawn_sub_397E4, heady_metal_palette_cycle_sub_379E8,
    heady_metal_spawn_animation_loc_3761E, heady_metal_tracking_velocity_loc_37582,
    shishka_bagel_head_1_init_loc_39478, shishka_bagel_head_2_init_loc_39598,
    shishka_bagel_head_3_init_loc_396BE,
};
use crate::projectiles::fire_demon_trail_has_floor_support_loc_3a63c;

// Per-family enemy engines split into sibling files (issue #40). Each child opens
// `use super::*;` to reach the shared types/engines that stay in this parent, and
// `pub(crate) use <family>::*;` re-exports its symbols so `enemies::X` paths and the
// lib.rs crate-root glob resolve exactly as before the move. Shared machinery
// (dispatcher, `ActiveEnemy`, pacer/walker engines, death-anim tables) stays here.
mod cloud;
pub(crate) use cloud::*;
mod ufo;
pub(crate) use ufo::*;
mod tornado;
pub(crate) use tornado::*;
mod hand;
pub(crate) use hand::*;
mod scorpion;
pub(crate) use scorpion::*;
mod emo_rock;
pub(crate) use emo_rock::*;
mod archer;
pub(crate) use archer::*;
mod goat;
pub(crate) use goat::*;
mod driller;
pub(crate) use driller::*;
mod diamond;
pub(crate) use diamond::*;
mod armadillo;
pub(crate) use armadillo::*;
mod orca;
pub(crate) use orca::*;
mod crab;
pub(crate) use crab::*;
mod fire_demon;
pub(crate) use fire_demon::*;
mod tar_monster;
pub(crate) use tar_monster::*;
mod robot;
pub(crate) use robot::*;
mod tank;
pub(crate) use tank::*;
mod big_hopping_skull;
pub(crate) use big_hopping_skull::*;
mod mini_hopping_skull;
pub(crate) use mini_hopping_skull::*;
mod dragon_walking;
pub(crate) use dragon_walking::*;
mod ninja;
pub(crate) use ninja::*;
mod lion;
pub(crate) use lion::*;
mod spinning_twins;
pub(crate) use spinning_twins::*;
mod drips;
pub(crate) use drips::*;
mod dragon;
pub(crate) use dragon::*;
mod fireball;
#[allow(unused_imports)]
pub(crate) use fireball::*;
mod sphere;
#[allow(unused_imports)]
pub(crate) use sphere::*;

// MiniHoppingSkull hopper constants recovered from the enemy module.
pub(crate) const MINI_SKULL_JUMP_X_SPEED_FP: i32 = 0x0001_8000;
pub(crate) const MINI_SKULL_JUMP_Y_SPEED_FP: i32 = -0x0003_8000;
pub(crate) const MINI_SKULL_GRAVITY_FP: i32 = 0x0000_1200;
pub(crate) const MINI_SKULL_INITIAL_Y_SPEED_FP: i32 = 0x0002_8000;
pub(crate) const MINI_SKULL_LEVEL_INITIAL_Y_SPEED_BONUS_FP: i32 = 0x0000_1000;
pub(crate) const MINI_SKULL_GROUND_DELAY_FRAMES: u8 = 21; // ASM Mini_Hopping_Skull.asm:99,141 — dbf #$14 = 21 iterations
pub(crate) const MINI_SKULL_FACING_MATCHED_PAUSE_FRAMES_LOC_3F09E: u16 = 11;
pub(crate) const MINI_SKULL_FACING_MATCHED_PAUSE_TIMER_FLAG_LOC_3F09E: u16 = 0x8000;

/// ASM Enemy11_Ninja_Init `loc_3D59A`/`loc_3D5AA` final speed arithmetic in `$50/$5E(a5)`:
/// - `current_hp < 3`: one unconditional `loc_3D5AA` fallthrough path (`+$8000` ×2).
/// - `current_hp == 3`: `loc_3D59A` then `loc_3D5AA` (`+$8000` ×2).
/// - `current_hp > 3`: base `+$8000` + both branch adds (`+$8000` ×3), so net 3×.
///   This is the native selection used by `step_pacer` speed resolution.
pub(crate) fn ninja_walk_speed_override_loc_3d59a_and_3d5aa(current_hp: u8) -> i32 {
    if current_hp > 3 {
        0x0002_4000
    } else {
        0x0001_8000
    }
}

/// Canonical collision-ID values shared by every enemy family.
///
/// These mirror the `; Collision` jump-table in the Kid Chameleon disassembly
/// (`constants.asm:163-175`): one 12-entry table, stride 4, that every enemy
/// collision handler indexes. The per-family `*_COLID_*` aliases below (e.g.
/// `NINJA_COLID_HURT`, `CRAB_COLID_HURT`) all resolve to these values — only the
/// Rust type varies between families (`u16` vs `i16`), never the number. One
/// source of truth makes that consistency structural rather than accidental: a
/// future edit to one family can no longer silently diverge from the ROM.
///
/// `EMPTY` and `KIDBELOW` have no standard-family consumer (only the
/// deliberately-remapped `MINI_SKULL_COLID_GROUND_*` set uses `0x28`), hence
/// `#[allow(dead_code)]` on the module.
#[allow(dead_code)]
mod colid {
    /// ASM `colid_empty` (constants.asm:164).
    pub const EMPTY: u16 = 0x0000;
    /// ASM `colid_rightwall` (constants.asm:165).
    pub const RIGHTWALL: u16 = 0x0004;
    /// ASM `colid_leftwall` (constants.asm:166).
    pub const LEFTWALL: u16 = 0x0008;
    /// ASM `colid_floor` (constants.asm:167).
    pub const FLOOR: u16 = 0x000C;
    /// ASM `colid_ceiling` (constants.asm:168).
    pub const CEILING: u16 = 0x0010;
    /// ASM `colid_slopeup` (constants.asm:169).
    pub const SLOPEUP: u16 = 0x0014;
    /// ASM `colid_slopedown` (constants.asm:170).
    pub const SLOPEDOWN: u16 = 0x0018;
    /// ASM `colid_hurt` (constants.asm:171).
    pub const HURT: u16 = 0x001C;
    /// ASM `colid_kidright` (constants.asm:172).
    pub const KIDRIGHT: u16 = 0x0020;
    /// ASM `colid_kidleft` (constants.asm:173).
    pub const KIDLEFT: u16 = 0x0024;
    /// ASM `colid_kidbelow` (constants.asm:174).
    pub const KIDBELOW: u16 = 0x0028;
    /// ASM `colid_kidabove` (constants.asm:175).
    pub const KIDABOVE: u16 = 0x002C;
}

pub(crate) const NINJA_COLID_RIGHTWALL: u16 = colid::RIGHTWALL;
pub(crate) const NINJA_COLID_LEFTWALL: u16 = colid::LEFTWALL;
pub(crate) const NINJA_COLID_FLOOR: u16 = colid::FLOOR;
pub(crate) const NINJA_COLID_CEILING: u16 = colid::CEILING;
pub(crate) const NINJA_COLID_HURT: u16 = colid::HURT;
pub(crate) const NINJA_COLID_KIDABOVE: u16 = colid::KIDABOVE;
pub(crate) const NINJA_COLID_KIDRIGHT: u16 = colid::KIDRIGHT;
pub(crate) const NINJA_COLID_KIDLEFT: u16 = colid::KIDLEFT;
pub(crate) const DRAGON_COLID_HURT: u16 = colid::HURT;
pub(crate) const DRAGON_COLID_KIDRIGHT: u16 = colid::KIDRIGHT;
pub(crate) const DRAGON_COLID_KIDLEFT: u16 = colid::KIDLEFT;
pub(crate) const DRAGON_COLID_KIDABOVE: u16 = colid::KIDABOVE;
pub(crate) const TANK_COLID_HURT: u16 = colid::HURT;
const CRAB_COLID_RIGHTWALL: u16 = colid::RIGHTWALL;
const CRAB_COLID_LEFTWALL: u16 = colid::LEFTWALL;
const CRAB_COLID_SLOPEUP: u16 = colid::SLOPEUP;
const CRAB_COLID_SLOPEDOWN: u16 = colid::SLOPEDOWN;
pub(crate) const DIAMOND_COLID_RIGHTWALL: u16 = colid::RIGHTWALL;
pub(crate) const DIAMOND_COLID_LEFTWALL: u16 = colid::LEFTWALL;
pub(crate) const DIAMOND_COLID_FLOOR: u16 = colid::FLOOR;
pub(crate) const DIAMOND_COLID_CEILING: u16 = colid::CEILING;
pub(crate) const DIAMOND_COLID_SLOPEUP: u16 = colid::SLOPEUP;
pub(crate) const DIAMOND_COLID_SLOPEDOWN: u16 = colid::SLOPEDOWN;
pub(crate) const DIAMOND_COLID_HURT: u16 = colid::HURT;
pub(crate) const DIAMOND_COLID_KIDRIGHT: u16 = colid::KIDRIGHT;
pub(crate) const DIAMOND_COLID_KIDLEFT: u16 = colid::KIDLEFT;

pub(crate) fn tank_shooting_walk_speed_loc_3b5f2(current_hp: u8) -> i32 {
    // ASM Tank_Shooting.asm loc_3B5F2: HP > 3 preloads `$50(a5)` with
    // `$8000`, HP == 3 falls directly into this `$8000` add, and all paths
    // then receive the final `$8000` at loc_3B5FA. HP < 3 skips loc_3B5F2
    // and receives only the loc_3B5FA final add.
    match current_hp {
        0..=2 => 0x0000_8000,
        3 => 0x0001_0000,
        _ => 0x0001_8000,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CrabMotionProfileLoc3B00A {
    pub(crate) walk_speed: i32,
    pub(crate) stored_speed: i32,
    pub(crate) timer_4a: u16,
    pub(crate) timer_48: u16,
    pub(crate) animation_script: &'static str,
    pub(crate) increments_enemy_count: bool,
}

pub(crate) fn crab_motion_profile_loc_3b00a(current_hp: u8) -> CrabMotionProfileLoc3B00A {
    // ASM Crab.asm loc_3B00A: both speed slots receive one final `$8000` add
    // after loc_3AFD0's optional `$10000` preload, then `$4A/$48` and
    // `stru_3AF62` are installed before the shared walker loop.
    let speed = if current_hp >= 3 {
        0x0001_8000
    } else {
        0x0000_8000
    };
    CrabMotionProfileLoc3B00A {
        walk_speed: speed,
        stored_speed: speed,
        timer_4a: 0x001E,
        timer_48: 0x000C,
        animation_script: "stru_3AF62",
        increments_enemy_count: true,
    }
}

pub(crate) const ORCA_BERZERKER_KNOCKBACK_WAIT_TIMER_LOC_3D2AA: u16 = 0xD2AA;

pub(crate) const MINI_SKULL_COLID_RIGHTWALL: u16 = colid::RIGHTWALL;
pub(crate) const MINI_SKULL_COLID_LEFTWALL: u16 = colid::LEFTWALL;
pub(crate) const MINI_SKULL_COLID_FLOOR: u16 = colid::FLOOR;
pub(crate) const MINI_SKULL_COLID_CEILING: u16 = colid::CEILING;
pub(crate) const MINI_SKULL_COLID_SLOPEUP: u16 = colid::SLOPEUP;
pub(crate) const MINI_SKULL_COLID_SLOPEDOWN: u16 = colid::SLOPEDOWN;
pub(crate) const MINI_SKULL_COLID_HURT: u16 = colid::HURT;
pub(crate) const MINI_SKULL_COLID_GROUND_KIDABOVE: u16 = 0x001C;
pub(crate) const MINI_SKULL_COLID_GROUND_KIDLEFT: u16 = 0x0020;
pub(crate) const MINI_SKULL_COLID_GROUND_KIDRIGHT: u16 = 0x0024;
pub(crate) const MINI_SKULL_COLID_GROUND_KIDBELOW: u16 = 0x0028;
pub(crate) const MINI_SKULL_COLID_GROUND_HURT: u16 = 0x002C;

pub(crate) const MINI_HOPPING_SKULL_AIR_DAMAGE_WAIT_TIMER_LOC_3F200: u16 = 0xF200;
pub(crate) const MINI_HOPPING_SKULL_CHARGING_BERZERKER_WAIT_TIMER_LOC_3F288: u16 = 0xF288;

// Big Hopping Skull: continuous bouncer from Big_Hopping_Skull.asm.
// FP16.16 velocities: x_vel = ±$10000 (1px/frame), y_vel init = $FFFE0000 (-2px/frame).
// Gravity: $FA0/frame. Deflects off all surfaces with no ground delay.
// ASM Big_Hopping_Skull.asm loc_3651E: `move.w #$FFFE,y_vel(a3)`;
// `move.w #1,x_vel(a3)` seeds the launch as high-word FP16.16 velocities.
pub(crate) const BIG_SKULL_INIT_X_SPEED: i32 = 0x0001_0000;
pub(crate) const BIG_SKULL_INIT_Y_SPEED: i32 = -0x0002_0000;
/// ASM Big_Hopping_Skull.asm loc_365D8 initialises `stru_367CC`.
/// Durations: $18,$0C,6,$0C,6,$0C,6,$0C,6,$10,$0C.
pub(crate) const BIG_HOPPING_SKULL_HIT_ANIMATION_FRAMES_LOC_365D8: u8 = 0x7C;
const BIG_HOPPING_SKULL_HIT_GROAN_ELAPSED_FRAMES_LOC_365F2: [u8; 4] = [0x24, 0x36, 0x48, 0x5A];
const BIG_HOPPING_SKULL_HIT_VOICE_ELAPSED_FRAMES_LOC_36616: u8 = 0x70;

fn write_velocity_high_word(value: i32, high_word: i16) -> i32 {
    (i32::from(high_word) << 16) | (value & 0x0000_FFFF)
}

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
/// Shishkaboss per-head HP: loc_394E2/loc_39608/loc_3972E overwrite all heads with $14.
pub(crate) const SHISHKABOSS_HEAD_HP: [u8; 3] = [20, 20, 20];

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
pub(crate) fn pacer_probe_half_width(family: EnemyFamily) -> i32 {
    match family {
        // ASM Crab.asm loc_3B00A and Driller.asm loc_3BD74 seed `$48(a5)=$C`
        // before entering the shared walker loops.
        EnemyFamily::Crab | EnemyFamily::Driller => 12,
        // ASM Tank.asm loc_3B31E and Tank_Shooting.asm loc_3B5A6 seed
        // `$48(a5)=$B` before entering the same shared walker loops.
        EnemyFamily::Tank | EnemyFamily::TankShooting => 11,
        // ASM Armadillo.asm loc_3C766, Goat.asm loc_3CA7E, and Scorpion.asm
        // loc_3DBD8 seed `$48(a5)=9`.
        EnemyFamily::Armadillo | EnemyFamily::Goat | EnemyFamily::Scorpion => 9,
        _ => ENEMY_PROBE_HALF_WIDTH,
    }
}

pub(crate) fn pacer_probe_height(family: EnemyFamily) -> i32 {
    match family {
        // ASM Tank.asm loc_3B31E and Tank_Shooting.asm loc_3B5A6 seed
        // `$4A(a5)=$20` before entering the shared walker loops.
        EnemyFamily::Tank | EnemyFamily::TankShooting => 32,
        _ => ENEMY_PROBE_HEIGHT,
    }
}

/// Compute pacer walk speed from the enemy family and HP.
///
/// The ASM init routines all follow the same pattern: start at $8000 and
/// add $10000 for each HP tier above the family's minimum.
pub(crate) fn pacer_walk_speed(family: EnemyFamily, hit_points: u8) -> i32 {
    if matches!(family, EnemyFamily::Ninja) {
        return ninja_walk_speed_override_loc_3d59a_and_3d5aa(hit_points);
    }
    if matches!(family, EnemyFamily::TankShooting) {
        return tank_shooting_walk_speed_loc_3b5f2(hit_points);
    }
    if matches!(family, EnemyFamily::Driller) {
        return driller_walk_speed_loc_3bd74(hit_points);
    }

    let (base_hp, base_speed, speed_step) = match family {
        // Goat.asm loc_3CA6E: HP 4+ reaches the `$10000` speed-tier add.
        // Goat.asm loc_3CA7E: every tier then receives the final `$8000`.
        // HP = enemy_level + 3 (min 3). Base speed $8000, step $10000.
        // Player-tracking callback (Goat.asm loc_3CC18) wired in step dispatch.
        EnemyFamily::Goat => (3u8, 0x0000_8000i32, 0x0001_0000i32),
        // Armadillo.asm loc_3C756/loc_3C766: HP 1 gets only `$8000`, HP 2
        // reaches one `$10000` add, and HP above 2 reaches the two-add high
        // path before the final `$8000`, so speed caps at `$28000`.
        EnemyFamily::Armadillo => return armadillo_walk_speed_loc_3c756_3c766(hit_points),
        // Crab.asm loc_3B00A: HP 2 keeps `$8000`; HP 3+ uses the single
        // `$10000` preload plus one final `$8000` add, so speed caps at `$18000`.
        EnemyFamily::Crab => return crab_motion_profile_loc_3b00a(hit_points).walk_speed,
        // Scorpion: HP = enemy_level + 3 (min 3). Fixed speed $18000.
        EnemyFamily::Scorpion => (3, 0x0001_8000, 0),
        // Tank: HP = enemy_level + 2 (min 2). Base speed $8000, step $8000.
        // ASM Tank.asm loc_3B36A/loc_3B372: HP 2 gets one `$8000` add,
        // HP 3 gets two, HP 4+ gets three.
        EnemyFamily::Tank => (2, 0x0000_8000, 0x0000_8000),
        // Fire_Demon.asm loc_3A40C: HP 4 executes the second `$8000` speed add;
        // HP 5+ reaches it after the earlier high-HP preload.
        // Fire_Demon.asm loc_3A414: every tier then falls through to the final
        // `$8000` speed add; HP 3 reaches only this add.
        EnemyFamily::FireDemon => (3, 0x0000_8000, 0x0000_8000),
        // Robot: HP = enemy_level + 2 (min 2). Base speed $8000, step $8000.
        EnemyFamily::Robot => (2, 0x0000_8000, 0x0000_8000),
        // Orca: fixed 4 HP. Fixed speed $8000.
        EnemyFamily::Orca => (4, 0x0000_8000, 0),
        // Lion: HP = enemy_level + 6 (min 6). Binary speed check: $28000 at HP==8,
        // $18000 otherwise. ASM: Lion.asm lines 32-41, `cmpi.b #8`.
        // Using step=0 and handling HP==8 via lion_charge_speed instead.
        EnemyFamily::Lion => return lion_charge_speed(hit_points),
        EnemyFamily::DragonWalking => {
            let setup = dragon_walking_setup_loc_3cdfc(u16::from(hit_points));
            debug_assert_eq!(setup.speed_slot_50, setup.target_speed_slot_5e);
            debug_assert_eq!(setup.state_word_4a, 0x0010);
            debug_assert_eq!(setup.timer_48, 9);
            debug_assert_eq!(setup.patrol_timer_pair_6e, 0x0032_012C);
            debug_assert_eq!(
                setup.short_patrol_timer_54,
                DRAGON_WALKING_PATROL_SHORT_FRAMES
            );
            debug_assert_eq!(setup.callback_role, "dragon_breath_callback");
            debug_assert!(matches!(setup.animation_label, "stru_3CC5E" | "stru_3CC90"));
            return setup.speed_slot_50;
        }
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

pub(crate) fn driller_walk_speed_loc_3bd74(hit_points: u8) -> i32 {
    let mut speed = if hit_points > 3 { 0x0000_8000 } else { 0 };
    if hit_points >= 3 {
        // Driller.asm loc_3BD6C: HP 3+ reaches the middle `$8000` speed add.
        speed += 0x0000_8000;
    }
    // Driller.asm loc_3BD74: every HP tier falls through to one final `$8000`
    // add, so HP above 4 remains capped at the three-add `$18000` tier.
    speed + 0x0000_8000
}

pub(crate) fn armadillo_walk_speed_loc_3c756_3c766(hit_points: u8) -> i32 {
    let high_tier_adds = if hit_points > 2 {
        2
    } else {
        i32::from(hit_points.saturating_sub(1))
    };
    0x0000_8000 + high_tier_adds * 0x0001_0000
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArmadilloDamageResponseLoc3C848 {
    RecoilActiveResume {
        current_hp: u16,
    },
    Death,
    Hurt {
        remaining_hp: u16,
        damage_animation: &'static str,
        waits_until_animation_finished: bool,
        recoil_state_word: i16,
        restored_walk_speed: i32,
        restored_animation: &'static str,
    },
}

pub(crate) fn armadillo_damage_response_loc_3c848(
    current_hp: u16,
    recoil_state_word: i16,
    saved_walk_speed: i32,
) -> ArmadilloDamageResponseLoc3C848 {
    // ASM Armadillo.asm loc_3C848: positive `$42(a3)` skips damage entirely.
    // Otherwise current_hp is decremented; zero goes to death, and survivors
    // play `stru_3C6D0`, seed `$42(a3)` with `$FF88`, restore walk speed from
    // `$5E(a5)`, and resume the saved patrol animation from `$62(a5)`.
    if recoil_state_word > 0 {
        return ArmadilloDamageResponseLoc3C848::RecoilActiveResume { current_hp };
    }

    let remaining_hp = current_hp.wrapping_sub(1);
    if remaining_hp == 0 {
        ArmadilloDamageResponseLoc3C848::Death
    } else {
        ArmadilloDamageResponseLoc3C848::Hurt {
            remaining_hp,
            damage_animation: "stru_3C6D0",
            waits_until_animation_finished: true,
            recoil_state_word: -0x0078,
            restored_walk_speed: saved_walk_speed,
            restored_animation: "saved_armadillo_patrol_animation",
        }
    }
}

pub(crate) const ARMADILLO_BERZERKER_LAUNCH_WAIT_TIMER_LOC_3C8CC: u16 = 0xC8CC;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoatDamageResponseLoc3CB38 {
    Death,
    Hurt {
        remaining_hp: u16,
        damage_animation: &'static str,
        recoil_state_word: i16,
        restored_walk_speed: i32,
        restored_animation: &'static str,
    },
}

pub(crate) fn goat_damage_response_loc_3cb38(
    current_hp: u16,
    saved_walk_speed: i32,
) -> GoatDamageResponseLoc3CB38 {
    // ASM Goat.asm loc_3CB38: `subq.w #1,$44(a3)` kills on zero; otherwise
    // play `stru_3C9E4`, set `$42(a3)` to `$FF88`, restore `$50(a5)` from
    // saved `$5E(a5)`, then resume the saved patrol animation.
    let remaining_hp = current_hp.wrapping_sub(1);
    if remaining_hp == 0 {
        GoatDamageResponseLoc3CB38::Death
    } else {
        GoatDamageResponseLoc3CB38::Hurt {
            remaining_hp,
            damage_animation: "stru_3C9E4",
            recoil_state_word: -0x0078,
            restored_walk_speed: saved_walk_speed,
            restored_animation: "saved_goat_patrol_animation",
        }
    }
}

pub(crate) const GOAT_BERZERKER_KNOCKBACK_GRAVITY_LOC_3CBA8: i32 = 0x0000_4000;
pub(crate) const GOAT_BERZERKER_KNOCKBACK_WAIT_TIMER_LOC_3CBA8: u16 = 0xCBA8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoatBerzerkerAirborneFrameLoc3CBA8 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) applies_motion_subroutine: bool,
    pub(crate) motion_subroutine: &'static str,
    pub(crate) next_y_velocity: i32,
    pub(crate) repeats_airborne_loop: bool,
}

pub(crate) fn goat_berzerker_airborne_frame_loc_3cba8(
    y_velocity: i32,
) -> GoatBerzerkerAirborneFrameLoc3CBA8 {
    // ASM Goat.asm loc_3CBA8: hibernate one frame, call `sub_3E8BA` with
    // the current velocities, add `$4000` to `y_vel(a3)`, then branch back
    // to repeat the airborne knockback loop.
    GoatBerzerkerAirborneFrameLoc3CBA8 {
        hibernates_one_frame: true,
        applies_motion_subroutine: true,
        motion_subroutine: "sub_3E8BA",
        next_y_velocity: y_velocity.wrapping_add(GOAT_BERZERKER_KNOCKBACK_GRAVITY_LOC_3CBA8),
        repeats_airborne_loop: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoatTrackingRightRouteLoc3CC36 {
    TurnRight,
    AlreadyFacingRight,
}

pub(crate) fn goat_tracking_right_route_loc_3cc36(
    goat_x_px: i16,
    kid_x_px: i16,
    facing: Direction,
) -> Option<GoatTrackingRightRouteLoc3CC36> {
    let d7 = goat_x_px.wrapping_sub(kid_x_px);
    if d7 >= -0x20 {
        return None;
    }

    // ASM Goat.asm loc_3CC36: negative `d7` means Kid is more than 32px to
    // the goat's right; clear `x_direction` branches to the shared turn-right
    // path, while set `x_direction` already faces Kid.
    Some(match facing {
        Direction::Left => GoatTrackingRightRouteLoc3CC36::TurnRight,
        Direction::Right => GoatTrackingRightRouteLoc3CC36::AlreadyFacingRight,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoatTrackingLeftRouteLoc3CC46 {
    TurnLeft,
    AlreadyFacingLeft,
}

pub(crate) fn goat_tracking_left_route_loc_3cc46(
    goat_x_px: i16,
    kid_x_px: i16,
    facing: Direction,
) -> Option<GoatTrackingLeftRouteLoc3CC46> {
    let d7 = goat_x_px.wrapping_sub(kid_x_px);
    if d7 <= 0x20 {
        return None;
    }

    // ASM Goat.asm loc_3CC46: positive `d7` means Kid is more than 32px to
    // the goat's left; set `x_direction` branches to the shared turn-left
    // path, while clear `x_direction` already faces Kid.
    Some(match facing {
        Direction::Left => GoatTrackingLeftRouteLoc3CC46::AlreadyFacingLeft,
        Direction::Right => GoatTrackingLeftRouteLoc3CC46::TurnLeft,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoatTrackingResumeRouteLoc3CC52 {
    Loc3E566,
    Loc3E3B2,
}

pub(crate) fn goat_tracking_resume_route_loc_3cc52(
    facing: Direction,
) -> GoatTrackingResumeRouteLoc3CC52 {
    // ASM Goat.asm loc_3CC52: `tst.b x_direction(a3); beq.w loc_3E566;
    // bra.w loc_3E3B2`. This is reached both when `$42(a3)` is positive
    // and when Kid is inside the inclusive +/-32px tracking dead zone.
    match facing {
        Direction::Left => GoatTrackingResumeRouteLoc3CC52::Loc3E566,
        Direction::Right => GoatTrackingResumeRouteLoc3CC52::Loc3E3B2,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoatTrackingRouteLoc3CC18 {
    Resume(GoatTrackingResumeRouteLoc3CC52),
    Right(GoatTrackingRightRouteLoc3CC36),
    Left(GoatTrackingLeftRouteLoc3CC46),
}

pub(crate) fn goat_tracking_route_loc_3cc18(
    shared_walker_state: i16,
    goat_x_px: i16,
    kid_x_px: i16,
    facing: Direction,
) -> GoatTrackingRouteLoc3CC18 {
    // ASM Goat.asm loc_3CC18: positive `$42(a3)` skips the Kid-distance
    // checks and resumes immediately through loc_3CC52.
    if shared_walker_state > 0 {
        return GoatTrackingRouteLoc3CC18::Resume(goat_tracking_resume_route_loc_3cc52(facing));
    }

    if let Some(route) = goat_tracking_right_route_loc_3cc36(goat_x_px, kid_x_px, facing) {
        return GoatTrackingRouteLoc3CC18::Right(route);
    }

    if let Some(route) = goat_tracking_left_route_loc_3cc46(goat_x_px, kid_x_px, facing) {
        return GoatTrackingRouteLoc3CC18::Left(route);
    }

    GoatTrackingRouteLoc3CC18::Resume(goat_tracking_resume_route_loc_3cc52(facing))
}

pub(crate) const DIAMOND_HIGH_HP_BERZERKER_LAUNCH_WAIT_TIMER_LOC_3AEB2: u16 = 0xAEB2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiamondLowHpCollisionRouteLoc3AEC4 {
    ResumeSharedWalker { next_collision_type: u16 },
    BerzerkerSideGate,
    HurtAnimation { remaining_hp: u16 },
    DeathAnimation,
}

pub(crate) fn diamond_low_hp_collision_dispatch_loc_3aec4(
    collision_type: i16,
    current_hp: u16,
) -> DiamondLowHpCollisionRouteLoc3AEC4 {
    // ASM Diamond.asm loc_3AEC4: read `collision_type(a3)` without a
    // pre-clear. Zero and unknown positives resume the shared walker tail,
    // signed-negative words delete the Diamond, side contacts enter the
    // Berzerker gate, and `colid_hurt` decrements `$44(a3)` before routing.
    if collision_type == 0 {
        return DiamondLowHpCollisionRouteLoc3AEC4::ResumeSharedWalker {
            next_collision_type: 0,
        };
    }
    if collision_type.is_negative() {
        return DiamondLowHpCollisionRouteLoc3AEC4::DeathAnimation;
    }

    match collision_type {
        v if v == DIAMOND_COLID_KIDRIGHT as i16 || v == DIAMOND_COLID_KIDLEFT as i16 => {
            DiamondLowHpCollisionRouteLoc3AEC4::BerzerkerSideGate
        }
        v if v == DIAMOND_COLID_HURT as i16 => {
            let remaining_hp = current_hp.wrapping_sub(1);
            if remaining_hp == 0 {
                DiamondLowHpCollisionRouteLoc3AEC4::DeathAnimation
            } else {
                DiamondLowHpCollisionRouteLoc3AEC4::HurtAnimation { remaining_hp }
            }
        }
        _ => DiamondLowHpCollisionRouteLoc3AEC4::ResumeSharedWalker {
            next_collision_type: 0,
        },
    }
}

pub(crate) const DIAMOND_LOW_HP_BERZERKER_LAUNCH_WAIT_TIMER_LOC_3AF50: u16 = 0xAF50;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FireDemonFireTrailGateLoc3A542 {
    pub(crate) spawns_fire_trail: bool,
    pub(crate) next_fire_trail_count: u16,
}

pub(crate) fn fire_demon_level2_fire_trail_timer_matches_loc_3a55c(animation_timer: u16) -> bool {
    // ASM Fire_Demon.asm loc_3A55C: raw enemy_level 2 reaches allocation only
    // when `animation_timer(a3)` equals 3; every other timer falls through to
    // loc_3A586 collision handling without spawning a trail.
    animation_timer == 3
}

pub(crate) fn fire_demon_fire_trail_allocation_loc_3a564(
    fire_trail_count: u16,
) -> FireDemonFireTrailGateLoc3A542 {
    // ASM Fire_Demon.asm loc_3A564: `Number_of_Fire_Trails >= 7` skips the
    // allocation; otherwise the ROM allocates a `loc_3A63C` trail object and
    // increments the global trail count.
    let spawns_fire_trail = fire_trail_count < 7;
    FireDemonFireTrailGateLoc3A542 {
        spawns_fire_trail,
        next_fire_trail_count: if spawns_fire_trail {
            fire_trail_count + 1
        } else {
            fire_trail_count
        },
    }
}

pub(crate) fn fire_demon_fire_trail_gate_loc_3a542(
    sprite_is_trail_frame: bool,
    raw_enemy_level: u16,
    animation_timer: u16,
    fire_trail_count: u16,
) -> FireDemonFireTrailGateLoc3A542 {
    // ASM Fire_Demon.asm loc_3A542: only the `LnkTo_unk_C771E` sprite frame can
    // reach the allocation block. Raw enemy_level 2 uses animation_timer 3; all
    // other levels use timer 6. Allocation runs while Number_of_Fire_Trails < 7.
    let timer_matches = if raw_enemy_level == 2 {
        fire_demon_level2_fire_trail_timer_matches_loc_3a55c(animation_timer)
    } else {
        animation_timer == 6
    };
    if sprite_is_trail_frame && timer_matches {
        fire_demon_fire_trail_allocation_loc_3a564(fire_trail_count)
    } else {
        FireDemonFireTrailGateLoc3A542 {
            spawns_fire_trail: false,
            next_fire_trail_count: fire_trail_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FireDemonWalkingTrailStepLoc3A542 {
    pub(crate) spawns_fire_trail: bool,
    pub(crate) next_fire_trail_count: u16,
    pub(crate) next_animation_phase: u8,
}

pub(crate) fn fire_demon_walking_trail_step_loc_3a542(
    raw_enemy_level: u16,
    animation_phase: u8,
    fire_trail_count: u16,
) -> FireDemonWalkingTrailStepLoc3A542 {
    // ASM Fire_Demon.asm loc_3A542 runs while the saved walking script from
    // `$62(a5)` is live. `stru_3A31A` uses six-frame entries; raw level 2
    // saved `stru_3A334`, whose entries last three frames. Only the first
    // timer tick of the first `LnkTo_unk_C771E` frame reaches allocation.
    let frame_duration = if raw_enemy_level == 2 { 3 } else { 6 };
    let loop_frames = frame_duration * 6;
    let phase = u16::from(animation_phase) % loop_frames;
    let sprite_is_trail_frame = phase < frame_duration;
    let animation_timer = frame_duration - (phase % frame_duration);
    let gate = fire_demon_fire_trail_gate_loc_3a542(
        sprite_is_trail_frame,
        raw_enemy_level,
        animation_timer,
        fire_trail_count,
    );

    FireDemonWalkingTrailStepLoc3A542 {
        spawns_fire_trail: gate.spawns_fire_trail,
        next_fire_trail_count: gate.next_fire_trail_count,
        next_animation_phase: ((phase + 1) % loop_frames) as u8,
    }
}

pub(crate) const FIRE_DEMON_BERZERKER_LAUNCH_GRAVITY_LOC_3A610: i32 = 0x0000_4000;
pub(crate) const FIRE_DEMON_BERZERKER_LAUNCH_WAIT_TIMER_LOC_3A610: u16 = 0xA610;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FireDemonBerzerkerLaunchLoopTickLoc3A610 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shared_retention_check_sub_3c3ce: bool,
    pub(crate) next_y_velocity: i32,
    pub(crate) loops: bool,
}

pub(crate) fn fire_demon_berzerker_launch_loop_tick_loc_3a610(
    y_velocity: i32,
) -> FireDemonBerzerkerLaunchLoopTickLoc3A610 {
    // ASM Fire_Demon.asm loc_3A610: each airborne tick hibernates one frame,
    // runs `sub_3C3CE`, adds `$4000` to `y_vel(a3)`, then branches back to
    // loc_3A610.
    FireDemonBerzerkerLaunchLoopTickLoc3A610 {
        hibernates_one_frame: true,
        runs_shared_retention_check_sub_3c3ce: true,
        next_y_velocity: y_velocity.wrapping_add(FIRE_DEMON_BERZERKER_LAUNCH_GRAVITY_LOC_3A610),
        loops: true,
    }
}

/// Tank bullet speed in FP16.16 (3 px/frame → $30000).
pub(crate) const TANK_BULLET_SPEED: i32 = 0x3_0000;
/// Tank bullets carry a sentinel lifetime because their native lifecycle is
/// handled by ASM Tank_Shooting.asm loc_3B7BA's camera-relative bounds loop.
pub(crate) const TANK_BULLET_LIFETIME: u16 = u16::MAX;
/// ASM Tank_Shooting.asm `stru_3B4D8`: one shot every 116 frames at HP 4.
pub(crate) const TANK_SHOOTING_FAST_CYCLE_FRAMES: u16 = 116;
/// ASM Tank_Shooting.asm loc_3B682 / `stru_3B4F6`: one shot every 232
/// frames when the turret setup value is not exactly 4.
pub(crate) const TANK_SHOOTING_SLOW_CYCLE_FRAMES: u16 = 232;
/// ASM GfxObject_Animate + `stru_3B4D8`/`stru_3B4F6`: `loc_3C50E` fires one
/// frame after the turret enters `LnkTo_unk_C7F10`, which is 74 frames after init.
pub(crate) const TANK_SHOOTING_FIRST_SHOT_DELAY_FRAMES: u16 = 74;
/// ASM Tank_Shooting.asm loc_3B5FA: after allocating the turret child,
/// `move.w #$49,$54(a5)` seeds the first shot countdown.
pub(crate) const TANK_SHOOTING_INIT_FIRE_TIMER_LOC_3B5FA: u16 =
    TANK_SHOOTING_FIRST_SHOT_DELAY_FRAMES - 1;
/// ASM Tank_Shooting.asm loc_3B700: nonfatal turret hits hibernate the parent
/// for five frames before falling through to loc_3B720.
pub(crate) const TANK_SHOOTING_HURT_HIBERNATE_FRAMES: u8 = 5;
/// ASM Tank.asm loc_3B49A: nonfatal mouth-child hits use the same five-frame
/// hibernate window before loc_3B4BA resumes the shared walker.
pub(crate) const TANK_HURT_HIBERNATE_FRAMES: u8 = 5;
/// ASM Tank.asm loc_3B49A seeds the mouth-child recoil before hibernating.
/// Native trace snapshots include the surrounding collision/sample frames; the
/// Issue #213 metal-block contact route keeps the parent parked through the
/// exported recoil frames after the raw `j_Hibernate_Object(5)` window.
pub(crate) const TANK_HURT_TRACE_HIBERNATE_FRAMES_LOC_3B49A: u8 = TANK_HURT_HIBERNATE_FRAMES + 9;
/// ASM Tank.asm loc_3B49A clears shared walker motion `$50(a5)` and
/// loc_3B4BA resumes through loc_3C0FA without restoring it.
pub(crate) const TANK_STOPPED_WALK_SPEED_OVERRIDE_LOC_3B49A: i32 = i32::MIN;
/// ASM Tank.asm loc_3B49A writes child `$46=6` before `j_Hibernate_Object(5)`.
/// The object-list trace samples the linked GfxObject across the collision and
/// hibernate boundary, so the visible `stru_3B294` recoil starts after the
/// oracle-confirmed pre-recoil frame and then persists because loc_3B4BA never
/// restores `$46`.
const TANK_CHILD_RECOIL_VISIBLE_FRAMES_LOC_3B49A: u8 = TANK_HURT_HIBERNATE_FRAMES + 3;
/// ASM Tank_Shooting.asm loc_3B700 mirrors the same linked-child recoil timing
/// through `stru_3B294`.
const TANK_SHOOTING_CHILD_RECOIL_VISIBLE_FRAMES_LOC_3B700: u8 =
    TANK_SHOOTING_HURT_HIBERNATE_FRAMES.saturating_sub(2);
/// ASM Tank_Shooting.asm loc_3B75A/loc_3B77E: spawn at ±15px from tank centre.
pub(crate) const TANK_BULLET_SPAWN_X_OFFSET_PX: i32 = 15;
/// ASM Tank_Shooting.asm loc_3B786/loc_3B78E: projectile Y = turret_y - (8 + 6) px.
pub(crate) const TANK_BULLET_SPAWN_Y_OFFSET_PX: i32 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScorpionTailGfxPosition {
    pub(crate) x_word: u16,
    pub(crate) y_word: u16,
}

pub(crate) fn scorpion_tail_sync_loc_3dc86(
    parent_x_pos: u32,
    parent_y_pos: u32,
    parent_facing: Direction,
) -> ScorpionTailGfxPosition {
    // ASM Scorpion.asm loc_3DC86 copies the parent integer X/Y words to the
    // linked tail gfx object, lifts it by $16 px, then applies a facing offset.
    let parent_x_word = (parent_x_pos >> 16) as u16;
    let parent_y_word = (parent_y_pos >> 16) as u16;
    let x_offset = match parent_facing {
        Direction::Left => 1_u16,
        // ASM Scorpion.asm loc_3DCAA: when `x_direction` is set, subtract $16
        // from the linked tail object's integer X word.
        Direction::Right => 0x16_u16,
    };

    ScorpionTailGfxPosition {
        x_word: parent_x_word.wrapping_sub(x_offset),
        y_word: parent_y_word.wrapping_sub(0x16),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScorpionDamageRouteLoc3DCD4 {
    Hurt {
        remaining_hp: u16,
        damage_animation: &'static str,
        restored_animation: &'static str,
    },
    Death,
}

pub(crate) fn scorpion_damage_route_loc_3dcd4(current_hp: u16) -> ScorpionDamageRouteLoc3DCD4 {
    // ASM Scorpion.asm loc_3DCD4: `subq.w #1,$44(a3); beq.s ...` branches to
    // death when the decremented current_hp word reaches zero; otherwise it
    // runs the hurt animation to completion and restores the base animation.
    let remaining_hp = current_hp.wrapping_sub(1);
    if remaining_hp == 0 {
        ScorpionDamageRouteLoc3DCD4::Death
    } else {
        ScorpionDamageRouteLoc3DCD4::Hurt {
            remaining_hp,
            damage_animation: "stru_3DB7A",
            restored_animation: "stru_3DB60",
        }
    }
}

pub(crate) const SCORPION_BERZERKER_KNOCKBACK_WAIT_TIMER_LOC_3DD4A: u16 = 0x3DD4;

pub(crate) const ARCHER_SHOOTING_CYCLE_DELAY_FRAMES_LOC_341BE: u16 =
    archer_shooting_cycle_delay_frames_loc_341BE();
#[allow(non_snake_case)]
pub(crate) const fn archer_shooting_cycle_delay_frames_loc_341BE() -> u16 {
    // ASM Archer.asm loc_341BE walks three delay words before the zero fire
    // sentinel. Each delay enters loc_34248 and is consumed by `dbf d3`, so
    // the hibernated wait is `delay + 1` frames for `$64`, `$06`, and `$0C`.
    (0x0064 + 1) + (0x0006 + 1) + (0x000C + 1)
}

fn fp16_integer_word(position: u32) -> u16 {
    u16::try_from(position >> 16).expect("FP16.16 integer word fits in u16")
}

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
    /// Terrain response after the descent hits the dragon's right side. ASM loc_34A00.
    TerrainSlideRight,
    /// Terrain response after the descent hits the dragon's left side. ASM loc_34A24.
    TerrainSlideLeft,
    /// Terrain response after the descent hits the floor. ASM loc_34A48.
    TerrainSlideUp,
    /// Terrain response after the descent hits the ceiling. ASM loc_34A68.
    TerrainSlideDown,
    /// Slope-up rebound path after snap-back. ASM loc_34A88.
    TerrainReboundLeft,
    /// Slope-down rebound path after snap-back. ASM loc_34ADC.
    TerrainReboundRight,
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

/// Ninja stealth/guard/attack state machine (Ninja.asm).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum NinjaState {
    /// Walking invisibly, checking detection cone each frame.
    #[default]
    Stealth,
    /// ASM Ninja.asm loc_3D826: Red Stealth / projectile guard pose.
    Guard,
    /// ASM Ninja.asm loc_3D84E: positive `$4C(a5)` jump-threat pose.
    JumpThreat,
    /// Airborne jump-attack toward the player.
    JumpAttack,
    /// ASM Ninja.asm loc_3DABC: HP-depleted Kid-above contact falls until floor before death.
    KidAboveKnockdown,
    /// ASM Ninja.asm loc_3D8E6: HP-depleted airborne knockdown falls until floor before death.
    AirborneKnockdown,
}

/// Orca lunge state machine (Orca.asm).
/// Paces normally, detects nearby player, launches a ballistic lunge arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum OrcaState {
    /// Walking patrol with proximity detection.
    #[default]
    Walking,
    /// ASM Orca.asm loc_3D320: six hibernated frames before the lunge launch.
    Windup,
    /// Airborne ballistic lunge toward the player.
    Lunging,
    /// ASM Orca.asm loc_3D420 -> loc_3D44A: grounded wait before patrol resumes.
    LandingWait,
    /// ASM Orca.asm loc_3D48A/loc_3D496: HP-depleted lunge falls until floor/slope before death.
    FatalFall,
}

/// Robot phase state machine (Robot.asm).
/// Paces at HP-dependent speed and periodically shoots; charge is entered by
/// collision/hit routing at loc_3A96C, which is still tracked by backlog #190.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum RobotState {
    /// Walking patrol with periodic shooting.
    #[default]
    Walking,
    /// ASM sub_3C4F8 loc_3C58E: head turn sequence before Enemy03_Robot_Shoot.
    Aiming,
    /// Charging at high speed after loc_3A96C collision/hit routing.
    Charging,
}

/// Lion charge state machine (Lion.asm).
/// Paces normally, then charges toward player and fires a fast projectile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum LionState {
    /// Walking patrol, checking for player in range.
    #[default]
    Walking,
    /// Roar wind-up before the projectile fire sprite.
    Roaring,
    /// Post-projectile roar animation before the charge resumes.
    PostFire,
    /// Charge branch resumed from the post-fire hibernate loop.
    ResumingCharge,
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
    /// ASM Spinning_Twins.asm loc_3B9B0 falling transition visual worker.
    FallingVisual,
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
    /// True after a known final-hit path has entered a family death animation.
    /// `hit_points == 0` also appears in live native/oracle traces for some
    /// spawned walkers, so scheduler deletion must key off this explicit state.
    pub(crate) death_animation_active: bool,
    pub(crate) grounded_frames: u8,
    /// Tank.asm `stru_3B27E` linked mouth-child animation phase.
    pub(crate) tank_mouth_animation_phase: u8,
    pub(crate) damage_flash_frames: u8,
    /// Palette variant (0, 1, or 2) from the enemy type slot.
    pub(crate) palette_variant: u8,
    /// Generic countdown timer for phased enemies (EmoRock hover wait, etc.)
    pub(crate) wait_timer: u16,
    pub(crate) dragon_phase: DragonFlyingState,
    pub(crate) dragon_phase_frames: u16,
    /// Dragon Flying: script variant (0, 1, 2) from layout skin byte.
    /// Dragon Walking reuses bit 7 for the ASM Dragon.asm loc_3CD16 `$5C(a5)`
    /// parent-spawned-child flag; no layout walking-dragon variant uses it.
    pub(crate) dragon_script_variant: u8,
    /// Dragon Flying: current command index within the script.
    /// Tornado reuses this as the next `loc_359F2` pattern index from `unk_35A0C`.
    /// Lion reuses this as the live `stru_3DEDA`/`stru_3DEF4` animation phase.
    pub(crate) dragon_script_cmd: u8,
    /// Dragon Flying: anchor Y position (FP16.16) — initial spawn Y to return to.
    /// Big Hopping Skull reuses this as packed `loc_364A0` patrol bounds because
    /// that family never reads the Dragon Flying anchor state.
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
    /// Fire Demon reuses this native-only flag for the two scheduler frames
    /// around ASM Fire_Demon.asm Enemy00_FireDemon_Init's leading
    /// `j_Hibernate_Object_1Frame`: hidden init, then visible-but-not-stepped.
    pub(crate) wall_pinned: bool,
    /// Drips: set once the ceiling-attach phase (ASM `Drips.asm loc_333F6`)
    /// converges. Until attached, the slime rises with `y_velocity = -$1_0000`
    /// and probes the ceiling each frame; once a solid cell is found above,
    /// the position snaps and this flag flips true, after which the existing
    /// wait/drop cycle runs.
    pub(crate) drips_attached: bool,
    /// Drips.asm loc_33488 advances through `off_33520` after each splash:
    /// entries 0 and 1 are pool-growth sprites; entry 2 is the negative
    /// sentinel that resets via loc_33506 and may allocate loc_335D8.
    pub(crate) drips_surface_table_index: u8,
    /// Drips.asm loc_33488 hibernates on `stru_33332` before copying the
    /// falling child's adjusted position into the surface child and advancing
    /// `off_33520`. Native stores that adjusted child position until the
    /// projectile shell consumes the splash state.
    pub(crate) drips_pending_splash_x_pos: u32,
    pub(crate) drips_pending_splash_y_pos: u32,
    pub(crate) tar_monster_state: TarMonsterState,
    pub(crate) spinning_twins_phase: SpinningTwinsPhase,
    pub(crate) spinning_twins_orbit_index: u8,
    /// UFO beam: segments remaining to spawn (0=not firing). ASM $48(a5).
    pub(crate) ufo_beam_segments: u8,
    /// UFO beam: frames until next segment. ASM $4A(a5).
    pub(crate) ufo_beam_interval: u8,
    /// Boss head index (0, 1, 2) for multi-head bosses. BOSS_HEAD_NONE if not a boss head.
    pub(crate) boss_head_index: u8,
    /// ASM `loc_3893E`/`loc_389D6` waiting boss-eye workers attached to this
    /// boss head before `$FFFFFB72/$FB74/$FB76` launches them.
    pub(crate) boss_eye_orbit_workers: BossEyeOrbitWorkers,
    /// ASM `sub_39846` `d0`: per-head movement-script countdown.
    /// HeadyMetal reuses this boss-only slot for the `$FFFFFB7A` anchor's
    /// `$3E(a3)` spawn-animation counter from `loc_375DE`.
    pub(crate) boss_movement_delay_counter: u16,
    /// ASM `sub_39846` `a2`: next fixed movement-script entry index.
    /// HeadyMetal reuses this boss-only slot for the `$FFFFFB4F` active
    /// eye-pop/thrown-head worker count that freezes `loc_37582` tracking.
    pub(crate) boss_movement_script_index: usize,
    /// HeadyMetal ASM `sub_379E8` `d2`: five-bit palette-wave phase.
    pub(crate) heady_metal_palette_phase_counter: u16,
    /// HeadyMetal ASM `sub_379E8`: word written to `Palette_Buffer+$5E`.
    pub(crate) heady_metal_palette_word_5e: u16,
    /// Tracking-lock flag at `$4E(a5)` for families that use this work slot.
    /// Scorpion suppresses re-facing while locked; Lion uses it as the
    /// loc_3E054 wider-range latch after a charge exit.
    pub(crate) scorpion_tracking_lock: bool,
    /// Scorpion.asm loc_3DDEE: frames remaining in `stru_3DB8E` while the
    /// linked tail object has Kid collision enabled.
    pub(crate) scorpion_sting_frames: u8,
    /// ASM UFO.asm `$4E(a5)`; for Tornado, `$46(a5)` gates loc_3592E X-term doubling.
    pub(crate) flight_double_acceleration: bool,
    /// ASM UFO.asm `$4F(a5)`; for Tornado, `$47(a5)` gates loc_3593A Y-term doubling.
    pub(crate) flight_double_terminal_speed: bool,
    /// Walk speed override from tracking callback ($50(a5)). When non-zero, step_pacer
    /// uses this instead of pacer_walk_speed(). Set by Scorpion tracking (Scorpion.asm loc_3DD7E).
    pub(crate) walk_speed_override: i32,
    /// Crab: frozen state flag ($6E(a5)). ASM loc_3B104 sets this when player
    /// within 64px and facing the crab; loc_3B170 clears it when distance >224px.
    pub(crate) crab_frozen: bool,
    /// Crab: charging-Berzerker side-contact loop from `Crab.asm loc_3B180`.
    pub(crate) crab_berzerker_knockback: bool,
    /// Shared Dragon/Lion grounded-state word ($42(a3)).
    pub(crate) shared_walker_state: i16,
    /// Shared Dragon/Lion patrol cadence countdown; Ninja reuses this for
    /// its attack cooldown `$54(a5)`.
    pub(crate) shared_walker_patrol_timer: u16,
    /// Shared walker stuck-turnaround byte `$5A(a5)`. `loc_3C026/loc_3C0D2`
    /// add 4 on each forced turnaround and route to `loc_3C464` on overflow.
    pub(crate) shared_walker_turnaround_counter: u8,
    /// Native phase debt for right-facing shared walkers that start one pixel
    /// behind the ROM hibernate pose. False until the first loc_3C04E catch-up
    /// turn has consumed the extra half-step.
    pub(crate) shared_walker_right_turn_extra_step_consumed: bool,
    /// Dragon alternates the next patrol cadence reload between $32 and $12C via $6E(a5).
    /// Ninja reuses the same two-period phase for `$6E(a5)` attack cadence. Driller and
    /// Rock Tanks reuse the native field as a visible-init drive-refcount latch.
    pub(crate) shared_walker_patrol_short_next: bool,
    /// Frontier-only trace label for the last shared-walker branch taken.
    pub(crate) shared_walker_debug_branch: Option<&'static str>,
    /// Signed horizontal distance to Kid when the grounded shared walker last evaluated him.
    pub(crate) shared_walker_dx_to_kid_px: Option<i16>,
    /// Frontier-only trace of the last terrain probes used by the shared walker.
    pub(crate) shared_walker_terrain_trace: SharedWalkerTerrainTrace,
}

pub(crate) const DRAGON_WALKING_PARENT_SPAWNED_CHILD_FLAG: u8 = 0x80;
pub(crate) const ARMADILLO_PARENT_SPAWNED_CHILD_FLAG: u8 = 0x80;
pub(crate) const ARMADILLO_SPAWNED_CHILD_FALL_WAIT_TIMER_LOC_3C954: u16 = 0xC954;

/// Native carrier for the two hibernating boss-eye workers owned by a
/// multi-head boss head. ASM `loc_3893E`/`loc_389D6` store the previous parent
/// x-direction byte in `d0`, rerun `sub_393C8`/`sub_39420` every frame, and
/// let those helpers bump `$A(a3)` when parent `$16` changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct BossEyeOrbitWorkers {
    pub(crate) active: bool,
    pub(crate) parent_phase_word_22: u16,
    pub(crate) current_parent_x_direction_byte_16: u8,
    pub(crate) phase_index: u8,
    pub(crate) previous_parent_x_direction_byte_16: u8,
    pub(crate) primary_sprite_order_word_a: u16,
    pub(crate) counter_sprite_order_word_a: u16,
    pub(crate) primary_x_pos_word: u16,
    pub(crate) primary_y_pos_word: u16,
    pub(crate) counter_x_pos_word: u16,
    pub(crate) counter_y_pos_word: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossEyeObjectSlotLoc3931a {
    pub(crate) object_meta: u16,
    pub(crate) priority: u8,
    pub(crate) has_kid_collision: bool,
    pub(crate) hidden_status_byte_13: bool,
    pub(crate) is_moved: bool,
    pub(crate) x_pos_word: u16,
    pub(crate) y_pos_word: u16,
    pub(crate) sprite_order_word_a: u16,
}

impl BossEyeOrbitWorkers {
    pub(crate) fn active_loc_38902_3899a(
        parent_x: u16,
        parent_y: u16,
        phase_index: u8,
        facing: Direction,
    ) -> Self {
        let parent_x_direction_byte_16 = boss_head_x_direction_byte_16(facing);
        let mut workers = Self {
            active: true,
            parent_phase_word_22: 0x0EE0 + (u16::from(phase_index % 5) * 2),
            current_parent_x_direction_byte_16: parent_x_direction_byte_16,
            phase_index: phase_index % 5,
            previous_parent_x_direction_byte_16: parent_x_direction_byte_16,
            ..Self::default()
        };
        workers.refresh_loc_3893e_389d6(parent_x, parent_y, facing);
        workers
    }

    pub(crate) fn object_slots_loc_3931a(&self) -> Vec<BossEyeObjectSlotLoc3931a> {
        if !self.active {
            return Vec::new();
        }

        // ASM loc_3931A: priority=0, object_meta=$21, clear
        // has_kid_collision, set `$13(a3)`, set is_moved, then hibernate and
        // keep refreshing the worker position/mapping via sub_39420.
        [
            (
                self.primary_x_pos_word,
                self.primary_y_pos_word,
                self.primary_sprite_order_word_a,
            ),
            (
                self.counter_x_pos_word,
                self.counter_y_pos_word,
                self.counter_sprite_order_word_a,
            ),
        ]
        .into_iter()
        .map(
            |(x_pos_word, y_pos_word, sprite_order_word_a)| BossEyeObjectSlotLoc3931a {
                object_meta: 0x21,
                priority: 0,
                has_kid_collision: false,
                hidden_status_byte_13: true,
                is_moved: true,
                x_pos_word,
                y_pos_word,
                sprite_order_word_a,
            },
        )
        .collect()
    }

    pub(crate) fn step_with_parent_animation_loc_3893e_389d6(
        &mut self,
        parent_x: u16,
        parent_y: u16,
        facing: Direction,
        parent_phase_word_22: u16,
        current_parent_x_direction_byte_16: u8,
    ) {
        if !self.active {
            return;
        }

        let prior_parent_x_direction = self.previous_parent_x_direction_byte_16;
        self.parent_phase_word_22 = parent_phase_word_22;
        self.current_parent_x_direction_byte_16 = current_parent_x_direction_byte_16;
        self.phase_index = ((parent_phase_word_22.saturating_sub(0x0EE0) >> 1) % 5) as u8;
        self.refresh_with_prior_x_direction_loc_3893e_389d6(
            parent_x,
            parent_y,
            facing,
            prior_parent_x_direction,
        );
        self.previous_parent_x_direction_byte_16 = current_parent_x_direction_byte_16;
    }

    fn refresh_loc_3893e_389d6(&mut self, parent_x: u16, parent_y: u16, facing: Direction) {
        self.refresh_with_prior_x_direction_loc_3893e_389d6(
            parent_x,
            parent_y,
            facing,
            self.previous_parent_x_direction_byte_16,
        );
    }

    fn refresh_with_prior_x_direction_loc_3893e_389d6(
        &mut self,
        parent_x: u16,
        parent_y: u16,
        facing: Direction,
        prior_parent_x_direction: u8,
    ) {
        let side_flag = u8::from(facing == Direction::Left);
        let primary = boss_eye_orbit_placement_sub_393C8(
            parent_x,
            parent_y,
            self.parent_phase_word_22,
            side_flag,
            prior_parent_x_direction,
            self.current_parent_x_direction_byte_16,
            self.primary_sprite_order_word_a,
        );
        let counter = boss_eye_counter_orbit_placement_sub_39420(
            parent_x,
            parent_y,
            self.parent_phase_word_22,
            side_flag,
            prior_parent_x_direction,
            self.current_parent_x_direction_byte_16,
            self.counter_sprite_order_word_a,
        );
        self.primary_x_pos_word = primary.x_pos;
        self.primary_y_pos_word = primary.y_pos;
        self.primary_sprite_order_word_a = primary.next_sprite_word_y_adjust;
        self.counter_x_pos_word = counter.x_pos;
        self.counter_y_pos_word = counter.y_pos;
        self.counter_sprite_order_word_a = counter.next_sprite_word_y_adjust;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyArchetype {
    Bouncer,
    Flyer,
    Hopper,
    Hoverer,
    Pacer,
    Stationary,
    Tracker,
    Inert,
}

/// Walking dragons also use the 1px Y offset.
const DRAGON_WALKING_SPAWN_Y_OFFSET_PX: u32 = 1;
/// ASM Dragon_Flying.asm loc_3492C: `move.w #2,-(sp); jsr Hibernate_Object`
/// before the walking-dragon child is allocated at loc_3CD16.
pub(crate) const DRAGON_FLYING_DEATH_PRE_CHILD_HIBERNATE_FRAMES: u16 = 2;
/// ASM `stru_345E4`: four `anim_frame 1, 2, ...` entries before the zero
/// terminator. GfxObject animation keeps advancing while the object hibernates.
pub(crate) const DRAGON_FLYING_DEATH_ANIMATION_FRAMES: u16 = 8;
/// ASM Fireball.asm Enemy19_Fireball_Kill reuses `stru_345E4` before the
/// loc_32ED8 delete tail.
pub(crate) const FIREBALL_DEATH_ANIMATION_FRAMES: u16 = DRAGON_FLYING_DEATH_ANIMATION_FRAMES;
/// ASM Mini_Hopping_Skull.asm loc_3F158 plays `stru_3E9AC`: frame
/// LnkTo_unk_C8040 for $0A ticks, then LnkTo_unk_C8048 for $1E ticks before
/// continuing to `j_Delete_CurrentObject`.
pub(crate) const MINI_HOPPING_SKULL_DEATH_ANIMATION_FRAMES: u16 = 0x0A + 0x1E;
/// ASM Big_Hopping_Skull.asm loc_3664E plays `stru_367FA`: $18, $0C, $0A,
/// then $64 ticks before the loc_366B2 delete tail.
pub(crate) const BIG_HOPPING_SKULL_DEATH_ANIMATION_FRAMES: u16 = 0x18 + 0x0C + 0x0A + 0x64;
/// ASM Archer.asm loc_3414A plays `stru_33FC2`: three 5-tick frames, then
/// $23 ticks before loc_34196.
pub(crate) const ARCHER_DEATH_ANIMATION_FRAMES: u16 = 0x05 + 0x05 + 0x05 + 0x23;
/// ASM Tornado.asm loc_359B2 plays `stru_35B1E`: $06, $03, $04, $04,
/// $03, then $06 ticks before loc_359EE.
pub(crate) const TORNADO_DEATH_ANIMATION_FRAMES: u16 = 0x06 + 0x03 + 0x04 + 0x04 + 0x03 + 0x06;
/// ASM Emo_Rock.asm loc_36346 plays `stru_36468`: five 6-tick frames before
/// the loc_3639A delete tail.
pub(crate) const EMO_ROCK_DEATH_ANIMATION_FRAMES: u16 = 6 * 5;
/// ASM Armadillo.asm loc_3C886 plays `stru_3C69A`: $16, $04, $04, $03, $03,
/// $02, $02, five $01-tick flashes, then $03 before the shared delete tail.
pub(crate) const ARMADILLO_DEATH_ANIMATION_FRAMES: u16 =
    0x16 + 0x04 + 0x04 + 0x03 + 0x03 + 0x02 + 0x02 + 0x01 + 0x01 + 0x01 + 0x01 + 0x01 + 0x03;
/// ASM Orca.asm loc_3D268 / loc_3D4BC plays `stru_3D11E`: $09, $04, $03,
/// $11, $04, $04, $03, $03, five $02-tick flashes, then $63 before loc_3E956.
pub(crate) const ORCA_DEATH_ANIMATION_FRAMES: u16 =
    0x09 + 0x04 + 0x03 + 0x11 + 0x04 + 0x04 + 0x03 + 0x03 + 0x02 + 0x02 + 0x02 + 0x02 + 0x02 + 0x63;
/// ASM Fire_Demon.asm loc_3A5A6 plays `stru_3A378`: three $03-tick frames
/// followed by three $04-tick frames before the loc_3C46E delete tail.
pub(crate) const FIRE_DEMON_DEATH_ANIMATION_FRAMES: u16 = 0x03 + 0x03 + 0x03 + 0x04 + 0x04 + 0x04;
/// ASM Crab.asm loc_3B094 plays `stru_3AF82`: $14, $0A, then $1E ticks
/// before the shared delete tail at loc_3C46E.
pub(crate) const CRAB_DEATH_ANIMATION_FRAMES: u16 = 0x14 + 0x0A + 0x1E;
/// ASM Cloud.asm loc_35C0E plays `stru_35EBA`: three 7-tick frames before
/// the loc_35C50 delete tail.
pub(crate) const CLOUD_DEATH_ANIMATION_FRAMES: u16 = 7 + 7 + 7;
/// ASM UFO.asm loc_35174/loc_35180 plays parent `stru_34CA0`: three
/// 7-tick frames before loc_351B6 observes `$19(a3)` and deletes the saucer.
pub(crate) const UFO_DEATH_ANIMATION_FRAMES: u16 = 7 + 7 + 7;
/// ASM Sphere.asm Enemy06_Sphere_Kill plays `stru_36182`: three $0A-tick
/// frames before the loc_36096 delete tail.
pub(crate) const SPHERE_DEATH_ANIMATION_FRAMES: u16 = 0x0A + 0x0A + 0x0A;
const SPHERE_LIVE_ZERO_FRAMES: u16 = 0x09;
const SPHERE_LIVE_ANIMATION_FRAMES: u16 = 0x09 + 0x09 + 0x09;
/// ASM Goat.asm loc_3CB64 plays `stru_3C9EA`: $1E, $0A, then $37 ticks
/// before the shared delete tail at loc_3E956.
pub(crate) const GOAT_DEATH_ANIMATION_FRAMES: u16 = 0x1E + 0x0A + 0x37;
/// ASM Ninja.asm loc_3DB4E plays `stru_3D50A`: $14, $0A, then $2D ticks
/// before the shared delete tail at loc_3E956.
pub(crate) const NINJA_DEATH_ANIMATION_FRAMES: u16 = 0x14 + 0x0A + 0x2D;
/// ASM Tank.asm loc_3B480 plays `stru_3B29A`: $10, $08, then $3C ticks,
/// and `Hibernate_UntilAnimFinished loc_1062` returns one terminal scheduler
/// frame before the shared delete tail at loc_3C46E.
pub(crate) const TANK_DEATH_ANIMATION_FRAMES: u16 = 0x10 + 0x08 + 0x3C + 1;
/// ASM Tank_Shooting.asm loc_3B6E6 plays `stru_3B518`: $10, $08, then $3C
/// ticks before the shared delete tail at loc_3C46E.
pub(crate) const TANK_SHOOTING_DEATH_ANIMATION_FRAMES: u16 = 0x10 + 0x08 + 0x3C;
/// ASM Robot.asm loc_3A9D2 plays `stru_3A74A`: two $09-tick frames before
/// the shared delete tail at loc_3C46E.
pub(crate) const ROBOT_DEATH_ANIMATION_FRAMES: u16 = 0x09 + 0x09;
/// ASM Diamond.asm loc_3AEE6 / loc_3AE28 plays `stru_3ACC4`: $08, $06,
/// $03, $03, then $02 ticks before the shared delete tail at loc_3C46E.
pub(crate) const DIAMOND_DEATH_ANIMATION_FRAMES: u16 = 0x08 + 0x06 + 0x03 + 0x03 + 0x02;
/// ASM Dragon.asm loc_3CEFC plays `stru_3CCEC`: $07, $06, $03, $16, $05,
/// $04, $04, $0C, $05, then $63 ticks before loc_3E956.
pub(crate) const DRAGON_WALKING_DEATH_ANIMATION_FRAMES: u16 =
    0x07 + 0x06 + 0x03 + 0x16 + 0x05 + 0x04 + 0x04 + 0x0C + 0x05 + 0x63;
/// ASM Scorpion.asm loc_3DCF4 plays `stru_3DB80`: $0A, $14, then $2D ticks
/// before the shared delete tail at loc_3E956.
pub(crate) const SCORPION_DEATH_ANIMATION_FRAMES: u16 = 0x0A + 0x14 + 0x2D;
/// ASM Driller.asm loc_3BF1C plays parent `stru_3BCD4`: $0A, $14, then
/// $28 ticks before the shared delete tail at loc_3C46E.
pub(crate) const DRILLER_DEATH_ANIMATION_FRAMES: u16 = 0x0A + 0x14 + 0x28;
/// ASM Lion.asm loc_3E378 plays `stru_3DF48`: three $0A-tick frames before
/// the shared delete tail at loc_3E956.
pub(crate) const LION_DEATH_ANIMATION_FRAMES: u16 = 0x0A + 0x0A + 0x0A;
/// ASM Spinning_Twins.asm loc_3BC56 plays `stru_3B83C`: $19, $0F, then
/// $14 ticks before the loc_3BC78/loc_3BCA2 delete path.
pub(crate) const SPINNING_TWINS_DEATH_ANIMATION_FRAMES: u16 = 0x19 + 0x0F + 0x14;
/// ASM Hand.asm loc_32AA2 plays `stru_32B78`: eleven 6-tick frames before
/// the loc_32AE6 delete tail.
pub(crate) const HAND_DEATH_ANIMATION_FRAMES: u16 = 6 * 11;
/// ASM Tar_Monster.asm loc_323E6 plays `stru_32700`: $09, $09, $03, $03,
/// $09, $03, $03, $09, $03, $03, then $09 ticks before loc_3243A.
pub(crate) const TAR_MONSTER_DEATH_ANIMATION_FRAMES: u16 =
    0x09 + 0x09 + 0x03 + 0x03 + 0x09 + 0x03 + 0x03 + 0x09 + 0x03 + 0x03 + 0x09;
/// ASM Drips.asm loc_338F8 plays `stru_33F06`: three 4-tick frames before
/// loc_3392C clears movement and hibernates to the delete tail.
pub(crate) const DRIPS_DEATH_ANIMATION_FRAMES: u16 = 4 + 4 + 4;
/// ASM loc_39EBE/loc_39F7A: boss heads play eight hibernate-until-finished
/// animation scripts (120 ticks total), then loop four 1-tick sweep scripts
/// ten times (17 ticks per loop) before `j_Delete_CurrentObject`.
pub(crate) const BOSS_HEAD_DEATH_PRE_SOUND_FRAMES_LOC_39EE2: u16 = 120;
pub(crate) const BOSS_HEAD_DEATH_SPIN_FRAMES_LOC_39F7A: u16 = 10 * 17;
pub(crate) const BOSS_HEAD_DEATH_ANIMATION_FRAMES: u16 =
    BOSS_HEAD_DEATH_PRE_SOUND_FRAMES_LOC_39EE2 + BOSS_HEAD_DEATH_SPIN_FRAMES_LOC_39F7A;

fn boss_head_death_script_frames_loc_39ebe() -> (u16, u16) {
    let pre_spin = boss_head_death_anim_sequence_loc_39EE2();
    let spin_loop = boss_head_death_spin_loop_loc_39F7A(pre_spin.death_spin_counter_d0);
    debug_assert_eq!(pre_spin.played_sound_effect, "sfx_Boss_dies");
    debug_assert!(spin_loop.delete_current_object);

    // ASM loc_39EE2 (kid.asm:43004) plays eight hibernate-until-finished
    // scripts for 120 ticks; loc_39F7A (kid.asm:43039) loops four sweep scripts
    // using d0 from loc_39EE2, 17 ticks per loop.
    (
        BOSS_HEAD_DEATH_PRE_SOUND_FRAMES_LOC_39EE2,
        spin_loop.loop_count_d0.saturating_mul(17),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossHeadDeathTransitionLoc39Ebe {
    pub(crate) next_death_count_fb4e: u8,
    pub(crate) starts_death_script_timer: u16,
    pub(crate) set_has_kid_collision: bool,
    pub(crate) enable_boss_body_and_flag_objects: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BossHeadDeathScriptPhaseLoc39Ebe {
    PreSpinLoc39Ee2,
    SpinLoopLoc39F7a,
}

impl BossHeadDeathScriptPhaseLoc39Ebe {
    pub(crate) fn asm_label(self) -> &'static str {
        match self {
            Self::PreSpinLoc39Ee2 => "loc_39EE2",
            Self::SpinLoopLoc39F7a => "loc_39F7A",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossHeadDeathScriptLoc39Ebe {
    pub(crate) phase: BossHeadDeathScriptPhaseLoc39Ebe,
    pub(crate) remaining_phase_frames: u16,
}

pub(crate) fn boss_head_death_transition_loc_39ebe(
    previous_death_count_fb4e: u8,
) -> BossHeadDeathTransitionLoc39Ebe {
    let next_death_count_fb4e = previous_death_count_fb4e.saturating_add(1);
    let (pre_spin_frames, spin_frames) = boss_head_death_script_frames_loc_39ebe();
    // ASM loc_39EBE: `st has_kid_collision(a3); addq.b #1,($FFFFFB4E).w`;
    // only count 3 releases the body/flag object chain before loc_39EE2.
    BossHeadDeathTransitionLoc39Ebe {
        next_death_count_fb4e,
        starts_death_script_timer: pre_spin_frames + spin_frames,
        set_has_kid_collision: true,
        enable_boss_body_and_flag_objects: next_death_count_fb4e == 3,
    }
}

pub(crate) fn boss_head_death_script_loc_39ebe(
    wait_timer: u16,
) -> Option<BossHeadDeathScriptLoc39Ebe> {
    if wait_timer == 0 {
        return None;
    }

    // ASM loc_39EBE: eight hibernate-until-finished scripts at loc_39EE2 run
    // before `sfx_Boss_dies`; loc_39F7A then loops ten 17-frame spin scripts
    // before j_Delete_CurrentObject.
    let (_, spin_frames) = boss_head_death_script_frames_loc_39ebe();
    if wait_timer > spin_frames {
        Some(BossHeadDeathScriptLoc39Ebe {
            phase: BossHeadDeathScriptPhaseLoc39Ebe::PreSpinLoc39Ee2,
            remaining_phase_frames: wait_timer - spin_frames,
        })
    } else {
        Some(BossHeadDeathScriptLoc39Ebe {
            phase: BossHeadDeathScriptPhaseLoc39Ebe::SpinLoopLoc39F7a,
            remaining_phase_frames: wait_timer,
        })
    }
}

pub(crate) fn enemy_death_animation_frames(family: EnemyFamily) -> Option<u16> {
    match family {
        EnemyFamily::Archer => Some(ARCHER_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Armadillo => Some(ARMADILLO_DEATH_ANIMATION_FRAMES),
        EnemyFamily::BigHoppingSkull => Some(BIG_HOPPING_SKULL_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Cloud => Some(CLOUD_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Crab => Some(CRAB_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Diamond => Some(DIAMOND_DEATH_ANIMATION_FRAMES),
        EnemyFamily::DragonWalking => Some(DRAGON_WALKING_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Driller => Some(DRILLER_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Drips => Some(DRIPS_DEATH_ANIMATION_FRAMES),
        EnemyFamily::EmoRock => Some(EMO_ROCK_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Fireball => Some(FIREBALL_DEATH_ANIMATION_FRAMES),
        EnemyFamily::FireDemon => Some(FIRE_DEMON_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Goat => Some(GOAT_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Hand => Some(HAND_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Lion => Some(LION_DEATH_ANIMATION_FRAMES),
        EnemyFamily::MiniHoppingSkull => Some(MINI_HOPPING_SKULL_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Ninja => Some(NINJA_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Orca => Some(ORCA_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Robot => Some(ROBOT_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Scorpion => Some(SCORPION_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Sphere => Some(SPHERE_DEATH_ANIMATION_FRAMES),
        EnemyFamily::SpinningTwins => Some(SPINNING_TWINS_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Shishkaboss | EnemyFamily::BoomerangBosses | EnemyFamily::BagelBrothers => {
            Some(BOSS_HEAD_DEATH_ANIMATION_FRAMES)
        }
        EnemyFamily::Tank => Some(TANK_DEATH_ANIMATION_FRAMES),
        EnemyFamily::TankShooting => Some(TANK_SHOOTING_DEATH_ANIMATION_FRAMES),
        EnemyFamily::TarMonster => Some(TAR_MONSTER_DEATH_ANIMATION_FRAMES),
        EnemyFamily::Tornado => Some(TORNADO_DEATH_ANIMATION_FRAMES),
        EnemyFamily::UFO => Some(UFO_DEATH_ANIMATION_FRAMES),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragonArtInitLoc3CDB8 {
    pub(crate) art_slot_index: u8,
    pub(crate) status_flag_13: bool,
    pub(crate) priority: u8,
    pub(crate) current_hp: u16,
}

pub(crate) fn dragon_art_init_loc_3cdb8(
    first_header_word: u16,
    second_header_word: u16,
    enemy_level: u8,
) -> DragonArtInitLoc3CDB8 {
    fn is_dragon_header_word(raw: u16) -> bool {
        matches!(raw & 0x0FFF, 0x000C | 0x000D)
    }

    let art_slot_index = if is_dragon_header_word(first_header_word) {
        0
    } else if is_dragon_header_word(second_header_word) {
        1
    } else {
        2
    };

    // ASM Dragon.asm loc_3CDB8: d4 selects the palette/VRAM table entry,
    // `$13(a3)` is set, priority is cleared, and current_hp is
    // `enemy_level + 1`.
    DragonArtInitLoc3CDB8 {
        art_slot_index,
        status_flag_13: true,
        priority: 0,
        current_hp: u16::from(enemy_level) + 1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonWalkingDamageRouteLoc3CEBE {
    Death {
        remaining_hp: u16,
    },
    SpawnedChildContinue {
        remaining_hp: u16,
    },
    ParentHurt {
        remaining_hp: u16,
        damage_animation: &'static str,
        restored_animation: &'static str,
    },
}

pub(crate) fn dragon_walking_damage_route_loc_3cebe(
    current_hp: u16,
    parent_spawned_child: bool,
) -> DragonWalkingDamageRouteLoc3CEBE {
    // ASM Dragon.asm loc_3CEBE: `subq.w #1,$44(a3)` sets the signed branch
    // flags; positive remaining HP on parent-spawned children skips the hurt
    // animation, while ordinary dragons play `stru_3CCE6` and restore `$62(a5)`.
    let remaining_hp = current_hp.wrapping_sub(1);
    let remaining_signed = i16::from_ne_bytes(remaining_hp.to_ne_bytes());
    if remaining_signed <= 0 {
        DragonWalkingDamageRouteLoc3CEBE::Death { remaining_hp }
    } else if parent_spawned_child {
        DragonWalkingDamageRouteLoc3CEBE::SpawnedChildContinue { remaining_hp }
    } else {
        DragonWalkingDamageRouteLoc3CEBE::ParentHurt {
            remaining_hp,
            damage_animation: "stru_3CCE6",
            restored_animation: "saved_dragon_walking_animation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragonWalkingTerminalRouteLoc3CEE4 {
    DeathAnimation {
        set_has_kid_collision: bool,
        animation: &'static str,
    },
    SpawnedChildResume {
        set_has_kid_collision: bool,
        next_collision_type: u16,
    },
}

pub(crate) fn dragon_walking_terminal_route_loc_3cee4(
    collision_type: i16,
    parent_spawned_child: bool,
) -> DragonWalkingTerminalRouteLoc3CEE4 {
    // ASM Dragon.asm loc_3CEE4: `$FFFF` skips the Kid-collision flag and
    // always plays `stru_3CCEC`; every other d4 sets `has_kid_collision`.
    // Spawned walking-dragon children then store `$FFFE` and resume instead.
    let set_has_kid_collision = collision_type != -1;
    if set_has_kid_collision && parent_spawned_child {
        DragonWalkingTerminalRouteLoc3CEE4::SpawnedChildResume {
            set_has_kid_collision,
            next_collision_type: 0xFFFE,
        }
    } else {
        DragonWalkingTerminalRouteLoc3CEE4::DeathAnimation {
            set_has_kid_collision,
            animation: "stru_3CCEC",
        }
    }
}

// --- Dragon Flying script interpreter (Dragon_Flying.asm) ---

/// Drift initial speed: $800 (subpixel FP16.16). ASM loc_34776.
const DRAGON_DRIFT_INITIAL_SPEED: i32 = 0x800;
const DRAGON_FLYING_COLID_RIGHTWALL: u16 = colid::RIGHTWALL;
const DRAGON_FLYING_COLID_LEFTWALL: u16 = colid::LEFTWALL;
const DRAGON_FLYING_COLID_FLOOR: u16 = colid::FLOOR;
const DRAGON_FLYING_COLID_CEILING: u16 = colid::CEILING;
const DRAGON_FLYING_COLID_SLOPEUP: u16 = colid::SLOPEUP;
const DRAGON_FLYING_COLID_SLOPEDOWN: u16 = colid::SLOPEDOWN;
const DRAGON_FLYING_MAIN_ANIMATION_FRAME_TICKS: u16 = 7;
const DRAGON_FLYING_MAIN_ANIMATION_FRAMES: u16 = 20;
const DRAGON_FLYING_MAIN_ANIMATION_TICKS: u16 =
    DRAGON_FLYING_MAIN_ANIMATION_FRAME_TICKS * DRAGON_FLYING_MAIN_ANIMATION_FRAMES;

pub(crate) fn dragon_flying_main_zero_contact_row_stru_34600(animation_tick: u16) -> bool {
    let frame = (animation_tick % DRAGON_FLYING_MAIN_ANIMATION_TICKS)
        / DRAGON_FLYING_MAIN_ANIMATION_FRAME_TICKS;
    matches!(frame, 2 | 3 | 6 | 7 | 10 | 11 | 14 | 15 | 18 | 19)
}

fn fp16_from_signed_word(word: i16) -> i32 {
    i32::from(word) << 16
}

fn fp16_high_signed_word(value: i32) -> i16 {
    let bytes = value.to_be_bytes();
    i16::from_be_bytes([bytes[0], bytes[1]])
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EnemyLayoutView<'a> {
    pub(crate) segment_index: usize,
    pub(crate) lower_bound: u16,
    pub(crate) upper_bound: u16,
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

fn enemy_spawn_inside_visible_inner_window_loc_36900(
    object_x_px: u32,
    object_y_px: u32,
    camera_x_px: u32,
    camera_y_px: u32,
) -> bool {
    let dx = object_x_px as i32 - camera_x_px as i32;
    let dy = object_y_px as i32 - camera_y_px as i32;
    // ASM Manage_EnemyLoading loc_36900 (kid.asm:38042-38053): after
    // `$FFFFFB51` has been set by the first pass, objects already inside the
    // visible inner box are skipped so they do not pop into view.
    (-0x20..=0x160).contains(&dx) && (-0x20..=0x100).contains(&dy)
}

impl ActiveEnemy {
    pub(crate) fn stomp_bounce_causes_contact_damage_loc_b6da(&self) -> bool {
        // ASM Scorpion.asm loc_3DDEE ORs `$C000` into the linked tail object's
        // object_meta while `stru_3DB8E` is active; Jump_On_Enemy loc_B6DA
        // tests bit14 with `bclr #$E,d6` after applying the stomp bounce.
        matches!(self.family, EnemyFamily::Scorpion) && self.scorpion_sting_frames > 0
    }

    pub(crate) fn is_dragon_walking_parent_spawned_child(&self) -> bool {
        matches!(self.family, EnemyFamily::DragonWalking)
            && self.dragon_script_variant & DRAGON_WALKING_PARENT_SPAWNED_CHILD_FLAG != 0
    }

    pub(crate) fn is_armadillo_parent_spawned_child(&self) -> bool {
        matches!(self.family, EnemyFamily::Armadillo)
            && self.dragon_script_variant & ARMADILLO_PARENT_SPAWNED_CHILD_FLAG != 0
    }

    pub(crate) fn is_armadillo_spawned_child_fall_waiting(&self) -> bool {
        matches!(self.family, EnemyFamily::Armadillo)
            && self.wait_timer == ARMADILLO_SPAWNED_CHILD_FALL_WAIT_TIMER_LOC_3C954
    }

    pub(crate) fn has_pending_dragon_walking_breath_child_loc_3ce90(&self) -> bool {
        // ASM Dragon.asm loc_3CE90 decrements `$66(a5)` and falls through to
        // loc_3CF6E/loc_3CF90 in the same object pass when the stored timer
        // becomes zero. Native spawns the projectile during `step`, but trace
        // counts sample before that mutation, so `wait_timer == 1` already has
        // a visible Object_RAM breath child in the ROM.
        matches!(self.family, EnemyFamily::DragonWalking)
            && self.dragon_script_cmd == DRAGON_WALKING_PHASE_WALK
            && self.wait_timer == 1
            && self.grounded_frames == 0
            && self.damage_flash_frames != ENEMY_DAMAGE_FLASH_FRAMES
    }

    fn take_dragon_walking_damage_or_queue_death(&mut self) -> bool {
        let parent_spawned_child = self.is_dragon_walking_parent_spawned_child();
        match dragon_walking_damage_route_loc_3cebe(
            u16::from(self.hit_points),
            parent_spawned_child,
        ) {
            DragonWalkingDamageRouteLoc3CEBE::Death { remaining_hp } => {
                match dragon_walking_terminal_route_loc_3cee4(
                    DRAGON_COLID_KIDABOVE as i16,
                    parent_spawned_child,
                ) {
                    DragonWalkingTerminalRouteLoc3CEE4::DeathAnimation { .. } => true,
                    DragonWalkingTerminalRouteLoc3CEE4::SpawnedChildResume { .. } => {
                        // ASM Dragon.asm loc_3CEE4: a `$5C(a5)` walking-dragon
                        // child stores collision `$FFFE` and resumes via loc_3CF50
                        // instead of entering the loc_3CEFC death animation.
                        self.hit_points = remaining_hp as u8;
                        self.death_animation_active = false;
                        false
                    }
                }
            }
            DragonWalkingDamageRouteLoc3CEBE::SpawnedChildContinue { remaining_hp } => {
                self.hit_points = remaining_hp as u8;
                false
            }
            DragonWalkingDamageRouteLoc3CEBE::ParentHurt { remaining_hp, .. } => {
                self.hit_points = remaining_hp as u8;
                self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                false
            }
        }
    }

    pub(crate) fn take_colid_hurt_or_queue_death(&mut self, player_x_pos: Option<u32>) -> bool {
        if matches!(self.family, EnemyFamily::Diamond) {
            match diamond_low_hp_collision_dispatch_loc_3aec4(
                DIAMOND_COLID_HURT as i16,
                u16::from(self.hit_points),
            ) {
                DiamondLowHpCollisionRouteLoc3AEC4::HurtAnimation { remaining_hp } => {
                    let hurt_animation = diamond_low_hp_hurt_animation_loc_3aef8();
                    debug_assert_eq!(hurt_animation.hurt_animation_script, "stru_3ACBE");
                    debug_assert_eq!(hurt_animation.next_collision_type, 0);
                    debug_assert_eq!(hurt_animation.restored_animation_script, "stru_3ACA8");
                    debug_assert!(hurt_animation.falls_through_to_shared_walker);
                    // ASM Diamond.asm loc_3AEC4 -> loc_3AEF8: generic
                    // `colid_hurt` decrements `$44(a3)`, plays the Diamond
                    // hurt script, clears collision, and resumes the base
                    // walker path. Native records the observable hibernate
                    // via the standard damage flash while preserving HP.
                    self.hit_points = remaining_hp as u8;
                    self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                    return false;
                }
                DiamondLowHpCollisionRouteLoc3AEC4::DeathAnimation => {
                    let death_animation = diamond_low_hp_death_animation_loc_3aee6();
                    debug_assert_eq!(death_animation.animation_script, "stru_3ACC4");
                    debug_assert!(death_animation.waits_until_animation_finished);
                    // ASM Diamond.asm loc_3AEE6: terminal `colid_hurt`
                    // initialises `stru_3ACC4` and waits for the Diamond
                    // death animation before the shared delete tail.
                    return death_animation.deletes_enemy;
                }
                DiamondLowHpCollisionRouteLoc3AEC4::ResumeSharedWalker { .. }
                | DiamondLowHpCollisionRouteLoc3AEC4::BerzerkerSideGate => return false,
            }
        }
        if matches!(self.family, EnemyFamily::Armadillo) {
            let object_meta =
                armadillo_object_meta_sync_loc_3c810(u16::from(self.family.object_type()));
            debug_assert_eq!(object_meta, u16::from(EnemyFamily::Armadillo.object_type()));
            match armadillo_collision_dispatch_loc_3c824(ARMADILLO_COLID_HURT) {
                ArmadilloCollisionRouteLoc3C824::Damage => {}
                ArmadilloCollisionRouteLoc3C824::ContinueSharedWalker
                | ArmadilloCollisionRouteLoc3C824::BerzerkerSideGate => return false,
                ArmadilloCollisionRouteLoc3C824::Death => return true,
            }
            let saved_walk_speed = if self.walk_speed_override != 0 {
                self.walk_speed_override
            } else {
                pacer_walk_speed(self.family, self.hit_points)
            };
            match armadillo_damage_response_loc_3c848(
                u16::from(self.hit_points),
                self.shared_walker_state,
                saved_walk_speed,
            ) {
                ArmadilloDamageResponseLoc3C848::RecoilActiveResume { current_hp } => {
                    // ASM Armadillo.asm loc_3C848: positive `$42(a3)` branches
                    // straight to loc_3C91A, so the projectile is consumed but
                    // HP, recoil state, and flash state are left untouched.
                    self.hit_points = current_hp as u8;
                    return false;
                }
                ArmadilloDamageResponseLoc3C848::Death => return true,
                ArmadilloDamageResponseLoc3C848::Hurt {
                    remaining_hp,
                    recoil_state_word,
                    restored_walk_speed,
                    ..
                } => {
                    // ASM Armadillo.asm loc_3C848: after the hurt animation
                    // finishes, `$42(a3)` is seeded with `$FF88`, `$50(a5)`
                    // is restored from `$5E(a5)`, and patrol resumes.
                    self.hit_points = remaining_hp as u8;
                    self.shared_walker_state = recoil_state_word;
                    self.walk_speed_override = restored_walk_speed;
                    self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                    return false;
                }
            }
        }
        if matches!(self.family, EnemyFamily::Crab)
            && let Some(player_x_pos) = player_x_pos
        {
            let crab_x = (self.x_pos >> 16) as u16;
            let kid_x = (player_x_pos >> 16) as u16;
            let crab_x_minus_kid_x = crab_x.wrapping_sub(kid_x) as i16;
            match crab_collision_dispatch_loc_3b048(
                CRAB_COLID_HURT,
                self.crab_frozen,
                crab_x_minus_kid_x,
                self.facing,
            ) {
                CrabCollisionRouteLoc3B048::ContinueSharedWalker
                | CrabCollisionRouteLoc3B048::BerzerkerSideGate => return false,
                CrabCollisionRouteLoc3B048::Death => return true,
                CrabCollisionRouteLoc3B048::Damage => {
                    let damage = crab_damage_after_hurt_loc_3b08c(u16::from(self.hit_points));
                    self.hit_points = damage.current_hp as u8;
                    match damage.route {
                        CrabDamageRouteLoc3B08C::DeathAnimation => return true,
                        CrabDamageRouteLoc3B08C::HurtAnimation => {
                            let hurt_sequence = crab_hurt_sequence_loc_3b0a6();
                            debug_assert_eq!(hurt_sequence.hurt_animation_script, "stru_3AF7C");
                            debug_assert_eq!(hurt_sequence.resumed_animation_script, "stru_3AF62");
                            debug_assert!(hurt_sequence.falls_through_to_proximity_callback);
                            // ASM Crab.asm loc_3B0A6 hibernates through the
                            // hurt animation before restoring the patrol script.
                            // Native represents that observable delay through
                            // the standard enemy damage-flash timer.
                            if hurt_sequence.waits_until_animation_finished {
                                self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                            }
                            return false;
                        }
                    }
                }
            }
        }

        self.take_damage_or_queue_death()
    }

    pub(crate) fn take_colid_hurt_or_queue_death_with_effects(
        &mut self,
        projectiles: &mut Vec<EnemyProjectile>,
        player_x_pos: Option<u32>,
    ) -> bool {
        let spawn_ufo_first_hurt_debris =
            matches!(self.family, EnemyFamily::UFO) && self.hit_points > 1;
        let parent_x_pos = self.x_pos;
        let parent_y_pos = self.y_pos;
        let parent_x_velocity = self.x_velocity;
        let parent_facing = self.facing;
        let dies = self.take_colid_hurt_or_queue_death(player_x_pos);
        if spawn_ufo_first_hurt_debris && !dies {
            // ASM UFO.asm loc_35150: clear `$47(a5)` routes the first hit
            // through loc_351EE debris allocation before loc_35228 marks the
            // saucer hurt.
            spawn_ufo_first_hurt_debris_loc_351ee(
                projectiles,
                parent_x_pos,
                parent_y_pos,
                parent_x_velocity,
                parent_facing,
            );
        }
        dies
    }

    pub(crate) fn take_damage_or_queue_death(&mut self) -> bool {
        if matches!(self.family, EnemyFamily::DragonWalking) {
            return self.take_dragon_walking_damage_or_queue_death();
        }
        if matches!(self.family, EnemyFamily::Robot) {
            if self.hit_points == 0 {
                return false;
            }
            let reaction = robot_hit_reaction_loc_3a8fc(u16::from(self.hit_points), false, false);
            self.hit_points = reaction.remaining_hp as u8;
            self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
            if matches!(reaction.route, RobotHitReactionRouteLoc3A8Fc::EnterCharge) {
                // ASM Robot.asm loc_3A8FC -> loc_3A96C: a final hit does not
                // delete the Robot; it seeds the `$800` collision charge state
                // and resumes via the current x_direction.
                self.robot_state = RobotState::Charging;
                self.wait_timer = ROBOT_CHARGE_FLIP_BASE;
                self.x_velocity = horizontal_velocity(self.facing, ROBOT_CHARGE_SPEED);
                self.y_velocity = 0;
            } else if matches!(self.robot_state, RobotState::Aiming) {
                // ASM Robot.asm loc_3AAD4 -> loc_3AB04 -> loc_3AAC4:
                // a nonfatal collision during the shooting coroutine restores
                // the walking animation path instead of resuming the shoot.
                finish_robot_shoot_sequence(self);
            }
            return false;
        }
        if matches!(self.family, EnemyFamily::Armadillo) {
            let object_meta =
                armadillo_object_meta_sync_loc_3c810(u16::from(self.family.object_type()));
            debug_assert_eq!(object_meta, u16::from(EnemyFamily::Armadillo.object_type()));
            match armadillo_collision_dispatch_loc_3c824(ARMADILLO_COLID_KIDABOVE) {
                ArmadilloCollisionRouteLoc3C824::Damage => {}
                ArmadilloCollisionRouteLoc3C824::ContinueSharedWalker
                | ArmadilloCollisionRouteLoc3C824::BerzerkerSideGate => return false,
                ArmadilloCollisionRouteLoc3C824::Death => return true,
            }
            let saved_walk_speed = if self.walk_speed_override != 0 {
                self.walk_speed_override
            } else {
                pacer_walk_speed(self.family, self.hit_points)
            };
            match armadillo_damage_response_loc_3c848(
                u16::from(self.hit_points),
                self.shared_walker_state,
                saved_walk_speed,
            ) {
                ArmadilloDamageResponseLoc3C848::RecoilActiveResume { current_hp } => {
                    // ASM Armadillo.asm loc_3C848: `colid_kidabove` shares
                    // the hurt path, and positive `$42(a3)` resumes without
                    // decrementing HP.
                    self.hit_points = current_hp as u8;
                    return false;
                }
                ArmadilloDamageResponseLoc3C848::Death => return true,
                ArmadilloDamageResponseLoc3C848::Hurt {
                    remaining_hp,
                    recoil_state_word,
                    restored_walk_speed,
                    ..
                } => {
                    // ASM Armadillo.asm loc_3C848: nonfatal Kid-above contact
                    // seeds `$42(a3)=$FF88`, restores `$50(a5)` from `$5E(a5)`,
                    // and hibernates on `stru_3C6D0` before patrol resumes.
                    self.hit_points = remaining_hp as u8;
                    self.shared_walker_state = recoil_state_word;
                    self.walk_speed_override = restored_walk_speed;
                    self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                    return false;
                }
            }
        }
        if matches!(self.family, EnemyFamily::SpinningTwins)
            && matches!(self.spinning_twins_phase, SpinningTwinsPhase::Walking)
        {
            match spinning_twins_damage_contact_loc_3bc34(
                u16::from(self.hit_points),
                match self.facing {
                    Direction::Left => 0,
                    Direction::Right => 0xFF,
                },
            ) {
                SpinningTwinsDamageContactLoc3BC34::Death { .. } => return true,
                SpinningTwinsDamageContactLoc3BC34::Recover(recovery) => {
                    // ASM Spinning_Twins.asm loc_3BC34: nonfatal settled
                    // damage hibernates through `stru_3B836`, restores the
                    // walking animation, then resumes through loc_3BBE6.
                    self.hit_points = recovery.next_hit_points as u8;
                    self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                    self.wait_timer = SPINNING_TWINS_HURT_RECOVERY_FRAMES_LOC_3BC34;
                    return false;
                }
            }
        }
        if self.hit_points <= 1 {
            return true;
        }
        if matches!(self.family, EnemyFamily::BigHoppingSkull) {
            let damage_entry =
                big_hopping_skull_damage_entry_loc_365d8(u16::from(self.hit_points), 0);
            debug_assert!(damage_entry.has_kid_collision);
            debug_assert_eq!(
                damage_entry.remaining_hit_points,
                u16::from(self.hit_points - 1)
            );
            debug_assert_eq!(
                damage_entry.route,
                BigHoppingSkullDamageEntryRouteLoc365D8::HitAnimationThenWait
            );
            self.hit_points = damage_entry.remaining_hit_points as u8;
            self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
            // ASM Big_Hopping_Skull.asm loc_365D8: nonfatal damage starts
            // `stru_367CC`; loc_365F2/loc_36616 hibernate the object until
            // the hit animation finishes before returning to loc_366B6.
            self.grounded_frames = BIG_HOPPING_SKULL_HIT_ANIMATION_FRAMES_LOC_365D8;
            return false;
        }
        if matches!(self.family, EnemyFamily::UFO) {
            let first_hurt = ufo_first_hurt_survive_loc_35228(self.y_velocity);
            self.hit_points = 1;
            self.y_velocity = first_hurt.parent_y_velocity;
            self.y_acceleration = first_hurt.d2;
            self.target_y_velocity = first_hurt.d3;
            self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
            return false;
        }
        if matches!(self.family, EnemyFamily::Tank) {
            let route = tank_update_loc_3b41c(
                0,
                TANK_COLID_HURT,
                u16::from(self.hit_points),
                TankMouthFrameLoc3B41C::Other,
                false,
            );
            self.hit_points = route.next_current_hp as u8;
            self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
            if matches!(route.route, TankUpdateRouteLoc3B41C::Hurt { .. }) {
                self.wait_timer = u16::from(TANK_HURT_TRACE_HIBERNATE_FRAMES_LOC_3B49A);
            }
            if route.clears_parent_motion_speed {
                self.x_velocity = 0;
                self.y_velocity = 0;
                self.walk_speed_override = TANK_STOPPED_WALK_SPEED_OVERRIDE_LOC_3B49A;
            }
            return matches!(route.route, TankUpdateRouteLoc3B41C::Death);
        }
        if matches!(self.family, EnemyFamily::Scorpion) {
            match scorpion_damage_route_loc_3dcd4(u16::from(self.hit_points)) {
                ScorpionDamageRouteLoc3DCD4::Death => return true,
                ScorpionDamageRouteLoc3DCD4::Hurt { remaining_hp, .. } => {
                    self.hit_points = remaining_hp as u8;
                    self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                    return false;
                }
            }
        }
        if matches!(self.family, EnemyFamily::Goat) {
            let saved_walk_speed = if self.walk_speed_override != 0 {
                self.walk_speed_override
            } else {
                pacer_walk_speed(self.family, self.hit_points)
            };
            match goat_damage_response_loc_3cb38(u16::from(self.hit_points), saved_walk_speed) {
                GoatDamageResponseLoc3CB38::Death => return true,
                GoatDamageResponseLoc3CB38::Hurt {
                    remaining_hp,
                    recoil_state_word,
                    restored_walk_speed,
                    ..
                } => {
                    self.hit_points = remaining_hp as u8;
                    self.shared_walker_state = recoil_state_word;
                    self.walk_speed_override = restored_walk_speed;
                    self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                    return false;
                }
            }
        }
        self.hit_points -= 1;
        self.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
        if matches!(self.family, EnemyFamily::TankShooting) {
            let x_direction = u8::from(matches!(self.facing, Direction::Right)) * 0xFF;
            let recovery = tank_shooting_hurt_recovery_loc_3b700(x_direction);
            // ASM Tank_Shooting.asm loc_3B700 clears parent motion `$50(a5)`
            // before the five-frame hibernate. Native stores the current
            // walker motion directly in `x_velocity`.
            self.x_velocity = recovery.parent_motion_speed;
            self.y_velocity = 0;
        }
        false
    }

    pub(crate) fn take_damage_or_queue_death_with_effects(
        &mut self,
        projectiles: &mut Vec<EnemyProjectile>,
    ) -> bool {
        let spawn_ufo_first_hurt_debris =
            matches!(self.family, EnemyFamily::UFO) && self.hit_points > 1;
        let parent_x_pos = self.x_pos;
        let parent_y_pos = self.y_pos;
        let parent_x_velocity = self.x_velocity;
        let parent_facing = self.facing;
        let dies = self.take_damage_or_queue_death();
        if spawn_ufo_first_hurt_debris && !dies {
            // ASM UFO.asm loc_35150: clear `$47(a5)` routes the first hit
            // through loc_351EE debris allocation before loc_35228 marks the
            // saucer hurt.
            spawn_ufo_first_hurt_debris_loc_351ee(
                projectiles,
                parent_x_pos,
                parent_y_pos,
                parent_x_velocity,
                parent_facing,
            );
        }
        dies
    }

    pub(crate) fn start_mini_hopping_skull_air_damage_loc_3f1e6(&mut self) {
        let setup = mini_hopping_skull_air_damage_setup_loc_3f1e6(
            u16::from(self.hit_points),
            self.x_velocity,
        );
        // ASM Mini_Hopping_Skull.asm loc_3F1E6 preserves live d2=x_vel while
        // the damage landing loop clears the object velocity. Native keeps d2
        // in target_x_velocity until loc_3F22A can restore it on slope contact.
        self.hit_points = setup.remaining_hits as u8;
        self.target_x_velocity = setup.preserved_x_velocity;
        self.x_velocity = setup.x_velocity;
        self.y_velocity = setup.y_velocity;
        self.wait_timer = MINI_HOPPING_SKULL_AIR_DAMAGE_WAIT_TIMER_LOC_3F200;
        self.grounded_frames = 0;
        self.on_ground = false;
        self.damage_flash_frames = 0;
        self.death_animation_active = false;
    }

    /// Whether this enemy can be damaged (stomped, hit by projectile, etc.)
    pub(crate) fn is_vulnerable(&self) -> bool {
        if self.hit_points == 0 {
            return false;
        }
        if matches!(
            (self.family, self.spinning_twins_phase),
            (
                EnemyFamily::SpinningTwins,
                SpinningTwinsPhase::FallingVisual
            )
        ) {
            return false;
        }
        if matches!(self.family, EnemyFamily::TarMonster) {
            // ORIGINAL_QUIRK: the Tar Monster phases in and out by design. ASM
            // Tar_Monster.asm script `unk_32264` cycles hibernate -> appear
            // (`loc_322BC`, sfx_Tar_Monster_appears) -> attack -> disappear
            // (`loc_322DA`, sfx_Tar_Monster_disappears), so it is only on-screen
            // during Visible/Attacking. The "flicker" reported in playtest is
            // this faithful emerge/submerge cycle, not a render bug.
            return matches!(
                self.tar_monster_state,
                TarMonsterState::Visible | TarMonsterState::Attacking
            );
        }
        if matches!(self.family, EnemyFamily::Crab) && self.crab_frozen {
            return false;
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
        let walk_phase = if matches!(self.family, EnemyFamily::DragonWalking)
            || (matches!(self.family, EnemyFamily::Lion)
                && matches!(
                    self.lion_state,
                    LionState::Walking | LionState::Charging | LionState::ResumingCharge
                )) {
            Some(self.dragon_script_cmd)
        } else {
            None
        };
        let wait_timer = if matches!(self.family, EnemyFamily::DragonWalking | EnemyFamily::Lion)
            || (matches!(self.family, EnemyFamily::BigHoppingSkull)
                && self.wait_timer > 0
                && !self.death_animation_active)
            || (self.death_animation_active && enemy_death_animation_frames(self.family).is_some())
        {
            Some(self.wait_timer)
        } else {
            None
        };
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
        if matches!(self.family, EnemyFamily::Lion) {
            // ASM Lion.asm loc_3E078/loc_3E0F4/loc_3E11E split the parent
            // timing across `$54(a5)`, `$5B(a5)`, and the animation callback;
            // expose native equivalents while Level 0x07 skip-settle parity is
            // being narrowed.
            debug_fields.insert("lion_state".into(), format!("{:?}", self.lion_state));
            debug_fields.insert(
                "lion_active_projectiles".into(),
                lion_active_projectile_count(self).to_string(),
            );
        }
        if matches!(self.family, EnemyFamily::HeadyMetal) {
            debug_fields.insert(
                "heady_metal_palette_phase_sub_379e8".into(),
                self.heady_metal_palette_phase_counter.to_string(),
            );
            debug_fields.insert(
                "heady_metal_palette_word_5e_sub_379e8".into(),
                format!("{:#06x}", self.heady_metal_palette_word_5e),
            );
        }
        if is_multi_head_boss(self.family) && self.boss_eye_orbit_workers.active {
            // ASM loc_3893E/loc_389D6 passes the parent `$22(a2)` and `$16(a2)`
            // values into sub_393C8/sub_39420; keep them visible while the
            // native parent animation model is still being promoted.
            debug_fields.insert(
                "boss_eye_parent_phase_word_22_loc_3893e".into(),
                format!("{:#06x}", self.boss_eye_orbit_workers.parent_phase_word_22),
            );
            debug_fields.insert(
                "boss_eye_parent_frame_byte_16_loc_3893e".into(),
                self.boss_eye_orbit_workers
                    .current_parent_x_direction_byte_16
                    .to_string(),
            );
            debug_fields.insert(
                "boss_eye_parent_x_direction_byte_16_loc_3893e".into(),
                self.boss_eye_orbit_workers
                    .current_parent_x_direction_byte_16
                    .to_string(),
            );
        }
        if is_multi_head_boss(self.family)
            && self.death_animation_active
            && let Some(script) = boss_head_death_script_loc_39ebe(self.wait_timer)
        {
            // ASM loc_39EBE owns the boss-head object until the death script
            // reaches j_Delete_CurrentObject; expose the typed ROM script
            // phase while live native still stores it as one hibernate timer.
            debug_fields.insert(
                "boss_head_death_script_loc_39ebe".into(),
                script.phase.asm_label().into(),
            );
            debug_fields.insert(
                "boss_head_death_script_remaining_loc_39ebe".into(),
                script.remaining_phase_frames.to_string(),
            );
        }
        let x_velocity = if matches!(self.family, EnemyFamily::Tank | EnemyFamily::TankShooting)
            && self.on_ground
        {
            // ASM Tank.asm loc_3B40A / Tank_Shooting.asm loc_3B6AE enter the
            // shared walker at kid.asm loc_3C026/loc_3C0D2, which clears the
            // object x_vel/y_vel. Native keeps `$50(a5)` patrol speed in
            // `x_velocity`; hide that storage detail from ROM-visible snapshots.
            0
        } else {
            self.x_velocity
        };
        let contact_phase = if matches!(self.family, EnemyFamily::Sphere)
            && !self.death_animation_active
            && self.wait_timer % SPHERE_LIVE_ANIMATION_FRAMES < SPHERE_LIVE_ZERO_FRAMES
        {
            // ASM Sphere.asm `stru_36174` starts with `LnkTo_unk_C8120` for
            // 9 hibernated ticks before the two active body frames. That link
            // resolves through GfxObjects_CollisionKid loc_1BCE to zero row
            // `unk_31AAE`, so the live parent has no Kid contact in this phase.
            Some(NativeContactPhase::SphereLiveZero)
        } else if matches!(self.family, EnemyFamily::Dragon)
            && matches!(self.dragon_phase, DragonFlyingState::Descending)
            && dragon_flying_dive_zero_contact_row_loc_348ce(self.x_velocity, self.y_velocity)
        {
            // ASM Dragon_Flying.asm loc_348CE indexes `off_345F6` from the
            // current dive-script velocity byte. `LnkTo_unk_C75FE` and
            // `LnkTo_unk_C7616` resolve through GfxObjects_CollisionKid
            // loc_1BCE to CollisionSize row `unk_31DE0`, so those descent
            // frames have no parent Kid contact.
            Some(NativeContactPhase::DragonFlyingDiveZero)
        } else if matches!(self.family, EnemyFamily::Dragon)
            && !self.death_animation_active
            && !matches!(self.dragon_phase, DragonFlyingState::Descending)
            && dragon_flying_main_zero_contact_row_stru_34600(self.wait_timer)
        {
            // ASM Dragon_Flying.asm `stru_34600` repeats 20 seven-tick
            // normal-animation frames. `LnkTo_unk_C75FE` and
            // `LnkTo_unk_C7606` resolve through GfxObjects_CollisionKid
            // loc_1BCE to CollisionSize row `unk_31DE0`, so those normal
            // frames have no parent Kid contact.
            Some(NativeContactPhase::DragonFlyingMainZero)
        } else if matches!(self.family, EnemyFamily::TarMonster)
            && !matches!(
                self.tar_monster_state,
                TarMonsterState::Visible | TarMonsterState::Attacking
            )
        {
            // ASM Tar_Monster.asm `stru_3272E`/`stru_3274C` expose
            // hidden/emerge/submerge frames whose sprite links map through
            // GfxObjects_CollisionKid loc_1BCE to zero row `unk_314CE`.
            Some(NativeContactPhase::TarMonsterHidden)
        } else if matches!(self.family, EnemyFamily::SpinningTwins)
            && (matches!(self.spinning_twins_phase, SpinningTwinsPhase::FallingVisual)
                || self.damage_flash_frames > 0
                || self.death_animation_active)
        {
            // ASM Spinning_Twins.asm `stru_3B810`, `stru_3B836`, and
            // `stru_3B83C` route through GfxObjects_CollisionKid loc_1BCE
            // to zero row `unk_314CE`.
            Some(NativeContactPhase::SpinningTwinsZero)
        } else if matches!(self.family, EnemyFamily::Tornado)
            && !self.death_animation_active
            && tornado_live_zero_contact_row_stru_35b10(self.grounded_frames)
        {
            // ASM Tornado.asm `stru_35B10` advances while loc_35908
            // hibernates. `LnkTo_unk_C7856` and `LnkTo_unk_C785E` resolve
            // through GfxObjects_CollisionKid loc_1BCE to zero rows
            // `unk_31D28`/`unk_31D2A`.
            Some(NativeContactPhase::TornadoLiveZero)
        } else {
            None
        };

        NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy {
                object_type: self.family.object_type(),
            },
            x_pos: self.x_pos,
            y_pos: self.y_pos,
            layout_index: Some(self.layout_index),
            x_velocity: Some(x_velocity),
            y_velocity: Some(self.y_velocity),
            facing: Some(self.facing),
            on_ground: Some(self.on_ground),
            hit_points: Some(self.hit_points),
            damage_flash: self.damage_flash_frames > 0,
            palette_variant: self.palette_variant,
            stationary: self.dragon_phase.is_stationary()
                || (self.death_animation_active
                    && enemy_death_animation_frames(self.family).is_some()),
            contact_phase,
            shared_walker_state,
            shared_walker_patrol_timer,
            shared_walker_patrol_short_next,
            walk_phase,
            wait_timer,
            debug_fields,
        }
    }

    pub(crate) fn child_attachment_snapshot(&self) -> Option<NativeObjectSnapshot> {
        if self.death_animation_active {
            // ASM Tank.asm loc_3B480 / Tank_Shooting.asm loc_3B6E6 and
            // Scorpion.asm loc_3DCF4 hide/disable linked children once the
            // parent enters the death sequence.
            return None;
        }

        let (x_pos, y_pos, x_velocity, facing) = match self.family {
            EnemyFamily::Robot => {
                // ASM Robot.asm loc_3A7EE/loc_3A816 allocates the linked head
                // object at `$3A(a5)`, copies parent x/y/direction, then
                // subtracts `$10` from child y_pos before `stru_3A716`.
                let parent_x_direction = match self.facing {
                    Direction::Left => 0x00,
                    Direction::Right => 0xFF,
                };
                let attachment =
                    shared_enemy_attachment_sub_3BF72(SharedEnemyAttachmentInputSub3BF72 {
                        family: self.family,
                        parent_x_pos: (self.x_pos >> 16) as i16,
                        parent_y_pos: (self.y_pos >> 16) as i16,
                        parent_x_direction,
                        child_y_offset: 0x10,
                        robot_parent_state: 0,
                        robot_child_mapping: 0x0800,
                        child_x_direction: parent_x_direction,
                    })
                    .primary;
                (
                    u32::from(attachment.x_pos as u16) << 16,
                    u32::from(attachment.y_pos as u16) << 16,
                    attachment.x_velocity,
                    attachment.x_direction.map(|x_direction| {
                        if x_direction == 0 {
                            Direction::Left
                        } else {
                            Direction::Right
                        }
                    }),
                )
            }
            EnemyFamily::Tank | EnemyFamily::TankShooting => {
                // ASM Tank.asm loc_3B372 / Tank_Shooting.asm loc_3B5FA allocate a
                // linked turret gfx object at `$3A(a5)`, then the shared walker
                // `sub_3BF72` keeps it synced using the child `$46` y-offset.
                let parent_x_direction = match self.facing {
                    Direction::Left => 0x00,
                    Direction::Right => 0xFF,
                };
                let attachment =
                    shared_enemy_attachment_sub_3BF72(SharedEnemyAttachmentInputSub3BF72 {
                        family: self.family,
                        parent_x_pos: (self.x_pos >> 16) as i16,
                        parent_y_pos: (self.y_pos >> 16) as i16,
                        parent_x_direction,
                        child_y_offset: tank_child_y_offset_loc_3b372_3b49a(
                            self.family,
                            self.damage_flash_frames,
                            self.wait_timer,
                            self.walk_speed_override == TANK_STOPPED_WALK_SPEED_OVERRIDE_LOC_3B49A,
                        ),
                        robot_parent_state: 0,
                        robot_child_mapping: 0,
                        child_x_direction: parent_x_direction,
                    })
                    .primary;
                (
                    u32::from(attachment.x_pos as u16) << 16,
                    u32::from(attachment.y_pos as u16) << 16,
                    attachment.x_velocity,
                    attachment.x_direction.map(|x_direction| {
                        if x_direction == 0 {
                            Direction::Left
                        } else {
                            Direction::Right
                        }
                    }),
                )
            }
            EnemyFamily::Scorpion => {
                // ASM Scorpion.asm loc_3DC86 keeps the linked tail gfx object
                // at `$3A(a5)` synced to the parent before collision dispatch.
                let attachment = scorpion_tail_sync_loc_3dc86(self.x_pos, self.y_pos, self.facing);
                (
                    u32::from(attachment.x_word as u16) << 16,
                    u32::from(attachment.y_word as u16) << 16,
                    0,
                    Some(self.facing),
                )
            }
            _ => return None,
        };

        let contact_phase = if matches!(self.family, EnemyFamily::Robot)
            && self.damage_flash_frames > 0
        {
            // ASM Robot.asm loc_3A8FC starts child hit animation `stru_3A744`;
            // its `LnkTo_unk_C8308/C8310` frames route through
            // GfxObjects_CollisionKid loc_1BCE to zero row `unk_31C20`.
            Some(NativeContactPhase::RobotHeadHitZero)
        } else if matches!(self.family, EnemyFamily::Scorpion) && self.scorpion_sting_frames > 0 {
            // ASM Scorpion.asm loc_3DDEE switches the linked tail object to
            // sting script `stru_3DB8E`; contact then uses CollisionSize rows
            // `unk_31926`/`unk_31930` through GfxObjects_CollisionKid loc_1BCE.
            Some(NativeContactPhase::ScorpionStingTail)
        } else {
            None
        };

        Some(NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy {
                object_type: self.family.object_type(),
            },
            x_pos,
            y_pos,
            layout_index: Some(self.layout_index),
            x_velocity: Some(x_velocity),
            y_velocity: Some(0),
            facing,
            on_ground: None,
            hit_points: Some(0),
            damage_flash: false,
            palette_variant: self.palette_variant,
            stationary: false,
            contact_phase,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        })
    }
}

fn tank_child_y_offset_loc_3b372_3b49a(
    family: EnemyFamily,
    damage_flash_frames: u8,
    wait_timer: u16,
    parent_motion_cleared_loc_3b49a: bool,
) -> i16 {
    match family {
        EnemyFamily::Tank
            if (1..=u16::from(TANK_CHILD_RECOIL_VISIBLE_FRAMES_LOC_3B49A))
                .contains(&wait_timer)
                || (wait_timer == 0 && parent_motion_cleared_loc_3b49a) =>
        {
            6
        }
        EnemyFamily::TankShooting
            if (1..=TANK_SHOOTING_CHILD_RECOIL_VISIBLE_FRAMES_LOC_3B700)
                .contains(&damage_flash_frames) =>
        {
            6
        }
        EnemyFamily::Tank | EnemyFamily::TankShooting => 8,
        _ => 0,
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
            Self::Diamond => EnemyArchetype::Pacer,
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
            contact_phase: None,
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
        false,
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
    enemy_loading_initial_pass_complete: bool,
    killed: &std::collections::HashSet<(usize, usize)>,
    rng: &mut RomRandomState,
) {
    activate_enemies_with_rng_inner(
        active,
        layout,
        window,
        camera_x_px,
        camera_y_px,
        level_bounds,
        player_x,
        player_y,
        blank_top_rows,
        enemy_loading_initial_pass_complete,
        killed,
        rng,
        false,
    );
}

#[allow(clippy::too_many_arguments)]
fn activate_enemies_with_rng_inner(
    active: &mut Vec<ActiveEnemy>,
    layout: &EnemyLayout,
    window: ActivationWindow,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    player_x: u32,
    player_y: u32,
    blank_top_rows: u16,
    enemy_loading_initial_pass_complete: bool,
    killed: &std::collections::HashSet<(usize, usize)>,
    rng: &mut RomRandomState,
    defer_rock_tank_spawn: bool,
) {
    let view = enemy_layout_view(layout, camera_x_px, camera_y_px);
    // Most enemies despawn on spawn-point exit. Flyers keep their live object
    // around longer; the Level 1 Dragon continues drifting after its layout
    // anchor scrolls behind the camera.
    // ASM loc_3C46E (kid.asm:43965-44020): the shared deactivation routine
    // entered via loc_3C436 trampoline. Reads object_meta, dispatches
    // SpinningTwins to loc_3BC56, Driller/Tank to loc_3C494 (drive sound
    // stop with refcount decrement and optional StopSequence via
    // j_PlaySound2), then falls
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
        let object_y_with_blank_rows = object_y + u32::from(blank_top_rows) * 8;
        if enemy_loading_initial_pass_complete
            && enemy_spawn_inside_visible_inner_window_loc_36900(
                object_x,
                object_y_with_blank_rows,
                camera_x_px,
                camera_y_px,
            )
        {
            continue;
        }
        let family = EnemyFamily::from_layout_object(view.enemy_type_slots, object);
        if matches!(family, EnemyFamily::MiniHoppingSkull)
            && !mini_hopping_skull_spawn_gate_loc_3e9d0(object_x, object_y, player_x, player_y)
        {
            continue;
        }
        if defer_rock_tank_spawn && matches!(family, EnemyFamily::Tank | EnemyFamily::TankShooting)
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
        let dragon_walking_art_init = if matches!(family, EnemyFamily::DragonWalking) {
            Some(dragon_art_init_loc_3cdb8(
                view.enemy_type_slots.first().copied().unwrap_or(0),
                view.enemy_type_slots.get(1).copied().unwrap_or(0),
                enemy_level,
            ))
        } else {
            None
        };

        // Multi-head bosses spawn 3 head objects per layout entry.
        let head_count = if is_multi_head_boss(family) { 3u8 } else { 1 };
        let (head_hp_table, head_x_offsets, head_y_offsets) = if is_multi_head_boss(family) {
            boss_head_config(family)
        } else {
            ([0u8; 3], [0i16; 3], [0i16; 3])
        };

        for head_idx in 0..head_count {
            let crab_visible_init = if matches!(family, EnemyFamily::Crab) {
                Some(crab_init_visible_state_loc_3afd0(enemy_level))
            } else {
                None
            };
            let orca_init = if matches!(family, EnemyFamily::Orca) {
                Some(orca_init_state_loc_3d194())
            } else {
                None
            };
            let ninja_init = if matches!(family, EnemyFamily::Ninja) {
                Some(ninja_init_state_loc_3d554(enemy_level))
            } else {
                None
            };
            let tar_monster_init = if matches!(family, EnemyFamily::TarMonster) {
                Some(tar_monster_init_loc_321e4(
                    object_x as u16,
                    object_y as u16,
                    enemy_level,
                    blank_top_rows,
                ))
            } else {
                None
            };
            let base_x = spawn_x_pos(family, object_x);
            let base_y = spawn_y;
            let head_x = if is_multi_head_boss(family) {
                let offset_px = head_x_offsets[head_idx as usize];
                (base_x as i64 + (i64::from(offset_px) << 16)).clamp(0, i64::from(u32::MAX)) as u32
            } else if let Some(init) = tar_monster_init {
                init.x_pos
            } else {
                base_x
            };
            let head_y = if is_multi_head_boss(family) {
                let offset_px = head_y_offsets[head_idx as usize];
                (base_y as i64 + (i64::from(offset_px) << 16)).clamp(0, i64::from(u32::MAX)) as u32
            } else if let Some(init) = tar_monster_init {
                init.y_pos
            } else {
                base_y
            };
            let lion_visible_init = if matches!(family, EnemyFamily::Lion) {
                Some(lion_init_visible_state_loc_3df92(
                    u16::from(enemy_level),
                    (head_x >> 16) as u16,
                    (player_x >> 16) as u16,
                ))
            } else {
                None
            };
            let hit_points = if matches!(
                family,
                EnemyFamily::Shishkaboss | EnemyFamily::BagelBrothers
            ) {
                // ASM loc_39478/loc_39598/loc_396BE: each Shishka/Bagel head's
                // init routine writes the per-family HP after the caller has
                // already copied that head's `$44/$46` anchor into x/y.
                let variant_d2 = if matches!(family, EnemyFamily::Shishkaboss) {
                    0
                } else {
                    1
                };
                let head_x_word = (head_x >> 16) as u16;
                let head_y_word = (head_y >> 16) as u16;
                match head_idx {
                    0 => {
                        shishka_bagel_head_1_init_loc_39478(head_x_word, head_y_word, variant_d2)
                            .hit_points as u8
                    }
                    1 => {
                        shishka_bagel_head_2_init_loc_39598(head_x_word, head_y_word, variant_d2)
                            .hit_points as u8
                    }
                    _ => {
                        shishka_bagel_head_3_init_loc_396BE(head_x_word, head_y_word, variant_d2)
                            .hit_points as u8
                    }
                }
            } else if is_multi_head_boss(family) {
                head_hp_table[head_idx as usize]
            } else if let Some(visible_init) = crab_visible_init {
                visible_init.current_hp
            } else if let Some(init) = orca_init {
                init.current_hp
            } else if let Some(init) = ninja_init {
                init.current_hp
            } else if let Some(init) = lion_visible_init {
                init.current_hp as u8
            } else if let Some(init) = dragon_walking_art_init {
                init.current_hp as u8
            } else if let Some(init) = tar_monster_init {
                init.hit_points
            } else {
                spawn_hit_points(family, enemy_level)
            };
            if let Some(init) = tar_monster_init {
                debug_assert_eq!(init.priority, 0);
                debug_assert_eq!(init.object_meta, u16::from(family.object_type()));
            }
            if let Some(init) = orca_init {
                debug_assert!(init.status_flag_13);
                debug_assert_eq!(init.object_meta, u16::from(family.object_type()));
                debug_assert_eq!(init.priority, 0);
            }
            if let Some(init) = ninja_init {
                debug_assert!(init.status_flag_13);
                debug_assert_eq!(init.object_meta, u16::from(family.object_type()));
                debug_assert_eq!(init.priority, 0);
                debug_assert_eq!(init.current_hp, hit_points);
                debug_assert_eq!(
                    init.walk_speed_override,
                    pacer_walk_speed(family, hit_points)
                );
            }
            if matches!(family, EnemyFamily::Tank) {
                let visible_init =
                    tank_init_visible_state_loc_3b31e(enemy_level, view.enemy_type_slots);
                debug_assert!(visible_init.status_flag_13);
                debug_assert_eq!(
                    visible_init.object_meta,
                    0x80 | u16::from(family.object_type())
                );
                debug_assert_eq!(
                    visible_init.art_slot_word,
                    tank_shooting_art_slot_loc_3b56c(view.enemy_type_slots).wrapping_mul(2)
                );
                debug_assert!(matches!(visible_init.palette_line, 1 | 2));
                debug_assert!(matches!(visible_init.vram_tile, 0x02FB | 0x0409 | 0x0517));
                debug_assert_eq!(visible_init.priority, 0);
                debug_assert_eq!(
                    visible_init.parent_timer_4a,
                    pacer_probe_height(family) as u16
                );
                debug_assert_eq!(
                    visible_init.parent_timer_48,
                    pacer_probe_half_width(family) as u16
                );
                debug_assert_eq!(visible_init.current_hp, hit_points);
                debug_assert_eq!(
                    visible_init.walk_speed,
                    pacer_walk_speed(family, hit_points)
                );
            }
            if matches!(family, EnemyFamily::TankShooting) {
                let visible_init =
                    tank_shooting_init_visible_state_loc_3b5a6(enemy_level, view.enemy_type_slots);
                debug_assert!(visible_init.status_flag_13);
                debug_assert_eq!(
                    visible_init.object_meta,
                    0x80 | u16::from(family.object_type())
                );
                debug_assert_eq!(
                    visible_init.art_slot_word,
                    tank_shooting_art_slot_loc_3b56c(view.enemy_type_slots).wrapping_mul(2)
                );
                debug_assert!(matches!(visible_init.palette_line, 1 | 2));
                debug_assert!(matches!(visible_init.vram_tile, 0x02FB | 0x0409 | 0x0517));
                debug_assert_eq!(visible_init.priority, 0);
                debug_assert_eq!(
                    visible_init.parent_timer_4a,
                    pacer_probe_height(family) as u16
                );
                debug_assert_eq!(
                    visible_init.parent_timer_48,
                    pacer_probe_half_width(family) as u16
                );
                debug_assert_eq!(visible_init.current_hp, hit_points);
                debug_assert_eq!(
                    visible_init.walk_speed,
                    pacer_walk_speed(family, hit_points)
                );
            }
            if let Some(visible_init) = crab_visible_init {
                debug_assert!(visible_init.status_flag_13);
                debug_assert!(visible_init.is_animated);
                debug_assert_eq!(visible_init.object_meta, 0x0009);
                debug_assert_eq!(visible_init.priority, 0);
                debug_assert_eq!(visible_init.current_hp, hit_points);
                debug_assert_eq!(
                    visible_init.speed_preload.wrapping_add(0x0000_8000),
                    pacer_walk_speed(family, hit_points)
                );
            }
            let goat_init_animation = if matches!(family, EnemyFamily::Goat) {
                goat_hp4_init_animation_loc_3_caca(hit_points)
                    .or_else(|| goat_high_hp_init_animation_loc_3_cae8(hit_points))
            } else {
                None
            };
            let armadillo_init_animation = if matches!(family, EnemyFamily::Armadillo) {
                armadillo_hp2_init_animation_loc_3c7c0(hit_points)
                    .or_else(|| armadillo_high_hp_init_animation_loc_3c7de(hit_points))
            } else {
                None
            };
            let fire_demon_init_animation = if matches!(family, EnemyFamily::FireDemon) {
                fire_demon_hp5_animation_loc_3a444(hit_points)
                    .or_else(|| fire_demon_non_hp5_animation_loc_3a464(hit_points))
            } else {
                None
            };
            let lion_motion_profile =
                lion_visible_init.map(|init| lion_init_motion_profile_loc_3dfd8(init.current_hp));
            if let Some(visible_init) = lion_visible_init {
                debug_assert!(visible_init.status_flag_13);
                debug_assert_eq!(visible_init.object_meta, 0x0092);
                debug_assert_eq!(visible_init.palette_line, 2);
                debug_assert_eq!(visible_init.current_hp as u8, hit_points);
            }
            if let Some(init) = dragon_walking_art_init {
                debug_assert!(init.art_slot_index <= 2);
                debug_assert!(init.status_flag_13);
                debug_assert_eq!(init.priority, 0);
                debug_assert_eq!(init.current_hp as u8, hit_points);
            }
            let facing = lion_visible_init.map_or_else(
                || spawn_facing(family, head_x, player_x),
                |init| init.facing,
            );
            let facing = if matches!(family, EnemyFamily::FireDemon) {
                // ASM Fire_Demon.asm loc_3A476: initial x_direction selects the
                // shared walker entry, preserving the decoded spawn facing.
                match fire_demon_init_walk_dispatch_loc_3a476(facing) {
                    FireDemonInitWalkDispatch::LeftSharedWalker => Direction::Left,
                    FireDemonInitWalkDispatch::RightSharedWalker => Direction::Right,
                }
            } else {
                facing
            };
            if let Some(profile) = lion_motion_profile {
                debug_assert_eq!(profile.walk_speed, pacer_walk_speed(family, hit_points));
                debug_assert_eq!(profile.charge_speed, profile.walk_speed);
                debug_assert!(matches!(
                    profile.animation_script,
                    "stru_3DEDA" | "stru_3DEF4"
                ));
                match lion_init_facing_dispatch_loc_3e030(facing) {
                    LionFacingDispatch::LeftPacing => {
                        debug_assert_eq!(facing, Direction::Left);
                    }
                    LionFacingDispatch::RightPacing => {
                        debug_assert_eq!(facing, Direction::Right);
                    }
                }
            }
            if let Some(profile) = goat_init_animation {
                debug_assert_eq!(profile.initial_animation_script, "stru_3C996");
                debug_assert_eq!(profile.saved_patrol_animation, "stru_3C996");
                debug_assert!(matches!(
                    profile.saved_damage_resume_animation,
                    "stru_3C9B0" | "stru_3C9CA"
                ));
            }
            if let Some(profile) = armadillo_init_animation {
                // ASM Armadillo.asm loc_3C7C0/loc_3C7DE selects the same patrol
                // script for HP >= 2, but high HP resumes damage with `stru_3C680`.
                debug_assert_eq!(profile.initial_animation_script, "stru_3C64C");
                debug_assert_eq!(profile.saved_patrol_animation, "stru_3C64C");
                debug_assert!(matches!(
                    profile.saved_damage_resume_animation,
                    "stru_3C666" | "stru_3C680"
                ));
            }
            if let Some(profile) = fire_demon_init_animation {
                // ASM Fire_Demon.asm loc_3A444/loc_3A464 keeps HP5 on the second
                // walking animation; lower tiers use the first walking animation.
                debug_assert_eq!(profile.collision_routine, "fire_demon_collision");
                debug_assert!(matches!(
                    profile.animation_script,
                    "stru_3A334" | "stru_3A31A"
                ));
                debug_assert_eq!(profile.saved_animation_script, profile.animation_script);
            }
            if matches!(family, EnemyFamily::Cloud) {
                // ASM Cloud.asm loc_35BAC/loc_35BB2 seeds the control stream as:
                // wait $78, set `$44`, wait $1E, clear `$44`, restart.
                let script_words = cloud_control_script_words_loc_35BAC();
                let first_wait = cloud_control_script_dispatch_loc_35BB2(script_words[0]);
                debug_assert_eq!(
                    first_wait,
                    CloudControlDispatchLoc35BB2::Wait { frames: 0x78 }
                );
                let wait_loop = cloud_wait_loop_loc_35C56(script_words[0]);
                debug_assert_eq!(wait_loop.dbf_seed, 0x78);
                debug_assert_eq!(wait_loop.hibernate_frames, 0x79);
                debug_assert!(wait_loop.resumes_control_script);

                debug_assert_eq!(
                    cloud_control_script_dispatch_loc_35BB2(script_words[1]),
                    CloudControlDispatchLoc35BB2::SetStatus44
                );
                let set_latch = cloud_status_latch_loc_35C72();
                debug_assert_eq!(set_latch.object_44_value, 0xFF);
                debug_assert!(set_latch.resumes_control_script);

                debug_assert_eq!(
                    cloud_control_script_dispatch_loc_35BB2(script_words[2]),
                    CloudControlDispatchLoc35BB2::Wait { frames: 0x1E }
                );
                debug_assert_eq!(
                    cloud_control_script_dispatch_loc_35BB2(script_words[3]),
                    CloudControlDispatchLoc35BB2::ClearStatus44
                );
                let clear_latch = cloud_status_latch_loc_35C7A();
                debug_assert_eq!(clear_latch.object_44_value, 0x00);
                debug_assert!(clear_latch.resumes_control_script);
                debug_assert_eq!(
                    cloud_control_script_dispatch_loc_35BB2(script_words[4]),
                    CloudControlDispatchLoc35BB2::RestartScript
                );
            }
            if matches!(family, EnemyFamily::Goat) {
                match goat_initial_dispatch_loc_3_cb02(facing) {
                    GoatInitialDispatchRouteLoc3CB02::ClearXDirection => {
                        debug_assert_eq!(facing, Direction::Left);
                    }
                    GoatInitialDispatchRouteLoc3CB02::FlippedXDirection => {
                        debug_assert_eq!(facing, Direction::Right);
                    }
                }
            }
            if matches!(family, EnemyFamily::DragonWalking) {
                let common_init = dragon_walking_common_init_loc_3ce5a(false, facing);
                debug_assert_eq!(common_init.collision_callback, "loc_3CE90");
                debug_assert_eq!(
                    common_init.next,
                    DragonWalkingInitialDispatchLoc3CE5A::NormalFacingDispatch
                );
                match dragon_walking_normal_dispatch_loc_3ce84(facing) {
                    DragonWalkingNormalDispatchLoc3CE84::Loc3E546 => {
                        debug_assert_eq!(facing, Direction::Left);
                    }
                    DragonWalkingNormalDispatchLoc3CE84::Loc3E392 => {
                        debug_assert_eq!(facing, Direction::Right);
                    }
                }
            }
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
                facing,
                on_ground: matches!(family.archetype(), EnemyArchetype::Pacer),
                hit_points,
                death_animation_active: false,
                grounded_frames: if matches!(family, EnemyFamily::DragonWalking) {
                    1
                } else {
                    0
                },
                tank_mouth_animation_phase: spawn_tank_mouth_animation_phase(family),
                damage_flash_frames: 0,
                palette_variant: resolved_enemy_palette(view.enemy_type_slots, object.object_type),
                wait_timer: orca_init.map_or_else(
                    || spawn_wait_timer(family, enemy_level, player_x, rng),
                    |init| init.initial_lunge_timer,
                ),
                dragon_phase: spawn_dragon_phase(family),
                dragon_phase_frames: 0,
                dragon_script_variant: dragon_variant,
                dragon_script_cmd: 0,
                dragon_anchor_y: if matches!(family, EnemyFamily::BigHoppingSkull) {
                    big_hopping_skull_pack_patrol_bounds_loc_364a0(
                        view.lower_bound,
                        view.upper_bound,
                    )
                } else {
                    head_y
                },
                dragon_descent_index: 0,
                dragon_descent_frames: 0,
                ninja_state: NinjaState::Stealth,
                hand_state: HandState::Walking,
                lion_state: spawn_lion_state_loc_3e030(family),
                robot_state: RobotState::Walking,
                orca_state: OrcaState::Walking,
                wall_pinned: matches!(family, EnemyFamily::FireDemon),
                drips_attached: false,
                drips_surface_table_index: 0,
                drips_pending_splash_x_pos: 0,
                drips_pending_splash_y_pos: 0,
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
                boss_eye_orbit_workers: if is_multi_head_boss(family) {
                    BossEyeOrbitWorkers::active_loc_38902_3899a(
                        (head_x >> 16) as u16,
                        (head_y >> 16) as u16,
                        0,
                        facing,
                    )
                } else {
                    BossEyeOrbitWorkers::default()
                },
                boss_movement_delay_counter: if is_multi_head_boss(family) {
                    1
                } else if matches!(family, EnemyFamily::HeadyMetal) {
                    HEADY_METAL_EYE_SPAWN_COUNTER_INIT_LOC_39264
                } else {
                    0
                },
                boss_movement_script_index: 0,
                heady_metal_palette_phase_counter: 0,
                heady_metal_palette_word_5e: 0x0200,
                scorpion_tracking_lock: false,
                scorpion_sting_frames: 0,
                crab_frozen: false,
                crab_berzerker_knockback: false,
                flight_double_acceleration: matches!(
                    family,
                    EnemyFamily::UFO | EnemyFamily::Tornado
                ) && enemy_level >= 1,
                flight_double_terminal_speed: matches!(
                    family,
                    EnemyFamily::UFO | EnemyFamily::Tornado
                ) && enemy_level >= 2,
                walk_speed_override: lion_motion_profile.map_or_else(
                    || orca_init.map_or(0, |init| init.walk_speed),
                    |init| init.walk_speed,
                ),
                shared_walker_state: spawn_shared_walker_state(family),
                shared_walker_patrol_timer: spawn_shared_walker_patrol_timer(family),
                shared_walker_turnaround_counter: 0,
                shared_walker_right_turn_extra_step_consumed:
                    spawn_shared_walker_right_turn_extra_step_consumed(family, facing),
                shared_walker_patrol_short_next: spawn_shared_walker_patrol_short_next(family),
                shared_walker_debug_branch: None,
                shared_walker_dx_to_kid_px: None,
                shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
            };
            if matches!(family, EnemyFamily::Diamond) && hit_points > 3 {
                // ASM Diamond.asm loc_3AD7C: HP above 3 bypasses the low-HP
                // shared-walker setup and seeds a Kid-facing physics velocity.
                let init =
                    diamond_high_hp_init_loc_3ad7c((head_x >> 16) as i32, (player_x >> 16) as i32);
                enemy.x_velocity = init.x_velocity;
                enemy.y_velocity = 0;
                enemy.on_ground = false;
            }
            if matches!(family, EnemyFamily::Tornado) {
                // ASM Tornado.asm seeds a movement pattern before entering the
                // per-frame loop, so activate the first script entry here.
                reload_tornado_pattern_script(&mut enemy);
            }
            if matches!(family, EnemyFamily::Dragon) {
                // ASM Dragon_Flying.asm Enemy0C_Dragon_flying_Init enters
                // loc_346C4 immediately, so the first script command's DBF
                // counter is already live before the first hibernated frame.
                dragon_restart_script(&mut enemy);
            }
            if matches!(family, EnemyFamily::BigHoppingSkull) {
                let startup = big_hopping_skull_startup_loc_3652e();
                debug_assert!(startup.hibernates_one_frame);
                debug_assert!(startup.checks_object_range);
                debug_assert_eq!(startup.animation, "stru_367BE");
            }
            active.push(enemy);
        }
    }
}

fn dragon_flying_dive_zero_contact_row_loc_348ce(x_velocity: i32, y_velocity: i32) -> bool {
    let x_px = x_velocity / 0x0001_0000;
    let y_px = y_velocity / 0x0001_0000;
    matches!(
        (x_px.abs(), y_px),
        // ASM Dragon_Flying.asm loc_348CE / off_345F6: `(2, 0)` selects
        // `LnkTo_unk_C75FE`; `(1, 1)` and `(2, 1..=3)` select
        // `LnkTo_unk_C7616`. All three route to `unk_31DE0`.
        (2, 0..=3) | (1, 1)
    )
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
    drilled_blocks: &mut Vec<DrilledBlockLoc3BE78>,
) {
    let mut rng = RomRandomState::default();
    step_active_enemies_with_rng(
        active,
        block_grid,
        player_x_pos,
        player_y_pos,
        None,
        None,
        player_facing,
        player_x_velocity,
        None,
        player_form,
        0,
        false,
        projectiles,
        &mut rng,
        drilled_blocks,
        None,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn step_active_enemies_with_rng(
    active: &mut [ActiveEnemy],
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    camera_x_px: Option<u32>,
    camera_y_px: Option<u32>,
    player_facing: Option<Direction>,
    player_x_velocity: Option<i32>,
    player_y_velocity: Option<i32>,
    player_form: PlayerForm,
    time_frames: u16,
    samurai_haze_active: bool,
    mut projectiles: Option<&mut Vec<EnemyProjectile>>,
    rng: &mut RomRandomState,
    drilled_blocks: &mut Vec<DrilledBlockLoc3BE78>,
    mut played_sound_effects: Option<&mut Vec<QueuedSoundEffect>>,
) {
    let mut drill_drive_refcount = active
        .iter()
        .filter(|enemy| {
            matches!(enemy.family, EnemyFamily::Driller) && enemy.shared_walker_patrol_short_next
        })
        .count()
        .min(usize::from(u8::MAX)) as u8;
    let mut tank_drive_refcount = active
        .iter()
        .filter(|enemy| {
            matches!(enemy.family, EnemyFamily::Tank | EnemyFamily::TankShooting)
                && enemy.shared_walker_patrol_short_next
        })
        .count()
        .min(usize::from(u8::MAX)) as u8;
    for enemy in active.iter_mut() {
        if matches!(enemy.family, EnemyFamily::Driller) && !enemy.shared_walker_patrol_short_next {
            // ASM Driller.asm loc_3BE60: increment the shared DrillMoving
            // refcount, install loc_3BE78 as the callback, then enter the
            // direction-specific shared walker loop.
            let x_direction_set = matches!(enemy.facing, Direction::Right);
            let init_tail = driller_init_tail_loc_3be60(x_direction_set, drill_drive_refcount);
            debug_assert_eq!(
                init_tail.next_drill_refcount,
                drill_drive_refcount.wrapping_add(1)
            );
            debug_assert_eq!(init_tail.callback, "loc_3BE78");
            let expected_route = if x_direction_set {
                DrillerInitTailRouteLoc3BE60::SetDirectionLoc3C026
            } else {
                DrillerInitTailRouteLoc3BE60::ClearDirectionLoc3C0D2
            };
            debug_assert_eq!(init_tail.next_route, expected_route);
            drill_drive_refcount = init_tail.next_drill_refcount;
            enemy.shared_walker_patrol_short_next = true;
        }
        if matches!(enemy.family, EnemyFamily::Tank | EnemyFamily::TankShooting)
            && !enemy.shared_walker_patrol_short_next
        {
            // ASM Tank.asm loc_3B40A / Tank_Shooting.asm loc_3B690: the
            // visible init tail plays `sfx_Tank_driving` only for the first
            // Rock Tank sharing the `$FFFFFA34+object_meta` refcount, then
            // increments that counter before entering loc_3C026/loc_3C0D2.
            // Native reuses `shared_walker_patrol_short_next` as this
            // Tank-only init latch; snapshots only expose that byte for
            // DragonWalking, where it retains its cadence meaning.
            let x_direction = if matches!(enemy.facing, Direction::Right) {
                0xFF
            } else {
                0
            };
            let result = if matches!(enemy.family, EnemyFamily::Tank) {
                tank_init_drive_sound_loc_3b40a(tank_drive_refcount, false, x_direction)
            } else {
                let result = tank_shooting_init_drive_sound_loc_3b690(
                    tank_drive_refcount,
                    false,
                    x_direction,
                );
                TankInitDriveSoundLoc3B40A {
                    next_tank_drive_refcount: result.next_tank_drive_refcount,
                    sound_call: result.sound_call,
                    walker_entry: match result.walker_entry {
                        TankShootingInitWalkerEntryLoc3B690::LeftLoc3C0D2 => {
                            TankInitWalkerEntryLoc3B40A::LeftLoc3C0D2
                        }
                        TankShootingInitWalkerEntryLoc3B690::RightLoc3C026 => {
                            TankInitWalkerEntryLoc3B40A::RightLoc3C026
                        }
                    },
                }
            };
            tank_drive_refcount = result.next_tank_drive_refcount;
            enemy.shared_walker_patrol_short_next = true;
            if matches!(
                result.sound_call,
                Some(SharedEnemyDriveSoundCallSub3C352::StartSequence(
                    SharedEnemyDriveSoundEffectSub3C352::TankDriving
                ))
            ) && let Some(sound_effects) = played_sound_effects.as_deref_mut()
            {
                sound_effects.push(QueuedSoundEffect::TankDriving);
            }
        }
        // ASM loc_E1E: enemies allocated through loc_3691E have `$10(a5)` set,
        // so Samurai Haze skips their object execution on 3 of every 4 frames.
        if samurai_haze_active && time_frames & 3 != 0 {
            continue;
        }
        enemy.damage_flash_frames = enemy.damage_flash_frames.saturating_sub(1);
        if enemy.death_animation_active && enemy_death_animation_frames(enemy.family).is_some() {
            // ASM family death tails such as Goat.asm loc_3CB64 and
            // Mini_Hopping_Skull.asm loc_3F158 clear movement while the death
            // animation hibernates, so the corpse should not run its normal
            // movement step.
            continue;
        }
        match enemy.family.archetype() {
            EnemyArchetype::Inert => {}
            EnemyArchetype::Bouncer => {
                if matches!(enemy.family, EnemyFamily::BigHoppingSkull) {
                    if enemy.grounded_frames > 0 {
                        enemy.grounded_frames = enemy.grounded_frames.saturating_sub(1);
                        let elapsed = BIG_HOPPING_SKULL_HIT_ANIMATION_FRAMES_LOC_365D8
                            .saturating_sub(enemy.grounded_frames);
                        if let Some(sound_effects) = played_sound_effects.as_deref_mut() {
                            // ASM Big_Hopping_Skull.asm loc_365F2/loc_36616:
                            // after each hit-animation hibernate, play the
                            // groan on `C7FB0` timer 6 and Voice_die on
                            // `C7FE0` timer $C.
                            let groan_gate = big_hopping_skull_hit_groan_gate_loc_365f2(
                                BIG_HOPPING_SKULL_HIT_GROAN_ELAPSED_FRAMES_LOC_365F2
                                    .contains(&elapsed),
                                6,
                            );
                            debug_assert!(groan_gate.hibernates_one_frame);
                            debug_assert!(groan_gate.checks_object_range);
                            debug_assert!(groan_gate.d0_preserved);
                            if groan_gate.sound_effect.is_some() {
                                sound_effects.push(QueuedSoundEffect::BigHoppingSkullGroan);
                            }
                            let voice_gate = big_hopping_skull_hit_voice_gate_loc_36616(
                                elapsed == BIG_HOPPING_SKULL_HIT_VOICE_ELAPSED_FRAMES_LOC_36616,
                                0x000C,
                            );
                            debug_assert!(voice_gate.d0_preserved);
                            if voice_gate.sound_effect.is_some() {
                                sound_effects.push(QueuedSoundEffect::VoiceDie);
                            }
                        }
                        let hit_wait = big_hopping_skull_hit_animation_wait_loc_36632(
                            true,
                            enemy.grounded_frames == 0,
                        );
                        debug_assert_eq!(hit_wait.collision_probe_extent, 8);
                        match hit_wait.route {
                            BigHoppingSkullHitAnimationWaitRouteLoc36632::LoopHitAnimation => {
                                debug_assert!(!hit_wait.clear_has_kid_collision);
                            }
                            BigHoppingSkullHitAnimationWaitRouteLoc36632::SharedTail => {
                                debug_assert!(hit_wait.clear_has_kid_collision);
                            }
                            BigHoppingSkullHitAnimationWaitRouteLoc36632::Relaunch => {
                                debug_assert!(
                                    false,
                                    "native Big Hopping Skull hit wait has no live loc_33286 clear-probe signal"
                                );
                            }
                        }
                        continue;
                    }
                    let (left_bound_word, right_bound_word) =
                        big_hopping_skull_unpack_patrol_bounds_loc_364a0(enemy.dragon_anchor_y);
                    step_big_hopping_skull_with_bounds(
                        enemy,
                        block_grid,
                        left_bound_word,
                        right_bound_word,
                    );
                } else {
                    step_bouncer_with_rng(
                        enemy,
                        block_grid,
                        camera_x_px,
                        rng,
                        projectiles.as_deref_mut(),
                        played_sound_effects.as_deref_mut(),
                    );
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
                    step_flyer_with_rng(enemy, block_grid, player_x_pos, player_y_pos, rng);
                    if matches!(enemy.family, EnemyFamily::Cloud)
                        && let Some(ref mut projs) = projectiles
                    {
                        step_cloud_attack(enemy, projs, player_x_pos, player_y_pos);
                    }
                    if matches!(enemy.family, EnemyFamily::UFO)
                        && let Some(ref mut projs) = projectiles
                    {
                        step_ufo_beam_with_rng(
                            enemy,
                            projs,
                            player_x_pos,
                            player_y_pos,
                            rng,
                            played_sound_effects.as_deref_mut(),
                        );
                    }
                }
            }
            EnemyArchetype::Hopper => {
                if matches!(enemy.family, EnemyFamily::MiniHoppingSkull)
                    && enemy.wait_timer == MINI_HOPPING_SKULL_AIR_DAMAGE_WAIT_TIMER_LOC_3F200
                {
                    step_mini_hopping_skull_air_damage_loop(enemy, block_grid);
                } else if matches!(enemy.family, EnemyFamily::MiniHoppingSkull)
                    && enemy.wait_timer
                        == MINI_HOPPING_SKULL_CHARGING_BERZERKER_WAIT_TIMER_LOC_3F288
                {
                    step_mini_hopping_skull_charging_berzerker_loop(enemy);
                } else {
                    step_hopper(enemy, block_grid, player_x_pos);
                }
            }
            EnemyArchetype::Hoverer => step_emo_rock_with_rng(
                enemy,
                block_grid,
                player_x_pos,
                player_y_pos,
                rng,
                played_sound_effects.as_deref_mut(),
            ),
            EnemyArchetype::Pacer => {
                if matches!(enemy.family, EnemyFamily::Ninja) {
                    if enemy.wait_timer == NINJA_BERZERKER_SIDE_STUN_WAIT_TIMER_LOC_3D6C8 {
                        step_ninja_berzerker_side_stun(enemy);
                    } else if enemy.wait_timer == NINJA_BERZERKER_SIDE_LAUNCH_WAIT_TIMER_LOC_3DB3C {
                        step_ninja_berzerker_side_launch(enemy);
                    } else {
                        let maniaxe_projectile_dx =
                            projectiles.as_deref().and_then(|projectiles| {
                                ninja_maniaxe_projectile_dx(projectiles, enemy)
                            });
                        step_ninja(
                            enemy,
                            block_grid,
                            player_x_pos,
                            player_y_pos,
                            player_form,
                            maniaxe_projectile_dx,
                        );
                    }
                } else if matches!(enemy.family, EnemyFamily::Hand) {
                    step_hand(enemy, block_grid, player_x_pos, player_y_pos, player_facing);
                } else if matches!(enemy.family, EnemyFamily::Lion) {
                    if enemy.damage_flash_frames > 0 {
                        // ASM Lion.asm loc_3E32C hibernates through
                        // `stru_3DF42`, then restores `$62(a5)` and resumes
                        // through loc_3E356 without advancing the charge loop.
                        continue;
                    }
                    step_lion(enemy, block_grid, player_x_pos, &mut projectiles);
                } else if matches!(enemy.family, EnemyFamily::Robot) {
                    step_robot_with_rng(
                        enemy,
                        block_grid,
                        player_x_pos,
                        &mut projectiles,
                        rng,
                        played_sound_effects.as_deref_mut(),
                    );
                } else if matches!(enemy.family, EnemyFamily::Orca) {
                    if enemy.wait_timer == ORCA_BERZERKER_KNOCKBACK_WAIT_TIMER_LOC_3D2AA {
                        step_orca_berzerker_knockback(enemy);
                    } else {
                        step_orca(enemy, block_grid, player_x_pos);
                    }
                } else if matches!(enemy.family, EnemyFamily::SpinningTwins) {
                    if enemy.wait_timer == SPINNING_TWINS_BERZERKER_LAUNCH_WAIT_TIMER_LOC_3BC22 {
                        step_spinning_twins_berzerker_launch(enemy);
                    } else if matches!(enemy.spinning_twins_phase, SpinningTwinsPhase::Walking)
                        && enemy.wait_timer > 0
                    {
                        // ASM Spinning_Twins.asm loc_3BC34: the settled hurt
                        // path hibernates until `stru_3B836` finishes before
                        // returning to the shared walker.
                        enemy.wait_timer = enemy.wait_timer.saturating_sub(1);
                    } else {
                        step_spinning_twins(enemy, block_grid, player_x_pos, player_y_pos);
                    }
                } else if matches!(enemy.family, EnemyFamily::DragonWalking) {
                    if enemy.wait_timer == DRAGON_WALKING_BERZERKER_KNOCKBACK_WAIT_TIMER_LOC_3CF3A {
                        step_dragon_walking_berzerker_knockback(enemy);
                        continue;
                    }
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
                    if matches!(enemy.family, EnemyFamily::Tank) && enemy.wait_timer > 0 {
                        // ASM Tank.asm loc_3B49A hibernates the parent after
                        // a nonfatal child hit before resuming the shared
                        // walker; native `wait_timer` spans the ROM hibernate
                        // plus trace-sampling collision frames.
                        if enemy.wait_timer == 1 {
                            // ASM Tank.asm loc_3B4BA: clear the linked mouth
                            // collision and branch by `x_direction` into the
                            // shared walker.
                            let x_direction = if matches!(enemy.facing, Direction::Right) {
                                0xFF
                            } else {
                                0x00
                            };
                            let resume = tank_resume_walk_loc_3b4ba(x_direction);
                            debug_assert_eq!(resume.next_child_collision_type, 0);
                            let expected_branch = if matches!(enemy.facing, Direction::Right) {
                                TankResumeWalkBranchLoc3B4BA::RightSharedWalker
                            } else {
                                TankResumeWalkBranchLoc3B4BA::LeftSharedWalker
                            };
                            debug_assert_eq!(resume.branch, expected_branch);
                        }
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.wait_timer = enemy.wait_timer.saturating_sub(1);
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::TankShooting)
                        && tank_shooting_hurt_hibernate_frames_remaining(enemy.damage_flash_frames)
                            > 0
                    {
                        // ASM Tank_Shooting.asm loc_3B700 hibernates the parent
                        // for five frames after nonfatal child/projectile hits.
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        continue;
                    }
                    // Crab: player-proximity freeze callback (Crab.asm loc_3B0BE-loc_3B174).
                    // Check freeze before stepping; if frozen, skip movement.
                    if matches!(enemy.family, EnemyFamily::Crab) {
                        if enemy.crab_berzerker_knockback {
                            step_crab_berzerker_knockback(enemy, block_grid);
                            continue;
                        }
                        if enemy.damage_flash_frames > 0 {
                            // ASM Crab.asm loc_3B0A6 initialises the hurt
                            // animation and hibernates until it finishes before
                            // restoring `stru_3AF62`; do not keep pacing while
                            // the native damage-flash window represents that hold.
                            continue;
                        }
                        step_crab_freeze(enemy, player_x_pos, player_facing, player_x_velocity);
                        if enemy.crab_frozen {
                            continue; // skip pacer walk while frozen
                        }
                    }
                    if matches!(enemy.family, EnemyFamily::FireDemon)
                        && enemy.wait_timer == FIRE_DEMON_BERZERKER_LAUNCH_WAIT_TIMER_LOC_3A610
                    {
                        step_fire_demon_berzerker_launch(enemy);
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::FireDemon) && enemy.wall_pinned {
                        // ASM Fire_Demon.asm Enemy00_FireDemon_Init first calls
                        // j_Hibernate_Object_1Frame before loading/copying the
                        // visible gfx object. The first shared walker entry
                        // (kid.asm loc_3C0F0) hibernates once more before
                        // loc_3C0FA can move left, so native must expose one
                        // hidden frame followed by one stationary visible frame.
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        if enemy.grounded_frames == 0 {
                            enemy.grounded_frames = 1;
                        } else {
                            enemy.grounded_frames = 0;
                            enemy.wall_pinned = false;
                        }
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::FireDemon)
                        && enemy.damage_flash_frames > 0
                    {
                        // ASM Fire_Demon.asm loc_3A5BE hibernates on
                        // `stru_3A34E` after a nonfatal hurt collision before
                        // restoring the saved walk animation.
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::Goat)
                        && enemy.wait_timer == GOAT_BERZERKER_KNOCKBACK_WAIT_TIMER_LOC_3CBA8
                    {
                        step_goat_berzerker_knockback(enemy);
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::Goat) && enemy.damage_flash_frames > 0 {
                        // ASM Goat.asm loc_3CB38 plays `stru_3C9E4` and
                        // hibernates before restoring the saved patrol
                        // animation and resuming the shared walker.
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::Armadillo)
                        && enemy.wait_timer == ARMADILLO_BERZERKER_LAUNCH_WAIT_TIMER_LOC_3C8CC
                    {
                        step_armadillo_berzerker_launch(enemy);
                        continue;
                    }
                    if enemy.is_armadillo_spawned_child_fall_waiting() {
                        step_armadillo_spawned_child_fall_wait_loc_3c954(enemy, block_grid);
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::Scorpion)
                        && enemy.wait_timer == SCORPION_BERZERKER_KNOCKBACK_WAIT_TIMER_LOC_3DD4A
                    {
                        step_scorpion_berzerker_knockback(enemy);
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::Scorpion)
                        && enemy.damage_flash_frames > 0
                    {
                        // ASM Scorpion.asm loc_3DCD4 plays `stru_3DB7A` and
                        // hibernates until it finishes, then restores
                        // `stru_3DB60` before resuming the shared walker.
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::Scorpion)
                        && enemy.scorpion_sting_frames > 0
                    {
                        // ASM Scorpion.asm loc_3DDEE runs `stru_3DB8E` and
                        // hibernates until the animation completes; loc_3DE40
                        // then clears the linked tail collision flag and
                        // resumes the shared-walker loop.
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        enemy.scorpion_sting_frames = enemy.scorpion_sting_frames.saturating_sub(1);
                        if enemy.scorpion_sting_frames == 0 {
                            let recovery = scorpion_sting_recovery_loc_3de40(
                                pacer_walk_speed(EnemyFamily::Scorpion, enemy.hit_points),
                                0xC000 | u16::from(EnemyFamily::Scorpion.object_type()),
                            );
                            if cfg!(debug_assertions) {
                                assert!(
                                    !recovery.linked_tail_has_kid_collision,
                                    "ASM Scorpion.asm loc_3DE40 clears linked tail `$3D`",
                                );
                                assert_eq!(
                                    recovery.sprite_mapping, "LnkTo_unk_C7DDE-Data_Index",
                                    "ASM Scorpion.asm loc_3DE40 restores base sprite mapping",
                                );
                                assert_eq!(
                                    recovery.animation_script, "stru_3DB60",
                                    "ASM Scorpion.asm loc_3DE40 restores base animation",
                                );
                                assert_eq!(
                                    recovery.object_meta,
                                    u16::from(EnemyFamily::Scorpion.object_type()),
                                    "ASM Scorpion.asm loc_3DE40 masks high object_meta bits",
                                );
                            }
                            enemy.walk_speed_override = recovery.walk_speed_override;
                        }
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::Diamond)
                        && enemy.wait_timer == DIAMOND_HIGH_HP_BERZERKER_LAUNCH_WAIT_TIMER_LOC_3AEB2
                    {
                        step_diamond_high_hp_berzerker_launch(enemy);
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::Diamond)
                        && enemy.wait_timer == DIAMOND_LOW_HP_BERZERKER_LAUNCH_WAIT_TIMER_LOC_3AF50
                    {
                        step_diamond_low_hp_berzerker_launch(enemy);
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::Diamond) && enemy.damage_flash_frames > 0
                    {
                        // ASM Diamond.asm loc_3ADFA/loc_3AEF8 hibernates on
                        // `stru_3ACBE` after nonfatal hurt collisions before
                        // restoring the live Diamond animation.
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::Diamond) && enemy.hit_points > 3 {
                        // ASM Diamond.asm loc_3ADA4: high-HP Diamonds run
                        // their physics-ball collision loop, not loc_3AF14.
                        step_diamond_high_hp(enemy, block_grid);
                        continue;
                    }
                    if matches!(enemy.family, EnemyFamily::Tank) {
                        let mouth_step =
                            tank_mouth_animation_step_loc_3b27e(enemy.tank_mouth_animation_phase);
                        enemy.tank_mouth_animation_phase = mouth_step.next_animation_phase;
                        let route = tank_update_loc_3b41c(
                            0,
                            0,
                            u16::from(enemy.hit_points),
                            mouth_step.mouth_frame,
                            false,
                        );
                        if let Some(mouth_sound) = route.mouth_sound
                            && let Some(sound_effects) = played_sound_effects.as_deref_mut()
                        {
                            sound_effects.push(mouth_sound.queued_sound_effect());
                        }
                    }
                    step_pacer(enemy, block_grid);
                    // Goat: player-tracking callback (Goat.asm loc_3CC18).
                    // Positive `$42(a3)` resumes through loc_3CC52; otherwise
                    // if player is >32px away, turn to face them.
                    if matches!(enemy.family, EnemyFamily::Goat)
                        && let Some(px) = player_x_pos
                    {
                        match goat_tracking_route_loc_3cc18(
                            enemy.shared_walker_state,
                            (enemy.x_pos >> 16) as i16,
                            (px >> 16) as i16,
                            enemy.facing,
                        ) {
                            GoatTrackingRouteLoc3CC18::Right(
                                GoatTrackingRightRouteLoc3CC36::TurnRight,
                            ) => enemy.facing = Direction::Right,
                            GoatTrackingRouteLoc3CC18::Left(
                                GoatTrackingLeftRouteLoc3CC46::TurnLeft,
                            ) => enemy.facing = Direction::Left,
                            GoatTrackingRouteLoc3CC18::Resume(_)
                            | GoatTrackingRouteLoc3CC18::Right(
                                GoatTrackingRightRouteLoc3CC36::AlreadyFacingRight,
                            )
                            | GoatTrackingRouteLoc3CC18::Left(
                                GoatTrackingLeftRouteLoc3CC46::AlreadyFacingLeft,
                            ) => {}
                        }
                    }
                    // Scorpion: tracking callback (Scorpion.asm loc_3DD7E).
                    // Within 170px: face toward player (unless tracking-locked),
                    // boost speed to $38000 if enemy_level > 1 (HP > 3).
                    // Outside 170px: reset speed to base $18000, clear tracking lock
                    // if player is airborne.
                    if matches!(enemy.family, EnemyFamily::Scorpion) {
                        step_scorpion_tracking(
                            enemy,
                            player_x_pos,
                            player_y_pos,
                            player_y_velocity,
                        );
                    }
                    // ASM Tank_Shooting.asm loc_3C50E/loc_3B730: only the
                    // shooting tank variant emits bullets, and only on the
                    // turret animation gate.
                    if matches!(enemy.family, EnemyFamily::TankShooting)
                        && let Some(ref mut projs) = projectiles
                    {
                        step_tank_shooting_fire(enemy, projs, played_sound_effects.as_deref_mut());
                    }
                }
            }
            EnemyArchetype::Stationary => {
                if matches!(enemy.family, EnemyFamily::TarMonster) {
                    let mut discarded_sound_effects = Vec::new();
                    let tar_monster_sound_effects = played_sound_effects
                        .as_deref_mut()
                        .unwrap_or(&mut discarded_sound_effects);
                    if let Some(ref mut projs) = projectiles {
                        step_tar_monster_with_sounds(enemy, projs, tar_monster_sound_effects);
                    } else {
                        step_tar_monster_with_sounds(
                            enemy,
                            &mut Vec::new(),
                            tar_monster_sound_effects,
                        );
                    }
                } else if matches!(enemy.family, EnemyFamily::Drips) {
                    if let Some(projs) = projectiles.as_deref_mut()
                        && drips_object_check_in_range_loc_333f6(enemy, camera_x_px, camera_y_px)
                    {
                        step_drips(enemy, projs, block_grid);
                    }
                } else if let Some(ref mut projs) = projectiles {
                    step_archer(
                        enemy,
                        projs,
                        player_x_pos,
                        player_y_pos,
                        played_sound_effects.as_deref_mut(),
                    );
                }
            }
            EnemyArchetype::Tracker => {
                if is_multi_head_boss(enemy.family) {
                    step_multi_head_boss_movement(enemy);
                } else {
                    step_tracker(enemy, player_x_pos, player_y_pos);
                }
                // Bosses periodically spawn projectiles while tracking
                if let Some(ref mut projs) = projectiles {
                    step_boss_fire_with_sounds(
                        enemy,
                        projs,
                        player_x_pos,
                        player_y_pos,
                        played_sound_effects.as_deref_mut(),
                    );
                }
            }
        }
    }
    seed_pending_drips_initial_waits_loc_3340e(active, player_x_pos, rng);
}

/// Draw the deferred ASM `Drips.asm loc_3340E` initial dormant seeds for every
/// Drips that attached to the ceiling this frame.
///
/// ASM `Allocate_ObjectSlot` (kid.asm loc_DE0) walks the execution list with
/// `cmp.w $E(a1),d6; bhi.s loc_DE0` — a strict unsigned compare — so an
/// equal-priority object is inserted BEFORE existing slots and
/// `Execute_Objects` (loc_E1E) runs simultaneously loaded enemies in reverse
/// load order. `Manage_EnemyLoading` loads layout entries ascending, so
/// same-frame Drips attaches consume `j_Get_RandomNumber_byte` in descending
/// (segment, layout, x) order. The issue-260 oracle pair confirms this: the
/// x=584 Drips holds the first draw and the x=456 Drips the second (the same
/// ordering the loc_33488 splash-continuation pass already uses).
pub(crate) fn seed_pending_drips_initial_waits_loc_3340e(
    active: &mut [ActiveEnemy],
    player_x_pos: Option<u32>,
    rng: &mut RomRandomState,
) {
    let mut pending_indices: Vec<usize> = active
        .iter()
        .enumerate()
        .filter(|(_, enemy)| {
            enemy.family == EnemyFamily::Drips
                && enemy.wait_timer == DRIPS_INITIAL_SEED_PENDING_TIMER_LOC_3340E
        })
        .map(|(index, _)| index)
        .collect();
    pending_indices.sort_by(|&left, &right| {
        let left_enemy = &active[left];
        let right_enemy = &active[right];
        (
            right_enemy.segment_index,
            right_enemy.layout_index,
            right_enemy.x_pos,
        )
            .cmp(&(
                left_enemy.segment_index,
                left_enemy.layout_index,
                left_enemy.x_pos,
            ))
    });
    for index in pending_indices {
        let enemy = &mut active[index];
        enemy.wait_timer =
            drips_wait_timer_with_rng_loc_3340e(rng, player_x_pos.unwrap_or(enemy.x_pos));
    }
}

pub(crate) fn step_flyer_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    rng: &mut RomRandomState,
) {
    // Cloud: horizontal flight with wall reversal
    if matches!(enemy.family, EnemyFamily::Cloud) {
        step_cloud(enemy, block_grid);
        return;
    }

    // UFO: acceleration-based drifting with periodic direction changes
    if matches!(enemy.family, EnemyFamily::UFO) {
        step_ufo_with_player(enemy, block_grid, player_x_pos, player_y_pos, rng);
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

fn fixed_integer_word(pos: u32) -> i16 {
    (pos >> 16) as u16 as i16
}

/// Tornado: acceleration-based flight. ASM UFO.asm loc_35326/unk_35344/loc_35444
/// shares `Enemy_HandleAcceleration` with UFO. Terminal velocity from the table
/// entries is typically $18000. Cycle timer from `unk_3588C`: 60/40/30 by enemy_level.
pub(crate) const TORNADO_ACCELERATION: i32 = 0xC00;
/// Per-level cycle timers: level 0 = 60, level 1 = 40, level 2 = 30 (unk_3588C).
pub(crate) const TORNADO_PATTERN_FRAMES: [u16; 3] = [60, 40, 30];
const TORNADO_LIVE_ANIMATION_FRAME_TICKS: u8 = 5;
const TORNADO_LIVE_ANIMATION_FRAMES: u8 = 3;
const TORNADO_LIVE_ANIMATION_TICKS: u8 =
    TORNADO_LIVE_ANIMATION_FRAME_TICKS * TORNADO_LIVE_ANIMATION_FRAMES;

pub(crate) fn tornado_live_zero_contact_row_stru_35b10(animation_tick: u8) -> bool {
    // ASM Tornado.asm `stru_35B10`: C784E is the active row for 5 ticks,
    // then C7856/C785E are zero rows for the next 10 ticks before looping.
    animation_tick % TORNADO_LIVE_ANIMATION_TICKS >= TORNADO_LIVE_ANIMATION_FRAME_TICKS
}

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
        // ASM UFO.asm loc_35452/loc_35496: zero accel entry means `$C00` toward zero.
        return if current_velocity < 0 {
            TORNADO_ACCELERATION
        } else {
            -TORNADO_ACCELERATION
        };
    }

    let mut acceleration = base_acceleration;
    if current_velocity >= 0 {
        // ASM UFO.asm loc_3546E/loc_354B0: non-negative velocity keeps acceleration unless a positive
        // target speed is already below the current velocity, where the acceleration register is negated.
        if target_velocity >= 0 && target_velocity < current_velocity {
            acceleration = -acceleration;
        }
    } else {
        // ASM UFO.asm loc_35488/loc_354CA: negative velocity keeps acceleration for non-negative targets;
        // only negate it when a negative target is above the current velocity.
        if target_velocity < 0 && target_velocity > current_velocity {
            acceleration = -acceleration;
        }
    }
    acceleration
}

fn apply_acceleration_axis(current_velocity: i32, target_velocity: i32, acceleration: i32) -> i32 {
    if acceleration >= 0 {
        ufo_apply_acceleration_axis_loc_3502a(current_velocity, target_velocity, acceleration)
    } else {
        ufo_apply_acceleration_axis_loc_3501e(current_velocity, target_velocity, acceleration)
    }
}

pub(crate) fn ufo_apply_acceleration_axis_loc_3502a(
    current_velocity: i32,
    target_velocity: i32,
    acceleration: i32,
) -> i32 {
    // ASM UFO.asm loc_3502A: add positive acceleration and cap to desired axis speed.
    let next_velocity = current_velocity.saturating_add(acceleration);
    ufo_apply_acceleration_axis_loc_35050(next_velocity.min(target_velocity))
}

pub(crate) fn ufo_apply_acceleration_axis_loc_3501e(
    current_velocity: i32,
    target_velocity: i32,
    acceleration: i32,
) -> i32 {
    // ASM UFO.asm loc_3501E: negative-branch accelerant path in
    // `Enemy_HandleAcceleration` follows `loc_35044`.
    ufo_apply_acceleration_axis_loc_35044(current_velocity, target_velocity, acceleration)
}

pub(crate) fn ufo_apply_acceleration_axis_loc_35044(
    current_velocity: i32,
    target_velocity: i32,
    acceleration: i32,
) -> i32 {
    // ASM UFO.asm loc_35044: keep the accelerated value while it stays above the target.
    let next_velocity = current_velocity.saturating_add(acceleration);
    if next_velocity > target_velocity {
        ufo_apply_acceleration_axis_loc_35050(next_velocity)
    } else {
        ufo_apply_acceleration_axis_loc_35050(target_velocity)
    }
}

pub(crate) fn ufo_apply_acceleration_axis_loc_35050(final_velocity: i32) -> i32 {
    // ASM UFO.asm loc_35050: write the chosen `d7` into Y velocity and return.
    final_velocity
}

fn swap_words_i32(value: i32) -> i32 {
    let bits = value as u32;
    bits.rotate_right(16) as i32
}

pub(crate) fn reload_tornado_pattern_script(enemy: &mut ActiveEnemy) {
    // ASM Tornado.asm loc_359F2 -> loc_35A02 / unk_35A0C: use the current
    // 16-byte motion-script entry, advance the script pointer, and wrap when
    // the following longword sentinel equals 1.
    let index = usize::from(enemy.dragon_script_cmd) % TORNADO_PATTERN_SCRIPT.len();
    let pattern = TORNADO_PATTERN_SCRIPT[index];
    enemy.dragon_script_cmd = ((index + 1) % TORNADO_PATTERN_SCRIPT.len()) as u8;
    let target_x_velocity = if enemy.flight_double_acceleration {
        // ASM Tornado.asm loc_3592E: non-zero `$46(a5)` doubles `d0` and `d2`,
        // the X target speed and X acceleration loaded from the script entry.
        pattern.target_x_velocity.saturating_mul(2)
    } else {
        pattern.target_x_velocity
    };
    let target_y_velocity = if enemy.flight_double_terminal_speed {
        // ASM Tornado.asm loc_3593A: non-zero `$47(a5)` doubles `d1` and `d3`,
        // the Y target speed and Y acceleration loaded from the script entry.
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
    enemy.y_acceleration = if enemy.flight_double_terminal_speed {
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

/// Sphere bounces off walls/floors/ceilings with gravity.
/// Sphere.asm: gravity +$800/frame, wall→negate x_vel, floor→negate y_vel,
/// ceiling→negate y_vel and halve.
pub(crate) const SPHERE_GRAVITY: i32 = 0x800;

/// Fireball max upward velocity indexed by live HP
/// (Fireball.asm `Enemy19_Fireball_Init`: `move.w 2(a0),d7; asl.w #3,d7` into `unk_32BA6`).
/// HP 1/2/3 use the second/third/final table entries because spawn HP already includes the +1 base.
pub(crate) const FIREBALL_UPWARD_CAP_BY_HP: [i32; 3] = [-0x4_0000, -0x5_0000, -0x5_0000];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballTrailInitLoc330D0 {
    pub(crate) gfx_object_long: u32,
    pub(crate) display_flag_13: u8,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) lifetime_frames: u16,
    pub(crate) x_direction: u8,
    pub(crate) x_px: i16,
    pub(crate) y_px: i16,
    pub(crate) animation: &'static str,
}

pub(crate) fn fireball_trail_init_loc_330d0(
    parent_x_direction: u8,
    parent_x_px: i16,
    parent_y_px: i16,
) -> FireballTrailInitLoc330D0 {
    // ASM Fireball.asm loc_330D0: allocate GfxObject `$03000003`, mark the
    // child visible, seed Fireball object metadata/collision, copy `$44/$46/$48(a5)`
    // into direction/x/y, set `d3=$96`, then start `stru_330C2`.
    FireballTrailInitLoc330D0 {
        gfx_object_long: 0x0300_0003,
        display_flag_13: 0xFF,
        priority: 0,
        object_meta: 0x19,
        lifetime_frames: 0x96,
        x_direction: parent_x_direction,
        x_px: parent_x_px,
        y_px: parent_y_px,
        animation: "stru_330C2",
    }
}

fn signed_word(word: u16) -> i16 {
    i16::from_be_bytes(word.to_be_bytes())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballBounceCommitLoc32CE4 {
    pub(crate) y_velocity: i32,
    pub(crate) spawns_fire_trail: bool,
    pub(crate) next_fire_trail_count: u16,
}

pub(crate) fn fireball_floor_ceiling_bounce_loc_32ce4(
    enemy: &ActiveEnemy,
    rng: &mut RomRandomState,
    camera_x_px: u32,
    fire_trail_count: u16,
) -> FireballBounceCommitLoc32CE4 {
    // ASM Fireball.asm loc_32CE4: commit d7 into `y_vel(a3)`, then allocate
    // `loc_33128` only while `Number_of_Fire_Trails` remains below five.
    let y_velocity = fireball_bounce_y_velocity_with_rng(enemy, rng, camera_x_px);
    let spawns_fire_trail = fire_trail_count < 5;
    FireballBounceCommitLoc32CE4 {
        y_velocity,
        spawns_fire_trail,
        next_fire_trail_count: if spawns_fire_trail {
            fire_trail_count.saturating_add(1)
        } else {
            fire_trail_count
        },
    }
}

fn count_live_fireball_trails(projectiles: &[EnemyProjectile]) -> u16 {
    projectiles
        .iter()
        .filter(|projectile| {
            matches!(
                projectile.kind,
                EnemyProjectileKind::FireballTrail | EnemyProjectileKind::FireballObstacleTrail
            )
        })
        .count()
        .min(usize::from(u16::MAX)) as u16
}

fn fireball_trail_projectile_from_loc_330d0(
    parent_x_direction: u8,
    x_px: i16,
    y_px: i16,
) -> EnemyProjectile {
    let init = fireball_trail_init_loc_330d0(parent_x_direction, x_px, y_px);
    debug_assert_eq!(init.gfx_object_long, 0x0300_0003);
    debug_assert_eq!(init.display_flag_13, 0xFF);
    debug_assert_eq!(init.object_meta, 0x19);
    debug_assert_eq!(init.animation, "stru_330C2");
    EnemyProjectile {
        x_pos: (i32::from(init.x_px) as u32) << 16,
        y_pos: (i32::from(init.y_px) as u32) << 16,
        x_velocity: 0,
        y_velocity: 0,
        lifetime: init.lifetime_frames,
        gravity: 0,
        friendly: false,
        terrain_kill: false,
        kind: EnemyProjectileKind::FireballTrail,
        ..Default::default()
    }
}

fn spawn_fireball_trail_projectile_loc_330d0(
    projectiles: Option<&mut Vec<EnemyProjectile>>,
    parent_x_direction: u8,
    x_px: i16,
    y_px: i16,
) {
    let Some(projectiles) = projectiles else {
        return;
    };
    if count_live_fireball_trails(projectiles) >= 5 {
        return;
    }
    projectiles.push(fireball_trail_projectile_from_loc_330d0(
        parent_x_direction,
        x_px,
        y_px,
    ));
}

fn spawn_fireball_obstacle_trail_projectile_loc_33128(
    projectiles: Option<&mut Vec<EnemyProjectile>>,
    enemy: &ActiveEnemy,
) {
    let Some(projectiles) = projectiles else {
        return;
    };
    if count_live_fireball_trails(projectiles) >= 5 {
        return;
    }

    // ASM Fireball.asm loc_33128: obstacle trails start at parent x + 2*x_vel,
    // with Y aligned to the next 16px row after adding $F.
    let x_px = ((enemy.x_pos as i64 + (i64::from(enemy.x_velocity) * 2)) >> 16) as i16;
    let y_px = ((((enemy.y_pos >> 16) as i32) + 0x0F) & !0x0F) as i16;
    let mut projectile = fireball_trail_projectile_from_loc_330d0(0, x_px, y_px);
    projectile.kind = EnemyProjectileKind::FireballObstacleTrail;
    projectiles.push(projectile);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballSlopeTrailMarkerLoc32D7E {
    pub(crate) x_px: i16,
    pub(crate) y_px: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballSlopeDownTrailMarkerLoc32E3C {
    pub(crate) x_px: i16,
    pub(crate) y_px: i16,
}

pub(crate) fn fireball_slope_up_trail_marker_loc_32d7e(
    probe_x_px: i16,
    probe_y_px: i16,
    tile_word: u16,
) -> Option<FireballSlopeTrailMarkerLoc32D7E> {
    // ASM Fireball.asm loc_32D7E: read the tile at d7/d6, require collision
    // nibble `$4000`, then align d6 to the tile row and subtract d7's low nibble.
    if tile_word & 0x7000 != 0x4000 {
        return None;
    }

    let x_fraction = probe_x_px & 0x000F_i16;
    let aligned_y = probe_y_px & !0x000F_i16;
    Some(FireballSlopeTrailMarkerLoc32D7E {
        x_px: probe_x_px.wrapping_add(2),
        y_px: aligned_y
            .wrapping_add(0x000F)
            .wrapping_sub(x_fraction)
            .wrapping_sub(2),
    })
}

pub(crate) fn fireball_slope_down_trail_marker_loc_32e3c(
    probe_x_px: i16,
    probe_y_px: i16,
    tile_word: u16,
) -> Option<FireballSlopeDownTrailMarkerLoc32E3C> {
    // ASM Fireball.asm loc_32E3C: after sampling d7/d6, require collision
    // nibble `$5000`, then align d6 to the tile row and add d7's low nibble.
    if tile_word & 0x7000 != 0x5000 {
        return None;
    }

    let x_fraction = probe_x_px & 0x000F_i16;
    let aligned_y = probe_y_px & !0x000F_i16;
    Some(FireballSlopeDownTrailMarkerLoc32E3C {
        x_px: probe_x_px.wrapping_sub(2),
        y_px: aligned_y.wrapping_add(x_fraction).wrapping_sub(2),
    })
}

fn terrain_collision_tile_word(cell: terrain::Cell) -> u16 {
    (cell.collision as u16) << 12
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

pub(crate) fn fireball_main_loop_y_velocity_loc_32c1a(enemy: &ActiveEnemy) -> i32 {
    // ASM Fireball.asm loc_32C1A: after the one-frame hibernate and
    // `Object_CheckInRange`, add d0 gravity into `y_vel(a3)` before `loc_32C60`.
    enemy.y_velocity.saturating_add(bouncer_gravity(enemy))
}

const BIG_HOPPING_SKULL_UNBOUNDED_PATROL_BOUNDS: u32 = 0;

fn big_hopping_skull_pack_patrol_bounds_loc_364a0(lower_bound: u16, upper_bound: u16) -> u32 {
    // ASM Big_Hopping_Skull.asm loc_364A0: copy the active enemy header
    // thresholds into `$3E/$40`, then add `$40` to the previous threshold
    // and subtract `$40` from the current threshold before loc_36770.
    let left_bound = lower_bound.wrapping_add(0x0040);
    let right_bound = upper_bound.wrapping_sub(0x0040);
    (u32::from(left_bound) << 16) | u32::from(right_bound)
}

fn big_hopping_skull_unpack_patrol_bounds_loc_364a0(packed: u32) -> (i16, i16) {
    if packed == BIG_HOPPING_SKULL_UNBOUNDED_PATROL_BOUNDS {
        (i16::MIN, i16::MAX)
    } else {
        ((packed >> 16) as u16 as i16, packed as u16 as i16)
    }
}

#[cfg(test)]
pub(crate) fn step_bouncer(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    let mut rng = RomRandomState::default();
    let camera_x_px = enemy.x_pos >> 16;
    step_bouncer_with_rng(enemy, block_grid, Some(camera_x_px), &mut rng, None, None);
}

fn queue_fireball_bounce_sound_loc_32c60(
    family: EnemyFamily,
    played_sound_effects: &mut Option<&mut Vec<QueuedSoundEffect>>,
    already_queued: &mut bool,
) {
    if !matches!(family, EnemyFamily::Fireball) || *already_queued {
        return;
    }

    if let Some(sound_effects) = played_sound_effects.as_deref_mut() {
        // ASM Fireball.asm loc_32C60: after a non-zero collision type and
        // before the behaviour jump table, play `sfx_Fireball_bouncing`.
        sound_effects.push(QueuedSoundEffect::FireballBouncing);
    }
    *already_queued = true;
}

fn update_fireball_sprite_state_loc_32f3e_3304a(enemy: &mut ActiveEnemy) {
    // ASM Fireball.asm loc_32C1A calls loc_32F3E then loc_3304A every frame:
    // the selected sprite writes `x_direction(a3)`, which the slope-trail
    // allocation paths later use when positioning loc_330D0 child workers.
    let selection = fireball_sprite_selection_loc_32f3e(enemy.x_velocity, enemy.y_velocity);
    let animation_countdown = if enemy.dragon_phase_frames == 0 {
        5
    } else {
        enemy.dragon_phase_frames
    };
    if let Some(commit) = fireball_sprite_commit_loc_3304a(
        selection.sprite_index,
        u16::from(enemy.dragon_script_cmd),
        animation_countdown,
    ) {
        enemy.facing = if commit.x_direction == 0 {
            Direction::Right
        } else {
            Direction::Left
        };
        enemy.dragon_script_cmd = commit.next_animation_phase as u8;
        enemy.dragon_phase_frames = commit.next_animation_countdown;
    }
}

pub(crate) fn step_bouncer_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    camera_x_px: Option<u32>,
    rng: &mut RomRandomState,
    mut projectiles: Option<&mut Vec<EnemyProjectile>>,
    mut played_sound_effects: Option<&mut Vec<QueuedSoundEffect>>,
) {
    if matches!(enemy.family, EnemyFamily::Sphere) && !enemy.death_animation_active {
        // ASM Sphere.asm `stru_36174`: C8120/C8128/C8130 each hibernate for
        // 9 ticks before the animation loops. Native keeps only the phase
        // counter needed by contact and rendering-facing snapshots.
        enemy.wait_timer = (enemy.wait_timer + 1) % SPHERE_LIVE_ANIMATION_FRAMES;
    }

    // Apply gravity
    enemy.y_velocity = if matches!(enemy.family, EnemyFamily::Fireball) {
        fireball_main_loop_y_velocity_loc_32c1a(enemy)
    } else {
        enemy.y_velocity.saturating_add(bouncer_gravity(enemy))
    };

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;
        let mut fireball_bounce_sound_queued = false;

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
            queue_fireball_bounce_sound_loc_32c60(
                enemy.family,
                &mut played_sound_effects,
                &mut fireball_bounce_sound_queued,
            );
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
                queue_fireball_bounce_sound_loc_32c60(
                    enemy.family,
                    &mut played_sound_effects,
                    &mut fireball_bounce_sound_queued,
                );
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
                    if matches!(enemy.family, EnemyFamily::Fireball) {
                        let tile_word = terrain_collision_tile_word(floor.cell);
                        if let Some(marker) = fireball_slope_up_trail_marker_loc_32d7e(
                            (enemy.x_pos >> 16) as u16 as i16,
                            (enemy.y_pos >> 16) as u16 as i16,
                            tile_word,
                        ) {
                            spawn_fireball_trail_projectile_loc_330d0(
                                projectiles.as_deref_mut(),
                                if enemy.facing == Direction::Right {
                                    0xFF
                                } else {
                                    0x00
                                },
                                marker.x_px,
                                marker.y_px,
                            );
                        }
                    }
                }
            } else if floor.cell.collision == terrain::CollisionKind::SlopeDown {
                queue_fireball_bounce_sound_loc_32c60(
                    enemy.family,
                    &mut played_sound_effects,
                    &mut fireball_bounce_sound_queued,
                );
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
                    if matches!(enemy.family, EnemyFamily::Fireball) {
                        let tile_word = terrain_collision_tile_word(floor.cell);
                        if let Some(marker) = fireball_slope_down_trail_marker_loc_32e3c(
                            (enemy.x_pos >> 16) as u16 as i16,
                            (enemy.y_pos >> 16) as u16 as i16,
                            tile_word,
                        ) {
                            spawn_fireball_trail_projectile_loc_330d0(
                                projectiles.as_deref_mut(),
                                if enemy.facing == Direction::Right {
                                    0xFF
                                } else {
                                    0x00
                                },
                                marker.x_px,
                                marker.y_px,
                            );
                        }
                    }
                }
            } else {
                queue_fireball_bounce_sound_loc_32c60(
                    enemy.family,
                    &mut played_sound_effects,
                    &mut fireball_bounce_sound_queued,
                );
                // Regular floor bounce
                if matches!(enemy.family, EnemyFamily::Fireball) {
                    let fire_trail_count = projectiles
                        .as_ref()
                        .map_or(0, |projectiles| count_live_fireball_trails(projectiles));
                    let bounce = fireball_floor_ceiling_bounce_loc_32ce4(
                        enemy,
                        rng,
                        camera_x_px.unwrap_or(0),
                        fire_trail_count,
                    );
                    enemy.y_velocity = bounce.y_velocity;
                    if bounce.spawns_fire_trail {
                        spawn_fireball_obstacle_trail_projectile_loc_33128(
                            projectiles.as_deref_mut(),
                            enemy,
                        );
                    }
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
            queue_fireball_bounce_sound_loc_32c60(
                enemy.family,
                &mut played_sound_effects,
                &mut fireball_bounce_sound_queued,
            );
            if matches!(enemy.family, EnemyFamily::Fireball) {
                // ASM Enemy19_Fireball_BounceFloorCeiling reuses the same
                // perturb-and-negate path for both floor and ceiling contact.
                let fire_trail_count = projectiles
                    .as_ref()
                    .map_or(0, |projectiles| count_live_fireball_trails(projectiles));
                let bounce = fireball_floor_ceiling_bounce_loc_32ce4(
                    enemy,
                    rng,
                    camera_x_px.unwrap_or(0),
                    fire_trail_count,
                );
                enemy.y_velocity = bounce.y_velocity;
                if bounce.spawns_fire_trail {
                    spawn_fireball_obstacle_trail_projectile_loc_33128(projectiles, enemy);
                }
            } else {
                // Sphere.asm: negate and halve
                enemy.y_velocity = -(enemy.y_velocity >> 1);
            }
        }
    }

    if matches!(enemy.family, EnemyFamily::Fireball) {
        update_fireball_sprite_state_loc_32f3e_3304a(enemy);
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
    // ASM Dragon_Flying.asm loc_346C4 reinitialises normal animation
    // `stru_34600`; restart the native contact-phase tick with it.
    enemy.wait_timer = 0;
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

pub(crate) fn enemy_should_remain_active(
    enemy: &ActiveEnemy,
    view: &EnemyLayoutView<'_>,
    window: ActivationWindow,
    camera_x_px: u32,
    camera_y_px: u32,
    level_bounds: physics::LevelBounds,
    active_enemy_count: usize,
) -> bool {
    if enemy.death_animation_active && enemy_death_animation_frames(enemy.family).is_some() {
        // ASM Dragon.asm loc_3CEFC and the other family death tails hibernate
        // until the death animation finishes, then enter their delete tail.
        // They do not run the live object offscreen/deactivation checks while
        // the corpse animation is hibernating.
        return true;
    }
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
        EnemyArchetype::Pacer if matches!(enemy.family, EnemyFamily::Diamond) => {
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
            if matches!(
                enemy.family,
                EnemyFamily::Armadillo
                    | EnemyFamily::DragonWalking
                    | EnemyFamily::Goat
                    | EnemyFamily::Lion
                    | EnemyFamily::Ninja
                    | EnemyFamily::Orca
                    | EnemyFamily::Scorpion
            ) =>
        {
            // ASM Armadillo.asm loc_3C930 and the analogous Goat/Scorpion/
            // Orca/Ninja/Dragon/Lion shared-walker branches route through
            // sub_3E8BA, which culls from the live object position rather
            // than the layout anchor.
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
    if matches!(enemy.family, EnemyFamily::Lion) && active_enemy_count <= 20 {
        // ASM Lion.asm loc_3E924: the medium-load `sub_3E8BA` camera gate
        // returns to the restored Lion callback inside the wide box and
        // branches through loc_3E920 to deletion outside it.
        return matches!(
            lion_medium_load_cull_loc_3e924(
                x_pos as u16,
                y_pos as u16,
                camera_x_px as u16,
                camera_y_px as u16,
                active_enemy_count as u16,
                enemy.layout_index as u8,
            ),
            LionMediumLoadCullLoc3E924::Return
        );
    }

    // ASM Dragon.asm loc_37552 and the analogous non-Lion shared-walker
    // culls: 11..20 active objects use the wider [-$1A4,+$2E4]x[-$1A4,+$284]
    // box, 21+ active objects tighten to [-$104,+$244]x[-$104,+$1E4].
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
    if active_enemy_count <= 20 {
        // ASM Mini_Hopping_Skull.asm loc_3F304: the medium-load live-object
        // camera gate returns to the caller inside the wide box and branches
        // through loc_3F300 to the delete tail outside it.
        return matches!(
            mini_hopping_skull_medium_load_camera_gate_loc_3f304(
                x_pos as u16,
                y_pos as u16,
                camera_x_px as u16,
                camera_y_px as u16,
                active_enemy_count as u16,
                enemy.layout_index as u8,
            ),
            MiniHoppingSkullMediumLoadGateLoc3F304::Return
        );
    }

    // ASM Mini_Hopping_Skull.asm sub_3F29A: 21+ active objects tighten to
    // [-$40,+$180]x[-$40,+$120] after loc_3F304's medium-load branch.
    let (left_margin, right_margin, top_margin, bottom_margin) = (0x40, 0x180, 0x40, 0x120);
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
            // ASM Mini_Hopping_Skull.asm loc_3F386: `addi.w #$10,d6` before
            // each collision check means the snap uses the matched slope
            // cell's right edge, then subtracts the low nibble of Y.
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
            // ASM Mini_Hopping_Skull.asm loc_3F3CE: the loop samples the tile,
            // subtracts 16px, and walks left until `$5000`; the final snap uses
            // the matched slope cell's left edge plus the current Y low nibble.
            let snapped_x_px = (col * 16) + (y_px & 0x0F);
            return (snapped_x_px.max(0) as u32) << 16;
        }
    }

    (x_px.max(0) as u32) << 16
}

fn mini_hopping_skull_facing_from_x_direction(x_direction: u8) -> Direction {
    // ASM enemy art faces left by default: x_direction=0 is left, `$FF` flips right.
    if x_direction == 0 {
        Direction::Left
    } else {
        Direction::Right
    }
}

fn mini_hopping_skull_x_direction_from_facing(facing: Direction) -> u8 {
    match facing {
        Direction::Left => 0x00,
        Direction::Right => 0xFF,
    }
}

pub(crate) fn mini_hopping_skull_air_damage_setup_loc_3f1e6(
    hit_counter: u16,
    x_velocity: i32,
) -> MiniHoppingSkullAirDamageSetup {
    // ASM Mini_Hopping_Skull.asm loc_3F1E6: `subi.w #1,$44(a3)`,
    // `move.l #$28800,y_vel(a3)`, preserve live X in d2, then clear live X
    // and `collision_type(a3)` before entering the landing loop.
    MiniHoppingSkullAirDamageSetup {
        remaining_hits: hit_counter.wrapping_sub(1),
        y_velocity: 0x0002_8800,
        preserved_x_velocity: x_velocity,
        x_velocity: 0,
        collision_type: 0,
        next_loop: "air_damage_landing_loop",
    }
}

fn step_mini_hopping_skull_charging_berzerker_loop(enemy: &mut ActiveEnemy) {
    // ASM Mini_Hopping_Skull.asm loc_3F288: hibernate one frame, run
    // `sub_3E8BA`, add `$4000` to y_vel, then branch back to loc_3F288.
    enemy.x_velocity = 0;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    enemy.y_velocity =
        mini_hopping_skull_charging_berzerker_loop_loc_3f288(enemy.y_velocity).y_velocity;
    enemy.on_ground = false;
}

fn step_mini_hopping_skull_air_damage_loop(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) {
    let Some(grid) = block_grid else {
        return;
    };

    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let floor = terrain::probe_floor(grid, cx, cy);
    let mut collision_type = 0;
    if floor.on_ground && enemy.y_velocity >= 0 {
        let floor_y_fp = (floor.floor_y_px as u32) << 16;
        if enemy.y_pos >= floor_y_fp {
            if floor.cell.collision == terrain::CollisionKind::SlopeUp {
                enemy.x_pos = mini_hopping_skull_snap_slopeup_x(
                    grid,
                    (enemy.x_pos >> 16) as i32,
                    (enemy.y_pos >> 16) as i32,
                );
                collision_type = MINI_SKULL_COLID_SLOPEUP;
            } else if floor.cell.collision == terrain::CollisionKind::SlopeDown {
                enemy.x_pos = mini_hopping_skull_snap_slopedown_x(
                    grid,
                    (enemy.x_pos >> 16) as i32,
                    (enemy.y_pos >> 16) as i32,
                );
                collision_type = MINI_SKULL_COLID_SLOPEDOWN;
            } else {
                collision_type = MINI_SKULL_COLID_FLOOR;
            }
            enemy.y_pos = floor_y_fp;
        }
    }

    let frame = mini_hopping_skull_air_damage_loop_loc_3f200(enemy.y_velocity, collision_type);
    enemy.y_velocity = frame.y_velocity;

    match frame.route {
        MiniHoppingSkullAirDamageLoopRoute::ContinueWait => {
            enemy.x_velocity = 0;
            enemy.on_ground = false;
            enemy.wait_timer = MINI_HOPPING_SKULL_AIR_DAMAGE_WAIT_TIMER_LOC_3F200;
        }
        MiniHoppingSkullAirDamageLoopRoute::FloorLanding => {
            match mini_hopping_skull_death_or_stopped_wait_loc_3f158(u16::from(enemy.hit_points)) {
                MiniHoppingSkullLoc3F158Route::RestartStoppedWait => {
                    let setup = mini_hopping_skull_stopped_wait_setup_loc_3f000();
                    enemy.x_velocity = setup.x_velocity;
                    enemy.y_velocity = setup.y_velocity;
                    enemy.grounded_frames = setup.wait_frames;
                    enemy.wait_timer = 0;
                    enemy.on_ground = true;
                }
                MiniHoppingSkullLoc3F158Route::DestroySkull(_) => {
                    enemy.x_velocity = 0;
                    enemy.y_velocity = 0;
                    enemy.wait_timer = MINI_HOPPING_SKULL_DEATH_ANIMATION_FRAMES;
                    enemy.on_ground = true;
                    enemy.death_animation_active = true;
                }
            }
        }
        MiniHoppingSkullAirDamageLoopRoute::SlopeLanding => {
            match mini_hopping_skull_air_damage_slope_landing_loc_3f22a(
                u16::from(enemy.hit_points),
                enemy.target_x_velocity,
            ) {
                MiniHoppingSkullAirDamageSlopeLanding::ResumeTerrainContact {
                    x_velocity, ..
                } => {
                    apply_mini_hopping_skull_slope_contact_wait_loc_3ead8(
                        enemy,
                        collision_type,
                        x_velocity,
                        enemy.y_pos,
                    );
                }
                MiniHoppingSkullAirDamageSlopeLanding::DestroySkull(_) => {
                    enemy.x_velocity = 0;
                    enemy.y_velocity = 0;
                    enemy.wait_timer = MINI_HOPPING_SKULL_DEATH_ANIMATION_FRAMES;
                    enemy.on_ground = true;
                    enemy.death_animation_active = true;
                }
            }
        }
    }
}

fn apply_mini_hopping_skull_slope_contact_wait_loc_3ead8(
    enemy: &mut ActiveEnemy,
    collision_type: u16,
    x_velocity: i32,
    floor_y_fp: u32,
) {
    // ASM Mini_Hopping_Skull.asm loc_3EAD8: slope terrain contact clears motion
    // and routes by collision ID plus the restored/live x_vel high word.
    let contact = mini_hopping_skull_terrain_contact_loc_3ead8(collision_type, x_velocity);
    let _cleared_flags = (
        contact.is_moved,
        contact.is_animated,
        contact.has_level_collision,
    );
    debug_assert_eq!(_cleared_flags, (false, false, false));

    match contact.route {
        MiniHoppingSkullTerrainContactRoute::SlopeUpPositiveXVelocity => {
            let setup = mini_hopping_skull_ground_wait_setup_loc_3eb00();
            enemy.facing = mini_hopping_skull_facing_from_x_direction(setup.x_direction);
            enemy.x_velocity = setup.x_velocity;
            enemy.y_velocity = setup.y_velocity;
            enemy.on_ground = true;
            enemy.grounded_frames = setup.wait_frames;
            enemy.wait_timer = 0;
            enemy.y_pos = floor_y_fp;
        }
        MiniHoppingSkullTerrainContactRoute::SlopeUpNonPositiveXVelocity => {
            let setup = mini_hopping_skull_left_hop_wait_setup_loc_3ed80();
            enemy.facing = mini_hopping_skull_facing_from_x_direction(setup.x_direction);
            enemy.x_velocity = setup.x_velocity;
            enemy.y_velocity = setup.y_velocity;
            enemy.on_ground = true;
            enemy.grounded_frames = setup.wait_frames;
            enemy.wait_timer = 0;
            enemy.y_pos = floor_y_fp;
        }
        MiniHoppingSkullTerrainContactRoute::NotSlopeUp => {
            if collision_type == MINI_SKULL_COLID_SLOPEDOWN {
                let contact = mini_hopping_skull_slopedown_contact_loc_3ed6c(x_velocity);
                enemy.facing = mini_hopping_skull_facing_from_x_direction(contact.x_direction);
                enemy.x_velocity = contact.x_velocity;
                enemy.y_velocity = contact.y_velocity;
                enemy.on_ground = true;
                enemy.grounded_frames = contact.wait_frames;
                enemy.wait_timer = 0;
                enemy.y_pos = floor_y_fp;
            }
        }
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
        let mut mini_skull_relaunched = false;

        if matches!(enemy.family, EnemyFamily::MiniHoppingSkull) {
            if enemy.grounded_frames > 0 {
                enemy.grounded_frames -= 1;
                enemy.x_velocity = 0;
                return;
            }

            if enemy.wait_timer & MINI_SKULL_FACING_MATCHED_PAUSE_TIMER_FLAG_LOC_3F09E != 0 {
                let pause_frames =
                    enemy.wait_timer & !MINI_SKULL_FACING_MATCHED_PAUSE_TIMER_FLAG_LOC_3F09E;
                let tail = mini_hopping_skull_facing_matched_pause_tail_loc_3f112(
                    pause_frames,
                    mini_hopping_skull_x_direction_from_facing(enemy.facing),
                    enemy.hit_points > 1,
                );

                match tail {
                    MiniHoppingSkullFacingMatchedPauseTail::ContinueWait { next_dbf_counter } => {
                        // ASM Mini_Hopping_Skull.asm loc_3F112: active DBF
                        // counts return to the matched-facing pause collision loop.
                        enemy.wait_timer =
                            MINI_SKULL_FACING_MATCHED_PAUSE_TIMER_FLAG_LOC_3F09E | next_dbf_counter;
                        enemy.x_velocity = 0;
                        return;
                    }
                    MiniHoppingSkullFacingMatchedPauseTail::BeginRelaunchArc {
                        y_velocity,
                        x_velocity,
                        sprite,
                        is_moved,
                        has_level_collision,
                        next_loop,
                    } => {
                        // ASM Mini_Hopping_Skull.asm loc_3F112 -> loc_3F146:
                        // expired matched-facing pause seeds the airborne
                        // relaunch and resumes the loc_3EA88 loop.
                        debug_assert_eq!(sprite, "LnkTo_unk_C8080");
                        debug_assert!(is_moved);
                        debug_assert!(has_level_collision);
                        debug_assert_eq!(next_loop, "loc_3EA88");
                        enemy.wait_timer = 0;
                        enemy.x_velocity = x_velocity;
                        enemy.y_velocity = y_velocity;
                        enemy.on_ground = false;
                        mini_skull_relaunched = true;
                    }
                }
            } else if let Some(kid_x_pos) = player_x_pos {
                let tail = mini_hopping_skull_stopped_wait_tail_loc_3f090(
                    0,
                    (kid_x_pos >> 16) as u16,
                    (enemy.x_pos >> 16) as u16,
                    mini_hopping_skull_x_direction_from_facing(enemy.facing),
                );

                match tail {
                    MiniHoppingSkullStoppedWaitTail::ContinueWait { next_dbf_counter } => {
                        debug_assert_ne!(next_dbf_counter, 0xFFFF);
                        enemy.grounded_frames = next_dbf_counter.saturating_add(1) as u8;
                        enemy.x_velocity = 0;
                        return;
                    }
                    MiniHoppingSkullStoppedWaitTail::RestartStoppedWaitFacingLeft {
                        x_direction,
                        dbf_counter,
                        wait_frames,
                    }
                    | MiniHoppingSkullStoppedWaitTail::RestartStoppedWaitFacingRight {
                        x_direction,
                        dbf_counter,
                        wait_frames,
                    } => {
                        // ASM Mini_Hopping_Skull.asm loc_3F074/loc_3F090:
                        // a facing mismatch restarts loc_3F000 with d1=$14.
                        debug_assert_eq!(dbf_counter, 0x0014);
                        enemy.facing = mini_hopping_skull_facing_from_x_direction(x_direction);
                        enemy.grounded_frames = wait_frames;
                        enemy.wait_timer = 0;
                        enemy.x_velocity = 0;
                        return;
                    }
                    MiniHoppingSkullStoppedWaitTail::BeginFacingMatchedPause {
                        sprite,
                        dbf_counter,
                        wait_frames,
                    } => {
                        // ASM Mini_Hopping_Skull.asm loc_3F09E seeds the
                        // matched-facing pause loop with d1=$A.
                        debug_assert_eq!(sprite, "LnkTo_unk_C8078");
                        debug_assert_eq!(dbf_counter, 0x000A);
                        debug_assert_eq!(
                            u16::from(wait_frames),
                            MINI_SKULL_FACING_MATCHED_PAUSE_FRAMES_LOC_3F09E
                        );
                        enemy.wait_timer = MINI_SKULL_FACING_MATCHED_PAUSE_TIMER_FLAG_LOC_3F09E
                            | u16::from(wait_frames);
                        enemy.x_velocity = 0;
                        return;
                    }
                }
            }
        }

        if !mini_skull_relaunched {
            let mut facing =
                player_relative_direction(enemy.x_pos, player_x_pos).unwrap_or(enemy.facing);
            let reverse_at_platform_edge = !matches!(enemy.family, EnemyFamily::MiniHoppingSkull);
            if hopper_should_reverse(facing, grid, cx, cy, reverse_at_platform_edge) {
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
        let should_bounce = if matches!(enemy.family, EnemyFamily::MiniHoppingSkull) {
            let collision_type = if moving_right {
                MINI_SKULL_COLID_RIGHTWALL
            } else {
                MINI_SKULL_COLID_LEFTWALL
            };
            matches!(
                mini_hopping_skull_collision_dispatch_loc_3eab2(collision_type),
                MiniHoppingSkullCollisionDispatch::WallBounce
            )
        } else {
            true
        };
        if should_bounce {
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
    }

    if matches!(enemy.family, EnemyFamily::MiniHoppingSkull) && enemy.y_velocity < 0 {
        let ceiling =
            terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
        if ceiling.hit
            && matches!(
                mini_hopping_skull_collision_dispatch_loc_3eab2(MINI_SKULL_COLID_CEILING),
                MiniHoppingSkullCollisionDispatch::CeilingHit
            )
        {
            // ASM Mini_Hopping_Skull.asm loc_3F1A2: `neg.l y_vel(a3)`;
            // execution then branches back to the airborne loc_3EA88 loop.
            enemy.y_velocity = -enemy.y_velocity;
        }
    }

    let floor = terrain::probe_floor(grid, cx, cy);
    if floor.on_ground {
        let floor_y_fp = (floor.floor_y_px as u32) << 16;
        if enemy.y_velocity >= 0 && enemy.y_pos >= floor_y_fp {
            if matches!(enemy.family, EnemyFamily::MiniHoppingSkull) {
                if floor.cell.collision == terrain::CollisionKind::SlopeUp {
                    let x_velocity = enemy.x_velocity;
                    enemy.x_pos = mini_hopping_skull_snap_slopeup_x(
                        grid,
                        (enemy.x_pos >> 16) as i32,
                        (enemy.y_pos >> 16) as i32,
                    );
                    apply_mini_hopping_skull_slope_contact_wait_loc_3ead8(
                        enemy,
                        MINI_SKULL_COLID_SLOPEUP,
                        x_velocity,
                        floor_y_fp,
                    );
                    return;
                }

                if floor.cell.collision == terrain::CollisionKind::SlopeDown {
                    let x_velocity = enemy.x_velocity;
                    enemy.x_pos = mini_hopping_skull_snap_slopedown_x(
                        grid,
                        (enemy.x_pos >> 16) as i32,
                        (enemy.y_pos >> 16) as i32,
                    );
                    apply_mini_hopping_skull_slope_contact_wait_loc_3ead8(
                        enemy,
                        MINI_SKULL_COLID_SLOPEDOWN,
                        x_velocity,
                        floor_y_fp,
                    );
                    return;
                }
            } else if floor.cell.collision == terrain::CollisionKind::SlopeUp {
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
    reverse_at_platform_edge: bool,
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
    if !reverse_at_platform_edge {
        return false;
    }
    // Reverse at platform edge: look ahead for floor support
    let look_ahead_x = if moving_right {
        cx + ENEMY_PROBE_HALF_WIDTH
    } else {
        cx - ENEMY_PROBE_HALF_WIDTH
    };
    !terrain::probe_floor(grid, look_ahead_x, cy).on_ground
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SharedWalkerTurnaroundLoc3C026 {
    pub(crate) next_counter: u8,
    pub(crate) collision_type: Option<i16>,
}

pub(crate) fn shared_walker_turnaround_loc_3c026(
    turnaround_counter: u8,
) -> SharedWalkerTurnaroundLoc3C026 {
    // ASM loc_3C026/loc_3C0D2: `addq.b #4,$5A(a5); beq.w loc_3C464`.
    // loc_3C464 writes `#$FFFF` to collision_type, which Driller.asm
    // loc_3BF02 treats as the death-animation route.
    let next_counter = turnaround_counter.wrapping_add(4);
    SharedWalkerTurnaroundLoc3C026 {
        next_counter,
        collision_type: (next_counter == 0).then_some(-1),
    }
}

fn register_shared_walker_turnaround(enemy: &mut ActiveEnemy) -> bool {
    let result = shared_walker_turnaround_loc_3c026(enemy.shared_walker_turnaround_counter);
    enemy.shared_walker_turnaround_counter = result.next_counter;
    if result.collision_type.is_some() {
        register_enemy_terrain_kill(enemy);
        true
    } else {
        false
    }
}

fn register_enemy_terrain_kill(enemy: &mut ActiveEnemy) {
    if let Some(death_animation_frames) = enemy_death_animation_frames(enemy.family) {
        // ASM loc_3C464 and Lion.asm loc_3E94E write collision_type=$FFFF;
        // family handlers route that word through the normal death-animation
        // tail before delete.
        enemy.wait_timer = death_animation_frames;
    }
    enemy.hit_points = 0;
    enemy.death_animation_active = true;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SharedWalkerGroundedTurn {
    BoundaryWallSnapContinue(i32),
    RightBoundaryAfterExtraAdvance,
    RightBoundaryAtCurrentPosition,
}

pub(crate) fn shared_walker_grounded_scan_turn_plan(
    enemy: &ActiveEnemy,
    grid: &terrain::BlockGrid,
    walk_delta: i32,
    half_width: i32,
    height: i32,
) -> Option<SharedWalkerGroundedTurn> {
    let moving_right = walk_delta >= 0;
    let current_x_px = (enemy.x_pos >> 16) as i32;
    let current_y_px = (enemy.y_pos >> 16) as i32;
    let scan = if moving_right {
        terrain::grounded_walker_scan_right_sub_36972(
            grid,
            current_x_px,
            current_y_px,
            half_width,
            height,
        )
    } else {
        terrain::grounded_walker_scan_left_sub_36A58(
            grid,
            current_x_px,
            current_y_px,
            half_width,
            height,
        )
    };

    // ASM loc_3C04E/loc_3C0FA: `tst.w d6; bmi` continues immediately when the
    // following row is fully clear. Grounded cliff turns only happen on the
    // mixed/blocked following-row path after the next horizontal step crosses a
    // 16px tile boundary.
    if scan.floor_class_d6 < 0 {
        return None;
    }

    let next_x = (enemy.x_pos as i64 + i64::from(walk_delta)) as u32;
    let next_x_px = (next_x >> 16) as i32;
    let pixel_delta = if moving_right {
        next_x_px - current_x_px
    } else {
        current_x_px - next_x_px
    };
    if pixel_delta <= 0 {
        return None;
    }

    let tile_remainder = if moving_right {
        (-(current_x_px + half_width)) & 0xF
    } else {
        (current_x_px - half_width) & 0xF
    };
    let edge_bit_set = scan.flags_d7 & 0b001 != 0;
    let wall_bit_set = scan.flags_d7 & 0b010 != 0;
    let crossed_boundary = tile_remainder < pixel_delta;
    if wall_bit_set
        && !edge_bit_set
        && crossed_boundary
        && i32::from(scan.column_span_d3) * 8 >= half_width
    {
        // ASM loc_3C04E (kid.asm:43533-43542): when the boundary-crossing
        // dispatch sees d7 bit 1, clear d5, and enough scanned width, it snaps
        // x_pos to the crossed tile boundary before continuing through loc_3C17A.
        let snap_x = if moving_right {
            current_x_px + tile_remainder
        } else {
            current_x_px - tile_remainder
        };
        return Some(SharedWalkerGroundedTurn::BoundaryWallSnapContinue(snap_x));
    }

    if wall_bit_set && !edge_bit_set {
        return None;
    }

    if wall_bit_set
        && moving_right
        && crossed_boundary
        && i32::from(scan.column_span_d3) * 8 < half_width
    {
        // ASM loc_3C04E (kid.asm:43529-43542): only once the predicted step crosses
        // the next 16px boundary (`cmp.w d1,d3; bcc.s loc_3C0C6` continues
        // otherwise) does the wall dispatch run. With the wall bit (d7 bit 1) set
        // and the scanned body column span * 8 below the half-width (`lsl.w #3,d4;
        // cmp.w $48(a5),d4; bcs.s loc_3C0A6`), it reverses in place via loc_3C0A6 ->
        // loc_3C0D2 (clr.l x_vel/y_vel, sf x_direction) and NEVER commits the
        // predicted x step. The right-edge "extra half-step" body
        // exposure modelled below is a CLIFF-edge behaviour (issue #260, wall bit
        // clear) only — advancing into a wall is wrong. Flip at the current body X.
        //
        // issue #331: the body-overlap kill (d7 bit 2 -> loc_3C464
        // collision_type=$FFFF -> loc_3B41C bmi death) is faithful, but it must scan
        // the ROM's resting x. The extra half-step advanced the body one pixel into
        // the wall column, so the next body scan read that wall tile and spuriously
        // set bit 2 -> a terrain self-kill the ROM tank never takes (BLW2 0x01
        // RockTank #03 at the col-118 wall: ROM rests at x=1877, native had
        // over-advanced to x=1878).
        return Some(SharedWalkerGroundedTurn::RightBoundaryAtCurrentPosition);
    }

    if moving_right && !enemy.shared_walker_right_turn_extra_step_consumed {
        if crossed_boundary {
            Some(SharedWalkerGroundedTurn::RightBoundaryAfterExtraAdvance)
        } else {
            None
        }
    } else if moving_right && tile_remainder == 0 {
        Some(SharedWalkerGroundedTurn::RightBoundaryAtCurrentPosition)
    } else {
        None
    }
}

fn shared_walker_walk_speed(enemy: &ActiveEnemy) -> i32 {
    if enemy.walk_speed_override == TANK_STOPPED_WALK_SPEED_OVERRIDE_LOC_3B49A {
        0
    } else if enemy.walk_speed_override != 0 {
        enemy.walk_speed_override
    } else {
        pacer_walk_speed(enemy.family, enemy.hit_points)
    }
}

/// Step a Pacer-archetype enemy: walk at a fixed speed, reverse on wall or
/// floor edge. Matches the shared walker subroutines in the disassembly
/// (loc_3E392 / loc_3E546 / loc_3E560 / loc_3E3AC).
pub(crate) fn step_pacer(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    let Some(grid) = block_grid else {
        return;
    };

    let walk_speed = shared_walker_walk_speed(enemy);
    let probe_half_width = pacer_probe_half_width(enemy.family);
    let probe_height = pacer_probe_height(enemy.family);

    if !enemy.on_ground {
        step_shared_walker_falling_loc_3c288(enemy, grid);
        return;
    }

    enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
    enemy.y_velocity = 0;

    let current_x_px = (enemy.x_pos >> 16) as i32;
    let current_y_px = (enemy.y_pos >> 16) as i32;
    let moving_right = enemy.x_velocity >= 0;
    let scan = if moving_right {
        terrain::grounded_walker_scan_right_sub_36972(
            grid,
            current_x_px,
            current_y_px,
            probe_half_width,
            probe_height,
        )
    } else {
        terrain::grounded_walker_scan_left_sub_36A58(
            grid,
            current_x_px,
            current_y_px,
            probe_half_width,
            probe_height,
        )
    };

    if scan.flags_d7 & 0b100 != 0 {
        // ASM loc_3C04E/loc_3C0FA: after sub_36972/sub_36A58, `btst #2,d7`
        // branches to loc_3C464 before the d6 floor-class path or the later
        // wall/floor dispatch. This is body-overlap terrain contact, not a
        // recoverable wall turnaround.
        register_enemy_terrain_kill(enemy);
        return;
    }

    if scan.floor_class_d6 < 0 {
        // ASM loc_3C17A / loc_3C266: a clear following row enters the falling
        // shared-walker loop, first adding one pixel to the parent Y word, then
        // running the loc_3C19C/loc_3C288 movement pass and loc_3C246/loc_3C332
        // gravity update.
        enemy.y_pos = enemy.y_pos.wrapping_add(1 << 16);
        enemy.on_ground = false;
        step_shared_walker_falling_loc_3c288(enemy, grid);
        return;
    }

    let grounded_turn_plan = shared_walker_grounded_scan_turn_plan(
        enemy,
        grid,
        enemy.x_velocity,
        probe_half_width,
        probe_height,
    );

    if matches!(
        grounded_turn_plan,
        Some(SharedWalkerGroundedTurn::RightBoundaryAtCurrentPosition)
    ) {
        // ASM Tank.asm loc_3B40A -> kid.asm loc_3C04E/loc_3C0D2: once the
        // native phase debt has been paid, the later right-edge boundary reaches
        // the ROM's hibernate pose before turnaround. Flip at the current body
        // X instead of adding the catch-up half-step used by the first edge.
        reverse_enemy_direction(enemy);
        enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
        register_shared_walker_turnaround(enemy);
        return;
    }

    if let Some(SharedWalkerGroundedTurn::BoundaryWallSnapContinue(snap_x)) = grounded_turn_plan {
        enemy.x_pos = (snap_x as u32) << 16;
        let floor = terrain::probe_floor(grid, snap_x, current_y_px);
        if floor.on_ground {
            enemy.y_pos = (floor.floor_y_px as u32) << 16;
        }
        enemy.shared_walker_turnaround_counter = 0;
        return;
    }

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
    let wall = terrain::probe_wall(grid, cx, cy, probe_half_width, probe_height, moving_right);
    if wall.hit {
        // ASM loc_3C026 (rightward walkers) / loc_3C0D2 (leftward walkers):
        // clr.l x_vel(a3); clr.l y_vel(a3); st/sf x_direction(a3) — stop
        // and face the opposite direction after wall collision. The ASM also
        // bumps a byte counter at $5A(a5) by 4 and deactivates (loc_3C464)
        // if the counter wraps to zero (i.e. after 64 turnarounds). Native
        // handles deactivation via the retain/cull check each frame instead.
        // Push out of wall.
        let wall_fp = (wall.wall_x_px as u32) << 16;
        let edge_fp = (probe_half_width as u32) << 16;
        if moving_right {
            if enemy.x_pos + edge_fp > wall_fp {
                enemy.x_pos = wall_fp.saturating_sub(edge_fp);
            }
        } else if enemy.x_pos < wall_fp + edge_fp {
            enemy.x_pos = wall_fp + edge_fp;
        }
        reverse_enemy_direction(enemy);
        enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
        register_shared_walker_turnaround(enemy);
        return;
    }

    if matches!(
        grounded_turn_plan,
        Some(SharedWalkerGroundedTurn::RightBoundaryAfterExtraAdvance)
    ) {
        // ASM Tank.asm loc_3B40A -> kid.asm loc_3C04E/loc_3C0A6: the ROM's
        // rightward boundary turn exposes the body one extra half-step ahead
        // before `loc_3C0D2` clears `x_direction`; issue #260's route frame 99
        // pins that hibernate-boundary body/turret phase against oracle.
        enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
        reverse_enemy_direction(enemy);
        enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
        enemy.shared_walker_right_turn_extra_step_consumed = true;
        register_shared_walker_turnaround(enemy);
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
    // ASM loc_3C160 (kid.asm:43626-43632): leftward mirror of loc_3C0B4 —
    // when d6 >= 0 and bit 1 is clear, set $6E(a5) and branch to loc_3C026
    // turnaround. Native uses the same probe_floor edge check for both sides.
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
    // ASM loc_3C246 (kid.asm:43731-43739): no-floor continuation adds
    // `$2000` to y_vel, caps at `$80000`, stores it at loc_3C25E, and loops
    // back to loc_3C18A. Native keeps walkers grounded via probe_floor, so
    // this leftward gravity continuation is intentionally unreachable.
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
        cx + probe_half_width
    } else {
        cx - probe_half_width
    };
    if terrain::probe_floor(grid, look_ahead_x, cy).on_ground {
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground {
            enemy.y_pos = (floor.floor_y_px as u32) << 16;
        }
        // ASM loc_3C0C6 clears `$5A(a5)` after a successful shared-walker
        // horizontal advance. Native collapses the mirrored left/right loops,
        // so any non-turning frame clears the stuck-turn counter.
        enemy.shared_walker_turnaround_counter = 0;
    } else {
        reverse_enemy_direction(enemy);
        enemy.x_velocity = horizontal_velocity(enemy.facing, walk_speed);
        register_shared_walker_turnaround(enemy);
    }
}

fn step_shared_walker_falling_loc_3c288(enemy: &mut ActiveEnemy, grid: &terrain::BlockGrid) {
    // ASM loc_3C19C / loc_3C288: falling shared walkers keep the horizontal
    // velocity seeded by loc_3C17A/loc_3C266, apply the current Y velocity, then
    // loc_3C246/loc_3C332 adds `$2000` gravity capped at `$80000`.
    let previous_y_px = enemy.y_pos >> 16;
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let crossed_y_pixel = (enemy.y_pos >> 16) != previous_y_px;
    if crossed_y_pixel && enemy.y_velocity >= 0 {
        let floor = terrain::probe_floor(grid, cx, cy);
        let floor_y_fp = (floor.floor_y_px as u32) << 16;
        if floor.on_ground && enemy.y_pos >= floor_y_fp {
            enemy.y_pos = floor_y_fp;
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            enemy.on_ground = true;
            register_shared_walker_turnaround(enemy);
            return;
        }
    }

    enemy.y_velocity = (enemy.y_velocity + 0x2000).min(0x0008_0000);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrilledBlockLoc3BE78 {
    pub(crate) col: u16,
    pub(crate) row: u16,
    pub(crate) action: DrillerBlockDrillActionLoc3BEDE,
    pub(crate) direction: u16,
}

pub(crate) const NINJA_BERZERKER_SIDE_STUN_WAIT_TIMER_LOC_3D6C8: u16 = 0xD6C8;
pub(crate) const NINJA_BERZERKER_SIDE_LAUNCH_WAIT_TIMER_LOC_3DB3C: u16 = 0xDB3C;

fn step_ninja_berzerker_side_stun(enemy: &mut ActiveEnemy) {
    // ASM Ninja.asm loc_3D6C8: hibernate one frame, run `sub_3E8BA`,
    // add `$4000` to y_vel, then branch back to the stun loop.
    enemy.x_velocity = 0;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    enemy.y_velocity = ninja_collision_stun_loop_loc_3d6c8(enemy.y_velocity).y_velocity;
    enemy.on_ground = false;
}

fn step_ninja_berzerker_side_launch(enemy: &mut ActiveEnemy) {
    // ASM Ninja.asm loc_3DB3C: hibernate one frame, run `sub_3E8BA`,
    // add `$4000` to y_vel, then branch back to the launch loop.
    enemy.x_velocity = 0;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    enemy.y_velocity = ninja_berzerker_side_launch_loop_tick_loc_3db3c(enemy.y_velocity).y_velocity;
    enemy.on_ground = false;
}

fn step_orca_berzerker_knockback(enemy: &mut ActiveEnemy) {
    // ASM Orca.asm loc_3D2AA: hibernate one frame, run `sub_3E8BA`,
    // add `$4000` to y_vel, then branch back to the knockback loop.
    enemy.x_velocity = 0;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    enemy.y_velocity = orca_berzerker_airborne_frame_loc_3d2aa(enemy.y_velocity).next_y_velocity;
    enemy.on_ground = false;
}

pub(crate) const NINJA_ATTACK_TIMER_INIT_LOC_3D5AA: u16 = 0x012C;
fn ninja_maniaxe_projectile_dx(
    projectiles: &[EnemyProjectile],
    enemy: &ActiveEnemy,
) -> Option<i16> {
    projectiles
        .iter()
        .find(|projectile| projectile.kind == EnemyProjectileKind::ManiaxeAxe)
        .map(|projectile| {
            ((i64::from(projectile.x_pos) - i64::from(enemy.x_pos)) >> 16)
                .clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16
        })
}

fn fixed_position_integer_word(position: u32) -> u16 {
    let [high_a, high_b, _, _] = position.to_be_bytes();
    u16::from_be_bytes([high_a, high_b])
}

#[allow(non_snake_case)]
pub(crate) fn hand_init_y_snap_loc_32aea(y_pos_word: u16) -> u16 {
    // ASM Hand.asm loc_32AEA: after init and blank-row adjustment, snap the
    // integer y word to `((y - 8) & $FFF0) + $F`.
    y_pos_word.wrapping_sub(8) & 0xFFF0 | 0x000F
}

fn asr4_word(value: u16) -> i16 {
    i16::from_be_bytes(value.to_be_bytes()) >> 4
}

#[allow(non_snake_case)]
pub(crate) fn hand_initial_motion_loc_327f2(
    hit_points: u8,
    x_direction_flag: bool,
) -> HandInitialMotionLoc327F2 {
    // ASM Hand.asm loc_327F2: store d7 into x_vel, initialise `stru_32B6A`,
    // then clear has_kid_collision before entering the patrol loop.
    HandInitialMotionLoc327F2 {
        x_velocity: hand_velocity_candidate_loc_327e4(hit_points, x_direction_flag),
        animation_label: "stru_32B6A",
        has_kid_collision: false,
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

// Lion homing accel is $2000/frame (Lion.asm loc_3E1C2) — set on projectile.
// Wall-bounce: Lion.asm loc_3E260 — reverses velocity on terrain collision.
// Speed cap: ±$60000 on both axes (Lion.asm loc_3E1C2).

fn lion_active_projectile_count(enemy: &ActiveEnemy) -> u8 {
    enemy.ufo_beam_segments
}

/// Robot charge speed: always $28000 (Robot.asm line 157).
pub(crate) const ROBOT_CHARGE_SPEED: i32 = 0x2_8000;

/// Robot charge direction-flip timer base: ASM Robot.asm sets $54(a5) = $F = 15.
/// Reloads to (random_byte >> 1) + $F (range 15-142) on each flip.
/// The charge is permanent until wall hit — no timed exit.
pub(crate) const ROBOT_CHARGE_FLIP_BASE: u16 = 15;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotHitReactionRouteLoc3A8Fc {
    ClearAndWalk,
    EnterCharge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RobotChildSpriteUpdateLoc3A8Fc {
    KeepCurrent,
    UseHitFrame,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RobotHitReactionLoc3A8Fc {
    pub(crate) remaining_hp: u16,
    pub(crate) route: RobotHitReactionRouteLoc3A8Fc,
    pub(crate) next_parent_collision_type: u16,
    pub(crate) next_child_collision_type: u16,
    pub(crate) child_collision_latched: bool,
    pub(crate) transient_child_y_offset_delta_px: i16,
    pub(crate) child_y_offset_delta_px: i16,
    pub(crate) child_sprite_update: RobotChildSpriteUpdateLoc3A8Fc,
    pub(crate) restores_walk_animations: bool,
    pub(crate) replay_jump_sound: bool,
}

pub(crate) fn robot_hit_reaction_loc_3a8fc(
    current_hp: u16,
    tracking_latch_set: bool,
    motion_suppressed: bool,
) -> RobotHitReactionLoc3A8Fc {
    // ASM Robot.asm loc_3A8FC: latch child collision and optionally select the
    // hit frame while `$4E(a5)` is clear. loc_3A90C then moves the child down
    // 4px for the hit animation before decrementing HP.
    let remaining_hp = current_hp.wrapping_sub(1);
    let child_sprite_update = if tracking_latch_set {
        RobotChildSpriteUpdateLoc3A8Fc::KeepCurrent
    } else {
        RobotChildSpriteUpdateLoc3A8Fc::UseHitFrame
    };

    if remaining_hp == 0 {
        return RobotHitReactionLoc3A8Fc {
            remaining_hp,
            route: RobotHitReactionRouteLoc3A8Fc::EnterCharge,
            next_parent_collision_type: 0x0800,
            next_child_collision_type: 0x0800,
            child_collision_latched: true,
            transient_child_y_offset_delta_px: 4,
            child_y_offset_delta_px: 4,
            child_sprite_update,
            restores_walk_animations: false,
            replay_jump_sound: false,
        };
    }

    // ASM Robot.asm loc_3A936: non-lethal hits clear the child collision latch,
    // subtract the same 4px from child Y, and reinitialise the body/head walk
    // animations before clearing the pending collision words.
    RobotHitReactionLoc3A8Fc {
        remaining_hp,
        route: RobotHitReactionRouteLoc3A8Fc::ClearAndWalk,
        next_parent_collision_type: 0,
        next_child_collision_type: 0,
        child_collision_latched: false,
        transient_child_y_offset_delta_px: 4,
        child_y_offset_delta_px: 0,
        child_sprite_update,
        restores_walk_animations: true,
        replay_jump_sound: !motion_suppressed,
    }
}

pub(crate) fn finish_robot_shoot_sequence(enemy: &mut ActiveEnemy) {
    // ASM Robot.asm loc_3AAC4: the completed shoot sequence clears the child
    // collision word and resumes the shared walker. loc_3A96C charge entry is
    // reached by collision/hit routes, not by ordinary shooting.
    enemy.robot_state = RobotState::Walking;
    enemy.wait_timer = ROBOT_SHOOT_INTERVAL;
}

/// Orca lunge cooldown (Orca.asm: $12C = 300 frames between lunges).
pub(crate) const ORCA_LUNGE_COOLDOWN: u16 = 300;

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
/// HP lookup table for TarMonster init (`Hitpoints_Enemy_05`, Tar_Monster.asm).
pub(crate) const TAR_MONSTER_HITPOINTS_BY_LEVEL: [u8; 4] = [2, 4, 6, 0];

#[allow(non_snake_case)]
pub(crate) fn tar_monster_hit_points_loc_321c6(enemy_level: u8) -> u8 {
    // ASM loc_321C6 indexes `dc.b Hitpoints_Enemy_05(pc,d3.w),d3`.
    TAR_MONSTER_HITPOINTS_BY_LEVEL[usize::from(enemy_level.min(3))]
}

/// Fire Demon death trail: one visual trail object at the death position.
pub(crate) fn fire_demon_death_trail_child_loc_3a63c(x: u32, y: u32) -> FireDemonTrailChild {
    // ASM Fire_Demon loc_3A63C copies the parent x/y, then `addi.w #$F,y_pos`
    // and `andi.w #$FFF0,y_pos` before initialising `stru_3A622`. The child
    // waits with d3=$96 and has no velocity.
    let y_px = ((y >> 16) as u16).wrapping_add(0x000F) & 0xFFF0;
    FireDemonTrailChild {
        x_pos: x,
        y_pos: u32::from(y_px) << 16,
        remaining_frames: 0x96,
    }
}

#[cfg(test)]
pub(crate) fn spawn_fire_demon_death_trail(projectiles: &mut Vec<EnemyProjectile>, x: u32, y: u32) {
    // ASM Fire_Demon loc_3A63C copies the parent x/y, then `addi.w #$F,y_pos`
    // and `andi.w #$FFF0,y_pos` before initialising `stru_3A622`.  The object
    // waits with d3=$96 and has no velocity; native keeps it in the projectile
    // list only so the renderer can draw the ROM fire-trail footprint.
    let child = fire_demon_death_trail_child_loc_3a63c(x, y);
    projectiles.push(EnemyProjectile {
        x_pos: child.x_pos,
        y_pos: child.y_pos,
        x_velocity: 0,
        y_velocity: 0,
        lifetime: child.remaining_frames,
        gravity: 0,
        friendly: false,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
        kind: EnemyProjectileKind::FireDemonDeathTrail,
        ..Default::default()
    });
}

// --- DragonWalking constants ---
// From ASM Dragon.asm: initial cooldown $12C (300f, line 95), reset after breath $1F4 (500f, line 194).
pub(crate) const DRAGON_WALKING_BREATH_INITIAL: u16 = 300;
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
pub(crate) fn spawn_shared_walker_state(family: EnemyFamily) -> i16 {
    match family {
        EnemyFamily::DragonWalking => {
            dragon_walking_common_init_loc_3ce5a(false, Direction::Left).shared_walker_state
        }
        EnemyFamily::Lion => SHARED_WALKER_STATE_PATROL,
        _ => 0,
    }
}

pub(crate) fn spawn_shared_walker_patrol_timer(family: EnemyFamily) -> u16 {
    match family {
        EnemyFamily::DragonWalking => {
            dragon_walking_common_init_loc_3ce5a(false, Direction::Left).patrol_timer
        }
        EnemyFamily::Lion => LION_CHARGE_TIMER_INIT,
        EnemyFamily::Ninja => NINJA_ATTACK_TIMER_INIT_LOC_3D5AA,
        _ => 0,
    }
}

pub(crate) fn spawn_shared_walker_patrol_short_next(family: EnemyFamily) -> bool {
    matches!(family, EnemyFamily::DragonWalking | EnemyFamily::Ninja)
}

pub(crate) fn spawn_tank_mouth_animation_phase(family: EnemyFamily) -> u8 {
    match family {
        // ASM Tank.asm Enemy0A_RockTank_Init loads the linked mouth child with
        // `stru_3B27E`, then the parent hibernates once through the shared
        // walker before loc_3B41C can test the child's `$22/$32` cue fields.
        EnemyFamily::Tank => 1,
        _ => 0,
    }
}

pub(crate) fn spawn_shared_walker_right_turn_extra_step_consumed(
    _family: EnemyFamily,
    _facing: Direction,
) -> bool {
    false
}

pub(crate) fn spawn_lion_state_loc_3e030(family: EnemyFamily) -> LionState {
    match family {
        // ASM Lion.asm loc_3E030 dispatches the freshly initialised Lion
        // directly into loc_3E392/loc_3E546, the direction-specific charge
        // locomotion paths. The `$54(a5)` roar timer still counts down from
        // loc_3E078 while that locomotion runs.
        EnemyFamily::Lion => LionState::Charging,
        _ => LionState::Walking,
    }
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

fn dragon_walking_patrol_timer_pair_loc_3dea0(short_next: bool) -> u32 {
    // ASM Dragon.asm loc_3CDFC seeds `$6E(a5)=$0032_012C`; Scorpion.asm
    // loc_3DEA0 later rotates that packed short/long pair before reloading
    // `$54(a5)`.
    let (high, low) = if short_next {
        (
            DRAGON_WALKING_PATROL_SHORT_FRAMES,
            DRAGON_WALKING_PATROL_LONG_FRAMES,
        )
    } else {
        (
            DRAGON_WALKING_PATROL_LONG_FRAMES,
            DRAGON_WALKING_PATROL_SHORT_FRAMES,
        )
    };
    u32::from(high) << 16 | u32::from(low)
}

fn dragon_walking_patrol_short_next_loc_3dea0(timer_pair_6e: u32) -> bool {
    timer_pair_6e == dragon_walking_patrol_timer_pair_loc_3dea0(true)
}

fn shared_walker_tick_patrol(enemy: &mut ActiveEnemy) -> bool {
    if !shared_walker_should_tick_patrol(enemy) || enemy.shared_walker_patrol_timer == 0 {
        return false;
    }

    if matches!(enemy.family, EnemyFamily::DragonWalking) {
        let gate = scorpion_timer_gate_loc_3dea0(
            enemy.shared_walker_state,
            enemy.shared_walker_patrol_timer,
            dragon_walking_patrol_timer_pair_loc_3dea0(enemy.shared_walker_patrol_short_next),
            enemy.facing,
        );
        enemy.shared_walker_patrol_timer = gate.timer_54;
        return match gate.route {
            ScorpionTimerRouteLoc3DEA0::ResumeSharedWalker { next } => {
                let resume_facing = match next {
                    ScorpionTimerResumeRouteLoc3DECE::ClearDirection => Direction::Left,
                    ScorpionTimerResumeRouteLoc3DECE::SetDirection => Direction::Right,
                };
                debug_assert_eq!(resume_facing, enemy.facing);
                false
            }
            ScorpionTimerRouteLoc3DEA0::RestartPacing { next } => {
                let restart_facing = match next {
                    ScorpionTimerRestartRouteLoc3DEA0::ClearDirection => Direction::Left,
                    ScorpionTimerRestartRouteLoc3DEA0::SetDirection => Direction::Right,
                };
                debug_assert_eq!(restart_facing, enemy.facing);
                enemy.shared_walker_patrol_short_next =
                    dragon_walking_patrol_short_next_loc_3dea0(gate.alternating_timer_pair_6e);
                true
            }
        };
    }

    enemy.shared_walker_patrol_timer -= 1;
    if enemy.shared_walker_patrol_timer != 0 {
        return false;
    }

    enemy.shared_walker_patrol_timer = match enemy.family {
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
    current_center_x_px: i32,
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
    let body_probe_ys = [
        lower_body_y,
        foot_y_px - ENEMY_PROBE_HEIGHT / 2,
        upper_body_y,
    ];

    let leading_side_boundary_offset = ENEMY_PROBE_HALF_WIDTH + 1;
    let leading_boundary_distance_px = if moving_right {
        (-(current_center_x_px + leading_side_boundary_offset)).rem_euclid(16)
    } else {
        (current_center_x_px - leading_side_boundary_offset).rem_euclid(16)
    };
    let step_px = if moving_right {
        center_x_px - current_center_x_px
    } else {
        current_center_x_px - center_x_px
    };
    if !moving_right && step_px > leading_boundary_distance_px {
        // ASM sub_36A58: the left-facing `$48(a5)` side check only reaches
        // the leading bottom-side tile when the proposed move crosses that
        // tile boundary. Native keeps the body-row probes unconditional but
        // gates this foot-row probe the same way to avoid treating ordinary
        // floor support as a wall.
        let probe = terrain::probe_at(grid, edge_x, foot_y_px);
        if probe.cell.collision == terrain::CollisionKind::SlopeUp {
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

    for &py in &body_probe_ys {
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
    let wall = dragon_walking_probe_wall(grid, cx, cx, cy, moving_right);
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

fn step_dragon_walking_airborne_after_x_commit(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) {
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    let Some(grid) = block_grid else {
        enemy.y_velocity = (enemy.y_velocity + DRAGON_WALKING_GRAVITY).min(physics::MAX_FALL_SPEED);
        enemy.shared_walker_debug_branch = Some("airborne_fall");
        return;
    };

    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    let floor = terrain::probe_floor(grid, cx, cy);
    enemy.shared_walker_terrain_trace = SharedWalkerTerrainTrace {
        next_x_px: Some(cx),
        wall: None,
        center_floor: Some(floor),
        left_floor: None,
        right_floor: None,
    };

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
/// Native-only sentinel set at the ceiling-attach frame. The loc_3340E RNG
/// byte is drawn in a post-pass over all same-frame attaches because ASM
/// `Allocate_ObjectSlot` (kid.asm loc_DE0: `cmp.w $E(a1),d6; bhi.s loc_DE0`)
/// inserts equal-priority objects BEFORE existing slots, so `Execute_Objects`
/// (loc_E1E) runs simultaneously loaded enemies in reverse load order and
/// their loc_3340E draws happen in descending layout order.
pub(crate) const DRIPS_INITIAL_SEED_PENDING_TIMER_LOC_3340E: u16 = u16::MAX - 1;
pub(crate) fn drips_wait_timer_with_rng_loc_3340e(rng: &mut RomRandomState, kid_x_pos: u32) -> u16 {
    let x_pos_bytes = kid_x_pos.to_be_bytes();
    let kid_x_word = u16::from_be_bytes([x_pos_bytes[0], x_pos_bytes[1]]);
    drips_wait_timer_loc_3340e(rng.next_byte(), kid_x_word)
}

/// Step a Drips enemy: stationary ceiling hanger that periodically drops projectiles.
/// ASM: Drips.asm — forming animation → drop at 10px/frame → splash → reform → repeat.
/// Native approximation uses a dedicated physics projectile for the preallocated falling child;
/// terrain splash can then spawn a hostile surface crawler through loc_335D8.
/// Initial rising velocity for the Drips ceiling-attach phase.
/// ASM Enemy16_Drip_Init (Drips.asm:35): `move.l #-$10000,y_vel(a3)` seeds
/// the slime with -1.0 fp/frame upward velocity before the loc_333F6 loop.
pub(crate) const DRIPS_CEILING_ATTACH_RISE_VELOCITY: i32 = -0x1_0000;

pub(crate) fn drips_object_check_in_range_loc_333f6(
    enemy: &ActiveEnemy,
    camera_x_px: Option<u32>,
    camera_y_px: Option<u32>,
) -> bool {
    let (Some(camera_x_px), Some(camera_y_px)) = (camera_x_px, camera_y_px) else {
        return true;
    };
    // ASM Drips.asm loc_333F6/loc_3345C calls `Object_CheckInRange` before the
    // attach and falling-child loops. Native retains layout-backed active
    // enemies instead of deleting the object, so an out-of-range controller
    // pauses here rather than consuming `off_33520` and spawning loc_335D8 early.
    let object_x_pos = (enemy.x_pos >> 16) as i16;
    let object_y_pos = (enemy.y_pos >> 16) as i16;
    let camera_x_pos = camera_x_px.min(u32::from(u16::MAX)) as i16;
    let camera_y_pos = camera_y_px.min(u32::from(u16::MAX)) as i16;
    crate::level::object_check_in_range_normal_x_loc_320C0(object_x_pos, camera_x_pos)
        && crate::level::object_check_in_range_normal_y_loc_320D8(object_y_pos, camera_y_pos)
}

pub(crate) const SPINNING_TWINS_BERZERKER_LAUNCH_WAIT_TIMER_LOC_3BC22: u16 = 0xBC22;

fn step_spinning_twins_berzerker_launch(enemy: &mut ActiveEnemy) {
    // ASM Spinning_Twins.asm loc_3BC22: hibernate one frame, run the shared
    // retention check `sub_3C3CE`, then add `$4000` to y_vel and loop.
    enemy.x_velocity = 0;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    enemy.y_velocity =
        spinning_twins_berzerker_launch_loop_tick_loc_3bc22(enemy.y_velocity).next_y_velocity;
    enemy.on_ground = false;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmoRockClearanceWindowLoc3639E {
    pub(crate) top_row: u16,
    pub(crate) bottom_row: u16,
    pub(crate) left_col: u16,
    pub(crate) right_col: u16,
    pub(crate) row_dbf_count: u16,
    pub(crate) col_dbf_count: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmoRockClearanceWindowRouteLoc3639E {
    Scan(EmoRockClearanceWindowLoc3639E),
    Blocked,
}

pub(crate) fn emo_rock_clearance_window_loc_3639e(
    level_width_pixels: i32,
    x_px: i32,
    y_px: i32,
    height_px: u16,
    half_width_px: u16,
) -> EmoRockClearanceWindowRouteLoc3639E {
    // ASM Emo_Rock.asm loc_3639E: subtract the height/half-width from y/x,
    // reject negative top/left edges and right edges at or beyond level width,
    // then convert the inclusive pixel rectangle into DBF row/column counts.
    let top_px = y_px - i32::from(height_px);
    if top_px < 0 {
        return EmoRockClearanceWindowRouteLoc3639E::Blocked;
    }

    let left_px = x_px - i32::from(half_width_px);
    if left_px < 0 {
        return EmoRockClearanceWindowRouteLoc3639E::Blocked;
    }

    let right_px = x_px + i32::from(half_width_px);
    if right_px >= level_width_pixels {
        return EmoRockClearanceWindowRouteLoc3639E::Blocked;
    }

    let Some(top_row) = u16::try_from(top_px >> 4).ok() else {
        return EmoRockClearanceWindowRouteLoc3639E::Blocked;
    };
    let Some(bottom_row) = u16::try_from(y_px >> 4).ok() else {
        return EmoRockClearanceWindowRouteLoc3639E::Blocked;
    };
    let Some(left_col) = u16::try_from(left_px >> 4).ok() else {
        return EmoRockClearanceWindowRouteLoc3639E::Blocked;
    };
    let Some(right_col) = u16::try_from(right_px >> 4).ok() else {
        return EmoRockClearanceWindowRouteLoc3639E::Blocked;
    };

    EmoRockClearanceWindowRouteLoc3639E::Scan(EmoRockClearanceWindowLoc3639E {
        top_row,
        bottom_row,
        left_col,
        right_col,
        row_dbf_count: bottom_row.saturating_sub(top_row),
        col_dbf_count: right_col.saturating_sub(left_col),
    })
}

pub(crate) fn emo_rock_clearance_cell_blocked_loc_363e0(cell: terrain::Cell) -> bool {
    // ASM Emo_Rock.asm loc_363E0: each scanned word is masked with `$4000`;
    // any set bit branches to the blocked return at loc_363FA.
    cell.is_solid_floor()
}

pub(crate) fn emo_rock_clearance_rows_loc_363dc(
    grid: &terrain::BlockGrid,
    window: EmoRockClearanceWindowLoc3639E,
) -> bool {
    // ASM Emo_Rock.asm loc_363DC: each outer DBF row restores d7 from d6 and
    // a1 from a0, so every row scans the full left-to-right column span.
    for row in window.top_row..=window.bottom_row {
        for col in window.left_col..=window.right_col {
            if emo_rock_clearance_cell_blocked_loc_363e0(grid.cell_at(col, row)) {
                return false;
            }
        }
    }
    true
}

pub(crate) fn emo_rock_clearance_probe_loc_3639e(
    grid: &terrain::BlockGrid,
    x_px: i32,
    y_px: i32,
    height_px: u16,
    half_width_px: u16,
) -> bool {
    match emo_rock_clearance_window_loc_3639e(
        grid.width_pixels(),
        x_px,
        y_px,
        height_px,
        half_width_px,
    ) {
        EmoRockClearanceWindowRouteLoc3639E::Scan(window) => {
            emo_rock_clearance_rows_loc_363dc(grid, window)
        }
        EmoRockClearanceWindowRouteLoc3639E::Blocked => false,
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

fn step_crab_berzerker_knockback(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    // ASM Crab.asm loc_3B1A6-loc_3B254: the Berzerker side-contact launch
    // owns movement until x_vel decelerates into the settle window, then
    // clears movement/level-collision flags and resumes loc_3B0BE.
    let deceleration = crab_airborne_deceleration_frame_loc_3b1a6(enemy.x_velocity);
    enemy.x_velocity = match deceleration.route {
        CrabAirborneDecelerationRouteLoc3B1A6::NonNegativeXVelocity { x_velocity } => x_velocity,
        CrabAirborneDecelerationRouteLoc3B1A6::NegativeXVelocityTail => {
            crab_negative_airborne_deceleration_loc_3b1be(enemy.x_velocity).x_velocity
        }
    };

    if matches!(
        crab_airborne_threshold_route_loc_3b1c6(enemy.x_velocity),
        CrabAirborneThresholdRouteLoc3B1C6::Settle
    ) {
        let settle = crab_airborne_settle_tail_loc_3b254();
        enemy.x_velocity = settle.x_velocity;
        enemy.y_velocity = settle.y_velocity;
        enemy.on_ground = true;
        enemy.crab_berzerker_knockback = false;
        return;
    }

    let gravity = crab_gravity_tail_loc_3b218(enemy.x_velocity, enemy.y_velocity);
    enemy.x_velocity = gravity.x_velocity;
    enemy.y_velocity = gravity.y_velocity;
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    enemy.on_ground = false;

    let Some(grid) = block_grid else {
        return;
    };

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
        let collision_type = if moving_right {
            CRAB_COLID_RIGHTWALL
        } else {
            CRAB_COLID_LEFTWALL
        };
        // ASM Crab.asm loc_3B1DA: Berzerker knockback wall contacts route
        // through the airborne collision dispatch before loc_3B24C reverses x_vel.
        if matches!(
            crab_airborne_collision_dispatch_loc_3b1da(collision_type),
            CrabAirborneCollisionRouteLoc3B1DA::ReverseXVelocity
        ) {
            enemy.x_velocity = crab_reverse_x_velocity_loc_3b24c(enemy.x_velocity);
        }
        return;
    }

    let floor = terrain::probe_floor(grid, cx, cy);
    let slope_collision_type = match floor.cell.collision {
        terrain::CollisionKind::SlopeUp if floor.on_ground => Some(CRAB_COLID_SLOPEUP),
        terrain::CollisionKind::SlopeDown if floor.on_ground => Some(CRAB_COLID_SLOPEDOWN),
        _ => None,
    };
    let Some(collision_type) = slope_collision_type else {
        return;
    };

    // ASM Crab.asm loc_3B1DA: slope contacts fall through the airborne
    // collision dispatch before loc_3B1F8 chooses the response.
    let CrabAirborneCollisionRouteLoc3B1DA::SlopeTail { collision_type } =
        crab_airborne_collision_dispatch_loc_3b1da(collision_type)
    else {
        return;
    };

    match crab_slope_collision_response_loc_3b1f8(
        collision_type,
        enemy.x_velocity,
        enemy.y_velocity,
    ) {
        CrabSlopeCollisionRouteLoc3B1F8::ReverseXVelocity { x_velocity } => {
            enemy.x_velocity = x_velocity;
        }
        CrabSlopeCollisionRouteLoc3B1F8::ReflectYVelocity { y_velocity } => {
            enemy.y_velocity = y_velocity;
        }
        CrabSlopeCollisionRouteLoc3B1F8::GravityTail => {
            // Native has already applied the collapsed loc_3B218 gravity tail
            // before this terrain probe in the current airborne frame.
        }
    }
}

fn step_fire_demon_berzerker_launch(enemy: &mut ActiveEnemy) {
    // ASM Fire_Demon.asm loc_3A610: the launched demon hibernates one frame,
    // remains active via sub_3C3CE, applies movement, then adds `$4000` to
    // y_vel and loops without re-entering the shared walker.
    enemy.x_velocity = 0;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    enemy.y_velocity =
        fire_demon_berzerker_launch_loop_tick_loc_3a610(enemy.y_velocity).next_y_velocity;
    enemy.on_ground = false;
}

fn step_goat_berzerker_knockback(enemy: &mut ActiveEnemy) {
    // ASM Goat.asm loc_3CBA8: hibernate one frame, run the motion helper,
    // add `$4000` to y_vel, and repeat the airborne Berzerker loop.
    enemy.x_velocity = 0;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    enemy.y_velocity = goat_berzerker_airborne_frame_loc_3cba8(enemy.y_velocity).next_y_velocity;
    enemy.on_ground = false;
}

fn step_dragon_walking_berzerker_knockback(enemy: &mut ActiveEnemy) {
    // ASM Dragon.asm loc_3CF3A: hibernate one frame, run the motion helper,
    // add `$4000` to y_vel, and repeat the airborne Berzerker loop.
    enemy.x_velocity = 0;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    let frame = dragon_walking_berzerker_airborne_frame_loc_3cf3a(enemy.y_velocity);
    debug_assert!(frame.applies_motion_subroutine);
    debug_assert!(frame.repeats_airborne_loop);
    enemy.y_velocity = frame.next_y_velocity;
    enemy.on_ground = false;
}

fn step_armadillo_berzerker_launch(enemy: &mut ActiveEnemy) {
    // ASM Armadillo.asm loc_3C8CC: hibernate one frame, apply shared motion,
    // add `$4000` to y_vel, and repeat the launched-armadillo loop.
    enemy.x_velocity = 0;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    enemy.y_velocity =
        armadillo_berzerker_launch_loop_tick_loc_3c8cc(enemy.y_velocity).next_y_velocity;
    enemy.on_ground = false;
}

fn step_armadillo_spawned_child_fall_wait_loc_3c954(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) {
    // ASM Armadillo.asm loc_3C954: after loc_3C93C seeds the fall, the child
    // hibernates one frame, applies movement/collision, and enters loc_3C886
    // only when the post-hibernate collision word is `colid_floor`.
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    let collision_type = if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground && enemy.y_velocity >= 0 {
            let floor_y_fp = (floor.floor_y_px as u32) << 16;
            if enemy.y_pos >= floor_y_fp {
                enemy.y_pos = floor_y_fp;
                ARMADILLO_COLID_FLOOR
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    let wait = armadillo_spawned_child_fall_wait_loc_3c954(
        collision_type,
        enemy.y_velocity,
        enemy.layout_index as u8,
    );
    enemy.x_velocity = 0;
    enemy.y_velocity = wait.y_velocity;
    enemy.on_ground = !wait.has_level_collision;
    if matches!(
        wait.route,
        ArmadilloSpawnedChildFallWaitRouteLoc3C954::DeathAnimation { .. }
    ) {
        enemy.death_animation_active = true;
        enemy.wait_timer = ARMADILLO_DEATH_ANIMATION_FRAMES;
    } else {
        enemy.wait_timer = ARMADILLO_SPAWNED_CHILD_FALL_WAIT_TIMER_LOC_3C954;
    }
}

fn step_scorpion_berzerker_knockback(enemy: &mut ActiveEnemy) {
    // ASM Scorpion.asm loc_3DD4A: hibernate one frame, run sub_3E8BA motion,
    // add `$4000` to y_vel, and repeat the launched Scorpion loop.
    enemy.x_velocity = 0;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    enemy.y_velocity =
        scorpion_berzerker_airborne_frame_loc_3dd4a(enemy.y_velocity).next_y_velocity;
    enemy.on_ground = false;
}

fn tank_shooting_cycle_frames(hit_points: u8) -> u16 {
    // ASM Tank_Shooting.asm loc_3B682: `cmpi.w #4,d7; bne.s loc_3B682`
    // installs the slow turret animation for every value except exact 4.
    if hit_points == 4 {
        TANK_SHOOTING_FAST_CYCLE_FRAMES
    } else {
        TANK_SHOOTING_SLOW_CYCLE_FRAMES
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TankInitWalkerEntryLoc3B40A {
    LeftLoc3C0D2,
    RightLoc3C026,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TankInitDriveSoundLoc3B40A {
    pub(crate) next_tank_drive_refcount: u8,
    pub(crate) sound_call: Option<SharedEnemyDriveSoundCallSub3C352>,
    pub(crate) walker_entry: TankInitWalkerEntryLoc3B40A,
}

pub(crate) fn tank_init_drive_sound_loc_3b40a(
    tank_drive_refcount: u8,
    motion_active_flag_19: bool,
    x_direction: u8,
) -> TankInitDriveSoundLoc3B40A {
    // ASM Tank.asm loc_3B40A: after the optional
    // `j_PlaySound(sfx_Tank_driving)`, increment the shared tank-drive
    // refcount and branch by x_direction. Zero enters loc_3C0D2; nonzero
    // enters loc_3C026.
    let shared_start =
        shared_enemy_drive_sound_sub_3C352(EnemyFamily::Tank, false, true, tank_drive_refcount);
    debug_assert!(!shared_start.next_previous_motion_active);
    TankInitDriveSoundLoc3B40A {
        next_tank_drive_refcount: shared_start.next_refcount,
        sound_call: if motion_active_flag_19 {
            None
        } else {
            shared_start.sound_call
        },
        walker_entry: if x_direction == 0 {
            TankInitWalkerEntryLoc3B40A::LeftLoc3C0D2
        } else {
            TankInitWalkerEntryLoc3B40A::RightLoc3C026
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TankMouthFrameLoc3B41C {
    OpeningTimerA,
    ClosingTimer37,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TankMouthSoundLoc3B41C {
    Open,
    Closed,
}

impl TankMouthSoundLoc3B41C {
    fn queued_sound_effect(self) -> QueuedSoundEffect {
        match self {
            Self::Open => QueuedSoundEffect::TankMouthOpen,
            Self::Closed => QueuedSoundEffect::TankMouthClosed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TankMouthAnimationStepLoc3B27E {
    pub(crate) mouth_frame: TankMouthFrameLoc3B41C,
    pub(crate) next_animation_phase: u8,
}

pub(crate) const TANK_MOUTH_ANIMATION_FRAMES_LOC_3B27E: u16 = 0x37 + 0x0A + 6 + 4 + 5;
pub(crate) const TANK_MOUTH_OPEN_PHASE_LOC_3B27E: u16 = 0x37;

pub(crate) fn tank_mouth_animation_step_loc_3b27e(
    animation_phase: u8,
) -> TankMouthAnimationStepLoc3B27E {
    // ASM ingame/anim/enemy/Tank.asm `stru_3B27E`: closed `$37`,
    // open `$A`, transition `6`, open `4`, closed `5`, loop. Tank.asm
    // loc_3B41C/loc_3B446 only plays sounds on the first tick of the
    // `$A` open frame and the first tick of the loop's `$37` closed frame.
    let phase = u16::from(animation_phase) % TANK_MOUTH_ANIMATION_FRAMES_LOC_3B27E;
    let mouth_frame = match phase {
        0 => TankMouthFrameLoc3B41C::ClosingTimer37,
        TANK_MOUTH_OPEN_PHASE_LOC_3B27E => TankMouthFrameLoc3B41C::OpeningTimerA,
        _ => TankMouthFrameLoc3B41C::Other,
    };

    TankMouthAnimationStepLoc3B27E {
        mouth_frame,
        next_animation_phase: ((phase + 1) % TANK_MOUTH_ANIMATION_FRAMES_LOC_3B27E) as u8,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TankUpdateRouteLoc3B41C {
    Walk,
    Hurt { remaining_hp: u16 },
    Death,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TankUpdateLoc3B41C {
    pub(crate) mouth_sound: Option<TankMouthSoundLoc3B41C>,
    pub(crate) next_current_hp: u16,
    pub(crate) clears_child_collision_type: bool,
    pub(crate) hides_child_sprite: bool,
    pub(crate) sets_child_kid_collision: bool,
    pub(crate) clears_parent_motion_speed: bool,
    pub(crate) child_recoil_timer: Option<u16>,
    pub(crate) hurt_hibernate_frames: Option<u16>,
    pub(crate) route: TankUpdateRouteLoc3B41C,
}

fn tank_collision_word_is_negative_loc_3b41c(collision_type: u16) -> bool {
    collision_type & 0x8000 != 0
}

pub(crate) fn tank_update_loc_3b41c(
    parent_collision_type: u16,
    child_collision_type: u16,
    current_hp: u16,
    mouth_frame: TankMouthFrameLoc3B41C,
    motion_active_flag_19: bool,
) -> TankUpdateLoc3B41C {
    // ASM Tank.asm loc_3B41C: a negative parent collision routes directly to
    // the death path. Otherwise the mouth child can play open/closed sounds
    // on its exact animation-frame timers, then child collision `$38(a2)`
    // decides whether to keep walking, hurt, or delete the tank.
    if tank_collision_word_is_negative_loc_3b41c(parent_collision_type) {
        return TankUpdateLoc3B41C {
            mouth_sound: None,
            next_current_hp: current_hp,
            clears_child_collision_type: false,
            hides_child_sprite: true,
            sets_child_kid_collision: true,
            clears_parent_motion_speed: false,
            child_recoil_timer: None,
            hurt_hibernate_frames: None,
            route: TankUpdateRouteLoc3B41C::Death,
        };
    }

    let mouth_sound = if motion_active_flag_19 {
        None
    } else {
        match mouth_frame {
            // ASM Tank.asm loc_3B444: after the optional open-mouth sound,
            // execution immediately joins the child collision routing.
            TankMouthFrameLoc3B41C::OpeningTimerA => Some(TankMouthSoundLoc3B41C::Open),
            // ASM Tank.asm loc_3B446: non-open mouth frames check for
            // closing timer `$37` before optionally playing the closed sound.
            TankMouthFrameLoc3B41C::ClosingTimer37 => Some(TankMouthSoundLoc3B41C::Closed),
            TankMouthFrameLoc3B41C::Other => None,
        }
    };

    // ASM Tank.asm loc_3B464: the mouth child collision word chooses
    // walk, death, or damage after mouth-sound handling has completed.
    if tank_collision_word_is_negative_loc_3b41c(child_collision_type) {
        return TankUpdateLoc3B41C {
            mouth_sound,
            next_current_hp: current_hp,
            clears_child_collision_type: false,
            hides_child_sprite: true,
            sets_child_kid_collision: true,
            clears_parent_motion_speed: false,
            child_recoil_timer: None,
            hurt_hibernate_frames: None,
            route: TankUpdateRouteLoc3B41C::Death,
        };
    }

    if !matches!(child_collision_type, 0x001C | 0x002C) {
        return TankUpdateLoc3B41C {
            mouth_sound,
            next_current_hp: current_hp,
            clears_child_collision_type: true,
            hides_child_sprite: false,
            sets_child_kid_collision: false,
            clears_parent_motion_speed: false,
            child_recoil_timer: None,
            hurt_hibernate_frames: None,
            route: TankUpdateRouteLoc3B41C::Walk,
        };
    }

    // ASM Tank.asm loc_3B478: damage collisions `$1C` and `$2C` subtract one
    // from current_hp before branching on the decremented value.
    let remaining_hp = current_hp.wrapping_sub(1);
    if remaining_hp == 0 {
        // ASM Tank.asm loc_3B480: zero HP hides the mouth child, restores
        // Kid collision on that child, runs the death animation, then exits
        // through the shared deactivation path.
        TankUpdateLoc3B41C {
            mouth_sound,
            next_current_hp: remaining_hp,
            clears_child_collision_type: false,
            hides_child_sprite: true,
            sets_child_kid_collision: true,
            clears_parent_motion_speed: false,
            child_recoil_timer: None,
            hurt_hibernate_frames: None,
            route: TankUpdateRouteLoc3B41C::Death,
        }
    } else {
        // ASM Tank.asm loc_3B49A: nonfatal damage clears parent motion,
        // sets the mouth child recoil timer to 6, and hibernates for 5 frames
        // before resuming normal walker cleanup.
        TankUpdateLoc3B41C {
            mouth_sound,
            next_current_hp: remaining_hp,
            clears_child_collision_type: true,
            hides_child_sprite: false,
            sets_child_kid_collision: false,
            clears_parent_motion_speed: true,
            child_recoil_timer: Some(6),
            hurt_hibernate_frames: Some(5),
            route: TankUpdateRouteLoc3B41C::Hurt { remaining_hp },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TankShootingResumeWalkBranchLoc3B720 {
    LeftSharedWalker,
    RightSharedWalker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TankShootingResumeWalkLoc3B720 {
    pub(crate) next_child_collision_type: u16,
    pub(crate) branch: TankShootingResumeWalkBranchLoc3B720,
}

pub(crate) fn tank_shooting_resume_walk_loc_3b720(
    x_direction: u8,
) -> TankShootingResumeWalkLoc3B720 {
    // ASM Tank_Shooting.asm loc_3B720: clear turret child `$38(a2)`, then
    // branch by `x_direction(a3)`; zero enters loc_3C0FA, nonzero loc_3C04E.
    TankShootingResumeWalkLoc3B720 {
        next_child_collision_type: 0,
        branch: if x_direction == 0 {
            TankShootingResumeWalkBranchLoc3B720::LeftSharedWalker
        } else {
            TankShootingResumeWalkBranchLoc3B720::RightSharedWalker
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TankShootingHurtRecoveryLoc3B700 {
    pub(crate) parent_motion_speed: i32,
    pub(crate) child_recoil_timer: u16,
    pub(crate) child_animation_script: &'static str,
    pub(crate) hibernate_frames: u16,
    pub(crate) resume_walk: TankShootingResumeWalkLoc3B720,
}

pub(crate) fn tank_shooting_hurt_recovery_loc_3b700(
    x_direction: u8,
) -> TankShootingHurtRecoveryLoc3B700 {
    // ASM Tank_Shooting.asm loc_3B700: clear parent motion `$50(a5)`, set
    // turret timer `$46(a2)=6`, install `stru_3B294`, hibernate five frames,
    // then fall through to loc_3B720.
    TankShootingHurtRecoveryLoc3B700 {
        parent_motion_speed: 0,
        child_recoil_timer: 6,
        child_animation_script: "stru_3B294",
        hibernate_frames: 5,
        resume_walk: tank_shooting_resume_walk_loc_3b720(x_direction),
    }
}

fn tank_shooting_hurt_hibernate_frames_remaining(damage_flash_frames: u8) -> u8 {
    // Native uses the shared damage-flash countdown as the visible part of
    // ASM Tank_Shooting.asm loc_3B700's `j_Hibernate_Object(5)` window.
    damage_flash_frames.saturating_sub(
        ENEMY_DAMAGE_FLASH_FRAMES
            .saturating_sub(TANK_SHOOTING_HURT_HIBERNATE_FRAMES)
            .saturating_sub(1),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TankShootingInitWalkerEntryLoc3B690 {
    LeftLoc3C0D2,
    RightLoc3C026,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TankShootingInitDriveSoundLoc3B690 {
    pub(crate) next_tank_drive_refcount: u8,
    pub(crate) sound_call: Option<SharedEnemyDriveSoundCallSub3C352>,
    pub(crate) walker_entry: TankShootingInitWalkerEntryLoc3B690,
}

pub(crate) fn tank_shooting_init_drive_sound_loc_3b690(
    tank_drive_refcount: u8,
    motion_active_flag_19: bool,
    x_direction: u8,
) -> TankShootingInitDriveSoundLoc3B690 {
    // ASM Tank_Shooting.asm loc_3B690: test the shared tank-drive refcount
    // at `$FFFFFA34+d5`, then test `$19(a3)`. Only a zero refcount and clear
    // motion-active flag call `j_PlaySound(sfx_Tank_driving)`.
    // ASM Tank_Shooting.asm loc_3B6AE: the shared refcount increments
    // unconditionally before branching by x_direction: zero enters loc_3C0D2,
    // nonzero enters loc_3C026.
    let shared_start = shared_enemy_drive_sound_sub_3C352(
        EnemyFamily::TankShooting,
        false,
        true,
        tank_drive_refcount,
    );
    debug_assert!(!shared_start.next_previous_motion_active);
    TankShootingInitDriveSoundLoc3B690 {
        next_tank_drive_refcount: shared_start.next_refcount,
        sound_call: if motion_active_flag_19 {
            None
        } else {
            shared_start.sound_call
        },
        walker_entry: if x_direction == 0 {
            TankShootingInitWalkerEntryLoc3B690::LeftLoc3C0D2
        } else {
            TankShootingInitWalkerEntryLoc3B690::RightLoc3C026
        },
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
    played_sound_effects: Option<&mut Vec<QueuedSoundEffect>>,
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
        kind: EnemyProjectileKind::TankBullet,
        ..Default::default()
    });

    // ASM Tank_Shooting.asm loc_3B730 (line 190): one hibernate frame after the
    // shot worker is loaded, `moveq #sfx_Tank_shoots,d0; jsr j_PlaySound` plays
    // $41. The native spawn edge above is the equivalent of that worker load.
    if let Some(sound_effects) = played_sound_effects {
        sound_effects.push(QueuedSoundEffect::TankShoots);
    }
}

/// Boss tracking movement: chase the player with fixed velocity components.
/// HeadyMetal ASM loc_37582: x_vel ±$8000 towards player, y_vel $4800 down /
/// `$FFFFB000` (`-0x5000`) up. Other bosses use the same substitute pattern
/// until their ROM waypoint scripts are wired.
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

    if matches!(enemy.family, EnemyFamily::HeadyMetal) {
        // ASM loc_3763A calls sub_379E8 once per live HeadyMetal tracker loop,
        // advancing `d2` and writing the selected wave colour to Palette_Buffer+$5E.
        let palette_cycle =
            heady_metal_palette_cycle_sub_379E8(enemy.heady_metal_palette_phase_counter);
        enemy.heady_metal_palette_phase_counter = palette_cycle.next_phase_counter;
        enemy.heady_metal_palette_word_5e = palette_cycle.palette_word;

        let tracking = heady_metal_tracking_velocity_loc_37582(
            (px >> 16) as u16,
            (py >> 16) as u16,
            (enemy.x_pos >> 16) as u16,
            (enemy.y_pos >> 16) as u16,
            37u16.saturating_sub(u16::from(enemy.hit_points)),
            enemy.boss_movement_script_index != 0,
        );
        enemy.x_velocity = tracking.x_velocity;
        enemy.y_velocity = tracking.y_velocity;
        enemy.facing = if (px as i64) > ex {
            Direction::Right
        } else {
            Direction::Left
        };
        enemy.x_pos = (ex + i64::from(enemy.x_velocity)).clamp(0, i64::from(u32::MAX)) as u32;
        enemy.y_pos = (ey + i64::from(enemy.y_velocity)).clamp(0, i64::from(u32::MAX)) as u32;
        return;
    }

    // Horizontal: chase player at $8000 (0.5 px/frame)
    let x_chase_speed: i32 = 0x8000;
    if (px as i64) > ex {
        enemy.x_velocity = x_chase_speed;
        enemy.facing = Direction::Right;
    } else {
        enemy.x_velocity = -x_chase_speed;
        enemy.facing = Direction::Left;
    }

    // Vertical: remaining tracker-family bosses still use the temporary
    // player-relative substitute until their ASM movement scripts are wired.
    let target_y = (py as i64).saturating_sub(0x28 << 16);
    let move_up = target_y < ey;

    if move_up {
        enemy.y_velocity = -0x5000;
    } else {
        enemy.y_velocity = 0x4800;
    }

    // Apply velocities
    enemy.x_pos = (ex + i64::from(enemy.x_velocity)).clamp(0, i64::from(u32::MAX)) as u32;
    enemy.y_pos = (ey + i64::from(enemy.y_velocity)).clamp(0, i64::from(u32::MAX)) as u32;
}

pub(crate) fn step_multi_head_boss_movement(enemy: &mut ActiveEnemy) {
    let movement = boss_head_movement_step_for_family_sub_39846(
        enemy.family,
        enemy.boss_head_index,
        enemy.boss_movement_delay_counter,
        enemy.boss_movement_script_index,
    );
    enemy.boss_movement_delay_counter = movement.next_delay_counter;
    enemy.boss_movement_script_index = movement.next_script_index;
    if let Some(x_velocity) = movement.next_x_velocity {
        enemy.x_velocity = x_velocity;
    }
    if let Some(y_velocity) = movement.next_y_velocity {
        enemy.y_velocity = y_velocity;
    }

    // ASM sub_39846 loads the fixed script velocity; native applies it in the
    // same active-object step because these boss heads are not player trackers.
    let next_x = enemy.x_pos as i64 + i64::from(enemy.x_velocity);
    let next_y = enemy.y_pos as i64 + i64::from(enemy.y_velocity);
    enemy.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
    enemy.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
    let parent_animation =
        boss_head_parent_animation_state_loc_39538_39570(enemy.wait_timer, enemy.facing);
    enemy
        .boss_eye_orbit_workers
        .step_with_parent_animation_loc_3893e_389d6(
            (enemy.x_pos >> 16) as u16,
            (enemy.y_pos >> 16) as u16,
            enemy.facing,
            parent_animation.parent_phase_word_22,
            parent_animation.parent_x_direction_byte_16,
        );
}

/// Boss periodic projectile fire interval (frames).
pub(crate) const BOSS_FIRE_INTERVAL: u16 = 180; // ~3 seconds
pub(crate) const BOSS_HEAD_ATTACK_WINDUP_TIMER_MASK_LOC_39538: u16 = 0x8000;
/// ASM `stru_39FC6`: frame holds `$14 + $28 + $19 + 1` before `$18(a3)` is set.
pub(crate) const BOSS_HEAD_ATTACK_WINDUP_FRAMES_STRU_39FC6: u16 = 0x14 + 0x28 + 0x19 + 1;
pub(crate) const BOSS_HEAD_ATTACK_WINDUP_TIMER_LOC_39538: u16 =
    BOSS_HEAD_ATTACK_WINDUP_TIMER_MASK_LOC_39538 | BOSS_HEAD_ATTACK_WINDUP_FRAMES_STRU_39FC6;
#[cfg(test)]
pub(crate) const BOSS_HEAD_ATTACK_READY_TIMER_LOC_39566: u16 =
    BOSS_HEAD_ATTACK_WINDUP_TIMER_MASK_LOC_39538;
/// ASM sub_397E4 loc_39836 starts `stru_39FD8`; loc_39570/loc_37FF4 then
/// hibernate/check `$18(a3)` until its five frame holds complete.
pub(crate) const BOSS_HEAD_POST_PROJECTILE_TIMER_LOC_39570: u16 = 0x22 + 0x05 + 0x0A + 0x0A + 0x0A;
pub(crate) const BOSS_HEAD_SPRITE_C8600_LINK_OFFSET: u16 = 0x0EE0;
pub(crate) const BOSS_HEAD_SPRITE_C8608_LINK_OFFSET: u16 = 0x0EE2;
pub(crate) const BOSS_HEAD_SPRITE_C8610_LINK_OFFSET: u16 = 0x0EE4;
pub(crate) const BOSS_HEAD_SPRITE_C8618_LINK_OFFSET: u16 = 0x0EE6;
pub(crate) const BOSS_HEAD_SPRITE_C8620_LINK_OFFSET: u16 = 0x0EE8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossHeadParentAnimationStateLoc39538 {
    pub(crate) parent_phase_word_22: u16,
    pub(crate) parent_x_direction_byte_16: u8,
}

fn boss_head_x_direction_byte_16(facing: Direction) -> u8 {
    match facing {
        Direction::Left => 0x00,
        Direction::Right => 0xFF,
    }
}

pub(crate) fn boss_head_parent_animation_state_loc_39538_39570(
    wait_timer: u16,
    facing: Direction,
) -> BossHeadParentAnimationStateLoc39538 {
    let parent_phase_word_22 = if wait_timer & BOSS_HEAD_ATTACK_WINDUP_TIMER_MASK_LOC_39538 != 0 {
        let remaining = wait_timer & !BOSS_HEAD_ATTACK_WINDUP_TIMER_MASK_LOC_39538;
        let elapsed = BOSS_HEAD_ATTACK_WINDUP_FRAMES_STRU_39FC6.saturating_sub(remaining);
        // ASM stru_39FC6: C8608 for `$14`, C8610 for `$28`, C8618 for `$19`,
        // then C8620 for one frame before `$18(a3)` ends the windup.
        match elapsed {
            0..=19 => BOSS_HEAD_SPRITE_C8608_LINK_OFFSET,
            20..=59 => BOSS_HEAD_SPRITE_C8610_LINK_OFFSET,
            60..=84 => BOSS_HEAD_SPRITE_C8618_LINK_OFFSET,
            _ => BOSS_HEAD_SPRITE_C8620_LINK_OFFSET,
        }
    } else if wait_timer > 0 && wait_timer <= BOSS_HEAD_POST_PROJECTILE_TIMER_LOC_39570 {
        let elapsed = BOSS_HEAD_POST_PROJECTILE_TIMER_LOC_39570.saturating_sub(wait_timer);
        // ASM stru_39FD8: C8620 for `$22`, C8618 for `5`, then C8610/C8608/C8600
        // for `$A` each while loc_39570 waits for `$18(a3)`.
        match elapsed {
            0..=33 => BOSS_HEAD_SPRITE_C8620_LINK_OFFSET,
            34..=38 => BOSS_HEAD_SPRITE_C8618_LINK_OFFSET,
            39..=48 => BOSS_HEAD_SPRITE_C8610_LINK_OFFSET,
            49..=58 => BOSS_HEAD_SPRITE_C8608_LINK_OFFSET,
            _ => BOSS_HEAD_SPRITE_C8600_LINK_OFFSET,
        }
    } else {
        BOSS_HEAD_SPRITE_C8600_LINK_OFFSET
    };

    BossHeadParentAnimationStateLoc39538 {
        parent_phase_word_22,
        parent_x_direction_byte_16: boss_head_x_direction_byte_16(facing),
    }
}
/// ASM Enemy20_HeadyMetal_Init (kid.asm:39264): the `$FFFFFB7A` anchor starts
/// its `$3E(a3)` counter at `$A` before `loc_375DE` starts eye spawns.
pub(crate) const HEADY_METAL_EYE_SPAWN_COUNTER_INIT_LOC_39264: u16 = 0x000A;
/// ASM `sub_370D2` starts `stru_37BD8` and loc_3712E falls until that script
/// sets animation flag `$18(a3)`.
pub(crate) const BOSS_DEATH_BURST_ENTRY_ANIMATION_FRAMES_STRU_37BD8: u16 =
    0x05 + 0x05 + 0x0A + 0x01;
/// ASM `stru_37B98`: high-count eye-spawn animation duration before `$18(a3)`.
pub(crate) const HEADY_METAL_EYE_HIGH_ANIMATION_FRAMES_STRU_37B98: u16 =
    0x2D + 0x32 + 0x05 + 0x14 + 0x14 + 0x19 + 0x05 + 0x0F;
/// ASM `stru_37BC0` then `stru_37BC6`: low-count rapid eye-spawn animation
/// duration before `$18(a3)` is set by the zero terminator.
pub(crate) const HEADY_METAL_EYE_LOW_ANIMATION_FRAMES_STRU_37BC0: u16 =
    0x0A + 0x0A + 0x0A + 0x0A + 0x0A;

fn heady_metal_eye_spawn_animation_frames(selected_animation_label: &str) -> u16 {
    match selected_animation_label {
        "stru_37B98" => HEADY_METAL_EYE_HIGH_ANIMATION_FRAMES_STRU_37B98,
        "stru_37BC0" => HEADY_METAL_EYE_LOW_ANIMATION_FRAMES_STRU_37BC0,
        _ => 0,
    }
}

/// Bosses periodically fire projectiles at the player.
#[cfg(test)]
pub(crate) fn step_boss_fire(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    step_boss_fire_with_sounds(enemy, projectiles, player_x_pos, player_y_pos, None);
}

pub(crate) fn step_boss_fire_with_sounds(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    played_sound_effects: Option<&mut Vec<QueuedSoundEffect>>,
) {
    if matches!(enemy.family, EnemyFamily::HeadyMetal)
        && 37u16.saturating_sub(u16::from(enemy.hit_points)) >= 0x0025
    {
        // ASM sub_37708 loc_377BC sets `$FFFFFB4E` once the breakup phase
        // reaches `$25`; loc_375DE checks that latch before allocating normal
        // `sub_370D2` eye fire.
        return;
    }

    let multi_head_boss = matches!(
        enemy.family,
        EnemyFamily::Shishkaboss | EnemyFamily::BoomerangBosses | EnemyFamily::BagelBrothers
    );
    let attack_windup_remaining =
        multi_head_boss && enemy.wait_timer & BOSS_HEAD_ATTACK_WINDUP_TIMER_MASK_LOC_39538 != 0;
    if attack_windup_remaining {
        let remaining_frames = enemy.wait_timer & !BOSS_HEAD_ATTACK_WINDUP_TIMER_MASK_LOC_39538;
        if remaining_frames > 0 {
            enemy.wait_timer =
                BOSS_HEAD_ATTACK_WINDUP_TIMER_MASK_LOC_39538 | remaining_frames.saturating_sub(1);
            return;
        }
    } else if enemy.wait_timer > 0 {
        enemy.wait_timer -= 1;
        return;
    }
    if multi_head_boss {
        let Some(player_y_pos) = player_y_pos else {
            return;
        };
        let head_y = ((enemy.y_pos >> 16) as u16) as i16;
        let kid_y = ((player_y_pos >> 16) as u16) as i16;
        let lower_y = head_y.wrapping_sub(0x0040_i16);
        if kid_y < lower_y || kid_y > head_y {
            // ASM loc_39506/loc_37F94: outside `head_y-$40..=head_y`,
            // branch back to the wait loop without starting the attack timer.
            return;
        }
        if !attack_windup_remaining {
            let head_x = ((enemy.x_pos >> 16) as u16) as i16;
            let kid_x = ((player_x_pos.unwrap_or(enemy.x_pos) >> 16) as u16) as i16;
            enemy.facing = if head_x > kid_x {
                Direction::Left
            } else {
                Direction::Right
            };
            // ASM loc_39534/loc_37FC2 sets `x_direction(a3)`, then
            // loc_39538/loc_37FC6 caches it to `$FFFFFB86/87/88` and starts
            // `stru_39FC6`; native carries that cached byte as `enemy.facing`.
            enemy.wait_timer = BOSS_HEAD_ATTACK_WINDUP_TIMER_LOC_39538;
            return;
        }
    }
    let interval = if matches!(enemy.family, EnemyFamily::HeadyMetal) {
        let piece_counter_before_spawn = if enemy.boss_movement_delay_counter == 0 {
            HEADY_METAL_EYE_SPAWN_COUNTER_INIT_LOC_39264
        } else {
            enemy.boss_movement_delay_counter
        };
        // ASM loc_375DE/loc_3761E: `$18(a3)` gates allocation. After spawning,
        // decrement `$3E(a3)`, reload `$A` only for the stored counter when the
        // decremented value is zero, then initialise `stru_37B98`/`stru_37BC0`.
        let spawn_animation = heady_metal_spawn_animation_loc_3761E(piece_counter_before_spawn);
        enemy.boss_movement_delay_counter = spawn_animation.next_piece_counter;
        heady_metal_eye_spawn_animation_frames(spawn_animation.selected_animation_label)
    } else if multi_head_boss {
        BOSS_HEAD_POST_PROJECTILE_TIMER_LOC_39570
    } else {
        BOSS_FIRE_INTERVAL
    };
    enemy.wait_timer = interval;

    if projectiles.len() >= MAX_PROJECTILES {
        return;
    }

    match enemy.family {
        EnemyFamily::HeadyMetal => {
            // ASM loc_375DE allocates `sub_370D2` for normal fire; `sub_37332`
            // is the paired eye-launch worker used by the breakup routine
            // `sub_37708 loc_37778`.
            let init = boss_death_burst_init_sub_370D2(
                (enemy.x_pos >> 16) as u16,
                (enemy.y_pos >> 16) as u16,
            );
            if let Some(sound_effects) = played_sound_effects {
                sound_effects.push(QueuedSoundEffect::VoiceDie);
            }
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                x_velocity: 0,
                y_velocity: 0,
                lifetime: init.chase_frames,
                gravity: init.landing_gravity,
                friendly: false,
                homing_accel: init.homing_accel,
                animation_elapsed_frames: BOSS_DEATH_BURST_ENTRY_ANIMATION_FRAMES_STRU_37BD8,
                boss_death_burst_state: BossDeathBurstState::FallingLoc3712E,
                speed_cap: 0,
                wall_bounce: false,
                kind: EnemyProjectileKind::BossDeathBurst,
                boss_eye_sprite_mapping_label: init.sprite_mapping_label,
                ..Default::default()
            });
        }
        EnemyFamily::Shishkaboss | EnemyFamily::BoomerangBosses | EnemyFamily::BagelBrothers => {
            let parent_x = (enemy.x_pos >> 16) as u16;
            let parent_y = (enemy.y_pos >> 16) as u16;
            let parent_x_direction = match enemy.facing {
                Direction::Left => 0,
                Direction::Right => 0xFF,
            };
            let variant_d1 = if matches!(enemy.family, EnemyFamily::BoomerangBosses) {
                1
            } else {
                0
            };
            let spawn = boss_head_projectile_spawn_sub_397E4(
                parent_x,
                parent_y,
                parent_x_direction,
                variant_d1,
            );
            // ASM sub_397E4/loc_3980A copies the cached parent x_direction
            // into the child and stores `$48=±3`. Native child coroutines now
            // use loc_3A0D2 for arrow heads and loc_3A1EA/loc_3A24C for
            // boomerangs, seeded from this helper.
            let is_boomerang = variant_d1 == 1;
            let direction_byte_d0 = ((spawn.velocity_word_48 as u16) >> 8) as u8;
            projectiles.push(EnemyProjectile {
                x_pos: u32::from(spawn.x_pos) << 16,
                y_pos: u32::from(spawn.y_pos) << 16,
                x_velocity: i32::from(spawn.velocity_word_48) << 16,
                y_velocity: 0,
                lifetime: if is_boomerang { 0x0190 } else { 50 },
                animation_elapsed_frames: 0,
                boss_head_boomerang_direction_byte_d0: direction_byte_d0,
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                kind: if is_boomerang {
                    EnemyProjectileKind::BossHeadBoomerang
                } else {
                    EnemyProjectileKind::BossHeadArrow
                },
                ..Default::default()
            });
        }
        _ => {}
    }
}

pub(crate) fn spawn_x_pos(_family: EnemyFamily, object_x: u32) -> u32 {
    object_x << 16
}

#[allow(dead_code)]
pub(crate) fn spawn_y_pos(family: EnemyFamily, object_y: u32) -> u32 {
    spawn_y_pos_sub_36FF4(family, object_y, 0)
}

#[allow(non_snake_case)]
pub(crate) fn enemy_spawn_y_adjustment_sub_36FF4(blank_top_rows: u16) -> u32 {
    // ASM sub_36FF4 (kid.asm:38882): load `Number_blank_top_rows`, shift left
    // by 4, and add that pixel offset to `y_pos(a3)`.
    u32::from(blank_top_rows) << 4
}

#[allow(non_snake_case)]
pub(crate) fn spawn_y_pos_sub_36FF4(
    family: EnemyFamily,
    object_y: u32,
    blank_top_rows: u16,
) -> u32 {
    let object_y = object_y.saturating_add(enemy_spawn_y_adjustment_sub_36FF4(blank_top_rows));
    match family {
        EnemyFamily::DragonWalking => {
            object_y.saturating_add(DRAGON_WALKING_SPAWN_Y_OFFSET_PX) << 16
        }
        EnemyFamily::Hand => {
            let y_word =
                u16::try_from(object_y & 0xFFFF).expect("masked Hand spawn y fits in a word");
            u32::from(hand_init_y_snap_loc_32aea(y_word)) << 16
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
        // Big_Hopping_Skull.asm loc_3651E: move.w #1,x_vel -> $10000 FP16.16
        EnemyFamily::BigHoppingSkull => BIG_SKULL_INIT_X_SPEED,
        // ASM Hand.asm loc_327F2 stores the direction-gated table velocity
        // into x_vel before the first patrol hibernate.
        EnemyFamily::Hand => {
            let hit_points = spawn_hit_points(EnemyFamily::Hand, enemy_level);
            hand_initial_motion_loc_327f2(hit_points, false).x_velocity
        }
        _ => 0,
    }
}

pub(crate) fn spawn_facing(family: EnemyFamily, enemy_x: u32, player_x: u32) -> Direction {
    match family {
        // ASM Hand.asm loc_327B6 clears x_direction; loc_3284C's clear-flag
        // branch detects Kid only on the left side.
        EnemyFamily::Hand => {
            let init = hand_init_loc_327b6();
            debug_assert!(init.status_flag_13);
            debug_assert!(init.is_moved);
            debug_assert!(!init.motion_flag_19);
            debug_assert!(init.has_level_collision);
            debug_assert_eq!(init.priority, 0);
            debug_assert_eq!(init.object_meta, 0x0017);
            if init.x_direction_flag {
                Direction::Right
            } else {
                Direction::Left
            }
        }
        // Drips always start facing right (Drip.asm init)
        EnemyFamily::Drips => Direction::Right,
        // ASM Armadillo.asm loc_3C718, Goat.asm loc_3CA30, Dragon.asm loc_3CD70,
        // Tank.asm loc_3B2E4, Robot.asm loc_3A7AC, Driller.asm loc_3BD2A,
        // and Scorpion.asm loc_3DBD8:
        // face TOWARD the player at spawn.
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

// Shared enemy test fixtures, lifted out of `mod tests` so each per-family
// sibling module's own test mod can reach them (issue #40). Imported via
// `use crate::enemies::test_fixtures::*;`. Mirrors the crate's tests/fixtures.rs.
#[cfg(test)]
pub(crate) mod test_fixtures {
    use super::*;

    pub(crate) fn mini_hopping_skull_test_enemy() -> ActiveEnemy {
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
            death_animation_active: false,
            grounded_frames: 0,
            tank_mouth_animation_phase: 0,
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
            drips_attached: false,
            drips_surface_table_index: 0,
            drips_pending_splash_x_pos: 0,
            drips_pending_splash_y_pos: 0,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: BOSS_HEAD_NONE,
            boss_eye_orbit_workers: BossEyeOrbitWorkers::default(),
            boss_movement_delay_counter: 0,
            boss_movement_script_index: 0,
            heady_metal_palette_phase_counter: 0,
            heady_metal_palette_word_5e: 0x0200,
            scorpion_tracking_lock: false,
            scorpion_sting_frames: 0,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            crab_berzerker_knockback: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
            shared_walker_turnaround_counter: 0,
            shared_walker_right_turn_extra_step_consumed: false,
            shared_walker_patrol_short_next: false,
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        }
    }

    pub(crate) fn mini_hopping_skull_test_grid() -> terrain::BlockGrid {
        let data = vec![0u8; 8 * 4 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        for col in 0..8 {
            grid.set_cell_at(col, 2, terrain::Cell::from_be_bytes(0x60, 0x00));
        }
        grid
    }

    pub(crate) fn tar_monster_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::TarMonster,
            hit_points: 2,
            death_animation_active: false,
            facing: Direction::Right,
            tar_monster_state: TarMonsterState::Attacking,
            wait_timer: TAR_MONSTER_ATTACK_FRAMES,
            ..mini_hopping_skull_test_enemy()
        }
    }

    pub(crate) fn fireball_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Fireball,
            hit_points: 1,
            death_animation_active: false,
            facing: Direction::Right,
            ..mini_hopping_skull_test_enemy()
        }
    }

    pub(crate) fn sphere_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Sphere,
            hit_points: 1,
            death_animation_active: false,
            facing: Direction::Right,
            x_velocity: 0x0001_0000,
            ..mini_hopping_skull_test_enemy()
        }
    }

    pub(crate) fn ninja_test_enemy() -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Ninja,
            hit_points: 2,
            death_animation_active: false,
            facing: Direction::Left,
            ..mini_hopping_skull_test_enemy()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    #[test]
    fn active_enemy_step_respects_samurai_haze_object_gate_loc_e1e() {
        let mut enemy = fireball_test_enemy();
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0;
        let mut enemies = [enemy];
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();

        step_active_enemies_with_rng(
            &mut enemies,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            1,
            true,
            None,
            &mut rng,
            &mut drilled_blocks,
            None,
        );

        assert_eq!(
            enemies[0].x_pos,
            32 << 16,
            "ASM loc_E1E skips `$10(a5)` enemy objects during Samurai Haze when `Time_Frames & 3 != 0`, before their per-frame movement can run",
        );

        step_active_enemies_with_rng(
            &mut enemies,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            4,
            true,
            None,
            &mut rng,
            &mut drilled_blocks,
            None,
        );

        assert_eq!(
            enemies[0].x_pos,
            (32 << 16) + 0x0001_0000,
            "ASM loc_E1E lets haze-flagged enemy objects execute normally when `Time_Frames & 3 == 0`",
        );
    }

    #[test]
    fn boss_head_config_matches_multi_head_init_asm() {
        assert_eq!(
            boss_head_config(EnemyFamily::Shishkaboss),
            ([20, 20, 20], [0, 0, 0], [0, 0x48, 0x90],),
            "ASM kid.asm loc_394E2/loc_39608/loc_3972E stores ShishkaBoss_HitPointsPerHead `$14` for all three heads; the `$1E` preload on heads 2/3 is overwritten",
        );
        assert_eq!(
            boss_head_config(EnemyFamily::BoomerangBosses),
            ([30, 30, 30], [0, -0x00C8, -0x01C2], [0, 0, 0],),
            "ASM kid.asm Enemy23_BoomerangBosses_Init spawns heads at x offsets 0/-$C8/-$1C2 and uses BoomerangBoss_HitPointsPerHead `$1E`",
        );
        assert_eq!(
            boss_head_config(EnemyFamily::BagelBrothers),
            ([30, 30, 30], [0, -0x0064, 0], [0, -0x0096, -0x012C],),
            "ASM kid.asm Enemy24_BagelBrothers_Init spawns heads at (0,0), (-$64,-$96), and (0,-$12C), each with BagelBrothers_HitPointsPerHead `$1E`",
        );
        assert_eq!(
            boss_head_config(EnemyFamily::Tank),
            ([1, 1, 1], [0, 0, 0], [0, 0, 0]),
            "Non-boss families have no multi-head ASM spawn table and keep the sentinel fallback",
        );
    }

    #[test]
    fn pacer_walk_speed_covers_family_specific_asm_tiers() {
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Goat, 3),
            0x0000_8000,
            "ASM Goat.asm loc_3CA7E gives base HP one final `$8000` add",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Goat, 4),
            0x0001_8000,
            "ASM Goat.asm loc_3CA6E lets HP 4 reach the `$10000` tier before the final add",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Scorpion, 5),
            0x0001_8000,
            "ASM Scorpion init uses a fixed `$18000` shared-walker speed",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::FireDemon, 3),
            0x0000_8000,
            "ASM Fire_Demon.asm loc_3A414 gives HP 3 only the final `$8000` add",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::FireDemon, 5),
            0x0001_8000,
            "ASM Fire_Demon.asm loc_3A40C/loc_3A414 gives HP 5 the three-add speed tier",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Robot, 2),
            0x0000_8000,
            "ASM Robot init starts the HP 2 walker at one `$8000` add, not the default `$10000` fallback",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Lion, 8),
            0x0002_8000,
            "ASM Lion.asm loc_3DFD8 skips the `$18000` branch only when current_hp is exactly 8",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Hand, 3),
            0x0000_6000,
            "ASM Hand speed table advances from `$3000` by `$1800` per HP tier",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::SpinningTwins, 1),
            0x0001_8000,
            "ASM Spinning_Twins shared-walker landing path uses fixed `$18000` walk speed",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::TarMonster, 2),
            0x0001_8000,
            "ASM Tar_Monster emerging walker uses fixed `$18000` walk speed",
        );
    }

    #[test]
    fn shared_walker_turnaround_counter_matches_loc_3c026() {
        assert_eq!(
            shared_walker_turnaround_loc_3c026(0xF8),
            SharedWalkerTurnaroundLoc3C026 {
                next_counter: 0xFC,
                collision_type: None,
            },
            "ASM loc_3C026 should add 4 to the shared walker stuck counter"
        );
        assert_eq!(
            shared_walker_turnaround_loc_3c026(0xFC),
            SharedWalkerTurnaroundLoc3C026 {
                next_counter: 0,
                collision_type: Some(-1),
            },
            "ASM loc_3C464 should write collision_type=$FFFF when $5A overflows"
        );
    }

    #[test]
    fn left_shared_walker_clear_floor_enters_loc_3c266_falling_loop() {
        let grid = terrain::BlockGrid::from_level_layout(&vec![0u8; 16 * 16 * 2], 16, 16);
        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.family = EnemyFamily::Tank;
        enemy.facing = Direction::Left;
        enemy.hit_points = 2;
        enemy.x_pos = 0x0288_0000;
        enemy.y_pos = 0x008F_0000;
        enemy.on_ground = true;

        step_pacer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_pos, 0x0287_8000,
            "ASM loc_3C266 -> loc_3C288 should keep the leftward Tank moving while entering the falling loop"
        );
        assert_eq!(
            enemy.y_pos, 0x0090_0000,
            "ASM loc_3C266 addq.w #1,y_pos(a3) exposes the one-pixel drop on the first unsupported frame"
        );
        assert_eq!(
            enemy.y_velocity, 0x0000_2000,
            "ASM loc_3C332 adds $2000 gravity after storing the unsupported Y position"
        );
        assert!(
            !enemy.on_ground,
            "ASM loc_3C266 should enter the falling walker loop rather than pinning the Tank as grounded"
        );

        step_pacer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_pos, 0x0287_0000,
            "ASM loc_3C288 subtracts the stored leftward x_vel on the next falling frame"
        );
        assert_eq!(
            enemy.y_pos, 0x0090_2000,
            "ASM loc_3C2D0 -> loc_3C32E should apply the previous $2000 y_vel before increasing gravity"
        );
        assert_eq!(
            enemy.y_velocity, 0x0000_4000,
            "ASM loc_3C332 should accumulate another $2000 of falling velocity"
        );
    }

    #[test]
    fn dragon_art_and_hp_seed_match_loc_3cdb8() {
        assert_eq!(
            dragon_art_init_loc_3cdb8(0x400D, 0xFFFF, 0),
            DragonArtInitLoc3CDB8 {
                art_slot_index: 0,
                status_flag_13: true,
                priority: 0,
                current_hp: 1,
            },
            "ASM Dragon.asm loc_3CDB8 uses d4=0 when header slot 0 masks to DragonWalking `$0D`"
        );
        assert_eq!(
            dragon_art_init_loc_3cdb8(0xFFFF, 0x800C, 2),
            DragonArtInitLoc3CDB8 {
                art_slot_index: 1,
                status_flag_13: true,
                priority: 0,
                current_hp: 3,
            },
            "ASM Dragon.asm loc_3CDB8 uses d4=1 when header slot 1 masks to Flying Dragon `$0C` and stores enemy_level+1 HP"
        );
        assert_eq!(
            dragon_art_init_loc_3cdb8(0x0001, 0x0002, 1).art_slot_index,
            2,
            "ASM Dragon.asm loc_3CDB8 falls through with d4=2 when neither loaded header slot is `$0C` or `$0D`"
        );
        assert_eq!(
            spawn_hit_points(EnemyFamily::DragonWalking, 2),
            3,
            "native DragonWalking HP should expose the same enemy_level+1 seed as ASM Dragon.asm loc_3CDB8"
        );
    }

    #[test]
    fn dragon_walking_spawn_uses_loc_3cdb8_hp_seed() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![0xFFFF, 0x800D, 0xFFFF],
            unknown_word: 0x7D00,
            header_byte: 0x80,
            object_count: 1,
            objects: vec![wildside_data::EnemyLayoutObject {
                object_type: 0x81,
                flags: 0xFF,
                hit_points_plus_one: 2,
                x: 200,
                y: 144,
            }],
            segments: vec![wildside_data::EnemyLayoutSegment {
                lower_bound: 0,
                upper_bound: 0x7D00,
                enemy_type_slots: vec![0xFFFF, 0x800D, 0xFFFF],
                object_batch_flags: 0x80,
                object_count: 1,
                objects: vec![wildside_data::EnemyLayoutObject {
                    object_type: 0x81,
                    flags: 0xFF,
                    hit_points_plus_one: 2,
                    x: 200,
                    y: 144,
                }],
            }],
        };
        let window = ActivationWindow {
            x_min: 0,
            x_max: 640,
            y_min: 0,
            y_max: 480,
        };
        let mut active = Vec::new();

        activate_enemies(
            &mut active,
            &layout,
            window,
            0,
            0,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            0,
            0,
            &std::collections::HashSet::new(),
        );

        let dragon = active.first().expect("walking dragon should activate");
        assert_eq!(dragon.family, EnemyFamily::DragonWalking);
        assert_eq!(
            dragon.hit_points, 3,
            "ASM Dragon.asm loc_3CDB8 stores enemy_level+1 in current_hp(a3), so raw level 2 spawns a 3 HP walking dragon"
        );
    }

    #[test]
    fn ninja_spawn_uses_loc_3d554_hp_seed() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![0x0011, 0xFFFF, 0xFFFF],
            unknown_word: 0x7D00,
            header_byte: 0x80,
            object_count: 1,
            objects: vec![wildside_data::EnemyLayoutObject {
                object_type: 0x11,
                flags: 0xFF,
                hit_points_plus_one: 1,
                x: 200,
                y: 144,
            }],
            segments: vec![wildside_data::EnemyLayoutSegment {
                lower_bound: 0,
                upper_bound: 0x7D00,
                enemy_type_slots: vec![0x0011, 0xFFFF, 0xFFFF],
                object_batch_flags: 0x80,
                object_count: 1,
                objects: vec![wildside_data::EnemyLayoutObject {
                    object_type: 0x11,
                    flags: 0xFF,
                    hit_points_plus_one: 1,
                    x: 200,
                    y: 144,
                }],
            }],
        };
        let window = ActivationWindow {
            x_min: 0,
            x_max: 640,
            y_min: 0,
            y_max: 480,
        };
        let mut active = Vec::new();

        activate_enemies(
            &mut active,
            &layout,
            window,
            0,
            0,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            0,
            0,
            &std::collections::HashSet::new(),
        );

        let ninja = active.first().expect("ninja should activate");
        assert_eq!(ninja.family, EnemyFamily::Ninja);
        assert_eq!(
            ninja.hit_points, 3,
            "ASM Ninja.asm loc_3D554 adds 2 to enemy_level before storing current_hp(a3)"
        );
        assert_eq!(
            pacer_walk_speed(ninja.family, ninja.hit_points),
            0x0001_8000,
            "ASM Ninja.asm loc_3D59A/loc_3D5AA keep HP 3 on the fallthrough speed tier"
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
    fn emo_rock_clearance_window_loc_3639e_builds_inclusive_scan_counts() {
        assert_eq!(
            emo_rock_clearance_window_loc_3639e(0x0200, 0x0040, 0x0050, 0x001C, 0x000E),
            EmoRockClearanceWindowRouteLoc3639E::Scan(EmoRockClearanceWindowLoc3639E {
                top_row: 0x0003,
                bottom_row: 0x0005,
                left_col: 0x0003,
                right_col: 0x0004,
                row_dbf_count: 0x0002,
                col_dbf_count: 0x0001,
            }),
            "ASM Emo_Rock.asm loc_3639E shifts `(y_pos-d6)..y_pos` and `(x_pos-d7)..(x_pos+d7)` by four, then subtracts starts from ends to seed inclusive DBF counts"
        );
    }

    #[test]
    fn emo_rock_clearance_window_loc_3639e_treats_edges_as_blocked() {
        assert_eq!(
            emo_rock_clearance_window_loc_3639e(0x0200, 0x0040, 0x001B, 0x001C, 0x000E),
            EmoRockClearanceWindowRouteLoc3639E::Blocked,
            "ASM Emo_Rock.asm loc_3639E branches to loc_363FA when `y_pos-d6` is negative"
        );
        assert_eq!(
            emo_rock_clearance_window_loc_3639e(0x0200, 0x000D, 0x0050, 0x001C, 0x000E),
            EmoRockClearanceWindowRouteLoc3639E::Blocked,
            "ASM Emo_Rock.asm loc_3639E branches to loc_363FA when `x_pos-d7` is negative"
        );
        assert_eq!(
            emo_rock_clearance_window_loc_3639e(0x0200, 0x01F2, 0x0050, 0x001C, 0x000E),
            EmoRockClearanceWindowRouteLoc3639E::Blocked,
            "ASM Emo_Rock.asm loc_3639E branches to loc_363FA when `x_pos+d7` is at or beyond Level_width_pixels"
        );
    }

    #[test]
    fn emo_rock_clearance_rows_loc_363dc_resets_the_inner_scan_each_row() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 6 * 6 * 2], 6, 6);
        grid.set_cell_at(4, 4, terrain::Cell::from_be_bytes(0x60, 0x00));

        let window = EmoRockClearanceWindowLoc3639E {
            top_row: 3,
            bottom_row: 4,
            left_col: 3,
            right_col: 4,
            row_dbf_count: 1,
            col_dbf_count: 1,
        };

        assert!(
            !emo_rock_clearance_rows_loc_363dc(&grid, window),
            "ASM Emo_Rock.asm loc_363DC copies a0 back into a1 for the second row, so the scan must still inspect the rightmost column there"
        );
    }

    #[test]
    fn emo_rock_clearance_cell_loc_363e0_checks_only_the_4000_bit() {
        assert!(
            !emo_rock_clearance_cell_blocked_loc_363e0(terrain::Cell::from_be_bytes(0x30, 0x00)),
            "ASM Emo_Rock.asm loc_363E0 should keep scanning when the masked `$4000` bit is clear"
        );
        assert!(
            emo_rock_clearance_cell_blocked_loc_363e0(terrain::Cell::from_be_bytes(0x40, 0x00)),
            "ASM Emo_Rock.asm loc_363E0 should branch blocked for a slope-up `$4000` word"
        );
        assert!(
            emo_rock_clearance_cell_blocked_loc_363e0(terrain::Cell::from_be_bytes(0x60, 0x00)),
            "ASM Emo_Rock.asm loc_363E0 should branch blocked for a solid `$6000` word because bit `$4000` remains set"
        );
    }

    #[test]
    fn emo_rock_clearance_probe_loc_363e0_blocks_drop_when_any_scanned_cell_is_solid() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 8 * 8 * 2], 8, 8);
        assert!(
            emo_rock_clearance_probe_loc_3639e(&grid, 0x0040, 0x0050, 0x001C, 0x000E),
            "ASM Emo_Rock.asm loc_363E0 should fall through with d7=0 when every scanned cell clears bit `$4000`"
        );

        grid.set_cell_at(4, 4, terrain::Cell::from_be_bytes(0x60, 0x00));
        assert!(
            !emo_rock_clearance_probe_loc_3639e(&grid, 0x0040, 0x0050, 0x001C, 0x000E),
            "ASM Emo_Rock.asm loc_363E0 should return blocked as soon as any cell in the loc_363DC row loop has bit `$4000` set"
        );
    }

    #[test]
    fn tornado_pattern_zero_x_acceleration_matches_loc_35452() {
        assert_eq!(
            acceleration_for_target(0x0000_1000, 0, 0),
            -TORNADO_ACCELERATION,
            "ASM UFO.asm loc_35452 substitutes `$C00` for a zero X acceleration entry and negates it when x_vel is non-negative"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_1000, 0, 0),
            TORNADO_ACCELERATION,
            "ASM UFO.asm loc_35452 keeps the substituted `$C00` positive when x_vel is negative"
        );
    }

    #[test]
    fn tornado_pattern_zero_y_acceleration_matches_loc_35496() {
        assert_eq!(
            acceleration_for_target(0x0000_1000, 0, 0),
            -TORNADO_ACCELERATION,
            "ASM UFO.asm loc_35496 substitutes `$C00` for a zero Y acceleration entry and negates it when y_vel is non-negative"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_1000, 0, 0),
            TORNADO_ACCELERATION,
            "ASM UFO.asm loc_35496 keeps the substituted `$C00` positive when y_vel is negative"
        );
    }

    #[test]
    fn tornado_pattern_non_negative_x_velocity_matches_loc_3546e() {
        assert_eq!(
            acceleration_for_target(0x0000_3000, -0x0001_0000, -0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_3546E keeps d0 when x_vel is non-negative and target d1 is negative"
        );
        assert_eq!(
            acceleration_for_target(0x0000_3000, 0x0000_4000, 0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_3546E keeps d0 when positive target d1 is not below current x_vel"
        );
        assert_eq!(
            acceleration_for_target(0x0000_3000, 0x0000_2000, 0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_3546E negates d0 when positive target d1 is below current x_vel"
        );
    }

    #[test]
    fn tornado_pattern_non_negative_y_velocity_matches_loc_354b0() {
        assert_eq!(
            acceleration_for_target(0x0000_3000, -0x0001_0000, -0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_354B0 keeps d2 when y_vel is non-negative and target d3 is negative"
        );
        assert_eq!(
            acceleration_for_target(0x0000_3000, 0x0000_4000, 0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_354B0 keeps d2 when positive target d3 is not below current y_vel"
        );
        assert_eq!(
            acceleration_for_target(0x0000_3000, 0x0000_2000, 0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_354B0 negates d2 when positive target d3 is below current y_vel"
        );
    }

    #[test]
    fn tornado_pattern_negative_x_velocity_matches_loc_35488() {
        assert_eq!(
            acceleration_for_target(-0x0000_3000, 0x0001_0000, 0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_35488 keeps d0 when x_vel is negative and target d1 is non-negative"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_3000, -0x0000_4000, -0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_35488 keeps d0 when negative target d1 is not above current x_vel"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_3000, -0x0000_2000, -0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_35488 negates d0 when negative target d1 is above current x_vel"
        );
    }

    #[test]
    fn tornado_pattern_negative_y_velocity_matches_loc_354ca() {
        assert_eq!(
            acceleration_for_target(-0x0000_3000, 0x0001_0000, 0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_354CA keeps d2 when y_vel is negative and target d3 is non-negative"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_3000, -0x0000_4000, -0x0000_0800),
            -0x0000_0800,
            "ASM UFO.asm loc_354CA keeps d2 when negative target d3 is not above current y_vel"
        );
        assert_eq!(
            acceleration_for_target(-0x0000_3000, -0x0000_2000, -0x0000_0800),
            0x0000_0800,
            "ASM UFO.asm loc_354CA negates d2 when negative target d3 is above current y_vel"
        );
    }

    #[test]
    fn drips_simultaneous_attach_seeds_consume_rng_in_descending_layout_order_loc_3340e() {
        // ASM Allocate_ObjectSlot (kid.asm loc_DE0): `cmp.w $E(a1),d6;
        // bhi.s loc_DE0` inserts an equal-priority object BEFORE existing
        // slots, so Execute_Objects (loc_E1E) runs simultaneously loaded
        // enemies in reverse load order. Two Drips attaching on the same
        // frame must therefore take their loc_3340E RNG draws in descending
        // layout order: the issue-260 oracle pair shows layout #08 (x=584)
        // holding the first draw and #07 (x=456) the second.
        let kid_x_pos = 0x0158_0000u32;
        let mut expected_rng = RomRandomState::default();
        let first_draw = drips_wait_timer_with_rng_loc_3340e(&mut expected_rng, kid_x_pos);
        let second_draw = drips_wait_timer_with_rng_loc_3340e(&mut expected_rng, kid_x_pos);

        let mut lower = mini_hopping_skull_test_enemy();
        lower.family = EnemyFamily::Drips;
        lower.layout_index = 7;
        lower.x_pos = 456 << 16;
        lower.drips_attached = true;
        lower.wait_timer = DRIPS_INITIAL_SEED_PENDING_TIMER_LOC_3340E;
        let mut upper = mini_hopping_skull_test_enemy();
        upper.family = EnemyFamily::Drips;
        upper.layout_index = 8;
        upper.x_pos = 584 << 16;
        upper.drips_attached = true;
        upper.wait_timer = DRIPS_INITIAL_SEED_PENDING_TIMER_LOC_3340E;

        // Native stores active enemies in ascending layout order.
        let mut enemies = [lower, upper];
        let mut rng = RomRandomState::default();
        seed_pending_drips_initial_waits_loc_3340e(&mut enemies, Some(kid_x_pos), &mut rng);

        assert_eq!(
            enemies[1].wait_timer, first_draw,
            "layout #08 executes first (Allocate_ObjectSlot LIFO), so it consumes the first loc_3340E RNG byte"
        );
        assert_eq!(
            enemies[0].wait_timer, second_draw,
            "layout #07 executes second, so it consumes the second loc_3340E RNG byte"
        );
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
    fn mini_hopping_skull_air_damage_slope_landing_resumes_terrain_contact_loc_3f22a_3ead8() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 8 * 4 * 2], 8, 4);
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0x40, 0x00));
        let mut enemy = ActiveEnemy {
            on_ground: false,
            facing: Direction::Right,
            x_pos: 41 << 16,
            y_pos: 37 << 16,
            x_velocity: 0,
            target_x_velocity: -0x0001_8000,
            y_velocity: 0x0002_8800,
            wait_timer: MINI_HOPPING_SKULL_AIR_DAMAGE_WAIT_TIMER_LOC_3F200,
            hit_points: 1,
            ..mini_hopping_skull_test_enemy()
        };

        step_mini_hopping_skull_air_damage_loop(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM Mini_Hopping_Skull.asm loc_3F22A restores saved x_vel, then loc_3EAD8 routes slope-up non-positive X to loc_3EC32"
        );
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);
        assert_eq!(enemy.grounded_frames, MINI_SKULL_GROUND_DELAY_FRAMES);
        assert_eq!(enemy.wait_timer, 0);
        assert!(enemy.on_ground);
    }

    #[test]
    fn mini_hopping_skull_slopeup_scan_matches_loc_3f386() {
        let data = vec![0u8; 8 * 4 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0x40, 0x00));

        assert_eq!(
            mini_hopping_skull_snap_slopeup_x(&grid, 16, 46),
            34u32 << 16,
            "ASM Mini_Hopping_Skull.asm loc_3F386 advances by `$10` before each `$4000` slope-up check, then writes `cell_right - (y_pos & $F)` to x_pos"
        );
    }

    #[test]
    fn mini_hopping_skull_slopedown_scan_matches_loc_3f3ce() {
        let data = vec![0u8; 8 * 4 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0x50, 0x00));

        assert_eq!(
            mini_hopping_skull_snap_slopedown_x(&grid, 55, 46),
            46u32 << 16,
            "ASM Mini_Hopping_Skull.asm loc_3F3CE samples from the cell to the right, walks left to `$5000`, then writes `cell_left + (y_pos & $F)` to x_pos"
        );
    }
}

pub(crate) fn spawn_y_velocity(family: EnemyFamily, enemy_level: u8) -> i32 {
    match family {
        // Big_Hopping_Skull.asm loc_3651E: move.w #$FFFE,y_vel -> $FFFE0000 FP16.16
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
        // ASM Enemy16_Drip_Init (Drips.asm:35): `move.l #-$10000,y_vel(a3)`
        // seeds the slime with -1.0 fp/frame upward velocity before the
        // ceiling-attach loop at loc_333F6.
        EnemyFamily::Drips => DRIPS_CEILING_ATTACH_RISE_VELOCITY,
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

pub(crate) fn big_hopping_skull_hit_points_loc_3651a(enemy_level: u8) -> u8 {
    match enemy_level {
        0 => 2,
        // Big_Hopping_Skull.asm loc_3650E maps raw layout hitpoint 1 to 3 HP
        // by branching past the 2-HP default and storing `#3` in d2.
        1 => 3,
        // Big_Hopping_Skull.asm loc_3651A caps all other raw layout hitpoint
        // values at 4 HP by storing `#4` in d2.
        _ => 4,
    }
}

/// Compute initial HP from the enemy family and the raw `enemy_level` field
/// in the layout. Each family's ASM init adds a base offset to `enemy_level`
/// to produce the actual HP value.
pub(crate) fn spawn_hit_points(family: EnemyFamily, enemy_level: u8) -> u8 {
    if family == EnemyFamily::BigHoppingSkull {
        return big_hopping_skull_hit_points_loc_3651a(enemy_level);
    }

    let base = match family {
        EnemyFamily::Goat => 3,
        EnemyFamily::Armadillo => 1,
        EnemyFamily::Crab => 2,
        EnemyFamily::Tank | EnemyFamily::TankShooting => 2,
        EnemyFamily::Scorpion => 3,
        // Driller.asm loc_3BD2A: after common visible setup, copy
        // `enemy_level(a3)` to d7, add 2, then store it in `current_hp(a3)`.
        EnemyFamily::Driller => 2,
        EnemyFamily::Cloud => 1,
        EnemyFamily::Sphere => 1,
        EnemyFamily::EmoRock => 1,
        EnemyFamily::Archer => 1,
        EnemyFamily::Fireball => 1,
        // Fire_Demon.asm loc_3A3D2: copy enemy_level(a3), add 3, store
        // current_hp in `$44(a3)`.
        EnemyFamily::FireDemon => 3,
        EnemyFamily::Robot => 2, // HP 2-3
        EnemyFamily::TarMonster => tar_monster_hit_points_loc_321c6(enemy_level),
        EnemyFamily::Orca => 4, // fixed 4 HP
        // ASM UFO.asm `$47(a5)` makes the first hurt enter loc_351EE and
        // survive; the second hurt enters loc_35174/loc_35180 death.
        EnemyFamily::UFO => 2,
        // Ninja: HP is `enemy_level + 2` and uses loc_3D59A/loc_3D5AA for speed tiers.
        EnemyFamily::Ninja => 2,
        EnemyFamily::Lion => 6, // HP 6-8
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
        // ASM Tank_Shooting.asm loc_3B5FA seeds `$54(a5)` with `$49`; the
        // `stru_3B4D8`/`stru_3B4F6` animation reaches `loc_3C50E` one frame
        // later. Plain Tank.asm never fires.
        EnemyFamily::Tank => 0,
        EnemyFamily::TankShooting => TANK_SHOOTING_INIT_FIRE_TIMER_LOC_3B5FA,
        // ASM Archer.asm loc_341BE/loc_34248: first shot waits through the
        // shooting-table pose delays, not the arrow_accuracy re-aim interval.
        EnemyFamily::Archer => ARCHER_SHOOTING_CYCLE_DELAY_FRAMES_LOC_341BE,
        // Lion: charge timer counts down from $3C (60 frames) before first charge.
        EnemyFamily::Lion => LION_CHARGE_TIMER_INIT,
        // Robot: ASM init $54(a5) low word = 1 (fires on 2nd frame).
        EnemyFamily::Robot => 1,
        // TarMonster: starts submerged, waits before first emergence.
        EnemyFamily::TarMonster => TAR_MONSTER_SUBMERGED_FRAMES,
        // ASM Drips.asm loc_3340E seeds the dormant delay AFTER the ceiling
        // attach (loc_333F6) converges. Native runs the rise-to-ceiling phase
        // first via `step_drips`, then seeds the RNG-driven dormant timer at
        // that point; on spawn the wait timer stays at 0 so the rise phase
        // executes immediately.
        EnemyFamily::Drips => 0,
        // DragonWalking: first breath after cooldown.
        EnemyFamily::DragonWalking => {
            dragon_walking_common_init_loc_3ce5a(false, Direction::Left).breath_timer
        }
        // Orca.asm loc_3D194 overwrites `$54(a5)` with `$12C` before the
        // first proximity check can trigger `loc_3D2DE`.
        EnemyFamily::Orca => ORCA_LUNGE_COOLDOWN,
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
            lower_bound: 0,
            upper_bound: u16::MAX,
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
        // ASM loc_31FD4 uses `cmp.w (a0),d1; blt.s loc_31FE6`, so both
        // camera and threshold words are signed 16-bit values.
        let position_word = i16::from_ne_bytes(position.to_ne_bytes());
        let lower_bound_word = i16::from_ne_bytes(segment.lower_bound.to_ne_bytes());
        let upper_bound_word = i16::from_ne_bytes(segment.upper_bound.to_ne_bytes());
        if position_word >= lower_bound_word {
            selected_index = index;
        }
        if position_word < upper_bound_word {
            break;
        }
    }
    let segment = &layout.segments[selected_index];
    EnemyLayoutView {
        segment_index: selected_index,
        lower_bound: segment.lower_bound,
        upper_bound: segment.upper_bound,
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
        // ASM loc_36E2C (kid.asm:38723): after skipping the Forest RAM path,
        // non-zero `d6` calls `j_DecompressToRAM_Special`.
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
    // sub_3C352 starts the pooled drive drone via j_PlaySound (GEMS $10
    // cmdstartseq) on first activation, and stops it via j_PlaySound2 (GEMS
    // $12 cmdstopseq) when the last driver exits and the refcount hits zero.
    StartSequence(SharedEnemyDriveSoundEffectSub3C352),
    StopSequence(SharedEnemyDriveSoundEffectSub3C352),
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
                .then_some(SharedEnemyDriveSoundCallSub3C352::StopSequence(effect)),
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
        sound_call: (refcount == 0)
            .then_some(SharedEnemyDriveSoundCallSub3C352::StartSequence(effect)),
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
            .then_some(SharedEnemyDriveSoundCallSub3C352::StopSequence(effect)),
    }
}

/// Result of the deactivation status table update at loc_3C4C0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeactivationStatusActionLoc3C4C0 {
    /// loc_3C4E6: bit 7 clear (positive slot) — write `$2168` to
    /// `EnemyStatus_Table[d0]`.
    WriteRespawnMarker { index: usize, word_value: u16 },
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
pub(crate) fn deactivation_status_update_loc_3C4C0(
    object_status_slot: u8,
) -> DeactivationStatusActionLoc3C4C0 {
    // ASM: moveq #0,d0; move.b $42(a5),d0; bpl.s loc_3C4E6
    if object_status_slot & 0x80 == 0 {
        // ASM loc_3C4E6: add.w d0,d0 → word index; move.w #$2168,(a0,d0.w)
        return DeactivationStatusActionLoc3C4C0::WriteRespawnMarker {
            index: usize::from(object_status_slot),
            word_value: 0x2168,
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
pub(crate) enum SharedEnemyRobotHelperKindSub3C4F8 {
    RockTankShooting,
    Robot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub(crate) fn boss_enemy_art_slot_assignment_loc_32146(
    enemy_type_slots: &[u16],
    enemy_id: u16,
) -> EnemyArtSlotAssignmentSub36E84 {
    // ASM loc_32146 (kid.asm:37566): boss object init uses the current enemy
    // layout header directly and masks slots with `$FF`, unlike generic
    // sub_36E84/loc_36EA6 which compares 12-bit enemy-load ids.
    const ENEMY_ART_PALETTE_LINES: [u8; 4] = [1, 2, 2, 0];
    const ENEMY_ART_VRAM_TILE_ADDRESSES: [u16; 3] = [0x02FB, 0x0409, 0x0517];

    let enemy_id = enemy_id & 0x00FF;
    let slot_index = if enemy_type_slots
        .first()
        .is_some_and(|slot| slot & 0x00FF == enemy_id)
    {
        0
    } else if enemy_type_slots
        .get(1)
        .is_some_and(|slot| slot & 0x00FF == enemy_id)
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
    pub(crate) fn spawn_fire_demon_walking_trails_loc_3a542(&mut self) {
        let grid = self.block_grid.as_ref();
        let mut fire_trail_count = self.fire_demon_trails.len().min(usize::from(u16::MAX)) as u16;
        let mut spawned_trails = Vec::new();

        for enemy in &mut self.active_enemies {
            if !matches!(enemy.family, EnemyFamily::FireDemon)
                || !enemy.on_ground
                || enemy.death_animation_active
                || enemy.wall_pinned
                || enemy.damage_flash_frames > 0
                || enemy.wait_timer == FIRE_DEMON_BERZERKER_LAUNCH_WAIT_TIMER_LOC_3A610
            {
                continue;
            }

            let step = fire_demon_walking_trail_step_loc_3a542(
                u16::from(enemy.enemy_level),
                enemy.grounded_frames,
                fire_trail_count,
            );
            enemy.grounded_frames = step.next_animation_phase;
            if !step.spawns_fire_trail {
                continue;
            }

            let trail = fire_demon_death_trail_child_loc_3a63c(enemy.x_pos, enemy.y_pos);
            if grid.is_some_and(|grid| {
                !fire_demon_trail_has_floor_support_loc_3a63c(grid, trail.x_pos, trail.y_pos)
            }) {
                // ASM Fire_Demon.asm loc_3A63C immediately reaches loc_3A6DA
                // when the three sampled floor cells fail the `$6000` support
                // checks, undoing the allocation count before the child can be
                // observed.
                continue;
            }

            fire_trail_count = step.next_fire_trail_count;
            spawned_trails.push(trail);
        }

        self.fire_demon_trails.extend(spawned_trails);
    }

    pub(crate) fn prime_respawn_settle_enemy_loading_loc_368a8(&mut self, level_id: LevelId) {
        if !self.enemy_loading_enabled() {
            return;
        }
        let Some(enemy_layout) = self.enemy_layout(level_id).cloned() else {
            return;
        };
        let level_bounds = self.level_bounds(level_id);
        let blank_top_rows = self.level_header(level_id).map_or(0, |header| {
            terrain::blank_top_rows_from_height_flag_bits_sub_140F6(header.height_flag_bits)
        });

        // ASM Manage_EnemyLoading loc_368A8 runs while the hidden spawn settle
        // camera is still converging. Native floor-snap computes that path
        // atomically, so replay the load windows here.
        let settle_frames = self.inspect_respawn_settle(level_id, 180);
        let final_camera_x_pos = settle_frames.last().map_or(0, |frame| frame.camera_x_pos);
        for frame in settle_frames {
            let camera = CameraState {
                x_pos: frame.camera_x_pos,
                y_pos: frame.camera_y_pos,
            };
            let window = ActivationWindow::from_camera(camera);
            activate_enemies_with_rng(
                &mut self.active_enemies,
                &enemy_layout,
                window,
                frame.camera_x_pos >> 16,
                frame.camera_y_pos >> 16,
                level_bounds,
                frame.kid_x_pos,
                frame.kid_y_pos,
                blank_top_rows,
                self.enemy_loading_initial_pass_complete,
                &self.killed_enemies,
                &mut self.rom_random,
            );
            self.ensure_heady_metal_breakup_slots_loc_39247();
            if frame.camera_x_pos == final_camera_x_pos
                && level_id != LevelId(0)
                && level_id != LevelId(1)
            {
                // ASM Mode_Level's hidden respawn settle keeps executing the
                // object scheduler after the horizontal camera axis has reached
                // its settled target. Level 0x07's Lion first exposed this via
                // Lion.asm loc_3E7CA's off-ledge charge fall; Level 0x06's
                // Flying Dragon proves the same tail must age active families
                // through Dragon_Flying.asm loc_34710 -> loc_348A0 -> loc_34BAC
                // before the recording helper's 60 visible live frames begin.
                step_active_enemies_with_rng(
                    &mut self.active_enemies,
                    self.block_grid.as_ref(),
                    Some(frame.kid_x_pos),
                    Some(frame.kid_y_pos),
                    Some(frame.camera_x_pos >> 16),
                    Some(frame.camera_y_pos >> 16),
                    None,
                    Some(frame.kid_x_velocity),
                    Some(frame.kid_y_velocity),
                    self.progression.current_form,
                    self.frame_index as u16,
                    false,
                    Some(&mut self.projectiles),
                    &mut self.rom_random,
                    &mut self.drilled_blocks,
                    None,
                );
                self.move_heady_metal_body_piece_workers_sub_376ec();
                self.spawn_fire_demon_walking_trails_loc_3a542();
                materialize_spinning_twins_drop_splits(&mut self.active_enemies);
                let drilled_blocks = std::mem::take(&mut self.drilled_blocks);
                for drilled_block in drilled_blocks {
                    self.apply_drilled_block_loc_3bede_3bef2(drilled_block);
                }
            }
        }
        self.rebuild_active_enemy_roster();
    }

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

    pub(crate) fn hibernate_enemy_death_animation_frame(enemy: &mut ActiveEnemy) -> bool {
        if !enemy.death_animation_active
            || enemy_death_animation_frames(enemy.family).is_none()
            || enemy.wait_timer == 0
        {
            return false;
        }

        if matches!(enemy.family, EnemyFamily::UFO) {
            // ASM UFO.asm loc_35180 sets `is_moved(a3)`, preserves the
            // saucer's existing x_vel, and forces y_vel to `$30000` while
            // loc_351B6 hibernates through the parent death animation.
            enemy.x_pos = enemy.x_pos.wrapping_add_signed(enemy.x_velocity);
            enemy.y_pos = enemy.y_pos.wrapping_add_signed(enemy.y_velocity);
        }
        if matches!(enemy.family, EnemyFamily::BigHoppingSkull) {
            let death_wait = big_hopping_skull_death_wait_loc_36664(
                true,
                enemy.wait_timer == 1,
                enemy.y_velocity,
            );
            debug_assert!(death_wait.hibernates_one_frame);
            debug_assert!(death_wait.checks_object_range);
            debug_assert_eq!(death_wait.collision_probe_extent, 8);
            enemy.y_velocity = death_wait.y_velocity;
            match death_wait.route {
                BigHoppingSkullDeathWaitRouteLoc36664::LoopDeathAnimation => {
                    debug_assert!(enemy.wait_timer > 1);
                }
                BigHoppingSkullDeathWaitRouteLoc36664::FinalizeDeletion => {
                    debug_assert_eq!(enemy.wait_timer, 1);
                }
            }
        }
        // ASM Dragon.asm loc_3CEFC and the other death-animation tails call
        // j_Hibernate_UntilAnimFinished; the frame that consumes the last
        // animation tick still returns to the scheduler as a hibernated object.
        enemy.wait_timer = enemy.wait_timer.saturating_sub(1);
        true
    }

    pub(crate) fn queue_death_animation_frame_sounds(&mut self, idx: usize) {
        let Some(enemy) = self.active_enemies.get(idx) else {
            return;
        };
        if is_multi_head_boss(enemy.family)
            && enemy.death_animation_active
            && enemy.wait_timer == BOSS_HEAD_DEATH_SPIN_FRAMES_LOC_39F7A
        {
            // ASM loc_39EE2: `sfx_Boss_dies` plays after the first eight
            // hibernate-until-finished scripts, immediately before loc_39F7A.
            self.queue_start_sequence(QueuedSoundEffect::BossDies);
        }
    }

    /// Pooled count of Rock Tanks (either variant) whose drive SFX has started
    /// this life. Native recomputes the ROM's `$FFFFFA34` drive-sound refcount
    /// each frame from `shared_walker_patrol_short_next` (see the START in
    /// `step_active_enemies_with_rng`); both RockTank variants resolve to the
    /// same sfx slot (`byte_3C46C[1] = sfx_Tank_driving`), so they pool into one
    /// count here exactly as they do at the START.
    pub(crate) fn driving_rock_tank_count(&self) -> usize {
        self.active_enemies
            .iter()
            .filter(|enemy| {
                matches!(enemy.family, EnemyFamily::Tank | EnemyFamily::TankShooting)
                    && enemy.shared_walker_patrol_short_next
            })
            .count()
    }

    /// ASM loc_3C46E -> loc_3C4A8 (kid.asm:43965-43997): the shared deactivation
    /// routine does `subi.b #1,(a4,d7.w)` on the `$FFFFFA34` drive-sound
    /// refcount as each Rock Tank leaves, then plays `sfx_Tank_driving` through
    /// `j_PlaySound2` only when that decrement reaches zero. Both exits route
    /// here: scroll-off via the loc_3C436 bounds trampoline (kid.asm:43937) and
    /// death via Tank.asm loc_3B480 / Tank_Shooting.asm (`bra.w loc_3C46E` after
    /// `Hibernate_UntilAnimFinished`). Native's pooled recompute model collapses
    /// that per-object decrement to a single edge, and `remove_dead_enemy` only
    /// drops a corpse once its death animation has finished — matching the
    /// ROM's post-hibernate timing — so the stop fires when a frame's
    /// activations and death-removals together drop the pooled count to zero.
    /// #339: the stop was unported, so the drive drone looped forever once the
    /// last Rock Tank left the screen.
    pub(crate) fn stop_rock_tank_drive_sound_on_last_exit_loc_3c46e(
        &mut self,
        driving_before: usize,
    ) {
        let driving_after = self.driving_rock_tank_count();
        let mut drive_refcount = driving_before.min(usize::from(u8::MAX)) as u8;
        let mut stop_drive_sound = false;

        for _ in 0..driving_before.saturating_sub(driving_after) {
            let cleanup = deactivation_drive_sound_cleanup_loc_3C494(
                EnemyFamily::Tank,
                false,
                false,
                drive_refcount,
            );
            drive_refcount = cleanup.next_refcount;
            if matches!(
                cleanup.sound_call,
                Some(SharedEnemyDriveSoundCallSub3C352::StopSequence(
                    SharedEnemyDriveSoundEffectSub3C352::TankDriving
                ))
            ) {
                stop_drive_sound = true;
            }
        }

        if driving_before >= driving_after {
            debug_assert_eq!(
                usize::from(drive_refcount),
                driving_after.min(usize::from(u8::MAX))
            );
        }
        if stop_drive_sound {
            self.queue_stop_sequence(QueuedSoundEffect::TankDriving);
        }
    }

    /// ASM `Load_InGame` loc_23629 (kid.asm:23629) calls `j_StopMusic` on every
    /// level (re)entry — a global GEMS driver reset that silences any looping
    /// drone still playing (Rock Tank drive `$1F`, clock-prize `$0E`). Native
    /// models those drones as discrete Start/StopSequence pairs whose stop is an
    /// edge on the live roster (`stop_rock_tank_drive_sound_on_last_exit_loc_3c46e`,
    /// `step_visual_effects`). That edge never fires across a death-respawn or
    /// level-skip, because those paths bulk-clear `active_enemies`/`floating_items`
    /// without the per-exit decrement — so the drone looped forever after a Kid
    /// death and re-accumulated on every skip (#339 REOPEN). Mirror j_StopMusic by
    /// stopping the drones that are still live at the same teardown, before the
    /// bulk clears drop them.
    pub(crate) fn stop_level_looping_drones_on_load_ingame_loc_23629(&mut self) {
        if self.driving_rock_tank_count() > 0 {
            self.queue_stop_sequence(QueuedSoundEffect::TankDriving);
        }
        if self.active_clock_prize_count() > 0 {
            self.queue_stop_sequence(QueuedSoundEffect::ClockPrize);
        }
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
        self.active_platform_standing_on = None;
        self.active_lift_platform_standing_on = None;
        if self.level_skip_resume_hold {
            // ASM Load_InGame loc_23629 j_StopMusic resets the GEMS driver as the
            // skip target loads; mirror it before clearing the prior level's live
            // Rock Tank / clock-prize drones so the drone does not re-accumulate
            // across each skip. #339.
            self.stop_level_looping_drones_on_load_ingame_loc_23629();
            self.active_enemies.clear();
            self.pending_dragon_death_children.clear();
            self.pending_dragon_death_animations.clear();
            self.enemy_status_words.fill(0);
            self.enemy_loading_initial_pass_complete = false;
            self.block_state.evanescent_blocks.clear();
            self.clear_block_object_workers_loc_6e2();
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
        if matches!(self.player_lifecycle, PlayerLifecycle::DeathWindup { .. }) {
            // ASM Death/loc_B80C loops inside the death routine until the
            // sprite exits, so Mode_Level loc_80C does not resume object
            // execution or enemy loading during the visible tumble.
            self.rebuild_active_enemy_roster();
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
                // #339: snapshot the pooled drive-sound refcount before this
                // frame's activations/death-removals so loc_3C46E's stop can
                // fire when the last driving Rock Tank leaves (see
                // `stop_rock_tank_drive_sound_on_last_exit_loc_3c46e`).
                let driving_rock_tanks_before = self.driving_rock_tank_count();
                if self.enemy_loading_initial_pass_complete {
                    // ASM Mode_Level loc_80C calls Manage_EnemyLoading after
                    // Execute_Objects for live post-initial-pass entrants.
                    // The Level 01 RockTank pair at `$0288,$008F` exposes that
                    // order in oracle traces; other families retain native's
                    // established pre-step timing until their own oracle cases
                    // prove a different first-frame phase.
                    activate_enemies_with_rng_inner(
                        &mut self.active_enemies,
                        enemy_layout,
                        window,
                        camera_x_px,
                        camera_y_px,
                        level_bounds,
                        slice.player.x_pos,
                        slice.player.y_pos,
                        blank_top_rows,
                        self.enemy_loading_initial_pass_complete,
                        &self.killed_enemies,
                        &mut self.rom_random,
                        true,
                    );
                } else {
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
                        self.enemy_loading_initial_pass_complete,
                        &self.killed_enemies,
                        &mut self.rom_random,
                    );
                }
                self.ensure_heady_metal_breakup_slots_loc_39247();
                self.apply_pending_enemy_collisions();
                let mut played_enemy_sound_effects = Vec::new();
                step_active_enemies_with_rng(
                    &mut self.active_enemies,
                    self.block_grid.as_ref(),
                    Some(slice.player.x_pos),
                    Some(slice.player.y_pos),
                    Some(camera_x_px),
                    Some(camera_y_px),
                    Some(slice.player.facing),
                    Some(slice.player.x_velocity),
                    Some(slice.player.y_velocity),
                    self.progression.current_form,
                    self.frame_index as u16,
                    self.progression.diamond_power_type
                        == crate::helmets::DiamondPowerType::SamuraiHaze
                        && self.progression.diamond_power_frames > 0,
                    Some(&mut self.projectiles),
                    &mut self.rom_random,
                    &mut self.drilled_blocks,
                    Some(&mut played_enemy_sound_effects),
                );
                self.move_heady_metal_body_piece_workers_sub_376ec();
                for effect in played_enemy_sound_effects {
                    self.queue_start_sequence(effect);
                }
                self.spawn_fire_demon_walking_trails_loc_3a542();
                materialize_spinning_twins_drop_splits(&mut self.active_enemies);
                let mut death_animation_hibernated_indices = Vec::new();
                let mut death_animation_sound_indices = Vec::new();
                for (idx, enemy) in self.active_enemies.iter_mut().enumerate() {
                    if Self::hibernate_enemy_death_animation_frame(enemy) {
                        if is_multi_head_boss(enemy.family)
                            && enemy.wait_timer == BOSS_HEAD_DEATH_SPIN_FRAMES_LOC_39F7A
                        {
                            death_animation_sound_indices.push(idx);
                        }
                        death_animation_hibernated_indices.push(idx);
                    }
                }
                for idx in death_animation_sound_indices {
                    self.queue_death_animation_frame_sounds(idx);
                }
                let dead_enemy_indices: Vec<usize> = self
                    .active_enemies
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, enemy)| {
                        (enemy.death_animation_active
                            && !death_animation_hibernated_indices.contains(&idx))
                        .then_some(idx)
                    })
                    .collect();
                for idx in dead_enemy_indices.into_iter().rev() {
                    self.remove_dead_enemy(idx);
                }
                // Apply drilled blocks to the block grid after enemy step.
                // Driller.asm loc_3BE78 dispatches Smash through loc_3BEDE
                // and Ice through loc_3BEF2 before the shared walker resumes.
                let drilled_blocks = std::mem::take(&mut self.drilled_blocks);
                for drilled_block in drilled_blocks {
                    self.apply_drilled_block_loc_3bede_3bef2(drilled_block);
                }
                self.drilled_blocks.clear();
                // ASM Mode_Level loc_80C calls Manage_EnemyLoading after
                // Execute_Objects. Newly allocated RockTank enemies are
                // visible in WRAM this frame, but their init/move routine does
                // not run until the next object pass.
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
                    self.enemy_loading_initial_pass_complete,
                    &self.killed_enemies,
                    &mut self.rom_random,
                );
                self.ensure_heady_metal_breakup_slots_loc_39247();
                self.stop_rock_tank_drive_sound_on_last_exit_loc_3c46e(driving_rock_tanks_before);
                self.rebuild_active_enemy_roster();
            } else {
                // ORIGINAL_QUIRK: during title-screen demo playback, the game
                // suppresses enemy loading for every level except Knights Isle.
                self.active_enemies.clear();
                self.pending_dragon_death_children.clear();
                self.pending_dragon_death_animations.clear();
                self.enemy_status_words.fill(0);
                self.enemy_loading_initial_pass_complete = false;
                self.active_enemy_roster.clear();
            }
        }
        if let Some(platform_layout) = platform_layout.as_ref() {
            let player_x_px = slice.player.x_pos >> 16;
            let player_y_px = slice.player.y_pos >> 16;
            let platform_window = ActivationWindow {
                x_min: player_x_px,
                x_max: player_x_px,
                y_min: player_y_px,
                y_max: player_y_px,
            };
            let mut objects = slice.objects;
            objects
                .platform_loader
                .advance_to_window(platform_layout, platform_window);
            self.active_platform_roster = objects
                .platform_loader
                .active_platforms(platform_layout, platform_window);
            let script_table = self.platform_script_table.as_ref();
            let previous_platforms = self.active_platforms.clone();
            let previous_standing_slot_addr = self.active_platform_standing_on;
            self.active_platforms = runtime_active_platforms_from_loader_loc_257E(
                platform_layout,
                objects.platform_loader,
                platform_window,
                &previous_platforms,
                script_table,
            );
            expand_platform_chain_segments_loc_428C(
                &mut self.active_platforms,
                &previous_platforms,
            );
            self.active_platform_standing_on = previous_standing_slot_addr.filter(|slot_addr| {
                self.active_platforms
                    .iter()
                    .any(|platform| platform.slot_addr == Some(*slot_addr))
            });
            self.opening_slice = Some(OpeningSliceState { objects, ..slice });
        }

        // ASM Hand.asm: the grabbing Hand object writes the global
        // `KidGrabbedByHand` flag ($FFFFFA28) — `st` on grab entry
        // (loc_32906), held through the follow loop (loc_3292E), and `sf` on
        // release (loc_32970). Native models the grab as `HandState::Grabbing`,
        // so the flag is true exactly while some active Hand is grabbing Kid.
        // The player jump/crawl/climb paths read this off the slice this same
        // frame because the enemy step (which sets it) runs in
        // `refresh_active_roster` before `refresh_opening_slice` (the player
        // step). Default-false is preserved whenever no Hand is grabbing, so
        // normal play is byte-identical.
        let kid_grabbed_by_hand = self.active_enemies.iter().any(|enemy| {
            matches!(enemy.family, EnemyFamily::Hand)
                && matches!(enemy.hand_state, HandState::Grabbing)
        });
        if let Some(slice) = self.opening_slice.as_mut() {
            slice.set_kid_grabbed_by_hand(kid_grabbed_by_hand);
        }
    }
}

/// Permanent guard for the collision-ID deduplication (issue #41).
///
/// Every per-family `*_COLID_*` alias now resolves to a value in the shared
/// `colid` module instead of its own hardcoded literal. A single wrong canonical
/// value would silently corrupt a collision ID across many families, and the
/// fast gate may not surface it — so this test pins the exact pre-refactor number
/// for every alias against a *hardcoded literal* (NOT `colid::X`, which would be a
/// tautology). A typo in any `colid::` value then fails here loudly.
#[cfg(test)]
mod colid_dedup_regression {
    use super::*;

    /// Layer 1 — the load-bearing layer. Each alias must still equal the exact
    /// literal it had before the refactor (`grep _COLID_` snapshot, 2026-06-22).
    /// Hardcoded numbers on purpose: comparing against `colid::X` would be circular.
    #[test]
    fn family_aliases_keep_their_pre_refactor_values() {
        // u16 families
        assert_eq!(LION_COLID_HURT, 0x001Cu16);
        assert_eq!(LION_COLID_KIDABOVE, 0x002Cu16);
        assert_eq!(NINJA_COLID_RIGHTWALL, 0x0004u16);
        assert_eq!(NINJA_COLID_LEFTWALL, 0x0008u16);
        assert_eq!(NINJA_COLID_FLOOR, 0x000Cu16);
        assert_eq!(NINJA_COLID_CEILING, 0x0010u16);
        assert_eq!(NINJA_COLID_HURT, 0x001Cu16);
        assert_eq!(NINJA_COLID_KIDABOVE, 0x002Cu16);
        assert_eq!(NINJA_COLID_KIDRIGHT, 0x0020u16);
        assert_eq!(NINJA_COLID_KIDLEFT, 0x0024u16);
        assert_eq!(DRAGON_COLID_HURT, 0x001Cu16);
        assert_eq!(DRAGON_COLID_KIDRIGHT, 0x0020u16);
        assert_eq!(DRAGON_COLID_KIDLEFT, 0x0024u16);
        assert_eq!(DRAGON_COLID_KIDABOVE, 0x002Cu16);
        assert_eq!(DRILLER_COLID_KIDABOVE, 0x002Cu16);
        assert_eq!(TANK_COLID_HURT, 0x001Cu16);
        assert_eq!(CRAB_COLID_RIGHTWALL, 0x0004u16);
        assert_eq!(CRAB_COLID_LEFTWALL, 0x0008u16);
        assert_eq!(CRAB_COLID_SLOPEUP, 0x0014u16);
        assert_eq!(CRAB_COLID_SLOPEDOWN, 0x0018u16);
        assert_eq!(DIAMOND_COLID_RIGHTWALL, 0x0004u16);
        assert_eq!(DIAMOND_COLID_LEFTWALL, 0x0008u16);
        assert_eq!(DIAMOND_COLID_FLOOR, 0x000Cu16);
        assert_eq!(DIAMOND_COLID_CEILING, 0x0010u16);
        assert_eq!(DIAMOND_COLID_SLOPEUP, 0x0014u16);
        assert_eq!(DIAMOND_COLID_SLOPEDOWN, 0x0018u16);
        assert_eq!(DIAMOND_COLID_HURT, 0x001Cu16);
        assert_eq!(DIAMOND_COLID_KIDRIGHT, 0x0020u16);
        assert_eq!(DIAMOND_COLID_KIDLEFT, 0x0024u16);
        assert_eq!(MINI_SKULL_COLID_RIGHTWALL, 0x0004u16);
        assert_eq!(MINI_SKULL_COLID_LEFTWALL, 0x0008u16);
        assert_eq!(MINI_SKULL_COLID_FLOOR, 0x000Cu16);
        assert_eq!(MINI_SKULL_COLID_CEILING, 0x0010u16);
        assert_eq!(MINI_SKULL_COLID_SLOPEUP, 0x0014u16);
        assert_eq!(MINI_SKULL_COLID_SLOPEDOWN, 0x0018u16);
        assert_eq!(MINI_SKULL_COLID_HURT, 0x001Cu16);
        assert_eq!(DRAGON_FLYING_COLID_RIGHTWALL, 0x0004u16);
        assert_eq!(DRAGON_FLYING_COLID_LEFTWALL, 0x0008u16);
        assert_eq!(DRAGON_FLYING_COLID_FLOOR, 0x000Cu16);
        assert_eq!(DRAGON_FLYING_COLID_CEILING, 0x0010u16);
        assert_eq!(DRAGON_FLYING_COLID_SLOPEUP, 0x0014u16);
        assert_eq!(DRAGON_FLYING_COLID_SLOPEDOWN, 0x0018u16);
        assert_eq!(DRIPS_COLID_CEILING, 0x0010u16);
        assert_eq!(SPINNING_TWINS_COLID_RIGHTWALL, 0x0004u16);
        assert_eq!(SPINNING_TWINS_COLID_LEFTWALL, 0x0008u16);
        assert_eq!(SPINNING_TWINS_COLID_FLOOR, 0x000Cu16);
        assert_eq!(SPINNING_TWINS_COLID_HURT, 0x001Cu16);
        assert_eq!(SPINNING_TWINS_COLID_KIDRIGHT, 0x0020u16);
        assert_eq!(SPINNING_TWINS_COLID_KIDLEFT, 0x0024u16);
        assert_eq!(SPINNING_TWINS_COLID_KIDABOVE, 0x002Cu16);

        // i16 families
        assert_eq!(CRAB_COLID_HURT, 0x001Ci16);
        assert_eq!(CRAB_COLID_KIDRIGHT, 0x0020i16);
        assert_eq!(CRAB_COLID_KIDLEFT, 0x0024i16);
        assert_eq!(GOAT_COLID_HURT, 0x001Ci16);
        assert_eq!(GOAT_COLID_KIDRIGHT, 0x0020i16);
        assert_eq!(GOAT_COLID_KIDLEFT, 0x0024i16);
        assert_eq!(GOAT_COLID_KIDABOVE, 0x002Ci16);
        assert_eq!(ARMADILLO_COLID_FLOOR, 0x000Ci16);
        assert_eq!(ARMADILLO_COLID_HURT, 0x001Ci16);
        assert_eq!(ARMADILLO_COLID_KIDRIGHT, 0x0020i16);
        assert_eq!(ARMADILLO_COLID_KIDLEFT, 0x0024i16);
        assert_eq!(ARMADILLO_COLID_KIDABOVE, 0x002Ci16);
        assert_eq!(SCORPION_COLID_HURT, 0x001Ci16);
        assert_eq!(SCORPION_COLID_KIDRIGHT, 0x0020i16);
        assert_eq!(SCORPION_COLID_KIDLEFT, 0x0024i16);
        assert_eq!(SCORPION_COLID_KIDABOVE, 0x002Ci16);
        assert_eq!(ORCA_COLID_RIGHTWALL, 0x0004i16);
        assert_eq!(ORCA_COLID_LEFTWALL, 0x0008i16);
        assert_eq!(ORCA_COLID_FLOOR, 0x000Ci16);
        assert_eq!(ORCA_COLID_CEILING, 0x0010i16);
        assert_eq!(ORCA_COLID_SLOPEUP, 0x0014i16);
        assert_eq!(ORCA_COLID_SLOPEDOWN, 0x0018i16);
        assert_eq!(ORCA_COLID_HURT, 0x001Ci16);
        assert_eq!(ORCA_COLID_KIDRIGHT, 0x0020i16);
        assert_eq!(ORCA_COLID_KIDLEFT, 0x0024i16);
        assert_eq!(ORCA_COLID_KIDABOVE, 0x002Ci16);
        assert_eq!(FIRE_DEMON_COLID_HURT, 0x001Ci16);
        assert_eq!(FIRE_DEMON_COLID_KIDRIGHT, 0x0020i16);
        assert_eq!(FIRE_DEMON_COLID_KIDLEFT, 0x0024i16);
    }

    /// Layer 2 — documents the canonical ROM map. Each shared value must match
    /// the `; Collision` table in `constants.asm:163-175` exactly.
    #[test]
    fn shared_module_matches_constants_asm() {
        assert_eq!(colid::EMPTY, 0x0000u16); // colid_empty
        assert_eq!(colid::RIGHTWALL, 0x0004u16); // colid_rightwall
        assert_eq!(colid::LEFTWALL, 0x0008u16); // colid_leftwall
        assert_eq!(colid::FLOOR, 0x000Cu16); // colid_floor
        assert_eq!(colid::CEILING, 0x0010u16); // colid_ceiling
        assert_eq!(colid::SLOPEUP, 0x0014u16); // colid_slopeup
        assert_eq!(colid::SLOPEDOWN, 0x0018u16); // colid_slopedown
        assert_eq!(colid::HURT, 0x001Cu16); // colid_hurt
        assert_eq!(colid::KIDRIGHT, 0x0020u16); // colid_kidright
        assert_eq!(colid::KIDLEFT, 0x0024u16); // colid_kidleft
        assert_eq!(colid::KIDBELOW, 0x0028u16); // colid_kidbelow
        assert_eq!(colid::KIDABOVE, 0x002Cu16); // colid_kidabove
    }

    /// Layer 3 — excluded-set sanity. The `MINI_SKULL_COLID_GROUND_*` set
    /// deliberately remaps suffix→non-canonical value (consumed by real combat
    /// at combat.rs:8752+); it must NOT be folded into `colid`. Fails loudly if a
    /// future edit wrongly canonicalises it.
    #[test]
    fn mini_skull_ground_set_stays_non_canonical() {
        assert_eq!(MINI_SKULL_COLID_GROUND_KIDABOVE, 0x001Cu16);
        assert_eq!(MINI_SKULL_COLID_GROUND_KIDLEFT, 0x0020u16);
        assert_eq!(MINI_SKULL_COLID_GROUND_KIDRIGHT, 0x0024u16);
        assert_eq!(MINI_SKULL_COLID_GROUND_KIDBELOW, 0x0028u16);
        assert_eq!(MINI_SKULL_COLID_GROUND_HURT, 0x002Cu16);
        // The remap is genuine: GROUND_KIDABOVE is the hurt value, not the
        // canonical kidabove value — proves these are NOT shared aliases.
        assert_ne!(MINI_SKULL_COLID_GROUND_KIDABOVE, colid::KIDABOVE);
    }
}
