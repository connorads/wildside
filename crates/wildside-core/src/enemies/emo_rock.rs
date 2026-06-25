use super::*;

/// EmoRock hover height above player, indexed by enemy_level (derived from HP).
/// Level 0: 80px, Level 1: 60px, Level 2: 40px.
pub(crate) const EMO_ROCK_HOVER_HEIGHT: [i32; 3] = [0x50, 0x3C, 0x28];

/// EmoRock gravity acceleration (applied each frame while dropping).
/// Level 0: $4000, Level 1: $6000, Level 2: $8000.
pub(crate) const EMO_ROCK_GRAVITY: [i32; 3] = [0x4000, 0x6000, 0x8000];

/// EmoRock terminal velocity threshold/cap while dropping.
/// ASM Emo_Rock.asm:72-74: if y_vel > $60000, reset to $40000.
pub(crate) const EMO_ROCK_VELOCITY_THRESHOLD: i32 = 0x6_0000;
pub(crate) const EMO_ROCK_VELOCITY_CAP: i32 = 0x4_0000;

/// Frames in the first `stru_363FE` landing animation script before `$18(a3)`
/// sends Emo Rock to `loc_36320`.
pub(crate) const EMO_ROCK_SETTLED_ANIMATION_FRAMES_LOC_362A4: u16 = 0x0088;

/// Frames in `stru_36456`, the disappear animation hibernated by `loc_36320`.
pub(crate) const EMO_ROCK_DISAPPEAR_ANIMATION_FRAMES_LOC_36320: u8 = 0x10;

pub(crate) const EMO_ROCK_BERZERKER_AIRBORNE_WAIT_TIMER_LOC_3630A: u16 = 0x3630;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmoRockWaitCountdownLoc36212 {
    pub(crate) remaining_wait_timer: u16,
    pub(crate) repeats_wait_loop: bool,
}

