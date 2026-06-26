use super::*;

#[cfg_attr(not(test), allow(dead_code))]
const ROCK_TANK_OBJECT_ID: u16 = 0x000A;
#[cfg_attr(not(test), allow(dead_code))]
const ROCK_TANK_SHOOTING_OBJECT_ID: u16 = 0x000B;
#[cfg_attr(not(test), allow(dead_code))]
const ROCK_TANK_VISIBLE_OBJECT_META: u16 = 0x008A;
#[cfg_attr(not(test), allow(dead_code))]
const ROCK_TANK_SHOOTING_VISIBLE_OBJECT_META: u16 = 0x008B;
#[cfg_attr(not(test), allow(dead_code))]
const ROCK_TANK_ART_PALETTE_LINES: [u8; 3] = [1, 2, 2];
#[cfg_attr(not(test), allow(dead_code))]
const ROCK_TANK_ART_VRAM_TILE_ADDRESSES: [u16; 3] = [0x02FB, 0x0409, 0x0517];

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tank_shooting_art_slot_loc_3b56c(enemy_type_slots: &[u16]) -> u16 {
    // ASM Tank_Shooting.asm loc_3B56C: after setting `$13(a3)` and
    // object_meta `$0B`, compare the first two enemy-layout header slots
    // against both RockTank ids (`d5=$0B`, `d6=$0A`). A match in slot 0/1
    // leaves `d4` at 0/1; no match falls through with `d4=2`.
    let is_rock_tank_slot = |slot: &u16| {
        let enemy_id = slot & 0x0FFF;
        enemy_id == ROCK_TANK_SHOOTING_OBJECT_ID || enemy_id == ROCK_TANK_OBJECT_ID
    };

    if enemy_type_slots.first().is_some_and(is_rock_tank_slot) {
        0
    } else if enemy_type_slots.get(1).is_some_and(is_rock_tank_slot) {
        1
    } else {
        2
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TankInitVisibleStateLoc3B31E {
    pub(crate) status_flag_13: bool,
    pub(crate) object_meta: u16,
    pub(crate) art_slot_word: u16,
    pub(crate) palette_line: u8,
    pub(crate) vram_tile: u16,
    pub(crate) priority: u8,
    pub(crate) parent_timer_4a: u16,
    pub(crate) parent_timer_48: u16,
    pub(crate) current_hp: u8,
    pub(crate) walk_speed: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tank_init_visible_state_loc_3b31e(
    enemy_level: u8,
    enemy_type_slots: &[u16],
) -> TankInitVisibleStateLoc3B31E {
    // ASM Tank.asm loc_3B31E: use `d4` from the two-slot RockTank header
    // scan to index EnemyArt_PaletteLines, then double `d4` before reading
    // EnemyArt_VRAMTileAddresses and storing `$3E(a3)`. The tail sets
    // object_meta bit 7, clears priority, seeds `$4A/$48(a5)`, and writes
    // `enemy_level + 2` into current_hp.
    let slot_index = tank_shooting_art_slot_loc_3b56c(enemy_type_slots);
    let slot = usize::from(slot_index);
    let current_hp = spawn_hit_points(EnemyFamily::Tank, enemy_level);
    TankInitVisibleStateLoc3B31E {
        status_flag_13: true,
        object_meta: ROCK_TANK_VISIBLE_OBJECT_META,
        art_slot_word: slot_index.wrapping_mul(2),
        palette_line: ROCK_TANK_ART_PALETTE_LINES[slot],
        vram_tile: ROCK_TANK_ART_VRAM_TILE_ADDRESSES[slot],
        priority: 0,
        parent_timer_4a: 0x0020,
        parent_timer_48: 0x000B,
        current_hp,
        walk_speed: pacer_walk_speed(EnemyFamily::Tank, current_hp),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TankShootingInitVisibleStateLoc3B5A6 {
    pub(crate) status_flag_13: bool,
    pub(crate) object_meta: u16,
    pub(crate) art_slot_word: u16,
    pub(crate) palette_line: u8,
    pub(crate) vram_tile: u16,
    pub(crate) priority: u8,
    pub(crate) parent_timer_4a: u16,
    pub(crate) parent_timer_48: u16,
    pub(crate) current_hp: u8,
    pub(crate) walk_speed: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tank_shooting_init_visible_state_loc_3b5a6(
    enemy_level: u8,
    enemy_type_slots: &[u16],
) -> TankShootingInitVisibleStateLoc3B5A6 {
    // ASM Tank_Shooting.asm loc_3B5A6: use `d4` from loc_3B56C to index
    // EnemyArt_PaletteLines, then double `d4` before reading
    // EnemyArt_VRAMTileAddresses and storing `$3E(a3)`. The same tail sets
    // object_meta bit 7, clears priority, seeds `$4A/$48(a5)`, and writes
    // `enemy_level + 2` into current_hp.
    let slot_index = tank_shooting_art_slot_loc_3b56c(enemy_type_slots);
    let slot = usize::from(slot_index);
    let current_hp = spawn_hit_points(EnemyFamily::TankShooting, enemy_level);
    TankShootingInitVisibleStateLoc3B5A6 {
        status_flag_13: true,
        object_meta: ROCK_TANK_SHOOTING_VISIBLE_OBJECT_META,
        art_slot_word: slot_index.wrapping_mul(2),
        palette_line: ROCK_TANK_ART_PALETTE_LINES[slot],
        vram_tile: ROCK_TANK_ART_VRAM_TILE_ADDRESSES[slot],
        priority: 0,
        parent_timer_4a: 0x0020,
        parent_timer_48: 0x000B,
        current_hp,
        walk_speed: tank_shooting_walk_speed_loc_3b5f2(current_hp),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TankBulletLoopRouteLoc3B7BA {
    Continue,
    /// ASM Tank_Shooting.asm loc_3B7EC: shared `j_Delete_CurrentObject` tail.
    Delete,
}

pub(crate) fn tank_bullet_loop_route_loc_3b7ba(
    x_pos_word: u16,
    y_pos_word: u16,
    camera_x_word: u16,
    camera_y_word: u16,
    collision_type: u16,
) -> TankBulletLoopRouteLoc3B7BA {
    // ASM Tank_Shooting.asm loc_3B7BA: after one hibernated frame, compare the
    // signed camera-relative x/y words against [-$80, $1C0]. Any failed bounds
    // check, or any nonzero collision_type, branches to loc_3B7EC.
    let relative_x = x_pos_word.wrapping_sub(camera_x_word) as i16;
    if !(-0x80..=0x01C0).contains(&relative_x) {
        return TankBulletLoopRouteLoc3B7BA::Delete;
    }

    let relative_y = y_pos_word.wrapping_sub(camera_y_word) as i16;
    if !(-0x80..=0x01C0).contains(&relative_y) {
        return TankBulletLoopRouteLoc3B7BA::Delete;
    }

    if collision_type == 0 {
        TankBulletLoopRouteLoc3B7BA::Continue
    } else {
        TankBulletLoopRouteLoc3B7BA::Delete
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum TankResumeWalkBranchLoc3B4BA {
    LeftSharedWalker,
    RightSharedWalker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct TankResumeWalkLoc3B4BA {
    pub(crate) next_child_collision_type: u16,
    pub(crate) branch: TankResumeWalkBranchLoc3B4BA,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tank_resume_walk_loc_3b4ba(x_direction: u8) -> TankResumeWalkLoc3B4BA {
    // ASM Tank.asm loc_3B4BA: clear mouth child collision `$38(a2)`, then
    // branch by `x_direction(a3)`; zero enters loc_3C0FA, nonzero loc_3C04E.
    TankResumeWalkLoc3B4BA {
        next_child_collision_type: 0,
        branch: if x_direction == 0 {
            TankResumeWalkBranchLoc3B4BA::LeftSharedWalker
        } else {
            TankResumeWalkBranchLoc3B4BA::RightSharedWalker
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum TankShootingCollisionRouteLoc3B6C0 {
    Death,
    Damage,
    Walk,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tank_shooting_collision_route_loc_3b6c0(
    parent_collision_type: u16,
    child_collision_type: u16,
) -> TankShootingCollisionRouteLoc3B6C0 {
    // ASM Tank_Shooting.asm loc_3B6C0: test the parent collision word before
    // reading the turret child collision. Negative parent or child words route
    // to death, child `$1C`/`$2C` route onward to the HP decrement, and
    // zero or unrecognised child collisions resume the walker.
    if parent_collision_type & 0x8000 != 0 || child_collision_type & 0x8000 != 0 {
        return TankShootingCollisionRouteLoc3B6C0::Death;
    }

    if matches!(child_collision_type, 0x001C | 0x002C) {
        TankShootingCollisionRouteLoc3B6C0::Damage
    } else {
        TankShootingCollisionRouteLoc3B6C0::Walk
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum TankShootingDamageRouteLoc3B6DE {
    Death,
    Hurt { remaining_hp: u16 },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn tank_shooting_damage_after_hit_loc_3b6de(
    current_hp: u16,
) -> TankShootingDamageRouteLoc3B6DE {
    // ASM Tank_Shooting.asm loc_3B6DE: `subi.w #1,$44(a3)` updates the
    // shooting tank's current_hp word. A zero result falls through to death;
    // any non-zero result branches to the hurt/recoil path.
    let remaining_hp = current_hp.wrapping_sub(1);
    if remaining_hp == 0 {
        TankShootingDamageRouteLoc3B6DE::Death
    } else {
        TankShootingDamageRouteLoc3B6DE::Hurt { remaining_hp }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TankShootingDeathRouteLoc3B6E6 {
    SharedDeactivateLoc3C46E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TankShootingDeathLoc3B6E6 {
    pub(crate) child_visible: bool,
    pub(crate) child_has_kid_collision: bool,
    pub(crate) death_animation_script: &'static str,
    pub(crate) hibernates_until_animation_finished: bool,
    pub(crate) route: TankShootingDeathRouteLoc3B6E6,
}

pub(crate) fn tank_shooting_death_loc_3b6e6() -> TankShootingDeathLoc3B6E6 {
    // ASM Tank_Shooting.asm loc_3B6E6: hide turret child `$13(a2)`, set
    // turret kid-collision `$3D(a2)`, install `stru_3B518`, hibernate until
    // that animation finishes, then branch to loc_3C46E.
    TankShootingDeathLoc3B6E6 {
        child_visible: false,
        child_has_kid_collision: true,
        death_animation_script: "stru_3B518",
        hibernates_until_animation_finished: true,
        route: TankShootingDeathRouteLoc3B6E6::SharedDeactivateLoc3C46E,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    #[test]
    fn tank_live_step_queues_initial_drive_sound_once_per_shared_refcount() {
        let mut enemies = [
            ActiveEnemy {
                family: EnemyFamily::Tank,
                facing: Direction::Left,
                hit_points: 2,
                tank_mouth_animation_phase: spawn_tank_mouth_animation_phase(EnemyFamily::Tank),
                ..mini_hopping_skull_test_enemy()
            },
            ActiveEnemy {
                family: EnemyFamily::TankShooting,
                facing: Direction::Right,
                hit_points: 2,
                ..mini_hopping_skull_test_enemy()
            },
        ];
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();
        let mut sound_effects = Vec::new();

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
            0,
            false,
            None,
            &mut rng,
            &mut drilled_blocks,
            Some(&mut sound_effects),
        );

        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::TankDriving],
            "ASM Tank.asm loc_3B40A / Tank_Shooting.asm loc_3B690 should play `sfx_Tank_driving` only while the shared Rock Tank refcount was zero",
        );
        assert_eq!(sound_effects[0].sound_id(), 0x1F);
        assert!(
            enemies
                .iter()
                .all(|enemy| enemy.shared_walker_patrol_short_next),
            "ASM loc_3B40A/loc_3B6AE increments the shared Rock Tank drive refcount for every visible Tank after the init tail",
        );

        sound_effects.clear();
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
            false,
            None,
            &mut rng,
            &mut drilled_blocks,
            Some(&mut sound_effects),
        );

        assert!(
            sound_effects.is_empty(),
            "ASM loc_3B40A/loc_3B690 are visible-init tails, so an already refcounted live Tank must not restart the drive loop every frame",
        );
    }

    #[test]
    fn tank_live_step_queues_mouth_creaks_from_stru_3b27e_cadence() {
        let tank = ActiveEnemy {
            family: EnemyFamily::Tank,
            facing: Direction::Left,
            hit_points: 2,
            shared_walker_patrol_short_next: true,
            tank_mouth_animation_phase: TANK_MOUTH_OPEN_PHASE_LOC_3B27E as u8,
            ..mini_hopping_skull_test_enemy()
        };
        let mut enemies = [tank];
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();
        let mut sound_effects = Vec::new();

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
            0,
            false,
            None,
            &mut rng,
            &mut drilled_blocks,
            Some(&mut sound_effects),
        );

        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::TankMouthOpen],
            "ASM Tank.asm loc_3B41C should queue sfx_Tank_mouth_open=$3C on the `stru_3B27E` `$A` open tick",
        );
        assert_eq!(sound_effects[0].sound_id(), 0x3C);

        sound_effects.clear();
        enemies[0].tank_mouth_animation_phase = 79;
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
            false,
            None,
            &mut rng,
            &mut drilled_blocks,
            Some(&mut sound_effects),
        );
        assert!(
            sound_effects.is_empty(),
            "the final `stru_3B27E` closed frame is timer 5, so loc_3B446 must not play `$42` until wrap",
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
            2,
            false,
            None,
            &mut rng,
            &mut drilled_blocks,
            Some(&mut sound_effects),
        );

        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::TankMouthClosed],
            "ASM Tank.asm loc_3B446 should queue sfx_Tank_mouth_closed=$42 only when the `stru_3B27E` loop returns to `$37`",
        );
        assert_eq!(sound_effects[0].sound_id(), 0x42);
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
    fn tank_init_visible_state_loc_3b31e_uses_selected_art_slot() {
        let state = tank_init_visible_state_loc_3b31e(1, &[0x0022, 0x400B, 0x001C]);

        assert_eq!(
            state,
            TankInitVisibleStateLoc3B31E {
                status_flag_13: true,
                object_meta: 0x008A,
                art_slot_word: 2,
                palette_line: 2,
                vram_tile: 0x0409,
                priority: 0,
                parent_timer_4a: 0x0020,
                parent_timer_48: 0x000B,
                current_hp: 3,
                walk_speed: 0x0001_0000,
            },
            "ASM Tank.asm loc_3B31E should double selected art slot 1 into `$3E=2`, use palette/VRAM slot 1, set object_meta bit 7, clear priority, and store enemy_level+2 HP",
        );
    }

    #[test]
    fn tank_init_drive_sound_loc_3b40a_starts_first_visible_loop() {
        let result = tank_init_drive_sound_loc_3b40a(0, false, 0);

        assert_eq!(
            result,
            TankInitDriveSoundLoc3B40A {
                next_tank_drive_refcount: 1,
                sound_call: Some(SharedEnemyDriveSoundCallSub3C352::StartSequence(
                    SharedEnemyDriveSoundEffectSub3C352::TankDriving,
                )),
                walker_entry: TankInitWalkerEntryLoc3B40A::LeftLoc3C0D2,
            },
            "ASM Tank.asm loc_3B40A should play TankDriving for the first visible tank, increment refcount, and branch to loc_3C0D2 when x_direction is zero",
        );
    }

    #[test]
    fn tank_init_drive_sound_loc_3b40a_skips_sound_but_still_increments() {
        let result = tank_init_drive_sound_loc_3b40a(2, true, 0xFF);

        assert_eq!(
            result,
            TankInitDriveSoundLoc3B40A {
                next_tank_drive_refcount: 3,
                sound_call: None,
                walker_entry: TankInitWalkerEntryLoc3B40A::RightLoc3C026,
            },
            "ASM Tank.asm loc_3B40A should skip sound when refcount or `$19(a3)` is nonzero, still increment, and branch to loc_3C026 for nonzero x_direction",
        );
    }

    #[test]
    fn tank_update_loc_3b41c_plays_mouth_sound_then_keeps_walking_without_damage() {
        let result = tank_update_loc_3b41c(0, 0, 3, TankMouthFrameLoc3B41C::OpeningTimerA, false);

        assert_eq!(
            result,
            TankUpdateLoc3B41C {
                mouth_sound: Some(TankMouthSoundLoc3B41C::Open),
                next_current_hp: 3,
                clears_child_collision_type: true,
                hides_child_sprite: false,
                sets_child_kid_collision: false,
                clears_parent_motion_speed: false,
                child_recoil_timer: None,
                hurt_hibernate_frames: None,
                route: TankUpdateRouteLoc3B41C::Walk,
            },
            "ASM Tank.asm loc_3B444 should join child collision routing after the open-mouth sound, then clear a zero child collision word and resume the walker",
        );
    }

    #[test]
    fn tank_mouth_animation_step_loc_3b27e_maps_only_asm_timer_cues() {
        assert_eq!(
            TANK_MOUTH_ANIMATION_FRAMES_LOC_3B27E, 80,
            "ASM Tank.asm `stru_3B27E` should total closed $37 + open $A + transition 6 + open 4 + closed 5 frames",
        );
        assert_eq!(
            tank_mouth_animation_step_loc_3b27e(0),
            TankMouthAnimationStepLoc3B27E {
                mouth_frame: TankMouthFrameLoc3B41C::ClosingTimer37,
                next_animation_phase: 1,
            },
            "ASM Tank.asm loc_3B446 should see `$22=C7EC8` and `$32=$37` only on the first closed tick of the loop",
        );
        assert_eq!(
            tank_mouth_animation_step_loc_3b27e(TANK_MOUTH_OPEN_PHASE_LOC_3B27E as u8),
            TankMouthAnimationStepLoc3B27E {
                mouth_frame: TankMouthFrameLoc3B41C::OpeningTimerA,
                next_animation_phase: (TANK_MOUTH_OPEN_PHASE_LOC_3B27E + 1) as u8,
            },
            "ASM Tank.asm loc_3B41C should play the open cue on `stru_3B27E`'s `$A` `C7F20` entry",
        );
        assert_eq!(
            tank_mouth_animation_step_loc_3b27e(79),
            TankMouthAnimationStepLoc3B27E {
                mouth_frame: TankMouthFrameLoc3B41C::Other,
                next_animation_phase: 0,
            },
            "ASM Tank.asm `stru_3B27E`'s final 5-frame `C7EC8` entry must not play the `$37` closed cue before the loop wraps",
        );
    }

    #[test]
    fn tank_spawn_mouth_phase_skips_init_hibernate_tick() {
        assert_eq!(
            spawn_tank_mouth_animation_phase(EnemyFamily::Tank),
            1,
            "ASM Tank.asm Enemy0A_RockTank_Init loads `stru_3B27E`, then hibernates through the shared walker before loc_3B41C can observe the child",
        );
        assert_eq!(
            spawn_tank_mouth_animation_phase(EnemyFamily::TankShooting),
            0
        );
    }

    #[test]
    fn tank_update_loc_3b41c_hurts_on_weapon_or_stomp_child_collision() {
        let result =
            tank_update_loc_3b41c(0, 0x001C, 3, TankMouthFrameLoc3B41C::ClosingTimer37, false);

        assert_eq!(
            result,
            TankUpdateLoc3B41C {
                mouth_sound: Some(TankMouthSoundLoc3B41C::Closed),
                next_current_hp: 2,
                clears_child_collision_type: true,
                hides_child_sprite: false,
                sets_child_kid_collision: false,
                clears_parent_motion_speed: true,
                child_recoil_timer: Some(6),
                hurt_hibernate_frames: Some(5),
                route: TankUpdateRouteLoc3B41C::Hurt { remaining_hp: 2 },
            },
            "ASM Tank.asm loc_3B446 should play the closed-mouth sound before decrementing HP for child collision `$1C`, stopping parent motion, setting child timer 6, hibernating 5 frames, then clearing child collision",
        );
    }

    #[test]
    fn tank_update_loc_3b478_decrements_hp_for_damage_collisions() {
        for child_collision_type in [0x001C, 0x002C] {
            let result = tank_update_loc_3b41c(
                0,
                child_collision_type,
                4,
                TankMouthFrameLoc3B41C::Other,
                false,
            );

            assert_eq!(
                result.next_current_hp, 3,
                "ASM Tank.asm loc_3B478 should decrement current_hp for child collision {child_collision_type:#06X}",
            );
        }
    }

    #[test]
    fn tank_update_loc_3b41c_deletes_on_final_child_hit() {
        let result = tank_update_loc_3b41c(0, 0x002C, 1, TankMouthFrameLoc3B41C::Other, false);

        assert_eq!(
            result,
            TankUpdateLoc3B41C {
                mouth_sound: None,
                next_current_hp: 0,
                clears_child_collision_type: false,
                hides_child_sprite: true,
                sets_child_kid_collision: true,
                clears_parent_motion_speed: false,
                child_recoil_timer: None,
                hurt_hibernate_frames: None,
                route: TankUpdateRouteLoc3B41C::Death,
            },
            "ASM Tank.asm loc_3B464 should accept `$2C` as a damage collision and route a final hit to death after the current_hp decrement reaches zero",
        );
    }

    #[test]
    fn tank_update_loc_3b480_death_path_hides_child_without_hurt_recovery() {
        let result = tank_update_loc_3b41c(0, 0x001C, 1, TankMouthFrameLoc3B41C::Other, false);

        assert_eq!(
            result,
            TankUpdateLoc3B41C {
                mouth_sound: None,
                next_current_hp: 0,
                clears_child_collision_type: false,
                hides_child_sprite: true,
                sets_child_kid_collision: true,
                clears_parent_motion_speed: false,
                child_recoil_timer: None,
                hurt_hibernate_frames: None,
                route: TankUpdateRouteLoc3B41C::Death,
            },
            "ASM Tank.asm loc_3B480 should hide the mouth child and skip hurt recovery when the loc_3B478 HP decrement reaches zero",
        );
    }

    #[test]
    fn tank_update_loc_3b49a_hurt_recovery_stops_motion_and_recoils_child() {
        let result = tank_update_loc_3b41c(0, 0x002C, 2, TankMouthFrameLoc3B41C::Other, false);

        assert_eq!(
            result,
            TankUpdateLoc3B41C {
                mouth_sound: None,
                next_current_hp: 1,
                clears_child_collision_type: true,
                hides_child_sprite: false,
                sets_child_kid_collision: false,
                clears_parent_motion_speed: true,
                child_recoil_timer: Some(6),
                hurt_hibernate_frames: Some(5),
                route: TankUpdateRouteLoc3B41C::Hurt { remaining_hp: 1 },
            },
            "ASM Tank.asm loc_3B49A should clear parent motion, set child recoil timer 6, hibernate 5 frames, then keep the damaged tank alive",
        );
    }

    #[test]
    fn tank_resume_walk_loc_3b4ba_clears_child_collision_and_branches() {
        assert_eq!(
            tank_resume_walk_loc_3b4ba(0),
            TankResumeWalkLoc3B4BA {
                next_child_collision_type: 0,
                branch: TankResumeWalkBranchLoc3B4BA::LeftSharedWalker,
            },
            "ASM Tank.asm loc_3B4BA should clear mouth child `$38` and branch clear x_direction to loc_3C0FA",
        );
        assert_eq!(
            tank_resume_walk_loc_3b4ba(0xFF),
            TankResumeWalkLoc3B4BA {
                next_child_collision_type: 0,
                branch: TankResumeWalkBranchLoc3B4BA::RightSharedWalker,
            },
            "ASM Tank.asm loc_3B4BA should branch any nonzero x_direction to loc_3C04E after clearing mouth child `$38`",
        );
    }

    #[test]
    fn tank_update_loc_3b41c_negative_parent_collision_deletes_before_mouth_sound() {
        let result =
            tank_update_loc_3b41c(0x8000, 0, 3, TankMouthFrameLoc3B41C::OpeningTimerA, false);

        assert_eq!(
            result,
            TankUpdateLoc3B41C {
                mouth_sound: None,
                next_current_hp: 3,
                clears_child_collision_type: false,
                hides_child_sprite: true,
                sets_child_kid_collision: true,
                clears_parent_motion_speed: false,
                child_recoil_timer: None,
                hurt_hibernate_frames: None,
                route: TankUpdateRouteLoc3B41C::Death,
            },
            "ASM Tank.asm loc_3B41C should test the parent collision word first and skip mouth sounds when the negative bit routes directly to death",
        );
    }

    #[test]
    fn tank_shooting_layout_object_maps_enemy_id_0b() {
        let object = wildside_data::EnemyLayoutObject {
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
    fn tank_shooting_art_slot_loc_3b56c_accepts_shooting_or_plain_tank_slots() {
        assert_eq!(
            tank_shooting_art_slot_loc_3b56c(&[0x000B, 0x001C, 0x0022]),
            0,
            "ASM Tank_Shooting.asm loc_3B56C should keep d4=0 when header slot 0 masks to shooting tank `$0B`"
        );
        assert_eq!(
            tank_shooting_art_slot_loc_3b56c(&[0x0022, 0x400A, 0x001C]),
            1,
            "ASM Tank_Shooting.asm loc_3B56C should accept plain tank `$0A` as the shared tank art in slot 1"
        );
        assert_eq!(
            tank_shooting_art_slot_loc_3b56c(&[0x0022, 0x401C, 0x000B]),
            2,
            "ASM Tank_Shooting.asm loc_3B56C only checks the first two header slots before falling back to d4=2"
        );
    }

    #[test]
    fn tank_shooting_init_visible_state_loc_3b5a6_uses_selected_art_slot() {
        let state = tank_shooting_init_visible_state_loc_3b5a6(1, &[0x0022, 0x400A, 0x001C]);

        assert_eq!(
            state,
            TankShootingInitVisibleStateLoc3B5A6 {
                status_flag_13: true,
                object_meta: 0x008B,
                art_slot_word: 2,
                palette_line: 2,
                vram_tile: 0x0409,
                priority: 0,
                parent_timer_4a: 0x0020,
                parent_timer_48: 0x000B,
                current_hp: 3,
                walk_speed: 0x0001_0000,
            },
            "ASM Tank_Shooting.asm loc_3B5A6 should double selected art slot 1 into `$3E=2`, use palette/VRAM slot 1, set object_meta bit 7, clear priority, and store enemy_level+2 HP",
        );
    }

    #[test]
    fn tank_shooting_walk_speed_loc_3b5f2_matches_hp_tiers() {
        assert_eq!(
            tank_shooting_walk_speed_loc_3b5f2(2),
            0x0000_8000,
            "ASM Tank_Shooting.asm loc_3B5F2 should be skipped below HP 3, leaving only the final `$8000` add"
        );
        assert_eq!(
            tank_shooting_walk_speed_loc_3b5f2(3),
            0x0001_0000,
            "ASM Tank_Shooting.asm loc_3B5F2 should add `$8000` for HP 3 before the final `$8000` add"
        );
        assert_eq!(
            tank_shooting_walk_speed_loc_3b5f2(4),
            0x0001_8000,
            "ASM Tank_Shooting.asm loc_3B5F2 should preserve the HP>3 `$8000` preload and add two more `$8000` steps"
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::TankShooting, 4),
            0x0001_8000,
            "native TankShooting pacer speed should route through Tank_Shooting.asm loc_3B5F2"
        );
    }

    #[test]
    fn tank_shooting_cycle_frames_loc_3b682_uses_slow_script_unless_hp_is_four() {
        assert_eq!(
            tank_shooting_cycle_frames(2),
            TANK_SHOOTING_SLOW_CYCLE_FRAMES,
            "ASM Tank_Shooting.asm loc_3B682 installs `stru_3B4F6` on the HP 2 slow branch"
        );
        assert_eq!(
            tank_shooting_cycle_frames(3),
            TANK_SHOOTING_SLOW_CYCLE_FRAMES,
            "ASM Tank_Shooting.asm loc_3B682 installs `stru_3B4F6` on the HP 3 slow branch"
        );
        assert_eq!(
            tank_shooting_cycle_frames(5),
            TANK_SHOOTING_SLOW_CYCLE_FRAMES,
            "ASM Tank_Shooting.asm loc_3B682 is reached by `cmpi.w #4,d7; bne.s loc_3B682`, so HP 5 is not the fast script"
        );
        assert_eq!(
            tank_shooting_cycle_frames(4),
            TANK_SHOOTING_FAST_CYCLE_FRAMES,
            "ASM Tank_Shooting.asm skips loc_3B682 only for exact HP 4"
        );
    }

    #[test]
    fn tank_shooting_init_drive_sound_loc_3b690_starts_first_visible_loop() {
        let result = tank_shooting_init_drive_sound_loc_3b690(0, false, 0);

        assert_eq!(
            result,
            TankShootingInitDriveSoundLoc3B690 {
                next_tank_drive_refcount: 1,
                sound_call: Some(SharedEnemyDriveSoundCallSub3C352::StartSequence(
                    SharedEnemyDriveSoundEffectSub3C352::TankDriving,
                )),
                walker_entry: TankShootingInitWalkerEntryLoc3B690::LeftLoc3C0D2,
            },
            "ASM Tank_Shooting.asm loc_3B690 should play TankDriving for the first visible tank; loc_3B6AE should increment refcount and branch to loc_3C0D2 when x_direction is zero",
        );
    }

    #[test]
    fn tank_shooting_init_drive_sound_loc_3b690_skips_sound_but_still_increments() {
        let result = tank_shooting_init_drive_sound_loc_3b690(2, true, 0xFF);

        assert_eq!(
            result,
            TankShootingInitDriveSoundLoc3B690 {
                next_tank_drive_refcount: 3,
                sound_call: None,
                walker_entry: TankShootingInitWalkerEntryLoc3B690::RightLoc3C026,
            },
            "ASM Tank_Shooting.asm loc_3B690 should skip sound when refcount or `$19(a3)` is nonzero; loc_3B6AE should still increment and branch to loc_3C026 for nonzero x_direction",
        );
    }

    #[test]
    fn tank_shooting_collision_route_loc_3b6c0_classifies_parent_and_child_death() {
        assert_eq!(
            tank_shooting_collision_route_loc_3b6c0(0x8000, 0x001C),
            TankShootingCollisionRouteLoc3B6C0::Death,
            "ASM Tank_Shooting.asm loc_3B6C0 should route a negative parent collision word to death before applying child damage collisions",
        );
        assert_eq!(
            tank_shooting_collision_route_loc_3b6c0(0, 0x8000),
            TankShootingCollisionRouteLoc3B6C0::Death,
            "ASM Tank_Shooting.asm loc_3B6C0 should route a negative turret-child collision word to death",
        );
    }

    #[test]
    fn tank_shooting_collision_route_loc_3b6c0_classifies_walk_and_damage() {
        assert_eq!(
            tank_shooting_collision_route_loc_3b6c0(0, 0),
            TankShootingCollisionRouteLoc3B6C0::Walk,
            "ASM Tank_Shooting.asm loc_3B6C0 should resume walking when the turret child collision is zero",
        );
        assert_eq!(
            tank_shooting_collision_route_loc_3b6c0(0, 0x001D),
            TankShootingCollisionRouteLoc3B6C0::Walk,
            "ASM Tank_Shooting.asm loc_3B6C0 should resume walking for child collisions other than `$1C` or `$2C`",
        );
        assert_eq!(
            tank_shooting_collision_route_loc_3b6c0(0, 0x001C),
            TankShootingCollisionRouteLoc3B6C0::Damage,
            "ASM Tank_Shooting.asm loc_3B6C0 should accept child collision `$1C` as a damage hit",
        );
        assert_eq!(
            tank_shooting_collision_route_loc_3b6c0(0, 0x002C),
            TankShootingCollisionRouteLoc3B6C0::Damage,
            "ASM Tank_Shooting.asm loc_3B6C0 should accept child collision `$2C` as a damage hit",
        );
    }

    #[test]
    fn tank_shooting_damage_after_hit_loc_3b6de_branches_on_decremented_hp() {
        assert_eq!(
            tank_shooting_damage_after_hit_loc_3b6de(1),
            TankShootingDamageRouteLoc3B6DE::Death,
            "ASM Tank_Shooting.asm loc_3B6DE should fall through to death when decrementing current_hp leaves zero",
        );
        assert_eq!(
            tank_shooting_damage_after_hit_loc_3b6de(3),
            TankShootingDamageRouteLoc3B6DE::Hurt { remaining_hp: 2 },
            "ASM Tank_Shooting.asm loc_3B6DE should branch to the hurt path when decremented current_hp remains nonzero",
        );
    }

    #[test]
    fn tank_shooting_resume_walk_loc_3b720_clears_child_collision_and_branches() {
        assert_eq!(
            tank_shooting_resume_walk_loc_3b720(0),
            TankShootingResumeWalkLoc3B720 {
                next_child_collision_type: 0,
                branch: TankShootingResumeWalkBranchLoc3B720::LeftSharedWalker,
            },
            "ASM Tank_Shooting.asm loc_3B720 should clear turret child `$38` and branch clear x_direction to loc_3C0FA",
        );
        assert_eq!(
            tank_shooting_resume_walk_loc_3b720(0xFF),
            TankShootingResumeWalkLoc3B720 {
                next_child_collision_type: 0,
                branch: TankShootingResumeWalkBranchLoc3B720::RightSharedWalker,
            },
            "ASM Tank_Shooting.asm loc_3B720 should branch any nonzero x_direction to loc_3C04E after clearing turret child `$38`",
        );
    }

    #[test]
    fn tank_shooting_hurt_recovery_loc_3b700_stops_motion_then_resumes_walk() {
        assert_eq!(
            tank_shooting_hurt_recovery_loc_3b700(0),
            TankShootingHurtRecoveryLoc3B700 {
                parent_motion_speed: 0,
                child_recoil_timer: 6,
                child_animation_script: "stru_3B294",
                hibernate_frames: 5,
                resume_walk: TankShootingResumeWalkLoc3B720 {
                    next_child_collision_type: 0,
                    branch: TankShootingResumeWalkBranchLoc3B720::LeftSharedWalker,
                },
            },
            "ASM Tank_Shooting.asm loc_3B700 should clear `$50(a5)`, set turret `$46=6`, play stru_3B294, hibernate five frames, then fall through to loc_3B720",
        );
    }

    #[test]
    fn tank_shooting_death_loc_3b6e6_hides_turret_and_deactivates_after_animation() {
        assert_eq!(
            tank_shooting_death_loc_3b6e6(),
            TankShootingDeathLoc3B6E6 {
                child_visible: false,
                child_has_kid_collision: true,
                death_animation_script: "stru_3B518",
                hibernates_until_animation_finished: true,
                route: TankShootingDeathRouteLoc3B6E6::SharedDeactivateLoc3C46E,
            },
            "ASM Tank_Shooting.asm loc_3B6E6 should hide turret `$13`, set turret `$3D`, play stru_3B518, hibernate until finished, then branch to loc_3C46E",
        );
    }

    #[test]
    fn tank_bullet_loop_route_loc_3b7ba_keeps_zero_collision_inside_camera_box() {
        assert_eq!(
            tank_bullet_loop_route_loc_3b7ba(0x0200, 0x0120, 0x0200, 0x0100, 0),
            TankBulletLoopRouteLoc3B7BA::Continue,
            "ASM Tank_Shooting.asm loc_3B7BA should loop while x/y camera-relative words are in range and collision_type is zero",
        );

        assert_eq!(
            tank_bullet_loop_route_loc_3b7ba(0x0180, 0x0080, 0x0200, 0x0100, 0),
            TankBulletLoopRouteLoc3B7BA::Continue,
            "ASM Tank_Shooting.asm loc_3B7BA should keep the signed lower bound `$FF80` inclusive",
        );

        assert_eq!(
            tank_bullet_loop_route_loc_3b7ba(0x03C0, 0x02C0, 0x0200, 0x0100, 0),
            TankBulletLoopRouteLoc3B7BA::Continue,
            "ASM Tank_Shooting.asm loc_3B7BA should keep the signed upper bound `$01C0` inclusive",
        );
    }

    #[test]
    fn tank_bullet_loop_route_loc_3b7ba_branches_to_loc_3b7ec_on_bounds_or_collision() {
        assert_eq!(
            tank_bullet_loop_route_loc_3b7ba(0x017F, 0x0100, 0x0200, 0x0100, 0),
            TankBulletLoopRouteLoc3B7BA::Delete,
            "ASM Tank_Shooting.asm loc_3B7BA should branch to loc_3B7EC when signed relative x is below `$FF80`",
        );

        assert_eq!(
            tank_bullet_loop_route_loc_3b7ba(0x03C1, 0x0100, 0x0200, 0x0100, 0),
            TankBulletLoopRouteLoc3B7BA::Delete,
            "ASM Tank_Shooting.asm loc_3B7BA should branch to loc_3B7EC when signed relative x is above `$01C0`",
        );

        assert_eq!(
            tank_bullet_loop_route_loc_3b7ba(0x0200, 0x007F, 0x0200, 0x0100, 0),
            TankBulletLoopRouteLoc3B7BA::Delete,
            "ASM Tank_Shooting.asm loc_3B7BA should branch to loc_3B7EC when signed relative y is below `$FF80`",
        );

        assert_eq!(
            tank_bullet_loop_route_loc_3b7ba(0x0200, 0x02C1, 0x0200, 0x0100, 0),
            TankBulletLoopRouteLoc3B7BA::Delete,
            "ASM Tank_Shooting.asm loc_3B7BA should branch to loc_3B7EC when signed relative y is above `$01C0`",
        );

        assert_eq!(
            tank_bullet_loop_route_loc_3b7ba(0x0200, 0x0100, 0x0200, 0x0100, 0x001C),
            TankBulletLoopRouteLoc3B7BA::Delete,
            "ASM Tank_Shooting.asm loc_3B7BA should branch to loc_3B7EC on any nonzero collision_type after the bounds checks",
        );

        assert_eq!(
            tank_bullet_loop_route_loc_3b7ba(0x0200, 0x0100, 0x0200, 0x0100, 0xFFFF),
            TankBulletLoopRouteLoc3B7BA::Delete,
            "ASM Tank_Shooting.asm loc_3B7EC is also reached for signed-negative nonzero collision_type words",
        );
    }

    #[test]
    fn tank_shooting_fire_uses_loc_3c50e_cadence_and_loc_3b730_offsets() {
        let mut enemy = ActiveEnemy {
            family: EnemyFamily::TankShooting,
            hit_points: 4,
            death_animation_active: false,
            facing: Direction::Right,
            x_pos: 100 << 16,
            y_pos: 80 << 16,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();

        step_tank_shooting_fire(&mut enemy, &mut projectiles, None);

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
    fn tank_shooting_fire_queues_tank_shoots_sfx_loc_3b730() {
        // ASM Tank_Shooting.asm loc_3B730 (line 190): one hibernate frame after
        // the bullet worker spawns, `moveq #sfx_Tank_shoots,d0; jsr j_PlaySound`
        // plays sfx id $41 (constants.asm:89). #338: native had no emit site, so
        // the shot was silent. The projectile-spawn edge in step_tank_shooting_fire
        // is the native equivalent of the loc_3B730 worker load.
        let mut enemy = ActiveEnemy {
            family: EnemyFamily::TankShooting,
            hit_points: 4,
            death_animation_active: false,
            facing: Direction::Right,
            x_pos: 100 << 16,
            y_pos: 80 << 16,
            wait_timer: 0,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();
        let mut sound_effects = Vec::new();

        step_tank_shooting_fire(&mut enemy, &mut projectiles, Some(&mut sound_effects));

        assert_eq!(
            projectiles.len(),
            1,
            "the loc_3C50E gate fires one bullet on the first ready frame"
        );
        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::TankShoots],
            "ASM Tank_Shooting.asm loc_3B730 plays sfx_Tank_shoots=$41 on the shot frame; native must queue it on the projectile-spawn edge"
        );
        assert_eq!(
            QueuedSoundEffect::TankShoots.sound_id(),
            0x41,
            "constants.asm: sfx_Tank_shoots = $41"
        );
    }

    #[test]
    fn tank_shooting_fire_silent_when_no_shot_this_frame_loc_3c50e() {
        // ASM loc_3C50E only spawns loc_3B730 on the turret gate; off-gate
        // frames must neither spawn a bullet nor play sfx_Tank_shoots.
        let mut enemy = ActiveEnemy {
            family: EnemyFamily::TankShooting,
            hit_points: 4,
            death_animation_active: false,
            facing: Direction::Right,
            x_pos: 100 << 16,
            y_pos: 80 << 16,
            wait_timer: 5,
            ..mini_hopping_skull_test_enemy()
        };
        let mut projectiles = Vec::new();
        let mut sound_effects = Vec::new();

        step_tank_shooting_fire(&mut enemy, &mut projectiles, Some(&mut sound_effects));

        assert!(
            projectiles.is_empty(),
            "no shot while wait_timer counts down"
        );
        assert!(
            sound_effects.is_empty(),
            "sfx_Tank_shoots only plays on the loc_3B730 spawn frame"
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
            TANK_SHOOTING_INIT_FIRE_TIMER_LOC_3B5FA,
            "ASM Tank_Shooting.asm loc_3B5FA seeds `$54(a5)` with `$49`; GfxObject_Animate reaches `loc_3C50E` one frame later"
        );
        assert_eq!(
            TANK_SHOOTING_INIT_FIRE_TIMER_LOC_3B5FA,
            TANK_SHOOTING_FIRST_SHOT_DELAY_FRAMES - 1,
            "ASM Tank_Shooting.asm loc_3B5FA stores the countdown value for a 74-frame first-shot delay"
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
    fn tank_active_snapshot_exposes_turret_child_from_loc_3b372() {
        let tank = ActiveEnemy {
            segment_index: 0,
            layout_index: 1,
            enemy_level: 0,
            family: EnemyFamily::Tank,
            x_pos: 0x00C2_8000,
            y_pos: 0x00E0_0000,
            x_velocity: -0x0000_8000,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: true,
            hit_points: 2,
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
            crab_frozen: false,
            crab_berzerker_knockback: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
            shared_walker_turnaround_counter: 0,
            shared_walker_right_turn_extra_step_consumed: false,
            shared_walker_patrol_short_next: false,
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        };

        let body = tank.snapshot();
        assert_eq!(
            body.x_velocity,
            Some(0),
            "ASM Tank.asm loc_3B40A -> kid.asm loc_3C026/loc_3C0D2 clears object x_vel; native `$50(a5)` patrol speed must not leak into body snapshots",
        );
        assert_eq!(
            body.y_velocity,
            Some(0),
            "ASM kid.asm loc_3C026/loc_3C0D2 also clears object y_vel for grounded Tank walkers",
        );

        let child = tank
            .child_attachment_snapshot()
            .expect("ASM Tank.asm loc_3B372 allocates a linked turret child");

        assert_eq!(
            child.kind,
            NativeObjectKind::Enemy {
                object_type: EnemyFamily::Tank.object_type()
            },
        );
        assert_eq!(child.layout_index, Some(1));
        assert_eq!(child.x_pos, 0x00C2_0000);
        assert_eq!(child.y_pos, 0x00D8_0000);
        assert_eq!(child.x_velocity, Some(0));
        assert_eq!(child.facing, Some(Direction::Right));
        assert_eq!(
            child.hit_points,
            Some(0),
            "ASM loc_3B372 creates a linked gfx child whose zeroed HP byte is still visible in oracle rosters",
        );
    }

    #[test]
    fn tank_hurt_snapshot_uses_recoiled_child_offset_from_loc_3b49a() {
        let mut tank = ActiveEnemy {
            segment_index: 0,
            layout_index: 9,
            enemy_level: 0,
            family: EnemyFamily::Tank,
            x_pos: 0x0249_8000,
            y_pos: 0x0250_0000,
            x_velocity: 0,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: true,
            hit_points: 1,
            death_animation_active: false,
            grounded_frames: 0,
            tank_mouth_animation_phase: 0,
            damage_flash_frames: ENEMY_DAMAGE_FLASH_FRAMES,
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
            crab_frozen: false,
            crab_berzerker_knockback: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
            shared_walker_turnaround_counter: 0,
            shared_walker_right_turn_extra_step_consumed: false,
            shared_walker_patrol_short_next: false,
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        };

        assert_eq!(
            TANK_CHILD_RECOIL_VISIBLE_FRAMES_LOC_3B49A, 8,
            "Issue #213 oracle route keeps Tank.asm loc_3B49A child `$46=8` through frame 363 and exposes `$46=6` from frame 364",
        );
        assert_eq!(
            TANK_HURT_TRACE_HIBERNATE_FRAMES_LOC_3B49A, 14,
            "Issue #213 route samples the Tank.asm loc_3B49A hibernate tail before the shared walker resumes with `$50(a5)=0`",
        );

        let child = tank
            .child_attachment_snapshot()
            .expect("ASM Tank.asm loc_3B372 allocates a linked turret child");

        assert_eq!(child.y_pos, 0x0248_0000);

        tank.wait_timer = TANK_HURT_TRACE_HIBERNATE_FRAMES_LOC_3B49A.into();
        let hibernate_child = tank
            .child_attachment_snapshot()
            .expect("ASM Tank.asm loc_3B49A hibernates before the recoil pose is visible");
        assert_eq!(hibernate_child.y_pos, 0x0248_0000);

        tank.wait_timer = (TANK_CHILD_RECOIL_VISIBLE_FRAMES_LOC_3B49A + 1).into();
        let pre_recoil_child = tank
            .child_attachment_snapshot()
            .expect("ASM Tank.asm loc_3B49A keeps the linked turret child visible before recoil");
        assert_eq!(
            pre_recoil_child.y_pos, 0x0248_0000,
            "ASM Tank.asm loc_3B49A should not expose the `stru_3B294` recoil offset until the exported hibernate tail reaches the oracle-confirmed recoil window",
        );

        tank.wait_timer = TANK_CHILD_RECOIL_VISIBLE_FRAMES_LOC_3B49A.into();
        let recoiled_child = tank
            .child_attachment_snapshot()
            .expect("ASM Tank.asm loc_3B49A keeps the linked turret child visible");

        assert_eq!(recoiled_child.y_pos, 0x024A_0000);
        assert_eq!(
            recoiled_child.hit_points,
            Some(0),
            "ASM Tank.asm loc_3B49A keeps the child object alive with zero HP while `stru_3B294` hibernates",
        );

        tank.wait_timer = 0;
        tank.walk_speed_override = TANK_STOPPED_WALK_SPEED_OVERRIDE_LOC_3B49A;
        tank.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
        let stopped_child = tank
            .child_attachment_snapshot()
            .expect("ASM Tank.asm loc_3B49A leaves the linked child recoiled after hibernate");
        assert_eq!(
            stopped_child.y_pos, 0x024A_0000,
            "ASM Tank.asm loc_3B4BA resumes the shared walker without restoring child `$46` from loc_3B49A",
        );

        tank.wait_timer = 0;
        tank.walk_speed_override = 0;
        let recovered_child = tank
            .child_attachment_snapshot()
            .expect("normal Tank child sync uses the loc_3B372 eight-pixel offset");
        assert_eq!(recovered_child.y_pos, 0x0248_0000);

        let mut data = vec![0u8; 16 * 16 * 2];
        for col in 0..16 {
            let offset = ((15 * 16) + col) * 2;
            data[offset] = 0x60;
        }
        let grid = terrain::BlockGrid::from_level_layout(&data, 16, 16);
        tank.walk_speed_override = TANK_STOPPED_WALK_SPEED_OVERRIDE_LOC_3B49A;
        tank.wait_timer = 0;
        tank.on_ground = true;
        tank.x_velocity = -0x8000;
        step_pacer(&mut tank, Some(&grid));
        assert_eq!(
            tank.x_velocity, 0,
            "ASM Tank.asm loc_3B49A clears `$50(a5)` and loc_3B4BA does not restore it before loc_3C0FA",
        );
    }
}
