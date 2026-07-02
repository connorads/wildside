use super::*;

pub(crate) const DRILLER_COLID_KIDABOVE: u16 = colid::KIDABOVE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrillerInitTailRouteLoc3BE60 {
    ClearDirectionLoc3C0D2,
    SetDirectionLoc3C026,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrillerInitTailLoc3BE60 {
    pub(crate) next_drill_refcount: u8,
    pub(crate) callback: &'static str,
    pub(crate) next_route: DrillerInitTailRouteLoc3BE60,
}

pub(crate) fn driller_init_tail_loc_3be60(
    x_direction_set: bool,
    drill_refcount: u8,
) -> DrillerInitTailLoc3BE60 {
    // ASM Driller.asm loc_3BE60: after the optional initial DrillMoving sound,
    // increment the shared refcount at $FFFFFA34+object_meta, set a0=loc_3BE78,
    // then dispatch by x_direction into the shared walker loops.
    DrillerInitTailLoc3BE60 {
        next_drill_refcount: drill_refcount.wrapping_add(1),
        callback: "loc_3BE78",
        next_route: if x_direction_set {
            DrillerInitTailRouteLoc3BE60::SetDirectionLoc3C026
        } else {
            DrillerInitTailRouteLoc3BE60::ClearDirectionLoc3C0D2
        },
    }
}

/// Driller linked drill child X offset for block drilling.
/// ASM Driller.asm loc_3BD74 initialises the two tread/drill children at
/// parent X -$18 and +$18; loc_3BE78 probes the selected child object, not
/// the parent's `$48(a5)` collision half-width.
pub(crate) const DRILLER_DRILL_CHILD_X_OFFSET: i32 = 0x18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrillerBlockDrillActionLoc3BEDE {
    None,
    SmashBits,
    IceBullets,
}

pub(crate) fn driller_block_drill_action_loc_3bede(
    cell: terrain::Cell,
) -> DrillerBlockDrillActionLoc3BEDE {
    if !(cell.has_block && cell.collision == terrain::CollisionKind::Solid) {
        return DrillerBlockDrillActionLoc3BEDE::None;
    }

    match cell.block_type {
        // ASM Driller.asm loc_3BEDE: block type mask `$000` or `$100`
        // calls Block_LoadSmashBits. In native block terms those are Smash
        // and Prize; Prize later restores through the normal deferred block path.
        terrain::BlockType::Smash | terrain::BlockType::Prize => {
            DrillerBlockDrillActionLoc3BEDE::SmashBits
        }
        // ASM Driller.asm loc_3BEF2: block type mask `$200` calls
        // IceBlock_SmashAndLoadBullets.
        terrain::BlockType::Ice => DrillerBlockDrillActionLoc3BEDE::IceBullets,
        _ => DrillerBlockDrillActionLoc3BEDE::None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrillerCollisionRouteLoc3BF02 {
    Continue,
    HurtAnimation,
    DeathAnimation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrillerCollisionDispatchLoc3BF02 {
    pub(crate) next_collision_type: u16,
    pub(crate) next_current_hp: u16,
    pub(crate) route: DrillerCollisionRouteLoc3BF02,
}

pub(crate) fn driller_collision_dispatch_loc_3bf02(
    collision_type: i16,
    current_hp: u16,
) -> DrillerCollisionDispatchLoc3BF02 {
    // ASM Driller.asm loc_3BF02: zero collision continues immediately;
    // negative words branch to loc_3BF1C before `clr.w collision_type(a3)`.
    // ASM Driller.asm loc_3BF66: every continue route rejoins the
    // `tst.b x_direction(a3)` dispatch into the shared walker loops.
    if collision_type == 0 {
        return DrillerCollisionDispatchLoc3BF02 {
            next_collision_type: 0,
            next_current_hp: current_hp,
            route: DrillerCollisionRouteLoc3BF02::Continue,
        };
    }

    if collision_type.is_negative() {
        return DrillerCollisionDispatchLoc3BF02 {
            next_collision_type: u16::from_ne_bytes(collision_type.to_ne_bytes()),
            next_current_hp: current_hp,
            route: DrillerCollisionRouteLoc3BF02::DeathAnimation,
        };
    }

    let collision_word = u16::from_ne_bytes(collision_type.to_ne_bytes());
    if collision_word != DRILLER_COLID_KIDABOVE {
        return DrillerCollisionDispatchLoc3BF02 {
            next_collision_type: 0,
            next_current_hp: current_hp,
            route: DrillerCollisionRouteLoc3BF02::Continue,
        };
    }

    let next_current_hp = current_hp.wrapping_sub(1);
    DrillerCollisionDispatchLoc3BF02 {
        next_collision_type: 0,
        next_current_hp,
        route: if next_current_hp == 0 {
            DrillerCollisionRouteLoc3BF02::DeathAnimation
        } else {
            // ASM Driller.asm loc_3BF4E: surviving stomp plays `stru_3BCCE`,
            // waits for it to finish, then restores `stru_3BCB2`.
            DrillerCollisionRouteLoc3BF02::HurtAnimation
        },
    }
}

/// Step a Driller enemy: paces like normal, but drills through Smash and Ice blocks.
/// ASM loc_3BE78: when sprite timer matches direction (8 for left, 4 for right),
/// checks cell at (sprite_x ± offset, sprite_y - 16) for drillable block type and destroys it.
/// Returns the drilled cells and their ASM block-dispatch action this frame.
pub(crate) fn step_driller(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) -> Vec<DrilledBlockLoc3BE78> {
    let Some(grid) = block_grid else {
        return Vec::new();
    };

    // ASM Driller.asm loc_3BE78 checks the drill child before `loc_3BF66`
    // dispatches back into the shared walker loop, so a front drillable block
    // is removed before the pacer wall probe can turn the Driller around.
    let drilled = driller_drilled_cells_loc_3be78(enemy, grid);
    if drilled.is_empty() {
        step_pacer(enemy, Some(grid));
    } else {
        let mut grid_after_drill = grid.clone();
        for drilled_block in &drilled {
            grid_after_drill.set_cell_at(
                drilled_block.col,
                drilled_block.row,
                terrain::Cell::from_be_bytes(0, 0),
            );
        }
        step_pacer(enemy, Some(&grid_after_drill));
    }

    drilled
}

fn driller_drilled_cells_loc_3be78(
    enemy: &ActiveEnemy,
    grid: &terrain::BlockGrid,
) -> Vec<DrilledBlockLoc3BE78> {
    let cx = (enemy.x_pos >> 16) as i32;
    let cy = (enemy.y_pos >> 16) as i32;
    // ASM Driller.asm loc_3BE8E gates drilling on the selected linked child
    // collision word (`d1=8` left child / `d1=4` right child), clears it,
    // then probes that child's x position and y-16 cell before dispatching to
    // loc_3BEDE/loc_3BEF2. Native has the child wall-contact gate in the
    // object-step shell; this helper preserves the child-space cell probe.
    let probe_x = if enemy.facing == Direction::Right {
        cx + DRILLER_DRILL_CHILD_X_OFFSET
    } else {
        cx - DRILLER_DRILL_CHILD_X_OFFSET
    };
    let direction = match enemy.facing {
        // ASM Driller.asm loc_3BE78 selects the right child and seeds d0=3.
        Direction::Right => 3,
        // ASM Driller.asm loc_3BE78 uses the left child with d0=1.
        Direction::Left => 1,
    };
    // ASM sub_3BEF2: Y probe is 16px above sprite (subi.w #$10,d7).
    let probe_y = cy - 16;

    let col = terrain::pixel_to_cell(probe_x).max(0) as u16;
    let row = terrain::pixel_to_cell(probe_y).max(0) as u16;
    let cell = grid.cell_at(col, row);

    let action = driller_block_drill_action_loc_3bede(cell);
    if matches!(action, DrillerBlockDrillActionLoc3BEDE::None) {
        Vec::new()
    } else {
        vec![DrilledBlockLoc3BE78 {
            col,
            row,
            action,
            direction,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    #[test]
    fn driller_live_step_latches_init_tail_once_per_visible_driller() {
        let mut enemies = [ActiveEnemy {
            family: EnemyFamily::Driller,
            facing: Direction::Right,
            hit_points: 3,
            ..mini_hopping_skull_test_enemy()
        }];
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
            0,
            false,
            None,
            &mut rng,
            &mut drilled_blocks,
            None,
        );

        assert!(
            enemies[0].shared_walker_patrol_short_next,
            "ASM Driller.asm loc_3BE60 increments the shared DrillMoving refcount after installing loc_3BE78",
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
            1,
            false,
            None,
            &mut rng,
            &mut drilled_blocks,
            None,
        );

        assert!(
            enemies[0].shared_walker_patrol_short_next,
            "ASM Driller.asm loc_3BE60 is an init tail, so the native latch must stay set after the first visible step",
        );
    }

    #[test]
    fn driller_spawn_setup_loc_3bd2a_faces_toward_player_and_seeds_hp() {
        assert_eq!(
            spawn_facing(EnemyFamily::Driller, 100 << 16, 80 << 16),
            Direction::Left,
            "ASM Driller.asm loc_3BD2A is reached without `st x_direction` when Kid is left of the Driller",
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Driller, 100 << 16, 100 << 16),
            Direction::Right,
            "ASM Driller.asm loc_3BD2A is reached after `st x_direction` when Kid is horizontally aligned",
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Driller, 100 << 16, 120 << 16),
            Direction::Right,
            "ASM Driller.asm loc_3BD2A is reached after `st x_direction` when Kid is right of the Driller",
        );
        assert_eq!(
            spawn_hit_points(EnemyFamily::Driller, 0),
            2,
            "ASM Driller.asm loc_3BD2A stores enemy_level + 2 in current_hp",
        );
        assert_eq!(
            spawn_hit_points(EnemyFamily::Driller, 2),
            4,
            "ASM Driller.asm loc_3BD2A keeps raw enemy_level as the speed/HP tier before adding 2",
        );
    }

    #[test]
    fn driller_walk_speed_matches_loc_3bd6c_and_loc_3bd74() {
        assert_eq!(
            driller_walk_speed_loc_3bd74(2),
            0x0000_8000,
            "ASM Driller.asm loc_3BD74 gives low HP only the final `$8000` add",
        );
        assert_eq!(
            driller_walk_speed_loc_3bd74(3),
            0x0001_0000,
            "ASM Driller.asm loc_3BD6C lets HP 3 reach the middle `$8000` add before loc_3BD74",
        );
        assert_eq!(
            driller_walk_speed_loc_3bd74(4),
            0x0001_8000,
            "ASM Driller.asm loc_3BD64 high-HP preload plus loc_3BD6C/loc_3BD74 caps Driller speed at three `$8000` adds",
        );
        assert_eq!(
            driller_walk_speed_loc_3bd74(5),
            0x0001_8000,
            "ASM Driller.asm loc_3BD64 keeps HP above 4 on the same capped high-speed tier",
        );
    }

    #[test]
    fn driller_turnaround_counter_overflow_marks_death_loc_3c464() {
        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.family = EnemyFamily::Driller;
        enemy.facing = Direction::Right;
        enemy.hit_points = 1;
        enemy.shared_walker_turnaround_counter = 0xFC;
        let mut grid = mini_hopping_skull_test_grid();
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0, 0));

        step_pacer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.hit_points, 0,
            "ASM loc_3C464 -> Driller.asm loc_3BF02 routes $FFFF collision_type to the death path"
        );
        assert_eq!(
            enemy.wait_timer, DRILLER_DEATH_ANIMATION_FRAMES,
            "ASM Driller.asm loc_3BF1C should play the parent death animation instead of disappearing on the loc_3C464 route"
        );
        assert_eq!(enemy.shared_walker_turnaround_counter, 0);
    }

    #[test]
    fn driller_hurt_animation_hibernates_shared_walker_loc_3bf4e() {
        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.family = EnemyFamily::Driller;
        enemy.facing = Direction::Right;
        enemy.hit_points = 2;
        enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
        enemy.shared_walker_patrol_short_next = true;
        let start_x = enemy.x_pos;
        let mut enemies = [enemy];
        let grid = mini_hopping_skull_test_grid();
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();

        step_active_enemies_with_rng(
            &mut enemies,
            Some(&grid),
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
            None,
        );

        assert_eq!(
            enemies[0].x_pos, start_x,
            "ASM Driller.asm loc_3BF4E hibernates through `stru_3BCCE` before loc_3BF66 can resume the shared walker"
        );
        assert_eq!(enemies[0].x_velocity, 0);
        assert_eq!(enemies[0].y_velocity, 0);
        assert!(drilled_blocks.is_empty());
    }
}
