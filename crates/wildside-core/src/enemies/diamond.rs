use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DiamondHighHpInitLoc3AD7C {
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) animation_script: &'static str,
    pub(crate) x_velocity: i32,
}

pub(crate) fn diamond_high_hp_init_loc_3ad7c(
    enemy_x_px: i32,
    player_x_px: i32,
) -> DiamondHighHpInitLoc3AD7C {
    // ASM Diamond.asm loc_3AD7C: high-HP Diamonds enable movement and level
    // collision, initialise `stru_3ACA8`, then seed `x_vel` to `$18000`.
    // `move.w x_pos(a3),d7; sub.w x_pos(a4),d7; bmi.s <collision-loop>`
    // keeps positive velocity only when the Diamond is left of Kid.
    let x_velocity = if enemy_x_px - player_x_px < 0 {
        0x0001_8000
    } else {
        -0x0001_8000
    };

    DiamondHighHpInitLoc3AD7C {
        is_moved: true,
        has_level_collision: true,
        animation_script: "stru_3ACA8",
        x_velocity,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiamondHighHpCollisionRouteLoc3ADA4 {
    Loop,
    FloorLoop,
    Fall { y_velocity: i32 },
    Damage,
    Bounce { x_velocity: i32, y_velocity: i32 },
    Slope { y_velocity: i32 },
    BerzerkerSideGate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DiamondHighHpCollisionDispatchLoc3ADA4 {
    pub(crate) clears_collision_before_poll: bool,
    pub(crate) route: DiamondHighHpCollisionRouteLoc3ADA4,
}

pub(crate) fn diamond_high_hp_collision_dispatch_loc_3ada4(
    collision_type: u16,
    x_velocity: i32,
    y_velocity: i32,
) -> DiamondHighHpCollisionDispatchLoc3ADA4 {
    // ASM Diamond.asm loc_3ADA4: clear `collision_type(a3)`, hibernate one
    // frame, run the shared range check, then dispatch the newly captured
    // collision word. Zero collision starts falling, floor and unknown values
    // loop, wall/ceiling contacts negate both velocities, and slopes choose
    // the next Y velocity from the current X direction.
    let route = match collision_type {
        0 => DiamondHighHpCollisionRouteLoc3ADA4::Fall {
            y_velocity: diamond_high_hp_fall_y_velocity_loc_3ae78(),
        },
        DIAMOND_COLID_HURT => DiamondHighHpCollisionRouteLoc3ADA4::Damage,
        DIAMOND_COLID_FLOOR => DiamondHighHpCollisionRouteLoc3ADA4::FloorLoop,
        DIAMOND_COLID_RIGHTWALL | DIAMOND_COLID_LEFTWALL | DIAMOND_COLID_CEILING => {
            // ASM Diamond.asm loc_3AE3E: wall and ceiling contacts negate both
            // velocity axes before returning to the high-HP collision loop.
            DiamondHighHpCollisionRouteLoc3ADA4::Bounce {
                x_velocity: -x_velocity,
                y_velocity: -y_velocity,
            }
        }
        DIAMOND_COLID_SLOPEUP => DiamondHighHpCollisionRouteLoc3ADA4::Slope {
            // ASM Diamond.asm loc_3AE4A: `tst.l x_vel(a3)` sends negative
            // X velocity to +$18000 Y velocity; zero/non-negative goes upward.
            y_velocity: if x_velocity < 0 {
                diamond_high_hp_slope_y_velocity_loc_3ae5c()
            } else {
                diamond_high_hp_slope_y_velocity_loc_3ae50()
            },
        },
        DIAMOND_COLID_SLOPEDOWN => DiamondHighHpCollisionRouteLoc3ADA4::Slope {
            y_velocity: diamond_high_hp_slopedown_y_velocity_loc_3ae68(x_velocity),
        },
        DIAMOND_COLID_KIDRIGHT | DIAMOND_COLID_KIDLEFT => {
            DiamondHighHpCollisionRouteLoc3ADA4::BerzerkerSideGate
        }
        _ => DiamondHighHpCollisionRouteLoc3ADA4::Loop,
    };

    DiamondHighHpCollisionDispatchLoc3ADA4 {
        clears_collision_before_poll: true,
        route,
    }
}

pub(crate) fn diamond_high_hp_fall_y_velocity_loc_3ae78() -> i32 {
    // ASM Diamond.asm loc_3AE78: zero collision stores `#$1C000` into
    // `y_vel(a3)` before returning to the high-HP collision loop.
    0x0001_C000
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiamondHighHpBerzerkerSideGateRouteLoc3AE84 {
    Loop,
    Launch(DiamondHighHpBerzerkerLaunchLoc3AE84),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DiamondHighHpBerzerkerLaunchLoc3AE84 {
    pub(crate) has_level_collision: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

pub(crate) fn diamond_high_hp_berzerker_side_gate_loc_3ae84(
    berzerker_charging: bool,
) -> DiamondHighHpBerzerkerSideGateRouteLoc3AE84 {
    // ASM Diamond.asm loc_3AE84: Kid side contacts loop unless
    // `(Berzerker_charging).w` is set. A charging Berzerker turns off terrain
    // collision, keeps Kid collision enabled, freezes animation on
    // `LnkTo_unk_C8160`, zeroes X velocity, and launches upward at `-$48000`.
    if !berzerker_charging {
        return DiamondHighHpBerzerkerSideGateRouteLoc3AE84::Loop;
    }

    DiamondHighHpBerzerkerSideGateRouteLoc3AE84::Launch(DiamondHighHpBerzerkerLaunchLoc3AE84 {
        has_level_collision: false,
        has_kid_collision: true,
        is_moved: true,
        is_animated: false,
        sprite_mapping: "LnkTo_unk_C8160-Data_Index",
        x_velocity: 0,
        y_velocity: -0x0004_8000,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DiamondHighHpBerzerkerLaunchTickLoc3AEB2 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shared_retention_check_sub_3c3ce: bool,
    pub(crate) next_y_velocity: i32,
    pub(crate) loops: bool,
}

pub(crate) fn diamond_high_hp_berzerker_launch_tick_loc_3aeb2(
    y_velocity: i32,
) -> DiamondHighHpBerzerkerLaunchTickLoc3AEB2 {
    // ASM Diamond.asm loc_3AEB2: each airborne tick hibernates one frame,
    // runs `sub_3C3CE`, then `addi.l #$4000,y_vel(a3)` before looping.
    DiamondHighHpBerzerkerLaunchTickLoc3AEB2 {
        hibernates_one_frame: true,
        runs_shared_retention_check_sub_3c3ce: true,
        next_y_velocity: y_velocity.wrapping_add(0x0000_4000),
        loops: true,
    }
}

pub(crate) fn diamond_high_hp_slope_y_velocity_loc_3ae50() -> i32 {
    // ASM Diamond.asm loc_3AE50: `move.l #-$18000,y_vel(a3)` sends the
    // high-HP Diamond upward along the slope before returning to loc_3ADA4.
    -0x0001_8000
}

pub(crate) fn diamond_high_hp_slope_y_velocity_loc_3ae5c() -> i32 {
    // ASM Diamond.asm loc_3AE5C: `move.l #$18000,y_vel(a3)` sends the
    // high-HP Diamond downward along the slope before returning to loc_3ADA4.
    0x0001_8000
}

pub(crate) fn diamond_high_hp_slopedown_y_velocity_loc_3ae68(x_velocity: i32) -> i32 {
    // ASM Diamond.asm loc_3AE68: `tst.l x_vel(a3); bmi.s loc_3AE50; bra.s
    // loc_3AE5C`. Down-slope contact inverts the slope-up sign mapping.
    if x_velocity < 0 {
        diamond_high_hp_slope_y_velocity_loc_3ae50()
    } else {
        diamond_high_hp_slope_y_velocity_loc_3ae5c()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiamondHighHpDamageRouteLoc3ADFA {
    HurtAnimation,
    DeathAnimation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DiamondHighHpDamageLoc3ADFA {
    pub(crate) remaining_hitpoints: u16,
    pub(crate) is_moved_during_response: bool,
    pub(crate) animation_script: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) is_moved_after_response: bool,
    pub(crate) restored_animation_script: Option<&'static str>,
    pub(crate) next_collision_type: u16,
    pub(crate) route: DiamondHighHpDamageRouteLoc3ADFA,
}

pub(crate) fn diamond_high_hp_damage_loc_3adfa(current_hp: u16) -> DiamondHighHpDamageLoc3ADFA {
    // ASM Diamond.asm loc_3ADFA: `subq.w #1,$44(a3)` chooses death only when
    // the decrement leaves exactly zero. Surviving Diamonds stop movement for
    // `stru_3ACBE`, wait for the animation to finish, re-enable movement,
    // clear `collision_type(a3)`, and restore `stru_3ACA8`.
    let remaining_hitpoints = current_hp.wrapping_sub(1);
    if remaining_hitpoints == 0 {
        // ASM Diamond.asm loc_3AE28: final hit stops movement, runs
        // `stru_3ACC4` to completion, then branches to shared object deletion.
        return DiamondHighHpDamageLoc3ADFA {
            remaining_hitpoints,
            is_moved_during_response: false,
            animation_script: "stru_3ACC4",
            waits_until_animation_finished: true,
            is_moved_after_response: false,
            restored_animation_script: None,
            next_collision_type: 0,
            route: DiamondHighHpDamageRouteLoc3ADFA::DeathAnimation,
        };
    }

    DiamondHighHpDamageLoc3ADFA {
        remaining_hitpoints,
        is_moved_during_response: false,
        animation_script: "stru_3ACBE",
        waits_until_animation_finished: true,
        is_moved_after_response: true,
        restored_animation_script: Some("stru_3ACA8"),
        next_collision_type: 0,
        route: DiamondHighHpDamageRouteLoc3ADFA::HurtAnimation,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DiamondLowHpDeathAnimationLoc3AEE6 {
    pub(crate) animation_script: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) deletes_enemy: bool,
}

pub(crate) fn diamond_low_hp_death_animation_loc_3aee6() -> DiamondLowHpDeathAnimationLoc3AEE6 {
    // ASM Diamond.asm loc_3AEE6: initialise `stru_3ACC4`, hibernate until the
    // death animation finishes, then branch to shared enemy deletion.
    DiamondLowHpDeathAnimationLoc3AEE6 {
        animation_script: "stru_3ACC4",
        waits_until_animation_finished: true,
        deletes_enemy: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DiamondLowHpHurtAnimationLoc3AEF8 {
    pub(crate) hurt_animation_script: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) next_collision_type: u16,
    pub(crate) restored_animation_script: &'static str,
    pub(crate) falls_through_to_shared_walker: bool,
}

pub(crate) fn diamond_low_hp_hurt_animation_loc_3aef8() -> DiamondLowHpHurtAnimationLoc3AEF8 {
    // ASM Diamond.asm loc_3AEF8: play `stru_3ACBE` to completion, clear
    // `collision_type(a3)`, restore the base `stru_3ACA8` animation, then fall
    // through to the shared walker clear/resume tail.
    DiamondLowHpHurtAnimationLoc3AEF8 {
        hurt_animation_script: "stru_3ACBE",
        waits_until_animation_finished: true,
        next_collision_type: 0,
        restored_animation_script: "stru_3ACA8",
        falls_through_to_shared_walker: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiamondLowHpSharedWalkerNextLoc3AF14 {
    DefaultDirectionRoutineLoc3C0FA,
    FlippedDirectionRoutineLoc3C04E,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DiamondLowHpSharedWalkerResumeLoc3AF14 {
    pub(crate) next_collision_type: u16,
    pub(crate) next_walker: DiamondLowHpSharedWalkerNextLoc3AF14,
}

pub(crate) fn diamond_low_hp_shared_walker_resume_loc_3af14(
    x_direction_flipped: bool,
) -> DiamondLowHpSharedWalkerResumeLoc3AF14 {
    // ASM Diamond.asm loc_3AF14: `clr.w collision_type(a3)`, then
    // `tst.b x_direction(a3)` selects the shared direction-specific walker.
    DiamondLowHpSharedWalkerResumeLoc3AF14 {
        next_collision_type: 0,
        next_walker: if x_direction_flipped {
            DiamondLowHpSharedWalkerNextLoc3AF14::FlippedDirectionRoutineLoc3C04E
        } else {
            DiamondLowHpSharedWalkerNextLoc3AF14::DefaultDirectionRoutineLoc3C0FA
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiamondLowHpBerzerkerSideGateRouteLoc3AF24 {
    ResumeSharedWalker(DiamondLowHpSharedWalkerResumeLoc3AF14),
    Launch(DiamondLowHpBerzerkerLaunchLoc3AF24),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DiamondLowHpBerzerkerLaunchLoc3AF24 {
    pub(crate) has_level_collision: bool,
    pub(crate) has_kid_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) is_animated: bool,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

pub(crate) fn diamond_low_hp_berzerker_side_gate_loc_3af24(
    berzerker_charging: bool,
    x_direction_flipped: bool,
) -> DiamondLowHpBerzerkerSideGateRouteLoc3AF24 {
    // ASM Diamond.asm loc_3AF24: clear `Berzerker_charging` branches back to
    // loc_3AF14. A charging Berzerker disables level collision, keeps Kid
    // collision and movement enabled, freezes animation on `LnkTo_unk_C8160`,
    // zeroes X velocity, and launches the Diamond upward at `-$48000`.
    if !berzerker_charging {
        return DiamondLowHpBerzerkerSideGateRouteLoc3AF24::ResumeSharedWalker(
            diamond_low_hp_shared_walker_resume_loc_3af14(x_direction_flipped),
        );
    }

    DiamondLowHpBerzerkerSideGateRouteLoc3AF24::Launch(DiamondLowHpBerzerkerLaunchLoc3AF24 {
        has_level_collision: false,
        has_kid_collision: true,
        is_moved: true,
        is_animated: false,
        sprite_mapping: "LnkTo_unk_C8160-Data_Index",
        x_velocity: 0,
        y_velocity: -0x0004_8000,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DiamondLowHpBerzerkerLaunchTickLoc3AF50 {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) runs_shared_retention_check_sub_3c3ce: bool,
    pub(crate) next_y_velocity: i32,
    pub(crate) loops: bool,
}

pub(crate) fn diamond_low_hp_berzerker_launch_tick_loc_3af50(
    y_velocity: i32,
) -> DiamondLowHpBerzerkerLaunchTickLoc3AF50 {
    // ASM Diamond.asm loc_3AF50: each airborne tick hibernates one frame,
    // runs `sub_3C3CE`, then `addi.l #$4000,y_vel(a3)` before looping.
    DiamondLowHpBerzerkerLaunchTickLoc3AF50 {
        hibernates_one_frame: true,
        runs_shared_retention_check_sub_3c3ce: true,
        next_y_velocity: y_velocity.wrapping_add(0x0000_4000),
        loops: true,
    }
}

pub(crate) fn step_diamond_high_hp(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) {
    // ASM Diamond.asm loc_3ADA4 hibernates one frame, runs sub_3C3CE, then
    // dispatches the collision word captured by the level-collision pass.
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    let collision_type = diamond_high_hp_collision_type(enemy, block_grid);
    match diamond_high_hp_collision_dispatch_loc_3ada4(
        collision_type,
        enemy.x_velocity,
        enemy.y_velocity,
    )
    .route
    {
        DiamondHighHpCollisionRouteLoc3ADA4::Loop
        | DiamondHighHpCollisionRouteLoc3ADA4::FloorLoop => {}
        DiamondHighHpCollisionRouteLoc3ADA4::Fall { y_velocity }
        | DiamondHighHpCollisionRouteLoc3ADA4::Slope { y_velocity } => {
            enemy.y_velocity = y_velocity;
        }
        DiamondHighHpCollisionRouteLoc3ADA4::Bounce {
            x_velocity,
            y_velocity,
        } => {
            enemy.x_velocity = x_velocity;
            enemy.y_velocity = y_velocity;
        }
        // ASM Diamond.asm loc_3ADFA and loc_3AE84 require the live Kid/enemy
        // collision word, which is not plumbed through this terrain probe yet.
        DiamondHighHpCollisionRouteLoc3ADA4::Damage
        | DiamondHighHpCollisionRouteLoc3ADA4::BerzerkerSideGate => {}
    }
}

pub(crate) fn step_diamond_high_hp_berzerker_launch(enemy: &mut ActiveEnemy) {
    // ASM Diamond.asm loc_3AEB2: move the launched Diamond with its current
    // y velocity, hibernate/run `sub_3C3CE`, then add `$4000` gravity.
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    let tick = diamond_high_hp_berzerker_launch_tick_loc_3aeb2(enemy.y_velocity);
    enemy.y_velocity = tick.next_y_velocity;
}

pub(crate) fn step_diamond_low_hp_berzerker_launch(enemy: &mut ActiveEnemy) {
    // ASM Diamond.asm loc_3AF50 mirrors loc_3AEB2 for the low-HP shared-walker
    // Diamond path after charging-Berzerker side contact.
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    let tick = diamond_low_hp_berzerker_launch_tick_loc_3af50(enemy.y_velocity);
    enemy.y_velocity = tick.next_y_velocity;
}

fn diamond_high_hp_collision_type(
    enemy: &ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) -> u16 {
    let Some(grid) = block_grid else {
        return 0;
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
        return if moving_right {
            DIAMOND_COLID_RIGHTWALL
        } else {
            DIAMOND_COLID_LEFTWALL
        };
    }

    if enemy.y_velocity < 0 {
        let ceiling =
            terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
        if ceiling.hit {
            return DIAMOND_COLID_CEILING;
        }
    }

    let floor = terrain::probe_floor(grid, cx, cy);
    if floor.on_ground && enemy.y_velocity >= 0 {
        return match floor.cell.collision {
            // ASM Diamond.asm loc_3ADA4 keeps slope collision words distinct:
            // loc_3AE4A/loc_3AE68 select the next y_vel from x_vel's sign.
            terrain::CollisionKind::SlopeUp => DIAMOND_COLID_SLOPEUP,
            terrain::CollisionKind::SlopeDown => DIAMOND_COLID_SLOPEDOWN,
            _ => DIAMOND_COLID_FLOOR,
        };
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    fn diamond_test_enemy(x_px: u32, y_px: u32, x_velocity: i32, y_velocity: i32) -> ActiveEnemy {
        ActiveEnemy {
            family: EnemyFamily::Diamond,
            hit_points: 4,
            x_pos: x_px << 16,
            y_pos: y_px << 16,
            x_velocity,
            y_velocity,
            ..mini_hopping_skull_test_enemy()
        }
    }

    fn diamond_grid_with_solid_cells(cells: &[(u16, u16)]) -> terrain::BlockGrid {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 8 * 8 * 2], 8, 8);
        for &(col, row) in cells {
            grid.set_cell_at(col, row, terrain::Cell::from_be_bytes(0x60, 0x00));
        }
        grid
    }

    fn diamond_grid_with_cells(cells: &[(u16, u16, terrain::Cell)]) -> terrain::BlockGrid {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 8 * 8 * 2], 8, 8);
        for &(col, row, cell) in cells {
            grid.set_cell_at(col, row, cell);
        }
        grid
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
    fn diamond_high_hp_init_loc_3ad7c_enables_movement_and_targets_kid() {
        assert_eq!(
            diamond_high_hp_init_loc_3ad7c(80, 112),
            DiamondHighHpInitLoc3AD7C {
                is_moved: true,
                has_level_collision: true,
                animation_script: "stru_3ACA8",
                x_velocity: 0x0001_8000,
            },
            "ASM Diamond.asm loc_3AD7C should keep +$18000 x_vel when enemy_x - Kid_x is negative",
        );
        assert_eq!(
            diamond_high_hp_init_loc_3ad7c(112, 80).x_velocity,
            -0x0001_8000,
            "ASM Diamond.asm loc_3AD7C should negate x_vel when the Diamond is right of or aligned with Kid",
        );
        assert_eq!(
            diamond_high_hp_init_loc_3ad7c(96, 96).x_velocity,
            -0x0001_8000,
            "ASM Diamond.asm loc_3AD7C uses `bmi`, so exact alignment takes the negate path",
        );
    }

    #[test]
    fn diamond_high_hp_runtime_uses_loc_3ad7c_collision_loop_instead_of_shared_walker() {
        let mut diamond = diamond_test_enemy(
            80,
            112,
            diamond_high_hp_init_loc_3ad7c(80, 112).x_velocity,
            0,
        );
        let mut enemies = [diamond];
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();

        step_active_enemies_with_rng(
            &mut enemies,
            None,
            Some(112 << 16),
            Some(112 << 16),
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
            None,
            &mut std::collections::HashSet::new(),
        );
        diamond = enemies[0];

        assert_eq!(
            diamond.x_velocity, 0x0001_8000,
            "ASM Diamond.asm loc_3AD7C should keep the high-HP seeded X velocity instead of entering the low-HP shared walker",
        );
        assert_eq!(
            diamond.x_pos,
            (80 << 16) + 0x0001_8000,
            "ASM Diamond.asm loc_3ADA4 hibernates then resumes the collision loop with high-HP horizontal motion",
        );
        assert_eq!(
            diamond.y_velocity, 0x0001_C000,
            "ASM Diamond.asm loc_3AE78 should store +$1C000 when loc_3ADA4 observes no collision",
        );
    }

    #[test]
    fn diamond_high_hp_step_loc_3ada4_loops_on_floor_without_fall_velocity() {
        let grid = diamond_grid_with_solid_cells(&[(5, 7)]);
        let mut diamond = diamond_test_enemy(80, 112, 0x0001_8000, 0x0000_4000);

        step_diamond_high_hp(&mut diamond, Some(&grid));

        assert_eq!(
            diamond.x_pos,
            (80 << 16) + 0x0001_8000,
            "ASM Diamond.asm loc_3ADA4 should move the high-HP Diamond before polling collision"
        );
        assert_eq!(
            diamond.y_pos,
            (112 << 16) + 0x0000_4000,
            "ASM Diamond.asm loc_3ADA4 should move the high-HP Diamond vertically before polling collision"
        );
        assert_eq!(
            diamond.y_velocity, 0x0000_4000,
            "ASM Diamond.asm loc_3ADA4 should loop on colid_floor instead of taking loc_3AE78's fall velocity"
        );
    }

    #[test]
    fn diamond_high_hp_berzerker_launch_step_loc_3aeb2_moves_then_adds_gravity() {
        let mut diamond = diamond_test_enemy(80, 112, 0, -0x0004_8000);

        step_diamond_high_hp_berzerker_launch(&mut diamond);

        assert_eq!(
            diamond.y_pos,
            ((112 << 16_i32) - 0x0004_8000) as u32,
            "ASM Diamond.asm loc_3AEB2 should move by the old y_vel before adding gravity"
        );
        assert_eq!(
            diamond.y_velocity, -0x0004_4000,
            "ASM Diamond.asm loc_3AEB2 should add +$4000 to y_vel after movement"
        );
    }

    #[test]
    fn diamond_low_hp_berzerker_launch_step_loc_3af50_moves_then_adds_gravity() {
        let mut diamond = diamond_test_enemy(80, 112, 0, -0x0004_8000);

        step_diamond_low_hp_berzerker_launch(&mut diamond);

        assert_eq!(
            diamond.y_pos,
            ((112 << 16_i32) - 0x0004_8000) as u32,
            "ASM Diamond.asm loc_3AF50 should move by the old y_vel before adding gravity"
        );
        assert_eq!(
            diamond.y_velocity, -0x0004_4000,
            "ASM Diamond.asm loc_3AF50 should add +$4000 to y_vel after movement"
        );
    }

    #[test]
    fn diamond_high_hp_collision_type_maps_terrain_to_loc_3ada4_collision_words() {
        assert_eq!(
            diamond_high_hp_collision_type(&diamond_test_enemy(80, 112, 0x0001_8000, 0), None),
            0,
            "ASM Diamond.asm loc_3ADA4 sees zero collision when no terrain probe is available"
        );

        let right_wall_grid = diamond_grid_with_solid_cells(&[(5, 6)]);
        assert_eq!(
            diamond_high_hp_collision_type(
                &diamond_test_enemy(80, 112, 0x0001_8000, 0),
                Some(&right_wall_grid),
            ),
            DIAMOND_COLID_RIGHTWALL,
            "ASM Diamond.asm loc_3ADA4 should route a right lead-cell hit to colid_rightwall"
        );

        let left_wall_grid = diamond_grid_with_solid_cells(&[(4, 6)]);
        assert_eq!(
            diamond_high_hp_collision_type(
                &diamond_test_enemy(80, 112, -0x0001_8000, 0),
                Some(&left_wall_grid),
            ),
            DIAMOND_COLID_LEFTWALL,
            "ASM Diamond.asm loc_3ADA4 should route a left lead-cell hit to colid_leftwall"
        );

        let floor_grid = diamond_grid_with_solid_cells(&[(5, 7)]);
        assert_eq!(
            diamond_high_hp_collision_type(
                &diamond_test_enemy(80, 112, 0x0001_8000, 0),
                Some(&floor_grid),
            ),
            DIAMOND_COLID_FLOOR,
            "ASM Diamond.asm loc_3ADA4 should route solid support while descending or level to colid_floor"
        );
        assert_eq!(
            diamond_high_hp_collision_type(
                &diamond_test_enemy(80, 112, 0x0001_8000, -0x0001_0000),
                Some(&floor_grid),
            ),
            0,
            "ASM Diamond.asm loc_3ADA4 should not report floor contact while the high-HP Diamond is rising"
        );
    }

    #[test]
    fn diamond_high_hp_collision_type_maps_ceiling_and_slopes_loc_3ada4() {
        let ceiling_grid = diamond_grid_with_solid_cells(&[(1, 2)]);
        assert_eq!(
            diamond_high_hp_collision_type(
                &diamond_test_enemy(32, 48, 0x0001_8000, -0x0001_0000),
                Some(&ceiling_grid),
            ),
            DIAMOND_COLID_CEILING,
            "ASM Diamond.asm loc_3ADA4 dispatches colid_ceiling to loc_3AE3E after the vertical collision pass"
        );
        assert_eq!(
            diamond_high_hp_collision_type(
                &diamond_test_enemy(32, 48, 0x0001_8000, 0),
                Some(&ceiling_grid),
            ),
            0,
            "ASM kid.asm loc_3C2D0 branches zero vertical movement through the non-negative path, so loc_3ADA4 should not see colid_ceiling",
        );

        let slope_up_grid =
            diamond_grid_with_cells(&[(2, 3, terrain::Cell::from_be_bytes(0x40, 0x00))]);
        assert_eq!(
            diamond_high_hp_collision_type(
                &diamond_test_enemy(40, 55, 0x0001_8000, 0),
                Some(&slope_up_grid),
            ),
            DIAMOND_COLID_SLOPEUP,
            "ASM Diamond.asm loc_3ADA4 must preserve colid_slopeup for loc_3AE4A's x-velocity sign branch"
        );

        let slope_down_grid =
            diamond_grid_with_cells(&[(2, 3, terrain::Cell::from_be_bytes(0x50, 0x00))]);
        assert_eq!(
            diamond_high_hp_collision_type(
                &diamond_test_enemy(40, 56, 0x0001_8000, 0),
                Some(&slope_down_grid),
            ),
            DIAMOND_COLID_SLOPEDOWN,
            "ASM Diamond.asm loc_3ADA4 must preserve colid_slopedown for loc_3AE68's x-velocity sign branch"
        );
    }

    #[test]
    fn diamond_high_hp_live_step_applies_ceiling_and_slope_dispatch_loc_3ada4() {
        let ceiling_grid = diamond_grid_with_solid_cells(&[(1, 1)]);
        let mut ceiling_diamond = diamond_test_enemy(32, 48, 0, -0x0000_8000);

        step_diamond_high_hp(&mut ceiling_diamond, Some(&ceiling_grid));

        assert_eq!(
            ceiling_diamond.y_velocity, 0x0000_8000,
            "ASM Diamond.asm loc_3AE3E should negate y_vel after loc_3ADA4 sees colid_ceiling"
        );

        let slope_up_grid =
            diamond_grid_with_cells(&[(2, 3, terrain::Cell::from_be_bytes(0x40, 0x00))]);
        let mut slope_diamond = diamond_test_enemy(40, 55, 0, 0);

        step_diamond_high_hp(&mut slope_diamond, Some(&slope_up_grid));

        assert_eq!(
            slope_diamond.y_velocity, -0x0001_8000,
            "ASM Diamond.asm loc_3AE4A should set -$18000 y_vel for colid_slopeup with non-negative x_vel"
        );
    }

    #[test]
    fn diamond_high_hp_collision_dispatch_loc_3ada4_routes_collision_words() {
        assert_eq!(
            diamond_high_hp_collision_dispatch_loc_3ada4(0, 0x0001_8000, -0x0000_8000),
            DiamondHighHpCollisionDispatchLoc3ADA4 {
                clears_collision_before_poll: true,
                route: DiamondHighHpCollisionRouteLoc3ADA4::Fall {
                    y_velocity: 0x0001_C000,
                },
            },
            "ASM Diamond.asm loc_3ADA4 should clear collision_type before polling and treat zero collision as the falling path",
        );
        assert_eq!(
            diamond_high_hp_collision_dispatch_loc_3ada4(
                DIAMOND_COLID_RIGHTWALL,
                0x0001_8000,
                -0x0000_4000,
            )
            .route,
            DiamondHighHpCollisionRouteLoc3ADA4::Bounce {
                x_velocity: -0x0001_8000,
                y_velocity: 0x0000_4000,
            },
            "ASM Diamond.asm loc_3ADA4 should negate both velocities for wall or ceiling contacts",
        );
        assert_eq!(
            diamond_high_hp_collision_dispatch_loc_3ada4(DIAMOND_COLID_SLOPEUP, -0x0001_8000, 0,)
                .route,
            DiamondHighHpCollisionRouteLoc3ADA4::Slope {
                y_velocity: 0x0001_8000,
            },
            "ASM Diamond.asm loc_3ADA4 should use +$18000 y_vel for an up-slope while moving left",
        );
        assert_eq!(
            diamond_high_hp_collision_dispatch_loc_3ada4(DIAMOND_COLID_SLOPEDOWN, 0x0001_8000, 0,)
                .route,
            DiamondHighHpCollisionRouteLoc3ADA4::Slope {
                y_velocity: 0x0001_8000,
            },
            "ASM Diamond.asm loc_3ADA4 should use +$18000 y_vel for a down-slope while moving right",
        );
        assert_eq!(
            diamond_high_hp_collision_dispatch_loc_3ada4(DIAMOND_COLID_FLOOR, 0, 0).route,
            DiamondHighHpCollisionRouteLoc3ADA4::FloorLoop,
            "ASM Diamond.asm loc_3ADA4 should loop on floor contact",
        );
        assert_eq!(
            diamond_high_hp_collision_dispatch_loc_3ada4(DIAMOND_COLID_HURT, 0, 0).route,
            DiamondHighHpCollisionRouteLoc3ADA4::Damage,
            "ASM Diamond.asm loc_3ADA4 should route hurt collision to the HP-damage path",
        );
        assert_eq!(
            diamond_high_hp_collision_dispatch_loc_3ada4(DIAMOND_COLID_KIDLEFT, 0, 0).route,
            DiamondHighHpCollisionRouteLoc3ADA4::BerzerkerSideGate,
            "ASM Diamond.asm loc_3ADA4 should route Kid side contacts to the Berzerker gate",
        );
    }

    #[test]
    fn diamond_high_hp_fall_y_velocity_loc_3ae78_is_one_and_three_quarter_pixels() {
        assert_eq!(
            diamond_high_hp_fall_y_velocity_loc_3ae78(),
            0x0001_C000,
            "ASM Diamond.asm loc_3AE78 should store +$1C000 into y_vel when no collision is reported",
        );
    }

    #[test]
    fn diamond_high_hp_berzerker_side_gate_loc_3ae84_requires_charge() {
        assert_eq!(
            diamond_high_hp_berzerker_side_gate_loc_3ae84(false),
            DiamondHighHpBerzerkerSideGateRouteLoc3AE84::Loop,
            "ASM Diamond.asm loc_3AE84 should return to the collision loop when Berzerker_charging is clear",
        );
        assert_eq!(
            diamond_high_hp_berzerker_side_gate_loc_3ae84(true),
            DiamondHighHpBerzerkerSideGateRouteLoc3AE84::Launch(
                DiamondHighHpBerzerkerLaunchLoc3AE84 {
                    has_level_collision: false,
                    has_kid_collision: true,
                    is_moved: true,
                    is_animated: false,
                    sprite_mapping: "LnkTo_unk_C8160-Data_Index",
                    x_velocity: 0,
                    y_velocity: -0x0004_8000,
                },
            ),
            "ASM Diamond.asm loc_3AE84 should set the charging Berzerker launch flags and velocities",
        );
    }

    #[test]
    fn diamond_high_hp_berzerker_launch_tick_loc_3aeb2_applies_gravity_and_loops() {
        assert_eq!(
            diamond_high_hp_berzerker_launch_tick_loc_3aeb2(-0x0004_8000),
            DiamondHighHpBerzerkerLaunchTickLoc3AEB2 {
                hibernates_one_frame: true,
                runs_shared_retention_check_sub_3c3ce: true,
                next_y_velocity: -0x0004_4000,
                loops: true,
            },
            "ASM Diamond.asm loc_3AEB2 should add +$4000 to y_vel after the shared retention check",
        );
    }

    #[test]
    fn diamond_high_hp_berzerker_launch_tick_loc_3aeb2_wraps_like_addi_l() {
        assert_eq!(
            diamond_high_hp_berzerker_launch_tick_loc_3aeb2(i32::MAX - 0x1FFF).next_y_velocity,
            i32::MIN + 0x2000,
            "ASM Diamond.asm loc_3AEB2 uses 68k longword addition, so native keeps wrapping semantics",
        );
    }

    #[test]
    fn diamond_high_hp_bounce_loc_3ae3e_negates_x_and_y_velocity() {
        for collision_type in [
            DIAMOND_COLID_RIGHTWALL,
            DIAMOND_COLID_LEFTWALL,
            DIAMOND_COLID_CEILING,
        ] {
            assert_eq!(
                diamond_high_hp_collision_dispatch_loc_3ada4(
                    collision_type,
                    0x0001_8000,
                    -0x0000_4000,
                )
                .route,
                DiamondHighHpCollisionRouteLoc3ADA4::Bounce {
                    x_velocity: -0x0001_8000,
                    y_velocity: 0x0000_4000,
                },
                "ASM Diamond.asm loc_3AE3E should negate both x_vel and y_vel for wall/ceiling collision {collision_type:#06X}",
            );
        }
    }

    #[test]
    fn diamond_high_hp_slope_up_loc_3ae4a_uses_x_velocity_sign() {
        assert_eq!(
            diamond_high_hp_collision_dispatch_loc_3ada4(DIAMOND_COLID_SLOPEUP, -0x0001_8000, 0)
                .route,
            DiamondHighHpCollisionRouteLoc3ADA4::Slope {
                y_velocity: 0x0001_8000,
            },
            "ASM Diamond.asm loc_3AE4A should set +$18000 y_vel when x_vel is negative",
        );
        assert_eq!(
            diamond_high_hp_collision_dispatch_loc_3ada4(DIAMOND_COLID_SLOPEUP, 0, 0).route,
            DiamondHighHpCollisionRouteLoc3ADA4::Slope {
                y_velocity: -0x0001_8000,
            },
            "ASM Diamond.asm loc_3AE4A uses `bmi`, so x_vel zero takes the non-negative path",
        );
    }

    #[test]
    fn diamond_high_hp_slope_y_velocity_loc_3ae50_is_negative() {
        assert_eq!(
            diamond_high_hp_slope_y_velocity_loc_3ae50(),
            -0x0001_8000,
            "ASM Diamond.asm loc_3AE50 should store -$18000 into y_vel",
        );
    }

    #[test]
    fn diamond_high_hp_slope_y_velocity_loc_3ae5c_is_positive() {
        assert_eq!(
            diamond_high_hp_slope_y_velocity_loc_3ae5c(),
            0x0001_8000,
            "ASM Diamond.asm loc_3AE5C should store +$18000 into y_vel",
        );
    }

    #[test]
    fn diamond_high_hp_slopedown_loc_3ae68_selects_by_x_velocity_sign() {
        assert_eq!(
            diamond_high_hp_slopedown_y_velocity_loc_3ae68(-0x0001_8000),
            -0x0001_8000,
            "ASM Diamond.asm loc_3AE68 should branch to loc_3AE50 when x_vel is negative",
        );
        assert_eq!(
            diamond_high_hp_slopedown_y_velocity_loc_3ae68(0),
            0x0001_8000,
            "ASM Diamond.asm loc_3AE68 uses `bmi`, so zero x_vel branches to loc_3AE5C",
        );
        assert_eq!(
            diamond_high_hp_slopedown_y_velocity_loc_3ae68(0x0001_8000),
            0x0001_8000,
            "ASM Diamond.asm loc_3AE68 should branch to loc_3AE5C when x_vel is positive",
        );
    }

    #[test]
    fn diamond_high_hp_damage_loc_3adfa_decrements_hp_and_restores_motion() {
        assert_eq!(
            diamond_high_hp_damage_loc_3adfa(2),
            DiamondHighHpDamageLoc3ADFA {
                remaining_hitpoints: 1,
                is_moved_during_response: false,
                animation_script: "stru_3ACBE",
                waits_until_animation_finished: true,
                is_moved_after_response: true,
                restored_animation_script: Some("stru_3ACA8"),
                next_collision_type: 0,
                route: DiamondHighHpDamageRouteLoc3ADFA::HurtAnimation,
            },
            "ASM Diamond.asm loc_3ADFA should stop movement for the hurt animation, then restore movement, clear collision_type, and restore the base animation when HP remains",
        );
        assert_eq!(
            diamond_high_hp_damage_loc_3adfa(1),
            DiamondHighHpDamageLoc3ADFA {
                remaining_hitpoints: 0,
                is_moved_during_response: false,
                animation_script: "stru_3ACC4",
                waits_until_animation_finished: true,
                is_moved_after_response: false,
                restored_animation_script: None,
                next_collision_type: 0,
                route: DiamondHighHpDamageRouteLoc3ADFA::DeathAnimation,
            },
            "ASM Diamond.asm loc_3ADFA should choose the death animation when the HP decrement reaches zero",
        );
    }

    #[test]
    fn diamond_high_hp_death_animation_loc_3ae28_stops_movement_until_delete() {
        assert_eq!(
            diamond_high_hp_damage_loc_3adfa(1),
            DiamondHighHpDamageLoc3ADFA {
                remaining_hitpoints: 0,
                is_moved_during_response: false,
                animation_script: "stru_3ACC4",
                waits_until_animation_finished: true,
                is_moved_after_response: false,
                restored_animation_script: None,
                next_collision_type: 0,
                route: DiamondHighHpDamageRouteLoc3ADFA::DeathAnimation,
            },
            "ASM Diamond.asm loc_3AE28 should `sf is_moved`, wait on `stru_3ACC4`, then branch to loc_3C46E deletion",
        );
    }

    #[test]
    fn diamond_low_hp_collision_dispatch_loc_3aec4_routes_collision_words() {
        assert_eq!(
            diamond_low_hp_collision_dispatch_loc_3aec4(0, 2),
            DiamondLowHpCollisionRouteLoc3AEC4::ResumeSharedWalker {
                next_collision_type: 0,
            },
            "ASM Diamond.asm loc_3AEC4 should send zero collision_type to the shared walker clear tail",
        );
        assert_eq!(
            diamond_low_hp_collision_dispatch_loc_3aec4(-1, 2),
            DiamondLowHpCollisionRouteLoc3AEC4::DeathAnimation,
            "ASM Diamond.asm loc_3AEC4 should route signed-negative collision_type directly to deletion",
        );
        assert_eq!(
            diamond_low_hp_collision_dispatch_loc_3aec4(DIAMOND_COLID_KIDRIGHT as i16, 2),
            DiamondLowHpCollisionRouteLoc3AEC4::BerzerkerSideGate,
            "ASM Diamond.asm loc_3AEC4 should route colid_kidright to the Berzerker side gate",
        );
        assert_eq!(
            diamond_low_hp_collision_dispatch_loc_3aec4(DIAMOND_COLID_KIDLEFT as i16, 2),
            DiamondLowHpCollisionRouteLoc3AEC4::BerzerkerSideGate,
            "ASM Diamond.asm loc_3AEC4 should route colid_kidleft to the Berzerker side gate",
        );
        assert_eq!(
            diamond_low_hp_collision_dispatch_loc_3aec4(DIAMOND_COLID_HURT as i16, 2),
            DiamondLowHpCollisionRouteLoc3AEC4::HurtAnimation { remaining_hp: 1 },
            "ASM Diamond.asm loc_3AEC4 should decrement current_hp before routing a surviving hurt hit",
        );
        assert_eq!(
            diamond_low_hp_collision_dispatch_loc_3aec4(DIAMOND_COLID_HURT as i16, 1),
            DiamondLowHpCollisionRouteLoc3AEC4::DeathAnimation,
            "ASM Diamond.asm loc_3AEC4 should route to deletion when the hurt decrement leaves zero",
        );
        assert_eq!(
            diamond_low_hp_collision_dispatch_loc_3aec4(DIAMOND_COLID_FLOOR as i16, 2),
            DiamondLowHpCollisionRouteLoc3AEC4::ResumeSharedWalker {
                next_collision_type: 0,
            },
            "ASM Diamond.asm loc_3AEC4 should ignore unknown positive collision IDs and resume walking",
        );
    }

    #[test]
    fn diamond_low_hp_death_animation_loc_3aee6_waits_then_deletes() {
        assert_eq!(
            diamond_low_hp_death_animation_loc_3aee6(),
            DiamondLowHpDeathAnimationLoc3AEE6 {
                animation_script: "stru_3ACC4",
                waits_until_animation_finished: true,
                deletes_enemy: true,
            },
            "ASM Diamond.asm loc_3AEE6 should play the death animation to completion before shared deletion",
        );
    }

    #[test]
    fn diamond_low_hp_hurt_animation_loc_3aef8_restores_base_animation() {
        assert_eq!(
            diamond_low_hp_hurt_animation_loc_3aef8(),
            DiamondLowHpHurtAnimationLoc3AEF8 {
                hurt_animation_script: "stru_3ACBE",
                waits_until_animation_finished: true,
                next_collision_type: 0,
                restored_animation_script: "stru_3ACA8",
                falls_through_to_shared_walker: true,
            },
            "ASM Diamond.asm loc_3AEF8 should play hurt, clear collision_type, restore base animation, and resume walking",
        );
    }

    #[test]
    fn diamond_low_hp_shared_walker_resume_loc_3af14_clears_collision_and_selects_direction() {
        assert_eq!(
            diamond_low_hp_shared_walker_resume_loc_3af14(false),
            DiamondLowHpSharedWalkerResumeLoc3AF14 {
                next_collision_type: 0,
                next_walker: DiamondLowHpSharedWalkerNextLoc3AF14::DefaultDirectionRoutineLoc3C0FA,
            },
            "ASM Diamond.asm loc_3AF14 should clear collision and branch to loc_3C0FA when x_direction is zero",
        );
        assert_eq!(
            diamond_low_hp_shared_walker_resume_loc_3af14(true),
            DiamondLowHpSharedWalkerResumeLoc3AF14 {
                next_collision_type: 0,
                next_walker: DiamondLowHpSharedWalkerNextLoc3AF14::FlippedDirectionRoutineLoc3C04E,
            },
            "ASM Diamond.asm loc_3AF14 should branch to loc_3C04E when x_direction is set",
        );
    }

    #[test]
    fn diamond_low_hp_berzerker_side_gate_loc_3af24_requires_charge() {
        assert_eq!(
            diamond_low_hp_berzerker_side_gate_loc_3af24(false, true),
            DiamondLowHpBerzerkerSideGateRouteLoc3AF24::ResumeSharedWalker(
                DiamondLowHpSharedWalkerResumeLoc3AF14 {
                    next_collision_type: 0,
                    next_walker:
                        DiamondLowHpSharedWalkerNextLoc3AF14::FlippedDirectionRoutineLoc3C04E,
                },
            ),
            "ASM Diamond.asm loc_3AF24 should branch to loc_3AF14 when Berzerker_charging is clear",
        );
        assert_eq!(
            diamond_low_hp_berzerker_side_gate_loc_3af24(true, false),
            DiamondLowHpBerzerkerSideGateRouteLoc3AF24::Launch(
                DiamondLowHpBerzerkerLaunchLoc3AF24 {
                    has_level_collision: false,
                    has_kid_collision: true,
                    is_moved: true,
                    is_animated: false,
                    sprite_mapping: "LnkTo_unk_C8160-Data_Index",
                    x_velocity: 0,
                    y_velocity: -0x0004_8000,
                },
            ),
            "ASM Diamond.asm loc_3AF24 should set the charging Berzerker launch flags and velocities",
        );
    }

    #[test]
    fn diamond_low_hp_berzerker_launch_tick_loc_3af50_applies_gravity_and_loops() {
        assert_eq!(
            diamond_low_hp_berzerker_launch_tick_loc_3af50(-0x0004_8000),
            DiamondLowHpBerzerkerLaunchTickLoc3AF50 {
                hibernates_one_frame: true,
                runs_shared_retention_check_sub_3c3ce: true,
                next_y_velocity: -0x0004_4000,
                loops: true,
            },
            "ASM Diamond.asm loc_3AF50 should add +$4000 to y_vel after the shared retention check",
        );
        assert_eq!(
            diamond_low_hp_berzerker_launch_tick_loc_3af50(i32::MAX).next_y_velocity,
            i32::MIN + 0x3FFF,
            "ASM Diamond.asm loc_3AF50 uses 68k longword addition, so native keeps wrapping semantics",
        );
    }
}