pub(crate) fn emo_rock_wait_countdown_loc_36212(wait_timer: u16) -> EmoRockWaitCountdownLoc36212 {
    // ASM Emo_Rock.asm loc_36212: after each one-frame hibernate, `subq.w #1,d0`
    // leaves the object in the wait loop while the word result is non-zero.
    let remaining_wait_timer = wait_timer.wrapping_sub(1);
    EmoRockWaitCountdownLoc36212 {
        remaining_wait_timer,
        repeats_wait_loop: remaining_wait_timer != 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmoRockHoverTargetLoc3621A {
    pub(crate) x_pos: u32,
    pub(crate) y_pos: u32,
    pub(crate) clearance_height_px: u16,
    pub(crate) clearance_half_width_px: u16,
}

pub(crate) fn emo_rock_hover_target_loc_3621a(
    player_x_pos: u32,
    player_y_pos: u32,
    hover_height_px: i32,
) -> EmoRockHoverTargetLoc3621A {
    // ASM Emo_Rock.asm loc_3621A: after hibernating, copy Kid X into x_pos,
    // subtract d1 from Kid Y for y_pos, then probe a `$1C` by `$E` clearance box.
    let hover_height_fp = (hover_height_px as u32) << 16;
    EmoRockHoverTargetLoc3621A {
        x_pos: player_x_pos,
        y_pos: player_y_pos.saturating_sub(hover_height_fp),
        clearance_height_px: 0x001C,
        clearance_half_width_px: 0x000E,
    }
}

pub(crate) fn emo_rock_drop_velocity_loc_3625e(y_velocity: i32, gravity: i32) -> i32 {
    // ASM Emo_Rock.asm loc_3625E: add d2 gravity to y_vel, keep the result when
    // it is <= `$60000`, otherwise snap the falling speed back to `$40000`.
    let accelerated_y_velocity = y_velocity + gravity;
    if accelerated_y_velocity > EMO_ROCK_VELOCITY_THRESHOLD {
        EMO_ROCK_VELOCITY_CAP
    } else {
        accelerated_y_velocity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum EmoRockDropRouteLoc36274 {
    ContinueDrop,
    Death,
    Land { facing: Direction },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct EmoRockDropFrameLoc36274 {
    pub(crate) y_velocity: i32,
    pub(crate) next_collision_type: u16,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) route: EmoRockDropRouteLoc36274,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn emo_rock_drop_frame_loc_36274(
    y_velocity: i32,
    collision_type: i16,
    enemy_x_pos: u32,
    player_x_pos: u32,
) -> EmoRockDropFrameLoc36274 {
    // ASM Emo_Rock.asm loc_36274: store d7 into y_vel, hibernate, then read
    // collision_type. Negative words go to the destroy path before the clear;
    // non-negative terrain contacts below `$1C` land and face Kid.
    let (next_collision_type, route) = if collision_type == 0 {
        (0, EmoRockDropRouteLoc36274::ContinueDrop)
    } else if collision_type.is_negative() {
        (collision_type as u16, EmoRockDropRouteLoc36274::Death)
    } else if collision_type >= 0x001C {
        (0, EmoRockDropRouteLoc36274::ContinueDrop)
    } else {
        let enemy_x_px = (enemy_x_pos >> 16) as i16;
        let player_x_px = (player_x_pos >> 16) as i16;
        let facing = if player_x_px < enemy_x_px {
            Direction::Left
        } else {
            Direction::Right
        };
        (0, EmoRockDropRouteLoc36274::Land { facing })
    };

    EmoRockDropFrameLoc36274 {
        y_velocity,
        next_collision_type,
        hibernates_one_frame: true,
        route,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct EmoRockLandingSetupLoc362A4 {
    pub(crate) y_velocity: i32,
    pub(crate) has_level_collision: bool,
    pub(crate) animation_script: &'static str,
    pub(crate) next_loop: &'static str,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn emo_rock_landing_setup_loc_362a4() -> EmoRockLandingSetupLoc362A4 {
    // ASM Emo_Rock.asm loc_362A4: landing clears vertical motion, disables level
    // collision while the settling animation plays, and then enters the settled wait loop.
    EmoRockLandingSetupLoc362A4 {
        y_velocity: 0,
        has_level_collision: false,
        animation_script: "stru_363FE",
        next_loop: "settled_wait_loop",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmoRockSettledWaitRouteLoc362B6 {
    ContinueWait,
    Disappear,
    Death,
    BerzerkerSideHit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmoRockSettledWaitFrameLoc362B6 {
    pub(crate) next_collision_type: u16,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) route: EmoRockSettledWaitRouteLoc362B6,
}

pub(crate) fn emo_rock_settled_wait_loc_362b6(
    animation_finished: bool,
    collision_type: i16,
    berzerker_charging: bool,
) -> EmoRockSettledWaitFrameLoc362B6 {
    // ASM Emo_Rock.asm loc_362B6: hibernate one frame, exit immediately when
    // `$18(a3)` says the landing animation finished, otherwise dispatch the
    // current collision word. Positive collision words are cleared before routing.
    if animation_finished {
        return EmoRockSettledWaitFrameLoc362B6 {
            next_collision_type: collision_type as u16,
            hibernates_one_frame: true,
            route: EmoRockSettledWaitRouteLoc362B6::Disappear,
        };
    }

    let (next_collision_type, route) = if collision_type == 0 {
        (0, EmoRockSettledWaitRouteLoc362B6::ContinueWait)
    } else if collision_type.is_negative() {
        (
            collision_type as u16,
            EmoRockSettledWaitRouteLoc362B6::Death,
        )
    } else {
        let route = match collision_type {
            0x001C | 0x002C => EmoRockSettledWaitRouteLoc362B6::Death,
            0x0020 | 0x0024 if berzerker_charging => {
                EmoRockSettledWaitRouteLoc362B6::BerzerkerSideHit
            }
            _ => EmoRockSettledWaitRouteLoc362B6::ContinueWait,
        };
        (0, route)
    };

    EmoRockSettledWaitFrameLoc362B6 {
        next_collision_type,
        hibernates_one_frame: true,
        route,
    }
}

const EMO_ROCK_BERZERKER_LAUNCH_Y_VELOCITY: i32 = -0x0003_0000;
const EMO_ROCK_BERZERKER_AIRBORNE_GRAVITY: i32 = 0x0000_6000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmoRockBerzerkerLaunchLoc362Ee {
    pub(crate) has_kid_collision: bool,
    pub(crate) has_level_collision: bool,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
    pub(crate) next_loop: &'static str,
}

pub(crate) fn emo_rock_berzerker_launch_loc_362ee(
    player_x_velocity: i32,
) -> EmoRockBerzerkerLaunchLoc362Ee {
    // ASM Emo_Rock.asm loc_362EE: Berzerker side hits re-enable Kid collision,
    // disable level collision, copy Kid's x_vel from a2, seed y_vel=$FFFD0000,
    // then fall through to the airborne loop.
    EmoRockBerzerkerLaunchLoc362Ee {
        has_kid_collision: true,
        has_level_collision: false,
        x_velocity: player_x_velocity,
        y_velocity: EMO_ROCK_BERZERKER_LAUNCH_Y_VELOCITY,
        next_loop: "berzerker_airborne_loop",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmoRockBerzerkerAirborneRouteLoc3630A {
    ContinueAirborne,
    Death,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmoRockBerzerkerAirborneFrameLoc3630A {
    pub(crate) y_velocity: i32,
    pub(crate) hibernates_one_frame: bool,
    pub(crate) route: EmoRockBerzerkerAirborneRouteLoc3630A,
}

pub(crate) fn emo_rock_berzerker_airborne_frame_loc_3630a(
    y_velocity: i32,
    flag_19_set: bool,
) -> EmoRockBerzerkerAirborneFrameLoc3630A {
    // ASM Emo_Rock.asm loc_3630A: hibernate once, branch to death when byte
    // `$19(a3)` is set, otherwise add `$6000` to y_vel and loop.
    if flag_19_set {
        return EmoRockBerzerkerAirborneFrameLoc3630A {
            y_velocity,
            hibernates_one_frame: true,
            route: EmoRockBerzerkerAirborneRouteLoc3630A::Death,
        };
    }

    EmoRockBerzerkerAirborneFrameLoc3630A {
        y_velocity: y_velocity + EMO_ROCK_BERZERKER_AIRBORNE_GRAVITY,
        hibernates_one_frame: true,
        route: EmoRockBerzerkerAirborneRouteLoc3630A::ContinueAirborne,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmoRockDisappearLoc36320 {
    pub(crate) has_kid_collision: bool,
    pub(crate) sound_effect: &'static str,
    pub(crate) animation_script: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) addroffset_sprite: u16,
    pub(crate) next_loop: &'static str,
}

pub(crate) fn emo_rock_disappear_loc_36320() -> EmoRockDisappearLoc36320 {
    // ASM Emo_Rock.asm loc_36320: set Kid collision, play the disappear sound,
    // run `stru_36456` to completion, clear addroffset_sprite, then restart
    // the hover cycle.
    EmoRockDisappearLoc36320 {
        has_kid_collision: true,
        sound_effect: "sfx_Emo_Rock_disappears",
        animation_script: "stru_36456",
        waits_until_animation_finished: true,
        addroffset_sprite: 0,
        next_loop: "restart_hover_cycle",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct EmoRockDeathStartLoc36346 {
    pub(crate) has_kid_collision: bool,
    pub(crate) is_moved: bool,
    pub(crate) animation_script: &'static str,
    pub(crate) hibernates_before_sound: bool,
    pub(crate) sound_effect: &'static str,
    pub(crate) waits_until_animation_finished: bool,
    pub(crate) reads_object_status_slot: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn emo_rock_death_start_loc_36346() -> EmoRockDeathStartLoc36346 {
    // ASM Emo_Rock.asm loc_36346: re-enable Kid collision, stop movement, run
    // the break animation/sound sequence, then read `$42(a5)` for status routing.
    EmoRockDeathStartLoc36346 {
        has_kid_collision: true,
        is_moved: false,
        animation_script: "stru_36468",
        hibernates_before_sound: true,
        sound_effect: "sfx_Prize_block",
        waits_until_animation_finished: true,
        reads_object_status_slot: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmoRockRespawnMarkerLoc3638C {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

pub(crate) fn emo_rock_respawn_marker_loc_3638c(
    object_status_slot: u8,
) -> EmoRockRespawnMarkerLoc3638C {
    // ASM Emo_Rock.asm loc_3638C: positive `$42(a5)` is doubled as an
    // EnemyStatus_Table word offset, then `$2168` is stored.
    EmoRockRespawnMarkerLoc3638C {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmoRockDeathStatusRouteLoc3639A {
    WriteRespawnMarker(EmoRockRespawnMarkerLoc3638C),
    SubtractSpawnCounter { index: usize, decrement: u16 },
    DeleteOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmoRockDeathStatusLoc3639A {
    pub(crate) status_route: EmoRockDeathStatusRouteLoc3639A,
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
pub(crate) fn emo_rock_death_status_loc_3639A(
    object_status_slot: u8,
) -> EmoRockDeathStatusLoc3639A {
    // ASM Emo_Rock.asm loc_3639A: every `$42(a5)` status route converges on
    // the `j_Delete_CurrentObject` tail after optional EnemyStatus_Table work.
    let status_route = if object_status_slot & 0x80 == 0 {
        EmoRockDeathStatusRouteLoc3639A::WriteRespawnMarker(emo_rock_respawn_marker_loc_3638c(
            object_status_slot,
        ))
    } else if object_status_slot & 0x40 != 0 {
        EmoRockDeathStatusRouteLoc3639A::SubtractSpawnCounter {
            index: usize::from(object_status_slot & 0x3F),
            decrement: 0x0400,
        }
    } else {
        EmoRockDeathStatusRouteLoc3639A::DeleteOnly
    };

    EmoRockDeathStatusLoc3639A {
        status_route,
        deletes_current_object: true,
    }
}

/// Step an EmoRock enemy through its state machine:
/// - wait_timer > 0: hover above player, count down
/// - wait_timer == 0 && !on_ground: drop with gravity
/// - on_ground: play the landing animation, disappear, then restart hover wait
#[cfg(test)]
pub(crate) fn step_emo_rock(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    let mut rng = RomRandomState::default();
    step_emo_rock_with_rng(
        enemy,
        block_grid,
        player_x_pos,
        player_y_pos,
        &mut rng,
        None,
    );
}

pub(crate) fn step_emo_rock_with_rng(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
    rng: &mut RomRandomState,
    played_sound_effects: Option<&mut Vec<QueuedSoundEffect>>,
) {
    if enemy.wait_timer == EMO_ROCK_BERZERKER_AIRBORNE_WAIT_TIMER_LOC_3630A {
        step_emo_rock_berzerker_airborne(enemy);
        return;
    }

    if enemy.on_ground {
        // ASM Emo_Rock.asm loc_362A4/loc_362B6: after landing, the rock clears
        // vertical motion and waits in place for the landing animation instead
        // of entering a pacer-style ground walk.
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        if enemy.grounded_frames > 0 {
            enemy.grounded_frames -= 1;
            if enemy.grounded_frames != 0 {
                return;
            }

            // ASM Emo_Rock.asm loc_36320: after `stru_36456` finishes, clear
            // addroffset_sprite and branch back to loc_361F4's hover delay.
            enemy.on_ground = false;
            enemy.wait_timer =
                emo_rock_wait_timer_with_rng(rng, player_x_pos.unwrap_or(enemy.x_pos));
            return;
        }
        if enemy.wait_timer > 0 {
            enemy.wait_timer =
                emo_rock_wait_countdown_loc_36212(enemy.wait_timer).remaining_wait_timer;
            if enemy.wait_timer != 0 {
                return;
            }
        }

        let settled = emo_rock_settled_wait_loc_362b6(true, 0, false);
        if matches!(settled.route, EmoRockSettledWaitRouteLoc362B6::Disappear) {
            let disappear = emo_rock_disappear_loc_36320();
            if let Some(sound_effects) = played_sound_effects {
                // ASM Emo_Rock.asm loc_36320: play `sfx_Emo_Rock_disappears`
                // before `stru_36456` hibernates and branches to loc_361F4.
                sound_effects.push(QueuedSoundEffect::EmoRockDisappears);
            }
            enemy.x_velocity = 0;
            enemy.y_velocity = 0;
            if disappear.has_kid_collision {
                enemy.grounded_frames = EMO_ROCK_DISAPPEAR_ANIMATION_FRAMES_LOC_36320;
            }
        }
        return;
    }

    if enemy.wait_timer > 0 {
        // Hovering phase: track player position, count down.
        let level_index = enemy.hit_points.saturating_sub(1).min(2) as usize;
        if let (Some(px), Some(py)) = (player_x_pos, player_y_pos) {
            let target =
                emo_rock_hover_target_loc_3621a(px, py, EMO_ROCK_HOVER_HEIGHT[level_index]);
            enemy.x_pos = target.x_pos;
            enemy.y_pos = target.y_pos;
        } else {
            let hover_height_fp = (EMO_ROCK_HOVER_HEIGHT[level_index] as u32) << 16;
            if let Some(px) = player_x_pos {
                enemy.x_pos = px;
            }
            if let Some(py) = player_y_pos {
                enemy.y_pos = py.saturating_sub(hover_height_fp);
            }
        }
        enemy.wait_timer = emo_rock_wait_countdown_loc_36212(enemy.wait_timer).remaining_wait_timer;
        return;
    }

    if let (Some(grid), Some(px), Some(py)) = (block_grid, player_x_pos, player_y_pos) {
        let level_index = enemy.hit_points.saturating_sub(1).min(2) as usize;
        let target = emo_rock_hover_target_loc_3621a(px, py, EMO_ROCK_HOVER_HEIGHT[level_index]);
        enemy.x_pos = target.x_pos;
        enemy.y_pos = target.y_pos;

        let x_px = i32::try_from(enemy.x_pos >> 16).map_or(i32::MAX, |value| value);
        let y_px = i32::try_from(enemy.y_pos >> 16).map_or(i32::MAX, |value| value);
        if !emo_rock_clearance_probe_loc_3639e(
            grid,
            x_px,
            y_px,
            target.clearance_height_px,
            target.clearance_half_width_px,
        ) {
            return;
        }
    }

    // Dropping phase: apply gravity, check floor.
    let level_index = enemy.hit_points.saturating_sub(1).min(2) as usize;
    let gravity = EMO_ROCK_GRAVITY[level_index];
    enemy.y_velocity = emo_rock_drop_velocity_loc_3625e(enemy.y_velocity, gravity);
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;

    if let Some(grid) = block_grid {
        let cx = (enemy.x_pos >> 16) as i32;
        let cy = (enemy.y_pos >> 16) as i32;
        let floor = terrain::probe_floor(grid, cx, cy);
        if floor.on_ground {
            // ASM Emo_Rock.asm loc_362A4: snap to floor, clear vertical
            // velocity, then let `stru_363FE` run before loc_36320.
            enemy.y_pos = (floor.floor_y_px as u32) << 16;
            enemy.y_velocity = 0;
            enemy.on_ground = true;
            enemy.wait_timer = EMO_ROCK_SETTLED_ANIMATION_FRAMES_LOC_362A4;
            enemy.grounded_frames = 0;
            if let Some(dir) = player_relative_direction(enemy.x_pos, player_x_pos) {
                enemy.facing = dir;
            }
        }
    }
}

fn step_emo_rock_berzerker_airborne(enemy: &mut ActiveEnemy) {
    // ASM Emo_Rock.asm loc_3630A: hibernate one frame with `is_moved` still
    // set from init, then add `$6000` to y_vel and loop until `$19(a3)` death.
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;
    enemy.y_pos = (enemy.y_pos as i64 + i64::from(enemy.y_velocity)) as u32;
    enemy.y_velocity =
        emo_rock_berzerker_airborne_frame_loc_3630a(enemy.y_velocity, false).y_velocity;
    enemy.on_ground = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emo_rock_test_enemy(x_pos: u32, y_pos: u32, hit_points: u8) -> ActiveEnemy {
        ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::EmoRock,
            x_pos,
            y_pos,
            x_velocity: 0,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points,
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
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            crab_berzerker_knockback: false,
            shared_walker_state: spawn_shared_walker_state(EnemyFamily::EmoRock),
            shared_walker_patrol_timer: spawn_shared_walker_patrol_timer(EnemyFamily::EmoRock),
            shared_walker_turnaround_counter: 0,
            shared_walker_right_turn_extra_step_consumed: false,
            shared_walker_patrol_short_next: spawn_shared_walker_patrol_short_next(
                EnemyFamily::EmoRock,
            ),
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        }
    }

    fn grid_with_solid_cell(
        width: u16,
        height: u16,
        solid_col: u16,
        solid_row: u16,
    ) -> terrain::BlockGrid {
        let mut data = vec![0u8; usize::from(width) * usize::from(height) * 2];
        let offset = (usize::from(solid_row) * usize::from(width) + usize::from(solid_col)) * 2;
        data[offset] = 0x60;
        data[offset + 1] = 0x01;
        terrain::BlockGrid::from_level_layout(&data, width, height)
    }

    #[test]
    fn emo_rock_wait_countdown_loc_36212_loops_until_subq_reaches_zero() {
        assert_eq!(
            emo_rock_wait_countdown_loc_36212(3),
            EmoRockWaitCountdownLoc36212 {
                remaining_wait_timer: 2,
                repeats_wait_loop: true,
            },
            "ASM Emo_Rock.asm loc_36212 branches back while `subq.w #1,d0` leaves a non-zero wait word"
        );
        assert_eq!(
            emo_rock_wait_countdown_loc_36212(1),
            EmoRockWaitCountdownLoc36212 {
                remaining_wait_timer: 0,
                repeats_wait_loop: false,
            },
            "ASM Emo_Rock.asm loc_36212 falls through when the countdown reaches zero"
        );
    }

    #[test]
    fn emo_rock_hover_target_loc_3621a_tracks_kid_and_seeds_clearance_probe() {
        assert_eq!(
            emo_rock_hover_target_loc_3621a(0x0123_0000, 0x00C8_0000, 0x50),
            EmoRockHoverTargetLoc3621A {
                x_pos: 0x0123_0000,
                y_pos: 0x0078_0000,
                clearance_height_px: 0x001C,
                clearance_half_width_px: 0x000E,
            },
            "ASM Emo_Rock.asm loc_3621A copies Kid X, stores Kid Y minus d1, then calls the clearance probe with d6=$1C and d7=$E"
        );
    }

    #[test]
    fn emo_rock_drop_velocity_loc_3625e_caps_only_after_threshold() {
        assert_eq!(
            emo_rock_drop_velocity_loc_3625e(0x0005_C000, 0x0000_4000),
            0x0006_0000,
            "ASM Emo_Rock.asm loc_3625E keeps the accelerated y velocity when the signed compare is equal to `$60000`"
        );
        assert_eq!(
            emo_rock_drop_velocity_loc_3625e(0x0006_0000, 0x0000_4000),
            0x0004_0000,
            "ASM Emo_Rock.asm loc_3625E snaps y velocity to `$40000` only when the accelerated value is greater than `$60000`"
        );
    }

    #[test]
    fn emo_rock_drop_frame_loc_36274_routes_collision_after_storing_y_velocity() {
        assert_eq!(
            emo_rock_drop_frame_loc_36274(0x0004_0000, 0, 0x0120_0000, 0x0110_0000),
            EmoRockDropFrameLoc36274 {
                y_velocity: 0x0004_0000,
                next_collision_type: 0,
                hibernates_one_frame: true,
                route: EmoRockDropRouteLoc36274::ContinueDrop,
            },
            "ASM Emo_Rock.asm loc_36274 writes d7 to y_vel, hibernates one frame, then loops while collision_type is zero"
        );
        assert_eq!(
            emo_rock_drop_frame_loc_36274(0x0004_0000, -1, 0x0120_0000, 0x0110_0000).route,
            EmoRockDropRouteLoc36274::Death,
            "ASM Emo_Rock.asm loc_36274 branches negative collision words to the destroy path before clearing them"
        );
        assert_eq!(
            emo_rock_drop_frame_loc_36274(0x0004_0000, 0x001C, 0x0120_0000, 0x0110_0000),
            EmoRockDropFrameLoc36274 {
                y_velocity: 0x0004_0000,
                next_collision_type: 0,
                hibernates_one_frame: true,
                route: EmoRockDropRouteLoc36274::ContinueDrop,
            },
            "ASM Emo_Rock.asm loc_36274 clears positive collision words but keeps dropping when the word is >= `$1C`"
        );
        assert_eq!(
            emo_rock_drop_frame_loc_36274(0x0004_0000, 0x000C, 0x0120_0000, 0x011F_0000),
            EmoRockDropFrameLoc36274 {
                y_velocity: 0x0004_0000,
                next_collision_type: 0,
                hibernates_one_frame: true,
                route: EmoRockDropRouteLoc36274::Land {
                    facing: Direction::Left,
                },
            },
            "ASM Emo_Rock.asm loc_36274 leaves x_direction clear when Kid's integer X is left of the rock"
        );
        assert_eq!(
            emo_rock_drop_frame_loc_36274(0x0004_0000, 0x000C, 0x0120_0000, 0x0120_0000).route,
            EmoRockDropRouteLoc36274::Land {
                facing: Direction::Right,
            },
            "ASM Emo_Rock.asm loc_36274 sets x_direction when Kid's integer X is equal to or right of the rock"
        );
    }

    #[test]
    fn emo_rock_landing_setup_loc_362a4_clears_drop_state() {
        assert_eq!(
            emo_rock_landing_setup_loc_362a4(),
            EmoRockLandingSetupLoc362A4 {
                y_velocity: 0,
                has_level_collision: false,
                animation_script: "stru_363FE",
                next_loop: "settled_wait_loop",
            },
            "ASM Emo_Rock.asm loc_362A4 clears y_vel, disables level collision, loads `stru_363FE`, and enters the settled wait loop"
        );
    }

    #[test]
    fn emo_rock_settled_wait_loc_362b6_routes_animation_and_collision() {
        assert_eq!(
            emo_rock_settled_wait_loc_362b6(true, 0x0020, false),
            EmoRockSettledWaitFrameLoc362B6 {
                next_collision_type: 0x0020,
                hibernates_one_frame: true,
                route: EmoRockSettledWaitRouteLoc362B6::Disappear,
            },
            "ASM Emo_Rock.asm loc_362B6 tests animation completion before reading collision_type"
        );
        assert_eq!(
            emo_rock_settled_wait_loc_362b6(false, 0, false).route,
            EmoRockSettledWaitRouteLoc362B6::ContinueWait,
            "ASM Emo_Rock.asm loc_362B6 loops while collision_type is zero"
        );
        assert_eq!(
            emo_rock_settled_wait_loc_362b6(false, -1, false),
            EmoRockSettledWaitFrameLoc362B6 {
                next_collision_type: 0xFFFF,
                hibernates_one_frame: true,
                route: EmoRockSettledWaitRouteLoc362B6::Death,
            },
            "ASM Emo_Rock.asm loc_362B6 branches negative collision words to death before clearing them"
        );
        assert_eq!(
            emo_rock_settled_wait_loc_362b6(false, 0x001C, false),
            EmoRockSettledWaitFrameLoc362B6 {
                next_collision_type: 0,
                hibernates_one_frame: true,
                route: EmoRockSettledWaitRouteLoc362B6::Death,
            },
            "ASM Emo_Rock.asm loc_362B6 clears positive collision words, then sends hurt contacts to death"
        );
        assert_eq!(
            emo_rock_settled_wait_loc_362b6(false, 0x002C, false).route,
            EmoRockSettledWaitRouteLoc362B6::Death,
            "ASM Emo_Rock.asm loc_362B6 sends Kid-above contacts to death"
        );
        assert_eq!(
            emo_rock_settled_wait_loc_362b6(false, 0x0020, false).route,
            EmoRockSettledWaitRouteLoc362B6::ContinueWait,
            "ASM Emo_Rock.asm loc_362B6 ignores side contacts unless Berzerker is charging"
        );
        assert_eq!(
            emo_rock_settled_wait_loc_362b6(false, 0x0024, true).route,
            EmoRockSettledWaitRouteLoc362B6::BerzerkerSideHit,
            "ASM Emo_Rock.asm loc_362B6 routes side contacts only when Berzerker_charging is set"
        );
        assert_eq!(
            emo_rock_settled_wait_loc_362b6(false, 0x0010, false).route,
            EmoRockSettledWaitRouteLoc362B6::ContinueWait,
            "ASM Emo_Rock.asm loc_362B6 clears unhandled positive collision words and keeps waiting"
        );
    }

    #[test]
    fn emo_rock_berzerker_launch_loc_362ee_copies_kid_velocity_and_flags() {
        assert_eq!(
            emo_rock_berzerker_launch_loc_362ee(0x0001_8000),
            EmoRockBerzerkerLaunchLoc362Ee {
                has_kid_collision: true,
                has_level_collision: false,
                x_velocity: 0x0001_8000,
                y_velocity: -0x0003_0000,
                next_loop: "berzerker_airborne_loop",
            },
            "ASM Emo_Rock.asm loc_362EE copies Kid x_vel, sets y_vel=$FFFD0000, and falls through to the airborne loop"
        );
    }

    #[test]
    fn emo_rock_berzerker_airborne_loc_3630a_adds_gravity_until_flag_19() {
        assert_eq!(
            emo_rock_berzerker_airborne_frame_loc_3630a(-0x0003_0000, false),
            EmoRockBerzerkerAirborneFrameLoc3630A {
                y_velocity: -0x0002_A000,
                hibernates_one_frame: true,
                route: EmoRockBerzerkerAirborneRouteLoc3630A::ContinueAirborne,
            },
            "ASM Emo_Rock.asm loc_3630A hibernates once, adds `$6000` to y_vel, and loops while `$19(a3)` is clear"
        );
        assert_eq!(
            emo_rock_berzerker_airborne_frame_loc_3630a(0x0001_2000, true),
            EmoRockBerzerkerAirborneFrameLoc3630A {
                y_velocity: 0x0001_2000,
                hibernates_one_frame: true,
                route: EmoRockBerzerkerAirborneRouteLoc3630A::Death,
            },
            "ASM Emo_Rock.asm loc_3630A branches to death before adding gravity when `$19(a3)` is set"
        );
    }

    #[test]
    fn emo_rock_disappear_loc_36320_resets_sprite_after_animation() {
        assert_eq!(
            emo_rock_disappear_loc_36320(),
            EmoRockDisappearLoc36320 {
                has_kid_collision: true,
                sound_effect: "sfx_Emo_Rock_disappears",
                animation_script: "stru_36456",
                waits_until_animation_finished: true,
                addroffset_sprite: 0,
                next_loop: "restart_hover_cycle",
            },
            "ASM Emo_Rock.asm loc_36320 plays the vanish sound/animation, waits for completion, clears addroffset_sprite, and restarts hovering"
        );
    }

    #[test]
    fn emo_rock_death_start_loc_36346_runs_break_sequence_before_status_route() {
        assert_eq!(
            emo_rock_death_start_loc_36346(),
            EmoRockDeathStartLoc36346 {
                has_kid_collision: true,
                is_moved: false,
                animation_script: "stru_36468",
                hibernates_before_sound: true,
                sound_effect: "sfx_Prize_block",
                waits_until_animation_finished: true,
                reads_object_status_slot: true,
            },
            "ASM Emo_Rock.asm loc_36346 stops movement, plays the break animation/sound sequence, then reads `$42(a5)` for status routing"
        );
    }

    #[test]
    fn emo_rock_respawn_marker_loc_3638c_writes_positive_status_slot() {
        assert_eq!(
            emo_rock_respawn_marker_loc_3638c(0x05),
            EmoRockRespawnMarkerLoc3638C {
                index: 0x05,
                word_value: 0x2168,
            },
            "ASM Emo_Rock.asm loc_3638C doubles positive `$42(a5)` into an EnemyStatus_Table word offset and writes `$2168`"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn emo_rock_death_status_loc_3639A_deletes_after_every_status_route() {
        let positive = emo_rock_death_status_loc_3639A(0x05);
        assert_eq!(
            positive.status_route,
            EmoRockDeathStatusRouteLoc3639A::WriteRespawnMarker(EmoRockRespawnMarkerLoc3638C {
                index: 0x05,
                word_value: 0x2168,
            }),
            "ASM Emo_Rock.asm loc_3639A is reached after positive status slots write the respawn marker"
        );
        assert!(positive.deletes_current_object);

        assert_eq!(
            emo_rock_death_status_loc_3639A(0xC7).status_route,
            EmoRockDeathStatusRouteLoc3639A::SubtractSpawnCounter {
                index: 0x07,
                decrement: 0x0400,
            },
            "ASM Emo_Rock.asm loc_3639A is reached after bit-7+bit-6 status slots subtract `$400` from the masked EnemyStatus_Table entry"
        );

        assert_eq!(
            emo_rock_death_status_loc_3639A(0x80),
            EmoRockDeathStatusLoc3639A {
                status_route: EmoRockDeathStatusRouteLoc3639A::DeleteOnly,
                deletes_current_object: true,
            },
            "ASM Emo_Rock.asm loc_3639A deletes directly when bit 7 is set and bit 6 is clear"
        );
    }

    #[test]
    fn step_emo_rock_settled_zero_wait_enters_disappear_loc_362b6_36320() {
        let mut enemy = emo_rock_test_enemy(0x0064_0000, 0x00E0_0000, 1);
        enemy.on_ground = true;
        enemy.wait_timer = 0;
        let mut sound_effects = Vec::new();

        step_emo_rock_with_rng(
            &mut enemy,
            None,
            None,
            None,
            &mut RomRandomState::default(),
            Some(&mut sound_effects),
        );

        assert_eq!(
            enemy.grounded_frames, EMO_ROCK_DISAPPEAR_ANIMATION_FRAMES_LOC_36320,
            "ASM Emo_Rock.asm loc_362B6 falls through immediately when the settled wait word is already zero"
        );
        assert_eq!(enemy.wait_timer, 0);
        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::EmoRockDisappears],
            "ASM Emo_Rock.asm loc_36320 plays the disappear sound before hibernating on stru_36456"
        );
    }

    #[test]
    fn step_emo_rock_hover_partial_y_update_uses_fp_height_loc_3621a() {
        let mut enemy = emo_rock_test_enemy(0x0040_0000, 0x0010_0000, 2);
        enemy.wait_timer = 2;

        step_emo_rock_with_rng(
            &mut enemy,
            None,
            None,
            Some(0x00C8_0000),
            &mut RomRandomState::default(),
            None,
        );

        assert_eq!(
            enemy.x_pos, 0x0040_0000,
            "ASM Emo_Rock.asm loc_3621A only copies Kid X when the caller has a player X sample"
        );
        assert_eq!(
            enemy.y_pos, 0x008C_0000,
            "ASM Emo_Rock.asm loc_3621A stores Kid Y minus the level-1 `$3C` hover height as FP16.16"
        );
        assert_eq!(enemy.wait_timer, 1);
    }

    #[test]
    fn step_emo_rock_drop_floor_probe_and_snap_use_fp_pixels_loc_36274_362a4() {
        let grid = grid_with_solid_cell(8, 5, 2, 3);
        let mut enemy = emo_rock_test_enemy(0x0020_0000, 0x002F_C000, 1);
        enemy.wait_timer = 0;

        step_emo_rock_with_rng(
            &mut enemy,
            Some(&grid),
            Some(0x0030_0000),
            None,
            &mut RomRandomState::default(),
            None,
        );

        assert!(enemy.on_ground);
        assert_eq!(
            enemy.y_pos, 0x0030_0000,
            "ASM Emo_Rock.asm loc_36274 probes the integer Y pixel, then loc_362A4 snaps the floor pixel back to FP16.16"
        );
        assert_eq!(enemy.y_velocity, 0);
        assert_eq!(
            enemy.wait_timer,
            EMO_ROCK_SETTLED_ANIMATION_FRAMES_LOC_362A4
        );
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Emo_Rock.asm loc_36274 sets x_direction when Kid is equal to or right of the rock"
        );
    }
}
