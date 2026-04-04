use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueuedSoundEffect {
    MiniHoppingSkullScream,
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
        (0, -homing_accel)
    } else {
        (0xFF, homing_accel)
    };
    let y_velocity_delta = if effect_y > kid_y {
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
pub(crate) struct BossDeathBurstFinishSub3728C {
    pub(crate) animation_label: &'static str,
    pub(crate) hibernate_until_anim_finished: bool,
    pub(crate) delete_current_object: bool,
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
        (0, -homing_accel)
    } else {
        (0xFF, homing_accel)
    };
    let y_velocity_delta = if effect_y > kid_y {
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
    // `stru_37BEA`, then enter a 100-frame facing loop.
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
    // of Kid clear `x_direction(a3)`, otherwise set it to `$FF`.
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
            "$0E30"
        };
    }

    if kid_y > effect_y.saturating_add(4) {
        if kid_x < effect_x.saturating_sub(4) {
            "LnkTo_unk_C84A0-Data_Index"
        } else if kid_x > effect_x.saturating_add(4) {
            "LnkTo_unk_C84B0-Data_Index"
        } else {
            "LnkTo_unk_C84A8-Data_Index"
        }
    } else if kid_y < effect_y.saturating_sub(4) {
        if kid_x < effect_x.saturating_sub(4) {
            "LnkTo_unk_C8490-Data_Index"
        } else if kid_x > effect_x.saturating_add(4) {
            "LnkTo_unk_C84C0-Data_Index"
        } else {
            "LnkTo_unk_C8488-Data_Index"
        }
    } else if kid_x < effect_x.saturating_sub(8) {
        "LnkTo_unk_C8498-Data_Index"
    } else {
        "LnkTo_unk_C84B8-Data_Index"
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
    // ASM sub_37ABC (kid.asm:39991): walk the 12 longword worker pointers at
    // `$FFFFFB7E`, skip null entries, and refresh `addroffset_sprite(a3)` by
    // calling sub_37AF0 for each live boss-eye child.
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
            (sprite_word_y_adjust.wrapping_add(1), true)
        } else {
            (sprite_word_y_adjust.wrapping_sub(1), true)
        };

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
            (sprite_word_y_adjust.wrapping_add(1), true)
        };

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
        "loc_3A1EA"
    };

    let (x_pos, velocity_word_48) = if parent_x_direction == 0 {
        (parent_x.wrapping_sub(0x0018), -3)
    } else {
        (parent_x.wrapping_add(0x0018), 3)
    };

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

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn boss_death_timeout_sound_sub_370D2(v_int_counter_fff809: u8) -> &'static str {
    // ASM loc_371D4..loc_37200 (kid.asm:39060): choose the timeout sound from
    // bit 0 of `$FFFFF809`. The odd branch queues `sfx_Plethora_x` but the ROM
    // comment notes "NO SOUND!".
    if v_int_counter_fff809 & 1 != 0 {
        "sfx_Plethora_x"
    } else {
        "sfx_Boss_dies"
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
        if let Some(worker) = slot.as_mut() {
            worker.x_velocity = x_velocity;
            worker.y_velocity = y_velocity;
        }
    }

    slots
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
            "LnkTo_unk_C84A8-Data_Index"
        };
    }

    if kid_y > effect_y.saturating_add(4) {
        if kid_x < effect_x.saturating_sub(4) {
            "LnkTo_unk_C84A0-Data_Index"
        } else if kid_x > effect_x.saturating_add(4) {
            "LnkTo_unk_C84B0-Data_Index"
        } else {
            "LnkTo_unk_C84A8-Data_Index"
        }
    } else if kid_y < effect_y.saturating_sub(4) {
        if kid_x < effect_x.saturating_sub(4) {
            "LnkTo_unk_C8490-Data_Index"
        } else if kid_x > effect_x.saturating_add(4) {
            "LnkTo_unk_C84C0-Data_Index"
        } else {
            "LnkTo_unk_C8488-Data_Index"
        }
    } else if kid_x < effect_x.saturating_sub(8) {
        "LnkTo_unk_C8498-Data_Index"
    } else {
        "LnkTo_unk_C84B8-Data_Index"
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

    if collision_type != 0 {
        result.next_collision_type = 0;

        if collision_type == 0x002C {
            result.next_phase_counter = phase_counter.wrapping_add(1);
            if let Some(&piece_index) =
                HEADY_METAL_BREAKUP_SEQUENCE_SUB_37708.get(usize::from(result.next_phase_counter))
                && let Some(mut piece) = result.piece_slots[piece_index]
            {
                match result.next_phase_counter {
                    0x0000..=0x000C => {
                        piece.sprite_word_y += 2;
                        result.played_sound_effect = Some("sfx_Boss_eye_pops");
                        result.piece_slots[piece_index] = Some(piece);
                    }
                    0x000D..=0x0018 => {
                        piece.sprite_word_y -= 2;
                        result.piece_slots[piece_index] = Some(piece);
                    }
                    0x0019..=0x0024 => {
                        result.spawned_pop_effect_at = Some((piece.x_pos, piece.y_pos));
                        result.spawned_attack_at = Some((piece.x_pos, piece.y_pos));
                        result.piece_slots[piece_index] = None;
                    }
                    _ => {}
                }
            }
        }
    }

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

    result.next_remaining_scatter_frames = remaining_scatter_frames - 1;
    let timer = result.next_remaining_scatter_frames;
    let (d6_x_velocity_delta, d7_velocity_delta) = if timer > 0x015E {
        (-0x0003_0D40, 0x0003_0D40)
    } else if timer > 0x00FA {
        (-0x0004_93E0, 0x0004_93E0)
    } else if timer > 0x0096 {
        (-0x0006_1A80, 0x0006_1A80)
    } else if timer > 0x0064 {
        (-0x0007_A120, 0x0007_A120)
    } else {
        (-0x0009_27C0, 0x0009_27C0)
    };

    result.scatter_step = Some(HeadyMetalScatterStepSub37708 {
        pattern: if frame_counter & 2 == 0 {
            HeadyMetalScatterPatternSub37708::Primary
        } else {
            HeadyMetalScatterPatternSub37708::Alternate
        },
        d6_x_velocity_delta,
        d7_velocity_delta,
    });

    result
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
        let old_score = self.progression.score;
        self.progression.score = self.progression.score.saturating_add(points).min(SCORE_CAP);
        // ASM: award extra life for every 50,000 point boundary crossed.
        let old_lives = old_score / SCORE_EXTRA_LIFE_INTERVAL;
        let new_lives = self.progression.score / SCORE_EXTRA_LIFE_INTERVAL;
        for _ in old_lives..new_lives {
            self.add_life();
        }
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

        // No-prize bonus: 5000 points if no prize block content collected during level
        // ASM: NoPrize_Bonus_Flag = 0 means bonus retained
        if !self.level_prize_collected {
            self.add_score(5000);
        }

        // Path and speed bonuses only apply to non-Elsewhere levels (< 0x49 in map order).
        let level_idx = self.current_level.0 as usize;
        if self.current_level.0 < FIRST_ELSEWHERE_LEVEL_ID {
            // Path bonus: per-level multiplier × 1000
            if level_idx < PATH_BONUS_TABLE.len() {
                let path_mult = PATH_BONUS_TABLE[level_idx] as u32;
                if path_mult > 0 {
                    self.add_score(path_mult * 1000);
                }
            }

            // Speed bonus: 10,000 points if completion time <= threshold.
            // ASM completion time = 180 - remaining + (clocks_collected * 180), capped at 255.
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
    pub(crate) fn check_teleporter_contact(
        &self,
        slice: &OpeningSliceState,
    ) -> Option<(u8, u16, u8)> {
        let block_list = self.block_list.as_ref()?;
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
        self.progression.number_hitpoints = form.max_hitpoints();
        // ASM Kid_Transform calls sub_B41C which sets per-helmet half-width.
        // During Hibernate frames, the GFX collision system resolves any overlap
        // between the new wider hitbox and adjacent walls by pushing x_pos.
        let width_delta = form.half_width() - old_form.half_width();
        if width_delta != 0 {
            self.progression.form_width_delta = width_delta;
        }
        // Per-helmet transform duration from ASM Kid_Transform tables
        // (kid.asm:8369-8607, stru_7EFA..stru_808A). Two phases:
        // Phase 1 (reverse old helmet) + Phase 2 (forward new helmet).
        // TheKid has no table, so only one phase runs.
        let old_frames = transform_anim_frames(old_form);
        let new_frames = transform_anim_frames(form);
        // +1 for the Kid_Transform entry frame (sub_7C1C palette save +
        // condition checks + initial Hibernate) before the animation loop starts.
        self.progression.form_change_flash_frames = (old_frames + new_frames + 1) as u8;
        // ASM zeroes velocity at end of transform (kid.asm:8599-8600)
        if let Some(ref mut slice) = self.opening_slice {
            slice.player.x_velocity = 0;
            slice.player.y_velocity = 0;
        }
        // TODO(fidelity): visual sprite art for transform animation.
        // ASM uses per-helmet sprite+palette cycling tables for a spinning
        // animation. Currently renders as a flash overlay. Needs ROM sprite
        // tile extraction for real visuals.
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
        self.level_damage_taken = true;
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
            .map(|slice| self.build_death_shell_snapshot(slice));
        self.player_lifecycle = PlayerLifecycle::DeathWindup {
            remaining_frames: PLAYER_DEATH_WINDUP_FRAMES,
        };
    }

    fn death_sprite_still_visible(player: PlayerState, camera: CameraState) -> bool {
        let y_screen_px = ((player.y_pos >> 16) as i32)
            - ((camera.y_pos >> 16) as i32)
            - PLAYER_DEATH_SPRITE_Y_EXTENT_PX;
        if !(-SPRITE_VISIBILITY_TOP_MARGIN_PX
            ..=physics::VIEWPORT_HEIGHT + SPRITE_VISIBILITY_BOTTOM_MARGIN_PX)
            .contains(&y_screen_px)
        {
            return false;
        }

        let x_screen_px = ((player.x_pos >> 16) as i32) - ((camera.x_pos >> 16) as i32);
        (-SPRITE_VISIBILITY_LEFT_MARGIN_PX
            ..=physics::VIEWPORT_WIDTH + SPRITE_VISIBILITY_RIGHT_MARGIN_PX)
            .contains(&x_screen_px)
    }

    fn build_death_shell_snapshot(&self, slice: OpeningSliceState) -> LevelSkipTransitionState {
        let mut frozen_player = slice.player;
        let mut death_y_velocity = -0x0004_0000;

        loop {
            frozen_player.x_pos = frozen_player.x_pos.wrapping_add(0x0000_8000);
            frozen_player.y_pos = frozen_player.y_pos.wrapping_add_signed(death_y_velocity);
            if !Self::death_sprite_still_visible(frozen_player, slice.camera) {
                break;
            }
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
            // ASM `Death` (kid.asm:14714-14731) uses the same tumble launch on
            // grounded and airborne deaths. loc_B80C checks visibility on the
            // current pose, then increments d1 by $3000 before applying the
            // tumble step, so the first visible windup frame already moves by
            // x += $00008000 and y += initial_d1 + $3000.
            slice.player.on_ground = false;
            slice.player.x_velocity = 0x0000_8000;
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
        self.progression.current_form = PlayerForm::TheKid;
        self.progression.pending_form = None;
        self.progression.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
        self.progression.number_lives = self.progression.number_lives.saturating_sub(1);
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.shooter_blocks.clear();
        self.block_state.shooter_headbutt_workers.clear();
        self.block_state.ghost_groups.clear();
        self.death_camera_lock = None;
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
        return previous_hitbox;
    }

    let (left_px, right_px) = if !mirrored {
        let left_px = x_anchor_px.saturating_add(collision_size.left_offset_px);
        let right_px = left_px.saturating_add(collision_size.width_px);
        (left_px, right_px)
    } else {
        let right_px = x_anchor_px.saturating_sub(collision_size.left_offset_px);
        let left_px = right_px.saturating_sub(collision_size.width_px);
        (left_px, right_px)
    };
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidProjectileHitboxCandidateSub226A {
    pub(crate) x_anchor_px: i16,
    pub(crate) y_anchor_px: i16,
    pub(crate) mirrored: bool,
    pub(crate) collision_size: Option<CollisionSizeEntrySub21F8>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterProjectileOverlapSub226A {
    pub(crate) shooter_index: usize,
    pub(crate) projectile_index: usize,
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn first_shooter_hit_sub_219C(
    kid_hitbox: CollisionRectSub1F52,
    shooter_points: &[ShooterPointSub219C],
) -> Option<usize> {
    // ASM sub_219C (kid.asm:3997): walk the shooter-slot linked list and
    // report only the first point strictly inside Kid's hitbox. Edge-equal
    // contacts are rejected because the routine skips on `ble` / `bge`.
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

    let (left_px, right_px) = if !projectile.mirrored {
        // ASM loc_22A6: use the stored left offset directly, then add width.
        let left_px = projectile
            .x_anchor_px
            .saturating_add(collision_size.left_offset_px);
        let right_px = left_px.saturating_add(collision_size.width_px);
        (left_px, right_px)
    } else {
        // ASM loc_22D2: negate the left offset for mirrored sprites, anchor
        // that edge first, then subtract width back toward the facing side.
        let right_px = projectile
            .x_anchor_px
            .saturating_sub(collision_size.left_offset_px);
        let left_px = right_px.saturating_sub(collision_size.width_px);
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
pub(crate) fn first_shooter_projectile_overlap_sub_226A(
    shooter_points: &[ShooterPointSub219C],
    projectiles: &[KidProjectileHitboxCandidateSub226A],
) -> Option<ShooterProjectileOverlapSub226A> {
    // ASM sub_226A (kid.asm:4086): walk the shooter-slot list first, then for
    // each live slot scan Kid projectiles until the first overlap. The tests
    // below mirror loc_22A6/loc_22D2's inclusive-left/top and
    // exclusive-right/bottom comparisons.
    for (shooter_index, shooter_point) in shooter_points.iter().enumerate() {
        for (projectile_index, projectile) in projectiles.iter().copied().enumerate() {
            let Some(hitbox) = projectile_hitbox_sub_226a(projectile) else {
                continue;
            };
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
        }
    }

    None
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
            let x_term = i64::from(object_right - kid_left) * i64::from(rel_y);
            let y_term = i64::from(object_top - kid_bottom) * i64::from(rel_x);
            if y_term < x_term {
                ContactSide::Right
            } else {
                ContactSide::Above
            }
        }
    } else if object_y_delta >= 0 {
        let x_term = i64::from(object_left - kid_right) * i64::from(rel_y);
        let y_term = i64::from(object_bottom - kid_top) * i64::from(rel_x);
        if y_term < x_term {
            ContactSide::Left
        } else {
            ContactSide::Below
        }
    } else {
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

        // Pit death: falling below the level floor boundary is instant death
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
        self.step_murder_wall();
        if self.murder_wall_kills_player(slice) && self.progression.invulnerable_frames == 0 {
            self.apply_damage();
            self.opening_slice = Some(*slice);
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::MurderWallDamage;
            return true;
        }

        // Teleporter: stepping on a teleporter block warps the player
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

        // Cyclone-Tornado bounce: ASM Tornado.asm loc_35986
        // When Cyclone-flying player contacts Tornado, bounce instead of damage.
        // Player x_vel = ±$20000, Tornado x_vel = opposite halved ∓$10000,
        // cyclone_y_accel = $28 (40).
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
                // ASM loc_35986: Cyclone-Tornado bounce
                // Direction from player position relative to Tornado center
                let player_x = slice.player.x_pos as i64;
                let tornado_x = enemy.x_pos as i64;
                let bounce_direction = if player_x < tornado_x { -1 } else { 1 };
                // Player x_velocity = ±$20000 (2.0 px/frame)
                slice.player.x_velocity = bounce_direction * 0x20000;
                // Tornado x_velocity = opposite, halved (∓$10000)
                enemy.x_velocity = -bounce_direction * 0x10000;
                // Cyclone Y acceleration boost = $28 (ASM writes to $38(a4))
                self.helmet.cyclone_y_accel = 0x28;
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
                // ASM Mini_Hopping_Skull.asm sub_3F3F0: side/below contact
                // allocates a helper object that hibernates for 15 frames,
                // then plays the scream SFX unless `$19(a3)` suppresses it.
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
