use super::*;

/// Emerge animation duration (`stru_3272E`: seven four-frame entries).
pub(crate) const TAR_MONSTER_EMERGE_FRAMES: u16 = 28;
/// Hurt animation duration (`stru_326EE`: two three-frame entries).
pub(crate) const TAR_MONSTER_HURT_FRAMES: u8 = 6;
/// Attack sequence duration. ASM loc_32310/loc_3231C/loc_32348/loc_32360:
/// three `moveq #8,d0` + `dbf` loops, so each phase lasts 9 frames.
pub(crate) const TAR_MONSTER_ATTACK_FRAMES: u16 = 27;
/// Zero-based elapsed frame index for the glob shot. ASM loc_3231C fires after
/// the first 9-frame `dbf` loop, which is elapsed frame 8 in the native countdown.
pub(crate) const TAR_MONSTER_ATTACK_FIRE_FRAME: u16 = 8;
/// Submerge animation duration (`stru_3274C`: seven four-frame entries).
pub(crate) const TAR_MONSTER_SUBMERGE_FRAMES: u16 = 28;
/// `status_flag_13` on the primary GfxObject at `$3A(a5)`.
/// Tar Monster has no layout script variant, so its private byte stores this
/// one-bit visibility latch without widening every `ActiveEnemy`.
pub(crate) const TAR_MONSTER_PRIMARY_VISIBLE_LOC_32308: u8 = 1;

