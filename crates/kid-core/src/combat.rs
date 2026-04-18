use super::*;

const HUNDRED_K_TRIP_START_LEVEL_ID: LevelId = LevelId(4);
const HUNDRED_K_TRIP_ARM_FRAMES_LOC_10AC2: u16 = 0x003C;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10000PointsInitLoc10Ae0 {
    pub(crate) cached_x_pos: u32,
    pub(crate) cached_y_pos: u32,
    pub(crate) worker_index_d3: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn prize_10000_points_init_loc_10AE0(
    x_pos: u32,
    y_pos: u32,
) -> Prize10000PointsInitLoc10Ae0 {
    // ASM loc_10AE0 (kid.asm:22384): cache the parent prize object's
    // `x_pos(a3)`/`y_pos(a3)` into `$6C(a5)`/`$70(a5)`, run `j_loc_1078`, then
    // seed `d3 = 0` before the later worker-spawn loop at `loc_10B00`.
    Prize10000PointsInitLoc10Ae0 {
        cached_x_pos: x_pos,
        cached_y_pos: y_pos,
        worker_index_d3: 0,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn prize_10000_points_clear_slots_loc_10AFA(mut worker_slots: [u32; 5]) -> [u32; 5] {
    // ASM loc_10AFA (kid.asm:22393): `moveq #4,d0` enters a `clr.l (a0)+;
    // dbf d0,loc_10AFA` loop, so exactly five longwords at `$44(a5)` are
    // zeroed before the later prize-number worker scan starts.
    worker_slots.fill(0);
    worker_slots
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueuedSoundEffect {
    MiniHoppingSkullScream,
    RedStealthAttackDown,
    #[cfg_attr(not(test), allow(dead_code))]
    Thunderstorm,
    #[cfg_attr(not(test), allow(dead_code))]
    NavigateJingle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PendingSoundEffect {
    pub(crate) effect: QueuedSoundEffect,
    pub(crate) delay_frames: u8,
    pub(crate) suppress: bool,
}

/// Per-helmet transform animation frame counts from ASM Kid_Transform tables
/// (kid.asm:8678-8812, stru_7EFA..stru_808A). TheKid (helmet 0) has no table.
/// These counts sum the first `anim_frame` field only; the second field is sprite/meta
/// data, not extra duration.
fn transform_anim_frames(form: PlayerForm) -> u16 {
    match form {
        PlayerForm::TheKid => 0,
        PlayerForm::Skycutter => 47,  // stru_7EFA
        PlayerForm::Cyclone => 72,    // stru_7F30
        PlayerForm::RedStealth => 54, // stru_7F5A
        PlayerForm::Eyeclops => 93,   // stru_7F90
        PlayerForm::Juggernaut => 33, // stru_7FCA
        PlayerForm::IronKnight => 81, // stru_7FF8
        PlayerForm::Berzerker => 51,  // stru_803A
        PlayerForm::Maniaxe => 45,    // stru_8064
        PlayerForm::Micromax => 59,   // stru_808A
    }
}

fn transform_hitpoints_sub_7eb2(form: PlayerForm, extra_hitpoint_slots: u16) -> u16 {
    // ASM sub_7EB2 (kid.asm:8614): load the base hitpoint byte from `unk_7EC2`
    // using the helmet id, extend it to a word, then add `Extra_hitpoint_slots`.
    form.max_hitpoints().saturating_add(extra_hitpoint_slots)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyDeathBurstInitSub37002 {
    pub(crate) sound_effect_label: &'static str,
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) x_direction: u8,
    pub(crate) sprite_mapping_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) sprite_word_y_adjust: u16,
    pub(crate) chase_frames: u16,
    pub(crate) homing_accel: i32,
    pub(crate) delete_collision_types: [i16; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyDeathBurstHomingStepSub37002 {
    pub(crate) x_direction: u8,
    pub(crate) x_velocity_delta: i32,
    pub(crate) y_velocity_delta: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyDeathBurstLoopNextLoc37054 {
    ChaseKid,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyDeathBurstLoopGateLoc37054 {
    pub(crate) next_collision_type: i16,
    pub(crate) next_chase_frames: u16,
    pub(crate) next: EnemyDeathBurstLoopNextLoc37054,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnemyDeathBurstCleanupLoc370C2 {
    pub(crate) collision_type: i16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) delete_current_object: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_death_burst_init_sub_37002(
    x_pos: u16,
    y_pos: u16,
) -> EnemyDeathBurstInitSub37002 {
    // ASM sub_37002 (kid.asm:38894): play `sfx_Boss_eye_pops`, allocate gfx
    // slot `$01010002`, seed the shared burst sprite, then run a 60-frame
    // homing loop that deletes on collision types `$001C`, `$002C`, or `$FFFF`.
    EnemyDeathBurstInitSub37002 {
        sound_effect_label: "sfx_Boss_eye_pops",
        gfx_slot_base: 0x0101_0002,
        priority: 0,
        object_meta: 0x0020,
        display_enabled: true,
        is_moved: true,
        x_direction: 0,
        sprite_mapping_label: "LnkTo_unk_C8460-Data_Index",
        x_pos,
        y_pos,
        sprite_word_y_adjust: 4,
        chase_frames: 0x003C,
        homing_accel: 0x0000_0FA0,
        delete_collision_types: [0x001C, 0x002C, -1],
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_death_burst_loop_gate_loc_37054(
    collision_type: i16,
    chase_frames: u16,
) -> EnemyDeathBurstLoopGateLoc37054 {
    // ASM loc_37054 (kid.asm:38915): after one hibernated frame, delete on
    // collision types `$001C`, `$002C`, or `$FFFF`; otherwise clear
    // `collision_type(a3)`, decrement `d0`, and continue only when non-zero.
    let collided = matches!(collision_type, 0x001C | 0x002C | -1);
    if collided {
        return EnemyDeathBurstLoopGateLoc37054 {
            next_collision_type: collision_type,
            next_chase_frames: chase_frames,
            next: EnemyDeathBurstLoopNextLoc37054::Delete,
        };
    }

    let next_chase_frames = chase_frames.wrapping_sub(1);
    let next = if next_chase_frames == 0 {
        EnemyDeathBurstLoopNextLoc37054::Delete
    } else {
        EnemyDeathBurstLoopNextLoc37054::ChaseKid
    };

    EnemyDeathBurstLoopGateLoc37054 {
        next_collision_type: 0,
        next_chase_frames,
        next,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_death_burst_cleanup_loc_370C2() -> EnemyDeathBurstCleanupLoc370C2 {
    // ASM loc_370C2 (kid.asm:38961): clear `collision_type(a3)`,
    // `x_vel(a3)`, and `y_vel(a3)`, then jump to `j_Delete_CurrentObject`.
    EnemyDeathBurstCleanupLoc370C2 {
        collision_type: 0,
        x_velocity: 0,
        y_velocity: 0,
        delete_current_object: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_death_burst_homing_step_sub_37002(
    effect_x: u16,
    effect_y: u16,
    kid_x: u16,
    kid_y: u16,
) -> EnemyDeathBurstHomingStepSub37002 {
    // ASM loc_37078..loc_370C0 (kid.asm:38930): compare the spawned burst
    // sprite against Kid and accelerate toward him by `$FA0` on each axis.
    let homing_accel = 0x0000_0FA0;
    let (x_direction, x_velocity_delta) = if effect_x > kid_x {
        // ASM loc_3709E (kid.asm:38943): burst right of Kid clears
        // `x_direction(a3)` and subtracts `$FA0` from `x_vel(a3)`.
        (0, -homing_accel)
    } else {
        (0xFF, homing_accel)
    };
    // ASM loc_370AA (kid.asm:38947): after horizontal steering, compare
    // `y_pos(a3)` against Kid's y position and accelerate vertically toward it.
    let y_velocity_delta = if effect_y > kid_y {
        // ASM loc_370B8 (kid.asm:38954): burst below Kid subtracts `$FA0`
        // from `y_vel(a3)`.
        -homing_accel
    } else {
        homing_accel
    };

    EnemyDeathBurstHomingStepSub37002 {
        x_direction,
        x_velocity_delta,
        y_velocity_delta,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossDeathBurstInitSub370D2 {
    pub(crate) sound_effect_label: &'static str,
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) x_direction: u8,
    pub(crate) sprite_mapping_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) sprite_word_y_adjust: u16,
    pub(crate) chase_frames: u16,
    pub(crate) landing_gravity: i32,
    pub(crate) homing_accel: i32,
    pub(crate) entry_animation_label: &'static str,
    pub(crate) collision_animation_label: &'static str,
    pub(crate) timeout_animation_label: &'static str,
    pub(crate) delete_collision_types: [i16; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossDeathBurstHomingStepSub370D2 {
    pub(crate) x_direction: u8,
    pub(crate) x_velocity_delta: i32,
    pub(crate) y_velocity_delta: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BossDeathBurstLoopNextLoc37148 {
    ChaseKid,
    CollisionFinish,
    TimeoutFinish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossDeathBurstLoopGateLoc37148 {
    pub(crate) next_collision_type: i16,
    pub(crate) next_chase_frames: u16,
    pub(crate) next: BossDeathBurstLoopNextLoc37148,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossDeathBurstCollisionFinishLoc371B6 {
    pub(crate) collision_type: i16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) animation_label: &'static str,
    pub(crate) hibernate_until_anim_finished: bool,
    pub(crate) delete_current_object: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BossDeathBurstTimeoutLoopNextLoc3720A {
    ContinueTimeoutFinish,
    CollisionFinish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossDeathBurstTimeoutLoopGateLoc3720A {
    pub(crate) hibernate_one_frame: bool,
    pub(crate) next_collision_type: i16,
    pub(crate) next: BossDeathBurstTimeoutLoopNextLoc3720A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossDeathBurstFinalSpawnLoc3724A {
    pub(crate) worker_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossDeathBurstTimeoutGroundGateLoc3724A {
    pub(crate) final_spawn: Option<BossDeathBurstFinalSpawnLoc3724A>,
    pub(crate) finish_animation_label: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossDeathBurstFinishSub3728C {
    pub(crate) animation_label: &'static str,
    pub(crate) hibernate_until_anim_finished: bool,
    pub(crate) delete_current_object: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireDemonFragmentCollisionStep {
    pub(crate) next_collision_type: i16,
    pub(crate) animation_label: Option<&'static str>,
    pub(crate) hibernate_until_anim_finished: bool,
    pub(crate) delete_current_object: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossHeadDeathAnimSequenceLoc39EE2 {
    /// 8 animation scripts played in order, each with hibernate-until-finished.
    pub(crate) animation_scripts: [&'static str; 8],
    /// Direction is flipped (eori.b #$FF) after scripts 0, 2, 4, 6.
    pub(crate) direction_flip_after_scripts: [bool; 8],
    pub(crate) played_sound_effect: &'static str,
    pub(crate) death_spin_counter_d0: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossHeadDeathSpinLoopLoc39F7A {
    /// 4 animation scripts per iteration, each with hibernate-until-finished.
    pub(crate) animation_scripts: [&'static str; 4],
    /// Direction is flipped (eori.b #$FF) after scripts 0 and 2.
    pub(crate) direction_flip_after_scripts: [bool; 4],
    pub(crate) loop_count_d0: u16,
    pub(crate) delete_current_object: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FireDemonFragmentMovementNextLoc3A134 {
    /// $40(a3) == 0: clear x_vel, switch to stru_3A1D6, enter fade loop at loc_3A14E.
    FadeLoopLoc3A14E,
    /// $40(a3) != 0: decrement $40, call sub_3A1A6, loop back to loc_3A134.
    ContinueMovementLoc3A168,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireDemonFragmentMovementStepLoc3A134 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) horizontal_counter_40: u16,
    pub(crate) clears_x_vel: bool,
    pub(crate) fade_animation_label: Option<&'static str>,
    pub(crate) next_horizontal_counter_40: u16,
    pub(crate) runs_collision_check_sub_3a1a6: bool,
    pub(crate) next: FireDemonFragmentMovementNextLoc3A134,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FireDemonFragmentFadeNextLoc3A14E {
    /// $42(a3) == 0: delete the object.
    DeleteCurrentObject,
    /// $42(a3) != 0: decrement $42, call sub_3A172, loop back to loc_3A14E.
    ContinueFadeLoc3A15E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireDemonFragmentFadeStepLoc3A14E {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) fade_counter_42: u16,
    pub(crate) delete_current_object: bool,
    pub(crate) next_fade_counter_42: u16,
    pub(crate) runs_collision_check_sub_3a172: bool,
    pub(crate) next: FireDemonFragmentFadeNextLoc3A14E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FireDemonFragmentDecelNextLoc3A24C {
    /// $40(a3) reached 0 after decrement: j_Delete_CurrentObject.
    DeleteLifetimeExpired,
    /// x_pos(a3) <= 0: j_Delete_CurrentObject.
    DeleteOutOfBoundsLeft,
    /// x_pos(a3) > Level_width_pixels: j_Delete_CurrentObject.
    DeleteOutOfBoundsRight,
    /// Bounds OK: bsr.w sub_3A1A6, bra.s loc_3A24C.
    ContinueDecelLoc3A24C,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireDemonFragmentDecelStepLoc3A24C {
    pub(crate) hibernates_one_frame: bool,
    /// d0 byte persisted from init: nonzero = originally leftward, zero = originally rightward.
    pub(crate) direction_byte_d0: u8,
    /// Signed 32-bit deceleration applied to x_vel: +$7D0 (leftward fragment) or -$7D0 (rightward).
    pub(crate) x_vel_delta: i32,
    /// $40(a3) after subq.w #1.
    pub(crate) next_horizontal_counter_40: u16,
    pub(crate) delete_current_object: bool,
    pub(crate) runs_collision_check_sub_3a1a6: bool,
    pub(crate) next: FireDemonFragmentDecelNextLoc3A24C,
}

/// Result of `sub_3A292` — shared boss collision/palette helper.
///
/// Called as a tail-call by all boss head movement wrappers (`sub_398BC`,
/// `sub_39952`, `sub_399E4`, `sub_39A7C`, `sub_39BF8`, `sub_39D4A`,
/// `sub_39EB4`).  Checks `collision_type(a3)` against accepted weapon IDs,
/// decrements HP on hit, manages the palette-flash timer, and returns
/// whether the head is dead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossCollisionPaletteSub3A292 {
    /// Whether `collision_type(a3)` was consumed (cleared to 0).
    pub(crate) collision_type_cleared: bool,
    /// Whether the head was hit this frame (loc_3A2C0 path taken).
    pub(crate) hit: bool,
    /// Sound effect queued on hit.
    pub(crate) played_sound_effect: Option<&'static str>,
    /// `$3E(a3)` after potential decrement.
    pub(crate) next_hit_points: i16,
    /// Flash timer byte (d1 low byte) after processing.
    pub(crate) next_flash_timer: u8,
    /// `palette_line(a3)` written.
    pub(crate) next_palette_line: u8,
    /// d7 return: 0 = alive, 1 = dead.
    pub(crate) head_dead: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossEyePopEffectInitSub3729E {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) x_direction: u8,
    pub(crate) sprite_mapping_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) sprite_word_y_adjust: u16,
    pub(crate) active_eye_delta: i8,
    pub(crate) entry_animation_label: &'static str,
    pub(crate) entry_hibernate_until_anim_finished: bool,
    pub(crate) chase_frames: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossEyePopEffectFinishSub3729E {
    pub(crate) animation_label: &'static str,
    pub(crate) hibernate_until_anim_finished: bool,
    pub(crate) active_eye_delta: i8,
    pub(crate) delete_current_object: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossEyeAttackInitSub37332 {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_mapping_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) sprite_word_y_adjust: u16,
    pub(crate) initial_x_velocity: i32,
    pub(crate) initial_y_velocity: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossEyeAttackStepSub37332 {
    pub(crate) next_y_velocity: i32,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) delete_current_object: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossEyeSpriteRefreshSlotSub37ABC {
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) sprite_mapping: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossEyeDetachStepSub393B2 {
    pub(crate) sound_effect_label: &'static str,
    pub(crate) next_sprite_word_y_adjust: u16,
    pub(crate) rerank_gfx_object: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossEyeOrbitPlacementSub393C8 {
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) next_sprite_word_y_adjust: u16,
    pub(crate) rerank_gfx_object: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossEyeCounterOrbitPlacementSub39420 {
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) next_sprite_word_y_adjust: u16,
    pub(crate) rerank_gfx_object: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossHeadProjectileSpawnSub397E4 {
    pub(crate) callback_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) x_direction: u8,
    pub(crate) velocity_word_48: i16,
    pub(crate) animation_script_label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossHeadMovementStepSub39846 {
    pub(crate) next_delay_counter: u16,
    pub(crate) next_script_index: usize,
    pub(crate) next_x_velocity: Option<i32>,
    pub(crate) next_y_velocity: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossHeadMovementWrapper {
    pub(crate) movement: BossHeadMovementStepSub39846,
    pub(crate) runs_shared_collision_helper_sub_3_a292: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHeadInitLoc37F30 {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) runs_common_boss_init_loc_32146: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) hit_points: u16,
    pub(crate) sprite_mapping_label: &'static str,
    pub(crate) cached_worker_slot_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) delay_counter_d0: u16,
    pub(crate) script_index_d1: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead1InitLoc39478 {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta_base: u16,
    pub(crate) sets_object_meta_bit_f: bool,
    pub(crate) runs_common_boss_init_loc_32146: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) hit_points: u16,
    pub(crate) sprite_mapping_label: &'static str,
    pub(crate) cached_worker_slot_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) delay_counter_d0: u16,
    pub(crate) script_index_d1: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead1WaitNextLoc394E8 {
    CheckKidWindowLoc39506,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead1WaitLoopLoc394E8 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shishka_movement_sub_398bc: bool,
    pub(crate) runs_bagel_movement_sub_39bf8: bool,
    pub(crate) cached_collision_word_fffffb72: u16,
    pub(crate) next: ShishkaBagelHead1WaitNextLoc394E8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead1ActivationNextLoc39506 {
    WaitLoopLoc394E8,
    AttackAnimationLoc39548,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead1ActivationWindowLoc39506 {
    pub(crate) kid_within_vertical_window: bool,
    pub(crate) x_direction: Option<u8>,
    pub(crate) cached_x_direction_fffffb86: Option<u8>,
    pub(crate) animation_script_label: Option<&'static str>,
    pub(crate) next: ShishkaBagelHead1ActivationNextLoc39506,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead1AttackWaitNextLoc39548 {
    CheckAttackAnimationDoneLoc39566,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead1AttackWaitLoopLoc39548 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shishka_movement_sub_398bc: bool,
    pub(crate) runs_bagel_movement_sub_39bf8: bool,
    pub(crate) cached_collision_word_fffffb72: u16,
    pub(crate) next: ShishkaBagelHead1AttackWaitNextLoc39548,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead1AttackAnimGateNextLoc39566 {
    AttackWaitLoopLoc39548,
    SpawnProjectileSub397E4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead1AttackAnimGateLoc39566 {
    pub(crate) animation_done_byte_18: u8,
    pub(crate) attack_animation_done: bool,
    pub(crate) runs_projectile_spawn_sub_397e4: bool,
    pub(crate) next: ShishkaBagelHead1AttackAnimGateNextLoc39566,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead1PostProjectileWaitNextLoc39570 {
    CheckAnimationDoneLoc3958E,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead1PostProjectileWaitLoopLoc39570 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shishka_movement_sub_398bc: bool,
    pub(crate) runs_bagel_movement_sub_39bf8: bool,
    pub(crate) cached_collision_word_fffffb72: u16,
    pub(crate) next: ShishkaBagelHead1PostProjectileWaitNextLoc39570,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead1PostProjectileAnimGateNextLoc3958E {
    AwaitPostProjectileWaitLoc39570,
    ReturnToActivationWaitLoc394E8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead1PostProjectileAnimGateLoc3958E {
    pub(crate) animation_done_byte_18: u8,
    pub(crate) post_projectile_animation_done: bool,
    pub(crate) next: ShishkaBagelHead1PostProjectileAnimGateNextLoc3958E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead2InitLoc39598 {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta_base: u16,
    pub(crate) sets_object_meta_bit_f: bool,
    pub(crate) runs_common_boss_init_loc_32146: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) initial_field_3e: u16,
    pub(crate) hit_points: u16,
    pub(crate) sprite_mapping_label: &'static str,
    pub(crate) cached_worker_slot_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) delay_counter_d0: u16,
    pub(crate) script_index_d1: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead2WaitNextLoc3960E {
    CheckKidWindowLoc3962C,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead2WaitLoopLoc3960E {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shishka_movement_sub_398bc: bool,
    pub(crate) runs_bagel_movement_sub_39d4a: bool,
    pub(crate) cached_collision_word_fffffb74: u16,
    pub(crate) next: ShishkaBagelHead2WaitNextLoc3960E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead2ActivationNextLoc3962C {
    WaitLoopLoc3960E,
    AttackAnimationLoc3966E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead2ActivationWindowLoc3962C {
    pub(crate) kid_within_vertical_window: bool,
    pub(crate) x_direction: Option<u8>,
    pub(crate) cached_x_direction_fffffb87: Option<u8>,
    pub(crate) animation_script_label: Option<&'static str>,
    pub(crate) next: ShishkaBagelHead2ActivationNextLoc3962C,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead2AttackWaitNextLoc3966E {
    CheckAttackAnimationDoneLoc3968C,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead2AttackWaitLoopLoc3966E {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shishka_movement_sub_398bc: bool,
    pub(crate) runs_bagel_movement_sub_39d4a: bool,
    pub(crate) cached_collision_word_fffffb74: u16,
    pub(crate) next: ShishkaBagelHead2AttackWaitNextLoc3966E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead2AttackAnimGateNextLoc3968C {
    AttackWaitLoopLoc3966E,
    SpawnProjectileSub397E4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead2AttackAnimGateLoc3968C {
    pub(crate) animation_done_byte_18: u8,
    pub(crate) attack_animation_done: bool,
    pub(crate) runs_projectile_spawn_sub_397e4: bool,
    pub(crate) next: ShishkaBagelHead2AttackAnimGateNextLoc3968C,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead2PostProjectileWaitNextLoc39696 {
    CheckAnimationDoneLoc396B4,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead2PostProjectileWaitLoopLoc39696 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shishka_movement_sub_398bc: bool,
    pub(crate) runs_bagel_movement_sub_39d4a: bool,
    pub(crate) cached_collision_word_fffffb74: u16,
    pub(crate) next: ShishkaBagelHead2PostProjectileWaitNextLoc39696,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead2PostProjectileAnimGateNextLoc396B4 {
    AwaitPostProjectileWaitLoc39696,
    ReturnToActivationWaitLoc3960E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead2PostProjectileAnimGateLoc396B4 {
    pub(crate) animation_done_byte_18: u8,
    pub(crate) post_projectile_animation_done: bool,
    pub(crate) next: ShishkaBagelHead2PostProjectileAnimGateNextLoc396B4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead3InitLoc396BE {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta_base: u16,
    pub(crate) sets_object_meta_bit_f: bool,
    pub(crate) runs_common_boss_init_loc_32146: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) initial_field_3e: u16,
    pub(crate) hit_points: u16,
    pub(crate) sprite_mapping_label: &'static str,
    pub(crate) cached_worker_slot_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) delay_counter_d0: u16,
    pub(crate) script_index_d1: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead3WaitNextLoc39734 {
    CheckKidWindowLoc39752,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead3WaitLoopLoc39734 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shishka_movement_sub_398bc: bool,
    pub(crate) runs_bagel_movement_sub_39eb4: bool,
    pub(crate) cached_collision_word_fffffb76: u16,
    pub(crate) next: ShishkaBagelHead3WaitNextLoc39734,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead3ActivationNextLoc39752 {
    WaitLoopLoc39734,
    AttackAnimationLoc39794,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead3ActivationWindowLoc39752 {
    pub(crate) kid_within_vertical_window: bool,
    pub(crate) x_direction: Option<u8>,
    pub(crate) cached_x_direction_fffffb88: Option<u8>,
    pub(crate) animation_script_label: Option<&'static str>,
    pub(crate) next: ShishkaBagelHead3ActivationNextLoc39752,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead3AttackWaitNextLoc39794 {
    CheckAttackAnimationDoneLoc397B2,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead3AttackWaitLoopLoc39794 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shishka_movement_sub_398bc: bool,
    pub(crate) runs_bagel_movement_sub_39eb4: bool,
    pub(crate) cached_collision_word_fffffb76: u16,
    pub(crate) next: ShishkaBagelHead3AttackWaitNextLoc39794,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead3PostProjectileWaitNextLoc397BC {
    CheckAnimationDoneLoc397DA,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead3PostProjectileWaitLoopLoc397BC {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shishka_movement_sub_398bc: bool,
    pub(crate) runs_bagel_movement_sub_39eb4: bool,
    pub(crate) cached_collision_word_fffffb76: u16,
    pub(crate) next: ShishkaBagelHead3PostProjectileWaitNextLoc397BC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShishkaBagelHead3PostProjectileAnimGateNextLoc397DA {
    AwaitPostProjectileWaitLoc397BC,
    ReturnToWaitLoopLoc39734,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShishkaBagelHead3PostProjectileAnimGateLoc397DA {
    pub(crate) animation_done_byte_18: u8,
    pub(crate) post_projectile_animation_done: bool,
    pub(crate) next: ShishkaBagelHead3PostProjectileAnimGateNextLoc397DA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHeadWaitNextLoc37F80 {
    CheckKidWindowLoc37F94,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHeadWaitLoopLoc37F80 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_wrapper_sub_39952: bool,
    pub(crate) cached_collision_word_fffffb72: u16,
    pub(crate) next: BoomerangBossHeadWaitNextLoc37F80,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHeadActivationNextLoc37F94 {
    WaitLoopLoc37F80,
    AttackAnimationLoc37FC6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHeadActivationLoc37F94 {
    pub(crate) kid_within_vertical_window: bool,
    pub(crate) x_direction: Option<u8>,
    pub(crate) cached_x_direction_fffffb86: Option<u8>,
    pub(crate) animation_script_label: Option<&'static str>,
    pub(crate) next: BoomerangBossHeadActivationNextLoc37F94,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHeadAttackWaitNextLoc37FD6 {
    CheckAttackAnimationDoneLoc37FEA,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHeadAttackWaitLoopLoc37FD6 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_wrapper_sub_39952: bool,
    pub(crate) cached_collision_word_fffffb72: u16,
    pub(crate) next: BoomerangBossHeadAttackWaitNextLoc37FD6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHeadAttackAnimationNextLoc37FEA {
    AwaitAnimationLoc37FD6,
    SpawnProjectileSub397E4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHeadAttackAnimationGateLoc37FEA {
    pub(crate) attack_animation_done: bool,
    pub(crate) runs_projectile_spawn_sub_397e4: bool,
    pub(crate) next: BoomerangBossHeadAttackAnimationNextLoc37FEA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHeadPostProjectileWaitNextLoc37FF4 {
    CheckAnimationDone,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHeadPostProjectileWaitLoopLoc37FF4 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_wrapper_sub_39952: bool,
    pub(crate) cached_collision_word_fffffb72: u16,
    pub(crate) next: BoomerangBossHeadPostProjectileWaitNextLoc37FF4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHeadPostProjectileAnimationNextLoc38008 {
    AwaitPostProjectileWaitLoc37FF4,
    ReturnToActivationWaitLoc37F80,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHeadPostProjectileAnimationGateLoc38008 {
    pub(crate) animation_done_byte_18: u8,
    pub(crate) post_projectile_animation_done: bool,
    pub(crate) next: BoomerangBossHeadPostProjectileAnimationNextLoc38008,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2WaitNextLoc38062 {
    CheckKidWindow,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2WaitLoopLoc38062 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_wrapper_sub_399e4: bool,
    pub(crate) cached_collision_word_fffffb74: u16,
    pub(crate) next: BoomerangBossHead2WaitNextLoc38062,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2ActivationNextLoc38076 {
    WaitLoopLoc38062,
    AttackAnimation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2ActivationWindowLoc38076 {
    pub(crate) kid_within_vertical_window: bool,
    pub(crate) x_direction: Option<u8>,
    pub(crate) cached_x_direction_fffffb87: Option<u8>,
    pub(crate) animation_script_label: Option<&'static str>,
    pub(crate) next: BoomerangBossHead2ActivationNextLoc38076,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2AttackWaitNextLoc380B8 {
    CheckAttackAnimationDone,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2AttackWaitLoopLoc380B8 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_wrapper_sub_399e4: bool,
    pub(crate) cached_collision_word_fffffb74: u16,
    pub(crate) next: BoomerangBossHead2AttackWaitNextLoc380B8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2AttackAnimationNextLoc380CC {
    AwaitAttackWaitLoopLoc380B8,
    SpawnProjectileSub397E4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2AttackAnimationGateLoc380CC {
    pub(crate) attack_animation_done: bool,
    pub(crate) runs_projectile_spawn_sub_397e4: bool,
    pub(crate) next: BoomerangBossHead2AttackAnimationNextLoc380CC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2PostProjectileWaitNextLoc380D6 {
    CheckAnimationDone,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2PostProjectileWaitLoopLoc380D6 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_wrapper_sub_399e4: bool,
    pub(crate) cached_collision_word_fffffb74: u16,
    pub(crate) next: BoomerangBossHead2PostProjectileWaitNextLoc380D6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2PostProjectileAnimationNextLoc380EA {
    AwaitPostProjectileWaitLoc380D6,
    ReturnToActivationWaitLoc38062,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2PostProjectileAnimationGateLoc380EA {
    pub(crate) post_projectile_animation_done: bool,
    pub(crate) next: BoomerangBossHead2PostProjectileAnimationNextLoc380EA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3WaitNextLoc38144 {
    CheckKidWindow,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3WaitLoopLoc38144 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_wrapper_sub_39a7c: bool,
    pub(crate) cached_collision_word_fffffb76: u16,
    pub(crate) next: BoomerangBossHead3WaitNextLoc38144,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3ActivationNextLoc38158 {
    WaitLoopLoc38144,
    AttackAnimation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3ActivationWindowLoc38158 {
    pub(crate) kid_within_vertical_window: bool,
    pub(crate) x_direction: Option<u8>,
    pub(crate) cached_x_direction_fffffb88: Option<u8>,
    pub(crate) animation_script_label: Option<&'static str>,
    pub(crate) next: BoomerangBossHead3ActivationNextLoc38158,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3AttackWaitNextLoc3819A {
    CheckAttackAnimationDone,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3AttackWaitLoopLoc3819A {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_wrapper_sub_39a7c: bool,
    pub(crate) cached_collision_word_fffffb76: u16,
    pub(crate) next: BoomerangBossHead3AttackWaitNextLoc3819A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3AttackAnimationNextLoc381AE {
    AwaitAttackWaitLoopLoc3819A,
    SpawnProjectileSub397E4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3AttackAnimationGateLoc381AE {
    pub(crate) attack_animation_done: bool,
    pub(crate) runs_projectile_spawn_sub_397e4: bool,
    pub(crate) next: BoomerangBossHead3AttackAnimationNextLoc381AE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3PostProjectileWaitNextLoc381B8 {
    CheckAnimationDone,
    SharedCollisionExitLoc39EBE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3PostProjectileWaitLoopLoc381B8 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_wrapper_sub_39a7c: bool,
    pub(crate) cached_collision_word_fffffb76: u16,
    pub(crate) next: BoomerangBossHead3PostProjectileWaitNextLoc381B8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3PostProjectileAnimationNextLoc381CC {
    AwaitPostProjectileWaitLoc381B8,
    ReturnToActivationWaitLoc38144,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3PostProjectileAnimationGateLoc381CC {
    pub(crate) post_projectile_animation_done: bool,
    pub(crate) next: BoomerangBossHead3PostProjectileAnimationNextLoc381CC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece1InitLoc38494 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) movement_script_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossSkewerPiece1WaitNextLoc384D6 {
    WaitLoopLoc384D6,
    LaunchLoc384E8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece1WaitLoopLoc384D6 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_step_sub_39846: bool,
    pub(crate) next: BoomerangBossSkewerPiece1WaitNextLoc384D6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece1LaunchLoc384E8 {
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38510: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece2InitLoc38512 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) movement_script_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossSkewerPiece2WaitNextLoc38554 {
    WaitLoopLoc38554,
    LaunchLoc38566,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece2WaitLoopLoc38554 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_step_sub_39846: bool,
    pub(crate) next: BoomerangBossSkewerPiece2WaitNextLoc38554,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece2LaunchLoc38566 {
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_3858e: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece3InitLoc38590 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) movement_script_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossSkewerPiece3WaitNextLoc385D2 {
    WaitLoopLoc385D2,
    LaunchLoc385E4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece3WaitLoopLoc385D2 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_step_sub_39846: bool,
    pub(crate) next: BoomerangBossSkewerPiece3WaitNextLoc385D2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece3LaunchLoc385E4 {
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_3860c: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece4InitLoc3860E {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) movement_script_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossSkewerPiece4WaitNextLoc38650 {
    WaitLoopLoc38650,
    LaunchLoc38662,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece4WaitLoopLoc38650 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_step_sub_39846: bool,
    pub(crate) next: BoomerangBossSkewerPiece4WaitNextLoc38650,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece4LaunchLoc38662 {
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_3868a: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece5InitLoc3868C {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) movement_script_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossSkewerPiece5WaitNextLoc386CE {
    WaitLoopLoc386CE,
    LaunchLoc386E0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece5WaitLoopLoc386CE {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_step_sub_39846: bool,
    pub(crate) next: BoomerangBossSkewerPiece5WaitNextLoc386CE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece5LaunchLoc386E0 {
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38708: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece6InitLoc3870A {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) movement_script_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossSkewerPiece6WaitNextLoc3874C {
    WaitLoopLoc3874C,
    LaunchLoc3875E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece6WaitLoopLoc3874C {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_step_sub_39846: bool,
    pub(crate) next: BoomerangBossSkewerPiece6WaitNextLoc3874C,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece6LaunchLoc3875E {
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38786: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece7InitLoc38788 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) movement_script_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossSkewerPiece7WaitNextLoc387CA {
    WaitLoopLoc387CA,
    LaunchLoc387DC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece7WaitLoopLoc387CA {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_step_sub_39846: bool,
    pub(crate) next: BoomerangBossSkewerPiece7WaitNextLoc387CA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece7LaunchLoc387DC {
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38804: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece8InitLoc38806 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) movement_script_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossSkewerPiece8WaitNextLoc38848 {
    WaitLoopLoc38848,
    LaunchLoc3885A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece8WaitLoopLoc38848 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_step_sub_39846: bool,
    pub(crate) next: BoomerangBossSkewerPiece8WaitNextLoc38848,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece8LaunchLoc3885A {
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38882: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece9InitLoc38884 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) movement_script_label: &'static str,
    pub(crate) movement_script_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossSkewerPiece9WaitNextLoc388C6 {
    WaitLoopLoc388C6,
    LaunchLoc388D8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece9WaitLoopLoc388C6 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_movement_step_sub_39846: bool,
    pub(crate) next: BoomerangBossSkewerPiece9WaitNextLoc388C6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossSkewerPiece9LaunchLoc388D8 {
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38900: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossEyePiece1InitLoc38902 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossEyePiece1TrackingNextLoc3893E {
    TrackingLoopLoc3893E,
    LaunchLoc38962,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossEyePiece1TrackingLoopLoc3893E {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_393c8: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossEyePiece1TrackingNextLoc3893E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossEyePiece1LaunchLoc38962 {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38998: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossEyePiece2InitLoc3899A {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossEyePiece2TrackingNextLoc389D6 {
    TrackingLoopLoc389D6,
    LaunchLoc389FA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossEyePiece2TrackingLoopLoc389D6 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_39420: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossEyePiece2TrackingNextLoc389D6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossEyePiece2LaunchLoc389FA {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38a30: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece1InitLoc38A32 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2EyePiece1TrackingNextLoc38A6E {
    TrackingLoopLoc38A6E,
    LaunchLoc38A92,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece1TrackingLoopLoc38A6E {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_393c8: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead2EyePiece1TrackingNextLoc38A6E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece1LaunchLoc38A92 {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38ac8: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece2InitLoc38ACA {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2EyePiece2TrackingNextLoc38B06 {
    TrackingLoopLoc38B06,
    LaunchLoc38B2A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece2TrackingLoopLoc38B06 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_39420: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead2EyePiece2TrackingNextLoc38B06,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece2LaunchLoc38B2A {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38b60: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece1InitLoc38B62 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3EyePiece1TrackingNextLoc38B9E {
    TrackingLoopLoc38B9E,
    LaunchLoc38BC2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece1TrackingLoopLoc38B9E {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_393c8: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead3EyePiece1TrackingNextLoc38B9E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece1LaunchLoc38BC2 {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38bf8: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece2InitLoc38BFA {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3EyePiece2TrackingNextLoc38C36 {
    TrackingLoopLoc38C36,
    LaunchLoc38C5A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece2TrackingLoopLoc38C36 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_39420: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead3EyePiece2TrackingNextLoc38C36,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece2LaunchLoc38C5A {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38c90: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece3InitLoc38C92 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead1EyePiece3TrackingNextLoc38CCE {
    TrackingLoopLoc38CCE,
    LaunchLoc38CF2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece3TrackingLoopLoc38CCE {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_393c8: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead1EyePiece3TrackingNextLoc38CCE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece3LaunchLoc38CF2 {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38d28: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece4InitLoc38D2A {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead1EyePiece4TrackingNextLoc38D66 {
    TrackingLoopLoc38D66,
    LaunchLoc38D8A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece4TrackingLoopLoc38D66 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_39420: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead1EyePiece4TrackingNextLoc38D66,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece4LaunchLoc38D8A {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38dc0: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece3InitLoc38DC2 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2EyePiece3TrackingNextLoc38DFE {
    TrackingLoopLoc38DFE,
    LaunchLoc38E22,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece3TrackingLoopLoc38DFE {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_393c8: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead2EyePiece3TrackingNextLoc38DFE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece3LaunchLoc38E22 {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38e58: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece4InitLoc38E5A {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2EyePiece4TrackingNextLoc38E96 {
    TrackingLoopLoc38E96,
    LaunchLoc38EBA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece4TrackingLoopLoc38E96 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_39420: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead2EyePiece4TrackingNextLoc38E96,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece4LaunchLoc38EBA {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38ef0: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece3InitLoc38EF2 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3EyePiece3TrackingNextLoc38F2E {
    TrackingLoopLoc38F2E,
    LaunchLoc38F52,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece3TrackingLoopLoc38F2E {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_393c8: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead3EyePiece3TrackingNextLoc38F2E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece3LaunchLoc38F52 {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_38f88: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3EyePiece4TrackingNextLoc38FC6 {
    TrackingLoopLoc38FC6,
    LaunchLoc38FEA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece4TrackingLoopLoc38FC6 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_39420: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead3EyePiece4TrackingNextLoc38FC6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece4LaunchLoc38FEA {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_39020: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece5InitLoc39022 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead1EyePiece5TrackingNextLoc3905E {
    TrackingLoopLoc3905E,
    LaunchLoc39082,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece5TrackingLoopLoc3905E {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_393c8: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead1EyePiece5TrackingNextLoc3905E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece5LaunchLoc39082 {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_390b8: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece6InitLoc390BA {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead1EyePiece6TrackingNextLoc390F6 {
    TrackingLoopLoc390F6,
    LaunchLoc3911A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece6TrackingLoopLoc390F6 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_39420: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead1EyePiece6TrackingNextLoc390F6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead1EyePiece6LaunchLoc3911A {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_39150: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece5InitLoc39152 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2EyePiece5TrackingNextLoc3918E {
    TrackingLoopLoc3918E,
    LaunchLoc391B2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece5TrackingLoopLoc3918E {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_393c8: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead2EyePiece5TrackingNextLoc3918E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece5LaunchLoc391B2 {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_391e8: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece6InitLoc391EA {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead2EyePiece6TrackingNextLoc39226 {
    TrackingLoopLoc39226,
    LaunchLoc3924A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece6TrackingLoopLoc39226 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_39420: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead2EyePiece6TrackingNextLoc39226,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead2EyePiece6LaunchLoc3924A {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_39280: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece5InitLoc39282 {
    pub(crate) gfx_slot: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) has_kid_collision: bool,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) sprite_label: &'static str,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) initial_frame_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3EyePiece5TrackingNextLoc392BE {
    TrackingLoopLoc392BE,
    LaunchLoc392E2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece5TrackingLoopLoc392BE {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_393c8: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead3EyePiece5TrackingNextLoc392BE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece5LaunchLoc392E2 {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_39318: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoomerangBossHead3EyePiece6TrackingNextLoc39356 {
    TrackingLoopLoc39356,
    LaunchLoc3937A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece6TrackingLoopLoc39356 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_orbit_placement_sub_39420: bool,
    pub(crate) runs_directional_sprite_sub_37a22: bool,
    pub(crate) updates_sprite_from_d6: bool,
    pub(crate) next: BoomerangBossHead3EyePiece6TrackingNextLoc39356,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoomerangBossHead3EyePiece6LaunchLoc3937A {
    pub(crate) runs_detach_step_sub_393b2: bool,
    pub(crate) y_vel: i16,
    pub(crate) x_vel: i32,
    pub(crate) gravity_per_frame: i32,
    pub(crate) runs_directional_sprite_during_flight: bool,
    pub(crate) deletes_below_level_height: bool,
    pub(crate) loops_back_loc_393b0: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeadyMetalSpriteInitSub3764A {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) x_direction: u8,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeadyMetalSpriteInitSub37680 {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) x_direction: u8,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeadyMetalSpriteInitSub376B6 {
    pub(crate) gfx_slot_base: u32,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) display_enabled: bool,
    pub(crate) is_moved: bool,
    pub(crate) x_direction: u8,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeadyMetalSpriteVelocitySub376EC {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct HeadyMetalTrackingVelocityLoc37582 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct HeadyMetalSpawnAnimationLoc3761E {
    pub(crate) next_piece_counter: u16,
    pub(crate) selected_animation_label: &'static str,
    pub(crate) runs_tail_update: bool,
    pub(crate) runs_breakup_step_sub_37708: bool,
    pub(crate) runs_palette_cycle_sub_379_e8: bool,
    pub(crate) runs_sprite_refresh_sub_37_abc: bool,
    pub(crate) loops_to_tracking_loc_37582: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NormalSpriteChunkLoc12B8 {
    pub(crate) x_screen_px: i16,
    pub(crate) y_screen_px: i16,
    pub(crate) size_code: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpriteDmaUploadLoc141A {
    pub(crate) sprite_tile_word: u16,
    pub(crate) dma_tile_count: u16,
    pub(crate) next_sprite_tile_word: u16,
    pub(crate) next_hardware_tile_word: u16,
    pub(crate) dma_source: u32,
    pub(crate) next_dma_source: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeadyMetalScatterVelocities {
    pub(crate) slot_fb72_x_velocity: i32,
    pub(crate) slot_fb76_x_velocity: i32,
    pub(crate) slot_fb7a_y_velocity: i32,
    pub(crate) slot_fbae_x_velocity: i32,
    pub(crate) slot_fbb2_x_velocity: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeadyMetalBreakupPieceSub37708 {
    pub(crate) sprite_word_y: i16,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeadyMetalScatterPatternSub37708 {
    Primary,
    Alternate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeadyMetalScatterStepSub37708 {
    pub(crate) pattern: HeadyMetalScatterPatternSub37708,
    pub(crate) d6_x_velocity_delta: i32,
    pub(crate) d7_velocity_delta: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeadyMetalBreakupStepSub37708 {
    pub(crate) next_collision_type: i16,
    pub(crate) next_phase_counter: u16,
    pub(crate) next_remaining_scatter_frames: u16,
    pub(crate) piece_slots: [Option<HeadyMetalBreakupPieceSub37708>; 13],
    pub(crate) played_sound_effect: Option<&'static str>,
    pub(crate) spawned_pop_effect_at: Option<(u16, u16)>,
    pub(crate) spawned_attack_at: Option<(u16, u16)>,
    pub(crate) set_has_kid_collision: bool,
    pub(crate) set_global_hit_latch: bool,
    pub(crate) reveal_anchor_worker: bool,
    pub(crate) clear_anchor_hidden_flag: bool,
    pub(crate) reveal_linked_worker: bool,
    pub(crate) trigger_cleanup_sub_378c8: bool,
    pub(crate) scatter_step: Option<HeadyMetalScatterStepSub37708>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeadyMetalCleanupSub378C8 {
    pub(crate) next_countdown_42: u16,
    pub(crate) should_cleanup_workers: bool,
    pub(crate) current_x_velocity_delta: i32,
    pub(crate) left_worker_x_velocity_delta: i32,
    pub(crate) centre_worker_y_velocity_delta: i32,
    pub(crate) centre_worker_sprite_label: Option<&'static str>,
    pub(crate) played_sound_effect: Option<&'static str>,
    pub(crate) right_worker_x_velocity_delta: i32,
    pub(crate) far_right_worker_x_velocity_delta: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BossEyeAttackFrameInputsSub37332 {
    pub(crate) effect_x: u16,
    pub(crate) effect_y: u16,
    pub(crate) kid_x: u16,
    pub(crate) kid_y: u16,
    pub(crate) diamond_power_active: bool,
    pub(crate) v_int_counter_fff809: u8,
    pub(crate) level_height_pixels: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeadyMetalPaletteCycleSub379E8 {
    pub(crate) next_phase_counter: u16,
    pub(crate) palette_word: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_death_burst_init_sub_370D2(
    x_pos: u16,
    y_pos: u16,
) -> BossDeathBurstInitSub370D2 {
    // ASM sub_370D2 (kid.asm:38972): play `sfx_Voice_die`, allocate the same
    // burst sprite slot as sub_37002, start animation `stru_37BD8`, then fall
    // with `$2710` gravity until grounded before entering the slower homing
    // loop.
    BossDeathBurstInitSub370D2 {
        sound_effect_label: "sfx_Voice_die",
        gfx_slot_base: 0x0101_0002,
        priority: 0,
        object_meta: 0x0020,
        display_enabled: true,
        is_moved: true,
        x_direction: 0,
        sprite_mapping_label: "LnkTo_unk_C8460-Data_Index",
        x_pos,
        y_pos,
        sprite_word_y_adjust: 3,
        chase_frames: 0x0078,
        landing_gravity: 0x0000_2710,
        homing_accel: 0x0000_07D0,
        entry_animation_label: "stru_37BD8",
        collision_animation_label: "stru_37BC6",
        timeout_animation_label: "stru_37C0E",
        delete_collision_types: [0x001C, 0x002C, -1],
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_death_burst_fall_step_sub_370D2(y_velocity: i32) -> i32 {
    // ASM loc_3712E (kid.asm:38995): after each hibernate, add `$2710` to
    // `y_vel(a3)` until the grounded byte `$18(a3)` becomes non-zero.
    y_velocity.saturating_add(0x0000_2710)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_death_burst_loop_gate_loc_37148(
    collision_type: i16,
    chase_frames: u16,
) -> BossDeathBurstLoopGateLoc37148 {
    // ASM loc_37148 (kid.asm:39002): after each grounded hibernate, branch to
    // the collision finish on `$001C`, `$002C`, or `$FFFF`; otherwise clear
    // `collision_type(a3)`, decrement `d0`, and chase Kid while the timer lives.
    let collided = matches!(collision_type, 0x001C | 0x002C | -1);
    if collided {
        return BossDeathBurstLoopGateLoc37148 {
            next_collision_type: collision_type,
            next_chase_frames: chase_frames,
            next: BossDeathBurstLoopNextLoc37148::CollisionFinish,
        };
    }

    let next_chase_frames = chase_frames.wrapping_sub(1);
    let next = if next_chase_frames == 0 {
        BossDeathBurstLoopNextLoc37148::TimeoutFinish
    } else {
        BossDeathBurstLoopNextLoc37148::ChaseKid
    };

    BossDeathBurstLoopGateLoc37148 {
        next_collision_type: 0,
        next_chase_frames,
        next,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_death_burst_collision_finish_loc_371B6() -> BossDeathBurstCollisionFinishLoc371B6
{
    // ASM loc_371B6 (kid.asm:39048): clear `collision_type(a3)`, both velocity
    // longwords, play `stru_37BC6`, hibernate until that animation finishes,
    // then jump to `Delete_CurrentObject`.
    BossDeathBurstCollisionFinishLoc371B6 {
        collision_type: 0,
        x_velocity: 0,
        y_velocity: 0,
        animation_label: "stru_37BC6",
        hibernate_until_anim_finished: true,
        delete_current_object: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_death_burst_homing_step_sub_370D2(
    effect_x: u16,
    effect_y: u16,
    kid_x: u16,
    kid_y: u16,
) -> BossDeathBurstHomingStepSub370D2 {
    // ASM loc_3716C..loc_371B4 (kid.asm:39017): same chase logic as sub_37002
    // but with the slower `$7D0` per-axis acceleration.
    let homing_accel = 0x0000_07D0;
    let (x_direction, x_velocity_delta) = if effect_x > kid_x {
        // ASM loc_37192 (kid.asm:39030): burst right of Kid clears
        // `x_direction(a3)` and subtracts `$7D0` from `x_vel(a3)`.
        (0, -homing_accel)
    } else {
        (0xFF, homing_accel)
    };
    // ASM loc_3719E (kid.asm:39034): compare `y_pos(a3)` with Kid's `$1E`;
    // add `$7D0` when the burst is above or level with Kid, subtract otherwise.
    let y_velocity_delta = if effect_y > kid_y {
        // ASM loc_371AC (kid.asm:39041): burst below Kid subtracts `$7D0`
        // from `y_vel(a3)`.
        -homing_accel
    } else {
        homing_accel
    };

    BossDeathBurstHomingStepSub370D2 {
        x_direction,
        x_velocity_delta,
        y_velocity_delta,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_death_burst_timeout_loop_gate_loc_3720A(
    collision_type: i16,
) -> BossDeathBurstTimeoutLoopGateLoc3720A {
    // ASM loc_3720A (kid.asm:39081): after `Hibernate_Object_1Frame`, branch
    // to loc_371B6 on collision ids `$001C`, `$002C`, or `$FFFF`; otherwise
    // clear `collision_type(a3)` before comparing against Kid and ground state.
    if matches!(collision_type, 0x001C | 0x002C | -1) {
        return BossDeathBurstTimeoutLoopGateLoc3720A {
            hibernate_one_frame: true,
            next_collision_type: collision_type,
            next: BossDeathBurstTimeoutLoopNextLoc3720A::CollisionFinish,
        };
    }

    BossDeathBurstTimeoutLoopGateLoc3720A {
        hibernate_one_frame: true,
        next_collision_type: 0,
        next: BossDeathBurstTimeoutLoopNextLoc3720A::ContinueTimeoutFinish,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_death_burst_timeout_face_kid_loc_37246(effect_x: u16, kid_x: u16) -> u8 {
    // ASM loc_3720A/loc_37246 (kid.asm:39094-39103): `cmp.w d5,d3; bgt.s`
    // only clears `x_direction(a3)` at loc_37246 when the burst is strictly
    // right of Kid. Equal X falls through the `st x_direction(a3)` path.
    if effect_x > kid_x { 0 } else { 0xFF }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_death_burst_timeout_ground_gate_loc_3724A(
    grounded: bool,
    x_direction: u8,
    x_pos: u16,
    y_pos: u16,
) -> BossDeathBurstTimeoutGroundGateLoc3724A {
    // ASM loc_3724A (kid.asm:39105): loop back to loc_3720A until `$18(a3)`
    // is non-zero. Once grounded, allocate a `sub_37002` helper at `y_pos+8`
    // and at `x_pos-5` for clear `x_direction`. ASM loc_37280 is the
    // non-zero `x_direction` tail that stores `x_pos+5` before sub_3728C.
    if !grounded {
        return BossDeathBurstTimeoutGroundGateLoc3724A {
            final_spawn: None,
            finish_animation_label: None,
        };
    }

    let x_offset = if x_direction == 0 { -5 } else { 5 };
    let x_pos = x_pos.wrapping_add_signed(x_offset);
    let y_pos = y_pos.wrapping_add(8);

    BossDeathBurstTimeoutGroundGateLoc3724A {
        final_spawn: Some(BossDeathBurstFinalSpawnLoc3724A {
            worker_label: "sub_37002",
            x_pos,
            y_pos,
        }),
        finish_animation_label: Some("stru_37BFC"),
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_death_burst_finish_sub_3728C() -> BossDeathBurstFinishSub3728C {
    // ASM sub_3728C (kid.asm:39131): switch to `stru_37BFC`, hibernate until
    // that animation ends, then delete the current object.
    BossDeathBurstFinishSub3728C {
        animation_label: "stru_37BFC",
        hibernate_until_anim_finished: true,
        delete_current_object: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_eye_pop_effect_init_sub_3729E(
    x_pos: u16,
    y_pos: u16,
) -> BossEyePopEffectInitSub3729E {
    // ASM sub_3729E (kid.asm:39142): allocate gfx slot `$01010002`, seed
    // sprite mapping `LnkTo_unk_C8460-Data_Index`, bump `$FFFFFB4F`, play
    // `stru_37BEA`, then enter ASM loc_372F6's 100-frame hibernated
    // facing loop.
    BossEyePopEffectInitSub3729E {
        gfx_slot_base: 0x0101_0002,
        priority: 0,
        object_meta: 0x0020,
        display_enabled: true,
        is_moved: true,
        x_direction: 0,
        sprite_mapping_label: "LnkTo_unk_C8460-Data_Index",
        x_pos,
        y_pos,
        sprite_word_y_adjust: 1,
        active_eye_delta: 1,
        entry_animation_label: "stru_37BEA",
        entry_hibernate_until_anim_finished: true,
        chase_frames: 0x0064,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_eye_pop_effect_face_kid_sub_3729E(effect_x: u16, kid_x: u16) -> u8 {
    // ASM loc_37300..loc_3731A (kid.asm:39170): if the effect is to the right
    // of Kid, ASM loc_37316 clears `x_direction(a3)`; otherwise loc_37300
    // sets it to `$FF`.
    if effect_x > kid_x { 0 } else { 0xFF }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_eye_pop_effect_finish_sub_3729E() -> BossEyePopEffectFinishSub3729E {
    // ASM loc_3731C (kid.asm:39187): play `stru_37BFC`, hibernate until the
    // animation ends, decrement `$FFFFFB4F`, then delete the object.
    BossEyePopEffectFinishSub3729E {
        animation_label: "stru_37BFC",
        hibernate_until_anim_finished: true,
        active_eye_delta: -1,
        delete_current_object: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_eye_attack_init_sub_37332(
    x_pos: u16,
    y_pos: u16,
    kid_x: u16,
) -> BossEyeAttackInitSub37332 {
    // ASM sub_37332 (kid.asm:39199): allocate gfx slot `$01010002`, seed the
    // boss-eye projectile sprite, set `y_vel=-4`, and launch left with
    // `$FFFF8001` or right with `$00007FFF` based on Kid's X position.
    let initial_x_velocity = if x_pos < kid_x {
        -0x0000_7FFF
    } else {
        // ASM loc_37394 (kid.asm:39224): `move.l #$7FFF,x_vel(a3)`.
        0x0000_7FFF
    };

    BossEyeAttackInitSub37332 {
        gfx_slot_base: 0x0101_0002,
        priority: 0,
        object_meta: 0x0021,
        has_kid_collision: true,
        display_enabled: true,
        is_moved: true,
        sprite_mapping_label: "LnkTo_unk_C8488-Data_Index",
        x_pos,
        y_pos,
        sprite_word_y_adjust: 2,
        initial_x_velocity,
        initial_y_velocity: -4,
    }
}

fn i16_word(value: u16) -> i16 {
    i16::from_be_bytes(value.to_be_bytes())
}

fn word_lt_signed(lhs: u16, rhs: u16) -> bool {
    i16_word(lhs) < i16_word(rhs)
}

fn word_gt_signed(lhs: u16, rhs: u16) -> bool {
    i16_word(lhs) > i16_word(rhs)
}

const BOSS_EYE_ATTACK_SPRITES_OFF_37AE0: [&str; 8] = [
    "LnkTo_unk_C8488-Data_Index",
    "LnkTo_unk_C8490-Data_Index",
    "LnkTo_unk_C8498-Data_Index",
    "LnkTo_unk_C84A0-Data_Index",
    "LnkTo_unk_C84A8-Data_Index",
    "LnkTo_unk_C84B0-Data_Index",
    "LnkTo_unk_C84B8-Data_Index",
    "LnkTo_unk_C84C0-Data_Index",
];

#[allow(non_snake_case)]
pub(crate) fn boss_eye_attack_sprite_mapping_loc_37B7C(table_index: usize) -> &'static str {
    // ASM loc_37B7C (kid.asm:40106): copy selected `d2` to `d7`, double it
    // as a word offset, then load the mapping word from `off_37AE0`.
    BOSS_EYE_ATTACK_SPRITES_OFF_37AE0[table_index]
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_eye_attack_sprite_sub_37332(
    diamond_power_active: bool,
    v_int_counter_fff809: u8,
    effect_x: u16,
    effect_y: u16,
    kid_x: u16,
    kid_y: u16,
) -> &'static str {
    // ASM sub_37AF0 via sub_37332 (kid.asm:40023): diamond power overrides the
    // projectile sprite to `$E20/$E30`; otherwise choose one of eight compass
    // mappings around the projectile.
    if diamond_power_active {
        return if v_int_counter_fff809 & 8 != 0 {
            "$0E20"
        } else {
            // ASM loc_37B06 (kid.asm:40033): active diamond power with
            // `$FFFFF809 & #8 == 0` returns the alternate `$E30` sprite word.
            "$0E30"
        };
    }

    // ASM loc_37B0C (kid.asm:40040): the non-diamond path snapshots Kid's
    // gfx-object word coordinates and branches with signed `cmp.w` tests
    // against `y_pos(a3)+4`, then `y_pos(a3)-4`.
    if word_gt_signed(kid_y, effect_y.wrapping_add(4)) {
        // ASM loc_37B46 (kid.asm:40066): Kid below `y_pos(a3)+4` enters the
        // lower band, where `x_pos(a3)-4..x_pos(a3)+4` selects table index 4.
        if word_lt_signed(kid_x, effect_x.wrapping_sub(4)) {
            // ASM loc_37B5A (kid.asm:40077): lower-band Kid X left of
            // `x_pos(a3)-4` selects table index 3.
            boss_eye_attack_sprite_mapping_loc_37B7C(3)
        } else if word_gt_signed(kid_x, effect_x.wrapping_add(4)) {
            // ASM loc_37B5E (kid.asm:40082): lower-band Kid X right of
            // `x_pos(a3)+4` selects table index 5.
            boss_eye_attack_sprite_mapping_loc_37B7C(5)
        } else {
            boss_eye_attack_sprite_mapping_loc_37B7C(4)
        }
    } else if word_lt_signed(kid_y, effect_y.wrapping_sub(4)) {
        // ASM loc_37B62 (kid.asm:40087): Kid above `y_pos(a3)-4` enters the
        // upper band, where `x_pos(a3)-4..x_pos(a3)+4` selects table index 0.
        if word_lt_signed(kid_x, effect_x.wrapping_sub(4)) {
            // ASM loc_37B76 (kid.asm:40098): upper-band Kid X left of
            // `x_pos(a3)-4` selects table index 1.
            boss_eye_attack_sprite_mapping_loc_37B7C(1)
        } else if word_gt_signed(kid_x, effect_x.wrapping_add(4)) {
            // ASM loc_37B7A (kid.asm:40103): upper-band Kid X right of
            // `x_pos(a3)+4` selects table index 7.
            boss_eye_attack_sprite_mapping_loc_37B7C(7)
        } else {
            boss_eye_attack_sprite_mapping_loc_37B7C(0)
        }
    } else if word_lt_signed(kid_x, effect_x.wrapping_sub(8)) {
        // ASM loc_37B42 (kid.asm:40061): middle vertical band plus signed
        // Kid X left of `x_pos(a3)-8` selects table index 2.
        boss_eye_attack_sprite_mapping_loc_37B7C(2)
    } else {
        boss_eye_attack_sprite_mapping_loc_37B7C(6)
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_eye_attack_step_sub_37332(
    y_velocity: i32,
    inputs: BossEyeAttackFrameInputsSub37332,
) -> BossEyeAttackStepSub37332 {
    // ASM loc_3739C..loc_373BE (kid.asm:39227): after each hibernate add
    // `$BB8` gravity, refresh the sprite mapping via sub_37AF0, and delete the
    // projectile once `y_pos(a3)` is strictly past the level height.
    BossEyeAttackStepSub37332 {
        next_y_velocity: y_velocity.saturating_add(0x0000_0BB8),
        sprite_mapping: boss_eye_attack_sprite_sub_37332(
            inputs.diamond_power_active,
            inputs.v_int_counter_fff809,
            inputs.effect_x,
            inputs.effect_y,
            inputs.kid_x,
            inputs.kid_y,
        ),
        delete_current_object: inputs.effect_y > inputs.level_height_pixels,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_eye_refresh_attack_sprites_sub_37ABC(
    mut slots: [Option<BossEyeSpriteRefreshSlotSub37ABC>; 12],
    diamond_power_active: bool,
    v_int_counter_fff809: u8,
    kid_x: u16,
    kid_y: u16,
) -> [Option<BossEyeSpriteRefreshSlotSub37ABC>; 12] {
    // ASM loc_37AC6 (kid.asm:39996): load each longword worker pointer from
    // `$FFFFFB7E`; live entries become `a3`, call `sub_37AF0`, then write the
    // returned mapping to `addroffset_sprite(a3)`.
    // ASM loc_37AD4 (kid.asm:40003): null pointers branch straight to the
    // `addq.w #4,a0`/`dbf` loop tail, so sparse tables still scan all 12 slots.
    for slot in &mut slots {
        if let Some(worker) = slot.as_mut() {
            worker.sprite_mapping = boss_eye_attack_sprite_sub_37332(
                diamond_power_active,
                v_int_counter_fff809,
                worker.x_pos,
                worker.y_pos,
                kid_x,
                kid_y,
            );
        }
    }

    slots
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_eye_detach_step_sub_393B2(
    sprite_word_y_adjust: u16,
) -> BossEyeDetachStepSub393B2 {
    // ASM sub_393B2 (kid.asm:41947): preserve `d0`, play
    // `sfx_Boss_eye_pops`, add 4 to `$A(a3)`, then run sub_FF6 so the gfx
    // object list is re-sorted around the new sprite-word Y offset.
    BossEyeDetachStepSub393B2 {
        sound_effect_label: "sfx_Boss_eye_pops",
        next_sprite_word_y_adjust: sprite_word_y_adjust.wrapping_add(4),
        rerank_gfx_object: true,
    }
}

const BOSS_EYE_ORBIT_X_OFFSETS_LEFT_SUB_393C8: [i16; 5] = [-12, 0, 13, 13, 13];
const BOSS_EYE_ORBIT_X_OFFSETS_RIGHT_SUB_393C8: [i16; 5] = [-12, -20, -24, -24, -24];
const BOSS_EYE_COUNTER_ORBIT_X_OFFSETS_LEFT_SUB_39420: [i16; 5] = [12, 20, 24, 24, 24];
const BOSS_EYE_COUNTER_ORBIT_X_OFFSETS_RIGHT_SUB_39420: [i16; 5] = [12, 0, -13, -13, -13];

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_eye_orbit_placement_sub_393C8(
    parent_x: u16,
    parent_y: u16,
    parent_phase_word_22: u16,
    side_flag_d2: u8,
    previous_parent_frame_d0: u8,
    current_parent_frame_16: u8,
    sprite_word_y_adjust: u16,
) -> BossEyeOrbitPlacementSub393C8 {
    // ASM sub_393C8 (kid.asm:41961): derive the five-entry table index from
    // `$22(a2)`, place the orbit child at `$1A/$1E(a2)` with a fixed `-$25`
    // Y shift, then nudge `$A(a3)` by `+1/-1` and run sub_FF6 when `$16(a2)`
    // changed since the last frame tracked in `d0`.
    let phase_index = usize::from(parent_phase_word_22.saturating_sub(0x0EE0) >> 1);
    let x_offset = if side_flag_d2 == 0 {
        BOSS_EYE_ORBIT_X_OFFSETS_RIGHT_SUB_393C8[phase_index]
    } else {
        BOSS_EYE_ORBIT_X_OFFSETS_LEFT_SUB_393C8[phase_index]
    };

    let (next_sprite_word_y_adjust, rerank_gfx_object) =
        if previous_parent_frame_d0 == current_parent_frame_16 {
            (sprite_word_y_adjust, false)
        } else if previous_parent_frame_d0 == 0 {
            // ASM loc_39402 (kid.asm:41983): a zero previous frame in `d0`
            // takes `addq.w #1,$A(a3)` before falling through to `loc_39406`.
            (sprite_word_y_adjust.wrapping_add(1), true)
        } else {
            (sprite_word_y_adjust.wrapping_sub(1), true)
        };

    // ASM loc_393DE (kid.asm:41970): add.w $1A(a2),d7 → x_pos; move.w $1E(a2)
    // → y_pos; subi.w #$25 → y_pos; cmp.b $16(a2),d0 gates frame-change path.
    // ASM loc_39406 (kid.asm:41986): jsr (j_sub_FF6).w — convergence point
    // reached by both the subq.w (d0!=0) and addq.w (d0==0) frame-change paths.
    BossEyeOrbitPlacementSub393C8 {
        x_pos: parent_x.wrapping_add_signed(x_offset),
        y_pos: parent_y.wrapping_sub(0x0025),
        next_sprite_word_y_adjust,
        rerank_gfx_object,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_eye_counter_orbit_placement_sub_39420(
    parent_x: u16,
    parent_y: u16,
    parent_phase_word_22: u16,
    side_flag_d2: u8,
    previous_parent_frame_d0: u8,
    current_parent_frame_16: u8,
    sprite_word_y_adjust: u16,
) -> BossEyeCounterOrbitPlacementSub39420 {
    // ASM sub_39420 (kid.asm:42008): use the mirrored five-entry X-offset
    // tables rooted at `word_39464/word_3946E`, keep the child at `$1E(a2)-$25`,
    // then nudge `$A(a3)` by `-1/+1` and rerun sub_FF6 when the parent's `$16(a2)`
    // frame byte changed since the previous `d0`.
    let phase_index = usize::from(parent_phase_word_22.saturating_sub(0x0EE0) >> 1);
    let x_offset = if side_flag_d2 == 0 {
        BOSS_EYE_COUNTER_ORBIT_X_OFFSETS_RIGHT_SUB_39420[phase_index]
    } else {
        BOSS_EYE_COUNTER_ORBIT_X_OFFSETS_LEFT_SUB_39420[phase_index]
    };

    let (next_sprite_word_y_adjust, rerank_gfx_object) =
        if previous_parent_frame_d0 == current_parent_frame_16 {
            (sprite_word_y_adjust, false)
        } else if previous_parent_frame_d0 == 0 {
            (sprite_word_y_adjust.wrapping_sub(1), true)
        } else {
            // ASM loc_3945A (kid.asm:42030): addq.w #1,$A(a3) — when d0 is nonzero
            // and the parent's frame byte $16(a2) has changed, increment the sprite
            // word then fall through to loc_3945E → jsr (j_sub_FF6).w for reranking.
            (sprite_word_y_adjust.wrapping_add(1), true)
        };

    // ASM loc_39436 (kid.asm:42017): add.w $1A(a2),d7 → x_pos; move.w $1E(a2)
    // → y_pos; subi.w #$25 → y_pos; cmp.b $16(a2),d0 gates frame-change path.
    // Mirrors loc_393DE in sub_393C8 but with swapped +1/-1 adjustment sense.
    BossEyeCounterOrbitPlacementSub39420 {
        x_pos: parent_x.wrapping_add_signed(x_offset),
        y_pos: parent_y.wrapping_sub(0x0025),
        next_sprite_word_y_adjust,
        rerank_gfx_object,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_projectile_spawn_sub_397E4(
    parent_x: u16,
    parent_y: u16,
    parent_x_direction: u8,
    variant_d1: u16,
) -> BossHeadProjectileSpawnSub397E4 {
    // ASM sub_397E4 (kid.asm:42436): allocate a fresh object slot, choose
    // `loc_3A0D2` vs `loc_3A1EA` from `d1`, copy the parent's `$1E/$49`,
    // then spawn the child 24px to the facing side with `$48(a0)=±3` before
    // starting `stru_39FD8`.
    let callback_label = if variant_d1 == 0 {
        "loc_3A0D2"
    } else {
        // ASM loc_397FA (kid.asm:42445): nonzero `d1` selects
        // `loc_3A1EA` for the child object's callback at `4(a0)`.
        "loc_3A1EA"
    };

    // ASM loc_3980A (kid.asm:42450): copy parent X/Y and x_direction, then
    // use the clear x_direction path for `$48(a0)=$FFFD` and X minus $18.
    let (x_pos, velocity_word_48) = if parent_x_direction == 0 {
        (parent_x.wrapping_sub(0x0018), -3)
    } else {
        // ASM loc_3982C (kid.asm:42461): set `$48(a0)=3` and place the
        // child 24px to the right when the parent x_direction byte is set.
        (parent_x.wrapping_add(0x0018), 3)
    };

    // ASM loc_39836 (kid.asm:42465): commit prepared X to `$44(a0)`, load
    // `d7=#stru_39FD8`, and call `j_Init_Animation` before returning.
    BossHeadProjectileSpawnSub397E4 {
        callback_label,
        x_pos,
        y_pos: parent_y,
        x_direction: parent_x_direction,
        velocity_word_48,
        animation_script_label: "stru_39FD8",
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_init_loc_37F30(
    anchor_x_44: u16,
    anchor_y_46: u16,
) -> BoomerangBossHeadInitLoc37F30 {
    // ASM loc_37F30 (kid.asm:40325): allocate gfx slot `$01040002`, seed the
    // first BoomerangBoss head from the anchor's `$44/$46`, cache it in
    // `$FFFFFB7A`, then enter the `unk_398F0` movement script at index 1.
    BoomerangBossHeadInitLoc37F30 {
        gfx_slot_base: 0x0104_0002,
        priority: 1,
        object_meta: 0x0023,
        runs_common_boss_init_loc_32146: true,
        has_kid_collision: false,
        display_enabled: true,
        is_moved: true,
        hit_points: 30,
        sprite_mapping_label: "LnkTo_unk_C8600-Data_Index",
        cached_worker_slot_label: "$FFFFFB7A",
        x_pos: anchor_x_44,
        y_pos: anchor_y_46,
        movement_script_label: "unk_398F0",
        delay_counter_d0: 1,
        script_index_d1: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_1_init_loc_39478(
    anchor_x_44: u16,
    anchor_y_46: u16,
    variant_d2: u16,
) -> ShishkaBagelHead1InitLoc39478 {
    // ASM loc_39478 (kid.asm:42057): shared ShishkaBoss/BagelBrothers head 1 init.
    // Allocates gfx slot $01040002, priority 0. Selects object_meta $22 (d2==0,
    // ShishkaBoss) or $24 (d2!=0, BagelBrothers) then bset #7 on the stored word.
    //
    // ASM loc_39494 (kid.asm:42066): fallthrough after object_meta selection.
    // Stores d0 into object_meta(a3), bset #7, jsr loc_32146, sf has_kid_collision,
    // st $13, st is_moved, LnkTo_unk_C8600 sprite, cache a3 at $FFFFFB7A,
    // copy anchor $44/$46, then branch on d2: unk_39872 + ShishkaBoss HP ($14)
    // when d2==0, unk_39AB2 + BagelBrothers HP ($1E) when d2!=0.
    let (object_meta_base, hit_points, movement_script_label) = if variant_d2 == 0 {
        // ASM loc_394E2 (kid.asm:42088): the d2==0 ShishkaBoss path keeps
        // `unk_39872` and writes ShishkaBoss_HitPointsPerHead ($14) to `$3E(a3)`.
        (0x0022_u16, 0x0014_u16, "unk_39872")
    } else {
        (0x0024, 0x001E, "unk_39AB2")
    };

    ShishkaBagelHead1InitLoc39478 {
        gfx_slot_base: 0x0104_0002,
        priority: 0,
        object_meta_base,
        sets_object_meta_bit_f: true,
        runs_common_boss_init_loc_32146: true,
        has_kid_collision: false,
        display_enabled: true,
        is_moved: true,
        hit_points,
        sprite_mapping_label: "LnkTo_unk_C8600-Data_Index",
        cached_worker_slot_label: "$FFFFFB7A",
        x_pos: anchor_x_44,
        y_pos: anchor_y_46,
        movement_script_label,
        delay_counter_d0: 1,
        script_index_d1: 0,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_2_init_loc_39598(
    anchor_x_44: u16,
    anchor_y_46: u16,
    variant_d2: u16,
) -> ShishkaBagelHead2InitLoc39598 {
    // ASM loc_39598 (kid.asm:42182): head 2 init for ShishkaBoss/BagelBrothers.
    // Allocates gfx slot $01040002, priority 0. Selects object_meta $22 (d2==0,
    // ShishkaBoss) or $24 (d2!=0, BagelBrothers) then falls through to loc_395B4.
    //
    // ASM loc_395B4 (kid.asm:42191): head 2 setup fallthrough — stores d0 into
    // object_meta(a3), bset #7, jsr loc_32146, sf has_kid_collision, st $13,
    // st is_moved, move.w #$1E,$3E(a3), LnkTo_unk_C8600 sprite, cache a3 at
    // $FFFFFB7E, copy anchor $44/$46. Then branch on d2: unk_39872 +
    // ShishkaBoss_HitPointsPerHead ($14) when d2==0, unk_39C2E +
    // BagelBrothers_HitPointsPerHead ($1E) when d2!=0.
    //
    // ASM loc_39608 (kid.asm:42214): ShishkaBoss HP path — move.w
    // #ShishkaBoss_HitPointsPerHead,$3E(a3), overwriting the initial $1E.
    let (object_meta_base, hit_points, movement_script_label) = if variant_d2 == 0 {
        (0x0022_u16, 0x0014_u16, "unk_39872")
    } else {
        (0x0024, 0x001E, "unk_39C2E")
    };

    ShishkaBagelHead2InitLoc39598 {
        gfx_slot_base: 0x0104_0002,
        priority: 0,
        object_meta_base,
        sets_object_meta_bit_f: true,
        runs_common_boss_init_loc_32146: true,
        has_kid_collision: false,
        display_enabled: true,
        is_moved: true,
        initial_field_3e: 0x001E,
        hit_points,
        sprite_mapping_label: "LnkTo_unk_C8600-Data_Index",
        cached_worker_slot_label: "$FFFFFB7E",
        x_pos: anchor_x_44,
        y_pos: anchor_y_46,
        movement_script_label,
        delay_counter_d0: 1,
        script_index_d1: 0,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_2_wait_loop_loc_3960E(
    variant_d2: u16,
    movement_status_d7: u16,
) -> ShishkaBagelHead2WaitLoopLoc3960E {
    // ASM loc_3960E (kid.asm:42217): head 2 wait loop — hibernate one frame,
    // then branch on d2: d2==0 calls sub_398BC (ShishkaBoss movement),
    // d2!=0 falls through to loc_3961C → bsr.w sub_39D4A (BagelBrothers movement).
    //
    // ASM loc_3961C (kid.asm:42225): BagelBrothers path — bsr.w sub_39D4A.
    //
    // ASM loc_39620 (kid.asm:42228): convergence point — move.w d7,($FFFFFB74).w,
    // tst.w d7, beq.s loc_3962C (activation window check),
    // bra.w loc_39EBE (shared collision exit) when d7 nonzero.
    let is_shishka = variant_d2 == 0;
    ShishkaBagelHead2WaitLoopLoc3960E {
        hibernates_one_frame: true,
        runs_shishka_movement_sub_398bc: is_shishka,
        runs_bagel_movement_sub_39d4a: !is_shishka,
        cached_collision_word_fffffb74: movement_status_d7,
        next: if movement_status_d7 == 0 {
            ShishkaBagelHead2WaitNextLoc3960E::CheckKidWindowLoc3962C
        } else {
            ShishkaBagelHead2WaitNextLoc3960E::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_2_activation_window_loc_3962C(
    kid_x: u16,
    kid_y: u16,
    head_x: u16,
    head_y: u16,
) -> ShishkaBagelHead2ActivationWindowLoc3962C {
    // ASM loc_3962C (kid.asm:42235): load Kid gfx coordinates from
    // `(Addr_GfxObject_Kid).w`, check signed Kid Y within inclusive
    // `[head_y-$40, head_y]` activation window, branch back to `loc_3960E`
    // wait loop if outside.
    let lower_y = head_y.wrapping_sub(0x0040);
    let kid_within_vertical_window =
        !word_lt_signed(kid_y, lower_y) && !word_gt_signed(kid_y, head_y);

    if !kid_within_vertical_window {
        return ShishkaBagelHead2ActivationWindowLoc3962C {
            kid_within_vertical_window,
            x_direction: None,
            cached_x_direction_fffffb87: None,
            animation_script_label: None,
            next: ShishkaBagelHead2ActivationNextLoc3962C::WaitLoopLoc3960E,
        };
    }

    // ASM loc_3965A (kid.asm:42253): `sf x_direction(a3)` — clear direction
    // to 0 when head 2 is strictly right of Kid (`cmp.w d5,d3; bgt.s`).
    // Fallthrough at kid.asm:42249 stores `$FF` via `st x_direction(a3)`.
    let x_direction = if word_gt_signed(head_x, kid_x) {
        0
    } else {
        0xFF
    };

    // ASM loc_3965E (kid.asm:42256): mirror selected `x_direction(a3)` byte
    // into `($FFFFFB87).w`, load `stru_39FC6` into `d7`, call
    // `j_Init_Animation` to start the attack animation.
    ShishkaBagelHead2ActivationWindowLoc3962C {
        kid_within_vertical_window,
        x_direction: Some(x_direction),
        cached_x_direction_fffffb87: Some(x_direction),
        animation_script_label: Some("stru_39FC6"),
        next: ShishkaBagelHead2ActivationNextLoc3962C::AttackAnimationLoc3966E,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_2_attack_wait_loop_loc_3966E(
    variant_d2: u16,
    movement_status_d7: u16,
) -> ShishkaBagelHead2AttackWaitLoopLoc3966E {
    // ASM loc_3966E (kid.asm:42261): hibernate one frame after `stru_39FC6`,
    // then branch on d2: d2==0 calls `sub_398BC` (ShishkaBoss movement),
    // d2!=0 falls through to loc_3967C → `bsr.w sub_39D4A` (BagelBrothers
    // movement).
    //
    // ASM loc_3967C (kid.asm:42269): `bsr.w sub_39D4A` — BagelBrothers
    // movement subroutine call, reached when d2!=0 (`tst.w d2; bne.s`).
    //
    // ASM loc_39680 (kid.asm:42272): convergence — `move.w d7,($FFFFFB74).w`,
    // `tst.w d7`, `beq.s loc_3968C` (check animation done),
    // `bra.w loc_39EBE` (shared collision exit) when d7 nonzero.
    let is_shishka = variant_d2 == 0;
    ShishkaBagelHead2AttackWaitLoopLoc3966E {
        hibernates_one_frame: true,
        runs_shishka_movement_sub_398bc: is_shishka,
        runs_bagel_movement_sub_39d4a: !is_shishka,
        cached_collision_word_fffffb74: movement_status_d7,
        next: if movement_status_d7 == 0 {
            ShishkaBagelHead2AttackWaitNextLoc3966E::CheckAttackAnimationDoneLoc3968C
        } else {
            ShishkaBagelHead2AttackWaitNextLoc3966E::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_2_attack_anim_gate_loc_3968C(
    animation_done_byte_18: u8,
) -> ShishkaBagelHead2AttackAnimGateLoc3968C {
    // ASM loc_3968C (kid.asm:42279): `tst.b $18(a3)` loops back to loc_3966E
    // while the attack animation is still active; nonzero byte calls
    // `sub_397E4` to spawn the head 2 projectile before the post-projectile
    // wait loop at loc_39696.
    let attack_animation_done = animation_done_byte_18 != 0;
    ShishkaBagelHead2AttackAnimGateLoc3968C {
        animation_done_byte_18,
        attack_animation_done,
        runs_projectile_spawn_sub_397e4: attack_animation_done,
        next: if attack_animation_done {
            ShishkaBagelHead2AttackAnimGateNextLoc3968C::SpawnProjectileSub397E4
        } else {
            ShishkaBagelHead2AttackAnimGateNextLoc3968C::AttackWaitLoopLoc3966E
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_2_post_projectile_wait_loop_loc_39696(
    variant_d2: u16,
    movement_status_d7: u16,
) -> ShishkaBagelHead2PostProjectileWaitLoopLoc39696 {
    // ASM loc_39696 (kid.asm:42284): post-projectile wait loop — hibernate one
    // frame, branch on d2: d2==0 calls `sub_398BC` (ShishkaBoss movement),
    // d2!=0 falls through to loc_396A4 → `bsr.w sub_39D4A` (BagelBrothers
    // movement). Structurally mirrors the attack wait loop at loc_3966E but
    // runs after `sub_397E4` has spawned the head 2 projectile.
    //
    // ASM loc_396A4 (kid.asm:42292): `bsr.w sub_39D4A` — BagelBrothers
    // movement subroutine call in the post-projectile wait loop, reached
    // when d2!=0 (`tst.w d2; bne.s`).
    //
    // ASM loc_396A8 (kid.asm:42295): convergence — `move.w d7,($FFFFFB74).w`,
    // `tst.w d7`, `beq.s loc_396B4` (check animation done),
    // `bra.w loc_39EBE` (shared collision exit) when d7 nonzero.
    let is_shishka = variant_d2 == 0;
    ShishkaBagelHead2PostProjectileWaitLoopLoc39696 {
        hibernates_one_frame: true,
        runs_shishka_movement_sub_398bc: is_shishka,
        runs_bagel_movement_sub_39d4a: !is_shishka,
        cached_collision_word_fffffb74: movement_status_d7,
        next: if movement_status_d7 == 0 {
            ShishkaBagelHead2PostProjectileWaitNextLoc39696::CheckAnimationDoneLoc396B4
        } else {
            ShishkaBagelHead2PostProjectileWaitNextLoc39696::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_2_post_projectile_anim_gate_loc_396B4(
    animation_done_byte_18: u8,
) -> ShishkaBagelHead2PostProjectileAnimGateLoc396B4 {
    // ASM loc_396B4 (kid.asm:42302): `tst.b $18(a3)` loops back to loc_39696
    // while the post-projectile animation is still active; nonzero byte returns
    // the head to the activation wait loop at loc_3960E (`bra.w loc_3960E`).
    let post_projectile_animation_done = animation_done_byte_18 != 0;
    ShishkaBagelHead2PostProjectileAnimGateLoc396B4 {
        animation_done_byte_18,
        post_projectile_animation_done,
        next: if post_projectile_animation_done {
            ShishkaBagelHead2PostProjectileAnimGateNextLoc396B4::ReturnToActivationWaitLoc3960E
        } else {
            ShishkaBagelHead2PostProjectileAnimGateNextLoc396B4::AwaitPostProjectileWaitLoc39696
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_3_init_loc_396BE(
    anchor_x_44: u16,
    anchor_y_46: u16,
    variant_d2: u16,
) -> ShishkaBagelHead3InitLoc396BE {
    // ASM loc_396BE (kid.asm:42308): head 3 init for ShishkaBoss/BagelBrothers.
    // Allocates gfx slot $01040002, priority 0. Selects object_meta $22 (d2==0,
    // ShishkaBoss) or $24 (d2!=0, BagelBrothers) then falls through to loc_396DA.
    //
    // ASM loc_396DA (kid.asm:42317): head 3 setup — stores d0 into object_meta(a3),
    // bset #7, jsr loc_32146, sf has_kid_collision, st $13, st is_moved,
    // move.w #$1E,$3E(a3), LnkTo_unk_C8600 sprite, cache a3 at $FFFFFB82,
    // copy anchor $44/$46. Then branch on d2: unk_39872 +
    // ShishkaBoss_HitPointsPerHead ($14) when d2==0 (via loc_3972E),
    // unk_39D80 + BagelBrothers_HitPointsPerHead ($1E) when d2!=0.
    //
    // ASM loc_3972E (kid.asm:42340): ShishkaBoss HP path — move.w
    // #ShishkaBoss_HitPointsPerHead,$3E(a3), overwriting the initial $1E.
    let (object_meta_base, hit_points, movement_script_label) = if variant_d2 == 0 {
        (0x0022_u16, 0x0014_u16, "unk_39872")
    } else {
        (0x0024, 0x001E, "unk_39D80")
    };

    ShishkaBagelHead3InitLoc396BE {
        gfx_slot_base: 0x0104_0002,
        priority: 0,
        object_meta_base,
        sets_object_meta_bit_f: true,
        runs_common_boss_init_loc_32146: true,
        has_kid_collision: false,
        display_enabled: true,
        is_moved: true,
        initial_field_3e: 0x001E,
        hit_points,
        sprite_mapping_label: "LnkTo_unk_C8600-Data_Index",
        cached_worker_slot_label: "$FFFFFB82",
        x_pos: anchor_x_44,
        y_pos: anchor_y_46,
        movement_script_label,
        delay_counter_d0: 1,
        script_index_d1: 0,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_3_wait_loop_loc_39734(
    variant_d2: u16,
    movement_status_d7: u16,
) -> ShishkaBagelHead3WaitLoopLoc39734 {
    // ASM loc_39734 (kid.asm:42343): head 3 wait loop — hibernate one frame,
    // then branch on d2: d2==0 calls sub_398BC (ShishkaBoss movement),
    // d2!=0 falls through to loc_39742 → bsr.w sub_39EB4 (BagelBrothers movement).
    //
    // ASM loc_39742 (kid.asm:42351): BagelBrothers path — bsr.w sub_39EB4.
    //
    // ASM loc_39746 (kid.asm:42354): convergence point — move.w d7,($FFFFFB76).w,
    // tst.w d7, beq.s loc_39752 (activation window check),
    // bra.w loc_39EBE (shared collision exit) when d7 nonzero.
    let is_shishka = variant_d2 == 0;
    ShishkaBagelHead3WaitLoopLoc39734 {
        hibernates_one_frame: true,
        runs_shishka_movement_sub_398bc: is_shishka,
        runs_bagel_movement_sub_39eb4: !is_shishka,
        cached_collision_word_fffffb76: movement_status_d7,
        next: if movement_status_d7 == 0 {
            ShishkaBagelHead3WaitNextLoc39734::CheckKidWindowLoc39752
        } else {
            ShishkaBagelHead3WaitNextLoc39734::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_3_activation_window_loc_39752(
    kid_x: u16,
    kid_y: u16,
    head_x: u16,
    head_y: u16,
) -> ShishkaBagelHead3ActivationWindowLoc39752 {
    // ASM loc_39752 (kid.asm:42361): load Kid gfx coordinates from
    // `(Addr_GfxObject_Kid).w`, check signed Kid Y within inclusive
    // `[head_y-$40, head_y]` activation window, branch back to `loc_39734`
    // wait loop if outside.
    let lower_y = head_y.wrapping_sub(0x0040);
    let kid_within_vertical_window =
        !word_lt_signed(kid_y, lower_y) && !word_gt_signed(kid_y, head_y);

    if !kid_within_vertical_window {
        return ShishkaBagelHead3ActivationWindowLoc39752 {
            kid_within_vertical_window,
            x_direction: None,
            cached_x_direction_fffffb88: None,
            animation_script_label: None,
            next: ShishkaBagelHead3ActivationNextLoc39752::WaitLoopLoc39734,
        };
    }

    // ASM loc_39780 (kid.asm:42379): `sf x_direction(a3)` — clear direction
    // to 0 when head 3 is strictly right of Kid (`cmp.w d5,d3; bgt.s`).
    // Fallthrough at kid.asm:42375 stores `$FF` via `st x_direction(a3)`.
    let x_direction = if word_gt_signed(head_x, kid_x) {
        0
    } else {
        0xFF
    };

    // ASM loc_39784 (kid.asm:42382): mirror selected `x_direction(a3)` byte
    // into `($FFFFFB88).w`, load `stru_39FC6` into `d7`, call
    // `j_Init_Animation` to start the attack animation.
    ShishkaBagelHead3ActivationWindowLoc39752 {
        kid_within_vertical_window,
        x_direction: Some(x_direction),
        cached_x_direction_fffffb88: Some(x_direction),
        animation_script_label: Some("stru_39FC6"),
        next: ShishkaBagelHead3ActivationNextLoc39752::AttackAnimationLoc39794,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_3_attack_wait_loop_loc_39794(
    variant_d2: u16,
    movement_status_d7: u16,
) -> ShishkaBagelHead3AttackWaitLoopLoc39794 {
    // ASM loc_39794 (kid.asm:42387): hibernate one frame after `stru_39FC6`,
    // then branch on d2: d2==0 calls `sub_398BC` (ShishkaBoss movement),
    // d2!=0 falls through to loc_397A2 → `bsr.w sub_39EB4` (BagelBrothers
    // movement).
    //
    // ASM loc_397A2 (kid.asm:42395): `bsr.w sub_39EB4` — BagelBrothers
    // movement subroutine call, reached when d2!=0 (`tst.w d2; bne.s`).
    //
    // ASM loc_397A6 (kid.asm:42398): convergence — `move.w d7,($FFFFFB76).w`,
    // `tst.w d7`, `beq.s loc_397B2` (check animation done),
    // `bra.w loc_39EBE` (shared collision exit) when d7 nonzero.
    let is_shishka = variant_d2 == 0;
    ShishkaBagelHead3AttackWaitLoopLoc39794 {
        hibernates_one_frame: true,
        runs_shishka_movement_sub_398bc: is_shishka,
        runs_bagel_movement_sub_39eb4: !is_shishka,
        cached_collision_word_fffffb76: movement_status_d7,
        next: if movement_status_d7 == 0 {
            ShishkaBagelHead3AttackWaitNextLoc39794::CheckAttackAnimationDoneLoc397B2
        } else {
            ShishkaBagelHead3AttackWaitNextLoc39794::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_3_post_projectile_wait_loop_loc_397bc(
    variant_d2: u16,
    movement_status_d7: u16,
) -> ShishkaBagelHead3PostProjectileWaitLoopLoc397BC {
    // ASM loc_397BC (kid.asm:42410): post-projectile wait loop — hibernate one
    // frame, branch on d2: d2==0 calls `sub_398BC` (ShishkaBoss movement),
    // d2!=0 falls through to loc_397CA → `bsr.w sub_39EB4` (BagelBrothers
    // movement). Structurally mirrors the attack wait loop at loc_39794 but
    // runs after `sub_397E4` has spawned the head 3 projectile.
    //
    // ASM loc_397CA (kid.asm:42418): `bsr.w sub_39EB4` — BagelBrothers
    // movement subroutine call in the post-projectile wait loop, reached
    // when d2!=0 (`tst.w d2; bne.s`).
    //
    // ASM loc_397CE (kid.asm:42421): convergence — `move.w d7,($FFFFFB76).w`,
    // `tst.w d7`, `beq.s loc_397DA` (check animation done),
    // `bra.w loc_39EBE` (shared collision exit) when d7 nonzero.
    let is_shishka = variant_d2 == 0;
    ShishkaBagelHead3PostProjectileWaitLoopLoc397BC {
        hibernates_one_frame: true,
        runs_shishka_movement_sub_398bc: is_shishka,
        runs_bagel_movement_sub_39eb4: !is_shishka,
        cached_collision_word_fffffb76: movement_status_d7,
        next: if movement_status_d7 == 0 {
            ShishkaBagelHead3PostProjectileWaitNextLoc397BC::CheckAnimationDoneLoc397DA
        } else {
            ShishkaBagelHead3PostProjectileWaitNextLoc397BC::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_3_post_projectile_anim_gate_loc_397da(
    animation_done_byte_18: u8,
) -> ShishkaBagelHead3PostProjectileAnimGateLoc397DA {
    // ASM loc_397DA (kid.asm:42428): `tst.b $18(a3)` loops back to loc_397BC
    // while the post-projectile animation is still active; nonzero byte returns
    // the head to the wait loop at loc_39734 (`bra.w loc_39734`).
    let post_projectile_animation_done = animation_done_byte_18 != 0;
    ShishkaBagelHead3PostProjectileAnimGateLoc397DA {
        animation_done_byte_18,
        post_projectile_animation_done,
        next: if post_projectile_animation_done {
            ShishkaBagelHead3PostProjectileAnimGateNextLoc397DA::ReturnToWaitLoopLoc39734
        } else {
            ShishkaBagelHead3PostProjectileAnimGateNextLoc397DA::AwaitPostProjectileWaitLoc397BC
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_1_wait_loop_loc_394E8(
    variant_d2: u16,
    movement_status_d7: u16,
) -> ShishkaBagelHead1WaitLoopLoc394E8 {
    // ASM loc_394E8 (kid.asm:42091): head 1 wait loop — hibernate one frame,
    // then branch on d2: d2==0 calls sub_398BC (ShishkaBoss movement),
    // d2!=0 falls through to loc_394F6 → bsr.w sub_39BF8 (BagelBrothers movement).
    //
    // ASM loc_394F6 (kid.asm:42099): BagelBrothers path — bsr.w sub_39BF8.
    //
    // ASM loc_394FA (kid.asm:42102): convergence point — move.w d7,($FFFFFB72).w,
    // tst.w d7, beq.s loc_39506 (activation window check),
    // bra.w loc_39EBE (shared collision exit) when d7 nonzero.
    let is_shishka = variant_d2 == 0;
    ShishkaBagelHead1WaitLoopLoc394E8 {
        hibernates_one_frame: true,
        runs_shishka_movement_sub_398bc: is_shishka,
        runs_bagel_movement_sub_39bf8: !is_shishka,
        cached_collision_word_fffffb72: movement_status_d7,
        next: if movement_status_d7 == 0 {
            ShishkaBagelHead1WaitNextLoc394E8::CheckKidWindowLoc39506
        } else {
            ShishkaBagelHead1WaitNextLoc394E8::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_1_activation_window_loc_39506(
    kid_x: u16,
    kid_y: u16,
    head_x: u16,
    head_y: u16,
) -> ShishkaBagelHead1ActivationWindowLoc39506 {
    // ASM loc_39506 (kid.asm:42109): load Kid gfx coordinates from
    // `(Addr_GfxObject_Kid).w`, check signed Kid Y within inclusive
    // `[head_y-$40, head_y]` activation window, branch back to `loc_394E8`
    // wait loop if outside.
    let lower_y = head_y.wrapping_sub(0x0040);
    let kid_within_vertical_window =
        !word_lt_signed(kid_y, lower_y) && !word_gt_signed(kid_y, head_y);

    if !kid_within_vertical_window {
        return ShishkaBagelHead1ActivationWindowLoc39506 {
            kid_within_vertical_window,
            x_direction: None,
            cached_x_direction_fffffb86: None,
            animation_script_label: None,
            next: ShishkaBagelHead1ActivationNextLoc39506::WaitLoopLoc394E8,
        };
    }

    // ASM loc_39534 (kid.asm:42127): `sf x_direction(a3)` — clear direction
    // to 0 when head 1 is strictly right of Kid (`cmp.w d5,d3; bgt.s`).
    // Fallthrough at kid.asm:42123 stores `$FF` via `st x_direction(a3)`.
    let x_direction = if word_gt_signed(head_x, kid_x) {
        0
    } else {
        0xFF
    };

    // ASM loc_39538 (kid.asm:42130): mirror selected `x_direction(a3)` byte
    // into `($FFFFFB86).w`, load `stru_39FC6` into `d7`, call
    // `j_Init_Animation` to start the attack animation.
    ShishkaBagelHead1ActivationWindowLoc39506 {
        kid_within_vertical_window,
        x_direction: Some(x_direction),
        cached_x_direction_fffffb86: Some(x_direction),
        animation_script_label: Some("stru_39FC6"),
        next: ShishkaBagelHead1ActivationNextLoc39506::AttackAnimationLoc39548,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_1_attack_wait_loop_loc_39548(
    variant_d2: u16,
    movement_status_d7: u16,
) -> ShishkaBagelHead1AttackWaitLoopLoc39548 {
    // ASM loc_39548 (kid.asm:42135): hibernate one frame after `stru_39FC6`,
    // then branch on d2: d2==0 calls `sub_398BC` (ShishkaBoss movement),
    // d2!=0 falls through to loc_39556 → `bsr.w sub_39BF8` (BagelBrothers
    // movement).
    //
    // ASM loc_39556 (kid.asm:42143): `bsr.w sub_39BF8` — BagelBrothers
    // movement subroutine call, reached when d2!=0 (`tst.w d2; bne.s`).
    //
    // ASM loc_3955A (kid.asm:42146): convergence — `move.w d7,($FFFFFB72).w`,
    // `tst.w d7`, `beq.s loc_39566` (check animation done),
    // `bra.w loc_39EBE` (shared collision exit) when d7 nonzero.
    let is_shishka = variant_d2 == 0;
    ShishkaBagelHead1AttackWaitLoopLoc39548 {
        hibernates_one_frame: true,
        runs_shishka_movement_sub_398bc: is_shishka,
        runs_bagel_movement_sub_39bf8: !is_shishka,
        cached_collision_word_fffffb72: movement_status_d7,
        next: if movement_status_d7 == 0 {
            ShishkaBagelHead1AttackWaitNextLoc39548::CheckAttackAnimationDoneLoc39566
        } else {
            ShishkaBagelHead1AttackWaitNextLoc39548::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_1_attack_anim_gate_loc_39566(
    animation_done_byte_18: u8,
) -> ShishkaBagelHead1AttackAnimGateLoc39566 {
    // ASM loc_39566 (kid.asm:42153): `tst.b $18(a3)` loops back to loc_39548
    // while the attack animation is still active; nonzero byte calls
    // `sub_397E4` to spawn the head 1 projectile before the post-projectile
    // wait loop at loc_39570.
    let attack_animation_done = animation_done_byte_18 != 0;
    ShishkaBagelHead1AttackAnimGateLoc39566 {
        animation_done_byte_18,
        attack_animation_done,
        runs_projectile_spawn_sub_397e4: attack_animation_done,
        next: if attack_animation_done {
            ShishkaBagelHead1AttackAnimGateNextLoc39566::SpawnProjectileSub397E4
        } else {
            ShishkaBagelHead1AttackAnimGateNextLoc39566::AttackWaitLoopLoc39548
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_1_post_projectile_wait_loop_loc_39570(
    variant_d2: u16,
    movement_status_d7: u16,
) -> ShishkaBagelHead1PostProjectileWaitLoopLoc39570 {
    // ASM loc_39570 (kid.asm:42158): post-projectile wait loop — hibernate one
    // frame, branch on d2: d2==0 calls `sub_398BC` (ShishkaBoss movement),
    // d2!=0 falls through to loc_3957E → `bsr.w sub_39BF8` (BagelBrothers
    // movement). Structurally mirrors the attack wait loop at loc_39548 but
    // runs after `sub_397E4` has spawned the head 1 projectile.
    //
    // ASM loc_3957E (kid.asm:42166): `bsr.w sub_39BF8` — BagelBrothers
    // movement subroutine call in the post-projectile wait loop, reached
    // when d2!=0 (`tst.w d2; bne.s`).
    //
    // ASM loc_39582 (kid.asm:42169): convergence — `move.w d7,($FFFFFB72).w`,
    // `tst.w d7`, `beq.s loc_3958E` (check animation done),
    // `bra.w loc_39EBE` (shared collision exit) when d7 nonzero.
    let is_shishka = variant_d2 == 0;
    ShishkaBagelHead1PostProjectileWaitLoopLoc39570 {
        hibernates_one_frame: true,
        runs_shishka_movement_sub_398bc: is_shishka,
        runs_bagel_movement_sub_39bf8: !is_shishka,
        cached_collision_word_fffffb72: movement_status_d7,
        next: if movement_status_d7 == 0 {
            ShishkaBagelHead1PostProjectileWaitNextLoc39570::CheckAnimationDoneLoc3958E
        } else {
            ShishkaBagelHead1PostProjectileWaitNextLoc39570::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn shishka_bagel_head_1_post_projectile_anim_gate_loc_3958E(
    animation_done_byte_18: u8,
) -> ShishkaBagelHead1PostProjectileAnimGateLoc3958E {
    // ASM loc_3958E (kid.asm:42176): `tst.b $18(a3)` loops back to loc_39570
    // while the post-projectile animation is still active; nonzero byte returns
    // the head to the activation wait loop at loc_394E8 (`bra.w loc_394E8`).
    let post_projectile_animation_done = animation_done_byte_18 != 0;
    ShishkaBagelHead1PostProjectileAnimGateLoc3958E {
        animation_done_byte_18,
        post_projectile_animation_done,
        next: if post_projectile_animation_done {
            ShishkaBagelHead1PostProjectileAnimGateNextLoc3958E::ReturnToActivationWaitLoc394E8
        } else {
            ShishkaBagelHead1PostProjectileAnimGateNextLoc3958E::AwaitPostProjectileWaitLoc39570
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_wait_loop_loc_37F80(
    movement_status_d7: u16,
) -> BoomerangBossHeadWaitLoopLoc37F80 {
    // ASM loc_37F80 (kid.asm:40344): hibernate one frame, run `sub_39952`,
    // mirror its `d7` status into `$FFFFFB72`, and branch to `loc_39EBE`
    // instead of the Kid-window check whenever that word is nonzero.
    BoomerangBossHeadWaitLoopLoc37F80 {
        hibernates_one_frame: true,
        runs_movement_wrapper_sub_39952: true,
        cached_collision_word_fffffb72: movement_status_d7,
        next: if movement_status_d7 == 0 {
            BoomerangBossHeadWaitNextLoc37F80::CheckKidWindowLoc37F94
        } else {
            BoomerangBossHeadWaitNextLoc37F80::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_activation_loc_37F94(
    kid_x: u16,
    kid_y: u16,
    head_x: u16,
    head_y: u16,
) -> BoomerangBossHeadActivationLoc37F94 {
    // ASM loc_37F94 (kid.asm:40353): branch back to `loc_37F80` unless Kid's
    // signed gfx Y lies in the inclusive `head_y-$40..=head_y` window, then
    // face left (`0`) only when `head_x > kid_x`.
    // ASM loc_37FC6 (kid.asm:40374): mirror the chosen `x_direction(a3)` into
    // `$FFFFFB86` and start attack animation `stru_39FC6`.
    let lower_y = head_y.wrapping_sub(0x0040);
    let kid_within_vertical_window =
        !word_lt_signed(kid_y, lower_y) && !word_gt_signed(kid_y, head_y);

    if !kid_within_vertical_window {
        return BoomerangBossHeadActivationLoc37F94 {
            kid_within_vertical_window,
            x_direction: None,
            cached_x_direction_fffffb86: None,
            animation_script_label: None,
            next: BoomerangBossHeadActivationNextLoc37F94::WaitLoopLoc37F80,
        };
    }

    let x_direction = if word_gt_signed(head_x, kid_x) {
        // ASM loc_37FC2 (kid.asm:40371): `sf x_direction(a3)` when the
        // signed head X word is greater than Kid's X word.
        0
    } else {
        0xFF
    };

    BoomerangBossHeadActivationLoc37F94 {
        kid_within_vertical_window,
        x_direction: Some(x_direction),
        cached_x_direction_fffffb86: Some(x_direction),
        animation_script_label: Some("stru_39FC6"),
        next: BoomerangBossHeadActivationNextLoc37F94::AttackAnimationLoc37FC6,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_attack_wait_loop_loc_37FD6(
    movement_status_d7: u16,
) -> BoomerangBossHeadAttackWaitLoopLoc37FD6 {
    // ASM loc_37FD6 (kid.asm:40379): hibernate one frame after `stru_39FC6`,
    // run `sub_39952`, mirror `d7` into `$FFFFFB72`, and branch to `loc_39EBE`
    // on nonzero status before the `$18(a3)` animation-done test at loc_37FEA.
    BoomerangBossHeadAttackWaitLoopLoc37FD6 {
        hibernates_one_frame: true,
        runs_movement_wrapper_sub_39952: true,
        cached_collision_word_fffffb72: movement_status_d7,
        next: if movement_status_d7 == 0 {
            BoomerangBossHeadAttackWaitNextLoc37FD6::CheckAttackAnimationDoneLoc37FEA
        } else {
            BoomerangBossHeadAttackWaitNextLoc37FD6::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_attack_animation_gate_loc_37FEA(
    animation_done_byte_18: u8,
) -> BoomerangBossHeadAttackAnimationGateLoc37FEA {
    // ASM loc_37FEA (kid.asm:40388): `tst.b $18(a3)` loops back to loc_37FD6
    // while the attack animation is still active; any nonzero byte calls
    // `sub_397E4` to spawn the head projectile.
    let attack_animation_done = animation_done_byte_18 != 0;
    BoomerangBossHeadAttackAnimationGateLoc37FEA {
        attack_animation_done,
        runs_projectile_spawn_sub_397e4: attack_animation_done,
        next: if attack_animation_done {
            BoomerangBossHeadAttackAnimationNextLoc37FEA::SpawnProjectileSub397E4
        } else {
            BoomerangBossHeadAttackAnimationNextLoc37FEA::AwaitAnimationLoc37FD6
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_post_projectile_wait_loop_loc_37FF4(
    movement_status_d7: u16,
) -> BoomerangBossHeadPostProjectileWaitLoopLoc37FF4 {
    // ASM loc_37FF4 (kid.asm:40393): after `sub_397E4`, hibernate one frame,
    // run `sub_39952`, mirror `d7` into `$FFFFFB72`, and branch to `loc_39EBE`
    // on nonzero status before the post-projectile animation-done check.
    BoomerangBossHeadPostProjectileWaitLoopLoc37FF4 {
        hibernates_one_frame: true,
        runs_movement_wrapper_sub_39952: true,
        cached_collision_word_fffffb72: movement_status_d7,
        next: if movement_status_d7 == 0 {
            BoomerangBossHeadPostProjectileWaitNextLoc37FF4::CheckAnimationDone
        } else {
            BoomerangBossHeadPostProjectileWaitNextLoc37FF4::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_post_projectile_animation_gate_loc_38008(
    animation_done_byte_18: u8,
) -> BoomerangBossHeadPostProjectileAnimationGateLoc38008 {
    // ASM loc_38008 (kid.asm:40402): after the post-projectile wait loop,
    // `tst.b $18(a3)` branches back to loc_37FF4 until the animation finishes;
    // any nonzero byte returns the head to the activation wait at loc_37F80.
    let post_projectile_animation_done = animation_done_byte_18 != 0;
    BoomerangBossHeadPostProjectileAnimationGateLoc38008 {
        animation_done_byte_18,
        post_projectile_animation_done,
        next: if post_projectile_animation_done {
            BoomerangBossHeadPostProjectileAnimationNextLoc38008::ReturnToActivationWaitLoc37F80
        } else {
            BoomerangBossHeadPostProjectileAnimationNextLoc38008::AwaitPostProjectileWaitLoc37FF4
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_init_loc_38012(
    anchor_x_44: u16,
    anchor_y_46: u16,
) -> BoomerangBossHeadInitLoc37F30 {
    // ASM loc_38012 (kid.asm:40408): allocate the second BoomerangBoss head
    // with the same slot/metadata/flags as loc_37F30, cache it at
    // `$FFFFFB7E`, copy anchor `$44/$46`, and enter `unk_39988` at index 1.
    BoomerangBossHeadInitLoc37F30 {
        gfx_slot_base: 0x0104_0002,
        priority: 1,
        object_meta: 0x0023,
        runs_common_boss_init_loc_32146: true,
        has_kid_collision: false,
        display_enabled: true,
        is_moved: true,
        hit_points: 30,
        sprite_mapping_label: "LnkTo_unk_C8600-Data_Index",
        cached_worker_slot_label: "$FFFFFB7E",
        x_pos: anchor_x_44,
        y_pos: anchor_y_46,
        movement_script_label: "unk_39988",
        delay_counter_d0: 1,
        script_index_d1: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_init_loc_380F4(
    anchor_x_44: u16,
    anchor_y_46: u16,
) -> BoomerangBossHeadInitLoc37F30 {
    // ASM loc_380F4 (kid.asm:40491): allocate the third BoomerangBoss head
    // with the same slot/metadata/flags as the earlier heads, cache it at
    // `$FFFFFB82`, copy anchor `$44/$46`, and enter `unk_39A1A` at index 1.
    BoomerangBossHeadInitLoc37F30 {
        gfx_slot_base: 0x0104_0002,
        priority: 1,
        object_meta: 0x0023,
        runs_common_boss_init_loc_32146: true,
        has_kid_collision: false,
        display_enabled: true,
        is_moved: true,
        hit_points: 30,
        sprite_mapping_label: "LnkTo_unk_C8600-Data_Index",
        cached_worker_slot_label: "$FFFFFB82",
        x_pos: anchor_x_44,
        y_pos: anchor_y_46,
        movement_script_label: "unk_39A1A",
        delay_counter_d0: 1,
        script_index_d1: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_wait_loop_loc_38144(
    movement_status_d7: u16,
) -> BoomerangBossHead3WaitLoopLoc38144 {
    // ASM loc_38144 (kid.asm:40510): hibernate one frame, run head 3's
    // `sub_39A7C` movement wrapper, mirror `d7` into `$FFFFFB76`, and branch
    // to shared `loc_39EBE` on nonzero status before checking Kid's window.
    BoomerangBossHead3WaitLoopLoc38144 {
        hibernates_one_frame: true,
        runs_movement_wrapper_sub_39a7c: true,
        cached_collision_word_fffffb76: movement_status_d7,
        next: if movement_status_d7 == 0 {
            BoomerangBossHead3WaitNextLoc38144::CheckKidWindow
        } else {
            BoomerangBossHead3WaitNextLoc38144::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_activation_window_loc_38158(
    kid_x: u16,
    kid_y: u16,
    head_x: u16,
    head_y: u16,
) -> BoomerangBossHead3ActivationWindowLoc38158 {
    // ASM loc_38158 (kid.asm:40519): load Kid/head coordinates, then branch
    // back to `loc_38144` unless signed Kid Y lies inside the inclusive
    // `head_y-$40..=head_y` activation window.
    let lower_y = head_y.wrapping_sub(0x0040);
    let kid_within_vertical_window =
        !word_lt_signed(kid_y, lower_y) && !word_gt_signed(kid_y, head_y);

    if !kid_within_vertical_window {
        return BoomerangBossHead3ActivationWindowLoc38158 {
            kid_within_vertical_window,
            x_direction: None,
            cached_x_direction_fffffb88: None,
            animation_script_label: None,
            next: BoomerangBossHead3ActivationNextLoc38158::WaitLoopLoc38144,
        };
    }

    // ASM loc_38186 (kid.asm:40537): the signed `cmp.w d5,d3; bgt` branch
    // clears `x_direction(a3)` when head 3 is strictly right of Kid; otherwise
    // the fallthrough path stores `$FF`.
    let x_direction = if word_gt_signed(head_x, kid_x) {
        0
    } else {
        0xFF
    };

    // ASM loc_3818A (kid.asm:40540): mirror head 3's selected `x_direction`
    // byte into `$FFFFFB88`, load `stru_39FC6` into `d7`, then initialise the
    // attack animation before entering the next wait loop.
    BoomerangBossHead3ActivationWindowLoc38158 {
        kid_within_vertical_window,
        x_direction: Some(x_direction),
        cached_x_direction_fffffb88: Some(x_direction),
        animation_script_label: Some("stru_39FC6"),
        next: BoomerangBossHead3ActivationNextLoc38158::AttackAnimation,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_attack_wait_loop_loc_3819A(
    movement_status_d7: u16,
) -> BoomerangBossHead3AttackWaitLoopLoc3819A {
    // ASM loc_3819A (kid.asm:40545): hibernate one frame after `stru_39FC6`,
    // run `sub_39A7C`, mirror `d7` into `$FFFFFB76`, and branch to
    // `loc_39EBE` on nonzero status before testing the attack animation.
    BoomerangBossHead3AttackWaitLoopLoc3819A {
        hibernates_one_frame: true,
        runs_movement_wrapper_sub_39a7c: true,
        cached_collision_word_fffffb76: movement_status_d7,
        next: if movement_status_d7 == 0 {
            BoomerangBossHead3AttackWaitNextLoc3819A::CheckAttackAnimationDone
        } else {
            BoomerangBossHead3AttackWaitNextLoc3819A::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_attack_animation_gate_loc_381AE(
    animation_done_byte_18: u8,
) -> BoomerangBossHead3AttackAnimationGateLoc381AE {
    // ASM loc_381AE (kid.asm:40554): `tst.b $18(a3)` loops back to loc_3819A
    // while the attack animation is still active; any nonzero byte calls
    // `sub_397E4` to spawn the head 3 projectile before the post-projectile wait.
    let attack_animation_done = animation_done_byte_18 != 0;
    BoomerangBossHead3AttackAnimationGateLoc381AE {
        attack_animation_done,
        runs_projectile_spawn_sub_397e4: attack_animation_done,
        next: if attack_animation_done {
            BoomerangBossHead3AttackAnimationNextLoc381AE::SpawnProjectileSub397E4
        } else {
            BoomerangBossHead3AttackAnimationNextLoc381AE::AwaitAttackWaitLoopLoc3819A
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_post_projectile_wait_loop_loc_381B8(
    movement_status_d7: u16,
) -> BoomerangBossHead3PostProjectileWaitLoopLoc381B8 {
    // ASM loc_381B8 (kid.asm:40559): after `sub_397E4`, hibernate one frame,
    // run head 3's `sub_39A7C`, mirror `d7` into `$FFFFFB76`, and branch to
    // shared `loc_39EBE` on nonzero status before checking `$18(a3)`.
    BoomerangBossHead3PostProjectileWaitLoopLoc381B8 {
        hibernates_one_frame: true,
        runs_movement_wrapper_sub_39a7c: true,
        cached_collision_word_fffffb76: movement_status_d7,
        next: if movement_status_d7 == 0 {
            BoomerangBossHead3PostProjectileWaitNextLoc381B8::CheckAnimationDone
        } else {
            BoomerangBossHead3PostProjectileWaitNextLoc381B8::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_post_projectile_animation_gate_loc_381CC(
    animation_done_byte_18: u8,
) -> BoomerangBossHead3PostProjectileAnimationGateLoc381CC {
    // ASM loc_381CC (kid.asm:40568): keep waiting at loc_381B8 while
    // `$18(a3)` is zero; when the post-projectile animation marks completion,
    // branch back to head 3's activation wait loop at loc_38144.
    let post_projectile_animation_done = animation_done_byte_18 != 0;
    BoomerangBossHead3PostProjectileAnimationGateLoc381CC {
        post_projectile_animation_done,
        next: if post_projectile_animation_done {
            BoomerangBossHead3PostProjectileAnimationNextLoc381CC::ReturnToActivationWaitLoc38144
        } else {
            BoomerangBossHead3PostProjectileAnimationNextLoc381CC::AwaitPostProjectileWaitLoc381B8
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_1_init_loc_38494(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossSkewerPiece1InitLoc38494 {
    // ASM loc_38494 (kid.asm:40703): allocate skewer piece 1 with gfx slot
    // `$01010002`, priority 1, object_meta `$22`. Calls `loc_32146(pc)` for
    // shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C85F8-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and starts `unk_39872` script at index 1.
    BoomerangBossSkewerPiece1InitLoc38494 {
        gfx_slot: 0x0101_0002,
        priority: 1,
        object_meta: 0x22,
        has_kid_collision: false,
        display_enabled: true, // st $13(a3)
        is_moved: true,        // st is_moved(a3)
        sprite_label: "LnkTo_unk_C85F8-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        movement_script_label: "unk_39872",
        movement_script_index: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_1_wait_loop_loc_384D6(
    head_death_counter_fb4e: u8,
) -> BoomerangBossSkewerPiece1WaitLoopLoc384D6 {
    // ASM loc_384D6 (kid.asm:40720): hibernate one frame, run `sub_39846`
    // (boss head movement step), then check `cmpi.b #3,($FFFFFB4E).w`.
    // Loop back to loc_384D6 unless all 3 heads are dead ($FFFFFB4E == 3).
    BoomerangBossSkewerPiece1WaitLoopLoc384D6 {
        hibernates_one_frame: true,
        runs_movement_step_sub_39846: true,
        next: if head_death_counter_fb4e == 3 {
            BoomerangBossSkewerPiece1WaitNextLoc384D6::LaunchLoc384E8
        } else {
            BoomerangBossSkewerPiece1WaitNextLoc384D6::WaitLoopLoc384D6
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_1_launch_loc_384E8()
-> BoomerangBossSkewerPiece1LaunchLoc384E8 {
    // ASM loc_384E8 (kid.asm:40728): set y_vel to `$FFF9` (word = -7) and
    // x_vel to `$FFFF2000` (longword = -0xE000 = -57344). Then enter the
    // `loc_384F6` gravity loop: each frame adds `$1B58` to y_vel (longword),
    // checks `y_pos` against `Level_height_pixels`, and deletes the object
    // when it falls off-screen. `loc_38510` (kid.asm:40741) is the loop-back
    // branch: `bra.s loc_384F6` when `y_pos <= Level_height_pixels`.
    // Piece 1 launches up-left.
    BoomerangBossSkewerPiece1LaunchLoc384E8 {
        y_vel: -7,                        // $FFF9 as signed word
        x_vel: -0xE000_i32,               // $FFFF2000 as signed longword
        gravity_per_frame: 0x1B58,        // addi.l #$1B58,y_vel(a3)
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38510: true,       // bra.s loc_384F6 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_2_init_loc_38512(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossSkewerPiece2InitLoc38512 {
    // ASM loc_38512 (kid.asm:40745): allocate skewer piece 2 with gfx slot
    // `$01010002`, priority 1, object_meta `$22`. Calls `loc_32146(pc)` for
    // shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C85F0-Data_Index` (different
    // from piece 1's `C85F8`), copies anchor from `$44/$46(a5)`, and starts
    // `unk_39872` script at index 1.
    BoomerangBossSkewerPiece2InitLoc38512 {
        gfx_slot: 0x0101_0002,
        priority: 1,
        object_meta: 0x22,
        has_kid_collision: false,
        display_enabled: true, // st $13(a3)
        is_moved: true,        // st is_moved(a3)
        sprite_label: "LnkTo_unk_C85F0-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        movement_script_label: "unk_39872",
        movement_script_index: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_2_wait_loop_loc_38554(
    head_death_counter_fb4e: u8,
) -> BoomerangBossSkewerPiece2WaitLoopLoc38554 {
    // ASM loc_38554 (kid.asm:40762): hibernate one frame, run `sub_39846`
    // (boss head movement step), then check `cmpi.b #3,($FFFFFB4E).w`.
    // Loop back to loc_38554 unless all 3 heads are dead ($FFFFFB4E == 3),
    // then branch to `loc_38566` for piece 2 launch.
    BoomerangBossSkewerPiece2WaitLoopLoc38554 {
        hibernates_one_frame: true,
        runs_movement_step_sub_39846: true,
        next: if head_death_counter_fb4e == 3 {
            BoomerangBossSkewerPiece2WaitNextLoc38554::LaunchLoc38566
        } else {
            BoomerangBossSkewerPiece2WaitNextLoc38554::WaitLoopLoc38554
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_2_launch_loc_38566()
-> BoomerangBossSkewerPiece2LaunchLoc38566 {
    // ASM loc_38566 (kid.asm:40770): set y_vel to `$FFF9` (word = -7) and
    // x_vel to `$E000` (longword = +0xE000 = 57344, rightward). Then enter
    // the `loc_38574` gravity loop: each frame adds `$1B58` to y_vel
    // (longword), checks `y_pos` against `Level_height_pixels`, and deletes
    // the object when it falls off-screen. `loc_3858E` (kid.asm:40783) is
    // the loop-back branch: `bra.s loc_38574` when
    // `y_pos <= Level_height_pixels`. Piece 2 launches up-right.
    BoomerangBossSkewerPiece2LaunchLoc38566 {
        y_vel: -7,                        // $FFF9 as signed word
        x_vel: 0xE000_i32,                // $0000E000 as signed longword (rightward)
        gravity_per_frame: 0x1B58,        // addi.l #$1B58,y_vel(a3)
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_3858e: true,       // bra.s loc_38574 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_3_init_loc_38590(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossSkewerPiece3InitLoc38590 {
    // ASM loc_38590 (kid.asm:40787): allocate skewer piece 3 with gfx slot
    // `$01010002`, priority 1, object_meta `$22`. Calls `loc_32146(pc)` for
    // shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C85F0-Data_Index` (same as
    // piece 2), copies anchor from `$44/$46(a5)`, and starts `unk_39872`
    // script at index 1.
    BoomerangBossSkewerPiece3InitLoc38590 {
        gfx_slot: 0x0101_0002,
        priority: 1,
        object_meta: 0x22,
        has_kid_collision: false,
        display_enabled: true, // st $13(a3)
        is_moved: true,        // st is_moved(a3)
        sprite_label: "LnkTo_unk_C85F0-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        movement_script_label: "unk_39872",
        movement_script_index: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_3_wait_loop_loc_385D2(
    head_death_counter_fb4e: u8,
) -> BoomerangBossSkewerPiece3WaitLoopLoc385D2 {
    // ASM loc_385D2 (kid.asm:40804): hibernate one frame, run `sub_39846`
    // (boss head movement step), then check `cmpi.b #3,($FFFFFB4E).w`.
    // Loop back to loc_385D2 unless all 3 heads are dead ($FFFFFB4E == 3),
    // then branch to `loc_385E4` for piece 3 launch.
    BoomerangBossSkewerPiece3WaitLoopLoc385D2 {
        hibernates_one_frame: true,
        runs_movement_step_sub_39846: true,
        next: if head_death_counter_fb4e == 3 {
            BoomerangBossSkewerPiece3WaitNextLoc385D2::LaunchLoc385E4
        } else {
            BoomerangBossSkewerPiece3WaitNextLoc385D2::WaitLoopLoc385D2
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_3_launch_loc_385E4()
-> BoomerangBossSkewerPiece3LaunchLoc385E4 {
    // ASM loc_385E4 (kid.asm:40812): set y_vel to `$FFF9` (word = -7) and
    // x_vel to `$FFFF4000` (longword = -0xC000 = -49152, leftward). Then
    // enter the `loc_385F2` gravity loop: each frame adds `$1B58` to y_vel
    // (longword), checks `y_pos` against `Level_height_pixels`, and deletes
    // the object when it falls off-screen. `loc_3860C` (kid.asm:40825) is
    // the loop-back branch: `bra.s loc_385F2` when
    // `y_pos <= Level_height_pixels`. Piece 3 launches up-left.
    BoomerangBossSkewerPiece3LaunchLoc385E4 {
        y_vel: -7,                        // $FFF9 as signed word
        x_vel: -0xC000_i32,               // $FFFF4000 as signed longword (leftward)
        gravity_per_frame: 0x1B58,        // addi.l #$1B58,y_vel(a3)
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_3860c: true,       // bra.s loc_385F2 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_4_init_loc_3860E(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossSkewerPiece4InitLoc3860E {
    // ASM loc_3860E (kid.asm:40829): allocate skewer piece 4 with gfx slot
    // `$01010002`, priority 1, object_meta `$22`. Calls `loc_32146(pc)` for
    // shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C85F0-Data_Index` (same as
    // pieces 2-3), copies anchor from `$44/$46(a5)`, and starts `unk_39872`
    // script at index 1.
    BoomerangBossSkewerPiece4InitLoc3860E {
        gfx_slot: 0x0101_0002,
        priority: 1,
        object_meta: 0x22,
        has_kid_collision: false,
        display_enabled: true, // st $13(a3)
        is_moved: true,        // st is_moved(a3)
        sprite_label: "LnkTo_unk_C85F0-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        movement_script_label: "unk_39872",
        movement_script_index: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_4_wait_loop_loc_38650(
    head_death_counter_fb4e: u8,
) -> BoomerangBossSkewerPiece4WaitLoopLoc38650 {
    // ASM loc_38650 (kid.asm:40846): hibernate one frame, run `sub_39846`
    // (boss head movement step), then check `cmpi.b #3,($FFFFFB4E).w`.
    // Loop back to loc_38650 unless all 3 heads are dead ($FFFFFB4E == 3),
    // then branch to `loc_38662` for piece 4 launch.
    BoomerangBossSkewerPiece4WaitLoopLoc38650 {
        hibernates_one_frame: true,
        runs_movement_step_sub_39846: true,
        next: if head_death_counter_fb4e == 3 {
            BoomerangBossSkewerPiece4WaitNextLoc38650::LaunchLoc38662
        } else {
            BoomerangBossSkewerPiece4WaitNextLoc38650::WaitLoopLoc38650
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_4_launch_loc_38662()
-> BoomerangBossSkewerPiece4LaunchLoc38662 {
    // ASM loc_38662 (kid.asm:40854): set y_vel to `$FFF9` (word = -7) and
    // x_vel to `$C000` (longword = $0000C000 = +49152, rightward). Then
    // enter the `loc_38670` gravity loop: each frame adds `$1B58` to y_vel
    // (longword), checks `y_pos` against `Level_height_pixels`, and deletes
    // the object when it falls off-screen. `loc_3868A` (kid.asm:40867) is
    // the loop-back branch: `bra.s loc_38670` when
    // `y_pos <= Level_height_pixels`. Piece 4 launches up-right.
    BoomerangBossSkewerPiece4LaunchLoc38662 {
        y_vel: -7,                        // $FFF9 as signed word
        x_vel: 0xC000,                    // $0000C000 as signed longword (rightward)
        gravity_per_frame: 0x1B58,        // addi.l #$1B58,y_vel(a3)
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_3868a: true,       // bra.s loc_38670 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_5_init_loc_3868C(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossSkewerPiece5InitLoc3868C {
    // ASM loc_3868C (kid.asm:40871): allocate skewer piece 5 with gfx slot
    // `$01010002`, priority 1, object_meta `$22`. Calls `loc_32146(pc)` for
    // shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C85F0-Data_Index` (same as
    // pieces 2-4), copies anchor from `$44/$46(a5)`, and starts `unk_39872`
    // script at index 1.
    BoomerangBossSkewerPiece5InitLoc3868C {
        gfx_slot: 0x0101_0002,
        priority: 1,
        object_meta: 0x22,
        has_kid_collision: false,
        display_enabled: true, // st $13(a3)
        is_moved: true,        // st is_moved(a3)
        sprite_label: "LnkTo_unk_C85F0-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        movement_script_label: "unk_39872",
        movement_script_index: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_5_wait_loop_loc_386CE(
    head_death_counter_fb4e: u8,
) -> BoomerangBossSkewerPiece5WaitLoopLoc386CE {
    // ASM loc_386CE (kid.asm:40888): hibernate one frame, run `sub_39846`
    // (boss head movement step), then check `cmpi.b #3,($FFFFFB4E).w`.
    // Loop back to loc_386CE unless all 3 heads are dead ($FFFFFB4E == 3),
    // then branch to `loc_386E0` for piece 5 launch.
    BoomerangBossSkewerPiece5WaitLoopLoc386CE {
        hibernates_one_frame: true,
        runs_movement_step_sub_39846: true,
        next: if head_death_counter_fb4e == 3 {
            BoomerangBossSkewerPiece5WaitNextLoc386CE::LaunchLoc386E0
        } else {
            BoomerangBossSkewerPiece5WaitNextLoc386CE::WaitLoopLoc386CE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_5_launch_loc_386E0()
-> BoomerangBossSkewerPiece5LaunchLoc386E0 {
    // ASM loc_386E0 (kid.asm:40896): set y_vel to `$FFF9` (word = -7) and
    // x_vel to `$FFFF6000` (longword = -0xA000 = -40960, leftward). Then
    // enter the `loc_386EE` gravity loop: each frame adds `$1B58` to y_vel
    // (longword), checks `y_pos` against `Level_height_pixels`, and deletes
    // the object when it falls off-screen. `loc_38708` (kid.asm:40909) is
    // the loop-back branch: `bra.s loc_386EE` when
    // `y_pos <= Level_height_pixels`. Piece 5 launches up-left (mirrors
    // piece 4's rightward `$C000`).
    BoomerangBossSkewerPiece5LaunchLoc386E0 {
        y_vel: -7,                        // $FFF9 as signed word
        x_vel: -0xA000_i32,               // $FFFF6000 as signed longword (leftward)
        gravity_per_frame: 0x1B58,        // addi.l #$1B58,y_vel(a3)
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38708: true,       // bra.s loc_386EE when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_6_init_loc_3870A(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossSkewerPiece6InitLoc3870A {
    // ASM loc_3870A (kid.asm:40913): allocate skewer piece 6 with gfx slot
    // `$01010002`, priority 1, object_meta `$22`. Calls `loc_32146(pc)` for
    // shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C85F0-Data_Index` (same as
    // pieces 2-5), copies anchor from `$44/$46(a5)`, and starts `unk_39872`
    // script at index 1.
    BoomerangBossSkewerPiece6InitLoc3870A {
        gfx_slot: 0x0101_0002,
        priority: 1,
        object_meta: 0x22,
        has_kid_collision: false,
        display_enabled: true, // st $13(a3)
        is_moved: true,        // st is_moved(a3)
        sprite_label: "LnkTo_unk_C85F0-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        movement_script_label: "unk_39872",
        movement_script_index: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_6_wait_loop_loc_3874C(
    head_death_counter_fb4e: u8,
) -> BoomerangBossSkewerPiece6WaitLoopLoc3874C {
    // ASM loc_3874C (kid.asm:40930): hibernate one frame, run `sub_39846`
    // (boss head movement step), then check `cmpi.b #3,($FFFFFB4E).w`.
    // Loop back to loc_3874C unless all 3 heads are dead ($FFFFFB4E == 3),
    // then branch to `loc_3875E` for piece 6 launch.
    BoomerangBossSkewerPiece6WaitLoopLoc3874C {
        hibernates_one_frame: true,
        runs_movement_step_sub_39846: true,
        next: if head_death_counter_fb4e == 3 {
            BoomerangBossSkewerPiece6WaitNextLoc3874C::LaunchLoc3875E
        } else {
            BoomerangBossSkewerPiece6WaitNextLoc3874C::WaitLoopLoc3874C
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_6_launch_loc_3875E()
-> BoomerangBossSkewerPiece6LaunchLoc3875E {
    // ASM loc_3875E (kid.asm:40938): set y_vel to `$FFF9` (word = -7) and
    // x_vel to `$A000` (longword = +0xA000 = +40960, rightward). Then
    // enter the `loc_3876C` gravity loop: each frame adds `$1B58` to y_vel
    // (longword), checks `y_pos` against `Level_height_pixels`, and deletes
    // the object when it falls off-screen. `loc_38786` (kid.asm:40951) is
    // the loop-back branch: `bra.s loc_3876C` when
    // `y_pos <= Level_height_pixels`. Piece 6 launches up-right (mirrors
    // piece 5's leftward `$FFFF6000`).
    BoomerangBossSkewerPiece6LaunchLoc3875E {
        y_vel: -7,                        // $FFF9 as signed word
        x_vel: 0xA000_i32,                // $A000 as signed longword (rightward)
        gravity_per_frame: 0x1B58,        // addi.l #$1B58,y_vel(a3)
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38786: true,       // bra.s loc_3876C when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_7_init_loc_38788(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossSkewerPiece7InitLoc38788 {
    // ASM loc_38788 (kid.asm:40955): allocate skewer piece 7 with gfx slot
    // `$01010002`, priority 1, object_meta `$22`. Calls `loc_32146(pc)` for
    // shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C85F0-Data_Index` (same as
    // pieces 2-6), copies anchor from `$44/$46(a5)`, and starts `unk_39872`
    // script at index 1.
    BoomerangBossSkewerPiece7InitLoc38788 {
        gfx_slot: 0x0101_0002,
        priority: 1,
        object_meta: 0x22,
        has_kid_collision: false,
        display_enabled: true, // st $13(a3)
        is_moved: true,        // st is_moved(a3)
        sprite_label: "LnkTo_unk_C85F0-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        movement_script_label: "unk_39872",
        movement_script_index: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_7_wait_loop_loc_387CA(
    head_death_counter_fb4e: u8,
) -> BoomerangBossSkewerPiece7WaitLoopLoc387CA {
    // ASM loc_387CA (kid.asm:40972): hibernate one frame, run `sub_39846`
    // (boss head movement step), then check `cmpi.b #3,($FFFFFB4E).w`.
    // Loop back to loc_387CA unless all 3 heads are dead ($FFFFFB4E == 3),
    // then branch to `loc_387DC` for piece 7 launch.
    BoomerangBossSkewerPiece7WaitLoopLoc387CA {
        hibernates_one_frame: true,
        runs_movement_step_sub_39846: true,
        next: if head_death_counter_fb4e == 3 {
            BoomerangBossSkewerPiece7WaitNextLoc387CA::LaunchLoc387DC
        } else {
            BoomerangBossSkewerPiece7WaitNextLoc387CA::WaitLoopLoc387CA
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_7_launch_loc_387DC()
-> BoomerangBossSkewerPiece7LaunchLoc387DC {
    // ASM loc_387DC (kid.asm:40980): set y_vel to `$FFF9` (word = -7) and
    // x_vel to `$FFFF8000` (longword = -0x8000 = -32768, leftward). Then
    // enter the `loc_387EA` gravity loop: each frame adds `$1B58` to y_vel
    // (longword), checks `y_pos` against `Level_height_pixels`, and deletes
    // the object when it falls off-screen. `loc_38804` (kid.asm:40993) is
    // the loop-back branch: `bra.s loc_387EA` when
    // `y_pos <= Level_height_pixels`. Piece 7 launches up-left (mirrors
    // piece 6's rightward `$A000`).
    BoomerangBossSkewerPiece7LaunchLoc387DC {
        y_vel: -7,                        // $FFF9 as signed word
        x_vel: -0x8000_i32,               // $FFFF8000 as signed longword (leftward)
        gravity_per_frame: 0x1B58,        // addi.l #$1B58,y_vel(a3)
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38804: true,       // bra.s loc_387EA when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_8_init_loc_38806(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossSkewerPiece8InitLoc38806 {
    // ASM loc_38806 (kid.asm:40997): allocate skewer piece 8 with gfx slot
    // `$01010002`, priority 1, object_meta `$22`. Calls `loc_32146(pc)` for
    // shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C85F0-Data_Index` (same as
    // pieces 2-7), copies anchor from `$44/$46(a5)`, and starts `unk_39872`
    // script at index 1.
    BoomerangBossSkewerPiece8InitLoc38806 {
        gfx_slot: 0x0101_0002,
        priority: 1,
        object_meta: 0x22,
        has_kid_collision: false,
        display_enabled: true, // st $13(a3)
        is_moved: true,        // st is_moved(a3)
        sprite_label: "LnkTo_unk_C85F0-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        movement_script_label: "unk_39872",
        movement_script_index: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_8_wait_loop_loc_38848(
    head_death_counter_fb4e: u8,
) -> BoomerangBossSkewerPiece8WaitLoopLoc38848 {
    // ASM loc_38848 (kid.asm:41014): hibernate one frame, run `sub_39846`
    // (boss head movement step), then check `cmpi.b #3,($FFFFFB4E).w`.
    // Loop back to loc_38848 unless all 3 heads are dead ($FFFFFB4E == 3),
    // then branch to `loc_3885A` for piece 8 launch.
    BoomerangBossSkewerPiece8WaitLoopLoc38848 {
        hibernates_one_frame: true,
        runs_movement_step_sub_39846: true,
        next: if head_death_counter_fb4e == 3 {
            BoomerangBossSkewerPiece8WaitNextLoc38848::LaunchLoc3885A
        } else {
            BoomerangBossSkewerPiece8WaitNextLoc38848::WaitLoopLoc38848
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_8_launch_loc_3885A()
-> BoomerangBossSkewerPiece8LaunchLoc3885A {
    // ASM loc_3885A (kid.asm:41022): set y_vel to `$FFF8` (word = -8) and
    // x_vel to `$8000` (longword = 0x8000 = 32768, rightward). Then enter
    // the `loc_38868` gravity loop: each frame adds `$1B58` to y_vel
    // (longword), checks `y_pos` against `Level_height_pixels`, and deletes
    // the object when it falls off-screen. `loc_38882` (kid.asm:41035) is
    // the loop-back branch: `bra.s loc_38868` when
    // `y_pos <= Level_height_pixels`. Piece 8 launches up-right (mirrors
    // piece 7's leftward `$FFFF8000`).
    BoomerangBossSkewerPiece8LaunchLoc3885A {
        y_vel: -8,                        // $FFF8 as signed word
        x_vel: 0x8000_i32,                // $00008000 as signed longword (rightward)
        gravity_per_frame: 0x1B58,        // addi.l #$1B58,y_vel(a3)
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38882: true,       // bra.s loc_38868 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_9_init_loc_38884(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossSkewerPiece9InitLoc38884 {
    // ASM loc_38884 (kid.asm:41039): allocate skewer piece 9 with gfx slot
    // `$01010002`, priority 1, object_meta `$22`. Calls `loc_32146(pc)` for
    // shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C85E8-Data_Index` (differs
    // from pieces 2-8 which use C85F0), copies anchor from `$44/$46(a5)`,
    // and starts `unk_39872` script at index 1.
    BoomerangBossSkewerPiece9InitLoc38884 {
        gfx_slot: 0x0101_0002,
        priority: 1,
        object_meta: 0x22,
        has_kid_collision: false,
        display_enabled: true, // st $13(a3)
        is_moved: true,        // st is_moved(a3)
        sprite_label: "LnkTo_unk_C85E8-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        movement_script_label: "unk_39872",
        movement_script_index: 1,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_9_wait_loop_loc_388C6(
    head_death_counter_fb4e: u8,
) -> BoomerangBossSkewerPiece9WaitLoopLoc388C6 {
    // ASM loc_388C6 (kid.asm:41056): hibernate one frame, run `sub_39846`
    // (boss head movement step), then check `cmpi.b #3,($FFFFFB4E).w`.
    // Loop back to loc_388C6 unless all 3 heads are dead ($FFFFFB4E == 3),
    // then branch to `loc_388D8` for piece 9 launch.
    BoomerangBossSkewerPiece9WaitLoopLoc388C6 {
        hibernates_one_frame: true,
        runs_movement_step_sub_39846: true,
        next: if head_death_counter_fb4e == 3 {
            BoomerangBossSkewerPiece9WaitNextLoc388C6::LaunchLoc388D8
        } else {
            BoomerangBossSkewerPiece9WaitNextLoc388C6::WaitLoopLoc388C6
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_skewer_piece_9_launch_loc_388D8()
-> BoomerangBossSkewerPiece9LaunchLoc388D8 {
    // ASM loc_388D8 (kid.asm:41064): set y_vel to `$FFFA` (word = -6) and
    // x_vel to `$FFFF6000` (longword = -0xA000 = -40960, leftward). Then
    // enter the `loc_388E6` gravity loop: each frame adds `$1B58` to y_vel
    // (longword), checks `y_pos` against `Level_height_pixels`, and deletes
    // the object when it falls off-screen. `loc_38900` (kid.asm:41077) is
    // the loop-back branch: `bra.s loc_388E6` when
    // `y_pos <= Level_height_pixels`. Piece 9 launches up-left (mirrors
    // piece 8's rightward `$8000`).
    BoomerangBossSkewerPiece9LaunchLoc388D8 {
        y_vel: -6,                        // $FFFA as signed word
        x_vel: -0xA000_i32,               // $FFFF6000 as signed longword (leftward)
        gravity_per_frame: 0x1B58,        // addi.l #$1B58,y_vel(a3)
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38900: true,       // bra.s loc_388E6 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_eye_piece_1_init_loc_38902(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossEyePiece1InitLoc38902 {
    // ASM loc_38902 (kid.asm:41081): allocate eye piece 1 with gfx slot
    // `$01020002`, priority 0, object_meta `$21`. Calls `loc_32146(pc)` for
    // shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_393C8).
    BoomerangBossEyePiece1InitLoc38902 {
        gfx_slot: 0x0102_0002,
        priority: 0,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_eye_piece_1_tracking_loop_loc_3893E(
    breakup_trigger_fb72: u16,
) -> BoomerangBossEyePiece1TrackingLoopLoc3893E {
    // ASM loc_3893E (kid.asm:41096): hibernate one frame, load parent pointer
    // from `($FFFFFB7A).w` into `a2`, load direction byte from `($FFFFFB86).w`
    // into `d2`, run `sub_393C8` (orbit placement) and `sub_37A22` (directional
    // sprite selection), store `d6` into `addroffset_sprite(a3)`. Then check
    // `tst.w ($FFFFFB72).w`: nonzero branches to `loc_38962` (launch); zero
    // loads `$16(a2)` into `d0` (parent frame byte) and loops back.
    BoomerangBossEyePiece1TrackingLoopLoc3893E {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_393c8: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb72 != 0 {
            BoomerangBossEyePiece1TrackingNextLoc3893E::LaunchLoc38962
        } else {
            BoomerangBossEyePiece1TrackingNextLoc3893E::TrackingLoopLoc3893E
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_eye_piece_1_launch_loc_38962() -> BoomerangBossEyePiece1LaunchLoc38962
{
    // ASM loc_38962 (kid.asm:41109): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `$FFFFC000` (longword = -0x4000 = -16384, leftward). Then enter the
    // `loc_38976` gravity loop: each frame runs `sub_37A22` (directional sprite
    // update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_38998` (kid.asm:41125) is the loop-back: `bra.s loc_38976` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossEyePiece1LaunchLoc38962 {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: -0x4000_i32,                          // $FFFFC000 as signed longword (leftward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38976
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38998: true,       // bra.s loc_38976 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_eye_piece_2_init_loc_3899A(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossEyePiece2InitLoc3899A {
    // ASM loc_3899A (kid.asm:41129): allocate eye piece 2 with gfx slot
    // `$01030002`, priority 0, object_meta `$21`. Calls `loc_32146(pc)` for
    // shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_39420).
    BoomerangBossEyePiece2InitLoc3899A {
        gfx_slot: 0x0103_0002,
        priority: 0,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_eye_piece_2_tracking_loop_loc_389D6(
    breakup_trigger_fb72: u16,
) -> BoomerangBossEyePiece2TrackingLoopLoc389D6 {
    // ASM loc_389D6 (kid.asm:41144): hibernate one frame, load parent pointer
    // from `($FFFFFB7A).w` into `a2`, load direction byte from `($FFFFFB86).w`
    // into `d2`, run `sub_39420` (orbit placement) and `sub_37A22` (directional
    // sprite selection), store `d6` into `addroffset_sprite(a3)`. Then check
    // `tst.w ($FFFFFB72).w`: nonzero branches to `loc_389FA` (launch); zero
    // loads `$16(a2)` into `d0` (parent frame byte) and loops back.
    BoomerangBossEyePiece2TrackingLoopLoc389D6 {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_39420: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb72 != 0 {
            BoomerangBossEyePiece2TrackingNextLoc389D6::LaunchLoc389FA
        } else {
            BoomerangBossEyePiece2TrackingNextLoc389D6::TrackingLoopLoc389D6
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_eye_piece_2_launch_loc_389FA() -> BoomerangBossEyePiece2LaunchLoc389FA
{
    // ASM loc_389FA (kid.asm:41157): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `$4000` (longword = +0x4000 = 16384, rightward). Then enter the
    // `loc_38A0E` gravity loop: each frame runs `sub_37A22` (directional sprite
    // update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_38A30` (kid.asm:41173) is the loop-back: `bra.s loc_38A0E` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossEyePiece2LaunchLoc389FA {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: 0x4000_i32,                           // $4000 as signed longword (rightward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38A0E
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38a30: true,       // bra.s loc_38A0E when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_1_init_loc_38A32(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead2EyePiece1InitLoc38A32 {
    // ASM loc_38A32 (kid.asm:41177): allocate head 2's eye piece 1 with gfx
    // slot `$01020002`, priority 0, object_meta `$21`. Calls `loc_32146(pc)`
    // for shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_393C8). Identical layout to head 1's eye piece 1
    // (loc_38902) but tracked by head 2's parent pointer `$FFFFFB7E`.
    BoomerangBossHead2EyePiece1InitLoc38A32 {
        gfx_slot: 0x0102_0002,
        priority: 0,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_1_tracking_loop_loc_38A6E(
    breakup_trigger_fb74: u16,
) -> BoomerangBossHead2EyePiece1TrackingLoopLoc38A6E {
    // ASM loc_38A6E (kid.asm:41192): hibernate one frame, load parent pointer
    // from `($FFFFFB7E).w` into `a2`, load direction byte from `($FFFFFB87).w`
    // into `d2`, run `sub_393C8` (orbit placement) and `sub_37A22` (directional
    // sprite selection), store `d6` into `addroffset_sprite(a3)`. Then check
    // `tst.w ($FFFFFB74).w`: nonzero branches to `loc_38A92` (launch); zero
    // loads `$16(a2)` into `d0` (parent frame byte) and loops back.
    BoomerangBossHead2EyePiece1TrackingLoopLoc38A6E {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_393c8: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb74 != 0 {
            BoomerangBossHead2EyePiece1TrackingNextLoc38A6E::LaunchLoc38A92
        } else {
            BoomerangBossHead2EyePiece1TrackingNextLoc38A6E::TrackingLoopLoc38A6E
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_1_launch_loc_38A92()
-> BoomerangBossHead2EyePiece1LaunchLoc38A92 {
    // ASM loc_38A92 (kid.asm:41205): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `$FFFFC000` (longword = -0x4000 = -16384, leftward). Then enter the
    // `loc_38AA6` gravity loop: each frame runs `sub_37A22` (directional sprite
    // update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_38AC8` (kid.asm:41221) is the loop-back: `bra.s loc_38AA6` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead2EyePiece1LaunchLoc38A92 {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: -0x4000_i32,                          // $FFFFC000 as signed longword (leftward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38AA6
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38ac8: true,       // bra.s loc_38AA6 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_2_init_loc_38ACA(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead2EyePiece2InitLoc38ACA {
    // ASM loc_38ACA (kid.asm:41225): allocate head 2's eye piece 2 with gfx
    // slot `$01030002`, priority 0, object_meta `$21`. Calls `loc_32146(pc)`
    // for shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_39420). Identical layout to head 1's eye piece 2
    // (loc_3899A) but tracked by head 2's parent pointer `$FFFFFB7E`.
    BoomerangBossHead2EyePiece2InitLoc38ACA {
        gfx_slot: 0x0103_0002,
        priority: 0,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_2_tracking_loop_loc_38B06(
    breakup_trigger_fb74: u16,
) -> BoomerangBossHead2EyePiece2TrackingLoopLoc38B06 {
    // ASM loc_38B06 (kid.asm:41240): hibernate one frame, load parent pointer
    // from `($FFFFFB7E).w` into `a2`, load direction byte from `($FFFFFB87).w`
    // into `d2`, run `sub_39420` (orbit placement) and `sub_37A22` (directional
    // sprite selection), store `d6` into `addroffset_sprite(a3)`. Then check
    // `tst.w ($FFFFFB74).w`: nonzero branches to `loc_38B2A` (launch); zero
    // loads `$16(a2)` into `d0` (parent frame byte) and loops back.
    BoomerangBossHead2EyePiece2TrackingLoopLoc38B06 {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_39420: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb74 != 0 {
            BoomerangBossHead2EyePiece2TrackingNextLoc38B06::LaunchLoc38B2A
        } else {
            BoomerangBossHead2EyePiece2TrackingNextLoc38B06::TrackingLoopLoc38B06
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_2_launch_loc_38B2A()
-> BoomerangBossHead2EyePiece2LaunchLoc38B2A {
    // ASM loc_38B2A (kid.asm:41253): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `$4000` (longword = +0x4000 = +16384, rightward). Then enter the
    // `loc_38B3E` gravity loop: each frame runs `sub_37A22` (directional sprite
    // update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_38B60` (kid.asm:41269) is the loop-back: `bra.s loc_38B3E` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead2EyePiece2LaunchLoc38B2A {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: 0x4000_i32,                           // $4000 as signed longword (rightward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38B3E
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38b60: true,       // bra.s loc_38B3E when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_1_init_loc_38B62(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead3EyePiece1InitLoc38B62 {
    // ASM loc_38B62 (kid.asm:41273): allocate head 3's eye piece 1 with gfx
    // slot `$01020002`, priority 0, object_meta `$21`. Calls `loc_32146(pc)`
    // for shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_393C8). Identical layout to head 2's eye piece 1
    // (loc_38A32) but tracked by head 3's parent pointer `$FFFFFB82`.
    BoomerangBossHead3EyePiece1InitLoc38B62 {
        gfx_slot: 0x0102_0002,
        priority: 0,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_1_tracking_loop_loc_38B9E(
    breakup_trigger_fb76: u16,
) -> BoomerangBossHead3EyePiece1TrackingLoopLoc38B9E {
    // ASM loc_38B9E (kid.asm:41288): hibernate one frame, load parent pointer
    // from `($FFFFFB82).w` into `a2`, load direction byte from `($FFFFFB88).w`
    // into `d2`, run `sub_393C8` (orbit placement) and `sub_37A22` (directional
    // sprite selection), store `d6` into `addroffset_sprite(a3)`. Then check
    // `tst.w ($FFFFFB76).w`: nonzero branches to `loc_38BC2` (launch); zero
    // loads `$16(a2)` into `d0` (parent frame byte) and loops back.
    BoomerangBossHead3EyePiece1TrackingLoopLoc38B9E {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_393c8: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb76 != 0 {
            BoomerangBossHead3EyePiece1TrackingNextLoc38B9E::LaunchLoc38BC2
        } else {
            BoomerangBossHead3EyePiece1TrackingNextLoc38B9E::TrackingLoopLoc38B9E
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_1_launch_loc_38BC2()
-> BoomerangBossHead3EyePiece1LaunchLoc38BC2 {
    // ASM loc_38BC2 (kid.asm:41301): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#-$4000` (longword = -0x4000 = -16384, leftward). Then enter the
    // `loc_38BD6` gravity loop: each frame runs `sub_37A22` (directional sprite
    // update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_38BF8` (kid.asm:41317) is the loop-back: `bra.s loc_38BD6` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead3EyePiece1LaunchLoc38BC2 {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: -0x4000_i32,                          // $FFFFC000 as signed longword (leftward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38BD6
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38bf8: true,       // bra.s loc_38BD6 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_2_init_loc_38BFA(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead3EyePiece2InitLoc38BFA {
    // ASM loc_38BFA (kid.asm:41321): allocate head 3's eye piece 2 with gfx
    // slot `$01030002`, priority 0, object_meta `$21`. Calls `loc_32146(pc)`
    // for shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_39420). Mirrors head 3's eye piece 1
    // (loc_38B62) but uses gfx slot `$01030002` and sub_39420 orbit.
    BoomerangBossHead3EyePiece2InitLoc38BFA {
        gfx_slot: 0x0103_0002,
        priority: 0,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_2_tracking_loop_loc_38C36(
    breakup_trigger_fb76: u16,
) -> BoomerangBossHead3EyePiece2TrackingLoopLoc38C36 {
    // ASM loc_38C36 (kid.asm:41336): hibernate one frame, load parent pointer
    // from `($FFFFFB82).w` into `a2`, load direction byte from `($FFFFFB88).w`
    // into `d2`, run `sub_39420` (orbit placement) and `sub_37A22` (directional
    // sprite selection), store `d6` into `addroffset_sprite(a3)`. Then check
    // `tst.w ($FFFFFB76).w`: nonzero branches to `loc_38C5A` (launch); zero
    // loads `$16(a2)` into `d0` (parent frame byte) and loops back.
    BoomerangBossHead3EyePiece2TrackingLoopLoc38C36 {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_39420: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb76 != 0 {
            BoomerangBossHead3EyePiece2TrackingNextLoc38C36::LaunchLoc38C5A
        } else {
            BoomerangBossHead3EyePiece2TrackingNextLoc38C36::TrackingLoopLoc38C36
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_2_launch_loc_38C5A()
-> BoomerangBossHead3EyePiece2LaunchLoc38C5A {
    // ASM loc_38C5A (kid.asm:41349): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$4000` (longword = +0x4000 = +16384, rightward — mirrors piece 1's
    // leftward `$FFFFC000`). Then enter the `loc_38C6E` gravity loop: each
    // frame runs `sub_37A22` (directional sprite update), adds `$7D0` to y_vel
    // (longword), checks `y_pos` against `Level_height_pixels`, and deletes
    // the object when it falls off-screen. `loc_38C90` (kid.asm:41365) is the
    // loop-back: `bra.s loc_38C6E` when `y_pos <= Level_height_pixels`.
    BoomerangBossHead3EyePiece2LaunchLoc38C5A {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: 0x4000_i32,                           // $4000 as signed longword (rightward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38C6E
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38c90: true,       // bra.s loc_38C6E when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_3_init_loc_38C92(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead1EyePiece3InitLoc38C92 {
    // ASM loc_38C92 (kid.asm:41369): allocate head 1's eye piece 3 with gfx
    // slot `$01030002`, priority 1, object_meta `$21`. Calls `loc_32146(pc)`
    // for shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_393C8). Uses head 1's parent pointer
    // `($FFFFFB7A)`, direction `($FFFFFB86)`, and breakup `($FFFFFB72)` but
    // priority 1 (unlike head 1's priority-0 pieces at loc_38902/loc_3899A).
    BoomerangBossHead1EyePiece3InitLoc38C92 {
        gfx_slot: 0x0103_0002,
        priority: 1,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_3_tracking_loop_loc_38CCE(
    breakup_trigger_fb72: u16,
) -> BoomerangBossHead1EyePiece3TrackingLoopLoc38CCE {
    // ASM loc_38CCE (kid.asm:41384): hibernate one frame, load parent pointer
    // from `($FFFFFB7A).w` into `a2`, load direction byte from `($FFFFFB86).w`
    // into `d2`, run `sub_393C8` (orbit placement) and `sub_37A22` (directional
    // sprite selection), store `d6` into `addroffset_sprite(a3)`. Then check
    // `tst.w ($FFFFFB72).w`: nonzero branches to `loc_38CF2` (launch); zero
    // loads `$16(a2)` into `d0` (parent frame byte) and loops back.
    BoomerangBossHead1EyePiece3TrackingLoopLoc38CCE {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_393c8: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb72 != 0 {
            BoomerangBossHead1EyePiece3TrackingNextLoc38CCE::LaunchLoc38CF2
        } else {
            BoomerangBossHead1EyePiece3TrackingNextLoc38CCE::TrackingLoopLoc38CCE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_3_launch_loc_38CF2()
-> BoomerangBossHead1EyePiece3LaunchLoc38CF2 {
    // ASM loc_38CF2 (kid.asm:41397): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$FFFFC000` (longword = -0x4000 = -16384, leftward — matches eye
    // piece 1's leftward direction at loc_38962). Then enter the `loc_38D06`
    // gravity loop: each frame runs `sub_37A22` (directional sprite update),
    // adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_38D28` (kid.asm:41413) is the loop-back: `bra.s loc_38D06` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead1EyePiece3LaunchLoc38CF2 {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: -0x4000_i32,                          // $FFFFC000 as signed longword (leftward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38D06
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38d28: true,       // bra.s loc_38D06 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_4_init_loc_38D2A(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead1EyePiece4InitLoc38D2A {
    // ASM loc_38D2A (kid.asm:41417): allocate head 1's eye piece 4 with gfx
    // slot `$01030002`, priority 1, object_meta `$21`. Calls `loc_32146(pc)`
    // for shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_39420). Uses head 1's parent pointer
    // `($FFFFFB7A)`, direction `($FFFFFB86)`, and breakup `($FFFFFB72)`.
    // Priority 1 (secondary wave, same as piece 3 at loc_38C92). Uses
    // sub_39420 (mirrors piece 3's sub_393C8).
    BoomerangBossHead1EyePiece4InitLoc38D2A {
        gfx_slot: 0x0103_0002,
        priority: 1,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_4_tracking_loop_loc_38D66(
    breakup_trigger_fb72: u16,
) -> BoomerangBossHead1EyePiece4TrackingLoopLoc38D66 {
    // ASM loc_38D66 (kid.asm:41432): hibernate one frame, load parent pointer
    // from `($FFFFFB7A).w` into `a2`, load direction byte from `($FFFFFB86).w`
    // into `d2`, run `sub_39420` (orbit placement — mirrors piece 3's
    // sub_393C8) and `sub_37A22` (directional sprite selection), store `d6`
    // into `addroffset_sprite(a3)`. Then check `tst.w ($FFFFFB72).w`: nonzero
    // branches to `loc_38D8A` (launch); zero loads `$16(a2)` into `d0`
    // (parent frame byte) and loops back.
    BoomerangBossHead1EyePiece4TrackingLoopLoc38D66 {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_39420: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb72 != 0 {
            BoomerangBossHead1EyePiece4TrackingNextLoc38D66::LaunchLoc38D8A
        } else {
            BoomerangBossHead1EyePiece4TrackingNextLoc38D66::TrackingLoopLoc38D66
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_4_launch_loc_38D8A()
-> BoomerangBossHead1EyePiece4LaunchLoc38D8A {
    // ASM loc_38D8A (kid.asm:41445): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$4000` (longword = +0x4000 = +16384, rightward — mirrors eye
    // piece 3's leftward direction at loc_38CF2). Then enter the `loc_38D9E`
    // gravity loop: each frame runs `sub_37A22` (directional sprite update),
    // adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_38DC0` (kid.asm:41461) is the loop-back: `bra.s loc_38D9E` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead1EyePiece4LaunchLoc38D8A {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: 0x4000_i32,                           // $4000 as signed longword (rightward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38D9E
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38dc0: true,       // bra.s loc_38D9E when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_3_init_loc_38DC2(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead2EyePiece3InitLoc38DC2 {
    // ASM loc_38DC2 (kid.asm:41465): allocate head 2's eye piece 3 with gfx
    // slot `$01030002`, priority 1, object_meta `$21`. Calls `loc_32146(pc)`
    // for shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_393C8). Uses head 2's parent pointer
    // `($FFFFFB7E)`, direction `($FFFFFB87)`, and breakup `($FFFFFB74)`.
    // Priority 1 (secondary wave, same as head 1's piece 3 at loc_38C92).
    BoomerangBossHead2EyePiece3InitLoc38DC2 {
        gfx_slot: 0x0103_0002,
        priority: 1,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_3_tracking_loop_loc_38DFE(
    breakup_trigger_fb74: u16,
) -> BoomerangBossHead2EyePiece3TrackingLoopLoc38DFE {
    // ASM loc_38DFE (kid.asm:41480): hibernate one frame, load parent pointer
    // from `($FFFFFB7E).w` into `a2`, load direction byte from `($FFFFFB87).w`
    // into `d2`, run `sub_393C8` (orbit placement) and `sub_37A22` (directional
    // sprite selection), store `d6` into `addroffset_sprite(a3)`. Then check
    // `tst.w ($FFFFFB74).w`: nonzero branches to `loc_38E22` (launch); zero
    // loads `$16(a2)` into `d0` (parent frame byte) and loops back.
    BoomerangBossHead2EyePiece3TrackingLoopLoc38DFE {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_393c8: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb74 != 0 {
            BoomerangBossHead2EyePiece3TrackingNextLoc38DFE::LaunchLoc38E22
        } else {
            BoomerangBossHead2EyePiece3TrackingNextLoc38DFE::TrackingLoopLoc38DFE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_3_launch_loc_38E22()
-> BoomerangBossHead2EyePiece3LaunchLoc38E22 {
    // ASM loc_38E22 (kid.asm:41493): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$FFFFC000` (longword = -0x4000 = -16384, leftward — same direction
    // as head 2's eye piece 1 at loc_38A92). Then enter the `loc_38E36`
    // gravity loop: each frame runs `sub_37A22` (directional sprite update),
    // adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_38E58` (kid.asm:41509) is the loop-back: `bra.s loc_38E36` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead2EyePiece3LaunchLoc38E22 {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: -0x4000_i32,                          // $FFFFC000 as signed longword (leftward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38E36
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38e58: true,       // bra.s loc_38E36 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_4_init_loc_38E5A(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead2EyePiece4InitLoc38E5A {
    // ASM loc_38E5A (kid.asm:41513): allocate head 2's eye piece 4 with gfx
    // slot `$01030002`, priority 1, object_meta `$21`. Calls `loc_32146(pc)`
    // for shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_39420). Uses head 2's parent pointer
    // `($FFFFFB7E)`, direction `($FFFFFB87)`, and breakup `($FFFFFB74)`.
    // Priority 1 (secondary wave, same as piece 3 at loc_38DC2). Uses
    // sub_39420 (mirrors piece 3's sub_393C8).
    BoomerangBossHead2EyePiece4InitLoc38E5A {
        gfx_slot: 0x0103_0002,
        priority: 1,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_4_tracking_loop_loc_38E96(
    breakup_trigger_fb74: u16,
) -> BoomerangBossHead2EyePiece4TrackingLoopLoc38E96 {
    // ASM loc_38E96 (kid.asm:41528): hibernate one frame, load parent pointer
    // from `($FFFFFB7E).w` into `a2`, load direction byte from `($FFFFFB87).w`
    // into `d2`, run `sub_39420` (orbit placement — mirrors piece 3's
    // sub_393C8) and `sub_37A22` (directional sprite selection), store `d6`
    // into `addroffset_sprite(a3)`. Then check `tst.w ($FFFFFB74).w`: nonzero
    // branches to `loc_38EBA` (launch); zero loads `$16(a2)` into `d0`
    // (parent frame byte) and loops back.
    BoomerangBossHead2EyePiece4TrackingLoopLoc38E96 {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_39420: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb74 != 0 {
            BoomerangBossHead2EyePiece4TrackingNextLoc38E96::LaunchLoc38EBA
        } else {
            BoomerangBossHead2EyePiece4TrackingNextLoc38E96::TrackingLoopLoc38E96
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_4_launch_loc_38EBA()
-> BoomerangBossHead2EyePiece4LaunchLoc38EBA {
    // ASM loc_38EBA (kid.asm:41541): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$4000` (longword = +0x4000 = +16384, rightward — mirrors eye
    // piece 3's leftward direction at loc_38E22). Then enter the `loc_38ECE`
    // gravity loop: each frame runs `sub_37A22` (directional sprite update),
    // adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_38EF0` (kid.asm:41557) is the loop-back: `bra.s loc_38ECE` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead2EyePiece4LaunchLoc38EBA {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: 0x4000_i32,                           // $4000 as signed longword (rightward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38ECE
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38ef0: true,       // bra.s loc_38ECE when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_3_init_loc_38EF2(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead3EyePiece3InitLoc38EF2 {
    // ASM loc_38EF2 (kid.asm:41561): allocate head 3's eye piece 3 with gfx
    // slot `$01030002`, priority 1, object_meta `$21`. Calls `loc_32146(pc)`
    // for shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_393C8). Uses head 3's parent pointer
    // `($FFFFFB82)`, direction `($FFFFFB88)`, and breakup `($FFFFFB76)`.
    // Priority 1 (secondary wave, same as piece 4 at loc_38F8A). Uses
    // sub_393C8 (mirrors piece 4's sub_39420).
    BoomerangBossHead3EyePiece3InitLoc38EF2 {
        gfx_slot: 0x0103_0002,
        priority: 1,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_3_tracking_loop_loc_38F2E(
    breakup_trigger_fb76: u16,
) -> BoomerangBossHead3EyePiece3TrackingLoopLoc38F2E {
    // ASM loc_38F2E (kid.asm:41576): hibernate one frame, load parent pointer
    // from `($FFFFFB82).w` into `a2`, load direction byte from `($FFFFFB88).w`
    // into `d2`, run `sub_393C8` (orbit placement) and `sub_37A22` (directional
    // sprite selection), store `d6` into `addroffset_sprite(a3)`. Then check
    // `tst.w ($FFFFFB76).w`: nonzero branches to `loc_38F52` (launch); zero
    // loads `$16(a2)` into `d0` (parent frame byte) and loops back.
    BoomerangBossHead3EyePiece3TrackingLoopLoc38F2E {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_393c8: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb76 != 0 {
            BoomerangBossHead3EyePiece3TrackingNextLoc38F2E::LaunchLoc38F52
        } else {
            BoomerangBossHead3EyePiece3TrackingNextLoc38F2E::TrackingLoopLoc38F2E
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_3_launch_loc_38F52()
-> BoomerangBossHead3EyePiece3LaunchLoc38F52 {
    // ASM loc_38F52 (kid.asm:41589): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$FFFFC000` (longword = -0x4000 = -16384, leftward — mirrors eye
    // piece 4's rightward direction at loc_38FEA). Then enter the `loc_38F66`
    // gravity loop: each frame runs `sub_37A22` (directional sprite update),
    // adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_38F88` (kid.asm:41605) is the loop-back: `bra.s loc_38F66` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead3EyePiece3LaunchLoc38F52 {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: -0x4000_i32,                          // $FFFFC000 as signed longword (leftward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38F66
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_38f88: true,       // bra.s loc_38F66 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_4_tracking_loop_loc_38FC6(
    breakup_trigger_fb76: u16,
) -> BoomerangBossHead3EyePiece4TrackingLoopLoc38FC6 {
    // ASM loc_38FC6 (kid.asm:41624): hibernate one frame, load parent pointer
    // from `($FFFFFB82).w` into `a2`, load direction byte from `($FFFFFB88).w`
    // into `d2`, run `sub_39420` (orbit placement — mirrors piece 3's
    // sub_393C8) and `sub_37A22` (directional sprite selection), store `d6`
    // into `addroffset_sprite(a3)`. Then check `tst.w ($FFFFFB76).w`: nonzero
    // branches to `loc_38FEA` (launch); zero loads `$16(a2)` into `d0` (parent
    // frame byte) and loops back.
    BoomerangBossHead3EyePiece4TrackingLoopLoc38FC6 {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_39420: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb76 != 0 {
            BoomerangBossHead3EyePiece4TrackingNextLoc38FC6::LaunchLoc38FEA
        } else {
            BoomerangBossHead3EyePiece4TrackingNextLoc38FC6::TrackingLoopLoc38FC6
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_4_launch_loc_38FEA()
-> BoomerangBossHead3EyePiece4LaunchLoc38FEA {
    // ASM loc_38FEA (kid.asm:41637): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$4000` (longword = +0x4000 = 16384, rightward — mirrors eye piece
    // 3's leftward direction at loc_38F52). Then enter the `loc_38FFE`
    // (kid.asm:41642) gravity loop: each frame runs `sub_37A22` (directional
    // sprite update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_39020` (kid.asm:41653) is the loop-back: `bra.s loc_38FFE` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead3EyePiece4LaunchLoc38FEA {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: 0x4000_i32,                           // $4000 as signed longword (rightward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_38FFE
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_39020: true,       // bra.s loc_38FFE when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_5_init_loc_39022(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead1EyePiece5InitLoc39022 {
    // ASM loc_39022 (kid.asm:41657): allocate head 1's eye piece 5 with gfx
    // slot `$01030002`, priority 0, object_meta `$21`. Calls `loc_32146(pc)`
    // for shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_393C8). Uses head 1's parent pointer
    // `($FFFFFB7A)`, direction `($FFFFFB86)`, and breakup `($FFFFFB72)`.
    // Priority 0 (tertiary wave). Uses sub_393C8 (mirrors piece 6's sub_39420).
    BoomerangBossHead1EyePiece5InitLoc39022 {
        gfx_slot: 0x0103_0002,
        priority: 0,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_5_tracking_loop_loc_3905E(
    breakup_trigger_fb72: u16,
) -> BoomerangBossHead1EyePiece5TrackingLoopLoc3905E {
    // ASM loc_3905E (kid.asm:41672): hibernate one frame, load parent pointer
    // from `($FFFFFB7A).w` into `a2`, load direction byte from `($FFFFFB86).w`
    // into `d2`, run `sub_393C8` (orbit placement) and `sub_37A22` (directional
    // sprite selection), store `d6` into `addroffset_sprite(a3)`. Then check
    // `tst.w ($FFFFFB72).w`: nonzero branches to `loc_39082` (launch); zero
    // loads `$16(a2)` into `d0` (parent frame byte) and loops back.
    BoomerangBossHead1EyePiece5TrackingLoopLoc3905E {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_393c8: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb72 != 0 {
            BoomerangBossHead1EyePiece5TrackingNextLoc3905E::LaunchLoc39082
        } else {
            BoomerangBossHead1EyePiece5TrackingNextLoc3905E::TrackingLoopLoc3905E
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_5_launch_loc_39082()
-> BoomerangBossHead1EyePiece5LaunchLoc39082 {
    // ASM loc_39082 (kid.asm:41685): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$FFFFC000` (longword = -0x4000 = -16384, leftward — mirrors eye
    // piece 6's rightward direction at loc_390BA). Then enter the `loc_39096`
    // (kid.asm:41690) gravity loop: each frame runs `sub_37A22` (directional
    // sprite update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_390B8` (kid.asm:41701) is the loop-back: `bra.s loc_39096` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead1EyePiece5LaunchLoc39082 {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: -0x4000_i32,                          // $FFFFC000 as signed longword (leftward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_39096
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_390b8: true,       // bra.s loc_39096 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_6_init_loc_390ba(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead1EyePiece6InitLoc390BA {
    // ASM loc_390BA (kid.asm:41705): allocate head 1's eye piece 6 with gfx
    // slot `$01030002`, priority 0, object_meta `$21`. Calls `loc_32146(pc)`
    // for shared init, clears `has_kid_collision(a3)`, sets `$13(a3)` and
    // `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`, copies
    // anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame byte
    // for orbit placement sub_39420). Uses head 1's parent pointer
    // `($FFFFFB7A)`, direction `($FFFFFB86)`, and breakup `($FFFFFB72)`.
    // Priority 0 (tertiary wave). Uses sub_39420 (mirrors piece 5's sub_393C8).
    BoomerangBossHead1EyePiece6InitLoc390BA {
        gfx_slot: 0x0103_0002,
        priority: 0,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_6_tracking_loop_loc_390f6(
    breakup_trigger_fb72: u16,
) -> BoomerangBossHead1EyePiece6TrackingLoopLoc390F6 {
    // ASM loc_390F6 (kid.asm:41720): hibernate one frame, load parent pointer
    // from `($FFFFFB7A).w` into `a2`, load direction byte from `($FFFFFB86).w`
    // into `d2`, run `sub_39420` (orbit placement) and `sub_37A22` (directional
    // sprite selection), store `d6` into `addroffset_sprite(a3)`. Then check
    // `tst.w ($FFFFFB72).w`: nonzero branches to `loc_3911A` (launch); zero
    // loads `$16(a2)` into `d0` (parent frame byte) and loops back.
    BoomerangBossHead1EyePiece6TrackingLoopLoc390F6 {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_39420: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb72 != 0 {
            BoomerangBossHead1EyePiece6TrackingNextLoc390F6::LaunchLoc3911A
        } else {
            BoomerangBossHead1EyePiece6TrackingNextLoc390F6::TrackingLoopLoc390F6
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_1_eye_piece_6_launch_loc_3911a()
-> BoomerangBossHead1EyePiece6LaunchLoc3911A {
    // ASM loc_3911A (kid.asm:41733): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$4000` (longword = +0x4000 = +16384, rightward — mirrors eye
    // piece 5's leftward direction at loc_39082). Then enter the `loc_3912E`
    // (kid.asm:41738) gravity loop: each frame runs `sub_37A22` (directional
    // sprite update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_39150` (kid.asm:41749) is the loop-back: `bra.s loc_3912E` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead1EyePiece6LaunchLoc3911A {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: 0x4000_i32,                           // $4000 as signed longword (rightward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_3912E
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_39150: true,       // bra.s loc_3912E when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_5_init_loc_39152(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead2EyePiece5InitLoc39152 {
    // ASM loc_39152 (kid.asm:41753): allocate head 2's eye piece 5 with gfx
    // slot `$01030002`, priority 0 (tertiary wave), object_meta `$21`. Calls
    // `loc_32146(pc)` for shared init, clears `has_kid_collision(a3)`, sets
    // `$13(a3)` and `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`,
    // copies anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame
    // byte for orbit placement sub_393C8). Uses head 2's parent pointer
    // `($FFFFFB7E)`, direction `($FFFFFB87)`, and breakup `($FFFFFB74)`.
    // Priority 0 (tertiary wave, same as piece 6 at loc_391EA). Uses sub_393C8
    // (mirrors piece 6's sub_39420).
    BoomerangBossHead2EyePiece5InitLoc39152 {
        gfx_slot: 0x0103_0002,
        priority: 0,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_5_tracking_loop_loc_3918E(
    breakup_trigger_fb74: u16,
) -> BoomerangBossHead2EyePiece5TrackingLoopLoc3918E {
    // ASM loc_3918E (kid.asm:41768): hibernate one frame, load parent pointer
    // from `($FFFFFB7E).w` into `a2`, load direction byte from `($FFFFFB87).w`
    // into `d2`, run `sub_393C8` (orbit placement — mirrors piece 6's
    // sub_39420) and `sub_37A22` (directional sprite selection), store `d6`
    // into `addroffset_sprite(a3)`. Then check `tst.w ($FFFFFB74).w`: nonzero
    // branches to `loc_391B2` (launch); zero loads `$16(a2)` into `d0`
    // (parent frame byte) and loops back.
    BoomerangBossHead2EyePiece5TrackingLoopLoc3918E {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_393c8: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb74 != 0 {
            BoomerangBossHead2EyePiece5TrackingNextLoc3918E::LaunchLoc391B2
        } else {
            BoomerangBossHead2EyePiece5TrackingNextLoc3918E::TrackingLoopLoc3918E
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_5_launch_loc_391B2()
-> BoomerangBossHead2EyePiece5LaunchLoc391B2 {
    // ASM loc_391B2 (kid.asm:41781): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$FFFFC000` (longword = -0x4000 = -16384, leftward — mirrors eye
    // piece 6's rightward direction at loc_3924A). Then enter the `loc_391C6`
    // (kid.asm:41786) gravity loop: each frame runs `sub_37A22` (directional
    // sprite update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_391E8` (kid.asm:41797) is the loop-back: `bra.s loc_391C6` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead2EyePiece5LaunchLoc391B2 {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: -0x4000_i32,                          // $FFFFC000 as signed longword (leftward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_391C6
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_391e8: true,       // bra.s loc_391C6 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_6_init_loc_391EA(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead2EyePiece6InitLoc391EA {
    // ASM loc_391EA (kid.asm:41801): allocate head 2's eye piece 6 with gfx
    // slot `$01030002`, priority 0 (tertiary wave), object_meta `$21`. Calls
    // `loc_32146(pc)` for shared init, clears `has_kid_collision(a3)`, sets
    // `$13(a3)` and `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`,
    // copies anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame
    // byte for orbit placement sub_39420). Uses head 2's parent pointer
    // `($FFFFFB7E)`, direction `($FFFFFB87)`, and breakup `($FFFFFB74)`.
    // Priority 0 (tertiary wave, same as piece 5 at loc_39152). Uses sub_39420
    // (mirrors piece 5's sub_393C8).
    BoomerangBossHead2EyePiece6InitLoc391EA {
        gfx_slot: 0x0103_0002,
        priority: 0,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_6_tracking_loop_loc_39226(
    breakup_trigger_fb74: u16,
) -> BoomerangBossHead2EyePiece6TrackingLoopLoc39226 {
    // ASM loc_39226 (kid.asm:41816): hibernate one frame, load parent pointer
    // from `($FFFFFB7E).w` into `a2`, load direction byte from `($FFFFFB87).w`
    // into `d2`, run `sub_39420` (orbit placement — mirrors piece 5's
    // sub_393C8) and `sub_37A22` (directional sprite selection), store `d6`
    // into `addroffset_sprite(a3)`. Then check `tst.w ($FFFFFB74).w`: nonzero
    // branches to `loc_3924A` (launch); zero loads `$16(a2)` into `d0`
    // (parent frame byte) and loops back.
    BoomerangBossHead2EyePiece6TrackingLoopLoc39226 {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_39420: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb74 != 0 {
            BoomerangBossHead2EyePiece6TrackingNextLoc39226::LaunchLoc3924A
        } else {
            BoomerangBossHead2EyePiece6TrackingNextLoc39226::TrackingLoopLoc39226
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_eye_piece_6_launch_loc_3924A()
-> BoomerangBossHead2EyePiece6LaunchLoc3924A {
    // ASM loc_3924A (kid.asm:41829): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$4000` (longword = +0x4000 = +16384, rightward — mirrors eye
    // piece 5's leftward direction at loc_391B2). Then enter the `loc_3925E`
    // (kid.asm:41834) gravity loop: each frame runs `sub_37A22` (directional
    // sprite update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_39280` (kid.asm:41845) is the loop-back: `bra.s loc_3925E` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead2EyePiece6LaunchLoc3924A {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: 0x4000_i32,                           // $4000 as signed longword (rightward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_3925E
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_39280: true,       // bra.s loc_3925E when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_5_init_loc_39282(
    anchor_x: u16,
    anchor_y: u16,
) -> BoomerangBossHead3EyePiece5InitLoc39282 {
    // ASM loc_39282 (kid.asm:41849): allocate head 3's eye piece 5 with gfx
    // slot `$01030002`, priority 0 (tertiary wave), object_meta `$21`. Calls
    // `loc_32146(pc)` for shared init, clears `has_kid_collision(a3)`, sets
    // `$13(a3)` and `is_moved(a3)`, assigns sprite `LnkTo_unk_C8488-Data_Index`,
    // copies anchor from `$44/$46(a5)`, and clears `d0` via `sf` (initial frame
    // byte for orbit placement sub_393C8). Uses head 3's parent pointer
    // `($FFFFFB82)`, direction `($FFFFFB88)`, and breakup `($FFFFFB76)`.
    // Priority 0 (tertiary wave, same as piece 6 at loc_3931A). Uses sub_393C8
    // (mirrors piece 6's sub_39420).
    BoomerangBossHead3EyePiece5InitLoc39282 {
        gfx_slot: 0x0103_0002,
        priority: 0,
        object_meta: 0x21,
        has_kid_collision: false, // sf has_kid_collision(a3)
        display_enabled: true,    // st $13(a3)
        is_moved: true,           // st is_moved(a3)
        sprite_label: "LnkTo_unk_C8488-Data_Index",
        x_pos: anchor_x,
        y_pos: anchor_y,
        initial_frame_byte: 0, // sf d0
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_5_tracking_loop_loc_392BE(
    breakup_trigger_fb76: u16,
) -> BoomerangBossHead3EyePiece5TrackingLoopLoc392BE {
    // ASM loc_392BE (kid.asm:41864): hibernate one frame, load parent pointer
    // from `($FFFFFB82).w` into `a2`, load direction byte from `($FFFFFB88).w`
    // into `d2`, run `sub_393C8` (orbit placement — mirrors piece 6's
    // sub_39420) and `sub_37A22` (directional sprite selection), store `d6`
    // into `addroffset_sprite(a3)`. Then check `tst.w ($FFFFFB76).w`: nonzero
    // branches to `loc_392E2` (launch); zero loads `$16(a2)` into `d0`
    // (parent frame byte) and loops back.
    BoomerangBossHead3EyePiece5TrackingLoopLoc392BE {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_393c8: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb76 != 0 {
            BoomerangBossHead3EyePiece5TrackingNextLoc392BE::LaunchLoc392E2
        } else {
            BoomerangBossHead3EyePiece5TrackingNextLoc392BE::TrackingLoopLoc392BE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_5_launch_loc_392E2()
-> BoomerangBossHead3EyePiece5LaunchLoc392E2 {
    // ASM loc_392E2 (kid.asm:41877): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$FFFFC000` (longword = -0x4000 = -16384, leftward — mirrors eye
    // piece 6's rightward direction at loc_3937A). Then enter the `loc_392F6`
    // (kid.asm:41882) gravity loop: each frame runs `sub_37A22` (directional
    // sprite update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_39318` (kid.asm:41893) is the loop-back: `bra.s loc_392F6` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead3EyePiece5LaunchLoc392E2 {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: -0x4000_i32,                          // $FFFFC000 as signed longword (leftward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_392F6
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_39318: true,       // bra.s loc_392F6 when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_6_tracking_loop_loc_39356(
    breakup_trigger_fb76: u16,
) -> BoomerangBossHead3EyePiece6TrackingLoopLoc39356 {
    // ASM loc_39356 (kid.asm:41912): hibernate one frame, load parent pointer
    // from `($FFFFFB82).w` into `a2`, load direction byte from `($FFFFFB88).w`
    // into `d2`, run `sub_39420` (orbit placement — mirrors piece 5's
    // sub_393C8) and `sub_37A22` (directional sprite selection), store `d6`
    // into `addroffset_sprite(a3)`. Then check `tst.w ($FFFFFB76).w`: nonzero
    // branches to `loc_3937A` (launch); zero loads `$16(a2)` into `d0`
    // (parent frame byte) and loops back.
    BoomerangBossHead3EyePiece6TrackingLoopLoc39356 {
        hibernates_one_frame: true,
        runs_orbit_placement_sub_39420: true,
        runs_directional_sprite_sub_37a22: true,
        updates_sprite_from_d6: true,
        next: if breakup_trigger_fb76 != 0 {
            BoomerangBossHead3EyePiece6TrackingNextLoc39356::LaunchLoc3937A
        } else {
            BoomerangBossHead3EyePiece6TrackingNextLoc39356::TrackingLoopLoc39356
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_3_eye_piece_6_launch_loc_3937A()
-> BoomerangBossHead3EyePiece6LaunchLoc3937A {
    // ASM loc_3937A (kid.asm:41925): call `sub_393B2` (detach step — sound
    // effect, rerank gfx object), set y_vel to `$FFFD` (word = -3) and x_vel
    // to `#$4000` (longword = +0x4000 = +16384, rightward — mirrors eye
    // piece 5's leftward direction at loc_392E2). Then enter the `loc_3938E`
    // (kid.asm:41930) gravity loop: each frame runs `sub_37A22` (directional
    // sprite update), adds `$7D0` to y_vel (longword), checks `y_pos` against
    // `Level_height_pixels`, and deletes the object when it falls off-screen.
    // `loc_393B0` (kid.asm:41941) is the loop-back: `bra.s loc_3938E` when
    // `y_pos <= Level_height_pixels`.
    BoomerangBossHead3EyePiece6LaunchLoc3937A {
        runs_detach_step_sub_393b2: true,
        y_vel: -3,                                   // $FFFD as signed word
        x_vel: 0x4000_i32,                           // $4000 as signed longword (rightward)
        gravity_per_frame: 0x7D0,                    // addi.l #$7D0,y_vel(a3)
        runs_directional_sprite_during_flight: true, // bsr.w sub_37A22 in loc_3938E
        deletes_below_level_height: true, // jmp j_Delete_CurrentObject when y_pos > level_height
        loops_back_loc_393b0: true,       // bra.s loc_3938E when y_pos <= level_height
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_wait_loop_loc_38062(
    movement_status_d7: u16,
) -> BoomerangBossHead2WaitLoopLoc38062 {
    // ASM loc_38062 (kid.asm:40427): hibernate one frame, run head 2's
    // `sub_399E4` movement wrapper, mirror `d7` into `$FFFFFB74`, and branch
    // to shared `loc_39EBE` on nonzero status before checking Kid's window.
    BoomerangBossHead2WaitLoopLoc38062 {
        hibernates_one_frame: true,
        runs_movement_wrapper_sub_399e4: true,
        cached_collision_word_fffffb74: movement_status_d7,
        next: if movement_status_d7 == 0 {
            BoomerangBossHead2WaitNextLoc38062::CheckKidWindow
        } else {
            BoomerangBossHead2WaitNextLoc38062::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_activation_window_loc_38076(
    kid_x: u16,
    kid_y: u16,
    head_x: u16,
    head_y: u16,
) -> BoomerangBossHead2ActivationWindowLoc38076 {
    // ASM loc_38076 (kid.asm:40436): load Kid/head coordinates, then branch
    // back to `loc_38062` unless signed Kid Y lies inside the inclusive
    // `head_y-$40..=head_y` activation window.
    let lower_y = head_y.wrapping_sub(0x0040);
    let kid_within_vertical_window =
        !word_lt_signed(kid_y, lower_y) && !word_gt_signed(kid_y, head_y);

    if !kid_within_vertical_window {
        return BoomerangBossHead2ActivationWindowLoc38076 {
            kid_within_vertical_window,
            x_direction: None,
            cached_x_direction_fffffb87: None,
            animation_script_label: None,
            next: BoomerangBossHead2ActivationNextLoc38076::WaitLoopLoc38062,
        };
    }

    // ASM loc_380A4 (kid.asm:40454): the signed `cmp.w d5,d3; bgt` branch
    // clears `x_direction(a3)` when head 2 is strictly right of Kid; otherwise
    // the fallthrough path stores `$FF`.
    let x_direction = if word_gt_signed(head_x, kid_x) {
        0
    } else {
        0xFF
    };

    // ASM loc_380A8 (kid.asm:40457): mirror head 2's selected `x_direction`
    // byte into `$FFFFFB87`, load `stru_39FC6` into `d7`, then initialise the
    // attack animation before entering the next wait loop.
    BoomerangBossHead2ActivationWindowLoc38076 {
        kid_within_vertical_window,
        x_direction: Some(x_direction),
        cached_x_direction_fffffb87: Some(x_direction),
        animation_script_label: Some("stru_39FC6"),
        next: BoomerangBossHead2ActivationNextLoc38076::AttackAnimation,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_attack_wait_loop_loc_380B8(
    movement_status_d7: u16,
) -> BoomerangBossHead2AttackWaitLoopLoc380B8 {
    // ASM loc_380B8 (kid.asm:40462): hibernate one frame after `stru_39FC6`,
    // run `sub_399E4`, mirror `d7` into `$FFFFFB74`, and branch to
    // `loc_39EBE` on nonzero status before testing the attack animation.
    BoomerangBossHead2AttackWaitLoopLoc380B8 {
        hibernates_one_frame: true,
        runs_movement_wrapper_sub_399e4: true,
        cached_collision_word_fffffb74: movement_status_d7,
        next: if movement_status_d7 == 0 {
            BoomerangBossHead2AttackWaitNextLoc380B8::CheckAttackAnimationDone
        } else {
            BoomerangBossHead2AttackWaitNextLoc380B8::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_attack_animation_gate_loc_380CC(
    animation_done_byte_18: u8,
) -> BoomerangBossHead2AttackAnimationGateLoc380CC {
    // ASM loc_380CC (kid.asm:40471): `tst.b $18(a3)` loops back to loc_380B8
    // while the attack animation is still active; any nonzero byte calls
    // `sub_397E4` to spawn the head projectile before the post-projectile wait.
    let attack_animation_done = animation_done_byte_18 != 0;
    BoomerangBossHead2AttackAnimationGateLoc380CC {
        attack_animation_done,
        runs_projectile_spawn_sub_397e4: attack_animation_done,
        next: if attack_animation_done {
            BoomerangBossHead2AttackAnimationNextLoc380CC::SpawnProjectileSub397E4
        } else {
            BoomerangBossHead2AttackAnimationNextLoc380CC::AwaitAttackWaitLoopLoc380B8
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_post_projectile_wait_loop_loc_380D6(
    movement_status_d7: u16,
) -> BoomerangBossHead2PostProjectileWaitLoopLoc380D6 {
    // ASM loc_380D6 (kid.asm:40476): after `sub_397E4`, hibernate one frame,
    // run head 2's `sub_399E4`, mirror `d7` into `$FFFFFB74`, and branch to
    // shared `loc_39EBE` on nonzero status before checking `$18(a3)`.
    BoomerangBossHead2PostProjectileWaitLoopLoc380D6 {
        hibernates_one_frame: true,
        runs_movement_wrapper_sub_399e4: true,
        cached_collision_word_fffffb74: movement_status_d7,
        next: if movement_status_d7 == 0 {
            BoomerangBossHead2PostProjectileWaitNextLoc380D6::CheckAnimationDone
        } else {
            BoomerangBossHead2PostProjectileWaitNextLoc380D6::SharedCollisionExitLoc39EBE
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boomerang_boss_head_2_post_projectile_animation_gate_loc_380EA(
    animation_done_byte_18: u8,
) -> BoomerangBossHead2PostProjectileAnimationGateLoc380EA {
    // ASM loc_380EA (kid.asm:40485): keep waiting at loc_380D6 while
    // `$18(a3)` is zero; when the post-projectile animation marks completion,
    // branch back to head 2's activation wait loop at loc_38062.
    let post_projectile_animation_done = animation_done_byte_18 != 0;
    BoomerangBossHead2PostProjectileAnimationGateLoc380EA {
        post_projectile_animation_done,
        next: if post_projectile_animation_done {
            BoomerangBossHead2PostProjectileAnimationNextLoc380EA::ReturnToActivationWaitLoc38062
        } else {
            BoomerangBossHead2PostProjectileAnimationNextLoc380EA::AwaitPostProjectileWaitLoc380D6
        },
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_step_sub_39846(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
    reset_script: &[(i16, i16, i16)],
) -> BossHeadMovementStepSub39846 {
    // ASM sub_39846 (kid.asm:42476): count `d0` down every frame, and only
    // when it reaches zero pull the next `(delay, x, y)` triple from `(a2)+`.
    // A negative delay sentinel rewinds `a2` to `unk_39872`, after which the
    // X/Y words are converted to FP16.16 via `swap; asr.l #1`.
    let decremented_delay = delay_counter_d0.wrapping_sub(1);
    if decremented_delay != 0 {
        return BossHeadMovementStepSub39846 {
            next_delay_counter: decremented_delay,
            next_script_index: current_script_index,
            next_x_velocity: None,
            next_y_velocity: None,
        };
    }

    let mut script = current_script;
    let mut script_index = current_script_index;
    loop {
        let (next_delay, x_word, y_word) = script[script_index];
        if next_delay < 0 {
            script = reset_script;
            script_index = 0;
            continue;
        }

        let to_fp16_16 = |word: i16| i32::from(word) << 15;
        return BossHeadMovementStepSub39846 {
            next_delay_counter: next_delay as u16,
            next_script_index: script_index + 1,
            next_x_velocity: Some(to_fp16_16(x_word)),
            next_y_velocity: Some(to_fp16_16(y_word)),
        };
    }
}

const SHISHKA_BOSS_HEAD_MOVEMENT_SUB_39872: [(i16, i16, i16); 12] = [
    (0x0168, 0, -1),
    (0x0168, -1, 0),
    (0x0168, 0, 1),
    (0x0168, 1, 0),
    (0x00B4, 0, -2),
    (0x01CC, -1, 0),
    (0x0168, 0, 1),
    (0x00E6, 2, 0),
    (0x00B4, -1, 0),
    (0x012C, 0, 0),
    (0x00B4, -1, 0),
    (0x0168, 1, 0),
];

const BOOMERANG_BOSS_HEAD_1_MOVEMENT_SUB_398F0: [(i16, i16, i16); 16] = [
    (0x0096, 0, 0),
    (0x012C, 0, -1),
    (0x0064, -1, 0),
    (0x0064, 1, 0),
    (0x012C, 0, -1),
    (0x0064, 1, 0),
    (0x0064, -1, 0),
    (0x0096, 0, 1),
    (0x0064, -1, 0),
    (0x0064, 1, 0),
    (0x0096, 0, 1),
    (0x0064, 1, 0),
    (0x0064, -1, 0),
    (0x012C, 0, 1),
    (0x012C, 0, -2),
    (0x012C, 0, 2),
];

const BOOMERANG_BOSS_HEAD_2_MOVEMENT_SUB_39988: [(i16, i16, i16); 15] = [
    (0x012C, 0, -1),
    (0x0064, -1, 0),
    (0x0064, 1, 0),
    (0x012C, 0, -1),
    (0x0064, 1, 0),
    (0x0064, -1, 0),
    (0x0096, 0, 1),
    (0x0064, -1, 0),
    (0x0064, 1, 0),
    (0x0096, 0, 1),
    (0x0064, 1, 0),
    (0x0064, -1, 0),
    (0x012C, 0, 1),
    (0x012C, 0, -2),
    (0x012C, 0, 2),
];

const BOOMERANG_BOSS_HEAD_3_MOVEMENT_SUB_39A1A: [(i16, i16, i16); 16] = [
    (0x00C8, 0, 0),
    (0x012C, 0, -1),
    (0x0064, -1, 0),
    (0x0064, 1, 0),
    (0x012C, 0, -1),
    (0x0064, 1, 0),
    (0x0064, -1, 0),
    (0x0096, 0, 1),
    (0x0064, -1, 0),
    (0x0064, 1, 0),
    (0x0096, 0, 1),
    (0x0064, 1, 0),
    (0x0064, -1, 0),
    (0x012C, 0, 1),
    (0x00C8, 0, -3),
    (0x00C8, 0, 3),
];

const BAGEL_BROTHERS_HEAD_MOVEMENT_SUB_39AB2: [(i16, i16, i16); 54] = [
    (0x0032, 0, 0),
    (0x001E, 0, -1),
    (0x0384, -1, 0),
    (0x0032, 0, 0),
    (0x001E, 0, 1),
    (0x012C, 3, 0),
    (0x0032, 0, 0),
    (0x0384, -1, 0),
    (0x0032, 0, 0),
    (0x00C8, 0, -1),
    (0x012C, 3, 0),
    (0x00C8, 0, 1),
    (0x0096, 0, 0),
    (0x0384, -1, 0),
    (0x0096, 0, 0),
    (0x0064, 0, -1),
    (0x012C, 3, 0),
    (0x0064, 0, 1),
    (0x012C, -3, 0),
    (0x00C8, 0, -1),
    (0x0384, 1, 0),
    (0x01C2, -2, 0),
    (0x00E1, 4, 0),
    (0x00B4, -5, 0),
    (0x0096, 6, 0),
    (0x00C8, 0, 1),
    (0x0032, 0, 0),
    (0x001E, 0, -1),
    (0x012C, -3, 0),
    (0x0032, 0, 0),
    (0x001E, 0, 1),
    (0x012C, 3, 0),
    (0x0096, 0, 0),
    (0x0384, -1, 0),
    (0x0096, 0, 0),
    (0x0064, 0, -1),
    (0x012C, 3, 0),
    (0x0064, 0, 1),
    (0x0032, 0, 0),
    (0x00E1, -4, 0),
    (0x00C8, 0, -1),
    (0x0384, 1, 0),
    (0x01C2, -2, 0),
    (0x0032, 0, 0),
    (0x00B4, 5, 0),
    (0x00E1, -4, 0),
    (0x0384, 1, 0),
    (0x00C8, 0, 1),
    (0x0064, 0, 0),
    (0x00E1, -4, 0),
    (0x0064, 0, 0),
    (0x0064, 0, -1),
    (0x00E1, 4, 0),
    (0x0064, 0, 1),
];

const BAGEL_BROTHERS_HEAD_MOVEMENT_SUB_39C2E: [(i16, i16, i16); 47] = [
    (0x001E, 0, -1),
    (0x0384, -1, 0),
    (0x001E, 0, 1),
    (0x012C, 3, 0),
    (0x0384, -1, 0),
    (0x0032, 0, 0),
    (0x00C8, 0, -1),
    (0x012C, 3, 0),
    (0x00C8, 0, 1),
    (0x0096, 0, 0),
    (0x0384, -1, 0),
    (0x0064, 0, -1),
    (0x012C, 3, 0),
    (0x0064, 0, 1),
    (0x012C, -3, 0),
    (0x00C8, 0, -1),
    (0x0384, 1, 0),
    (0x01C2, -2, 0),
    (0x00E1, 4, 0),
    (0x012C, -3, 0),
    (0x0096, 6, 0),
    (0x00C8, 0, 1),
    (0x001E, 0, -1),
    (0x012C, -3, 0),
    (0x0032, 0, 0),
    (0x001E, 0, 1),
    (0x012C, 3, 0),
    (0x0384, -1, 0),
    (0x0096, 0, 0),
    (0x0064, 0, -1),
    (0x012C, 3, 0),
    (0x0064, 0, 1),
    (0x012C, -3, 0),
    (0x00C8, 0, -1),
    (0x0384, 1, 0),
    (0x01C2, -2, 0),
    (0x0032, 0, 0),
    (0x00B4, 5, 0),
    (0x00E1, -4, 0),
    (0x00B4, 5, 0),
    (0x00C8, 0, 1),
    (0x0064, 0, 0),
    (0x012C, -3, 0),
    (0x001E, 0, 0),
    (0x0064, 0, -1),
    (0x00E1, 4, 0),
    (0x0064, 0, 1),
];

const BAGEL_BROTHERS_HEAD_MOVEMENT_SUB_39D80: [(i16, i16, i16); 51] = [
    (0x001E, 0, -1),
    (0x0384, -1, 0),
    (0x0032, 0, 0),
    (0x001E, 0, 1),
    (0x012C, 3, 0),
    (0x0032, 0, 0),
    (0x0384, -1, 0),
    (0x0032, 0, 0),
    (0x00C8, 0, -1),
    (0x012C, 3, 0),
    (0x00C8, 0, 1),
    (0x0096, 0, 0),
    (0x0384, -1, 0),
    (0x0096, 0, 0),
    (0x0064, 0, -1),
    (0x012C, 3, 0),
    (0x0064, 0, 1),
    (0x012C, -3, 0),
    (0x00C8, 0, -1),
    (0x00E1, 4, 0),
    (0x00B4, -5, 0),
    (0x0096, 6, 0),
    (0x00C8, 0, 1),
    (0x0032, 0, 0),
    (0x001E, 0, -1),
    (0x012C, -3, 0),
    (0x0032, 0, 0),
    (0x001E, 0, 1),
    (0x012C, 3, 0),
    (0x0096, 0, 0),
    (0x0384, -1, 0),
    (0x0096, 0, 0),
    (0x0064, 0, -1),
    (0x012C, 3, 0),
    (0x0064, 0, 1),
    (0x0032, 0, 0),
    (0x00E1, -4, 0),
    (0x00C8, 0, -1),
    (0x0384, 1, 0),
    (0x01C2, -2, 0),
    (0x0032, 0, 0),
    (0x00B4, 5, 0),
    (0x00E1, -4, 0),
    (0x0384, 1, 0),
    (0x00C8, 0, 1),
    (0x0064, 0, 0),
    (0x00E1, -4, 0),
    (0x0064, 0, 0),
    (0x0064, 0, -1),
    (0x00E1, 4, 0),
    (0x0064, 0, 1),
];

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_wrapper_sub_398BC(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementWrapper {
    // ASM sub_398BC (kid.asm:42522): run the Shishka head movement step via
    // `sub_39846`, which rewinds sentinels to `unk_39872`, then tail-call the
    // shared boss collision/palette helper `sub_3A292`.
    BossHeadMovementWrapper {
        movement: boss_head_movement_step_sub_39846(
            delay_counter_d0,
            current_script_index,
            current_script,
            &SHISHKA_BOSS_HEAD_MOVEMENT_SUB_39872,
        ),
        runs_shared_collision_helper_sub_3_a292: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_step_sub_398C4(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementStepSub39846 {
    // ASM sub_398C4 (kid.asm:42532): same countdown/script loader as
    // `sub_39846`, but a negative sentinel always rewinds to
    // `unk_398F0` (Boomerang boss head 1's fixed loop table).
    boss_head_movement_step_sub_39846(
        delay_counter_d0,
        current_script_index,
        current_script,
        &BOOMERANG_BOSS_HEAD_1_MOVEMENT_SUB_398F0,
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_wrapper_sub_39952(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementWrapper {
    // ASM sub_39952 (kid.asm:42582): run `sub_398C4`'s Boomerang head-1 script
    // step, then tail-call the shared boss collision/palette helper
    // `sub_3A292`.
    BossHeadMovementWrapper {
        movement: boss_head_movement_step_sub_398C4(
            delay_counter_d0,
            current_script_index,
            current_script,
        ),
        runs_shared_collision_helper_sub_3_a292: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_step_sub_3995C(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementStepSub39846 {
    // ASM sub_3995C (kid.asm:42592): same countdown/script loader as
    // `sub_39846`, but a negative sentinel always rewinds to
    // `unk_39988` (Boomerang boss head 2's fixed loop table).
    boss_head_movement_step_sub_39846(
        delay_counter_d0,
        current_script_index,
        current_script,
        &BOOMERANG_BOSS_HEAD_2_MOVEMENT_SUB_39988,
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_wrapper_sub_399E4(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementWrapper {
    // ASM sub_399E4 (kid.asm:42641): run `sub_3995C`'s Boomerang head-2 script
    // step, then tail-call the shared boss collision/palette helper
    // `sub_3A292`.
    BossHeadMovementWrapper {
        movement: boss_head_movement_step_sub_3995C(
            delay_counter_d0,
            current_script_index,
            current_script,
        ),
        runs_shared_collision_helper_sub_3_a292: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_step_sub_399EE(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementStepSub39846 {
    // ASM sub_399EE (kid.asm:42651): same countdown/script loader as
    // `sub_39846`, but a negative sentinel always rewinds to
    // `unk_39A1A` (Boomerang boss head 3's fixed loop table).
    boss_head_movement_step_sub_39846(
        delay_counter_d0,
        current_script_index,
        current_script,
        &BOOMERANG_BOSS_HEAD_3_MOVEMENT_SUB_39A1A,
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_wrapper_sub_39A7C(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementWrapper {
    // ASM sub_39A7C (kid.asm:42701): run `sub_399EE`'s Boomerang head-3 script
    // step, then tail-call the shared boss collision/palette helper
    // `sub_3A292`.
    BossHeadMovementWrapper {
        movement: boss_head_movement_step_sub_399EE(
            delay_counter_d0,
            current_script_index,
            current_script,
        ),
        runs_shared_collision_helper_sub_3_a292: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_step_sub_39A86(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementStepSub39846 {
    // ASM sub_39A86 (kid.asm:42711): same countdown/script loader as
    // `sub_39846`, but a negative sentinel always rewinds to
    // `unk_39AB2` (Bagel Brothers head movement).
    boss_head_movement_step_sub_39846(
        delay_counter_d0,
        current_script_index,
        current_script,
        &BAGEL_BROTHERS_HEAD_MOVEMENT_SUB_39AB2,
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_wrapper_sub_39BF8(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementWrapper {
    // ASM sub_39BF8 (kid.asm:42799): run `sub_39A86`'s Bagel Brothers head
    // script step, then tail-call the shared boss collision/palette helper
    // `sub_3A292`.
    BossHeadMovementWrapper {
        movement: boss_head_movement_step_sub_39A86(
            delay_counter_d0,
            current_script_index,
            current_script,
        ),
        runs_shared_collision_helper_sub_3_a292: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_step_sub_39C02(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementStepSub39846 {
    // ASM sub_39C02 (kid.asm:42809): same countdown/script loader as
    // `sub_39846`, but a negative sentinel always rewinds to
    // `unk_39C2E` (Bagel Brothers head movement variant).
    boss_head_movement_step_sub_39846(
        delay_counter_d0,
        current_script_index,
        current_script,
        &BAGEL_BROTHERS_HEAD_MOVEMENT_SUB_39C2E,
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_wrapper_sub_39D4A(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementWrapper {
    // ASM sub_39D4A (kid.asm:42890): run `sub_39C02`'s Bagel Brothers variant
    // head script step, then tail-call the shared boss collision/palette
    // helper `sub_3A292`.
    BossHeadMovementWrapper {
        movement: boss_head_movement_step_sub_39C02(
            delay_counter_d0,
            current_script_index,
            current_script,
        ),
        runs_shared_collision_helper_sub_3_a292: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_step_sub_39D54(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementStepSub39846 {
    // ASM sub_39D54 (kid.asm:42900): same countdown/script loader as
    // `sub_39846`, but a negative sentinel always rewinds to
    // `unk_39D80` (Bagel Brothers head movement variant).
    boss_head_movement_step_sub_39846(
        delay_counter_d0,
        current_script_index,
        current_script,
        &BAGEL_BROTHERS_HEAD_MOVEMENT_SUB_39D80,
    )
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_movement_wrapper_sub_39EB4(
    delay_counter_d0: u16,
    current_script_index: usize,
    current_script: &[(i16, i16, i16)],
) -> BossHeadMovementWrapper {
    // ASM sub_39EB4 (kid.asm:42985): run `sub_39D54`'s Bagel Brothers variant
    // head script step, then tail-call the shared boss collision/palette
    // helper `sub_3A292`.
    BossHeadMovementWrapper {
        movement: boss_head_movement_step_sub_39D54(
            delay_counter_d0,
            current_script_index,
            current_script,
        ),
        runs_shared_collision_helper_sub_3_a292: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_death_timeout_sound_sub_370D2(v_int_counter_fff809: u8) -> &'static str {
    // ASM loc_371D4..loc_37200 (kid.asm:39060): choose the timeout sound from
    // bit 0 of `$FFFFF809`. The odd branch queues `sfx_Plethora_x` but the ROM
    // comment notes "NO SOUND!".
    if v_int_counter_fff809 & 1 != 0 {
        "sfx_Plethora_x"
    } else {
        // ASM loc_371F4 (kid.asm:39071): an even `$FFFFF809` bit queues
        // `sfx_Boss_dies` before the timeout animation starts at loc_37200.
        "sfx_Boss_dies"
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_fragment_collision_step_sub_3A172(
    collision_type: i16,
) -> FireDemonFragmentCollisionStep {
    // ASM sub_3A172 (kid.asm:43214): clear `collision_type(a3)` whenever the
    // fragment is hit; only `$001C`, `$002C`, and `$FFFF` switch into
    // `stru_3A1E0`, hibernate until the animation ends, then delete.
    let mut result = FireDemonFragmentCollisionStep {
        next_collision_type: collision_type,
        animation_label: None,
        hibernate_until_anim_finished: false,
        delete_current_object: false,
    };

    if collision_type == 0 {
        return result;
    }

    result.next_collision_type = 0;
    if matches!(collision_type, 0x001C | 0x002C | -1) {
        result.animation_label = Some("stru_3A1E0");
        result.hibernate_until_anim_finished = true;
        result.delete_current_object = true;
    }

    result
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_fragment_collision_step_sub_3A1A6(
    collision_type: i16,
) -> FireDemonFragmentCollisionStep {
    // ASM sub_3A1A6 (kid.asm:43240): same collision clear/filter as
    // `sub_3A172`, but the matching fragment hit deletes immediately instead
    // of switching into `stru_3A1E0`.
    let mut result = FireDemonFragmentCollisionStep {
        next_collision_type: collision_type,
        animation_label: None,
        hibernate_until_anim_finished: false,
        delete_current_object: false,
    };

    if collision_type == 0 {
        return result;
    }

    result.next_collision_type = 0;
    if matches!(collision_type, 0x001C | 0x002C | -1) {
        result.delete_current_object = true;
    }

    result
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_collision_palette_sub_3A292(
    collision_type: u16,
    berzerker_charging: bool,
    flash_timer_d1: u8,
    hit_points_3e: i16,
) -> BossCollisionPaletteSub3A292 {
    // ASM sub_3A292 (kid.asm:43339): shared boss collision/palette helper.
    // Entry: swap d1 brings flash timer from high word to low byte.
    // Checks collision_type(a3): $1C/$2C always hit; $20/$24 only when
    // Berzerker_charging is set; anything else is ignored.
    let mut flash = flash_timer_d1;
    let mut hp = hit_points_3e;
    let mut hit = false;
    let mut cleared = false;
    let mut sfx: Option<&'static str> = None;

    if collision_type != 0 {
        cleared = true;
        // ASM: cmpi.w #$1C,d7; beq.s loc_3A2C0 / cmpi.w #$2C,d7; beq.s loc_3A2C0
        let direct_hit = collision_type == 0x1C || collision_type == 0x2C;
        // ASM: tst.b (Berzerker_charging).w; beq.s loc_3A2E0
        // cmpi.w #$20,d7; beq.s loc_3A2C0 / cmpi.w #$24,d7; beq.s loc_3A2C0
        let berzerker_hit =
            berzerker_charging && (collision_type == 0x20 || collision_type == 0x24);
        hit = direct_hit || berzerker_hit;
    }

    if hit {
        // ASM loc_3A2C0 (kid.asm:43356): move.b #$A,d1; play sfx; subi.w #1,$3E(a3).
        flash = 0x0A;
        sfx = Some("sfx_Big_Hopping_Skull_groan");
        hp -= 1;
    }

    if hit && hp <= 0 {
        // ASM loc_3A2FA: swap d1; move.w #1,d7; move.b #3,palette_line(a3); rts.
        return BossCollisionPaletteSub3A292 {
            collision_type_cleared: cleared,
            hit,
            played_sound_effect: sfx,
            next_hit_points: hp,
            next_flash_timer: flash,
            next_palette_line: 3,
            head_dead: true,
        };
    }

    // ASM loc_3A2E0 (kid.asm:43366): tst.b d1; beq.s loc_3A2EE.
    let palette_line = if flash != 0 {
        // subq.b #1,d1; move.b #3,palette_line(a3)
        flash -= 1;
        3
    } else {
        // ASM loc_3A2EE (kid.asm:43374): move.b #2,palette_line(a3).
        2
    };

    // ASM loc_3A2F4: swap d1; clr.w d7; rts.
    BossCollisionPaletteSub3A292 {
        collision_type_cleared: cleared,
        hit,
        played_sound_effect: sfx,
        next_hit_points: hp,
        next_flash_timer: flash,
        next_palette_line: palette_line,
        head_dead: false,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_death_anim_sequence_loc_39EE2() -> BossHeadDeathAnimSequenceLoc39EE2 {
    // ASM loc_39EE2 (kid.asm:43004): boss head death animation — plays 8
    // animation scripts with `j_Hibernate_UntilAnimFinished` between each.
    // `eori.b #$FF,x_direction(a3)` flips the direction after scripts at
    // indices 0, 2, 4, 6 (i.e. after stru_39FEE, stru_3A016, stru_3A03A,
    // stru_3A062). After the 8th animation, plays `sfx_Boss_dies` and loads
    // `d0=#$A` as the loop counter for the subsequent `loc_39F7A` spin loop.
    BossHeadDeathAnimSequenceLoc39EE2 {
        animation_scripts: [
            "stru_39FEE", // 6-frame reverse sweep
            "stru_3A004", // 6-frame forward sweep
            "stru_3A016", // 4-frame reverse sweep
            "stru_3A028", // 4-frame forward sweep
            "stru_3A03A", // 2-frame reverse sweep (5 frames)
            "stru_3A050", // 2-frame forward sweep (4 frames)
            "stru_3A062", // 2-frame reverse sweep (4 frames)
            "stru_3A074", // 2-frame forward sweep (4 frames)
        ],
        direction_flip_after_scripts: [true, false, true, false, true, false, true, false],
        played_sound_effect: "sfx_Boss_dies",
        death_spin_counter_d0: 0x000A,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_head_death_spin_loop_loc_39F7A(
    loop_counter_d0: u16,
) -> BossHeadDeathSpinLoopLoc39F7A {
    // ASM loc_39F7A (kid.asm:43039): each iteration plays 4 animation scripts
    // with `j_Hibernate_UntilAnimFinished`, flipping direction after indices 0
    // and 2 via `eori.b #$FF,x_direction(a3)`. After each set of 4,
    // `subq.w #1,d0` decrements the counter and loops back on `bne.s loc_39F7A`.
    // When d0 reaches 0, falls through to `j_Delete_CurrentObject`.
    BossHeadDeathSpinLoopLoc39F7A {
        animation_scripts: [
            "stru_3A086", // 1-frame reverse sweep (5 frames)
            "stru_3A09C", // 1-frame forward sweep (4 frames)
            "stru_3A0AE", // 1-frame reverse sweep (4 frames)
            "stru_3A0C0", // 1-frame forward sweep (4 frames)
        ],
        direction_flip_after_scripts: [true, false, true, false],
        loop_count_d0: loop_counter_d0,
        delete_current_object: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_fragment_movement_step_loc_3A134(
    horizontal_counter_40: u16,
) -> FireDemonFragmentMovementStepLoc3A134 {
    // ASM loc_3A134 (kid.asm:43185): hibernate one frame, then test `$40(a3)`.
    // If zero: `clr.l x_vel(a3)`, switch to `stru_3A1D6`, enter fade loop at
    // `loc_3A14E`. If nonzero: fall through to `loc_3A168` — `subq.w #1,$40(a3)`,
    // `bsr.w sub_3A1A6` (collision check), then loop back to `loc_3A134`.
    if horizontal_counter_40 == 0 {
        FireDemonFragmentMovementStepLoc3A134 {
            hibernates_one_frame: true,
            horizontal_counter_40,
            clears_x_vel: true,
            fade_animation_label: Some("stru_3A1D6"),
            next_horizontal_counter_40: 0,
            runs_collision_check_sub_3a1a6: false,
            next: FireDemonFragmentMovementNextLoc3A134::FadeLoopLoc3A14E,
        }
    } else {
        // ASM loc_3A168 (kid.asm:43206): `subq.w #1,$40(a3)`, `bsr.w sub_3A1A6`,
        // `bra.s loc_3A134`.
        FireDemonFragmentMovementStepLoc3A134 {
            hibernates_one_frame: true,
            horizontal_counter_40,
            clears_x_vel: false,
            fade_animation_label: None,
            next_horizontal_counter_40: horizontal_counter_40.wrapping_sub(1),
            runs_collision_check_sub_3a1a6: true,
            next: FireDemonFragmentMovementNextLoc3A134::ContinueMovementLoc3A168,
        }
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_fragment_fade_step_loc_3A14E(
    fade_counter_42: u16,
) -> FireDemonFragmentFadeStepLoc3A14E {
    // ASM loc_3A14E (kid.asm:43193): hibernate one frame, then test `$42(a3)`.
    // If zero: `j_Delete_CurrentObject`. If nonzero: fall through to
    // `loc_3A15E` — `subq.w #1,$42(a3)`, `bsr.w sub_3A172`, `bra.s loc_3A14E`.
    if fade_counter_42 == 0 {
        FireDemonFragmentFadeStepLoc3A14E {
            hibernates_one_frame: true,
            fade_counter_42,
            delete_current_object: true,
            next_fade_counter_42: 0,
            runs_collision_check_sub_3a172: false,
            next: FireDemonFragmentFadeNextLoc3A14E::DeleteCurrentObject,
        }
    } else {
        // ASM loc_3A15E (kid.asm:43200): `subq.w #1,$42(a3)`, `bsr.w sub_3A172`,
        // `bra.s loc_3A14E`.
        FireDemonFragmentFadeStepLoc3A14E {
            hibernates_one_frame: true,
            fade_counter_42,
            delete_current_object: false,
            next_fade_counter_42: fade_counter_42.wrapping_sub(1),
            runs_collision_check_sub_3a172: true,
            next: FireDemonFragmentFadeNextLoc3A14E::ContinueFadeLoc3A15E,
        }
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_fragment_decel_step_loc_3A24C(
    direction_byte_d0: u8,
    horizontal_counter_40: u16,
    x_pos: u16,
    level_width_pixels: u16,
) -> FireDemonFragmentDecelStepLoc3A24C {
    // ASM loc_3A24C (kid.asm:43300): hibernate one frame, then `tst.b d0`.
    // d0 is the high byte of the initial x_vel word — nonzero means the fragment
    // was launched leftward, zero means rightward.
    //
    // loc_3A24C: `tst.b d0; beq.s loc_3A25E; addi.l #$7D0,x_vel(a3); bra.s loc_3A266`
    // loc_3A25E: `addi.l #-$7D0,x_vel(a3)` — rightward fragment deceleration.
    // loc_3A266: `subq.w #1,$40(a3); bne.s loc_3A270; jmp (j_Delete_CurrentObject).w`
    // loc_3A270: `cmpi.w #0,x_pos(a3); bgt.s loc_3A27C; jmp (j_Delete_CurrentObject).w`
    // loc_3A27C: `cmp.w (Level_width_pixels).w,x_pos(a3); ble.s loc_3A28A; jmp (j_Delete_CurrentObject).w`
    // loc_3A28A: `bsr.w sub_3A1A6; bra.s loc_3A24C`
    let x_vel_delta = if direction_byte_d0 != 0 {
        // ASM loc_3A24C: leftward fragment — add $7D0 (pull rightward to decelerate)
        0x7D0_i32
    } else {
        // ASM loc_3A25E: rightward fragment — subtract $7D0 (pull leftward to decelerate)
        -0x7D0_i32
    };

    let next_counter = horizontal_counter_40.wrapping_sub(1);

    // ASM loc_3A266: subq.w #1,$40(a3); bne.s loc_3A270
    if next_counter == 0 {
        return FireDemonFragmentDecelStepLoc3A24C {
            hibernates_one_frame: true,
            direction_byte_d0,
            x_vel_delta,
            next_horizontal_counter_40: 0,
            delete_current_object: true,
            runs_collision_check_sub_3a1a6: false,
            next: FireDemonFragmentDecelNextLoc3A24C::DeleteLifetimeExpired,
        };
    }

    // ASM loc_3A270: cmpi.w #0,x_pos(a3); bgt.s loc_3A27C
    if (x_pos as i16) <= 0 {
        return FireDemonFragmentDecelStepLoc3A24C {
            hibernates_one_frame: true,
            direction_byte_d0,
            x_vel_delta,
            next_horizontal_counter_40: next_counter,
            delete_current_object: true,
            runs_collision_check_sub_3a1a6: false,
            next: FireDemonFragmentDecelNextLoc3A24C::DeleteOutOfBoundsLeft,
        };
    }

    // ASM loc_3A27C: cmp.w (Level_width_pixels).w,d5; ble.s loc_3A28A
    if (x_pos as i16) > (level_width_pixels as i16) {
        return FireDemonFragmentDecelStepLoc3A24C {
            hibernates_one_frame: true,
            direction_byte_d0,
            x_vel_delta,
            next_horizontal_counter_40: next_counter,
            delete_current_object: true,
            runs_collision_check_sub_3a1a6: false,
            next: FireDemonFragmentDecelNextLoc3A24C::DeleteOutOfBoundsRight,
        };
    }

    // ASM loc_3A28A: bsr.w sub_3A1A6; bra.s loc_3A24C
    FireDemonFragmentDecelStepLoc3A24C {
        hibernates_one_frame: true,
        direction_byte_d0,
        x_vel_delta,
        next_horizontal_counter_40: next_counter,
        delete_current_object: false,
        runs_collision_check_sub_3a1a6: true,
        next: FireDemonFragmentDecelNextLoc3A24C::ContinueDecelLoc3A24C,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_sprite_init_sub_3764A(
    x_pos: u16,
    y_pos: u16,
) -> HeadyMetalSpriteInitSub3764A {
    // ASM sub_3764A (kid.asm:39423): allocate gfx slot `$01010002`, copy the
    // boss anchor position into `$1A/$1E(a1)`, clear `x_direction(a1)`, seed
    // `object_meta(a1)` with `$20`, call `loc_32146`, then set `$13/$14(a1)`.
    HeadyMetalSpriteInitSub3764A {
        gfx_slot_base: 0x0101_0002,
        priority: 0,
        object_meta: 0x0020,
        display_enabled: true,
        is_moved: true,
        x_direction: 0,
        x_pos,
        y_pos,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_sprite_init_sub_37680(
    x_pos: u16,
    y_pos: u16,
) -> HeadyMetalSpriteInitSub37680 {
    // ASM sub_37680 (kid.asm:39443): allocate gfx slot `$01000002`, copy the
    // boss anchor position into `$1A/$1E(a1)`, clear `x_direction(a1)`, seed
    // `object_meta(a1)` with `$21`, call `loc_32146`, then set `$13/$14(a1)`.
    HeadyMetalSpriteInitSub37680 {
        gfx_slot_base: 0x0100_0002,
        priority: 0,
        object_meta: 0x0021,
        display_enabled: true,
        is_moved: true,
        x_direction: 0,
        x_pos,
        y_pos,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_sprite_init_sub_376B6(
    x_pos: u16,
    y_pos: u16,
) -> HeadyMetalSpriteInitSub376B6 {
    // ASM sub_376B6 (kid.asm:39463): allocate gfx slot `$00FF0002`, copy the
    // boss anchor position into `$1A/$1E(a1)`, clear `x_direction(a1)`, seed
    // `object_meta(a1)` with `$20`, call `loc_32146`, then set `$13/$14(a1)`.
    HeadyMetalSpriteInitSub376B6 {
        gfx_slot_base: 0x00FF_0002,
        priority: 0,
        object_meta: 0x0020,
        display_enabled: true,
        is_moved: true,
        x_direction: 0,
        x_pos,
        y_pos,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_apply_piece_velocities_sub_376EC(
    mut slots: [Option<HeadyMetalSpriteVelocitySub376EC>; 17],
    x_velocity: i32,
    y_velocity: i32,
) -> [Option<HeadyMetalSpriteVelocitySub376EC>; 17] {
    // ASM sub_376EC (kid.asm:39483): walk the 17 longword worker pointers at
    // `$FFFFFB72`, skip null entries, and copy `d6/d7` into `x_vel/y_vel(a3)`.
    for slot in &mut slots {
        // ASM loc_376F2 (kid.asm:39487): load the next longword worker pointer
        // with `(a4)+`; live pointers become `a3` before the velocity writes.
        if let Some(worker) = slot.as_mut() {
            worker.x_velocity = x_velocity;
            worker.y_velocity = y_velocity;
        }
        // ASM loc_37702 (kid.asm:39494): null pointers branch here, then `dbf`
        // decrements `d5` and continues until all `#$10+1` entries are visited.
    }

    slots
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_tracking_velocity_loc_37582(
    kid_gfx_x: u16,
    kid_gfx_y: u16,
    centre_piece_x: u16,
    centre_piece_y: u16,
    anchor_phase_counter: u16,
    global_velocity_freeze: bool,
) -> HeadyMetalTrackingVelocityLoc37582 {
    // ASM loc_37582 (kid.asm:39344): after the one-frame hibernate, compare
    // `$FFFFFB7A` against Kid's gfx-object position and seed `d6/d7` before
    // the shared `sub_376EC` worker velocity write.
    let piece_x = i16::from_ne_bytes(centre_piece_x.to_ne_bytes());
    let kid_x = i16::from_ne_bytes(kid_gfx_x.to_ne_bytes());
    let piece_target_y = i16::from_ne_bytes(centre_piece_y.to_ne_bytes()).wrapping_sub(0x0028_i16);
    let kid_y = i16::from_ne_bytes(kid_gfx_y.to_ne_bytes());

    let mut x_velocity = if piece_x > kid_x {
        // ASM loc_375AA (kid.asm:39358): `bgt.s loc_375AA` replaces the
        // default rightward drift with `d6 = #$FFFF8000`.
        -0x0000_8000
    } else {
        0x0000_8000
    };
    // ASM loc_375B0 (kid.asm:39361): compare `centre_piece_y - $28` against
    // Kid's gfx Y, using `#$4800` unless the signed `bgt.s loc_375BC` fires.
    let mut y_velocity = if piece_target_y > kid_y {
        -0x0000_5000
    } else {
        0x0000_4800
    };

    // ASM loc_375C2 (kid.asm:39371): load anchor worker `$FFFFFB72`,
    // compare `$3E(a2)` with `#4`, and clear `d6` while the counter is < 4.
    if anchor_phase_counter < 4 {
        x_velocity = 0;
    }
    // ASM loc_375D4 (kid.asm:39377): test global HeadyMetal freeze flag
    // `$FFFFFB4F`; when clear, keep the selected `d6/d7` for the worker update.
    if global_velocity_freeze {
        x_velocity = 0;
        y_velocity = 0;
    }

    // ASM loc_375DE (kid.asm:39383): both freeze paths join here before
    // `bsr.w sub_376EC`; return the final `d6/d7` pair for that worker update.
    HeadyMetalTrackingVelocityLoc37582 {
        x_velocity,
        y_velocity,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_spawn_animation_loc_3761E(
    piece_counter_before_spawn: u16,
) -> HeadyMetalSpawnAnimationLoc3761E {
    let decremented_piece_counter = piece_counter_before_spawn.wrapping_sub(1);
    let next_piece_counter = if decremented_piece_counter == 0 {
        0x000A
    } else {
        decremented_piece_counter
    };
    // ASM loc_3761E (kid.asm:39400): compare the decremented `d2` with `#6`;
    // counters 6+ initialise the high-count HeadyMetal spawn animation.
    let selected_animation_label = if decremented_piece_counter >= 6 {
        "stru_37B98"
    } else {
        // ASM loc_37630 (kid.asm:39408): counters below 6 initialise the
        // low-count HeadyMetal spawn animation, including the d2=0 wrap case.
        "stru_37BC0"
    };

    HeadyMetalSpawnAnimationLoc3761E {
        next_piece_counter,
        selected_animation_label,
        // ASM loc_3763A (kid.asm:39412): both animation branches join here,
        // run the breakup, palette-cycle, and sprite-refresh helpers, then
        // loop back to the tracking velocity start at `loc_37582`.
        runs_tail_update: true,
        runs_breakup_step_sub_37708: true,
        runs_palette_cycle_sub_379_e8: true,
        runs_sprite_refresh_sub_37_abc: true,
        loops_to_tracking_loc_37582: true,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_scatter_step_sub_3787C(
    mut velocities: HeadyMetalScatterVelocities,
    d6_x_velocity_delta: i32,
    d7_velocity_delta: i32,
) -> HeadyMetalScatterVelocities {
    // ASM sub_3787C (kid.asm:39641): apply the primary alternating breakup
    // pattern to the five fixed HeadyMetal worker slots rooted at
    // `$FFFFFB72/$76/$7A/$BAE/$BB2`.
    velocities.slot_fb72_x_velocity += d6_x_velocity_delta;
    velocities.slot_fb76_x_velocity += d7_velocity_delta;
    velocities.slot_fb7a_y_velocity += d7_velocity_delta;
    velocities.slot_fbae_x_velocity += d6_x_velocity_delta;
    velocities.slot_fbb2_x_velocity += d7_velocity_delta;
    velocities
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_scatter_step_sub_378A2(
    mut velocities: HeadyMetalScatterVelocities,
    d6_x_velocity_delta: i32,
    d7_velocity_delta: i32,
) -> HeadyMetalScatterVelocities {
    // ASM sub_378A2 (kid.asm:39658): apply the alternate breakup pattern to
    // the same five HeadyMetal worker slots at `$FFFFFB72/$76/$7A/$BAE/$BB2`.
    velocities.slot_fb72_x_velocity += d7_velocity_delta;
    velocities.slot_fb76_x_velocity += d6_x_velocity_delta;
    velocities.slot_fb7a_y_velocity += d6_x_velocity_delta;
    velocities.slot_fbae_x_velocity += d7_velocity_delta;
    velocities.slot_fbb2_x_velocity += d6_x_velocity_delta;
    velocities
}

const HEADY_METAL_PALETTE_WAVE_SUB_379E8: [u16; 32] = [
    0x0200, 0x0200, 0x0202, 0x0202, 0x0204, 0x0204, 0x0206, 0x0206, 0x0208, 0x0208, 0x020A, 0x020A,
    0x020C, 0x020C, 0x020E, 0x020E, 0x020E, 0x020E, 0x020C, 0x020C, 0x020A, 0x020A, 0x0208, 0x0208,
    0x0206, 0x0206, 0x0204, 0x0204, 0x0202, 0x0202, 0x0200, 0x0200,
];

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_palette_cycle_sub_379E8(
    phase_counter: u16,
) -> HeadyMetalPaletteCycleSub379E8 {
    // ASM sub_379E8 (kid.asm:39865): increment `d2`, wrap it to five bits, and
    // copy `unk_379A8[d2]` into `Palette_Buffer+$5E`.
    let next_phase_counter = phase_counter.wrapping_add(1) & 0x001F;
    HeadyMetalPaletteCycleSub379E8 {
        next_phase_counter,
        palette_word: HEADY_METAL_PALETTE_WAVE_SUB_379E8[usize::from(next_phase_counter)],
    }
}

const HEADY_METAL_PIECE_SPRITES_OFF_37AE0: [&str; 8] = [
    "LnkTo_unk_C8488-Data_Index",
    "LnkTo_unk_C8490-Data_Index",
    "LnkTo_unk_C8498-Data_Index",
    "LnkTo_unk_C84A0-Data_Index",
    "LnkTo_unk_C84A8-Data_Index",
    "LnkTo_unk_C84B0-Data_Index",
    "LnkTo_unk_C84B8-Data_Index",
    "LnkTo_unk_C84C0-Data_Index",
];

#[allow(non_snake_case)]
pub(crate) fn heady_metal_piece_sprite_mapping_loc_37AAE(table_index: usize) -> &'static str {
    // ASM loc_37AAE (kid.asm:39979): copy selected `d2` to `d7`, double it
    // as a word offset, then load the mapping word from `off_37AE0`.
    HEADY_METAL_PIECE_SPRITES_OFF_37AE0[table_index]
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_piece_sprite_sub_37A22(
    diamond_power_active: bool,
    v_int_counter_fff809: u8,
    effect_x: u16,
    effect_y: u16,
    kid_x: u16,
    kid_y: u16,
) -> &'static str {
    // ASM sub_37A22 (kid.asm:39896): diamond power forces the top/centre
    // HeadyMetal mappings; otherwise pick one of the first eight entries from
    // `off_37A02` using the same directional buckets as sub_37AF0.
    if diamond_power_active {
        return if v_int_counter_fff809 & 8 != 0 {
            "LnkTo_unk_C8488-Data_Index"
        } else {
            // ASM loc_37A38 (kid.asm:39906): when diamond power is active
            // and `$FFFFF809 & #8` is clear, force the alternate centre map.
            "LnkTo_unk_C84A8-Data_Index"
        };
    }

    // ASM loc_37A3E (kid.asm:39913): the non-diamond path snapshots Kid and
    // worker word positions, then uses `worker_y + 4` / `worker_y - 4` as the
    // signed vertical band before choosing a sprite-table index.
    if kid_y > effect_y.saturating_add(4) {
        // ASM loc_37A78 (kid.asm:39939): Kid below `y_pos(a3)+4` enters the
        // lower band, where `x_pos(a3)-4..x_pos(a3)+4` selects index 4.
        if kid_x < effect_x.saturating_sub(4) {
            // ASM loc_37A8C (kid.asm:39950): below-and-left chooses table
            // index 3, `LnkTo_unk_C84A0-Data_Index`.
            heady_metal_piece_sprite_mapping_loc_37AAE(3)
        } else if kid_x > effect_x.saturating_add(4) {
            // ASM loc_37A90 (kid.asm:39955): below-and-right chooses table
            // index 5, `LnkTo_unk_C84B0-Data_Index`.
            heady_metal_piece_sprite_mapping_loc_37AAE(5)
        } else {
            heady_metal_piece_sprite_mapping_loc_37AAE(4)
        }
    } else if kid_y < effect_y.saturating_sub(4) {
        // ASM loc_37A94 (kid.asm:39960): Kid above `y_pos(a3)-4` enters the
        // upper band, where `x_pos(a3)-4..x_pos(a3)+4` selects index 0.
        if kid_x < effect_x.saturating_sub(4) {
            // ASM loc_37AA8 (kid.asm:39971): upper-band Kid X left of
            // `x_pos(a3)-4` selects table index 1.
            heady_metal_piece_sprite_mapping_loc_37AAE(1)
        } else if kid_x > effect_x.saturating_add(4) {
            // ASM loc_37AAC (kid.asm:39976): upper-band Kid X right of
            // `x_pos(a3)+4` selects table index 7.
            heady_metal_piece_sprite_mapping_loc_37AAE(7)
        } else {
            heady_metal_piece_sprite_mapping_loc_37AAE(0)
        }
    } else if kid_x < effect_x.saturating_sub(8) {
        // ASM loc_37A74 (kid.asm:39934): in the middle vertical band, Kid
        // left of `x_pos(a3)-8` selects table index 2.
        heady_metal_piece_sprite_mapping_loc_37AAE(2)
    } else {
        heady_metal_piece_sprite_mapping_loc_37AAE(6)
    }
}

const HEADY_METAL_BREAKUP_SEQUENCE_SUB_37708: [usize; 37] = [
    0, 7, 2, 10, 4, 3, 8, 5, 9, 12, 6, 11, 1, 12, 2, 7, 4, 9, 6, 3, 10, 5, 8, 11, 1, 2, 1, 7, 4, 9,
    6, 3, 10, 5, 8, 11, 12,
];

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_breakup_step_sub_37708(
    anchor_present: bool,
    collision_type: i16,
    phase_counter: u16,
    remaining_scatter_frames: u16,
    frame_counter: u16,
    piece_slots: [Option<HeadyMetalBreakupPieceSub37708>; 13],
) -> HeadyMetalBreakupStepSub37708 {
    // ASM sub_37708 (kid.asm:39503): advance the HeadyMetal breakup sequence
    // rooted at `$FFFFFB72`, using `unk_3795E` to pick one of the 13 child
    // worker pointers at `$FFFFFB7A` before the final alternating scatter.
    let mut result = HeadyMetalBreakupStepSub37708 {
        next_collision_type: collision_type,
        next_phase_counter: phase_counter,
        next_remaining_scatter_frames: remaining_scatter_frames,
        piece_slots,
        played_sound_effect: None,
        spawned_pop_effect_at: None,
        spawned_attack_at: None,
        set_has_kid_collision: false,
        set_global_hit_latch: false,
        reveal_anchor_worker: false,
        clear_anchor_hidden_flag: false,
        reveal_linked_worker: false,
        trigger_cleanup_sub_378c8: false,
        scatter_step: None,
    };

    if !anchor_present {
        return result;
    }

    // ASM loc_37710 (kid.asm:39509): after `$FFFFFB72` is nonzero, read
    // `collision_type(a3)` into `d7`; zero branches to the shared tail.
    if collision_type != 0 {
        result.next_collision_type = 0;

        if collision_type == 0x002C {
            // ASM loc_3772A (kid.asm:39518): increment `$3E(a3)`, use that
            // counter as a word index into `unk_3795E`, then retarget `a3`
            // from `$FFFFFB7A` to the selected breakup worker pointer.
            result.next_phase_counter = phase_counter.wrapping_add(1);
            if let Some(&piece_index) =
                HEADY_METAL_BREAKUP_SEQUENCE_SUB_37708.get(usize::from(result.next_phase_counter))
                && let Some(mut piece) = result.piece_slots[piece_index]
            {
                match result.next_phase_counter {
                    0x0000..=0x000C => {
                        // ASM loc_3772A: counters `<= $C` add 2 to `$A(a3)`,
                        // refresh the object, and play `sfx_Boss_eye_pops`.
                        piece.sprite_word_y += 2;
                        result.played_sound_effect = Some("sfx_Boss_eye_pops");
                        result.piece_slots[piece_index] = Some(piece);
                    }
                    0x000D..=0x0018 => {
                        // ASM loc_37768 (kid.asm:39540): counters above `$C`
                        // and `<= $18` subtract 2 from `$A(a3)` before the
                        // shared object refresh tail.
                        piece.sprite_word_y -= 2;
                        result.piece_slots[piece_index] = Some(piece);
                    }
                    0x0019..=0x0024 => {
                        // ASM loc_37778 (kid.asm:39548): counters above `$18`
                        // and `<= $24` allocate `sub_3729E` and `sub_37332`
                        // at the selected worker's position, then clear `(a1)`.
                        result.spawned_pop_effect_at = Some((piece.x_pos, piece.y_pos));
                        result.spawned_attack_at = Some((piece.x_pos, piece.y_pos));
                        result.piece_slots[piece_index] = None;
                    }
                    _ => {}
                }
            }
        }
    }

    // ASM loc_377BC (kid.asm:39564): reload `$FFFFFB72`, compare `$3E(a3)`
    // against `#$25`, and return before arming Kid collision or the global
    // breakup latch until the counter reaches the final scatter phase.
    if result.next_phase_counter < 0x0025 {
        return result;
    }

    result.set_has_kid_collision = true;
    result.set_global_hit_latch = true;

    if remaining_scatter_frames == 0 {
        result.reveal_anchor_worker = true;
        result.clear_anchor_hidden_flag = true;
        result.reveal_linked_worker = true;
        result.trigger_cleanup_sub_378c8 = true;
        return result;
    }

    // ASM loc_377F6 (kid.asm:39582): seed the final-scatter deltas, decrement
    // `$40(a3)`, then use the decremented timer for the `#$15E` high-tier
    // branch before falling through to the lower tiers.
    result.next_remaining_scatter_frames = remaining_scatter_frames - 1;
    let timer = result.next_remaining_scatter_frames;
    let (d6_x_velocity_delta, d7_velocity_delta) = if timer > 0x015E {
        (-0x0003_0D40, 0x0003_0D40)
    } else if timer > 0x00FA {
        // ASM loc_3781E (kid.asm:39594): once the high-tier branch is skipped,
        // compare the decremented `$40(a3)` with `#$FA`; values above it use
        // the `$493E0` deltas before the still-lower scatter bands.
        (-0x0004_93E0, 0x0004_93E0)
    } else if timer > 0x0096 {
        // ASM loc_37832 (kid.asm:39602): when the decremented timer is at or
        // below `#$FA`, values above `#$96` use the `$61A80` scatter deltas.
        (-0x0006_1A80, 0x0006_1A80)
    } else if timer > 0x0064 {
        // ASM loc_37846 (kid.asm:39610): after the `#$96` branch is skipped,
        // values above `#$64` use the `$7A120` scatter deltas.
        (-0x0007_A120, 0x0007_A120)
    } else {
        // ASM loc_3785A (kid.asm:39618): timers at or below `#$64` fall
        // through to the final `$927C0` scatter deltas.
        (-0x0009_27C0, 0x0009_27C0)
    };

    // ASM loc_37866 (kid.asm:39622): mask `Frame_Counter` with `#2`; the
    // zero-bit path calls `sub_3787C`, otherwise execution branches onward.
    result.scatter_step = Some(HeadyMetalScatterStepSub37708 {
        pattern: if frame_counter & 2 == 0 {
            HeadyMetalScatterPatternSub37708::Primary
        } else {
            // ASM loc_37876 (kid.asm:39630): nonzero `Frame_Counter & #2`
            // branches here and calls the alternate `sub_378A2` helper.
            HeadyMetalScatterPatternSub37708::Alternate
        },
        d6_x_velocity_delta,
        d7_velocity_delta,
    });

    result
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn heady_metal_cleanup_sub_378C8(
    global_hit_latch_fb4e: bool,
    countdown_42: u16,
) -> HeadyMetalCleanupSub378C8 {
    // ASM sub_378C8 (kid.asm:39675): once the final HeadyMetal breakup pair is
    // visible, either decrement `$42(a3)` while the shared hit latch is set
    // and fan the five workers outward, or call `j_loc_1078` on the cached
    // worker pointers at `$FFFFFB72/$76/$7A/$BAE/$BB2` and clear those slots
    // once the countdown reaches zero.
    let mut next_countdown_42 = countdown_42;
    if global_hit_latch_fb4e {
        next_countdown_42 = next_countdown_42.wrapping_sub(1);
        if next_countdown_42 == 0 {
            // ASM loc_37922 (kid.asm:39701): delete the five cached workers
            // with `j_loc_1078`, then clear `$FFFFFB72/$76/$7A/$BAE/$BB2`.
            return HeadyMetalCleanupSub378C8 {
                next_countdown_42,
                should_cleanup_workers: true,
                current_x_velocity_delta: 0,
                left_worker_x_velocity_delta: 0,
                centre_worker_y_velocity_delta: 0,
                centre_worker_sprite_label: None,
                played_sound_effect: None,
                right_worker_x_velocity_delta: 0,
                far_right_worker_x_velocity_delta: 0,
            };
        }
    }

    HeadyMetalCleanupSub378C8 {
        next_countdown_42,
        should_cleanup_workers: false,
        // ASM loc_378D6 (kid.asm:39683): apply the same split impulse every
        // frame until the countdown expires and refresh the centre worker art
        // before replaying `sfx_Boss_dies`.
        current_x_velocity_delta: -0x0002_6D18,
        left_worker_x_velocity_delta: 0x0002_6D18,
        centre_worker_y_velocity_delta: 0x0001_3880,
        centre_worker_sprite_label: Some("LnkTo_unk_C8458-Data_Index"),
        played_sound_effect: Some("sfx_Boss_dies"),
        right_worker_x_velocity_delta: -0x0002_6D18,
        far_right_worker_x_velocity_delta: 0x0002_6D18,
    }
}

impl NativeRuntime {
    // PrizeDiamondCollected_Init runs a 32-step HUD fly-to-counter loop after
    // the contact frame, so the queued native award needs to survive 33 frame
    // starts before the diamond count increments.
    const PRIZE_DIAMOND_AWARD_DELAY_FRAMES: u8 = 33;
    const MINI_HOPPING_SKULL_SCREAM_DELAY_FRAMES: u8 = 15;

    pub(crate) fn apply_pending_stomp_bounce(&mut self, slice: &mut OpeningSliceState) {
        let Some(pending) = self.pending_stomp_bounce.take() else {
            return;
        };

        // ASM loc_B6DA (kid.asm:14621): Jump_On_Enemy bounce tail.
        //   move.l  d7,y_vel(a3)        — store bounce y_vel (d7 is $FFFC0000
        //     unless $FA6A on-board, then 0; native always uses STOMP_BOUNCE_VELOCITY)
        //   move.l  x_vel(a3),d7        — d7 = current x_vel
        //   sub.l   d7,x_pos(a3)        — rewind x_pos by one frame of x movement
        //   asr.l   #1,d7               — d7 = x_vel / 2 (arithmetic shift)
        //   add.l   d7,x_vel(a3)        — x_vel += x_vel/2 (net: 1.5× original)
        //   bclr    #$E,d6              — test+clear bit 14 of object_meta
        //   bne.w   Check_for_recent_damge — bit 14 set → also take damage after stomp
        //   rts
        let current_x_velocity = slice.player.x_velocity;
        slice.player.x_pos =
            (pending.anchor_x_pos & 0xFFFF_0000) | (slice.player.x_pos & 0x0000_FFFF);
        slice.player.y_pos =
            (pending.anchor_y_pos & 0xFFFF_0000) | (slice.player.y_pos & 0x0000_FFFF);
        slice.player.y_velocity = STOMP_BOUNCE_VELOCITY;
        let rewound_x = i64::from(slice.player.x_pos) - i64::from(current_x_velocity);
        slice.player.x_pos = rewound_x.clamp(0, i64::from(u32::MAX)) as u32;
        slice.player.x_velocity = slice
            .player
            .x_velocity
            .saturating_add(current_x_velocity >> 1);
        slice.player.on_ground = false;
        self.stomped_this_frame = true;
    }

    pub(crate) fn spawn_enemy_death_effect(&mut self, family: EnemyFamily, x_pos: u32, y_pos: u32) {
        let remaining = if matches!(family, EnemyFamily::HeadyMetal) {
            HEADYMETAL_DEATH_EFFECT_FRAMES
        } else {
            GENERIC_ENEMY_DEATH_EFFECT_FRAMES
        };
        self.enemy_death_effects.push(EnemyDeathEffect {
            x_pos,
            y_pos,
            remaining,
        });
    }

    pub(crate) fn remove_dead_enemy(&mut self, idx: usize) {
        let enemy = &self.active_enemies[idx];
        let family = enemy.family;
        let x_pos = enemy.x_pos;
        let y_pos = enemy.y_pos;
        self.killed_enemies
            .insert((enemy.segment_index, enemy.layout_index));
        self.spawn_enemy_death_effect(family, x_pos, y_pos);
        if matches!(family, EnemyFamily::FireDemon) {
            spawn_fire_demon_death_trail(&mut self.projectiles, x_pos, y_pos);
        }
        self.active_enemies.remove(idx);
        self.add_score(1000);
    }

    /// Check for stomp-kills: player landing on enemies from above.
    /// Decrements HP on stomp; removes enemy at 0 HP. Always bounces player.
    pub(crate) fn process_stomps(
        &mut self,
        slice: &mut OpeningSliceState,
        pre_physics_y_velocity: i32,
        pre_physics_y_pos: u32,
        frame_start_x_pos: u32,
        frame_start_y_pos: u32,
    ) -> bool {
        if !matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
            return false;
        }

        let mut stomped_indices = Vec::new();
        for (i, enemy) in self.active_enemy_roster.iter().enumerate() {
            let is_diamond = matches!(enemy.kind, NativeObjectKind::Enemy { object_type: 0x01 });
            if is_diamond {
                continue;
            }
            // Skip invulnerable enemies (e.g. submerged TarMonster)
            if i < self.active_enemies.len() && !self.active_enemies[i].is_vulnerable() {
                continue;
            }
            if classify_enemy_contact_with_half_width(
                slice.player,
                enemy,
                pre_physics_y_velocity,
                pre_physics_y_pos,
                self.progression.current_form.half_width() as i64,
            ) == EnemyContactResult::Stomp
            {
                stomped_indices.push(i);
            }
        }

        if stomped_indices.is_empty() {
            return false;
        }

        // Decrement HP; collect indices of enemies that should be removed
        let mut kill_indices = Vec::new();
        for &roster_idx in &stomped_indices {
            if roster_idx < self.active_enemies.len() {
                let enemy = &mut self.active_enemies[roster_idx];
                enemy.hit_points = enemy.hit_points.saturating_sub(1);
                if enemy.hit_points == 0 {
                    kill_indices.push(roster_idx);
                } else {
                    enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                    if matches!(enemy.family, EnemyFamily::DragonWalking) {
                        // The stomp resolves after the landing frame. Give the
                        // hurt hibernate a one-frame arm so DragonWalking still
                        // reaches the oracle-backed frame-2938 contact tile
                        // before freezing from frame 2939 onward.
                        enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES + 1;
                        enemy.grounded_frames = DRAGON_WALKING_NONLETHAL_STOMP_HIBERNATE_FRAMES;
                        enemy.x_velocity = 0;
                        enemy.y_velocity = 0;
                    }
                    // Bosses retaliate with projectiles when damaged
                    if matches!(
                        enemy.family,
                        EnemyFamily::HeadyMetal
                            | EnemyFamily::Shishkaboss
                            | EnemyFamily::BoomerangBosses
                            | EnemyFamily::BagelBrothers
                    ) {
                        let boss_x = enemy.x_pos;
                        let boss_y = enemy.y_pos;
                        let is_heady = matches!(enemy.family, EnemyFamily::HeadyMetal);
                        self.projectiles.push(EnemyProjectile {
                            x_pos: boss_x,
                            y_pos: boss_y,
                            x_velocity: 0x2_0000,
                            y_velocity: -0x1_0000,
                            lifetime: if is_heady { 120 } else { 90 },
                            gravity: if is_heady { 0x2710 } else { 0x800 },
                            friendly: false,
                            homing_accel: if is_heady { 0x7D0 } else { 0 },
                            speed_cap: 0,
                            wall_bounce: false,
                            ..Default::default()
                        });
                        self.projectiles.push(EnemyProjectile {
                            x_pos: boss_x,
                            y_pos: boss_y,
                            x_velocity: -0x2_0000,
                            y_velocity: -0x1_0000,
                            lifetime: if is_heady { 120 } else { 90 },
                            gravity: if is_heady { 0x2710 } else { 0x800 },
                            friendly: false,
                            homing_accel: if is_heady { 0x7D0 } else { 0 },
                            speed_cap: 0,
                            wall_bounce: false,
                            ..Default::default()
                        });
                    }
                    // TarMonster fires 5 tar blobs in a spread pattern when damaged
                    if matches!(enemy.family, EnemyFamily::TarMonster) {
                        let tx = enemy.x_pos;
                        let ty = enemy.y_pos;
                        spawn_tar_monster_retaliation(&mut self.projectiles, tx, ty);
                    }
                }
            }
        }

        // Remove dead enemies, spawn death effects, and award points.
        // Track killed identity so activate_enemies won't respawn them
        // (ASM uses per-segment killed bitfield at $FFFFF66A).
        for &idx in kill_indices.iter().rev() {
            self.remove_dead_enemy(idx);
        }

        // Rebuild roster after changes
        self.active_enemy_roster = self
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();

        self.pending_stomp_bounce = Some(PendingStompBounce {
            anchor_x_pos: frame_start_x_pos,
            anchor_y_pos: frame_start_y_pos,
        });
        true
    }

    pub(crate) fn first_stomp_contact_candidate(
        &self,
        slice: &OpeningSliceState,
        pre_physics_y_velocity: i32,
        pre_physics_y_pos: u32,
    ) -> Option<&NativeObjectSnapshot> {
        if !matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
            return None;
        }

        for (i, enemy) in self.active_enemy_roster.iter().enumerate() {
            let is_diamond = matches!(enemy.kind, NativeObjectKind::Enemy { object_type: 0x01 });
            if is_diamond {
                continue;
            }
            if i < self.active_enemies.len() && !self.active_enemies[i].is_vulnerable() {
                continue;
            }
            if classify_enemy_contact_with_half_width(
                slice.player,
                enemy,
                pre_physics_y_velocity,
                pre_physics_y_pos,
                self.progression.current_form.half_width() as i64,
            ) == EnemyContactResult::Stomp
            {
                return Some(enemy);
            }
        }

        None
    }

    /// Collect Diamond enemies on touch: remove from active list, increment count.
    pub(crate) fn collect_diamonds(&mut self, slice: &OpeningSliceState) {
        if !matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
            return;
        }

        let mut collected_indices = Vec::new();
        for (i, enemy) in self.active_enemies.iter().enumerate() {
            if enemy.family != EnemyFamily::Diamond {
                continue;
            }
            let snapshot = NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy {
                    object_type: enemy.family.object_type(),
                },
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                layout_index: Some(enemy.layout_index),
                x_velocity: Some(enemy.x_velocity),
                y_velocity: Some(enemy.y_velocity),
                facing: Some(enemy.facing),
                on_ground: Some(enemy.on_ground),
                hit_points: Some(enemy.hit_points),
                damage_flash: false,
                palette_variant: enemy.palette_variant,
                stationary: false,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            };
            if player_touches_enemy_with_half_width(
                slice.player,
                &snapshot,
                self.progression.current_form.half_width() as i64,
            ) {
                collected_indices.push(i);
            }
        }

        for &idx in collected_indices.iter().rev() {
            self.active_enemies.remove(idx);
            self.collect_diamond();
        }

        if !collected_indices.is_empty() {
            // Rebuild roster after removal
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
    }

    /// Add one life, capped at 9 per ASM.
    pub(crate) fn add_life(&mut self) {
        if self.progression.number_lives < 9 {
            self.progression.number_lives += 1;
        }
    }

    /// Add points with score cap (9,999,999) and extra-life-per-50K tracking.
    pub(crate) fn add_score(&mut self, points: u32) {
        // ASM Prize10000Points_Init -> loc_10AAE (kid.asm:22367): after the
        // optional `#9999999` clamp, write `d1` back to `(Score).w`, divide
        // both the old and new scores by `50000`, and only award lives for
        // boundaries crossed by that post-clamp value.
        let old_score = self.progression.score;
        self.progression.score = self.progression.score.saturating_add(points).min(SCORE_CAP);
        // ASM: award extra life for every 50,000 point boundary crossed.
        let old_lives = old_score / SCORE_EXTRA_LIFE_INTERVAL;
        let new_lives = self.progression.score / SCORE_EXTRA_LIFE_INTERVAL;
        for _ in old_lives..new_lives {
            self.add_life();
        }
        self.arm_hundred_k_trip_timer_loc_10_ac2();
    }

    fn arm_hundred_k_trip_timer_loc_10_ac2(&mut self) {
        // ASM loc_10AC2 (kid.asm:22375): once Prize10000Points_Init has
        // written the new score, arm `$FFFFFB4C` for 60 frames only when the
        // score is at least `100000`, the current level is
        // `HundredKTripStart_LevelID`, and the timer is still zero.
        if self.progression.score < 100_000 {
            return;
        }
        if self.current_level != HUNDRED_K_TRIP_START_LEVEL_ID {
            return;
        }
        if self.hundred_k_trip_timer != 0 {
            return;
        }
        self.hundred_k_trip_timer = HUNDRED_K_TRIP_ARM_FRAMES_LOC_10AC2;
    }

    /// Increment diamond count (capped at 99 per ASM). Awards extra life at 50/100.
    pub(crate) fn collect_diamond(&mut self) {
        if self.progression.diamond_count >= 99 {
            return;
        }
        self.progression.diamond_count += 1;
        self.add_score(500);
        if self.progression.diamond_count.is_multiple_of(50) {
            self.add_life();
        }
    }

    fn award_prize_diamond(&mut self) {
        if self.progression.diamond_count >= 99 {
            return;
        }
        self.level_prize_collected = true;
        self.progression.diamond_count += 1;
    }

    pub(crate) fn queue_prize_diamond_award(&mut self) {
        self.pending_prize_diamond_awards
            .push(Self::PRIZE_DIAMOND_AWARD_DELAY_FRAMES);
    }

    pub(crate) fn step_pending_prize_diamond_awards(&mut self) {
        let mut awards_to_apply = 0usize;
        for delay in &mut self.pending_prize_diamond_awards {
            *delay = delay.saturating_sub(1);
            if *delay == 0 {
                awards_to_apply += 1;
            }
        }
        self.pending_prize_diamond_awards.retain(|delay| *delay > 0);
        for _ in 0..awards_to_apply {
            self.award_prize_diamond();
        }
    }

    pub(crate) fn queue_mini_hopping_skull_scream(&mut self, suppress: bool) {
        self.pending_sound_effects.push(PendingSoundEffect {
            effect: QueuedSoundEffect::MiniHoppingSkullScream,
            delay_frames: Self::MINI_HOPPING_SKULL_SCREAM_DELAY_FRAMES,
            suppress,
        });
    }

    pub(crate) fn step_pending_sound_effects(&mut self) {
        let mut played_effects = Vec::new();
        for pending in &mut self.pending_sound_effects {
            pending.delay_frames = pending.delay_frames.saturating_sub(1);
            if pending.delay_frames == 0 && !pending.suppress {
                played_effects.push(pending.effect);
            }
        }
        // ASM Mini_Hopping_Skull.asm loc_3F406: after the optional scream branch,
        // the transient worker always jumps to `j_Delete_CurrentObject`.
        self.pending_sound_effects
            .retain(|pending| pending.delay_frames > 0);
        self.played_sound_effects.extend(played_effects);
    }

    #[cfg_attr(not(test), allow(dead_code))]
    #[allow(non_snake_case)]
    pub(crate) fn play_thunderstorm_sound_sub_1B36C(&mut self, d0: u16) -> u16 {
        // ASM sub_1B36C (kid.asm:29577-29583) saves d0 on the stack, plays
        // `sfx_Thunderstorm`, then restores d0 before returning.
        self.played_sound_effects
            .push(QueuedSoundEffect::Thunderstorm);
        d0
    }

    #[cfg_attr(not(test), allow(dead_code))]
    #[allow(non_snake_case)]
    pub(crate) fn play_navigate_jingle_sub_1BC26(&mut self, d0: u16) -> u16 {
        // ASM sub_1BC26 (kid.asm:30506-30512) saves d0 on the stack, plays
        // `sfx_Navigate_jingle`, then restores d0 before returning.
        self.played_sound_effects
            .push(QueuedSoundEffect::NavigateJingle);
        d0
    }

    pub(crate) fn apply_prize_content(&mut self, content: terrain::PrizeContent) {
        // ASM: NoPrize_Bonus_Flag is set whenever any prize block content is collected.
        self.level_prize_collected = true;
        match content {
            terrain::PrizeContent::Diamond => {
                self.collect_diamond();
            }
            terrain::PrizeContent::TenDiamonds => {
                for _ in 0..10 {
                    self.collect_diamond();
                }
            }
            terrain::PrizeContent::Clock => {
                self.level_timer.add_clock_bonus();
                // ASM loc_10002 (kid.asm:21298) increments `Clocks_collected`
                // after the timer bonus/sub-second reset path completes.
                self.clocks_collected += 1;
            }
            terrain::PrizeContent::Continue => {
                self.progression.number_continues =
                    self.progression.number_continues.saturating_add(1);
            }
            terrain::PrizeContent::Ankh => {
                self.add_life();
            }
            terrain::PrizeContent::Points10000 => {
                self.add_score(10_000);
            }
            terrain::PrizeContent::Skycutter => self.queue_form_change(PlayerForm::Skycutter),
            terrain::PrizeContent::Cyclone => self.queue_form_change(PlayerForm::Cyclone),
            terrain::PrizeContent::RedStealth => self.queue_form_change(PlayerForm::RedStealth),
            terrain::PrizeContent::Eyeclops => self.queue_form_change(PlayerForm::Eyeclops),
            terrain::PrizeContent::Juggernaut => self.queue_form_change(PlayerForm::Juggernaut),
            terrain::PrizeContent::IronKnight => self.queue_form_change(PlayerForm::IronKnight),
            terrain::PrizeContent::Berzerker => self.queue_form_change(PlayerForm::Berzerker),
            terrain::PrizeContent::Maniaxe => self.queue_form_change(PlayerForm::Maniaxe),
            terrain::PrizeContent::Micromax => self.queue_form_change(PlayerForm::Micromax),
        }
    }

    /// Returns the x_pos and family of the damaging enemy if contact damage should apply.
    pub(crate) fn find_contact_damage_source(
        &self,
        slice: &OpeningSliceState,
    ) -> Option<(u32, EnemyFamily)> {
        if self.progression.invulnerable_frames != 0
            || !matches!(self.player_lifecycle, PlayerLifecycle::Alive)
        {
            return None;
        }
        // ASM loc_B678 (kid.asm:14582): kid-left/right-of-enemy dispatch inside
        // Character_CheckCollision. `cmpi.w #$18,d5` — if enemy is tornado
        // ($18), branch to loc_B69E which checks Cyclone_flying: if flying,
        // ignore contact (rts); otherwise fall through to Check_for_recent_damge.
        // If not tornado: `tst.b (Berzerker_charging).w; beq.w Check_for_recent_damge`
        // — if NOT berzerker charging, take damage normally.
        // If berzerker IS charging: `bclr #$F,d6; bne.w loc_B692` — bit 15 of
        // object_meta flags whether to negate x_vel on hit. loc_B692:
        // `move.l x_vel(a3),d7; neg.l d7; move.l d7,x_vel(a3); rts` — bounce.
        // Bit 15 clear → return_B690 (rts, no velocity change).
        // Native: berzerker_charging immunity is handled here; the tornado
        // and velocity-negation sub-paths are not yet implemented.
        //
        // ASM loc_B672 (kid.asm:14576): unexpected collision type dispatch.
        // Character_CheckCollision's off_B640 jump table maps leftwall, floor,
        // ceiling, slopeup, and slopedown collision_type values (which
        // "shouldn't happen here") to loc_B672: `jmp (j_loc_6E2).w` — restart
        // the current level. Only rightwall maps to Crushed_to_Death, and
        // kid-above-enemy maps to Jump_On_Enemy. Native: the decomposed
        // collision architecture never produces these compound collision types
        // in the character-vs-object context, so the restart fallback is moot.
        //
        // Berzerker charging is immune to contact damage (they damage enemies instead)
        if self.helmet.berzerker_charging {
            return None;
        }

        self.active_enemies.iter().find_map(|enemy| {
            // All enemies damage on contact except collectible diamonds
            if matches!(enemy.family, EnemyFamily::Diamond) {
                return None;
            }
            // Skip invulnerable enemies (e.g. submerged TarMonster)
            if !enemy.is_vulnerable() {
                return None;
            }
            let snapshot = enemy.snapshot();
            if player_touches_enemy_with_half_width(
                slice.player,
                &snapshot,
                self.progression.current_form.half_width() as i64,
            ) {
                Some((enemy.x_pos, enemy.family))
            } else {
                None
            }
        })
    }

    #[allow(clippy::unused_self, reason = "semantically a method on game state")]
    pub(crate) fn fell_below_level(&self, slice: &OpeningSliceState) -> bool {
        let player_y_px = slice.player.y_pos >> 16;
        player_y_px > u32::from(slice.level_bounds.height_pixels)
    }

    pub(crate) fn player_touches_flag(&self, slice: &OpeningSliceState) -> bool {
        let Some(header) = self.level_header(self.current_level) else {
            return false;
        };
        let flag_x_fp = u32::from(header.flag_x) << 16;
        let flag_y_fp = u32::from(header.flag_y) << 16;
        let half_fp = FLAG_HIT_HALF_SIZE << 16;
        let dx = slice.player.x_pos.abs_diff(flag_x_fp);
        let dy = slice.player.y_pos.abs_diff(flag_y_fp);
        dx <= half_fp && dy <= half_fp
    }

    /// Apply all end-of-level bonuses per ASM (kid.asm:16650-16900).
    pub(crate) fn apply_level_completion_bonuses(&mut self) {
        // Time bonus: remaining seconds × 100
        self.add_score(u32::from(self.level_timer.seconds) * 100);

        // No-hit bonus: 5000 points if no damage taken during level
        if !self.level_damage_taken {
            self.add_score(5000);
        }

        // ASM loc_D5D2 (kid.asm:16732): NoPrize_Bonus_Flag = 0 means bonus
        // retained, so the EOL builder adds a 5000-point "No Prize" bonus.
        if !self.level_prize_collected {
            self.add_score(5000);
        }

        // Path and speed bonuses only apply to non-Elsewhere levels (< 0x49 in map order).
        let level_idx = self.current_level.0 as usize;
        if self.current_level.0 < FIRST_ELSEWHERE_LEVEL_ID {
            // ASM loc_D60E (kid.asm:16750): before Elsewhere, index
            // PathBonus_Values by Current_LevelID and multiply the byte by
            // 1000; zero skips the Path Bonus object.
            if level_idx < PATH_BONUS_TABLE.len() {
                let path_mult = PATH_BONUS_TABLE[level_idx] as u32;
                if path_mult > 0 {
                    self.add_score(path_mult * 1000);
                }
            }

            // ASM loc_D660 (kid.asm:16775): before Elsewhere, index
            // SpeedBonus_Values by Current_LevelID and skip only when
            // Level_completion_time is strictly greater than the threshold.
            // Passing the gate allocates the 10,000-point Speed Bonus object.
            //
            // ASM loc_C0CC (kid.asm:15687): reconstruct remaining seconds from
            // `Time_Seconds_low_digit + Time_Seconds_high_digit*$A +
            // Time_Minutes*$3C`, then `neg.w d0; addi.w #$B4,d0` →
            // elapsed = 180 - remaining.  `Clocks_collected * $B4` is added.
            //
            // ASM loc_C10A (kid.asm:15707): `cmpi.w #$FF,d0; ble.s loc_C10A;
            // move.w #$FF,d0` caps the value at 255, then
            // `move.b d0,(Level_completion_time).w` stores the result.
            if level_idx < SPEED_BONUS_TABLE.len() {
                let threshold = SPEED_BONUS_TABLE[level_idx];
                if threshold > 0 {
                    let elapsed_secs = LEVEL_TIMER_START_SECONDS
                        .saturating_sub(self.level_timer.seconds)
                        .saturating_add(self.clocks_collected * LEVEL_TIMER_START_SECONDS);
                    let completion_time = elapsed_secs.min(255) as u8;
                    if completion_time <= threshold {
                        self.add_score(10_000);
                    }
                }
            }
        }
    }

    /// Check if the player is standing on a teleporter block and return the destination.
    ///
    /// ASM loc_BC80 (kid.asm:15219): telepad destination lookup entry.
    /// `move.w ($FFFFF8DE).w,d7; beq loc_BCAC` — load teleporter count from
    /// block list header; zero count → loc_BCAC (j_loc_6E2 = lose life).
    /// Native: empty teleporters vec → return None (death path unreachable).
    pub(crate) fn check_teleporter_contact(
        &self,
        slice: &OpeningSliceState,
    ) -> Option<(u8, u16, u8)> {
        let block_list = self.block_list.as_ref()?;
        // ASM loc_BCAC (kid.asm:15237): `jmp (j_loc_6E2)` — no teleporter
        // entries means no match is possible; ASM kills player, native
        // returns None (the teleporter contact path is simply not taken).
        if block_list.teleporters.is_empty() {
            return None;
        }
        // Player's feet position in cell coordinates
        let px = (slice.player.x_pos >> 16) as i32;
        let py = (slice.player.y_pos >> 16) as i32;
        let col = terrain::pixel_to_cell(px) as u16;
        let row = terrain::pixel_to_cell(py) as u16;

        block_list
            .teleporter_at(col, row)
            .map(|t| (t.dest_map_id, t.dest_x, t.dest_y))
    }

    pub(crate) fn queue_form_change(&mut self, form: PlayerForm) {
        self.progression.pending_form = Some(form);
    }

    pub(crate) fn resolve_pending_form_change(&mut self) {
        let Some(form) = self.progression.pending_form.take() else {
            return;
        };
        let old_form = self.progression.current_form;
        self.progression.current_form = form;
        // ASM loc_7C76 (kid.asm:8387): `move.w (Current_Helmet).w,d7` — main
        // dispatch after sub_7C1C GFX deletion. Three paths:
        //   d7==0 (TheKid): `beq.w loc_7CD0` — skip reverse animation, go to
        //     put-on-new phase directly.
        //   d7==Current_Helmet_Available (same helmet): play sfx_Replenish_Health,
        //     `bra.w loc_7E14` — skip both animation phases, jump to cleanup.
        //   d7!=Current_Helmet_Available: fall through to loc_7C96 for reverse.
        // Native collapses these into the frame counter formula below.
        // TODO(fidelity): ASM same-helmet replenish path (loc_7C76 → loc_7E14)
        // has no animation frames; native computes old_frames+new_frames+1.
        self.progression.number_hitpoints =
            transform_hitpoints_sub_7eb2(form, self.progression.extra_hitpoint_slots);
        // ASM Kid_Transform calls sub_B41C which sets per-helmet half-width.
        // During Hibernate frames, the GFX collision system resolves any overlap
        // between the new wider hitbox and adjacent walls by pushing x_pos.
        let width_delta = form.half_width() - old_form.half_width();
        if width_delta != 0 {
            self.progression.form_width_delta = width_delta;
        }
        // ASM loc_7C96 (kid.asm:8399): `addq.w #1,d7; add.w d7,d7` — remove-
        // old-helmet animation setup. Indexes off_7EE2[old_helmet] to find the
        // end of the old helmet's animation table, then iterates backwards.
        //
        // ASM loc_7CA6 (kid.asm:8407): `move.w -(a0),d7; bmi.w loc_7CD0` —
        // inner loop reading animation entries in reverse. Each entry has a repeat
        // count (high byte) and function index (low byte, into off_7D34 dispatch).
        // The $FFFF sentinel terminates and falls through to loc_7CD0.
        //
        // ASM loc_7CBE (kid.asm:8418): `jsr (j_sub_8E0).w; dbf d0,loc_7CBE` —
        // per-frame Hibernate wait loop inside the reverse animation. Calls
        // j_sub_8E0 (VBlank/palette DMA) `d0+1` times, then returns to loc_7CA6
        // for the next entry. Native doesn't run a coroutine; the total repeat
        // counts are pre-summed into form_change_flash_frames.
        //
        // ASM loc_8FE (kid.asm:1015): second half of sub_8E0 —
        // `jsr j_Transfer_ScrollDataToVRAM; bsr.w sub_1596;
        // jsr j_Make_SpriteAttr_HUD; move.b #1,($FFFFFAD6).w; rts`.
        // Transfers scroll/tile data to VDP, drains custom FG update queue
        // (sub_1596), and builds HUD sprite attributes. Native models sub_1596
        // as drain_fg_update_queue_custom_sub_1596; the VDP DMA and HUD sprite
        // generation are handled by the rendering pipeline outside kid-core.
        //
        // ASM loc_7CD0 (kid.asm:8426): `move.w (Current_Helmet_Available).w,
        // (Current_Helmet).w` — put-on-new-helmet phase. Sets Current_Helmet to
        // the target, plays per-helmet transform sound (unk_7ED8), then walks
        // the new helmet's off_7EE2 table forward via loc_7D0A/loc_7D22.
        // Native sets current_form above; frame counting below covers both phases.
        let old_frames = transform_anim_frames(old_form);
        let new_frames = transform_anim_frames(form);
        // +1 for the Kid_Transform entry frame (sub_7C1C + palette save +
        // condition checks + initial Hibernate) before the animation loop starts.
        // ASM loc_7C24 (kid.asm:8350): `move.l 4(a0),d7; beq.w loc_7C3A;
        // move.l d7,a0; move.l $C(a0),a0; jsr Delete_Object_a0; bra.s loc_7C24`
        // — sub_7C1C walks the linked object list at $FFFFF862, deleting each
        // child's GFX object ($C(a0)) before transform. Native doesn't model
        // linked GFX child objects, so this is a no-op here.
        self.progression.form_change_flash_frames = (old_frames + new_frames + 1) as u8;
        // ASM loc_7E26 (kid.asm:8564): `move.b (a4,d0.w),d0; jsr (sub_E1330).l;
        // jsr (sub_E1338).l` — index unk_7ECC[Foreground_theme] (or
        // MusicSpeed_Murderwall if MurderWall_flag set) for the post-transform
        // music speed byte, then queue it via sub_E1330 and reset via sub_E1338.
        // Sound-only; native doesn't drive the GEMS sound driver.
        //
        // ASM loc_7E48 (kid.asm:8575): cleanup after the optional
        // sfx_Voice_bummer (TheKid only). `bsr.w sub_80D0` (palette load),
        // `bsr.w sub_B41C` (set half-width from d0=1), `bsr.w sub_DB22`
        // (clear Maniaxe/RedStealth/Berzerker flags), `clr.w ($FFFFFB70).w`
        // (reset walk accumulator), then reset both Kid and companion GFX
        // objects' sprite/visibility fields. Native covers half-width via
        // form_width_delta, flag clearing via helmet.reset(), and walk
        // accumulator implicitly.
        //
        // ASM loc_7E8C (kid.asm:8594): `st $13(a4)` — show companion GFX
        // object for Skycutter (direct beq.s) and Juggernaut (which sets
        // companion sprite to LnkTo_unk_BEDF0 before falling through).
        // Visual-only; native doesn't model per-frame companion GFX objects.
        //
        // ASM loc_7E90 (kid.asm:8597): `sf (Check_Helmet_Change).w;
        // clr.l x_vel(a3); clr.l y_vel(a3); sf (Currently_transforming).w`
        // — clear transform-in-progress flags and zero both velocity longwords.
        // Then restore 3 palette words from the stack and rts via $44(a5).
        // Native zeroes velocity below; pending_form.take() above clears the
        // transform request; form_change_flash_frames drives the flash overlay.
        if let Some(ref mut slice) = self.opening_slice {
            slice.player.x_velocity = 0;
            slice.player.y_velocity = 0;
        }
        // TODO(fidelity): visual sprite art for transform animation.
        // ASM uses per-helmet sprite+palette cycling tables for a spinning
        // animation. Currently renders as a flash overlay. Needs ROM sprite
        // tile extraction for real visuals.
        //
        // ASM loc_7CFE (kid.asm:8439): `add.w d7,d7; lea off_7EE2(pc),a0;
        // move.w (a0,d7.w),d7; add.w d7,a0` — forward animation table setup.
        // Indexes off_7EE2[Current_Helmet] to find the new helmet's animation
        // table start, then falls through to loc_7D0A forward loop. Each entry
        // is 4 bytes: anim_frame(mode, duration, spriteframe). The mode byte
        // (high byte of first word) is the hibernate repeat count; the duration
        // byte (low byte) is the dispatch index into off_7D34. Native pre-sums
        // repeat counts into form_change_flash_frames above.
        //
        // ASM off_7D34 dispatch table (kid.asm:8463): 4 visual-only handlers
        // called per animation entry during both reverse and forward phases:
        //
        //   off_7D34[0] = loc_7D80 (kid.asm:8494): main handler — set Kid
        //     sprite from d6 (low 15 bits). If bit 15 set (from loc_7D7C),
        //     position child GFX at Kid (x, y-14), flip direction for
        //     RedStealth/IronKnight/Berzerker, copy palette. Then load TheKid
        //     base palette (sub_80D0 with d7=0) and 3-word secondary palette
        //     (sub_813C from off_8162[Current_Helmet]).
        //
        //   off_7D34[4] = loc_7D44 (kid.asm:8469): hide child GFX ($13=0),
        //     set Kid sprite (addroffset_sprite = d7), load 15-word palette
        //     via sub_8106(Current_Helmet) from off_8128.
        //
        //   off_7D34[8] = loc_7D5A (kid.asm:8478): hide child GFX, set Kid
        //     sprite to standing frame from off_79B2[Current_Helmet], load
        //     12-word palette via sub_80D0(Current_Helmet) from off_80F2.
        //
        //   off_7D34[$C] = loc_7D7C (kid.asm:8491): `ori.w #$8000,d7` — set
        //     bit 15 ("show child GFX" flag), then fall through to loc_7D80.
        //
        // None of these affect gameplay state — they only update sprite
        // mappings, palette buffer entries, and child GFX object visibility.
        // Native covers their duration contribution via transform_anim_frames.
        //
        // TheKid (helmet 0) has no animation table — off_7EE2[0] points to
        // sub_80BC (kid.asm:8817), which fills Palette_Buffer+$62 with 12
        // words of $FFF (white). ASM loc_80C8 (kid.asm:8823): inner loop
        // `move.w d6,(a2)+; dbf d7,loc_80C8`. Visual-only palette flash.
        //
        // ASM loc_80E8 (kid.asm:8842): inner copy loop of sub_80D0 —
        // `move.w (a4)+,(a2)+; dbf d7,loc_80E8` copies 12 palette words
        // from the helmet-indexed off_80F2 table into Palette_Buffer+$62.
        // Visual-only; called by loc_7D5A and loc_7D80 dispatch handlers.
        //
        // ASM loc_811E (kid.asm:8873): inner copy loop of sub_8106 —
        // `move.w (a4)+,(a2)+; dbf d7,loc_811E` copies 15 palette words
        // from the helmet-indexed off_8128 table into Palette_Buffer+$62.
        // Visual-only; called by loc_7D44 dispatch handler.
        // Clear ability state on form change
        self.helmet.reset();
    }

    pub(crate) fn apply_damage_with_knockback(
        &mut self,
        _slice: &mut OpeningSliceState,
        _enemy_x: u32,
    ) {
        // ASM does NOT apply knockback velocity on contact damage —
        // the player's velocity is unchanged. The perceived "push" in
        // the original comes from bounding-box separation, not an
        // impulse. Previous native code applied KNOCKBACK_X/Y_VELOCITY
        // which was an invented approximation.
        self.apply_damage();
    }

    pub(crate) fn apply_damage(&mut self) {
        // ASM loc_B700 (kid.asm:14638): entered from Check_for_recent_damge when
        // Just_received_damage is clear.  Loads HelmetHitpoint_Table[Current_Helmet]
        // + Extra_hitpoint_slots → max HP.  Forces Number_Hitpoints ≥ 1, then caps
        // it to max (loc_B722).  Sets NoHit_Bonus_Flag + Just_received_damage,
        // decrements HP.  If HP == 0 → loc_B786 (lose helmet or die).
        //
        // ASM loc_B722 (kid.asm:14650): HP cap — `cmp.w (Number_Hitpoints).w,d7;
        // bge.s loc_B72C; move.w d7,(Number_Hitpoints).w`.  Ensures current HP
        // never exceeds max for the current helmet + extra slots.  Native handles
        // this implicitly: HP is set via transform_hitpoints_sub_7eb2 on form
        // change and extra HP awards are capped at 4 slots (loc_3FBE4).
        //
        // ASM loc_B73C/loc_B750/loc_B764/loc_B778 (kid.asm:14661-14696): damage
        // sound dispatch.  Selects one of four "ouch" SFX based on enemy type (d5):
        //   d5 == $50 → sfx_Voice_ouch_1 ($6B, original recording)
        //   d5 == $40 → sfx_Voice_ouch_2 ($6C, slowed down)
        //   d5 == $30 → sfx_Voice_ouch_3 ($6D, slowed down)
        //   else      → sfx_Voice_ouch   ($61, default)
        // Native does not yet implement sound.
        self.level_damage_taken = true;
        // ASM loc_BC40 (kid.asm:15187): the damage-flash thread polls
        // Just_received_damage each hibernate; setting it here wakes the
        // thread which runs the 66-frame palette flash cycle (loc_BC56–
        // loc_BC6E) before clearing the flag.
        self.just_received_damage = true;
        // Clear wall-grab so the player isn't stuck to the wall during invulnerability
        if let Some(slice) = self.opening_slice.as_mut() {
            slice.player.right_wall_sticky = false;
            slice.player.left_wall_sticky = false;
            slice.player.wall_cling_direction = None;
        }
        if self.progression.current_form != PlayerForm::TheKid {
            self.queue_form_change(PlayerForm::TheKid);
            self.resolve_pending_form_change();
            self.progression.invulnerable_frames = PLAYER_DAMAGE_INVULNERABLE_FRAMES;
            return;
        }

        if self.progression.number_hitpoints > 1 {
            self.progression.number_hitpoints -= 1;
            self.progression.invulnerable_frames = PLAYER_DAMAGE_INVULNERABLE_FRAMES;
            return;
        }

        self.progression.number_hitpoints = 0;
        self.begin_death_windup();
    }

    pub(crate) fn begin_death_windup(&mut self) {
        self.paused = false;
        self.pause_toggle_pending = false;
        self.progression.invulnerable_frames = 0;
        self.death_camera_lock = self.opening_slice.map(|slice| slice.camera);
        self.death_shell_snapshot = self
            .opening_slice
            .map(|slice| self.build_death_shell_snapshot(&slice));
        self.player_lifecycle = PlayerLifecycle::DeathWindup {
            remaining_frames: PLAYER_DEATH_WINDUP_FRAMES,
        };
    }

    #[allow(non_snake_case)]
    pub(crate) fn sprite_vertical_extent_loc_11EA(
        use_alternate_extent: bool,
        default_extent_px: i32,
        alternate_extent_px: i32,
    ) -> i32 {
        // ASM loc_11EA (kid.asm:2205): when `$12(a3)` is non-zero,
        // `Make_SpriteFromGfxObject` loads the vertical extent from `4(a4)`
        // instead of the default `6(a4)` word before subtracting it from the
        // camera-relative Y position.
        if use_alternate_extent {
            alternate_extent_px
        } else {
            default_extent_px
        }
    }

    #[allow(non_snake_case)]
    pub(crate) fn sprite_visibility_window_loc_11EE(
        sprite_x_px: i32,
        sprite_y_px: i32,
        camera: CameraState,
        sprite_vertical_extent_px: i32,
    ) -> bool {
        // ASM loc_11EE (kid.asm:2208): `Make_SpriteFromGfxObject` subtracts
        // the selected sprite extent from `y_pos-Camera_Y_pos`, rejects
        // values above `$160` or below `$FF80`, then applies the same pattern
        // to `x_pos-Camera_X_pos` with horizontal limits `$160/$FFC0`.
        let y_screen_px = sprite_y_px - ((camera.y_pos >> 16) as i32) - sprite_vertical_extent_px;
        if !(-SPRITE_VISIBILITY_TOP_MARGIN_PX
            ..=physics::VIEWPORT_HEIGHT + SPRITE_VISIBILITY_BOTTOM_MARGIN_PX)
            .contains(&y_screen_px)
        {
            return false;
        }

        let x_screen_px = sprite_x_px - ((camera.x_pos >> 16) as i32);
        (-SPRITE_VISIBILITY_LEFT_MARGIN_PX
            ..=physics::VIEWPORT_WIDTH + SPRITE_VISIBILITY_RIGHT_MARGIN_PX)
            .contains(&x_screen_px)
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn sprite_priority_bit_loc_1234(
        priority_byte: u8,
        high_priority_hw_flag: bool,
    ) -> u16 {
        // ASM loc_1234 (kid.asm:2230-2231): sets the high-priority bit $8000
        // when sprite priority conditions are met. The priority field $10(a3)
        // controls how the tile ID is modified:
        //   cmpi.b #1,priority(a3)   ; priority field: $10(a3)
        //   bcs.s loc_1238           ; branch if priority < 1 (priority = 0)
        //   beq.s loc_1234           ; branch if priority == 1, set bit
        //   tst.b ($FFFFF896).w      ; check high-priority hardware flag
        //   beq.s loc_1238           ; skip if flag is 0
        // loc_1234:
        //   ori.w #$8000,d0          ; set high-priority bit
        // loc_1238:
        // Priority byte semantics:
        // - 0: low priority (no bit)
        // - 1: high priority (set bit unconditionally)
        // - 2+: conditional priority (set bit only if hw flag is set)
        match priority_byte {
            0 => 0,
            1 => 0x8000,
            _ if high_priority_hw_flag => 0x8000,
            _ => 0,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn sprite_tile_id_loc_1238(
        base_tile_id: u16,
        sprite_offset_word: u16,
        vram_tile: u16,
        use_hardware_offset: bool,
    ) -> u16 {
        // ASM loc_1238 (kid.asm:2233-2238): computes the final sprite tile ID by
        // adding offsets based on the object's $12(a3) field. When $12(a3) is
        // non-zero, the routine uses a hardware address instead of sprite data:
        // loc_1238:
        //   tst.b $12(a3)
        //   bne.s loc_1246
        //   add.w (a4)+,d0           ; add from sprite art data
        //   add.w vram_tile(a3),d0   ; add vram_tile field
        //   bra.s loc_124A
        // loc_1246:
        //   add.w ($FFFFF83C).w,d0   ; add from hardware offset table
        // loc_124A:
        // When use_hardware_offset is false, adds sprite_offset_word + vram_tile.
        // When true, the hardware offset path adds only vram_tile (the $F83C
        // lookup is handled separately in the calling context).
        if use_hardware_offset {
            base_tile_id.wrapping_add(vram_tile)
        } else {
            base_tile_id
                .wrapping_add(sprite_offset_word)
                .wrapping_add(vram_tile)
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn sprite_dma_upload_loc_141A(
        size_code: u8,
        sprite_tile_word: u16,
        hardware_tile_word: u16,
        dma_source: u32,
    ) -> SpriteDmaUploadLoc141A {
        const NUMBER_TILES_PER_SPRITE_SIZE_LOC_141A: [u16; 16] =
            [0, 1, 2, 3, 1, 3, 5, 7, 2, 5, 8, 0x0B, 3, 7, 0x0B, 0x0F];

        // ASM loc_141A (kid.asm:2491-2501): the `$12(a3) != 0` sprite path in
        // `sub_13C0` reuses `(FFFFF830).w` as the emitted sprite's tile word,
        // then looks up the chunk length from `NumberTilesPerSpriteSize`,
        // adds one tile, advances both `$FFFFF830/$FFFFF83C`, queues that
        // tile count plus the current art pointer for DMA, and finally bumps
        // `a4` by `tile_count << 5` bytes.
        let dma_tile_count =
            NUMBER_TILES_PER_SPRITE_SIZE_LOC_141A[usize::from(size_code & 0x0F)].wrapping_add(1);

        SpriteDmaUploadLoc141A {
            sprite_tile_word,
            dma_tile_count,
            next_sprite_tile_word: sprite_tile_word.wrapping_add(dma_tile_count),
            next_hardware_tile_word: hardware_tile_word.wrapping_add(dma_tile_count),
            dma_source,
            next_dma_source: dma_source.wrapping_add(u32::from(dma_tile_count) << 5),
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn sprite_special_y_offset_loc_1286(sprite_offset: u16, y_screen_px: i32) -> i32 {
        const MANIAXE_AXE_FRAME_3_SPRITE_OFFSET_LOC_1286: u16 = 0x00B0;

        // ASM loc_1286 (kid.asm:2268): once the non-Kid `$17(a3)` path falls
        // through from `loc_124A`, only `addroffset_sprite(a3) ==
        // LnkTo_unk_A59AA-Data_Index` gets the extra `subq.w #6,d7` Maniaxe
        // axe lift before control rejoins `loc_1292`.
        if sprite_offset == MANIAXE_AXE_FRAME_3_SPRITE_OFFSET_LOC_1286 {
            y_screen_px.wrapping_sub(6)
        } else {
            y_screen_px
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn sprite_uses_x_flip_loc_1292(x_direction: u8) -> bool {
        // ASM loc_1292 (kid.asm:2273): `tst.b x_direction(a3)` branches to the
        // x-flipped renderer at `loc_132A` for any non-zero byte and reaches
        // the normal-orientation path at `loc_129E` only when the byte is 0.
        x_direction != 0
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn sprite_normal_orientation_chunks_loc_12B8(
        base_x_screen_px: i16,
        base_y_screen_px: i16,
        width_px: i16,
        height_px: i16,
    ) -> Vec<NormalSpriteChunkLoc12B8> {
        const CHUNK_SPAN_LOC_12B8: i16 = 0x20;

        // ASM loc_12B8 (kid.asm:2296-2335): outer row loop for the non-flipped
        // sprite path.
        // ASM loc_12C0: inner width loop keeps issuing `moveq #$F,d0` and
        // `bsr.w sub_13C0` while `d2 >= $20`, subtracting/advancing by 32px
        // each iteration before the remainder handling at `loc_12D0`.
        // ASM loc_12E6: after the width remainder call returns, subtract the
        // 32px row span from `d5`, add it to `d7`, and branch back to
        // `loc_12B8`, so each completed row advances vertically before the
        // next width pass starts.
        // ASM loc_12EC: once fewer than 32px of height remain, round the tail
        // row up to the next 8px multiple, emit `$20`-wide strips with
        // `ori.w #$C,d0`, then finish with the combined width/height corner
        // chunk before `return_1328`.
        // The combined routine emits as many 32x32 chunks as possible first,
        // rounds each leftover axis up to the next multiple of 8, then encodes
        // the width and height remainder bits into the final `sub_13C0` size
        // byte.
        let mut emitted = Vec::new();
        let mut row_y = base_y_screen_px;
        let mut remaining_height = height_px;

        while remaining_height >= CHUNK_SPAN_LOC_12B8 {
            let mut column_x = base_x_screen_px;
            let mut remaining_width = width_px;

            while remaining_width >= CHUNK_SPAN_LOC_12B8 {
                emitted.push(NormalSpriteChunkLoc12B8 {
                    x_screen_px: column_x,
                    y_screen_px: row_y,
                    size_code: 0x0F,
                });
                remaining_width -= CHUNK_SPAN_LOC_12B8;
                column_x = column_x.wrapping_add(CHUNK_SPAN_LOC_12B8);
            }

            let rounded_width = (remaining_width + 7) & !7;
            if rounded_width != 0 {
                emitted.push(NormalSpriteChunkLoc12B8 {
                    x_screen_px: column_x,
                    y_screen_px: row_y,
                    size_code: (((rounded_width - 8) >> 1) | 0x0003) as u8,
                });
            }

            remaining_height -= CHUNK_SPAN_LOC_12B8;
            row_y = row_y.wrapping_add(CHUNK_SPAN_LOC_12B8);
        }

        // ASM loc_1380 (kid.asm:2411-2414): once the x-flipped path reaches a
        // tail shorter than `$20`, it rounds the remaining height up to the
        // next 8px multiple before entering `loc_1388`'s full-width strip
        // loop or `loc_13A2`'s closing corner chunk.
        let rounded_height = (remaining_height + 7) & !7;
        if rounded_height == 0 {
            return emitted;
        }

        let mut column_x = base_x_screen_px;
        let mut remaining_width = width_px;

        // ASM loc_12F4 (kid.asm:2333-2345): while the rounded tail height in
        // `d5` stays fixed, keep emitting `$20`-wide strips with
        // `ori.w #$C,d0`, then subtract/add `$20` from the remaining width and
        // screen X before looping back to `loc_12F4`.
        while remaining_width >= CHUNK_SPAN_LOC_12B8 {
            emitted.push(NormalSpriteChunkLoc12B8 {
                x_screen_px: column_x,
                y_screen_px: row_y,
                size_code: (((rounded_height - 8) >> 3) | 0x000C) as u8,
            });
            remaining_width -= CHUNK_SPAN_LOC_12B8;
            column_x = column_x.wrapping_add(CHUNK_SPAN_LOC_12B8);
        }

        let rounded_width = (remaining_width + 7) & !7;
        if rounded_width != 0 {
            // ASM loc_130E (kid.asm:2347-2355): round the final width
            // remainder up to the next 8px multiple, fold its encoded bits
            // together with the fixed tail height remainder from `d6`, and
            // issue the closing `sub_13C0` corner chunk before `return_1328`.
            emitted.push(NormalSpriteChunkLoc12B8 {
                x_screen_px: column_x,
                y_screen_px: row_y,
                size_code: (((rounded_width - 8) >> 1) | ((rounded_height - 8) >> 3)) as u8,
            });
        }

        emitted
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn sprite_x_flipped_chunks_loc_134A(
        base_x_screen_px: i16,
        base_y_screen_px: i16,
        width_px: i16,
        height_px: i16,
    ) -> Vec<NormalSpriteChunkLoc12B8> {
        const CHUNK_SPAN_LOC_134A: i16 = 0x20;

        // ASM loc_134A (kid.asm:2378-2408): the x-flipped sprite path reuses
        // the same 32px row cadence as `loc_12B8`, but each width pass starts
        // from the current right edge in `d6` and subtracts the emitted chunk
        // width from `d3` before every `sub_13C0` call. `loc_1352` therefore
        // walks full 32x32 chunks from right to left, `loc_1362` rounds the
        // tail width up to the next 8px multiple, and `loc_137A` advances to
        // the next row once the row height budget in `d5` loses another `$20`.
        // Once fewer than 32px of height remain, `loc_1388`/`loc_13A2` keep
        // the same right-to-left layout with `$20`-wide vertical strips and a
        // final rounded corner chunk.
        let mut emitted = Vec::new();
        let mut row_y = base_y_screen_px;
        let mut remaining_height = height_px;

        while remaining_height >= CHUNK_SPAN_LOC_134A {
            let mut row_right_edge = base_x_screen_px;
            let mut remaining_width = width_px;

            while remaining_width >= CHUNK_SPAN_LOC_134A {
                row_right_edge -= CHUNK_SPAN_LOC_134A;
                emitted.push(NormalSpriteChunkLoc12B8 {
                    x_screen_px: row_right_edge,
                    y_screen_px: row_y,
                    size_code: 0x0F,
                });
                remaining_width -= CHUNK_SPAN_LOC_134A;
            }

            let rounded_width = (remaining_width + 7) & !7;
            if rounded_width != 0 {
                row_right_edge -= rounded_width;
                emitted.push(NormalSpriteChunkLoc12B8 {
                    x_screen_px: row_right_edge,
                    y_screen_px: row_y,
                    size_code: (((rounded_width - 8) >> 1) | 0x0003) as u8,
                });
            }

            remaining_height -= CHUNK_SPAN_LOC_134A;
            row_y = row_y.wrapping_add(CHUNK_SPAN_LOC_134A);
        }

        let rounded_height = (remaining_height + 7) & !7;
        if rounded_height == 0 {
            return emitted;
        }

        let mut row_right_edge = base_x_screen_px;
        let mut remaining_width = width_px;

        while remaining_width >= CHUNK_SPAN_LOC_134A {
            row_right_edge -= CHUNK_SPAN_LOC_134A;
            emitted.push(NormalSpriteChunkLoc12B8 {
                x_screen_px: row_right_edge,
                y_screen_px: row_y,
                size_code: (((rounded_height - 8) >> 3) | 0x000C) as u8,
            });
            remaining_width -= CHUNK_SPAN_LOC_134A;
        }

        let rounded_width = (remaining_width + 7) & !7;
        if rounded_width != 0 {
            row_right_edge -= rounded_width;
            emitted.push(NormalSpriteChunkLoc12B8 {
                x_screen_px: row_right_edge,
                y_screen_px: row_y,
                size_code: (((rounded_width - 8) >> 1) | ((rounded_height - 8) >> 3)) as u8,
            });
        }

        emitted
    }

    fn death_sprite_still_visible(player: PlayerState, camera: CameraState) -> bool {
        let sprite_vertical_extent_px = Self::sprite_vertical_extent_loc_11EA(
            true,
            PLAYER_HEIGHT,
            PLAYER_DEATH_SPRITE_Y_EXTENT_PX,
        );
        Self::sprite_visibility_window_loc_11EE(
            (player.x_pos >> 16) as i32,
            (player.y_pos >> 16) as i32,
            camera,
            sprite_vertical_extent_px,
        )
    }

    fn build_death_shell_snapshot(&self, slice: &OpeningSliceState) -> LevelSkipTransitionState {
        // ASM `Death` (kid.asm:14714-14760) has two death animation paths
        // selected by d7:
        //
        // d7 != 0 → loc_B7CA: gravity-only fall using current y_vel with
        //   $4000 gravity per frame, y-axis only (no x movement).  Only
        //   loc_9142 (sub_902A pit-death) sets d7=1.
        //
        // d7 == 0 → loc_B7EA: strip helmet if any (clr.w
        //   Current_Helmet_Available; bsr.w Kid_Transform), then fall
        //   through to loc_B7FA which launches the tumble arc:
        //   d0=$8000 (x step), d1=$FFFC0000 (initial y velocity),
        //   death sprite = LnkTo_unk_A3FF8.
        //
        // Both paths loop via j_sub_8E0 until $19(a3) signals the sprite
        // is offscreen, then branch to `lose_life`.
        //
        // Native always uses the loc_B7FA/loc_B80C tumble arc — the
        // loc_B7CA gravity-fall path is not yet differentiated.
        // TODO(fidelity): loc_B7CA gravity-fall uses $4000 per frame
        // (not $3000) and moves y-only; pit-death should use this path.
        let mut frozen_player = slice.player;
        // ASM loc_B7FA (kid.asm:14746): move.l #$FFFC0000,d1
        let mut death_y_velocity = -0x0004_0000;

        loop {
            // ASM loc_B80C (kid.asm:14751): add.l d0,x_pos(a3)
            frozen_player.x_pos = frozen_player.x_pos.wrapping_add(0x0000_8000);
            frozen_player.y_pos = frozen_player.y_pos.wrapping_add_signed(death_y_velocity);
            if !Self::death_sprite_still_visible(frozen_player, slice.camera) {
                break;
            }
            // ASM loc_B80C: addi.l #$3000,d1 (tumble gravity)
            death_y_velocity = death_y_velocity.saturating_add(0x0000_3000);
        }

        // ASM Death loc_B80C exits after the hidden render pass, but the
        // oracle-backed gameplay capture samples the latched shell pose one
        // half-step earlier in X and one 4px carry-forward later in Y.
        frozen_player.x_pos = frozen_player.x_pos.wrapping_sub(0x0000_8000);
        frozen_player.y_pos = frozen_player.y_pos.wrapping_add(0x0004_0000);

        frozen_player.x_velocity = 0;
        frozen_player.y_velocity = 0;
        frozen_player.on_ground = true;
        frozen_player.just_jumped = false;
        frozen_player.jump_buffer_frames = 0;

        let mut frozen_active_objects = self.active_objects();
        if let Some(player_object) = frozen_active_objects
            .iter_mut()
            .find(|object| matches!(object.kind, NativeObjectKind::Player))
        {
            player_object.x_pos = frozen_player.x_pos;
            player_object.y_pos = frozen_player.y_pos;
        }

        let (base_number_objects, base_number_gfx_objects) = self.object_counts_from_roster(
            &frozen_active_objects,
            slice.camera.x_pos,
            slice.camera.y_pos,
        );

        LevelSkipTransitionState {
            elapsed_frames: 0,
            frozen_player,
            frozen_camera: slice.camera,
            frozen_active_objects,
            frozen_number_objects: base_number_objects.saturating_add(2),
            frozen_number_gfx_objects: base_number_gfx_objects.saturating_add(3),
        }
    }

    pub(crate) fn advance_death_windup_motion(&mut self) -> bool {
        if matches!(self.boot_state, BootState::LevelSkipHold) || self.paused {
            return false;
        }
        let bounds = self.level_bounds(self.current_level);
        let mut slice = self
            .opening_slice
            .unwrap_or_else(|| OpeningSliceState::new(bounds));
        let first_windup_frame = matches!(
            self.player_lifecycle,
            PlayerLifecycle::DeathWindup { remaining_frames }
                if remaining_frames == PLAYER_DEATH_WINDUP_FRAMES
        );
        if first_windup_frame {
            // ASM loc_B7FA (kid.asm:14746): move.l #$8000,d0; move.l
            // #$FFFC0000,d1 — tumble arc launch.  loc_B80C checks
            // visibility, then increments d1 by $3000 before applying the
            // step, so the first visible frame already moves by
            // x += $00008000 and y += initial_d1 + $3000.
            // NOTE: loc_B7CA (kid.asm:14729) is the alternate d7≠0 path
            // that uses $4000 gravity and y-only; not yet differentiated.
            slice.player.on_ground = false;
            // ASM loc_B7FA: move.l #$8000,d0
            slice.player.x_velocity = 0x0000_8000;
            // ASM loc_B7FA: move.l #$FFFC0000,d1
            slice.player.y_velocity = -0x0004_0000;
        }
        if let Some(camera_lock) = self.death_camera_lock {
            // ASM loc_B80C: j_sub_8E0 tests the current pose before the
            // per-frame gravity increment and tumble step. Once the current
            // death pose is offscreen, the loop falls straight into lose_life.
            if !Self::death_sprite_still_visible(slice.player, camera_lock) {
                self.opening_slice = Some(slice);
                return true;
            }
            slice.camera = camera_lock;
        }
        // ASM loc_B80C increases d1 by $3000 before applying the tumble step.
        slice.player.y_velocity = slice.player.y_velocity.saturating_add(0x0000_3000);
        let next_x = i64::from(slice.player.x_pos) + i64::from(slice.player.x_velocity);
        let next_y = i64::from(slice.player.y_pos) + i64::from(slice.player.y_velocity);
        slice.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
        slice.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
        slice.player.on_ground = false;
        self.opening_slice = Some(slice);
        false
    }

    pub(crate) fn enter_death_hidden_state(&mut self) {
        // ASM loc_B7EA (kid.asm:14740): d7==0 death path strips the helmet
        // before the tumble arc — `tst.w (Current_Helmet).w; beq.w loc_B7FA;
        // clr.w (Current_Helmet_Available).w; bsr.w Kid_Transform`.
        // Native defers the form reset to this function (post-windup) rather
        // than doing it before the visible tumble.
        self.progression.current_form = PlayerForm::TheKid;
        self.progression.pending_form = None;
        // ASM `lose_life` (kid.asm:14763): move.w #2,(Number_Hitpoints).w
        self.progression.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
        // ASM `lose_life` (kid.asm:14766): subq.w #1,(Number_Lives).w
        self.progression.number_lives = self.progression.number_lives.saturating_sub(1);
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.reset_shooter_state_sub_F06A();
        self.block_state.ghost_groups.clear();
        self.death_camera_lock = None;
        // ASM `Teleport` (kid.asm:14772-14794) then dispatches:
        //   tst.b (Two_player_flag).w; bne.w loc_B894
        // Single-player falls through to the Game_Mode=8 level-transition
        // path at loc_B8DE.
        //
        // ASM loc_B894 (kid.asm:14797): two-player dispatch — checks
        //   Player_1_Lives and Player_2_Lives to decide which player
        //   continues or whether to game-over.
        //
        // ASM loc_B8A8 (kid.asm:14804): within loc_B894 — when only one
        //   player has lives remaining, checks $FFFFFC29 (continue flag);
        //   if clear, sets Game_Mode=8 for game-over screen, then jumps
        //   to loc_B8DE (j_loc_6E2 level restart).
        //
        // ASM loc_B8BA (kid.asm:14811): both players still have lives —
        //   `tst.b ($FFFFFC29).w; bne.w loc_B8DE` (continue flag gate),
        //   `move.w #8,(Game_Mode).w` (level transition),
        //   `not.b (Current_player).w` (swap active player 0↔$FF),
        //   `bra.w loc_B8DE` (j_loc_6E2 restart). The NOT.b swaps the
        //   current player so the other player takes over.
        //
        // ASM loc_B8D0 (kid.asm:14819): P1 dead (Player_1_Lives==0),
        //   P2 still alive — `tst.w (Player_2_Lives).w; beq.w loc_B8E2`
        //   (both dead → game over via j_EntryPoint),
        //   `st (Current_player).w` (force P2 active),
        //   `bra.s loc_B8A8` (check continue flag + restart).
        //
        // Native: two-player mode is not yet fully implemented; the
        // single-player path is always taken.
        self.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: PLAYER_DEATH_HIDE_FRAMES,
        };
    }

    /// Reset all state and return to title screen.
    pub(crate) fn return_to_title(&mut self) {
        self.enter_title_screen();
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.progression.number_lives = 3;
        self.progression.number_continues = 3;
        self.progression.diamond_count = 0;
        self.progression.score = 0;
        self.current_level = LevelId(0);
        self.opening_slice = None;
        self.two_player = false;
        self.current_player_index = 0;
        self.player_slots = [PlayerSlot::default(), PlayerSlot::default()];
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyContactResult {
    None,
    Stomp,
    Damage,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EnemyContactBounds {
    pub(crate) left_px: i64,
    pub(crate) right_px: i64,
    pub(crate) top_px: i64,
    pub(crate) bottom_px: i64,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CollisionRectSub1F52 {
    pub(crate) left_px: i16,
    pub(crate) right_px: i16,
    pub(crate) top_px: i16,
    pub(crate) bottom_px: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CollisionSizeEntrySub21F8 {
    pub(crate) left_offset_px: i16,
    pub(crate) width_px: i16,
    pub(crate) top_offset_px: i16,
    pub(crate) height_px: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn kid_hitbox_from_collision_size_sub_21F8(
    previous_hitbox: CollisionRectSub1F52,
    x_anchor_px: i16,
    y_anchor_px: i16,
    mirrored: bool,
    collision_size: Option<CollisionSizeEntrySub21F8>,
) -> CollisionRectSub1F52 {
    // ASM sub_21F8 (kid.asm:4036): if `$22(a0)` is zero, collapse Kid's
    // hitbox to the current anchor point. Otherwise, load a CollisionSize
    // entry; a zero first word is a sentinel that leaves the prior hitbox
    // untouched. Non-mirrored entries use `left + width`, mirrored entries use
    // `-left` then subtract the width back toward the facing side.
    let Some(collision_size) = collision_size else {
        return CollisionRectSub1F52 {
            left_px: x_anchor_px,
            right_px: x_anchor_px,
            top_px: y_anchor_px,
            bottom_px: y_anchor_px,
        };
    };

    if collision_size.left_offset_px == 0 {
        // ASM loc_224C (kid.asm:4070-4072): a zero first word branches
        // straight to the common return without rewriting any hitbox edge, so
        // the prior rectangle stays live.
        return previous_hitbox;
    }

    let (left_px, right_px) = if mirrored {
        // ASM loc_2228 (kid.asm:4054-4059): negate the stored left offset,
        // anchor the facing-side edge first, then subtract the width back to
        // recover the opposite edge.
        let right_px = x_anchor_px.saturating_sub(collision_size.left_offset_px);
        let left_px = right_px.saturating_sub(collision_size.width_px);
        (left_px, right_px)
    } else {
        let left_px = x_anchor_px.saturating_add(collision_size.left_offset_px);
        let right_px = left_px.saturating_add(collision_size.width_px);
        (left_px, right_px)
    };
    // ASM loc_2232 (kid.asm:4060-4069): both facing branches rejoin here to
    // add the top offset from `$1E(a0)`, extend by height, and write the four
    // Kid hitbox edges from the shared `(d0,d1,d2,d3)` registers.
    let top_px = y_anchor_px.saturating_add(collision_size.top_offset_px);
    let bottom_px = top_px.saturating_add(collision_size.height_px);

    CollisionRectSub1F52 {
        left_px,
        right_px,
        top_px,
        bottom_px,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterPointSub219C {
    pub(crate) x_px: i16,
    pub(crate) y_px: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn kid_hitbox_point_loc_2252(
    x_anchor_px: i16,
    y_anchor_px: i16,
) -> CollisionRectSub1F52 {
    // ASM loc_2252 (kid.asm:4075): when `sub_21F8` receives a zero collision
    // word it writes Kid's current `$1A/$1E` anchors into all four hitbox
    // globals, yielding a one-point rectangle.
    CollisionRectSub1F52 {
        left_px: x_anchor_px,
        right_px: x_anchor_px,
        top_px: y_anchor_px,
        bottom_px: y_anchor_px,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidProjectileHitboxCandidateSub226A {
    pub(crate) x_anchor_px: i16,
    pub(crate) y_anchor_px: i16,
    pub(crate) mirrored: bool,
    pub(crate) collision_size: Option<CollisionSizeEntrySub21F8>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SupportOverlapCandidateSub9832 {
    pub(crate) x_anchor_px: i16,
    pub(crate) y_anchor_px: i16,
    pub(crate) mirrored: bool,
    pub(crate) active: bool,
    pub(crate) collision_sizes: Vec<CollisionSizeEntrySub21F8>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterProjectileOverlapSub226A {
    pub(crate) shooter_index: usize,
    pub(crate) projectile_index: usize,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterProjectileRecycleLoc22FE {
    pub(crate) projectile_collision_id: u16,
    pub(crate) predecessor_next_slot_addr: u16,
    pub(crate) recycled_slot_next_free_addr: u16,
    pub(crate) next_free_shooter_object_slot_addr: u16,
    pub(crate) next_scan_slot_addr: u16,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn shooter_projectile_recycle_loc_22FE(
    predecessor_slot_addr: u16,
    current_slot_addr: u16,
    current_slot_next_addr: u16,
    next_free_shooter_object_slot_addr: u16,
) -> ShooterProjectileRecycleLoc22FE {
    // ASM loc_22FE (kid.asm:4149-4154): mark the collided Kid projectile as
    // `colid_hurt`, splice the current shooter slot out of the active list by
    // copying its `$A(a0)` link into the predecessor's `$A(a1)`, push the
    // removed slot onto `(Addr_NextFreeShooterObjectSlot).w`, then resume the
    // scan from the predecessor slot via `move.w a1,a0`.
    ShooterProjectileRecycleLoc22FE {
        projectile_collision_id: 0x001C,
        predecessor_next_slot_addr: current_slot_next_addr,
        recycled_slot_next_free_addr: next_free_shooter_object_slot_addr,
        next_free_shooter_object_slot_addr: current_slot_addr,
        next_scan_slot_addr: predecessor_slot_addr,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn next_shooter_point_loc_226E(
    shooter_points: &[ShooterPointSub219C],
    cursor: usize,
) -> Option<(usize, ShooterPointSub219C)> {
    // ASM loc_226E (kid.asm:4089): read the next shooter-slot link word,
    // return immediately when it is zero, otherwise copy the slot pointer and
    // load that slot's `$2/$4` coordinates before entering the projectile scan.
    shooter_points
        .get(cursor)
        .copied()
        .map(|point| (cursor, point))
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn first_shooter_hit_sub_219C(
    kid_hitbox: CollisionRectSub1F52,
    shooter_points: &[ShooterPointSub219C],
) -> Option<usize> {
    // ASM sub_219C / loc_21B8 (kid.asm:3997-4019): walk the shooter-slot
    // linked list one slot at a time, skipping forward on every strict miss.
    // Edge-equal contacts are rejected because the four bounds checks branch
    // back to `loc_21B8` on `ble` / `bge` rather than accepting equality.
    // ASM loc_21F2 then restores registers and returns once the next-slot word
    // is zero, which maps to `None` here when no strict interior point exists.
    shooter_points.iter().position(|point| {
        point.x_px > kid_hitbox.left_px
            && point.x_px < kid_hitbox.right_px
            && point.y_px > kid_hitbox.top_px
            && point.y_px < kid_hitbox.bottom_px
    })
}

fn projectile_hitbox_sub_226a(
    projectile: KidProjectileHitboxCandidateSub226A,
) -> Option<CollisionRectSub1F52> {
    let collision_size = projectile.collision_size?;
    if collision_size.left_offset_px == 0 {
        return None;
    }

    let (left_px, right_px) = if projectile.mirrored {
        // ASM loc_22D2: negate the left offset for mirrored sprites, anchor
        // that edge first, then subtract width back toward the facing side.
        let right_px = projectile
            .x_anchor_px
            .saturating_sub(collision_size.left_offset_px);
        let left_px = right_px.saturating_sub(collision_size.width_px);
        (left_px, right_px)
    } else {
        // ASM loc_22A6: use the stored left offset directly, then add width.
        let left_px = projectile
            .x_anchor_px
            .saturating_add(collision_size.left_offset_px);
        let right_px = left_px.saturating_add(collision_size.width_px);
        (left_px, right_px)
    };
    let top_px = projectile
        .y_anchor_px
        .saturating_add(collision_size.top_offset_px);
    let bottom_px = top_px.saturating_add(collision_size.height_px);

    Some(CollisionRectSub1F52 {
        left_px,
        right_px,
        top_px,
        bottom_px,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn next_projectile_hitbox_loc_2286(
    projectiles: &[KidProjectileHitboxCandidateSub226A],
    cursor: usize,
) -> Option<(usize, CollisionRectSub1F52)> {
    // ASM loc_2286 (kid.asm:4098): reload the Kid-projectile gfx-object head
    // on each pass, skip null/deleted entries by branching back to `loc_226E`
    // or `loc_2286`, and stop only once a CollisionSize record yields a live
    // hitbox to compare against the current shooter point.
    projectiles
        .iter()
        .copied()
        .enumerate()
        .skip(cursor)
        .find_map(|(projectile_index, projectile)| {
            projectile_hitbox_sub_226a(projectile).map(|hitbox| (projectile_index, hitbox))
        })
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn first_shooter_projectile_overlap_sub_226A(
    shooter_points: &[ShooterPointSub219C],
    projectiles: &[KidProjectileHitboxCandidateSub226A],
) -> Option<ShooterProjectileOverlapSub226A> {
    // ASM sub_226A (kid.asm:4086): walk the shooter-slot list first, then for
    // each live slot scan Kid projectiles until the first overlap. The tests
    // below mirror loc_22A6/loc_22D2's inclusive-left/top and
    // exclusive-right/bottom comparisons.
    let mut shooter_cursor = 0;
    while let Some((shooter_index, shooter_point)) =
        next_shooter_point_loc_226E(shooter_points, shooter_cursor)
    {
        let mut projectile_cursor = 0;
        while let Some((projectile_index, hitbox)) =
            next_projectile_hitbox_loc_2286(projectiles, projectile_cursor)
        {
            if shooter_point.x_px >= hitbox.left_px
                && shooter_point.x_px < hitbox.right_px
                && shooter_point.y_px >= hitbox.top_px
                && shooter_point.y_px < hitbox.bottom_px
            {
                return Some(ShooterProjectileOverlapSub226A {
                    shooter_index,
                    projectile_index,
                });
            }
            projectile_cursor = projectile_index + 1;
        }
        shooter_cursor = shooter_index + 1;
    }

    None
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn juggernaut_support_overlap_sub_9832(
    x_px: i16,
    y_px: i16,
    candidates: &[SupportOverlapCandidateSub9832],
) -> bool {
    // ASM sub_9832 (kid.asm:11367): build Kid's support rectangle as
    // `x_pos-$28 .. x_pos+$28` and `y_pos-$1F .. y_pos-$0F`, then walk the
    // live object list from `$FFFFF86A`. Deleted objects (`$3D(a2)`) are
    // skipped, and each candidate scans successive 8-byte CollisionSize
    // entries until a zero first word terminates that object.
    let kid_left_px = x_px.saturating_sub(0x28);
    let kid_right_px = x_px.saturating_add(0x28);
    let kid_top_px = y_px.saturating_sub(0x1F);
    let kid_bottom_px = kid_top_px.saturating_add(0x10);

    for candidate in candidates {
        if !candidate.active {
            continue;
        }

        for collision_size in &candidate.collision_sizes {
            if collision_size.left_offset_px == 0 {
                break;
            }

            let (left_px, right_px) = if candidate.mirrored {
                let right_px = candidate
                    .x_anchor_px
                    .saturating_sub(collision_size.left_offset_px);
                let left_px = right_px.saturating_sub(collision_size.width_px);
                (left_px, right_px)
            } else {
                let left_px = candidate
                    .x_anchor_px
                    .saturating_add(collision_size.left_offset_px);
                let right_px = left_px.saturating_add(collision_size.width_px);
                (left_px, right_px)
            };
            let top_px = candidate
                .y_anchor_px
                .saturating_add(collision_size.top_offset_px);
            let bottom_px = top_px.saturating_add(collision_size.height_px);

            if left_px > kid_right_px
                || right_px < kid_left_px
                || top_px > kid_bottom_px
                || bottom_px < kid_top_px
            {
                continue;
            }

            return true;
        }
    }

    false
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SolidOverlapAxisSub1FA2 {
    Horizontal,
    Vertical,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SolidOverlapResolutionSub1FA2 {
    pub(crate) axis: SolidOverlapAxisSub1FA2,
    pub(crate) x_delta_px: i16,
    pub(crate) y_delta_px: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn first_solid_overlap_resolution_sub_1FA2(
    kid_hitbox: CollisionRectSub1F52,
    solid_tiles: &[CollisionRectSub1F52],
) -> Option<SolidOverlapResolutionSub1FA2> {
    // ASM sub_1FA2 (kid.asm:3784) scans the overlapped level cells, finds the
    // first ordinary solid tile region, then compares horizontal vs vertical
    // escape distances. This helper mirrors that core push-out choice and the
    // ROM tie-breaks (`horizontal` on axis ties, then `left` / `up` on equal
    // edge distances). TODO(fidelity): the special-tile hurt branch at
    // `loc_211C..loc_216A` is intentionally not modelled here yet.
    let solid = solid_tiles.iter().copied().find(|solid| {
        solid.left_px <= kid_hitbox.right_px
            && solid.right_px >= kid_hitbox.left_px
            && solid.top_px <= kid_hitbox.bottom_px
            && solid.bottom_px >= kid_hitbox.top_px
    })?;

    let push_down = solid
        .bottom_px
        .saturating_sub(kid_hitbox.top_px)
        .saturating_add(1);
    let push_up = kid_hitbox
        .bottom_px
        .saturating_sub(solid.top_px)
        .saturating_add(1);
    let vertical_span = push_down.min(push_up);

    let push_right = solid
        .right_px
        .saturating_sub(kid_hitbox.left_px)
        .saturating_add(1);
    let push_left = kid_hitbox
        .right_px
        .saturating_sub(solid.left_px)
        .saturating_add(1);
    let horizontal_span = push_right.min(push_left);

    if horizontal_span <= vertical_span {
        let x_delta_px = if push_left <= push_right {
            -push_left
        } else {
            push_right
        };
        Some(SolidOverlapResolutionSub1FA2 {
            axis: SolidOverlapAxisSub1FA2::Horizontal,
            x_delta_px,
            y_delta_px: 0,
        })
    } else {
        let y_delta_px = if push_up <= push_down {
            -push_up
        } else {
            push_down
        };
        Some(SolidOverlapResolutionSub1FA2 {
            axis: SolidOverlapAxisSub1FA2::Vertical,
            x_delta_px: 0,
            y_delta_px,
        })
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn support_overlap_detected_sub_1F52(
    kid_hitbox: CollisionRectSub1F52,
    overlap_candidates: &[CollisionRectSub1F52],
) -> bool {
    // ASM sub_1F52 (kid.asm:3740): walk the `$FFFFF20A` rectangle list in
    // 8-byte steps until a zero left-edge word terminates the scan. Treat the
    // four comparisons as inclusive signed bounds; any overlap marks Kid as
    // `colid_kidbelow`.
    for candidate in overlap_candidates {
        if candidate.left_px == 0 {
            return false;
        }
        if candidate.left_px > kid_hitbox.right_px
            || candidate.right_px < kid_hitbox.left_px
            || candidate.top_px > kid_hitbox.bottom_px
            || candidate.bottom_px < kid_hitbox.top_px
        {
            continue;
        }
        return true;
    }
    false
}

pub(crate) fn enemy_contact_bounds(enemy: &NativeObjectSnapshot) -> EnemyContactBounds {
    match enemy.kind {
        NativeObjectKind::Enemy { object_type: 0x0D } => dragon_walking_contact_bounds(enemy),
        NativeObjectKind::Enemy { object_type: 0x1C } => EnemyContactBounds {
            left_px: -MINI_HOPPING_SKULL_CONTACT_HALF_WIDTH,
            right_px: MINI_HOPPING_SKULL_CONTACT_HALF_WIDTH,
            top_px: -ENEMY_CONTACT_HALF_HEIGHT,
            bottom_px: ENEMY_CONTACT_HALF_HEIGHT,
        },
        _ => EnemyContactBounds {
            left_px: -ENEMY_CONTACT_HALF_WIDTH,
            right_px: ENEMY_CONTACT_HALF_WIDTH,
            top_px: -ENEMY_CONTACT_HALF_HEIGHT,
            bottom_px: ENEMY_CONTACT_HALF_HEIGHT,
        },
    }
}

fn dragon_walking_contact_bounds(enemy: &NativeObjectSnapshot) -> EnemyContactBounds {
    // CollisionSize_Index entries for the live DragonWalking frames:
    // - walk/breath (`unk_31630` / `unk_31658`): `dc.w -$10,$20,-$20,$20`
    // - hurt (`unk_31610`): `dc.w -$13,$21,-$2B,$29`
    // These are raw left/top offsets plus width/height, so the boxes can sit
    // entirely above the feet anchor (`y_pos`) on the landing frames.
    if enemy.damage_flash {
        let (left_px, right_px) = match enemy.facing {
            Some(Direction::Right) => (-14, 19),
            Some(Direction::Left) => (-19, 14),
            None => (-19, 14),
        };
        return EnemyContactBounds {
            left_px,
            right_px,
            top_px: -43,
            bottom_px: -2,
        };
    }

    EnemyContactBounds {
        left_px: -16,
        right_px: 16,
        top_px: -32,
        bottom_px: 0,
    }
}

fn enemy_contact_min_overlap_fp(enemy: &NativeObjectSnapshot) -> i64 {
    match enemy.kind {
        // GfxObjects_CollisionKid accepts edge contacts on DragonWalking's
        // real collision boxes; the previous 8px gate was an invented guard
        // that suppressed the frame-3399 stomp in the live TAS frontier.
        NativeObjectKind::Enemy { object_type: 0x0D } => 0,
        _ => ENEMY_CONTACT_MIN_OVERLAP_FP,
    }
}

/// Classify contact between player and enemy as stomp, damage, or no contact.
///
/// Stomp: player is falling (y_velocity > 0) and the player's feet are at or
/// above the enemy's top edge. Contacts lower on the sprite deal damage
/// instead of granting a bounce.
pub(crate) fn classify_enemy_contact_with_half_width(
    player: PlayerState,
    enemy: &NativeObjectSnapshot,
    pre_physics_y_velocity: i32,
    _pre_physics_y_pos: u32,
    player_contact_half_width: i64,
) -> EnemyContactResult {
    let player_contact_half_width = match enemy.kind {
        // ASM Character_CheckCollision / GfxObjects_CollisionKid accept the
        // late Level 1 landing graze against the left-facing walking dragon on
        // the frame-1684 death path. The native form box needs one extra pixel
        // of horizontal reach there once Kid has already landed; keep the
        // airborne stomp path unchanged.
        NativeObjectKind::Enemy { object_type: 0x0D }
            if player.on_ground && pre_physics_y_velocity <= 0 =>
        {
            player_contact_half_width + 1
        }
        _ => player_contact_half_width,
    };
    let px = player.x_pos as i64;
    let py = player.y_pos as i64;
    let p_left = px - (player_contact_half_width << 16);
    let p_right = px + (player_contact_half_width << 16);
    let p_top = py - (i64::from(PLAYER_HEIGHT) << 16);
    let p_bottom = py;

    let ex = enemy.x_pos as i64;
    let ey = enemy.y_pos as i64;
    let bounds = enemy_contact_bounds(enemy);
    let e_left = ex + (bounds.left_px << 16);
    let e_right = ex + (bounds.right_px << 16);
    let e_top = ey + (bounds.top_px << 16);
    let e_bottom = ey + (bounds.bottom_px << 16);

    // MiniHoppingSkull immunity while airborne and rising
    if matches!(enemy.kind, NativeObjectKind::Enemy { object_type: 0x1C })
        && enemy.on_ground == Some(false)
        && enemy.y_velocity.is_some_and(|vy| vy < 0)
    {
        return EnemyContactResult::None;
    }

    let kid_left = (p_left >> 16) as i32;
    let kid_right = (p_right >> 16) as i32;
    let kid_top = (p_top >> 16) as i32;
    let kid_bottom = (p_bottom >> 16) as i32;
    let object_left = (e_left >> 16) as i32;
    let object_right = (e_right >> 16) as i32;
    let object_top = (e_top >> 16) as i32;
    let object_bottom = (e_bottom >> 16) as i32;

    // GfxObjects_CollisionKid truncates world positions to word pixel bounds
    // before testing overlap, and the comparisons are inclusive. The live
    // frame-3399 stomp only clips on that integer edge-touch.
    if object_left > kid_right
        || object_right < kid_left
        || object_top > kid_bottom
        || object_bottom < kid_top
    {
        return EnemyContactResult::None;
    }

    let x_overlap_px = object_right.min(kid_right) - object_left.max(kid_left);
    if (i64::from(x_overlap_px) << 16) < enemy_contact_min_overlap_fp(enemy) {
        return EnemyContactResult::None;
    }

    let object_x_delta = enemy.x_velocity.unwrap_or(0) - player.x_velocity;
    let object_y_delta = enemy.y_velocity.unwrap_or(0) - pre_physics_y_velocity;
    let rel_x = object_x_delta >> 8;
    let rel_y = object_y_delta >> 8;

    // ASM loc_1C42 (kid.asm:3390): GfxObjects_CollisionKid subtracts Kid's
    // velocity from the object's velocity, scales the deltas with
    // `lsl.l #8; swap`, then dispatches through the four sign-specific
    // overlap comparisons below to choose the collision side.
    enum ContactSide {
        Above,
        Below,
        Left,
        Right,
    }

    let relation = if object_x_delta >= 0 {
        if object_y_delta >= 0 {
            let x_term = i64::from(object_right - kid_left) * i64::from(rel_y);
            let y_term = i64::from(object_bottom - kid_top) * i64::from(rel_x);
            if y_term > x_term {
                ContactSide::Right
            } else {
                ContactSide::Below
            }
        } else {
            // ASM loc_1C76 (kid.asm:3412): object X delta is non-negative
            // while Kid is falling faster in Y, so the ROM compares
            // `(object_right - kid_left) * rel_y` against
            // `(object_top - kid_bottom) * rel_x`.
            let x_term = i64::from(object_right - kid_left) * i64::from(rel_y);
            let y_term = i64::from(object_top - kid_bottom) * i64::from(rel_x);
            if y_term < x_term {
                ContactSide::Right
            } else {
                ContactSide::Above
            }
        }
    } else if object_y_delta >= 0 {
        // ASM loc_1C94 (kid.asm:3427): object X delta is negative
        // (object slower in X), while the Y delta stays non-negative after
        // `move.l $2A(a2),d7; sub.l $2A(a0),d7`; the negative-Y branch is skipped.
        let x_term = i64::from(object_left - kid_right) * i64::from(rel_y);
        let y_term = i64::from(object_bottom - kid_top) * i64::from(rel_x);
        if y_term < x_term {
            ContactSide::Left
        } else {
            ContactSide::Below
        }
    } else {
        // ASM loc_1CB8 (kid.asm:3446): both object deltas are negative, so
        // the ROM compares `(object_left - kid_right) * rel_y` with
        // `(object_top - kid_bottom) * rel_x`.
        let x_term = i64::from(object_left - kid_right) * i64::from(rel_y);
        let y_term = i64::from(object_top - kid_bottom) * i64::from(rel_x);
        if y_term > x_term {
            ContactSide::Left
        } else {
            ContactSide::Above
        }
    };

    if matches!(relation, ContactSide::Above) {
        return EnemyContactResult::Stomp;
    }

    EnemyContactResult::Damage
}

pub(crate) fn player_touches_enemy_with_half_width(
    player: PlayerState,
    enemy: &NativeObjectSnapshot,
    player_contact_half_width: i64,
) -> bool {
    !matches!(
        classify_enemy_contact_with_half_width(
            player,
            enemy,
            player.y_velocity,
            player.y_pos,
            player_contact_half_width,
        ),
        EnemyContactResult::None
    )
}

impl NativeRuntime {
    /// Check all frame hazards in priority order. Returns true if the frame was
    /// consumed by a hazard (damage, death, level transition) — caller should
    /// return immediately. Returns false if no hazard fired and gameplay continues.
    pub(crate) fn resolve_frame_hazards(
        &mut self,
        slice: &mut OpeningSliceState,
        _input: FrameInput,
        pre_physics_y_velocity: i32,
        pre_physics_y_pos: u32,
        frame_start_x_pos: u32,
        frame_start_y_pos: u32,
    ) -> bool {
        // Level timer: tick down and kill the player on expiry
        if self.level_timer.tick() {
            self.opening_slice = Some(*slice);
            self.progression.number_hitpoints = 0;
            self.begin_death_windup();
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::LevelTimerDeath;
            return true;
        }

        // ASM loc_9142 (kid.asm:10549): sub_902A downward floor probe path
        // — `cmp.w Level_height_pixels,d7; bge.w loc_9142`. When new foot
        // pixel y >= Level_height_pixels, `addq.w #4,sp` pops the sub_902A
        // return address and `moveq #1,d7; bra.w Death` kills the player.
        // Native: fell_below_level uses `>` instead of `>=`; the difference
        // is academic since Level_height_pixels is the row below the last
        // tile row and landing exactly on it is unreachable in practice.
        if self.fell_below_level(slice) {
            self.opening_slice = Some(*slice);
            self.progression.number_hitpoints = 0;
            self.begin_death_windup();
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::PitDeath;
            return true;
        }

        // Spike death: contacting a spike cell is instant death
        if slice.player.on_spikes {
            self.opening_slice = Some(*slice);
            self.progression.number_hitpoints = 0;
            self.begin_death_windup();
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::SpikeDeath;
            return true;
        }

        // Projectiles: advance and damage player on hit.
        // ASM Character_CheckCollision checks invulnerability timer before all
        // non-spike damage, including projectiles.
        self.step_projectiles();
        if self.progression.invulnerable_frames == 0 && self.projectile_hits_player(slice) {
            self.opening_slice = Some(*slice);
            self.apply_damage();
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::ProjectileDamage;
            return true;
        }

        // Murder wall: advance and damage player on contact.
        // ASM loc_7396/loc_73B8: reversed wall can push camera_x.
        self.step_murder_wall(&mut slice.camera.x_pos);
        if self.murder_wall_kills_player(slice) && self.progression.invulnerable_frames == 0 {
            self.apply_damage();
            self.opening_slice = Some(*slice);
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::MurderWallDamage;
            return true;
        }

        // Teleporter: stepping on a teleporter block warps the player.
        // ASM loc_BCB2 (kid.asm:15245): destination extraction — `addq.w #6,a4`
        // skips to offset +6 of the matched 10-byte entry. Word at +6:
        // high byte = dest_map_id (`asr.w #8,d7`), low byte = dest_y
        // (`andi.w #$FF,d7; asl.w #4,d7` → PlayerStart_Y_pos). Word at +8:
        // dest_x (`asl.w #4,d7` → PlayerStart_X_pos). Both positions are in
        // 16-pixel blocks (asl #4 = ×16).
        // ASM loc_BCD6 (kid.asm:15259): MapOrder_Index scan loop — `move.b
        // (a4)+,d5; ext.w d5; cmpi.w #$FFFF,d5; beq loc_BCAC` (end = death);
        // `cmp.w d5,d7; beq loc_BCEA` (found target level). d6 counts the
        // level index from 0. Native stores dest_map_id directly as a level
        // ID, so the MapOrder_Index scan is implicit.
        // ASM loc_BCEA (kid.asm:15270): level transition — `cmp.w
        // (Current_LevelID).w,d6; beq loc_BCF6` (same level skips flag set);
        // different level: `st ($FFFFFC36)` (level-change flag). Then
        // `move.w d6,(Current_LevelID)`, `st ($FFFFFC29)`,
        // `move.w #$EE,($FFFFFBCC)` (yellow teleport warp colour),
        // `moveq #-5,d6; bra Teleport` (d6=-5 triggers sfx_Teleport sound).
        if let Some((dest_map, dest_x, dest_y)) = self.check_teleporter_contact(slice) {
            self.opening_slice = Some(*slice);
            // Same-level teleport: reposition. Cross-level: trigger level skip.
            if dest_map == self.current_level.0 {
                if let Some(ref mut s) = self.opening_slice {
                    s.player.x_pos = u32::from(dest_x) << 20; // blocks to FP16.16
                    s.player.y_pos = u32::from(dest_y as u16) << 20;
                }
            } else {
                self.pending_level_skip = Some(LevelId(dest_map));
            }
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::Teleporter;
            return true;
        }

        // Flag collision: touching the flag completes the level
        if self.player_touches_flag(slice) {
            self.apply_level_completion_bonuses();
            self.opening_slice = Some(*slice);
            self.player_lifecycle = PlayerLifecycle::LevelComplete {
                remaining_frames: LEVEL_COMPLETE_PAUSE_FRAMES,
            };
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::FlagTouch;
            return true;
        }

        // Stomp check: kill enemies by landing on them from above
        if self.process_stomps(
            slice,
            pre_physics_y_velocity,
            pre_physics_y_pos,
            frame_start_x_pos,
            frame_start_y_pos,
        ) {
            self.opening_slice = Some(*slice);
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::StompBounce;
            return true;
        }

        // Combat abilities: sword/axe/beam/charge damage enemies
        self.process_sword_hits(slice);
        self.process_friendly_projectile_hits();
        self.process_berzerker_charge_hits(slice);

        // Diamond collection: touching a Diamond picks it up
        self.collect_diamonds(slice);

        // Cyclone-Tornado bounce: ASM Tornado.asm loc_3597E/loc_35984 -> loc_35986.
        // When Cyclone-flying player contacts Tornado, bounce instead of damage.
        if !self.stomped_this_frame
            && self.progression.current_form == PlayerForm::Cyclone
            && self.helmet.cyclone_flying
        {
            for enemy in &mut self.active_enemies {
                if enemy.family != EnemyFamily::Tornado {
                    continue;
                }
                if !enemy.is_vulnerable() {
                    continue;
                }
                let snapshot = enemy.snapshot();
                if !player_touches_enemy_with_half_width(
                    slice.player,
                    &snapshot,
                    self.progression.current_form.half_width() as i64,
                ) {
                    continue;
                }
                let player_x = slice.player.x_pos as i64;
                let tornado_x = enemy.x_pos as i64;
                let bounce = tornado_cyclone_bounce_for_positions_loc_35986(player_x, tornado_x);
                slice.player.x_velocity = bounce.player_x_velocity;
                enemy.x_velocity = bounce.tornado_x_velocity;
                // TODO(fidelity): ASM writes `$28` to Kid `collision_type(a4)`.
                // Native has no player collision_type field here, so preserve
                // the existing Cyclone accumulator surrogate until that boundary
                // is modelled directly.
                self.helmet.cyclone_y_accel = bounce.player_collision_type as i16;
                self.opening_slice = Some(*slice);
                self.last_kid_frame_post_motion_mutator =
                    KidFramePostMotionMutator::CycloneTornadoBounce;
                return true;
            }
        }

        // Contact damage with knockback
        if !self.stomped_this_frame
            && let Some((enemy_x, enemy_family)) = self.find_contact_damage_source(slice)
        {
            if matches!(enemy_family, EnemyFamily::DragonWalking)
                && slice.player.on_ground
                && pre_physics_y_velocity > 0
            {
                return false;
            }
            if matches!(enemy_family, EnemyFamily::MiniHoppingSkull) {
                // ASM Mini_Hopping_Skull.asm loc_3EB40 allocates sub_3F3F0:
                // side/below contact hibernates for 15 frames, then plays the
                // scream SFX unless `$19(a3)` suppresses it.
                self.queue_mini_hopping_skull_scream(false);
            }
            self.apply_damage_with_knockback(slice, enemy_x);
            self.opening_slice = Some(*slice);
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::ContactDamage;
            return true;
        }

        false
    }
}
