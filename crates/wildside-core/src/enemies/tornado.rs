use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TornadoCollisionBehaviorLoc35948 {
    BounceWall,
    BounceFloorCeil,
    BounceSlopeUp,
    BounceSlopeDown,
    Hurt,
    CycloneBounceKidRight,
    CycloneBounceKidLeft,
    ReturnOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TornadoCollisionRouteLoc35056 {
    NegativeClear,
    ZeroReturn,
    TableDispatch,
    UnknownPositiveClear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TornadoCollisionDispatchLoc35948 {
    pub(crate) collision_type: u16,
    pub(crate) route: TornadoCollisionRouteLoc35056,
    pub(crate) behavior: Option<TornadoCollisionBehaviorLoc35948>,
}

pub(crate) fn tornado_collision_dispatch_loc_35948(
    collision_type: i16,
) -> TornadoCollisionDispatchLoc35948 {
    // ASM Tornado.asm loc_35948: load Tornado's `off_35950` table into `a4`,
    // then branch to shared UFO collision dispatch at `loc_35056`.
    if collision_type < 0 {
        return TornadoCollisionDispatchLoc35948 {
            collision_type: 0,
            route: TornadoCollisionRouteLoc35056::NegativeClear,
            behavior: None,
        };
    }

    if collision_type == 0 {
        return TornadoCollisionDispatchLoc35948 {
            collision_type: 0,
            route: TornadoCollisionRouteLoc35056::ZeroReturn,
            behavior: None,
        };
    }

    let (route, behavior) = match collision_type as u16 {
        0x0004 | 0x0008 => (
            TornadoCollisionRouteLoc35056::TableDispatch,
            Some(TornadoCollisionBehaviorLoc35948::BounceWall),
        ),
        0x000C | 0x0010 => (
            TornadoCollisionRouteLoc35056::TableDispatch,
            Some(TornadoCollisionBehaviorLoc35948::BounceFloorCeil),
        ),
        0x0014 => (
            TornadoCollisionRouteLoc35056::TableDispatch,
            Some(TornadoCollisionBehaviorLoc35948::BounceSlopeUp),
        ),
        0x0018 => (
            TornadoCollisionRouteLoc35056::TableDispatch,
            Some(TornadoCollisionBehaviorLoc35948::BounceSlopeDown),
        ),
        0x001C | 0x002C => (
            TornadoCollisionRouteLoc35056::TableDispatch,
            Some(TornadoCollisionBehaviorLoc35948::Hurt),
        ),
        0x0020 => (
            TornadoCollisionRouteLoc35056::TableDispatch,
            Some(TornadoCollisionBehaviorLoc35948::CycloneBounceKidRight),
        ),
        0x0024 => (
            TornadoCollisionRouteLoc35056::TableDispatch,
            Some(TornadoCollisionBehaviorLoc35948::CycloneBounceKidLeft),
        ),
        0x0028 => (
            TornadoCollisionRouteLoc35056::TableDispatch,
            Some(TornadoCollisionBehaviorLoc35948::ReturnOnly),
        ),
        _ => (TornadoCollisionRouteLoc35056::UnknownPositiveClear, None),
    };

    TornadoCollisionDispatchLoc35948 {
        collision_type: 0,
        route,
        behavior,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TornadoHitStatusRouteLoc359B2 {
    WriteRespawnMarker(TornadoRespawnMarkerLoc359E0),
    SubtractSpawnCounter { index: usize, decrement: u16 },
    DeleteOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TornadoRespawnMarkerLoc359E0 {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

pub(crate) fn tornado_respawn_marker_loc_359e0(
    object_status_slot: u8,
) -> TornadoRespawnMarkerLoc359E0 {
    // ASM Tornado.asm loc_359E0: positive `$42(a5)` doubles as an
    // EnemyStatus_Table word offset, then writes `$2168`.
    TornadoRespawnMarkerLoc359E0 {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TornadoDeleteTailLoc359EE {
    pub(crate) deletes_current_object: bool,
}

pub(crate) fn tornado_delete_tail_loc_359ee() -> TornadoDeleteTailLoc359EE {
    // ASM Tornado.asm loc_359EE: all status-table branches finish by jumping to
    // `j_Delete_CurrentObject`.
    TornadoDeleteTailLoc359EE {
        deletes_current_object: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TornadoHitSequenceLoc359B2 {
    pub(crate) waits_for_hit_animation: bool,
    pub(crate) status_route: TornadoHitStatusRouteLoc359B2,
    pub(crate) delete_tail: TornadoDeleteTailLoc359EE,
}

pub(crate) fn tornado_hit_sequence_loc_359b2(object_status_slot: u8) -> TornadoHitSequenceLoc359B2 {
    // ASM Tornado.asm loc_359B2: play `stru_35B1E`, hibernate until it
    // finishes, then inspect `$42(a5)` to choose the status-table update.
    let status_route = if object_status_slot & 0x80 == 0 {
        TornadoHitStatusRouteLoc359B2::WriteRespawnMarker(tornado_respawn_marker_loc_359e0(
            object_status_slot,
        ))
    } else if object_status_slot & 0x40 != 0 {
        TornadoHitStatusRouteLoc359B2::SubtractSpawnCounter {
            index: usize::from(object_status_slot & 0x3F),
            decrement: 0x0400,
        }
    } else {
        TornadoHitStatusRouteLoc359B2::DeleteOnly
    };

    TornadoHitSequenceLoc359B2 {
        waits_for_hit_animation: true,
        status_route,
        delete_tail: tornado_delete_tail_loc_359ee(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TornadoCycloneBounceLoc35986 {
    pub(crate) player_x_velocity: i32,
    pub(crate) tornado_x_velocity: i32,
    pub(crate) player_collision_type: u16,
}

fn tornado_cyclone_bounce_from_d7_loc_35986(d7: i32) -> TornadoCycloneBounceLoc35986 {
    // ASM Tornado.asm loc_35986: `swap d7`, write it to Kid's x_vel, then
    // `neg.l d7; asr.l #1,d7` for the Tornado's reflected x_vel.
    let player_x_velocity = swap_words_i32(d7);
    let tornado_x_velocity = player_x_velocity.wrapping_neg() >> 1;
    TornadoCycloneBounceLoc35986 {
        player_x_velocity,
        tornado_x_velocity,
        player_collision_type: 0x0028,
    }
}

pub(crate) fn tornado_cyclone_bounce_kid_right_loc_3597e() -> TornadoCycloneBounceLoc35986 {
    // ASM Tornado.asm loc_3597E: `moveq #2,d7; bra.w loc_35986`.
    tornado_cyclone_bounce_from_d7_loc_35986(2)
}

pub(crate) fn tornado_cyclone_bounce_kid_left_loc_35984() -> TornadoCycloneBounceLoc35986 {
    // ASM Tornado.asm loc_35984: `moveq #-2,d7`, then fall through to `loc_35986`.
    tornado_cyclone_bounce_from_d7_loc_35986(-2)
}

pub(crate) fn tornado_cyclone_bounce_for_positions_loc_35986(
    player_x: i64,
    tornado_x: i64,
) -> TornadoCycloneBounceLoc35986 {
    if player_x < tornado_x {
        tornado_cyclone_bounce_kid_left_loc_35984()
    } else {
        tornado_cyclone_bounce_kid_right_loc_3597e()
    }
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

    if !enemy.death_animation_active {
        // ASM Tornado.asm loc_35908 hibernates one frame after acceleration
        // while `stru_35B10` continues to advance. Tornado never uses
        // `grounded_frames`, so native stores the live contact-animation tick
        // there for GfxObjects_CollisionKid loc_1BCE row selection.
        enemy.grounded_frames = (enemy.grounded_frames + 1) % TORNADO_LIVE_ANIMATION_TICKS;
    }

    // ASM Tornado.asm loc_35908: the main flight loop decrements `$44(a5)`;
    // only the post-decrement zero result falls through to loc_359F2 and
    // resets from `unk_3588C`.
    enemy.wait_timer = enemy.wait_timer.wrapping_sub(1);
    if enemy.wait_timer == 0 {
        let level = usize::from(enemy.enemy_level).min(2);
        enemy.wait_timer = TORNADO_PATTERN_FRAMES[level];
        reload_tornado_pattern_script(enemy);
    }

    // ASM Tornado.asm loc_35948 uses Tornado's `off_35950` table, then shares
    // UFO.asm loc_35056 terrain responses for wall/floor/ceiling/slope hits.
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
            let collision_type = if moving_right { 0x0004 } else { 0x0008 };
            apply_tornado_terrain_collision_loc_35948(enemy, collision_type);
            return;
        }

        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground {
            let collision_type = match floor.cell.collision {
                terrain::CollisionKind::SlopeUp => 0x0014,
                terrain::CollisionKind::SlopeDown => 0x0018,
                _ if enemy.y_velocity > 0 => 0x000C,
                _ => 0,
            };
            if collision_type != 0 {
                apply_tornado_terrain_collision_loc_35948(enemy, collision_type);
                return;
            }
        }

        let ceiling =
            terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
        if ceiling.hit && enemy.y_velocity < 0 {
            apply_tornado_terrain_collision_loc_35948(enemy, 0x0010);
        }
    }
}

fn apply_tornado_terrain_collision_loc_35948(enemy: &mut ActiveEnemy, collision_type: i16) {
    let dispatch = tornado_collision_dispatch_loc_35948(collision_type);
    match dispatch.behavior {
        Some(TornadoCollisionBehaviorLoc35948::BounceWall) => {
            let bounce = ufo_bounce_wall_tail_loc_35120(enemy.x_velocity, enemy.x_acceleration);
            enemy.x_velocity = bounce.x_velocity;
            enemy.x_acceleration = bounce.d0;
            enemy.target_x_velocity = 0;
            enemy.facing = opposite_direction(enemy.facing);
        }
        Some(TornadoCollisionBehaviorLoc35948::BounceFloorCeil) => {
            let bounce =
                ufo_bounce_floor_ceil_tail_loc_35146(enemy.y_velocity, enemy.y_acceleration);
            enemy.y_velocity = bounce.y_velocity;
            enemy.y_acceleration = bounce.d2;
            enemy.target_y_velocity = 0;
        }
        Some(TornadoCollisionBehaviorLoc35948::BounceSlopeUp) => {
            // ASM UFO.asm Enemy0F_UFO_BounceUpSlope: loc_350F2 rolls back the
            // current movement before forcing the up-left launch vector.
            let restore = ufo_restore_position_loc_350f2(
                enemy.x_pos as i32,
                enemy.y_pos as i32,
                enemy.x_velocity,
                enemy.y_velocity,
            );
            enemy.x_pos = restore.x_pos as u32;
            enemy.y_pos = restore.y_pos as u32;
            enemy.x_velocity = -0x0001_0000;
            enemy.y_velocity = -0x0001_0000;
            enemy.x_acceleration = 0x800;
            enemy.y_acceleration = 0x800;
            enemy.target_x_velocity = 0;
            enemy.target_y_velocity = 0;
            enemy.facing = Direction::Left;
        }
        Some(TornadoCollisionBehaviorLoc35948::BounceSlopeDown) => {
            // ASM UFO.asm Enemy0F_UFO_BounceDownSlope mirrors the slope-up path
            // with a rightward launch and negative X acceleration.
            let restore = ufo_restore_position_loc_350f2(
                enemy.x_pos as i32,
                enemy.y_pos as i32,
                enemy.x_velocity,
                enemy.y_velocity,
            );
            enemy.x_pos = restore.x_pos as u32;
            enemy.y_pos = restore.y_pos as u32;
            enemy.x_velocity = 0x0001_0000;
            enemy.y_velocity = -0x0001_0000;
            enemy.x_acceleration = -0x800;
            enemy.y_acceleration = 0x800;
            enemy.target_x_velocity = 0;
            enemy.target_y_velocity = 0;
            enemy.facing = Direction::Right;
        }
        _ => {}
    }
}
