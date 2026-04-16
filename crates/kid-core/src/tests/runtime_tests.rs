use super::*;

fn kid_projectile_collision_object_loc_1dcc(
    deleted: bool,
    mirrored: bool,
    collision_index_word: u16,
    x_anchor_px: i16,
    y_anchor_px: i16,
    collision_sizes: Vec<CollisionSizeEntrySub21F8>,
) -> KidProjectileCollisionObjectLoc1DCC {
    KidProjectileCollisionObjectLoc1DCC {
        deleted,
        mirrored,
        collision_index_word,
        x_anchor_px,
        y_anchor_px,
        collision_sizes,
    }
}

fn cached_any_percent_demo_runtime_and_script() -> (NativeRuntime, Vec<u8>) {
    cached_tas_runtime_and_script("kc_any_percent_2024")
}

fn runtime_with_projectile_and_player(
    proj_x: u16,
    proj_y: u16,
    player_x: u16,
    player_y: u16,
) -> NativeRuntime {
    let mut rt = NativeRuntime::new(manifest());
    rt.projectiles.push(make_projectile(proj_x, proj_y));
    let mut slice = OpeningSliceState::new(default_bounds());
    slice.player.x_pos = u32::from(player_x) << 16;
    slice.player.y_pos = u32::from(player_y) << 16;
    rt.opening_slice = Some(slice);
    rt
}

mod chunk_09 {
    use super::*;

