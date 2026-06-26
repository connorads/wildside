use super::*;

fn active_platform_handle_sub_2728(layout_offset_words: u16) -> ActivePlatformHandleSub2728 {
    ActivePlatformHandleSub2728 {
        layout_offset_words,
    }
}

fn manifest_with_static_tornado() -> AssetManifest {
    let mut m = manifest();
    if let Some(enemy) = m.levels[0].enemy_layout_metadata.as_mut() {
        enemy.objects[0] = wildside_data::EnemyLayoutObject {
            object_type: 0x18,
            flags: 0,
            hit_points_plus_one: 1,
            x: 0x0750,
            y: 0x00E0,
        };
    }
    m
}

fn manifest_with_nearby_skull() -> AssetManifest {
    let mut m = manifest();
    if let Some(header) = m.levels[0].header_metadata.as_mut() {
        header.player_start_x = 32;
        header.player_start_y = 224;
    }
    if let Some(enemy) = m.levels[0].enemy_layout_metadata.as_mut() {
        enemy.objects = vec![wildside_data::EnemyLayoutObject {
            object_type: 0x1C,
            flags: 0,
            hit_points_plus_one: 1,
            x: 96,
            y: 224,
        }];
        enemy.object_count = 1;
    }
    m
}

fn setup_stomp_scenario(runtime: &mut NativeRuntime, stomp: bool) {
    if let Some(ref mut slice) = runtime.opening_slice {
        // ASM Mini_Hopping_Skull.asm loc_3E9D0 gates activation on Kid being
        // level with or below the layout point before the live skull spawns.
        slice.player.x_pos = 0x0120_0000;
        slice.player.y_pos = 0x00E0_0000;
        slice.camera.x_pos = 0;
        slice.camera.y_pos = 0;
    }
    runtime.step_frame(FrameInput::default());
    if let Some(ref mut slice) = runtime.opening_slice {
        slice.player.x_pos = 0x0120_0000;
        slice.camera.x_pos = 0;
        slice.camera.y_pos = 0;
        if stomp {
            slice.player.y_pos = 0x00D8_0000;
            slice.player.y_velocity = 0x0002_0000;
            slice.player.on_ground = false;
        } else {
            slice.player.y_pos = 0x00E0_0000;
            slice.player.y_velocity = 0;
            slice.player.on_ground = true;
        }
    }
    // The activation teleport overlapped the enemy, arming the ASM
    // $FFFFFA74/$FFFFFA75 sustained-contact latch (kid.asm:936/3383) which
    // would deny the stomp as a non-fresh contact. Clear it so the staged
    // pose lands as a first-contact frame, as it would in real play.
    runtime.kid_touched_enemy_last_frame_loc_fa74 = false;
    runtime.kid_touched_enemy_this_frame_loc_fa75 = false;
}

fn manifest_with_tough_skull() -> AssetManifest {
    let mut m = manifest();
    if let Some(enemy) = m.levels[0].enemy_layout_metadata.as_mut() {
        enemy.objects[0] = wildside_data::EnemyLayoutObject {
            object_type: 0x1C,
            flags: 0,
            hit_points_plus_one: 2,
            x: 0x0120,
            y: 0x00E0,
        };
    }
    m
}

fn setup_stomp_against_first_active_enemy(runtime: &mut NativeRuntime) {
    let (enemy_x_pos, enemy_y_pos) = runtime.active_enemies.first().map_or_else(
        || panic!("expected an active enemy"),
        |enemy| (enemy.x_pos, enemy.y_pos),
    );

    if let Some(ref mut slice) = runtime.opening_slice {
        slice.player.x_pos = enemy_x_pos;
        slice.player.y_pos = enemy_y_pos.saturating_sub(0x0008_0000);
        slice.player.y_velocity = 0x0002_0000;
        slice.player.on_ground = false;
        slice.camera.x_pos = 0;
        slice.camera.y_pos = 0;
    }
    // See setup_stomp_scenario: the staged pose must read as a fresh contact,
    // so disarm the FA74/FA75 sustained-contact latch left by setup frames.
    runtime.kid_touched_enemy_last_frame_loc_fa74 = false;
    runtime.kid_touched_enemy_this_frame_loc_fa75 = false;
}

mod chunk_04 {
    use super::*;

    #[test]
    fn trace_frame_reports_prize_diamond_frontier_debug_fields() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = 96 << 16;
        slice.player.y_pos = 208 << 16;
        runtime.opening_slice = Some(slice);
        runtime.pending_prize_diamond_awards = vec![31, 4];
        runtime.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: i64::from(201 << 16),
            y_velocity: 0,
            remaining: 144,
            spawn_delay: 0,
            level_col: 0,
            level_row: 0,
            first_row_grace: false,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });

        let trace = runtime.trace_frame();
        assert_eq!(
            trace
                .gameplay_debug_fields
                .get("pending_prize_diamond_awards"),
            Some(&"[31,4]".to_string()),
            "the trace should expose the live queued diamond-award delays",
        );

        let candidate_debug = trace
            .gameplay_debug_fields
            .get("floating_diamond_contact_candidates")
            .unwrap_or_else(|| {
                panic!("trace frame should emit floating diamond contact diagnostics")
            });
        assert!(
            candidate_debug.contains("diamond@100,201"),
            "the floating-diamond trace should report the diamond position in pixels",
        );
        assert!(
            candidate_debug.contains("overlap_x=1"),
            "the floating-diamond trace should report overlap information from the pickup box",
        );
    }

    #[test]
    fn opening_level_one_object_counts_at_grounded_camera() {
        assert_eq!(
            opening_level_one_object_counts(
                LevelId(0),
                2,
                429u32 << 16,
                OPENING_GROUNDED_CAMERA_Y_FP,
            ),
            Some((4, 5))
        );
        assert_eq!(
            opening_level_one_object_counts(LevelId(0), 2, 0, OPENING_GROUNDED_CAMERA_Y_FP),
            Some((4, 5))
        );
        assert_eq!(
            opening_level_one_object_counts(
                LevelId(1),
                2,
                429u32 << 16,
                OPENING_GROUNDED_CAMERA_Y_FP
            ),
            None
        );
        assert_eq!(
            opening_level_one_object_counts(LevelId(1), 3, 0, 0x0014_0000),
            Some((4, 4)),
            "ASM loc_74E0 level-skip resume / oracle full-frame Level 0x01: player + first RockTank pair reports Number_Objects=4 at the settled opening camera",
        );
    }

    #[test]
    fn opening_level_one_object_counts_pin_camera_boundary_branches() {
        assert_eq!(
            opening_level_one_object_counts(LevelId(2), 2, 0, OPENING_GROUNDED_CAMERA_Y_FP),
            None,
            "only opening Level 1 and the Level 2 resume special case use these trace object-count overrides",
        );
        assert_eq!(
            opening_level_one_object_counts(
                LevelId(0),
                2,
                0x0200_0000,
                OPENING_GROUNDED_CAMERA_Y_FP
            ),
            Some((4, 5)),
            "camera_x == $0200.0000 is still inside the opening Level 1 trace-count window",
        );
        assert_eq!(
            opening_level_one_object_counts(
                LevelId(0),
                2,
                0x0200_0001,
                OPENING_GROUNDED_CAMERA_Y_FP
            ),
            None,
            "camera_x one subpixel past $0200.0000 leaves the opening Level 1 trace-count window",
        );
        assert_eq!(
            opening_level_one_object_counts(
                LevelId(0),
                2,
                0,
                OPENING_GROUNDED_CAMERA_Y_FP + 0x0004_0000
            ),
            Some((4, 5)),
            "camera_y at the grounded threshold keeps the regular zero-camera count split",
        );
        assert_eq!(
            opening_level_one_object_counts(
                LevelId(0),
                2,
                0,
                OPENING_GROUNDED_CAMERA_Y_FP + 0x0004_0001
            ),
            Some((4, 4)),
            "camera_y one subpixel beyond the grounded threshold uses the fully hidden count split",
        );
        assert_eq!(
            opening_level_one_object_counts(
                LevelId(0),
                3,
                0,
                OPENING_GROUNDED_CAMERA_Y_FP + 0x0004_0001
            ),
            Some((5, 5)),
            "the fully hidden zero-camera branch adds two system objects to arbitrary roster counts",
        );
        assert_eq!(
            opening_level_one_object_counts(LevelId(0), 3, 0, OPENING_GROUNDED_CAMERA_Y_FP),
            Some((5, 6)),
            "the regular zero-camera branch adds two objects and three gfx objects to arbitrary roster counts",
        );
        assert_eq!(
            opening_level_one_object_counts(
                LevelId(0),
                3,
                0x0001_0000,
                OPENING_GROUNDED_CAMERA_Y_FP
            ),
            Some((5, 6)),
            "the scrolling opening branch adds two objects and three gfx objects to arbitrary roster counts",
        );
    }

    #[test]
    fn platform_script_nothing_all_zero_record_is_not_active() {
        let platform = PlatformPlacement {
            x: 0,
            y: 0,
            buffer_left: 0,
            buffer_right: 0,
            buffer_top: 0,
            buffer_bottom: 0,
            kind: 0,
            subtype: 0,
            h_variant: 0,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let window = ActivationWindow {
            x_min: 0,
            x_max: 320,
            y_min: 0,
            y_max: 224,
        };

        assert!(
            !platform_is_active(&platform, window),
            "Generated platform/00.asm's all-zero PlatformScript_Nothing record is a no-op sentinel before `$FFFF`, not an allocated Level 0x01 platform object",
        );
    }

    #[test]
    fn hopper_lands_on_terrain_before_launching() {
        let grid = grid_with_floor_at_row(17);
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::MiniHoppingSkull,
            x_pos: 96u32 << 16,
            y_pos: 224u32 << 16,
            x_velocity: 0,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };

        let mut grounded_skull = None;
        for _ in 0..80 {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                None,
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
                &mut Vec::new(),
            );
            if enemy.on_ground {
                grounded_skull = Some(enemy);
                break;
            }
        }

        let skull = grounded_skull.expect("hopper should have landed before relaunching");
        let floor_y_fp = 272u32 << 16;
        assert_eq!(skull.y_pos, floor_y_fp, "hopper should be on floor surface");
    }

    #[test]
    fn hopper_launches_into_an_air_arc() {
        let grid = grid_with_floor_at_row(17);
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::MiniHoppingSkull,
            x_pos: 96u32 << 16,
            y_pos: 224u32 << 16,
            x_velocity: 0,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };

        let mut grounded_skull = None;
        for _ in 0..80 {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                None,
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
                &mut Vec::new(),
            );
            if enemy.on_ground {
                grounded_skull = Some(enemy);
                break;
            }
        }

        let grounded_skull = grounded_skull.expect("skull should settle on the floor first");
        assert!(
            grounded_skull.on_ground,
            "skull should be grounded before launch"
        );

        for _ in 0..MINI_SKULL_GROUND_DELAY_FRAMES {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                None,
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
                &mut Vec::new(),
            );
        }
        assert!(
            enemy.on_ground,
            "hopper should remain grounded through the disassembly-backed wait window"
        );

        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut Vec::new(),
        );
        for _ in 0..12 {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                None,
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
                &mut Vec::new(),
            );
        }

        let airborne_skull = enemy;
        assert!(
            !airborne_skull.on_ground,
            "hopper should have left the floor after the grounded delay"
        );
        assert!(
            airborne_skull.x_velocity != 0,
            "hopper should carry horizontal velocity during the leap"
        );
        assert!(
            airborne_skull.y_velocity < 0,
            "hopper should still be rising during the first airborne frames"
        );
    }

    #[test]
    fn hopper_hp2_launches_with_boosted_velocity() {
        let grid = grid_with_floor_at_row(17);
        let floor_y_fp = 272u32 << 16;
        // HP 2 = enemy_level 1 → should get boosted X and Y velocity
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::MiniHoppingSkull,
            x_pos: 96u32 << 16,
            y_pos: floor_y_fp,
            x_velocity: 0,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: true,
            hit_points: 2,
            death_animation_active: false,
            grounded_frames: 0, // ready to launch immediately
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut Vec::new(),
        );
        // Mini_Hopping_Skull.asm: HP>1 boosts X by $4000 and Y by -$10000
        let base_x = MINI_SKULL_JUMP_X_SPEED_FP;
        let boosted_x = base_x + 0x4000;
        let base_y = MINI_SKULL_JUMP_Y_SPEED_FP;
        let boosted_y = base_y - 0x1_0000;
        assert_eq!(
            enemy.x_velocity, boosted_x,
            "HP2 skull should hop with boosted X velocity"
        );
        assert_eq!(
            enemy.y_velocity.signum(),
            -1,
            "HP2 skull should have upward Y velocity"
        );
        // The exact y_velocity after one frame includes gravity
        // but the initial value was boosted_y + gravity
        let expected_y_after_gravity = boosted_y + MINI_SKULL_GRAVITY_FP;
        assert_eq!(
            enemy.y_velocity, expected_y_after_gravity,
            "HP2 skull Y velocity = boosted initial + 1 frame gravity"
        );
    }

    #[test]
    fn mini_hopping_skull_restarts_wait_after_facing_flip_loc_3f090() {
        let grid = grid_with_floor_at_row(17);
        let floor_y_fp = 272u32 << 16;
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::MiniHoppingSkull,
            x_pos: 96u32 << 16,
            y_pos: floor_y_fp,
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
            grounded_frames: MINI_SKULL_GROUND_DELAY_FRAMES,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };

        for _ in 0..MINI_SKULL_GROUND_DELAY_FRAMES {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                Some(128u32 << 16),
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
                &mut Vec::new(),
            );
        }
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            Some(128u32 << 16),
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut Vec::new(),
        );

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Mini_Hopping_Skull.asm loc_3F090 should flip toward Kid only after loc_3F000's wait expires"
        );
        assert_eq!(
            enemy.x_velocity, 0,
            "ASM loc_3F090 branches back to loc_3F000 instead of launching on the flip frame"
        );
        assert_eq!(
            enemy.grounded_frames, MINI_SKULL_GROUND_DELAY_FRAMES,
            "ASM loc_3F090 restarts the `$14` stopped-wait counter after changing x_direction"
        );

        for _ in 0..MINI_SKULL_GROUND_DELAY_FRAMES {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                Some(128u32 << 16),
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
                &mut Vec::new(),
            );
        }
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            Some(128u32 << 16),
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut Vec::new(),
        );

        assert_eq!(
            enemy.x_velocity, 0,
            "ASM Mini_Hopping_Skull.asm loc_3F09E seeds an 11-frame matched-facing pause before loc_3F112 can relaunch"
        );
        assert_ne!(
            enemy.wait_timer, 0,
            "native wait_timer carries the live loc_3F09E matched-facing pause"
        );

        for _ in 0..MINI_SKULL_FACING_MATCHED_PAUSE_FRAMES_LOC_3F09E {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                Some(128u32 << 16),
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
                &mut Vec::new(),
            );
            assert_eq!(
                enemy.x_velocity, 0,
                "ASM loc_3F0A8 hibernates through the matched-facing pause"
            );
        }

        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            Some(128u32 << 16),
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut Vec::new(),
        );

        assert!(
            enemy.x_velocity > 0,
            "ASM Mini_Hopping_Skull.asm loc_3F112/loc_3F146 should launch right after the matched-facing pause expires"
        );
    }

    #[test]
    fn mini_hopping_skull_does_not_edge_reverse_before_launch_loc_3f090() {
        let grid = grid_with_platform(17, 3, 10);
        let floor_y_fp = 272u32 << 16;
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::MiniHoppingSkull,
            x_pos: 48u32 << 16,
            y_pos: floor_y_fp,
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
            grounded_frames: MINI_SKULL_GROUND_DELAY_FRAMES,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };

        for _ in 0..MINI_SKULL_GROUND_DELAY_FRAMES {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                Some(32u32 << 16),
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
                &mut Vec::new(),
            );
        }
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            Some(32u32 << 16),
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut Vec::new(),
        );

        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM Mini_Hopping_Skull.asm loc_3F090 repicks from Kid X before launch instead of using generic platform-edge lookahead"
        );
        assert_eq!(
            enemy.x_velocity, 0,
            "ASM Mini_Hopping_Skull.asm loc_3F09E pauses when x_direction already faces Kid"
        );

        for _ in 0..MINI_SKULL_FACING_MATCHED_PAUSE_FRAMES_LOC_3F09E {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                Some(&grid),
                Some(32u32 << 16),
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
                &mut Vec::new(),
            );
        }
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            Some(&grid),
            Some(32u32 << 16),
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut Vec::new(),
        );

        assert!(
            enemy.x_velocity < 0,
            "Mini Hopping Skull should launch toward the Kid even when the platform ends in that direction"
        );
    }

    #[test]
    fn hopper_reverses_on_wall_collision() {
        let mut runtime = NativeRuntime::new(manifest_with_nearby_skull());
        runtime.set_block_grid(grid_with_wall(17, 0, 20, 4));
        start_level(&mut runtime);

        for _ in 0..100 {
            runtime.step_frame(FrameInput::default());
        }

        let mut ever_walked_right = false;
        for _ in 0..200 {
            runtime.step_frame(FrameInput::default());
            if let Some(skull) = runtime
                .active_enemies
                .iter()
                .find(|e| e.family == EnemyFamily::MiniHoppingSkull)
                && skull.x_velocity > 0
            {
                ever_walked_right = true;
                break;
            }
        }
        assert!(
            ever_walked_right,
            "hopper should reverse direction on wall hit"
        );
    }

    #[test]
    fn hopper_does_not_cross_a_wall() {
        let mut runtime = NativeRuntime::new(manifest_with_nearby_skull());
        runtime.set_block_grid(grid_with_wall(17, 0, 20, 4));
        start_level(&mut runtime);

        let wall_right_edge_px = (4 + 1) * 16; // 80px
        let wall_limit_fp = ((wall_right_edge_px + ENEMY_PROBE_HALF_WIDTH) as u32) << 16;

        for _ in 0..300 {
            runtime.step_frame(FrameInput::default());
            if let Some(skull) = runtime
                .active_enemies
                .iter()
                .find(|e| e.family == EnemyFamily::MiniHoppingSkull)
            {
                assert!(
                    skull.x_pos >= wall_limit_fp.saturating_sub(MINI_SKULL_JUMP_X_SPEED_FP as u32),
                    "hopper should not cross the wall. x_pos={:#010X}, wall_limit={:#010X}",
                    skull.x_pos,
                    wall_limit_fp,
                );
            }
        }
    }

    #[test]
    fn enemy_snapshots_include_layout_identity_and_motion_fields() {
        let mut runtime = NativeRuntime::new(manifest_with_nearby_skull());
        runtime.set_block_grid(grid_with_floor_at_row(17));
        start_level(&mut runtime);
        for _ in 0..80 {
            runtime.step_frame(FrameInput::default());
        }

        let frame = runtime.trace_frame();
        let skull = frame
            .active_objects
            .iter()
            .find(|object| matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x1C }))
            .expect("skull snapshot should be present");
        assert_eq!(skull.layout_index, Some(0));
        assert!(skull.x_velocity.is_some());
        assert!(skull.y_velocity.is_some());
        assert!(skull.facing.is_some());
        assert!(skull.on_ground.is_some());
        assert_eq!(skull.hit_points, Some(1));
    }

    #[test]
    fn headbutt_prize_block_changes_to_empty() {
        let mut grid = grid_with_blocks();

        // Verify prize block is at col 2, row 2
        let cell = grid.cell_at(2, 2);
        assert!(cell.has_block);
        assert_eq!(cell.block_type, terrain::BlockType::Prize);
        assert_eq!(cell.collision, terrain::CollisionKind::Solid);

        // Simulate: player at col 2 (x=40px centre), feet at row 3 floor (y=48px),
        // jumping upward (negative y_velocity). The head at y=48-31=17 (row 1)
        // won't hit row 2. We need the player closer to the ceiling.
        // Head at top of row 2 area: feet = row2_bottom + height = 48 + 31 = 79?
        // No — ceiling_y_px is (row+1)*16 = 48. Head = feet - 31.
        // For head to be in row 2: head_y in [32..48). head_y = feet - 31.
        // feet = 63 → head = 32 → pixel_to_cell(32) = 2 → row 2. Yes.
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 40 << 16, // col 2 centre
                y_pos: 63 << 16, // head ends in row 2 after the synthetic step
                x_velocity: 0,
                y_velocity: -0x0011_0000, // old head starts one row higher, so the guard probes row 2
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_ceiling_collision(
            Some(&mut grid),
            None,
            PlayerForm::TheKid,
            PLAYER_HALF_WIDTH,
            &[],
        );

        // Prize block should be neutralised: Solid/no-block (ASM sub_1007A writes $6F)
        let cell_after = grid.cell_at(2, 2);
        assert!(!cell_after.has_block);
        assert_eq!(cell_after.collision, terrain::CollisionKind::Solid);

        // ASM sub_902A zeroes y_vel on ceiling hit (clr.l y_vel)
        assert_eq!(slice.player.y_velocity, 0);
    }

    #[test]
    fn ceiling_collision_skips_when_upward_motion_stays_within_same_tile() {
        let mut grid = grid_with_blocks();

        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 40 << 16, // col 2 centre
                y_pos: 75 << 16, // already moved upward from 76px this frame
                x_velocity: 0,
                y_velocity: -0x0001_0000, // 1px upward, not enough to cross next tile boundary
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_ceiling_collision(
            Some(&mut grid),
            None,
            PlayerForm::TheKid,
            PLAYER_HALF_WIDTH,
            &[],
        );

        let cell_after = grid.cell_at(2, 2);
        assert!(
            cell_after.has_block,
            "tile-boundary guard should skip the ceiling probe"
        );
        assert_eq!(slice.player.y_pos, 75 << 16);
        assert_eq!(slice.player.y_velocity, -0x0001_0000);
    }

    #[test]
    fn headbutt_metal_block_does_not_clear() {
        let mut grid = grid_with_blocks();

        // Player under col 3 (metal block — indestructible)
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 56 << 16, // col 3 centre (56px)
                y_pos: 63 << 16,
                x_velocity: 0,
                y_velocity: -0x0011_0000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_ceiling_collision(
            Some(&mut grid),
            None,
            PlayerForm::TheKid,
            PLAYER_HALF_WIDTH,
            &[],
        );

        // Metal block remains unchanged
        let cell_after = grid.cell_at(3, 2);
        assert!(cell_after.has_block);
        assert_eq!(cell_after.block_type, terrain::BlockType::Metal);
        assert_eq!(cell_after.collision, terrain::CollisionKind::Solid);
    }

    #[test]
    fn headbutt_smash_block_changes_to_empty() {
        let mut grid = grid_with_blocks();

        // Verify smash block is at col 4, row 2
        let cell = grid.cell_at(4, 2);
        assert!(cell.has_block);
        assert_eq!(cell.block_type, terrain::BlockType::Smash);
        assert_eq!(cell.collision, terrain::CollisionKind::Solid);

        // Player under col 4 (x=72px centre), jumping upward
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 72 << 16, // col 4 centre
                y_pos: 63 << 16, // head ends in row 2 after the synthetic step
                x_velocity: 0,
                y_velocity: -0x0011_0000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_ceiling_collision(
            Some(&mut grid),
            None,
            PlayerForm::TheKid,
            PLAYER_HALF_WIDTH,
            &[],
        );

        // Smash block should now be empty
        let cell_after = grid.cell_at(4, 2);
        assert!(!cell_after.has_block);
        assert_eq!(cell_after.collision, terrain::CollisionKind::Empty);

        // ASM sub_902A zeroes y_vel on ceiling hit (clr.l y_vel)
        assert_eq!(slice.player.y_velocity, 0);
    }

    #[test]
    fn cleared_smash_block_no_longer_blocks_movement() {
        let mut grid = grid_with_blocks();

        // First headbutt: clear the smash block at col 4
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 72 << 16,
                y_pos: 63 << 16,
                x_velocity: 0,
                y_velocity: -0x0011_0000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_ceiling_collision(
            Some(&mut grid),
            None,
            PlayerForm::TheKid,
            PLAYER_HALF_WIDTH,
            &[],
        );
        assert!(!grid.cell_at(4, 2).has_block);

        // Second jump through the now-empty cell: no ceiling hit
        slice.player.y_pos = 63 << 16;
        slice.player.y_velocity = -0x0011_0000;
        slice.apply_ceiling_collision(
            Some(&mut grid),
            None,
            PlayerForm::TheKid,
            PLAYER_HALF_WIDTH,
            &[],
        );

        // Velocity should remain negative (no ceiling hit)
        assert!(slice.player.y_velocity < 0);
    }

    #[test]
    fn cleared_prize_block_no_longer_blocks_movement() {
        let mut grid = grid_with_blocks();

        // First headbutt: clear the prize block
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 40 << 16,
                y_pos: 63 << 16,
                x_velocity: 0,
                y_velocity: -0x0011_0000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_ceiling_collision(
            Some(&mut grid),
            None,
            PlayerForm::TheKid,
            PLAYER_HALF_WIDTH,
            &[],
        );
        assert!(!grid.cell_at(2, 2).has_block);

        // ASM sub_1007A: cell stays Solid (0x6F) after Prize dispatch.
        // Second pass still hits the structural ceiling — velocity is zeroed.
        slice.player.y_pos = 63 << 16;
        slice.player.y_velocity = -0x0011_0000;
        slice.prize_block_hit = None;
        slice.ceiling_prize_clear = None;
        slice.apply_ceiling_collision(
            Some(&mut grid),
            None,
            PlayerForm::TheKid,
            PLAYER_HALF_WIDTH,
            &[],
        );

        // Ceiling is still Solid (no block), so velocity is still zeroed
        assert_eq!(slice.player.y_velocity, 0);
        // But no prize is dispatched (has_block is false)
        assert!(slice.prize_block_hit.is_none());
    }

    #[test]
    fn wall_collision_clamps_one_pixel_tighter_when_moving_right() {
        let w = 4u16;
        let h = 4u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let offset = (usize::from(1u16) * usize::from(w) + usize::from(3u16)) * 2;
        data[offset] = 0x60;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 43 << 16,
                y_pos: 31 << 16,
                x_velocity: 0x0001_0000,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 48 << 16,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 64,
                height_pixels: 64,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_wall_collision(Some(&grid), PLAYER_HALF_WIDTH);

        assert_eq!(slice.player.x_pos, 42 << 16);
        assert_eq!(slice.player.x_velocity, 0);
    }

    #[test]
    fn airborne_right_wall_stickiness_waits_for_top_edge_to_clear() {
        let w = 5u16;
        let h = 5u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let offset = (usize::from(1u16) * usize::from(w) + usize::from(2u16)) * 2;
        data[offset] = 0x60;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let mut slice = OpeningSliceState {
            player: PlayerState {
                // Start 1px closer to wall so tile-boundary guard doesn't skip the
                // initial approach (ASM sub_8F26: distance_to_tile=0 at right_edge=31).
                x_pos: 27 << 16,
                y_pos: 48 << 16,
                x_velocity: 0x0001_0000,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 64 << 16,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 80,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_wall_collision(Some(&grid), PLAYER_HALF_WIDTH);
        assert_eq!(slice.player.x_pos, 26 << 16);
        assert_eq!(slice.player.x_velocity, 0);
        assert!(slice.player.right_wall_sticky);

        slice.player.y_pos = 56 << 16;
        slice.player.x_velocity = 0x0000_4000;
        slice.apply_wall_collision(Some(&grid), PLAYER_HALF_WIDTH);
        assert_eq!(slice.player.x_pos, 26 << 16);
        assert_eq!(slice.player.x_velocity, 0);
        assert!(slice.player.right_wall_sticky);

        slice.player.y_pos = 64 << 16;
        slice.player.x_velocity = 0x0000_4000;
        slice.player.x_pos = (26 << 16) + 0x0000_4000;
        slice.apply_wall_collision(Some(&grid), PLAYER_HALF_WIDTH);
        assert_eq!(slice.player.x_pos, 27 << 16);
        assert_eq!(slice.player.x_velocity, 0);
        assert!(!slice.player.right_wall_sticky);
    }

    #[test]
    fn airborne_wall_column_scan_latches_top_row_contact_loc_922c() {
        let w = 5u16;
        let h = 5u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let offset = (usize::from(1u16) * usize::from(w) + usize::from(2u16)) * 2;
        data[offset] = 0x60;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let mut slice = OpeningSliceState {
            player: PlayerState {
                // ASM sub_8F26 -> sub_922C scans the leading wall column from
                // the foot row up through the adjusted head row. At y=56 this
                // still includes row 1, so the overhang is a fresh side hit.
                x_pos: 27 << 16,
                y_pos: 56 << 16,
                x_velocity: 0x0001_0000,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 64 << 16,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 80,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_wall_collision(Some(&grid), PLAYER_HALF_WIDTH);
        assert_eq!(slice.player.x_pos, 26 << 16);
        assert_eq!(slice.player.x_velocity, 0);
        assert!(slice.player.right_wall_sticky);
        assert_eq!(slice.player.wall_cling_direction, Some(Direction::Right));
    }

    #[test]
    fn landing_frame_rewinds_airborne_horizontal_step() {
        let grid = grid_with_floor_at_row(4);
        let previous_x_pos = 100 << 16;
        let x_velocity = 0x0001_8000;
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: previous_x_pos + x_velocity as u32,
                y_pos: 64 << 16,
                x_velocity,
                y_velocity: 0x0001_0000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 0,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_floor_collision(Some(&grid), true, false, PLAYER_HALF_WIDTH, true);

        assert_eq!(slice.player.x_pos, previous_x_pos);
        assert_eq!(slice.player.y_pos, 63 << 16);
        assert_eq!(slice.player.x_velocity, x_velocity);
        assert_eq!(slice.player.y_velocity, 0);
        assert!(slice.player.on_ground);
    }

    #[test]
    fn solid_floor_snap_lands_from_sub_half_pixel_gap() {
        let grid = grid_with_floor_at_row(4);
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 100 << 16,
                y_pos: (63 << 16) + 0x0000_4000,
                x_velocity: 0,
                y_velocity: 0x0000_4000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 0,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_floor_collision(Some(&grid), false, false, PLAYER_HALF_WIDTH, true);

        assert!(slice.player.on_ground);
        assert_eq!(slice.player.y_pos, 0x003F_0000);
    }

    #[test]
    fn solid_floor_snap_waits_at_half_pixel_gap() {
        let grid = grid_with_floor_at_row(4);
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 100 << 16,
                y_pos: (63 << 16) + 0x0000_8000,
                x_velocity: 0,
                y_velocity: 0x0000_4000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 0,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_floor_collision(Some(&grid), false, false, PLAYER_HALF_WIDTH, true);

        assert!(!slice.player.on_ground);
        assert_eq!(slice.player.y_pos, (63 << 16) + 0x0000_8000);
    }

    #[test]
    fn asm_faithful_solid_floor_snap_waits_for_contact_row() {
        let grid = grid_with_floor_at_row(4);
        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 100 << 16,
                y_pos: (63 << 16) + 0x0000_4000,
                x_velocity: 0,
                y_velocity: 0x0000_4000,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 0,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_floor_collision(Some(&grid), false, false, PLAYER_HALF_WIDTH, false);

        assert!(!slice.player.on_ground);
        assert_eq!(slice.player.y_pos, (63 << 16) + 0x0000_4000);
    }

    #[test]
    fn stomp_from_above_kills_enemy() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        setup_stomp_scenario(&mut runtime, true);
        let enemies_before = runtime.active_enemies.len();
        assert!(enemies_before > 0, "Skull should be activated");

        runtime.step_frame(FrameInput::default());
        assert_eq!(
            runtime.active_enemies.len(),
            enemies_before,
            "ASM Mini_Hopping_Skull.asm loc_3F1E6 keeps the stomped skull alive through the air-damage loop"
        );
        assert_eq!(runtime.active_enemies[0].hit_points, 0);
        assert_eq!(
            runtime.active_enemies[0].wait_timer,
            MINI_HOPPING_SKULL_AIR_DAMAGE_WAIT_TIMER_LOC_3F200
        );
    }

    #[test]
    fn stomp_bounces_player_upward() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        setup_stomp_scenario(&mut runtime, true);
        runtime.step_frame(FrameInput::default());
        runtime.step_frame(FrameInput::default());

        if let Some(ref slice) = runtime.opening_slice {
            assert!(
                slice.player.y_velocity < 0,
                "player should bounce upward after stomp, got y_vel={:#010X}",
                slice.player.y_velocity
            );
        }
    }

    #[test]
    fn side_contact_damages_not_stomps() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        advance_past_level_entry_damage_grace(&mut runtime);

        runtime.progression.number_hitpoints = 3;
        setup_stomp_scenario(&mut runtime, false);
        let enemies_before = runtime.active_enemies.len();
        assert!(enemies_before > 0, "Skull should be activated");

        runtime.step_frame(FrameInput::default());
        assert_eq!(
            runtime.active_enemies.len(),
            enemies_before,
            "side contact should not kill enemy"
        );
        assert!(
            runtime.progression.number_hitpoints < 3
                || !matches!(runtime.player_lifecycle, PlayerLifecycle::Alive),
            "side contact should deal damage to player"
        );
    }

    #[test]
    fn falling_straight_onto_enemy_is_stomp() {
        // Player falling directly onto enemy — ASM velocity-direction classification
        // treats this as "kid above" even with slight interpenetration, because the
        // dominant approach direction is vertical.
        let player = PlayerState {
            x_pos: 100 << 16,
            y_pos: 219 << 16,
            x_velocity: 0,
            y_velocity: 0x0002_0000,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x1C },
            x_pos: 100 << 16,
            y_pos: 224 << 16,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Left),
            on_ground: Some(false),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };

        assert_eq!(
            classify_enemy_contact_with_half_width(
                player,
                &enemy,
                player.y_velocity,
                player.y_pos,
                PlayerForm::TheKid.half_width() as i64,
            ),
            EnemyContactResult::Stomp
        );
    }

    #[test]
    fn mini_hopping_skull_graze_does_not_damage() {
        let player = PlayerState {
            x_pos: 0x042E_1980,
            y_pos: 0x008E_4600,
            x_velocity: 0x0002_6E80,
            y_velocity: 0x0002_3E00,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x1C },
            x_pos: 0x0434_0000,
            y_pos: 0x007D_8800,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Left),
            on_ground: Some(false),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };

        assert_eq!(
            classify_enemy_contact_with_half_width(
                player,
                &enemy,
                player.y_velocity,
                player.y_pos,
                PlayerForm::TheKid.half_width() as i64,
            ),
            EnemyContactResult::None
        );
    }

    #[test]
    fn walking_dragon_top_heavy_contact_box_registers_as_stomp() {
        let player = PlayerState {
            x_pos: 0x01B4_1700,
            y_pos: 0x0188_3300,
            x_velocity: 0,
            y_velocity: 0x0008_0000,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x01C2_8000,
            y_pos: 0x01B0_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(2),
            damage_flash: true,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };

        assert_eq!(
            classify_enemy_contact_with_half_width(
                player,
                &enemy,
                player.y_velocity,
                player.y_pos,
                PlayerForm::TheKid.half_width() as i64,
            ),
            EnemyContactResult::Stomp,
            "the frame-2938 landing coordinates only intersect DragonWalking's taller top-heavy sub-box",
        );
    }

    #[test]
    fn walking_dragon_pre_landing_frame_has_not_reached_contact_threshold() {
        let player = PlayerState {
            x_pos: 0x01B2_3D00,
            y_pos: 0x018C_1300,
            x_velocity: 0x0002_9300,
            y_velocity: 0x0008_0000,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x01C1_8000,
            y_pos: 0x01B0_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(2),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };

        assert_eq!(
            classify_enemy_contact_with_half_width(
                player,
                &enemy,
                player.y_velocity,
                player.y_pos,
                PlayerForm::TheKid.half_width() as i64,
            ),
            EnemyContactResult::None
        );
    }

    #[test]
    fn walking_dragon_right_facing_edge_contact_matches_live_tas_stomp_window() {
        let player = PlayerState {
            x_pos: 0x037E_F100,
            y_pos: 0x01A1_C000,
            x_velocity: 0x0002_9640,
            y_velocity: 0x0006_DE00,
            facing: Direction::Left,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x0367_0000,
            y_pos: 0x01C0_0000,
            layout_index: Some(0),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Right),
            on_ground: Some(true),
            hit_points: Some(2),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };

        assert_eq!(
            classify_enemy_contact_with_half_width(
                player,
                &enemy,
                player.y_velocity,
                player.y_pos,
                PlayerForm::IronKnight.half_width() as i64,
            ),
            EnemyContactResult::Stomp,
            "the live frame-3399 second-dragon setup should classify as a stomp when IronKnight's wider hitbox clips the flipped dragon box",
        );
    }

    #[test]
    fn level0_walking_dragon_landing_frame_classifies_damage_not_stomp_loc_1c76() {
        // Fresh playtest repro: the walk-right/jump recording grazes the first
        // Level 0 walking dragon at native frame 114. ASM kid.asm loc_1C76
        // still resolves that overlap as side contact, so this frame explains
        // the reported hurt without proving a missing death animation.
        let player = PlayerState {
            x_pos: 0x007C_5100,
            y_pos: 0x0102_F0C0,
            x_velocity: 0x0002_9F00,
            y_velocity: 0x0004_8220,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x008E_8000,
            y_pos: 0x0110_0000,
            layout_index: Some(2),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Left),
            on_ground: Some(true),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: Some(10),
            shared_walker_patrol_timer: Some(185),
            shared_walker_patrol_short_next: Some(true),
            walk_phase: Some(0),
            wait_timer: Some(185),
            debug_fields: std::collections::BTreeMap::new(),
        };

        assert_eq!(
            classify_enemy_contact_with_half_width(
                player,
                &enemy,
                player.y_velocity,
                player.y_pos,
                PlayerForm::TheKid.half_width() as i64,
            ),
            EnemyContactResult::Damage,
            "ASM kid.asm loc_1C76: frame-114 overlap is side contact, not a stomp; use a sharper repro before changing death-animation code"
        );
    }

    #[test]
    fn walking_dragon_idle_edge_contact_keeps_exact_asm_box() {
        let player = PlayerState {
            x_pos: 0x008C_0000,
            y_pos: 0x010F_0000,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Right,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x00A2_8000,
            y_pos: 0x0110_0000,
            layout_index: Some(2),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Left),
            on_ground: Some(true),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: Some(10),
            shared_walker_patrol_timer: Some(185),
            shared_walker_patrol_short_next: Some(true),
            walk_phase: Some(0),
            wait_timer: Some(185),
            debug_fields: std::collections::BTreeMap::new(),
        };

        assert_eq!(
            classify_enemy_contact_with_half_width(
                player,
                &enemy,
                player.y_velocity,
                player.y_pos,
                PlayerForm::TheKid.half_width() as i64,
            ),
            EnemyContactResult::None,
            "ASM GfxObjects_CollisionKid keeps the issue-216 x140 idle Dragon edge outside Kid's exact hitbox; the native Dragon landing-graze allowance must not widen zero-velocity contact"
        );
    }

    #[test]
    fn walking_dragon_leftward_ground_pass_keeps_exact_asm_box_loc_1c42() {
        let player = PlayerState {
            x_pos: 0x00A6_F300,
            y_pos: 0x010F_0000,
            x_velocity: -0x0002_0000,
            y_velocity: 0,
            facing: Direction::Left,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x0D },
            x_pos: 0x0090_8000,
            y_pos: 0x0110_0000,
            layout_index: Some(2),
            x_velocity: Some(0),
            y_velocity: Some(0),
            facing: Some(Direction::Left),
            on_ground: Some(true),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: Some(10),
            shared_walker_patrol_timer: Some(185),
            shared_walker_patrol_short_next: Some(true),
            walk_phase: Some(0),
            wait_timer: Some(185),
            debug_fields: std::collections::BTreeMap::new(),
        };

        assert_eq!(
            classify_enemy_contact_with_half_width(
                player,
                &enemy,
                player.y_velocity,
                player.y_pos,
                PlayerForm::TheKid.half_width() as i64,
            ),
            EnemyContactResult::None,
            "ASM GfxObjects_CollisionKid loc_1C42 uses TheKid's 5px hitbox; issue-216 x210-L must queue DragonWalking damage one frame later, not from this leftward ground-pass edge"
        );
    }

    #[test]
    fn stomp_works_during_invulnerability() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        runtime.progression.invulnerable_frames = 100;
        setup_stomp_scenario(&mut runtime, true);
        let enemies_before = runtime.active_enemies.len();
        assert!(enemies_before > 0, "Skull should be activated");

        runtime.step_frame(FrameInput::default());
        assert_eq!(
            runtime.active_enemies.len(),
            enemies_before,
            "ASM Mini_Hopping_Skull.asm loc_3F1E6 keeps the invulnerability stomp kill visible through the air-damage loop"
        );
        assert_eq!(runtime.active_enemies[0].hit_points, 0);
        assert_eq!(
            runtime.active_enemies[0].wait_timer,
            MINI_HOPPING_SKULL_AIR_DAMAGE_WAIT_TIMER_LOC_3F200
        );
    }

    #[test]
    fn falling_onto_diamond_enemy_does_not_collect_or_stomp_bounce_loc_3ad16() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        let diamonds_before = runtime.progression.diamond_count;

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        let diamond_pos = runtime
            .active_enemies
            .iter()
            .find(|enemy| enemy.family == EnemyFamily::Diamond)
            .map(|enemy| (enemy.x_pos, enemy.y_pos))
            .expect("Diamond should activate before the fall-through check");

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = diamond_pos.0;
            slice.player.y_pos = diamond_pos.1.saturating_sub(8 << 16);
            slice.player.y_velocity = 0x0002_0000;
            slice.player.on_ground = false;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        let slice = runtime
            .opening_slice
            .expect("opening slice should remain active");
        assert_eq!(
            runtime.progression.diamond_count, diamonds_before,
            "ASM Diamond.asm loc_3AD16 creates an enemy object, not a prize diamond award"
        );
        assert_ne!(
            slice.player.y_velocity, STOMP_BOUNCE_VELOCITY,
            "Diamond enemy contact should not reuse the prize-pickup path as a stomp bounce"
        );
        assert!(
            runtime
                .active_enemies
                .iter()
                .any(|enemy| enemy.family == EnemyFamily::Diamond),
            "Diamond enemy should remain active instead of being removed as a collected prize"
        );
    }

    #[test]
    fn contact_damage_does_not_apply_knockback_velocity() {
        // ASM does NOT apply knockback velocity on contact damage — only HP loss + invulnerability
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        advance_past_level_entry_damage_grace(&mut runtime);
        runtime.progression.number_hitpoints = 3;

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x011C_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.player.x_velocity = 0;
            slice.player.y_velocity = 0;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        runtime.step_frame(FrameInput::default());

        let slice = runtime.opening_slice.unwrap();
        assert_eq!(
            slice.player.x_velocity, 0,
            "no knockback x velocity on contact damage"
        );
    }

    #[test]
    fn no_knockback_on_lethal_contact() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);
        advance_past_level_entry_damage_grace(&mut runtime);
        runtime.progression.number_hitpoints = 1; // will die on contact

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x011C_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.player.on_ground = true;
            slice.player.x_velocity = 0;
            slice.player.y_velocity = 0;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        runtime.step_frame(FrameInput::default());

        assert!(
            !matches!(runtime.player_lifecycle, PlayerLifecycle::Alive),
            "player should be dead after lethal contact"
        );
    }

    #[test]
    fn collision_velocity_comparison_classifies_above_when_object_slower_y_dominate() {
        let player = PlayerState {
            x_pos: 100 << 16,
            y_pos: 200 << 16,
            x_velocity: 0x0001_0000,
            y_velocity: 0x0002_0000,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x1C },
            x_pos: 100 << 16,
            y_pos: 204 << 16,
            layout_index: Some(0),
            x_velocity: Some(0x0002_0000),
            y_velocity: Some(0x0001_0000),
            facing: Some(Direction::Left),
            on_ground: Some(false),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };

        let result = classify_enemy_contact_with_half_width(
            player,
            &enemy,
            player.y_velocity,
            player.y_pos,
            PlayerForm::TheKid.half_width() as i64,
        );
        assert_eq!(
            result,
            EnemyContactResult::Stomp,
            "ASM loc_1C42: when object_x_delta < 0 and object_y_delta < 0, the collision resolution compares y_term vs x_term to classify contact side; object slower in Y means dominant approach is vertical, so player contacts from above"
        );
    }

    #[test]
    fn collision_velocity_comparison_classifies_above_when_object_slower_y_branch_loc_1c76() {
        // ASM loc_1C76 (kid.asm:3412-3424): when object_x_delta >= 0 and object_y_delta < 0,
        // the object is faster in X but the player is falling faster (object_y_delta < 0).
        // The code computes x_term = (obj_right - kid_left) * rel_y and
        // y_term = (obj_top - kid_bottom) * rel_x (note the different bounding terms
        // compared to loc_1C94 and loc_1CB8). When y_term < x_term, the dominant
        // approach direction is vertical from above, so the player stomps.
        let player = PlayerState {
            x_pos: 100 << 16,
            y_pos: 200 << 16,
            x_velocity: 0x0001_0000,
            y_velocity: 0x0003_0000,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x1C },
            x_pos: 100 << 16,
            y_pos: 204 << 16,
            layout_index: Some(0),
            x_velocity: Some(0x0002_0000),
            y_velocity: Some(0x0001_0000),
            facing: Some(Direction::Left),
            on_ground: Some(false),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };

        let result = classify_enemy_contact_with_half_width(
            player,
            &enemy,
            player.y_velocity,
            player.y_pos,
            PlayerForm::TheKid.half_width() as i64,
        );
        assert_eq!(
            result,
            EnemyContactResult::Stomp,
            "ASM loc_1C76: when object_x_delta >= 0 and object_y_delta < 0, the player is falling faster; the modified overlap terms favour vertical detection, so player stomps"
        );
    }

    #[test]
    fn collision_velocity_comparison_classifies_right_when_x_dominant_loc_1c76() {
        // ASM loc_1C76 (kid.asm:3422-3424): when y_term >= x_term in this branch,
        // the dominant approach direction is horizontal, so the player contacts
        // from the side (right) rather than above.
        // Branch condition: object_x_delta >= 0 (enemy faster in X) AND object_y_delta < 0 (player falling faster)
        let player = PlayerState {
            x_pos: 100 << 16,
            y_pos: 200 << 16,
            x_velocity: 0,
            y_velocity: 0x0003_0000,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        // Position enemy overlapping with player (same X, 4px below in Y)
        let enemy = NativeObjectSnapshot {
            kind: NativeObjectKind::Enemy { object_type: 0x1C },
            x_pos: 100 << 16,
            y_pos: 204 << 16,
            layout_index: Some(0),
            x_velocity: Some(0x0010_0000),
            y_velocity: Some(0x0001_0000),
            facing: Some(Direction::Left),
            on_ground: Some(false),
            hit_points: Some(1),
            damage_flash: false,
            palette_variant: 0,
            stationary: false,
            contact_phase: None,
            shared_walker_state: None,
            shared_walker_patrol_timer: None,
            shared_walker_patrol_short_next: None,
            walk_phase: None,
            wait_timer: None,
            debug_fields: std::collections::BTreeMap::new(),
        };

        let result = classify_enemy_contact_with_half_width(
            player,
            &enemy,
            player.y_velocity,
            player.y_pos,
            PlayerForm::TheKid.half_width() as i64,
        );
        assert_eq!(
            result,
            EnemyContactResult::Damage,
            "ASM loc_1C76: with strong horizontal enemy approach (x_vel=0x100000) and player falling fast (y_vel=0x300000), the overlap comparison should classify this as side contact"
        );
    }

    #[test]
    fn two_hp_enemy_survives_first_stomp() {
        let mut runtime = NativeRuntime::new(manifest_with_tough_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        // Activate enemy
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        let enemies_before = runtime.active_enemies.len();
        assert!(enemies_before > 0, "skull should be active");

        // First stomp
        setup_stomp_against_first_active_enemy(&mut runtime);
        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.active_enemies.len(),
            enemies_before,
            "2-HP enemy should survive first stomp"
        );
        assert_eq!(
            runtime.active_enemies[0].hit_points, 1,
            "HP should be decremented to 1"
        );
        assert_eq!(
            runtime.active_enemies[0].wait_timer,
            MINI_HOPPING_SKULL_AIR_DAMAGE_WAIT_TIMER_LOC_3F200,
            "ASM Mini_Hopping_Skull.asm loc_3F1E6 enters the air-damage loop instead of generic flash damage"
        );
        assert_eq!(runtime.active_enemies[0].damage_flash_frames, 0);
    }

    #[test]
    fn mini_hopping_skull_stomp_enters_air_damage_loop() {
        let mut runtime = NativeRuntime::new(manifest_with_tough_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        runtime.active_enemies[0].x_velocity = 0x0001_4000;

        setup_stomp_against_first_active_enemy(&mut runtime);
        let mut slice = runtime.opening_slice.unwrap();
        runtime.active_enemy_roster = runtime
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();
        let pre_y_velocity = slice.player.y_velocity;
        let pre_y_pos = slice.player.y_pos;
        let frame_start_x_pos = slice.player.x_pos;
        let frame_start_y_pos = slice.player.y_pos;
        assert!(runtime.process_stomps(
            &mut slice,
            pre_y_velocity,
            pre_y_pos,
            frame_start_x_pos,
            frame_start_y_pos,
        ));

        let skull = &runtime.active_enemies[0];
        assert_eq!(skull.hit_points, 1);
        assert_eq!(
            skull.wait_timer, MINI_HOPPING_SKULL_AIR_DAMAGE_WAIT_TIMER_LOC_3F200,
            "ASM Mini_Hopping_Skull.asm loc_3F1E6 enters the loc_3F200 landing loop"
        );
        assert_eq!(skull.target_x_velocity, 0x0001_4000);
        assert_eq!(skull.x_velocity, 0);
        assert_eq!(skull.y_velocity, 0x0002_8800);
        assert!(!skull.on_ground);
        assert_eq!(skull.damage_flash_frames, 0);
    }

    #[test]
    fn two_hp_enemy_dies_on_second_stomp() {
        let mut runtime = NativeRuntime::new(manifest_with_tough_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        // Activate enemy
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());
        let enemies_before = runtime.active_enemies.len();

        // First stomp
        setup_stomp_against_first_active_enemy(&mut runtime);
        runtime.step_frame(FrameInput::default());
        assert_eq!(
            runtime.active_enemies.len(),
            enemies_before,
            "survives first stomp"
        );

        // The bounce now resolves on the next frame, so consume it before setting
        // up the second stomp attempt.
        runtime.step_frame(FrameInput::default());

        // HP-2 MiniHoppingSkulls relaunch upward after a non-lethal stomp, and
        // the enemy stays stomp-immune while it is still rising.
        let mut found_stompable_window = false;
        for _ in 0..32 {
            let enemy = runtime
                .active_enemies
                .first()
                .unwrap_or_else(|| panic!("skull should remain active before the second stomp"));
            if enemy.on_ground || enemy.y_velocity >= 0 {
                found_stompable_window = true;
                break;
            }
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            found_stompable_window,
            "expected the damaged skull to leave its rising immunity window"
        );

        // Second stomp
        setup_stomp_against_first_active_enemy(&mut runtime);
        runtime.step_frame(FrameInput::default());
        assert_eq!(
            runtime.active_enemies.len(),
            enemies_before,
            "ASM Mini_Hopping_Skull.asm loc_3F1E6 keeps the second-stomp kill visible through the air-damage loop"
        );
        assert_eq!(runtime.active_enemies[0].hit_points, 0);
        assert_eq!(
            runtime.active_enemies[0].wait_timer,
            MINI_HOPPING_SKULL_AIR_DAMAGE_WAIT_TIMER_LOC_3F200
        );

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0;
            slice.player.y_pos = 0;
            slice.player.y_velocity = 0;
            slice.player.on_ground = true;
        }
        runtime.kid_touched_enemy_last_frame_loc_fa74 = false;
        runtime.kid_touched_enemy_this_frame_loc_fa75 = false;

        let mut entered_death_animation = false;
        for _ in 0..16 {
            runtime.step_frame(FrameInput::default());
            if runtime
                .active_enemies
                .first()
                .is_some_and(|enemy| enemy.death_animation_active)
            {
                entered_death_animation = true;
                break;
            }
        }
        assert!(
            entered_death_animation,
            "ASM Mini_Hopping_Skull.asm loc_3F200 -> loc_3F158 should start the visible death animation after floor contact"
        );
    }

    #[test]
    fn stomp_always_bounces_even_on_surviving_enemy() {
        let mut runtime = NativeRuntime::new(manifest_with_tough_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00E0_0000;
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        // Stomp the 2-HP enemy
        setup_stomp_against_first_active_enemy(&mut runtime);
        runtime.step_frame(FrameInput::default());
        runtime.step_frame(FrameInput::default());

        let slice = runtime.opening_slice.unwrap();
        assert!(
            slice.player.y_velocity < 0,
            "player should bounce upward even when enemy survives (y_vel={})",
            slice.player.y_velocity
        );
    }

    #[test]
    fn sustained_contact_never_stomps_loc_1d4c() {
        // ASM GfxObjects_CollisionKid_Collide (kid.asm:3384): `tst.b
        // ($FFFFFA74).w; bne.w loc_1D4C` — while the Kid overlapped an enemy
        // on the previous frame's collision pass, the loc_1C42 side classifier
        // is skipped and loc_1D4C writes colid_kidbelow to the Kid slot only,
        // so a stomp can never register on a sustained-contact frame. Oracle
        // WRAM proof: jugg60 frames 238-240 show FA74=$FF, kid col38=$28,
        // tank turret col38=$0000 with Character_Movement=3 (Jump) — the kid
        // falls past the tank top with no bounce.
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        setup_stomp_scenario(&mut runtime, true);
        // Re-arm the latch the helper cleared: pretend last frame's collision
        // pass already overlapped an enemy.
        runtime.kid_touched_enemy_this_frame_loc_fa75 = true;
        let hp_before = runtime.active_enemies[0].hit_points;

        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.active_enemies[0].hit_points, hp_before,
            "latched (sustained) contact must not register a stomp"
        );
        assert!(
            runtime.pending_stomp_bounce.is_none(),
            "no stomp bounce may be queued from a sustained-contact frame"
        );
    }

    #[test]
    fn stomp_gate_loc_1ce4_blocks_grounded_fresh_contact() {
        // ASM loc_1CE4 (kid.asm:3479): `cmpi.w #MoveID_Jump,
        // (Character_Movement).w; bne.w loc_1D4C` — a fresh kidabove contact
        // only writes the $2C stomp colid while the Kid is in Jump movement.
        // Oracle WRAM proof: jugg60 frames 267/275 show FA74=0 (fresh
        // contact) with Character_Movement=0 (Standing) and kid col38=$28.
        let runtime = NativeRuntime::new(manifest_with_static_skull());
        let player = PlayerState {
            x_pos: 0x0120_0000,
            y_pos: 0x00D8_0000,
            x_velocity: 0,
            y_velocity: 0x0002_0000,
            facing: Direction::Right,
            on_ground: false,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        };
        assert!(
            !runtime.stomp_registration_blocked_loc_1ce4(player),
            "airborne fresh contact is Jump movement: stomp allowed"
        );
        assert!(
            !runtime.stomp_registration_blocked_loc_1ce4(PlayerState {
                on_ground: true,
                just_jumped: true,
                ..player
            }),
            "jump launch frame still counts as Jump movement"
        );
        assert!(
            runtime.stomp_registration_blocked_loc_1ce4(PlayerState {
                on_ground: true,
                ..player
            }),
            "grounded movement IDs fail the loc_1CE4 MoveID_Jump gate"
        );

        let mut stale_jump_runtime = NativeRuntime::new(manifest_with_static_skull());
        stale_jump_runtime.probe_start_slope_settle_frames = 1;
        let stale_jump_player = PlayerState {
            on_ground: true,
            just_jumped: false,
            y_velocity: 0,
            ..player
        };
        assert_eq!(
            stale_jump_runtime.current_character_movement(stale_jump_player),
            3,
            "test setup should expose stale Character_Movement=MoveID_Jump"
        );
        assert!(
            !stale_jump_runtime.stomp_registration_blocked_loc_1ce4(stale_jump_player),
            "ASM loc_1CE4 gates on the WRAM Character_Movement word, not native on_ground"
        );

        let mut latched = NativeRuntime::new(manifest_with_static_skull());
        latched.kid_touched_enemy_last_frame_loc_fa74 = true;
        assert!(
            latched.stomp_registration_blocked_loc_1ce4(player),
            "the FA74 sustained-contact latch blocks even airborne stomps"
        );
    }

    // -----------------------------------------------------------------------
    // Pacer archetype (Goat, Armadillo, Tank, Crab, Scorpion, Driller)
    // -----------------------------------------------------------------------

    #[test]
    fn pacer_walk_speed_goat_scales_with_hp() {
        // Goat: base_hp=3, base_speed=$8000, step=$10000
        assert_eq!(pacer_walk_speed(EnemyFamily::Goat, 3), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Goat, 4), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Goat, 5), 0x0002_8000);
    }

    #[test]
    fn goat_spawn_faces_toward_player_per_loc_3ca30() {
        // ASM Goat.asm loc_3CA30 is reached after the init branch skips
        // `st x_direction(a3)` only when Kid is left of the goat.
        assert_eq!(
            spawn_facing(EnemyFamily::Goat, 100 << 16, 80 << 16),
            Direction::Left
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Goat, 100 << 16, 100 << 16),
            Direction::Right
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Goat, 100 << 16, 120 << 16),
            Direction::Right
        );
    }

    #[test]
    fn goat_hp4_speed_adds_loc_3ca6e_tier() {
        let hit_points = spawn_hit_points(EnemyFamily::Goat, 1);

        assert_eq!(hit_points, 4);
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Goat, hit_points),
            0x0001_8000,
            "ASM Goat.asm loc_3CA6E adds the HP4 `$10000` speed tier before walk dispatch"
        );
    }

    #[test]
    fn goat_hp3_speed_uses_loc_3ca7e_common_final_add() {
        let hit_points = spawn_hit_points(EnemyFamily::Goat, 0);

        assert_eq!(hit_points, 3);
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Goat, hit_points),
            0x0000_8000,
            "ASM Goat.asm loc_3CA7E is the common final `$8000` speed add"
        );
    }

    #[test]
    fn goat_hp4_init_animation_uses_loc_3caca_profile() {
        assert_eq!(
            goat_hp4_init_animation_loc_3_caca(spawn_hit_points(EnemyFamily::Goat, 1)),
            Some(GoatInitAnimationProfile {
                initial_animation_script: "stru_3C996",
                saved_patrol_animation: "stru_3C996",
                saved_damage_resume_animation: "stru_3C9B0",
            }),
            "ASM Goat.asm loc_3CACA handles current_hp == 4 before the shared direction dispatch"
        );
        assert_eq!(goat_hp4_init_animation_loc_3_caca(3), None);
        assert_eq!(goat_hp4_init_animation_loc_3_caca(5), None);
    }

    #[test]
    fn goat_high_hp_init_animation_uses_loc_3cae8_profile() {
        assert_eq!(
            goat_high_hp_init_animation_loc_3_cae8(spawn_hit_points(EnemyFamily::Goat, 2)),
            Some(GoatInitAnimationProfile {
                initial_animation_script: "stru_3C996",
                saved_patrol_animation: "stru_3C996",
                saved_damage_resume_animation: "stru_3C9CA",
            }),
            "ASM Goat.asm loc_3CAE8 handles current_hp > 4 with the high-HP damage resume animation"
        );
        assert_eq!(goat_high_hp_init_animation_loc_3_cae8(4), None);
    }

    #[test]
    fn goat_initial_dispatch_loc_3cb02_preserves_x_direction_convention() {
        assert_eq!(
            goat_initial_dispatch_loc_3_cb02(Direction::Left),
            GoatInitialDispatchRouteLoc3CB02::ClearXDirection
        );
        assert_eq!(
            goat_initial_dispatch_loc_3_cb02(Direction::Right),
            GoatInitialDispatchRouteLoc3CB02::FlippedXDirection,
            "ASM Goat.asm loc_3CB02 tests x_direction after installing the goat collision callback"
        );
    }

    #[test]
    fn goat_collision_dispatch_loc_3cb14_routes_collision_words() {
        assert_eq!(
            goat_collision_dispatch_loc_3cb14(0),
            GoatCollisionRouteLoc3CB14::NoCollisionClearResume,
            "ASM Goat.asm loc_3CB14 sends zero collision_type to the shared walker resume path"
        );
        assert_eq!(
            goat_collision_dispatch_loc_3cb14(-1),
            GoatCollisionRouteLoc3CB14::Death,
            "ASM Goat.asm loc_3CB14 sends signed-negative collision words to the death branch"
        );
        assert_eq!(
            goat_collision_dispatch_loc_3cb14(GOAT_COLID_HURT),
            GoatCollisionRouteLoc3CB14::Damage,
            "ASM Goat.asm loc_3CB14 sends colid_hurt to the HP damage branch"
        );
        assert_eq!(
            goat_collision_dispatch_loc_3cb14(GOAT_COLID_KIDABOVE),
            GoatCollisionRouteLoc3CB14::Damage,
            "ASM Goat.asm loc_3CB14 sends colid_kidabove to the same HP damage path"
        );
        assert_eq!(
            goat_collision_dispatch_loc_3cb14(GOAT_COLID_KIDRIGHT),
            GoatCollisionRouteLoc3CB14::BerzerkerSideGate,
            "ASM Goat.asm loc_3CB14 routes colid_kidright to the Berzerker side-contact gate"
        );
        assert_eq!(
            goat_collision_dispatch_loc_3cb14(GOAT_COLID_KIDLEFT),
            GoatCollisionRouteLoc3CB14::BerzerkerSideGate,
            "ASM Goat.asm loc_3CB14 routes colid_kidleft to the same side-contact gate"
        );
        assert_eq!(
            goat_collision_dispatch_loc_3cb14(0x0010),
            GoatCollisionRouteLoc3CB14::UnknownPositiveClearResume,
            "ASM Goat.asm loc_3CB14 sends unrelated positive collision words to the loc_3CBF6 tail through the final bne branch"
        );
    }

    #[test]
    fn goat_damage_response_loc_3cb38_decrements_hp_before_routing() {
        assert_eq!(
            goat_damage_response_loc_3cb38(3, 0x0002_8000),
            GoatDamageResponseLoc3CB38::Hurt {
                remaining_hp: 2,
                damage_animation: "stru_3C9E4",
                recoil_state_word: -0x0078,
                restored_walk_speed: 0x0002_8000,
                restored_animation: "saved_goat_patrol_animation",
            },
            "ASM Goat.asm loc_3CB38 should play the hurt animation and restore saved walk state when the decremented HP word remains non-zero"
        );
        assert_eq!(
            goat_damage_response_loc_3cb38(1, 0x0001_8000),
            GoatDamageResponseLoc3CB38::Death,
            "ASM Goat.asm loc_3CB38 should route to death when the decremented HP word reaches zero"
        );
    }

    #[test]
    fn goat_nonfatal_hurt_sets_recoil_state_and_hibernates_loc_3cb38() {
        let grid = grid_with_floor_at_row(14);
        let mut goat = make_pacer(EnemyFamily::Goat, 3, 0x80, 0xE0);
        goat.x_velocity = -0x0000_8000;

        assert!(!goat.take_damage_or_queue_death());
        assert_eq!(goat.hit_points, 2);
        assert_eq!(
            goat.shared_walker_state, -0x0078,
            "ASM Goat.asm loc_3CB38 should set `$42(a3)` to $FF88 after nonfatal hurt"
        );
        assert_eq!(
            goat.walk_speed_override, 0x0000_8000,
            "ASM Goat.asm loc_3CB38 should restore `$50(a5)` from saved `$5E(a5)`"
        );

        let mut active = vec![goat];
        let mut drilled_blocks = Vec::new();
        step_active_enemies(
            &mut active,
            Some(&grid),
            Some(0x0040_0000),
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut drilled_blocks,
        );

        assert_eq!(
            active[0].x_pos, 0x0080_0000,
            "ASM Goat.asm loc_3CB38 hibernates through `stru_3C9E4` before resuming the shared walker",
        );
        assert_eq!(active[0].x_velocity, 0);
        assert_eq!(active[0].y_velocity, 0);
    }

    #[test]
    fn goat_death_sequence_matches_loc_3cb64() {
        assert_eq!(
            goat_death_sequence_loc_3cb64(0x05),
            GoatDeathSequenceLoc3CB64 {
                death_animation: "stru_3C9EA",
                waits_until_animation_finished: true,
                status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker { index: 5 },
                exits_enemy: true,
            },
            "ASM Goat.asm loc_3CB64 should play the death animation to completion before exiting through loc_3E956"
        );
    }

    #[test]
    fn goat_berzerker_side_gate_matches_loc_3cb76() {
        assert_eq!(
            goat_berzerker_side_gate_loc_3cb76(false, 0),
            GoatBerzerkerSideGateLoc3CB76::ResumeSharedWalker {
                next_collision_type: 0,
            },
            "ASM Goat.asm loc_3CB76 should clear collision and resume while Berzerker_charging is zero"
        );
        assert_eq!(
            goat_berzerker_side_gate_loc_3cb76(true, 1),
            GoatBerzerkerSideGateLoc3CB76::SideShoveThenHurtResume,
            "ASM Goat.asm loc_3CB76 should branch to the shove/hurt-resume path when `$42(a3)` is positive"
        );
        assert_eq!(
            goat_berzerker_side_gate_loc_3cb76(true, 0),
            GoatBerzerkerSideGateLoc3CB76::AirborneKnockback(GoatBerzerkerKnockbackStartLoc3CB76 {
                has_level_collision: false,
                has_kid_collision: true,
                is_moved: true,
                is_animated: false,
                sprite_mapping: "LnkTo_unk_C7C56-Data_Index",
                x_velocity: 0,
                y_velocity: -0x0004_8000,
            }),
            "ASM Goat.asm loc_3CB76 should seed airborne knockback when charge is active and `$42(a3)` is not positive"
        );
    }

    #[test]
    fn goat_berzerker_airborne_loop_matches_loc_3cba8() {
        assert_eq!(GOAT_BERZERKER_KNOCKBACK_GRAVITY_LOC_3CBA8, 0x0000_4000);
        assert_eq!(
            goat_berzerker_airborne_frame_loc_3cba8(-0x0004_8000),
            GoatBerzerkerAirborneFrameLoc3CBA8 {
                hibernates_one_frame: true,
                applies_motion_subroutine: true,
                motion_subroutine: "sub_3E8BA",
                next_y_velocity: -0x0004_4000,
                repeats_airborne_loop: true,
            },
            "ASM Goat.asm loc_3CBA8 should hibernate, run sub_3E8BA, add `$4000`, and repeat"
        );
        assert_eq!(
            goat_berzerker_airborne_frame_loc_3cba8(-0x0004_4000).next_y_velocity,
            -0x0004_0000,
            "ASM Goat.asm loc_3CBA8 should apply the same gravity step on the next loop"
        );
    }

    #[test]
    fn goat_berzerker_side_shove_matches_loc_3cbba() {
        assert_eq!(GOAT_BERZERKER_SIDE_SHOVE_PIXELS_LOC_3CBBA, 0x0014_0000);
        assert_eq!(
            goat_berzerker_side_shove_loc_3cbba(Direction::Left, 0x0080_0000, 0x0002_8000),
            GoatBerzerkerSideShoveLoc3CBBA {
                x_position: 0x0094_0000,
                damage_animation: "stru_3C9E4",
                waits_until_animation_finished: true,
                recoil_state_word: -0x0078,
                restored_walk_speed: 0x0002_8000,
                restored_animation: "saved_goat_patrol_animation",
                clears_object_meta_bit_7: true,
                falls_through_to_collision_clear: true,
            },
            "ASM Goat.asm loc_3CBBA should add 20px for clear x_direction before the hurt-resume sequence"
        );
        assert_eq!(
            goat_berzerker_side_shove_loc_3cbba(Direction::Right, 0x0080_0000, 0x0001_8000),
            GoatBerzerkerSideShoveLoc3CBBA {
                x_position: 0x006C_0000,
                damage_animation: "stru_3C9E4",
                waits_until_animation_finished: true,
                recoil_state_word: -0x0078,
                restored_walk_speed: 0x0001_8000,
                restored_animation: "saved_goat_patrol_animation",
                clears_object_meta_bit_7: true,
                falls_through_to_collision_clear: true,
            },
            "ASM Goat.asm loc_3CBC8 should subtract 20px for flipped x_direction before restoring patrol state"
        );
        let hurt_resume =
            goat_berzerker_side_shove_loc_3cbba(Direction::Left, 0x0080_0000, 0x0002_8000);
        assert_eq!(
            (
                hurt_resume.damage_animation,
                hurt_resume.waits_until_animation_finished,
                hurt_resume.recoil_state_word,
                hurt_resume.restored_walk_speed,
                hurt_resume.restored_animation,
                hurt_resume.clears_object_meta_bit_7,
                hurt_resume.falls_through_to_collision_clear,
            ),
            (
                "stru_3C9E4",
                true,
                -0x0078,
                0x0002_8000,
                "saved_goat_patrol_animation",
                true,
                true,
            ),
            "ASM Goat.asm loc_3CBCE should play the hurt-resume animation, restore saved patrol state, and fall through to collision clear"
        );
    }

    #[test]
    fn goat_collision_clear_resume_matches_loc_3cbf6() {
        assert_eq!(
            goat_collision_clear_resume_loc_3cbf6(true, Direction::Left),
            GoatCollisionClearResumeLoc3CBF6 {
                collision_type: 0,
                next: GoatCollisionClearResumeRouteLoc3CBF6::SpawnedChildLeft,
            },
            "ASM Goat.asm loc_3CBF6 should clear collision_type and send `$5C(a5)` spawned children with clear x_direction to loc_3E7CA"
        );
        assert_eq!(
            goat_collision_clear_resume_loc_3cbf6(true, Direction::Right),
            GoatCollisionClearResumeLoc3CBF6 {
                collision_type: 0,
                next: GoatCollisionClearResumeRouteLoc3CBF6::SpawnedChildRight,
            },
            "ASM Goat.asm loc_3CBF6 should send `$5C(a5)` spawned children with set x_direction to loc_3E6AE"
        );
        assert_eq!(
            goat_collision_clear_resume_loc_3cbf6(false, Direction::Left),
            GoatCollisionClearResumeLoc3CBF6 {
                collision_type: 0,
                next: GoatCollisionClearResumeRouteLoc3CBF6::OrdinaryLeft,
            },
            "ASM Goat.asm loc_3CC0C should send ordinary goats with clear x_direction to loc_3E560"
        );
        assert_eq!(
            goat_collision_clear_resume_loc_3cbf6(false, Direction::Right),
            GoatCollisionClearResumeLoc3CBF6 {
                collision_type: 0,
                next: GoatCollisionClearResumeRouteLoc3CBF6::OrdinaryRight,
            },
            "ASM Goat.asm loc_3CC0C should send ordinary goats with set x_direction to loc_3E3AC"
        );
    }

    #[test]
    fn goat_tracking_right_route_matches_loc_3cc36() {
        assert_eq!(
            goat_tracking_right_route_loc_3cc36(100, 133, Direction::Left),
            Some(GoatTrackingRightRouteLoc3CC36::TurnRight),
            "ASM Goat.asm loc_3CC36 should set x_direction when d7 is negative and Kid is more than 32px right"
        );
        assert_eq!(
            goat_tracking_right_route_loc_3cc36(100, 133, Direction::Right),
            Some(GoatTrackingRightRouteLoc3CC36::AlreadyFacingRight),
            "ASM Goat.asm loc_3CC36 should keep set x_direction when already facing Kid on the right"
        );
        assert_eq!(
            goat_tracking_right_route_loc_3cc36(100, 132, Direction::Left),
            None,
            "ASM Goat.asm loc_3CC36 is not the right-side reface path while d7 is still within -32px"
        );
    }

    #[test]
    fn goat_tracking_left_route_matches_loc_3cc46() {
        assert_eq!(
            goat_tracking_left_route_loc_3cc46(133, 100, Direction::Right),
            Some(GoatTrackingLeftRouteLoc3CC46::TurnLeft),
            "ASM Goat.asm loc_3CC46 should clear x_direction when d7 is positive and Kid is more than 32px left"
        );
        assert_eq!(
            goat_tracking_left_route_loc_3cc46(133, 100, Direction::Left),
            Some(GoatTrackingLeftRouteLoc3CC46::AlreadyFacingLeft),
            "ASM Goat.asm loc_3CC46 should keep clear x_direction when already facing Kid on the left"
        );
        assert_eq!(
            goat_tracking_left_route_loc_3cc46(132, 100, Direction::Right),
            None,
            "ASM Goat.asm loc_3CC46 is not reached while d7 is still within +32px"
        );
    }

    #[test]
    fn goat_tracking_positive_shared_state_resumes_by_direction_loc_3cc52() {
        assert_eq!(
            goat_tracking_route_loc_3cc18(1, 100, 133, Direction::Left),
            GoatTrackingRouteLoc3CC18::Resume(GoatTrackingResumeRouteLoc3CC52::Loc3E566),
            "ASM Goat.asm loc_3CC18 should branch positive `$42(a3)` to loc_3CC52, which sends clear x_direction to loc_3E566 without rechecking Kid distance"
        );
        assert_eq!(
            goat_tracking_route_loc_3cc18(1, 133, 100, Direction::Right),
            GoatTrackingRouteLoc3CC18::Resume(GoatTrackingResumeRouteLoc3CC52::Loc3E3B2),
            "ASM Goat.asm loc_3CC18 should branch positive `$42(a3)` to loc_3CC52, which sends set x_direction to loc_3E3B2 without rechecking Kid distance"
        );
        assert_eq!(
            goat_tracking_route_loc_3cc18(0, 100, 132, Direction::Left),
            GoatTrackingRouteLoc3CC18::Resume(GoatTrackingResumeRouteLoc3CC52::Loc3E566),
            "ASM Goat.asm loc_3CC18 should also reach loc_3CC52 inside the inclusive +/-32px dead zone"
        );
    }

    #[test]
    fn pacer_walk_speed_armadillo_caps_high_hp_speed() {
        // ASM Armadillo.asm loc_3C756/loc_3C766 caps the high-HP path at two
        // `$10000` adds plus the final `$8000`.
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, 1), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, 2), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, 3), 0x0002_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, 4), 0x0002_8000);
    }

    #[test]
    fn pacer_probe_half_width_matches_family_specific_48_words() {
        assert_eq!(
            pacer_probe_half_width(EnemyFamily::Crab),
            12,
            "ASM Crab.asm loc_3B00A seeds `$48(a5)=$C` before the shared walker loop",
        );
        assert_eq!(
            pacer_probe_half_width(EnemyFamily::Driller),
            12,
            "ASM Driller.asm loc_3BD74 seeds `$48(a5)=$C` before the shared walker loop",
        );
        assert_eq!(
            pacer_probe_half_width(EnemyFamily::Tank),
            11,
            "ASM Tank.asm loc_3B31E seeds `$48(a5)=$B` before the shared walker loop",
        );
        assert_eq!(
            pacer_probe_half_width(EnemyFamily::TankShooting),
            11,
            "ASM Tank_Shooting.asm loc_3B5A6 seeds `$48(a5)=$B` before the shared walker loop",
        );
        for family in [
            EnemyFamily::Armadillo,
            EnemyFamily::Goat,
            EnemyFamily::Scorpion,
        ] {
            assert_eq!(
                pacer_probe_half_width(family),
                9,
                "ASM Armadillo/Goat/Scorpion init seeds `$48(a5)=9` for the shared walker loop",
            );
        }
        assert_eq!(
            pacer_probe_half_width(EnemyFamily::Robot),
            ENEMY_PROBE_HALF_WIDTH,
            "families without an audited `$48(a5)` override keep the generic 8px probe",
        );
    }

    #[test]
    fn pacer_probe_height_matches_family_specific_4a_words() {
        assert_eq!(
            pacer_probe_height(EnemyFamily::Tank),
            32,
            "ASM Tank.asm loc_3B31E seeds `$4A(a5)=$20` before the shared walker loop",
        );
        assert_eq!(
            pacer_probe_height(EnemyFamily::TankShooting),
            32,
            "ASM Tank_Shooting.asm loc_3B5A6 seeds `$4A(a5)=$20` before the shared walker loop",
        );
        assert_eq!(
            pacer_probe_height(EnemyFamily::Driller),
            ENEMY_PROBE_HEIGHT,
            "Driller keeps the generic native height until its `$4A(a5)` path is audited for shared walkers",
        );
    }

    #[test]
    fn armadillo_spawned_child_resume_matches_loc_3c91a() {
        assert_eq!(
            armadillo_spawned_child_resume_loc_3c91a(true, Direction::Left),
            ArmadilloSpawnedChildResumeLoc3C91A {
                collision_type: 0,
                next: ArmadilloSpawnedChildResumeRouteLoc3C91A::SpawnedChildLeft,
            },
            "ASM Armadillo.asm loc_3C91A clears collision_type and sends `$5C(a5)` spawned children with clear x_direction to the left resume"
        );
        assert_eq!(
            armadillo_spawned_child_resume_loc_3c91a(true, Direction::Right),
            ArmadilloSpawnedChildResumeLoc3C91A {
                collision_type: 0,
                next: ArmadilloSpawnedChildResumeRouteLoc3C91A::SpawnedChildRight,
            },
            "ASM Armadillo.asm loc_3C91A sends `$5C(a5)` spawned children with set x_direction to the right resume"
        );
        assert_eq!(
            armadillo_spawned_child_resume_loc_3c91a(false, Direction::Left),
            ArmadilloSpawnedChildResumeLoc3C91A {
                collision_type: 0,
                next: ArmadilloSpawnedChildResumeRouteLoc3C91A::OrdinaryTail,
            },
            "ASM Armadillo.asm loc_3C91A falls through to the ordinary resume tail when `$5C(a5)` is clear"
        );
    }

    #[test]
    fn armadillo_ordinary_resume_matches_loc_3c930() {
        assert_eq!(
            armadillo_ordinary_resume_loc_3c930(Direction::Left),
            ArmadilloOrdinaryResumeLoc3C930 {
                collision_type: 0,
                next: ArmadilloOrdinaryResumeRouteLoc3C930::OrdinaryLeft,
            },
            "ASM Armadillo.asm loc_3C930 sends ordinary Armadillos with clear x_direction to the left shared walker"
        );
        assert_eq!(
            armadillo_ordinary_resume_loc_3c930(Direction::Right),
            ArmadilloOrdinaryResumeLoc3C930 {
                collision_type: 0,
                next: ArmadilloOrdinaryResumeRouteLoc3C930::OrdinaryRight,
            },
            "ASM Armadillo.asm loc_3C930 sends ordinary Armadillos with set x_direction to the right shared walker"
        );
    }

    #[test]
    fn armadillo_spawned_child_fall_setup_matches_loc_3c93c() {
        assert_eq!(
            armadillo_spawned_child_fall_setup_loc_3c93c(),
            ArmadilloSpawnedChildFallSetupLoc3C93C {
                x_velocity: 0,
                y_velocity: 0x0002_C000,
                has_level_collision: true,
                is_moved: true,
            },
            "ASM Armadillo.asm loc_3C93C seeds spawned-child fall with zero x_vel, `$2C000` y_vel, level collision, and movement"
        );
    }

    #[test]
    fn pacer_walk_speed_scorpion_is_fixed() {
        // Scorpion: fixed $18000 regardless of HP.
        assert_eq!(pacer_walk_speed(EnemyFamily::Scorpion, 3), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Scorpion, 4), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Scorpion, 5), 0x0001_8000);
    }

    #[test]
    fn scorpion_tracking_boosts_speed_at_high_hp() {
        // Scorpion.asm loc_3DD7E: within 170px and enemy_level > 1 (HP >= 5),
        // walk_speed_override set to $38000.
        use enemies::step_scorpion_tracking;
        let mut enemy = make_pacer(EnemyFamily::Scorpion, 5, 100, 224);
        let player_x = Some(150u32 << 16); // 50px away (within 170px range)
        step_scorpion_tracking(&mut enemy, player_x, None, None);
        assert_eq!(
            enemy.walk_speed_override, 0x3_8000,
            "HP>=5 within range should boost"
        );
        assert_eq!(enemy.facing, Direction::Right, "should face toward player");
    }

    #[test]
    fn scorpion_tracking_faces_right_when_kid_is_right_per_loc_3dda4() {
        use enemies::step_scorpion_tracking;

        let mut enemy = make_pacer(EnemyFamily::Scorpion, 4, 100, 224);
        enemy.facing = Direction::Left;

        step_scorpion_tracking(&mut enemy, Some(150u32 << 16), None, None);

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Scorpion.asm loc_3DDA4 sets x_direction when d4 = scorpion_x - Kid_x is negative"
        );
    }

    #[test]
    fn scorpion_tracking_no_boost_at_low_hp() {
        // HP 3-4 (enemy_level 0-1): no speed boost even within range.
        use enemies::step_scorpion_tracking;
        let mut enemy = make_pacer(EnemyFamily::Scorpion, 4, 100, 224);
        let player_x = Some(150u32 << 16);
        step_scorpion_tracking(&mut enemy, player_x, None, None);
        assert_eq!(enemy.walk_speed_override, 0, "HP<5 should not boost");
    }

    #[test]
    fn scorpion_tracking_resets_speed_outside_range() {
        // Scorpion.asm loc_3DE7E: outside 170px, reset to $18000.
        use enemies::step_scorpion_tracking;
        let mut enemy = make_pacer(EnemyFamily::Scorpion, 5, 100, 224);
        enemy.walk_speed_override = 0x3_8000; // previously boosted
        let player_x = Some(500u32 << 16); // 400px away (outside 170px)
        step_scorpion_tracking(&mut enemy, player_x, None, None);
        assert_eq!(
            enemy.walk_speed_override, 0x1_8000,
            "outside range should reset"
        );
    }

    #[test]
    fn scorpion_tracking_loc_3de7e_clears_lock_only_when_kid_y_velocity_nonzero() {
        use enemies::step_scorpion_tracking;

        let player_x = Some(500u32 << 16); // outside +/-$AA range
        let mut grounded_kid = make_pacer(EnemyFamily::Scorpion, 5, 100, 224);
        grounded_kid.scorpion_tracking_lock = true;
        step_scorpion_tracking(&mut grounded_kid, player_x, None, Some(0));
        assert!(
            grounded_kid.scorpion_tracking_lock,
            "ASM Scorpion.asm loc_3DE7E leaves `$4E(a5)` set when Kid y_vel is zero"
        );

        let mut airborne_kid = make_pacer(EnemyFamily::Scorpion, 5, 100, 224);
        airborne_kid.scorpion_tracking_lock = true;
        step_scorpion_tracking(&mut airborne_kid, player_x, None, Some(0x0001_0000));
        assert!(
            !airborne_kid.scorpion_tracking_lock,
            "ASM Scorpion.asm loc_3DE7E clears `$4E(a5)` only after `tst.l y_vel(a4)` is non-zero"
        );
    }

    #[test]
    fn scorpion_tracking_refaces_before_range_exit_per_loc_3dda8() {
        use enemies::step_scorpion_tracking;

        let mut enemy = make_pacer(EnemyFamily::Scorpion, 5, 100, 224);
        enemy.facing = Direction::Left;
        enemy.walk_speed_override = 0x3_8000;

        step_scorpion_tracking(&mut enemy, Some(500u32 << 16), None, None);

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Scorpion.asm loc_3DDA8 is reached after loc_3DDA4 can set x_direction, so out-of-range Kid to the right still re-faces first"
        );
        assert_eq!(
            enemy.walk_speed_override, 0x1_8000,
            "ASM Scorpion.asm loc_3DDA8 exits to loc_3DE7E outside +/-$AA and restores base speed"
        );
    }

    #[test]
    fn scorpion_close_range_gate_loc_3ddc8_requires_36px_and_facing() {
        use enemies::{ScorpionCloseRangeRouteLoc3DDC8, scorpion_close_range_route_loc_3ddc8};

        assert_eq!(
            scorpion_close_range_route_loc_3ddc8(0x25, 0, Direction::Left, 1),
            ScorpionCloseRangeRouteLoc3DDC8::ContinueSharedWalker,
            "ASM Scorpion.asm loc_3DDC8 exits to loc_3DE64 when d4 is greater than +$24"
        );
        assert_eq!(
            scorpion_close_range_route_loc_3ddc8(-0x25, 0, Direction::Right, 1),
            ScorpionCloseRangeRouteLoc3DDC8::ContinueSharedWalker,
            "ASM Scorpion.asm loc_3DDC8 exits to loc_3DE64 when d4 is less than -$24"
        );
        assert!(
            scorpion_close_range_route_loc_3ddc8(0x24, 0, Direction::Left, 1).locks_tracking(),
            "ASM Scorpion.asm loc_3DDC8 keeps +$24 inside the close-range gate"
        );
        assert!(
            scorpion_close_range_route_loc_3ddc8(-0x24, 0, Direction::Right, 1).locks_tracking(),
            "ASM Scorpion.asm loc_3DDC8 keeps -$24 inside the close-range gate"
        );
    }

    #[test]
    fn scorpion_close_range_gate_loc_3dde6_rejects_wrong_facing() {
        use enemies::{ScorpionCloseRangeRouteLoc3DDC8, scorpion_close_range_route_loc_3ddc8};

        assert_eq!(
            scorpion_close_range_route_loc_3ddc8(-0x10, 0, Direction::Left, 1),
            ScorpionCloseRangeRouteLoc3DDC8::ContinueSharedWalker,
            "ASM Scorpion.asm loc_3DDE6 rejects negative d4 when x_direction is clear"
        );
        assert_eq!(
            scorpion_close_range_route_loc_3ddc8(0x10, 0, Direction::Right, 1),
            ScorpionCloseRangeRouteLoc3DDC8::ContinueSharedWalker,
            "ASM Scorpion.asm loc_3DDC8 positive d4 falls through only when x_direction is clear"
        );
    }

    #[test]
    fn scorpion_sting_lock_loc_3ddee_matches_vertical_and_level_gates() {
        use enemies::{ScorpionCloseRangeRouteLoc3DDC8, scorpion_close_range_route_loc_3ddc8};

        assert_eq!(
            scorpion_close_range_route_loc_3ddc8(0, -0x21, Direction::Left, 1),
            ScorpionCloseRangeRouteLoc3DDC8::TrackingLockedVerticalMiss,
            "ASM Scorpion.asm loc_3DDEE sets $4E(a5) before y delta below -$20 exits through loc_3DE96"
        );
        assert_eq!(
            scorpion_close_range_route_loc_3ddc8(0, 0x31, Direction::Left, 1),
            ScorpionCloseRangeRouteLoc3DDC8::TrackingLockedVerticalMiss,
            "ASM Scorpion.asm loc_3DDEE sets $4E(a5) before y delta above +$30 exits through loc_3DE96"
        );
        assert_eq!(
            scorpion_close_range_route_loc_3ddc8(0, 0, Direction::Left, 0),
            ScorpionCloseRangeRouteLoc3DDC8::TrackingLockedNoStingAnimation,
            "ASM Scorpion.asm loc_3DDEE branches to loc_3DE3C without the sting animation when enemy_level < 1"
        );
        assert_eq!(
            scorpion_close_range_route_loc_3ddc8(0, 0, Direction::Left, 1),
            ScorpionCloseRangeRouteLoc3DDC8::StingAnimation {
                linked_tail_has_kid_collision: true,
                walk_speed_override: 0,
                object_meta_or_mask: 0xC000,
                animation: "stru_3DB8E",
                hibernate_until_animation_finished: true,
            },
            "ASM Scorpion.asm loc_3DDEE enables tail collision, stops $50 speed, ORs object_meta with $C000, and waits for stru_3DB8E"
        );
    }

    #[test]
    fn scorpion_sting_recovery_matches_loc_3de40_reset_side_effects() {
        use enemies::{ScorpionStingRecoveryLoc3DE40, scorpion_sting_recovery_loc_3de40};

        assert_eq!(
            scorpion_sting_recovery_loc_3de40(0x0001_8000, 0xC013),
            ScorpionStingRecoveryLoc3DE40 {
                linked_tail_has_kid_collision: false,
                sprite_mapping: "LnkTo_unk_C7DDE-Data_Index",
                animation_script: "stru_3DB60",
                walk_speed_override: 0x0001_8000,
                object_meta: 0x0013,
            },
            "ASM Scorpion.asm loc_3DE40 clears linked tail collision, restores the base animation/speed, and masks object_meta with $0FFF"
        );
    }

    #[test]
    fn scorpion_sting_recovery_is_wired_after_hibernation_loc_3de40() {
        let grid = grid_with_floor_at_row(14);
        let mut scorpion = make_pacer(EnemyFamily::Scorpion, 5, 0x80, 0xE0);
        scorpion.scorpion_sting_frames = 1;
        scorpion.walk_speed_override = 0;
        scorpion.x_velocity = 0x0001_8000;
        let start_x = scorpion.x_pos;
        let mut active = vec![scorpion];
        let mut drilled_blocks = Vec::new();

        step_active_enemies(
            &mut active,
            Some(&grid),
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut drilled_blocks,
        );

        assert_eq!(
            active[0].x_pos, start_x,
            "ASM Scorpion.asm loc_3DDEE hibernates on the final sting frame before loc_3DE40 resumes"
        );
        assert_eq!(
            active[0].scorpion_sting_frames, 0,
            "ASM Scorpion.asm loc_3DE40 clears the native linked-tail collision phase"
        );
        assert_eq!(
            active[0].walk_speed_override, 0x0001_8000,
            "ASM Scorpion.asm loc_3DE40 restores `$50(a5)` from saved `$5E(a5)`"
        );

        step_active_enemies(
            &mut active,
            Some(&grid),
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut drilled_blocks,
        );

        assert!(
            active[0].x_pos < start_x,
            "ASM Scorpion.asm loc_3DE40 resumes the shared-walker left branch on the next scheduler pass"
        );
    }

    #[test]
    fn scorpion_resume_after_tracking_matches_loc_3de6c_cache_and_clear() {
        use enemies::{
            ScorpionResumeRouteLoc3DE6C, ScorpionResumeStateLoc3DE6C,
            scorpion_resume_after_tracking_loc_3de6c,
        };

        assert_eq!(
            scorpion_resume_after_tracking_loc_3de6c(0x0123, Direction::Left),
            ScorpionResumeStateLoc3DE6C {
                cached_kid_x_word: 0x0123,
                next_collision_type: 0,
                next: ScorpionResumeRouteLoc3DE6C::ClearDirection,
            },
            "ASM Scorpion.asm loc_3DE6C moves d3 into a2, clears collision_type, and sends clear x_direction down the left-facing resume branch"
        );
        assert_eq!(
            scorpion_resume_after_tracking_loc_3de6c(0x00FE, Direction::Right).next,
            ScorpionResumeRouteLoc3DE6C::SetDirection,
            "ASM Scorpion.asm loc_3DE6C sends set x_direction down the right-facing resume branch after the same collision clear"
        );
    }

    #[test]
    fn scorpion_timer_gate_matches_loc_3dea0_threshold_and_swap() {
        use enemies::{
            ScorpionTimerGateLoc3DEA0, ScorpionTimerRestartRouteLoc3DEA0,
            ScorpionTimerResumeRouteLoc3DECE, ScorpionTimerRouteLoc3DEA0,
            scorpion_timer_gate_loc_3dea0,
        };

        assert_eq!(
            scorpion_timer_gate_loc_3dea0(1, 7, 0x0010_0020, Direction::Left),
            ScorpionTimerGateLoc3DEA0 {
                timer_54: 7,
                alternating_timer_pair_6e: 0x0010_0020,
                route: ScorpionTimerRouteLoc3DEA0::ResumeSharedWalker {
                    next: ScorpionTimerResumeRouteLoc3DECE::ClearDirection,
                },
            },
            "ASM Scorpion.asm loc_3DEA0 leaves the timer pair untouched when $42 is positive and not $000A"
        );
        assert_eq!(
            scorpion_timer_gate_loc_3dea0(-1, 2, 0x0010_0020, Direction::Left),
            ScorpionTimerGateLoc3DEA0 {
                timer_54: 1,
                alternating_timer_pair_6e: 0x0010_0020,
                route: ScorpionTimerRouteLoc3DEA0::ResumeSharedWalker {
                    next: ScorpionTimerResumeRouteLoc3DECE::ClearDirection,
                },
            },
            "ASM Scorpion.asm loc_3DEA0 decrements $54 when $42 is signed non-positive and resumes while the decremented timer is nonzero"
        );
        assert_eq!(
            scorpion_timer_gate_loc_3dea0(0x000A, 1, 0x0010_0020, Direction::Right),
            ScorpionTimerGateLoc3DEA0 {
                timer_54: 0x0010,
                alternating_timer_pair_6e: 0x0020_0010,
                route: ScorpionTimerRouteLoc3DEA0::RestartPacing {
                    next: ScorpionTimerRestartRouteLoc3DEA0::SetDirection,
                },
            },
            "ASM Scorpion.asm loc_3DEA0 treats $42 == $000A as active, swaps the packed $6E timer halves when $54 reaches zero, and restarts by direction"
        );
    }

    #[test]
    fn scorpion_timer_restart_matches_loc_3deae_swap_and_reload() {
        use enemies::{
            ScorpionTimerGateLoc3DEA0, ScorpionTimerRestartRouteLoc3DEA0,
            ScorpionTimerRouteLoc3DEA0, scorpion_timer_gate_loc_3dea0,
        };

        assert_eq!(
            scorpion_timer_gate_loc_3dea0(0, 1, 0x1234_00AB, Direction::Left),
            ScorpionTimerGateLoc3DEA0 {
                timer_54: 0x1234,
                alternating_timer_pair_6e: 0x00AB_1234,
                route: ScorpionTimerRouteLoc3DEA0::RestartPacing {
                    next: ScorpionTimerRestartRouteLoc3DEA0::ClearDirection,
                },
            },
            "ASM Scorpion.asm loc_3DEAE decrements $54 to zero, swaps `$6E(a5)`, reloads $54 from the swapped low word, then restarts through the clear x_direction branch"
        );
    }

    #[test]
    fn scorpion_timer_resume_matches_loc_3dece_direction_dispatch() {
        use enemies::{ScorpionTimerResumeRouteLoc3DECE, scorpion_timer_resume_route_loc_3dece};

        assert_eq!(
            scorpion_timer_resume_route_loc_3dece(Direction::Left),
            ScorpionTimerResumeRouteLoc3DECE::ClearDirection,
            "ASM Scorpion.asm loc_3DECE reaches loc_3E566 when x_direction is clear"
        );
        assert_eq!(
            scorpion_timer_resume_route_loc_3dece(Direction::Right),
            ScorpionTimerResumeRouteLoc3DECE::SetDirection,
            "ASM Scorpion.asm loc_3DECE reaches loc_3E3B2 when x_direction is set"
        );
    }

    #[test]
    fn scorpion_collision_resume_patrol_branches_match_loc_3dd72() {
        use enemies::{ScorpionCollisionResumeBranchLoc3DD5C, scorpion_collision_resume_loc_3dd5c};

        assert_eq!(
            scorpion_collision_resume_loc_3dd5c(false, Direction::Left).next,
            ScorpionCollisionResumeBranchLoc3DD5C::PatrolLeft,
            "ASM Scorpion.asm loc_3DD72 reaches loc_3E560 when x_direction is clear"
        );
        assert_eq!(
            scorpion_collision_resume_loc_3dd5c(false, Direction::Right).next,
            ScorpionCollisionResumeBranchLoc3DD5C::PatrolRight,
            "ASM Scorpion.asm loc_3DD72 reaches loc_3E3AC when x_direction is set"
        );
    }

    #[test]
    fn pacer_walk_speed_driller_scales_with_hp() {
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Driller, 2),
            0x0000_8000,
            "ASM Driller.asm loc_3BD74 is the only speed add reached below HP 3",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Driller, 3),
            0x0001_0000,
            "ASM Driller.asm loc_3BD6C applies the second `$8000` speed add at HP 3",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Driller, 4),
            0x0001_8000,
            "ASM Driller.asm loc_3BD6C is also reached after the high-HP `$8000` preload",
        );
        assert_eq!(
            pacer_walk_speed(EnemyFamily::Driller, 5),
            0x0001_8000,
            "ASM Driller.asm loc_3BD74 caps high HP speed after the preload and loc_3BD6C add",
        );
    }

    #[test]
    fn fire_demon_spawn_loc_3a3d2_seeds_enemy_level_plus_three_hp() {
        assert_eq!(
            spawn_hit_points(EnemyFamily::FireDemon, 0),
            3,
            "ASM Fire_Demon.asm loc_3A3D2 adds 3 to the raw layout enemy_level"
        );
        assert_eq!(
            spawn_hit_points(EnemyFamily::FireDemon, 2),
            5,
            "ASM Fire_Demon.asm loc_3A3D2 stores the adjusted value in current_hp"
        );
    }

    #[test]
    fn pacer_walk_speed_fire_demon_scales_with_hp() {
        assert_eq!(pacer_walk_speed(EnemyFamily::FireDemon, 3), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::FireDemon, 4), 0x0001_0000);
        assert_eq!(pacer_walk_speed(EnemyFamily::FireDemon, 5), 0x0001_8000);
    }

    #[test]
    fn fire_demon_speed_loc_3a40c_adds_hp4_speed_step() {
        let hit_points = spawn_hit_points(EnemyFamily::FireDemon, 1);

        assert_eq!(hit_points, 4);
        assert_eq!(
            pacer_walk_speed(EnemyFamily::FireDemon, hit_points),
            0x0001_0000,
            "ASM Fire_Demon.asm loc_3A40C adds the HP4 speed step before the final speed add"
        );
    }

    #[test]
    fn fire_demon_speed_loc_3a414_applies_final_speed_step() {
        let hit_points = spawn_hit_points(EnemyFamily::FireDemon, 0);

        assert_eq!(hit_points, 3);
        assert_eq!(
            pacer_walk_speed(EnemyFamily::FireDemon, hit_points),
            0x0000_8000,
            "ASM Fire_Demon.asm loc_3A414 is the common final `$8000` speed add"
        );
    }

    #[test]
    fn fire_demon_animation_loc_3a444_selects_hp5_walk_script() {
        assert_eq!(
            fire_demon_hp5_animation_loc_3a444(5),
            Some(FireDemonInitAnimation {
                collision_routine: "fire_demon_collision",
                animation_script: "stru_3A334",
                saved_animation_script: "stru_3A334",
            }),
            "ASM Fire_Demon.asm loc_3A444 points a0 at the Fire Demon collision routine and keeps HP5 on the `stru_3A334` animation branch"
        );
        assert_eq!(
            fire_demon_hp5_animation_loc_3a444(4),
            None,
            "ASM Fire_Demon.asm loc_3A444 branches away from the HP5-only animation when d7 != 5"
        );
    }

    #[test]
    fn fire_demon_animation_loc_3a464_selects_default_walk_script() {
        assert_eq!(
            fire_demon_non_hp5_animation_loc_3a464(3),
            Some(FireDemonInitAnimation {
                collision_routine: "fire_demon_collision",
                animation_script: "stru_3A31A",
                saved_animation_script: "stru_3A31A",
            }),
            "ASM Fire_Demon.asm loc_3A464 initialises and saves `stru_3A31A` for non-HP5 Fire Demons"
        );
        assert_eq!(
            fire_demon_non_hp5_animation_loc_3a464(4),
            Some(FireDemonInitAnimation {
                collision_routine: "fire_demon_collision",
                animation_script: "stru_3A31A",
                saved_animation_script: "stru_3A31A",
            }),
            "ASM Fire_Demon.asm loc_3A464 is also the HP4/default walk-animation path"
        );
        assert_eq!(
            fire_demon_non_hp5_animation_loc_3a464(5),
            None,
            "ASM Fire_Demon.asm loc_3A464 is skipped by the HP5 walk-animation branch"
        );
    }

    #[test]
    fn fire_demon_dispatch_loc_3a476_follows_initial_direction() {
        assert_eq!(
            fire_demon_init_walk_dispatch_loc_3a476(Direction::Left),
            FireDemonInitWalkDispatch::LeftSharedWalker,
            "ASM Fire_Demon.asm loc_3A476 sends clear raw x_direction into the left shared-walker route"
        );
        assert_eq!(
            fire_demon_init_walk_dispatch_loc_3a476(Direction::Right),
            FireDemonInitWalkDispatch::RightSharedWalker,
            "ASM Fire_Demon.asm loc_3A476 sends set raw x_direction into the right shared-walker route"
        );
        assert_eq!(
            fire_demon_init_walk_dispatch_loc_3a476(spawn_facing(
                EnemyFamily::FireDemon,
                100 << 16,
                80 << 16
            )),
            FireDemonInitWalkDispatch::LeftSharedWalker,
            "native Fire Demon spawn-facing feeds the same left route when Kid starts left of the demon"
        );
        assert_eq!(
            fire_demon_init_walk_dispatch_loc_3a476(spawn_facing(
                EnemyFamily::FireDemon,
                100 << 16,
                120 << 16
            )),
            FireDemonInitWalkDispatch::RightSharedWalker,
            "native Fire Demon spawn-facing feeds the same right route when Kid starts right of the demon"
        );
    }

    #[test]
    fn fire_demon_palette_loc_3a48e_selects_target_palette_slot() {
        assert_eq!(
            fire_demon_palette_target_loc_3a48e(0),
            Some(0x22),
            "ASM Fire_Demon.asm loc_3A48E indexes dword_3A482 slot 0 as Palette_Buffer+$22"
        );
        assert_eq!(
            fire_demon_palette_target_loc_3a48e(1),
            Some(0x42),
            "ASM Fire_Demon.asm loc_3A48E indexes dword_3A482 slot 1 as Palette_Buffer+$42"
        );
        assert_eq!(
            fire_demon_palette_target_loc_3a48e(2),
            Some(0x52),
            "ASM Fire_Demon.asm loc_3A48E indexes dword_3A482 slot 2 as Palette_Buffer+$52"
        );
        assert_eq!(
            fire_demon_palette_target_loc_3a48e(3),
            None,
            "native boundary keeps the three-entry loc_3A48E palette table explicit"
        );
    }

    #[test]
    fn fire_demon_palette_loc_3a4b8_uses_base_data_index_for_level_zero() {
        assert_eq!(
            fire_demon_palette_base_pulse_seed_loc_3a4b8(0),
            Some(FireDemonPalettePulseSeedLoc3A4B8 {
                palette_buffer_byte_offset: 0x22,
                data_index_byte_offset: 0x0634,
            }),
            "ASM Fire_Demon.asm loc_3A4B8 loads Data_Index+$634 after loc_3A48E selects the palette target"
        );
    }

    #[test]
    fn fire_demon_palette_loc_3a4b4_steps_data_index_for_higher_levels() {
        assert_eq!(
            fire_demon_palette_pulse_seed_loc_3a4b4(0, 0).map(|seed| seed.data_index_byte_offset),
            Some(0x0634),
            "ASM Fire_Demon.asm loc_3A48E/loc_3A4B8 keeps raw enemy_level 0 on the base Data_Index+$634 palette pulse"
        );
        assert_eq!(
            fire_demon_palette_pulse_seed_loc_3a4b4(0, 1).map(|seed| seed.data_index_byte_offset),
            Some(0x0640),
            "ASM Fire_Demon.asm loc_3A4B4 adds one $C step for raw enemy_level 1"
        );
        assert_eq!(
            fire_demon_palette_pulse_seed_loc_3a4b4(0, 2).map(|seed| seed.data_index_byte_offset),
            Some(0x064C),
            "ASM Fire_Demon.asm loc_3A4B4 is reached after the pre-add, producing two $C steps for raw enemy_level > 1"
        );
        assert_eq!(
            fire_demon_palette_pulse_seed_loc_3a4b4(0, 9).map(|seed| seed.data_index_byte_offset),
            Some(0x064C),
            "ASM Fire_Demon.asm loc_3A4B4 has no additional tiers beyond the >1 fall-through"
        );
    }

    #[test]
    fn fire_demon_palette_loop_loc_3a4c2_hibernates_then_checks_liveness() {
        assert_eq!(
            fire_demon_palette_loop_head_loc_3a4c2(0),
            FireDemonPaletteLoopHeadLoc3A4C2::DeleteCurrentObject,
            "ASM Fire_Demon.asm loc_3A4C2 branches to loc_3A53E when the global Fire_Demon byte is clear after the 3-frame hibernate"
        );
        assert_eq!(
            fire_demon_palette_loop_head_loc_3a4c2(1),
            FireDemonPaletteLoopHeadLoc3A4C2::ContinueAfterHibernate {
                hibernate_frames: 3,
            },
            "ASM Fire_Demon.asm loc_3A4C2 keeps the palette helper alive and resets a1/a0 before the first copy when Fire_Demon is non-zero"
        );
    }

    #[test]
    fn fire_demon_palette_copy_loc_3a4d6_copies_seven_base_words() {
        assert_eq!(
            fire_demon_palette_first_copy_loc_3a4d6(),
            FireDemonPaletteCopyStep {
                source_byte_offset: 0,
                copied_words: 7,
                next_hibernate_frames: 3,
            },
            "ASM Fire_Demon.asm loc_3A4D6 uses moveq #6 with DBF, so the first palette pulse copies seven words from the base data pointer before the next 3-frame hibernate"
        );
    }

    #[test]
    fn fire_demon_palette_copy_loc_3a4f6_copies_seven_middle_words() {
        assert_eq!(
            fire_demon_palette_middle_copy_loc_3a4f6(),
            FireDemonPaletteCopyStep {
                source_byte_offset: 0x000E,
                copied_words: 7,
                next_hibernate_frames: 3,
            },
            "ASM Fire_Demon.asm loc_3A4F6 copies seven words after the preceding addi.l #$E,a0 advances to the middle palette pulse frame"
        );
    }

    #[test]
    fn fire_demon_palette_copy_loc_3a516_copies_seven_peak_words() {
        assert_eq!(
            fire_demon_palette_peak_copy_loc_3a516(),
            FireDemonPaletteCopyStep {
                source_byte_offset: 0x001C,
                copied_words: 7,
                next_hibernate_frames: 3,
            },
            "ASM Fire_Demon.asm loc_3A516 copies seven words after addi.l #$1C,a0 selects the peak palette pulse frame"
        );
    }

    #[test]
    fn fire_demon_palette_copy_loc_3a536_copies_middle_words_before_loop_head() {
        assert_eq!(
            fire_demon_palette_return_copy_loc_3a536(),
            FireDemonPaletteCopyStep {
                source_byte_offset: 0x000E,
                copied_words: 7,
                next_hibernate_frames: 3,
            },
            "ASM Fire_Demon.asm loc_3A536 copies seven middle-frame words and branches to loc_3A4C2 for the next hibernate"
        );
    }

    #[test]
    fn fire_demon_collision_loc_3a542_gates_fire_trail_spawn() {
        assert_eq!(
            fire_demon_fire_trail_gate_loc_3a542(true, 0, 6, 6),
            FireDemonFireTrailGateLoc3A542 {
                spawns_fire_trail: true,
                next_fire_trail_count: 7,
            },
            "ASM Fire_Demon.asm loc_3A542 lets non-level-2 demons spawn a trail at animation_timer 6 while the trail count is below 7"
        );
        assert_eq!(
            fire_demon_fire_trail_gate_loc_3a542(true, 2, 3, 0),
            FireDemonFireTrailGateLoc3A542 {
                spawns_fire_trail: true,
                next_fire_trail_count: 1,
            },
            "ASM Fire_Demon.asm loc_3A542 uses animation_timer 3 for raw enemy_level 2"
        );
        assert_eq!(
            fire_demon_fire_trail_gate_loc_3a542(true, 2, 6, 0),
            FireDemonFireTrailGateLoc3A542 {
                spawns_fire_trail: false,
                next_fire_trail_count: 0,
            },
            "ASM Fire_Demon.asm loc_3A542 skips the non-level-2 timer for raw enemy_level 2"
        );
        assert_eq!(
            fire_demon_fire_trail_gate_loc_3a542(true, 0, 6, 7),
            FireDemonFireTrailGateLoc3A542 {
                spawns_fire_trail: false,
                next_fire_trail_count: 7,
            },
            "ASM Fire_Demon.asm loc_3A542 blocks allocation when Number_of_Fire_Trails is already 7"
        );
    }

    #[test]
    fn fire_demon_collision_loc_3a55c_requires_level2_timer_three() {
        assert!(
            fire_demon_level2_fire_trail_timer_matches_loc_3a55c(3),
            "ASM Fire_Demon.asm loc_3A55C: raw enemy_level 2 branches to allocation only when animation_timer is 3"
        );
        assert!(
            !fire_demon_level2_fire_trail_timer_matches_loc_3a55c(6),
            "ASM Fire_Demon.asm loc_3A55C: raw enemy_level 2 skips the ordinary timer-6 trail spawn"
        );
    }

    #[test]
    fn fire_demon_walking_trail_step_loc_3a542_uses_saved_walk_cadence() {
        assert_eq!(
            fire_demon_walking_trail_step_loc_3a542(0, 0, 0),
            FireDemonWalkingTrailStepLoc3A542 {
                spawns_fire_trail: true,
                next_fire_trail_count: 1,
                next_animation_phase: 1,
            },
            "ASM Fire_Demon.asm loc_3A542 allocates on the first timer tick of the first LnkTo_unk_C771E frame; stru_3A31A uses six-frame entries"
        );
        assert_eq!(
            fire_demon_walking_trail_step_loc_3a542(0, 1, 0),
            FireDemonWalkingTrailStepLoc3A542 {
                spawns_fire_trail: false,
                next_fire_trail_count: 0,
                next_animation_phase: 2,
            },
            "ASM Fire_Demon.asm loc_3A542 skips the remaining ticks of the same visible fire frame because animation_timer is below 6"
        );
        assert_eq!(
            fire_demon_walking_trail_step_loc_3a542(0, 35, 0),
            FireDemonWalkingTrailStepLoc3A542 {
                spawns_fire_trail: false,
                next_fire_trail_count: 0,
                next_animation_phase: 0,
            },
            "ASM Fire_Demon.asm stru_3A31A is a six-frame loop with six ticks per frame"
        );
        assert_eq!(
            fire_demon_walking_trail_step_loc_3a542(2, 0, 0),
            FireDemonWalkingTrailStepLoc3A542 {
                spawns_fire_trail: true,
                next_fire_trail_count: 1,
                next_animation_phase: 1,
            },
            "ASM Fire_Demon.asm loc_3A542 uses raw enemy_level 2's saved stru_3A334, whose frame timer reaches 3 instead of 6"
        );
        assert_eq!(
            fire_demon_walking_trail_step_loc_3a542(2, 1, 0),
            FireDemonWalkingTrailStepLoc3A542 {
                spawns_fire_trail: false,
                next_fire_trail_count: 0,
                next_animation_phase: 2,
            },
            "ASM Fire_Demon.asm loc_3A55C only lets raw level 2 allocate while animation_timer is exactly 3"
        );
        assert_eq!(
            fire_demon_walking_trail_step_loc_3a542(0, 0, 7),
            FireDemonWalkingTrailStepLoc3A542 {
                spawns_fire_trail: false,
                next_fire_trail_count: 7,
                next_animation_phase: 1,
            },
            "ASM Fire_Demon.asm loc_3A564 enforces Number_of_Fire_Trails < 7 while animation continues"
        );
    }

    #[test]
    fn fire_demon_collision_loc_3a564_allocates_while_trail_count_below_seven() {
        assert_eq!(
            fire_demon_fire_trail_allocation_loc_3a564(6),
            FireDemonFireTrailGateLoc3A542 {
                spawns_fire_trail: true,
                next_fire_trail_count: 7,
            },
            "ASM Fire_Demon.asm loc_3A564 allocates loc_3A63C and increments Number_of_Fire_Trails while the count is below 7"
        );
        assert_eq!(
            fire_demon_fire_trail_allocation_loc_3a564(7),
            FireDemonFireTrailGateLoc3A542 {
                spawns_fire_trail: false,
                next_fire_trail_count: 7,
            },
            "ASM Fire_Demon.asm loc_3A564 branches to collision handling without allocating once Number_of_Fire_Trails is 7"
        );
    }

    #[test]
    fn fire_demon_collision_loc_3a586_routes_collision_types() {
        assert_eq!(
            fire_demon_collision_dispatch_loc_3a586(-1, 3),
            FireDemonCollisionRouteLoc3A586::Death {
                death_animation: "stru_3A378",
                decrements_fire_demon_count: true,
            },
            "ASM Fire_Demon.asm loc_3A586 sends negative collision_type to the loc_3A5A6 death path"
        );
        assert_eq!(
            fire_demon_collision_dispatch_loc_3a586(0, 3),
            FireDemonCollisionRouteLoc3A586::ResumeSharedWalker {
                next_collision_type: 0,
                source: FireDemonResumeSharedWalkerSourceLoc3A586::NoCollision,
            },
            "ASM Fire_Demon.asm loc_3A586 sends zero collision_type to the shared clear tail without damage"
        );
        assert_eq!(
            fire_demon_collision_dispatch_loc_3a586(FIRE_DEMON_COLID_KIDRIGHT, 3),
            FireDemonCollisionRouteLoc3A586::BerzerkerSideGate,
            "ASM Fire_Demon.asm loc_3A586 routes colid_kidright to the Berzerker side-contact gate"
        );
        assert_eq!(
            fire_demon_collision_dispatch_loc_3a586(FIRE_DEMON_COLID_KIDLEFT, 3),
            FireDemonCollisionRouteLoc3A586::BerzerkerSideGate,
            "ASM Fire_Demon.asm loc_3A586 routes colid_kidleft to the Berzerker side-contact gate"
        );
        assert_eq!(
            fire_demon_collision_dispatch_loc_3a586(FIRE_DEMON_COLID_HURT, 3),
            FireDemonCollisionRouteLoc3A586::Hurt {
                remaining_hp: 2,
                next_collision_type: 0,
                damage_animation: "stru_3A34E",
                restored_animation: "saved_fire_demon_walk_animation",
            },
            "ASM Fire_Demon.asm loc_3A586 decrements current_hp for colid_hurt and branches to loc_3A5BE while HP remains"
        );
        assert_eq!(
            fire_demon_collision_dispatch_loc_3a586(FIRE_DEMON_COLID_HURT, 1),
            FireDemonCollisionRouteLoc3A586::Death {
                death_animation: "stru_3A378",
                decrements_fire_demon_count: true,
            },
            "ASM Fire_Demon.asm loc_3A586 falls through to loc_3A5A6 when the colid_hurt decrement reaches zero"
        );
        assert_eq!(
            fire_demon_collision_dispatch_loc_3a586(0x0004, 3),
            FireDemonCollisionRouteLoc3A586::ResumeSharedWalker {
                next_collision_type: 0,
                source: FireDemonResumeSharedWalkerSourceLoc3A586::UnknownPositive,
            },
            "ASM Fire_Demon.asm loc_3A586 ignores unrecognised positive collision IDs via the shared clear tail"
        );
    }

    #[test]
    fn fire_demon_death_loc_3a5a6_uses_death_animation_and_liveness_count() {
        assert_eq!(
            fire_demon_death_loc_3a5a6(),
            FireDemonCollisionRouteLoc3A586::Death {
                death_animation: "stru_3A378",
                decrements_fire_demon_count: true,
            },
            "ASM Fire_Demon.asm loc_3A5A6 should hibernate on `stru_3A378` before decrementing `(Fire_Demon).w`"
        );
    }

    #[test]
    fn fire_demon_hurt_recovery_loc_3a5be_restores_saved_walk_animation() {
        assert_eq!(
            fire_demon_hurt_recovery_loc_3a5be(2),
            FireDemonCollisionRouteLoc3A586::Hurt {
                remaining_hp: 2,
                next_collision_type: 0,
                damage_animation: "stru_3A34E",
                restored_animation: "saved_fire_demon_walk_animation",
            },
            "ASM Fire_Demon.asm loc_3A5BE should hibernate on `stru_3A34E` before restoring the saved `$62(a5)` animation"
        );
    }

    #[test]
    fn fire_demon_nonfatal_hurt_hibernates_before_resuming_walk_loc_3a5be() {
        let grid = grid_with_floor_at_row(14);
        let mut demon = make_pacer(EnemyFamily::FireDemon, 4, 0x80, 0xE0);
        demon.x_velocity = -0x0001_8000;
        demon.damage_flash_frames = 3;
        let mut active = vec![demon];
        let mut drilled_blocks = Vec::new();
        let mut projectiles = Vec::new();

        step_active_enemies(
            &mut active,
            Some(&grid),
            Some(0x0040_0000),
            None,
            None,
            None,
            PlayerForm::TheKid,
            Some(&mut projectiles),
            &mut drilled_blocks,
        );

        assert_eq!(
            active[0].x_pos, 0x0080_0000,
            "ASM Fire_Demon.asm loc_3A5BE should hibernate during the nonfatal hurt animation instead of pacing"
        );
        assert_eq!(active[0].x_velocity, 0);
        assert_eq!(active[0].y_velocity, 0);
    }

    #[test]
    fn fire_demon_resume_shared_walker_loc_3a5d4_clears_collision_and_branches_by_direction() {
        assert_eq!(
            fire_demon_resume_shared_walker_loc_3a5d4(Direction::Left),
            FireDemonResumeSharedWalkerLoc3A5D4 {
                next_collision_type: 0,
                branch: FireDemonResumeSharedWalkerBranchLoc3A5D4::LeftSharedWalker,
            },
            "ASM Fire_Demon.asm loc_3A5D4 should clear collision_type and send clear x_direction to the left shared walker"
        );
        assert_eq!(
            fire_demon_resume_shared_walker_loc_3a5d4(Direction::Right),
            FireDemonResumeSharedWalkerLoc3A5D4 {
                next_collision_type: 0,
                branch: FireDemonResumeSharedWalkerBranchLoc3A5D4::RightSharedWalker,
            },
            "ASM Fire_Demon.asm loc_3A5D4 should clear collision_type and send set x_direction to the right shared walker"
        );
    }

    #[test]
    fn fire_demon_berzerker_side_gate_loc_3a5e4_launches_only_while_charging() {
        assert_eq!(
            fire_demon_berzerker_side_gate_loc_3a5e4(false, Direction::Left),
            FireDemonBerzerkerSideGateLoc3A5E4::ResumeSharedWalker(
                FireDemonResumeSharedWalkerLoc3A5D4 {
                    next_collision_type: 0,
                    branch: FireDemonResumeSharedWalkerBranchLoc3A5D4::LeftSharedWalker,
                },
            ),
            "ASM Fire_Demon.asm loc_3A5E4 should branch to loc_3A5D4 when Berzerker_charging is clear"
        );
        assert_eq!(
            fire_demon_berzerker_side_gate_loc_3a5e4(true, Direction::Right),
            FireDemonBerzerkerSideGateLoc3A5E4::Launch(FireDemonBerzerkerLaunchLoc3A5E4 {
                has_level_collision: false,
                has_kid_collision: true,
                is_moved: true,
                is_animated: false,
                sprite_mapping: "LnkTo_unk_C773E-Data_Index",
                x_velocity: 0,
                y_velocity: -0x0004_8000,
            }),
            "ASM Fire_Demon.asm loc_3A5E4 should freeze animation and launch upward while Berzerker_charging is set"
        );
    }

    #[test]
    fn fire_demon_berzerker_launch_loop_loc_3a610_applies_gravity_and_loops() {
        assert_eq!(
            fire_demon_berzerker_launch_loop_tick_loc_3a610(-0x0004_8000),
            FireDemonBerzerkerLaunchLoopTickLoc3A610 {
                hibernates_one_frame: true,
                runs_shared_retention_check_sub_3c3ce: true,
                next_y_velocity: -0x0004_4000,
                loops: true,
            },
            "ASM Fire_Demon.asm loc_3A610 should hibernate, run sub_3C3CE, add `$4000` to y_vel, and loop"
        );
    }

    #[test]
    fn fire_demon_fire_trail_wait_loop_loc_3a6ca_deletes_on_flag_or_timer() {
        assert_eq!(
            fire_demon_fire_trail_wait_loop_tick_loc_3a6ca(true, 0x0096),
            FireDemonFireTrailWaitStepLoc3A6CA {
                hibernates_one_frame: true,
                remaining_timer: 0x0096,
                next: FireDemonFireTrailWaitNextLoc3A6CA::DeleteFlagSet {
                    decrements_fire_trail_count: true,
                },
            },
            "ASM Fire_Demon.asm loc_3A6CA should delete immediately when `$19(a3)` is nonzero"
        );
        assert_eq!(
            fire_demon_fire_trail_wait_loop_tick_loc_3a6ca(false, 1),
            FireDemonFireTrailWaitStepLoc3A6CA {
                hibernates_one_frame: true,
                remaining_timer: 0,
                next: FireDemonFireTrailWaitNextLoc3A6CA::DeleteTimerExpired {
                    decrements_fire_trail_count: true,
                },
            },
            "ASM Fire_Demon.asm loc_3A6CA should fall through to deletion when `subq.w #1,d3` reaches zero"
        );
        assert_eq!(
            fire_demon_fire_trail_wait_loop_tick_loc_3a6ca(false, 0),
            FireDemonFireTrailWaitStepLoc3A6CA {
                hibernates_one_frame: true,
                remaining_timer: 0xFFFF,
                next: FireDemonFireTrailWaitNextLoc3A6CA::ContinueLoop,
            },
            "ASM Fire_Demon.asm loc_3A6CA uses word subtraction, so d3=0 wraps to `$FFFF` and keeps looping"
        );
    }

    #[test]
    fn fire_demon_fire_trail_delete_tail_loc_3a6da_decrements_global_count() {
        assert_eq!(
            fire_demon_fire_trail_delete_tail_loc_3a6da(7),
            FireDemonFireTrailDeleteTailLoc3A6DA {
                next_fire_trail_count: 6,
                deletes_current_object: true,
            },
            "ASM Fire_Demon.asm loc_3A6DA should decrement Number_of_Fire_Trails before deleting the current object"
        );
        assert_eq!(
            fire_demon_fire_trail_delete_tail_loc_3a6da(0),
            FireDemonFireTrailDeleteTailLoc3A6DA {
                next_fire_trail_count: 0xFFFF,
                deletes_current_object: true,
            },
            "ASM Fire_Demon.asm loc_3A6DA uses word subtraction, so an unexpected zero Number_of_Fire_Trails wraps to `$FFFF`"
        );
    }

    #[test]
    fn pacer_walk_speed_lion_binary_hp_check() {
        // Lion: ASM binary check — $18000 for HP<8, $28000 for HP==8.
        assert_eq!(pacer_walk_speed(EnemyFamily::Lion, 6), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Lion, 7), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Lion, 8), 0x0002_8000);
    }

    #[test]
    fn pacer_walk_speed_orca_is_fixed() {
        assert_eq!(pacer_walk_speed(EnemyFamily::Orca, 4), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Orca, 5), 0x0000_8000);
    }

    #[test]
    fn goat_walks_left_from_spawn() {
        // Full-width floor at row 14 (y=224px). Enemy foot at y=224 (row 14)
        // so probe_floor sees solid beneath.
        let grid = grid_with_floor_at_row(14);
        let mut enemy = make_pacer(EnemyFamily::Goat, 3, 128, 224);
        let x_before = enemy.x_pos;
        step_pacer(&mut enemy, Some(&grid));
        assert!(
            enemy.x_pos < x_before,
            "goat should move left (x_before={:#x}, x_after={:#x})",
            x_before,
            enemy.x_pos
        );
        assert_eq!(enemy.facing, Direction::Left);
    }

    #[test]
    fn goat_reverses_at_floor_edge() {
        // Small platform: floor only at columns 5-10, row 13 (y=208..224).
        let solid: Vec<(u16, u16)> = (5..=10).map(|col| (col, 13)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);
        // Place goat near left edge of platform, x=88px (col 5.5).
        // With speed 0x8000 (0.5 px/frame) it reaches the edge in ~8 frames.
        let mut enemy = make_pacer(EnemyFamily::Goat, 3, 88, 208);

        let mut reversed = false;
        for _ in 0..50 {
            let facing_before = enemy.facing;
            step_pacer(&mut enemy, Some(&grid));
            if enemy.facing != facing_before {
                reversed = true;
                break;
            }
        }
        assert!(reversed, "goat should reverse at floor edge");
    }

    #[test]
    fn goat_reverses_at_wall() {
        // Floor at row 13 across full width, wall at col 2 blocking leftward path.
        let mut solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 13)).collect();
        for row in 10..13 {
            solid.push((2, row));
        }
        let grid = grid_with_solid_cells(20, 15, &solid);
        // Goat at col 5 (x=80+8=88), foot at row 13 (y=208).
        let mut enemy = make_pacer(EnemyFamily::Goat, 3, 88, 208);

        for _ in 0..300 {
            step_pacer(&mut enemy, Some(&grid));
        }
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "goat should reverse on hitting wall"
        );
    }

    #[test]
    fn crab_wall_turnaround_uses_twelve_pixel_probe_width() {
        let mut solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 13)).collect();
        for row in 10..13 {
            solid.push((6, row));
        }
        let grid = grid_with_solid_cells(20, 15, &solid);
        let mut enemy = make_pacer(EnemyFamily::Crab, 3, 83, 208);
        enemy.facing = Direction::Right;

        step_pacer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM Crab.asm loc_3B00A `$48(a5)=$C` should reach the wall at x=96; the old 8px generic probe would still miss it",
        );
        assert_eq!(
            enemy.x_pos,
            84 << 16,
            "wall pushback should use the same 12px edge that detected the collision",
        );
    }

    #[test]
    fn pacer_families_map_to_pacer_archetype() {
        assert_eq!(EnemyFamily::Goat.archetype(), EnemyArchetype::Pacer);
        assert_eq!(EnemyFamily::Armadillo.archetype(), EnemyArchetype::Pacer);
        assert_eq!(EnemyFamily::Tank.archetype(), EnemyArchetype::Pacer);
        assert_eq!(EnemyFamily::Crab.archetype(), EnemyArchetype::Pacer);
        assert_eq!(EnemyFamily::Scorpion.archetype(), EnemyArchetype::Pacer);
        assert_eq!(EnemyFamily::Driller.archetype(), EnemyArchetype::Pacer);
    }

    #[test]
    fn spawn_hit_points_applies_family_base() {
        // Goat: enemy_level + 3
        assert_eq!(spawn_hit_points(EnemyFamily::Goat, 0), 3);
        assert_eq!(spawn_hit_points(EnemyFamily::Goat, 1), 4);
        assert_eq!(spawn_hit_points(EnemyFamily::Goat, 2), 5);
        // Armadillo: enemy_level + 1
        assert_eq!(spawn_hit_points(EnemyFamily::Armadillo, 0), 1);
        assert_eq!(spawn_hit_points(EnemyFamily::Armadillo, 1), 2);
        assert_eq!(spawn_hit_points(EnemyFamily::Armadillo, 2), 3);
        // Diamond.asm loc_3AD16: enemy_level + 2.
        assert_eq!(spawn_hit_points(EnemyFamily::Diamond, 0), 2);
        assert_eq!(spawn_hit_points(EnemyFamily::Diamond, 2), 4);
    }

    #[test]
    fn pacer_from_layout_object_resolves_goat() {
        let slots = vec![0xFFFF, 0x0010, 0x001C]; // slot 1 = Goat ($10)
        let object = wildside_data::EnemyLayoutObject {
            object_type: 0x81, // slot 1 (0x80 | 1)
            flags: 0,
            hit_points_plus_one: 1,
            x: 0x0100,
            y: 0x00C0,
        };
        assert_eq!(
            EnemyFamily::from_layout_object(&slots, &object),
            EnemyFamily::Goat
        );
    }

    // --- Cloud enemy ---

    #[test]
    fn cloud_maps_to_flyer_archetype() {
        assert_eq!(EnemyFamily::Cloud.archetype(), EnemyArchetype::Flyer);
    }

    #[test]
    fn cloud_spawn_velocity_uses_speed_table() {
        // enemy_level 0 → -$3000, 1 → -$5000, 2 → -$7000
        assert_eq!(spawn_x_velocity(EnemyFamily::Cloud, 0), -0x3000);
        assert_eq!(spawn_x_velocity(EnemyFamily::Cloud, 1), -0x5000);
        assert_eq!(spawn_x_velocity(EnemyFamily::Cloud, 2), -0x7000);
        // Clamp above table length
        assert_eq!(spawn_x_velocity(EnemyFamily::Cloud, 5), -0x7000);
    }

    #[test]
    fn cloud_control_script_pointer_loc_35bac_starts_at_unk_35bc4() {
        assert_eq!(
            cloud_control_script_words_loc_35BAC(),
            &[0x7800, 0x0004, 0x1E00, 0x0008, 0xFFFF],
            "ASM Cloud.asm loc_35BAC loads unk_35BC4 before command dispatch"
        );
    }

    #[test]
    fn cloud_control_script_dispatch_loc_35bb2_decodes_wait_flags_and_restart() {
        let dispatches: Vec<_> = cloud_control_script_words_loc_35BAC()
            .iter()
            .copied()
            .map(cloud_control_script_dispatch_loc_35BB2)
            .collect();

        assert_eq!(
            dispatches,
            vec![
                CloudControlDispatchLoc35BB2::Wait { frames: 0x78 },
                CloudControlDispatchLoc35BB2::SetStatus44,
                CloudControlDispatchLoc35BB2::Wait { frames: 0x1E },
                CloudControlDispatchLoc35BB2::ClearStatus44,
                CloudControlDispatchLoc35BB2::RestartScript,
            ],
            "ASM Cloud.asm loc_35BB2 dispatches by command low byte and loops on the negative sentinel"
        );
    }

    #[test]
    fn cloud_wait_loop_loc_35c56_runs_inclusive_dbf_iterations() {
        assert_eq!(
            cloud_wait_loop_loc_35C56(0x7800),
            CloudWaitLoopLoc35C56 {
                dbf_seed: 0x78,
                hibernate_frames: 0x79,
                checks_object_range_each_frame: true,
                polls_collision_each_frame: true,
                updates_idle_sprite_each_frame: true,
                checks_attack_setup_each_frame: true,
                resumes_control_script: true,
            },
            "ASM Cloud.asm loc_35C56 hibernates once before each inclusive DBF iteration"
        );
        assert_eq!(cloud_wait_loop_loc_35C56(0x1E00).hibernate_frames, 0x1F);
    }

    #[test]
    fn cloud_status_latch_loc_35c72_sets_object_44_and_resumes_dispatch() {
        assert_eq!(
            cloud_status_latch_loc_35C72(),
            CloudStatusLatchLoc35C72 {
                object_44_value: 0xFF,
                resumes_control_script: true,
            },
            "ASM Cloud.asm loc_35C72 sets `$44(a5)` with ST, then branches back to loc_35BB2"
        );
    }

    #[test]
    fn cloud_status_latch_loc_35c7a_clears_object_44_and_resumes_dispatch() {
        assert_eq!(
            cloud_status_latch_loc_35C7A(),
            CloudStatusLatchLoc35C7A {
                object_44_value: 0x00,
                resumes_control_script: true,
            },
            "ASM Cloud.asm loc_35C7A clears `$44(a5)` with SF, then branches back to loc_35BB2"
        );
    }

    #[test]
    fn cloud_transition_loc_35c82_runs_animation_then_restarts_script() {
        assert_eq!(
            cloud_transition_loc_35C82(),
            CloudTransitionLoc35C82 {
                animation_script: "stru_35EC8",
                hibernates_until_animation_finished: true,
                restarts_control_script: true,
            },
            "ASM Cloud.asm loc_35C82 initialises stru_35EC8, hibernates until it finishes, then branches to loc_35BAC"
        );
    }

    #[test]
    fn cloud_idle_sprite_gate_loc_35ca2_decrements_only_while_armed() {
        assert_eq!(
            cloud_idle_sprite_gate_loc_35CA2(0x00, 0x00, 5, 0),
            CloudIdleSpriteGateLoc35CA2 {
                d2_after_subq: 5,
                sprite_link: None,
                reaches_countdown_reload: false,
            },
            "ASM Cloud.asm loc_35CA2 returns before `subq.w #1,d2` when `$44(a5)` is clear"
        );
        assert_eq!(
            cloud_idle_sprite_gate_loc_35CA2(0xFF, 0xFF, 5, 0),
            CloudIdleSpriteGateLoc35CA2 {
                d2_after_subq: 5,
                sprite_link: None,
                reaches_countdown_reload: false,
            },
            "ASM Cloud.asm loc_35CA2 returns before the countdown while `$45(a5)` is non-zero"
        );
        assert_eq!(
            cloud_idle_sprite_gate_loc_35CA2(0xFF, 0x00, 5, 0),
            CloudIdleSpriteGateLoc35CA2 {
                d2_after_subq: 4,
                sprite_link: None,
                reaches_countdown_reload: false,
            },
            "ASM Cloud.asm loc_35CA2 keeps the decremented d2 when the result is non-zero"
        );
    }

    #[test]
    fn cloud_idle_sprite_gate_loc_35ca2_uses_masked_even_random_table_offset() {
        assert_eq!(
            cloud_idle_sprite_gate_loc_35CA2(0xFF, 0x00, 1, 0x07),
            CloudIdleSpriteGateLoc35CA2 {
                d2_after_subq: 0,
                sprite_link: Some("LnkTo_unk_C7896-Data_Index"),
                reaches_countdown_reload: true,
            },
            "ASM Cloud.asm loc_35CA2 masks random with $2F, clears bit 0, and indexes off_35C94"
        );
        assert_eq!(
            cloud_idle_sprite_gate_loc_35CA2(0xFF, 0x00, 1, 0x2F),
            CloudIdleSpriteGateLoc35CA2 {
                d2_after_subq: 0,
                sprite_link: None,
                reaches_countdown_reload: true,
            },
            "ASM Cloud.asm loc_35CA2 skips the sprite write when the masked random word is above $A"
        );
    }

    #[test]
    fn cloud_idle_sprite_reset_loc_35cd2_reloads_five_frames() {
        assert_eq!(
            cloud_idle_sprite_reset_loc_35CD2(),
            CloudIdleSpriteResetLoc35CD2 { d2: 5 },
            "ASM Cloud.asm loc_35CD2 reloads d2 with #5 after every elapsed idle-sprite attempt"
        );
    }

    #[test]
    fn cloud_diagonal_attack_setup_loc_35cd8_respects_latch_and_y_gate() {
        let active = CloudDiagonalAttackInputLoc35CD8 {
            object_45: 0xFF,
            animation_done_flag_18: false,
            x_direction: 0,
            cloud_x: 100,
            cloud_y: 50,
            kid_x: 92,
            kid_y: 58,
            attack_x_word: -3,
            attack_y_word: 3,
        };
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(active),
            CloudDiagonalAttackSetupLoc35CD8 {
                route: CloudDiagonalAttackRouteLoc35CD8::ContinueExistingAttack,
                animation_script: None,
                object_45_value: None,
            },
            "ASM Cloud.asm loc_35CD8 branches away immediately while `$45(a5)` is non-zero"
        );

        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
                object_45: 0x00,
                animation_done_flag_18: false,
                x_direction: 0,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 92,
                kid_y: 20,
                attack_x_word: 0,
                attack_y_word: 0,
            }),
            CloudDiagonalAttackSetupLoc35CD8 {
                route: CloudDiagonalAttackRouteLoc35CD8::LaterRangeChecks,
                animation_script: None,
                object_45_value: None,
            },
            "ASM Cloud.asm loc_35D62 rejects Kid-minus-cloud Y deltas below the signed -24 px side-attack window"
        );
    }

    #[test]
    fn cloud_diagonal_attack_setup_loc_35dae_spawns_latched_attack_when_animation_done() {
        let finished = cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
            object_45: 0xFF,
            animation_done_flag_18: true,
            x_direction: 0xFF,
            cloud_x: 100,
            cloud_y: 50,
            kid_x: 110,
            kid_y: 40,
            attack_x_word: 6,
            attack_y_word: 0,
        });

        assert_eq!(
            finished,
            CloudDiagonalAttackSetupLoc35CD8 {
                route: CloudDiagonalAttackRouteLoc35CD8::SpawnAttackSprite {
                    allocation_selector: 0x8000,
                    projectile_worker: "loc_35DF0",
                    x_word: 6,
                    y_word: 0,
                    projectile_x_direction: 0x00,
                },
                animation_script: Some("stru_35EA0"),
                object_45_value: Some(0x00),
            },
            "ASM Cloud.asm loc_35DAE waits for `$18(a3)`, then loc_35DB8 allocates `loc_35DF0`, copies `d1`, flips x_direction, and clears `$45(a5)`"
        );
    }

    #[test]
    fn cloud_attack_projectile_loop_loc_35e8a_waits_until_collision_or_animation_flag() {
        assert_eq!(
            cloud_attack_projectile_loop_loc_35E8A(0, false),
            CloudAttackProjectileLoopLoc35E8A {
                hibernates_one_frame: true,
                route: CloudAttackProjectileRouteLoc35E8A::ContinueLoop,
            },
            "ASM Cloud.asm loc_35E8A hibernates, then loops while `collision_type(a3)` and `$19(a3)` are both clear"
        );
        assert_eq!(
            cloud_attack_projectile_loop_loc_35E8A(0x0004, false).route,
            CloudAttackProjectileRouteLoc35E8A::DeleteTail,
            "ASM Cloud.asm loc_35E8A checks non-zero `collision_type(a3)` before byte `$19(a3)`"
        );
        assert_eq!(
            cloud_attack_projectile_loop_loc_35E8A(0, true).route,
            CloudAttackProjectileRouteLoc35E8A::DeleteTail,
            "ASM Cloud.asm loc_35E8A exits when byte `$19(a3)` is set after the collision check"
        );
    }

    #[test]
    fn cloud_attack_projectile_delete_tail_loc_35e9c_deletes_current_object() {
        assert_eq!(
            cloud_attack_projectile_delete_tail_loc_35E9C(),
            CloudAttackProjectileDeleteTailLoc35E9C {
                deletes_current_object: true,
            },
            "ASM Cloud.asm loc_35E9C is only `jmp (j_Delete_CurrentObject).w`"
        );
    }

    #[test]
    fn cloud_diagonal_attack_setup_loc_35cd8_starts_when_kid_is_in_front_band() {
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
                object_45: 0x00,
                animation_done_flag_18: false,
                x_direction: 0,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 92,
                kid_y: 58,
                attack_x_word: 0,
                attack_y_word: 0,
            }),
            CloudDiagonalAttackSetupLoc35CD8 {
                route: CloudDiagonalAttackRouteLoc35CD8::StartAttack {
                    x_word: -3,
                    y_word: 3,
                },
                animation_script: Some("stru_35EC8"),
                object_45_value: Some(0xFF),
            },
            "ASM Cloud.asm loc_35CD8 clear x_direction starts the diagonal attack only for Kid left/in-front"
        );
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
                object_45: 0x00,
                animation_done_flag_18: false,
                x_direction: 0xFF,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 108,
                kid_y: 58,
                attack_x_word: 0,
                attack_y_word: 0,
            }),
            CloudDiagonalAttackSetupLoc35CD8 {
                route: CloudDiagonalAttackRouteLoc35CD8::StartAttack {
                    x_word: 3,
                    y_word: 3,
                },
                animation_script: Some("stru_35EC8"),
                object_45_value: Some(0xFF),
            },
            "ASM Cloud.asm loc_35D1E non-zero x_direction starts the mirrored diagonal attack for Kid right/in-front"
        );
    }

    #[test]
    fn cloud_diagonal_attack_setup_loc_35d40_starts_vertical_down_in_column() {
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
                object_45: 0x00,
                animation_done_flag_18: false,
                x_direction: 0,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 100,
                kid_y: 70,
                attack_x_word: 0,
                attack_y_word: 0,
            }),
            CloudDiagonalAttackSetupLoc35CD8 {
                route: CloudDiagonalAttackRouteLoc35CD8::StartAttack {
                    x_word: 0,
                    y_word: 6,
                },
                animation_script: Some("stru_35EC8"),
                object_45_value: Some(0xFF),
            },
            "ASM Cloud.asm loc_35D40 starts a straight-down attack when Kid is below within the signed -15..16 px column"
        );
    }

    #[test]
    fn cloud_diagonal_attack_setup_loc_35d9e_initialises_attack_animation_and_latch() {
        let setup = cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
            object_45: 0x00,
            animation_done_flag_18: false,
            x_direction: 0,
            cloud_x: 100,
            cloud_y: 50,
            kid_x: 100,
            kid_y: 70,
            attack_x_word: 0,
            attack_y_word: 0,
        });

        assert_eq!(
            setup.animation_script,
            Some("stru_35EC8"),
            "ASM Cloud.asm loc_35D9E initialises the attack animation after the vector is selected"
        );
        assert_eq!(
            setup.object_45_value,
            Some(0xFF),
            "ASM Cloud.asm loc_35D9E uses `st $45(a5)` to latch an active attack"
        );
    }

    #[test]
    fn cloud_diagonal_attack_setup_loc_35d62_starts_horizontal_side_attack() {
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
                object_45: 0x00,
                animation_done_flag_18: false,
                x_direction: 0,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 90,
                kid_y: 40,
                attack_x_word: 0,
                attack_y_word: 0,
            }),
            CloudDiagonalAttackSetupLoc35CD8 {
                route: CloudDiagonalAttackRouteLoc35CD8::StartAttack {
                    x_word: -6,
                    y_word: 0,
                },
                animation_script: Some("stru_35EC8"),
                object_45_value: Some(0xFF),
            },
            "ASM Cloud.asm loc_35D62 starts a horizontal attack when Kid is within the signed +/-24 px Y window and in front of a clear-x_direction cloud"
        );
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
                object_45: 0x00,
                animation_done_flag_18: false,
                x_direction: 0xFF,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 110,
                kid_y: 40,
                attack_x_word: 0,
                attack_y_word: 0,
            }),
            CloudDiagonalAttackSetupLoc35CD8 {
                route: CloudDiagonalAttackRouteLoc35CD8::StartAttack {
                    x_word: 6,
                    y_word: 0,
                },
                animation_script: Some("stru_35EC8"),
                object_45_value: Some(0xFF),
            },
            "ASM Cloud.asm loc_35D8C mirrors the horizontal launch for non-zero x_direction after the loc_35D62 Y-window gate"
        );
    }

    #[test]
    fn cloud_death_sequence_loc_35c0e_clears_motion_updates_status_and_deletes() {
        let positive = cloud_death_sequence_loc_35C0E(0x02);
        assert_eq!(positive.x_velocity, 0);
        assert_eq!(positive.animation_script, "stru_35EBA");
        assert_eq!(
            positive.status_route,
            CloudDeathStatusRouteLoc35C0E::WriteRespawnMarker(CloudRespawnMarkerLoc35C42 {
                index: 2,
                word_value: 0x2168,
            }),
            "ASM Cloud.asm loc_35C42 writes the $2168 respawn marker for positive object-status slots"
        );
        assert_eq!(
            positive.delete_tail,
            CloudDeleteTailLoc35C50 {
                deletes_current_object: true,
            },
            "ASM Cloud.asm loc_35C50 jumps to j_Delete_CurrentObject after every status route"
        );

        assert_eq!(
            cloud_death_sequence_loc_35C0E(0xC7).status_route,
            CloudDeathStatusRouteLoc35C0E::SubtractSpawnCounter {
                index: 7,
                decrement: 0x0400,
            },
            "ASM Cloud.asm loc_35C0E masks bit-7+bit-6 status slots before decrementing the spawn counter"
        );
        assert_eq!(
            cloud_death_sequence_loc_35C0E(0x80).status_route,
            CloudDeathStatusRouteLoc35C0E::DeleteOnly,
            "ASM Cloud.asm loc_35C0E deletes without a status-table write when bit 6 is clear"
        );
    }

    #[test]
    fn cloud_flies_horizontally() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Cloud,
            x_pos: 128 << 16,
            y_pos: 100 << 16,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let x_before = enemy.x_pos;
        step_cloud(&mut enemy, None);
        assert!(enemy.x_pos < x_before, "cloud should move left");
        assert_eq!(enemy.y_pos, 100 << 16, "cloud y should not change");
    }

    #[test]
    fn cloud_reverses_at_wall() {
        // Wall at col 2 across rows 5-8
        let mut solid: Vec<(u16, u16)> = Vec::new();
        for row in 5..=8 {
            solid.push((2, row));
        }
        let grid = grid_with_solid_cells(20, 15, &solid);
        // Cloud at col 4 (x=64), y at row 6 (y=96), flying left
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Cloud,
            x_pos: 64 << 16,
            y_pos: 96 << 16,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        // Run until it hits the wall and reverses
        let mut reversed = false;
        for _ in 0..500 {
            step_cloud(&mut enemy, Some(&grid));
            if enemy.x_velocity > 0 {
                reversed = true;
                break;
            }
        }
        assert!(reversed, "cloud should reverse on hitting wall");
        assert_eq!(enemy.facing, Direction::Right);
    }

    #[test]
    fn cloud_keeps_flying_immediately_after_wall_reversal() {
        let mut solid: Vec<(u16, u16)> = Vec::new();
        for row in 5..=8 {
            solid.push((2, row));
        }
        let grid = grid_with_solid_cells(20, 15, &solid);
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Cloud,
            x_pos: 64 << 16,
            y_pos: 96 << 16,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };

        for _ in 0..500 {
            step_cloud(&mut enemy, Some(&grid));
            if enemy.x_velocity > 0 {
                break;
            }
        }

        let x_after_reversal = enemy.x_pos;
        let grounded_frames_after_reversal = enemy.grounded_frames;
        step_cloud(&mut enemy, Some(&grid));

        // ASM Cloud.asm loc_35BDE negates x_vel immediately, then the loop at
        // loc_35C54 hibernates only one frame before moving again; the `move.w
        // #$3C,d3` write does not create a 60-frame idle.
        assert_eq!(grounded_frames_after_reversal, 0);
        assert!(enemy.x_pos > x_after_reversal);
        assert_eq!(enemy.x_velocity, 0x3000);
        assert_eq!(enemy.facing, Direction::Right);
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
    fn cloud_runtime_loc_35cd8_latches_then_spawns_diagonal_lightning() {
        let mut active = vec![cloud_enemy_for_attack()];
        let mut projectiles = Vec::new();
        let mut drilled_blocks = Vec::new();

        step_active_enemies(
            &mut active,
            None,
            Some(60 << 16),
            Some(89 << 16),
            None,
            None,
            PlayerForm::TheKid,
            Some(&mut projectiles),
            &mut drilled_blocks,
        );

        assert!(
            projectiles.is_empty(),
            "ASM loc_35D9E should latch `$45(a5)` before loc_35DB8 allocates the attack worker",
        );
        assert_eq!(active[0].ufo_beam_segments, 1);
        assert_eq!(active[0].target_x_velocity, -0x3_0000);
        assert_eq!(active[0].target_y_velocity, 0x3_0000);

        step_active_enemies(
            &mut active,
            None,
            Some(60 << 16),
            Some(89 << 16),
            None,
            None,
            PlayerForm::TheKid,
            Some(&mut projectiles),
            &mut drilled_blocks,
        );

        assert_eq!(
            projectiles.len(),
            1,
            "ASM loc_35DB8 should allocate one loc_35DF0 lightning worker after the attack animation latch",
        );
        assert_eq!(active[0].ufo_beam_segments, 0);
        let projectile = projectiles[0];
        assert_eq!(projectile.x_velocity, -0x3_0000);
        assert_eq!(projectile.y_velocity, 0x3_0000);
        assert_eq!(projectile.y_pos, 39 << 16);
        assert_eq!(
            projectile.kind,
            EnemyProjectileKind::CloudLightning,
            "ASM Cloud.asm loc_35DF0 allocates the loc_35E8A lightning worker"
        );
        assert_eq!(
            projectile.lifetime, 0,
            "ASM Cloud.asm loc_35E8A has no DBF lifetime counter"
        );
        assert!(!projectile.friendly);
        assert!(projectile.terrain_kill);
    }

    #[test]
    fn cloud_lightning_loc_35e8a_persists_without_generic_lifetime_expiry() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0x1_0000,
            y_velocity: 0,
            lifetime: 0,
            friendly: false,
            terrain_kill: true,
            kind: EnemyProjectileKind::CloudLightning,
            ..Default::default()
        });

        for _ in 0..40 {
            runtime.step_projectiles();
        }

        assert_eq!(
            runtime.projectiles.len(),
            1,
            "ASM Cloud.asm loc_35E8A loops until collision_type or `$19`; the old 31-frame generic lifetime must not delete lightning"
        );
        assert_eq!(runtime.projectiles[0].x_pos, 140 << 16);
    }

    #[test]
    fn cloud_spawn_hit_points() {
        // Cloud base HP = 1
        assert_eq!(spawn_hit_points(EnemyFamily::Cloud, 0), 1);
        assert_eq!(spawn_hit_points(EnemyFamily::Cloud, 1), 2);
        assert_eq!(spawn_hit_points(EnemyFamily::Cloud, 2), 3);
    }

    // --- Tier 2 enemy wiring ---

    #[test]
    fn emo_rock_maps_to_hoverer() {
        assert_eq!(EnemyFamily::EmoRock.archetype(), EnemyArchetype::Hoverer);
        assert_eq!(EnemyFamily::EmoRock.object_type(), 0x1B);
    }

    #[test]
    fn archer_maps_to_inert() {
        assert_eq!(EnemyFamily::Archer.archetype(), EnemyArchetype::Stationary);
        assert_eq!(EnemyFamily::Archer.object_type(), 0x07);
    }

    #[test]
    fn fireball_maps_to_bouncer() {
        assert_eq!(EnemyFamily::Fireball.archetype(), EnemyArchetype::Bouncer);
        assert_eq!(EnemyFamily::Fireball.object_type(), 0x19);
    }

    #[test]
    fn fireball_gravity_scales_with_hp() {
        let make = |hp| ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Fireball,
            x_pos: 0,
            y_pos: 0,
            x_velocity: 0,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: hp,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        // Fireball.asm `Enemy19_Fireball_Init`: raw `hit_points_plus_one` indexes
        // `unk_32BA6`, so live HP 1/2/3 uses gravity $2000/$3000/$3000.
        assert_eq!(bouncer_gravity(&make(1)), 0x2000);
        assert_eq!(bouncer_gravity(&make(2)), 0x3000);
        assert_eq!(bouncer_gravity(&make(3)), 0x3000);
    }

    #[test]
    fn fireball_bounce_cap_scales_with_live_hp() {
        let make = |hp| ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Fireball,
            x_pos: 0,
            y_pos: (159u32 << 16) + 0xC000,
            x_velocity: 0,
            y_velocity: 0x80000,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: hp,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 10)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);

        let mut hp_1 = make(1);
        step_bouncer(&mut hp_1, Some(&grid));
        assert_eq!(hp_1.y_velocity, -0x4_0000);

        let mut hp_2 = make(2);
        step_bouncer(&mut hp_2, Some(&grid));
        assert_eq!(hp_2.y_velocity, -0x5_0000);

        let mut hp_3 = make(3);
        step_bouncer(&mut hp_3, Some(&grid));
        assert_eq!(hp_3.y_velocity, -0x5_0000);
    }

    #[test]
    fn fireball_floor_bounce_applies_signed_rom_perturbation_before_cap() {
        let solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 10)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Fireball,
            x_pos: 0x0123_0000,
            y_pos: (159u32 << 16) + 0xC000,
            x_velocity: 0,
            y_velocity: 0x3_0000,
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
            wait_timer: 7,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };

        let expected = {
            let mut bounced = enemy;
            let mut rng = RomRandomState::default();
            bounced.y_velocity += enemies::bouncer_gravity(&bounced);
            enemies::fireball_bounce_y_velocity_with_rng(&bounced, &mut rng, bounced.x_pos >> 16)
        };
        assert!(
            (-0x4_0000..=0x4_0000).contains(&enemies::fireball_bounce_perturbation(&enemy)),
            "ASM Enemy19_Fireball_BounceFloorCeiling keeps the perturbation inside signed ±$4000",
        );
        // Seeded RNG first byte is $C0; $C0 ^ $23 = $E3, sign-extended then << 7 = -$0E80.
        assert_eq!(enemies::fireball_bounce_perturbation(&enemy), -0x0E80);

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(enemy.y_velocity, expected);
    }

    #[test]
    fn fireball_floor_bounce_spawns_trail_child_loc_32ce4_33128() {
        let solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 10)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);
        let mut active = vec![test_enemy(
            EnemyFamily::Fireball,
            0x0123_0000,
            (159u32 << 16) + 0xC000,
            1,
        )];
        active[0].y_velocity = 0x3_0000;
        let mut projectiles = Vec::new();
        let mut drilled_blocks = Vec::new();

        step_active_enemies(
            &mut active,
            Some(&grid),
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            Some(&mut projectiles),
            &mut drilled_blocks,
        );

        assert_eq!(
            projectiles.len(),
            1,
            "ASM Fireball.asm loc_32CE4 allocates a loc_33128 trail child while Number_of_Fire_Trails < 5"
        );
        assert_eq!(projectiles[0].kind, EnemyProjectileKind::FireballTrail);
        assert_eq!(projectiles[0].lifetime, 0x96);
        assert_eq!(projectiles[0].y_pos, 0x00B0_0000);
    }

    #[test]
    fn fireball_floor_bounce_respects_five_trail_cap_loc_32ce4() {
        let solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 10)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);
        let mut active = vec![test_enemy(
            EnemyFamily::Fireball,
            0x0123_0000,
            (159u32 << 16) + 0xC000,
            1,
        )];
        active[0].y_velocity = 0x3_0000;
        let mut projectiles = vec![
            EnemyProjectile {
                kind: EnemyProjectileKind::FireballTrail,
                lifetime: 0x96,
                ..Default::default()
            };
            5
        ];
        let mut drilled_blocks = Vec::new();

        step_active_enemies(
            &mut active,
            Some(&grid),
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            Some(&mut projectiles),
            &mut drilled_blocks,
        );

        assert_eq!(
            projectiles.len(),
            5,
            "ASM Fireball.asm loc_32CE4 returns without allocation once Number_of_Fire_Trails is already five"
        );
    }

    #[test]
    fn fireball_slope_up_spawns_trail_child_loc_330d0() {
        let grid = grid_with_cells(
            10,
            10,
            &[
                (5, 5, 0x40, 0), // SlopeUp at (5, 5)
            ],
        );
        let mut active = vec![test_enemy(
            EnemyFamily::Fireball,
            (5 * 16 + 8) << 16,
            89 << 16,
            1,
        )];
        active[0].x_velocity = 0x2_0000;
        active[0].y_velocity = 0x1_0000;
        active[0].facing = Direction::Right;
        let mut projectiles = Vec::new();
        let mut drilled_blocks = Vec::new();

        step_active_enemies(
            &mut active,
            Some(&grid),
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            Some(&mut projectiles),
            &mut drilled_blocks,
        );

        assert_eq!(
            projectiles.len(),
            1,
            "ASM Fireball.asm Enemy19_Fireball_SlopeUp allocates a loc_330D0 trail child while Number_of_Fire_Trails < 5"
        );
        assert_eq!(projectiles[0].kind, EnemyProjectileKind::FireballTrail);
        assert_eq!(projectiles[0].lifetime, 0x96);
    }

    #[test]
    fn fireball_ceiling_rebound_uses_the_same_perturbed_velocity_rule() {
        let enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Fireball,
            x_pos: 0x0038_0000,
            y_pos: 0x0031_0000,
            x_velocity: 0,
            y_velocity: -0x2_8000,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 2,
            death_animation_active: false,
            grounded_frames: 0,
            tank_mouth_animation_phase: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 3,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };

        let expected = {
            let mut rng = RomRandomState::default();
            let perturbation =
                enemies::fireball_bounce_perturbation_with_rng(&mut rng, enemy.x_pos >> 16);
            -(enemy.y_velocity + perturbation)
        };

        // Seeded RNG first byte is $C0; $C0 ^ $38 = $F8, sign-extended then << 7 = -$0400.
        assert_eq!(enemies::fireball_bounce_perturbation(&enemy), -0x0400);
        assert_eq!(enemies::fireball_bounce_y_velocity(&enemy), expected);
    }

    // --- Big Hopping Skull enemy ---

    #[test]
    fn big_hopping_skull_maps_to_bouncer() {
        assert_eq!(
            EnemyFamily::BigHoppingSkull.archetype(),
            EnemyArchetype::Bouncer
        );
    }

    #[test]
    fn big_hopping_skull_hp_table() {
        // enemy_level 0->2, 1->3, anything else->4.
        assert_eq!(spawn_hit_points(EnemyFamily::BigHoppingSkull, 0), 2);
        assert_eq!(
            spawn_hit_points(EnemyFamily::BigHoppingSkull, 1),
            3,
            "ASM Big_Hopping_Skull.asm loc_3650E should map raw layout hitpoint 1 to 3 HP",
        );
        assert_eq!(
            spawn_hit_points(EnemyFamily::BigHoppingSkull, 2),
            4,
            "ASM Big_Hopping_Skull.asm loc_3651A should map raw layout hitpoint 2 to the 4-HP fallback",
        );
        assert_eq!(
            spawn_hit_points(EnemyFamily::BigHoppingSkull, 7),
            4,
            "ASM Big_Hopping_Skull.asm loc_3651A should cap every non-0/non-1 raw hitpoint value at 4 HP",
        );
    }

    #[test]
    fn big_hopping_skull_initial_velocities() {
        assert_eq!(
            spawn_x_velocity(EnemyFamily::BigHoppingSkull, 0),
            0x10000,
            "ASM Big_Hopping_Skull.asm loc_3651E should seed x_vel high word with #1",
        );
        assert_eq!(
            spawn_y_velocity(EnemyFamily::BigHoppingSkull, 0),
            -0x20000,
            "ASM Big_Hopping_Skull.asm loc_3651E should seed y_vel high word with #$FFFE",
        );
    }

    #[test]
    fn big_hopping_skull_gravity_matches_asm() {
        // ASM: addi.l #$FA0,y_vel (Big_Hopping_Skull.asm:54)
        assert_eq!(BIG_SKULL_GRAVITY, 0xFA0);
    }

    #[test]
    fn big_hopping_skull_startup_matches_loc_3652e() {
        assert_eq!(
            enemies::big_hopping_skull_startup_loc_3652e(),
            enemies::BigHoppingSkullStartupLoc3652E {
                hibernates_one_frame: true,
                checks_object_range: true,
                animation: "stru_367BE",
            },
            "ASM Big_Hopping_Skull.asm loc_3652E should hibernate once, range-check, and seed the startup animation before loc_36540",
        );
    }

    #[test]
    fn big_hopping_skull_level_height_gate_matches_loc_3656c() {
        assert_eq!(
            enemies::big_hopping_skull_level_height_gate_loc_3656c(0x004F, 0x0050),
            enemies::BigHoppingSkullLevelHeightRouteLoc3656C::ReturnLoc3657C,
            "ASM Big_Hopping_Skull.asm loc_3656C should keep the skull alive while y_pos is strictly below Level_height_pixels",
        );
        assert_eq!(
            enemies::big_hopping_skull_level_height_gate_loc_3656c(0x0050, 0x0050),
            enemies::BigHoppingSkullLevelHeightRouteLoc3656C::DeleteCurrentObject,
            "ASM Big_Hopping_Skull.asm loc_3656C should delete on the exact Level_height_pixels boundary",
        );
        assert_eq!(
            enemies::big_hopping_skull_level_height_gate_loc_3656c(0x0051, 0x0050),
            enemies::BigHoppingSkullLevelHeightRouteLoc3656C::DeleteCurrentObject,
            "ASM Big_Hopping_Skull.asm loc_3656C should delete when the signed y_pos word reaches or passes the level height",
        );
    }

    #[test]
    fn big_hopping_skull_contact_wait_matches_loc_365ac() {
        assert_eq!(
            enemies::big_hopping_skull_contact_wait_loc_365ac(false, 0x001C),
            enemies::BigHoppingSkullContactWaitLoc365AC {
                hibernates_one_frame: true,
                checks_object_range: true,
                collision_probe_extent: 8,
                route: enemies::BigHoppingSkullContactWaitRouteLoc365AC::Relaunch,
            },
            "ASM Big_Hopping_Skull.asm loc_365AC should relaunch when the d6 collision probe is zero before checking collision_type",
        );
        assert_eq!(
            enemies::big_hopping_skull_contact_wait_loc_365ac(true, 0),
            enemies::BigHoppingSkullContactWaitLoc365AC {
                hibernates_one_frame: true,
                checks_object_range: true,
                collision_probe_extent: 8,
                route: enemies::BigHoppingSkullContactWaitRouteLoc365AC::SharedTail,
            },
            "ASM Big_Hopping_Skull.asm loc_365AC should route zero collision_type to the shared tail after a non-zero d6 probe",
        );
        assert_eq!(
            enemies::big_hopping_skull_contact_wait_loc_365ac(true, 0x001C).route,
            enemies::BigHoppingSkullContactWaitRouteLoc365AC::DamageSkull,
            "ASM Big_Hopping_Skull.asm loc_365AC should route colid_hurt to skull damage",
        );
        assert_eq!(
            enemies::big_hopping_skull_contact_wait_loc_365ac(true, 0x002C).route,
            enemies::BigHoppingSkullContactWaitRouteLoc365AC::DamageSkull,
            "ASM Big_Hopping_Skull.asm loc_365AC should route colid_kidabove to the same skull damage path",
        );
        assert_eq!(
            enemies::big_hopping_skull_contact_wait_loc_365ac(true, 0x0020).route,
            enemies::BigHoppingSkullContactWaitRouteLoc365AC::SharedTail,
            "ASM Big_Hopping_Skull.asm loc_365AC should ignore non-damage side contact in this wait loop",
        );
    }

    #[test]
    fn big_hopping_skull_damage_entry_matches_loc_365d8() {
        assert_eq!(
            enemies::big_hopping_skull_damage_entry_loc_365d8(3, 0x0000_1234),
            enemies::BigHoppingSkullDamageEntryLoc365D8 {
                has_kid_collision: true,
                kid_y_velocity: i32::from_ne_bytes(0xFFFD_1234_u32.to_ne_bytes()),
                remaining_hit_points: 2,
                animation: Some("stru_367CC"),
                route: enemies::BigHoppingSkullDamageEntryRouteLoc365D8::HitAnimationThenWait,
            },
            "ASM Big_Hopping_Skull.asm loc_365D8 should set skull/Kid collision, write only Kid y_vel's high word to $FFFD, decrement d2, and play the non-lethal hit animation",
        );
        assert_eq!(
            enemies::big_hopping_skull_damage_entry_loc_365d8(1, 0x0000_ABCD),
            enemies::BigHoppingSkullDamageEntryLoc365D8 {
                has_kid_collision: true,
                kid_y_velocity: i32::from_ne_bytes(0xFFFD_ABCD_u32.to_ne_bytes()),
                remaining_hit_points: 0,
                animation: None,
                route: enemies::BigHoppingSkullDamageEntryRouteLoc365D8::DestroySkull,
            },
            "ASM Big_Hopping_Skull.asm loc_365D8 should enter the destroy path when the decremented d2 hit counter reaches zero",
        );
    }

    #[test]
    fn big_hopping_skull_hit_groan_gate_matches_loc_365f2() {
        assert_eq!(
            enemies::big_hopping_skull_hit_groan_gate_loc_365f2(true, 6),
            enemies::BigHoppingSkullHitGroanGateLoc365F2 {
                hibernates_one_frame: true,
                checks_object_range: true,
                sound_effect: Some("sfx_Big_Hopping_Skull_groan"),
                d0_preserved: true,
            },
            "ASM Big_Hopping_Skull.asm loc_365F2 should play the groan on the exact C7FB0 sprite frame with animation_timer 6",
        );
        assert_eq!(
            enemies::big_hopping_skull_hit_groan_gate_loc_365f2(false, 6).sound_effect,
            None,
            "ASM Big_Hopping_Skull.asm loc_365F2 should skip the groan on any other sprite link",
        );
        assert_eq!(
            enemies::big_hopping_skull_hit_groan_gate_loc_365f2(true, 5).sound_effect,
            None,
            "ASM Big_Hopping_Skull.asm loc_365F2 should skip the groan outside animation_timer 6",
        );
    }

    #[test]
    fn big_hopping_skull_hit_voice_gate_matches_loc_36616() {
        assert_eq!(
            enemies::big_hopping_skull_hit_voice_gate_loc_36616(true, 0x000C),
            enemies::BigHoppingSkullHitVoiceGateLoc36616 {
                sound_effect: Some("sfx_Voice_die"),
                d0_preserved: true,
            },
            "ASM Big_Hopping_Skull.asm loc_36616 should play sfx_Voice_die on the exact C7FE0 sprite frame with animation_timer $C",
        );
        assert_eq!(
            enemies::big_hopping_skull_hit_voice_gate_loc_36616(false, 0x000C).sound_effect,
            None,
            "ASM Big_Hopping_Skull.asm loc_36616 should skip the voice sound on any other sprite link",
        );
        assert_eq!(
            enemies::big_hopping_skull_hit_voice_gate_loc_36616(true, 0x000B).sound_effect,
            None,
            "ASM Big_Hopping_Skull.asm loc_36616 should skip the voice sound outside animation_timer $C",
        );
    }

    #[test]
    fn big_hopping_skull_hit_animation_wait_matches_loc_36632() {
        assert_eq!(
            enemies::big_hopping_skull_hit_animation_wait_loc_36632(true, false),
            enemies::BigHoppingSkullHitAnimationWaitLoc36632 {
                collision_probe_extent: 8,
                clear_has_kid_collision: false,
                route: enemies::BigHoppingSkullHitAnimationWaitRouteLoc36632::LoopHitAnimation,
            },
            "ASM Big_Hopping_Skull.asm loc_36632 should keep looping while the hit animation completion flag is clear",
        );
        assert_eq!(
            enemies::big_hopping_skull_hit_animation_wait_loc_36632(false, true).route,
            enemies::BigHoppingSkullHitAnimationWaitRouteLoc36632::Relaunch,
            "ASM Big_Hopping_Skull.asm loc_36632 should relaunch before testing animation completion when the d6 collision probe is zero",
        );
        assert_eq!(
            enemies::big_hopping_skull_hit_animation_wait_loc_36632(true, true),
            enemies::BigHoppingSkullHitAnimationWaitLoc36632 {
                collision_probe_extent: 8,
                clear_has_kid_collision: true,
                route: enemies::BigHoppingSkullHitAnimationWaitRouteLoc36632::SharedTail,
            },
            "ASM Big_Hopping_Skull.asm loc_36632 should clear has_kid_collision and enter the shared tail after the hit animation finishes",
        );
    }

    #[test]
    fn big_hopping_skull_death_entry_matches_loc_3664e() {
        assert_eq!(
            enemies::big_hopping_skull_death_entry_loc_3664e(),
            enemies::BigHoppingSkullDeathEntryLoc3664E {
                sound_effect: "sfx_Big_Hopping_Skull_dies",
                d0_preserved: true,
                animation: "stru_367FA",
                next_loop: "death_animation_wait",
            },
            "ASM Big_Hopping_Skull.asm loc_3664E should preserve d0, play the death sound, initialise stru_367FA, and fall through to the death wait loop",
        );
    }

    #[test]
    fn big_hopping_skull_death_wait_matches_loc_36664() {
        assert_eq!(
            enemies::big_hopping_skull_death_wait_loc_36664(true, false, -0x0001_1234),
            enemies::BigHoppingSkullDeathWaitLoc36664 {
                hibernates_one_frame: true,
                checks_object_range: true,
                collision_probe_extent: 8,
                y_velocity: -0x0001_1234,
                route: enemies::BigHoppingSkullDeathWaitRouteLoc36664::LoopDeathAnimation,
            },
            "ASM Big_Hopping_Skull.asm loc_36664 should leave y_vel unchanged and loop while d6 is non-zero and the animation flag is clear",
        );
        assert_eq!(
            enemies::big_hopping_skull_death_wait_loc_36664(false, false, 0x0000_ABCD).y_velocity,
            0x0002_ABCD,
            "ASM Big_Hopping_Skull.asm loc_36664 should write only y_vel's high word to #2 when the d6 collision probe is zero",
        );
        assert_eq!(
            enemies::big_hopping_skull_death_wait_loc_36664(true, true, 0x0003_4567).route,
            enemies::BigHoppingSkullDeathWaitRouteLoc36664::FinalizeDeletion,
            "ASM Big_Hopping_Skull.asm loc_36664 should leave the wait loop once the animation completion flag is set",
        );
    }

    #[test]
    fn big_hopping_skull_death_animation_gate_matches_loc_3667e() {
        assert_eq!(
            enemies::big_hopping_skull_death_animation_gate_loc_3667e(false),
            enemies::BigHoppingSkullDeathAnimationGateLoc3667E::LoopDeathAnimation,
            "ASM Big_Hopping_Skull.asm loc_3667E should branch back to loc_36664 while the death animation flag is clear",
        );
        assert_eq!(
            enemies::big_hopping_skull_death_animation_gate_loc_3667e(true),
            enemies::BigHoppingSkullDeathAnimationGateLoc3667E::FinalizeDeathStatus,
            "ASM Big_Hopping_Skull.asm loc_3667E should read `$42(a5)` and continue into death-status finalisation once the animation flag is set",
        );
    }

    #[test]
    fn big_hopping_skull_respawn_marker_matches_loc_366a4() {
        assert_eq!(
            enemies::big_hopping_skull_respawn_marker_loc_366a4(0x05),
            enemies::BigHoppingSkullRespawnMarkerLoc366A4 {
                index: 0x05,
                word_value: 0x2168,
            },
            "ASM Big_Hopping_Skull.asm loc_366A4 should double positive `$42(a5)` into an EnemyStatus_Table word offset and write `$2168`",
        );
    }

    #[test]
    fn big_hopping_skull_delete_tail_matches_loc_366b2() {
        assert_eq!(
            enemies::big_hopping_skull_delete_tail_loc_366b2(),
            enemies::BigHoppingSkullDeleteTailLoc366B2 {
                deletes_current_object: true,
            },
            "ASM Big_Hopping_Skull.asm loc_366B2 should jump to j_Delete_CurrentObject after every death-status route",
        );
    }

    #[test]
    fn big_hopping_skull_relaunch_entry_matches_loc_366b6() {
        let relaunch_input = enemies::BigHoppingSkullRelaunchInputLoc366B6 {
            y_pos_word: 0x004F,
            level_height_pixels: 0x0050,
            animation_finished: true,
            saved_x_velocity: i32::from_ne_bytes(0xFFFF_2222_u32.to_ne_bytes()),
            current_y_velocity: 0x0000_3333,
            x_pos_word: 0x0080,
            left_bound_word: 0x0040,
            right_bound_word: 0x00C0,
            facing: Direction::Right,
        };

        assert_eq!(
            enemies::big_hopping_skull_relaunch_entry_loc_366b6(
                enemies::BigHoppingSkullRelaunchInputLoc366B6 {
                    y_pos_word: 0x0050,
                    ..relaunch_input
                },
            )
            .route,
            enemies::BigHoppingSkullRelaunchRouteLoc366B6::DeleteCurrentObject,
            "ASM Big_Hopping_Skull.asm loc_366B6 should delete before animation work when the height gate reaches Level_height_pixels",
        );
        assert_eq!(
            enemies::big_hopping_skull_relaunch_entry_loc_366b6(
                enemies::BigHoppingSkullRelaunchInputLoc366B6 {
                    animation_finished: false,
                    ..relaunch_input
                },
            ),
            enemies::BigHoppingSkullRelaunchEntryLoc366B6 {
                checks_level_height: true,
                animation: None,
                hibernates_until_animation_finished: false,
                has_level_collision: false,
                route: enemies::BigHoppingSkullRelaunchRouteLoc366B6::ReturnToContactWait,
            },
            "ASM Big_Hopping_Skull.asm loc_366B6 should return to the contact wait while animation_finished is clear",
        );
        assert_eq!(
            enemies::big_hopping_skull_relaunch_entry_loc_366b6(relaunch_input),
            enemies::BigHoppingSkullRelaunchEntryLoc366B6 {
                checks_level_height: true,
                animation: Some("stru_367B4"),
                hibernates_until_animation_finished: true,
                has_level_collision: true,
                route: enemies::BigHoppingSkullRelaunchRouteLoc366B6::Relaunch(
                    enemies::BigHoppingSkullBoundsCorrectionLoc36770 {
                        x_velocity: i32::from_ne_bytes(0xFFFF_2222_u32.to_ne_bytes()),
                        y_velocity: i32::from_ne_bytes(0xFFFE_3333_u32.to_ne_bytes()),
                        facing: Direction::Right,
                        route: enemies::BigHoppingSkullBoundsRouteLoc36770::Unchanged,
                    },
                ),
            },
            "ASM Big_Hopping_Skull.asm loc_366B6 should initialise stru_367B4, wait for it, restore level collision, and relaunch with saved d0 x_vel",
        );
    }

    #[test]
    fn big_hopping_skull_live_floor_contact_stops_like_loc_36580() {
        let grid = grid_with_floor_at_row(14);
        let mut skull = test_enemy(EnemyFamily::BigHoppingSkull, 0x0050_0000, 0x00DF_0000, 3);
        skull.x_velocity = 0x0001_2222;
        skull.y_velocity = 0x0000_F061;
        skull.facing = Direction::Right;

        enemies::step_big_hopping_skull(&mut skull, Some(&grid));

        assert_eq!(skull.y_pos, 0x00E0_0000);
        assert_eq!(
            skull.x_velocity, 0,
            "ASM Big_Hopping_Skull.asm loc_36580 should clear x_vel during the floor contact wait",
        );
        assert_eq!(skull.y_velocity, 0);
        assert_eq!(
            skull.target_x_velocity, 0x0001_2222,
            "ASM Big_Hopping_Skull.asm loc_36580 should preserve d0=x_vel for loc_366D0",
        );
        assert_eq!(
            skull.wait_timer, BIG_HOPPING_SKULL_FLOOR_CONTACT_WAIT_FRAMES_LOC_36580,
            "ASM Big_Hopping_Skull.asm loc_36580/loc_366B6 should hibernate through the contact and relaunch animations",
        );
    }

    #[test]
    fn big_hopping_skull_live_floor_contact_timer_relaunches_like_loc_366d0() {
        let mut skull = test_enemy(EnemyFamily::BigHoppingSkull, 0x0050_0000, 0x00E0_0000, 3);
        skull.x_velocity = 0;
        skull.y_velocity = 0x0000_0001;
        skull.target_x_velocity = 0x0001_2222;
        skull.wait_timer = 1;
        skull.facing = Direction::Right;

        enemies::step_big_hopping_skull(&mut skull, None);

        assert_eq!(
            skull.x_velocity, 0x0001_2222,
            "ASM Big_Hopping_Skull.asm loc_366D0 should restore d0=x_vel after the floor wait",
        );
        assert_eq!(
            skull.y_velocity,
            i32::from_ne_bytes(0xFFFE_0001_u32.to_ne_bytes()),
            "ASM Big_Hopping_Skull.asm loc_366D0 should write only y_vel's high word to $FFFE",
        );
        assert_eq!(skull.wait_timer, 0);
    }

    #[test]
    fn big_hopping_skull_live_relaunch_applies_patrol_bounds_like_loc_36770() {
        let mut skull = test_enemy(EnemyFamily::BigHoppingSkull, 0x00C0_0000, 0x00E0_0000, 3);
        skull.x_velocity = 0;
        skull.y_velocity = 0x0000_0001;
        skull.target_x_velocity = 0x0001_2222;
        skull.wait_timer = 1;
        skull.facing = Direction::Right;

        enemies::step_big_hopping_skull_with_bounds(&mut skull, None, 0x0040, 0x00C0);

        assert_eq!(
            skull.x_velocity,
            i32::from_ne_bytes(0xFFFF_2222_u32.to_ne_bytes()),
            "ASM Big_Hopping_Skull.asm loc_36798 should force leftward relaunch at the stored right patrol bound",
        );
        assert_eq!(
            skull.y_velocity,
            i32::from_ne_bytes(0xFFFE_0001_u32.to_ne_bytes()),
            "ASM Big_Hopping_Skull.asm loc_36798 should still write y_vel high word $FFFE at the corridor edge",
        );
        assert_eq!(skull.facing, Direction::Left);
        assert_eq!(skull.wait_timer, 0);
    }

    #[test]
    fn big_hopping_skull_activation_carries_patrol_bounds_to_live_relaunch_loc_364a0_36770() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![],
            unknown_word: 0,
            header_byte: 0,
            object_count: 1,
            objects: vec![EnemyLayoutObject {
                object_type: 0x1D,
                flags: 0,
                hit_points_plus_one: 1,
                x: 0x00C0,
                y: 0x00E0,
            }],
            segments: vec![wildside_data::EnemyLayoutSegment {
                lower_bound: 0x0040,
                upper_bound: 0x0100,
                enemy_type_slots: vec![],
                object_batch_flags: 0,
                object_count: 1,
                objects: vec![EnemyLayoutObject {
                    object_type: 0x1D,
                    flags: 0,
                    hit_points_plus_one: 1,
                    x: 0x00C0,
                    y: 0x00E0,
                }],
            }],
        };
        let mut active = Vec::new();
        activate_enemies(
            &mut active,
            &layout,
            ActivationWindow {
                x_min: 0,
                x_max: 0x0200,
                y_min: 0,
                y_max: 0x0200,
            },
            0,
            0,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            0,
            0x00E0_0000,
            &HashSet::new(),
        );
        let skull = active.first_mut().expect("Big Hopping Skull should spawn");
        assert_eq!(skull.family, EnemyFamily::BigHoppingSkull);
        assert_eq!(
            skull.dragon_anchor_y, 0x0080_00C0,
            "ASM Big_Hopping_Skull.asm loc_364A0 should preserve `$3E/$40` as lower+$40 and upper-$40 for live relaunch",
        );
        skull.x_velocity = 0;
        skull.y_velocity = 0x0000_0001;
        skull.target_x_velocity = 0x0001_2222;
        skull.wait_timer = 1;
        skull.facing = Direction::Right;

        step_active_enemies(
            &mut active,
            None,
            None,
            None,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut Vec::new(),
        );

        assert_eq!(
            active[0].x_velocity,
            i32::from_ne_bytes(0xFFFF_2222_u32.to_ne_bytes()),
            "ASM Big_Hopping_Skull.asm loc_364A0 stores `$3E=lower+$40`/`$40=upper-$40`; live loc_366D0 must pass those words into loc_36798 at the right bound",
        );
        assert_eq!(
            active[0].y_velocity,
            i32::from_ne_bytes(0xFFFE_0001_u32.to_ne_bytes()),
            "ASM Big_Hopping_Skull.asm loc_366D0/loc_36798 should preserve the low y_vel word while writing high word `$FFFE`",
        );
        assert_eq!(active[0].facing, Direction::Left);
    }

    #[test]
    fn big_hopping_skull_nonfatal_damage_enters_stru_367cc_hibernate() {
        let mut skull = test_enemy(EnemyFamily::BigHoppingSkull, 0x0080_0000, 0x00E0_0000, 3);

        assert!(!skull.take_damage_or_queue_death());

        assert_eq!(skull.hit_points, 2);
        assert_eq!(
            skull.grounded_frames, BIG_HOPPING_SKULL_HIT_ANIMATION_FRAMES_LOC_365D8,
            "ASM Big_Hopping_Skull.asm loc_365D8 should hibernate through `stru_367CC` on nonfatal damage",
        );
    }

    #[test]
    fn big_hopping_skull_hit_animation_queues_stru_367cc_sounds_like_loc_365f2() {
        let mut enemies = [test_enemy(
            EnemyFamily::BigHoppingSkull,
            0x0080_0000,
            0x00E0_0000,
            2,
        )];
        enemies[0].grounded_frames = BIG_HOPPING_SKULL_HIT_ANIMATION_FRAMES_LOC_365D8 - 0x23;
        enemies[0].x_velocity = 0x0001_0000;
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
            vec![QueuedSoundEffect::BigHoppingSkullGroan],
            "ASM Big_Hopping_Skull.asm loc_365F2 should play the groan when `stru_367CC` reaches C7FB0 timer 6",
        );
        assert_eq!(
            enemies[0].x_pos, 0x0080_0000,
            "ASM loc_365F2 hibernates the object during the hit animation instead of advancing movement",
        );

        enemies[0].grounded_frames = BIG_HOPPING_SKULL_HIT_ANIMATION_FRAMES_LOC_365D8 - 0x6F;
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
            0,
            false,
            None,
            &mut rng,
            &mut drilled_blocks,
            Some(&mut sound_effects),
        );

        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::VoiceDie],
            "ASM Big_Hopping_Skull.asm loc_36616 should play Voice_die when `stru_367CC` reaches C7FE0 timer $C",
        );
    }

    #[test]
    fn big_hopping_skull_collision_dispatch_matches_loc_3674e() {
        assert_eq!(
            enemies::big_hopping_skull_collision_dispatch_loc_3674e(0),
            enemies::BigHoppingSkullCollisionDispatchLoc3674E {
                captured_collision_type: 0,
                next_collision_type: 0,
                action: enemies::BigHoppingSkullCollisionActionLoc3674E::Return,
            },
            "ASM Big_Hopping_Skull.asm loc_3674E should return without clearing when collision_type is zero",
        );
        assert_eq!(
            enemies::big_hopping_skull_collision_dispatch_loc_3674e(0x0004),
            enemies::BigHoppingSkullCollisionDispatchLoc3674E {
                captured_collision_type: 0x0004,
                next_collision_type: 0,
                action: enemies::BigHoppingSkullCollisionActionLoc3674E::WallBounce,
            },
            "ASM Big_Hopping_Skull.asm loc_3674E should clear collision_type and table-select wall bounce for right-wall contact",
        );
        assert_eq!(
            enemies::big_hopping_skull_collision_dispatch_loc_3674e(0x0010).action,
            enemies::BigHoppingSkullCollisionActionLoc3674E::CeilingBounce,
            "ASM Big_Hopping_Skull.asm loc_3674E should table-select ceiling bounce for ceiling contact",
        );
        assert_eq!(
            enemies::big_hopping_skull_collision_dispatch_loc_3674e(0x0014).action,
            enemies::BigHoppingSkullCollisionActionLoc3674E::SlopeUpDeflect,
            "ASM Big_Hopping_Skull.asm loc_3674E should table-select up-slope deflection for up-slope contact",
        );
        assert_eq!(
            enemies::big_hopping_skull_collision_dispatch_loc_3674e(0x0018).action,
            enemies::BigHoppingSkullCollisionActionLoc3674E::SlopeDownDeflect,
            "ASM Big_Hopping_Skull.asm loc_3674E should table-select down-slope deflection for down-slope contact",
        );
        assert_eq!(
            enemies::big_hopping_skull_collision_dispatch_loc_3674e(0x001C),
            enemies::BigHoppingSkullCollisionDispatchLoc3674E {
                captured_collision_type: 0x001C,
                next_collision_type: 0,
                action: enemies::BigHoppingSkullCollisionActionLoc3674E::Return,
            },
            "ASM Big_Hopping_Skull.asm loc_3674E should clear colid_hurt but return before terrain dispatch",
        );
    }

    #[test]
    fn big_hopping_skull_left_bound_launch_matches_loc_36782() {
        assert_eq!(
            enemies::big_hopping_skull_bounds_correction_loc_36770(
                0x003F,
                0x0040,
                0x00C0,
                i32::from_ne_bytes(0xFFFF_1234_u32.to_ne_bytes()),
                0x0000_ABCD,
                Direction::Left,
            ),
            enemies::BigHoppingSkullBoundsCorrectionLoc36770 {
                x_velocity: 0x0001_1234,
                y_velocity: i32::from_ne_bytes(0xFFFE_ABCD_u32.to_ne_bytes()),
                facing: Direction::Right,
                route: enemies::BigHoppingSkullBoundsRouteLoc36770::HitLeftBound,
            },
            "ASM Big_Hopping_Skull.asm loc_36782 should force rightward launch at or left of `$3E(a3)` and preserve low velocity words",
        );
    }

    #[test]
    fn big_hopping_skull_right_bound_launch_matches_loc_36798() {
        assert_eq!(
            enemies::big_hopping_skull_bounds_correction_loc_36770(
                0x00C0,
                0x0040,
                0x00C0,
                0x0001_5678,
                0x0000_9ABC,
                Direction::Right,
            ),
            enemies::BigHoppingSkullBoundsCorrectionLoc36770 {
                x_velocity: i32::from_ne_bytes(0xFFFF_5678_u32.to_ne_bytes()),
                y_velocity: i32::from_ne_bytes(0xFFFE_9ABC_u32.to_ne_bytes()),
                facing: Direction::Left,
                route: enemies::BigHoppingSkullBoundsRouteLoc36770::HitRightBound,
            },
            "ASM Big_Hopping_Skull.asm loc_36798 should force leftward launch at or right of `$40(a3)` and preserve low velocity words",
        );
    }

    #[test]
    fn big_hopping_skull_bounds_correction_matches_loc_36770() {
        assert_eq!(
            enemies::big_hopping_skull_bounds_correction_loc_36770(
                0x0080,
                0x0040,
                0x00C0,
                0x0001_5678,
                -0x0002_1234,
                Direction::Right,
            ),
            enemies::BigHoppingSkullBoundsCorrectionLoc36770 {
                x_velocity: 0x0001_5678,
                y_velocity: -0x0002_1234,
                facing: Direction::Right,
                route: enemies::BigHoppingSkullBoundsRouteLoc36770::Unchanged,
            },
            "ASM Big_Hopping_Skull.asm loc_36770 should return without edits while x_pos is inside the corridor",
        );
    }

    #[test]
    fn enemy_spawn_y_adjustment_sub_36ff4_scales_blank_top_rows_to_pixels() {
        assert_eq!(
            enemies::enemy_spawn_y_adjustment_sub_36FF4(0),
            0,
            "ASM sub_36FF4 should not move enemy Y when Number_blank_top_rows is zero",
        );
        assert_eq!(
            enemies::enemy_spawn_y_adjustment_sub_36FF4(6),
            0x60,
            "ASM sub_36FF4 should shift Number_blank_top_rows left by four to convert rows into 16px units",
        );
    }

    #[test]
    fn enemy_spawn_y_pos_sub_36ff4_applies_blank_top_rows_before_family_offsets() {
        assert_eq!(
            enemies::spawn_y_pos_sub_36FF4(EnemyFamily::BigHoppingSkull, 0x00E0, 4),
            0x0120_0000,
            "ASM sub_36FF4 should add the blank-top-row pixel shift directly onto generic enemy spawn Y",
        );
        assert_eq!(
            enemies::spawn_y_pos_sub_36FF4(EnemyFamily::DragonWalking, 0x00E0, 4),
            0x0121_0000,
            "ASM sub_36FF4 should run before Dragon Walking's extra +1px spawn offset",
        );
    }

    #[test]
    fn dragon_flying_spawn_copies_layout_position_before_script_start() {
        assert_eq!(
            enemies::spawn_x_pos(EnemyFamily::Dragon, 0x0088),
            0x0088_0000,
            "ASM Dragon_Flying.asm Enemy0C_Dragon_flying_Init copies layout +4 directly into x_pos(a3)",
        );
        assert_eq!(
            enemies::spawn_y_pos_sub_36FF4(EnemyFamily::Dragon, 0x001F, 0),
            0x001F_0000,
            "ASM Dragon_Flying.asm Enemy0C_Dragon_flying_Init copies layout +6 into y_pos(a3); kid.asm sub_36FF4 only adds blank top rows",
        );
        assert_eq!(
            enemies::spawn_y_pos_sub_36FF4(EnemyFamily::Dragon, 0x001F, 2),
            0x003F_0000,
            "ASM kid.asm sub_36FF4 shifts Number_blank_top_rows by four before adding it to Dragon Flying y_pos(a3)",
        );
    }

    // --- Sphere enemy ---

    #[test]
    fn sphere_maps_to_bouncer_archetype() {
        assert_eq!(EnemyFamily::Sphere.archetype(), EnemyArchetype::Bouncer);
    }

    #[test]
    fn sphere_spawn_velocity_is_fixed() {
        assert_eq!(spawn_x_velocity(EnemyFamily::Sphere, 0), 0x10000);
        assert_eq!(spawn_x_velocity(EnemyFamily::Sphere, 2), 0x10000);
    }

    #[test]
    fn sphere_applies_gravity() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Sphere,
            x_pos: 128 << 16,
            y_pos: 64 << 16,
            x_velocity: 0x10000,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        step_bouncer(&mut enemy, None);
        assert_eq!(
            enemy.y_velocity, SPHERE_GRAVITY,
            "gravity should be applied"
        );
        step_bouncer(&mut enemy, None);
        assert_eq!(enemy.y_velocity, SPHERE_GRAVITY * 2, "gravity accumulates");
    }

    #[test]
    fn sphere_bounces_off_floor() {
        // Floor at row 10 (y=160)
        let solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 10)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Sphere,
            x_pos: 80 << 16,
            y_pos: 155 << 16,
            x_velocity: 0x10000,
            y_velocity: 0x5000, // falling
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        // Run until it bounces (y_velocity becomes negative)
        let mut bounced = false;
        for _ in 0..50 {
            step_bouncer(&mut enemy, Some(&grid));
            if enemy.y_velocity < 0 {
                bounced = true;
                break;
            }
        }
        assert!(bounced, "sphere should bounce off floor");
    }

    #[test]
    fn sphere_bounces_off_wall() {
        // Wall at col 12 across rows 5-10
        let mut solid: Vec<(u16, u16)> = Vec::new();
        for row in 5..=10 {
            solid.push((12, row));
        }
        let grid = grid_with_solid_cells(20, 15, &solid);
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Sphere,
            x_pos: 180 << 16,
            y_pos: 112 << 16,
            x_velocity: 0x10000, // moving right
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let mut bounced = false;
        for _ in 0..50 {
            step_bouncer(&mut enemy, Some(&grid));
            if enemy.x_velocity < 0 {
                bounced = true;
                break;
            }
        }
        assert!(bounced, "sphere should bounce off wall");
    }

    // ASM Sphere.asm loc_36012: slope-up swaps velocities and negates both.
    // y_vel_new = -x_vel_old, x_vel_new = -y_vel_old.
    #[test]
    fn sphere_deflects_up_slope_by_swapping_and_negating_velocities() {
        // SlopeUp at col 5, row 5 (collision=4, slope rising left-to-right).
        // Floor_y = cell_top + (15 - x_within_cell) for SlopeUp.
        // At x=88 (center of cell 5), floor_y = 80 + 15 - 8 = 87.
        let grid = grid_with_cells(
            10,
            10,
            &[
                (5, 5, 0x40, 0), // SlopeUp at (5, 5)
            ],
        );
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Sphere,
            x_pos: (5 * 16 + 8) << 16, // center of slope cell: 88 px
            y_pos: 85 << 16,           // start above slope so gravity+vel lands on floor_y=87
            x_velocity: 0x20000,       // moving right at 2px/frame
            y_velocity: 0x20000,       // falling at 2px/frame
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        // After gravity: y_velocity = 0x20000 + 0x800 = 0x20800
        // After position: y = 85 + ~2 = ~87px (on slope floor_y=87)
        // After collision: slope deflects
        let expected_y_vel_after_gravity = 0x20000 + 0x800; // gravity applied first
        let old_x_vel = enemy.x_velocity; // 0x20000
        // y_velocity after gravity but before slope deflection:
        let old_y_vel = expected_y_vel_after_gravity;
        step_bouncer(&mut enemy, Some(&grid));
        // ASM: swap and negate both velocities
        assert_eq!(enemy.y_velocity, -old_x_vel, "SlopeUp: y_vel = -old_x_vel");
        assert_eq!(enemy.x_velocity, -old_y_vel, "SlopeUp: x_vel = -old_y_vel");
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Sphere.asm loc_36012 only rewrites x_vel/y_vel, so slope-up contact must preserve x_direction"
        );
    }

    // ASM Sphere.asm loc_36032: slope-down swaps velocities without negation.
    // y_vel_new = x_vel_old, x_vel_new = y_vel_old.
    #[test]
    fn sphere_deflects_down_slope_by_swapping_velocities() {
        // SlopeDown at col 5, row 5 (collision=5, slope falling left-to-right).
        // Floor_y = cell_top + x_within_cell for SlopeDown.
        // At x=88 (center of cell 5), floor_y = 80 + 8 = 88.
        // Start enemy AT the slope surface (y=89 just on/below floor_y=88).
        let grid = grid_with_cells(
            10,
            10,
            &[
                (5, 5, 0x50, 0), // SlopeDown at (5, 5)
            ],
        );
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Sphere,
            x_pos: (5 * 16 + 8) << 16, // center of slope cell: 88 px
            y_pos: 89 << 16,           // on slope surface (floor_y=88 at x=88, so y=89 >= floor_y)
            x_velocity: 0x20000,       // moving right at 2px/frame
            y_velocity: 0x10000,       // falling at 1px/frame
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        // After gravity: y_velocity = 0x10000 + 0x800 = 0x10800
        // Position advance: y = 89 + ~1 = ~90
        // At y~90, slope floor_y at x=90 is 80 + 10 = 90, so on_ground is true
        let expected_y_vel_after_gravity = 0x10000 + 0x800;
        let old_x_vel = enemy.x_velocity;
        let old_y_vel = expected_y_vel_after_gravity;
        step_bouncer(&mut enemy, Some(&grid));
        // ASM: swap without negation
        assert_eq!(enemy.y_velocity, old_x_vel, "SlopeDown: y_vel = old_x_vel");
        assert_eq!(enemy.x_velocity, old_y_vel, "SlopeDown: x_vel = old_y_vel");
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Sphere.asm loc_36032 only rewrites x_vel/y_vel, so slope-down contact must preserve x_direction"
        );
    }

    // ASM Fireball.asm loc_32D06/loc_32D46: same swap logic as Sphere.
    #[test]
    fn fireball_deflects_up_slope_by_swapping_and_negating_velocities() {
        let grid = grid_with_cells(
            10,
            10,
            &[
                (5, 5, 0x40, 0), // SlopeUp at (5, 5)
            ],
        );
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Fireball,
            x_pos: (5 * 16 + 8) << 16,
            y_pos: 85 << 16,
            x_velocity: 0x20000,
            y_velocity: 0x20000,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        // Fireball HP=1 gravity is $2000
        let expected_y_vel_after_gravity = 0x20000 + 0x2000;
        let old_x_vel = enemy.x_velocity;
        let old_y_vel = expected_y_vel_after_gravity;
        step_bouncer(&mut enemy, Some(&grid));
        assert_eq!(
            enemy.y_velocity, -old_x_vel,
            "Fireball SlopeUp: y_vel = -old_x_vel"
        );
        assert_eq!(
            enemy.x_velocity, -old_y_vel,
            "Fireball SlopeUp: x_vel = -old_y_vel"
        );
        assert_eq!(
            enemy.facing,
            Direction::Left,
            "Fireball SlopeUp flips facing"
        );
    }

    #[test]
    fn fireball_deflects_down_slope_by_swapping_velocities() {
        // SlopeDown at col 5, row 5 (collision=5).
        // At x=88, floor_y = 88. Start enemy on slope (y=89 >= floor_y).
        let grid = grid_with_cells(
            10,
            10,
            &[
                (5, 5, 0x50, 0), // SlopeDown at (5, 5)
            ],
        );
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Fireball,
            x_pos: (5 * 16 + 8) << 16, // 88 px
            y_pos: 89 << 16,           // on slope surface
            x_velocity: 0x20000,
            y_velocity: 0x10000,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        // Fireball HP=1 gravity is $2000
        let expected_y_vel_after_gravity = 0x10000 + 0x2000;
        let old_x_vel = enemy.x_velocity;
        let old_y_vel = expected_y_vel_after_gravity;
        step_bouncer(&mut enemy, Some(&grid));
        assert_eq!(
            enemy.y_velocity, old_x_vel,
            "Fireball SlopeDown: y_vel = old_x_vel"
        );
        assert_eq!(
            enemy.x_velocity, old_y_vel,
            "Fireball SlopeDown: x_vel = old_y_vel"
        );
        assert_eq!(
            enemy.facing,
            Direction::Left,
            "Fireball SlopeDown flips facing"
        );
    }

    // --- EmoRock enemy ---

    #[test]
    fn emo_rock_maps_to_hoverer_archetype() {
        assert_eq!(EnemyFamily::EmoRock.archetype(), EnemyArchetype::Hoverer);
    }

    #[test]
    fn emo_rock_hovers_above_player_then_drops() {
        let solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 14)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);
        let player_x: u32 = 100 << 16;
        let player_y: u32 = 200 << 16;
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::EmoRock,
            x_pos: 80 << 16,
            y_pos: 50 << 16,
            x_velocity: 0,
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
            wait_timer: 3, // short wait for test
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };

        // During hover: tracks player x, hovers above player y
        step_emo_rock(&mut enemy, Some(&grid), Some(player_x), Some(player_y));
        assert_eq!(
            enemy.x_pos, player_x,
            "should track player x while hovering"
        );
        assert!(enemy.y_pos < player_y, "should hover above player");
        assert_eq!(enemy.wait_timer, 2);

        // Count down remaining wait
        step_emo_rock(&mut enemy, Some(&grid), Some(player_x), Some(player_y));
        step_emo_rock(&mut enemy, Some(&grid), Some(player_x), Some(player_y));
        assert_eq!(enemy.wait_timer, 0);

        // Now drops: gravity applied
        step_emo_rock(&mut enemy, Some(&grid), Some(player_x), Some(player_y));
        assert!(enemy.y_velocity > 0, "should fall with positive y velocity");
        assert!(!enemy.on_ground, "should not be on ground while falling");
    }

    #[test]
    fn emo_rock_lands_and_enters_settled_wait() {
        // Floor at row 5 (y=80px)
        let solid: Vec<(u16, u16)> = (0..20).map(|col| (col, 5)).collect();
        let grid = grid_with_solid_cells(20, 15, &solid);
        let player_x: u32 = 200 << 16;
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::EmoRock,
            x_pos: 100 << 16,
            y_pos: 60 << 16, // above the floor
            x_velocity: 0,
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
            wait_timer: 0, // already past hover, will drop
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };

        // Step until it lands
        for _ in 0..100 {
            step_emo_rock(&mut enemy, Some(&grid), Some(player_x), None);
            if enemy.on_ground {
                break;
            }
        }
        assert!(enemy.on_ground, "should have landed on the floor");
        assert_eq!(
            enemy.wait_timer, EMO_ROCK_SETTLED_ANIMATION_FRAMES_LOC_362A4,
            "ASM Emo_Rock.asm loc_362A4 starts the first stru_363FE landing script"
        );

        // Once on ground, ASM loc_362B6 waits in place instead of walking.
        let x_before = enemy.x_pos;
        step_emo_rock(&mut enemy, Some(&grid), Some(player_x), None);
        assert_eq!(enemy.facing, Direction::Right, "landing should face Kid");
        assert_eq!(enemy.x_pos, x_before, "settled wait should not walk");
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(
            enemy.wait_timer,
            EMO_ROCK_SETTLED_ANIMATION_FRAMES_LOC_362A4 - 1
        );
    }

    #[test]
    fn emo_rock_settled_wait_does_not_walk_like_pacer_loc_362b6() {
        let grid = grid_with_floor_at_row(14);
        let mut enemy = test_enemy(EnemyFamily::EmoRock, 0x0064_0000, 0x00E0_0000, 1);
        enemy.on_ground = true;
        enemy.wait_timer = 2;
        enemy.facing = Direction::Left;

        step_emo_rock(
            &mut enemy,
            Some(&grid),
            Some(0x00C8_0000),
            Some(0x00E0_0000),
        );

        assert_eq!(
            enemy.x_pos, 0x0064_0000,
            "ASM Emo_Rock.asm loc_362B6 should keep the settled rock in its wait loop instead of walking toward Kid",
        );
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);
        assert_eq!(
            enemy.wait_timer, 1,
            "ASM Emo_Rock.asm loc_362B6 only exits once the landing animation-finished byte is set",
        );
        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM Emo_Rock.asm loc_36274 chooses facing at landing; loc_362B6 does not re-face Kid each frame",
        );
    }

    #[test]
    fn emo_rock_settled_wait_disappears_and_restarts_hover_loc_36320() {
        let grid = grid_with_floor_at_row(14);
        let player_x = 0x00C8_0000;
        let mut enemy = test_enemy(EnemyFamily::EmoRock, 0x0064_0000, 0x00E0_0000, 1);
        enemy.on_ground = true;
        enemy.wait_timer = 1;
        enemy.facing = Direction::Left;
        let mut rng = RomRandomState::default();
        let mut sound_effects = Vec::new();

        step_emo_rock_with_rng(
            &mut enemy,
            Some(&grid),
            Some(player_x),
            Some(0x00E0_0000),
            &mut rng,
            Some(&mut sound_effects),
        );

        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::EmoRockDisappears],
            "ASM Emo_Rock.asm loc_36320 plays sfx_Emo_Rock_disappears before restarting loc_361F4",
        );
        assert_eq!(sound_effects[0].sound_id(), 0x25);
        assert!(
            enemy.on_ground,
            "ASM Emo_Rock.asm loc_36320 hibernates through stru_36456 before restarting loc_361F4",
        );
        assert_eq!(
            enemy.grounded_frames,
            EMO_ROCK_DISAPPEAR_ANIMATION_FRAMES_LOC_36320
        );
        assert_eq!(enemy.wait_timer, 0);
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);

        for _ in 1..EMO_ROCK_DISAPPEAR_ANIMATION_FRAMES_LOC_36320 {
            step_emo_rock_with_rng(
                &mut enemy,
                Some(&grid),
                Some(player_x),
                Some(0x00E0_0000),
                &mut rng,
                Some(&mut sound_effects),
            );
            assert!(
                enemy.on_ground,
                "stru_36456 should keep hibernating until its final frame"
            );
            assert_eq!(enemy.wait_timer, 0);
        }

        step_emo_rock_with_rng(
            &mut enemy,
            Some(&grid),
            Some(player_x),
            Some(0x00E0_0000),
            &mut rng,
            Some(&mut sound_effects),
        );

        assert!(
            !enemy.on_ground,
            "loc_36320 branches back to the airborne hover cycle, not a ground walk",
        );
        assert_eq!(enemy.grounded_frames, 0);
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);
        assert!(
            (0x003C..=0x00BB).contains(&enemy.wait_timer),
            "ASM Emo_Rock.asm loc_361F4 seeds the restarted hover wait from $3C plus a masked RNG byte",
        );
        assert_eq!(
            sound_effects,
            vec![QueuedSoundEffect::EmoRockDisappears],
            "loc_36320 should only play the disappear cue once per animation"
        );
    }

    // --- Projectile system ---

    #[test]
    fn projectile_moves_by_velocity_each_step() {
        let mut projs = vec![EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0x2_0000,
            y_velocity: -0x1_0000,
            lifetime: 10,
            friendly: false,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        }];
        // Simulate step_projectiles inline (since it's on NativeRuntime)
        projs.retain_mut(|p| {
            if p.lifetime == 0 {
                return false;
            }
            p.lifetime -= 1;
            p.x_pos = (p.x_pos as i64 + i64::from(p.x_velocity)) as u32;
            p.y_pos = (p.y_pos as i64 + i64::from(p.y_velocity)) as u32;
            true
        });
        assert_eq!(projs.len(), 1);
        assert_eq!(projs[0].x_pos, 102 << 16);
        assert_eq!(projs[0].y_pos, 99 << 16);
        assert_eq!(projs[0].lifetime, 9);
    }

    #[test]
    fn projectile_removed_when_lifetime_expires() {
        let mut projs = vec![EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0x1000,
            y_velocity: 0,
            lifetime: 1,
            friendly: false,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        }];
        // Step once: lifetime goes 1→0, projectile moves
        projs.retain_mut(|p| {
            if p.lifetime == 0 {
                return false;
            }
            p.lifetime -= 1;
            p.x_pos = (p.x_pos as i64 + i64::from(p.x_velocity)) as u32;
            true
        });
        assert_eq!(
            projs.len(),
            1,
            "lifetime was 1, not 0, so still alive this step"
        );
        // Step again: lifetime is 0, removed
        projs.retain_mut(|p| {
            if p.lifetime == 0 {
                return false;
            }
            p.lifetime -= 1;
            true
        });
        assert_eq!(projs.len(), 0, "projectile removed after lifetime expires");
    }

    // --- Archer enemy ---

    #[test]
    fn archer_maps_to_stationary_archetype() {
        assert_eq!(EnemyFamily::Archer.archetype(), EnemyArchetype::Stationary);
    }

    #[test]
    fn archer_faces_player_direction() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Archer,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
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
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 10, // not ready to fire yet
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let mut projs = Vec::new();
        let player_x = Some(200u32 << 16);
        step_archer(&mut enemy, &mut projs, player_x, Some(100u32 << 16), None);
        assert_eq!(enemy.facing, Direction::Right, "should face toward player");
        assert!(projs.is_empty(), "should not fire while timer > 0");
    }

    #[test]
    fn archer_fires_arrow_when_timer_expires() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Archer,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
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
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0, // ready to fire
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let mut projs = Vec::new();
        let player_x = Some(200u32 << 16);
        let player_y = Some(100u32 << 16);
        step_archer(&mut enemy, &mut projs, player_x, player_y, None);
        assert_eq!(projs.len(), 1, "should spawn an arrow");
        assert_eq!(
            projs[0].kind,
            EnemyProjectileKind::ArcherArrow,
            "ASM Archer.asm loc_34248 counts live arrows through Number_of_Arrows"
        );
        assert!(
            projs[0].x_velocity > 0,
            "arrow should move toward player (right)"
        );
        assert_eq!(
            enemy.wait_timer, ARCHER_SHOOTING_CYCLE_DELAY_FRAMES_LOC_341BE,
            "ASM loc_341BE/loc_34248 resets to the shooting-table pose delay, not arrow_accuracy"
        );
    }

    #[test]
    fn archer_loc_34248_caps_only_live_archer_arrows() {
        let mut enemy = test_enemy(EnemyFamily::Archer, 100 << 16, 100 << 16, 1);
        enemy.wait_timer = 0;
        let mut projs = vec![
            EnemyProjectile {
                kind: EnemyProjectileKind::Generic,
                lifetime: 100,
                ..Default::default()
            },
            EnemyProjectile {
                kind: EnemyProjectileKind::RobotShot,
                lifetime: 0,
                ..Default::default()
            },
        ];

        step_archer(
            &mut enemy,
            &mut projs,
            Some(200u32 << 16),
            Some(100u32 << 16),
            None,
        );

        assert_eq!(
            projs
                .iter()
                .filter(|projectile| projectile.kind == EnemyProjectileKind::ArcherArrow)
                .count(),
            1,
            "ASM Archer.asm loc_34248 checks Number_of_Arrows, not unrelated projectiles in the shared pool"
        );

        enemy.wait_timer = 0;
        projs.push(EnemyProjectile {
            kind: EnemyProjectileKind::ArcherArrow,
            lifetime: u16::MAX,
            ..Default::default()
        });

        step_archer(
            &mut enemy,
            &mut projs,
            Some(200u32 << 16),
            Some(100u32 << 16),
            None,
        );

        assert_eq!(
            projs
                .iter()
                .filter(|projectile| projectile.kind == EnemyProjectileKind::ArcherArrow)
                .count(),
            2,
            "ASM Archer.asm loc_34248 should block firing when Number_of_Arrows is already 2"
        );
    }

    #[test]
    fn archer_angle_straight_right() {
        // Player directly right (dy=0 after 16px offset → quadrant 3 → angle 27, same as 0)
        let angle = archer_angle_to_player(100 << 16, 100 << 16, 200 << 16, (100 + 16) << 16);
        assert_eq!(angle, 27);
        // Velocity is identical to angle 0
        assert_eq!(ARROW_ANGLE_TABLE[27], ARROW_ANGLE_TABLE[0]);
    }

    #[test]
    fn archer_angle_straight_left() {
        // Player directly left (dy=0 after offset → quadrant 2 → angle 14, same vel as 13)
        let angle = archer_angle_to_player(200 << 16, 100 << 16, 100 << 16, (100 + 16) << 16);
        assert_eq!(angle, 14);
        assert_eq!(ARROW_ANGLE_TABLE[14], ARROW_ANGLE_TABLE[13]);
    }

    #[test]
    fn archer_angle_straight_up() {
        // ORIGINAL_QUIRK: ASM loc_34530 sets d7=0 then tests d7 (not d4/dy), so the
        // bmi never fires. dx==0 always returns angle 21 (straight down) regardless of
        // whether the player is above or below.
        let angle = archer_angle_to_player(100 << 16, 200 << 16, 100 << 16, 100 << 16);
        assert_eq!(
            angle, 0x15,
            "ASM dx==0 quirk: always angle 21 (straight down)"
        );
    }

    #[test]
    fn archer_angle_upper_right() {
        // ASM Archer.asm loc_344C6: right+up maps the ratio-table value
        // directly to the returned angle.
        let angle = archer_angle_to_player(100 << 16, 200 << 16, 200 << 16, 100 << 16);
        assert_eq!(angle, 0x03, "ratio 9 maps directly to sub-angle 3");
        let (xv, yv) = ARROW_ANGLE_TABLE[angle];
        assert!(xv > 0, "x velocity should be positive (rightward)");
        assert!(yv < 0, "y velocity should be negative (upward)");
    }

    #[test]
    fn archer_angle_upper_left_matches_loc_34492_ratio_path() {
        // ASM Archer.asm loc_34492 negates dx/dy for the left+up quadrant,
        // maps (abs_dy * 8 / abs_dx) through unk_34404, then returns $D - sub.
        let angle = archer_angle_to_player(200 << 16, 200 << 16, 100 << 16, 100 << 16);
        assert_eq!(angle, 0x0A, "ratio 9 maps to sub-angle 3, so $D - 3");
        let (xv, yv) = ARROW_ANGLE_TABLE[angle];
        assert!(xv < 0, "x velocity should be negative (leftward)");
        assert!(yv < 0, "y velocity should be negative (upward)");
    }

    #[test]
    fn archer_angle_lower_left_matches_loc_344e6_ratio_path() {
        // ASM Archer.asm loc_344E6: left+down maps the ratio-table value,
        // then adds $E to the returned angle.
        let angle = archer_angle_to_player(200 << 16, 100 << 16, 100 << 16, 200 << 16);
        assert_eq!(angle, 0x11, "ratio 6 maps to sub-angle 3, so $E + 3");
        let (xv, yv) = ARROW_ANGLE_TABLE[angle];
        assert!(xv < 0, "x velocity should be negative (leftward)");
        assert!(yv > 0, "y velocity should be positive (downward)");
    }

    #[test]
    fn archer_angle_lower_right_matches_loc_3450a_ratio_path() {
        // ASM Archer.asm loc_3450A: right+down maps the ratio-table value,
        // then subtracts it from $1B for the returned angle.
        let angle = archer_angle_to_player(100 << 16, 100 << 16, 200 << 16, 200 << 16);
        assert_eq!(angle, 0x18, "ratio 6 maps to sub-angle 3, so $1B - 3");
        let (xv, yv) = ARROW_ANGLE_TABLE[angle];
        assert!(xv > 0, "x velocity should be positive (rightward)");
        assert!(yv > 0, "y velocity should be positive (downward)");
    }

    #[test]
    fn archer_angle_loc_3452c_returns_quadrant_selector() {
        // ASM Archer.asm loc_3452C copies d5 to d7 after each ratio path.
        // The caller currently uses d6's angle, but d7 still identifies the
        // right+up, left+up, left+down, or right+down quadrant that produced it.
        let cases = [
            (
                "right+up",
                100 << 16,
                200 << 16,
                200 << 16,
                100 << 16,
                0x03,
                0,
            ),
            (
                "left+up",
                200 << 16,
                200 << 16,
                100 << 16,
                100 << 16,
                0x0A,
                1,
            ),
            (
                "left+down",
                200 << 16,
                100 << 16,
                100 << 16,
                200 << 16,
                0x11,
                2,
            ),
            (
                "right+down",
                100 << 16,
                100 << 16,
                200 << 16,
                200 << 16,
                0x18,
                3,
            ),
        ];

        for (case, archer_x, archer_y, player_x, player_y, expected_angle, expected_selector) in
            cases
        {
            let result = archer_angle_loc_34458(archer_x, archer_y, player_x, player_y);
            assert_eq!(result.angle, expected_angle, "{case} angle");
            assert_eq!(
                result.quadrant_selector, expected_selector,
                "{case} selector"
            );
        }
    }

    #[test]
    fn archer_fire_timer_varies_by_hp() {
        assert_eq!(ARCHER_FIRE_TIMER[0], 0x1E, "HP 1: 30 frames");
        assert_eq!(ARCHER_FIRE_TIMER[1], 0x14, "HP 2: 20 frames");
        assert_eq!(ARCHER_FIRE_TIMER[2], 0x0A, "HP 3: 10 frames");
    }

    #[test]
    fn archer_shooting_cycle_delay_loc_341be_uses_pose_table_not_arrow_accuracy() {
        assert_eq!(
            archer_shooting_cycle_delay_frames_loc_341BE(),
            0x79,
            "ASM Archer.asm loc_341BE/loc_34248 waits `$64+1`, `$06+1`, and `$0C+1` DBF frames before the zero fire sentinel"
        );
        assert_eq!(
            ARCHER_SHOOTING_CYCLE_DELAY_FRAMES_LOC_341BE,
            archer_shooting_cycle_delay_frames_loc_341BE()
        );
        assert!(
            ARCHER_SHOOTING_CYCLE_DELAY_FRAMES_LOC_341BE > ARCHER_FIRE_TIMER[0],
            "arrow_accuracy must stay an arrow re-aim cadence, not the archer firing cadence"
        );
    }

    // --- Archer arrow re-aiming (ASM loc_342D6 / loc_34308 / loc_34320) ---

    #[test]
    fn archer_dx_zero_always_returns_angle_21() {
        // ORIGINAL_QUIRK: ASM loc_34530 bug — tests d7 after clearing it, not d4 (dy).
        // Player below: angle 21 (expected).
        let below = archer_angle_to_player(100 << 16, 100 << 16, 100 << 16, 200 << 16);
        assert_eq!(below, 0x15);
        // Player above: still angle 21 (quirk — should be 6 if bug didn't exist).
        let above = archer_angle_to_player(100 << 16, 200 << 16, 100 << 16, 50 << 16);
        assert_eq!(above, 0x15);
    }

    #[test]
    fn archer_ratio_overflow_returns_vertical() {
        // Nearly vertical: abs_dy >> abs_dx → ratio >= 0x3F → loc_34546.
        // Player above (dy < 0): angle 6 (straight up).
        let angle = archer_angle_to_player(100 << 16, 200 << 16, 101 << 16, 50 << 16);
        assert_eq!(angle, 6, "nearly vertical, player above → straight up");
        // Player below (dy > 0): angle 21 (straight down).
        let angle = archer_angle_to_player(100 << 16, 50 << 16, 101 << 16, 200 << 16);
        assert_eq!(angle, 0x15, "nearly vertical, player below → straight down");
    }

    #[test]
    fn archer_shot_selection_loc_340c0_uses_fixed_opening_angles() {
        // ASM Archer.asm loc_340C0 gates the target angle into
        // vertical/horizontal pose tables before the table reader loads the
        // fixed opening angle for the first arrow.
        assert_eq!(
            archer_shot_selection_loc_340C0(5),
            ArcherShotSelectionLoc340C0 {
                initial_angle: 3,
                facing: Direction::Right,
                uses_vertical_table: true,
            }
        );
        assert_eq!(
            archer_shot_selection_loc_340C0(6),
            ArcherShotSelectionLoc340C0 {
                initial_angle: 0x0A,
                facing: Direction::Left,
                uses_vertical_table: true,
            }
        );
        assert_eq!(
            archer_shot_selection_loc_340C0(20),
            ArcherShotSelectionLoc340C0 {
                initial_angle: 0x0D,
                facing: Direction::Left,
                uses_vertical_table: false,
            }
        );
        assert_eq!(
            archer_shot_selection_loc_340C0(21),
            ArcherShotSelectionLoc340C0 {
                initial_angle: 0,
                facing: Direction::Right,
                uses_vertical_table: false,
            }
        );
    }

    #[test]
    fn archer_opening_angle_loc_3419a_selects_table_word_by_facing() {
        // ASM Archer.asm loc_3419A loads the first shooting-table word for
        // default-left and the second word when x_direction is flipped/right.
        assert_eq!(
            archer_opening_angle_loc_3419A(ArcherShootingTableLoc3419A::Vertical, Direction::Left),
            0x0A
        );
        assert_eq!(
            archer_opening_angle_loc_3419A(ArcherShootingTableLoc3419A::Vertical, Direction::Right),
            0x03
        );
        assert_eq!(
            archer_opening_angle_loc_3419A(
                ArcherShootingTableLoc3419A::Horizontal,
                Direction::Left
            ),
            0x0D
        );
        assert_eq!(
            archer_opening_angle_loc_3419A(
                ArcherShootingTableLoc3419A::Horizontal,
                Direction::Right
            ),
            0x00
        );
    }

    #[test]
    fn archer_arrow_setup_loc_341aa_sets_velocity_and_inverts_held_arrow_facing() {
        // ASM Archer.asm loc_341AA converts the chosen angle into velocity for
        // the held arrow, then writes the bitwise inverse of the archer's
        // x_direction to the held-arrow object.
        assert_eq!(
            archer_arrow_setup_loc_341AA(0x03, Direction::Right),
            ArcherArrowSetupLoc341AA {
                x_velocity: 0x18 * 0x1000,
                y_velocity: -0x18 * 0x1000,
                held_arrow_facing: Direction::Left,
            }
        );
        assert_eq!(
            archer_arrow_setup_loc_341AA(0x0A, Direction::Left),
            ArcherArrowSetupLoc341AA {
                x_velocity: -0x18 * 0x1000,
                y_velocity: -0x18 * 0x1000,
                held_arrow_facing: Direction::Right,
            }
        );
    }

    #[test]
    fn archer_shooting_entry_loc_341be_routes_table_delay_word() {
        // ASM Archer.asm loc_341BE treats the shooting-table delay word as a
        // three-way control value before either looping, updating the held
        // arrow pose, or allocating a fired arrow.
        assert_eq!(
            archer_shooting_entry_loc_341BE(-1),
            ArcherShootingEntryLoc341BE::RestartCycle
        );
        assert_eq!(
            archer_shooting_entry_loc_341BE(6),
            ArcherShootingEntryLoc341BE::PoseFrame { frame_timer: 6 }
        );
        assert_eq!(
            archer_shooting_entry_loc_341BE(0),
            ArcherShootingEntryLoc341BE::FireArrow {
                hold_frame_timer: 0x14,
                held_arrow_reset_sprite_offset: 0,
                arrow_count_increment: 1,
                held_arrow_position_nudge_px: 3,
            }
        );
    }

    #[test]
    fn archer_arrow_delete_loc_34336_decrements_arrow_count_and_deletes() {
        // ASM Archer.asm loc_34336 decrements the global Number_of_Arrows word
        // before tail-calling j_Delete_CurrentObject.
        assert_eq!(
            archer_arrow_delete_loc_34336(2),
            ArcherArrowDeleteLoc34336 {
                number_of_arrows: 1,
                deletes_current_object: true,
            }
        );
        assert_eq!(
            archer_arrow_delete_loc_34336(0).number_of_arrows,
            0xFFFF,
            "68k word decrement wraps if the counter is already zero"
        );
    }

    #[test]
    fn archer_pose_frame_loc_34222_mirrors_only_x_offset_when_facing_right() {
        // ASM Archer.asm loc_34222 reads the pose sprite, d6 X offset, and d7 Y
        // offset, then negates d6 only for flipped/right-facing archers.
        assert_eq!(
            archer_pose_frame_loc_34222(0x81A8, -14, -17, Direction::Left),
            ArcherPoseFrameLoc34222 {
                held_arrow_sprite_offset: 0x81A8,
                x_offset_px: -14,
                y_offset_px: -17,
            }
        );
        assert_eq!(
            archer_pose_frame_loc_34222(0x81A8, -14, -17, Direction::Right),
            ArcherPoseFrameLoc34222 {
                held_arrow_sprite_offset: 0x81A8,
                x_offset_px: 14,
                y_offset_px: -17,
            }
        );
    }

    #[test]
    fn archer_held_arrow_position_loc_34234_adds_offsets_to_position_words() {
        // ASM Archer.asm loc_34234 copies the archer position words to the held
        // arrow and applies the already-oriented d6/d7 offsets as word adds.
        assert_eq!(
            archer_held_arrow_position_loc_34234(100 << 16, 100 << 16, -14, -17),
            ArcherHeldArrowPositionLoc34234 {
                x_pos_word: 86,
                y_pos_word: 83,
            }
        );
        assert_eq!(
            archer_held_arrow_position_loc_34234(
                (100 << 16) | 0x8000,
                (100 << 16) | 0x4000,
                14,
                -17
            ),
            ArcherHeldArrowPositionLoc34234 {
                x_pos_word: 114,
                y_pos_word: 83,
            },
            "fractional halves do not affect the word-level ASM operation"
        );
    }

    #[test]
    fn archer_death_sequence_loc_3414a_marks_contact_bounces_kid_and_deletes() {
        // ASM Archer.asm loc_3414A applies these side effects before routing the
        // object-status byte and jumping through loc_34196 to delete the object.
        let delete_only = archer_death_sequence_loc_3414A(0x80);
        assert_eq!(
            (
                delete_only.has_kid_collision,
                delete_only.kid_y_velocity,
                delete_only.deletes_current_object,
            ),
            (true, ARCHER_KILL_BOUNCE_Y_VELOCITY_LOC_3414A, true)
        );
        assert_eq!(
            delete_only.status_route,
            ArcherDeathStatusRouteLoc3414A::DeleteOnly
        );

        assert_eq!(
            archer_death_sequence_loc_3414A(0xC7).status_route,
            ArcherDeathStatusRouteLoc3414A::SubtractSpawnCounter {
                index: 7,
                decrement: 0x0400,
            }
        );
    }

    #[test]
    fn archer_respawn_marker_loc_34188_writes_positive_status_slot() {
        // ASM Archer.asm loc_34188 doubles positive `$42(a5)` as the
        // EnemyStatus_Table word offset and writes `$2168`.
        let marker = ArcherRespawnMarkerLoc34188 {
            index: 5,
            word_value: 0x2168,
        };
        assert_eq!(archer_respawn_marker_loc_34188(0x05), marker);
        assert_eq!(
            archer_death_sequence_loc_3414A(0x05).status_route,
            ArcherDeathStatusRouteLoc3414A::WriteRespawnMarker(marker)
        );
    }

    #[test]
    fn archer_adjust_angle_no_change_when_equal() {
        assert_eq!(archer_adjust_angle(10, 10), 10);
        assert_eq!(archer_adjust_angle(0, 0), 0);
        assert_eq!(archer_adjust_angle(27, 27), 27);
    }

    #[test]
    fn archer_adjust_angle_direct_increment() {
        // target > current, diff < 14 → increment by 1
        assert_eq!(archer_adjust_angle(5, 8), 6);
        assert_eq!(archer_adjust_angle(0, 5), 1);
    }

    #[test]
    fn archer_adjust_angle_direct_decrement() {
        // target < current, |diff| < 14 → decrement by 1
        assert_eq!(archer_adjust_angle(10, 5), 9);
        assert_eq!(archer_adjust_angle(5, 2), 4);
    }

    #[test]
    fn archer_adjust_angle_wrap_forward() {
        // target < current, |diff| >= 14 → wrap forward (increment through 27→0)
        assert_eq!(archer_adjust_angle(25, 5), 26);
    }

    #[test]
    fn archer_adjust_angle_wrap_backward() {
        // target > current, diff >= 14 → wrap backward (decrement through 0→27)
        assert_eq!(archer_adjust_angle(5, 25), 4);
    }

    #[test]
    fn archer_adjust_angle_wraps_at_boundaries() {
        // Increment from 27 → wraps to 0
        assert_eq!(archer_adjust_angle(27, 1), 0);
        // Decrement from 0 → wraps to 27
        assert_eq!(archer_adjust_angle(0, 26), 27);
    }

    #[test]
    fn archer_reaim_loc_3432e_refreshes_velocity_for_adjusted_angle() {
        // ASM Archer.asm loc_3432E copies the adjusted angle from d2 to d6,
        // reruns Enemy07_ArcherArrow_AngleToSpeedAndAnim, then loops.
        assert_eq!(
            archer_reaim_loc_3432E(5, 8),
            ArcherReaimLoc3432E {
                current_angle: 6,
                x_velocity: 0,
                y_velocity: -0x20 * 0x1000,
            }
        );
    }

    #[test]
    fn archer_arrow_spawns_with_reaim_and_terrain_kill() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Archer,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let mut projs = Vec::new();
        step_archer(
            &mut enemy,
            &mut projs,
            Some(200u32 << 16),
            Some(100u32 << 16),
            None,
        );
        assert_eq!(projs.len(), 1, "should spawn one arrow");
        let arrow = &projs[0];
        assert_eq!(
            arrow.lifetime,
            u16::MAX,
            "arrows live until terrain contact"
        );
        assert!(arrow.terrain_kill, "arrows die on terrain collision");
        assert!(arrow.reaim_interval > 0, "arrows re-aim periodically");
        assert_eq!(
            arrow.reaim_interval, ARCHER_FIRE_TIMER[0],
            "re-aim interval matches Archer.asm arrow_accuracy"
        );
    }

    #[test]
    fn archer_live_step_waits_pose_table_delay_between_arrows_loc_341be() {
        let mut enemy = test_enemy(EnemyFamily::Archer, 100 << 16, 100 << 16, 1);
        enemy.wait_timer = ARCHER_FIRE_TIMER[0];
        let mut projs = Vec::new();

        for _ in 0..ARCHER_FIRE_TIMER[0] {
            step_archer(
                &mut enemy,
                &mut projs,
                Some(200u32 << 16),
                Some(100u32 << 16),
                None,
            );
        }

        assert!(
            projs.is_empty(),
            "ASM Archer.asm loc_341BE should not fire after only arrow_accuracy frames"
        );

        enemy.wait_timer = 0;
        step_archer(
            &mut enemy,
            &mut projs,
            Some(200u32 << 16),
            Some(100u32 << 16),
            None,
        );

        assert_eq!(projs.len(), 1);
        assert_eq!(
            enemy.wait_timer, ARCHER_SHOOTING_CYCLE_DELAY_FRAMES_LOC_341BE,
            "live archer should reload the full pose-table delay after firing"
        );
    }

    #[test]
    fn archer_arrow_velocity_from_angle_table() {
        // ASM Enemy07_ArcherArrow_AngleToSpeedAndAnim: byte * 0x1000.
        let (xv, yv) = arrow_velocity_from_angle(0);
        assert_eq!(xv, 0x20 * 0x1000, "angle 0 x_vel = 0x20000");
        assert_eq!(yv, 0, "angle 0 y_vel = 0");
        let (xv, yv) = arrow_velocity_from_angle(6);
        assert_eq!(xv, 0, "angle 6 x_vel = 0 (straight up)");
        assert_eq!(yv, -32 * 0x1000, "angle 6 y_vel = -0x20000");
        let (xv, yv) = arrow_velocity_from_angle(21);
        assert_eq!(xv, 0, "angle 21 x_vel = 0 (straight down)");
        assert_eq!(yv, 32 * 0x1000, "angle 21 y_vel = 0x20000");
    }

    #[test]
    fn archer_arrow_spawn_offset_applied() {
        // Arrow spawns offset from archer centre (ASM 3rd shooting animation frame).
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Archer,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let mut projs = Vec::new();
        // Player to the right → archer re-faces Right, horizontal shooting path.
        step_archer(
            &mut enemy,
            &mut projs,
            Some(200u32 << 16),
            Some(100u32 << 16),
            None,
        );
        let arrow = &projs[0];
        // Facing Right → X offset +8px (negate of -8). Y offset: horizontal → -17px.
        assert_eq!(
            arrow.x_pos >> 16,
            100 + 8,
            "right-facing: 8px right of archer"
        );
        assert_eq!(arrow.y_pos >> 16, 100 - 17, "horizontal: 17px above archer");
    }

    // --- Tank firing ---

    #[test]
    fn tank_fires_bullet_after_timer_expires() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Tank,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0x8000,
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
            wait_timer: 0, // ready to fire
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let mut projs = Vec::new();
        enemy.family = EnemyFamily::TankShooting;
        step_tank_shooting_fire(&mut enemy, &mut projs, None);
        assert_eq!(projs.len(), 1, "shooting tank should fire a bullet");
        assert!(
            projs[0].x_velocity > 0,
            "bullet should go right (same as facing)"
        );
        assert_eq!(projs[0].y_velocity, 0, "bullet should be horizontal");
        assert_eq!(
            enemy.wait_timer,
            TANK_SHOOTING_SLOW_CYCLE_FRAMES - 1,
            "ASM `stru_3B4F6` reloads a 232-frame cycle after the shot"
        );
    }

    #[test]
    fn tank_does_not_fire_while_timer_active() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::TankShooting,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0x8000,
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
            wait_timer: 10,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let mut projs = Vec::new();
        step_tank_shooting_fire(&mut enemy, &mut projs, None);
        assert!(projs.is_empty(), "should not fire while timer > 0");
        assert_eq!(enemy.wait_timer, 9, "timer should decrement");
    }

    // --- Boss tracker ---

    #[test]
    fn boss_families_map_to_tracker_archetype() {
        assert_eq!(EnemyFamily::HeadyMetal.archetype(), EnemyArchetype::Tracker);
        assert_eq!(
            EnemyFamily::Shishkaboss.archetype(),
            EnemyArchetype::Tracker
        );
        assert_eq!(
            EnemyFamily::BoomerangBosses.archetype(),
            EnemyArchetype::Tracker
        );
        assert_eq!(
            EnemyFamily::BagelBrothers.archetype(),
            EnemyArchetype::Tracker
        );
    }

    #[test]
    fn tracker_chases_player_rightward() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::HeadyMetal,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 10,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let px = Some(200u32 << 16);
        let py = Some(100u32 << 16);
        step_tracker(&mut enemy, px, py);
        assert!(
            enemy.x_pos > 100 << 16,
            "should move rightward toward player"
        );
        assert_eq!(enemy.facing, Direction::Right);
    }

    #[test]
    fn tracker_chases_player_leftward() {
        let mut enemy = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::HeadyMetal,
            x_pos: 200 << 16,
            y_pos: 100 << 16,
            x_velocity: 0,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Right,
            on_ground: false,
            hit_points: 10,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let px = Some(100u32 << 16);
        let py = Some(100u32 << 16);
        step_tracker(&mut enemy, px, py);
        assert!(
            enemy.x_pos < 200 << 16,
            "should move leftward toward player"
        );
        assert_eq!(enemy.facing, Direction::Left);
    }

    #[test]
    fn boss_fires_projectiles_when_stomped() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        // Place a boss below the player
        let boss = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::HeadyMetal,
            x_pos: 100 << 16,
            y_pos: 120 << 16,
            x_velocity: 0,
            y_velocity: 0,
            target_x_velocity: 0,
            target_y_velocity: 0,
            x_acceleration: 0,
            y_acceleration: 0,
            facing: Direction::Left,
            on_ground: false,
            hit_points: 10,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        runtime.active_enemies = vec![boss];
        runtime.projectiles.clear();

        // Simulate a stomp: player overlapping boss from above, falling.
        // Boss top = 120-8 = 112, so place player feet just inside at y=113.
        let mut slice = runtime.opening_slice.unwrap();
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 113 << 16;
        slice.player.y_velocity = 0x2_0000; // falling
        slice.player.on_ground = false;
        // Rebuild roster
        runtime.active_enemy_roster = runtime
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();
        let pre_y_vel = slice.player.y_velocity;
        let pre_y_pos = slice.player.y_pos;
        let frame_start_x_pos = slice.player.x_pos;
        let frame_start_y_pos = slice.player.y_pos;
        let stomped = runtime.process_stomps(
            &mut slice,
            pre_y_vel,
            pre_y_pos,
            frame_start_x_pos,
            frame_start_y_pos,
        );
        assert!(stomped, "should register a stomp");
        assert_eq!(
            runtime.projectiles.len(),
            2,
            "boss should fire 2 projectiles on damage"
        );
        for projectile in &runtime.projectiles {
            assert_eq!(
                projectile.x_velocity, 0x0000_7FFF,
                "ASM sub_37332/loc_37394 launches the eye right when the boss is aligned with Kid"
            );
            assert_eq!(
                projectile.y_velocity, -0x0004_0000,
                "ASM sub_37332 writes high-word `$FFFC` to y_vel for the upward eye launch"
            );
            assert_eq!(projectile.gravity, 0x0BB8, "ASM loc_3739C eye gravity");
            assert_eq!(
                projectile.homing_accel, 0,
                "ASM sub_37332 eye launch is not homing"
            );
        }
    }

    #[test]
    fn headymetal_kill_uses_long_death_wobble_effect() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        let boss = ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::HeadyMetal,
            x_pos: 100 << 16,
            y_pos: 120 << 16,
            x_velocity: 0,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        runtime.active_enemies = vec![boss];
        runtime.projectiles.clear();

        let mut slice = runtime.opening_slice.unwrap();
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 113 << 16;
        slice.player.y_velocity = 0x2_0000;
        slice.player.on_ground = false;
        runtime.active_enemy_roster = runtime
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();

        let pre_y_velocity = slice.player.y_velocity;
        let pre_y_pos = slice.player.y_pos;
        let frame_start_x_pos = slice.player.x_pos;
        let frame_start_y_pos = slice.player.y_pos;
        let stomped = runtime.process_stomps(
            &mut slice,
            pre_y_velocity,
            pre_y_pos,
            frame_start_x_pos,
            frame_start_y_pos,
        );
        assert!(stomped, "should register a stomp kill");
        assert!(runtime.active_enemies.is_empty(), "boss should be removed");
        assert_eq!(runtime.enemy_death_effects.len(), 1);
        let expected_pieces = [
            HeadyMetalDeathPieceState {
                slot_offset: 0xFB72,
                x_pos: 100 << 16,
                y_pos: 120 << 16,
                x_velocity: -0x0002_6D18,
                y_velocity: 0,
            },
            HeadyMetalDeathPieceState {
                slot_offset: 0xFB76,
                x_pos: 100 << 16,
                y_pos: 120 << 16,
                x_velocity: 0x0002_6D18,
                y_velocity: 0,
            },
            HeadyMetalDeathPieceState {
                slot_offset: 0xFB7A,
                x_pos: 100 << 16,
                y_pos: 120 << 16,
                x_velocity: 0,
                y_velocity: 0x0001_3880,
            },
            HeadyMetalDeathPieceState {
                slot_offset: 0xFBAE,
                x_pos: 100 << 16,
                y_pos: 120 << 16,
                x_velocity: -0x0002_6D18,
                y_velocity: 0,
            },
            HeadyMetalDeathPieceState {
                slot_offset: 0xFBB2,
                x_pos: 100 << 16,
                y_pos: 120 << 16,
                x_velocity: 0x0002_6D18,
                y_velocity: 0,
            },
        ];
        assert_eq!(
            runtime.enemy_death_effects[0].remaining,
            HEADYMETAL_DEATH_EFFECT_FRAMES
        );
        assert_eq!(
            runtime.enemy_death_effects[0].heady_metal_split,
            Some(HeadyMetalDeathEffectSplit {
                current_x_velocity_delta: -0x0002_6D18,
                left_worker_x_velocity_delta: 0x0002_6D18,
                centre_worker_y_velocity_delta: 0x0001_3880,
                centre_worker_sprite_label: Some("LnkTo_unk_C8458-Data_Index"),
                right_worker_x_velocity_delta: -0x0002_6D18,
                far_right_worker_x_velocity_delta: 0x0002_6D18,
                cleanup_worker_slot_offsets: [0xFB72, 0xFB76, 0xFB7A, 0xFBAE, 0xFBB2],
                pieces: expected_pieces,
            }),
            "ASM sub_378C8 loc_378D6 applies these per-worker split impulses before loc_37922 deletes the cached HeadyMetal workers"
        );
        assert_eq!(
            runtime.trace_frame().heady_metal_death_splits,
            vec![HeadyMetalDeathSplitTrace {
                current_x_velocity_delta: -0x0002_6D18,
                left_worker_x_velocity_delta: 0x0002_6D18,
                centre_worker_y_velocity_delta: 0x0001_3880,
                centre_worker_sprite_label: Some("LnkTo_unk_C8458-Data_Index".to_owned()),
                right_worker_x_velocity_delta: -0x0002_6D18,
                far_right_worker_x_velocity_delta: 0x0002_6D18,
                cleanup_worker_slot_offsets: [0xFB72, 0xFB76, 0xFB7A, 0xFBAE, 0xFBB2],
                pieces: expected_pieces
                    .iter()
                    .map(|piece| HeadyMetalDeathPieceTrace {
                        slot_offset: piece.slot_offset,
                        x_pos: piece.x_pos,
                        y_pos: piece.y_pos,
                        x_velocity: piece.x_velocity,
                        y_velocity: piece.y_velocity,
                    })
                    .collect(),
            }],
            "trace_frame should expose the live HeadyMetal sub_378C8 split impulse for oracle/debug comparisons"
        );
        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::BossDies],
            "ASM HeadyMetal sub_378C8 loc_378D6 plays `sfx_Boss_dies` as the breakup pieces scatter"
        );
    }

    #[test]
    fn multi_head_boss_spawns_three_heads() {
        use enemies::{
            BOSS_HEAD_NONE, SHISHKABOSS_HEAD_Y_OFFSETS, activate_enemies, is_multi_head_boss,
        };
        // Verify multi-head detection
        assert!(is_multi_head_boss(EnemyFamily::Shishkaboss));
        assert!(is_multi_head_boss(EnemyFamily::BoomerangBosses));
        assert!(is_multi_head_boss(EnemyFamily::BagelBrothers));
        assert!(!is_multi_head_boss(EnemyFamily::HeadyMetal));

        // Test Shishkaboss: 3 heads with Y offsets 0, +72, +144
        // object_type 0x80 = slot index 0, which resolves to enemy ID 0x22 (Shishkaboss)
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![0x22, 0, 0],
            unknown_word: 0,
            header_byte: 0,
            object_count: 1,
            objects: vec![EnemyLayoutObject {
                object_type: 0x80,
                flags: 0,
                hit_points_plus_one: 1,
                x: 200,
                y: 100,
            }],
            segments: vec![],
        };
        let window = ActivationWindow {
            x_min: 0,
            x_max: 800,
            y_min: 0,
            y_max: 600,
        };
        let mut active = Vec::new();
        activate_enemies(
            &mut active,
            &layout,
            window,
            0,
            0,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            200 << 16,
            100 << 16,
            &HashSet::new(),
        );
        assert_eq!(active.len(), 3, "Shishkaboss should spawn 3 heads");
        assert_eq!(active[0].boss_head_index, 0);
        assert_eq!(active[1].boss_head_index, 1);
        assert_eq!(active[2].boss_head_index, 2);
        assert_eq!(
            [
                active[0].boss_movement_delay_counter,
                active[1].boss_movement_delay_counter,
                active[2].boss_movement_delay_counter,
            ],
            [1, 1, 1],
            "ASM loc_39478/loc_395B4/loc_396DA seed `d0=1` before the first sub_39846 movement step"
        );
        assert_eq!(
            [
                active[0].boss_movement_script_index,
                active[1].boss_movement_script_index,
                active[2].boss_movement_script_index,
            ],
            [0, 0, 0],
            "ASM loc_39478/loc_395B4/loc_396DA seed movement script index `d1=0`"
        );
        assert_eq!(
            [
                active[0].hit_points,
                active[1].hit_points,
                active[2].hit_points
            ],
            [20, 20, 20],
            "ASM loc_394E2/loc_39608/loc_3972E writes ShishkaBoss_HitPointsPerHead `$14` to every head"
        );
        // Y offsets applied
        let base_y = 100u32 << 16;
        assert_eq!(active[0].y_pos, base_y);
        let head2_y = (base_y as i64 + (i64::from(SHISHKABOSS_HEAD_Y_OFFSETS[1]) << 16)) as u32;
        assert_eq!(active[1].y_pos, head2_y);

        // Verify non-boss enemy has BOSS_HEAD_NONE
        // object_type 0x01 = direct Diamond ID (bit 7 clear)
        let layout2 = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![],
            unknown_word: 0,
            header_byte: 0,
            object_count: 1,
            objects: vec![EnemyLayoutObject {
                object_type: 0x01,
                flags: 0,
                hit_points_plus_one: 0,
                x: 200,
                y: 100,
            }],
            segments: vec![],
        };
        let mut active2 = Vec::new();
        activate_enemies(
            &mut active2,
            &layout2,
            window,
            0,
            0,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            200 << 16,
            100 << 16,
            &HashSet::new(),
        );
        assert_eq!(active2.len(), 1);
        assert_eq!(active2[0].boss_head_index, BOSS_HEAD_NONE);
    }

    #[test]
    fn ufo_beam_fires_when_player_below() {
        use enemies::{ActiveEnemy, EnemyFamily, step_ufo_beam};

        let mut enemy = ActiveEnemy {
            family: EnemyFamily::UFO,
            x_pos: 100 << 16,
            y_pos: 50 << 16,
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
            tank_mouth_animation_phase: 0,
            damage_flash_frames: 0,
            palette_variant: 0,
            wait_timer: 0,
            layout_index: 0,
            enemy_level: 0,
            segment_index: 0,
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
            boss_head_index: enemies::BOSS_HEAD_NONE,
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
        };
        let mut projectiles = Vec::new();

        // Player directly below UFO (within 2px)
        let player_x = Some(100 << 16);
        let player_y = Some(120 << 16);

        step_ufo_beam(&mut enemy, &mut projectiles, player_x, player_y);
        assert_eq!(enemy.ufo_beam_segments, 4, "should start beam sequence");
        assert_eq!(
            enemy.ufo_beam_interval, 5,
            "ASM UFO.asm loc_34EEC starts the beam interval at 5"
        );

        // Step through interval frames to spawn segments
        for _ in 0..24 {
            step_ufo_beam(&mut enemy, &mut projectiles, player_x, player_y);
        }
        assert_eq!(
            projectiles.len(),
            3,
            "ASM UFO.asm loc_34FD6 decrements `$48(a5)` before allocating; the `$48=4` start sequence clears shooting on the fourth interval instead of spawning a fourth beam"
        );
        assert!(
            projectiles.iter().all(|p| p.y_velocity > 0),
            "beams go downward"
        );
        let first_beam = &projectiles[0];
        assert_eq!(
            first_beam.x_pos, enemy.x_pos,
            "ASM UFO.asm loc_34FD6 allocates each beam segment from the shooting UFO; native projectile bridge keeps the launcher X origin"
        );
        assert_eq!(
            first_beam.y_pos,
            enemy.y_pos + (10 << 16),
            "ASM UFO.asm Enemy0F_UFOBeam_Init adds `$A` to the source Y before the beam loop"
        );
        assert_eq!(
            first_beam.x_velocity, 0,
            "ASM UFO.asm Enemy0F_UFOBeam_Init only seeds beam y_vel"
        );
        assert_eq!(
            first_beam.y_velocity, 0x3_0000,
            "ASM UFO.asm Enemy0F_UFOBeam_Init seeds beam y_vel with `$30000`"
        );
        assert_eq!(
            first_beam.lifetime, 60,
            "native projectile bridge gives UFO beam segments a bounded lifetime while preserving loc_355D4 collision deletion"
        );
        assert_eq!(first_beam.gravity, 0);
        assert!(!first_beam.friendly);
        assert_eq!(first_beam.homing_accel, 0);
        assert_eq!(first_beam.speed_cap, 0);
        assert!(!first_beam.wall_bounce);
        assert_eq!(
            first_beam.kind,
            EnemyProjectileKind::DragonWalkingBreath,
            "native bridge keeps UFO beam segments in the hostile beam projectile bucket"
        );
    }

    #[test]
    fn loc_3215e_enemy_art_slot_assignment_matches_second_slot_before_fallback() {
        use enemies::{
            boss_enemy_art_slot_assignment_loc_32146, enemy_art_slot_assignment_sub_36E84,
        };

        let first = enemy_art_slot_assignment_sub_36E84(&[0x0022, 0x401C, 0x800D], 0x0022);
        assert_eq!(first.slot_index, 0);
        assert_eq!(first.palette_line, 1);
        assert_eq!(first.vram_tile, 0x02FB);

        let second = enemy_art_slot_assignment_sub_36E84(&[0x0022, 0x401C, 0x800D], 0x001C);
        assert_eq!(second.slot_index, 1);
        assert_eq!(
            second.palette_line, 2,
            "ASM loc_3215E should match the masked second header slot before falling through to loc_3216C's slot-2 fallback",
        );
        assert_eq!(
            second.vram_tile, 0x0409,
            "ASM loc_3215E should leave `d1=1` so loc_3216C loads the slot-1 VRAM tile `$0409` after the second-slot match",
        );

        let fallback = enemy_art_slot_assignment_sub_36E84(&[0x0022, 0x401C, 0x800D], 0x0007);
        assert_eq!(fallback.slot_index, 2);
        assert_eq!(fallback.palette_line, 2);
        assert_eq!(fallback.vram_tile, 0x0517);

        let boss_first = boss_enemy_art_slot_assignment_loc_32146(&[0x0122, 0x011C], 0x0022);
        assert_eq!(
            boss_first.slot_index, 0,
            "ASM loc_32146 masks boss header slots with `$FF`, so `$0122` matches enemy id `$22`",
        );
        let boss_second = boss_enemy_art_slot_assignment_loc_32146(&[0x0122, 0x011C], 0x001C);
        assert_eq!(
            boss_second.slot_index, 1,
            "ASM loc_3215E masks the second boss header slot with `$FF`, so `$011C` matches enemy id `$1C`",
        );
    }

    #[test]
    fn loc_3217c_loads_the_selected_enemy_art_vram_tile() {
        use enemies::enemy_art_slot_assignment_sub_36E84;

        let first = enemy_art_slot_assignment_sub_36E84(&[0x0022, 0x401C, 0x800D], 0x0022);
        assert_eq!(
            first.vram_tile, 0x02FB,
            "ASM loc_3217C should read the first `EnemyArt_VRAMTileAddresses` word when `d1=0`",
        );

        let fallback = enemy_art_slot_assignment_sub_36E84(&[0x0022, 0x401C, 0x800D], 0x0007);
        assert_eq!(
            fallback.vram_tile, 0x0517,
            "ASM loc_3217C should double the slot-2 fallback index and load the third `EnemyArt_VRAMTileAddresses` word",
        );
    }

    #[test]
    fn loc_36ea6_writes_palette_line_slot_word_and_vram_tile_from_selected_enemy_art_slot() {
        use enemies::enemy_art_slot_assignment_sub_36E84;

        let second = enemy_art_slot_assignment_sub_36E84(&[0x0022, 0x401C, 0x800D], 0x001C);
        assert_eq!(
            second.slot_index, 1,
            "ASM loc_36EA6 (kid.asm:38819) should keep `d4=1` after the masked second header slot matches the enemy id",
        );
        assert_eq!(
            second.palette_line, 2,
            "ASM loc_36EA6 should write `EnemyArt_PaletteLines[d4]` into `palette_line(a3)`",
        );
        assert_eq!(
            second.vram_tile, 0x0409,
            "ASM loc_36EA6 should double `d4` and write the selected `EnemyArt_VRAMTileAddresses` word into `vram_tile(a3)`",
        );
    }

    #[test]
    fn loc_36d16_decodes_enemy_art_palette_header_entries() {
        use enemies::{EnemyArtPaletteEntryLoc36D16, enemy_art_palette_entry_loc_36D16};

        assert_eq!(
            enemy_art_palette_entry_loc_36D16(0xC01C),
            Some(EnemyArtPaletteEntryLoc36D16 {
                enemy_id: 0x001C,
                palette_variant: 3,
                enemy_load_index_offset: 0x00E0,
            }),
            "ASM loc_36D16 (kid.asm:38603) should mask the low 12-bit enemy id, multiply it by eight for `EnemyLoad_Index`, and rotate the top two palette bits down",
        );
        assert_eq!(
            enemy_art_palette_entry_loc_36D16(0xFFFF),
            None,
            "ASM loc_36D16 should branch to loc_36E42 for a `$FFFF` blank header slot before loading art or palette data",
        );
    }

    #[test]
    fn loc_36d4e_adds_the_second_palette_pointer_word_for_nonzero_variants() {
        use enemies::enemy_palette_variant_addend_loc_36D4E;

        assert_eq!(
            enemy_palette_variant_addend_loc_36D4E(0),
            0,
            "ASM loc_36D4E is skipped when the rotated palette variant is zero",
        );
        assert_eq!(
            enemy_palette_variant_addend_loc_36D4E(1),
            4,
            "ASM loc_36D4E (kid.asm:38623) should add one longword offset for palette variant 1",
        );
        assert_eq!(
            enemy_palette_variant_addend_loc_36D4E(2),
            8,
            "ASM loc_36D4E should supply the second `+4` after variants 2 and 3 run the preceding `addi.w #4,d2`",
        );
        assert_eq!(
            enemy_palette_variant_addend_loc_36D4E(3),
            8,
            "ASM loc_36D4E should treat palette variant 3 like variant 2 after the high bits are rotated into `d1`",
        );
    }

    #[test]
    fn loc_36d52_dereferences_the_variant_adjusted_palette_pointer_offset() {
        use enemies::enemy_palette_data_index_offset_loc_36D52;

        assert_eq!(
            enemy_palette_data_index_offset_loc_36D52(0x1200, 0),
            0x1200,
            "ASM loc_36D52 (kid.asm:38626) should use the unadjusted `EnemyLoad_Index` palette offset for variant 0",
        );
        assert_eq!(
            enemy_palette_data_index_offset_loc_36D52(0x1200, 1),
            0x1204,
            "ASM loc_36D52 should dereference the first alternate palette pointer after loc_36D4E adds one longword",
        );
        assert_eq!(
            enemy_palette_data_index_offset_loc_36D52(0x1200, 2),
            0x1208,
            "ASM loc_36D52 should dereference the second alternate palette pointer after variants 2 and 3 add two longwords",
        );
    }

    #[test]
    fn loc_36e24_routes_first_slot_forest_enemy_art_through_ram_decompression() {
        use enemies::{
            EnemyArtDecompressRouteLoc36E24, EnemyFamily, enemy_art_decompress_route_loc_36E24,
        };

        assert_eq!(
            enemy_art_decompress_route_loc_36E24(0, true, EnemyFamily::Crab, false),
            EnemyArtDecompressRouteLoc36E24::ForestRam,
            "ASM loc_36E24 (kid.asm:38718) should call `j_DecompressToRAM` for the first enemy slot when `Background_theme == Forest`",
        );
        assert_eq!(
            enemy_art_decompress_route_loc_36E24(0, true, EnemyFamily::TarMonster, false),
            EnemyArtDecompressRouteLoc36E24::ForestTarMonsterRam,
            "ASM loc_36E24 should preserve the TarMonster-only forest permutation selected just before the shared RAM decompressor call",
        );
        assert_eq!(
            enemy_art_decompress_route_loc_36E24(1, true, EnemyFamily::Crab, true),
            EnemyArtDecompressRouteLoc36E24::SpecialVram,
            "ASM loc_36E24 should be skipped for later enemy slots, leaving loc_36E2C to choose the special VRAM decompressor when `d6` is non-zero",
        );
        assert_eq!(
            enemy_art_decompress_route_loc_36E24(0, false, EnemyFamily::Crab, false),
            EnemyArtDecompressRouteLoc36E24::Vram,
            "ASM loc_36E24 should be skipped when the background is not Forest, falling through to ordinary VRAM decompression",
        );
    }

    #[test]
    fn loc_36e38_routes_plain_enemy_art_to_vram_decompression() {
        use enemies::{
            EnemyArtDecompressRouteLoc36E24, EnemyFamily, enemy_art_decompress_route_loc_36E24,
        };

        assert_eq!(
            enemy_art_decompress_route_loc_36E24(2, false, EnemyFamily::Robot, false),
            EnemyArtDecompressRouteLoc36E24::Vram,
            "ASM loc_36E38 (kid.asm:38730) should call `j_DecompressToVRAM` when `d6` is zero after loc_36E2C skips Forest-only RAM decompression",
        );
    }

    #[test]
    fn loc_36e3e_restores_then_advances_until_three_enemy_art_slots_are_done() {
        use enemies::{EnemyArtLoadTailLoc36E3E, enemy_art_load_tail_loc_36E3E};

        assert_eq!(
            enemy_art_load_tail_loc_36E3E(0),
            EnemyArtLoadTailLoc36E3E {
                next_slot_index: 1,
                finished: false,
            },
            "ASM loc_36E3E (kid.asm:38733) should restore `d7`, then loc_36E42 increments it and loops while the next slot is not three",
        );
        assert_eq!(
            enemy_art_load_tail_loc_36E3E(2),
            EnemyArtLoadTailLoc36E3E {
                next_slot_index: 3,
                finished: true,
            },
            "ASM loc_36E3E should preserve the current slot through decompression so loc_36E42 exits exactly after slot two advances to three",
        );
    }

    #[test]
    fn loc_32188_pops_the_first_free_gfx_slot_and_splices_it_onto_the_owner_head() {
        use enemies::{EnemyGfxSlotLinkLoc32188, enemy_gfx_slot_link_loc_32188};

        assert_eq!(
            enemy_gfx_slot_link_loc_32188(0x00FF_2448, 0x00FF_2494, 0x00FF_2600),
            EnemyGfxSlotLinkLoc32188 {
                next_free_gfx_slot: 0x00FF_2494,
                allocated_slot_next: 0x00FF_2600,
                owner_gfx_head: 0x00FF_2448,
            },
            "ASM loc_32188 should advance `Addr_NextFreeGfxObjectSlot`, copy `0(a3)` into `0(a0)`, and install the popped slot as the new owner head",
        );
    }

    #[test]
    fn loc_3219e_marks_the_allocated_gfx_slot_active_before_the_body_clear_setup() {
        use enemies::{EnemyGfxSlotInitLoc3219E, enemy_gfx_slot_init_loc_3219E};

        assert_eq!(
            enemy_gfx_slot_init_loc_3219E(0x00FF_2448, 0x0014),
            EnemyGfxSlotInitLoc3219E {
                active_word_offset: 0x0032,
                active_word_value: 0x0001,
                next_number_gfx_objects: 0x0015,
                a1_slot_address: 0x00FF_2448,
                clear_start_offset: 0x0004,
                cleared_byte_count: 0x0048,
            },
            "ASM loc_3219E should write `#1` to `$32(a0)`, increment `Number_GfxObjects`, preserve the slot address in `a1`, and seed loc_321B2 to clear bytes `$04..$4B`",
        );
    }

    // --- UFO and Tornado flyer tests ---

    #[test]
    fn ufo_accelerates_and_moves() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 200 << 16, 100 << 16, 1);
        enemy.x_acceleration = UFO_ACCELERATION;
        enemy.y_acceleration = -UFO_ACCELERATION;
        enemy.target_x_velocity = 0x20000;
        enemy.target_y_velocity = -0x20000;
        enemy.wait_timer = UFO_PATTERN_FRAMES;
        let start_x = enemy.x_pos;
        step_ufo(&mut enemy, None);
        // After one frame: velocity = 0 + accel, position = start + velocity
        assert_eq!(enemy.x_velocity, UFO_ACCELERATION);
        assert!(
            enemy.x_pos > start_x,
            "UFO should move right with positive x accel"
        );
    }

    #[test]
    fn ufo_counts_down_existing_motion_without_reloading_until_timer_zero() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 200 << 16, 100 << 16, 1);
        enemy.x_acceleration = UFO_ACCELERATION;
        enemy.y_acceleration = -UFO_ACCELERATION;
        enemy.target_x_velocity = 0x20000;
        enemy.target_y_velocity = -0x20000;
        enemy.wait_timer = 1;
        step_ufo(&mut enemy, None);

        assert_eq!(enemy.x_acceleration, UFO_ACCELERATION);
        assert_eq!(enemy.y_acceleration, -UFO_ACCELERATION);
        assert_eq!(enemy.x_velocity, UFO_ACCELERATION);
        assert_eq!(enemy.y_velocity, -UFO_ACCELERATION);
        assert_eq!(enemy.wait_timer, 0);
    }

    #[test]
    fn ufo_timer_zero_uses_loc_35326_motion_picker_before_acceleration() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 0x0120_0000, 0x0080_0000, 1);
        enemy.wait_timer = 0;

        step_ufo(&mut enemy, None);

        assert_eq!(
            enemy.target_x_velocity, 0,
            "ASM UFO.asm loc_35326 selects entry 0 when `(rng_byte ^ x_pos.low) & $F == 0`"
        );
        assert_eq!(enemy.target_y_velocity, -0x20000);
        assert_eq!(
            enemy.x_acceleration, -0x0C00,
            "ASM UFO.asm loc_35452 substitutes `$C00` when the selected X acceleration is zero"
        );
        assert_eq!(enemy.y_acceleration, -0x1000);
        assert_eq!(
            enemy.x_velocity, 0,
            "ASM UFO.asm Enemy_HandleAcceleration clamps the timer-zero X step to target speed 0"
        );
        assert_eq!(enemy.y_velocity, -0x1000);
        assert_eq!(enemy.wait_timer, UFO_PATTERN_FRAMES - 1);
    }

    #[test]
    fn ufo_random_beam_uses_rom_rng_byte_loc_34f8e() {
        let mut rng = RomRandomState::default();
        while rng.clone().next_byte() > 4 {
            let _ = rng.next_byte();
        }
        let mut enemy = test_enemy(EnemyFamily::UFO, 0x0100_0000, 0x0080_0000, 1);
        let mut projectiles = Vec::new();

        enemies::step_ufo_beam_with_rng(&mut enemy, &mut projectiles, None, None, &mut rng, None);

        assert_eq!(
            enemy.ufo_beam_segments, 4,
            "ASM UFO.asm Enemy0F_UFO_ChkLoadBeam starts on random bytes 0..=4"
        );
        assert_eq!(enemy.ufo_beam_interval, 5);
        assert!(projectiles.is_empty());
    }

    #[test]
    fn ufo_live_step_invokes_loc_34e2a_tracking_when_kid_below_near() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 0x0120_8000, 0x0080_4000, 1);
        enemy.enemy_level = 2;
        enemy.flight_double_acceleration = true;
        enemy.flight_double_terminal_speed = true;
        enemy.y_velocity = 0x0001_0000;
        enemy.wait_timer = UFO_PATTERN_FRAMES;
        let mut active = vec![enemy];
        let mut drilled_blocks = Vec::new();

        step_active_enemies(
            &mut active,
            None,
            Some(0x0100_0000),
            Some(0x0090_0000),
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut drilled_blocks,
        );

        assert_eq!(
            active[0].x_acceleration, -0x2000,
            "ASM UFO.asm loc_34E2A should steer left and loc_34E10 should double acceleration",
        );
        assert_eq!(active[0].target_x_velocity, -0x40000);
        assert_eq!(active[0].y_acceleration, 0);
        assert_eq!(active[0].target_y_velocity, 0);
        assert_eq!(
            active[0].y_velocity, 0,
            "ASM UFO.asm loc_34E2A clears y_vel before steering toward nearby lower Kid",
        );
        assert_eq!(active[0].facing, Direction::Left);
    }

    #[test]
    fn ufo_floor_collision_uses_loc_35056_floorceil_bounce() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 72 << 16, 47 << 16, 1);
        enemy.y_velocity = 0x0001_0000;
        enemy.y_acceleration = 0;
        enemy.target_y_velocity = 0x0003_0000;
        enemy.wait_timer = UFO_PATTERN_FRAMES;
        let grid = bounce_grid(&[(4, 3)]);

        step_ufo(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.y_velocity, -0x0001_0000,
            "ASM UFO.asm loc_35056 maps colid_floor through Enemy0F_UFO_BounceFloorCeil",
        );
        assert_eq!(enemy.y_acceleration, 0);
        assert_eq!(enemy.target_y_velocity, 0);
    }

    #[test]
    fn ufo_ceiling_collision_uses_loc_35056_floorceil_bounce() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 40 << 16, 34 << 16, 1);
        enemy.y_velocity = -0x0001_0000;
        enemy.y_acceleration = 0;
        enemy.target_y_velocity = -0x0001_0000;
        enemy.wait_timer = UFO_PATTERN_FRAMES;
        let grid = bounce_grid(&[(2, 1)]);

        step_ufo(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.y_velocity, 0x0001_0000,
            "ASM UFO.asm loc_35056 maps colid_ceiling through Enemy0F_UFO_BounceFloorCeil",
        );
        assert_eq!(enemy.y_acceleration, 0);
        assert_eq!(enemy.target_y_velocity, 0);
    }

    #[test]
    fn ufo_slopeup_collision_uses_loc_35056_slope_bounce() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 72 << 16, 54 << 16, 1);
        enemy.y_velocity = 0x0001_0000;
        enemy.target_x_velocity = 0x0003_0000;
        enemy.target_y_velocity = 0x0003_0000;
        enemy.x_acceleration = 0;
        enemy.y_acceleration = 0;
        enemy.wait_timer = UFO_PATTERN_FRAMES;
        let grid = grid_with_cells(8, 8, &[(4, 3, 0x40, 0)]);

        step_ufo(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, -0x0001_0000,
            "ASM UFO.asm Enemy0F_UFO_BounceUpSlope sets x_vel to -$10000",
        );
        assert_eq!(
            enemy.y_velocity, -0x0001_0000,
            "ASM UFO.asm Enemy0F_UFO_BounceUpSlope sets y_vel to -$10000",
        );
        assert_eq!(enemy.x_acceleration, 0x800);
        assert_eq!(enemy.y_acceleration, 0x800);
        assert_eq!(enemy.target_x_velocity, 0);
        assert_eq!(enemy.target_y_velocity, 0);
    }

    #[test]
    fn ufo_slopedown_collision_uses_loc_35056_slope_bounce() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 72 << 16, 55 << 16, 1);
        enemy.y_velocity = 0x0001_0000;
        enemy.target_x_velocity = -0x0003_0000;
        enemy.target_y_velocity = 0x0003_0000;
        enemy.x_acceleration = 0;
        enemy.y_acceleration = 0;
        enemy.wait_timer = UFO_PATTERN_FRAMES;
        let grid = grid_with_cells(8, 8, &[(4, 3, 0x50, 0)]);

        step_ufo(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, 0x0001_0000,
            "ASM UFO.asm Enemy0F_UFO_BounceDownSlope sets x_vel to $10000",
        );
        assert_eq!(
            enemy.y_velocity, -0x0001_0000,
            "ASM UFO.asm Enemy0F_UFO_BounceDownSlope sets y_vel to -$10000",
        );
        assert_eq!(enemy.x_acceleration, -0x800);
        assert_eq!(enemy.y_acceleration, 0x800);
        assert_eq!(enemy.target_x_velocity, 0);
        assert_eq!(enemy.target_y_velocity, 0);
    }

    #[test]
    fn ufo_first_hurt_survives_and_primes_loc_35228() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 0x0100_0000, 0x0080_0000, 2);
        enemy.y_velocity = -0x0001_0000;
        enemy.y_acceleration = 0x0800;
        enemy.target_y_velocity = 0x0002_0000;

        assert!(
            !enemy.take_damage_or_queue_death(),
            "ASM UFO.asm Enemy0F_UFO_Hurt with `$47(a5)==0` should run loc_351EE/loc_35228 and keep the UFO alive"
        );

        assert_eq!(enemy.hit_points, 1);
        assert_eq!(
            enemy.y_velocity, 0x0003_0000,
            "ASM UFO.asm loc_35228 adds `$40000` to parent y_vel"
        );
        assert_eq!(
            enemy.y_acceleration, -0x0C00,
            "ASM UFO.asm loc_35228 returns with d2=`$FFFFF400`"
        );
        assert_eq!(enemy.target_y_velocity, 0);
        assert!(enemy.damage_flash_frames > 0);
    }

    #[test]
    fn ufo_hurt_driver_berzerker_charge_fall_matches_loc_35788() {
        use enemies::{
            UfoHurtDriverBerzerkerChargeFallLoc35788,
            ufo_hurt_driver_berzerker_charge_fall_loc_35788,
        };

        assert_eq!(
            ufo_hurt_driver_berzerker_charge_fall_loc_35788(-0x0005_0000),
            UfoHurtDriverBerzerkerChargeFallLoc35788 {
                x_velocity: -0x0002_8000,
                y_velocity: -0x0004_0000,
                has_level_collision: false,
                animation: "stru_34CAE",
                enters_falling_loop: true,
            },
            "ASM UFO.asm loc_35788 copies arithmetic half of Kid x_vel, clears `has_level_collision(a3)`, starts `stru_34CAE`, and enters the falling loop"
        );
    }

    #[test]
    fn ufo_hurt_driver_stop_tail_matches_loc_357c2() {
        use enemies::{UfoHurtDriverStopTailLoc357C2, ufo_hurt_driver_stop_tail_loc_357C2};

        assert_eq!(
            ufo_hurt_driver_stop_tail_loc_357C2(1),
            UfoHurtDriverStopTailLoc357C2 {
                x_velocity: 0,
                y_velocity: 0,
                has_level_collision: false,
                animation: "stru_34CAE",
                enters_falling_loop: true,
                waits_for_animation_before_delete: false,
            },
            "ASM UFO.asm loc_357C2 clears both velocities and enters the falling loop when j_Init_Animation returns non-zero"
        );

        assert_eq!(
            ufo_hurt_driver_stop_tail_loc_357C2(0),
            UfoHurtDriverStopTailLoc357C2 {
                x_velocity: 0,
                y_velocity: 0,
                has_level_collision: false,
                animation: "stru_34CAE",
                enters_falling_loop: false,
                waits_for_animation_before_delete: true,
            },
            "ASM UFO.asm loc_357C2 waits for the animation to finish before object deletion when j_Init_Animation returns zero"
        );
    }

    #[test]
    fn ufo_hurt_driver_delete_tail_matches_loc_357e2() {
        use enemies::{UfoHurtDriverDeleteTailLoc357E2, ufo_hurt_driver_delete_tail_loc_357E2};

        assert_eq!(
            ufo_hurt_driver_delete_tail_loc_357E2(),
            UfoHurtDriverDeleteTailLoc357E2 {
                deletes_current_object: true,
            },
            "ASM UFO.asm loc_357E2 is only the j_Delete_CurrentObject jump tail"
        );
    }

    #[test]
    fn ufo_hurt_driver_falling_loop_matches_loc_357e6() {
        use enemies::{UfoHurtDriverFallingLoopLoc357E6, ufo_hurt_driver_falling_loop_loc_357E6};

        assert_eq!(
            ufo_hurt_driver_falling_loop_loc_357E6(0x0001_0000, false),
            UfoHurtDriverFallingLoopLoc357E6 {
                y_velocity: 0x0001_4000,
                hibernates_one_frame: true,
                deletes_current_object: false,
                next_label: "loc_357E6",
            },
            "ASM UFO.asm loc_357E6 hibernates, applies `$4000` gravity, and loops while `$19(a3)` is clear"
        );

        assert_eq!(
            ufo_hurt_driver_falling_loop_loc_357E6(i32::MAX, true),
            UfoHurtDriverFallingLoopLoc357E6 {
                y_velocity: i32::MIN.wrapping_add(0x0000_3FFF),
                hibernates_one_frame: true,
                deletes_current_object: true,
                next_label: "loc_357E2",
            },
            "ASM UFO.asm loc_357E6 still applies gravity before deleting when `$19(a3)` is set"
        );
    }

    #[test]
    fn ufo_hurt_driver_fall_turnaround_matches_loc_357fa() {
        use enemies::{
            UfoHurtDriverFallTurnaroundLoc357FA, ufo_hurt_driver_fall_turnaround_loc_357FA,
        };

        assert_eq!(
            ufo_hurt_driver_fall_turnaround_loc_357FA(-0x0000_8000, 0x0010_0000, 0),
            UfoHurtDriverFallTurnaroundLoc357FA {
                x_velocity: 0x0000_8000,
                x_pos: 0x0012_0000,
                x_direction: 0xFF,
            },
            "ASM UFO.asm loc_357FA negates x_vel, applies four frames of reversed motion to x_pos, and flips clear x_direction to `$FF`"
        );

        assert_eq!(
            ufo_hurt_driver_fall_turnaround_loc_357FA(0x0000_8000, 0x0010_0000, 0xFF),
            UfoHurtDriverFallTurnaroundLoc357FA {
                x_velocity: -0x0000_8000,
                x_pos: 0x000E_0000,
                x_direction: 0,
            },
            "ASM UFO.asm loc_357FA uses longword wrapping for displacement and `not.b` for raw direction"
        );
    }

    #[test]
    fn ufo_hurt_driver_vertical_bounce_matches_loc_35810() {
        use enemies::{
            UfoHurtDriverVerticalBounceLoc35810, ufo_hurt_driver_vertical_bounce_loc_35810,
        };

        assert_eq!(
            ufo_hurt_driver_vertical_bounce_loc_35810(0x0004_0000),
            UfoHurtDriverVerticalBounceLoc35810 {
                y_velocity: -0x0002_0000,
            },
            "ASM UFO.asm loc_35810 negates y_vel and arithmetic-shifts the rebound down by one"
        );

        assert_eq!(
            ufo_hurt_driver_vertical_bounce_loc_35810(i32::MIN),
            UfoHurtDriverVerticalBounceLoc35810 {
                y_velocity: -0x4000_0000,
            },
            "ASM UFO.asm loc_35810 uses 68k longword wrapping before the signed `asr.l #1`"
        );
    }

    #[test]
    fn ufo_hurt_driver_floor_landing_matches_loc_3581e() {
        use enemies::{
            UfoHurtDriverFloorLandingLoc3581E, UfoHurtDriverSlideSeedLoc3566C,
            ufo_hurt_driver_floor_landing_loc_3581E,
        };

        assert_eq!(
            ufo_hurt_driver_floor_landing_loc_3581E(0x0127_8000, 0),
            UfoHurtDriverFloorLandingLoc3581E {
                stack_bytes_discarded: 4,
                y_velocity: 0,
                y_pos: 0x0120_0000,
                next_label: "loc_3566C",
                slide_seed: UfoHurtDriverSlideSeedLoc3566C {
                    x_velocity: -0x0000_8000,
                    falls_through_to_slide_loop: true,
                },
            },
            "ASM UFO.asm loc_3581E clears y_vel, clears the y_pos fraction, adds 8 to the integer word, aligns it down to 16px, and branches to loc_3566C"
        );

        assert_eq!(
            ufo_hurt_driver_floor_landing_loc_3581E(0xFFF9_1234, 0xFF),
            UfoHurtDriverFloorLandingLoc3581E {
                stack_bytes_discarded: 4,
                y_velocity: 0,
                y_pos: 0,
                next_label: "loc_3566C",
                slide_seed: UfoHurtDriverSlideSeedLoc3566C {
                    x_velocity: 0x0000_8000,
                    falls_through_to_slide_loop: true,
                },
            },
            "ASM UFO.asm loc_3581E uses 16-bit word wrapping for the aligned integer Y and preserves loc_3566C raw direction semantics"
        );
    }

    #[test]
    fn ufo_hurt_driver_up_left_launch_matches_loc_35838() {
        use enemies::{UfoHurtDriverLaunchLoc35838, ufo_hurt_driver_up_left_launch_loc_35838};

        assert_eq!(
            ufo_hurt_driver_up_left_launch_loc_35838(),
            UfoHurtDriverLaunchLoc35838 {
                x_velocity: -0x0001_0000,
                y_velocity: -0x0001_0000,
            },
            "ASM UFO.asm loc_35838 writes -$10000 to both x_vel and y_vel before returning"
        );
    }

    #[test]
    fn ufo_hurt_driver_up_right_launch_matches_loc_35864() {
        use enemies::{UfoHurtDriverLaunchLoc35864, ufo_hurt_driver_up_right_launch_loc_35864};

        assert_eq!(
            ufo_hurt_driver_up_right_launch_loc_35864(),
            UfoHurtDriverLaunchLoc35864 {
                x_velocity: 0x0001_0000,
                y_velocity: -0x0001_0000,
            },
            "ASM UFO.asm loc_35864 writes +$10000 to x_vel and -$10000 to y_vel before returning"
        );
    }

    #[test]
    fn tornado_loads_rom_pattern_on_timer_expiry() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 0, 0, 1);
        enemy.wait_timer = 0;
        step_tornado(&mut enemy, None);
        // ASM Tornado.asm loc_359F2 / unk_35A0C: timer expiry loads script entry 0.
        assert_eq!(enemy.target_x_velocity, 0x0001_8000);
        assert_eq!(enemy.target_y_velocity, -0x0001_8000);
        assert_eq!(enemy.x_acceleration, 0x0000_1800);
        assert_eq!(enemy.y_acceleration, -0x0000_1800);
        assert_eq!(enemy.wait_timer, TORNADO_PATTERN_FRAMES[0]);
    }

    #[test]
    fn tornado_pattern_tail_loc_35a02_advances_and_wraps_after_last_entry() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 0, 0, 1);
        enemy.dragon_script_cmd = (TORNADO_PATTERN_SCRIPT.len() - 1) as u8;

        reload_tornado_pattern_script(&mut enemy);

        assert_eq!(
            enemy.target_x_velocity, -0x0001_8000,
            "ASM Tornado.asm loc_35A02 uses the current a0 entry as a4 before adding `$10` to the next script pointer"
        );
        assert_eq!(enemy.target_y_velocity, 0x0001_8000);
        assert_eq!(enemy.x_acceleration, -0x0000_1800);
        assert_eq!(enemy.y_acceleration, 0x0000_1800);
        assert_eq!(
            enemy.dragon_script_cmd, 0,
            "ASM Tornado.asm loc_359F2 sees the `$00000001` sentinel after loc_35A02 advances past the last entry and reloads unk_35A0C"
        );
    }

    #[test]
    fn tornado_loc_3592e_level_1_doubles_x_script_terms() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 0, 0, 2);
        enemy.enemy_level = 1;
        enemy.flight_double_acceleration = true;
        enemy.wait_timer = 0;
        step_tornado(&mut enemy, None);
        // ASM Tornado.asm loc_3592E: `$46(a5)` doubles d0 and d2 only.
        assert_eq!(enemy.target_x_velocity, 0x0003_0000);
        assert_eq!(enemy.target_y_velocity, -0x0001_8000);
        assert_eq!(enemy.x_acceleration, 0x0000_3000);
        // ASM Tornado.asm loc_3593A: `$47(a5)=0` branches to return_35946,
        // leaving d1 and d3 unscaled for level-1 Tornado.
        assert_eq!(enemy.y_acceleration, -0x0000_1800);
        assert_eq!(enemy.wait_timer, TORNADO_PATTERN_FRAMES[1]);
    }

    #[test]
    fn tornado_loc_3593a_level_2_doubles_y_script_terms() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 0, 0, 3);
        enemy.enemy_level = 2;
        enemy.flight_double_acceleration = true;
        enemy.flight_double_terminal_speed = true;
        enemy.wait_timer = 0;
        step_tornado(&mut enemy, None);
        // ASM Tornado.asm loc_3593A: `$47(a5)` doubles d1 and d3.
        assert_eq!(enemy.target_x_velocity, 0x0003_0000);
        assert_eq!(enemy.x_acceleration, 0x0000_3000);
        assert_eq!(enemy.target_y_velocity, -0x0003_0000);
        assert_eq!(enemy.y_acceleration, -0x0000_3000);
        assert_eq!(enemy.wait_timer, TORNADO_PATTERN_FRAMES[2]);
    }

    #[test]
    fn tornado_accelerates_toward_pattern_targets() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 100 << 16, 100 << 16, 1);
        enemy.target_x_velocity = 0x0001_0000;
        enemy.target_y_velocity = -0x0001_0000;
        enemy.x_acceleration = 0x0000_0800;
        enemy.y_acceleration = -0x0000_0800;
        step_tornado(&mut enemy, None);
        assert_eq!(enemy.x_velocity, 0x0000_0800);
        assert_eq!(enemy.y_velocity, -0x0000_0800);
    }

    #[test]
    fn tornado_loc_35908_decrements_timer_without_reloading_pattern() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 100 << 16, 100 << 16, 1);
        enemy.target_x_velocity = 0x0001_0000;
        enemy.target_y_velocity = -0x0001_0000;
        enemy.x_acceleration = 0x0000_0800;
        enemy.y_acceleration = -0x0000_0800;
        enemy.wait_timer = 2;
        enemy.dragon_script_cmd = 7;

        step_tornado(&mut enemy, None);

        // ASM Tornado.asm loc_35908: `subq.w #1,$44(a5); bne.s loc_35908`
        // keeps the current script while the countdown remains non-zero.
        assert_eq!(enemy.wait_timer, 1);
        assert_eq!(enemy.dragon_script_cmd, 7);
        assert_eq!(enemy.x_velocity, 0x0000_0800);
        assert_eq!(enemy.y_velocity, -0x0000_0800);
    }

    #[test]
    fn tornado_can_reach_doubled_level_2_target_speed() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 100 << 16, 100 << 16, 3);
        enemy.target_x_velocity = 0x0004_0000;
        enemy.target_y_velocity = 0;
        enemy.x_acceleration = 0x0000_2000;
        enemy.y_acceleration = 0;
        enemy.wait_timer = 600;
        for _ in 0..512 {
            step_tornado(&mut enemy, None);
        }
        assert_eq!(
            enemy.x_velocity, 0x0004_0000,
            "ASM Enemy_HandleAcceleration caps at the scaled target speed, not a fixed $18000 ceiling",
        );
    }

    #[test]
    fn tornado_wall_collision_bounces_x_velocity_loc_35948() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 40 << 16, 48 << 16, 1);
        enemy.x_velocity = 0x0002_0000;
        enemy.target_x_velocity = enemy.x_velocity;
        enemy.x_acceleration = 0x0000_0800;
        enemy.wait_timer = 4;
        let grid = bounce_grid(&[(3, 2)]);

        step_tornado(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, -0x0002_0000,
            "ASM Tornado.asm loc_35948 maps wall collisions through UFO.asm loc_35120, negating x_vel",
        );
        assert_eq!(enemy.x_acceleration, -0x0000_0800);
        assert_eq!(enemy.target_x_velocity, 0);
        assert_eq!(enemy.facing, Direction::Right);
    }

    #[test]
    fn tornado_floor_collision_bounces_only_while_falling_loc_35948() {
        let grid = bounce_grid(&[(2, 3)]);

        let mut falling = test_enemy(EnemyFamily::Tornado, 40 << 16, 47 << 16, 1);
        falling.y_velocity = 0x0001_0000;
        falling.target_y_velocity = falling.y_velocity;
        falling.y_acceleration = 0x0000_0800;
        falling.wait_timer = 4;
        step_tornado(&mut falling, Some(&grid));
        assert_eq!(
            falling.y_velocity, -0x0001_0000,
            "ASM Tornado.asm loc_35948 should bounce floor contact only after a downward y_vel reaches the floor",
        );
        assert_eq!(falling.y_acceleration, -0x0000_0800);
        assert_eq!(falling.target_y_velocity, 0);

        let mut stationary = test_enemy(EnemyFamily::Tornado, 40 << 16, 48 << 16, 1);
        stationary.y_velocity = 0;
        stationary.target_y_velocity = 0;
        stationary.y_acceleration = 0x0000_0800;
        stationary.wait_timer = 4;
        step_tornado(&mut stationary, Some(&grid));
        assert_eq!(
            stationary.y_acceleration, 0x0000_0800,
            "ASM Tornado.asm loc_35948 should not floor-bounce the zero-y-velocity boundary",
        );

        let mut rising = test_enemy(EnemyFamily::Tornado, 40 << 16, 49 << 16, 1);
        rising.y_velocity = -0x0001_0000;
        rising.target_y_velocity = rising.y_velocity;
        rising.y_acceleration = 0x0000_0800;
        rising.wait_timer = 4;
        step_tornado(&mut rising, Some(&grid));
        assert_eq!(
            rising.y_velocity, -0x0001_0000,
            "ASM Tornado.asm loc_35948 should not floor-bounce while the Tornado is still moving upward",
        );
    }

    #[test]
    fn tornado_ceiling_collision_bounces_y_velocity_loc_35948() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 40 << 16, 34 << 16, 1);
        enemy.y_velocity = -0x0002_0000;
        enemy.target_y_velocity = enemy.y_velocity;
        enemy.y_acceleration = 0;
        enemy.wait_timer = 4;
        let grid = bounce_grid(&[(2, 1)]);

        step_tornado(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.y_velocity, 0x0002_0000,
            "ASM Tornado.asm loc_35948 maps colid_ceiling through UFO.asm loc_35146, negating y_vel",
        );
        assert_eq!(enemy.y_acceleration, 0);
        assert_eq!(enemy.target_y_velocity, 0);
    }

    #[test]
    fn tornado_ceiling_collision_requires_upward_velocity_loc_35948() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 40 << 16, 34 << 16, 1);
        enemy.y_velocity = 0;
        enemy.target_y_velocity = 0;
        enemy.y_acceleration = 0x0000_0800;
        enemy.wait_timer = 4;
        let grid = bounce_grid(&[(2, 1)]);

        step_tornado(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.y_velocity, 0,
            "ASM Tornado.asm loc_35948 should not ceiling-bounce without upward velocity",
        );
        assert_eq!(
            enemy.y_acceleration, 0x0000_0800,
            "ASM UFO.asm loc_35146 is skipped for the zero-y-velocity ceiling boundary",
        );
    }

    #[test]
    fn tornado_slopeup_collision_uses_ufo_slope_bounce_loc_35948() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 72 << 16, 53 << 16, 1);
        enemy.x_velocity = 0;
        enemy.y_velocity = 0x0002_0000;
        enemy.target_x_velocity = 0x0003_0000;
        enemy.target_y_velocity = 0x0003_0000;
        enemy.x_acceleration = 0;
        enemy.y_acceleration = 0;
        enemy.wait_timer = 4;
        let grid = grid_with_cells(8, 8, &[(4, 3, 0x40, 0)]);

        step_tornado(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, -0x0001_0000,
            "ASM UFO.asm Enemy0F_UFO_BounceUpSlope sets x_vel to -$10000",
        );
        assert_eq!(
            enemy.y_velocity, -0x0001_0000,
            "ASM UFO.asm Enemy0F_UFO_BounceUpSlope sets y_vel to -$10000",
        );
        assert_eq!(enemy.x_acceleration, 0x800);
        assert_eq!(enemy.y_acceleration, 0x800);
        assert_eq!(enemy.target_x_velocity, 0);
        assert_eq!(enemy.target_y_velocity, 0);
    }

    #[test]
    fn tornado_slopedown_collision_uses_ufo_slope_bounce_loc_35948() {
        let mut enemy = test_enemy(EnemyFamily::Tornado, 72 << 16, 53 << 16, 1);
        enemy.x_velocity = 0;
        enemy.y_velocity = 0x0002_0000;
        enemy.target_x_velocity = -0x0003_0000;
        enemy.target_y_velocity = 0x0003_0000;
        enemy.x_acceleration = 0;
        enemy.y_acceleration = 0;
        enemy.wait_timer = 4;
        let grid = grid_with_cells(8, 8, &[(4, 3, 0x50, 0)]);

        step_tornado(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, 0x0001_0000,
            "ASM UFO.asm Enemy0F_UFO_BounceDownSlope sets x_vel to $10000",
        );
        assert_eq!(
            enemy.y_velocity, -0x0001_0000,
            "ASM UFO.asm Enemy0F_UFO_BounceDownSlope sets y_vel to -$10000",
        );
        assert_eq!(enemy.x_acceleration, -0x800);
        assert_eq!(enemy.y_acceleration, 0x800);
        assert_eq!(enemy.target_x_velocity, 0);
        assert_eq!(enemy.target_y_velocity, 0);
    }

    #[test]
    fn tornado_collision_dispatch_loc_35948_uses_tornado_table() {
        use enemies::{
            TornadoCollisionBehaviorLoc35948, TornadoCollisionDispatchLoc35948,
            TornadoCollisionRouteLoc35056, tornado_collision_dispatch_loc_35948,
        };

        assert_eq!(
            tornado_collision_dispatch_loc_35948(-1),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                route: TornadoCollisionRouteLoc35056::NegativeClear,
                behavior: None,
            },
            "ASM UFO.asm loc_35056 clears negative collision_type before returning",
        );

        assert_eq!(
            tornado_collision_dispatch_loc_35948(0),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                route: TornadoCollisionRouteLoc35056::ZeroReturn,
                behavior: None,
            },
            "ASM Tornado.asm loc_35948 reaches shared `loc_35056`, which returns directly when collision_type is zero",
        );
        assert_eq!(
            tornado_collision_dispatch_loc_35948(0x0004),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                route: TornadoCollisionRouteLoc35056::TableDispatch,
                behavior: Some(TornadoCollisionBehaviorLoc35948::BounceWall),
            },
            "ASM Tornado.asm loc_35948 table-selects wall bounce for collision_type=$04",
        );
        assert_eq!(
            tornado_collision_dispatch_loc_35948(0x001C),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                route: TornadoCollisionRouteLoc35056::TableDispatch,
                behavior: Some(TornadoCollisionBehaviorLoc35948::Hurt),
            },
            "ASM Tornado.asm loc_35948 table-selects the hurt/deletion path for collision_type=$1C",
        );
        assert_eq!(
            tornado_collision_dispatch_loc_35948(0x0020),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                route: TornadoCollisionRouteLoc35056::TableDispatch,
                behavior: Some(TornadoCollisionBehaviorLoc35948::CycloneBounceKidRight),
            },
            "ASM Tornado.asm loc_35948 routes collision_type=$20 through the Tornado-specific side-contact table entry",
        );
        assert_eq!(
            tornado_collision_dispatch_loc_35948(0x0024),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                route: TornadoCollisionRouteLoc35056::TableDispatch,
                behavior: Some(TornadoCollisionBehaviorLoc35948::CycloneBounceKidLeft),
            },
            "ASM Tornado.asm loc_35948 routes collision_type=$24 through the opposite side-contact table entry",
        );
        assert_eq!(
            tornado_collision_dispatch_loc_35948(0x0028),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                route: TornadoCollisionRouteLoc35056::TableDispatch,
                behavior: Some(TornadoCollisionBehaviorLoc35948::ReturnOnly),
            },
            "ASM Tornado.asm off_35950 keeps collision_type=$28 as an explicit return-only table slot",
        );
    }

    #[test]
    fn tornado_hit_sequence_loc_359b2_routes_status_slot() {
        use enemies::{
            TornadoDeleteTailLoc359EE, TornadoHitSequenceLoc359B2, TornadoHitStatusRouteLoc359B2,
            TornadoRespawnMarkerLoc359E0, tornado_hit_sequence_loc_359b2,
        };

        assert_eq!(
            tornado_hit_sequence_loc_359b2(0x05),
            TornadoHitSequenceLoc359B2 {
                waits_for_hit_animation: true,
                status_route: TornadoHitStatusRouteLoc359B2::WriteRespawnMarker(
                    TornadoRespawnMarkerLoc359E0 {
                        index: 5,
                        word_value: 0x2168,
                    },
                ),
                delete_tail: TornadoDeleteTailLoc359EE {
                    deletes_current_object: true,
                },
            },
            "ASM Tornado.asm loc_359B2 branches on non-negative `$42(a5)` after the hit animation",
        );
        assert_eq!(
            tornado_hit_sequence_loc_359b2(0xC7),
            TornadoHitSequenceLoc359B2 {
                waits_for_hit_animation: true,
                status_route: TornadoHitStatusRouteLoc359B2::SubtractSpawnCounter {
                    index: 7,
                    decrement: 0x0400,
                },
                delete_tail: TornadoDeleteTailLoc359EE {
                    deletes_current_object: true,
                },
            },
            "ASM Tornado.asm loc_359B2 masks bit 6 from negative `$42(a5)` before subtracting the spawn counter",
        );
        assert_eq!(
            tornado_hit_sequence_loc_359b2(0x87),
            TornadoHitSequenceLoc359B2 {
                waits_for_hit_animation: true,
                status_route: TornadoHitStatusRouteLoc359B2::DeleteOnly,
                delete_tail: TornadoDeleteTailLoc359EE {
                    deletes_current_object: true,
                },
            },
            "ASM Tornado.asm loc_359B2 skips the status table when negative `$42(a5)` has bit 6 clear",
        );
    }

    #[test]
    fn tornado_respawn_marker_loc_359e0_writes_status_word() {
        use enemies::{TornadoRespawnMarkerLoc359E0, tornado_respawn_marker_loc_359e0};

        assert_eq!(
            tornado_respawn_marker_loc_359e0(0x12),
            TornadoRespawnMarkerLoc359E0 {
                index: 0x12,
                word_value: 0x2168,
            },
            "ASM Tornado.asm loc_359E0 writes `$2168` to EnemyStatus_Table[$42(a5)] before deletion",
        );
    }

    #[test]
    fn tornado_delete_tail_loc_359ee_deletes_current_object() {
        use enemies::{TornadoDeleteTailLoc359EE, tornado_delete_tail_loc_359ee};

        assert_eq!(
            tornado_delete_tail_loc_359ee(),
            TornadoDeleteTailLoc359EE {
                deletes_current_object: true,
            },
            "ASM Tornado.asm loc_359EE jumps to j_Delete_CurrentObject after every status route",
        );
    }

    // --- Cyclone-Tornado bounce interaction (ASM Tornado.asm loc_35986) ---

    #[test]
    fn tornado_cyclone_bounce_kid_right_loc_3597e_swaps_positive_d7() {
        use enemies::{TornadoCycloneBounceLoc35986, tornado_cyclone_bounce_kid_right_loc_3597e};

        assert_eq!(
            tornado_cyclone_bounce_kid_right_loc_3597e(),
            TornadoCycloneBounceLoc35986 {
                player_x_velocity: 0x0002_0000,
                tornado_x_velocity: -0x0001_0000,
                player_collision_type: 0x0028,
            },
            "ASM Tornado.asm loc_3597E seeds d7=2 before loc_35986 swaps it into Kid x_vel and reflects half speed into the Tornado",
        );
    }

    #[test]
    fn tornado_cyclone_bounce_kid_left_loc_35984_preserves_moveq_sign_extension() {
        use enemies::{TornadoCycloneBounceLoc35986, tornado_cyclone_bounce_kid_left_loc_35984};

        assert_eq!(
            tornado_cyclone_bounce_kid_left_loc_35984(),
            TornadoCycloneBounceLoc35986 {
                player_x_velocity: -0x0001_0001,
                tornado_x_velocity: 0x0000_8000,
                player_collision_type: 0x0028,
            },
            "ASM Tornado.asm loc_35984 uses `moveq #-2,d7`; after loc_35986 `swap d7`, Kid x_vel is $FFFEFFFF rather than symmetric -$00020000",
        );
    }

    #[test]
    fn tornado_cyclone_bounce_for_positions_loc_35986_selects_side_branch() {
        use enemies::tornado_cyclone_bounce_for_positions_loc_35986;

        assert_eq!(
            tornado_cyclone_bounce_for_positions_loc_35986(0x0100_0000, 0x0110_0000)
                .player_x_velocity,
            -0x0001_0001,
            "ASM Tornado.asm loc_35986 should use the loc_35984 left-contact branch when Kid is left of the Tornado",
        );
        assert_eq!(
            tornado_cyclone_bounce_for_positions_loc_35986(0x0120_0000, 0x0110_0000)
                .player_x_velocity,
            0x0002_0000,
            "ASM Tornado.asm loc_35986 should use the loc_3597E right-contact branch when Kid is right of the Tornado",
        );
        assert_eq!(
            tornado_cyclone_bounce_for_positions_loc_35986(0x0110_0000, 0x0110_0000)
                .player_x_velocity,
            0x0002_0000,
            "Native side-branch derivation keeps the equal-X boundary with the right-contact loc_3597E path",
        );
    }

    /// Integration test: regular form takes damage from Tornado.
    #[test]
    fn regular_form_takes_damage_from_tornado() {
        let mut rt = NativeRuntime::new(manifest_with_static_tornado());
        rt.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut rt);
        rt.progression.invulnerable_frames = 0;
        rt.progression.current_form = PlayerForm::TheKid; // Not Cyclone flying
        let initial_hp = rt.progression.number_hitpoints;
        // Position player and camera near the Tornado
        if let Some(ref mut slice) = rt.opening_slice.as_mut() {
            slice.player.x_pos = ((0x0750 - 5) as u32) << 16;
            slice.player.y_pos = 0x00E0_u32 << 16;
            slice.player.on_ground = true;
            slice.camera.x_pos = 0x0750_u32 << 16;
            slice.camera.y_pos = 0x00E0_u32 << 16;
        }
        // Step frames to allow Tornado activation and potential contact
        for _ in 0..20 {
            rt.step_frame(FrameInput::default());
        }
        // Player should have taken damage (lower HP) since not Cyclone-flying
        assert!(
            rt.progression.number_hitpoints < initial_hp,
            "Player should take damage from Tornado when not Cyclone-flying"
        );
    }

    // --- Ninja ---

    #[test]
    fn ninja_paces_in_stealth_without_player() {
        let grid = grid_with_platform(17, 0, 120);
        let mut enemy = test_enemy(EnemyFamily::Ninja, 60 << 16, (17 * 16) << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        enemy.on_ground = true;
        let start_x = enemy.x_pos;
        step_ninja(
            &mut enemy,
            Some(&grid),
            None,
            None,
            PlayerForm::TheKid,
            None,
        );
        assert_eq!(enemy.ninja_state, NinjaState::Stealth);
        assert_ne!(enemy.x_pos, start_x, "should move while pacing");
    }

    #[test]
    fn ninja_detects_player_in_cone_and_enters_jump_threat_loc_3d706_3d84e() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        // Player 0x30 px above (within 0x20-0x50 range) and 0x08 px right (within ±0x10)
        let player_x = Some((100 + 0x08) << 16);
        let player_y = Some((200 - 0x30) << 16);
        step_ninja(
            &mut enemy,
            None,
            player_x,
            player_y,
            PlayerForm::TheKid,
            None,
        );
        assert_eq!(enemy.ninja_state, NinjaState::JumpThreat);
        assert_eq!(enemy.shared_walker_state, 1);
        assert_eq!(enemy.facing, Direction::Right);
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);
    }

    #[test]
    fn ninja_far_kid_turns_right_during_pacing_loc_3d92a_3d978() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        enemy.facing = Direction::Left;
        let player_x = Some((100 + 0x81) << 16);
        let player_y = Some(200 << 16);

        step_ninja(
            &mut enemy,
            None,
            player_x,
            player_y,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Ninja.asm loc_3D92A -> loc_3D978 turns toward Kid when he is more than $80 px to the right"
        );
    }

    #[test]
    fn ninja_far_kid_turns_left_during_pacing_loc_3d92a_3d98a() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 200 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        enemy.facing = Direction::Right;
        let player_x = Some((200 - 0x81) << 16);
        let player_y = Some(200 << 16);

        step_ninja(
            &mut enemy,
            None,
            player_x,
            player_y,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM Ninja.asm loc_3D92A -> loc_3D98A turns toward Kid when he is more than $80 px to the left"
        );
    }

    #[test]
    fn ninja_cadence_timer_blocks_jump_until_zero_loc_3d942() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        enemy.facing = Direction::Right;
        enemy.shared_walker_patrol_timer = 2;
        enemy.wait_timer = 37;
        let player_x = Some(100 << 16);
        let player_y = Some(200 << 16);

        step_ninja(
            &mut enemy,
            None,
            player_x,
            player_y,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.ninja_state,
            NinjaState::Stealth,
            "ASM Ninja.asm loc_3D942 should continue pacing while `$54(a5)` remains non-zero after decrement"
        );
        assert_eq!(enemy.shared_walker_patrol_timer, 1);
    }

    #[test]
    fn ninja_cadence_wrong_walk_phase_resets_timer_one_loc_3d958() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        enemy.facing = Direction::Right;
        enemy.shared_walker_patrol_timer = 1;
        enemy.wait_timer = 36;
        let player_x = Some(100 << 16);
        let player_y = Some(200 << 16);

        step_ninja(
            &mut enemy,
            None,
            player_x,
            player_y,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.ninja_state,
            NinjaState::Stealth,
            "ASM Ninja.asm loc_3D958 should require the exact C7DBE frame with animation_timer 3 before rotating `$6E(a5)`"
        );
        assert_eq!(enemy.shared_walker_patrol_timer, 1);
    }

    #[test]
    fn ninja_cadence_throw_frame_starts_jump_loc_3d960_3d9a4() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        enemy.facing = Direction::Right;
        enemy.shared_walker_patrol_timer = 1;
        enemy.shared_walker_patrol_short_next = true;
        enemy.wait_timer = 37;
        let player_x = Some(100 << 16);
        let player_y = Some(200 << 16);

        step_ninja(
            &mut enemy,
            None,
            player_x,
            player_y,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.ninja_state,
            NinjaState::JumpAttack,
            "ASM Ninja.asm loc_3D960 should fall through to loc_3D9A4 when `$4C(a5)` is zero on the throw-frame gate"
        );
        assert_eq!(enemy.shared_walker_patrol_timer, 0x0032);
        assert!(!enemy.shared_walker_patrol_short_next);
        assert_eq!(enemy.x_velocity, 0x0001_0000);
        assert_eq!(enemy.y_velocity, -0x0004_8000);
        assert!(!enemy.on_ground);
    }

    #[test]
    fn ninja_red_stealth_still_enters_overhead_jump_threat_loc_3d706() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        let player_x = Some(100 << 16);
        let player_y = Some((200 - 0x30) << 16);
        step_ninja(
            &mut enemy,
            None,
            player_x,
            player_y,
            PlayerForm::RedStealth,
            None,
        );
        assert_eq!(
            enemy.ninja_state,
            NinjaState::JumpThreat,
            "ASM Ninja.asm loc_3D706 checks the overhead jump-threat window before the later Red Stealth guard-pose branch"
        );
    }

    #[test]
    fn ninja_overhead_right_inside_edge_enters_positive_jump_threat_loc_3d706_3d84e() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0x0001_0000;
        let player_x = Some((100 + 0x0F) << 16);
        let player_y = Some((200 - 0x30) << 16);

        step_ninja(
            &mut enemy,
            None,
            player_x,
            player_y,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.ninja_state,
            NinjaState::JumpThreat,
            "ASM Ninja.asm loc_3D706 branches to loc_3D84E at the half-open +$0F jump-threat edge"
        );
        assert_eq!(enemy.shared_walker_state, 1);
        assert_eq!(enemy.walk_speed_override, 0);
        assert_eq!(enemy.facing, Direction::Right);
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);
    }

    #[test]
    fn ninja_red_stealth_side_threat_enters_guard_pose_loc_3d734_3d826() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0x0001_0000;
        let player_x = Some((100 + 0x30) << 16);
        let player_y = Some(200 << 16);

        step_ninja(
            &mut enemy,
            None,
            player_x,
            player_y,
            PlayerForm::RedStealth,
            None,
        );

        assert_eq!(
            enemy.ninja_state,
            NinjaState::Guard,
            "ASM Ninja.asm loc_3D734 should route Red Stealth in the close side band to loc_3D826 instead of pacing"
        );
        assert_eq!(enemy.shared_walker_state, -1);
        assert_eq!(enemy.walk_speed_override, 0);
        assert_eq!(enemy.facing, Direction::Right);
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);
    }

    #[test]
    fn ninja_maniaxe_projectile_threat_enters_guard_pose_loc_3d776_3d826() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0x0001_0000;
        let mut active = vec![enemy];
        let mut projectiles = vec![EnemyProjectile {
            x_pos: (100 + 0x59) << 16,
            kind: EnemyProjectileKind::ManiaxeAxe,
            ..EnemyProjectile::default()
        }];
        let mut drilled_blocks = Vec::new();
        let player_x = Some((100 + 0x70) << 16);
        let player_y = Some(200 << 16);

        step_active_enemies(
            &mut active,
            None,
            player_x,
            player_y,
            None,
            None,
            PlayerForm::Maniaxe,
            Some(&mut projectiles),
            &mut drilled_blocks,
        );

        assert_eq!(
            active[0].ninja_state,
            NinjaState::Guard,
            "ASM Ninja.asm loc_3D776 should route a live Maniaxe projectile inside the half-open +$5A band to loc_3D826"
        );
        assert_eq!(active[0].shared_walker_state, -1);
        assert_eq!(active[0].walk_speed_override, 0);
        assert_eq!(active[0].facing, Direction::Right);
        assert_eq!(active[0].x_velocity, 0);
        assert_eq!(active[0].y_velocity, 0);
    }

    #[test]
    fn ninja_positive_jump_threat_releases_when_kid_leaves_box_loc_3d7fa_3d7ca() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::JumpThreat;
        enemy.shared_walker_state = 1;
        enemy.facing = Direction::Right;
        enemy.walk_speed_override = 0;
        let player_x = Some((100 + 0x11) << 16);
        let player_y = Some((200 - 0x30) << 16);

        step_ninja(
            &mut enemy,
            None,
            player_x,
            player_y,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.ninja_state,
            NinjaState::Stealth,
            "ASM Ninja.asm loc_3D7FA should reset positive `$4C(a5)` via loc_3D7CA once Kid leaves the inclusive +/-$10 X band"
        );
        assert_eq!(enemy.shared_walker_state, 0);
        assert_eq!(
            enemy.walk_speed_override,
            pacer_walk_speed(EnemyFamily::Ninja, enemy.hit_points)
        );
    }

    #[test]
    fn ninja_red_stealth_guard_releases_when_kid_leaves_band_loc_3d79c_3d7ca() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Guard;
        enemy.shared_walker_state = -1;
        enemy.facing = Direction::Right;
        enemy.walk_speed_override = 0;
        let player_x = Some((100 + 0x31) << 16);
        let player_y = Some(200 << 16);

        step_ninja(
            &mut enemy,
            None,
            player_x,
            player_y,
            PlayerForm::RedStealth,
            None,
        );

        assert_eq!(
            enemy.ninja_state,
            NinjaState::Stealth,
            "ASM Ninja.asm loc_3D79C should reset the negative guard state via loc_3D7CA once Red Stealth leaves the inclusive +/-$30 band"
        );
        assert_eq!(enemy.shared_walker_state, 0);
        assert_eq!(
            enemy.walk_speed_override,
            pacer_walk_speed(EnemyFamily::Ninja, enemy.hit_points)
        );
    }

    #[test]
    fn ninja_jump_attack_ceiling_contact_negates_y_velocity_loc_3da0c_3dab0() {
        let width_cells = 20u16;
        let height_cells = 8u16;
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        for col in 0..width_cells {
            let offset = (usize::from(2u16) * usize::from(width_cells) + usize::from(col)) * 2;
            data[offset] = 0x60;
        }
        let grid = terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells);
        let mut enemy = test_enemy(EnemyFamily::Ninja, 80 << 16, 63 << 16, 2);
        enemy.ninja_state = NinjaState::JumpAttack;
        enemy.x_velocity = 0;
        enemy.y_velocity = -0x0002_0000;
        enemy.on_ground = false;

        step_ninja(
            &mut enemy,
            Some(&grid),
            None,
            None,
            PlayerForm::TheKid,
            None,
        );

        assert_eq!(
            enemy.ninja_state,
            NinjaState::JumpAttack,
            "ASM Ninja.asm loc_3DAB0 returns to the loc_3D9DA airborne loop after ceiling contact"
        );
        assert_eq!(
            enemy.y_velocity, 0x0001_D800,
            "ASM Ninja.asm loc_3D9DA adds $2800, then loc_3DA0C dispatches colid_ceiling to loc_3DAB0, which negates y_vel"
        );
    }

    #[test]
    fn ninja_lands_and_returns_to_stealth() {
        // Floor at row 17 spanning full width — ninja will land on it
        let grid = grid_with_platform(17, 0, 120);
        let mut enemy = test_enemy(EnemyFamily::Ninja, 50 << 16, (17 * 16 - 40) << 16, 2);
        enemy.ninja_state = NinjaState::JumpAttack;
        enemy.x_velocity = NINJA_JUMP_X_SPEED_RIGHT;
        enemy.y_velocity = NINJA_JUMP_Y_SPEED;
        enemy.on_ground = false;
        // Step until ninja lands (gravity pulls it down)
        for _ in 0..300 {
            step_ninja(
                &mut enemy,
                Some(&grid),
                None,
                None,
                PlayerForm::TheKid,
                None,
            );
            if enemy.ninja_state == NinjaState::Stealth {
                break;
            }
        }
        assert_eq!(
            enemy.ninja_state,
            NinjaState::Stealth,
            "ninja should return to stealth after landing"
        );
        assert!(enemy.on_ground);
    }

    #[test]
    fn ninja_jump_attack_live_gravity_is_uncapped_loc_3d9da() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 50 << 16, 80 << 16, 2);
        enemy.ninja_state = NinjaState::JumpAttack;
        enemy.y_velocity = physics::MAX_FALL_SPEED;
        enemy.on_ground = false;

        step_ninja(&mut enemy, None, None, None, PlayerForm::TheKid, None);

        assert_eq!(
            enemy.y_velocity,
            physics::MAX_FALL_SPEED + NINJA_GRAVITY,
            "ASM Ninja.asm loc_3D9DA adds `$2800` to y_vel without the shared native max-fall clamp"
        );
    }

    #[test]
    fn ninja_kid_above_knockdown_starts_death_after_floor_loc_3dabc_3db4e() {
        let grid = grid_with_platform(17, 0, 120);
        let mut enemy = test_enemy(EnemyFamily::Ninja, 50 << 16, (17 * 16 - 1) << 16, 0);
        enemy.ninja_state = NinjaState::KidAboveKnockdown;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0x0001_0000;
        enemy.on_ground = false;

        step_ninja(
            &mut enemy,
            Some(&grid),
            None,
            None,
            PlayerForm::TheKid,
            None,
        );

        assert!(
            enemy.death_animation_active,
            "ASM Ninja.asm loc_3DABC should start loc_3DB4E only after floor contact"
        );
        assert_eq!(enemy.wait_timer, NINJA_DEATH_ANIMATION_FRAMES);
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);
        assert!(enemy.on_ground);
    }

    // --- Hand ---

    #[test]
    fn hand_patrol_loc_32804_continues_positive_velocity_when_d7_nonzero() {
        assert_eq!(
            hand_patrol_decision_loc_32804(0x0000_3000, 0, 1),
            HandPatrolDecision::ContinuePatrol,
            "ASM Hand.asm loc_32804: after positive x_vel, `tst.w d7; bne.s loc_32804` loops without reversing or jumping"
        );
    }

    #[test]
    fn hand_patrol_loc_32804_zero_velocity_uses_nonnegative_branch() {
        assert_eq!(
            hand_patrol_decision_loc_32804(0, 1, 0),
            HandPatrolDecision::ReverseDirection,
            "ASM Hand.asm loc_32804 tests `bmi.w loc_32836`, so zero x_vel follows the nonnegative d7/d6 branch"
        );
    }

    #[test]
    fn hand_patrol_loc_32832_reverses_positive_velocity_when_d6_nonzero() {
        assert_eq!(
            hand_patrol_decision_loc_32804(0x0000_3000, 1, 0),
            HandPatrolDecision::ReverseDirection,
            "ASM Hand.asm loc_32832: positive x_vel with d7 clear and d6 non-zero branches to the shared reversal path"
        );
    }

    #[test]
    fn hand_patrol_loc_32836_reverses_negative_velocity_when_d7_nonzero() {
        assert_eq!(
            hand_patrol_decision_loc_32804(-0x0000_3000, 0, 1),
            HandPatrolDecision::ReverseDirection,
            "ASM Hand.asm loc_32836: negative x_vel with d6 clear and d7 non-zero branches to the shared reversal path"
        );
    }

    #[test]
    fn hand_reversal_loc_32840_negates_velocity_and_inverts_direction_flag() {
        assert_eq!(
            hand_reversal_loc_32840(-0x0000_4800, false),
            HandReversalLoc32840 {
                x_velocity: 0x0000_4800,
                x_direction_flag: true,
            },
            "ASM Hand.asm loc_32840: `neg.l d5; move.l d5,x_vel(a3); not.b x_direction(a3)` reverses both patrol velocity and raw direction flag"
        );
        assert_eq!(
            hand_reversal_loc_32840(0x0000_3000, true).x_velocity,
            -0x0000_3000,
            "ASM Hand.asm loc_32840 uses long negation symmetrically when turning back left"
        );
    }

    #[test]
    fn hand_left_detection_loc_32886_accepts_only_forward_cone() {
        assert!(
            hand_left_detection_loc_32886(-0x30, -0x18, 0x40),
            "ASM Hand.asm loc_32886 accepts a left-side Kid within range when the vertical compare stays inside the 45-degree cone"
        );
        assert!(
            !hand_left_detection_loc_32886(1, -0x18, 0x40),
            "ASM Hand.asm loc_32886: `tst.w d6; bpl` rejects a Kid behind the clear x_direction branch"
        );
        assert!(
            !hand_left_detection_loc_32886(-0x41, -0x18, 0x40),
            "ASM Hand.asm loc_32886 rejects when d6 is below the negated `$44(a5)` range"
        );
        assert!(
            hand_left_detection_loc_32886(-0x40, -0x18, 0x40),
            "ASM Hand.asm loc_32886 accepts the exact negative range boundary because `blt` rejects only d6 < -range"
        );
        assert!(
            !hand_left_detection_loc_32886(-0x20, -0x28, 0x40),
            "ASM Hand.asm loc_32886 rejects when `cmp.w d6,d7; blt` finds the vertical lead larger than horizontal lead"
        );
    }

    #[test]
    fn hand_detection_reject_loc_328a2_returns_zero_d7() {
        assert_eq!(
            hand_detection_reject_loc_328a2(),
            0,
            "ASM Hand.asm loc_328A2: `moveq #0,d7; rts` is the shared no-attack probe return"
        );
    }

    #[test]
    fn hand_jump_seed_loc_328a6_loads_hp_tier_attack_seed() {
        assert_eq!(
            hand_jump_seed_loc_328a6(1),
            1,
            "ASM Hand.asm loc_328A6 loads `$46(a5)` as the jump speed seed for HP=1"
        );
        assert_eq!(
            hand_jump_seed_loc_328a6(2),
            2,
            "ASM Hand.asm loc_328A6 preserves the HP=2 table seed"
        );
        assert_eq!(
            hand_jump_seed_loc_328a6(3),
            3,
            "ASM Hand.asm loc_328A6 preserves the HP=3 table seed"
        );
        assert_eq!(
            hand_jump_seed_loc_328a6(8),
            3,
            "native clamps oversized HP to the final Hand.asm attack tier"
        );
    }

    #[test]
    fn hand_jump_components_loc_328aa_mix_random_bytes_with_kid_position() {
        let components = hand_jump_components_loc_328aa(2, false, 0xAA, 0x1234, 0x0F, 0x0010);
        assert_eq!(
            components,
            HandJumpComponentsLoc328AA {
                y_velocity: -0x0004_0000,
                x_velocity_word: -5,
            },
            "ASM Hand.asm loc_328AA: Y uses `(random ^ KidYLo) & 3`, then `neg.w; swap`; clear x_direction negates the X word before the X store"
        );

        assert_eq!(
            hand_jump_components_loc_328aa(2, true, 0xAA, 0x1234, 0x0F, 0x0010).x_velocity_word,
            5,
            "ASM Hand.asm loc_328AA skips the X negation when raw x_direction is non-zero"
        );
    }

    #[test]
    fn hand_x_velocity_loc_328e0_swaps_signed_word_into_fixed_velocity() {
        assert_eq!(
            hand_x_velocity_loc_328e0(5),
            0x0005_0000,
            "ASM Hand.asm loc_328E0: `swap d7; move.l d7,x_vel(a3)` stores a positive X word as 16.16 velocity"
        );
        assert_eq!(
            hand_x_velocity_loc_328e0(-5),
            -0x0005_0000,
            "ASM Hand.asm loc_328E0 preserves the sign bit when the clear x_direction path prepared a negative X word"
        );
    }

    #[test]
    fn hand_jump_loop_loc_328f0_adds_uncapped_gravity() {
        assert_eq!(
            hand_jump_loop_gravity_loc_328f0(0x0018_0000),
            0x0018_4000,
            "ASM Hand.asm loc_328F0 applies `addi.l #$4000,y_vel(a3)` each loop and does not clamp at $180000"
        );
    }

    #[test]
    fn hand_ground_collision_loc_3299c_dispatches_ground_table() {
        assert_eq!(
            hand_ground_collision_dispatch_loc_3299c(-1),
            HandGroundCollisionDispatchLoc3299C {
                collision_type: 0xFFFF,
                behavior: Some(HandGroundCollisionBehaviorLoc3299C::Destroy),
            },
            "ASM Hand.asm loc_3299C branches negative collision_type to the destroy path before clearing it"
        );
        assert_eq!(
            hand_ground_collision_dispatch_loc_3299c(0),
            HandGroundCollisionDispatchLoc3299C {
                collision_type: 0,
                behavior: None,
            },
            "ASM Hand.asm loc_3299C returns directly when collision_type is zero"
        );
        assert_eq!(
            hand_ground_collision_dispatch_loc_3299c(0x0004),
            HandGroundCollisionDispatchLoc3299C {
                collision_type: 0,
                behavior: Some(HandGroundCollisionBehaviorLoc3299C::Reverse),
            },
            "ASM Hand.asm loc_3299C clears positive collision_type before routing right-wall contact through the table"
        );
        assert_eq!(
            hand_ground_collision_dispatch_loc_3299c(0x0028).behavior,
            Some(HandGroundCollisionBehaviorLoc3299C::KidBelowContact),
            "ASM Hand.asm loc_3299C maps the $28 contact slot to the Kid-below contact route"
        );
        assert_eq!(
            hand_ground_collision_dispatch_loc_3299c(0x002C).behavior,
            Some(HandGroundCollisionBehaviorLoc3299C::Destroy),
            "ASM Hand.asm loc_3299C maps the final table slot to the shared destroy path"
        );
    }

    #[test]
    fn hand_jump_collision_loc_329f8_dispatches_airborne_table() {
        assert_eq!(
            hand_jump_collision_dispatch_loc_329f8(-1),
            HandJumpCollisionDispatchLoc329F8 {
                collision_type: 0xFFFF,
                behavior: Some(HandJumpCollisionBehaviorLoc329F8::Destroy),
            },
            "ASM Hand.asm loc_329F8 branches negative collision_type to the shared destroy path before clearing it"
        );
        assert_eq!(
            hand_jump_collision_dispatch_loc_329f8(0),
            HandJumpCollisionDispatchLoc329F8 {
                collision_type: 0,
                behavior: None,
            },
            "ASM Hand.asm loc_329F8 returns directly when collision_type is zero"
        );
        assert_eq!(
            hand_jump_collision_dispatch_loc_329f8(0x0004),
            HandJumpCollisionDispatchLoc329F8 {
                collision_type: 0,
                behavior: Some(HandJumpCollisionBehaviorLoc329F8::ClearXVelocity),
            },
            "ASM Hand.asm loc_329F8 clears positive collision_type before routing the first airborne contact slot"
        );
        assert_eq!(
            hand_jump_collision_dispatch_loc_329f8(0x0010).behavior,
            Some(HandJumpCollisionBehaviorLoc329F8::ClearYVelocity),
            "ASM Hand.asm loc_329F8 maps the $10 airborne contact slot to vertical velocity clearing"
        );
        assert_eq!(
            hand_jump_collision_dispatch_loc_329f8(0x000C).behavior,
            Some(HandJumpCollisionBehaviorLoc329F8::ResetToPatrol),
            "ASM Hand.asm off_32A12 maps the floor slot `$0C` to loc_32A4A, which clears velocity and resumes patrol setup"
        );
        assert_eq!(
            hand_jump_collision_dispatch_loc_329f8(0x0018).behavior,
            Some(HandJumpCollisionBehaviorLoc329F8::BounceUpRight),
            "ASM Hand.asm off_32A12 maps the down-slope slot `$18` to loc_32A70's up-right bounce"
        );
        assert_eq!(
            hand_jump_collision_dispatch_loc_329f8(0x0028).behavior,
            Some(HandJumpCollisionBehaviorLoc329F8::TryGrabKid),
            "ASM Hand.asm loc_329F8 maps Kid-contact airborne slots to the grab gate"
        );
        assert_eq!(
            hand_jump_collision_dispatch_loc_329f8(0x002C).behavior,
            Some(HandJumpCollisionBehaviorLoc329F8::ReturnOnly),
            "ASM Hand.asm loc_329F8 maps the final airborne table slot to an immediate return"
        );
    }

    #[test]
    fn hand_kid_contact_gate_loc_32a86_requires_grabbable_kid_state() {
        assert_eq!(
            hand_kid_contact_gate_loc_32a86(true, false, false),
            HandKidContactRouteLoc32A86::ReturnOnly,
            "ASM Hand.asm loc_32A86 returns immediately when Character_Movement is MoveID_Crawling"
        );
        assert_eq!(
            hand_kid_contact_gate_loc_32a86(false, true, false),
            HandKidContactRouteLoc32A86::ReturnOnly,
            "ASM Hand.asm loc_32A86 returns immediately when Current_Helmet is Juggernaut"
        );
        assert_eq!(
            hand_kid_contact_gate_loc_32a86(false, false, true),
            HandKidContactRouteLoc32A86::ReturnOnly,
            "ASM Hand.asm loc_32A86 returns when KidGrabbedByHand is already set"
        );
        assert_eq!(
            hand_kid_contact_gate_loc_32a86(false, false, false),
            HandKidContactRouteLoc32A86::StartGrab,
            "ASM Hand.asm loc_32A86 branches to grab setup only after all three grab guards are clear"
        );
    }

    #[test]
    fn hand_death_sequence_loc_32aa2_runs_animation_then_status_route() {
        let positive = hand_death_sequence_loc_32aa2(0x05);
        assert_eq!(
            positive,
            HandDeathSequenceLoc32AA2 {
                has_kid_collision: true,
                is_moved: false,
                animation_label: "stru_32B78",
                hibernate_until_animation_finished: true,
                status_route: HandDeathStatusRouteLoc32AA2::WriteRespawnMarker(
                    HandRespawnMarkerLoc32AD8 {
                        index: 0x05,
                        word_value: 0x2168,
                    }
                ),
                deletes_current_object: true,
            },
            "ASM Hand.asm loc_32AA2 sets has_kid_collision, clears is_moved, waits for stru_32B78, then routes positive `$42(a5)` to the respawn-marker path"
        );

        assert_eq!(
            hand_death_sequence_loc_32aa2(0xC7).status_route,
            HandDeathStatusRouteLoc32AA2::SubtractSpawnCounter {
                index: 0x07,
                decrement: 0x0400,
            },
            "ASM Hand.asm loc_32AA2 masks bit-7+bit-6 `$42(a5)` with `$3F` before subtracting `$400` from EnemyStatus_Table"
        );

        let delete_only = hand_death_sequence_loc_32aa2(0x80);
        assert_eq!(
            delete_only.status_route,
            HandDeathStatusRouteLoc32AA2::DeleteOnly,
            "ASM Hand.asm loc_32AA2 sends bit-7 set and bit-6 clear status bytes straight to delete"
        );
        assert!(delete_only.deletes_current_object);
    }

    #[test]
    fn hand_respawn_marker_loc_32ad8_writes_positive_status_slot() {
        assert_eq!(
            hand_respawn_marker_loc_32ad8(0x05),
            HandRespawnMarkerLoc32AD8 {
                index: 0x05,
                word_value: 0x2168,
            },
            "ASM Hand.asm loc_32AD8 doubles positive `$42(a5)` into an EnemyStatus_Table word offset and writes `$2168`"
        );
    }

    #[test]
    fn hand_terrain_scan_loop_loc_32b32_accumulates_middle_tile_hits() {
        let scan = hand_terrain_scan_loc_32afe(0x0048, 0x004F, &[0x0000, 0x4000, 0x0000]);
        assert_eq!(
            scan.sample_count, 3,
            "ASM Hand.asm loc_32B32 should scan all columns from `(x-$E)>>4` through `(x+$E)>>4`"
        );
        assert!(
            scan.any_collision_4000,
            "ASM Hand.asm loc_32B32 sets d4 when any loop body sample carries bit `$4000`, even when the first and last samples are clear"
        );
        assert_eq!(
            (scan.first_collision_4000, scan.last_collision_4000),
            (0, 0),
            "ASM Hand.asm loc_32B32 must not collapse the middle hit into either edge mask"
        );
    }

    #[test]
    fn hand_terrain_scan_loop_loc_32b3e_uses_inclusive_dbf_bound() {
        let scan = hand_terrain_scan_loc_32afe(0x0048, 0x004F, &[0x0000, 0x0000, 0x4000, 0x0000]);
        assert_eq!(
            scan.sample_count, 3,
            "ASM Hand.asm loc_32B3E starts with `d7=end_col-start_col`, so `dbf` runs three iterations for a span of two"
        );
        assert_eq!(
            scan.last_collision_4000, 0x4000,
            "ASM Hand.asm loc_32B3E exits with `-2(a4)` pointing at the third and final scanned word"
        );
        assert!(
            scan.any_collision_4000,
            "ASM Hand.asm loc_32B3E must include the final iteration in the returned `tst.w d4` state"
        );
    }

    #[test]
    fn hand_jump_collision_loc_32a3e_clears_only_y_velocity() {
        assert_eq!(
            hand_jump_collision_clear_y_velocity_loc_32a3e(0x0002_0000, -0x0003_4000),
            HandJumpCollisionVelocityLoc32A3E {
                x_velocity: 0x0002_0000,
                y_velocity: 0,
            },
            "ASM Hand.asm loc_32A3E executes only `clr.l y_vel(a3)`, preserving x_vel"
        );
    }

    #[test]
    fn hand_jump_collision_loc_32a44_clears_only_x_velocity() {
        assert_eq!(
            hand_jump_collision_clear_x_velocity_loc_32a44(0x0002_0000, -0x0003_4000),
            HandJumpCollisionVelocityLoc32A44 {
                x_velocity: 0,
                y_velocity: -0x0003_4000,
            },
            "ASM Hand.asm loc_32A44 executes only `clr.l x_vel(a3)`, preserving y_vel"
        );
    }

    #[test]
    fn hand_jump_collision_loc_32a4a_resets_to_initial_patrol_motion() {
        assert_eq!(
            hand_jump_collision_reset_loc_32a4a(2, false),
            HandJumpCollisionResetLoc32A4A {
                x_velocity: 0,
                y_velocity: 0,
                initial_motion: HandInitialMotionLoc327F2 {
                    x_velocity: -0x0000_4800,
                    animation_label: "stru_32B6A",
                    has_kid_collision: false,
                },
            },
            "ASM Hand.asm loc_32A4A clears x/y velocity, unwinds the collision caller, and resumes patrol setup"
        );
    }

    #[test]
    fn hand_jump_collision_loc_32a5a_rolls_back_and_bounces_up_left() {
        assert_eq!(
            hand_jump_collision_bounce_up_left_loc_32a5a(
                0x1000_0000,
                0x2000_0000,
                0x0000_8000,
                -0x0000_4000,
            ),
            HandJumpCollisionBounceLoc32A5A {
                x_pos: 0x0FFF_8000,
                y_pos: 0x2000_4000,
                x_velocity: -0x0001_8000,
                y_velocity: -0x0001_8000,
            },
            "ASM Hand.asm loc_32A5A rolls back position before storing `$FFFE8000` into x_vel and y_vel"
        );
    }

    #[test]
    fn hand_jump_collision_loc_32a70_rolls_back_and_bounces_up_right() {
        assert_eq!(
            hand_jump_collision_bounce_up_right_loc_32a70(
                0x1000_0000,
                0x2000_0000,
                -0x0000_8000,
                -0x0000_4000,
            ),
            HandJumpCollisionBounceLoc32A70 {
                x_pos: 0x1000_8000,
                y_pos: 0x2000_4000,
                x_velocity: 0x0001_8000,
                y_velocity: -0x0001_8000,
            },
            "ASM Hand.asm loc_32A70 rolls back position before storing `$18000` into x_vel and `$FFFE8000` into y_vel"
        );
    }

    #[test]
    fn hand_ground_collision_loc_329e2_reverses_velocity_and_direction() {
        assert_eq!(
            hand_ground_collision_reverse_loc_329e2(-0x0000_3000, false),
            HandGroundCollisionReverseLoc329E2 {
                x_velocity: 0x0000_3000,
                x_direction_flag: true,
            },
            "ASM Hand.asm loc_329E2 negates x_vel as a long word and inverts clear x_direction to set"
        );
        assert_eq!(
            hand_ground_collision_reverse_loc_329e2(0x0000_4800, true),
            HandGroundCollisionReverseLoc329E2 {
                x_velocity: -0x0000_4800,
                x_direction_flag: false,
            },
            "ASM Hand.asm loc_329E2 applies the same long-word reversal when the hand turns back left"
        );
    }

    #[test]
    fn hand_ground_collision_loc_329ec_marks_kid_below_contact() {
        assert_eq!(
            hand_kid_below_contact_loc_329ec(),
            HandKidBelowContactLoc329EC {
                kid_collision_type: 0x0028,
                kid_object_meta: 0,
            },
            "ASM Hand.asm loc_329EC writes colid_kidbelow to Kid collision_type and clears Kid object_meta"
        );
    }

    #[test]
    fn hand_grab_snap_loc_3294c_uses_raw_direction_and_clears_fraction() {
        assert_eq!(
            hand_grab_snap_loc_3294c(0x0064_8000, 0x00C8_4000, false),
            HandGrabSnapLoc3294C {
                x_pos: 0x0066_0000,
                y_pos: 0x00C4_0000,
            },
            "ASM Hand.asm loc_3294C: clear x_direction stores Kid integer X+2 and clears both fractional words"
        );
        assert_eq!(
            hand_grab_snap_loc_3294c(0x0064_8000, 0x00C8_4000, true).x_pos,
            0x0062_0000,
            "ASM Hand.asm loc_3294C: set x_direction subtracts four from the prepared Kid X+2 word, yielding Kid X-2"
        );
    }

    #[test]
    fn hand_grabbing_step_uses_loc_3294c_snap() {
        let mut enemy = test_enemy(EnemyFamily::Hand, 0, 0, 1);
        enemy.hand_state = HandState::Grabbing;
        enemy.facing = Direction::Left;
        enemy.wait_timer = 1;
        enemy.x_velocity = 0x0003_0000;
        enemy.y_velocity = -0x0004_0000;

        step_hand(&mut enemy, None, Some(0x0064_8000), Some(0x00C8_4000), None);

        assert_eq!(
            (enemy.x_pos, enemy.y_pos, enemy.x_velocity, enemy.y_velocity),
            (0x0066_0000, 0x00C4_0000, 0, 0),
            "ASM Hand.asm loc_3294C drives the native grabbing step through integer X+2/Y-4 snap and velocity clear"
        );
        assert_eq!(
            enemy.wait_timer, 0x0000,
            "ASM Hand.asm loc_3292E still decrements d3 when no Kid direction byte is available to match"
        );
    }

    #[test]
    fn hand_grab_timer_loc_3292e_decrements_only_on_kid_facing_change() {
        let mut enemy = test_enemy(EnemyFamily::Hand, 0, 0, 1);
        enemy.hand_state = HandState::Grabbing;
        enemy.facing = Direction::Left;
        enemy.wait_timer = HAND_GRAB_DURATION;

        step_hand(
            &mut enemy,
            None,
            Some(0x0064_8000),
            Some(0x00C8_4000),
            Some(Direction::Left),
        );
        let after_first_sample = enemy.wait_timer & 0x00FF;

        step_hand(
            &mut enemy,
            None,
            Some(0x0064_8000),
            Some(0x00C8_4000),
            Some(Direction::Left),
        );
        assert_eq!(
            enemy.wait_timer & 0x00FF,
            after_first_sample,
            "ASM Hand.asm loc_3292E loops without decrementing d3 while Kid `$16` is unchanged"
        );

        step_hand(
            &mut enemy,
            None,
            Some(0x0064_8000),
            Some(0x00C8_4000),
            Some(Direction::Right),
        );
        assert_eq!(
            enemy.wait_timer & 0x00FF,
            after_first_sample - 1,
            "ASM Hand.asm loc_3292E decrements the grab counter only after Kid `$16` changes"
        );
    }

    #[test]
    fn hand_release_loc_32970_restarts_jump_setup() {
        assert_eq!(
            hand_release_loc_32970(),
            HandReleaseLoc32970 {
                kid_grabbed_by_hand: false,
                hand_state: HandState::Jumping,
                on_ground: false,
                jump_speed_seed: 2,
                collision_probe_half_width_px: 0x0E,
                collision_probe_height_px: 9,
            },
            "ASM Hand.asm loc_32970 clears KidGrabbedByHand, probes 9x14, restores movement/collision, and branches to loc_328AA with d5=2"
        );
    }

    #[test]
    fn hand_grabbing_step_uses_loc_32970_release_jump() {
        let mut enemy = test_enemy(EnemyFamily::Hand, 0, 0, 1);
        enemy.hand_state = HandState::Grabbing;
        enemy.facing = Direction::Left;
        enemy.on_ground = true;
        enemy.wait_timer = 0;

        step_hand(&mut enemy, None, Some(0x0064_8000), Some(0x00C8_4000), None);

        assert_eq!(
            (
                enemy.hand_state,
                enemy.on_ground,
                enemy.x_pos,
                enemy.y_pos,
                enemy.x_velocity,
                enemy.y_velocity,
            ),
            (
                HandState::Jumping,
                false,
                0x0066_0000,
                0x00C4_0000,
                -0x0003_0000,
                -0x0002_0000,
            ),
            "ASM Hand.asm loc_32970 releases the grab into a loc_328AA jump with d5=2 instead of returning to patrol"
        );
    }

    #[test]
    fn hand_walks_until_player_detected_ahead() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 1);
        enemy.hand_state = HandState::Walking;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        // Player 0x30 px ahead (within 0x40 range for HP=1), 8px above
        let player_x = Some((60 + 0x30) << 16);
        let player_y = Some(((17 * 16) - 8) << 16);
        step_hand(&mut enemy, Some(&grid), player_x, player_y, None);
        assert_eq!(
            enemy.hand_state,
            HandState::Jumping,
            "should jump at detected player"
        );
        assert!(enemy.y_velocity < 0, "should jump upward");
        assert!(
            enemy.x_velocity > 0,
            "ASM Hand.asm loc_328AA skips the X negation when raw x_direction is set for right-facing Hands"
        );
    }

    #[test]
    fn hand_detection_loc_3284c_uses_kid_y_minus_16_cone_origin() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 1);
        enemy.hand_state = HandState::Walking;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;

        step_hand(
            &mut enemy,
            Some(&grid),
            Some((60 + 0x30) << 16),
            Some(((17 * 16) - 0x18) << 16),
            None,
        );

        assert_eq!(
            enemy.hand_state,
            HandState::Jumping,
            "ASM Hand.asm loc_3284C negates `KidY-16-HandY` before the cone compare, so dx=$30 still accepts a Kid 24px above"
        );
    }

    #[test]
    fn hand_walking_loc_32804_integrates_subpixel_motion_before_floor_probe() {
        let grid = grid_with_platform(3, 0, 20);
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, 48 << 16, 1);
        enemy.hand_state = HandState::Walking;
        enemy.facing = Direction::Right;

        for _ in 0..6 {
            step_hand(&mut enemy, Some(&grid), None, None, None);
        }

        assert_eq!(
            enemy.x_pos,
            (60 << 16) + (6 * HAND_WALK_SPEED[0] as u32),
            "ASM Hand.asm loc_32804 hibernates after movement; native must accumulate the tier-1 subpixel walk delta before probes"
        );
        assert_eq!(enemy.facing, Direction::Right);
        assert_eq!(enemy.x_velocity, HAND_WALK_SPEED[0]);
    }

    #[test]
    fn hand_walking_loc_32840_reverses_at_right_floor_edge() {
        let grid = grid_with_platform(3, 0, 4);
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, 48 << 16, 1);
        enemy.hand_state = HandState::Walking;
        enemy.facing = Direction::Right;

        step_hand(&mut enemy, Some(&grid), None, None, None);

        assert_eq!(
            (enemy.facing, enemy.x_velocity),
            (Direction::Left, -HAND_WALK_SPEED[0]),
            "ASM Hand.asm loc_32804/loc_32840 reverses when the right lead probe loses `$4000` floor support"
        );
    }

    #[test]
    fn hand_walking_loc_32840_reverses_at_left_floor_edge() {
        let grid = grid_with_solid_cells(8, 5, &[(0, 3), (5, 3)]);
        let mut enemy = test_enemy(EnemyFamily::Hand, 68 << 16, 48 << 16, 1);
        enemy.hand_state = HandState::Walking;
        enemy.facing = Direction::Left;

        step_hand(&mut enemy, Some(&grid), None, None, None);

        assert_eq!(
            (enemy.facing, enemy.x_velocity),
            (Direction::Right, HAND_WALK_SPEED[0]),
            "ASM Hand.asm loc_32836/loc_32840 reverses when loc_32AFE reports left edge d6 clear but right edge d7 still has `$4000` support"
        );
    }

    #[test]
    fn hand_walking_loc_32afe_no_support_enters_death_sequence() {
        let grid = grid_with_solid_cells(8, 5, &[]);
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, 48 << 16, 1);
        enemy.hand_state = HandState::Walking;
        enemy.facing = Direction::Right;

        step_hand(&mut enemy, Some(&grid), None, None, None);

        assert_eq!(enemy.hit_points, 0);
        assert!(enemy.death_animation_active);
        assert_eq!(
            enemy.wait_timer, HAND_DEATH_ANIMATION_FRAMES,
            "ASM Hand.asm loc_32804 branches to loc_32AA2 when loc_32AFE leaves d4 zero because no scanned word has `$4000` support"
        );
    }

    #[test]
    fn hand_walking_loc_32840_snaps_and_reverses_at_right_wall() {
        let grid = grid_with_solid_cells(8, 6, &[(5, 3)]);
        let mut enemy = test_enemy(EnemyFamily::Hand, 72 << 16, 64 << 16, 1);
        enemy.hand_state = HandState::Walking;
        enemy.facing = Direction::Right;

        step_hand(&mut enemy, Some(&grid), None, None, None);

        assert_eq!(
            (enemy.x_pos, enemy.facing, enemy.x_velocity),
            (72 << 16, Direction::Left, -HAND_WALK_SPEED[0]),
            "ASM Hand.asm loc_32840 reverses after native snaps the right edge to the solid wall surface"
        );
    }

    #[test]
    fn hand_walking_loc_32840_snaps_and_reverses_at_left_wall() {
        let grid = grid_with_solid_cells(8, 6, &[(4, 3)]);
        let mut enemy = test_enemy(EnemyFamily::Hand, 88 << 16, 64 << 16, 1);
        enemy.hand_state = HandState::Walking;
        enemy.facing = Direction::Left;

        step_hand(&mut enemy, Some(&grid), None, None, None);

        assert_eq!(
            (enemy.x_pos, enemy.facing, enemy.x_velocity),
            (88 << 16, Direction::Right, HAND_WALK_SPEED[0]),
            "ASM Hand.asm loc_32840 reverses after native snaps the left edge to the solid wall surface"
        );
    }

    #[test]
    fn hand_detection_loc_3284c_keeps_strict_front_and_vertical_edges() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;

        let mut right_equal_x = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 1);
        right_equal_x.hand_state = HandState::Walking;
        right_equal_x.facing = Direction::Right;
        step_hand(
            &mut right_equal_x,
            Some(&grid),
            Some(61 << 16),
            Some(floor_y),
            None,
        );
        assert_eq!(
            right_equal_x.hand_state,
            HandState::Walking,
            "ASM Hand.asm loc_3284C right-facing path rejects dx == 0 because `tst.w d6; bmi` is false and range must be in front"
        );

        let mut left_equal_x = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 1);
        left_equal_x.hand_state = HandState::Walking;
        left_equal_x.facing = Direction::Left;
        step_hand(
            &mut left_equal_x,
            Some(&grid),
            Some(60 << 16),
            Some(floor_y),
            None,
        );
        assert_eq!(
            left_equal_x.hand_state,
            HandState::Walking,
            "ASM Hand.asm loc_32886 rejects dx == 0 for the clear x_direction left-facing path"
        );

        let mut left_boundary = test_enemy(EnemyFamily::Hand, 100 << 16, floor_y, 1);
        left_boundary.hand_state = HandState::Walking;
        left_boundary.facing = Direction::Left;
        step_hand(
            &mut left_boundary,
            Some(&grid),
            Some((100 - 0x40) << 16),
            Some(((17 * 16) - 8) << 16),
            None,
        );
        assert_eq!(
            left_boundary.hand_state,
            HandState::Jumping,
            "ASM Hand.asm loc_32886 accepts the exact left range boundary because only `blt` rejects beyond it"
        );
        assert!(
            left_boundary.x_velocity < 0,
            "ASM Hand.asm loc_328AA negates X when raw x_direction is clear for left-facing Hands"
        );

        let mut left_beyond = test_enemy(EnemyFamily::Hand, 100 << 16, floor_y, 1);
        left_beyond.hand_state = HandState::Walking;
        left_beyond.facing = Direction::Left;
        step_hand(
            &mut left_beyond,
            Some(&grid),
            Some((100 - 0x42) << 16),
            Some(((17 * 16) - 8) << 16),
            None,
        );
        assert_eq!(
            left_beyond.hand_state,
            HandState::Walking,
            "ASM Hand.asm loc_32886 rejects one pixel beyond the negative detection range after the hibernated subpixel walk updates x_pos"
        );

        let mut vertical_beyond = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 1);
        vertical_beyond.hand_state = HandState::Walking;
        vertical_beyond.facing = Direction::Right;
        step_hand(
            &mut vertical_beyond,
            Some(&grid),
            Some((60 + 0x20) << 16),
            Some(((17 * 16) - 17) << 16),
            None,
        );
        assert_eq!(
            vertical_beyond.hand_state,
            HandState::Walking,
            "ASM Hand.asm loc_3284C rejects Kid positions more than 16px above the Hand before the cone check"
        );
    }

    #[test]
    fn hand_jumping_loc_328f0_applies_gravity_then_moves_without_grid() {
        let mut enemy = test_enemy(EnemyFamily::Hand, 20 << 16, 30 << 16, 1);
        enemy.hand_state = HandState::Jumping;
        enemy.x_velocity = 0x0002_0000;
        enemy.y_velocity = -0x0001_0000;

        step_hand(&mut enemy, None, None, None, None);

        assert_eq!(
            (enemy.x_pos, enemy.y_pos, enemy.x_velocity, enemy.y_velocity),
            (22 << 16, 0x001D_4000, 0x0002_0000, -0x0000_C000),
            "ASM Hand.asm loc_328F0 applies `$4000` gravity and native moves by the updated velocity before terrain probes"
        );
    }

    #[test]
    fn hand_jumping_loc_329f8_wall_probe_uses_moved_pixel_position() {
        let grid = grid_with_solid_cells(8, 6, &[(3, 3)]);
        let mut enemy = test_enemy(EnemyFamily::Hand, 40 << 16, 64 << 16, 1);
        enemy.hand_state = HandState::Jumping;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0;

        step_hand(&mut enemy, Some(&grid), None, None, None);

        assert_eq!(
            (enemy.x_pos, enemy.x_velocity),
            (41 << 16, 0),
            "ASM Hand.asm loc_329F8 collision step must use the moved integer X and positive wall side before clearing x_vel"
        );
    }

    #[test]
    fn hand_jumping_loc_328f0_lands_only_while_falling() {
        let grid = grid_with_platform(3, 0, 8);

        let mut falling = test_enemy(EnemyFamily::Hand, 40 << 16, 48 << 16, 1);
        falling.hand_state = HandState::Jumping;
        falling.y_velocity = 0;
        step_hand(&mut falling, Some(&grid), None, None, None);
        assert_eq!(
            (
                falling.hand_state,
                falling.on_ground,
                falling.y_pos,
                falling.x_velocity,
                falling.y_velocity,
            ),
            (HandState::Walking, true, 48 << 16, 0, 0),
            "ASM Hand.asm loc_328F0 floor contact returns to patrol after downward movement reaches the floor surface"
        );

        let mut not_falling = test_enemy(EnemyFamily::Hand, 40 << 16, 48 << 16, 1);
        not_falling.hand_state = HandState::Jumping;
        not_falling.y_velocity = -HAND_GRAVITY;
        step_hand(&mut not_falling, Some(&grid), None, None, None);
        assert_eq!(
            (
                not_falling.hand_state,
                not_falling.on_ground,
                not_falling.y_pos,
                not_falling.y_velocity,
            ),
            (HandState::Jumping, false, 48 << 16, 0),
            "ASM Hand.asm loc_328F0 applies gravity but does not run the floor landing branch when post-gravity y_vel is zero"
        );
    }

    #[test]
    fn hand_jumping_landing_grab_window_uses_exact_range_and_height() {
        let grid = grid_with_platform(3, 0, 12);

        let mut inside = test_enemy(EnemyFamily::Hand, 60 << 16, 48 << 16, 1);
        inside.hand_state = HandState::Jumping;
        step_hand(
            &mut inside,
            Some(&grid),
            Some((60 + 0x40) << 16),
            Some((48 - 16) << 16),
            None,
        );
        assert_eq!(
            (inside.hand_state, inside.wait_timer),
            (HandState::Grabbing, HAND_GRAB_DURATION),
            "ASM Hand.asm loc_32906 grab setup is reachable at the exact range and 16px vertical landing boundary"
        );

        let mut x_beyond = test_enemy(EnemyFamily::Hand, 60 << 16, 48 << 16, 1);
        x_beyond.hand_state = HandState::Jumping;
        step_hand(
            &mut x_beyond,
            Some(&grid),
            Some((60 + 0x41) << 16),
            Some((48 - 16) << 16),
            None,
        );
        assert_eq!(
            x_beyond.hand_state,
            HandState::Walking,
            "ASM Hand.asm loc_3284C-style landing grab rejects one pixel beyond `$44(a5)` range"
        );

        let mut y_beyond = test_enemy(EnemyFamily::Hand, 60 << 16, 48 << 16, 1);
        y_beyond.hand_state = HandState::Jumping;
        step_hand(
            &mut y_beyond,
            Some(&grid),
            Some((60 + 0x40) << 16),
            Some((48 - 17) << 16),
            None,
        );
        assert_eq!(
            y_beyond.hand_state,
            HandState::Walking,
            "ASM Hand.asm loc_32906 grab setup requires the landing target to stay within the 16px vertical window"
        );
    }

    #[test]
    fn hand_ignores_player_behind() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 1);
        enemy.hand_state = HandState::Walking;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        // Player 0x30 px behind (to the left)
        let player_x = Some((60 - 0x30) << 16);
        let player_y = Some(((17 * 16) - 8) << 16);
        step_hand(&mut enemy, Some(&grid), player_x, player_y, None);
        assert_eq!(
            enemy.hand_state,
            HandState::Walking,
            "should not detect player behind"
        );
    }

    #[test]
    fn hand_ignores_player_beyond_range() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 1);
        enemy.hand_state = HandState::Walking;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        // Player 0x50 px ahead (beyond 0x40 range for HP=1)
        let player_x = Some((60 + 0x50) << 16);
        let player_y = Some(((17 * 16) - 8) << 16);
        step_hand(&mut enemy, Some(&grid), player_x, player_y, None);
        assert_eq!(
            enemy.hand_state,
            HandState::Walking,
            "should not detect player beyond detection range"
        );
    }

    #[test]
    fn hand_higher_hp_has_longer_detection_range() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y, 3);
        enemy.hand_state = HandState::Walking;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        // Player 0x50 px ahead — beyond HP=1 range (0x40) but within HP=3 range (0x64)
        let player_x = Some((60 + 0x50) << 16);
        let player_y = Some(((17 * 16) - 8) << 16);
        step_hand(&mut enemy, Some(&grid), player_x, player_y, None);
        assert_eq!(
            enemy.hand_state,
            HandState::Jumping,
            "HP=3 hand should detect player at 0x50 range"
        );
    }

    #[test]
    fn hand_lands_and_returns_to_walking() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Hand, 60 << 16, floor_y - (30 << 16), 1);
        enemy.hand_state = HandState::Jumping;
        let jump = hand_jump_components_loc_328aa(hand_jump_seed_loc_328a6(1), true, 0, 0, 5, 0);
        enemy.x_velocity = hand_x_velocity_loc_328e0(jump.x_velocity_word);
        enemy.y_velocity = jump.y_velocity;
        enemy.on_ground = false;
        for _ in 0..300 {
            step_hand(&mut enemy, Some(&grid), None, None, None);
            if enemy.hand_state == HandState::Walking {
                break;
            }
        }
        assert_eq!(
            enemy.hand_state,
            HandState::Walking,
            "hand should return to walking after landing"
        );
        assert!(enemy.on_ground);
    }

    // --- Lion ---

    #[test]
    fn lion_paces_then_charges_when_timer_expires_and_player_in_range() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Lion, 60 << 16, floor_y, 6);
        enemy.lion_state = LionState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 1; // about to expire
        let player_x = Some((60 + 0x50) << 16); // within detection range
        let mut projs = vec![];
        step_lion(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        // Timer ticked to 0 but hasn't charged yet (timer was 1, decremented to 0)
        assert_eq!(enemy.lion_state, LionState::Walking);
        // One more step with timer at 0 should start the roar wind-up.
        step_lion(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        assert_eq!(enemy.lion_state, LionState::Roaring);
        assert!(projs.is_empty());

        for _ in 0..=LION_ROAR_FIRE_DELAY {
            step_lion(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        }
        assert_eq!(enemy.lion_state, LionState::PostFire);
        assert_eq!(enemy.facing, Direction::Right);
        assert_eq!(
            projs.len(),
            1,
            "should fire projectile at the roar fire sprite"
        );

        for _ in 0..=LION_POST_FIRE_RESUME_DELAY {
            step_lion(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        }
        assert_eq!(enemy.lion_state, LionState::ResumingCharge);
    }

    #[test]
    fn lion_postfire_resume_skips_roar_gate_loc_3e0f4_3e11e() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Lion, 60 << 16, floor_y, 6);
        enemy.lion_state = LionState::PostFire;
        enemy.on_ground = true;
        enemy.facing = Direction::Left;
        enemy.wait_timer = 0;
        enemy.ufo_beam_segments = 0;
        let player_x = Some(20u32 << 16);
        let start_x = enemy.x_pos;

        step_lion(&mut enemy, Some(&grid), player_x, &mut None);
        assert_eq!(
            enemy.lion_state,
            LionState::ResumingCharge,
            "ASM Lion.asm loc_3E0F4 restores the charge animation and branches to loc_3E11E, not loc_3E078's roar timer gate"
        );

        step_lion(&mut enemy, Some(&grid), player_x, &mut None);
        assert!(
            enemy.x_pos < start_x,
            "ASM Lion.asm loc_3E11E should resume charge movement on the next frame instead of starting another roar"
        );
    }

    #[test]
    fn lion_charges_unconditionally_when_timer_expires_regardless_of_distance() {
        // ASM loc_3E078: timer expiry always starts roar → charge, no distance gate.
        // The distance check at loc_3E03C only affects facing direction and the $4E
        // "has seen Kid" flag, NOT whether the timer ticks or fires.
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Lion, 60 << 16, floor_y, 6);
        enemy.lion_state = LionState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 0;
        let player_x = Some((60 + 0xFF) << 16); // far away (255px)
        let mut projs = vec![];
        step_lion(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        assert_eq!(
            enemy.lion_state,
            LionState::Roaring,
            "ASM starts the roar unconditionally on timer expiry — no distance gate"
        );
        for _ in 0..=LION_ROAR_FIRE_DELAY {
            step_lion(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        }
        assert_eq!(
            projs.len(),
            1,
            "should fire projectile even when Kid is far"
        );
    }

    #[test]
    fn lion_timer_expiry_enters_roar_before_projectile_loc_3e078_3e09e() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Lion, 60 << 16, floor_y, 6);
        enemy.lion_state = LionState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 0;
        let player_x = Some((60 + 0x50) << 16);
        let mut projs = vec![];

        step_lion(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));

        assert_eq!(
            enemy.lion_state,
            LionState::Roaring,
            "ASM Lion.asm loc_3E078 starts stru_3DF0E; loc_3E09E waits for LnkTo_unk_C7E78 before firing"
        );
        assert!(
            projs.is_empty(),
            "ASM Lion.asm loc_3E09E should not allocate the projectile on the timer-expiry frame"
        );
    }

    #[test]
    fn lion_walking_tracks_far_kid_before_timer_tick_loc_3e03c_3e074() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Lion, 60 << 16, floor_y, 6);
        enemy.lion_state = LionState::Walking;
        enemy.on_ground = true;
        enemy.facing = Direction::Left;
        enemy.wait_timer = 2;
        let player_x = Some((60 + 0x50) << 16);

        step_lion(&mut enemy, Some(&grid), player_x, &mut None);

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Lion.asm loc_3E03C -> loc_3E074 should face Kid on the right before loc_3E078 ticks `$54(a5)`"
        );
        assert_eq!(
            enemy.wait_timer, 1,
            "ASM Lion.asm loc_3E078 should still tick the charge timer after tracking"
        );
    }

    #[test]
    fn lion_walking_tracking_latch_holds_until_outer_band_exit_loc_3e054_3e066() {
        let grid = grid_with_platform(17, 0, 240);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Lion, 120 << 16, floor_y, 6);
        enemy.lion_state = LionState::Walking;
        enemy.on_ground = true;
        enemy.facing = Direction::Left;
        enemy.wait_timer = 4;
        enemy.scorpion_tracking_lock = true;

        step_lion(&mut enemy, Some(&grid), Some((120 + 0x50) << 16), &mut None);

        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM Lion.asm loc_3E054 should hold current facing while `$4E(a5)` is set and Kid is inside the wider +/-$A0 band"
        );
        assert!(
            enemy.scorpion_tracking_lock,
            "ASM Lion.asm loc_3E054 should keep `$4E(a5)` set while holding"
        );

        step_lion(&mut enemy, Some(&grid), Some((120 + 0xA0) << 16), &mut None);

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Lion.asm loc_3E066 -> loc_3E074 should clear `$4E(a5)` and face Kid once the outer band is exceeded"
        );
        assert!(
            !enemy.scorpion_tracking_lock,
            "ASM Lion.asm loc_3E066 should clear `$4E(a5)` on outer-band exit"
        );
    }

    #[test]
    fn lion_charge_returns_to_walking_on_overshoot() {
        // Lion charge ends when overshooting player by >112px ($70), not a
        // fixed duration timer. ASM: Lion.asm loc_3E450/3E604.
        let grid = grid_with_platform(17, 0, 200);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(
            EnemyFamily::Lion,
            (60 + LION_CHARGE_OVERSHOOT as u32) << 16,
            floor_y,
            6,
        );
        enemy.lion_state = LionState::Charging;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        enemy.wait_timer = LION_CHARGE_TIMER_INIT;
        enemy.shared_walker_state = SHARED_WALKER_STATE_FACE_RIGHT;
        // Player is at x=60; start exactly at the $70 edge so the next
        // charge movement enters loc_3E476's overshoot exit.
        let player_x = Some(60u32 << 16);
        for _ in 0..20 {
            if enemy.lion_state == LionState::Walking {
                break;
            }
            step_lion(&mut enemy, Some(&grid), player_x, &mut None);
        }
        assert_eq!(
            enemy.lion_state,
            LionState::Walking,
            "should return to walking after overshooting player"
        );
        assert_eq!(enemy.wait_timer, LION_COOLDOWN_AFTER_FIRE);
    }

    #[test]
    fn lion_charge_state_zero_enters_right_alert_before_x_commit_loc_3e450_3e4ce() {
        let grid = grid_with_platform(17, 0, 200);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Lion, 60 << 16, floor_y, 6);
        enemy.lion_state = LionState::Charging;
        enemy.on_ground = true;
        enemy.facing = Direction::Left;
        enemy.wait_timer = LION_CHARGE_TIMER_INIT;
        enemy.shared_walker_state = 0;
        let start_x = enemy.x_pos;

        step_lion(&mut enemy, Some(&grid), Some((60 + 0x20) << 16), &mut None);

        assert_eq!(
            enemy.x_pos, start_x,
            "ASM Lion.asm loc_3E450 should choose loc_3E4CE before loc_3E4C6 commits the candidate X position"
        );
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Lion.asm loc_3E4CE should set x_direction when Kid is to the right inside the +/-$60 calm band"
        );
        assert_eq!(
            enemy.shared_walker_state, SHARED_WALKER_STATE_FACE_RIGHT,
            "ASM Lion.asm loc_3E4CE should store `$42(a3)=1` for the right-facing alert state"
        );
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);
    }

    #[test]
    fn lion_hp8_charges_faster() {
        assert!(lion_charge_speed(8) > lion_charge_speed(7));
    }

    // --- Robot ---

    #[test]
    fn robot_spawn_loc_3a7ac_faces_toward_kid_and_seeds_hp() {
        assert_eq!(
            spawn_facing(EnemyFamily::Robot, 100u32 << 16, 80u32 << 16),
            Direction::Left,
            "ASM Robot.asm loc_3A7AC keeps default x_direction=0 when Kid is left"
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Robot, 100u32 << 16, 100u32 << 16),
            Direction::Right,
            "ASM Robot.asm loc_3A7AC sets x_direction=$FF when Kid is level or right"
        );
        assert_eq!(
            spawn_hit_points(EnemyFamily::Robot, 0),
            2,
            "ASM Robot.asm loc_3A7AC stores enemy_level+2 in current_hp"
        );
        assert_eq!(
            spawn_hit_points(EnemyFamily::Robot, 1),
            3,
            "ASM Robot.asm loc_3A7AC keeps raw layout level as the HP tier"
        );
    }

    #[test]
    fn robot_walks_then_shoots_and_resumes_walking_loc_3aac4() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Robot, 60 << 16, floor_y, 2);
        enemy.robot_state = RobotState::Walking;
        enemy.facing = Direction::Right;
        enemy.on_ground = true;
        enemy.wait_timer = 1;
        let player_x = Some(80u32 << 16);
        let mut projs = vec![];
        let mut rng = RomRandomState::default();
        while (rng.clone().next_long() as u16) > 0x000A {
            rng.next_long();
        }
        // Timer at 1 → decrements to 0
        step_robot_with_rng(
            &mut enemy,
            Some(&grid),
            player_x,
            &mut Some(&mut projs),
            &mut rng,
            None,
        );
        assert_eq!(enemy.robot_state, RobotState::Walking);
        assert!(projs.is_empty());
        // Timer at 0 → shoot, then resume walking through loc_3AAC4.
        step_robot_with_rng(
            &mut enemy,
            Some(&grid),
            player_x,
            &mut Some(&mut projs),
            &mut rng,
            None,
        );
        assert_eq!(
            enemy.robot_state,
            RobotState::Walking,
            "ASM Robot.asm loc_3AAC4 resumes the walker after shooting; loc_3A96C charge is collision-routed"
        );
        assert_eq!(
            enemy.wait_timer, ROBOT_SHOOT_INTERVAL,
            "native approximates the next shared sub_3C4F8 shoot check with the existing shoot interval"
        );
        assert_eq!(projs.len(), 1, "robot should fire a projectile");
        assert_eq!(
            projs[0].x_pos,
            enemy.x_pos + (26 << 16),
            "ASM Robot.asm loc_3AC0C should spawn the right-facing shot 26px from the muzzle"
        );
        assert_eq!(
            projs[0].y_pos,
            enemy.y_pos - (16 << 16),
            "ASM Robot.asm loc_3AC0C should apply child head offset `$12` into a -16px shot Y delta"
        );
        assert_eq!(
            projs[0].x_velocity, ROBOT_PROJECTILE_SPEED,
            "ASM Robot.asm loc_3AC0C should launch right-facing shots at `$50000`"
        );
        assert_eq!(
            projs[0].kind,
            EnemyProjectileKind::RobotShot,
            "live Robot shots need the loc_3AC48 camera/collision lifecycle, not generic timer expiry"
        );
        assert_eq!(enemy.facing, Direction::Right, "should face toward player");
    }

    #[test]
    fn robot_sub_3c4f8_aim_gate_turns_before_shooting_when_facing_away() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Robot, 60 << 16, floor_y, 2);
        enemy.robot_state = RobotState::Walking;
        enemy.facing = Direction::Left;
        enemy.on_ground = true;
        enemy.wait_timer = 0;
        let player_x = Some(80u32 << 16);
        let mut projs = vec![];
        let mut rng = RomRandomState::default();
        while (rng.clone().next_long() as u16) > 0x000A {
            rng.next_long();
        }

        step_robot_with_rng(
            &mut enemy,
            Some(&grid),
            player_x,
            &mut Some(&mut projs),
            &mut rng,
            None,
        );

        assert_eq!(
            enemy.robot_state,
            RobotState::Aiming,
            "ASM sub_3C4F8 loc_3C588 should arm the Robot turn sequence instead of snapping to face Kid and firing"
        );
        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM sub_3C4F8 loc_3C58E keeps the current x_direction until the phase-2 flip"
        );
        assert!(
            projs.is_empty(),
            "ASM sub_3C4F8 should delay Enemy03_Robot_Shoot until the turn sequence completes"
        );
    }

    #[test]
    fn robot_sub_3c4f8_aim_sequence_flips_then_shoots() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Robot, 60 << 16, floor_y, 2);
        enemy.robot_state = RobotState::Aiming;
        enemy.facing = Direction::Left;
        enemy.on_ground = true;
        enemy.wait_timer = 26;
        let mut projs = vec![];

        for _ in 0..17 {
            step_robot(
                &mut enemy,
                Some(&grid),
                Some(80u32 << 16),
                &mut Some(&mut projs),
            );
        }
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM sub_3C4F8 loc_3C5A2 flips x_direction during phase 2, before Enemy03_Robot_Shoot"
        );
        assert!(projs.is_empty());

        while enemy.robot_state == RobotState::Aiming {
            step_robot(
                &mut enemy,
                Some(&grid),
                Some(80u32 << 16),
                &mut Some(&mut projs),
            );
        }

        assert_eq!(
            enemy.robot_state,
            RobotState::Walking,
            "ASM Robot.asm loc_3AAC4 resumes walking after the delayed shoot sequence"
        );
        assert_eq!(enemy.wait_timer, ROBOT_SHOOT_INTERVAL);
        assert_eq!(projs.len(), 1);
        assert_eq!(projs[0].x_velocity, ROBOT_PROJECTILE_SPEED);
    }

    #[test]
    fn robot_walk_speed_loc_3a7ee_uses_binary_hp_threshold() {
        assert_eq!(
            robot_walk_speed(2),
            0x8000,
            "ASM Robot.asm loc_3A7EE adds one `$8000` for HP 2"
        );
        assert_eq!(
            robot_walk_speed(3),
            0x1_0000,
            "ASM Robot.asm loc_3A7EE preloads `$8000` before the shared add at HP 3"
        );
        assert_eq!(
            robot_walk_speed(4),
            0x1_0000,
            "ASM Robot.asm loc_3A7EE does not add extra speed tiers above HP 3"
        );
    }

    #[test]
    fn robot_walk_loc_3a884_dispatches_from_x_direction() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut left = test_enemy(EnemyFamily::Robot, 60 << 16, floor_y, 2);
        left.robot_state = RobotState::Walking;
        left.facing = Direction::Left;
        left.wait_timer = 10;
        let left_start = left.x_pos;

        step_robot(&mut left, Some(&grid), None, &mut None);

        assert_eq!(
            left.x_velocity,
            -robot_walk_speed(2),
            "ASM Robot.asm loc_3A884 x_direction=0 branches to the leftward shared walker"
        );
        assert!(left.x_pos < left_start);

        let mut right = test_enemy(EnemyFamily::Robot, 60 << 16, floor_y, 2);
        right.robot_state = RobotState::Walking;
        right.facing = Direction::Right;
        right.wait_timer = 10;
        let right_start = right.x_pos;

        step_robot(&mut right, Some(&grid), None, &mut None);

        assert_eq!(
            right.x_velocity,
            robot_walk_speed(2),
            "ASM Robot.asm loc_3A884 x_direction=$FF branches to the rightward shared walker"
        );
        assert!(right.x_pos > right_start);
    }

    #[test]
    fn robot_jump_sound_edge_loc_3a890_requires_sprite_timer_and_active_motion() {
        assert!(
            robot_jump_sound_edge_loc_3a890(true, 4, false),
            "ASM Robot.asm loc_3A890 plays sfx_Robot_jumping_on only when `addroffset_sprite` is C82A8, `animation_timer` is 4, and `$19(a3)` is clear"
        );
        assert!(
            !robot_jump_sound_edge_loc_3a890(false, 4, false),
            "ASM Robot.asm loc_3A890 skips sound on any other sprite link"
        );
        assert!(
            !robot_jump_sound_edge_loc_3a890(true, 3, false),
            "ASM Robot.asm loc_3A890 skips sound outside animation_timer 4"
        );
        assert!(
            !robot_jump_sound_edge_loc_3a890(true, 4, true),
            "ASM Robot.asm loc_3A890 skips sound while `$19(a3)` suppresses motion/audio"
        );
    }

    #[test]
    fn robot_collision_dispatch_loc_3a8ae_routes_parent_and_child_contacts() {
        assert_eq!(
            robot_collision_dispatch_loc_3a8ae(-1, 0x001C, false),
            RobotCollisionRouteLoc3A8Ae::Destroy,
            "ASM Robot.asm loc_3A8AE sends negative parent collision_type straight to the destroy route"
        );
        assert_eq!(
            robot_collision_dispatch_loc_3a8ae(0x0800, 0x001C, false),
            RobotCollisionRouteLoc3A8Ae::ContinueCharge,
            "ASM Robot.asm loc_3A8AE tests parent collision_type $800 before child damage IDs"
        );
        assert_eq!(
            robot_collision_dispatch_loc_3a8ae(0x001C, 0, false),
            RobotCollisionRouteLoc3A8Ae::HitReaction,
            "ASM Robot.asm loc_3A8AE routes parent collision_type $1C to the hit-reaction route"
        );
        assert_eq!(
            robot_collision_dispatch_loc_3a8ae(0, -1, false),
            RobotCollisionRouteLoc3A8Ae::EnterCharge,
            "ASM Robot.asm loc_3A8AE sends negative child `$38(a2)` to the charge-entry route"
        );
        assert_eq!(
            robot_collision_dispatch_loc_3a8ae(0, 0x002C, false),
            RobotCollisionRouteLoc3A8Ae::HitReaction,
            "ASM Robot.asm loc_3A8AE routes child `$38(a2)` $2C to the hit-reaction route"
        );
        assert_eq!(
            robot_collision_dispatch_loc_3a8ae(0, 0x0020, false),
            RobotCollisionRouteLoc3A8Ae::ClearAndWalk,
            "ASM Robot.asm loc_3A8AE clears Berzerker body-hit collisions when Berzerker_charging is zero"
        );
        assert_eq!(
            robot_collision_dispatch_loc_3a8ae(0, 0x0024, true),
            RobotCollisionRouteLoc3A8Ae::Destroy,
            "ASM Robot.asm loc_3A8AE destroys on Berzerker body-hit collisions when Berzerker_charging is set"
        );
        assert_eq!(
            robot_collision_dispatch_loc_3a8ae(0, 0x0800, false),
            RobotCollisionRouteLoc3A8Ae::ContinueCharge,
            "ASM Robot.asm loc_3A8AE routes child `$38(a2)` $800 to the charge continuation route"
        );
    }

    #[test]
    fn robot_berzerker_side_gate_loc_3a9cc_requires_active_charge() {
        assert_eq!(
            robot_berzerker_side_gate_loc_3a9cc(false),
            RobotCollisionRouteLoc3A8Ae::ClearAndWalk,
            "ASM Robot.asm loc_3A9CC should branch to loc_3A958 when `Berzerker_charging` is clear"
        );
        assert_eq!(
            robot_berzerker_side_gate_loc_3a9cc(true),
            RobotCollisionRouteLoc3A8Ae::Destroy,
            "ASM Robot.asm loc_3A9CC should fall through to the destruction prep path when `Berzerker_charging` is set"
        );
    }

    #[test]
    fn robot_destroy_prep_loc_3a9d2_sets_collision_flags_before_destroy() {
        assert_eq!(
            robot_destroy_prep_loc_3a9d2(),
            RobotDestroyPrepLoc3A9D2 {
                child_status_13: false,
                child_has_kid_collision: true,
                parent_has_kid_collision: true,
                plays_destroy_animation: true,
                hibernates_until_animation_finished: true,
                route: RobotCollisionRouteLoc3A8Ae::Destroy,
            },
            "ASM Robot.asm loc_3A9D2 should clear child `$13`, set child `$3D` and parent has_kid_collision, play the destroy animation, then leave via the destroy route"
        );
    }

    #[test]
    fn robot_hit_reaction_loc_3a8fc_decrements_hp_and_restores_child_on_nonlethal_hit() {
        // ASM Robot.asm loc_3A8FC enters the hit-reaction sequence; the cases
        // below pin the child offset applied by loc_3A90C within that path.
        assert_eq!(
            robot_hit_reaction_loc_3a8fc(3, false, false),
            RobotHitReactionLoc3A8Fc {
                remaining_hp: 2,
                route: RobotHitReactionRouteLoc3A8Fc::ClearAndWalk,
                next_parent_collision_type: 0,
                next_child_collision_type: 0,
                child_collision_latched: false,
                transient_child_y_offset_delta_px: 4,
                child_y_offset_delta_px: 0,
                child_sprite_update: RobotChildSpriteUpdateLoc3A8Fc::UseHitFrame,
                restores_walk_animations: true,
                replay_jump_sound: true,
            },
            "ASM Robot.asm loc_3A936 should restore the walk animations, clear the child collision latch, and undo the loc_3A90C child +4px offset on non-lethal active-motion hits"
        );
        assert_eq!(
            robot_hit_reaction_loc_3a8fc(2, true, true),
            RobotHitReactionLoc3A8Fc {
                remaining_hp: 1,
                route: RobotHitReactionRouteLoc3A8Fc::ClearAndWalk,
                next_parent_collision_type: 0,
                next_child_collision_type: 0,
                child_collision_latched: false,
                transient_child_y_offset_delta_px: 4,
                child_y_offset_delta_px: 0,
                child_sprite_update: RobotChildSpriteUpdateLoc3A8Fc::KeepCurrent,
                restores_walk_animations: true,
                replay_jump_sound: false,
            },
            "ASM Robot.asm loc_3A936 should still restore the walk animations after a non-lethal hit when `$4E(a5)` keeps the current child sprite and `$19(a3)` suppresses sound"
        );
        assert_eq!(
            robot_hit_reaction_loc_3a8fc(1, false, false),
            RobotHitReactionLoc3A8Fc {
                remaining_hp: 0,
                route: RobotHitReactionRouteLoc3A8Fc::EnterCharge,
                next_parent_collision_type: 0x0800,
                next_child_collision_type: 0x0800,
                child_collision_latched: true,
                transient_child_y_offset_delta_px: 4,
                child_y_offset_delta_px: 4,
                child_sprite_update: RobotChildSpriteUpdateLoc3A8Fc::UseHitFrame,
                restores_walk_animations: false,
                replay_jump_sound: false,
            },
            "ASM Robot.asm loc_3A90C should keep the child +4px offset on lethal hits before the charge-entry route seeds $800 collisions and skips the non-lethal sound path"
        );
    }

    #[test]
    fn robot_live_final_hit_enters_charge_instead_of_death_loc_3a8fc_3a96c() {
        let mut enemy = test_enemy(EnemyFamily::Robot, 60 << 16, 80 << 16, 0);
        enemy.hit_points = 1;
        enemy.robot_state = RobotState::Walking;
        enemy.facing = Direction::Right;

        let queued_death = enemy.take_damage_or_queue_death();

        assert!(
            !queued_death,
            "ASM Robot.asm loc_3A8FC branches to loc_3A96C on the final HP hit instead of deleting the Robot"
        );
        assert_eq!(
            enemy.robot_state,
            RobotState::Charging,
            "ASM Robot.asm loc_3A96C should seed the live Robot charge state"
        );
        assert_eq!(enemy.hit_points, 0);
        assert_eq!(enemy.wait_timer, ROBOT_CHARGE_FLIP_BASE);
    }

    #[test]
    fn robot_collision_clear_loc_3a958_clears_words_then_resumes_walk_direction() {
        assert_eq!(
            robot_collision_clear_loc_3a958(Direction::Left),
            RobotCollisionClearLoc3A958 {
                next_parent_collision_type: 0,
                next_child_collision_type: 0,
                resume_walk_direction: Direction::Left,
            },
            "ASM Robot.asm loc_3A958 should clear child `$38(a2)` and parent collision_type before x_direction=0 resumes the left walker"
        );
        assert_eq!(
            robot_collision_clear_loc_3a958(Direction::Right),
            RobotCollisionClearLoc3A958 {
                next_parent_collision_type: 0,
                next_child_collision_type: 0,
                resume_walk_direction: Direction::Right,
            },
            "ASM Robot.asm loc_3A958 should clear child `$38(a2)` and parent collision_type before x_direction=$FF resumes the right walker"
        );
    }

    #[test]
    fn robot_charge_entry_loc_3a96c_seeds_charge_state_and_collision_words() {
        assert_eq!(
            robot_charge_entry_loc_3a96c(Direction::Left),
            RobotChargeEntryLoc3A96C {
                tracking_latch_cleared: true,
                child_status_13_during_windup: false,
                child_status_13_after_windup: true,
                parent_has_kid_collision: true,
                child_has_kid_collision: true,
                charge_speed: ROBOT_CHARGE_SPEED,
                next_parent_collision_type: 0x0800,
                next_child_collision_type: 0x0800,
                flip_timer: ROBOT_CHARGE_FLIP_BASE,
                resume_walk_direction: Direction::Left,
            },
            "ASM Robot.asm loc_3A96C should clear `$4E(a5)`, seed `$800` collision words, load `$50(a5)=$28000`, and resume by x_direction"
        );
    }

    #[test]
    fn robot_charge_flips_direction_on_timer() {
        // ASM: charge is permanent — timer expiry flips direction, stays charging
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Robot, 60 << 16, floor_y, 2);
        enemy.robot_state = RobotState::Charging;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        enemy.wait_timer = 1;
        step_robot(&mut enemy, Some(&grid), None, &mut None);
        assert_eq!(enemy.wait_timer, 0);
        step_robot(&mut enemy, Some(&grid), None, &mut None);
        // Should flip direction but stay charging
        assert_eq!(enemy.robot_state, RobotState::Charging);
        assert_eq!(enemy.facing, Direction::Left);
        assert!(
            (ROBOT_CHARGE_FLIP_BASE..=ROBOT_CHARGE_FLIP_BASE + 127).contains(&enemy.wait_timer)
        );
        // Seeded RNG first byte is $C0; ($C0 >> 1) + $0F = $6F = 111.
        assert_eq!(enemy.wait_timer, 111);
    }

    #[test]
    fn robot_charge_exits_on_edge() {
        // ASM: edge during charge → return to walking
        let grid = grid_with_platform(17, 0, 10); // short platform
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Robot, 60 << 16, floor_y, 2);
        enemy.robot_state = RobotState::Charging;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        enemy.wait_timer = 100;
        // Step until edge detected
        for _ in 0..60 {
            step_robot(&mut enemy, Some(&grid), None, &mut None);
            if matches!(enemy.robot_state, RobotState::Walking) {
                break;
            }
        }
        assert_eq!(enemy.robot_state, RobotState::Walking);
        assert_eq!(enemy.wait_timer, ROBOT_SHOOT_INTERVAL);
    }

    // --- Orca ---

    #[test]
    fn orca_activation_wires_init_state_loc_3d194() {
        let init = orca_init_state_loc_3d194();
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![],
            unknown_word: 0,
            header_byte: 0,
            object_count: 1,
            objects: vec![EnemyLayoutObject {
                object_type: EnemyFamily::Orca.object_type(),
                flags: 0,
                hit_points_plus_one: 3,
                x: 0x0060,
                y: 0x00E0,
            }],
            segments: vec![],
        };
        let mut active = Vec::new();

        activate_enemies(
            &mut active,
            &layout,
            ActivationWindow {
                x_min: 0,
                x_max: 0x0200,
                y_min: 0,
                y_max: 0x0200,
            },
            0,
            0,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            0x0060_0000,
            0x00E0_0000,
            &HashSet::new(),
        );

        let orca = active.first().expect("Orca should activate from layout");
        assert_eq!(orca.family, EnemyFamily::Orca);
        assert_eq!(
            orca.hit_points, init.current_hp,
            "ASM Orca.asm loc_3D194 overwrites current_hp to 4 during activation"
        );
        assert_eq!(
            orca.walk_speed_override, init.walk_speed,
            "ASM Orca.asm loc_3D194 seeds both `$50(a5)` and `$5E(a5)` with `$8000`"
        );
        assert_eq!(
            orca.wait_timer, init.initial_lunge_timer,
            "ASM Orca.asm loc_3D194 writes `$12C` to `$54(a5)` before the first proximity check"
        );
    }

    #[test]
    fn orca_enters_windup_before_lunging_loc_3d320() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, floor_y, 4);
        enemy.orca_state = OrcaState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 0; // no cooldown
        let player_x = Some((60 + 0x20) << 16); // within $30 range
        step_orca(&mut enemy, Some(&grid), player_x);
        assert_eq!(enemy.orca_state, OrcaState::Windup);
        assert_eq!(
            enemy.wait_timer,
            orca_lunge_windup_loc_3d320(ORCA_LUNGE_WINDUP_DBF_SEED_LOC_3D320).hibernate_frames
                as u16,
            "ASM Orca.asm loc_3D320 hibernates six frames before loc_3D340/loc_3D348 launch"
        );
        assert_eq!(enemy.y_velocity, 0, "wind-up should not launch upward yet");
        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM Orca.asm loc_3D2F4 enters the lunge wind-up without refacing toward Kid"
        );
    }

    #[test]
    fn orca_lunge_preserves_current_facing_from_loc_3d2f4() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, floor_y, 4);
        enemy.orca_state = OrcaState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 0;
        enemy.facing = Direction::Left;

        // ASM Orca.asm loc_3D2F4 enters the lunge wind-up when Kid is within
        // +/-$30 px, but does not update `x_direction(a3)` before launch.
        let player_x = Some((60 + 0x20) << 16);
        step_orca(&mut enemy, Some(&grid), player_x);

        assert_eq!(enemy.orca_state, OrcaState::Windup);
        assert_eq!(enemy.facing, Direction::Left);
        for _ in
            0..orca_lunge_windup_loc_3d320(ORCA_LUNGE_WINDUP_DBF_SEED_LOC_3D320).hibernate_frames
        {
            step_orca(&mut enemy, Some(&grid), player_x);
        }
        assert_eq!(enemy.orca_state, OrcaState::Lunging);
        assert_eq!(enemy.x_velocity, -ORCA_LUNGE_X_SPEED);
    }

    #[test]
    fn orca_lunge_windup_matches_loc_3d320_dbf_count() {
        assert_eq!(
            orca_lunge_windup_loc_3d320(ORCA_LUNGE_WINDUP_DBF_SEED_LOC_3D320),
            OrcaLungeWindupLoc3D320 {
                hibernate_frames: 6,
                polls_collision_each_frame: true,
                branches_on_nonzero_collision: true,
            },
            "ASM Orca.asm loc_3D320 should hibernate six frames from d1=5 and poll collision before launch"
        );
        assert_eq!(
            orca_lunge_windup_loc_3d320(0).hibernate_frames,
            1,
            "ASM Orca.asm loc_3D320 DBF loop executes once before d1=0 exits"
        );
    }

    #[test]
    fn orca_lunge_right_x_velocity_matches_loc_3d340() {
        assert_eq!(
            orca_lunge_x_velocity_loc_3d340(Direction::Right),
            ORCA_LUNGE_X_SPEED,
            "ASM Orca.asm loc_3D340 should set x_vel to $30000 when x_direction is set"
        );
        assert_eq!(
            orca_lunge_x_velocity_loc_3d340(Direction::Left),
            -ORCA_LUNGE_X_SPEED,
            "Orca launch should mirror the right-facing velocity when x_direction is clear"
        );
    }

    #[test]
    fn orca_lunge_launch_tail_matches_loc_3d348() {
        assert_eq!(
            ORCA_LUNGE_Y_SPEED, -0x0002_8000,
            "ASM Orca.asm loc_3D348 stores `$FFFD8000` into y_vel(a3)"
        );
        assert_eq!(
            orca_lunge_launch_tail_loc_3d348(),
            OrcaLungeLaunchTailLoc3D348 {
                y_velocity: ORCA_LUNGE_Y_SPEED,
                sprite_mapping_offset_delta: 4,
                has_level_collision: true,
            },
            "ASM Orca.asm loc_3D348 should launch upward, advance the sprite mapping offset, and enable level collision"
        );
    }

    #[test]
    fn orca_lunge_collision_dispatch_matches_loc_3d35a() {
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(0),
            OrcaLungeCollisionRouteLoc3D35A::GravityAndAcceleration,
            "ASM Orca.asm loc_3D35A should treat zero collision as the airborne gravity path"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_FLOOR),
            OrcaLungeCollisionRouteLoc3D35A::FloorLanding,
            "ASM Orca.asm loc_3D35A should route floor collisions to loc_3D420"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_RIGHTWALL),
            OrcaLungeCollisionRouteLoc3D35A::WallBounce,
            "ASM Orca.asm loc_3D35A should route right-wall collisions to loc_3D40C"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_LEFTWALL),
            OrcaLungeCollisionRouteLoc3D35A::WallBounce,
            "ASM Orca.asm loc_3D35A should route left-wall collisions to loc_3D40C"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_CEILING),
            OrcaLungeCollisionRouteLoc3D35A::CeilingBounce,
            "ASM Orca.asm loc_3D35A should route ceiling collisions to loc_3D47A"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_SLOPEUP),
            OrcaLungeCollisionRouteLoc3D35A::SlopeRelaunch,
            "ASM Orca.asm loc_3D35A should route slope-up collisions to loc_3D3DE"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_SLOPEDOWN),
            OrcaLungeCollisionRouteLoc3D35A::SlopeRelaunch,
            "ASM Orca.asm loc_3D35A should route slope-down collisions to loc_3D3DE"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_HURT),
            OrcaLungeCollisionRouteLoc3D35A::Damage,
            "ASM Orca.asm loc_3D35A should route hurt collisions to loc_3D472"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_KIDABOVE),
            OrcaLungeCollisionRouteLoc3D35A::Damage,
            "ASM Orca.asm loc_3D35A should route Kid-above collisions to loc_3D472"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_KIDRIGHT),
            OrcaLungeCollisionRouteLoc3D35A::BerzerkerSideGate,
            "ASM Orca.asm loc_3D35A should route side contacts to loc_3D4DE"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(ORCA_COLID_KIDLEFT),
            OrcaLungeCollisionRouteLoc3D35A::BerzerkerSideGate,
            "ASM Orca.asm loc_3D35A should route side contacts to loc_3D4DE"
        );
        assert_eq!(
            orca_lunge_collision_dispatch_loc_3d35a(0x0030),
            OrcaLungeCollisionRouteLoc3D35A::GravityAndAcceleration,
            "ASM Orca.asm loc_3D35A should fall through unrecognised collision IDs to loc_3D3B4"
        );
    }

    #[test]
    fn orca_lunge_right_acceleration_matches_loc_3d3d2() {
        assert_eq!(
            orca_lunge_right_x_velocity_loc_3d3d2(ORCA_LUNGE_X_SPEED),
            ORCA_LUNGE_X_SPEED + ORCA_LUNGE_X_ACCEL,
            "ASM Orca.asm loc_3D3D2 should add `$200` to x_vel when x_direction is set"
        );
        assert_eq!(
            orca_lunge_right_x_velocity_loc_3d3d2(-0x0000_0100),
            0x0000_0100,
            "ASM Orca.asm loc_3D3D2 should use signed long addition, not clamp at zero"
        );
    }

    #[test]
    fn orca_lunge_frame_moves_by_updated_velocity_loc_3d35a() {
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, 80 << 16, 4);
        enemy.orca_state = OrcaState::Lunging;
        enemy.facing = Direction::Right;
        enemy.x_velocity = ORCA_LUNGE_X_SPEED;
        enemy.y_velocity = ORCA_LUNGE_Y_SPEED;
        enemy.on_ground = false;
        enemy.wall_pinned = false;

        let x_before = enemy.x_pos;
        let y_before = enemy.y_pos;
        let expected_x_velocity = ORCA_LUNGE_X_SPEED + ORCA_LUNGE_X_ACCEL;
        let expected_y_velocity = ORCA_LUNGE_Y_SPEED + ORCA_LUNGE_GRAVITY;

        step_orca(&mut enemy, None, None);

        assert_eq!(enemy.x_velocity, expected_x_velocity);
        assert_eq!(enemy.y_velocity, expected_y_velocity);
        assert_eq!(
            enemy.x_pos,
            (i64::from(x_before) + i64::from(expected_x_velocity)) as u32,
            "ASM Orca.asm loc_3D35A applies sub_3E8BA motion before the lunge loop continues"
        );
        assert_eq!(
            enemy.y_pos,
            (i64::from(y_before) + i64::from(expected_y_velocity)) as u32,
            "ASM Orca.asm loc_3D35A should move by the signed upward lunge velocity"
        );
        assert_eq!(enemy.orca_state, OrcaState::Lunging);
    }

    #[test]
    fn orca_lunge_slope_contact_relaunches_loc_3d3de() {
        let grid = grid_with_cells(8, 8, &[(5, 5, 0x50, 0)]);
        let mut enemy = test_enemy(EnemyFamily::Orca, (5 * 16 + 8) << 16, 88 << 16, 4);
        enemy.orca_state = OrcaState::Lunging;
        enemy.facing = Direction::Right;
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        enemy.on_ground = false;
        enemy.wall_pinned = true;

        step_orca(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.facing,
            Direction::Left,
            "ASM Orca.asm loc_3D3DE flips x_direction on slope contact before re-entering loc_3D320"
        );
        assert_eq!(enemy.x_velocity, -ORCA_LUNGE_X_SPEED);
        assert_eq!(enemy.y_velocity, ORCA_LUNGE_Y_SPEED);
        assert!(!enemy.wall_pinned);
        assert_eq!(enemy.orca_state, OrcaState::Lunging);
    }

    #[test]
    fn orca_lunge_wall_contact_sets_wall_pin_loc_3d40c() {
        let grid = grid_with_cells(8, 8, &[(5, 3, 0x60, 0)]);
        let mut enemy = test_enemy(EnemyFamily::Orca, 71 << 16, 56 << 16, 4);
        enemy.orca_state = OrcaState::Lunging;
        enemy.facing = Direction::Right;
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        enemy.on_ground = false;
        enemy.wall_pinned = false;

        step_orca(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.x_velocity, 0,
            "ASM Orca.asm loc_3D40C clears x_vel on wall contact"
        );
        assert!(
            enemy.wall_pinned,
            "ASM Orca.asm loc_3D40C sets `$5C(a5)` so later lunge frames suppress X acceleration"
        );
        assert_eq!(enemy.facing, Direction::Left);
        assert_eq!(enemy.orca_state, OrcaState::Lunging);
    }

    #[test]
    fn orca_lunge_ceiling_bounces_only_while_moving_up_loc_3d47a() {
        let ceiling_grid = grid_with_cells(6, 6, &[(2, 1, 0x60, 0)]);
        let mut upward = test_enemy(EnemyFamily::Orca, 40 << 16, 48 << 16, 4);
        upward.orca_state = OrcaState::Lunging;
        upward.facing = Direction::Right;
        upward.x_velocity = 0;
        upward.y_velocity = -0x0001_0000;
        upward.on_ground = false;
        upward.wall_pinned = true;

        step_orca(&mut upward, Some(&ceiling_grid), None);

        assert_eq!(
            upward.y_velocity, 0x0001_8000,
            "ASM Orca.asm loc_3D47A bounces an upward lunge downward after ceiling collision"
        );

        let zero_grid = grid_with_cells(6, 6, &[(2, 2, 0x60, 0)]);
        let mut zero = test_enemy(EnemyFamily::Orca, 40 << 16, 48 << 16, 4);
        zero.orca_state = OrcaState::Lunging;
        zero.facing = Direction::Right;
        zero.x_velocity = 0;
        zero.y_velocity = -ORCA_LUNGE_GRAVITY;
        zero.on_ground = false;
        zero.wall_pinned = true;

        step_orca(&mut zero, Some(&zero_grid), None);

        assert_eq!(
            zero.y_velocity, 0,
            "ASM Orca.asm loc_3D35A only reaches the ceiling branch for colid_ceiling, not for a neutral vertical frame"
        );
        assert_eq!(zero.orca_state, OrcaState::Lunging);
        assert!(!zero.on_ground);
    }

    #[test]
    fn orca_lunge_zero_y_velocity_does_not_land_on_floor_loc_3d35a() {
        let grid = grid_with_cells(8, 8, &[(2, 5, 0x60, 0)]);
        let mut enemy = test_enemy(EnemyFamily::Orca, 40 << 16, 80 << 16, 4);
        enemy.orca_state = OrcaState::Lunging;
        enemy.facing = Direction::Right;
        enemy.x_velocity = 0;
        enemy.y_velocity = -ORCA_LUNGE_GRAVITY;
        enemy.on_ground = false;
        enemy.wall_pinned = true;

        step_orca(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.orca_state,
            OrcaState::Lunging,
            "ASM Orca.asm loc_3D35A continues the lunge loop when the native floor probe has no downward frame"
        );
        assert_eq!(enemy.y_velocity, 0);
        assert!(!enemy.on_ground);
    }

    #[test]
    fn orca_landing_wait_matches_loc_3d44a() {
        assert_eq!(
            orca_landing_wait_loc_3d44a(ORCA_LANDING_WAIT_DBF_SEED_LOC_3D44A),
            OrcaLandingWaitLoc3D44A {
                hibernate_frames: 6,
                is_animated: true,
                animation_script: "stru_3D0A2",
                next_worker: "loc_3D21E",
            },
            "ASM Orca.asm loc_3D44A should wait six hibernated frames from d1=5 before restoring animation"
        );
        assert_eq!(
            orca_landing_wait_loc_3d44a(0).hibernate_frames,
            1,
            "ASM Orca.asm loc_3D44A DBF loop should execute once when d1 starts at zero"
        );
    }

    #[test]
    fn orca_landing_resume_matches_loc_3d466() {
        assert_eq!(
            orca_landing_resume_loc_3d466(Direction::Left),
            OrcaLandingResumeLoc3D466::Loc3E566,
            "ASM Orca.asm loc_3D466 should branch clear x_direction to loc_3E566"
        );
        assert_eq!(
            orca_landing_resume_loc_3d466(Direction::Right),
            OrcaLandingResumeLoc3D466::Loc3E3B2,
            "ASM Orca.asm loc_3D466 should branch set x_direction to loc_3E3B2"
        );
    }

    #[test]
    fn orca_fatal_fall_start_matches_loc_3d48a() {
        assert_eq!(
            orca_fatal_fall_start_loc_3d48a(),
            OrcaFatalFallStartLoc3D48A {
                y_velocity: 0x0001_8000,
                has_kid_collision: true,
                falls_through_to_landing_loop: true,
            },
            "ASM Orca.asm loc_3D48A should start the HP-depleted fall with y_vel $18000 and Kid collision enabled"
        );
    }

    #[test]
    fn orca_fatal_fall_death_animation_matches_loc_3d4bc() {
        assert_eq!(
            orca_fatal_fall_death_animation_loc_3d4bc(0x05),
            OrcaFatalFallDeathAnimationLoc3D4BC {
                next_collision_type: 0,
                x_velocity: 0,
                y_velocity: 0,
                is_moved: false,
                has_level_collision: false,
                animation_script: "stru_3D11E",
                waits_until_animation_finished: true,
                status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker { index: 5 },
                exits_enemy: true,
            },
            "ASM Orca.asm loc_3D4BC should stop motion, disable movement/level collision, play `stru_3D11E`, and exit"
        );
    }

    #[test]
    fn orca_fatal_fall_loop_matches_loc_3d496() {
        assert_eq!(ORCA_FATAL_FALL_GRAVITY_LOC_3D496, 0x0000_1800);
        assert_eq!(
            orca_fatal_fall_frame_loc_3d496(0, 0x0000_4000, 0x0001_8000, 0x05),
            OrcaFatalFallFrameLoc3D496 {
                next_collision_type: 0,
                x_velocity: 0x0000_4000,
                y_velocity: 0x0001_9800,
                is_moved: true,
                has_level_collision: true,
                hibernates_one_frame: true,
                route: OrcaFatalFallRouteLoc3D496::Loop,
            },
            "ASM Orca.asm loc_3D496 should clear collision, hibernate, add $1800 to y_vel, and loop without floor/slope contact"
        );

        for collision_type in [ORCA_COLID_FLOOR, ORCA_COLID_SLOPEUP, ORCA_COLID_SLOPEDOWN] {
            assert_eq!(
                orca_fatal_fall_frame_loc_3d496(collision_type, 0x0000_4000, 0x0001_8000, 0x05),
                OrcaFatalFallFrameLoc3D496 {
                    next_collision_type: 0,
                    x_velocity: 0,
                    y_velocity: 0,
                    is_moved: false,
                    has_level_collision: false,
                    hibernates_one_frame: true,
                    route: OrcaFatalFallRouteLoc3D496::DeathAnimation {
                        animation_script: "stru_3D11E",
                        waits_until_animation_finished: true,
                        status_action: DeactivationStatusActionLoc3C4C0::WriteRespawnMarker {
                            index: 5
                        },
                        exits_enemy: true,
                    },
                },
                "ASM Orca.asm loc_3D496 should route floor/slope contact through loc_3D4BC death animation"
            );
        }
    }

    #[test]
    fn orca_fatal_fall_collision_type_matches_loc_3d496() {
        let empty = grid_with_cells(8, 8, &[]);
        assert_eq!(
            orca_fatal_fall_collision_type_loc_3d496(terrain::probe_floor(&empty, 72, 64)),
            0,
            "ASM Orca.asm loc_3D496 keeps looping while collision_type is zero"
        );

        let flat = grid_with_cells(8, 8, &[(4, 4, 0x60, 0)]);
        assert_eq!(
            orca_fatal_fall_collision_type_loc_3d496(terrain::probe_floor(&flat, 72, 64)),
            ORCA_COLID_FLOOR,
            "ASM Orca.asm loc_3D496 routes colid_floor to loc_3D4BC"
        );

        let slope_up = grid_with_cells(8, 8, &[(4, 4, 0x40, 0)]);
        assert_eq!(
            orca_fatal_fall_collision_type_loc_3d496(terrain::probe_floor(&slope_up, 72, 71)),
            ORCA_COLID_SLOPEUP,
            "ASM Orca.asm loc_3D496 routes colid_slopeup to loc_3D4BC"
        );

        let slope_down = grid_with_cells(8, 8, &[(4, 4, 0x50, 0)]);
        assert_eq!(
            orca_fatal_fall_collision_type_loc_3d496(terrain::probe_floor(&slope_down, 72, 72)),
            ORCA_COLID_SLOPEDOWN,
            "ASM Orca.asm loc_3D496 routes colid_slopedown to loc_3D4BC"
        );
    }

    #[test]
    fn orca_ignores_distant_player() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, floor_y, 4);
        enemy.orca_state = OrcaState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 0;
        let player_x = Some((60 + 0x50) << 16); // beyond $30 range
        step_orca(&mut enemy, Some(&grid), player_x);
        assert_eq!(
            enemy.orca_state,
            OrcaState::Walking,
            "should not lunge at distant player"
        );
    }

    #[test]
    fn orca_cooldown_prevents_immediate_relunge() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, floor_y, 4);
        enemy.orca_state = OrcaState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 100; // still cooling down
        let player_x = Some((60 + 0x10) << 16);
        step_orca(&mut enemy, Some(&grid), player_x);
        assert_eq!(
            enemy.orca_state,
            OrcaState::Walking,
            "should not lunge during cooldown"
        );
        assert_eq!(enemy.wait_timer, 99);
    }

    #[test]
    fn orca_lands_waits_then_returns_to_walking_loc_3d420_3d44a_3d466() {
        let grid = grid_with_platform(17, 0, 120);
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, (17 * 16 - 30) << 16, 4);
        enemy.orca_state = OrcaState::Lunging;
        enemy.x_velocity = ORCA_LUNGE_X_SPEED;
        enemy.y_velocity = ORCA_LUNGE_Y_SPEED;
        enemy.on_ground = false;
        let mut saw_landing_wait = false;
        for _ in 0..300 {
            step_orca(&mut enemy, Some(&grid), None);
            if enemy.orca_state == OrcaState::LandingWait {
                if !saw_landing_wait {
                    assert_eq!(
                        enemy.wait_timer,
                        u16::from(
                            orca_landing_wait_loc_3d44a(ORCA_LANDING_WAIT_DBF_SEED_LOC_3D44A)
                                .hibernate_frames
                        ),
                        "ASM Orca.asm loc_3D420 should enter loc_3D44A's six-frame landing wait before loc_3D466 resumes patrol"
                    );
                }
                saw_landing_wait = true;
            }
            if enemy.orca_state == OrcaState::Walking {
                break;
            }
        }
        assert!(
            saw_landing_wait,
            "ASM Orca.asm loc_3D420 should not return directly to walking before loc_3D44A"
        );
        assert_eq!(enemy.orca_state, OrcaState::Walking);
        assert_eq!(enemy.wait_timer, ORCA_LUNGE_COOLDOWN);
        assert!(enemy.on_ground);
        assert_eq!(
            enemy.y_pos,
            (17 * 16) << 16,
            "ASM Orca.asm loc_3D420 snaps y_pos to the contacted floor before resuming patrol"
        );
    }

    #[test]
    fn orca_lunge_accelerates_x_per_frame() {
        let grid = grid_with_platform(17, 0, 120);
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, (17 * 16 - 30) << 16, 4);
        enemy.orca_state = OrcaState::Lunging;
        enemy.facing = Direction::Right;
        enemy.x_velocity = ORCA_LUNGE_X_SPEED;
        enemy.y_velocity = ORCA_LUNGE_Y_SPEED;
        enemy.on_ground = false;
        enemy.wall_pinned = false;

        let x_before = enemy.x_velocity;
        step_orca(&mut enemy, Some(&grid), None);
        // X velocity should increase by ORCA_LUNGE_X_ACCEL (facing Right)
        assert_eq!(
            enemy.x_velocity,
            x_before + ORCA_LUNGE_X_ACCEL,
            "X should accelerate during lunge"
        );
    }

    #[test]
    fn orca_wall_pinned_suppresses_x_accel() {
        let grid = grid_with_platform(17, 0, 120);
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, (17 * 16 - 20) << 16, 4);
        enemy.orca_state = OrcaState::Lunging;
        enemy.x_velocity = 0;
        enemy.y_velocity = 0x1_8000; // falling after wall bounce
        enemy.on_ground = false;
        enemy.wall_pinned = true;

        step_orca(&mut enemy, Some(&grid), None);
        // X velocity should stay 0 when wall-pinned
        assert_eq!(
            enemy.x_velocity, 0,
            "X accel should be suppressed when wall-pinned"
        );
    }

    #[test]
    fn orca_fatal_fall_frame_moves_before_gravity_loc_3d496() {
        let mut enemy = test_enemy(EnemyFamily::Orca, 64 << 16, 48 << 16, 1);
        enemy.orca_state = OrcaState::FatalFall;
        enemy.x_velocity = 0x0000_4000;
        enemy.y_velocity = 0x0001_8000;
        enemy.on_ground = false;

        let x_before = enemy.x_pos;
        let y_before = enemy.y_pos;

        step_orca(&mut enemy, None, None);

        assert_eq!(
            enemy.x_pos,
            (i64::from(x_before) + 0x0000_4000) as u32,
            "ASM Orca.asm loc_3D496 applies movement before the fatal-fall gravity increment"
        );
        assert_eq!(
            enemy.y_pos,
            (i64::from(y_before) + 0x0001_8000) as u32,
            "ASM Orca.asm loc_3D496 moves downward by the current y_vel"
        );
        assert_eq!(enemy.y_velocity, 0x0001_9800);
        assert!(!enemy.death_animation_active);
        assert_eq!(enemy.orca_state, OrcaState::FatalFall);
    }

    #[test]
    fn orca_fatal_fall_floor_probe_uses_fp_position_loc_3d496() {
        let grid = grid_with_cells(8, 8, &[(4, 4, 0x60, 0)]);
        let mut enemy = test_enemy(EnemyFamily::Orca, (4 * 16 + 8) << 16, 64 << 16, 1);
        enemy.orca_state = OrcaState::FatalFall;
        enemy.x_velocity = 0;
        enemy.y_velocity = 0;
        enemy.on_ground = false;

        step_orca(&mut enemy, Some(&grid), None);

        assert!(
            enemy.death_animation_active,
            "ASM Orca.asm loc_3D496 converts FP position to pixel coordinates before floor contact starts loc_3D4BC"
        );
        assert_eq!(enemy.wait_timer, ORCA_DEATH_ANIMATION_FRAMES);
        assert_eq!(enemy.x_velocity, 0);
        assert_eq!(enemy.y_velocity, 0);
    }

    // --- TarMonster ---

    #[test]
    fn tar_monster_cycles_through_states() {
        let mut enemy = test_enemy(EnemyFamily::TarMonster, 60 << 16, 200 << 16, 4);
        let mut projs = Vec::new();
        enemy.tar_monster_state = TarMonsterState::Submerged;
        enemy.wait_timer = 0;

        // Submerged → Emerging
        step_tar_monster(&mut enemy, &mut projs);
        assert_eq!(enemy.tar_monster_state, TarMonsterState::Emerging);
        assert_eq!(enemy.wait_timer, TAR_MONSTER_EMERGE_FRAMES);

        // Wait through emerge animation
        for _ in 0..TAR_MONSTER_EMERGE_FRAMES {
            step_tar_monster(&mut enemy, &mut projs);
        }
        assert_eq!(enemy.wait_timer, 0);
        step_tar_monster(&mut enemy, &mut projs);
        assert_eq!(enemy.tar_monster_state, TarMonsterState::Visible);
        assert_eq!(enemy.wait_timer, TAR_MONSTER_VISIBLE_FRAMES);

        // Wait through visible period
        for _ in 0..TAR_MONSTER_VISIBLE_FRAMES {
            step_tar_monster(&mut enemy, &mut projs);
        }
        step_tar_monster(&mut enemy, &mut projs);
        assert_eq!(enemy.tar_monster_state, TarMonsterState::Attacking);
        assert_eq!(enemy.wait_timer, TAR_MONSTER_ATTACK_FRAMES);

        // During attack, a glob shot fires at frame 8
        projs.clear();
        for _ in 0..TAR_MONSTER_ATTACK_FRAMES {
            step_tar_monster(&mut enemy, &mut projs);
        }
        assert_eq!(projs.len(), 1, "glob shot should fire during attack");
        assert_ne!(projs[0].x_velocity, 0, "glob shot has horizontal velocity");

        step_tar_monster(&mut enemy, &mut projs);
        assert_eq!(enemy.tar_monster_state, TarMonsterState::Submerging);

        // Wait through submerge animation
        for _ in 0..TAR_MONSTER_SUBMERGE_FRAMES {
            step_tar_monster(&mut enemy, &mut projs);
        }
        step_tar_monster(&mut enemy, &mut projs);
        assert_eq!(enemy.tar_monster_state, TarMonsterState::Submerged);
        assert_eq!(enemy.wait_timer, TAR_MONSTER_SUBMERGED_FRAMES);
    }

    #[test]
    fn tar_monster_is_invulnerable_when_submerged() {
        let mut enemy = test_enemy(EnemyFamily::TarMonster, 60 << 16, 200 << 16, 4);
        enemy.tar_monster_state = TarMonsterState::Submerged;
        assert!(!enemy.is_vulnerable());

        enemy.tar_monster_state = TarMonsterState::Emerging;
        assert!(!enemy.is_vulnerable());

        enemy.tar_monster_state = TarMonsterState::Visible;
        assert!(enemy.is_vulnerable());

        enemy.tar_monster_state = TarMonsterState::Submerging;
        assert!(!enemy.is_vulnerable());
    }

    #[test]
    fn tar_monster_retaliation_spawns_five_projectiles() {
        let mut projectiles = Vec::new();
        spawn_tar_monster_retaliation(&mut projectiles, 100 << 16, 200 << 16);
        assert_eq!(projectiles.len(), 5);
        for p in &projectiles {
            assert!(p.y_velocity < 0, "all tar blobs should launch upward");
            assert!(!p.friendly);
            assert!(p.gravity > 0, "tar blobs have gravity");
        }
        // Verify spread: some go left, some go right
        let left_count = projectiles.iter().filter(|p| p.x_velocity < 0).count();
        let right_count = projectiles.iter().filter(|p| p.x_velocity > 0).count();
        assert!(left_count >= 2, "at least 2 blobs go left");
        assert!(right_count >= 2, "at least 2 blobs go right");
    }

    #[test]
    fn fire_demon_death_spawns_trail() {
        let mut projectiles = Vec::new();
        spawn_fire_demon_death_trail(&mut projectiles, 100 << 16, 200 << 16);
        assert_eq!(
            projectiles.len(),
            1,
            "ASM Fire_Demon loc_3A63C allocates one fire-trail object"
        );
        let trail = projectiles[0];
        assert_eq!(trail.x_pos, 100 << 16);
        assert_eq!(
            trail.y_pos,
            208 << 16,
            "ASM loc_3A63C: y_pos += $F, then align down with $FFF0"
        );
        assert_eq!(trail.x_velocity, 0);
        assert_eq!(trail.y_velocity, 0);
        assert_eq!(trail.gravity, 0);
        assert_eq!(trail.lifetime, 0x96);
        assert_eq!(trail.kind, EnemyProjectileKind::FireDemonDeathTrail);
        assert!(!trail.friendly);
    }

    // --- Drips ---

    #[test]
    fn drips_drops_projectile_on_timer_expiry() {
        let mut enemy = test_enemy(EnemyFamily::Drips, 60 << 16, 20 << 16, 1);
        enemy.wait_timer = 0;
        let mut projectiles = Vec::new();
        let mut expected_rng = RomRandomState::default();
        let expected_wait_timer =
            drips_wait_timer_with_rng_loc_3340e(&mut expected_rng, 0x1234_0000);
        for _ in 0..=DRIPS_PRE_DROP_ANIMATION_FRAMES_LOC_33424 {
            step_drips(&mut enemy, &mut projectiles, None);
        }
        assert_eq!(projectiles.len(), 1, "should drop one projectile");
        assert_eq!(projectiles[0].x_velocity, 0, "drops straight down");
        assert_eq!(
            projectiles[0].y_velocity, DRIPS_DROP_INITIAL_Y_SPEED,
            "ASM loc_33424 clears the drop child's y velocity before the fall begins"
        );
        assert_eq!(
            projectiles[0].gravity, DRIPS_DROP_GRAVITY,
            "ASM loc_3345C accelerates the drop by `$2000` each frame"
        );
        assert_eq!(
            projectiles[0].lifetime,
            u16::MAX,
            "ASM loc_3345C has no short DBF lifetime; collision ends the falling drop"
        );
        assert_eq!(
            projectiles[0].kind,
            EnemyProjectileKind::DripsDrop,
            "ASM Drips.asm loc_33424 reuses the dedicated falling child object, so native should not leave the drop as a generic projectile"
        );
        assert!(
            projectiles[0].terrain_kill,
            "native drop should expire on terrain contact instead of only a short timer"
        );
        assert!(!projectiles[0].friendly);
        assert_eq!(
            projectiles[0].owner,
            Some(EnemyProjectileOwner {
                segment_index: enemy.segment_index,
                layout_index: enemy.layout_index,
                token: 0,
            }),
            "ASM Drips.asm loc_33424 reuses the parent-owned falling child until loc_33488 returns control to the parent",
        );
        assert_eq!(
            enemy.wait_timer, DRIPS_DROP_IN_FLIGHT_TIMER,
            "ASM Drips.asm loc_33424/loc_3345C should keep the parent dormant while the falling child is active",
        );
        assert_ne!(
            enemy.wait_timer, expected_wait_timer,
            "ASM Drips.asm loc_3350E reseeds the dormant delay only after the splash path",
        );
    }

    #[test]
    fn drips_pool_table_creates_crawler_only_after_two_splashes_loc_33488() {
        // ASM Drips.asm loc_33488 walks `off_33520`: the first two splashes
        // write pool sprite entries, then the negative sentinel path resets
        // through loc_33506 and may allocate the loc_335D8 crawler.
        let mut enemy = test_enemy(EnemyFamily::Drips, 60 << 16, 20 << 16, 1);
        enemy.drips_attached = true;
        enemy.wait_timer = 0;
        let mut projectiles = Vec::new();

        for _ in 0..3 {
            for _ in 0..=DRIPS_PRE_DROP_ANIMATION_FRAMES_LOC_33424 {
                step_drips(&mut enemy, &mut projectiles, None);
            }
            enemy.wait_timer = 0;
        }

        assert_eq!(projectiles.len(), 3);
        assert!(
            !projectiles[0].drips_spawn_crawler_on_splash,
            "ASM off_33520 is consumed on splash, not when loc_33424 spawns the falling child"
        );
        assert!(
            !projectiles[1].drips_spawn_crawler_on_splash,
            "ASM off_33520 is consumed on splash, so queued falling children do not predeclare pool growth"
        );
        assert!(
            !projectiles[2].drips_spawn_crawler_on_splash,
            "ASM off_33520 sentinel is reached by loc_33488 after terrain collision, not by the drop spawn path"
        );
        assert_eq!(
            enemy.drips_surface_table_index, 0,
            "ASM loc_33424 should not advance the off_33520 cursor before the child splashes"
        );
    }

    #[test]
    fn drips_rise_to_ceiling_snaps_position_and_attaches_loc_333f6() {
        // ASM Enemy16_Drip_Init (Drips.asm:35-64): the slime spawns with
        // `y_vel=-$10000`, then loops via loc_333F6 until colliding with a
        // ceiling cell. Native `step_drips` runs this rise phase against the
        // block grid until `probe_ceiling` reports a hit, snaps the position,
        // and flips `drips_attached` so the existing wait/drop cycle takes
        // over.
        // Use the shared floor-row helper to build a small grid, but with the
        // solid row at row 1 acting as a CEILING above the Drips spawn point.
        let grid = grid_with_floor_at_row(1);
        // The helper builds a 120×56 grid (6*20 × 4*14). Row 1 is solid.
        // Place the Drips at x=64, y=120 (well below the ceiling).
        let mut enemy = test_enemy(EnemyFamily::Drips, 0x0040_0000, 0x0078_0000, 1);
        enemy.y_velocity = DRIPS_CEILING_ATTACH_RISE_VELOCITY;
        enemy.drips_attached = false;
        enemy.wait_timer = 0;

        let mut projectiles = Vec::new();
        let mut frames = 0;
        while !enemy.drips_attached && frames < 200 {
            step_drips(&mut enemy, &mut projectiles, Some(&grid));
            frames += 1;
        }

        assert!(
            enemy.drips_attached,
            "ASM loc_333F6 should converge inside the safety frame budget",
        );
        assert_eq!(
            enemy.y_velocity, 0,
            "ASM loc_333F6 clears y_vel on ceiling contact",
        );
        assert_eq!(
            enemy.y_pos, 0x0022_0000,
            "ASM GfxObjects_Collision loc_1A6C writes `y = contact_tile_top - y_off + $10` with the rising box top 2px above y_pos; issue-260 oracle shows Level 01 attach at 530 = row-32 tile top 512 + $12, so the row-1 ceiling (tile top 16) leaves y_pos at 34",
        );
        assert!(
            projectiles.is_empty(),
            "Drips must reach the ceiling before its first drop spawns",
        );
        assert_eq!(
            enemy.wait_timer, DRIPS_INITIAL_SEED_DELAY_2_TIMER_LOC_3340E,
            "ASM Drips.asm loc_333F6/loc_3340E reaches the RNG draw after the oracle #260 two-frame coroutine bridge from ceiling contact",
        );

        step_drips(&mut enemy, &mut projectiles, Some(&grid));
        step_drips(&mut enemy, &mut projectiles, Some(&grid));
        assert_eq!(
            enemy.wait_timer, DRIPS_INITIAL_SEED_PENDING_TIMER_LOC_3340E,
            "after the loc_333F6 bridge, native defers the loc_3340E RNG draw to the seed ordering pass",
        );

        let mut expected_rng = RomRandomState::default();
        let expected_wait_timer =
            drips_wait_timer_with_rng_loc_3340e(&mut expected_rng, 0x0040_0000);
        let mut rng = RomRandomState::default();
        let mut enemies = [enemy];
        seed_pending_drips_initial_waits_loc_3340e(&mut enemies, Some(0x0040_0000), &mut rng);
        assert_eq!(
            enemies[0].wait_timer, expected_wait_timer,
            "ASM loc_3340E seeds the dormant timer when the ceiling attach completes",
        );
    }

    #[test]
    fn drips_waits_between_drops() {
        let mut enemy = test_enemy(EnemyFamily::Drips, 60 << 16, 20 << 16, 1);
        enemy.wait_timer = 5;
        let mut projectiles = Vec::new();
        step_drips(&mut enemy, &mut projectiles, None);
        assert!(projectiles.is_empty(), "should not drop while timer active");
        assert_eq!(enemy.wait_timer, 4);
    }

    #[test]
    fn drips_dispatch_pauses_outside_object_check_in_range_loc_333f6() {
        // ASM Drips.asm loc_333F6/loc_3345C calls `Object_CheckInRange` before
        // the attach and falling-child loops. The object is outside
        // Object_CheckInRange_NormalRange X high edge (`camera+$2E4`) here, so
        // the native dispatcher must not advance the wait/drop cycle.
        let mut enemies = [test_enemy(EnemyFamily::Drips, 0x02E5 << 16, 20 << 16, 1)];
        enemies[0].drips_attached = true;
        enemies[0].wait_timer = 5;
        let mut projectiles = Vec::new();
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();

        step_active_enemies_with_rng(
            &mut enemies,
            None,
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            None,
            None,
            None,
            PlayerForm::TheKid,
            0,
            false,
            Some(&mut projectiles),
            &mut rng,
            &mut drilled_blocks,
            None,
        );

        assert_eq!(
            enemies[0].wait_timer, 5,
            "ASM Object_CheckInRange should stop off-window Drips from consuming its dormant timer",
        );
        assert!(
            projectiles.is_empty(),
            "off-window Drips must not spawn falling children early",
        );

        step_active_enemies_with_rng(
            &mut enemies,
            None,
            Some(0),
            Some(0),
            Some(1),
            Some(0),
            None,
            None,
            None,
            PlayerForm::TheKid,
            0,
            false,
            Some(&mut projectiles),
            &mut rng,
            &mut drilled_blocks,
            None,
        );

        assert_eq!(
            enemies[0].wait_timer, 4,
            "once inside the ROM range window, Drips should resume the loc_3340E wait cycle",
        );
    }

    #[test]
    fn drips_drop_uses_dedicated_child_slot_not_generic_projectile_cap_loc_33424() {
        let mut enemy = test_enemy(EnemyFamily::Drips, 60 << 16, 20 << 16, 1);
        enemy.wait_timer = 0;
        let mut projectiles = vec![EnemyProjectile::default(); MAX_PROJECTILES];

        step_drips(&mut enemy, &mut projectiles, None);

        assert_eq!(
            projectiles.len(),
            MAX_PROJECTILES,
            "ASM Drips.asm loc_33424 starts stru_33F90 before exposing the falling child",
        );

        for _ in 0..DRIPS_PRE_DROP_ANIMATION_FRAMES_LOC_33424 {
            step_drips(&mut enemy, &mut projectiles, None);
        }

        assert_eq!(
            projectiles.len(),
            MAX_PROJECTILES + 1,
            "ASM Drips.asm loc_33424 uses Drips' preallocated falling child after stru_33F90, so unrelated generic projectiles must not block the drop"
        );
        assert_eq!(
            projectiles.last().map(|projectile| projectile.kind),
            Some(EnemyProjectileKind::DripsDrop),
            "ASM Drips.asm loc_3345C should leave the native Drips child distinguishable from generic enemy projectiles"
        );
    }

    // --- SpinningTwins ---

    #[test]
    fn spinning_twins_orbits_indefinitely_without_player_contact() {
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 100 << 16, 50 << 16, 1);
        assert_eq!(enemy.spinning_twins_phase, SpinningTwinsPhase::Orbiting);

        // ASM Spinning_Twins.asm loc_3B904 keeps the parent object at the
        // spawn X (`d2`) and only applies `word_3B9A0` to the linked child GFX.
        let initial_x = enemy.x_pos;
        step_spinning_twins(&mut enemy, None, None, None);
        assert_eq!(enemy.spinning_twins_phase, SpinningTwinsPhase::Orbiting);
        assert_eq!(enemy.x_pos, initial_x, "orbit parent should stay anchored");

        // Step many more frames — should still orbit without player contact
        for _ in 0..200 {
            step_spinning_twins(&mut enemy, None, None, None);
        }
        assert_eq!(
            enemy.spinning_twins_phase,
            SpinningTwinsPhase::Orbiting,
            "should orbit indefinitely without player contact (ASM: no timer fallback)"
        );
    }

    #[test]
    fn spinning_twins_orbit_loc_3b904_advances_only_on_animation_timer_one() {
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 100 << 16, 50 << 16, 1);

        for frame in 1..=5 {
            step_spinning_twins(&mut enemy, None, None, None);
            assert_eq!(
                enemy.spinning_twins_orbit_index, 0,
                "ASM Spinning_Twins.asm loc_3B904 should not advance d1 before animation_timer(a3) reaches 1; frame {frame}"
            );
        }

        step_spinning_twins(&mut enemy, None, None, None);

        assert_eq!(
            enemy.spinning_twins_orbit_index, 1,
            "ASM Spinning_Twins.asm loc_3B904 increments d1 only after `cmpi.w #1,animation_timer(a3)` matches"
        );
        assert_eq!(
            enemy.x_pos,
            100 << 16,
            "ASM Spinning_Twins.asm loc_3B904 writes the offset X to linked child GFX `$1A(a1)`, not parent `x_pos(a3)`"
        );
    }

    #[test]
    fn spinning_twins_orbit_offset_table_matches_word_3b9a0() {
        assert_eq!(
            SPINNING_TWINS_ORBIT_OFFSETS,
            [-0x10, -0x0A, 0x05, 0x11, 0x10, 0x0A, -0x05, -0x11],
            "ASM Spinning_Twins.asm word_3B9A0 is the signed child-GFX X orbit table"
        );
    }

    #[test]
    fn spinning_twins_orbit_collision_route_loc_3b904_drops_only_for_rom_contact_words() {
        assert_eq!(
            spinning_twins_orbit_collision_route_loc_3b904(-1),
            SpinningTwinsOrbitCollisionRouteLoc3B904::DropSplit,
            "ASM Spinning_Twins.asm loc_3B904 `bmi.s loc_3B93C` drops on signed-negative collision_type"
        );
        assert_eq!(
            spinning_twins_orbit_collision_route_loc_3b904(SPINNING_TWINS_COLID_KIDABOVE as i16),
            SpinningTwinsOrbitCollisionRouteLoc3B904::DropSplit,
            "ASM Spinning_Twins.asm loc_3B904 drops on `colid_kidabove` `$2C`"
        );
        assert_eq!(
            spinning_twins_orbit_collision_route_loc_3b904(SPINNING_TWINS_COLID_HURT as i16),
            SpinningTwinsOrbitCollisionRouteLoc3B904::DropSplit,
            "ASM Spinning_Twins.asm loc_3B904 drops on `colid_hurt` `$1C`"
        );
        for collision_type in [
            0,
            SPINNING_TWINS_COLID_RIGHTWALL as i16,
            SPINNING_TWINS_COLID_KIDLEFT as i16,
            SPINNING_TWINS_COLID_KIDRIGHT as i16,
        ] {
            assert_eq!(
                spinning_twins_orbit_collision_route_loc_3b904(collision_type),
                SpinningTwinsOrbitCollisionRouteLoc3B904::ContinueOrbit,
                "ASM Spinning_Twins.asm loc_3B904 should keep orbiting for collision_type {collision_type:#06X}"
            );
        }
    }

    #[test]
    fn spinning_twins_drops_on_player_collision() {
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 100 << 16, 50 << 16, 1);
        assert_eq!(enemy.spinning_twins_phase, SpinningTwinsPhase::Orbiting);

        // Place player overlapping the orbiting twins
        let player_x = Some(100u32 << 16);
        let player_y = Some(50u32 << 16);
        step_spinning_twins(&mut enemy, None, player_x, player_y);

        assert_eq!(enemy.spinning_twins_phase, SpinningTwinsPhase::Walking);
        assert!(!enemy.on_ground, "should be airborne after drop");
        assert!(enemy.y_velocity < 0, "should have upward launch velocity");
    }

    #[test]
    fn spinning_twins_live_contact_box_uses_strict_native_edges() {
        let cases = [
            (84, 50, SpinningTwinsPhase::Orbiting, "x exactly 16px away"),
            (85, 50, SpinningTwinsPhase::Walking, "x 15px away"),
            (100, 26, SpinningTwinsPhase::Orbiting, "y exactly 24px away"),
            (100, 27, SpinningTwinsPhase::Walking, "y 23px away"),
            (
                85,
                150,
                SpinningTwinsPhase::Orbiting,
                "x inside but y far outside",
            ),
            (
                0,
                50,
                SpinningTwinsPhase::Orbiting,
                "y inside but x far outside",
            ),
            (
                100,
                30,
                SpinningTwinsPhase::Walking,
                "y inside the +8 vertical extension",
            ),
            (
                100,
                150,
                SpinningTwinsPhase::Orbiting,
                "y outside the non-scaled vertical extension",
            ),
        ];

        for (player_x_px, player_y_px, expected_phase, label) in cases {
            let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 100 << 16, 50 << 16, 1);

            step_spinning_twins(
                &mut enemy,
                None,
                Some(player_x_px << 16),
                Some(player_y_px << 16),
            );

            assert_eq!(
                enemy.spinning_twins_phase, expected_phase,
                "ASM Spinning_Twins.asm loc_3B904 native contact approximation should classify {label}"
            );
        }
    }

    #[test]
    fn spinning_twins_runtime_drop_materializes_two_children_and_visual_loc_3b93c() {
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 100 << 16, 50 << 16, 1);
        enemy.facing = Direction::Right;
        let mut enemies = vec![enemy];
        let mut drilled_blocks = Vec::new();

        step_active_enemies(
            &mut enemies,
            None,
            Some(100 << 16),
            Some(50 << 16),
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut drilled_blocks,
        );
        materialize_spinning_twins_drop_splits(&mut enemies);

        assert_eq!(
            enemies.len(),
            3,
            "ASM Spinning_Twins.asm loc_3B93C allocates two loc_3BA74 walking children plus one loc_3B9B0 falling visual"
        );
        assert_eq!(enemies[0].spinning_twins_phase, SpinningTwinsPhase::Walking);
        assert_eq!(enemies[1].spinning_twins_phase, SpinningTwinsPhase::Walking);
        assert_eq!(
            enemies[2].spinning_twins_phase,
            SpinningTwinsPhase::FallingVisual
        );
        let split_source_x = 100 << 16;
        assert_eq!(enemies[0].x_pos, split_source_x - (0x20 << 16));
        assert_eq!(enemies[1].x_pos, split_source_x + (0x20 << 16));
        assert_eq!(
            enemies[2].x_pos,
            split_source_x - (0x13 << 16),
            "ASM loc_3B9B0 subtracts `$13` from the parent X for the falling visual"
        );
        assert_eq!(enemies[0].x_velocity, SPINNING_TWINS_DROP_X_VEL_LEFT);
        assert_eq!(enemies[1].x_velocity, SPINNING_TWINS_DROP_X_VEL_RIGHT);
        assert_eq!(enemies[0].y_velocity, SPINNING_TWINS_DROP_Y_VEL);
        assert_eq!(enemies[1].y_velocity, SPINNING_TWINS_DROP_Y_VEL);
        assert!(!enemies[2].is_vulnerable());
    }

    #[test]
    fn spinning_twins_drop_materialization_skips_only_non_pending_sources_loc_3b93c() {
        let mut wrong_family = test_enemy(EnemyFamily::FireDemon, 0x0080_0000, 0x0040_0000, 1);
        wrong_family.wait_timer = SPINNING_TWINS_DROP_SPLIT_PENDING_LOC_3B93C;

        let mut wrong_phase = test_enemy(EnemyFamily::SpinningTwins, 0x00A0_0000, 0x0040_0000, 1);
        wrong_phase.spinning_twins_phase = SpinningTwinsPhase::Orbiting;
        wrong_phase.wait_timer = SPINNING_TWINS_DROP_SPLIT_PENDING_LOC_3B93C;

        let mut first_pending = test_enemy(EnemyFamily::SpinningTwins, 0x0100_0000, 0x0080_0000, 1);
        first_pending.spinning_twins_phase = SpinningTwinsPhase::Walking;
        first_pending.wait_timer = SPINNING_TWINS_DROP_SPLIT_PENDING_LOC_3B93C;

        let mut second_pending =
            test_enemy(EnemyFamily::SpinningTwins, 0x0200_0000, 0x0090_0000, 1);
        second_pending.spinning_twins_phase = SpinningTwinsPhase::Walking;
        second_pending.wait_timer = SPINNING_TWINS_DROP_SPLIT_PENDING_LOC_3B93C;

        let mut enemies = vec![wrong_family, wrong_phase, first_pending, second_pending];

        materialize_spinning_twins_drop_splits(&mut enemies);

        assert_eq!(
            enemies.len(),
            8,
            "ASM Spinning_Twins.asm loc_3B93C should materialize only Walking SpinningTwins with the pending split marker"
        );
        assert_eq!(enemies[0], wrong_family);
        assert_eq!(enemies[1], wrong_phase);
        assert_eq!(enemies[2].x_pos, 0x00E0_0000);
        assert_eq!(enemies[3].x_pos, 0x0120_0000);
        assert_eq!(enemies[4].x_pos, 0x00ED_0000);
        assert_eq!(
            enemies[4].spinning_twins_phase,
            SpinningTwinsPhase::FallingVisual
        );
        assert_eq!(
            enemies[5].x_pos, 0x01E0_0000,
            "the idx advance must skip the first source's inserted children before processing the next pending source"
        );
        assert_eq!(enemies[6].x_pos, 0x0220_0000);
        assert_eq!(enemies[7].x_pos, 0x01ED_0000);
        assert_eq!(
            enemies[7].spinning_twins_phase,
            SpinningTwinsPhase::FallingVisual
        );
    }

    #[test]
    fn spinning_twins_drop_transition_loc_3b93c_splits_orbit_object() {
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 100 << 16, 50 << 16, 1);
        enemy.facing = Direction::Right;

        let transition = spinning_twins_drop_transition_loc_3b93c(&mut enemy);

        assert_eq!(enemy.spinning_twins_phase, SpinningTwinsPhase::Walking);
        assert_eq!(enemy.x_velocity, SPINNING_TWINS_DROP_X_VEL_RIGHT);
        assert_eq!(enemy.y_velocity, SPINNING_TWINS_DROP_Y_VEL);
        assert!(!enemy.on_ground);
        assert_eq!(
            transition.split_child_count, 2,
            "ASM loc_3B93C allocates two walking twin child objects"
        );
        assert!(
            transition.orbit_child_hidden,
            "ASM loc_3B93C clears `$13` on the orbit child GFX object"
        );
        assert!(
            transition.falling_visual_spawned,
            "ASM loc_3B93C allocates the falling visual worker"
        );
        assert!(
            transition.original_deleted_after_animation,
            "ASM loc_3B93C deletes the original orbit object after the transition animation"
        );
    }

    #[test]
    fn spinning_twins_first_walking_child_init_loc_3bb0c_launches_left() {
        let child = spinning_twins_first_walking_child_init_loc_3bb0c(0x0100_0000, 0x0080_0000);

        assert_eq!(
            child.x_pos, 0x00E0_0000,
            "ASM loc_3BB0C is reached by the first child after the `$20` left offset"
        );
        assert_eq!(child.y_pos, 0x0080_0000);
        assert!(
            child.x_direction_flipped,
            "ASM loc_3BB0C tests the set x_direction flag for the first child"
        );
        assert_eq!(
            child.x_velocity, SPINNING_TWINS_DROP_X_VEL_LEFT,
            "ASM loc_3BB0C assigns `$FFFC8000` when x_direction is set"
        );
        assert_eq!(
            child.y_velocity, SPINNING_TWINS_DROP_Y_VEL,
            "ASM loc_3BB24 assigns `-$10000` after either X launch branch"
        );
    }

    #[test]
    fn spinning_twins_second_walking_child_init_loc_3bb06_offsets_right() {
        let child = spinning_twins_second_walking_child_init_loc_3bb06(0x0100_0000, 0x0080_0000);

        assert_eq!(
            child.x_pos, 0x0120_0000,
            "ASM loc_3BB06 adds `$20` to the copied parent X position"
        );
        assert_eq!(
            child.y_pos, 0x0080_0000,
            "ASM loc_3BB06 leaves the copied parent Y position unchanged"
        );
        assert!(
            !child.x_direction_flipped,
            "ASM loc_3BB06 leaves x_direction clear for the second child"
        );
        assert_eq!(
            child.x_velocity, SPINNING_TWINS_DROP_X_VEL_RIGHT,
            "ASM loc_3BB1C assigns `$38000` when x_direction is clear"
        );
        assert_eq!(
            child.y_velocity, SPINNING_TWINS_DROP_Y_VEL,
            "ASM loc_3BB06 shares the walking-child upward launch"
        );
    }

    #[test]
    fn spinning_twins_walking_child_gravity_loc_3bb2c_stops_when_y_velocity_zero() {
        assert_eq!(
            spinning_twins_walking_child_y_velocity_loc_3bb2c(SPINNING_TWINS_DROP_Y_VEL),
            -0x0000_D800,
            "ASM loc_3BB2C adds `$2800` to non-zero y_vel before the X-drag block"
        );
        assert_eq!(
            spinning_twins_walking_child_y_velocity_loc_3bb2c(0),
            0,
            "ASM loc_3BB2C skips gravity when y_vel is zero"
        );
    }

    #[test]
    fn spinning_twins_walking_child_left_drag_loc_3bb42_eases_negative_x_velocity() {
        assert_eq!(
            spinning_twins_walking_child_left_x_velocity_loc_3bb42(SPINNING_TWINS_DROP_X_VEL_LEFT),
            -0x0003_6E00,
            "ASM loc_3BB42 adds `$1200` when x_direction is set"
        );
    }

    #[test]
    fn spinning_twins_walking_child_right_drag_loc_3bb52_eases_positive_x_velocity() {
        assert_eq!(
            spinning_twins_walking_child_right_x_velocity_loc_3bb52(
                SPINNING_TWINS_DROP_X_VEL_RIGHT
            ),
            0x0003_6E00,
            "ASM loc_3BB52 subtracts `$1200` when x_direction is clear"
        );
    }

    #[test]
    fn spinning_twins_live_walking_child_applies_x_motion_and_drag_loc_3bb2c_3bb52() {
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 100 << 16, 50 << 16, 1);
        enemy.spinning_twins_phase = SpinningTwinsPhase::Walking;
        enemy.on_ground = false;
        enemy.x_velocity = SPINNING_TWINS_DROP_X_VEL_RIGHT;
        enemy.y_velocity = SPINNING_TWINS_DROP_Y_VEL;
        let start_x = enemy.x_pos;
        let start_y = enemy.y_pos;

        step_spinning_twins(&mut enemy, None, None, None);

        assert_eq!(
            enemy.x_pos,
            (i64::from(start_x) + i64::from(SPINNING_TWINS_DROP_X_VEL_RIGHT)) as u32,
            "ASM Spinning_Twins.asm loc_3BB2C runs sub_3C3CE before applying the x-drag block"
        );
        assert_eq!(
            enemy.y_pos,
            (i64::from(start_y) + i64::from(SPINNING_TWINS_DROP_Y_VEL)) as u32
        );
        assert_eq!(
            enemy.x_velocity,
            spinning_twins_walking_child_right_x_velocity_loc_3bb52(
                SPINNING_TWINS_DROP_X_VEL_RIGHT
            )
        );
        assert_eq!(
            enemy.y_velocity,
            spinning_twins_walking_child_y_velocity_loc_3bb2c(SPINNING_TWINS_DROP_Y_VEL)
        );
    }

    #[test]
    fn spinning_twins_live_walking_child_applies_left_x_motion_and_drag_loc_3bb2c_3bb42() {
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 100 << 16, 50 << 16, 1);
        enemy.spinning_twins_phase = SpinningTwinsPhase::Walking;
        enemy.on_ground = false;
        enemy.x_velocity = SPINNING_TWINS_DROP_X_VEL_LEFT;
        enemy.y_velocity = SPINNING_TWINS_DROP_Y_VEL;
        let start_x = enemy.x_pos;
        let start_y = enemy.y_pos;

        step_spinning_twins(&mut enemy, None, None, None);

        assert_eq!(
            enemy.x_pos,
            (i64::from(start_x) + i64::from(SPINNING_TWINS_DROP_X_VEL_LEFT)) as u32,
            "ASM Spinning_Twins.asm loc_3BB2C moves by the current negative x_vel before loc_3BB42 drag"
        );
        assert_eq!(
            enemy.y_pos,
            (i64::from(start_y) + i64::from(SPINNING_TWINS_DROP_Y_VEL)) as u32
        );
        assert_eq!(
            enemy.x_velocity,
            spinning_twins_walking_child_left_x_velocity_loc_3bb42(SPINNING_TWINS_DROP_X_VEL_LEFT)
        );
        assert_eq!(
            enemy.y_velocity,
            spinning_twins_walking_child_y_velocity_loc_3bb2c(SPINNING_TWINS_DROP_Y_VEL)
        );
    }

    #[test]
    fn spinning_twins_live_airborne_child_floor_probe_uses_moved_pixel_position_loc_3bb6e() {
        let grid = grid_with_solid_cells(8, 5, &[(2, 3)]);
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 27 << 16, 47 << 16, 1);
        enemy.spinning_twins_phase = SpinningTwinsPhase::Walking;
        enemy.on_ground = false;
        enemy.x_velocity = 0x0005_0000;
        enemy.y_velocity = 0;

        step_spinning_twins(&mut enemy, Some(&grid), None, None);

        assert_eq!(
            enemy.x_pos,
            32 << 16,
            "ASM Spinning_Twins.asm loc_3BB2C/sub_3C3CE applies X motion before floor collision checks"
        );
        assert_eq!(
            enemy.y_pos,
            48 << 16,
            "native floor probe should use the moved integer Y and snap back to the solid cell top"
        );
        assert!(enemy.on_ground);
        assert_eq!(enemy.y_velocity, 0);
    }

    #[test]
    fn spinning_twins_walking_velocity_gate_loc_3bb5a_settles_only_inside_asymmetric_band() {
        assert_eq!(
            spinning_twins_walking_velocity_gate_loc_3bb5a(0x0000_3001),
            SpinningTwinsWalkingVelocityGateLoc3BB5A::CollisionDispatch,
            "ASM loc_3BB5A keeps x_vel greater than `$3000` in the collision-dispatch path"
        );
        assert_eq!(
            spinning_twins_walking_velocity_gate_loc_3bb5a(-0x0000_3000),
            SpinningTwinsWalkingVelocityGateLoc3BB5A::CollisionDispatch,
            "ASM loc_3BB5A keeps x_vel equal to `-$3000` in the collision-dispatch path"
        );
        assert_eq!(
            spinning_twins_walking_velocity_gate_loc_3bb5a(0x0000_3000),
            SpinningTwinsWalkingVelocityGateLoc3BB5A::Settle,
            "ASM loc_3BB5A selects the settle path when x_vel equals `$3000`"
        );
        assert_eq!(
            spinning_twins_walking_velocity_gate_loc_3bb5a(-0x0000_2FFF),
            SpinningTwinsWalkingVelocityGateLoc3BB5A::Settle,
            "ASM loc_3BB5A selects the settle path once negative x_vel is above `-$3000`"
        );
    }

    #[test]
    fn spinning_twins_walking_collision_dispatch_loc_3bb6e_clears_and_routes_contacts() {
        assert_eq!(
            spinning_twins_walking_collision_dispatch_loc_3bb6e(SPINNING_TWINS_COLID_FLOOR),
            SpinningTwinsWalkingCollisionDispatchLoc3BB6E {
                next_collision_type: 0,
                route: SpinningTwinsWalkingCollisionRouteLoc3BB6E::FloorLanding
            },
            "ASM loc_3BB6E sends colid_floor to the landing path after reading d4"
        );
        assert_eq!(
            spinning_twins_walking_collision_dispatch_loc_3bb6e(SPINNING_TWINS_COLID_RIGHTWALL),
            SpinningTwinsWalkingCollisionDispatchLoc3BB6E {
                next_collision_type: 0,
                route: SpinningTwinsWalkingCollisionRouteLoc3BB6E::WallSettle
            },
            "ASM loc_3BB6E sends colid_rightwall to the settle path"
        );
        assert_eq!(
            spinning_twins_walking_collision_dispatch_loc_3bb6e(SPINNING_TWINS_COLID_LEFTWALL),
            SpinningTwinsWalkingCollisionDispatchLoc3BB6E {
                next_collision_type: 0,
                route: SpinningTwinsWalkingCollisionRouteLoc3BB6E::WallSettle
            },
            "ASM loc_3BB6E sends colid_leftwall to the same settle path"
        );
        assert_eq!(
            spinning_twins_walking_collision_dispatch_loc_3bb6e(0x001C),
            SpinningTwinsWalkingCollisionDispatchLoc3BB6E {
                next_collision_type: 0,
                route: SpinningTwinsWalkingCollisionRouteLoc3BB6E::ContinueAirborne
            },
            "ASM loc_3BB6E clears unmatched positive collision words and loops"
        );
    }

    #[test]
    fn spinning_twins_floor_landing_loc_3bb8a_clears_collision_and_y_velocity() {
        assert_eq!(
            spinning_twins_floor_landing_loc_3bb8a(
                SPINNING_TWINS_COLID_FLOOR,
                SPINNING_TWINS_DROP_Y_VEL
            ),
            SpinningTwinsFloorLandingLoc3BB8A {
                next_collision_type: 0,
                y_velocity: 0
            },
            "ASM loc_3BB8A clears collision_type and y_vel after floor contact"
        );
    }

    #[test]
    fn spinning_twins_settle_entry_loc_3bb94_clears_motion_and_resumes_walker() {
        assert_eq!(
            spinning_twins_settle_entry_loc_3bb94(0),
            SpinningTwinsSettleEntryLoc3BB94 {
                next_collision_type: 0,
                x_velocity: 0,
                y_velocity: 0,
                has_level_collision: false,
                is_moved: false,
                restart_walk_animation: true,
                walker_entry: SpinningTwinsSettleWalkerEntryLoc3BB94::LeftLoc3C0D2
            },
            "ASM loc_3BB94 clears motion/collision state and sends clear x_direction to loc_3C0D2"
        );
        assert_eq!(
            spinning_twins_settle_entry_loc_3bb94(0xFF),
            SpinningTwinsSettleEntryLoc3BB94 {
                next_collision_type: 0,
                x_velocity: 0,
                y_velocity: 0,
                has_level_collision: false,
                is_moved: false,
                restart_walk_animation: true,
                walker_entry: SpinningTwinsSettleWalkerEntryLoc3BB94::RightLoc3C026
            },
            "ASM loc_3BB94 sends set x_direction to loc_3C026 after the same settle clears"
        );
    }

    #[test]
    fn spinning_twins_settled_collision_dispatch_loc_3bbc4_routes_contact_words() {
        assert_eq!(
            spinning_twins_settled_collision_dispatch_loc_3bbc4(0),
            SpinningTwinsSettledCollisionRouteLoc3BBC4::ResumeWalk,
            "ASM loc_3BBC4 sends a clear collision word straight back to walking"
        );
        assert_eq!(
            spinning_twins_settled_collision_dispatch_loc_3bbc4(0x8000),
            SpinningTwinsSettledCollisionRouteLoc3BBC4::Death,
            "ASM loc_3BBC4 branches to death for negative collision words"
        );
        assert_eq!(
            spinning_twins_settled_collision_dispatch_loc_3bbc4(SPINNING_TWINS_COLID_KIDABOVE),
            SpinningTwinsSettledCollisionRouteLoc3BBC4::Damage,
            "ASM loc_3BBC4 sends colid_kidabove to the damage branch"
        );
        assert_eq!(
            spinning_twins_settled_collision_dispatch_loc_3bbc4(SPINNING_TWINS_COLID_HURT),
            SpinningTwinsSettledCollisionRouteLoc3BBC4::Damage,
            "ASM loc_3BBC4 sends colid_hurt to the same damage branch"
        );
        assert_eq!(
            spinning_twins_settled_collision_dispatch_loc_3bbc4(SPINNING_TWINS_COLID_KIDRIGHT),
            SpinningTwinsSettledCollisionRouteLoc3BBC4::BerzerkerSideGate,
            "ASM loc_3BBC4 sends colid_kidright to the Berzerker side-contact gate"
        );
        assert_eq!(
            spinning_twins_settled_collision_dispatch_loc_3bbc4(SPINNING_TWINS_COLID_KIDLEFT),
            SpinningTwinsSettledCollisionRouteLoc3BBC4::BerzerkerSideGate,
            "ASM loc_3BBC4 sends colid_kidleft to the same side-contact gate"
        );
        assert_eq!(
            spinning_twins_settled_collision_dispatch_loc_3bbc4(SPINNING_TWINS_COLID_FLOOR),
            SpinningTwinsSettledCollisionRouteLoc3BBC4::ResumeWalk,
            "ASM loc_3BBC4 falls through to walking for unmatched positive collision words"
        );
    }

    #[test]
    fn spinning_twins_resume_walk_loc_3bbe6_clears_collision_and_decodes_direction() {
        assert_eq!(
            spinning_twins_resume_walk_loc_3bbe6(0),
            SpinningTwinsResumeWalkLoc3BBE6 {
                next_collision_type: 0,
                resume_direction: Direction::Left
            },
            "ASM loc_3BBE6 clears collision_type and resumes left when x_direction is clear"
        );
        assert_eq!(
            spinning_twins_resume_walk_loc_3bbe6(0xFF),
            SpinningTwinsResumeWalkLoc3BBE6 {
                next_collision_type: 0,
                resume_direction: Direction::Right
            },
            "ASM loc_3BBE6 clears collision_type and resumes right when x_direction is set"
        );
    }

    #[test]
    fn spinning_twins_berzerker_side_gate_loc_3bbf6_requires_active_charge() {
        assert_eq!(
            spinning_twins_berzerker_side_gate_loc_3bbf6(false, 0),
            SpinningTwinsBerzerkerSideGateLoc3BBF6::ResumeWalk(SpinningTwinsResumeWalkLoc3BBE6 {
                next_collision_type: 0,
                resume_direction: Direction::Left,
            }),
            "ASM loc_3BBF6 should branch back to loc_3BBE6 while Berzerker_charging is clear"
        );
        assert_eq!(
            spinning_twins_berzerker_side_gate_loc_3bbf6(true, 0xFF),
            SpinningTwinsBerzerkerSideGateLoc3BBF6::Launch(
                SpinningTwinsBerzerkerLaunchSetupLoc3BBF6 {
                    has_level_collision: false,
                    has_kid_collision: true,
                    is_moved: true,
                    is_animated: false,
                    sprite_mapping: "LnkTo_unk_C7CDE-Data_Index",
                    x_velocity: 0,
                    y_velocity: -0x0004_8000,
                }
            ),
            "ASM loc_3BBF6 should seed the upward Berzerker launch when charging"
        );
    }

    #[test]
    fn spinning_twins_berzerker_launch_loop_loc_3bc22_adds_gravity_and_repeats() {
        assert_eq!(
            SPINNING_TWINS_BERZERKER_LAUNCH_GRAVITY_LOC_3BC22, 0x0000_4000,
            "ASM loc_3BC22 adds `$4000` to y_vel on every airborne tick"
        );
        assert_eq!(
            spinning_twins_berzerker_launch_loop_tick_loc_3bc22(-0x0004_8000),
            SpinningTwinsBerzerkerLaunchLoopTickLoc3BC22 {
                hibernates_one_frame: true,
                runs_shared_retention_check_sub_3c3ce: true,
                next_y_velocity: -0x0004_4000,
                loops: true,
            },
            "ASM loc_3BC22 should hibernate, run sub_3C3CE, add gravity, and loop"
        );
        assert_eq!(
            spinning_twins_berzerker_launch_loop_tick_loc_3bc22(-0x0004_4000).next_y_velocity,
            -0x0004_0000,
            "ASM loc_3BC22 should use the same `$4000` gravity step on the next loop"
        );
    }

    #[test]
    fn spinning_twins_damage_contact_loc_3bc34_decrements_hp_before_recovery_or_death() {
        assert_eq!(
            spinning_twins_damage_contact_loc_3bc34(1, 0),
            SpinningTwinsDamageContactLoc3BC34::Death { next_hit_points: 0 },
            "ASM loc_3BC34 should branch to the death path when `$44(a3)` decrements to zero"
        );
        assert_eq!(
            spinning_twins_damage_contact_loc_3bc34(2, 0xFF),
            SpinningTwinsDamageContactLoc3BC34::Recover(SpinningTwinsDamageRecoveryLoc3BC34 {
                next_hit_points: 1,
                hit_animation: "stru_3B836",
                waits_until_hit_animation_finished: true,
                restored_walk_animation: "stru_3B81C",
                resume_walk: SpinningTwinsResumeWalkLoc3BBE6 {
                    next_collision_type: 0,
                    resume_direction: Direction::Right,
                },
            }),
            "ASM loc_3BC34 should play hit animation, restore walking, then resume through loc_3BBE6"
        );
    }

    #[test]
    fn spinning_twins_death_status_route_loc_3bc78_decodes_status_byte_sign_and_bit6() {
        assert_eq!(
            spinning_twins_death_status_route_loc_3bc78(0x05),
            SpinningTwinsDeathStatusRouteLoc3BC78::ClearStatusSlot(
                SpinningTwinsClearStatusSlotLoc3BC98 {
                    index: 5,
                    word_value: 0,
                }
            ),
            "ASM loc_3BC78 should route a positive `$42(a5)` byte to the status-slot path"
        );
        assert_eq!(
            spinning_twins_death_status_route_loc_3bc78(0xC7),
            SpinningTwinsDeathStatusRouteLoc3BC78::SubtractSpawnCounter {
                index: 7,
                decrement: 0x0400,
            },
            "ASM loc_3BC78 should mask bit-7+bit-6 `$42(a5)` to a six-bit EnemyStatus_Table index"
        );
        assert_eq!(
            spinning_twins_death_status_route_loc_3bc78(0x80),
            SpinningTwinsDeathStatusRouteLoc3BC78::DeleteOnly,
            "ASM loc_3BC78 should leave the status table alone when only bit 7 is set"
        );
    }

    #[test]
    fn spinning_twins_clear_status_slot_loc_3bc98_clears_positive_status_word() {
        assert_eq!(
            spinning_twins_clear_status_slot_loc_3bc98(0x0A),
            SpinningTwinsClearStatusSlotLoc3BC98 {
                index: 0x0A,
                word_value: 0,
            },
            "ASM loc_3BC98 should double positive `$42(a5)` as an EnemyStatus_Table word offset and clear that word"
        );
    }

    #[test]
    fn spinning_twins_delete_tail_loc_3bca2_deletes_after_every_status_route() {
        let delete_tail = SpinningTwinsDeleteTailLoc3BCA2 {
            deletes_current_object: true,
        };
        assert_eq!(
            spinning_twins_delete_tail_loc_3bca2(),
            delete_tail,
            "ASM loc_3BCA2 should jump to j_Delete_CurrentObject"
        );
        assert_eq!(
            spinning_twins_death_status_loc_3bc78(0x05),
            SpinningTwinsDeathStatusLoc3BC78 {
                status_route: SpinningTwinsDeathStatusRouteLoc3BC78::ClearStatusSlot(
                    SpinningTwinsClearStatusSlotLoc3BC98 {
                        index: 5,
                        word_value: 0,
                    },
                ),
                delete_tail,
            },
            "ASM loc_3BCA2 should also be reached after the positive status-slot route"
        );
        assert_eq!(
            spinning_twins_death_status_loc_3bc78(0xC7).delete_tail,
            delete_tail,
            "ASM loc_3BCA2 should also be reached after the spawn-counter route"
        );
        assert_eq!(
            spinning_twins_death_status_loc_3bc78(0x80).delete_tail,
            delete_tail,
            "ASM loc_3BCA2 should also be reached by the delete-only status route"
        );
    }

    #[test]
    fn spinning_twins_partner_signal_tail_loc_3bca6_marks_linked_twin_before_delete() {
        assert_eq!(
            spinning_twins_partner_signal_tail_loc_3bca6(0x03),
            SpinningTwinsPartnerSignalTailLoc3BCA6 {
                partner_slot: 0x03,
                next_partner_flag_5b: 0xFF,
                deletes_current_object: true,
            },
            "ASM loc_3BCA6 should follow `$5C(a5)`, set linked twin `$5B` with `st`, then delete"
        );
    }

    #[test]
    fn spinning_twins_falling_visual_init_loc_3b9b0_offsets_parent_position() {
        let visual = spinning_twins_falling_visual_init_loc_3b9b0(0x0100_0000, 0x0080_0000);

        assert_eq!(
            visual.x_pos, 0x00ED_0000,
            "ASM loc_3B9B0 subtracts `$13` from the copied parent X position"
        );
        assert_eq!(
            visual.y_pos, 0x0040_0000,
            "ASM loc_3B9B0 subtracts `$40` from the copied parent Y position"
        );
        assert_eq!(visual.priority, 1, "ASM loc_3B9B0 writes priority 1");
        assert!(
            visual.display_enabled,
            "ASM loc_3B9B0 sets `$13` so the visual worker displays"
        );
        assert_eq!(
            visual.step_index, 0,
            "ASM loc_3B9B0 clears d0 before entering the movement loop"
        );
        assert_eq!(
            visual.frames_until_next_step_index, 4,
            "ASM loc_3B9B0 seeds d1 with four frames per movement-table entry"
        );
    }

    #[test]
    fn spinning_twins_falling_visual_step_loc_3ba00_uses_signed_camera_relative_position() {
        let state = SpinningTwinsFallingVisualStateLoc3BA00 {
            x_pos: 256 << 16,
            y_pos: 128 << 16,
            step_index: 0,
            frames_until_next_step_index: 4,
            reverse_x_motion: false,
        };

        let stepped = spinning_twins_falling_visual_step_loc_3ba00(state, 300 << 16, 100 << 16);

        assert!(
            stepped.alive,
            "ASM Spinning_Twins.asm loc_3BA00 subtracts camera words after converting FP positions to signed screen pixels"
        );
        assert_eq!(stepped.state.x_pos, (256 << 16) + 0x1_2000);
        assert_eq!(stepped.state.y_pos, 129 << 16);
    }

    #[test]
    fn spinning_twins_falling_visual_step_loc_3ba3c_moves_and_uses_table_cadence() {
        let mut state = SpinningTwinsFallingVisualStateLoc3BA00 {
            x_pos: 0x0100_0000,
            y_pos: 0x0080_0000,
            step_index: 0,
            frames_until_next_step_index: 4,
            reverse_x_motion: false,
        };

        let first = spinning_twins_falling_visual_step_loc_3ba00(state, 0, 0);
        assert!(first.alive);
        assert_eq!(
            first.state.x_pos, 0x0101_2000,
            "ASM loc_3BA3C adds dword_3BA60[0] to X on the first frame"
        );
        assert_eq!(
            first.state.y_pos, 0x0081_0000,
            "ASM loc_3BA3C adds one pixel to Y each frame"
        );
        assert_eq!(first.state.step_index, 0);
        assert_eq!(first.state.frames_until_next_step_index, 3);

        state = first.state;
        for _ in 0..3 {
            state = spinning_twins_falling_visual_step_loc_3ba00(state, 0, 0).state;
        }
        assert_eq!(
            state.step_index, 1,
            "ASM loc_3BA00 advances d0 after four frames"
        );
        assert_eq!(state.frames_until_next_step_index, 4);

        for _ in 0..16 {
            state = spinning_twins_falling_visual_step_loc_3ba00(state, 0, 0).state;
        }
        assert_eq!(
            state.step_index, 0,
            "ASM loc_3BA00 wraps d0 after the fifth movement-table entry"
        );
        assert!(
            state.reverse_x_motion,
            "ASM loc_3BA00 toggles d3 before restarting the movement table"
        );
        assert_eq!(
            state.x_pos, 0x0124_0000,
            "ASM loc_3BA00 table covers 36px of horizontal drift per half-cycle"
        );
        assert_eq!(state.y_pos, 0x0094_0000);
    }

    #[test]
    fn spinning_twins_falling_visual_step_loc_3ba3c_reverses_x_drift_after_table_wrap() {
        let state = SpinningTwinsFallingVisualStateLoc3BA00 {
            x_pos: 0x0124_0000,
            y_pos: 0x0094_0000,
            step_index: 0,
            frames_until_next_step_index: 4,
            reverse_x_motion: true,
        };

        let stepped = spinning_twins_falling_visual_step_loc_3ba00(state, 0, 0);

        assert!(stepped.alive);
        assert_eq!(
            stepped.state.x_pos, 0x0122_E000,
            "ASM Spinning_Twins.asm loc_3BA00 negates dword_3BA60 when d3 is negative"
        );
        assert_eq!(stepped.state.y_pos, 0x0095_0000);
        assert_eq!(stepped.state.step_index, 0);
        assert_eq!(stepped.state.frames_until_next_step_index, 3);
        assert!(stepped.state.reverse_x_motion);
    }

    #[test]
    fn spinning_twins_falling_visual_step_loc_3ba5c_deletes_outside_camera_box() {
        let state = SpinningTwinsFallingVisualStateLoc3BA00 {
            x_pos: 100 << 16,
            y_pos: 128 << 16,
            step_index: 0,
            frames_until_next_step_index: 4,
            reverse_x_motion: false,
        };

        let stepped = spinning_twins_falling_visual_step_loc_3ba00(state, 361 << 16, 0);

        assert!(
            !stepped.alive,
            "ASM loc_3BA5C deletes the visual when screen X is less than -$104"
        );
        assert_eq!(
            stepped.state, state,
            "ASM loc_3BA5C is reached before applying movement"
        );
    }

    #[test]
    fn spinning_twins_live_falling_visual_marks_zero_hp_when_outside_loc_3ba5c() {
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 0x0300_0000, 0x0080_0000, 1);
        enemy.spinning_twins_phase = SpinningTwinsPhase::FallingVisual;
        enemy.grounded_frames = 4;

        step_spinning_twins(&mut enemy, None, None, None);

        assert_eq!(
            enemy.hit_points, 0,
            "ASM Spinning_Twins.asm loc_3BA5C deletes the falling visual once it leaves the signed screen box"
        );
    }

    // --- Ghost blocks ---

    #[test]
    fn ghost_block_toggles_on_timer_expiry() {
        let mut group = GhostGroup {
            col: 3,
            row: 5,
            count: 2,
            vertical: false,
            solid_time: 10,
            vanished_time: 5,
            timer: 1,
            state: 0,
            solid: true,
        };
        let (timer, state) = NativeRuntime::shooter_slot_tick_sub_F730(
            group.timer,
            group.state,
            group.solid_time,
            group.vanished_time,
        );
        group.timer = timer;
        group.state = state;
        group.solid = false;

        assert_eq!(group.state, 0x8001, "ASM loc_F8A6 starts vanish lead-in");
        assert_eq!(group.timer, 2, "vanish lead-in uses the two-frame ladder");
        assert!(!group.solid, "state 0 -> $8001 should leave the solid hold");
    }

    #[test]
    fn ghost_block_group_cells_horizontal() {
        let group = GhostGroup {
            col: 3,
            row: 5,
            count: 3,
            vertical: false,
            solid_time: 10,
            vanished_time: 10,
            timer: 0,
            state: 0,
            solid: true,
        };
        let cells: Vec<(u16, u16)> = (0..group.count)
            .map(|i| {
                if group.vertical {
                    (group.col, group.row + u16::from(i))
                } else {
                    (group.col + u16::from(i), group.row)
                }
            })
            .collect();
        assert_eq!(cells, vec![(3, 5), (4, 5), (5, 5)]);
    }

    #[test]
    fn ghost_block_group_cells_vertical() {
        let group = GhostGroup {
            col: 3,
            row: 5,
            count: 2,
            vertical: true,
            solid_time: 10,
            vanished_time: 10,
            timer: 0,
            state: 0,
            solid: true,
        };
        let cells: Vec<(u16, u16)> = (0..group.count)
            .map(|i| {
                if group.vertical {
                    (group.col, group.row + u16::from(i))
                } else {
                    (group.col + u16::from(i), group.row)
                }
            })
            .collect();
        assert_eq!(cells, vec![(3, 5), (3, 6)]);
    }

    #[test]
    fn ghost_block_list_parsing() {
        // Ghost entry with timer=0 (the prize parser sees layout_addr=0 → ends prizes,
        // then ghost parser picks up). This mirrors a level with no prizes.
        let ghost_entry: [u8; 16] = [
            0x00, 0x00, // timer = 0 (also acts as prize terminator since layout_addr=0)
            0x00, 0x00, // ASM sub_F730 state = 0 (solid `$E304` hold)
            0x00, 0x14, // solid_time = 20
            0x00, 0x0A, // vanished_time = 10
            0x00, 0x10, // layout_addr (non-zero → not a ghost terminator)
            0x00, 0x03, // col = 3
            0x00, 0x05, // row = 5
            0x02, // count nibble = 2 → count = 3
            0x01, // vertical = true
        ];
        // Ghost terminator (all zeros)
        let ghost_end: [u8; 16] = [0u8; 16];

        let mut data = Vec::new();
        data.extend_from_slice(&ghost_entry);
        data.extend_from_slice(&ghost_end);

        let bl = terrain::BlockList::from_raw(&data, 20);
        assert_eq!(bl.prize_blocks.len(), 0, "no prizes");
        assert_eq!(bl.ghost_blocks.len(), 1);
        assert_eq!(bl.ghost_blocks[0].col, 3);
        assert_eq!(bl.ghost_blocks[0].row, 5);
        assert_eq!(bl.ghost_blocks[0].count, 3);
        assert!(bl.ghost_blocks[0].vertical);
        assert_eq!(bl.ghost_blocks[0].solid_time, 20);
        assert_eq!(bl.ghost_blocks[0].vanished_time, 10);
        assert_eq!(bl.ghost_blocks[0].state, 0);
        assert!(bl.ghost_blocks[0].solid, "state=0 means solid `$E304` hold");
    }

    // --- Terrain type speed table selection ---

    #[test]
    fn terrain_type_from_block_type_maps_ice_and_rubber() {
        assert_eq!(
            TerrainType::from_block_type(terrain::BlockType::Ice),
            TerrainType::Ice
        );
        assert_eq!(
            TerrainType::from_block_type(terrain::BlockType::Rubber),
            TerrainType::Rubber
        );
        assert_eq!(
            TerrainType::from_block_type(terrain::BlockType::Prize),
            TerrainType::Normal
        );
        assert_eq!(
            TerrainType::from_block_type(terrain::BlockType::None),
            TerrainType::Normal
        );
    }

    #[test]
    fn ground_speed_table_selects_by_form_and_terrain() {
        assert_eq!(
            PlayerForm::TheKid.ground_speed_table(TerrainType::Normal),
            &physics::NORMAL_KID
        );
        assert_eq!(
            PlayerForm::TheKid.ground_speed_table(TerrainType::Ice),
            &physics::RUBBER_KID
        );
        assert_eq!(
            PlayerForm::TheKid.ground_speed_table(TerrainType::Rubber),
            &physics::ICE_KID
        );
        assert_eq!(
            PlayerForm::Micromax.ground_speed_table(TerrainType::Ice),
            &physics::RUBBER_MICROMAX
        );
        assert_eq!(
            PlayerForm::Juggernaut.ground_speed_table(TerrainType::Rubber),
            &physics::ICE_JUGGERNAUT
        );
    }

    #[test]
    fn floor_collision_sets_terrain_type_from_block() {
        // Build a grid with an ice block at row 4 (y=64)
        let w = 4u16;
        let h = 6u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Row 4, col 1: solid ice block
        // has_block=1, collision=6 (solid), block_type=2 (ice)
        // hi byte: 1_110_0010 = 0xE2
        let offset = (4 * usize::from(w) + 1) * 2;
        data[offset] = 0xE2;
        data[offset + 1] = 0x01;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 24 << 16, // col 1 centre
                y_pos: 64 << 16, // at floor level
                x_velocity: 0,
                y_velocity: 0x1_0000, // falling
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 0,
                floor_terrain_type: TerrainType::Normal,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: w * 16,
                height_pixels: h * 16,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        slice.apply_floor_collision(Some(&grid), false, false, PLAYER_HALF_WIDTH, true);
        assert!(slice.player.on_ground);
        assert_eq!(slice.terrain.floor_terrain_type, TerrainType::Ice);
    }

    #[test]
    fn leaving_ground_resets_terrain_type_to_normal() {
        let w = 4u16;
        let h = 4u16;
        let data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let mut slice = OpeningSliceState {
            player: PlayerState {
                x_pos: 24 << 16,
                y_pos: 32 << 16,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState { x_pos: 0, y_pos: 0 },
            terrain: TerrainState {
                floor_y_pos: 32 << 16,
                floor_terrain_type: TerrainType::Ice,
                slope_walk_kind: None,
                slope_sliding: false,
            },
            level_bounds: LevelBounds {
                width_pixels: w * 16,
                height_pixels: h * 16,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            walk_visual_accumulator: 0,
            prize_block_hit: None,
            lift_block_hit: None,
            shifting_block_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            ice_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            telepad_timer: 0x5A,
            pending_walking_floor_release: false,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
            kid_grabbed_by_hand: false,
        };

        // No floor under player → should leave ground and reset terrain type
        slice.apply_floor_collision(Some(&grid), false, false, PLAYER_HALF_WIDTH, true);
        assert!(!slice.player.on_ground);
        assert_eq!(slice.terrain.floor_terrain_type, TerrainType::Normal);
    }

    #[test]
    fn grounded_surface_type_sub_8600_requires_a_strict_ice_majority() {
        let grid = grid_with_cells(
            6,
            6,
            &[(0, 4, 0x60, 0x00), (1, 4, 0xE2, 0x01), (2, 4, 0xE2, 0x01)],
        );

        assert_eq!(
            grounded_surface_type_sub_8600(&grid, 24, 63, 16),
            TerrainType::Ice,
            "ASM sub_8600 should promote ice only when the scanned solid span has a strict ice majority",
        );

        let tie_grid = grid_with_cells(6, 6, &[(0, 4, 0x60, 0x00), (1, 4, 0xE2, 0x01)]);
        assert_eq!(
            grounded_surface_type_sub_8600(&tie_grid, 16, 63, 8),
            TerrainType::Normal,
            "ASM sub_8600 should leave the terrain at normal on an ice tie because `add.w d2,d2` must be strictly greater than the solid count",
        );
    }

    #[test]
    fn grounded_surface_type_sub_8600_promotes_rubber_on_ties() {
        let grid = grid_with_cells(6, 6, &[(0, 4, 0x60, 0x00), (1, 4, 0xE6, 0x01)]);

        assert_eq!(
            grounded_surface_type_sub_8600(&grid, 16, 63, 8),
            TerrainType::Rubber,
            "ASM sub_8600 should let rubber win ties because loc_8680 uses `blt` before writing terrain type 2",
        );
    }

    #[test]
    fn platform_collision_lands_falling_player() {
        // Platform at (40, 100), h=2 v=0 → 47px wide, 15px tall
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(50, 100); // feet at platform top

        apply_platform_collision(&mut slice, &[platform]);

        assert!(slice.player.on_ground);
        assert_eq!(slice.player.y_velocity, 0);
        // Lands 1 pixel above platform top (same as solid floor convention)
        assert_eq!(slice.player.y_pos, 99u32 << 16);
    }

    #[test]
    fn platform_collision_uses_horizontal_escape_when_side_overlap_is_smaller_loc_2686() {
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(40, 110);

        apply_platform_collision(&mut slice, &[platform]);

        assert_eq!(
            slice.player.x_pos,
            34u32 << 16,
            "ASM Platforms_CheckCollision loc_2672 should choose the smaller 6px horizontal escape and loc_2686 should push Kid left instead of landing",
        );
        assert_eq!(
            slice.player.y_pos,
            110u32 << 16,
            "ASM loc_2686 horizontal escape should not snap Kid to platform_y - 1",
        );
        assert!(
            !slice.player.on_ground,
            "ASM loc_2686 side escape should not record platform support via loc_26D4",
        );
    }

    #[test]
    fn platform_collision_horizontal_escape_carries_signed_platform_velocity_loc_26b6() {
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.x_velocity = -0x2_0000;
        let mut slice = make_airborne_slice(40, 110);

        apply_platform_collision(&mut slice, &[platform]);

        assert_eq!(
            slice.player.x_velocity, -0x2_0000,
            "ASM loc_26B6/loc_26C0 should carry the negative `$A(a2)` platform velocity into `$26(a0)` after a leftward side escape",
        );
    }

    #[test]
    fn platform_collision_horizontal_escape_carries_signed_platform_velocity_loc_26a2() {
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.x_velocity = 0x1_8000;
        let mut slice = make_airborne_slice(92, 100);

        apply_platform_collision(&mut slice, &[platform]);

        assert_eq!(
            slice.player.x_velocity, 0x1_8000,
            "ASM loc_26A2 should carry the positive `$A(a2)` platform velocity into `$26(a0)` after a rightward side escape",
        );
    }

    #[test]
    fn platform_collision_horizontal_escape_clamps_opposite_velocity_loc_26a2_26b6() {
        let mut right_escape_platform = make_test_platform(40, 100, 2, 0);
        right_escape_platform.x_velocity = -0x1_8000;
        let mut right_escape = make_airborne_slice(92, 100);

        apply_platform_collision(&mut right_escape, &[right_escape_platform]);

        assert_eq!(
            right_escape.player.x_velocity, 0,
            "ASM loc_26A2 should clamp negative `$A(a2)` to zero after a rightward side escape",
        );

        let mut left_escape_platform = make_test_platform(40, 100, 2, 0);
        left_escape_platform.x_velocity = 0x1_8000;
        let mut left_escape = make_airborne_slice(40, 110);

        apply_platform_collision(&mut left_escape, &[left_escape_platform]);

        assert_eq!(
            left_escape.player.x_velocity, 0,
            "ASM loc_26B6 should clamp positive `$A(a2)` to zero after a leftward side escape",
        );
    }

    #[test]
    fn platform_collision_opposite_side_escapes_escalate_to_crush_loc_271a() {
        let left_escape = make_test_platform(40, 100, 2, 0);
        let right_escape = make_test_platform(18, 100, 0, 0);
        let mut slice = make_airborne_slice(40, 110);

        let outcome = apply_platform_collision(&mut slice, &[left_escape, right_escape]);

        assert_eq!(
            outcome,
            PlatformCollisionOutcome::CrushDeath,
            "ASM loc_26B6 sets `$FFFFFAA2`, restarts at loc_2590, then loc_26A2 must jump to loc_271A when the opposite side also pushes Kid",
        );
    }

    #[test]
    fn platform_collision_bottom_face_carries_y_velocity_bias_loc_26fe() {
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.y_velocity = 0x2_8000;
        let mut slice = make_airborne_slice(63, 132);
        slice.player.y_velocity = -0x1_0000;
        slice.pre_gravity_y_vel = -0x1_0000;

        let outcome = apply_platform_collision_with_ceiling_probe(
            &mut slice,
            &[platform],
            None,
            PlayerForm::TheKid,
            false,
            None,
            true,
        );

        assert_eq!(
            outcome,
            PlatformCollisionOutcome::None,
            "ASM loc_26FE should branch back to loc_2590 for MoveID_Jump after an underside platform carry, not record platform support",
        );
        assert_eq!(
            slice.player.y_pos,
            147u32 << 16,
            "ASM loc_26D4 should first push Kid below the platform bottom before loc_26FE carries y velocity",
        );
        assert_eq!(
            slice.player.y_velocity, 0x2_C000,
            "ASM loc_26FE should copy `$E(a2)+$4000` into Kid y_vel after a bottom-face hit",
        );
        assert!(
            !slice.player.on_ground,
            "ASM loc_26FE bottom-face carry must not set Addr_PlatformStandingOn",
        );
    }

    #[test]
    fn platform_collision_bottom_face_uses_character_movement_jump_loc_26fe() {
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.y_velocity = 0x2_8000;
        let mut slice = make_airborne_slice(63, 132);
        slice.player.y_velocity = 0x1_0000;
        slice.pre_gravity_y_vel = 0x1_0000;

        let outcome = apply_platform_collision_with_ceiling_probe(
            &mut slice,
            &[platform],
            None,
            PlayerForm::TheKid,
            false,
            None,
            true,
        );

        assert_eq!(
            outcome,
            PlatformCollisionOutcome::None,
            "ASM loc_26FE branches back to loc_2590 when Character_Movement is MoveID_Jump, even after y_vel has become positive",
        );
        assert_eq!(
            slice.player.y_velocity, 0x2_C000,
            "ASM loc_26FE still copies `$E(a2)+$4000` before the MoveID_Jump branch",
        );
    }

    #[test]
    fn platform_collision_top_support_restarts_scan_for_bottom_crush_loc_26d4_271a() {
        let lower_support = make_test_platform(40, 100, 2, 0);
        let upper_crush = make_test_platform(40, 53, 2, 0);
        let mut slice = make_airborne_slice(63, 100);

        let outcome = apply_platform_collision(&mut slice, &[lower_support, upper_crush]);

        assert_eq!(
            outcome,
            PlatformCollisionOutcome::CrushDeath,
            "ASM loc_26D4 sets `$FFFFFAA4` and restarts at loc_2590; the later non-jump loc_26FE underside hit must still reach loc_271A in the same Platforms_CheckCollision call",
        );
    }

    #[test]
    fn platform_collision_airborne_aabb_uses_current_form_half_width_loc_2590() {
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(34, 100);

        let outcome = apply_platform_collision_with_ceiling_probe(
            &mut slice,
            &[platform],
            None,
            PlayerForm::RedStealth,
            false,
            None,
            false,
        );

        assert_eq!(
            outcome,
            PlatformCollisionOutcome::StandingOn(0xEDBA),
            "ASM loc_2590 reads Kid_hitbox_left/right after current-form hitbox setup; Red Stealth's 7px half-width overlaps the platform edge where The Kid's 5px box would miss",
        );
    }

    #[test]
    fn platform_support_check_sub_8ed0_reports_unsupported_without_a_platform() {
        assert!(
            platform_support_check_sub_8ED0(None, 50u32 << 16, 7, PlayerForm::RedStealth, false),
            "ASM sub_8ED0 should return unsupported when Addr_PlatformStandingOn is zero",
        );
    }

    #[test]
    fn platform_support_check_sub_8ed0_uses_the_supplied_half_width_when_not_crawling() {
        let platform = make_test_platform(40, 100, 2, 0);

        assert!(
            !platform_support_check_sub_8ED0(
                Some(&platform),
                35u32 << 16,
                7,
                PlayerForm::RedStealth,
                false,
            ),
            "ASM sub_8ED0 should keep support when centre+half-width still overlaps the platform edge",
        );
        assert!(
            platform_support_check_sub_8ED0(
                Some(&platform),
                32u32 << 16,
                7,
                PlayerForm::RedStealth,
                false,
            ),
            "ASM sub_8ED0 should drop support once centre+half-width falls strictly left of the platform",
        );
    }

    #[test]
    fn platform_carry_uses_current_form_half_width_loc_8ed0() {
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.x_velocity = 0x0001_0000;
        let mut slice = make_airborne_slice(34, 100);
        slice.player.on_ground = true;

        apply_platform_collision_with_ceiling_probe(
            &mut slice,
            &[platform],
            None,
            PlayerForm::RedStealth,
            false,
            Some(0xEDBA),
            false,
        );

        assert_eq!(
            slice.player.x_pos,
            35u32 << 16,
            "ASM sub_8ED0 uses the current helmet half-width; Red Stealth's 7px support still overlaps where The Kid's 5px support would detach",
        );
    }

    #[test]
    fn platform_support_check_sub_8ed0_uses_crawl_width_overrides() {
        let platform = make_test_platform(40, 100, 2, 0);

        assert!(
            !platform_support_check_sub_8ED0(
                Some(&platform),
                34u32 << 16,
                23,
                PlayerForm::Juggernaut,
                true,
            ),
            "ASM sub_8ED0 should clamp crawling non-Kid forms to a 7px support half-width",
        );
        assert!(
            platform_support_check_sub_8ED0(
                Some(&platform),
                32u32 << 16,
                23,
                PlayerForm::Juggernaut,
                true,
            ),
            "ASM sub_8ED0 should still drop crawling non-Kid forms once centre+7px misses the edge",
        );
        assert!(
            !platform_support_check_sub_8ED0(
                Some(&platform),
                35u32 << 16,
                7,
                PlayerForm::TheKid,
                true,
            ),
            "ASM sub_8ED0 should narrow crawling Kid support to 5px, keeping the last overlapping position",
        );
        assert!(
            platform_support_check_sub_8ED0(
                Some(&platform),
                34u32 << 16,
                7,
                PlayerForm::TheKid,
                true,
            ),
            "ASM sub_8ED0 should drop crawling Kid support one pixel earlier than the generic 7px path",
        );
    }

    #[test]
    fn standing_platform_index_loc_8ed0_uses_stored_platform_and_crawling_width_override() {
        let platform = make_test_platform(40, 100, 2, 0);
        let mut player = make_airborne_slice(32, 100).player;
        player.on_ground = true;

        assert_eq!(
            standing_platform_index_loc_8ED0(
                &[platform],
                Some(0xEDBA),
                player,
                PlayerForm::Juggernaut,
                false
            ),
            Some(0),
            "ASM loc_8EFA: standing Juggernaut uses the supplied 23px half-width and still overlaps the platform edge",
        );
        assert_eq!(
            standing_platform_index_loc_8ED0(
                &[platform],
                Some(0xEDBA),
                player,
                PlayerForm::Juggernaut,
                true
            ),
            None,
            "ASM sub_8ED0 (kid.asm:10288-10294): MoveID_Crawling clamps non-Kid support to 7px, so x=32 drops support at the left edge",
        );
        assert_eq!(
            standing_platform_index_loc_8ED0(
                &[platform],
                None,
                player,
                PlayerForm::Juggernaut,
                false
            ),
            None,
            "ASM sub_8ED0 reads Addr_PlatformStandingOn first and must not discover a nearby platform when that pointer is zero",
        );
    }

    #[test]
    fn remap_standing_platform_addr_preserves_slot_identity_after_deletion_loc_2850() {
        let mut removed = make_test_platform(40, 100, 2, 0);
        removed.slot_addr = Some(0xEDBA);
        removed.layout_index = Some(3);
        let mut stood_on = make_test_platform(80, 100, 2, 0);
        stood_on.slot_addr = Some(0xEDDC);
        stood_on.layout_index = Some(7);
        let before = [removed, stood_on];
        let after = [stood_on];

        assert_eq!(
            remap_standing_platform_addr_by_layout_index_loc_2850(Some(0xEDDC), &before, &after),
            Some(0xEDDC),
            "ASM Addr_PlatformStandingOn stores a platform slot address, so native must preserve the same layout-offset identity when loc_2850/deletion compacts vector indices",
        );
        assert_eq!(
            remap_standing_platform_addr_by_layout_index_loc_2850(Some(0xEDBA), &before, &after),
            None,
            "when the stood-on slot itself is deleted, native should clear the Addr_PlatformStandingOn equivalent",
        );
    }

    #[test]
    fn platform_collision_ignores_grounded_player() {
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(50, 100);
        slice.player.on_ground = true; // already grounded

        apply_platform_collision(&mut slice, &[platform]);

        // Should not modify position — already on ground
        assert_eq!(slice.player.y_pos, 100u32 << 16);
    }

    #[test]
    fn platform_collision_ignores_rising_player() {
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(50, 100);
        slice.player.y_velocity = -0x2_0000; // moving upward

        apply_platform_collision(&mut slice, &[platform]);

        assert!(!slice.player.on_ground);
    }

    #[test]
    fn platform_collision_misses_when_player_is_not_above() {
        let platform = make_test_platform(40, 100, 2, 0);
        // Player is far left of platform (x=10, platform starts at x=40)
        let mut slice = make_airborne_slice(10, 100);

        apply_platform_collision(&mut slice, &[platform]);

        assert!(!slice.player.on_ground);
    }

    #[test]
    fn platform_collision_misses_when_player_is_well_above() {
        let platform = make_test_platform(40, 100, 2, 0);
        // Player at y=50, platform at y=100 — too high to land
        let mut slice = make_airborne_slice(50, 50);

        apply_platform_collision(&mut slice, &[platform]);

        assert!(!slice.player.on_ground);
    }

    #[test]
    fn platform_riding_carries_grounded_player() {
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.x_velocity = 0x1_0000; // 1 px/frame rightward
        platform.y_velocity = 0;

        // Player is grounded on the platform (feet at y=99, platform at y=100)
        let mut slice = make_airborne_slice(50, 99);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;

        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);

        assert!(
            slice.player.x_pos > x_before,
            "grounded player should be carried rightward by moving platform"
        );
        assert_eq!(
            slice.player.x_pos - x_before,
            0x1_0000,
            "player should move by exactly the platform's x velocity"
        );
    }

    #[test]
    fn platform_riding_runs_sub_8f26_wall_clamp() {
        let mut platform = make_test_platform(20, 48, 2, 0);
        platform.x_velocity = 0x8_0000;
        platform.y_velocity = 0;
        let mut slice = make_airborne_slice(41, 47);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;

        let w = 5u16;
        let h = 4u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        for row in 1u16..=3 {
            let offset = (usize::from(row) * usize::from(w) + usize::from(3u16)) * 2;
            data[offset] = 0x60;
        }
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        apply_platform_collision_with_ceiling_probe(
            &mut slice,
            &[platform],
            Some(&grid),
            PlayerForm::TheKid,
            false,
            Some(0xEDBA),
            false,
        );

        assert_eq!(
            slice.player.x_pos,
            42u32 << 16,
            "ASM loc_8D72/loc_82D8 carry reaches sub_8F26, so a right-moving platform must clamp Kid to wall_x - (half_width + 1) instead of carrying through the wall",
        );
    }

    #[test]
    fn platform_riding_ignores_stationary_platform() {
        let platform = make_test_platform(40, 100, 2, 0);
        // Stationary platform (zero velocity)
        let mut slice = make_airborne_slice(50, 99);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;

        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);

        assert_eq!(
            slice.player.x_pos, x_before,
            "stationary platform should not move player"
        );
    }

    #[test]
    fn platform_dimensions_from_h_v_variants() {
        // h_variant=4, v_variant=0 → 79×15
        let p = make_test_platform(0, 0, 4, 0);
        assert_eq!(p.width_px, 79);
        assert_eq!(p.height_px, 15);

        // h_variant=2, v_variant=1 → 47×31
        let p = make_test_platform(0, 0, 2, 1);
        assert_eq!(p.width_px, 47);
        assert_eq!(p.height_px, 31);
    }

    #[test]
    fn platform_render_metadata_follows_loc_2814_and_platform_art_table() {
        let (width_px, height_px) = platform_size_words_loc_2814(0, 3, 0);
        assert_eq!(
            (width_px, height_px),
            (63, 13),
            "ASM loc_2814/loc_2822 decodes `(h << 4) | v` into size words and applies the type-zero `subq.w #2,$1C(a3)` height quirk",
        );

        let render_flags = platform_render_flags_loc_2814(8, 6);
        assert_eq!(
            render_flags, 6,
            "ASM loc_2822/loc_244C clears only bit 7 of the packed `$1F(a3)` byte in the retail `platforms_newtype = 0` build",
        );
        assert_eq!(
            platform_mapping_addr_word_loc_2822(render_flags),
            0x025A,
            "ASM PlatformArtAddr_Table indexes by the post-`bclr #7` byte offset, so shipped `t8 s6` special platforms use the fourth art word",
        );
    }

    #[test]
    fn active_platform_snapshot_carries_loader_offset_words_loc_286e() {
        let mut platform = make_test_platform(0, 0, 2, 1);
        platform.layout_index = Some(3);

        let snapshot = platform.snapshot();

        assert_eq!(
            snapshot
                .debug_fields
                .get("platform_loader_offset_words")
                .map(String::as_str),
            Some("36"),
            "ASM Manage_PlatformLoading stamps `$20(a3)` with `layout_index * $C`, so live platform snapshots should expose that ROM identity after movement updates",
        );
    }

    #[test]
    fn active_platform_from_placement_loc_2822_keeps_collision_shape_without_modelling_art_words() {
        let platform = ActivePlatform::from_placement(
            &PlatformPlacement {
                x: 24,
                y: 25,
                buffer_left: 0,
                buffer_right: 40,
                buffer_top: 0,
                buffer_bottom: 19,
                kind: 0,
                subtype: 2,
                h_variant: 3,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                    relative_offset: None,
                },
            },
            None,
        );

        assert_eq!(
            platform.width_px, 63,
            "ASM loc_2822 (kid.asm:4748) only swaps the platform art word after the size fields are ready, so the native gameplay model should preserve the collision width from `h_variant` while omitting `$18(a3)`",
        );
        assert_eq!(
            platform.height_px, 13,
            "ASM loc_2822 (kid.asm:4744-4746) should subtract 2px from decoded type-0 platform height before the art lookup",
        );
        assert_eq!(
            platform.script, None,
            "ASM loc_2822 is still on the stationary-platform path; without a relative script offset the native port should leave motion scripting unset",
        );
        assert_eq!(
            platform.x_pos,
            (24u32 << 4) << 16,
            "ASM loc_2762 (kid.asm:4717) should scale the `ptfm` x word by 16 before native collision uses fixed-point world pixels",
        );
        assert_eq!(
            platform.y_pos,
            (25u32 << 4) << 16,
            "ASM loc_2762 (kid.asm:4721) should scale the `ptfm` y word by 16 before native collision uses fixed-point world pixels",
        );

        let non_zero_type = ActivePlatform::from_placement(
            &PlatformPlacement {
                kind: 1,
                ..PlatformPlacement {
                    x: 24,
                    y: 25,
                    buffer_left: 0,
                    buffer_right: 40,
                    buffer_top: 0,
                    buffer_bottom: 19,
                    kind: 0,
                    subtype: 2,
                    h_variant: 3,
                    v_variant: 0,
                    script: PlatformScriptRef {
                        raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                        relative_offset: None,
                    },
                }
            },
            None,
        );
        assert_eq!(
            non_zero_type.height_px, 15,
            "ASM loc_2822 should only apply the `subq.w #2,$1C(a3)` height quirk when the decoded type byte is zero",
        );
    }

    #[test]
    fn active_platform_from_placement_loc_2836_splits_scripted_and_special_platforms() {
        use std::collections::BTreeMap;

        let script_offset = 0x001A;
        let script_table = wildside_data::PlatformScriptTable {
            scripts: BTreeMap::from([(
                script_offset,
                wildside_data::PlatformScript {
                    commands: vec![wildside_data::PlatformMoveCommand {
                        duration: 6,
                        x_velocity: 0x1_0000,
                        y_velocity: -0x2_0000,
                    }],
                    loop_target_offset: script_offset,
                },
            )]),
        };

        let scripted = ActivePlatform::from_placement(
            &PlatformPlacement {
                x: 24,
                y: 25,
                buffer_left: 0,
                buffer_right: 40,
                buffer_top: 0,
                buffer_bottom: 19,
                kind: 0,
                subtype: 2,
                h_variant: 3,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("PlatformScript_361C-PlatformScript_BaseAddress"),
                    // Historical generated caches stored the absolute label
                    // value. ASM loc_2836 consumes the relative `$A(a4)`
                    // script offset, so runtime must normalise this to
                    // `$001A` or Under Skull Mountain 3's moving platforms
                    // remain static.
                    relative_offset: Some(0x361C),
                },
            },
            Some(&script_table),
        );
        assert_eq!(
            scripted.script,
            Some(PlatformScriptState {
                script_offset,
                command_index: 0,
                remaining_duration: 6,
            }),
            "ASM loc_2836 (kid.asm:4762) should send non-negative platform types down the scripted path, seeding `$12(a3)` from `PlatformScript_BaseAddress + $A(a4)` and arming the first script command",
        );

        let special = ActivePlatform::from_placement(
            &PlatformPlacement {
                x: 24,
                y: 25,
                buffer_left: 0,
                buffer_right: 40,
                buffer_top: 0,
                buffer_bottom: 19,
                kind: 0x80,
                subtype: 2,
                h_variant: 3,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                    relative_offset: None,
                },
            },
            Some(&script_table),
        );
        assert_eq!(
            special.script, None,
            "ASM loc_2836 should branch special platforms with bit 7 set to `loc_2876`, so the native gameplay model must leave motion scripting unset for `relative_offset: None` placements",
        );
    }

    #[test]
    fn platform_script_moves_platform() {
        use std::collections::BTreeMap;
        let mut scripts = BTreeMap::new();
        scripts.insert(
            0x0000,
            wildside_data::PlatformScript {
                commands: vec![
                    wildside_data::PlatformMoveCommand {
                        duration: 3,
                        x_velocity: 0x1_0000,
                        y_velocity: 0,
                    },
                    wildside_data::PlatformMoveCommand {
                        duration: 3,
                        x_velocity: -0x1_0000,
                        y_velocity: 0,
                    },
                ],
                loop_target_offset: 0x0000,
            },
        );
        let table = wildside_data::PlatformScriptTable { scripts };

        let mut platform = ActivePlatform {
            slot_addr: Some(crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C),
            layout_index: None,
            kind: 0,
            subtype: 0,
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            deactivated_flag: 0,
            script: Some(PlatformScriptState {
                script_offset: 0,
                command_index: 0,
                remaining_duration: 3,
            }),
            special: None,
        };

        let start_x = platform.x_pos;

        // Step 3 frames of moving right
        for _ in 0..3 {
            platform.step_script(&table);
        }
        // Should have moved 3 pixels right
        assert_eq!(platform.x_pos, start_x + 3 * 0x1_0000);
        // Should now be on second command (moving left)
        assert_eq!(platform.script.unwrap().command_index, 1);

        // Step 3 more frames of moving left
        for _ in 0..3 {
            platform.step_script(&table);
        }
        // Should be back at start
        assert_eq!(platform.x_pos, start_x);
        // Should have looped back to first command
        assert_eq!(platform.script.unwrap().command_index, 0);
    }

    #[test]
    fn platform_step_script_loc_23f2_applies_the_current_velocity_before_reloading_script_state() {
        use std::collections::BTreeMap;

        let mut scripts = BTreeMap::new();
        scripts.insert(
            0x0000,
            wildside_data::PlatformScript {
                commands: vec![
                    wildside_data::PlatformMoveCommand {
                        duration: 1,
                        x_velocity: 0x1_0000,
                        y_velocity: 0,
                    },
                    wildside_data::PlatformMoveCommand {
                        duration: 4,
                        x_velocity: -0x2_0000,
                        y_velocity: 0,
                    },
                ],
                loop_target_offset: 0x0000,
            },
        );
        let table = wildside_data::PlatformScriptTable { scripts };

        let mut platform = ActivePlatform {
            slot_addr: Some(crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C),
            layout_index: None,
            kind: 0,
            subtype: 0,
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            deactivated_flag: 0,
            script: Some(PlatformScriptState {
                script_offset: 0,
                command_index: 0,
                remaining_duration: 1,
            }),
            special: None,
        };

        platform.step_script(&table);

        assert_eq!(
            platform.x_pos,
            101 << 16,
            "ASM loc_23F2 (kid.asm:4269) adds `$A(a2)` into `2(a2)` before the duration countdown can fall through into the script reload path",
        );
        assert_eq!(
            platform.script,
            Some(PlatformScriptState {
                script_offset: 0,
                command_index: 1,
                remaining_duration: 4,
            }),
            "ASM loc_23F2 should leave the platform armed with the next command after the terminal tick finishes",
        );
    }

    #[test]
    fn execute_scripted_platforms_sub_23ea_fa64_freezes_the_whole_pass() {
        use std::collections::BTreeMap;

        let mut scripts = BTreeMap::new();
        scripts.insert(
            0x0000,
            wildside_data::PlatformScript {
                commands: vec![wildside_data::PlatformMoveCommand {
                    duration: 2,
                    x_velocity: 0x1_0000,
                    y_velocity: 0,
                }],
                loop_target_offset: 0x0000,
            },
        );
        let table = wildside_data::PlatformScriptTable { scripts };

        let mut platforms = vec![ActivePlatform {
            slot_addr: Some(crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C),
            layout_index: None,
            kind: 0,
            subtype: 0,
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            deactivated_flag: 0,
            script: Some(PlatformScriptState {
                script_offset: 0,
                command_index: 0,
                remaining_duration: 2,
            }),
            special: None,
        }];
        let frozen = platforms[0];

        execute_scripted_platforms_sub_23EA(&mut platforms, &table, true);

        assert_eq!(
            platforms[0], frozen,
            "ASM Execute_ScriptedPlatforms/sub_23EA (kid.asm:4265): non-zero `$FFFFFA64` branches to return_2442 before moving or ticking any platform",
        );

        execute_scripted_platforms_sub_23EA(&mut platforms, &table, false);

        assert_eq!(
            platforms[0].x_pos,
            frozen.x_pos + 0x1_0000,
            "ASM loc_23F2 should run normally again when `$FFFFFA64` is clear",
        );
        assert_eq!(
            platforms[0].script.expect("script").remaining_duration,
            frozen.script.expect("script").remaining_duration - 1,
            "ASM loc_23F2 countdown should tick only on the unfrozen pass",
        );
    }

    #[test]
    fn platform_step_script_loc_23f2_wraps_position_like_add_l() {
        use std::collections::BTreeMap;

        let mut scripts = BTreeMap::new();
        scripts.insert(
            0x0000,
            wildside_data::PlatformScript {
                commands: vec![wildside_data::PlatformMoveCommand {
                    duration: 2,
                    x_velocity: 0x2_0000,
                    y_velocity: -0x2_0000,
                }],
                loop_target_offset: 0x0000,
            },
        );
        let table = wildside_data::PlatformScriptTable { scripts };

        let mut platform = ActivePlatform {
            slot_addr: Some(crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C),
            layout_index: None,
            kind: 0,
            subtype: 0,
            x_pos: 0xFFFF_0000,
            y_pos: 0x0001_0000,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            deactivated_flag: 0,
            script: Some(PlatformScriptState {
                script_offset: 0,
                command_index: 0,
                remaining_duration: 2,
            }),
            special: None,
        };

        platform.step_script(&table);

        assert_eq!(
            platform.x_pos, 0x0001_0000,
            "ASM loc_23F2 (kid.asm:4282): `add.l d7,2(a2)` wraps 32-bit x_pos instead of clamping at u32::MAX",
        );
        assert_eq!(
            platform.y_pos, 0xFFFF_0000,
            "ASM loc_23F2 (kid.asm:4284): `add.l d7,6(a2)` wraps negative y velocity below zero instead of clamping",
        );
    }

    #[test]
    fn platform_step_script_loc_2422_immediately_loads_the_next_velocity_on_expiry() {
        use std::collections::BTreeMap;

        let mut scripts = BTreeMap::new();
        scripts.insert(
            0x0000,
            wildside_data::PlatformScript {
                commands: vec![
                    wildside_data::PlatformMoveCommand {
                        duration: 1,
                        x_velocity: 0x1_0000,
                        y_velocity: 0,
                    },
                    wildside_data::PlatformMoveCommand {
                        duration: 5,
                        x_velocity: -0x3_0000,
                        y_velocity: 0x2_0000,
                    },
                ],
                loop_target_offset: 0x0000,
            },
        );
        let table = wildside_data::PlatformScriptTable { scripts };

        let mut platform = ActivePlatform {
            slot_addr: Some(crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C),
            layout_index: None,
            kind: 0,
            subtype: 0,
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            deactivated_flag: 0,
            script: Some(PlatformScriptState {
                script_offset: 0,
                command_index: 0,
                remaining_duration: 1,
            }),
            special: None,
        };

        platform.step_script(&table);

        assert_eq!(
            platform.x_velocity, -0x3_0000,
            "ASM loc_2422 (kid.asm:4290) stores the next script entry's x velocity into `$A(a2)` as soon as the countdown expires",
        );
        assert_eq!(
            platform.y_velocity, 0x2_0000,
            "ASM loc_2422 should also store the next script entry's y velocity into `$E(a2)` on the same frame",
        );
        assert_eq!(
            platform.script,
            Some(PlatformScriptState {
                script_offset: 0,
                command_index: 1,
                remaining_duration: 5,
            }),
            "ASM loc_2422 should reload `$16(a2)` with the next duration while advancing the script pointer",
        );
    }

    #[test]
    fn platform_step_script_loc_243e_rewinds_to_the_first_typed_command_after_the_loop_sentinel() {
        use std::collections::BTreeMap;

        let mut scripts = BTreeMap::new();
        scripts.insert(
            0x0000,
            wildside_data::PlatformScript {
                commands: vec![
                    wildside_data::PlatformMoveCommand {
                        duration: 2,
                        x_velocity: 0x4_0000,
                        y_velocity: -0x1_0000,
                    },
                    wildside_data::PlatformMoveCommand {
                        duration: 1,
                        x_velocity: -0x2_0000,
                        y_velocity: 0x3_0000,
                    },
                ],
                loop_target_offset: 0x0000,
            },
        );
        let table = wildside_data::PlatformScriptTable { scripts };

        let mut platform = ActivePlatform {
            slot_addr: Some(crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C),
            layout_index: None,
            kind: 0,
            subtype: 0,
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            deactivated_flag: 0,
            script: Some(PlatformScriptState {
                script_offset: 0,
                command_index: 1,
                remaining_duration: 1,
            }),
            special: None,
        };

        platform.step_script(&table);

        assert_eq!(
            platform.script,
            Some(PlatformScriptState {
                script_offset: 0,
                command_index: 0,
                remaining_duration: 2,
            }),
            "ASM loc_243E (kid.asm:4302) follows the `$FFFF` loop-back pointer; ordinary retail scripts point that longword back to their own first command",
        );
        assert_eq!(
            platform.x_velocity, 0x4_0000,
            "ASM loc_243E should leave `$A(a2)` armed with the first command's x velocity after the loop-back",
        );
        assert_eq!(
            platform.y_velocity, -0x1_0000,
            "ASM loc_243E should leave `$E(a2)` armed with the first command's y velocity after the loop-back",
        );
    }

    #[test]
    fn platform_step_script_loc_243e_follows_cross_script_loop_pointer() {
        use std::collections::BTreeMap;

        let mut scripts = BTreeMap::new();
        scripts.insert(
            0x0972,
            wildside_data::PlatformScript {
                commands: vec![wildside_data::PlatformMoveCommand {
                    duration: 1,
                    x_velocity: 0x1_0000,
                    y_velocity: 0,
                }],
                loop_target_offset: 0x097C,
            },
        );
        scripts.insert(
            0x097C,
            wildside_data::PlatformScript {
                commands: vec![wildside_data::PlatformMoveCommand {
                    duration: 4,
                    x_velocity: -0x2_0000,
                    y_velocity: 0x3_0000,
                }],
                loop_target_offset: 0x097C,
            },
        );
        let table = wildside_data::PlatformScriptTable { scripts };

        let mut platform = ActivePlatform {
            slot_addr: Some(crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C),
            layout_index: None,
            kind: 0,
            subtype: 0,
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            deactivated_flag: 0,
            script: Some(PlatformScriptState {
                script_offset: 0x0972,
                command_index: 0,
                remaining_duration: 1,
            }),
            special: None,
        };

        platform.step_script(&table);

        assert_eq!(
            platform.script,
            Some(PlatformScriptState {
                script_offset: 0x097C,
                command_index: 0,
                remaining_duration: 4,
            }),
            "ASM loc_243E (kid.asm:4302) loads the longword loop pointer after `$FFFF`; retail PlatformScript_3F74 targets PlatformScript_3F7E rather than its own first command",
        );
        assert_eq!(platform.x_velocity, -0x2_0000);
        assert_eq!(platform.y_velocity, 0x3_0000);
    }

    // --- Platform collision targeted mutation tests ---
    // Tests below target surviving mutants from cargo-mutants on platforms.rs.
    // Each test name references the line/mutation it catches.

    #[test]
    fn advance_to_window_loc_2762_consumes_exact_left_buffer_edge() {
        let window = ActivationWindow {
            x_min: 0,
            x_max: 96,
            y_min: 0,
            y_max: 224,
        };
        let placement = PlatformPlacement {
            x: 100,
            y: 50,
            buffer_left: 3,
            buffer_right: 0,
            buffer_top: 0,
            buffer_bottom: 0,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement],
        };
        let mut state = PlatformLoaderState::default();
        state.advance_to_window(&layout, window);
        assert_eq!(
            state.next_index, 1,
            "ASM loc_2762 compares Kid/player pixels against absolute buffer edges scaled by 32; equality does not take the `bgt` out-of-range branch"
        );
    }

    #[test]
    fn advance_to_window_loc_2762_stops_past_left_buffer_edge() {
        let window = ActivationWindow {
            x_min: 0,
            x_max: 96,
            y_min: 0,
            y_max: 224,
        };
        let placement = PlatformPlacement {
            x: 100,
            y: 50,
            buffer_left: 4,
            buffer_right: 0,
            buffer_top: 0,
            buffer_bottom: 0,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement],
        };
        let mut state = PlatformLoaderState::default();
        state.advance_to_window(&layout, window);
        assert_eq!(
            state.next_index, 0,
            "ASM loc_2762 branches out of range when the absolute left buffer edge is greater than Kid/player X"
        );
    }

    #[test]
    fn advance_to_window_loc_2744_processes_one_platform_record_per_call() {
        let window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement.clone(), placement],
        };
        let mut state = PlatformLoaderState::default();

        state.advance_to_window(&layout, window);
        assert_eq!(
            state.next_index, 1,
            "ASM Manage_PlatformLoading loc_2744 processes one 12-byte platform record per call, so the second active placement must wait for the next frame",
        );

        state.advance_to_window(&layout, window);
        assert_eq!(
            state.next_index, 2,
            "ASM loc_286E advances the cursor by one record, allowing the second placement on the next call",
        );
    }

    #[test]
    fn platform_loader_does_not_materialise_y_rejected_high_water_record_loc_2762() {
        let x_seen_y_rejected = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 640,
            y_max: 640,
        };
        let now_in_range_before_cursor_wraps = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement],
        };
        let mut state = PlatformLoaderState::default();

        state.advance_to_window(&layout, x_seen_y_rejected);

        assert_eq!(
            state.next_index, 1,
            "native may still record the high-water cursor for debug/back-compat, but ASM loc_2762 did not allocate this Y-rejected platform",
        );
        assert!(
            state
                .active_platforms(&layout, now_in_range_before_cursor_wraps)
                .is_empty(),
            "ASM Manage_PlatformLoading only allocates the current 12-byte record when all loc_2762 range checks pass; high-water reach alone must not create a platform",
        );
    }

    #[test]
    fn platform_is_active_requires_both_x_and_y_in_window() {
        // Mutant: line 67, replace && with ||
        // A platform outside x range but inside y range must NOT be active.
        let window = ActivationWindow {
            x_min: 100,
            x_max: 200,
            y_min: 50,
            y_max: 150,
        };
        let outside_x = PlatformPlacement {
            x: 50,  // outside x range
            y: 100, // inside y range
            buffer_left: 0,
            buffer_right: 2,
            buffer_top: 0,
            buffer_bottom: 5,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        assert!(
            !platform_is_active(&outside_x, window),
            "platform outside x range should be inactive even if y is in range"
        );
        let outside_y = PlatformPlacement {
            x: 150, // inside x
            y: 200, // outside y
            buffer_left: 0,
            buffer_right: 7,
            buffer_top: 0,
            buffer_bottom: 1,
            ..outside_x.clone()
        };
        assert!(
            !platform_is_active(&outside_y, window),
            "platform outside y range should be inactive even if x is in range"
        );
    }

    #[test]
    fn platform_is_active_loc_2762_keeps_exact_buffer_edges_in_range() {
        let window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        assert!(
            platform_is_active(&placement, window),
            "ASM loc_2762 (kid.asm:4684) should keep a platform active when Kid lands exactly on the buffered left/top edge because the ROM only branches away on strict `bgt`/`blt` misses",
        );

        let right_bottom_edge = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 0,
            buffer_right: 10,
            buffer_top: 0,
            buffer_bottom: 10,
            ..placement
        };
        assert!(
            platform_is_active(&right_bottom_edge, window),
            "ASM loc_2762 should also keep the exact buffered right/bottom edge active; equality reaches the loader instead of the `loc_2850` reject path",
        );
    }

    #[test]
    fn platform_loader_active_roster_loc_23ae_caps_at_17_slots() {
        let window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement; 18],
        };
        let state = PlatformLoaderState {
            next_index: layout.placements.len(),
            offset_words: 0,
            loaded_mask: (1u64 << 18) - 1,
            ..PlatformLoaderState::default()
        };

        let active = state.active_platforms(&layout, window);

        assert_eq!(
            active.len(),
            17,
            "ASM Allocate_PlatformSlot loc_23AE returns when the 17-slot free list is empty, so the 18th in-range platform must not materialise",
        );
    }

    #[test]
    fn platform_loader_number_platforms_loc_231c_counts_loaded_slots_with_pool_cap() {
        let state = PlatformLoaderState {
            next_index: 20,
            offset_words: 0,
            loaded_mask: (1u64 << 20) - 1,
            ..PlatformLoaderState::default()
        };

        assert_eq!(
            state.number_platforms_loc_231C(),
            17,
            "ASM Initialize_Platforms loc_231C builds exactly 17 free slots, so `(Number_Platforms).w` cannot exceed the pool even if native debug state sees more loaded-record bits",
        );
    }

    #[test]
    fn platform_loader_synthetic_slot_addr_loc_231c_uses_platform_ram_stride() {
        assert_eq!(
            PlatformLoaderState::synthetic_slot_addr_loc_231C(0),
            Some(0xEDBA),
            "ASM Initialize_Platforms sub_231C starts the Platform_RAM free list at $FFFFEDBA",
        );
        assert_eq!(
            PlatformLoaderState::synthetic_slot_addr_loc_231C(1),
            Some(0xEDDC),
            "ASM sub_231C links slots by adding the $22-byte platform object stride",
        );
        assert_eq!(
            PlatformLoaderState::synthetic_slot_addr_loc_231C(17),
            None,
            "ASM sub_231C terminates after 17 platform slots, so native synthetic addresses must not run past the pool",
        );
    }

    #[test]
    fn platform_loader_active_roster_orders_new_allocations_at_front_loc_238e() {
        let window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement.clone(), placement],
        };
        let state = PlatformLoaderState {
            next_index: 2,
            offset_words: 0,
            loaded_mask: 0b11,
            ..PlatformLoaderState::default()
        };

        let active = state.active_platforms(&layout, window);
        let layout_indices: Vec<Option<usize>> = active
            .iter()
            .map(|platform| platform.layout_index)
            .collect();

        assert_eq!(
            layout_indices,
            vec![Some(1), Some(0)],
            "ASM Allocate_PlatformSlot sub_238E links each new platform at Addr_FirstPlatformSlot, so snapshots should expose newest allocations first",
        );
    }

    #[test]
    fn platform_loader_slot_model_loc_2744_reuses_deallocated_slot_on_later_record() {
        let active_window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let inactive_window = ActivationWindow {
            x_min: 800,
            x_max: 800,
            y_min: 800,
            y_max: 800,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement.clone(), placement],
        };
        let mut model = PlatformLoaderSlotModelLoc2744::initialize_sub_231C();

        model.advance_to_window(&layout, active_window);
        model.advance_to_window(&layout, active_window);
        assert_eq!(
            model.active_slot_records_loc_2728(),
            vec![
                PlatformLoaderSlotRecordLoc2728 {
                    slot_addr: 0xEDDC,
                    layout_offset_words: 0x000C,
                    deactivated_flag: 0,
                },
                PlatformLoaderSlotRecordLoc2728 {
                    slot_addr: 0xEDBA,
                    layout_offset_words: 0x0000,
                    deactivated_flag: 0,
                },
            ],
            "ASM sub_238E should allocate records 0 and 1 into Platform_RAM slots and scan the newest slot first",
        );

        model.advance_to_window(&layout, inactive_window);
        assert_eq!(
            model.active_slot_records_loc_2728(),
            vec![PlatformLoaderSlotRecordLoc2728 {
                slot_addr: 0xEDDC,
                layout_offset_words: 0x000C,
                deactivated_flag: 0,
            }],
            "ASM loc_2744 wraps to record 0, then loc_2850 -> loc_286A deallocates the normal slot whose `$20(a3)` is offset 0",
        );

        model.advance_to_window(&layout, active_window);
        model.advance_to_window(&layout, active_window);
        assert_eq!(
            model.active_slot_records_loc_2728(),
            vec![
                PlatformLoaderSlotRecordLoc2728 {
                    slot_addr: 0xEDBA,
                    layout_offset_words: 0x0000,
                    deactivated_flag: 0,
                },
                PlatformLoaderSlotRecordLoc2728 {
                    slot_addr: 0xEDDC,
                    layout_offset_words: 0x000C,
                    deactivated_flag: 0,
                },
            ],
            "ASM sub_23B4 pushes the deallocated slot to the free-list head, so the next sub_238E allocation should reuse $EDBA and relink it at Addr_FirstPlatformSlot",
        );
    }

    #[test]
    fn platform_loader_slot_model_loc_2850_marks_special_slot_deactivated_until_reloaded() {
        let active_window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let inactive_window = ActivationWindow {
            x_min: 800,
            x_max: 800,
            y_min: 800,
            y_max: 800,
        };
        let layout = PlatformLayout {
            placements: vec![PlatformPlacement {
                x: 20,
                y: 20,
                buffer_left: 10,
                buffer_right: 10,
                buffer_top: 10,
                buffer_bottom: 10,
                kind: 8,
                subtype: 4,
                h_variant: 1,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("4"),
                    relative_offset: None,
                },
            }],
        };
        let mut model = PlatformLoaderSlotModelLoc2744::initialize_sub_231C();

        model.advance_to_window(&layout, active_window);
        assert_eq!(
            model.active_slot_records_loc_2728(),
            vec![PlatformLoaderSlotRecordLoc2728 {
                slot_addr: 0xEDBA,
                layout_offset_words: 0x0000,
                deactivated_flag: 0,
            }],
            "ASM loc_2876 clears `$12(a3)` when a retained special platform is in range",
        );

        model.advance_to_window(&layout, inactive_window);
        assert_eq!(
            model.active_slot_records_loc_2728(),
            vec![PlatformLoaderSlotRecordLoc2728 {
                slot_addr: 0xEDBA,
                layout_offset_words: 0x0000,
                deactivated_flag: 0xFF,
            }],
            "ASM loc_2850 keeps special platforms linked and writes `st $12(a3)` instead of branching to loc_286A deallocation",
        );

        model.advance_to_window(&layout, active_window);
        assert_eq!(
            model.active_slot_records_loc_2728(),
            vec![PlatformLoaderSlotRecordLoc2728 {
                slot_addr: 0xEDBA,
                layout_offset_words: 0x0000,
                deactivated_flag: 0,
            }],
            "ASM loc_2876 should clear the retained special platform's deactivation byte when the loader sees it in range again",
        );
    }

    #[test]
    fn platform_loader_slot_model_exposes_loaded_mask_and_high_water_like_live_state_loc_2762() {
        let active_window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let inactive_window = ActivationWindow {
            x_min: 800,
            x_max: 800,
            y_min: 800,
            y_max: 800,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement.clone(), placement],
        };
        let mut live_state = PlatformLoaderState::default();
        let mut slot_model = PlatformLoaderSlotModelLoc2744::initialize_sub_231C();

        live_state.advance_to_window(&layout, active_window);
        slot_model.advance_to_window(&layout, active_window);
        live_state.advance_to_window(&layout, active_window);
        slot_model.advance_to_window(&layout, active_window);

        assert_eq!(
            slot_model.next_index_loc_2762(),
            live_state.next_index,
            "ASM loc_2762 updates the loader high-water bound independently of the underlying slot storage",
        );
        assert_eq!(
            slot_model.loaded_mask_loc_2762(),
            live_state.loaded_mask,
            "the slot-owned model should expose the same loaded-record mask as the current live loader before it replaces that approximation",
        );

        live_state.advance_to_window(&layout, inactive_window);
        slot_model.advance_to_window(&layout, inactive_window);
        assert_eq!(
            slot_model.next_index_loc_2762(),
            live_state.next_index,
            "ASM loc_2744 wraps through the terminator without lowering the high-water bound",
        );
        assert_eq!(
            slot_model.loaded_mask_loc_2762(),
            live_state.loaded_mask,
            "ASM loc_2850 -> loc_286A deallocation should clear the same record bit that the current live loaded-mask approximation clears",
        );
    }

    #[test]
    fn platform_loader_slot_model_snapshots_follow_reused_slot_chain_loc_257e() {
        let active_window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let inactive_window = ActivationWindow {
            x_min: 800,
            x_max: 800,
            y_min: 800,
            y_max: 800,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement.clone(), placement],
        };
        let mut model = PlatformLoaderSlotModelLoc2744::initialize_sub_231C();

        model.advance_to_window(&layout, active_window);
        model.advance_to_window(&layout, active_window);
        model.advance_to_window(&layout, inactive_window);
        model.advance_to_window(&layout, active_window);
        model.advance_to_window(&layout, active_window);

        let slot_model_indices: Vec<Option<usize>> = model
            .active_platforms_loc_257E(&layout, active_window)
            .into_iter()
            .map(|platform| platform.layout_index)
            .collect();
        let loaded_mask_indices: Vec<Option<usize>> = PlatformLoaderState {
            next_index: model.next_index_loc_2762(),
            offset_words: 0,
            loaded_mask: model.loaded_mask_loc_2762(),
            ..PlatformLoaderState::default()
        }
        .active_platforms(&layout, active_window)
        .into_iter()
        .map(|platform| platform.layout_index)
        .collect();

        assert_eq!(
            slot_model_indices,
            vec![Some(0), Some(1)],
            "ASM Platforms_CheckCollision loc_257E walks Addr_FirstPlatformSlot order, so after sub_23B4 frees record 0 and sub_238E reloads it, the reused slot becomes the scan head",
        );
        assert_eq!(
            loaded_mask_indices,
            vec![Some(1), Some(0)],
            "the current loaded-mask snapshot approximation still reverses layout order and cannot represent Platform_RAM slot reuse order",
        );
    }

    #[test]
    fn platform_loader_state_snapshots_follow_reused_slot_chain_loc_257e() {
        let active_window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let inactive_window = ActivationWindow {
            x_min: 800,
            x_max: 800,
            y_min: 800,
            y_max: 800,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement.clone(), placement],
        };
        let mut state = PlatformLoaderState::default();

        state.advance_to_window(&layout, active_window);
        state.advance_to_window(&layout, active_window);
        state.advance_to_window(&layout, inactive_window);
        state.advance_to_window(&layout, active_window);
        state.advance_to_window(&layout, active_window);

        let live_indices: Vec<Option<usize>> = state
            .active_platforms(&layout, active_window)
            .into_iter()
            .map(|platform| platform.layout_index)
            .collect();

        assert_eq!(
            live_indices,
            vec![Some(0), Some(1)],
            "ASM Platforms_CheckCollision loc_257E scans Addr_FirstPlatformSlot order; live PlatformLoaderState must preserve sub_23B4/sub_238E slot reuse order, not rebuild by reverse layout index",
        );
    }

    #[test]
    fn runtime_platform_roster_follows_loader_slot_chain_loc_257e() {
        let active_window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let inactive_window = ActivationWindow {
            x_min: 800,
            x_max: 800,
            y_min: 800,
            y_max: 800,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement.clone(), placement],
        };
        let mut state = PlatformLoaderState::default();

        state.advance_to_window(&layout, active_window);
        state.advance_to_window(&layout, active_window);
        state.advance_to_window(&layout, inactive_window);
        state.advance_to_window(&layout, active_window);
        state.advance_to_window(&layout, active_window);

        let active =
            runtime_active_platforms_from_loader_loc_257E(&layout, state, active_window, &[], None);
        let live_indices: Vec<Option<usize>> = active
            .iter()
            .map(|platform| platform.layout_index)
            .collect();

        assert_eq!(
            live_indices,
            vec![Some(0), Some(1)],
            "ASM Platforms_CheckCollision loc_257E consumes the live Platform_RAM chain, so the runtime ActivePlatform roster must keep the loader slot order used by collision",
        );
    }

    #[test]
    fn platform_loader_state_deallocates_deleted_special_slot_for_reuse_loc_41ee_23b4() {
        let active_window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let inactive_window = ActivationWindow {
            x_min: 800,
            x_max: 800,
            y_min: 800,
            y_max: 800,
        };
        let special_placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 8,
            subtype: 4,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("4"),
                relative_offset: None,
            },
        };
        let normal_placement = PlatformPlacement {
            kind: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
            ..special_placement.clone()
        };
        let layout = PlatformLayout {
            placements: vec![special_placement, normal_placement],
        };
        let mut state = PlatformLoaderState::default();

        state.advance_to_window(&layout, active_window);
        state.advance_to_window(&layout, inactive_window);
        assert_eq!(
            state
                .slot_model
                .active_slot_records_loc_2728()
                .first()
                .map(|record| (record.slot_addr, record.layout_offset_words)),
            Some((0xEDBA, 0x0000)),
            "ASM loc_2850 keeps the special slot linked with `$12(a3)` set before TrapPlatformDown_Delete runs",
        );

        assert_eq!(
            state.deallocate_layout_offset_sub_23B4(0x0000),
            DeallocatePlatformSlotResultSub23B4::Deallocated,
            "ASM TrapPlatformDown_Delete loc_41EE tail-calls Deallocate_PlatformSlot before Delete_CurrentObject",
        );
        assert!(
            state.slot_model.active_slot_records_loc_2728().is_empty(),
            "Deallocate_PlatformSlot/sub_23B4 must unlink the deleted special platform from Addr_FirstPlatformSlot",
        );

        state.advance_to_window(&layout, active_window);
        assert_eq!(
            state
                .slot_model
                .active_slot_records_loc_2728()
                .first()
                .map(|record| (record.slot_addr, record.layout_offset_words)),
            Some((0xEDBA, 0x0000)),
            "Deallocate_PlatformSlot/sub_23B4 pushes the deleted slot to Addr_NextFreePlatformSlot, so the next Allocate_PlatformSlot/sub_238E reuses $EDBA when the cursor revisits record 0",
        );
    }

    #[test]
    fn platform_loader_keeps_loaded_special_slot_out_of_range_loc_2850() {
        let active_window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let inactive_window = ActivationWindow {
            x_min: 640,
            x_max: 640,
            y_min: 320,
            y_max: 320,
        };
        let special_placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 8,
            subtype: 4,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("4"),
                relative_offset: None,
            },
        };
        let normal_placement = PlatformPlacement {
            kind: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
            ..special_placement.clone()
        };

        let special_layout = PlatformLayout {
            placements: vec![special_placement],
        };
        let mut special_state = PlatformLoaderState::default();
        special_state.advance_to_window(&special_layout, active_window);
        special_state.advance_to_window(&special_layout, inactive_window);
        assert_eq!(
            special_state.loaded_mask, 0b1,
            "ASM loc_2850 keeps already-loaded special platforms linked and sets `$12(a3)` instead of branching to loc_286A deallocation",
        );

        let normal_layout = PlatformLayout {
            placements: vec![normal_placement],
        };
        let mut normal_state = PlatformLoaderState::default();
        normal_state.advance_to_window(&normal_layout, active_window);
        normal_state.advance_to_window(&normal_layout, inactive_window);
        assert_eq!(
            normal_state.loaded_mask, 0,
            "ASM loc_2850 sends normal already-loaded platforms to loc_286A, so their loaded bit should clear when the current record is out of range",
        );
    }

    #[test]
    fn platform_loader_snapshot_keeps_loaded_special_slot_out_of_range_loc_2850() {
        let inactive_window = ActivationWindow {
            x_min: 640,
            x_max: 640,
            y_min: 320,
            y_max: 320,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 8,
            subtype: 4,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("4"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement],
        };
        let state = PlatformLoaderState {
            next_index: 1,
            offset_words: 0,
            loaded_mask: 0b1,
            ..PlatformLoaderState::default()
        };

        let active = state.active_platforms(&layout, inactive_window);

        assert_eq!(
            active.len(),
            1,
            "ASM loc_2850 keeps an already-loaded special platform in Addr_FirstPlatformSlot even when the current loader range check rejects it",
        );
        assert_eq!(
            active[0]
                .debug_fields
                .get("platform_deactivated_flag")
                .map(String::as_str),
            Some("255"),
            "ASM loc_2850 sets `$12(a3)` with `st`, so snapshots should expose the retained special slot as deactivated",
        );
    }

    #[test]
    fn special_platform_out_of_range_sets_deactivation_flag_loc_2850() {
        let inactive_window = ActivationWindow {
            x_min: 640,
            x_max: 640,
            y_min: 320,
            y_max: 320,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 8,
            subtype: 4,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("4"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement],
        };
        let previous = vec![ActivePlatform::from_placement_with_index(
            0,
            &layout.placements[0],
            None,
        )];

        let active =
            runtime_active_platforms_loc_2762(&layout, 1, 0b1, inactive_window, &previous, None);

        assert_eq!(
            active.len(),
            1,
            "ASM loc_2850 keeps an already-loaded special platform slot instead of deallocating it immediately"
        );
        assert_eq!(
            active[0].deactivated_flag, 0xFF,
            "ASM loc_2850 uses `st $12(a3)` so TrapPlatformDown_WaitLoop can observe a nonzero deactivation flag"
        );
    }

    #[test]
    fn platform_loader_offset_for_index_uses_twelve_byte_records_loc_286e() {
        assert_eq!(platform_loader_offset_for_index_loc_286E(0), 0x0000);
        assert_eq!(platform_loader_offset_for_index_loc_286E(1), 0x000C);
        assert_eq!(platform_loader_offset_for_index_loc_286E(4), 0x0030);
        assert_eq!(
            platform_loader_offset_for_index_loc_286E(0x2000),
            0x8000,
            "ASM loc_286E uses 16-bit wrapping arithmetic when adding one 12-byte platform record to `(PlatformLoader_Offset).w`",
        );
    }

    #[test]
    fn active_platform_snapshot_exposes_loader_offset_words_loc_286e() {
        let window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let inactive = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 0,
            buffer_right: 0,
            buffer_top: 0,
            buffer_bottom: 0,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let active = PlatformPlacement {
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            ..inactive.clone()
        };
        let layout = PlatformLayout {
            placements: vec![inactive, active],
        };
        let state = PlatformLoaderState {
            next_index: layout.placements.len(),
            offset_words: 0,
            loaded_mask: 0b10,
            ..PlatformLoaderState::default()
        };

        let active = state.active_platforms(&layout, window);

        assert_eq!(active.len(), 1);
        assert_eq!(
            active[0].layout_index,
            Some(1),
            "pre-runtime platform snapshots should keep the same source layout index whose `$C`-scaled offset is stamped into `$20(a3)`",
        );
        assert_eq!(
            active[0]
                .debug_fields
                .get("platform_loader_offset_words")
                .map(String::as_str),
            Some("12"),
            "ASM Manage_PlatformLoading stamps `$20(a3)` with the current 12-byte platform-layout offset before loc_286E advances it",
        );
    }

    #[test]
    fn runtime_platform_roster_preserves_previous_slot_by_loader_offset_sub_2728() {
        let window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement.clone(), placement],
        };
        let mut previous =
            ActivePlatform::from_placement_with_index(1, &layout.placements[1], None);
        previous.x_velocity = 0x0001_4000;

        let active = runtime_active_platforms_loc_2762(&layout, 2, 0b10, window, &[previous], None);

        let preserved = active
            .iter()
            .find(|platform| platform.layout_index == Some(1))
            .expect("second platform should stay active");
        assert_eq!(
            preserved.x_velocity, 0x0001_4000,
            "ASM sub_2728 preserves the already-loaded platform slot whose `$20(a3)` matches the current PlatformLoader_Offset",
        );
    }

    #[test]
    fn runtime_platform_roster_uses_loaded_mask_not_high_water_loc_2762() {
        let window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement],
        };

        let active = runtime_active_platforms_loc_2762(&layout, 1, 0, window, &[], None);

        assert!(
            active.is_empty(),
            "ASM loc_2762 only allocates a platform after the current loader record passes all range checks; high-water `next_index` alone must not create a live collision platform",
        );
    }

    #[test]
    fn runtime_platform_roster_drops_previous_normal_slot_when_loaded_mask_clears_loc_286a() {
        let window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement],
        };
        let previous = ActivePlatform::from_placement_with_index(0, &layout.placements[0], None);

        let active = runtime_active_platforms_loc_2762(&layout, 1, 0, window, &[previous], None);

        assert!(
            active.is_empty(),
            "ASM loc_2850 branches normal existing slots to loc_286A, where Deallocate_PlatformSlot removes them from Addr_FirstPlatformSlot; a cleared loaded bit must not preserve the stale slot even if the current window is active",
        );
    }

    #[test]
    fn runtime_platform_roster_allocates_new_slots_at_front_loc_238e() {
        let window = ActivationWindow {
            x_min: 320,
            x_max: 320,
            y_min: 320,
            y_max: 320,
        };
        let placement = PlatformPlacement {
            x: 20,
            y: 20,
            buffer_left: 10,
            buffer_right: 10,
            buffer_top: 10,
            buffer_bottom: 10,
            kind: 0,
            subtype: 0,
            h_variant: 1,
            v_variant: 0,
            script: PlatformScriptRef {
                raw: String::from("PlatformScript_Nothing-PlatformScript_BaseAddress"),
                relative_offset: None,
            },
        };
        let layout = PlatformLayout {
            placements: vec![placement.clone(), placement],
        };
        let mut previous =
            ActivePlatform::from_placement_with_index(0, &layout.placements[0], None);
        previous.x_velocity = 0x0001_4000;

        let active = runtime_active_platforms_loc_2762(&layout, 2, 0b11, window, &[previous], None);

        let layout_indices: Vec<Option<usize>> = active
            .iter()
            .map(|platform| platform.layout_index)
            .collect();
        assert_eq!(
            layout_indices,
            vec![Some(1), Some(0)],
            "ASM Allocate_PlatformSlot sub_238E writes the new slot to Addr_FirstPlatformSlot, so a newly loaded platform must be scanned before older linked slots",
        );
        assert_eq!(
            active[1].x_velocity, 0x0001_4000,
            "the older slot should keep its live state while moving behind the newly allocated front slot",
        );
    }

    #[test]
    fn find_active_platform_by_layout_offset_sub_2728_returns_the_first_matching_slot() {
        assert_eq!(
            find_active_platform_by_layout_offset_sub_2728(
                0x0030,
                &[
                    active_platform_handle_sub_2728(0x0018),
                    active_platform_handle_sub_2728(0x0030),
                    active_platform_handle_sub_2728(0x0030),
                ],
            ),
            Some(1),
            "ASM loc_272C (kid.asm:4652) should keep scanning active platform slots until `$20(a2)` matches `d5`, then stop on the first matching slot and leave later duplicates untouched",
        );
    }

    #[test]
    fn find_active_platform_by_layout_offset_sub_2728_returns_none_when_no_slot_matches() {
        assert_eq!(
            find_active_platform_by_layout_offset_sub_2728(
                0x0048,
                &[
                    active_platform_handle_sub_2728(0x0018),
                    active_platform_handle_sub_2728(0x0030),
                ],
            ),
            None,
            "ASM loc_2740 (kid.asm:4662) should return zero once `loc_272C` reaches the end of the active platform-slot walk without finding a `$20(a2)` match",
        );
    }

    #[test]
    fn begin_platform_collision_loc_257e_clears_the_collision_scratch_and_reloads_loop_pointers() {
        assert_eq!(
            begin_platform_collision_loc_257E(0xFFFF_2450),
            PlatformCollisionEntryLoc257E {
                collision_response_word: 0,
                first_platform_slot_pointer_addr: 0xFA8E,
                kid_gfx_object_addr: 0xFFFF_2450,
            },
            "ASM loc_257E (kid.asm:4471) should clear `$FFFFFAA2`, point `a2` at `(Addr_FirstPlatformSlot).w`, and reload `(Addr_GfxObject_Kid).w` into `a0` before the platform scan begins",
        );
    }

    #[test]
    fn load_player_hitbox_loc_2590_preserves_the_current_hitbox_edges_for_platform_scan() {
        assert_eq!(
            load_player_hitbox_loc_2590(0x002C, 0x003A, 0x0070, 0x0086),
            PlayerHitboxLoc2590 {
                left: 0x002C,
                right: 0x003A,
                top: 0x0070,
                bottom: 0x0086,
            },
            "ASM loc_2590 (kid.asm:4478) should copy `Kid_hitbox_left/right/top/bottom` into `d0-d3` unchanged before `loc_25A0` starts testing platform overlap",
        );
    }

    #[test]
    fn platform_collision_return_loc_2720_preserves_collision_type_through_the_epilogue() {
        assert_eq!(
            platform_collision_return_loc_2720(0x0004),
            PlatformCollisionReturnLoc2720 {
                collision_type_word: 0x0004,
                scratch_bytes_released: 8,
            },
            "ASM loc_2720 (kid.asm:4639) should release the 8-byte platform scratch frame and return with the `loc_271A` collision type still set to `$0004`",
        );
    }

    #[test]
    fn horizontal_overlap_resolution_loc_25ee_keeps_the_nearer_exit_distance() {
        assert_eq!(
            horizontal_overlap_resolution_loc_25EE(40, 87, 50, 70),
            HorizontalOverlapResolutionLoc25EE {
                left_exit_px: -31,
                right_exit_px: 38,
                push_x_px: -31,
            },
            "ASM loc_25EE (kid.asm:4513) should compute both horizontal exits and keep the smaller-magnitude push in `d7` before control reaches `loc_261E`",
        );
        assert_eq!(
            horizontal_overlap_resolution_loc_25EE(40, 87, 60, 80),
            HorizontalOverlapResolutionLoc25EE {
                left_exit_px: -41,
                right_exit_px: 28,
                push_x_px: 28,
            },
            "ASM loc_25EE should switch to the right-hand exit when `-(platform_left-player_right-1)` is not less than `platform_right-player_left+1`",
        );
    }

    #[test]
    fn horizontal_overlap_resolution_loc_25ee_ties_choose_the_right_exit() {
        assert_eq!(
            horizontal_overlap_resolution_loc_25EE(40, 59, 50, 49),
            HorizontalOverlapResolutionLoc25EE {
                left_exit_px: -10,
                right_exit_px: 10,
                push_x_px: 10,
            },
            "ASM loc_25EE uses `blt.w loc_261E`, so equal exit magnitudes fall through and keep the right-hand push in `d7`",
        );
    }

    #[test]
    fn horizontal_overlap_right_exit_loc_2616_uses_the_platform_right_edge_only() {
        assert_eq!(
            horizontal_overlap_right_exit_loc_2616(87, 60),
            HorizontalOverlapResolutionLoc2616 { push_x_px: 28 },
            "ASM loc_2616 (kid.asm:4530) should skip the magnitude compare and set `d7 = platform_right-player_left+1` when the player already crosses the platform's right edge",
        );
    }

    #[test]
    fn vertical_overlap_top_exit_loc_261e_uses_the_platform_top_edge_only() {
        assert_eq!(
            vertical_overlap_top_exit_loc_261E(100, 118),
            VerticalOverlapResolutionLoc261E { push_y_px: -19 },
            "ASM loc_261E (kid.asm:4535) should skip the mixed vertical compare and set `d6 = platform_top-player_bottom-1` when the player top is still above the platform",
        );
    }

    #[test]
    fn vertical_overlap_resolution_loc_2632_keeps_the_nearer_vertical_exit_distance() {
        assert_eq!(
            vertical_overlap_resolution_loc_2632(100, 139, 110, 125),
            VerticalOverlapResolutionLoc2632 {
                top_exit_px: -26,
                bottom_exit_px: 30,
                push_y_px: -26,
            },
            "ASM loc_2632 (kid.asm:4544) should keep the upward exit in `d6` when `-(platform_top-player_bottom-1)` is still smaller than `platform_bottom-player_top+1`",
        );
        assert_eq!(
            vertical_overlap_resolution_loc_2632(100, 139, 130, 135),
            VerticalOverlapResolutionLoc2632 {
                top_exit_px: -36,
                bottom_exit_px: 10,
                push_y_px: 10,
            },
            "ASM loc_2632 should switch `d6` to the downward exit once the bottom escape has the smaller magnitude",
        );
    }

    #[test]
    fn vertical_overlap_resolution_loc_2632_ties_choose_the_bottom_exit() {
        assert_eq!(
            vertical_overlap_resolution_loc_2632(100, 129, 120, 109),
            VerticalOverlapResolutionLoc2632 {
                top_exit_px: -10,
                bottom_exit_px: 10,
                push_y_px: 10,
            },
            "ASM loc_2632 uses `blt.w loc_2662`, so equal exit magnitudes fall through and keep the bottom push in `d6`",
        );
    }

    #[test]
    fn vertical_overlap_bottom_exit_loc_265a_uses_the_platform_bottom_edge_only() {
        assert_eq!(
            vertical_overlap_bottom_exit_loc_265A(139, 130),
            VerticalOverlapResolutionLoc265A { push_y_px: 10 },
            "ASM loc_265A (kid.asm:4561) should skip the mixed vertical compare and set `d6 = platform_bottom-player_top+1` when the player already crosses the platform bottom",
        );
    }

    #[test]
    fn vertical_push_magnitude_loc_266a_normalises_negative_pushes_with_word_wraparound() {
        assert_eq!(
            vertical_push_magnitude_loc_266A(-19),
            VerticalPushMagnitudeLoc266A { magnitude_word: 19 },
            "ASM loc_266A (kid.asm:4571) should negate negative `d6` values before `loc_2672` compares absolute push magnitudes",
        );
        assert_eq!(
            vertical_push_magnitude_loc_266A(i16::MIN),
            VerticalPushMagnitudeLoc266A {
                magnitude_word: 0x8000,
            },
            "ASM loc_266A keeps the 16-bit `neg.w` overflow case as `$8000`, so the helper must preserve that wraparound word exactly",
        );
    }

    #[test]
    fn apply_horizontal_platform_push_loc_2686_updates_hitbox_edges_and_branch_direction() {
        assert_eq!(
            apply_horizontal_platform_push_loc_2686(40, 55, 48, -7),
            HorizontalPlatformPushLoc2686 {
                kid_hitbox_left_px: 33,
                kid_hitbox_right_px: 48,
                kid_gfx_x_px: 41,
                takes_negative_branch: true,
            },
            "ASM loc_2686 (kid.asm:4583) should add the signed `d7` push into `Kid_hitbox_left/right` and `$1A(a0)` before `tst.w d7` chooses the `loc_26B6` negative branch",
        );
    }

    #[test]
    fn apply_horizontal_platform_push_loc_2686_zero_push_takes_non_negative_branch() {
        assert_eq!(
            apply_horizontal_platform_push_loc_2686(40, 55, 48, 0),
            HorizontalPlatformPushLoc2686 {
                kid_hitbox_left_px: 40,
                kid_hitbox_right_px: 55,
                kid_gfx_x_px: 48,
                takes_negative_branch: false,
            },
            "ASM loc_2686 uses `bmi.w loc_26B6`, so `d7 == 0` stays on the non-negative loc_26A2 side",
        );
    }

    #[test]
    fn positive_platform_carry_loc_26a2_clamps_negative_velocity_and_restarts_scan() {
        assert_eq!(
            positive_platform_carry_loc_26A2(-0x1_8000, false),
            PositivePlatformCarryLoc26A2 {
                carried_x_velocity_fp: 0,
                touched_right_wall: true,
                escalates_to_loc_271_a: false,
            },
            "ASM loc_26A2 (kid.asm:4593) should clamp negative `$A(a2)` to zero, set `$FFFFFAA3`, and branch back to `loc_2590` when `$FFFFFAA2` is still clear",
        );
    }

    #[test]
    fn negative_platform_carry_loc_26b6_clamps_positive_velocity_and_escalates_on_opposite_wall() {
        assert_eq!(
            negative_platform_carry_loc_26B6(0x2_0000, true),
            NegativePlatformCarryLoc26B6 {
                carried_x_velocity_fp: 0,
                touched_left_wall: true,
                escalates_to_loc_271_a: true,
            },
            "ASM loc_26B6 (kid.asm:4601) should clamp positive `$A(a2)` to zero, set `$FFFFFAA2`, and jump to `loc_271A` once `$FFFFFAA3` is already set",
        );
    }

    #[test]
    fn negative_platform_carry_commit_loc_26c0_sets_the_left_wall_flag_and_only_escalates_after_both_sides_hit()
     {
        assert_eq!(
            negative_platform_carry_commit_loc_26C0(-0x1_0000, false),
            NegativePlatformCarryCommitLoc26C0 {
                carried_x_velocity_fp: -0x1_0000,
                touched_left_wall: true,
                escalates_to_loc_271_a: false,
            },
            "ASM loc_26C0 (kid.asm:4606) should store the chosen negative carry in `$26(a0)`, set `$FFFFFAA2`, and branch back to `loc_2590` while `$FFFFFAA3` is still clear",
        );
        assert_eq!(
            negative_platform_carry_commit_loc_26C0(0, true),
            NegativePlatformCarryCommitLoc26C0 {
                carried_x_velocity_fp: 0,
                touched_left_wall: true,
                escalates_to_loc_271_a: true,
            },
            "ASM loc_26C0 should jump to `loc_271A` once the opposite-side probe has already set `$FFFFFAA3`",
        );
    }

    #[test]
    fn platform_landing_loc_26d4_moves_kid_onto_the_platform_and_records_support() {
        assert_eq!(
            platform_landing_loc_26D4(96, 111, 104, -12, 0xEDBA, false),
            PlatformLandingLoc26D4 {
                kid_hitbox_top_px: 84,
                kid_hitbox_bottom_px: 99,
                kid_gfx_y_px: 92,
                kid_y_velocity_fp: 0,
                standing_platform_addr: 0xEDBA,
                touched_top_face: true,
                escalates_to_loc_271_a: false,
            },
            "ASM loc_26D4 (kid.asm:4614) should apply the signed `d6` lift to `Kid_hitbox_top/bottom` and `$1E(a0)`, clear `y_vel(a3)`, record `(Addr_PlatformStandingOn).w`, and restart the scan while `$FFFFFAA5` is clear",
        );
        assert_eq!(
            platform_landing_loc_26D4(96, 111, 104, -1, 0xEDBA, true),
            PlatformLandingLoc26D4 {
                kid_hitbox_top_px: 95,
                kid_hitbox_bottom_px: 110,
                kid_gfx_y_px: 103,
                kid_y_velocity_fp: 0,
                standing_platform_addr: 0xEDBA,
                touched_top_face: true,
                escalates_to_loc_271_a: true,
            },
            "ASM loc_26D4 should jump to `loc_271A` once the opposite-face path has already set `$FFFFFAA5`",
        );
    }

    #[test]
    fn bottom_platform_carry_loc_26fe_adds_the_bias_and_only_restarts_during_jump() {
        assert_eq!(
            bottom_platform_carry_loc_26FE(0x2_8000, true),
            BottomPlatformCarryLoc26FE {
                carried_y_velocity_fp: 0x2_C000,
                touched_bottom_face: true,
                escalates_to_loc_271_a: false,
            },
            "ASM loc_26FE (kid.asm:4628) should copy `$E(a2)+$4000` into `$2A(a0)`, set `$FFFFFAA5`, and branch back to `loc_2590` while `(Character_Movement).w == MoveID_Jump`",
        );
        assert_eq!(
            bottom_platform_carry_loc_26FE(-0x1_0000, false),
            BottomPlatformCarryLoc26FE {
                carried_y_velocity_fp: -0xC000,
                touched_bottom_face: true,
                escalates_to_loc_271_a: true,
            },
            "ASM loc_26FE should fall through to `loc_271A` for non-jump movement states after setting the bottom-face flag",
        );
    }

    #[test]
    fn initialize_platform_slots_sub_231c_builds_the_free_list_and_clears_state() {
        let init = initialize_platform_slots_sub_231C();

        assert_eq!(
            init.next_free_platform_slot_addr, 0xEDBA,
            "ASM sub_231C stores the first Platform_RAM slot address in `(Addr_NextFreePlatformSlot).w`",
        );
        assert_eq!(
            init.slot_next_addrs[0], 0xEDDC,
            "ASM sub_231C links each slot's leading word to the next $22-byte slot",
        );
        assert_eq!(
            init.slot_next_addrs[15], 0xEFDA,
            "ASM sub_231C uses `dbf #$10` to write links through the seventeenth Platform_RAM slot",
        );
        assert_eq!(
            init.slot_next_addrs[16], 0,
            "ASM sub_231C clears the final slot link to terminate the free list",
        );
        assert_eq!(init.first_platform_slot_addr, 0);
        assert_eq!(init.number_platforms, 0);
        assert_eq!(
            init.platform_loader_offset_words, 0,
            "ASM sub_231C clears `PlatformLoader_Offset` along with the active-list head and platform count",
        );
    }

    #[test]
    fn platform_slot_pool_loc_231c_reuses_a_deallocated_slot_as_next_free_head() {
        let mut pool = PlatformSlotPoolLoc231C::initialize_sub_231C();

        assert_eq!(
            pool.allocate_sub_238E(),
            AllocatePlatformSlotResultSub238E::Allocated(AllocatePlatformSlotSub238E {
                allocated_slot_addr: 0xEDBA,
                next_free_platform_slot_addr: 0xEDDC,
                first_platform_slot_addr: 0xEDBA,
                number_platforms: 1,
            }),
            "ASM sub_238E should pop the initial sub_231C free-list head and make it the active head",
        );
        assert_eq!(
            pool.allocate_sub_238E(),
            AllocatePlatformSlotResultSub238E::Allocated(AllocatePlatformSlotSub238E {
                allocated_slot_addr: 0xEDDC,
                next_free_platform_slot_addr: 0xEDFE,
                first_platform_slot_addr: 0xEDDC,
                number_platforms: 2,
            }),
            "ASM sub_238E should link the second allocation in front of the previous active slot",
        );
        assert_eq!(
            pool.slot_next_addr_sub_231C(0xEDDC),
            Some(0xEDBA),
            "ASM sub_238E writes the previous `(Addr_FirstPlatformSlot).w` into the newly allocated slot's leading word",
        );

        assert_eq!(
            pool.deallocate_sub_23B4(0xEDBA),
            DeallocatePlatformSlotResultSub23B4::Deallocated,
            "ASM loc_23CC should push the deallocated slot onto `(Addr_NextFreePlatformSlot).w`",
        );
        assert_eq!(pool.next_free_platform_slot_addr(), 0xEDBA);
        assert_eq!(
            pool.allocate_sub_238E(),
            AllocatePlatformSlotResultSub238E::Allocated(AllocatePlatformSlotSub238E {
                allocated_slot_addr: 0xEDBA,
                next_free_platform_slot_addr: 0xEDFE,
                first_platform_slot_addr: 0xEDBA,
                number_platforms: 2,
            }),
            "ASM sub_238E should immediately reuse the slot that sub_23B4 just pushed to the free-list head",
        );
        assert_eq!(
            pool.slot_next_addr_sub_231C(0xEDBA),
            Some(0xEDDC),
            "ASM sub_238E should relink the recycled slot in front of the remaining active slot",
        );
    }

    #[test]
    fn platform_slot_pool_loc_23ae_reports_failure_after_all_17_slots_are_allocated() {
        let mut pool = PlatformSlotPoolLoc231C::initialize_sub_231C();

        for expected_index in 0..PLATFORM_SLOT_CAPACITY_LOC_231C {
            let expected_slot_addr =
                PlatformLoaderState::synthetic_slot_addr_loc_231C(expected_index)
                    .expect("test index should be inside the ASM 17-slot Platform_RAM pool");
            assert!(
                matches!(
                    pool.allocate_sub_238E(),
                    AllocatePlatformSlotResultSub238E::Allocated(AllocatePlatformSlotSub238E {
                        allocated_slot_addr,
                        number_platforms,
                        ..
                    }) if allocated_slot_addr == expected_slot_addr
                        && number_platforms == expected_index as u16 + 1
                ),
                "ASM sub_238E should consume Platform_RAM slots in sub_231C free-list order until the seventeenth allocation",
            );
        }

        assert_eq!(
            pool.next_free_platform_slot_addr(),
            0,
            "ASM sub_231C terminates the final free-list link, so the eighteenth allocation sees a zero free-list head",
        );
        assert_eq!(
            pool.allocate_sub_238E(),
            AllocatePlatformSlotResultSub238E::NoFreeSlot {
                failure_label: "loc_6E2",
            },
            "ASM loc_23AE should jump to `loc_6E2` when the 17-slot Platform_RAM free list is exhausted",
        );
    }

    #[test]
    fn platform_slot_pool_active_chain_loc_23b4_relinks_when_deallocating_head() {
        let mut pool = PlatformSlotPoolLoc231C::initialize_sub_231C();
        assert!(matches!(
            pool.allocate_sub_238E(),
            AllocatePlatformSlotResultSub238E::Allocated(_)
        ));
        assert!(matches!(
            pool.allocate_sub_238E(),
            AllocatePlatformSlotResultSub238E::Allocated(_)
        ));
        assert!(matches!(
            pool.allocate_sub_238E(),
            AllocatePlatformSlotResultSub238E::Allocated(_)
        ));
        assert_eq!(
            pool.active_slot_addrs_sub_23B4(),
            vec![0xEDFE, 0xEDDC, 0xEDBA],
            "ASM sub_238E inserts each allocation at `(Addr_FirstPlatformSlot).w`, so active-chain order is newest slot first",
        );

        assert_eq!(
            pool.deallocate_sub_23B4(0xEDFE),
            DeallocatePlatformSlotResultSub23B4::Deallocated,
            "ASM loc_23CC should support the sentinel case where `a4` points at `(Addr_FirstPlatformSlot).w`",
        );
        assert_eq!(
            pool.active_slot_addrs_sub_23B4(),
            vec![0xEDDC, 0xEDBA],
            "ASM loc_23CC copies `0(a3)` into the sentinel when the active-list head is deallocated",
        );
        assert_eq!(
            pool.next_free_platform_slot_addr(),
            0xEDFE,
            "ASM loc_23CC pushes the removed head slot onto the free-list head",
        );
    }

    #[test]
    fn allocate_platform_slot_sub_238e_reports_loc_6e2_when_no_free_slot_exists() {
        assert_eq!(
            allocate_platform_slot_sub_238E(0, 0xEDE0, 0xEE24, 3),
            AllocatePlatformSlotResultSub238E::NoFreeSlot {
                failure_label: "loc_6E2",
            },
            "ASM loc_23AE should jump to `loc_6E2` immediately when `(Addr_NextFreePlatformSlot).w` is zero",
        );
    }

    #[test]
    fn allocate_platform_slot_sub_238e_pops_the_free_head_and_pushes_it_active() {
        assert_eq!(
            allocate_platform_slot_sub_238E(0xEDBA, 0xEDDC, 0xEE24, 3),
            AllocatePlatformSlotResultSub238E::Allocated(AllocatePlatformSlotSub238E {
                allocated_slot_addr: 0xEDBA,
                next_free_platform_slot_addr: 0xEDDC,
                first_platform_slot_addr: 0xEDBA,
                number_platforms: 4,
            }),
            "ASM sub_238E should pop the current free-list head, install it as the new `(Addr_FirstPlatformSlot).w`, and increment `(Number_Platforms).w`",
        );
    }

    #[test]
    fn deallocate_platform_slot_sub_23b4_walks_the_list_then_recycles_the_matched_slot() {
        let mut active_links = [
            PlatformSlotLinkSub23B4 {
                slot_addr: 0xEDBA,
                next_slot_addr: 0xEDDC,
            },
            PlatformSlotLinkSub23B4 {
                slot_addr: 0xEDDC,
                next_slot_addr: 0xEDFE,
            },
            PlatformSlotLinkSub23B4 {
                slot_addr: 0xEDFE,
                next_slot_addr: 0,
            },
        ];
        let mut first_platform_slot_addr = 0xEDBA;
        let mut next_free_platform_slot_addr = 0xEE20;
        let mut number_platforms = 3;

        assert_eq!(
            deallocate_platform_slot_sub_23B4(
                &mut active_links,
                &mut first_platform_slot_addr,
                &mut next_free_platform_slot_addr,
                &mut number_platforms,
                0xEDDC,
            ),
            DeallocatePlatformSlotResultSub23B4::Deallocated,
            "ASM loc_23BA should keep following each active platform slot's word link until the target `a3` is found",
        );
        assert_eq!(first_platform_slot_addr, 0xEDBA);
        assert_eq!(next_free_platform_slot_addr, 0xEDDC);
        assert_eq!(number_platforms, 2);
        assert_eq!(
            active_links,
            [
                PlatformSlotLinkSub23B4 {
                    slot_addr: 0xEDBA,
                    next_slot_addr: 0xEDFE,
                },
                PlatformSlotLinkSub23B4 {
                    slot_addr: 0xEDDC,
                    next_slot_addr: 0xEE20,
                },
                PlatformSlotLinkSub23B4 {
                    slot_addr: 0xEDFE,
                    next_slot_addr: 0,
                },
            ],
            "ASM loc_23CC should splice the matched platform out of the active list, push it onto `(Addr_NextFreePlatformSlot).w`, and leave the rest of the chain intact",
        );
    }

    #[test]
    fn deallocate_platform_slot_sub_23b4_jumps_to_loc_6e2_when_the_target_is_missing() {
        let mut active_links = [PlatformSlotLinkSub23B4 {
            slot_addr: 0xEDBA,
            next_slot_addr: 0,
        }];
        let mut first_platform_slot_addr = 0xEDBA;
        let mut next_free_platform_slot_addr = 0xEE20;
        let mut number_platforms = 1;

        assert_eq!(
            deallocate_platform_slot_sub_23B4(
                &mut active_links,
                &mut first_platform_slot_addr,
                &mut next_free_platform_slot_addr,
                &mut number_platforms,
                0xEDDC,
            ),
            DeallocatePlatformSlotResultSub23B4::MissingSlot {
                failure_label: "loc_6E2",
            },
            "ASM loc_23BA should fall through to loc_23E4's `jmp loc_6E2` once the active platform-slot walk reaches a zero link without finding the target",
        );
    }

    #[test]
    fn resolve_duplicate_platform_load_loc_286a_deallocates_scripted_slots_but_marks_special_ones()
    {
        assert_eq!(
            crate::platforms::resolve_duplicate_platform_load_loc_286A(0x02),
            crate::platforms::DuplicatePlatformResolutionLoc286A {
                deallocate_existing_slot: true,
                mark_special_platform_loaded: false,
            },
            "ASM loc_286A (kid.asm:4782) should deallocate a duplicate scripted platform slot when `$1F(a3)` is non-negative",
        );
        assert_eq!(
            crate::platforms::resolve_duplicate_platform_load_loc_286A(0x80),
            crate::platforms::DuplicatePlatformResolutionLoc286A {
                deallocate_existing_slot: false,
                mark_special_platform_loaded: true,
            },
            "ASM loc_2850 should skip loc_286A for negative `$1F(a3)` kinds and execute `st $12(a3)` to mark the special platform as already loaded",
        );
    }

    #[test]
    fn advance_platform_loader_offset_loc_286e_steps_one_12_byte_layout_record() {
        assert_eq!(
            crate::platforms::advance_platform_loader_offset_loc_286E(0x0000),
            0x000C,
            "ASM loc_286E (kid.asm:4785) should advance `(PlatformLoader_Offset).w` by one 12-byte platform record after each placement",
        );
        assert_eq!(
            crate::platforms::advance_platform_loader_offset_loc_286E(0xFFF8),
            0x0004,
            "ASM loc_286E uses `addi.w`, so the 16-bit platform-loader offset must wrap on overflow",
        );
    }

    #[test]
    fn platform_loader_cursor_step_loc_2744_processes_exactly_one_layout_record() {
        assert_eq!(
            crate::platforms::platform_loader_cursor_step_loc_2744(4, 0x000C),
            crate::platforms::PlatformLoaderCursorStepLoc2744 {
                current_index: Some(1),
                current_offset_words: 0x000C,
                next_offset_words: 0x0018,
                wrapped_to_start: false,
            },
            "ASM Manage_PlatformLoading loc_2744 reads only the current 12-byte platform record before loc_286E advances the cursor by `$C`",
        );
    }

    #[test]
    fn platform_loader_cursor_step_loc_2744_wraps_at_generated_terminator() {
        assert_eq!(
            crate::platforms::platform_loader_cursor_step_loc_2744(3, 0x0024),
            crate::platforms::PlatformLoaderCursorStepLoc2744 {
                current_index: Some(0),
                current_offset_words: 0x0000,
                next_offset_words: 0x000C,
                wrapped_to_start: true,
            },
            "ASM Manage_PlatformLoading (kid.asm:4677-4681) clears PlatformLoader_Offset and retries when the current layout word is the `$FFFF` terminator",
        );
    }

    #[test]
    fn platform_loader_cursor_step_loc_2744_does_not_advance_without_layout() {
        assert_eq!(
            crate::platforms::platform_loader_cursor_step_loc_2744(0, 0x0018),
            crate::platforms::PlatformLoaderCursorStepLoc2744 {
                current_index: None,
                current_offset_words: 0x0018,
                next_offset_words: 0x0018,
                wrapped_to_start: false,
            },
            "ASM Manage_PlatformLoading returns before loc_286E when Addr_PlatformLayout is zero, so an empty native layout should not advance the cursor",
        );
    }

    #[test]
    fn platform_sprite_dispatch_loc_244c_biases_visible_platforms_and_selects_loc_24c0() {
        assert_eq!(
            platform_sprite_dispatch_loc_244C(PlatformSpriteDispatchInputLoc244C {
                world_x_px: 0x0120,
                world_y_px: 0x0090,
                camera_x_px: 0x0100,
                camera_y_px: 0x0080,
                width_px: 0x002F,
                height_px: 0x001F,
                mapping_addr_word: 0x2468,
                render_flags: 8,
            }),
            Some(PlatformSpriteDispatchLoc244C {
                screen_x_word: 0x00A0,
                screen_y_word: 0x0090,
                width_tiles_minus_one: 2,
                height_tiles_minus_one: 1,
                base_tile_word: 0x0500,
                mapping_addr_word: 0x2468,
                variant: PlatformSpriteVariantLoc244C::Loc24C0,
            }),
            "ASM loc_244C should bias visible platform coordinates by +$80, derive tile counts from `$1A/$1C >> 4`, seed `d2=#$500`, and dispatch through `off_24AC` using even `$1F(a2)` bytes after `bclr #7,d3`",
        );
    }

    #[test]
    fn platform_sprite_dispatch_loc_244c_culls_platforms_whose_right_edge_is_left_of_camera() {
        assert_eq!(
            platform_sprite_dispatch_loc_244C(PlatformSpriteDispatchInputLoc244C {
                world_x_px: 0x0070,
                world_y_px: 0x0090,
                camera_x_px: 0x0100,
                camera_y_px: 0x0080,
                width_px: 0x0010,
                height_px: 0x001F,
                mapping_addr_word: 0x1357,
                render_flags: 0,
            }),
            None,
            "ASM loc_244C should skip the platform and loop back immediately when `d7 + $1A(a2)` stays negative after subtracting `Camera_X_pos`",
        );
    }

    #[test]
    fn platform_sprite_dispatch_loc_244c_keeps_exact_zero_trailing_edges_visible() {
        let exact_left_edge =
            platform_sprite_dispatch_loc_244C(PlatformSpriteDispatchInputLoc244C {
                world_x_px: 0x00F0,
                world_y_px: 0x0090,
                camera_x_px: 0x0100,
                camera_y_px: 0x0080,
                width_px: 0x0010,
                height_px: 0x001F,
                mapping_addr_word: 0x1357,
                render_flags: 0,
            });
        assert!(
            exact_left_edge.is_some(),
            "ASM loc_244C uses `bmi`, so `relative_x + width == 0` stays visible",
        );

        let exact_top_edge =
            platform_sprite_dispatch_loc_244C(PlatformSpriteDispatchInputLoc244C {
                world_x_px: 0x0120,
                world_y_px: 0x0070,
                camera_x_px: 0x0100,
                camera_y_px: 0x0080,
                width_px: 0x002F,
                height_px: 0x0010,
                mapping_addr_word: 0x1357,
                render_flags: 0,
            });
        assert!(
            exact_top_edge.is_some(),
            "ASM loc_244C uses `bmi`, so `relative_y + height == 0` stays visible",
        );
    }

    #[test]
    fn platform_sprite_dispatch_loc_244c_selects_every_off_24ac_arm() {
        let expected_variants = [
            (0, PlatformSpriteVariantLoc244C::Loc24CC),
            (2, PlatformSpriteVariantLoc244C::Loc24F0),
            (4, PlatformSpriteVariantLoc244C::Loc251C),
            (6, PlatformSpriteVariantLoc244C::Loc24D4),
            (8, PlatformSpriteVariantLoc244C::Loc24C0),
        ];

        for (render_flags, expected_variant) in expected_variants {
            let result = platform_sprite_dispatch_loc_244C(PlatformSpriteDispatchInputLoc244C {
                world_x_px: 0x0120,
                world_y_px: 0x0090,
                camera_x_px: 0x0100,
                camera_y_px: 0x0080,
                width_px: 0x002F,
                height_px: 0x001F,
                mapping_addr_word: 0x2468,
                render_flags,
            })
            .expect("visible platform should dispatch through off_24AC");

            assert_eq!(
                result.variant, expected_variant,
                "ASM loc_244C should dispatch even render byte {render_flags} through the off_24AC long-pointer table",
            );
        }

        assert_eq!(
            platform_sprite_dispatch_loc_244C(PlatformSpriteDispatchInputLoc244C {
                world_x_px: 0x0120,
                world_y_px: 0x0090,
                camera_x_px: 0x0100,
                camera_y_px: 0x0080,
                width_px: 0x002F,
                height_px: 0x001F,
                mapping_addr_word: 0x2468,
                render_flags: 5,
            }),
            None,
            "native keeps off_24AC bounded to the five even-byte entries present at ASM loc_244C; shipped data avoids odd bytes that would land inside a long pointer",
        );
    }

    #[test]
    fn platform_single_sprite_loc_24c0_emits_one_400_series_platform_sprite() {
        assert_eq!(
            platform_single_sprite_loc_24C0(
                PlatformSpriteDispatchLoc244C {
                    screen_x_word: 0x00A0,
                    screen_y_word: 0x0090,
                    width_tiles_minus_one: 2,
                    height_tiles_minus_one: 1,
                    base_tile_word: 0x0500,
                    mapping_addr_word: 0x2468,
                    variant: PlatformSpriteVariantLoc244C::Loc24C0,
                },
                0x12,
            ),
            PlatformSpriteEmitLoc2548 {
                sprite: PlatformSpriteEntryLoc2548 {
                    y_word: 0x0090,
                    tile_word: 0x0413,
                    mapping_word: 0xA468,
                    x_word: 0x00A0,
                },
                next_number_sprites: 0x13,
            },
            "ASM loc_24C0 should override `d2` to `#$400` and emit exactly one sprite via loc_2548, where the incremented `Number_Sprites` low byte is folded into the tile word",
        );
    }

    #[test]
    fn finish_platform_sprite_list_loc_256e_publishes_the_post_incremented_sprite_pointer() {
        assert_eq!(
            finish_platform_sprite_list_loc_256E(0xFFFF_0000 + (3 * 8)),
            0xFFFF_0018,
            "ASM loc_256E (kid.asm:4456) should store the post-incremented `a0` back into `Addr_NextSpriteSlot`, so three emitted 8-byte sprites advance the free pointer by `$18`",
        );
    }

    #[test]
    fn platform_strip_sprites_loc_24cc_emits_one_row_of_700_series_cells() {
        assert_eq!(
            platform_strip_sprites_loc_24CC(
                PlatformSpriteDispatchLoc244C {
                    screen_x_word: 0x00A0,
                    screen_y_word: 0x0090,
                    width_tiles_minus_one: 2,
                    height_tiles_minus_one: 1,
                    base_tile_word: 0x0500,
                    mapping_addr_word: 0x2468,
                    variant: PlatformSpriteVariantLoc244C::Loc24CC,
                },
                0,
            ),
            PlatformSpriteStripLoc24CC {
                sprites: vec![
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x008E,
                        tile_word: 0x0701,
                        mapping_word: 0xA468,
                        x_word: 0x00A0,
                    },
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x008E,
                        tile_word: 0x0702,
                        mapping_word: 0xA468,
                        x_word: 0x00B0,
                    },
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x008E,
                        tile_word: 0x0703,
                        mapping_word: 0xA468,
                        x_word: 0x00C0,
                    },
                ],
                next_number_sprites: 3,
            },
            "ASM loc_24CC should subtract 2 from `d6`, seed `d2=#$700`, and enter loc_24D4/loc_24D8 for exactly one row of `d0 + 1` 16px-wide cells when `d1` is cleared first",
        );
    }

    #[test]
    fn platform_strip_sprites_loc_24f0_emits_the_left_cap_before_advancing_mapping_and_x() {
        assert_eq!(
            platform_strip_sprites_loc_24F0(
                PlatformSpriteDispatchLoc244C {
                    screen_x_word: 0x00A0,
                    screen_y_word: 0x0090,
                    width_tiles_minus_one: 2,
                    height_tiles_minus_one: 0,
                    base_tile_word: 0x0500,
                    mapping_addr_word: 0x2468,
                    variant: PlatformSpriteVariantLoc244C::Loc24F0,
                },
                0,
            ),
            PlatformSpriteStripLoc24F0 {
                sprites: vec![
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x0090,
                        tile_word: 0x0501,
                        mapping_word: 0xA468,
                        x_word: 0x00A0,
                    },
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x0090,
                        tile_word: 0x0502,
                        mapping_word: 0xA46C,
                        x_word: 0x00B0,
                    },
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x0090,
                        tile_word: 0x0503,
                        mapping_word: 0xA470,
                        x_word: 0x00C0,
                    },
                ],
                next_number_sprites: 3,
            },
            "ASM loc_24F0 should emit the left cap first with the original mapping word, then advance `a4` and `d4` before the remaining width handling runs",
        );
    }

    #[test]
    fn platform_strip_sprites_loc_2506_repeats_the_middle_mapping_word_across_interior_tiles() {
        let result = platform_strip_sprites_loc_24F0(
            PlatformSpriteDispatchLoc244C {
                screen_x_word: 0x00A0,
                screen_y_word: 0x0090,
                width_tiles_minus_one: 4,
                height_tiles_minus_one: 0,
                base_tile_word: 0x0500,
                mapping_addr_word: 0x2468,
                variant: PlatformSpriteVariantLoc244C::Loc24F0,
            },
            0x10,
        );

        assert_eq!(
            result.sprites[1..4],
            [
                PlatformSpriteEntryLoc2548 {
                    y_word: 0x0090,
                    tile_word: 0x0512,
                    mapping_word: 0xA46C,
                    x_word: 0x00B0,
                },
                PlatformSpriteEntryLoc2548 {
                    y_word: 0x0090,
                    tile_word: 0x0513,
                    mapping_word: 0xA46C,
                    x_word: 0x00C0,
                },
                PlatformSpriteEntryLoc2548 {
                    y_word: 0x0090,
                    tile_word: 0x0514,
                    mapping_word: 0xA46C,
                    x_word: 0x00D0,
                },
            ],
            "ASM loc_2506 should keep reusing the middle mapping word while `dbf d1,loc_2506` walks the interior span one 16px cell at a time",
        );
    }

    #[test]
    fn platform_strip_sprites_loc_2512_skips_the_middle_loop_for_a_two_tile_span() {
        assert_eq!(
            platform_strip_sprites_loc_24F0(
                PlatformSpriteDispatchLoc244C {
                    screen_x_word: 0x00A0,
                    screen_y_word: 0x0090,
                    width_tiles_minus_one: 1,
                    height_tiles_minus_one: 0,
                    base_tile_word: 0x0500,
                    mapping_addr_word: 0x2468,
                    variant: PlatformSpriteVariantLoc244C::Loc24F0,
                },
                0x20,
            ),
            PlatformSpriteStripLoc24F0 {
                sprites: vec![
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x0090,
                        tile_word: 0x0521,
                        mapping_word: 0xA468,
                        x_word: 0x00A0,
                    },
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x0090,
                        tile_word: 0x0522,
                        mapping_word: 0xA470,
                        x_word: 0x00B0,
                    },
                ],
                next_number_sprites: 0x22,
            },
            "ASM loc_2512 should branch around loc_2506 for a two-tile span and emit the final sprite with the right-cap mapping word after the second `addq.w #4,a4`",
        );
    }

    #[test]
    fn platform_strip_sprites_loc_251c_emits_the_top_cap_before_advancing_mapping_and_y() {
        assert_eq!(
            platform_strip_sprites_loc_251C(
                PlatformSpriteDispatchLoc244C {
                    screen_x_word: 0x00A0,
                    screen_y_word: 0x0090,
                    width_tiles_minus_one: 0,
                    height_tiles_minus_one: 2,
                    base_tile_word: 0x0500,
                    mapping_addr_word: 0x2468,
                    variant: PlatformSpriteVariantLoc244C::Loc251C,
                },
                0,
            ),
            PlatformSpriteStripLoc24CC {
                sprites: vec![
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x0090,
                        tile_word: 0x0501,
                        mapping_word: 0xA468,
                        x_word: 0x00A0,
                    },
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x00A0,
                        tile_word: 0x0502,
                        mapping_word: 0xA46C,
                        x_word: 0x00A0,
                    },
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x00B0,
                        tile_word: 0x0503,
                        mapping_word: 0xA470,
                        x_word: 0x00A0,
                    },
                ],
                next_number_sprites: 3,
            },
            "ASM loc_251C should emit the top cap first with the original mapping word, then advance `a4` and `d6` before the remaining height handling runs",
        );
    }

    #[test]
    fn platform_strip_sprites_loc_2532_repeats_the_middle_mapping_word_across_interior_rows() {
        let result = platform_strip_sprites_loc_251C(
            PlatformSpriteDispatchLoc244C {
                screen_x_word: 0x00A0,
                screen_y_word: 0x0090,
                width_tiles_minus_one: 0,
                height_tiles_minus_one: 4,
                base_tile_word: 0x0500,
                mapping_addr_word: 0x2468,
                variant: PlatformSpriteVariantLoc244C::Loc251C,
            },
            0x10,
        );

        assert_eq!(
            result.sprites[1..4],
            [
                PlatformSpriteEntryLoc2548 {
                    y_word: 0x00A0,
                    tile_word: 0x0512,
                    mapping_word: 0xA46C,
                    x_word: 0x00A0,
                },
                PlatformSpriteEntryLoc2548 {
                    y_word: 0x00B0,
                    tile_word: 0x0513,
                    mapping_word: 0xA46C,
                    x_word: 0x00A0,
                },
                PlatformSpriteEntryLoc2548 {
                    y_word: 0x00C0,
                    tile_word: 0x0514,
                    mapping_word: 0xA46C,
                    x_word: 0x00A0,
                },
            ],
            "ASM loc_2532 should keep reusing the middle mapping word while `dbf d0,loc_2532` walks the interior span one 16px row at a time",
        );
    }

    #[test]
    fn platform_strip_sprites_loc_253e_skips_the_middle_loop_for_a_two_tile_column() {
        assert_eq!(
            platform_strip_sprites_loc_251C(
                PlatformSpriteDispatchLoc244C {
                    screen_x_word: 0x00A0,
                    screen_y_word: 0x0090,
                    width_tiles_minus_one: 0,
                    height_tiles_minus_one: 1,
                    base_tile_word: 0x0500,
                    mapping_addr_word: 0x2468,
                    variant: PlatformSpriteVariantLoc244C::Loc251C,
                },
                0x20,
            ),
            PlatformSpriteStripLoc24CC {
                sprites: vec![
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x0090,
                        tile_word: 0x0521,
                        mapping_word: 0xA468,
                        x_word: 0x00A0,
                    },
                    PlatformSpriteEntryLoc2548 {
                        y_word: 0x00A0,
                        tile_word: 0x0522,
                        mapping_word: 0xA470,
                        x_word: 0x00A0,
                    },
                ],
                next_number_sprites: 0x22,
            },
            "ASM loc_253E should branch around loc_2532 for a two-tile column and emit the final sprite with the bottom-cap mapping word after the second `addq.w #4,a4`",
        );
    }

    // ---------------------------------------------------------------------------
    // SoftPlatform / TrapPlatformDown / TrapPlatformUp script routine tests
    // ---------------------------------------------------------------------------

    #[test]
    fn soft_platform_settled_step_loc_4160_stays_settled_when_kid_stands_on_this_platform() {
        // ASM loc_4160 (kid.asm:5242): cmp.w a3,d7 matches → bra.s loc_4160
        assert_eq!(
            soft_platform_settled_step_loc_4160(0x1234, 0x1234),
            SoftPlatformSettledActionLoc4160::StaySettled,
            "when Addr_PlatformStandingOn equals this platform's address, the ROM loops back to loc_4160",
        );
    }

    #[test]
    fn soft_platform_settled_step_loc_4160_returns_to_wait_when_no_platform() {
        // ASM loc_4160: beq.s SoftPlatform_WaitLoop (zero → Kid stepped off)
        assert_eq!(
            soft_platform_settled_step_loc_4160(0x0000, 0x1234),
            SoftPlatformSettledActionLoc4160::ReturnToWaitLoop,
            "zero Addr_PlatformStandingOn means Kid is not standing on any platform",
        );
    }

    #[test]
    fn soft_platform_settled_step_loc_4160_returns_to_wait_when_different_platform() {
        // ASM loc_4160: cmp.w a3,d7; bne.s SoftPlatform_WaitLoop
        assert_eq!(
            soft_platform_settled_step_loc_4160(0x5678, 0x1234),
            SoftPlatformSettledActionLoc4160::ReturnToWaitLoop,
            "Kid standing on a different platform should return to SoftPlatform_WaitLoop",
        );
    }

    #[test]
    fn soft_platform_launch_velocity_loc_40e2_scales_by_current_helmet() {
        // ASM SoftPlatform_Init loc_40E2 (kid.asm:5201): Iron Knight keeps
        // Kid y velocity, normal helmets divide by 2, Micromax divides by 4.
        assert_eq!(
            soft_platform_launch_velocity_loc_40E2(0x0004_0000, PlayerForm::IronKnight),
            0x0004_0000,
        );
        assert_eq!(
            soft_platform_launch_velocity_loc_40E2(0x0004_0000, PlayerForm::TheKid),
            0x0002_0000,
        );
        assert_eq!(
            soft_platform_launch_velocity_loc_40E2(0x0004_0000, PlayerForm::Micromax),
            0x0001_0000,
        );
    }

    #[test]
    fn soft_platform_wait_loop_loc_40e2_samples_pre_hibernate_kid_y_velocity() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.progression.current_form = PlayerForm::TheKid;

        let mut platform = ActivePlatform::from_placement(
            &PlatformPlacement {
                x: 64,
                y: 128,
                buffer_left: 25,
                buffer_right: 75,
                buffer_top: 0,
                buffer_bottom: 14,
                kind: 8,
                subtype: 2,
                h_variant: 1,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("0"),
                    relative_offset: None,
                },
            },
            None,
        );
        platform.slot_addr = Some(0xEDBA);
        platform.layout_index = Some(0);
        runtime.active_platforms = vec![platform];
        runtime.active_platform_standing_on = Some(0xEDBA);

        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = platform.x_pos + (16 << 16);
        slice.player.y_pos = platform.y_pos;
        slice.player.on_ground = true;
        slice.player.y_velocity = 0x0008_0000;
        slice.pre_gravity_y_vel = 0x0004_0000;

        // ASM SoftPlatform_WaitLoop loc_40E2 (kid.asm:5206): Kid y velocity is
        // copied into d1 before the hibernate frame; native stores that sample
        // in pre_gravity_y_vel, while player.y_velocity is already post-physics.
        runtime.trigger_standing_soft_platform_bounce_loc_40E2(&slice, false);

        assert_eq!(
            runtime.active_platforms[0].y_velocity, 0x0002_0000,
            "TheKid divides the pre-hibernate sample by 2; using post-physics y_velocity would yield 0x0004_0000",
        );
        assert_eq!(
            runtime.active_platforms[0].special,
            Some(PlatformSpecialState::Soft(SoftPlatformState {
                original_y_pos: platform.y_pos,
                phase: SoftPlatformPhase::Bounce,
            })),
        );
    }

    #[test]
    fn soft_platform_bounce_loop_loc_4140_applies_upward_acceleration_and_clamp() {
        // ASM SoftPlatform_BounceLoop loc_4140 (kid.asm:5225): subtract
        // `$7000`, then clamp any value <= -$18000 to exactly -$18000.
        assert_eq!(soft_platform_next_bounce_velocity_loc_4140(0), -0x7000);
        assert_eq!(
            soft_platform_next_bounce_velocity_loc_4140(-0x1_7000),
            -0x1_8000,
        );
        assert_eq!(
            soft_platform_next_bounce_velocity_loc_4140(-0x1_8000),
            -0x1_8000,
        );
    }

    #[test]
    fn soft_platform_special_state_loc_40e2_moves_platform_object_not_player_directly() {
        let mut platform = ActivePlatform::from_placement(
            &PlatformPlacement {
                x: 64,
                y: 128,
                buffer_left: 25,
                buffer_right: 75,
                buffer_top: 0,
                buffer_bottom: 14,
                kind: 8,
                subtype: 2,
                h_variant: 1,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("0"),
                    relative_offset: None,
                },
            },
            None,
        );

        assert_eq!(
            platform.special,
            Some(PlatformSpecialState::Soft(SoftPlatformState {
                original_y_pos: (128u32 << 4) << 16,
                phase: SoftPlatformPhase::Wait,
            })),
            "ASM loc_2876/off_40D2 (kid.asm:4799/5195): special platform script word 0 dispatches to SoftPlatform_Init",
        );

        platform.trigger_soft_platform_bounce_loc_40E2(0x0004_0000, PlayerForm::TheKid);
        assert_eq!(platform.y_velocity, 0x0002_0000);
        assert_eq!(
            platform.special,
            Some(PlatformSpecialState::Soft(SoftPlatformState {
                original_y_pos: (128u32 << 4) << 16,
                phase: SoftPlatformPhase::Bounce,
            })),
        );

        platform.step_special_platform_loc_40D2(true, 0, 0);
        assert_eq!(
            platform.y_pos,
            ((128u32 << 4) + 2) << 16,
            "ASM SoftPlatform_Move adds platform `$E(a3)` into platform `6(a3)`; Kid is carried later by platform collision",
        );
        assert_eq!(platform.y_velocity, 0x0001_9000);

        for _ in 0..16 {
            platform.step_special_platform_loc_40D2(true, 0, 0);
        }
        assert_eq!(platform.y_pos, (128u32 << 4) << 16);
        assert_eq!(platform.y_velocity, 0);
        assert_eq!(
            platform.special,
            Some(PlatformSpecialState::Soft(SoftPlatformState {
                original_y_pos: (128u32 << 4) << 16,
                phase: SoftPlatformPhase::Settled,
            })),
        );
    }

    #[test]
    fn special_platform_dispatch_loc_2876_uses_off_40d2_indices() {
        let trap_down = ActivePlatform::from_placement(
            &PlatformPlacement {
                x: 64,
                y: 128,
                buffer_left: 25,
                buffer_right: 75,
                buffer_top: 0,
                buffer_bottom: 14,
                kind: 8,
                subtype: 2,
                h_variant: 1,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("4"),
                    relative_offset: None,
                },
            },
            None,
        );
        assert_eq!(
            trap_down.special,
            Some(PlatformSpecialState::TrapDown(TrapPlatformDownState {
                phase: TrapPlatformDownPhase::Wait,
            })),
            "ASM loc_2876/off_40D2 (kid.asm:4799/5195): pp=4 dispatches to TrapPlatformDown_Init",
        );

        let trap_up = ActivePlatform::from_placement(
            &PlatformPlacement {
                x: 64,
                y: 128,
                buffer_left: 25,
                buffer_right: 75,
                buffer_top: 0,
                buffer_bottom: 14,
                kind: 8,
                subtype: 2,
                h_variant: 1,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("8"),
                    relative_offset: None,
                },
            },
            None,
        );
        assert_eq!(
            trap_up.special,
            Some(PlatformSpecialState::TrapUp(TrapPlatformUpState {
                phase: TrapPlatformUpPhase::Wait,
            })),
            "ASM loc_2876/off_40D2: pp=8 dispatches to TrapPlatformUp_Init",
        );

        let chain = ActivePlatform::from_placement_with_index(
            3,
            &PlatformPlacement {
                x: 64,
                y: 128,
                buffer_left: 25,
                buffer_right: 75,
                buffer_top: 0,
                buffer_bottom: 14,
                kind: 8,
                subtype: 2,
                h_variant: 1,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("12"),
                    relative_offset: None,
                },
            },
            None,
        );
        assert_eq!(
            chain.special,
            Some(PlatformSpecialState::Chain(PlatformChainState {
                source_layout_index: Some(3),
                segment_index: 0,
                phase: PlatformChainPhase::Down,
                frames_remaining: PLATFORM_CHAIN_INITIAL_DOWN_FRAMES_LOC_428C,
            })),
            "ASM loc_2876/off_40D2 (kid.asm:4799/5195): pp=12 dispatches to PlatformChain_Init",
        );
    }

    #[test]
    fn platform_chain_y_velocity_table_loc_436e_matches_rom_segments() {
        // ASM PlatformChain_YVelocities loc_436E (kid.asm:5492): seven
        // FP16.16 y velocities are assigned to the loaded chain slots.
        let down: Vec<i32> = (0..PLATFORM_CHAIN_SEGMENT_COUNT_LOC_428C)
            .map(|segment| platform_chain_y_velocity_loc_436e(segment, PlatformChainPhase::Down))
            .collect();
        assert_eq!(
            down,
            vec![
                0x0000_4000,
                0x0000_8000,
                0x0001_0000,
                0x0001_4000,
                0x0001_0000,
                0x0000_8000,
                0x0000_4000,
            ],
        );
        let up: Vec<i32> = (0..PLATFORM_CHAIN_SEGMENT_COUNT_LOC_428C)
            .map(|segment| platform_chain_y_velocity_loc_436e(segment, PlatformChainPhase::Up))
            .collect();
        assert_eq!(
            up,
            down.into_iter()
                .map(|velocity| -velocity)
                .collect::<Vec<_>>(),
            "ASM PlatformChain_SetMoveUp loc_42EE negates the loc_436E table",
        );
    }

    #[test]
    fn platform_chain_expand_loc_428c_loads_six_offset_segments() {
        let parent = ActivePlatform::from_placement_with_index(
            7,
            &PlatformPlacement {
                x: 10,
                y: 20,
                buffer_left: 0,
                buffer_right: 255,
                buffer_top: 0,
                buffer_bottom: 255,
                kind: 8,
                subtype: 2,
                h_variant: 1,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("12"),
                    relative_offset: None,
                },
            },
            None,
        );
        let mut platforms = vec![parent];

        expand_platform_chain_segments_loc_428C(&mut platforms, &[]);

        assert_eq!(platforms.len(), 7);
        for (segment_index, platform) in platforms.iter().enumerate() {
            assert_eq!(
                platform.x_pos,
                parent.x_pos + ((u32::from(parent.width_px + 1) * segment_index as u32) << 16),
                "ASM PlatformChain_Init loc_428C adds horizontal size + 1 before each PlatformChain_LoadPlatform",
            );
            assert_eq!(platform.y_pos, parent.y_pos);
            assert_eq!(
                platform.y_velocity,
                platform_chain_y_velocity_loc_436e(segment_index as u8, PlatformChainPhase::Down),
            );
            assert_eq!(
                platform.special,
                Some(PlatformSpecialState::Chain(PlatformChainState {
                    source_layout_index: Some(7),
                    segment_index: segment_index as u8,
                    phase: PlatformChainPhase::Down,
                    frames_remaining: PLATFORM_CHAIN_INITIAL_DOWN_FRAMES_LOC_428C,
                })),
            );
        }
    }

    #[test]
    fn platform_chain_expand_loc_428c_preserves_previous_child_state() {
        let parent = ActivePlatform::from_placement_with_index(
            7,
            &PlatformPlacement {
                x: 10,
                y: 20,
                buffer_left: 0,
                buffer_right: 255,
                buffer_top: 0,
                buffer_bottom: 255,
                kind: 8,
                subtype: 2,
                h_variant: 1,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("12"),
                    relative_offset: None,
                },
            },
            None,
        );
        let mut previous_child = parent;
        previous_child.layout_index = None;
        previous_child.y_pos += 3 << 16;
        previous_child.special = Some(PlatformSpecialState::Chain(PlatformChainState {
            source_layout_index: Some(7),
            segment_index: 2,
            phase: PlatformChainPhase::Up,
            frames_remaining: 42,
        }));
        previous_child.y_velocity = platform_chain_y_velocity_loc_436e(2, PlatformChainPhase::Up);
        let mut platforms = vec![parent];

        expand_platform_chain_segments_loc_428C(&mut platforms, &[previous_child]);

        assert_eq!(
            platforms[2], previous_child,
            "Native rebuilds active platforms from the layout each frame; chain child slots must survive by ASM source layout and segment index",
        );
    }

    #[test]
    fn platform_chain_step_loc_42c0_switches_down_to_up_after_initial_span() {
        let mut platform = ActivePlatform::from_placement_with_index(
            7,
            &PlatformPlacement {
                x: 10,
                y: 20,
                buffer_left: 0,
                buffer_right: 255,
                buffer_top: 0,
                buffer_bottom: 255,
                kind: 8,
                subtype: 2,
                h_variant: 1,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("12"),
                    relative_offset: None,
                },
            },
            None,
        );

        for _ in 0..PLATFORM_CHAIN_INITIAL_DOWN_FRAMES_LOC_428C {
            platform.step_special_platform_loc_40D2(false, 0, 0);
        }

        assert_eq!(
            platform.special,
            Some(PlatformSpecialState::Chain(PlatformChainState {
                source_layout_index: Some(7),
                segment_index: 0,
                phase: PlatformChainPhase::Up,
                frames_remaining: PLATFORM_CHAIN_LOOP_FRAMES_LOC_42C0,
            })),
            "ASM PlatformChain_Init loc_42C0 loads d0=$3C; DBF gives 61 moved frames before PlatformChain_SetMoveUp",
        );
        assert_eq!(
            platform.y_velocity,
            platform_chain_y_velocity_loc_436e(0, PlatformChainPhase::Up),
        );
    }

    #[test]
    fn trap_platform_down_loc_4182_launches_from_kid_velocity_and_falls() {
        assert_eq!(
            trap_platform_down_launch_velocity_loc_4182(0x0008_0000),
            0x0002_0000,
            "ASM TrapPlatformDown_WaitLoop loc_4182 divides sampled Kid y velocity by 4",
        );

        let mut platform = ActivePlatform::from_placement(
            &PlatformPlacement {
                x: 64,
                y: 128,
                buffer_left: 25,
                buffer_right: 75,
                buffer_top: 0,
                buffer_bottom: 14,
                kind: 8,
                subtype: 2,
                h_variant: 1,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("4"),
                    relative_offset: None,
                },
            },
            None,
        );

        platform.trigger_special_platform_standing_loc_40D2(0x0008_0000, PlayerForm::TheKid);
        assert_eq!(platform.y_velocity, 0x0002_0000);
        assert_eq!(
            platform.special,
            Some(PlatformSpecialState::TrapDown(TrapPlatformDownState {
                phase: TrapPlatformDownPhase::Falling,
            })),
        );

        platform.step_special_platform_loc_40D2(true, (platform.y_pos >> 16) as i16, 0);
        assert_eq!(
            platform.y_pos,
            ((128u32 << 4) + 2) << 16,
            "ASM TrapPlatformDown_Move adds platform `$E(a3)` into platform `6(a3)`",
        );
        assert_eq!(platform.y_velocity, 0x0002_4000);
    }

    #[test]
    fn trap_platform_down_loc_4182_preserves_original_velocity_clamp_bug() {
        let result = trap_platform_down_move_step_loc_4182(0x0003_C000, 128, 0);
        assert_eq!(
            result.new_y_velocity, 0x0002_0000,
            "ORIGINAL_QUIRK: shipped ROM clamps >= $40000 to $20000, not the commented bug-fix $40000",
        );
        assert!(
            result.continue_moving,
            "platform_y - Kid_y = 128, still less than $E0",
        );
    }

    #[test]
    fn trap_platform_up_loc_420a_enters_rise_loop_when_kid_stands_on_it() {
        let mut platform = ActivePlatform::from_placement(
            &PlatformPlacement {
                x: 64,
                y: 128,
                buffer_left: 25,
                buffer_right: 75,
                buffer_top: 0,
                buffer_bottom: 14,
                kind: 8,
                subtype: 2,
                h_variant: 1,
                v_variant: 0,
                script: PlatformScriptRef {
                    raw: String::from("8"),
                    relative_offset: None,
                },
            },
            None,
        );

        platform.trigger_special_platform_standing_loc_40D2(0x0008_0000, PlayerForm::TheKid);
        assert_eq!(platform.y_velocity, 0);
        assert_eq!(
            platform.special,
            Some(PlatformSpecialState::TrapUp(TrapPlatformUpState {
                phase: TrapPlatformUpPhase::Rising,
            })),
            "ASM TrapPlatformUp_Loop enters loc_4230 after Kid stands on this platform",
        );

        platform.step_special_platform_loc_40D2(true, (platform.y_pos >> 16) as i16, 0);
        assert_eq!(
            platform.y_velocity, -0x4000,
            "ASM loc_4230 starts from zero velocity, then subtracts $4000 after the first move",
        );
    }

    #[test]
    fn trap_platform_down_post_fall_step_loc_41ee_continues_when_flag_clear() {
        // ASM loc_41EE (kid.asm:5304): tst.b $12(a3); beq.s loc_41EE
        assert_eq!(
            trap_platform_down_post_fall_step_loc_41EE(0x00),
            TrapPlatformDownPostFallActionLoc41EE::ContinueWaiting,
            "zero deactivation flag keeps the platform in the post-fall wait loop",
        );
    }

    #[test]
    fn trap_platform_down_post_fall_step_loc_41ee_deletes_when_flag_set() {
        // ASM loc_41EE: tst.b $12(a3) nonzero → TrapPlatformDown_Delete
        assert_eq!(
            trap_platform_down_post_fall_step_loc_41EE(0xFF),
            TrapPlatformDownPostFallActionLoc41EE::Delete,
            "nonzero deactivation flag falls through to TrapPlatformDown_Delete",
        );
    }

    #[test]
    fn trap_platform_down_post_fall_live_step_deletes_on_deactivation_loc_41ee() {
        let mut platform = make_test_platform(64, 128, 1, 0);
        platform.special = Some(PlatformSpecialState::TrapDown(TrapPlatformDownState {
            phase: TrapPlatformDownPhase::PostFall,
        }));

        assert_eq!(
            platform.step_special_platform_loc_40D2(false, 128, 0x01),
            SpecialPlatformStepActionLoc40D2::Delete,
            "ASM loc_41EE should make the live special-platform step delete once `$12(a3)` is nonzero",
        );
    }

    #[test]
    fn trap_platform_down_post_fall_step_loc_41ee_any_nonzero_byte_triggers_delete() {
        // Verify that any nonzero value (not just $FF) triggers deletion
        assert_eq!(
            trap_platform_down_post_fall_step_loc_41EE(0x01),
            TrapPlatformDownPostFallActionLoc41EE::Delete,
            "tst.b tests the whole byte — $01 is nonzero and should trigger delete",
        );
    }

    #[test]
    fn trap_platform_up_move_step_loc_4230_decelerates_by_0x4000_per_frame() {
        // ASM loc_4230 (kid.asm:5341): subi.l #$4000,d7
        // Start with velocity 0 (just stepped on), Kid at y=200, platform at y=190
        let result = trap_platform_up_move_step_loc_4230(0, 190, 200);
        assert_eq!(
            result.new_y_velocity, -0x4000,
            "first frame: 0 - $4000 = -$4000",
        );
        assert!(
            result.continue_moving,
            "190 - 200 = -10, which is > -$E0 (-224), so continue",
        );
    }

    #[test]
    fn trap_platform_up_move_step_loc_4230_clamps_velocity_to_negative_0x40000() {
        // ASM loc_4230: cmpi.l #-$40000,d7; bgt.w +; move.l #-$40000,d7
        let result = trap_platform_up_move_step_loc_4230(-0x3C001, 100, 200);
        assert_eq!(
            result.new_y_velocity, -0x40000,
            "velocity past the cap should clamp to -$40000",
        );
    }

    #[test]
    fn trap_platform_up_move_step_loc_4230_does_not_clamp_when_above_minimum() {
        // Velocity -$38000 minus $4000 = -$3C000, which is > -$40000
        let result = trap_platform_up_move_step_loc_4230(-0x38000, 100, 200);
        assert_eq!(
            result.new_y_velocity, -0x3C000,
            "-$38000 - $4000 = -$3C000, above the -$40000 floor",
        );
    }

    #[test]
    fn trap_platform_up_move_step_loc_4230_exits_when_far_above_kid() {
        // ASM: cmpi.w #-$E0,d7; bgt.s loc_4230 — exit when platform_y - kid_y <= -$E0
        // platform at y=0, Kid at y=224 → relative = 0 - 224 = -224 = -$E0
        let result = trap_platform_up_move_step_loc_4230(-0x10000, 0, 224);
        assert!(
            !result.continue_moving,
            "0 - 224 = -224 = -$E0, which is NOT > -$E0, so exit"
        );
    }

    #[test]
    fn trap_platform_up_move_step_loc_4230_continues_when_just_above_threshold() {
        // platform at y=1, Kid at y=224 → relative = 1 - 224 = -223 = -$DF
        let result = trap_platform_up_move_step_loc_4230(-0x10000, 1, 224);
        assert!(result.continue_moving, "1 - 224 = -223 > -224, so continue");
    }

    #[test]
    fn trap_platform_up_move_step_loc_4230_exact_clamp_boundary() {
        // Velocity exactly at -$40000 minus $4000 = -$44000 → should clamp to -$40000
        let result = trap_platform_up_move_step_loc_4230(-0x40000, 100, 200);
        assert_eq!(
            result.new_y_velocity, -0x40000,
            "-$40000 - $4000 = -$44000, clamped back to -$40000",
        );
    }

    #[test]
    fn trap_platform_up_post_rise_step_loc_4270_continues_when_flag_zero() {
        // ASM loc_4270 (kid.asm:5358): tst.b $12(a3); beq.s loc_4270
        assert_eq!(
            trap_platform_up_post_rise_step_loc_4270(0x00),
            TrapPlatformUpPostRiseActionLoc4270::ContinueWaiting,
            "when deactivation flag is zero, the ROM loops back to loc_4270",
        );
    }

    #[test]
    fn trap_platform_up_post_rise_step_loc_4270_deletes_when_flag_set() {
        // ASM loc_4270 (kid.asm:5358): nonzero → Deallocate_PlatformSlot + Delete_CurrentObject
        assert_eq!(
            trap_platform_up_post_rise_step_loc_4270(0xFF),
            TrapPlatformUpPostRiseActionLoc4270::Delete,
            "when deactivation flag is nonzero, the ROM deallocates and deletes",
        );
    }

    #[test]
    fn trap_platform_up_post_rise_live_step_deletes_on_deactivation_loc_4270() {
        let mut platform = make_test_platform(64, 128, 1, 0);
        platform.special = Some(PlatformSpecialState::TrapUp(TrapPlatformUpState {
            phase: TrapPlatformUpPhase::PostRise,
        }));

        assert_eq!(
            platform.step_special_platform_loc_40D2(false, 128, 0x01),
            SpecialPlatformStepActionLoc40D2::Delete,
            "ASM loc_4270 should make the live special-platform step delete once `$12(a3)` is nonzero",
        );
    }

    #[test]
    fn trap_platform_up_post_rise_step_loc_4270_any_nonzero_triggers_delete() {
        // ASM tst.b: any nonzero byte triggers the branch-not-taken path
        assert_eq!(
            trap_platform_up_post_rise_step_loc_4270(0x01),
            TrapPlatformUpPostRiseActionLoc4270::Delete,
            "even flag=1 (minimal nonzero) triggers deletion",
        );
    }

    #[test]
    fn platform_collision_shift_directions_matter() {
        // Mutant: lines 80, 91, 92 — replace >> with << in pixel extraction
        // If >> became <<, pixel values would be wildly wrong and collision would fail.
        // Place player at exactly (50, 100) in FP16.16 and platform at (40, 100).
        let platform = make_test_platform(40, 100, 4, 0); // 79px wide
        let mut slice = make_airborne_slice(50, 100);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.on_ground,
            "player should land on wide platform"
        );
        // Verify the floor is set at platform top minus 1
        assert_eq!(slice.player.y_pos, 99u32 << 16);
    }

    #[test]
    fn platform_collision_player_half_width_affects_overlap() {
        // Mutants: lines 81-82, replace - with + and + with -
        // If PLAYER_HALF_WIDTH subtraction/addition were wrong, horizontal overlap
        // would be inverted and near-edge landings would fail.
        let platform = make_test_platform(40, 100, 2, 0); // 47px wide, so right edge at x=87
        // Player at x=87 (right edge of platform) with half_width=5 → player_left=82, player_right=92
        // Platform: plat_x_px=40, plat_right=87. player_right(92) >= plat_x_px(40) ✓, player_left(82) <= plat_right(87) ✓
        let mut slice = make_airborne_slice(87, 100);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.on_ground,
            "player at platform right edge should land (half-width extends overlap)"
        );

        // Player at x=34 → player_left=29, player_right=39. plat_x_px=40 → player_right(39) < plat_x_px(40) → miss
        let mut slice = make_airborne_slice(34, 100);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            !slice.player.on_ground,
            "player just left of platform should NOT land"
        );
    }

    #[test]
    fn platform_collision_horizontal_overlap_boundary() {
        // Mutants: lines 95, 128 — replace < with <=, > with >=
        // Test exact boundary: player_right == plat_x_px overlaps, but ASM
        // loc_2672 resolves the equal corner escape horizontally instead of
        // entering loc_26D4 landing.
        let platform = make_test_platform(40, 100, 2, 0); // plat_x_px=40, plat_right=87
        // player at x=35 → player_left=30, player_right=40. player_right(40) < plat_x_px(40) is false → overlap
        let mut slice = make_airborne_slice(35, 100);
        apply_platform_collision(&mut slice, &[platform]);
        assert_eq!(slice.player.x_pos, 34u32 << 16);
        assert!(!slice.player.on_ground);

        // player at x=93 → player_left=88, player_right=98. player_left(88) > plat_right(87) → miss
        let mut slice = make_airborne_slice(93, 100);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            !slice.player.on_ground,
            "player_left > plat_right should NOT overlap"
        );

        // player at x=92 → player_left=87, player_right=97. player_left(87) > plat_right(87) is false → overlap
        let mut slice = make_airborne_slice(92, 100);
        apply_platform_collision(&mut slice, &[platform]);
        assert_eq!(slice.player.x_pos, 93u32 << 16);
        assert!(!slice.player.on_ground);
    }

    #[test]
    fn platform_collision_airborne_path_ignores_snap_tolerance_loc_25a0() {
        let platform = make_test_platform(40, 100, 2, 0);
        // ASM Platforms_CheckCollision loc_25A0 requires Kid_hitbox_bottom to
        // overlap the platform top before loc_26D4 can land. Terrain's 6px
        // snap tolerance is not part of this platform routine.
        let snap_offset = GROUND_SNAP_TOLERANCE_FP;
        let mut slice = make_airborne_slice(50, 100);
        slice.player.y_pos = (100u32 << 16) - snap_offset;
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            !slice.player.on_ground,
            "ASM loc_25A0 should skip a platform when Kid's feet are above it, even inside terrain snap tolerance"
        );

        let mut slice = make_airborne_slice(50, 100);
        slice.player.y_pos = (100u32 << 16) - snap_offset - 1;
        slice.player.y_velocity = 0x1_0000;
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            !slice.player.on_ground,
            "ASM loc_25A0 should also skip when just beyond the old native snap tolerance"
        );
    }

    #[test]
    fn platform_collision_riding_snap_distance_check() {
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.x_velocity = 0x1_0000;
        // ASM loc_8DA0/loc_8306 follows Addr_PlatformStandingOn and does not
        // recheck terrain's snap tolerance before reseating to platform_y - 1.
        let mut slice = make_airborne_slice(50, 93);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.x_pos > x_before,
            "stored platform support should carry even when Kid's current feet are outside the old terrain snap tolerance"
        );
        assert_eq!(slice.player.y_pos, 99 << 16);

        let mut slice = make_airborne_slice(50, 98);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.x_pos > x_before,
            "grounded player 2px from platform should be carried"
        );
    }

    #[test]
    fn platform_collision_grounded_carry_requires_stored_platform_loc_8ed0() {
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.x_velocity = 0x1_0000;
        let mut slice = make_airborne_slice(50, 100);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;

        let outcome = apply_platform_collision_with_ceiling_probe(
            &mut slice,
            &[platform],
            None,
            PlayerForm::TheKid,
            false,
            None,
            false,
        );

        assert_eq!(
            outcome,
            PlatformCollisionOutcome::None,
            "ASM sub_8ED0 loc_8F1C reports unsupported when Addr_PlatformStandingOn is zero"
        );
        assert_eq!(
            slice.player.x_pos, x_before,
            "grounded carry must not discover a nearby platform via terrain-style snap tolerance"
        );
    }

    #[test]
    fn platform_collision_riding_updates_floor_y() {
        // Mutant: line 111 — replace >> with << in floor_y_pos calc
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.y_velocity = -0x1_0000; // moving up

        let mut slice = make_airborne_slice(50, 100);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        apply_platform_collision(&mut slice, &[platform]);

        // floor_y_pos should track carried player position
        let carried_y_px = (slice.player.y_pos >> 16) as i32;
        assert_eq!(
            slice.terrain.floor_y_pos,
            (carried_y_px.max(0) as u32) << 16,
            "floor_y_pos must be updated to carried position (pixel << 16)"
        );
    }

    #[test]
    fn platform_collision_riding_horizontal_overlap_boundary() {
        // Mutants: line 95 riding path — || vs &&, < vs <=, > vs >=
        let mut platform = make_test_platform(40, 100, 2, 0); // plat_right = 87
        platform.x_velocity = 0x1_0000;

        // Player right edge == plat_x_px (40): player at x=45 → right=50 ≥ 40 ✓
        let mut slice = make_airborne_slice(45, 99);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.x_pos > x_before,
            "player overlapping platform should be carried"
        );

        // Player entirely left: x=34 → right=39 < 40 → no overlap
        let mut slice = make_airborne_slice(34, 99);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);
        assert_eq!(
            slice.player.x_pos, x_before,
            "player left of platform should NOT be carried"
        );
    }

    #[test]
    fn platform_collision_reseats_rider_to_platform_y_loc_8da0() {
        // ASM loc_8DA0/loc_8306: copy platform y_pos then subtract one pixel.
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.y_velocity = -0x2_0000; // moving up 2px/frame

        let mut slice = make_airborne_slice(50, 100);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        apply_platform_collision(&mut slice, &[platform]);

        assert_eq!(
            slice.player.y_pos,
            99 << 16,
            "ASM loc_8DA0/loc_8306 reseats the rider to platform_y - 1"
        );
        assert_eq!(
            slice.player.y_velocity, platform.y_velocity,
            "ASM loc_8DA0/loc_8306 still copies platform y_vel for follow-up collision"
        );
    }

    #[test]
    fn platform_collision_carry_x_velocity_sign() {
        // Mutant: lines 106-107 — ensure + with - is caught for x carry
        let mut platform = make_test_platform(40, 100, 4, 0); // wide platform
        platform.x_velocity = -0x1_0000; // moving left

        let mut slice = make_airborne_slice(60, 100);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);

        assert!(
            slice.player.x_pos < x_before,
            "player should be carried leftward by platform with negative x_velocity"
        );
    }

    #[test]
    fn platform_collision_player_inside_platform_does_not_land() {
        // Mutants: lines 140-141 — player_foot_y_px > plat_bottom check
        let platform = make_test_platform(40, 100, 2, 1); // height=31, so bottom at y=131
        // Player at y=132 — feet below platform bottom → should NOT land
        let mut slice = make_airborne_slice(50, 132);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            !slice.player.on_ground,
            "player below platform bottom should NOT land"
        );

        // Player at y=131 — feet at platform bottom still resolves sideways
        // because ASM loc_2672 compares full AABB escape magnitudes first.
        let mut slice = make_airborne_slice(50, 131);
        apply_platform_collision(&mut slice, &[platform]);
        assert_eq!(slice.player.x_pos, 34u32 << 16);
        assert!(!slice.player.on_ground);
    }

    #[test]
    fn platform_collision_floor_y_set_correctly() {
        // Mutants: lines 110-111, 146-147 — shift direction and sub-pixel in landing
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(50, 101); // 1px below platform top
        apply_platform_collision(&mut slice, &[platform]);

        assert!(slice.player.on_ground);
        // Floor should be 1px above platform (99), shifted to FP
        assert_eq!(slice.terrain.floor_y_pos, 99u32 << 16);
        // Player y should match floor (preserving sub-pixel since y_vel >= 0)
        assert_eq!(slice.player.y_pos >> 16, 99);
    }

    #[test]
    fn platform_collision_sub_pixel_preservation_falling() {
        // Mutant: line 151 — pre_gravity_y_vel >= 0 sub-pixel preservation
        // ASM loc_AAF8: falling landing preserves sub-pixel
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(50, 100);
        slice.player.y_pos |= 0x8000; // add sub-pixel
        slice.pre_gravity_y_vel = 0x1_0000; // was falling before gravity

        apply_platform_collision(&mut slice, &[platform]);

        assert!(slice.player.on_ground);
        // Sub-pixel from original y_pos should be preserved
        assert_eq!(
            slice.player.y_pos & 0xFFFF,
            0x8000,
            "falling landing should preserve sub-pixel (ASM loc_AAF8)"
        );
    }

    #[test]
    fn platform_collision_sub_pixel_cleared_rising() {
        // Mutant: line 151 — pre_gravity_y_vel < 0 clears sub-pixel
        // ASM loc_AB22: rising landing clears sub-pixel
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(50, 100);
        slice.player.y_pos |= 0x8000; // add sub-pixel
        slice.pre_gravity_y_vel = -0x1_0000; // was rising before gravity
        slice.player.y_velocity = 0x1_0000; // but now falling (gravity flipped)

        apply_platform_collision(&mut slice, &[platform]);

        assert!(slice.player.on_ground);
        assert_eq!(
            slice.player.y_pos & 0xFFFF,
            0,
            "rising landing should clear sub-pixel (ASM loc_AB22)"
        );
    }

    #[test]
    fn platform_collision_feet_at_or_below_without_snap() {
        // Mutants: lines 136-138 — feet_at_or_below vs within_snap logic
        let platform = make_test_platform(40, 100, 2, 1); // height=31
        // Player at y=105 — below platform top, above bottom (100+31=131) → should land
        let mut slice = make_airborne_slice(50, 105);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.on_ground,
            "player feet below platform top but above bottom should land"
        );
    }

    #[test]
    fn platform_collision_or_vs_and_in_feet_check() {
        // Mutant: line 138 — replace || with &&
        // If || became &&, landing would require BOTH feet_at_or_below AND within_snap
        // but a player at y=102 (2px below platform at 100) is feet_at_or_below but
        // may not be within snap if snap tolerance is distance-based from exact top.
        let platform = make_test_platform(40, 100, 2, 0);
        // Player at y=107 → abs_diff = 0x7_0000 > 0x6_0000 (outside snap), but feet_at_or_below = true
        let mut slice = make_airborne_slice(50, 107);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.on_ground,
            "feet_at_or_below alone should suffice for landing (|| not &&)"
        );
    }

    #[test]
    fn platform_collision_y_velocity_check_blocks_zero() {
        // Mutant: line 117 — replace < with <=
        // y_velocity == 0 should still allow landing (player not rising)
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(50, 100);
        slice.player.y_velocity = 0; // zero velocity, not rising
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.on_ground,
            "player with y_velocity==0 should be able to land (< 0 check, not <= 0)"
        );
    }
}