pub(crate) fn tar_monster_live_sprite_phase_stru_3272e_3274c(
    state: TarMonsterState,
    wait_timer: u16,
) -> u8 {
    // ASM stru_3272E/stru_3274C contain seven four-frame mappings. The
    // Object coroutine hibernates while the linked GfxObject walks the table;
    // native's remaining wait reproduces that independent animation cursor.
    let elapsed =
        TAR_MONSTER_EMERGE_FRAMES.saturating_sub(wait_timer.min(TAR_MONSTER_EMERGE_FRAMES));
    let transition_phase = (elapsed / 4).min(6) as u8;
    match state {
        TarMonsterState::Submerged => 0,
        TarMonsterState::Emerging => transition_phase,
        TarMonsterState::Visible => 6,
        TarMonsterState::Attacking => {
            // ASM loc_32310/loc_3231C/loc_32348/loc_32360 holds C77DE for
            // nine hibernates, C77E6 for nine, then C77DE for nine before
            // restoring C77D6. These are fixed mappings, not stru_3272E
            // animation entries.
            let elapsed =
                TAR_MONSTER_ATTACK_FRAMES.saturating_sub(wait_timer.min(TAR_MONSTER_ATTACK_FRAMES));
            if (9..18).contains(&elapsed) { 14 } else { 13 }
        }
        TarMonsterState::Submerging => 6 - transition_phase,
    }
}
/// Horizontal glob shot speed: ±4px/frame (Tar_Monster.asm line 316: $FFFC = -4).
pub(crate) const TAR_MONSTER_GLOB_SPEED_X: i32 = 4 << 16;
/// ASM loc_32534: final three child velocities in the Tar Monster burst sequence.
pub(crate) const TAR_MONSTER_BURST_TAIL_VELOCITIES_LOC_32534: [(i32, i32); 3] = [
    (0x0_8000, -0x2_0000),
    (-0x0_8000, -0x2_0000),
    (0x0_2000, -0x1_C000),
];
/// ASM loc_324F8 burst child velocities after the straight glob impact/expiry.
pub(crate) const TAR_MONSTER_BURST_BLOB_VELOCITIES_LOC_324F8: [(i32, i32); 5] = [
    (0x1_0000, -0x1_0000),
    (-0x1_0000, -0x1_0000),
    TAR_MONSTER_BURST_TAIL_VELOCITIES_LOC_32534[0],
    TAR_MONSTER_BURST_TAIL_VELOCITIES_LOC_32534[1],
    TAR_MONSTER_BURST_TAIL_VELOCITIES_LOC_32534[2],
];
/// Raw control script bytes used by `unk_32264` in `loc_32248`.
/// Bytecode pairs are read with `move.w (a0)+,d0`, so each table entry is a signed
/// 16-bit step command in the native script interpreter.
pub(crate) const TAR_MONSTER_CONTROL_BYTES_LOC_32248: [u8; 44] = [
    0x00, 0x04, 0x00, 0x10, 0x5A, 0x00, 0x00, 0x10, 0x05, 0x00, 0x00, 0x10, 0x0A, 0x00, 0x00, 0x10,
    0x07, 0x00, 0x00, 0x10, 0x32, 0x00, 0x00, 0x14, 0x00, 0x10, 0x1E, 0x00, 0x00, 0x08, 0x46, 0x00,
    0x00, 0x0C, 0x00, 0x04, 0x14, 0x00, 0x00, 0x08, 0x46, 0x00, 0xFF, 0xFF,
];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterScriptCommandLoc32256 {
    Wait { dbf_seed: u8 },
    Appear,
    Disappear,
    ToggleDirection,
    TogglePrimary,
    Attack,
    Restart,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterScriptStepLoc32256 {
    pub(crate) command: TarMonsterScriptCommandLoc32256,
    pub(crate) next_offset: u8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterLoc32290Target {
    /// `off_32290[0]`: jump to `loc_322A8`.
    Loc322A8,
    /// `off_32290[1]`: jump to `loc_322BC`.
    Loc322BC,
    /// `off_32290[2]`: jump to `loc_322DA`.
    Loc322DA,
    /// `off_32290[3]`: jump to `loc_322FC`.
    Loc322FC,
    /// `off_32290[4]`: jump to `loc_32308`.
    Loc32308,
    /// `off_32290[5]`: jump to `loc_32310`.
    Loc32310,
    /// Byte index outside `off_32290` (out-of-range dispatch).
    Invalid(u8),
}

/// ASM `off_32290` jump table for `loc_32256`/`loc_3231C` dispatch.
pub(crate) const TAR_MONSTER_LOC_32290_TABLE: [TarMonsterLoc32290Target; 6] = [
    TarMonsterLoc32290Target::Loc322A8,
    TarMonsterLoc32290Target::Loc322BC,
    TarMonsterLoc32290Target::Loc322DA,
    TarMonsterLoc32290Target::Loc322FC,
    TarMonsterLoc32290Target::Loc32308,
    TarMonsterLoc32290Target::Loc32310,
];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterLoc32256Route {
    Restart,
    Dispatch(TarMonsterLoc32290Target),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterLoc3224EReturn {
    pub(crate) collision_type: u16,
    pub(crate) jump_table: &'static [TarMonsterLoc32290Target; 6],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterLoc32256Return {
    pub(crate) raw_command: i16,
    pub(crate) route: TarMonsterLoc32256Route,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterWaitLoopLoc322AA {
    pub(crate) hibernate_frames: u32,
    pub(crate) checks_object_range_each_frame: bool,
    pub(crate) polls_collision_each_frame: bool,
    pub(crate) next_collision_type: u16,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterAppearSoundLoc322C0 {
    pub(crate) sound_effect_label: &'static str,
    pub(crate) preserves_script_command_d0: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterAppearAnimationLoc322C8 {
    pub(crate) animation_label: &'static str,
    pub(crate) hibernate_until_animation_finished: bool,
    pub(crate) animation_frames: u16,
    pub(crate) resumes_control_script: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterThirdAttackWaitLoc3235E {
    pub(crate) dbf_seed: u16,
    pub(crate) hibernate_frames: u16,
    pub(crate) checks_object_range_each_frame: bool,
    pub(crate) polls_collision_each_frame: bool,
    pub(crate) restored_sprite_link: &'static str,
    pub(crate) resumes_control_script: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterCollisionGateLoc3237A {
    Kill,
    Return,
    NonZero { captured_collision_type: u16 },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterCollisionRouteLoc32386 {
    ReturnOnly,
    Hurt,
    BerzerkerKill,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterBerzerkerCollisionLoc323AA {
    pub(crate) route: TarMonsterCollisionRouteLoc32386,
    pub(crate) clears_kid_x_velocity: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterHurtRouteLoc323C0 {
    Death,
    HurtAnimation,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterHurtCollisionLoc323C0 {
    pub(crate) saved_child_visible: bool,
    pub(crate) child_visible_during_response: bool,
    pub(crate) remaining_hitpoints: u16,
    pub(crate) route: TarMonsterHurtRouteLoc323C0,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterHurtAnimationLoc323D6 {
    pub(crate) animation_label: &'static str,
    pub(crate) hibernate_until_animation_finished: bool,
    pub(crate) restored_child_visible: bool,
    pub(crate) resumes_control_script: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterRespawnMarkerLoc3242C {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterDeathStatusRouteLoc3243A {
    WriteRespawnMarker(TarMonsterRespawnMarkerLoc3242C),
    SubtractSpawnCounter { index: usize, decrement: u16 },
    DeleteOnly,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterDeathStatusUpdateLoc3243A {
    pub(crate) route: TarMonsterDeathStatusRouteLoc3243A,
    pub(crate) deletes_current_object: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterShotSoundLoc32468 {
    pub(crate) sound_effect_label: &'static str,
    pub(crate) preserves_script_command_d0: bool,
    pub(crate) gfx_object_slot_word: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterShotSpawnLoc3247E {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) object_meta: u16,
    pub(crate) status_flag_13: bool,
    pub(crate) is_moved: bool,
    pub(crate) priority: u8,
    pub(crate) has_level_collision: bool,
    pub(crate) sprite_frame_link: &'static str,
    pub(crate) copied_x_direction: Direction,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterShotSpawnRouteLoc3247E {
    StraightShot(TarMonsterShotSpawnLoc3247E),
    RetaliationBurst {
        x_pos: u32,
        y_pos: u32,
        proceeds_to_burst_cleanup: bool,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterRetaliationEntryLoc325AC {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) proceeds_to_burst_cleanup: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterSeededShotLoc325B6 {
    pub(crate) gfx_object_slot_word: u32,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_velocity: i32,
    pub(crate) sprite_frame_link: &'static str,
    pub(crate) object_meta: u16,
    pub(crate) status_flag_13: bool,
    pub(crate) is_moved: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) priority: u8,
    pub(crate) has_level_collision: bool,
    pub(crate) collision_sprite_swap_enabled: bool,
    pub(crate) collision_response_budget: u16,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterBurstChildEntryLoc325E6 {
    pub(crate) gfx_object_slot_word: u32,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) sprite_frame_link: &'static str,
    pub(crate) collision_sprite_swap_enabled: bool,
    pub(crate) falls_through_to_meta_seed: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterBurstChildMetaLoc32618 {
    pub(crate) object_meta: u16,
    pub(crate) proceeds_to_projectile_setup: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterProjectileSetupLoc3261C {
    pub(crate) object_meta: u16,
    pub(crate) status_flag_13: bool,
    pub(crate) is_moved: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) priority: u8,
    pub(crate) has_level_collision: bool,
    pub(crate) collision_response_budget: u16,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterProjectileDispatchSeedLoc32698 {
    pub(crate) remaining_collision_budget: u16,
    pub(crate) collision_offset: u16,
    pub(crate) clears_collision_type: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterProjectileCollisionBudgetLoc32698 {
    DeleteCurrentObject,
    DispatchSeed(TarMonsterProjectileDispatchSeedLoc32698),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterProjectileCollisionTargetLoc32682 {
    Loc326AA,
    Loc326B8,
    Loc326C6,
    Loc326DC,
    Loc32652,
    UnknownOffset(u16),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterProjectileCollisionDispatchLoc32682 {
    pub(crate) sprite_frame_link: Option<&'static str>,
    pub(crate) remaining_collision_budget: u16,
    pub(crate) clears_collision_type: bool,
    pub(crate) target: TarMonsterProjectileCollisionTargetLoc32682,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterProjectileCollisionRouteLoc32682 {
    ReturnOnly,
    DeleteCurrentObject,
    Dispatch(TarMonsterProjectileCollisionDispatchLoc32682),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterProjectileCollisionLoc32682 {
    pub(crate) route: TarMonsterProjectileCollisionRouteLoc32682,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterProjectileVelocityPair {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterProjectileLoopRouteLoc3263C {
    ContinueLoop,
    DeleteCurrentObject,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterProjectileLoopLoc3263C {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) y_velocity: i32,
    pub(crate) collision: TarMonsterProjectileCollisionLoc32682,
    pub(crate) route: TarMonsterProjectileLoopRouteLoc3263C,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterShotMotionLoc324D0 {
    pub(crate) x_pos: u32,
    pub(crate) x_velocity: i32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TarMonsterShotWaitRouteLoc324E6 {
    ContinueLoop,
    BurstCleanup,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterShotWaitLoc324E6 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) route: TarMonsterShotWaitRouteLoc324E6,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterBurstCleanupLoc324F8 {
    pub(crate) hides_original_shot: bool,
    pub(crate) clears_original_x_velocity: bool,
    pub(crate) clears_original_y_velocity: bool,
    pub(crate) child_count: u8,
    pub(crate) child_velocities: [(i32, i32); 5],
    pub(crate) hibernates_before_delete: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterBurstTailLoc32534 {
    pub(crate) child_count: u8,
    pub(crate) child_velocities: [(i32, i32); 3],
    pub(crate) hibernates_before_delete: bool,
    pub(crate) deletes_original_shot: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterCollisionDispatchLoc32386 {
    pub(crate) next_collision_type: u16,
    pub(crate) route: TarMonsterCollisionRouteLoc32386,
    pub(crate) clears_kid_x_velocity: bool,
}

pub(crate) fn tar_monster_loc_32248() -> &'static [u8] {
    // ASM loc_32248: `lea (unk_32264).l,a0` loads this control script pointer.
    &TAR_MONSTER_CONTROL_BYTES_LOC_32248
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_loc_3224e() -> TarMonsterLoc3224EReturn {
    // ASM loc_3224E: clear `collision_type(a3)` and preload `off_32290` into `a1`.
    TarMonsterLoc3224EReturn {
        collision_type: 0,
        jump_table: &TAR_MONSTER_LOC_32290_TABLE,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_loc_32256(raw_command: i16) -> TarMonsterLoc32256Return {
    // ASM loc_32256: read one signed word; negative -> restart script at loc_32248;
    // otherwise `move.b d0,d7` indexes `off_32290` by low-byte byte offset.
    if raw_command.is_negative() {
        return TarMonsterLoc32256Return {
            raw_command,
            route: TarMonsterLoc32256Route::Restart,
        };
    }

    let index = (raw_command & 0x00FF) as u8;
    let target = match index {
        0 => TarMonsterLoc32290Target::Loc322A8,
        4 => TarMonsterLoc32290Target::Loc322BC,
        8 => TarMonsterLoc32290Target::Loc322DA,
        12 => TarMonsterLoc32290Target::Loc322FC,
        16 => TarMonsterLoc32290Target::Loc32308,
        20 => TarMonsterLoc32290Target::Loc32310,
        _ => TarMonsterLoc32290Target::Invalid(index),
    };

    TarMonsterLoc32256Return {
        raw_command,
        route: TarMonsterLoc32256Route::Dispatch(target),
    }
}

/// Decode one word from `unk_32264`, preserving the ROM's byte-addressed cursor.
pub(crate) fn tar_monster_script_step_loc_32256(offset: u8) -> TarMonsterScriptStepLoc32256 {
    let offset = usize::from(offset);
    let bytes = TAR_MONSTER_CONTROL_BYTES_LOC_32248
        .get(offset..offset + 2)
        .expect("Tar Monster script cursor must address a complete word");
    let raw_command = i16::from_be_bytes([bytes[0], bytes[1]]);
    let route = tar_monster_loc_32256(raw_command).route;
    let command = match route {
        TarMonsterLoc32256Route::Restart => TarMonsterScriptCommandLoc32256::Restart,
        TarMonsterLoc32256Route::Dispatch(TarMonsterLoc32290Target::Loc322A8) => {
            TarMonsterScriptCommandLoc32256::Wait { dbf_seed: bytes[0] }
        }
        TarMonsterLoc32256Route::Dispatch(TarMonsterLoc32290Target::Loc322BC) => {
            TarMonsterScriptCommandLoc32256::Appear
        }
        TarMonsterLoc32256Route::Dispatch(TarMonsterLoc32290Target::Loc322DA) => {
            TarMonsterScriptCommandLoc32256::Disappear
        }
        TarMonsterLoc32256Route::Dispatch(TarMonsterLoc32290Target::Loc322FC) => {
            TarMonsterScriptCommandLoc32256::ToggleDirection
        }
        TarMonsterLoc32256Route::Dispatch(TarMonsterLoc32290Target::Loc32308) => {
            TarMonsterScriptCommandLoc32256::TogglePrimary
        }
        TarMonsterLoc32256Route::Dispatch(TarMonsterLoc32290Target::Loc32310) => {
            TarMonsterScriptCommandLoc32256::Attack
        }
        TarMonsterLoc32256Route::Dispatch(TarMonsterLoc32290Target::Invalid(command)) => {
            panic!("invalid Tar Monster script command byte offset {command:#04x}")
        }
    };

    TarMonsterScriptStepLoc32256 {
        command,
        next_offset: if command == TarMonsterScriptCommandLoc32256::Restart {
            0
        } else {
            (offset + 2) as u8
        },
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_wait_loop_loc_322aa(
    d0_after_loc_322a8: u16,
) -> TarMonsterWaitLoopLoc322AA {
    // ASM loc_322AA: hibernate one frame, check range, poll collision, then
    // `dbf d0,loc_322AA`. After the inclusive DBF loop, branch to loc_3224E.
    TarMonsterWaitLoopLoc322AA {
        hibernate_frames: u32::from(d0_after_loc_322a8) + 1,
        checks_object_range_each_frame: true,
        polls_collision_each_frame: true,
        next_collision_type: tar_monster_loc_3224e().collision_type,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_appear_sound_loc_322c0() -> TarMonsterAppearSoundLoc322C0 {
    // ASM loc_322C0: call `j_PlaySound` with `sfx_Tar_Monster_appears`, then
    // restore the script command from the stack before animation setup.
    TarMonsterAppearSoundLoc322C0 {
        sound_effect_label: "sfx_Tar_Monster_appears",
        preserves_script_command_d0: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_appear_animation_loc_322c8() -> TarMonsterAppearAnimationLoc322C8 {
    // ASM loc_322C8: load `stru_3272E`, initialise animation, hibernate until
    // it finishes, then branch back to the control-script dispatcher.
    TarMonsterAppearAnimationLoc322C8 {
        animation_label: "stru_3272E",
        hibernate_until_animation_finished: true,
        animation_frames: TAR_MONSTER_EMERGE_FRAMES,
        resumes_control_script: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_third_attack_wait_loc_3235e() -> TarMonsterThirdAttackWaitLoc3235E {
    // ASM loc_3235E seeds d0 with 8, then the following loop hibernates,
    // range-checks, and polls collision once per inclusive DBF iteration before restoring
    // the idle sprite link and resuming the bytecode loop at loc_3224E.
    TarMonsterThirdAttackWaitLoc3235E {
        dbf_seed: 8,
        hibernate_frames: 9,
        checks_object_range_each_frame: true,
        polls_collision_each_frame: true,
        restored_sprite_link: "LnkTo_unk_C77D6",
        resumes_control_script: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_collision_gate_loc_3237a(
    collision_type: i16,
) -> TarMonsterCollisionGateLoc3237A {
    // ASM loc_3237A copies `collision_type(a3)` into d7; negative values go
    // straight to the death path, zero returns, and positive values continue
    // with the captured collision id.
    if collision_type < 0 {
        return TarMonsterCollisionGateLoc3237A::Kill;
    }
    if collision_type == 0 {
        return TarMonsterCollisionGateLoc3237A::Return;
    }
    TarMonsterCollisionGateLoc3237A::NonZero {
        captured_collision_type: collision_type as u16,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_berzerker_collision_loc_323AA(
    berzerker_charging: bool,
) -> TarMonsterBerzerkerCollisionLoc323AA {
    // ASM loc_323AA: side contact only kills while `Berzerker_charging` is
    // non-zero. That accepted branch clears Kid's `$26(a4)` x velocity before
    // skipping the collision subroutine return and entering loc_323E6.
    if berzerker_charging {
        return TarMonsterBerzerkerCollisionLoc323AA {
            route: TarMonsterCollisionRouteLoc32386::BerzerkerKill,
            clears_kid_x_velocity: true,
        };
    }

    TarMonsterBerzerkerCollisionLoc323AA {
        route: TarMonsterCollisionRouteLoc32386::ReturnOnly,
        clears_kid_x_velocity: false,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_hurt_collision_loc_323C0(
    current_hitpoints: u16,
    child_visible: bool,
) -> TarMonsterHurtCollisionLoc323C0 {
    // ASM loc_323C0 saves `$13(a2)` into d2, clears `$13(a2)`, then decrements
    // d3. A zero result enters the death path; otherwise the hurt animation
    // runs while the linked child sprite stays hidden.
    let remaining_hitpoints = current_hitpoints.wrapping_sub(1);
    let route = if remaining_hitpoints == 0 {
        TarMonsterHurtRouteLoc323C0::Death
    } else {
        TarMonsterHurtRouteLoc323C0::HurtAnimation
    };

    TarMonsterHurtCollisionLoc323C0 {
        saved_child_visible: child_visible,
        child_visible_during_response: false,
        remaining_hitpoints,
        route,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_hurt_animation_loc_323D6(
    saved_child_visible: bool,
) -> TarMonsterHurtAnimationLoc323D6 {
    // ASM loc_323D6 runs the `stru_326EE` hurt animation to completion, restores
    // the child sprite visibility saved in d2, then resumes at loc_3224E.
    TarMonsterHurtAnimationLoc323D6 {
        animation_label: "stru_326EE",
        hibernate_until_animation_finished: true,
        restored_child_visible: saved_child_visible,
        resumes_control_script: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_respawn_marker_loc_3242C(
    object_status_slot: u8,
) -> TarMonsterRespawnMarkerLoc3242C {
    // ASM loc_3242C: the positive `$42(a5)` byte is doubled as a word offset
    // into EnemyStatus_Table, then `$2168` is written as the respawn marker.
    TarMonsterRespawnMarkerLoc3242C {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_death_status_update_loc_3243A(
    object_status_slot: u8,
) -> TarMonsterDeathStatusUpdateLoc3243A {
    // ASM loc_3243A: every status-table route converges on
    // j_Delete_CurrentObject; bit 7 clear reaches loc_3242C, bit 7+6 subtracts
    // `$400`, and bit 7 without bit 6 deletes without a table write.
    let route = if object_status_slot & 0x80 == 0 {
        TarMonsterDeathStatusRouteLoc3243A::WriteRespawnMarker(
            tar_monster_respawn_marker_loc_3242C(object_status_slot),
        )
    } else if object_status_slot & 0x40 != 0 {
        TarMonsterDeathStatusRouteLoc3243A::SubtractSpawnCounter {
            index: usize::from(object_status_slot & 0x3F),
            decrement: 0x0400,
        }
    } else {
        TarMonsterDeathStatusRouteLoc3243A::DeleteOnly
    };

    TarMonsterDeathStatusUpdateLoc3243A {
        route,
        deletes_current_object: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_shot_sound_loc_32468() -> TarMonsterShotSoundLoc32468 {
    // ASM loc_32468: preserve d0 around `j_PlaySound`, play
    // `sfx_Tar_Monster_shoots`, then load a `$3000003` GfxObject slot.
    TarMonsterShotSoundLoc32468 {
        sound_effect_label: "sfx_Tar_Monster_shoots",
        preserves_script_command_d0: true,
        gfx_object_slot_word: 0x0300_0003,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_shot_spawn_loc_3247E(
    source_x_pos: u32,
    source_y_pos: u32,
    source_x_direction: Direction,
    object_44_nonzero: bool,
) -> TarMonsterShotSpawnRouteLoc3247E {
    // ASM loc_3247E copies the source object's integer x/y words, then either
    // branches to the burst path or prepares the straight shot.
    if object_44_nonzero {
        let entry = tar_monster_retaliation_entry_loc_325AC(source_x_pos, source_y_pos);
        return TarMonsterShotSpawnRouteLoc3247E::RetaliationBurst {
            x_pos: entry.x_pos,
            y_pos: entry.y_pos,
            proceeds_to_burst_cleanup: entry.proceeds_to_burst_cleanup,
        };
    }

    let source_x_word = (source_x_pos >> 16) as u16;
    let source_y_word = (source_y_pos >> 16) as u16;
    TarMonsterShotSpawnRouteLoc3247E::StraightShot(TarMonsterShotSpawnLoc3247E {
        x_pos: u32::from(source_x_word.wrapping_sub(0x19)) << 16,
        y_pos: u32::from(source_y_word.wrapping_sub(0x12)) << 16,
        object_meta: 5,
        status_flag_13: true,
        is_moved: true,
        priority: 0,
        has_level_collision: true,
        sprite_frame_link: "LnkTo_unk_C781E",
        copied_x_direction: source_x_direction,
    })
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_retaliation_entry_loc_325AC(
    source_x_pos: u32,
    source_y_pos: u32,
) -> TarMonsterRetaliationEntryLoc325AC {
    // ASM loc_325AC subtracts `$10` from the copied y word, leaves x unchanged,
    // then branches directly into the shared burst cleanup at loc_324F8.
    let source_x_word = (source_x_pos >> 16) as u16;
    let source_y_word = (source_y_pos >> 16) as u16;
    TarMonsterRetaliationEntryLoc325AC {
        x_pos: u32::from(source_x_word) << 16,
        y_pos: u32::from(source_y_word.wrapping_sub(0x10)) << 16,
        proceeds_to_burst_cleanup: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_seeded_shot_loc_325B6(
    source_x_word: u16,
    source_y_word: u16,
    source_x_velocity: i32,
) -> TarMonsterSeededShotLoc325B6 {
    // ASM loc_325B6 loads slot `$3000003`, writes `$44/$46(a5)` as integer
    // x/y words, doubles `$48(a5)` with `asl.l #1`, and enters the shared
    // projectile setup with sprite `LnkTo_unk_C7BDE`, d1=0, and d0=$16.
    let setup = tar_monster_projectile_setup_loc_3261C(0x16);
    TarMonsterSeededShotLoc325B6 {
        gfx_object_slot_word: 0x0300_0003,
        x_pos: u32::from(source_x_word) << 16,
        y_pos: u32::from(source_y_word) << 16,
        x_velocity: source_x_velocity.wrapping_mul(2),
        sprite_frame_link: "LnkTo_unk_C7BDE",
        object_meta: setup.object_meta,
        status_flag_13: setup.status_flag_13,
        is_moved: setup.is_moved,
        has_kid_collision: setup.has_kid_collision,
        priority: setup.priority,
        has_level_collision: setup.has_level_collision,
        collision_sprite_swap_enabled: false,
        collision_response_budget: setup.collision_response_budget,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_burst_child_entry_loc_325E6(
    source_visual_x_pos: u32,
    source_visual_y_pos: u32,
    worker_x_velocity: i32,
    worker_y_velocity: i32,
) -> TarMonsterBurstChildEntryLoc325E6 {
    // ASM loc_325E6 loads a `$3000003` GfxObject slot, follows the worker's
    // source-object pointer to `$36(a0)`, copies that sprite x/y, then copies
    // `$44/$48(a5)` velocities before selecting `LnkTo_unk_C782E`.
    TarMonsterBurstChildEntryLoc325E6 {
        gfx_object_slot_word: 0x0300_0003,
        x_pos: source_visual_x_pos,
        y_pos: source_visual_y_pos,
        x_velocity: worker_x_velocity,
        y_velocity: worker_y_velocity,
        sprite_frame_link: "LnkTo_unk_C782E",
        collision_sprite_swap_enabled: true,
        falls_through_to_meta_seed: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_burst_child_meta_loc_32618() -> TarMonsterBurstChildMetaLoc32618 {
    // ASM loc_32618 seeds d0 with object_meta `$0005` for the burst child and
    // falls through to the shared Tar Monster projectile setup.
    TarMonsterBurstChildMetaLoc32618 {
        object_meta: 5,
        proceeds_to_projectile_setup: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_projectile_setup_loc_3261C(
    object_meta: u16,
) -> TarMonsterProjectileSetupLoc3261C {
    // ASM loc_3261C writes d0 to object_meta, runs the shared object setup,
    // enables display/movement/Kid+level collision, lowers priority, and seeds
    // d3 with two accepted collision responses before deletion.
    TarMonsterProjectileSetupLoc3261C {
        object_meta,
        status_flag_13: true,
        is_moved: true,
        has_kid_collision: true,
        priority: 0,
        has_level_collision: true,
        collision_response_budget: 2,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_projectile_dispatch_seed_loc_32698(
    collision_type: u16,
    collision_response_budget: u16,
) -> TarMonsterProjectileCollisionBudgetLoc32698 {
    // ASM loc_32698: `subq.w #1,d3; bmi.s loc_32652` spends one collision
    // response before clearing collision_type(a3), subtracting 4 from the
    // captured collision id, and using that offset to index off_32656.
    let remaining_collision_budget = collision_response_budget.wrapping_sub(1);
    if remaining_collision_budget & 0x8000 != 0 {
        return TarMonsterProjectileCollisionBudgetLoc32698::DeleteCurrentObject;
    }

    TarMonsterProjectileCollisionBudgetLoc32698::DispatchSeed(
        TarMonsterProjectileDispatchSeedLoc32698 {
            remaining_collision_budget,
            collision_offset: collision_type.wrapping_sub(4),
            clears_collision_type: true,
        },
    )
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_projectile_reflect_x_loc_326AA(x_velocity: i32) -> i32 {
    // ASM loc_326AA: copy x_vel(a3), arithmetic-shift it right by one, negate
    // the halved value, and write it back to x_vel(a3).
    (x_velocity >> 1).wrapping_neg()
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_projectile_reflect_y_loc_326B8(y_velocity: i32) -> i32 {
    // ASM loc_326B8: copy y_vel(a3), arithmetic-shift it right by one, negate
    // the halved value, then fall through to the shared y_vel(a3) writeback.
    tar_monster_projectile_write_y_velocity_loc_326C0((y_velocity >> 1).wrapping_neg())
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_projectile_write_y_velocity_loc_326C0(y_velocity_d7: i32) -> i32 {
    // ASM loc_326C0: `move.l d7,y_vel(a3); rts` writes the prepared d7 long
    // directly to the projectile's y_vel field.
    y_velocity_d7
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_projectile_negated_velocity_swap_loc_326C6(
    x_velocity: i32,
    y_velocity: i32,
) -> TarMonsterProjectileVelocityPair {
    // ASM loc_326C6: negate copied x_vel into d7, negate copied y_vel into d6,
    // then cross-write d6 to x_vel(a3) and d7 to y_vel(a3).
    TarMonsterProjectileVelocityPair {
        x_velocity: y_velocity.wrapping_neg(),
        y_velocity: x_velocity.wrapping_neg(),
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_projectile_velocity_swap_loc_326DC(
    x_velocity: i32,
    y_velocity: i32,
) -> TarMonsterProjectileVelocityPair {
    // ASM loc_326DC: copy x_vel to d7 and y_vel to d6, then cross-write d6 to
    // x_vel(a3) and d7 to y_vel(a3) without negation.
    TarMonsterProjectileVelocityPair {
        x_velocity: y_velocity,
        y_velocity: x_velocity,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_projectile_collision_loc_32682(
    collision_type: i16,
    collision_response_budget: u16,
    collision_sprite_swap_enabled: bool,
) -> TarMonsterProjectileCollisionLoc32682 {
    // ASM loc_32682 copies collision_type(a3) into d7: negative branches to
    // loc_32652, zero returns, and positive values optionally select the
    // collision sprite before spending one d3 response-budget word.
    if collision_type < 0 {
        return TarMonsterProjectileCollisionLoc32682 {
            route: TarMonsterProjectileCollisionRouteLoc32682::DeleteCurrentObject,
        };
    }

    if collision_type == 0 {
        return TarMonsterProjectileCollisionLoc32682 {
            route: TarMonsterProjectileCollisionRouteLoc32682::ReturnOnly,
        };
    }

    let collision_seed = match tar_monster_projectile_dispatch_seed_loc_32698(
        collision_type as u16,
        collision_response_budget,
    ) {
        TarMonsterProjectileCollisionBudgetLoc32698::DeleteCurrentObject => {
            return TarMonsterProjectileCollisionLoc32682 {
                route: TarMonsterProjectileCollisionRouteLoc32682::DeleteCurrentObject,
            };
        }
        TarMonsterProjectileCollisionBudgetLoc32698::DispatchSeed(collision_seed) => collision_seed,
    };

    let target = match collision_seed.collision_offset {
        0 | 4 | 28 | 32 => TarMonsterProjectileCollisionTargetLoc32682::Loc326AA,
        8 | 12 | 36 => TarMonsterProjectileCollisionTargetLoc32682::Loc326B8,
        16 => TarMonsterProjectileCollisionTargetLoc32682::Loc326C6,
        20 | 24 => TarMonsterProjectileCollisionTargetLoc32682::Loc326DC,
        40 => TarMonsterProjectileCollisionTargetLoc32682::Loc32652,
        _ => TarMonsterProjectileCollisionTargetLoc32682::UnknownOffset(
            collision_seed.collision_offset,
        ),
    };

    TarMonsterProjectileCollisionLoc32682 {
        route: TarMonsterProjectileCollisionRouteLoc32682::Dispatch(
            TarMonsterProjectileCollisionDispatchLoc32682 {
                sprite_frame_link: collision_sprite_swap_enabled.then_some("LnkTo_unk_C7836"),
                remaining_collision_budget: collision_seed.remaining_collision_budget,
                clears_collision_type: collision_seed.clears_collision_type,
                target,
            },
        ),
    }
}

#[allow(non_snake_case)]
fn tar_monster_projectile_collision_deletes_current_object_loc_3263C(
    collision: TarMonsterProjectileCollisionLoc32682,
) -> bool {
    match collision.route {
        TarMonsterProjectileCollisionRouteLoc32682::DeleteCurrentObject => true,
        TarMonsterProjectileCollisionRouteLoc32682::Dispatch(dispatch) => {
            dispatch.target == TarMonsterProjectileCollisionTargetLoc32682::Loc32652
        }
        TarMonsterProjectileCollisionRouteLoc32682::ReturnOnly => false,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_projectile_loop_loc_3263C(
    y_velocity: i32,
    collision_type: i16,
    collision_response_budget: u16,
    collision_sprite_swap_enabled: bool,
    byte_19_nonzero: bool,
) -> TarMonsterProjectileLoopLoc3263C {
    // ASM loc_3263C hibernates one frame, adds `$1000` to y_vel(a3), calls
    // loc_32682, then loops while byte `$19(a3)` is clear. Any collision path
    // that reaches loc_32652 deletes before the byte-19 loop test can repeat.
    let y_velocity = y_velocity.wrapping_add(0x1000);
    let collision = tar_monster_projectile_collision_loc_32682(
        collision_type,
        collision_response_budget,
        collision_sprite_swap_enabled,
    );
    let route = if byte_19_nonzero
        || tar_monster_projectile_collision_deletes_current_object_loc_3263C(collision)
    {
        TarMonsterProjectileLoopRouteLoc3263C::DeleteCurrentObject
    } else {
        TarMonsterProjectileLoopRouteLoc3263C::ContinueLoop
    };

    TarMonsterProjectileLoopLoc3263C {
        hibernates_one_frame: true,
        y_velocity,
        collision,
        route,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_shot_motion_loc_324D0(
    x_pos_after_spawn_setup: u32,
    copied_x_direction: Direction,
) -> TarMonsterShotMotionLoc324D0 {
    // ASM loc_324D0 starts with x_vel `$FFFC` (-4 px/frame). A non-zero
    // copied x_direction branches to loc_324DC, negating velocity and adding $32 px.
    match copied_x_direction {
        Direction::Left => TarMonsterShotMotionLoc324D0 {
            x_pos: x_pos_after_spawn_setup,
            x_velocity: -TAR_MONSTER_GLOB_SPEED_X,
        },
        Direction::Right => TarMonsterShotMotionLoc324D0 {
            x_pos: x_pos_after_spawn_setup.wrapping_add(0x32 << 16),
            x_velocity: TAR_MONSTER_GLOB_SPEED_X,
        },
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_shot_wait_loc_324E6(
    collision_type: u16,
    byte_19_nonzero: bool,
) -> TarMonsterShotWaitLoc324E6 {
    // ASM loc_324E6 hibernates once, exits on any non-zero collision_type, and
    // otherwise loops while byte `$19(a3)` is clear.
    let route = if collision_type != 0 || byte_19_nonzero {
        TarMonsterShotWaitRouteLoc324E6::BurstCleanup
    } else {
        TarMonsterShotWaitRouteLoc324E6::ContinueLoop
    };

    TarMonsterShotWaitLoc324E6 {
        hibernates_one_frame: true,
        route,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_burst_cleanup_loc_324F8() -> TarMonsterBurstCleanupLoc324F8 {
    // ASM loc_324F8 hides the impacted glob, clears both velocity longs, spawns
    // two fixed-velocity children, then falls through loc_32534 for the final
    // three children, one-frame hibernate, and original-glob delete.
    TarMonsterBurstCleanupLoc324F8 {
        hides_original_shot: true,
        clears_original_x_velocity: true,
        clears_original_y_velocity: true,
        child_count: 5,
        child_velocities: TAR_MONSTER_BURST_BLOB_VELOCITIES_LOC_324F8,
        hibernates_before_delete: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_burst_tail_loc_32534() -> TarMonsterBurstTailLoc32534 {
    // ASM loc_32534 allocates the final three burst workers with fixed `$44/$48`
    // velocity pairs, hibernates once, then deletes the source glob.
    TarMonsterBurstTailLoc32534 {
        child_count: 3,
        child_velocities: TAR_MONSTER_BURST_TAIL_VELOCITIES_LOC_32534,
        hibernates_before_delete: true,
        deletes_original_shot: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_collision_dispatch_loc_32386(
    collision_type: u16,
    berzerker_charging: bool,
) -> TarMonsterCollisionDispatchLoc32386 {
    // ASM loc_32386 immediately clears `collision_type(a3)`, then accepts
    // `$1C/$2C` as ordinary hit ids. `$20/$24` kill only while the Berzerker
    // charge flag is set; the charge path also clears Kid's x velocity.
    let route = match collision_type {
        0x001C | 0x002C => TarMonsterCollisionRouteLoc32386::Hurt,
        0x0020 | 0x0024 => tar_monster_berzerker_collision_loc_323AA(berzerker_charging).route,
        _ => TarMonsterCollisionRouteLoc32386::ReturnOnly,
    };
    TarMonsterCollisionDispatchLoc32386 {
        next_collision_type: 0,
        route,
        clears_kid_x_velocity: matches!(route, TarMonsterCollisionRouteLoc32386::BerzerkerKill),
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterInitLoc321C6 {
    pub(crate) hit_points: u8,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_init_loc_321c6(
    queued_x_word: u16,
    queued_y_word: u16,
    enemy_level: u8,
) -> TarMonsterInitLoc321C6 {
    // ASM loc_321C6 copies layout words 4/6 directly into x_pos/y_pos and uses the
    // layout level word as the Hitpoints_Enemy_05 table index.
    TarMonsterInitLoc321C6 {
        hit_points: tar_monster_hit_points_loc_321c6(enemy_level),
        x_pos: u32::from(queued_x_word) << 16,
        y_pos: u32::from(queued_y_word) << 16,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterInitLoc321E4 {
    pub(crate) hit_points: u8,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_init_loc_321e4(
    queued_x_word: u16,
    queued_y_word: u16,
    enemy_level: u8,
    blank_top_rows: u16,
) -> TarMonsterInitLoc321E4 {
    // ASM loc_321E4 calls sub_36FF4, sets object priority and object_meta to 5,
    // then runs the shared setup path (loc_32146) before storing `a3` in `$3A(a5)`.
    let base = tar_monster_init_loc_321c6(queued_x_word, queued_y_word, enemy_level);
    debug_assert_eq!(base.y_pos, u32::from(queued_y_word) << 16);
    TarMonsterInitLoc321E4 {
        hit_points: base.hit_points,
        priority: 0,
        object_meta: 5,
        x_pos: base.x_pos,
        y_pos: spawn_y_pos_sub_36FF4(
            EnemyFamily::TarMonster,
            u32::from(queued_y_word),
            blank_top_rows,
        ),
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct TarMonsterChildInitLoc32200 {
    pub(crate) sprite_frame_link: u16,
    pub(crate) object_meta: u16,
    pub(crate) priority: u8,
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) status_flag_13: bool,
    pub(crate) animation_phase: u16,
    pub(crate) vram_tile: u16,
}

#[allow(non_snake_case)]
pub(crate) fn tar_monster_child_init_loc_32200(
    parent_init: TarMonsterInitLoc321E4,
    sprite_frame_link: u16,
) -> TarMonsterChildInitLoc32200 {
    // ASM loc_32200 configures the child gfx object: writes the linked sprite frame,
    // runs the shared setup path (loc_32146), sets status/animation fields, then
    // links it through `$FFFFF86A` as an active object.
    TarMonsterChildInitLoc32200 {
        sprite_frame_link,
        object_meta: 5,
        priority: 0,
        x_pos: parent_init.x_pos,
        y_pos: parent_init.y_pos,
        status_flag_13: true,
        animation_phase: 2,
        vram_tile: 0x0100,
    }
}

/// Step a TarMonster through its bytecode-driven emerge/attack/submerge cycle.
/// ASM: Tar_Monster.asm, jump table off_32290.
/// Submerged → Emerging → Visible → Attacking → Submerging → Submerged (loop).
/// While Submerged, the enemy is invulnerable and invisible (handled by collision/render).
#[cfg(test)]
pub(crate) fn step_tar_monster(enemy: &mut ActiveEnemy, projectiles: &mut Vec<EnemyProjectile>) {
    let mut discarded_sound_effects = Vec::new();
    step_tar_monster_with_sounds(enemy, projectiles, &mut discarded_sound_effects);
}

pub(crate) fn step_tar_monster_with_sounds(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    played_sound_effects: &mut Vec<QueuedSoundEffect>,
) {
    let script = tar_monster_loc_32248();
    let reset = tar_monster_loc_3224e();
    debug_assert_eq!(script, TAR_MONSTER_CONTROL_BYTES_LOC_32248.as_slice());
    debug_assert_eq!(reset.collision_type, 0);
    debug_assert_eq!(reset.jump_table, &TAR_MONSTER_LOC_32290_TABLE);

    if enemy.wait_timer > 0 {
        // Check if we need to fire during attack sequence
        if enemy.tar_monster_state == TarMonsterState::Attacking {
            let attack_wait = tar_monster_third_attack_wait_loc_3235e();
            debug_assert_eq!(attack_wait.dbf_seed, 8);
            debug_assert!(attack_wait.checks_object_range_each_frame);
            debug_assert!(attack_wait.polls_collision_each_frame);
            debug_assert_eq!(attack_wait.restored_sprite_link, "LnkTo_unk_C77D6");
            debug_assert!(attack_wait.resumes_control_script);
            // ASM Tar_Monster.asm loc_3231C/loc_32348/loc_32360 each use the
            // loc_3235E `moveq #8,d0` DBF cadence, so each attack phase is
            // nine frames and the shot fires after the first phase.
            let attack_phase_frames = attack_wait.hibernate_frames;
            let elapsed = TAR_MONSTER_ATTACK_FRAMES - enemy.wait_timer;
            debug_assert_eq!(TAR_MONSTER_ATTACK_FRAMES, attack_phase_frames * 3);
            debug_assert_eq!(TAR_MONSTER_ATTACK_FIRE_FRAME + 1, attack_phase_frames);
            if elapsed + 1 == attack_phase_frames {
                let sound = tar_monster_shot_sound_loc_32468();
                debug_assert_eq!(sound.sound_effect_label, "sfx_Tar_Monster_shoots");
                debug_assert!(sound.preserves_script_command_d0);
                debug_assert_eq!(sound.gfx_object_slot_word, 0x0300_0003);
                // ASM Tar_Monster.asm loc_32468 plays the shoot cue before
                // entering loc_3247E.
                played_sound_effects.push(QueuedSoundEffect::TarMonsterShoots);
                // ASM loc_3231C/loc_32348: spawn the glob after the first `dbf` loop.
                // The spawn setup follows loc_3247E, and loc_324D0 applies the
                // direction-dependent velocity/mirror offset.
                let spawn = match tar_monster_shot_spawn_loc_3247E(
                    enemy.x_pos,
                    enemy.y_pos,
                    enemy.facing,
                    false,
                ) {
                    TarMonsterShotSpawnRouteLoc3247E::StraightShot(spawn) => spawn,
                    TarMonsterShotSpawnRouteLoc3247E::RetaliationBurst { .. } => return,
                };
                let motion =
                    tar_monster_shot_motion_loc_324D0(spawn.x_pos, spawn.copied_x_direction);
                let setup = tar_monster_projectile_setup_loc_3261C(0x16);
                debug_assert_eq!(setup.object_meta, 0x16);
                debug_assert!(setup.status_flag_13);
                debug_assert!(setup.is_moved);
                debug_assert!(setup.has_kid_collision);
                debug_assert_eq!(setup.priority, 0);
                debug_assert!(setup.has_level_collision);
                projectiles.push(EnemyProjectile {
                    x_pos: motion.x_pos,
                    y_pos: spawn.y_pos,
                    x_velocity: motion.x_velocity,
                    lifetime: 120,
                    // ASM loc_324D0 sets only x_vel. The default zero gravity
                    // remains; loc_3263C's `$1000` belongs to burst children.
                    kind: EnemyProjectileKind::TarMonsterStraightGlob,
                    tar_monster_collision_response_budget: setup.collision_response_budget,
                    ..Default::default()
                });
            }
        }
        enemy.wait_timer -= 1;
        if enemy.wait_timer > 0 {
            return;
        }
        match enemy.tar_monster_state {
            TarMonsterState::Emerging => enemy.tar_monster_state = TarMonsterState::Visible,
            TarMonsterState::Attacking => enemy.tar_monster_state = TarMonsterState::Visible,
            TarMonsterState::Submerging => enemy.tar_monster_state = TarMonsterState::Submerged,
            TarMonsterState::Submerged | TarMonsterState::Visible => {}
        }
    }

    // ASM loc_32256 runs immediate commands back-to-back until a wait or
    // animation hibernates. `dragon_script_cmd` stores the byte offset in
    // `unk_32264`, not a dense command index.
    loop {
        let step = tar_monster_script_step_loc_32256(enemy.dragon_script_cmd);
        enemy.dragon_script_cmd = step.next_offset;
        match step.command {
            TarMonsterScriptCommandLoc32256::Wait { dbf_seed } => {
                let wait = tar_monster_wait_loop_loc_322aa(u16::from(dbf_seed));
                enemy.wait_timer = wait.hibernate_frames as u16;
                return;
            }
            TarMonsterScriptCommandLoc32256::Appear => {
                let sound = tar_monster_appear_sound_loc_322c0();
                debug_assert_eq!(sound.sound_effect_label, "sfx_Tar_Monster_appears");
                // ASM loc_322BC/loc_322C0 plays the cue before `stru_3272E`.
                played_sound_effects.push(QueuedSoundEffect::TarMonsterAppears);
                let animation = tar_monster_appear_animation_loc_322c8();
                debug_assert_eq!(animation.animation_label, "stru_3272E");
                enemy.tar_monster_state = TarMonsterState::Emerging;
                enemy.wait_timer = animation.animation_frames;
                return;
            }
            TarMonsterScriptCommandLoc32256::Disappear => {
                // ASM loc_322DA hides primary GfxObject `$13(a2)` and runs
                // `stru_3274C` on the linked animated body.
                enemy.dragon_script_variant &= !TAR_MONSTER_PRIMARY_VISIBLE_LOC_32308;
                played_sound_effects.push(QueuedSoundEffect::TarMonsterDisappears);
                enemy.tar_monster_state = TarMonsterState::Submerging;
                enemy.wait_timer = TAR_MONSTER_SUBMERGE_FRAMES;
                return;
            }
            TarMonsterScriptCommandLoc32256::ToggleDirection => {
                // ASM loc_322FC toggles both the body and linked child direction.
                enemy.facing = opposite_direction(enemy.facing);
            }
            TarMonsterScriptCommandLoc32256::TogglePrimary => {
                // ASM loc_32308: `not.b $13(a2)` toggles the static primary
                // GfxObject independently from the linked animated body.
                enemy.dragon_script_variant ^= TAR_MONSTER_PRIMARY_VISIBLE_LOC_32308;
            }
            TarMonsterScriptCommandLoc32256::Attack => {
                // ASM loc_32310: `sf $13(a2)`.
                enemy.dragon_script_variant &= !TAR_MONSTER_PRIMARY_VISIBLE_LOC_32308;
                enemy.tar_monster_state = TarMonsterState::Attacking;
                enemy.wait_timer = TAR_MONSTER_ATTACK_FRAMES;
                return;
            }
            TarMonsterScriptCommandLoc32256::Restart => {}
        }
    }
}

/// Spawn the five-child burst from the Tar Monster glob impact path.
pub(crate) fn spawn_tar_monster_retaliation(
    projectiles: &mut Vec<EnemyProjectile>,
    x: u32,
    y: u32,
) {
    let cleanup = tar_monster_burst_cleanup_loc_324F8();
    let tail = tar_monster_burst_tail_loc_32534();
    debug_assert_eq!(
        usize::from(cleanup.child_count),
        2 + usize::from(tail.child_count)
    );
    debug_assert!(tail.hibernates_before_delete);
    debug_assert!(tail.deletes_original_shot);
    let meta = tar_monster_burst_child_meta_loc_32618();
    debug_assert!(meta.proceeds_to_projectile_setup);
    let setup = tar_monster_projectile_setup_loc_3261C(meta.object_meta);
    debug_assert_eq!(setup.object_meta, 5);
    debug_assert!(setup.status_flag_13);
    debug_assert!(setup.is_moved);
    debug_assert!(setup.has_kid_collision);
    debug_assert_eq!(setup.priority, 0);
    debug_assert!(setup.has_level_collision);
    let child_velocities = cleanup.child_velocities[..2]
        .iter()
        .copied()
        .chain(tail.child_velocities);
    for (xv, yv) in child_velocities {
        let entry = tar_monster_burst_child_entry_loc_325E6(x, y, xv, yv);
        projectiles.push(EnemyProjectile {
            x_pos: entry.x_pos,
            y_pos: entry.y_pos,
            x_velocity: entry.x_velocity,
            y_velocity: entry.y_velocity,
            lifetime: 60,
            gravity: 0x1000,
            kind: EnemyProjectileKind::TarMonsterBurstChild,
            // ASM loc_324F8 allocates all five Object slots before their
            // loc_325E6 callbacks load GfxObjects on the next scheduler pass.
            animation_elapsed_frames: 1,
            tar_monster_collision_response_budget: setup.collision_response_budget,
            ..Default::default()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    #[test]
    fn tar_monster_live_sprite_phase_walks_and_reverses_stru_3272e_3274c() {
        // ASM stru_3272E/stru_3274C: seven mappings last four frames each,
        // with the disappearance table reversing the emergence table.
        for (wait_timer, emerging, submerging) in [
            (28, 0, 6),
            (25, 0, 6),
            (24, 1, 5),
            (20, 2, 4),
            (8, 5, 1),
            (4, 6, 0),
            (1, 6, 0),
        ] {
            assert_eq!(
                tar_monster_live_sprite_phase_stru_3272e_3274c(
                    TarMonsterState::Emerging,
                    wait_timer,
                ),
                emerging
            );
            assert_eq!(
                tar_monster_live_sprite_phase_stru_3272e_3274c(
                    TarMonsterState::Submerging,
                    wait_timer,
                ),
                submerging
            );
        }
        assert_eq!(
            tar_monster_live_sprite_phase_stru_3272e_3274c(TarMonsterState::Submerged, 0),
            0
        );
        assert_eq!(
            tar_monster_live_sprite_phase_stru_3272e_3274c(TarMonsterState::Visible, 0),
            6
        );
        for (wait_timer, expected) in [(27, 13), (19, 13), (18, 14), (10, 14), (9, 13), (1, 13)] {
            assert_eq!(
                tar_monster_live_sprite_phase_stru_3272e_3274c(
                    TarMonsterState::Attacking,
                    wait_timer,
                ),
                expected,
                "ASM loc_32310/loc_3231C/loc_32348 holds C77DE/C77E6/C77DE for three nine-frame attack spans",
            );
        }
    }

    #[test]
    fn tar_monster_hp_matches_loc_321c6_hitpoint_table() {
        // ASM Tar_Monster.asm `Hitpoints_Enemy_05` is indexed by `enemy_level`.
        assert_eq!(tar_monster_hit_points_loc_321c6(0), 2);
        assert_eq!(tar_monster_hit_points_loc_321c6(1), 4);
        assert_eq!(tar_monster_hit_points_loc_321c6(2), 6);
        assert_eq!(tar_monster_hit_points_loc_321c6(3), 0);
        assert_eq!(tar_monster_hit_points_loc_321c6(10), 0);
    }

    #[test]
    fn tar_monster_init_loc_321c6_copies_layout_xy_word_values_per_native() {
        // ASM loc_321C6 loads layout word-2/word-3 directly into x/y and reads
        // word-1 as the level index.
        assert_eq!(
            tar_monster_init_loc_321c6(0x0120, 0x0080, 1),
            TarMonsterInitLoc321C6 {
                hit_points: 4,
                x_pos: 0x0120 << 16,
                y_pos: 0x0080 << 16,
            }
        );

        assert_eq!(
            tar_monster_init_loc_321c6(0x003F, 0x0010, 2),
            TarMonsterInitLoc321C6 {
                hit_points: 6,
                x_pos: 0x003F << 16,
                y_pos: 0x0010 << 16,
            }
        );
    }

    #[test]
    fn tar_monster_init_loc_321e4_calls_36ff4_and_forwards_meta() {
        // ASM loc_321E4 applies the blank-row Y adjustment via sub_36FF4,
        // clears priority, and writes object_meta to 5.
        let init = tar_monster_init_loc_321e4(0x0120, 0x0080, 0, 2);
        assert_eq!(
            init,
            TarMonsterInitLoc321E4 {
                hit_points: 2,
                priority: 0,
                object_meta: 5,
                x_pos: 0x0120 << 16,
                y_pos: 0x0080_u32.saturating_add(2 * 16) << 16,
            }
        );
    }

    #[test]
    fn tar_monster_control_script_pointer_loc_32248_starts_at_unk_32264() {
        let bytes = tar_monster_loc_32248();

        assert_eq!(
            bytes,
            TAR_MONSTER_CONTROL_BYTES_LOC_32248.as_slice(),
            "ASM Tar_Monster.asm loc_32248 loads `unk_32264` into `a0`"
        );
        assert!(
            !bytes.is_empty(),
            "loc_32248 control-script source table must be populated"
        );
    }

    #[test]
    fn tar_monster_loc_3224e_clears_collision_type_and_picks_off_32290() {
        let init = tar_monster_loc_3224e();

        assert_eq!(
            init.collision_type, 0,
            "loc_3224E executes `clr.w collision_type(a3)`"
        );
        assert_eq!(init.jump_table[0], TarMonsterLoc32290Target::Loc322A8);
        assert_eq!(init.jump_table[1], TarMonsterLoc32290Target::Loc322BC);
        assert_eq!(init.jump_table[2], TarMonsterLoc32290Target::Loc322DA);
        assert_eq!(init.jump_table[3], TarMonsterLoc32290Target::Loc322FC);
        assert_eq!(init.jump_table[4], TarMonsterLoc32290Target::Loc32308);
        assert_eq!(init.jump_table[5], TarMonsterLoc32290Target::Loc32310);
    }

    #[test]
    fn tar_monster_loc_32256_restarts_on_negative_and_dispatches_by_low_byte_offset() {
        let sentinel = tar_monster_loc_32256(-1);
        assert_eq!(
            sentinel,
            TarMonsterLoc32256Return {
                raw_command: -1,
                route: TarMonsterLoc32256Route::Restart,
            }
        );

        for (command, target) in [
            (0x0000, TarMonsterLoc32290Target::Loc322A8),
            (0x0004, TarMonsterLoc32290Target::Loc322BC),
            (0x0008, TarMonsterLoc32290Target::Loc322DA),
            (0x000C, TarMonsterLoc32290Target::Loc322FC),
            (0x0010, TarMonsterLoc32290Target::Loc32308),
            (0x0014, TarMonsterLoc32290Target::Loc32310),
        ] {
            assert_eq!(
                tar_monster_loc_32256(command).route,
                TarMonsterLoc32256Route::Dispatch(target),
                "ASM Tar_Monster.asm loc_32256 uses byte offset ${command:04X} to index off_32290"
            );
        }
        let dense_index_five = i16::from_be_bytes([0x00, 0x05]);
        assert_eq!(
            tar_monster_loc_32256(dense_index_five).route,
            TarMonsterLoc32256Route::Dispatch(TarMonsterLoc32290Target::Invalid(5)),
            "ASM Tar_Monster.asm loc_32256 uses the command low byte as a byte offset into a longword table, not as a dense target index"
        );
    }

    #[test]
    fn tar_monster_script_decoder_preserves_unk_32264_words_and_byte_offsets() {
        use TarMonsterScriptCommandLoc32256 as Command;

        let expected = [
            (0, Command::Appear),
            (2, Command::TogglePrimary),
            (4, Command::Wait { dbf_seed: 0x5A }),
            (6, Command::TogglePrimary),
            (8, Command::Wait { dbf_seed: 0x05 }),
            (10, Command::TogglePrimary),
            (12, Command::Wait { dbf_seed: 0x0A }),
            (14, Command::TogglePrimary),
            (16, Command::Wait { dbf_seed: 0x07 }),
            (18, Command::TogglePrimary),
            (20, Command::Wait { dbf_seed: 0x32 }),
            (22, Command::Attack),
            (24, Command::TogglePrimary),
            (26, Command::Wait { dbf_seed: 0x1E }),
            (28, Command::Disappear),
            (30, Command::Wait { dbf_seed: 0x46 }),
            (32, Command::ToggleDirection),
            (34, Command::Appear),
            (36, Command::Wait { dbf_seed: 0x14 }),
            (38, Command::Disappear),
            (40, Command::Wait { dbf_seed: 0x46 }),
            (42, Command::Restart),
        ];

        for (offset, command) in expected {
            let step = tar_monster_script_step_loc_32256(offset);
            assert_eq!(step.command, command, "ASM unk_32264 byte offset {offset}");
            assert_eq!(
                step.next_offset,
                if command == Command::Restart {
                    0
                } else {
                    offset + 2
                }
            );
        }
    }

    #[test]
    fn tar_monster_wait_loop_loc_322aa_uses_inclusive_dbf_count() {
        assert_eq!(
            tar_monster_wait_loop_loc_322aa(0),
            TarMonsterWaitLoopLoc322AA {
                hibernate_frames: 1,
                checks_object_range_each_frame: true,
                polls_collision_each_frame: true,
                next_collision_type: 0,
            },
            "ASM Tar_Monster.asm loc_322AA executes the loop body once before `dbf` exits from d0=0"
        );
        assert_eq!(
            tar_monster_wait_loop_loc_322aa(0x5A).hibernate_frames,
            91,
            "ASM Tar_Monster.asm loc_322AA makes script wait byte $5A last 91 one-frame hibernates"
        );
    }

    #[test]
    fn tar_monster_appear_sound_loc_322c0_preserves_script_command() {
        assert_eq!(
            tar_monster_appear_sound_loc_322c0(),
            TarMonsterAppearSoundLoc322C0 {
                sound_effect_label: "sfx_Tar_Monster_appears",
                preserves_script_command_d0: true,
            },
            "ASM Tar_Monster.asm loc_322C0 restores d0 after j_PlaySound so the script command survives the audio call"
        );
    }

    #[test]
    fn tar_monster_appear_animation_loc_322c8_waits_until_finished() {
        assert_eq!(
            tar_monster_appear_animation_loc_322c8(),
            TarMonsterAppearAnimationLoc322C8 {
                animation_label: "stru_3272E",
                hibernate_until_animation_finished: true,
                animation_frames: TAR_MONSTER_EMERGE_FRAMES,
                resumes_control_script: true,
            },
            "ASM Tar_Monster.asm loc_322C8 starts stru_3272E and waits for the animation to finish before reading the next command"
        );
    }

    #[test]
    fn tar_monster_third_attack_wait_loc_3235e_is_nine_collision_polled_frames() {
        assert_eq!(
            tar_monster_third_attack_wait_loc_3235e(),
            TarMonsterThirdAttackWaitLoc3235E {
                dbf_seed: 8,
                hibernate_frames: 9,
                checks_object_range_each_frame: true,
                polls_collision_each_frame: true,
                restored_sprite_link: "LnkTo_unk_C77D6",
                resumes_control_script: true,
            },
            "ASM Tar_Monster.asm loc_3235E seeds d0=8, so loc_32360 performs nine hibernated collision polls before restoring LnkTo_unk_C77D6"
        );
    }

    #[test]
    fn tar_monster_collision_gate_loc_3237a_routes_negative_zero_and_positive() {
        assert_eq!(
            tar_monster_collision_gate_loc_3237a(-1),
            TarMonsterCollisionGateLoc3237A::Kill,
            "ASM Tar_Monster.asm loc_3237A branches to the death path on a negative collision word"
        );
        assert_eq!(
            tar_monster_collision_gate_loc_3237a(0),
            TarMonsterCollisionGateLoc3237A::Return,
            "ASM Tar_Monster.asm loc_3237A returns immediately when collision_type(a3) is zero"
        );
        assert_eq!(
            tar_monster_collision_gate_loc_3237a(0x001C),
            TarMonsterCollisionGateLoc3237A::NonZero {
                captured_collision_type: 0x001C
            },
            "ASM Tar_Monster.asm loc_3237A preserves the non-zero collision id in d7 for the following dispatch"
        );
    }

    #[test]
    fn tar_monster_collision_dispatch_loc_32386_clears_and_routes_hit_ids() {
        assert_eq!(
            tar_monster_collision_dispatch_loc_32386(0x001C, false),
            TarMonsterCollisionDispatchLoc32386 {
                next_collision_type: 0,
                route: TarMonsterCollisionRouteLoc32386::Hurt,
                clears_kid_x_velocity: false,
            },
            "ASM Tar_Monster.asm loc_32386 clears collision_type(a3), then treats $1C as an ordinary hit"
        );
        assert_eq!(
            tar_monster_collision_dispatch_loc_32386(0x002C, false).route,
            TarMonsterCollisionRouteLoc32386::Hurt,
            "ASM Tar_Monster.asm loc_32386 accepts $2C through the same ordinary-hit branch"
        );
    }

    #[test]
    fn tar_monster_collision_dispatch_loc_32386_requires_berzerker_charge_for_side_ids() {
        assert_eq!(
            tar_monster_collision_dispatch_loc_32386(0x0020, false),
            TarMonsterCollisionDispatchLoc32386 {
                next_collision_type: 0,
                route: TarMonsterCollisionRouteLoc32386::ReturnOnly,
                clears_kid_x_velocity: false,
            },
            "ASM Tar_Monster.asm loc_32386 consumes $20 but ignores it when Berzerker_charging is clear"
        );
        assert_eq!(
            tar_monster_collision_dispatch_loc_32386(0x0024, true),
            TarMonsterCollisionDispatchLoc32386 {
                next_collision_type: 0,
                route: TarMonsterCollisionRouteLoc32386::BerzerkerKill,
                clears_kid_x_velocity: true,
            },
            "ASM Tar_Monster.asm loc_32386 accepts $24 only when Berzerker_charging is set and clears Kid x_vel on that path"
        );
        assert_eq!(
            tar_monster_collision_dispatch_loc_32386(0x0028, true),
            TarMonsterCollisionDispatchLoc32386 {
                next_collision_type: 0,
                route: TarMonsterCollisionRouteLoc32386::ReturnOnly,
                clears_kid_x_velocity: false,
            },
            "ASM Tar_Monster.asm loc_32386 clears even unrecognised non-zero collision ids before returning"
        );
    }

    #[test]
    fn tar_monster_berzerker_collision_loc_323aa_requires_charging_and_clears_kid_x_velocity() {
        assert_eq!(
            tar_monster_berzerker_collision_loc_323AA(false),
            TarMonsterBerzerkerCollisionLoc323AA {
                route: TarMonsterCollisionRouteLoc32386::ReturnOnly,
                clears_kid_x_velocity: false,
            },
            "ASM Tar_Monster.asm loc_323AA returns when Berzerker_charging is zero"
        );
        assert_eq!(
            tar_monster_berzerker_collision_loc_323AA(true),
            TarMonsterBerzerkerCollisionLoc323AA {
                route: TarMonsterCollisionRouteLoc32386::BerzerkerKill,
                clears_kid_x_velocity: true,
            },
            "ASM Tar_Monster.asm loc_323AA clears Kid `$26(a4)` x velocity before jumping to loc_323E6"
        );
    }

    #[test]
    fn tar_monster_hurt_collision_loc_323c0_hides_child_and_decrements_hitpoints() {
        assert_eq!(
            tar_monster_hurt_collision_loc_323C0(3, true),
            TarMonsterHurtCollisionLoc323C0 {
                saved_child_visible: true,
                child_visible_during_response: false,
                remaining_hitpoints: 2,
                route: TarMonsterHurtRouteLoc323C0::HurtAnimation,
            },
            "ASM Tar_Monster.asm loc_323C0 saves `$13(a2)`, clears it, subtracts one from d3, and continues the hurt response when d3 is still non-zero"
        );
    }

    #[test]
    fn tar_monster_hurt_collision_loc_323c0_enters_death_path_when_hp_reaches_zero() {
        assert_eq!(
            tar_monster_hurt_collision_loc_323C0(1, false),
            TarMonsterHurtCollisionLoc323C0 {
                saved_child_visible: false,
                child_visible_during_response: false,
                remaining_hitpoints: 0,
                route: TarMonsterHurtRouteLoc323C0::Death,
            },
            "ASM Tar_Monster.asm loc_323C0 branches to loc_323E6 when `subq.w #1,d3` leaves zero"
        );
    }

    #[test]
    fn tar_monster_hurt_animation_loc_323d6_restores_saved_child_visibility() {
        assert_eq!(
            tar_monster_hurt_animation_loc_323D6(true),
            TarMonsterHurtAnimationLoc323D6 {
                animation_label: "stru_326EE",
                hibernate_until_animation_finished: true,
                restored_child_visible: true,
                resumes_control_script: true,
            },
            "ASM Tar_Monster.asm loc_323D6 should wait for stru_326EE, restore d2 into `$13(a2)`, and return to loc_3224E"
        );
        assert!(
            !tar_monster_hurt_animation_loc_323D6(false).restored_child_visible,
            "ASM Tar_Monster.asm loc_323D6 restores the exact saved `$13(a2)` byte rather than forcing visibility on"
        );
    }

    #[test]
    fn tar_monster_death_status_loc_3242c_writes_respawn_marker_for_positive_slot() {
        assert_eq!(
            tar_monster_respawn_marker_loc_3242C(0x05),
            TarMonsterRespawnMarkerLoc3242C {
                index: 0x05,
                word_value: 0x2168,
            },
            "ASM Tar_Monster.asm loc_3242C doubles positive `$42(a5)` into an EnemyStatus_Table word offset and writes `$2168`"
        );
    }

    #[test]
    fn tar_monster_death_status_loc_3243a_always_deletes_after_status_route() {
        let positive = tar_monster_death_status_update_loc_3243A(0x02);
        assert!(
            positive.deletes_current_object,
            "ASM Tar_Monster.asm loc_3243A is the convergence jump to j_Delete_CurrentObject after loc_3242C"
        );

        let delete_only = tar_monster_death_status_update_loc_3243A(0x80);
        assert_eq!(
            delete_only.route,
            TarMonsterDeathStatusRouteLoc3243A::DeleteOnly,
            "ASM Tar_Monster.asm loc_3243A is also reached directly when `$42(a5)` has bit 7 set and bit 6 clear"
        );
        assert!(delete_only.deletes_current_object);

        assert_eq!(
            tar_monster_death_status_update_loc_3243A(0xC7).route,
            TarMonsterDeathStatusRouteLoc3243A::SubtractSpawnCounter {
                index: 0x07,
                decrement: 0x0400,
            },
            "ASM Tar_Monster.asm loc_3243A receives the bit-6 path after subtracting `$400` from EnemyStatus_Table[d0 & $3F]"
        );
    }

    #[test]
    fn tar_monster_shot_sound_loc_32468_preserves_d0_and_loads_slot() {
        assert_eq!(
            tar_monster_shot_sound_loc_32468(),
            TarMonsterShotSoundLoc32468 {
                sound_effect_label: "sfx_Tar_Monster_shoots",
                preserves_script_command_d0: true,
                gfx_object_slot_word: 0x0300_0003,
            },
            "ASM Tar_Monster.asm loc_32468 saves d0 around the shoot SFX before loading GfxObject slot `$3000003`"
        );
    }

    #[test]
    fn tar_monster_shot_spawn_loc_3247e_copies_pose_and_sets_straight_shot_flags() {
        assert_eq!(
            tar_monster_shot_spawn_loc_3247E(0x0120_0000, 0x0080_0000, Direction::Right, false),
            TarMonsterShotSpawnRouteLoc3247E::StraightShot(TarMonsterShotSpawnLoc3247E {
                x_pos: 0x0107_0000,
                y_pos: 0x006E_0000,
                object_meta: 5,
                status_flag_13: true,
                is_moved: true,
                priority: 0,
                has_level_collision: true,
                sprite_frame_link: "LnkTo_unk_C781E",
                copied_x_direction: Direction::Right,
            }),
            "ASM Tar_Monster.asm loc_3247E copies source x/y, subtracts $19/$12 for the straight shot, then copies x_direction"
        );
    }

    #[test]
    fn tar_monster_shot_spawn_loc_3247e_routes_nonzero_object_44_to_burst_path() {
        assert_eq!(
            tar_monster_shot_spawn_loc_3247E(0x0120_0000, 0x0080_0000, Direction::Left, true),
            TarMonsterShotSpawnRouteLoc3247E::RetaliationBurst {
                x_pos: 0x0120_0000,
                y_pos: 0x0070_0000,
                proceeds_to_burst_cleanup: true,
            },
            "ASM Tar_Monster.asm loc_3247E branches when `$44(a5)` is non-zero, subtracting only $10 from y before the burst cleanup"
        );
    }

    #[test]
    fn tar_monster_shot_loc_3247e_325ac_copies_and_wraps_position_words() {
        let source_x = (0x0010 << 16) | 0xBEEF;
        let source_y = (0x0005 << 16) | 0xCAFE;

        assert_eq!(
            tar_monster_shot_spawn_loc_3247E(source_x, source_y, Direction::Left, false),
            TarMonsterShotSpawnRouteLoc3247E::StraightShot(TarMonsterShotSpawnLoc3247E {
                x_pos: 0xFFF7_0000,
                y_pos: 0xFFF3_0000,
                object_meta: 5,
                status_flag_13: true,
                is_moved: true,
                priority: 0,
                has_level_collision: true,
                sprite_frame_link: "LnkTo_unk_C781E",
                copied_x_direction: Direction::Left,
            }),
            "ASM Tar_Monster.asm loc_3247E copies position words before wrapping SUBI.W offsets"
        );
        assert_eq!(
            tar_monster_shot_spawn_loc_3247E(source_x, source_y, Direction::Right, true),
            TarMonsterShotSpawnRouteLoc3247E::RetaliationBurst {
                x_pos: 0x0010_0000,
                y_pos: 0xFFF5_0000,
                proceeds_to_burst_cleanup: true,
            },
            "ASM Tar_Monster.asm loc_3247E/loc_325AC copies position words before the retaliation Y subtraction"
        );
    }

    #[test]
    fn tar_monster_retaliation_entry_loc_325ac_keeps_x_and_offsets_y_before_cleanup() {
        assert_eq!(
            tar_monster_retaliation_entry_loc_325AC(0x0120_0000, 0x0080_0000),
            TarMonsterRetaliationEntryLoc325AC {
                x_pos: 0x0120_0000,
                y_pos: 0x0070_0000,
                proceeds_to_burst_cleanup: true,
            },
            "ASM Tar_Monster.asm loc_325AC subtracts `$10` from y_pos only, then enters the shared burst cleanup"
        );
    }

    #[test]
    fn tar_monster_seeded_shot_loc_325b6_copies_words_and_doubles_x_velocity() {
        assert_eq!(
            tar_monster_seeded_shot_loc_325B6(0x0120, 0x0080, -0x0001_0000),
            TarMonsterSeededShotLoc325B6 {
                gfx_object_slot_word: 0x0300_0003,
                x_pos: 0x0120_0000,
                y_pos: 0x0080_0000,
                x_velocity: -0x0002_0000,
                sprite_frame_link: "LnkTo_unk_C7BDE",
                object_meta: 0x16,
                status_flag_13: true,
                is_moved: true,
                has_kid_collision: true,
                priority: 0,
                has_level_collision: true,
                collision_sprite_swap_enabled: false,
                collision_response_budget: 2,
            },
            "ASM Tar_Monster.asm loc_325B6 copies `$44/$46(a5)` into x/y, doubles `$48(a5)` with `asl.l #1`, and seeds object_meta `$16`"
        );

        assert_eq!(
            tar_monster_seeded_shot_loc_325B6(0x0001, 0x0002, i32::MIN).x_velocity,
            0,
            "ASM Tar_Monster.asm loc_325B6 uses 32-bit ASL semantics, so the native model must wrap when the top bit shifts out"
        );
    }

    #[test]
    fn tar_monster_burst_child_entry_loc_325e6_copies_source_visual_pose_and_worker_velocity() {
        assert_eq!(
            tar_monster_burst_child_entry_loc_325E6(
                0x0120_8000,
                0x0070_4000,
                0x0000_8000,
                -0x0002_0000
            ),
            TarMonsterBurstChildEntryLoc325E6 {
                gfx_object_slot_word: 0x0300_0003,
                x_pos: 0x0120_8000,
                y_pos: 0x0070_4000,
                x_velocity: 0x0000_8000,
                y_velocity: -0x0002_0000,
                sprite_frame_link: "LnkTo_unk_C782E",
                collision_sprite_swap_enabled: true,
                falls_through_to_meta_seed: true,
            },
            "ASM Tar_Monster.asm loc_325E6 copies the linked source sprite pose, uses `$44/$48(a5)` as velocities, and leaves d1 non-zero for collision sprite swaps"
        );
    }

    #[test]
    fn tar_monster_burst_child_meta_loc_32618_seeds_object_meta_five() {
        assert_eq!(
            tar_monster_burst_child_meta_loc_32618(),
            TarMonsterBurstChildMetaLoc32618 {
                object_meta: 5,
                proceeds_to_projectile_setup: true,
            },
            "ASM Tar_Monster.asm loc_32618 writes `#5` to d0 so burst children reuse the Tar Monster projectile object_meta"
        );
    }

    #[test]
    fn tar_monster_projectile_setup_loc_3261c_applies_shared_flags_and_collision_budget() {
        assert_eq!(
            tar_monster_projectile_setup_loc_3261C(0x16),
            TarMonsterProjectileSetupLoc3261C {
                object_meta: 0x16,
                status_flag_13: true,
                is_moved: true,
                has_kid_collision: true,
                priority: 0,
                has_level_collision: true,
                collision_response_budget: 2,
            },
            "ASM Tar_Monster.asm loc_3261C stores d0 into object_meta, sets movement/collision flags, and starts d3 at 2"
        );

        assert_eq!(
            tar_monster_projectile_setup_loc_3261C(5).object_meta,
            tar_monster_burst_child_meta_loc_32618().object_meta,
            "burst children enter the same shared setup after their meta seed"
        );
    }

    #[test]
    fn tar_monster_projectile_dispatch_seed_loc_32698_spends_budget_before_table_offset() {
        assert_eq!(
            tar_monster_projectile_dispatch_seed_loc_32698(0x000C, 2),
            TarMonsterProjectileCollisionBudgetLoc32698::DispatchSeed(
                TarMonsterProjectileDispatchSeedLoc32698 {
                    remaining_collision_budget: 1,
                    collision_offset: 8,
                    clears_collision_type: true,
                }
            ),
            "ASM Tar_Monster.asm loc_32698 decrements d3, clears collision_type(a3), then subtracts 4 before indexing off_32656"
        );

        assert_eq!(
            tar_monster_projectile_dispatch_seed_loc_32698(0x0004, 0),
            TarMonsterProjectileCollisionBudgetLoc32698::DeleteCurrentObject,
            "ASM Tar_Monster.asm loc_32698 branches to loc_32652 when the post-decrement d3 word is negative"
        );
    }

    #[test]
    fn tar_monster_projectile_reflect_x_loc_326aa_halves_and_negates_x_velocity() {
        assert_eq!(
            tar_monster_projectile_reflect_x_loc_326AA(0x0002_0000),
            -0x0001_0000,
            "ASM Tar_Monster.asm loc_326AA reflects a positive x_vel by ASR.L #1 followed by NEG.L"
        );

        assert_eq!(
            tar_monster_projectile_reflect_x_loc_326AA(-0x0003_0000),
            0x0001_8000,
            "ASM Tar_Monster.asm loc_326AA uses arithmetic shift before negation, preserving signed fixed-point halves"
        );
    }

    #[test]
    fn tar_monster_projectile_reflect_y_loc_326b8_halves_and_negates_y_velocity() {
        assert_eq!(
            tar_monster_projectile_reflect_y_loc_326B8(0x0004_0000),
            -0x0002_0000,
            "ASM Tar_Monster.asm loc_326B8 reflects a positive y_vel by ASR.L #1 followed by NEG.L"
        );

        assert_eq!(
            tar_monster_projectile_reflect_y_loc_326B8(-0x0001_8000),
            0x0000_C000,
            "ASM Tar_Monster.asm loc_326B8 preserves signed half-pixel velocity before the shared y_vel writeback"
        );
    }

    #[test]
    fn tar_monster_projectile_write_y_velocity_loc_326c0_stores_prepared_d7() {
        assert_eq!(
            tar_monster_projectile_write_y_velocity_loc_326C0(-0x0002_0000),
            -0x0002_0000,
            "ASM Tar_Monster.asm loc_326C0 writes the prepared d7 long directly to y_vel(a3)"
        );

        assert_eq!(
            tar_monster_projectile_write_y_velocity_loc_326C0(0x0000_C000),
            0x0000_C000,
            "ASM Tar_Monster.asm loc_326C0 has no extra clamping or sign change after loc_326B8"
        );
    }

    #[test]
    fn tar_monster_projectile_negated_velocity_swap_loc_326c6_cross_writes_negated_longs() {
        assert_eq!(
            tar_monster_projectile_negated_velocity_swap_loc_326C6(0x0002_0000, -0x0003_0000),
            TarMonsterProjectileVelocityPair {
                x_velocity: 0x0003_0000,
                y_velocity: -0x0002_0000,
            },
            "ASM Tar_Monster.asm loc_326C6 negates x_vel/y_vel as longs, then writes -y to x_vel and -x to y_vel"
        );

        assert_eq!(
            tar_monster_projectile_negated_velocity_swap_loc_326C6(i32::MIN, 0x0001_0000),
            TarMonsterProjectileVelocityPair {
                x_velocity: -0x0001_0000,
                y_velocity: i32::MIN,
            },
            "ASM Tar_Monster.asm loc_326C6 uses 68k NEG.L wrapping behaviour for the minimum negative long"
        );
    }

    #[test]
    fn tar_monster_projectile_velocity_swap_loc_326dc_cross_writes_without_negation() {
        assert_eq!(
            tar_monster_projectile_velocity_swap_loc_326DC(0x0002_0000, -0x0003_0000),
            TarMonsterProjectileVelocityPair {
                x_velocity: -0x0003_0000,
                y_velocity: 0x0002_0000,
            },
            "ASM Tar_Monster.asm loc_326DC writes old y_vel to x_vel and old x_vel to y_vel without negation"
        );

        assert_eq!(
            tar_monster_projectile_velocity_swap_loc_326DC(i32::MIN, 0x0001_0000),
            TarMonsterProjectileVelocityPair {
                x_velocity: 0x0001_0000,
                y_velocity: i32::MIN,
            },
            "ASM Tar_Monster.asm loc_326DC only moves long words, so the minimum negative value is preserved"
        );
    }

    #[test]
    fn tar_monster_projectile_collision_loc_32682_deletes_on_negative_or_spent_budget() {
        assert_eq!(
            tar_monster_projectile_collision_loc_32682(-1, 2, true),
            TarMonsterProjectileCollisionLoc32682 {
                route: TarMonsterProjectileCollisionRouteLoc32682::DeleteCurrentObject,
            },
            "ASM Tar_Monster.asm loc_32682 branches to loc_32652 when collision_type(a3) is negative"
        );

        assert_eq!(
            tar_monster_projectile_collision_loc_32682(4, 0, false),
            TarMonsterProjectileCollisionLoc32682 {
                route: TarMonsterProjectileCollisionRouteLoc32682::DeleteCurrentObject,
            },
            "ASM Tar_Monster.asm loc_32682 decrements d3 before clearing collision_type, so an already-spent collision budget deletes instead of dispatching"
        );
    }

    #[test]
    fn tar_monster_projectile_collision_loc_32682_returns_or_dispatches_positive_collisions() {
        assert_eq!(
            tar_monster_projectile_collision_loc_32682(0, 2, true),
            TarMonsterProjectileCollisionLoc32682 {
                route: TarMonsterProjectileCollisionRouteLoc32682::ReturnOnly,
            },
            "ASM Tar_Monster.asm loc_32682 returns immediately when collision_type(a3) is zero"
        );

        assert_eq!(
            tar_monster_projectile_collision_loc_32682(4, 2, true),
            TarMonsterProjectileCollisionLoc32682 {
                route: TarMonsterProjectileCollisionRouteLoc32682::Dispatch(
                    TarMonsterProjectileCollisionDispatchLoc32682 {
                        sprite_frame_link: Some("LnkTo_unk_C7836"),
                        remaining_collision_budget: 1,
                        clears_collision_type: true,
                        target: TarMonsterProjectileCollisionTargetLoc32682::Loc326AA,
                    }
                ),
            },
            "ASM Tar_Monster.asm loc_32682 swaps to LnkTo_unk_C7836 when d1 is non-zero, clears collision_type, and dispatches collision id $04 through off_32656[0]"
        );

        assert_eq!(
            tar_monster_projectile_collision_loc_32682(0x002C, 1, false),
            TarMonsterProjectileCollisionLoc32682 {
                route: TarMonsterProjectileCollisionRouteLoc32682::Dispatch(
                    TarMonsterProjectileCollisionDispatchLoc32682 {
                        sprite_frame_link: None,
                        remaining_collision_budget: 0,
                        clears_collision_type: true,
                        target: TarMonsterProjectileCollisionTargetLoc32682::Loc32652,
                    }
                ),
            },
            "ASM Tar_Monster.asm loc_32682 subtracts 4 before indexing off_32656, so collision id $2C reaches the table's loc_32652 delete entry"
        );

        for (collision_type, target) in [
            (
                0x000C,
                TarMonsterProjectileCollisionTargetLoc32682::Loc326B8,
            ),
            (
                0x0010,
                TarMonsterProjectileCollisionTargetLoc32682::Loc326B8,
            ),
            (
                0x0028,
                TarMonsterProjectileCollisionTargetLoc32682::Loc326B8,
            ),
            (
                0x0014,
                TarMonsterProjectileCollisionTargetLoc32682::Loc326C6,
            ),
            (
                0x0018,
                TarMonsterProjectileCollisionTargetLoc32682::Loc326DC,
            ),
            (
                0x001C,
                TarMonsterProjectileCollisionTargetLoc32682::Loc326DC,
            ),
        ] {
            assert_eq!(
                tar_monster_projectile_collision_loc_32682(collision_type, 2, false),
                TarMonsterProjectileCollisionLoc32682 {
                    route: TarMonsterProjectileCollisionRouteLoc32682::Dispatch(
                        TarMonsterProjectileCollisionDispatchLoc32682 {
                            sprite_frame_link: None,
                            remaining_collision_budget: 1,
                            clears_collision_type: true,
                            target,
                        }
                    ),
                },
                "ASM Tar_Monster.asm loc_32682 maps collision id ${collision_type:04X} through off_32656 after subtracting 4"
            );
        }
    }

    #[test]
    fn tar_monster_projectile_loop_loc_3263c_applies_gravity_and_continues_while_byte_19_clear() {
        assert_eq!(
            tar_monster_projectile_loop_loc_3263C(0x0001_0000, 0, 2, true, false),
            TarMonsterProjectileLoopLoc3263C {
                hibernates_one_frame: true,
                y_velocity: 0x0001_1000,
                collision: TarMonsterProjectileCollisionLoc32682 {
                    route: TarMonsterProjectileCollisionRouteLoc32682::ReturnOnly,
                },
                route: TarMonsterProjectileLoopRouteLoc3263C::ContinueLoop,
            },
            "ASM Tar_Monster.asm loc_3263C hibernates, adds `$1000` to y_vel(a3), calls loc_32682, and loops while byte `$19(a3)` is clear"
        );
    }

    #[test]
    fn tar_monster_projectile_loop_loc_3263c_deletes_on_byte_19_or_delete_collision() {
        assert_eq!(
            tar_monster_projectile_loop_loc_3263C(0, 0, 2, false, true).route,
            TarMonsterProjectileLoopRouteLoc3263C::DeleteCurrentObject,
            "ASM Tar_Monster.asm loc_3263C falls through to loc_32652 once byte `$19(a3)` becomes non-zero"
        );

        assert_eq!(
            tar_monster_projectile_loop_loc_3263C(0, 0x002C, 1, false, false).route,
            TarMonsterProjectileLoopRouteLoc3263C::DeleteCurrentObject,
            "ASM Tar_Monster.asm loc_3263C does not repeat when loc_32682 dispatches collision id $2C to loc_32652"
        );
    }

    #[test]
    fn tar_monster_shot_motion_loc_324d0_sets_velocity_and_mirrors_right_facing_spawn() {
        assert_eq!(
            tar_monster_shot_motion_loc_324D0(0x0107_0000, Direction::Left),
            TarMonsterShotMotionLoc324D0 {
                x_pos: 0x0107_0000,
                x_velocity: -TAR_MONSTER_GLOB_SPEED_X,
            },
            "ASM Tar_Monster.asm loc_324D0 keeps the initial -4 px/frame shot when copied x_direction is zero"
        );

        assert_eq!(
            tar_monster_shot_motion_loc_324D0(0x0107_0000, Direction::Right),
            TarMonsterShotMotionLoc324D0 {
                x_pos: 0x0139_0000,
                x_velocity: TAR_MONSTER_GLOB_SPEED_X,
            },
            "ASM Tar_Monster.asm loc_324DC negates x_vel and adds $32 to x when copied x_direction is non-zero"
        );
    }

    #[test]
    fn tar_monster_shot_wait_loc_324e6_loops_until_collision_or_byte_19() {
        assert_eq!(
            tar_monster_shot_wait_loc_324E6(0, false),
            TarMonsterShotWaitLoc324E6 {
                hibernates_one_frame: true,
                route: TarMonsterShotWaitRouteLoc324E6::ContinueLoop,
            },
            "ASM Tar_Monster.asm loc_324E6 loops only after the one-frame hibernate when collision_type is zero and byte `$19(a3)` is clear"
        );

        assert_eq!(
            tar_monster_shot_wait_loc_324E6(0x000C, false),
            TarMonsterShotWaitLoc324E6 {
                hibernates_one_frame: true,
                route: TarMonsterShotWaitRouteLoc324E6::BurstCleanup,
            },
            "ASM Tar_Monster.asm loc_324E6 exits the wait loop on any non-zero collision_type"
        );

        assert_eq!(
            tar_monster_shot_wait_loc_324E6(0, true).route,
            TarMonsterShotWaitRouteLoc324E6::BurstCleanup,
            "ASM Tar_Monster.asm loc_324E6 also exits when byte `$19(a3)` is non-zero"
        );
    }

    #[test]
    fn tar_monster_burst_cleanup_loc_324f8_spawns_five_fixed_velocity_children() {
        assert_eq!(
            tar_monster_burst_cleanup_loc_324F8(),
            TarMonsterBurstCleanupLoc324F8 {
                hides_original_shot: true,
                clears_original_x_velocity: true,
                clears_original_y_velocity: true,
                child_count: 5,
                child_velocities: [
                    (0x1_0000, -0x1_0000),
                    (-0x1_0000, -0x1_0000),
                    (0x0_8000, -0x2_0000),
                    (-0x0_8000, -0x2_0000),
                    (0x0_2000, -0x1_C000),
                ],
                hibernates_before_delete: true,
            },
            "ASM Tar_Monster.asm loc_324F8 hides and stops the original glob before allocating five fixed-velocity burst children"
        );

        let mut projectiles = Vec::new();
        spawn_tar_monster_retaliation(&mut projectiles, 0x0120_0000, 0x0070_0000);

        assert_eq!(
            projectiles
                .iter()
                .map(|projectile| (projectile.x_velocity, projectile.y_velocity))
                .collect::<Vec<_>>(),
            TAR_MONSTER_BURST_BLOB_VELOCITIES_LOC_324F8.to_vec(),
            "ASM Tar_Monster.asm loc_324F8 velocity table should drive the native retaliation burst"
        );
        for (projectile, (x_velocity, y_velocity)) in projectiles
            .iter()
            .zip(TAR_MONSTER_BURST_BLOB_VELOCITIES_LOC_324F8)
        {
            assert_eq!(
                *projectile,
                EnemyProjectile {
                    x_pos: 0x0120_0000,
                    y_pos: 0x0070_0000,
                    x_velocity,
                    y_velocity,
                    lifetime: 60,
                    gravity: 0x1000,
                    friendly: false,
                    homing_accel: 0,
                    speed_cap: 0,
                    wall_bounce: false,
                    kind: EnemyProjectileKind::TarMonsterBurstChild,
                    animation_elapsed_frames: 1,
                    tar_monster_collision_response_budget: 2,
                    ..Default::default()
                },
                "ASM Tar_Monster.asm loc_325E6/loc_3261C should initialise every burst child field from the caller-prepared source position"
            );
        }
    }

    #[test]
    fn tar_monster_burst_tail_loc_32534_spawns_last_three_children_then_deletes() {
        assert_eq!(
            tar_monster_burst_tail_loc_32534(),
            TarMonsterBurstTailLoc32534 {
                child_count: 3,
                child_velocities: [
                    (0x0_8000, -0x2_0000),
                    (-0x0_8000, -0x2_0000),
                    (0x0_2000, -0x1_C000),
                ],
                hibernates_before_delete: true,
                deletes_original_shot: true,
            },
            "ASM Tar_Monster.asm loc_32534 allocates the final three burst children, hibernates once, then deletes the original glob"
        );
    }

    #[test]
    fn tar_monster_child_init_loc_32200_copies_position_and_anim_meta() {
        // ASM loc_32200 stores the parent coordinates onto the linked child object
        // and applies the animation defaults (`status_flag_13`, animation phase 2, vram=$100).
        let parent = tar_monster_init_loc_321e4(0x0100, 0x0040, 1, 1);
        let child = tar_monster_child_init_loc_32200(parent, 0x0F00);

        assert_eq!(
            child,
            TarMonsterChildInitLoc32200 {
                sprite_frame_link: 0x0F00,
                object_meta: 5,
                priority: 0,
                x_pos: 0x0100 << 16,
                y_pos: 0x0040_u32.saturating_add(16) << 16,
                status_flag_13: true,
                animation_phase: 2,
                vram_tile: 0x0100,
            }
        );
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
    fn tar_monster_sequence_queues_appear_sound_loc_322c0() {
        let mut enemy = tar_monster_test_enemy();
        enemy.tar_monster_state = TarMonsterState::Submerged;
        enemy.wait_timer = 0;
        let mut projectiles = Vec::new();
        let mut sound_effects = Vec::new();

        step_tar_monster_with_sounds(&mut enemy, &mut projectiles, &mut sound_effects);

        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::TarMonsterAppears],
            "ASM Tar_Monster.asm loc_322BC/loc_322C0 plays sfx_Tar_Monster_appears at emerge start",
        );
        assert_eq!(sound_effects[0].sound_id(), 0x54);
    }

    #[test]
    fn tar_monster_attack_queues_shoot_sound_loc_32468() {
        let mut enemy = tar_monster_test_enemy();
        let mut projectiles = Vec::new();
        let mut sound_effects = Vec::new();

        for _ in 0..9 {
            step_tar_monster_with_sounds(&mut enemy, &mut projectiles, &mut sound_effects);
        }

        assert_eq!(projectiles.len(), 1);
        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::TarMonsterShoots],
            "ASM Tar_Monster.asm loc_32468 plays sfx_Tar_Monster_shoots before loc_3247E initialises the glob",
        );
        assert_eq!(sound_effects[0].sound_id(), 0x5E);
    }

    #[test]
    fn tar_monster_sequence_queues_disappear_sound_loc_322da() {
        let mut enemy = tar_monster_test_enemy();
        enemy.tar_monster_state = TarMonsterState::Visible;
        enemy.wait_timer = 0;
        enemy.dragon_script_cmd = 28;
        let mut projectiles = Vec::new();
        let mut sound_effects = Vec::new();

        step_tar_monster_with_sounds(&mut enemy, &mut projectiles, &mut sound_effects);

        assert_eq!(enemy.tar_monster_state, TarMonsterState::Submerging);
        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::TarMonsterDisappears],
            "ASM Tar_Monster.asm loc_322DA plays sfx_Tar_Monster_disappears before the submerge animation",
        );
        assert_eq!(sound_effects[0].sound_id(), 0x53);
    }

    #[test]
    fn tar_monster_primary_gfx_visibility_follows_loc_32308_32310_322da() {
        let mut enemy = tar_monster_test_enemy();
        enemy.tar_monster_state = TarMonsterState::Visible;
        enemy.wait_timer = 0;
        enemy.dragon_script_cmd = 2;
        let mut projectiles = Vec::new();

        step_tar_monster(&mut enemy, &mut projectiles);
        let primary = enemy
            .tar_monster_primary_snapshot()
            .expect("ASM loc_32308 toggles primary `$13(a2)` visible");
        assert_eq!(primary.sprite_animation_phase, Some(12));
        assert_eq!(primary.hit_points, Some(0));
        assert_eq!((primary.x_pos, primary.y_pos), (enemy.x_pos, enemy.y_pos));

        enemy.wait_timer = 0;
        enemy.dragon_script_cmd = 22;
        step_tar_monster(&mut enemy, &mut projectiles);
        assert_eq!(
            enemy
                .tar_monster_primary_snapshot()
                .expect("primary GfxObject remains allocated")
                .sprite_animation_phase,
            None,
            "ASM loc_32310 clears primary `$13(a2)` before the attack"
        );

        enemy.dragon_script_variant = TAR_MONSTER_PRIMARY_VISIBLE_LOC_32308;
        enemy.wait_timer = 0;
        enemy.dragon_script_cmd = 28;
        step_tar_monster(&mut enemy, &mut projectiles);
        assert_eq!(
            enemy
                .tar_monster_primary_snapshot()
                .expect("primary GfxObject remains allocated")
                .sprite_animation_phase,
            None,
            "ASM loc_322DA clears primary `$13(a2)` before submerging"
        );
    }

    #[test]
    fn tar_monster_attack_glob_uses_loc_3247e_and_loc_324d0_spawn_offsets() {
        let mut right_facing = tar_monster_test_enemy();
        right_facing.x_pos = 0x0120_0000;
        right_facing.y_pos = 0x0080_0000;
        right_facing.facing = Direction::Right;
        let mut projectiles = Vec::new();

        for _ in 0..9 {
            step_tar_monster(&mut right_facing, &mut projectiles);
        }

        assert_eq!(projectiles.len(), 1);
        assert_eq!(
            projectiles[0].x_pos, 0x0139_0000,
            "ASM Tar_Monster.asm loc_324D0 mirrors the loc_3247E `x-$19` setup by adding $32 when x_direction is non-zero"
        );
        assert_eq!(projectiles[0].y_pos, 0x006E_0000);
        assert_eq!(
            projectiles[0].x_velocity, 0x4_0000,
            "ASM Tar_Monster.asm loc_324D0 sets +4px/frame when x_direction is non-zero"
        );
        assert_eq!(
            projectiles[0],
            EnemyProjectile {
                x_pos: 0x0139_0000,
                y_pos: 0x006E_0000,
                x_velocity: TAR_MONSTER_GLOB_SPEED_X,
                y_velocity: 0,
                lifetime: 120,
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                kind: EnemyProjectileKind::TarMonsterStraightGlob,
                tar_monster_collision_response_budget: 2,
                ..Default::default()
            },
            "ASM Tar_Monster.asm loc_3247E/loc_3261C should initialise every straight glob field"
        );

        let mut left_facing = tar_monster_test_enemy();
        left_facing.x_pos = 0x0120_0000;
        left_facing.y_pos = 0x0080_0000;
        left_facing.facing = Direction::Left;
        let mut projectiles = Vec::new();

        for _ in 0..9 {
            step_tar_monster(&mut left_facing, &mut projectiles);
        }

        assert_eq!(projectiles.len(), 1);
        assert_eq!(projectiles[0].x_pos, 0x0107_0000);
        assert_eq!(projectiles[0].y_pos, 0x006E_0000);
        assert_eq!(
            projectiles[0].x_velocity, -0x4_0000,
            "ASM Tar_Monster.asm loc_324D0 sets -4px/frame when x_direction is zero"
        );
    }

    #[test]
    fn tar_monster_attack_runs_twenty_seven_frames_per_loc_3231c() {
        let mut enemy = tar_monster_test_enemy();
        enemy.dragon_script_cmd = 24;
        let mut projectiles = Vec::new();

        for _ in 0..27 {
            step_tar_monster(&mut enemy, &mut projectiles);
        }

        assert_eq!(enemy.tar_monster_state, TarMonsterState::Visible);
        assert_eq!(enemy.wait_timer, 0x1E + 1);

        for _ in 0..=0x1E {
            step_tar_monster(&mut enemy, &mut projectiles);
        }

        assert_eq!(enemy.tar_monster_state, TarMonsterState::Submerging);
    }
}