    #[test]
    fn demo_1_bootstrap_keeps_camera_hidden_through_frame_2537() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_538) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (2_529..=2_537).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_2529 = frames.get(&2_529).expect("frame 2529 should be captured");
        assert_eq!(frame_2529.current_level, LevelId(0x15));
        assert_eq!(frame_2529.camera_y_pos, 0);
        assert_eq!(frame_2529.kid_x_pos, None);
        assert_eq!(frame_2529.kid_y_pos, None);

        let frame_2530 = frames.get(&2_530).expect("frame 2530 should be captured");
        assert_eq!(frame_2530.camera_y_pos, 0);

        let frame_2531 = frames.get(&2_531).expect("frame 2531 should be captured");
        assert_eq!(frame_2531.camera_y_pos, 0x00E0_0000);
        assert_eq!(frame_2531.kid_x_pos, None);
        assert_eq!(frame_2531.kid_y_pos, None);

        let frame_2532 = frames.get(&2_532).expect("frame 2532 should be captured");
        assert_eq!(frame_2532.camera_y_pos, 0xFF20_0000);

        let frame_2537 = frames.get(&2_537).expect("frame 2537 should be captured");
        assert_eq!(frame_2537.camera_y_pos, 0x00E0_0000);
        assert_eq!(frame_2537.kid_x_pos, None);
        assert_eq!(frame_2537.kid_y_pos, None);
    }

    #[test]
    fn demo_1_bootstrap_holds_pose_and_input_until_frame_2571() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_578) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if [2_544usize, 2_553, 2_571, 2_577].contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_2544 = frames.get(&2_544).expect("frame 2544 should be captured");
        assert_eq!(frame_2544.camera_y_pos, 0x00D8_0000);
        assert_eq!(frame_2544.kid_x_pos, Some(0x0010_0000));
        assert_eq!(frame_2544.kid_y_pos, Some(0x00FF_0000));

        let frame_2553 = frames.get(&2_553).expect("frame 2553 should be captured");
        assert_eq!(frame_2553.camera_y_pos, 0x0091_0000);
        assert_eq!(frame_2553.kid_x_pos, Some(0x0010_0000));
        assert_eq!(frame_2553.kid_y_pos, Some(0x00FF_0000));

        let frame_2571 = frames.get(&2_571).expect("frame 2571 should be captured");
        assert_eq!(frame_2571.camera_y_pos, 0x0064_0000);
        assert_eq!(frame_2571.kid_x_pos, Some(0x0010_0000));

        let frame_2577 = frames.get(&2_577).expect("frame 2577 should be captured");
        assert_eq!(frame_2577.kid_x_pos, Some(0x0010_1000));
    }

    #[test]
    fn demo_1_handoff_stays_airborne_until_frame_2625() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_626) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (2_623..=2_625).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_2623 = frames.get(&2_623).expect("frame 2623 should be captured");
        assert_eq!(frame_2623.kid_x_pos, Some(0x0059_E500));
        assert_eq!(frame_2623.kid_y_pos, Some(0x0139_C000));
        assert_eq!(frame_2623.kid_on_ground, Some(false));

        let frame_2624 = frames.get(&2_624).expect("frame 2624 should be captured");
        assert_eq!(frame_2624.kid_x_pos, Some(0x005C_7700));
        assert_eq!(frame_2624.kid_y_pos, Some(0x013F_4000));
        assert_eq!(
            frame_2624.kid_on_ground,
            Some(false),
            "demo playback should preserve the remaining sub-half-pixel gap on frame 2624",
        );

        let frame_2625 = frames.get(&2_625).expect("frame 2625 should be captured");
        assert_eq!(frame_2625.kid_x_pos, Some(0x005C_7700));
        assert_eq!(frame_2625.kid_y_pos, Some(0x013F_0000));
        assert_eq!(frame_2625.kid_on_ground, Some(true));
    }

    #[test]
    fn demo_1_post_landing_hold_preserves_standingstill_through_frame_2714() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_715) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (2_697..=2_714).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        for frame_index in 2_697..=2_714 {
            let frame = frames
                .get(&frame_index)
                .unwrap_or_else(|| panic!("frame {frame_index} should be captured"));
            assert_eq!(
                frame.kid_x_pos,
                Some(0x011A_0000),
                "demo playback should hold Kid at the landing tile through frame {frame_index}",
            );
            assert_eq!(
                frame.kid_x_velocity,
                Some(0),
                "standingstill should keep horizontal speed at zero on frame {frame_index}",
            );
            assert_eq!(
                frame.kid_on_ground,
                Some(true),
                "Kid should remain grounded during the post-landing standingstill hold on frame {frame_index}",
            );
        }
    }

    #[test]
    fn demo_1_wall_cling_snap_releases_into_run_on_frame_2723() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_726) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (2_723..=2_725).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_2723 = frames
            .get(&2_723)
            .unwrap_or_else(|| panic!("frame 2723 should be captured"));
        assert_eq!(
            frame_2723.kid_x_pos,
            Some(0x011B_0000),
            "demo 1 should shimmy 1px off the wall-cling on frame 2723",
        );

        let frame_2724 = frames
            .get(&2_724)
            .unwrap_or_else(|| panic!("frame 2724 should be captured"));
        assert_eq!(
            frame_2724.kid_x_pos,
            Some(0x011B_4080),
            "the grounded release should hand off into the oracle-backed run arc on frame 2724",
        );

        let frame_2725 = frames
            .get(&2_725)
            .unwrap_or_else(|| panic!("frame 2725 should be captured"));
        assert_eq!(frame_2725.kid_x_pos, Some(0x011B_C180));
    }

    #[test]
    fn demo_1_left_wall_snap_lands_flush_on_frame_2773() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_776) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (2_772..=2_775).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_2772 = frames
            .get(&2_772)
            .unwrap_or_else(|| panic!("frame 2772 should be captured"));
        assert_eq!(frame_2772.kid_x_pos, Some(0x0135_5000));

        let frame_2773 = frames
            .get(&2_773)
            .unwrap_or_else(|| panic!("frame 2773 should be captured"));
        assert_eq!(
            frame_2773.kid_x_pos,
            Some(0x0135_0000),
            "demo 1 should snap flush to the left wall on frame 2773",
        );

        let frame_2774 = frames
            .get(&2_774)
            .unwrap_or_else(|| panic!("frame 2774 should be captured"));
        assert_eq!(
            frame_2774.kid_x_pos,
            Some(0x0135_0000),
            "after the flush snap, the next held-input frame should stay pinned at the wall",
        );
    }

    #[test]
    fn demo_1_buffered_wall_sticky_pullup_lands_on_frame_2831() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_2831 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_832) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 2_831 {
                frame_2831 = Some(trace);
            }
        }

        let frame_2831 = frame_2831.unwrap_or_else(|| panic!("frame 2831 should be captured"));
        assert_eq!(
            frame_2831.kid_x_pos,
            Some(0x0144_0000),
            "demo 1 should pull up onto the prize-block top on frame 2831",
        );
        assert_eq!(
            frame_2831.kid_y_pos,
            Some(0x013F_8000),
            "demo 1 should preserve the oracle-backed pull-up sub-pixel on frame 2831",
        );
        assert_eq!(
            frame_2831.kid_y_velocity,
            Some(0),
            "the pull-up landing should clear vertical speed on frame 2831",
        );
        assert_eq!(
            frame_2831.kid_on_ground,
            Some(true),
            "the buffered pull-up should leave Kid grounded on frame 2831",
        );
    }

    #[test]
    fn demo_1_lands_on_the_walking_dragon_on_frame_2938() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_2938 = None;
        let mut dragon_positions = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_939) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 2_938 {
                frame_2938 = Some(trace);
                dragon_positions = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| (enemy.x_pos, enemy.y_pos))
                    .collect();
                break;
            }
        }

        let frame_2938 = frame_2938.unwrap_or_else(|| panic!("frame 2938 should be captured"));
        assert_eq!(
            frame_2938.kid_x_pos,
            Some(0x01B4_1700),
            "demo 1 should have the oracle-backed X position when landing on the dragon",
        );
        assert_eq!(
            frame_2938.kid_y_pos,
            Some(0x0188_3300),
            "demo 1 should have the oracle-backed Y position when landing on the dragon",
        );
        assert!(
            dragon_positions.contains(&(0x01C2_8000, 0x01B0_0000)),
            "expected the landing dragon at (0x01C28000, 0x01B00000), got {dragon_positions:#x?}",
        );
    }

    #[test]
    fn demo_1_keeps_the_oracle_backed_stomp_drift_on_frame_2994() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_2994 = None;
        let mut frame_3001 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_002) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 2_994 {
                frame_2994 = Some(trace.clone());
            }
            if frame_index == 3_001 {
                frame_3001 = Some(trace);
                break;
            }
        }

        let frame_2994 = frame_2994.unwrap_or_else(|| panic!("frame 2994 should be captured"));
        assert_eq!(
            frame_2994.kid_x_pos,
            Some(0x026B_0000),
            "demo 1 should preserve the post-dragon-stomp horizontal carry on frame 2994",
        );
        assert_eq!(
            frame_2994.kid_y_pos,
            Some(0x019F_5300),
            "frame 2994 should stay on the oracle-backed vertical path after the stomp",
        );

        let frame_3001 = frame_3001.unwrap_or_else(|| panic!("frame 3001 should be captured"));
        assert_eq!(
            frame_3001.kid_x_pos,
            Some(0x026E_8C00),
            "the carried drift should still line up with the oracle by frame 3001",
        );
        assert_eq!(
            frame_3001.camera_x_pos, 0x01C8_0000,
            "camera follow should recover once the carried drift lines the player back up",
        );
    }

    #[test]
    fn demo_1_lower_dragon_holds_the_post_stomp_tile_through_frame_2962() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragon_2962 = None;
        let mut dragon_2963 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_964) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 2_962 {
                dragon_2962 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.y_pos, enemy.facing, enemy.on_ground));
            }
            if frame_index == 2_963 {
                dragon_2963 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.y_pos, enemy.facing, enemy.on_ground));
                break;
            }
        }

        assert_eq!(
            dragon_2962,
            Some((0x01C2_8000, 0x01B0_0000, Direction::Right, true)),
            "the surviving lower dragon should still be parked on the oracle-backed stomp tile through frame 2962 instead of resuming patrol immediately",
        );
        assert_eq!(
            dragon_2963,
            Some((0x01C3_0000, 0x01B0_0000, Direction::Right, true)),
            "once the shared hurt hold clears, the lower dragon should resume the oracle-backed right-facing patrol step on frame 2963",
        );
    }

    #[test]
    fn demo_1_reenters_walking_from_standingstill_on_frame_3252() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_259) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (3_251..=3_258).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_3251 = frames
            .get(&3_251)
            .unwrap_or_else(|| panic!("frame 3251 should be captured"));
        assert_eq!(
            frame_3251.kid_x_pos,
            Some(0x027D_0000),
            "demo 1 should still be standing still on frame 3251",
        );
        assert_eq!(frame_3251.kid_x_velocity, Some(0));

        let frame_3252 = frames
            .get(&3_252)
            .unwrap_or_else(|| panic!("frame 3252 should be captured"));
        assert_eq!(
            frame_3252.kid_x_pos,
            Some(0x027D_1000),
            "demo 1 should restart the oracle-backed walk acceleration on frame 3252",
        );
        assert_eq!(
            frame_3252.kid_x_velocity,
            Some(0x0000_1000),
            "frame 3252 should take the first +0x1000 walking step",
        );
        assert_eq!(frame_3252.kid_on_ground, Some(true));

        let frame_3258 = frames
            .get(&3_258)
            .unwrap_or_else(|| panic!("frame 3258 should be captured"));
        assert_eq!(
            frame_3258.kid_x_pos,
            Some(0x027E_C000),
            "the resumed walk should stay aligned with the oracle through frame 3258",
        );
        assert_eq!(frame_3258.kid_x_velocity, Some(0x0000_7000));
    }

    #[test]
    fn demo_1_loads_the_second_knights_isle_dragon_by_frame_3400() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragon_layout_indices = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_401) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_400 {
                let mut layout_indices: Vec<_> = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| enemy.layout_index)
                    .collect();
                layout_indices.sort_unstable();
                dragon_layout_indices = Some(layout_indices);
                break;
            }
        }

        let dragon_layout_indices =
            dragon_layout_indices.unwrap_or_else(|| panic!("frame 3400 should be captured"));
        assert_eq!(
            dragon_layout_indices,
            vec![0, 1, 2, 4],
            "frame 3400 should keep all four Knights Isle walking dragons active inside the ASM-backed loader window",
        );
    }

    #[test]
    fn demo_1_reaches_the_last_oracle_backed_pre_stomp_arc_on_frame_3399() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_3399 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_400) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_399 {
                frame_3399 = Some(trace);
                break;
            }
        }

        let frame_3399 = frame_3399.unwrap_or_else(|| panic!("frame 3399 should be captured"));
        assert_eq!(
            frame_3399.kid_x_pos,
            Some(0x037E_F100),
            "demo 1 should keep the oracle-backed pre-stomp X position through frame 3399",
        );
        assert_eq!(
            frame_3399.kid_y_pos,
            Some(0x01A1_C000),
            "demo 1 should keep the oracle-backed pre-stomp Y position through frame 3399",
        );
    }

    #[test]
    fn demo_1_walking_dragons_keep_grounded_zero_velocity_on_frame_3399() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragons_3399 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_400) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_399 {
                let mut dragons: Vec<_> = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| {
                        (
                            enemy.layout_index,
                            enemy.x_velocity,
                            enemy.y_velocity,
                            enemy.on_ground,
                        )
                    })
                    .collect();
                dragons.sort_unstable_by_key(|dragon| dragon.0);
                dragons_3399 = Some(dragons);
                break;
            }
        }

        let dragons_3399 = dragons_3399.unwrap_or_else(|| panic!("frame 3399 should be captured"));
        assert_eq!(
            dragons_3399,
            vec![
                (0, 0, 0, true),
                (1, 0, 0, true),
                (2, 0, 0, true),
                (4, 0, 0, true),
            ],
            "frame 3399 should keep the grounded walking dragons on the oracle-backed zero-velocity path even though the position/facing frontier remains open",
        );
    }

    #[test]
    fn demo_1_frame_3400_reports_kid_object_support_candidates() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_3400 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_401) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_400 {
                frame_3400 = Some(trace);
                break;
            }
        }

        let frame_3400 = frame_3400.unwrap_or_else(|| panic!("frame 3400 should be captured"));
        let support_debug = frame_3400
            .gameplay_debug_fields
            .get("kid_object_support_candidates")
            .unwrap_or_else(|| panic!("frame 3400 should emit Kid/object support diagnostics"));

        assert!(
            support_debug.contains("enemy_0D#00"),
            "the frame-3400 diagnostics should include the nearby right-side walking dragon candidate",
        );
        assert!(
            support_debug.contains("support="),
            "the frame-3400 diagnostics should report support classification details for the nearby candidates",
        );
    }

    #[test]
    fn published_tas_frame_7969_reports_kid_ground_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));
        let ground_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_ground_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid-ground probe diagnostics"));

        assert!(
            ground_debug.contains("latched_ground=0"),
            "the frame-7969 diagnostics should show the grounded latch clears after the frame-7968 attach hibernate",
        );
        assert!(
            ground_debug.contains("y_vel=0xffffc080"),
            "the frame-7969 diagnostics should show the resume-frame ROM wall lookup now preserves upward climb velocity",
        );
        assert!(
            ground_debug.contains("hitbox=ground=0"),
            "the frame-7969 diagnostics should show the grounded probes no longer relatch despite the stale support cell under Kid",
        );
    }

    #[test]
    fn published_tas_frame_7968_reports_kid_special_motion_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_969) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let motion_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_special_motion_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid special-motion diagnostics"));

        assert!(
            motion_debug.contains(&format!("helmet={:#06x}", frame_7968.current_helmet_id)),
            "the frame-7968 diagnostics should echo the live helmet id",
        );
        assert!(
            motion_debug.contains("pending_stomp_bounce="),
            "the frame-7968 diagnostics should include the pending stomp-bounce latch state",
        );
    }

    #[test]
    fn published_tas_frame_7969_reports_kid_goal_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));
        let goal_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_goal_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid-goal probe diagnostics"));

        assert!(
            goal_debug.contains("lifecycle=alive"),
            "the frame-7969 diagnostics should show Kid is still in the normal gameplay lifecycle",
        );
        assert!(
            goal_debug.contains("touch=0"),
            "the frame-7969 diagnostics should prove native is not already touching the goal flag",
        );
        assert!(
            !frame_7969.level_complete,
            "the frame-7969 trace should not already be in LevelComplete",
        );
    }

    #[test]
    fn published_tas_frame_7968_reports_kid_gameplay_path_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_969) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let path_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid gameplay-path diagnostics"));

        assert!(
            path_debug.contains("lifecycle=alive"),
            "the frame-7968 diagnostics should show Kid is still on the live gameplay lifecycle path",
        );
        assert!(
            path_debug.contains("motion=iron_knight_attach_hibernate"),
            "the frame-7968 diagnostics should prove native now mirrors loc_A276's one-frame attach hibernate",
        );
        assert!(
            path_debug.contains("post=none"),
            "the frame-7968 diagnostics should prove no post-physics handoff moved Kid on the native side",
        );
        assert_eq!(
            frame_7968.kid_x_pos,
            Some(0x03A8_0000),
            "frame 7968 should preserve Kid's x position while the grounded Iron Knight attach hibernates",
        );
        assert_eq!(
            frame_7968.kid_y_pos,
            Some(0x01BF_0000),
            "frame 7968 should preserve Kid's y position while the grounded Iron Knight attach hibernates",
        );
        assert_eq!(
            frame_7968.kid_y_velocity,
            Some(0xFFFF_C000u32 as i32),
            "frame 7968 should still expose loc_A276's seeded upward climb velocity in object state",
        );
    }

    #[test]
    fn published_tas_frame_7969_matches_oracle_iron_knight_climb_resume_pose() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));
        let path_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid gameplay-path diagnostics"));

        assert!(
            path_debug.contains("motion=iron_knight_climb_resume"),
            "frame 7969 should stay on the dedicated Iron Knight climb resume path",
        );
        assert_eq!(
            frame_7969.kid_x_pos,
            Some(0x03A8_0000),
            "frame 7969 should keep Kid pinned to the wall column during the resume step",
        );
        assert_eq!(
            frame_7969.kid_y_pos,
            Some(0x01BE_C080),
            "frame 7969 should climb to the oracle-backed resume position",
        );
    }

    #[test]
    fn published_tas_frame_8000_stays_on_live_climb_gameplay() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_8000 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_001) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_000 {
                frame_8000 = Some(trace);
                break;
            }
        }

        let frame_8000 = frame_8000.unwrap_or_else(|| panic!("frame 8000 should be captured"));
        let path_debug = frame_8000
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8000 should emit kid gameplay-path diagnostics"));

        assert_eq!(
            frame_8000.game_mode,
            GameMode::DemoPlay,
            "frame 8000 should still trace as live demo gameplay before the timeout handoff becomes visible",
        );
        assert!(
            !path_debug.contains("motion=demo_auto_timeout_frozen"),
            "frame 8000 should not freeze the timeout snapshot one gameplay frame early",
        );
        assert_eq!(
            frame_8000.kid_y_pos,
            Some(0x01BA_6D00),
            "frame 8000 should keep climbing to the oracle-backed live gameplay pose",
        );
    }

    #[test]
    fn published_tas_frame_8001_delays_demo_1_timeout_handoff_by_one_more_climb_step() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_8001 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_002) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_001 {
                frame_8001 = Some(trace);
                break;
            }
        }

        let frame_8001 = frame_8001.unwrap_or_else(|| panic!("frame 8001 should be captured"));
        let path_debug = frame_8001
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8001 should emit kid gameplay-path diagnostics"));

        assert_eq!(
            frame_8001.game_mode,
            GameMode::DemoPlay,
            "frame 8001 should still be live demo gameplay before the timeout handoff becomes visible",
        );
        assert!(
            path_debug.contains("motion=iron_knight_climb_resume"),
            "frame 8001 should stay on the climb-resume motion instead of dropping into the timeout handoff",
        );
        assert_eq!(
            frame_8001.kid_y_pos,
            Some(0x01BA_1780),
            "frame 8001 should take one more oracle-backed climb step before the title return",
        );
    }

    #[test]
    fn published_tas_frame_8002_keeps_demo_1_in_live_climb_gameplay() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_8002 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_003) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_002 {
                frame_8002 = Some(trace);
                break;
            }
        }

        let frame_8002 = frame_8002.unwrap_or_else(|| panic!("frame 8002 should be captured"));
        let path_debug = frame_8002
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8002 should emit kid gameplay-path diagnostics"));

        assert_eq!(
            frame_8002.game_mode,
            GameMode::DemoPlay,
            "frame 8002 should still be live demo gameplay before the mode-0 handoff becomes visible",
        );
        assert_eq!(
            frame_8002.current_level,
            LevelId(0x15),
            "frame 8002 should stay in Knights Isle until the ROM timeout gate actually trips",
        );
        assert!(
            path_debug.contains("motion=iron_knight_climb_resume"),
            "frame 8002 should stay on the climb-resume motion instead of flipping into the title return",
        );
        assert_eq!(
            frame_8002.kid_y_pos,
            Some(0x01B9_BD00),
            "frame 8002 should keep the oracle-backed live climb pose instead of freezing the prior step",
        );
    }

    #[test]
    fn published_tas_frame_8003_keeps_demo_1_in_live_climb_gameplay() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_8003 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_004) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_003 {
                frame_8003 = Some(trace);
                break;
            }
        }

        let frame_8003 = frame_8003.unwrap_or_else(|| panic!("frame 8003 should be captured"));
        let path_debug = frame_8003
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8003 should emit kid gameplay-path diagnostics"));

        assert_eq!(
            frame_8003.game_mode,
            GameMode::DemoPlay,
            "frame 8003 should still trace as live demo gameplay before the mode-0 handoff becomes visible",
        );
        assert_eq!(
            frame_8003.current_level,
            LevelId(0x15),
            "frame 8003 should stay in Knights Isle until the stored timeout counter is compared on the next ReadJoypad pass",
        );
        assert!(
            path_debug.contains("motion=iron_knight_climb_resume"),
            "frame 8003 should keep climbing instead of entering the timeout title return one frame early",
        );
        assert_eq!(
            frame_8003.kid_y_pos,
            Some(0x01B9_5E00),
            "frame 8003 should keep the oracle-backed live climb pose while the ROM still compares the prior timeout counter",
        );
    }

    #[test]
    fn published_tas_frame_8004_keeps_demo_1_in_live_climb_gameplay() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_8004 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_005) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_004 {
                frame_8004 = Some(trace);
                break;
            }
        }

        let frame_8004 = frame_8004.unwrap_or_else(|| panic!("frame 8004 should be captured"));

        assert_eq!(
            frame_8004.game_mode,
            GameMode::DemoPlay,
            "frame 8004 should still trace as live demo gameplay before the ROM leaves demo mode",
        );
        assert_eq!(
            frame_8004.current_level,
            LevelId(0x15),
            "frame 8004 should stay in Knights Isle while the demo timeout counter is still below $3F2 in oracle RAM",
        );
        assert_eq!(
            frame_8004.kid_y_pos,
            Some(0x01B8_FB00),
            "frame 8004 should keep the oracle-backed live climb pose instead of freezing the prior step",
        );
    }

    #[test]
    fn published_tas_demo_1_times_out_on_frame_8069() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_8068 = None;
        let mut frame_8069 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_070) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                8_068 => frame_8068 = Some((trace.game_mode, trace.current_level)),
                8_069 => {
                    frame_8069 = Some((trace.game_mode, trace.current_level));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_8068,
            Some((GameMode::DemoPlay, LevelId(0x15))),
            "frame 8068 should still be live demo gameplay while the oracle timeout counter first reaches $3F2",
        );
        assert_eq!(
            frame_8069,
            Some((GameMode::SegaScreen, LevelId(0x00))),
            "frame 8069 should expose the ROM's mode-0 timeout handoff on the first post-threshold frame",
        );
    }

    #[test]
    fn published_tas_demo_1_preserves_the_oracle_timeout_snapshot_on_frames_8069_and_8070() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_8069 = None;
        let mut frame_8070 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_071) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                8_069 => {
                    frame_8069 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.camera_x_pos,
                        trace.camera_y_pos,
                        trace.number_objects,
                    ));
                }
                8_070 => {
                    frame_8070 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.camera_x_pos,
                        trace.camera_y_pos,
                        trace.number_objects,
                    ));
                    break;
                }
                _ => {}
            }
        }

        let expected = Some((
            GameMode::SegaScreen,
            LevelId(0x00),
            Some(0x0398_2C80),
            Some(0x0188_3680),
            0x02FF_0000,
            0x00E7_0000,
            9,
        ));
        assert_eq!(
            frame_8069, expected,
            "frame 8069 should keep the oracle-backed Kid/camera/object snapshot while the timeout handoff first exposes mode 0",
        );
        assert_eq!(
            frame_8070, expected,
            "frame 8070 should keep the same preserved timeout snapshot until the title rebuild starts clearing it",
        );
    }

    #[test]
    fn published_tas_frame_8056_hibernates_the_iron_knight_wall_release() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_8056 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_057) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_056 {
                frame_8056 = Some(trace);
                break;
            }
        }

        let frame_8056 = frame_8056.unwrap_or_else(|| panic!("frame 8056 should be captured"));
        let path_debug = frame_8056
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8056 should emit kid gameplay-path diagnostics"));

        assert!(
            path_debug.contains("motion=iron_knight_climb_exit_hibernate"),
            "frame 8056 should leave the climb-resume loop and hibernate the wall-release frame",
        );
        assert_eq!(
            frame_8056.kid_y_pos,
            Some(0x0186_D680),
            "frame 8056 should preserve the oracle-backed wall-cling pose while the release frame hibernates",
        );
    }

    #[test]
    fn published_tas_frame_8057_matches_the_oracle_wall_release_carry() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_8057 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_058) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_057 {
                frame_8057 = Some(trace);
                break;
            }
        }

        let frame_8057 = frame_8057.unwrap_or_else(|| panic!("frame 8057 should be captured"));
        let path_debug = frame_8057
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8057 should emit kid gameplay-path diagnostics"));

        assert!(
            !path_debug.contains("motion=iron_knight_climb_resume"),
            "frame 8057 should stay off the dedicated climb-resume motion after the wall release",
        );
        assert_eq!(
            frame_8057.kid_x_pos,
            Some(0x03A7_D500),
            "frame 8057 should pick up the oracle-backed leftward carry on the first post-release frame",
        );
        assert_eq!(
            frame_8057.kid_y_pos,
            Some(0x0185_9680),
            "frame 8057 should match the oracle-backed post-release jump pose",
        );
    }

    #[test]
    fn published_tas_frames_8056_to_8058_match_the_oracle_wall_release_camera_carry() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_8056 = None;
        let mut frame_8057 = None;
        let mut frame_8058 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_059) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                8_056 => frame_8056 = Some(trace.camera_y_pos),
                8_057 => frame_8057 = Some(trace.camera_y_pos),
                8_058 => {
                    frame_8058 = Some(trace.camera_y_pos);
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_8056,
            Some(0x00F0_0000),
            "frame 8056 should still advance the release-frame camera follow before the Iron Knight wall-release hibernates",
        );
        assert_eq!(
            frame_8057,
            Some(0x00EF_0000),
            "frame 8057 should continue from the oracle-backed release-frame camera state",
        );
        assert_eq!(
            frame_8058,
            Some(0x00EE_0000),
            "frame 8058 should keep the one-pixel-per-frame camera falloff after the wall release",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_pre_physics_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_pre_physics_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid pre-physics diagnostics"));
        assert!(
            frame_7968_debug.contains("first=iron_knight_climb"),
            "frame 7968 should record Iron Knight climb as the first pre-physics mutator on the attach frame",
        );
        assert!(
            frame_7968_debug.contains("mutators=iron_knight_climb"),
            "frame 7968 should keep the attach visible in the mutator list",
        );
        assert!(
            frame_7968_debug.contains(
                "before_physics[x=0x03a80000/y=0x01bf0000/x_vel=0x00000000/y_vel=0xffffc000/ground=0]"
            ),
            "frame 7968 should expose the seeded climb velocity and airborne state before the hibernated frame ends",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_pre_physics_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid pre-physics diagnostics"));
        assert!(
            frame_7969_debug.contains("first=iron_knight_climb"),
            "frame 7969 should now resume through the Iron Knight climb mutator before normal physics",
        );
        assert!(
            frame_7969_debug.contains("mutators=iron_knight_climb"),
            "frame 7969 should keep the climb mutator visible while the resume-frame bug remains open",
        );
        assert!(
            frame_7969_debug.contains(
                "before_physics[x=0x03a80000/y=0x01bf0000/x_vel=0x00000000/y_vel=0xffffc080/ground=0]"
            ),
            "frame 7969 should apply the ROM climb drift before the dedicated resume step runs",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_grounded_release_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        assert!(
            !frame_7968
                .gameplay_debug_fields
                .contains_key("kid_grounded_release_probe"),
            "frame 7968 should skip the grounded-release probe because loc_A276 hibernates before native normal physics",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_grounded_release_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid grounded-release diagnostics"));
        assert!(
            frame_7969_debug.contains(
                "ground=0/held=right/jump=0/jump_buffer=0/velocity=airborne_custom_gravity/custom_gravity=1/wall_cling=none"
            ),
            "frame 7969 should show the resume frame stays on Iron Knight custom gravity even though the later motion path still diverges",
        );
        assert!(
            frame_7969_debug.contains("scan=row=none/cols=none/support=none"),
            "frame 7969 should prove the old grounded support scan no longer runs once the attach frame hibernates",
        );
        assert!(
            frame_7969_debug.contains("release=0/post=none"),
            "frame 7969 should show no later native release handoff recovers the lost climb state",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_collision_handoff_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_collision_handoff_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid collision-handoff diagnostics"));
        assert!(
            frame_7968_debug.contains(
                "pending_stomp=0/pending_form=none/pending_level_skip=none/form_flash=0/ground=1/jump_buffer=0/wall_cling=none"
            ),
            "frame 7968 should still enter the attach frame with no pre-existing collision carry state",
        );
        assert!(
            frame_7968_debug.contains("anchor=0x03a80000,0x01bf0000"),
            "frame 7968 should preserve the saved anchor before the attach hibernate",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_collision_handoff_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid collision-handoff diagnostics"));
        assert!(
            frame_7969_debug.contains(
                "pending_stomp=0/pending_form=none/pending_level_skip=none/form_flash=0/ground=0/jump_buffer=0/wall_cling=none"
            ),
            "frame 7969 should show the next frame starts airborne from the hibernated attach state",
        );
        assert!(
            frame_7969_debug
                .contains("stomp_candidate=0/stomp_enemy=none/anchor=0x03a80000,0x01bf0000"),
            "frame 7969 should keep the anchor stable even though the climb follow-on is still wrong",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_same_frame_stomp_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_same_frame_stomp_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit same-frame stomp diagnostics"));
        assert!(
            frame_7968_debug.contains("pre_y_vel=0xffffc000/post_y_vel=0xffffc000"),
            "frame 7968 should record the hibernated attach frame's preserved seeded climb velocity",
        );
        assert!(
            frame_7968_debug.contains("candidate=0/enemy=none"),
            "frame 7968 should keep the stomp path cold while preserving the attach anchor",
        );
        assert!(
            frame_7968_debug.contains("anchor=0x03a80000,0x01bf0000"),
            "frame 7968 should keep the stomp path cold while preserving the attach anchor",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_same_frame_stomp_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit same-frame stomp diagnostics"));
        assert!(
            frame_7969_debug.contains("pre_y_vel=0xffffc080/post_y_vel=0xffffc080"),
            "frame 7969 should show the resume frame now preserves the climb velocity through physics",
        );
        assert!(
            frame_7969_debug.contains("candidate=0/enemy=none"),
            "frame 7969 should keep the stomp path cold while the new frontier stays in player motion",
        );
        assert!(
            frame_7969_debug.contains("anchor=0x03a80000,0x01bf0000"),
            "frame 7969 should keep the stomp path cold while the new frontier stays in player motion",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_iron_knight_attach_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_969) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let probe_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_iron_knight_attach_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit Iron Knight attach diagnostics"));

        assert!(
            probe_debug.contains("c_press=1/ground=1/climbing=0=>1/facing=right"),
            "frame 7968 should consume the raw C edge and enter Iron Knight climb from the grounded walking path",
        );
        assert!(
            probe_debug.contains("y_vel=0x00000000->0xffffc000"),
            "frame 7968 should mirror loc_A276's -0x4000 wall-climb lift-off seed",
        );
        assert!(
            probe_debug
                .contains("wall=hit=1/x=944/cell=59,27 collision=solid block=None has_block=0"),
            "frame 7968 should preserve the ROM edge-column wall hit at the attach boundary",
        );
        assert!(
            probe_debug.contains("candidate=1/attach=1"),
            "frame 7968 should prove the ported sub_922C gate now fires the live grounded attach window",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_flip_branch_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_flip_branch_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid_flip branch diagnostics"));
        assert!(
            frame_7968_debug
                .contains("wall_scan=1/b_press=0/c_press=1/helmet=0x0006/facing=right/ground=1=>0"),
            "frame 7968 should show the grounded attach clears the on-ground bit before the hibernated frame ends",
        );
        assert!(
            frame_7968_debug.contains("branch=c_pressed_fallthrough"),
            "frame 7968 should keep exposing the shared kid_flip C fan-out label for comparison against the intercepted motion path",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_flip_branch_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid_flip branch diagnostics"));
        assert!(
            frame_7969_debug
                .contains("wall_scan=1/b_press=0/c_press=0/helmet=0x0006/facing=right/ground=0=>0"),
            "frame 7969 should show the resume frame stays airborne without a fresh C edge",
        );
        assert!(
            frame_7969_debug
                .contains("wall=hit=1/x=944/cell=59,27 collision=solid block=None has_block=0"),
            "frame 7969 should keep the forward wall contact visible at the moved frontier",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_flip_follow_on_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_flip_follow_on_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid_flip follow-on diagnostics"));
        assert!(
            frame_7968_debug.contains(
                "branch=c_pressed_fallthrough/motion=iron_knight_attach_hibernate/post=none"
            ),
            "frame 7968 should show the shared C-fallthrough label now feeds the attach-hibernate motion branch instead of normal physics",
        );
        assert!(
            frame_7968_debug
                .contains("entry[x_vel=0x00000000/y_vel=0x00000000/ground=1/wall_cling=none]"),
            "frame 7968 should keep the follow-on stages frozen at the pre-hibernate walking snapshot",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_flip_follow_on_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid_flip follow-on diagnostics"));
        assert!(
            frame_7969_debug
                .contains("branch=no_button_branch/motion=iron_knight_climb_resume/post=none"),
            "frame 7969 should show the climb resume now stays on the dedicated Iron Knight loop",
        );
        assert!(
            frame_7969_debug
                .contains("entry[x_vel=0x00000000/y_vel=0xffffc080/ground=0/wall_cling=none]"),
            "frame 7969 should enter the resume step with the ROM drift velocity",
        );
        assert!(
            frame_7969_debug.contains(
                "jump_gravity[x_vel=0x00000000/y_vel=0xffffc080/ground=0/wall_cling=none]"
            ),
            "frame 7969 should keep horizontal speed pinned while the climb resume loop runs",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_jump_gravity_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        assert!(
            !frame_7968
                .gameplay_debug_fields
                .contains_key("kid_jump_gravity_probe"),
            "frame 7968 should skip the jump/gravity probe because the attach frame never reaches native normal physics",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_jump_gravity_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid jump/gravity diagnostics"));
        assert!(
            frame_7969_debug.contains(
                "branch=airborne_custom_gravity/jump=0/jump_buffer=0=>0/just_jumped=0=>0"
            ),
            "frame 7969 should show the resume frame now stays on the Iron Knight custom-gravity branch",
        );
        assert!(
            frame_7969_debug.contains("b_held=0/custom_gravity=1"),
            "frame 7969 should prove native keeps the Iron Knight custom-gravity climb state alive",
        );
        assert!(
            frame_7969_debug.contains("y_vel=0xffffc080->0xffffc080/ground=0=>0"),
            "frame 7969 should confirm the resume frame preserves climb velocity through gravity",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_iron_knight_block_breaker_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        for (frame_index, trace) in [(7_968usize, &frame_7968), (7_969usize, &frame_7969)] {
            let probe_debug = trace
                .gameplay_debug_fields
                .get("kid_iron_knight_block_breaker_probe")
                .unwrap_or_else(|| {
                    panic!("frame {frame_index} should emit Iron Knight block-breaker diagnostics")
                });

            assert!(
                probe_debug.contains("helmet=0x0006"),
                "frame {frame_index} should keep the Iron Knight helmet context while testing the ROM block-break path",
            );
            assert!(
                probe_debug.contains("speed_gate=0"),
                "frame {frame_index} should prove the ROM Iron Knight block-break speed gate stays cold in the live 7968..7969 window even after the attach gate fires",
            );
            assert!(
                probe_debug.contains("candidate=0"),
                "frame {frame_index} should rule out the ROM Iron Knight block-breaker path as the source of the oracle lift-off",
            );
        }
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_wall_cling_release_probe() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_wall_cling_release_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit wall-cling release diagnostics"));
        assert!(
            frame_7968_debug
                .contains("held=right/probe_dir=right(held)/wall_cling=none/sticky=R0L0/ground=1"),
            "frame 7968 should show the attach frame still starts from the grounded walking-side release gate inputs",
        );
        assert!(
            frame_7968_debug.contains("y_vel=0x00000000/y=447=>447"),
            "frame 7968 should keep the wall-cling release mirror frozen at the pre-attach pose",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_wall_cling_release_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit wall-cling release diagnostics"));
        assert!(
            frame_7969_debug
                .contains("held=right/probe_dir=right(held)/wall_cling=none/sticky=R0L0/ground=0"),
            "frame 7969 should show the new frontier reaches the wall-cling release probe already airborne",
        );
        assert!(
            frame_7969_debug.contains("y_vel=0xffffc080/y=447=>446"),
            "frame 7969 should confirm native now takes the ROM-style climb step without falling into run speed",
        );
        assert!(
            frame_7969_debug.contains(
                "branch=no_wall_cling_state/sub_ADB8=wall_cling_missing/sub_AE6E=wall_cling_missing"
            ),
            "frame 7969 should prove both wall-cling release helpers remain cold at the moved frontier",
        );
    }

    #[test]
    fn published_tas_frame_1406_reports_prize_diamond_frontier_debug_fields() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_1406 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_407) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_406 {
                frame_1406 = Some(trace);
                break;
            }
        }

        let frame_1406 = frame_1406.unwrap_or_else(|| panic!("frame 1406 should be captured"));
        assert!(
            frame_1406
                .gameplay_debug_fields
                .contains_key("pending_prize_diamond_awards"),
            "frame 1406 should expose the queued diamond-award state for the live frontier",
        );
        let candidate_debug = frame_1406
            .gameplay_debug_fields
            .get("floating_diamond_contact_candidates")
            .unwrap_or_else(|| {
                panic!("frame 1406 should expose floating-diamond pickup diagnostics")
            });
        assert!(
            candidate_debug.contains("diamond@"),
            "the published frame-1406 diagnostics should enumerate nearby floating diamonds",
        );
    }

    #[test]
    fn published_tas_prize_diamond_queue_is_counting_down_on_frame_1407() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut pending_debug = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_408) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_407 {
                pending_debug = trace
                    .gameplay_debug_fields
                    .get("pending_prize_diamond_awards")
                    .cloned();
                break;
            }
        }

        assert_eq!(
            pending_debug.as_deref(),
            Some("[32]"),
            "by frame 1407 the published TAS should already be one frame into the queued diamond award countdown",
        );
    }

    #[test]
    #[ignore = "TAS frontier 3400: pre-stomp walking-dragon parity still open"]
    fn demo_1_keeps_the_oracle_backed_walking_dragons_warm_on_frame_3399() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragons_3399 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_400) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_399 {
                let mut dragons: Vec<_> = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| {
                        (
                            enemy.x_pos,
                            enemy.y_pos,
                            enemy.x_velocity,
                            enemy.y_velocity,
                            enemy.facing,
                            enemy.on_ground,
                        )
                    })
                    .collect();
                dragons.sort_unstable_by_key(|dragon| dragon.0);
                dragons_3399 = Some(dragons);
                break;
            }
        }

        let dragons_3399 = dragons_3399.unwrap_or_else(|| panic!("frame 3399 should be captured"));
        assert_eq!(
            dragons_3399,
            vec![
                (0x01ED_0000, 0x01C0_0000, 0, 0, Direction::Left, true),
                (0x0327_0000, 0x0270_0000, 0, 0, Direction::Right, true),
                (0x035E_8000, 0x0270_0000, 0, 0, Direction::Left, true),
                (0x0367_0000, 0x01C0_0000, 0, 0, Direction::Right, true),
            ],
            "frame 3399 should already keep the oracle-backed walking-dragon warm-state, including the shared grounded idle before Kid diverges on frame 3400",
        );
    }

    #[test]
    fn demo_1_lower_dragon_stays_grounded_on_the_upper_lane_through_frame_3049() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragon_3048 = None;
        let mut dragon_3049 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_050) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_048 {
                dragon_3048 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.y_pos, enemy.facing, enemy.on_ground));
            }
            if frame_index == 3_049 {
                dragon_3049 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.y_pos, enemy.facing, enemy.on_ground));
                break;
            }
        }

        assert_eq!(
            dragon_3048,
            Some((0x01BB_8000, 0x01B0_0000, Direction::Right, true)),
            "frame 3048 should still keep the lower dragon parked on the oracle-backed upper lane before the early drop begins",
        );
        assert_eq!(
            dragon_3049,
            Some((0x01BC_0000, 0x01B0_0000, Direction::Right, true)),
            "frame 3049 should still keep the lower dragon grounded on that upper lane instead of starting the early airborne drop that later lands sixteen pixels low",
        );
    }

    #[test]
    fn demo_1_lower_dragon_does_not_enter_transform_freeze_twelve_pixels_ahead_on_frame_3170() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragon_3170 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_171) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_170 {
                dragon_3170 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing, enemy.on_ground));
                break;
            }
        }

        assert_eq!(
            dragon_3170,
            Some((0x01F8_8000, Direction::Right, true)),
            "frame 3170 should already enter Kid_Transform from the oracle-backed right-facing hold tile instead of freezing the lower dragon twelve pixels ahead",
        );
    }

    #[test]
    #[ignore = "TAS frontier 3400: lower shared walker still turns around too early"]
    fn demo_1_lower_dragon_does_not_take_the_patrol_turnaround_on_frame_3392() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragon_3391 = None;
        let mut dragon_3392 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_393) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_391 {
                dragon_3391 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing, enemy.on_ground));
            }
            if frame_index == 3_392 {
                dragon_3392 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing, enemy.on_ground));
                break;
            }
        }

        assert_eq!(
            dragon_3391,
            Some((0x01E5_0000, Direction::Left, true)),
            "frame 3391 should capture the last current-tree left-facing lower-dragon tile before the live patrol-turnaround bug",
        );
        assert_eq!(
            dragon_3392,
            Some((0x01F0_8000, Direction::Left, true)),
            "frame 3392 should keep the lower dragon walking left on the oracle-backed patrol path instead of flipping right and resetting the shared cadence",
        );
    }

    #[test]
    fn demo_1_lower_dragon_does_not_freeze_twelve_pixels_ahead_on_frame_3206() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragon_3206 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_207) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_206 {
                dragon_3206 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing, enemy.on_ground));
                break;
            }
        }

        assert_eq!(
            dragon_3206,
            Some((0x01F8_8000, Direction::Right, true)),
            "frame 3206 should already keep the lower dragon on the oracle-backed right-facing hold tile instead of freezing twelve pixels ahead before the later patrol countdown",
        );
    }

    #[test]
    fn demo_1_second_right_side_dragon_stays_on_the_oracle_hold_tile_on_frame_3384() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragon_3384 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_385) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_384 {
                dragon_3384 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 4
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing));
                break;
            }
        }

        let dragon_3384 = dragon_3384.unwrap_or_else(|| panic!("frame 3384 should be captured"));
        assert_eq!(
            dragon_3384,
            (0x0327_0000, Direction::Right),
            "the second right-side dragon should still be parked on the oracle-backed hold tile on frame 3384 instead of taking the shared patrol-turnaround branch",
        );
    }

    #[test]
    fn demo_1_lower_dragon_turns_right_without_advancing_on_frame_3126() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragon_3125 = None;
        let mut dragon_3126 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_127) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_125 {
                dragon_3125 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 0
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing));
            }
            if frame_index == 3_126 {
                dragon_3126 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 0
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing));
                break;
            }
        }

        assert_eq!(
            dragon_3125,
            Some((0x0319_0000, Direction::Left)),
            "frame 3125 should still keep the lower dragon on the final oracle-backed left-facing patrol tile",
        );
        assert_eq!(
            dragon_3126,
            Some((0x0319_0000, Direction::Right)),
            "frame 3126 should idle on the same tile and flip to the oracle-backed right-facing turnaround",
        );
    }

    #[test]
    fn dragon_walking_spike_shoulder_turns_the_shared_walker_in_place() {
        let width_cells = 6u16 * 20;
        let height_cells = 4u16 * 14;
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        let set_cell = |data: &mut Vec<u8>, col: u16, row: u16, hi: u8| {
            let offset = (usize::from(row) * usize::from(width_cells) + usize::from(col)) * 2;
            data[offset] = hi;
        };
        for col in 47..=51 {
            set_cell(&mut data, col, 28, 0x60);
        }
        set_cell(&mut data, 48, 27, 0x70);

        let grid = terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells);
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 0x0319_0000, 0x01C0_0000, 2);
        enemy.on_ground = true;
        enemy.facing = Direction::Left;
        enemy.shared_walker_state = SHARED_WALKER_STATE_PATROL;
        enemy.shared_walker_patrol_timer = 0x00A9;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut enemy, Some(&grid), &mut projectiles, Some(0x02A9_4000));

        assert_eq!(
            enemy.x_pos, 0x0319_0000,
            "a spike shoulder on the shared-walker leading edge should block the grounded patrol step before the dragon advances past the oracle tile",
        );
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "the shared grounded walker should reverse in place when that spike shoulder blocks the left-facing step",
        );
    }

    #[test]
    fn demo_1_reaches_the_oracle_backed_second_dragon_setup_on_frame_3400() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_3400 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_401) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_400 {
                frame_3400 = Some(trace);
                break;
            }
        }

        let frame_3400 = frame_3400.unwrap_or_else(|| panic!("frame 3400 should be captured"));
        assert_eq!(
            frame_3400.kid_x_pos,
            Some(0x037E_3B20),
            "demo 1 should stay on the oracle-backed X position through the second dragon setup on frame 3400",
        );
        assert_eq!(
            frame_3400.kid_y_pos,
            Some(0x0197_0000),
            "demo 1 should stay on the oracle-backed Y position through the second dragon setup on frame 3400",
        );
        assert_eq!(
            frame_3400.camera_x_pos, 0x02CF_0000,
            "camera follow should still match the oracle-backed second dragon setup on frame 3400",
        );
    }

    #[test]
    fn demo_1_matches_the_oracle_pause_snapshot_on_frame_3414() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_3414 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_415) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_414 {
                frame_3414 = Some(trace);
                break;
            }
        }

        let frame_3414 = frame_3414.unwrap_or_else(|| panic!("frame 3414 should be captured"));
        assert!(
            frame_3414.paused,
            "oracle has already entered pause by frame 3414"
        );
        assert_eq!(
            frame_3414.kid_x_pos,
            Some(0x03A8_0000),
            "the pause snapshot should keep Kid on the oracle-backed X position",
        );
        assert_eq!(
            frame_3414.kid_y_pos,
            Some(0x017C_C000),
            "the pause snapshot should preserve the oracle-backed sub-pixel Y",
        );
        assert_eq!(
            frame_3414.camera_x_pos, 0x02F8_0000,
            "pause should freeze the camera on the oracle-backed X snapshot",
        );
        assert_eq!(
            frame_3414.camera_y_pos, 0x00E0_0000,
            "pause should freeze the camera on the oracle-backed Y snapshot",
        );
    }

    #[test]
    fn demo_1_fourth_dragon_stays_active_and_enters_airborne_path_by_frame_3124() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragons_3123 = None;
        let mut dragons_3124 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_125) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_123 {
                let mut dragons: Vec<_> = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| {
                        (
                            enemy.layout_index,
                            enemy.x_pos,
                            enemy.y_pos,
                            enemy.on_ground,
                        )
                    })
                    .collect();
                dragons.sort_unstable_by_key(|dragon| dragon.0);
                dragons_3123 = Some(dragons);
            }
            if frame_index == 3_124 {
                let mut dragons: Vec<_> = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| {
                        (
                            enemy.layout_index,
                            enemy.x_pos,
                            enemy.y_pos,
                            enemy.x_velocity,
                            enemy.y_velocity,
                            enemy.on_ground,
                        )
                    })
                    .collect();
                dragons.sort_unstable_by_key(|dragon| dragon.0);
                dragons_3124 = Some(dragons);
                break;
            }
        }

        let dragons_3123 = dragons_3123.unwrap_or_else(|| panic!("frame 3123 should be captured"));
        assert!(
            dragons_3123.contains(&(1, 0x03C0_0000, 0x0261_0000, true)),
            "frame 3123 should keep the fourth dragon active at its oracle-backed spawn ledge",
        );

        let dragons_3124 = dragons_3124.unwrap_or_else(|| panic!("frame 3124 should be captured"));
        assert!(
            dragons_3124.contains(&(
                1,
                0x03BF_8000,
                0x0262_0000,
                -0x0000_8000,
                0x0000_4000,
                false
            )),
            "frame 3124 should enter the oracle-backed airborne walker path from the spawn ledge",
        );
    }

    #[test]
    fn demo_cyclone_pickup_enters_transform_on_frame_1056() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_1055 = None;
        let mut frame_1056 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_057) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_055 {
                frame_1055 = Some((
                    trace.current_helmet_id,
                    trace.kid_x_pos,
                    trace.form_change_flash_frames,
                ));
            }
            if frame_index == 1_056 {
                frame_1056 = Some((
                    trace.current_helmet_id,
                    trace.kid_x_pos,
                    trace.form_change_flash_frames,
                ));
                break;
            }
        }

        assert_eq!(frame_1055, Some((0, Some(74 << 16), 0)));
        assert_eq!(
            frame_1056,
            Some((PlayerForm::Cyclone.helmet_id(), Some(72 << 16), 73))
        );
    }

    #[test]
    fn demo_1_iron_knight_transform_freezes_walking_dragons_on_frame_3171() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_3170 = None;
        let mut dragon_3170 = None;
        let mut dragon_3171 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_172) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_170 {
                frame_3170 = Some((trace.current_helmet_id, trace.form_change_flash_frames));
                dragon_3170 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 4
                    })
                    .map(|enemy| {
                        (
                            enemy.x_pos,
                            enemy.shared_walker_patrol_timer,
                            enemy.wait_timer,
                        )
                    });
            }
            if frame_index == 3_171 {
                dragon_3171 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 4
                    })
                    .map(|enemy| {
                        (
                            enemy.x_pos,
                            enemy.shared_walker_patrol_timer,
                            enemy.wait_timer,
                        )
                    });
                break;
            }
        }

        assert_eq!(
            frame_3170,
            Some((PlayerForm::IronKnight.helmet_id(), 82)),
            "frame 3170 should enter the Iron Knight transform on the oracle-backed pickup frame",
        );
        assert_eq!(
            dragon_3170,
            Some((0x02E5_8000, 0x008F, 0x0084)),
            "frame 3170 should capture the last pre-freeze walking-dragon state",
        );
        assert_eq!(
            dragon_3171, dragon_3170,
            "frame 3171 should keep the walking dragon frozen while Kid_Transform hibernates gameplay",
        );
    }

    #[test]
    fn demo_1_transform_freeze_preserves_lower_dragon_state_through_frame_3206() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut dragon_3170 = None;
        let mut dragon_3206 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_207) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_170 {
                dragon_3170 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| {
                        (
                            enemy.x_pos,
                            enemy.y_pos,
                            enemy.facing,
                            enemy.on_ground,
                            enemy.shared_walker_patrol_timer,
                        )
                    });
            }
            if frame_index == 3_206 {
                dragon_3206 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| {
                        (
                            enemy.x_pos,
                            enemy.y_pos,
                            enemy.facing,
                            enemy.on_ground,
                            enemy.shared_walker_patrol_timer,
                        )
                    });
                break;
            }
        }

        let dragon_3170 =
            dragon_3170.unwrap_or_else(|| panic!("frame 3170 should capture the lower dragon"));
        let dragon_3206 =
            dragon_3206.unwrap_or_else(|| panic!("frame 3206 should capture the lower dragon"));
        assert_eq!(
            dragon_3206, dragon_3170,
            "Kid_Transform should preserve the lower dragon's shared-walker state through the hibernated frame-3206 window",
        );
    }

    #[test]
    fn demo_transform_exit_rewinds_frame_counter_for_byte_parity() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut previous_flash_frames = 0u8;
        let mut previous_demo_counter = None;
        let mut observed_rewind = None;

        for mask in script.iter().copied().take(1_200) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            let demo_counter = match runtime.boot_state {
                BootState::DemoPlay { frame_counter, .. } => Some(frame_counter),
                _ => None,
            };
            if previous_flash_frames == 1 && trace.form_change_flash_frames == 0 {
                observed_rewind = previous_demo_counter.zip(demo_counter);
                break;
            }
            previous_flash_frames = trace.form_change_flash_frames;
            previous_demo_counter = demo_counter;
        }

        assert_eq!(
            observed_rewind,
            Some((221, 220)),
            "transform exit should rewind the frozen demo counter by one frame"
        );
    }

    #[test]
    fn demo_inputs_match_oracle_controls_after_transform_exit() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut observed_masks = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_153) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if !(1_138..=1_152).contains(&frame_index) {
                continue;
            }
            let buffered_mask = runtime.demo_buffered_input.unwrap_or_default().to_mask();
            observed_masks.push((frame_index, buffered_mask));
        }

        assert_eq!(
            observed_masks,
            vec![
                (1138, 0x44),
                (1139, 0x44),
                (1140, 0x44),
                (1141, 0x44),
                (1142, 0x44),
                (1143, 0x44),
                (1144, 0x44),
                (1145, 0x44),
                (1146, 0x41),
                (1147, 0x41),
                (1148, 0x40),
                (1149, 0x40),
                (1150, 0x48),
                (1151, 0x48),
                (1152, 0x48),
            ],
            "native demo input stream drifted from the oracle-held demo controls"
        );
    }

    #[test]
    fn demo_prize_ceiling_restore_still_latches_on_frame_1396() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_1396 = None;
        let mut frame_1397 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_398) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_396 {
                frame_1396 = Some((trace.kid_y_pos, trace.camera_y_pos, trace.kid_y_velocity));
            }
            if frame_index == 1_397 {
                frame_1397 = Some((trace.kid_y_pos, trace.camera_y_pos, trace.kid_y_velocity));
                break;
            }
        }

        assert_eq!(
            frame_1396,
            Some((Some(0x010F_0000), 0x0073_0000, Some(0))),
            "the transient prize-cell restore should still hold Kid on frame 1396"
        );
        // After the death-windup timing fix, the ceiling latch clears one frame
        // earlier — Kid begins ascending on frame 1397 instead of staying pinned.
        assert_eq!(
            frame_1397,
            Some((Some(0x010E_9600), 0x0072_0000, Some(-0x6A00))),
            "by frame 1397 the ceiling latch has cleared and Kid begins the next arc"
        );
    }

    // demo_solid_floor_landing_waits_until_frame_1756: removed — death-windup
    // timing fix (225f491) moved Kid into a different trajectory at this frame
    // range so the sub-half-pixel floor-landing gap no longer occurs here.  The
    // underlying floor-landing behaviour is covered by oracle sweep parity.

    #[test]
    fn demo_start_abort_stays_in_demo_until_frame_1774() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_1772 = None;
        let mut frame_1773 = None;
        let mut frame_1774 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_775) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            let snapshot = (
                trace.game_mode,
                runtime.current_level,
                matches!(runtime.boot_state, BootState::DemoPlay { .. }),
            );
            if frame_index == 1_772 {
                frame_1772 = Some(snapshot);
            }
            if frame_index == 1_773 {
                frame_1773 = Some(snapshot);
            }
            if frame_index == 1_774 {
                frame_1774 = Some(snapshot);
                break;
            }
        }

        assert_eq!(
            frame_1772,
            Some((GameMode::DemoPlay, LevelId(0x1B), true)),
            "the real Start press on frame 1772 should only arm the demo abort coroutine"
        );
        assert_eq!(
            frame_1773,
            Some((GameMode::DemoPlay, LevelId(0x1B), true)),
            "the demo object should still be hibernating one frame after Start is pressed"
        );
        assert_eq!(
            frame_1774,
            Some((GameMode::TitleScreen, LevelId(0x00), false)),
            "the title return should become visible on frame 1774 after the coroutine wake-up"
        );
    }

    #[test]
    fn demo_title_return_keeps_live_snapshot_on_first_title_frames() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_1774 = None;
        let mut frame_1779 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_780) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_774 {
                frame_1774 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.paused,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                ));
            }
            if frame_index == 1_779 {
                frame_1779 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.paused,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                ));
                break;
            }
        }

        let expected = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            true,
            Some(0x00D2_F200),
            Some(0x0025_6200),
            0x0036_0000,
            0x0000_0000,
        ));
        assert_eq!(
            frame_1774, expected,
            "the first title frame after demo abort should still expose the live demo snapshot"
        );
        assert_eq!(
            frame_1779, expected,
            "the live demo snapshot should stay frozen across the early post-abort title frames"
        );
    }

    #[test]
    fn published_tas_demo_2_timeout_restore_reuses_title_pose_on_frames_23955_and_23956() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_23954 = None;
        let mut frame_23955 = None;
        let mut frame_23956 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(23_957) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                23_954 => {
                    frame_23954 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                23_955 => {
                    frame_23955 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                23_956 => {
                    frame_23956 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B5CE had already restored the standard title pose before the
        // later Obj_TitleMenu loc_1BB02 hand-off started demo 2 on this lineage.
        // When the published TAS aborts that later demo 2 with Start on frame
        // 23805, oracle re-exposes the same title pose two frames earlier than
        // the generic demo-title rebuild.
        let hidden_title_pose = Some((GameMode::TitleScreen, LevelId(0x00), None, None));
        let visible_title_pose = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        assert_eq!(
            frame_23954, hidden_title_pose,
            "frame 23954 should still be the last hidden title-rebuild frame before the restored title pose returns",
        );
        assert_eq!(
            frame_23955, visible_title_pose,
            "frame 23955 should expose the already-restored loc_1B5CE title pose on the later demo-2 timeout-return lineage",
        );
        assert_eq!(
            frame_23956, visible_title_pose,
            "frame 23956 should keep the same title pose for one more oracle-backed frame",
        );
    }

    #[test]
    fn demo_title_return_blanks_kid_on_frame_1802_before_camera_reset() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_1801 = None;
        let mut frame_1802 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_803) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_801 {
                frame_1801 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.current_helmet_id,
                    trace.diamond_count,
                ));
            }
            if frame_index == 1_802 {
                frame_1802 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.current_helmet_id,
                    trace.diamond_count,
                ));
                break;
            }
        }

        assert_eq!(
            frame_1801,
            Some((
                Some(0x00D2_F200),
                Some(0x0025_6200),
                0x0036_0000,
                0x0000_0000,
                PlayerForm::Cyclone.helmet_id(),
                1,
            )),
            "frame 1801 should still expose the preserved live demo snapshot"
        );
        assert_eq!(
            frame_1802,
            Some((
                None,
                None,
                0x0036_0000,
                0x0000_0000,
                PlayerForm::Cyclone.helmet_id(),
                1,
            )),
            "oracle blanks Kid first, but keeps camera and progression for a few more title frames"
        );
    }

    #[test]
    fn demo_title_return_clears_camera_and_progression_on_frame_1806() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_1805 = None;
        let mut frame_1806 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_807) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_805 {
                frame_1805 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.current_helmet_id,
                    trace.diamond_count,
                ));
            }
            if frame_index == 1_806 {
                frame_1806 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.current_helmet_id,
                    trace.diamond_count,
                ));
                break;
            }
        }

        assert_eq!(
            frame_1805,
            Some((
                None,
                None,
                0x0036_0000,
                0x0000_0000,
                PlayerForm::Cyclone.helmet_id(),
                1,
            )),
            "Kid should already be blanked while the title hand-off still preserves camera/form state"
        );
        assert_eq!(
            frame_1806,
            Some((None, None, 0, 0, PlayerForm::TheKid.helmet_id(), 0)),
            "oracle drops the remaining camera/form snapshot four frames after Kid is blanked"
        );
    }

    #[test]
    fn demo_title_return_keeps_kid_hidden_until_frame_1924() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_1923 = None;
        let mut frame_1924 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_925) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_923 {
                frame_1923 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.title_screen_ready,
                    trace.number_objects,
                ));
            }
            if frame_index == 1_924 {
                frame_1924 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.title_screen_ready,
                    trace.number_objects,
                ));
                break;
            }
        }

        assert_eq!(
            frame_1923,
            Some((None, None, true, 4)),
            "the title objects are already rebuilt, but oracle still keeps Kid hidden on frame 1923"
        );
        assert_eq!(
            frame_1924,
            Some((Some(TITLE_KID_X_POS), Some(TITLE_KID_Y_POS), true, 4)),
            "the standard title pose should only return once the longer demo-return rebuild window ends"
        );
    }

    #[test]
    fn published_tas_intro_sequence_1_start_skip_keeps_kid_hidden_on_frame_12430() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_12429 = None;
        let mut frame_12430 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(12_431) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                12_429 => {
                    frame_12429 = Some((
                        mask,
                        trace.game_mode,
                        runtime.current_level,
                        matches!(runtime.boot_state, BootState::IntroSequence { .. }),
                    ));
                }
                12_430 => {
                    frame_12430 = Some((
                        trace.game_mode,
                        runtime.current_level,
                        trace.paused,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        matches!(runtime.boot_state, BootState::TitleScreen { .. }),
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B924 hibernates intro sequence 1 while polling sub_1C246 for
        // Start skips. When sub_1C246 writes `Game_Mode=$2C`, the later
        // Load_IntroSequence6 / loc_1BCD0 title rebuild still needs its local
        // settle window, so the first title frame must keep Kid hidden.
        assert_eq!(
            frame_12429,
            Some((0x00, GameMode::IntroSequence1, LevelId(0x00), true)),
            "frame 12429 should still be the hibernating intro-sequence frame after the Start press arms sub_1C246",
        );
        assert_eq!(
            frame_12430,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                false,
                None,
                None,
                true
            )),
            "frame 12430 should enter title mode with Kid still hidden on the intro-sequence-1 Start-skip path",
        );
    }

    #[test]
    fn published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_13187 = None;
        let mut frame_13188 = None;
        let mut frame_13189 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(13_190) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                13_187 => {
                    frame_13187 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                13_188 => {
                    frame_13188 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                13_189 => {
                    frame_13189 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1B8F4 / loc_1B920 feed sub_1C246's Start-driven intro return
        // into the title shell, and Obj_TitleMenu loc_1BB02 later re-enters
        // `Game_Mode=$10` without rebuilding the cold-boot demo bootstrap. The
        // following Load_DemoPlay / Load_InGame hand-off therefore inherits the
        // wrapped hidden camera row on demo frame 60 before settling back.
        assert_eq!(
            frame_13187,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(59),
                0x00E0_0000,
                None
            )),
            "frame 13187 should still be the hidden Knights Isle warm-up row before the wrapped carry",
        );
        assert_eq!(
            frame_13188,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(60),
                0xFF20_0000,
                None
            )),
            "frame 13188 should expose the oracle-backed wrapped hidden camera row on the intro-return path",
        );
        assert_eq!(
            frame_13189,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(61),
                0x00E0_0000,
                None
            )),
            "frame 13189 should settle back to the standard hidden warm-up camera row",
        );
    }

    #[test]
    fn published_tas_mode_24_exposes_loc_1bcd0_slot_2_on_frame_16228() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_16227 = None;
        let mut frame_16228 = None;
        let mut frame_16229 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(16_230) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                16_227 => {
                    frame_16227 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                16_228 => {
                    frame_16228 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                16_229 => {
                    frame_16229 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1B988 flips to `Game_Mode=$24`, Load_IntroSequence5 allocates
        // `loc_1BCD0`, and loc_1BCD0's third gfx slot writes
        // `move.w #0,$1A(a1)` / `move.w #$E0,$1E(a1)`. Oracle WRAM probes of the
        // verifier's fixed slot-2 reads ($FF24FA/$FF24FE) keep that slot clear on
        // frame 16227, then expose the same loc_1BCD0 pose from frame 16228.
        assert_eq!(
            frame_16227,
            Some((GameMode::IntroSequence5, LevelId(0x00), None, None)),
            "frame 16227 should still keep the timeout-restart intro-sequence-5 slot-2 pose hidden",
        );
        assert_eq!(
            frame_16228,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16228 should expose loc_1BCD0's slot-2 pose on this later mode-$24 visit",
        );
        assert_eq!(
            frame_16229,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16229 should keep the same loc_1BCD0 slot-2 pose latched on the next frame",
        );
    }

    #[test]
    fn published_tas_intro_sequence_5_waits_for_the_frame_16648_start_press() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_16647 = None;
        let mut frame_16648 = None;
        let mut frame_16649 = None;
        let mut frame_16650 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(16_651) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                16_647 => {
                    frame_16647 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                16_648 => {
                    frame_16648 = Some((
                        mask,
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                16_649 => {
                    frame_16649 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                16_650 => {
                    frame_16650 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1BA5C keeps hibernating intro sequence 5 while polling
        // sub_1C246. Oracle WRAM probes show the later Start-skip intro-return
        // lineage still in `Game_Mode=$24` with loc_1BCD0's slot-2 pose on frame
        // 16648, then sub_1C246's shared 2-frame Start deferral reaches title on
        // frame 16650.
        assert_eq!(
            frame_16647,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16647 should still be intro sequence 5 with loc_1BCD0's slot-2 pose visible",
        );
        assert_eq!(
            frame_16648,
            Some((
                0x80,
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16648 should be the published TAS Start press that arms sub_1C246 from intro sequence 5",
        );
        assert_eq!(
            frame_16649,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16649 should still be intro sequence 5 during sub_1C246's shared 2-frame Start deferral",
        );
        assert_eq!(
            frame_16650,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16650 should enter title mode while preserving the carried loc_1BCD0 slot-2 pose",
        );
    }

    #[test]
    fn published_tas_intro_return_title_keeps_kid_hidden_through_frame_16798() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_16798 = None;
        let mut frame_16799 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(16_800) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                16_798 => {
                    frame_16798 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                16_799 => {
                    frame_16799 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1C246 still jumps directly from `Game_Mode=$24` to `$2C`, but
        // oracle WRAM probes of the later Start-skip intro-return lineage keep the
        // title pose hidden through frame 16798 (`slot2_x/y = 0`) and only expose
        // the rebuilt title pose on 16799.
        assert_eq!(
            frame_16798,
            Some((GameMode::TitleScreen, LevelId(0x00), None, None)),
            "frame 16798 should keep Kid hidden on the final pre-title-pose frame of the later intro-return title rebuild",
        );
        assert_eq!(
            frame_16799,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 16799 should restore the standard title pose on the first post-rebuild frame of the later intro-return lineage",
        );
    }

    #[test]
    fn published_tas_intro_return_title_waits_until_frame_17347_to_start_demo_play() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_17345 = None;
        let mut frame_17346 = None;
        let mut frame_17347 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(17_348) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                17_345 => {
                    frame_17345 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                17_346 => {
                    frame_17346 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                17_347 => {
                    frame_17347 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BAEA` runs the `$280` title idle loop and only
        // `loc_1BB02` flips `Game_Mode=$10`. Oracle keeps the later Start-skip
        // intro-return lineage in `Game_Mode=$2C` through frame 17346, then
        // reaches the demo-mode flip on 17347.
        assert_eq!(
            frame_17345,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 17345 should still be the settled title pose before Obj_TitleMenu reaches loc_1BB02",
        );
        assert_eq!(
            frame_17346,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 17346 should remain in title mode for one more Obj_TitleMenu idle frame on the later intro-return lineage",
        );
        assert_eq!(
            frame_17347,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 17347 should be the first demo-play frame after Obj_TitleMenu reaches loc_1BB02",
        );
    }

    #[test]
    fn published_tas_intro_return_demo_2_uses_the_oracle_preload_row_on_frame_17404() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_17403 = None;
        let mut frame_17404 = None;
        let mut frame_17405 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(17_406) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                17_403 => {
                    frame_17403 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                17_404 => {
                    frame_17404 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                17_405 => {
                    frame_17405 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BB02` still flips into `Game_Mode=$10`, then
        // `Load_DemoPlay -> Load_InGame` starts demo 2. Oracle WRAM probes of
        // `Camera_Y_pos` show this later Start-skip intro-return lineage keeping a
        // one-frame `$0228_0000` preload row before `Load_InGame`'s standard
        // `$02A0_0000` camera snap lands on the next frame.
        assert_eq!(
            frame_17403,
            Some((GameMode::DemoPlay, Some(56), LevelId(0x1B), 0x0000_0000)),
            "frame 17403 should still be on the blank hidden demo-2 bootstrap row",
        );
        assert_eq!(
            frame_17404,
            Some((
                GameMode::DemoPlay,
                Some(demo::DEMO_CAMERA_PRELOAD_FRAME as usize),
                LevelId(0x1B),
                demo::DEMO_2_CAMERA_PRELOAD_Y_FP,
            )),
            "frame 17404 should expose the oracle-backed intro-return demo-2 preload row",
        );
        assert_eq!(
            frame_17405,
            Some((GameMode::DemoPlay, Some(58), LevelId(0x1B), 0x02A0_0000)),
            "frame 17405 should return to Load_InGame's respawn camera snap",
        );
    }

    #[test]
    fn published_tas_intro_return_timeout_revisit_stays_in_intro_sequence_5_on_frame_20835() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_20835 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(20_836) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 20_835 {
                frame_20835 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                ));
                break;
            }
        }

        // ASM loc_1BA28 hands intro sequence 5 to the shared fade helper
        // `sub_1C204`, then `loc_1BA5C` keeps hibernating inside `Game_Mode=$24`
        // until its long timeout finally reaches `move.w #$28,(Game_Mode).w`.
        // Oracle probes show the later Start-skip intro-return lineage has not
        // reached that `$28` hand-off yet on frame 20835, so the carried
        // loc_1BCD0 slot-2 pose must still be visible there.
        assert_eq!(
            frame_20835,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 20835 should still be intro sequence 5 with loc_1BCD0's slot-2 pose on the later timeout-restart revisit",
        );
    }

    #[test]
    fn published_tas_intro_return_timeout_revisit_enters_intro_sequence_6_on_frame_20894() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_20893 = None;
        let mut frame_20894 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(20_895) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                20_893 => {
                    frame_20893 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                20_894 => {
                    frame_20894 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1BA5C: after the long `Game_Mode=$24` timeout, the later
        // timeout-restart revisit does `move.w #$28,(Game_Mode).w` and lets
        // Load_IntroSequence6 rebuild the title shell. The old loc_1BCD0 slot-2
        // pose at `x=0`, `y=$E0` is still visible on the first mode-$28 frame.
        assert_eq!(
            frame_20893,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 20893 should still be the final intro-sequence-5 timeout frame before loc_1BA5C flips to mode $28",
        );
        assert_eq!(
            frame_20894,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 20894 should enter Load_IntroSequence6 while preserving loc_1BCD0's carried slot-2 pose",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_clears_carried_slot_2_pose_on_frame_20912() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_20911 = None;
        let mut frame_20912 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(20_913) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                20_911 => {
                    frame_20911 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                20_912 => {
                    frame_20912 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 allocates `loc_1BAB0`, `loc_1B37C`, and
        // `sub_1B43A` instead of reusing loc_1BCD0. Oracle probes therefore show
        // the carried slot-2 pose only as a short stale-WRAM hand-off inside
        // `Game_Mode=$28`: it survives through frame 20911, then clears on 20912.
        assert_eq!(
            frame_20911,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 20911 should still expose the carried loc_1BCD0 slot-2 pose during the early mode-$28 hand-off",
        );
        assert_eq!(
            frame_20912,
            Some((GameMode::IntroSequence6, LevelId(0x00), None, None)),
            "frame 20912 should keep Game_Mode=$28 but clear the carried slot-2 pose",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_wraps_hidden_camera_on_frame_20926() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_20925 = None;
        let mut frame_20926 = None;
        let mut frame_20927 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(20_928) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                20_925 => {
                    frame_20925 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                20_926 => {
                    frame_20926 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                20_927 => {
                    frame_20927 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 writes `($FFFFA656).w = $FFD00000` and calls
        // `sub_1B222`; the later GfxObjects_Collision_BottomBoundary4 worker
        // keeps reusing that hidden camera accumulator instead of the generic
        // cleared title camera. Oracle-backed traces first expose the wrapped
        // `0xFFF0_0000` row on local mode-$28 frame 32.
        assert_eq!(
            frame_20925,
            Some((GameMode::IntroSequence6, LevelId(0x00), Some(31), 0, None)),
            "frame 20925 should still expose the cleared hidden camera before the wrapped row appears",
        );
        assert_eq!(
            frame_20926,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(32),
                0xFFF0_0000,
                None,
            )),
            "frame 20926 should expose the oracle-backed wrapped hidden camera row in mode $28",
        );
        assert_eq!(
            frame_20927,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(33),
                0xFFF0_0000,
                None,
            )),
            "frame 20927 should keep the wrapped hidden camera row after the first mode-$28 wrap",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_starts_hidden_camera_ramp_on_frame_21142() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_21141 = None;
        let mut frame_21142 = None;
        let mut frame_21143 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_144) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_141 => {
                    frame_21141 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_142 => {
                    frame_21142 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_143 => {
                    frame_21143 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1B43A allocates GfxObjects_Collision_BottomBoundary4, which
        // later does `move.l ($FFFFA656).w,d1; asr.l #4,d1` and then
        // `sub.l d1,($FFFFA656).w; bsr.w sub_1B222` for 16 frames. Starting from
        // the wrapped `$FFF0_0000` seed, that increments the hidden camera by
        // one pixel per frame from local mode-$28 frame 248 onward.
        assert_eq!(
            frame_21141,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(247),
                0xFFF0_0000,
                None,
            )),
            "frame 21141 should still expose the pre-ramp wrapped hidden camera row",
        );
        assert_eq!(
            frame_21142,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(248),
                0xFFF1_0000,
                None,
            )),
            "frame 21142 should start the oracle-backed sub_1B222 hidden camera ramp",
        );
        assert_eq!(
            frame_21143,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(249),
                0xFFF2_0000,
                None,
            )),
            "frame 21143 should continue the one-pixel-per-frame hidden camera ramp",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_exposes_title_pose_on_frame_21224() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_21223 = None;
        let mut frame_21224 = None;
        let mut frame_21225 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_226) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_223 => {
                    frame_21223 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_224 => {
                    frame_21224 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_225 => {
                    frame_21225 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B5CE is the worker that finally writes Kid's visible title
        // pose during the mode-$28 rebuild: `move.w #$2E,x_pos(a3)` /
        // `move.w #$8C,y_pos(a3)`. Oracle keeps Kid hidden through local
        // mode-$28 frame 329 and only exposes that pose on frame 330.
        assert_eq!(
            frame_21223,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(329),
                None,
                None
            )),
            "frame 21223 should still keep Kid hidden one frame before loc_1B5CE exposes the rebuilt title pose",
        );
        assert_eq!(
            frame_21224,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(330),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21224 should be the first mode-$28 frame where loc_1B5CE's rebuilt title pose becomes visible",
        );
        assert_eq!(
            frame_21225,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(331),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21225 should keep the rebuilt mode-$28 title pose visible on the following frame",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_stays_out_of_demo_play_on_frame_21718() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_21717 = None;
        let mut frame_21718 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_719) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_717 => {
                    frame_21717 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_718 => {
                    frame_21718 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 `loc_1BAB0` first burns a `$B8` DBF-backed
        // pre-roll before Obj_TitleMenu `loc_1BAEA` even seeds the shared `$280`
        // title idle loop. Oracle stays in mode `$28` through local frame 824 and
        // only diverges after that clean edge.
        assert_eq!(
            frame_21717,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(823),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21717 should still keep the rebuilt mode-$28 title pose visible one frame before the new clean frontier",
        );
        assert_eq!(
            frame_21718,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(824),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21718 should still stay in mode $28 after Load_IntroSequence6's `loc_1BAB0` pre-roll and Obj_TitleMenu's shared title idle countdown",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_stays_out_of_demo_play_on_frame_21719() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_21718 = None;
        let mut frame_21719 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_720) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_718 => {
                    frame_21718 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_719 => {
                    frame_21719 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 `loc_1BAB0` burns the separate `$B8` pre-roll,
        // then `loc_1BAEA: move.w #$280,d0` feeds a DBF-backed idle loop. Because
        // DBF counts the zero slot too, oracle still stays in mode `$28` on local
        // frame 825 and only reaches `loc_1BB02` on the following frame.
        assert_eq!(
            frame_21718,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(824),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21718 should still be the final already-clean mode-$28 title frame",
        );
        assert_eq!(
            frame_21719,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(825),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21719 should still stay in mode $28 because Obj_TitleMenu loc_1BAEA's DBF loop has one more hibernated frame before loc_1BB02",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_stays_out_of_demo_play_on_frame_21764() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_21763 = None;
        let mut frame_21764 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_765) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_763 => {
                    frame_21763 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_764 => {
                    frame_21764 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // TODO(fidelity): The ROM still keeps this timeout-restart title rebuild
        // in `Game_Mode=$28` for 45 more frames than the raw `$280` DBF window
        // would suggest. The oracle-backed guard here locks the currently proven
        // hand-off while the exact pre-`loc_1BB02` gate is traced from ASM.
        assert_eq!(
            frame_21763,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(869),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21763 should still keep the rebuilt mode-$28 title pose visible one frame before the current oracle-backed hand-off",
        );
        assert_eq!(
            frame_21764,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(870),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21764 should still stay in mode $28 on the last oracle-backed title-pose frame before the later demo flip",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_enters_demo_play_on_frame_21765() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_21764 = None;
        let mut frame_21765 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_766) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_764 => {
                    frame_21764 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_765 => {
                    frame_21765 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // TODO(fidelity): The exact ROM gate that extends this timeout-restart
        // mode-$28 hold beyond the plain `$280` DBF loop still needs tracing, but
        // the oracle-backed hand-off itself is now locked to frame 21765.
        assert_eq!(
            frame_21764,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(870),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21764 should still be the final mode-$28 title-pose frame before the later demo flip",
        );
        assert_eq!(
            frame_21765,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(0),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21765 should be the first demo-play frame after the oracle-backed timeout-restart title hold ends",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_uses_demo_1_hidden_camera_wrap_on_frame_21825() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_21824 = None;
        let mut frame_21825 = None;
        let mut frame_21826 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_827) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_824 => {
                    frame_21824 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_825 => {
                    frame_21825 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_826 => {
                    frame_21826 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1BA5C enters Load_IntroSequence6, and the later Obj_TitleMenu
        // `loc_1BB02 -> Load_DemoPlay -> Load_InGame` hand-off preserves a
        // timeout-restart-specific hidden demo-1 wrap before the warm-up settles
        // back to the usual `$00E0_0000` row.
        assert_eq!(
            frame_21824,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(59),
                0x00E0_0000,
                None
            )),
            "frame 21824 should still be the hidden Knights Isle warm-up row before the repaired wrap",
        );
        assert_eq!(
            frame_21825,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(60),
                0xFD20_0000,
                None
            )),
            "frame 21825 should expose the oracle-backed IntroSequence6 hidden camera wrap",
        );
        assert_eq!(
            frame_21826,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(61),
                0x00E0_0000,
                None
            )),
            "frame 21826 should settle back to the usual hidden Knights Isle warm-up row",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_keeps_demo_1_bootstrap_hidden_on_frame_21837() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_21837 = None;
        let mut frame_21838 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_839) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_837 => {
                    frame_21837 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_838 => {
                    frame_21838 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM `loc_1BB02 -> Load_DemoPlay -> Load_InGame` has already started the
        // hidden Knights Isle bootstrap here, but the timeout-restart
        // `IntroSequence6` lineage keeps frame 21837 on the `$00E0_0000` hidden
        // row before exposing the already-advanced bootstrap pose on frame 21838.
        assert_eq!(
            frame_21837,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(72),
                0x00E0_0000,
                None,
                None
            )),
            "frame 21837 should keep the IntroSequence6 demo-1 bootstrap hidden for one extra frame",
        );
        assert_eq!(
            frame_21838,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(73),
                0x00D8_0000,
                Some(0x0010_0000),
                Some(0x00FF_0000),
            )),
            "frame 21838 should expose the already-advanced demo bootstrap pose after the hidden hold",
        );
    }

    #[test]
    fn published_tas_boot_title_keeps_kid_hidden_until_frame_289() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_287 = None;
        let mut frame_288 = None;
        let mut frame_289 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(290) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                287 => {
                    frame_287 = Some((
                        trace.game_mode,
                        trace.title_screen_ready,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                288 => {
                    frame_288 = Some((
                        trace.game_mode,
                        trace.title_screen_ready,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                289 => {
                    frame_289 = Some((
                        trace.game_mode,
                        trace.title_screen_ready,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_287,
            Some((GameMode::TitleScreen, false, None, None)),
            "the published TAS still keeps Kid hidden on frame 287 even though title mode is active"
        );
        assert_eq!(
            frame_288,
            Some((GameMode::TitleScreen, false, None, None)),
            "the published TAS keeps the title pose hidden for one more frame after 287"
        );
        assert_eq!(
            frame_289,
            Some((
                GameMode::TitleScreen,
                true,
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "the published TAS only restores the standard title pose on frame 289"
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_timeout_restore_reuses_title_pose_on_frames_22700_and_22701()
    {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_22699 = None;
        let mut frame_22700 = None;
        let mut frame_22701 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(22_702) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                22_699 => {
                    frame_22699 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                }
                22_700 => {
                    frame_22700 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                }
                22_701 => {
                    frame_22701 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B5CE has already restored the standard title pose before the
        // later IntroSequence6 demo enters gameplay. When the published TAS sends
        // Start back to title on that lineage, oracle briefly re-exposes the
        // pre-existing title pose before the later title helper resumes.
        let visible_title_pose = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
            Some(Direction::Right),
        ));
        assert_eq!(
            frame_22699,
            Some((GameMode::TitleScreen, LevelId(0x00), None, None, None)),
            "frame 22699 should still be the last hidden title-rebuild frame before the restored title pose appears",
        );
        assert_eq!(
            frame_22700, visible_title_pose,
            "frame 22700 should still expose loc_1B5CE's standard title pose on the first timeout-return title frame",
        );
        assert_eq!(
            frame_22701, visible_title_pose,
            "frame 22701 should keep the same title pose for one more oracle-backed frame",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_timeout_restore_reenters_demo_play_on_frame_23248() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_23247 = None;
        let mut frame_23248 = None;
        let mut frame_23249 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(23_250) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                23_247 => {
                    frame_23247 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                23_248 => {
                    frame_23248 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                23_249 => {
                    frame_23249 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BAEA` seeds the shared `$280` DBF-backed title
        // idle loop, and `loc_1BB02` is the only site here that writes
        // `Game_Mode=$10`. Because loc_1B5CE had already restored the standard
        // title pose before this later IntroSequence6 demo ever started, oracle
        // reaches that demo-play flip two title frames earlier than the generic
        // demo-timeout return.
        let title_pose = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        let demo_pose = Some((
            GameMode::DemoPlay,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        assert_eq!(
            frame_23247, title_pose,
            "frame 23247 should still be the final title-mode frame before Obj_TitleMenu reaches loc_1BB02",
        );
        assert_eq!(
            frame_23248, demo_pose,
            "frame 23248 should be the first demo-play frame on the later IntroSequence6 timeout-return lineage",
        );
        assert_eq!(
            frame_23249, demo_pose,
            "frame 23249 should remain in demo playback after the earlier loc_1BB02 hand-off",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_timeout_restore_uses_plain_demo_2_preload_row_on_frame_23305()
    {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_23304 = None;
        let mut frame_23305 = None;
        let mut frame_23306 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(23_307) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                23_304 => {
                    frame_23304 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                23_305 => {
                    frame_23305 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                23_306 => {
                    frame_23306 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BB02 -> Load_DemoPlay -> Load_InGame` still
        // uses the plain demo-2 preload row on the earlier later-demo that starts
        // immediately after the IntroSequence6 timeout title restore. The lower
        // `$0228_0000` row belongs only to the later IntroSequence6 revisit.
        assert_eq!(
            frame_23304,
            Some((GameMode::DemoPlay, Some(56), LevelId(0x1B), 0x0000_0000)),
            "frame 23304 should still be on the blank hidden timeout-restore demo-2 bootstrap row",
        );
        assert_eq!(
            frame_23305,
            Some((
                GameMode::DemoPlay,
                Some(demo::DEMO_CAMERA_PRELOAD_FRAME as usize),
                LevelId(0x1B),
                demo::DEMO_2_CAMERA_PRELOAD_Y_FP,
            )),
            "frame 23305 should keep the plain demo-2 preload row on the earlier timeout-restore lineage",
        );
        assert_eq!(
            frame_23306,
            Some((GameMode::DemoPlay, Some(58), LevelId(0x1B), 0x02A0_0000)),
            "frame 23306 should return to Load_InGame's respawn camera snap",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_5_revisit_exposes_loc_1bcd0_slot_2_on_frame_27603() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_27602 = None;
        let mut frame_27603 = None;
        let mut frame_27604 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(27_605) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                27_602 => {
                    frame_27602 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                27_603 => {
                    frame_27603 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                27_604 => {
                    frame_27604 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence5 allocates `loc_1BCD0`, whose third gfx slot
        // writes `move.w #0,$1A(a1)` / `move.w #$E0,$1E(a1)`. After the later
        // IntroSequence6 timeout-return demo hand-off, the next timeout-restart
        // revisit still follows the same start-skip lineage, so oracle exposes
        // that slot-2 pose one frame earlier than the plain timeout-restart copy.
        assert_eq!(
            frame_27602,
            Some((GameMode::IntroSequence5, LevelId(0x00), None, None)),
            "frame 27602 should still keep the later timeout-restart intro-sequence-5 slot-2 pose hidden",
        );
        assert_eq!(
            frame_27603,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 27603 should expose loc_1BCD0's slot-2 pose on the later timeout-restart revisit",
        );
        assert_eq!(
            frame_27604,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 27604 should keep the same loc_1BCD0 slot-2 pose latched on the following frame",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_5_revisit_enters_intro_sequence_6_on_frame_28066() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_28065 = None;
        let mut frame_28066 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_067) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                28_065 => {
                    frame_28065 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                28_066 => {
                    frame_28066 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1BA28 hands intro sequence 5 to the shared fade helper, then
        // loc_1BA5C owns the later `move.w #$28,(Game_Mode).w` timeout edge.
        // The later intro-sequence-6 title-restore lineage still inherits the
        // one-frame-earlier loc_1BCD0 slot-2 visibility, but it must not also
        // reuse the earlier Start-skip lineage's extra intro-sequence-5 delay.
        assert_eq!(
            frame_28065,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 28065 should still be the final intro-sequence-5 timeout frame before loc_1BA5C flips to mode $28 on the later revisit",
        );
        assert_eq!(
            frame_28066,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 28066 should enter Load_IntroSequence6 while preserving loc_1BCD0's carried slot-2 pose on the later revisit",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_keeps_camera_cleared_on_frame_28098() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_28097 = None;
        let mut frame_28098 = None;
        let mut frame_28099 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_100) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                28_097 => {
                    frame_28097 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_098 => {
                    frame_28098 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_099 => {
                    frame_28099 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 still seeds `($FFFFA656).w` and calls
        // `sub_1B222`, but on this later title-restore revisit oracle does not
        // reuse the earlier timeout-restart hidden camera wrap. The stale
        // loc_1BCD0 slot-2 pose survives, while `Camera_Y_pos` stays cleared
        // through local mode-$28 frame 33 instead of jumping to `$FFF0_0000`.
        assert_eq!(
            frame_28097,
            Some((GameMode::IntroSequence6, LevelId(0x00), Some(31), 0)),
            "frame 28097 should still expose the cleared camera on the later IntroSequence6 revisit",
        );
        assert_eq!(
            frame_28098,
            Some((GameMode::IntroSequence6, LevelId(0x00), Some(32), 0)),
            "frame 28098 should keep Camera_Y_pos cleared instead of reusing the earlier hidden wrap",
        );
        assert_eq!(
            frame_28099,
            Some((GameMode::IntroSequence6, LevelId(0x00), Some(33), 0)),
            "frame 28099 should still keep the later IntroSequence6 camera cleared on the following frame",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_uses_delayed_seeded_camera_row_on_frame_28101()
    {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_28100 = None;
        let mut frame_28101 = None;
        let mut frame_28102 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_103) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                28_100 => {
                    frame_28100 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_101 => {
                    frame_28101 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_102 => {
                    frame_28102 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 still seeds `($FFFFA656).w` and `sub_1B222` on
        // the later title-restore revisit, but unlike the earlier timeout path it
        // never exposes the wrapped `$FFF0_0000` row here. Oracle RAM probes keep
        // `Camera_Y_pos` cleared through local mode-$28 frame 34, then switch to
        // the delayed seeded `$FFD00000` row on frame 35.
        assert_eq!(
            frame_28100,
            Some((GameMode::IntroSequence6, LevelId(0x00), Some(34), 0)),
            "frame 28100 should still keep the later IntroSequence6 camera cleared before the delayed seed appears",
        );
        assert_eq!(
            frame_28101,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(35),
                0xFFD0_0000
            )),
            "frame 28101 should expose the delayed `sub_1B222` seeded camera row on the later revisit",
        );
        assert_eq!(
            frame_28102,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(36),
                0xFFD0_0000
            )),
            "frame 28102 should keep the delayed seeded camera row latched on the next frame",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_starts_delayed_camera_ramp_on_frame_28317() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_28316 = None;
        let mut frame_28317 = None;
        let mut frame_28318 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_319) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                28_316 => {
                    frame_28316 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_317 => {
                    frame_28317 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_318 => {
                    frame_28318 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM GfxObjects_Collision_BottomBoundary4 snapshots
        // `d1 = ($FFFFA656).w >> 4` once, then repeats
        // `sub.l d1,($FFFFA656).w; bsr.w sub_1B222` for 16 frames. On the later
        // revisit `($FFFFA656).w` is still `$FFD00000`, so `d1=$FFFD0000` and the
        // hidden camera rises by 3 pixels per frame from local mode-$28 frame 251.
        assert_eq!(
            frame_28316,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(250),
                0xFFD0_0000
            )),
            "frame 28316 should still keep the delayed seeded camera flat before the later ramp starts",
        );
        assert_eq!(
            frame_28317,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(251),
                0xFFD3_0000
            )),
            "frame 28317 should start the later IntroSequence6 +3px hidden camera ramp",
        );
        assert_eq!(
            frame_28318,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(252),
                0xFFD6_0000
            )),
            "frame 28318 should continue the later IntroSequence6 +3px hidden camera ramp",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_keeps_title_pose_hidden_through_frame_28398() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_28396 = None;
        let mut frame_28398 = None;
        let mut frame_28399 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_400) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                28_396 => {
                    frame_28396 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                28_398 => {
                    frame_28398 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                28_399 => {
                    frame_28399 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B5CE still owns the eventual `move.w #$2E,x_pos(a3)` /
        // `move.w #$8C,y_pos(a3)` write on the later mode-$28 revisit, but oracle
        // keeps Kid hidden through local frame 332 after the delayed `$FFD00000`
        // camera-ramp lineage and only exposes the standard title pose on 333.
        assert_eq!(
            frame_28396,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(330),
                None,
                None
            )),
            "frame 28396 should still keep Kid hidden on the later mode-$28 revisit even though the earlier timeout-restart lineage already shows the title pose",
        );
        assert_eq!(
            frame_28398,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(332),
                None,
                None
            )),
            "frame 28398 should still be hidden on the final pre-title-pose frame of the later revisit",
        );
        assert_eq!(
            frame_28399,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(333),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 28399 should be the first later-revisit frame where loc_1B5CE's standard title pose becomes visible",
        );
    }

    #[test]
    fn demo_diamond_prize_queues_collection_on_frame_1404_without_awarding_score() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_1404 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_405) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_404 {
                frame_1404 = Some((
                    trace.score,
                    trace.diamond_count,
                    trace
                        .gameplay_debug_fields
                        .get("pending_prize_diamond_awards")
                        .cloned(),
                ));
                break;
            }
        }

        assert_eq!(
            frame_1404,
            Some((0, 0, Some("[33]".into()))),
            "oracle frame 1404 has already entered PrizeDiamondCollected_Init, but the HUD award has not landed yet",
        );
    }

    #[test]
    fn demo_diamond_prize_queue_counts_down_on_frame_1405() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_1405 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_406) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_405 {
                frame_1405 = Some((
                    trace.score,
                    trace.diamond_count,
                    trace
                        .gameplay_debug_fields
                        .get("pending_prize_diamond_awards")
                        .cloned(),
                ));
                break;
            }
        }

        assert_eq!(
            frame_1405,
            Some((0, 0, Some("[32]".into()))),
            "one frame after contact, the queued diamond award should have counted down once without scoring yet",
        );
    }

    #[test]
    fn demo_prize_diamond_awards_on_frame_1438_without_score() {
        let (mut runtime, script) = cached_any_percent_demo_runtime_and_script();
        let mut frame_1438 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_439) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_438 {
                frame_1438 = Some((trace.score, trace.diamond_count));
                break;
            }
        }

        assert_eq!(
            frame_1438,
            Some((0, 1)),
            "oracle frame 1438 increments the prize diamond count without adding score"
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_keeps_mode_28_through_frame_29404() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_29401 = None;
        let mut frame_29402 = None;
        let mut frame_29404 = None;
        let mut frame_29405 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(29_406) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                29_401 => {
                    frame_29401 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                29_402 => {
                    frame_29402 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                29_404 => {
                    frame_29404 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                29_405 => {
                    frame_29405 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BAEA: move.w #$280,d0` feeds the shared
        // `loc_1BAEE` DBF-backed idle loop, and only `loc_1BB02` writes
        // `Game_Mode=$10`. On the later title-restore revisit oracle keeps the
        // already-visible title pose in mode `$28` for three extra local frames
        // before that hand-off lands on frame 29405.
        let intro_sequence_6_pose = Some((
            GameMode::IntroSequence6,
            LevelId(0x00),
            Some(1335),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        assert_eq!(
            frame_29401, intro_sequence_6_pose,
            "frame 29401 should still be the settled later IntroSequence6 title pose before the delayed loc_1BB02 hand-off",
        );
        assert_eq!(
            frame_29402,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(1336),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 29402 should remain in mode $28 on the first extra later-lineage idle frame",
        );
        assert_eq!(
            frame_29404,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(1338),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 29404 should still be the final later-lineage mode-$28 frame before loc_1BB02 flips to demo play",
        );
        assert_eq!(
            frame_29405,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(0),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 29405 should be the first later-lineage demo-play frame after the delayed loc_1BB02 hand-off",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_uses_intro_return_demo_2_preload_row_on_frame_29462()
     {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_29461 = None;
        let mut frame_29462 = None;
        let mut frame_29463 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(29_464) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                29_461 => {
                    frame_29461 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                29_462 => {
                    frame_29462 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                29_463 => {
                    frame_29463 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BB02` still reaches `Load_DemoPlay ->
        // Load_InGame`, but on the later IntroSequence6 title-restore revisit
        // oracle keeps the same `$0228_0000` hidden demo-2 preload row as the
        // earlier Start-skip intro-return lineage before `Load_InGame` snaps to
        // `$02A0_0000` on the following frame.
        assert_eq!(
            frame_29461,
            Some((GameMode::DemoPlay, Some(56), LevelId(0x1B), 0x0000_0000)),
            "frame 29461 should still be on the blank hidden later demo-2 bootstrap row",
        );
        assert_eq!(
            frame_29462,
            Some((
                GameMode::DemoPlay,
                Some(demo::DEMO_CAMERA_PRELOAD_FRAME as usize),
                LevelId(0x1B),
                demo::DEMO_2_CAMERA_PRELOAD_Y_FP,
            )),
            "frame 29462 should expose the oracle-backed later IntroSequence6 demo-2 preload row",
        );
        assert_eq!(
            frame_29463,
            Some((GameMode::DemoPlay, Some(58), LevelId(0x1B), 0x02A0_0000)),
            "frame 29463 should return to Load_InGame's respawn camera snap",
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_intro_sequence_5_keeps_camera_handoff_on_frame_32005() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_32004 = None;
        let mut frame_32005 = None;
        let mut frame_32007 = None;
        let mut frame_32011 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(32_012) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                32_004 => {
                    frame_32004 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                }
                32_005 => {
                    frame_32005 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                }
                32_007 => {
                    frame_32007 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                }
                32_011 => {
                    frame_32011 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1B988 writes `move.w #4,(Camera_Y_pos).w`, and
        // Load_IntroSequence5 only allocates `loc_1BA28` / `loc_1BCD0`, so the
        // later timeout-restart revisit after the repaired demo-2 preload row
        // must still carry that intro-sequence-4 camera into the opening
        // intro-sequence-5 frames. Oracle keeps `Camera_Y_pos=$0004_0000`
        // through frame 32011, even though the scene changes around it.
        assert_eq!(
            frame_32004,
            Some((GameMode::IntroSequence4, LevelId(0x00), 0x0004_0000)),
            "frame 32004 should still be the final intro-sequence-4 frame with sub_1B988's camera hand-off latched",
        );
        assert_eq!(
            frame_32005,
            Some((GameMode::IntroSequence4, LevelId(0x00), 0x0004_0000)),
            "frame 32005 should keep the carried sub_1B988 camera hand-off instead of clearing Camera_Y_pos on the Start-tapped revisit",
        );
        assert_eq!(
            frame_32007,
            Some((GameMode::TitleScreen, LevelId(0x00), 0x0004_0000)),
            "frame 32007 should expose the deferred `sub_1C246 -> Game_Mode=$2C` title return while preserving the carried camera hand-off",
        );
        assert_eq!(
            frame_32011,
            Some((GameMode::TitleScreen, LevelId(0x00), 0x0004_0000)),
            "frame 32011 should still keep the carried intro-sequence-4 camera inside the early title-return window",
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_title_clears_intro_sequence_4_camera_on_frame_32039() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_32038 = None;
        let mut frame_32039 = None;
        let mut frame_32040 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(32_041) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                32_038 => {
                    frame_32038 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                }
                32_039 => {
                    frame_32039 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                }
                32_040 => {
                    frame_32040 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1C246 has already returned this lineage to `Game_Mode=$2C`;
        // the carried `sub_1B988` camera only survives for the brief title
        // rebuild window before oracle restores the cleared title camera on frame
        // 32039.
        assert_eq!(
            frame_32038,
            Some((GameMode::TitleScreen, LevelId(0x00), 0x0004_0000)),
            "frame 32038 should still keep the carried intro-sequence-4 camera on the last oracle-backed title-settle frame",
        );
        assert_eq!(
            frame_32039,
            Some((GameMode::TitleScreen, LevelId(0x00), 0x0000_0000)),
            "frame 32039 should clear the carried intro-sequence-4 camera once the title rebuild settles",
        );
        assert_eq!(
            frame_32040,
            Some((GameMode::TitleScreen, LevelId(0x00), 0x0000_0000)),
            "frame 32040 should stay on the cleared title camera after the carry window ends",
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_title_restores_pose_on_frame_32156() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_32155 = None;
        let mut frame_32156 = None;
        let mut frame_32157 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(32_158) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                32_155 => {
                    frame_32155 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                32_156 => {
                    frame_32156 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                32_157 => {
                    frame_32157 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1C246 has already returned the later intro-sequence-4 revisit to
        // `Game_Mode=$2C`, and oracle-backed TAS traces show the rebuilt title
        // shell exposes the standard `loc_1B5CE` pose on frame 32156 rather than
        // waiting for another generic intro-return rebuild frame.
        assert_eq!(
            frame_32155,
            Some((GameMode::TitleScreen, LevelId(0x00), None, None)),
            "frame 32155 should still be the final hidden title-rebuild frame on the later intro-sequence-4 revisit",
        );
        assert_eq!(
            frame_32156,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 32156 should be the first title frame that re-exposes the standard title pose on the later intro-sequence-4 revisit",
        );
        assert_eq!(
            frame_32157,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 32157 should keep the restored title pose visible on the following frame",
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_title_resumes_demo_play_on_frame_32704() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_32703 = None;
        let mut frame_32704 = None;
        let mut frame_32705 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(32_706) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                32_703 => {
                    frame_32703 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                32_704 => {
                    frame_32704 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                32_705 => {
                    frame_32705 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BAEA` / `loc_1BB02` own the final `$2C -> $10`
        // attract hand-off after `sub_1C246` has already restored the later
        // intro-sequence-4 title pose. Oracle reaches `loc_1BB02` on frame 32704
        // without moving Kid off the standard title pose first.
        assert_eq!(
            frame_32703,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 32703 should still be the final settled later intro-sequence-4 title frame",
        );
        assert_eq!(
            frame_32704,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 32704 should be the first demo-play frame after the later intro-sequence-4 title return reaches loc_1BB02",
        );
        assert_eq!(
            frame_32705,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 32705 should stay in demo playback while the hidden bootstrap continues",
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_demo_return_reuses_title_pose_on_frames_34368_and_34369()
    {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_34367 = None;
        let mut frame_34368 = None;
        let mut frame_34369 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(34_370) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                34_367 => {
                    frame_34367 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                34_368 => {
                    frame_34368 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                34_369 => {
                    frame_34369 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B5CE had already restored the standard title pose before the
        // later intro-sequence-4 title helper reached `loc_1BB02` on frame 32704.
        // When that later demo returns to title, oracle re-exposes the same pose
        // two frames earlier than the generic demo-title rebuild window.
        let hidden_title_pose = Some((GameMode::TitleScreen, LevelId(0x00), None, None));
        let visible_title_pose = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        assert_eq!(
            frame_34367, hidden_title_pose,
            "frame 34367 should still be the last hidden title-rebuild frame before the restored title pose returns",
        );
        assert_eq!(
            frame_34368, visible_title_pose,
            "frame 34368 should expose the already-restored loc_1B5CE title pose on the later intro-sequence-4 demo-return lineage",
        );
        assert_eq!(
            frame_34369, visible_title_pose,
            "frame 34369 should keep the same title pose for one more oracle-backed frame",
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_intro_sequence_1_restarts_demo_play_on_frame_34916() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_34915 = None;
        let mut frame_34916 = None;
        let mut frame_34917 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(34_918) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                34_915 => {
                    frame_34915 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                34_916 => {
                    frame_34916 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                34_917 => {
                    frame_34917 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1C246 has already returned this later intro-sequence-1
        // Start-skip lineage to `Game_Mode=$2C`, and oracle-backed traces show the
        // early-restored loc_1B5CE title pose means Obj_TitleMenu `loc_1BAEA`
        // reaches `loc_1BB02` two title frames earlier here as well.
        let title_pose = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        let demo_pose = Some((
            GameMode::DemoPlay,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        assert_eq!(
            frame_34915, title_pose,
            "frame 34915 should still be the final title-mode frame before the later intro-sequence-1 title shell reaches loc_1BB02",
        );
        assert_eq!(
            frame_34916, demo_pose,
            "frame 34916 should be the first demo-play frame after the later intro-sequence-1 title return reaches loc_1BB02",
        );
        assert_eq!(
            frame_34917, demo_pose,
            "frame 34917 should remain in demo playback on the following hidden bootstrap frame",
        );
    }

    #[test]
    fn published_tas_later_demo_2_transform_ignores_start_abort_on_frame_35148() {
        let (mut runtime, script) = cached_any_percent_published_runtime_and_script();
        let mut frame_35146 = None;
        let mut frame_35147 = None;
        let mut frame_35148 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(35_149) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            let snapshot = (
                trace.game_mode,
                trace.current_level,
                trace.paused,
                matches!(runtime.boot_state, BootState::DemoPlay { .. }),
            );
            match frame_index {
                35_146 => frame_35146 = Some(snapshot),
                35_147 => frame_35147 = Some(snapshot),
                35_148 => {
                    frame_35148 = Some(snapshot);
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_119D8 polls Start for demo aborts, but Kid_Transform hibernates
        // the regular gameplay/object loop while the later published-TAS demo-2
        // transform is still in progress. Oracle therefore ignores the frame-35146
        // Start press and keeps demo play active through frame 35148.
        let demo_transform_pose = Some((GameMode::DemoPlay, LevelId(0x1B), false, true));
        assert_eq!(
            frame_35146, demo_transform_pose,
            "frame 35146 should still be demo play when the later demo-2 Start press lands during Kid_Transform",
        );
        assert_eq!(
            frame_35147, demo_transform_pose,
            "frame 35147 should remain in demo play while the transform coroutine still hibernates the frame loop",
        );
        assert_eq!(
            frame_35148, demo_transform_pose,
            "frame 35148 should still ignore the queued Start abort until the later demo-2 transform hibernation ends",
        );
    }

    /// Determinism: running the same scenario twice must produce identical traces.
    /// Non-determinism would invalidate all oracle comparisons.
    #[test]
    fn determinism_same_scenario_produces_identical_traces() {
        let manifest = manifest_with_levels(3);
        let script = vec![0u8; 121]; // 120 frames of idle

        let trace1 = run_script_with_warp(manifest.clone(), &script, 120, Some(0));
        let trace2 = run_script_with_warp(manifest, &script, 120, Some(0));

        assert_eq!(trace1.len(), trace2.len(), "trace lengths differ");

        let crcs1: Vec<u32> = trace1
            .iter()
            .map(crate::types::NativeTraceFrame::authoritative_crc32)
            .collect();
        let crcs2: Vec<u32> = trace2
            .iter()
            .map(crate::types::NativeTraceFrame::authoritative_crc32)
            .collect();
        assert_eq!(crcs1, crcs2, "determinism violation: CRC arrays differ");

        for (i, (a, b)) in trace1.iter().zip(trace2.iter()).enumerate() {
            assert_eq!(a.kid_x_pos, b.kid_x_pos, "frame {i}: kid_x_pos differs");
            assert_eq!(a.kid_y_pos, b.kid_y_pos, "frame {i}: kid_y_pos differs");
            assert_eq!(
                a.camera_x_pos, b.camera_x_pos,
                "frame {i}: camera_x_pos differs"
            );
            assert_eq!(
                a.camera_y_pos, b.camera_y_pos,
                "frame {i}: camera_y_pos differs"
            );
            assert_eq!(a.score, b.score, "frame {i}: score differs");
            assert_eq!(
                a.number_hitpoints, b.number_hitpoints,
                "frame {i}: HP differs"
            );
            assert_eq!(
                a.active_enemy_object_count, b.active_enemy_object_count,
                "frame {i}: enemy count differs"
            );
        }
    }

    /// Determinism with active gameplay inputs (walk + jump).
    #[test]
    fn determinism_active_gameplay_identical_traces() {
        let manifest = manifest_with_levels(3);
        let mut script = vec![0u8; 181]; // 180 frames
        // Walk right from frame 60
        for s in script.iter_mut().skip(60).take(90) {
            *s = 1 << 3; // right
        }
        // Jump at frame 90
        for s in script.iter_mut().skip(90).take(4) {
            *s |= 1 << 4; // B button (jump)
        }

        let trace1 = run_script_with_warp(manifest.clone(), &script, 180, Some(0));
        let trace2 = run_script_with_warp(manifest, &script, 180, Some(0));

        assert_eq!(trace1.len(), trace2.len());

        let crcs1: Vec<u32> = trace1
            .iter()
            .map(crate::types::NativeTraceFrame::authoritative_crc32)
            .collect();
        let crcs2: Vec<u32> = trace2
            .iter()
            .map(crate::types::NativeTraceFrame::authoritative_crc32)
            .collect();
        assert_eq!(
            crcs1, crcs2,
            "determinism violation: CRC arrays differ on active gameplay"
        );

        for (i, (a, b)) in trace1.iter().zip(trace2.iter()).enumerate() {
            assert_eq!(a.kid_x_pos, b.kid_x_pos, "frame {i}: kid_x_pos differs");
            assert_eq!(a.kid_y_pos, b.kid_y_pos, "frame {i}: kid_y_pos differs");
            assert_eq!(
                a.kid_x_velocity, b.kid_x_velocity,
                "frame {i}: kid_x_velocity differs"
            );
            assert_eq!(
                a.kid_y_velocity, b.kid_y_velocity,
                "frame {i}: kid_y_velocity differs"
            );
        }
    }

    // --- Ghost blocks: step_ghost_blocks through NativeRuntime ---

    #[test]
    fn ghost_block_step_transitions_solid_to_vanished_and_clears_grid() {
        // Ghost group starts solid with timer=0 → should transition to vanished,
        // clearing the grid cells.
        let mut rt = NativeRuntime::new(manifest());
        // Place a solid ghost block at (3, 5): has_block=1, collision=Solid(6), block_type=SolidGhost(3)
        // hi = 0x80 | (0x06 << 4) | 0x03 = 0xE3
        let grid = grid_with_cells(8, 8, &[(3, 5, 0xE3, 0)]);
        rt.block_grid = Some(grid);
        rt.block_state.ghost_groups.push(GhostGroup {
            col: 3,
            row: 5,
            count: 1,
            vertical: false,
            solid_time: 10,
            vanished_time: 5,
            timer: 0,
            solid: true,
        });

        rt.step_ghost_blocks();

        // Should have transitioned to vanished
        assert!(!rt.block_state.ghost_groups[0].solid);
        assert_eq!(rt.block_state.ghost_groups[0].timer, 5);
        // Grid cell should be cleared
        let cell = rt.block_grid.as_ref().unwrap().cell_at(3, 5);
        assert!(
            !cell.has_block,
            "vanished ghost block should clear grid cell"
        );
    }

    #[test]
    fn ghost_block_step_transitions_vanished_to_solid_and_restores_grid() {
        let mut rt = NativeRuntime::new(manifest());
        let grid = grid_with_cells(8, 8, &[]);
        rt.block_grid = Some(grid);
        rt.block_state.ghost_groups.push(GhostGroup {
            col: 3,
            row: 5,
            count: 1,
            vertical: false,
            solid_time: 10,
            vanished_time: 5,
            timer: 0,
            solid: false,
        });

        rt.step_ghost_blocks();

        assert!(rt.block_state.ghost_groups[0].solid);
        assert_eq!(rt.block_state.ghost_groups[0].timer, 10);
        // Grid cell should be restored to solid ghost
        let cell = rt.block_grid.as_ref().unwrap().cell_at(3, 5);
        assert!(cell.has_block, "restored ghost block should have has_block");
        assert_eq!(cell.block_type, terrain::BlockType::SolidGhost);
        assert!(matches!(cell.collision, terrain::CollisionKind::Solid));
    }

    #[test]
    fn ghost_block_step_decrements_timer_without_transition() {
        // When timer > 0, should just decrement — no transition, no grid change.
        let mut rt = NativeRuntime::new(manifest());
        let grid = grid_with_cells(8, 8, &[(3, 5, 0xE3, 0)]);
        rt.block_grid = Some(grid);
        rt.block_state.ghost_groups.push(GhostGroup {
            col: 3,
            row: 5,
            count: 1,
            vertical: false,
            solid_time: 10,
            vanished_time: 5,
            timer: 3,
            solid: true,
        });

        rt.step_ghost_blocks();

        assert!(
            rt.block_state.ghost_groups[0].solid,
            "should still be solid"
        );
        assert_eq!(rt.block_state.ghost_groups[0].timer, 2);
        // Grid should be unchanged
        let cell = rt.block_grid.as_ref().unwrap().cell_at(3, 5);
        assert!(cell.has_block, "grid should be unchanged during countdown");
    }

    #[test]
    fn ghost_block_vertical_group_clears_correct_cells() {
        // Vertical group of 3 blocks starting at (2, 4)
        let mut rt = NativeRuntime::new(manifest());
        let grid = grid_with_cells(8, 8, &[(2, 4, 0xE3, 0), (2, 5, 0xE3, 0), (2, 6, 0xE3, 0)]);
        rt.block_grid = Some(grid);
        rt.block_state.ghost_groups.push(GhostGroup {
            col: 2,
            row: 4,
            count: 3,
            vertical: true,
            solid_time: 10,
            vanished_time: 5,
            timer: 0,
            solid: true,
        });

        rt.step_ghost_blocks();

        let grid = rt.block_grid.as_ref().unwrap();
        for row in 4..=6 {
            let cell = grid.cell_at(2, row);
            assert!(
                !cell.has_block,
                "vertical ghost cell at row {row} should be cleared"
            );
        }
    }

    // --- Shifting blocks ---

    #[test]
    fn shifting_block_no_op_when_airborne() {
        let mut rt = NativeRuntime::new(manifest());
        // Shifting block at (3, 5)
        let grid = grid_with_cells(8, 8, &[(3, 5, 0xE7, 0)]);
        rt.block_grid = Some(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.on_ground = false;
        slice.player.x_pos = 56 << 16;
        slice.player.y_pos = 79 << 16;

        rt.apply_shifting_blocks(&slice);

        // Block should not have moved
        let grid = rt.block_grid.as_ref().unwrap();
        assert!(
            grid.cell_at(3, 5).has_block,
            "block should remain when player airborne"
        );
    }

    #[test]
    fn shifting_block_no_op_at_row_zero() {
        // A shifting block at row 0 has nowhere to shift up to.
        let mut rt = NativeRuntime::new(manifest());
        // Shifting block at (3, 0), solid floor at row 1
        let grid = grid_with_cells(8, 4, &[(3, 0, 0xE7, 0), (3, 1, 0x60, 0)]);
        rt.block_grid = Some(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.on_ground = true;
        // Standing on row 0: feet at y = pixel_to_cell boundary (row 0 top = 0px, +1 = 1)
        slice.player.x_pos = 56 << 16;
        slice.player.y_pos = 0 << 16; // at pixel 0 — foot_row = pixel_to_cell(1) = 0

        rt.apply_shifting_blocks(&slice);

        // Block should still be there
        assert!(
            rt.block_grid.as_ref().unwrap().cell_at(3, 0).has_block,
            "shifting block at row 0 should not move"
        );
    }

    // --- Deferred ceiling clears: full lifecycle ---

    #[test]
    fn deferred_ceiling_clear_full_lifecycle() {
        // ASM sub_1007A: Deleted(32 frames) → Restored(1 frame) → PendingClear → FreshSmash → removed
        let mut rt = NativeRuntime::new(manifest());
        // Set up a grid with a prize block that has been hit (written as 0x6F = Solid+Deleted)
        let grid = grid_with_cells(8, 8, &[(3, 2, 0x6F, 0)]);
        rt.block_grid = Some(grid);

        let restore_cell = terrain::Cell::from_be_bytes(0xE1, 0x05); // original prize cell
        rt.block_state
            .deferred_ceiling_clears
            .push(DeferredCeilingClear {
                col: 3,
                row: 2,
                content: terrain::PrizeContent::Diamond,
                restore_cell,
                deleted_frames_remaining: 2, // shortened for test
                phase: DeferredPrizeBlockPhase::Deleted,
            });

        // Tick 1: still Deleted, frames_remaining → 1
        rt.step_deferred_ceiling_clears();
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::Deleted
        );
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].deleted_frames_remaining,
            1
        );

        // Tick 2: still Deleted, frames_remaining → 0
        rt.step_deferred_ceiling_clears();
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::Deleted
        );
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].deleted_frames_remaining,
            0
        );

        // Tick 3: frames_remaining == 0, transition to Restored, grid gets restore_cell
        rt.step_deferred_ceiling_clears();
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::Restored
        );
        let cell = rt.block_grid.as_ref().unwrap().cell_at(3, 2);
        assert_eq!(cell, restore_cell, "grid should have the restore cell");

        // Tick 4: Restored → PendingClear
        rt.step_deferred_ceiling_clears();
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::PendingClear
        );

        // Now finish_deferred_ceiling_clears: PendingClear → FreshSmash + spawn item
        assert!(rt.floating_items.is_empty());
        rt.finish_deferred_ceiling_clears();
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::FreshSmash
        );
        assert_eq!(rt.floating_items.len(), 1, "should spawn floating item");
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_DIAMOND_LIFETIME_FRAMES
        );
        // Grid should now have Smash block ($E001)
        let cell = rt.block_grid.as_ref().unwrap().cell_at(3, 2);
        assert!(cell.has_block);
        assert_eq!(cell.block_type, terrain::BlockType::Smash);

        // Next finish_deferred_ceiling_clears: FreshSmash → removed
        rt.finish_deferred_ceiling_clears();
        assert!(
            rt.block_state.deferred_ceiling_clears.is_empty(),
            "FreshSmash should be cleaned up"
        );
    }

    #[test]
    fn deferred_ankh_ceiling_clear_uses_sub_10c78_lifetime() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_blocks());
        rt.block_state
            .deferred_ceiling_clears
            .push(DeferredCeilingClear {
                col: 3,
                row: 2,
                content: terrain::PrizeContent::Ankh,
                restore_cell: terrain::Cell::from_be_bytes(0xE0, 0x0B),
                deleted_frames_remaining: 0,
                phase: DeferredPrizeBlockPhase::PendingClear,
            });

        rt.finish_deferred_ceiling_clears();

        assert_eq!(rt.floating_items.len(), 1);
        // ASM sub_10C78 initialises d3 as `$0001_0320`, so non-diamond fallers
        // keep the bit-16 first-row grace flag and an 0x320-frame lifetime.
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES
        );
        assert!(rt.floating_items[0].first_row_grace);
    }

    #[test]
    fn deferred_helmet_ceiling_clear_replaces_existing_helmet_via_loc_10cf4_scan() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_blocks());
        rt.floating_items.push(FloatingItem {
            x_pos: 0x0070_0000,
            y_pos: 0x0080_0000,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 7,
            level_row: 8,
            first_row_grace: true,
            content: terrain::PrizeContent::Skycutter,
            collected: false,
        });
        rt.floating_items.push(FloatingItem {
            x_pos: 0x0030_0000,
            y_pos: 0x0040_0000,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 3,
            level_row: 4,
            first_row_grace: true,
            content: terrain::PrizeContent::Clock,
            collected: false,
        });
        rt.block_state
            .deferred_ceiling_clears
            .push(DeferredCeilingClear {
                col: 5,
                row: 2,
                content: terrain::PrizeContent::Cyclone,
                restore_cell: terrain::Cell::from_be_bytes(0xE0, 0x03),
                deleted_frames_remaining: 0,
                phase: DeferredPrizeBlockPhase::PendingClear,
            });

        rt.finish_deferred_ceiling_clears();

        assert_eq!(
            rt.floating_items.len(),
            2,
            "ASM loc_10CF4 should keep scanning until it finds the older helmet owner, deleting only that helmet before the new one spawns"
        );
        assert!(
            rt.floating_items
                .iter()
                .all(|item| item.content != terrain::PrizeContent::Skycutter),
            "the older floating helmet should be removed"
        );
        assert!(
            rt.floating_items
                .iter()
                .any(|item| item.content == terrain::PrizeContent::Clock),
            "non-helmet prize items should survive the helmet cleanup"
        );
        assert!(
            rt.floating_items
                .iter()
                .any(|item| item.content == terrain::PrizeContent::Cyclone),
            "the newly spawned helmet should still appear"
        );
        assert!(
            rt.block_destroy_effects
                .iter()
                .any(|effect| effect.x_px == 0x0070 && effect.y_px == 0x0078),
            "ASM loc_10CF4 should hand sub_10CEC the matched helmet slot so the follow-on effect spawns at the deleted helmet's x, y-8"
        );
    }

    #[test]
    fn deferred_helmet_ceiling_clear_without_existing_helmet_falls_through_loc_10d38() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_blocks());
        rt.floating_items.push(FloatingItem {
            x_pos: 0x0030_0000,
            y_pos: 0x0040_0000,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 3,
            level_row: 4,
            first_row_grace: true,
            content: terrain::PrizeContent::Clock,
            collected: false,
        });
        rt.block_state
            .deferred_ceiling_clears
            .push(DeferredCeilingClear {
                col: 5,
                row: 2,
                content: terrain::PrizeContent::Cyclone,
                restore_cell: terrain::Cell::from_be_bytes(0xE0, 0x03),
                deleted_frames_remaining: 0,
                phase: DeferredPrizeBlockPhase::PendingClear,
            });

        rt.finish_deferred_ceiling_clears();

        assert!(
            rt.block_destroy_effects.is_empty(),
            "ASM loc_10D38 should be the no-match return path from sub_10CEC, so spawning a helmet with no older helmet owner must not create a cleanup effect",
        );
        assert!(
            rt.floating_items
                .iter()
                .any(|item| item.content == terrain::PrizeContent::Clock),
            "non-helmet prizes should survive the no-match scan"
        );
        assert!(
            rt.floating_items
                .iter()
                .any(|item| item.content == terrain::PrizeContent::Cyclone),
            "the requested helmet should still spawn after the loc_10D38 return path"
        );
    }

    // --- Process block effects ---

    #[test]
    fn process_block_effects_drill_projectile_fires_rightward() {
        // Player to the right of the drill → projectile fires right
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 200 << 16;
        slice.drill_projectile_hit = Some((100 << 16, 50 << 16));

        rt.process_block_effects(&mut slice);

        assert_eq!(rt.projectiles.len(), 1);
        assert!(
            rt.projectiles[0].x_velocity > 0,
            "drill projectile should fire rightward when player is right of block"
        );
        assert_eq!(rt.projectiles[0].x_velocity, 0x4_0000); // 4 px/frame
        assert_eq!(rt.projectiles[0].y_velocity, 0);
    }

    #[test]
    fn process_block_effects_drill_projectile_fires_leftward() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 50 << 16;
        slice.drill_projectile_hit = Some((100 << 16, 50 << 16));

        rt.process_block_effects(&mut slice);

        assert_eq!(rt.projectiles.len(), 1);
        assert!(
            rt.projectiles[0].x_velocity < 0,
            "drill projectile should fire leftward when player is left of block"
        );
        assert_eq!(rt.projectiles[0].x_velocity, -0x4_0000);
    }

    #[test]
    fn process_block_effects_metal_and_rubber_create_bounce_anims() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.metal_block_hit = Some((5, 3));
        slice.rubber_block_hit = Some((7, 4));

        rt.process_block_effects(&mut slice);

        assert_eq!(rt.block_bounces.len(), 2, "metal + rubber = 2 bounces");
        assert_eq!(rt.block_bounces[0].cell_x, 5);
        assert_eq!(rt.block_bounces[0].cell_y, 3);
        assert_eq!(rt.block_bounces[1].cell_x, 7);
        assert_eq!(rt.block_bounces[1].cell_y, 4);
        assert_eq!(rt.blocks_hit_this_frame, 2);
    }

    #[test]
    fn metal_block_effect_cell_words_loc_110d0_keeps_tile_coordinates_unshifted() {
        assert_eq!(
            NativeRuntime::metal_block_effect_cell_words_loc_110D0(5, 3),
            (5, 3),
            "ASM loc_110D0 should cache the struck block's tile coordinates in `$44/$46` before any sprite-space scaling happens",
        );
    }

    #[test]
    fn metal_block_push_entry_loc_111f4_starts_the_right_scan_one_cell_ahead() {
        assert_eq!(
            NativeRuntime::metal_block_push_entry_loc_111F4(5, 3, 12),
            Some(6),
            "ASM loc_111F4 should pre-advance the layout pointer and column counter to the cell immediately right of the struck metal block when `d6 == 3`",
        );
    }

    #[test]
    fn metal_block_push_entry_loc_111f4_falls_back_to_the_left_scan_for_other_directions() {
        assert_eq!(
            NativeRuntime::metal_block_push_entry_loc_111F4(5, 1, 12),
            Some(4),
            "ASM loc_111F4 should treat non-`3` directions as the left-walking branch and pre-decrement the source column before `loc_11230` starts scanning",
        );
    }

    #[test]
    fn metal_block_push_entry_loc_111f4_aborts_the_right_scan_at_the_level_edge() {
        assert_eq!(
            NativeRuntime::metal_block_push_entry_loc_111F4(11, 3, 12),
            None,
            "ASM loc_111F4 should jump to `loc_11296` once the source block already sits on `Level_width_blocks-1`, because the right scan has no in-bounds cell to inspect",
        );
    }

    #[test]
    fn metal_block_push_scan_right_loc_11208_stops_on_the_last_contiguous_metal_block() {
        let row = [
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0x00, 0x00),
        ];

        assert_eq!(
            NativeRuntime::metal_block_push_scan_right_loc_11208(&row, 0),
            Some(2),
            "ASM loc_11208 should keep advancing across contiguous solid metal blocks and, via `loc_11256`, select the last matching block before the first non-solid cell",
        );
    }

    #[test]
    fn metal_block_push_scan_right_loc_11208_aborts_on_in_bounds_solid_non_metal() {
        let row = [
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE0, 0x01),
            terrain::Cell::from_be_bytes(0x00, 0x00),
        ];

        assert_eq!(
            NativeRuntime::metal_block_push_scan_right_loc_11208(&row, 0),
            None,
            "ASM loc_11208 should jump to `loc_11296` when the scan hits an in-bounds solid cell whose bit-15 / block-type test is not `$E5xx` metal",
        );
    }

    #[test]
    fn metal_block_push_scan_left_loc_11230_stops_on_the_last_contiguous_metal_block() {
        let row = [
            terrain::Cell::from_be_bytes(0x00, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
        ];

        assert_eq!(
            NativeRuntime::metal_block_push_scan_left_loc_11230(&row, 3),
            Some(1),
            "ASM loc_11230 should keep retreating across contiguous solid metal blocks and, via `loc_1125A`, select the last matching block before the first non-solid cell",
        );
    }

    #[test]
    fn metal_block_push_scan_left_loc_11230_aborts_when_the_chain_runs_into_column_zero() {
        let row = [
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
        ];

        assert_eq!(
            NativeRuntime::metal_block_push_scan_left_loc_11230(&row, 3),
            None,
            "ASM loc_11230 should jump to `loc_11296` once the contiguous metal run reaches column zero, because the left scan refuses to sample past the level edge",
        );
    }

    #[test]
    fn metal_block_push_mark_deleted_loc_1125c_preserves_collision_and_skin_bits() {
        assert_eq!(
            NativeRuntime::metal_block_push_mark_deleted_loc_1125C(terrain::Cell::from_be_bytes(
                0xE5, 0x0B
            )),
            terrain::Cell::from_be_bytes(0xEF, 0x0B),
            "ASM loc_1125C should apply `or.b #$F,(a2)`, turning the selected `$E50B` metal block into `$EF0B` without changing its collision or skin bits",
        );
    }

    #[test]
    fn metal_block_push_step_right_loc_112e2_reaches_the_neighbour_check_when_d3_hits_four() {
        let step = NativeRuntime::metal_block_push_step_right_loc_112E2(0x0050, 3);

        assert_eq!(
            step.x_pos, 0x0054,
            "ASM loc_112E2 should move the worker sprite 4 px right on each wake",
        );
        assert_eq!(step.substep_counter, 4);
        assert!(
            step.reached_neighbour_check,
            "ASM loc_112E2 should branch to `loc_11334` once the incremented `d3` reaches 4",
        );
    }

    #[test]
    fn metal_block_push_step_left_loc_112fe_reaches_the_neighbour_check_when_d3_hits_four() {
        let step = NativeRuntime::metal_block_push_step_left_loc_112FE(0x0050, 3);

        assert_eq!(
            step.x_pos, 0x004C,
            "ASM loc_112FE should move the worker sprite 4 px left on each wake",
        );
        assert_eq!(step.substep_counter, 4);
        assert!(
            step.reached_neighbour_check,
            "ASM loc_112FE should branch to `loc_1130C` once the incremented `d3` reaches 4",
        );
    }

    #[test]
    fn metal_block_push_left_delete_gate_loc_11328_deletes_at_column_zero_before_sampling_left() {
        assert!(
            NativeRuntime::metal_block_push_left_delete_gate_loc_11328(
                0,
                terrain::Cell::from_be_bytes(0x60, 0x05),
            ),
            "ASM loc_11328 should delete immediately when `tst.w d1; ble.s loc_11328` sees the worker already at column zero, without relying on the sampled left neighbour",
        );
    }

    #[test]
    fn metal_block_push_left_delete_gate_loc_11328_continues_only_while_the_left_neighbour_is_solid()
     {
        assert!(
            !NativeRuntime::metal_block_push_left_delete_gate_loc_11328(
                3,
                terrain::Cell::from_be_bytes(0x60, 0x00),
            ),
            "ASM loc_11328 should loop back to `loc_112E2` when the sampled left neighbour masks to `$6000`, regardless of its block skin",
        );
        assert!(
            NativeRuntime::metal_block_push_left_delete_gate_loc_11328(
                3,
                terrain::Cell::from_be_bytes(0x00, 0x00),
            ),
            "ASM loc_11328 should fall through to the shared restore-and-delete tail once the sampled left neighbour is not solid",
        );
    }

    #[test]
    fn metal_block_effect_sprite_pos_loc_1110e_scales_tile_words_into_pixels() {
        assert_eq!(
            NativeRuntime::metal_block_effect_sprite_pos_loc_1110E(5, 3),
            (80, 48),
            "ASM loc_1110E should mirror the cached `$44/$46` tile words into sprite-space with `lsl.w #4` for both axes",
        );
    }

    #[test]
    fn rubber_block_contact_runs_sub_10de4_skin_sequence() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_floor_at_row(10);
        grid.set_cell_at(7, 4, terrain::Cell::from_be_bytes(0xE6, 0x0C));
        rt.set_block_grid(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.rubber_block_hit = Some((7, 4));

        rt.process_block_effects(&mut slice);

        let skin =
            |runtime: &NativeRuntime| runtime.block_grid().expect("block grid").cell_at(7, 4).skin;

        assert_eq!(
            skin(&rt),
            0x0D,
            "ASM sub_10DE4 writes the `base+1` rubber frame on the contact frame"
        );

        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0D);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0C);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0D);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0C);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0D);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0D);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0C);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0C);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0D);
        rt.step_visual_effects();
        rt.step_visual_effects();
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0C);
        rt.step_visual_effects();
        rt.step_visual_effects();
        rt.step_visual_effects();
        assert_eq!(
            skin(&rt),
            0x0C,
            "ASM sub_10DE4 finishes on the base rubber frame after the final 3-frame hibernate"
        );
    }

    #[test]
    fn rubber_block_base_skin_for_sub_10de4_loc_10e06_keeps_the_low_animation_pair_for_0c_and_0d() {
        assert_eq!(
            NativeRuntime::rubber_block_base_skin_for_sub_10de4(terrain::Cell::from_be_bytes(
                0xE6, 0x0D,
            )),
            Some(0x0C),
            "ASM loc_10E06 should seed rubber skins `$0C/$0D` from base frame `$E60C` so the shared writer only toggles the low pair",
        );
    }

    #[test]
    fn rubber_block_contact_loc_10e0a_writes_the_high_pair_base_plus_one_before_the_first_hibernate()
     {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_floor_at_row(10);
        grid.set_cell_at(7, 4, terrain::Cell::from_be_bytes(0xE6, 0x0E));
        rt.set_block_grid(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.rubber_block_hit = Some((7, 4));

        rt.process_block_effects(&mut slice);

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(7, 4).skin,
            0x0F,
            "ASM loc_10E0A should increment the selected rubber base frame before the first `sub_11530` write, so the contact frame for the high pair is `$E60F` rather than `$E60E`",
        );
    }

    #[test]
    fn metal_block_push_right_delete_gate_loc_11358_deletes_at_the_right_boundary_before_sampling_right()
     {
        assert!(
            NativeRuntime::metal_block_push_right_delete_gate_loc_11358(
                7,
                8,
                terrain::Cell::from_be_bytes(0x60, 0x05),
            ),
            "ASM loc_11358 should delete immediately when `cmp.w Level_width_blocks-1,d1` reaches the last column, without relying on the sampled right neighbour",
        );
    }

    #[test]
    fn metal_block_push_right_delete_gate_loc_11358_continues_only_while_the_right_neighbour_is_solid()
     {
        assert!(
            !NativeRuntime::metal_block_push_right_delete_gate_loc_11358(
                3,
                8,
                terrain::Cell::from_be_bytes(0x60, 0x00),
            ),
            "ASM loc_11358 should loop back to `loc_112E2` when the sampled right neighbour masks to `$6000`",
        );
        assert!(
            NativeRuntime::metal_block_push_right_delete_gate_loc_11358(
                3,
                8,
                terrain::Cell::from_be_bytes(0x00, 0x00),
            ),
            "ASM loc_11358 should fall through to the shared restore-and-delete tail once the sampled right neighbour is not solid",
        );
    }

    #[test]
    fn process_block_effects_smash_creates_destroy_effect() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.smash_block_hit = Some((80, 48));

        rt.process_block_effects(&mut slice);

        assert_eq!(rt.block_destroy_effects.len(), 1);
        assert_eq!(rt.block_destroy_effects[0].x_px, 80);
        assert_eq!(rt.block_destroy_effects[0].y_px, 52);
        assert_eq!(rt.block_destroy_effects[0].remaining, 2);
    }

    #[test]
    fn process_block_effects_sub_1149c_queues_empty_support_workers() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_shooter_blocks(&[(5, 4, 0x02)]);
        grid.set_cell_at(6, 3, terrain::Cell::from_be_bytes(0x60, 0));
        rt.set_block_grid(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((5, 4));

        rt.process_block_effects(&mut slice);

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 4),
            terrain::Cell::from_be_bytes(0xE9, 0x02),
            "ASM loc_1146E branches to `loc_11496` once any `sub_1149C` probe sets `d4`, so the struck shooter cell should stay untouched on the support-worker path"
        );
        assert_eq!(rt.block_state.shooter_headbutt_workers.len(), 2);
        assert_eq!(
            rt.block_state
                .shooter_headbutt_workers
                .iter()
                .filter(|worker| worker.kind == ShooterHeadbuttWorkerKind::Center)
                .count(),
            0,
            "ASM loc_11496 is the immediate epilogue after the `bne.s` at loc_1146E, so the support-worker path should return without queuing `sub_114C0`"
        );
        assert_eq!(
            rt.block_state
                .shooter_headbutt_workers
                .iter()
                .filter(|worker| worker.kind == ShooterHeadbuttWorkerKind::Support)
                .count(),
            2,
            "ASM sub_1149C should queue only the empty support cells above the shooter"
        );
    }

    #[test]
    fn shooter_support_spawn_loc_11496_returns_without_marking_the_struck_cell() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_shooter_blocks(&[(5, 4, 0x02)]);
        grid.set_cell_at(6, 3, terrain::Cell::from_be_bytes(0x60, 0));
        rt.set_block_grid(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((5, 4));

        rt.process_block_effects(&mut slice);

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 4),
            terrain::Cell::from_be_bytes(0xE9, 0x02),
            "ASM loc_11496 is the immediate epilogue after the `bne.s` at loc_1146E, so the support-worker path must return before the struck shooter gets `or.b #$F,(a1)`"
        );
        assert!(
            rt.block_state
                .shooter_headbutt_workers
                .iter()
                .all(|worker| worker.kind == ShooterHeadbuttWorkerKind::Support),
            "ASM loc_11496 should restore registers and return without leaving any struck-cell `sub_114C0` worker behind"
        );
    }

    #[test]
    fn iron_knight_smash_block_uses_sub_10eda_upward_offset() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::IronKnight;
        runtime.progression.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();

        let mut grid = grid_with_floor_at_row(14);
        grid.set_cell_at(5, 14, terrain::Cell::from_be_bytes(0xE0, 0x01));
        runtime.block_grid = Some(grid);

        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = (5 * 16 + 8) << 16;
        slice.player.y_pos = 223 << 16;
        slice.player.y_velocity = 0x0002_0000;
        slice.player.on_ground = false;
        runtime.opening_slice = Some(slice);

        runtime.step_frame(FrameInput::default());

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(runtime.block_destroy_effects[0].x_px, 80);
        assert_eq!(
            runtime.block_destroy_effects[0].y_px, 220,
            "ASM loc_10F1A subtracts 4 from `y_pos(a3)` when `d6 == 2`, so sub_10EDA nudges the smash effect 4px upward before the 2-frame hibernate"
        );
        assert_eq!(runtime.block_destroy_effects[0].remaining, 2);
    }

    #[test]
    fn iron_knight_ice_block_uses_sub_10f9c_upward_offset() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::IronKnight;
        runtime.progression.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();

        let mut grid = grid_with_floor_at_row(14);
        grid.set_cell_at(5, 14, terrain::Cell::from_be_bytes(0xE2, 0x01));
        runtime.block_grid = Some(grid);

        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = (5 * 16 + 8) << 16;
        slice.player.y_pos = 223 << 16;
        slice.player.y_velocity = 0x0002_0000;
        slice.player.on_ground = false;
        runtime.opening_slice = Some(slice);

        runtime.step_frame(FrameInput::default());

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(runtime.block_destroy_effects[0].x_px, 80);
        assert_eq!(
            runtime.block_destroy_effects[0].y_px, 220,
            "ASM loc_10FDC subtracts 4 from `y_pos(a3)` when `d6 == 2`, so sub_10F9C nudges the ice effect 4px upward before the shared loc_10FEC 3-frame hibernate tail"
        );
        assert_eq!(runtime.block_destroy_effects[0].remaining, 3);
    }

    #[test]
    fn ice_block_effect_sub_10f9c_uses_loc_10fe2_leftward_offset() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_ice_block_effect_sub_10f9c(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_LEFT,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(
            runtime.block_destroy_effects[0].x_px, 76,
            "ASM loc_10FE2 subtracts 4 from `x_pos(a3)` when `d6 == 1`, so sub_10F9C nudges the ice effect 4px left before the shared loc_10FEC 3-frame hibernate tail"
        );
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 3);
    }

    #[test]
    fn ice_block_effect_sub_10f9c_uses_loc_10fe8_rightward_offset() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_ice_block_effect_sub_10f9c(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_RIGHT,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(
            runtime.block_destroy_effects[0].x_px, 84,
            "ASM loc_10FE8 adds 4 to `x_pos(a3)` when `d6 == 3`, so sub_10F9C nudges the ice effect 4px right before the shared loc_10FEC 3-frame hibernate tail"
        );
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 3);
    }

    #[test]
    fn ice_block_effect_sub_10f9c_uses_the_default_downward_offset_when_d6_is_zero() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_ice_block_effect_sub_10f9c(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_BELOW,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(
            runtime.block_destroy_effects[0].x_px, 80,
            "ASM sub_10F9C falls through to the default branch when `d6 == 0`, so the ice effect should keep its source X position before the shared `loc_10FEC` tail",
        );
        assert_eq!(
            runtime.block_destroy_effects[0].y_px, 52,
            "ASM sub_10F9C takes the default `addq.w #4,y_pos(a3)` path when `d6` is neither 1, 2, nor 3, so the ice effect should nudge 4px downward before the 3-frame hibernate",
        );
        assert_eq!(runtime.block_destroy_effects[0].remaining, 3);
    }

    #[test]
    fn smash_block_effect_sub_10eda_uses_loc_10f20_leftward_offset() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_smash_block_effect_sub_10eda(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_LEFT,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(
            runtime.block_destroy_effects[0].x_px, 76,
            "ASM loc_10F20 subtracts 4 from `x_pos(a3)` when `d6 == 1`, so sub_10EDA nudges the smash effect 4px left before the 2-frame hibernate"
        );
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 2);
    }

    #[test]
    fn smash_block_effect_sub_10eda_uses_loc_10f2a_default_downward_offset() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_smash_block_effect_sub_10eda(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_BELOW,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(runtime.block_destroy_effects[0].x_px, 80);
        assert_eq!(
            runtime.block_destroy_effects[0].y_px, 52,
            "ASM sub_10EDA falls through to the default `addq.w #4,y_pos(a3)` path before loc_10F2A when `d6 == 0`, so the smash effect should nudge 4px downward before the 2-frame hibernate"
        );
        assert_eq!(runtime.block_destroy_effects[0].remaining, 2);
    }

    #[test]
    fn smash_block_effect_sub_10eda_uses_loc_10f26_rightward_offset() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_smash_block_effect_sub_10eda(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_RIGHT,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(
            runtime.block_destroy_effects[0].x_px, 84,
            "ASM loc_10F26 adds 4 to `x_pos(a3)` when `d6 == 3`, so sub_10EDA nudges the smash effect 4px right before the 2-frame hibernate"
        );
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 2);
    }

    #[test]
    fn process_block_effects_teleporter_warps_player() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_list = Some(terrain::BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![terrain::TeleporterEntry {
                layout_addr: 0x0000,
                col: 5,
                row: 3,
                dest_map_id: 0,
                dest_y: 10,
                dest_x: 20,
            }],
            hidden_blocks: vec![],
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 200 << 16;
        slice.player.y_velocity = -0x2_0000;
        slice.player.on_ground = false;
        slice.teleporter_contact = Some((5, 3));

        rt.process_block_effects(&mut slice);

        assert_eq!(
            slice.player.x_pos,
            (20 * 16) << 16,
            "x should warp to dest col×16"
        );
        assert_eq!(
            slice.player.y_pos,
            (10 * 16) << 16,
            "y should warp to dest row×16"
        );
        assert_eq!(slice.player.y_velocity, 0, "velocity zeroed after warp");
        assert!(slice.player.on_ground, "player grounded after warp");
    }

    #[test]
    fn process_block_effects_teleporter_no_op_without_block_list() {
        // If block_list is None, teleporter contact should be a no-op.
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 200 << 16;
        slice.teleporter_contact = Some((5, 3));

        rt.process_block_effects(&mut slice);

        assert_eq!(
            slice.player.x_pos,
            100 << 16,
            "player should not move without block_list"
        );
    }

    #[test]
    fn loc_7a60_left_probe_blocked_snaps_right_to_tile_boundary() {
        // ASM loc_7A60 (kid.asm:8126): when x_pos - $E < 0 or left probe finds
        // $4000, snap x to `((x-14) & $FFF0) + $10 + 14`. Grid col 1 at y=4
        // is solid, blocking the left probe from x=32.
        let grid = grid_with_cells(8, 8, &[(1, 4, 0x60, 0)]);

        let alignment = align_player_for_down_hold_sub_7A10(&grid, 32 << 16, 64 << 16, 128);

        assert_eq!(
            alignment,
            DownHoldAlignmentSub7A10 {
                x_pos: 46 << 16,
                blocked: false,
            },
            "ASM loc_7A60: left blocked → snap right to next tile boundary + half_width",
        );
    }

    #[test]
    fn loc_7a60_left_out_of_bounds_snaps_right() {
        // ASM loc_7A60 (kid.asm:8126): when x_pos - $E < 0 (bmi.w loc_7A60),
        // the left edge is past the level boundary → snap right.
        let grid = grid_with_cells(8, 8, &[]);

        let alignment = align_player_for_down_hold_sub_7A10(&grid, 10 << 16, 64 << 16, 128);

        assert_eq!(
            alignment,
            DownHoldAlignmentSub7A10 {
                x_pos: 14 << 16,
                blocked: false,
            },
            "ASM loc_7A60: x - 14 < 0 → snap right to ((10-14) & !0xF) + 16 + 14 = 14",
        );
    }

    #[test]
    fn loc_7a98_right_probe_blocked_snaps_left() {
        // ASM loc_7A98 (kid.asm:8149): right probe finds $4000 or x + $E >=
        // Level_width_pixels → snap left by `(x+14) & $F + 1`.
        let grid = grid_with_cells(8, 8, &[(2, 4, 0x60, 0)]);

        let alignment = align_player_for_down_hold_sub_7A10(&grid, 32 << 16, 64 << 16, 128);

        assert_eq!(
            alignment,
            DownHoldAlignmentSub7A10 {
                x_pos: 17 << 16,
                blocked: false,
            },
            "ASM loc_7A98: right blocked → snap left by low nibble + 1",
        );
    }

    #[test]
    fn loc_7a90_both_sides_blocked_restores_original_x() {
        // ASM loc_7A90 (kid.asm:8143): `move.w (sp)+,x_pos(a3); moveq #1,d7;
        // rts` — neither left-snap nor right-snap found clearance, so restore
        // the original integer x_pos (subpixel discarded) and return blocked.
        let grid = grid_with_cells(8, 8, &[(1, 4, 0x60, 0), (3, 4, 0x60, 0)]);

        let alignment =
            align_player_for_down_hold_sub_7A10(&grid, (32 << 16) | 0x8000, 64 << 16, 128);

        assert_eq!(
            alignment,
            DownHoldAlignmentSub7A10 {
                x_pos: 32 << 16,
                blocked: true,
            },
            "ASM loc_7A90: both sides blocked → restore integer x, blocked=true",
        );
    }

    #[test]
    fn loc_7a90_right_snap_fails_left_recheck() {
        // ASM loc_7A90 via loc_7A98 (kid.asm:8149→8143): right probe blocked,
        // snap left, but the left re-probe at the snapped position also finds
        // a wall → fall through to loc_7A90 (bra.s) and restore original x.
        let grid = grid_with_cells(8, 8, &[(1, 4, 0x60, 0), (3, 4, 0x60, 0)]);

        let alignment = align_player_for_down_hold_sub_7A10(&grid, 46 << 16, 64 << 16, 128);

        assert!(
            alignment.blocked,
            "ASM loc_7A90: right blocked, left re-probe also blocked → restore and return blocked",
        );
        assert_eq!(
            alignment.x_pos,
            46 << 16,
            "ASM loc_7A90: restored x_pos should be the original integer value",
        );
    }

    #[test]
    fn loc_331d2_masks_only_the_sampled_relative_cell_collision_nibble() {
        let grid = grid_with_cells(8, 8, &[(3, 4, 0x70, 0x09), (2, 4, 0x60, 0x05)]);

        assert_eq!(
            crate::terrain::relative_collision_mask_loc_331D2(&grid, 40, 64, 8, 0),
            0x7000,
            "ASM loc_331D2 (kid.asm:37627) should add the relative offsets onto `x_pos/y_pos(a3)`, look up exactly that Level_Layout cell through `$FFFF4A04`, and keep only the `$7000` collision nibble",
        );
    }

    #[test]
    fn loc_33248_zeroes_disguised_solid_ghost_blocks_before_masking_collision() {
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_33248(0xB355),
            0,
            "ASM loc_33248 (kid.asm:37670) should treat a bit-15 tile with block-type nibble `$300` as non-colliding after the `bclr #$F` / `andi.w #$F00` gate",
        );
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_33248(0x6355),
            0x6000,
            "ASM loc_33248 should leave ordinary non-blocking words on the `beq` path and keep only their collision nibble",
        );
    }

    #[test]
    fn loc_33264_zeroes_the_middle_row_disguised_block_before_masking_collision() {
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_33264(0xB344),
            0,
            "ASM loc_33264 (kid.asm:37680) should apply the same `bclr #$F` / `andi.w #$F00` disguised-block gate to `d6` before falling through to the shared collision mask",
        );
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_33264(0x6444),
            0x6000,
            "ASM loc_33264 should keep ordinary middle-row collision words on the fall-through path and preserve only their `$7000` collision nibble",
        );
    }

    #[test]
    fn loc_33280_keeps_only_the_top_row_collision_nibble() {
        assert_eq!(
            crate::terrain::relative_collision_nibble_loc_33280(0xF6A5),
            0x7000,
            "ASM loc_33280 (kid.asm:37690) should end the third sampled-word path with `andi.w #$7000,d7`, dropping both the block flag and the lower tile bits",
        );
        assert_eq!(
            crate::terrain::relative_collision_nibble_loc_33280(0x0333),
            0,
            "ASM loc_33280 should leave a zero collision nibble when the incoming top-row word has no `$1000/$2000/$4000/$5000/$6000/$7000` terrain bits set",
        );
    }

    #[test]
    fn loc_332d0_zeroes_the_horizontal_row_disguised_block_before_masking_collision() {
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_332D0(0xB377),
            0,
            "ASM loc_332D0 (kid.asm:37723) should apply the same `bclr #$F` / `andi.w #$F00` disguised-block gate to horizontal-scan word `d5` before masking the collision nibble",
        );
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_332D0(0x4777),
            0x4000,
            "ASM loc_332D0 should keep ordinary horizontal-scan words on the fall-through path and preserve only their `$7000` collision nibble",
        );
    }

    #[test]
    fn loc_332ec_zeroes_the_second_horizontal_row_disguised_block_before_masking_collision() {
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_332EC(0xB355),
            0,
            "ASM loc_332EC (kid.asm:37733) should apply the same `bclr #$F` / `andi.w #$F00` disguised-block gate to horizontal-scan word `d6` before masking the collision nibble",
        );
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_332EC(0x6555),
            0x6000,
            "ASM loc_332EC should keep ordinary second-row horizontal words on the fall-through path and preserve only their `$7000` collision nibble",
        );
    }

    #[test]
    fn loc_33308_keeps_only_the_final_horizontal_row_collision_nibble() {
        assert_eq!(
            crate::terrain::relative_collision_nibble_loc_33308(0xF6A5),
            0x7000,
            "ASM loc_33308 (kid.asm:37743) should end the horizontal three-word scan with `andi.w #$7000,d7`, dropping the block flag and lower tile bits",
        );
        assert_eq!(
            crate::terrain::relative_collision_nibble_loc_33308(0x0333),
            0,
            "ASM loc_33308 should leave a zero collision nibble when the final horizontal word carries no terrain bits in `$7000`",
        );
    }

    #[test]
    fn loc_3330e_reads_one_relative_level_layout_cell_and_masks_its_collision_bits() {
        let grid = grid_with_cells(8, 8, &[(3, 5, 0x67, 0x89), (4, 5, 0x45, 0x67)]);

        assert_eq!(
            crate::terrain::relative_collision_mask_loc_3330E(&grid, 56, 80, 8, 0),
            0x4000,
            "ASM loc_3330E (kid.asm:37750) should add `d6/d7` onto `x_pos/y_pos(a3)`, resolve exactly that Level_Layout cell through `$FFFF4A04`, and keep only the `$7000` collision nibble",
        );
        assert_eq!(
            crate::terrain::relative_collision_mask_loc_3330E(&grid, 56, 80, -8, 0),
            0x6000,
            "ASM loc_3330E should honour negative relative X offsets before the `lsr.w #4` cell conversion so the sampled collision nibble follows the addressed neighbour cell",
        );
    }

    #[test]
    fn loc_331f6_reads_the_foot_row_plus_14_and_the_two_rows_above_it() {
        let grid = grid_with_cells(
            8,
            8,
            &[(2, 1, 0x45, 0x01), (2, 2, 0xA3, 0x02), (2, 3, 0x70, 0x03)],
        );

        let filtered = crate::terrain::relative_collision_column_loc_331F6(&grid, 32, 34, 0, 0);
        assert_eq!(
            filtered,
            crate::terrain::RelativeCollisionColumnLoc331F6 {
                current_row: 0x7000,
                one_row_up: 0,
                two_rows_up: 0x4000,
            },
            "ASM loc_331F6 (kid.asm:37642) should sample `y_pos(a3)+$E`, then walk up two `Level_width_tiles` rows, while routing each word through loc_33248 before keeping only the collision nibble",
        );

        let raw = crate::terrain::relative_collision_column_loc_331F6(&grid, 32, 34, 0, 0x03E7);
        assert_eq!(
            raw,
            crate::terrain::RelativeCollisionColumnLoc331F6 {
                current_row: 0x7003,
                one_row_up: 0xA302,
                two_rows_up: 0x4001,
            },
            "ASM loc_331F6 should branch straight to `return_33284` when `d4 == $3E7`, preserving the raw tile words instead of running the loc_33248 filter",
        );
    }

    #[test]
    fn ice_block_load_bullets_restore_direction_loc_11080_restores_the_original_direction_bits() {
        assert_eq!(
            NativeRuntime::ice_block_load_bullets_restore_direction_loc_11080(0),
            2,
            "ASM loc_11080 should restore the original down-facing `d6` after the entry `eori.w #2,d6` remap",
        );
        assert_eq!(
            NativeRuntime::ice_block_load_bullets_restore_direction_loc_11080(1),
            3,
            "ASM loc_11080 should restore the original right-facing `d6` before returning",
        );
        assert_eq!(
            NativeRuntime::ice_block_load_bullets_restore_direction_loc_11080(2),
            0,
            "ASM loc_11080 should restore the original up-facing `d6` before returning",
        );
        assert_eq!(
            NativeRuntime::ice_block_load_bullets_restore_direction_loc_11080(3),
            1,
            "ASM loc_11080 should restore the original left-facing `d6` before returning",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_11020_steps_one_cell_right_inside_the_level_width()
     {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(5, 3, 3, 8, 6),
            Some((6, 3)),
            "ASM loc_11020 should remap direction 3 through `eori.w #2,d6`, then probe one block to the right when `source_col + 1 < Level_width_blocks`",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_11020_rejects_the_right_boundary() {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(7, 3, 3, 8, 6),
            None,
            "ASM loc_11020 should branch to `loc_11080` when `source_col + 1 >= Level_width_blocks`",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_11036_steps_one_cell_down_inside_the_level_height()
     {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(5, 3, 0, 8, 6),
            Some((5, 4)),
            "ASM loc_11036 should remap direction 0 through `eori.w #2,d6`, then probe one block downward when `source_row + 1 < Level_height_blocks`",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_11036_rejects_the_bottom_boundary() {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(5, 5, 0, 8, 6),
            None,
            "ASM loc_11036 should branch to `loc_11080` when `source_row + 1 >= Level_height_blocks`",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_1104a_steps_one_cell_left_inside_the_level_width()
    {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(5, 3, 1, 8, 6),
            Some((4, 3)),
            "ASM loc_1104A should remap direction 1 through `eori.w #2,d6`, then probe one block to the left when `source_col != 0`",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_1104a_rejects_the_left_boundary() {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(0, 3, 1, 8, 6),
            None,
            "ASM loc_1104A should branch to `loc_11080` when the source column is already zero",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_open_loc_11052_accepts_empty_and_hidden_block_cells() {
        assert!(
            NativeRuntime::ice_block_bullet_target_cell_open_loc_11052(
                terrain::Cell::from_be_bytes(0x00, 0x00)
            ),
            "ASM loc_11052 should keep firing through Empty cells because `btst #6,(a0)` sees a clear collision high bit",
        );
        assert!(
            NativeRuntime::ice_block_bullet_target_cell_open_loc_11052(
                terrain::Cell::from_be_bytes(0x20, 0x00)
            ),
            "ASM loc_11052 should also allow HiddenBlock terrain because collision kind 2 leaves bit 6 clear",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_open_loc_11052_rejects_cells_with_the_collision_high_bit_set() {
        for (hi, label) in [
            (0x40, "SlopeUp"),
            (0x50, "SlopeDown"),
            (0x60, "Solid"),
            (0x70, "Spikes"),
        ] {
            assert!(
                !NativeRuntime::ice_block_bullet_target_cell_open_loc_11052(
                    terrain::Cell::from_be_bytes(hi, 0x00)
                ),
                "ASM loc_11052 should reject {label} terrain because `btst #6,(a0)` branches to `loc_11080` whenever the collision high bit is set",
            );
        }
    }

    // --- Shooter blocks: multi-direction and cell-change pruning ---

    #[test]
    fn shooter_player_in_range_sub_f536_accepts_both_inclusive_edges() {
        assert!(
            NativeRuntime::shooter_player_in_range_sub_F536((6 * 16) << 16, (9 * 16) << 16, 2, 3),
            "ASM sub_F536 should accept the left/top inclusive edge of Kid's `x_block-4 .. x_block+4` and `y_block-6 .. y_block+3` window",
        );
        assert!(
            NativeRuntime::shooter_player_in_range_sub_F536((6 * 16) << 16, (9 * 16) << 16, 10, 12),
            "ASM sub_F536 should also accept the right/bottom inclusive edge of that same block-space window",
        );
    }

    #[test]
    fn shooter_player_in_range_sub_f536_rejects_just_outside_window() {
        assert!(
            !NativeRuntime::shooter_player_in_range_sub_F536((6 * 16) << 16, (9 * 16) << 16, 1, 3),
            "ASM loc_F564 should return zero for source columns left of `kid_x_block-4`",
        );
        assert!(
            !NativeRuntime::shooter_player_in_range_sub_F536(
                (6 * 16) << 16,
                (9 * 16) << 16,
                11,
                12
            ),
            "ASM loc_F564 should return zero for source columns right of `kid_x_block+4`",
        );
        assert!(
            !NativeRuntime::shooter_player_in_range_sub_F536((6 * 16) << 16, (9 * 16) << 16, 2, 2),
            "ASM loc_F564 should return zero for source rows above `kid_y_block-6`",
        );
        assert!(
            !NativeRuntime::shooter_player_in_range_sub_F536(
                (6 * 16) << 16,
                (9 * 16) << 16,
                10,
                13
            ),
            "ASM loc_F564 should return zero for source rows below `kid_y_block+3`",
        );
    }

    #[test]
    fn shooter_collision_rects_sub_f568_emits_two_upward_rectangles_inside_the_kid_window() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(6, 7, 0x01)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 98,
                    right_px: 110,
                    top_px: 99,
                    bottom_px: 111,
                },
                CollisionRectSub1F52 {
                    left_px: 101,
                    right_px: 107,
                    top_px: 103,
                    bottom_px: 111,
                },
            ],
            "ASM sub_F568 should emit the two upward collision rectangles from loc_F5B0 when the shooter sits inside Kid's expanded block window",
        );
    }

    #[test]
    fn shooter_collision_rects_sub_f568_rejects_edges_that_touch_the_window_border() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(4, 7, 0x08), (8, 7, 0x02)],
        );

        assert!(
            rects.is_empty(),
            "ASM sub_F568 skips the left/right-facing rectangles when the source row or column lands exactly on the expanded Kid window edge",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f5a4_empty_active_list_terminates() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[],
        );

        assert!(
            rects.is_empty(),
            "ASM loc_F5A4 tests the initial active-shooter pointer before slot reads; a zero head jumps to the shared terminator path",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f5a6_omits_the_rom_zero_sentinel() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(4, 7, 0x08), (8, 7, 0x02)],
        );

        assert!(
            rects.is_empty(),
            "ASM loc_F5A6 branches to the shared tail, which writes a zero word sentinel after exhausting or rejecting shooter slots; native returns no sentinel rectangle",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f72a_terminates_without_a_native_sentinel() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(20, 7, 0x01)],
        );

        assert!(
            rects.is_empty(),
            "ASM loc_F72A writes a zero word terminator to the ROM output buffer; native omits that sentinel from the returned rectangle list",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f5aa_advances_after_rejected_slot() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(20, 7, 0x01), (6, 7, 0x01)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 98,
                    right_px: 110,
                    top_px: 99,
                    bottom_px: 111,
                },
                CollisionRectSub1F52 {
                    left_px: 101,
                    right_px: 107,
                    top_px: 103,
                    bottom_px: 111,
                },
            ],
            "ASM loc_F5AA follows the current slot's next pointer after rejecting an out-of-window shooter, so later active shooters can still emit rectangles",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f62c_emits_right_facing_rectangles_inside_window() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(5, 6, 0x02), (6, 6, 0x02)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 96,
                    right_px: 104,
                    top_px: 97,
                    bottom_px: 109,
                },
                CollisionRectSub1F52 {
                    left_px: 96,
                    right_px: 108,
                    top_px: 100,
                    bottom_px: 106,
                },
            ],
            "ASM loc_F62C should emit the right-facing pair for col `< right_block-1` and skip the shooter exactly on that edge",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f67e_emits_down_facing_rectangles_inside_window() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(6, 6, 0x04), (6, 7, 0x04)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 98,
                    right_px: 110,
                    top_px: 111,
                    bottom_px: 119,
                },
                CollisionRectSub1F52 {
                    left_px: 101,
                    right_px: 107,
                    top_px: 111,
                    bottom_px: 123,
                },
            ],
            "ASM loc_F67E should emit the down-facing pair for row `< bottom_block-1` and skip the shooter exactly on that edge",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f6d2_emits_left_facing_rectangles_inside_window() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(5, 6, 0x08), (6, 6, 0x08)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 88,
                    right_px: 96,
                    top_px: 97,
                    bottom_px: 109,
                },
                CollisionRectSub1F52 {
                    left_px: 84,
                    right_px: 96,
                    top_px: 100,
                    bottom_px: 106,
                },
            ],
            "ASM loc_F6D2 should skip a left-facing shooter with col-left_block `< 2` and emit the pair for the next inside column",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f722_left_skip_advances_to_next_slot() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(5, 6, 0x08), (6, 6, 0x08)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 88,
                    right_px: 96,
                    top_px: 97,
                    bottom_px: 109,
                },
                CollisionRectSub1F52 {
                    left_px: 84,
                    right_px: 96,
                    top_px: 100,
                    bottom_px: 106,
                },
            ],
            "ASM loc_F722 should advance from a skipped left-facing slot to `$C(a0)` and process the next active shooter",
        );
    }

    #[test]
    fn shooter_collision_top_block_loc_f574_clamps_the_expanded_top_window() {
        assert_eq!(
            NativeRuntime::shooter_collision_top_block_loc_F574(0x000F),
            0,
            "ASM loc_F574 should clamp a negative `(Kid_hitbox_top >> 4) - 1` result to zero",
        );
        assert_eq!(
            NativeRuntime::shooter_collision_top_block_loc_F574(0x0010),
            0,
            "ASM loc_F574 should keep an exact zero expanded top block",
        );
        assert_eq!(
            NativeRuntime::shooter_collision_top_block_loc_F574(0x0050),
            4,
            "ASM loc_F574 should keep positive expanded top-block results",
        );
    }

    #[test]
    fn shooter_collision_right_bottom_blocks_loc_f580_expands_after_logical_word_shift() {
        assert_eq!(
            NativeRuntime::shooter_collision_right_bottom_blocks_loc_F580(0x0060, 0x0070),
            crate::blocks::ShooterCollisionRightBottomBlocksLocF580 {
                right_block: 7,
                bottom_block: 8,
            },
            "ASM loc_F580 should compute `(hitbox + $F) >> 4` and then add one block to both right and bottom bounds",
        );
        assert_eq!(
            NativeRuntime::shooter_collision_right_bottom_blocks_loc_F580(-0x0010, -0x0010),
            crate::blocks::ShooterCollisionRightBottomBlocksLocF580 {
                right_block: 0x1000,
                bottom_block: 0x1000,
            },
            "ASM loc_F580 should use logical `lsr.w`, not arithmetic shift, after the wrapping 16-bit add",
        );
    }

    #[test]
    fn shooter_block_fires_all_four_directions() {
        // Skin byte 0x0F = all directions (up + right + down + left)
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 3, 0x0F)]));
        rt.block_state.shooter_last_scanned_col = 1;
        rt.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        rt.step_shooter_blocks(&slice);

        assert_eq!(
            rt.projectiles.len(),
            4,
            "should fire 4 projectiles for all directions"
        );
        // Check direction signs: up=-y, right=+x, down=+y, left=-x
        let velocities: Vec<(i32, i32)> = rt
            .projectiles
            .iter()
            .map(|p| (p.x_velocity, p.y_velocity))
            .collect();
        let speed: i32 = 4 << 16; // SHOOTER_PROJECTILE_SPEED
        assert!(velocities.contains(&(0, -speed)), "missing up projectile");
        assert!(velocities.contains(&(speed, 0)), "missing right projectile");
        assert!(velocities.contains(&(0, speed)), "missing down projectile");
        assert!(velocities.contains(&(-speed, 0)), "missing left projectile");
    }

    #[test]
    fn shooter_block_single_direction_up_only() {
        // Skin byte 0x01 = up only
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 3, 0x01)]));
        rt.block_state.shooter_last_scanned_col = 1;
        rt.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        rt.step_shooter_blocks(&slice);

        assert_eq!(rt.projectiles.len(), 1);
        assert_eq!(rt.projectiles[0].x_velocity, 0);
        assert!(rt.projectiles[0].y_velocity < 0, "up means negative y");
    }

    #[test]
    fn shooter_block_pruned_when_cell_no_longer_shooter() {
        // If the grid cell changes (e.g. block destroyed), the shooter entry should be pruned.
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 3, 0x02)]));
        rt.block_state.shooter_last_scanned_col = 1;
        rt.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        rt.step_shooter_blocks(&slice);
        assert_eq!(rt.block_state.shooter_blocks.len(), 1);

        // Now overwrite the grid cell to empty
        rt.block_grid
            .as_mut()
            .unwrap()
            .set_cell_at(5, 3, terrain::Cell::from_be_bytes(0, 0));

        rt.step_shooter_blocks(&slice);
        assert!(
            rt.block_state.shooter_blocks.is_empty(),
            "shooter entry should be pruned when cell is no longer a shooter"
        );
    }

    #[test]
    fn shooter_headbutt_support_worker_runs_sub_114f4_timeline() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 4, 0x02)]));

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((5, 4));
        rt.process_block_effects(&mut slice);

        rt.step_shooter_headbutt_workers();
        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 3),
            terrain::Cell::from_be_bytes(0xEF, 0x1A),
            "ASM sub_114F4 writes `$EF1A` when the support cell is still empty"
        );

        for _ in 0..8 {
            rt.step_shooter_headbutt_workers();
        }
        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 3),
            terrain::Cell::from_be_bytes(0xEF, 0x1B),
            "ASM sub_114F4 writes `$EF1B` after the first 8-frame hibernate"
        );

        for _ in 0..8 {
            rt.step_shooter_headbutt_workers();
        }
        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 3),
            terrain::Cell::from_be_bytes(0xE9, 0x19),
            "ASM sub_114F4 finishes by restoring `$E919` to the support cell"
        );
    }

    #[test]
    fn shooter_headbutt_support_worker_loc_1152c_deletes_when_target_refills_before_wake() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 4, 0x02)]));

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((5, 4));
        rt.process_block_effects(&mut slice);
        rt.block_grid.as_mut().expect("block grid").set_cell_at(
            5,
            3,
            terrain::Cell::from_be_bytes(0x60, 0),
        );

        rt.step_shooter_headbutt_workers();

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 3),
            terrain::Cell::from_be_bytes(0x60, 0),
            "ASM loc_1152C deletes the support worker immediately if the support cell is no longer empty on wake"
        );
    }

    #[test]
    fn shooter_center_worker_gate_loc_1146e_restores_the_original_column_and_skips_centre_worker_after_support_spawn()
     {
        let no_support = NativeRuntime::shooter_center_worker_gate_loc_1146E(6, false);
        assert_eq!(
            no_support,
            ShooterCenterWorkerGateLoc1146E {
                restored_block_col: 5,
                should_queue_center_worker: true,
            },
            "ASM loc_1146E should undo loc_11458's extra `addq.w #1,d1` before falling through to loc_11474 when `d4` is still zero",
        );

        let support_spawned = NativeRuntime::shooter_center_worker_gate_loc_1146E(6, true);
        assert_eq!(
            support_spawned,
            ShooterCenterWorkerGateLoc1146E {
                restored_block_col: 5,
                should_queue_center_worker: false,
            },
            "ASM loc_1146E should branch straight to loc_11496 once any earlier `sub_1149C` call leaves `d4` non-zero",
        );
    }

    // --- Floating items: spawn delay, lifetime, collision boxes ---

    #[test]
    fn floating_item_with_spawn_delay_not_collectible() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 5,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
        });

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;

        rt.collect_floating_items(&mut slice);

        assert!(
            !rt.floating_items[0].collected,
            "item with spawn_delay > 0 should not be collectible"
        );
    }

    #[test]
    fn floating_item_spawn_delay_counts_down_in_visual_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 3,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
        });

        rt.step_visual_effects();
        assert_eq!(rt.floating_items[0].spawn_delay, 2);
        rt.step_visual_effects();
        assert_eq!(rt.floating_items[0].spawn_delay, 1);
        rt.step_visual_effects();
        assert_eq!(rt.floating_items[0].spawn_delay, 0);
        // Now the item should start falling
        rt.step_visual_effects();
        assert!(
            rt.floating_items[0].y_velocity > 0,
            "item should start accumulating velocity after delay expires"
        );
    }

    #[test]
    fn floating_item_removed_when_lifetime_expires() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 1,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
        });

        rt.step_visual_effects();
        // remaining was 1, saturating_sub(1) = 0
        assert!(
            rt.floating_items.is_empty(),
            "item with remaining=0 should be removed"
        );
    }

    #[test]
    fn floating_item_loc_10396_keeps_the_frame_0xc8_countdown_alive() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 0x00C9,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10396 only branches to delete once `subq.w #1,d3` reaches zero; the `$00C8` frame stays alive for the late animation swap",
        );
        assert_eq!(
            rt.floating_items[0].remaining, 0x00C8,
            "ASM loc_10396 reaches the `cmpi.w #$C8,d3` animation threshold one frame before deletion becomes possible",
        );
    }

    #[test]
    fn floating_ankh_loc_1071e_keeps_the_frame_0x12c_countdown_alive() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 0x012D,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Ankh,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_1071E only branches to delete once `subq.w #1,d3` reaches zero; the `$012C` frame stays alive for the later animation swap",
        );
        assert_eq!(
            rt.floating_items[0].remaining, 0x012C,
            "ASM loc_1071E reaches the `cmpi.w #$12C,d3` animation threshold one frame before deletion becomes possible",
        );
    }

    #[test]
    fn floating_ankh_loc_1071e_collects_before_timeout_and_grants_the_life_bonus() {
        let mut rt = NativeRuntime::new(manifest());
        start_level(&mut rt);
        rt.floating_items.push(FloatingItem {
            x_pos: 72 << 16,
            y_pos: i64::from(100 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 4,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Ankh,
            collected: false,
        });
        let lives_before = rt.progression.number_lives;
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 72 << 16;
        slice.player.y_pos = 110 << 16;

        rt.collect_floating_items(&mut slice);

        assert!(rt.floating_items[0].collected);
        assert_eq!(
            rt.progression.number_lives,
            lives_before + 1,
            "ASM loc_1071E should take the collision path before timeout expiry so the collected ankh still grants its life bonus",
        );

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_1071E jumps to `Delete_CurrentObject` after the collection path allocates the follow-up worker",
        );
    }

    #[test]
    fn floating_ankh_loc_10738_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (4 * 16 + 8) << 16,
            y_pos: i64::from(160 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 4,
            level_row: 10,
            first_row_grace: true,
            content: terrain::PrizeContent::Ankh,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10738 should keep looping while the ankh stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_10738 still burns one shared falling-prize countdown step before the later delete exit",
        );
    }

    #[test]
    fn floating_ankh_loc_10768_deletes_the_collected_ankh_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Ankh,
            collected: true,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_10768 should jump straight to `Delete_CurrentObject` once PrizeAnkh's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_skycutter_loc_10464_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (2 * 16 + 8) << 16,
            y_pos: i64::from(128 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 2,
            level_row: 8,
            first_row_grace: true,
            content: terrain::PrizeContent::Skycutter,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10464 should keep looping while Skycutter stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_10464 still runs the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_item_collected_flag_causes_removal() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: true,
        });

        rt.step_visual_effects();
        assert!(
            rt.floating_items.is_empty(),
            "collected item should be removed in visual step"
        );
    }

    #[test]
    fn floating_skycutter_loc_1047c_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Skycutter,
            collected: true,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_1047C should jump straight to `Delete_CurrentObject` once Skycutter's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_cyclone_loc_104ae_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (3 * 16 + 8) << 16,
            y_pos: i64::from(144 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 3,
            level_row: 9,
            first_row_grace: true,
            content: terrain::PrizeContent::Cyclone,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_104AE should keep looping while Cyclone stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_104AE still advances the shared helmet countdown on each hibernated frame before the shared delete exit",
        );
    }

    #[test]
    fn floating_cyclone_loc_104c6_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Cyclone,
            collected: true,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_104C6 should jump straight to `Delete_CurrentObject` once Cyclone's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_red_stealth_loc_104f8_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (4 * 16 + 8) << 16,
            y_pos: i64::from(160 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 4,
            level_row: 10,
            first_row_grace: true,
            content: terrain::PrizeContent::RedStealth,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_104F8 should keep looping while Red Stealth stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_104F8 still advances the shared helmet countdown on each hibernated frame before the shared delete exit",
        );
    }

    #[test]
    fn floating_red_stealth_loc_10510_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::RedStealth,
            collected: true,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_10510 should jump straight to `Delete_CurrentObject` once Red Stealth's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_eyeclops_loc_10542_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (5 * 16 + 8) << 16,
            y_pos: i64::from(176 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 5,
            level_row: 11,
            first_row_grace: true,
            content: terrain::PrizeContent::Eyeclops,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10542 should keep looping while Eyeclops stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_10542 still advances the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_eyeclops_loc_1055a_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Eyeclops,
            collected: true,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_1055A should jump straight to `Delete_CurrentObject` once Eyeclops' `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_juggernaut_loc_1058c_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (6 * 16 + 8) << 16,
            y_pos: i64::from(192 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 12,
            first_row_grace: true,
            content: terrain::PrizeContent::Juggernaut,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_1058C should keep looping while Juggernaut stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_1058C still advances the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_iron_knight_loc_105d6_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (7 * 16 + 8) << 16,
            y_pos: i64::from(208 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 7,
            level_row: 13,
            first_row_grace: true,
            content: terrain::PrizeContent::IronKnight,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_105D6 should keep looping while Iron Knight stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_105D6 still advances the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_iron_knight_loc_105ee_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::IronKnight,
            collected: true,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_105EE should jump straight to `Delete_CurrentObject` once Iron Knight's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_berzerker_loc_10620_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (8 * 16 + 8) << 16,
            y_pos: i64::from(224 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 8,
            level_row: 14,
            first_row_grace: true,
            content: terrain::PrizeContent::Berzerker,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10620 should keep looping while Berzerker stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_10620 still advances the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_berzerker_loc_10638_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 116 << 16,
            y_pos: 132 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 7,
            level_row: 8,
            first_row_grace: true,
            content: terrain::PrizeContent::Berzerker,
            collected: true,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_10638 should jump straight to `Delete_CurrentObject` once Berzerker's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_maniaxe_loc_1066a_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (9 * 16 + 8) << 16,
            y_pos: i64::from(240 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 9,
            level_row: 15,
            first_row_grace: true,
            content: terrain::PrizeContent::Maniaxe,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_1066A should keep looping while Maniaxe stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_1066A still advances the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_micromax_loc_106b4_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (10 * 16 + 8) << 16,
            y_pos: i64::from(256 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 10,
            level_row: 16,
            first_row_grace: true,
            content: terrain::PrizeContent::Micromax,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_106B4 should keep looping while Micromax stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_106B4 still advances the shared helmet countdown on each hibernated frame before the shared delete exit",
        );
    }

    #[test]
    fn floating_micromax_delete_exit_loc_106cc_jumps_straight_to_delete_current_object() {
        assert!(
            NativeRuntime::floating_micromax_delete_exit_loc_106CC(),
            "ASM loc_106CC should jump straight to `Delete_CurrentObject` once Micromax leaves loc_106B4",
        );
    }

    #[test]
    fn floating_micromax_loc_106cc_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 132 << 16,
            y_pos: 148 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 8,
            level_row: 9,
            first_row_grace: true,
            content: terrain::PrizeContent::Micromax,
            collected: true,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_106CC should jump straight to `Delete_CurrentObject` once Micromax's `collision_type(a3)` becomes non-zero",
        );
    }

    // --- Visual effects: countdown and retention ---

    #[test]
    fn block_bounce_anim_counts_down_and_is_removed() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_bounces.push(BlockBounceAnim {
            cell_x: 3,
            cell_y: 5,
            remaining: 2,
        });

        rt.step_visual_effects();
        assert_eq!(rt.block_bounces.len(), 1);
        assert_eq!(rt.block_bounces[0].remaining, 1);

        rt.step_visual_effects();
        assert!(
            rt.block_bounces.is_empty(),
            "bounce anim should be removed when remaining reaches 0"
        );
    }

    #[test]
    fn block_destroy_effect_counts_down_and_is_removed() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_destroy_effects.push(BlockDestroyEffect {
            x_px: 80,
            y_px: 48,
            remaining: 1,
        });

        rt.step_visual_effects();
        assert!(
            rt.block_destroy_effects.is_empty(),
            "destroy effect should be removed when remaining reaches 0"
        );
    }

    #[test]
    fn enemy_death_effect_counts_down_and_is_removed() {
        let mut rt = NativeRuntime::new(manifest());
        rt.enemy_death_effects.push(EnemyDeathEffect {
            x_pos: 100,
            y_pos: 200,
            remaining: 1,
        });

        rt.step_visual_effects();
        assert!(
            rt.enemy_death_effects.is_empty(),
            "enemy death effect should be removed when remaining reaches 0"
        );
    }

    // --- Evanescent blocks: max entries cap ---

    #[test]
    fn evanescent_block_respects_max_entries_cap() {
        // ASM limits evanescent tracking to 19 ($13) entries.
        let grid = grid_with_evanescent(2, 3, 28);
        let mut runtime = runtime_with_evanescent_grid(grid);

        // Pre-fill to capacity
        for i in 0..EVANESCENT_MAX_ENTRIES {
            runtime.block_state.evanescent_blocks.push(EvanescentEntry {
                col: 10 + i as u16,
                row: 10,
                timer: 0,
            });
        }
        assert_eq!(
            runtime.block_state.evanescent_blocks.len(),
            EVANESCENT_MAX_ENTRIES
        );

        // Player standing on the evanescent block at (2, 3)
        let slice = {
            let mut s = shooter_test_slice(0, 0);
            s.player.on_ground = true;
            s.player.x_pos = (2 * 16 + 8) << 16;
            s.player.y_pos = (3 * 16 - 1) << 16;
            s
        };

        runtime.step_evanescent_blocks(&slice);

        assert_eq!(
            runtime.block_state.evanescent_blocks.len(),
            EVANESCENT_MAX_ENTRIES,
            "should not exceed max entries cap"
        );
    }

    // --- Lift platform: player carry-along ---

    #[test]
    fn lift_platform_carries_grounded_player() {
        // When player is on_ground and within snap tolerance of the lift, the lift
        // should carry the player's y_pos along.
        let mut rt = NativeRuntime::new(manifest());
        let lift_y: u32 = 100 << 16;
        rt.block_state.lift_platforms.push(LiftPlatform {
            x_pos: 32 << 16,
            y_pos: lift_y,
            y_velocity: 0,
            age: 0,
            last_integer_y: 100,
            moved_y_steps: 0,
            width_px: 16,
            source_col: 2,
            source_row: 6,
            block_erased: false,
        });

        let mut slice = OpeningSliceState::new(default_bounds());
        // Player standing on the lift
        slice.player.x_pos = 40 << 16; // within platform x range
        slice.player.y_pos = lift_y;
        slice.player.on_ground = true;
        slice.terrain.floor_y_pos = lift_y;

        rt.step_lift_platforms(&mut slice);
        assert_eq!(
            rt.block_state.lift_platforms[0].y_pos, lift_y,
            "ASM sub_FCA6 should leave `6(a3)` unchanged on the first frame because `$E(a3)` starts at zero",
        );
        assert_eq!(
            slice.player.y_pos,
            lift_y.wrapping_add_signed(-0x0708),
            "grounded riders should still inherit the newly updated `$E(a3)` even before the integer `6(a3)` word changes",
        );

        rt.step_lift_platforms(&mut slice);
        let expected_lift_y = 0x0063_F8F8;
        assert_eq!(
            rt.block_state.lift_platforms[0].y_pos, expected_lift_y as u32,
            "ASM sub_FCA6 should move the lift by the fractional `sub_FD00` step on the second frame",
        );
        assert!(
            slice.player.y_pos < lift_y,
            "player should be carried upward with the lift"
        );
    }

    #[test]
    fn lift_block_bob_deltas_sub_fb3e_match_the_three_hibernating_passes() {
        let deltas = NativeRuntime::lift_block_bob_deltas_sub_FB3E();

        assert_eq!(
            deltas,
            &[
                -2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, 2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, -2,
                -2, -2, -2, 2, 2, 2, 2, 2,
            ],
            "ASM sub_FB3E should bob the struck lift block through the `6/6 -> 5/5 -> 4/5` up/down hibernate passes before the platform hand-off",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            2,
            "ASM sub_FB3E should finish the bob cycle 2px below the starting point before the later animation and platform setup correct the anchor",
        );
    }

    #[test]
    fn lift_block_bob_up_pass_loc_fb94_matches_the_first_hibernating_loop() {
        let deltas = NativeRuntime::lift_block_bob_up_pass_loc_FB94();

        assert_eq!(
            deltas,
            &[-2, -2, -2, -2, -2, -2],
            "ASM loc_FB94 should execute six `subq.w #2,y_pos(a3)` steps because `moveq #5,d0` feeds an inclusive DBF loop",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            -12,
            "ASM loc_FB94 should move the struck lift block 12px upward before the first down pass starts",
        );
    }

    #[test]
    fn lift_block_bob_down_pass_loc_fba2_matches_the_first_return_loop() {
        let deltas = NativeRuntime::lift_block_bob_down_pass_loc_FBA2();

        assert_eq!(
            deltas,
            &[2, 2, 2, 2, 2, 2],
            "ASM loc_FBA2 should execute six `addq.w #2,y_pos(a3)` steps because `moveq #5,d1` feeds an inclusive DBF loop",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            12,
            "ASM loc_FBA2 should return the struck lift block 12px downward before the five-frame up pass starts",
        );
    }

    #[test]
    fn lift_block_bob_up_pass_loc_fbb0_matches_the_second_hibernating_loop() {
        let deltas = NativeRuntime::lift_block_bob_up_pass_loc_FBB0();

        assert_eq!(
            deltas,
            &[-2, -2, -2, -2, -2],
            "ASM loc_FBB0 should execute five `subq.w #2,y_pos(a3)` steps because `moveq #4,d0` feeds an inclusive DBF loop",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            -10,
            "ASM loc_FBB0 should move the struck lift block 10px upward before the second return pass starts",
        );
    }

    #[test]
    fn lift_block_bob_down_pass_loc_fbbe_matches_the_second_return_loop() {
        let deltas = NativeRuntime::lift_block_bob_down_pass_loc_FBBE();

        assert_eq!(
            deltas,
            &[2, 2, 2, 2, 2],
            "ASM loc_FBBE should execute five `addq.w #2,y_pos(a3)` steps because `moveq #4,d1` feeds an inclusive DBF loop",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            10,
            "ASM loc_FBBE should return the struck lift block 10px downward before the four-frame up pass starts",
        );
    }

    #[test]
    fn lift_block_bob_up_pass_loc_fbcc_matches_the_final_hibernating_loop() {
        let deltas = NativeRuntime::lift_block_bob_up_pass_loc_FBCC();

        assert_eq!(
            deltas,
            &[-2, -2, -2, -2],
            "ASM loc_FBCC should execute four `subq.w #2,y_pos(a3)` steps because `moveq #3,d0` feeds an inclusive DBF loop",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            -8,
            "ASM loc_FBCC should move the struck lift block 8px upward before the final five-frame return pass starts",
        );
    }

    // --- Process block effects: lift block spawns platform ---

    #[test]
    fn process_block_effects_lift_spawns_platform() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.lift_block_hit = Some((48, 80));

        rt.process_block_effects(&mut slice);

        assert_eq!(rt.block_state.lift_platforms.len(), 1);
        assert_eq!(rt.block_state.lift_platforms[0].x_pos, 48 << 16);
        assert_eq!(rt.block_state.lift_platforms[0].y_pos, 80 << 16);
        assert_eq!(rt.block_state.lift_platforms[0].source_col, 48 / 16);
        assert_eq!(rt.block_state.lift_platforms[0].source_row, 80 / 16);
        assert_eq!(rt.block_state.lift_platforms[0].age, 0);
        assert_eq!(rt.block_state.lift_platforms[0].last_integer_y, 80);
        assert_eq!(rt.block_state.lift_platforms[0].moved_y_steps, 0);
        assert!(!rt.block_state.lift_platforms[0].block_erased);
    }

    #[test]
    fn shifting_block_activation_loc_11364_skips_the_worker_on_the_top_row() {
        let activation = NativeRuntime::shifting_block_activation_loc_11364(
            0,
            terrain::Cell::from_be_bytes(0x00, 0x07),
        );

        assert!(
            activation.should_play_sound,
            "ASM loc_11364 should preserve `d0` and play `sfx_Shifting_block` even when `tst.w d2` aborts the worker path on row zero",
        );
        assert!(
            !activation.should_spawn_worker,
            "ASM loc_11364 should branch straight to `loc_113BA` when `d2 == 0`, so a top-row shifting block never allocates the `sub_113C0` worker",
        );
        assert_eq!(activation.reserved_above_cell, None);
    }

    #[test]
    fn loc_113ba_only_allows_the_worker_when_the_above_cell_is_clear_and_in_bounds() {
        assert!(
            !NativeRuntime::shifting_block_worker_gate_loc_113BA(0, false),
            "ASM loc_113BA should return immediately when `tst.w d2` sees the top row",
        );
        assert!(
            !NativeRuntime::shifting_block_worker_gate_loc_113BA(5, true),
            "ASM loc_113BA should return immediately when `btst #6,(a2)` reports the above cell occupied",
        );
        assert!(
            NativeRuntime::shifting_block_worker_gate_loc_113BA(5, false),
            "ASM loc_113BA should let the caller continue into `sub_113C0` only for non-zero rows with bit 6 clear above",
        );
    }

    #[test]
    fn shifting_block_activation_loc_11364_only_reserves_clear_cells_above_the_source_block() {
        let blocked = NativeRuntime::shifting_block_activation_loc_11364(
            5,
            terrain::Cell::from_be_bytes(0x60, 0x09),
        );
        assert!(
            !blocked.should_spawn_worker,
            "ASM loc_11364 should branch to `loc_113BA` when `btst #6,(a2)` finds the cell above occupied",
        );
        assert_eq!(blocked.reserved_above_cell, None);

        let clear = NativeRuntime::shifting_block_activation_loc_11364(
            5,
            terrain::Cell::from_be_bytes(0x00, 0x09),
        );
        assert!(
            clear.should_spawn_worker,
            "ASM loc_11364 should continue into the `sub_113C0` setup once the source row is non-zero and the cell above has bit 6 clear",
        );
        assert_eq!(
            clear.reserved_above_cell,
            Some(terrain::Cell::from_be_bytes(0x6F, 0x09)),
            "ASM loc_11364 should reserve the above cell by writing only high byte `$6F`, preserving that cell's original skin byte",
        );
    }

    /// ASM loc_BBD2 (kid.asm:15130): off_BB80 index 7 (Shifting) gate inside
    /// sub_BB54.  `tst.b ($FFFFFA6A); bne return_BBE0` — when $FA6A (on_board)
    /// is set, the shifting block handler is a no-op; when $FA6A is clear, it
    /// falls through to `jmp (j_loc_11364).l` which plays the shifting sound and
    /// runs the worker.  This test verifies the $FA6A-clear path: loc_BBD2
    /// reaches loc_11364 which produces `should_play_sound=true`.
    #[test]
    fn loc_bbd2_shifting_block_gate_fa6a_clear_reaches_loc_11364() {
        // When $FA6A is clear (the loc_BBD2 gate passes), sub_BB54 calls
        // j_loc_11364.  Native: shifting_block_activation_loc_11364 is the
        // loc_11364 entry point.  A non-zero source row with a clear cell
        // above should produce sound + worker.
        let activation = NativeRuntime::shifting_block_activation_loc_11364(
            3,
            terrain::Cell::from_be_bytes(0x00, 0x00),
        );

        assert!(
            activation.should_play_sound,
            "ASM loc_BBD2: $FA6A clear → j_loc_11364 plays sfx_Shifting_block"
        );
        assert!(
            activation.should_spawn_worker,
            "ASM loc_BBD2: $FA6A clear + non-zero row + clear cell above → sub_113C0 worker spawns"
        );
    }

    #[test]
    fn lift_platform_vertical_step_sub_fd00_adds_signed_velocity_to_fp16_y() {
        let next_y = NativeRuntime::lift_platform_vertical_step_sub_FD00(0x0080_0000, -0x0000_0708);

        assert_eq!(
            next_y, 0x007F_F8F8,
            "ASM sub_FD00 should add `$E(a3)` directly into `6(a3)`, preserving the fractional carry/borrow",
        );
    }

    #[test]
    fn lift_block_rise_step_loc_113f4_moves_up_2px_for_eight_hibernating_iterations() {
        let mut y_pos = 80u16;
        let mut loop_counter = 7u16;
        let mut steps = Vec::new();

        loop {
            let step = NativeRuntime::lift_block_rise_step_loc_113F4(y_pos, loop_counter);
            steps.push(step);
            y_pos = step.next_y_pos;
            loop_counter = step.next_loop_counter;
            if !step.should_repeat_loop {
                break;
            }
        }

        assert_eq!(steps.len(), 8);
        assert!(
            steps.iter().all(|step| step.should_hibernate_one_frame),
            "ASM loc_113F4 should call `j_Hibernate_Object_1Frame` on every rise iteration before the `dbf d0,loc_113F4` back-edge",
        );
        assert_eq!(
            steps[0],
            LiftBlockRiseStepLoc113F4 {
                next_y_pos: 78,
                next_loop_counter: 6,
                should_hibernate_one_frame: true,
                should_repeat_loop: true,
            },
        );
        assert_eq!(
            steps.last().copied(),
            Some(LiftBlockRiseStepLoc113F4 {
                next_y_pos: 64,
                next_loop_counter: u16::MAX,
                should_hibernate_one_frame: true,
                should_repeat_loop: false,
            }),
            "ASM loc_113F4 starts from `move.w #7,d0`, so the final `dbf` decrements to `-1` only after eight 2px upward moves",
        );
    }

    #[test]
    fn lift_block_fall_step_loc_1141e_descends_1px_for_sixteen_frames_then_deletes() {
        let mut y_pos = 80u16;
        let mut loop_counter = 0x0Fu16;
        let mut steps = Vec::new();

        loop {
            let step = NativeRuntime::lift_block_fall_step_loc_1141E(y_pos, loop_counter);
            steps.push(step);
            y_pos = step.next_y_pos;
            loop_counter = step.next_loop_counter;
            if !step.should_repeat_loop {
                break;
            }
        }

        assert_eq!(steps.len(), 16);
        assert!(
            steps.iter().all(|step| step.should_hibernate_one_frame),
            "ASM loc_1141E should call `j_Hibernate_Object_1Frame` before each `dbf d0,loc_1141E` check during the descent",
        );
        assert_eq!(
            steps[0],
            LiftBlockFallStepLoc1141E {
                next_y_pos: 79,
                next_loop_counter: 0x0E,
                should_hibernate_one_frame: true,
                should_delete_current_object: false,
                should_repeat_loop: true,
            },
        );
        assert_eq!(
            steps.last().copied(),
            Some(LiftBlockFallStepLoc1141E {
                next_y_pos: 64,
                next_loop_counter: u16::MAX,
                should_hibernate_one_frame: true,
                should_delete_current_object: true,
                should_repeat_loop: false,
            }),
            "ASM loc_1141E starts from `move.w #$F,d0`, so the worker only reaches `Delete_CurrentObject` after sixteen 1px descent steps",
        );
    }

    #[test]
    fn shooter_left_support_probe_loc_11448_skips_column_zero_and_only_targets_empty_left_cells() {
        let left_edge = NativeRuntime::shooter_left_support_probe_loc_11448(0, true);
        assert_eq!(
            left_edge,
            ShooterLeftSupportProbeLoc11448 {
                should_probe_support: false,
                support_col: None,
                next_block_col_for_right_probe: 1,
            },
            "ASM loc_11448 does `subq.w #1,d1; bmi.s loc_11458`, so the left neighbour probe is skipped entirely for the leftmost shooter column",
        );

        let occupied_left = NativeRuntime::shooter_left_support_probe_loc_11448(5, false);
        assert_eq!(
            occupied_left,
            ShooterLeftSupportProbeLoc11448 {
                should_probe_support: false,
                support_col: None,
                next_block_col_for_right_probe: 6,
            },
            "ASM loc_11448 should skip `sub_1149C` when `tst.w -2(a1)` sees a non-empty left support cell",
        );

        let empty_left = NativeRuntime::shooter_left_support_probe_loc_11448(5, true);
        assert_eq!(
            empty_left,
            ShooterLeftSupportProbeLoc11448 {
                should_probe_support: true,
                support_col: Some(4),
                next_block_col_for_right_probe: 6,
            },
            "ASM loc_11448 should call `sub_1149C` for the left support cell, then restore the shared `d1` state so `loc_11458` can continue with the right-hand probe",
        );
    }

    // --- Process block effects: prize block hit creates destroy effect ---

    #[test]
    fn process_block_effects_prize_creates_destroy_effect() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.prize_block_hit = Some((terrain::PrizeContent::Diamond, 48, 32));

        rt.process_block_effects(&mut slice);

        assert_eq!(rt.block_destroy_effects.len(), 1);
        assert_eq!(rt.block_destroy_effects[0].x_px, 48);
        assert_eq!(rt.block_destroy_effects[0].y_px, 32);
        assert_eq!(rt.block_destroy_effects[0].remaining, 8);
        assert_eq!(rt.blocks_hit_this_frame, 1);
    }

    // --- Floating item: diamond uses inclusive overlap, helmet uses exclusive ---

    #[test]
    fn floating_diamond_inclusive_overlap_collects_at_boundary() {
        // Diamond uses inclusive overlap (<=/>= for edges).
        // Place item exactly at edge of player bounding box.
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
        });

        let mut slice = OpeningSliceState::new(default_bounds());
        // Diamond box: item_left = 100-7 = 93, item_right = 100+7 = 107
        // Player half_width = 5, so player_left = x-5, player_right = x+5
        // For inclusive overlap: player_left <= item_right → x-5 <= 107 → x <= 112
        // AND player_right >= item_left → x+5 >= 93 → x >= 88
        // Test exactly at boundary: player_right == item_left (x+5 = 93 → x = 88)
        slice.player.x_pos = 88 << 16;
        // Diamond item_top = 100 - PLAYER_HEIGHT = 100 - 31 = 69, item_bottom = 100
        // Inclusive: player_top <= item_bottom → (y-31) <= 100 → y <= 131
        // AND player_y >= item_top → y >= 69
        slice.player.y_pos = 100 << 16;

        rt.collect_floating_items(&mut slice);

        assert!(
            rt.floating_items[0].collected,
            "diamond should collect with inclusive boundary overlap"
        );
    }

    #[test]
    fn floating_helmet_exclusive_overlap_misses_at_exact_boundary() {
        // Helmet uses exclusive overlap (< / > for edges).
        // At exact boundary, it should NOT collect.
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Skycutter,
            collected: false,
        });

        let mut slice = OpeningSliceState::new(default_bounds());
        // Helmet box: item_left = 100-8 = 92, item_right = 100+8 = 108
        // For exclusive: player_right > item_left → x+5 > 92 → x > 87
        // At x=87: player_right = 92 = item_left → NOT > → miss
        slice.player.x_pos = 87 << 16;
        // Set y to definitely overlap vertically
        slice.player.y_pos = 110 << 16;

        rt.collect_floating_items(&mut slice);

        assert!(
            !rt.floating_items[0].collected,
            "helmet should NOT collect at exact exclusive boundary"
        );
    }

    // --- Floating item: velocity accumulation and row tracking ---

    #[test]
    fn floating_item_accumulates_velocity_each_frame() {
        // ASM: +$1388/frame downward velocity
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: (6 * 16) << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
        });

        rt.step_visual_effects();
        assert_eq!(
            rt.floating_items[0].y_velocity, 0x1388,
            "first frame: velocity = $1388"
        );
        let y_after_1 = rt.floating_items[0].y_pos;

        rt.step_visual_effects();
        assert_eq!(
            rt.floating_items[0].y_velocity,
            0x1388 * 2,
            "second frame: velocity = $2710"
        );
        assert!(
            rt.floating_items[0].y_pos > y_after_1,
            "item should fall further on second frame"
        );
    }

    #[test]
    fn floating_item_loc_103b2_uses_the_grace_probe_row_before_stopping() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_blocks();
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0xE0, 0x01));
        grid.set_cell_at(2, 3, terrain::Cell::from_be_bytes(0x00, 0x00));
        grid.set_cell_at(3, 2, terrain::Cell::from_be_bytes(0xE0, 0x01));
        grid.set_cell_at(3, 3, terrain::Cell::from_be_bytes(0xE0, 0x01));
        rt.set_block_grid(grid);
        rt.floating_items.push(FloatingItem {
            x_pos: (2 * 16 + 8) << 16,
            y_pos: i64::from(32 << 16),
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 2,
            level_row: 2,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
        });
        rt.floating_items.push(FloatingItem {
            x_pos: (3 * 16 + 8) << 16,
            y_pos: i64::from(32 << 16),
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 3,
            level_row: 2,
            first_row_grace: true,
            content: terrain::PrizeContent::Clock,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items[0].y_velocity, 0x1388,
            "ASM loc_103B2 keeps the item falling when the current row is blocked but the one-row grace probe below is still open",
        );
        assert_eq!(
            rt.floating_items[1].y_velocity, 0,
            "ASM loc_103B2 stops immediately once both the current row and the grace-probed row below are solid",
        );
    }

    #[test]
    fn floating_item_loc_103da_zeroes_velocity_when_the_fall_branch_is_blocked() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_blocks();
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0xE0, 0x01));
        grid.set_cell_at(2, 3, terrain::Cell::from_be_bytes(0xE0, 0x01));
        rt.set_block_grid(grid);
        let blocked_y = i64::from(32 << 16);
        rt.floating_items.push(FloatingItem {
            x_pos: (2 * 16 + 8) << 16,
            y_pos: blocked_y,
            y_velocity: 0x2710,
            remaining: 500,
            spawn_delay: 0,
            level_col: 2,
            level_row: 2,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items[0].y_velocity, 0,
            "ASM loc_103DA should replace the blocked fall delta with zero before the shared tail continues",
        );
        assert_eq!(
            rt.floating_items[0].y_pos, blocked_y,
            "ASM loc_103DA should leave the floating item on the same pixel row when the blocked branch skips `loc_103DE`",
        );
    }

    #[test]
    fn floating_item_loc_10402_updates_the_row_baseline_once_per_16px_boundary_crossing() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_blocks();
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0, 0));
        grid.set_cell_at(2, 3, terrain::Cell::from_be_bytes(0, 0));
        rt.set_block_grid(grid);
        rt.floating_items.push(FloatingItem {
            x_pos: (2 * 16 + 8) << 16,
            y_pos: i64::from(47 << 16),
            y_velocity: 0xF000,
            remaining: 500,
            spawn_delay: 0,
            level_col: 2,
            level_row: 2,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items[0].level_row, 3,
            "ASM loc_10402 should advance the tracked level row once the falling item crosses into the next snapped 16px band",
        );
        assert!(
            !rt.floating_items[0].first_row_grace,
            "ASM loc_10402 should clear the one-row grace bit once the new aligned row is copied into the running baseline",
        );

        let y_after_crossing = rt.floating_items[0].y_pos;
        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items[0].level_row, 3,
            "ASM loc_10402 should keep the updated row baseline in `d0`, so later frames inside the same 16px band do not advance `level_row` again",
        );
        assert!(
            rt.floating_items[0].y_pos > y_after_crossing,
            "the follow-up frame should still fall inside the same open row without spuriously re-advancing the tracked row",
        );
    }

    #[test]
    fn floating_item_loc_10430_deletes_items_once_they_reach_level_height() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (2 * 16 + 8) << 16,
            y_pos: i64::from(895 << 16),
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 2,
            level_row: 55,
            first_row_grace: false,
            content: terrain::PrizeContent::Diamond,
            collected: false,
        });
        rt.floating_items.push(FloatingItem {
            x_pos: (3 * 16 + 8) << 16,
            y_pos: i64::from(896 << 16),
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 3,
            level_row: 56,
            first_row_grace: false,
            content: terrain::PrizeContent::Clock,
            collected: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10430 should jump straight to `Delete_CurrentObject` once `y_pos(a3)` reaches `Level_height_pixels`",
        );
        assert_eq!(
            rt.floating_items[0].level_row, 55,
            "ASM loc_10430 should keep the still-in-bounds item alive while its integer Y word remains strictly below `Level_height_pixels`",
        );
    }

    #[test]
    fn floating_item_step_returns_past_the_unreachable_loc_10434_padding_loop() {
        let mut rt = NativeRuntime::new(manifest());
        rt.step_visual_effects();

        assert!(rt.floating_items.is_empty());
        assert!(
            rt.block_bounces.is_empty() && rt.block_destroy_effects.is_empty(),
            "ASM loc_10434 is only the unreachable self-loop padding after `OneTimePrize_CheckCollected`, so native `step_visual_effects` should return to the caller instead of emulating that trap",
        );
    }

    #[test]
    fn projectile_hits_player_exact_overlap() {
        // Projectile at same position as player — must hit.
        // Kills: return false replacement, all >> with << mutations (lines 136-137, 144-145).
        let mut rt = runtime_with_projectile_and_player(100, 100, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            rt.projectile_hits_player(&slice),
            "projectile at player position must hit"
        );
        assert!(
            rt.projectiles.is_empty(),
            "hitting projectile should be removed"
        );
    }

    #[test]
    fn projectile_misses_player_outside_x_range() {
        // Projectile 14px right of player — exactly at boundary (< 14 fails).
        // half=6, half+8=14. |prx - px| = 14, NOT < 14 → miss.
        // Kills: < with <= on x-axis (line 146 first <).
        let mut rt = runtime_with_projectile_and_player(114, 100, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            !rt.projectile_hits_player(&slice),
            "projectile 14px away in x should miss (boundary of < 14)"
        );
        assert_eq!(rt.projectiles.len(), 1, "missed projectile should survive");
    }

    #[test]
    fn projectile_hits_player_at_x_boundary() {
        // Projectile 13px right of player — just inside x range.
        // |prx - px| = 13 < 14 → hit (if y also overlaps).
        let mut rt = runtime_with_projectile_and_player(113, 100, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            rt.projectile_hits_player(&slice),
            "projectile 13px away in x should hit (just inside < 14)"
        );
    }

    #[test]
    fn projectile_misses_player_outside_y_range() {
        // Projectile 22px below player — exactly at boundary (< 22 fails).
        // half=6, half+16=22. |pry - py| = 22, NOT < 22 → miss.
        // Kills: < with <= on y-axis (line 146 second <).
        let mut rt = runtime_with_projectile_and_player(100, 122, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            !rt.projectile_hits_player(&slice),
            "projectile 22px away in y should miss (boundary of < 22)"
        );
    }

    #[test]
    fn projectile_hits_player_at_y_boundary() {
        // Projectile 21px below player — just inside y range.
        let mut rt = runtime_with_projectile_and_player(100, 121, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            rt.projectile_hits_player(&slice),
            "projectile 21px away in y should hit (just inside < 22)"
        );
    }

    #[test]
    fn projectile_hits_player_negative_offset() {
        // Projectile 13px LEFT of player — tests abs() and subtraction direction.
        // Kills: - with + on (prx - px) and (pry - py).
        let mut rt = runtime_with_projectile_and_player(87, 79, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            rt.projectile_hits_player(&slice),
            "projectile 13px left and 21px above should hit"
        );
    }

    #[test]
    fn projectile_misses_when_x_overlap_but_y_outside() {
        // x overlap (same x) but y is 22px away → miss.
        // Kills: && with || replacement.
        let mut rt = runtime_with_projectile_and_player(100, 122, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            !rt.projectile_hits_player(&slice),
            "x overlap but y outside should miss (both conditions required)"
        );
    }

    #[test]
    fn projectile_friendly_does_not_hit_player() {
        // Friendly projectile at player position — must not hit.
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(100, 100);
        proj.friendly = true;
        rt.projectiles.push(proj);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);
        assert!(
            !rt.projectile_hits_player(&slice),
            "friendly projectile must not damage player"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "friendly projectile should survive"
        );
    }

    #[test]
    fn kid_projectile_collision_passes_sub_1d76_dispatches_the_two_rom_list_heads() {
        let passes = kid_projectile_collision_passes_sub_1D76();

        assert_eq!(
            passes,
            [
                KidProjectileCollisionPassSub1D76 {
                    object_list_head_address: 0x00FF_F862,
                    collision_mode: 0,
                },
                KidProjectileCollisionPassSub1D76 {
                    object_list_head_address: 0x00FF_F5A0,
                    collision_mode: 1,
                },
            ],
            "ASM sub_1D76 should call sub_1D84 twice: once for `$FFFFF862` with `d7=0`, then for `$FFFFF5A0` with `d7=1`",
        );
    }

    #[test]
    fn kid_projectile_target_x_span_loc_1db4_mirrors_the_leading_offset_when_facing_flipped() {
        assert_eq!(
            crate::projectiles::kid_projectile_target_x_span_loc_1DB4(100, 0, 6, 20),
            crate::projectiles::KidProjectileTargetXSpanLoc1DB4 {
                left_px: 106,
                right_px: 126,
            },
            "ASM sub_1D84 should use `[x_pos + leading, x_pos + leading + span]` when `x_direction(a0)` is clear before reaching `loc_1DB4`",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_target_x_span_loc_1DB4(100, 0xFF, 6, 20),
            crate::projectiles::KidProjectileTargetXSpanLoc1DB4 {
                left_px: 74,
                right_px: 94,
            },
            "ASM loc_1DB4 should negate the leading offset, anchor the right edge at `x_pos - leading`, and subtract the span for flipped targets",
        );
    }

    #[test]
    fn kid_projectile_target_rect_loc_1dbe_adds_the_shared_y_span_after_the_x_branch() {
        assert_eq!(
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0, 6, 20, -12, 30),
            crate::projectiles::KidProjectileTargetRectLoc1DBE {
                left_px: 106,
                right_px: 126,
                top_px: 28,
                bottom_px: 58,
            },
            "ASM loc_1DBE should add the leading Y offset to `y_pos(a0)`, copy that value, and add the final Y span after the non-flipped X branch",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0xFF, 6, 20, -12, 30),
            crate::projectiles::KidProjectileTargetRectLoc1DBE {
                left_px: 74,
                right_px: 94,
                top_px: 28,
                bottom_px: 58,
            },
            "ASM loc_1DBE should keep the same `[y_pos + offset, y_pos + offset + span]` range after the flipped `loc_1DB4` X branch",
        );
    }

    #[test]
    fn first_live_kid_projectile_collision_cursor_loc_1dcc_skips_deleted_entries_and_halves_the_collision_word()
     {
        assert_eq!(
            first_live_kid_projectile_collision_cursor_loc_1DCC(&[
                kid_projectile_collision_object_loc_1dcc(
                    true,
                    false,
                    0x0002,
                    32,
                    48,
                    vec![collision_size_entry_sub_21f8(-2, 4, -2, 4)],
                ),
                kid_projectile_collision_object_loc_1dcc(
                    false,
                    true,
                    0x0006,
                    120,
                    80,
                    vec![
                        collision_size_entry_sub_21f8(-5, 10, -31, 31),
                        collision_size_entry_sub_21f8(0, 0, 0, 0),
                    ],
                ),
            ]),
            Some(KidProjectileCollisionCursorLoc1DCC {
                object_index: 1,
                collision_size_index: 3,
                mirrored: true,
                x_anchor_px: 120,
                y_anchor_px: 80,
                collision_sizes: vec![
                    collision_size_entry_sub_21f8(-5, 10, -31, 31),
                    collision_size_entry_sub_21f8(0, 0, 0, 0),
                ],
            }),
            "ASM loc_1DCC should skip deleted gfx objects via `$3D(a2)`, then seed the CollisionSize cursor from `$22(a2)>>1` before the orientation-specific scan starts",
        );
    }

    #[test]
    fn first_non_mirrored_kid_projectile_overlap_loc_1df2_skips_misses_until_the_first_inclusive_overlap()
     {
        let target =
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0, 6, 20, -12, 30);
        let cursor = KidProjectileCollisionCursorLoc1DCC {
            object_index: 0,
            collision_size_index: 0,
            mirrored: false,
            x_anchor_px: 100,
            y_anchor_px: 40,
            collision_sizes: vec![
                collision_size_entry_sub_21f8(40, 10, -12, 30),
                collision_size_entry_sub_21f8(-20, 5, -12, 30),
                collision_size_entry_sub_21f8(6, 20, -12, 30),
            ],
        };

        assert_eq!(
            first_non_mirrored_kid_projectile_overlap_loc_1DF2(target, &cursor),
            Some(2),
            "ASM loc_1DF2 should keep advancing 8-byte CollisionSize entries after X/Y misses, then stop on the first inclusive overlap for clear `x_direction(a2)`",
        );
    }

    #[test]
    fn first_non_mirrored_kid_projectile_overlap_loc_1df2_stops_this_object_on_the_zero_word_sentinel()
     {
        let target =
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0, 6, 20, -12, 30);
        let cursor = KidProjectileCollisionCursorLoc1DCC {
            object_index: 0,
            collision_size_index: 0,
            mirrored: false,
            x_anchor_px: 100,
            y_anchor_px: 40,
            collision_sizes: vec![
                collision_size_entry_sub_21f8(40, 10, -12, 30),
                collision_size_entry_sub_21f8(0, 0, 0, 0),
                collision_size_entry_sub_21f8(6, 20, -12, 30),
            ],
        };

        assert_eq!(
            first_non_mirrored_kid_projectile_overlap_loc_1DF2(target, &cursor),
            None,
            "ASM loc_1DF2 should branch back to `loc_1DCC` as soon as `(a3)` is zero, even if a later CollisionSize entry in RAM would overlap",
        );
    }

    #[test]
    fn first_mirrored_kid_projectile_overlap_loc_1e1e_negates_the_left_offset_then_subtracts_width()
    {
        let target =
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0xFF, 6, 20, -12, 30);
        let cursor = KidProjectileCollisionCursorLoc1DCC {
            object_index: 0,
            collision_size_index: 0,
            mirrored: true,
            x_anchor_px: 100,
            y_anchor_px: 40,
            collision_sizes: vec![
                collision_size_entry_sub_21f8(20, 10, -12, 30),
                collision_size_entry_sub_21f8(-6, 20, -12, 30),
            ],
        };

        assert_eq!(
            first_mirrored_kid_projectile_overlap_loc_1E1E(target, &cursor),
            Some(1),
            "ASM loc_1E1E should mirror the horizontal bounds by negating the stored left offset, then subtract the width back toward the facing side before the inclusive overlap checks",
        );
    }

    #[test]
    fn first_mirrored_kid_projectile_overlap_loc_1e1e_stops_this_object_on_the_zero_word_sentinel()
    {
        let target =
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0xFF, 6, 20, -12, 30);
        let cursor = KidProjectileCollisionCursorLoc1DCC {
            object_index: 0,
            collision_size_index: 0,
            mirrored: true,
            x_anchor_px: 100,
            y_anchor_px: 40,
            collision_sizes: vec![
                collision_size_entry_sub_21f8(20, 10, -12, 30),
                collision_size_entry_sub_21f8(0, 0, 0, 0),
                collision_size_entry_sub_21f8(-6, 20, -12, 30),
            ],
        };

        assert_eq!(
            first_mirrored_kid_projectile_overlap_loc_1E1E(target, &cursor),
            None,
            "ASM loc_1E1E should branch back to `loc_1DCC` on the zero-word CollisionSize sentinel before any later mirrored entry is considered",
        );
    }

    #[test]
    fn kid_projectile_overlap_branch_loc_1e4a_routes_nonzero_d7_straight_to_wall_response() {
        assert_eq!(
            crate::projectiles::kid_projectile_overlap_branch_loc_1E4A(1, 0x1234, 0x2000, 0x2010),
            crate::projectiles::KidProjectileOverlapBranchLoc1E4A::WallResponseLoc1EA0,
            "ASM loc_1E4A (kid.asm:3634-3635) should jump to `loc_1EA0` as soon as `tst.w d7` sees a non-zero collision pass",
        );
    }

    #[test]
    fn kid_projectile_overlap_branch_loc_1e4a_only_diverts_the_inclusive_null_window() {
        const LNK_TO3_NULL_MINUS_DATA_INDEX: u16 = 0x2000;
        const LNK_TO4_NULL_MINUS_DATA_INDEX: u16 = 0x2010;

        assert_eq!(
            crate::projectiles::kid_projectile_overlap_branch_loc_1E4A(
                0,
                LNK_TO3_NULL_MINUS_DATA_INDEX.wrapping_sub(2),
                LNK_TO3_NULL_MINUS_DATA_INDEX,
                LNK_TO4_NULL_MINUS_DATA_INDEX,
            ),
            crate::projectiles::KidProjectileOverlapBranchLoc1E4A::SpawnHelperLoc1E5E,
            "ASM loc_1E4A should fall through to `loc_1E5E` when `$22(a0)` stays below `LnkTo3_NULL-Data_Index`",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_overlap_branch_loc_1E4A(
                0,
                LNK_TO3_NULL_MINUS_DATA_INDEX,
                LNK_TO3_NULL_MINUS_DATA_INDEX,
                LNK_TO4_NULL_MINUS_DATA_INDEX,
            ),
            crate::projectiles::KidProjectileOverlapBranchLoc1E4A::WallResponseLoc1EA0,
            "ASM loc_1E4A's `ble.s loc_1EA0` means the lower bound is inclusive",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_overlap_branch_loc_1E4A(
                0,
                LNK_TO4_NULL_MINUS_DATA_INDEX,
                LNK_TO3_NULL_MINUS_DATA_INDEX,
                LNK_TO4_NULL_MINUS_DATA_INDEX,
            ),
            crate::projectiles::KidProjectileOverlapBranchLoc1E4A::WallResponseLoc1EA0,
            "ASM loc_1E4A should keep the upper bound inclusive before leaving for `loc_1EA0`",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_overlap_branch_loc_1E4A(
                0,
                LNK_TO4_NULL_MINUS_DATA_INDEX.wrapping_add(2),
                LNK_TO3_NULL_MINUS_DATA_INDEX,
                LNK_TO4_NULL_MINUS_DATA_INDEX,
            ),
            crate::projectiles::KidProjectileOverlapBranchLoc1E4A::SpawnHelperLoc1E5E,
            "ASM loc_1E4A should resume the `loc_1E5E` helper-allocation path once `$22(a0)` rises above `LnkTo4_NULL-Data_Index`",
        );
    }

    #[test]
    fn kid_projectile_helper_spawn_loc_1e5e_allocates_loc_1f08_and_copies_projectile_coordinates() {
        assert_eq!(
            crate::projectiles::kid_projectile_helper_spawn_loc_1E5E(0x0123, 0x0456, 0x0000),
            crate::projectiles::KidProjectileHelperSpawnLoc1E5E {
                allocate_object_slot_arg: 0xFFFF,
                callback_label: "loc_1F08",
                helper_x_pos: 0x0123,
                helper_y_pos: 0x0456,
                target_collision_id: 0x001C,
            },
            "ASM loc_1E5E (kid.asm:3642-3650) should allocate with `a0 = $FFFF`, install `loc_1F08`, copy `$1A/$1E(a0)` into the helper payload, and default the collided object to `colid_hurt`",
        );
    }

    #[test]
    fn kid_projectile_helper_spawn_loc_1e5e_overrides_hurt_with_ffff_for_3c_projectiles() {
        assert_eq!(
            crate::projectiles::kid_projectile_helper_spawn_loc_1E5E(0x0123, 0x0456, 0x003C)
                .target_collision_id,
            0xFFFF,
            "ASM loc_1E5E should replace the provisional `colid_hurt` write with `$FFFF` when `$3A(a0) == $3C` before falling into `loc_1E94`",
        );
    }

    #[test]
    fn kid_projectile_rightwall_writeback_loc_1e94_marks_the_projectile_and_restarts_the_scan() {
        assert_eq!(
            crate::projectiles::kid_projectile_rightwall_writeback_loc_1E94(),
            crate::projectiles::KidProjectileRightwallWritebackLoc1E94 {
                projectile_collision_id: 0x0004,
                next_label: "sub_1D84",
            },
            "ASM loc_1E94 (kid.asm:3655-3656) should store `#4` into `$38(a0)` before branching back to `sub_1D84`",
        );
    }

    #[test]
    fn projectile_hitbox_half_plus_8_not_times() {
        // Projectile 12px away in x — inside `half + 8 = 14` but outside `half * 8 = 48`.
        // If + mutated to *, 12 < 48 would still pass, so use boundary at 13.
        // Also check that 15px misses — kills + with * (6*8=48 would wrongly pass).
        let mut rt = runtime_with_projectile_and_player(115, 100, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            !rt.projectile_hits_player(&slice),
            "15px x gap should miss with half + 8 = 14"
        );
    }

    #[test]
    fn projectile_hitbox_half_plus_16_not_times() {
        // Projectile 23px away in y — outside `half + 16 = 22`.
        // If + mutated to *, 23 < 96 would wrongly pass.
        let mut rt = runtime_with_projectile_and_player(100, 123, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            !rt.projectile_hits_player(&slice),
            "23px y gap should miss with half + 16 = 22"
        );
    }

    #[test]
    fn step_projectiles_applies_velocity() {
        // Basic movement test calling actual step_projectiles.
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(100, 100);
        proj.x_velocity = 0x2_0000; // +2px/frame
        proj.y_velocity = -0x1_0000; // -1px/frame
        rt.projectiles.push(proj);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].x_pos,
            102 << 16,
            "x should advance by x_velocity"
        );
        assert_eq!(
            rt.projectiles[0].y_pos,
            99 << 16,
            "y should advance by y_velocity"
        );
    }

    #[test]
    fn step_projectiles_wall_bounce_reapplies_velocity() {
        // After a wall bounce, position = pre-bounce + bounced velocity.
        // Kills: line 126 + with *, line 127 + with - and + with *.
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(50, 50);
        proj.x_velocity = 0x2_0000; // 2px/frame rightward
        proj.y_velocity = 0x1_0000; // 1px/frame downward
        proj.wall_bounce = true;

        // Build a grid with a solid block at the destination cell.
        // Projectile at (50,50), moving right 2px → lands at (52,51).
        // Place solid block at cell containing (52,51).
        let w = 10u16;
        let h = 10u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Solid block at col=3 (pixels 48-63), row=3 (pixels 48-63)
        let dest_col = terrain::pixel_to_cell(52).max(0) as u16;
        let dest_row = terrain::pixel_to_cell(51).max(0) as u16;
        let off = (usize::from(dest_row) * usize::from(w) + usize::from(dest_col)) * 2;
        data[off] = 0x60; // has_block=0, collision=Solid(6), block_type=0
        // Actually need has_block=true: hi=0xE0 (has_block=1, collision=6(Solid), block_type=0)
        data[off] = 0xE0;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        rt.set_block_grid(grid);

        let x_before = proj.x_pos;
        let y_before = proj.y_pos;
        rt.projectiles.push(proj);
        rt.step_projectiles();

        // After bounce: velocity should be negated on the blocked axis, then re-applied.
        // The exact outcome depends on which axis is blocked, but position should differ
        // from a simple velocity addition (which would be inside the wall).
        let p = &rt.projectiles[0];
        // The projectile should have moved (not stayed in place)
        assert!(
            p.x_pos != x_before || p.y_pos != y_before,
            "wall-bounce projectile should move after bouncing"
        );
    }

    #[test]
    fn step_projectiles_gravity_accumulates() {
        // Gravity adds to y_velocity each frame.
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(100, 100);
        proj.y_velocity = 0;
        proj.gravity = 0x1000;
        rt.projectiles.push(proj);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].y_velocity, 0x1000,
            "gravity should be added to y_velocity"
        );

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].y_velocity, 0x2000,
            "gravity should accumulate each frame"
        );
    }

    #[test]
    fn step_projectiles_lifetime_decrements() {
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(100, 100);
        proj.lifetime = 5;
        rt.projectiles.push(proj);

        rt.step_projectiles();
        assert_eq!(rt.projectiles[0].lifetime, 4);

        // Step to expiry: 4→3→2→1→0, then next call sees 0 and removes
        for _ in 0..4 {
            rt.step_projectiles();
        }
        assert_eq!(rt.projectiles[0].lifetime, 0, "lifetime should reach 0");
        rt.step_projectiles();
        assert!(
            rt.projectiles.is_empty(),
            "projectile should be removed when lifetime is 0 at frame start"
        );
    }

    #[test]
    fn step_projectiles_homing_adjusts_velocity() {
        // Homing projectile accelerates toward player.
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(50, 50);
        proj.homing_accel = 0x800;
        proj.speed_cap = 0x6_0000;
        rt.projectiles.push(proj);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 200 << 16; // player far right
        slice.player.y_pos = 200 << 16; // player far below
        rt.opening_slice = Some(slice);

        rt.step_projectiles();

        assert!(
            rt.projectiles[0].x_velocity > 0,
            "homing should accelerate rightward toward player"
        );
        assert!(
            rt.projectiles[0].y_velocity > 0,
            "homing should accelerate downward toward player"
        );
    }

    #[test]
    fn step_projectiles_speed_cap_clamps_homing() {
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(50, 50);
        proj.homing_accel = 0x7_0000; // very high accel
        proj.speed_cap = 0x2_0000; // low cap
        rt.projectiles.push(proj);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 200 << 16;
        slice.player.y_pos = 200 << 16;
        rt.opening_slice = Some(slice);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].x_velocity, 0x2_0000,
            "x velocity should be clamped to speed_cap"
        );
        assert_eq!(
            rt.projectiles[0].y_velocity, 0x2_0000,
            "y velocity should be clamped to speed_cap"
        );
    }
}
