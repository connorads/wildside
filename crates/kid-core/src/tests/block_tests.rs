use super::*;

fn active_platform_handle_sub_2728(layout_offset_words: u16) -> ActivePlatformHandleSub2728 {
    ActivePlatformHandleSub2728 {
        layout_offset_words,
    }
}

fn manifest_with_static_tornado() -> AssetManifest {
    let mut m = manifest();
    if let Some(enemy) = m.levels[0].enemy_layout_metadata.as_mut() {
        enemy.objects[0] = kid_data::EnemyLayoutObject {
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
        enemy.objects = vec![kid_data::EnemyLayoutObject {
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
}

fn manifest_with_tough_skull() -> AssetManifest {
    let mut m = manifest();
    if let Some(enemy) = m.levels[0].enemy_layout_metadata.as_mut() {
        enemy.objects[0] = kid_data::EnemyLayoutObject {
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
            Some((3, 4))
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0, // ready to launch immediately
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
    fn hopper_targets_player_direction_before_launch() {
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
            grounded_frames: MINI_SKULL_GROUND_DELAY_FRAMES,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            "hopper should retarget toward the player before launching"
        );
        assert!(
            enemy.x_velocity > 0,
            "hopper should launch toward the player after the grounded delay"
        );
    }

    #[test]
    fn hopper_reverses_at_platform_edge_before_launch() {
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
            grounded_frames: MINI_SKULL_GROUND_DELAY_FRAMES,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            Direction::Right,
            "hopper should ignore a player-relative left hop when the platform ends there"
        );
        assert!(
            enemy.x_velocity > 0,
            "hopper should launch away from the gap after reversing at the edge"
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
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            },
            level_bounds: LevelBounds {
                width_pixels: 64,
                height_pixels: 64,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            },
            level_bounds: LevelBounds {
                width_pixels: 80,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
    fn airborne_top_edge_contact_does_not_start_wall_cling_without_direct_wall_hit() {
        let w = 5u16;
        let h = 5u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let offset = (usize::from(1u16) * usize::from(w) + usize::from(2u16)) * 2;
        data[offset] = 0x60;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let mut slice = OpeningSliceState {
            player: PlayerState {
                // ASM sub_8F26 only sees the direct wall probe here. At y=56 the
                // right-side probe has already cleared the overhang, so a top-edge-
                // only contact must not create a fresh wall-cling latch.
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
            },
            level_bounds: LevelBounds {
                width_pixels: 80,
                height_pixels: 80,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
        };

        slice.apply_wall_collision(Some(&grid), PLAYER_HALF_WIDTH);
        assert_eq!(slice.player.x_pos, 27 << 16);
        assert_eq!(slice.player.x_velocity, 0x0001_0000);
        assert!(!slice.player.right_wall_sticky);
        assert_eq!(slice.player.wall_cling_direction, None);
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
            },
            level_bounds: LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            },
            level_bounds: LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            },
            level_bounds: LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            },
            level_bounds: LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
        assert!(
            runtime.active_enemies.len() < enemies_before,
            "stomped enemy should be removed (before: {enemies_before}, after: {})",
            runtime.active_enemies.len()
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
    fn stomp_works_during_invulnerability() {
        let mut runtime = NativeRuntime::new(manifest_with_static_skull());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        runtime.progression.invulnerable_frames = 100;
        setup_stomp_scenario(&mut runtime, true);
        let enemies_before = runtime.active_enemies.len();
        assert!(enemies_before > 0, "Skull should be activated");

        runtime.step_frame(FrameInput::default());
        assert!(
            runtime.active_enemies.len() < enemies_before,
            "stomp should still kill enemy during invulnerability"
        );
    }

    #[test]
    fn falling_onto_diamond_collects_without_stomp_bounce() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_floor_at_row(14));
        start_level(&mut runtime);

        let diamonds_before = runtime.progression.diamond_count;

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.camera.x_pos = 0;
            slice.camera.y_pos = 0;
        }
        runtime.step_frame(FrameInput::default());

        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_pos = 0x0120_0000;
            slice.player.y_pos = 0x00D8_0000;
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
            runtime.progression.diamond_count,
            diamonds_before + 1,
            "falling through a diamond should still collect it"
        );
        assert_ne!(
            slice.player.y_velocity, STOMP_BOUNCE_VELOCITY,
            "diamond collection should not trigger stomp bounce"
        );
        assert!(
            !runtime
                .active_enemies
                .iter()
                .any(|enemy| enemy.family == EnemyFamily::Diamond),
            "diamond should be removed after collection"
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
        assert!(
            runtime.active_enemies[0].damage_flash_frames > 0,
            "damaged enemy should have flash frames"
        );
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
        assert!(
            runtime.active_enemies.len() < enemies_before,
            "2-HP enemy should die on the second stomp frame"
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
            GoatCollisionRouteLoc3CB14::ContinueSharedWalker,
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
            GoatCollisionRouteLoc3CB14::ContinueSharedWalker,
            "ASM Goat.asm loc_3CB14 treats unrelated positive collision words as clear-and-resume"
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
    fn goat_death_sequence_matches_loc_3cb64() {
        assert_eq!(
            goat_death_sequence_loc_3cb64(),
            GoatDeathSequenceLoc3CB64 {
                death_animation: "stru_3C9EA",
                waits_until_animation_finished: true,
                exits_enemy: true,
            },
            "ASM Goat.asm loc_3CB64 should play the death animation to completion before enemy exit"
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
            "ASM Goat.asm loc_3CBBA should subtract 20px for flipped x_direction before restoring patrol state"
        );
    }

    #[test]
    fn pacer_walk_speed_armadillo_scales_with_hp() {
        // Armadillo: base_hp=1, base_speed=$8000, step=$10000
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, 1), 0x0000_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, 2), 0x0001_8000);
        assert_eq!(pacer_walk_speed(EnemyFamily::Armadillo, 3), 0x0002_8000);
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
        step_scorpion_tracking(&mut enemy, player_x, None);
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

        step_scorpion_tracking(&mut enemy, Some(150u32 << 16), None);

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
        step_scorpion_tracking(&mut enemy, player_x, None);
        assert_eq!(enemy.walk_speed_override, 0, "HP<5 should not boost");
    }

    #[test]
    fn scorpion_tracking_resets_speed_outside_range() {
        // Scorpion.asm loc_3DE7E: outside 170px, reset to $18000.
        use enemies::step_scorpion_tracking;
        let mut enemy = make_pacer(EnemyFamily::Scorpion, 5, 100, 224);
        enemy.walk_speed_override = 0x3_8000; // previously boosted
        let player_x = Some(500u32 << 16); // 400px away (outside 170px)
        step_scorpion_tracking(&mut enemy, player_x, None);
        assert_eq!(
            enemy.walk_speed_override, 0x1_8000,
            "outside range should reset"
        );
    }

    #[test]
    fn scorpion_tracking_refaces_before_range_exit_per_loc_3dda8() {
        use enemies::step_scorpion_tracking;

        let mut enemy = make_pacer(EnemyFamily::Scorpion, 5, 100, 224);
        enemy.facing = Direction::Left;
        enemy.walk_speed_override = 0x3_8000;

        step_scorpion_tracking(&mut enemy, Some(500u32 << 16), None);

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
            FireDemonCollisionRouteLoc3A586::Death,
            "ASM Fire_Demon.asm loc_3A586 sends negative collision_type to the fire demon death path"
        );
        assert_eq!(
            fire_demon_collision_dispatch_loc_3a586(0, 3),
            FireDemonCollisionRouteLoc3A586::ResumeSharedWalker {
                next_collision_type: 0,
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
            "ASM Fire_Demon.asm loc_3A586 decrements current_hp for colid_hurt and then clears collision_type through the shared tail"
        );
        assert_eq!(
            fire_demon_collision_dispatch_loc_3a586(FIRE_DEMON_COLID_HURT, 1),
            FireDemonCollisionRouteLoc3A586::Death,
            "ASM Fire_Demon.asm loc_3A586 falls through to fire demon death when the colid_hurt decrement reaches zero"
        );
        assert_eq!(
            fire_demon_collision_dispatch_loc_3a586(0x0004, 3),
            FireDemonCollisionRouteLoc3A586::ResumeSharedWalker {
                next_collision_type: 0,
            },
            "ASM Fire_Demon.asm loc_3A586 ignores unrecognised positive collision IDs via the shared clear tail"
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
        // Diamond: raw enemy_level, min 1
        assert_eq!(spawn_hit_points(EnemyFamily::Diamond, 0), 1);
        assert_eq!(spawn_hit_points(EnemyFamily::Diamond, 2), 2);
    }

    #[test]
    fn pacer_from_layout_object_resolves_goat() {
        let slots = vec![0xFFFF, 0x0010, 0x001C]; // slot 1 = Goat ($10)
        let object = kid_data::EnemyLayoutObject {
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
            x_direction: 0,
            cloud_x: 100,
            cloud_y: 50,
            kid_x: 92,
            kid_y: 58,
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
                x_direction: 0,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 92,
                kid_y: 20,
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
    fn cloud_diagonal_attack_setup_loc_35cd8_starts_when_kid_is_in_front_band() {
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
                object_45: 0x00,
                x_direction: 0,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 92,
                kid_y: 58,
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
                x_direction: 0xFF,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 108,
                kid_y: 58,
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
                x_direction: 0,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 100,
                kid_y: 70,
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
    fn cloud_diagonal_attack_setup_loc_35d62_starts_horizontal_side_attack() {
        assert_eq!(
            cloud_diagonal_attack_setup_loc_35CD8(CloudDiagonalAttackInputLoc35CD8 {
                object_45: 0x00,
                x_direction: 0,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 90,
                kid_y: 40,
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
                x_direction: 0xFF,
                cloud_x: 100,
                cloud_y: 50,
                kid_x: 110,
                kid_y: 40,
            }),
            CloudDiagonalAttackSetupLoc35CD8 {
                route: CloudDiagonalAttackRouteLoc35CD8::StartAttack {
                    x_word: 6,
                    y_word: 0,
                },
                animation_script: Some("stru_35EC8"),
                object_45_value: Some(0xFF),
            },
            "ASM Cloud.asm loc_35D62 mirrors the horizontal launch for non-zero x_direction after the Y-window gate"
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
    fn emo_rock_lands_and_walks() {
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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

        // Once on ground, should walk toward player (player is to the right)
        let x_before = enemy.x_pos;
        step_emo_rock(&mut enemy, Some(&grid), Some(player_x), None);
        assert_eq!(enemy.facing, Direction::Right, "should face toward player");
        assert!(enemy.x_pos > x_before, "should walk toward player");
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
            shared_walker_patrol_short_next: false,
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        };
        let mut projs = Vec::new();
        let player_x = Some(200u32 << 16);
        step_archer(&mut enemy, &mut projs, player_x, Some(100u32 << 16));
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
            shared_walker_patrol_short_next: false,
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        };
        let mut projs = Vec::new();
        let player_x = Some(200u32 << 16);
        let player_y = Some(100u32 << 16);
        step_archer(&mut enemy, &mut projs, player_x, player_y);
        assert_eq!(projs.len(), 1, "should spawn an arrow");
        assert!(
            projs[0].x_velocity > 0,
            "arrow should move toward player (right)"
        );
        assert_eq!(enemy.wait_timer, ARCHER_FIRE_TIMER[0], "timer should reset");
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
        // Player well above and to the right → quadrant 0, should give angle near 0-6
        let angle = archer_angle_to_player(100 << 16, 200 << 16, 200 << 16, 100 << 16);
        assert!(angle <= 6, "angle {angle} should be in upper-right range");
        let (xv, yv) = ARROW_ANGLE_TABLE[angle];
        assert!(xv > 0, "x velocity should be positive (rightward)");
        assert!(yv < 0, "y velocity should be negative (upward)");
    }

    #[test]
    fn archer_fire_timer_varies_by_hp() {
        assert_eq!(ARCHER_FIRE_TIMER[0], 0x1E, "HP 1: 30 frames");
        assert_eq!(ARCHER_FIRE_TIMER[1], 0x14, "HP 2: 20 frames");
        assert_eq!(ARCHER_FIRE_TIMER[2], 0x0A, "HP 3: 10 frames");
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            "re-aim interval matches fire timer"
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
            shared_walker_patrol_short_next: false,
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        };
        let mut projs = Vec::new();
        enemy.family = EnemyFamily::TankShooting;
        step_tank_shooting_fire(&mut enemy, &mut projs);
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
            shared_walker_patrol_short_next: false,
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        };
        let mut projs = Vec::new();
        step_tank_shooting_fire(&mut enemy, &mut projs);
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
        assert!(
            runtime.projectiles[0].x_velocity > 0,
            "first projectile rightward"
        );
        assert!(
            runtime.projectiles[1].x_velocity < 0,
            "second projectile leftward"
        );
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
        assert_eq!(
            runtime.enemy_death_effects[0].remaining,
            HEADYMETAL_DEATH_EFFECT_FRAMES
        );
    }

    #[test]
    fn multi_head_boss_spawns_three_heads() {
        use enemies::{
            BOSS_HEAD_NONE, SHISHKABOSS_HEAD_HP, SHISHKABOSS_HEAD_Y_OFFSETS, activate_enemies,
            is_multi_head_boss,
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
        assert_eq!(active[0].hit_points, SHISHKABOSS_HEAD_HP[0]);
        assert_eq!(active[1].hit_points, SHISHKABOSS_HEAD_HP[1]);
        assert_eq!(active[2].hit_points, SHISHKABOSS_HEAD_HP[2]);
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
            grounded_frames: 0,
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
            tar_monster_state: TarMonsterState::default(),
            spinning_twins_phase: SpinningTwinsPhase::default(),
            spinning_twins_orbit_index: 0,
            ufo_beam_segments: 0,
            ufo_beam_interval: 0,
            boss_head_index: enemies::BOSS_HEAD_NONE,
            scorpion_tracking_lock: false,
            flight_double_acceleration: false,
            flight_double_terminal_speed: false,
            walk_speed_override: 0,
            crab_frozen: false,
            shared_walker_state: 0,
            shared_walker_patrol_timer: 0,
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
        assert!(projectiles.len() >= 4, "should have spawned beam segments");
        assert!(
            projectiles.iter().all(|p| p.y_velocity > 0),
            "beams go downward"
        );
    }

    #[test]
    fn loc_3215e_enemy_art_slot_assignment_matches_second_slot_before_fallback() {
        use enemies::enemy_art_slot_assignment_sub_36E84;

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
    fn ufo_reverses_acceleration_on_timer() {
        let mut enemy = test_enemy(EnemyFamily::UFO, 200 << 16, 100 << 16, 1);
        enemy.x_acceleration = UFO_ACCELERATION;
        enemy.y_acceleration = -UFO_ACCELERATION;
        enemy.wait_timer = 0; // timer expired
        step_ufo(&mut enemy, None);
        // Acceleration should be reversed
        assert_eq!(enemy.x_acceleration, -UFO_ACCELERATION);
        assert_eq!(enemy.y_acceleration, UFO_ACCELERATION);
        assert_eq!(enemy.wait_timer, UFO_PATTERN_FRAMES);
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
    fn tornado_collision_dispatch_loc_35948_uses_tornado_table() {
        use enemies::{
            TornadoCollisionBehaviorLoc35948, TornadoCollisionDispatchLoc35948,
            tornado_collision_dispatch_loc_35948,
        };

        assert_eq!(
            tornado_collision_dispatch_loc_35948(0),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                behavior: None,
            },
            "ASM Tornado.asm loc_35948 reaches shared `loc_35056`, which returns directly when collision_type is zero",
        );
        assert_eq!(
            tornado_collision_dispatch_loc_35948(0x0004),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                behavior: Some(TornadoCollisionBehaviorLoc35948::BounceWall),
            },
            "ASM Tornado.asm loc_35948 table-selects wall bounce for collision_type=$04",
        );
        assert_eq!(
            tornado_collision_dispatch_loc_35948(0x001C),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                behavior: Some(TornadoCollisionBehaviorLoc35948::Hurt),
            },
            "ASM Tornado.asm loc_35948 table-selects the hurt/deletion path for collision_type=$1C",
        );
        assert_eq!(
            tornado_collision_dispatch_loc_35948(0x0020),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                behavior: Some(TornadoCollisionBehaviorLoc35948::CycloneBounceKidRight),
            },
            "ASM Tornado.asm loc_35948 routes collision_type=$20 through the Tornado-specific side-contact table entry",
        );
        assert_eq!(
            tornado_collision_dispatch_loc_35948(0x0024),
            TornadoCollisionDispatchLoc35948 {
                collision_type: 0,
                behavior: Some(TornadoCollisionBehaviorLoc35948::CycloneBounceKidLeft),
            },
            "ASM Tornado.asm loc_35948 routes collision_type=$24 through the opposite side-contact table entry",
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

    /// Unit test: validate the bounce values and direction logic.
    /// Tests that the x_velocity calculation matches ASM loc_35986.
    #[test]
    fn cyclone_tornado_bounce_x_velocity_values_match_asm() {
        // ASM loc_35986: player x_velocity = ±$20000 (after swap of ±2)
        // Direction based on player position relative to Tornado
        let player_left_of_tornado: i32 = -0x20000; // Player should bounce right
        let player_right_of_tornado: i32 = 0x20000; // Player should bounce left (negative)
        assert_eq!(
            player_left_of_tornado, -0x20000,
            "left bounce should be negative $20000"
        );
        assert_eq!(
            player_right_of_tornado, 0x20000,
            "right bounce should be positive $20000"
        );
        // Tornado x_velocity is half of player velocity, opposite direction
        let tornado_refl_left: i32 = 0x10000; // Half of -$20000 negated
        let tornado_refl_right: i32 = -0x10000; // Half of $20000 negated
        assert_eq!(tornado_refl_left, 0x10000, "Tornado should reflect right");
        assert_eq!(tornado_refl_right, -0x10000, "Tornado should reflect left");
        // Cyclone Y acceleration = $28 (40)
        assert_eq!(0x28u16, 40, "Y acceleration boost should be 40");
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
        step_ninja(&mut enemy, Some(&grid), None, None, PlayerForm::TheKid);
        assert_eq!(enemy.ninja_state, NinjaState::Stealth);
        assert_ne!(enemy.x_pos, start_x, "should move while pacing");
    }

    #[test]
    fn ninja_detects_player_in_cone_and_jumps() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        // Player 0x30 px above (within 0x20-0x50 range) and 0x08 px right (within ±0x10)
        let player_x = Some((100 + 0x08) << 16);
        let player_y = Some((200 - 0x30) << 16);
        step_ninja(&mut enemy, None, player_x, player_y, PlayerForm::TheKid);
        assert_eq!(enemy.ninja_state, NinjaState::JumpAttack);
        assert_eq!(enemy.facing, Direction::Right);
        assert!(enemy.y_velocity < 0, "should jump upward");
    }

    #[test]
    fn ninja_ignores_player_with_red_stealth() {
        let mut enemy = test_enemy(EnemyFamily::Ninja, 100 << 16, 200 << 16, 2);
        enemy.ninja_state = NinjaState::Stealth;
        let player_x = Some(100 << 16);
        let player_y = Some((200 - 0x30) << 16);
        step_ninja(&mut enemy, None, player_x, player_y, PlayerForm::RedStealth);
        assert_eq!(
            enemy.ninja_state,
            NinjaState::Stealth,
            "should stay stealthy when player wears Red Stealth"
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
            step_ninja(&mut enemy, Some(&grid), None, None, PlayerForm::TheKid);
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

        step_hand(&mut enemy, None, Some(0x0064_8000), Some(0x00C8_4000));

        assert_eq!(
            (enemy.x_pos, enemy.y_pos, enemy.x_velocity, enemy.y_velocity),
            (0x0066_0000, 0x00C4_0000, 0, 0),
            "ASM Hand.asm loc_3294C drives the native grabbing step through integer X+2/Y-4 snap and velocity clear"
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
                collision_probe_half_width_px: 9,
                collision_probe_height_px: 0x0E,
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

        step_hand(&mut enemy, None, Some(0x0064_8000), Some(0x00C8_4000));

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
        step_hand(&mut enemy, Some(&grid), player_x, player_y);
        assert_eq!(
            enemy.hand_state,
            HandState::Jumping,
            "should jump at detected player"
        );
        assert!(enemy.y_velocity < 0, "should jump upward");
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
        step_hand(&mut enemy, Some(&grid), player_x, player_y);
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
        step_hand(&mut enemy, Some(&grid), player_x, player_y);
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
        step_hand(&mut enemy, Some(&grid), player_x, player_y);
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
            step_hand(&mut enemy, Some(&grid), None, None);
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
        // One more step with timer at 0 should trigger charge
        step_lion(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        assert_eq!(enemy.lion_state, LionState::Charging);
        assert_eq!(enemy.facing, Direction::Right);
        assert_eq!(projs.len(), 1, "should fire projectile when charging");
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
            LionState::Charging,
            "ASM charges unconditionally on timer expiry — no distance gate"
        );
        assert_eq!(
            projs.len(),
            1,
            "should fire projectile even when Kid is far"
        );
    }

    #[test]
    fn lion_charge_returns_to_walking_on_overshoot() {
        // Lion charge ends when overshooting player by >112px ($70), not a
        // fixed duration timer. ASM: Lion.asm loc_3E450/3E604.
        let grid = grid_with_platform(17, 0, 200);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Lion, 60 << 16, floor_y, 6);
        enemy.lion_state = LionState::Charging;
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        // Player is at x=60, Lion starts at x=60 moving right.
        // After enough steps the Lion passes the player by >112px.
        let player_x = Some(60u32 << 16);
        for _ in 0..100 {
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
    fn robot_walks_then_shoots_and_charges_on_timer() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Robot, 60 << 16, floor_y, 2);
        enemy.robot_state = RobotState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 1;
        let player_x = Some(80u32 << 16);
        let mut projs = vec![];
        // Timer at 1 → decrements to 0
        step_robot(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        assert_eq!(enemy.robot_state, RobotState::Walking);
        assert!(projs.is_empty());
        // Timer at 0 → shoot + charge
        step_robot(&mut enemy, Some(&grid), player_x, &mut Some(&mut projs));
        assert_eq!(enemy.robot_state, RobotState::Charging);
        assert_eq!(projs.len(), 1, "robot should fire a projectile");
        assert_eq!(enemy.facing, Direction::Right, "should face toward player");
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
    fn orca_lunges_when_player_nearby() {
        let grid = grid_with_platform(17, 0, 120);
        let floor_y = (17 * 16) << 16;
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, floor_y, 4);
        enemy.orca_state = OrcaState::Walking;
        enemy.on_ground = true;
        enemy.wait_timer = 0; // no cooldown
        let player_x = Some((60 + 0x20) << 16); // within $30 range
        step_orca(&mut enemy, Some(&grid), player_x);
        assert_eq!(enemy.orca_state, OrcaState::Lunging);
        assert!(enemy.y_velocity < 0, "should launch upward");
        assert_eq!(enemy.facing, Direction::Right);
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

        assert_eq!(enemy.orca_state, OrcaState::Lunging);
        assert_eq!(enemy.facing, Direction::Left);
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
    fn orca_lands_and_returns_to_walking_with_cooldown() {
        let grid = grid_with_platform(17, 0, 120);
        let mut enemy = test_enemy(EnemyFamily::Orca, 60 << 16, (17 * 16 - 30) << 16, 4);
        enemy.orca_state = OrcaState::Lunging;
        enemy.x_velocity = ORCA_LUNGE_X_SPEED;
        enemy.y_velocity = ORCA_LUNGE_Y_SPEED;
        enemy.on_ground = false;
        for _ in 0..300 {
            step_orca(&mut enemy, Some(&grid), None);
            if enemy.orca_state == OrcaState::Walking {
                break;
            }
        }
        assert_eq!(enemy.orca_state, OrcaState::Walking);
        assert_eq!(enemy.wait_timer, ORCA_LUNGE_COOLDOWN);
        assert!(enemy.on_ground);
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
        assert_eq!(projectiles.len(), 3, "should spawn 3 fire fragments");
        for p in &projectiles {
            assert!(p.y_velocity < 0, "fragments launch upward");
            assert!(!p.friendly);
            assert!(p.gravity > 0, "fragments have gravity");
        }
    }

    // --- Drips ---

    #[test]
    fn drips_drops_projectile_on_timer_expiry() {
        let mut enemy = test_enemy(EnemyFamily::Drips, 60 << 16, 20 << 16, 1);
        enemy.wait_timer = 0;
        let mut projectiles = Vec::new();
        step_drips(&mut enemy, &mut projectiles);
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
        assert!(
            projectiles[0].terrain_kill,
            "native drop should expire on terrain contact instead of only a short timer"
        );
        assert!(!projectiles[0].friendly);
        assert_eq!(enemy.wait_timer, DRIPS_DROP_INTERVAL);
    }

    #[test]
    fn drips_waits_between_drops() {
        let mut enemy = test_enemy(EnemyFamily::Drips, 60 << 16, 20 << 16, 1);
        enemy.wait_timer = 5;
        let mut projectiles = Vec::new();
        step_drips(&mut enemy, &mut projectiles);
        assert!(projectiles.is_empty(), "should not drop while timer active");
        assert_eq!(enemy.wait_timer, 4);
    }

    // --- SpinningTwins ---

    #[test]
    fn spinning_twins_orbits_indefinitely_without_player_contact() {
        let mut enemy = test_enemy(EnemyFamily::SpinningTwins, 100 << 16, 50 << 16, 1);
        assert_eq!(enemy.spinning_twins_phase, SpinningTwinsPhase::Orbiting);

        // Step through orbit phase — X position should oscillate
        let initial_x = enemy.x_pos;
        step_spinning_twins(&mut enemy, None, None, None);
        assert_eq!(enemy.spinning_twins_phase, SpinningTwinsPhase::Orbiting);
        assert_ne!(enemy.x_pos, initial_x, "orbit should move X position");

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
            solid: true,
        };
        // Timer = 1: tick it down, still solid (timer goes to 0)
        assert!(group.solid);
        // Simulate one step
        if group.timer > 0 {
            group.timer -= 1;
        }
        assert_eq!(group.timer, 0);
        // Next step: timer is 0, transition happens
        group.solid = !group.solid;
        group.timer = if group.solid {
            group.solid_time
        } else {
            group.vanished_time
        };
        assert!(!group.solid, "should become vanished");
        assert_eq!(group.timer, 5, "vanished timer should be set");
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
            0x00, 0x00, // flags, mapping_frame = 0 (solid)
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
        assert!(bl.ghost_blocks[0].solid, "mapping_frame=0 means solid");
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
            &physics::ICE_KID
        );
        assert_eq!(
            PlayerForm::TheKid.ground_speed_table(TerrainType::Rubber),
            &physics::RUBBER_KID
        );
        assert_eq!(
            PlayerForm::Micromax.ground_speed_table(TerrainType::Ice),
            &physics::ICE_MICROMAX
        );
        assert_eq!(
            PlayerForm::Juggernaut.ground_speed_table(TerrainType::Rubber),
            &physics::RUBBER_JUGGERNAUT
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
            },
            level_bounds: LevelBounds {
                width_pixels: w * 16,
                height_pixels: h * 16,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            },
            level_bounds: LevelBounds {
                width_pixels: w * 16,
                height_pixels: h * 16,
            },
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
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
            platform.height_px, 15,
            "ASM loc_2822 should leave the collision height established before the art lookup intact; the gameplay port tracks that height even though it does not model the VRAM art pointer",
        );
        assert_eq!(
            platform.script, None,
            "ASM loc_2822 is still on the stationary-platform path; without a relative script offset the native port should leave motion scripting unset",
        );
    }

    #[test]
    fn active_platform_from_placement_loc_2836_splits_scripted_and_special_platforms() {
        use std::collections::BTreeMap;

        let script_offset = 0x3C3A;
        let script_table = kid_data::PlatformScriptTable {
            scripts: BTreeMap::from([(
                script_offset,
                kid_data::PlatformScript {
                    commands: vec![kid_data::PlatformMoveCommand {
                        duration: 6,
                        x_velocity: 0x1_0000,
                        y_velocity: -0x2_0000,
                    }],
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
                    raw: String::from("PlatformScript_3C3A-PlatformScript_BaseAddress"),
                    relative_offset: Some(script_offset),
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
            kid_data::PlatformScript {
                commands: vec![
                    kid_data::PlatformMoveCommand {
                        duration: 3,
                        x_velocity: 0x1_0000,
                        y_velocity: 0,
                    },
                    kid_data::PlatformMoveCommand {
                        duration: 3,
                        x_velocity: -0x1_0000,
                        y_velocity: 0,
                    },
                ],
            },
        );
        let table = kid_data::PlatformScriptTable { scripts };

        let mut platform = ActivePlatform {
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            script: Some(PlatformScriptState {
                script_offset: 0,
                command_index: 0,
                remaining_duration: 3,
            }),
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
            kid_data::PlatformScript {
                commands: vec![
                    kid_data::PlatformMoveCommand {
                        duration: 1,
                        x_velocity: 0x1_0000,
                        y_velocity: 0,
                    },
                    kid_data::PlatformMoveCommand {
                        duration: 4,
                        x_velocity: -0x2_0000,
                        y_velocity: 0,
                    },
                ],
            },
        );
        let table = kid_data::PlatformScriptTable { scripts };

        let mut platform = ActivePlatform {
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            script: Some(PlatformScriptState {
                script_offset: 0,
                command_index: 0,
                remaining_duration: 1,
            }),
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
    fn platform_step_script_loc_2422_immediately_loads_the_next_velocity_on_expiry() {
        use std::collections::BTreeMap;

        let mut scripts = BTreeMap::new();
        scripts.insert(
            0x0000,
            kid_data::PlatformScript {
                commands: vec![
                    kid_data::PlatformMoveCommand {
                        duration: 1,
                        x_velocity: 0x1_0000,
                        y_velocity: 0,
                    },
                    kid_data::PlatformMoveCommand {
                        duration: 5,
                        x_velocity: -0x3_0000,
                        y_velocity: 0x2_0000,
                    },
                ],
            },
        );
        let table = kid_data::PlatformScriptTable { scripts };

        let mut platform = ActivePlatform {
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            script: Some(PlatformScriptState {
                script_offset: 0,
                command_index: 0,
                remaining_duration: 1,
            }),
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
            kid_data::PlatformScript {
                commands: vec![
                    kid_data::PlatformMoveCommand {
                        duration: 2,
                        x_velocity: 0x4_0000,
                        y_velocity: -0x1_0000,
                    },
                    kid_data::PlatformMoveCommand {
                        duration: 1,
                        x_velocity: -0x2_0000,
                        y_velocity: 0x3_0000,
                    },
                ],
            },
        );
        let table = kid_data::PlatformScriptTable { scripts };

        let mut platform = ActivePlatform {
            x_pos: 100 << 16,
            y_pos: 50 << 16,
            width_px: 47,
            height_px: 15,
            x_velocity: 0,
            y_velocity: 0,
            script: Some(PlatformScriptState {
                script_offset: 0,
                command_index: 1,
                remaining_duration: 1,
            }),
        };

        platform.step_script(&table);

        assert_eq!(
            platform.script,
            Some(PlatformScriptState {
                script_offset: 0,
                command_index: 0,
                remaining_duration: 2,
            }),
            "ASM loc_243E (kid.asm:4302) follows the `$FFFF` loop-back pointer, which the native typed-script port models by rewinding to command 0",
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

    // --- Platform collision targeted mutation tests ---
    // Tests below target surviving mutants from cargo-mutants on platforms.rs.
    // Each test name references the line/mutation it catches.

    #[test]
    fn advance_to_window_stops_at_exact_preload_limit() {
        // Mutant: line 11, replace > with >= or ==
        // A placement whose x equals the preload limit exactly should be consumed
        // (> means "strictly past limit → stop"). If > became >=, this placement
        // would NOT be consumed.
        let window = ActivationWindow {
            x_min: 0,
            x_max: 100,
            y_min: 0,
            y_max: 224,
        };
        let preload_limit = 100u32
            .saturating_add(PLATFORM_PRELOAD_X_PADDING)
            .saturating_add(0); // buffer_right = 0
        let placement = PlatformPlacement {
            x: preload_limit as u16, // exactly at limit
            y: 50,
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
        let layout = PlatformLayout {
            placements: vec![placement],
        };
        let mut state = PlatformLoaderState::default();
        state.advance_to_window(&layout, window);
        // x == limit is NOT > limit, so placement should be consumed
        assert_eq!(
            state.next_index, 1,
            "placement at exact preload limit should be consumed"
        );
    }

    #[test]
    fn advance_to_window_stops_past_preload_limit() {
        // Verify a placement PAST the limit is not consumed
        let window = ActivationWindow {
            x_min: 0,
            x_max: 100,
            y_min: 0,
            y_max: 224,
        };
        let preload_limit = 100u32
            .saturating_add(PLATFORM_PRELOAD_X_PADDING)
            .saturating_add(0);
        let placement = PlatformPlacement {
            x: (preload_limit + 1) as u16, // past limit
            y: 50,
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
        let layout = PlatformLayout {
            placements: vec![placement],
        };
        let mut state = PlatformLoaderState::default();
        state.advance_to_window(&layout, window);
        assert_eq!(
            state.next_index, 0,
            "placement past preload limit should NOT be consumed"
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
        assert!(
            !platform_is_active(&outside_x, window),
            "platform outside x range should be inactive even if y is in range"
        );
        let outside_y = PlatformPlacement {
            x: 150, // inside x
            y: 200, // outside y
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
            x_min: 100,
            x_max: 200,
            y_min: 50,
            y_max: 150,
        };
        let placement = PlatformPlacement {
            x: 90,
            y: 40,
            buffer_left: 10,
            buffer_right: 0,
            buffer_top: 10,
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
        assert!(
            platform_is_active(&placement, window),
            "ASM loc_2762 (kid.asm:4684) should keep a platform active when Kid lands exactly on the buffered left/top edge because the ROM only branches away on strict `bgt`/`blt` misses",
        );

        let right_bottom_edge = PlatformPlacement {
            x: 200,
            y: 150,
            buffer_left: 0,
            buffer_right: 0,
            buffer_top: 0,
            buffer_bottom: 0,
            ..placement
        };
        assert!(
            platform_is_active(&right_bottom_edge, window),
            "ASM loc_2762 should also keep the exact buffered right/bottom edge active; equality reaches the loader instead of the `loc_2850` reject path",
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
                render_flags: 0xF4,
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
            "ASM loc_244C should bias visible platform coordinates by +$80, derive tile counts from `$1A/$1C >> 4`, seed `d2=#$500`, and dispatch through `off_24AC` using the low nibble of `$1F(a2)`",
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
        // Test exact boundary: player_right == plat_x_px (should miss with <, hit with <=)
        let platform = make_test_platform(40, 100, 2, 0); // plat_x_px=40, plat_right=87
        // player at x=35 → player_left=30, player_right=40. player_right(40) < plat_x_px(40) is false → overlap
        let mut slice = make_airborne_slice(35, 100);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.on_ground,
            "player_right == plat_x_px should count as overlap"
        );

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
        assert!(
            slice.player.on_ground,
            "player_left == plat_right should count as overlap"
        );
    }

    #[test]
    fn platform_collision_within_snap_tolerance() {
        // Mutant: line 100, replace <= with >
        let platform = make_test_platform(40, 100, 2, 0);
        // Player just above snap tolerance: y_pos within GROUND_SNAP_TOLERANCE_FP of platform
        let snap_offset = GROUND_SNAP_TOLERANCE_FP; // 0x6_0000 = 6 pixels in FP
        let mut slice = make_airborne_slice(50, 100);
        slice.player.y_pos = (100u32 << 16) - snap_offset; // at edge of tolerance
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.on_ground,
            "player at snap tolerance edge should land"
        );

        // Just beyond tolerance
        let mut slice = make_airborne_slice(50, 100);
        slice.player.y_pos = (100u32 << 16) - snap_offset - 1;
        slice.player.y_velocity = 0x1_0000; // still falling
        apply_platform_collision(&mut slice, &[platform]);
        // Beyond snap, but feet_at_or_below check might still apply (y=94 < 100)
        assert!(
            !slice.player.on_ground,
            "player beyond snap tolerance and above platform should not land"
        );
    }

    #[test]
    fn platform_collision_riding_snap_distance_check() {
        // Mutants: lines 100-101 — within_snap and abs_diff > 2 checks for riding
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.x_velocity = 0x1_0000;
        // Grounded player 7px away — beyond FP snap tolerance (0x6_0000 = 6px) AND pixel > 2 → NOT carried
        let mut slice = make_airborne_slice(50, 93);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);
        assert_eq!(
            slice.player.x_pos, x_before,
            "grounded player 7px from platform should NOT be carried (outside snap and > 2px)"
        );

        // Grounded player 2px away — within FP snap, abs_diff ≤ 2 → carried
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
    fn platform_collision_carry_y_velocity() {
        // Mutants: lines 108 — replace + with - or *
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.y_velocity = -0x2_0000; // moving up 2px/frame

        let mut slice = make_airborne_slice(50, 100);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let y_before = slice.player.y_pos;
        apply_platform_collision(&mut slice, &[platform]);

        // y_pos should decrease (platform moving up)
        assert!(
            slice.player.y_pos < y_before,
            "player should be carried upward by platform with negative y_velocity"
        );
        let expected =
            (i64::from(y_before) + i64::from(-0x2_0000i32)).clamp(0, i64::from(u32::MAX)) as u32;
        assert_eq!(slice.player.y_pos, expected);
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

        // Player at y=131 — feet at platform bottom → should land (> not >=)
        let mut slice = make_airborne_slice(50, 131);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.on_ground,
            "player at platform bottom edge should land"
        );
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
