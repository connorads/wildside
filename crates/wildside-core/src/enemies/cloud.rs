use super::*;

/// Cloud flies horizontally, reversing direction on wall contact.
/// ASM `Cloud.asm loc_35BDE -> loc_35C54`: `move.w #$3C,d3` is written after
/// wall reversal, but the active loop never reads `d3`, so Cloud resumes
/// flying immediately on the next frame instead of idling.
#[cfg_attr(not(test), allow(dead_code))]
const CLOUD_CONTROL_SCRIPT_WORDS_LOC_35BAC: [u16; 5] = [0x7800, 0x0004, 0x1E00, 0x0008, 0xFFFF];

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn cloud_control_script_words_loc_35BAC() -> &'static [u16] {
    // ASM Cloud.asm loc_35BAC: `lea (unk_35BC4).l,a0` restarts the command
    // stream before the dispatcher fetches the next big-endian word.
    &CLOUD_CONTROL_SCRIPT_WORDS_LOC_35BAC
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum CloudControlDispatchLoc35BB2 {
    Wait { frames: u8 },
    SetStatus44,
    ClearStatus44,
    PlayTransitionAnimation,
    RestartScript,
    Unknown { table_offset: u8 },
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn cloud_control_script_dispatch_loc_35BB2(
    command_word: u16,
) -> CloudControlDispatchLoc35BB2 {
    // ASM Cloud.asm loc_35BB2: `move.w (a0)+,d0; bmi.s ...; move.b d0,d7`
    // uses the low byte as a longword-table offset while wait commands keep
    // their duration in the high byte for the wait loop.
    if command_word & 0x8000 != 0 {
        return CloudControlDispatchLoc35BB2::RestartScript;
    }

    let [frames, table_offset] = command_word.to_be_bytes();
    match table_offset {
        0x00 => CloudControlDispatchLoc35BB2::Wait { frames },
        0x04 => CloudControlDispatchLoc35BB2::SetStatus44,
        0x08 => CloudControlDispatchLoc35BB2::ClearStatus44,
        0x0C => CloudControlDispatchLoc35BB2::PlayTransitionAnimation,
        _ => CloudControlDispatchLoc35BB2::Unknown { table_offset },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CloudWaitLoopLoc35C56 {
    pub(crate) dbf_seed: u16,
    pub(crate) hibernate_frames: u32,
    pub(crate) checks_object_range_each_frame: bool,
    pub(crate) polls_collision_each_frame: bool,
    pub(crate) updates_idle_sprite_each_frame: bool,
    pub(crate) checks_attack_setup_each_frame: bool,
    pub(crate) resumes_control_script: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn cloud_wait_loop_loc_35C56(command_word: u16) -> CloudWaitLoopLoc35C56 {
    // ASM Cloud.asm loc_35C54 shifts the command word down by one byte; loc_35C56
    // then hibernates, range-checks, polls collision, idles/attacks, and DBFs.
    let dbf_seed = command_word >> 8;
    CloudWaitLoopLoc35C56 {
        dbf_seed,
        hibernate_frames: u32::from(dbf_seed) + 1,
        checks_object_range_each_frame: true,
        polls_collision_each_frame: true,
        updates_idle_sprite_each_frame: true,
        checks_attack_setup_each_frame: true,
        resumes_control_script: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CloudStatusLatchLoc35C72 {
    pub(crate) object_44_value: u8,
    pub(crate) resumes_control_script: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn cloud_status_latch_loc_35C72() -> CloudStatusLatchLoc35C72 {
    // ASM Cloud.asm loc_35C72: `st $44(a5)` writes `$FF`, then branches back
    // to the command dispatcher at loc_35BB2.
    CloudStatusLatchLoc35C72 {
        object_44_value: 0xFF,
        resumes_control_script: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CloudStatusLatchLoc35C7A {
    pub(crate) object_44_value: u8,
    pub(crate) resumes_control_script: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn cloud_status_latch_loc_35C7A() -> CloudStatusLatchLoc35C7A {
    // ASM Cloud.asm loc_35C7A: `sf $44(a5)` writes `$00`, then branches back
    // to the command dispatcher at loc_35BB2.
    CloudStatusLatchLoc35C7A {
        object_44_value: 0x00,
        resumes_control_script: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CloudTransitionLoc35C82 {
    pub(crate) animation_script: &'static str,
    pub(crate) hibernates_until_animation_finished: bool,
    pub(crate) restarts_control_script: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn cloud_transition_loc_35C82() -> CloudTransitionLoc35C82 {
    // ASM Cloud.asm loc_35C82: initialise `stru_35EC8`, hibernate until that
    // animation completes, then branch to loc_35BAC to reload the command stream.
    CloudTransitionLoc35C82 {
        animation_script: "stru_35EC8",
        hibernates_until_animation_finished: true,
        restarts_control_script: true,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
const CLOUD_IDLE_SPRITES_LOC_35CA2: [&str; 6] = [
    "LnkTo_unk_C787E-Data_Index",
    "LnkTo_unk_C7886-Data_Index",
    "LnkTo_unk_C788E-Data_Index",
    "LnkTo_unk_C7896-Data_Index",
    "LnkTo_unk_C788E-Data_Index",
    "LnkTo_unk_C7886-Data_Index",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CloudIdleSpriteGateLoc35CA2 {
    pub(crate) d2_after_subq: u16,
    pub(crate) sprite_link: Option<&'static str>,
    pub(crate) reaches_countdown_reload: bool,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn cloud_idle_sprite_gate_loc_35CA2(
    object_44: u8,
    object_45: u8,
    d2: u16,
    random_byte: u8,
) -> CloudIdleSpriteGateLoc35CA2 {
    // ASM Cloud.asm loc_35CA2: only an armed idle cloud (`$44(a5)` set and
    // `$45(a5)` clear) decrements d2; when it reaches zero, `(random & $2F)`
    // is even-aligned and used as a byte offset into off_35C94.
    if object_44 == 0 || object_45 != 0 {
        return CloudIdleSpriteGateLoc35CA2 {
            d2_after_subq: d2,
            sprite_link: None,
            reaches_countdown_reload: false,
        };
    }

    let d2_after_subq = d2.wrapping_sub(1);
    if d2_after_subq != 0 {
        return CloudIdleSpriteGateLoc35CA2 {
            d2_after_subq,
            sprite_link: None,
            reaches_countdown_reload: false,
        };
    }

    let masked_random = u16::from(random_byte) & 0x002F;
    let sprite_link = if masked_random <= 0x000A {
        let table_index = usize::from((masked_random & !1) / 2);
        Some(CLOUD_IDLE_SPRITES_LOC_35CA2[table_index])
    } else {
        None
    };

    CloudIdleSpriteGateLoc35CA2 {
        d2_after_subq,
        sprite_link,
        reaches_countdown_reload: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CloudIdleSpriteResetLoc35CD2 {
    pub(crate) d2: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn cloud_idle_sprite_reset_loc_35CD2() -> CloudIdleSpriteResetLoc35CD2 {
    // ASM Cloud.asm loc_35CD2: every elapsed idle-sprite attempt reloads the
    // d2 countdown with five frames before returning to the wait loop.
    CloudIdleSpriteResetLoc35CD2 { d2: 5 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CloudDiagonalAttackInputLoc35CD8 {
    pub(crate) object_45: u8,
    pub(crate) animation_done_flag_18: bool,
    pub(crate) x_direction: u8,
    pub(crate) cloud_x: i16,
    pub(crate) cloud_y: i16,
    pub(crate) kid_x: i16,
    pub(crate) kid_y: i16,
    pub(crate) attack_x_word: i16,
    pub(crate) attack_y_word: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloudDiagonalAttackRouteLoc35CD8 {
    ContinueExistingAttack,
    LaterRangeChecks,
    StartAttack {
        x_word: i16,
        y_word: i16,
    },
    SpawnAttackSprite {
        allocation_selector: u16,
        projectile_worker: &'static str,
        x_word: i16,
        y_word: i16,
        projectile_x_direction: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CloudDiagonalAttackSetupLoc35CD8 {
    pub(crate) route: CloudDiagonalAttackRouteLoc35CD8,
    pub(crate) animation_script: Option<&'static str>,
    pub(crate) object_45_value: Option<u8>,
}

fn cloud_diagonal_attack_later_range_checks_loc_35cd8() -> CloudDiagonalAttackSetupLoc35CD8 {
    CloudDiagonalAttackSetupLoc35CD8 {
        route: CloudDiagonalAttackRouteLoc35CD8::LaterRangeChecks,
        animation_script: None,
        object_45_value: None,
    }
}

fn cloud_diagonal_attack_start_loc_35cd8(
    x_word: i16,
    y_word: i16,
) -> CloudDiagonalAttackSetupLoc35CD8 {
    // ASM Cloud.asm loc_35D9E: every attack vector converges here to initialise
    // `stru_35EC8` and set `$45(a5)` with `st`.
    CloudDiagonalAttackSetupLoc35CD8 {
        route: CloudDiagonalAttackRouteLoc35CD8::StartAttack { x_word, y_word },
        animation_script: Some("stru_35EC8"),
        object_45_value: Some(0xFF),
    }
}

#[allow(non_snake_case)]
pub(crate) fn cloud_diagonal_attack_setup_loc_35CD8(
    input: CloudDiagonalAttackInputLoc35CD8,
) -> CloudDiagonalAttackSetupLoc35CD8 {
    // ASM Cloud.asm loc_35CD8: ignore new setup while `$45(a5)` is set, then
    // compare signed Kid-minus-cloud position words to find the diagonal band
    // in front of the cloud before starting `stru_35EC8` and setting `$45(a5)`.
    if input.object_45 != 0 {
        // ASM Cloud.asm loc_35DAE: an active attack returns while `$18(a3)` is
        // clear; once set, loc_35DB8 allocates `loc_35DF0`, copies the live
        // attack vector from `d1`, flips `x_direction`, restores `stru_35EA0`,
        // and clears `$45(a5)`.
        if input.animation_done_flag_18 {
            return CloudDiagonalAttackSetupLoc35CD8 {
                route: CloudDiagonalAttackRouteLoc35CD8::SpawnAttackSprite {
                    allocation_selector: 0x8000,
                    projectile_worker: "loc_35DF0",
                    x_word: input.attack_x_word,
                    y_word: input.attack_y_word,
                    projectile_x_direction: !input.x_direction,
                },
                animation_script: Some("stru_35EA0"),
                object_45_value: Some(0x00),
            };
        }

        return CloudDiagonalAttackSetupLoc35CD8 {
            route: CloudDiagonalAttackRouteLoc35CD8::ContinueExistingAttack,
            animation_script: None,
            object_45_value: None,
        };
    }

    let d7 = input.kid_y.wrapping_sub(input.cloud_y);
    if d7 >= 0 {
        let d6 = input.kid_x.wrapping_sub(input.cloud_x);
        if input.x_direction == 0 {
            if d6 < 0 {
                let abs_dx = d6.wrapping_neg();
                if d7 <= abs_dx && d7 >= abs_dx.wrapping_sub(0x10) {
                    return cloud_diagonal_attack_start_loc_35cd8(-3, 3);
                }
            }
        } else {
            // ASM Cloud.asm loc_35D1E: non-zero `x_direction` mirrors the
            // down-diagonal attack band to Kid's right side.
            if d6 >= 0 && d7 <= d6 && d7 >= d6.wrapping_sub(0x10) {
                return cloud_diagonal_attack_start_loc_35cd8(3, 3);
            }
        }
    }

    // ASM Cloud.asm loc_35D40: if Kid is below and horizontally inside
    // (-$10, +$10], start the straight-down attack before later checks.
    if d7 >= 0 {
        let d6 = input.kid_x.wrapping_sub(input.cloud_x);
        if d6 <= 0x10 && d6 > -0x10 {
            return cloud_diagonal_attack_start_loc_35cd8(0, 6);
        }
    }

    // ASM Cloud.asm loc_35D62: within the signed +/-$18 Y window, launch
    // horizontally only when Kid is in front of the cloud's current direction.
    if (-0x18..=0x18).contains(&d7) {
        let d6 = input.kid_x.wrapping_sub(input.cloud_x);
        if input.x_direction == 0 {
            if d6 < 0 {
                return cloud_diagonal_attack_start_loc_35cd8(-6, 0);
            }
        } else if d6 >= 0 {
            // ASM Cloud.asm loc_35D8C: non-zero `x_direction` accepts only
            // non-negative Kid-minus-cloud X and swaps x=6/y=0 into `d1`.
            return cloud_diagonal_attack_start_loc_35cd8(6, 0);
        }
    }

    cloud_diagonal_attack_later_range_checks_loc_35cd8()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloudAttackProjectileRouteLoc35E8A {
    ContinueLoop,
    DeleteTail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CloudAttackProjectileLoopLoc35E8A {
    pub(crate) hibernates_one_frame: bool,
    pub(crate) route: CloudAttackProjectileRouteLoc35E8A,
}

#[allow(non_snake_case)]
pub(crate) fn cloud_attack_projectile_loop_loc_35E8A(
    collision_type: u16,
    animation_flag_19: bool,
) -> CloudAttackProjectileLoopLoc35E8A {
    // ASM Cloud.asm loc_35E8A: after `j_Hibernate_Object_1Frame`, any non-zero
    // `collision_type(a3)` exits before byte `$19(a3)` is tested.
    let route = if collision_type != 0 || animation_flag_19 {
        CloudAttackProjectileRouteLoc35E8A::DeleteTail
    } else {
        CloudAttackProjectileRouteLoc35E8A::ContinueLoop
    };

    CloudAttackProjectileLoopLoc35E8A {
        hibernates_one_frame: true,
        route,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CloudAttackProjectileDeleteTailLoc35E9C {
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
pub(crate) fn cloud_attack_projectile_delete_tail_loc_35E9C()
-> CloudAttackProjectileDeleteTailLoc35E9C {
    // ASM Cloud.asm loc_35E9C: spawned attack projectile tail-calls
    // `j_Delete_CurrentObject` without extra cleanup.
    CloudAttackProjectileDeleteTailLoc35E9C {
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CloudDeathSequenceLoc35C0E {
    pub(crate) x_velocity: i32,
    pub(crate) animation_script: &'static str,
    pub(crate) status_route: CloudDeathStatusRouteLoc35C0E,
    pub(crate) delete_tail: CloudDeleteTailLoc35C50,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CloudRespawnMarkerLoc35C42 {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloudDeathStatusRouteLoc35C0E {
    WriteRespawnMarker(CloudRespawnMarkerLoc35C42),
    SubtractSpawnCounter { index: usize, decrement: u16 },
    DeleteOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CloudDeleteTailLoc35C50 {
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
pub(crate) fn cloud_delete_tail_loc_35C50() -> CloudDeleteTailLoc35C50 {
    // ASM Cloud.asm loc_35C50: every death-status route converges on
    // `jmp (j_Delete_CurrentObject).w`.
    CloudDeleteTailLoc35C50 {
        deletes_current_object: true,
    }
}

#[allow(non_snake_case)]
pub(crate) fn cloud_respawn_marker_loc_35C42(object_status_slot: u8) -> CloudRespawnMarkerLoc35C42 {
    // ASM Cloud.asm loc_35C42: positive `$42(a5)` is doubled as an
    // EnemyStatus_Table word offset, then `$2168` is written as the respawn marker.
    CloudRespawnMarkerLoc35C42 {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[allow(non_snake_case)]
pub(crate) fn cloud_death_sequence_loc_35C0E(object_status_slot: u8) -> CloudDeathSequenceLoc35C0E {
    // ASM Cloud.asm loc_35C0E: clear `x_vel(a3)`, run `stru_35EBA` to
    // completion, apply the `$42(a5)` status route, then delete the object.
    let status_route = if object_status_slot & 0x80 == 0 {
        CloudDeathStatusRouteLoc35C0E::WriteRespawnMarker(cloud_respawn_marker_loc_35C42(
            object_status_slot,
        ))
    } else if object_status_slot & 0x40 != 0 {
        CloudDeathStatusRouteLoc35C0E::SubtractSpawnCounter {
            index: usize::from(object_status_slot & 0x3F),
            decrement: 0x0400,
        }
    } else {
        CloudDeathStatusRouteLoc35C0E::DeleteOnly
    };

    CloudDeathSequenceLoc35C0E {
        x_velocity: 0,
        animation_script: "stru_35EBA",
        status_route,
        delete_tail: cloud_delete_tail_loc_35C50(),
    }
}

pub(crate) fn step_cloud(enemy: &mut ActiveEnemy, block_grid: Option<&terrain::BlockGrid>) {
    // Advance position
    enemy.x_pos = (enemy.x_pos as i64 + i64::from(enemy.x_velocity)) as u32;

    // Wall probe: reverse on wall contact (Cloud.asm loc_35BDE)
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
            enemy.x_velocity = -enemy.x_velocity;
            enemy.facing = opposite_direction(enemy.facing);
        }
    }
}

fn cloud_x_direction_byte(facing: Direction) -> u8 {
    match facing {
        // ASM Cloud.asm loc_35CD8: clear x_direction means the attack bands
        // are in front of a left-moving cloud; non-zero mirrors them right.
        Direction::Left => 0x00,
        Direction::Right => 0xFF,
    }
}

fn cloud_projectile_spawn_loc_35df0(
    enemy: &ActiveEnemy,
    x_word: i16,
    y_word: i16,
    projectile_x_direction: u8,
) -> EnemyProjectile {
    // ASM Cloud.asm loc_35DF0: the projectile copies Cloud's x/y words,
    // subtracts 11px from Y, stores $44/$46 as x/y velocity words, and nudges
    // diagonal sprites by +4px or -4px depending on the copied direction.
    let mut x_px = (enemy.x_pos >> 16) as i32;
    let y_px = ((enemy.y_pos >> 16) as i32 - 0x0B).max(0);
    if x_word != 0 && y_word != 0 {
        x_px += 4;
        if projectile_x_direction != 0 {
            x_px -= 8;
        }
    }

    EnemyProjectile {
        x_pos: (x_px.max(0) as u32) << 16,
        y_pos: (y_px as u32) << 16,
        x_velocity: i32::from(x_word) << 16,
        y_velocity: i32::from(y_word) << 16,
        // ASM loc_35E8A deletes on collision or the projectile animation's
        // `$19` flag, with no DBF lifetime counter.
        lifetime: 0,
        friendly: false,
        gravity: 0,
        homing_accel: 0,
        speed_cap: 0,
        wall_bounce: false,
        lion_projectile_bounce_empty_checks_loc_3e2c0: 0,
        reaim_interval: 0,
        reaim_countdown: 0,
        current_angle: 0,
        terrain_kill: true,
        drips_spawn_crawler_on_splash: false,
        owner: None,
        kind: EnemyProjectileKind::CloudLightning,
        palette_variant: 0,
        shooter_tracks_layout: false,
        shooter_layout_col: 0,
        shooter_layout_row: 0,
        shooter_state_word: 0,
        shooter_skip_projectile_step_frame: None,
        animation_elapsed_frames: 0,
        boss_head_boomerang_direction_byte_d0: 0,
        drips_surface_crawler_state: DripsSurfaceCrawlerState::HorizontalLoc33650,
        drips_surface_crawler_x_direction_flipped: false,
        ufo_beam_child_x_direction: 0,
        ufo_beam_child_loop_timer_loc_3554a: 0,
        drips_child_byte_44: 0,
        drips_child_byte_45: 0,
        tar_monster_collision_response_budget: 0,
        dragon_flying_descent_fire_state: DragonFlyingDescentFireState::None,
        dragon_flying_descent_fire_counter: 0,
        boss_death_burst_state: BossDeathBurstState::None,
        collision_type: 0,
        boss_eye_sprite_mapping_label: "",
        boss_eye_pop_effect_x_direction: 0,
    }
}

pub(crate) fn step_cloud_attack(
    enemy: &mut ActiveEnemy,
    projectiles: &mut Vec<EnemyProjectile>,
    player_x_pos: Option<u32>,
    player_y_pos: Option<u32>,
) {
    // Cloud uses the otherwise-UFO-only fields as the small native state for
    // ASM `$45(a5)` plus the latched `$44/$46(a0)` projectile vector.
    let x_direction = cloud_x_direction_byte(enemy.facing);
    if enemy.ufo_beam_segments != 0 {
        let setup = cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
            object_45: 0xFF,
            animation_done_flag_18: true,
            x_direction,
            // ASM Cloud.asm loc_35DAE/loc_35DB8 only reuses latched `d1`,
            // `$18(a3)`, and the flipped direction byte; positions are ignored.
            cloud_x: 0,
            cloud_y: 0,
            kid_x: 0,
            kid_y: 0,
            attack_x_word: (enemy.target_x_velocity >> 16) as i16,
            attack_y_word: (enemy.target_y_velocity >> 16) as i16,
        });
        if let CloudDiagonalAttackRouteLoc35CD8::SpawnAttackSprite {
            x_word,
            y_word,
            projectile_x_direction,
            ..
        } = setup.route
        {
            if projectiles.len() < MAX_PROJECTILES {
                projectiles.push(cloud_projectile_spawn_loc_35df0(
                    enemy,
                    x_word,
                    y_word,
                    projectile_x_direction,
                ));
            }
            enemy.ufo_beam_segments = 0;
            enemy.target_x_velocity = 0;
            enemy.target_y_velocity = 0;
        }
        return;
    }

    let (Some(player_x_pos), Some(player_y_pos)) = (player_x_pos, player_y_pos) else {
        return;
    };
    if projectiles.len() >= MAX_PROJECTILES {
        return;
    }

    let setup = cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
        object_45: 0,
        animation_done_flag_18: false,
        x_direction,
        cloud_x: (enemy.x_pos >> 16) as i16,
        cloud_y: (enemy.y_pos >> 16) as i16,
        kid_x: (player_x_pos >> 16) as i16,
        kid_y: (player_y_pos >> 16) as i16,
        attack_x_word: 0,
        attack_y_word: 0,
    });
    if let CloudDiagonalAttackRouteLoc35CD8::StartAttack { x_word, y_word } = setup.route {
        enemy.ufo_beam_segments = 1;
        enemy.target_x_velocity = i32::from(x_word) << 16;
        enemy.target_y_velocity = i32::from(y_word) << 16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain;

    fn cloud_attack_input(
        x_direction: u8,
        cloud_x: i16,
        cloud_y: i16,
        kid_x: i16,
        kid_y: i16,
    ) -> CloudDiagonalAttackInputLoc35CD8 {
        CloudDiagonalAttackInputLoc35CD8 {
            object_45: 0x00,
            animation_done_flag_18: false,
            x_direction,
            cloud_x,
            cloud_y,
            kid_x,
            kid_y,
            attack_x_word: 0,
            attack_y_word: 0,
        }
    }

    fn cloud_start_attack(x_word: i16, y_word: i16) -> CloudDiagonalAttackSetupLoc35CD8 {
        CloudDiagonalAttackSetupLoc35CD8 {
            route: CloudDiagonalAttackRouteLoc35CD8::StartAttack { x_word, y_word },
            animation_script: Some("stru_35EC8"),
            object_45_value: Some(0xFF),
        }
    }

    fn cloud_later_range_checks() -> CloudDiagonalAttackSetupLoc35CD8 {
        CloudDiagonalAttackSetupLoc35CD8 {
            route: CloudDiagonalAttackRouteLoc35CD8::LaterRangeChecks,
            animation_script: None,
            object_45_value: None,
        }
    }

    fn grid_with_solid_cells(
        width_cells: u16,
        height_cells: u16,
        solid_cells: &[(u16, u16)],
    ) -> terrain::BlockGrid {
        let mut data = vec![0; usize::from(width_cells) * usize::from(height_cells) * 2];
        for &(col, row) in solid_cells {
            let offset = (usize::from(row) * usize::from(width_cells) + usize::from(col)) * 2;
            data[offset] = 0xE0;
            data[offset + 1] = 0;
        }
        terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells)
    }

    fn cloud_enemy_for_attack() -> ActiveEnemy {
        ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Cloud,
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            x_velocity: -0x3000,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 1,
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
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
            shared_walker_turnaround_counter: 0,
            shared_walker_right_turn_extra_step_consumed: false,
            shared_walker_patrol_short_next: false,
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        }
    }

    #[test]
    fn cloud_control_script_dispatch_loc_35bb2_decodes_transition_offset() {
        assert_eq!(
            cloud_control_script_dispatch_loc_35BB2(0x000C),
            CloudControlDispatchLoc35BB2::PlayTransitionAnimation,
            "ASM Cloud.asm off_35BCE maps table offset $0C to loc_35C82 transition animation"
        );
    }

    #[test]
    fn cloud_diagonal_attack_setup_loc_35cd8_keeps_diagonal_bands_strict() {
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(cloud_attack_input(0, 100, 50, 100, 50)),
            cloud_start_attack(0, 6),
            "ASM Cloud.asm loc_35CD8 clear x_direction requires Kid-minus-cloud X to be negative before the diagonal band; dx=0 falls through to loc_35D40"
        );
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(cloud_attack_input(0, 100, 50, 92, 70)),
            cloud_start_attack(0, 6),
            "ASM Cloud.asm loc_35CD8 rejects d7 > abs(dx) before loc_35D40 can start the straight-down attack"
        );
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(cloud_attack_input(0xFF, 100, 50, 108, 70)),
            cloud_start_attack(0, 6),
            "ASM Cloud.asm loc_35D1E rejects d7 > dx for mirrored diagonal attacks before loc_35D40"
        );
    }

    #[test]
    fn cloud_diagonal_attack_setup_loc_35d40_keeps_column_bounds_asymmetric() {
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(cloud_attack_input(0, 100, 50, 84, 70)),
            cloud_start_attack(-6, 0),
            "ASM Cloud.asm loc_35D40 uses `cmpi #$FFF0; ble` so dx=-16 is outside the straight-down column and reaches loc_35D62"
        );
    }

    #[test]
    fn cloud_diagonal_attack_setup_loc_35d62_requires_kid_in_front() {
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(cloud_attack_input(0, 100, 50, 100, 40)),
            cloud_later_range_checks(),
            "ASM Cloud.asm loc_35D62 clear x_direction uses `bpl return_35DAC`, so dx=0 is not in front"
        );
    }

    #[test]
    fn cloud_loc_35bde_treats_zero_velocity_as_right_wall_probe() {
        let grid = grid_with_solid_cells(20, 15, &[(4, 4), (4, 5)]);
        let mut enemy = cloud_enemy_for_attack();
        enemy.x_pos = 64 << 16;
        enemy.y_pos = 96 << 16;
        enemy.x_velocity = 0;
        enemy.facing = Direction::Left;

        step_cloud(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Cloud.asm loc_35BDE treats zero x_vel as the right-facing wall probe before toggling x_direction on contact"
        );
    }

    #[test]
    fn cloud_runtime_loc_35d1e_latches_right_diagonal_lightning() {
        let mut active = vec![cloud_enemy_for_attack()];
        active[0].facing = Direction::Right;
        active[0].x_velocity = 0x3000;
        let mut projectiles = Vec::new();
        let mut drilled_blocks = Vec::new();

        step_active_enemies(
            &mut active,
            None,
            Some(140 << 16),
            Some(89 << 16),
            None,
            None,
            PlayerForm::TheKid,
            Some(&mut projectiles),
            &mut drilled_blocks,
        );

        assert!(
            projectiles.is_empty(),
            "ASM loc_35D1E should latch before loc_35DB8 allocates the mirrored attack worker",
        );
        assert_eq!(active[0].ufo_beam_segments, 1);
        assert_eq!(active[0].target_x_velocity, 0x3_0000);
        assert_eq!(active[0].target_y_velocity, 0x3_0000);
    }

    #[test]
    fn cloud_projectile_spawn_loc_35df0_positions_vectors_and_offsets() {
        let enemy = cloud_enemy_for_attack();

        let vertical = cloud_projectile_spawn_loc_35df0(&enemy, 0, 6, 0);
        assert_eq!(vertical.x_pos, 100 << 16);
        assert_eq!(vertical.y_pos, 39 << 16);
        assert_eq!(vertical.x_velocity, 0);
        assert_eq!(vertical.y_velocity, 0x6_0000);

        let diagonal_left = cloud_projectile_spawn_loc_35df0(&enemy, -3, 3, 0xFF);
        assert_eq!(
            diagonal_left.x_pos,
            96 << 16,
            "ASM Cloud.asm loc_35DF0 nudges diagonal sprites +4px, then subtracts 8px when copied x_direction is non-zero"
        );
        assert_eq!(diagonal_left.y_pos, 39 << 16);
        assert_eq!(diagonal_left.x_velocity, -0x3_0000);
        assert_eq!(diagonal_left.y_velocity, 0x3_0000);

        let diagonal_right = cloud_projectile_spawn_loc_35df0(&enemy, 3, 3, 0x00);
        assert_eq!(
            diagonal_right.x_pos,
            104 << 16,
            "ASM Cloud.asm loc_35DF0 leaves the +4px diagonal nudge when copied x_direction is clear"
        );
        assert_eq!(diagonal_right.y_pos, 39 << 16);
        assert_eq!(diagonal_right.x_velocity, 0x3_0000);
        assert_eq!(diagonal_right.y_velocity, 0x3_0000);
    }

    #[test]
    fn cloud_runtime_loc_35db8_respects_projectile_capacity() {
        let mut enemy = cloud_enemy_for_attack();
        enemy.ufo_beam_segments = 1;
        enemy.target_x_velocity = -0x3_0000;
        enemy.target_y_velocity = 0x3_0000;
        let mut projectiles = vec![EnemyProjectile::default(); MAX_PROJECTILES];

        step_cloud_attack(&mut enemy, &mut projectiles, None, None);

        assert_eq!(
            projectiles.len(),
            MAX_PROJECTILES,
            "ASM Cloud.asm loc_35DB8 allocation is mirrored by native projectile capacity; a full pool must not append a lightning worker"
        );
        assert_eq!(enemy.ufo_beam_segments, 0);
        assert_eq!(enemy.target_x_velocity, 0);
        assert_eq!(enemy.target_y_velocity, 0);
    }
}
