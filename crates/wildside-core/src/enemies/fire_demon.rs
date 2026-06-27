use super::*;

pub(crate) const FIRE_DEMON_COLID_HURT: i16 = colid::HURT as i16;
pub(crate) const FIRE_DEMON_COLID_KIDRIGHT: i16 = colid::KIDRIGHT as i16;
pub(crate) const FIRE_DEMON_COLID_KIDLEFT: i16 = colid::KIDLEFT as i16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FireDemonInitAnimation {
    pub(crate) collision_routine: &'static str,
    pub(crate) animation_script: &'static str,
    pub(crate) saved_animation_script: &'static str,
}

pub(crate) fn fire_demon_hp5_animation_loc_3a444(current_hp: u8) -> Option<FireDemonInitAnimation> {
    // ASM Fire_Demon.asm loc_3A444: set `a0` to the Fire Demon collision routine,
    // and keep HP5 on the `stru_3A334` branch while saving the same script in
    // `$62(a5)` for later collision recovery.
    (current_hp == 5).then_some(FireDemonInitAnimation {
        collision_routine: "fire_demon_collision",
        animation_script: "stru_3A334",
        saved_animation_script: "stru_3A334",
    })
}

pub(crate) fn fire_demon_non_hp5_animation_loc_3a464(
    current_hp: u8,
) -> Option<FireDemonInitAnimation> {
    // ASM Fire_Demon.asm loc_3A464: the non-HP5 branch initialises `stru_3A31A`
    // and stores the same animation pointer in `$62(a5)`.
    (current_hp != 5).then_some(FireDemonInitAnimation {
        collision_routine: "fire_demon_collision",
        animation_script: "stru_3A31A",
        saved_animation_script: "stru_3A31A",
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FireDemonInitWalkDispatch {
    LeftSharedWalker,
    RightSharedWalker,
}

pub(crate) fn fire_demon_init_walk_dispatch_loc_3a476(
    facing: Direction,
) -> FireDemonInitWalkDispatch {
    // ASM Fire_Demon.asm loc_3A476: `tst.b x_direction(a3)` dispatches clear
    // raw direction into the left shared walker and set raw direction into the
    // right shared walker. Native `Direction` is the decoded raw flag.
    match facing {
        Direction::Left => FireDemonInitWalkDispatch::LeftSharedWalker,
        Direction::Right => FireDemonInitWalkDispatch::RightSharedWalker,
    }
}

const FIRE_DEMON_PALETTE_TARGET_BYTE_OFFSETS_LOC_3A48E: [u16; 3] = [0x22, 0x42, 0x52];

pub(crate) fn fire_demon_palette_target_loc_3a48e(vram_slot: u8) -> Option<u16> {
    // ASM Fire_Demon.asm loc_3A48E: `$3E(a4)` indexes dword_3A482,
    // selecting Palette_Buffer+$22/+$42/+$52 as the palette pulse target.
    FIRE_DEMON_PALETTE_TARGET_BYTE_OFFSETS_LOC_3A48E
        .get(usize::from(vram_slot))
        .copied()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FireDemonPalettePulseSeedLoc3A4B8 {
    pub(crate) palette_buffer_byte_offset: u16,
    pub(crate) data_index_byte_offset: u16,
}

const FIRE_DEMON_PALETTE_DATA_INDEX_BASE_LOC_3A4B8: u16 = 0x0634;
const FIRE_DEMON_PALETTE_DATA_INDEX_STEP_LOC_3A4B4: u16 = 0x000C;

pub(crate) fn fire_demon_palette_base_pulse_seed_loc_3a4b8(
    vram_slot: u8,
) -> Option<FireDemonPalettePulseSeedLoc3A4B8> {
    // ASM Fire_Demon.asm loc_3A4B8: `lea (Data_Index).l,a4; move.l (a4,d7.w),a2`
    // materialises the base palette animation data pointer after loc_3A48E has
    // selected the palette target.
    Some(FireDemonPalettePulseSeedLoc3A4B8 {
        palette_buffer_byte_offset: fire_demon_palette_target_loc_3a48e(vram_slot)?,
        data_index_byte_offset: FIRE_DEMON_PALETTE_DATA_INDEX_BASE_LOC_3A4B8,
    })
}

pub(crate) fn fire_demon_palette_pulse_seed_loc_3a4b4(
    vram_slot: u8,
    enemy_level: u8,
) -> Option<FireDemonPalettePulseSeedLoc3A4B8> {
    let mut seed = fire_demon_palette_base_pulse_seed_loc_3a4b8(vram_slot)?;
    // ASM Fire_Demon.asm loc_3A4B4: raw enemy_level 1 reaches one `$C` add;
    // levels above 1 execute a pre-add and then fall through here for two steps.
    let data_index_steps = match enemy_level {
        0 => 0,
        1 => 1,
        _ => 2,
    };
    seed.data_index_byte_offset += data_index_steps * FIRE_DEMON_PALETTE_DATA_INDEX_STEP_LOC_3A4B4;
    Some(seed)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum FireDemonPaletteLoopHeadLoc3A4C2 {
    DeleteCurrentObject,
    ContinueAfterHibernate { hibernate_frames: u16 },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_palette_loop_head_loc_3a4c2(
    fire_demon_count: u8,
) -> FireDemonPaletteLoopHeadLoc3A4C2 {
    // ASM Fire_Demon.asm loc_3A4C2: hibernate for three frames, then re-test
    // the global Fire_Demon byte before resetting a1/a0 for the first palette
    // copy. A clear byte deletes this helper object instead.
    if fire_demon_count == 0 {
        // ASM Fire_Demon.asm loc_3A53E: the clear liveness byte jumps straight
        // to `j_Delete_CurrentObject`.
        FireDemonPaletteLoopHeadLoc3A4C2::DeleteCurrentObject
    } else {
        FireDemonPaletteLoopHeadLoc3A4C2::ContinueAfterHibernate {
            hibernate_frames: 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct FireDemonPaletteCopyStep {
    pub(crate) source_byte_offset: u16,
    pub(crate) copied_words: u8,
    pub(crate) next_hibernate_frames: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_palette_first_copy_loc_3a4d6() -> FireDemonPaletteCopyStep {
    // ASM Fire_Demon.asm loc_3A4D6: after loc_3A4C2 resets a0 to the selected
    // Data_Index palette script, `moveq #6,d0` plus `dbf` copies seven words
    // into the target palette, then sleeps another three frames.
    FireDemonPaletteCopyStep {
        source_byte_offset: 0,
        copied_words: 7,
        next_hibernate_frames: 3,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_palette_middle_copy_loc_3a4f6() -> FireDemonPaletteCopyStep {
    // ASM Fire_Demon.asm loc_3A4F6: the preceding `addi.l #$E,a0` selects the
    // middle palette frame; the same `moveq #6,d0`/`dbf` loop copies seven words
    // before the helper hibernates for another three frames.
    FireDemonPaletteCopyStep {
        source_byte_offset: 0x000E,
        copied_words: 7,
        next_hibernate_frames: 3,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_palette_peak_copy_loc_3a516() -> FireDemonPaletteCopyStep {
    // ASM Fire_Demon.asm loc_3A516: the preceding `addi.l #$1C,a0` selects the
    // peak palette frame; `moveq #6,d0` plus `dbf` copies seven words before
    // the next three-frame hibernate.
    FireDemonPaletteCopyStep {
        source_byte_offset: 0x001C,
        copied_words: 7,
        next_hibernate_frames: 3,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn fire_demon_palette_return_copy_loc_3a536() -> FireDemonPaletteCopyStep {
    // ASM Fire_Demon.asm loc_3A536: after the post-peak hibernate/liveness
    // check, `addi.l #$E,a0` returns to the middle palette frame, copies seven
    // words, then branches to loc_3A4C2 for the next hibernate.
    FireDemonPaletteCopyStep {
        source_byte_offset: 0x000E,
        copied_words: 7,
        next_hibernate_frames: 3,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FireDemonFireTrailWaitNextLoc3A6CA {
    ContinueLoop,
    DeleteFlagSet { decrements_fire_trail_count: bool },
    DeleteTimerExpired { decrements_fire_trail_count: bool },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FireDemonFireTrailWaitStepLoc3A6CA {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) remaining_timer: u16,
    pub(crate) next: FireDemonFireTrailWaitNextLoc3A6CA,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FireDemonFireTrailDeleteTailLoc3A6DA {
    pub(crate) next_fire_trail_count: u16,
    pub(crate) deletes_current_object: bool,
}

pub(crate) fn fire_demon_fire_trail_delete_tail_loc_3a6da(
    fire_trail_count: u16,
) -> FireDemonFireTrailDeleteTailLoc3A6DA {
    // ASM Fire_Demon.asm loc_3A6DA: `subq.w #1,(Number_of_Fire_Trails).w`,
    // then jump directly to `j_Delete_CurrentObject`.
    FireDemonFireTrailDeleteTailLoc3A6DA {
        next_fire_trail_count: fire_trail_count.wrapping_sub(1),
        deletes_current_object: true,
    }
}

pub(crate) fn fire_demon_fire_trail_wait_loop_tick_loc_3a6ca(
    status_flag_19_set: bool,
    timer_d3: u16,
) -> FireDemonFireTrailWaitStepLoc3A6CA {
    // ASM Fire_Demon.asm loc_3A6CA: after one hibernate, nonzero `$19(a3)`
    // jumps to deletion. Otherwise `subq.w #1,d3` loops while the word result
    // is nonzero; only a zero result falls through to deletion.
    if status_flag_19_set {
        return FireDemonFireTrailWaitStepLoc3A6CA {
            hibernates_one_frame: true,
            remaining_timer: timer_d3,
            next: FireDemonFireTrailWaitNextLoc3A6CA::DeleteFlagSet {
                decrements_fire_trail_count: true,
            },
        };
    }

    let remaining_timer = timer_d3.wrapping_sub(1);
    let next = if remaining_timer == 0 {
        FireDemonFireTrailWaitNextLoc3A6CA::DeleteTimerExpired {
            decrements_fire_trail_count: true,
        }
    } else {
        FireDemonFireTrailWaitNextLoc3A6CA::ContinueLoop
    };

    FireDemonFireTrailWaitStepLoc3A6CA {
        hibernates_one_frame: true,
        remaining_timer,
        next,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FireDemonResumeSharedWalkerSourceLoc3A586 {
    NoCollision,
    UnknownPositive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FireDemonCollisionRouteLoc3A586 {
    ResumeSharedWalker {
        next_collision_type: u16,
        source: FireDemonResumeSharedWalkerSourceLoc3A586,
    },
    Death {
        death_animation: &'static str,
        decrements_fire_demon_count: bool,
    },
    BerzerkerSideGate,
    Hurt {
        remaining_hp: u16,
        next_collision_type: u16,
        damage_animation: &'static str,
        restored_animation: &'static str,
    },
}

pub(crate) fn fire_demon_death_loc_3a5a6() -> FireDemonCollisionRouteLoc3A586 {
    // ASM Fire_Demon.asm loc_3A5A6: initialise `stru_3A378`, hibernate until
    // the animation finishes, decrement `(Fire_Demon).w`, then branch to delete.
    FireDemonCollisionRouteLoc3A586::Death {
        death_animation: "stru_3A378",
        decrements_fire_demon_count: true,
    }
}

pub(crate) fn fire_demon_hurt_recovery_loc_3a5be(
    remaining_hp: u16,
) -> FireDemonCollisionRouteLoc3A586 {
    // ASM Fire_Demon.asm loc_3A5BE: play `stru_3A34E`, hibernate until it
    // finishes, then restore the saved animation pointer from `$62(a5)`.
    FireDemonCollisionRouteLoc3A586::Hurt {
        remaining_hp,
        next_collision_type: 0,
        damage_animation: "stru_3A34E",
        restored_animation: "saved_fire_demon_walk_animation",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FireDemonResumeSharedWalkerBranchLoc3A5D4 {
    LeftSharedWalker,
    RightSharedWalker,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FireDemonResumeSharedWalkerLoc3A5D4 {
    pub(crate) next_collision_type: u16,
    pub(crate) branch: FireDemonResumeSharedWalkerBranchLoc3A5D4,
}

pub(crate) fn fire_demon_resume_shared_walker_loc_3a5d4(
    facing: Direction,
) -> FireDemonResumeSharedWalkerLoc3A5D4 {
    // ASM Fire_Demon.asm loc_3A5D4: clear `collision_type(a3)`, then `tst.b
    // x_direction(a3)` branches clear direction to `loc_3C0FA` and set direction
    // to `loc_3C04E`.
    let branch = match facing {
        Direction::Left => FireDemonResumeSharedWalkerBranchLoc3A5D4::LeftSharedWalker,
        Direction::Right => FireDemonResumeSharedWalkerBranchLoc3A5D4::RightSharedWalker,
    };
    FireDemonResumeSharedWalkerLoc3A5D4 {
        next_collision_type: 0,
        branch,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FireDemonBerzerkerLaunchLoc3A5E4 {
    pub(crate) has_level_collision: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FireDemonBerzerkerSideGateLoc3A5E4 {
    ResumeSharedWalker(FireDemonResumeSharedWalkerLoc3A5D4),
    Launch(FireDemonBerzerkerLaunchLoc3A5E4),
}

pub(crate) fn fire_demon_berzerker_side_gate_loc_3a5e4(
    berzerker_charging: bool,
    facing: Direction,
) -> FireDemonBerzerkerSideGateLoc3A5E4 {
    // ASM Fire_Demon.asm loc_3A5E4: clear `(Berzerker_charging).w` branches
    // back to loc_3A5D4. A charging Berzerker disables terrain collision,
    // leaves Kid collision and movement enabled, freezes animation on
    // `LnkTo_unk_C773E`, zeroes X velocity, and launches upward at `-$48000`.
    if !berzerker_charging {
        return FireDemonBerzerkerSideGateLoc3A5E4::ResumeSharedWalker(
            fire_demon_resume_shared_walker_loc_3a5d4(facing),
        );
    }

    FireDemonBerzerkerSideGateLoc3A5E4::Launch(FireDemonBerzerkerLaunchLoc3A5E4 {
        has_level_collision: false,
        has_kid_collision: true,
        is_moved: true,
        is_animated: false,
        sprite_mapping: "LnkTo_unk_C773E-Data_Index",
        x_velocity: 0,
        y_velocity: -0x0004_8000,
    })
}

pub(crate) fn fire_demon_collision_dispatch_loc_3a586(
    collision_type: i16,
    current_hp: u16,
) -> FireDemonCollisionRouteLoc3A586 {
    // ASM Fire_Demon.asm loc_3A586: read `collision_type(a3)` into d4. Negative
    // values enter the death animation before the shared clear tail, zero and
    // unknown nonzero IDs fall through to that clear tail, side contacts branch
    // to the Berzerker gate, and `colid_hurt` decrements `$44(a3)`.
    if collision_type.is_negative() {
        return fire_demon_death_loc_3a5a6();
    }

    match collision_type {
        0 => FireDemonCollisionRouteLoc3A586::ResumeSharedWalker {
            next_collision_type: 0,
            source: FireDemonResumeSharedWalkerSourceLoc3A586::NoCollision,
        },
        FIRE_DEMON_COLID_KIDRIGHT | FIRE_DEMON_COLID_KIDLEFT => {
            FireDemonCollisionRouteLoc3A586::BerzerkerSideGate
        }
        FIRE_DEMON_COLID_HURT => {
            let remaining_hp = current_hp.wrapping_sub(1);
            if remaining_hp == 0 {
                fire_demon_death_loc_3a5a6()
            } else {
                fire_demon_hurt_recovery_loc_3a5be(remaining_hp)
            }
        }
        _ => FireDemonCollisionRouteLoc3A586::ResumeSharedWalker {
            next_collision_type: 0,
            source: FireDemonResumeSharedWalkerSourceLoc3A586::UnknownPositive,
        },
    }
}
