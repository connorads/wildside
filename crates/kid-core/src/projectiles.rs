use super::*;
#[cfg(test)]
use crate::combat::CollisionSizeEntrySub21F8;
use crate::enemies::{
    DragonFlyingDescentFireFirstLoopLoc34C26, DragonFlyingDescentFireTravelLoopLoc34C3A,
    DripsLeftBoundsNextLoc335AA, DripsRightBoundsNextLoc33588,
    DripsSurfaceCrawlerTurnFieldLoc33AF6, DripsSurfaceCrawlerTurnNextLoc33AF6,
    DripsSurfaceCrawlerTurnScriptLoc33AF6, ROBOT_SHOT_IMPACT_ANIMATION_FRAMES_LOC_3A762,
    RobotShotFlightRouteLoc3AC48, TankBulletLoopRouteLoc3B7BA,
    TarMonsterProjectileCollisionRouteLoc32682, TarMonsterProjectileCollisionTargetLoc32682,
    archer_angle_to_player, archer_reaim_loc_3432E, dragon_flying_descent_fire_cleanup_loc_34C4A,
    dragon_flying_descent_fire_first_loop_loc_34C26,
    dragon_flying_descent_fire_travel_loop_loc_34C3A, drips_child_state_bytes_loc_3333e,
    drips_enter_splash_animation_loc_33488, drips_left_bounds_gate_loc_335aa,
    drips_right_bounds_gate_loc_33588, drips_splash_animation_remaining_loc_33488,
    drips_splash_reset_loc_33488, drips_surface_crawler_init_loc_335d8,
    drips_surface_crawler_turn_script_loc_33a0e, drips_surface_crawler_turn_script_loc_33a2c,
    drips_surface_crawler_turn_script_loc_33a04, drips_surface_crawler_turn_script_loc_33a5e,
    drips_surface_crawler_turn_script_loc_33a18, drips_surface_crawler_turn_script_loc_33a22,
    drips_surface_crawler_turn_script_loc_33a36, drips_surface_crawler_turn_script_loc_33a40,
    drips_surface_crawler_turn_script_loc_33a72, drips_surface_crawler_turn_script_loc_339a0,
    drips_surface_crawler_turn_script_loc_339aa, drips_surface_crawler_turn_script_loc_339b4,
    drips_surface_crawler_turn_script_loc_339be, drips_surface_crawler_turn_script_loc_339c8,
    drips_surface_crawler_turn_script_loc_339d2, drips_surface_crawler_turn_script_loc_339dc,
    drips_surface_crawler_turn_script_loc_339e6, drips_surface_crawler_turn_script_loc_339f0,
    drips_surface_crawler_turn_script_loc_339fa, drips_surface_crawler_turn_script_loc_3393c,
    drips_surface_crawler_turn_script_loc_3395a, drips_surface_crawler_turn_script_loc_3396e,
    drips_surface_crawler_turn_script_loc_3398c, drips_surface_crawler_turn_script_loc_33946,
    drips_surface_crawler_turn_script_loc_33950, drips_surface_crawler_turn_script_loc_33964,
    drips_surface_crawler_turn_script_loc_33978, drips_surface_crawler_turn_script_loc_33982,
    drips_surface_crawler_turn_script_loc_33996, drips_velocity_apply_loc_33546,
    drips_velocity_deltas_loc_33526, drips_vertical_bounds_contact_flag_loc_335b0,
    drips_wait_timer_loc_3340e, robot_shot_flight_loop_loc_3ac48, robot_shot_impact_prep_loc_3ac7e,
    spawn_tar_monster_retaliation, tank_bullet_loop_route_loc_3b7ba,
    tar_monster_projectile_collision_loc_32682,
    tar_monster_projectile_negated_velocity_swap_loc_326C6,
    tar_monster_projectile_reflect_x_loc_326AA, tar_monster_projectile_reflect_y_loc_326B8,
    tar_monster_projectile_velocity_swap_loc_326DC,
};
use crate::helmets::{
    RainSlotSpriteAction, SnakeTargetCandidate, diamond_power_rain_compute_tracking_velocity,
    diamond_power_rain_find_nearest_enemy, diamond_power_rain_screen_cull_and_compile,
};
use crate::terrain::{
    relative_collision_column_loc_331F6, relative_collision_mask_loc_331D2,
    relative_collision_mask_loc_3330E, relative_collision_row_loc_33286,
};

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidProjectileCollisionPassSub1D76 {
    pub(crate) object_list_head_address: u32,
    pub(crate) collision_mode: u8,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn kid_projectile_collision_passes_sub_1D76() -> [KidProjectileCollisionPassSub1D76; 2] {
    // ASM sub_1D76 (kid.asm:3537): dispatch Kid projectile collision through
    // sub_1D84 twice, first with list head `$FFFFF862` and `d7=0`, then with
    // list head `$FFFFF5A0` and `d7=1` before falling through into sub_1D84.
    [
        KidProjectileCollisionPassSub1D76 {
            object_list_head_address: 0x00FF_F862,
            collision_mode: 0,
        },
        KidProjectileCollisionPassSub1D76 {
            object_list_head_address: 0x00FF_F5A0,
            collision_mode: 1,
        },
    ]
}

fn fire_demon_trail_has_floor_support_loc_3a63c(
    grid: &terrain::BlockGrid,
    x_pos: u32,
    y_pos: u32,
) -> bool {
    // ASM Fire_Demon loc_3A63C calls loc_33286 with d7=#8 and d6=#$3E7.
    // loc_33286 samples three consecutive cells starting at x_pos-$E on
    // row (y_pos+8)>>4 and returns raw d5/d6/d7 words on the #$3E7 bypass.
    let x_px = (x_pos >> 16) as u16 as i16;
    let y_px = (y_pos >> 16) as u16 as i16;
    let support =
        relative_collision_row_loc_33286(grid, i32::from(x_px), i32::from(y_px), 8, 0x03E7);

    // ASM Fire_Demon loc_3A63C: `bclr #$F,d6; bne loc_3A6DA` deletes when
    // the centre support word has the block flag set, then all three sampled
    // collision nibbles must compare equal to `$6000`.
    (support.centre & 0x8000) == 0
        && (support.left & 0x7000) == 0x6000
        && (support.centre & 0x7000) == 0x6000
        && (support.right & 0x7000) == 0x6000
}

pub(crate) const DRIPS_SURFACE_CRAWLER_LIMIT_LOC_33488: usize = 5;
const DRIPS_SURFACE_CRAWLER_LIFETIME: u16 = u16::MAX;
const DRIPS_SURFACE_CRAWLER_STARTUP_FRAMES_LOC_335D8: u16 = 31;
const DRIPS_SURFACE_CRAWLER_KID_CONTACT_HALF_WIDTH_LOC_33550: i32 = PROJECTILE_HITBOX_HALF + 9;
const DRIPS_DROP_FLOOR_PROBE_HALF_WIDTH_LOC_3345C: i32 = 5;
/// ASM Drips.asm loc_3345C exits the fall loop when GfxObjects_Collision
/// reports `$38(a1) == $C`. GfxObjects_Collision_ChkBoundaries (kid.asm
/// loc_173E) builds the hitbox as `(x_off, width, y_off, height)` words, so
/// the falling child's box bottom sits `$A` below y_pos (the same `$A` that
/// loc_33488 later adds via `addi.w #$A,$1E(a1)` before the crawler spawn).
/// Oracle issue-260 floor probe: the drop leaves y=530 at frame 152, the last
/// loc_3345C frame is 181 at y=582, and the splash rests at y=592 — contact
/// fires exactly when `y+$A` enters the solid row-37 tile, not `y+$1F`.
const DRIPS_DROP_FLOOR_PROBE_HEIGHT_LOC_3345C: i32 = 0x0A;
const DRAGON_FLYING_DESCENT_FIRE_CLEANUP_HIBERNATE_FRAMES_LOC_34C4A: u16 = 3;

fn dragon_flying_descent_fire_start_cleanup_loc_34c4a(projectile: &mut EnemyProjectile) {
    let cleanup = dragon_flying_descent_fire_cleanup_loc_34C4A();
    projectile.x_velocity = cleanup.x_velocity;
    projectile.y_velocity = cleanup.y_velocity;
    projectile.terrain_kill = cleanup.has_level_collision;
    projectile.dragon_flying_descent_fire_state =
        DragonFlyingDescentFireState::CleanupFirstLoc34C4A;
    projectile.dragon_flying_descent_fire_counter = cleanup.first_animation_hibernate_frames;
}

fn dragon_flying_descent_fire_offscreen_flag_loc_34c3a(
    projectile: &EnemyProjectile,
    camera: CameraState,
) -> bool {
    let screen_x = ((projectile.x_pos >> 16) as i32) - ((camera.x_pos >> 16) as i32);
    let screen_y = ((projectile.y_pos >> 16) as i32) - ((camera.y_pos >> 16) as i32);

    // ASM Dragon_Flying.asm loc_34C3A tests byte `$19` after the hibernated
    // travel frame. Native treats the same lifecycle flag as set once the child
    // has left the broad object-retention box around the camera.
    !(-0x80..=0x01C0).contains(&screen_x) || !(-0x80..=0x01C0).contains(&screen_y)
}

fn step_dragon_flying_descent_fire_loc_34c26(
    projectile: &mut EnemyProjectile,
    grid: Option<&terrain::BlockGrid>,
    camera: Option<CameraState>,
) -> bool {
    let collision_type =
        u16::from(grid.is_some_and(|grid| projectile_terrain_contact_loc_3345c(grid, projectile)));

    match projectile.dragon_flying_descent_fire_state {
        DragonFlyingDescentFireState::FirstAnimationLoc34C26 => {
            match dragon_flying_descent_fire_first_loop_loc_34C26(
                projectile.dragon_flying_descent_fire_counter,
                collision_type,
            ) {
                DragonFlyingDescentFireFirstLoopLoc34C26::CollisionCleanup { .. } => {
                    dragon_flying_descent_fire_start_cleanup_loc_34c4a(projectile);
                }
                DragonFlyingDescentFireFirstLoopLoc34C26::ContinueFirstAnimation { next_d0 } => {
                    projectile.dragon_flying_descent_fire_counter = next_d0;
                }
                DragonFlyingDescentFireFirstLoopLoc34C26::EnterTravelAnimation { .. } => {
                    projectile.dragon_flying_descent_fire_state =
                        DragonFlyingDescentFireState::TravelAnimationLoc34C3A;
                    projectile.dragon_flying_descent_fire_counter = 0;
                }
            }
            true
        }
        DragonFlyingDescentFireState::TravelAnimationLoc34C3A => {
            let flag_19 = camera.is_some_and(|camera| {
                dragon_flying_descent_fire_offscreen_flag_loc_34c3a(projectile, camera)
            });
            match dragon_flying_descent_fire_travel_loop_loc_34C3A(collision_type, flag_19) {
                DragonFlyingDescentFireTravelLoopLoc34C3A::ContinueTravelAnimation => true,
                DragonFlyingDescentFireTravelLoopLoc34C3A::CollisionCleanup { .. }
                | DragonFlyingDescentFireTravelLoopLoc34C3A::AnimationFinishedCleanup { .. } => {
                    dragon_flying_descent_fire_start_cleanup_loc_34c4a(projectile);
                    true
                }
            }
        }
        DragonFlyingDescentFireState::CleanupFirstLoc34C4A => {
            if projectile.dragon_flying_descent_fire_counter > 1 {
                projectile.dragon_flying_descent_fire_counter -= 1;
                true
            } else {
                projectile.dragon_flying_descent_fire_state =
                    DragonFlyingDescentFireState::CleanupSecondLoc34C4A;
                projectile.dragon_flying_descent_fire_counter =
                    DRAGON_FLYING_DESCENT_FIRE_CLEANUP_HIBERNATE_FRAMES_LOC_34C4A;
                true
            }
        }
        DragonFlyingDescentFireState::CleanupSecondLoc34C4A => {
            if projectile.dragon_flying_descent_fire_counter > 1 {
                projectile.dragon_flying_descent_fire_counter -= 1;
                true
            } else {
                false
            }
        }
        DragonFlyingDescentFireState::None => true,
    }
}

fn apply_tar_monster_burst_child_collision_loc_32682(
    projectile: &mut EnemyProjectile,
    grid: &terrain::BlockGrid,
) -> bool {
    let px_i = (projectile.x_pos >> 16) as i32;
    let py_i = (projectile.y_pos >> 16) as i32;
    let col = terrain::pixel_to_cell(px_i).max(0) as u16;
    let row = terrain::pixel_to_cell(py_i).max(0) as u16;
    let cell = grid.cell_at(col, row);
    if !cell.has_block || cell.collision == terrain::CollisionKind::Empty {
        return true;
    }

    let prev_px =
        ((projectile.x_pos as i64 - i64::from(projectile.x_velocity)) as u32 >> 16) as i32;
    let prev_py =
        ((projectile.y_pos as i64 - i64::from(projectile.y_velocity)) as u32 >> 16) as i32;

    let col_h = terrain::pixel_to_cell(px_i).max(0) as u16;
    let row_h = terrain::pixel_to_cell(prev_py).max(0) as u16;
    let cell_h = grid.cell_at(col_h, row_h);

    let col_v = terrain::pixel_to_cell(prev_px).max(0) as u16;
    let row_v = terrain::pixel_to_cell(py_i).max(0) as u16;
    let cell_v = grid.cell_at(col_v, row_v);

    let h_blocked = cell_h.has_block && cell_h.collision != terrain::CollisionKind::Empty;
    let v_blocked = cell_v.has_block && cell_v.collision != terrain::CollisionKind::Empty;
    let collision_type = lion_projectile_terrain_collision_id_loc_3e260(
        h_blocked,
        v_blocked,
        projectile.x_velocity,
        projectile.y_velocity,
        cell,
    );
    let collision = tar_monster_projectile_collision_loc_32682(
        collision_type as i16,
        projectile.tar_monster_collision_response_budget,
        true,
    );

    match collision.route {
        TarMonsterProjectileCollisionRouteLoc32682::ReturnOnly => true,
        TarMonsterProjectileCollisionRouteLoc32682::DeleteCurrentObject => false,
        TarMonsterProjectileCollisionRouteLoc32682::Dispatch(dispatch) => {
            projectile.tar_monster_collision_response_budget = dispatch.remaining_collision_budget;
            match dispatch.target {
                TarMonsterProjectileCollisionTargetLoc32682::Loc326AA => {
                    projectile.x_velocity =
                        tar_monster_projectile_reflect_x_loc_326AA(projectile.x_velocity);
                    true
                }
                TarMonsterProjectileCollisionTargetLoc32682::Loc326B8 => {
                    projectile.y_velocity =
                        tar_monster_projectile_reflect_y_loc_326B8(projectile.y_velocity);
                    true
                }
                TarMonsterProjectileCollisionTargetLoc32682::Loc326C6 => {
                    let pair = tar_monster_projectile_negated_velocity_swap_loc_326C6(
                        projectile.x_velocity,
                        projectile.y_velocity,
                    );
                    projectile.x_velocity = pair.x_velocity;
                    projectile.y_velocity = pair.y_velocity;
                    true
                }
                TarMonsterProjectileCollisionTargetLoc32682::Loc326DC => {
                    let pair = tar_monster_projectile_velocity_swap_loc_326DC(
                        projectile.x_velocity,
                        projectile.y_velocity,
                    );
                    projectile.x_velocity = pair.x_velocity;
                    projectile.y_velocity = pair.y_velocity;
                    true
                }
                TarMonsterProjectileCollisionTargetLoc32682::Loc32652
                | TarMonsterProjectileCollisionTargetLoc32682::UnknownOffset(_) => false,
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PendingDripsSplash {
    owner: EnemyProjectileOwner,
    drop: EnemyProjectile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShooterObjectLayoutInteractionLocF00A {
    col: u16,
    row: u16,
    block_type: terrain::BlockType,
    skin: u8,
    destroy_direction: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DripsSurfaceCrawlerSideColumnStepLoc33650 {
    Loop,
    Turn(DripsSurfaceCrawlerTurnScriptLoc33AF6),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DripsSurfaceCrawlerVerticalStepLoc33730 {
    Loop,
    Turn(DripsSurfaceCrawlerTurnScriptLoc33AF6),
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DripsSurfaceCrawlerVerticalStepLoc336D0 {
    Loop,
    Turn(DripsSurfaceCrawlerTurnScriptLoc33AF6),
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DripsSurfaceCrawlerDiagonalStepLoc33810 {
    Loop,
    Turn(DripsSurfaceCrawlerTurnScriptLoc33AF6),
    Exit,
}

fn drips_surface_crawler_from_drop_loc_335d8(
    drop: &EnemyProjectile,
    random_byte: u8,
) -> EnemyProjectile {
    let splash_reset = drips_splash_reset_loc_33488(drop.x_pos, drop.y_pos);
    drips_surface_crawler_from_splash_loc_335d8(
        splash_reset.surface_child_x_pos,
        splash_reset.surface_child_y_pos,
        drop.drips_child_byte_44,
        drop.drips_child_byte_45,
        random_byte,
    )
}

fn drips_surface_crawler_from_splash_loc_335d8(
    splash_x_pos: u32,
    splash_y_pos: u32,
    child_byte_44: u8,
    child_byte_45: u8,
    random_byte: u8,
) -> EnemyProjectile {
    let drop_x_word = (splash_x_pos >> 16) as u16;
    let drop_y_word = (splash_y_pos >> 16) as u16;
    let queued_x_word = drop_x_word.wrapping_add(1);
    let queued_y_word = drop_y_word.wrapping_add(0x000A) & 0xFFF0;
    // ASM Drips.asm loc_33488 first adds `$A` to the falling child Y, then
    // the negative off_33520 sentinel queues loc_335D8 with another `$A`
    // aligned into `$46(a0)` before loc_33630 tails through loc_33526.
    let init = drips_surface_crawler_init_loc_335d8(
        queued_x_word,
        queued_y_word,
        child_byte_44,
        child_byte_45,
        random_byte,
    );
    EnemyProjectile {
        kind: EnemyProjectileKind::DripsSurfaceCrawler,
        x_pos: u32::from(init.x_pos_word) << 16,
        y_pos: u32::from(init.y_pos_word) << 16,
        x_velocity: init.x_velocity,
        y_velocity: 0,
        lifetime: DRIPS_SURFACE_CRAWLER_LIFETIME,
        friendly: false,
        gravity: 0,
        terrain_kill: false,
        // ASM Drips.asm loc_335D8 runs `stru_33FA6` to completion before
        // `st is_moved(a3)`, `loc_33526`, and the first loc_33650 hibernate.
        drips_surface_crawler_state: DripsSurfaceCrawlerState::StartupLoc335D8,
        drips_surface_crawler_x_direction_flipped: init.x_direction != 0,
        drips_child_byte_44: init.child_byte_44,
        drips_child_byte_45: init.child_byte_45,
        animation_elapsed_frames: DRIPS_SURFACE_CRAWLER_STARTUP_FRAMES_LOC_335D8,
        ..Default::default()
    }
}

fn drips_surface_crawler_finish_startup_loc_335d8(projectile: &mut EnemyProjectile) {
    projectile.animation_elapsed_frames = 0;
    projectile.drips_surface_crawler_state = DripsSurfaceCrawlerState::HorizontalLoc33650;
    let startup_deltas = drips_velocity_deltas_loc_33526(
        projectile.drips_child_byte_44,
        projectile.drips_child_byte_45,
        projectile.x_velocity,
        projectile.y_velocity,
    );
    let startup_velocity = drips_velocity_apply_loc_33546(
        projectile.x_velocity,
        projectile.y_velocity,
        startup_deltas.x_delta,
        startup_deltas.y_delta,
    );
    projectile.x_velocity = startup_velocity.x_velocity;
    projectile.y_velocity = startup_velocity.y_velocity;
}

pub(crate) fn drips_surface_crawler_burst_helper_loc_338f8(
    crawler: &EnemyProjectile,
    x_offset: i16,
    y_offset: i16,
) -> EnemyProjectile {
    let x_word = ((crawler.x_pos >> 16) as u16).wrapping_add(x_offset as u16);
    let y_word = ((crawler.y_pos >> 16) as u16).wrapping_add(y_offset as u16);
    EnemyProjectile {
        kind: EnemyProjectileKind::DripsSurfaceCrawlerBurst,
        x_pos: u32::from(x_word) << 16,
        y_pos: u32::from(y_word) << 16,
        x_velocity: crawler.x_velocity.wrapping_shl(1),
        y_velocity: 0,
        lifetime: 60,
        gravity: 0x1000,
        friendly: false,
        terrain_kill: false,
        tar_monster_collision_response_budget: 2,
        ..Default::default()
    }
}

fn drips_surface_crawler_finish_animation_loc_3392c(crawler: &EnemyProjectile) -> EnemyProjectile {
    EnemyProjectile {
        kind: EnemyProjectileKind::DripsSurfaceCrawlerFinish,
        x_pos: crawler.x_pos,
        y_pos: crawler.y_pos,
        x_velocity: 0,
        y_velocity: 0,
        lifetime: 60,
        gravity: 0,
        friendly: false,
        terrain_kill: false,
        ..Default::default()
    }
}

fn drips_surface_crawler_exit_via_loc_338f8(crawler: &mut EnemyProjectile) {
    let (x_offset, y_offset) =
        drips_surface_crawler_burst_offset_loc_33550(crawler.drips_surface_crawler_state)
            .unwrap_or((0, 0));
    *crawler = drips_surface_crawler_burst_helper_loc_338f8(crawler, x_offset, y_offset);
}

pub(crate) fn drips_surface_crawler_burst_offset_loc_33550(
    state: DripsSurfaceCrawlerState,
) -> Option<(i16, i16)> {
    match state {
        // ASM Drips.asm loc_33650: `moveq #0,d6; moveq #-8,d7`
        // before loc_33550 and the support-probe loc_338F8 exit.
        DripsSurfaceCrawlerState::HorizontalLoc33650 => Some((0, -8)),
        // ASM Drips.asm loc_336D0: `moveq #0,d6; moveq #8,d7`.
        DripsSurfaceCrawlerState::VerticalLoc336D0 => Some((0, 8)),
        // ASM Drips.asm loc_33730: `moveq #8,d6; moveq #0,d7`.
        DripsSurfaceCrawlerState::VerticalLoc33730 => Some((8, 0)),
        // ASM Drips.asm loc_337A0/loc_33810/loc_33892 seed these offsets
        // immediately before the contact/bounds guard.
        DripsSurfaceCrawlerState::VerticalLoc337A0 => Some((-8, 0)),
        DripsSurfaceCrawlerState::DiagonalLoc33810 => Some((-8, -8)),
        DripsSurfaceCrawlerState::DiagonalLoc33892 => Some((8, -8)),
        DripsSurfaceCrawlerState::StartupLoc335D8 => None,
        DripsSurfaceCrawlerState::IdleLoc33A7C => None,
    }
}

#[cfg(test)]
mod drips_surface_crawler_tests {
    use super::*;
    use std::path::PathBuf;

    fn minimal_manifest() -> kid_data::AssetManifest {
        kid_data::AssetManifest {
            generated_bundle_version: kid_data::GENERATED_BUNDLE_VERSION,
            rom_sha1: String::from("projectiles-test"),
            upstream_commit: String::from("projectiles-test"),
            source_root: PathBuf::new(),
            output_root: PathBuf::new(),
            map_order: Vec::new(),
            levels: Vec::new(),
            constants: Vec::new(),
            ram_variables: Vec::new(),
            enemy_sources: Vec::new(),
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_levels: Vec::new(),
        }
    }

    fn drips_ready_splash_enemy(layout_index: usize, x_pos: u32, y_pos: u32) -> ActiveEnemy {
        ActiveEnemy {
            segment_index: 0,
            layout_index,
            enemy_level: 0,
            family: EnemyFamily::Drips,
            x_pos,
            y_pos,
            x_velocity: 0,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 1,
            death_animation_active: false,
            grounded_frames: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: drips_splash_animation_timer_loc_33488(0),
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
            drips_attached: true,
            drips_surface_table_index: 1,
            drips_pending_splash_x_pos: 0,
            drips_pending_splash_y_pos: 0,
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: BOSS_HEAD_NONE,
            boss_movement_delay_counter: 0,
            boss_movement_script_index: 0,
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
    fn drips_surface_crawler_spawn_applies_parent_velocity_gate_loc_33526() {
        let drop = EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 0x0123_0000,
            y_pos: 0x00E6_0000,
            drips_child_byte_44: 0xFF,
            drips_child_byte_45: 0xFF,
            ..Default::default()
        };

        let mut crawler = drips_surface_crawler_from_drop_loc_335d8(&drop, 0);

        assert_eq!(
            crawler.x_velocity, 0x0000_3000,
            "ASM Drips.asm loc_335D8 seeds x_vel before the startup animation; loc_33526 does not run until `stru_33FA6` finishes"
        );
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::StartupLoc335D8
        );
        drips_surface_crawler_finish_startup_loc_335d8(&mut crawler);

        assert_eq!(
            crawler.x_velocity, 0x0000_9000,
            "ASM Drips.asm loc_335D8 copies `$44/$45`; loc_33630 tails through loc_33526, so both non-zero bytes add two extra `$3000` x-velocity deltas to the initial crawler speed"
        );
        assert_eq!(crawler.drips_child_byte_44, 0xFF);
        assert_eq!(crawler.drips_child_byte_45, 0xFF);
    }

    #[test]
    fn drips_surface_crawler_spawn_uses_post_splash_y_loc_33488() {
        let drop = EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 0x0247_0000,
            y_pos: 0x023E_0000,
            ..Default::default()
        };

        let crawler = drips_surface_crawler_from_drop_loc_335d8(&drop, 0);

        assert_eq!(
            crawler.y_pos, 0x0250_0000,
            "ASM Drips.asm loc_33488 adds 10px to the falling child before the loc_335D8 queue adds another 10px and aligns down into `$46(a0)`"
        );
    }

    #[test]
    fn drips_ready_splash_reseeds_in_object_slot_order_loc_33488() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.rom_random = RomRandomState::default();
        runtime.active_enemies = vec![
            drips_ready_splash_enemy(7, 0x01C8_0000, 0x021F_0000),
            drips_ready_splash_enemy(8, 0x0248_0000, 0x021F_0000),
        ];
        let mut crawler_count = 0;
        let mut spawned_projectiles = Vec::new();

        runtime.resolve_ready_drips_splash_animations(
            &mut crawler_count,
            Some(0x0241_0000),
            &mut spawned_projectiles,
        );
        let mut expected_rng = RomRandomState::default();
        let first_reseed_byte = expected_rng.next_byte();
        let second_reseed_byte = expected_rng.next_byte();

        let left_wait = runtime
            .active_enemies
            .iter()
            .find(|enemy| enemy.layout_index == 7)
            .expect("left Drips should remain active")
            .wait_timer;
        let right_wait = runtime
            .active_enemies
            .iter()
            .find(|enemy| enemy.layout_index == 8)
            .expect("right Drips should remain active")
            .wait_timer;

        assert_eq!(
            right_wait,
            drips_wait_timer_loc_3340e(first_reseed_byte, 0x0241),
            "ASM Drips.asm loc_33488 resumes each controller coroutine in object-slot order; oracle issue-260 frame 250 shows layout #08 consumes the first ready-splash reseed"
        );
        assert_eq!(
            left_wait,
            drips_wait_timer_loc_3340e(second_reseed_byte, 0x0241),
            "layout #07 must consume the next loc_3340E RNG byte after the right-side Drips continuation"
        );
    }

    #[test]
    fn drips_surface_crawler_spawn_persists_as_hostile_loc_335d8() {
        let crawler =
            drips_surface_crawler_from_splash_loc_335d8(0x0248_0000, 0x0229_0000, 0, 0, 0);

        assert_eq!(
            crawler.kind,
            EnemyProjectileKind::DripsSurfaceCrawler,
            "ASM Drips.asm loc_335D8 allocates the surface-crawler child object"
        );
        assert_eq!(
            crawler.lifetime, DRIPS_SURFACE_CRAWLER_LIFETIME,
            "ASM Drips.asm loc_335D8 enters the crawler coroutine; lifetime must not expire before loc_3392C"
        );
        assert!(
            !crawler.friendly,
            "ASM Drips.asm loc_335D8 creates a hostile object_meta `$16` child"
        );
        assert_eq!(
            crawler.y_velocity, 0,
            "ASM Drips.asm loc_335D8 seeds x_vel only; loc_33630 has no Y delta when `$44` is zero"
        );
        assert!(!crawler.terrain_kill);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::StartupLoc335D8
        );
        assert_eq!(
            crawler.animation_elapsed_frames, DRIPS_SURFACE_CRAWLER_STARTUP_FRAMES_LOC_335D8,
            "ASM `stru_33FA6` has three 10-frame entries, then `Hibernate_UntilAnimFinished` consumes the resume frame before loc_335D8 reaches loc_33630/loc_33526"
        );

        let flipped =
            drips_surface_crawler_from_splash_loc_335d8(0x0248_0000, 0x0229_0000, 0, 0, 0x04);
        assert!(
            flipped.drips_surface_crawler_x_direction_flipped,
            "ASM Drips.asm loc_335D8 uses `bclr #2,d7` to set flipped x_direction when random bit 2 was set"
        );
        assert_eq!(
            flipped.x_velocity, -0x0000_3000,
            "ASM Drips.asm loc_335D8 negates the initial crawler x_vel on the flipped random branch"
        );
    }

    #[test]
    fn drips_surface_crawler_burst_helper_shell_matches_loc_338f8() {
        let crawler = EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0248_0000,
            y_pos: 0x0220_0000,
            x_velocity: 0x0000_3000,
            y_velocity: 0x0000_5000,
            lifetime: DRIPS_SURFACE_CRAWLER_LIFETIME,
            ..Default::default()
        };

        let helper = drips_surface_crawler_burst_helper_loc_338f8(&crawler, 0, -8);

        assert_eq!(
            helper.kind,
            EnemyProjectileKind::DripsSurfaceCrawlerBurst,
            "ASM Drips.asm loc_338F8 allocates the shared Tar_Monster.asm loc_325B6 helper shell"
        );
        assert_eq!(helper.x_pos, 0x0248_0000);
        assert_eq!(
            helper.y_pos, 0x0218_0000,
            "ASM Drips.asm loc_33650 passes d6=0,d7=-8 before the loc_338F8 helper allocation"
        );
        assert_eq!(
            helper.x_velocity, 0x0000_6000,
            "ASM Tar_Monster.asm loc_325B6 doubles the copied x_vel for the helper shell"
        );
        assert_eq!(
            helper.y_velocity, 0,
            "ASM Tar_Monster.asm loc_325B6/loc_3261C does not copy crawler y_vel into this helper"
        );
        assert_eq!(
            helper.lifetime, 60,
            "native helper lifetime stands in for the loc_3261C/loc_32682 finish animation window"
        );
        assert_eq!(
            helper.gravity, 0x1000,
            "native helper gravity models the falling loc_32682 collision-response shell"
        );
        assert!(!helper.friendly);
        assert_eq!(helper.tar_monster_collision_response_budget, 2);
    }

    #[test]
    fn drips_surface_crawler_finish_shell_matches_loc_3392c() {
        let crawler = EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0248_0000,
            y_pos: 0x0220_0000,
            x_velocity: -0x0000_3000,
            y_velocity: 0x0000_5000,
            lifetime: DRIPS_SURFACE_CRAWLER_LIFETIME,
            ..Default::default()
        };

        let finish = drips_surface_crawler_finish_animation_loc_3392c(&crawler);

        assert_eq!(
            finish.kind,
            EnemyProjectileKind::DripsSurfaceCrawlerFinish,
            "ASM Drips.asm loc_3392C keeps an animation shell alive until the finish animation completes"
        );
        assert_eq!(finish.x_pos, crawler.x_pos);
        assert_eq!(finish.y_pos, crawler.y_pos);
        assert_eq!(
            finish.x_velocity, 0,
            "ASM Drips.asm loc_3392C stops the crawler before hibernating through the finish animation"
        );
        assert_eq!(finish.y_velocity, 0);
        assert_eq!(
            finish.lifetime, 60,
            "native finish-shell lifetime stands in for `Hibernate_UntilAnimFinished` before `$FFFFFB42` decrements"
        );
        assert_eq!(finish.gravity, 0);
        assert!(!finish.friendly);
    }

    #[test]
    fn drips_surface_crawler_burst_offsets_match_loc_33550_callers() {
        let cases = [
            (
                DripsSurfaceCrawlerState::HorizontalLoc33650,
                Some((0, -8)),
                "ASM Drips.asm loc_33650 sets d6=0,d7=-8",
            ),
            (
                DripsSurfaceCrawlerState::VerticalLoc336D0,
                Some((0, 8)),
                "ASM Drips.asm loc_336D0 sets d6=0,d7=8",
            ),
            (
                DripsSurfaceCrawlerState::VerticalLoc33730,
                Some((8, 0)),
                "ASM Drips.asm loc_33730 sets d6=8,d7=0",
            ),
            (
                DripsSurfaceCrawlerState::VerticalLoc337A0,
                Some((-8, 0)),
                "ASM Drips.asm loc_337A0 sets d6=-8,d7=0",
            ),
            (
                DripsSurfaceCrawlerState::DiagonalLoc33810,
                Some((-8, -8)),
                "ASM Drips.asm loc_33810 sets d6=-8,d7=-8",
            ),
            (
                DripsSurfaceCrawlerState::DiagonalLoc33892,
                Some((8, -8)),
                "ASM Drips.asm loc_33892 sets d6=8,d7=-8",
            ),
            (
                DripsSurfaceCrawlerState::IdleLoc33A7C,
                None,
                "ASM Drips.asm loc_33A7C idles instead of calling loc_33550",
            ),
        ];

        for (state, expected, reason) in cases {
            assert_eq!(
                drips_surface_crawler_burst_offset_loc_33550(state),
                expected,
                "{reason}"
            );
        }
    }
}

fn projectile_terrain_contact_loc_3345c(
    grid: &terrain::BlockGrid,
    projectile: &EnemyProjectile,
) -> bool {
    let px_i = (projectile.x_pos >> 16) as i32;
    let py_i = (projectile.y_pos >> 16) as i32;
    if projectile.kind == EnemyProjectileKind::DripsDrop {
        // ASM Drips.asm loc_3345C waits until the falling child reports
        // collision_type `$0C` (floor). GfxObjects_Collision's tile loop
        // (kid.asm loc_1792) overlaps the child's box rows top..bottom against
        // `$4000`-collision tiles, ignoring spikes (`$7000` at loc_17C6), so
        // contact fires on the frame the bottom edge (`y+$A`) enters a solid
        // row — oracle issue-260: last fall frame y=582, splash rest y=592.
        // A child whose origin row is already inside terrain takes the
        // wall-classified branch of GfxObjects_Collision_Solid, not `$0C`,
        // and keeps falling.
        let bottom_row =
            terrain::pixel_to_cell(py_i + DRIPS_DROP_FLOOR_PROBE_HEIGHT_LOC_3345C).max(0) as u16;
        let origin_row = terrain::pixel_to_cell(py_i).max(0) as u16;
        let left_col = terrain::pixel_to_cell(px_i - DRIPS_DROP_FLOOR_PROBE_HALF_WIDTH_LOC_3345C)
            .max(0) as u16;
        let right_col = terrain::pixel_to_cell(px_i + DRIPS_DROP_FLOOR_PROBE_HALF_WIDTH_LOC_3345C)
            .max(0) as u16;
        let row_has_floor_tile = |row: u16| {
            (left_col..=right_col).any(|col| {
                matches!(
                    grid.cell_at(col, row).collision,
                    terrain::CollisionKind::Solid
                        | terrain::CollisionKind::SlopeUp
                        | terrain::CollisionKind::SlopeDown
                )
            })
        };
        return row_has_floor_tile(bottom_row) && !row_has_floor_tile(origin_row);
    }

    {
        let col = terrain::pixel_to_cell(px_i).max(0) as u16;
        let row = terrain::pixel_to_cell(py_i).max(0) as u16;
        let cell = grid.cell_at(col, row);
        if projectile.kind == EnemyProjectileKind::ManiaxeAxe {
            // ASM loc_8A84 (kid.asm:9895): `andi.w #$7000,d7;
            // cmpi.w #$6000,d7; beq loc_8ACC` deletes the axe only on
            // solid-block collision bits. Slopes/hurt cells fall through.
            return cell.has_block && cell.collision == terrain::CollisionKind::Solid;
        }
        cell.has_block && cell.collision != terrain::CollisionKind::Empty
    }
}

fn drips_surface_crawler_out_of_level_bounds_loc_33588(
    projectile: &EnemyProjectile,
    level_width_pixels: u16,
    level_height_pixels: u16,
) -> bool {
    let x_pos_word = (projectile.x_pos >> 16) as u16 as i16;
    let y_pos_word = (projectile.y_pos >> 16) as u16 as i16;
    let x_gate = drips_right_bounds_gate_loc_33588(
        false,
        x_pos_word,
        projectile.x_velocity,
        level_width_pixels as i16,
    );
    let vertical_contact = match x_gate {
        DripsRightBoundsNextLoc33588::ContactLoc335D4 => return true,
        DripsRightBoundsNextLoc33588::CheckLeftLoc335AA => {
            match drips_left_bounds_gate_loc_335aa(x_pos_word) {
                DripsLeftBoundsNextLoc335AA::ContactLoc335D4 => return true,
                DripsLeftBoundsNextLoc335AA::CheckVerticalLoc335B0 => {
                    drips_vertical_bounds_contact_flag_loc_335b0(
                        y_pos_word,
                        projectile.y_velocity,
                        level_height_pixels as i16,
                    )
                }
            }
        }
        DripsRightBoundsNextLoc33588::CheckVerticalLoc335B0 => {
            drips_vertical_bounds_contact_flag_loc_335b0(
                y_pos_word,
                projectile.y_velocity,
                level_height_pixels as i16,
            )
        }
    };
    vertical_contact != 0
}

fn drips_surface_crawler_supported_below_loc_33650(
    grid: &terrain::BlockGrid,
    projectile: &EnemyProjectile,
) -> bool {
    let x_pos_word = (projectile.x_pos >> 16) as u16 as i16;
    let y_pos_word = (projectile.y_pos >> 16) as u16 as i16;
    // ASM Drips.asm loc_33650: `moveq #0,d1; moveq #8,d0; bsr.w loc_331D2;
    // cmpi.w #$6000,d0; bne.w loc_338F8`. The crawler only stays on the
    // horizontal path while the cell 8px below the object reports solid.
    relative_collision_mask_loc_331D2(grid, i32::from(x_pos_word), i32::from(y_pos_word), 0, 8)
        == 0x6000
}

fn drips_surface_crawler_vertical_step_loc_33730(
    grid: &terrain::BlockGrid,
    projectile: &EnemyProjectile,
    level_bounds: LevelBounds,
) -> DripsSurfaceCrawlerVerticalStepLoc33730 {
    let x_pos_word = (projectile.x_pos >> 16) as u16 as i16;
    let y_pos_word = (projectile.y_pos >> 16) as u16 as i16;
    // ASM Drips.asm loc_33730: the vertical crawl label probes 8px above the
    // object through loc_331D2 before checking level bounds and loc_33286.
    if relative_collision_mask_loc_331D2(grid, i32::from(x_pos_word), i32::from(y_pos_word), 0, -8)
        != 0x6000
    {
        return DripsSurfaceCrawlerVerticalStepLoc33730::Exit;
    }
    if drips_surface_crawler_out_of_level_bounds_loc_33588(
        projectile,
        level_bounds.width_pixels,
        level_bounds.height_pixels,
    ) {
        return DripsSurfaceCrawlerVerticalStepLoc33730::Exit;
    }

    let y_offset = if projectile.y_velocity < 0 { -6 } else { 6 };
    let row = relative_collision_row_loc_33286(
        grid,
        i32::from(x_pos_word),
        i32::from(y_pos_word),
        y_offset,
        0,
    );

    if projectile.y_velocity < 0 {
        if row.centre == 0x6000 {
            DripsSurfaceCrawlerVerticalStepLoc33730::Turn(
                drips_surface_crawler_turn_script_loc_33964(),
            )
        } else if row.left == 0x6000 {
            DripsSurfaceCrawlerVerticalStepLoc33730::Loop
        } else if row.left == 0x5000 {
            DripsSurfaceCrawlerVerticalStepLoc33730::Turn(
                drips_surface_crawler_turn_script_loc_339fa(),
            )
        } else {
            DripsSurfaceCrawlerVerticalStepLoc33730::Turn(
                drips_surface_crawler_turn_script_loc_339d2(),
            )
        }
    } else if row.centre == 0x6000 {
        DripsSurfaceCrawlerVerticalStepLoc33730::Turn(drips_surface_crawler_turn_script_loc_33946())
    } else if row.centre == 0x5000 {
        DripsSurfaceCrawlerVerticalStepLoc33730::Turn(drips_surface_crawler_turn_script_loc_33a72())
    } else if row.left == 0x6000 {
        DripsSurfaceCrawlerVerticalStepLoc33730::Loop
    } else {
        DripsSurfaceCrawlerVerticalStepLoc33730::Turn(drips_surface_crawler_turn_script_loc_339a0())
    }
}

fn drips_surface_crawler_vertical_step_loc_336d0(
    grid: &terrain::BlockGrid,
    projectile: &EnemyProjectile,
    level_bounds: LevelBounds,
) -> DripsSurfaceCrawlerVerticalStepLoc336D0 {
    let x_pos_word = (projectile.x_pos >> 16) as u16 as i16;
    let y_pos_word = (projectile.y_pos >> 16) as u16 as i16;
    // ASM Drips.asm loc_336D0: the crawler must have support 8px to the left
    // before the side-column probe and bounds gate may keep it alive.
    if relative_collision_mask_loc_331D2(grid, i32::from(x_pos_word), i32::from(y_pos_word), -8, 0)
        != 0x6000
    {
        return DripsSurfaceCrawlerVerticalStepLoc336D0::Exit;
    }
    if drips_surface_crawler_out_of_level_bounds_loc_33588(
        projectile,
        level_bounds.width_pixels,
        level_bounds.height_pixels,
    ) {
        return DripsSurfaceCrawlerVerticalStepLoc336D0::Exit;
    }

    let x_offset = if projectile.drips_surface_crawler_x_direction_flipped {
        6
    } else {
        -6
    };
    let column = relative_collision_column_loc_331F6(
        grid,
        i32::from(x_pos_word),
        i32::from(y_pos_word),
        x_offset,
        0,
    );

    if projectile.drips_surface_crawler_x_direction_flipped {
        if column.current_row == 0x6000 {
            DripsSurfaceCrawlerVerticalStepLoc336D0::Turn(
                drips_surface_crawler_turn_script_loc_339b4(),
            )
        } else if column.one_row_up == 0x6000 {
            DripsSurfaceCrawlerVerticalStepLoc336D0::Loop
        } else {
            DripsSurfaceCrawlerVerticalStepLoc336D0::Turn(
                drips_surface_crawler_turn_script_loc_339aa(),
            )
        }
    } else if column.current_row == 0x6000 {
        DripsSurfaceCrawlerVerticalStepLoc336D0::Loop
    } else if column.one_row_up == 0x6000 {
        DripsSurfaceCrawlerVerticalStepLoc336D0::Turn(drips_surface_crawler_turn_script_loc_3396e())
    } else {
        DripsSurfaceCrawlerVerticalStepLoc336D0::Turn(drips_surface_crawler_turn_script_loc_33950())
    }
}

fn drips_surface_crawler_vertical_step_loc_337a0(
    grid: &terrain::BlockGrid,
    projectile: &EnemyProjectile,
    level_bounds: LevelBounds,
) -> DripsSurfaceCrawlerVerticalStepLoc33730 {
    let x_pos_word = (projectile.x_pos >> 16) as u16 as i16;
    let y_pos_word = (projectile.y_pos >> 16) as u16 as i16;
    // ASM Drips.asm loc_337A0: this mirrored vertical crawl label probes 8px
    // below through loc_331D2, then routes loc_33286 d5/d6 outcomes.
    if relative_collision_mask_loc_331D2(grid, i32::from(x_pos_word), i32::from(y_pos_word), 0, 8)
        != 0x6000
    {
        return DripsSurfaceCrawlerVerticalStepLoc33730::Exit;
    }
    if drips_surface_crawler_out_of_level_bounds_loc_33588(
        projectile,
        level_bounds.width_pixels,
        level_bounds.height_pixels,
    ) {
        return DripsSurfaceCrawlerVerticalStepLoc33730::Exit;
    }

    let y_offset = if projectile.y_velocity < 0 { -6 } else { 6 };
    let row = relative_collision_row_loc_33286(
        grid,
        i32::from(x_pos_word),
        i32::from(y_pos_word),
        y_offset,
        0,
    );

    if projectile.y_velocity < 0 {
        if row.left == 0x6000 {
            DripsSurfaceCrawlerVerticalStepLoc33730::Turn(
                drips_surface_crawler_turn_script_loc_339be(),
            )
        } else if row.centre == 0x6000 {
            DripsSurfaceCrawlerVerticalStepLoc33730::Loop
        } else if row.centre == 0x4000 {
            DripsSurfaceCrawlerVerticalStepLoc33730::Turn(
                drips_surface_crawler_turn_script_loc_33a04(),
            )
        } else {
            DripsSurfaceCrawlerVerticalStepLoc33730::Turn(
                drips_surface_crawler_turn_script_loc_33978(),
            )
        }
    } else if row.left == 0x6000 {
        DripsSurfaceCrawlerVerticalStepLoc33730::Turn(drips_surface_crawler_turn_script_loc_3398c())
    } else if row.left == 0x4000 {
        DripsSurfaceCrawlerVerticalStepLoc33730::Turn(drips_surface_crawler_turn_script_loc_33a5e())
    } else if row.centre == 0x6000 {
        DripsSurfaceCrawlerVerticalStepLoc33730::Loop
    } else {
        DripsSurfaceCrawlerVerticalStepLoc33730::Turn(drips_surface_crawler_turn_script_loc_3395a())
    }
}

fn drips_surface_crawler_diagonal_step_loc_33810(
    grid: &terrain::BlockGrid,
    projectile: &EnemyProjectile,
    level_bounds: LevelBounds,
) -> DripsSurfaceCrawlerDiagonalStepLoc33810 {
    let x_pos_word = (projectile.x_pos >> 16) as u16 as i16;
    let y_pos_word = (projectile.y_pos >> 16) as u16 as i16;
    // ASM Drips.asm loc_33810: the diagonal crawler leaves this label through
    // loc_338F8 on level-edge contact, otherwise it waits until the low nibbles
    // of X/Y sum to `$F` before running the `loc_3330E` corner probe.
    if drips_surface_crawler_out_of_level_bounds_loc_33588(
        projectile,
        level_bounds.width_pixels,
        level_bounds.height_pixels,
    ) {
        return DripsSurfaceCrawlerDiagonalStepLoc33810::Exit;
    }

    if (((projectile.x_pos >> 16) as u16) & 0x000F) + (((projectile.y_pos >> 16) as u16) & 0x000F)
        != 0x000F
    {
        return DripsSurfaceCrawlerDiagonalStepLoc33810::Loop;
    }

    let (x_offset, y_offset) = if projectile.y_velocity < 0 {
        (5, -5)
    } else {
        (-5, 5)
    };
    let tile_flags = relative_collision_mask_loc_3330E(
        grid,
        i32::from(x_pos_word),
        i32::from(y_pos_word),
        x_offset,
        y_offset,
    );

    if projectile.y_velocity < 0 {
        if tile_flags == 0x4000 {
            DripsSurfaceCrawlerDiagonalStepLoc33810::Loop
        } else if tile_flags == 0x6000 {
            DripsSurfaceCrawlerDiagonalStepLoc33810::Turn(
                drips_surface_crawler_turn_script_loc_33a54(),
            )
        } else {
            DripsSurfaceCrawlerDiagonalStepLoc33810::Turn(
                drips_surface_crawler_turn_script_loc_339e6(),
            )
        }
    } else if tile_flags == 0x4000 {
        DripsSurfaceCrawlerDiagonalStepLoc33810::Loop
    } else if tile_flags == 0x6000 {
        DripsSurfaceCrawlerDiagonalStepLoc33810::Turn(drips_surface_crawler_turn_script_loc_33a36())
    } else {
        DripsSurfaceCrawlerDiagonalStepLoc33810::Turn(drips_surface_crawler_turn_script_loc_33a0e())
    }
}

fn drips_surface_crawler_diagonal_step_loc_33892(
    grid: &terrain::BlockGrid,
    projectile: &EnemyProjectile,
    level_bounds: LevelBounds,
) -> DripsSurfaceCrawlerDiagonalStepLoc33810 {
    let x_pos_word = (projectile.x_pos >> 16) as u16 as i16;
    let y_pos_word = (projectile.y_pos >> 16) as u16 as i16;
    // ASM Drips.asm loc_33892/loc_338CE: after the same level-edge guard, the
    // diagonal crawler probes `loc_3330E` with mirrored offsets and loops only
    // while d7 is `$5000`.
    if drips_surface_crawler_out_of_level_bounds_loc_33588(
        projectile,
        level_bounds.width_pixels,
        level_bounds.height_pixels,
    ) {
        return DripsSurfaceCrawlerDiagonalStepLoc33810::Exit;
    }

    let (x_offset, y_offset) = if projectile.y_velocity < 0 {
        (-5, -5)
    } else {
        (5, 5)
    };
    let tile_flags = relative_collision_mask_loc_3330E(
        grid,
        i32::from(x_pos_word),
        i32::from(y_pos_word),
        x_offset,
        y_offset,
    );

    if projectile.y_velocity < 0 {
        if tile_flags == 0x5000 {
            DripsSurfaceCrawlerDiagonalStepLoc33810::Loop
        } else if tile_flags == 0x6000 {
            DripsSurfaceCrawlerDiagonalStepLoc33810::Turn(
                drips_surface_crawler_turn_script_loc_33a68(),
            )
        } else {
            DripsSurfaceCrawlerDiagonalStepLoc33810::Turn(
                drips_surface_crawler_turn_script_loc_33a4a(),
            )
        }
    } else if tile_flags == 0x5000 {
        DripsSurfaceCrawlerDiagonalStepLoc33810::Loop
    } else if tile_flags == 0x6000 {
        DripsSurfaceCrawlerDiagonalStepLoc33810::Turn(drips_surface_crawler_turn_script_loc_33a22())
    } else {
        DripsSurfaceCrawlerDiagonalStepLoc33810::Turn(drips_surface_crawler_turn_script_loc_339f0())
    }
}

fn drips_surface_crawler_side_column_step_loc_33650(
    grid: &terrain::BlockGrid,
    projectile: &EnemyProjectile,
) -> DripsSurfaceCrawlerSideColumnStepLoc33650 {
    let x_pos_word = (projectile.x_pos >> 16) as u16 as i16;
    let y_pos_word = (projectile.y_pos >> 16) as u16 as i16;
    let x_offset = if projectile.x_velocity < 0 { -6 } else { 6 };
    // ASM Drips.asm loc_33650/loc_336AA: after the support and bounds gates,
    // `loc_331F6` samples the side column. Native runs the direct horizontal
    // turn-script targets; later vertical/diagonal movement states still use
    // the simplified carrier once the script jumps onward.
    let column = relative_collision_column_loc_331F6(
        grid,
        i32::from(x_pos_word),
        i32::from(y_pos_word),
        x_offset,
        0,
    );

    if projectile.x_velocity < 0 {
        if column.one_row_up == 0x6000 {
            return DripsSurfaceCrawlerSideColumnStepLoc33650::Turn(
                drips_surface_crawler_turn_script_loc_3393c(),
            );
        }
        if column.one_row_up == 0x5000 {
            return DripsSurfaceCrawlerSideColumnStepLoc33650::Turn(
                drips_surface_crawler_turn_script_loc_33a40(),
            );
        }
        if column.current_row == 0x4000 {
            return DripsSurfaceCrawlerSideColumnStepLoc33650::Turn(
                drips_surface_crawler_turn_script_loc_33a2c(),
            );
        }
        if column.current_row == 0x6000 {
            return DripsSurfaceCrawlerSideColumnStepLoc33650::Loop;
        }
        return DripsSurfaceCrawlerSideColumnStepLoc33650::Turn(
            drips_surface_crawler_turn_script_loc_33982(),
        );
    }

    if column.one_row_up == 0x4000 {
        return DripsSurfaceCrawlerSideColumnStepLoc33650::Turn(
            drips_surface_crawler_turn_script_loc_339dc(),
        );
    }
    if column.one_row_up == 0x6000 {
        return DripsSurfaceCrawlerSideColumnStepLoc33650::Turn(
            drips_surface_crawler_turn_script_loc_33996(),
        );
    }
    if column.current_row == 0x6000 {
        return DripsSurfaceCrawlerSideColumnStepLoc33650::Loop;
    }
    if column.current_row == 0x5000 {
        return DripsSurfaceCrawlerSideColumnStepLoc33650::Turn(
            drips_surface_crawler_turn_script_loc_33a18(),
        );
    }
    DripsSurfaceCrawlerSideColumnStepLoc33650::Turn(drips_surface_crawler_turn_script_loc_339c8())
}

fn drips_surface_crawler_add_word_delta_loc_33af6(pos: u32, delta: i16) -> u32 {
    let pos_word = (pos >> 16) as u16;
    u32::from(pos_word.wrapping_add(delta as u16)) << 16
}

fn drips_surface_crawler_apply_turn_script_loc_33af6(
    projectile: &mut EnemyProjectile,
    script: DripsSurfaceCrawlerTurnScriptLoc33AF6,
) {
    // ASM Drips.asm loc_33AF6 clears velocity/fraction words before snapping
    // the selected position word and running the two animation movement tuples.
    projectile.x_velocity = 0;
    projectile.y_velocity = 0;
    projectile.x_pos &= 0xFFFF_0000;
    projectile.y_pos &= 0xFFFF_0000;

    match script.snap_field {
        DripsSurfaceCrawlerTurnFieldLoc33AF6::XPos => {
            let snapped = ((projectile.x_pos >> 16) as u16).wrapping_add(script.snap_offset as u16)
                & script.snap_mask;
            projectile.x_pos = u32::from(snapped) << 16;
        }
        DripsSurfaceCrawlerTurnFieldLoc33AF6::YPos => {
            let snapped = ((projectile.y_pos >> 16) as u16).wrapping_add(script.snap_offset as u16)
                & script.snap_mask;
            projectile.y_pos = u32::from(snapped) << 16;
        }
        DripsSurfaceCrawlerTurnFieldLoc33AF6::Unknown(_) => {}
    }

    projectile.x_pos =
        drips_surface_crawler_add_word_delta_loc_33af6(projectile.x_pos, script.first_x_delta);
    projectile.y_pos =
        drips_surface_crawler_add_word_delta_loc_33af6(projectile.y_pos, script.first_y_delta);
    projectile.x_pos =
        drips_surface_crawler_add_word_delta_loc_33af6(projectile.x_pos, script.second_x_delta);
    projectile.y_pos =
        drips_surface_crawler_add_word_delta_loc_33af6(projectile.y_pos, script.second_y_delta);
    projectile.x_velocity = projectile.x_velocity.wrapping_add(script.x_velocity_delta);
    projectile.y_velocity = projectile.y_velocity.wrapping_add(script.y_velocity_delta);
    if script.flip_before_first_animation {
        projectile.drips_surface_crawler_x_direction_flipped =
            !projectile.drips_surface_crawler_x_direction_flipped;
    }
    if script.flip_after_first_animation {
        projectile.drips_surface_crawler_x_direction_flipped =
            !projectile.drips_surface_crawler_x_direction_flipped;
    }
    projectile.drips_surface_crawler_state = match script.next {
        DripsSurfaceCrawlerTurnNextLoc33AF6::Loc336D0 => DripsSurfaceCrawlerState::VerticalLoc336D0,
        DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33730 => DripsSurfaceCrawlerState::VerticalLoc33730,
        DripsSurfaceCrawlerTurnNextLoc33AF6::Loc337A0 => DripsSurfaceCrawlerState::VerticalLoc337A0,
        DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33A7C => DripsSurfaceCrawlerState::IdleLoc33A7C,
        DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33810 => DripsSurfaceCrawlerState::DiagonalLoc33810,
        DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33892 => DripsSurfaceCrawlerState::DiagonalLoc33892,
        DripsSurfaceCrawlerTurnNextLoc33AF6::Loc33650
        | DripsSurfaceCrawlerTurnNextLoc33AF6::Unknown(_) => {
            DripsSurfaceCrawlerState::HorizontalLoc33650
        }
    };
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidProjectileTargetXSpanLoc1DB4 {
    pub(crate) left_px: i16,
    pub(crate) right_px: i16,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidProjectileTargetRectLoc1DBE {
    pub(crate) left_px: i16,
    pub(crate) right_px: i16,
    pub(crate) top_px: i16,
    pub(crate) bottom_px: i16,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KidProjectileCollisionObjectLoc1DCC {
    pub(crate) deleted: bool,
    pub(crate) mirrored: bool,
    pub(crate) collision_index_word: u16,
    pub(crate) x_anchor_px: i16,
    pub(crate) y_anchor_px: i16,
    pub(crate) collision_sizes: Vec<CollisionSizeEntrySub21F8>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KidProjectileCollisionCursorLoc1DCC {
    pub(crate) object_index: usize,
    pub(crate) collision_size_index: usize,
    pub(crate) mirrored: bool,
    pub(crate) x_anchor_px: i16,
    pub(crate) y_anchor_px: i16,
    pub(crate) collision_sizes: Vec<CollisionSizeEntrySub21F8>,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn kid_projectile_target_x_span_loc_1DB4(
    x_pos: i16,
    x_direction: u8,
    x_leading_offset: i16,
    x_span: i16,
) -> KidProjectileTargetXSpanLoc1DB4 {
    // ASM sub_1D84 (kid.asm:3560-3567): when `x_direction(a0)` is clear the
    // pre-`loc_1DB4` path does `d0 = x_pos + leading_offset`, copies that to
    // `d1`, then adds the span so the target runs `[d0, d0 + span]`.
    //
    // ASM loc_1DB4 (kid.asm:3568-3572): when `x_direction(a0)` is non-zero the
    // routine mirrors that X interval by negating the leading offset, adding it
    // to `x_pos`, copying the result to `d1`, then subtracting the span so the
    // target runs `[d1 - span, d1]`.
    if x_direction == 0 {
        let left_px = x_pos.wrapping_add(x_leading_offset);
        KidProjectileTargetXSpanLoc1DB4 {
            left_px,
            right_px: left_px.wrapping_add(x_span),
        }
    } else {
        let right_px = x_pos.wrapping_sub(x_leading_offset);
        KidProjectileTargetXSpanLoc1DB4 {
            left_px: right_px.wrapping_sub(x_span),
            right_px,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn kid_projectile_target_rect_loc_1DBE(
    x_pos: i16,
    y_pos: i16,
    x_direction: u8,
    x_leading_offset: i16,
    x_span: i16,
    y_offset: i16,
    y_span: i16,
) -> KidProjectileTargetRectLoc1DBE {
    let x_span =
        kid_projectile_target_x_span_loc_1DB4(x_pos, x_direction, x_leading_offset, x_span);
    // ASM loc_1DBE (kid.asm:3574-3578): after either X-span path, load the
    // leading Y offset into `d2`, add `y_pos(a0)`, copy that to `d3`, then add
    // the final span so the shared target rectangle runs `[d2, d2 + span]`.
    let top_px = y_pos.wrapping_add(y_offset);
    KidProjectileTargetRectLoc1DBE {
        left_px: x_span.left_px,
        right_px: x_span.right_px,
        top_px,
        bottom_px: top_px.wrapping_add(y_span),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn first_live_kid_projectile_collision_cursor_loc_1DCC(
    objects: &[KidProjectileCollisionObjectLoc1DCC],
) -> Option<KidProjectileCollisionCursorLoc1DCC> {
    for (object_index, object) in objects.iter().enumerate() {
        // ASM loc_1DCC (kid.asm:3581-3589): follow the gfx-object list, skip
        // deleted entries via `tst.b $3D(a2)`, then load `$22(a2)`, halve it,
        // index `CollisionSize_Index`, and back the cursor up by one 8-byte
        // slot so `loc_1DF2`/`loc_1E1E` can start with `addq.w #8,a3`.
        if object.deleted {
            continue;
        }

        return Some(KidProjectileCollisionCursorLoc1DCC {
            object_index,
            collision_size_index: usize::from(object.collision_index_word >> 1),
            mirrored: object.mirrored,
            x_anchor_px: object.x_anchor_px,
            y_anchor_px: object.y_anchor_px,
            collision_sizes: object.collision_sizes.clone(),
        });
    }

    None
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn first_non_mirrored_kid_projectile_overlap_loc_1DF2(
    target: KidProjectileTargetRectLoc1DBE,
    cursor: &KidProjectileCollisionCursorLoc1DCC,
) -> Option<usize> {
    // ASM loc_1DF2 (kid.asm:3595-3608): after `loc_1DCC` has selected a live
    // object with clear `x_direction(a2)`, advance one 8-byte CollisionSize
    // entry at a time. A zero first word aborts this object, otherwise the
    // scan uses inclusive overlap checks and returns on the first hit.
    if cursor.mirrored {
        return None;
    }

    for (collision_size_index, collision_size) in cursor.collision_sizes.iter().enumerate() {
        if collision_size.left_offset_px == 0 {
            return None;
        }

        let left_px = cursor
            .x_anchor_px
            .saturating_add(collision_size.left_offset_px);
        if left_px > target.right_px {
            continue;
        }

        let right_px = left_px.saturating_add(collision_size.width_px);
        if right_px < target.left_px {
            continue;
        }

        let top_px = cursor
            .y_anchor_px
            .saturating_add(collision_size.top_offset_px);
        if top_px > target.bottom_px {
            continue;
        }

        let bottom_px = top_px.saturating_add(collision_size.height_px);
        if bottom_px < target.top_px {
            continue;
        }

        return Some(collision_size_index);
    }

    None
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn first_mirrored_kid_projectile_overlap_loc_1E1E(
    target: KidProjectileTargetRectLoc1DBE,
    cursor: &KidProjectileCollisionCursorLoc1DCC,
) -> Option<usize> {
    // ASM loc_1E1E (kid.asm:3615-3629): for set `x_direction(a2)`, advance one
    // 8-byte CollisionSize entry at a time, mirror the horizontal interval by
    // negating the stored left offset, and subtract the width back from that
    // anchor before applying the same inclusive overlap tests.
    if !cursor.mirrored {
        return None;
    }

    for (collision_size_index, collision_size) in cursor.collision_sizes.iter().enumerate() {
        if collision_size.left_offset_px == 0 {
            return None;
        }

        let right_px = cursor
            .x_anchor_px
            .saturating_sub(collision_size.left_offset_px);
        if right_px < target.left_px {
            continue;
        }

        let left_px = right_px.saturating_sub(collision_size.width_px);
        if left_px > target.right_px {
            continue;
        }

        let top_px = cursor
            .y_anchor_px
            .saturating_add(collision_size.top_offset_px);
        if top_px > target.bottom_px {
            continue;
        }

        let bottom_px = top_px.saturating_add(collision_size.height_px);
        if bottom_px < target.top_px {
            continue;
        }

        return Some(collision_size_index);
    }

    None
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KidProjectileOverlapBranchLoc1E4A {
    SpawnHelperLoc1E5E,
    WallResponseLoc1EA0,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn kid_projectile_overlap_branch_loc_1E4A(
    collision_mode_d7: u16,
    collision_size_word_22: u16,
    lnk_to3_null_minus_data_index: u16,
    lnk_to4_null_minus_data_index: u16,
) -> KidProjectileOverlapBranchLoc1E4A {
    // ASM loc_1E4A (kid.asm:3634-3639): `d7 != 0` jumps straight to
    // `loc_1EA0`. In the `d7 == 0` pass, only the inclusive
    // `[LnkTo3_NULL-Data_Index, LnkTo4_NULL-Data_Index]` window diverts to
    // `loc_1EA0`; words below or above that range keep falling through to
    // `loc_1E5E`.
    if collision_mode_d7 != 0 {
        return KidProjectileOverlapBranchLoc1E4A::WallResponseLoc1EA0;
    }

    if collision_size_word_22 >= lnk_to3_null_minus_data_index
        && collision_size_word_22 <= lnk_to4_null_minus_data_index
    {
        KidProjectileOverlapBranchLoc1E4A::WallResponseLoc1EA0
    } else {
        KidProjectileOverlapBranchLoc1E4A::SpawnHelperLoc1E5E
    }
}

#[allow(non_snake_case)]
pub(crate) fn kid_projectile_dp_collision_word_loc_1EA0(object_meta_low_byte_3a: u8) -> u16 {
    // ASM loc_1EA0 (kid.asm:3665): the Diamond Power pass swaps d7, reads
    // `$3A(a2) & $00FF`, doubles it, and indexes `unk_1EBE` for the collision
    // word written back to the collided object at `$38(a2)`.
    const DP_COLLISION_WORDS_UNK_1EBE: [u16; 37] = [
        0x001C, 0x001C, 0x0000, 0x002C, 0x001C, 0x001C, 0x001C, 0x001C, 0x002C, 0x002C, 0x002C,
        0x002C, 0x001C, 0x002C, 0x001C, 0x001C, 0x002C, 0x002C, 0x002C, 0x002C, 0x002C, 0x001C,
        0x001C, 0x001C, 0x001C, 0x001C, 0x002C, 0x001C, 0x002C, 0x001C, 0x001C, 0x001C, 0xFFFF,
        0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    ];

    DP_COLLISION_WORDS_UNK_1EBE
        .get(usize::from(object_meta_low_byte_3a))
        .copied()
        .unwrap_or(0xFFFF)
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidProjectileHelperSpawnLoc1E5E {
    pub(crate) allocate_object_slot_arg: u16,
    pub(crate) callback_label: &'static str,
    pub(crate) helper_x_pos: u16,
    pub(crate) helper_y_pos: u16,
    pub(crate) target_collision_id: u16,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn kid_projectile_helper_spawn_loc_1E5E(
    projectile_x_pos: u16,
    projectile_y_pos: u16,
    projectile_collision_flags_word_3a: u16,
) -> KidProjectileHelperSpawnLoc1E5E {
    // ASM loc_1E5E (kid.asm:3642-3653): exchange `a0/a1`, call
    // `j_Allocate_ObjectSlot` with `a0 = $FFFF`, install `loc_1F08`, then
    // restore the projectile pointer and copy its `$1A/$1E` coordinates into
    // the helper's `$44/$46`. The collided object usually gets `colid_hurt`,
    // but the `$3A(a0) == $3C` special case overwrites that with `$FFFF`.
    KidProjectileHelperSpawnLoc1E5E {
        allocate_object_slot_arg: 0xFFFF,
        callback_label: "loc_1F08",
        helper_x_pos: projectile_x_pos,
        helper_y_pos: projectile_y_pos,
        target_collision_id: if projectile_collision_flags_word_3a == 0x003C {
            0xFFFF
        } else {
            0x001C
        },
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidProjectileRightwallWritebackLoc1E94 {
    pub(crate) projectile_collision_id: u16,
    pub(crate) next_label: &'static str,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn kid_projectile_rightwall_writeback_loc_1E94() -> KidProjectileRightwallWritebackLoc1E94
{
    // ASM loc_1E94 (kid.asm:3655-3656): write `#4` into `$38(a0)` and branch
    // back into `sub_1D84` for the next collision scan step.
    KidProjectileRightwallWritebackLoc1E94 {
        projectile_collision_id: 0x0004,
        next_label: "sub_1D84",
    }
}

impl NativeRuntime {
    #[allow(non_snake_case)]
    pub(crate) fn process_new_shooter_objects_loc_EC22(&mut self, start_index: usize) {
        let mut index = start_index;
        let mut processed = 0usize;
        while index < self.projectiles.len() && processed < 0x27 {
            processed += 1;
            let should_process = self.projectiles[index].kind == EnemyProjectileKind::ShooterBlock
                && self.projectiles[index].shooter_tracks_layout;
            if !should_process {
                index += 1;
                continue;
            }

            let outcome = {
                let Some(grid) = self.block_grid.as_ref() else {
                    return;
                };
                Self::step_tracked_shooter_object_loc_EDFA(grid, &mut self.projectiles[index])
            };

            match outcome {
                Ok(()) => {
                    // ASM loc_EC76 falls straight into Process_ShooterObjects
                    // (loc_EC22/loc_EDFA). In native's split frame shell,
                    // `step_projectiles` runs later, so mark this slot to
                    // avoid applying a second loc_EDFA step in the same frame.
                    self.projectiles[index].shooter_skip_projectile_step_frame =
                        Some(self.frame_index);
                    index += 1;
                }
                Err(interaction) => {
                    self.projectiles.remove(index);
                    self.apply_shooter_object_layout_interaction_loc_F00A(interaction);
                }
            }
        }
    }

    fn is_juggernaut_skull_projectile(projectile: &EnemyProjectile) -> bool {
        projectile.friendly
            && projectile.gravity == JUGGERNAUT_SKULL_GRAVITY
            && projectile.homing_accel == 0
            && projectile.speed_cap == 0
            && !projectile.terrain_kill
    }

    fn juggernaut_skull_solid_probe_loc_b55c(
        grid: &terrain::BlockGrid,
        x_px: i32,
        y_px: i32,
    ) -> bool {
        // ASM sub_B55C callers in sub_87B0 compare `(tileword & $7000)` with
        // `$6000`; slopes, spikes and hidden blocks are not skull walls.
        terrain::probe_relative_collision_sub_B55C(grid, x_px, y_px, 0, 0)
            == terrain::CollisionKind::Solid
    }

    #[allow(non_snake_case)]
    fn shooter_object_direction_from_velocity_loc_EDFA(projectile: &EnemyProjectile) -> u16 {
        if projectile.y_velocity < 0 {
            0
        } else if projectile.x_velocity > 0 {
            1
        } else if projectile.y_velocity > 0 {
            2
        } else {
            3
        }
    }

    #[allow(non_snake_case)]
    fn shooter_object_entered_layout_cell_loc_EDFA(projectile: &EnemyProjectile) -> bool {
        let direction = Self::shooter_object_direction_from_velocity_loc_EDFA(projectile);
        let x_low = ((projectile.x_pos >> 16) & 0x0F) as u8;
        let y_low = ((projectile.y_pos >> 16) & 0x0F) as u8;
        match direction {
            0 => y_low >= 12,
            1 => x_low <= 3,
            2 => y_low <= 3,
            _ => x_low >= 12,
        }
    }

    #[allow(non_snake_case)]
    fn shooter_object_layout_interaction_loc_F00A(
        grid: &terrain::BlockGrid,
        projectile: &EnemyProjectile,
    ) -> Option<ShooterObjectLayoutInteractionLocF00A> {
        if !Self::shooter_object_entered_layout_cell_loc_EDFA(projectile) {
            return None;
        }

        let col = terrain::pixel_to_cell((projectile.x_pos >> 16) as i32).max(0) as u16;
        let row = terrain::pixel_to_cell((projectile.y_pos >> 16) as i32).max(0) as u16;
        let cell = grid.cell_at(col, row);
        if !cell.has_block {
            return None;
        }

        let direction = Self::shooter_object_direction_from_velocity_loc_EDFA(projectile);
        Some(ShooterObjectLayoutInteractionLocF00A {
            col,
            row,
            block_type: cell.block_type,
            skin: cell.skin,
            // ASM ShooterObject_InteractLevelLayout loc_F00A: `eori.w #2,d6`
            // remaps the projectile travel direction before dispatching
            // through Block_LoadSmashBits / IceBlock_SmashAndLoadBullets.
            destroy_direction: direction ^ 2,
        })
    }

    #[allow(non_snake_case)]
    fn shooter_layout_word_loc_EE38(cell: terrain::Cell) -> u16 {
        let block_flag = if cell.has_block { 0x8000 } else { 0 };
        let collision_bits = (cell.collision as u16) << 12;
        let block_type_bits = if cell.has_block {
            (cell.block_type as u16) << 8
        } else {
            0
        };
        block_flag | collision_bits | block_type_bits | u16::from(cell.skin)
    }

    #[allow(non_snake_case)]
    fn shooter_layout_interaction_at_cell_loc_F00A(
        grid: &terrain::BlockGrid,
        col: u16,
        row: u16,
        direction: u16,
    ) -> ShooterObjectLayoutInteractionLocF00A {
        let cell = grid.cell_at(col, row);
        ShooterObjectLayoutInteractionLocF00A {
            col,
            row,
            block_type: cell.block_type,
            skin: cell.skin,
            // ASM ShooterObject_InteractLevelLayout loc_F00A: `eori.w #2,d6`.
            destroy_direction: direction ^ 2,
        }
    }

    #[allow(non_snake_case)]
    fn step_tracked_shooter_object_loc_EDFA(
        grid: &terrain::BlockGrid,
        projectile: &mut EnemyProjectile,
    ) -> Result<(), ShooterObjectLayoutInteractionLocF00A> {
        let direction = Self::shooter_object_direction_from_velocity_loc_EDFA(projectile);
        let age_after_increment = projectile.shooter_state_word.wrapping_add(1);
        let mut state_word = age_after_increment;
        let mut layout_col = projectile.shooter_layout_col;
        let mut layout_row = projectile.shooter_layout_row;
        let mut check_collision = true;

        match direction {
            0 => {
                if (projectile.y_pos >> 16) <= 4 {
                    return Err(Self::shooter_layout_interaction_at_cell_loc_F00A(
                        grid, layout_col, layout_row, direction,
                    ));
                }
                if age_after_increment == 0 {
                    layout_row = layout_row.saturating_sub(1);
                    check_collision = false;
                } else if age_after_increment == 0xFFFF {
                    check_collision = false;
                } else if age_after_increment == 4 {
                    state_word = 0;
                    layout_row = layout_row.saturating_sub(1);
                }
            }
            1 => {
                if ((projectile.x_pos >> 16) as i32 + 4) >= grid.width_pixels() {
                    return Err(Self::shooter_layout_interaction_at_cell_loc_F00A(
                        grid, layout_col, layout_row, direction,
                    ));
                }
                if age_after_increment == 0 {
                    layout_col = layout_col.saturating_add(1);
                    check_collision = false;
                } else if age_after_increment == 0xFFFF {
                    check_collision = false;
                } else if age_after_increment == 4 {
                    state_word = 0;
                    layout_col = layout_col.saturating_add(1);
                }
            }
            2 => {
                if ((projectile.y_pos >> 16) as i32 + 4) >= grid.height_pixels() {
                    return Err(Self::shooter_layout_interaction_at_cell_loc_F00A(
                        grid, layout_col, layout_row, direction,
                    ));
                }
                if age_after_increment == 0 {
                    layout_row = layout_row.saturating_add(1);
                    check_collision = false;
                } else if age_after_increment == 0xFFFF {
                    check_collision = false;
                } else if age_after_increment == 4 {
                    state_word = 0;
                    layout_row = layout_row.saturating_add(1);
                }
            }
            _ => {
                if (projectile.x_pos >> 16) <= 4 {
                    return Err(Self::shooter_layout_interaction_at_cell_loc_F00A(
                        grid, layout_col, layout_row, direction,
                    ));
                }
                if age_after_increment == 0 {
                    layout_col = layout_col.saturating_sub(1);
                    check_collision = false;
                } else if age_after_increment == 0xFFFF {
                    check_collision = false;
                } else if age_after_increment == 4 {
                    state_word = 0;
                    layout_col = layout_col.saturating_sub(1);
                }
            }
        }

        if check_collision {
            let layout_word =
                Self::shooter_layout_word_loc_EE38(grid.cell_at(layout_col, layout_row));
            let interacts = match direction {
                // ASM loc_EE92: moving up interacts with bit-14 cells while
                // state<2, or at older states when the moving-up `$2000` bit remains.
                0 => (layout_word & 0x4000) != 0 && (state_word < 2 || (layout_word & 0x2000) != 0),
                // ASM loc_EE38: moving right ages >=2 interact immediately;
                // ages 0/1 require bits `$3000`.
                1 => {
                    (layout_word & 0x4000) != 0 && (state_word >= 2 || (layout_word & 0x3000) != 0)
                }
                // ASM loc_EF40: moving down matches right except the young-age
                // directional bit is `$2000`.
                2 => {
                    (layout_word & 0x4000) != 0 && (state_word >= 2 || (layout_word & 0x2000) != 0)
                }
                // ASM loc_EEE4: moving left ages >=2 interact immediately;
                // ages 0/1 interact when the `$1000` left bit is clear.
                _ => {
                    (layout_word & 0x4000) != 0 && (state_word >= 2 || (layout_word & 0x1000) == 0)
                }
            };

            if interacts {
                return Err(Self::shooter_layout_interaction_at_cell_loc_F00A(
                    grid, layout_col, layout_row, direction,
                ));
            }
        }

        projectile.shooter_state_word = state_word;
        projectile.shooter_layout_col = layout_col;
        projectile.shooter_layout_row = layout_row;

        match direction {
            0 => projectile.y_pos = projectile.y_pos.wrapping_sub(4 << 16),
            1 => projectile.x_pos = projectile.x_pos.wrapping_add(4 << 16),
            2 => projectile.y_pos = projectile.y_pos.wrapping_add(4 << 16),
            _ => projectile.x_pos = projectile.x_pos.wrapping_sub(4 << 16),
        }

        Ok(())
    }

    #[allow(non_snake_case)]
    fn apply_shooter_object_layout_interaction_loc_F00A(
        &mut self,
        interaction: ShooterObjectLayoutInteractionLocF00A,
    ) {
        match interaction.block_type {
            terrain::BlockType::Smash
            | terrain::BlockType::Prize
            | terrain::BlockType::Mushroom => {
                if let Some(grid) = &mut self.block_grid {
                    grid.set_cell_at(
                        interaction.col,
                        interaction.row,
                        terrain::Cell::from_be_bytes(0, 0),
                    );
                }
                self.spawn_smash_block_effect_sub_10eda(
                    interaction.col * 16,
                    interaction.row * 16,
                    interaction.destroy_direction,
                );
            }
            terrain::BlockType::Ice => {
                if let Some(grid) = &mut self.block_grid {
                    grid.set_cell_at(
                        interaction.col,
                        interaction.row,
                        terrain::Cell::from_be_bytes(0, 0),
                    );
                }
                self.spawn_ice_block_effect_sub_10f9c(
                    interaction.col * 16,
                    interaction.row * 16,
                    interaction.destroy_direction,
                );
            }
            terrain::BlockType::Evanescent => {
                // ASM loc_F04A dispatches shooter-object hits on type-$A
                // blocks through EvanescentBlock_Trigger (`loc_DF68`) but
                // does not branch through LoadBulletsFromEvanescentBlock like
                // Kid's body-contact loc_EB72 path. The slot's bullets load
                // on the next loc_EC22 pass.
                self.trigger_evanescent_block_from_shooter_loc_F04A(
                    interaction.col,
                    interaction.row,
                    interaction.skin,
                );
            }
            _ => {}
        }
    }

    /// ASM loc_8810 (kid.asm:9653): skull projectile main loop.
    /// `jsr Hibernate_Object_1Frame; tst.b $19(a3); bne loc_899C` — checks
    /// animation-done flag (delete if set). `tst.w collision_type(a3); bne loc_899C`
    /// — checks object collision (delete if hit). `addi.l #$1000,y_vel(a3)` applies
    /// gravity each frame. Then horizontal movement with tile-boundary wall detection:
    /// right-moving → loc_8852, left-moving → loc_8890, each probes sub_B55C for
    /// solid terrain and snaps/bounces on contact.
    ///
    /// ASM loc_89A8 (kid.asm:9815): skull explosion/deletion path — reached when
    /// bounce budget exhausted (loc_899C subq.w d0 → 0) or from anim-done/collision.
    /// `sf is_moved(a3); clr.b $12(a3); clr.b palette_line(a3)` — clear visual fields.
    /// `moveq #sfx_Juggernaut_skull_explode,d0; jsr j_PlaySound` — play explosion SFX.
    /// `Init_Animation stru_8BDE; Hibernate_UntilAnimFinished; Delete_CurrentObject`.
    /// Native: `return false` removes from projectile list (sound/animation visual-only).
    /// loc_899C also decrements $FFFFFB70 (live skull count) — native counts active
    /// skulls inline via `projectiles.iter().filter(is_juggernaut_skull_projectile)`.
    #[allow(non_snake_case)]
    fn step_juggernaut_skull_sub_87B0(
        projectile: &mut EnemyProjectile,
        grid: Option<&terrain::BlockGrid>,
    ) -> bool {
        let Some(grid) = grid else {
            projectile.x_pos = (projectile.x_pos as i64 + i64::from(projectile.x_velocity)) as u32;
            projectile.y_pos = (projectile.y_pos as i64 + i64::from(projectile.y_velocity)) as u32;
            return true;
        };

        let original_y = projectile.y_pos;
        let x_velocity = projectile.x_velocity;
        let next_x = (i64::from(projectile.x_pos) + i64::from(x_velocity)) as u32;
        projectile.x_pos = next_x;
        let next_x_px = (next_x >> 16) as i32;
        let current_y_px = (original_y >> 16) as i32;

        // ASM loc_8852 (kid.asm:9674): right-moving tile-boundary wall check.
        // `add.w d5,d7; move.w d7,x_pos(a3)` — moves x_pos first, then
        // `bsr.w sub_B55C; cmpi.w #$6000,d5` probes for solid terrain.
        // If solid: `andi.w #$FFF0,d7; subq.w #1,d7` snaps to tile left edge - 1.
        // ASM loc_8864 (kid.asm:9682): `bne.w loc_88E6` — not-solid branch
        // skips wall snap entirely, falls through to y-axis movement.
        // ASM loc_887A (kid.asm:9691): left-moving pre-boundary check.
        // `andi.w #$F,d6; neg.w d7; cmp.w d6,d7` compares the leftward
        // displacement against the current tile nibble. If the projectile
        // stays within the tile, `sub.w d7,d5; move.w d5,x_pos(a3)` applies
        // the move and falls through to loc_88E6.
        // ASM loc_8890 (kid.asm:9700): left-moving mirror — snaps to
        // `(x & $FFF0) + $10` (tile right edge). Both paths fall through to
        // loc_88B6 bounce logic: `subq.w #1,d0` (bounce counter), x_vel decay
        // by ±1, `neg.w d7; not.b x_direction` reverses direction.
        if Self::juggernaut_skull_solid_probe_loc_b55c(grid, next_x_px, current_y_px) {
            let snapped_x_px = if x_velocity >= 0 {
                (next_x_px & !0xF) - 1
            } else {
                (next_x_px & !0xF) + 16
            };
            projectile.x_pos = (snapped_x_px.max(0) as u32) << 16;
            projectile.reaim_countdown = projectile.reaim_countdown.saturating_sub(1);
            if projectile.reaim_countdown == 0 {
                return false;
            }

            // ASM loc_88D2 (kid.asm:9725): negative x_vel bounce decay.
            // `cmpi.w #$FFFF,d7; beq.w loc_88DC; addq.w #1,d7` — if x_vel
            // is -1 (word), skip decay; otherwise add 1 to reduce magnitude.
            // Positive mirror at loc_88B6: `cmpi.w #1,d7; beq; subq.w #1`.
            // ASM loc_88DC (kid.asm:9730): `neg.w d7; move.w d7,x_vel(a3);
            // not.b x_direction(a3)` — negate decayed velocity, flip facing.
            let decayed_x_velocity = if x_velocity > 0x1_0000 {
                x_velocity - 0x1_0000
            } else if x_velocity < -0x1_0000 {
                x_velocity + 0x1_0000
            } else {
                x_velocity
            };
            projectile.x_velocity = -decayed_x_velocity;
        }

        // ASM loc_88E6 (kid.asm:9735): y-axis movement.
        // `move.l y_pos(a3),d7; move.l y_vel(a3),d4; bmi.w loc_893C` —
        // loads y_pos and y_vel, branches to upward path if y_vel < 0.
        // Downward: adds y_vel, checks tile-boundary crossing to detect floor.
        // Upward (loc_893C): mirror check for ceiling crossing.
        let y_velocity = projectile.y_velocity;
        let next_y = (i64::from(projectile.y_pos) + i64::from(y_velocity)) as u32;
        projectile.y_pos = next_y;
        let current_x_px = (projectile.x_pos >> 16) as i32;
        let next_y_px = (next_y >> 16) as i32;
        // ASM loc_8916 (kid.asm:9755): downward floor collision.
        // `bsr.w sub_B55C; cmpi.w #$6000,d5` probes terrain at new position.
        // If solid: `andi.w #$FFF0,d7; subq.w #1,d7` snaps y to tile top - 1,
        // `clr.w $20(a3)` zeroes the bounce counter field, falls to loc_897C
        // y-bounce: `neg.l d7; asr.l #1,d7` + `asr.l #1,d6; add.l d6,d7` = ¾ rebound.
        // ASM loc_8958 (kid.asm:9782): upward ceiling collision mirror —
        // snaps to `(y & $FFF0) + $10`, same ¾ rebound via loc_897C.
        if Self::juggernaut_skull_solid_probe_loc_b55c(grid, current_x_px, next_y_px) {
            let snapped_y_px = if y_velocity >= 0 {
                (next_y_px & !0xF) - 1
            } else {
                (next_y_px & !0xF) + 16
            };
            projectile.y_pos = (snapped_y_px.max(0) as u32) << 16;
            projectile.reaim_countdown = projectile.reaim_countdown.saturating_sub(1);
            if projectile.reaim_countdown == 0 {
                return false;
            }

            // ASM loc_897C (kid.asm:9794): y-bounce rebound speed.
            // `neg.l d7` negate y_vel, `asr.l #1,d7` → half,
            // `asr.l #1,d6; add.l d6,d7` → half + quarter = ¾ speed.
            let negated = -y_velocity;
            let half = negated >> 1;
            let quarter = half >> 1;
            projectile.y_velocity = half + quarter;
        }

        true
    }

    fn release_lion_projectile_owners(&mut self, owners: Vec<EnemyProjectileOwner>) {
        for owner in owners {
            if let Some(enemy) = self.active_enemies.iter_mut().find(|enemy| {
                enemy.family == EnemyFamily::Lion
                    && enemy.segment_index == owner.segment_index
                    && enemy.layout_index == owner.layout_index
                    && enemy.ufo_beam_interval == owner.token
            }) {
                enemy.ufo_beam_segments = enemy.ufo_beam_segments.saturating_sub(1);
            }
        }
    }

    fn resolve_drips_splashes(&mut self, splashes: Vec<PendingDripsSplash>) {
        for splash in splashes {
            let Some(enemy) = self.active_enemies.iter_mut().find(|enemy| {
                enemy.family == EnemyFamily::Drips
                    && enemy.segment_index == splash.owner.segment_index
                    && enemy.layout_index == splash.owner.layout_index
            }) else {
                continue;
            };
            drips_enter_splash_animation_loc_33488(enemy, splash.drop.x_pos, splash.drop.y_pos);
        }
    }

    fn resolve_ready_drips_splash_animations(
        &mut self,
        drips_surface_crawler_count: &mut usize,
        player_x: Option<u32>,
        spawned_projectiles: &mut Vec<EnemyProjectile>,
    ) {
        let player_x = player_x.unwrap_or(0);
        let kid_x_word = ((player_x >> 16) & 0xFFFF) as u16;
        let mut reseed_events = Vec::new();
        let mut ready_drips_indices = self
            .active_enemies
            .iter()
            .enumerate()
            .filter(|(_, enemy)| {
                enemy.family == EnemyFamily::Drips
                    && drips_splash_animation_remaining_loc_33488(enemy.wait_timer) == Some(0)
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        // ASM Drips.asm loc_33488 resumes each controller as its object-slot
        // coroutine wakes. The issue-260 oracle trace shows simultaneous
        // layout #08/#07 continuations consume loc_3340E RNG in descending
        // layout order, while the native active vector stores them ascending.
        ready_drips_indices.sort_by(|&left, &right| {
            let left_enemy = &self.active_enemies[left];
            let right_enemy = &self.active_enemies[right];
            (
                right_enemy.segment_index,
                right_enemy.layout_index,
                right_enemy.x_pos,
            )
                .cmp(&(
                    left_enemy.segment_index,
                    left_enemy.layout_index,
                    left_enemy.x_pos,
                ))
        });
        for enemy_index in ready_drips_indices {
            let enemy = &mut self.active_enemies[enemy_index];
            if drips_splash_animation_remaining_loc_33488(enemy.wait_timer) != Some(0) {
                continue;
            }
            let table_index = enemy.drips_surface_table_index;
            let spawn_crawler_on_splash = if enemy.drips_surface_table_index < 2 {
                enemy.drips_surface_table_index += 1;
                false
            } else {
                // ASM Drips.asm loc_33488 reaches the negative `off_33520`
                // sentinel, then loc_33506 resets the cursor before
                // returning to loc_3340E.
                enemy.drips_surface_table_index = 0;
                true
            };
            // ASM Drips.asm loc_33488 allocates the loc_335D8 child object,
            // then the parent continues through loc_3350E back to loc_3340E.
            // The newly allocated child does not run its own loc_335D8 random
            // branch until a later object pass, so the parent's wait reseed
            // consumes the next RNG byte first.
            let rng_before = self.rom_random.offset_word();
            let reseed_random_byte = self.rom_random.next_byte();
            let crawler_random_byte = if spawn_crawler_on_splash
                && *drips_surface_crawler_count < DRIPS_SURFACE_CRAWLER_LIMIT_LOC_33488
            {
                Some(self.rom_random.next_byte())
            } else {
                None
            };
            let rng_after = self.rom_random.offset_word();
            if spawn_crawler_on_splash
                && *drips_surface_crawler_count < DRIPS_SURFACE_CRAWLER_LIMIT_LOC_33488
            {
                let (drips_child_byte_44, drips_child_byte_45) =
                    drips_child_state_bytes_loc_3333e(enemy.enemy_level);
                spawned_projectiles.push(drips_surface_crawler_from_splash_loc_335d8(
                    enemy.drips_pending_splash_x_pos,
                    enemy.drips_pending_splash_y_pos,
                    drips_child_byte_44,
                    drips_child_byte_45,
                    crawler_random_byte.expect("crawler random byte is reserved above"),
                ));
                *drips_surface_crawler_count += 1;
                self.drips_surface_crawler_count_loc_fffffb42 += 1;
            }
            // ASM loc_3350E branches back to loc_3340E after the splash path.
            let wait_timer = drips_wait_timer_loc_3340e(reseed_random_byte, kid_x_word);
            reseed_events.push(format!(
                "#{:02}@{:#010x},{:#010x} table:{}->{} crawler={} crawler_rng={} kid_x={:#06x} rng:{:#06x}->{:#06x} reseed_rng={:#04x} wait={:#06x}",
                enemy.layout_index,
                enemy.x_pos,
                enemy.y_pos,
                table_index,
                enemy.drips_surface_table_index,
                u8::from(spawn_crawler_on_splash),
                crawler_random_byte
                    .map(|byte| format!("{byte:#04x}"))
                    .unwrap_or_else(|| String::from("none")),
                kid_x_word,
                rng_before,
                rng_after,
                reseed_random_byte,
                wait_timer
            ));
            enemy.wait_timer = wait_timer;
            enemy.drips_pending_splash_x_pos = 0;
            enemy.drips_pending_splash_y_pos = 0;
        }
        if !reseed_events.is_empty() {
            self.last_drips_splash_reseed_probe = Some(reseed_events.join(";"));
        }
    }

    pub(crate) fn step_fire_demon_trails(&mut self) {
        let grid = self.block_grid.as_ref();
        self.fire_demon_trails.retain_mut(|trail| {
            if let Some(g) = grid
                && !fire_demon_trail_has_floor_support_loc_3a63c(g, trail.x_pos, trail.y_pos)
            {
                return false;
            }
            // ASM Fire_Demon loc_3A6CA: after one hibernated frame,
            // `subq.w #1,d3; bne.s loc_3A6CA`; falling through deletes only
            // when the post-decrement word becomes zero.
            trail.remaining_frames = trail.remaining_frames.wrapping_sub(1);
            trail.remaining_frames != 0
        });
    }

    pub(crate) fn step_diamond_power_rain_slots(&mut self) {
        let player_x_velocity = self
            .opening_slice
            .as_ref()
            .map_or(0, |s| s.player.x_velocity);
        let camera = self.opening_slice.as_ref().map(|s| s.camera);
        let tracking_rain_scan_enabled = self.progression.diamond_power_type
            == crate::helmets::DiamondPowerType::TrackingRain
            && (self.frame_index & 7) == 1;
        let rain_target_candidates: Vec<SnakeTargetCandidate> = self
            .active_enemies
            .iter()
            .map(|enemy| SnakeTargetCandidate {
                x_pos: (enemy.x_pos >> 16) as u16 as i16,
                y_pos: (enemy.y_pos >> 16) as u16 as i16,
            })
            .collect();

        self.diamond_power_rain_slots.retain_mut(|slot| {
            if slot.tracking_word_3e == 0
                && tracking_rain_scan_enabled
                && let Some(camera) = camera
                && let Some(target) = diamond_power_rain_find_nearest_enemy(
                    (slot.x_pos >> 16) as u16 as i16,
                    (slot.y_pos >> 16) as u16 as i16,
                    (camera.x_pos >> 16) as u16 as i16,
                    (camera.y_pos >> 16) as u16 as i16,
                    &rain_target_candidates,
                )
                && let Some(tracking) = diamond_power_rain_compute_tracking_velocity(
                    target.distance,
                    (target.x as i32 as u32) << 16,
                    (target.y as i32 as u32) << 16,
                    slot.x_pos,
                    slot.y_pos,
                )
            {
                // ASM loc_3FF3C/loc_3FF6A: valid Tracking Rain target writes
                // `$26/$2A` velocity, then increments `$3E(a0)` so later
                // frames use loc_3FE98 instead of the scan path.
                slot.x_velocity = tracking.x_vel;
                slot.y_velocity = tracking.y_vel;
                slot.tracking_word_3e = slot.tracking_word_3e.wrapping_add(1);
            }

            let x_velocity = if slot.tracking_word_3e == 0 {
                // ASM loc_3FE62: a5 = Kid x_vel * 3/4.
                // ASM loc_3FF70: non-tracking rain adds that drift after the
                // slot's own `$26(a0)` x velocity.
                let kid_drift = player_x_velocity - (player_x_velocity >> 2);
                slot.x_velocity.wrapping_add(kid_drift)
            } else {
                slot.x_velocity
            };

            slot.x_pos = (slot.x_pos as i64 + i64::from(x_velocity)) as u32;
            slot.y_pos = (slot.y_pos as i64 + i64::from(slot.y_velocity)) as u32;

            if let Some(camera) = camera {
                let screen_x = ((slot.x_pos >> 16) as i32)
                    .wrapping_add(0x7F)
                    .wrapping_sub((camera.x_pos >> 16) as i32);
                let screen_y = ((slot.y_pos >> 16) as i32)
                    .wrapping_add(0x7F)
                    .wrapping_sub((camera.y_pos >> 16) as i32);
                // ASM loc_3FF90/loc_3FFC0: DiamondPower_Main_SlashingRain
                // removes slots only when the compiled diamond sprite leaves
                // the ROM camera-relative screen bounds.
                let (_, action) = diamond_power_rain_screen_cull_and_compile(
                    screen_x as i16,
                    screen_y as i16,
                    slot.size_word_24,
                );
                if action == RainSlotSpriteAction::Removed {
                    return false;
                }
            }

            true
        });
    }

    /// Advance all active projectiles: move by velocity, decrement lifetime, remove expired.
    /// Homing projectiles accelerate toward `player_pos` each frame.
    /// Re-aiming projectiles (Archer arrows) adjust angle ±1 every `reaim_interval` frames.
    /// Wall-bounce projectiles (Lion) reverse velocity on terrain collision (Lion.asm loc_3E260).
    pub(crate) fn step_projectiles(&mut self) {
        self.step_fire_demon_trails();
        self.step_diamond_power_rain_slots();

        let player_x = self.opening_slice.as_ref().map(|s| s.player.x_pos);
        let player_y = self.opening_slice.as_ref().map(|s| s.player.y_pos);
        let player_x_velocity = self
            .opening_slice
            .as_ref()
            .map_or(0, |s| s.player.x_velocity);
        let camera = self.opening_slice.as_ref().map(|s| s.camera);
        let level_bounds = self.level_bounds(self.current_level);
        let current_frame_index = self.frame_index;
        let tracking_rain_scan_enabled = self.progression.diamond_power_type
            == crate::helmets::DiamondPowerType::TrackingRain
            && (self.frame_index & 7) == 1;
        let rain_target_candidates: Vec<SnakeTargetCandidate> = self
            .active_enemies
            .iter()
            .map(|enemy| SnakeTargetCandidate {
                x_pos: (enemy.x_pos >> 16) as u16 as i16,
                y_pos: (enemy.y_pos >> 16) as u16 as i16,
            })
            .collect();
        let mut released_owners = Vec::new();
        // ASM Drips.asm loc_33488 increments `$FFFFFB42` when loc_335D8 is
        // allocated. The count is decremented only at loc_3392C, after the
        // crawler has entered its loc_338F8/loc_338EA finish animation, so
        // native must include the burst-helper lifetime in the allocation cap.
        let visible_drips_surface_crawler_count = self
            .projectiles
            .iter()
            .filter(|projectile| {
                matches!(
                    projectile.kind,
                    EnemyProjectileKind::DripsSurfaceCrawler
                        | EnemyProjectileKind::DripsSurfaceCrawlerBurst
                        | EnemyProjectileKind::DripsSurfaceCrawlerFinish
                )
            })
            .count();
        self.drips_surface_crawler_count_loc_fffffb42 = self
            .drips_surface_crawler_count_loc_fffffb42
            .max(visible_drips_surface_crawler_count);
        let mut drips_surface_crawler_count = self.drips_surface_crawler_count_loc_fffffb42;
        let mut spawned_projectiles = Vec::new();
        self.resolve_ready_drips_splash_animations(
            &mut drips_surface_crawler_count,
            player_x,
            &mut spawned_projectiles,
        );
        let grid = self.block_grid.as_ref();
        let mut pending_drips_splashes = Vec::new();
        let mut pending_tar_monster_bursts = Vec::new();
        let mut pending_shooter_layout_interactions = Vec::new();
        let mut expired_drips_surface_crawler_finishes = 0usize;
        self.projectiles.retain_mut(|p| {
            let is_tank_bullet = p.kind == EnemyProjectileKind::TankBullet;
            let is_robot_shot = p.kind == EnemyProjectileKind::RobotShot;
            let is_cloud_lightning = p.kind == EnemyProjectileKind::CloudLightning;
            let is_dragon_flying_descent_fire =
                p.kind == EnemyProjectileKind::DragonFlyingDescentFire;
            let is_hail = p.kind == EnemyProjectileKind::Hail;
            let is_diamond_power_rain = p.kind == EnemyProjectileKind::DiamondPowerRain;
            let is_maniaxe_axe = p.kind == EnemyProjectileKind::ManiaxeAxe;
            let is_drips_drop = p.kind == EnemyProjectileKind::DripsDrop;
            let is_lion_projectile = p.kind == EnemyProjectileKind::LionProjectile;
            let is_boss_head_arrow = p.kind == EnemyProjectileKind::BossHeadArrow;
            let is_boss_head_boomerang = p.kind == EnemyProjectileKind::BossHeadBoomerang;
            let is_juggernaut_skull = Self::is_juggernaut_skull_projectile(p);
            let uses_loc_3263c_post_move_gravity = matches!(
                p.kind,
                EnemyProjectileKind::TarMonsterBurstChild
                    | EnemyProjectileKind::DripsSurfaceCrawlerBurst
            );
            if is_boss_head_arrow && p.lifetime == 0 {
                if p.animation_elapsed_frames == 0 {
                    // ASM loc_3A134 -> loc_3A14E: once `$40(a3)` reaches 0,
                    // clear x_vel, switch to the stuck animation, and seed
                    // `$42(a3)=$C8` for the wait-before-delete loop.
                    p.x_velocity = 0;
                    p.y_velocity = 0;
                    p.animation_elapsed_frames = 0x00C8;
                } else if p.animation_elapsed_frames == 1 {
                    return false;
                }
            }
            if is_boss_head_boomerang && p.lifetime == 0 {
                return false;
            }
            if !is_tank_bullet
                && !is_robot_shot
                && !is_cloud_lightning
                && !is_dragon_flying_descent_fire
                && !is_hail
                && !is_diamond_power_rain
                && !is_maniaxe_axe
                && !is_boss_head_arrow
                && !is_boss_head_boomerang
                && p.lifetime == 0
            {
                if p.kind == EnemyProjectileKind::DripsSurfaceCrawlerFinish {
                    // ASM Drips.asm loc_3392C decrements `$FFFFFB42` after
                    // the original crawler's finish animation completes.
                    expired_drips_surface_crawler_finishes += 1;
                }
                if let Some(owner) = p.owner {
                    released_owners.push(owner);
                }
                return false;
            }
            if !is_tank_bullet
                && !is_robot_shot
                && !is_cloud_lightning
                && !is_dragon_flying_descent_fire
                && !is_hail
                && !is_diamond_power_rain
                && !is_maniaxe_axe
                && !is_boss_head_arrow
                && !is_boss_head_boomerang
                // ASM Lion.asm loc_3E192 has no DBF lifetime; zero is only
                // native's deferred-delete marker after a Kid collision.
                && !is_lion_projectile
                // ASM sub_87B0/loc_8810 has no DBF lifetime; zero remains
                // native's generic delete marker for this projectile slot.
                && !is_juggernaut_skull
            {
                p.lifetime -= 1;
            }
            if is_robot_shot {
                // ASM Robot.asm loc_3AC0C initialises `stru_3A754`; loc_3AC48
                // then loops without a DBF lifetime, so keep an independent
                // animation clock rather than deleting on timer expiry.
                p.animation_elapsed_frames = p.animation_elapsed_frames.wrapping_add(1);
            }
            if p.kind == EnemyProjectileKind::DripsSurfaceCrawler
                && p.drips_surface_crawler_state == DripsSurfaceCrawlerState::StartupLoc335D8
            {
                if p.animation_elapsed_frames > 1 {
                    p.animation_elapsed_frames -= 1;
                    return true;
                }
                drips_surface_crawler_finish_startup_loc_335d8(p);
                // ASM loc_33650 starts with `Hibernate_Object_1Frame`, so the
                // first support/collision probe runs on the next native frame.
                return true;
            }
            if !is_drips_drop && !uses_loc_3263c_post_move_gravity {
                p.y_velocity += p.gravity;
            }

            // Arrow re-aiming: ASM loc_342D6 loop. Every `reaim_interval` frames,
            // compute target angle toward player, adjust current_angle by ±1,
            // then update velocity from the new angle.
            if p.reaim_interval > 0 {
                if p.reaim_countdown == 0 {
                    p.reaim_countdown = p.reaim_interval;
                    if let (Some(px), Some(py)) = (player_x, player_y) {
                        let target = archer_angle_to_player(p.x_pos, p.y_pos, px, py) as u8;
                        let reaim = archer_reaim_loc_3432E(p.current_angle, target);
                        p.current_angle = reaim.current_angle;
                        p.x_velocity = reaim.x_velocity;
                        p.y_velocity = reaim.y_velocity;
                    }
                } else {
                    p.reaim_countdown -= 1;
                }
            }
            // ASM loc_3E192: after lifecycle checks, this label drives per-frame
            // lion projectile homing adjustments and then loops back for the next
            // frame unless collision/terrain branches are taken.
            // Homing: accelerate toward player (per-projectile accel from ASM).
            if p.homing_accel != 0 {
                if let (Some(px), Some(py)) = (player_x, player_y) {
                    let dx_from_kid = p.x_pos as i64 - px as i64;
                    match dx_from_kid.cmp(&0) {
                        std::cmp::Ordering::Less => {
                            // ASM loc_3E1F4: projectile left of Kid, add `$2000`.
                            p.x_velocity += p.homing_accel;
                        }
                        std::cmp::Ordering::Greater => {
                            // ASM loc_3E1C2: projectile right of Kid, subtract `$2000`.
                            p.x_velocity -= p.homing_accel;
                        }
                        std::cmp::Ordering::Equal => {}
                    }
                    let dy_from_target = p.y_pos as i64 - py as i64 - (0x10 << 16);
                    match dy_from_target.cmp(&0) {
                        std::cmp::Ordering::Less => {
                            // ASM loc_3E240: projectile above Kid+16, add `$2000`.
                            p.y_velocity += p.homing_accel;
                        }
                        std::cmp::Ordering::Greater => {
                            // ASM loc_3E20E fallthrough: projectile below Kid+16, subtract `$2000`.
                            p.y_velocity -= p.homing_accel;
                        }
                        std::cmp::Ordering::Equal => {}
                    }
                }
                // Speed cap (Lion.asm: ±$60000 on each axis)
                if p.speed_cap > 0 {
                    p.x_velocity = p.x_velocity.clamp(-p.speed_cap, p.speed_cap);
                    p.y_velocity = p.y_velocity.clamp(-p.speed_cap, p.speed_cap);
                }
            }
            if is_juggernaut_skull {
                return Self::step_juggernaut_skull_sub_87B0(p, grid);
            }

            let drips_movement_state_handled = if p.kind == EnemyProjectileKind::DripsSurfaceCrawler
                && p.drips_surface_crawler_state == DripsSurfaceCrawlerState::VerticalLoc336D0
                && let Some(g) = grid
            {
                match drips_surface_crawler_vertical_step_loc_336d0(g, p, level_bounds) {
                    DripsSurfaceCrawlerVerticalStepLoc336D0::Loop => true,
                    DripsSurfaceCrawlerVerticalStepLoc336D0::Turn(script) => {
                        // ASM Drips.asm loc_336D0/loc_3371A dispatch their
                        // side-column outcomes through loc_33AF6.
                        drips_surface_crawler_apply_turn_script_loc_33af6(p, script);
                        return true;
                    }
                    DripsSurfaceCrawlerVerticalStepLoc336D0::Exit => {
                        // ASM Drips.asm loc_336D0 exits through loc_338F8,
                        // then loc_3392C delays the `$FFFFFB42` decrement
                        // until the finish animation completes.
                        drips_surface_crawler_exit_via_loc_338f8(p);
                        return true;
                    }
                }
            } else {
                false
            };

            let drips_movement_state_handled = if p.kind == EnemyProjectileKind::DripsSurfaceCrawler
                && p.drips_surface_crawler_state == DripsSurfaceCrawlerState::VerticalLoc33730
                && let Some(g) = grid
            {
                match drips_surface_crawler_vertical_step_loc_33730(g, p, level_bounds) {
                    DripsSurfaceCrawlerVerticalStepLoc33730::Loop => true,
                    DripsSurfaceCrawlerVerticalStepLoc33730::Turn(script) => {
                        // ASM Drips.asm loc_33730 dispatches row-probe outcomes
                        // through loc_33964/loc_339FA/loc_339D2 into loc_33AF6.
                        drips_surface_crawler_apply_turn_script_loc_33af6(p, script);
                        return true;
                    }
                    DripsSurfaceCrawlerVerticalStepLoc33730::Exit => {
                        // ASM Drips.asm loc_33730 exits through loc_338F8 and
                        // keeps the crawler counted until loc_3392C.
                        drips_surface_crawler_exit_via_loc_338f8(p);
                        return true;
                    }
                }
            } else {
                drips_movement_state_handled
            };

            let drips_movement_state_handled = if p.kind == EnemyProjectileKind::DripsSurfaceCrawler
                && p.drips_surface_crawler_state == DripsSurfaceCrawlerState::VerticalLoc337A0
                && let Some(g) = grid
            {
                match drips_surface_crawler_vertical_step_loc_337a0(g, p, level_bounds) {
                    DripsSurfaceCrawlerVerticalStepLoc33730::Loop => true,
                    DripsSurfaceCrawlerVerticalStepLoc33730::Turn(script) => {
                        // ASM Drips.asm loc_337A0/loc_337F2 dispatch row-probe
                        // outcomes through loc_33AF6 just like loc_33730.
                        drips_surface_crawler_apply_turn_script_loc_33af6(p, script);
                        return true;
                    }
                    DripsSurfaceCrawlerVerticalStepLoc33730::Exit => {
                        // ASM Drips.asm loc_337A0 exits through loc_338F8 and
                        // keeps the crawler counted until loc_3392C.
                        drips_surface_crawler_exit_via_loc_338f8(p);
                        return true;
                    }
                }
            } else {
                drips_movement_state_handled
            };

            let drips_movement_state_handled = if p.kind == EnemyProjectileKind::DripsSurfaceCrawler
                && p.drips_surface_crawler_state == DripsSurfaceCrawlerState::DiagonalLoc33810
                && let Some(g) = grid
            {
                match drips_surface_crawler_diagonal_step_loc_33810(g, p, level_bounds) {
                    DripsSurfaceCrawlerDiagonalStepLoc33810::Loop => true,
                    DripsSurfaceCrawlerDiagonalStepLoc33810::Turn(script) => {
                        // ASM Drips.asm loc_33810/loc_33878 dispatch diagonal
                        // probe outcomes through loc_33AF6.
                        drips_surface_crawler_apply_turn_script_loc_33af6(p, script);
                        return true;
                    }
                    DripsSurfaceCrawlerDiagonalStepLoc33810::Exit => {
                        // ASM Drips.asm loc_33810 exits through loc_338F8 and
                        // keeps the crawler counted until loc_3392C.
                        drips_surface_crawler_exit_via_loc_338f8(p);
                        return true;
                    }
                }
            } else {
                drips_movement_state_handled
            };

            let drips_movement_state_handled = if p.kind == EnemyProjectileKind::DripsSurfaceCrawler
                && p.drips_surface_crawler_state == DripsSurfaceCrawlerState::DiagonalLoc33892
                && let Some(g) = grid
            {
                match drips_surface_crawler_diagonal_step_loc_33892(g, p, level_bounds) {
                    DripsSurfaceCrawlerDiagonalStepLoc33810::Loop => true,
                    DripsSurfaceCrawlerDiagonalStepLoc33810::Turn(script) => {
                        // ASM Drips.asm loc_33892/loc_338CE dispatch diagonal
                        // probe outcomes through loc_33AF6.
                        drips_surface_crawler_apply_turn_script_loc_33af6(p, script);
                        return true;
                    }
                    DripsSurfaceCrawlerDiagonalStepLoc33810::Exit => {
                        // ASM Drips.asm loc_33892 exits through loc_338F8 and
                        // keeps the crawler counted until loc_3392C.
                        drips_surface_crawler_exit_via_loc_338f8(p);
                        return true;
                    }
                }
            } else {
                drips_movement_state_handled
            };

            let drips_movement_state_handled = if p.kind == EnemyProjectileKind::DripsSurfaceCrawler
                && p.drips_surface_crawler_state == DripsSurfaceCrawlerState::IdleLoc33A7C
            {
                true
            } else {
                drips_movement_state_handled
            };

            if p.kind == EnemyProjectileKind::DripsSurfaceCrawler
                && !drips_movement_state_handled
                && let Some(g) = grid
                && !drips_surface_crawler_supported_below_loc_33650(g, p)
            {
                // ASM Drips.asm loc_33650 leaves d6=0,d7=-8 from loc_33550;
                // a non-$6000 loc_331D2 support probe branches to loc_338F8,
                // which shifts the crawler before allocating the loc_325B6
                // helper. Native keeps that helper in-place as the remaining
                // non-damaging lifecycle, preserving the `$FFFFFB42` cap until
                // loc_3392C would delete it.
                *p = drips_surface_crawler_burst_helper_loc_338f8(p, 0, -8);
                return true;
            }

            if p.kind == EnemyProjectileKind::DripsSurfaceCrawler
                && !drips_movement_state_handled
                && drips_surface_crawler_out_of_level_bounds_loc_33588(
                    p,
                    level_bounds.width_pixels,
                    level_bounds.height_pixels,
                )
            {
                // ASM Drips.asm loc_33588/loc_335B0 route out-of-level edge
                // probes through loc_338EA/loc_338F8, then loc_3392C deletes
                // the crawler after its finish animation and decrements
                // `$FFFFFB42`.
                *p = drips_surface_crawler_finish_animation_loc_3392c(p);
                return true;
            }

            if p.kind == EnemyProjectileKind::DripsSurfaceCrawler
                && !drips_movement_state_handled
                && let Some(g) = grid
            {
                match drips_surface_crawler_side_column_step_loc_33650(g, p) {
                    DripsSurfaceCrawlerSideColumnStepLoc33650::Loop => {}
                    DripsSurfaceCrawlerSideColumnStepLoc33650::Turn(script) => {
                        // ASM Drips.asm loc_3393C/loc_33A40 branch into the
                        // shared loc_33AF6 script and then hibernate in the
                        // next movement label. Return now so the generic
                        // projectile mover does not add an extra frame.
                        drips_surface_crawler_apply_turn_script_loc_33af6(p, script);
                        return true;
                    }
                }
            }

            if is_diamond_power_rain
                && p.diamond_power_rain_tracking_word_3e == 0
                && tracking_rain_scan_enabled
                && let Some(camera) = camera
                && let Some(target) = diamond_power_rain_find_nearest_enemy(
                    (p.x_pos >> 16) as u16 as i16,
                    (p.y_pos >> 16) as u16 as i16,
                    (camera.x_pos >> 16) as u16 as i16,
                    (camera.y_pos >> 16) as u16 as i16,
                    &rain_target_candidates,
                )
                && let Some(tracking) = diamond_power_rain_compute_tracking_velocity(
                    target.distance,
                    (target.x as i32 as u32) << 16,
                    (target.y as i32 as u32) << 16,
                    p.x_pos,
                    p.y_pos,
                )
            {
                // ASM loc_3FF3C: once a valid target in [8,$40) is found, the
                // rain slot stores `$26/$2A` tracking velocities and increments
                // `$3E(a0)`, causing future frames to skip the scan path.
                p.x_velocity = tracking.x_vel;
                p.y_velocity = tracking.y_vel;
                p.diamond_power_rain_tracking_word_3e =
                    p.diamond_power_rain_tracking_word_3e.wrapping_add(1);
            }

            if p.kind == EnemyProjectileKind::ShooterBlock
                && p.shooter_tracks_layout
                && let Some(g) = grid
            {
                if p.shooter_skip_projectile_step_frame == Some(current_frame_index) {
                    p.shooter_skip_projectile_step_frame = None;
                    return true;
                }
                match Self::step_tracked_shooter_object_loc_EDFA(g, p) {
                    Ok(()) => return true,
                    Err(interaction) => {
                        pending_shooter_layout_interactions.push(interaction);
                        return false;
                    }
                }
            }

            let x_velocity = if is_diamond_power_rain && p.diamond_power_rain_tracking_word_3e == 0
            {
                // ASM loc_3FE62 (kid.asm:45210): a5 = Kid x_vel * 3/4.
                // ASM loc_3FF70: non-tracking rain adds that drift after the
                // slot's own `$26(a0)` x velocity.
                let kid_drift = player_x_velocity - (player_x_velocity >> 2);
                p.x_velocity.wrapping_add(kid_drift)
            } else {
                p.x_velocity
            };

            p.x_pos = (p.x_pos as i64 + i64::from(x_velocity)) as u32;
            p.y_pos = (p.y_pos as i64 + i64::from(p.y_velocity)) as u32;
            if is_boss_head_arrow {
                if p.lifetime > 0 {
                    // ASM loc_3A168: during the moving phase, each hibernated
                    // frame decrements `$40(a3)` after the object has moved.
                    p.lifetime -= 1;
                } else {
                    // ASM loc_3A15E: after x_vel is cleared, each stuck frame
                    // decrements `$42(a3)` until the next loop sees zero and
                    // deletes the object.
                    p.animation_elapsed_frames = p.animation_elapsed_frames.saturating_sub(1);
                }
            }
            if is_boss_head_boomerang {
                let route = crate::combat::fire_demon_fragment_decel_step_loc_3A24C(
                    p.boss_head_boomerang_direction_byte_d0,
                    p.lifetime,
                    (p.x_pos >> 16) as u16,
                    level_bounds.width_pixels,
                );
                p.x_velocity = p.x_velocity.wrapping_add(route.x_vel_delta);
                p.lifetime = route.next_horizontal_counter_40;
                if route.delete_current_object {
                    return false;
                }
            }
            if is_drips_drop || uses_loc_3263c_post_move_gravity {
                // ASM Drips.asm loc_3345C and Tar_Monster.asm loc_3263C: the
                // hibernated object movement uses the previous y-velocity; the
                // gravity step prepares the next frame.
                p.y_velocity += p.gravity;
            }

            if is_dragon_flying_descent_fire
                && !step_dragon_flying_descent_fire_loc_34c26(p, grid, camera)
            {
                return false;
            }

            if p.kind == EnemyProjectileKind::ShooterBlock
                && let Some(g) = grid
                && let Some(interaction) = Self::shooter_object_layout_interaction_loc_F00A(g, p)
            {
                pending_shooter_layout_interactions.push(interaction);
                return false;
            }

            if is_diamond_power_rain && let Some(camera) = camera {
                let screen_x = ((p.x_pos >> 16) as i32)
                    .wrapping_add(0x7F)
                    .wrapping_sub((camera.x_pos >> 16) as i32);
                let screen_y = ((p.y_pos >> 16) as i32)
                    .wrapping_add(0x7F)
                    .wrapping_sub((camera.y_pos >> 16) as i32);
                // ASM loc_3FF90/loc_3FFC0: DiamondPower_Main_SlashingRain
                // removes rain slots only when the compiled diamond sprite
                // leaves the ROM's camera-relative screen bounds.
                let (_, action) = diamond_power_rain_screen_cull_and_compile(
                    screen_x as i16,
                    screen_y as i16,
                    p.diamond_power_rain_size_word_24,
                );
                if action == RainSlotSpriteAction::Removed {
                    return false;
                }
            }

            if is_hail && let Some(camera) = camera {
                let delete_bottom_px = (camera.y_pos >> 16) as i32 + 0x00E0;
                let delete_left_px = (camera.x_pos >> 16) as i32 - 0x0010;
                let projectile_y = (p.y_pos >> 16) as i32;
                let projectile_x = (p.x_pos >> 16) as i32;
                // ASM loc_6212/loc_6226: after `addi.w #4,(a0)` and
                // `subi.w #1,2(a0)`, clear the hail slot when it falls below
                // Camera_Y_pos+$E0 or drifts left of Camera_X_pos-$10.
                if projectile_y > delete_bottom_px || projectile_x < delete_left_px {
                    return false;
                }
            }

            if is_tank_bullet && let Some(camera) = camera {
                // ASM Tank_Shooting.asm loc_3B7BA: tank shots do not use a DBF
                // lifetime; after each hibernated frame they survive only while
                // their signed camera-relative x/y words stay in [-$80, $1C0]
                // and the worker's level-collision word stays clear.
                let collision_type = if let Some(g) = grid
                    && projectile_terrain_contact_loc_3345c(g, p)
                {
                    1
                } else {
                    0
                };
                let route = tank_bullet_loop_route_loc_3b7ba(
                    (p.x_pos >> 16) as u16,
                    (p.y_pos >> 16) as u16,
                    (camera.x_pos >> 16) as u16,
                    (camera.y_pos >> 16) as u16,
                    collision_type,
                );
                if route == TankBulletLoopRouteLoc3B7BA::Delete {
                    return false;
                }
            }

            if is_robot_shot && let Some(camera) = camera {
                // ASM Robot.asm loc_3AC48: Robot shots have no DBF lifetime.
                // Each frame clears collision_type, moves, then exits flight
                // once the signed camera box is left or level collision becomes
                // non-zero. loc_3AC7E/loc_3AC92 then clears velocity and runs
                // the short `stru_3A762` impact animation before deletion.
                let collision_type = if let Some(g) = grid
                    && projectile_terrain_contact_loc_3345c(g, p)
                {
                    1
                } else {
                    0
                };
                let screen_x =
                    ((p.x_pos >> 16) as i32).wrapping_sub((camera.x_pos >> 16) as i32) as i16;
                let screen_y =
                    ((p.y_pos >> 16) as i32).wrapping_sub((camera.y_pos >> 16) as i32) as i16;
                if robot_shot_flight_loop_loc_3ac48(screen_x, screen_y, collision_type).route
                    == RobotShotFlightRouteLoc3AC48::StartImpact
                {
                    let x_direction = if p.x_velocity < 0 { 0 } else { 0xFF };
                    let impact = robot_shot_impact_prep_loc_3ac7e(x_direction);
                    p.x_velocity = impact.projectile_x_velocity;
                    p.y_velocity = 0;
                    p.x_pos = (p.x_pos as i64 + (i64::from(impact.impact_x_delta_px) << 16)) as u32;
                    p.kind = EnemyProjectileKind::RobotShotImpact;
                    // ASM loc_3AC92 hibernates until `stru_3A762` finishes:
                    // waits 3, 4, and 8 frames before deleting the shot.
                    p.lifetime = ROBOT_SHOT_IMPACT_ANIMATION_FRAMES_LOC_3A762;
                }
            }

            if p.kind == EnemyProjectileKind::TarMonsterStraightGlob
                && let Some(g) = grid
                && projectile_terrain_contact_loc_3345c(g, p)
            {
                // ASM Tar_Monster.asm loc_324E6 -> loc_324F8: once the
                // straight glob's collision word is non-zero, hide/delete the
                // original shot and allocate the five burst children.
                spawn_tar_monster_retaliation(&mut pending_tar_monster_bursts, p.x_pos, p.y_pos);
                return false;
            }

            if matches!(
                p.kind,
                EnemyProjectileKind::TarMonsterBurstChild
                    | EnemyProjectileKind::DripsSurfaceCrawlerBurst
            ) && let Some(g) = grid
                && !apply_tar_monster_burst_child_collision_loc_32682(p, g)
            {
                return false;
            }

            if p.kind == EnemyProjectileKind::ManiaxeAxe
                && let Some(camera) = camera
            {
                let projectile_x = (p.x_pos >> 16) as i32;
                let projectile_y = (p.y_pos >> 16) as i32;
                let camera_x = (camera.x_pos >> 16) as i32;
                let camera_y = (camera.y_pos >> 16) as i32;
                let min_x = camera_x - 0x10;
                let max_x = min_x + 0x160;
                let min_y = camera_y - 0x10;
                let max_y = min_y + 0x100;
                // ASM ManiaxeAxe_CheckOutOfBounds (kid.asm:9933): x/y are
                // compared against the camera-padded box before loc_8B32
                // returns true and loc_8AF2 deletes the axe.
                if projectile_x < min_x
                    || projectile_x > max_x
                    || projectile_y < min_y
                    || projectile_y > max_y
                {
                    return false;
                }
            }

            if p.kind == EnemyProjectileKind::FireDemonDeathTrail
                && let Some(g) = grid
                && !fire_demon_trail_has_floor_support_loc_3a63c(g, p.x_pos, p.y_pos)
            {
                return false;
            }

            // ASM Lion.asm sub_3E2CE: lion projectiles only survive while the
            // parent token matches and the object stays inside the camera box
            // x=[-0x40,0x180], y=[-0x40,0x120]. Drips.asm loc_3345C instead
            // checks the hidden controller through Object_CheckInRange's wider
            // normal range, so the falling child must not use this narrow
            // generic projectile cull.
            if let (Some(owner), Some(camera)) = (p.owner, camera)
                && p.kind != EnemyProjectileKind::DripsDrop
            {
                let screen_x = ((p.x_pos >> 16) as i32) - ((camera.x_pos >> 16) as i32);
                let screen_y = ((p.y_pos >> 16) as i32) - ((camera.y_pos >> 16) as i32);
                let in_camera_box =
                    (-0x40..=0x180).contains(&screen_x) && (-0x40..=0x120).contains(&screen_y);
                if !in_camera_box {
                    released_owners.push(owner);
                    return false;
                }
            }

            // Terrain-kill: projectiles such as Archer arrows and Maniaxe axes
            // die on level/object collision.
            // ASM loc_342D6: `tst.b $19(a3)` (has_level_collision) / `tst.w collision_type(a3)`.
            // ASM Archer.asm loc_34336 then decrements Number_of_Arrows; native
            // projectile retention represents that count by shrinking the Vec.
            if p.terrain_kill
                && let Some(g) = grid
                && projectile_terrain_contact_loc_3345c(g, p)
            {
                if p.kind == EnemyProjectileKind::DripsDrop {
                    // ASM Drips.asm loc_33488 -> loc_335D8: after the
                    // first two falling-child splashes, `off_33520`
                    // writes pool-growth sprites only. The negative
                    // sentinel path can allocate a hostile surface crawler
                    // while `$FFFFFB42 < 5`; loc_335D8 seeds x_vel to
                    // +/-$3000 from `Get_RandomNumber_byte` bit 2.
                    if let Some(owner) = p.owner {
                        // ASM Drips.asm loc_3345C plays `sfx_Drips_dripping`
                        // before handing the falling child to loc_33488.
                        self.played_sound_effects
                            .push(crate::combat::QueuedSoundEffect::DripsDripping);
                        pending_drips_splashes.push(PendingDripsSplash { owner, drop: *p });
                    } else if p.drips_spawn_crawler_on_splash
                        && drips_surface_crawler_count < DRIPS_SURFACE_CRAWLER_LIMIT_LOC_33488
                    {
                        spawned_projectiles.push(drips_surface_crawler_from_drop_loc_335d8(p, 0));
                        drips_surface_crawler_count += 1;
                        self.drips_surface_crawler_count_loc_fffffb42 += 1;
                    }
                    return false;
                }
                if let Some(owner) = p.owner {
                    released_owners.push(owner);
                }
                return false; // destroyed on terrain contact
            }

            // Wall-bounce: reverse velocity on terrain contact (Lion.asm loc_3E260).
            // ASM uses `moveq #$F,d0` + `dbf d0,loc_3E2A8` (16 attempts).
            // Native: check if projectile centre is inside a solid cell after moving.
            if p.wall_bounce
                && let Some(g) = grid
            {
                let mut remaining_bounces = 0x0Fi16;
                // ASM Lion.asm loc_3E2C0: `dbf d0,loc_3E2A8` with d0 seeded to
                // `moveq #$F,d0`. The ROM's bound is the `dbf` decrement-and-
                // branch instruction itself — it fires at the loop tail and
                // caps execution at exactly 16 iterations. Express that bound
                // structurally with `for _ in 0..16` so the cap is faithful to
                // the ROM (the same place the ROM has it) and the loop cannot
                // be turned into an infinite spin by mutating the helper that
                // models `dbf` (`lion_projectile_bounce_iterations_continue_loc_3e2c0`).
                for _ in 0..16 {
                    // ASM loc_3E2A8 then loc_3E2C0: loop up to 16 bounce attempts using `dbf d0`.
                    let px_i = (p.x_pos >> 16) as i32;
                    let py_i = (p.y_pos >> 16) as i32;
                    let col = terrain::pixel_to_cell(px_i).max(0) as u16;
                    let row = terrain::pixel_to_cell(py_i).max(0) as u16;
                    let cell = g.cell_at(col, row);
                    if !cell.has_block || cell.collision == terrain::CollisionKind::Empty {
                        break;
                    }

                    // Undo the move.
                    p.x_pos = (p.x_pos as i64 - i64::from(p.x_velocity)) as u32;
                    p.y_pos = (p.y_pos as i64 - i64::from(p.y_velocity)) as u32;

                    // Determine bounce axis: probe horizontally and vertically.
                    let prev_px = (p.x_pos >> 16) as i32;
                    let prev_py = (p.y_pos >> 16) as i32;
                    let next_cx = prev_px + (p.x_velocity >> 16);
                    let next_cy = prev_py + (p.y_velocity >> 16);

                    let col_h = terrain::pixel_to_cell(next_cx).max(0) as u16;
                    let row_h = terrain::pixel_to_cell(prev_py).max(0) as u16;
                    let cell_h = g.cell_at(col_h, row_h);

                    let col_v = terrain::pixel_to_cell(prev_px).max(0) as u16;
                    let row_v = terrain::pixel_to_cell(next_cy).max(0) as u16;
                    let cell_v = g.cell_at(col_v, row_v);

                    let h_blocked =
                        cell_h.has_block && cell_h.collision != terrain::CollisionKind::Empty;
                    let v_blocked =
                        cell_v.has_block && cell_v.collision != terrain::CollisionKind::Empty;

                    let terrain_collision_id = lion_projectile_terrain_collision_id_loc_3e260(
                        h_blocked,
                        v_blocked,
                        p.x_velocity,
                        p.y_velocity,
                        cell,
                    );

                    let (x_velocity, y_velocity) =
                        lion_projectile_bounce_velocity_dispatch_loc_3e266(
                            h_blocked,
                            v_blocked,
                            p.x_velocity,
                            p.y_velocity,
                            terrain_collision_id,
                        );

                    p.x_velocity = x_velocity;
                    p.y_velocity = y_velocity;

                    // Re-apply movement with bounced velocity.
                    p.x_pos = (p.x_pos as i64 + i64::from(p.x_velocity)) as u32;
                    p.y_pos = (p.y_pos as i64 + i64::from(p.y_velocity)) as u32;

                    if let (Some(owner), Some(camera)) = (p.owner, camera) {
                        let screen_x = ((p.x_pos >> 16) as i32) - ((camera.x_pos >> 16) as i32);
                        let screen_y = ((p.y_pos >> 16) as i32) - ((camera.y_pos >> 16) as i32);
                        let in_camera_box = (-0x40..=0x180).contains(&screen_x)
                            && (-0x40..=0x120).contains(&screen_y);
                        if !in_camera_box {
                            released_owners.push(owner);
                            return false;
                        }
                    }

                    if !lion_projectile_bounce_iterations_continue_loc_3e2c0(&mut remaining_bounces)
                    {
                        break;
                    }
                }
            }
            true
        });
        self.resolve_drips_splashes(pending_drips_splashes);
        self.drips_surface_crawler_count_loc_fffffb42 = self
            .drips_surface_crawler_count_loc_fffffb42
            .saturating_sub(expired_drips_surface_crawler_finishes);
        for interaction in pending_shooter_layout_interactions {
            self.apply_shooter_object_layout_interaction_loc_F00A(interaction);
        }
        self.projectiles.extend(spawned_projectiles);
        self.projectiles.extend(pending_tar_monster_bursts);
        self.release_lion_projectile_owners(released_owners);
    }

    /// Check if any projectile overlaps the player hitbox, returning true if hit.
    pub(crate) fn projectile_hits_player(&mut self, slice: &OpeningSliceState) -> bool {
        let px = (slice.player.x_pos >> 16) as i32;
        let py = (slice.player.y_pos >> 16) as i32;
        let half = PROJECTILE_HITBOX_HALF;
        let mut hit = false;
        let mut released_owners = Vec::new();
        self.last_projectile_damage_probe = None;
        self.projectiles.retain_mut(|p| {
            if p.kind == EnemyProjectileKind::DripsDrop {
                // ASM Drips.asm loc_3345C only advances the falling child and
                // waits for floor collision `$0C`; live Kid collision starts
                // later when loc_33650 calls loc_33550 for the surface crawler.
                return true;
            }
            if matches!(
                p.kind,
                EnemyProjectileKind::DripsSurfaceCrawlerBurst
                    | EnemyProjectileKind::DripsSurfaceCrawlerFinish
            ) {
                // ASM Drips.asm loc_338F8 allocates Tar_Monster.asm loc_325B6
                // after the crawler has already resolved contact, while
                // loc_3392C keeps the original crawler alive only for its
                // finish animation. Neither performs a fresh Kid damage check.
                return true;
            }
            if p.kind == EnemyProjectileKind::DripsSurfaceCrawler
                && p.drips_surface_crawler_state == DripsSurfaceCrawlerState::StartupLoc335D8
            {
                // ASM Drips.asm loc_335D8 finishes `stru_33FA6` before
                // `st is_moved(a3)` and before loc_33650's live Kid collision
                // path, so the startup shell is visible but not hostile.
                return true;
            }
            if p.friendly {
                return true; // player-fired projectiles don't damage player
            }
            let prx = (p.x_pos >> 16) as i32;
            let pry = (p.y_pos >> 16) as i32;
            let x_overlaps = if p.kind == EnemyProjectileKind::DripsSurfaceCrawler {
                // ASM Drips.asm loc_33650/loc_33550 reads the collision word
                // already produced by GfxObjects_CollisionKid, whose
                // inclusive compares keep issue-260 frame-550's live crawler
                // contact at |dx|=15 active instead of using the generic
                // hostile projectile's strict 14px centre window.
                (prx - px).abs() <= DRIPS_SURFACE_CRAWLER_KID_CONTACT_HALF_WIDTH_LOC_33550
            } else {
                (prx - px).abs() < half + 8
            };
            let overlaps = x_overlaps && (pry - py).abs() < half + 16;
            if overlaps {
                hit = true;
                if self.last_projectile_damage_probe.is_none() {
                    let dx = prx - px;
                    let dy = pry - py;
                    let owner = p.owner.map_or_else(
                        || String::from("none"),
                        |owner| {
                            format!(
                                "{}:{}:{:#04x}",
                                owner.segment_index, owner.layout_index, owner.token
                            )
                        },
                    );
                    self.last_projectile_damage_probe = Some(format!(
                        "hit:{:?}@{:#010x},{:#010x} vel={:#010x},{:#010x} life={} owner={} dx={dx} dy={dy}",
                        p.kind,
                        p.x_pos,
                        p.y_pos,
                        p.x_velocity as u32,
                        p.y_velocity as u32,
                        p.lifetime,
                        owner,
                    ));
                }
                if p.kind == EnemyProjectileKind::LionProjectile {
                    // ASM Lion.asm loc_3E192 -> sub_3E2CE keeps the child
                    // object count-visible for the current collision frame;
                    // native removes and releases parent `$5B` during the
                    // following projectile step when this zero lifetime is
                    // observed.
                    self.lion_projectile_object_only_tail_count = self
                        .lion_projectile_object_only_tail_count
                        .saturating_add(1);
                    p.lifetime = 0;
                    true
                } else if p.kind == EnemyProjectileKind::FireDemonDeathTrail
                    || p.kind == EnemyProjectileKind::FireballTrail
                    || p.kind == EnemyProjectileKind::DragonWalkingBreath
                {
                    // ASM Fire_Demon loc_3A6CA, Fireball.asm loc_33110, and
                    // Dragon.asm loc_3D082: these child hazards keep their own
                    // animation/lifetime loops after hurting Kid; player
                    // collision does not delete the child object.
                    true
                } else {
                    if let Some(owner) = p.owner {
                        released_owners.push(owner);
                    }
                    false // remove projectile on hit
                }
            } else {
                true
            }
        });
        hit |= self.fire_demon_trails.iter().any(|trail| {
            let prx = (trail.x_pos >> 16) as i32;
            let pry = (trail.y_pos >> 16) as i32;
            let overlaps = (prx - px).abs() < half + 8 && (pry - py).abs() < half + 16;
            if overlaps && self.last_projectile_damage_probe.is_none() {
                let dx = prx - px;
                let dy = pry - py;
                self.last_projectile_damage_probe = Some(format!(
                    "hit:FireDemonDeathTrail@{:#010x},{:#010x} life={} dx={dx} dy={dy}",
                    trail.x_pos, trail.y_pos, trail.remaining_frames,
                ));
            }
            overlaps
        });
        self.release_lion_projectile_owners(released_owners);
        hit
    }
}

fn lion_projectile_bounce_iterations_continue_loc_3e2c0(remaining_bounces: &mut i16) -> bool {
    // ASM Lion.asm loc_3E2C0: `dbf d0,loc_3E2A8` (loop if decremented D0 != -1).
    *remaining_bounces = remaining_bounces.wrapping_sub(1);
    *remaining_bounces != -1
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum LionProjectileOutOfRangeLoc3E314 {
    KeepAlive,
    Delete,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_projectile_owner_token_mismatch_loc_3e30e(active_projectile_count: u8) -> u8 {
    // ASM Lion.asm loc_3E30E: when `$71(a5)` does not match `$71(a0)`,
    // decrement the owner's active-projecile counter (`$5B(a0)`) with byte wrap.
    active_projectile_count.wrapping_sub(1)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lion_projectile_screen_range_delete_loc_3e314(
    owner_token_matches: bool,
    in_camera_box: bool,
) -> LionProjectileOutOfRangeLoc3E314 {
    // ASM Lion.asm loc_3E314: this path is reached after owner-match plus
    // the `sub_3E2CE` camera-box test. If outside bounds, delete the
    // projectile.
    if owner_token_matches && !in_camera_box {
        LionProjectileOutOfRangeLoc3E314::Delete
    } else {
        LionProjectileOutOfRangeLoc3E314::KeepAlive
    }
}

#[allow(clippy::manual_range_contains)]
pub(crate) fn lion_projectile_bounce_velocities_loc_3e266(
    x_velocity: i32,
    y_velocity: i32,
    terrain_collision_id: u16,
) -> (i32, i32) {
    // ASM Lion.asm loc_3E266: side collisions (right/left walls) negate `x_vel`,
    // while floor/ceiling collisions (colid_floor/colid_ceiling) negate `y_vel`.
    let mut x_velocity = x_velocity;
    let mut y_velocity = y_velocity;

    if terrain_collision_id == 0x04 || terrain_collision_id == 0x08 {
        x_velocity = lion_projectile_x_reflection_vel_loc_3e2a4(x_velocity);
    } else if terrain_collision_id == 0x0C || terrain_collision_id == 0x10 {
        y_velocity = lion_projectile_y_reflection_vel_loc_3e2c8(y_velocity);
    }

    (x_velocity, y_velocity)
}

fn lion_projectile_x_reflection_vel_loc_3e2a4(x_velocity: i32) -> i32 {
    // ASM Lion.asm loc_3E2A4: negate x_vel on wall collisions.
    -x_velocity
}

fn lion_projectile_y_reflection_vel_loc_3e2c8(y_velocity: i32) -> i32 {
    // ASM Lion.asm loc_3E2C8: negate y_vel on floor/ceiling collisions.
    -y_velocity
}

pub(crate) fn lion_projectile_bounce_velocities_loc_3e29e(
    x_velocity: i32,
    y_velocity: i32,
    terrain_collision_id: u16,
) -> (i32, i32) {
    // ASM Lion.asm loc_3E29E: for `colid_slopeup`, negate `y_vel` before
    // swapping into `x_vel`, then move old `x_vel` to `y_vel`.
    let bounced_x_velocity = if terrain_collision_id == 0x14 {
        -y_velocity
    } else {
        y_velocity
    };
    (bounced_x_velocity, x_velocity)
}

fn terrain_collision_id_from_cell_loc_3e2a8(cell: terrain::Cell) -> u16 {
    match cell.collision {
        terrain::CollisionKind::SlopeUp => 0x14,
        terrain::CollisionKind::SlopeDown => 0x18,
        _ => 0,
    }
}

pub(crate) fn lion_projectile_bounce_velocity_dispatch_loc_3e266(
    h_blocked: bool,
    v_blocked: bool,
    x_velocity: i32,
    y_velocity: i32,
    terrain_collision_id: u16,
) -> (i32, i32) {
    // ASM Lion.asm loc_3E266 dispatch: the four straight-wall ids ($04/$08/
    // $0C/$10) reach the colid_rightwall/leftwall/floor/ceiling reflectors
    // through `loc_3E2A4`/`loc_3E2C8`; slope ids ($14/$18) skip the
    // reflectors and run the swap at `loc_3E29E`. Native must additionally
    // collapse simultaneous h+v probe blockings into a corner-flip
    // ((-x_vel, -y_vel)) because, unlike the ASM where `d4` always names
    // a single contact face, the native probe can stamp both axes at once.
    // Without that branch, both axes would otherwise feed through the
    // single-face reflector and one component would be left untouched.
    if h_blocked && v_blocked {
        (-x_velocity, -y_velocity)
    } else if matches!(terrain_collision_id, 0x04 | 0x08 | 0x0C | 0x10) {
        lion_projectile_bounce_velocities_loc_3e266(x_velocity, y_velocity, terrain_collision_id)
    } else {
        lion_projectile_bounce_velocities_loc_3e29e(x_velocity, y_velocity, terrain_collision_id)
    }
}

pub(crate) fn lion_projectile_terrain_collision_id_loc_3e260(
    h_blocked: bool,
    v_blocked: bool,
    x_velocity: i32,
    y_velocity: i32,
    cell: terrain::Cell,
) -> u16 {
    // ASM Lion.asm loc_3E266: branches off `d4` (collision_type) into the
    // straight-wall reflectors at $04 (colid_rightwall) / $08 (colid_leftwall)
    // / $0C (colid_floor) / $10 (colid_ceiling) and the slope swap at $14/$18.
    // Native does not have a hardware collision probe filling `d4`, so the
    // bounce loop derives the id from which probe cell blocked the move and
    // the projectile's velocity sign: a rightward-moving projectile that
    // hits the horizontal probe must have run into a right wall, etc. The
    // sign comparison must use `>= 0`, not `< 0`, so that a stationary axis
    // (vel == 0) still classifies as the "forward" wall — matching the ASM
    // where `d4` already encodes the contact face.
    if h_blocked && v_blocked {
        0
    } else if h_blocked {
        if x_velocity >= 0 { 0x04 } else { 0x08 }
    } else if v_blocked {
        if y_velocity >= 0 { 0x0C } else { 0x10 }
    } else {
        terrain_collision_id_from_cell_loc_3e2a8(cell)
    }
}

#[cfg(test)]
mod loc_3e266_tests {
    #[test]
    fn loc_3e2a4_reflection_negates_x_velocity() {
        assert_eq!(
            super::lion_projectile_x_reflection_vel_loc_3e2a4(0x2_0000),
            -0x2_0000,
            "ASM Lion.asm loc_3E2A4 must negate `x_vel`"
        );
    }

    #[test]
    fn loc_3e266_negates_x_velocity_on_wall_collision() {
        let (x_velocity, y_velocity) =
            super::lion_projectile_bounce_velocities_loc_3e266(0x2_0000, 0x1_0000, 0x0004);

        assert_eq!(
            x_velocity, -0x2_0000,
            "ASM Lion.asm loc_3E266 should negate x_vel for colid_rightwall"
        );
        assert_eq!(
            y_velocity, 0x1_0000,
            "Other axis must remain untouched at loc_3E266"
        );
    }

    #[test]
    fn loc_3e266_negates_y_velocity_on_floor_collision() {
        let (x_velocity, y_velocity) =
            super::lion_projectile_bounce_velocities_loc_3e266(0x1_0000, 0x2_0000, 0x000C);

        assert_eq!(
            x_velocity, 0x1_0000,
            "x_vel must remain unchanged for colid_floor at loc_3E266"
        );
        assert_eq!(
            y_velocity, -0x2_0000,
            "ASM Lion.asm loc_3E266 should negate y_vel for colid_floor"
        );
    }

    #[test]
    fn loc_3e266_leaves_velocities_untouched_for_slope_collision_id_14() {
        // ASM Lion.asm loc_3E266 only mirrors an axis when the collided
        // face matches one of the four straight wall/floor/ceiling collision
        // ids ($04 rightwall, $08 leftwall, $0C floor, $10 ceiling). Slope
        // ids ($14/$18) are routed through `lion_projectile_bounce_velocities_loc_3e29e`
        // via the `loc_3E2A8` lookup instead, so loc_3E266 must leave both
        // velocities alone. A `replace == with !=` mutation on the ceiling
        // check (`id == $10`) would otherwise let any non-floor id flip y_vel
        // before the slope swap had a chance to run.
        let (x_velocity, y_velocity) =
            super::lion_projectile_bounce_velocities_loc_3e266(0x3_0000, 0x4_0000, 0x0014);

        assert_eq!(
            (x_velocity, y_velocity),
            (0x3_0000, 0x4_0000),
            "ASM Lion.asm loc_3E266: a slope (`$14`) collision id must leave both velocities untouched"
        );
    }

    #[test]
    fn terrain_collision_id_from_cell_loc_3e2a8_returns_slope_down_for_slope_down_cell() {
        // ASM Lion.asm loc_3E2A8: the bounce-classification probe maps
        // `colid_slopedown` to $18 so the caller routes through
        // `lion_projectile_bounce_velocities_loc_3e29e` for the slope swap.
        // Without an explicit SlopeDown match arm the function silently
        // falls through to `_ => 0` and downhill cells never reach the
        // swap path, leaving the projectile to plough through them with
        // raw, unmodified velocity.
        let cell = super::terrain::Cell {
            has_block: true,
            collision: super::terrain::CollisionKind::SlopeDown,
            block_type: super::terrain::BlockType::None,
            skin: 0,
        };
        assert_eq!(
            super::terrain_collision_id_from_cell_loc_3e2a8(cell),
            0x0018,
            "ASM Lion.asm loc_3E2A8: SlopeDown cells must map to colid_slopedown ($18)"
        );
    }

    fn empty_cell() -> super::terrain::Cell {
        super::terrain::Cell {
            has_block: false,
            collision: super::terrain::CollisionKind::Empty,
            block_type: super::terrain::BlockType::None,
            skin: 0,
        }
    }

    #[test]
    fn loc_3e260_h_blocked_with_rightward_velocity_yields_colid_rightwall() {
        // ASM Lion.asm loc_3E266: `cmpi.w #4,d4 / beq.s loc_3E2A4` selects the
        // x-axis reflector for `colid_rightwall` ($04). Native must classify
        // a horizontally-blocked, right-moving projectile as $04 so the bounce
        // loop reaches `lion_projectile_bounce_velocities_loc_3e266` and the
        // x-axis is the one that flips. The sign test is `>= 0`, not `< 0`,
        // so a +x velocity must produce $04, not $08.
        assert_eq!(
            super::lion_projectile_terrain_collision_id_loc_3e260(
                true,
                false,
                0x0002_0000,
                0,
                empty_cell(),
            ),
            0x0004,
            "ASM Lion.asm loc_3E266: h_blocked + x_vel >= 0 must map to colid_rightwall ($04)"
        );
    }

    #[test]
    fn loc_3e260_h_blocked_with_leftward_velocity_yields_colid_leftwall() {
        // ASM Lion.asm loc_3E266: `cmpi.w #8,d4 / beq.s loc_3E2A4` is the
        // mirror branch for `colid_leftwall` ($08). A horizontally-blocked
        // projectile with negative x_vel must classify as $08 so the bounce
        // table still negates x_vel via the same reflector — a `>= 0` -> `< 0`
        // mutation would invert the assignment and emit $04 here, breaking
        // the sign-correlated id even though the resulting velocity flip is
        // identical for both walls.
        assert_eq!(
            super::lion_projectile_terrain_collision_id_loc_3e260(
                true,
                false,
                -0x0002_0000,
                0,
                empty_cell(),
            ),
            0x0008,
            "ASM Lion.asm loc_3E266: h_blocked + x_vel < 0 must map to colid_leftwall ($08)"
        );
    }

    #[test]
    fn loc_3e260_v_blocked_with_downward_velocity_yields_colid_floor() {
        // ASM Lion.asm loc_3E266: `cmpi.w #$C,d4 / beq.w loc_3E2C8` routes a
        // floor contact ($0C) through the y-axis reflector. Native infers $0C
        // when the vertical probe is blocked and y_vel >= 0 (downward in
        // screen coordinates).
        assert_eq!(
            super::lion_projectile_terrain_collision_id_loc_3e260(
                false,
                true,
                0,
                0x0002_0000,
                empty_cell(),
            ),
            0x000C,
            "ASM Lion.asm loc_3E266: v_blocked + y_vel >= 0 must map to colid_floor ($0C)"
        );
    }

    #[test]
    fn loc_3e260_v_blocked_with_upward_velocity_yields_colid_ceiling() {
        // ASM Lion.asm loc_3E266: `cmpi.w #$10,d4 / beq.w loc_3E2C8` is the
        // ceiling branch ($10). Negative y_vel (rising) must classify as $10
        // — `>= 0` -> `< 0` would emit $0C here, swapping floor and ceiling
        // ids while still bouncing y_vel; only the id distinction catches
        // the mutation.
        assert_eq!(
            super::lion_projectile_terrain_collision_id_loc_3e260(
                false,
                true,
                0,
                -0x0002_0000,
                empty_cell(),
            ),
            0x0010,
            "ASM Lion.asm loc_3E266: v_blocked + y_vel < 0 must map to colid_ceiling ($10)"
        );
    }

    #[test]
    fn loc_3e260_corner_block_yields_zero_collision_id() {
        // ASM Lion.asm loc_3E266: when both probes report contact, native
        // returns $00 so the bounce-velocity dispatcher hits the corner-case
        // arm at `step_projectiles` (`(-x_vel, -y_vel)`) instead of running
        // the `loc_3E266` table. Any non-corner id here would push the
        // projectile through `lion_projectile_bounce_velocities_loc_3e266`
        // and skip the dual-axis flip.
        assert_eq!(
            super::lion_projectile_terrain_collision_id_loc_3e260(
                true,
                true,
                0x0002_0000,
                0x0002_0000,
                empty_cell(),
            ),
            0,
            "ASM Lion.asm loc_3E266: simultaneous h+v blocking must yield $00 so the corner branch runs"
        );
    }

    #[test]
    fn loc_3e266_dispatch_corner_block_negates_both_velocities_with_exact_values() {
        // ASM Lion.asm loc_3E266: when both probes report contact native
        // routes through the corner branch, which flips both axes in a
        // single dispatch step. The bounce loop's gameplay test
        // (`wall_bounce_corner_negates_both_velocities`) cannot pin down
        // either negation in isolation because the next loop iteration
        // re-classifies the now-mirrored axis as a single wall and runs
        // through `lion_projectile_x_reflection_vel_loc_3e2a4` again,
        // independently restoring the missing flip. Pin the corner
        // dispatch with exact values so a `delete -` mutation on either
        // axis fails immediately.
        assert_eq!(
            super::lion_projectile_bounce_velocity_dispatch_loc_3e266(
                true,
                true,
                0x0002_0000,
                0x0003_0000,
                0,
            ),
            (-0x0002_0000, -0x0003_0000),
            "ASM Lion.asm loc_3E266 corner branch must negate both velocities — \
             deleting the `-` on either axis would leave the original sign intact"
        );
    }

    #[test]
    fn loc_3e266_dispatch_straight_wall_id_routes_through_loc_3e266_reflector() {
        // ASM Lion.asm loc_3E266: id $04 must reach the x-axis reflector
        // and leave y_vel untouched. A `&& -> ||` mutation on the corner
        // condition would force the corner branch even with v_blocked=false
        // and flip y_vel here.
        assert_eq!(
            super::lion_projectile_bounce_velocity_dispatch_loc_3e266(
                true,
                false,
                0x0002_0000,
                0x0003_0000,
                0x0004,
            ),
            (-0x0002_0000, 0x0003_0000),
            "ASM Lion.asm loc_3E266: a single-axis hit must run through `loc_3E2A4`/`loc_3E2C8`, not the corner flip"
        );
    }

    #[test]
    fn loc_3e266_dispatch_slope_id_routes_through_loc_3e29e_swap() {
        // ASM Lion.asm loc_3E29E: slope ids ($14/$18) bypass the straight-wall
        // reflectors and run the swap (-y -> x, x -> y for slope-up).
        assert_eq!(
            super::lion_projectile_bounce_velocity_dispatch_loc_3e266(
                false,
                false,
                0x0003_0000,
                0x0004_0000,
                0x0014,
            ),
            (-0x0004_0000, 0x0003_0000),
            "ASM Lion.asm loc_3E29E: slope id $14 must reach the swap branch, not the straight-wall reflector"
        );
    }

    #[test]
    fn loc_3e260_unblocked_falls_back_to_cell_collision() {
        // ASM Lion.asm loc_3E2A8: when neither probe reports an axis-aligned
        // hit, the bounce loop classifies the contact via the cell at the
        // projectile's centre, so a slope-up cell must still emit $14 and
        // route through `lion_projectile_bounce_velocities_loc_3e29e`.
        let cell = super::terrain::Cell {
            has_block: true,
            collision: super::terrain::CollisionKind::SlopeUp,
            block_type: super::terrain::BlockType::None,
            skin: 0,
        };
        assert_eq!(
            super::lion_projectile_terrain_collision_id_loc_3e260(
                false,
                false,
                0x0001_0000,
                0x0001_0000,
                cell,
            ),
            0x0014,
            "ASM Lion.asm loc_3E2A8: unblocked probes must fall through to the cell-based slope id"
        );
    }

    #[test]
    fn loc_3e2c8_reflection_negates_y_velocity() {
        assert_eq!(
            super::lion_projectile_y_reflection_vel_loc_3e2c8(0x2_0000),
            -0x2_0000,
            "ASM Lion.asm loc_3E2C8 must negate `y_vel`"
        );
    }

    #[test]
    fn loc_3e29e_swaps_velocities_with_slope_flip() {
        let (x_velocity, y_velocity) =
            super::lion_projectile_bounce_velocities_loc_3e29e(0x3_0000, 0x4_0000, 0x0014);

        assert_eq!(
            x_velocity, -0x4_0000,
            "ASM Lion.asm loc_3E29E should negate `y_vel` into `x_vel` for `colid_slopeup`"
        );
        assert_eq!(
            y_velocity, 0x3_0000,
            "ASM Lion.asm loc_3E29E should move old `x_vel` into `y_vel`"
        );
    }

    #[test]
    fn loc_3e29e_uses_unmodified_y_for_other_collision_ids() {
        let (x_velocity, y_velocity) =
            super::lion_projectile_bounce_velocities_loc_3e29e(0x3_0000, 0x4_0000, 0x0018);

        assert_eq!(
            x_velocity, 0x4_0000,
            "ASM Lion.asm loc_3E29E uses raw `y_vel` when `colid_slopedown` is not slope-up"
        );
        assert_eq!(
            y_velocity, 0x3_0000,
            "ASM Lion.asm loc_3E29E still copies old `x_vel` into `y_vel`"
        );
    }

    #[test]
    fn loc_3e2c0_dbf_continues_for_sixteen_iterations() {
        // ASM Lion.asm loc_3E2C0: `dbf d0,loc_3E2A8` with d0 seeded to #$F at
        // the loop entry — `dbf` decrements then branches while d0 != -1, so
        // the loop is bounded to exactly 16 iterations. The hard cap inside
        // the loop body pins that ROM bound: any mutation that turns the
        // helper into an unconditional `true` (or otherwise drops the
        // decrement-and-exit contract) blows past 16 and trips the assertion
        // instead of spinning the test runner into a cargo-mutants timeout.
        let mut remaining_bounces = 0x0Fi16;
        let mut iterations = 0i32;
        loop {
            iterations += 1;
            assert!(
                iterations <= 16,
                "ASM Lion.asm loc_3E2C0: `dbf d0` with d0=$F must exit after \
                 exactly 16 iterations; the helper failed to decrement or \
                 return false at the -1 edge."
            );
            if !super::lion_projectile_bounce_iterations_continue_loc_3e2c0(&mut remaining_bounces)
            {
                break;
            }
        }
        assert_eq!(iterations, 16);
        assert_eq!(remaining_bounces, -1);
    }

    #[test]
    fn loc_3e2c0_returns_false_exactly_when_decremented_counter_is_minus_one() {
        // ASM Lion.asm loc_3E2C0: `dbf d0,loc_3E2A8` — `dbf` decrements d0 then
        // branches if d0 != -1. A single call with `d0 = 0` must therefore
        // decrement to -1 and return `false`. The companion `dbf_continues`
        // test covers the iteration count from d0=$0F; this one pins the
        // exact decrement-and-branch contract for both the exit edge (d0=0
        // -> false, remaining -> -1) and the continue edge (d0=1 -> true,
        // remaining -> 0) so any single-call mutation on the helper (sign
        // flip, off-by-one comparison, swapped operands) fails fast against
        // the asserted ground truth instead of relying on the outer loop's
        // count-equals-16 invariant.
        let mut remaining_bounces = 0i16;
        let kept =
            super::lion_projectile_bounce_iterations_continue_loc_3e2c0(&mut remaining_bounces);
        assert!(
            !kept,
            "ASM Lion.asm loc_3E2C0: `dbf d0` with d0=0 decrements to -1 and \
             must NOT branch back to loc_3E2A8 — returning `true` here would \
             leave the bounce loop spinning forever."
        );
        assert_eq!(remaining_bounces, -1);

        let mut remaining_bounces = 1i16;
        let kept =
            super::lion_projectile_bounce_iterations_continue_loc_3e2c0(&mut remaining_bounces);
        assert!(
            kept,
            "ASM Lion.asm loc_3E2C0: `dbf d0` with d0=1 decrements to 0 and \
             must branch back to loc_3E2A8 for one more bounce attempt."
        );
        assert_eq!(remaining_bounces, 0);
    }

    #[test]
    fn loc_3e30e_decrements_owner_projectile_count_with_wrap() {
        assert_eq!(
            super::lion_projectile_owner_token_mismatch_loc_3e30e(4),
            3,
            "ASM Lion.asm loc_3E30E should reduce a live owner count by one"
        );
        assert_eq!(
            super::lion_projectile_owner_token_mismatch_loc_3e30e(0),
            255,
            "ASM Lion.asm loc_3E30E is byte arithmetic and wraps on underflow"
        );
    }

    #[test]
    fn loc_3e314_deletes_when_owner_token_matches_and_projectile_is_offscreen() {
        assert_eq!(
            super::lion_projectile_screen_range_delete_loc_3e314(true, false),
            super::LionProjectileOutOfRangeLoc3E314::Delete,
            "ASM Lion.asm loc_3E314 should delete only when owner matches and the object exits the camera box"
        );
        assert_eq!(
            super::lion_projectile_screen_range_delete_loc_3e314(true, true),
            super::LionProjectileOutOfRangeLoc3E314::KeepAlive,
            "in-range owner-matched projectiles should stay alive"
        );
        assert_eq!(
            super::lion_projectile_screen_range_delete_loc_3e314(false, false),
            super::LionProjectileOutOfRangeLoc3E314::KeepAlive,
            "owner mismatch routes through loc_3E30E first, not loc_3E314"
        );
    }
}
