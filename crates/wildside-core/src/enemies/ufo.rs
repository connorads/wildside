use super::*;

/// UFO: acceleration-based drifting through a once-per-second ROM motion table.
pub(crate) const UFO_ACCELERATION: i32 = 0x400;
pub(crate) const UFO_PATTERN_FRAMES: u16 = 60;

// ASM UFO.asm `unk_35344`: 16 motion records selected by loc_35326 using
// `(Get_RandomNumber_byte() ^ x_pos.low_byte) & $0F`.
pub(crate) const UFO_PATTERN_SCRIPT_LOC_35344: [UfoMotionCommand; 16] = [
    UfoMotionCommand {
        target_x_velocity: 0,
        target_y_velocity: -0x20000,
        x_acceleration: 0,
        y_acceleration: -0x1000,
    },
    UfoMotionCommand {
        target_x_velocity: 0x10000,
        target_y_velocity: -0x20000,
        x_acceleration: 0x0800,
        y_acceleration: -0x1000,
    },
    UfoMotionCommand {
        target_x_velocity: 0x18000,
        target_y_velocity: -0x18000,
        x_acceleration: 0x0C00,
        y_acceleration: -0x0C00,
    },
    UfoMotionCommand {
        target_x_velocity: 0x20000,
        target_y_velocity: -0x10000,
        x_acceleration: 0x1000,
        y_acceleration: -0x0800,
    },
    UfoMotionCommand {
        target_x_velocity: 0x20000,
        target_y_velocity: 0,
        x_acceleration: 0x1000,
        y_acceleration: 0,
    },
    UfoMotionCommand {
        target_x_velocity: 0x20000,
        target_y_velocity: 0x10000,
        x_acceleration: 0x1000,
        y_acceleration: 0x0800,
    },
    UfoMotionCommand {
        target_x_velocity: 0x18000,
        target_y_velocity: 0x18000,
        x_acceleration: 0x0C00,
        y_acceleration: 0x0C00,
    },
    UfoMotionCommand {
        target_x_velocity: 0x10000,
        target_y_velocity: 0x20000,
        x_acceleration: 0x0800,
        y_acceleration: 0x1000,
    },
    UfoMotionCommand {
        target_x_velocity: 0,
        target_y_velocity: 0x20000,
        x_acceleration: 0,
        y_acceleration: 0x1000,
    },
    UfoMotionCommand {
        target_x_velocity: -0x10000,
        target_y_velocity: 0x20000,
        x_acceleration: -0x0800,
        y_acceleration: 0x1000,
    },
    UfoMotionCommand {
        target_x_velocity: -0x18000,
        target_y_velocity: 0x18000,
        x_acceleration: -0x0C00,
        y_acceleration: 0x0C00,
    },
    UfoMotionCommand {
        target_x_velocity: -0x20000,
        target_y_velocity: 0x10000,
        x_acceleration: -0x1000,
        y_acceleration: 0x0800,
    },
    UfoMotionCommand {
        target_x_velocity: -0x20000,
        target_y_velocity: 0,
        x_acceleration: -0x1000,
        y_acceleration: 0,
    },
    UfoMotionCommand {
        target_x_velocity: -0x20000,
        target_y_velocity: -0x10000,
        x_acceleration: -0x1000,
        y_acceleration: -0x0800,
    },
    UfoMotionCommand {
        target_x_velocity: -0x18000,
        target_y_velocity: -0x18000,
        x_acceleration: -0x0C00,
        y_acceleration: -0x0C00,
    },
    UfoMotionCommand {
        target_x_velocity: -0x10000,
        target_y_velocity: -0x20000,
        x_acceleration: -0x0800,
        y_acceleration: -0x1000,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoMotionCommand {
    pub(crate) x_acceleration: i32,
    pub(crate) target_x_velocity: i32,
    pub(crate) y_acceleration: i32,
    pub(crate) target_y_velocity: i32,
}

pub(crate) fn ufo_double_acceleration_loc_34e10(
    mut command: UfoMotionCommand,
    double_acceleration: bool,
) -> UfoMotionCommand {
    if double_acceleration {
        // ASM UFO.asm loc_34E10: non-zero `$4E(a5)` doubles d0 and d2 only.
        command.x_acceleration = command.x_acceleration.saturating_mul(2);
        command.y_acceleration = command.y_acceleration.saturating_mul(2);
    }
    command
}

pub(crate) fn ufo_double_terminal_speed_loc_34e1c(
    mut command: UfoMotionCommand,
    double_terminal_speed: bool,
) -> UfoMotionCommand {
    if double_terminal_speed {
        // ASM UFO.asm loc_34E1C: non-zero `$4F(a5)` doubles d1 and d3 only.
        command.target_x_velocity = command.target_x_velocity.saturating_mul(2);
        command.target_y_velocity = command.target_y_velocity.saturating_mul(2);
    }
    command
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoTrackingCommandLoc34E2A {
    pub(crate) motion: UfoMotionCommand,
    pub(crate) y_velocity: i32,
    pub(crate) facing: Direction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoBeamStartLoc34Eec {
    pub(crate) beam_segments: u8,
    pub(crate) beam_interval: u8,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) wait_timer: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoOnScreenCountLoc34F64 {
    pub(crate) number_ufos_on_screen: u16,
    pub(crate) ufo_on_screen_latch: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoOffScreenGateLoc34F6E {
    pub(crate) number_ufos_on_screen: u16,
    pub(crate) play_hover_stop_sound: bool,
    pub(crate) clear_latch_tail: bool,
}

pub(crate) fn ufo_on_screen_count_loc_34f64(
    number_ufos_on_screen: u16,
) -> UfoOnScreenCountLoc34F64 {
    // ASM UFO.asm loc_34F64: when a previously-unlatched UFO is inside the
    // screen window, `addq.w #1,(Number_UFOs_OnScreen).w; st $50(a5)`.
    UfoOnScreenCountLoc34F64 {
        number_ufos_on_screen: number_ufos_on_screen.wrapping_add(1),
        ufo_on_screen_latch: true,
    }
}

pub(crate) fn ufo_off_screen_gate_loc_34f6e(
    number_ufos_on_screen: u16,
    ufo_on_screen_latch: bool,
) -> UfoOffScreenGateLoc34F6E {
    // ASM UFO.asm loc_34F6E: off-screen UFOs with clear `$50(a5)` return
    // without touching `Number_UFOs_OnScreen`; latched UFOs decrement it and
    // only call `j_PlaySound2` when the decremented word is zero.
    if !ufo_on_screen_latch {
        return UfoOffScreenGateLoc34F6E {
            number_ufos_on_screen,
            play_hover_stop_sound: false,
            clear_latch_tail: false,
        };
    }

    let number_ufos_on_screen = number_ufos_on_screen.wrapping_sub(1);
    UfoOffScreenGateLoc34F6E {
        number_ufos_on_screen,
        play_hover_stop_sound: number_ufos_on_screen == 0,
        clear_latch_tail: true,
    }
}

pub(crate) fn ufo_clear_on_screen_latch_loc_34f88() -> bool {
    // ASM UFO.asm loc_34F88: the off-screen tail clears `$50(a5)` with `sf`.
    false
}

pub(crate) fn ufo_on_screen_gate_loc_35008(
    ufo_x_pos: u32,
    ufo_y_pos: u32,
    kid_x_pos: u32,
    kid_y_pos: u32,
    number_ufos_on_screen: u16,
    ufo_on_screen_latch: bool,
) -> (u16, bool, Option<QueuedSoundEffect>) {
    let ufo_x = fixed_integer_word(ufo_x_pos);
    let ufo_y = fixed_integer_word(ufo_y_pos);
    let kid_x = fixed_integer_word(kid_x_pos);
    let kid_y = fixed_integer_word(kid_y_pos);
    let dx = ufo_x.wrapping_sub(kid_x);
    let dy = ufo_y.wrapping_sub(kid_y);

    // ASM UFO.asm loc_34F16-loc_34F6E: on-screen means within +/-$A0 X
    // pixels of Kid, at least $A0 pixels above, and no more than $50 below.
    let on_screen = (-0x00A0..=0x00A0).contains(&dx) && (-0x00A0..=0x0050).contains(&dy);

    if on_screen {
        if ufo_on_screen_latch {
            return (number_ufos_on_screen, ufo_on_screen_latch, None);
        }
        let counted = ufo_on_screen_count_loc_34f64(number_ufos_on_screen);
        let sound = (number_ufos_on_screen == 0).then_some(QueuedSoundEffect::UfoHovering);
        return (
            counted.number_ufos_on_screen,
            counted.ufo_on_screen_latch,
            sound,
        );
    }

    let off_screen = ufo_off_screen_gate_loc_34f6e(number_ufos_on_screen, ufo_on_screen_latch);
    let latch = if off_screen.clear_latch_tail {
        ufo_clear_on_screen_latch_loc_34f88()
    } else {
        ufo_on_screen_latch
    };
    let sound = off_screen
        .play_hover_stop_sound
        .then_some(QueuedSoundEffect::UfoHovering);
    (off_screen.number_ufos_on_screen, latch, sound)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn ufo_tracking_command_loc_34e2a(
    ufo_x_pos: u32,
    ufo_y_pos: u32,
    kid_x_pos: u32,
    kid_y_pos: u32,
    tracking_enabled: bool,
    some_ufo_shooting: bool,
    double_acceleration: bool,
    double_terminal_speed: bool,
) -> Option<UfoTrackingCommandLoc34E2A> {
    // ASM UFO.asm loc_34E2A: `$4D(a5)` and `Some_UFO_Shooting` gate the whole
    // below-and-near Kid chase branch before any velocity state is changed.
    if !tracking_enabled || some_ufo_shooting {
        return None;
    }

    let ufo_x = fixed_integer_word(ufo_x_pos);
    let ufo_y = fixed_integer_word(ufo_y_pos);
    let kid_x = fixed_integer_word(kid_x_pos);
    let kid_y = fixed_integer_word(kid_y_pos);
    if kid_y <= ufo_y {
        return None;
    }

    let left_edge = kid_x.wrapping_sub(0x30);
    if ufo_x < left_edge {
        return None;
    }
    let right_edge = left_edge.wrapping_add(0x60);
    if ufo_x > right_edge {
        return None;
    }

    let command = if ufo_x < kid_x {
        ufo_right_tracking_command_loc_34e86()
    } else {
        UfoMotionCommand {
            x_acceleration: -0x1000,
            target_x_velocity: -0x20000,
            y_acceleration: 0,
            target_y_velocity: 0,
        }
    };
    let motion = ufo_tracking_tail_loc_34e96(command, double_acceleration, double_terminal_speed);

    Some(UfoTrackingCommandLoc34E2A {
        motion,
        y_velocity: 0,
        facing: if ufo_x < kid_x {
            Direction::Right
        } else {
            Direction::Left
        },
    })
}

pub(crate) fn ufo_right_tracking_command_loc_34e86() -> UfoMotionCommand {
    // ASM UFO.asm loc_34E86: Kid is to the right, so d1/d0 become
    // +$20000/+$1000 and `st x_direction(a3)` marks the UFO as flipped.
    UfoMotionCommand {
        x_acceleration: 0x1000,
        target_x_velocity: 0x20000,
        y_acceleration: 0,
        target_y_velocity: 0,
    }
}

pub(crate) fn ufo_tracking_tail_loc_34e96(
    command: UfoMotionCommand,
    double_acceleration: bool,
    double_terminal_speed: bool,
) -> UfoMotionCommand {
    // ASM UFO.asm loc_34E96: both left/right Kid chase branches share the
    // `bsr.w loc_34E10` scaling tail before returning.
    ufo_double_terminal_speed_loc_34e1c(
        ufo_double_acceleration_loc_34e10(command, double_acceleration),
        double_terminal_speed,
    )
}

pub(crate) fn ufo_motion_picker_loc_35326(
    random_byte: u8,
    ufo_x_pos: u32,
    x_velocity: i32,
    y_velocity: i32,
    double_acceleration: bool,
    double_terminal_speed: bool,
) -> (UfoMotionCommand, Direction) {
    let x_word_low = ((ufo_x_pos >> 16) & 0xFF) as u8;
    let index = usize::from((random_byte ^ x_word_low) & 0x0F);
    ufo_motion_from_script_loc_35444(
        UFO_PATTERN_SCRIPT_LOC_35344[index],
        x_velocity,
        y_velocity,
        double_acceleration,
        double_terminal_speed,
    )
}

pub(crate) fn ufo_motion_from_script_loc_35444(
    script: UfoMotionCommand,
    x_velocity: i32,
    y_velocity: i32,
    double_acceleration: bool,
    double_terminal_speed: bool,
) -> (UfoMotionCommand, Direction) {
    let command = UfoMotionCommand {
        target_x_velocity: script.target_x_velocity,
        target_y_velocity: script.target_y_velocity,
        x_acceleration: acceleration_for_target(
            x_velocity,
            script.target_x_velocity,
            script.x_acceleration,
        ),
        y_acceleration: acceleration_for_target(
            y_velocity,
            script.target_y_velocity,
            script.y_acceleration,
        ),
    };
    let command = ufo_double_terminal_speed_loc_34e1c(
        ufo_double_acceleration_loc_34e10(command, double_acceleration),
        double_terminal_speed,
    );
    let facing = if script.target_x_velocity < 0 {
        Direction::Left
    } else {
        Direction::Right
    };
    (command, facing)
}

#[cfg(test)]
pub(crate) fn step_ufo(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    let mut rng = RomRandomState::default();
    step_ufo_with_player_and_rng(enemy, block_grid, None, None, &mut rng);
}

pub(crate) fn step_ufo_with_player(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    rng: &mut RomRandomState,
) {
    step_ufo_with_player_and_rng(enemy, block_grid, player_x_pos, player_y_pos, rng);
}

pub(crate) fn step_ufo_with_player_and_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    rng: &mut RomRandomState,
) {
    if enemy.wait_timer == 0 {
        let (motion, facing) = ufo_motion_picker_loc_35326(
            rng.next_byte(),
            enemy.x_pos,
            enemy.x_velocity,
            enemy.y_velocity,
            enemy.flight_double_acceleration,
            enemy.flight_double_terminal_speed,
        );
        enemy.x_acceleration = motion.x_acceleration;
        enemy.target_x_velocity = motion.target_x_velocity;
        enemy.y_acceleration = motion.y_acceleration;
        enemy.target_y_velocity = motion.target_y_velocity;
        enemy.facing = facing;
        enemy.wait_timer = UFO_PATTERN_FRAMES;
    }

    // ASM UFO.asm Enemy_HandleAcceleration: d0/d2 accelerate toward d1/d3.
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

    if let (Some(kid_x), Some(kid_y)) = (player_x_pos, player_y_pos)
        && let Some(tracking) = ufo_tracking_command_loc_34e2a(
            enemy.x_pos,
            enemy.y_pos,
            kid_x,
            kid_y,
            enemy.enemy_level >= 1 || enemy.flight_double_acceleration,
            enemy.ufo_beam_segments != 0,
            enemy.flight_double_acceleration,
            enemy.flight_double_terminal_speed,
        )
    {
        enemy.x_acceleration = tracking.motion.x_acceleration;
        enemy.target_x_velocity = tracking.motion.target_x_velocity;
        enemy.y_acceleration = tracking.motion.y_acceleration;
        enemy.target_y_velocity = tracking.motion.target_y_velocity;
        enemy.y_velocity = tracking.y_velocity;
        enemy.facing = tracking.facing;
    }

    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    enemy.wait_timer = enemy.wait_timer.saturating_sub(1);

    // ASM UFO.asm Enemy0F_UFO_ExecCollisionBehavior: terrain contacts enter
    // loc_35056 through `Enemy0F_UFO_CollisionBehaviors`.
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
            apply_ufo_terrain_collision_loc_35056(enemy, collision_type);
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
                apply_ufo_terrain_collision_loc_35056(enemy, collision_type);
                return;
            }
        }

        let ceiling =
            terrain::probe_ceiling(grid, cx, cy, ENEMY_PROBE_HALF_WIDTH, ENEMY_PROBE_HEIGHT);
        if ceiling.hit && enemy.y_velocity < 0 {
            apply_ufo_terrain_collision_loc_35056(enemy, 0x0010);
        }
    }
}

fn apply_ufo_terrain_collision_loc_35056(enemy: &mut ActiveEnemy, collision_type: i16) {
    let dispatch = ufo_collision_dispatch_loc_35056(collision_type);
    match dispatch.behavior {
        Some(UfoCollisionBehaviorLoc35056::BounceWall) => {
            let bounce = ufo_bounce_wall_tail_loc_35120(enemy.x_velocity, enemy.x_acceleration);
            enemy.x_velocity = bounce.x_velocity;
            enemy.x_acceleration = bounce.d0;
            enemy.target_x_velocity = 0;
            enemy.facing = opposite_direction(enemy.facing);
        }
        Some(UfoCollisionBehaviorLoc35056::BounceFloorCeil) => {
            let bounce =
                ufo_bounce_floor_ceil_tail_loc_35146(enemy.y_velocity, enemy.y_acceleration);
            enemy.y_velocity = bounce.y_velocity;
            enemy.y_acceleration = bounce.d2;
            enemy.target_y_velocity = 0;
        }
        Some(UfoCollisionBehaviorLoc35056::BounceSlopeUp) => {
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
        Some(UfoCollisionBehaviorLoc35056::BounceSlopeDown) => {
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

/// UFO beam: spawns downward beam segments at the UFO's position.
/// ASM: `ChkShoot` fires when player is directly below (±2px), `ChkLoadBeam` fires
/// randomly (~4/256 per frame). Spawns 4 segments at 5-frame intervals.
const UFO_BEAM_SEGMENT_COUNT: u8 = 4;
const UFO_BEAM_INTERVAL_FRAMES: u8 = 5;
/// Beam segment y_velocity: $30000 from ASM.
const UFO_BEAM_Y_VELOCITY: i32 = 0x3_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoBeamLoadTickLoc34FD6 {
    pub(crate) next_beam_segments: u8,
    pub(crate) next_beam_interval: u8,
    pub(crate) spawn_beam_segment: bool,
    pub(crate) clear_ufo_shooting_state: bool,
}

pub(crate) fn ufo_beam_load_tick_loc_34fd6(
    shooting_active: bool,
    beam_segments: u8,
    beam_interval: u8,
) -> UfoBeamLoadTickLoc34FD6 {
    // ASM UFO.asm loc_34FD6: when `$46(a5)` is set, this counts down `$4A(a5)`
    // each frame, reloads it to 5 every beam segment, and decrements `$48(a5)`.
    if !shooting_active {
        return UfoBeamLoadTickLoc34FD6 {
            next_beam_segments: beam_segments,
            next_beam_interval: beam_interval,
            spawn_beam_segment: false,
            clear_ufo_shooting_state: false,
        };
    }

    let mut beam_interval = beam_interval.wrapping_sub(1);
    if beam_interval != 0 {
        return UfoBeamLoadTickLoc34FD6 {
            next_beam_segments: beam_segments,
            next_beam_interval: beam_interval,
            spawn_beam_segment: false,
            clear_ufo_shooting_state: false,
        };
    }

    beam_interval = UFO_BEAM_INTERVAL_FRAMES;
    let next_beam_segments = beam_segments.wrapping_sub(1);
    let clear_ufo_shooting_state = next_beam_segments == 0;
    UfoBeamLoadTickLoc34FD6 {
        next_beam_segments,
        next_beam_interval: beam_interval,
        spawn_beam_segment: !clear_ufo_shooting_state,
        clear_ufo_shooting_state,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoBeamClearLoc34FFE {
    pub(crate) ufo_shooting_latch: bool,
    pub(crate) global_shooting_flag: bool,
}

pub(crate) fn ufo_beam_clear_shooting_state_loc_34ffe() -> UfoBeamClearLoc34FFE {
    // ASM UFO.asm loc_34FFE: when `$48(a5)` reaches zero, clear `$46(a5)` and
    // `Some_UFO_Shooting`.
    UfoBeamClearLoc34FFE {
        ufo_shooting_latch: false,
        global_shooting_flag: false,
    }
}

pub(crate) fn ufo_beam_start_loc_34eec() -> UfoBeamStartLoc34Eec {
    // ASM UFO.asm loc_34EEC: targeted shooting starts the shared UFO beam
    // sequence, freezes the UFO, and reloads the one-second loop counter to $32.
    UfoBeamStartLoc34Eec {
        beam_segments: UFO_BEAM_SEGMENT_COUNT,
        beam_interval: 5,
        x_velocity: 0,
        y_velocity: 0,
        wait_timer: 0x32,
    }
}

fn apply_ufo_beam_start_loc_34eec(enemy: &mut ActiveEnemy) {
    let beam_start = ufo_beam_start_loc_34eec();
    enemy.ufo_beam_segments = beam_start.beam_segments;
    enemy.ufo_beam_interval = beam_start.beam_interval;
    enemy.x_velocity = beam_start.x_velocity;
    enemy.y_velocity = beam_start.y_velocity;
    enemy.wait_timer = beam_start.wait_timer;
}

#[cfg(test)]
pub(crate) fn step_ufo_beam(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x: Option<u32>,
    player_y: Option<u32>,
) {
    let mut rng = RomRandomState::default();
    step_ufo_beam_with_rng(enemy, projectiles, player_x, player_y, &mut rng, None);
}

pub(crate) fn step_ufo_beam_with_rng(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x: Option<u32>,
    player_y: Option<u32>,
    rng: &mut RomRandomState,
    mut played_sound_effects: Option<&mut Vec<QueuedSoundEffect>>,
) {
    // Active beam sequence: spawn segments at intervals
    if enemy.ufo_beam_segments > 0 {
        let tick =
            ufo_beam_load_tick_loc_34fd6(true, enemy.ufo_beam_segments, enemy.ufo_beam_interval);
        enemy.ufo_beam_segments = tick.next_beam_segments;
        enemy.ufo_beam_interval = tick.next_beam_interval;
        if tick.clear_ufo_shooting_state {
            let clear = ufo_beam_clear_shooting_state_loc_34ffe();
            if !clear.ufo_shooting_latch {
                enemy.ufo_beam_segments = 0;
            }
            let _ = clear.global_shooting_flag;
        }

        if tick.spawn_beam_segment {
            projectiles.push(EnemyProjectile {
                x_pos: enemy.x_pos,
                y_pos: (enemy.y_pos as i64 + (10 << 16)) as u32,
                y_velocity: UFO_BEAM_Y_VELOCITY,
                lifetime: 60,
                kind: EnemyProjectileKind::DragonWalkingBreath,
                ..Default::default()
            });
        }
        return;
    }

    // Check if player is directly below — targeted shoot (ASM ChkShoot)
    if let (Some(px), Some(py)) = (player_x, player_y) {
        let ex = enemy.x_pos >> 16;
        let ey = enemy.y_pos >> 16;
        let kx = px >> 16;
        let ky = py >> 16;
        // Player must be below UFO and within ±2px horizontally
        if ky > ey && kx.abs_diff(ex) <= 2 {
            apply_ufo_beam_start_loc_34eec(enemy);
            if let Some(sound_effects) = played_sound_effects.as_deref_mut() {
                // ASM UFO.asm loc_34EDC: targeted beam starts play
                // `sfx_UFO_shoots` before loading the beam counters.
                sound_effects.push(QueuedSoundEffect::UfoShoots);
            }
            return;
        }
    }

    if ufo_random_beam_should_start_loc_34f8e(rng.next_byte()) {
        apply_ufo_beam_start_loc_34eec(enemy);
        if let Some(sound_effects) = played_sound_effects {
            // ASM UFO.asm loc_34FAE: random beam starts use the same
            // `sfx_UFO_shoots` cue before setting Some_UFO_Shooting.
            sound_effects.push(QueuedSoundEffect::UfoShoots);
        }
    }
}

pub(crate) fn ufo_random_beam_should_start_loc_34f8e(random_byte: u8) -> bool {
    // ASM UFO.asm Enemy0F_UFO_ChkLoadBeam: after `Get_RandomNumber_byte`,
    // `cmpi.w #4,d7; bgt return` starts the beam for byte values 0..=4.
    random_byte <= 4
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UfoCollisionBehaviorLoc35056 {
    BounceWall,
    BounceFloorCeil,
    BounceSlopeUp,
    BounceSlopeDown,
    Hurt,
    ReturnOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoCollisionDispatchLoc35056 {
    pub(crate) collision_type: u16,
    pub(crate) behavior: Option<UfoCollisionBehaviorLoc35056>,
}

pub(crate) fn ufo_collision_dispatch_loc_35056(
    collision_type: i16,
) -> UfoCollisionDispatchLoc35056 {
    // ASM UFO.asm loc_35056: negative values skip to `loc_35070` (clear only),
    // zero returns directly, otherwise table-dispatch uses `collision_type-4` and
    // `move.l (a4,d7.w),a4`.
    if collision_type < 0 {
        let clear = ufo_collision_clear_loc_35070(collision_type as u16);
        return UfoCollisionDispatchLoc35056 {
            collision_type: clear.collision_type,
            behavior: None,
        };
    }

    if collision_type == 0 {
        let clear = ufo_collision_clear_loc_35070(0);
        return UfoCollisionDispatchLoc35056 {
            collision_type: clear.collision_type,
            behavior: None,
        };
    }

    let behavior = match collision_type as u16 {
        0x0004 | 0x0008 => Some(UfoCollisionBehaviorLoc35056::BounceWall),
        0x000C | 0x0010 => Some(UfoCollisionBehaviorLoc35056::BounceFloorCeil),
        0x0014 => Some(UfoCollisionBehaviorLoc35056::BounceSlopeUp),
        0x0018 => Some(UfoCollisionBehaviorLoc35056::BounceSlopeDown),
        0x001C | 0x002C => Some(UfoCollisionBehaviorLoc35056::Hurt),
        0x0020 | 0x0024 | 0x0028 => Some(UfoCollisionBehaviorLoc35056::ReturnOnly),
        _ => None,
    };

    let clear = ufo_collision_clear_loc_35070(collision_type as u16);
    UfoCollisionDispatchLoc35056 {
        collision_type: clear.collision_type,
        behavior,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoCollisionClearLoc35070 {
    pub(crate) collision_type: u16,
}

pub(crate) fn ufo_collision_clear_loc_35070(collision_type: u16) -> UfoCollisionClearLoc35070 {
    // ASM UFO.asm loc_35070: clear collision type and return.
    let _ = collision_type;
    UfoCollisionClearLoc35070 { collision_type: 0 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoPositionRestoreLoc350F2 {
    pub(crate) x_pos: i32,
    pub(crate) y_pos: i32,
}

pub(crate) fn ufo_restore_position_loc_350f2(
    x_pos: i32,
    y_pos: i32,
    x_vel: i32,
    y_vel: i32,
) -> UfoPositionRestoreLoc350F2 {
    // ASM UFO.asm loc_350F2: rollback one frame by subtracting the per-frame
    // velocity delta from object position.
    UfoPositionRestoreLoc350F2 {
        x_pos: x_pos.wrapping_sub(x_vel),
        y_pos: y_pos.wrapping_sub(y_vel),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoWallBounceLoc35120 {
    pub(crate) x_velocity: i32,
    pub(crate) d0: i32,
}

pub(crate) fn ufo_bounce_wall_tail_loc_35120(x_velocity: i32, d0: i32) -> UfoWallBounceLoc35120 {
    // ASM UFO.asm loc_35120: after the wall bounce has negated `x_vel`, keep it.
    // Clear-only behavior means positive `d0` values are negated so they can be
    // used as a bounce impulse; negative `d0` passes through unchanged.
    let x_velocity = x_velocity.wrapping_neg();
    let d0 = if d0 >= 0 { d0.wrapping_neg() } else { d0 };

    UfoWallBounceLoc35120 { x_velocity, d0 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoFloorCeilBounceLoc35146 {
    pub(crate) y_velocity: i32,
    pub(crate) d2: i32,
}

pub(crate) fn ufo_bounce_floor_ceil_tail_loc_35146(
    y_velocity: i32,
    d2: i32,
) -> UfoFloorCeilBounceLoc35146 {
    // ASM UFO.asm loc_35146: after the floor/ceiling bounce has negated `y_vel`,
    // keep it and negate only non-negative `d2` values as a secondary impulse.
    let y_velocity = y_velocity.wrapping_neg();
    let d2 = if d2 >= 0 { d2.wrapping_neg() } else { d2 };

    UfoFloorCeilBounceLoc35146 { y_velocity, d2 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtSpawnTailLoc35174 {
    pub(crate) some_ufo_shooting_cleared: bool,
    pub(crate) allocation_selector: u16,
    pub(crate) driver_routine: &'static str,
    pub(crate) driver_x_velocity: i32,
    pub(crate) driver_timer_flag: i16,
    pub(crate) parent_moved: bool,
    pub(crate) parent_y_velocity: i32,
    pub(crate) driver_animation: &'static str,
    pub(crate) parent_animation_flag_13_cleared: bool,
}

pub(crate) fn ufo_hurt_tail_loc_35174(
    parent_x_velocity: i32,
    some_ufo_shooting: bool,
) -> UfoHurtSpawnTailLoc35174 {
    // ASM UFO.asm loc_35174: clear `$46(a5)`/`Some_UFO_Shooting` only when set,
    // allocate a driver with selector `$6000`, install `loc_355E2`, seed `$44` with
    // parent `x_vel`, set a timer word to `-1`, clear child `$13`, set
    // `is_moved(a3)`, force parent `y_vel` to `$30000`, then run `stru_34CA0`.
    let driver_spawn = ufo_hurt_driver_spawn_loc_35180(parent_x_velocity);
    UfoHurtSpawnTailLoc35174 {
        some_ufo_shooting_cleared: some_ufo_shooting,
        allocation_selector: driver_spawn.allocation_selector,
        driver_routine: driver_spawn.driver_routine,
        driver_x_velocity: driver_spawn.driver_x_velocity,
        driver_timer_flag: driver_spawn.driver_flag_46,
        parent_moved: driver_spawn.parent_moved,
        parent_y_velocity: driver_spawn.parent_y_velocity,
        driver_animation: "stru_34CA0",
        parent_animation_flag_13_cleared: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverSpawnLoc35180 {
    pub(crate) allocation_selector: u16,
    pub(crate) driver_routine: &'static str,
    pub(crate) driver_flag_46: i16,
    pub(crate) driver_x_velocity: i32,
    pub(crate) parent_moved: bool,
    pub(crate) parent_y_velocity: i32,
}

pub(crate) fn ufo_hurt_driver_spawn_loc_35180(
    parent_x_velocity: i32,
) -> UfoHurtDriverSpawnLoc35180 {
    // ASM UFO.asm loc_35180: allocate `$6000` object slot and install `loc_355E2`,
    // set `$46(a0)=$FFFF`, copy parent `x_vel(a3)` into child `current_hp` (word write),
    // clear parent `$13(a1)`, set is_moved(parent), and force parent `$2A(a3)=$30000`.
    UfoHurtDriverSpawnLoc35180 {
        allocation_selector: 0x6000,
        driver_routine: "loc_355E2",
        driver_flag_46: -1,
        driver_x_velocity: parent_x_velocity,
        parent_moved: true,
        parent_y_velocity: 0x0003_0000,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtHibernateGateLoc351B6 {
    pub(crate) keep_hibernating: bool,
    pub(crate) delete_current_object: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoFirstHurtSurviveLoc35228 {
    pub(crate) parent_y_velocity: i32,
    pub(crate) d2: i32,
    pub(crate) d3: i32,
    pub(crate) hurt_primed: bool,
}

pub(crate) fn ufo_first_hurt_survive_loc_35228(
    parent_y_velocity: i32,
) -> UfoFirstHurtSurviveLoc35228 {
    // ASM UFO.asm Enemy0F_UFO_Hurt with `$47(a5)==0` routes through
    // loc_351EE debris allocation, then loc_35228 sets `$47`, seeds the
    // linked object, adds `$40000` to parent y_vel, and returns with
    // `d2=$FFFFF400`/`d3=0` for the flight acceleration registers.
    let finish = ufo_beam_finish_loc_35228(parent_y_velocity);
    debug_assert_eq!(finish.linked_object_meta, 0x000F);
    debug_assert_eq!(finish.linked_object_animation, "stru_34D02");
    debug_assert!(finish.linked_object_flag_13);
    debug_assert!(!finish.linked_object_flag_19);
    debug_assert!(!finish.linked_object_is_moved);
    debug_assert!(!finish.linked_object_level_collision);
    debug_assert_eq!(finish.linked_object_routine, 1);
    debug_assert_eq!(finish.parent_animation, "stru_34C92");
    UfoFirstHurtSurviveLoc35228 {
        parent_y_velocity: finish.parent_y_velocity,
        d2: finish.d2,
        d3: finish.d3,
        hurt_primed: finish.parent_beam_spawned_latch,
    }
}

pub(crate) fn ufo_hurt_hibernate_gate_loc_351b6(
    a3_flag_19_set: bool,
) -> UfoHurtHibernateGateLoc351B6 {
    // ASM UFO.asm loc_351B6: after one-frame hibernate, stay in this routine
    // while `$19(a3)` is zero; delete the object when the byte is nonzero.
    UfoHurtHibernateGateLoc351B6 {
        keep_hibernating: !a3_flag_19_set,
        delete_current_object: a3_flag_19_set,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoBeamSpawnLoc351EE {
    pub(crate) allocation_selector: u16,
    pub(crate) beam_routine: &'static str,
    pub(crate) beam_x_velocity: i32,
    pub(crate) beam_y_velocity: i32,
}

pub(crate) fn ufo_beam_segment_spawn_loc_351f2(
    parent_x_velocity: i32,
    raw_x_velocity: i16,
    raw_y_velocity: i16,
) -> Option<UfoBeamSpawnLoc351EE> {
    // ASM UFO.asm loc_351F2: allocate selector `$8000`, install `loc_354DA`,
    // stop on a zero x-word, then convert table words with `swap; asr.l #1`.
    if raw_x_velocity == 0 {
        return None;
    }

    Some(UfoBeamSpawnLoc351EE {
        allocation_selector: 0x8000,
        beam_routine: "loc_354DA",
        beam_x_velocity: (i32::from(raw_x_velocity) << 15).wrapping_add(parent_x_velocity >> 1),
        beam_y_velocity: i32::from(raw_y_velocity) << 15,
    })
}

pub(crate) fn ufo_beam_spawn_loc_351ee(
    parent_x_velocity: i32,
    beam_velocity_pairs: &[(i16, i16)],
) -> Vec<UfoBeamSpawnLoc351EE> {
    // ASM UFO.asm loc_351EE: load `unk_351C4` and repeatedly allocate
    // `loc_354DA` beams from paired table entries until the first x-word sentinel 0.
    let mut spawns = Vec::new();
    for (raw_x_vel, raw_y_vel) in beam_velocity_pairs {
        let Some(spawn) =
            ufo_beam_segment_spawn_loc_351f2(parent_x_velocity, *raw_x_vel, *raw_y_vel)
        else {
            break;
        };
        spawns.push(spawn);
    }

    spawns
}

const UFO_FIRST_HURT_DEBRIS_VELOCITIES_LOC_351C4: [(i16, i16); 11] = [
    (-5, -3),
    (-3, -4),
    (-6, -5),
    (-4, -7),
    (-2, -6),
    (1, -6),
    (1, -2),
    (3, -4),
    (4, -6),
    (5, -3),
    (0, 0),
];

pub(crate) fn spawn_ufo_first_hurt_debris_loc_351ee(
    projectiles: &mut Vec<EnemyProjectile>,
    parent_x_pos: u32,
    parent_y_pos: u32,
    parent_x_velocity: i32,
    parent_facing: Direction,
) {
    let child_x_direction = match parent_facing {
        Direction::Left => 0,
        Direction::Right => 0xFF,
    };
    let linked_x_pos = (parent_x_pos >> 16) as u16;
    let linked_y_pos = (parent_y_pos >> 16) as u16;

    for debris in ufo_beam_spawn_loc_351ee(
        parent_x_velocity,
        &UFO_FIRST_HURT_DEBRIS_VELOCITIES_LOC_351C4,
    ) {
        let child = ufo_beam_child_init_loc_35512(
            linked_x_pos,
            linked_y_pos,
            child_x_direction,
            debris.beam_x_velocity,
            debris.beam_y_velocity,
        );
        projectiles.push(EnemyProjectile {
            x_pos: u32::from(child.x_pos) << 16,
            y_pos: u32::from(child.y_pos) << 16,
            x_velocity: child.x_velocity,
            y_velocity: child.y_velocity,
            gravity: 0x0000_3000,
            terrain_kill: true,
            lifetime: u16::MAX,
            kind: EnemyProjectileKind::UfoFirstHurtDebris,
            ufo_beam_child_x_direction: child_x_direction,
            ufo_beam_child_loop_timer_loc_3554a: child.loop_timer,
            ..EnemyProjectile::default()
        });
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoBeamChildInitLoc35512 {
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) flag_13: bool,
    pub(crate) is_moved: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) flag_19: bool,
    pub(crate) priority: u8,
    pub(crate) object_meta: u16,
    pub(crate) sprite_mapping: &'static str,
    pub(crate) loop_timer: u16,
}

pub(crate) fn ufo_beam_child_init_loc_35512(
    linked_x_pos: u16,
    linked_y_pos: u16,
    child_x_direction: u8,
    source_x_velocity: i32,
    source_y_velocity: i32,
) -> UfoBeamChildInitLoc35512 {
    // ASM UFO.asm loc_35512: loc_354DA has just copied `$44/$48(a5)`, placed X at
    // linked `$1A(a0)-$B` and mirrors it to `+$B` for non-zero `x_direction(a3)`.
    let x_pos = linked_x_pos.wrapping_sub(0x000B);
    let x_pos = if child_x_direction == 0 {
        x_pos
    } else {
        x_pos.wrapping_add(0x0016)
    };

    UfoBeamChildInitLoc35512 {
        x_pos,
        y_pos: linked_y_pos.wrapping_sub(0x0018),
        x_velocity: source_x_velocity,
        y_velocity: source_y_velocity,
        flag_13: true,
        is_moved: true,
        has_level_collision: false,
        flag_19: false,
        priority: 1,
        object_meta: 0x000F,
        sprite_mapping: "LnkTo_unk_C8408",
        loop_timer: 5,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoBeamChildTurnaroundLoc3554A {
    pub(crate) loop_timer: u16,
    pub(crate) x_direction: u8,
    pub(crate) flipped_direction: bool,
}

pub(crate) fn ufo_beam_child_turnaround_loc_3554a(
    loop_timer: u16,
    x_direction: u8,
) -> UfoBeamChildTurnaroundLoc3554A {
    // ASM UFO.asm loc_3554A: after hibernating one frame, decrement d0; only
    // the zero result runs `not.b x_direction(a3)` and reloads d0 to `$A`.
    let loop_timer = loop_timer.wrapping_sub(1);
    if loop_timer == 0 {
        UfoBeamChildTurnaroundLoc3554A {
            loop_timer: 0x000A,
            x_direction: !x_direction,
            flipped_direction: true,
        }
    } else {
        UfoBeamChildTurnaroundLoc3554A {
            loop_timer,
            x_direction,
            flipped_direction: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoBeamChildGravityGateLoc3555A {
    pub(crate) y_velocity: i32,
    pub(crate) keep_hibernating: bool,
    pub(crate) delete_current_object: bool,
}

pub(crate) fn ufo_beam_child_gravity_gate_loc_3555a(
    y_velocity: i32,
    flag_19_set: bool,
) -> UfoBeamChildGravityGateLoc3555A {
    // ASM UFO.asm loc_3555A: add `$3000` to `y_vel(a3)` every loop and delete
    // only after `$19(a3)` becomes non-zero; otherwise branch back to loc_3554A.
    let y_velocity = y_velocity.wrapping_add(0x0000_3000);
    UfoBeamChildGravityGateLoc3555A {
        y_velocity,
        keep_hibernating: !flag_19_set,
        delete_current_object: flag_19_set,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct UfoHurtDriverFallLoopLoc3564C {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) range_check_mode: u8,
    pub(crate) y_velocity: i32,
    pub(crate) collision_table: &'static str,
    pub(crate) collision_dispatch_mode: u8,
    pub(crate) loops_forever: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_fall_loop_loc_3564c(
    y_velocity: i32,
) -> UfoHurtDriverFallLoopLoc3564C {
    // ASM UFO.asm loc_3564C: after one-frame hibernate, call Object_CheckInRange
    // with d0=1, add `$2000` to `y_vel(a3)`, dispatch through `off_3569E`,
    // then branch back to the same loop.
    UfoHurtDriverFallLoopLoc3564C {
        hibernates_one_frame: true,
        range_check_mode: 1,
        y_velocity: y_velocity.wrapping_add(0x0000_2000),
        collision_table: "off_3569E",
        collision_dispatch_mode: 1,
        loops_forever: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverSlideSeedLoc3566C {
    pub(crate) x_velocity: i32,
    pub(crate) falls_through_to_slide_loop: bool,
}

pub(crate) fn ufo_hurt_driver_slide_seed_loc_3566c(
    x_direction: u8,
) -> UfoHurtDriverSlideSeedLoc3566C {
    // ASM UFO.asm loc_3566C: seed `x_vel(a3)` with `$FFFF8000`; a non-zero
    // raw `x_direction(a3)` negates that longword before falling into the slide loop.
    let leftward_velocity: i32 = -0x0000_8000;
    let x_velocity = if x_direction == 0 {
        leftward_velocity
    } else {
        leftward_velocity.wrapping_neg()
    };

    UfoHurtDriverSlideSeedLoc3566C {
        x_velocity,
        falls_through_to_slide_loop: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverSlideLoopLoc35680 {
    pub(crate) has_kid_collision: bool,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) checks_in_range: bool,
    pub(crate) collision_table: &'static str,
    pub(crate) collision_dispatch_mode: u8,
    pub(crate) checks_ground_tile: bool,
    pub(crate) loops_forever: bool,
}

pub(crate) fn ufo_hurt_driver_slide_loop_loc_35680() -> UfoHurtDriverSlideLoopLoc35680 {
    // ASM UFO.asm loc_35680: clear `has_kid_collision(a3)`, hibernate one
    // frame, range-check, dispatch collisions through `off_356B6` with d0=0,
    // probe the ground tile, then branch back to the same slide loop.
    UfoHurtDriverSlideLoopLoc35680 {
        has_kid_collision: false,
        hibernates_one_frame: true,
        checks_in_range: true,
        collision_table: "off_356B6",
        collision_dispatch_mode: 0,
        checks_ground_tile: true,
        loops_forever: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverSlideTurnaroundLoc356CE {
    pub(crate) x_velocity: i32,
    pub(crate) x_pos: u32,
    pub(crate) x_direction: u8,
}

pub(crate) fn ufo_hurt_driver_slide_turnaround_loc_356ce(
    x_velocity: i32,
    x_pos: u32,
    x_direction: u8,
) -> UfoHurtDriverSlideTurnaroundLoc356CE {
    // ASM UFO.asm loc_356CE: negate `x_vel(a3)`, advance `x_pos(a3)` by that
    // negated velocity shifted left two bits, then `not.b x_direction(a3)`.
    let x_velocity = x_velocity.wrapping_neg();
    let displacement = x_velocity.wrapping_shl(2);
    UfoHurtDriverSlideTurnaroundLoc356CE {
        x_velocity,
        x_pos: x_pos.wrapping_add(u32::from_ne_bytes(displacement.to_ne_bytes())),
        x_direction: !x_direction,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UfoHurtDriverSlideGroundMissLoc356F2 {
    Turnaround(UfoHurtDriverSlideTurnaroundLoc356CE),
    RelaunchFall {
        x_velocity: i32,
        y_velocity: i32,
        next_label: &'static str,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_slide_ground_miss_loc_356f2(
    first_rng_byte: u8,
    second_rng_byte: u8,
    x_velocity: i32,
    x_pos: u32,
    y_velocity: i32,
    x_direction: u8,
) -> UfoHurtDriverSlideGroundMissLoc356F2 {
    // ASM UFO.asm loc_356F2: `(rng & $2F) > $0A` jumps to loc_356CE.
    if u16::from(first_rng_byte & 0x2F) > 0x000A {
        return UfoHurtDriverSlideGroundMissLoc356F2::Turnaround(
            ufo_hurt_driver_slide_turnaround_loc_356ce(x_velocity, x_pos, x_direction),
        );
    }

    // ASM UFO.asm loc_356F2: the second `(rng & 6)` selects word_356E4 entries
    // `$FFFF/$FFFE/$FFFD/$FFFC`, writes only the high word of `y_vel(a3)`,
    // doubles `x_vel(a3)`, then branches to loc_3564C.
    const Y_VELOCITY_HIGH_WORDS: [i16; 4] = [-1, -2, -3, -4];
    let table_index = usize::from((second_rng_byte & 0x06) >> 1);
    let y_velocity =
        (i32::from(Y_VELOCITY_HIGH_WORDS[table_index]) << 16) | (y_velocity & 0x0000_FFFF);

    UfoHurtDriverSlideGroundMissLoc356F2::RelaunchFall {
        x_velocity: x_velocity.wrapping_shl(1),
        y_velocity,
        next_label: "loc_3564C",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverSlideCollisionReturnLoc3571E {
    pub(crate) stack_bytes_discarded: u16,
    pub(crate) next_label: &'static str,
    pub(crate) slide_loop: UfoHurtDriverSlideLoopLoc35680,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_slide_collision_return_loc_3571e()
-> UfoHurtDriverSlideCollisionReturnLoc3571E {
    // ASM UFO.asm loc_3571E: `addq.w #4,sp` discards the collision dispatch
    // return address, then branches straight back to the loc_35680 slide loop.
    UfoHurtDriverSlideCollisionReturnLoc3571E {
        stack_bytes_discarded: 4,
        next_label: "loc_35680",
        slide_loop: ufo_hurt_driver_slide_loop_loc_35680(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverSlideProbeXLoc35724 {
    pub(crate) probe_x_word: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_slide_probe_x_loc_35724(
    x_pos: u32,
    x_direction: u8,
) -> UfoHurtDriverSlideProbeXLoc35724 {
    // ASM UFO.asm loc_35724: read the integer word of `x_pos(a3)`, subtract
    // `$0F`, and add `$1E` back for any non-zero raw `x_direction(a3)`.
    let [b0, b1, _, _] = x_pos.to_be_bytes();
    let mut probe_x_word = u16::from_be_bytes([b0, b1]).wrapping_sub(0x000F);
    if x_direction != 0 {
        probe_x_word = probe_x_word.wrapping_add(0x001E);
    }

    UfoHurtDriverSlideProbeXLoc35724 { probe_x_word }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverSlideGroundTileLoc35738 {
    pub(crate) probe_y_word: u16,
    pub(crate) tile_column: u16,
    pub(crate) tile_row: u16,
    pub(crate) tile_collision_nibble: u16,
    pub(crate) ground_present: bool,
    pub(crate) miss_label: Option<&'static str>,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_slide_ground_tile_loc_35738(
    y_pos: u32,
    probe_x_word: u16,
    tile_word: u16,
) -> UfoHurtDriverSlideGroundTileLoc35738 {
    // ASM UFO.asm loc_35738: add 8px to the integer `y_pos(a3)` word, use
    // logical word shifts to index the level-layout tile, and require the
    // tile collision nibble to equal `$6000`; any other nibble branches to
    // loc_356F2.
    let [b0, b1, _, _] = y_pos.to_be_bytes();
    let probe_y_word = u16::from_be_bytes([b0, b1]).wrapping_add(0x0008);
    let tile_collision_nibble = tile_word & 0x7000;
    let ground_present = tile_collision_nibble == 0x6000;

    UfoHurtDriverSlideGroundTileLoc35738 {
        probe_y_word,
        tile_column: probe_x_word >> 4,
        tile_row: probe_y_word >> 4,
        tile_collision_nibble,
        ground_present,
        miss_label: if ground_present {
            None
        } else {
            Some("loc_356F2")
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum UfoHurtDriverCollisionTableLoc35762 {
    FallLoop,
    SlideLoop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UfoHurtDriverCollisionBehaviorLoc35762 {
    FallTurnaround,
    FallFloorLanding,
    FallCeilingBounce,
    FallSlopeUpBounce,
    FallSlopeDownBounce,
    SlideTurnaround,
    SlideCollisionReturn,
    SlideGroundMiss,
    StopAndDeleteAfterAnimation,
    StopAndFallUntilDeleted,
    BerzerkerChargeFall { x_velocity: i32, y_velocity: i32 },
    ReturnOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverCollisionDispatchLoc35762 {
    pub(crate) clears_collision_type: bool,
    pub(crate) behavior: Option<UfoHurtDriverCollisionBehaviorLoc35762>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverBerzerkerChargeFallLoc35788 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) has_level_collision: bool,
    pub(crate) animation: &'static str,
    pub(crate) enters_falling_loop: bool,
}

pub(crate) fn ufo_hurt_driver_berzerker_charge_fall_loc_35788(
    kid_x_velocity: i32,
) -> UfoHurtDriverBerzerkerChargeFallLoc35788 {
    // ASM UFO.asm loc_35788: when `$20/$24` collision reaches this branch with
    // `Berzerker_charging` set, copy half Kid x_vel into the UFO, set y_vel to
    // `-$40000`, clear `has_level_collision(a3)`, start `stru_34CAE`, then
    // enter the shared falling loop.
    UfoHurtDriverBerzerkerChargeFallLoc35788 {
        x_velocity: kid_x_velocity >> 1,
        y_velocity: -0x0004_0000,
        has_level_collision: false,
        animation: "stru_34CAE",
        enters_falling_loop: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverStopTailLoc357C2 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) has_level_collision: bool,
    pub(crate) animation: &'static str,
    pub(crate) enters_falling_loop: bool,
    pub(crate) waits_for_animation_before_delete: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_stop_tail_loc_357C2(
    init_animation_result: i16,
) -> UfoHurtDriverStopTailLoc357C2 {
    // ASM UFO.asm loc_357C2: clear both velocity longwords, clear
    // `has_level_collision(a3)`, initialise `stru_34CAE`, and use the returned
    // d0 word to choose between entering the falling loop or waiting for the
    // animation to finish before deleting the object.
    UfoHurtDriverStopTailLoc357C2 {
        x_velocity: 0,
        y_velocity: 0,
        has_level_collision: false,
        animation: "stru_34CAE",
        enters_falling_loop: init_animation_result != 0,
        waits_for_animation_before_delete: init_animation_result == 0,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverDeleteTailLoc357E2 {
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_delete_tail_loc_357E2() -> UfoHurtDriverDeleteTailLoc357E2 {
    // ASM UFO.asm loc_357E2: jump directly through `j_Delete_CurrentObject`;
    // no velocity, collision, or animation state is changed in this tail.
    UfoHurtDriverDeleteTailLoc357E2 {
        deletes_current_object: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverFallingLoopLoc357E6 {
    pub(crate) y_velocity: i32,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) deletes_current_object: bool,
    pub(crate) next_label: &'static str,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_falling_loop_loc_357E6(
    y_velocity: i32,
    animation_finished_flag: bool,
) -> UfoHurtDriverFallingLoopLoc357E6 {
    // ASM UFO.asm loc_357E6: hibernate one frame, add `$4000` to `y_vel(a3)`,
    // and keep looping until byte `$19(a3)` is set, then branch to loc_357E2.
    UfoHurtDriverFallingLoopLoc357E6 {
        y_velocity: y_velocity.wrapping_add(0x0000_4000),
        hibernates_one_frame: true,
        deletes_current_object: animation_finished_flag,
        next_label: if animation_finished_flag {
            "loc_357E2"
        } else {
            "loc_357E6"
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverFallTurnaroundLoc357FA {
    pub(crate) x_velocity: i32,
    pub(crate) x_pos: u32,
    pub(crate) x_direction: u8,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_fall_turnaround_loc_357FA(
    x_velocity: i32,
    x_pos: u32,
    x_direction: u8,
) -> UfoHurtDriverFallTurnaroundLoc357FA {
    // ASM UFO.asm loc_357FA: negate `x_vel(a3)`, shift the reversed velocity
    // left two bits into a four-frame displacement, add it to `x_pos(a3)`,
    // then flip the raw `x_direction(a3)` byte with `not.b`.
    let x_velocity = x_velocity.wrapping_neg();
    let displacement = x_velocity.wrapping_shl(2);
    UfoHurtDriverFallTurnaroundLoc357FA {
        x_velocity,
        x_pos: x_pos.wrapping_add(u32::from_ne_bytes(displacement.to_ne_bytes())),
        x_direction: !x_direction,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverVerticalBounceLoc35810 {
    pub(crate) y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_vertical_bounce_loc_35810(
    y_velocity: i32,
) -> UfoHurtDriverVerticalBounceLoc35810 {
    // ASM UFO.asm loc_35810: copy `y_vel(a3)` to d7, `neg.l` it, then
    // `asr.l #1` before writing the half-speed rebound back to `y_vel(a3)`.
    UfoHurtDriverVerticalBounceLoc35810 {
        y_velocity: y_velocity.wrapping_neg() >> 1,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverFloorLandingLoc3581E {
    pub(crate) stack_bytes_discarded: u16,
    pub(crate) y_velocity: i32,
    pub(crate) y_pos: u32,
    pub(crate) next_label: &'static str,
    pub(crate) slide_seed: UfoHurtDriverSlideSeedLoc3566C,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_floor_landing_loc_3581E(
    y_pos: u32,
    x_direction: u8,
) -> UfoHurtDriverFloorLandingLoc3581E {
    // ASM UFO.asm loc_3581E: discard the collision return, clear `y_vel(a3)`,
    // clear the y-position fraction word, add 8 to the integer word, align
    // down to a 16px boundary, then branch to loc_3566C.
    let aligned_y_word = ((y_pos >> 16) as u16).wrapping_add(8) & 0xFFF0;

    UfoHurtDriverFloorLandingLoc3581E {
        stack_bytes_discarded: 4,
        y_velocity: 0,
        y_pos: u32::from(aligned_y_word) << 16,
        next_label: "loc_3566C",
        slide_seed: ufo_hurt_driver_slide_seed_loc_3566c(x_direction),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverLaunchLoc35838 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_up_left_launch_loc_35838() -> UfoHurtDriverLaunchLoc35838 {
    // ASM UFO.asm loc_35838: write `#-$10000` to both `x_vel(a3)` and
    // `y_vel(a3)`, then return.
    UfoHurtDriverLaunchLoc35838 {
        x_velocity: -0x0001_0000,
        y_velocity: -0x0001_0000,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoHurtDriverLaunchLoc35864 {
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_up_right_launch_loc_35864() -> UfoHurtDriverLaunchLoc35864 {
    // ASM UFO.asm loc_35864: write `#$10000` to `x_vel(a3)`, write
    // `#-$10000` to `y_vel(a3)`, then return.
    UfoHurtDriverLaunchLoc35864 {
        x_velocity: 0x0001_0000,
        y_velocity: -0x0001_0000,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_hurt_driver_collision_dispatch_loc_35762(
    collision_type: i16,
    table: UfoHurtDriverCollisionTableLoc35762,
    berzerker_charging: bool,
    kid_x_velocity: i32,
) -> UfoHurtDriverCollisionDispatchLoc35762 {
    // ASM UFO.asm loc_35762: zero returns without clearing, negative values
    // route straight to the hurt tail, positive non-zero values clear
    // `collision_type(a3)` before either table-dispatching or returning.
    if collision_type == 0 {
        return UfoHurtDriverCollisionDispatchLoc35762 {
            clears_collision_type: false,
            behavior: None,
        };
    }

    let hurt_behavior = match table {
        UfoHurtDriverCollisionTableLoc35762::FallLoop => {
            UfoHurtDriverCollisionBehaviorLoc35762::StopAndFallUntilDeleted
        }
        UfoHurtDriverCollisionTableLoc35762::SlideLoop => {
            UfoHurtDriverCollisionBehaviorLoc35762::StopAndDeleteAfterAnimation
        }
    };

    if collision_type < 0 {
        return UfoHurtDriverCollisionDispatchLoc35762 {
            clears_collision_type: false,
            behavior: Some(hurt_behavior),
        };
    }

    let collision_word = u16::from_be_bytes(collision_type.to_be_bytes());
    if collision_word == 0x002C || collision_word == 0x001C {
        return UfoHurtDriverCollisionDispatchLoc35762 {
            clears_collision_type: true,
            behavior: Some(hurt_behavior),
        };
    }

    if collision_word > 0x001C {
        let behavior = if collision_word < 0x0028 && berzerker_charging {
            let fall = ufo_hurt_driver_berzerker_charge_fall_loc_35788(kid_x_velocity);
            UfoHurtDriverCollisionBehaviorLoc35762::BerzerkerChargeFall {
                x_velocity: fall.x_velocity,
                y_velocity: fall.y_velocity,
            }
        } else {
            UfoHurtDriverCollisionBehaviorLoc35762::ReturnOnly
        };

        return UfoHurtDriverCollisionDispatchLoc35762 {
            clears_collision_type: true,
            behavior: Some(behavior),
        };
    }

    let behavior = match (table, collision_word) {
        (UfoHurtDriverCollisionTableLoc35762::FallLoop, 0x0004 | 0x0008) => {
            UfoHurtDriverCollisionBehaviorLoc35762::FallTurnaround
        }
        (UfoHurtDriverCollisionTableLoc35762::FallLoop, 0x000C) => {
            UfoHurtDriverCollisionBehaviorLoc35762::FallFloorLanding
        }
        (UfoHurtDriverCollisionTableLoc35762::FallLoop, 0x0010) => {
            UfoHurtDriverCollisionBehaviorLoc35762::FallCeilingBounce
        }
        (UfoHurtDriverCollisionTableLoc35762::FallLoop, 0x0014) => {
            UfoHurtDriverCollisionBehaviorLoc35762::FallSlopeUpBounce
        }
        (UfoHurtDriverCollisionTableLoc35762::FallLoop, 0x0018) => {
            UfoHurtDriverCollisionBehaviorLoc35762::FallSlopeDownBounce
        }
        (UfoHurtDriverCollisionTableLoc35762::SlideLoop, 0x0004 | 0x0008) => {
            UfoHurtDriverCollisionBehaviorLoc35762::SlideTurnaround
        }
        (UfoHurtDriverCollisionTableLoc35762::SlideLoop, 0x000C | 0x0010) => {
            UfoHurtDriverCollisionBehaviorLoc35762::SlideCollisionReturn
        }
        (UfoHurtDriverCollisionTableLoc35762::SlideLoop, 0x0014 | 0x0018) => {
            UfoHurtDriverCollisionBehaviorLoc35762::SlideGroundMiss
        }
        _ => UfoHurtDriverCollisionBehaviorLoc35762::ReturnOnly,
    };

    UfoHurtDriverCollisionDispatchLoc35762 {
        clears_collision_type: true,
        behavior: Some(behavior),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoBeamFinishLoc35228 {
    pub(crate) parent_beam_spawned_latch: bool,
    pub(crate) linked_object_meta: u16,
    pub(crate) linked_object_animation: &'static str,
    pub(crate) linked_object_flag_13: bool,
    pub(crate) linked_object_flag_19: bool,
    pub(crate) linked_object_is_moved: bool,
    pub(crate) linked_object_level_collision: bool,
    pub(crate) linked_object_routine: u8,
    pub(crate) parent_animation: &'static str,
    pub(crate) parent_y_velocity: i32,
    pub(crate) d2: i32,
    pub(crate) d3: i32,
}

pub(crate) fn ufo_beam_finish_loc_35228(parent_y_velocity: i32) -> UfoBeamFinishLoc35228 {
    // ASM UFO.asm loc_35228: mark `$47(a5)`, link the loc_32188 gfx object,
    // initialise it as UFO object meta `$0F`, reset its state flags, replay the
    // parent `stru_34C92` animation, then bias parent y velocity downward.
    UfoBeamFinishLoc35228 {
        parent_beam_spawned_latch: true,
        linked_object_meta: 0x000F,
        linked_object_animation: "stru_34D02",
        linked_object_flag_13: true,
        linked_object_flag_19: false,
        linked_object_is_moved: false,
        linked_object_level_collision: false,
        linked_object_routine: 1,
        parent_animation: "stru_34C92",
        parent_y_velocity: parent_y_velocity.wrapping_add(0x0004_0000),
        d2: -0x0000_0C00,
        d3: 0,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum UfoLinkedObjectRangeGateLoc35280 {
    Return,
    RunCleanupDeleteTail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoLinkedCleanupCountLoc3528A {
    pub(crate) number_ufos_on_screen: u16,
    pub(crate) play_hover_sound: bool,
}

pub(crate) fn ufo_linked_cleanup_count_loc_3528a(
    number_ufos_on_screen: u16,
    ufo_on_screen_latch: bool,
) -> UfoLinkedCleanupCountLoc3528A {
    // ASM UFO.asm loc_3528A: if `$50(a5)` is clear, skip the global
    // `Number_UFOs_OnScreen` decrement; otherwise decrement and replay
    // `sfx_UFO_hovering` only when the word reaches zero.
    if !ufo_on_screen_latch {
        return UfoLinkedCleanupCountLoc3528A {
            number_ufos_on_screen,
            play_hover_sound: false,
        };
    }

    let number_ufos_on_screen = number_ufos_on_screen.wrapping_sub(1);
    UfoLinkedCleanupCountLoc3528A {
        number_ufos_on_screen,
        play_hover_sound: number_ufos_on_screen == 0,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UfoLinkedCleanupShootingLoc352A6 {
    pub(crate) global_shooting_flag: bool,
}

pub(crate) fn ufo_linked_cleanup_shooting_loc_352a6(
    ufo_shooting_latch: bool,
    global_shooting_flag: bool,
) -> UfoLinkedCleanupShootingLoc352A6 {
    // ASM UFO.asm loc_352A6: test `$46(a5)` and clear `Some_UFO_Shooting`
    // only when this UFO's shooting latch is set.
    UfoLinkedCleanupShootingLoc352A6 {
        global_shooting_flag: if ufo_shooting_latch {
            false
        } else {
            global_shooting_flag
        },
    }
}

pub(crate) fn ufo_linked_cleanup_status_loc_352b2(
    object_status_slot: u8,
) -> DeactivationStatusActionLoc3C4C0 {
    // ASM UFO.asm loc_352B2: read `$42(a5)` into d0, update the corresponding
    // `EnemyStatus_Table` slot for positive or bit-6 sentinel values, then
    // delete the current object.
    // ASM UFO.asm loc_352D2: positive slots write `$2168` as the respawn marker.
    // ASM UFO.asm loc_352E0: every branch reaches `j_Delete_CurrentObject`.
    deactivation_status_update_loc_3C4C0(object_status_slot)
}

pub(crate) fn ufo_linked_object_x_loc_3531a(parent_x_pos: u32, parent_facing: Direction) -> u16 {
    // ASM UFO.asm loc_3531A: after `Enemy0F_UFO_SpeedToPos`, the linked object
    // gets the parent's integer X word minus d6. Clear `x_direction` leaves
    // d6 as +4; set `x_direction` negates d6 to -4, so the subtraction adds 4.
    let parent_x_word = (parent_x_pos >> 16) as u16;
    let d6 = match parent_facing {
        Direction::Left => 4_u16,
        Direction::Right => 0xFFFC_u16,
    };
    parent_x_word.wrapping_sub(d6)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn ufo_linked_object_range_gate_loc_35280(
    range_check_result_nonzero: bool,
) -> UfoLinkedObjectRangeGateLoc35280 {
    // ASM UFO.asm loc_35280: after the range-check subroutine, a zero result returns
    // immediately; any non-zero result falls through into the cleanup/delete tail.
    if range_check_result_nonzero {
        UfoLinkedObjectRangeGateLoc35280::RunCleanupDeleteTail
    } else {
        UfoLinkedObjectRangeGateLoc35280::Return
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    #[test]
    fn ufo_live_step_queues_shoot_sound_when_targeted_beam_starts() {
        let ufo = ActiveEnemy {
            family: EnemyFamily::UFO,
            hit_points: 2,
            x_pos: 100 << 16,
            y_pos: 80 << 16,
            ..mini_hopping_skull_test_enemy()
        };
        let mut enemies = [ufo];
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();
        let mut projectiles = Vec::new();
        let mut sound_effects = Vec::new();

        step_active_enemies_with_rng(
            &mut enemies,
            None,
            Some(100 << 16),
            Some(100 << 16),
            None,
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
        );

        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::UfoShoots, QueuedSoundEffect::UfoHovering],
            "ASM UFO.asm loc_34EDC plays `sfx_UFO_shoots` when Kid is directly below, then loc_34F5A starts the hover loop when the unlatched UFO reaches ChkOnScreen",
        );
        assert_eq!(sound_effects[0].sound_id(), 0x3A);
        assert_eq!(sound_effects[1].sound_id(), 0x19);
        assert_eq!(
            enemies[0].ufo_beam_segments, UFO_BEAM_SEGMENT_COUNT,
            "ASM UFO.asm loc_34EEC should arm the beam segment counter after the start cue",
        );
    }

    #[test]
    fn ufo_motion_scaling_matches_loc_34e10_acceleration_gate() {
        let command = UfoMotionCommand {
            x_acceleration: -0x1000,
            target_x_velocity: -0x20000,
            y_acceleration: 0x0800,
            target_y_velocity: 0x18000,
        };

        assert_eq!(
            ufo_double_acceleration_loc_34e10(command, false),
            command,
            "ASM UFO.asm loc_34E10 falls through without changing d0/d2 when `$4E(a5)` is zero"
        );
        assert_eq!(
            ufo_double_acceleration_loc_34e10(command, true),
            UfoMotionCommand {
                x_acceleration: -0x2000,
                target_x_velocity: -0x20000,
                y_acceleration: 0x1000,
                target_y_velocity: 0x18000,
            },
            "ASM UFO.asm loc_34E10 doubles acceleration registers d0/d2 while leaving target speed d1/d3 unchanged"
        );
    }

    #[test]
    fn ufo_motion_scaling_matches_loc_34e1c_terminal_gate() {
        let command = UfoMotionCommand {
            x_acceleration: -0x1000,
            target_x_velocity: -0x20000,
            y_acceleration: 0x0800,
            target_y_velocity: 0x18000,
        };

        assert_eq!(
            ufo_double_terminal_speed_loc_34e1c(command, false),
            command,
            "ASM UFO.asm loc_34E1C returns without changing d1/d3 when `$4F(a5)` is zero"
        );
        assert_eq!(
            ufo_double_terminal_speed_loc_34e1c(command, true),
            UfoMotionCommand {
                x_acceleration: -0x1000,
                target_x_velocity: -0x40000,
                y_acceleration: 0x0800,
                target_y_velocity: 0x30000,
            },
            "ASM UFO.asm loc_34E1C doubles terminal-speed registers d1/d3 while leaving acceleration d0/d2 unchanged"
        );
    }

    #[test]
    fn ufo_tracking_matches_loc_34e2a_below_near_kid_branch() {
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x0120_8000,
                0x0080_4000,
                0x0100_0000,
                0x0090_0000,
                true,
                false,
                true,
                true,
            ),
            Some(UfoTrackingCommandLoc34E2A {
                motion: UfoMotionCommand {
                    x_acceleration: -0x2000,
                    target_x_velocity: -0x40000,
                    y_acceleration: 0,
                    target_y_velocity: 0,
                },
                y_velocity: 0,
                facing: Direction::Left,
            }),
            "ASM UFO.asm loc_34E2A clears y_vel, steers left when UFO X is not less than Kid X, then calls loc_34E10 scaling"
        );

        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x00D0_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                true,
                false,
                false,
                false,
            ),
            Some(UfoTrackingCommandLoc34E2A {
                motion: UfoMotionCommand {
                    x_acceleration: 0x1000,
                    target_x_velocity: 0x20000,
                    y_acceleration: 0,
                    target_y_velocity: 0,
                },
                y_velocity: 0,
                facing: Direction::Right,
            }),
            "ASM UFO.asm loc_34E2A includes the Kid X - $30 edge and steers right when UFO X is less than Kid X"
        );
    }

    #[test]
    fn ufo_motion_picker_matches_loc_35326_selector_and_loc_35444_scaling() {
        let (motion, facing) =
            ufo_motion_picker_loc_35326(0xC2, 0x0120_0000, 0x0002_0000, 0, true, true);

        assert_eq!(
            motion,
            UfoMotionCommand {
                target_x_velocity: 0x30000,
                target_y_velocity: -0x30000,
                x_acceleration: -0x1800,
                y_acceleration: -0x1800,
            },
            "ASM UFO.asm loc_35326 uses `(random_byte ^ x_pos.low) & $F`, then loc_35444 derives acceleration signs before loc_34E10/loc_34E1C scaling"
        );
        assert_eq!(
            facing,
            Direction::Right,
            "ASM UFO.asm loc_35444 leaves `x_direction` set when the selected X target speed is non-negative"
        );
    }

    #[test]
    fn ufo_tracking_right_command_matches_loc_34e86() {
        assert_eq!(
            ufo_right_tracking_command_loc_34e86(),
            UfoMotionCommand {
                x_acceleration: 0x1000,
                target_x_velocity: 0x20000,
                y_acceleration: 0,
                target_y_velocity: 0,
            },
            "ASM UFO.asm loc_34E86 writes positive d0/d1 for rightward Kid chase before setting flipped x_direction"
        );

        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x00D0_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                true,
                false,
                false,
                false,
            )
            .map(|command| command.facing),
            Some(Direction::Right),
            "ASM UFO.asm loc_34E86 reaches `st x_direction(a3)`, represented by the right-facing/flipped native command"
        );
    }

    #[test]
    fn ufo_tracking_tail_matches_loc_34e96() {
        let command = UfoMotionCommand {
            x_acceleration: 0x1000,
            target_x_velocity: 0x20000,
            y_acceleration: -0x0800,
            target_y_velocity: -0x18000,
        };

        assert_eq!(
            ufo_tracking_tail_loc_34e96(command, false, false),
            command,
            "ASM UFO.asm loc_34E96 calls loc_34E10; with both scale flags clear the motion command is unchanged"
        );
        assert_eq!(
            ufo_tracking_tail_loc_34e96(command, true, true),
            UfoMotionCommand {
                x_acceleration: 0x2000,
                target_x_velocity: 0x40000,
                y_acceleration: -0x1000,
                target_y_velocity: -0x30000,
            },
            "ASM UFO.asm loc_34E96 shares the loc_34E10 acceleration and terminal-speed scaling tail"
        );
    }

    #[test]
    fn ufo_beam_start_matches_loc_34eec() {
        assert_eq!(
            ufo_beam_start_loc_34eec(),
            UfoBeamStartLoc34Eec {
                beam_segments: 4,
                beam_interval: 5,
                x_velocity: 0,
                y_velocity: 0,
                wait_timer: 0x32,
            },
            "ASM UFO.asm loc_34EEC writes `$48=4`, `$4A=5`, clears x/y velocity, and reloads `$44=$32`"
        );
    }

    #[test]
    fn ufo_random_beam_gate_matches_loc_34f8e_rng_threshold() {
        assert!(
            ufo_random_beam_should_start_loc_34f8e(4),
            "ASM UFO.asm Enemy0F_UFO_ChkLoadBeam starts when random byte is equal to 4"
        );
        assert!(
            !ufo_random_beam_should_start_loc_34f8e(5),
            "ASM UFO.asm Enemy0F_UFO_ChkLoadBeam returns for random bytes greater than 4"
        );
    }

    #[test]
    fn ufo_beam_load_tick_matches_loc_34fd6() {
        assert_eq!(
            ufo_beam_load_tick_loc_34fd6(false, 4, 5),
            UfoBeamLoadTickLoc34FD6 {
                next_beam_segments: 4,
                next_beam_interval: 5,
                spawn_beam_segment: false,
                clear_ufo_shooting_state: false,
            },
            "ASM UFO.asm loc_34FD6 does nothing while `$46(a5)` is clear"
        );

        assert_eq!(
            ufo_beam_load_tick_loc_34fd6(true, 4, 1),
            UfoBeamLoadTickLoc34FD6 {
                next_beam_segments: 3,
                next_beam_interval: 5,
                spawn_beam_segment: true,
                clear_ufo_shooting_state: false,
            },
            "ASM UFO.asm loc_34FD6 reloads the 5-frame interval counter and decrements `$48(a5)` on each segment"
        );

        assert_eq!(
            ufo_beam_load_tick_loc_34fd6(true, 1, 1),
            UfoBeamLoadTickLoc34FD6 {
                next_beam_segments: 0,
                next_beam_interval: 5,
                spawn_beam_segment: false,
                clear_ufo_shooting_state: true,
            },
            "ASM UFO.asm loc_34FD6 enters loc_34FFE when the segment countdown reaches zero"
        );
    }

    #[test]
    fn ufo_apply_acceleration_axis_matches_loc_3501e() {
        assert_eq!(
            ufo_apply_acceleration_axis_loc_3501e(0x0800, 0x0300, -0x0100),
            0x0700,
            "ASM UFO.asm loc_3501E keeps accelerating while the next value stays above target"
        );
        assert_eq!(
            ufo_apply_acceleration_axis_loc_3501e(0x0800, 0x0700, -0x0200),
            0x0700,
            "ASM UFO.asm loc_3501E clamps to target when the next value would undershoot"
        );
    }

    #[test]
    fn ufo_apply_acceleration_axis_matches_loc_35044() {
        assert_eq!(
            ufo_apply_acceleration_axis_loc_35044(0x0300, 0x0180, -0x0100),
            0x0200,
            "ASM UFO.asm loc_35044 keeps accelerating on the negative branch"
        );
        assert_eq!(
            ufo_apply_acceleration_axis_loc_35044(0x0300, 0x0201, -0x0100),
            0x0201,
            "ASM UFO.asm loc_35044 clamps to target when the next value would pass below"
        );
    }

    #[test]
    fn ufo_apply_acceleration_axis_matches_loc_35050() {
        assert_eq!(
            ufo_apply_acceleration_axis_loc_35050(0x1234),
            0x1234,
            "ASM UFO.asm loc_35050 stores the chosen velocity and returns"
        );
    }

    #[test]
    fn ufo_collision_dispatch_matches_loc_35056() {
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: None,
            },
            "ASM UFO.asm loc_35056 returns through `rts` when collision_type is zero"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(-1),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: None,
            },
            "ASM UFO.asm loc_35056 sends negatives to loc_35070 and clears collision_type"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0x0004),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: Some(UfoCollisionBehaviorLoc35056::BounceWall),
            },
            "ASM UFO.asm loc_35056 table-selects wall bounce for collision_type=$04"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0x000C),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: Some(UfoCollisionBehaviorLoc35056::BounceFloorCeil),
            },
            "ASM UFO.asm loc_35056 table-selects floor/ceil bounce for collision_type=$0C"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0x0014),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: Some(UfoCollisionBehaviorLoc35056::BounceSlopeUp),
            },
            "ASM UFO.asm loc_35056 table-selects up-slope bounce for collision_type=$14"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0x001C),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: Some(UfoCollisionBehaviorLoc35056::Hurt),
            },
            "ASM UFO.asm loc_35056 routes projectile hits (and kid-jump equivalents) to hurt behavior"
        );
        assert_eq!(
            ufo_collision_dispatch_loc_35056(0x0020),
            UfoCollisionDispatchLoc35056 {
                collision_type: 0,
                behavior: Some(UfoCollisionBehaviorLoc35056::ReturnOnly),
            },
            "ASM UFO.asm loc_35056 maps `$20` to a return-only jump"
        );
    }

    #[test]
    fn ufo_collision_clear_matches_loc_35070() {
        assert_eq!(
            ufo_collision_clear_loc_35070(0x1234),
            UfoCollisionClearLoc35070 { collision_type: 0 },
            "ASM UFO.asm loc_35070 clears the `collision_type(a3)` word"
        );
    }

    #[test]
    fn ufo_position_restore_matches_loc_350f2() {
        assert_eq!(
            ufo_restore_position_loc_350f2(0x1000_0000, 0x2000_0000, 0x0100_0000, -0x0200_0000),
            UfoPositionRestoreLoc350F2 {
                x_pos: 0x0F00_0000,
                y_pos: 0x2200_0000,
            },
            "ASM UFO.asm loc_350F2 rolls the UFO back by one velocity step before slope bounces"
        );
    }

    #[test]
    fn ufo_bounce_wall_tail_matches_loc_35120() {
        assert_eq!(
            ufo_bounce_wall_tail_loc_35120(0x0100_0000, 0x0200_0000),
            UfoWallBounceLoc35120 {
                x_velocity: -0x0100_0000,
                d0: -0x0200_0000,
            },
            "ASM UFO.asm loc_35120 keeps negated `x_vel` and negates positive `d0`"
        );

        assert_eq!(
            ufo_bounce_wall_tail_loc_35120(-0x0100_0000, -0x0200_0000),
            UfoWallBounceLoc35120 {
                x_velocity: 0x0100_0000,
                d0: -0x0200_0000,
            },
            "ASM UFO.asm loc_35120 preserves negative `d0` and negates `x_vel`"
        );
    }

    #[test]
    fn ufo_bounce_floor_ceil_tail_matches_loc_35146() {
        assert_eq!(
            ufo_bounce_floor_ceil_tail_loc_35146(0x0100_0000, 0x0200_0000),
            UfoFloorCeilBounceLoc35146 {
                y_velocity: -0x0100_0000,
                d2: -0x0200_0000,
            },
            "ASM UFO.asm loc_35146 keeps negated `y_vel` and negates positive `d2`"
        );

        assert_eq!(
            ufo_bounce_floor_ceil_tail_loc_35146(-0x0100_0000, -0x0200_0000),
            UfoFloorCeilBounceLoc35146 {
                y_velocity: 0x0100_0000,
                d2: -0x0200_0000,
            },
            "ASM UFO.asm loc_35146 preserves negative `d2` and negates `y_vel`"
        );
    }

    #[test]
    fn ufo_hurt_tail_matches_loc_35174() {
        assert_eq!(
            ufo_hurt_tail_loc_35174(0x00C0_0000, true),
            UfoHurtSpawnTailLoc35174 {
                some_ufo_shooting_cleared: true,
                allocation_selector: 0x6000,
                driver_routine: "loc_355E2",
                driver_x_velocity: 0x00C0_0000,
                driver_timer_flag: -1,
                parent_moved: true,
                parent_y_velocity: 0x0003_0000,
                driver_animation: "stru_34CA0",
                parent_animation_flag_13_cleared: true,
            },
            "ASM UFO.asm loc_35174 clears `Some_UFO_Shooting`, allocates `$6000` driver, and seeds parent motion"
        );

        assert_eq!(
            ufo_hurt_tail_loc_35174(-0x0040_0000, false),
            UfoHurtSpawnTailLoc35174 {
                some_ufo_shooting_cleared: false,
                allocation_selector: 0x6000,
                driver_routine: "loc_355E2",
                driver_x_velocity: -0x0040_0000,
                driver_timer_flag: -1,
                parent_moved: true,
                parent_y_velocity: 0x0003_0000,
                driver_animation: "stru_34CA0",
                parent_animation_flag_13_cleared: true,
            },
            "ASM UFO.asm loc_35174 leaves `Some_UFO_Shooting` untouched when `$46(a5)` is already clear"
        );
    }

    #[test]
    fn ufo_hurt_driver_spawn_matches_loc_35180() {
        assert_eq!(
            ufo_hurt_driver_spawn_loc_35180(0x00C0_0000),
            UfoHurtDriverSpawnLoc35180 {
                allocation_selector: 0x6000,
                driver_routine: "loc_355E2",
                driver_flag_46: -1,
                driver_x_velocity: 0x00C0_0000,
                parent_moved: true,
                parent_y_velocity: 0x0003_0000,
            },
            "ASM UFO.asm loc_35180 allocates `$6000` driver, sets `$46=$FFFF`, copies parent x_vel and sets parent y_vel"
        );

        assert_eq!(
            ufo_hurt_driver_spawn_loc_35180(-0x0080_0000),
            UfoHurtDriverSpawnLoc35180 {
                allocation_selector: 0x6000,
                driver_routine: "loc_355E2",
                driver_flag_46: -1,
                driver_x_velocity: -0x0080_0000,
                parent_moved: true,
                parent_y_velocity: 0x0003_0000,
            },
            "ASM UFO.asm loc_35180 applies the same negative parent x_vel path as the hurt tail"
        );
    }

    #[test]
    fn ufo_hurt_hibernate_gate_matches_loc_351b6() {
        assert_eq!(
            ufo_hurt_hibernate_gate_loc_351b6(false),
            UfoHurtHibernateGateLoc351B6 {
                keep_hibernating: true,
                delete_current_object: false,
            },
            "ASM UFO.asm loc_351B6 keeps hibernating while `$19(a3)` is zero"
        );

        assert_eq!(
            ufo_hurt_hibernate_gate_loc_351b6(true),
            UfoHurtHibernateGateLoc351B6 {
                keep_hibernating: false,
                delete_current_object: true,
            },
            "ASM UFO.asm loc_351B6 deletes current object when `$19(a3)` is set"
        );
    }

    #[test]
    fn ufo_beam_segment_spawn_matches_loc_351f2() {
        assert_eq!(
            ufo_beam_segment_spawn_loc_351f2(-0x0003_0000, -5, 4),
            Some(UfoBeamSpawnLoc351EE {
                allocation_selector: 0x8000,
                beam_routine: "loc_354DA",
                beam_x_velocity: -0x0004_0000,
                beam_y_velocity: 0x0002_0000,
            }),
            "ASM UFO.asm loc_351F2 adds half parent x_vel to the signed table x-word and shifts the y-word independently"
        );

        assert_eq!(
            ufo_beam_segment_spawn_loc_351f2(0x0010_0000, 0, -5),
            None,
            "ASM UFO.asm loc_351F2 branches to the finish tail before reading the y-word when the x-word sentinel is zero"
        );
    }

    #[test]
    fn ufo_beam_spawn_matches_loc_351ee() {
        assert_eq!(
            ufo_beam_spawn_loc_351ee(0x0006_0000, &[(5, 4), (-5, -3), (0, 0), (1, 1)]),
            vec![
                UfoBeamSpawnLoc351EE {
                    allocation_selector: 0x8000,
                    beam_routine: "loc_354DA",
                    beam_x_velocity: 0x0005_8000,
                    beam_y_velocity: 0x0002_0000,
                },
                UfoBeamSpawnLoc351EE {
                    allocation_selector: 0x8000,
                    beam_routine: "loc_354DA",
                    beam_x_velocity: 0x0000_8000,
                    beam_y_velocity: -0x0001_8000,
                },
            ],
            "ASM UFO.asm loc_351EE reads the beam table, spawns each segment, and stops on x-word zero"
        );

        assert_eq!(
            ufo_beam_spawn_loc_351ee(0x0010_0000, &[(0, -5), (5, 5)]),
            Vec::<UfoBeamSpawnLoc351EE>::new(),
            "ASM UFO.asm loc_351EE exits immediately when sentinel x-word is zero"
        );
    }

    #[test]
    fn ufo_beam_child_init_matches_loc_35512() {
        assert_eq!(
            ufo_beam_child_init_loc_35512(0x0200, 0x0100, 0, -0x0004_0000, 0x0002_0000,),
            UfoBeamChildInitLoc35512 {
                x_pos: 0x01F5,
                y_pos: 0x00E8,
                x_velocity: -0x0004_0000,
                y_velocity: 0x0002_0000,
                flag_13: true,
                is_moved: true,
                has_level_collision: false,
                flag_19: false,
                priority: 1,
                object_meta: 0x000F,
                sprite_mapping: "LnkTo_unk_C8408",
                loop_timer: 5,
            },
            "ASM UFO.asm loc_35512 seeds the beam gfx child 24px above the linked object with object meta `$0F` and a five-frame turn timer"
        );

        assert_eq!(
            ufo_beam_child_init_loc_35512(0x0200, 0x0100, 0xFF, 0, 0).x_pos,
            0x020B,
            "ASM UFO.asm loc_354DA/loc_35512 mirrors the child X offset from -$B to +$B when `x_direction(a3)` is non-zero"
        );
    }

    #[test]
    fn ufo_beam_child_turnaround_matches_loc_3554a() {
        assert_eq!(
            ufo_beam_child_turnaround_loc_3554a(5, 0),
            UfoBeamChildTurnaroundLoc3554A {
                loop_timer: 4,
                x_direction: 0,
                flipped_direction: false,
            },
            "ASM UFO.asm loc_3554A branches to the gravity tail while the decremented d0 word is non-zero"
        );

        assert_eq!(
            ufo_beam_child_turnaround_loc_3554a(1, 0),
            UfoBeamChildTurnaroundLoc3554A {
                loop_timer: 0x000A,
                x_direction: 0xFF,
                flipped_direction: true,
            },
            "ASM UFO.asm loc_3554A flips clear `x_direction(a3)` to `$FF` and reloads d0 to `$A` when the countdown reaches zero"
        );

        assert_eq!(
            ufo_beam_child_turnaround_loc_3554a(1, 0xFF).x_direction,
            0,
            "ASM UFO.asm loc_3554A uses `not.b`, so `$FF` flips back to clear direction on the next zero-count frame"
        );
    }

    #[test]
    fn ufo_beam_child_gravity_gate_matches_loc_3555a() {
        assert_eq!(
            ufo_beam_child_gravity_gate_loc_3555a(0x0002_0000, false),
            UfoBeamChildGravityGateLoc3555A {
                y_velocity: 0x0002_3000,
                keep_hibernating: true,
                delete_current_object: false,
            },
            "ASM UFO.asm loc_3555A adds `$3000` to `y_vel(a3)` and loops while `$19(a3)` is clear"
        );

        assert_eq!(
            ufo_beam_child_gravity_gate_loc_3555a(-0x0000_1000, true),
            UfoBeamChildGravityGateLoc3555A {
                y_velocity: 0x0000_2000,
                keep_hibernating: false,
                delete_current_object: true,
            },
            "ASM UFO.asm loc_3555A still applies gravity before deleting when `$19(a3)` is set"
        );
    }

    #[test]
    fn ufo_hurt_driver_fall_loop_matches_loc_3564c() {
        assert_eq!(
            ufo_hurt_driver_fall_loop_loc_3564c(0x0002_0000),
            UfoHurtDriverFallLoopLoc3564C {
                hibernates_one_frame: true,
                range_check_mode: 1,
                y_velocity: 0x0002_2000,
                collision_table: "off_3569E",
                collision_dispatch_mode: 1,
                loops_forever: true,
            },
            "ASM UFO.asm loc_3564C hibernates, checks range with d0=1, adds `$2000` gravity, and loops through `off_3569E`"
        );

        assert_eq!(
            ufo_hurt_driver_fall_loop_loc_3564c(i32::MAX).y_velocity,
            i32::MIN.wrapping_add(0x0000_1FFF),
            "ASM UFO.asm loc_3564C uses 68k longword wrapping for `addi.l #$2000,y_vel(a3)`"
        );
    }

    #[test]
    fn ufo_hurt_driver_slide_seed_matches_loc_3566c() {
        assert_eq!(
            ufo_hurt_driver_slide_seed_loc_3566c(0),
            UfoHurtDriverSlideSeedLoc3566C {
                x_velocity: -0x0000_8000,
                falls_through_to_slide_loop: true,
            },
            "ASM UFO.asm loc_3566C keeps `$FFFF8000` when raw `x_direction(a3)` is clear"
        );

        assert_eq!(
            ufo_hurt_driver_slide_seed_loc_3566c(0xFF),
            UfoHurtDriverSlideSeedLoc3566C {
                x_velocity: 0x0000_8000,
                falls_through_to_slide_loop: true,
            },
            "ASM UFO.asm loc_3566C negates `x_vel(a3)` for any non-zero raw direction before the slide loop"
        );
    }

    #[test]
    fn ufo_hurt_driver_slide_loop_matches_loc_35680() {
        assert_eq!(
            ufo_hurt_driver_slide_loop_loc_35680(),
            UfoHurtDriverSlideLoopLoc35680 {
                has_kid_collision: false,
                hibernates_one_frame: true,
                checks_in_range: true,
                collision_table: "off_356B6",
                collision_dispatch_mode: 0,
                checks_ground_tile: true,
                loops_forever: true,
            },
            "ASM UFO.asm loc_35680 clears Kid collision, uses d0=0 collision dispatch, checks the ground tile, and loops"
        );
    }

    #[test]
    fn ufo_hurt_driver_slide_turnaround_matches_loc_356ce() {
        assert_eq!(
            ufo_hurt_driver_slide_turnaround_loc_356ce(-0x0000_8000, 0x0010_0000, 0),
            UfoHurtDriverSlideTurnaroundLoc356CE {
                x_velocity: 0x0000_8000,
                x_pos: 0x0012_0000,
                x_direction: 0xFF,
            },
            "ASM UFO.asm loc_356CE negates x_vel, applies four frames of the reversed velocity to x_pos, and flips clear x_direction to `$FF`"
        );

        assert_eq!(
            ufo_hurt_driver_slide_turnaround_loc_356ce(0x0000_8000, 0x0010_0000, 0xFF),
            UfoHurtDriverSlideTurnaroundLoc356CE {
                x_velocity: -0x0000_8000,
                x_pos: 0x000E_0000,
                x_direction: 0,
            },
            "ASM UFO.asm loc_356CE uses longword wrapping for the shifted negative displacement and `not.b` for direction"
        );
    }

    #[test]
    fn ufo_hurt_driver_slide_ground_miss_matches_loc_356f2() {
        assert_eq!(
            ufo_hurt_driver_slide_ground_miss_loc_356f2(0x2F, 0, -0x0000_8000, 0x0010_0000, 0, 0,),
            UfoHurtDriverSlideGroundMissLoc356F2::Turnaround(
                UfoHurtDriverSlideTurnaroundLoc356CE {
                    x_velocity: 0x0000_8000,
                    x_pos: 0x0012_0000,
                    x_direction: 0xFF,
                },
            ),
            "ASM UFO.asm loc_356F2 branches to loc_356CE when the masked first RNG byte is greater than `$0A`"
        );

        let y_velocity = i32::from_ne_bytes(0xFFFD_1234_u32.to_ne_bytes());
        assert_eq!(
            ufo_hurt_driver_slide_ground_miss_loc_356f2(
                0x0A,
                0x04,
                0x0000_8000,
                0x0010_0000,
                0x0000_1234,
                0xFF,
            ),
            UfoHurtDriverSlideGroundMissLoc356F2::RelaunchFall {
                x_velocity: 0x0001_0000,
                y_velocity,
                next_label: "loc_3564C",
            },
            "ASM UFO.asm loc_356F2 writes the selected word_356E4 high word to y_vel, preserves its low word, doubles x_vel, and branches to loc_3564C"
        );
    }

    #[test]
    fn ufo_hurt_driver_slide_collision_return_matches_loc_3571e() {
        assert_eq!(
            ufo_hurt_driver_slide_collision_return_loc_3571e(),
            UfoHurtDriverSlideCollisionReturnLoc3571E {
                stack_bytes_discarded: 4,
                next_label: "loc_35680",
                slide_loop: UfoHurtDriverSlideLoopLoc35680 {
                    has_kid_collision: false,
                    hibernates_one_frame: true,
                    checks_in_range: true,
                    collision_table: "off_356B6",
                    collision_dispatch_mode: 0,
                    checks_ground_tile: true,
                    loops_forever: true,
                },
            },
            "ASM UFO.asm loc_3571E adjusts the stack by four bytes and branches back to the loc_35680 slide loop"
        );
    }

    #[test]
    fn ufo_hurt_driver_slide_probe_x_matches_loc_35724() {
        assert_eq!(
            ufo_hurt_driver_slide_probe_x_loc_35724(0x0123_8000, 0),
            UfoHurtDriverSlideProbeXLoc35724 {
                probe_x_word: 0x0114,
            },
            "ASM UFO.asm loc_35724 probes 15px left when raw x_direction is clear"
        );

        assert_eq!(
            ufo_hurt_driver_slide_probe_x_loc_35724(0x0123_8000, 0xFF),
            UfoHurtDriverSlideProbeXLoc35724 {
                probe_x_word: 0x0132,
            },
            "ASM UFO.asm loc_35724 adds `$1E` after the left offset for any non-zero raw x_direction"
        );

        assert_eq!(
            ufo_hurt_driver_slide_probe_x_loc_35724(0x0004_0000, 0).probe_x_word,
            0xFFF5,
            "ASM UFO.asm loc_35724 uses 16-bit word wrapping for the `$0F` subtraction"
        );
    }

    #[test]
    fn ufo_hurt_driver_slide_ground_tile_matches_loc_35738() {
        assert_eq!(
            ufo_hurt_driver_slide_ground_tile_loc_35738(0x0048_0000, 0x0123, 0x6000),
            UfoHurtDriverSlideGroundTileLoc35738 {
                probe_y_word: 0x0050,
                tile_column: 0x0012,
                tile_row: 0x0005,
                tile_collision_nibble: 0x6000,
                ground_present: true,
                miss_label: None,
            },
            "ASM UFO.asm loc_35738 returns when the probed tile collision nibble is `$6000`"
        );

        assert_eq!(
            ufo_hurt_driver_slide_ground_tile_loc_35738(0x0048_0000, 0x0123, 0x5000),
            UfoHurtDriverSlideGroundTileLoc35738 {
                probe_y_word: 0x0050,
                tile_column: 0x0012,
                tile_row: 0x0005,
                tile_collision_nibble: 0x5000,
                ground_present: false,
                miss_label: Some("loc_356F2"),
            },
            "ASM UFO.asm loc_35738 branches to loc_356F2 for any non-`$6000` collision nibble"
        );

        assert_eq!(
            ufo_hurt_driver_slide_ground_tile_loc_35738(0xFFFC_0000, 0xFFF0, 0x0000).tile_row,
            0x0000,
            "ASM UFO.asm loc_35738 uses wrapping word addition before `lsr.w #4`"
        );
    }

    #[test]
    fn ufo_hurt_driver_collision_dispatch_matches_loc_35762() {
        assert_eq!(
            ufo_hurt_driver_collision_dispatch_loc_35762(
                0,
                UfoHurtDriverCollisionTableLoc35762::SlideLoop,
                false,
                0,
            ),
            UfoHurtDriverCollisionDispatchLoc35762 {
                clears_collision_type: false,
                behavior: None,
            },
            "ASM UFO.asm loc_35762 returns immediately when collision_type is zero"
        );

        assert_eq!(
            ufo_hurt_driver_collision_dispatch_loc_35762(
                -1,
                UfoHurtDriverCollisionTableLoc35762::SlideLoop,
                false,
                0,
            ),
            UfoHurtDriverCollisionDispatchLoc35762 {
                clears_collision_type: false,
                behavior: Some(UfoHurtDriverCollisionBehaviorLoc35762::StopAndDeleteAfterAnimation,),
            },
            "ASM UFO.asm loc_35762 routes negative collision_type to the slide hurt tail without clearing it first"
        );

        assert_eq!(
            ufo_hurt_driver_collision_dispatch_loc_35762(
                0x0004,
                UfoHurtDriverCollisionTableLoc35762::SlideLoop,
                false,
                0,
            ),
            UfoHurtDriverCollisionDispatchLoc35762 {
                clears_collision_type: true,
                behavior: Some(UfoHurtDriverCollisionBehaviorLoc35762::SlideTurnaround),
            },
            "ASM UFO.asm loc_35762 clears positive collision_type, subtracts 4, and table-selects slide turnaround"
        );

        assert_eq!(
            ufo_hurt_driver_collision_dispatch_loc_35762(
                0x000C,
                UfoHurtDriverCollisionTableLoc35762::FallLoop,
                false,
                0,
            ),
            UfoHurtDriverCollisionDispatchLoc35762 {
                clears_collision_type: true,
                behavior: Some(UfoHurtDriverCollisionBehaviorLoc35762::FallFloorLanding),
            },
            "ASM UFO.asm loc_35762 uses the caller's fall-loop table for the same collision_type index"
        );

        assert_eq!(
            ufo_hurt_driver_collision_dispatch_loc_35762(
                0x001C,
                UfoHurtDriverCollisionTableLoc35762::FallLoop,
                false,
                0,
            ),
            UfoHurtDriverCollisionDispatchLoc35762 {
                clears_collision_type: true,
                behavior: Some(UfoHurtDriverCollisionBehaviorLoc35762::StopAndFallUntilDeleted,),
            },
            "ASM UFO.asm loc_35762 treats `$1C` as a hurt collision before table dispatch"
        );

        assert_eq!(
            ufo_hurt_driver_collision_dispatch_loc_35762(
                0x0020,
                UfoHurtDriverCollisionTableLoc35762::SlideLoop,
                false,
                0x0004_0000,
            ),
            UfoHurtDriverCollisionDispatchLoc35762 {
                clears_collision_type: true,
                behavior: Some(UfoHurtDriverCollisionBehaviorLoc35762::ReturnOnly),
            },
            "ASM UFO.asm loc_35762 returns after clearing `$20` when Berzerker is not charging"
        );

        assert_eq!(
            ufo_hurt_driver_collision_dispatch_loc_35762(
                0x0020,
                UfoHurtDriverCollisionTableLoc35762::SlideLoop,
                true,
                -0x0004_0000,
            ),
            UfoHurtDriverCollisionDispatchLoc35762 {
                clears_collision_type: true,
                behavior: Some(
                    UfoHurtDriverCollisionBehaviorLoc35762::BerzerkerChargeFall {
                        x_velocity: -0x0002_0000,
                        y_velocity: -0x0004_0000,
                    }
                ),
            },
            "ASM UFO.asm loc_35762 uses half Kid x velocity and `-$40000` y velocity for a charging Berzerker hit"
        );
    }

    #[test]
    fn ufo_beam_finish_matches_loc_35228() {
        assert_eq!(
            ufo_beam_finish_loc_35228(0x0001_0000),
            UfoBeamFinishLoc35228 {
                parent_beam_spawned_latch: true,
                linked_object_meta: 0x000F,
                linked_object_animation: "stru_34D02",
                linked_object_flag_13: true,
                linked_object_flag_19: false,
                linked_object_is_moved: false,
                linked_object_level_collision: false,
                linked_object_routine: 1,
                parent_animation: "stru_34C92",
                parent_y_velocity: 0x0005_0000,
                d2: -0x0000_0C00,
                d3: 0,
            },
            "ASM UFO.asm loc_35228 links the child gfx object, resets its state flags, bumps parent y_vel by $40000, and returns d2=$FFFFF400/d3=0"
        );
    }

    #[test]
    fn ufo_linked_object_range_gate_matches_loc_35280() {
        assert_eq!(
            ufo_linked_object_range_gate_loc_35280(false),
            UfoLinkedObjectRangeGateLoc35280::Return,
            "ASM UFO.asm loc_35280 returns when the range check leaves the zero condition set"
        );

        assert_eq!(
            ufo_linked_object_range_gate_loc_35280(true),
            UfoLinkedObjectRangeGateLoc35280::RunCleanupDeleteTail,
            "ASM UFO.asm loc_35280 enters the cleanup/delete tail when the range check reports a non-zero result"
        );
    }

    #[test]
    fn ufo_linked_cleanup_count_matches_loc_3528a() {
        assert_eq!(
            ufo_linked_cleanup_count_loc_3528a(3, false),
            UfoLinkedCleanupCountLoc3528A {
                number_ufos_on_screen: 3,
                play_hover_sound: false,
            },
            "ASM UFO.asm loc_3528A skips Number_UFOs_OnScreen while `$50(a5)` is clear"
        );

        assert_eq!(
            ufo_linked_cleanup_count_loc_3528a(2, true),
            UfoLinkedCleanupCountLoc3528A {
                number_ufos_on_screen: 1,
                play_hover_sound: false,
            },
            "ASM UFO.asm loc_3528A decrements the UFO count and falls through silently when another UFO remains"
        );

        assert_eq!(
            ufo_linked_cleanup_count_loc_3528a(1, true),
            UfoLinkedCleanupCountLoc3528A {
                number_ufos_on_screen: 0,
                play_hover_sound: true,
            },
            "ASM UFO.asm loc_3528A plays sfx_UFO_hovering through j_PlaySound2 when the decrement reaches zero"
        );
    }

    #[test]
    fn ufo_linked_cleanup_shooting_matches_loc_352a6() {
        assert_eq!(
            ufo_linked_cleanup_shooting_loc_352a6(false, true),
            UfoLinkedCleanupShootingLoc352A6 {
                global_shooting_flag: true,
            },
            "ASM UFO.asm loc_352A6 leaves Some_UFO_Shooting untouched while `$46(a5)` is clear"
        );

        assert_eq!(
            ufo_linked_cleanup_shooting_loc_352a6(true, true),
            UfoLinkedCleanupShootingLoc352A6 {
                global_shooting_flag: false,
            },
            "ASM UFO.asm loc_352A6 clears Some_UFO_Shooting when `$46(a5)` is set"
        );
    }

    #[test]
    fn ufo_linked_cleanup_status_matches_loc_352b2() {
        assert_eq!(
            ufo_linked_cleanup_status_loc_352b2(0x05),
            DeactivationStatusActionLoc3C4C0::WriteRespawnMarker {
                index: 5,
                word_value: 0x2168
            },
            "ASM UFO.asm loc_352D2 writes `$2168` to the positive `$42(a5)` status slot"
        );

        assert_eq!(
            ufo_linked_cleanup_status_loc_352b2(0xC3),
            DeactivationStatusActionLoc3C4C0::SubtractSpawnCounter { index: 3 },
            "ASM UFO.asm loc_352B2 masks bit-7/bit-6 sentinel slots to six bits before subtracting `$400`"
        );

        assert_eq!(
            ufo_linked_cleanup_status_loc_352b2(0x82),
            DeactivationStatusActionLoc3C4C0::DeleteOnly,
            "ASM UFO.asm loc_352E0 deletes without an EnemyStatus_Table write when bit 7 is set and bit 6 is clear"
        );
    }

    #[test]
    fn ufo_linked_object_x_matches_loc_3531a() {
        assert_eq!(
            ufo_linked_object_x_loc_3531a(0x0120_8000, Direction::Left),
            0x011C,
            "ASM UFO.asm loc_3531A subtracts d6=4 when x_direction is clear"
        );

        assert_eq!(
            ufo_linked_object_x_loc_3531a(0x0120_8000, Direction::Right),
            0x0124,
            "ASM UFO.asm loc_3531A subtracts d6=-4 when x_direction is set"
        );
    }

    #[test]
    fn ufo_apply_acceleration_axis_matches_loc_3502a() {
        assert_eq!(
            ufo_apply_acceleration_axis_loc_3502a(0x0800, 0x1000, 0x0100),
            0x0900,
            "ASM UFO.asm loc_3502A keeps accelerating on the positive branch while below target"
        );
        assert_eq!(
            ufo_apply_acceleration_axis_loc_3502a(0x0800, 0x0400, 0x0100),
            0x0400,
            "ASM UFO.asm loc_3502A caps velocity to target when the next value would exceed it"
        );
    }

    #[test]
    fn ufo_beam_clear_state_matches_loc_34ffe() {
        assert_eq!(
            ufo_beam_clear_shooting_state_loc_34ffe(),
            UfoBeamClearLoc34FFE {
                ufo_shooting_latch: false,
                global_shooting_flag: false,
            },
            "ASM UFO.asm loc_34FFE clears `$46(a5)` and `Some_UFO_Shooting`"
        );
    }

    #[test]
    fn ufo_on_screen_count_matches_loc_34f64() {
        assert_eq!(
            ufo_on_screen_count_loc_34f64(0),
            UfoOnScreenCountLoc34F64 {
                number_ufos_on_screen: 1,
                ufo_on_screen_latch: true,
            },
            "ASM UFO.asm loc_34F64 increments Number_UFOs_OnScreen and sets `$50(a5)` for a newly visible UFO"
        );

        assert_eq!(
            ufo_on_screen_count_loc_34f64(0xFFFF),
            UfoOnScreenCountLoc34F64 {
                number_ufos_on_screen: 0,
                ufo_on_screen_latch: true,
            },
            "ASM UFO.asm loc_34F64 uses 16-bit addq.w wrapping semantics for the global UFO count"
        );
    }

    #[test]
    fn ufo_off_screen_gate_matches_loc_34f6e() {
        assert_eq!(
            ufo_off_screen_gate_loc_34f6e(3, false),
            UfoOffScreenGateLoc34F6E {
                number_ufos_on_screen: 3,
                play_hover_stop_sound: false,
                clear_latch_tail: false,
            },
            "ASM UFO.asm loc_34F6E returns immediately when `$50(a5)` is clear"
        );

        assert_eq!(
            ufo_off_screen_gate_loc_34f6e(2, true),
            UfoOffScreenGateLoc34F6E {
                number_ufos_on_screen: 1,
                play_hover_stop_sound: false,
                clear_latch_tail: true,
            },
            "ASM UFO.asm loc_34F6E decrements Number_UFOs_OnScreen and skips the stop sound while another UFO remains"
        );

        assert_eq!(
            ufo_off_screen_gate_loc_34f6e(1, true),
            UfoOffScreenGateLoc34F6E {
                number_ufos_on_screen: 0,
                play_hover_stop_sound: true,
                clear_latch_tail: true,
            },
            "ASM UFO.asm loc_34F6E plays the hover-stop sound only when the decrement reaches zero"
        );
    }

    #[test]
    fn ufo_clear_on_screen_latch_matches_loc_34f88() {
        assert!(
            !ufo_clear_on_screen_latch_loc_34f88(),
            "ASM UFO.asm loc_34F88 clears `$50(a5)` with `sf` after the off-screen count/sound gate"
        );
    }

    #[test]
    fn ufo_tracking_matches_loc_34e2a_early_returns() {
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x0100_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                false,
                false,
                false,
                false,
            ),
            None,
            "ASM UFO.asm loc_34E2A returns before checking positions when `$4D(a5)` is zero"
        );
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x0100_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                true,
                true,
                false,
                false,
            ),
            None,
            "ASM UFO.asm loc_34E2A returns while `Some_UFO_Shooting` is non-zero"
        );
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x0100_0000,
                0x0080_0000,
                0x0100_0000,
                0x0080_0000,
                true,
                false,
                false,
                false,
            ),
            None,
            "ASM UFO.asm loc_34E2A requires Kid's integer Y word to be greater than UFO's"
        );
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x00CF_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                true,
                false,
                false,
                false,
            ),
            None,
            "ASM UFO.asm loc_34E2A rejects UFO X below Kid X - $30"
        );
        assert_eq!(
            ufo_tracking_command_loc_34e2a(
                0x0131_0000,
                0x0080_0000,
                0x0100_0000,
                0x0090_0000,
                true,
                false,
                false,
                false,
            ),
            None,
            "ASM UFO.asm loc_34E2A rejects UFO X above Kid X + $30"
        );
    }
}
