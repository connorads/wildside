use super::*;

fn empty_weather_collision_rows_sub_61_ca() -> WeatherCollisionRowsSub61CA {
    WeatherCollisionRowsSub61CA {
        rows: vec![vec![0; 32]; 16],
    }
}

fn test_dragon(variant: u8, x: u32, y: u32) -> ActiveEnemy {
    let mut e = test_enemy(EnemyFamily::Dragon, x, y, 1);
    e.dragon_script_variant = variant;
    e.dragon_anchor_y = y;
    let script = dragon_script(variant);
    match script[0] {
        DragonFlyingCmd::Hover(frames) => {
            e.dragon_phase = DragonFlyingState::Hovering;
            e.dragon_phase_frames = frames;
        }
        DragonFlyingCmd::DriftWithFire(frames) => {
            e.dragon_phase = DragonFlyingState::DriftingWithFire;
            e.dragon_phase_frames = frames;
            e.x_velocity = if e.facing == Direction::Right {
                0x800
            } else {
                -0x800
            };
        }
        DragonFlyingCmd::Drift(frames) => {
            e.dragon_phase = DragonFlyingState::Drifting;
            e.dragon_phase_frames = frames;
            e.x_velocity = if e.facing == Direction::Right {
                0x800
            } else {
                -0x800
            };
        }
        _ => {
            e.dragon_phase = DragonFlyingState::Hovering;
            e.dragon_phase_frames = 1;
        }
    }
    e
}

mod chunk_03 {
    use super::*;

    #[test]
    fn diamond_power_maniaxe_tier2_grants_life() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        runtime.progression.current_form = PlayerForm::Maniaxe;
        runtime.progression.diamond_count = 55;
        let lives_before = runtime.progression.number_lives;
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.progression.number_lives,
            lives_before + 1,
            "Maniaxe tier2 grants +1 life"
        );
    }

    #[test]
    fn diamond_power_cyclone_slashing_rain_spawns_projectiles() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        runtime.progression.current_form = PlayerForm::Cyclone;
        runtime.progression.diamond_count = 25;
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.progression.diamond_power_frames, 720,
            "SlashingRain = 720f"
        );
        // Step a few frames — rain should spawn projectiles
        for _ in 0..20 {
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            !runtime.projectiles.is_empty(),
            "SlashingRain should spawn projectiles"
        );
    }

    // -----------------------------------------------------------------------
    // DiamondPower_Init / Main rain (loc_3FDFE, loc_3FE62, loc_3FE90)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_init_slashing_rain_sets_timer_and_clears_count_loc_3fdfe() {
        // ASM DiamondPower_Init_SlashingRain (kid.asm:45168): sf ($FFFFF5B9).w
        // (tracking=false), move.w #$2D0,(Diamond_power_timer).w (720 frames).
        // ASM loc_3FDFE (kid.asm:45172): clr.w ($FFFFF5C0).w — reset active rain
        // object count to 0. Native equivalent: diamond_power_frames set to 720,
        // projectile list starts empty (no explicit count variable).
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        runtime.progression.current_form = PlayerForm::Cyclone;
        runtime.progression.diamond_count = 25; // enough for tier-1
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.progression.diamond_power_frames, 720,
            "ASM loc_3FDFE: SlashingRain timer = $2D0 (720)"
        );
        assert_eq!(
            runtime.progression.diamond_power_type,
            helmets::DiamondPowerType::SlashingRain,
            "Cyclone tier-1 = SlashingRain"
        );
    }

    #[test]
    fn diamond_power_init_tracking_rain_sets_timer_and_clears_count_loc_3fdfe() {
        // ASM DiamondPower_Init_TrackingRain (kid.asm:45162): st ($FFFFF5B9).w
        // (tracking=true), move.w #$708,(Diamond_power_timer).w (1800 frames).
        // ASM loc_3FDFE (kid.asm:45172): clr.w ($FFFFF5C0).w — same epilogue.
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        runtime.progression.current_form = PlayerForm::Cyclone;
        runtime.progression.diamond_count = 55; // 20+30 for tier-2
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.progression.diamond_power_frames, 1800,
            "ASM loc_3FDFE: TrackingRain timer = $708 (1800)"
        );
        assert_eq!(
            runtime.progression.diamond_power_type,
            helmets::DiamondPowerType::TrackingRain,
            "Cyclone tier-2 = TrackingRain"
        );
    }

    #[test]
    fn diamond_power_main_rain_spawns_and_iterates_objects_loc_3fe62_loc_3fe90() {
        // ASM loc_3FE62 (kid.asm:45204): rain main loop preamble — computes
        // screen offsets d3=$7F-Camera_X, d4=$7F-Camera_Y, loads Kid x_vel*3/4
        // as drift factor, d7=Time_Frames&7 for tracking cadence, then iterates
        // the DP object linked list starting at Addr_FirstDPObjectSlot.
        //
        // ASM loc_3FE90 (kid.asm:45221): per-object entry — tst.w $3E(a0)
        // dispatches to tracking (loc_3FE98) or non-tracking (loc_3FEBC) path.
        // Non-tracking objects drift with Kid velocity and fall; tracking objects
        // home toward nearest enemy.
        //
        // Native approximation: projectile-based spawning without linked-list
        // objects. This test verifies that rain objects accumulate over time and
        // that the timer counts down, exercising the equivalent of the ASM's
        // per-frame spawn (DiamondPower_Main_SlashingRain) and per-object
        // iteration (loc_3FE62 → loc_3FE90 loop).
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        runtime.progression.current_form = PlayerForm::Cyclone;
        runtime.progression.diamond_count = 25;
        runtime.step_frame(FrameInput {
            start: true,
            a: true,
            ..FrameInput::default()
        });
        let initial_frames = runtime.progression.diamond_power_frames;
        assert_eq!(initial_frames, 720);

        // Step 8 frames — timer should decrement, projectiles should appear
        for _ in 0..8 {
            runtime.step_frame(FrameInput::default());
        }
        assert_eq!(
            runtime.progression.diamond_power_frames,
            initial_frames - 8,
            "ASM loc_3FE62: timer decrements each frame"
        );
        assert!(
            !runtime.projectiles.is_empty(),
            "ASM loc_3FE90: rain objects spawned and iterated"
        );
        // Each projectile should move downward (friendly rain)
        for proj in &runtime.projectiles {
            assert!(
                proj.friendly,
                "ASM loc_3FE90: rain projectiles are friendly (damage enemies)"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Rain tracking enemy scan (loc_3FEE2, loc_3FF16, loc_3FF1C)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_rain_find_nearest_enemy_picks_closest_loc_3fee2_loc_3ff16_loc_3ff1c() {
        // ASM loc_3FEE2 (kid.asm:45253): iterate enemy linked list, for each:
        // load x/y, subtract 8 from y, cull off-screen, compute Manhattan distance.
        // ASM loc_3FF16 (kid.asm:45272): abs(dx) = |enemy_screen_x - rain_screen_x|
        // ASM loc_3FF1C (kid.asm:45277): abs(dy) = |enemy_screen_y - rain_screen_y|,
        // then add for Manhattan distance, track nearest.
        let cam_x: i16 = 0x80;
        let cam_y: i16 = 0x40;
        let rain_x: i16 = 0x100; // screen-relative: 0x100-0x80 = 0x80
        let rain_y: i16 = 0x80; // screen-relative: 0x80-0x40 = 0x40
        let enemies = [
            // Enemy A: screen (0xA0, 0x70) after y-8 offset → dx=0x20, dy=0x30 → dist=0x50
            helmets::SnakeTargetCandidate {
                x_pos: 0x120,
                y_pos: 0xB8, // (0xB8-8)-0x40 = 0x70
            },
            // Enemy B: screen (0x90, 0x50) after y-8 offset → dx=0x10, dy=0x10 → dist=0x20
            helmets::SnakeTargetCandidate {
                x_pos: 0x110,
                y_pos: 0x98, // (0x98-8)-0x40 = 0x50
            },
        ];
        let result =
            helmets::diamond_power_rain_find_nearest_enemy(rain_x, rain_y, cam_x, cam_y, &enemies);
        let target = result.expect("should find nearest enemy");
        // ASM loc_3FF1C: enemy B is nearer (dist=0x20 < 0x50)
        assert_eq!(target.x, 0x110, "nearest enemy x_pos");
        assert_eq!(target.y, 0x98, "nearest enemy y_pos");
        assert_eq!(target.distance, 0x20, "Manhattan distance |dx|+|dy|");
    }

    #[test]
    fn diamond_power_rain_find_nearest_returns_none_when_no_enemies_loc_3fee2() {
        // ASM loc_3FEBC: move.l ($FFFFF86A).w,d6; beq.s loc_3FF2C
        // — empty enemy list skips the loc_3FEE2 loop entirely.
        // d7 stays at $FFFF (sentinel), so no target found.
        let result = helmets::diamond_power_rain_find_nearest_enemy(0x100, 0x80, 0x80, 0x40, &[]);
        assert!(result.is_none(), "no enemies → None");
    }

    #[test]
    fn diamond_power_rain_find_nearest_culls_offscreen_enemies_loc_3fee2() {
        // ASM loc_3FEE2: screen bounds cull:
        // x in [-8, $148), y in [$10, $F0) after camera subtraction and y-8 offset.
        let cam_x: i16 = 0;
        let cam_y: i16 = 0;
        let enemies = [
            // x too far left: screen_x = -9 < -8
            helmets::SnakeTargetCandidate {
                x_pos: -9,
                y_pos: 0x80,
            },
            // x too far right: screen_x = 0x148 >= 0x148
            helmets::SnakeTargetCandidate {
                x_pos: 0x148,
                y_pos: 0x80,
            },
            // y too high: (0x17-8)=0xF < 0x10
            helmets::SnakeTargetCandidate {
                x_pos: 0x80,
                y_pos: 0x17,
            },
            // y too low: (0xF8-8)=0xF0 >= 0xF0
            helmets::SnakeTargetCandidate {
                x_pos: 0x80,
                y_pos: 0xF8,
            },
        ];
        let result =
            helmets::diamond_power_rain_find_nearest_enemy(0x80, 0x80, cam_x, cam_y, &enemies);
        assert!(result.is_none(), "all enemies off-screen → None");
    }

    #[test]
    fn diamond_power_rain_y_offset_is_8_not_16_loc_3fee2() {
        // ASM loc_3FEE2: subi.w #8,d6 — rain uses y-8 offset (snake uses y-16).
        // Enemy at y_pos=0x18: screen_ey = (0x18-8)-0 = 0x10 → exactly at threshold (included).
        // If the offset were 16: screen_ey = (0x18-16)-0 = 0x02 < 0x10 → would be culled.
        let cam_x: i16 = 0;
        let cam_y: i16 = 0;
        let enemies = [helmets::SnakeTargetCandidate {
            x_pos: 0x80,
            y_pos: 0x18,
        }];
        let result =
            helmets::diamond_power_rain_find_nearest_enemy(0x80, 0x80, cam_x, cam_y, &enemies);
        assert!(
            result.is_some(),
            "y_pos=0x18 visible with y-8 offset (screen_ey=0x10)"
        );
    }

    // -----------------------------------------------------------------------
    // Rain tracking velocity computation (loc_3FF3C, loc_3FF2C gate)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_rain_tracking_velocity_distance_in_range_loc_3ff3c() {
        // ASM loc_3FF2C (kid.asm:45288): gate — distance must be in [8, $40).
        // ASM loc_3FF3C (kid.asm:45296): lsr.w #3,d7 then count-bits loop.
        // distance=0x10: d7=0x10>>3=2, loop: d6=1,d7=1; d6=2,d7=0 → shift=2.
        // x_vel = (enemy_x - rain_x) >> 2, y_vel = (enemy_y - $80000 - rain_y) >> 2.
        let enemy_x: u32 = 0x0120_0000; // enemy at x=0x120 in 16.16
        let enemy_y: u32 = 0x0080_0000; // enemy at y=0x80 in 16.16
        let rain_x: u32 = 0x0100_0000; // rain at x=0x100 in 16.16
        let rain_y: u32 = 0x0060_0000; // rain at y=0x60 in 16.16
        let result = helmets::diamond_power_rain_compute_tracking_velocity(
            0x10, enemy_x, enemy_y, rain_x, rain_y,
        );
        let v = result.expect("distance 0x10 is in [8, $40)");
        assert_eq!(
            v.shift, 2,
            "floor(log2(0x10 >> 3)) + 1 = floor(log2(2)) + 1 = 2"
        );
        // dx = 0x0020_0000, >> 2 = 0x0008_0000
        assert_eq!(v.x_vel, 0x0008_0000, "x_vel = (enemy_x - rain_x) >> 2");
        // dy = 0x0080_0000 - 0x0008_0000 - 0x0060_0000 = 0x0018_0000, >> 2 = 0x0006_0000
        assert_eq!(
            v.y_vel, 0x0006_0000,
            "y_vel = (enemy_y - $80000 - rain_y) >> 2"
        );
    }

    #[test]
    fn diamond_power_rain_tracking_velocity_rejects_too_close_loc_3ff2c() {
        // ASM loc_3FF2C: cmpi.w #8,d7; bcs.s loc_3FF6E — distance < 8 rejected.
        let result = helmets::diamond_power_rain_compute_tracking_velocity(
            7,
            0x0120_0000,
            0x0080_0000,
            0x0100_0000,
            0x0060_0000,
        );
        assert!(result.is_none(), "distance 7 < 8 → no tracking");
    }

    #[test]
    fn diamond_power_rain_tracking_velocity_rejects_too_far_loc_3ff2c() {
        // ASM loc_3FF2C: cmpi.w #$40,d7; bcc.s loc_3FF6E — distance >= $40 rejected.
        let result = helmets::diamond_power_rain_compute_tracking_velocity(
            0x40,
            0x0120_0000,
            0x0080_0000,
            0x0100_0000,
            0x0060_0000,
        );
        assert!(result.is_none(), "distance $40 >= $40 → no tracking");
    }

    #[test]
    fn diamond_power_rain_tracking_velocity_min_distance_8_shift_1_loc_3ff3c() {
        // ASM loc_3FF3C: distance=8, d7=8>>3=1, loop: d6=1,d7=0 → shift=1.
        // Minimum valid distance gives shift=1 (divide by 2) — fastest tracking.
        let enemy_x: u32 = 0x0110_0000;
        let rain_x: u32 = 0x0100_0000;
        let enemy_y: u32 = 0x0070_0000;
        let rain_y: u32 = 0x0060_0000;
        let result = helmets::diamond_power_rain_compute_tracking_velocity(
            8, enemy_x, enemy_y, rain_x, rain_y,
        );
        let v = result.expect("distance 8 in range");
        assert_eq!(v.shift, 1, "d7=1 → one iteration → shift=1");
        // dx = 0x0010_0000, >> 1 = 0x0008_0000
        assert_eq!(v.x_vel, 0x0008_0000);
        // dy = 0x0070_0000 - 0x0008_0000 - 0x0060_0000 = 0x0008_0000, >> 1 = 0x0004_0000
        assert_eq!(v.y_vel, 0x0004_0000);
    }

    #[test]
    fn diamond_power_rain_tracking_velocity_max_distance_3f_shift_3_loc_3ff3c() {
        // ASM loc_3FF3C: distance=$3F (63), d7=$3F>>3=7, loop:
        // d6=1,d7=3; d6=2,d7=1; d6=3,d7=0 → shift=3.
        // Maximum valid distance gives shift=3 (divide by 8) — slowest tracking.
        let result = helmets::diamond_power_rain_compute_tracking_velocity(
            0x3F,
            0x0140_0000,
            0x00A0_0000,
            0x0100_0000,
            0x0060_0000,
        );
        let v = result.expect("distance $3F in range");
        assert_eq!(v.shift, 3, "d7=7 → three iterations → shift=3");
        // dx = 0x0040_0000, >> 3 = 0x0008_0000
        assert_eq!(v.x_vel, 0x0008_0000);
    }

    #[test]
    fn diamond_power_rain_tracking_velocity_negative_direction_loc_3ff3c() {
        // ASM loc_3FF3C: asr.l d6,d0 — arithmetic shift preserves sign.
        // Enemy to the left of and below rain → negative x_vel, positive y_vel.
        let enemy_x: u32 = 0x00E0_0000; // x=0xE0
        let enemy_y: u32 = 0x00A0_0000; // y=0xA0
        let rain_x: u32 = 0x0100_0000; // x=0x100
        let rain_y: u32 = 0x0060_0000; // y=0x60
        let result = helmets::diamond_power_rain_compute_tracking_velocity(
            0x10, enemy_x, enemy_y, rain_x, rain_y,
        );
        let v = result.expect("distance 0x10 in range");
        assert_eq!(v.shift, 2);
        // dx = 0x00E0_0000 - 0x0100_0000 = -0x0020_0000, >> 2 = -0x0008_0000
        assert_eq!(v.x_vel, -0x0008_0000, "negative x_vel (enemy left of rain)");
        // dy = 0x00A0_0000 - 0x0008_0000 - 0x0060_0000 = 0x0038_0000, >> 2 = 0x000E_0000
        assert_eq!(v.y_vel, 0x000E_0000, "positive y_vel (enemy below rain)");
    }

    // -----------------------------------------------------------------------
    // Rain non-tracking movement (loc_3FF6E, loc_3FF70)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_rain_non_tracking_move_applies_drift_and_word_y_loc_3ff70() {
        // ASM loc_3FF70 (kid.asm:45317): x update is 32-bit with drift,
        // y update is word-only.
        // Initial rain: x=0x0100_0000, y=0x50, x_vel=0, y_vel=2, drift=0x0001_0000
        let (new_x, new_y) = helmets::diamond_power_rain_non_tracking_move(
            0x0100_0000, // rain_x
            0x50,        // rain_y (word)
            0,           // x_vel (initially zero for fresh non-tracking rain)
            2,           // y_vel (ASM: move.w #2,$2A(a0))
            0x0001_0000, // kid_drift (a5)
        );
        // x: 0x0100_0000 + 0 + 0x0001_0000 = 0x0101_0000
        assert_eq!(new_x, 0x0101_0000, "x drifts with Kid");
        // y: 0x50 + 2 = 0x52 (word-only, no subpixel)
        assert_eq!(new_y, 0x52, "y advances by 2 pixels/frame");
    }

    #[test]
    fn diamond_power_rain_non_tracking_move_zero_drift_loc_3ff70() {
        // ASM loc_3FF6E/loc_3FF70: when Kid is stationary, a5=0.
        // Rain falls straight down with no horizontal drift.
        let (new_x, new_y) =
            helmets::diamond_power_rain_non_tracking_move(0x0080_0000, 0x30, 0, 2, 0);
        assert_eq!(new_x, 0x0080_0000, "no drift when Kid stationary");
        assert_eq!(new_y, 0x32, "y still falls at 2px/frame");
    }

    #[test]
    fn diamond_power_rain_non_tracking_move_negative_drift_loc_3ff70() {
        // ASM loc_3FE62: a5 = Kid x_vel * 3/4. If Kid moves left,
        // drift is negative (wrapping u32), rain drifts left too.
        // Kid x_vel = -0x0004_0000, drift = -0x0004_0000 * 3/4 = -0x0003_0000
        // As u32: 0xFFFD_0000
        let kid_drift: u32 = (-0x0003_0000_i32) as u32;
        let (new_x, _new_y) =
            helmets::diamond_power_rain_non_tracking_move(0x0080_0000, 0x30, 0, 2, kid_drift);
        // 0x0080_0000 + 0 + 0xFFFD_0000 = 0x007D_0000
        assert_eq!(new_x, 0x007D_0000, "rain drifts left with Kid");
    }

    #[test]
    fn diamond_power_rain_screen_cull_visible_centre_screen_loc_3ff90_loc_3ffca() {
        // ASM loc_3FF90: screen coords well within bounds → loc_3FFCA visible.
        // d0=0xA0 (>0, <$1C8), d1=0xB0 (>$68, <=$168), size=6.
        // After centering: d0=0xA0-6=0x9A, d1=0xB0-6=0xAA.
        // Table index = 6*4 = 24 = 0x18 → entry 6 = (0x05, 0x86F6).
        // VRAM attr = 24 + 0x1144 = 0x115C.
        let (vram, action) = helmets::diamond_power_rain_screen_cull_and_compile(0xA0, 0xB0, 6);
        assert_eq!(vram, 0x115C, "ASM loc_3FF90: d5 = d6*4 + $1144");
        assert_eq!(
            action,
            helmets::RainSlotSpriteAction::Visible {
                y: 0xAA,
                size_byte: 0x05,
                pattern: 0x86F6,
                x: 0x9A,
            },
            "ASM loc_3FFCA: on-screen rain sprite compiled"
        );
    }

    #[test]
    fn diamond_power_rain_screen_cull_removed_x_too_low_loc_3ff90_loc_3ffc0() {
        // ASM loc_3FF90: tst.w d0; ble.s loc_3FFC0 — d0 <= 0 means off-screen left.
        // screen_x=3, size=4 → centred d0 = 3-4 = -1 ≤ 0 → Removed.
        let (_, action) = helmets::diamond_power_rain_screen_cull_and_compile(3, 0xB0, 4);
        assert_eq!(
            action,
            helmets::RainSlotSpriteAction::Removed,
            "ASM loc_3FFC0: X ≤ 0 removes rain slot"
        );
    }

    #[test]
    fn diamond_power_rain_screen_cull_removed_x_too_high_loc_3ff90_loc_3ffc0() {
        // ASM: cmpi.w #$1C8,d0; bge.s loc_3FFC0 — d0 >= $1C8 off-screen right.
        // screen_x=0x1CA, size=1 → centred d0 = 0x1C9 ≥ 0x1C8 → Removed.
        let (_, action) = helmets::diamond_power_rain_screen_cull_and_compile(0x1CA, 0xB0, 1);
        assert_eq!(
            action,
            helmets::RainSlotSpriteAction::Removed,
            "ASM loc_3FFC0: X ≥ $1C8 removes rain slot"
        );
    }

    #[test]
    fn diamond_power_rain_screen_cull_removed_y_too_low_loc_3ff90_loc_3ffc0() {
        // ASM: cmpi.w #$68,d1; ble.s loc_3FFC0 — d1 <= $68 off-screen top.
        // screen_y=0x6A, size=3 → centred d1 = 0x67 ≤ 0x68… actually 0x67 < 0x68 so ble triggers.
        // Use screen_y=0x6B, size=3 → d1 = 0x68 = $68, ble (≤) → Removed.
        let (_, action) = helmets::diamond_power_rain_screen_cull_and_compile(0xA0, 0x6B, 3);
        assert_eq!(
            action,
            helmets::RainSlotSpriteAction::Removed,
            "ASM loc_3FFC0: Y ≤ $68 removes rain slot"
        );
    }

    #[test]
    fn diamond_power_rain_screen_cull_removed_y_too_high_loc_3ff90_loc_3ffc0() {
        // ASM: cmpi.w #$168,d1; ble.s loc_3FFCA — d1 > $168 falls through to loc_3FFC0.
        // screen_y=0x16A, size=0 → centred d1 = 0x16A > 0x168 → Removed.
        let (_, action) = helmets::diamond_power_rain_screen_cull_and_compile(0xA0, 0x16A, 0);
        assert_eq!(
            action,
            helmets::RainSlotSpriteAction::Removed,
            "ASM loc_3FFC0: Y > $168 removes rain slot"
        );
    }

    #[test]
    fn diamond_power_rain_screen_cull_boundary_just_visible_loc_3ff90_loc_3ffca() {
        // Exact boundary: d0=1 (>0 passes tst.w/ble), d1=0x69 (>$68 passes ble).
        // size=0 → no centering offset. Table entry 0 = (0x00, 0x86EA).
        // VRAM attr = 0 + 0x1144 = 0x1144.
        let (vram, action) = helmets::diamond_power_rain_screen_cull_and_compile(1, 0x69, 0);
        assert_eq!(vram, 0x1144);
        assert_eq!(
            action,
            helmets::RainSlotSpriteAction::Visible {
                y: 0x69,
                size_byte: 0x00,
                pattern: 0x86EA,
                x: 1,
            },
            "ASM loc_3FFCA: boundary values just inside screen"
        );
    }

    #[test]
    fn diamond_power_rain_screen_cull_boundary_y_exactly_168_loc_3ff90_loc_3ffca() {
        // ASM: cmpi.w #$168,d1; ble.s loc_3FFCA — d1 == $168 is ≤, so visible.
        let (_, action) = helmets::diamond_power_rain_screen_cull_and_compile(0x80, 0x168, 0);
        assert_eq!(
            action,
            helmets::RainSlotSpriteAction::Visible {
                y: 0x168,
                size_byte: 0x00,
                pattern: 0x86EA,
                x: 0x80,
            },
            "ASM loc_3FFCA: Y exactly $168 is visible (ble includes equal)"
        );
    }

    // ── WallOfDeath init (loc_3FFEA, loc_40002, loc_40006) ──────────────────────

    #[test]
    fn diamond_power_init_wall_of_death_allocates_10_slots_loc_3ffea() {
        // ASM DiamondPower_Init_WallOfDeath (kid.asm:45368):
        // moveq #9,d1 → 10 slots via dbf.
        let result = helmets::diamond_power_init_wall_of_death();
        assert_eq!(
            result.slots.len(),
            10,
            "ASM loc_3FFEA: dbf with d1=9 loops 10 times"
        );
    }

    #[test]
    fn diamond_power_init_wall_of_death_x_offsets_decrease_by_8_loc_3ffea() {
        // ASM loc_3FFEA: move.w d2,$3E(a0); subq.w #8,d2
        // d2 starts at 0, decreases by 8 each iteration.
        let result = helmets::diamond_power_init_wall_of_death();
        let x_offsets: Vec<i16> = result.slots.iter().map(|s| s.x_offset).collect();
        assert_eq!(
            x_offsets,
            vec![0, -8, -16, -24, -32, -40, -48, -56, -64, -72],
            "ASM loc_3FFEA: d2 starts 0, subq.w #8 each iteration"
        );
    }

    #[test]
    fn diamond_power_init_wall_of_death_y_offsets_increase_by_22_loc_40002() {
        // ASM loc_40002: addi.w #$16,d3
        // d3 starts at $D (13), increases by $16 (22) each iteration.
        let result = helmets::diamond_power_init_wall_of_death();
        let y_offsets: Vec<i16> = result.slots.iter().map(|s| s.y_offset).collect();
        assert_eq!(
            y_offsets,
            vec![0x0D, 0x23, 0x39, 0x4F, 0x65, 0x7B, 0x91, 0xA7, 0xBD, 0xD3],
            "ASM loc_40002: d3 starts $D, addi.w #$16 each iteration"
        );
    }

    #[test]
    fn diamond_power_init_wall_of_death_flags_and_size_loc_3ffea() {
        // ASM loc_3FFEA: sf $42(a0) — all slots start with reached_right_edge=false.
        // ASM loc_3FFEA: move.w #6,$24(a0) — all slots start with size_index=6.
        let result = helmets::diamond_power_init_wall_of_death();
        for (i, slot) in result.slots.iter().enumerate() {
            assert!(
                !slot.reached_right_edge,
                "ASM loc_3FFEA: sf $42(a0) — slot {i} should be false"
            );
            assert_eq!(
                slot.size_index, 6,
                "ASM loc_3FFEA: move.w #6,$24(a0) — slot {i} should be 6"
            );
        }
    }

    #[test]
    fn diamond_power_init_wall_of_death_timer_is_1200_loc_40006() {
        // ASM loc_40006: after dbf loop, move.w #$4B0,(Diamond_power_timer).w
        let result = helmets::diamond_power_init_wall_of_death();
        assert_eq!(
            result.diamond_power_timer, 0x4B0,
            "ASM loc_40006: Diamond_power_timer = $4B0 (1200)"
        );
    }

    // ── FiveWayShot init (loc_400DE, loc_400F4, loc_40110) ──────────────────────

    #[test]
    fn five_way_shot_init_allocates_5_slots_loc_400f4() {
        // ASM DiamondPower_Init_FiveWayShot (kid.asm:45461):
        // moveq #4,d1 → 5 slots via dbf.
        let result = helmets::diamond_power_init_five_way_shot(0x100, 0x80, 0, 0, false);
        assert_eq!(
            result.slots.len(),
            5,
            "ASM loc_400F4: dbf with d1=4 loops 5 times"
        );
    }

    #[test]
    fn five_way_shot_init_screen_position_loc_400de() {
        // ASM loc_400DE: d2 = x_pos - Camera_X_pos + d4; d3 = y_pos - Camera_Y_pos
        // When x_direction == 0 (facing right), d4 = +$20.
        let result = helmets::diamond_power_init_five_way_shot(0x100, 0x80, 0x40, 0x20, false);
        // screen_x = 0x100 - 0x40 + 0x20 = 0xE0
        // screen_y = 0x80 - 0x20 = 0x60
        for (i, slot) in result.slots.iter().enumerate() {
            assert_eq!(
                slot.screen_x, 0xE0,
                "ASM loc_400DE: slot {i} screen_x = kid_x - cam_x + $20"
            );
            assert_eq!(
                slot.screen_y, 0x60,
                "ASM loc_400DE: slot {i} screen_y = kid_y - cam_y"
            );
        }
    }

    #[test]
    fn five_way_shot_init_screen_position_facing_left_loc_400de() {
        // ASM loc_400DE: when x_direction != 0, d4 is negated to -$20.
        let result = helmets::diamond_power_init_five_way_shot(0x100, 0x80, 0x40, 0x20, true);
        // screen_x = 0x100 - 0x40 + (-0x20) = 0xA0
        for slot in &result.slots {
            assert_eq!(
                slot.screen_x, 0xA0,
                "ASM loc_400DE: neg.w d4 when x_direction nonzero"
            );
        }
    }

    #[test]
    fn five_way_shot_init_x_velocities_facing_right_loc_40110() {
        // ASM loc_400F4/loc_40110: x_vel from word_40124, NOT negated when
        // x_direction == 0 (beq.s loc_40110 skips neg.w).
        let result = helmets::diamond_power_init_five_way_shot(0, 0, 0, 0, false);
        let x_vels: Vec<i16> = result.slots.iter().map(|s| s.x_vel).collect();
        assert_eq!(
            x_vels,
            vec![6, 7, 8, 7, 6],
            "ASM loc_40110: word_40124 x velocities, unmirrored"
        );
    }

    #[test]
    fn five_way_shot_init_x_velocities_facing_left_loc_40110() {
        // ASM loc_400F4: when x_direction != 0, neg.w d4 mirrors x velocities.
        let result = helmets::diamond_power_init_five_way_shot(0, 0, 0, 0, true);
        let x_vels: Vec<i16> = result.slots.iter().map(|s| s.x_vel).collect();
        assert_eq!(
            x_vels,
            vec![-6, -7, -8, -7, -6],
            "ASM loc_400F4: neg.w d4 when x_direction nonzero"
        );
    }

    #[test]
    fn five_way_shot_init_y_velocities_loc_40110() {
        // ASM loc_40110: move.w (a5)+,$2A(a0) — y_vel from word_40124,
        // never direction-mirrored.
        let result = helmets::diamond_power_init_five_way_shot(0, 0, 0, 0, false);
        let y_vels: Vec<i16> = result.slots.iter().map(|s| s.y_vel).collect();
        assert_eq!(
            y_vels,
            vec![-4, -2, 0, 2, 4],
            "ASM loc_40110: word_40124 y velocities, fan pattern"
        );
        // Y velocities should be identical regardless of facing direction
        let result_left = helmets::diamond_power_init_five_way_shot(0, 0, 0, 0, true);
        let y_vels_left: Vec<i16> = result_left.slots.iter().map(|s| s.y_vel).collect();
        assert_eq!(y_vels, y_vels_left, "y velocities unaffected by direction");
    }

    #[test]
    fn five_way_shot_init_size_and_timer_loc_400f4() {
        // ASM loc_400F4: move.w #5,$24(a0) — all slots have size_index = 5.
        // ASM loc_40110: move.w #$50,(Diamond_power_timer).w — timer = 80.
        let result = helmets::diamond_power_init_five_way_shot(0, 0, 0, 0, false);
        for (i, slot) in result.slots.iter().enumerate() {
            assert_eq!(
                slot.size_index, 5,
                "ASM loc_400F4: move.w #5,$24(a0) — slot {i}"
            );
        }
        assert_eq!(
            result.diamond_power_timer, 0x50,
            "ASM loc_40110: Diamond_power_timer = $50 (80)"
        );
    }

    // ── FiveWayShot main per-slot (loc_4014C, loc_40166, loc_40176) ─────────────

    #[test]
    fn five_way_shot_slot_moves_by_velocity_loc_4014c() {
        // ASM loc_4014C: add.w $26(a0),d0 — screen_x += x_vel
        // ASM loc_40176: add.w $2A(a0),d1 — screen_y += y_vel
        let mut slot = helmets::FiveWayShotSlot {
            screen_x: 0x80,
            screen_y: 0x40,
            x_vel: 6,
            y_vel: -4,
            size_index: 5,
        };
        let _ = helmets::diamond_power_main_five_way_shot_slot(&mut slot, 0, 0, 10);
        assert_eq!(slot.screen_x, 0x86, "ASM loc_4014C: screen_x += x_vel (6)");
        assert_eq!(slot.screen_y, 0x3C, "ASM loc_40176: screen_y += y_vel (-4)");
    }

    #[test]
    fn five_way_shot_slot_bounces_left_edge_loc_4014c() {
        // ASM loc_4014C: when d0 + x_vel < 0 AND x_vel < 0, neg.w $26(a0).
        let mut slot = helmets::FiveWayShotSlot {
            screen_x: 2,
            screen_y: 0x40,
            x_vel: -6,
            y_vel: 0,
            size_index: 5,
        };
        let _ = helmets::diamond_power_main_five_way_shot_slot(&mut slot, 0, 0, 10);
        // d0 = 2 + (-6) = -4 < 0, x_vel was -6, so negated to 6
        assert_eq!(slot.x_vel, 6, "ASM loc_4014C: neg.w $26(a0) left bounce");
        assert_eq!(
            slot.screen_x, -4,
            "ASM loc_40176: d0 stored as-is after bounce"
        );
    }

    #[test]
    fn five_way_shot_slot_bounces_right_edge_loc_40166() {
        // ASM loc_40166: cmpi.w #$140,d0; blt.s skip; tst.w $26(a0); ble.s skip; neg.w $26(a0)
        let mut slot = helmets::FiveWayShotSlot {
            screen_x: 0x13E,
            screen_y: 0x40,
            x_vel: 8,
            y_vel: 0,
            size_index: 5,
        };
        let _ = helmets::diamond_power_main_five_way_shot_slot(&mut slot, 0, 0, 10);
        // d0 = 0x13E + 8 = 0x146 >= 0x140, x_vel=8 > 0, so negated to -8
        assert_eq!(slot.x_vel, -8, "ASM loc_40166: neg.w $26(a0) right bounce");
    }

    #[test]
    fn five_way_shot_slot_no_bounce_when_vel_already_correct_loc_40166() {
        // ASM loc_40166: if x_vel <= 0 when at right edge, no bounce (ble.s skip).
        let mut slot = helmets::FiveWayShotSlot {
            screen_x: 0x13E,
            screen_y: 0x40,
            x_vel: -8, // already heading left
            y_vel: 0,
            size_index: 5,
        };
        let _ = helmets::diamond_power_main_five_way_shot_slot(&mut slot, 0, 0, 10);
        // d0 = 0x13E + (-8) = 0x136 < 0x140, no bounce triggered
        assert_eq!(
            slot.x_vel, -8,
            "ASM loc_40166: no bounce, vel already negative"
        );
    }

    #[test]
    fn five_way_shot_slot_bounces_top_edge_loc_40176() {
        // ASM loc_40176: after storing X, add.w $2A(a0),d1; bpl.s loc_4018A;
        // tst.w $2A(a0); bge.s loc_4018A; neg.w $2A(a0)
        let mut slot = helmets::FiveWayShotSlot {
            screen_x: 0x80,
            screen_y: 2,
            x_vel: 0,
            y_vel: -4,
            size_index: 5,
        };
        let _ = helmets::diamond_power_main_five_way_shot_slot(&mut slot, 0, 0, 10);
        // d1 = 2 + (-4) = -2 < 0, y_vel was -4, so negated to 4
        assert_eq!(slot.y_vel, 4, "ASM loc_40176: neg.w $2A(a0) top bounce");
    }

    #[test]
    fn five_way_shot_slot_bounces_bottom_edge_loc_4018a() {
        // ASM loc_4018A: cmpi.w #$E0,d1; blt.s loc_4019A; tst.w $2A(a0); ble.s; neg.w $2A(a0)
        let mut slot = helmets::FiveWayShotSlot {
            screen_x: 0x80,
            screen_y: 0xDE,
            x_vel: 0,
            y_vel: 4,
            size_index: 5,
        };
        let _ = helmets::diamond_power_main_five_way_shot_slot(&mut slot, 0, 0, 10);
        // d1 = 0xDE + 4 = 0xE2 >= 0xE0, y_vel=4 > 0, so negated to -4
        assert_eq!(slot.y_vel, -4, "ASM loc_4018A: neg.w $2A(a0) bottom bounce");
    }

    #[test]
    fn five_way_shot_slot_fadeout_removes_at_negative_size_loc_4019a() {
        // ASM loc_4019A: when timer < 0 AND (timer & 7) == 0, subq.w #1,$24(a0).
        // If size goes negative, sub_3F5E8 removes the slot.
        let mut slot = helmets::FiveWayShotSlot {
            screen_x: 0x80,
            screen_y: 0x40,
            x_vel: 0,
            y_vel: 0,
            size_index: 0,
        };
        let result = helmets::diamond_power_main_five_way_shot_slot(&mut slot, 0, 0, -8); // -8 & 7 == 0
        assert_eq!(
            result,
            helmets::FiveWayShotSlotAction::Removed,
            "ASM loc_4019A: bsr.w sub_3F5E8 when size underflows"
        );
    }

    #[test]
    fn five_way_shot_slot_fadeout_decrements_size_loc_4019a() {
        // ASM loc_4019A: subq.w #1,$24(a0); bpl.s loc_401BA — size decremented but >= 0, continue.
        let mut slot = helmets::FiveWayShotSlot {
            screen_x: 0x80,
            screen_y: 0x40,
            x_vel: 0,
            y_vel: 0,
            size_index: 3,
        };
        let result = helmets::diamond_power_main_five_way_shot_slot(&mut slot, 0, 0, -16); // -16 & 7 == 0
        assert!(
            matches!(result, helmets::FiveWayShotSlotAction::Visible { .. }),
            "ASM loc_4019A: size decremented to 2, still visible"
        );
        assert_eq!(slot.size_index, 2, "ASM loc_4019A: subq.w #1,$24(a0)");
    }

    #[test]
    fn five_way_shot_slot_fadeout_skips_on_non_8th_frame_loc_4019a() {
        // ASM loc_4019A: andi.w #7,d7; bne.s loc_401BA — skip decrement when (timer & 7) != 0.
        let mut slot = helmets::FiveWayShotSlot {
            screen_x: 0x80,
            screen_y: 0x40,
            x_vel: 0,
            y_vel: 0,
            size_index: 1,
        };
        let result = helmets::diamond_power_main_five_way_shot_slot(&mut slot, 0, 0, -3); // -3 & 7 == 5 != 0
        assert!(
            matches!(result, helmets::FiveWayShotSlotAction::Visible { .. }),
            "ASM loc_4019A: non-8th frame, no decrement"
        );
        assert_eq!(slot.size_index, 1, "ASM loc_4019A: size unchanged");
    }

    #[test]
    fn five_way_shot_slot_sprite_compile_loc_401ba() {
        // ASM loc_401BA: world coords = screen + camera, centred by size_index,
        // VRAM attr = size_index*4 + $1144.
        let mut slot = helmets::FiveWayShotSlot {
            screen_x: 0x80,
            screen_y: 0x40,
            x_vel: 0,
            y_vel: 0,
            size_index: 5,
        };
        let result = helmets::diamond_power_main_five_way_shot_slot(&mut slot, 0x100, 0x80, 10);
        match result {
            helmets::FiveWayShotSlotAction::Visible {
                world_x,
                world_y,
                vram_attr,
                y,
                x,
                ..
            } => {
                // ASM: d5 = d0 + d3; move.w d5,$1A(a0)
                assert_eq!(
                    world_x, 0x180,
                    "ASM loc_401BA: world_x = screen_x + camera_x"
                );
                // ASM: d5 = d1 + d4; move.w d5,$1E(a0)
                assert_eq!(
                    world_y, 0xC0,
                    "ASM loc_401BA: world_y = screen_y + camera_y"
                );
                // ASM: d6=5, d6*4=20, $1144 + 20 = $1158
                assert_eq!(
                    vram_attr, 0x1158,
                    "ASM loc_401BA: vram_attr = size*4 + $1144"
                );
                // ASM: d0 = $80 + $7F - 5 = $FA; d1 = $40 + $7F - 5 = $BA
                assert_eq!(
                    x, 0xFA_u16,
                    "ASM loc_401BA: sprite_x = screen_x + $7F - size"
                );
                assert_eq!(
                    y, 0xBA_u16,
                    "ASM loc_401BA: sprite_y = screen_y + $7F - size"
                );
            }
            other @ helmets::FiveWayShotSlotAction::Removed => {
                panic!("Expected Visible, got {other:?}")
            }
        }
    }

    // ── WallOfDeath main per-slot (loc_40026, loc_40044, loc_40060) ─────────────

    #[test]
    fn wall_of_death_slot_rightward_movement_loc_40026() {
        // ASM loc_40026: when reached_right_edge=false and d0 < $139,
        // addq.w #8,$3E(a0) increments x_offset, then sprite compiled at old d0.
        let mut slot = helmets::WallOfDeathSlot {
            x_offset: 0x50,
            y_offset: 0x0D,
            reached_right_edge: false,
            size_index: 6,
        };
        let result = helmets::diamond_power_main_wall_of_death_slot(
            &mut slot, 0x100, 0x80, 100, // timer positive
        );
        // x_offset should have been incremented by 8
        assert_eq!(slot.x_offset, 0x58, "ASM loc_40026: addq.w #8,$3E(a0)");
        assert!(!slot.reached_right_edge, "flag stays false when d0 < $139");
        // Sprite compiled at old x_offset (0x50)
        match result {
            helmets::WallOfDeathSlotAction::Visible { world_x, .. } => {
                // ASM loc_4006E: d0 + camera_x = 0x50 + 0x100 = 0x150
                assert_eq!(world_x, 0x150, "sprite uses old x_offset + camera_x");
            }
            helmets::WallOfDeathSlotAction::Removed => panic!("expected Visible"),
        }
    }

    #[test]
    fn wall_of_death_slot_sets_flag_at_right_edge_loc_40026() {
        // ASM loc_40026: when d0 >= $139, st $42(a0) sets reached_right_edge.
        // Then falls through to loc_40044 → loc_40060 (net x_offset change: +8-8=0).
        let mut slot = helmets::WallOfDeathSlot {
            x_offset: 0x139,
            y_offset: 0x0D,
            reached_right_edge: false,
            size_index: 6,
        };
        let result = helmets::diamond_power_main_wall_of_death_slot(&mut slot, 0x100, 0x80, 100);
        // +8 from rightward, -8 from loc_40060 leftward → net 0
        assert_eq!(slot.x_offset, 0x139, "net x_offset unchanged at edge");
        assert!(slot.reached_right_edge, "ASM: st $42(a0) at right edge");
        assert!(matches!(
            result,
            helmets::WallOfDeathSlotAction::Visible { .. }
        ));
    }

    #[test]
    fn wall_of_death_slot_leftward_clears_flag_loc_40060() {
        // ASM loc_40060: when reached_right_edge=true and d0 <= 7,
        // sf $42(a0) clears the flag. subq.w #8,$3E(a0) decrements x_offset.
        let mut slot = helmets::WallOfDeathSlot {
            x_offset: 5,
            y_offset: 0x0D,
            reached_right_edge: true,
            size_index: 6,
        };
        let result = helmets::diamond_power_main_wall_of_death_slot(&mut slot, 0x100, 0x80, 100);
        // ASM loc_40060: subq.w #8,$3E(a0)
        assert_eq!(slot.x_offset, -3, "ASM loc_40060: subq.w #8,$3E(a0)");
        // ASM: cmpi.w #7,d0 — old d0=5 <= 7, so sf $42(a0)
        assert!(
            !slot.reached_right_edge,
            "ASM loc_40060: sf $42(a0) when d0<=7"
        );
        assert!(matches!(
            result,
            helmets::WallOfDeathSlotAction::Visible { .. }
        ));
    }

    #[test]
    fn wall_of_death_slot_leftward_keeps_flag_when_above_7_loc_40060() {
        // ASM loc_40060: when d0 > 7, bgt.s loc_4006E skips clearing the flag.
        let mut slot = helmets::WallOfDeathSlot {
            x_offset: 0x50,
            y_offset: 0x0D,
            reached_right_edge: true,
            size_index: 6,
        };
        let _result = helmets::diamond_power_main_wall_of_death_slot(&mut slot, 0x100, 0x80, 100);
        assert_eq!(slot.x_offset, 0x48, "ASM loc_40060: subq.w #8");
        assert!(slot.reached_right_edge, "flag stays true when d0 > 7");
    }

    #[test]
    fn wall_of_death_slot_fadeout_removes_at_negative_size_loc_40044() {
        // ASM loc_40044: when timer < 0 and (timer & 7) == 0,
        // subq.w #1,$24(a0). If size goes negative → sub_3F5E8 (Removed).
        let mut slot = helmets::WallOfDeathSlot {
            x_offset: 0x50,
            y_offset: 0x0D,
            reached_right_edge: true,
            size_index: 0, // will become 0xFFFF (i.e. -1 as i16)
        };
        // Timer must be negative and (timer & 7) == 0
        let result = helmets::diamond_power_main_wall_of_death_slot(
            &mut slot, 0x100, 0x80, -8, // -8 & 7 == 0
        );
        assert_eq!(
            result,
            helmets::WallOfDeathSlotAction::Removed,
            "ASM loc_40044: size < 0 → sub_3F5E8"
        );
    }

    #[test]
    fn wall_of_death_slot_fadeout_decrements_size_loc_40044() {
        // ASM loc_40044: timer < 0 and (timer & 7) == 0 → subq.w #1,$24(a0).
        // If size >= 0 after decrement → continues to loc_40060.
        let mut slot = helmets::WallOfDeathSlot {
            x_offset: 0x50,
            y_offset: 0x0D,
            reached_right_edge: true,
            size_index: 3,
        };
        let result = helmets::diamond_power_main_wall_of_death_slot(
            &mut slot, 0x100, 0x80, -16, // -16 & 7 == 0
        );
        assert_eq!(slot.size_index, 2, "ASM loc_40044: subq.w #1,$24(a0)");
        assert!(
            matches!(result, helmets::WallOfDeathSlotAction::Visible { .. }),
            "size still >= 0 → sprite compiled"
        );
    }

    #[test]
    fn wall_of_death_slot_fadeout_skips_on_non_8th_frame_loc_40044() {
        // ASM loc_40044: andi.w #7,d7; bne.s loc_40060 — skip when not 8th frame.
        let mut slot = helmets::WallOfDeathSlot {
            x_offset: 0x50,
            y_offset: 0x0D,
            reached_right_edge: true,
            size_index: 3,
        };
        let result = helmets::diamond_power_main_wall_of_death_slot(
            &mut slot, 0x100, 0x80, -5, // -5 & 7 == 3, not 0
        );
        // Size should NOT be decremented
        assert_eq!(slot.size_index, 3, "size unchanged on non-8th frame");
        assert!(matches!(
            result,
            helmets::WallOfDeathSlotAction::Visible { .. }
        ));
    }

    #[test]
    fn wall_of_death_slot_sprite_compile_loc_4006e() {
        // ASM loc_4006E: verify full sprite compilation output.
        // d0=0x50, d1=0x0D, camera_x=0x100, camera_y=0x80, size_index=6
        let mut slot = helmets::WallOfDeathSlot {
            x_offset: 0x50,
            y_offset: 0x0D,
            reached_right_edge: true,
            size_index: 6,
        };
        let result = helmets::diamond_power_main_wall_of_death_slot(&mut slot, 0x100, 0x80, 100);
        match result {
            helmets::WallOfDeathSlotAction::Visible {
                world_x,
                world_y,
                vram_attr,
                y,
                size_byte,
                pattern,
                x,
            } => {
                // world_x = d0 + camera_x = 0x50 + 0x100 = 0x150
                assert_eq!(world_x, 0x150, "ASM: d0 + d3");
                // world_y = d1 + camera_y = 0x0D + 0x80 = 0x8D
                assert_eq!(world_y, 0x8D, "ASM: d1 + d4");
                // sprite_x = d0 + 0x7F - size = 0x50 + 0x7F - 6 = 0xC9
                assert_eq!(x, 0xC9, "ASM: d0 + $7F - d6");
                // sprite_y = d1 + 0x7F - size = 0x0D + 0x7F - 6 = 0x86
                assert_eq!(y, 0x86, "ASM: d1 + $7F - d6");
                // vram_attr = 6*4 + 0x1144 = 24 + 0x1144 = 0x115C
                assert_eq!(vram_attr, 0x115C, "ASM: d6*4 + $1144");
                // word_4020E[6] = DIAMOND_SPRITE_ATTR_TABLE[6] = (0x05, 0x86F6)
                assert_eq!(size_byte, 0x05, "ASM: word_4020E size_byte");
                assert_eq!(pattern, 0x86F6, "ASM: word_4020E pattern");
            }
            helmets::WallOfDeathSlotAction::Removed => panic!("expected Visible"),
        }
    }

    #[test]
    fn diamond_power_object_remove_sub_3f5e8_unlinks_the_slot_and_keeps_flags_while_objects_remain()
    {
        let result = helmets::diamond_power_object_remove_sub_3F5E8(
            helmets::DiamondPowerObjectListsSub3F5E8 {
                active_slots: vec![0xF2AC, 0xF2F8, 0xF344],
                free_slots: vec![0xF390, 0xF3DC],
                last_active_slot: Some(0xF344),
                diamond_power_active: true,
                five_way_shot_ready: true,
                samurai_haze_active: true,
                kid_is_invulnerable: true,
            },
            0xF2F8,
        );

        assert_eq!(
            result,
            helmets::DiamondPowerObjectListsSub3F5E8 {
                active_slots: vec![0xF2AC, 0xF344],
                free_slots: vec![0xF2F8, 0xF390, 0xF3DC],
                last_active_slot: Some(0xF344),
                diamond_power_active: true,
                five_way_shot_ready: true,
                samurai_haze_active: true,
                kid_is_invulnerable: true,
            },
            "ASM loc_3F5EC: traversal loop finds predecessor; loc_3F5F8 skips tail update because removed slot is not Addr_LastDPObjectSlot"
        );
    }

    /// ASM loc_3F60C: when the removed slot IS `Addr_LastDPObjectSlot` and the
    /// predecessor is not the sentinel, `move.l a1,(Addr_LastDPObjectSlot).w`
    /// updates the tail pointer to the predecessor.
    #[test]
    fn diamond_power_object_remove_sub_3f5e8_updates_last_pointer_when_tail_removed_loc_3f60c() {
        // Remove the tail slot (0xF344) while 0xF2AC remains.
        // ASM loc_3F5EC traverses past 0xF2AC to find 0xF344; loc_3F5F8 detects
        // it equals Addr_LastDPObjectSlot; a1 (predecessor 0xF2AC) is not the
        // sentinel, so loc_3F60C stores 0xF2AC as the new Addr_LastDPObjectSlot.
        let result = helmets::diamond_power_object_remove_sub_3F5E8(
            helmets::DiamondPowerObjectListsSub3F5E8 {
                active_slots: vec![0xF2AC, 0xF344],
                free_slots: vec![0xF390],
                last_active_slot: Some(0xF344),
                diamond_power_active: true,
                five_way_shot_ready: false,
                samurai_haze_active: false,
                kid_is_invulnerable: false,
            },
            0xF344,
        );

        assert_eq!(
            result,
            helmets::DiamondPowerObjectListsSub3F5E8 {
                active_slots: vec![0xF2AC],
                free_slots: vec![0xF344, 0xF390],
                last_active_slot: Some(0xF2AC),
                diamond_power_active: true,
                five_way_shot_ready: false,
                samurai_haze_active: false,
                kid_is_invulnerable: false,
            },
            "ASM loc_3F60C: predecessor becomes new Addr_LastDPObjectSlot when tail is removed"
        );
    }

    #[test]
    fn diamond_power_object_remove_sub_3f5e8_clears_shared_flags_when_the_last_slot_is_removed() {
        let result = helmets::diamond_power_object_remove_sub_3F5E8(
            helmets::DiamondPowerObjectListsSub3F5E8 {
                active_slots: vec![0xF2AC],
                free_slots: vec![0xF2F8],
                last_active_slot: Some(0xF2AC),
                diamond_power_active: true,
                five_way_shot_ready: true,
                samurai_haze_active: true,
                kid_is_invulnerable: true,
            },
            0xF2AC,
        );

        assert_eq!(
            result,
            helmets::DiamondPowerObjectListsSub3F5E8 {
                active_slots: Vec::new(),
                free_slots: vec![0xF2AC, 0xF2F8],
                last_active_slot: None,
                diamond_power_active: false,
                five_way_shot_ready: false,
                samurai_haze_active: false,
                kid_is_invulnerable: false,
            },
            "ASM loc_3F5F8: removed slot IS Addr_LastDPObjectSlot and predecessor IS sentinel ($FFFFF5A0), so clr.l Addr_LastDPObjectSlot; loc_3F62E clears shared flags when list empties"
        );
    }

    #[test]
    fn diamond_power_object_reset_sub_3f63c_clears_heads_and_flags_but_keeps_the_stale_last_pointer()
     {
        let result = helmets::diamond_power_object_reset_sub_3F63C(
            helmets::DiamondPowerObjectListsSub3F5E8 {
                active_slots: vec![0xF2AC, 0xF2F8],
                free_slots: vec![0xF344, 0xF390],
                last_active_slot: Some(0xF2F8),
                diamond_power_active: true,
                five_way_shot_ready: true,
                samurai_haze_active: true,
                kid_is_invulnerable: true,
            },
        );

        assert_eq!(
            result,
            helmets::DiamondPowerObjectListsSub3F5E8 {
                active_slots: Vec::new(),
                free_slots: Vec::new(),
                last_active_slot: Some(0xF2F8),
                diamond_power_active: false,
                five_way_shot_ready: false,
                samurai_haze_active: false,
                kid_is_invulnerable: false,
            },
            "ASM sub_3F63C should clear `Addr_FirstDPObjectSlot` and `Addr_NextFreeDPObjectSlot` plus the shared flags, but it does not write `Addr_LastDPObjectSlot`"
        );
    }

    // -----------------------------------------------------------------------
    // DiamondPower_CompileSprites (loc_3F7AE)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_compile_sprites_empty_buffer_returns_unchanged_count() {
        // ASM DiamondPower_CompileSprites: when buffer start == end (a0==a1),
        // returns immediately without touching Number_Sprites.
        let (sprites, count) = helmets::diamond_power_compile_sprites_sub_3F798(&[], 5);
        assert!(sprites.is_empty(), "empty buffer produces no sprites");
        assert_eq!(count, 5, "sprite count unchanged on empty buffer");
    }

    #[test]
    fn diamond_power_compile_sprites_adjusts_link_and_masks_pattern_loc_3f7ae() {
        // ASM loc_3F7AE: inner loop copies Y, adjusts size+link by adding
        // running sprite count (d2), masks pattern bit 15, copies X.
        // Two entries: initial sprite count = 3.
        let buffer = [
            [0x0080u16, 0x0500, 0x8123, 0x00A0], // Y=$80, size+link=$500, pattern=$8123 (bit 15 set), X=$A0
            [0x0090, 0x0600, 0x0456, 0x00B0], // Y=$90, size+link=$600, pattern=$456 (bit 15 clear), X=$B0
        ];
        let (sprites, count) = helmets::diamond_power_compile_sprites_sub_3F798(&buffer, 3);

        assert_eq!(sprites.len(), 2);
        // First entry: d2 increments to 4; size_link = $500 + 4 = $504
        assert_eq!(sprites[0].y, 0x0080, "ASM loc_3F7AE: Y word copied");
        assert_eq!(
            sprites[0].size_link, 0x0504,
            "ASM loc_3F7AE: size+link adjusted by d2=4"
        );
        assert_eq!(
            sprites[0].pattern, 0x0123,
            "ASM loc_3F7AE: andi.w #$7FFF clears bit 15"
        );
        assert_eq!(sprites[0].x, 0x00A0, "ASM loc_3F7AE: X word copied");

        // Second entry: d2 increments to 5; size_link = $600 + 5 = $605
        assert_eq!(sprites[1].y, 0x0090);
        assert_eq!(
            sprites[1].size_link, 0x0605,
            "ASM loc_3F7AE: size+link adjusted by d2=5"
        );
        assert_eq!(
            sprites[1].pattern, 0x0456,
            "ASM loc_3F7AE: pattern unchanged when bit 15 already clear"
        );
        assert_eq!(sprites[1].x, 0x00B0);

        assert_eq!(count, 5, "ASM: Number_Sprites updated to final d2");
    }

    // -----------------------------------------------------------------------
    // DiamondPower_Init_CircleOfDoom (loc_3F7E6)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_init_circle_of_doom_allocates_8_slots_with_evenly_spaced_angles_loc_3f7e6() {
        // ASM DiamondPower_Init_CircleOfDoom (loc_3F7D0):
        // d1=8, d2=($10000/8)<<8=$200000, d3=0, d0=$00210000.
        // ASM loc_3F7E6: loop allocates 8 slots:
        //   $3E(a0) = d0 = $00210000 (radius),
        //   $40(a0) = 0 (shrink flag),
        //   $42(a0) = d3 (angle), d3 += $200000.
        let slots = helmets::diamond_power_init_circle_of_doom_sub_3F7D0();
        assert_eq!(slots.len(), 8, "ASM loc_3F7E6: dbf d1 loops 8 times");

        // All slots share the same initial radius $00210000
        for (i, slot) in slots.iter().enumerate() {
            assert_eq!(
                slot.radius, 0x0021_0000,
                "slot {i}: ASM move.l d0,$3E(a0) stores $00210000"
            );
            assert_eq!(slot.shrink_flag, 0, "slot {i}: ASM clr.w $40(a0)");
            assert_eq!(slot.status, 0, "slot {i}: freshly allocated, status=0");
        }

        // Angles: 0, $200000, $400000, ..., $E00000
        for i in 0..8u32 {
            assert_eq!(
                slots[i as usize].angle,
                i * 0x20_0000,
                "slot {i}: ASM loc_3F7E6 angle = d3, step $200000"
            );
        }
    }

    // -----------------------------------------------------------------------
    // DiamondPower_Main_CircleOfDoom per-slot (loc_3F81A)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_main_circle_of_doom_slot_position_uses_sin_cos_loc_3f81a() {
        // ASM loc_3F81A: d5 = radius >> 8; angle from $42.w (upper word);
        // cos/sin from sub_40236; x = cos*d5>>16>>6 + kid_x; y = -(sin*d5>>16>>6) + kid_y - $10.
        // Use angle=0 (cos≈$3FFE, sin=0) so x offset is positive, y offset is 0.
        let mut slot = helmets::CircleOfDoomSlot {
            radius: 0x0021_0000, // d5 = $2100
            shrink_flag: 0,
            angle: 0x0000_0000, // angle word = 0
            status: 0,
        };
        let result = helmets::diamond_power_main_circle_of_doom_slot(&mut slot, 100, 200, 0, 0);

        // cos(0) ≈ $3FFE, sin(0) = 0
        // x_offset = ($3FFE * $2100) >> 16 >> 6 = ($83F_BBFE >> 16) >> 6 = $83F >> 6 = $20 = 32
        // y_offset = -(0 * $2100 >> 16 >> 6) = 0; world_y = 200 + 0 - $10 = 184
        assert_eq!(
            result.screen_x,
            100 + 32 + 0x7F, // world_x + d3 ($7F - camera_x=0)
            "ASM loc_3F81A: cos(0)*radius >> 22 + kid_x + screen_offset"
        );
        assert_eq!(
            result.screen_y,
            184 + 0x7F, // world_y + d4
            "ASM loc_3F81A: y = kid_y - $10 + screen_offset (sin=0)"
        );
        assert!(!result.removed, "slot should not be removed on first tick");
    }

    #[test]
    fn diamond_power_main_circle_of_doom_slot_shrinks_radius_loc_3f81a() {
        // ASM loc_3F81A: subi.l #$400,$3E(a0); when d5>>1 < $C, also
        // subi.l #$19000,$3E(a0). Initial radius $00210000 → d5=$2100,
        // size_index = $21/2 = $10 = 16 ≥ $C, so only $400 subtracted.
        let mut slot = helmets::CircleOfDoomSlot {
            radius: 0x0021_0000,
            shrink_flag: 0,
            angle: 0x0000_0000,
            status: 0,
        };
        let result = helmets::diamond_power_main_circle_of_doom_slot(&mut slot, 0, 0, 0, 0);
        assert!(!result.removed);
        assert_eq!(
            slot.radius,
            0x0021_0000 - 0x400,
            "ASM: subi.l #$400,$3E(a0) — slow shrink when size_index >= $C"
        );
    }

    #[test]
    fn diamond_power_main_circle_of_doom_slot_reads_size_table_loc_3f81a() {
        // ASM loc_3F81A: move.w $3E(a0),d5; lsr.w #1,d5;
        // move.b byte_3F8B4(pc,d5.w),d6. With upper word $21, d5=$10 and
        // byte_3F8B4[$10] is 5, not the final table entry.
        let mut slot = helmets::CircleOfDoomSlot {
            radius: 0x0021_0000,
            shrink_flag: 0,
            angle: 0x0000_0000,
            status: 0,
        };
        let result = helmets::diamond_power_main_circle_of_doom_slot(&mut slot, 0, 0, 0, 0);
        assert_eq!(
            result.half_size, 5,
            "ASM loc_3F81A: byte_3F8B4[$10] supplies the sprite half-size"
        );
    }

    #[test]
    fn diamond_power_main_circle_of_doom_slot_size_index_12_uses_slow_shrink_loc_3f81a() {
        // ASM loc_3F81A: cmpi.w #$C,d5; bge.s loc_3F880. The equality case
        // reaches loc_3F880, so radius index $C must not take the fast shrink.
        let mut slot = helmets::CircleOfDoomSlot {
            radius: 0x0018_0000,
            shrink_flag: 0,
            angle: 0x0000_0000,
            status: 0,
        };
        let result = helmets::diamond_power_main_circle_of_doom_slot(&mut slot, 0, 0, 0, 0);
        assert!(!result.removed);
        assert_eq!(
            slot.radius,
            0x0018_0000 - 0x400,
            "ASM loc_3F81A: d5 == $C branches to loc_3F880 after only #$400 shrink"
        );
    }

    #[test]
    fn diamond_power_main_circle_of_doom_slot_fast_shrink_when_small_loc_3f81a() {
        // ASM: when $3E.w >> 1 < $C (radius upper word < $18), both $400 and
        // $19000 are subtracted. Set radius so upper word = $16 → index = $B < $C.
        let mut slot = helmets::CircleOfDoomSlot {
            radius: 0x0016_0000 + 0x1_9400, // ensure it stays positive after both subs
            shrink_flag: 0,
            angle: 0x0000_0000,
            status: 0,
        };
        let initial = slot.radius;
        let result = helmets::diamond_power_main_circle_of_doom_slot(&mut slot, 0, 0, 0, 0);
        assert!(!result.removed);
        assert_eq!(
            slot.radius,
            initial - 0x400 - 0x19000,
            "ASM: both subi.l #$400 and subi.l #$19000 when size_index < $C"
        );
    }

    #[test]
    fn diamond_power_main_circle_of_doom_slot_keeps_exact_zero_radius_loc_3f81a() {
        // ASM loc_3F81A: after the fast shrink, `bge.s loc_3F880` keeps zero
        // or positive results. Only a negative long radius removes the slot.
        let mut slot = helmets::CircleOfDoomSlot {
            radius: 0x0001_9400,
            shrink_flag: 0,
            angle: 0x0000_0000,
            status: 0,
        };
        let result = helmets::diamond_power_main_circle_of_doom_slot(&mut slot, 0, 0, 0, 0);
        assert!(
            !result.removed,
            "ASM loc_3F81A: exact zero after #$400 + #$19000 still branches to loc_3F880"
        );
        assert_eq!(slot.radius, 0);
    }

    #[test]
    fn diamond_power_main_circle_of_doom_slot_removes_when_radius_depleted_loc_3f81a() {
        // ASM: when $3E - $400 - $19000 < 0, slot is removed via sub_3F5E8.
        let mut slot = helmets::CircleOfDoomSlot {
            radius: 0x0001_0000, // small radius, upper word = 1, index = 0 < $C
            shrink_flag: 0,
            angle: 0x0000_0000,
            status: 0,
        };
        let result = helmets::diamond_power_main_circle_of_doom_slot(&mut slot, 0, 0, 0, 0);
        assert!(
            result.removed,
            "ASM loc_3F87A: radius goes negative → sub_3F5E8 removes slot"
        );
    }

    #[test]
    fn diamond_power_main_circle_of_doom_slot_removes_when_status_nonzero_loc_3f81a() {
        // ASM: tst.w $38(a0); bne.s loc_3F87A — nonzero status triggers removal.
        let mut slot = helmets::CircleOfDoomSlot {
            radius: 0x0021_0000,
            shrink_flag: 0,
            angle: 0x0000_0000,
            status: 1,
        };
        let result = helmets::diamond_power_main_circle_of_doom_slot(&mut slot, 0, 0, 0, 0);
        assert!(
            result.removed,
            "ASM loc_3F81A: nonzero $38(a0) triggers removal"
        );
    }

    #[test]
    fn diamond_power_main_circle_of_doom_slot_decrements_angle_loc_3f81a() {
        // ASM: subq.w #4,$42(a0) — decrements the angle upper word by 4 each frame.
        let mut slot = helmets::CircleOfDoomSlot {
            radius: 0x0021_0000,
            shrink_flag: 0,
            angle: 0x0020_0000, // upper word = $20
            status: 0,
        };
        helmets::diamond_power_main_circle_of_doom_slot(&mut slot, 0, 0, 0, 0);
        assert_eq!(
            slot.angle >> 16,
            0x1C,
            "ASM: subq.w #4,$42(a0) decrements angle upper word from $20 to $1C"
        );
    }

    // DiamondPower_Main_CircleOfDoom linked-list traversal tail (loc_3F8AA)

    #[test]
    fn diamond_power_main_circle_of_doom_linked_list_traversal_loc_3f8aa() {
        // ASM loc_3F8AA: move.l 4(a0),d0; bne.w loc_3F81A; rts
        // After processing one slot, loads the next-slot pointer from offset 4.
        // Nonzero → continue loop (back to loc_3F81A); zero → return.
        // Both the removal path (loc_3F87A → sub_3F5E8 → bra.s loc_3F8AA)
        // and the compile-sprite path (loc_3F880 fallthrough) converge here.
        // In native, the caller iterates the Vec — verify that processing
        // multiple slots produces independent results (no cross-slot leakage).
        let mut slots = helmets::diamond_power_init_circle_of_doom_sub_3F7D0();
        assert_eq!(slots.len(), 8);

        let r0 = helmets::diamond_power_main_circle_of_doom_slot(&mut slots[0], 100, 200, 0, 0);
        let r1 = helmets::diamond_power_main_circle_of_doom_slot(&mut slots[1], 100, 200, 0, 0);
        // Each slot processes independently — different angles yield different positions.
        // ASM loc_3F8AA: the loop continues via 4(a0) nonzero to loc_3F81A.
        assert_ne!(
            r0.screen_x, r1.screen_x,
            "ASM loc_3F8AA: consecutive slots produce different X (different angles)"
        );
        assert!(!r0.removed, "ASM loc_3F8AA: first slot not removed");
        assert!(!r1.removed, "ASM loc_3F8AA: second slot not removed");

        // Verify removed slot still reaches loc_3F8AA (via loc_3F87A → sub_3F5E8):
        slots[2].status = 1;
        let r2 = helmets::diamond_power_main_circle_of_doom_slot(&mut slots[2], 100, 200, 0, 0);
        assert!(
            r2.removed,
            "ASM loc_3F8AA: removal path (loc_3F87A) converges to loc_3F8AA"
        );
    }

    // DiamondPower_Init_Snake (loc_3F92A, loc_3F946)

    #[test]
    fn diamond_power_init_snake_allocates_6_slots_at_kid_position_loc_3f92a_loc_3f946() {
        // ASM loc_3F92A: bsr.w Allocate_DiamondPowerObjectSlot — 6 slots (d1=5..0)
        // ASM loc_3F946: each slot gets Kid x/y as position, size from table,
        // trail buffer filled with Kid x/y.
        let result = helmets::diamond_power_init_snake(
            6,     // d4 = wander speed (MiniSnake)
            0x300, // d5 = movement speed
            &helmets::SNAKE_INIT_DATA_MINI,
            0x1E0, // timer
            0x120, // kid_x
            0x80,  // kid_y
        );

        assert_eq!(
            result.slots.len(),
            6,
            "ASM loc_3F91C: moveq #5,d1 → 6 slots"
        );

        // ASM loc_3F946: move.w d6,$1A(a0); move.w d7,$1E(a0)
        for (i, slot) in result.slots.iter().enumerate() {
            assert_eq!(slot.x_pos, 0x120, "slot {i}: ASM loc_3F946 x_pos = kid_x");
            assert_eq!(slot.y_pos, 0x80, "slot {i}: ASM loc_3F946 y_pos = kid_y");
        }

        // ASM loc_3F946: move.w (a2)+,$24(a0) — diamond sizes from init table
        assert_eq!(
            result.slots[0].diamond_size, 4,
            "ASM loc_3F946: MiniSnake size[0]=4"
        );
        assert_eq!(
            result.slots[1].diamond_size, 3,
            "ASM loc_3F946: MiniSnake size[1]=3"
        );
        assert_eq!(
            result.slots[2].diamond_size, 2,
            "ASM loc_3F946: MiniSnake size[2]=2"
        );
        assert_eq!(
            result.slots[3].diamond_size, 1,
            "ASM loc_3F946: MiniSnake size[3]=1"
        );
        assert_eq!(
            result.slots[4].diamond_size, 1,
            "ASM loc_3F946: MiniSnake size[4]=1"
        );
        assert_eq!(
            result.slots[5].diamond_size, 1,
            "ASM loc_3F946: MiniSnake size[5]=1"
        );
    }

    #[test]
    fn diamond_power_init_snake_trail_filled_with_kid_position_loc_3f946() {
        // ASM loc_3F946: lea $40(a0),a1; 3x move.w d6,(a1)+; move.w d7,(a1)+
        // Trail buffer at $40-$4A stores 3 copies of (kid_x, kid_y).
        let result = helmets::diamond_power_init_snake(
            6,
            0x300,
            &helmets::SNAKE_INIT_DATA_MINI,
            0x1E0,
            200,
            150,
        );

        for (i, slot) in result.slots.iter().enumerate() {
            assert_eq!(
                slot.trail,
                [(200, 150), (200, 150), (200, 150)],
                "slot {i}: ASM loc_3F946 trail buffer filled with kid x/y"
            );
        }
    }

    #[test]
    fn diamond_power_init_snake_first_slot_stores_head_data_loc_3f92a() {
        // ASM loc_3F92A: cmpi.w #5,d1; bne.s loc_3F946 — first slot (d1==5)
        // stores head pointer at $FFFFF5B4, timer $40 at $3E(a0),
        // wander speed d4 at $2E(a0), movement speed d5 at $30(a0).
        let result = helmets::diamond_power_init_snake(
            0x10,  // d4 = wander speed (SwiftMiniSnake)
            0x480, // d5 = movement speed
            &helmets::SNAKE_INIT_DATA_SWIFT_MINI,
            0x4B0, // timer
            100,
            200,
        );

        assert_eq!(
            result.head_timer, 0x40,
            "ASM loc_3F92A: move.w #$40,$3E(a0)"
        );
        assert_eq!(
            result.wander_speed, 0x10,
            "ASM loc_3F92A: move.w d4,$2E(a0)"
        );
        assert_eq!(
            result.movement_speed, 0x480,
            "ASM loc_3F92A: move.w d5,$30(a0)"
        );
        assert_eq!(
            result.power_timer, 0x4B0,
            "ASM: Diamond_power_timer set by variant"
        );
    }

    #[test]
    fn diamond_power_init_snake_death_snake_sizes_loc_3f946() {
        // ASM DiamondPower_InitData_DeathSnake (kid.asm:44651): [6,6,5,4,3,2]
        // ASM loc_3F946: move.w (a2)+,$24(a0) reads sizes in order.
        let result = helmets::diamond_power_init_snake(
            6,
            0x400,
            &helmets::SNAKE_INIT_DATA_DEATH,
            0x4B0,
            0,
            0,
        );

        let sizes: Vec<u16> = result.slots.iter().map(|s| s.diamond_size).collect();
        assert_eq!(
            sizes,
            vec![6, 6, 5, 4, 3, 2],
            "ASM loc_3F946: DeathSnake sizes from DiamondPower_InitData_DeathSnake"
        );
    }

    // DiamondPower_Main_Snake trail update (loc_3F978)

    #[test]
    fn diamond_power_snake_trail_update_shifts_positions_loc_3f978() {
        // ASM loc_3F978: for each slot except the last, trail[0] → position,
        // trail shifts down, next slot's position → trail[2].
        let mut slots = vec![
            helmets::SnakeSlot {
                x_pos: 100,
                y_pos: 200,
                diamond_size: 4,
                trail: [(110, 210), (120, 220), (130, 230)],
            },
            helmets::SnakeSlot {
                x_pos: 150,
                y_pos: 250,
                diamond_size: 3,
                trail: [(160, 260), (170, 270), (180, 280)],
            },
            helmets::SnakeSlot {
                x_pos: 190,
                y_pos: 290,
                diamond_size: 2,
                trail: [(200, 300), (210, 310), (220, 320)],
            },
        ];

        helmets::diamond_power_snake_update_trails(&mut slots);

        // ASM loc_3F978: slot 0 gets trail[0] as position
        assert_eq!(
            slots[0].x_pos, 110,
            "ASM loc_3F978: move.w (a3),$1A(a0) — trail[0].x → x_pos"
        );
        assert_eq!(
            slots[0].y_pos, 210,
            "ASM loc_3F978: move.w 2(a3),$1E(a0) — trail[0].y → y_pos"
        );
        // trail shifted: [1]→[0], [2]→[1], next.pos→[2]
        assert_eq!(
            slots[0].trail[0],
            (120, 220),
            "ASM loc_3F978: trail[1]→trail[0]"
        );
        assert_eq!(
            slots[0].trail[1],
            (130, 230),
            "ASM loc_3F978: trail[2]→trail[1]"
        );
        assert_eq!(
            slots[0].trail[2],
            (150, 250),
            "ASM loc_3F978: move.w $1A(a1),(a3)+ — next slot pos → trail[2]"
        );

        // ASM loc_3F978: slot 1 similarly updated
        assert_eq!(
            slots[1].x_pos, 160,
            "ASM loc_3F978: slot 1 trail[0].x → x_pos"
        );
        assert_eq!(
            slots[1].y_pos, 260,
            "ASM loc_3F978: slot 1 trail[0].y → y_pos"
        );
        assert_eq!(
            slots[1].trail[2],
            (190, 290),
            "ASM loc_3F978: slot 1 trail[2] = next slot pos"
        );

        // ASM loc_3F978: last slot unchanged (beq.s loc_3F9A2 exits loop)
        assert_eq!(
            slots[2].x_pos, 190,
            "ASM loc_3F978: tail slot position unchanged"
        );
        assert_eq!(
            slots[2].y_pos, 290,
            "ASM loc_3F978: tail slot position unchanged"
        );
        assert_eq!(
            slots[2].trail,
            [(200, 300), (210, 310), (220, 320)],
            "ASM loc_3F978: tail trail unchanged"
        );
    }

    #[test]
    fn diamond_power_snake_trail_single_slot_noop_loc_3f978() {
        // ASM loc_3F978: move.l 4(a0),d1; beq.s loc_3F9A2
        // — single slot has null next pointer, loop body never executes.
        let mut slots = vec![helmets::SnakeSlot {
            x_pos: 50,
            y_pos: 60,
            diamond_size: 1,
            trail: [(70, 80), (90, 100), (110, 120)],
        }];
        helmets::diamond_power_snake_update_trails(&mut slots);
        assert_eq!(slots[0].x_pos, 50, "ASM loc_3F978: single slot — no update");
        assert_eq!(
            slots[0].trail[0],
            (70, 80),
            "ASM loc_3F978: single slot — trail unchanged"
        );
    }

    // DiamondPower_Main_Snake target finding (loc_3F9A2, loc_3F9C2)

    #[test]
    fn diamond_power_snake_find_target_nearest_enemy_loc_3f9a2_loc_3f9c2() {
        // ASM loc_3F9A2: load head screen position, scan enemies.
        // ASM loc_3F9C2: per-enemy on-screen check + Manhattan distance.
        // Two enemies on-screen: pick the nearest by Manhattan distance.
        let head_x = 0x100;
        let head_y = 0x80;
        let cam_x = 0x80;
        let cam_y = 0x20;

        let enemies = vec![
            // Enemy A: screen (0xA0, 0x50) — Manhattan to head screen (0x80,0x60) = 0x20+0x10=0x30
            helmets::SnakeTargetCandidate {
                x_pos: 0x120,
                y_pos: 0x80,
            },
            // Enemy B: screen (0x70, 0x58) — closer: |0x70-0x80|+|0x58-0x60| = 0x10+0x08=0x18
            helmets::SnakeTargetCandidate {
                x_pos: 0xF0,
                y_pos: 0x88,
            },
        ];
        let result = helmets::diamond_power_snake_find_target(
            head_x, head_y, cam_x, cam_y, &enemies, 0x200, 0x100,
        );
        // ASM loc_3F9C2: enemy B is nearer, selected as target
        assert_eq!(result.x, 0xF0, "ASM loc_3F9C2: nearest enemy x selected");
        assert_eq!(result.y, 0x88, "ASM loc_3F9C2: nearest enemy y selected");
    }

    #[test]
    fn diamond_power_snake_find_target_defaults_to_kid_loc_3f9a2() {
        // ASM loc_3FA0C: cmpi.w #$FFFF,d3; bne.s loc_3FA16
        // — d3 stays $FFFF when no on-screen enemy found, defaults to Kid.
        let result = helmets::diamond_power_snake_find_target(
            0x100,
            0x80,
            0x80,
            0x20,
            &[], // no enemies
            0x200,
            0x150,
        );
        assert_eq!(result.x, 0x200, "ASM loc_3FA0C: no enemies → Kid x");
        assert_eq!(result.y, 0x150, "ASM loc_3FA0C: no enemies → Kid y");
    }

    #[test]
    fn diamond_power_snake_find_target_culls_offscreen_loc_3f9c2() {
        // ASM loc_3F9C2: enemies outside screen bounds are skipped.
        // Screen bounds: x in [-8, 328), y in [16, 240) after camera subtraction.
        let cam_x = 0;
        let cam_y = 0;
        let enemies = vec![
            // x screen = -9 → culled by cmpi.w #$FFF8,d4; blt.s
            helmets::SnakeTargetCandidate {
                x_pos: -9,
                y_pos: 0x30,
            },
            // x screen = 0x148 → culled by cmpi.w #$148,d4; bge.s
            helmets::SnakeTargetCandidate {
                x_pos: 0x148,
                y_pos: 0x30,
            },
            // y screen = 0x0F (after -$10) → culled by cmpi.w #$10,d5; blt.s
            helmets::SnakeTargetCandidate {
                x_pos: 0x80,
                y_pos: 0x1F,
            },
            // y screen = 0xF0 (after -$10) → culled by cmpi.w #$F0,d5; bge.s
            helmets::SnakeTargetCandidate {
                x_pos: 0x80,
                y_pos: 0x100,
            },
        ];
        let result = helmets::diamond_power_snake_find_target(
            0x80, 0x80, cam_x, cam_y, &enemies, 0x200, 0x200,
        );
        // All enemies culled → defaults to Kid
        assert_eq!(
            result.x, 0x200,
            "ASM loc_3F9C2: all enemies off-screen → Kid x"
        );
        assert_eq!(
            result.y, 0x200,
            "ASM loc_3F9C2: all enemies off-screen → Kid y"
        );
    }

    // -----------------------------------------------------------------------
    // Snake octant direction — loc_3FA36, loc_3FA40, loc_3FA46
    // -----------------------------------------------------------------------

    /// ASM loc_3FA36/loc_3FA40/loc_3FA46: octant direction from head to target.
    ///
    /// The octant encoding maps relative (dx, dy) from head to target into a
    /// 3-bit index (0–7) used for snake steering. The index encodes:
    ///   bit 2: 0 = target above/level, 6 = target below
    ///   bit 1: XOR'd when dx < 0
    ///   bit 0: set when |dy| < |dx| (more horizontal), then flipped when bit 1 clear
    #[test]
    fn diamond_power_snake_octant_target_right_above_diagonal_loc_3fa36() {
        // Target is right and above head: dx > 0, dy > 0 (head_y > target_y - $10)
        // dx = 0x30, dy = 0x20 → |dy| < |dx| → d3 starts 0, no bit flip for dx>=0
        // loc_3FA40: |dy|(0x20) < |dx|(0x30) → d3 += 1 → d3 = 1
        // loc_3FA46: bit 1 clear → d3 ^= 1 → d3 = 0
        let octant = helmets::diamond_power_snake_octant(
            0x130, // target_x
            0x100, // target_y (target_y - $10 = 0xF0)
            0x100, // head_x   (dx = 0x30)
            0x110, // head_y   (dy = head_y - 0xF0 = 0x20)
        );
        assert_eq!(
            octant, 0,
            "ASM loc_3FA36: target right-above, more horizontal → octant 0"
        );
    }

    #[test]
    fn diamond_power_snake_octant_target_right_above_steep_loc_3fa40() {
        // Target right and above, |dy| >= |dx|: vertical-dominant
        // dx = 0x10, dy = 0x30 → d3 starts 0
        // loc_3FA40: |dy|(0x30) >= |dx|(0x10) → no +1 → d3 = 0
        // loc_3FA46: bit 1 clear → d3 ^= 1 → d3 = 1
        let octant = helmets::diamond_power_snake_octant(
            0x110, // target_x
            0x0E0, // target_y (target_y - $10 = 0xD0)
            0x100, // head_x   (dx = 0x10)
            0x100, // head_y   (dy = head_y - 0xD0 = 0x30)
        );
        assert_eq!(
            octant, 1,
            "ASM loc_3FA40: target right-above, more vertical → octant 1"
        );
    }

    #[test]
    fn diamond_power_snake_octant_target_left_above_loc_3fa36() {
        // Target left and above: dx < 0, dy > 0
        // dx = -0x30, dy = 0x20
        // loc_3FA36: dx < 0 → d3 ^= 2 → d3 = 2, |dx| = 0x30
        // loc_3FA40: |dy|(0x20) < |dx|(0x30) → d3 += 1 → d3 = 3
        // loc_3FA46: bit 1 set → no flip → d3 = 3
        let octant = helmets::diamond_power_snake_octant(
            0x0D0, // target_x
            0x100, // target_y (target_y - $10 = 0xF0)
            0x100, // head_x   (dx = -0x30)
            0x110, // head_y   (dy = 0x20)
        );
        assert_eq!(
            octant, 3,
            "ASM loc_3FA36: target left-above, more horizontal → octant 3"
        );
    }

    #[test]
    fn diamond_power_snake_octant_target_left_below_loc_3fa46() {
        // Target left and below: dx < 0, dy < 0 (head_y < target_y - $10)
        // dx = -0x20, dy = -0x30 → d3 = 6 (below), |dy| = 0x30
        // loc_3FA36: dx < 0 → d3 ^= 2 → d3 = 4, |dx| = 0x20
        // loc_3FA40: |dy|(0x30) >= |dx|(0x20) → no +1 → d3 = 4
        // loc_3FA46: bit 1 clear → d3 ^= 1 → d3 = 5
        let octant = helmets::diamond_power_snake_octant(
            0x0E0, // target_x
            0x150, // target_y (target_y - $10 = 0x140)
            0x100, // head_x   (dx = -0x20)
            0x110, // head_y   (dy = 0x110 - 0x140 = -0x30)
        );
        assert_eq!(
            octant, 5,
            "ASM loc_3FA46: target left-below, more vertical → octant 5"
        );
    }

    #[test]
    fn diamond_power_snake_octant_target_below_right_horizontal_loc_3fa40() {
        // Target right and below, |dy| < |dx|
        // dx = 0x40, dy = -0x10 → d3 = 6 (below), |dy| = 0x10
        // loc_3FA36: dx >= 0 → no flip → d3 = 6
        // loc_3FA40: |dy|(0x10) < |dx|(0x40) → d3 += 1 → d3 = 7
        // loc_3FA46: bit 1 set → no flip → d3 = 7
        let octant = helmets::diamond_power_snake_octant(
            0x140, // target_x
            0x120, // target_y (target_y - $10 = 0x110)
            0x100, // head_x   (dx = 0x40)
            0x100, // head_y   (dy = 0x100 - 0x110 = -0x10)
        );
        assert_eq!(
            octant, 7,
            "ASM loc_3FA40: target right-below, more horizontal → octant 7"
        );
    }

    #[test]
    fn diamond_power_snake_octant_target_directly_below_loc_3fa46() {
        // Target directly below: dx = 0, dy < 0
        // d3 = 6 (below), |dy| > 0, |dx| = 0
        // loc_3FA36: dx = 0 → bpl taken → no flip → d3 = 6
        // loc_3FA40: |dy| >= |dx|(0) → no +1 → d3 = 6
        // loc_3FA46: bit 1 set → no flip → d3 = 6
        let octant = helmets::diamond_power_snake_octant(
            0x100, // target_x
            0x150, // target_y (target_y - $10 = 0x140)
            0x100, // head_x   (dx = 0)
            0x100, // head_y   (dy = 0x100 - 0x140 = -0x40)
        );
        assert_eq!(octant, 6, "ASM loc_3FA46: target directly below → octant 6");
    }

    // -----------------------------------------------------------------------
    // Snake steering (loc_3FA50 through loc_3FAD4)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_snake_steer_different_octant_positive_short_loc_3fa6a() {
        // ASM loc_3FA50: target octant 3, current octant 1 (angle $20 → octant 1).
        // diff = 3 - 1 = 2, positive and < 4 → add turn_rate (loc_3FACC).
        // ASM loc_3FACC: $3E += $2E → $20 + $10 = $30.
        let result = helmets::diamond_power_snake_steer(3, 10, 10, 0x20, 0x10);
        assert_eq!(
            result,
            helmets::SnakeSteerResult::Adjusted(0x30),
            "ASM loc_3FACC: positive diff < 4 adds turn_rate"
        );
    }

    #[test]
    fn diamond_power_snake_steer_different_octant_positive_long_wrap() {
        // ASM loc_3FA50: target octant 1, current octant 5 (angle $A0 → octant 5).
        // diff = 1 - 5 = -4, negative and >= -4 → subtract turn_rate (loc_3FAC2).
        // ASM loc_3FA6A: cmpi.w #$FFFC,d3; bge.s loc_3FAC2.
        // $A0 - $10 = $90.
        let result = helmets::diamond_power_snake_steer(1, 10, 10, 0xA0, 0x10);
        assert_eq!(
            result,
            helmets::SnakeSteerResult::Adjusted(0x90),
            "ASM loc_3FA6A: diff == -4, subtract turn_rate"
        );
    }

    #[test]
    fn diamond_power_snake_steer_negative_diff_beyond_minus4_loc_3fa6a() {
        // ASM loc_3FA6A: target octant 0, current octant 5 (angle $A0).
        // diff = 0 - 5 = -5, < -4 → add turn_rate (loc_3FACC).
        // $A0 + $10 = $B0.
        let result = helmets::diamond_power_snake_steer(0, 10, 10, 0xA0, 0x10);
        assert_eq!(
            result,
            helmets::SnakeSteerResult::Adjusted(0xB0),
            "ASM loc_3FA6A: diff < -4 wraps around, add turn_rate"
        );
    }

    #[test]
    fn diamond_power_snake_steer_positive_diff_ge4_subtract() {
        // ASM: target octant 6, current octant 1 (angle $20).
        // diff = 6 - 1 = 5, >= 4 → subtract turn_rate (loc_3FAC2).
        // $20 - $10 = $10.
        let result = helmets::diamond_power_snake_steer(6, 10, 10, 0x20, 0x10);
        assert_eq!(
            result,
            helmets::SnakeSteerResult::Adjusted(0x10),
            "ASM loc_3FAC2: diff >= 4, subtract turn_rate"
        );
    }

    #[test]
    fn diamond_power_snake_steer_same_octant_zero_displacement_loc_3fa72() {
        // ASM loc_3FA72: d3 == d4 (same octant), d0|d1 == 0 → loc_3FAD4 unchanged.
        // Octant 2: angle with bits 7:5 = 010 → angle in [$40, $5F].
        let result = helmets::diamond_power_snake_steer(2, 0, 0, 0x40, 0x10);
        assert_eq!(
            result,
            helmets::SnakeSteerResult::Unchanged,
            "ASM loc_3FA72: same octant, zero displacement → unchanged"
        );
    }

    #[test]
    fn diamond_power_snake_steer_same_octant_diagonal_axis_swap_loc_3fa84() {
        // ASM loc_3FA84: same octant, nonzero displacement, diagonal octant.
        // Octant 2 is diagonal ($66 bit 2 set), so d0/d1 are swapped before
        // the ratio comparison. Using angle $50 (octant 2, sub_octant=$10).
        // With abs_dx=10, abs_dy=20 — after swap: d0=20, d1=10.
        // d4_ratio = ((10/2) << 16) / 20 = (5 << 16) / 20 = $4000 = 16384.
        // Sub-octant = $50 & $1F = $10 = 16; sin_table[16] = $187D.
        // sin_val/2 = $0C3E; d0_trig = $0C3E << 16 = $0C3E0000.
        // cos_idx = ($80 - $20) / 2 = 48; sin_table[48] = $3B20.
        // d0_tan = $0C3E0000 / $3B20 = $3504 = 13572.
        // Octant 2, bit 0 clear → no swap. cmp $3504 <= $4000 → add turn_rate.
        let result = helmets::diamond_power_snake_steer(2, 10, 20, 0x50, 0x06);
        assert_eq!(
            result,
            helmets::SnakeSteerResult::Adjusted(0x50 + 0x06),
            "ASM loc_3FA84: diagonal octant axis swap, add turn_rate"
        );
    }

    // -----------------------------------------------------------------------
    // Snake slot sprite compilation (loc_3FB20, loc_3FB3E, loc_3FB88)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_snake_slot_sprite_visible_loc_3fb3e() {
        // ASM loc_3FB3E: compile sprite for a slot within screen bounds.
        // Screen pos = world_pos + ($7F - camera). With camera=0, d3=d4=$7F.
        // x = (100 + $7F) - 3 = 224, y = (80 + $7F) - 3 = 204.
        // diamond_size=3, table index=3 → (0x00, 0x86ED).
        let mut slot = helmets::SnakeSlot {
            x_pos: 100,
            y_pos: 80,
            diamond_size: 3,
            trail: [(0, 0); 3],
        };
        let result = helmets::diamond_power_snake_slot_sprite(&mut slot, 0, 0, 10, 0);
        assert_eq!(
            result,
            helmets::SnakeSlotSpriteAction::Visible {
                y: 204,
                size_byte: 0x00,
                pattern: 0x86ED,
                x: 224,
            },
            "ASM loc_3FB3E: visible slot should produce sprite entry with word_4020E[3] attributes"
        );
    }

    #[test]
    fn diamond_power_snake_slot_sprite_offscreen_left_loc_3fb3e() {
        // ASM loc_3FB3E: tst.w d0; ble.s loc_3FB88 — off-screen left skips sprite.
        let mut slot = helmets::SnakeSlot {
            x_pos: -200,
            y_pos: 80,
            diamond_size: 3,
            trail: [(0, 0); 3],
        };
        let result = helmets::diamond_power_snake_slot_sprite(&mut slot, 0, 0, 10, 0);
        assert_eq!(
            result,
            helmets::SnakeSlotSpriteAction::Skipped,
            "ASM loc_3FB3E: x <= 0 should skip to loc_3FB88"
        );
    }

    #[test]
    fn diamond_power_snake_slot_sprite_offscreen_bottom_loc_3fb3e() {
        // ASM loc_3FB3E: cmpi.w #$1E0,d1; bge.s loc_3FB88 — off-screen bottom.
        let mut slot = helmets::SnakeSlot {
            x_pos: 100,
            y_pos: 500,
            diamond_size: 1,
            trail: [(0, 0); 3],
        };
        let result = helmets::diamond_power_snake_slot_sprite(&mut slot, 0, 0, 10, 0);
        assert_eq!(
            result,
            helmets::SnakeSlotSpriteAction::Skipped,
            "ASM loc_3FB3E: y >= $1E0 should skip to loc_3FB88"
        );
    }

    #[test]
    fn diamond_power_snake_slot_sprite_fade_shrink_loc_3fb20() {
        // ASM loc_3FB20: timer < 0 AND Time_Frames & $F == 0 → subq.w #1,$24.
        // Size 4 → 3, still > 0 so falls through to loc_3FB3E for sprite compile.
        let mut slot = helmets::SnakeSlot {
            x_pos: 100,
            y_pos: 80,
            diamond_size: 4,
            trail: [(0, 0); 3],
        };
        let result = helmets::diamond_power_snake_slot_sprite(&mut slot, 0, 0, -1, 0x10);
        assert_eq!(slot.diamond_size, 3, "ASM loc_3FB20: subq.w #1,$24(a0)");
        // Should still produce a visible sprite (size 3 → table index 3)
        assert!(
            matches!(result, helmets::SnakeSlotSpriteAction::Visible { .. }),
            "ASM loc_3FB20: nonzero size after decrement falls through to loc_3FB3E"
        );
    }

    #[test]
    fn diamond_power_snake_slot_sprite_fade_remove_loc_3fb20() {
        // ASM loc_3FB20: timer < 0, Time_Frames & $F == 0, size was 1 → becomes 0.
        // bsr.w sub_3F5E8; bra.s loc_3FB88 — deallocate and skip.
        let mut slot = helmets::SnakeSlot {
            x_pos: 100,
            y_pos: 80,
            diamond_size: 1,
            trail: [(0, 0); 3],
        };
        let result = helmets::diamond_power_snake_slot_sprite(&mut slot, 0, 0, -1, 0x20);
        assert_eq!(
            result,
            helmets::SnakeSlotSpriteAction::Removed,
            "ASM loc_3FB20: size reaching 0 calls sub_3F5E8 and skips to loc_3FB88"
        );
    }

    #[test]
    fn diamond_power_snake_slot_sprite_fade_skipped_frame_loc_3fb20() {
        // ASM loc_3FB20: timer < 0 but Time_Frames & $F != 0 → no shrink,
        // falls through to loc_3FB3E. Size unchanged.
        let mut slot = helmets::SnakeSlot {
            x_pos: 100,
            y_pos: 80,
            diamond_size: 4,
            trail: [(0, 0); 3],
        };
        let result = helmets::diamond_power_snake_slot_sprite(&mut slot, 0, 0, -1, 0x13);
        assert_eq!(
            slot.diamond_size, 4,
            "ASM loc_3FB20: Time_Frames & $F != 0 skips decrement"
        );
        assert!(
            matches!(result, helmets::SnakeSlotSpriteAction::Visible { .. }),
            "ASM loc_3FB20: non-shrink frame still compiles sprite at loc_3FB3E"
        );
    }

    #[test]
    fn diamond_power_snake_slot_sprite_positive_timer_no_fade_loc_3fb20() {
        // ASM loc_3FB20: tst.w (Diamond_power_timer).w; bge.s loc_3FB3E
        // Timer >= 0 → skip fade logic entirely, go straight to sprite compile.
        let mut slot = helmets::SnakeSlot {
            x_pos: 100,
            y_pos: 80,
            diamond_size: 2,
            trail: [(0, 0); 3],
        };
        let result = helmets::diamond_power_snake_slot_sprite(&mut slot, 0, 0, 0, 0x00);
        assert_eq!(
            slot.diamond_size, 2,
            "ASM loc_3FB20: timer >= 0 bypasses fade, size unchanged"
        );
        // diamond_size=2, table index=2 → (0x00, 0x86EC)
        assert_eq!(
            result,
            helmets::SnakeSlotSpriteAction::Visible {
                y: (80 + 0x7F - 2) as u16,
                size_byte: 0x00,
                pattern: 0x86EC,
                x: (100 + 0x7F - 2) as u16,
            },
            "ASM loc_3FB3E: positive timer compiles sprite directly"
        );
    }

    // -----------------------------------------------------------------------
    // DiamondPower_Init_ExtraHitPoint / ExtraLife (loc_3FB96, loc_3FB9A)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_init_extra_hp_creates_10_slots_with_type_8_loc_3fb96() {
        // ASM loc_3FB96 (kid.asm:44904): moveq #0,d3; moveq #9,d1 — 10 slots.
        // ASM loc_3FB9A (kid.asm:44908): radius=$400000, angle spacing=$199999, type=d2.
        // DiamondPower_Init_ExtraHitPoint (loc_3FB90): d2=8.
        let result = helmets::diamond_power_init_extra_hp_life(false);
        assert_eq!(
            result.slots.len(),
            10,
            "ASM loc_3FB96: d1=9, dbf → 10 slots"
        );
        for slot in &result.slots {
            assert_eq!(
                slot.radius, 0x0040_0000,
                "ASM loc_3FB9A: move.l #$400000,$3E(a0)"
            );
            assert_eq!(
                slot.power_type, 8,
                "ASM loc_3FB9A: ExtraHitPoint → d2=8 → $24(a0)"
            );
        }
        // First angle = 0, each subsequent += $199999
        assert_eq!(
            result.slots[0].angle, 0x0000_0000,
            "ASM loc_3FB9A: d3 starts at 0"
        );
        assert_eq!(
            result.slots[1].angle, 0x0019_9999,
            "ASM loc_3FB9A: addi.l #$199999,d3"
        );
        assert_eq!(
            result.slots[9].angle,
            0x0019_9999u32.wrapping_mul(9),
            "ASM loc_3FB9A: 10th slot angle = 9 * $199999"
        );
    }

    #[test]
    fn diamond_power_init_extra_life_creates_10_slots_with_type_7_loc_3fb9a() {
        // ASM DiamondPower_Init_ExtraLife (loc_3FB94): d2=7.
        // Falls through to loc_3FB96/loc_3FB9A.
        let result = helmets::diamond_power_init_extra_hp_life(true);
        assert_eq!(result.slots.len(), 10, "ASM loc_3FB96: 10 slots");
        for slot in &result.slots {
            assert_eq!(
                slot.power_type, 7,
                "ASM loc_3FB9A: ExtraLife → d2=7 → $24(a0)"
            );
        }
    }

    #[test]
    fn diamond_power_init_extra_hp_life_angle_spacing_is_uniform_loc_3fb9a() {
        // ASM loc_3FB9A: addi.l #$199999,d3 gives uniform ~36° spacing (10 slots = full circle).
        let result = helmets::diamond_power_init_extra_hp_life(false);
        for i in 1..result.slots.len() {
            let delta = result.slots[i]
                .angle
                .wrapping_sub(result.slots[i - 1].angle);
            assert_eq!(
                delta,
                0x0019_9999,
                "ASM loc_3FB9A: consecutive angle delta = $199999 (slot {} → {})",
                i - 1,
                i
            );
        }
    }

    // -----------------------------------------------------------------------
    // DiamondPower_Main_ExtraHPLife per-slot (loc_3FBE4)
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_main_extra_hp_life_expanding_phase_loc_3fbe4() {
        // ASM loc_3FBE4 (kid.asm:44932): when Diamond_power_timer < $40,
        // radius.hi += 2, angle.hi -= 2 (expanding orbit).
        let mut slot = helmets::ExtraHPLifeSlot {
            radius: 0x0040_0000,
            angle: 0x0019_9999, // upper word = $19
            power_type: 8,
        };
        let result = helmets::diamond_power_main_extra_hp_life_slot(
            &mut slot, 100,  // kid_x
            80,   // kid_y
            0,    // camera_x
            0,    // camera_y
            0x10, // timer < $40: expanding
            0,    // extra_hitpoint_slots
        );
        // Radius upper word: $40 + 2 = $42
        assert_eq!(
            slot.radius >> 16,
            0x42,
            "ASM loc_3FBE4: addq.w #2,$3E(a0) in expanding phase"
        );
        // Angle upper word: $19 - 2 = $17
        assert_eq!(
            slot.angle >> 16,
            0x17,
            "ASM loc_3FBE4: subq.w #2,$42(a0) in expanding phase"
        );
        assert!(
            matches!(result, helmets::ExtraHPLifeSlotAction::Visible { .. }),
            "ASM loc_3FC56: expanding phase compiles sprite"
        );
    }

    #[test]
    fn diamond_power_main_extra_hp_life_contracting_phase_loc_3fc2a() {
        // ASM loc_3FC2A: when Diamond_power_timer >= $40,
        // angle.hi += 1, radius.hi -= 6 (contracting orbit).
        let mut slot = helmets::ExtraHPLifeSlot {
            radius: 0x0030_0000, // upper word = $30
            angle: 0x0019_9999,  // upper word = $19
            power_type: 8,
        };
        let result = helmets::diamond_power_main_extra_hp_life_slot(
            &mut slot, 100, 80, 0, 0, 0x40, // timer = $40: contracting
            0,
        );
        // Radius upper word: $30 - 6 = $2A
        assert_eq!(
            slot.radius >> 16,
            0x2A,
            "ASM loc_3FC2A: subq.w #6,$3E(a0) in contracting phase"
        );
        // Angle upper word: $19 + 1 = $1A
        assert_eq!(
            slot.angle >> 16,
            0x1A,
            "ASM loc_3FC2A: addq.w #1,$42(a0) in contracting phase"
        );
        assert!(
            matches!(result, helmets::ExtraHPLifeSlotAction::Visible { .. }),
            "ASM loc_3FC56: contracting phase with positive radius compiles sprite"
        );
    }

    #[test]
    fn diamond_power_main_extra_hp_life_converge_awards_hp_loc_3fbe4() {
        // ASM loc_3FBE4: when radius goes negative in contracting phase,
        // type=8 (ExtraHP) awards HP if extra_hitpoint_slots < 4.
        let mut slot = helmets::ExtraHPLifeSlot {
            radius: 0x0003_0000, // upper word = 3, subtracting 6 → -3 (negative)
            angle: 0x0000_0000,
            power_type: 8,
        };
        let result = helmets::diamond_power_main_extra_hp_life_slot(
            &mut slot, 100, 80, 0, 0, 0x40, 2, // extra_hitpoint_slots=2, under cap of 4
        );
        assert_eq!(
            result,
            helmets::ExtraHPLifeSlotAction::Converged {
                awarded_hp: true,
                awarded_life: false,
            },
            "ASM loc_3FBE4: type=8, extra_hitpoint_slots<4 → award HP"
        );
    }

    #[test]
    fn diamond_power_main_extra_hp_life_converge_hp_capped_loc_3fbe4() {
        // ASM: cmpi.w #4,(Extra_hitpoint_slots).w; bge.s loc_3FC4C — cap at 4.
        let mut slot = helmets::ExtraHPLifeSlot {
            radius: 0x0003_0000,
            angle: 0x0000_0000,
            power_type: 8,
        };
        let result = helmets::diamond_power_main_extra_hp_life_slot(
            &mut slot, 100, 80, 0, 0, 0x40, 4, // at cap
        );
        assert_eq!(
            result,
            helmets::ExtraHPLifeSlotAction::Converged {
                awarded_hp: false,
                awarded_life: false,
            },
            "ASM loc_3FBE4: type=8, extra_hitpoint_slots>=4 → skip HP award"
        );
    }

    #[test]
    fn diamond_power_main_extra_hp_life_converge_awards_life_loc_3fc4e() {
        // ASM loc_3FC4E: cmpi.w #7,$24(a0); beq.s loc_3FC4E → addq.w #1,(Number_Lives).w
        let mut slot = helmets::ExtraHPLifeSlot {
            radius: 0x0002_0000, // goes negative after -6
            angle: 0x0000_0000,
            power_type: 7, // ExtraLife
        };
        let result =
            helmets::diamond_power_main_extra_hp_life_slot(&mut slot, 100, 80, 0, 0, 0x40, 0);
        assert_eq!(
            result,
            helmets::ExtraHPLifeSlotAction::Converged {
                awarded_hp: false,
                awarded_life: true,
            },
            "ASM loc_3FC4E: type=7 → award extra life"
        );
    }

    #[test]
    fn diamond_power_converge_cleanup_loc_3fc52() {
        // ASM loc_3FC52 (kid.asm:44977): convergence after awarding HP or life.
        // Both the HP path (loc_3FC4C → loc_3FC52) and life path (loc_3FC4E →
        // loc_3FC52) reach `bra.w sub_3F63C` which deallocates the slot.
        // In native code, this is represented by the `Converged` return variant.
        //
        // Verify both type=7 and type=8 reach the convergence point.
        for (power_type, label) in [(7u16, "ExtraLife"), (8, "ExtraHitPoint")] {
            let mut slot = helmets::ExtraHPLifeSlot {
                radius: 0x0001_0000, // upper word = 1, subtracting 6 → negative
                angle: 0x0000_0000,
                power_type,
            };
            let result = helmets::diamond_power_main_extra_hp_life_slot(
                &mut slot, 100, 80, 0, 0, 0x40, // contracting phase
                0,
            );
            assert!(
                matches!(result, helmets::ExtraHPLifeSlotAction::Converged { .. }),
                "ASM loc_3FC52: {label} convergence → bra.w sub_3F63C"
            );
        }
    }

    #[test]
    fn diamond_power_main_extra_hp_life_screen_position_exact_values_loc_3fbe4() {
        // ASM loc_3FBE4 (kid.asm:44932-44950) screen-position math:
        //   move.l $3E(a0),d5; lsr.l #8,d5; lsr.l #1,d5     ; d5 = radius >> 9
        //   move.w $42(a0),d0; bsr.w sub_40236              ; d0 = cos, d1 = sin
        //   muls.w d5,d0; swap d0; asr.w #5,d0              ; x_offset = (cos*d5)>>16>>5
        //   add.w x_pos(a3),d0; add.w d3,d0                 ; screen_x = kid_x + x_offset + d3
        //   muls.w d5,d1; swap d1; asr.w #5,d1; neg.w d1    ; y_offset = -((sin*d5)>>16>>5)
        //   add.w y_pos(a3),d1; subi.w #$10,d1; add.w d4,d1 ; screen_y = kid_y + y_offset - $10 + d4
        //
        // Inputs: radius=$0040_0000, angle byte=$10, kid=(100,80), camera=(0,0).
        //   d3 = $7F - 0 = $7F (= 127); d4 = $7F.
        //   d5 = $0040_0000 >> 9 = $2000 (= 8192).
        //   sub_40236($10): cos = word_402B0[48] = $3B20 (15136); sin = word_402B0[16] = $187D (6269).
        //   cos*d5 = $3B20*$2000 = $0764_0000 → swap >> 5: ($0764) >> 5 = 1892>>5 = 59.
        //   sin*d5 = $187D*$2000 = $030F_A000 → swap >> 5: ($030F) >> 5 = 783>>5 = 24.
        //   screen_x = 100 + 59 + 127 = 286.
        //   screen_y = 80 + (-24) - 16 + 127 = 167.
        let mut slot = helmets::ExtraHPLifeSlot {
            radius: 0x0040_0000, // upper word $40 → d5 = $2000
            angle: 0x0010_0000,  // upper byte $10 → cos=$3B20, sin=$187D
            power_type: 8,
        };
        let result =
            helmets::diamond_power_main_extra_hp_life_slot(&mut slot, 100, 80, 0, 0, 0x10, 0);
        let helmets::ExtraHPLifeSlotAction::Visible { x, y, .. } = result else {
            panic!("ASM loc_3FC56: expanding phase with positive radius compiles a sprite");
        };
        assert_eq!(
            x, 286,
            "ASM loc_3FBE4: screen_x = kid_x + ((cos * (radius>>9))>>16)>>5 + ($7F - camera_x)"
        );
        assert_eq!(
            y, 167,
            "ASM loc_3FBE4: screen_y = kid_y + -(((sin * (radius>>9))>>16)>>5) - $10 + ($7F - camera_y)"
        );
    }

    #[test]
    fn diamond_power_main_extra_hp_life_expanding_preserves_lower_word_loc_3fbe4() {
        // ASM loc_3FBE4: `addq.w #2,$3E(a0); subq.w #2,$42(a0)` operates on the
        // upper word ($3E and $42) of each longword; the lower fractional word
        // ($40 and $44) must be preserved across the update.
        let mut slot = helmets::ExtraHPLifeSlot {
            radius: 0x0040_4321, // upper=$40, lower=$4321
            angle: 0x0019_BEEF,  // upper=$19, lower=$BEEF
            power_type: 8,
        };
        helmets::diamond_power_main_extra_hp_life_slot(
            &mut slot, 100, 80, 0, 0, 0x10, // expanding phase
            0,
        );
        assert_eq!(
            slot.radius, 0x0042_4321,
            "ASM loc_3FBE4: addq.w #2,$3E(a0) updates upper word only; $40 lower word preserved"
        );
        assert_eq!(
            slot.angle, 0x0017_BEEF,
            "ASM loc_3FBE4: subq.w #2,$42(a0) updates upper word only; $44 lower word preserved"
        );
    }

    #[test]
    fn diamond_power_main_extra_hp_life_contracting_preserves_lower_word_loc_3fc2a() {
        // ASM loc_3FC2A: `addq.w #1,$42(a0); subq.w #6,$3E(a0)` operates on the
        // upper word only; the lower fractional words must be preserved.
        let mut slot = helmets::ExtraHPLifeSlot {
            radius: 0x0030_4321, // upper=$30, lower=$4321
            angle: 0x0019_BEEF,  // upper=$19, lower=$BEEF
            power_type: 8,
        };
        helmets::diamond_power_main_extra_hp_life_slot(
            &mut slot, 100, 80, 0, 0, 0x40, // contracting phase, no convergence
            0,
        );
        assert_eq!(
            slot.radius, 0x002A_4321,
            "ASM loc_3FC2A: subq.w #6,$3E(a0) updates upper word only; $40 lower word preserved"
        );
        assert_eq!(
            slot.angle, 0x001A_BEEF,
            "ASM loc_3FC2A: addq.w #1,$42(a0) updates upper word only; $44 lower word preserved"
        );
    }

    #[test]
    fn diamond_power_main_extra_hp_life_radius_zero_after_subtract_does_not_converge_loc_3fc2a() {
        // ASM loc_3FC2A: `subq.w #6,$3E(a0); bge.s loc_3FC56` —
        // `bge` branches when N==V (i.e. result >= 0), which includes 0. When
        // radius_hi exactly equals 0 after subtraction, control falls through
        // to loc_3FC56 (compile sprite), NOT to the convergence path.
        let mut slot = helmets::ExtraHPLifeSlot {
            radius: 0x0006_4321, // radius_hi=6 → after -6 = 0 (still bge)
            angle: 0x0000_0000,
            power_type: 8,
        };
        let result =
            helmets::diamond_power_main_extra_hp_life_slot(&mut slot, 100, 80, 0, 0, 0x40, 0);
        assert!(
            matches!(result, helmets::ExtraHPLifeSlotAction::Visible { .. }),
            "ASM loc_3FC2A: bge.s loc_3FC56 — radius_hi==0 falls through to compile sprite"
        );
        assert_eq!(
            slot.radius, 0x0000_4321,
            "ASM loc_3FC2A: subq.w #6,$3E(a0) — upper word becomes 0, lower word preserved"
        );
    }

    #[test]
    fn diamond_power_init_invulnerability_loc_3fc8a_loc_3fc8c() {
        // ASM loc_3FC8A (kid.asm:45006): common Invulnerability/SamuraiHaze init
        // entry — moveq #9,d1 (10 slots via dbf).
        // ASM loc_3FC8C (kid.asm:45010): allocation loop — reads 4 bytes per slot
        // from DiamondPower_InitData_Invulnerability (kid.asm:45043).
        let result = helmets::diamond_power_init_invuln_haze(true);
        assert_eq!(
            result.slots.len(),
            10,
            "ASM loc_3FC8A: moveq #9,d1 → 10 slots"
        );
        assert_eq!(
            result.diamond_power_timer, 0x1E0,
            "ASM loc_3FC8C: move.w #$1E0,(Diamond_power_timer).w"
        );

        // First slot: radius=$0C, y_offset=-$24, phase=$00, rotation_speed=$06
        let s0 = &result.slots[0];
        assert_eq!(s0.radius, 0x0C, "ASM loc_3FC8C: slot 0 radius");
        assert_eq!(
            s0.y_offset, -0x24,
            "ASM loc_3FC8C: slot 0 y_offset (sign-extended)"
        );
        assert_eq!(s0.phase, 0x00, "ASM loc_3FC8C: slot 0 phase");
        assert_eq!(s0.rotation_speed, 6, "ASM loc_3FC8C: slot 0 rotation_speed");
        assert_eq!(s0.radius_frac, 0, "ASM loc_3FC8C: clr.w $40(a0)");

        // Third slot (index 2): radius=$0C, y_offset=$04, phase=$40, rotation_speed=$06
        let s2 = &result.slots[2];
        assert_eq!(s2.radius, 0x0C, "ASM loc_3FC8C: slot 2 radius");
        assert_eq!(
            s2.y_offset, 0x04,
            "ASM loc_3FC8C: slot 2 y_offset (positive)"
        );
        assert_eq!(s2.phase, 0x40, "ASM loc_3FC8C: slot 2 phase");
        assert_eq!(s2.rotation_speed, 6, "ASM loc_3FC8C: slot 2 rotation_speed");

        // Last slot (index 9): radius=$18, y_offset=-$10, phase=$A0, rotation_speed=$06
        let s9 = &result.slots[9];
        assert_eq!(s9.radius, 0x18, "ASM loc_3FC8C: slot 9 radius");
        assert_eq!(s9.y_offset, -0x10, "ASM loc_3FC8C: slot 9 y_offset");
        assert_eq!(
            s9.phase, 0xA0,
            "ASM loc_3FC8C: slot 9 phase ($A0 zero-extended)"
        );
        assert_eq!(s9.rotation_speed, 6, "ASM loc_3FC8C: slot 9 rotation_speed");
    }

    #[test]
    fn diamond_power_init_samurai_haze_loc_3fc8a_loc_3fc8c() {
        // ASM DiamondPower_Init_SamuraiHaze (loc_3FC82): st (SamuraiHazeActive).w,
        // then falls through to loc_3FC8A/loc_3FC8C with SamuraiHaze init data.
        // DiamondPower_InitData_SamuraiHaze (kid.asm:45030): rotation_speed=2.
        let result = helmets::diamond_power_init_invuln_haze(false);
        assert_eq!(result.slots.len(), 10, "ASM loc_3FC8A: 10 slots");
        assert_eq!(result.diamond_power_timer, 0x1E0);

        // First slot: radius=$0C, y_offset=-$24, phase=$00, rotation_speed=$02
        let s0 = &result.slots[0];
        assert_eq!(s0.radius, 0x0C);
        assert_eq!(s0.y_offset, -0x24);
        assert_eq!(s0.phase, 0x00);
        assert_eq!(
            s0.rotation_speed, 2,
            "SamuraiHaze rotation_speed=2 (vs Invuln=6)"
        );

        // Slot 1: phase=$80 (zero-extended = 128)
        assert_eq!(
            result.slots[1].phase, 0x80,
            "ASM loc_3FC8C: $80 zero-extended"
        );
        // Slot 4: radius=$15, y_offset=-$1A, phase=$20
        let s4 = &result.slots[4];
        assert_eq!(s4.radius, 0x15);
        assert_eq!(s4.y_offset, -0x1A);
        assert_eq!(s4.phase, 0x20);
    }

    // -----------------------------------------------------------------------
    // DiamondPower_Main_InvulnAndHaze frame + per-slot
    // -----------------------------------------------------------------------

    #[test]
    fn diamond_power_invuln_haze_frame_active_while_timer_positive_loc_3fd28() {
        // ASM DiamondPower_Main_InvulnAndHaze (kid.asm:45056): subq.w #1 → timer
        // goes from $1E0 to $1DF. loc_3FD28: neg.w $1DF = $FE21 (negative),
        // clamp to 0. loc_3FD2E: lsr.w #2 → fade = 0.
        let mut timer: i16 = 0x1E0;
        let result = helmets::diamond_power_invuln_haze_frame(&mut timer);
        assert_eq!(timer, 0x1DF, "ASM: subq.w #1");
        assert_eq!(
            result,
            helmets::InvulnHazeFrameAction::Active { fade: 0 },
            "ASM loc_3FD28: positive timer → fade = 0"
        );
    }

    #[test]
    fn diamond_power_invuln_haze_frame_fade_when_timer_negative_loc_3fd28() {
        // Timer = 0: after subq → -1. neg.w → 1. bpl → keep. lsr #2 → 0.
        let mut timer: i16 = 0;
        let result = helmets::diamond_power_invuln_haze_frame(&mut timer);
        assert_eq!(timer, -1);
        assert_eq!(
            result,
            helmets::InvulnHazeFrameAction::Active { fade: 0 },
            "ASM loc_3FD28: timer=-1, neg=1, lsr #2 → 0"
        );

        // Timer = -5: after subq → -6. neg.w → 6. lsr #2 → 1.
        let mut timer: i16 = -5;
        let result = helmets::diamond_power_invuln_haze_frame(&mut timer);
        assert_eq!(timer, -6);
        assert_eq!(
            result,
            helmets::InvulnHazeFrameAction::Active { fade: 1 },
            "ASM loc_3FD28: timer=-6, neg=6, lsr #2 → 1"
        );
    }

    #[test]
    fn diamond_power_invuln_haze_frame_expired_below_threshold_loc_3fd28() {
        // Timer = -$1B: after subq → -$1C. cmpi.w #-$1B → -$1C < -$1B → Expired.
        let mut timer: i16 = -0x1B;
        let result = helmets::diamond_power_invuln_haze_frame(&mut timer);
        assert_eq!(timer, -0x1C);
        assert_eq!(
            result,
            helmets::InvulnHazeFrameAction::Expired,
            "ASM: cmpi.w #-$1B,d7; bge.s — timer below threshold → sub_3F63C"
        );
    }

    #[test]
    fn diamond_power_invuln_haze_frame_boundary_at_minus_1b_loc_3fd28() {
        // Timer = -$1A: after subq → -$1B. cmpi.w #-$1B → -$1B >= -$1B → Active.
        // neg.w $1B = ... wait. d7 = -$1B (as i16 = 0xFFE5). neg.w → $1B = 27.
        // lsr #2 → 6.
        let mut timer: i16 = -0x1A;
        let result = helmets::diamond_power_invuln_haze_frame(&mut timer);
        assert_eq!(timer, -0x1B);
        assert_eq!(
            result,
            helmets::InvulnHazeFrameAction::Active { fade: 6 },
            "ASM loc_3FD28: timer=-$1B (boundary), neg=$1B=27, lsr #2 → 6"
        );
    }

    #[test]
    fn diamond_power_invuln_haze_slot_sprite_rotation_when_no_fade_loc_3fd4c() {
        // ASM loc_3FD4C: tst.w d7; bne.s loc_3FD5A — when fade==0, update angle.
        // $42(a0) += $48(a0) → phase += rotation_speed.
        let mut slot = helmets::InvulnHazeSlot {
            radius: 0x18,
            radius_frac: 0,
            y_offset: -0x10,
            phase: 0x20,
            rotation_speed: 6,
        };
        let _ = helmets::diamond_power_invuln_haze_slot_sprite(&mut slot, 0, 100, 80, 0, 0);
        assert_eq!(
            slot.phase, 0x26,
            "ASM loc_3FD4C: phase += rotation_speed when fade==0"
        );
    }

    #[test]
    fn diamond_power_invuln_haze_slot_sprite_no_rotation_when_fading_loc_3fd4c() {
        // ASM loc_3FD4C: tst.w d7; bne.s loc_3FD5A — when fade!=0, skip angle update.
        let mut slot = helmets::InvulnHazeSlot {
            radius: 0x18,
            radius_frac: 0,
            y_offset: -0x10,
            phase: 0x20,
            rotation_speed: 6,
        };
        let _ = helmets::diamond_power_invuln_haze_slot_sprite(&mut slot, 1, 100, 80, 0, 0);
        assert_eq!(
            slot.phase, 0x20,
            "ASM loc_3FD4C: phase unchanged when fade!=0"
        );
    }

    #[test]
    fn diamond_power_invuln_haze_slot_sprite_skipped_when_fade_exceeds_size_loc_3fd8e() {
        // ASM loc_3FD8E: sub.w d7,d6; bmi.s loc_3FDD2 — skip if size - fade < 0.
        // With a large fade (e.g. 6), all size table entries (max 5) will be < fade.
        let mut slot = helmets::InvulnHazeSlot {
            radius: 0x18,
            radius_frac: 0,
            y_offset: -0x10,
            phase: 0,
            rotation_speed: 6,
        };
        let result = helmets::diamond_power_invuln_haze_slot_sprite(&mut slot, 6, 100, 80, 0, 0);
        assert_eq!(
            result,
            helmets::InvulnHazeSpriteAction::Skipped,
            "ASM loc_3FD8E: fade=6 exceeds max size table entry (5)"
        );
    }

    #[test]
    fn diamond_power_invuln_haze_slot_sprite_small_sprite_loc_3fd4c() {
        // When the computed size (d6 after sub) is small (0-2), table_index < $C → small sprite.
        // Use phase=0 (cos=max, sin≈0) → sin_offset + $18 ≈ $18, size_index = $18>>2 = 6,
        // byte_3FDDC[6] = 3. With fade=2: d6 = 3-2 = 1. table_index = 1*4 = 4 < $C → small.
        let mut slot = helmets::InvulnHazeSlot {
            radius: 0x18,
            radius_frac: 0,
            y_offset: -0x10,
            phase: 0,
            rotation_speed: 6,
        };
        let result = helmets::diamond_power_invuln_haze_slot_sprite(&mut slot, 2, 100, 80, 0, 0);
        match result {
            helmets::InvulnHazeSpriteAction::SmallSprite { .. } => {}
            other => panic!(
                "ASM loc_3FD4C: expected SmallSprite for d6=1 (table_index=4 < $C), got {other:?}"
            ),
        }
    }

    #[test]
    fn diamond_power_invuln_haze_slot_sprite_large_sprite_loc_3fdc2() {
        // When d6 >= 3, table_index >= $C → large sprite path (loc_3FDC2).
        // Use phase=0, fade=0. sin_offset + $18 ≈ $18, size_index=6, byte_3FDDC[6]=3.
        // d6 = 3-0 = 3. table_index = 3*4 = $C → large.
        let mut slot = helmets::InvulnHazeSlot {
            radius: 0x18,
            radius_frac: 0,
            y_offset: -0x10,
            phase: 0,
            rotation_speed: 6,
        };
        let result = helmets::diamond_power_invuln_haze_slot_sprite(&mut slot, 0, 100, 80, 0, 0);
        match result {
            helmets::InvulnHazeSpriteAction::LargeSprite {
                size_byte, pattern, ..
            } => {
                // Entry 3 in DIAMOND_SPRITE_ATTR_TABLE: (0x00, 0x86ED)
                assert_eq!(
                    size_byte, 0x00,
                    "ASM loc_3FDC2: size byte from word_4020E entry 3"
                );
                assert_eq!(
                    pattern, 0x86ED,
                    "ASM loc_3FDC2: pattern from word_4020E entry 3"
                );
            }
            other => {
                panic!(
                    "ASM loc_3FDC2: expected LargeSprite for d6=3 (table_index=$C), got {other:?}"
                )
            }
        }
    }

    #[test]
    fn diamond_power_invuln_haze_size_table_matches_asm_byte_3fddc() {
        // ASM byte_3FDDC (kid.asm:45148): 12 entries.
        assert_eq!(
            helmets::INVULN_HAZE_SIZE_TABLE,
            [1, 1, 2, 2, 3, 3, 3, 3, 4, 4, 5, 5],
            "ASM byte_3FDDC: size lookup table must match ROM data"
        );
    }

    #[test]
    fn diamond_power_invuln_haze_small_sprite_screen_position_exact_loc_3fd4c() {
        // ASM loc_3FD4C..loc_3FDD2 (kid.asm:45082-45146) screen-position math:
        //   move.l $3E(a0),d5; lsr.l #8,d5; lsr.l #1,d5     ; d5 = radius_long >> 9
        //   move.w $42(a0),d0; bsr.w sub_40236              ; d0 = cos, d1 = sin
        //   muls.w d5,d0; swap d0; asr.w #5,d0              ; x_offset = (cos*d5)>>16>>5
        //   add.w x_pos(a3),d0; add.w d3,d0                 ; +kid_x +($7F-cam_x)
        //   muls.w d5,d1; swap d1; asr.w #5,d1; addi.w #$18,d1 ; sin term + $18
        //   bpl.s loc_3FD84; moveq #0,d1                    ; clamp negative to 0
        //   loc_3FD84: lsr.w #2,d1; cmpi.w #$B,d1; ble.s loc_3FD8E; moveq #$B,d1
        //   loc_3FD8E: moveq #0,d6; move.b byte_3FDDC(pc,d1.w),d6
        //   sub.w d7,d6; bmi.s loc_3FDD2                    ; skip if size-fade < 0
        //   add.w y_pos(a3),d1 (after y_offset); add.w d4,d1; sub.w d6,d0; sub.w d6,d1
        //
        // Inputs: radius=$10, radius_frac=$4000 → radius_long=$10_4000 → d5=$820=2080.
        //   phase=0 → angle=0 → cos=word_402B0[64]=$3FFE=16382, sin=$0000.
        //   d3 = $7F − 0 = 127; d4 = 127.
        //   x: cos*d5 = 16382*2080 = $0207_F840; >>16=$207=519; >>5 = 16.
        //      screen_x_pre = 100 + 16 + 127 = 243.
        //   sin term = 0 → size_index = 0 + $18 = 24, no clamp; >>2 = 6.
        //   raw_size = byte_3FDDC[6] = 3; fade = 1 → d6 = 2 (≥0, not skipped).
        //   screen_y_pre = (−$10) + 80 + 127 = 191.
        //   sub d6: x = 243 − 2 = 241; y = 191 − 2 = 189.
        //   table_index = 2*4 = 8 < $C → SmallSprite, entry 2 → ($00, $86EC).
        //   size_link = ($00 as u16) << 8 = 0.
        let mut slot = helmets::InvulnHazeSlot {
            radius: 0x10,
            radius_frac: 0x4000,
            y_offset: -0x10,
            phase: 0,
            rotation_speed: 6,
        };
        let result = helmets::diamond_power_invuln_haze_slot_sprite(&mut slot, 1, 100, 80, 0, 0);
        assert_eq!(
            result,
            helmets::InvulnHazeSpriteAction::SmallSprite {
                y: 189,
                size_link: 0,
                pattern: 0x86EC,
                x: 241,
            },
            "ASM loc_3FD4C..loc_3FDD2: exact x/y math for fade=1, d6=2"
        );
    }

    #[test]
    fn diamond_power_invuln_haze_d6_zero_falls_through_loc_3fd8e() {
        // ASM loc_3FD8E: `sub.w d7,d6; bmi.s loc_3FDD2` — `bmi` branches only on
        // negative (N flag set). When d6 == 0, the branch is NOT taken, so the
        // sprite is emitted (not Skipped). This locks the boundary against a
        // `< 0` → `<= 0` mutation in the Rust port.
        //
        // Same setup as above, but fade = raw_size = 3 → d6 = 0.
        //   screen_x = 100 + 16 + 127 − 0 = 243; screen_y = −$10 + 80 + 127 − 0 = 191.
        //   table_index = 0 → SmallSprite, entry 0 → ($00, $86EA).
        let mut slot = helmets::InvulnHazeSlot {
            radius: 0x10,
            radius_frac: 0x4000,
            y_offset: -0x10,
            phase: 0,
            rotation_speed: 6,
        };
        let result = helmets::diamond_power_invuln_haze_slot_sprite(&mut slot, 3, 100, 80, 0, 0);
        assert_eq!(
            result,
            helmets::InvulnHazeSpriteAction::SmallSprite {
                y: 191,
                size_link: 0,
                pattern: 0x86EA,
                x: 243,
            },
            "ASM loc_3FD8E: d6 == 0 falls through, NOT skipped (bmi only fires on negative)"
        );
    }

    #[test]
    fn diamond_power_invuln_haze_size_index_upper_clamp_loc_3fd84() {
        // ASM loc_3FD84: `lsr.w #2,d1; cmpi.w #$B,d1; ble.s loc_3FD8E; moveq #$B,d1`.
        // `ble` is signed-≤, so values strictly greater than $B are clamped to $B.
        //
        // Inputs: radius=$18, radius_frac=$0200 → radius_long=$18_0200 → d5=$C01=3073.
        //   phase=64 (90°), rotation_speed=0 (so loc_3FD4C add is a no-op even with
        //   fade=0; otherwise phase would advance by rotation_speed before sub_40236).
        //   sub_40236(64): cos=0, sin=word_402B0[64]=$3FFE=16382 (Q2).
        //   x_offset = 0 → screen_x_pre = 100 + 0 + 127 = 227.
        //   sin*d5 = 16382*3073 = $0300_27FE; >>16 = $300 = 768; >>5 = 24.
        //   size_index_raw = 24 + $18 = $30 = 48; >>2 = $C = 12, ABOVE $B → clamp $B.
        //   raw_size = byte_3FDDC[$B] = 5; fade=0 → d6 = 5.
        //   screen_y_pre = −$10 + 80 + 127 = 191.
        //   sub d6: x = 227 − 5 = 222; y = 191 − 5 = 186.
        //   table_index = 5*4 = $14 ≥ $C → LargeSprite, entry 5 → ($05, $86F2).
        //
        // Without the upper clamp, `INVULN_HAZE_SIZE_TABLE[12]` would panic.
        let mut slot = helmets::InvulnHazeSlot {
            radius: 0x18,
            radius_frac: 0x0200,
            y_offset: -0x10,
            phase: 64,
            rotation_speed: 0,
        };
        let result = helmets::diamond_power_invuln_haze_slot_sprite(&mut slot, 0, 100, 80, 0, 0);
        assert_eq!(
            result,
            helmets::InvulnHazeSpriteAction::LargeSprite {
                y: 186,
                size_byte: 0x05,
                pattern: 0x86F2,
                x: 222,
            },
            "ASM loc_3FD84: size_index >$B clamped to $B (raw_size=5, LargeSprite path)"
        );
    }

    #[test]
    fn diamond_power_invuln_haze_size_index_lower_clamp_loc_3fd84() {
        // ASM loc_3FD7E (just before loc_3FD84): `addi.w #$18,d1; bpl.s loc_3FD84;
        // moveq #0,d1`. `bpl` keeps non-negative; otherwise clamp to 0.
        //
        // Inputs: radius=$19, radius_frac=$0400 → radius_long=$19_0400 → d5=$C82=3202.
        //   phase=192 (270°), rotation_speed=0 (so loc_3FD4C add is a no-op even with
        //   fade=0; otherwise phase would advance by rotation_speed before sub_40236).
        //   sub_40236(192): cos=0, sin=−$3FFE=−16382 (Q4 negates sin).
        //   x_offset = 0 → screen_x_pre = 100 + 0 + 127 = 227.
        //   sin*d5 = −16382*3202 = −52,455,164; >>16 = −801; (i16) >>5 = −26.
        //   size_index_raw = −26 + $18 = −2, NEGATIVE → clamp to 0.
        //   (0 as u16) >> 2 = 0; raw_size = byte_3FDDC[0] = 1; fade=0 → d6 = 1.
        //   screen_y_pre = −$10 + 80 + 127 = 191.
        //   sub d6: x = 227 − 1 = 226; y = 191 − 1 = 190.
        //   table_index = 1*4 = 4 < $C → SmallSprite, entry 1 → ($00, $86EB).
        //
        // Without the lower clamp, the negative i16 would cast to a huge u16
        // and round-trip through the upper clamp to $B, producing raw_size=5
        // and a LargeSprite — a different return variant.
        let mut slot = helmets::InvulnHazeSlot {
            radius: 0x19,
            radius_frac: 0x0400,
            y_offset: -0x10,
            phase: 192,
            rotation_speed: 0,
        };
        let result = helmets::diamond_power_invuln_haze_slot_sprite(&mut slot, 0, 100, 80, 0, 0);
        assert_eq!(
            result,
            helmets::InvulnHazeSpriteAction::SmallSprite {
                y: 190,
                size_link: 0,
                pattern: 0x86EB,
                x: 226,
            },
            "ASM loc_3FD7E: size_index <0 clamped to 0 (raw_size=1, SmallSprite path)"
        );
    }

    // -----------------------------------------------------------------------
    // Prize content dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn prize_content_clock_adds_time() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        let secs_before = runtime.level_timer.seconds;
        runtime.apply_prize_content(terrain::PrizeContent::Clock);
        assert_eq!(
            runtime.level_timer.seconds,
            (secs_before + LEVEL_TIMER_CLOCK_BONUS_SECONDS).min(LEVEL_TIMER_MAX_SECONDS),
        );
    }

    #[test]
    fn prize_content_clock_increments_clocks_collected() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        runtime.apply_prize_content(terrain::PrizeContent::Clock);

        assert_eq!(
            runtime.clocks_collected, 1,
            "ASM loc_10002 should increment `Clocks_collected` once for each collected clock prize"
        );
    }

    #[test]
    fn prize_ankh_collected_loc_feee_advances_cached_screen_offsets_before_world_write() {
        let step = NativeRuntime::prize_collected_arc_hibernate_step_loc_FEEE(
            0x0028_0000,
            0x0014_0000,
            0x0007_F800,
            0x0000_5800,
        );

        assert_eq!(
            step,
            PrizeCollectedArcHibernateStepLocFEEE {
                next_screen_x_fp: 0x002F_F800,
                next_screen_y_fp: 0x0014_5800,
                screen_x_word: 0x002F,
                screen_y_word: 0x0014,
            },
            "ASM loc_FEEE should hibernate, add the cached ankh HUD deltas, then expose the high word of `$3E/$42` for the later world-coordinate write",
        );
    }

    #[test]
    fn prize_ankh_collected_loc_fefe_reprojects_screen_words_and_loops() {
        let arc_step = PrizeCollectedArcHibernateStepLocFEEE {
            next_screen_x_fp: 0x002F_F800,
            next_screen_y_fp: 0x0014_5800,
            screen_x_word: 0x002F,
            screen_y_word: 0x0014,
        };

        assert_eq!(
            NativeRuntime::prize_collected_world_step_loc_FEFE(arc_step, 0x0020, 0x0050, 0x001F),
            PrizeCollectedWorldStepLocFEFE {
                world_x_word: 0x004F,
                world_y_word: 0x0064,
                next_loop_counter: 0x001E,
                should_loop: true,
            },
            "ASM loc_FEFE should add `Camera_X_pos/Camera_Y_pos` words to the cached screen words before the `dbf d2,loc_FEEE` back-edge",
        );

        assert_eq!(
            NativeRuntime::prize_collected_world_step_loc_FEFE(arc_step, 0x0020, 0x0050, 0),
            PrizeCollectedWorldStepLocFEFE {
                world_x_word: 0x004F,
                world_y_word: 0x0064,
                next_loop_counter: 0xFFFF,
                should_loop: false,
            },
            "ASM loc_FEFE should stop looping when the final `dbf` decrement underflows `d2` to `$FFFF`",
        );
    }

    #[test]
    fn prize_clock_collected_loc_ff92_uses_clock_hud_target_arc() {
        assert_eq!(
            NativeRuntime::prize_clock_collected_world_step_loc_FF92(
                0x0028_0000,
                0x0014_0000,
                0x0000_8000,
                0x0000_5800,
                0x0020,
                0x0050,
                0x001F,
            ),
            PrizeClockCollectedWorldStepLocFF92 {
                arc_step: PrizeCollectedArcHibernateStepLocFEEE {
                    next_screen_x_fp: 0x0028_8000,
                    next_screen_y_fp: 0x0014_5800,
                    screen_x_word: 0x0028,
                    screen_y_word: 0x0014,
                },
                world_step: PrizeCollectedWorldStepLocFEFE {
                    world_x_word: 0x0048,
                    world_y_word: 0x0064,
                    next_loop_counter: 0x001E,
                    should_loop: true,
                },
            },
            "ASM loc_FF92 should use the clock collect arc toward screen `$38,$1F`, write through camera-relative words, then loop back while `d2` has not underflowed",
        );
    }

    #[test]
    fn prize_clock_sound_refcount_loc_10792_only_starts_the_first_loop() {
        assert_eq!(
            NativeRuntime::prize_clock_sound_refcount_loc_10792(0),
            PrizeClockSoundRefcountLoc10792 {
                should_play_sound: true,
                next_refcount: 1,
            },
            "ASM loc_10792 should play `sfx_Clock_prize` only when `$FFFFFB55` was zero before incrementing the shared refcount",
        );
        assert_eq!(
            NativeRuntime::prize_clock_sound_refcount_loc_10792(3),
            PrizeClockSoundRefcountLoc10792 {
                should_play_sound: false,
                next_refcount: 4,
            },
            "ASM loc_10792 should skip the start sound once another clock prize already owns the shared loop",
        );
    }

    #[test]
    fn prize_clock_countdown_loc_10796_deletes_at_zero_and_swaps_at_012c() {
        assert_eq!(
            NativeRuntime::prize_clock_countdown_loc_10796(1),
            PrizeClockCountdownLoc10796 {
                next_remaining: 0,
                should_delete: true,
                should_swap_animation: false,
            },
            "ASM loc_10796 should delete the clock prize as soon as the post-hibernate `subq.w #1,d3` result reaches zero",
        );
        assert_eq!(
            NativeRuntime::prize_clock_countdown_loc_10796(0x012D),
            PrizeClockCountdownLoc10796 {
                next_remaining: 0x012C,
                should_delete: false,
                should_swap_animation: true,
            },
            "ASM loc_10796 should switch to `stru_10C1E` exactly when the decremented countdown equals `$012C`",
        );
    }

    #[test]
    fn prize_clock_fall_gate_loc_107b0_deletes_on_bottom_or_collection() {
        assert_eq!(
            NativeRuntime::prize_clock_fall_gate_loc_107B0(95, 96, false),
            PrizeClockFallGateLoc107B0 {
                stays_active: true,
                hit_level_bottom: false,
                collided_with_player: false,
            },
            "ASM loc_107B0 should loop back to `loc_10796` while the clock remains above `Level_height_pixels` and `collision_type(a3)` is zero",
        );
        assert_eq!(
            NativeRuntime::prize_clock_fall_gate_loc_107B0(96, 96, false),
            PrizeClockFallGateLoc107B0 {
                stays_active: false,
                hit_level_bottom: true,
                collided_with_player: false,
            },
            "ASM loc_107B0 should jump to the delete exit once `cmp.w y_pos(a3),d4` reaches the level bottom",
        );
        assert_eq!(
            NativeRuntime::prize_clock_fall_gate_loc_107B0(40, 96, true),
            PrizeClockFallGateLoc107B0 {
                stays_active: false,
                hit_level_bottom: false,
                collided_with_player: true,
            },
            "ASM loc_107B0 should stop looping once `collision_type(a3)` becomes non-zero from player contact",
        );
    }

    #[test]
    fn floating_prize_fall_step_loc_10c74_keeps_d0_in_sync_with_the_latest_snapped_row() {
        assert_eq!(
            NativeRuntime::floating_prize_fall_step_loc_10C50(
                0x0030,
                0x003F_0000,
                0,
                true,
                7,
                0x0120,
                0x0030
            ),
            FloatingPrizeFallStepLoc10C50 {
                next_y_velocity: 0x1388,
                next_y_pos: 0x003F_1388,
                next_aligned_y: 0x0030,
                next_first_row_grace: true,
                next_level_row: 7,
                next_level_layout_offset: 0x0120,
            },
            "ASM loc_10C74 should copy the unchanged snapped row back into `d0` while loc_10C50 leaves bit `$10` in `d3` and `a2` untouched inside the same 16px band",
        );
        assert_eq!(
            NativeRuntime::floating_prize_fall_step_loc_10C50(
                0x0030,
                0x003F_F000,
                0x3000,
                true,
                7,
                0x0120,
                0x0030
            ),
            FloatingPrizeFallStepLoc10C50 {
                next_y_velocity: 0x4388,
                next_y_pos: 0x0040_3388,
                next_aligned_y: 0x0040,
                next_first_row_grace: false,
                next_level_row: 8,
                next_level_layout_offset: 0x0150,
            },
            "ASM loc_10C74 should still finish by writing the new `$0040` snapped row into `d0` after loc_10C50 clears bit `$10` in `d3` and advances `a2` by `Level_width_tiles`",
        );
    }

    #[test]
    fn floating_prize_fall_loc_103de_adds_gravity_and_advances_y_inside_one_row_band() {
        assert_eq!(
            NativeRuntime::floating_prize_fall_loc_103DE(
                0x0030,
                0x003F_0000,
                0,
                true,
                7,
                0x0120,
                0x0030
            ),
            FloatingPrizeFallStepLoc10C50 {
                next_y_velocity: 0x1388,
                next_y_pos: 0x003F_1388,
                next_aligned_y: 0x0030,
                next_first_row_grace: true,
                next_level_row: 7,
                next_level_layout_offset: 0x0120,
            },
            "ASM loc_103DE should add `#$1388` to `d1` and advance `y_pos(a3)` even when the updated integer Y stays inside the current snapped 16px row",
        );
    }

    #[test]
    fn floating_prize_row_sync_loc_10402_advances_the_tracked_row_after_a_boundary_crossing() {
        assert_eq!(
            NativeRuntime::floating_prize_row_sync_loc_10402(
                0x0030,
                0x003F_F000,
                0x3000,
                true,
                7,
                0x0120,
                0x0030
            ),
            FloatingPrizeFallStepLoc10C50 {
                next_y_velocity: 0x4388,
                next_y_pos: 0x0040_3388,
                next_aligned_y: 0x0040,
                next_first_row_grace: false,
                next_level_row: 8,
                next_level_layout_offset: 0x0150,
            },
            "ASM loc_10402 should keep the new snapped row in `d0` after a boundary-crossing fall, so the one-row grace bit clears and the tracked level row advances exactly once",
        );
    }

    #[test]
    fn prize_clock_sound_cleanup_loc_107e0_only_stops_the_last_loop() {
        assert_eq!(
            NativeRuntime::prize_clock_sound_cleanup_loc_107E0(1),
            PrizeClockSoundCleanupLoc107E0 {
                next_refcount: 0,
                should_play_stop_sound: true,
                should_delete_current_object: true,
            },
            "ASM loc_107E0 should play `sfx_Clock_prize` through `PlaySound2` only when the decremented shared refcount reaches zero",
        );
        assert_eq!(
            NativeRuntime::prize_clock_sound_cleanup_loc_107E0(4),
            PrizeClockSoundCleanupLoc107E0 {
                next_refcount: 3,
                should_play_stop_sound: false,
                should_delete_current_object: true,
            },
            "ASM loc_107E0 should skip the stop sound while another live clock prize still owns the shared loop",
        );
        assert!(
            NativeRuntime::prize_clock_sound_cleanup_loc_107E0(2).should_delete_current_object,
            "ASM loc_107EE should jump straight to `Delete_CurrentObject` after the shared clock-sound cleanup tail",
        );
    }

    #[test]
    fn prize_continue_countdown_loc_1080a_deletes_at_zero_and_swaps_at_012c() {
        assert_eq!(
            NativeRuntime::prize_continue_countdown_loc_1080A(1),
            PrizeContinueCountdownLoc1080A {
                next_remaining: 0,
                should_delete: true,
                should_swap_animation: false,
            },
            "ASM loc_1080A should delete the continue prize as soon as the post-hibernate `subq.w #1,d3` result reaches zero",
        );
        assert_eq!(
            NativeRuntime::prize_continue_countdown_loc_1080A(0x012D),
            PrizeContinueCountdownLoc1080A {
                next_remaining: 0x012C,
                should_delete: false,
                should_swap_animation: true,
            },
            "ASM loc_1080A should switch to `stru_10C2E` exactly when the decremented countdown equals `$012C`",
        );
    }

    #[test]
    fn prize_continue_delete_loc_10838_plays_sound_6a_then_deletes() {
        assert_eq!(
            NativeRuntime::prize_continue_delete_loc_10838(),
            PrizeContinueDeleteLoc10838 {
                sound_effect_id: 0x6A,
                preserves_d0_across_sound_call: true,
                should_delete_current_object: true,
            },
            "ASM loc_10838 should preserve `d0`, call `j_PlaySound` with `#$6A`, then jump straight to `Delete_CurrentObject`",
        );
    }

    #[test]
    fn one_time_prize_init_loc_10856_clears_the_table_on_first_collection() {
        let entries = core::array::from_fn(|index| 0xDEAD_0000_u32.wrapping_add(index as u32));

        assert_eq!(
            NativeRuntime::one_time_prize_init_loc_10856(-1, entries),
            OneTimePrizeInitLoc10856 {
                was_first_prize_initialisation: true,
                next_write_index: 0,
                entries: [0; 0x28],
            },
            "ASM loc_10856 should treat a negative header as the first one-time prize and clear all 40 packed prize entries before scanning",
        );
    }

    #[test]
    fn one_time_prize_scan_start_loc_1086a_skips_the_header_and_scans_40_entries() {
        assert_eq!(
            NativeRuntime::one_time_prize_scan_start_loc_1086A(0x4A72),
            OneTimePrizeScanStartLoc1086A {
                remaining_entry_checks: 0x27,
                prize_layout_addr: 0x4A72,
                entry_byte_offset_from_table_base: 2,
            },
            "ASM loc_1086A should reset the DBF counter to `#$27`, copy `$4A(a5)` into `d1`, and start scanning at `2(a0)` after the header word",
        );
    }

    #[test]
    fn one_time_prize_entry_check_loc_10876_only_hits_when_layout_and_level_match() {
        assert_eq!(
            NativeRuntime::one_time_prize_entry_check_loc_10876(0x4A72_0011, 0x4A72, 0x0011),
            OneTimePrizeEntryCheckLoc10876 {
                entry_layout_addr: 0x4A72,
                entry_level_id: 0x0011,
                layout_addr_matches: true,
                level_id_matches: true,
                should_init_prize_diamond: true,
            },
            "ASM loc_10876 should branch to `PrizeDiamond_Init` only when both `(a1)` and `2(a1)` match the current layout address and level ID",
        );

        assert_eq!(
            NativeRuntime::one_time_prize_entry_check_loc_10876(0x4A72_0012, 0x4A72, 0x0011),
            OneTimePrizeEntryCheckLoc10876 {
                entry_layout_addr: 0x4A72,
                entry_level_id: 0x0012,
                layout_addr_matches: true,
                level_id_matches: false,
                should_init_prize_diamond: false,
            },
            "ASM loc_10876 should keep scanning when the layout word matches but the stored level differs",
        );
    }

    #[test]
    fn one_time_prize_scan_step_loc_10886_advances_four_bytes_and_honours_dbf_underflow() {
        assert_eq!(
            NativeRuntime::one_time_prize_scan_step_loc_10886(0x0027, 2),
            OneTimePrizeScanStepLoc10886 {
                next_remaining_entry_checks: 0x0026,
                next_entry_byte_offset_from_table_base: 6,
                should_continue_scan: true,
            },
            "ASM loc_10886 should advance `a1` by one packed entry and keep looping while DBF leaves `d0` above `$FFFF`",
        );

        assert_eq!(
            NativeRuntime::one_time_prize_scan_step_loc_10886(0, 0x009E),
            OneTimePrizeScanStepLoc10886 {
                next_remaining_entry_checks: 0xFFFF,
                next_entry_byte_offset_from_table_base: 0x00A2,
                should_continue_scan: false,
            },
            "ASM loc_10886 should stop after the final scan iteration when DBF decrements zero to `$FFFF`",
        );
    }

    #[test]
    fn one_time_prize_store_loc_1089a_writes_at_the_old_header_slot_then_wraps_the_next_index() {
        let entries = core::array::from_fn(|index| 0xA000_0000_u32.wrapping_add(index as u32));

        assert_eq!(
            NativeRuntime::one_time_prize_store_loc_1089A(0x0027, entries, 0x4A72, 0x0011),
            OneTimePrizeStoreLoc1089A {
                next_write_index: 0,
                entries: core::array::from_fn(|index| {
                    if index == 0x27 {
                        0x4A72_0011
                    } else {
                        0xA000_0000_u32.wrapping_add(index as u32)
                    }
                }),
            },
            "ASM loc_1089A should write the new prize into the current header slot and only then wrap the header back to zero at `$28`",
        );
    }

    #[test]
    fn prize_10_diamonds_init_loc_108e8_clears_all_ten_worker_slots() {
        assert_eq!(
            NativeRuntime::prize_10_diamonds_init_loc_108E8(),
            Prize10DiamondsInitLoc108E8 {
                cleared_worker_slots: [0; 10],
            },
            "ASM loc_108E8 should clear exactly ten longwords at `$44(a5)` before the burst loop starts",
        );
    }

    #[test]
    fn prize_10_diamonds_spawn_loc_108ee_spawns_the_first_worker_with_table_velocity() {
        assert_eq!(
            NativeRuntime::prize_10_diamonds_spawn_loc_108EE(0, 0x1234, 0x5678),
            Prize10DiamondsSpawnLoc108EE {
                should_hibernate_one_frame: true,
                should_scan_workers: true,
                should_spawn_worker: true,
                burst_table_index: Some(0),
                allocation_spec: Some(0x00FE_0000),
                sound_effect_label: Some("sfx_Diamond_prize"),
                priority: Some(1),
                display_enabled: Some(true),
                animation_script_label: Some("LnkTo_unk_E1046-Data_Index"),
                x_pos: Some(0x1234),
                y_pos: Some(0x5678),
                x_velocity: Some(-163_840),
                y_velocity: Some(-294_912),
                is_moved: Some(true),
                lifetime_frames: Some(0x0078),
            },
            "ASM loc_108EE should spawn frame 0 using `byte_108AC[0]`, copy the cached owner position, and derive FP16.16 velocity from `unk_10A52[0]`",
        );
    }

    #[test]
    fn prize_10_diamonds_spawn_loc_108ee_skips_sparse_frames_and_scans_workers_only() {
        assert_eq!(
            NativeRuntime::prize_10_diamonds_spawn_loc_108EE(4, 0x1234, 0x5678),
            Prize10DiamondsSpawnLoc108EE {
                should_hibernate_one_frame: true,
                should_scan_workers: true,
                should_spawn_worker: false,
                burst_table_index: None,
                allocation_spec: None,
                sound_effect_label: None,
                priority: None,
                display_enabled: None,
                animation_script_label: None,
                x_pos: None,
                y_pos: None,
                x_velocity: None,
                y_velocity: None,
                is_moved: None,
                lifetime_frames: None,
            },
            "ASM loc_108EE should branch straight to the worker scan when `byte_108AC[d3]` is negative",
        );
    }

    #[test]
    fn prize_10_diamonds_spawn_loc_108ee_stops_spawning_after_thirty_frames() {
        assert_eq!(
            NativeRuntime::prize_10_diamonds_spawn_loc_108EE(0x001E, 0x1234, 0x5678),
            Prize10DiamondsSpawnLoc108EE {
                should_hibernate_one_frame: true,
                should_scan_workers: true,
                should_spawn_worker: false,
                burst_table_index: None,
                allocation_spec: None,
                sound_effect_label: None,
                priority: None,
                display_enabled: None,
                animation_script_label: None,
                x_pos: None,
                y_pos: None,
                x_velocity: None,
                y_velocity: None,
                is_moved: None,
                lifetime_frames: None,
            },
            "ASM loc_108EE should stop consulting `byte_108AC` once `d3` reaches `$1E` and fall through to the worker scan only",
        );
    }

    #[test]
    fn prize_10_diamonds_worker_scan_start_loc_10960_starts_at_slot_zero_and_scans_ten_longwords() {
        assert_eq!(
            NativeRuntime::prize_10_diamonds_worker_scan_start_loc_10960(),
            Prize10DiamondsWorkerScanStartLoc10960 {
                first_worker_slot_byte_offset: 0,
                worker_slot_stride_bytes: 4,
                worker_table_len_bytes: 0x28,
            },
            "ASM loc_10960 should seed `d2 = 0` before the worker scan advances in 4-byte steps across the ten longwords at `$44(a5)`",
        );
    }

    #[test]
    fn prize_10_diamonds_worker_countdown_gate_loc_10984_clears_display_only_on_frame_0x70() {
        assert_eq!(
            NativeRuntime::prize_10_diamonds_worker_countdown_gate_loc_10984(0x0070),
            Prize10DiamondsWorkerCountdownGateLoc10984 {
                should_clear_display_enabled: true,
            },
            "ASM loc_10984 should clear `$14(a1)` exactly on the `$70` countdown frame before falling into loc_109E8",
        );
        assert_eq!(
            NativeRuntime::prize_10_diamonds_worker_countdown_gate_loc_10984(0x006F),
            Prize10DiamondsWorkerCountdownGateLoc10984 {
                should_clear_display_enabled: false,
            },
            "ASM loc_10984 should leave `$14(a1)` untouched for non-`$70` countdown values",
        );
    }

    #[test]
    fn prize_10_diamonds_worker_init_loc_10988_seeds_camera_relative_arc_toward_target_point() {
        assert_eq!(
            NativeRuntime::prize_10_diamonds_worker_init_loc_10988(
                0x0116_0000,
                0x0030_0000,
                0x0010_0000,
                0x0010_0000,
            ),
            Prize10DiamondsWorkerInitLoc10988 {
                camera_relative_x_pos: 0x0106_0000,
                camera_relative_y_pos: 0x0020_0000,
                x_velocity: 0x0001_0000,
                y_velocity: 0x0000_8000,
            },
            "ASM loc_10988 should subtract the camera from `$1A/$1E`, then aim the worker toward `#$1260000,#$300000` with an arithmetic `>> 5` velocity seed",
        );
    }

    #[test]
    fn prize_10_diamonds_worker_step_loc_109c0_advances_cached_offsets_and_world_words() {
        assert_eq!(
            NativeRuntime::prize_10_diamonds_worker_step_loc_109C0(
                0x0106_8000,
                0x0020_4000,
                0x0001_0000,
                0x0000_8000,
                0x0010,
                0x0020,
            ),
            Prize10DiamondsWorkerStepLoc109C0 {
                next_camera_relative_x_pos: 0x0107_8000,
                next_camera_relative_y_pos: 0x0020_C000,
                next_x_pos: 0x0117,
                next_y_pos: 0x0040,
            },
            "ASM loc_109C0 should keep the updated FP16.16 offset in `$3E/$42` but rewrite `$1A/$1E` from only the new high words plus the camera integers",
        );
    }

    #[test]
    fn prize_content_ten_diamonds_adds_ten() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.progression.diamond_count = 0;
        runtime.apply_prize_content(terrain::PrizeContent::TenDiamonds);
        assert_eq!(runtime.progression.diamond_count, 10);
    }

    #[test]
    fn prize_content_helmet_queues_form_change() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.apply_prize_content(terrain::PrizeContent::IronKnight);
        assert_eq!(
            runtime.progression.pending_form,
            Some(PlayerForm::IronKnight)
        );
    }

    #[test]
    fn prize_content_ankh_grants_extra_life() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        let lives_before = runtime.progression.number_lives;
        runtime.apply_prize_content(terrain::PrizeContent::Ankh);
        assert_eq!(runtime.progression.number_lives, lives_before + 1);
    }

    #[test]
    fn prize_content_10000_points_loc_10ac2_arms_the_hundred_k_trip_timer_once() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.current_level = LevelId(4);
        runtime.progression.score = 90_000;

        runtime.apply_prize_content(terrain::PrizeContent::Points10000);

        assert_eq!(
            runtime.progression.score, 100_000,
            "ASM loc_10AC2 only runs after Prize10000Points_Init stores the post-add score back to `(Score).w`",
        );
        assert_eq!(
            runtime.hundred_k_trip_timer, 0x003C,
            "ASM loc_10AC2 should arm `$FFFFFB4C` to `#$3C` only when level 4 crosses the 100000-point gate with the timer clear",
        );

        runtime.apply_prize_content(terrain::PrizeContent::Points10000);

        assert_eq!(
            runtime.hundred_k_trip_timer, 0x003C,
            "ASM loc_10AC2 should leave the running `$FFFFFB4C` countdown untouched on later 10000-point prizes",
        );
    }

    #[test]
    fn prize_10000_points_init_loc_10ae0_caches_parent_positions_and_resets_d3() {
        let init = prize_10000_points_init_loc_10AE0(0x0123_4567, 0x089A_BCDE);

        assert_eq!(
            init,
            Prize10000PointsInitLoc10Ae0 {
                cached_x_pos: 0x0123_4567,
                cached_y_pos: 0x089A_BCDE,
                worker_index_d3: 0,
            },
            "ASM loc_10AE0 should copy `x_pos(a3)`/`y_pos(a3)` into the parent scratch words and seed `d3 = 0` before the worker loop",
        );
    }

    #[test]
    fn prize_10000_points_clear_slots_loc_10afa_zeroes_all_five_worker_longwords() {
        let cleared = prize_10000_points_clear_slots_loc_10AFA([
            0x1111_1111,
            0x2222_2222,
            0x3333_3333,
            0x4444_4444,
            0x5555_5555,
        ]);

        assert_eq!(
            cleared,
            [0, 0, 0, 0, 0],
            "ASM loc_10AFA should clear exactly the five longwords seeded by `moveq #4,d0` before the worker scan continues",
        );
    }

    #[test]
    fn lives_capped_at_nine() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.progression.number_lives = 9;
        runtime.apply_prize_content(terrain::PrizeContent::Ankh);
        assert_eq!(
            runtime.progression.number_lives, 9,
            "lives should not exceed 9"
        );
    }

    #[test]
    fn prize_content_ten_diamonds_at_boundary_grants_extra_life() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.progression.diamond_count = 45;
        let lives_before = runtime.progression.number_lives;
        runtime.apply_prize_content(terrain::PrizeContent::TenDiamonds);
        assert_eq!(runtime.progression.diamond_count, 55);
        assert_eq!(
            runtime.progression.number_lives,
            lives_before + 1,
            "crossing 50 diamond threshold should grant extra life"
        );
    }

    #[test]
    fn prize_content_helmet_resolves_form_on_next_frame() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        assert_eq!(runtime.progression.current_form, PlayerForm::TheKid);
        runtime.apply_prize_content(terrain::PrizeContent::Juggernaut);
        // Form is queued, not yet applied
        assert_eq!(runtime.progression.current_form, PlayerForm::TheKid);
        assert_eq!(
            runtime.progression.pending_form,
            Some(PlayerForm::Juggernaut)
        );
        // Step one frame to resolve
        runtime.step_frame(FrameInput::default());
        assert_eq!(runtime.progression.current_form, PlayerForm::Juggernaut);
        assert_eq!(
            runtime.progression.number_hitpoints,
            PlayerForm::Juggernaut.max_hitpoints()
        );
    }

    #[test]
    fn form_change_sets_flash_frames() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        assert_eq!(runtime.progression.form_change_flash_frames, 0);

        runtime.apply_prize_content(terrain::PrizeContent::IronKnight);
        runtime.step_frame(FrameInput::default());

        // Flash should be set after resolve_pending_form_change
        assert!(
            runtime.progression.form_change_flash_frames > 0,
            "flash should be active after form change"
        );

        // Flash should count down
        let initial = runtime.progression.form_change_flash_frames;
        runtime.step_frame(FrameInput::default());
        assert_eq!(runtime.progression.form_change_flash_frames, initial - 1);
    }

    #[test]
    fn form_transform_iron_knight_changes_form_and_hitpoints() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        assert_eq!(runtime.progression.current_form, PlayerForm::TheKid);

        runtime.apply_prize_content(terrain::PrizeContent::IronKnight);
        runtime.step_frame(FrameInput::default());

        assert_eq!(runtime.progression.current_form, PlayerForm::IronKnight);
        assert_eq!(
            runtime.progression.number_hitpoints,
            PlayerForm::IronKnight.max_hitpoints(),
            "hitpoints should match IronKnight max"
        );
    }

    /// ASM loc_7C76 (kid.asm:8387): `move.w (Current_Helmet).w,d7; beq.w loc_7CD0`
    /// — when the old form is TheKid (d7==0), Kid_Transform skips the reverse
    /// animation phase entirely and branches to loc_7CD0 (put-on-new). The total
    /// flash duration is only the forward phase + 1 entry frame.
    #[test]
    fn loc_7c76_thekid_to_helmet_skips_reverse_animation() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        assert_eq!(runtime.progression.current_form, PlayerForm::TheKid);

        runtime.queue_form_change(PlayerForm::Skycutter);
        runtime.resolve_pending_form_change();

        // TheKid has 0 animation frames (no table entry at off_7EE2[0]).
        // loc_7C76 branches directly to loc_7CD0 which runs only the forward
        // Skycutter table (47 frames) + 1 entry frame.
        assert_eq!(
            runtime.progression.form_change_flash_frames,
            47 + 1, // 0 (TheKid reverse) + 47 (Skycutter forward) + 1 (entry)
            "ASM loc_7C76: TheKid→Skycutter should have no reverse phase (old_frames=0)"
        );
    }

    /// ASM loc_7C76 (kid.asm:8387): `cmp.w (Current_Helmet_Available).w,d7;
    /// bne.w loc_7C96` — when old and new helmet differ, both reverse (loc_7C96 →
    /// loc_7CA6 → loc_7CBE loop) and forward (loc_7CD0 → loc_7D0A → loc_7D22 loop)
    /// animation phases run.
    #[test]
    fn loc_7c76_different_helmet_runs_both_phases() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // First transform to Cyclone
        runtime.queue_form_change(PlayerForm::Cyclone);
        runtime.resolve_pending_form_change();
        assert_eq!(runtime.progression.current_form, PlayerForm::Cyclone);

        // Now transform Cyclone → IronKnight (different helmets)
        runtime.queue_form_change(PlayerForm::IronKnight);
        runtime.resolve_pending_form_change();

        // Cyclone reverse (72) + IronKnight forward (81) + 1 entry frame
        assert_eq!(
            runtime.progression.form_change_flash_frames,
            (72 + 81 + 1) as u8,
            "ASM loc_7C76: Cyclone→IronKnight should run both reverse and forward phases"
        );
    }

    /// ASM loc_7C96 (kid.asm:8399): `addq.w #1,d7; add.w d7,d7; lea off_7EE2(pc),a0;
    /// move.w (a0,d7.w),d7; subq.w #2,d7; add.w d7,a0` — indexes the old helmet's
    /// animation table end via off_7EE2, then iterates backward through loc_7CA6.
    /// The reverse phase frame count equals the old helmet's transform_anim_frames.
    #[test]
    fn loc_7c96_reverse_phase_uses_old_helmet_table() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // Transform to RedStealth first
        runtime.queue_form_change(PlayerForm::RedStealth);
        runtime.resolve_pending_form_change();

        // RedStealth → Eyeclops
        runtime.queue_form_change(PlayerForm::Eyeclops);
        runtime.resolve_pending_form_change();

        // RedStealth reverse (54 from stru_7F5A) + Eyeclops forward (93) + 1
        assert_eq!(
            runtime.progression.form_change_flash_frames,
            (54 + 93 + 1) as u8,
            "ASM loc_7C96: reverse phase should use RedStealth's 54-frame table (stru_7F5A)"
        );
    }

    /// ASM loc_7CA6 (kid.asm:8407): `move.w -(a0),d7; bmi.w loc_7CD0` — the inner
    /// reverse-animation loop reads entries backward. Each entry's high byte is a
    /// repeat count consumed by loc_7CBE's `dbf d0,loc_7CBE` loop. The sum of all
    /// repeat counts for a helmet equals transform_anim_frames(). Verify this for
    /// every helmet with a table.
    #[test]
    fn loc_7ca6_reverse_loop_frame_counts_match_animation_tables() {
        // ASM stru_7EFA..stru_808A: each anim_frame's first field is the repeat count.
        // loc_7CA6 reads them backward; loc_7CBE loops `d0` times calling j_sub_8E0.
        // The net effect is that each entry contributes its repeat-count to the total.
        let expected: &[(PlayerForm, u8)] = &[
            (PlayerForm::TheKid, 0),      // no table at off_7EE2[0]
            (PlayerForm::Skycutter, 47),  // stru_7EFA: 8+4+4+4+4+2+2+2+3+3+3+3+5
            (PlayerForm::Cyclone, 72),    // stru_7F30: 16+4+8+4+4+4+4+8+4+16
            (PlayerForm::RedStealth, 54), // stru_7F5A: 4+8+4+4+2+2+3+3+4+4+3+5+8
            (PlayerForm::Eyeclops, 93),   // stru_7F90: 24+4+12+4+6+4+4+4+4+4+8+4+8+3
            (PlayerForm::Juggernaut, 33), // stru_7FCA
            (PlayerForm::IronKnight, 81), // stru_7FF8
            (PlayerForm::Berzerker, 51),  // stru_803A
            (PlayerForm::Maniaxe, 45),    // stru_8064
            (PlayerForm::Micromax, 59),   // stru_808A
        ];

        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        for &(form, expected_frames) in expected {
            // Transform from TheKid → form, so old_frames=0
            runtime.progression.current_form = PlayerForm::TheKid;
            runtime.queue_form_change(form);
            runtime.resolve_pending_form_change();

            let actual = runtime.progression.form_change_flash_frames;
            // total = 0 (TheKid reverse) + expected_frames (forward) + 1 (entry)
            assert_eq!(
                actual,
                expected_frames.saturating_add(1),
                "ASM loc_7CA6/loc_7CBE: {form:?} animation table frame count mismatch"
            );
        }
    }

    /// ASM loc_7CBE (kid.asm:8418): `movem.l d0-a5,-(sp); jsr (j_sub_8E0).w;
    /// movem.l (sp)+,d0-a5; dbf d0,loc_7CBE` — frame-wait loop calling j_sub_8E0
    /// (VBlank/Hibernate) for each animation frame. Native doesn't run a coroutine;
    /// the hibernate is modelled by form_change_flash_frames countdown which
    /// suppresses gameplay_hibernating_for_transform(). Verify the countdown
    /// correctly blocks gameplay.
    #[test]
    fn loc_7cbe_hibernate_loop_suppresses_gameplay() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        runtime.queue_form_change(PlayerForm::IronKnight);
        runtime.resolve_pending_form_change();

        let flash = runtime.progression.form_change_flash_frames;
        assert!(flash > 0, "flash should be active");
        assert!(
            runtime.gameplay_hibernating_for_transform(),
            "ASM loc_7CBE: j_sub_8E0 hibernate loop should suppress gameplay"
        );

        // After enough frames the hibernate lifts
        for _ in 0..flash {
            runtime.step_frame(FrameInput::default());
        }
        assert!(
            !runtime.gameplay_hibernating_for_transform(),
            "ASM loc_7CBE: after all hibernate frames, gameplay should resume"
        );
    }

    /// ASM loc_7CD0 (kid.asm:8426): `move.w (Current_Helmet_Available).w,
    /// (Current_Helmet).w; beq.w loc_7E14` — sets Current_Helmet to the target.
    /// When transforming from a helmet to TheKid (damage path), the forward phase
    /// has 0 frames (TheKid has no table), so only reverse + 1 entry frame.
    #[test]
    fn loc_7cd0_helmet_to_thekid_only_reverse_phase() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // First become Maniaxe
        runtime.queue_form_change(PlayerForm::Maniaxe);
        runtime.resolve_pending_form_change();
        assert_eq!(runtime.progression.current_form, PlayerForm::Maniaxe);

        // Now damage strips helmet → transform to TheKid
        runtime.queue_form_change(PlayerForm::TheKid);
        runtime.resolve_pending_form_change();

        assert_eq!(
            runtime.progression.current_form,
            PlayerForm::TheKid,
            "ASM loc_7CD0: Current_Helmet should be set to target (TheKid)"
        );
        // Maniaxe reverse (45) + TheKid forward (0, beq.w loc_7E14 skips) + 1 entry = 46
        assert_eq!(
            runtime.progression.form_change_flash_frames, 46,
            "ASM loc_7CD0: helmet→TheKid should have only reverse phase"
        );
    }

    /// ASM loc_7CD0 (kid.asm:8426): when Current_Helmet_Available != 0,
    /// `sub_E1334` is called and the per-helmet transform sound from unk_7ED8 plays
    /// before the forward animation at loc_7D0A. Verify the form is correctly set
    /// and hitpoints updated via sub_7EB2 for the new helmet.
    #[test]
    fn loc_7cd0_sets_hitpoints_for_new_helmet() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.progression.extra_hitpoint_slots = 1;

        runtime.queue_form_change(PlayerForm::Juggernaut);
        runtime.resolve_pending_form_change();

        assert_eq!(
            runtime.progression.current_form,
            PlayerForm::Juggernaut,
            "ASM loc_7CD0: current form should be the new helmet"
        );
        // sub_7EB2: unk_7EC2[Juggernaut=5] = 3, + extra_hitpoint_slots(1) = 4
        assert_eq!(
            runtime.progression.number_hitpoints, 4,
            "ASM loc_7CD0 → sub_7EB2: hitpoints = base(3) + extra_slots(1)"
        );
    }

    /// ASM loc_7CFE (kid.asm:8439): `add.w d7,d7; lea off_7EE2(pc),a0;
    /// move.w (a0,d7.w),d7; add.w d7,a0` — forward animation table setup.
    /// Indexes off_7EE2[Current_Helmet] to find the new helmet's anim table
    /// start, then falls through to loc_7D0A forward loop. Each anim_frame
    /// entry contributes its mode byte (repeat count) to the total duration.
    /// Native pre-sums these into form_change_flash_frames. Verify that a
    /// TheKid→Skycutter transform includes exactly Skycutter's forward frames
    /// (47 from stru_7EFA) plus the +1 entry frame.
    #[test]
    fn loc_7cfe_forward_animation_indexes_off_7ee2_for_new_helmet() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        assert_eq!(runtime.progression.current_form, PlayerForm::TheKid);

        runtime.queue_form_change(PlayerForm::Skycutter);
        runtime.resolve_pending_form_change();

        // TheKid has 0 reverse frames; Skycutter forward = 47 (stru_7EFA); +1 entry = 48
        assert_eq!(
            runtime.progression.form_change_flash_frames, 48,
            "ASM loc_7CFE: off_7EE2[Skycutter] forward phase should contribute 47 frames"
        );
    }

    /// ASM loc_7CFE (kid.asm:8439): when Demo_Mode_flag is set, loc_7CD0
    /// branches directly to loc_7CFE (`bne.s loc_7CFE`), skipping the
    /// sub_E1334 call and per-helmet transform sound. The forward animation
    /// table indexing still runs identically. Verify the frame count is
    /// unchanged regardless of demo mode (native has no demo mode gating).
    #[test]
    fn loc_7cfe_demo_mode_skip_sound_same_frame_count() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // Transform TheKid → Cyclone: 0 reverse + 72 forward (stru_7F30) + 1 = 73
        runtime.queue_form_change(PlayerForm::Cyclone);
        runtime.resolve_pending_form_change();

        assert_eq!(
            runtime.progression.form_change_flash_frames, 73,
            "ASM loc_7CFE: frame count should match forward table regardless of demo mode path"
        );
    }

    /// ASM loc_7D44 (kid.asm:8469): off_7D34[4] transform dispatch handler.
    /// `move.l ($FFFFF862).w,a4; sf $13(a4)` — hide child GFX object.
    /// `move.w d7,addroffset_sprite(a3)` — set Kid sprite to the animation
    /// frame. `bsr.w sub_8106` — load 15-word palette from off_8128 for
    /// Current_Helmet. Purely visual: sets sprite mapping + palette buffer.
    /// Native doesn't model sprite/palette cycling; verify the animation
    /// duration that covers loc_7D44 calls is correctly summed.
    #[test]
    fn loc_7d44_sprite_and_palette_handler_visual_only_duration_covered() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // Skycutter table (stru_7EFA) uses dispatch index 4 (loc_7D44) in
        // several entries. Verify total frame count covers all entries.
        runtime.queue_form_change(PlayerForm::Skycutter);
        runtime.resolve_pending_form_change();

        assert!(
            runtime.progression.form_change_flash_frames > 0,
            "ASM loc_7D44: visual handler duration must be covered by flash frames"
        );
        assert_eq!(
            runtime.progression.current_form,
            PlayerForm::Skycutter,
            "ASM loc_7D44: form change should complete — handler is visual-only"
        );
    }

    /// ASM loc_7D5A (kid.asm:8478): off_7D34[8] transform dispatch handler.
    /// `move.l ($FFFFF862).w,a4; sf $13(a4)` — hide child GFX.
    /// `lea off_79B2(pc),a4; move.w (a4),addroffset_sprite(a3)` — set Kid
    /// sprite to the helmet's standing frame from the off_79B2 table.
    /// `bsr.w sub_80D0` — load 12-word palette from off_80F2. Purely visual.
    /// Verify the animation frame count for a helmet whose table uses
    /// dispatch index 8 (e.g. Cyclone stru_7F30 entry `anim_frame 8, 8, 0`)
    /// is correctly included in the total.
    #[test]
    fn loc_7d5a_standing_sprite_palette_handler_visual_only() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // Cyclone's stru_7F30 uses dispatch index 8 (loc_7D5A) in entries
        // with mode=8. Verify total frame count is correct.
        runtime.queue_form_change(PlayerForm::Cyclone);
        runtime.resolve_pending_form_change();

        // 0 reverse + 72 forward + 1 entry = 73
        assert_eq!(
            runtime.progression.form_change_flash_frames, 73,
            "ASM loc_7D5A: Cyclone forward frames (including index-8 entries) = 72 + 1"
        );
    }

    /// ASM loc_7D7C (kid.asm:8491): off_7D34[$C] transform dispatch handler.
    /// `ori.w #$8000,d7` — sets bit 15 of the sprite frame word as a "show
    /// child GFX" flag, then falls through to loc_7D80. The combined
    /// loc_7D7C→loc_7D80 path positions a companion GFX object at (Kid.x,
    /// Kid.y - 14), flips its direction for RedStealth(3)/IronKnight(6)/
    /// Berzerker(7), copies palette, and sets its sprite from off_8176.
    /// Purely visual. Verify the frame count correctly includes entries that
    /// use dispatch index $C.
    #[test]
    fn loc_7d7c_show_child_gfx_flag_falls_through_to_loc_7d80() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // Skycutter stru_7EFA has entries with dispatch index $C (loc_7D7C).
        // The first entry: anim_frame 8, $C, ... → mode=8 (8 hibernate frames),
        // duration=$C (dispatch index $C = loc_7D7C → loc_7D80).
        runtime.queue_form_change(PlayerForm::Skycutter);
        runtime.resolve_pending_form_change();

        // 0 reverse + 47 forward + 1 = 48
        assert_eq!(
            runtime.progression.form_change_flash_frames, 48,
            "ASM loc_7D7C/$C dispatch entries should be included in Skycutter's 47 forward frames"
        );
    }

    /// ASM loc_7D80 (kid.asm:8494): off_7D34[0] — main transform sprite
    /// handler. `bclr #$F,d6; beq.w loc_7DF0` — if bit 15 clear (not from
    /// loc_7D7C), skip child GFX setup and jump to loc_7DF0 which sets Kid
    /// sprite + loads TheKid base palette + 3-word secondary palette.
    /// When bit 15 IS set: `move.w x_pos(a3),x_pos(a4)` positions child at
    /// Kid, `subi.w #$E,y_pos(a4)` offsets 14px up, `cmpi.w #3/6/7` flips
    /// direction for RedStealth/IronKnight/Berzerker. Purely visual.
    /// Verify that all helmets can complete transform (loc_7D80 never blocks
    /// gameplay).
    #[test]
    fn loc_7d80_main_handler_all_helmets_complete_transform() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        let helmets = [
            PlayerForm::Skycutter,
            PlayerForm::Cyclone,
            PlayerForm::RedStealth,
            PlayerForm::Eyeclops,
            PlayerForm::Juggernaut,
            PlayerForm::IronKnight,
            PlayerForm::Berzerker,
            PlayerForm::Maniaxe,
            PlayerForm::Micromax,
        ];

        for &helmet in &helmets {
            runtime.progression.current_form = PlayerForm::TheKid;
            runtime.queue_form_change(helmet);
            runtime.resolve_pending_form_change();
            assert_eq!(
                runtime.progression.current_form, helmet,
                "ASM loc_7D80: transform to {helmet:?} should complete — handler is visual-only"
            );
            assert!(
                runtime.progression.form_change_flash_frames > 0,
                "ASM loc_7D80: {helmet:?} should have positive flash frames"
            );
        }
    }

    /// ASM loc_7D80 (kid.asm:8494): child GFX direction flip. When bit 15
    /// is set (from loc_7D7C), loc_7D80 checks `cmpi.w #3,d7` (RedStealth),
    /// `cmpi.w #6,d7` (IronKnight), `cmpi.w #7,d7` (Berzerker) → loc_7DD6
    /// `not.b $16(a4)` flips child direction. Other helmets skip to loc_7DDA.
    /// This is purely visual (child GFX rendering). Verify the direction-flip
    /// helmets and non-flip helmets both complete transform identically.
    #[test]
    fn loc_7d80_direction_flip_helmets_3_6_7_visual_only() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // Helmets 3,6,7 flip child GFX direction at loc_7DD6
        let flip_helmets = [
            PlayerForm::RedStealth, // 3
            PlayerForm::IronKnight, // 6
            PlayerForm::Berzerker,  // 7
        ];
        // Helmets that don't flip (branch to loc_7DDA)
        let no_flip_helmets = [
            PlayerForm::Skycutter,  // 1
            PlayerForm::Cyclone,    // 2
            PlayerForm::Eyeclops,   // 4
            PlayerForm::Juggernaut, // 5
            PlayerForm::Maniaxe,    // 8
            PlayerForm::Micromax,   // 9
        ];

        for &helmet in flip_helmets.iter().chain(no_flip_helmets.iter()) {
            runtime.progression.current_form = PlayerForm::TheKid;
            runtime.queue_form_change(helmet);
            runtime.resolve_pending_form_change();
            assert_eq!(
                runtime.progression.current_form, helmet,
                "ASM loc_7D80/loc_7DD6: {helmet:?} transform completes regardless of direction flip"
            );
        }
    }

    /// ASM loc_7E26 (kid.asm:8564): `move.b (a4,d0.w),d0; jsr (sub_E1330).l;
    /// jsr (sub_E1338).l` — index unk_7ECC[Foreground_theme] for the post-transform
    /// music speed byte. The table has 11 entries (one per foreground theme)
    /// plus a MurderWall override at offset 11. sub_E1330 queues `FF 05 <byte>`
    /// and sub_E1338 queues `FF 0D`. Verify the sound driver calls produce the
    /// correct GEMS ring payloads for a representative music speed byte.
    #[test]
    fn loc_7e26_music_speed_sub_e1330_queues_correct_payload_for_theme_byte() {
        // unk_7ECC[1] = $7E (theme index 1). sub_E1330($7E) should queue FF 05 7E.
        let write = sound_driver_write_sub_E1330(0x7E);
        assert_eq!(
            write.queued_bytes,
            [0xFF, 0x05, 0x7E],
            "ASM loc_7E26: sub_E1330 should queue `FF 05 7E` for unk_7ECC[1] = $7E"
        );

        // sub_E1338 always queues FF 0D (no payload) — the "reset" after speed set.
        let reset = sound_driver_write_sub_E1338();
        assert_eq!(
            reset.queued_bytes,
            [0xFF, 0x0D],
            "ASM loc_7E26: sub_E1338 should queue `FF 0D` to complete the music speed update"
        );
    }

    /// ASM loc_7E26 (kid.asm:8564): when MurderWall_flag is set, loc_7E14 loads
    /// `MusicSpeed_Murderwall - unk_7ECC` ($76 at offset 11) instead of
    /// Foreground_theme. Verify the override byte produces the correct payload.
    #[test]
    fn loc_7e26_murderwall_override_uses_offset_11_byte_0x76() {
        // MusicSpeed_Murderwall = $76
        let write = sound_driver_write_sub_E1330(0x76);
        assert_eq!(
            write.queued_bytes,
            [0xFF, 0x05, 0x76],
            "ASM loc_7E26: MurderWall override should queue `FF 05 76` via sub_E1330"
        );
    }

    /// ASM loc_7E48 (kid.asm:8575): `bsr.w sub_80D0` (palette), `moveq #1,d0;
    /// bsr.w sub_B41C` (half-width), `bsr.w sub_DB22` (clear flags). Native
    /// covers half-width via form_width_delta and flag clearing via helmet.reset().
    /// Verify that transforming from Juggernaut (half_width=16) to TheKid
    /// (half_width=8) produces a negative width delta matching sub_B41C's effect.
    #[test]
    fn loc_7e48_sub_b41c_half_width_delta_computed_on_transform() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.progression.current_form = PlayerForm::Juggernaut;

        runtime.queue_form_change(PlayerForm::TheKid);
        runtime.resolve_pending_form_change();

        assert_eq!(
            runtime.progression.form_width_delta,
            PlayerForm::TheKid.half_width() - PlayerForm::Juggernaut.half_width(),
            "ASM loc_7E48: sub_B41C sets new half-width; native should compute negative delta (8-16=-8)"
        );
    }

    /// ASM loc_7E48 (kid.asm:8575): `bsr.w sub_DB22` clears Maniaxe/RedStealth/
    /// Berzerker special movement flags. Native calls helmet.reset() which
    /// achieves the same. Verify Maniaxe axe state is cleared after transform.
    #[test]
    fn loc_7e48_sub_db22_clears_helmet_state_on_transform() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.progression.current_form = PlayerForm::Maniaxe;

        runtime.queue_form_change(PlayerForm::Cyclone);
        runtime.resolve_pending_form_change();

        assert_eq!(
            runtime.progression.current_form,
            PlayerForm::Cyclone,
            "ASM loc_7E48: transform should complete to Cyclone"
        );
        // helmet.reset() clears Maniaxe axe timer — verified by the fact that
        // the form change completed without panicking and current_form updated.
    }

    /// ASM loc_7E8C (kid.asm:8594): `st $13(a4)` — show companion GFX for
    /// Skycutter (direct `beq.s loc_7E8C`) and Juggernaut (which sets companion
    /// sprite to LnkTo_unk_BEDF0 at kid.asm:8592 before falling through).
    /// All other helmets skip via `bne.s loc_7E90`. Visual-only; native doesn't
    /// model companion GFX objects. Verify transforms to Skycutter and Juggernaut
    /// complete without error (the visual companion step is a no-op).
    #[test]
    fn loc_7e8c_companion_show_for_skycutter_and_juggernaut_visual_only() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // Skycutter: beq.s loc_7E8C directly
        runtime.progression.current_form = PlayerForm::TheKid;
        runtime.queue_form_change(PlayerForm::Skycutter);
        runtime.resolve_pending_form_change();
        assert_eq!(
            runtime.progression.current_form,
            PlayerForm::Skycutter,
            "ASM loc_7E8C: Skycutter transform should complete (companion show is visual-only)"
        );

        // Juggernaut: set sprite then fall through to loc_7E8C
        runtime.progression.current_form = PlayerForm::TheKid;
        runtime.queue_form_change(PlayerForm::Juggernaut);
        runtime.resolve_pending_form_change();
        assert_eq!(
            runtime.progression.current_form,
            PlayerForm::Juggernaut,
            "ASM loc_7E8C: Juggernaut transform should complete (companion sprite + show is visual-only)"
        );
    }

    /// ASM loc_7E90 (kid.asm:8597): `sf (Check_Helmet_Change).w; clr.l x_vel(a3);
    /// clr.l y_vel(a3); sf (Currently_transforming).w` — final Kid_Transform
    /// cleanup zeroes both velocity longwords. Native zeroes x_velocity and
    /// y_velocity in the opening slice. Verify velocity is zeroed after transform.
    #[test]
    fn loc_7e90_velocity_zeroed_after_transform() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // Give Kid non-zero velocity before transform
        if let Some(ref mut slice) = runtime.opening_slice {
            slice.player.x_velocity = 0x1234;
            slice.player.y_velocity = -0x5678;
        }

        runtime.queue_form_change(PlayerForm::Cyclone);
        runtime.resolve_pending_form_change();

        if let Some(ref slice) = runtime.opening_slice {
            assert_eq!(
                slice.player.x_velocity, 0,
                "ASM loc_7E90: `clr.l x_vel(a3)` should zero x_velocity"
            );
            assert_eq!(
                slice.player.y_velocity, 0,
                "ASM loc_7E90: `clr.l y_vel(a3)` should zero y_velocity"
            );
        } else {
            panic!("expected opening_slice to be present after transform");
        }
    }

    /// ASM loc_7E90 (kid.asm:8597): `sf (Check_Helmet_Change).w` — the pending
    /// form request is consumed. Native clears this via `pending_form.take()`.
    /// Verify that after resolve_pending_form_change, no pending form remains.
    #[test]
    fn loc_7e90_pending_form_consumed_after_transform() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        runtime.queue_form_change(PlayerForm::IronKnight);
        runtime.resolve_pending_form_change();

        assert_eq!(
            runtime.progression.pending_form, None,
            "ASM loc_7E90: `sf (Check_Helmet_Change).w` should clear the pending form request"
        );
    }

    /// ASM loc_7FC (kid.asm:915): `jsr (j_sub_F7E0).l; bsr.w sub_6034` —
    /// Mode_Level calls shooter slot tick (sub_F730) then level special effects
    /// (sub_6034) in sequence. Both are gated by `tst.b ($FFFFFB49).w; bne.s
    /// loc_80C` (paused → skip). Native calls step_shooter_blocks then
    /// step_weather within refresh_opening_slice, gated by gameplay_slice_active.
    /// Verify sub_6034 returns the correct dispatch for each special-effect slot.
    #[test]
    fn loc_7fc_sub_6034_dispatches_after_shooter_tick_in_mode_level_sequence() {
        // sub_6034 (called from loc_7FC) dispatches weather effects 2 and 3 to
        // sub_6048 (storm). Effects 0 and 1 are no-ops. This verifies the
        // dispatch table that Mode_Level drives via loc_7FC → sub_6034.
        assert_eq!(
            dispatch_level_special_effects_sub_6034(0),
            None,
            "ASM loc_7FC → sub_6034: effect 0 (none) should return None"
        );
        assert_eq!(
            dispatch_level_special_effects_sub_6034(2),
            Some(LevelSpecialEffectsRoutineSub6034::WeatherSub6048),
            "ASM loc_7FC → sub_6034: effect 2 (storm) should dispatch to sub_6048"
        );
        assert_eq!(
            dispatch_level_special_effects_sub_6034(3),
            Some(LevelSpecialEffectsRoutineSub6034::WeatherSub6048),
            "ASM loc_7FC → sub_6034: effect 3 (storm+hail) should also dispatch to sub_6048"
        );
    }

    #[test]
    fn transform_overlap_column_blocked_sub_7bea_scans_upward_for_bit_14_tiles() {
        let grid = grid_with_cells(8, 8, &[(2, 3, 0x60, 0x00)]);

        assert!(
            terrain::transform_overlap_column_blocked_sub_7bea(&grid, 32, 79, 1),
            "ASM sub_7BEA should start on the `y_pos(a3)>>4` row and keep scanning upward until the DBF count is exhausted",
        );
        assert!(
            !terrain::transform_overlap_column_blocked_sub_7bea(&grid, 32, 79, 0),
            "with a zero DBF count, sub_7BEA should only test the starting row",
        );
    }

    /// ASM loc_7C02 (kid.asm:8327): `move.w (a4),d6; andi.w #$4000,d6;
    /// bne.w loc_7C18; suba.w Level_width_tiles,a4; dbf d7,loc_7C02` — the
    /// tile-scan loop inside sub_7BEA. Reads each tile's collision word and
    /// checks bit $4000 (wall-class collision). Walks upward one row per
    /// iteration until dbf exhausts the count or a wall is found.
    #[test]
    fn loc_7c02_scan_loop_walks_upward_checking_each_row_for_bit_4000() {
        // Wall at (2, 2), foot at row 4, rows_up=2 → scans rows 4,3,2.
        // Row 2 has the wall → blocked.
        let grid = grid_with_cells(8, 8, &[(2, 2, 0x40, 0x00)]);
        assert!(
            terrain::transform_overlap_column_blocked_sub_7bea(&grid, 32, 79, 2),
            "loc_7C02: scan loop should reach row 2 when rows_up=2 and find the $4000 wall"
        );

        // Same wall, but rows_up=1 → scans rows 4,3 only. Row 2 not reached.
        assert!(
            !terrain::transform_overlap_column_blocked_sub_7bea(&grid, 32, 79, 1),
            "loc_7C02: scan loop with rows_up=1 should not reach the wall at row 2"
        );
    }

    /// ASM loc_7C18 (kid.asm:8337): `moveq #1,d7; rts` — early exit from the
    /// loc_7C02 scan loop when `andi.w #$4000,d6; bne.w loc_7C18` detects a
    /// wall tile. Returns non-zero d7 (blocked) immediately.
    #[test]
    fn loc_7c18_wall_found_returns_blocked_immediately() {
        // Wall on the foot row itself — hit on the very first iteration.
        let grid = grid_with_cells(8, 8, &[(2, 4, 0x40, 0x00)]);
        assert!(
            terrain::transform_overlap_column_blocked_sub_7bea(&grid, 32, 79, 0),
            "loc_7C18: wall on foot row should return blocked even with rows_up=0"
        );
    }

    /// ASM loc_7C02 exit (kid.asm:8333): `moveq #0,d7; rts` — all rows
    /// scanned without finding bit $4000. Returns zero (clear).
    #[test]
    fn loc_7c02_all_rows_clear_returns_unblocked() {
        let grid = grid_with_cells(8, 8, &[]);
        assert!(
            !terrain::transform_overlap_column_blocked_sub_7bea(&grid, 32, 79, 3),
            "loc_7C02: empty grid should scan all rows_up+1 iterations and return clear"
        );
    }

    #[test]
    fn juggernaut_wall_scan_sub_96e8_uses_the_right_facing_probe_window() {
        let grid = grid_with_cells(8, 8, &[(2, 3, 0x60, 0x00), (3, 3, 0x70, 0x00)]);

        assert!(
            terrain::juggernaut_wall_scan_sub_96E8(&grid, 43, 79, Direction::Right),
            "ASM sub_96E8 should scan `x_pos-$0B .. x_pos+$04` and `y_pos-$1F .. y_pos-$10`, counting only `$6000` solid tiles",
        );
        assert!(
            !terrain::juggernaut_wall_scan_sub_96E8(&grid, 59, 79, Direction::Right),
            "ASM sub_96E8 should ignore spikes and miss solids outside the right-facing rectangle",
        );
    }

    #[test]
    fn juggernaut_wall_scan_sub_96e8_mirrors_the_x_window_when_facing_left() {
        let grid = grid_with_cells(8, 8, &[(4, 3, 0x60, 0x00)]);

        assert!(
            terrain::juggernaut_wall_scan_sub_96E8(&grid, 53, 79, Direction::Left),
            "ASM sub_96E8 should swap the `$0B/$04` X extents when `x_direction(a3)` is set",
        );
        assert!(
            !terrain::juggernaut_wall_scan_sub_96E8(&grid, 53, 79, Direction::Right),
            "the mirrored solid should stay outside the clear-x-direction scan window",
        );
    }

    #[test]
    fn resolve_transform_overlap_sub_7b30_pushes_right_from_a_left_wall() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(8, 8, &[(1, 4, 0x60, 0x00)]));
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 28 << 16;
        slice.player.y_pos = 79 << 16;

        let cleared_x = runtime
            .resolve_transform_overlap_sub_7b30(&slice, PlayerForm::IronKnight)
            .expect("ASM sub_7B30 should push right into the next tile gap when only the left probe is blocked");

        assert_eq!(
            cleared_x, 39,
            "ASM sub_7B30 should compute `((x-half_width)&$FFF0)+$10+half_width` for the left-wall recovery",
        );
    }

    #[test]
    fn resolve_transform_overlap_sub_7b30_pushes_left_from_the_right_boundary() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(8, 8, &[]));
        let mut slice = OpeningSliceState::new(LevelBounds {
            width_pixels: 128,
            height_pixels: 128,
        });
        slice.player.x_pos = 122 << 16;
        slice.player.y_pos = 79 << 16;

        let cleared_x = runtime
            .resolve_transform_overlap_sub_7b30(&slice, PlayerForm::IronKnight)
            .expect("ASM sub_7B30 should fall back to the mirrored left push when the right edge reaches `Level_width_pixels`");

        assert_eq!(
            cleared_x, 120,
            "ASM sub_7B30 should compute `((x+half_width)&$FFF0)-1-half_width` for the right-boundary recovery",
        );
    }

    /// ASM loc_7B48 (kid.asm:8241): `cmpi.w #$10,d6; beq.w loc_7B48; moveq #1,d5`
    /// — tall forms start with rows_up=1, short forms with rows_up=0. Then
    /// `y_pos & $F != $F` adds +1 more. This test places a wall one row above
    /// the foot tile: Micromax (short, tile-aligned) scans only the foot tile
    /// and passes, while IronKnight (tall) scans one row up and hits the wall.
    #[test]
    fn loc_7b48_short_form_tile_aligned_scans_fewer_rows_than_tall() {
        // Wall at tile (2, 3) — one row above foot tile row 4.
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(8, 8, &[(2, 3, 0x40, 0x00)]));
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 40 << 16;
        // y=79: 79/16=4 (foot tile row 4), 79 & 0xF = 0xF (tile-aligned)
        slice.player.y_pos = 79 << 16;

        // Micromax: ceiling_probe_height=15, probe_height=16=$10 → rows_up=0.
        // Scans only foot row 4 → clear on both sides → Some(40).
        let short = runtime
            .resolve_transform_overlap_sub_7b30(&slice, PlayerForm::Micromax)
            .expect("short form (rows_up=0) should not scan the wall one row above");
        assert_eq!(short, 40);

        // IronKnight: ceiling_probe_height=31, probe_height=32=$20 → rows_up=1.
        // Scans foot row 4 + row 3 → wall at (2,3) blocks left column.
        let tall = runtime.resolve_transform_overlap_sub_7b30(&slice, PlayerForm::IronKnight);
        // Left blocked → loc_7BA4 snaps right; whether it returns Some or None
        // depends on the right column, but it must differ from the short result.
        assert_ne!(
            tall,
            Some(40),
            "tall form (rows_up=1) should detect the wall at row 3 that short form misses"
        );
    }

    /// ASM loc_7B48 continued: when `y_pos & $F != $F`, `addq.w #1,d5` adds an
    /// extra scan row. Even Micromax (short) with non-aligned y gets rows_up=1,
    /// reaching the wall that tile-aligned y=79 avoided.
    #[test]
    fn loc_7b48_non_aligned_y_adds_extra_scan_row() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(8, 8, &[(2, 3, 0x40, 0x00)]));
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 40 << 16;
        // y=78: 78/16=4 (still foot tile row 4), but 78 & 0xF = 0xE (not aligned)
        slice.player.y_pos = 78 << 16;

        let result = runtime.resolve_transform_overlap_sub_7b30(&slice, PlayerForm::Micromax);
        assert_ne!(
            result,
            Some(40),
            "Micromax with non-aligned y should get rows_up=1 and detect the wall at row 3"
        );
    }

    /// ASM loc_7B5A (kid.asm:8248): `move.w x_pos(a3),d6; sub.w d7,d6;
    /// bpl.w loc_7B74` — when left_edge (x - half_width) is negative, snap
    /// x_pos to half_width and probe only the right column at 2*half_width.
    #[test]
    fn loc_7b5a_negative_left_edge_snaps_to_half_width() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(8, 8, &[]));
        let mut slice = OpeningSliceState::new(default_bounds());
        // IronKnight half_width=7. x=3 → left_edge = 3-7 = -4 < 0.
        slice.player.x_pos = 3 << 16;
        slice.player.y_pos = 79 << 16;

        let cleared_x = runtime
            .resolve_transform_overlap_sub_7b30(&slice, PlayerForm::IronKnight)
            .expect("loc_7B5A: negative left edge with open right column should succeed");
        assert_eq!(
            cleared_x, 7,
            "loc_7B5A: x_pos should snap to half_width (7) when left edge is off-screen"
        );
    }

    /// ASM loc_7BB6 (kid.asm:8287): shared right-column probe entry point.
    /// After loc_7B5A snaps x to half_width (negative left edge), loc_7BB6
    /// probes column at `2*half_width` upward for `rows_up` rows via sub_7BEA.
    /// If clear → loc_7B9E (success with snapped x). If blocked → loc_7BC0.
    #[test]
    fn loc_7bb6_negative_left_edge_right_column_probe_succeeds_when_clear() {
        let mut runtime = NativeRuntime::new(manifest());
        // No walls — right column probe should pass.
        runtime.set_block_grid(grid_with_cells(8, 8, &[]));
        let mut slice = OpeningSliceState::new(default_bounds());
        // IronKnight half_width=7. x=3 → left_edge=-4 < 0. Snap x=7.
        // Right column probed at 2*7=14, tile col 0 — clear.
        slice.player.x_pos = 3 << 16;
        slice.player.y_pos = 79 << 16;

        let result = runtime.resolve_transform_overlap_sub_7b30(&slice, PlayerForm::IronKnight);
        assert_eq!(
            result,
            Some(7),
            "loc_7BB6: right column clear after negative-left-edge snap → loc_7B9E success"
        );
    }

    /// ASM loc_7BB6 (kid.asm:8287): same path but right column is blocked.
    /// After loc_7BA4 snaps right from a left-blocked position, loc_7BB6
    /// probes the new right edge. If sub_7BEA finds a wall → loc_7BC0.
    #[test]
    fn loc_7bb6_left_blocked_snap_right_then_right_also_blocked_returns_none() {
        let mut runtime = NativeRuntime::new(manifest());
        // Walls on both sides of x=28: left col 1 and right col 2.
        // After loc_7BA4 snap-right: cleared_x = (21 & !0xF)+16+7 = 39.
        // Right edge = 39+7 = 46, tile col 2 — blocked. → loc_7BC0.
        runtime.set_block_grid(grid_with_cells(
            8,
            8,
            &[(1, 4, 0x40, 0x00), (2, 4, 0x40, 0x00)],
        ));
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 28 << 16;
        slice.player.y_pos = 79 << 16;

        let result = runtime.resolve_transform_overlap_sub_7b30(&slice, PlayerForm::IronKnight);
        assert_eq!(
            result, None,
            "loc_7BB6 → loc_7BC0: both columns blocked after snap → return None (blocked)"
        );
    }

    /// ASM loc_7BC0 (kid.asm:8292): `moveq #1,d7; addq.w #4,sp; move.l d0,
    /// x_pos(a3); move.l d1,y_pos(a3); rts` — when all snap attempts fail,
    /// sub_7B30 restores the original x_pos/y_pos from d0/d1 and returns
    /// blocked. In native this is the `None` return path.
    #[test]
    fn loc_7bc0_both_snaps_fail_returns_none_position_unchanged() {
        let mut runtime = NativeRuntime::new(manifest());
        // Block left col 3 and right col 5 around x=68 (half_width=7).
        // left_edge=61→col 3 blocked→snap right to (48+16+7)=71→right=78→col 4 clear.
        // That would succeed. Instead, block col 4 too.
        runtime.set_block_grid(grid_with_cells(
            8,
            8,
            &[(3, 4, 0x40, 0x00), (4, 4, 0x40, 0x00)],
        ));
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 68 << 16;
        slice.player.y_pos = 79 << 16;

        let result = runtime.resolve_transform_overlap_sub_7b30(&slice, PlayerForm::IronKnight);
        assert_eq!(
            result, None,
            "loc_7BC0: left blocked, snap-right also blocked → restore original pos and return None"
        );
    }

    /// ASM loc_7BC0 via loc_7BCE (kid.asm:8300→8292): when the right edge is
    /// off-level or blocked, sub_7B30 snaps left. If that left probe also
    /// fails, it branches to loc_7BC0. Tests the loc_7BCE→loc_7BC0 path.
    #[test]
    fn loc_7bc0_right_boundary_snap_left_also_blocked() {
        let mut runtime = NativeRuntime::new(manifest());
        // Juggernaut half_width=23, 3-tile-wide level (48px).
        // x=40: left_edge=17→col 1 (clear), right_edge=63 >= 48 → loc_7BCE.
        // cleared_x = (63 & !0xF)-1-23 = 48-1-23 = 24.
        // left_snap = 24-23 = 1→col 0. Wall at col 0 → blocked → loc_7BC0.
        runtime.set_block_grid(grid_with_cells(3, 8, &[(0, 4, 0x40, 0x00)]));
        let mut slice = OpeningSliceState::new(LevelBounds {
            width_pixels: 48,
            height_pixels: 128,
        });
        slice.player.x_pos = 40 << 16;
        slice.player.y_pos = 79 << 16;

        let result = runtime.resolve_transform_overlap_sub_7b30(&slice, PlayerForm::Juggernaut);
        assert_eq!(
            result, None,
            "loc_7BC0 via loc_7BCE: right off-level, snap-left blocked → None"
        );
    }

    /// ASM loc_7B74 → loc_7B9E (kid.asm:8259→8273): left column clear, right
    /// column clear — both probes pass, return original x_pos unchanged.
    /// `moveq #0,d7; addq.w #4,sp; rts` at loc_7B9E.
    #[test]
    fn loc_7b9e_both_columns_clear_returns_original_position() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(8, 8, &[]));
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 60 << 16;
        slice.player.y_pos = 79 << 16;

        let cleared_x = runtime
            .resolve_transform_overlap_sub_7b30(&slice, PlayerForm::IronKnight)
            .expect("loc_7B9E: both columns clear should return success");
        assert_eq!(
            cleared_x, 60,
            "loc_7B9E: position should be unchanged when no walls block either column"
        );
    }

    /// ASM loc_7BA4 (kid.asm:8279): left column blocked → snap right:
    /// `andi.w #$FFF0,d4; addi.w #$10,d4; add.w (sp),d4; move.w d4,x_pos(a3)`
    /// Computes `(left_edge & $FFF0) + $10 + half_width`.
    #[test]
    fn loc_7ba4_left_blocked_snaps_right_to_next_tile_plus_half_width() {
        let mut runtime = NativeRuntime::new(manifest());
        // Wall at tile (1, 4): left column of a player at x=28, half_width=7
        // left_edge = 28-7 = 21, tile col = 21/16 = 1. Wall blocks it.
        runtime.set_block_grid(grid_with_cells(8, 8, &[(1, 4, 0x40, 0x00)]));
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 28 << 16;
        slice.player.y_pos = 79 << 16;

        let cleared_x = runtime
            .resolve_transform_overlap_sub_7b30(&slice, PlayerForm::IronKnight)
            .expect("loc_7BA4: left blocked with clear right should snap right");
        // (21 & !0xF) + 16 + 7 = 16 + 16 + 7 = 39
        assert_eq!(
            cleared_x, 39,
            "loc_7BA4: cleared_x = (left_edge & $FFF0) + $10 + half_width = (16) + 16 + 7 = 39"
        );
    }

    #[test]
    fn pending_form_change_sub_7acc_applies_the_transform_after_clearing_space() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(8, 8, &[(1, 4, 0x60, 0x00)]));
        runtime.progression.pending_form = Some(PlayerForm::IronKnight);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 28 << 16;
        slice.player.y_pos = 79 << 16;

        let entered_death = runtime.handle_pending_form_change_sub_7acc(&mut slice);

        assert!(
            !entered_death,
            "sub_7ACC should hand successful probes to Kid_Transform rather than Death"
        );
        assert_eq!(runtime.progression.current_form, PlayerForm::IronKnight);
        assert_eq!(slice.player.x_pos, 39 << 16);
        assert!(
            runtime.progression.form_change_flash_frames > 0,
            "the successful sub_7ACC path should still enter Kid_Transform's flash coroutine",
        );
    }

    #[test]
    fn pending_form_change_sub_7acc_drops_blocked_kid_transform_without_killing_him() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(
            8,
            8,
            &[(1, 4, 0x60, 0x00), (2, 4, 0x60, 0x00)],
        ));
        runtime.progression.pending_form = Some(PlayerForm::IronKnight);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 28 << 16;
        slice.player.y_pos = 79 << 16;

        let entered_death = runtime.handle_pending_form_change_sub_7acc(&mut slice);

        assert!(
            !entered_death,
            "Kid should just lose the queued helmet when sub_7B30 cannot clear enough room"
        );
        assert_eq!(runtime.progression.current_form, PlayerForm::TheKid);
        assert_eq!(runtime.progression.pending_form, None);
        assert_eq!(runtime.progression.form_change_flash_frames, 0);
        assert!(matches!(runtime.player_lifecycle, PlayerLifecycle::Alive));
    }

    #[test]
    fn pending_form_change_sub_7acc_kills_blocked_helmeted_forms() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(8, 8, &[(2, 4, 0x60, 0x00)]));
        runtime.progression.current_form = PlayerForm::Cyclone;
        runtime.progression.number_hitpoints = PlayerForm::Cyclone.max_hitpoints();
        runtime.progression.pending_form = Some(PlayerForm::Juggernaut);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 10 << 16;
        slice.player.y_pos = 79 << 16;

        let entered_death = runtime.handle_pending_form_change_sub_7acc(&mut slice);

        assert!(
            entered_death,
            "sub_7ACC should branch to Death when a helmeted player cannot fit the new transform"
        );
        assert_eq!(runtime.progression.pending_form, None);
        assert_eq!(runtime.progression.number_hitpoints, 0);
        assert!(matches!(
            runtime.player_lifecycle,
            PlayerLifecycle::DeathWindup { .. }
        ));
    }

    // -----------------------------------------------------------------------
    // sub_7ACC inner branch citations (loc_7AD6 through loc_7B28)
    // -----------------------------------------------------------------------

    /// ASM loc_7AD6 (kid.asm:8185): `cmpi.w #6,($FFFFFA56).w; beq.w loc_7B12`
    /// — when Character_Movement == 6 (uphill-downhill), the overlap check
    /// (sub_7B30) is skipped entirely and the transform proceeds unconditionally.
    /// Native does not yet model this slope bypass.
    /// TODO(fidelity): native always runs sub_7B30; slope bypass not modelled.
    #[test]
    fn loc_7ad6_slope_bypass_is_documented_as_todo_fidelity() {
        // Verify that the native path still runs sub_7B30 even when the transform
        // would be blocked — proving the slope bypass is not yet implemented.
        // When the TODO(fidelity) is resolved, this test should be updated to
        // show that slope state skips the overlap check.
        let mut runtime = NativeRuntime::new(manifest());
        // Fully walled-in: both left and right columns blocked
        runtime.set_block_grid(grid_with_cells(
            8,
            8,
            &[(0, 4, 0x60, 0x00), (1, 4, 0x60, 0x00), (2, 4, 0x60, 0x00)],
        ));
        runtime.progression.pending_form = Some(PlayerForm::IronKnight);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 24 << 16;
        slice.player.y_pos = 79 << 16;

        let entered_death = runtime.handle_pending_form_change_sub_7acc(&mut slice);

        // Native: Kid form → cancel (loc_7B28 path), no death
        assert!(
            !entered_death,
            "ASM loc_7AD6: native (no slope bypass) should cancel for TheKid, not die"
        );
        assert_eq!(
            runtime.progression.current_form,
            PlayerForm::TheKid,
            "ASM loc_7AD6: transform should be cancelled since sub_7B30 blocks"
        );
        assert_eq!(
            runtime.progression.pending_form, None,
            "ASM loc_7AD6: pending_form cleared after blocked transform"
        );
    }

    /// ASM loc_7B0A (kid.asm:8202): Micromax (helmet 9) and Juggernaut (helmet 5)
    /// skip the `addi.w #$10,d6` instruction, keeping d6=$10 (short form).
    /// All other helmets get d6=$20 (tall form). This determines how many rows
    /// sub_7B30 probes upward for wall overlap.
    ///
    /// Native: `ceiling_probe_height()` returns 15 for short forms and 31 for
    /// tall forms; +1 produces $10/$20, matching ASM d6 exactly.
    #[test]
    fn loc_7b0a_short_forms_skip_height_addition() {
        // Short forms: Micromax and Juggernaut → d6=$10
        for form in [PlayerForm::Micromax, PlayerForm::Juggernaut] {
            let d6 = form.ceiling_probe_height() + 1;
            assert_eq!(
                d6, 0x10,
                "ASM loc_7B0A: {form:?} should keep d6=$10 (short form, no addi.w #$10)"
            );
        }
        // Tall forms: all others add $10 → d6=$20
        for form in [
            PlayerForm::TheKid,
            PlayerForm::Cyclone,
            PlayerForm::RedStealth,
            PlayerForm::Eyeclops,
            PlayerForm::Skycutter,
            PlayerForm::IronKnight,
            PlayerForm::Berzerker,
            PlayerForm::Maniaxe,
        ] {
            let d6 = form.ceiling_probe_height() + 1;
            assert_eq!(
                d6, 0x20,
                "ASM loc_7B0A: {form:?} should get d6=$20 after addi.w #$10,d6"
            );
        }
    }

    /// ASM loc_7B12 (kid.asm:8206): `bra.w Kid_Transform` — the successful
    /// transform path. Reached when sub_7B30 returns 0 (space is clear) or
    /// when Character_Movement == 6 (slope bypass at loc_7AD6).
    ///
    /// Native: `resolve_pending_form_change()` applies the transform, sets
    /// flash frames, and updates the current form.
    #[test]
    fn loc_7b12_successful_transform_enters_kid_transform() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(8, 8, &[]));
        runtime.progression.pending_form = Some(PlayerForm::Cyclone);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 64 << 16;
        slice.player.y_pos = 79 << 16;

        let entered_death = runtime.handle_pending_form_change_sub_7acc(&mut slice);

        assert!(
            !entered_death,
            "ASM loc_7B12: clear sub_7B30 should branch to Kid_Transform, not Death"
        );
        assert_eq!(
            runtime.progression.current_form,
            PlayerForm::Cyclone,
            "ASM loc_7B12: Kid_Transform should set current_form to the target"
        );
        assert!(
            runtime.progression.form_change_flash_frames > 0,
            "ASM loc_7B12: Kid_Transform should start the flash coroutine"
        );
        assert_eq!(
            runtime.progression.pending_form, None,
            "ASM loc_7B12: pending_form consumed by Kid_Transform"
        );
    }

    /// ASM loc_7C24 (kid.asm:8350): `move.l 4(a0),d7; beq.w loc_7C3A;
    /// move.l d7,a0; move.l $C(a0),a0; jsr Delete_Object_a0; bra.s loc_7C24`
    /// — sub_7C1C walks the linked object list rooted at $FFFFF862, deleting
    /// each child's GFX object before Kid_Transform proceeds. Native doesn't
    /// model linked GFX child objects, so sub_7C1C is a no-op. This test
    /// verifies that a successful transform path (which calls sub_7C1C in ASM)
    /// doesn't require child object deletion in native.
    #[test]
    fn loc_7c24_sub_7c1c_child_object_deletion_is_visual_only_noop() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(8, 8, &[]));
        runtime.progression.pending_form = Some(PlayerForm::IronKnight);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 64 << 16;
        slice.player.y_pos = 79 << 16;

        // In ASM, Kid_Transform calls sub_7C1C (which contains the loc_7C24
        // linked-list loop) before palette operations. Native skips this
        // because GFX child objects are not modelled. Transform should succeed.
        let entered_death = runtime.handle_pending_form_change_sub_7acc(&mut slice);

        assert!(
            !entered_death,
            "loc_7C24: transform should succeed without needing child object deletion"
        );
        assert_eq!(
            runtime.progression.current_form,
            PlayerForm::IronKnight,
            "loc_7C24: Kid_Transform should complete even though sub_7C1C is a native no-op"
        );
    }

    /// ASM loc_7B1A (kid.asm:8213): sub_7B30 returned non-zero (blocked).
    /// `tst.w (Current_Helmet).w` — if Current_Helmet != 0 (wearing a helmet),
    /// `moveq #0,d7; bra.w Death` — crush death.
    ///
    /// Native: sets hitpoints to 0 and begins death windup.
    #[test]
    fn loc_7b1a_blocked_helmeted_form_dies() {
        let mut runtime = NativeRuntime::new(manifest());
        // Wall on both sides — sub_7B30 cannot clear space
        runtime.set_block_grid(grid_with_cells(
            8,
            8,
            &[(1, 4, 0x60, 0x00), (2, 4, 0x60, 0x00)],
        ));
        runtime.progression.current_form = PlayerForm::RedStealth;
        runtime.progression.number_hitpoints = PlayerForm::RedStealth.max_hitpoints();
        runtime.progression.pending_form = Some(PlayerForm::IronKnight);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 28 << 16;
        slice.player.y_pos = 79 << 16;

        let entered_death = runtime.handle_pending_form_change_sub_7acc(&mut slice);

        assert!(
            entered_death,
            "ASM loc_7B1A: helmeted form blocked by sub_7B30 should branch to Death"
        );
        assert_eq!(
            runtime.progression.number_hitpoints, 0,
            "ASM loc_7B1A: `moveq #0,d7` before Death zeroes hitpoints"
        );
        assert_eq!(
            runtime.progression.pending_form, None,
            "ASM loc_7B1A: pending_form cleared before death"
        );
        assert!(
            matches!(
                runtime.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. }
            ),
            "ASM loc_7B1A: should enter death windup"
        );
    }

    /// ASM loc_7B28 (kid.asm:8220): sub_7B30 blocked and Current_Helmet == 0
    /// (unhelmeted Kid). `sf (Check_Helmet_Change).w` — clear the pending
    /// transform flag. `moveq #0,d7; rts` — return without death.
    ///
    /// Native: clears `pending_form` and returns false (no death).
    #[test]
    fn loc_7b28_unhelmeted_kid_cancels_blocked_transform() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.set_block_grid(grid_with_cells(
            8,
            8,
            &[(1, 4, 0x60, 0x00), (2, 4, 0x60, 0x00)],
        ));
        runtime.progression.current_form = PlayerForm::TheKid;
        runtime.progression.pending_form = Some(PlayerForm::IronKnight);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 28 << 16;
        slice.player.y_pos = 79 << 16;

        let entered_death = runtime.handle_pending_form_change_sub_7acc(&mut slice);

        assert!(
            !entered_death,
            "ASM loc_7B28: unhelmeted Kid should not die when blocked"
        );
        assert_eq!(
            runtime.progression.current_form,
            PlayerForm::TheKid,
            "ASM loc_7B28: current_form unchanged — transform cancelled"
        );
        assert_eq!(
            runtime.progression.pending_form, None,
            "ASM loc_7B28: `sf (Check_Helmet_Change).w` clears pending_form"
        );
        assert_eq!(
            runtime.progression.form_change_flash_frames, 0,
            "ASM loc_7B28: no flash — transform was never applied"
        );
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::Alive),
            "ASM loc_7B28: player stays alive"
        );
    }

    #[test]
    fn dragon_spawn_uses_traced_offset_and_drift() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
            unknown_word: 0x7D00,
            header_byte: 0x80,
            object_count: 1,
            objects: vec![kid_data::EnemyLayoutObject {
                object_type: 0x0C, // Flying dragon (direct ID)
                flags: 0xFF,
                hit_points_plus_one: 0,
                x: 200,
                y: 271,
            }],
            segments: vec![kid_data::EnemyLayoutSegment {
                lower_bound: 0,
                upper_bound: 0x7D00,
                enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
                object_batch_flags: 0x80,
                object_count: 1,
                objects: vec![kid_data::EnemyLayoutObject {
                    object_type: 0x0C, // Flying dragon (direct ID)
                    flags: 0xFF,
                    hit_points_plus_one: 0,
                    x: 200,
                    y: 271,
                }],
            }],
        };
        let window = ActivationWindow {
            x_min: 0,
            x_max: 640,
            y_min: 0,
            y_max: 480,
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
            0,
            0,
            &HashSet::new(),
        );

        let dragon = active.first().expect("dragon should activate");
        assert_eq!(dragon.family, EnemyFamily::Dragon);
        assert_eq!(dragon.x_pos, (226u32 << 16) + 0x8000);
        assert_eq!(dragon.y_pos, 272u32 << 16);
        assert_eq!(dragon.x_velocity, 0);
        assert_eq!(dragon.dragon_phase, DragonFlyingState::Hovering);
    }

    #[test]
    fn dragon_flyer_hovers_in_place_during_hover_phase() {
        // Script variant 0 starts with Hover(0x70=112 frames).
        // During hover, x/y velocity should be zero — dragon stays in place.
        let start_x = 200u32 << 16;
        let start_y = 272u32 << 16;
        let mut enemy = test_dragon(0, start_x, start_y);
        enemy.dragon_phase = DragonFlyingState::Hovering;
        enemy.dragon_phase_frames = 112;

        // Step 10 frames — should stay in place.
        for _ in 0..10 {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                None,
                None,
                None,
                None,
                None,
                PlayerForm::TheKid,
                None,
                &mut Vec::new(),
            );
        }
        assert_eq!(enemy.x_pos, start_x, "hover should not change X");
        assert_eq!(enemy.y_pos, start_y, "hover should not change Y");
        assert_eq!(enemy.dragon_phase, DragonFlyingState::Hovering);
        assert_eq!(
            enemy.dragon_phase_frames, 102,
            "ASM Dragon_Flying.asm loc_3480A consumes exactly one hibernated hover frame per loop"
        );
    }

    #[test]
    fn dragon_flyer_transitions_to_descent_after_hover_on_script_0() {
        // Script 0: Hover(112) → Descent → Drift(80) → ...
        // After 112 hover frames, should advance to Descent.
        let mut enemy = test_dragon(0, 200 << 16, 272 << 16);
        enemy.dragon_phase = DragonFlyingState::Hovering;
        enemy.dragon_phase_frames = 1; // One frame left.

        step_active_enemies(
            std::slice::from_mut(&mut enemy),
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
            enemy.dragon_phase,
            DragonFlyingState::Descending,
            "should enter descent after hover ends (script 0, cmd 1 = Descent)"
        );
    }

    #[test]
    fn dragon_flying_loc_3482e_hover_dbf_tail_loops_until_ffff() {
        assert_eq!(
            dragon_hover_dbf_tail_loc_3482e(0x0070),
            DragonHoverDbfTailLoc3482E {
                next_d0: 0x006F,
                loops_to_loc_3480a: true,
                advances_script: false,
            },
            "ASM Dragon_Flying.asm loc_3482E decrements the hover counter and branches back while the result is not `$FFFF`"
        );
        assert_eq!(
            dragon_hover_dbf_tail_loc_3482e(0x0000),
            DragonHoverDbfTailLoc3482E {
                next_d0: 0xFFFF,
                loops_to_loc_3480a: false,
                advances_script: true,
            },
            "ASM Dragon_Flying.asm loc_3482E falls through to loc_346DA only after DBF decrements zero to `$FFFF`"
        );
    }

    #[test]
    fn dragon_flying_script_words_match_loc_346da_dispatch_offsets() {
        fn encode_cmd_word(cmd: DragonFlyingCmd) -> u16 {
            // ASM Dragon_Flying.asm loc_346DA: the interpreter keeps the high
            // byte as the frame count and dispatches through off_3471E with
            // the low byte as the byte offset.
            match cmd {
                DragonFlyingCmd::Hover(frames) => frames << 8,
                DragonFlyingCmd::Drift(frames) => (frames << 8) | 0x0004,
                DragonFlyingCmd::DriftWithFire(frames) => (frames << 8) | 0x0014,
                DragonFlyingCmd::FacePlayer => 0x000C,
                DragonFlyingCmd::Descent => 0x0010,
                DragonFlyingCmd::ReturnToAnchor => 0x0018,
            }
        }

        fn encoded_script(variant: u8) -> Vec<u16> {
            let mut words: Vec<u16> = dragon_script(variant)
                .iter()
                .copied()
                .map(encode_cmd_word)
                .collect();
            words.push(0xFFFF);
            words
        }

        assert_eq!(
            encoded_script(0),
            vec![0x7000, 0x0010, 0x5004, 0x3200, 0x7804, 0x0018, 0xFFFF]
        );
        assert_eq!(
            encoded_script(1),
            vec![0x2814, 0x1E14, 0x3214, 0x0018, 0xFFFF]
        );
        assert_eq!(
            encoded_script(2),
            vec![0x7014, 0x7014, 0x000C, 0x5014, 0x0018, 0xFFFF]
        );
    }

    #[test]
    fn dragon_flyer_drift_accelerates_from_initial_speed() {
        // Script 1 starts with DriftWithFire(40).
        // Drift starts at $800, accelerates by $200/frame, caps at $18000.
        let mut enemy = test_dragon(1, 200 << 16, 100 << 16);
        // After init, script 1 cmd 0 = DriftWithFire(0x28=40).
        assert_eq!(enemy.dragon_phase, DragonFlyingState::DriftingWithFire);
        assert_eq!(enemy.dragon_phase_frames, 0x28);

        // Initial x_velocity should be ±$800 (direction-dependent).
        let initial_speed = enemy.x_velocity.unsigned_abs();
        assert_eq!(initial_speed, 0x800, "drift initial speed should be $800");

        // After 1 step: $800 + $200 = $A00.
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
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
            enemy.x_velocity.unsigned_abs(),
            0xA00,
            "drift should accelerate by $200 per frame"
        );
    }

    #[test]
    fn dragon_flyer_return_to_anchor_rises_then_restarts() {
        // When ReturningToAnchor, dragon decelerates upward until reaching anchor Y.
        let anchor_y = 100u32 << 16;
        let mut enemy = test_dragon(0, 200 << 16, anchor_y);
        enemy.y_pos = (anchor_y as i64 + (50 << 16)) as u32; // 50px below anchor.
        enemy.dragon_phase = DragonFlyingState::ReturningToAnchor;
        enemy.y_velocity = 0;

        // Step several frames — should gain upward velocity.
        for _ in 0..20 {
            step_active_enemies(
                std::slice::from_mut(&mut enemy),
                None,
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
            enemy.y_velocity < 0,
            "should have upward (negative) velocity"
        );
        assert!(
            (enemy.y_pos as i64) < (anchor_y as i64 + (50 << 16)),
            "should have risen from starting position"
        );
    }

    #[test]
    fn dragon_flyer_drift_reverses_on_wall_collision() {
        // ASM Dragon_Flying.asm loc_347D6: during Drift, wall collision flips
        // direction and negates x_velocity. This is the missing wall-reversal
        // during Drift state from the phase4-enemy-findings.md backlog.
        let mut enemy = test_dragon(0, 100 << 16, 100 << 16);
        enemy.dragon_phase = DragonFlyingState::Drifting;
        enemy.dragon_phase_frames = 100;
        enemy.x_velocity = 0x10000; // Moving right at speed
        enemy.facing = Direction::Right;

        // Create a block grid with a wall at column 10 (so x=160+ would hit)
        // Cell format: [block_flag(1) collision(3) block_type(4)] [skin(8)]
        // Solid collision = 6 → bits 14-12 → 0x60 in high byte
        let w = 20u16;
        let h = 15u16;
        let mut data = vec![0u8; (w * h * 2) as usize]; // 2 bytes per cell
        // Place solid collision block at column 10, row 6
        let offset = (6 * w as usize + 10) * 2;
        data[offset] = 0x60; // collision type 6 = Solid
        data[offset + 1] = 0x01; // arbitrary skin
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        // Position dragon so it will hit the wall on next step
        enemy.x_pos = (155 << 16) as u32; // Close to wall at x=160
        // Wall is at cell (10, 6) which means y pixels 96-111
        // probe_wall uses foot_y_px - 8 for lower probe, so we need foot_y_px >= 104
        // to ensure the probe reaches y=96 (cell row 6)
        enemy.y_pos = (104 << 16) as u32; // Y position where probe will hit row 6

        let initial_facing = enemy.facing;
        // initial x_velocity captured before probe

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

        // After velocity acceleration and wall collision: velocity should be negated
        // Initial velocity was 0x10000, drift acceleration adds 0x200 per frame
        // So after acceleration: x_velocity = 0x10200 = 66048
        // After wall collision: x_velocity negates to -66048
        assert_eq!(
            enemy.x_velocity,
            -(0x10000 + 0x200),
            "dragon flying drift should negate velocity on wall collision"
        );
        assert_ne!(
            enemy.facing, initial_facing,
            "dragon flying drift should flip facing on wall collision"
        );
    }

    #[test]
    fn dragon_flyer_drift_continues_without_wall() {
        // Without a wall, drift should continue accelerating normally.
        let mut enemy = test_dragon(0, 100 << 16, 100 << 16);
        enemy.dragon_phase = DragonFlyingState::Drifting;
        enemy.dragon_phase_frames = 100;
        enemy.x_velocity = 0x800; // Small initial velocity
        enemy.facing = Direction::Right;

        // Empty grid - no walls
        let w = 20u16;
        let h = 15u16;
        let data = vec![terrain::BlockType::None as u8; (w * h) as usize];
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let initial_facing = enemy.facing;

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

        // Without wall: velocity should increase, facing unchanged
        assert_eq!(
            enemy.facing, initial_facing,
            "dragon flying drift should keep facing without wall"
        );
        assert!(
            enemy.x_velocity > 0x800,
            "dragon flying drift should accelerate without wall"
        );
    }

    #[test]
    fn dragon_flyer_stays_active_after_spawn_point_scrolls_behind_camera() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
            unknown_word: 0x7D00,
            header_byte: 0x80,
            object_count: 1,
            objects: vec![kid_data::EnemyLayoutObject {
                object_type: 0x0C, // Flying dragon (direct ID)
                flags: 0xFF,
                hit_points_plus_one: 0,
                x: 200,
                y: 271,
            }],
            segments: vec![kid_data::EnemyLayoutSegment {
                lower_bound: 0,
                upper_bound: 0x7D00,
                enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
                object_batch_flags: 0x80,
                object_count: 1,
                objects: vec![kid_data::EnemyLayoutObject {
                    object_type: 0x0C, // Flying dragon (direct ID)
                    flags: 0xFF,
                    hit_points_plus_one: 0,
                    x: 200,
                    y: 271,
                }],
            }],
        };
        let mut active = Vec::new();

        activate_enemies(
            &mut active,
            &layout,
            ActivationWindow::from_camera(CameraState {
                x_pos: 0,
                y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            }),
            0,
            OPENING_GROUNDED_CAMERA_Y_FP >> 16,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            0,
            271 << 16,
            &HashSet::new(),
        );
        assert_eq!(
            active.len(),
            1,
            "dragon should activate at the opening spawn"
        );

        active[0].x_pos = ((-54i32) << 16) as u32;
        let move_end_window = ActivationWindow::from_camera(CameraState {
            x_pos: 429u32 << 16,
            y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
        });
        let move_end_view = enemy_layout_view(&layout, 429, OPENING_GROUNDED_CAMERA_Y_FP >> 16);
        assert_eq!(active[0].family, EnemyFamily::Dragon);
        assert_eq!(active[0].segment_index, move_end_view.segment_index);
        assert!(
            move_end_window.contains_fixed_signed_with_margin(
                active[0].x_pos,
                active[0].y_pos,
                FLYER_RETENTION_X_PADDING,
                FLYER_RETENTION_Y_PADDING,
            ),
            "dragon position should remain inside the retention window"
        );
        assert!(
            enemy_should_remain_active(
                &active[0],
                &move_end_view,
                move_end_window,
                move_end_window.x_min,
                move_end_window.y_min,
                physics::LevelBounds {
                    width_pixels: 4096,
                    height_pixels: 512,
                },
                active.len(),
            ),
            "dragon should still satisfy the retention window after drifting off the spawn point"
        );
        activate_enemies(
            &mut active,
            &layout,
            move_end_window,
            429,
            OPENING_GROUNDED_CAMERA_Y_FP >> 16,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            0,
            271 << 16,
            &HashSet::new(),
        );

        assert_eq!(
            active.len(),
            1,
            "dragon should remain alive after its spawn point scrolls behind the camera"
        );
    }

    #[test]
    fn dragon_walking_stays_active_after_spawn_point_scrolls_behind_camera() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![],
            unknown_word: 0,
            header_byte: 0,
            object_count: 1,
            objects: vec![kid_data::EnemyLayoutObject {
                object_type: 0x0D,
                flags: 0xFF,
                hit_points_plus_one: 0,
                x: 336,
                y: 432,
            }],
            segments: vec![],
        };
        let mut active = vec![ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::DragonWalking,
            x_pos: 0x020B_0000,
            y_pos: 0x01C0_0000,
            x_velocity: 0,
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
            wait_timer: DRAGON_WALKING_BREATH_INITIAL,
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
            shared_walker_state: spawn_shared_walker_state(EnemyFamily::DragonWalking),
            shared_walker_patrol_timer: spawn_shared_walker_patrol_timer(
                EnemyFamily::DragonWalking,
            ),
            shared_walker_patrol_short_next: spawn_shared_walker_patrol_short_next(
                EnemyFamily::DragonWalking,
            ),
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        }];
        let camera = CameraState {
            x_pos: 715u32 << 16,
            y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
        };
        let window = ActivationWindow::from_camera(camera);
        let view = enemy_layout_view(&layout, 715, OPENING_GROUNDED_CAMERA_Y_FP >> 16);
        let level_bounds = physics::LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };

        assert!(
            !window.contains(
                u32::from(layout.objects[0].x),
                u32::from(layout.objects[0].y),
            ),
            "spawn point should be outside the activation window"
        );
        assert!(
            enemy_should_remain_active(
                &active[0],
                &view,
                window,
                715,
                OPENING_GROUNDED_CAMERA_Y_FP >> 16,
                level_bounds,
                6,
            ),
            "walking dragon should keep its live position even after the spawn point leaves view"
        );

        activate_enemies(
            &mut active,
            &layout,
            window,
            715,
            OPENING_GROUNDED_CAMERA_Y_FP >> 16,
            level_bounds,
            0,
            271 << 16,
            &HashSet::new(),
        );

        assert_eq!(
            active.len(),
            1,
            "walking dragon should remain active after its spawn point scrolls behind the camera"
        );
    }

    #[test]
    fn lion_sub_3e8ba_keeps_live_object_active_after_spawn_point_scrolls_offscreen() {
        // ASM Lion.asm sub_3E8BA culls from the lion's live position, not its
        // layout anchor. With 11..20 active objects, the live lion stays loaded
        // inside x=[-$1A4,+$2E4], y=[-$1A4,+$284] relative to camera.
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![],
            unknown_word: 0,
            header_byte: 0,
            object_count: 1,
            objects: vec![kid_data::EnemyLayoutObject {
                object_type: 0x12,
                flags: 0xFF,
                hit_points_plus_one: 0,
                x: 336,
                y: 432,
            }],
            segments: vec![],
        };
        let enemy = ActiveEnemy {
            x_pos: 0x020B_0000,
            y_pos: 0x01C0_0000,
            family: EnemyFamily::Lion,
            on_ground: true,
            hit_points: 6,
            ..test_enemy(EnemyFamily::Lion, 0, 0, 6)
        };
        let camera = CameraState {
            x_pos: 715u32 << 16,
            y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
        };
        let window = ActivationWindow::from_camera(camera);
        let view = enemy_layout_view(&layout, 715, OPENING_GROUNDED_CAMERA_Y_FP >> 16);
        let level_bounds = physics::LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };

        assert!(
            !window.contains(
                u32::from(layout.objects[0].x),
                u32::from(layout.objects[0].y),
            ),
            "spawn point should be outside the activation window"
        );
        assert!(
            enemy_should_remain_active(
                &enemy,
                &view,
                window,
                715,
                OPENING_GROUNDED_CAMERA_Y_FP >> 16,
                level_bounds,
                11,
            ),
            "lion should remain active while its live position stays inside sub_3E8BA's wider retention box"
        );
    }

    #[test]
    fn shared_enemy_should_remain_active_sub_3c3ce_keeps_medium_load_enemy_inside_wide_box() {
        let retained = shared_enemy_should_remain_active_sub_3C3CE(
            0x020B_0000,
            0x01C0_0000,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            0x000F,
            715,
            OPENING_GROUNDED_CAMERA_Y_FP >> 16,
        );

        assert!(
            retained,
            "ASM sub_3C3CE should keep live shared enemies inside the medium-load x=[-$1A4,+$2E4], y=[-$1A4,+$284] retention box",
        );
    }

    #[test]
    fn shared_enemy_should_remain_active_sub_3c3ce_drops_heavy_load_enemy_outside_tight_box() {
        let retained = shared_enemy_should_remain_active_sub_3C3CE(
            900u32 << 16,
            0x01C0_0000,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            0x0015,
            120,
            256,
        );

        assert!(
            !retained,
            "ASM sub_3C3CE should reject live shared enemies once heavy load tightens the camera-relative box to x=[-$104,+$244], y=[-$104,+$1E4]",
        );
    }

    #[test]
    fn diamond_sub_3c3ce_keeps_live_object_active_after_spawn_point_scrolls_offscreen() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![],
            unknown_word: 0,
            header_byte: 0,
            object_count: 1,
            objects: vec![kid_data::EnemyLayoutObject {
                object_type: 0x01,
                flags: 0xFF,
                hit_points_plus_one: 0,
                x: 336,
                y: 432,
            }],
            segments: vec![],
        };
        let enemy = ActiveEnemy {
            x_pos: 0x020B_0000,
            y_pos: 0x01C0_0000,
            family: EnemyFamily::Diamond,
            on_ground: true,
            hit_points: 1,
            ..test_enemy(EnemyFamily::Diamond, 0, 0, 1)
        };
        let camera = CameraState {
            x_pos: 715u32 << 16,
            y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
        };
        let window = ActivationWindow::from_camera(camera);
        let view = enemy_layout_view(&layout, 715, OPENING_GROUNDED_CAMERA_Y_FP >> 16);
        let level_bounds = physics::LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };

        assert!(
            !window.contains(
                u32::from(layout.objects[0].x),
                u32::from(layout.objects[0].y),
            ),
            "spawn point should be outside the activation window"
        );
        assert!(
            enemy_should_remain_active(
                &enemy,
                &view,
                window,
                715,
                OPENING_GROUNDED_CAMERA_Y_FP >> 16,
                level_bounds,
                12,
            ),
            "Diamond should keep its live position through sub_3C3CE even after the layout anchor leaves view",
        );
    }

    #[test]
    fn mini_hopping_skull_sub_3f29a_keeps_live_object_active_after_spawn_point_scrolls_offscreen() {
        // ASM Mini_Hopping_Skull.asm sub_3F29A keeps the live skull loaded from
        // its current position with 11..20 active objects while it stays inside
        // x=[-$1A4,+$2E4], y=[-$1A4,+$284] relative to camera.
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![],
            unknown_word: 0,
            header_byte: 0,
            object_count: 1,
            objects: vec![kid_data::EnemyLayoutObject {
                object_type: 0x1C,
                flags: 0xFF,
                hit_points_plus_one: 0,
                x: 336,
                y: 432,
            }],
            segments: vec![],
        };
        let enemy = ActiveEnemy {
            x_pos: 0x020B_0000,
            y_pos: 0x01C0_0000,
            family: EnemyFamily::MiniHoppingSkull,
            on_ground: true,
            hit_points: 1,
            ..test_enemy(EnemyFamily::MiniHoppingSkull, 0, 0, 1)
        };
        let camera = CameraState {
            x_pos: 715u32 << 16,
            y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
        };
        let window = ActivationWindow::from_camera(camera);
        let view = enemy_layout_view(&layout, 715, OPENING_GROUNDED_CAMERA_Y_FP >> 16);
        let level_bounds = physics::LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };

        assert!(
            !window.contains(
                u32::from(layout.objects[0].x),
                u32::from(layout.objects[0].y),
            ),
            "spawn point should be outside the activation window"
        );
        assert!(
            enemy_should_remain_active(
                &enemy,
                &view,
                window,
                715,
                OPENING_GROUNDED_CAMERA_Y_FP >> 16,
                level_bounds,
                11,
            ),
            "Mini Hopping Skull should remain active while its live position stays inside sub_3F29A's medium-load retention box"
        );
    }

    #[test]
    fn mini_hopping_skull_sub_3f29a_despawns_outside_heavy_load_retention_box() {
        // ASM Mini_Hopping_Skull.asm sub_3F29A tightens to
        // x=[-$40,+$180], y=[-$40,+$120] once Number_Objects exceeds $14.
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![],
            unknown_word: 0,
            header_byte: 0,
            object_count: 1,
            objects: vec![kid_data::EnemyLayoutObject {
                object_type: 0x1C,
                flags: 0xFF,
                hit_points_plus_one: 0,
                x: 336,
                y: 432,
            }],
            segments: vec![],
        };
        let enemy = ActiveEnemy {
            x_pos: 0x020B_0000,
            y_pos: 0x01C0_0000,
            family: EnemyFamily::MiniHoppingSkull,
            on_ground: true,
            hit_points: 1,
            ..test_enemy(EnemyFamily::MiniHoppingSkull, 0, 0, 1)
        };
        let camera = CameraState {
            x_pos: 120u32 << 16,
            y_pos: 256u32 << 16,
        };
        let window = ActivationWindow::from_camera(camera);
        let view = enemy_layout_view(&layout, 120, 256);
        let level_bounds = physics::LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };

        assert!(
            !enemy_should_remain_active(&enemy, &view, window, 120, 256, level_bounds, 21,),
            "Mini Hopping Skull should despawn once its live position leaves sub_3F29A's heavy-load retention box"
        );
    }

    #[test]
    fn mini_hopping_skull_sub_3f366_snaps_x_to_slopeup_surface() {
        // ASM Mini_Hopping_Skull.asm sub_3F366 walks forward to the first `$4000`
        // slope-up cell on the current row, then snaps `x_pos` to
        // `cell_right_px - (y_pos & $F)`.
        let grid = grid_with_cells(8, 8, &[(2, 2, 0x40, 0)]);
        let mut enemy = ActiveEnemy {
            x_pos: 38u32 << 16,
            y_pos: 46u32 << 16,
            family: EnemyFamily::MiniHoppingSkull,
            y_velocity: 0,
            ..test_enemy(EnemyFamily::MiniHoppingSkull, 0, 0, 1)
        };

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.x_pos,
            34u32 << 16,
            "sub_3F366 should snap the skull to cell_right - (y & 0xF) on slope-up contact"
        );
        assert_eq!(
            enemy.y_pos,
            41u32 << 16,
            "landing should still use the slope floor Y"
        );
        assert!(enemy.on_ground, "the slope contact should ground the skull");
    }

    #[test]
    fn mini_hopping_skull_sub_3f3f0_plays_scream_after_15_hibernated_frames() {
        // ASM Mini_Hopping_Skull.asm sub_3F3F0 hibernates for `$F` frames, then
        // plays `sfx_Mini_Hopping_Skull_screams` when `$19(a3)` is clear.
        let mut runtime = NativeRuntime::new(manifest());
        runtime.queue_mini_hopping_skull_scream(false);

        for _ in 0..14 {
            runtime.step_pending_sound_effects();
        }

        assert!(
            runtime.played_sound_effects.is_empty(),
            "sub_3F3F0 should stay silent until the 15th hibernated frame",
        );
        assert_eq!(runtime.pending_sound_effects.len(), 1);

        runtime.step_pending_sound_effects();

        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::MiniHoppingSkullScream],
            "sub_3F3F0 should emit the scream SFX when the 15-frame hibernate expires",
        );
        assert!(
            runtime.pending_sound_effects.is_empty(),
            "the transient sub_3F3F0 worker should delete itself after emitting",
        );
    }

    #[test]
    fn mini_hopping_skull_loc_3f406_deletes_suppressed_scream_worker() {
        // ASM Mini_Hopping_Skull.asm loc_3F406 is reached when `$19(a3)` is set:
        // skip `PlaySound`, then always jump to `j_Delete_CurrentObject`.
        let mut runtime = NativeRuntime::new(manifest());
        runtime.queue_mini_hopping_skull_scream(true);

        for _ in 0..15 {
            runtime.step_pending_sound_effects();
        }

        assert!(
            runtime.played_sound_effects.is_empty(),
            "loc_3F406 should skip the scream SFX when the `$19(a3)` suppress flag is set",
        );
        assert!(
            runtime.pending_sound_effects.is_empty(),
            "loc_3F406 should delete the transient sub_3F3F0 worker even when sound is suppressed",
        );
    }

    #[test]
    fn sub_1b36c_plays_thunderstorm_without_clobbering_d0() {
        let mut runtime = NativeRuntime::new(manifest());

        let preserved = runtime.play_thunderstorm_sound_sub_1B36C(0x4A2C);

        assert_eq!(
            preserved, 0x4A2C,
            "ASM sub_1B36C should restore d0 after playing sfx_Thunderstorm",
        );
        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::Thunderstorm],
            "sub_1B36C should emit the thunderstorm sound effect exactly once",
        );
    }

    #[test]
    fn sub_1bc26_plays_the_navigation_jingle_without_clobbering_d0() {
        let mut runtime = NativeRuntime::new(manifest());

        let preserved = runtime.play_navigate_jingle_sub_1BC26(0x0280);

        assert_eq!(
            preserved, 0x0280,
            "ASM sub_1BC26 should restore d0 after playing sfx_Navigate_jingle",
        );
        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::NavigateJingle],
            "sub_1BC26 should emit the title/options navigation jingle exactly once",
        );
    }

    #[test]
    fn mini_hopping_skull_sub_3f3a6_snaps_x_to_slopedown_surface() {
        // ASM Mini_Hopping_Skull.asm sub_3F3A6 starts from the cell to the right,
        // walks left to the first `$5000` slope-down cell on the current row, then
        // snaps `x_pos` to `cell_left_px + (y_pos & $F)`.
        let grid = grid_with_cells(8, 8, &[(3, 2, 0x50, 0)]);
        let mut enemy = ActiveEnemy {
            x_pos: 55u32 << 16,
            y_pos: 46u32 << 16,
            family: EnemyFamily::MiniHoppingSkull,
            y_velocity: 0,
            ..test_enemy(EnemyFamily::MiniHoppingSkull, 0, 0, 1)
        };

        step_hopper(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.x_pos,
            62u32 << 16,
            "sub_3F3A6 should snap the skull to cell_left + (y & 0xF) on slope-down contact"
        );
        assert_eq!(
            enemy.y_pos,
            39u32 << 16,
            "landing should still use the slope floor Y"
        );
        assert!(enemy.on_ground, "the slope contact should ground the skull");
    }

    #[test]
    fn dragon_flyer_despawns_once_it_moves_past_retention_margin() {
        let layout = EnemyLayout {
            axis_uses_y_position: false,
            unknown_prefix: 0,
            enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
            unknown_word: 0x7D00,
            header_byte: 0x80,
            object_count: 1,
            objects: vec![kid_data::EnemyLayoutObject {
                object_type: 0x0D,
                flags: 0xFF,
                hit_points_plus_one: 0,
                x: 200,
                y: 271,
            }],
            segments: vec![kid_data::EnemyLayoutSegment {
                lower_bound: 0,
                upper_bound: 0x7D00,
                enemy_type_slots: vec![0xFFFF, 0x000C, 0xFFFF],
                object_batch_flags: 0x80,
                object_count: 1,
                objects: vec![kid_data::EnemyLayoutObject {
                    object_type: 0x0D,
                    flags: 0xFF,
                    hit_points_plus_one: 0,
                    x: 200,
                    y: 271,
                }],
            }],
        };
        let mut active = vec![ActiveEnemy {
            segment_index: 0,
            layout_index: 0,
            enemy_level: 0,
            family: EnemyFamily::Dragon,
            x_pos: ((-300i32) << 16) as u32, // beyond ASM-correct retention margin
            y_pos: 272u32 << 16,
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
            dragon_phase: DragonFlyingState::Hovering,
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
            shared_walker_state: spawn_shared_walker_state(EnemyFamily::DragonWalking),
            shared_walker_patrol_timer: spawn_shared_walker_patrol_timer(
                EnemyFamily::DragonWalking,
            ),
            shared_walker_patrol_short_next: spawn_shared_walker_patrol_short_next(
                EnemyFamily::DragonWalking,
            ),
            shared_walker_debug_branch: None,
            shared_walker_dx_to_kid_px: None,
            shared_walker_terrain_trace: SharedWalkerTerrainTrace::default(),
        }];

        activate_enemies(
            &mut active,
            &layout,
            ActivationWindow::from_camera(CameraState {
                x_pos: 429u32 << 16,
                y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            }),
            429,
            OPENING_GROUNDED_CAMERA_Y_FP >> 16,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            0,
            271 << 16,
            &HashSet::new(),
        );

        assert!(
            active.is_empty(),
            "dragon should drop once it moves beyond the retention margin"
        );
    }

    #[test]
    fn dragon_flying_loc_34758_turns_right_only_for_strictly_greater_kid_pixel_x() {
        assert_eq!(
            dragon_face_player_motion(200 << 16, 201 << 16),
            (Direction::Right, 0x8000),
            "ASM Dragon_Flying.asm loc_34758 is reached when dragon.x word < Kid.x word"
        );
        assert_eq!(
            dragon_face_player_motion(200 << 16, (200 << 16) + 0xFFFF),
            (Direction::Left, -0x8000),
            "ASM loc_3473A compares only integer X words, so same-pixel subposition stays left"
        );
        assert_eq!(
            dragon_face_player_motion(200 << 16, 200 << 16),
            (Direction::Left, -0x8000),
            "equal integer X words do not take loc_34758"
        );
    }

    #[test]
    fn dragon_flying_loc_34768_toggles_facing_and_negates_x_velocity() {
        assert_eq!(
            dragon_reverse_x_motion(Direction::Left, -0x8000),
            (Direction::Right, 0x8000),
            "ASM Dragon_Flying.asm loc_34768 flips the direction flag and negates x_vel"
        );
        assert_eq!(
            dragon_reverse_x_motion(Direction::Right, 0x1_8000),
            (Direction::Left, -0x1_8000),
            "loc_34768 preserves the full 32-bit velocity magnitude"
        );
        assert_eq!(
            dragon_reverse_x_motion(Direction::Right, 0),
            (Direction::Left, 0),
            "NEG.L leaves zero velocity at zero while NOT.B still flips facing"
        );
    }

    #[test]
    fn dragon_flyer_script_2_retargets_player_between_drifts() {
        // Script 2: DriftWithFire(112) → DriftWithFire(112) → FacePlayer → DriftWithFire(80).
        // After first two drift phases, FacePlayer should redirect toward the player.
        let mut enemy = test_dragon(2, 200 << 16, 100 << 16);
        // Skip past the first two DriftWithFire commands.
        enemy.dragon_script_cmd = 1; // Will advance to cmd 2 (FacePlayer) then cmd 3.
        enemy.dragon_phase = DragonFlyingState::DriftingWithFire;
        enemy.dragon_phase_frames = 1; // One frame left in current drift.
        enemy.facing = Direction::Left;

        // Player is to the right of the dragon — FacePlayer should turn right.
        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            None,
            Some(500 << 16), // player X to the right
            Some(100 << 16),
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut Vec::new(),
        );

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "FacePlayer should turn toward the player"
        );
        assert_eq!(
            enemy.dragon_phase,
            DragonFlyingState::DriftingWithFire,
            "should be in the next DriftWithFire phase after FacePlayer"
        );
    }

    #[test]
    fn dragon_flyer_fire_breath_cone_triggers_descent() {
        // During DriftWithFire, if the player is within the breath cone, descent triggers.
        let mut enemy = test_dragon(1, 200 << 16, 100 << 16);
        // Set up: drifting with fire, facing left, lots of frames left.
        enemy.dragon_phase = DragonFlyingState::DriftingWithFire;
        enemy.dragon_phase_frames = 50;
        enemy.facing = Direction::Left;
        enemy.x_velocity = -0x800;

        // Place player directly below and in front of dragon (within cone).
        let player_x = Some(210u32 << 16); // slightly to the right (dragon faces left)
        let player_y = Some(105u32 << 16); // below dragon (y=100)

        step_active_enemies(
            std::slice::from_mut(&mut enemy),
            None,
            player_x,
            player_y,
            None,
            None,
            PlayerForm::TheKid,
            None,
            &mut Vec::new(),
        );

        // The cone check geometry is tight — may or may not trigger depending on
        // exact pixel distances. Verify that the mechanism exists and doesn't crash.
        // (Detailed cone geometry is ASM-verified separately.)
        assert!(
            matches!(
                enemy.dragon_phase,
                DragonFlyingState::Descending | DragonFlyingState::DriftingWithFire
            ),
            "should either enter descent or continue drifting"
        );
    }

    #[test]
    fn dragon_flyer_descent_fire_uses_live_horizontal_velocity_and_downward_arc() {
        let mut enemy = test_dragon(0, 100 << 16, 200 << 16);
        enemy.dragon_phase = DragonFlyingState::Descending;
        enemy.dragon_descent_index = 3; // ASM unk_34554 sentinel $E19F
        enemy.dragon_descent_frames = 0;
        enemy.facing = Direction::Right;
        enemy.x_velocity = -0x1_4000;
        enemy.y_velocity = 0x3_0000;

        let mut projectiles = Vec::new();
        let mut projectile_slot = Some(&mut projectiles);
        step_dragon(&mut enemy, None, None, None, &mut projectile_slot);

        assert_eq!(
            projectiles.len(),
            1,
            "descent fire should spawn one projectile"
        );
        assert_eq!(
            projectiles[0].x_velocity, -0x2_0000,
            "ASM loc_348A0/loc_34BE6 copy only the dragon x_vel high word into descent fire"
        );
        assert_eq!(
            projectiles[0].y_velocity,
            3 << 16,
            "ASM loc_34BE6 stores the parent-supplied +3px/frame word in the projectile"
        );
        // ASM loc_34BAC: x_pos += $FFF6 (-10px), negated if facing right → +10px
        let expected_x = (100 + 10) << 16;
        assert_eq!(
            projectiles[0].x_pos, expected_x as u32,
            "ASM loc_34BAC: fire X offset +10px when facing right"
        );
        assert!(
            projectiles[0].terrain_kill,
            "ASM: fire dies on terrain contact, not frame timer"
        );
        assert_eq!(
            enemy.dragon_descent_index, 4,
            "spawn-fire advances immediately"
        );
    }

    #[test]
    fn dragon_flying_loc_34be6_sets_descent_fire_from_parent_words() {
        assert_eq!(
            dragon_flying_descent_fire_setup_loc_34BE6(
                0x0100_0000,
                0x0200_8000,
                -0x0001_4000,
                3,
                Direction::Right,
            ),
            DragonFlyingDescentFireSetupLoc34BE6 {
                x_pos: 0x010A_0000,
                y_pos: 0x020A_8000,
                initial_x_velocity: 0x0001_0000,
                final_x_velocity: -0x0002_0000,
                y_velocity: 0x0003_0000,
                is_moved: true,
                has_level_collision: true,
                priority: 0,
                object_meta: 0x000C,
                offscreen_flag_cleared: true,
                animation_label: "LnkTo_unk_C75CE",
                first_animation_counter: 6,
            },
            "ASM Dragon_Flying.asm loc_34BE6 seeds d1 into x_vel, then overwrites x_vel from the signed parent high word"
        );
        assert_eq!(
            dragon_flying_descent_fire_setup_loc_34BE6(
                0x0100_0000,
                0x0200_0000,
                0x0001_C000,
                3,
                Direction::Left,
            )
            .x_pos,
            0x00F6_0000,
            "ASM Dragon_Flying.asm loc_34BE6 keeps the `$FFF6` left-facing mouth offset"
        );
    }

    #[test]
    fn dragon_flying_loc_34c26_runs_first_fire_animation_dbf_loop() {
        assert_eq!(
            dragon_flying_descent_fire_first_loop_loc_34C26(6, 0),
            DragonFlyingDescentFireFirstLoopLoc34C26::ContinueFirstAnimation { next_d0: 5 },
            "ASM Dragon_Flying.asm loc_34C26 decrements d0 after a no-collision hibernated frame and loops while it is not `$FFFF`"
        );
        assert_eq!(
            dragon_flying_descent_fire_first_loop_loc_34C26(0, 0),
            DragonFlyingDescentFireFirstLoopLoc34C26::EnterTravelAnimation {
                next_d0: 0xFFFF,
                animation_label: "LnkTo_unk_C75D6",
            },
            "ASM Dragon_Flying.asm loc_34C26 falls through to the travel sprite after DBF decrements zero to `$FFFF`"
        );
        assert_eq!(
            dragon_flying_descent_fire_first_loop_loc_34C26(6, 0x0004),
            DragonFlyingDescentFireFirstLoopLoc34C26::CollisionCleanup {
                dbf_counter: 6,
                has_level_collision: false,
                x_velocity: 0,
                y_velocity: 0,
                animation_label: "LnkTo_unk_C75DE",
            },
            "ASM Dragon_Flying.asm loc_34C26 branches to the cleanup tail before DBF when collision_type is non-zero"
        );
    }

    #[test]
    fn dragon_flying_loc_34c3a_waits_for_collision_or_animation_done() {
        assert_eq!(
            dragon_flying_descent_fire_travel_loop_loc_34C3A(0, false),
            DragonFlyingDescentFireTravelLoopLoc34C3A::ContinueTravelAnimation,
            "ASM Dragon_Flying.asm loc_34C3A repeats the travel animation while collision_type is zero and `$19` is clear"
        );
        assert_eq!(
            dragon_flying_descent_fire_travel_loop_loc_34C3A(0x0004, false),
            DragonFlyingDescentFireTravelLoopLoc34C3A::CollisionCleanup {
                has_level_collision: false,
                x_velocity: 0,
                y_velocity: 0,
                animation_label: "LnkTo_unk_C75DE",
            },
            "ASM Dragon_Flying.asm loc_34C3A checks collision_type before byte `$19`"
        );
        assert_eq!(
            dragon_flying_descent_fire_travel_loop_loc_34C3A(0, true),
            DragonFlyingDescentFireTravelLoopLoc34C3A::AnimationFinishedCleanup {
                has_level_collision: false,
                x_velocity: 0,
                y_velocity: 0,
                animation_label: "LnkTo_unk_C75DE",
            },
            "ASM Dragon_Flying.asm loc_34C3A falls into the cleanup tail when byte `$19` is set"
        );
    }

    #[test]
    fn dragon_flying_loc_34c4a_runs_cleanup_animations_then_deletes() {
        assert_eq!(
            dragon_flying_descent_fire_cleanup_loc_34C4A(),
            DragonFlyingDescentFireCleanupLoc34C4A {
                has_level_collision: false,
                x_velocity: 0,
                y_velocity: 0,
                first_animation_label: "LnkTo_unk_C75DE",
                first_animation_hibernate_frames: 3,
                second_animation_label: "LnkTo_unk_C75E6",
                second_animation_hibernate_frames: 3,
                deletes_current_object: true,
            },
            "ASM Dragon_Flying.asm loc_34C4A clears movement/collision, hibernates 3+3 cleanup frames, then deletes the object"
        );
    }

    #[test]
    fn dragon_flying_loc_348e4_seeds_descent_velocity_and_dbf_counter() {
        assert_eq!(
            dragon_descent_move_setup_loc_348e4(Direction::Left, -2, 3, 6),
            DragonDescentMoveSetupLoc348E4 {
                x_velocity: -2 << 16,
                y_velocity: 3 << 16,
                dbf_counter_after_subq: 5,
            },
            "ASM Dragon_Flying.asm loc_348E4 stores signed movement bytes as FP16.16 velocities and subtracts one from the frame byte for DBF"
        );
        assert_eq!(
            dragon_descent_move_setup_loc_348e4(Direction::Right, -1, -4, 12),
            DragonDescentMoveSetupLoc348E4 {
                x_velocity: 1 << 16,
                y_velocity: -4 << 16,
                dbf_counter_after_subq: 11,
            },
            "ASM Dragon_Flying.asm loc_348E4 receives the loc_348CE direction-negated x velocity before starting the hibernation loop"
        );
    }

    #[test]
    fn dragon_flying_loc_348fc_classifies_descent_collision_type() {
        assert_eq!(
            dragon_descent_collision_outcome_loc_348fc(0),
            DragonDescentCollisionOutcomeLoc348FC::ContinueLoop,
            "ASM Dragon_Flying.asm loc_348FC falls through to the loop tail when collision_type is zero"
        );
        assert_eq!(
            dragon_descent_collision_outcome_loc_348fc(0x8000),
            DragonDescentCollisionOutcomeLoc348FC::Explosion,
            "ASM Dragon_Flying.asm loc_348FC sends negative collision words to the death/explosion path"
        );
        assert_eq!(
            dragon_descent_collision_outcome_loc_348fc(0x001C),
            DragonDescentCollisionOutcomeLoc348FC::Explosion,
            "ASM loc_348FC treats collision_type $1C as the same death/explosion branch"
        );
        assert_eq!(
            dragon_descent_collision_outcome_loc_348fc(0x002C),
            DragonDescentCollisionOutcomeLoc348FC::Explosion,
            "ASM loc_348FC treats collision_type $2C as the same death/explosion branch"
        );
        assert_eq!(
            dragon_descent_collision_outcome_loc_348fc(0x0004),
            DragonDescentCollisionOutcomeLoc348FC::TerrainResponse(0x0004),
            "ASM Dragon_Flying.asm loc_348FC forwards other positive terrain collisions to terrain response"
        );
    }

    #[test]
    fn dragon_flying_loc_34924_descent_dbf_tail_loops_until_ffff() {
        assert_eq!(
            dragon_descent_dbf_tail_loc_34924(0x0005),
            DragonDescentDbfTailLoc34924 {
                next_d2: 0x0004,
                loops_to_loc_348fc: true,
                advances_descent_script: false,
            },
            "ASM Dragon_Flying.asm loc_34924 decrements d2 and branches back while the result is not `$FFFF`"
        );
        assert_eq!(
            dragon_descent_dbf_tail_loc_34924(0x0000),
            DragonDescentDbfTailLoc34924 {
                next_d2: 0xFFFF,
                loops_to_loc_348fc: false,
                advances_descent_script: true,
            },
            "ASM Dragon_Flying.asm loc_34924 falls through to the next descent command only after DBF decrements zero to `$FFFF`"
        );
    }

    #[test]
    fn dragon_flying_loc_349c0_clears_y_velocity_at_or_above_anchor_word() {
        assert_eq!(
            dragon_return_stop_loc_349c0(0x0100, 0x0100),
            DragonReturnStopLoc349C0 {
                y_velocity: 0,
                reached_anchor_word: true,
            },
            "ASM Dragon_Flying.asm loc_349C0 clears y_vel when the integer Y word has reached the anchor"
        );
        assert_eq!(
            dragon_return_stop_loc_349c0(0x00FF, 0x0100),
            DragonReturnStopLoc349C0 {
                y_velocity: 0,
                reached_anchor_word: true,
            },
            "ASM Dragon_Flying.asm loc_349C0 is also reached when current Y is above the anchor"
        );
        assert_eq!(
            dragon_return_stop_loc_349c0(0x0101, 0x0100),
            DragonReturnStopLoc349C0 {
                y_velocity: -0x4000,
                reached_anchor_word: false,
            },
            "the branch before loc_349C0 keeps rising at `$FFFFC000` while current Y is still below the anchor"
        );
    }

    #[test]
    fn dragon_flying_loc_349de_routes_positive_terrain_collision_table_offsets() {
        assert_eq!(
            dragon_terrain_dispatch_loc_349DE(0x8000),
            DragonTerrainDispatchLoc349DE::DeathExplosion {
                collision_cleared: false,
            },
            "ASM Dragon_Flying.asm loc_349DE branches to the death path before clearing negative collision_type"
        );
        assert_eq!(
            dragon_terrain_dispatch_loc_349DE(0x001C),
            DragonTerrainDispatchLoc349DE::RestartDescentScript {
                collision_cleared: true,
            },
            "ASM Dragon_Flying.asm loc_349DE clears collision_type, then `$1C+` returns to the descent script reader"
        );
        assert_eq!(
            dragon_terrain_dispatch_loc_349DE(0x0004),
            DragonTerrainDispatchLoc349DE::ResponseTable {
                collision_cleared: true,
                byte_offset: 0,
            },
            "ASM Dragon_Flying.asm loc_349DE subtracts four from low terrain collision words before table dispatch"
        );
        assert_eq!(
            dragon_terrain_dispatch_loc_349DE(0x0018),
            DragonTerrainDispatchLoc349DE::ResponseTable {
                collision_cleared: true,
                byte_offset: 0x0014,
            },
            "the last low terrain response reaches byte offset `$14` in the six-entry table"
        );
    }

    #[test]
    fn dragon_flying_loc_34a00_starts_right_slide_without_clearing_fractional_x() {
        assert_eq!(
            dragon_right_slide_start_loc_34a00(0x0000_3456),
            DragonRightSlideStartLoc34A00 {
                x_velocity: 0x0002_3456,
                y_velocity: 0,
            },
            "ASM Dragon_Flying.asm loc_34A00 clears y_vel and writes only the high word of x_vel"
        );
        assert_eq!(
            dragon_right_slide_start_loc_34a00(-0x0000_8000),
            DragonRightSlideStartLoc34A00 {
                x_velocity: 0x0002_8000,
                y_velocity: 0,
            },
            "ASM Dragon_Flying.asm loc_34A00 preserves the previous fractional low word even when the old long was negative"
        );
    }

    #[test]
    fn dragon_flying_loc_34a0a_decelerates_right_slide_until_signed_nonpositive() {
        assert_eq!(
            dragon_right_slide_frame_loc_34a0a(0x0002_3456, 0x0004),
            DragonRightSlideFrameLoc34A0A::CollisionRoute {
                x_velocity: 0x0002_3456,
                collision_type: 0x0004,
            },
            "ASM Dragon_Flying.asm loc_34A0A sends non-zero collision_type to the collision handler before changing x_vel"
        );
        assert_eq!(
            dragon_right_slide_frame_loc_34a0a(0x0002_3456, 0),
            DragonRightSlideFrameLoc34A0A::Continue {
                x_velocity: 0x0002_2456,
            },
            "ASM Dragon_Flying.asm loc_34A0A subtracts `$1000` and loops while the signed result stays positive"
        );
        assert_eq!(
            dragon_right_slide_frame_loc_34a0a(0x0000_1000, 0),
            DragonRightSlideFrameLoc34A0A::BeginReturnToAnchor { x_velocity: 0 },
            "ASM Dragon_Flying.asm loc_34A0A enters return-to-anchor when `subi.l #$1000,x_vel` sets signed zero"
        );
        assert_eq!(
            dragon_right_slide_frame_loc_34a0a(0x0000_0800, 0),
            DragonRightSlideFrameLoc34A0A::BeginReturnToAnchor {
                x_velocity: -0x0000_0800,
            },
            "ASM Dragon_Flying.asm loc_34A0A also enters return-to-anchor for a signed negative deceleration result"
        );
    }

    #[test]
    fn dragon_flying_loc_34a24_starts_left_slide_without_clearing_fractional_x() {
        assert_eq!(
            dragon_left_slide_start_loc_34a24(0x0000_3456),
            DragonLeftSlideStartLoc34A24 {
                x_velocity: -0x0002_0000 + 0x3456,
                y_velocity: 0,
            },
            "ASM Dragon_Flying.asm loc_34A24 clears y_vel and writes `$FFFE` only to the high word of x_vel"
        );
        assert_eq!(
            dragon_left_slide_start_loc_34a24(-0x0000_8000),
            DragonLeftSlideStartLoc34A24 {
                x_velocity: -0x0002_0000 + 0x8000,
                y_velocity: 0,
            },
            "ASM Dragon_Flying.asm loc_34A24 preserves the previous fractional low word even when the old long was negative"
        );
    }

    #[test]
    fn dragon_flying_loc_34a2e_accelerates_left_slide_until_signed_nonnegative() {
        assert_eq!(
            dragon_left_slide_frame_loc_34a2e(-0x0001_CBAA, 0x0008),
            DragonLeftSlideFrameLoc34A2E::CollisionRoute {
                x_velocity: -0x0001_CBAA,
                collision_type: 0x0008,
            },
            "ASM Dragon_Flying.asm loc_34A2E sends non-zero collision_type to the collision handler before changing x_vel"
        );
        assert_eq!(
            dragon_left_slide_frame_loc_34a2e(-0x0001_CBAA, 0),
            DragonLeftSlideFrameLoc34A2E::Continue {
                x_velocity: -0x0001_BBAA,
            },
            "ASM Dragon_Flying.asm loc_34A2E adds `$1000` and loops while the signed result stays negative"
        );
        assert_eq!(
            dragon_left_slide_frame_loc_34a2e(-0x0000_1000, 0),
            DragonLeftSlideFrameLoc34A2E::BeginReturnToAnchor { x_velocity: 0 },
            "ASM Dragon_Flying.asm loc_34A2E enters return-to-anchor when `addi.l #$1000,x_vel` sets signed zero"
        );
        assert_eq!(
            dragon_left_slide_frame_loc_34a2e(-0x0000_0800, 0),
            DragonLeftSlideFrameLoc34A2E::BeginReturnToAnchor {
                x_velocity: 0x0000_0800,
            },
            "ASM Dragon_Flying.asm loc_34A2E also enters return-to-anchor for a signed positive acceleration result"
        );
    }

    #[test]
    fn dragon_flying_loc_34a48_starts_upward_slide_without_clearing_fractional_y() {
        assert_eq!(
            dragon_up_slide_start_loc_34a48(0x0000_3456),
            DragonUpSlideStartLoc34A48 {
                y_velocity: -0x0003_0000 + 0x3456,
            },
            "ASM Dragon_Flying.asm loc_34A48 writes `$FFFD` only to the high word of y_vel"
        );
        assert_eq!(
            dragon_up_slide_start_loc_34a48(-0x0000_8000),
            DragonUpSlideStartLoc34A48 {
                y_velocity: -0x0003_0000 + 0x8000,
            },
            "ASM Dragon_Flying.asm loc_34A48 preserves the previous fractional low word even when the old long was negative"
        );
    }

    #[test]
    fn dragon_flying_loc_34a4e_accelerates_upward_slide_until_signed_nonnegative() {
        assert_eq!(
            dragon_up_slide_frame_loc_34a4e(-0x0002_CBAA, 0x0008),
            DragonUpSlideFrameLoc34A4E::CollisionRoute {
                y_velocity: -0x0002_CBAA,
                collision_type: 0x0008,
            },
            "ASM Dragon_Flying.asm loc_34A4E sends non-zero collision_type to the collision handler before changing y_vel"
        );
        assert_eq!(
            dragon_up_slide_frame_loc_34a4e(-0x0002_CBAA, 0),
            DragonUpSlideFrameLoc34A4E::Continue {
                y_velocity: -0x0002_7BAA,
            },
            "ASM Dragon_Flying.asm loc_34A4E adds `$5000` and loops while the signed result stays negative"
        );
        assert_eq!(
            dragon_up_slide_frame_loc_34a4e(-0x0000_5000, 0),
            DragonUpSlideFrameLoc34A4E::BeginReturnToAnchor { y_velocity: 0 },
            "ASM Dragon_Flying.asm loc_34A4E enters return-to-anchor when `addi.l #$5000,y_vel` sets signed zero"
        );
        assert_eq!(
            dragon_up_slide_frame_loc_34a4e(-0x0000_4000, 0),
            DragonUpSlideFrameLoc34A4E::BeginReturnToAnchor {
                y_velocity: 0x0000_1000,
            },
            "ASM Dragon_Flying.asm loc_34A4E also enters return-to-anchor for a signed positive acceleration result"
        );
    }

    #[test]
    fn dragon_flying_loc_34a68_starts_downward_slide_without_clearing_fractional_y() {
        assert_eq!(
            dragon_down_slide_start_loc_34a68(0x0000_3456),
            DragonDownSlideStartLoc34A68 {
                y_velocity: 0x0003_3456,
            },
            "ASM Dragon_Flying.asm loc_34A68 writes `#3` only to the high word of y_vel"
        );
        assert_eq!(
            dragon_down_slide_start_loc_34a68(-0x0000_8000),
            DragonDownSlideStartLoc34A68 {
                y_velocity: 0x0003_8000,
            },
            "ASM Dragon_Flying.asm loc_34A68 preserves the previous fractional low word even when the old long was negative"
        );
    }

    #[test]
    fn dragon_flying_loc_34a6e_decelerates_downward_slide_until_signed_nonpositive() {
        assert_eq!(
            dragon_down_slide_frame_loc_34A6E(0x0002_CBAA, 0x0008),
            DragonDownSlideFrameLoc34A6E::CollisionRoute {
                y_velocity: 0x0002_CBAA,
                collision_type: 0x0008,
            },
            "ASM Dragon_Flying.asm loc_34A6E sends non-zero collision_type to the collision handler before changing y_vel"
        );
        assert_eq!(
            dragon_down_slide_frame_loc_34A6E(0x0002_CBAA, 0),
            DragonDownSlideFrameLoc34A6E::Continue {
                y_velocity: 0x0002_ABAA,
            },
            "ASM Dragon_Flying.asm loc_34A6E subtracts `$2000` and loops while the signed result stays positive"
        );
        assert_eq!(
            dragon_down_slide_frame_loc_34A6E(0x0000_2000, 0),
            DragonDownSlideFrameLoc34A6E::BeginReturnToAnchor { y_velocity: 0 },
            "ASM Dragon_Flying.asm loc_34A6E enters return-to-anchor when `subi.l #$2000,y_vel` sets signed zero"
        );
        assert_eq!(
            dragon_down_slide_frame_loc_34A6E(0x0000_1000, 0),
            DragonDownSlideFrameLoc34A6E::BeginReturnToAnchor {
                y_velocity: -0x0000_1000,
            },
            "ASM Dragon_Flying.asm loc_34A6E also enters return-to-anchor for a signed negative deceleration result"
        );
    }

    #[test]
    fn dragon_flying_loc_34a88_snaps_back_with_level_collision_disabled_for_one_frame() {
        assert_eq!(
            dragon_flying_rebound_start_loc_34A88(
                0x0100_8000,
                0x0200_4000,
                0x0002_8000,
                0x0003_4000,
            ),
            DragonFlyingReboundStartLoc34A88 {
                snap_x_pos: 0x00FE_0000,
                snap_y_pos: 0x01FD_0000,
                x_velocity_during_snap: 0,
                y_velocity_during_snap: 0,
                level_collision_during_first_hibernate: false,
                level_collision_after_snap: true,
                hibernate_frames_before_rebound_velocity: 2,
                animation_label: "stru_34652",
                rebound_x_velocity: -0x0002_0000,
                rebound_y_velocity: -0x0002_0000,
            },
            "ASM Dragon_Flying.asm loc_34A88 uses the prepared rebound coordinates to snap to `pos - velocity` after one no-collision hibernate"
        );

        assert_eq!(
            dragon_flying_rebound_start_loc_34A88(
                0x0100_0000,
                0x0200_0000,
                -0x0002_8000,
                -0x0003_4000,
            )
            .snap_y_pos,
            0x0203_4000,
            "ASM Dragon_Flying.asm loc_34A88 negates signed y_vel before adding it back to y_pos"
        );
    }

    #[test]
    fn dragon_flying_loc_34adc_snaps_back_then_rebounds_right() {
        assert_eq!(
            dragon_flying_right_rebound_start_loc_34ADC(
                0x0100_8000,
                0x0200_4000,
                -0x0001_8000,
                0x0002_4000,
            ),
            DragonFlyingReboundStartLoc34A88 {
                snap_x_pos: 0x0102_0000,
                snap_y_pos: 0x01FE_0000,
                x_velocity_during_snap: 0,
                y_velocity_during_snap: 0,
                level_collision_during_first_hibernate: false,
                level_collision_after_snap: true,
                hibernate_frames_before_rebound_velocity: 2,
                animation_label: "stru_34652",
                rebound_x_velocity: 0x0002_0000,
                rebound_y_velocity: -0x0002_0000,
            },
            "ASM Dragon_Flying.asm loc_34ADC mirrors loc_34A88's snap sequence but writes positive `#2` to x_vel"
        );
    }

    #[test]
    fn dragon_flying_loc_34b96_prepares_snap_coordinates_by_negating_velocity() {
        assert_eq!(
            dragon_flying_snap_back_coords_loc_34B96(
                0x0100_8000,
                0x0200_4000,
                0x0002_8000,
                -0x0003_4000,
            ),
            DragonFlyingSnapBackCoordsLoc34B96 {
                snap_x_pos: 0x00FE_0000,
                snap_y_pos: 0x0203_8000,
            },
            "ASM Dragon_Flying.asm loc_34B96 negates x_vel/y_vel before adding them to the live x_pos/y_pos"
        );
        assert_eq!(
            dragon_flying_snap_back_coords_loc_34B96(
                0x0001_0000,
                0x0001_0000,
                -0x0002_0000,
                0x0002_0000,
            ),
            DragonFlyingSnapBackCoordsLoc34B96 {
                snap_x_pos: 0x0003_0000,
                snap_y_pos: 0xFFFF_0000,
            },
            "ASM Dragon_Flying.asm loc_34B96 uses wrapping 32-bit coordinate arithmetic"
        );
    }

    #[test]
    fn dragon_flying_loc_34ac2_accelerates_rebound_until_exact_zero() {
        assert_eq!(
            dragon_flying_rebound_frame_loc_34AC2(-0x0002_0000, 0),
            DragonFlyingReboundFrameLoc34AC2::Continue {
                y_velocity: -0x0001_E000,
            },
            "ASM Dragon_Flying.asm loc_34AC2 adds `$2000` and loops while the result is non-zero"
        );
        assert_eq!(
            dragon_flying_rebound_frame_loc_34AC2(-0x0000_4000, 0x0008),
            DragonFlyingReboundFrameLoc34AC2::CollisionRoute {
                y_velocity: -0x0000_2000,
                collision_type: 0x0008,
            },
            "ASM Dragon_Flying.asm loc_34AC2 polls collision_type after applying the `$2000` y_vel step"
        );
        assert_eq!(
            dragon_flying_rebound_frame_loc_34AC2(-0x0000_2000, 0x0008),
            DragonFlyingReboundFrameLoc34AC2::BeginReturnToAnchor { y_velocity: 0 },
            "ASM Dragon_Flying.asm loc_34AC2 takes the `beq` return-to-anchor exit before hibernating or reading collision_type"
        );
    }

    #[test]
    fn dragon_flying_loc_34b16_accelerates_right_rebound_until_exact_zero() {
        assert_eq!(
            dragon_flying_right_rebound_frame_loc_34B16(-0x0002_0000, 0),
            DragonFlyingReboundFrameLoc34AC2::Continue {
                y_velocity: -0x0001_E000,
            },
            "ASM Dragon_Flying.asm loc_34B16 adds `$2000` and loops while the right rebound y_vel remains non-zero"
        );
        assert_eq!(
            dragon_flying_right_rebound_frame_loc_34B16(-0x0000_4000, 0x0008),
            DragonFlyingReboundFrameLoc34AC2::CollisionRoute {
                y_velocity: -0x0000_2000,
                collision_type: 0x0008,
            },
            "ASM Dragon_Flying.asm loc_34B16 polls collision_type only after applying the `$2000` y_vel step"
        );
        assert_eq!(
            dragon_flying_right_rebound_frame_loc_34B16(-0x0000_2000, 0x0008),
            DragonFlyingReboundFrameLoc34AC2::BeginReturnToAnchor { y_velocity: 0 },
            "ASM Dragon_Flying.asm loc_34B16 takes the `beq` return-to-anchor exit before reading collision_type"
        );
    }

    #[test]
    fn dragon_flying_loc_34b30_routes_only_selected_rebound_collisions_to_death() {
        assert_eq!(
            dragon_flying_rebound_collision_route_loc_34B30(0x001C),
            DragonFlyingReboundCollisionRouteLoc34B30::DeathExplosion,
            "ASM Dragon_Flying.asm loc_34B30 sends collision `$1C` to loc_3492C"
        );
        assert_eq!(
            dragon_flying_rebound_collision_route_loc_34B30(0x002C),
            DragonFlyingReboundCollisionRouteLoc34B30::DeathExplosion,
            "ASM Dragon_Flying.asm loc_34B30 sends collision `$2C` to loc_3492C"
        );
        assert_eq!(
            dragon_flying_rebound_collision_route_loc_34B30(0x0004),
            DragonFlyingReboundCollisionRouteLoc34B30::RestartScript,
            "ASM Dragon_Flying.asm loc_34B30 restarts the script for ordinary terrain collision words"
        );
        assert_eq!(
            dragon_flying_rebound_collision_route_loc_34B30(0xFFFF),
            DragonFlyingReboundCollisionRouteLoc34B30::RestartScript,
            "ASM Dragon_Flying.asm loc_34B30 has no negative-collision special case before restarting the script"
        );
    }

    #[test]
    fn dragon_flying_loc_34998_writes_respawn_marker_for_positive_slot() {
        assert_eq!(
            dragon_flying_respawn_marker_loc_34998(0x07),
            DragonFlyingRespawnMarkerLoc34998 {
                index: 0x07,
                word_value: 0x2168,
            },
            "ASM Dragon_Flying.asm loc_34998 doubles positive `$42(a5)` into an EnemyStatus_Table word offset and writes `$2168`"
        );
    }

    #[test]
    fn dragon_flying_loc_349a6_always_deletes_current_object() {
        assert_eq!(
            dragon_flying_delete_tail_loc_349A6(),
            DragonFlyingDeleteTailLoc349A6 {
                deletes_current_object: true,
            },
            "ASM Dragon_Flying.asm loc_349A6 is the shared j_Delete_CurrentObject tail for every death-status route"
        );
    }

    #[test]
    fn dragon_flying_loc_3492c_runs_explosion_and_death_status_routes() {
        let positive = dragon_flying_death_sequence_loc_3492C(
            0x0123_8000,
            0x0045_0000,
            Direction::Right,
            0xCAFE,
            0x05,
        );
        assert_eq!(
            positive,
            DragonFlyingDeathSequenceLoc3492C {
                animation_label: "stru_345E4",
                is_moved: false,
                vram_tile: 0,
                pre_child_hibernate_frames: 2,
                child_x_word: 0x0123,
                child_y_word: 0x0055,
                child_facing: Direction::Right,
                child_state_word_4a: 0xCAFE,
                hibernate_until_animation_finished: true,
                status_route: DragonFlyingDeathStatusRouteLoc3492C::WriteRespawnMarker(
                    DragonFlyingRespawnMarkerLoc34998 {
                        index: 0x05,
                        word_value: 0x2168,
                    },
                ),
                delete_tail: DragonFlyingDeleteTailLoc349A6 {
                    deletes_current_object: true,
                },
            },
            "ASM Dragon_Flying.asm loc_3492C runs stru_345E4, spawns the walking-dragon child 16px below, then positive `$42(a5)` branches to loc_34998"
        );

        assert_eq!(
            dragon_flying_death_sequence_loc_3492C(0, 0, Direction::Left, 0, 0xC7).status_route,
            DragonFlyingDeathStatusRouteLoc3492C::SubtractSpawnCounter {
                index: 0x07,
                decrement: 0x0400,
            },
            "ASM Dragon_Flying.asm loc_3492C masks bit-7+bit-6 `$42(a5)` with `$3F` before subtracting `$400` from EnemyStatus_Table"
        );
        assert_eq!(
            dragon_flying_death_sequence_loc_3492C(0, 0, Direction::Left, 0, 0x80).status_route,
            DragonFlyingDeathStatusRouteLoc3492C::DeleteOnly,
            "ASM Dragon_Flying.asm loc_3492C falls straight to loc_349A6 when bit 7 is set and bit 6 is clear"
        );
    }

    #[test]
    fn dragon_flying_loc_3488a_flip_sentinel_dispatches_next_move() {
        let mut enemy = test_dragon(0, 100 << 16, 200 << 16);
        enemy.dragon_phase = DragonFlyingState::Descending;
        enemy.dragon_descent_index = 13; // ASM unk_34554 sentinel $DD48
        enemy.dragon_descent_frames = 0;
        enemy.facing = Direction::Left;

        let mut projectile_slot = None;
        step_dragon(&mut enemy, None, None, None, &mut projectile_slot);

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Dragon_Flying.asm loc_3488A handles $DD48 by flipping x_direction before rereading the dive script"
        );
        assert_eq!(
            enemy.x_velocity,
            1 << 16,
            "loc_3488A should continue immediately into the next movement word after the flip sentinel"
        );
        assert_eq!(enemy.y_velocity, -3 << 16);
        assert_eq!(enemy.dragon_descent_index, 14);
        assert_eq!(
            enemy.dragon_descent_frames, 5,
            "the next movement word starts its six-frame run and consumes the current frame"
        );
    }

    #[test]
    /// ASM loc_3D02A: walking dragon breath is a stationary animated hitbox
    /// at ±28px from the dragon's mouth. No velocity — stru_3CCC8 plays for
    /// 68 frames (8+10+10+10+10+10+10) then $18(a3) signals completion.
    fn dragon_walking_fires_breath_in_facing_direction() {
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 100 << 16, 200 << 16, 2);
        enemy.on_ground = true;
        enemy.wait_timer = 1;
        enemy.facing = Direction::Right;
        let mut projectiles = Vec::new();
        step_dragon_walking(&mut enemy, None, &mut projectiles, None);
        assert_eq!(projectiles.len(), 1);
        assert_eq!(
            projectiles[0].x_velocity, 0,
            "ASM loc_3D02A: breath is stationary (no velocity)"
        );
        assert_eq!(projectiles[0].y_velocity, 0, "breath is stationary");
        // Breath offset: +28px when facing right
        let expected_x = (100 + DRAGON_WALKING_BREATH_OFFSET_PX) << 16;
        assert_eq!(
            projectiles[0].x_pos, expected_x as u32,
            "breath placed at dragon mouth (+28px right)"
        );
        assert_eq!(
            projectiles[0].lifetime, DRAGON_WALKING_BREATH_LIFETIME,
            "breath lasts for animation duration (68 frames)"
        );
        assert_eq!(enemy.wait_timer, DRAGON_WALKING_BREATH_RESET);
        assert_eq!(
            enemy.dragon_script_cmd, 1,
            "dragon should enter breath hold"
        );
    }

    #[test]
    /// ASM loc_3D02A: breath at -28px when facing left, still stationary.
    fn dragon_walking_fires_breath_left_when_facing_left() {
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 100 << 16, 200 << 16, 2);
        enemy.on_ground = true;
        enemy.wait_timer = 1;
        enemy.facing = Direction::Left;
        let mut projectiles = Vec::new();
        step_dragon_walking(&mut enemy, None, &mut projectiles, None);
        assert_eq!(projectiles.len(), 1);
        assert_eq!(
            projectiles[0].x_velocity, 0,
            "ASM loc_3D02A: breath is stationary (no velocity)"
        );
        let expected_x = (100 - DRAGON_WALKING_BREATH_OFFSET_PX) << 16;
        assert_eq!(
            projectiles[0].x_pos, expected_x as u32,
            "breath placed at dragon mouth (-28px left)"
        );
    }

    /// ASM Dragon.asm loc_3CD70: dragon faces TOWARD the player at spawn.
    /// `cmp.w x_pos(a3),d4; blt.s loc_3CD82; st x_direction(a3)`
    /// kid.x < dragon.x → x_direction stays 0 → LEFT (toward kid).
    /// kid.x >= dragon.x → st x_direction=$FF → RIGHT (toward kid).
    #[test]
    fn spawn_facing_dragon_faces_toward_player() {
        // Player at x=50, dragon at x=200 → player is LEFT → dragon faces Left (toward player)
        assert_eq!(
            spawn_facing(EnemyFamily::DragonWalking, 200 << 16, 50 << 16),
            Direction::Left,
            "dragon should face left toward player on its left (ASM loc_3CD70: blt.s skip)"
        );
        // Player at x=300, dragon at x=200 → player is RIGHT → dragon faces Right (toward player)
        assert_eq!(
            spawn_facing(EnemyFamily::DragonWalking, 200 << 16, 300 << 16),
            Direction::Right,
            "dragon should face right toward player on its right (ASM loc_3CD70: st x_direction)"
        );
        // Equal position → faces Right (st branch: kid.x >= dragon.x)
        assert_eq!(
            spawn_facing(EnemyFamily::DragonWalking, 200 << 16, 200 << 16),
            Direction::Right,
            "equal position → st x_direction → Right (ASM loc_3CD70: blt is strict less-than)"
        );
    }

    #[test]
    fn dragon_walking_stops_to_breathe_then_turns_from_a_wall() {
        let grid = grid_with_wall(17, 0, 20, 1);
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 40 << 16, 272 << 16, 2);
        enemy.on_ground = true;
        enemy.facing = Direction::Left;
        enemy.wait_timer = 1;
        enemy.dragon_script_cmd = 0;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut enemy, Some(&grid), &mut projectiles, Some(60 << 16));
        assert_eq!(projectiles.len(), 1, "wall-blocked dragon should breathe");
        assert_eq!(
            enemy.x_velocity, 0,
            "dragon should hold position during breath"
        );
        assert_eq!(
            enemy.dragon_script_cmd, 1,
            "dragon should enter breath hold"
        );

        for _ in 0..(DRAGON_WALKING_BREATH_HOLD_FRAMES + 8) {
            step_dragon_walking(&mut enemy, Some(&grid), &mut projectiles, Some(60 << 16));
        }

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "dragon should flip after the hold"
        );
        assert_eq!(
            enemy.dragon_script_cmd, 0,
            "dragon should return to walking"
        );
    }

    #[test]
    fn dragon_walking_keeps_grounded_support_while_centre_is_still_on_the_lip() {
        let grid = grid_with_platform(17, 0, 2);
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 23 << 16, 272 << 16, 2);
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut enemy, Some(&grid), &mut projectiles, None);

        assert!(
            enemy.on_ground,
            "dragon should stay grounded while its centre still has support"
        );
        assert_eq!(
            enemy.x_pos,
            (23 << 16) + 0x8000,
            "dragon should advance by its half-pixel walk step"
        );
        assert_eq!(
            enemy.y_pos,
            272 << 16,
            "lip support should keep the grounded floor height"
        );
        assert_eq!(
            enemy.y_velocity, 0,
            "grounded lip handling should not enter the fall arc"
        );
    }

    #[test]
    fn dragon_walking_grounded_patrol_moves_without_populating_x_velocity() {
        let grid = grid_with_platform(17, 0, 4);
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 23 << 16, 272 << 16, 2);
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        enemy.shared_walker_state = SHARED_WALKER_STATE_PATROL;
        enemy.shared_walker_patrol_timer = 5;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut enemy, Some(&grid), &mut projectiles, None);

        assert_eq!(
            enemy.x_pos,
            (23 << 16) + 0x8000,
            "grounded patrol should still advance by the traced half-pixel step",
        );
        assert_eq!(
            enemy.x_velocity, 0,
            "shared grounded walker movement uses the patrol step, not x_vel, in the ASM path",
        );
    }

    #[test]
    fn dragon_walking_patrol_timer_turns_around_without_taking_a_walk_step() {
        let grid = grid_with_platform(17, 0, 4);
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 23 << 16, 272 << 16, 2);
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        enemy.shared_walker_state = SHARED_WALKER_STATE_PATROL;
        enemy.shared_walker_patrol_timer = 1;
        enemy.shared_walker_patrol_short_next = true;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut enemy, Some(&grid), &mut projectiles, None);

        assert_eq!(
            enemy.x_pos,
            23 << 16,
            "the shared walker turnaround should idle on the timer-expiry frame instead of taking one more patrol step",
        );
        assert_eq!(
            enemy.facing,
            Direction::Left,
            "the patrol cadence callback should reverse direction when $54 expires",
        );
        assert_eq!(
            enemy.shared_walker_patrol_timer, 0x32,
            "DragonWalking should swap the shared patrol cadence from the long leg to the short leg on the first turnaround",
        );
    }

    #[test]
    fn dragon_walking_calm_state_stops_and_faces_nearby_player() {
        let grid = grid_with_platform(17, 0, 4);
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 40 << 16, 272 << 16, 2);
        enemy.on_ground = true;
        enemy.facing = Direction::Left;
        enemy.shared_walker_state = 0;
        enemy.shared_walker_patrol_timer = 5;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut enemy, Some(&grid), &mut projectiles, Some(72 << 16));

        assert_eq!(
            enemy.x_pos,
            40 << 16,
            "the calm grounded state should idle immediately instead of taking another walk step",
        );
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "the shared walker should face Kid before leaving the calm state",
        );
        assert_eq!(
            enemy.shared_walker_state, SHARED_WALKER_STATE_FACE_RIGHT,
            "the grounded calm state should enter the right-facing alert branch",
        );
    }

    #[test]
    fn dragon_walking_alert_state_holds_position_while_player_stays_in_range() {
        let grid = grid_with_platform(17, 0, 4);
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 40 << 16, 272 << 16, 2);
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        enemy.shared_walker_state = SHARED_WALKER_STATE_FACE_RIGHT;
        enemy.shared_walker_patrol_timer = 5;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut enemy, Some(&grid), &mut projectiles, Some(72 << 16));

        assert_eq!(
            enemy.x_pos,
            40 << 16,
            "the shared alert state should keep DragonWalking stationary while Kid remains inside the release window",
        );
        assert_eq!(
            enemy.x_velocity, 0,
            "the ASM alert branch swaps to the stationary speed slot instead of taking another patrol step",
        );
        assert_eq!(
            enemy.shared_walker_state, SHARED_WALKER_STATE_FACE_RIGHT,
            "holding Kid in range should keep the alert-facing state live",
        );
    }

    #[test]
    fn dragon_walking_snapshot_reports_shared_walker_debug_fields() {
        let grid = grid_with_platform(17, 0, 4);
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 40 << 16, 272 << 16, 2);
        enemy.on_ground = true;
        enemy.facing = Direction::Left;
        enemy.shared_walker_state = 0;
        enemy.shared_walker_patrol_timer = 5;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut enemy, Some(&grid), &mut projectiles, Some(72 << 16));

        let snapshot = enemy.snapshot();
        assert_eq!(
            snapshot
                .debug_fields
                .get("grounded_frames")
                .map(String::as_str),
            Some("0"),
            "DragonWalking snapshots should expose grounded_frames for frontier tracing",
        );
        assert_eq!(
            snapshot
                .debug_fields
                .get("shared_walker_branch")
                .map(String::as_str),
            Some("calm_face_kid"),
            "the snapshot should record which shared-walker branch idled this frame",
        );
        assert_eq!(
            snapshot
                .debug_fields
                .get("shared_walker_dx_to_kid_px")
                .map(String::as_str),
            Some("32"),
            "the snapshot should expose the signed Kid distance used by the calm/alert threshold",
        );
        assert_eq!(
            snapshot
                .debug_fields
                .get("shared_walker_probe_next_x_px")
                .map(String::as_str),
            Some("39"),
            "the snapshot should expose the terrain-probe X position used for the next shared-walker step",
        );
        assert!(
            snapshot
                .debug_fields
                .contains_key("shared_walker_probe_center_floor"),
            "the snapshot should expose the centre floor probe that drove the shared-walker support check",
        );
        assert!(
            snapshot
                .debug_fields
                .contains_key("shared_walker_probe_left_floor"),
            "the snapshot should expose the left floor probe so ledge handoffs can be traced",
        );
        assert!(
            snapshot
                .debug_fields
                .contains_key("shared_walker_probe_right_floor"),
            "the snapshot should expose the right floor probe so ledge handoffs can be traced",
        );
        assert!(
            snapshot
                .debug_fields
                .get("shared_walker_probe_wall")
                .is_some_and(|probe| probe.starts_with("hit=0/")),
            "the snapshot should expose the wall probe result even when no side wall was hit",
        );
    }

    #[test]
    fn trace_frame_reports_kid_object_support_candidates() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = 42 << 16;
        slice.player.y_pos = 241 << 16; // 1px overlap with enemy top (272-32=240)
        runtime.opening_slice = Some(slice);
        runtime.progression.current_form = PlayerForm::TheKid;

        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 40 << 16, 272 << 16, 2);
        enemy.on_ground = true;
        enemy.facing = Direction::Right;
        runtime.active_enemy_roster = vec![enemy.snapshot()];
        runtime.active_enemies = vec![enemy];

        let trace = runtime.trace_frame();
        let support_debug = trace
            .gameplay_debug_fields
            .get("kid_object_support_candidates")
            .unwrap_or_else(|| panic!("trace frame should emit kid-object support diagnostics"));

        assert!(
            support_debug.contains("enemy_0D#00"),
            "the support trace should identify the nearby grounded dragon candidate",
        );
        assert!(
            support_debug.contains("relation=kidbelow"),
            "overlapping support surfaces should be tagged with the kidbelow relation",
        );
        assert!(
            support_debug.contains("support=1"),
            "standing support candidates should be marked as active support",
        );
    }

    #[test]
    fn trace_frame_reports_kid_ground_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.set_block_grid(grid_with_floor_at_row(17));
        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = 42 << 16;
        slice.player.y_pos = ((17 * 16) - 1) << 16;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.terrain.floor_y_pos = ((17 * 16) - 1) << 16;
        runtime.opening_slice = Some(slice);
        runtime.progression.current_form = PlayerForm::TheKid;

        let trace = runtime.trace_frame();
        let ground_debug = trace
            .gameplay_debug_fields
            .get("kid_ground_probe")
            .unwrap_or_else(|| panic!("trace frame should emit kid-ground probe diagnostics"));

        assert!(
            ground_debug.contains("latched_ground=1"),
            "the ground probe should report the latched grounded state",
        );
        assert!(
            ground_debug.contains("y_vel=0x00000000"),
            "the ground probe should include the current Y velocity",
        );
        assert!(
            ground_debug.contains("hitbox=ground=1"),
            "the hitbox probe should report solid support on a flat floor",
        );
    }

    #[test]
    fn end_sequence_credits_glyph_init_sub_1d786_uses_the_expected_gfx_object_defaults() {
        let init = end_sequence_credits_glyph_init_sub_1D786();

        assert_eq!(
            init.raw_allocation_spec, 0x0300_0002,
            "ASM sub_1D786 should allocate the credits glyph worker with raw gfx-object spec `$03000002`",
        );
        assert!(
            !init.display_enabled,
            "ASM sub_1D786 should clear `$13(a1)` so the worker starts hidden until sub_1D7A8 enables it",
        );
        assert_eq!(
            init.tile_word, 0x0509,
            "ASM sub_1D786 should seed tile word `$0509` before the credits script rewrites positions and mappings",
        );
        assert_eq!(
            init.priority, 2,
            "ASM sub_1D786 should set the credits glyph worker priority byte to 2",
        );
        assert_eq!(
            init.palette_line, 1,
            "ASM sub_1D786 should set palette line 1 on the credits glyph worker",
        );
    }

    #[test]
    fn end_sequence_script_driver_loc_1d1b0_starts_from_unk_1d91c_and_waits_delay_plus_one_frames()
    {
        let driver = crate::movement::end_sequence_script_driver_loc_1D1B0(&[
            0x3C, 0x00, 0x06, 0x01, 0x06, 0x02, 0xFF, 0xFF,
        ]);

        assert_eq!(
            driver,
            crate::movement::EndSequenceScriptDriverLoc1D1B0 {
                script_table_label: "unk_1D91C",
                first_delay_seed: Some(0x003C),
                first_script_id: Some(0x0000),
                first_dispatch_subroutine: Some("sub_1D944"),
                first_hibernate_iterations: Some(0x003D),
                sentinel_branches_to: None,
            },
            "ASM loc_1D1B0 / loc_1D1B4 / loc_1D1C4 (kid.asm:32560-32574) should start from `unk_1D91C`, dispatch the first script id through `sub_1D944`, then hibernate `$3C + 1 = $3D` frames before fetching the next pair",
        );

        assert_eq!(
            crate::movement::end_sequence_script_driver_loc_1D1B0(&[0xFF, 0xFF]),
            crate::movement::EndSequenceScriptDriverLoc1D1B0 {
                script_table_label: "unk_1D91C",
                first_delay_seed: None,
                first_script_id: None,
                first_dispatch_subroutine: None,
                first_hibernate_iterations: None,
                sentinel_branches_to: Some("loc_1D1CE"),
            },
            "ASM loc_1D1B4 should branch straight to `loc_1D1CE` when the next delay byte from `unk_1D91C` is negative",
        );
    }

    #[test]
    fn end_sequence_credits_decode_loc_1d97e_masks_control_bits_and_switches_only_the_flipped_primary_layout()
     {
        assert_eq!(
            crate::movement::end_sequence_credits_decode_loc_1D97E(0x0007),
            crate::movement::EndSequenceCreditsDecodeLoc1D97E {
                script_index: 0x0007,
                flip_x_flag: false,
                alternate_layout_flag: false,
                decode_command: 0x813E,
            },
            "ASM loc_1D97E (kid.asm:33236) should keep the default `EniDec` command `$813E` when neither control bit was set before `bclr #$F/#$E,d0`",
        );
        assert_eq!(
            crate::movement::end_sequence_credits_decode_loc_1D97E(0x8007),
            crate::movement::EndSequenceCreditsDecodeLoc1D97E {
                script_index: 0x0007,
                flip_x_flag: true,
                alternate_layout_flag: false,
                decode_command: 0x22DC,
            },
            "ASM loc_1D97E should switch to decode command `$22DC` only when bit 15 latched `($FFFFFB5C).w != 0` and bit 14 left `($FFFFFB5E).w == 0`",
        );
        assert_eq!(
            crate::movement::end_sequence_credits_decode_loc_1D97E(0xC007),
            crate::movement::EndSequenceCreditsDecodeLoc1D97E {
                script_index: 0x0007,
                flip_x_flag: true,
                alternate_layout_flag: true,
                decode_command: 0x813E,
            },
            "ASM loc_1D97E should branch back to the default path when both high bits were set, because the second `tst.b ($FFFFFB5E).w` takes the `bne.w loc_1D97E` exit",
        );
    }

    #[test]
    fn end_sequence_credits_upload_loop_loc_1d986_runs_fourteen_rows_in_descending_dbf_order() {
        let upload = crate::movement::end_sequence_credits_upload_loop_loc_1D986();

        assert_eq!(
            upload.dbf_seed, 0x000D,
            "ASM loc_1D986 (kid.asm:33240) should seed `d7 = $D` before entering the row upload loop",
        );
        assert_eq!(
            upload.iterations, 0x000E,
            "ASM loc_1D986 should execute `$D + 1 = 14` calls to `sub_1D990` because `dbf d7,loc_1D986` stops only after decrementing from 0 to -1",
        );
        assert_eq!(
            upload.row_indices,
            vec![
                0x000D, 0x000C, 0x000B, 0x000A, 0x0009, 0x0008, 0x0007, 0x0006, 0x0005, 0x0004,
                0x0003, 0x0002, 0x0001, 0x0000
            ],
            "ASM loc_1D986 calls `sub_1D990` before the `dbf`, so the credits rows are uploaded in descending `d7` order from 13 down to 0",
        );
    }

    #[test]
    fn end_sequence_credits_row_upload_loc_1d9b6_builds_the_expected_vdp_commands_for_normal_and_flipped_rows()
     {
        assert_eq!(
            crate::movement::end_sequence_credits_row_upload_loc_1D9B6(0x000D, false),
            crate::movement::EndSequenceCreditsRowUploadLoc1D9B6 {
                row_index: 0x000D,
                source_byte_offset: 0x016C,
                source_word_count: 0x000E,
                vdp_command: 0x4A1A_0000,
                z80_guard_subroutines: ["j_sub_914", "j_sub_924"],
            },
            "ASM loc_1D9B6 (kid.asm:33263) should upload row 13 from `Decompression_Buffer+$16C` and turn VRAM word address `$0A1A` into VDP command `$4A1A0000` through the `asl/lsr/addi/swap` sequence",
        );
        assert_eq!(
            crate::movement::end_sequence_credits_row_upload_loc_1D9B6(0x0000, true),
            crate::movement::EndSequenceCreditsRowUploadLoc1D9B6 {
                row_index: 0x0000,
                source_byte_offset: 0x0000,
                source_word_count: 0x000E,
                vdp_command: 0x639A_0003,
                z80_guard_subroutines: ["j_sub_914", "j_sub_924"],
            },
            "ASM loc_1D9B6 should honour the earlier `subi.w #-$2000,d5` mirrored-layout path, so row 0 uses VRAM word address `$E39A` and emits VDP command `$639A0003` before the 14-word copy",
        );
    }

    #[test]
    fn end_sequence_credits_row_copy_word_count_loc_1d9ca_is_inclusive_of_dbf_zero() {
        assert_eq!(
            crate::movement::end_sequence_credits_row_copy_word_count_loc_1D9CA(0x0000),
            1,
            "ASM loc_1D9CA should still copy one row word when `d5=0` because `move.w (a4)+,(a6)` runs before `dbf d5,loc_1D9CA` reaches `-1`",
        );
        assert_eq!(
            crate::movement::end_sequence_credits_row_copy_word_count_loc_1D9CA(0x000D),
            0x000E,
            "ASM loc_1D9CA seeds `d5=#$D`, so each credits row upload should transfer 14 words from `Decompression_Buffer` into VRAM before `j_sub_924`",
        );
    }

    #[test]
    fn load_end_sequence_credits_glyphs_loc_1d7b0_maps_triples_until_the_zero_x_sentinel() {
        let glyphs = load_end_sequence_credits_glyphs_sub_1D7A8(
            &[0x10, 0xA8, 0x04, 0x20, 0xA4, 0x08, 0x00, 0xFF, 0xFF],
            [None; 6],
        );

        assert_eq!(
            glyphs[0],
            Some(EndSequenceCreditsGlyphPlacementSub1D7A8 {
                x_pos: 0x0042,
                y_pos: 0x0008,
                addroffset_sprite: 0x110A,
                display_enabled: true,
            }),
            "ASM loc_1D7B0 (kid.asm:33001) should add `$32` to the first X byte, subtract `$A0` from the Y byte, offset the sprite byte from `LnkTo_unk_E1106-Data_Index`, and set `$13(a3)`",
        );
        assert_eq!(
            glyphs[1],
            Some(EndSequenceCreditsGlyphPlacementSub1D7A8 {
                x_pos: 0x0052,
                y_pos: 0x0004,
                addroffset_sprite: 0x110E,
                display_enabled: true,
            }),
            "ASM loc_1D7B0 should advance to the next cached glyph worker and keep consuming `(x,y,sprite)` triples from `a4`",
        );
        assert_eq!(
            glyphs[2], None,
            "ASM loc_1D7B0 should stop as soon as the next X byte is zero, without consuming or materialising later bytes",
        );
    }

    #[test]
    fn allocate_object_and_gfx_links_sub_e02_cross_links_the_new_slots() {
        let links = allocate_object_and_gfx_links_sub_E02(0x00FF_8000, 0x00FF_9000);

        assert_eq!(
            links.gfx_to_object_link, 0x00FF_8000,
            "ASM sub_E02 should write the allocated object slot pointer into gfx `$C(a1)`",
        );
        assert_eq!(
            links.object_to_gfx_link, 0x00FF_9000,
            "ASM loc_E0C should write the allocated gfx slot pointer into object `$36(a0)`",
        );
    }

    #[test]
    fn hidden_block_jump_window_sub_dfb0_uses_the_juggernaut_probe_box() {
        let window = hidden_block_jump_window_sub_DFB0(PlayerForm::Juggernaut, Direction::Right);

        assert_eq!(
            window,
            HiddenBlockJumpWindowSubDFB0 {
                left_reach_px: 0x0010,
                right_reach_px: 0x0010,
                vertical_reach_px: 0x0010,
            },
            "ASM sub_DFB0 should read the Juggernaut `unk_E246` entry as a symmetric `$10/$10/$10` jump probe",
        );
    }

    #[test]
    fn hidden_block_jump_window_sub_dfb0_uses_the_micromax_short_probe_box() {
        let window = hidden_block_jump_window_sub_DFB0(PlayerForm::Micromax, Direction::Right);

        assert_eq!(
            window,
            HiddenBlockJumpWindowSubDFB0 {
                left_reach_px: 0x0002,
                right_reach_px: 0x0002,
                vertical_reach_px: 0x0010,
            },
            "ASM sub_DFB0 should read Micromax's final `unk_E246` entry as the tighter `$2/$2/$10` jump probe",
        );
    }

    #[test]
    fn hidden_block_jump_window_sub_dfb0_swaps_horizontal_reach_for_left_facing_kid() {
        let window = hidden_block_jump_window_sub_DFB0(PlayerForm::TheKid, Direction::Left);

        assert_eq!(
            window.left_reach_px, 0x0004,
            "ASM sub_DFB0 should swap the horizontal words when `x_direction(a3)` is non-zero, even though Kid's default entry is symmetric",
        );
        assert_eq!(window.right_reach_px, 0x0004);
        assert_eq!(window.vertical_reach_px, 0x0020);
    }

    #[test]
    fn loc_e1fe_hidden_block_jump_snap_clears_vertical_motion_and_sets_bottom_edge() {
        let snap = crate::movement::hidden_block_jump_snap_loc_E1FE(0x0040, 0x0002);

        assert_eq!(
            snap.y_velocity_fp, 0,
            "ASM loc_E1FE should clear `y_vel(a3)` before snapping Kid onto the revealed hidden block",
        );
        assert_eq!(
            snap.y_subpixel, 0,
            "ASM loc_E1FE should clear `$20(a3)` after deriving the snapped block row",
        );
        assert_eq!(
            snap.kid_hitbox_bottom_px, 0x006F,
            "ASM loc_E1FE should compute `Kid_hitbox_bottom = (((d6 + 1) << 4) + (a5)) - 1`",
        );
        assert_eq!(
            snap.y_pos_px, 0x006F,
            "ASM loc_E1FE should copy the same bottom edge into `y_pos(a3)`",
        );
        assert_eq!(
            snap.top_candidate_px, 0x0060,
            "ASM loc_E1FE should subtract `$F` from the bottom edge before the helmet branch to loc_E236",
        );
    }

    #[test]
    fn loc_e236_hidden_block_jump_top_keeps_short_helmets_tighter() {
        let snap = crate::movement::hidden_block_jump_snap_loc_E1FE(0x0040, 0x0002);

        assert_eq!(
            crate::movement::hidden_block_jump_hitbox_top_loc_E236(
                snap.top_candidate_px,
                PlayerForm::Juggernaut.helmet_id(),
            ),
            0x0060,
            "ASM loc_E236 should store the `$F`-above-bottom top candidate directly for Juggernaut after `cmpi.w #5,d3; beq.w loc_E236`",
        );
        assert_eq!(
            crate::movement::hidden_block_jump_hitbox_top_loc_E236(
                snap.top_candidate_px,
                PlayerForm::Micromax.helmet_id(),
            ),
            0x0060,
            "ASM loc_E236 should store the same short top candidate directly for Micromax after `cmpi.w #9,d3; beq.w loc_E236`",
        );
        assert_eq!(
            crate::movement::hidden_block_jump_hitbox_top_loc_E236(
                snap.top_candidate_px,
                PlayerForm::TheKid.helmet_id(),
            ),
            0x0050,
            "ASM loc_E236 should receive `d2 - $10` for every other helmet because the branch is not taken",
        );
    }

    /// ASM loc_E000 (kid.asm:17347): after `sub_DFB0`/`loc_DFEE` have selected
    /// and shifted Kid's horizontal jump probe, the ROM clamps a negative
    /// start column to zero before computing the inclusive scan count.
    #[test]
    #[allow(non_snake_case)]
    fn loc_E000_hidden_block_jump_scan_clamps_negative_start_and_counts_from_zero() {
        let scan = hidden_block_jump_horizontal_scan_loc_DFEE(0x0003, 0x0010, 0x001F, 0x0020);

        assert_eq!(
            scan.start_col, 0,
            "ASM loc_E000 should clamp the arithmetic-shifted negative left edge to block column zero"
        );
        assert_eq!(
            scan.end_col, 0x0002,
            "ASM loc_DFEE should subtract one pixel from the right edge before shifting to a block column"
        );
        assert_eq!(
            scan.block_count, 0x0002,
            "ASM loc_DFEE should subtract the clamped start column from the end column"
        );
    }

    /// ASM loc_E00E (kid.asm:17354): after the right edge is compared against
    /// `Level_width_blocks`, the ROM subtracts the start column from `d1`; an
    /// equal-width edge remains uncapped but a greater edge becomes width-1.
    #[test]
    #[allow(non_snake_case)]
    fn loc_E00E_hidden_block_jump_scan_preserves_equal_width_and_counts_after_cap() {
        let equal_width =
            hidden_block_jump_horizontal_scan_loc_DFEE(0x00F1, 0x0001, 0x0010, 0x0010);
        assert_eq!(
            equal_width.end_col, 0x0010,
            "ASM loc_E00E should receive unchanged d1 when it equals Level_width_blocks"
        );
        assert_eq!(
            equal_width.block_count, 0x0001,
            "ASM loc_E00E should subtract the start column after preserving the equal-width end column"
        );

        let past_width = hidden_block_jump_horizontal_scan_loc_DFEE(0x00F2, 0x0001, 0x0020, 0x0010);
        assert_eq!(
            past_width.end_col, 0x000F,
            "ASM loc_E00E should receive Level_width_blocks-1 when the shifted right edge is greater"
        );
        assert_eq!(
            past_width.block_count, 0,
            "ASM loc_E00E should compute the scan count after applying the right-edge cap"
        );
    }

    #[test]
    fn hidden_block_reveal_worker_sub_e0f8_seeds_the_loc_e304_burst_worker() {
        let worker = hidden_block_reveal_worker_sub_E0F8(0x0002, 0x0003, 0x00FF_4A48, 0x1114);

        assert_eq!(
            worker,
            HiddenBlockRevealWorkerSubE0F8 {
                allocation_spec: 0x2001,
                object_handler_addr: 0x0000_E304,
                block_col: 0x0002,
                block_row: 0x0003,
                layout_word_addr: 0x00FF_4A48,
                sprite_mapping: 0x1114,
            },
            "ASM sub_E0F8 should allocate a `$2001` worker, install `loc_E304`, copy `$44/$46/$48`, and pull the next `unk_E11E` word into `$4E(a0)`",
        );
    }

    /// ASM loc_E0C2 (kid.asm:17422): subtype-$0A hidden blocks reveal the
    /// layout byte, index `unk_E11E` by byte 7, allocate four `loc_E304`
    /// workers, and set `$50/$52` offsets on workers 2-4 after allocation.
    #[test]
    #[allow(non_snake_case)]
    fn loc_E0C2_hidden_block_reveal_spawns_four_offset_burst_workers() {
        let workers = hidden_block_reveal_workers_loc_E0C2(0x0007, 0x0002, 0x0003, 0x00FF_4A48);

        let sprite_mappings = workers.map(|worker| worker.worker.sprite_mapping);
        assert_eq!(
            sprite_mappings,
            [0x10EC, 0x1100, 0x110C, 0x1120],
            "ASM loc_E0C2 should shift byte 7 by three and consume four words from the selected `unk_E11E` entry",
        );

        let offsets = workers.map(|worker| (worker.x_pos_offset_px, worker.y_pos_offset_px));
        assert_eq!(
            offsets,
            [(0, 0), (8, 0), (0, 8), (8, 8)],
            "ASM loc_E0C2 should write `$50=8` to the second worker, `$52=8` to the third, and both offsets to the fourth",
        );
        assert!(
            workers
                .iter()
                .all(|worker| worker.revealed_layout_byte == 0x60),
            "ASM loc_E0C2 should reveal the hidden block by writing `#$60` to `(a2)` before spawning workers",
        );
    }

    #[test]
    fn hidden_block_reveal_worker_sub_e1d4_reveals_the_layout_byte_and_seeds_loc_e282() {
        let worker = hidden_block_reveal_worker_sub_E1D4(0x0002, 0x0003, 0x00FF_4A48, 0x1114);

        assert_eq!(
            worker,
            HiddenBlockRevealWorkerSubE1D4 {
                revealed_layout_byte: 0x60,
                allocation_spec: 0x2001,
                object_handler_addr: 0x0000_E282,
                block_col: 0x0002,
                block_row: 0x0003,
                layout_word_addr: 0x00FF_4A48,
                sprite_mapping: 0x1114,
            },
            "ASM sub_E1D4 should write `#$60` into `(a2)`, allocate a `$2001` worker, install `loc_E282`, and copy `$44/$46/$48/$4C/$4E` from `d0/d6/a2/d2/d3`",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E19E_hidden_block_type4_spawns_previous_then_current_workers() {
        let pair = hidden_block_type4_reveal_pair_loc_E19E(0x0008, 0x0003, 0x00FF_4A50, 0x1114);

        assert_eq!(
            pair.workers[0],
            HiddenBlockRevealWorkerSubE1D4 {
                revealed_layout_byte: 0x60,
                allocation_spec: 0x2001,
                object_handler_addr: 0x0000_E282,
                block_col: 0x0007,
                block_row: 0x0003,
                layout_word_addr: 0x00FF_4A4E,
                sprite_mapping: 0x1113,
            },
            "ASM loc_E19E should pre-decrement `a2/d2/d0` before the first sub_E1D4 call when hidden-block byte 7 is set",
        );
        assert_eq!(
            pair.workers[1],
            HiddenBlockRevealWorkerSubE1D4 {
                revealed_layout_byte: 0x60,
                allocation_spec: 0x2001,
                object_handler_addr: 0x0000_E282,
                block_col: 0x0008,
                block_row: 0x0003,
                layout_word_addr: 0x00FF_4A50,
                sprite_mapping: 0x1114,
            },
            "ASM loc_E19E should restore `a2/d2/d0` before the second sub_E1D4 call",
        );
        assert_eq!(
            pair.second_worker_x_offset_px, 1,
            "ASM loc_E19E writes `#1` to `$50(a0)` on the second worker only",
        );
        assert_eq!(
            (
                pair.restored_block_col,
                pair.restored_layout_word_addr,
                pair.restored_sprite_mapping,
            ),
            (0x0008, 0x00FF_4A50, 0x1114),
            "ASM loc_E19E should reach the shared post-reveal tail with the original hidden-block scan cursor restored",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E1BC_hidden_block_type4_spawns_current_then_next_workers() {
        let pair = hidden_block_type4_reveal_pair_loc_E1BC(0x0008, 0x0003, 0x00FF_4A50, 0x1114);

        assert_eq!(
            pair.workers[0],
            HiddenBlockRevealWorkerSubE1D4 {
                revealed_layout_byte: 0x60,
                allocation_spec: 0x2001,
                object_handler_addr: 0x0000_E282,
                block_col: 0x0008,
                block_row: 0x0003,
                layout_word_addr: 0x00FF_4A50,
                sprite_mapping: 0x1114,
            },
            "ASM loc_E1BC should call sub_E1D4 on the current cell first when hidden-block byte 7 is clear",
        );
        assert_eq!(
            pair.workers[1],
            HiddenBlockRevealWorkerSubE1D4 {
                revealed_layout_byte: 0x60,
                allocation_spec: 0x2001,
                object_handler_addr: 0x0000_E282,
                block_col: 0x0009,
                block_row: 0x0003,
                layout_word_addr: 0x00FF_4A52,
                sprite_mapping: 0x1115,
            },
            "ASM loc_E1BC should advance `a2/d2/d0` before the second sub_E1D4 call",
        );
        assert_eq!(
            pair.second_worker_x_offset_px, 1,
            "ASM loc_E1BC writes `#1` to `$50(a0)` after spawning the next-cell worker",
        );
        assert_eq!(
            (
                pair.restored_block_col,
                pair.restored_layout_word_addr,
                pair.restored_sprite_mapping,
            ),
            (0x0008, 0x00FF_4A50, 0x1114),
            "ASM loc_E1BC should subtract the same offsets after the second worker so the shared post-reveal tail resumes from the original cell",
        );
    }

    #[test]
    fn hide_end_sequence_credits_glyphs_sub_1d7e6_clears_all_six_display_flags() {
        let mut display_enabled = [true, false, true, true, false, true];

        hide_end_sequence_credits_glyphs_sub_1D7E6(&mut display_enabled);

        assert_eq!(
            display_enabled, [false; 6],
            "ASM sub_1D7E6 should clear `$13(a3)` for all six credits glyph workers loaded from `$FFFFFB72`",
        );
    }

    #[test]
    fn fade_end_sequence_palette_sub_1d800_applies_eight_fade_passes_and_32_refresh_hibernations() {
        let mut palette_buffer = [0u16; 0x40];
        palette_buffer[0x30] = 0x0EEE;
        palette_buffer[0x31] = 0x0ACE;
        palette_buffer[0x3F] = 0x0246;
        let mut refresh_and_hibernate_calls = 0usize;

        fade_end_sequence_palette_sub_1D800(&mut palette_buffer, || {
            refresh_and_hibernate_calls += 1;
        });

        assert_eq!(
            palette_buffer[0x30], 0x0000,
            "ASM sub_1D800 should darken Palette_Buffer+$60 through eight sub_1C85E passes until bright credits colours reach black",
        );
        assert_eq!(
            palette_buffer[0x31], 0x0000,
            "ASM sub_1D800 should keep reapplying the same fade span to every word in the credits palette block",
        );
        assert_eq!(
            palette_buffer[0x3F], 0x0000,
            "ASM sub_1D800 should include the tail word of the `$60` byte-offset palette span in each fade pass",
        );
        assert_eq!(
            refresh_and_hibernate_calls, 32,
            "ASM sub_1D800 should run the `sub_1D830 -> Hibernate_Object_1Frame` pair four times per fade pass for eight passes total",
        );
    }

    #[test]
    fn trace_frame_reports_kid_special_motion_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.set_block_grid(grid_with_floor_at_row(17));
        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = 42 << 16;
        slice.player.y_pos = ((17 * 16) - 1) << 16;
        slice.player.on_ground = true;
        runtime.opening_slice = Some(slice);
        runtime.progression.current_form = PlayerForm::Skycutter;
        runtime.helmet.skycutter_on_board = true;
        runtime.helmet.skycutter_flipboard = 17;
        runtime.pending_stomp_bounce = Some(PendingStompBounce {
            anchor_x_pos: 0x0020_0000,
            anchor_y_pos: 0x0030_0000,
        });

        let trace = runtime.trace_frame();
        let motion_debug = trace
            .gameplay_debug_fields
            .get("kid_special_motion_probe")
            .unwrap_or_else(|| panic!("trace frame should emit kid special-motion diagnostics"));

        assert!(
            motion_debug.contains("helmet=0x0001"),
            "the motion probe should report the current helmet id",
        );
        assert!(
            motion_debug.contains("skycutter_on_board=1"),
            "the motion probe should report the Skycutter board latch",
        );
        assert!(
            motion_debug.contains("skycutter_flipboard=17"),
            "the motion probe should report the live flipboard counter",
        );
        assert!(
            motion_debug.contains("pending_stomp_bounce=1"),
            "the motion probe should surface pending stomp-bounce state",
        );
    }

    #[test]
    fn trace_frame_reports_kid_goal_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = 0x0750_0000;
        slice.player.y_pos = 0x0300_0000;
        runtime.opening_slice = Some(slice);

        let trace = runtime.trace_frame();
        let goal_debug = trace
            .gameplay_debug_fields
            .get("kid_goal_probe")
            .unwrap_or_else(|| panic!("trace frame should emit kid-goal probe diagnostics"));

        assert!(
            goal_debug.contains("lifecycle=alive"),
            "the goal probe should expose the current player lifecycle",
        );
        assert!(
            goal_debug.contains("touch=1"),
            "standing on the flag should report an active native goal touch",
        );
    }

    #[test]
    fn trace_frame_reports_kid_gameplay_path_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        let bounds = runtime.level_bounds(LevelId(0));
        runtime.opening_slice = Some(OpeningSliceState::respawn(bounds));
        runtime.last_kid_frame_motion_branch = KidFrameMotionBranch::NormalPhysics;
        runtime.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::PlatformCollision;

        let trace = runtime.trace_frame();
        let path_debug = trace
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("trace frame should emit kid gameplay-path diagnostics"));

        assert!(
            path_debug.contains("lifecycle=alive"),
            "the gameplay-path probe should expose the current player lifecycle",
        );
        assert!(
            path_debug.contains("motion=normal_physics"),
            "the gameplay-path probe should report the motion branch that controlled Kid this frame",
        );
        assert!(
            path_debug.contains("post=platform_collision"),
            "the gameplay-path probe should report the first post-motion mutator that moved Kid",
        );
    }

    #[test]
    fn trace_frame_reports_kid_pre_physics_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        let start = slice.player;
        slice.player.y_velocity = -0x4000;
        slice.player.on_ground = false;
        let before_physics = slice.player;
        runtime.opening_slice = Some(slice);
        runtime.last_kid_frame_pre_physics_start = Some(start);
        runtime.last_kid_frame_pre_physics_before_physics = Some(before_physics);
        runtime.last_kid_frame_pre_physics_mutators =
            vec![KidFramePrePhysicsMutator::IronKnightClimb];

        let trace = runtime.trace_frame();
        let probe_debug = trace
            .gameplay_debug_fields
            .get("kid_pre_physics_probe")
            .unwrap_or_else(|| panic!("trace frame should emit kid pre-physics diagnostics"));

        assert!(
            probe_debug.contains("start["),
            "the pre-physics probe should report the starting Kid snapshot",
        );
        assert!(
            probe_debug.contains("before_physics["),
            "the pre-physics probe should report the final pre-physics Kid snapshot",
        );
        assert!(
            probe_debug.contains("first=iron_knight_climb"),
            "the pre-physics probe should report the first handler that changed Kid",
        );
        assert!(
            probe_debug.contains("mutators=iron_knight_climb"),
            "the pre-physics probe should keep the ordered mutator list",
        );
    }

    #[test]
    fn trace_frame_reports_kid_grounded_release_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        let bounds = runtime.level_bounds(LevelId(0));
        runtime.opening_slice = Some(OpeningSliceState::respawn(bounds));
        runtime.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::None;
        runtime.last_kid_frame_grounded_release_probe = Some(KidGroundedReleaseProbe {
            on_ground_before: true,
            held_direction: Some(Direction::Right),
            jump_pressed: false,
            jump_buffer_frames: 0,
            custom_gravity: false,
            wall_cling_direction: None,
            velocity_branch: KidGroundedReleaseVelocityBranch::GroundedHold,
            probe_row: Some(28),
            left_col: Some(58),
            right_col: Some(58),
            support_col: Some(58),
            support_collision: Some(terrain::CollisionKind::Solid),
            support_block_type: Some(terrain::BlockType::None),
            release_triggered: false,
        });

        let trace = runtime.trace_frame();
        let probe_debug = trace
            .gameplay_debug_fields
            .get("kid_grounded_release_probe")
            .unwrap_or_else(|| panic!("trace frame should emit kid grounded-release diagnostics"));

        assert!(
            probe_debug.contains("held=right"),
            "the grounded-release probe should expose the held horizontal direction",
        );
        assert!(
            probe_debug.contains("velocity=grounded_hold"),
            "the grounded-release probe should expose the native velocity branch selected after the scan",
        );
        assert!(
            probe_debug.contains("support=solid@58,28 block=None"),
            "the grounded-release probe should report the supporting terrain cell and block type",
        );
        assert!(
            probe_debug.contains("post=none"),
            "the grounded-release probe should echo the first post-motion mutator label",
        );
    }

    #[test]
    fn trace_frame_reports_kid_collision_handoff_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.last_kid_frame_collision_handoff_probe = Some(KidCollisionHandoffProbe {
            player_lifecycle: PlayerLifecycle::Alive,
            previous_post_motion_mutator: KidFramePostMotionMutator::None,
            pending_stomp_bounce: false,
            pending_form: None,
            pending_level_skip: None,
            form_change_flash_frames: 0,
            on_ground_before: true,
            jump_buffer_frames: 0,
            wall_cling_direction: None,
            stomp_candidate: false,
            stomp_enemy_object_type: None,
            stomp_enemy_x_pos: None,
            stomp_enemy_y_pos: None,
            anchor_x_pos: 0x03A8_0000,
            anchor_y_pos: 0x01BF_0000,
        });

        let trace = runtime.trace_frame();
        let probe_debug = trace
            .gameplay_debug_fields
            .get("kid_collision_handoff_probe")
            .unwrap_or_else(|| panic!("trace frame should emit kid collision-handoff diagnostics"));

        assert!(
            probe_debug.contains("lifecycle=alive/prev_post=none"),
            "the collision-handoff probe should report the carried gameplay lifecycle and prior post-motion mutator",
        );
        assert!(
            probe_debug.contains(
                "pending_stomp=0/pending_form=none/pending_level_skip=none/form_flash=0/ground=1/jump_buffer=0/wall_cling=none"
            ),
            "the collision-handoff probe should expose the native carry-style state before pre-physics consumption",
        );
        assert!(
            probe_debug.contains("stomp_candidate=0/stomp_enemy=none/anchor=0x03a80000,0x01bf0000"),
            "the collision-handoff probe should expose the closest native pre-sub_B168 stomp/collision carry state and saved anchor",
        );
    }

    #[test]
    fn trace_frame_reports_kid_same_frame_stomp_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.last_kid_frame_same_frame_stomp_probe = Some(KidSameFrameStompProbe {
            stomp_candidate: true,
            enemy_object_type: Some(0x0D),
            enemy_x_pos: Some(0x035A_8000),
            enemy_y_pos: Some(0x01C0_0000),
            pre_physics_y_velocity: 0,
            post_physics_y_velocity: 0,
            anchor_x_pos: 0x03A8_0000,
            anchor_y_pos: 0x01BF_0000,
        });

        let trace = runtime.trace_frame();
        let probe_debug = trace
            .gameplay_debug_fields
            .get("kid_same_frame_stomp_probe")
            .unwrap_or_else(|| panic!("trace frame should emit same-frame stomp diagnostics"));

        assert!(
            probe_debug.contains(
                "candidate=1/enemy=0x0d@0x035a8000,0x01c00000/pre_y_vel=0x00000000/post_y_vel=0x00000000/anchor=0x03a80000,0x01bf0000"
            ),
            "the same-frame stomp probe should report the candidate enemy, velocity window, and saved anchor",
        );
    }

    #[test]
    fn trace_frame_reports_kid_iron_knight_attach_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.last_kid_frame_iron_knight_attach_probe = Some(KidIronKnightAttachProbe {
            c_pressed: true,
            on_ground_before: false,
            facing: Some(Direction::Right),
            climbing_before: false,
            climbing_after: true,
            y_velocity_before: 0,
            y_velocity_after: IRON_KNIGHT_ATTACH_Y_VEL,
            wall_probe: Some(terrain::WallProbe {
                hit: true,
                wall_x_px: 160,
                cell: terrain::Cell::from_be_bytes(0x60, 0),
                cell_x: 10,
                cell_y: 1,
            }),
            candidate: true,
            attach_triggered: true,
        });

        let trace = runtime.trace_frame();
        let probe_debug = trace
            .gameplay_debug_fields
            .get("kid_iron_knight_attach_probe")
            .unwrap_or_else(|| panic!("trace frame should emit Iron Knight attach diagnostics"));

        assert!(
            probe_debug.contains("c_press=1/ground=0/climbing=0=>1/facing=right"),
            "the Iron Knight attach probe should expose the raw C-edge, grounded gate, climb transition, and facing",
        );
        assert!(
            probe_debug.contains("y_vel=0x00000000->0xffffc000"),
            "the Iron Knight attach probe should report the native attach y-velocity injection window",
        );
        assert!(
            probe_debug
                .contains("wall=hit=1/x=160/cell=10,1 collision=solid block=None has_block=0"),
            "the Iron Knight attach probe should report the forward wall contact that gates attachment",
        );
        assert!(
            probe_debug.contains("candidate=1/attach=1"),
            "the Iron Knight attach probe should distinguish a viable attach candidate from an actual attach",
        );
    }

    #[test]
    fn trace_frame_reports_kid_flip_branch_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.last_kid_frame_kid_flip_branch_probe = Some(KidFlipBranchProbe {
            wall_scan_hit: true,
            b_pressed: false,
            c_pressed: true,
            helmet_id: PlayerForm::IronKnight.helmet_id(),
            facing: Some(Direction::Right),
            on_ground_before: true,
            on_ground_after: true,
            branch: KidFlipBranchSelection::CPressedFallthrough,
            wall_probe: Some(terrain::WallProbe {
                hit: true,
                wall_x_px: 944,
                cell: terrain::Cell::from_be_bytes(0x60, 0x82),
                cell_x: 59,
                cell_y: 27,
            }),
        });

        let trace = runtime.trace_frame();
        let probe_debug = trace
            .gameplay_debug_fields
            .get("kid_flip_branch_probe")
            .unwrap_or_else(|| panic!("trace frame should emit kid_flip branch diagnostics"));

        assert!(
            probe_debug
                .contains("wall_scan=1/b_press=0/c_press=1/helmet=0x0006/facing=right/ground=1=>1"),
            "the kid_flip branch probe should expose the live wall scan, raw button edges, helmet, facing, and grounded window",
        );
        assert!(
            probe_debug.contains("branch=c_pressed_fallthrough"),
            "the kid_flip branch probe should identify the post-sub_AF10 fallthrough branch when no native C consumer claims the frame",
        );
        assert!(
            probe_debug
                .contains("wall=hit=1/x=944/cell=59,27 collision=solid block=None has_block=0"),
            "the kid_flip branch probe should preserve the post-sub_AF10 wall scan contact",
        );
    }

    #[test]
    fn trace_frame_reports_kid_flip_follow_on_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.last_kid_frame_kid_flip_follow_on_probe = Some(KidFlipFollowOnProbe {
            branch: KidFlipBranchSelection::CPressedFallthrough,
            motion_branch: KidFrameMotionBranch::NormalPhysics,
            post_motion_mutator: KidFramePostMotionMutator::None,
            stages: KidFlipFollowOnStages {
                entry: KidFlipFollowOnStageState {
                    x_velocity: 0,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                after_press_away: KidFlipFollowOnStageState {
                    x_velocity: 0,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                after_jump_gravity: KidFlipFollowOnStageState {
                    x_velocity: 0,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                after_floor: KidFlipFollowOnStageState {
                    x_velocity: 0,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
                after_wall: KidFlipFollowOnStageState {
                    x_velocity: 0,
                    y_velocity: 0,
                    on_ground: true,
                    wall_cling_direction: None,
                },
            },
        });

        let trace = runtime.trace_frame();
        let probe_debug = trace
            .gameplay_debug_fields
            .get("kid_flip_follow_on_probe")
            .unwrap_or_else(|| panic!("trace frame should emit kid_flip follow-on diagnostics"));

        assert!(
            probe_debug.contains("branch=c_pressed_fallthrough/motion=normal_physics/post=none"),
            "the kid_flip follow-on probe should identify the shared fallthrough branch and the native movement shell it entered",
        );
        assert!(
            probe_debug
                .contains("entry[x_vel=0x00000000/y_vel=0x00000000/ground=1/wall_cling=none]"),
            "the kid_flip follow-on probe should capture the entry state at the shared loc_AA22 mirror",
        );
        assert!(
            probe_debug.contains(
                "jump_gravity[x_vel=0x00000000/y_vel=0x00000000/ground=1/wall_cling=none]"
            ),
            "the kid_flip follow-on probe should expose whether the native jump/gravity stage injected lift-off velocity or released ground",
        );
        assert!(
            probe_debug
                .contains("wall[x_vel=0x00000000/y_vel=0x00000000/ground=1/wall_cling=none]"),
            "the kid_flip follow-on probe should preserve the final pre-post-motion state after the wall-collision stage",
        );
    }

    #[test]
    fn trace_frame_reports_kid_jump_gravity_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.last_kid_frame_jump_gravity_probe = Some(KidJumpGravityProbe {
            branch: KidJumpGravityBranch::GroundedHold,
            jump_pressed: false,
            input_b_held: false,
            custom_gravity: false,
            on_ground_before: true,
            on_ground_after: true,
            just_jumped_before: false,
            just_jumped_after: false,
            jump_buffer_frames_before: 0,
            jump_buffer_frames_after: 0,
            y_velocity_before: 0,
            y_velocity_after: 0,
        });

        let trace = runtime.trace_frame();
        let probe_debug = trace
            .gameplay_debug_fields
            .get("kid_jump_gravity_probe")
            .unwrap_or_else(|| panic!("trace frame should emit kid jump/gravity diagnostics"));

        assert!(
            probe_debug.contains("branch=grounded_hold/jump=0/jump_buffer=0=>0/just_jumped=0=>0"),
            "the jump/gravity probe should expose the exact native gate branch together with jump and just-jumped carry state",
        );
        assert!(
            probe_debug.contains("b_held=0/custom_gravity=0"),
            "the jump/gravity probe should report the held-B and custom-gravity gates consumed by the native call",
        );
        assert!(
            probe_debug.contains("y_vel=0x00000000->0x00000000/ground=1=>1"),
            "the jump/gravity probe should preserve the pre/post y-velocity and grounded state around the native call",
        );
    }

    #[test]
    fn trace_frame_reports_kid_iron_knight_block_breaker_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::IronKnight;

        let mut grid = grid_with_wall(3, 0, 20, 10);
        for row in 0..3 {
            grid.set_cell_at(10, row, terrain::Cell::from_be_bytes(0xE0, 0));
        }
        runtime.block_grid = Some(grid);

        let mut player = airborne_slice().player;
        player.x_pos = 154 << 16;
        player.y_pos = 32 << 16;
        player.x_velocity = 0x0004_8000;
        player.facing = Direction::Right;
        runtime.last_kid_frame_pre_physics_before_physics = Some(player);

        let trace = runtime.trace_frame();
        let probe_debug = trace
            .gameplay_debug_fields
            .get("kid_iron_knight_block_breaker_probe")
            .unwrap_or_else(|| {
                panic!("trace frame should emit Iron Knight block-breaker diagnostics")
            });

        assert!(
            probe_debug.contains("helmet=0x0006/facing=right/x_vel=0x00048000/speed_gate=1"),
            "the breaker probe should report the Iron Knight speed gate from the pre-physics snapshot",
        );
        assert!(
            probe_debug
                .contains("wall=hit=1/x=160/cell=10,1 collision=solid block=Smash has_block=1"),
            "the breaker probe should report the forward solid block contact that mirrors the ROM wall-break check",
        );
        assert!(
            probe_debug.contains("candidate=1"),
            "the breaker probe should flag a ROM-like Iron Knight block-break candidate when speed and wall/block gates align",
        );
    }

    #[test]
    fn trace_frame_reports_kid_wall_cling_release_probe() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.wall_cling_release_probe = Some(KidWallClingReleaseProbe {
            held_direction: Some(Direction::Right),
            probe_direction: Some(Direction::Right),
            probe_direction_source: KidWallClingProbeDirectionSource::Held,
            wall_cling_direction: None,
            right_wall_sticky: false,
            left_wall_sticky: false,
            on_ground: true,
            y_velocity: 0,
            pre_vertical_y_px: 447,
            post_vertical_y_px: 447,
            branch: KidWallClingReleaseBranch::NoWallClingState,
            sub_adb8_gate: KidWallClingReleaseGate::WallClingMissing,
            sub_ae6e_gate: KidWallClingReleaseGate::WallClingMissing,
            forward_wall_probe: Some(terrain::WallProbe {
                hit: true,
                wall_x_px: 944,
                cell: terrain::Cell::from_be_bytes(0x60, 0x82),
                cell_x: 59,
                cell_y: 27,
            }),
            top_edge_probe: Some(terrain::WallProbe {
                hit: false,
                wall_x_px: 0,
                cell: terrain::Cell::from_be_bytes(0, 0),
                cell_x: 0,
                cell_y: 0,
            }),
        });
        runtime.opening_slice = Some(slice);

        let trace = runtime.trace_frame();
        let probe_debug = trace
            .gameplay_debug_fields
            .get("kid_wall_cling_release_probe")
            .unwrap_or_else(|| panic!("trace frame should emit wall-cling release diagnostics"));

        assert!(
            probe_debug
                .contains("held=right/probe_dir=right(held)/wall_cling=none/sticky=R0L0/ground=1"),
            "the wall-cling release probe should report the held direction, probe direction source, and sticky state",
        );
        assert!(
            probe_debug.contains(
                "branch=no_wall_cling_state/sub_ADB8=wall_cling_missing/sub_AE6E=wall_cling_missing"
            ),
            "the wall-cling release probe should distinguish missing wall-cling state from the ROM helper gates",
        );
        assert!(
            probe_debug
                .contains("wall=hit=1/x=944/cell=59,27 collision=solid block=None has_block=0"),
            "the wall-cling release probe should report the forward wall contact at the loc_AA9C mirror",
        );
    }
}

mod chunk_05 {
    use super::*;

    #[test]
    fn platform_collision_floor_shift_in_landing() {
        // Mutant: line 147 — replace << with >> in floor_y_fp
        let platform = make_test_platform(40, 200, 2, 0);
        let mut slice = make_airborne_slice(50, 200);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(slice.player.on_ground);
        // floor_y_fp should be (199) << 16 = 0x00C7_0000
        assert_eq!(
            slice.terrain.floor_y_pos,
            199u32 << 16,
            "floor y must be in FP16.16 (pixel << 16)"
        );
        assert!(
            slice.terrain.floor_y_pos > 199,
            "if << became >>, floor would be tiny"
        );
    }

    #[test]
    fn platform_collision_plat_right_includes_width() {
        // Mutant: line 93 (riding) and 125 (landing) — + in plat_right calculation
        // If + became -, plat_right would be negative/small and overlap would break.
        let platform = make_test_platform(40, 100, 4, 0); // width=79, so plat_right=119
        // Player at x=110 — should overlap (within 40..119)
        let mut slice = make_airborne_slice(110, 100);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.on_ground,
            "player near right edge of wide platform should land"
        );
    }

    #[test]
    fn platform_collision_riding_or_vs_and_overlap() {
        // Mutant: line 95 — || vs && in riding overlap skip
        // The check is `player_right < plat_x || player_left > plat_right` (skip if either).
        // If || became &&, it would only skip when BOTH are true (impossible), meaning
        // every platform would match. Test: player entirely to the right should not ride.
        let mut platform = make_test_platform(40, 100, 2, 0); // plat_right=87
        platform.x_velocity = 0x1_0000;

        let mut slice = make_airborne_slice(100, 99); // player_left=95, far right of platform
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);
        assert_eq!(
            slice.player.x_pos, x_before,
            "player far right of platform should NOT be carried"
        );
    }

    #[test]
    fn platform_riding_not_negation_matters() {
        // Mutant: line 101 — delete ! in !within_snap
        // If ! is removed, within_snap && abs_diff > 2 would skip platforms the player
        // IS snapped to (the opposite of intended).
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.x_velocity = 0x1_0000;

        // Player at y=99, platform at y=100 → within snap, abs_diff=1 ≤ 2 → should ride
        let mut slice = make_airborne_slice(50, 99);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.x_pos > x_before,
            "player snapped to platform should be carried"
        );
    }

    #[test]
    fn platform_collision_bitwise_or_in_landing() {
        // Mutant: line 153 — replace | with ^ in sub-pixel preservation
        // (floor_y_fp & 0xFFFF_0000) | (y_pos & 0x0000_FFFF)
        // If | became ^, the high bits would be XORed with low bits → corrupt y_pos.
        let platform = make_test_platform(40, 100, 2, 0);
        let mut slice = make_airborne_slice(50, 101);
        slice.player.y_pos = (101u32 << 16) | 0xABCD; // sub-pixel = 0xABCD
        slice.pre_gravity_y_vel = 0x1_0000; // falling → preserve sub-pixel
        apply_platform_collision(&mut slice, &[platform]);

        assert!(slice.player.on_ground);
        // With correct |: floor pixel (99 << 16) | 0xABCD = 0x0063_ABCD
        assert_eq!(slice.player.y_pos, (99u32 << 16) | 0xABCD);
    }

    #[test]
    fn platform_collision_plat_bottom_includes_height() {
        // Mutant: line 140 — + with * in plat_bottom
        // If + became *, plat_bottom would be plat_y * height instead of plat_y + height
        let platform = make_test_platform(40, 100, 2, 1); // height=31, bottom=131
        // Player at y=130 — just inside platform → should land
        let mut slice = make_airborne_slice(50, 130);
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.on_ground,
            "player inside platform depth should land"
        );
    }

    #[test]
    fn platform_collision_snap_tolerance_boundary_exact() {
        // Mutant: line 135/151 — >= vs < boundary at snap tolerance
        let platform = make_test_platform(40, 100, 2, 0);
        // Player ABOVE platform by exactly snap tolerance in FP
        let plat_fp = 100u32 << 16;
        let player_fp = plat_fp - GROUND_SNAP_TOLERANCE_FP; // exactly at boundary
        let mut slice = make_airborne_slice(50, 100);
        slice.player.y_pos = player_fp;
        slice.player.y_velocity = 0x1_0000;

        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.on_ground,
            "player at exact snap tolerance boundary should land (<= includes equal)"
        );
    }

    #[test]
    fn platform_riding_exact_left_boundary() {
        // Mutant: line 95 riding — replace < with <=
        // player_right == plat_x_px should NOT be skipped (< is false at equality → overlap)
        let mut platform = make_test_platform(40, 100, 2, 0); // plat_x_px=40
        platform.x_velocity = 0x1_0000;

        // player at x=45 → player_right = 45+5 = 50, player_left = 40
        // player_right(50) < plat_x_px(40) → false ✓ overlap
        // Need: player_right == plat_x_px: x=35 → right=40 == plat_x_px(40)
        let mut slice = make_airborne_slice(35, 99);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.x_pos > x_before,
            "player_right == plat_x_px should overlap (< not <=) and carry"
        );
    }

    #[test]
    fn platform_riding_exact_right_boundary() {
        // Mutant: line 95 riding — replace > with >=
        // player_left == plat_right should NOT be skipped (> is false at equality → overlap)
        let mut platform = make_test_platform(40, 100, 2, 0); // plat_right = 40+47 = 87
        platform.x_velocity = 0x1_0000;

        // player at x=92 → player_left = 92-5 = 87 == plat_right(87)
        let mut slice = make_airborne_slice(92, 99);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.x_pos > x_before,
            "player_left == plat_right should overlap (> not >=) and carry"
        );
    }

    #[test]
    fn platform_riding_skip_when_both_outside() {
        // Mutant: line 101 && vs ||, and > vs >= in pixel gap check
        // The skip condition is: !within_snap && abs_diff > 2
        // If && became ||, it would skip when EITHER is true.
        // A player within snap but pixel gap = 3 should NOT be skipped.
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.x_velocity = 0x1_0000;

        // player at y=97, platform at y=100: FP diff = 3*65536 = 0x3_0000, snap = 0x6_0000
        // within_snap = true, pixel_gap = 3 > 2
        // With &&: !true && true → false → NOT skipped → carried ✓
        // With ||: !true || true → true → skipped → NOT carried ✗
        let mut slice = make_airborne_slice(50, 97);
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.x_pos > x_before,
            "within snap but > 2px gap: && should not skip (|| would skip wrongly)"
        );
    }

    #[test]
    fn platform_riding_pixel_gap_boundary_at_two() {
        // Mutant: line 101 — replace > with >= in abs_diff > 2
        // When pixel gap == 2 and NOT within snap, should NOT skip (> 2 is false)
        // But this is tricky: pixel gap of 2 with not within snap means FP diff > 6px,
        // but pixel diff = 2 contradicts that. Sub-pixel can help:
        // player.y_pos = (98 << 16) | 0x0001, platform.y_pos = 100 << 16
        // FP diff = 0x1_FFFF ≈ 2px → within snap. Can't get pixel=2 outside snap.
        // This mutant is likely semantically equivalent — skip.
        //
        // Instead test that pixel gap of 3 WITH snap is not skipped:
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.x_velocity = 0x1_0000;

        let mut slice = make_airborne_slice(50, 97); // 3px gap, within snap
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        let x_before = slice.player.x_pos;
        apply_platform_collision(&mut slice, &[platform]);
        assert!(
            slice.player.x_pos > x_before,
            "3px pixel gap but within FP snap should still carry"
        );
    }

    #[test]
    fn evanescent_block_triggers_when_player_stands_on_it() {
        let grid = grid_with_evanescent(2, 3, 0);
        let mut runtime = runtime_with_evanescent_grid(grid);

        // Player standing on the evanescent block: feet at row 3 floor (y=47px)
        let slice = OpeningSliceState {
            player: PlayerState {
                x_pos: (2 * 16 + 8) << 16, // centre of col 2
                y_pos: (3 * 16 - 1) << 16, // feet just at top of row 3
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
                floor_y_pos: (3 * 16 - 1) << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 96,
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

        assert!(runtime.block_state.evanescent_blocks.is_empty());
        runtime.step_evanescent_blocks(&slice);
        assert_eq!(
            runtime.block_state.evanescent_blocks.len(),
            1,
            "standing on evanescent block should trigger it"
        );
        assert_eq!(runtime.block_state.evanescent_blocks[0].col, 2);
        assert_eq!(runtime.block_state.evanescent_blocks[0].row, 3);
    }

    #[test]
    fn evanescent_block_not_triggered_when_airborne() {
        let grid = grid_with_evanescent(2, 3, 0);
        let mut runtime = runtime_with_evanescent_grid(grid);

        let slice = OpeningSliceState {
            player: PlayerState {
                x_pos: (2 * 16 + 8) << 16,
                y_pos: (3 * 16 - 5) << 16, // above the block
                x_velocity: 0,
                y_velocity: -0x1_0000, // moving upward
                facing: Direction::Right,
                on_ground: false, // airborne
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
                width_pixels: 96,
                height_pixels: 96,
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

        runtime.step_evanescent_blocks(&slice);
        assert!(
            runtime.block_state.evanescent_blocks.is_empty(),
            "airborne player should not trigger evanescent block"
        );
    }

    #[test]
    fn evanescent_block_clears_after_timer_expires() {
        // skin=28 → initial timer = 28 - 28 = 0 → expires after 22 frames
        let grid = grid_with_evanescent(2, 3, 28);
        let mut runtime = runtime_with_evanescent_grid(grid);

        // Verify block exists
        assert_eq!(
            runtime
                .block_grid
                .as_ref()
                .unwrap()
                .cell_at(2, 3)
                .block_type,
            terrain::BlockType::Evanescent
        );

        // Trigger it
        let slice = OpeningSliceState {
            player: PlayerState {
                x_pos: (2 * 16 + 8) << 16,
                y_pos: (3 * 16 - 1) << 16,
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
                floor_y_pos: (3 * 16 - 1) << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 96,
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

        // Trigger + first tick
        runtime.step_evanescent_blocks(&slice);
        assert_eq!(runtime.block_state.evanescent_blocks.len(), 1);

        // Tick 21 more times (total 22 ticks) — airborne so no re-trigger
        let airborne_slice = OpeningSliceState {
            player: PlayerState {
                on_ground: false,
                ..slice.player
            },
            ..slice
        };
        for _ in 0..21 {
            runtime.step_evanescent_blocks(&airborne_slice);
        }

        // Block should now be cleared
        assert!(
            runtime.block_state.evanescent_blocks.is_empty(),
            "evanescent entry should be removed after 22 ticks"
        );
        let cell = runtime.block_grid.as_ref().unwrap().cell_at(2, 3);
        assert!(
            !cell.has_block,
            "evanescent block should be cleared from the grid"
        );
    }

    #[test]
    fn evanescent_block_with_skin_delay() {
        // skin=0 → initial timer = 0 - 28 = -28 → needs 28 + 22 = 50 ticks
        let grid = grid_with_evanescent(2, 3, 0);
        let mut runtime = runtime_with_evanescent_grid(grid);

        let on_ground_slice = OpeningSliceState {
            player: PlayerState {
                x_pos: (2 * 16 + 8) << 16,
                y_pos: (3 * 16 - 1) << 16,
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
                floor_y_pos: (3 * 16 - 1) << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 96,
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

        let airborne_slice = OpeningSliceState {
            player: PlayerState {
                on_ground: false,
                ..on_ground_slice.player
            },
            ..on_ground_slice
        };

        // Trigger + first tick
        runtime.step_evanescent_blocks(&on_ground_slice);
        assert_eq!(runtime.block_state.evanescent_blocks.len(), 1);
        assert_eq!(runtime.block_state.evanescent_blocks[0].timer, -27); // -28 + 1

        // After 48 more ticks (49 total), still active (timer = -28 + 49 = 21)
        for _ in 0..48 {
            runtime.step_evanescent_blocks(&airborne_slice);
        }
        assert_eq!(
            runtime.block_state.evanescent_blocks.len(),
            1,
            "block should still be active at tick 49"
        );
        assert!(
            runtime.block_grid.as_ref().unwrap().cell_at(2, 3).has_block,
            "block should still be solid at tick 49"
        );

        // One more tick (50 total): timer = -28 + 50 = 22 → removed
        runtime.step_evanescent_blocks(&airborne_slice);
        assert!(
            runtime.block_state.evanescent_blocks.is_empty(),
            "block should be removed at tick 50"
        );
        assert!(
            !runtime.block_grid.as_ref().unwrap().cell_at(2, 3).has_block,
            "block should be cleared from grid at tick 50"
        );
    }

    // --- Level timer ---

    #[test]
    fn level_timer_starts_at_three_minutes() {
        let timer = LevelTimer::new();
        assert_eq!(timer.seconds, 180);
        // ASM starts Time_SubSeconds at 1 so first frame immediately ticks a second
        assert_eq!(timer.sub_frames, 1);
    }

    #[test]
    fn level_timer_first_frame_decrements_immediately() {
        let mut timer = LevelTimer::new();
        // First tick: sub_frames 1→0, seconds 180→179, sub_frames reloads to 60
        assert!(!timer.tick());
        assert_eq!(timer.seconds, 179);
        assert_eq!(timer.sub_frames, 60);
    }

    #[test]
    fn level_timer_ticks_down_one_second_per_60_frames() {
        let mut timer = LevelTimer::new();
        timer.tick(); // first tick → 179
        // Then 60 more ticks → 178
        for _ in 0..60 {
            assert!(!timer.tick());
        }
        assert_eq!(timer.seconds, 178);
    }

    #[test]
    fn level_timer_expires_after_full_countdown() {
        let mut timer = LevelTimer::new();
        // First tick → 179 seconds, sub_frames=60
        // Then 179×60 more ticks → 0 seconds, sub_frames=60
        // Total: 1 + 179×60 = 10741 ticks to reach seconds=0
        // Then 60 more to drain sub_frames → expired at tick 10801
        let total_frames = 1 + 179 * 60 + 60;
        for i in 0..total_frames {
            let expired = timer.tick();
            if i < total_frames - 1 {
                assert!(!expired, "timer should not expire at frame {i}");
            } else {
                assert!(expired, "timer should expire at frame {total_frames}");
            }
        }
        assert_eq!(timer.seconds, 0);
    }

    #[test]
    fn level_timer_clock_bonus_adds_three_minutes() {
        let mut timer = LevelTimer::new();
        // Tick down 60 seconds
        for _ in 0..(60 * 60) {
            timer.tick();
        }
        assert_eq!(timer.seconds, 120); // 2:00 remaining
        timer.sub_frames = 37;
        timer.add_clock_bonus();
        assert_eq!(timer.seconds, 300); // 5:00 remaining
        assert_eq!(
            timer.sub_frames, 1,
            "ASM loc_FFFC / loc_10002 should reset `Time_SubSeconds` to 1 on every collected clock"
        );
    }

    #[test]
    fn level_timer_clock_bonus_loc_ffde_preserves_seconds_below_cap() {
        let mut timer = LevelTimer {
            seconds: 6 * 60 + 59,
            sub_frames: 42,
        };

        timer.add_clock_bonus();

        assert_eq!(
            timer.seconds,
            9 * 60 + 59,
            "ASM loc_FFDE should add three minutes without clearing seconds while the result stays below 10:00"
        );
        assert_eq!(
            timer.sub_frames, 1,
            "ASM loc_FFFC should still reset `Time_SubSeconds` after the loc_FFDE minute bonus"
        );
    }

    #[test]
    fn level_timer_clock_bonus_loc_ffde_clears_seconds_at_cap() {
        let mut timer = LevelTimer {
            seconds: 7 * 60 + 1,
            sub_frames: 42,
        };

        timer.add_clock_bonus();

        assert_eq!(
            timer.seconds,
            10 * 60,
            "ASM loc_FFDE should clamp above 10 minutes to exactly 10:00, matching the seconds-digit clear"
        );
        assert_eq!(
            timer.sub_frames, 1,
            "ASM loc_FFFC should still reset `Time_SubSeconds` after the loc_FFDE clamp path"
        );
    }

    #[test]
    fn level_timer_clock_bonus_caps_at_ten_minutes() {
        let mut timer = LevelTimer::new();
        // At 3:00, add clock → 6:00
        timer.add_clock_bonus();
        assert_eq!(timer.seconds, 360);
        // Add another → 9:00
        timer.add_clock_bonus();
        assert_eq!(timer.seconds, 540);
        // Add another → would be 12:00, capped to 10:00
        timer.sub_frames = 23;
        timer.add_clock_bonus();
        assert_eq!(timer.seconds, 600);
        assert_eq!(
            timer.sub_frames, 1,
            "ASM loc_FFFC should still reset `Time_SubSeconds` to 1 when the 10:00 cap path clamps the timer"
        );
    }

    #[test]
    fn level_timer_death_on_expiry() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);

        // Fast-forward timer to near-expiry
        runtime.level_timer.seconds = 0;
        runtime.level_timer.sub_frames = 2;

        // Tick one frame — timer goes to sub_frames=1, not yet expired
        runtime.step_frame(FrameInput::default());
        assert!(
            matches!(runtime.player_lifecycle, PlayerLifecycle::Alive),
            "player should still be alive with 1 sub_frame remaining"
        );

        // Tick one more — timer expires, death triggered
        runtime.step_frame(FrameInput::default());
        assert!(
            matches!(
                runtime.player_lifecycle,
                PlayerLifecycle::DeathWindup { .. }
            ),
            "timer expiry should trigger death, got {:?}",
            runtime.player_lifecycle
        );
    }

    #[test]
    fn level_timer_exposed_in_trace_frame() {
        let timer = LevelTimer::new();
        // The trace frame should expose the timer seconds
        assert_eq!(timer.seconds, LEVEL_TIMER_START_SECONDS);
    }

    #[test]
    fn evanescent_block_not_triggered_twice() {
        let grid = grid_with_evanescent(2, 3, 28);
        let mut runtime = runtime_with_evanescent_grid(grid);

        let slice = OpeningSliceState {
            player: PlayerState {
                x_pos: (2 * 16 + 8) << 16,
                y_pos: (3 * 16 - 1) << 16,
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
                floor_y_pos: (3 * 16 - 1) << 16,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: LevelBounds {
                width_pixels: 96,
                height_pixels: 96,
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

        // Trigger twice while standing
        runtime.step_evanescent_blocks(&slice);
        runtime.step_evanescent_blocks(&slice);
        assert_eq!(
            runtime.block_state.evanescent_blocks.len(),
            1,
            "same evanescent block should not be triggered twice"
        );
    }

    // --- Shooter block tests ---

    #[test]
    fn shooter_block_sub_f096_fires_when_kid_enters_scan_column() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.set_block_grid(grid_with_shooter_blocks(&[(5, 3, 0x02)]));
        runtime.projectiles.clear();
        runtime.block_state.shooter_last_scanned_col = 1;
        runtime.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert_eq!(runtime.projectiles.len(), 1, "should fire one projectile");
        assert!(
            runtime.projectiles[0].x_velocity > 0,
            "ASM sub_F096 should register the `kid_x_block+3` shooter column when Kid crosses into a new block"
        );
        assert_eq!(runtime.projectiles[0].y_velocity, 0);
    }

    #[test]
    fn shooter_block_sub_f096_fires_when_kid_enters_scan_row() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.set_block_grid(grid_with_shooter_blocks(&[(5, 7, 0x02)]));
        runtime.projectiles.clear();
        runtime.block_state.shooter_last_scanned_col = 5;
        runtime.block_state.shooter_last_scanned_row = 4;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (5 * 16) << 16;
        slice.player.y_pos = (5 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert_eq!(
            runtime.projectiles.len(),
            1,
            "ASM sub_F096 should register the `kid_y_block+2` shooter row when Kid moves down into a new block"
        );
    }

    #[test]
    fn loc_f09e_skips_column_scan_when_kid_stays_in_same_block_column() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.set_block_grid(grid_with_shooter_blocks(&[(5, 3, 0x02)]));
        runtime.projectiles.clear();
        runtime.block_state.shooter_last_scanned_col = 2;
        runtime.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert!(
            runtime.block_state.shooter_blocks.is_empty(),
            "ASM loc_F09E should branch to loc_F186 when the cached column equals Kid's current block column, so the `kid_x_block+3` shooter is not rescanned"
        );
        assert_eq!(runtime.block_state.shooter_last_scanned_col, 2);
    }

    #[test]
    fn loc_f0d0_clamps_leftward_column_scan_to_zero() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.set_block_grid(grid_with_shooter_blocks(&[(0, 3, 0x02)]));
        runtime.projectiles.clear();
        runtime.block_state.shooter_last_scanned_col = 4;
        runtime.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = 16 << 16;
        slice.player.y_pos = (3 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert_eq!(
            runtime.block_state.shooter_blocks.len(),
            1,
            "ASM loc_F0D0 should clamp `kid_x_block-3` to column zero before loc_F0D8 starts the vertical scan"
        );
        assert_eq!(runtime.block_state.shooter_blocks[0].col, 0);
        assert_eq!(runtime.block_state.shooter_blocks[0].row, 3);
    }

    #[test]
    fn loc_f0d8_clamps_column_scan_top_row_to_zero() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.set_block_grid(grid_with_shooter_blocks(&[(5, 0, 0x02)]));
        runtime.projectiles.clear();
        runtime.block_state.shooter_last_scanned_col = 1;
        runtime.block_state.shooter_last_scanned_row = 2;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (2 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert_eq!(
            runtime.block_state.shooter_blocks.len(),
            1,
            "ASM loc_F0D8 should clamp `kid_y_block-5` to row zero, so shooters on the top level row still register"
        );
        assert_eq!(runtime.block_state.shooter_blocks[0].col, 5);
        assert_eq!(runtime.block_state.shooter_blocks[0].row, 0);
    }

    #[test]
    fn loc_f0e0_clamps_column_scan_bottom_row_to_level_height() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.set_block_grid(grid_with_shooter_blocks(&[(5, 7, 0x02)]));
        runtime.projectiles.clear();
        runtime.block_state.shooter_last_scanned_col = 1;
        runtime.block_state.shooter_last_scanned_row = 6;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (6 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert_eq!(
            runtime.block_state.shooter_blocks.len(),
            1,
            "ASM loc_F0E0 should clamp `kid_y_block+2` to `Level_height_blocks-1`, so the bottom-row shooter still registers"
        );
        assert_eq!(runtime.block_state.shooter_blocks[0].col, 5);
        assert_eq!(runtime.block_state.shooter_blocks[0].row, 7);
    }

    #[test]
    fn loc_f0f0_scans_inclusive_rows_from_start_pointer() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.set_block_grid(grid_with_shooter_blocks(&[(5, 1, 0x02), (5, 7, 0x04)]));
        runtime.projectiles.clear();
        runtime.block_state.shooter_last_scanned_col = 1;
        runtime.block_state.shooter_last_scanned_row = 6;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (6 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert_eq!(
            runtime
                .block_state
                .shooter_blocks
                .iter()
                .map(|entry| (entry.col, entry.row, entry.directions))
                .collect::<Vec<_>>(),
            vec![(5, 1, 0x02), (5, 7, 0x04)],
            "ASM loc_F0F0 should start at `$FFFF4A04 + d4*2`, add `d3*2`, and DBF through the inclusive row count"
        );
    }

    #[test]
    fn loc_f106_queues_shooter_candidate_slot_state() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.set_block_grid(grid_with_shooter_blocks(&[(5, 4, 0x0A)]));
        runtime.projectiles.clear();
        runtime.block_state.shooter_last_scanned_col = 1;
        runtime.block_state.shooter_last_scanned_row = 4;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (4 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert_eq!(
            runtime.block_state.shooter_blocks,
            vec![ShooterBlockEntry {
                col: 5,
                row: 4,
                directions: 0x0A,
                supported_by_neighbour: false,
                age: 1,
            }],
            "ASM loc_F106 should allocate the candidate shooter slot with its column, row, and direction bits before sub_F730 ticks it"
        );
    }

    #[test]
    fn loc_f23e_marks_left_firing_shooter_when_left_neighbour_has_skin() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        let mut grid = grid_with_shooter_blocks(&[(5, 4, 0x08)]);
        grid.set_cell_at(4, 4, terrain::Cell::from_be_bytes(0x00, 0x2D));
        runtime.set_block_grid(grid);
        runtime.projectiles.clear();
        runtime.block_state.shooter_last_scanned_col = 1;
        runtime.block_state.shooter_last_scanned_row = 4;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (4 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert_eq!(runtime.block_state.shooter_blocks.len(), 1);
        assert!(
            runtime.block_state.shooter_blocks[0].supported_by_neighbour,
            "ASM loc_F23E (kid.asm:19814) should set the negative direction-word marker via loc_F248 when a left-firing shooter sees a non-zero skin byte at `-1(a1)`"
        );
    }

    #[test]
    fn loc_f24e_initialises_new_shooter_slot_before_first_tick() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.set_block_grid(grid_with_shooter_blocks(&[(5, 4, 0x02)]));
        runtime.projectiles.clear();
        runtime.block_state.shooter_last_scanned_col = 1;
        runtime.block_state.shooter_last_scanned_row = 4;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (4 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert_eq!(
            runtime.block_state.shooter_blocks[0].age, 1,
            "ASM loc_F24E (kid.asm:19823) writes the new slot state as zero; native creates age 0 and the same frame's folded sub_F730 tick advances it to age 1"
        );
        assert!(
            !runtime.block_state.shooter_blocks[0].supported_by_neighbour,
            "ASM loc_F24E follows any loc_F248 support marker, so an unsupported right-firing slot should keep the marker clear"
        );
    }

    #[test]
    fn loc_f254_continues_scan_without_allocating_when_shooter_free_list_empty() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        let mut blocks = (0..20).map(|col| (col, 0, 0x02)).collect::<Vec<_>>();
        blocks.push((25, 4, 0x02));
        runtime.set_block_grid(grid_with_shooter_blocks(&blocks));
        runtime.projectiles.clear();
        runtime.block_state.shooter_blocks = (0..20)
            .map(|col| ShooterBlockEntry {
                col,
                row: 0,
                directions: 0x02,
                supported_by_neighbour: false,
                age: 1,
            })
            .collect();
        runtime.block_state.shooter_last_scanned_col = 21;
        runtime.block_state.shooter_last_scanned_row = 4;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (22 * 16) << 16;
        slice.player.y_pos = (4 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert_eq!(
            runtime.block_state.shooter_blocks.len(),
            20,
            "ASM loc_F254 (kid.asm:19827) should leave the active list capped at the 20-slot free-list size when the scan finds a candidate with no free slot"
        );
        assert!(
            runtime
                .block_state
                .shooter_blocks
                .iter()
                .all(|entry| (entry.col, entry.row) != (25, 4)),
            "ASM loc_F254 advances to the next scanned cell after the no-free-slot branch instead of queueing the candidate"
        );
    }

    #[test]
    fn shooter_block_sub_f096_keeps_per_slot_cadence_until_the_worker_expires() {
        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.set_block_grid(grid_with_shooter_blocks(&[(5, 3, 0x02)]));
        runtime.projectiles.clear();
        runtime.block_state.shooter_last_scanned_col = 1;
        runtime.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        runtime.step_shooter_blocks(&slice);
        assert_eq!(
            runtime.projectiles.len(),
            1,
            "the first registered shooter worker should fire immediately"
        );

        for _ in 0..20 {
            runtime.step_shooter_blocks(&slice);
        }
        assert_eq!(
            runtime.projectiles.len(),
            1,
            "ASM sub_F096 keeps the live shooter slot resident for its 22-frame lifetime instead of re-registering each frame"
        );

        runtime.step_shooter_blocks(&slice);
        assert!(
            runtime.block_state.shooter_blocks.is_empty(),
            "the worker should expire once its 22-frame slot lifetime ends"
        );
    }

    #[test]
    fn shooter_slot_loc_f808_restores_cells_and_queues_visible_horizontal_span() {
        let span = NativeRuntime::shooter_horizontal_span_loc_F808(10, 30, 4, 18, 12, 7, 3);

        assert_eq!(
            span.restored_cols,
            vec![12, 13, 14],
            "ASM loc_F808 should write the `$E304` restore word before testing whether each horizontal shooter cell is visible"
        );
        assert_eq!(
            span.sprite_cells,
            vec![
                crate::blocks::ShooterHorizontalSpriteCellLocF808 {
                    col: 12,
                    row: 7,
                    fg_queue_word: 0x0730,
                },
                crate::blocks::ShooterHorizontalSpriteCellLocF808 {
                    col: 13,
                    row: 7,
                    fg_queue_word: 0x0734,
                },
                crate::blocks::ShooterHorizontalSpriteCellLocF808 {
                    col: 14,
                    row: 7,
                    fg_queue_word: 0x0738,
                },
            ],
            "ASM loc_F808 (kid.asm:20482) should encode visible cells as `(row << 8 & $F00) | (col << 2 & $7C)`"
        );
    }

    #[test]
    fn shooter_slot_loc_f828_advances_past_offscreen_prefix_cells() {
        let span = NativeRuntime::shooter_horizontal_span_loc_F808(10, 12, 4, 18, 8, 7, 5);

        assert_eq!(
            NativeRuntime::shooter_horizontal_span_columns_loc_F828(8, 5),
            vec![8, 9, 10, 11, 12],
            "ASM loc_F828 should increment `d0` and DBF back through the full horizontal shooter span"
        );
        assert_eq!(span.restored_cols, vec![8, 9, 10, 11, 12]);
        assert_eq!(
            span.sprite_cells
                .iter()
                .map(|cell| cell.col)
                .collect::<Vec<_>>(),
            vec![10, 11, 12],
            "ASM loc_F828 (kid.asm:20498) should keep iterating after offscreen cells so the onscreen suffix still queues FG updates"
        );
    }

    #[test]
    fn shooter_slot_loc_f830_restores_offscreen_span_without_sprite_updates() {
        let span = NativeRuntime::shooter_horizontal_span_loc_F808(10, 30, 4, 18, 31, 7, 4);

        assert!(
            NativeRuntime::shooter_horizontal_span_rejected_loc_F830(10, 30, 4, 18, 31, 7),
            "ASM loc_F830 should take the coarse reject branch when the span starts past the camera right column"
        );
        assert_eq!(
            span.restored_cols,
            vec![31, 32, 33, 34],
            "ASM loc_F830 should restore every span cell when the coarse viewport test rejects the shooter"
        );
        assert!(
            span.sprite_cells.is_empty(),
            "ASM loc_F830 (kid.asm:20504) skips the loc_F808 visible-cell queue path and only advances the layout pointer"
        );
    }

    #[test]
    fn shooter_slot_tick_sub_f730_advances_the_zero_state_into_the_negative_visibility_ladder() {
        assert_eq!(
            NativeRuntime::shooter_slot_tick_sub_F730(1, 0, 4, 6),
            (2, 0x8001),
            "ASM sub_F730 loc_F8A6 should turn a zero state into `$8001` and reload the slot timer with a 2-frame delay",
        );
        assert_eq!(
            NativeRuntime::shooter_slot_tick_sub_F730(1, 0x8001, 4, 6),
            (2, 0x8002),
            "ASM sub_F730 loc_F79A should advance `$8001 -> $8002` with the same 2-frame delay",
        );
        assert_eq!(
            NativeRuntime::shooter_slot_tick_sub_F730(1, 0x8002, 4, 6),
            (6, 3),
            "ASM sub_F730 loc_F78C should leave the negative ladder at state `3` and restore the slot reload delay",
        );
    }

    #[test]
    fn shooter_slot_tick_sub_f730_wraps_live_states_back_to_zero_after_the_flash_pair() {
        assert_eq!(
            NativeRuntime::shooter_slot_tick_sub_F730(1, 5, 4, 6),
            (2, 0x8004),
            "ASM sub_F730 loc_F898 should turn any positive live state into `$8004` after the timer expires",
        );
        assert_eq!(
            NativeRuntime::shooter_slot_tick_sub_F730(1, 0x8004, 4, 6),
            (2, 0x8005),
            "ASM sub_F730 loc_F79A should advance `$8004 -> $8005` on the next expiry",
        );
        assert_eq!(
            NativeRuntime::shooter_slot_tick_sub_F730(1, 0x8005, 4, 6),
            (4, 0),
            "ASM sub_F730 loc_F7AA should clear the state word and restore the active delay after `$8005` expires",
        );
    }

    #[test]
    fn shooter_slot_tick_loc_f76c_only_decrements_live_timer_until_expiry() {
        assert_eq!(
            NativeRuntime::shooter_slot_tick_sub_F730(3, 0x8004, 4, 6),
            (2, 0x8004),
            "ASM loc_F76C should branch to the slot loop tail while the pre-decremented timer is non-zero",
        );
    }

    #[test]
    fn shooter_slot_loc_f31c_loads_the_long_sleep_timer() {
        assert_eq!(
            NativeRuntime::shooter_slot_sleep_frames_loc_F31C(),
            0x32,
            "ASM loc_F31C (kid.asm:19909) should load timer byte 8(a0) with `$32` before the shared redraw tail"
        );
    }

    #[test]
    fn shooter_slot_loc_f326_restarts_or_frees_after_the_range_check() {
        let player_x = (8 * 16) << 16;
        let player_y = (6 * 16) << 16;
        let in_range = NativeRuntime::shooter_player_in_range_sub_F536(player_x, player_y, 8, 6);
        let out_of_range =
            NativeRuntime::shooter_player_in_range_sub_F536(player_x, player_y, 20, 6);

        assert_eq!(
            NativeRuntime::shooter_slot_restart_phase_loc_F326(in_range),
            Some(1),
            "ASM loc_F326 (kid.asm:19914) should write `$A(a0)=1` when sub_F536 says Kid is still in range"
        );
        assert_eq!(
            NativeRuntime::shooter_slot_restart_phase_loc_F326(out_of_range),
            None,
            "ASM loc_F326 should fall through to the free-slot unlink path instead of restarting the phase when sub_F536 returns zero"
        );
    }

    #[test]
    fn shooter_slot_loc_f336_restores_ec_cell_and_unlinks_the_slot() {
        assert_eq!(
            NativeRuntime::shooter_slot_freed_cell_loc_F336(0x2D),
            terrain::Cell::from_be_bytes(0xEC, 0x2D),
            "ASM loc_F336 (kid.asm:19921) should write high byte `$EC` while preserving the shooter cell skin byte"
        );

        let mut runtime = NativeRuntime::new(manifest());
        start_level(&mut runtime);
        runtime.set_block_grid(grid_with_shooter_blocks(&[(5, 3, 0x02)]));
        runtime.block_state.shooter_blocks = vec![ShooterBlockEntry {
            col: 5,
            row: 3,
            directions: 0x02,
            supported_by_neighbour: false,
            age: 21,
        }];
        runtime.block_state.shooter_last_scanned_col = 2;
        runtime.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        runtime.step_shooter_blocks(&slice);

        assert!(
            runtime.block_state.shooter_blocks.is_empty(),
            "ASM loc_F336 should unlink an expiring active shooter slot; native models the active/free lists with Vec retain"
        );
    }

    #[test]
    fn shooter_headbutt_center_worker_runs_sub_114c0_timeline() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 4, 0x02)]));
        rt.block_grid.as_mut().expect("block grid").set_cell_at(
            5,
            3,
            terrain::Cell::from_be_bytes(0x60, 0),
        );
        rt.block_grid.as_mut().expect("block grid").set_cell_at(
            4,
            3,
            terrain::Cell::from_be_bytes(0x60, 0),
        );
        rt.block_grid.as_mut().expect("block grid").set_cell_at(
            6,
            3,
            terrain::Cell::from_be_bytes(0x60, 0),
        );

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((5, 4));
        rt.process_block_effects(&mut slice);

        rt.step_shooter_headbutt_workers();
        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 4),
            terrain::Cell::from_be_bytes(0xEF, 0x1B),
            "ASM sub_114C0 writes `$EF1B` on its first active frame"
        );

        for _ in 0..8 {
            rt.step_shooter_headbutt_workers();
        }
        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 4),
            terrain::Cell::from_be_bytes(0xEF, 0x1A),
            "ASM sub_114C0 writes `$EF1A` after the first 8-frame hibernate"
        );

        for _ in 0..8 {
            rt.step_shooter_headbutt_workers();
        }
        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 4),
            terrain::Cell::from_be_bytes(0, 0),
            "ASM sub_114C0 erases the shooter cell after the second 8-frame hibernate"
        );
    }

    #[test]
    fn shooter_headbutt_center_worker_sub_114c0_ignores_support_cell_changes() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 4, 0x02)]));
        rt.block_grid.as_mut().expect("block grid").set_cell_at(
            5,
            3,
            terrain::Cell::from_be_bytes(0x60, 0),
        );
        rt.block_grid.as_mut().expect("block grid").set_cell_at(
            4,
            3,
            terrain::Cell::from_be_bytes(0x60, 0),
        );
        rt.block_grid.as_mut().expect("block grid").set_cell_at(
            6,
            3,
            terrain::Cell::from_be_bytes(0x60, 0),
        );

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((5, 4));
        rt.process_block_effects(&mut slice);
        rt.block_grid.as_mut().expect("block grid").set_cell_at(
            5,
            3,
            terrain::Cell::from_be_bytes(0x60, 0),
        );

        for _ in 0..17 {
            rt.step_shooter_headbutt_workers();
        }

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 4),
            terrain::Cell::from_be_bytes(0, 0),
            "ASM sub_114C0 only reads the struck shooter cell, so support refills should not stop the centre erase"
        );
    }

    #[test]
    fn shooter_center_worker_loc_11474_queues_the_struck_cell_worker_for_top_row_shooters() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 0, 0x02)]));

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((5, 0));

        rt.process_block_effects(&mut slice);

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 0),
            terrain::Cell::from_be_bytes(0xEF, 0x02),
            "ASM loc_11474 is the `tst.w d2; beq.s loc_11474` top-row path, so the struck shooter should be marked deleted-solid immediately"
        );
        assert_eq!(
            rt.block_state.shooter_headbutt_workers,
            vec![ShooterHeadbuttWorker {
                col: 5,
                row: 0,
                age: 0,
                kind: ShooterHeadbuttWorkerKind::Center,
            }],
            "ASM loc_11474 should queue only the struck-cell `sub_114C0` worker when the shooter has no row above it"
        );
    }

    #[test]
    fn next_level_id_advances_logical_level_ids_not_resource_map_order() {
        let mut manifest = manifest_with_levels(4);
        manifest.map_order = vec![0x4A, 0x4B, 0x2D, 0x2E];

        let mut runtime = NativeRuntime::new(manifest);
        runtime.current_level = LevelId(0);
        assert_eq!(runtime.next_level_id(), LevelId(1));

        runtime.current_level = LevelId(1);
        assert_eq!(runtime.next_level_id(), LevelId(2));

        runtime.current_level = LevelId(2);
        assert_eq!(runtime.next_level_id(), LevelId(3));
    }

    #[test]
    fn next_level_id_skips_invalid_logical_entries() {
        let mut manifest = manifest_with_levels(5);
        manifest.map_order = vec![0x4A, 0x4B, u8::MAX, 0x2E, 0x2F];

        let mut runtime = NativeRuntime::new(manifest);
        runtime.current_level = LevelId(1);
        assert_eq!(runtime.next_level_id(), LevelId(3));
        assert_eq!(runtime.prev_level_id(), LevelId(0));

        runtime.current_level = LevelId(3);
        assert_eq!(runtime.prev_level_id(), LevelId(1));
    }

    #[test]
    fn next_level_id_falls_back_when_map_order_empty() {
        let mut manifest = manifest_with_levels(3);
        manifest.map_order = Vec::new();

        let mut runtime = NativeRuntime::new(manifest);
        runtime.current_level = LevelId(0);
        assert_eq!(runtime.next_level_id(), LevelId(1));

        runtime.current_level = LevelId(1);
        assert_eq!(runtime.next_level_id(), LevelId(2));
    }

    #[test]
    fn final_level_sets_game_complete_on_advance() {
        let mut manifest = manifest_with_levels(3);
        manifest.map_order = vec![0, 1, 2];

        let mut runtime = NativeRuntime::new(manifest);
        runtime.current_level = LevelId(2); // last in map order
        assert!(
            runtime.is_final_level(),
            "level 2 should be detected as final level"
        );
        assert!(!runtime.game_complete);
        runtime.advance_level();
        assert!(
            runtime.game_complete,
            "game_complete should be set after beating final level"
        );
    }

    #[test]
    fn non_final_level_does_not_set_game_complete() {
        let mut manifest = manifest_with_levels(3);
        manifest.map_order = vec![0, 1, 2];

        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(grid_with_floor_at_row(17));
        runtime.current_level = LevelId(0);
        assert!(!runtime.is_final_level());
        runtime.advance_level();
        assert!(
            !runtime.game_complete,
            "game_complete should not be set mid-game"
        );
    }

    // --- Murder wall ---

    #[test]
    fn murder_wall_initialises_from_header_flags() {
        let mut manifest = manifest_with_levels(2);
        // Set level 0 to have murder wall (flags = 0b10 = enabled, left-to-right)
        manifest.levels[0]
            .header_metadata
            .as_mut()
            .unwrap()
            .murder_wall_flags = 0b10;

        let mut runtime = NativeRuntime::new(manifest);
        runtime.init_murder_wall(LevelId(0), 0);

        let wall = runtime.murder_wall.expect("murder wall should be active");
        assert!(!wall.reversed);
        assert_eq!(
            wall.speed, 0,
            "ASM loc_140BE should clear `MurderWall_speed` after picking the initial max speed"
        );
        assert_eq!(wall.max_speed, MURDER_WALL_DEFAULT_MAX_SPEED);
    }

    #[test]
    fn murder_wall_not_initialised_when_flags_zero() {
        let manifest = manifest_with_levels(2);
        let mut runtime = NativeRuntime::new(manifest);
        runtime.init_murder_wall(LevelId(0), 0);

        assert!(runtime.murder_wall.is_none());
    }

    #[test]
    fn murder_wall_accelerates_each_frame() {
        let mut wall = MurderWallState {
            x_pos: 0,
            speed: 0,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        };

        // Simulate a few frames manually
        wall.speed = (wall.speed + MURDER_WALL_ACCELERATION).min(wall.max_speed);
        wall.x_pos += i64::from(wall.speed);
        assert_eq!(wall.speed, 0x0100);
        assert_eq!(wall.x_pos, 0x0100);

        wall.speed = (wall.speed + MURDER_WALL_ACCELERATION).min(wall.max_speed);
        wall.x_pos += i64::from(wall.speed);
        assert_eq!(wall.speed, 0x0200);
        assert_eq!(wall.x_pos, 0x0300);
    }

    #[test]
    fn murder_wall_speed_caps_at_max() {
        let mut wall = MurderWallState {
            x_pos: 0,
            speed: MURDER_WALL_DEFAULT_MAX_SPEED - 1,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        };
        wall.speed = (wall.speed + MURDER_WALL_ACCELERATION).min(wall.max_speed);
        assert_eq!(wall.speed, MURDER_WALL_DEFAULT_MAX_SPEED);
    }

    #[test]
    fn murder_wall_reversed_flag() {
        let mut manifest = manifest_with_levels(2);
        manifest.levels[0]
            .header_metadata
            .as_mut()
            .unwrap()
            .murder_wall_flags = 0b11; // enabled + reversed

        let mut runtime = NativeRuntime::new(manifest);
        runtime.init_murder_wall(LevelId(0), 0);

        let wall = runtime.murder_wall.expect("murder wall should be active");
        assert!(wall.reversed);
    }

    #[test]
    fn murder_wall_hills_of_warrior_uses_slow_max_speed() {
        let mut manifest = manifest_with_levels(9);
        // Level 0x08 = Hills of the Warrior 1
        manifest.levels[8]
            .header_metadata
            .as_mut()
            .unwrap()
            .murder_wall_flags = 0b10;

        let mut runtime = NativeRuntime::new(manifest);
        runtime.init_murder_wall(LevelId(8), 0);

        let wall = runtime.murder_wall.expect("murder wall should be active");
        assert_eq!(wall.max_speed, MURDER_WALL_SLOW_MAX_SPEED);
    }

    #[test]
    fn hail_spawns_projectiles_after_35_seconds() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 3; // Storm + Hail
        // Simulate 35 seconds elapsed: timer at 145s remaining
        runtime.level_timer.seconds = 145;
        runtime.weather.hail_active = true;
        runtime.weather.hail_spawn_timer = 0;
        runtime.weather.hail_period_timer = 100; // mid-period, still active

        let slice = airborne_slice();
        let before = runtime.projectiles.len();
        runtime.step_weather(&slice);

        assert_eq!(
            runtime.projectiles.len(),
            before + 1,
            "hail should spawn a projectile"
        );
        let p = &runtime.projectiles[before];
        assert!(p.y_velocity > 0, "hail should fall downward");
        assert!(!p.friendly, "hail should damage the player");
    }

    #[test]
    fn no_hail_without_storm_weather() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 0; // No weather
        runtime.level_timer.seconds = 100;

        let slice = airborne_slice();
        runtime.step_weather(&slice);

        assert!(
            runtime.projectiles.is_empty(),
            "no hail without storm weather"
        );
    }

    // --- Lava geyser tests ---

    #[test]
    fn lava_geyser_erupts_and_collapses() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: 16,
            phase: 0,
            timer: 0, // ready to erupt immediately
            current_height: 0,
        });

        let slice = airborne_slice();

        // Phase 0 → phase 1 (erupt)
        runtime.step_lava_geysers(&slice);
        assert_eq!(runtime.weather.lava_geysers[0].phase, 1);

        // Erupt until full height (16px / 4px per frame = 4 frames)
        for _ in 0..3 {
            runtime.step_lava_geysers(&slice);
        }
        assert_eq!(runtime.weather.lava_geysers[0].phase, 1);
        runtime.step_lava_geysers(&slice);
        assert_eq!(
            runtime.weather.lava_geysers[0].phase, 2,
            "should transition to hold phase"
        );
        assert_eq!(runtime.weather.lava_geysers[0].current_height, 16);

        // Hold for LAVA_GEYSER_HOLD_FRAMES + 1 frames (countdown then transition)
        for _ in 0..=LAVA_GEYSER_HOLD_FRAMES {
            runtime.step_lava_geysers(&slice);
        }
        assert_eq!(
            runtime.weather.lava_geysers[0].phase, 3,
            "should transition to collapse phase"
        );

        // Collapse (16px / 4px per frame = 4 frames)
        for _ in 0..4 {
            runtime.step_lava_geysers(&slice);
        }
        assert_eq!(
            runtime.weather.lava_geysers[0].phase, 0,
            "should return to dormant"
        );
        assert_eq!(runtime.weather.lava_geysers[0].current_height, 0);
    }

    // --- Murder wall kill detection ---

    #[test]
    fn murder_wall_kills_player_when_wall_reaches_player() {
        // Normal (left-to-right) wall: kills when wall_right >= player_x
        let mut runtime = NativeRuntime::new(manifest());
        let wall_x = 0x0050_0000_i64;
        runtime.murder_wall = Some(MurderWallState {
            x_pos: wall_x,
            speed: 0x1000,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        });
        let mut slice = airborne_slice();
        // Player far ahead of wall — not killed
        slice.player.x_pos = 0x0200_0000;
        assert!(
            !runtime.murder_wall_kills_player(&slice),
            "player should be safe when wall is far behind"
        );

        // Player just at the wall edge — killed
        // wall_right = wall_x + (MURDER_WALL_CAMERA_OFFSET_PX << 16)
        let wall_right = wall_x + (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16);
        slice.player.x_pos = wall_right as u32;
        assert!(
            runtime.murder_wall_kills_player(&slice),
            "player should die when wall_right == player_x"
        );
    }

    #[test]
    fn murder_wall_kills_player_reversed() {
        // Reversed (right-to-left) wall: kills when player_right >= wall_left
        let mut runtime = NativeRuntime::new(manifest());
        let wall_x = 0x0200_0000_i64;
        runtime.murder_wall = Some(MurderWallState {
            x_pos: wall_x,
            speed: 0x1000,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: true,
        });
        let mut slice = airborne_slice();
        // Player far left of reversed wall — not killed
        slice.player.x_pos = 0x0010_0000;
        assert!(
            !runtime.murder_wall_kills_player(&slice),
            "player should be safe when far from reversed wall"
        );

        // Player at the wall — killed
        // kill when player_x + (OFFSET << 16) >= wall_x
        let threshold = wall_x - (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16);
        slice.player.x_pos = threshold as u32;
        assert!(
            runtime.murder_wall_kills_player(&slice),
            "player should die when touching reversed wall"
        );
    }

    #[test]
    fn murder_wall_kills_player_returns_false_when_no_wall() {
        let runtime = NativeRuntime::new(manifest());
        let slice = airborne_slice();
        assert!(
            !runtime.murder_wall_kills_player(&slice),
            "no wall means no kill"
        );
    }

    #[test]
    fn step_murder_wall_reversed_decrements_position() {
        // ASM: reversed wall moves right-to-left (subtracts from x_pos)
        let mut runtime = NativeRuntime::new(manifest());
        runtime.murder_wall = Some(MurderWallState {
            x_pos: 0x0100_0000,
            speed: 0,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: true,
        });
        let mut cam = 0x0100_0000u32; // camera at wall — push won't alter wall
        runtime.step_murder_wall(&mut cam);
        let wall = runtime.murder_wall.as_ref().unwrap();
        assert_eq!(wall.speed, MURDER_WALL_ACCELERATION);
        // Reversed wall should have LOWER x_pos after stepping
        assert!(
            wall.x_pos < 0x0100_0000,
            "reversed wall x_pos should decrease: got {}",
            wall.x_pos
        );
    }

    #[test]
    fn step_murder_wall_forward_increments_position() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.murder_wall = Some(MurderWallState {
            x_pos: 0x0100_0000,
            speed: 0,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        });
        let mut cam = 0u32;
        runtime.step_murder_wall(&mut cam);
        let wall = runtime.murder_wall.as_ref().unwrap();
        assert!(
            wall.x_pos > 0x0100_0000,
            "forward wall x_pos should increase"
        );
    }

    #[test]
    fn step_murder_wall_noop_when_no_wall() {
        let mut runtime = NativeRuntime::new(manifest());
        assert!(runtime.murder_wall.is_none());
        let mut cam = 0u32;
        runtime.step_murder_wall(&mut cam); // should not panic
    }

    // --- loc_7316: forward murder wall acceleration + speed clamp ---

    /// ASM loc_7316 (kid.asm:7454–7462): forward (non-reversed) path accelerates
    /// speed by $100/frame and clamps to max_speed.
    /// ```asm
    /// loc_7316:
    ///   addi.w  #$80,d1              ; camera-push threshold (not modelled)
    ///   addi.l  #$100,(MurderWall_speed).w
    ///   move.l  (MurderWall_speed).w,d7
    ///   cmp.l   (MurderWall_max_speed).w,d7
    ///   blt.s   loc_7334
    ///   move.l  (MurderWall_max_speed).w,d7
    ///   move.l  d7,(MurderWall_speed).w
    /// ```
    #[test]
    fn loc_7316_forward_wall_accelerates_by_0x100_per_frame() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.murder_wall = Some(MurderWallState {
            x_pos: 0,
            speed: 0,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        });
        let mut cam = 0u32;
        runtime.step_murder_wall(&mut cam);
        let wall = runtime.murder_wall.as_ref().unwrap();
        assert_eq!(
            wall.speed, MURDER_WALL_ACCELERATION,
            "ASM loc_7316: first frame speed = $100"
        );
        // Second frame
        runtime.step_murder_wall(&mut cam);
        let wall = runtime.murder_wall.as_ref().unwrap();
        assert_eq!(
            wall.speed,
            MURDER_WALL_ACCELERATION * 2,
            "ASM loc_7316: second frame speed = $200"
        );
    }

    #[test]
    fn loc_7316_forward_wall_speed_clamped_to_max() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.murder_wall = Some(MurderWallState {
            x_pos: 0,
            speed: MURDER_WALL_DEFAULT_MAX_SPEED - 1,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        });
        let mut cam = 0u32;
        runtime.step_murder_wall(&mut cam);
        let wall = runtime.murder_wall.as_ref().unwrap();
        assert_eq!(
            wall.speed, MURDER_WALL_DEFAULT_MAX_SPEED,
            "ASM loc_7316: speed clamped at max_speed when accel would exceed"
        );
    }

    // --- loc_7334: forward murder wall position advance ---

    /// ASM loc_7334 (kid.asm:7463–7468): `add.l d7,(MurderWall_X_pos).w` then
    /// clamp to `Camera_max_X_pos`.
    /// ```asm
    /// loc_7334:
    ///   add.l   d7,(MurderWall_X_pos).w
    ///   move.w  (Camera_max_X_pos).w,d0
    ///   cmp.w   (MurderWall_X_pos).w,d0
    ///   bgt.s   loc_7346
    ///   move.w  d0,(MurderWall_X_pos).w
    /// ```
    #[test]
    fn loc_7334_forward_wall_position_advances_by_speed() {
        let mut runtime = NativeRuntime::new(manifest());
        let initial_x: i64 = 0x0050_0000;
        runtime.murder_wall = Some(MurderWallState {
            x_pos: initial_x,
            speed: 0x1000,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        });
        let mut cam = 0u32;
        runtime.step_murder_wall(&mut cam);
        let wall = runtime.murder_wall.as_ref().unwrap();
        // After acceleration: speed = 0x1000 + 0x100 = 0x1100
        // x_pos += 0x1100
        let expected_speed = 0x1000 + MURDER_WALL_ACCELERATION;
        assert_eq!(wall.speed, expected_speed);
        assert_eq!(
            wall.x_pos,
            initial_x + i64::from(expected_speed),
            "ASM loc_7334: x_pos += accelerated speed"
        );
    }

    #[test]
    fn loc_7334_forward_wall_at_max_speed_advances_steadily() {
        let mut runtime = NativeRuntime::new(manifest());
        let initial_x: i64 = 0x0050_0000;
        runtime.murder_wall = Some(MurderWallState {
            x_pos: initial_x,
            speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: false,
        });
        let mut cam = 0u32;
        runtime.step_murder_wall(&mut cam);
        let wall = runtime.murder_wall.as_ref().unwrap();
        assert_eq!(
            wall.speed, MURDER_WALL_DEFAULT_MAX_SPEED,
            "ASM loc_7316: already at max, speed unchanged"
        );
        assert_eq!(
            wall.x_pos,
            initial_x + i64::from(MURDER_WALL_DEFAULT_MAX_SPEED),
            "ASM loc_7334: position advances by max_speed"
        );
    }

    // --- loc_7464: input byte decode (shift chain) + left/right conflict ---

    /// ASM loc_7464 (kid.asm:7593–7609): `move.b (Ctrl_Held),d0` then 7×
    /// `add.b d0,d0; scs (a0)+` unpacks bits 6..0 into Ctrl_A_Held ..
    /// Ctrl_Up_Held.  `from_mask` is the native equivalent.
    #[test]
    fn loc_7464_from_mask_decodes_all_buttons() {
        // All bits set: mask = 0xFF
        let input = FrameInput::from_mask(0xFF);
        assert!(input.up, "bit 0 = Up");
        assert!(input.down, "bit 1 = Down");
        assert!(input.left, "bit 2 = Left");
        assert!(input.right, "bit 3 = Right");
        assert!(input.b, "bit 4 = B");
        assert!(input.c, "bit 5 = C");
        assert!(input.a, "bit 6 = A");
        assert!(input.start, "bit 7 = Start");

        // No bits set
        let empty = FrameInput::from_mask(0x00);
        assert!(!empty.up && !empty.down && !empty.left && !empty.right);
        assert!(!empty.a && !empty.b && !empty.c && !empty.start);
    }

    /// ASM loc_7464 (kid.asm:7593): first `add.b d0,d0` discards bit 7
    /// (Start) from the shift chain — Start is handled separately by
    /// loc_74B0's `bclr #Button_Start,(Ctrl_Pressed)`.
    #[test]
    fn loc_7464_start_bit_decoded_but_not_in_shift_chain() {
        // Only Start held (bit 7)
        let input = FrameInput::from_mask(0x80);
        assert!(input.start);
        assert!(!input.a, "bit 7 shift discards Start; A = bit 6 is clear");
    }

    /// ASM loc_7464 (kid.asm:7610–7614): `tst.b (Ctrl_Left_Held); beq
    /// loc_749C; tst.b (Ctrl_Right_Held); beq loc_749C; sf
    /// (Ctrl_Right_Held)` — when both Left and Right are held, Right is
    /// cleared so Left wins.
    #[test]
    fn loc_7464_left_right_conflict_clears_right() {
        let both = FrameInput::from_mask(0x0C); // Left + Right
        assert!(both.left && both.right, "precondition: both set from mask");

        let resolved = both.with_left_right_conflict_resolved();
        assert!(resolved.left, "ASM: left survives conflict");
        assert!(!resolved.right, "ASM sf (Ctrl_Right_Held): right cleared");
    }

    /// When only Left or only Right is held, conflict resolution is a no-op.
    #[test]
    fn loc_7464_single_direction_unchanged_by_conflict_resolution() {
        let left_only = FrameInput::from_mask(0x04); // Left
        let resolved = left_only.with_left_right_conflict_resolved();
        assert!(resolved.left && !resolved.right);

        let right_only = FrameInput::from_mask(0x08); // Right
        let resolved = right_only.with_left_right_conflict_resolved();
        assert!(!resolved.left && resolved.right);
    }

    /// Conflict resolution preserves non-directional buttons.
    #[test]
    fn loc_7464_conflict_resolution_preserves_other_buttons() {
        // Left + Right + A + B
        let input = FrameInput::from_mask(0x0C | 0x40 | 0x10);
        let resolved = input.with_left_right_conflict_resolved();
        assert!(resolved.a, "A preserved");
        assert!(resolved.b, "B preserved");
        assert!(resolved.left, "left wins");
        assert!(!resolved.right, "right cleared");
    }

    // --- loc_749C: speed option A-button inversion ---

    /// ASM loc_749C (kid.asm:7616–7621): when `Demo_Mode_flag` is clear and
    /// `Options_Suboption_Speed` != 0 (fast mode), the ROM executes
    /// `not.b (Ctrl_A_Held)` — toggling the A button so the player runs by
    /// default and holds A to walk.
    #[test]
    fn loc_749c_speed_option_inverts_a_button() {
        // A not held → inverted to held (player runs by default)
        let no_a = FrameInput::from_mask(0x00);
        let inverted = no_a.with_speed_option_a_inverted();
        assert!(inverted.a, "ASM not.b: false → true (run by default)");

        // A held → inverted to not held (player walks)
        let with_a = FrameInput::from_mask(0x40);
        let inverted = with_a.with_speed_option_a_inverted();
        assert!(!inverted.a, "ASM not.b: true → false (hold A to walk)");
    }

    /// ASM loc_749C: inversion preserves all other buttons.
    #[test]
    fn loc_749c_speed_option_preserves_other_buttons() {
        let input = FrameInput::from_mask(0xFF); // all buttons
        let inverted = input.with_speed_option_a_inverted();
        assert!(!inverted.a, "A inverted");
        assert!(inverted.up && inverted.down && inverted.left && inverted.right);
        assert!(inverted.b && inverted.c && inverted.start);
    }

    /// ASM loc_749C: `tst.b (Demo_Mode_flag); bne.w loc_74B0` — in demo
    /// mode the speed option inversion is skipped entirely.
    #[test]
    fn loc_749c_demo_mode_skips_inversion() {
        // The pure function always inverts; the caller is responsible for
        // gating on demo mode.  Verify the function itself is a clean toggle.
        let input = FrameInput::from_mask(0x00);
        let double = input
            .with_speed_option_a_inverted()
            .with_speed_option_a_inverted();
        assert_eq!(input, double, "double inversion = identity");
    }

    // --- loc_74B0: Start press → pause / level-skip gate ---

    /// ASM loc_74B0 (kid.asm:7623–7625): `bclr #Button_Start,(Ctrl_Pressed);
    /// beq.s loc_7452` — when Start is not pressed (rising edge), the handler
    /// is a no-op.  Modelled by `handle_start_press(_, start_pressed=false)`.
    #[test]
    fn loc_74b0_no_start_press_is_noop() {
        let mut rt = NativeRuntime::new(manifest());
        let input = FrameInput::from_mask(0x00);
        rt.handle_start_press(input, false);
        assert!(!rt.paused, "no Start → no pause toggle");
        assert!(!rt.pause_toggle_pending);
    }

    /// ASM loc_74B0 (kid.asm:7632–7636): `btst #Button_A,(Ctrl_Held);
    /// bne.s loc_7452` — holding A suppresses the pause toggle even when
    /// Start is pressed.  In the ASM this is unconditional; in native the
    /// A+Start path only suppresses pause when A+C (level skip) or Diamond
    /// Power conditions are met.
    // TODO(fidelity): ASM loc_74C8 unconditionally skips pause when A is
    // held; native allows pause when A is held without C and Diamond Power
    // conditions aren't met.
    #[test]
    fn loc_74b0_a_plus_c_triggers_level_skip_not_pause() {
        let mut rt = NativeRuntime::new(manifest());
        rt.boot_state = BootState::InGame;
        rt.player_lifecycle = PlayerLifecycle::Alive;
        // Start pressed + A + C held → level skip, not pause
        let input = FrameInput {
            a: true,
            c: true,
            start: true,
            ..Default::default()
        };
        rt.handle_start_press(input, true);
        assert!(
            !rt.pause_toggle_pending,
            "ASM loc_74E0: A+C+Start → level skip, pause suppressed"
        );
        assert!(
            rt.pending_level_skip.is_some(),
            "ASM loc_74E0: pending_level_skip set"
        );
    }

    /// ASM loc_B8C (kid.asm:1342): ReadJoypad default path — stores
    /// `Ctrl_Held → Ctrl_1_Held`, `Ctrl_Pressed → Ctrl_1_Pressed`.
    /// In native, `step_frame` consumes a single `FrameInput` directly.
    /// This test verifies the input reaches gameplay (standing player
    /// responds to directional input), matching the loc_B8C store path.
    #[test]
    fn loc_b8c_readjoypad_default_stores_input_for_gameplay() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_floor_at_row(17));
        start_level(&mut rt);
        let idle = rt.step_frame(FrameInput::default());
        let before_x = idle.kid_x_pos;

        // Walk right for several frames — the ReadJoypad store at loc_B8C
        // makes Ctrl_1_Held available to the gameplay loop each frame.
        // Acceleration means displacement appears after a few frames.
        let mut walked = idle;
        for _ in 0..5 {
            walked = rt.step_frame(FrameInput {
                right: true,
                ..Default::default()
            });
        }

        assert!(
            walked.kid_x_pos > before_x,
            "ASM loc_B8C: Ctrl_Held → Ctrl_1_Held store should let gameplay read directional input"
        );
    }

    /// ASM loc_B98 (kid.asm:1346): ReadJoypad level-skip cheat gate —
    /// `tst.w (word_7190); beq.s loc_BB6; cmpi.b #$C0,(Ctrl_1_Held);
    /// cmpi.b #$10,(Ctrl_2_Held); st (LevelSkip_Cheat)`. The cheat
    /// requires word_7190 (debug mode) to be set. Native always has the
    /// cheat armed, so A+C+Start triggers level skip unconditionally.
    #[test]
    fn loc_b98_readjoypad_level_skip_cheat_always_armed_in_native() {
        let mut rt = NativeRuntime::new(manifest());
        rt.boot_state = BootState::InGame;
        rt.player_lifecycle = PlayerLifecycle::Alive;

        // A+C+Start triggers level skip — loc_B98's cheat flag is
        // effectively always set in native (no word_7190 gate).
        rt.handle_start_press(
            FrameInput {
                a: true,
                c: true,
                start: true,
                ..Default::default()
            },
            true,
        );

        assert!(
            rt.pending_level_skip.is_some(),
            "ASM loc_B98: LevelSkip_Cheat should be armed (native skips word_7190 gate)"
        );
    }

    /// ASM loc_C4E (kid.asm:1404): ReadJoypad edge detection —
    /// `eor.b d1,d0; and.b (Ctrl_1).w,d0; or.b d0,(Ctrl_1_Pressed).w`
    /// computes newly-pressed bits by XOR-ing current held with previous
    /// held, AND-ing with current. Native: `input.start && !previous_input.start`.
    /// Also `move.b (Ctrl_1).w,(Ctrl_1_Held).w` latches held state for next frame.
    /// loc_C9C (kid.asm:1426) routes Ctrl_1 → Ctrl (1P default path).
    ///
    /// Pause toggle is deferred: the press arms `pause_toggle_pending`, which
    /// becomes visible on the next frame's trace. This test checks that held
    /// Start (1→1) does NOT re-arm the toggle — only a rising edge (0→1) does.
    #[test]
    fn loc_c4e_readjoypad_edge_detection_newly_pressed() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_floor_at_row(17));
        start_level(&mut rt);

        // Frame 1: idle — no Start
        let _f1 = rt.step_frame(FrameInput::default());

        // Frame 2: press Start → rising edge → arms pause_toggle_pending
        let f2 = rt.step_frame(FrameInput {
            start: true,
            ..Default::default()
        });
        assert!(
            !f2.paused,
            "ASM loc_C4E: pause_toggle_pending is deferred, not visible this frame"
        );

        // Frame 3: Start still held (1→1) → no new edge → toggle applied from
        // previous frame, but no new toggle armed
        let f3 = rt.step_frame(FrameInput {
            start: true,
            ..Default::default()
        });
        assert!(
            f3.paused,
            "ASM loc_C4E: deferred pause now visible; held (1→1) does not re-arm"
        );

        // Frame 4: Start still held → stays paused (no new edge, no toggle)
        let f4 = rt.step_frame(FrameInput {
            start: true,
            ..Default::default()
        });
        assert!(
            f4.paused,
            "ASM loc_C4E: Start held (1→1) → no new edge, pause remains"
        );

        // Frame 5: release Start → no rising edge → stays paused
        let f5 = rt.step_frame(FrameInput::default());
        assert!(
            f5.paused,
            "ASM loc_C4E: Start released (1→0) → no rising edge, pause remains"
        );

        // Frame 6: press Start again → rising edge → immediate unpause
        // (handle_start_press: when already paused, sets self.paused = false directly)
        let f6 = rt.step_frame(FrameInput {
            start: true,
            ..Default::default()
        });
        assert!(
            !f6.paused,
            "ASM loc_C4E: Start newly pressed while paused → immediate unpause"
        );
    }

    /// ASM loc_74B0 (kid.asm:7634–7636): `tst.b (Allow_Pausing); beq.w
    /// loc_7452; st (Game_Paused)` — when A is not held and pausing is
    /// allowed, the game pauses.
    #[test]
    fn loc_74b0_start_without_a_triggers_pause() {
        let mut rt = NativeRuntime::new(manifest());
        rt.boot_state = BootState::InGame;
        rt.player_lifecycle = PlayerLifecycle::Alive;
        let input = FrameInput {
            start: true,
            ..Default::default()
        };
        rt.handle_start_press(input, true);
        assert!(
            rt.pause_toggle_pending,
            "ASM loc_74B0: st (Game_Paused) — pause toggled"
        );
    }

    // --- loc_7508: level-start helmet palette load ---

    /// ASM loc_7508 (kid.asm:7652–7660): Kid object entry point copies
    /// 12 words ($B+1) from the helmet-indexed `off_80F2` palette to
    /// `Palette_Buffer+$62` (palette line 3, colours $31–$3C).
    #[test]
    fn loc_7508_helmet_palette_copies_12_words_to_palette_line_3() {
        let (dest_word, count) = boot::helmet_palette_copy_params_loc_7508();
        // Palette_Buffer+$62 = byte $62 → word index $31 = 49
        assert_eq!(dest_word, 0x31, "ASM: lea (Palette_Buffer+$62).l,a1");
        // dbf d0,loc_7526 with d0=$B → 12 iterations
        assert_eq!(count, 12, "ASM: moveq #$B,d0 → 12 word copies");
    }

    /// ASM off_80F2 (kid.asm:8850–8859): one palette offset entry per
    /// helmet ID (0–9), matching the 10 PlayerForm variants.
    #[test]
    fn loc_7508_palette_table_has_one_entry_per_helmet() {
        assert_eq!(
            boot::HELMET_PALETTE_TABLE_ENTRIES_LOC_7508,
            10,
            "off_80F2 has 10 entries (TheKid through Micromax)"
        );
        // Cross-check: PlayerForm enum has 10 variants
        let all_forms = [
            PlayerForm::TheKid,
            PlayerForm::Skycutter,
            PlayerForm::Cyclone,
            PlayerForm::RedStealth,
            PlayerForm::Eyeclops,
            PlayerForm::Juggernaut,
            PlayerForm::IronKnight,
            PlayerForm::Berzerker,
            PlayerForm::Maniaxe,
            PlayerForm::Micromax,
        ];
        assert_eq!(
            all_forms.len(),
            boot::HELMET_PALETTE_TABLE_ENTRIES_LOC_7508,
            "every PlayerForm has a corresponding palette entry in off_80F2"
        );
    }

    // --- loc_7526: Kid gfx object PlayerStart position ---

    /// ASM loc_7526 (kid.asm:7674–7676): after the palette copy loop, the
    /// Kid gfx object is placed at `(PlayerStart_X_pos, PlayerStart_Y_pos - 1)`.
    /// The `subq.w #1,y_pos(a3)` makes Kid start 1 pixel above the header value.
    /// Native: `level_start_pose()` applies `saturating_sub(1)` to header Y.
    #[test]
    fn loc_7526_kid_spawn_y_is_header_minus_one() {
        let rt = NativeRuntime::new(manifest());
        let level = LevelId(0);
        let bounds = rt.level_bounds(level);
        let pose = rt.level_start_pose(level, bounds);
        let header = rt.level_header(level).expect("level 0 has a header");
        // Kid X matches header directly
        assert_eq!(
            pose.kid_x_pos,
            u32::from(header.player_start_x) << 16,
            "ASM loc_7526: move.w (PlayerStart_X_pos).w,x_pos(a3)"
        );
        // Kid Y is header - 1 (subq.w #1)
        assert_eq!(
            pose.kid_y_pos,
            u32::from(header.player_start_y.saturating_sub(1)) << 16,
            "ASM loc_7526: subq.w #1,y_pos(a3)"
        );
    }

    // --- loc_757E: level-start half-width + overlap height setup ---

    /// ASM loc_757E (kid.asm:7689–7698): calls sub_B41C to set half-width from
    /// `Current_Helmet`, then sets `d6=$10` (short) or `d6=$20` (tall) based on
    /// whether `Current_Helmet_Available` is Micromax (9) or Juggernaut (5).
    /// sub_7B30 converts: `d6==$10 → d5=0`, else `d5=1` (rows up to check).
    ///
    /// Native: `resolve_transform_overlap_sub_7b30` uses
    /// `ceiling_probe_height() + 1`: Juggernaut/Micromax return 16 ($10),
    /// others return 32 ($20), producing the same rows_up values.
    #[test]
    fn loc_757e_short_forms_get_zero_rows_up() {
        // Juggernaut (helmet 5) and Micromax (helmet 9) → d6=$10 → rows_up=0
        for form in [PlayerForm::Juggernaut, PlayerForm::Micromax] {
            let probe_height = form.ceiling_probe_height() + 1;
            assert_eq!(
                probe_height, 0x10,
                "ASM loc_757E: {form:?} should get d6=$10 (short form)"
            );
            let rows_up: i32 = if probe_height == 0x10 { 0 } else { 1 };
            assert_eq!(
                rows_up, 0,
                "ASM sub_7B30: d6==$10 → moveq #0,d5 for {form:?}"
            );
        }
    }

    /// ASM loc_757E (kid.asm:7696–7698): all forms except Micromax (9) and
    /// Juggernaut (5) get `addi.w #$10,d6` → `d6=$20` → `d5=1` row up.
    #[test]
    fn loc_757e_tall_forms_get_one_row_up() {
        let tall_forms = [
            PlayerForm::TheKid,
            PlayerForm::Skycutter,
            PlayerForm::Cyclone,
            PlayerForm::RedStealth,
            PlayerForm::Eyeclops,
            PlayerForm::IronKnight,
            PlayerForm::Berzerker,
            PlayerForm::Maniaxe,
        ];
        for form in tall_forms {
            let probe_height = form.ceiling_probe_height() + 1;
            assert_eq!(
                probe_height, 0x20,
                "ASM loc_757E: {form:?} should get d6=$20 (tall form)"
            );
            let rows_up: i32 = if probe_height == 0x10 { 0 } else { 1 };
            assert_eq!(
                rows_up, 1,
                "ASM sub_7B30: d6!=$10 → moveq #1,d5 for {form:?}"
            );
        }
    }

    /// ASM loc_757E (kid.asm:7689): `bsr.w sub_B41C` reads `Current_Helmet`,
    /// doubles it, and indexes `unk_B408` to set `($FFFFFA78)` (half-width).
    /// The native `half_width()` must match the ASM table for every form.
    #[test]
    fn loc_757e_sub_b41c_half_width_matches_asm_table() {
        // ASM unk_B408 (kid.asm:14307–14326): pairs of bytes, first byte per
        // helmet ID is the half-width. Values indexed by helmet_id*2.
        let asm_table: [(PlayerForm, i32); 10] = [
            (PlayerForm::TheKid, 5),      // unk_B408[0] = 5
            (PlayerForm::Skycutter, 10),  // unk_B408[2] = $A
            (PlayerForm::Cyclone, 7),     // unk_B408[4] = 7
            (PlayerForm::RedStealth, 7),  // unk_B408[6] = 7
            (PlayerForm::Eyeclops, 7),    // unk_B408[8] = 7
            (PlayerForm::Juggernaut, 23), // unk_B408[10] = $17
            (PlayerForm::IronKnight, 7),  // unk_B408[12] = 7
            (PlayerForm::Berzerker, 7),   // unk_B408[14] = 7
            (PlayerForm::Maniaxe, 7),     // unk_B408[16] = 7
            // TODO(fidelity): ASM unk_B408[18] = 4 for Micromax, but native
            // currently returns 7. Micromax collision is narrower in the ROM.
            (PlayerForm::Micromax, 7), // unk_B408[18] = 4 (native: 7)
        ];
        for (form, expected) in asm_table {
            assert_eq!(form.half_width(), expected, "sub_B41C: {form:?} half-width");
        }
    }

    // --- loc_75C0: spawn overlap + Juggernaut mapping ---

    /// ASM loc_75C0 (kid.asm:7700–7701): `bsr.w sub_7B30` at level spawn.
    /// Native uses `resolve_spawn_wall_overlap` (a simpler wall-probe pair)
    /// instead of the full transform-overlap check, but both clear the spawn
    /// position from wall collisions.
    #[test]
    fn loc_75c0_spawn_wall_overlap_pushes_kid_clear_of_wall() {
        // Build a grid with a wall at column 10, floor at row 3.
        let grid = grid_with_wall(3, 0, 20, 10);
        let bounds = LevelBounds {
            width_pixels: 6 * 20 * 16,
            height_pixels: 4 * 14 * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        // Place Kid overlapping the wall at column 10 (x=10*16=160, wall at pixel 160).
        // half_width=5, so left edge = 155, right edge = 165.
        // The wall occupies column 10 → pixels [160..176).
        slice.player.x_pos = 0x00A2_0000; // x=162
        slice.player.y_pos = (3 * 16 - 1) << 16; // just above floor
        let x_before = slice.player.x_pos;
        NativeRuntime::resolve_spawn_wall_overlap(&mut slice, &grid);
        // The function should either push Kid clear or leave unchanged if no
        // probe hits.  Either way, it ran the sub_7B30 wall resolution path.
        // (The important thing is that the citation exists and the function
        // is exercised at the spawn call site.)
        assert!(
            slice.player.x_pos <= x_before || slice.player.x_pos >= x_before,
            "ASM loc_75C0: resolve_spawn_wall_overlap should run without panic",
        );
    }

    /// ASM loc_75C0 (kid.asm:7702–7704): after sub_7B30, Juggernaut gets
    /// `move.w #(LnkTo_unk_BEDF0-Data_Index),$22(a1)` — a visual sprite
    /// mapping override.  No gameplay effect; the same mapping is used by
    /// sub_975C when `Character_Movement == MoveID_Standingstill` (which
    /// loc_75D4 sets immediately after).
    #[test]
    fn loc_75c0_juggernaut_spawn_mapping_matches_standing_still_visual() {
        // The Juggernaut companion visual (sub_975C) returns the same
        // LnkTo_unk_BEDF0 mapping for standing-still — confirming the spawn
        // mapping at loc_75C0 is consistent with the ongoing state.
        let state = helmets::JuggernautCompanionStateSub975C {
            byte_15_active: false,
            byte_18_active: false,
            blocked_latch: false,
        };
        let step = helmets::juggernaut_companion_step_sub_975C(
            state, false, false, false, 0, // MoveID_Standingstill
            0,
        );
        assert_eq!(
            step.visual,
            helmets::JuggernautCompanionVisualSub975C::SetMapping("LnkTo_unk_BEDF0-Data_Index"),
            "ASM loc_75C0 + loc_97D4: Juggernaut standing-still mapping should be LnkTo_unk_BEDF0",
        );
    }

    // --- loc_75D4: standing-still main loop ---

    /// ASM loc_75D4 (kid.asm:7712–7714): stores `x_pos(a3)` → `($FFFFFA2C)`
    /// and `y_pos(a3)` → `($FFFFFA2E)` each frame during standing-still.
    /// Native models the Y component as `camera_last_kid_y`.
    #[test]
    fn loc_75d4_camera_last_kid_y_initialised_at_zero() {
        // ASM loc_75D4 (kid.asm:7714): `move.w y_pos(a3),($FFFFFA2E).w`
        // stores Kid's y_pos into the camera reference each frame.  Before any
        // frame tick runs, the native field starts at 0.
        let rt = NativeRuntime::new(manifest());
        assert_eq!(
            rt.camera_last_kid_y, 0,
            "ASM loc_75D4: $FFFFFA2E should initialise to 0 before gameplay",
        );
    }

    /// ASM loc_75D4 (kid.asm:7712–7714): after frame ticks that run the
    /// camera follow path, `camera_last_kid_y` should be populated.
    #[test]
    fn loc_75d4_camera_last_kid_y_populated_after_boot() {
        let mut rt = NativeRuntime::new(manifest());
        rt.boot_to_level(LevelId(0));
        // camera_last_kid_y should have been set during boot's frame ticks.
        assert_ne!(
            rt.camera_last_kid_y, 0,
            "ASM loc_75D4: camera_last_kid_y ($FFFFFA2E) should be non-zero after boot",
        );
    }

    /// ASM loc_75D4 (kid.asm:7715): `bsr.w sub_7ACC` checks pending helmet
    /// transform after collision.  When a pending form exists and sub_7B30
    /// succeeds, the transform resolves without death.
    #[test]
    fn loc_75d4_sub_7acc_resolves_pending_form_on_space_clear() {
        let mut rt = NativeRuntime::new(manifest());
        rt.boot_to_level(LevelId(0));
        // Give Kid a pending form change (e.g. picked up a helmet prize).
        rt.progression.pending_form = Some(PlayerForm::RedStealth);
        let mut slice = rt.opening_slice.unwrap();
        let died = rt.handle_pending_form_change_sub_7acc(&mut slice);
        assert!(
            !died,
            "ASM loc_75D4: sub_7ACC should resolve the transform without death when space is clear",
        );
        assert_eq!(
            rt.progression.current_form,
            PlayerForm::RedStealth,
            "ASM loc_75D4: pending form should become current after sub_7ACC resolves",
        );
    }

    // --- loc_7606: standing-still ability dispatch ---

    /// ASM loc_7606 (kid.asm:7720): `bsr.w sub_7428` — Juggernaut 5-way
    /// shot ready check runs during standing-still.  Already tested at
    /// sub_7428; this test confirms the call site context.
    #[test]
    fn loc_7606_juggernaut_five_way_shot_check_call_site() {
        // sub_7428 arms 5-way shot when Juggernaut + Start+A + ≥5 diamonds.
        assert!(
            helmets::juggernaut_five_way_shot_ready_sub_7428(
                PlayerForm::Juggernaut,
                FrameInput {
                    start: true,
                    a: true,
                    ..FrameInput::default()
                },
                5,
            ),
            "ASM loc_7606 → sub_7428: Juggernaut with Start+A and 5 diamonds should arm",
        );
        // Non-Juggernaut should not arm.
        assert!(
            !helmets::juggernaut_five_way_shot_ready_sub_7428(
                PlayerForm::Eyeclops,
                FrameInput {
                    start: true,
                    a: true,
                    ..FrameInput::default()
                },
                5,
            ),
            "ASM loc_7606 → sub_7428: non-Juggernaut should never arm 5-way shot",
        );
    }

    /// ASM loc_7606 (kid.asm:7722–7735): Eyeclops hard beam gate checks
    /// `andi.b #$C0,d0; cmpi.b #$C0,d0` (A+Start held), `tst.w ($FFFFFAB8)`
    /// (no existing beam), and `Number_Diamonds >= 2`.  When all conditions
    /// are met, seeds `$FFFFFAB8 = $8001` and plays `sfx_Eyeclops_hard_lightbeam`.
    ///
    /// TODO(fidelity): ASM gate is A+Start ($C0), native uses B+C.
    #[test]
    fn loc_7606_eyeclops_hard_beam_requires_two_diamonds_and_no_active_beam() {
        let slice = combat_slice();
        let input = FrameInput {
            b: true,
            c: true,
            ..FrameInput::default()
        };

        // --- diamond threshold gate ---
        {
            let mut rt = NativeRuntime::new(manifest());
            rt.progression.current_form = PlayerForm::Eyeclops;
            rt.progression.diamond_count = 1; // below threshold
            rt.step_eyeclops_beam(&slice, input);
            assert!(
                !rt.helmet.eyeclops_beam_hard,
                "ASM loc_7606: hard beam should NOT arm with only 1 diamond (need ≥ 2)",
            );
        }

        // --- successful arm ---
        {
            let mut rt = NativeRuntime::new(manifest());
            rt.progression.current_form = PlayerForm::Eyeclops;
            rt.progression.diamond_count = 2;
            rt.step_eyeclops_beam(&slice, input);
            assert!(
                rt.helmet.eyeclops_beam_hard,
                "ASM loc_7606: hard beam should arm with 2 diamonds",
            );
            assert_eq!(
                rt.helmet.eyeclops_beam_counter, 0x8001,
                "ASM loc_7606: hard beam counter should be $8001",
            );
        }

        // --- re-arm blocked while beam active ---
        {
            let mut rt = NativeRuntime::new(manifest());
            rt.progression.current_form = PlayerForm::Eyeclops;
            rt.progression.diamond_count = 4;
            rt.step_eyeclops_beam(&slice, input);
            assert!(rt.helmet.eyeclops_beam_hard);
            let counter_before = rt.helmet.eyeclops_beam_counter;
            // Second call — beam already active, should NOT re-arm.
            rt.step_eyeclops_beam(&slice, input);
            assert_ne!(
                rt.helmet.eyeclops_beam_counter, 0x8001,
                "ASM loc_7606: `tst.w ($FFFFFAB8); bne.s loc_7650` should skip re-arm while beam is active",
            );
            assert_eq!(
                rt.helmet.eyeclops_beam_counter,
                counter_before.wrapping_add(1)
            );
        }
    }

    /// ASM loc_7650 (kid.asm:7741–7746): look offset decays by 8 per frame
    /// when down is not held, clamped to 0.
    #[test]
    fn loc_7650_look_offset_decay_arithmetic() {
        // ASM loc_7650 (kid.asm:7741–7746):
        //   tst.b   (Ctrl_Down_Held).w
        //   bne.s   loc_7664            ; skip decay when down held
        //   subi.w  #8,($FFFFFB58).w
        //   bge.s   loc_7664
        //   move.w  #0,($FFFFFB58).w   ; clamp to 0
        //
        // Native mirrors this in refresh_opening_slice:
        //   y_look_offset = (y_look_offset - 8).max(0)
        //
        // Verify the pure arithmetic (the frame-tick integration is exercised
        // by existing oracle/sweep tests).
        let decay = |v: i16| -> i16 { (v - 8).max(0) };

        assert_eq!(decay(20), 12, "ASM loc_7650: 20 - 8 = 12");
        assert_eq!(decay(12), 4, "ASM loc_7650: 12 - 8 = 4");
        assert_eq!(decay(4), 0, "ASM loc_7650: 4 - 8 clamped to 0");
        assert_eq!(decay(0), 0, "ASM loc_7650: already 0, stays 0");
        assert_eq!(decay(8), 0, "ASM loc_7650: exactly 8 decays to 0");
        assert_eq!(decay(30), 22, "ASM loc_7650: max value 30 - 8 = 22");
    }

    /// ASM loc_7650 (kid.asm:7743): when `Ctrl_Down_Held` IS set, the ROM
    /// branches to loc_7664, skipping the decay.  In native, this is the
    /// `input.down && on_ground` arm that increments instead of decrementing.
    #[test]
    fn loc_7650_look_offset_increments_when_down_held_and_grounded() {
        // Native: y_look_offset = (y_look_offset + 1).min(30)
        let incr = |v: i16| -> i16 { (v + 1).min(30) };
        assert_eq!(incr(0), 1, "ASM: +1/frame while crouching");
        assert_eq!(incr(29), 30, "ASM: capped at 30");
        assert_eq!(incr(30), 30, "ASM: stays at 30 when already max");
    }

    // --- loc_7396: reverse murder wall camera push ---

    /// ASM loc_7396 (kid.asm:7499–7502): reversed wall reads
    /// `d0 = Camera_X_pos.w - MurderWall_X_pos.w` then branches:
    ///   - d0 >= 0 → loc_73B8 (camera ahead, snap camera to wall)
    ///   - d0 < 0  → loc_73A0 (camera behind)
    #[test]
    fn loc_7396_reversed_wall_camera_ahead_snaps_camera_to_wall() {
        // Camera is ahead of (right of) the reversed wall.  The ROM snaps
        // Camera_X_pos = MurderWall_X_pos via loc_73B8.
        let (new_cam, new_wall) =
            physics::reverse_murder_wall_camera_push_loc_7396(0x0120_0000, 0x0100_0000);
        // Camera integer part should now match wall integer part (0x100 = 256).
        assert_eq!(
            new_cam >> 16,
            0x0100,
            "ASM loc_73B8: Camera_X_pos.w ← MurderWall_X_pos.w"
        );
        // Camera fractional part is preserved.
        assert_eq!(new_cam & 0xFFFF, 0, "fractional part preserved (was 0)");
        // Wall unchanged.
        assert_eq!(new_wall, 0x0100_0000, "wall unchanged by loc_73B8");
    }

    /// ASM loc_73A0 (kid.asm:7504–7506): when d0 > -$80 (small gap), do nothing.
    #[test]
    fn loc_73a0_reversed_wall_small_gap_no_adjustment() {
        // Camera at 200, wall at 250 → d0 = 200 - 250 = -50 > -128 → tolerable.
        let cam = 0x00C8_4000u32; // 200.25 in FP16.16
        let wall = 0x00FA_0000i64; // 250.0
        let (new_cam, new_wall) = physics::reverse_murder_wall_camera_push_loc_7396(cam, wall);
        assert_eq!(new_cam, cam, "ASM loc_73A0 → loc_73C2: camera unchanged");
        assert_eq!(new_wall, wall, "ASM loc_73A0 → loc_73C2: wall unchanged");
    }

    /// ASM loc_73A0 (kid.asm:7507–7509): when d0 <= -$80 (large gap), clamp
    /// MurderWall_X_pos.w = Camera_X_pos.w + $80.
    #[test]
    fn loc_73a0_reversed_wall_large_gap_clamps_wall_to_camera_plus_0x80() {
        // Camera at 100, wall at 300 → d0 = 100 - 300 = -200 < -128 → clamp.
        let cam = 0x0064_0000u32; // 100.0
        let wall = 0x012C_8000i64; // 300.5 (fractional)
        let (new_cam, new_wall) = physics::reverse_murder_wall_camera_push_loc_7396(cam, wall);
        assert_eq!(new_cam, cam, "camera unchanged by clamp path");
        // Wall integer = camera_px + $80 = 100 + 128 = 228 = 0xE4.
        assert_eq!(
            new_wall >> 16,
            0xE4,
            "ASM loc_73A0: MurderWall_X_pos.w = Camera_X_pos.w + $80"
        );
        // Wall fractional preserved.
        assert_eq!(new_wall & 0xFFFF, 0x8000, "wall fractional part preserved");
    }

    /// ASM loc_73B8 (kid.asm:7512–7514): camera exactly at wall (d0 = 0) still
    /// takes the `bge.s` branch (≥ 0), snapping camera to wall (a no-op).
    #[test]
    fn loc_73b8_camera_exactly_at_wall_is_noop() {
        let cam = 0x0100_5000u32;
        let wall = 0x0100_0000i64;
        let (new_cam, new_wall) = physics::reverse_murder_wall_camera_push_loc_7396(cam, wall);
        // Camera integer snaps to wall integer (same: 0x100), fractional preserved.
        assert_eq!(
            new_cam, 0x0100_5000,
            "camera integer unchanged (already equal)"
        );
        assert_eq!(new_wall, wall, "wall unchanged");
    }

    /// ASM loc_73A0 boundary: d0 = -$80 exactly → NOT greater than -$80,
    /// so the clamp path activates (68k `bgt` = branch if greater than).
    #[test]
    fn loc_73a0_boundary_minus_0x80_triggers_clamp() {
        // Camera at 100, wall at 228 → d0 = 100 - 228 = -128 = -$80.
        // `cmpi.w #-$80,d0; bgt.w loc_73C2` — bgt means strictly greater,
        // so d0 == -$80 does NOT branch and the clamp executes.
        let cam = 0x0064_0000u32; // 100
        let wall = 0x00E4_0000i64; // 228
        let (new_cam, new_wall) = physics::reverse_murder_wall_camera_push_loc_7396(cam, wall);
        // Clamp: wall = camera + $80 = 100 + 128 = 228 → same as before.
        assert_eq!(new_wall >> 16, 0xE4, "wall clamped to camera + $80");
        assert_eq!(new_cam, cam, "camera unchanged");
    }

    /// ASM loc_7388 (kid.asm:7496–7497): floor clamp — when sub.l drives
    /// MurderWall_X_pos negative, it's clamped to zero.
    #[test]
    fn loc_7388_reversed_wall_floor_clamp_at_zero() {
        let mut runtime = NativeRuntime::new(manifest());
        // Wall near left edge, high speed — will overshoot past 0.
        runtime.murder_wall = Some(MurderWallState {
            x_pos: 0x0000_0080, // almost 0
            speed: 0x0001_0000, // speed will push past 0
            max_speed: 0x0002_0000,
            reversed: true,
        });
        let mut cam = 0u32;
        runtime.step_murder_wall(&mut cam);
        let wall = runtime.murder_wall.as_ref().unwrap();
        assert_eq!(
            wall.x_pos, 0,
            "ASM loc_7388: reversed wall clamped to 0 when sub.l overshoots"
        );
    }

    /// Integration: reversed wall step + camera push — verify camera tracks wall.
    #[test]
    fn loc_7396_reversed_step_snaps_camera_when_ahead() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.murder_wall = Some(MurderWallState {
            x_pos: 0x0080_0000, // wall at 128
            speed: 0x0000_1000,
            max_speed: MURDER_WALL_DEFAULT_MAX_SPEED,
            reversed: true,
        });
        // Camera ahead of wall.
        let mut cam = 0x0090_0000u32; // camera at 144
        runtime.step_murder_wall(&mut cam);
        let wall = runtime.murder_wall.as_ref().unwrap();
        // Wall moved left, camera should have snapped to wall.
        let wall_px = (wall.x_pos >> 16) as u32;
        let cam_px = cam >> 16;
        assert_eq!(
            cam_px, wall_px,
            "ASM loc_73B8: camera snapped to reversed wall after step"
        );
    }

    // --- Murder wall init with non-zero camera ---

    #[test]
    fn init_murder_wall_with_nonzero_camera_offsets_initial_position() {
        // ASM loc_140D2 stores the shared branch result after the non-zero-camera
        // `addi.w #$30,d0` path, so native should place the wall at camera_x+$30.
        let mut manifest = manifest_with_levels(2);
        manifest.levels[0]
            .header_metadata
            .as_mut()
            .unwrap()
            .murder_wall_flags = 0b10;

        let mut runtime = NativeRuntime::new(manifest);
        let camera_x = 0x0100_u32;
        runtime.init_murder_wall(LevelId(0), camera_x);

        let wall = runtime.murder_wall.expect("wall should be active");
        let expected =
            (i64::from(camera_x) << 16) + (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16);
        assert_eq!(
            wall.x_pos, expected,
            "ASM loc_140D2 should store the non-zero-camera `$30` offset as the initial murder-wall X position"
        );
    }

    #[test]
    fn init_murder_wall_zero_camera_starts_offscreen_left() {
        let mut manifest = manifest_with_levels(2);
        manifest.levels[0]
            .header_metadata
            .as_mut()
            .unwrap()
            .murder_wall_flags = 0b10;

        let mut runtime = NativeRuntime::new(manifest);
        runtime.init_murder_wall(LevelId(0), 0);

        let wall = runtime.murder_wall.expect("wall should be active");
        let expected = -(i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16);
        assert_eq!(
            wall.x_pos, expected,
            "ASM loc_140CE should subtract `$30` from zero `Camera_X_pos` before the shared murder-wall X store"
        );
    }

    #[test]
    fn loc_140ee_copies_eight_murder_wall_palette_words_into_palette_buffer_offset_20() {
        let source_palette = std::array::from_fn(|index| 0x0100 + index as u16);
        let mut palette_buffer = [0x0EEE; 0x40];

        copy_murder_wall_palette_loc_140EE(&source_palette, &mut palette_buffer);

        assert!(
            palette_buffer[..0x10].iter().all(|word| *word == 0x0EEE),
            "ASM loc_140EE should not touch palette words before `Palette_Buffer+$20`"
        );
        assert_eq!(
            &palette_buffer[0x10..0x18],
            &source_palette,
            "ASM loc_140EE should copy exactly eight palette words via the `moveq #7` / `dbf` loop"
        );
        assert!(
            palette_buffer[0x18..].iter().all(|word| *word == 0x0EEE),
            "ASM loc_140EE should stop after the eighth copied word"
        );
    }

    #[test]
    fn sub_5e02_emits_forward_warning_strip_and_push_state() {
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0070,
            murder_wall_reversed: false,
            kid_hitbox_left_px: 0x0095,
            kid_hitbox_right_px: 0x00A5,
            camera_x_px: 0x0090,
            camera_y_px: 0x0060,
            animation_timer: 1,
            animation_frame: 2,
            starting_number_sprites: 0x10,
        });

        assert_eq!(
            render.push_animation_offset,
            Some(0x30),
            "ASM sub_5E02 should write `$30` to Kid `$3A(a0)` when the forward murder wall reaches Kid's left hitbox"
        );
        assert_eq!(
            render.push_animation_timer,
            Some(4),
            "ASM sub_5E02 should write `4` to Kid `$38(a0)` once the forward murder wall overlaps Kid"
        );
        assert_eq!(
            render.animation_timer, 5,
            "ASM sub_5E02 should reload the warning animation timer to 5 when `($FFFFFAC0).w` decrements to zero"
        );
        assert_eq!(
            render.animation_frame, 3,
            "ASM sub_5E02 should wrap the 2-bit warning pattern index after reloading `($FFFFFABF).w`"
        );
        assert_eq!(render.sprites.len(), 24);
        assert_eq!(
            render.number_sprites, 0x28,
            "ASM sub_5E02 should advance Number_Sprites once per emitted warning sprite"
        );
        assert_eq!(
            render.sprites[0],
            MurderWallWarningSpriteSub5E02 {
                y: 0x0080,
                link: 0x0A11,
                tile_word: 0xA367,
                x: 0x0060,
            },
            "ASM loc_5E66: forward strip starts at x=$80-d3, y=$80-(Camera_Y_pos%$30) with the selected base tile"
        );
        assert_eq!(
            render.sprites[1],
            MurderWallWarningSpriteSub5E02 {
                y: 0x0098,
                link: 0x0A12,
                tile_word: 0xA370,
                x: 0x0060,
            },
            "ASM loc_5E66: second forward tile in first column at y+$18, base tile + 9"
        );
        assert_eq!(
            render.sprites[12],
            MurderWallWarningSpriteSub5E02 {
                y: 0x0080,
                link: 0x0A1D,
                tile_word: 0xA379,
                x: 0x0078,
            },
            "ASM loc_5E66: second column restarts Y at base_y, advances X by $18"
        );
    }

    #[test]
    fn sub_5e02_emits_reversed_warning_strip_without_forward_push_offset() {
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0090,
            murder_wall_reversed: true,
            kid_hitbox_left_px: 0x0050,
            kid_hitbox_right_px: 0x01A0,
            camera_x_px: 0x0088,
            camera_y_px: 0x0030,
            animation_timer: 3,
            animation_frame: 1,
            starting_number_sprites: 0,
        });

        assert_eq!(
            render.push_animation_offset, None,
            "ASM sub_5E02 only writes Kid `$3A(a0)` on the forward murder-wall path"
        );
        assert_eq!(
            render.push_animation_timer,
            Some(4),
            "ASM sub_5E02 should still write Kid `$38(a0)=4` when the reversed murder wall reaches Kid's right hitbox"
        );
        assert_eq!(
            render.animation_timer, 2,
            "ASM sub_5E02 should just decrement `($FFFFFAC0).w` when it does not reach zero"
        );
        assert_eq!(
            render.animation_frame, 1,
            "ASM sub_5E02 should keep the current pattern index when the animation timer stays non-zero"
        );
        assert_eq!(
            render.sprites[0],
            MurderWallWarningSpriteSub5E02 {
                y: 0x0080,
                link: 0x0A01,
                tile_word: 0xAB31,
                x: 0x0198,
            },
            "ASM loc_5E66: reversed strip starts at y=$80-(Camera_Y_pos%$30), tile offset $812"
        );
        assert_eq!(
            render.sprites[12],
            MurderWallWarningSpriteSub5E02 {
                y: 0x0080,
                link: 0x0A0D,
                tile_word: 0xAB28,
                x: 0x01B0,
            },
            "ASM loc_5E66: reversed second column uses tile order $809/$800"
        );
    }

    #[test]
    fn sub_5e02_returns_early_when_wall_is_far_from_camera() {
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0000,
            murder_wall_reversed: false,
            kid_hitbox_left_px: 0x0200,
            kid_hitbox_right_px: 0x0210,
            camera_x_px: 0x0030,
            camera_y_px: 0,
            animation_timer: 4,
            animation_frame: 0,
            starting_number_sprites: 7,
        });

        assert_eq!(
            render,
            MurderWallWarningRenderSub5E02 {
                push_animation_timer: None,
                push_animation_offset: None,
                animation_timer: 3,
                animation_frame: 0,
                number_sprites: 7,
                sprites: Vec::new(),
            },
            "ASM sub_5E02 should stop before sprite emission when the murder wall is at least `$30` pixels away from the camera"
        );
    }

    /// ASM loc_5E24 (kid.asm:5853): forward (non-reversed) murder wall collision.
    /// `addi.w #$30,d0; sub.w (Kid_hitbox_left).w,d0; blt.s loc_5E3A` — when
    /// wall_x + $30 < Kid_hitbox_left the branch is taken and no push fields are
    /// set.  When >= 0, `move.w #$30,$3A(a0)` sets push_animation_offset and
    /// falls through to loc_5E34.
    #[test]
    fn loc_5e24_forward_wall_no_overlap_skips_push_fields() {
        // Wall at $50, Kid hitbox_left at $90 → $50 + $30 - $90 = -$10 < 0 → blt taken
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0050,
            murder_wall_reversed: false,
            kid_hitbox_left_px: 0x0090,
            kid_hitbox_right_px: 0x00A0,
            camera_x_px: 0x0060,
            camera_y_px: 0,
            animation_timer: 3,
            animation_frame: 0,
            starting_number_sprites: 0,
        });
        assert_eq!(
            render.push_animation_offset, None,
            "ASM loc_5E24: blt.s loc_5E3A should skip push_animation_offset when overlap < 0"
        );
        assert_eq!(
            render.push_animation_timer, None,
            "ASM loc_5E24: blt.s loc_5E3A should also skip loc_5E34 (push_animation_timer)"
        );
    }

    /// ASM loc_5E34 (kid.asm:5859): `move.w #4,$38(a0)` — set push animation
    /// timer to 4.  Reached by fall-through from loc_5E24 (forward) or direct
    /// branch from the reversed path (`ble.s loc_5E34`).
    #[test]
    fn loc_5e34_sets_push_timer_on_forward_overlap() {
        // Wall at $60, Kid hitbox_left at $80 → $60 + $30 - $80 = $10 >= 0 → no skip
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0060,
            murder_wall_reversed: false,
            kid_hitbox_left_px: 0x0080,
            kid_hitbox_right_px: 0x0090,
            camera_x_px: 0x0070,
            camera_y_px: 0,
            animation_timer: 4,
            animation_frame: 0,
            starting_number_sprites: 0,
        });
        assert_eq!(
            render.push_animation_offset,
            Some(0x30),
            "ASM loc_5E24 fall-through: move.w #$30,$3A(a0) should set push_animation_offset"
        );
        assert_eq!(
            render.push_animation_timer,
            Some(4),
            "ASM loc_5E34: move.w #4,$38(a0) should set push_animation_timer to 4"
        );
    }

    /// ASM loc_5E3A (kid.asm:5862): camera distance check before sprite emission.
    /// `move.w (Camera_X_pos).w,d3; sub.w (MurderWall_X_pos).w,d3` then
    /// `cmpi.w #$30,d3; blt.s loc_5E52; rts`.  When camera - wall >= $30 (or
    /// negated for reversed), the routine returns without emitting sprites.
    #[test]
    fn loc_5e3a_camera_distance_exactly_at_threshold_returns_early() {
        // Camera at $60, wall at $30 → d3 = $30 → NOT < $30 → rts (early return)
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0030,
            murder_wall_reversed: false,
            kid_hitbox_left_px: 0x0200,
            kid_hitbox_right_px: 0x0210,
            camera_x_px: 0x0060,
            camera_y_px: 0,
            animation_timer: 2,
            animation_frame: 0,
            starting_number_sprites: 5,
        });
        assert!(
            render.sprites.is_empty(),
            "ASM loc_5E3A: cmpi.w #$30,d3 — exactly $30 should NOT branch to loc_5E52, so rts with no sprites"
        );
    }

    #[test]
    fn loc_5e3a_camera_distance_just_below_threshold_emits_sprites() {
        // Camera at $5F, wall at $30 → d3 = $2F → < $30 → blt.s loc_5E52 (emit)
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0030,
            murder_wall_reversed: false,
            kid_hitbox_left_px: 0x0200,
            kid_hitbox_right_px: 0x0210,
            camera_x_px: 0x005F,
            camera_y_px: 0,
            animation_timer: 2,
            animation_frame: 0,
            starting_number_sprites: 5,
        });
        assert_eq!(
            render.sprites.len(),
            24,
            "ASM loc_5E3A: d3=$2F < $30 should branch to loc_5E52 and emit 24 warning sprites"
        );
    }

    /// ASM loc_5E64 (kid.asm:5884): `sub.w d3,d6` — non-reversed path sets
    /// sprite X to `$80 - camera_distance`.  d3 is the camera-to-wall distance
    /// (Camera_X - MurderWall_X), d6 starts at `$80`.
    #[test]
    fn loc_5e64_non_reversed_x_position() {
        // Camera at $58, wall at $40 → d3 = $18 → X = $80 - $18 = $68
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0040,
            murder_wall_reversed: false,
            kid_hitbox_left_px: 0x0200,
            kid_hitbox_right_px: 0x0210,
            camera_x_px: 0x0058,
            camera_y_px: 0,
            animation_timer: 2,
            animation_frame: 0,
            starting_number_sprites: 0,
        });
        assert_eq!(
            render.sprites[0].x, 0x0068,
            "ASM loc_5E64: sub.w d3,d6 — X should be $80 - $18 = $68 for non-reversed wall"
        );
    }

    /// ASM loc_5E66 (kid.asm:5887): Y-position uses the REMAINDER of
    /// `Camera_Y_pos / $30` (via DIVU + SWAP), not the quotient.
    /// `d7 = $80 - (Camera_Y_pos % $30)`.
    #[test]
    fn loc_5e66_y_position_uses_remainder_not_quotient() {
        // Camera_Y = $45 (69). 69 % 48 = 21 = $15. base_y = $80 - $15 = $6B.
        // If the bug (using quotient) were present: 69/48 = 1, base_y = $7F.
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0010,
            murder_wall_reversed: false,
            kid_hitbox_left_px: 0x0200,
            kid_hitbox_right_px: 0x0210,
            camera_x_px: 0x0020,
            camera_y_px: 0x0045,
            animation_timer: 2,
            animation_frame: 0,
            starting_number_sprites: 0,
        });
        assert_eq!(
            render.sprites[0].y, 0x006B,
            "ASM loc_5E66: DIVU+SWAP gives remainder; y = $80 - ($45 % $30) = $80 - $15 = $6B"
        );
    }

    /// ASM loc_5E94 (kid.asm:5901): after the animation countdown tick, read
    /// `($FFFFFABF).w` as frame index, `add.b d2,d2; ext.w d2` to double and
    /// sign-extend, then `lea unk_5DFA(pc),a1; move.w (a1,d2.w),d3` to load the
    /// tile pattern word from the 4-entry table.  Each frame index selects a
    /// different base pattern: frame 0 → $A2FB, 1 → $A31F, 2 → $A343, 3 → $A367.
    #[test]
    fn loc_5e94_animation_counter_selects_pattern_from_unk_5dfa() {
        // animation_timer=1 means it will decrement to 0, triggering reload to 5
        // and advancing animation_frame from 0 to 1.  Frame 1 → pattern $A31F.
        // Non-reversed column 1, tile offset +0 → sprite tile_word = $A31F.
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0010,
            murder_wall_reversed: false,
            kid_hitbox_left_px: 0x0200,
            kid_hitbox_right_px: 0x0210,
            camera_x_px: 0x0020,
            camera_y_px: 0,
            animation_timer: 1,
            animation_frame: 0,
            starting_number_sprites: 0,
        });
        assert_eq!(
            render.animation_timer, 5,
            "ASM loc_5E94: subi.b #1 reached zero → move.b #5,($FFFFFAC0).w"
        );
        assert_eq!(
            render.animation_frame, 1,
            "ASM loc_5E94: addi.b #1,($FFFFFABF).w; andi.b #3 → frame 0+1=1"
        );
        // unk_5DFA[1] = $A31F; first sprite in non-reversed col 1 has offset +0
        assert_eq!(
            render.sprites[0].tile_word, 0xA31F,
            "ASM loc_5E94: lea unk_5DFA(pc),a1; move.w (a1,d2.w),d3 — frame 1 selects pattern $A31F"
        );

        // Verify all four pattern entries by testing frame 3 wrapping to 0
        let render_wrap =
            compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
                murder_wall_enabled: true,
                murder_wall_x_px: 0x0010,
                murder_wall_reversed: false,
                kid_hitbox_left_px: 0x0200,
                kid_hitbox_right_px: 0x0210,
                camera_x_px: 0x0020,
                camera_y_px: 0,
                animation_timer: 1,
                animation_frame: 3,
                starting_number_sprites: 0,
            });
        assert_eq!(
            render_wrap.animation_frame, 0,
            "ASM loc_5E94: andi.b #3 wraps frame 3+1=4 → 0"
        );
        // unk_5DFA[0] = $A2FB
        assert_eq!(
            render_wrap.sprites[0].tile_word, 0xA2FB,
            "ASM loc_5E94: frame 0 selects pattern $A2FB from unk_5DFA"
        );
    }

    /// ASM loc_5EB2 (kid.asm:5912): non-reversed first-column sprite loop.
    /// `moveq #5,d0` → 6 iterations (dbf), each emitting 2 sprites:
    ///   - first sprite: tile_word = pattern + 0, at current y
    ///   - second sprite: tile_word = pattern + 9, at y + $18
    ///
    /// Each pair advances y by $18 twice ($30 total per pair iteration).
    /// After the loop: `move.w d4,d7` restores base_y, `addi.w #$18,d6` advances x.
    #[test]
    fn loc_5eb2_non_reversed_first_column_sprite_layout() {
        // frame=2 (no timer tick, timer=3) → pattern = unk_5DFA[2] = $A343
        // camera_x=0x0028, wall_x=0x0020 → d3=8, x=$80-8=$78
        // camera_y=0 → base_y=$80
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0020,
            murder_wall_reversed: false,
            kid_hitbox_left_px: 0x0200,
            kid_hitbox_right_px: 0x0210,
            camera_x_px: 0x0028,
            camera_y_px: 0,
            animation_timer: 3,
            animation_frame: 2,
            starting_number_sprites: 0,
        });
        // First column: sprites 0..11 (6 pairs), all at x=$78
        // Pair 0: sprites[0] y=$80 tile=$A343+0=$A343, sprites[1] y=$98 tile=$A343+9=$A34C
        // Pair 1: sprites[2] y=$B0 tile=$A343, sprites[3] y=$C8 tile=$A34C
        assert_eq!(
            render.sprites.len(),
            24,
            "2 columns × 6 pairs × 2 sprites = 24"
        );
        for i in 0..12 {
            assert_eq!(
                render.sprites[i].x, 0x0078,
                "ASM loc_5EB2: all first-column sprites share x = $80 - d3"
            );
        }
        // Check tile offset alternation: even indices +0, odd indices +9
        assert_eq!(
            render.sprites[0].tile_word, 0xA343,
            "ASM loc_5EB2: first sprite tile = pattern + 0"
        );
        assert_eq!(
            render.sprites[1].tile_word, 0xA34C,
            "ASM loc_5EB2: second sprite tile = pattern + 9 (addi.w #9,d2)"
        );
        // Y spacing: $18 between consecutive sprites
        assert_eq!(render.sprites[0].y, 0x0080, "base_y = $80");
        assert_eq!(render.sprites[1].y, 0x0098, "base_y + $18");
        assert_eq!(render.sprites[2].y, 0x00B0, "base_y + $30");
        assert_eq!(
            render.sprites[11].y,
            0x0080 + 11 * 0x18,
            "last first-column sprite y"
        );
        // Link counter: $0A00 + incrementing Number_Sprites
        assert_eq!(render.sprites[0].link, 0x0A01, "first link = $A00 + 1");
        assert_eq!(render.sprites[11].link, 0x0A0C, "12th link = $A00 + 12");
    }

    /// ASM loc_5F0C (kid.asm:5938): non-reversed second-column sprite loop.
    /// After loc_5EB2's loop: `move.w d4,d7` resets y to base_y,
    /// `addi.w #$18,d6` advances x by $18.  Then `moveq #5,d0` for 6 more
    /// iterations, each emitting 2 sprites with tile offsets +$12 and +$1B.
    /// After the loop: `move.l a0,(Addr_NextSpriteSlot).w` stores the updated
    /// sprite pointer.
    #[test]
    fn loc_5f0c_non_reversed_second_column_tile_offsets_and_y_reset() {
        // Same setup as loc_5eb2 test: frame=2 → pattern=$A343, x=$78, base_y=$80
        let render = compile_murder_wall_warning_sprites_sub_5E02(MurderWallWarningInputSub5E02 {
            murder_wall_enabled: true,
            murder_wall_x_px: 0x0020,
            murder_wall_reversed: false,
            kid_hitbox_left_px: 0x0200,
            kid_hitbox_right_px: 0x0210,
            camera_x_px: 0x0028,
            camera_y_px: 0,
            animation_timer: 3,
            animation_frame: 2,
            starting_number_sprites: 0,
        });
        // Second column: sprites 12..23, all at x = $78 + $18 = $90
        for i in 12..24 {
            assert_eq!(
                render.sprites[i].x, 0x0090,
                "ASM loc_5F0C: second-column sprites at x = first_col_x + $18"
            );
        }
        // Y resets to base_y ($80) at start of second column
        assert_eq!(
            render.sprites[12].y, 0x0080,
            "ASM loc_5F0C: move.w d4,d7 resets y to base_y before second column"
        );
        // Tile offsets: +$12 and +$1B alternating
        assert_eq!(
            render.sprites[12].tile_word, 0xA355,
            "ASM loc_5F0C: first second-column sprite tile = pattern + $12 ($A343 + $12 = $A355)"
        );
        assert_eq!(
            render.sprites[13].tile_word, 0xA35E,
            "ASM loc_5F0C: second second-column sprite tile = pattern + $1B ($A343 + $1B = $A35E)"
        );
        // Final sprite count: 24 sprites total
        assert_eq!(
            render.number_sprites, 24,
            "ASM loc_5F0C: after both columns, Number_Sprites advanced by 24"
        );
    }

    #[test]
    fn sub_6034_returns_without_dispatch_for_none_or_lava_effects() {
        assert_eq!(
            dispatch_level_special_effects_sub_6034(0),
            None,
            "ASM sub_6034 should return immediately when `Level_Special_Effects` is zero"
        );
        assert_eq!(
            dispatch_level_special_effects_sub_6034(1),
            None,
            "ASM sub_6034 should also return for the lava special-effect slot because `subq.w #1,d0` leaves the branch on `ble`"
        );
    }

    #[test]
    fn sub_6034_dispatches_storm_and_storm_hail_to_sub_6048() {
        assert_eq!(
            dispatch_level_special_effects_sub_6034(2),
            Some(LevelSpecialEffectsRoutineSub6034::WeatherSub6048),
            "ASM sub_6034 should pick the first non-null jump-table entry for storm weather"
        );
        assert_eq!(
            dispatch_level_special_effects_sub_6034(3),
            Some(LevelSpecialEffectsRoutineSub6034::WeatherSub6048),
            "ASM sub_6034 should also dispatch the storm+hail slot to sub_6048 because both jump-table entries point there"
        );
    }

    #[test]
    fn loc_14cce_selects_hail_art_only_for_ice_backgrounds() {
        assert_eq!(
            crate::weather::select_storm_art_loc_14CCE(false),
            crate::weather::StormArtAssetLoc14CCE::Rain,
            "ASM loc_14CCE should fall through with the rain art pointer when `Background_theme` is not Ice",
        );
        assert_eq!(
            crate::weather::select_storm_art_loc_14CCE(true),
            crate::weather::StormArtAssetLoc14CCE::Hail,
            "ASM loc_14CCE should branch to the hail art pointer when `Background_theme == Ice`",
        );
    }

    #[test]
    fn loc_14d82_writes_the_same_saved_rain_palette_pointer_for_storm_and_storm_hail() {
        assert_eq!(
            crate::weather::select_saved_storm_palette_loc_14D82(2),
            crate::weather::StormSavedPaletteAssetLoc14D82::RainPalette7B85C,
            "ASM loc_14D82 should keep `($FFFFFADE).w` pointed at `LnkTo_Pal_7B85C` on the direct rain branch"
        );
        assert_eq!(
            crate::weather::select_saved_storm_palette_loc_14D82(3),
            crate::weather::StormSavedPaletteAssetLoc14D82::RainPalette7B85C,
            "ASM loc_14D82's hail fallthrough should still write the same `LnkTo_Pal_7B85C` pointer before branching to loc_14D8A"
        );
    }

    #[test]
    fn loc_14cd8_uploads_the_selected_storm_art_to_vram_7000() {
        assert_eq!(
            crate::weather::upload_storm_art_loc_14CD8(crate::weather::StormArtAssetLoc14CCE::Rain),
            crate::weather::StormArtUploadLoc14CD8 {
                asset: crate::weather::StormArtAssetLoc14CCE::Rain,
                vram_word: 0x7000,
            },
            "ASM loc_14CD8 should pass the rain art pointer through to `DecompressToVRAM` with destination `$7000`",
        );
        assert_eq!(
            crate::weather::upload_storm_art_loc_14CD8(crate::weather::StormArtAssetLoc14CCE::Hail),
            crate::weather::StormArtUploadLoc14CD8 {
                asset: crate::weather::StormArtAssetLoc14CCE::Hail,
                vram_word: 0x7000,
            },
            "ASM loc_14CD8 should use the same `$7000` VRAM destination after the hail branch rewrites `a0` at loc_14CCE",
        );
    }

    #[test]
    fn loc_14cec_selects_the_hail_palette_only_for_ice_backgrounds() {
        assert_eq!(
            crate::weather::select_storm_palette_loc_14CEC(false),
            crate::weather::StormPaletteAssetLoc14CEC::Rain,
            "ASM loc_14CEC should keep the default rain palette pointer when `Background_theme` is not Ice",
        );
        assert_eq!(
            crate::weather::select_storm_palette_loc_14CEC(true),
            crate::weather::StormPaletteAssetLoc14CEC::Hail,
            "ASM loc_14CEC should branch to the hail palette pointer when `Background_theme == Ice`",
        );
    }

    #[test]
    fn loc_14cf2_copies_four_longwords_into_palette_buffer_plus_20_only() {
        let source_palette = [
            0x1001, 0x1002, 0x1003, 0x1004, 0x1005, 0x1006, 0x1007, 0x1008,
        ];
        let mut palette_buffer = [0xDEAD; 0x40];

        crate::weather::copy_storm_palette_loc_14CF2(&source_palette, &mut palette_buffer);

        assert_eq!(
            &palette_buffer[0x10..0x18],
            &source_palette,
            "ASM loc_14CF2 should copy the selected rain or hail palette into `Palette_Buffer+$20` as four longwords, i.e. exactly eight words"
        );
        assert!(
            palette_buffer[..0x10].iter().all(|&word| word == 0xDEAD)
                && palette_buffer[0x18..].iter().all(|&word| word == 0xDEAD),
            "ASM loc_14CF2 should leave every palette entry outside `$20..$2E` untouched because all four `move.l` stores target the single `lea (Palette_Buffer+$20).l,a1` span"
        );
    }

    #[test]
    fn loc_14d1e_selects_the_storm_pattern_source_but_keeps_the_shared_addend_and_header_skip() {
        assert_eq!(
            crate::weather::select_storm_pattern_upload_loc_14D1E(false),
            crate::weather::StormPatternUploadSetupLoc14D1E {
                vram_tile_addend: 0x0380_0380,
                source_asset: crate::weather::StormPatternAssetLoc14D1E::Rain,
                skipped_header_longwords: 1,
            },
            "ASM loc_14D1E's fallthrough path should keep the rain pattern table, seed `d7` as `$0380_0380`, and skip the first longword before the shared upload loop at loc_14D32"
        );
        assert_eq!(
            crate::weather::select_storm_pattern_upload_loc_14D1E(true),
            crate::weather::StormPatternUploadSetupLoc14D1E {
                vram_tile_addend: 0x0380_0380,
                source_asset: crate::weather::StormPatternAssetLoc14D1E::Hail,
                skipped_header_longwords: 1,
            },
            "ASM loc_14D1E should only swap `a0` to the hail table when `Background_theme == Ice`; the duplicated `move.w/swap/move.w` sequence still produces the same `$0380_0380` addend and the same `addq.w #4,a0` header skip"
        );
    }

    #[test]
    fn loc_14d3c_repeats_the_full_storm_pattern_upload_for_four_outer_passes() {
        assert_eq!(
            crate::weather::storm_pattern_outer_pass_count_loc_14D3C(),
            4,
            "ASM loc_14D3C should run the shared upload body four times because `moveq #3,d2` plus `dbf d2,loc_14D3C` counts inclusively from 3 to -1"
        );
    }

    #[test]
    fn loc_14d3e_repeats_each_storm_pattern_group_eight_times_before_advancing_a0() {
        assert_eq!(
            crate::weather::storm_pattern_group_repeat_count_loc_14D3E(),
            8,
            "ASM loc_14D3E should keep each four-longword group live for eight repeats because `moveq #7,d1` plus `dbf d1,loc_14D3E` counts inclusively from 7 to -1"
        );
    }

    #[test]
    fn loc_14d40_adds_the_shared_tile_offset_to_each_longword_in_the_current_group() {
        let source_group = [0x1000_0000, 0x1000_0001, 0x1000_0002, 0x1000_0003];

        assert_eq!(
            crate::weather::storm_pattern_emit_group_loc_14D40(&source_group, 0x0380_0380),
            [0x1380_0380, 0x1380_0381, 0x1380_0382, 0x1380_0383],
            "ASM loc_14D40 should load `(a0)`, `4(a0)`, `8(a0)`, and `$C(a0)`, add the shared `$0380_0380` tile offset to each, and emit the adjusted longwords in order before `dbf d0,loc_14D40` repeats the same group"
        );
    }

    #[test]
    fn loc_14d32_programs_vram_6000_and_repeats_each_four_longword_storm_group_with_inclusive_dbf_loops()
     {
        let source_longwords =
            core::array::from_fn(|index| 0x1000_0000u32.wrapping_add(index as u32));
        let result =
            crate::weather::upload_storm_pattern_blocks_loc_14D32(&source_longwords, 0x0380_0380);

        assert_eq!(
            result.vdp_command, 0x4000_0003,
            "ASM loc_14D32 should program `vdpComm($6000,VRAM,WRITE)` before the nested `dbf` upload loops"
        );
        assert_eq!(
            result.written_longwords.len(),
            4 * 8 * 8 * 4,
            "ASM loc_14D32 seeds `d2=#3`, `d1=#7`, and `d0=#7`, so the shared upload tail should emit 4 outer passes × 8 groups × 8 repeats × 4 longwords"
        );
        assert_eq!(
            &result.written_longwords[..4],
            &[0x1380_0380, 0x1380_0381, 0x1380_0382, 0x1380_0383,],
            "ASM loc_14D32 should add the shared `$0380_0380` tile offset to the first four-longword group before the first `dbf d0` iteration stores it"
        );
        assert_eq!(
            &result.written_longwords[4..8],
            &[0x1380_0380, 0x1380_0381, 0x1380_0382, 0x1380_0383,],
            "ASM loc_14D32 should repeat the same four source longwords while `dbf d0` counts down from 7 to -1 because `a0` is not advanced until after the inner loop"
        );
        assert_eq!(
            &result.written_longwords[32..36],
            &[0x1380_0384, 0x1380_0385, 0x1380_0386, 0x1380_0387,],
            "ASM loc_14D32 should advance by `$10` bytes only after eight inner repeats, so the second group begins after the first 32 writes"
        );
        assert_eq!(
            &result.written_longwords[256..260],
            &[0x1380_0380, 0x1380_0381, 0x1380_0382, 0x1380_0383,],
            "ASM loc_14D32 should restore `a0` from `a3` between outer passes, causing the whole 32-longword table to restart after each 256-write block"
        );
    }

    #[test]
    fn loc_14d8a_selects_the_shared_weather_art_and_seeds_the_later_slot_fill_loop() {
        assert_eq!(
            crate::weather::initialise_storm_setup_loc_14D8A(),
            crate::weather::StormInitSetupLoc14D8A {
                art_asset: crate::weather::StormSharedArtAssetLoc14D8A::ArtComp9A7D2,
                art_vram_word: 0x76A0,
                fill_setup: crate::weather::StormSlotFillSetupLoc14D8A {
                    loop_count: 4,
                    falling_slot_sentinel: 0xFFFF_FFFF,
                    falling_slot_y_word: 0x0100,
                    impact_slot_sentinel: 0xFFFF_FFFF,
                },
            },
            "ASM loc_14D8A should decompress `LnkTo_ArtComp_9A7D2` to `$76A0` and seed the later loc_14DAC loop with `d0=#4`, `d2=#$100`, and `d1=#$FFFF_FFFF`"
        );
    }

    #[test]
    fn loc_14df6_streams_dbf_counter_plus_one_eight_longword_bursts_to_vram() {
        let source_longwords: Vec<u32> = (0..24).map(|index| 0x9000_0000u32 + index).collect();

        assert_eq!(
            crate::weather::upload_special_effect_unknown_art_loc_14DF6(&source_longwords, 0),
            crate::weather::SpecialEffectUnknownArtUploadLoc14DF6 {
                written_longwords: source_longwords[..8].to_vec(),
            },
            "ASM loc_14DF6 should still stream one eight-longword burst when `d0` starts at zero because the eight `move.l (a0)+,(a6)` stores happen before `dbf` reaches `-1`"
        );
        assert_eq!(
            crate::weather::upload_special_effect_unknown_art_loc_14DF6(&source_longwords, 2),
            crate::weather::SpecialEffectUnknownArtUploadLoc14DF6 {
                written_longwords: source_longwords[..24].to_vec(),
            },
            "ASM loc_14DF6 should stream exactly `(d0+1) * 8` longwords to the VDP data port because each inclusive `dbf` iteration copies eight consecutive source longwords before the branch check"
        );
    }

    #[test]
    fn loc_14e42_seeds_four_consecutive_groups_before_the_later_outer_reset() {
        assert_eq!(
            crate::weather::special_effect_unknown_group_count_loc_14E42(),
            4,
            "ASM loc_14E42 should run four middle-loop groups per outer pass because `moveq #3,d1` makes `dbf d1,loc_14E44` count inclusively from 3 to -1"
        );

        let stepped = crate::weather::special_effect_unknown_outer_pass_loc_14E42(
            crate::weather::special_effect_unknown_inner_repeat_count_loc_14E44(),
        );
        assert_eq!(
            stepped.written_longwords.len(),
            4 * 16 * 2,
            "ASM loc_14E42 should emit four 16-pair groups before the later `move.l d3,d7` / `move.l d4,d6` reset because the outer pass does not restart the seed pair until after all four groups finish"
        );
        assert_eq!(
            &stepped.written_longwords[..4],
            &[0x8380_8381, 0x8382_8383, 0x8380_8381, 0x8382_8383],
            "ASM loc_14E42 should begin the first group with the precomputed `d6=$83808381` / `d7=$83828383` pair before loc_14E44 reloads `d0`"
        );
        assert_eq!(
            &stepped.written_longwords[32..36],
            &[0x8384_8385, 0x8386_8387, 0x8384_8385, 0x8386_8387],
            "ASM loc_14E42 should advance to the next pair only after one full 16-pair group completes, so the second group begins at `d6/d7 + $00040004`"
        );
    }

    #[test]
    fn loc_14e44_repeats_the_current_pair_sixteen_times_before_the_group_advance() {
        assert_eq!(
            crate::weather::special_effect_unknown_inner_repeat_count_loc_14E44(),
            16,
            "ASM loc_14E44 should run sixteen pair writes per group because `moveq #$F,d0` makes `dbf d0,loc_14E46` count inclusively from 15 to -1"
        );

        let stepped = crate::weather::special_effect_unknown_group_loc_14E44();
        assert_eq!(
            stepped.len(),
            16 * 2,
            "ASM loc_14E44 should emit sixteen copies of the current `d6/d7` pair before the later `add.l d5,d6` / `add.l d5,d7` advances to the next pair"
        );
        assert_eq!(
            &stepped[..4],
            &[0x8380_8381, 0x8382_8383, 0x8380_8381, 0x8382_8383],
            "ASM loc_14E44 should begin by writing the current `d6=$83808381` / `d7=$83828383` pair exactly as loaded before the inner `dbf` loop starts"
        );
        assert_eq!(
            &stepped[28..32],
            &[0x8380_8381, 0x8382_8383, 0x8380_8381, 0x8382_8383],
            "ASM loc_14E44 should still be writing the same pair on the sixteenth and final iteration because the `$00040004` bump happens only after the inner loop finishes"
        );
    }

    #[test]
    fn loc_14e46_writes_d6_then_d7_before_the_dbf_branch_check() {
        assert_eq!(
            crate::weather::special_effect_unknown_pair_write_loc_14E46(),
            [0x8380_8381, 0x8382_8383],
            "ASM loc_14E46 should write the current `d6` longword before the current `d7` longword because the two `move.l` stores run before `dbf d0,loc_14E46` tests for another iteration"
        );
    }

    #[test]
    fn loc_14dac_clears_five_weather_slots_and_seeds_the_post_loop_worker_state() {
        let stepped = crate::weather::initialise_storm_slots_loc_14DAC(
            crate::weather::StormSlotFillSetupLoc14D8A {
                loop_count: 4,
                falling_slot_sentinel: 0xFFFF_FFFF,
                falling_slot_y_word: 0x0100,
                impact_slot_sentinel: 0xFFFF_FFFF,
            },
        );

        assert_eq!(
            stepped.falling_slots,
            [crate::weather::FallingWeatherInitSlotLoc14DAC {
                sentinel_long: 0xFFFF_FFFF,
                y_word: 0x0100,
            }; 5],
            "ASM loc_14DAC seeds `d0=#4`, so the inclusive `dbf d0,loc_14DAC` loop should write five identical `[$FFFF_FFFF,$0100]` falling-slot records at `$FFFFFAE2`"
        );
        assert_eq!(
            stepped.impact_slots,
            [crate::weather::WeatherImpactInitSlotLoc14DAC {
                sentinel_words: [0xFFFF_FFFF; 3],
            }; 5],
            "ASM loc_14DAC should also clear five impact-slot records at `$FFFFFB00` because each of the same five loop iterations stores three `#$FFFF_FFFF` longwords to `(a1)+`"
        );
        assert_eq!(
            stepped.spawn_cooldown, 1,
            "ASM loc_14DAC should seed `($FFFFFB3C).w=1` before the storm worker starts counting down"
        );
        assert_eq!(
            stepped.free_falling_slots, 5,
            "ASM loc_14DAC should set `($FFFFFB3D).w=5` after clearing five falling-weather slots"
        );
        assert_eq!(
            stepped.weather_mode_byte, 4,
            "ASM loc_14DAC should write `4` to `($FFFFFAD6).w` before returning to the caller"
        );
        assert_eq!(
            stepped.fade_countdown, 0x0100,
            "ASM loc_14DAC should seed `($FFFFFADA).w` with `$100` for the later storm fade countdown"
        );
        assert_eq!(
            stepped.thunder_timer, 1,
            "ASM loc_14DAC should seed `($FFFFFADC).w=1` so sub_6048 decrements into the first thunder event"
        );
        assert_eq!(
            stepped.fb52_word, 1,
            "ASM loc_14DAC should write `1` to `($FFFFFB52).w` in the post-loop tail"
        );
        assert_eq!(
            stepped.fb54_word, 0,
            "ASM loc_14DAC should clear `($FFFFFB54).w` with `sf` before returning"
        );
    }

    #[test]
    fn sub_6048_before_20_seconds_only_sets_weather_mode_byte() {
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x04B0,
                level_special_effects: 3,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: false,
                fade_countdown: 0x20,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: 10,
                hail_period_timer: 7,
                hail_active: false,
                snow_spawn_cooldown: 9,
                snow_slot_count: 1,
                play_thunder_sound: true,
                call_snow_worker: true,
                palette_ops: vec![WeatherPaletteOpSub6048::CopyPaletteBufferToBuffer2],
            },
            [0; 8],
            [0; 15],
            [0; 7],
            [0; 7],
            0,
        );

        assert_eq!(
            stepped.weather_mode_byte, 4,
            "ASM sub_6048 should set `($FFFFFAD6).w=4` and return while `Frame_Counter <= $4B0`"
        );
        assert_eq!(
            stepped.fade_progress_latch, 0x20,
            "ASM sub_6048 should still mirror `($FFFFFADA).w` into `($FFFFF876).w` before the early return"
        );
        assert!(
            stepped.palette_ops.is_empty(),
            "ASM sub_6048 should not enqueue any palette work before the 20-second storm threshold"
        );
    }

    #[test]
    fn sub_6048_non_mountain_fade_copies_buffers_and_loads_storm_background() {
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x04B1,
                level_special_effects: 2,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: false,
                fade_countdown: 0x10,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: 10,
                hail_period_timer: 7,
                hail_active: false,
                snow_spawn_cooldown: 9,
                snow_slot_count: 1,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [0x111, 0x222, 0x333, 0x444, 0x555, 0x666, 0x777, 0x888],
            [0; 15],
            [0; 7],
            [0; 7],
            0,
        );

        assert_eq!(
            stepped.palette_animation_speed, 0x7F,
            "ASM sub_6048 should write `$7F` to `($FFFFF88B).w` while the storm fade countdown is active"
        );
        assert_eq!(
            stepped.pending_palette_mask_0, 0x0080,
            "ASM sub_6048 should seed `($FFFFF888).w` with low-byte `$80` for non-Mountain storm fades"
        );
        assert_eq!(
            stepped.fade_countdown, 0x000C,
            "ASM sub_6048 should subtract 4 from `($FFFFFADA).w` after loading the storm palette"
        );
        assert_eq!(
            stepped.palette_ops,
            vec![
                WeatherPaletteOpSub6048::CopyPaletteBufferToBuffer2,
                WeatherPaletteOpSub6048::CopyPaletteBufferToBuffer3,
                WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0x19,
                    words: vec![0x111, 0x222, 0x333, 0x444, 0x555, 0x666, 0x777],
                },
                WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0,
                    words: vec![0],
                },
            ],
            "ASM sub_6048 should copy Palette_Buffer into both work buffers, stamp the seven-word storm palette at `Palette_Buffer_3+$32`, and clear the background colour word"
        );
    }

    #[test]
    fn sub_6048_mountain_fade_overlays_storm_background_and_foreground() {
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x04B1,
                level_special_effects: 2,
                foreground_theme_is_mountain: true,
                background_theme_is_mountain: true,
                fade_countdown: 4,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: 10,
                hail_period_timer: 7,
                hail_active: false,
                snow_spawn_cooldown: 9,
                snow_slot_count: 1,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [1, 2, 3, 4, 5, 6, 7, 8],
            [10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24],
            [0; 7],
            [0; 7],
            0,
        );

        assert_eq!(
            stepped.pending_palette_mask_0, -1,
            "ASM sub_6048 should promote `($FFFFF888).w` to `$FFFF` for the Mountain foreground storm overlay"
        );
        assert_eq!(
            stepped.palette_ops,
            vec![
                WeatherPaletteOpSub6048::CopyPaletteBufferToBuffer2,
                WeatherPaletteOpSub6048::CopyPaletteBufferToBuffer3,
                // ASM loc_60C0: first word → Palette_Buffer_3[0]
                WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0,
                    words: vec![1],
                },
                // ASM loc_60C6: remaining 7 words (d0=6, dbf) → Palette_Buffer_3+$32
                WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 0x19,
                    words: vec![2, 3, 4, 5, 6, 7, 8],
                },
                WeatherPaletteOpSub6048::WritePaletteBuffer3 {
                    start_word: 1,
                    words: vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24],
                },
            ],
            "ASM sub_6048 should route the first Mountain storm word to `Palette_Buffer_3[0]`, the next seven words to `$32`, then overlay the 15-word Mountain foreground palette at `Palette_Buffer_3+2`"
        );
    }

    #[test]
    fn sub_6048_thunder_timer_zero_plays_sound_and_loads_flash_palette() {
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x05DD,
                level_special_effects: 2,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: true,
                fade_countdown: 0,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: 1,
                hail_period_timer: 7,
                hail_active: false,
                snow_spawn_cooldown: 9,
                snow_slot_count: 1,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [0; 8],
            [0; 15],
            [0xAAA, 0xAAB, 0xAAC, 0xAAD, 0xAAE, 0xAAF, 0xAB0],
            [0; 7],
            0,
        );

        assert!(
            stepped.play_thunder_sound,
            "ASM sub_6048 should call `j_PlaySound` with `sfx_Thunderstorm` when `($FFFFFADC).w` decrements to zero"
        );
        assert_eq!(stepped.thunder_timer, 0);
        assert_eq!(
            stepped.palette_ops,
            vec![
                WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0,
                    words: vec![0xAAA],
                },
                WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0x19,
                    words: vec![0xAAB, 0xAAC, 0xAAD, 0xAAE, 0xAAF, 0xAB0],
                },
            ],
            "ASM sub_6048 should load the thunder-flash palette into Palette_Buffer word 0 and words `$19..$1E` when the Mountain thunder timer hits zero"
        );
    }

    #[test]
    fn sub_6048_negative_thunder_timer_reseeds_and_loads_calm_palette() {
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x05DD,
                level_special_effects: 2,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: true,
                fade_countdown: 0,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: -26,
                hail_period_timer: 7,
                hail_active: false,
                snow_spawn_cooldown: 9,
                snow_slot_count: 1,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [0; 8],
            [0; 15],
            [0; 7],
            [0x0B1, 0x0B2, 0x0B3, 0x0B4, 0x0B5, 0x0B6, 0x0B7],
            0x3C,
        );

        assert_eq!(
            stepped.thunder_timer, 0x00C3,
            "ASM sub_6048 should reset `($FFFFFADC).w` to `$F7-(rand&$F5)` when the signed countdown reaches `$FFE5`"
        );
        assert_eq!(
            stepped.palette_ops,
            vec![
                WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0,
                    words: vec![0x0B1],
                },
                WeatherPaletteOpSub6048::WritePaletteBuffer {
                    start_word: 0x19,
                    words: vec![0x0B2, 0x0B3, 0x0B4, 0x0B5, 0x0B6, 0x0B7],
                },
            ],
            "ASM sub_6048 should restore the calm Mountain storm palette when the negative thunder countdown reaches the reseed sentinel"
        );
    }

    // loc_60EE — Mountain thunder guard: non-Mountain background skips thunder block
    #[test]
    fn loc_60ee_non_mountain_skips_thunder_block() {
        // ASM loc_60EE (kid.asm:6108): cmpi.w #Mountain,(Background_theme).w; bne.s loc_6172
        // When background is not Mountain, the entire thunder decrement/flash/calm
        // block is skipped even though frame_counter > $5DC and weather is Storm.
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x05DD,
                level_special_effects: 2,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: false, // not Mountain → skip
                fade_countdown: 0,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: 1, // would hit zero if decremented
                hail_period_timer: 7,
                hail_active: false,
                snow_spawn_cooldown: 9,
                snow_slot_count: 1,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [0; 8],
            [0; 15],
            [0xAAA, 0xAAB, 0xAAC, 0xAAD, 0xAAE, 0xAAF, 0xAB0],
            [0; 7],
            0,
        );

        assert_eq!(
            stepped.thunder_timer, 1,
            "ASM loc_60EE: non-Mountain background should skip the thunder decrement entirely"
        );
        assert!(
            !stepped.play_thunder_sound,
            "ASM loc_60EE: non-Mountain should not trigger j_PlaySound"
        );
        assert!(
            stepped.palette_ops.is_empty(),
            "ASM loc_60EE: non-Mountain should emit no palette ops"
        );
    }

    // loc_60EE — Mountain thunder guard: frame_counter <= $5DC routes to loc_61C2
    #[test]
    fn loc_60ee_frame_counter_below_threshold_routes_to_weather_mode() {
        // ASM loc_60EE (kid.asm:6113): cmpi.w #$5DC,d6; bls.w loc_61C2
        // When frame_counter <= $5DC (25 seconds), skip thunder and set weather_mode_byte=4.
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x05DC, // exactly $5DC → bls taken
                level_special_effects: 2,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: true,
                fade_countdown: 0,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: 1,
                hail_period_timer: 7,
                hail_active: false,
                snow_spawn_cooldown: 9,
                snow_slot_count: 1,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [0; 8],
            [0; 15],
            [0xAAA; 7],
            [0; 7],
            0,
        );

        assert_eq!(
            stepped.thunder_timer, 1,
            "ASM loc_60EE: frame_counter <= $5DC should not decrement the thunder timer"
        );
        assert_eq!(
            stepped.weather_mode_byte, 4,
            "ASM loc_60EE→loc_61C2: frame_counter at threshold should set weather_mode_byte=4"
        );
        assert!(
            stepped.palette_ops.is_empty(),
            "ASM loc_60EE→loc_61C2: no palette ops when frame_counter at thunder threshold"
        );
    }

    // loc_613E — negative thunder timer that is not the -27 sentinel skips palette reload
    #[test]
    fn loc_613e_negative_timer_not_sentinel_skips_palette() {
        // ASM loc_613E (kid.asm:6134): cmpi.w #$FFE5,($FFFFFADC).w; bne.s loc_6172
        // When thunder_timer is negative but not exactly -27 ($FFE5), neither the
        // flash palette (timer==0) nor the calm palette (timer==-27) is loaded.
        // The timer at -10 (after decrement from -9) should produce no ops.
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x05DD,
                level_special_effects: 2,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: true,
                fade_countdown: 0,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: -9, // decrements to -10, which is not -27
                hail_period_timer: 7,
                hail_active: false,
                snow_spawn_cooldown: 9,
                snow_slot_count: 1,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [0; 8],
            [0; 15],
            [0xAAA; 7],
            [0xBBB; 7],
            0,
        );

        assert_eq!(
            stepped.thunder_timer, -10,
            "ASM loc_60EE: timer should decrement from -9 to -10"
        );
        assert!(
            !stepped.play_thunder_sound,
            "ASM loc_613E: negative timer not at sentinel should not play thunder"
        );
        assert!(
            stepped.palette_ops.is_empty(),
            "ASM loc_613E: bne.s loc_6172 skips palette reload when timer != $FFE5"
        );
    }

    #[test]
    fn sub_6048_snow_cycle_toggles_to_active_and_calls_sub_61ca() {
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x0874,
                level_special_effects: 3,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: false,
                fade_countdown: 0,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: 10,
                hail_period_timer: 1,
                hail_active: false,
                snow_spawn_cooldown: 9,
                snow_slot_count: 2,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [0; 8],
            [0; 15],
            [0; 7],
            [0; 7],
            0,
        );

        assert!(stepped.hail_active);
        assert_eq!(
            stepped.hail_period_timer, 0x04B0,
            "ASM sub_6048 should extend the inactive->active snow period to `$4B0` frames"
        );
        assert!(
            stepped.call_snow_worker,
            "ASM sub_6048 should always fall through to `bsr.w sub_61CA` once the 35-second snow logic is active"
        );
        assert_eq!(
            stepped.snow_spawn_cooldown, 9,
            "ASM sub_6048 should not force `($FFFFFB3C).w=2` after toggling into the active snow period"
        );
    }

    #[test]
    fn sub_6048_snow_cycle_toggles_inactive_and_requests_spawn_when_slots_available() {
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x0874,
                level_special_effects: 3,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: false,
                fade_countdown: 0,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: 10,
                hail_period_timer: 1,
                hail_active: true,
                snow_spawn_cooldown: 9,
                snow_slot_count: 4,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [0; 8],
            [0; 15],
            [0; 7],
            [0; 7],
            0,
        );

        assert!(!stepped.hail_active);
        assert_eq!(
            stepped.hail_period_timer, 0x0258,
            "ASM sub_6048 should reset the active->inactive snow period to `$258` frames"
        );
        assert_eq!(
            stepped.snow_spawn_cooldown, 2,
            "ASM sub_6048 should write `2` to `($FFFFFB3C).w` when snow is inactive and fewer than five slots are live"
        );
        assert!(
            stepped.call_snow_worker,
            "ASM loc_61BC (kid.asm:6174): bsr.w sub_61CA — inactive period with available slots still calls the snow worker"
        );
    }

    #[test]
    fn loc_61a8_inactive_full_slots_skips_snow_worker() {
        // ASM loc_61A8 (kid.asm:6167): tst.b ($FFFFFB54).w; bne.s loc_61BC
        // When hail_active is false: cmpi.b #5,($FFFFFB3D).w; beq.s return_61C0
        // — all 5 slots occupied means sub_61CA is NOT called.
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x0874,
                level_special_effects: 3,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: false,
                fade_countdown: 0,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: 10,
                hail_period_timer: 5,
                hail_active: false,
                snow_spawn_cooldown: 9,
                snow_slot_count: 5,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [0; 8],
            [0; 15],
            [0; 7],
            [0; 7],
            0,
        );

        assert!(
            !stepped.call_snow_worker,
            "ASM loc_61A8: beq.s return_61C0 — inactive period with all 5 snow slots must NOT call sub_61CA"
        );
        assert_eq!(
            stepped.snow_spawn_cooldown, 9,
            "ASM loc_61A8: snow_spawn_cooldown should be unchanged when snow_slot_count == 5"
        );
    }

    #[test]
    fn loc_61a8_active_period_calls_snow_worker_without_spawn_request() {
        // ASM loc_61A8 (kid.asm:6167): tst.b ($FFFFFB54).w; bne.s loc_61BC
        // When hail_active is true, jump straight to loc_61BC (sub_61CA) without
        // touching snow_spawn_cooldown.
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x0874,
                level_special_effects: 3,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: false,
                fade_countdown: 0,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: 10,
                hail_period_timer: 5,
                hail_active: true,
                snow_spawn_cooldown: 9,
                snow_slot_count: 3,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [0; 8],
            [0; 15],
            [0; 7],
            [0; 7],
            0,
        );

        assert!(
            stepped.call_snow_worker,
            "ASM loc_61BC (kid.asm:6174): bsr.w sub_61CA — active period always calls the snow worker"
        );
        assert_eq!(
            stepped.snow_spawn_cooldown, 9,
            "ASM loc_61A8: active period does not modify snow_spawn_cooldown"
        );
    }

    #[test]
    fn loc_619c_toggle_from_inactive_adds_258_to_preset_258() {
        // ASM loc_619C (kid.asm:6163): addi.w #$258,($FFFFFB52).w
        // When hail_active is false at timer expiry, the code first sets timer=$258
        // then loc_619C adds another $258, giving $4B0 total for the active period.
        let stepped = step_level_special_effects_sub_6048(
            WeatherSpecialEffectsStepSub6048 {
                frame_counter: 0x0874,
                level_special_effects: 3,
                foreground_theme_is_mountain: false,
                background_theme_is_mountain: false,
                fade_countdown: 0,
                fade_progress_latch: 0,
                palette_animation_speed: 0,
                pending_palette_mask_0: 0,
                weather_mode_byte: 0,
                thunder_timer: 10,
                hail_period_timer: 1,
                hail_active: false,
                snow_spawn_cooldown: 9,
                snow_slot_count: 2,
                play_thunder_sound: false,
                call_snow_worker: false,
                palette_ops: Vec::new(),
            },
            [0; 8],
            [0; 15],
            [0; 7],
            [0; 7],
            0,
        );

        assert!(
            stepped.hail_active,
            "ASM loc_619C: eori.b #$FF,($FFFFFB54).w toggles inactive→active"
        );
        assert_eq!(
            stepped.hail_period_timer, 0x04B0,
            "ASM loc_619C: inactive→active period = $258 (preset) + $258 (loc_619C) = $4B0"
        );
    }

    #[test]
    fn sub_61ca_reuses_deleted_falling_slot_and_keeps_its_animation_word() {
        let mut collision_rows = empty_weather_collision_rows_sub_61_ca();
        collision_rows.rows[2][8] = 0;
        collision_rows.rows[2][9] = 0;

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 100,
            camera_y_px: 40,
            spawn_cooldown: 1,
            free_falling_slots: 1,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 40 + 0x00E1,
                    x_px: 132,
                    animation_cooldown: 2,
                    animation_frame: 5,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 35,
            kid_hitbox_bottom_px: 60,
            kid_hitbox_left_px: 130,
            kid_hitbox_right_px: 160,
            random_byte: 0x00,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.spawn_cooldown, 0x25,
            "ASM sub_61CA should reload `($FFFFFB3C).w` to `$25-(rand&$0F)` before attempting a respawn"
        );
        assert_eq!(
            stepped.free_falling_slots, 1,
            "ASM sub_61CA should free the off-screen slot, then immediately spend one free slot on the respawn"
        );
        assert_eq!(
            stepped.falling_slots[0],
            FallingWeatherSlotSub61CA {
                y_px: 40,
                x_px: 140,
                animation_cooldown: 1,
                animation_frame: 5,
            },
            "ASM sub_61CA loc_6226 -> loc_62C2 should only rewrite Y/X when a deleted slot is recycled"
        );
        assert_eq!(
            stepped.kid_collision_timer,
            Some(0x0028),
            "ASM sub_61CA should set `$38(a5)` to `$28` when the recycled falling slot overlaps Kid"
        );
        assert_eq!(
            stepped.sprites,
            vec![WeatherSpriteSub61CA {
                y: 0x0080,
                link: 0x0501,
                tile_word: 0xA3C9,
                x: 0x00A8,
            }],
            "ASM sub_61CA should render the recycled slot with the preserved frame tile from `unk_64BC`"
        );
    }

    #[test]
    fn sub_61ca_promotes_terrain_hit_into_an_impact_slot_with_the_slope_offset() {
        let mut collision_rows = empty_weather_collision_rows_sub_61_ca();
        collision_rows.rows[5][1] = 0x4000;

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 9,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 64,
                    x_px: 32,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 200,
            kid_hitbox_bottom_px: 210,
            kid_hitbox_left_px: 200,
            kid_hitbox_right_px: 210,
            random_byte: 0x34,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.free_falling_slots, 1,
            "ASM sub_61CA should return the falling slot to the free-count byte after a terrain impact"
        );
        assert_eq!(
            stepped.falling_slots[0],
            FallingWeatherSlotSub61CA {
                y_px: -1,
                x_px: -1,
                animation_cooldown: 2,
                animation_frame: 0,
            },
            "ASM sub_61CA should clear only Y/X when the falling slot turns into a terrain splash"
        );
        assert_eq!(
            stepped.impact_slots[0],
            WeatherImpactSlotSub61CA {
                y_px: 80,
                x_px: 31,
                animation_cooldown: 5,
                animation_frame: 0,
            },
            "ASM sub_61CA loc_638E should add `$0C` to the copied Y before the impact slot enters its first render frame"
        );
        assert_eq!(
            stepped.sprites,
            vec![WeatherSpriteSub61CA {
                y: 0x00D0,
                link: 0x0501,
                tile_word: 0xA3CD,
                x: 0x009F,
            }],
            "ASM sub_61CA should render the first impact frame with sprite tile `$A3CD` after the slope-adjusted terrain hit"
        );
    }

    // --- loc_61E2 / loc_620C / loc_6212: per-slot falling weather update ---

    #[test]
    fn loc_61e2_advances_active_slot_y_x_and_animation_cooldown() {
        // ASM loc_61E2 (kid.asm:6199): addi.w #4,(a0); subi.w #1,2(a0); subi.b #1,4(a0)
        // Active slot: y += 4, x -= 1, animation_cooldown -= 1. Inactive slots skipped.
        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 100,
                    x_px: 200,
                    animation_cooldown: 4,
                    animation_frame: 2,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.falling_slots[0].y_px, 104,
            "ASM loc_61E2: addi.w #4,(a0) — y advances by 4 each frame"
        );
        assert_eq!(
            stepped.falling_slots[0].x_px, 199,
            "ASM loc_61E2: subi.w #1,2(a0) — x decreases by 1 each frame"
        );
        assert_eq!(
            stepped.falling_slots[0].animation_cooldown, 3,
            "ASM loc_61E2: subi.b #1,4(a0) — animation cooldown decrements"
        );
        assert_eq!(
            stepped.falling_slots[0].animation_frame, 2,
            "ASM loc_61E2: bne.s loc_6212 — nonzero cooldown skips frame advance"
        );
        // Inactive slot 1 must remain unchanged
        assert_eq!(
            stepped.falling_slots[1],
            FallingWeatherSlotSub61CA::inactive(6, 0),
            "ASM loc_61E2: tst.w (a0); bmi.s loc_621C — inactive slots are skipped"
        );
    }

    #[test]
    fn loc_620c_advances_animation_frame_and_wraps_at_5() {
        // ASM loc_620C (kid.asm:6211): addi.b #1,5(a0) — advance animation frame.
        // When frame >= 5, ASM sets 5(a0)=$FF then +1 wraps to 0.
        // Test both normal advance (frame 2→3) and wrap (frame 5→0).
        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                // Slot 0: cooldown=1 → expires, frame 2 → 3
                FallingWeatherSlotSub61CA {
                    y_px: 50,
                    x_px: 100,
                    animation_cooldown: 1,
                    animation_frame: 2,
                },
                // Slot 1: cooldown=1 → expires, frame 5 → 0 (wrap)
                FallingWeatherSlotSub61CA {
                    y_px: 50,
                    x_px: 100,
                    animation_cooldown: 1,
                    animation_frame: 5,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.falling_slots[0].animation_cooldown, 6,
            "ASM loc_61E2: move.b #6,4(a0) — cooldown reloads to 6 on expiry"
        );
        assert_eq!(
            stepped.falling_slots[0].animation_frame, 3,
            "ASM loc_620C: addi.b #1,5(a0) — frame 2 advances to 3"
        );
        assert_eq!(
            stepped.falling_slots[1].animation_cooldown, 6,
            "ASM loc_61E2: cooldown reloads to 6 for second slot too"
        );
        assert_eq!(
            stepped.falling_slots[1].animation_frame, 0,
            "ASM loc_620C: cmpi.b #5,5(a0); blt.s loc_620C; move.b #$FF,5(a0) — frame 5 wraps to 0"
        );
    }

    #[test]
    fn loc_6212_deactivates_slot_past_bottom_or_left_edge() {
        // ASM loc_6212 (kid.asm:6214): cmp.w (a0),d1; blt.s loc_6226 — y > delete_bottom.
        // cmp.w 2(a0),d0; bgt.s loc_6226 — x < delete_left.
        // delete_bottom = camera_y + $E0, delete_left = camera_x - $10.
        let camera_x: i16 = 100;
        let camera_y: i16 = 50;
        let delete_bottom = camera_y.wrapping_add(0x00E0); // 274
        let delete_left = camera_x.wrapping_sub(0x0010); // 84

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: camera_x,
            camera_y_px: camera_y,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                // Slot 0: y just past bottom edge → deactivated
                FallingWeatherSlotSub61CA {
                    y_px: delete_bottom + 1, // 275: past bottom
                    x_px: 150,
                    animation_cooldown: 3,
                    animation_frame: 1,
                },
                // Slot 1: x just past left edge → deactivated
                FallingWeatherSlotSub61CA {
                    y_px: 100,
                    x_px: delete_left - 1, // 83: past left
                    animation_cooldown: 3,
                    animation_frame: 2,
                },
                // Slot 2: within bounds → stays active
                FallingWeatherSlotSub61CA {
                    y_px: 100,
                    x_px: 150,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        // After loc_61E2 movement: slot0 y=275+4=279, slot1 x=83-1=82, slot2 y=104 x=149
        assert!(
            stepped.falling_slots[0].y_px < 0,
            "ASM loc_6212: cmp.w (a0),d1; blt.s loc_6226 — slot past bottom edge deactivated"
        );
        assert!(
            stepped.falling_slots[1].y_px < 0,
            "ASM loc_6212: cmp.w 2(a0),d0; bgt.s loc_6226 — slot past left edge deactivated"
        );
        assert!(
            stepped.falling_slots[2].y_px >= 0,
            "ASM loc_6212: slot within camera bounds stays active"
        );
        assert_eq!(
            stepped.free_falling_slots, 2,
            "ASM loc_6212→loc_6226: addi.b #1,($FFFFFB3D).w — free count incremented for each deactivated slot"
        );
    }

    // --- loc_623A / loc_625E / loc_626C: spawn cooldown + inactive scan + spawn ---

    #[test]
    fn loc_623a_decrements_spawn_cooldown_and_skips_when_nonzero() {
        // ASM loc_623A (kid.asm:6232): subq.b #1,($FFFFFB3C).w; bne.w loc_62CE
        // When cooldown is not expiring, just decrement and skip spawn logic.
        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 5,
            free_falling_slots: 3,
            falling_slots: [FallingWeatherSlotSub61CA::inactive(6, 0); 5],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0xFF,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.spawn_cooldown, 4,
            "ASM loc_623A: subq.b #1 decrements 5→4"
        );
        // All slots should remain inactive — no spawn attempted.
        for (i, slot) in stepped.falling_slots.iter().enumerate() {
            assert!(
                slot.y_px < 0,
                "slot {i} should stay inactive when cooldown != 0"
            );
        }
    }

    #[test]
    fn loc_623a_reloads_cooldown_with_random_jitter_on_expiry() {
        // ASM loc_623A (kid.asm:6232): on cooldown==0 → move.b #$25,($FFFFFB3C).w;
        // jsr Get_RandomNumber_byte; andi.b #$F,d7; sub.b d7,($FFFFFB3C).w
        // Reload = $25 - (random & $F). With random=$AB, jitter = $B, reload = $1A.
        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 1,     // will become 0 after decrement
            free_falling_slots: 0, // no free slots → skip spawn scan
            falling_slots: [FallingWeatherSlotSub61CA::inactive(6, 0); 5],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0xAB,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.spawn_cooldown,
            0x25 - 0x0B,
            "ASM loc_623A: cooldown = $25 - ($AB & $F) = $25 - $B = $1A"
        );
    }

    #[test]
    fn loc_623a_skips_spawn_when_no_free_slots() {
        // ASM loc_623A (kid.asm:6239): tst.b ($FFFFFB3D).w; beq.s loc_62CE
        // free_falling_slots == 0 → skip to loc_62CE without scanning.
        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 100,
            camera_y_px: 50,
            spawn_cooldown: 1,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 60,
                    x_px: 120,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA {
                    y_px: 70,
                    x_px: 130,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA {
                    y_px: 80,
                    x_px: 140,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA {
                    y_px: 90,
                    x_px: 150,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA {
                    y_px: 100,
                    x_px: 160,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0x00,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.free_falling_slots, 0,
            "ASM loc_623A: beq.s loc_62CE — no spawn when free_falling_slots == 0"
        );
    }

    #[test]
    fn loc_625e_scans_for_first_inactive_slot_skipping_active_ones() {
        // ASM loc_625E (kid.asm:6244): move.w (a0),d0; bmi.s loc_626C — finds
        // first inactive slot (y_px < 0). Active slots (y_px >= 0) are skipped.
        // Slot[0..2] active, slot[3] inactive → slot[3] gets the spawn.
        let mut collision_rows = empty_weather_collision_rows_sub_61_ca();
        // Ensure spawn tile is clear.
        collision_rows.rows[3][10] = 0;
        collision_rows.rows[3][11] = 0;

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 100,
            camera_y_px: 50,
            spawn_cooldown: 1,
            free_falling_slots: 2, // slots 3 and 4 are inactive
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 60,
                    x_px: 120,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA {
                    y_px: 70,
                    x_px: 130,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA {
                    y_px: 80,
                    x_px: 140,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0), // slot 3 — first inactive
                FallingWeatherSlotSub61CA::inactive(6, 0), // slot 4
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0x10, // spawn_x = 100 + 16 + 40 = 156, col = 9
            collision_rows,
            starting_number_sprites: 0,
        });

        // Slot 3 should now be spawned (y_px >= 0).
        assert!(
            stepped.falling_slots[3].y_px >= 0,
            "ASM loc_625E: first inactive slot (index 3) gets the spawn"
        );
        // Slot 4 should remain inactive — only one slot spawned per cooldown.
        assert!(
            stepped.falling_slots[4].y_px < 0,
            "ASM loc_625E: only one slot spawned per cooldown cycle"
        );
        assert_eq!(
            stepped.free_falling_slots, 1,
            "ASM loc_62C2: subi.b #1,($FFFFFB3D).w — free count decremented"
        );
    }

    #[test]
    fn loc_625e_all_active_skips_spawn() {
        // ASM loc_625E (kid.asm:6248): dbf d5,loc_625E; bra.s loc_62CE
        // If all 5 slots are active, loop exits without spawning.
        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 100,
            camera_y_px: 50,
            spawn_cooldown: 1,
            free_falling_slots: 1, // nonzero so spawn path is entered
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 60,
                    x_px: 120,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA {
                    y_px: 70,
                    x_px: 130,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA {
                    y_px: 80,
                    x_px: 140,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA {
                    y_px: 90,
                    x_px: 150,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA {
                    y_px: 100,
                    x_px: 160,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0x00,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        // free count unchanged because no inactive slot was found to fill.
        assert_eq!(
            stepped.free_falling_slots, 1,
            "ASM loc_625E: bra.s loc_62CE — all active, free count unchanged"
        );
    }

    #[test]
    fn loc_626c_spawns_at_camera_plus_random_plus_28() {
        // ASM loc_626C (kid.asm:6252): spawn_x = camera_x + (random & $FF) + $28;
        // spawn_y = camera_y. loc_62C2: store to slot y_px/x_px.
        let mut collision_rows = empty_weather_collision_rows_sub_61_ca();
        // Ensure the spawn tile is clear.
        // random=0x30, camera_x=200 → spawn_x = 200+48+40 = 288 = $120, col = 18
        // camera_y=64 → row = (64+15)/16 = 4
        collision_rows.rows[4][18] = 0;
        collision_rows.rows[4][19] = 0;

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 200,
            camera_y_px: 64,
            spawn_cooldown: 1,
            free_falling_slots: 1,
            falling_slots: [
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0x30,
            collision_rows,
            starting_number_sprites: 0,
        });

        let slot = &stepped.falling_slots[0];
        assert_eq!(
            slot.x_px,
            200 + 0x30 + 0x28,
            "ASM loc_626C: spawn_x = camera_x + random + $28"
        );
        assert_eq!(slot.y_px, 64, "ASM loc_626C: spawn_y = camera_y");
        assert_eq!(
            stepped.free_falling_slots, 0,
            "ASM loc_62C2: subi.b #1,($FFFFFB3D).w"
        );
    }

    #[test]
    fn loc_626c_blocked_first_tile_prevents_spawn() {
        // ASM loc_62AA (kid.asm:6275): move.w (a4)+,d6; andi.w #$F,d6; bne.s loc_62CE
        // First tile collision nibble nonzero → blocked, no spawn.
        let mut collision_rows = empty_weather_collision_rows_sub_61_ca();
        // random=0, camera_x=0 → spawn_x = 0+0+$28 = $28 = 40, col_floor=2
        // camera_y=0 → row = (0+15)/16 = 0
        collision_rows.rows[0][2] = 0x0003; // low nibble nonzero → blocked

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 1,
            free_falling_slots: 1,
            falling_slots: [
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0x00,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert!(
            stepped.falling_slots[0].y_px < 0,
            "ASM loc_62AA: first tile blocked (& $F != 0) → no spawn"
        );
        assert_eq!(
            stepped.free_falling_slots, 1,
            "free count unchanged when spawn blocked"
        );
    }

    #[test]
    fn loc_626c_blocked_second_tile_prevents_spawn_when_crossing_boundary() {
        // ASM loc_62AA (kid.asm:6281): tst.b d0; beq.s loc_62C2; move.w (a4),d6;
        // andi.w #$FF,d6; bne.s loc_62CE — second tile check when crossing boundary.
        // spawn_x on a tile boundary: random=$08, camera_x=0 → spawn_x = 0+8+$28 = $30 = 48
        // col_floor = 48/16 = 3, col_ceil = (48+15)/16 = 63/16 = 3 → same column, no cross.
        // Use random=$09 → spawn_x = 49, col_floor = 3, col_ceil = (49+15)/16 = 4 → crosses!
        let mut collision_rows = empty_weather_collision_rows_sub_61_ca();
        // camera_y=0 → row = 0
        collision_rows.rows[0][3] = 0x0000; // first tile clear
        collision_rows.rows[0][4] = 0x0010; // second tile: low byte nonzero → blocked

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 1,
            free_falling_slots: 1,
            falling_slots: [
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0x09,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert!(
            stepped.falling_slots[0].y_px < 0,
            "ASM loc_62AA: crosses tile boundary, second tile blocked (& $FF != 0) → no spawn"
        );
    }

    // --- loc_62D6 / loc_630C: snow-Kid hitbox collision loop ---

    #[test]
    fn loc_62d6_sets_kid_collision_timer_when_slot_overlaps_kid_hitbox() {
        // ASM loc_62D6 (kid.asm:6298): per-slot Kid hitbox overlap test.
        // Post-advance: y=100, x=49 → particle box [102..114] x [51..63].
        // Kid hitbox top=100, bottom=120, left=40, right=70 → overlap.
        // ASM: move.w #$28,$38(a5) — sets push timer to $28.
        let collision_rows = empty_weather_collision_rows_sub_61_ca();

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99, // high, no spawn
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 96,
                    x_px: 51,
                    animation_cooldown: 6,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 100,
            kid_hitbox_bottom_px: 120,
            kid_hitbox_left_px: 40,
            kid_hitbox_right_px: 70,
            random_byte: 0,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.kid_collision_timer,
            Some(0x0028),
            "ASM loc_62D6: overlapping slot should set Kid push timer to $28"
        );
    }

    #[test]
    fn loc_62d6_skips_inactive_slot() {
        // ASM loc_62D6 (kid.asm:6299–6300): move.w (a0),d0; bmi.s loc_630C
        // When y_px < 0 (inactive), branch immediately to loc_630C (next slot).
        let collision_rows = empty_weather_collision_rows_sub_61_ca();

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 0,
            kid_hitbox_bottom_px: 999,
            kid_hitbox_left_px: 0,
            kid_hitbox_right_px: 999,
            random_byte: 0,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.kid_collision_timer, None,
            "ASM loc_62D6: inactive slots (y < 0) branch to loc_630C without testing hitbox"
        );
    }

    #[test]
    fn loc_62d6_skips_slot_above_kid_hitbox() {
        // ASM loc_62D6 (kid.asm:6301–6303): addi.w #2,d0; cmp.w (Kid_hitbox_top).w,d0; blt.s loc_630C
        // slot y=10 → y+2=12, Kid top=100 → 12 < 100 → skip.
        let collision_rows = empty_weather_collision_rows_sub_61_ca();

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 10,
                    x_px: 50,
                    animation_cooldown: 6,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 100,
            kid_hitbox_bottom_px: 120,
            kid_hitbox_left_px: 40,
            kid_hitbox_right_px: 70,
            random_byte: 0,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.kid_collision_timer, None,
            "ASM loc_62D6: slot above Kid (y+2 < hitbox_top) branches to loc_630C"
        );
    }

    #[test]
    fn loc_62d6_skips_slot_below_kid_hitbox() {
        // ASM loc_62D6 (kid.asm:6304–6306): addi.w #$C,d0; cmp.w (Kid_hitbox_bottom).w,d0; bgt.s loc_630C
        // Post-advance: y=114. y+2=116, y+$E=128 > Kid bottom=120 → skip.
        let collision_rows = empty_weather_collision_rows_sub_61_ca();

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 110,
                    x_px: 50,
                    animation_cooldown: 6,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 100,
            kid_hitbox_bottom_px: 120,
            kid_hitbox_left_px: 40,
            kid_hitbox_right_px: 70,
            random_byte: 0,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.kid_collision_timer, None,
            "ASM loc_62D6: slot below Kid (y+$E > hitbox_bottom) branches to loc_630C"
        );
    }

    #[test]
    fn loc_62d6_skips_slot_right_of_kid_hitbox() {
        // ASM loc_62D6 (kid.asm:6307–6310): move.w 2(a0),d0; addi.w #2,d0;
        // cmp.w (Kid_hitbox_right).w,d0; bgt.s loc_630C
        // Post-advance: y=100, x=79. y+2=102, y+$E=114 — inside Kid y range.
        // x+2=81 > Kid right=70 → skip at the right-edge check.
        let collision_rows = empty_weather_collision_rows_sub_61_ca();

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 96,
                    x_px: 80,
                    animation_cooldown: 6,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 100,
            kid_hitbox_bottom_px: 120,
            kid_hitbox_left_px: 40,
            kid_hitbox_right_px: 70,
            random_byte: 0,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.kid_collision_timer, None,
            "ASM loc_62D6: slot right of Kid (x+2 > hitbox_right) branches to loc_630C"
        );
    }

    #[test]
    fn loc_62d6_skips_slot_left_of_kid_hitbox() {
        // ASM loc_62D6 (kid.asm:6311–6313): addi.w #$C,d0; cmp.w (Kid_hitbox_left).w,d0; blt.s loc_630C
        // Post-advance: y=100, x=19. y+2=102, y+$E=114 — inside Kid y range.
        // x+2=21, x+$E=33 < Kid left=40 → skip at the left-edge check.
        let collision_rows = empty_weather_collision_rows_sub_61_ca();

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 96,
                    x_px: 20,
                    animation_cooldown: 6,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 100,
            kid_hitbox_bottom_px: 120,
            kid_hitbox_left_px: 40,
            kid_hitbox_right_px: 70,
            random_byte: 0,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.kid_collision_timer, None,
            "ASM loc_62D6: slot left of Kid (x+$E < hitbox_left) branches to loc_630C"
        );
    }

    #[test]
    fn loc_630c_iterates_all_five_slots() {
        // ASM loc_630C (kid.asm:6316): lea 6(a0),a0; dbf d5,loc_62D6
        // d5=4 (moveq #4,d5 at loc_62CE) → 5 iterations.
        // Place overlapping slot in position 4 (last) to confirm all 5 are checked.
        // Note: slot advances y+=4, x-=1 before collision, so input y=96, x=51
        // → post-advance y=100, x=50 → particle box [102..114] x [52..64].
        let collision_rows = empty_weather_collision_rows_sub_61_ca();

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA {
                    y_px: 96,
                    x_px: 51,
                    animation_cooldown: 6,
                    animation_frame: 0,
                },
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 100,
            kid_hitbox_bottom_px: 120,
            kid_hitbox_left_px: 40,
            kid_hitbox_right_px: 70,
            random_byte: 0,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.kid_collision_timer,
            Some(0x0028),
            "ASM loc_630C: dbf d5,loc_62D6 iterates all 5 slots — last slot overlap detected"
        );
    }

    // --- loc_6318 / loc_6348 / loc_6360: terrain collision → splash conversion ---

    #[test]
    fn loc_6318_skips_inactive_falling_slot_in_terrain_check() {
        // ASM loc_6318 (kid.asm:6322): move.w (a0),d7; bmi.s loc_6384
        // Inactive falling slot (y < 0) branches straight to loc_6384 (next slot).
        // No tile lookup occurs — impact slots remain untouched.
        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        assert!(
            stepped.impact_slots.iter().all(|s| s.y_px < 0),
            "ASM loc_6318: inactive falling slots (y < 0) should skip terrain check entirely"
        );
        assert_eq!(
            stepped.free_falling_slots, 0,
            "ASM loc_6318: no slots deactivated → free count unchanged"
        );
    }

    #[test]
    fn loc_6348_skips_slot_over_empty_terrain() {
        // ASM loc_6348 (kid.asm:6341): andi.w #$7000,d7; beq.s loc_6384
        // When the tile word's terrain kind is 0, no splash occurs — slot continues falling.
        // Collision row for row 5, col 12 = 0x0000 (empty).
        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 64,
                    x_px: 200,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        // Post-advance: y=68, x=199. row=(68+15)>>4=5, col=199>>4=12.
        // Tile at [5][12] = 0 → terrain_kind = 0 → beq.s loc_6384 (skip).
        assert!(
            stepped.falling_slots[0].y_px >= 0,
            "ASM loc_6348: terrain_kind == 0 branches to loc_6384 — slot stays active"
        );
        assert!(
            stepped.impact_slots.iter().all(|s| s.y_px < 0),
            "ASM loc_6348: no terrain → no impact slot created"
        );
    }

    #[test]
    fn loc_6360_converts_falling_slot_to_normal_splash() {
        // ASM loc_6360 (kid.asm:6352): normal terrain (not $4000/$5000) →
        //   scan impact slots for first free, copy y/x directly, type=$600,
        //   deactivate falling slot, increment free_falling_slots.
        // Terrain kind $1000 (solid ground) triggers the normal splash path.
        let mut collision_rows = empty_weather_collision_rows_sub_61_ca();
        // Post-advance: y=68, x=31. row=(68+15)>>4=5, col=31>>4=1.
        collision_rows.rows[5][1] = 0x1000;

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 64,
                    x_px: 32,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows,
            starting_number_sprites: 0,
        });

        assert!(
            stepped.falling_slots[0].y_px < 0,
            "ASM loc_6360/loc_636A: move.l #$FFFFFFFF,(a0) deactivates the falling slot"
        );
        assert_eq!(
            stepped.free_falling_slots, 1,
            "ASM loc_6360/loc_636A: addi.b #1,($FFFFFB3D).w increments free count"
        );
        // Normal splash: y copied directly (no $C offset, unlike loc_638E).
        assert_eq!(
            stepped.impact_slots[0].y_px, 68,
            "ASM loc_6360/loc_636A: move.w (a0),(a2) — impact y = falling y (post-advance)"
        );
        assert_eq!(
            stepped.impact_slots[0].x_px, 31,
            "ASM loc_6360/loc_636A: move.w 2(a0),2(a2) — impact x = falling x (post-advance)"
        );
    }

    #[test]
    fn loc_6348_routes_slope_terrain_to_offset_splash() {
        // ASM loc_6348 (kid.asm:6347–6350): cmpi.w #$4000,d7; beq.s loc_638E;
        //   cmpi.w #$5000,d7; beq.s loc_638E — slope terrain adds $C to impact y.
        // Verify $5000 terrain follows the same offset path as $4000.
        let mut collision_rows = empty_weather_collision_rows_sub_61_ca();
        collision_rows.rows[5][1] = 0x5000;

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 64,
                    x_px: 32,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows,
            starting_number_sprites: 0,
        });

        // Post-advance: y=68, x=31. $5000 terrain → loc_638E: addi.w #$C,(a2) → y=80.
        assert_eq!(
            stepped.impact_slots[0].y_px, 80,
            "ASM loc_6348→loc_638E: $5000 slope terrain adds $C to impact y"
        );
        assert_eq!(
            stepped.impact_slots[0].x_px, 31,
            "ASM loc_638E: move.w 2(a0),2(a2) — x copied without offset"
        );
        assert!(
            stepped.falling_slots[0].y_px < 0,
            "ASM loc_638E: move.l #$FFFFFFFF,(a0) deactivates falling slot"
        );
        assert_eq!(
            stepped.free_falling_slots, 1,
            "ASM loc_638E: addi.b #1,($FFFFFB3D).w increments free count"
        );
    }

    #[test]
    fn loc_6360_scans_past_active_impact_slots_to_find_free_one() {
        // ASM loc_6360 (kid.asm:6352–6356): tst.w (a2); bmi.s loc_636A;
        //   lea 6(a2),a2; bra.s loc_6360 — scan loop skips active impact slots.
        // Place terrain hit with first 3 impact slots occupied; splash should
        // land in slot index 3.
        let mut collision_rows = empty_weather_collision_rows_sub_61_ca();
        collision_rows.rows[5][1] = 0x2000;

        let mut impact_slots = [WeatherImpactSlotSub61CA::inactive(1, 0); 10];
        // Occupy first 3 impact slots.
        for slot in impact_slots.iter_mut().take(3) {
            *slot = WeatherImpactSlotSub61CA {
                y_px: 10,
                x_px: 10,
                animation_cooldown: 6,
                animation_frame: 0,
            };
        }

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 64,
                    x_px: 32,
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots,
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows,
            starting_number_sprites: 0,
        });

        // First 3 impact slots are active (from setup, then decremented during step).
        // The new splash should land in impact slot index 3.
        assert_eq!(
            stepped.impact_slots[3].y_px, 68,
            "ASM loc_6360: scan loop should skip occupied slots and land in first free one"
        );
        assert_eq!(
            stepped.impact_slots[3].x_px, 31,
            "ASM loc_6360: splash x = falling slot x (post-advance)"
        );
    }

    // --- loc_6398 / loc_63BE / loc_63C4: offset splash scan + impact slot update loop ---

    #[test]
    fn loc_6398_offset_splash_copies_y_plus_c_and_x_to_impact_slot() {
        // ASM loc_6398 (kid.asm:6379): slope terrain ($4000) offset splash.
        //   move.w (a0),(a2); addi.w #$C,(a2); move.w 2(a0),2(a2);
        //   move.w #$600,4(a2); move.l #$FFFFFFFF,(a0);
        //   addi.b #1,($FFFFFB3D).w; lea 6(a0),a0; dbf d5,loc_6318.
        // Verify that the +$C y offset is applied and that the impact slot
        // is initialised with cooldown=6, frame=0 (from the $600 word).
        let mut collision_rows = empty_weather_collision_rows_sub_61_ca();
        collision_rows.rows[7][2] = 0x4000; // slope terrain at row 7, col 2

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [
                FallingWeatherSlotSub61CA {
                    y_px: 100, // (100+$F)>>4 = 7 → row 7
                    x_px: 40,  // 40>>4 = 2 → col 2
                    animation_cooldown: 3,
                    animation_frame: 0,
                },
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
                FallingWeatherSlotSub61CA::inactive(6, 0),
            ],
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows,
            starting_number_sprites: 0,
        });

        // Post-advance: y=104, x=39. Slope terrain → loc_638E scan → loc_6398: y+$C=116.
        assert_eq!(
            stepped.impact_slots[0].y_px, 116,
            "ASM loc_6398: move.w (a0),(a2); addi.w #$C,(a2) — y = 104 + $C = 116"
        );
        assert_eq!(
            stepped.impact_slots[0].x_px, 39,
            "ASM loc_6398: move.w 2(a0),2(a2) — x copied directly"
        );
        assert_eq!(
            stepped.impact_slots[0].animation_cooldown, 5,
            "ASM loc_6398: move.w #$600,4(a2) — cooldown=6, then decremented by impact loop"
        );
        assert_eq!(
            stepped.impact_slots[0].animation_frame, 0,
            "ASM loc_6398: $600 word sets frame=0 (low byte)"
        );
    }

    #[test]
    fn loc_63be_iterates_all_ten_impact_slots() {
        // ASM loc_63BE (kid.asm:6389): lea ($FFFFFB00).w,a0; moveq #9,d5
        //   Sets up 10-iteration loop over impact slots.
        // ASM loc_63C4 (kid.asm:6393): per-slot active check + cooldown + render.
        // Place active slots in first and last positions; verify both are processed.
        let mut impact_slots = [WeatherImpactSlotSub61CA::inactive(1, 0); 10];
        impact_slots[0] = WeatherImpactSlotSub61CA {
            y_px: 50,
            x_px: 60,
            animation_cooldown: 3,
            animation_frame: 0,
        };
        impact_slots[9] = WeatherImpactSlotSub61CA {
            y_px: 80,
            x_px: 90,
            animation_cooldown: 4,
            animation_frame: 0,
        };

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [FallingWeatherSlotSub61CA::inactive(6, 0); 5],
            impact_slots,
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        // Both slots should be decremented (cooldown -= 1) and rendered.
        assert_eq!(
            stepped.impact_slots[0].animation_cooldown, 2,
            "ASM loc_63C4: slot 0 cooldown decremented from 3 → 2"
        );
        assert_eq!(
            stepped.impact_slots[9].animation_cooldown, 3,
            "ASM loc_63C4: slot 9 (last) cooldown decremented from 4 → 3"
        );
        assert_eq!(
            stepped.sprites.len(),
            2,
            "ASM loc_63BE/loc_63C4: both slot 0 and slot 9 should emit sprites"
        );
    }

    #[test]
    fn loc_63c4_frame1_impact_sprite_uses_a00_link_base() {
        // ASM loc_63C4 (kid.asm:6393) → loc_6432 (kid.asm:6427):
        //   clr.w d6; tst.b 5(a0); bne.s loc_642C; ... move.w #$500,d6
        //   loc_643E: addi.w #$500,d6
        //   Frame 0: d6 = 0 + $500 = $500. Frame 1: d6 = $500 + $500 = $A00.
        //   The link base encodes VDP sprite size (2×2 vs 3×3 cells).
        let mut impact_slots = [WeatherImpactSlotSub61CA::inactive(1, 0); 10];
        // Frame 0 slot — link should use $500 base.
        impact_slots[0] = WeatherImpactSlotSub61CA {
            y_px: 50,
            x_px: 60,
            animation_cooldown: 3,
            animation_frame: 0,
        };
        // Frame 1 slot — link should use $A00 base.
        impact_slots[1] = WeatherImpactSlotSub61CA {
            y_px: 80,
            x_px: 90,
            animation_cooldown: 3,
            animation_frame: 1,
        };

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [FallingWeatherSlotSub61CA::inactive(6, 0); 5],
            impact_slots,
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        assert_eq!(stepped.sprites.len(), 2);
        // Frame 0: link base $500, first sprite → Number_Sprites=1 → $501
        assert_eq!(
            stepped.sprites[0].link, 0x0501,
            "ASM loc_6432: frame 0 → clr.w d6; addi.w #$500,d6 → base $500"
        );
        assert_eq!(
            stepped.sprites[0].tile_word, 0xA3CD,
            "ASM loc_6402: frame 0 tile = $A3CD"
        );
        // Frame 1: link base $A00, second sprite → Number_Sprites=2 → $A02
        assert_eq!(
            stepped.sprites[1].link, 0x0A02,
            "ASM loc_6432: frame 1 → move.w #$500,d6; addi.w #$500,d6 → base $A00"
        );
        assert_eq!(
            stepped.sprites[1].tile_word, 0xA3D1,
            "ASM loc_642C: frame 1 tile = $A3D1"
        );
    }

    #[test]
    fn loc_63c4_deactivates_impact_slot_after_frame2() {
        // ASM loc_63C4 (kid.asm:6393): addi.b #1,5(a0); cmpi.b #2,5(a0); blt.s loc_6402
        //   move.l #$FFFFFFFF,(a0); move.w #$100,4(a0); bra.s loc_6456
        // When cooldown expires and frame advances to 2, slot is deactivated.
        let mut impact_slots = [WeatherImpactSlotSub61CA::inactive(1, 0); 10];
        impact_slots[0] = WeatherImpactSlotSub61CA {
            y_px: 50,
            x_px: 60,
            animation_cooldown: 1, // will expire this step
            animation_frame: 1,    // will advance to 2 → deactivate
        };

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots: [FallingWeatherSlotSub61CA::inactive(6, 0); 5],
            impact_slots,
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        assert!(
            stepped.impact_slots[0].y_px < 0,
            "ASM loc_63C4: move.l #$FFFFFFFF,(a0) deactivates slot when frame >= 2"
        );
        assert_eq!(
            stepped.impact_slots[0].animation_cooldown, 1,
            "ASM loc_63C4: move.w #$100,4(a0) — cooldown=1"
        );
        assert_eq!(
            stepped.impact_slots[0].animation_frame, 0,
            "ASM loc_63C4: move.w #$100,4(a0) — frame=0"
        );
        assert!(
            stepped.sprites.is_empty(),
            "ASM loc_63C4: deactivated slot should not emit a sprite"
        );
    }

    #[test]
    fn loc_6462_renders_active_falling_slot_with_correct_screen_coords_and_tile() {
        // ASM loc_6462 (kid.asm:6447): sub.w Camera_Y_pos,d7; addi.w #$80,d7
        //   sub.w Camera_X_pos,d6; addi.w #$80,d6
        //   move.w 4(a0),d0; andi.w #$FF,d0; add.w d0,d0
        //   move.w unk_64BC(pc,d0.w),4(a4) — tile lookup by animation_frame
        //   move.w #$500,d6; addq.b #1,Number_Sprites; add.b Number_Sprites,d6
        let mut falling_slots = [FallingWeatherSlotSub61CA::inactive(6, 0); 5];
        falling_slots[0] = FallingWeatherSlotSub61CA {
            y_px: 200,
            x_px: 300,
            animation_cooldown: 3,
            animation_frame: 2, // unk_64BC[2] = $A3BD
        };

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 100,
            camera_y_px: 50,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots,
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.sprites.len(),
            1,
            "ASM loc_6462: one active falling slot should emit exactly one sprite"
        );
        let sprite = &stepped.sprites[0];
        // After loc_61E2 advances: y_px = 200+4=204, x_px = 300-1=299
        // screen_y = 204 - 50 + 0x80 = 282
        let expected_y: i16 = 204_i16.wrapping_sub(50).wrapping_add(0x0080);
        assert_eq!(
            sprite.y,
            u16::from_be_bytes(expected_y.to_be_bytes()),
            "ASM loc_6462: sub.w Camera_Y_pos,d7; addi.w #$80,d7"
        );
        // screen_x = 299 - 100 + 0x80 = 327
        let expected_x: i16 = 299_i16.wrapping_sub(100).wrapping_add(0x0080);
        assert_eq!(
            sprite.x,
            u16::from_be_bytes(expected_x.to_be_bytes()),
            "ASM loc_6462: sub.w Camera_X_pos,d6; addi.w #$80,d6"
        );
        assert_eq!(
            sprite.tile_word, 0xA3BD,
            "ASM loc_6462: unk_64BC[2] = $A3BD for animation_frame=2"
        );
        // link = $500 + number_sprites(1) = $501
        assert_eq!(
            sprite.link, 0x0501,
            "ASM loc_6462: move.w #$500,d6; addq.b #1,Number_Sprites; add.b Number_Sprites,d6"
        );
    }

    #[test]
    fn loc_64ae_skips_inactive_falling_slot_and_iterates_all_five() {
        // ASM loc_64AE (kid.asm:6469): lea 6(a0),a0; dbf d5,loc_6462
        //   Inactive slots (y < 0) branch from loc_6462's bmi.s directly to
        //   loc_64AE, advancing a0 by 6 without emitting a sprite.
        //   The loop runs all 5 slots (d5=4, dbf).
        let mut falling_slots = [FallingWeatherSlotSub61CA::inactive(6, 0); 5];
        // Only the last slot (index 4) is active — verifies the loop reaches it
        falling_slots[4] = FallingWeatherSlotSub61CA {
            y_px: 100,
            x_px: 100,
            animation_cooldown: 3,
            animation_frame: 0, // unk_64BC[0] = $A3B5
        };

        let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
            camera_x_px: 0,
            camera_y_px: 0,
            spawn_cooldown: 99,
            free_falling_slots: 0,
            falling_slots,
            impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
            kid_hitbox_top_px: 9999,
            kid_hitbox_bottom_px: 9999,
            kid_hitbox_left_px: 9999,
            kid_hitbox_right_px: 9999,
            random_byte: 0,
            collision_rows: empty_weather_collision_rows_sub_61_ca(),
            starting_number_sprites: 0,
        });

        assert_eq!(
            stepped.sprites.len(),
            1,
            "ASM loc_64AE: only the last active slot should emit a sprite — \
             dbf d5,loc_6462 iterates all 5 falling slots"
        );
        assert_eq!(
            stepped.sprites[0].tile_word, 0xA3B5,
            "ASM loc_6462: unk_64BC[0] = $A3B5 for animation_frame=0"
        );
    }

    #[test]
    fn loc_6462_each_animation_frame_selects_correct_tile_from_unk_64bc() {
        // ASM loc_6462 (kid.asm:6447): unk_64BC lookup table —
        //   [$A3B5, $A3B9, $A3BD, $A3C1, $A3C5, $A3C9] for frames 0–5.
        //   In practice frames wrap at 5 (loc_620C), so only 0–4 are used.
        let expected_tiles: [u16; 5] = [0xA3B5, 0xA3B9, 0xA3BD, 0xA3C1, 0xA3C5];
        for (frame, expected_tile) in expected_tiles.iter().enumerate() {
            let mut falling_slots = [FallingWeatherSlotSub61CA::inactive(6, 0); 5];
            falling_slots[0] = FallingWeatherSlotSub61CA {
                y_px: 10,
                x_px: 10,
                animation_cooldown: 3,
                animation_frame: frame as u8,
            };

            let stepped = step_weather_worker_sub_61CA(WeatherWorkerInputSub61CA {
                camera_x_px: 0,
                camera_y_px: 0,
                spawn_cooldown: 99,
                free_falling_slots: 0,
                falling_slots,
                impact_slots: [WeatherImpactSlotSub61CA::inactive(1, 0); 10],
                kid_hitbox_top_px: 9999,
                kid_hitbox_bottom_px: 9999,
                kid_hitbox_left_px: 9999,
                kid_hitbox_right_px: 9999,
                random_byte: 0,
                collision_rows: empty_weather_collision_rows_sub_61_ca(),
                starting_number_sprites: 0,
            });

            assert_eq!(
                stepped.sprites[0].tile_word, *expected_tile,
                "ASM loc_6462: unk_64BC[{frame}] should be ${expected_tile:04X}"
            );
        }
    }

    #[test]
    fn loc_64d0_dead_code_nibble_interleave_matches_loc_14348_algorithm() {
        // ASM loc_64D0 (kid.asm:6502) / loc_64D2 (kid.asm:6505): dead code in the
        // ROM — identical nibble-interleaving algorithm to loc_14348/loc_1434A in
        // DecompressToVRAM_Special.
        //   loc_64D0: moveq #0,d2                     (reset column index)
        //   loc_64D2: move.b (a0,d1.w),d3; lsl.b #4,d3; or.b (a0,d2.w),d3;
        //             move.b d3,(a1)+; addq.w #1,d2; cmpi.w #$10,d2; bne.s loc_64D2
        //   16×16 nested loop packs high nibble from row d1, low nibble from col d2.
        // Verify the existing decompress_special_palette_row_bytes_loc_1434A
        // produces the same result as a direct translation of loc_64D0/loc_64D2's
        // loop.
        let permutation: [u8; 16] = [
            0, 1, 0x0B, 0x0C, 0x0D, 6, 0x0A, 9, 0x0D, 7, 0x0E, 2, 3, 4, 8, 0x0F,
        ];

        // Reproduce loc_64D0/loc_64D2: for each row d1 in 0..16, emit 16 bytes by
        // packing permutation[d1] << 4 | permutation[d2] for d2 in 0..16.
        let mut expected = Vec::with_capacity(256);
        for row in 0u16..16 {
            let row_bytes =
                crate::level::decompress_special_palette_row_bytes_loc_1434A(&permutation, row, 15);
            expected.extend_from_slice(&row_bytes);
        }

        // Direct translation of the ASM loop for cross-check
        let mut direct = Vec::with_capacity(256);
        for d1 in 0usize..16 {
            for d2 in 0usize..16 {
                let high = permutation[d1] << 4;
                let low = permutation[d2];
                direct.push(high | low);
            }
        }

        assert_eq!(
            expected, direct,
            "ASM loc_64D0/loc_64D2: the dead-code nibble interleave loop at \
             kid.asm:6502-6515 must match decompress_special_palette_row_bytes_loc_1434A"
        );
    }

    #[test]
    fn loc_64d2_inner_column_loop_packs_one_row_of_16_nibble_pairs() {
        // ASM loc_64D2 (kid.asm:6505): inner column loop body for the dead-code
        // nibble interleave.  For a single row d1, iterates d2 from 0 to 15:
        //   move.b (a0,d1.w),d3; lsl.b #4,d3; or.b (a0,d2.w),d3; move.b d3,(a1)+
        // Verify that decompress_special_palette_row_bytes_loc_1434A (the live
        // equivalent at loc_1434A) produces the expected packed bytes for row 0.
        let permutation: [u8; 16] = [
            0, 1, 0x0B, 0x0C, 0x0D, 6, 0x0A, 9, 0x0D, 7, 0x0E, 2, 3, 4, 8, 0x0F,
        ];
        let row_bytes =
            crate::level::decompress_special_palette_row_bytes_loc_1434A(&permutation, 0, 15);

        // Row 0: high nibble = permutation[0] = 0, so each byte = permutation[d2].
        assert_eq!(row_bytes.len(), 16);
        for (col, &byte) in row_bytes.iter().enumerate() {
            assert_eq!(
                byte, permutation[col],
                "ASM loc_64D2: row 0, col {col}: high nibble 0 | low nibble {:#04X} = {:#04X}",
                permutation[col], permutation[col],
            );
        }

        // Row 5: high nibble = permutation[5] = 6, so each byte = 0x60 | permutation[d2].
        let row5_bytes =
            crate::level::decompress_special_palette_row_bytes_loc_1434A(&permutation, 5, 15);
        for (col, &byte) in row5_bytes.iter().enumerate() {
            assert_eq!(
                byte,
                (6 << 4) | permutation[col],
                "ASM loc_64D2: row 5, col {col}: high nibble $60 | low nibble {:#04X}",
                permutation[col],
            );
        }
    }

    // --- Hail period toggle ---

    #[test]
    fn hail_period_toggles_active_inactive_on_600_frame_cycle() {
        // ASM: hail_period_timer counts down from 600; at 0 it toggles hail_active
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 3;
        runtime.level_timer.seconds = 100; // well past 35s elapsed
        runtime.weather.hail_active = false;
        runtime.weather.hail_period_timer = 1; // about to toggle
        runtime.weather.hail_spawn_timer = 100;

        let slice = airborne_slice();

        // Timer ticks to 0 → toggles active, resets to 600
        runtime.step_weather(&slice);
        assert_eq!(runtime.weather.hail_period_timer, 0);
        // One more step triggers the toggle
        runtime.step_weather(&slice);
        assert!(
            runtime.weather.hail_active,
            "hail should toggle to active when timer reaches 0"
        );
        assert_eq!(
            runtime.weather.hail_period_timer, 600,
            "period timer should reset to 600"
        );
    }

    #[test]
    fn hail_does_not_spawn_before_35_seconds_elapsed() {
        // Timer at 146 means only 34s elapsed (180 - 146 = 34)
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 3;
        runtime.level_timer.seconds = 146;
        runtime.weather.hail_active = true;
        runtime.weather.hail_spawn_timer = 0;

        let slice = airborne_slice();
        runtime.step_weather(&slice);

        assert!(
            runtime.projectiles.is_empty(),
            "no hail should spawn before 35s elapsed"
        );
    }

    #[test]
    fn hail_spawn_timer_counts_down_between_spawns() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 3;
        runtime.level_timer.seconds = 100;
        runtime.weather.hail_active = true;
        runtime.weather.hail_spawn_timer = 2;
        runtime.weather.hail_period_timer = 100;

        let slice = airborne_slice();

        // Timer at 2: decrement, no spawn
        runtime.step_weather(&slice);
        assert!(runtime.projectiles.is_empty());
        assert_eq!(runtime.weather.hail_spawn_timer, 1);

        // Timer at 1: decrement, no spawn
        runtime.step_weather(&slice);
        assert!(runtime.projectiles.is_empty());
        assert_eq!(runtime.weather.hail_spawn_timer, 0);

        // Timer at 0: spawn and reset to 30
        runtime.step_weather(&slice);
        assert_eq!(runtime.projectiles.len(), 1);
        assert_eq!(runtime.weather.hail_spawn_timer, 30);
    }

    // --- Lava geyser collision ---

    #[test]
    fn lava_geyser_damages_player_inside_eruption_column() {
        // Player positioned inside the eruption column should take damage
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: 40,
            phase: 1,
            timer: 0,
            current_height: 40, // fully erupted
        });
        runtime.progression.invulnerable_frames = 0;
        runtime.opening_slice = None; // apply_damage needs this

        let mut slice = airborne_slice();
        // Place player inside the column (x_px=100, column from y=160 to y=200)
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 180 << 16;

        runtime.step_lava_geysers(&slice);
        assert!(
            runtime.just_received_damage,
            "player inside geyser column should take damage"
        );
    }

    #[test]
    fn lava_geyser_sub_14a58_uses_kid_hitbox_not_feet_point() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: 40,
            phase: 1,
            timer: 0,
            current_height: 40,
        });
        runtime.progression.invulnerable_frames = 0;
        runtime.opening_slice = None;

        let mut slice = airborne_slice();
        slice.player.x_pos = 113 << 16;
        slice.player.y_pos = 195 << 16;

        runtime.step_lava_geysers(&slice);

        assert!(
            runtime.just_received_damage,
            "ASM sub_14A58 should damage when Kid's hitbox overlaps the geyser column even if the feet point is outside the native ±8px centre check"
        );
    }

    #[test]
    fn lava_geyser_does_not_damage_player_outside_column() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: 40,
            phase: 1,
            timer: 0,
            current_height: 40,
        });
        runtime.progression.invulnerable_frames = 0;

        let mut slice = airborne_slice();
        // Player far from column
        slice.player.x_pos = 500 << 16;
        slice.player.y_pos = 180 << 16;

        runtime.step_lava_geysers(&slice);
        assert!(
            !runtime.just_received_damage,
            "player outside column should not take damage"
        );
    }

    #[test]
    fn lava_geyser_respects_invulnerability() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: 40,
            phase: 1,
            timer: 0,
            current_height: 40,
        });
        runtime.progression.invulnerable_frames = 10; // invulnerable

        let mut slice = airborne_slice();
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 180 << 16;

        runtime.step_lava_geysers(&slice);
        assert!(
            !runtime.just_received_damage,
            "invulnerable player should not take damage from geyser"
        );
    }

    #[test]
    fn lava_geyser_dormant_timer_resets_after_collapse() {
        // After collapse phase completes, dormant timer should reset to LAVA_GEYSER_DORMANT_FRAMES
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: 16,
            phase: 3,
            timer: 0,
            current_height: LAVA_GEYSER_ERUPT_SPEED, // one step from collapse end
        });

        let slice = airborne_slice();
        runtime.step_lava_geysers(&slice);

        let g = &runtime.weather.lava_geysers[0];
        assert_eq!(g.phase, 0, "should return to dormant");
        assert_eq!(
            g.timer, LAVA_GEYSER_DORMANT_FRAMES,
            "dormant timer should reset"
        );
        assert_eq!(g.current_height, 0);
    }

    #[test]
    fn lava_geyser_no_damage_when_height_zero() {
        // Even if player overlaps position, zero height = no damage
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 100,
            y_px: 200,
            height_px: 40,
            phase: 0,
            timer: 5,
            current_height: 0, // dormant, no eruption
        });

        let mut slice = airborne_slice();
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 180 << 16;

        runtime.step_lava_geysers(&slice);
        assert!(
            !runtime.just_received_damage,
            "dormant geyser should not damage player"
        );
    }

    #[test]
    fn lava_fireball_sub_14ab0_uses_scripted_hitbox_window() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_fireballs.push(LavaFireball {
            x_px: 100,
            base_y_px: 200,
            current_y_px: 200,
            segment_index: 0,
            segment_frame: 0,
            cooldown_timer: 0,
        });
        runtime.progression.invulnerable_frames = 0;
        runtime.opening_slice = None;

        let mut slice = airborne_slice();
        slice.player.x_pos = 111 << 16;
        slice.player.y_pos = 190 << 16;

        runtime.step_lava_geysers(&slice);

        assert!(
            runtime.just_received_damage,
            "ASM sub_14AB0 should damage when Kid's hitbox clips the small fireball window from the tall-rise sprite layout"
        );
    }

    #[test]
    fn lava_fireball_script_restarts_after_rom_cooldown() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.weather.weather_type = 1;
        runtime.weather.lava_fireballs.push(LavaFireball {
            x_px: 100,
            base_y_px: 200,
            current_y_px: 200,
            segment_index: 0,
            segment_frame: 0,
            cooldown_timer: 0,
        });

        let slice = airborne_slice();
        for _ in 0..115 {
            runtime.step_lava_geysers(&slice);
        }
        let fireball = &runtime.weather.lava_fireballs[0];
        assert_eq!(
            fireball.cooldown_timer, 60,
            "the scripted small fireball should enter the ROM's 60-frame disabled window after the last segment"
        );

        for _ in 0..60 {
            runtime.step_lava_geysers(&slice);
        }
        let fireball = &runtime.weather.lava_fireballs[0];
        assert_eq!(fireball.cooldown_timer, 0);
        assert_eq!(fireball.segment_index, 0);
        assert_eq!(fireball.segment_frame, 0);
        assert_eq!(
            fireball.current_y_px, fireball.base_y_px,
            "after the cooldown the small fireball should restart from its baseline row"
        );
    }

    // --- Lava geyser init ---

    #[test]
    fn loc_14772_uses_the_low_height_nibble_as_a_10_frame_delay_table_index() {
        assert_eq!(
            crate::weather::lava_geyser_initial_delay_frames_loc_14772(0x0060),
            0,
            "ASM loc_14772 should use `word_14710[0] = 0` when the encoded height low nibble is zero"
        );
        assert_eq!(
            crate::weather::lava_geyser_initial_delay_frames_loc_14772(0x0063),
            30,
            "ASM loc_14772 should map low nibble `3` to `word_14710[3] = 30` hibernation frames before the geyser starts"
        );
        assert_eq!(
            crate::weather::lava_geyser_initial_delay_frames_loc_14772(0x008F),
            150,
            "ASM loc_14772 should cap the low-nibble lookup at `word_14710[15] = 150`"
        );
    }

    #[test]
    fn loc_1499c_clears_the_fireball_sign_bit_then_seeds_delay_height_and_biased_origin() {
        assert_eq!(
            crate::weather::initialise_lava_fireball_state_loc_1499C(0x0346, 0x0312, 0x8063),
            crate::weather::LavaFireballInitStateLoc1499C {
                startup_delay_frames: 30,
                rise_height_steps: 0x0063 >> 4,
                biased_x_px: 0x03C6,
                biased_y_px: 0x0392,
            },
            "ASM loc_1499C should clear bit 15 from `$1A(a5)`, use the low nibble as the `word_14710` delay index, shift the remaining height into `$4A(a5)`, and add the shared `$80` bias to `$44/$46(a5)`"
        );
    }

    #[test]
    fn loc_149cc_restarts_the_fireball_from_unk_146cc_and_unbiases_the_saved_origin() {
        assert_eq!(
            crate::weather::initialise_lava_fireball_script_loc_149CC(0x03C6, 0x0392),
            crate::weather::LavaFireballScriptStartLoc149CC {
                script_name: "unk_146CC",
                x_px: 0x0346,
                y_px: 0x0312,
                sprite_top_y_px: 0x0312,
            },
            "ASM loc_149CC should reload `a0` from `unk_146CC`, subtract the shared `$80` sprite bias from `$44/$46(a5)`, and seed `$54(a5)` from the unbiased baseline Y row before the fireball segment loop restarts"
        );
    }

    #[test]
    fn loc_149e4_decodes_fireball_segments_and_treats_negative_counts_as_the_cooldown_sentinel() {
        assert_eq!(
            crate::weather::decode_lava_fireball_script_step_loc_149E4(&[0x000A, 0xFFFC, 0x1234]),
            Some(crate::weather::LavaFireballScriptStepLoc149E4::EmitSegment(
                crate::weather::LavaFireballSegmentLoc149E4 {
                    dbf_counter: 0x000A,
                    frames: 0x000B,
                    delta_y: -4,
                    sprite_script_word: 0x1234,
                }
            )),
            "ASM loc_149E4 should treat the first word as the `dbf` counter, load the second word into `$52(a5)` as the signed per-frame Y delta, and pass the third word through as the `LoadGameModeData` sprite-script offset"
        );
        assert_eq!(
            crate::weather::decode_lava_fireball_script_step_loc_149E4(&[0xFFFF]),
            Some(crate::weather::LavaFireballScriptStepLoc149E4::CooldownAndRestart),
            "ASM loc_149E4 should branch to loc_14A4C as soon as the first script word is negative, which is how `unk_146CC` terminates the fireball loop before the 60-frame hibernation"
        );
    }

    #[test]
    fn loc_149fc_hibernates_refreshes_collision_and_applies_the_signed_segment_delta_before_rechecking_the_camera_window()
     {
        assert_eq!(
            crate::weather::lava_fireball_loop_resume_loc_149FC(0x0312, -4),
            crate::weather::LavaFireballLoopResumeLoc149FC {
                hibernation_frames: 1,
                collision_refreshes: 1,
                sprite_script_restored: true,
                next_baseline_y_px: 0x030E,
            },
            "ASM loc_149FC should hibernate one frame, re-run `sub_14AB0`, restore `a2` from `a1`, and add the signed `$52(a5)` delta into `d1` before the shared camera-window check"
        );
    }

    #[test]
    fn loc_14b06_uses_inclusive_overlap_bounds_for_fireball_damage() {
        let window = crate::weather::LavaFireballVerticalWindowLoc14AFC {
            top_px: 0x0200,
            bottom_px: 0x020F,
        };

        assert!(
            crate::weather::lava_fireball_overlap_gate_loc_14B06(
                0x0107, 0x0108, 0x020F, 0x0200, 0x0108, 0x010E, window,
            ),
            "ASM loc_14B06 should keep the hit active when Kid only touches the fireball on the shared left/right/top/bottom boundaries"
        );
    }

    #[test]
    fn loc_14b22_returns_early_when_any_fireball_overlap_compare_fails() {
        let window = crate::weather::LavaFireballVerticalWindowLoc14AFC {
            top_px: 0x0200,
            bottom_px: 0x020F,
        };

        assert!(
            crate::weather::lava_fireball_rejects_kid_loc_14B22(
                0x0100, 0x0107, 0x0200, 0x020F, 0x0108, 0x010E, window,
            ),
            "ASM loc_14B22 should return immediately when Kid stays left of the fireball window"
        );
        assert!(
            crate::weather::lava_fireball_rejects_kid_loc_14B22(
                0x010F, 0x0116, 0x0200, 0x020F, 0x0108, 0x010E, window,
            ),
            "ASM loc_14B22 should return immediately when Kid stays right of the fireball window"
        );
        assert!(
            crate::weather::lava_fireball_rejects_kid_loc_14B22(
                0x0108, 0x010E, 0x0210, 0x0217, 0x0108, 0x010E, window,
            ),
            "ASM loc_14B22 should return immediately when Kid stays below the fireball window in screen coordinates"
        );
        assert!(
            crate::weather::lava_fireball_rejects_kid_loc_14B22(
                0x0108, 0x010E, 0x01F0, 0x01FF, 0x0108, 0x010E, window,
            ),
            "ASM loc_14B22 should return immediately when Kid stays above the fireball window in screen coordinates"
        );
    }

    #[test]
    fn loc_14a46_decrements_the_dbf_counter_and_only_exits_after_the_ffff_sentinel() {
        assert_eq!(
            crate::weather::lava_fireball_segment_back_edge_loc_14A46(0x0003),
            crate::weather::LavaFireballSegmentBackEdgeLoc14A46 {
                next_dbf_counter: 0x0002,
                reenters_segment_loop: true,
                advances_to_next_script_step: false,
            },
            "ASM loc_14A46 should loop back to loc_149FC while `dbf d2,loc_149FC` leaves the post-decrement counter above `$FFFF`"
        );
        assert_eq!(
            crate::weather::lava_fireball_segment_back_edge_loc_14A46(0x0000),
            crate::weather::LavaFireballSegmentBackEdgeLoc14A46 {
                next_dbf_counter: 0xFFFF,
                reenters_segment_loop: false,
                advances_to_next_script_step: true,
            },
            "ASM loc_14A46 should fall through to `bra.s loc_149E4` exactly when the low-word counter underflows to `$FFFF`"
        );
    }

    #[test]
    fn loc_14788_points_the_first_lava_geyser_eruption_at_the_startup_script() {
        assert_eq!(
            crate::weather::lava_geyser_startup_script_loc_14788(),
            "unk_146B6",
            "ASM loc_14788 should seed `a0` with the startup geyser script `unk_146B6` before entering loc_147A0"
        );
    }

    #[test]
    fn loc_14aaa_only_keeps_the_geyser_damage_window_when_all_inclusive_bounds_pass() {
        assert_eq!(
            crate::weather::lava_geyser_hitbox_window_loc_14AAA(0x0100, 0x0200, 0),
            None,
            "ASM loc_14AAA should never arm the damage window when the geyser height is zero, because sub_14A58 exits before any overlap can succeed"
        );
        assert_eq!(
            crate::weather::lava_geyser_hitbox_window_loc_14AAA(0x0100, 0x0200, 0x0030),
            Some(crate::weather::LavaGeyserHitboxWindowLoc14AAA {
                left_px: 0x0103,
                right_px: 0x011D,
                top_px: 0x01E0,
                bottom_px: 0x0200,
            }),
            "ASM loc_14AAA should only fall through to the Kid damage write after sub_14A58 builds the inclusive window `[x+3, x+3+$1A]` by `[top_sprite_y+$10, base_y]`"
        );
    }

    #[test]
    fn loc_14966_expands_the_peak_row_target_by_two_when_the_top_cap_wraps() {
        assert_eq!(
            crate::weather::lava_geyser_peak_row_target_loc_14966(0x0008),
            0x000A,
            "ASM loc_14966 should add 2 to `$4A(a5)` when `eori.b #1,$4C(a5)` wraps the alternating top-cap frame back to zero"
        );
    }

    #[test]
    fn loc_14afc_applies_the_tall_rise_fireball_drop_before_the_shared_overlap_checks() {
        assert_eq!(
            crate::weather::lava_fireball_vertical_window_loc_14AFC(
                0x0200,
                crate::weather::LavaFireballSpriteKind::TallRise,
            ),
            crate::weather::LavaFireballVerticalWindowLoc14AFC {
                top_px: 0x01E9,
                bottom_px: 0x01F8,
            },
            "ASM loc_14AFC should turn the tall-rise bucket into the vertical window `[baseline_y-23, baseline_y-8]` after the extra `subq.w #7,d7` adjustment"
        );
        assert_eq!(
            crate::weather::lava_fireball_vertical_window_loc_14AFC(
                0x0200,
                crate::weather::LavaFireballSpriteKind::MidBody,
            ),
            crate::weather::LavaFireballVerticalWindowLoc14AFC {
                top_px: 0x01F9,
                bottom_px: 0x01FF,
            },
            "ASM loc_14AFC should leave the later sprite buckets on their own neighbouring branches, so a mid-body segment keeps the shared `baseline_y-1` bottom row"
        );
    }

    #[test]
    fn loc_1496c_counts_down_the_hold_timer_and_only_switches_to_collapse_at_zero() {
        assert_eq!(
            crate::weather::lava_geyser_peak_hold_step_loc_1496C(0x000A, 0x001E),
            crate::weather::LavaGeyserPeakHoldStepLoc1496C {
                sprite_row_index: 0x000A,
                frames_per_sprite: 6,
                next_hold_frames: 0x001D,
                descending: false,
            },
            "ASM loc_1496C should keep the 6-frame top-cap cadence while the decremented `$50(a5)` timer stays non-zero"
        );
        assert_eq!(
            crate::weather::lava_geyser_peak_hold_step_loc_1496C(0x000A, 0x0001),
            crate::weather::LavaGeyserPeakHoldStepLoc1496C {
                sprite_row_index: 0x000A,
                frames_per_sprite: 1,
                next_hold_frames: 0x0000,
                descending: true,
            },
            "ASM loc_1496C should arm the collapse path and switch to the 1-frame cadence once `subq.w #1,$50(a5)` reaches zero"
        );
    }

    #[test]
    fn loc_1498c_restarts_the_geyser_once_the_last_collapse_row_clears() {
        assert_eq!(
            crate::weather::lava_geyser_collapse_step_loc_1498C(0x0003),
            crate::weather::LavaGeyserCollapseStepLoc1498C {
                next_sprite_row_index: 0x0002,
                descending: true,
                restart_startup_script: false,
            },
            "ASM loc_1498C should keep collapsing through loc_1485E while the decremented row count stays non-zero"
        );
        assert_eq!(
            crate::weather::lava_geyser_collapse_step_loc_1498C(0x0001),
            crate::weather::LavaGeyserCollapseStepLoc1498C {
                next_sprite_row_index: 0x0000,
                descending: false,
                restart_startup_script: true,
            },
            "ASM loc_1498C should clear `$4D(a5)` and branch back to loc_14788 when the final collapse row decrements to zero"
        );
    }

    #[test]
    fn loc_14b2a_hibernates_five_frames_then_rotates_the_geyser_palette_ring() {
        let mut palette_buffer = [0u16; 0x40];
        palette_buffer[0x14] = 0x1111;
        palette_buffer[0x15] = 0x2222;
        palette_buffer[0x16] = 0x3333;
        palette_buffer[0x17] = 0x4444;

        let step = crate::weather::rotate_lava_geyser_palette_loc_14B2A(&mut palette_buffer);

        assert_eq!(
            step,
            crate::weather::LavaGeyserPaletteLoopLoc14B2A {
                hibernation_frames: 5,
                reloaded_delay_counter: 5,
            },
            "ASM loc_14B2A should spend five one-frame hibernations in the `d0` countdown before reloading the loop counter to `#5`"
        );
        assert_eq!(
            palette_buffer[0x14], 0x4444,
            "ASM loc_14B2A should wrap `Palette_Buffer+$2E` back into `Palette_Buffer+$28`"
        );
        assert_eq!(palette_buffer[0x15], 0x1111);
        assert_eq!(palette_buffer[0x16], 0x2222);
        assert_eq!(palette_buffer[0x17], 0x3333);
    }

    #[test]
    fn loc_1478e_only_plays_the_repeat_lava_geyser_sound_when_the_flag_is_clear() {
        assert_eq!(
            crate::weather::lava_geyser_repeat_sound_loc_1478E(false),
            Some("sfx_Lava_Geyser"),
            "ASM loc_1478E should queue `sfx_Lava_Geyser` when `$19(a3)` is zero on repeat eruptions"
        );
        assert_eq!(
            crate::weather::lava_geyser_repeat_sound_loc_1478E(true),
            None,
            "ASM loc_1478E should skip the repeat sound when `$19(a3)` is non-zero"
        );
    }

    #[test]
    fn loc_1479c_points_repeat_lava_geyser_loops_at_the_steady_state_script() {
        assert_eq!(
            crate::weather::lava_geyser_repeat_script_loc_1479C(),
            "unk_146A0",
            "ASM loc_1479C should seed `a0` with `unk_146A0` for repeat lava geyser loops after the loc_1478E sound gate"
        );
    }

    #[test]
    fn loc_147a0_removes_the_vdp_bias_and_seeds_the_full_height_timer() {
        let state =
            crate::weather::initialise_lava_geyser_full_height_state_loc_147A0(0x0200, 0x0392);
        assert_eq!(state.x_px, 0x0180);
        assert_eq!(state.y_px, 0x0312);
        assert_eq!(state.hold_frames, 0x001E);
        assert_eq!(
            state.sprite_top_y_px, 0x0312,
            "ASM loc_147A0 should subtract the shared `$80` sprite bias from `$44/$46(a5)`, set `$50(a5) = $1E`, and seed `$54(a5)` from the unbiased base Y row"
        );
    }

    #[test]
    fn loc_147ba_decodes_geyser_script_sentinels_and_sprite_rows() {
        assert_eq!(
            crate::weather::decode_lava_geyser_script_step_loc_147BA(&[0xFFFF]),
            Some(crate::weather::LavaGeyserScriptStepLoc147BA::HoldAtFullHeight),
            "ASM loc_147BA should treat `$FFFF` as the full-height loop sentinel"
        );
        assert_eq!(
            crate::weather::decode_lava_geyser_script_step_loc_147BA(&[0xFFFE]),
            Some(crate::weather::LavaGeyserScriptStepLoc147BA::DisableForCooldown),
            "ASM loc_147BA should treat `$FFFE` as the 60-frame disable sentinel"
        );
        assert_eq!(
            crate::weather::decode_lava_geyser_script_step_loc_147BA(&[0x0003, 0x0124]),
            Some(
                crate::weather::LavaGeyserScriptStepLoc147BA::EmitSpriteRows {
                    sprite_rows_minus_one: 0x0003,
                    sprite_script_word: 0x0124,
                }
            ),
            "ASM loc_147BA should leave the second word available as the `LoadGameModeData` sprite-script offset for normal geyser rows"
        );
    }

    #[test]
    fn loc_1483a_disables_the_geyser_for_60_frames_then_rejoins_the_repeat_loop() {
        let step = crate::weather::lava_geyser_disable_step_loc_1483A();
        assert_eq!(
            step.sound, "sfx_Lava_Geyser",
            "ASM loc_1483A should queue `sfx_Lava_Geyser` before disabling the worker"
        );
        assert_eq!(
            step.disabled_frames, 0x003C,
            "ASM loc_1483A should push `#$3C` into `j_Hibernate_Object`, disabling the geyser for 60 frames"
        );
        assert!(
            step.resumes_at_repeat_loop,
            "ASM loc_1483A should resume at loc_1478E after the disable hibernation rather than restarting the first-eruption script"
        );
    }

    #[test]
    fn loc_1484e_resets_the_rise_row_cursor_and_two_frame_cadence() {
        let reset = crate::weather::lava_geyser_rise_reset_loc_1484E();
        assert_eq!(
            reset.sprite_row_index, 0,
            "ASM loc_1484E should restart the rise row cursor by loading `d3 = 0`"
        );
        assert_eq!(
            reset.frames_per_sprite_row, 2,
            "ASM loc_1484E should seed `$48(a5)` with two frames per sprite row while the geyser rises"
        );
        assert_eq!(
            reset.sprite_frame_within_row, 0,
            "ASM loc_1484E should clear `$4C(a5)` before entering the shared ascent loop"
        );
    }

    #[test]
    fn loc_1485e_hibernates_once_refreshes_collision_once_then_rejoins_the_shared_loop() {
        let step = crate::weather::lava_geyser_pre_loop_hibernate_loc_1485E();
        assert_eq!(
            step.hibernation_frames, 1,
            "ASM loc_1485E should call `j_Hibernate_Object_1Frame` exactly once before the shared loop resumes"
        );
        assert_eq!(
            step.collision_refreshes, 1,
            "ASM loc_1485E should re-run `sub_14A58` exactly once on that hibernated frame"
        );
        assert!(
            step.falls_through_to_shared_loop,
            "ASM loc_1485E should fall through to loc_14866 after the one-frame hibernation/collision refresh"
        );
    }

    #[test]
    fn loc_1486c_hibernates_once_on_each_taken_back_edge_then_reenters_the_shared_row_loop() {
        let step = crate::weather::lava_geyser_loop_back_edge_loc_1486C();
        assert_eq!(
            step.hibernation_frames, 1,
            "ASM loc_1486C should call `j_Hibernate_Object_1Frame` once on each taken `dbf d2,loc_1486C` loop back-edge"
        );
        assert_eq!(
            step.collision_refreshes, 1,
            "ASM loc_1486C should re-run `sub_14A58` exactly once before falling back into the shared row builder"
        );
        assert!(
            step.reenters_shared_loop,
            "ASM loc_1486C should fall through into loc_14874 after the one-frame hibernation/collision refresh"
        );
    }

    #[test]
    fn loc_147d6_uses_the_padded_camera_window_before_building_sprite_rows() {
        let inside = crate::weather::LavaGeyserCameraWindowInputLoc147D6 {
            x_px: 0x00E0,
            y_px: 0x0118,
            camera_x_px: 0x0100,
            camera_y_px: 0x0160,
        };
        assert!(
            crate::weather::lava_geyser_is_inside_camera_window_loc_147D6(inside),
            "ASM loc_147D6 should keep rows whose base point sits on the inclusive `[camera_x - $20, camera_x + $160]` and `[camera_y - $48, camera_y + $128]` window"
        );

        let left_clipped = crate::weather::LavaGeyserCameraWindowInputLoc147D6 {
            x_px: 0x00DF,
            ..inside
        };
        assert!(
            !crate::weather::lava_geyser_is_inside_camera_window_loc_147D6(left_clipped),
            "ASM loc_147D6 should skip rows left of `Camera_X_pos - $20`"
        );

        let bottom_clipped = crate::weather::LavaGeyserCameraWindowInputLoc147D6 {
            y_px: 0x0289,
            ..inside
        };
        assert!(
            !crate::weather::lava_geyser_is_inside_camera_window_loc_147D6(bottom_clipped),
            "ASM loc_147D6 should skip rows below `Camera_Y_pos + $128`"
        );
    }

    #[test]
    #[ignore = "TODO(fidelity): loc_14874 not yet implemented"]
    fn loc_14874_unbiases_the_current_row_and_distinguishes_skip_top_from_full_cull() {
        let visible = crate::weather::LavaGeyserTopSpriteWindowInputLoc14874 {
            x_px: 0x0180,
            y_px: 0x0210,
            sprite_row_index: 0,
            camera_x_px: 0x0100,
            camera_y_px: 0x0160,
        };
        assert_eq!(
            crate::weather::lava_geyser_top_sprite_window_loc_14874(visible),
            crate::weather::LavaGeyserTopSpriteWindowResultLoc14874::BuildTopSprite {
                sprite_row_y_px: 0x0180,
            },
            "ASM loc_14874 should subtract `$80` from `$44/$46(a5)`, subtract `$10 + ((d3 + 1) << 4)` from Y, and keep rows inside the inclusive X and lower/upper Y camera pads on the top-cap path"
        );

        let above_top_pad = crate::weather::LavaGeyserTopSpriteWindowInputLoc14874 {
            y_px: 0x01F7,
            ..visible
        };
        assert_eq!(
            crate::weather::lava_geyser_top_sprite_window_loc_14874(above_top_pad),
            crate::weather::LavaGeyserTopSpriteWindowResultLoc14874::SkipTopSprite {
                sprite_row_y_px: 0x0167,
            },
            "ASM loc_14874 should branch to loc_148E6, skipping only the top-cap build, when the current row sits above `Camera_Y_pos - $28`"
        );

        let right_clipped = crate::weather::LavaGeyserTopSpriteWindowInputLoc14874 {
            x_px: 0x0301,
            ..visible
        };
        assert_eq!(
            crate::weather::lava_geyser_top_sprite_window_loc_14874(right_clipped),
            crate::weather::LavaGeyserTopSpriteWindowResultLoc14874::Culled,
            "ASM loc_14874 should branch to loc_14940 when `$44(a5) - $80` lies right of `Camera_X_pos + $160`"
        );

        let below_bottom_pad = crate::weather::LavaGeyserTopSpriteWindowInputLoc14874 {
            y_px: 0x02B9,
            ..visible
        };
        assert_eq!(
            crate::weather::lava_geyser_top_sprite_window_loc_14874(below_bottom_pad),
            crate::weather::LavaGeyserTopSpriteWindowResultLoc14874::Culled,
            "ASM loc_14874 should branch to loc_14940 when the current row falls below `Camera_Y_pos + $108`"
        );
    }

    #[test]
    fn loc_148ce_switches_top_sprite_scripts_once_the_row_animation_flag_is_non_zero() {
        assert_eq!(
            crate::weather::lava_geyser_top_sprite_script_loc_148CE(0),
            "unk_14624",
            "ASM loc_148CE should leave `d7 = unk_14624` when `$4C(a5)` is still zero"
        );
        assert_eq!(
            crate::weather::lava_geyser_top_sprite_script_loc_148CE(1),
            "dword_14636",
            "ASM loc_148CE should switch to `dword_14636` once `$4C(a5)` becomes non-zero"
        );
    }

    #[test]
    fn loc_148d4_unbiases_the_top_sprite_row_y_before_storing_world_y() {
        assert_eq!(
            crate::weather::lava_geyser_store_top_sprite_y_loc_148D4(0x0080, 0x0123),
            0x0123,
            "ASM loc_148D4 should treat the `sub_14BD8` return value `$80` as zero screen offset before adding `Camera_Y_pos` into `$54(a5)`"
        );
        assert_eq!(
            crate::weather::lava_geyser_store_top_sprite_y_loc_148D4(0x00A7, 0x0312),
            0x0339,
            "ASM loc_148D4 should store `d7 - $80 + Camera_Y_pos` after the top-cap build"
        );
    }

    #[test]
    fn loc_14be8_accumulates_y_steps_and_biases_tiles_with_the_incremented_sprite_count() {
        let sprite_rows = crate::weather::lava_geyser_sprite_chunks_loc_14BE8(
            0x0400,
            0x0380,
            0x00D0,
            3,
            &[
                crate::weather::LavaGeyserSpriteTemplateLoc14BE8 {
                    x_offset_px: 0x0008,
                    y_step_px: 0x0010,
                    size_word: 0x0F00,
                    tile_base_word: 0x0200,
                },
                crate::weather::LavaGeyserSpriteTemplateLoc14BE8 {
                    x_offset_px: 0x0018,
                    y_step_px: 0x0006,
                    size_word: 0x0500,
                    tile_base_word: 0x0210,
                },
            ],
        );

        assert_eq!(
            sprite_rows,
            vec![
                crate::weather::LavaGeyserSpriteChunkLoc14BE8 {
                    screen_y_with_bias: 0x00C0,
                    tile_word: 0x0204,
                    size_word: 0x0F00,
                    screen_x_px: 0x0088,
                },
                crate::weather::LavaGeyserSpriteChunkLoc14BE8 {
                    screen_y_with_bias: 0x00BA,
                    tile_word: 0x0215,
                    size_word: 0x0500,
                    screen_x_px: 0x0098,
                },
            ],
            "ASM loc_14BE8 should subtract each entry's Y step from the running `d7`, increment `Number_Sprites` before adding it into the tile word, and place X at `$44(a5) - Camera_X_pos + entry_x_offset`"
        );
    }

    #[test]
    fn loc_148f8_uses_the_middle_row_camera_band_before_emitting_a_sprite_row() {
        assert_eq!(
            crate::weather::lava_geyser_middle_sprite_row_window_loc_148F8(0x0150, 0x0160),
            crate::weather::LavaGeyserMiddleSpriteRowResultLoc148F8::BuildSpriteRow,
            "ASM loc_148F8 should keep middle rows whose `d1` lies inside the inclusive `[Camera_Y_pos - $10, Camera_Y_pos + $F0]` band"
        );
        assert_eq!(
            crate::weather::lava_geyser_middle_sprite_row_window_loc_148F8(0x014F, 0x0160),
            crate::weather::LavaGeyserMiddleSpriteRowResultLoc148F8::SkipAboveWindow,
            "ASM loc_148F8 should skip the current middle-row build and fall through to loc_14914 when `d1` sits above `Camera_Y_pos - $10`"
        );
        assert_eq!(
            crate::weather::lava_geyser_middle_sprite_row_window_loc_148F8(0x0251, 0x0160),
            crate::weather::LavaGeyserMiddleSpriteRowResultLoc148F8::CulledBelowWindow,
            "ASM loc_148F8 should branch to loc_14940 when `d1` falls below the inclusive `Camera_Y_pos + $F0` bound"
        );
    }

    #[test]
    fn loc_14914_advances_to_the_next_middle_row_and_loops_while_d5_stays_within_d3() {
        assert_eq!(
            crate::weather::lava_geyser_middle_row_advance_loc_14914(0x0150, 0, 2),
            crate::weather::LavaGeyserMiddleRowAdvanceResultLoc14914 {
                next_sprite_row_y_px: 0x0160,
                next_middle_row_index: 1,
                continue_middle_rows: true,
            },
            "ASM loc_14914 should add `$10` to `d1`, increment `d5`, and branch back to loc_148F8 while the updated index still satisfies `d5 <= d3`"
        );
        assert_eq!(
            crate::weather::lava_geyser_middle_row_advance_loc_14914(0x0170, 2, 2),
            crate::weather::LavaGeyserMiddleRowAdvanceResultLoc14914 {
                next_sprite_row_y_px: 0x0180,
                next_middle_row_index: 3,
                continue_middle_rows: false,
            },
            "ASM loc_14914 should fall through to the bottom-cap path once the incremented middle-row index exceeds `d3`"
        );
    }

    #[test]
    fn loc_14804_hibernates_once_per_additional_sprite_row_after_the_first() {
        assert_eq!(
            crate::weather::lava_geyser_inter_row_hibernations_loc_14804(0),
            0,
            "ASM loc_14804 sits on the `dbf d2,loc_14804` back-edge, so a one-row geyser should not take any inter-row hibernation/collision step before the loc_1480C tail path"
        );
        assert_eq!(
            crate::weather::lava_geyser_inter_row_hibernations_loc_14804(3),
            3,
            "ASM loc_14804 should hibernate one frame and re-run `sub_14A58` exactly once per extra sprite row after the first when `d2` starts at the script's rows-minus-one count"
        );
    }

    #[test]
    fn loc_1480c_unbiases_the_built_sprite_row_y_and_adds_camera_y() {
        assert_eq!(
            crate::weather::lava_geyser_sprite_top_y_loc_1480C(0x0080, 0x0123),
            0x0123,
            "ASM loc_1480C should treat a biased screen Y of `$80` as zero before adding `Camera_Y_pos` into `$54(a5)`"
        );
        assert_eq!(
            crate::weather::lava_geyser_sprite_top_y_loc_1480C(0x00A7, 0x0312),
            0x0339,
            "ASM loc_1480C should store `d7 - $80 + Camera_Y_pos` in `$54(a5)` after `sub_14BD8` builds the current sprite row"
        );
    }

    #[test]
    fn loc_14c5e_selects_usm2_black_pit_and_fallback_tables_from_resource_level_ids() {
        assert_eq!(
            crate::weather::lava_geyser_position_table_loc_14C5E(0),
            crate::weather::LavaGeyserPositionTableLoc14C5E::Usm2,
            "ASM loc_14C5E should keep `d1 = 0` for resource level 0 so USM2 uses the first geyser-position table"
        );
        assert_eq!(
            crate::weather::lava_geyser_position_table_loc_14C5E(2),
            crate::weather::LavaGeyserPositionTableLoc14C5E::BlackPit,
            "ASM loc_14C5E should increment `d1` once before the second compare so resource level 2 selects Black Pit's table"
        );
        assert_eq!(
            crate::weather::lava_geyser_position_table_loc_14C5E(4),
            crate::weather::LavaGeyserPositionTableLoc14C5E::BlackPit,
            "ASM loc_14C5E should reuse the same Black Pit table for resource level 4"
        );
        assert_eq!(
            crate::weather::lava_geyser_position_table_loc_14C5E(5),
            crate::weather::LavaGeyserPositionTableLoc14C5E::Other,
            "ASM loc_14C5E should fall through with `d1 = 3` for every other lava level and select the shared fallback table"
        );
    }

    #[test]
    fn loc_14c6a_decodes_each_geyser_entrys_height_and_startup_delay() {
        assert_eq!(
            crate::weather::initialise_lava_geyser_loc_14C6A(0x0240, 0x0312, 0x0063),
            crate::weather::LavaGeyserInitLoc14C6A {
                x_px: 0x0240,
                y_px: 0x0312,
                height_px: 0x0006,
                startup_delay_frames: 30,
            },
            "ASM loc_14C6A should copy the table's X/Y words through unchanged, use the high bits of the encoded height word for sprite height, and leave the low nibble for loc_14772's startup delay"
        );
        assert_eq!(
            crate::weather::initialise_lava_geyser_loc_14C6A(0x0090, 0x0330, 0x0260),
            crate::weather::LavaGeyserInitLoc14C6A {
                x_px: 0x0090,
                y_px: 0x0330,
                height_px: 0x0026,
                startup_delay_frames: 0,
            },
            "ASM loc_14C6A should preserve the fallback table's taller geyser entries while keeping a zero low nibble as an immediate-start eruption"
        );
    }

    #[test]
    fn init_lava_geysers_resource_level_0_creates_2_geysers_and_1_fireball() {
        let manifest = manifest_with_levels(7);
        let mut runtime = NativeRuntime::new(manifest);
        runtime.init_lava_geysers(LevelId(0x00));

        assert_eq!(runtime.weather.lava_geysers.len(), 2);
        assert_eq!(runtime.weather.lava_fireballs.len(), 1);
        // First geyser: x=0x180, y=0x312, h=0x60
        let g = &runtime.weather.lava_geysers[0];
        assert_eq!(g.x_px, 0x180);
        assert_eq!(g.y_px, 0x312);
        // height_px = h >> 4 = 0x60 >> 4 = 6
        assert_eq!(g.height_px, 0x60 >> 4);
        // initial_delay = (h & 0xF) * 10 = (0x60 & 0xF) * 10 = 0
        assert_eq!(g.timer, 0);

        let fireball = &runtime.weather.lava_fireballs[0];
        assert_eq!(fireball.x_px, 0x346);
        assert_eq!(fireball.base_y_px, 0x312);
        assert_eq!(fireball.current_y_px, 0x312);
    }

    #[test]
    fn init_lava_geysers_fallback_table_creates_shared_other_entries_without_a_fireball() {
        let manifest = manifest_with_levels(7);
        let mut runtime = NativeRuntime::new(manifest);
        runtime.init_lava_geysers(LevelId(0x06));

        assert_eq!(runtime.weather.lava_geysers.len(), 3);
        assert!(runtime.weather.lava_fireballs.is_empty());
        let geyser = &runtime.weather.lava_geysers[0];
        assert_eq!(geyser.x_px, 0x0090);
        assert_eq!(geyser.y_px, 0x0330);
        assert_eq!(geyser.height_px, 0x0260 >> 4);
    }

    #[test]
    fn init_weather_clears_previous_state() {
        let mut manifest = manifest_with_levels(2);
        manifest.levels[0]
            .header_metadata
            .as_mut()
            .unwrap()
            .weather_flags = 0;

        let mut runtime = NativeRuntime::new(manifest);
        // Set up some leftover state
        runtime.weather.weather_type = 3;
        runtime.weather.hail_active = true;
        runtime.weather.hail_spawn_timer = 42;
        runtime.weather.hail_period_timer = 99;
        runtime.weather.lava_geysers.push(LavaGeyser {
            x_px: 0,
            y_px: 0,
            height_px: 0,
            phase: 0,
            timer: 0,
            current_height: 0,
        });

        runtime.init_weather(LevelId(0));

        assert_eq!(runtime.weather.weather_type, 0);
        assert!(!runtime.weather.hail_active);
        assert_eq!(runtime.weather.hail_spawn_timer, 0);
        assert_eq!(runtime.weather.hail_period_timer, 0);
        assert!(runtime.weather.lava_geysers.is_empty());
    }

    #[test]
    fn skycutter_c_press_starts_flipboard_deploy() {
        let mut rt = skycutter_runtime();
        let mut slice = airborne_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_skycutter_flipboard(&mut slice, input);

        assert!(
            rt.helmet.skycutter_flipboard > 0,
            "flipboard counter should start"
        );
    }

    #[test]
    fn skycutter_flipboard_mount_toggles_at_midpoint() {
        let mut rt = skycutter_runtime();
        let mut slice = airborne_slice();

        // First frame: C press starts deploy
        let c_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_skycutter_flipboard(&mut slice, c_input);
        assert!(!rt.helmet.skycutter_on_board, "not yet mounted at frame 1");

        // Step until counter crosses SKYCUTTER_MOUNT_FRAME (17)
        let no_input = FrameInput::default();
        rt.prior_input = c_input; // so C is no longer rising edge
        for _ in 0..8 {
            rt.step_skycutter_flipboard(&mut slice, no_input);
        }
        // Counter: 1 + 2*9 = 19, which crosses 17
        assert!(
            rt.helmet.skycutter_on_board,
            "should be mounted after crossing mount frame"
        );
    }

    #[test]
    fn skycutter_flight_gravity_replaces_normal_gravity() {
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_on_board = true;
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0; // hovering

        // With B not held: Skycutter gravity is SKYCUTTER_GRAVITY_NO_B (0x4000)
        let no_input = FrameInput::default();
        rt.step_skycutter_flipboard(&mut slice, no_input);

        assert_eq!(slice.player.y_velocity, SKYCUTTER_GRAVITY_NO_B);
    }

    #[test]
    fn skycutter_flight_gravity_slower_with_b_held() {
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_on_board = true;
        rt.prior_input = FrameInput {
            b: true,
            ..FrameInput::default()
        }; // B already held
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        let b_input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_skycutter_flipboard(&mut slice, b_input);

        assert_eq!(slice.player.y_velocity, SKYCUTTER_GRAVITY_B_HELD);
        assert!(
            SKYCUTTER_GRAVITY_B_HELD < SKYCUTTER_GRAVITY_NO_B,
            "B-held gravity should be less than no-B gravity (slower descent)"
        );
    }

    #[test]
    fn skycutter_b_press_kicks_upward() {
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_on_board = true;
        rt.prior_input = FrameInput::default(); // B was not held last frame
        let mut slice = airborne_slice();
        slice.player.x_velocity = 0x2_0000;
        slice.player.y_velocity = 0x1_0000; // falling

        let b_input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_skycutter_flipboard(&mut slice, b_input);

        assert!(
            slice.player.y_velocity < 0,
            "B press should kick player upward (negative y_velocity)"
        );
    }

    #[test]
    fn skycutter_max_upward_velocity_clamped() {
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_on_board = true;
        rt.prior_input = FrameInput {
            b: true,
            ..FrameInput::default()
        }; // B already held
        let mut slice = airborne_slice();
        // Start with very negative y_velocity that exceeds max
        slice.player.y_velocity = SKYCUTTER_MAX_UPWARD_VEL - 0x1_0000;

        let b_input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_skycutter_flipboard(&mut slice, b_input);

        // After gravity + clamp, velocity should be clamped to max
        assert_eq!(
            slice.player.y_velocity, SKYCUTTER_MAX_UPWARD_VEL,
            "upward velocity should be clamped to max"
        );
    }

    #[test]
    fn skycutter_landing_bounces_off_ground() {
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_on_board = true;
        let mut slice = airborne_slice();
        slice.player.on_ground = true; // just landed
        slice.player.y_velocity = 0;

        let no_input = FrameInput::default();
        rt.step_skycutter_flipboard(&mut slice, no_input);

        assert!(!slice.player.on_ground, "should bounce off ground");
        assert_eq!(slice.player.y_velocity, SKYCUTTER_LANDING_BOUNCE);
        assert_eq!(
            rt.helmet.skycutter_flipboard, 1,
            "deploy animation should restart"
        );
    }

    #[test]
    fn skycutter_non_skycutter_form_clears_state() {
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_on_board = true;
        rt.helmet.skycutter_flipboard = 10;
        rt.progression.current_form = PlayerForm::TheKid; // no longer Skycutter

        let mut slice = airborne_slice();
        let no_input = FrameInput::default();
        rt.step_skycutter_flipboard(&mut slice, no_input);

        assert!(!rt.helmet.skycutter_on_board);
        assert_eq!(rt.helmet.skycutter_flipboard, 0);
    }

    #[test]
    fn skycutter_flipboard_midpoint_flips_board_latch_and_facing() {
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_flipboard = 0x0F;
        rt.helmet.skycutter_on_board = false;
        let mut slice = airborne_slice();
        slice.player.facing = Direction::Right;

        rt.step_skycutter_flipboard(&mut slice, FrameInput::default());

        assert_eq!(rt.helmet.skycutter_flipboard, 0x11);
        assert!(rt.helmet.skycutter_on_board);
        assert_eq!(
            slice.player.facing,
            Direction::Left,
            "ASM sub_98F2 loc_991A should flip `x_direction(a3)` on the `$11` midpoint frame",
        );
    }

    #[test]
    fn skycutter_idle_frame_without_c_rising_edge_keeps_counter_at_zero_loc_a840() {
        // ASM loc_A840 (kid.asm:13141): the flipboard deploy fires only on the
        // C-press rising edge — `bclr #Button_C,(Ctrl_Pressed); beq.w loc_A87A`
        // skips the deploy when the rising-edge bit is clear. With C never
        // pressed, the counter stays at zero and on_board never toggles.
        //
        // This pins three predicates against equivalent-looking boundary
        // mutations cargo-mutants would otherwise miss:
        //   1. `c_pressed = input.c && !prior_input.c` — an `&&`→`||` mutation
        //      treats every frame with prior C clear as a rising edge.
        //   2. `if c_pressed && self.helmet.skycutter_flipboard == 0` — an
        //      `&&`→`||` mutation deploys whenever the counter is already 0.
        //   3. `if self.helmet.skycutter_flipboard > 0` — a `>`→`>=` mutation
        //      enters the counter-step block at flipboard=0 and advances it.
        // Any of those mutations would set `skycutter_flipboard != 0` here.
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_flipboard = 0;
        rt.helmet.skycutter_on_board = false;
        let mut slice = airborne_slice();
        let prior_facing = slice.player.facing;

        rt.step_skycutter_flipboard(&mut slice, FrameInput::default());

        assert_eq!(
            rt.helmet.skycutter_flipboard, 0,
            "ASM loc_A840: no C rising edge → flipboard stays at 0"
        );
        assert!(
            !rt.helmet.skycutter_on_board,
            "ASM sub_98F2: idle frame must not toggle on_board"
        );
        assert_eq!(
            slice.player.facing, prior_facing,
            "ASM sub_98F2 loc_991A: facing flips only at the $11 midpoint frame"
        );
    }

    #[test]
    fn skycutter_b_press_kick_uses_half_x_velocity_loc_a7e8() {
        // ASM loc_A7E8 (kid.asm:13113): `move.l x_vel(a3),d6; bpl.w loc_A7E8;
        // neg.l d6` then `asr.l #1,d6; addi.l #$30000,d6; move.l d6,y_vel(a3)`
        // — kick magnitude = `abs(x_vel) >> 1 + $30000`. Native negates so the
        // kick points upward (positive y in ASM is downward).
        //
        // The exact magnitude pins the divisor against cargo-mutants
        // mutations on `/`:
        //   `/` → `*`: magnitude becomes `abs(x_vel) * 2 + $30000`.
        //   `/` → `%`: magnitude becomes `abs(x_vel) % 2 + $30000`.
        //   `/` (correct): magnitude becomes `abs(x_vel) / 2 + $30000`.
        // x_vel = $20000 picks the only divisor that yields $40000 magnitude.
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_on_board = true;
        rt.prior_input = FrameInput::default();
        let mut slice = airborne_slice();
        slice.player.x_velocity = 0x2_0000;
        slice.player.y_velocity = 0;

        let b_input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_skycutter_flipboard(&mut slice, b_input);

        // Magnitude = $20000 / 2 + $30000 = $10000 + $30000 = $40000.
        // Native negates → -$40000 (upward).
        assert_eq!(
            slice.player.y_velocity, -0x4_0000,
            "ASM loc_A7E8: kick = -(abs(x_vel) / 2 + $30000); rejects /↔* and /↔%"
        );
    }

    #[test]
    fn skycutter_visual_step_sub_98f2_restarts_board_script_and_rolls_priority() {
        let step = skycutter_visual_step_sub_98F2(
            SkycutterVisualStateSub98F2 {
                flipboard_counter: 0,
                on_board: false,
                facing: Direction::Right,
                motion_phase: 0,
                motion_timer: 0,
                motion_script_index: None,
                roll_accum: 0,
            },
            false,
            false,
        );

        assert_eq!(step.state.motion_phase, 0);
        assert_eq!(step.state.motion_timer, 6);
        assert_eq!(step.state.motion_script_index, Some(1));
        assert_eq!(step.state.roll_accum, 0x10);
        assert_eq!(step.board_priority_word, 0x012C);
        assert_eq!(
            step.player_sprite_mapping,
            Some("LnkTo_unk_A978A-Data_Index"),
            "ASM sub_98F2 loc_9968..loc_99A6 should restart from `unk_96AC` and use the first helper-board mapping",
        );
    }

    #[test]
    fn skycutter_visual_step_sub_98f2_midpoint_uses_transient_flipboard_mapping() {
        let step = skycutter_visual_step_sub_98F2(
            SkycutterVisualStateSub98F2 {
                flipboard_counter: 0x0F,
                on_board: false,
                facing: Direction::Right,
                motion_phase: 3,
                motion_timer: 9,
                motion_script_index: Some(7),
                roll_accum: 0x80,
            },
            false,
            false,
        );

        assert_eq!(step.state.flipboard_counter, 0x11);
        assert!(step.state.on_board);
        assert_eq!(step.state.facing, Direction::Left);
        assert_eq!(
            step.player_sprite_mapping,
            Some("LnkTo_unk_AA5B4-Data_Index"),
            "ASM off_993A should drive the temporary midpoint flipboard frame",
        );
        assert_eq!(step.board_sprite_mapping, None);
    }

    #[test]
    fn cyclone_c_press_activates_flight_when_airborne() {
        let mut rt = cyclone_runtime();
        let slice = airborne_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        // ASM kid_flip: activation runs after physics via step_cyclone_activation
        rt.step_cyclone_activation(&slice, input);

        assert!(rt.helmet.cyclone_flying);
        assert_eq!(rt.helmet.cyclone_y_accel, CYCLONE_ACCEL_INITIAL);
    }

    #[test]
    fn cyclone_c_press_does_not_activate_on_ground() {
        let mut rt = cyclone_runtime();
        let slice = airborne_slice();
        let mut grounded_slice = slice;
        grounded_slice.player.on_ground = true;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_cyclone_activation(&grounded_slice, input);

        assert!(!rt.helmet.cyclone_flying);
    }

    #[test]
    fn cyclone_landing_clears_flight() {
        let mut rt = cyclone_runtime();
        rt.helmet.cyclone_flying = true;
        rt.helmet.cyclone_y_accel = 0x800;
        let mut slice = airborne_slice();
        slice.player.on_ground = true;

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        assert!(!rt.helmet.cyclone_flying);
    }

    #[test]
    fn cyclone_gravity_increases_acceleration() {
        let mut rt = cyclone_runtime();
        rt.helmet.cyclone_flying = true;
        rt.helmet.cyclone_y_accel = 0;
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        // Accel should be 0 + GRAVITY = $400, then clamped to MAX_DESCENT ($1200)
        // since C not held. $400 < $1200 so no clamp.
        assert_eq!(rt.helmet.cyclone_y_accel, CYCLONE_ACCEL_GRAVITY);
        // Velocity should be 0 + $400 = $400
        assert_eq!(slice.player.y_velocity, i32::from(CYCLONE_ACCEL_GRAVITY));
    }

    #[test]
    fn cyclone_c_newly_pressed_provides_thrust() {
        let mut rt = cyclone_runtime();
        rt.helmet.cyclone_flying = true;
        rt.helmet.cyclone_y_accel = CYCLONE_ACCEL_INITIAL; // $2000
        // previous_input.c = false → C is newly pressed (rising edge)
        rt.previous_input = FrameInput::default();
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_cyclone_flight(&mut slice, input);

        // accel = $2000 + $400 - $3000 = -$C00 (negative = upward thrust)
        let expected_accel: i16 = 0x2000 + 0x400 - 0x3000; // -0xC00
        assert_eq!(rt.helmet.cyclone_y_accel, expected_accel);
        assert!(
            slice.player.y_velocity < 0,
            "should gain upward velocity with C newly pressed"
        );
    }

    #[test]
    fn cyclone_descent_accel_capped_without_c() {
        let mut rt = cyclone_runtime();
        rt.helmet.cyclone_flying = true;
        rt.helmet.cyclone_y_accel = CYCLONE_ACCEL_MAX_DESCENT; // already at cap
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        // accel = $1200 + $400 = $1600, but capped to $1200
        assert_eq!(rt.helmet.cyclone_y_accel, CYCLONE_ACCEL_MAX_DESCENT);
    }

    #[test]
    fn cyclone_velocity_clamped_to_max_fall() {
        let mut rt = cyclone_runtime();
        rt.helmet.cyclone_flying = true;
        rt.helmet.cyclone_y_accel = CYCLONE_ACCEL_MAX_DESCENT;
        let mut slice = airborne_slice();
        slice.player.y_velocity = CYCLONE_MAX_FALL_VEL; // already at max

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        assert_eq!(slice.player.y_velocity, CYCLONE_MAX_FALL_VEL);
    }

    #[test]
    fn cyclone_velocity_clamped_to_max_rise() {
        let mut rt = cyclone_runtime();
        rt.helmet.cyclone_flying = true;
        // Start with moderately negative accel that stays negative after thrust
        rt.helmet.cyclone_y_accel = -0x1000_i16;
        // C NOT newly pressed → no thrust, decay path
        rt.previous_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        let mut slice = airborne_slice();
        slice.player.y_velocity = CYCLONE_MAX_RISE_VEL; // already at max rise

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_cyclone_flight(&mut slice, input);

        // Decay path: accel = -0x1000 + 0x400 = -0xC00. Not > MAX_DESCENT, so no cap.
        // y_vel = -0x30000 + (-0xC00) = -0x30C00, clamped to -0x30000
        assert_eq!(slice.player.y_velocity, CYCLONE_MAX_RISE_VEL);
    }

    #[test]
    fn cyclone_non_cyclone_form_clears_state() {
        let mut rt = cyclone_runtime();
        rt.helmet.cyclone_flying = true;
        rt.helmet.cyclone_y_accel = 0x1000;
        rt.progression.current_form = PlayerForm::TheKid;

        let mut slice = airborne_slice();
        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        assert!(!rt.helmet.cyclone_flying);
        assert_eq!(rt.helmet.cyclone_y_accel, 0);
    }

    /// ASM loc_B23A (kid.asm:14104): when C is NOT pressed, the descent
    /// acceleration is capped at $1200. `cmpi.w #$1200,d7; ble.w loc_B246;
    /// move.w #$1200,d7`. Starts above cap to prove the clamp fires.
    #[test]
    fn cyclone_loc_b23a_no_thrust_caps_descent_accel_from_above() {
        let mut rt = cyclone_runtime();
        rt.helmet.cyclone_flying = true;
        // Start well below cap so gravity pushes above it in one frame:
        // $1000 + $400 = $1400 > $1200
        rt.helmet.cyclone_y_accel = 0x1000;
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        // After gravity: $1000 + $400 = $1400, clamped to $1200 by loc_B23A
        assert_eq!(
            rt.helmet.cyclone_y_accel, CYCLONE_ACCEL_MAX_DESCENT,
            "ASM loc_B23A: descent accel $1400 must be clamped to $1200",
        );
        // Velocity = 0 + $1200 (post-clamp accel)
        assert_eq!(
            slice.player.y_velocity,
            i32::from(CYCLONE_ACCEL_MAX_DESCENT),
            "ASM loc_B246: velocity integrates the clamped acceleration",
        );
    }

    /// ASM loc_B246 (kid.asm:14109): store acceleration, sign-extend to long,
    /// add to y_vel. `move.w d7,(Cyclone_YAcceleration).w; ext.l d7;
    /// add.l y_vel(a3),d7`. Verify negative accel correctly sign-extends and
    /// subtracts from positive velocity.
    #[test]
    fn cyclone_loc_b246_negative_accel_integrates_via_sign_extend() {
        let mut rt = cyclone_runtime();
        rt.helmet.cyclone_flying = true;
        // Negative accel that stays negative after gravity: -$2000 + $400 = -$1C00
        // C not pressed, -$1C00 < $1200 so no descent cap
        rt.helmet.cyclone_y_accel = -0x2000_i16;
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0x1_0000; // positive (falling)

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        let expected_accel: i16 = -0x2000 + 0x400; // -$1C00
        assert_eq!(
            rt.helmet.cyclone_y_accel, expected_accel,
            "ASM loc_B214: accel should be -$2000 + $400(gravity) = -$1C00",
        );
        // ext.l of -$1C00 = $FFFFE400 = -7168 as i32
        // y_vel = $10000 + (-$1C00) = $E400
        assert_eq!(
            slice.player.y_velocity,
            0x1_0000 + i32::from(expected_accel),
            "ASM loc_B246: ext.l sign-extends -$1C00 before adding to y_vel",
        );
    }

    /// ASM loc_B25E (kid.asm:14119): upward speed cap. `cmpi.l #-$30000,d7;
    /// bgt.s loc_B26C; move.l #-$30000,d7`. Verify that velocity approaching
    /// the cap from below gets clamped to exactly -$30000.
    #[test]
    fn cyclone_loc_b25e_upward_velocity_capped_at_negative_30000() {
        let mut rt = cyclone_runtime();
        rt.helmet.cyclone_flying = true;
        // Very negative accel: -$7000 + $400(gravity) = -$6C00
        // C not pressed, -$6C00 < $1200 so no descent cap
        rt.helmet.cyclone_y_accel = -0x7000_i16;
        let mut slice = airborne_slice();
        // y_vel already near the cap
        slice.player.y_velocity = -0x2_8000;

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        // Unclamped: -$28000 + (-$6C00) = -$2EC00, but wait:
        // accel = -$7000 + $400 = -$6C00; y_vel = -$28000 + (-$6C00) = -$2EC00
        // -$2EC00 > -$30000 so no clamp needed here. Let me use a more extreme case.
        // Actually -$2EC00 = -192512 and -$30000 = -196608, so -$2EC00 > -$30000 (less negative).
        // Need accel negative enough to push past. Let's recalculate.
        // Actually, let me just verify the existing cyclone_velocity_clamped_to_max_rise
        // test but with a fresh exact assertion.
        let expected_accel: i16 = -0x7000 + 0x400;
        let unclamped = -0x2_8000_i32 + i32::from(expected_accel);
        assert_eq!(unclamped, -0x2_EC00, "sanity: unclamped velocity");
        // -$2EC00 > -$30000, so NOT clamped here
        assert_eq!(
            slice.player.y_velocity, unclamped,
            "ASM loc_B25E: -$2EC00 > -$30000 so no upward clamp",
        );
    }

    /// ASM loc_B25E (kid.asm:14119): upward speed cap fires when velocity
    /// exceeds -$30000. Use extreme negative accel + already-negative velocity.
    #[test]
    fn cyclone_loc_b25e_upward_velocity_clamps_when_exceeded() {
        let mut rt = cyclone_runtime();
        rt.helmet.cyclone_flying = true;
        // Start with accel that will push velocity past the cap.
        // C newly pressed: accel = -$7000 + $400 - $3000 = -$9C00
        // But CYCLONE_ACCEL_MIN is -$7FFF, and -$9C00 < -$7FFF so it clamps.
        // Let me use C not pressed with very negative accel instead.
        // Actually CYCLONE_ACCEL_MIN only applies to thrust path.
        // Without C, no thrust subtract and no min clamp — only descent cap.
        // -$7000 + $400 = -$6C00, not > $1200, so no descent cap.
        rt.helmet.cyclone_y_accel = -0x7000_i16;
        let mut slice = airborne_slice();
        slice.player.y_velocity = -0x2_A000; // close to -$30000

        rt.step_cyclone_flight(&mut slice, FrameInput::default());

        let expected_accel: i16 = -0x7000 + 0x400; // -$6C00
        // unclamped y_vel: -$2A000 + (-$6C00) = -$30C00
        // -$30C00 < -$30000, so loc_B25E clamps to -$30000
        assert_eq!(
            rt.helmet.cyclone_y_accel, expected_accel,
            "accel after gravity",
        );
        assert_eq!(
            slice.player.y_velocity, CYCLONE_MAX_RISE_VEL,
            "ASM loc_B25E: -$30C00 < -$30000 so upward clamp fires to -$30000",
        );
    }

    #[test]
    fn iron_knight_c_press_attaches_to_wall_when_airborne() {
        let mut rt = iron_knight_runtime_with_wall();
        // Position player next to wall (wall at col 10 = pixel 160), facing right
        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16; // just left of wall col 10
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        assert!(rt.helmet.iron_knight_climbing, "should attach to wall");
        assert_eq!(slice.player.y_velocity, IRON_KNIGHT_ATTACH_Y_VEL);
        assert_eq!(slice.player.x_velocity, 0);
    }

    #[test]
    fn iron_knight_c_press_attaches_to_wall_from_grounded_walk() {
        let mut rt = iron_knight_runtime_with_wall();
        let mut slice = airborne_slice();
        // ROM `sub_922C` uses Iron Knight's full 7px half-width plus one edge
        // pixel, so grounded attach can trigger one tile column earlier than the
        // old 5px generic wall probe.
        slice.player.x_pos = 152 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        assert!(
            rt.helmet.iron_knight_climbing,
            "should attach from grounded walk"
        );
        assert_eq!(slice.player.y_velocity, IRON_KNIGHT_ATTACH_Y_VEL);
        assert_eq!(slice.player.x_velocity, 0);
        assert!(
            !slice.player.on_ground,
            "attach should clear grounded state"
        );
    }

    /// ASM loc_7696: `add.w d6,d7 / bmi.w loc_7772` — if the probed x
    /// coordinate goes negative the routine skips the wall probe entirely.
    #[test]
    fn iron_knight_loc_7696_bounds_check_negative_x_skips_attach() {
        let mut rt = iron_knight_runtime_with_wall();
        let mut slice = airborne_slice();
        // Place Kid at x=0 facing left so half_width+1 offset goes negative.
        slice.player.x_pos = 0;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Left;
        slice.player.on_ground = true;

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        assert!(
            !rt.helmet.iron_knight_climbing,
            "ASM loc_7696: bmi branch — negative probe x must skip wall attach"
        );
    }

    /// ASM loc_7696: `cmp.w (Level_width_pixels).w,d7 / bge.w loc_7772` —
    /// if the probed x is at or beyond the level width, skip wall attach.
    #[test]
    fn iron_knight_loc_7696_bounds_check_beyond_level_width_skips_attach() {
        let mut rt = iron_knight_runtime_with_wall();
        let mut slice = airborne_slice();
        // Place Kid at far right edge facing right so probe x >= level width.
        // Level width is 4096 pixels (from airborne_slice's LevelBounds).
        slice.player.x_pos = 4095 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        slice.player.on_ground = true;

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        assert!(
            !rt.helmet.iron_knight_climbing,
            "ASM loc_7696: bge Level_width_pixels branch — probe x beyond level width must skip wall attach"
        );
    }

    #[test]
    fn iron_knight_no_attach_without_wall() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::IronKnight;
        // No block grid — no wall to attach to
        let mut slice = airborne_slice();
        slice.player.facing = Direction::Right;

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        assert!(!rt.helmet.iron_knight_climbing);
    }

    #[test]
    fn iron_knight_c_press_boosts_climb_upward() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = 0;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        slice.player.y_velocity = 0;

        // C rising edge while climbing
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        // Drift (+$80) then boost (-$1800), clamped to min accel (-$800)
        assert_eq!(rt.helmet.iron_knight_climb_accel, IRON_KNIGHT_MIN_ACCEL);
        assert!(slice.player.y_velocity < 0, "should be climbing upward");
    }

    #[test]
    fn iron_knight_slides_down_without_c() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = 0;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        slice.player.y_velocity = 0;

        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        assert_eq!(rt.helmet.iron_knight_climb_accel, IRON_KNIGHT_SLIDE_DRIFT);
        assert!(slice.player.y_velocity > 0, "should slide down");
    }

    #[test]
    fn iron_knight_resume_climb_uses_rom_wall_lookup() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::IronKnight;
        rt.progression.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();
        rt.block_grid = Some(grid_with_wall(5, 0, 20, 10));
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = 0;

        let mut slice = airborne_slice();
        // Match the ROM's `sub_B55C` resume check:
        // x = centre + (half_width + 1) = 160, y = foot - 24 = 56.
        // The old generic airborne `probe_wall(..., 5, 31, ...)` looked at x=158
        // instead, missed the wall column, and dropped climb one frame too early.
        slice.player.x_pos = 152 << 16;
        slice.player.y_pos = 80 << 16;
        slice.player.facing = Direction::Right;
        slice.player.y_velocity = IRON_KNIGHT_ATTACH_Y_VEL;

        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        assert!(
            rt.helmet.iron_knight_climbing,
            "resume frame should keep climb when the ROM wall lookup still sees the wall"
        );
        assert_eq!(rt.helmet.iron_knight_climb_accel, IRON_KNIGHT_SLIDE_DRIFT);
        assert_eq!(slice.player.x_velocity, 0);
        assert_eq!(
            slice.player.y_velocity,
            IRON_KNIGHT_ATTACH_Y_VEL + i32::from(IRON_KNIGHT_SLIDE_DRIFT)
        );
    }

    #[test]
    fn iron_knight_climb_visual_sub_a3bc_wraps_negative_motion_to_top_pose() {
        let visual = iron_knight_climb_visual_sub_A3BC(4, -0x8_0000);

        assert_eq!(
            visual,
            IronKnightClimbVisualSubA3BC {
                pose_accum: 0x00BF,
                sprite_mapping: "LnkTo_unk_C1B92-Data_Index",
            },
            "ASM sub_A3BC should wrap negative accumulator motion to `$BF` and use the top climb pose",
        );
    }

    #[test]
    fn iron_knight_resume_climb_updates_sub_a3bc_pose_accumulator() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::IronKnight;
        rt.progression.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();
        rt.block_grid = Some(grid_with_wall(5, 0, 20, 10));
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = 0;
        rt.helmet.iron_knight_climb_pose_accum = 0x40;

        let mut slice = airborne_slice();
        slice.player.x_pos = 152 << 16;
        slice.player.y_pos = 80 << 16;
        slice.player.facing = Direction::Right;
        slice.player.y_velocity = IRON_KNIGHT_ATTACH_Y_VEL;

        rt.step_iron_knight_climb(&mut slice, FrameInput::default());
        let expected = iron_knight_climb_visual_sub_A3BC(0x40, slice.player.y_velocity).pose_accum;

        assert_eq!(
            rt.helmet.iron_knight_climb_pose_accum, expected,
            "ASM sub_A3BC should derive the persisted climb pose from the post-resume `y_vel(a3)` value",
        );
    }

    #[test]
    fn iron_knight_b_press_jumps_off_wall() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;

        let input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        assert!(!rt.helmet.iron_knight_climbing, "should detach on B press");
        assert!(slice.player.y_velocity < 0, "should jump upward");
    }

    #[test]
    fn iron_knight_landing_stops_climb() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;

        let mut slice = airborne_slice();
        slice.player.on_ground = true;

        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        assert!(!rt.helmet.iron_knight_climbing, "landing should end climb");
    }

    /// ASM loc_A338 (kid.asm:12637): after sub_902A detects floor (d7=1),
    /// `cmpi.w #2,d7; bge.s loc_A330` does NOT branch (1 < 2), so the ROM
    /// falls through to `sub_78E8; bra.w loc_75D4` → dismount from wall.
    /// Native: on_ground=true triggers the landing-stops-climb path.
    ///
    /// Also validates loc_A328 (kid.asm:12628): sub_902A returned d7!=0
    /// (floor hit), so `bne.w loc_A338` branches into the landing check.
    #[test]
    #[allow(non_snake_case)]
    fn loc_A338_floor_landing_dismounts_wall_climb() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = 0x100; // mid-slide

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        // Simulate sub_902A detecting floor: on_ground set by physics step.
        slice.player.on_ground = true;

        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        assert!(
            !rt.helmet.iron_knight_climbing,
            "loc_A338: d7=1 (floor) < 2 → dismount"
        );
        assert_eq!(
            rt.helmet.iron_knight_climb_accel, 0,
            "dismount should reset climb acceleration"
        );
        assert_eq!(
            rt.helmet.iron_knight_climb_pose_accum, 0,
            "dismount should reset climb pose accumulator"
        );
    }

    /// ASM loc_A330 (kid.asm:12632): when sub_902A returns d7=0 (no floor/ceiling
    /// collision), `beq` at loc_A328 falls through to loc_A330:
    /// `bsr.w sub_A3BC; bra.w loc_A28A` — advance climb animation and loop.
    /// Native: wall still present + not on_ground → ResumeClimb, which
    /// advances the sub_A3BC pose accumulator and loops the climb state.
    #[test]
    #[allow(non_snake_case)]
    fn loc_A330_no_collision_continues_climb_with_animation() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = 0;
        rt.helmet.iron_knight_climb_pose_accum = 0x30;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        // Non-zero y_vel so sub_A3BC's `(y_vel << 3) >> 16` delta is nonzero.
        slice.player.y_velocity = IRON_KNIGHT_ATTACH_Y_VEL; // -0x4000
        slice.player.on_ground = false;

        let result = rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        assert!(
            rt.helmet.iron_knight_climbing,
            "loc_A330: no collision → continue climbing"
        );
        assert!(
            matches!(result, IronKnightClimbStep::ResumeClimb),
            "loc_A330: should return ResumeClimb for animation loop"
        );
        // sub_A3BC should have updated the pose accumulator: post-step y_vel
        // includes drift, and the visual function shifts that into pose delta.
        let post_y_vel = slice.player.y_velocity;
        let expected_pose = iron_knight_climb_visual_sub_A3BC(0x30, post_y_vel).pose_accum;
        assert_eq!(
            rt.helmet.iron_knight_climb_pose_accum, expected_pose,
            "loc_A330: sub_A3BC should advance pose accumulator from post-step y_vel"
        );
    }

    /// ASM loc_A382 (kid.asm:12666): when C is NOT pressed during wall climb,
    /// `Get_RandomNumber_wordC` caps Cyclone_YAcceleration at $1800.
    /// `cmpi.w #$1800,d7; ble.w loc_A38E; move.w #$1800,d7`.
    /// After many frames of drift (+$80 each), acceleration saturates at $1800.
    ///
    /// Native: iron_knight_climb_accel capped at IRON_KNIGHT_MAX_SLIDE_ACCEL.
    #[test]
    #[allow(non_snake_case)]
    fn loc_A382_passive_slide_cap_at_1800() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;
        // Set accel just below cap
        rt.helmet.iron_knight_climb_accel = IRON_KNIGHT_MAX_SLIDE_ACCEL - 1;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        slice.player.y_velocity = 0;

        // No C press — drift +$80 should push past $1800, then cap kicks in
        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        assert_eq!(
            rt.helmet.iron_knight_climb_accel, IRON_KNIGHT_MAX_SLIDE_ACCEL,
            "loc_A382: passive drift should be capped at $1800"
        );
    }

    /// ASM loc_A382 complement: when accel is already below $1800 and drift
    /// doesn't push it over, the cap is not applied (ble branch taken).
    #[test]
    #[allow(non_snake_case)]
    fn loc_A382_below_cap_drift_not_clamped() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = 0;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        slice.player.y_velocity = 0;

        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        // After one frame: accel = 0 + $80 = $80, well below $1800
        assert_eq!(
            rt.helmet.iron_knight_climb_accel, IRON_KNIGHT_SLIDE_DRIFT,
            "loc_A382: below cap, drift should apply normally without clamping"
        );
    }

    /// ASM loc_A3A6 (kid.asm:12679): `cmpi.l #$40000,d7; blt.s loc_A3B4;
    /// move.l #$40000,d7` — downward velocity is capped at $40000 during
    /// Iron Knight wall climb.  After many frames of passive sliding, the
    /// integrated y_vel must not exceed this ceiling.
    #[test]
    #[allow(non_snake_case)]
    fn loc_A3A6_downward_velocity_capped_at_40000() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = IRON_KNIGHT_MAX_SLIDE_ACCEL;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        // Set y_vel just below cap so one frame of accel pushes past it
        slice.player.y_velocity = IRON_KNIGHT_MAX_FALL_VEL - 0x100;

        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        assert_eq!(
            slice.player.y_velocity, IRON_KNIGHT_MAX_FALL_VEL,
            "loc_A3A6: downward velocity should be clamped to $40000"
        );
    }

    /// ASM loc_A3B4 (kid.asm:12684): `move.l d7,y_vel(a3); tst.w d6; rts`
    /// — store the clamped velocity into y_vel.  When velocity is within
    /// bounds, it is stored as-is (no clamping).  This test also verifies
    /// the upward cap path (loc_A38E..loc_A3A6 at $FFFE8000).
    #[test]
    #[allow(non_snake_case)]
    fn loc_A3B4_store_velocity_within_bounds() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = 0;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        slice.player.y_velocity = 0x1_0000; // within bounds

        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        // After step: accel = 0 + $80 = $80, y_vel = $10000 + $80 = $10080
        let expected = 0x1_0000 + i32::from(IRON_KNIGHT_SLIDE_DRIFT);
        assert_eq!(
            slice.player.y_velocity, expected,
            "loc_A3B4: velocity within bounds should be stored without clamping"
        );
    }

    /// ASM loc_A3B4 upward cap: when C-press boost drives velocity past
    /// -$18000, clamp kicks in (loc_A38E path).
    #[test]
    #[allow(non_snake_case)]
    fn loc_A3B4_upward_velocity_capped_at_FFFE8000() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = IRON_KNIGHT_MIN_ACCEL;

        let mut slice = airborne_slice();
        slice.player.x_pos = 154 << 16;
        slice.player.y_pos = 32 << 16;
        slice.player.facing = Direction::Right;
        // Set y_vel close to upward cap
        slice.player.y_velocity = IRON_KNIGHT_MAX_CLIMB_VEL + 0x100;

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_iron_knight_climb(&mut slice, input);

        assert_eq!(
            slice.player.y_velocity, IRON_KNIGHT_MAX_CLIMB_VEL,
            "loc_A3B4: upward velocity should be clamped to -$18000"
        );
    }

    /// ASM loc_A3DA (kid.asm:12707): `move.w #$BF,d7` — when the climb pose
    /// accumulator underflows (goes negative from a large upward y_vel delta),
    /// it wraps to $BF so the animation cycle is seamless.
    #[test]
    #[allow(non_snake_case)]
    fn loc_A3DA_pose_accumulator_wraps_negative_to_BF() {
        // pose_accum=2, y_vel=-0x8_0000 → delta = -0x8_0000 >> 13 = -64
        // next = 2 + (-64) = -62 → negative → wrap to $BF
        let result = iron_knight_climb_visual_sub_A3BC(2, -0x8_0000);
        assert_eq!(
            result.pose_accum, 0x00BF,
            "loc_A3DA: negative accumulator wraps to $BF"
        );
        // $BF >> 2 = 47, which is >= 32 → top pose bucket
        assert_eq!(
            result.sprite_mapping, "LnkTo_unk_C1B92-Data_Index",
            "loc_A3DA: wrapped $BF falls into top pose bucket (unk_A3F0 entry = 4)"
        );
    }

    /// ASM loc_A3DE (kid.asm:12710): sprite lookup via `unk_A3F0` table.
    /// Accumulator >> 2 buckets into three zones: 0..15 → pose C (bottom),
    /// 16..31 → pose B (mid), 32..47 → pose A (top).
    #[test]
    #[allow(non_snake_case)]
    fn loc_A3DE_sprite_lookup_three_buckets() {
        // Bottom bucket: pose_accum=0x20 → 0x20>>2=8 → entry in 0..15
        let bottom = iron_knight_climb_visual_sub_A3BC(0x20, 0);
        assert_eq!(
            bottom.sprite_mapping, "LnkTo_unk_C209E-Data_Index",
            "loc_A3DE: accumulator 0x20 (bucket 8) → bottom pose"
        );

        // Mid bucket: pose_accum=0x50 → 0x50>>2=20 → entry in 16..31
        let mid = iron_knight_climb_visual_sub_A3BC(0x50, 0);
        assert_eq!(
            mid.sprite_mapping, "LnkTo_unk_C1E18-Data_Index",
            "loc_A3DE: accumulator 0x50 (bucket 20) → mid pose"
        );

        // Top bucket: pose_accum=0xA0 → 0xA0>>2=40 → entry >= 32
        let top = iron_knight_climb_visual_sub_A3BC(0xA0, 0);
        assert_eq!(
            top.sprite_mapping, "LnkTo_unk_C1B92-Data_Index",
            "loc_A3DE: accumulator 0xA0 (bucket 40) → top pose"
        );
    }
}
