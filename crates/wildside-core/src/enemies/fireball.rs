use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballRespawnMarkerLoc32ECA {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

pub(crate) fn fireball_respawn_marker_loc_32eca(
    object_status_slot: u8,
) -> FireballRespawnMarkerLoc32ECA {
    // ASM Fireball.asm loc_32ECA: the positive `$42(a5)` byte is doubled as a
    // word offset into EnemyStatus_Table, then `$2168` is written.
    FireballRespawnMarkerLoc32ECA {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballDeleteTailLoc32ED8 {
    pub(crate) deletes_current_object: bool,
}

pub(crate) fn fireball_delete_tail_loc_32ed8() -> FireballDeleteTailLoc32ED8 {
    // ASM Fireball.asm loc_32ED8: every death-status route converges on the
    // `j_Delete_CurrentObject` tail after the explosion animation finishes.
    FireballDeleteTailLoc32ED8 {
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FireballTrailLoopStepLoc33110 {
    Continue { lifetime_frames: u16 },
    Delete,
}

pub(crate) fn fireball_trail_loop_step_loc_33110(
    flag_19_is_set: bool,
    lifetime_frames: u16,
) -> FireballTrailLoopStepLoc33110 {
    // ASM Fireball.asm loc_33110: after hibernating one frame, `$19(a3)` takes
    // the delete tail. Otherwise `subq.w #1,d3` runs as a wrapping word
    // decrement; only a zero result falls through to that same tail.
    if flag_19_is_set {
        return FireballTrailLoopStepLoc33110::Delete;
    }

    let next_lifetime_frames = lifetime_frames.wrapping_sub(1);
    if next_lifetime_frames == 0 {
        FireballTrailLoopStepLoc33110::Delete
    } else {
        FireballTrailLoopStepLoc33110::Continue {
            lifetime_frames: next_lifetime_frames,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FireballObstacleTrailLoopStepLoc331BA {
    Continue { lifetime_frames: u16 },
    Delete,
}

pub(crate) fn fireball_obstacle_trail_loop_step_loc_331ba(
    flag_19_is_set: bool,
    lifetime_frames: u16,
) -> FireballObstacleTrailLoopStepLoc331BA {
    // ASM Fireball.asm loc_331BA: after `loc_33128`'s wall/floor probes accept
    // the spawned trail, each iteration hibernates once, checks `$19(a3)`, then
    // decrements `d3` as a wrapping word before either looping or deleting.
    if flag_19_is_set {
        return FireballObstacleTrailLoopStepLoc331BA::Delete;
    }

    let next_lifetime_frames = lifetime_frames.wrapping_sub(1);
    if next_lifetime_frames == 0 {
        FireballObstacleTrailLoopStepLoc331BA::Delete
    } else {
        FireballObstacleTrailLoopStepLoc331BA::Continue {
            lifetime_frames: next_lifetime_frames,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballTrailDeleteTailLoc33120 {
    pub(crate) next_fire_trail_count: u16,
    pub(crate) deletes_current_object: bool,
}

pub(crate) fn fireball_trail_delete_tail_loc_33120(
    fire_trail_count: u16,
) -> FireballTrailDeleteTailLoc33120 {
    // ASM Fireball.asm loc_33120: `subq.w #1,(Number_of_Fire_Trails).w`, then
    // jump directly to `j_Delete_CurrentObject`.
    FireballTrailDeleteTailLoc33120 {
        next_fire_trail_count: fire_trail_count.wrapping_sub(1),
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballObstacleTrailDeleteTailLoc331CA {
    pub(crate) next_fire_trail_count: u16,
    pub(crate) deletes_current_object: bool,
}

pub(crate) fn fireball_obstacle_trail_delete_tail_loc_331ca(
    fire_trail_count: u16,
) -> FireballObstacleTrailDeleteTailLoc331CA {
    // ASM Fireball.asm loc_331CA: every `loc_33128` rejection and the
    // `loc_331BA` lifetime fall-through decrement Number_of_Fire_Trails before
    // jumping directly to `j_Delete_CurrentObject`.
    FireballObstacleTrailDeleteTailLoc331CA {
        next_fire_trail_count: fire_trail_count.wrapping_sub(1),
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FireballSpriteBranchLoc32F3E {
    Vertical { sprite_index: i16 },
    NegativeX,
    PositiveXNegativeY,
    PositiveXNonNegativeY,
}

fn fireball_velocity_word_after_shift8_loc_32f3e(velocity: i32) -> u16 {
    let shifted = velocity >> 8;
    let bytes = shifted.to_be_bytes();
    u16::from_be_bytes([bytes[2], bytes[3]])
}

pub(crate) fn fireball_vertical_sprite_loc_3300a(y_velocity: i32) -> i16 {
    // ASM Fireball.asm loc_3300A: vertical fireballs start with sprite `4`;
    // only a negative shifted Y word keeps it, otherwise the routine returns
    // straight down/idle sprite `$F`.
    let y_word = fireball_velocity_word_after_shift8_loc_32f3e(y_velocity);
    if signed_word(y_word) < 0 { 4 } else { 0x0F }
}

pub(crate) fn fireball_sprite_branch_loc_32f3e(
    x_velocity: i32,
    y_velocity: i32,
) -> FireballSpriteBranchLoc32F3E {
    // ASM Fireball.asm loc_32F3E: `asr.l #8` both velocities, keep the low
    // words, then branch first on signed X. A zero X velocity bypasses the
    // quadrant table and selects the straight-up/straight-down sprite index.
    let y_word = fireball_velocity_word_after_shift8_loc_32f3e(y_velocity);
    let x_word = fireball_velocity_word_after_shift8_loc_32f3e(x_velocity);
    let signed_x = signed_word(x_word);

    if signed_x == 0 {
        return FireballSpriteBranchLoc32F3E::Vertical {
            sprite_index: fireball_vertical_sprite_loc_3300a(y_velocity),
        };
    }

    if signed_x.is_negative() {
        return FireballSpriteBranchLoc32F3E::NegativeX;
    }

    if signed_word(y_word) < 0 {
        FireballSpriteBranchLoc32F3E::PositiveXNegativeY
    } else {
        FireballSpriteBranchLoc32F3E::PositiveXNonNegativeY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballSpriteSelectionLoc32F3E {
    pub(crate) sprite_index: i16,
    pub(crate) quadrant_index: u16,
}

const FIREBALL_SPRITE_DIRECTION_AND_BASE_LOC_3304A: [(u8, u8); 20] = [
    (0x00, 0x08),
    (0x00, 0x06),
    (0x00, 0x04),
    (0x00, 0x02),
    (0x00, 0x00),
    (0xFF, 0x00),
    (0xFF, 0x02),
    (0xFF, 0x04),
    (0xFF, 0x06),
    (0xFF, 0x08),
    (0xFF, 0x08),
    (0xFF, 0x0A),
    (0xFF, 0x0C),
    (0xFF, 0x0E),
    (0xFF, 0x10),
    (0x00, 0x10),
    (0x00, 0x0E),
    (0x00, 0x0C),
    (0x00, 0x0A),
    (0x00, 0x08),
];

const FIREBALL_SPRITE_LINKS_LOC_33072: [&str; 18] = [
    "LnkTo_unk_C78D6-Data_Index",
    "LnkTo_unk_C78DE-Data_Index",
    "LnkTo_unk_C78E6-Data_Index",
    "LnkTo_unk_C78EE-Data_Index",
    "LnkTo_unk_C78F6-Data_Index",
    "LnkTo_unk_C78FE-Data_Index",
    "LnkTo_unk_C7906-Data_Index",
    "LnkTo_unk_C790E-Data_Index",
    "LnkTo_unk_C7916-Data_Index",
    "LnkTo_unk_C791E-Data_Index",
    "LnkTo_unk_C7926-Data_Index",
    "LnkTo_unk_C792E-Data_Index",
    "LnkTo_unk_C7936-Data_Index",
    "LnkTo_unk_C793E-Data_Index",
    "LnkTo_unk_C7946-Data_Index",
    "LnkTo_unk_C794E-Data_Index",
    "LnkTo_unk_C7956-Data_Index",
    "LnkTo_unk_C795E-Data_Index",
];

pub(crate) fn fireball_sprite_link_loc_33072(sprite_table_index: u8) -> Option<&'static str> {
    // ASM Fireball.asm loc_33072: sign-extend the selected byte, double it as
    // a word offset into `off_33026`, then store that link into addroffset_sprite.
    FIREBALL_SPRITE_LINKS_LOC_33072
        .get(usize::from(sprite_table_index))
        .copied()
}

pub(crate) fn fireball_zero_phase_expiry_loc_33070() -> u16 {
    // ASM Fireball.asm loc_33070: when loc_3304A's five-frame countdown expires
    // while `d2` is zero, the branch sets `d2` to one for the next sprite write.
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FireballSpriteCommitLoc3304A {
    pub(crate) x_direction: u8,
    pub(crate) sprite_table_index: u8,
    pub(crate) sprite_link: &'static str,
    pub(crate) next_animation_phase: u16,
    pub(crate) next_animation_countdown: u16,
}

pub(crate) fn fireball_sprite_commit_loc_3304a(
    sprite_index: i16,
    animation_phase: u16,
    animation_countdown: u16,
) -> Option<FireballSpriteCommitLoc3304A> {
    // ASM Fireball.asm loc_3304A: double the selected angle sprite index,
    // copy byte_33080 into `x_direction(a3)`, add live `d2` to byte_33081,
    // then age/toggle the five-frame phase countdown before loc_33072 writes
    // addroffset_sprite.
    let sprite_table_entry =
        FIREBALL_SPRITE_DIRECTION_AND_BASE_LOC_3304A.get(usize::try_from(sprite_index).ok()?)?;
    let animation_phase_byte = u8::try_from(animation_phase).ok()?;
    let sprite_table_index = sprite_table_entry.1.checked_add(animation_phase_byte)?;
    let sprite_link = fireball_sprite_link_loc_33072(sprite_table_index)?;

    let decremented_countdown = animation_countdown.wrapping_sub(1);
    let (next_animation_countdown, next_animation_phase) = if decremented_countdown == 0 {
        let next_phase = if animation_phase == 0 {
            fireball_zero_phase_expiry_loc_33070()
        } else {
            0
        };
        (5, next_phase)
    } else {
        (decremented_countdown, animation_phase)
    };

    Some(FireballSpriteCommitLoc3304A {
        x_direction: sprite_table_entry.0,
        sprite_table_index,
        sprite_link,
        next_animation_phase,
        next_animation_countdown,
    })
}

fn fireball_ratio_lookup_unk_32edc(index: usize) -> i16 {
    // ASM Fireball.asm unk_32EDC: the first $3F ratio slots map to 0, 1, 2,
    // then 3; loc_32F78/loc_32FAC mirror or offset that base sprite index.
    match index {
        0..=2 => 0,
        3..=5 => 1,
        6..=13 => 2,
        _ => 3,
    }
}

fn fireball_ratio_sprite_index(abs_x_word: u16, abs_y_word: u16) -> Option<i16> {
    let scaled_y_word = abs_y_word.wrapping_shl(3);
    let divisor = i32::from(signed_word(abs_x_word));
    if divisor == 0 {
        return None;
    }

    let ratio = i32::from(scaled_y_word) / divisor;
    let ratio_index = u16::try_from(ratio).ok().map(usize::from)?;
    if ratio_index >= 0x3F {
        None
    } else {
        Some(fireball_ratio_lookup_unk_32edc(ratio_index))
    }
}

pub(crate) fn fireball_negative_x_negative_y_sprite_loc_32f78(
    x_velocity: i32,
    y_velocity: i32,
) -> FireballSpriteSelectionLoc32F3E {
    // ASM Fireball.asm loc_32F78: after negative X is detected, `neg.w d6`
    // makes the X divisor positive. With negative Y, `d5=1`, `neg.w d7`,
    // `(abs_y << 3) / abs_x` indexes unk_32EDC, then the sprite index is
    // mirrored with `neg.w d6; addi.w #9,d6`.
    let x_word = fireball_velocity_word_after_shift8_loc_32f3e(x_velocity);
    let y_word = fireball_velocity_word_after_shift8_loc_32f3e(y_velocity);
    let abs_x_word = x_word.wrapping_neg();
    let abs_y_word = y_word.wrapping_neg();
    let sprite_index = fireball_ratio_sprite_index(abs_x_word, abs_y_word)
        .map_or(4, |base_sprite_index| -base_sprite_index + 9);

    FireballSpriteSelectionLoc32F3E {
        sprite_index,
        quadrant_index: 1,
    }
}

pub(crate) fn fireball_positive_x_negative_y_sprite_loc_32fac(
    x_velocity: i32,
    y_velocity: i32,
) -> FireballSpriteSelectionLoc32F3E {
    // ASM Fireball.asm loc_32FAC: positive X and negative Y keep `d5=0`,
    // convert Y to magnitude, use `(abs_y << 3) / x` as the unk_32EDC index,
    // and pass the table byte through unchanged before loc_33006 copies d5.
    let x_word = fireball_velocity_word_after_shift8_loc_32f3e(x_velocity);
    let y_word = fireball_velocity_word_after_shift8_loc_32f3e(y_velocity);
    let abs_y_word = y_word.wrapping_neg();
    let sprite_index = fireball_ratio_sprite_index(x_word, abs_y_word).unwrap_or(4);

    FireballSpriteSelectionLoc32F3E {
        sprite_index,
        quadrant_index: 0,
    }
}

pub(crate) fn fireball_negative_x_nonnegative_y_sprite_loc_32fc8(
    x_velocity: i32,
    y_velocity: i32,
) -> FireballSpriteSelectionLoc32F3E {
    // ASM Fireball.asm loc_32FC8: negative X has already been converted to
    // magnitude and selected `d5=2`; non-negative Y indexes unk_32EDC with
    // `(y << 3) / abs_x`, then offsets the table byte by `$A`.
    let x_word = fireball_velocity_word_after_shift8_loc_32f3e(x_velocity);
    let y_word = fireball_velocity_word_after_shift8_loc_32f3e(y_velocity);
    let abs_x_word = x_word.wrapping_neg();
    let sprite_index = fireball_ratio_sprite_index(abs_x_word, y_word)
        .map_or(0x0F, |base_sprite_index| base_sprite_index + 0x0A);

    FireballSpriteSelectionLoc32F3E {
        sprite_index,
        quadrant_index: 2,
    }
}

pub(crate) fn fireball_positive_x_nonnegative_y_sprite_loc_32fe8(
    x_velocity: i32,
    y_velocity: i32,
) -> FireballSpriteSelectionLoc32F3E {
    // ASM Fireball.asm loc_32FE8: positive X/non-negative Y keeps `d5=3`,
    // indexes unk_32EDC with `(y << 3) / x`, then mirrors the table byte with
    // `neg.w d6; addi.w #$13,d6` unless loc_33018's fallback is taken first.
    let x_word = fireball_velocity_word_after_shift8_loc_32f3e(x_velocity);
    let y_word = fireball_velocity_word_after_shift8_loc_32f3e(y_velocity);
    let sprite_index = fireball_ratio_sprite_index(x_word, y_word)
        .map_or(0x0F, |base_sprite_index| -base_sprite_index + 0x13);

    FireballSpriteSelectionLoc32F3E {
        sprite_index,
        quadrant_index: 3,
    }
}

pub(crate) fn fireball_sprite_selection_loc_32f3e(
    x_velocity: i32,
    y_velocity: i32,
) -> FireballSpriteSelectionLoc32F3E {
    // ASM Fireball.asm loc_32F3E branches by shifted X/Y velocity words before
    // loc_3304A commits x_direction and the animated sprite link.
    match fireball_sprite_branch_loc_32f3e(x_velocity, y_velocity) {
        FireballSpriteBranchLoc32F3E::Vertical { sprite_index } => {
            FireballSpriteSelectionLoc32F3E {
                sprite_index,
                quadrant_index: 0,
            }
        }
        FireballSpriteBranchLoc32F3E::NegativeX => {
            let y_word = fireball_velocity_word_after_shift8_loc_32f3e(y_velocity);
            if signed_word(y_word) < 0 {
                fireball_negative_x_negative_y_sprite_loc_32f78(x_velocity, y_velocity)
            } else {
                fireball_negative_x_nonnegative_y_sprite_loc_32fc8(x_velocity, y_velocity)
            }
        }
        FireballSpriteBranchLoc32F3E::PositiveXNegativeY => {
            fireball_positive_x_negative_y_sprite_loc_32fac(x_velocity, y_velocity)
        }
        FireballSpriteBranchLoc32F3E::PositiveXNonNegativeY => {
            fireball_positive_x_nonnegative_y_sprite_loc_32fe8(x_velocity, y_velocity)
        }
    }
}

#[cfg(test)]
pub(crate) fn fireball_bounce_perturbation(enemy: &ActiveEnemy) -> i32 {
    let mut rng = RomRandomState::default();
    let camera_x_px = enemy.x_pos >> 16;
    fireball_bounce_perturbation_with_rng(&mut rng, camera_x_px)
}

#[cfg(test)]
pub(crate) fn fireball_bounce_y_velocity(enemy: &ActiveEnemy) -> i32 {
    let mut rng = RomRandomState::default();
    let camera_x_px = enemy.x_pos >> 16;
    fireball_bounce_y_velocity_with_rng(enemy, &mut rng, camera_x_px)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    #[test]
    fn fireball_live_step_queues_bounce_sound_on_collision_loc_32c60() {
        let mut grid = terrain::BlockGrid::from_level_layout(&vec![0u8; 20 * 15 * 2], 20, 15);
        for col in 0..20 {
            grid.set_cell_at(col, 10, terrain::Cell::from_be_bytes(0x60, 0x00));
        }

        let fireball = ActiveEnemy {
            family: EnemyFamily::Fireball,
            hit_points: 1,
            x_pos: 0x0123_0000,
            y_pos: (159u32 << 16) + 0xC000,
            x_velocity: 0,
            y_velocity: 0x0003_0000,
            facing: Direction::Left,
            on_ground: false,
            ..mini_hopping_skull_test_enemy()
        };
        let mut enemies = [fireball];
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();
        let mut projectiles = Vec::new();
        let mut sound_effects = Vec::new();

        step_active_enemies_with_rng(
            &mut enemies,
            Some(&grid),
            None,
            None,
            Some(0x0123),
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            0,
            false,
            Some(&mut projectiles),
            &mut rng,
            &mut drilled_blocks,
            Some(&mut sound_effects),
            &mut std::collections::HashMap::new(),
        );

        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::FireballBouncing],
            "ASM Fireball.asm loc_32C60 should play `sfx_Fireball_bouncing` before the collision behaviour dispatch",
        );
        assert_eq!(sound_effects[0].sound_id(), 0x5D);
        assert_eq!(
            projectiles.len(),
            1,
            "ASM Fireball.asm loc_32CE4 should still allocate the obstacle trail child after the bounce cue",
        );
        assert_eq!(
            projectiles[0].kind,
            EnemyProjectileKind::FireballObstacleTrail
        );
    }

    #[test]
    fn fireball_wall_bounce_commits_sprite_direction_after_loc_32c60_dispatch() {
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
            Direction::Left,
            "ASM Fireball.asm loc_32C60/loc_32C86 only negates x_vel, then the same loc_32C1A loop runs loc_32F3E/loc_3304A and commits x_direction from the new velocity"
        );
        assert_eq!(enemy.x_velocity, -0x0001_8000);
    }

    #[test]
    fn fireball_main_loop_loc_32c1a_adds_hp_gravity_before_collision_dispatch() {
        let mut hp1 = fireball_test_enemy();
        hp1.y_velocity = 0x0001_8000;
        assert_eq!(
            fireball_main_loop_y_velocity_loc_32c1a(&hp1),
            0x0001_A000,
            "ASM Fireball.asm loc_32C1A should add the HP1 `$2000` gravity d0 into `y_vel(a3)` before `loc_32C60` dispatch"
        );

        let mut hp2 = fireball_test_enemy();
        hp2.hit_points = 2;
        hp2.y_velocity = 0x0001_8000;
        assert_eq!(
            fireball_main_loop_y_velocity_loc_32c1a(&hp2),
            0x0001_B000,
            "ASM Fireball.asm loc_32C1A should use the HP2 `$3000` gravity selected from `unk_32BA6`"
        );
    }

    #[test]
    fn fireball_main_loop_gravity_wraps_as_long_word_addition_loc_32c1a() {
        let mut enemy = fireball_test_enemy();
        let gravity = bouncer_gravity(&enemy);
        enemy.y_velocity = i32::MAX - (gravity - 1);

        step_bouncer(&mut enemy, None);

        assert_eq!(
            enemy.y_velocity,
            i32::MIN,
            "ASM Fireball.asm loc_32C1A uses ADD.L, so gravity wraps at the signed long-word boundary"
        );
    }

    #[test]
    fn fireball_floor_ceiling_bounce_loc_32ce4_commits_velocity_and_gates_trails() {
        let mut enemy = fireball_test_enemy();
        enemy.x_pos = 0x0123_0000;
        enemy.y_velocity = 0x0003_0000;

        let mut rng = RomRandomState::default();
        let open_slot =
            fireball_floor_ceiling_bounce_loc_32ce4(&enemy, &mut rng, enemy.x_pos >> 16, 4);
        assert_eq!(
            open_slot,
            FireballBounceCommitLoc32CE4 {
                y_velocity: -0x0002_F180,
                spawns_fire_trail: true,
                next_fire_trail_count: 5,
            },
            "ASM Fireball.asm loc_32CE4 should store the rebound d7 into `y_vel(a3)` and allocate `loc_33128` while the trail count is below five"
        );

        let mut rng = RomRandomState::default();
        let full = fireball_floor_ceiling_bounce_loc_32ce4(&enemy, &mut rng, enemy.x_pos >> 16, 5);
        assert_eq!(
            full.next_fire_trail_count, 5,
            "ASM Fireball.asm loc_32CE4 should return without allocation when `Number_of_Fire_Trails` is already five"
        );
        assert!(!full.spawns_fire_trail);
    }

    #[test]
    fn fireball_slope_up_trail_marker_loc_32d7e_accepts_only_slope_up_tiles() {
        assert_eq!(
            fireball_slope_up_trail_marker_loc_32d7e(0x005B, 0x007A, 0x4000),
            Some(FireballSlopeTrailMarkerLoc32D7E {
                x_px: 0x005D,
                y_px: 0x0072,
            }),
            "ASM Fireball.asm loc_32D7E should place the slope-up trail at x+2 and `(y & $FFF0) + $F - (x & $F) - 2`"
        );

        assert_eq!(
            fireball_slope_up_trail_marker_loc_32d7e(0x005B, 0x007A, 0x5000),
            None,
            "ASM Fireball.asm loc_32D7E should return through loc_32D06 unless the sampled tile collision nibble is exactly `$4000`"
        );
    }

    #[test]
    fn fireball_slope_down_trail_marker_loc_32e3c_accepts_only_slope_down_tiles() {
        assert_eq!(
            fireball_slope_down_trail_marker_loc_32e3c(0x005B, 0x007A, 0x5000),
            Some(FireballSlopeDownTrailMarkerLoc32E3C {
                x_px: 0x0059,
                y_px: 0x0079,
            }),
            "ASM Fireball.asm loc_32E3C should place the slope-down trail at x-2 and `(y & $FFF0) + (x & $F) - 2`"
        );

        assert_eq!(
            fireball_slope_down_trail_marker_loc_32e3c(0x005B, 0x007A, 0x4000),
            None,
            "ASM Fireball.asm loc_32E3C should return through loc_32D06 unless the sampled tile collision nibble is exactly `$5000`"
        );
    }

    #[test]
    fn fireball_respawn_marker_loc_32eca_writes_positive_status_slot() {
        assert_eq!(
            fireball_respawn_marker_loc_32eca(0x05),
            FireballRespawnMarkerLoc32ECA {
                index: 5,
                word_value: 0x2168,
            },
            "ASM Fireball.asm loc_32ECA doubles positive `$42(a5)` into an EnemyStatus_Table word offset and writes `$2168`"
        );
    }

    #[test]
    fn fireball_delete_tail_loc_32ed8_jumps_to_delete_current_object() {
        assert_eq!(
            fireball_delete_tail_loc_32ed8(),
            FireballDeleteTailLoc32ED8 {
                deletes_current_object: true,
            },
            "ASM Fireball.asm loc_32ED8 should jump straight to `j_Delete_CurrentObject` after every death-status route"
        );
    }

    #[test]
    fn fireball_trail_init_loc_330d0_seeds_child_from_parent_slots() {
        assert_eq!(
            fireball_trail_init_loc_330d0(0xFF, 0x0123, 0x0456),
            FireballTrailInitLoc330D0 {
                gfx_object_long: 0x0300_0003,
                display_flag_13: 0xFF,
                priority: 0,
                object_meta: 0x19,
                lifetime_frames: 0x96,
                x_direction: 0xFF,
                x_px: 0x0123,
                y_px: 0x0456,
                animation: "stru_330C2",
            },
            "ASM Fireball.asm loc_330D0 should allocate the child object, copy `$44/$46/$48(a5)` into direction/x/y, and seed d3 with `$96`"
        );
    }

    #[test]
    fn fireball_trail_projectile_loc_330d0_copies_nonzero_worker_position() {
        let projectile = fireball_trail_projectile_from_loc_330d0(0xFF, 0x0123, 0x0456);

        assert_eq!(
            (projectile.x_pos, projectile.y_pos),
            (0x0123_0000, 0x0456_0000),
            "ASM Fireball.asm loc_330D0 copies `$46/$48(a5)` into the trail child's x/y words",
        );
        assert_eq!((projectile.x_velocity, projectile.y_velocity), (0, 0));
        assert_eq!(projectile.gravity, 0);
        assert!(!projectile.friendly);
        assert!(!projectile.terrain_kill);
        assert_eq!(projectile.kind, EnemyProjectileKind::FireballTrail);
        assert_eq!(projectile.lifetime, 0x96);
    }

    #[test]
    fn fireball_trail_loop_loc_33110_prioritises_flag_then_countdown() {
        assert_eq!(
            fireball_trail_loop_step_loc_33110(true, 0x96),
            FireballTrailLoopStepLoc33110::Delete,
            "ASM Fireball.asm loc_33110 should take the delete tail when `$19(a3)` is non-zero before touching d3"
        );
        assert_eq!(
            fireball_trail_loop_step_loc_33110(false, 0x96),
            FireballTrailLoopStepLoc33110::Continue {
                lifetime_frames: 0x95,
            },
            "ASM Fireball.asm loc_33110 should decrement d3 after a clear `$19(a3)` and keep hibernating while the result is non-zero"
        );
        assert_eq!(
            fireball_trail_loop_step_loc_33110(false, 1),
            FireballTrailLoopStepLoc33110::Delete,
            "ASM Fireball.asm loc_33110 should take the delete tail when `subq.w #1,d3` produces zero"
        );
        assert_eq!(
            fireball_trail_loop_step_loc_33110(false, 0),
            FireballTrailLoopStepLoc33110::Continue {
                lifetime_frames: 0xFFFF,
            },
            "ASM Fireball.asm loc_33110 uses word subtraction, so an unexpected zero d3 wraps to `$FFFF` and branches back"
        );
    }

    #[test]
    fn fireball_obstacle_trail_loop_loc_331ba_prioritises_flag_then_countdown() {
        assert_eq!(
            fireball_obstacle_trail_loop_step_loc_331ba(true, 0x96),
            FireballObstacleTrailLoopStepLoc331BA::Delete,
            "ASM Fireball.asm loc_331BA should take the delete tail when `$19(a3)` is non-zero before touching d3"
        );
        assert_eq!(
            fireball_obstacle_trail_loop_step_loc_331ba(false, 0x96),
            FireballObstacleTrailLoopStepLoc331BA::Continue {
                lifetime_frames: 0x95,
            },
            "ASM Fireball.asm loc_331BA should decrement d3 after a clear `$19(a3)` and keep hibernating while the result is non-zero"
        );
        assert_eq!(
            fireball_obstacle_trail_loop_step_loc_331ba(false, 1),
            FireballObstacleTrailLoopStepLoc331BA::Delete,
            "ASM Fireball.asm loc_331BA should take the delete tail when `subq.w #1,d3` produces zero"
        );
        assert_eq!(
            fireball_obstacle_trail_loop_step_loc_331ba(false, 0),
            FireballObstacleTrailLoopStepLoc331BA::Continue {
                lifetime_frames: 0xFFFF,
            },
            "ASM Fireball.asm loc_331BA uses word subtraction, so an unexpected zero d3 wraps to `$FFFF` and branches back"
        );
    }

    #[test]
    fn fireball_trail_delete_tail_loc_33120_decrements_global_count() {
        assert_eq!(
            fireball_trail_delete_tail_loc_33120(5),
            FireballTrailDeleteTailLoc33120 {
                next_fire_trail_count: 4,
                deletes_current_object: true,
            },
            "ASM Fireball.asm loc_33120 should decrement Number_of_Fire_Trails before deleting the current object"
        );
        assert_eq!(
            fireball_trail_delete_tail_loc_33120(0),
            FireballTrailDeleteTailLoc33120 {
                next_fire_trail_count: 0xFFFF,
                deletes_current_object: true,
            },
            "ASM Fireball.asm loc_33120 uses word subtraction, so an unexpected zero Number_of_Fire_Trails wraps to `$FFFF`"
        );
    }

    #[test]
    fn fireball_obstacle_trail_delete_tail_loc_331ca_decrements_global_count() {
        assert_eq!(
            fireball_obstacle_trail_delete_tail_loc_331ca(5),
            FireballObstacleTrailDeleteTailLoc331CA {
                next_fire_trail_count: 4,
                deletes_current_object: true,
            },
            "ASM Fireball.asm loc_331CA should decrement Number_of_Fire_Trails before deleting the current object"
        );
        assert_eq!(
            fireball_obstacle_trail_delete_tail_loc_331ca(0),
            FireballObstacleTrailDeleteTailLoc331CA {
                next_fire_trail_count: 0xFFFF,
                deletes_current_object: true,
            },
            "ASM Fireball.asm loc_331CA uses word subtraction, so an unexpected zero Number_of_Fire_Trails wraps to `$FFFF`"
        );
    }

    #[test]
    fn fireball_sprite_branch_loc_32f3e_handles_vertical_and_quadrant_dispatch() {
        assert_eq!(
            fireball_sprite_branch_loc_32f3e(0, -0x0001_0000),
            FireballSpriteBranchLoc32F3E::Vertical { sprite_index: 4 },
            "ASM Fireball.asm loc_32F3E should return sprite index 4 for straight upward motion"
        );
        assert_eq!(
            fireball_sprite_branch_loc_32f3e(0, 0x0001_0000),
            FireballSpriteBranchLoc32F3E::Vertical { sprite_index: 0x0F },
            "ASM Fireball.asm loc_32F3E should return sprite index $F for straight downward motion"
        );
        assert_eq!(
            fireball_sprite_branch_loc_32f3e(0x0001_0000, -0x0000_8000),
            FireballSpriteBranchLoc32F3E::PositiveXNegativeY,
            "ASM Fireball.asm loc_32F3E should route positive X / negative Y into the loc_32FAC branch"
        );
        assert_eq!(
            fireball_sprite_branch_loc_32f3e(0x0001_0000, 0),
            FireballSpriteBranchLoc32F3E::PositiveXNonNegativeY,
            "ASM Fireball.asm loc_32F3E should route positive X / zero Y through the loc_32FE8 branch"
        );
        assert_eq!(
            fireball_sprite_branch_loc_32f3e(-0x0001_0000, -0x0000_8000),
            FireballSpriteBranchLoc32F3E::NegativeX,
            "ASM Fireball.asm loc_32F3E should route negative X into the loc_32F78 branch"
        );
    }

    #[test]
    fn fireball_vertical_sprite_loc_3300a_selects_up_or_down_sprite() {
        assert_eq!(
            fireball_vertical_sprite_loc_3300a(-0x0001_0000),
            4,
            "ASM Fireball.asm loc_3300A should keep sprite 4 when the shifted Y word is negative"
        );
        assert_eq!(
            fireball_vertical_sprite_loc_3300a(0),
            0x0F,
            "ASM Fireball.asm loc_3300A should switch to sprite `$F` when the shifted Y word is zero"
        );
    }

    #[test]
    fn fireball_sprite_link_loc_33072_writes_the_off_33026_entry() {
        assert_eq!(
            fireball_sprite_link_loc_33072(0),
            Some("LnkTo_unk_C78D6-Data_Index"),
            "ASM Fireball.asm loc_33072 should map index 0 through off_33026 before writing addroffset_sprite"
        );
        assert_eq!(
            fireball_sprite_link_loc_33072(17),
            Some("LnkTo_unk_C795E-Data_Index"),
            "ASM Fireball.asm loc_33072 should allow the last animation slot used by loc_3304A's phase toggle"
        );
        assert_eq!(
            fireball_sprite_link_loc_33072(18),
            None,
            "ASM Fireball.asm loc_33072 has no off_33026 entry past the 18 sprite links"
        );
    }

    #[test]
    fn fireball_zero_phase_expiry_loc_33070_sets_d2_for_the_next_write() {
        assert_eq!(
            fireball_zero_phase_expiry_loc_33070(),
            1,
            "ASM Fireball.asm loc_33070 should set d2 to one when the expired phase was zero"
        );
    }

    #[test]
    fn fireball_sprite_commit_loc_3304a_applies_direction_phase_and_countdown() {
        assert_eq!(
            fireball_sprite_commit_loc_3304a(0, 0, 2),
            Some(FireballSpriteCommitLoc3304A {
                x_direction: 0x00,
                sprite_table_index: 8,
                sprite_link: "LnkTo_unk_C7916-Data_Index",
                next_animation_phase: 0,
                next_animation_countdown: 1,
            }),
            "ASM Fireball.asm loc_3304A should use byte_33080/byte_33081 before decrementing the phase countdown"
        );
        assert_eq!(
            fireball_sprite_commit_loc_3304a(0, 0, 1),
            Some(FireballSpriteCommitLoc3304A {
                x_direction: 0x00,
                sprite_table_index: 8,
                sprite_link: "LnkTo_unk_C7916-Data_Index",
                next_animation_phase: 1,
                next_animation_countdown: 5,
            }),
            "ASM Fireball.asm loc_3304A should write the current phase frame before loc_33070 arms d2 for the next frame"
        );
        assert_eq!(
            fireball_sprite_commit_loc_3304a(14, 1, 1),
            Some(FireballSpriteCommitLoc3304A {
                x_direction: 0xFF,
                sprite_table_index: 17,
                sprite_link: "LnkTo_unk_C795E-Data_Index",
                next_animation_phase: 0,
                next_animation_countdown: 5,
            }),
            "ASM Fireball.asm loc_3304A should keep set x_direction from byte_33080 and clear non-zero d2 after expiry"
        );
    }

    #[test]
    fn fireball_ratio_lookup_unk_32edc_matches_table_edges() {
        assert_eq!(
            fireball_ratio_lookup_unk_32edc(0),
            0,
            "ASM Fireball.asm unk_32EDC should start with three 0 entries"
        );
        assert_eq!(
            fireball_ratio_lookup_unk_32edc(2),
            0,
            "ASM Fireball.asm unk_32EDC should end the 0 range at index 2"
        );
        assert_eq!(
            fireball_ratio_lookup_unk_32edc(3),
            1,
            "ASM Fireball.asm unk_32EDC should start the 1 range at index 3"
        );
        assert_eq!(
            fireball_ratio_lookup_unk_32edc(5),
            1,
            "ASM Fireball.asm unk_32EDC should end the 1 range at index 5"
        );
        assert_eq!(
            fireball_ratio_lookup_unk_32edc(6),
            2,
            "ASM Fireball.asm unk_32EDC should start the 2 range at index 6"
        );
        assert_eq!(
            fireball_ratio_lookup_unk_32edc(13),
            2,
            "ASM Fireball.asm unk_32EDC should end the 2 range at index 13"
        );
        assert_eq!(
            fireball_ratio_lookup_unk_32edc(14),
            3,
            "ASM Fireball.asm unk_32EDC should map index 14 and later ratio slots to 3"
        );
        assert_eq!(
            fireball_ratio_lookup_unk_32edc(62),
            3,
            "ASM Fireball.asm loc_32F78/loc_32FAC/loc_32FC8/loc_32FE8 should use unk_32EDC through the last ratio slot before `$3F` fallback"
        );
    }

    #[test]
    fn fireball_negative_x_negative_y_sprite_loc_32f78_mirrors_ratio_table() {
        assert_eq!(
            fireball_negative_x_negative_y_sprite_loc_32f78(-0x0001_0000, -0x0000_8000),
            FireballSpriteSelectionLoc32F3E {
                sprite_index: 8,
                quadrant_index: 1,
            },
            "ASM Fireball.asm loc_32F78 should mirror unk_32EDC[4] = 1 into sprite index 8 and quadrant 1"
        );
        assert_eq!(
            fireball_negative_x_negative_y_sprite_loc_32f78(-0x0000_0100, -0x0001_0000),
            FireballSpriteSelectionLoc32F3E {
                sprite_index: 4,
                quadrant_index: 1,
            },
            "ASM Fireball.asm loc_32F78 should use loc_33018's upward fallback sprite 4 when the ratio is at least $3F"
        );
    }

    #[test]
    fn fireball_positive_x_negative_y_sprite_loc_32fac_uses_ratio_table_directly() {
        assert_eq!(
            fireball_positive_x_negative_y_sprite_loc_32fac(0x0001_0000, -0x0000_8000),
            FireballSpriteSelectionLoc32F3E {
                sprite_index: 1,
                quadrant_index: 0,
            },
            "ASM Fireball.asm loc_32FAC should pass unk_32EDC[4] = 1 through directly and keep quadrant 0"
        );
        assert_eq!(
            fireball_positive_x_negative_y_sprite_loc_32fac(0x0000_0100, -0x0001_0000),
            FireballSpriteSelectionLoc32F3E {
                sprite_index: 4,
                quadrant_index: 0,
            },
            "ASM Fireball.asm loc_32FAC should use loc_33018's upward fallback sprite 4 when the ratio is at least $3F"
        );
    }

    #[test]
    fn fireball_negative_x_nonnegative_y_sprite_loc_32fc8_offsets_ratio_table() {
        assert_eq!(
            fireball_negative_x_nonnegative_y_sprite_loc_32fc8(-0x0001_0000, 0x0000_8000),
            FireballSpriteSelectionLoc32F3E {
                sprite_index: 0x0B,
                quadrant_index: 2,
            },
            "ASM Fireball.asm loc_32FC8 should offset unk_32EDC[4] = 1 by `$A` and use quadrant 2"
        );
        assert_eq!(
            fireball_negative_x_nonnegative_y_sprite_loc_32fc8(-0x0000_0100, 0x0001_0000),
            FireballSpriteSelectionLoc32F3E {
                sprite_index: 0x0F,
                quadrant_index: 2,
            },
            "ASM Fireball.asm loc_32FC8 should branch to loc_33018 before applying the `$A` offset when the ratio is at least $3F"
        );
        assert_eq!(
            fireball_sprite_selection_loc_32f3e(-0x0001_0000, 0),
            FireballSpriteSelectionLoc32F3E {
                sprite_index: 0x0A,
                quadrant_index: 2,
            },
            "ASM Fireball.asm loc_32F78 uses `tst.w d7; bpl.w loc_32FC8`, so zero Y is non-negative for negative-X fireballs"
        );
    }

    #[test]
    fn fireball_positive_x_nonnegative_y_sprite_loc_32fe8_mirrors_ratio_table() {
        assert_eq!(
            fireball_positive_x_nonnegative_y_sprite_loc_32fe8(0x0001_0000, 0x0000_8000),
            FireballSpriteSelectionLoc32F3E {
                sprite_index: 0x12,
                quadrant_index: 3,
            },
            "ASM Fireball.asm loc_32FE8 should mirror unk_32EDC[4] = 1 around `$13` and use quadrant 3"
        );
        assert_eq!(
            fireball_positive_x_nonnegative_y_sprite_loc_32fe8(0x0000_0100, 0x0001_0000),
            FireballSpriteSelectionLoc32F3E {
                sprite_index: 0x0F,
                quadrant_index: 3,
            },
            "ASM Fireball.asm loc_32FE8 should branch to loc_33018 before `neg.w d6; addi.w #$13,d6` when the ratio is at least $3F"
        );
    }
}
