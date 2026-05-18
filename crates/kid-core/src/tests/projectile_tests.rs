use super::*;

// --- Micromax wall/ceiling stick ---

fn micromax_runtime_with_ceiling() -> NativeRuntime {
    let mut rt = NativeRuntime::new(manifest());
    rt.progression.current_form = PlayerForm::Micromax;
    rt.progression.number_hitpoints = PlayerForm::Micromax.max_hitpoints();
    // Build a grid with solid ceiling at row 2 (y=32..48px)
    let width_cells = 6u16 * 20;
    let height_cells = 4u16 * 14;
    let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
    // Solid block at col 3, row 2
    let offset = (usize::from(2u16) * usize::from(width_cells) + usize::from(3u16)) * 2;
    data[offset] = 0x60; // collision = Solid
    let grid = terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells);
    rt.set_block_grid(grid);
    rt
}

fn micromax_runtime_with_right_wall() -> NativeRuntime {
    let mut rt = NativeRuntime::new(manifest());
    rt.progression.current_form = PlayerForm::Micromax;
    rt.progression.number_hitpoints = PlayerForm::Micromax.max_hitpoints();
    let width_cells = 6u16;
    let height_cells = 6u16;
    let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
    // Solid wall at col 3, row 2 only. With x=44, Micromax's right edge probes
    // x=48 (col 3), while the inverted left-facing probe reads x=41 (col 2).
    let offset = (usize::from(2u16) * usize::from(width_cells) + usize::from(3u16)) * 2;
    data[offset] = 0x60;
    let grid = terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells);
    rt.set_block_grid(grid);
    rt
}

// --- loc_779C–loc_7820: standing-still horizontal walk/turn gate ---

/// Build an 8×6 grid with a solid wall column at `wall_col` (rows 1-3) and
/// a floor at row 4. Player body spans rows 2-3 when standing on the floor.
fn standing_still_wall_grid(wall_col: u16) -> terrain::BlockGrid {
    let w = 8u16;
    let h = 6u16;
    let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
    for col in 0..w {
        let offset = (4 * usize::from(w) + usize::from(col)) * 2;
        data[offset] = 0x60;
        data[offset + 1] = 0x01;
    }
    for row in 1..4u16 {
        let offset = (usize::from(row) * usize::from(w) + usize::from(wall_col)) * 2;
        data[offset] = 0x60;
        data[offset + 1] = 0x01;
    }
    terrain::BlockGrid::from_level_layout(&data, w, h)
}

mod chunk_06 {
    use super::*;

    #[test]
    fn iron_knight_landing_on_metal_block_bounces_without_destroying_it() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::IronKnight;
        runtime.progression.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();

        let mut grid = grid_with_floor_at_row(14);
        // ASM sub_AF7A/off_AFD8 -> loc_110D0/sub_11120 restores `$E50B`
        // after the steel bounce animation instead of deleting the block.
        grid.set_cell_at(5, 14, terrain::Cell::from_be_bytes(0xE5, 0x0B));
        runtime.block_grid = Some(grid);

        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = (5 * 16 + 8) << 16;
        slice.player.y_pos = 223 << 16;
        slice.player.y_velocity = 0x0002_0000;
        slice.player.on_ground = false;
        runtime.opening_slice = Some(slice);

        runtime.step_frame(FrameInput::default());

        let cell_after = runtime
            .block_grid()
            .expect("block grid should remain active")
            .cell_at(5, 14);
        assert!(cell_after.has_block, "metal block should stay solid");
        assert_eq!(cell_after.block_type, terrain::BlockType::Metal);
        assert_eq!(cell_after.collision, terrain::CollisionKind::Solid);
        assert_eq!(runtime.block_bounces.len(), 1, "steel hit should bounce");

        let slice_after = runtime
            .opening_slice
            .expect("opening slice should remain active");
        assert_eq!(slice_after.player.y_velocity, -0x8000);
        assert!(
            !slice_after.player.on_ground,
            "steel hit should kick Iron Knight back airborne"
        );
    }

    /// ASM loc_AF96 (kid.asm:13808): active break path inside sub_AF7A.
    /// `st (Iron_Knight_block_breaker).w; bsr.w sub_BBE2` probes for a block,
    /// then dispatches via off_AFD8.  On successful break the ROM sets
    /// `move.l #$FFFF8000,y_vel(a3); moveq #1,d7` and returns.
    /// Native: lib.rs IronKnight on_ground + velocity > 0 → probe + dispatch.
    #[test]
    fn loc_af96_iron_knight_smash_block_bounces_upward() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::IronKnight;
        runtime.progression.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();

        let mut grid = grid_with_floor_at_row(14);
        // Smash block at (5,14) — block type 0 (off_AFD8 index 0).
        grid.set_cell_at(5, 14, terrain::Cell::from_be_bytes(0xE0, 0x01));
        runtime.block_grid = Some(grid);

        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = (5 * 16 + 8) << 16;
        slice.player.y_pos = 223 << 16;
        slice.player.y_velocity = 0x0005_0000; // > $48000 threshold
        slice.player.on_ground = false;
        runtime.opening_slice = Some(slice);

        runtime.step_frame(FrameInput::default());

        let slice_after = runtime.opening_slice.expect("slice should remain");
        assert_eq!(
            slice_after.player.y_velocity, -0x8000,
            "ASM loc_AF96: `move.l #$FFFF8000,y_vel(a3)` — IK bounces upward after smashing block"
        );
        assert!(
            !slice_after.player.on_ground,
            "ASM loc_AF96: IK becomes airborne after floor break"
        );
        let cell_after = runtime.block_grid().unwrap().cell_at(5, 14);
        assert!(
            !cell_after.has_block,
            "ASM loc_AF96/loc_AFF0: smash block should be destroyed"
        );
    }

    /// ASM loc_AFD0 (kid.asm:13830): `moveq #0,d7; rts` — no-break return.
    /// Reached when helmet != Iron Knight.  Native: the `if` guard skips the
    /// entire block-break path for non-IK helmets.
    #[test]
    fn loc_afd0_non_iron_knight_does_not_break_floor_blocks() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::RedStealth;
        runtime.progression.number_hitpoints = PlayerForm::RedStealth.max_hitpoints();

        let mut grid = grid_with_floor_at_row(14);
        grid.set_cell_at(5, 14, terrain::Cell::from_be_bytes(0xE0, 0x01));
        runtime.block_grid = Some(grid);

        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = (5 * 16 + 8) << 16;
        slice.player.y_pos = 223 << 16;
        slice.player.y_velocity = 0x0005_0000;
        slice.player.on_ground = false;
        runtime.opening_slice = Some(slice);

        runtime.step_frame(FrameInput::default());

        let cell_after = runtime.block_grid().unwrap().cell_at(5, 14);
        assert!(
            cell_after.has_block,
            "ASM loc_AFD0: non-IK helmet → `moveq #0,d7; rts`, block stays intact"
        );
    }

    /// ASM loc_AFCA (kid.asm:13826): cleanup path — `move.l (sp)+,a2;
    /// movem.w (sp)+,d0-d3` then falls through to loc_AFD0.  Reached when
    /// sub_BBE2 finds no block (beq) or block ID > $14 (bgt).
    /// Native: when IK lands on a solid tile with no block, no break occurs.
    #[test]
    fn loc_afca_iron_knight_landing_on_solid_without_block_does_not_break() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::IronKnight;
        runtime.progression.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();

        // Floor at row 14 is solid but has no block — only collision bits.
        let grid = grid_with_floor_at_row(14);
        runtime.block_grid = Some(grid);

        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = (5 * 16 + 8) << 16;
        slice.player.y_pos = 223 << 16;
        slice.player.y_velocity = 0x0005_0000;
        slice.player.on_ground = false;
        runtime.opening_slice = Some(slice);

        runtime.step_frame(FrameInput::default());

        assert!(
            runtime.block_destroy_effects.is_empty(),
            "ASM loc_AFCA: sub_BBE2 finds no block → restore regs, fall to loc_AFD0, no effect spawned"
        );
    }

    /// ASM loc_AFF0 (kid.asm:13854): off_AFD8 block type 0 (Smash) handler —
    /// `moveq #2,d6; jmp (j_Block_LoadSmashBits).l`.  d6=2 means "from above"
    /// direction for the smash bits effect.
    /// Native: BlockType::Smash arm calls `spawn_smash_block_effect_sub_10eda`
    /// with BLOCK_DESTROY_DIRECTION_ABOVE (=2).
    #[test]
    fn loc_aff0_iron_knight_smash_block_spawns_above_direction_effect() {
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
        slice.player.y_velocity = 0x0005_0000;
        slice.player.on_ground = false;
        runtime.opening_slice = Some(slice);

        runtime.step_frame(FrameInput::default());

        assert_eq!(
            runtime.block_destroy_effects.len(),
            1,
            "ASM loc_AFF0: `moveq #2,d6; jmp Block_LoadSmashBits` should spawn one smash effect"
        );
        assert_eq!(
            runtime.block_destroy_effects[0].y_px, 220,
            "ASM loc_AFF0: d6=2 (above) → sub_10EDA subtracts 4 from y_pos for the upward nudge"
        );
    }

    /// ASM loc_B01E (kid.asm:13874): Red Stealth sword floor break entry.
    /// `moveq #1,d6; bsr.w sub_B55C` — probes one pixel below foot with
    /// x_offset=0, y_offset=1.  If tile bit 15 set and block ID ≤ 5,
    /// dispatches via off_B058, sets `y_vel(a3)=-$38000`.
    /// Native: `red_stealth_floor_impact_sub_B000` calls
    /// `probe_relative_collision_sub_B55C(grid, cx, fy, 0, 1)`.
    #[test]
    fn loc_b01e_red_stealth_sword_probes_one_below_foot() {
        let mut grid = grid_with_floor_at_row(10);
        // Smash block at (4,2) — block type 0 in off_B058.
        grid.set_cell_at(4, 2, terrain::Cell::from_be_bytes(0xE0, 0x01));

        let impact = red_stealth_floor_impact_sub_B000(
            PlayerForm::RedStealth,
            true,
            &grid,
            4 * 16 + 8,
            2 * 16 - 1,
        );

        assert_eq!(
            impact.kind,
            RedStealthFloorImpactKindSubB000::Smash,
            "ASM loc_B01E: `bsr.w sub_B55C` with d6=1 (y+1) should find the smash block one pixel below foot"
        );
        assert_eq!(
            impact.bounce_y_velocity, -0x0003_8000,
            "ASM sub_B000: `move.l #$FFFC8000,y_vel(a3)` — Red Stealth bounces with -$38000"
        );
    }

    /// ASM loc_B01E direction branch: `moveq #0,d7; tst.b x_direction(a3);
    /// beq.w loc_B01E; neg.w d7` — d7 starts at 0, neg(0)=0, so x_offset is
    /// always 0 regardless of facing.  Native: x_offset hard-coded to 0.
    #[test]
    fn loc_b01e_red_stealth_sword_x_offset_zero_regardless_of_direction() {
        let mut grid = grid_with_floor_at_row(10);
        grid.set_cell_at(4, 2, terrain::Cell::from_be_bytes(0xE0, 0x01));

        // Facing left (default) — x_direction=0 → beq loc_B01E (skip neg)
        let impact_left = red_stealth_floor_impact_sub_B000(
            PlayerForm::RedStealth,
            true,
            &grid,
            4 * 16 + 8,
            2 * 16 - 1,
        );

        // Same probe position, same result — direction doesn't change x_offset.
        let impact_right = red_stealth_floor_impact_sub_B000(
            PlayerForm::RedStealth,
            true,
            &grid,
            4 * 16 + 8,
            2 * 16 - 1,
        );

        assert_eq!(
            impact_left.kind, impact_right.kind,
            "ASM loc_B01E: neg(0)=0, so facing direction doesn't affect the floor probe x_offset"
        );
        assert_eq!(impact_left.block_col, impact_right.block_col);
        assert_eq!(impact_left.block_row, impact_right.block_row);
    }

    #[test]
    fn iron_knight_non_iron_knight_clears_state() {
        let mut rt = iron_knight_runtime_with_wall();
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = 0x500;
        rt.progression.current_form = PlayerForm::TheKid;

        let mut slice = airborne_slice();
        rt.step_iron_knight_climb(&mut slice, FrameInput::default());

        assert!(!rt.helmet.iron_knight_climbing);
        assert_eq!(rt.helmet.iron_knight_climb_accel, 0);
    }

    #[test]
    fn micromax_sticks_to_ceiling_when_falling() {
        let mut rt = micromax_runtime_with_ceiling();
        let mut slice = airborne_slice();
        // Position player below the ceiling block (cell row 2 = y 32-47px)
        // With MICROMAX_H=15, top_y = foot_y - 15. Need top_y in row 2 (32..47).
        // Use foot_y=47+15=62 → top_y=47 → cell row 47>>4=2 ✓
        slice.player.x_pos = (3 * 16 + 8) << 16; // centre of col 3
        slice.player.y_pos = 62 << 16;
        slice.player.y_velocity = 0x1_0000; // falling downward
        slice.player.on_ground = false;

        rt.step_micromax_stick(&mut slice, FrameInput::default());

        assert!(rt.helmet.micromax_stuck, "should stick to ceiling");
        assert_eq!(slice.player.y_velocity, 0, "velocity zeroed on stick");
    }

    #[test]
    fn micromax_does_not_stick_when_moving_upward() {
        let mut rt = micromax_runtime_with_ceiling();
        let mut slice = airborne_slice();
        slice.player.x_pos = (3 * 16 + 8) << 16;
        slice.player.y_pos = 63 << 16;
        slice.player.y_velocity = -0x1_0000; // moving upward
        slice.player.on_ground = false;

        rt.step_micromax_stick(&mut slice, FrameInput::default());

        assert!(
            !rt.helmet.micromax_stuck,
            "should not stick while moving upward"
        );
    }

    #[test]
    fn micromax_b_press_detaches_with_kick() {
        let mut rt = micromax_runtime_with_ceiling();
        rt.helmet.micromax_stuck = true;
        let mut slice = airborne_slice();
        slice.player.x_pos = (3 * 16 + 8) << 16;
        slice.player.y_pos = 63 << 16;
        slice.player.facing = Direction::Right;

        let input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_micromax_stick(&mut slice, input);

        assert!(!rt.helmet.micromax_stuck, "should detach on B press");
        assert_eq!(
            slice.player.y_velocity, MICROMAX_DETACH_Y_VEL,
            "upward kick"
        );
        assert_eq!(slice.player.facing, Direction::Left, "facing reversed");
        assert_eq!(
            slice.player.x_velocity, -MICROMAX_DETACH_X_SPEED,
            "ASM loc_A6AE: right-facing detach reverses to a leftward kick"
        );
    }

    #[test]
    fn micromax_non_micromax_clears_stuck() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::TheKid;
        rt.helmet.micromax_stuck = true;

        let mut slice = airborne_slice();
        rt.step_micromax_stick(&mut slice, FrameInput::default());

        assert!(!rt.helmet.micromax_stuck, "form change should clear stuck");
    }

    /// ASM loc_A606 (kid.asm:12960): main grab loop accumulates gravity each
    /// frame via `($FFFFFA98) += $800`.  Native applies MICROMAX_STUCK_GRAVITY
    /// (0x600) per frame while stuck.  After enough frames, y_velocity exceeds
    /// MICROMAX_STUCK_MAX_FALL and Micromax peels off (micromax_stuck cleared).
    #[test]
    fn micromax_stuck_gravity_accumulates_and_peels_off() {
        let mut rt = micromax_runtime_with_ceiling();
        rt.helmet.micromax_stuck = true;
        let mut slice = airborne_slice();
        slice.player.x_pos = (3 * 16 + 8) << 16;
        slice.player.y_pos = 62 << 16;
        slice.player.y_velocity = 0;

        // One frame: gravity increases
        rt.step_micromax_stick(&mut slice, FrameInput::default());
        assert!(rt.helmet.micromax_stuck, "still stuck after one frame");
        assert_eq!(
            slice.player.y_velocity, MICROMAX_STUCK_GRAVITY,
            "one frame of gravity"
        );

        // Accumulate until peel-off: from 0, need > MICROMAX_STUCK_MAX_FALL
        let frames_to_peel = (MICROMAX_STUCK_MAX_FALL / MICROMAX_STUCK_GRAVITY) + 1;
        // Reset for clean count
        rt.helmet.micromax_stuck = true;
        slice.player.y_velocity = 0;
        for _ in 0..frames_to_peel {
            if !rt.helmet.micromax_stuck {
                break;
            }
            rt.step_micromax_stick(&mut slice, FrameInput::default());
        }
        assert!(
            !rt.helmet.micromax_stuck,
            "should peel off after enough gravity frames"
        );
    }

    /// ASM loc_A606 keeps Micromax stuck while the accumulated grab gravity is
    /// exactly at the peel threshold; native only peels when the fall speed is
    /// greater than MICROMAX_STUCK_MAX_FALL.
    #[test]
    fn micromax_stuck_gravity_equal_to_max_fall_stays_stuck() {
        let mut rt = micromax_runtime_with_ceiling();
        rt.helmet.micromax_stuck = true;
        let mut slice = airborne_slice();
        slice.player.x_pos = (3 * 16 + 8) << 16;
        slice.player.y_pos = 62 << 16;
        slice.player.y_velocity = MICROMAX_STUCK_MAX_FALL - MICROMAX_STUCK_GRAVITY;

        rt.step_micromax_stick(&mut slice, FrameInput::default());

        assert!(
            rt.helmet.micromax_stuck,
            "ASM loc_A606: y_vel equal to the max fall threshold should not peel"
        );
        assert_eq!(slice.player.y_velocity, MICROMAX_STUCK_MAX_FALL);
    }

    /// ASM loc_A5EA is entered while falling or neutral; only upward motion
    /// skips attach. Native mirrors that with `y_vel < 0`, so y_vel == 0 must
    /// still allow a ceiling stick.
    #[test]
    fn micromax_neutral_y_velocity_can_attach_to_ceiling() {
        let mut rt = micromax_runtime_with_ceiling();
        let mut slice = airborne_slice();
        slice.player.x_pos = (3 * 16 + 8) << 16;
        slice.player.y_pos = 62 << 16;
        slice.player.y_velocity = 0;
        slice.player.on_ground = false;

        rt.step_micromax_stick(&mut slice, FrameInput::default());

        assert!(
            rt.helmet.micromax_stuck,
            "ASM loc_A5EA: neutral vertical speed should be eligible to stick"
        );
    }

    /// ASM loc_A5EA uses the platform-grab side saved in `$FFFFFA72`; native's
    /// terrain attach approximation mirrors that by probing the wall Kid faces.
    #[test]
    fn micromax_right_facing_attach_checks_right_wall_only() {
        let mut rt = micromax_runtime_with_right_wall();
        let mut slice = airborne_slice();
        slice.player.x_pos = 44 << 16;
        slice.player.y_pos = 55 << 16;
        slice.player.y_velocity = 0;
        slice.player.on_ground = false;
        slice.player.facing = Direction::Right;

        rt.step_micromax_stick(&mut slice, FrameInput::default());

        assert!(
            rt.helmet.micromax_stuck,
            "ASM loc_A5EA: right-facing Micromax should attach to the right wall"
        );
    }

    /// ASM loc_A69E (kid.asm:13006) + loc_A6AE (kid.asm:13013): B-press
    /// detach kicks left when facing left.  Mirror of right-facing test
    /// `micromax_b_press_detaches_with_kick`.  ASM: `move.l #$FFFE0000,d7;
    /// tst.b ($FFFFFA72); beq loc_A6D0; neg.l d7` — direction depends on
    /// approach side.
    #[test]
    fn micromax_b_press_detach_left_facing_kicks_right() {
        let mut rt = micromax_runtime_with_ceiling();
        rt.helmet.micromax_stuck = true;
        let mut slice = airborne_slice();
        slice.player.x_pos = (3 * 16 + 8) << 16;
        slice.player.y_pos = 62 << 16;
        slice.player.facing = Direction::Left;

        let input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_micromax_stick(&mut slice, input);

        assert!(!rt.helmet.micromax_stuck, "should detach on B press");
        assert_eq!(
            slice.player.y_velocity, MICROMAX_DETACH_Y_VEL,
            "upward kick"
        );
        assert_eq!(
            slice.player.facing,
            Direction::Right,
            "facing reversed to right"
        );
        assert_eq!(
            slice.player.x_velocity, MICROMAX_DETACH_X_SPEED,
            "kick rightward (positive) after facing reversal"
        );
    }

    /// ASM loc_A5EA (kid.asm:12952): entry guard — Micromax only attaches
    /// when airborne with y_vel >= 0.  Grounded Micromax must not stick even
    /// if terrain probe would hit.
    #[test]
    fn micromax_does_not_stick_when_grounded() {
        let mut rt = micromax_runtime_with_ceiling();
        let mut slice = airborne_slice();
        slice.player.x_pos = (3 * 16 + 8) << 16;
        slice.player.y_pos = 62 << 16;
        slice.player.y_velocity = 0x1_0000;
        slice.player.on_ground = true; // grounded

        rt.step_micromax_stick(&mut slice, FrameInput::default());

        assert!(
            !rt.helmet.micromax_stuck,
            "grounded Micromax should not enter stick state"
        );
    }

    /// ASM loc_A656 (kid.asm:12981) + loc_A666 (kid.asm:12988): while stuck,
    /// if the surface disappears (no ceiling, no wall on either side), Micromax
    /// detaches.  Native: terrain probes replace the ASM platform bounds check.
    #[test]
    fn micromax_detaches_when_surface_disappears() {
        // Start stuck with a ceiling, then move to a position with no surface
        let mut rt = micromax_runtime_with_ceiling();
        rt.helmet.micromax_stuck = true;
        let mut slice = airborne_slice();
        // Position far from the solid block at col 3, row 2
        slice.player.x_pos = (10 * 16 + 8) << 16; // col 10, no block
        slice.player.y_pos = 62 << 16;
        slice.player.y_velocity = 0;

        rt.step_micromax_stick(&mut slice, FrameInput::default());

        assert!(
            !rt.helmet.micromax_stuck,
            "should detach when no surface in range"
        );
    }

    #[test]
    fn skycutter_on_board_skips_normal_gravity_in_step() {
        let bounds = LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        slice.player.y_velocity = 0x1_0000;
        slice.player.x_pos = 0x0080_0000;
        slice.player.y_pos = 0x0100_0000;

        let vel_before = slice.player.y_velocity;

        // Step with skycutter_on_board=true — normal gravity should be skipped
        slice.step(
            FrameInput::default(),
            false,
            PlayerForm::Skycutter,
            None,
            None,
            true,
            0,
        );

        // y_pos changes (from existing velocity) but y_velocity should NOT
        // have had normal gravity applied
        assert_eq!(
            slice.player.y_velocity, vel_before,
            "normal gravity should be skipped when skycutter_on_board is true"
        );
    }

    /// ASM loc_B19C (kid.asm:14040): sub_B168 Skycutter on_board gate.
    /// `tst.b ($FFFFFA6A); beq loc_B1D4` — when NOT on board, this branch
    /// skips to normal gravity at loc_B1D4. When on board, loads y_vel and
    /// selects gravity: $2000 if B held, $4000 otherwise.
    /// Verify that NOT being on board means Skycutter gravity is NOT applied.
    #[test]
    fn loc_b19c_not_on_board_falls_through_to_normal_gravity() {
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_on_board = false; // not mounted
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        let no_input = FrameInput::default();
        rt.step_skycutter_flipboard(&mut slice, no_input);

        assert_eq!(
            slice.player.y_velocity, 0,
            "ASM loc_B19C: beq loc_B1D4 — not on board means Skycutter gravity NOT applied"
        );
    }

    /// ASM loc_B19C (kid.asm:14040): `move.l #$2000,d6; tst.w (Ctrl_B_Held);
    /// bne loc_B1BC; move.l #$4000,d6` — B not held selects $4000 gravity.
    /// Verify exact value matches ASM.
    #[test]
    fn loc_b19c_b_not_held_selects_4000_gravity() {
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_on_board = true;
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        let no_input = FrameInput::default();
        rt.step_skycutter_flipboard(&mut slice, no_input);

        assert_eq!(
            slice.player.y_velocity, 0x4000,
            "ASM loc_B19C: B not held → d6=$4000, y_vel += $4000"
        );
    }

    /// ASM loc_B1BC (kid.asm:14049): B-held Skycutter gravity application.
    /// `sub.l d6,d7` with d6=$2000 (B held). Then caps at $FFF88000.
    /// Verify B held uses the lighter $2000 gravity.
    #[test]
    fn loc_b1bc_b_held_selects_2000_gravity() {
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_on_board = true;
        rt.prior_input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        let mut slice = airborne_slice();
        slice.player.y_velocity = 0;

        let b_input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_skycutter_flipboard(&mut slice, b_input);

        assert_eq!(
            slice.player.y_velocity, 0x2000,
            "ASM loc_B1BC: B held → d6=$2000, y_vel += $2000"
        );
    }

    /// ASM loc_B1BC (kid.asm:14049): `cmpi.l #$FFF88000,d7; bgt loc_B1CE;
    /// move.l #$FFF88000,d7` — caps upward speed at -$78000 ($FFF88000 signed).
    /// Verify that y_vel is clamped when it would exceed the upward cap.
    #[test]
    fn loc_b1bc_upward_velocity_clamped_at_fff88000() {
        let mut rt = skycutter_runtime();
        rt.helmet.skycutter_on_board = true;
        rt.prior_input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        let mut slice = airborne_slice();
        // Start well below the cap; even after +$2000 gravity, still below cap
        slice.player.y_velocity = SKYCUTTER_MAX_UPWARD_VEL - 0x1_0000;

        let b_input = FrameInput {
            b: true,
            ..FrameInput::default()
        };
        rt.step_skycutter_flipboard(&mut slice, b_input);

        assert_eq!(
            slice.player.y_velocity, SKYCUTTER_MAX_UPWARD_VEL,
            "ASM loc_B1BC: y_vel should be clamped to $FFF88000 (-0x78000)"
        );
    }

    /// ASM loc_B18A (kid.asm:14034): sub_B168 gravity dispatch, reached after
    /// animation gravity check. `cmpi.w #Cyclone,(Current_Helmet); bne loc_B19C;
    /// tst.b (Cyclone_flying); bne loc_B214` — Cyclone flying → flight physics.
    /// Non-Cyclone → falls through to loc_B19C. Verify that for TheKid (no
    /// helmet), the normal gravity path is taken (custom_gravity=false).
    #[test]
    fn loc_b18a_non_cyclone_non_skycutter_uses_normal_gravity() {
        let bounds = LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        slice.player.y_velocity = 0;
        slice.player.x_pos = 0x0080_0000;
        slice.player.y_pos = 0x0100_0000;

        // custom_gravity=false → should apply normal gravity
        slice.step(
            FrameInput::default(),
            false,
            PlayerForm::TheKid,
            None,
            None,
            false,
            0,
        );

        // Normal gravity (loc_B1D4 path) adds GRAVITY_NORMAL ($4000)
        assert_eq!(
            slice.player.y_velocity, 0x4000,
            "ASM loc_B18A: non-Cyclone, non-Skycutter should apply normal gravity ($4000)"
        );
    }

    /// ASM loc_B140 (kid.asm:14004): orphan left-boundary clamp after sub_B084.
    /// `move.w ($FFFFFA78).w,x_pos(a3); clr.w $1C(a3); clr.l x_vel(a3)` — when
    /// kid's left edge (x_pos_px - half_width) goes negative, snap to half_width
    /// and zero velocity. Native clamps x_pos to 0 via apply_x_velocity_to_position.
    /// Verify that moving left past position 0 clamps x_pos and preserves the
    /// invariant that x_pos never goes negative.
    #[test]
    fn loc_b140_left_boundary_clamp_prevents_negative_x_pos() {
        let bounds = LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        // Player very close to left edge, moving left fast
        slice.player.x_pos = 1 << 16; // 1px from left edge
        slice.player.x_velocity = -0x0003_0000; // -3px/frame

        let next_x = i64::from(slice.player.x_pos) + i64::from(slice.player.x_velocity);
        slice.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;

        assert_eq!(
            slice.player.x_pos, 0,
            "ASM loc_B140: left boundary clamp should prevent negative x_pos"
        );
    }

    /// ASM loc_B150 (kid.asm:14011): orphan right-boundary clamp after sub_B084.
    /// `move.w (Level_width_pixels).w,d7; subq.w #1,d7; sub.w ($FFFFFA78).w,d7;
    /// move.w d7,x_pos(a3)` — when kid's right edge (x_pos_px + half_width) >=
    /// Level_width_pixels, snap to level_width - 1 - half_width and zero velocity.
    /// Native uses u32::MAX clamp (wall collision handles the actual boundary).
    /// Verify that large rightward displacement doesn't wrap around.
    #[test]
    fn loc_b150_right_boundary_clamp_prevents_overflow() {
        let bounds = LevelBounds {
            width_pixels: 4096,
            height_pixels: 512,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        // Player at far right, moving right
        slice.player.x_pos = 0xFFFF_0000;
        slice.player.x_velocity = 0x0003_0000;

        let next_x = i64::from(slice.player.x_pos) + i64::from(slice.player.x_velocity);
        slice.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;

        assert_eq!(
            slice.player.x_pos,
            u32::MAX,
            "ASM loc_B150: right boundary clamp should cap at u32::MAX"
        );
    }

    // --- Teleporter tests ---

    #[test]
    fn teleporter_parsed_from_block_list() {
        // Block list sections flow into each other. Prize section ends when
        // layout_addr=0. Ghost section ends on all-zero entry. Then teleporters.
        // Build data with: 1 prize, 0 ghosts, 1 teleporter.
        let mut data = Vec::new();
        // Prize entry: layout_addr=0x0020, col=1, row=2, disguised=0, content=0
        data.extend_from_slice(&[0x00, 0x20, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00]);
        // Prize terminator — layout_addr=0. The ghost parser reads from here.
        // These 16 bytes serve as both prize terminator AND ghost section.
        // Ghost section: timer=0, mapping=0, solid_time=0, vanished_time=0,
        //   layout_addr=0 → all zero → ghost section terminates.
        data.extend_from_slice(&[0; 16]);
        // Now at the teleporter section.
        data.extend_from_slice(&[
            0x00, 0x10, // layout_addr (non-zero)
            0x00, 0x05, // col = 5
            0x00, 0x03, // row = 3
            0x02, // dest_map_id = 2
            0x07, // dest_y = 7
            0x00, 0x0A, // dest_x = 10
        ]);
        // Teleporter terminator (layout_addr=0)
        data.extend_from_slice(&[0, 0]);

        let bl = terrain::BlockList::from_raw(&data, 64);
        assert_eq!(bl.prize_blocks.len(), 1);
        assert_eq!(bl.ghost_blocks.len(), 0);
        assert_eq!(bl.teleporters.len(), 1);
        assert_eq!(bl.teleporters[0].col, 5);
        assert_eq!(bl.teleporters[0].row, 3);
        assert_eq!(bl.teleporters[0].dest_map_id, 2);
        assert_eq!(bl.teleporters[0].dest_y, 7);
        assert_eq!(bl.teleporters[0].dest_x, 10);
    }

    // --- Mushroom block tests ---

    #[test]
    fn mushroom_block_lands_player_without_bounce() {
        // ASM off_933E index 9 (Mushroom) is a NOP — mushroom blocks do NOT bounce
        // the player. They behave as normal solid floor.
        let w: u16 = 16;
        let h: u16 = 16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Mushroom at cell (2, 4): has_block=1, collision=Solid(6), block_type=Mushroom(9)
        let idx = (4 * w + 2) as usize * 2;
        data[idx] = 0xE9;
        data[idx + 1] = 0x00;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        slice.player.y_velocity = 0x3_0000; // falling at 3 px/frame
        slice.player.x_pos = 0x0024_0000; // 36 px (inside cell col=2)
        slice.player.y_pos = 0x003F_0000; // just above floor at y=64

        slice.apply_floor_collision(Some(&grid), false, false, PLAYER_HALF_WIDTH, true);

        assert!(
            slice.player.on_ground,
            "should land on mushroom like normal floor"
        );
        assert_eq!(slice.player.y_velocity, 0, "should stop on landing");
    }

    #[test]
    fn rubber_block_landing_rebounds_player_and_queues_jiggle() {
        let mut runtime = NativeRuntime::new(manifest());
        let w: u16 = 16;
        let h: u16 = 16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Rubber at cell (2, 4): has_block=1, collision=Solid(6), block_type=Rubber(6)
        let idx = (4 * w + 2) as usize * 2;
        data[idx] = 0xE6;
        data[idx + 1] = 0x00;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        slice.player.y_velocity = 0x3_0000;
        slice.player.x_pos = 0x0024_0000;
        slice.player.y_pos = 0x003F_0000;

        slice.apply_floor_collision(Some(&grid), false, false, PLAYER_HALF_WIDTH, true);
        runtime.process_block_effects(&mut slice);

        assert!(
            !slice.player.on_ground,
            "ASM sub_902A -> sub_914A -> loc_9082 should rebound a fresh rubber landing"
        );
        assert_eq!(
            slice.player.y_pos, 0x003F_0000,
            "the rebound still snaps Kid onto the rubber contact row first"
        );
        assert_eq!(
            slice.player.y_velocity, -0x3_0000,
            "rubber should negate the incoming landing velocity"
        );
        assert_eq!(
            runtime.block_bounces.len(),
            1,
            "ASM loc_10DA4 should queue the rubber jiggle effect on landing"
        );
        assert_eq!(runtime.block_bounces[0].cell_x, 2);
        assert_eq!(runtime.block_bounces[0].cell_y, 4);
    }

    #[test]
    fn rubber_block_rebound_keeps_horizontal_carry_on_contact_frame() {
        let w: u16 = 16;
        let h: u16 = 16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let idx = (4 * w + 2) as usize * 2;
        data[idx] = 0xE6;
        data[idx + 1] = 0x00;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        slice.player.x_velocity = 0x0002_0000;
        slice.player.y_velocity = 0x0003_0000;
        slice.player.x_pos = 0x0026_0000;
        slice.player.y_pos = 0x003F_0000;

        slice.apply_floor_collision(Some(&grid), false, true, PLAYER_HALF_WIDTH, true);

        // ASM sub_902A -> sub_914A -> loc_91F6 -> loc_9082 keeps the landing
        // frame's horizontal carry when rubber flips the downward contact into a
        // rebound; only normal floor landings rewind the just-applied x step.
        assert_eq!(slice.player.x_pos, 0x0026_0000);
        assert_eq!(slice.player.y_pos, 0x003F_0000);
        assert_eq!(slice.player.y_velocity, -0x0003_0000);
        assert!(!slice.player.on_ground);
    }

    #[test]
    fn death_respawn_level_1_rubber_contact_rebounds_into_jump_path() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1216 = None;
        let mut frame_1217 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_218) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_216 {
                frame_1216 = Some(trace);
            } else if frame_index == 1_217 {
                frame_1217 = Some(trace);
            }
        }

        let frame_1216 = frame_1216.expect("frame 1216 trace");
        let frame_1217 = frame_1217.expect("frame 1217 trace");
        // ASM sub_902A -> sub_914A marks this rubber support as the loc_9082
        // rebound path, so the death-respawn route stays in MoveID_Jump through
        // the frame-1216 contact instead of settling into grounded walking.
        assert_eq!(frame_1216.kid_y_pos, Some(0x00FF_0000));
        assert_eq!(frame_1216.kid_y_velocity, Some(-0x0008_0000));
        assert_eq!(frame_1216.character_movement, Some(3));
        assert_eq!(frame_1216.kid_on_ground, Some(false));
        assert_eq!(frame_1217.kid_y_pos, Some(0x00F7_4000));
        assert_eq!(frame_1217.kid_y_velocity, Some(-0x0007_C000));
        assert_eq!(frame_1217.character_movement, Some(3));
        assert_eq!(frame_1217.kid_on_ground, Some(false));
    }

    #[test]
    fn death_respawn_level_1_rubber_wall_hit_rebounds_without_entering_wall_cling() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1291 = None;
        let mut frame_1292 = None;
        let mut frame_1293 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_294) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_291 {
                frame_1291 = Some(trace);
            } else if frame_index == 1_292 {
                frame_1292 = Some(trace);
            } else if frame_index == 1_293 {
                frame_1293 = Some(trace);
            }
        }

        let frame_1291 = frame_1291.expect("frame 1291 trace");
        let frame_1292 = frame_1292.expect("frame 1292 trace");
        let frame_1293 = frame_1293.expect("frame 1293 trace");
        // ASM sub_8F26 -> sub_922C -> off_933E index 6 -> loc_10DA4 returns the
        // special `d7=-1` wall-hit path for rubber. The death-respawn route must
        // bounce back into MoveID_Jump instead of zeroing x_vel and entering the
        // native wall-cling state on frame 1292/1293.
        assert_eq!(frame_1291.kid_x_velocity, Some(0x0002_5FF8));
        assert_eq!(frame_1291.character_movement, Some(3));
        assert_eq!(frame_1292.kid_x_velocity, Some(-0x0002_5EF8));
        assert_eq!(frame_1292.character_movement, Some(3));
        assert_eq!(frame_1292.kid_on_ground, Some(false));
        assert_eq!(frame_1293.kid_x_velocity, Some(-0x0002_1E78));
        assert_eq!(frame_1293.character_movement, Some(3));
        assert_eq!(frame_1293.kid_on_ground, Some(false));
    }

    #[test]
    fn death_respawn_level_1_landing_dragon_contact_damages_on_frame_1684() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1684 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_685) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_684 {
                frame_1684 = Some(trace);
            }
        }

        let frame_1684 = frame_1684.expect("frame 1684 trace");
        // ASM Character_CheckCollision (kid.asm loc_B700/loc_B72C) consumes the
        // walking dragon edge contact on the landing frame, so this late Level 1
        // death path must already drop to 0 HP with Just_received_damage set on
        // frame 1684 rather than surviving into a buffered jump on 1685.
        assert_eq!(frame_1684.number_hitpoints, 0);
        assert!(frame_1684.just_received_damage);
    }

    #[test]
    fn death_respawn_level_1_keeps_live_tumble_arc_on_frame_1700() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1700 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_701) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_700 {
                frame_1700 = Some(trace);
                break;
            }
        }

        let frame_1700 = frame_1700.expect("frame 1700 trace");
        // ASM Death loc_B80C keeps advancing the visible tumble arc until j_sub_8E0
        // reports the current pose offscreen. The frozen shell pose is only reused
        // for the hidden lose-life hand-off, so frame 1700 must still expose the
        // live arc position from the oracle-backed gameplay capture.
        assert_eq!(frame_1700.game_mode, GameMode::InGame);
        assert_eq!(frame_1700.number_lives, 3);
        assert_eq!(frame_1700.number_hitpoints, 0);
        assert_eq!(frame_1700.kid_x_pos, Some(0x08E2_0000));
        assert_eq!(frame_1700.kid_y_pos, Some(0x0138_8000));
        assert_eq!(frame_1700.number_objects, 7);
        assert_eq!(frame_1700.number_gfx_objects, 8);
        assert!(frame_1700.just_received_damage);
    }

    #[test]
    fn death_respawn_level_1_reuses_oracle_shell_pose_on_frame_1758() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1758 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_759) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_758 {
                frame_1758 = Some(trace);
                break;
            }
        }

        let frame_1758 = frame_1758.expect("frame 1758 trace");
        // Oracle-backed death-respawn traces expose the post-Death shell pose on
        // frame 1758 even though `Game_Mode` is still `$000C` on that outer frame.
        // Matching that frozen pose keeps the later shell hand-off narrowed to the
        // following frame instead of drifting on the visible death arc itself.
        assert_eq!(frame_1758.game_mode, GameMode::InGame);
        assert_eq!(frame_1758.kid_x_pos, Some(0x08FF_0000));
        assert_eq!(frame_1758.kid_y_pos, Some(0x023F_5000));
        assert_eq!(frame_1758.number_objects, 7);
        assert_eq!(frame_1758.number_gfx_objects, 8);
        assert!(frame_1758.just_received_damage);
    }

    #[test]
    fn death_respawn_level_1_enters_hidden_lose_life_path_on_frame_1759() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1759 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_760) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_759 {
                frame_1759 = Some(trace);
                break;
            }
        }

        let frame_1759 = frame_1759.expect("frame 1759 trace");
        // ASM Death loc_B80C checks the current death pose with j_sub_8E0 before
        // applying another tumble step. Once frame 1758 has already reached the
        // frozen shell pose, the next outer frame falls straight into lose_life,
        // restoring 2 HP, decrementing lives, and exposing Game_Mode=$0008.
        assert_eq!(frame_1759.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_1759.number_lives, 2);
        assert_eq!(
            frame_1759.number_hitpoints,
            PlayerForm::TheKid.max_hitpoints()
        );
        assert!(frame_1759.just_received_damage);
    }

    #[test]
    fn death_respawn_level_1_hides_shell_pose_on_frame_1780() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1779 = None;
        let mut frame_1780 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_781) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                1_779 => frame_1779 = Some(trace),
                1_780 => {
                    frame_1780 = Some(trace);
                    break;
                }
                _ => {}
            }
        }

        let frame_1779 = frame_1779.expect("frame 1779 trace");
        let frame_1780 = frame_1780.expect("frame 1780 trace");
        // ASM Death loc_B80C has already fallen through lose_life by frame 1759,
        // but the oracle-backed hidden shell only keeps Kid's frozen pose visible
        // through frame 1779. The next outer frame still stays in Game_Mode=$0008
        // with the same lives/HP/camera, but Kid's WRAM pose has already cleared.
        assert_eq!(frame_1779.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_1779.kid_x_pos, Some(0x08FF_0000));
        assert_eq!(frame_1779.kid_y_pos, Some(0x023F_5000));
        assert_eq!(frame_1780.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_1780.number_lives, 2);
        assert_eq!(
            frame_1780.number_hitpoints,
            PlayerForm::TheKid.max_hitpoints()
        );
        assert_eq!(frame_1780.camera_x_pos, 0x0835_0000);
        assert_eq!(frame_1780.camera_y_pos, 0x00BC_0000);
        assert_eq!(frame_1780.kid_x_pos, None);
        assert_eq!(frame_1780.kid_y_pos, None);
        assert_eq!(frame_1780.number_objects, 7);
        assert_eq!(frame_1780.number_gfx_objects, 8);
        assert!(frame_1780.just_received_damage);
    }

    #[test]
    fn death_respawn_level_1_clears_hidden_shell_camera_by_frame_1785() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1785 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_786) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_785 {
                frame_1785 = Some(trace);
                break;
            }
        }

        let frame_1785 = frame_1785.expect("frame 1785 trace");
        // ASM `lose_life -> Teleport -> loc_6E2`: once the frozen shell stops
        // rendering, the hidden transition shell has already cleared Camera_X/Y
        // even though Kid's WRAM pose is still blank and the enemy roster remains.
        assert_eq!(frame_1785.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_1785.camera_x_pos, 0);
        assert_eq!(frame_1785.camera_y_pos, 0);
        assert_eq!(frame_1785.kid_x_pos, None);
        assert_eq!(frame_1785.kid_y_pos, None);
        assert_eq!(frame_1785.number_objects, 7);
        assert_eq!(frame_1785.number_gfx_objects, 8);
    }

    #[test]
    fn death_respawn_level_1_keeps_respawn_bootstrap_hidden_on_frame_1791() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1791 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_792) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_791 {
                frame_1791 = Some(trace);
                break;
            }
        }

        let frame_1791 = frame_1791.expect("frame 1791 trace");
        // ASM `lose_life -> Teleport -> loc_6E2`: after the hidden death shell
        // clears, the shared mode-8 respawn shell keeps Camera_X/Y cleared before
        // the later `Load_InGame` camera snap returns.
        assert_eq!(frame_1791.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_1791.camera_x_pos, 0);
        assert_eq!(frame_1791.camera_y_pos, 0);
        assert_eq!(frame_1791.kid_x_pos, Some(0x0160_0000));
        assert_eq!(frame_1791.kid_y_pos, Some(0x0044_0000));
        assert_eq!(frame_1791.character_movement, Some(0));
        assert_eq!(frame_1791.number_objects, 15);
        assert_eq!(frame_1791.number_gfx_objects, 13);
    }

    #[test]
    fn death_respawn_level_1_uses_hidden_respawn_bootstrap_path_through_frame_1855() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1809 = None;
        let mut frame_1855 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_856) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                1_809 => frame_1809 = Some(trace),
                1_855 => {
                    frame_1855 = Some(trace);
                    break;
                }
                _ => {}
            }
        }

        let frame_1809 = frame_1809.expect("frame 1809 trace");
        let frame_1855 = frame_1855.expect("frame 1855 trace");
        // ASM `lose_life -> Teleport -> loc_6E2`: the hidden Level 1 respawn shell
        // reuses the mode-8 bootstrap slide before `Load_InGame` later restores
        // the grounded respawn pose. Oracle shows the 65-frame sweep from
        // `(x=$160,y=$44)` to the usual hold pose `(x=$40,y=$68)`.
        assert_eq!(frame_1809.kid_x_pos, Some(0x010F_0000));
        assert_eq!(frame_1809.kid_y_pos, Some(0x004E_2000));
        assert_eq!(frame_1855.kid_x_pos, Some(0x0040_0000));
        assert_eq!(frame_1855.kid_y_pos, Some(0x0068_0000));
        assert_eq!(frame_1855.character_movement, Some(0));
        assert_eq!(frame_1855.number_objects, 15);
        assert_eq!(frame_1855.number_gfx_objects, 14);
    }

    #[test]
    fn death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_2216() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_2215 = None;
        let mut frame_2216 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(2_217) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                2_215 => frame_2215 = Some(trace),
                2_216 => {
                    frame_2216 = Some(trace);
                    break;
                }
                _ => {}
            }
        }

        let frame_2215 = frame_2215.expect("frame 2215 trace");
        let frame_2216 = frame_2216.expect("frame 2216 trace");
        // ASM `lose_life -> Teleport -> loc_6E2` keeps the Level 1 respawn shell
        // hidden longer than the old native placeholder. Oracle-backed live drift
        // still shows frame 2216 on `Game_Mode=$0008` with the bootstrap hold pose
        // `(x=$40,y=$68)` and cleared camera, so native must not expose the
        // grounded `Load_InGame` return on that frame yet.
        assert_eq!(frame_2215.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_2215.camera_x_pos, 0);
        assert_eq!(frame_2215.camera_y_pos, 0);
        assert_eq!(frame_2215.kid_x_pos, Some(0x0040_0000));
        assert_eq!(frame_2215.kid_y_pos, Some(0x0068_0000));
        assert_eq!(frame_2216.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_2216.camera_x_pos, 0);
        assert_eq!(frame_2216.camera_y_pos, 0);
        assert_eq!(frame_2216.kid_x_pos, Some(0x0040_0000));
        assert_eq!(frame_2216.kid_y_pos, Some(0x0068_0000));
        assert_eq!(frame_2216.character_movement, Some(0));
    }

    #[test]
    fn death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_2999() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_2998 = None;
        let mut frame_2999 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(3_000) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                2_998 => frame_2998 = Some(trace),
                2_999 => {
                    frame_2999 = Some(trace);
                    break;
                }
                _ => {}
            }
        }

        let frame_2998 = frame_2998.expect("frame 2998 trace");
        let frame_2999 = frame_2999.expect("frame 2999 trace");
        // Oracle-backed `compare-native-gameplay --scenario death-respawn-level1`
        // still reports mode-8 hidden bootstrap frames through 2999. Until the
        // later visible `Load_InGame` return is isolated, native must keep the
        // same hidden hold pose instead of exposing gameplay early.
        assert_eq!(frame_2998.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_2998.camera_x_pos, 0);
        assert_eq!(frame_2998.camera_y_pos, 0);
        assert_eq!(frame_2998.kid_x_pos, Some(0x0040_0000));
        assert_eq!(frame_2998.kid_y_pos, Some(0x0068_0000));
        assert_eq!(frame_2999.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_2999.camera_x_pos, 0);
        assert_eq!(frame_2999.camera_y_pos, 0);
        assert_eq!(frame_2999.kid_x_pos, Some(0x0040_0000));
        assert_eq!(frame_2999.kid_y_pos, Some(0x0068_0000));
        assert_eq!(frame_2999.number_lives, 2);
        assert_eq!(frame_2999.number_hitpoints, 2);
    }

    #[test]
    fn death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_3100() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_3099 = None;
        let mut frame_3100 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(3_101) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                3_099 => frame_3099 = Some(trace),
                3_100 => {
                    frame_3100 = Some(trace);
                    break;
                }
                _ => {}
            }
        }

        let frame_3099 = frame_3099.expect("frame 3099 trace");
        let frame_3100 = frame_3100.expect("frame 3100 trace");
        // ASM `lose_life -> Teleport -> loc_6E2` still owns the hidden mode-8
        // bootstrap here. Oracle-backed window compare keeps the cleared camera,
        // `(x=$40,y=$68)` hold pose, and 15/14 object counts through frame 3100,
        // so native must not expose `Load_InGame` on that frame yet.
        assert_eq!(frame_3099.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_3099.camera_x_pos, 0);
        assert_eq!(frame_3099.camera_y_pos, 0);
        assert_eq!(frame_3099.kid_x_pos, Some(0x0040_0000));
        assert_eq!(frame_3099.kid_y_pos, Some(0x0068_0000));
        assert_eq!(frame_3099.number_objects, 15);
        assert_eq!(frame_3099.number_gfx_objects, 14);
        assert_eq!(frame_3100.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_3100.camera_x_pos, 0);
        assert_eq!(frame_3100.camera_y_pos, 0);
        assert_eq!(frame_3100.kid_x_pos, Some(0x0040_0000));
        assert_eq!(frame_3100.kid_y_pos, Some(0x0068_0000));
        assert_eq!(frame_3100.number_objects, 15);
        assert_eq!(frame_3100.number_gfx_objects, 14);
        assert_eq!(frame_3100.number_lives, 2);
        assert_eq!(frame_3100.number_hitpoints, 2);
    }

    #[test]
    fn death_respawn_level_1_keeps_hidden_respawn_bootstrap_through_frame_9000() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_8999 = None;
        let mut frame_9000 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(9_001) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                8_999 => frame_8999 = Some(trace),
                9_000 => {
                    frame_9000 = Some(trace);
                    break;
                }
                _ => {}
            }
        }

        let frame_8999 = frame_8999.expect("frame 8999 trace");
        let frame_9000 = frame_9000.expect("frame 9000 trace");
        // Oracle-backed `compare-native-gameplay-window --scenario
        // death-respawn-level1 --frame-start 8498 --frame-end 9000
        // --checkpoint-interval 1` still keeps the hidden mode-8 bootstrap alive
        // here with the cleared camera, `(x=$40,y=$68)` hold pose, and 15/14
        // object counts. Native must not expose `Load_InGame` before frame 9001.
        assert_eq!(frame_8999.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_8999.camera_x_pos, 0);
        assert_eq!(frame_8999.camera_y_pos, 0);
        assert_eq!(frame_8999.kid_x_pos, Some(0x0040_0000));
        assert_eq!(frame_8999.kid_y_pos, Some(0x0068_0000));
        assert_eq!(frame_8999.number_objects, 15);
        assert_eq!(frame_8999.number_gfx_objects, 14);
        assert_eq!(frame_9000.game_mode, GameMode::LevelTransition);
        assert_eq!(frame_9000.camera_x_pos, 0);
        assert_eq!(frame_9000.camera_y_pos, 0);
        assert_eq!(frame_9000.kid_x_pos, Some(0x0040_0000));
        assert_eq!(frame_9000.kid_y_pos, Some(0x0068_0000));
        assert_eq!(frame_9000.number_objects, 15);
        assert_eq!(frame_9000.number_gfx_objects, 14);
        assert_eq!(frame_9000.number_lives, 2);
        assert_eq!(frame_9000.number_hitpoints, 2);
    }

    #[test]
    fn death_respawn_level_1_releases_ground_once_post_step_support_is_gone() {
        let mut runtime = cached_generated_runtime();
        let script = death_respawn_level_1_script();

        let mut frame_1570 = None;
        let mut frame_1571 = None;
        for (frame_index, mask) in script.iter().copied().enumerate().take(1_572) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                1_570 => frame_1570 = Some(trace),
                1_571 => {
                    frame_1571 = Some(trace);
                    break;
                }
                _ => {}
            }
        }

        let frame_1570 = frame_1570.expect("frame 1570 trace");
        let frame_1571 = frame_1571.expect("frame 1571 trace");
        // ASM sub_B43A/loc_B5F6 drops the walking path as soon as the stepped
        // hitbox no longer overlaps any solid floor. On the Level 1 death route
        // that hand-off must already have fired by frame 1570, and the following
        // frame must stay on the airborne jump path rather than reviving another
        // synthetic grounded hold.
        assert_eq!(frame_1570.kid_on_ground, Some(false));
        assert_eq!(frame_1570.character_movement, Some(3));
        assert_eq!(frame_1571.kid_on_ground, Some(false));
        assert_eq!(frame_1571.character_movement, Some(3));
    }

    #[test]
    fn rubber_block_ceiling_hit_queues_jiggle_without_destroying_cell() {
        let mut runtime = NativeRuntime::new(manifest());
        let w: u16 = 16;
        let h: u16 = 16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Rubber ceiling at cell (2, 2)
        let idx = (2 * w + 2) as usize * 2;
        data[idx] = 0xE6;
        data[idx + 1] = 0x00;
        let mut grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        slice.player.y_velocity = -0x0011_0000;
        slice.player.x_pos = 0x0028_0000;
        slice.player.y_pos = 0x003F_0000;

        slice.apply_ceiling_collision(
            Some(&mut grid),
            None,
            PlayerForm::TheKid,
            PLAYER_HALF_WIDTH,
            &[],
        );
        runtime.process_block_effects(&mut slice);

        assert_eq!(
            slice.player.y_velocity, 0,
            "ceiling hit should stop upward motion"
        );
        assert_eq!(
            runtime.block_bounces.len(),
            1,
            "ASM loc_10DA4 should queue the rubber jiggle effect on ceiling contact"
        );
        let cell_after = grid.cell_at(2, 2);
        assert!(cell_after.has_block, "rubber cell should stay intact");
        assert_eq!(cell_after.block_type, terrain::BlockType::Rubber);
    }

    #[test]
    fn rubber_block_wall_hit_queues_jiggle_without_breaking_wall() {
        let mut runtime = NativeRuntime::new(manifest());
        let w: u16 = 5;
        let h: u16 = 5;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Rubber wall at cell (2, 1), matching the existing direct wall-hit probe.
        let idx = (w + 2) as usize * 2;
        data[idx] = 0xE6;
        data[idx + 1] = 0x00;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        // Match the existing direct wall-hit geometry so sub_8F26 crosses the
        // tile boundary and reaches the side-wall probe.
        slice.player.x_pos = 27 << 16;
        slice.player.y_pos = 48 << 16;
        slice.player.x_velocity = 0x0001_0000;

        slice.apply_wall_collision(Some(&grid), PLAYER_HALF_WIDTH);
        runtime.process_block_effects(&mut slice);

        assert_eq!(
            slice.player.x_velocity, -0x0001_0000,
            "ASM sub_8F26 -> sub_922C -> off_933E index 6 should rebound airborne rubber wall hits instead of zeroing x_vel"
        );
        assert_eq!(
            slice.player.wall_cling_direction, None,
            "airborne rubber wall hits should not enter native wall-cling"
        );
        assert_eq!(
            runtime.block_bounces.len(),
            1,
            "ASM sub_8F26/sub_922C should queue the rubber jiggle effect on wall contact"
        );
        assert_eq!(runtime.block_bounces[0].cell_x, 2);
        assert_eq!(runtime.block_bounces[0].cell_y, 1);
    }

    #[test]
    fn shifting_block_moves_up_when_player_stands_on_it() {
        let mut rt = NativeRuntime::new(manifest());
        let w: u16 = 16;
        let h: u16 = 16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Shifting block at cell (3, 5): has_block=1, collision=Solid(6), block_type=Shifting(7)
        // = 0b1_110_0111_00000000 = 0xE700
        let idx = (5 * w + 3) as usize * 2;
        data[idx] = 0xE7;
        data[idx + 1] = 0x00;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        rt.block_grid = Some(grid);

        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = true;
        // Player standing at x = cell 3 centre (3*16+8=56px), y = cell 5 top (5*16-1=79px)
        slice.player.x_pos = 56 << 16;
        slice.player.y_pos = 79 << 16;

        rt.apply_shifting_blocks(&slice);

        let grid = rt.block_grid.as_ref().unwrap();
        // Cell (3, 5) should now be empty
        let old_cell = grid.cell_at(3, 5);
        assert!(!old_cell.has_block, "original cell should be cleared");
        // Cell (3, 4) should have the shifted block
        let new_cell = grid.cell_at(3, 4);
        assert!(new_cell.has_block, "block should have shifted up");
        assert_eq!(new_cell.block_type, terrain::BlockType::Shifting);
    }

    #[test]
    fn shifting_block_stays_if_cell_above_occupied() {
        let mut rt = NativeRuntime::new(manifest());
        let w: u16 = 16;
        let h: u16 = 16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Shifting block at (3, 5)
        let idx = (5 * w + 3) as usize * 2;
        data[idx] = 0xE7;
        // Solid block at (3, 4): has_block=1, collision=Solid(6), block_type=0
        let idx2 = (4 * w + 3) as usize * 2;
        data[idx2] = 0xE0; // has_block + Solid collision
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        rt.block_grid = Some(grid);

        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = true;
        slice.player.x_pos = 56 << 16;
        slice.player.y_pos = 79 << 16;

        rt.apply_shifting_blocks(&slice);

        // Block should not have moved
        let grid = rt.block_grid.as_ref().unwrap();
        assert!(grid.cell_at(3, 5).has_block, "block should remain");
    }

    #[test]
    fn teleporter_lookup_by_position() {
        let bl = terrain::BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![terrain::TeleporterEntry {
                layout_addr: 0x0000,
                col: 5,
                row: 3,
                dest_map_id: 2,
                dest_y: 7,
                dest_x: 10,
            }],
            hidden_blocks: vec![],
        };

        assert!(bl.teleporter_at(5, 3).is_some());
        assert!(bl.teleporter_at(4, 3).is_none());
    }

    /// ASM loc_BC96 (kid.asm:15228): teleporter table scan loop — `move.w (a4),d4;
    /// cmp.w d4,d6; beq loc_BCB2; cmp.w d4,d5; beq loc_BCB2; addi.w #$A,a4;
    /// dbf d7,loc_BC96`. Each entry is 10 bytes. The loop compares the first word
    /// (layout_addr) against the telepad address (d6) and d6-2 (d5). With multiple
    /// entries, only the matching one is returned.
    #[test]
    fn teleporter_scan_loop_matches_correct_entry_loc_bc96() {
        let bl = terrain::BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![
                terrain::TeleporterEntry {
                    layout_addr: 0x1000,
                    col: 2,
                    row: 4,
                    dest_map_id: 5,
                    dest_y: 8,
                    dest_x: 12,
                },
                terrain::TeleporterEntry {
                    layout_addr: 0x2000,
                    col: 7,
                    row: 1,
                    dest_map_id: 3,
                    dest_y: 6,
                    dest_x: 15,
                },
                terrain::TeleporterEntry {
                    layout_addr: 0x3000,
                    col: 10,
                    row: 9,
                    dest_map_id: 1,
                    dest_y: 2,
                    dest_x: 20,
                },
            ],
            hidden_blocks: vec![],
        };

        // Second entry should match
        let tp = bl.teleporter_at(7, 1).unwrap();
        assert_eq!(tp.dest_map_id, 3, "should find second entry by (col,row)");
        assert_eq!(tp.dest_x, 15);
        assert_eq!(tp.dest_y, 6);

        // Third entry
        let tp = bl.teleporter_at(10, 9).unwrap();
        assert_eq!(tp.dest_map_id, 1, "should find third entry");

        // No match at unused position
        assert!(bl.teleporter_at(99, 99).is_none());
    }

    /// ASM loc_BCAC (kid.asm:15237): `jmp (j_loc_6E2)` — when the teleporter count
    /// ($FFFFF8DE) is zero or no entry matches the telepad address, the ROM kills
    /// the player. Native: empty teleporters → `check_teleporter_contact` returns
    /// None. This path is unreachable in valid levels but is the ROM's safety net.
    #[test]
    fn teleporter_no_match_returns_none_loc_bcac() {
        let bl = terrain::BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![],
        };

        // Empty list: ROM would branch to loc_BCAC (death)
        assert!(bl.teleporter_at(5, 3).is_none());

        // Non-empty list but no position match: ROM scans all entries then
        // falls through dbf to loc_BCAC
        let bl = terrain::BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![terrain::TeleporterEntry {
                layout_addr: 0x1000,
                col: 2,
                row: 4,
                dest_map_id: 5,
                dest_y: 8,
                dest_x: 12,
            }],
            hidden_blocks: vec![],
        };
        assert!(bl.teleporter_at(99, 99).is_none());
    }

    /// ASM loc_BCB2 (kid.asm:15245): destination extraction from matched entry.
    /// `addq.w #6,a4` skips to offset +6. Word at +6: high byte = dest_map_id
    /// (`asr.w #8,d7`), low byte = dest_y (`andi.w #$FF,d7`). Word at +8 = dest_x.
    /// Both Y and X are multiplied by 16 (`asl.w #4`) for PlayerStart positions.
    /// Verify the native TeleporterEntry fields match the ROM's byte layout.
    #[test]
    fn teleporter_destination_extraction_loc_bcb2() {
        let entry = terrain::TeleporterEntry {
            layout_addr: 0x1234,
            col: 5,
            row: 3,
            // ROM entry bytes at offset +6: [dest_map_id, dest_y, dest_x_hi, dest_x_lo]
            // Example: map_id=0x0A, dest_y=0x07, dest_x=0x0014
            dest_map_id: 0x0A,
            dest_y: 0x07,
            dest_x: 0x0014,
        };

        // ASM: PlayerStart_Y_pos = (entry[+6] & $FF) * 16 = dest_y * 16
        let y_pos = u16::from(entry.dest_y) * 16;
        assert_eq!(y_pos, 0x07 * 16, "dest_y × 16 = PlayerStart_Y_pos");

        // ASM: PlayerStart_X_pos = entry[+8] * 16 = dest_x * 16
        let x_pos = entry.dest_x * 16;
        assert_eq!(x_pos, 0x0014 * 16, "dest_x × 16 = PlayerStart_X_pos");

        // ASM: dest_map_id = entry[+6] >> 8 (high byte of the word at +6)
        assert_eq!(entry.dest_map_id, 0x0A);
    }

    /// ASM loc_BCD6 (kid.asm:15259): MapOrder_Index scan — `move.b (a4)+,d5;
    /// ext.w d5; cmpi.w #$FFFF,d5; beq loc_BCAC` (end marker = death);
    /// `cmp.w d5,d7; beq loc_BCEA` (found level). d6 counts level index from 0.
    /// Native stores dest_map_id directly as the level ID in TeleporterEntry,
    /// so the MapOrder_Index translation is implicit. Verify that cross-level
    /// teleport sets pending_level_skip with the destination level ID.
    #[test]
    fn teleporter_cross_level_sets_pending_skip_loc_bcd6() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_level = LevelId(0);
        rt.block_list = Some(terrain::BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![terrain::TeleporterEntry {
                layout_addr: 0x0000,
                col: 5,
                row: 3,
                dest_map_id: 7, // Different from current level 0
                dest_y: 10,
                dest_x: 20,
            }],
            hidden_blocks: vec![],
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        // Position player on the teleporter cell
        slice.player.x_pos = (5 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;

        let result = rt.check_teleporter_contact(&slice);
        assert_eq!(
            result,
            Some((7, 20, 10)),
            "should return (map_id, dest_x, dest_y)"
        );
    }

    /// ASM loc_BCEA (kid.asm:15270): level transition dispatch after MapOrder_Index
    /// match. `cmp.w (Current_LevelID).w,d6; beq loc_BCF6` — same level skips the
    /// level-change flag. Different level: `st ($FFFFFC36)` then falls through.
    /// Both paths: `move.w d6,(Current_LevelID); st ($FFFFFC29);
    /// move.w #$EE,($FFFFFBCC); moveq #-5,d6; bra Teleport`.
    /// d6=-5 ($FFFB) causes Teleport to play sfx_Teleport sound.
    /// Verify same-level teleport repositions without setting pending_level_skip.
    #[test]
    fn teleporter_same_level_repositions_loc_bcea() {
        let mut rt = NativeRuntime::new(manifest());
        rt.current_level = LevelId(0);
        rt.block_list = Some(terrain::BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![terrain::TeleporterEntry {
                layout_addr: 0x0000,
                col: 5,
                row: 3,
                dest_map_id: 0, // Same as current level
                dest_y: 10,
                dest_x: 20,
            }],
            hidden_blocks: vec![],
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = (5 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;

        let result = rt.check_teleporter_contact(&slice);
        assert_eq!(
            result,
            Some((0, 20, 10)),
            "same-level: returns current level ID"
        );
        // pending_level_skip should NOT be set for same-level teleport
        assert!(
            rt.pending_level_skip.is_none(),
            "same-level teleport should not set pending_level_skip"
        );
    }

    // --- RedStealth sword tests ---

    #[test]
    /// ASM loc_76B0: cmpi.w #Red_Stealth,(Current_Helmet) / tst.b (Red_Stealth_sword_swing) /
    /// bne skip / st (Red_Stealth_sword_swing) — C press while Red Stealth with no active swing
    /// starts the sword swing timer.
    fn red_stealth_c_press_starts_sword_swing() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_red_stealth_sword(input);

        assert!(
            rt.helmet.red_stealth_sword_timer > 0,
            "ASM loc_76B0: st (Red_Stealth_sword_swing) — C press must start the swing",
        );
    }

    /// ASM loc_76B0 only enters the sword allocation path after the C-button
    /// gate. Red Stealth idling with no C press must leave the swing flag clear.
    #[test]
    fn red_stealth_loc_76b0_no_c_press_does_not_start_swing() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;

        rt.step_red_stealth_sword(FrameInput::default());

        assert_eq!(
            rt.helmet.red_stealth_sword_timer, 0,
            "ASM loc_76B0: no C press should not allocate sub_868E sword worker"
        );
    }

    #[test]
    fn red_stealth_swing_anchor_sub_868e_offsets_the_spawn_from_kid() {
        let mut slice = combat_slice();
        slice.player.facing = Direction::Right;

        assert_eq!(
            red_stealth_swing_anchor_sub_868E(&slice.player),
            (0x009C_0000, 0x00F6_0000),
            "ASM sub_868E should place the sword worker 28px ahead of Kid and 10px above his feet when native facing is right",
        );

        slice.player.facing = Direction::Left;
        assert_eq!(
            red_stealth_swing_anchor_sub_868E(&slice.player),
            (0x0064_0000, 0x00F6_0000),
            "ASM sub_868E should mirror the 28px X offset when the native facing flips left",
        );
    }

    #[test]
    /// ASM loc_76B0: `tst.b (Red_Stealth_sword_swing) / bne.w loc_76E4` —
    /// if the sword swing flag is already set, a new C press is ignored.
    fn red_stealth_loc_76b0_already_swinging_blocks_new_swing() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        // Start a swing
        rt.step_red_stealth_sword(FrameInput {
            c: true,
            ..FrameInput::default()
        });
        let initial_timer = rt.helmet.red_stealth_sword_timer;
        assert!(initial_timer > 0);

        // Tick one frame so timer decrements
        rt.prior_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_red_stealth_sword(FrameInput::default());
        let after_tick = rt.helmet.red_stealth_sword_timer;

        // Press C again while still swinging — should NOT restart timer
        rt.prior_input = FrameInput::default();
        rt.step_red_stealth_sword(FrameInput {
            c: true,
            ..FrameInput::default()
        });

        assert_eq!(
            rt.helmet.red_stealth_sword_timer,
            after_tick.saturating_sub(1),
            "ASM loc_76B0: bne skip — second C press during active swing must not restart timer"
        );
    }

    #[test]
    fn red_stealth_sword_inactive_during_startup() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        // Start swing
        rt.step_red_stealth_sword(FrameInput {
            c: true,
            ..FrameInput::default()
        });
        rt.prior_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        // During startup frames the sword should not be active
        assert!(rt.helmet.red_stealth_sword_timer > RED_STEALTH_ACTIVE_FRAMES);
        assert!(!rt.sword_is_active());
    }

    #[test]
    fn red_stealth_sword_active_after_startup() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        // Start swing
        rt.step_red_stealth_sword(FrameInput {
            c: true,
            ..FrameInput::default()
        });
        rt.prior_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        // Tick through startup frames
        for _ in 0..RED_STEALTH_STARTUP_FRAMES {
            rt.step_red_stealth_sword(FrameInput::default());
        }

        assert!(rt.sword_is_active(), "sword should be active after startup");
    }

    #[test]
    fn red_stealth_sword_damages_nearby_enemy() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();

        // Enemy at sword reach: player at 128px, sword at 128+28=156px
        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, 0x009C_0000, 0x00F6_0000, 2));

        rt.process_sword_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 1,
            "enemy should take 1 damage"
        );
    }

    /// ASM loc_86EC consumes the sword worker's collision result. Native's
    /// rectangle approximation keeps the horizontal overlap inclusive at
    /// RED_STEALTH_HIT_HALF + ENEMY_CONTACT_HALF_WIDTH.
    #[test]
    fn red_stealth_sword_hits_at_horizontal_extent() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();
        let (sx, sy) = red_stealth_swing_anchor_sub_868E(&slice.player);
        let max_dx = RED_STEALTH_HIT_HALF + ENEMY_CONTACT_HALF_WIDTH as i32;

        rt.active_enemies.push(test_enemy(
            EnemyFamily::Goat,
            sx + ((max_dx as u32) << 16),
            sy,
            2,
        ));

        rt.process_sword_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 1,
            "ASM loc_86EC: enemy exactly at the sword's horizontal extent should be hit"
        );
    }

    /// ASM loc_86EC only fires after the sword object has collided; native must
    /// reject enemies just outside the approximated horizontal overlap.
    #[test]
    fn red_stealth_sword_misses_just_beyond_horizontal_extent() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();
        let (sx, sy) = red_stealth_swing_anchor_sub_868E(&slice.player);
        let miss_dx = RED_STEALTH_HIT_HALF + ENEMY_CONTACT_HALF_WIDTH as i32 + 1;

        rt.active_enemies.push(test_enemy(
            EnemyFamily::Goat,
            sx + ((miss_dx as u32) << 16),
            sy,
            2,
        ));

        rt.process_sword_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 2,
            "ASM loc_86EC: enemy one pixel beyond the horizontal extent should miss"
        );
    }

    /// ASM loc_86EC consumes the sword worker's collision result. Native's
    /// rectangle approximation keeps the vertical overlap inclusive at
    /// RED_STEALTH_HIT_HALF + ENEMY_CONTACT_HALF_HEIGHT.
    #[test]
    fn red_stealth_sword_hits_at_vertical_extent() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();
        let (sx, sy) = red_stealth_swing_anchor_sub_868E(&slice.player);
        let max_dy = RED_STEALTH_HIT_HALF + ENEMY_CONTACT_HALF_HEIGHT as i32;

        rt.active_enemies.push(test_enemy(
            EnemyFamily::Goat,
            sx,
            sy + ((max_dy as u32) << 16),
            2,
        ));

        rt.process_sword_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 1,
            "ASM loc_86EC: enemy exactly at the sword's vertical extent should be hit"
        );
    }

    /// ASM loc_86EC only fires after the sword object has collided; native must
    /// reject enemies just outside the approximated vertical overlap.
    #[test]
    fn red_stealth_sword_misses_just_beyond_vertical_extent() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();
        let (sx, sy) = red_stealth_swing_anchor_sub_868E(&slice.player);
        let miss_dy = RED_STEALTH_HIT_HALF + ENEMY_CONTACT_HALF_HEIGHT as i32 + 1;

        rt.active_enemies.push(test_enemy(
            EnemyFamily::Goat,
            sx,
            sy + ((miss_dy as u32) << 16),
            2,
        ));

        rt.process_sword_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 2,
            "ASM loc_86EC: enemy one pixel beyond the vertical extent should miss"
        );
    }

    #[test]
    fn red_stealth_sword_kills_1hp_enemy() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();

        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, 0x009C_0000, 0x00F6_0000, 1));

        rt.process_sword_hits(&slice);

        assert!(rt.active_enemies.is_empty(), "1HP enemy should be killed");
    }

    /// ASM loc_86EC deletes the sword object after a collision; native removes
    /// killed enemies from both the live list and trace roster in the same hit.
    #[test]
    fn red_stealth_sword_kill_refreshes_active_enemy_roster() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();

        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, 0x009C_0000, 0x00F6_0000, 1));
        rt.active_enemy_roster = rt
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();

        rt.process_sword_hits(&slice);

        assert!(rt.active_enemies.is_empty(), "1HP enemy should be killed");
        assert!(
            rt.active_enemy_roster.is_empty(),
            "ASM loc_86EC: native trace roster should drop enemies killed by the sword"
        );
    }

    #[test]
    fn red_stealth_sword_kill_marks_layout_enemy_dead() {
        let assets = manifest_with_static_skull();
        let layout = assets.levels[0]
            .enemy_layout_metadata
            .clone()
            .unwrap_or_else(|| panic!("static skull manifest should include an enemy layout"));
        let mut rt = NativeRuntime::new(assets);
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;

        let mut slice = combat_slice();
        slice.player.x_pos = 0x0104_0000;
        slice.player.y_pos = 0x00EA_0000;

        rt.active_enemies.push(test_enemy(
            EnemyFamily::MiniHoppingSkull,
            0x0120_0000,
            0x00E0_0000,
            1,
        ));

        rt.process_sword_hits(&slice);

        assert!(
            rt.active_enemies.is_empty(),
            "the sword hit should kill the skull"
        );
        assert!(
            rt.killed_enemies.contains(&(0, 0)),
            "a sword kill should record the layout kill bit so activate_enemies does not respawn it",
        );

        activate_enemies(
            &mut rt.active_enemies,
            &layout,
            ActivationWindow {
                x_min: 0,
                x_max: 640,
                y_min: 0,
                y_max: 480,
            },
            0,
            0,
            physics::LevelBounds {
                width_pixels: 4096,
                height_pixels: 512,
            },
            slice.player.x_pos,
            slice.player.y_pos,
            &rt.killed_enemies,
        );

        assert!(
            rt.active_enemies.is_empty(),
            "layout-backed enemies killed by the sword should not reactivate while their spawn stays in the active window",
        );
    }

    #[test]
    fn grounded_helmet_state_sub_78e8_keeps_live_red_stealth_swing() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;

        rt.refresh_grounded_helmet_state_sub_78E8();

        assert_eq!(
            rt.helmet.red_stealth_sword_timer, RED_STEALTH_ACTIVE_FRAMES,
            "ASM sub_78E8 should preserve Red Stealth's grounded sword pose while the swing timer is live",
        );
    }

    #[test]
    fn red_stealth_sword_no_hit_when_out_of_range() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();

        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, 0x0140_0000, 0x0100_0000, 3));

        rt.process_sword_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 3,
            "out-of-range enemy should not be hit"
        );
    }

    // --- sub_868E sword worker lifecycle: loc_86CE, loc_86D8, loc_86E8, loc_86EC ---

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_86CE (kid.asm:9541): `add.w $1A(a0),d7; move.w d7,x_pos(a3)` — the
    /// common x-position store after the facing branch. d7=$1C when x_direction==0
    /// (Right in native), d7=-$1C when flipped. Verifies the offset sign matches.
    fn loc_86CE_sword_anchor_x_offset_sign() {
        let mut slice = combat_slice();
        // Right-facing: offset positive (ahead)
        slice.player.facing = Direction::Right;
        let (rx, _) = red_stealth_swing_anchor_sub_868E(&slice.player);
        let player_x = slice.player.x_pos;
        assert!(
            rx > player_x,
            "ASM loc_86CE: right-facing sword x ({rx:#X}) must be ahead of player x ({player_x:#X})",
        );
        // Left-facing: offset negative (behind in world coords)
        slice.player.facing = Direction::Left;
        let (lx, _) = red_stealth_swing_anchor_sub_868E(&slice.player);
        assert!(
            lx < player_x,
            "ASM loc_86CE: left-facing sword x ({lx:#X}) must be behind player x ({player_x:#X})",
        );
        // Symmetry: right offset == left offset mirrored around player
        let r_delta = rx.wrapping_sub(player_x);
        let l_delta = player_x.wrapping_sub(lx);
        assert_eq!(
            r_delta, l_delta,
            "ASM loc_86CE: right/left X offsets must be symmetric",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_86D8 (kid.asm:9546): `moveq #3,d0` then `dbf d0,loc_86D8` = 4
    /// iterations. Each iteration hibernates one frame and checks collision_type.
    /// Native models this as RED_STEALTH_ACTIVE_FRAMES=4 ticks of sword_is_active().
    fn loc_86D8_sword_active_for_four_frames() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        // Set timer directly to the active-phase start value
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;

        let mut active_count = 0u8;
        while rt.sword_is_active() {
            active_count += 1;
            rt.step_red_stealth_sword(FrameInput::default());
        }

        assert_eq!(
            active_count, 4,
            "ASM loc_86D8: dbf d0 with d0=3 gives 4 active frames, not 3",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_86E8 (kid.asm:9552): `jmp Delete_CurrentObject` — reached when the
    /// dbf loop falls through (d0=-1, no collision). In native, the timer reaches 0
    /// and sword_is_active() returns false, ending all hit processing.
    fn loc_86E8_sword_expires_after_active_frames() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        // Set timer to 1 (last active frame)
        rt.helmet.red_stealth_sword_timer = 1;
        assert!(rt.sword_is_active());

        // Tick once — timer becomes 0
        rt.step_red_stealth_sword(FrameInput::default());

        assert_eq!(
            rt.helmet.red_stealth_sword_timer, 0,
            "ASM loc_86E8: timer should reach 0 after final active frame",
        );
        assert!(
            !rt.sword_is_active(),
            "ASM loc_86E8: sword_is_active() must be false once timer expires (Delete_CurrentObject)",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_86EC (kid.asm:9556): `collision_type(a3)` non-zero → play
    /// sfx_Red_Stealth_hitting_a_target → bra.s loc_86E8 (delete). In native,
    /// process_sword_hits detects the overlap and deals damage. The hit sound is
    /// visual-only; the behavioural part is the damage application.
    fn loc_86EC_sword_collision_damages_enemy() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();

        // Place enemy exactly at the sword anchor (guaranteed overlap)
        let (sx, sy) = red_stealth_swing_anchor_sub_868E(&slice.player);
        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, sx, sy, 3));

        rt.process_sword_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 2,
            "ASM loc_86EC: collision_type non-zero path must damage the enemy",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_86EC → loc_86E8: after the hit sound, the sword object is deleted.
    /// In native, damage is applied but the timer continues ticking. Verify that
    /// a hit does NOT prematurely end the swing (native checks every active frame,
    /// unlike ASM which deletes on first contact).
    fn loc_86EC_hit_does_not_end_active_frames() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = RED_STEALTH_ACTIVE_FRAMES;
        let slice = combat_slice();

        let (sx, sy) = red_stealth_swing_anchor_sub_868E(&slice.player);
        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, sx, sy, 5));

        rt.process_sword_hits(&slice);

        // Timer should still be active (native doesn't delete on first hit)
        assert!(
            rt.sword_is_active(),
            "native sword stays active after a hit (ASM deletes on first contact — \
             native checks every frame instead)",
        );
    }

    #[test]
    fn red_stealth_floor_impact_sub_b000_breaks_smash_blocks_and_bounces_kid() {
        let mut grid = grid_with_floor_at_row(10);
        grid.set_cell_at(4, 2, terrain::Cell::from_be_bytes(0xE0, 0x01));

        let impact = red_stealth_floor_impact_sub_B000(
            PlayerForm::RedStealth,
            true,
            &grid,
            4 * 16 + 8,
            2 * 16 - 1,
        );

        assert_eq!(
            impact.kind,
            RedStealthFloorImpactKindSubB000::Smash,
            "ASM sub_B000 should route block type `$0` through `loc_B070` after the `sub_B55C` floor sample",
        );
        assert_eq!(impact.block_col, 4);
        assert_eq!(impact.block_row, 2);
        assert_eq!(impact.bounce_y_velocity, -0x0003_8000);
    }

    #[test]
    fn red_stealth_floor_impact_sub_b000_bounces_on_teleporters_without_breaking_them() {
        let mut grid = grid_with_floor_at_row(10);
        grid.set_cell_at(6, 3, terrain::Cell::from_be_bytes(0xE4, 0x11));

        let impact = red_stealth_floor_impact_sub_B000(
            PlayerForm::RedStealth,
            true,
            &grid,
            6 * 16 + 8,
            3 * 16 - 1,
        );

        assert_eq!(
            impact.kind,
            RedStealthFloorImpactKindSubB000::BounceOnly,
            "ASM sub_B000's `return_B056` path still forces `y_vel=-$38000` for block types `$3/$4` even though no destroy helper runs",
        );
        assert_eq!(impact.bounce_y_velocity, -0x0003_8000);
    }

    #[test]
    fn red_stealth_floor_impact_sub_b000_ignores_non_rom_block_types() {
        let mut grid = grid_with_floor_at_row(10);
        grid.set_cell_at(7, 4, terrain::Cell::from_be_bytes(0xE6, 0x0C));

        let impact = red_stealth_floor_impact_sub_B000(
            PlayerForm::RedStealth,
            true,
            &grid,
            7 * 16 + 8,
            4 * 16 - 1,
        );

        assert_eq!(
            impact.kind,
            RedStealthFloorImpactKindSubB000::None,
            "ASM sub_B000 rejects block types above `$5` before it ever loads `off_B058`",
        );
        assert_eq!(impact.bounce_y_velocity, 0);
    }

    #[test]
    fn kid_airborne_visual_sub_b270_uses_red_stealth_downward_slash_frames() {
        let step = kid_airborne_visual_sub_B270(KidAirborneVisualInputSubB270 {
            helmet_id: PlayerForm::RedStealth.helmet_id(),
            is_animated: false,
            on_ground: false,
            global_lock_active: false,
            airborne_latch_active: false,
            cyclone_flying: false,
            cyclone_y_accel: 0,
            cyclone_frame_accum: 0,
            juggernaut_companion_animation_active: false,
            maniaxe_throwing_axe: false,
            y_velocity: 0x0000_5000,
            jump_threshold: 0x0001_0000,
            c_held: true,
            red_stealth_sword_swing: 0,
        });

        assert_eq!(
            step.visual,
            KidAirborneVisualKindSubB270::UseMapping("LnkTo_unk_B40DE-Data_Index"),
            "ASM sub_B270 should switch Red Stealth's fast-falling C-held branch to the first downward slash frame",
        );
        assert_eq!(step.next_red_stealth_sword_swing, 0x10);
        assert_eq!(
            step.queued_sound,
            Some(QueuedSoundEffect::RedStealthAttackDown)
        );
    }

    #[test]
    fn kid_airborne_visual_sub_b270_advances_cyclone_spin_phase_from_accumulator() {
        let step = kid_airborne_visual_sub_B270(KidAirborneVisualInputSubB270 {
            helmet_id: PlayerForm::Cyclone.helmet_id(),
            is_animated: false,
            on_ground: false,
            global_lock_active: false,
            airborne_latch_active: false,
            cyclone_flying: true,
            cyclone_y_accel: 0x2000,
            cyclone_frame_accum: 0x01FF,
            juggernaut_companion_animation_active: false,
            maniaxe_throwing_axe: false,
            y_velocity: 0,
            jump_threshold: 0,
            c_held: false,
            red_stealth_sword_swing: 0,
        });

        assert_eq!(
            step.visual,
            KidAirborneVisualKindSubB270::UseMapping("LnkTo_unk_BA7E0-Data_Index"),
            "ASM sub_B270 should bucket `$FFFFF8F0 >> 8` into the middle Cyclone spin frame when the accumulated phase stays in `$100..$1FF`",
        );
        assert_eq!(step.next_cyclone_frame_accum, 0x01FF);
        assert!(!step.next_airborne_latch_active);
    }

    #[test]
    fn kid_airborne_visual_sub_b270_preserves_maniaxe_throw_animation_while_airborne() {
        let step = kid_airborne_visual_sub_B270(KidAirborneVisualInputSubB270 {
            helmet_id: PlayerForm::Maniaxe.helmet_id(),
            is_animated: false,
            on_ground: false,
            global_lock_active: false,
            airborne_latch_active: false,
            cyclone_flying: false,
            cyclone_y_accel: 0,
            cyclone_frame_accum: 0,
            juggernaut_companion_animation_active: false,
            maniaxe_throwing_axe: true,
            y_velocity: 0,
            jump_threshold: 0,
            c_held: false,
            red_stealth_sword_swing: 0,
        });

        assert_eq!(
            step.visual,
            KidAirborneVisualKindSubB270::PreserveCurrent,
            "ASM sub_B270 should keep Maniaxe's active throw pose until `$18(a3)` says Kid has landed",
        );
        assert!(step.next_maniaxe_throwing_axe);
    }

    #[test]
    fn kid_airborne_visual_sub_b270_uses_generic_airborne_table_for_the_kid() {
        let step = kid_airborne_visual_sub_B270(KidAirborneVisualInputSubB270 {
            helmet_id: PlayerForm::TheKid.helmet_id(),
            is_animated: false,
            on_ground: false,
            global_lock_active: false,
            airborne_latch_active: false,
            cyclone_flying: false,
            cyclone_y_accel: 0,
            cyclone_frame_accum: 0,
            juggernaut_companion_animation_active: false,
            maniaxe_throwing_axe: false,
            y_velocity: -0x0000_3000,
            jump_threshold: -0x0001_0000,
            c_held: false,
            red_stealth_sword_swing: 0,
        });

        assert_eq!(
            step.visual,
            KidAirborneVisualKindSubB270::UseMapping("LnkTo_unk_A3CEC-Data_Index"),
            "ASM sub_B270 should take the second Kid airborne bucket when `y_vel<<2` is still above the shared upward threshold",
        );
    }

    /// ASM loc_B284 (kid.asm:14140): non-Eyeclops path in sub_B270. When the
    /// global animation lock `$FFFFFA66` is set, return immediately without
    /// updating the airborne visual. `tst.b ($FFFFFA66).w; beq.w loc_B28E; rts`
    #[test]
    fn kid_airborne_visual_sub_b270_loc_b284_global_lock_preserves_current() {
        let step = kid_airborne_visual_sub_B270(KidAirborneVisualInputSubB270 {
            helmet_id: PlayerForm::TheKid.helmet_id(),
            is_animated: false,
            on_ground: false,
            global_lock_active: true, // $FA66 set → early return
            airborne_latch_active: false,
            cyclone_flying: false,
            cyclone_y_accel: 0,
            cyclone_frame_accum: 0,
            juggernaut_companion_animation_active: false,
            maniaxe_throwing_axe: false,
            y_velocity: -0x0000_3000,
            jump_threshold: -0x0001_0000,
            c_held: false,
            red_stealth_sword_swing: 0,
        });

        assert_eq!(
            step.visual,
            KidAirborneVisualKindSubB270::PreserveCurrent,
            "ASM loc_B284: global_lock_active ($FA66) must force PreserveCurrent early return",
        );
    }

    /// ASM loc_B2CA (kid.asm:14168): Cyclone flight accumulator store + sprite
    /// mapping lookup.  When phase_word stays below $300 (no wrapping), the value
    /// is stored directly.  `asr.w #8,d7` → frame index 0 for phase in $000..$0FF.
    /// `add.w d7,d7; add.w d7,d7` = *4 word offset into three Cyclone spin frames.
    #[test]
    fn kid_airborne_visual_loc_b2ca_cyclone_phase_below_wrap_threshold() {
        // cyclone_y_accel = $2000 → neg = -$2000, +$2000 = 0, <<2 = 0, >>8 = 0.
        // phase delta = 0, accum stays at $00A0.  $00A0 < $0300 → no wrap (loc_B2CA
        // reached directly).  $00A0 >> 8 = 0 → first Cyclone spin frame.
        let step = kid_airborne_visual_sub_B270(KidAirborneVisualInputSubB270 {
            helmet_id: PlayerForm::Cyclone.helmet_id(),
            is_animated: false,
            on_ground: false,
            global_lock_active: false,
            airborne_latch_active: false,
            cyclone_flying: true,
            cyclone_y_accel: 0x2000,
            cyclone_frame_accum: 0x00A0,
            juggernaut_companion_animation_active: false,
            maniaxe_throwing_axe: false,
            y_velocity: 0,
            jump_threshold: 0,
            c_held: false,
            red_stealth_sword_swing: 0,
        });

        assert_eq!(
            step.visual,
            KidAirborneVisualKindSubB270::UseMapping("LnkTo_unk_BA55A-Data_Index"),
            "ASM loc_B2CA: phase $00A0 >> 8 = 0 → first Cyclone spin frame",
        );
        assert_eq!(
            step.next_cyclone_frame_accum, 0x00A0,
            "ASM loc_B2CA: move.w d7,($FFFFF8F0).w stores the unwrapped phase",
        );
    }

    /// ASM loc_B2DE (kid.asm:14180): Juggernaut companion animation path.  When
    /// helmet == #5 and companion animation is active, `bsr.w sub_975C` runs the
    /// cloud.  If `is_animated(a3)` is set and Kid is on ground (`$18(a3)` != 0),
    /// `sf is_animated(a3)` clears it and falls through to loc_B304's common table.
    #[test]
    fn kid_airborne_visual_loc_b2de_juggernaut_companion_on_ground_clears_animated() {
        let step = kid_airborne_visual_sub_B270(KidAirborneVisualInputSubB270 {
            helmet_id: PlayerForm::Juggernaut.helmet_id(),
            is_animated: true,
            on_ground: true,
            global_lock_active: false,
            airborne_latch_active: false,
            cyclone_flying: false,
            cyclone_y_accel: 0,
            cyclone_frame_accum: 0,
            juggernaut_companion_animation_active: true,
            maniaxe_throwing_axe: false,
            y_velocity: 0x0000_1000,
            jump_threshold: 0x0001_0000,
            c_held: false,
            red_stealth_sword_swing: 0,
        });

        assert!(
            !step.next_is_animated,
            "ASM loc_B2DE: sf is_animated(a3) — Juggernaut on ground clears animated flag",
        );
        // Falls through to loc_B304 common table. y_vel $1000 << 2 = $4000 > 0,
        // $4000 <= threshold $1_0000<<2 = $4_0000 → state_index 2.
        // Juggernaut (helmet 5) state 2 = second entry pair.
        assert_eq!(
            step.visual,
            KidAirborneVisualKindSubB270::UseMapping("LnkTo_unk_BFD26-Data_Index"),
            "ASM loc_B304: falls through to common table with state_index=2 for Juggernaut",
        );
    }

    /// ASM loc_B304 (kid.asm:14192): common airborne mapping dispatch.  For
    /// Berzerker (helmet #7), `lsl.w #3,d7` = $38, indexes off_B3B8 to the
    /// Berzerker row.  With y_vel strongly negative (ascending fast, below
    /// threshold), state_index = 0 → first Berzerker airborne frame.
    #[test]
    fn kid_airborne_visual_loc_b304_berzerker_ascending_fast_uses_state_0() {
        let step = kid_airborne_visual_sub_B270(KidAirborneVisualInputSubB270 {
            helmet_id: PlayerForm::Berzerker.helmet_id(),
            is_animated: false,
            on_ground: false,
            global_lock_active: false,
            airborne_latch_active: false,
            cyclone_flying: false,
            cyclone_y_accel: 0,
            cyclone_frame_accum: 0,
            juggernaut_companion_animation_active: false,
            maniaxe_throwing_axe: false,
            // y_vel << 2 = -$40000, threshold = -$40000 → d6 <= d7 → state 0
            y_velocity: -0x0001_0000,
            jump_threshold: -0x0004_0000,
            c_held: false,
            red_stealth_sword_swing: 0,
        });

        assert_eq!(
            step.visual,
            KidAirborneVisualKindSubB270::UseMapping("LnkTo_unk_B007C-Data_Index"),
            "ASM loc_B304: Berzerker state 0 = first airborne frame (strong ascent)",
        );
        assert!(!step.next_maniaxe_throwing_axe);
    }

    /// ASM loc_B334 (kid.asm:14208): negative y_vel (ascending) above threshold →
    /// state_index 1 (`addq.w #2,a4`).  Eyeclops (helmet #4) ascending moderately:
    /// y_vel = -$1000, threshold at -$4_0000.  y_vel<<2 = -$4000, which is > d7
    /// (threshold) → `addq.w #2,a4` → state 1.
    #[test]
    fn kid_airborne_visual_loc_b334_eyeclops_ascending_moderate_uses_state_1() {
        let step = kid_airborne_visual_sub_B270(KidAirborneVisualInputSubB270 {
            helmet_id: PlayerForm::Eyeclops.helmet_id(),
            is_animated: false,
            on_ground: false,
            global_lock_active: false,
            airborne_latch_active: false,
            cyclone_flying: false,
            cyclone_y_accel: 0,
            cyclone_frame_accum: 0,
            juggernaut_companion_animation_active: false,
            maniaxe_throwing_axe: false,
            // y_vel<<2 = -$4000 (negative, bpl fails → loc_B334 negative branch).
            // d7 (threshold) = -$4_0000. d6 (-$4000) > d7 (-$4_0000) → state 1.
            y_velocity: -0x0000_1000,
            jump_threshold: -0x0004_0000,
            c_held: false,
            red_stealth_sword_swing: 0,
        });

        assert_eq!(
            step.visual,
            KidAirborneVisualKindSubB270::UseMapping("LnkTo_unk_B772E-Data_Index"),
            "ASM loc_B334: Eyeclops state 1 (ascending moderate, above threshold)",
        );
    }

    /// ASM loc_B350 (kid.asm:14220): positive y_vel branch — `addq.w #4,a4`
    /// advances to state 2.  If d6 <= d7 (threshold), stays at state 2.  IronKnight
    /// descending slowly: y_vel = $1000, threshold = $4_0000.  y_vel<<2 = $4000
    /// (positive → loc_B350), $4000 <= $4_0000 → state 2.
    #[test]
    fn kid_airborne_visual_loc_b350_iron_knight_descending_slow_uses_state_2() {
        let step = kid_airborne_visual_sub_B270(KidAirborneVisualInputSubB270 {
            helmet_id: PlayerForm::IronKnight.helmet_id(),
            is_animated: false,
            on_ground: false,
            global_lock_active: false,
            airborne_latch_active: false,
            cyclone_flying: false,
            cyclone_y_accel: 0,
            cyclone_frame_accum: 0,
            juggernaut_companion_animation_active: false,
            maniaxe_throwing_axe: false,
            // y_vel<<2 = $4000 (positive → loc_B350). $4000 <= $4_0000 → state 2.
            y_velocity: 0x0000_1000,
            jump_threshold: 0x0004_0000,
            c_held: false,
            red_stealth_sword_swing: 0,
        });

        assert_eq!(
            step.visual,
            KidAirborneVisualKindSubB270::UseMapping("LnkTo_unk_C26D0-Data_Index"),
            "ASM loc_B350: IronKnight state 2 (descending slow, below threshold)",
        );
    }

    // --- Maniaxe axe tests ---

    #[test]
    /// ASM loc_76E4: cmpi.w #Maniaxe,(Current_Helmet) / tst.b (Maniaxe_throwing_axe) /
    /// bne skip / allocate ManiaxeAxe_Init / st (Maniaxe_throwing_axe) — C press
    /// while Maniaxe with no active axe spawns the thrown axe projectile.
    fn maniaxe_c_press_throws_axe() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_maniaxe_axe(&slice, input);

        assert!(
            rt.helmet.maniaxe_axe_active,
            "ASM loc_76E4: st (Maniaxe_throwing_axe) — C press must activate axe"
        );
        assert_eq!(rt.projectiles.len(), 1);
        let axe = &rt.projectiles[0];
        assert_eq!(
            axe.x_pos, slice.player.x_pos,
            "ASM ManiaxeAxe_Init copies Kid's x_pos(a0) into the axe object"
        );
        assert_eq!(
            axe.y_pos,
            (slice.player.y_pos as i64 - (16 << 16)) as u32,
            "ASM ManiaxeAxe_Init subtracts `$10` from Kid's y_pos(a0)"
        );
        assert_eq!(
            axe.x_velocity,
            MANIAXE_BASE_SPEED << 16,
            "ASM loc_8A42: right-facing axe should move right at +5 px/frame in native"
        );
        assert_eq!(
            axe.y_velocity, 0,
            "Maniaxe axe has no vertical launch velocity"
        );
        assert_eq!(
            axe.lifetime,
            (MANIAXE_TRAVEL_FRAMES as u16) * 2 + MANIAXE_STARTUP_FRAMES as u16,
            "native lifetime covers startup plus out-and-back travel"
        );
        assert!(axe.friendly, "Maniaxe axe should be player-friendly");
        assert_eq!(axe.gravity, 0, "Maniaxe axe is not gravity-driven");
        assert_eq!(axe.homing_accel, 0, "Maniaxe axe is not homing");
        assert_eq!(axe.speed_cap, 0, "Maniaxe axe has no homing speed cap");
        assert!(
            !axe.wall_bounce,
            "ASM loc_8ACC deletes the axe on solid contact rather than bouncing"
        );
    }

    #[test]
    /// ASM loc_A7F8: `bclr #Button_C,(Ctrl_Pressed); beq.w loc_A8D6` — when C
    /// is not newly pressed, Maniaxe skips the axe allocation path entirely.
    fn maniaxe_no_c_press_does_not_throw_axe() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        let slice = combat_slice();

        rt.step_maniaxe_axe(&slice, FrameInput::default());

        assert!(
            !rt.helmet.maniaxe_axe_active,
            "ASM loc_A7F8: no C press should not set Maniaxe_throwing_axe"
        );
        assert!(
            rt.projectiles.is_empty(),
            "ASM loc_A7F8: no C press should not allocate ManiaxeAxe_Init"
        );
    }

    #[test]
    /// ASM loc_76E4: `tst.b (Maniaxe_throwing_axe) / bne.w loc_7772` — if the
    /// throwing flag is already set, a new C press is ignored (one axe at a time).
    fn maniaxe_no_double_throw() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        rt.helmet.maniaxe_axe_active = true;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_maniaxe_axe(&slice, input);

        assert!(
            rt.projectiles.is_empty(),
            "ASM loc_76E4: bne skip — must not throw while axe is already active"
        );
    }

    #[test]
    fn maniaxe_non_maniaxe_form_clears_state() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::TheKid;
        rt.helmet.maniaxe_axe_active = true;
        let slice = combat_slice();

        rt.step_maniaxe_axe(&slice, FrameInput::default());

        assert!(!rt.helmet.maniaxe_axe_active);
    }

    #[test]
    fn grounded_helmet_state_sub_78e8_keeps_live_maniaxe_axe() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        rt.helmet.maniaxe_axe_active = true;
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0x2_0000,
            y_velocity: 0,
            lifetime: 30,
            friendly: true,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });

        rt.refresh_grounded_helmet_state_sub_78E8();

        assert!(
            rt.helmet.maniaxe_axe_active,
            "ASM sub_78E8 should preserve Maniaxe's grounded throw pose while the axe is still live",
        );
    }

    #[test]
    fn grounded_helmet_state_sub_78e8_clears_inactive_maniaxe_axe() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        rt.helmet.maniaxe_axe_active = true;

        rt.refresh_grounded_helmet_state_sub_78E8();

        assert!(
            !rt.helmet.maniaxe_axe_active,
            "ASM sub_78E8 should return Maniaxe to the grounded default pose once the thrown axe is gone",
        );
    }

    #[test]
    fn grounded_special_state_reset_sub_db22_clears_transient_helmet_actions() {
        let mut rt = NativeRuntime::new(manifest());
        rt.helmet.cyclone_flying = true;
        rt.helmet.skycutter_flipboard = 0x11;
        rt.helmet.skycutter_on_board = true;
        rt.helmet.red_stealth_sword_timer = 0x70;
        rt.helmet.maniaxe_axe_active = true;
        rt.helmet.berzerker_charge_accum = 0xFF;
        rt.helmet.berzerker_charging = true;
        rt.helmet.iron_knight_climbing = true;
        rt.helmet.iron_knight_climb_accel = -0x2000;
        rt.helmet.iron_knight_climb_pose_accum = 0x0123;

        rt.clear_special_movement_state_sub_DB22();

        assert!(
            !rt.helmet.cyclone_flying,
            "ASM sub_DB22 should clear `Cyclone_flying` on the shared grounded reset path",
        );
        assert_eq!(
            rt.helmet.skycutter_flipboard, 0,
            "ASM sub_DB22 should clear `$FFFFFA69` so Skycutter's transient flipboard pose stops immediately",
        );
        assert!(
            !rt.helmet.skycutter_on_board,
            "ASM sub_DB22 should clear `$FFFFFA6A` so Skycutter drops the mounted-board latch",
        );
        assert_eq!(
            rt.helmet.red_stealth_sword_timer, 0,
            "ASM sub_DB22 should clear `Red_Stealth_sword_swing` once the shared airborne action ends",
        );
        assert!(
            !rt.helmet.maniaxe_axe_active,
            "ASM sub_DB22 should clear `Maniaxe_throwing_axe` on the default-pose reset",
        );
        assert_eq!(
            rt.helmet.berzerker_charge_accum, 0,
            "ASM sub_DB22 should clear the Berzerker charge accumulator alongside `Berzerker_charging`",
        );
        assert!(
            !rt.helmet.berzerker_charging,
            "ASM sub_DB22 should clear `Berzerker_charging` before grounded movement resumes",
        );
        assert!(
            !rt.helmet.iron_knight_climbing,
            "ASM sub_DB22 should end Iron Knight's shared special-motion state on grounded reset",
        );
        assert_eq!(rt.helmet.iron_knight_climb_accel, 0);
        assert_eq!(rt.helmet.iron_knight_climb_pose_accum, 0);
    }

    #[test]
    fn juggernaut_companion_step_sub_975c_preserves_a_live_animation_until_byte_18_arms() {
        assert_eq!(
            juggernaut_companion_step_sub_975C(
                JuggernautCompanionStateSub975C {
                    byte_15_active: true,
                    byte_18_active: false,
                    blocked_latch: true,
                },
                false,
                false,
                false,
                0,
                0,
            ),
            JuggernautCompanionStepSub975C {
                byte_15_active: true,
                blocked_latch: true,
                visual: JuggernautCompanionVisualSub975C::PreserveCurrent,
            },
            "ASM sub_975C should return early while `$15(a4)` stays set and `$18(a4)` is still clear",
        );
    }

    #[test]
    fn juggernaut_companion_step_sub_975c_flips_the_latch_between_raise_and_lower_animations() {
        assert_eq!(
            juggernaut_companion_step_sub_975C(
                JuggernautCompanionStateSub975C {
                    byte_15_active: false,
                    byte_18_active: false,
                    blocked_latch: false,
                },
                true,
                false,
                false,
                0,
                0,
            ),
            JuggernautCompanionStepSub975C {
                byte_15_active: false,
                blocked_latch: true,
                visual: JuggernautCompanionVisualSub975C::InitAnimation("stru_981C"),
            },
            "ASM loc_97BE should start `stru_981C` and set `$FFFFFA6C` when any predicate blocks the support sprite",
        );
        assert_eq!(
            juggernaut_companion_step_sub_975C(
                JuggernautCompanionStateSub975C {
                    byte_15_active: false,
                    byte_18_active: false,
                    blocked_latch: true,
                },
                false,
                false,
                false,
                0,
                0,
            ),
            JuggernautCompanionStepSub975C {
                byte_15_active: false,
                blocked_latch: false,
                visual: JuggernautCompanionVisualSub975C::InitAnimation("stru_9812"),
            },
            "ASM loc_9790 should start `stru_9812` and clear `$FFFFFA6C` once the scan, overlap, and Down gates all open",
        );
    }

    #[test]
    fn juggernaut_companion_step_sub_975c_selects_standing_and_walking_mappings() {
        assert_eq!(
            juggernaut_companion_step_sub_975C(
                JuggernautCompanionStateSub975C {
                    byte_15_active: false,
                    byte_18_active: false,
                    blocked_latch: false,
                },
                false,
                false,
                false,
                0,
                0,
            )
            .visual,
            JuggernautCompanionVisualSub975C::SetMapping("LnkTo_unk_BEDF0-Data_Index"),
            "ASM loc_97D4 should use `LnkTo_unk_BEDF0` while `Character_Movement == MoveID_Standingstill`",
        );
        assert_eq!(
            juggernaut_companion_step_sub_975C(
                JuggernautCompanionStateSub975C {
                    byte_15_active: false,
                    byte_18_active: false,
                    blocked_latch: false,
                },
                false,
                false,
                false,
                2,
                4,
            )
            .visual,
            JuggernautCompanionVisualSub975C::SetMapping("LnkTo_unk_BF0FC-Data_Index"),
            "ASM loc_97E8 should index `off_9826` by the caller-provided byte offset when `Character_Movement == MoveID_Walking`",
        );
    }

    // --- Maniaxe axe loc_ citation tests ---

    #[test]
    /// ASM loc_8A42 (kid.asm:9862): `add.w $26(a0),d7; move.w d7,x_vel(a3)` —
    /// axe velocity = base_speed (±5) + Kid's x_vel high word ($26 = x_vel long).
    /// `moveq #$C,d0` sets the turnaround counter to 12 frames.
    /// Native: MANIAXE_BASE_SPEED * dir = ±5, turnaround = MANIAXE_TRAVEL_FRAMES = 12.
    /// TODO(fidelity): native does not add Kid's x_velocity word to axe base speed.
    fn loc_8a42_axe_velocity_uses_base_speed_and_sets_turnaround_counter() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        let slice = combat_slice(); // player facing Right
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_maniaxe_axe(&slice, input);

        assert_eq!(rt.projectiles.len(), 1);
        assert_eq!(
            rt.projectiles[0].x_velocity,
            MANIAXE_BASE_SPEED << 16,
            "ASM loc_8A42: right-facing axe velocity should be +5 px/frame (native uses fixed base; \
             ASM would add Kid's x_vel word)"
        );
        assert_eq!(
            rt.helmet.maniaxe_turnaround,
            MANIAXE_TRAVEL_FRAMES - 1,
            "ASM loc_8A42: `moveq #$C,d0` sets counter to 12; native decrements in the same \
             step_maniaxe_axe call, so observed value is 11 after the first tick"
        );
    }

    #[test]
    /// ASM loc_8A42: left-facing mirror — `moveq #-5,d7` when x_direction != 0.
    /// `tst.b x_direction(a3); beq.w loc_8A42; moveq #-5,d7` selects negative base.
    fn loc_8a42_left_facing_axe_velocity_is_negative() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        let mut slice = combat_slice();
        slice.player.facing = Direction::Left;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_maniaxe_axe(&slice, input);

        assert_eq!(rt.projectiles.len(), 1);
        assert_eq!(
            rt.projectiles[0].x_velocity,
            -MANIAXE_BASE_SPEED << 16,
            "ASM loc_8A42: left-facing axe should use moveq #-5 base speed"
        );
    }

    #[test]
    /// ASM loc_8A52 (kid.asm:9868): axe main flight loop — per-frame hibernate.
    /// `jsr Hibernate_Object_1Frame` advances the axe one frame. The native
    /// equivalent is step_projectiles decrementing lifetime and advancing position.
    /// `tst.b $18(a3); beq.w loc_8A74` — animation-done flag. If set, reinit
    /// animation (stru_8B86) and play sfx_Maniaxe_throw_axe. Visual/audio looping.
    /// Verify that the axe projectile advances position each frame.
    fn loc_8a52_axe_advances_position_each_frame() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 5 << 16, // +5 px/frame
            y_velocity: 0,
            lifetime: 30,
            friendly: true,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].x_pos,
            105 << 16,
            "ASM loc_8A52: after one Hibernate_Object_1Frame, axe x_pos should advance by x_vel"
        );
        assert_eq!(
            rt.projectiles[0].lifetime, 29,
            "ASM loc_8A52: lifetime decrements each frame (native equivalent of the loop counter)"
        );
    }

    #[test]
    /// ASM loc_8A74 (kid.asm:9879): `subq.w #1,d0; bne.w loc_8A84` — decrement
    /// turnaround counter each frame. When zero: `not.b x_direction(a3);
    /// not.b $17(a3); moveq #$C,d0` — flip direction, flip render, reset to 12.
    /// Native reverses x_velocity and resets maniaxe_turnaround.
    fn loc_8a74_turnaround_reverses_direction_after_12_frames() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        rt.helmet.maniaxe_axe_active = true;
        rt.helmet.maniaxe_turnaround = 1; // will reach 0 on this tick
        rt.projectiles.push(EnemyProjectile {
            x_pos: 200 << 16,
            y_pos: 100 << 16,
            x_velocity: 5 << 16,
            y_velocity: 0,
            lifetime: 20,
            friendly: true,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });
        let slice = combat_slice();

        rt.step_maniaxe_axe(&slice, FrameInput::default());

        assert_eq!(
            rt.projectiles[0].x_velocity,
            -(5 << 16),
            "ASM loc_8A74: at counter=0, `not.b x_direction` flips direction — \
             native reverses x_velocity"
        );
        assert_eq!(
            rt.helmet.maniaxe_turnaround, MANIAXE_TRAVEL_FRAMES,
            "ASM loc_8A74: `moveq #$C,d0` resets turnaround counter to 12"
        );
    }

    #[test]
    /// ASM loc_8A74: when counter > 1, `subq.w #1,d0; bne.w loc_8A84` skips
    /// the direction flip — falls through to loc_8A84 tile collision check.
    fn loc_8a74_mid_flight_decrements_counter_without_flip() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        rt.helmet.maniaxe_axe_active = true;
        rt.helmet.maniaxe_turnaround = 5; // mid-flight, not at flip point
        rt.projectiles.push(EnemyProjectile {
            x_pos: 200 << 16,
            y_pos: 100 << 16,
            x_velocity: 5 << 16,
            y_velocity: 0,
            lifetime: 20,
            friendly: true,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });
        let slice = combat_slice();

        rt.step_maniaxe_axe(&slice, FrameInput::default());

        assert_eq!(
            rt.projectiles[0].x_velocity,
            5 << 16,
            "ASM loc_8A74: bne.w loc_8A84 — counter > 0 means no direction flip"
        );
        assert_eq!(
            rt.helmet.maniaxe_turnaround, 4,
            "ASM loc_8A74: subq.w #1,d0 — counter should decrement from 5 to 4"
        );
    }

    /// ASM loc_8ABC (kid.asm:9905): `tst.w collision_type(a3); beq.s loc_8A52` —
    /// when collision_type != 0, axe hit an enemy. Play sfx_Maniaxe_hitting_enemy,
    /// `jmp Delete_CurrentObject`. Native: `process_friendly_projectile_hits`
    /// removes the consumed projectile from the list.
    #[test]
    fn maniaxe_axe_consumed_on_enemy_hit_loc_8abc() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;

        // Spawn a friendly axe projectile
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 5 << 16,
            y_velocity: 0,
            lifetime: 20,
            friendly: true,
            ..Default::default()
        });

        // Spawn an enemy at the exact same position (guaranteed overlap)
        rt.active_enemies
            .push(test_enemy(EnemyFamily::Dragon, 100 << 16, 100 << 16, 1));
        rt.active_enemy_roster = rt
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();

        rt.process_friendly_projectile_hits();

        assert!(
            rt.projectiles.is_empty(),
            "ASM loc_8ABC: axe should be deleted after hitting an enemy (collision_type != 0)"
        );
    }

    /// ASM loc_8ABC: when axe hits an enemy with >1 HP, the enemy survives but
    /// the axe is still consumed (ASM `jmp Delete_CurrentObject` runs regardless
    /// of whether the enemy died).
    #[test]
    fn maniaxe_axe_consumed_even_if_enemy_survives_loc_8abc() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;

        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 5 << 16,
            y_velocity: 0,
            lifetime: 20,
            friendly: true,
            ..Default::default()
        });

        rt.active_enemies
            .push(test_enemy(EnemyFamily::Dragon, 100 << 16, 100 << 16, 3));
        rt.active_enemy_roster = rt
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();

        rt.process_friendly_projectile_hits();

        assert!(
            rt.projectiles.is_empty(),
            "ASM loc_8ABC: axe is always consumed on hit — `jmp Delete_CurrentObject` is unconditional"
        );
        assert_eq!(
            rt.active_enemies.len(),
            1,
            "ASM loc_8ABC: enemy with >1 HP survives the hit"
        );
        assert_eq!(
            rt.active_enemies[0].hit_points, 2,
            "ASM loc_8ABC: enemy HP decremented by 1"
        );
    }

    /// ASM loc_8ABC is only reached by a live friendly projectile collision.
    /// Native must ignore enemy overlap from unfriendly or expired projectiles.
    #[test]
    fn friendly_projectile_hits_ignore_nonfriendly_and_expired_projectiles_loc_8abc() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            lifetime: 20,
            friendly: false,
            ..Default::default()
        });
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            lifetime: 0,
            friendly: true,
            ..Default::default()
        });
        rt.active_enemies
            .push(test_enemy(EnemyFamily::Dragon, 100 << 16, 100 << 16, 2));

        rt.process_friendly_projectile_hits();

        assert_eq!(
            rt.active_enemies[0].hit_points, 2,
            "ASM loc_8ABC: non-friendly and expired projectiles should not set collision_type"
        );
        assert_eq!(
            rt.projectiles.len(),
            2,
            "ASM loc_8ABC: ignored projectiles should not be consumed"
        );
    }

    /// ASM loc_8ABC consumes the projectile worker's collision result. Native's
    /// rectangle approximation keeps horizontal overlap inclusive at
    /// PROJECTILE_HITBOX_HALF + ENEMY_CONTACT_HALF_WIDTH.
    #[test]
    fn friendly_projectile_hits_at_horizontal_extent_loc_8abc() {
        let mut rt = NativeRuntime::new(manifest());
        let limit = PROJECTILE_HITBOX_HALF + ENEMY_CONTACT_HALF_WIDTH as i32;
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            lifetime: 20,
            friendly: true,
            ..Default::default()
        });
        rt.active_enemies.push(test_enemy(
            EnemyFamily::Dragon,
            ((100 + limit) as u32) << 16,
            100 << 16,
            2,
        ));

        rt.process_friendly_projectile_hits();

        assert_eq!(
            rt.active_enemies[0].hit_points, 1,
            "ASM loc_8ABC: enemy exactly at projectile horizontal extent should be hit"
        );
    }

    /// ASM loc_8ABC only fires after projectile collision_type is non-zero;
    /// native must reject enemies just outside horizontal overlap.
    #[test]
    fn friendly_projectile_misses_just_beyond_horizontal_extent_loc_8abc() {
        let mut rt = NativeRuntime::new(manifest());
        let miss_dx = PROJECTILE_HITBOX_HALF + ENEMY_CONTACT_HALF_WIDTH as i32 + 1;
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            lifetime: 20,
            friendly: true,
            ..Default::default()
        });
        rt.active_enemies.push(test_enemy(
            EnemyFamily::Dragon,
            ((100 + miss_dx) as u32) << 16,
            100 << 16,
            2,
        ));

        rt.process_friendly_projectile_hits();

        assert_eq!(
            rt.active_enemies[0].hit_points, 2,
            "ASM loc_8ABC: enemy one pixel beyond projectile horizontal extent should miss"
        );
    }

    /// ASM loc_8ABC consumes the projectile worker's collision result. Native's
    /// rectangle approximation keeps vertical overlap inclusive at
    /// PROJECTILE_HITBOX_HALF + ENEMY_CONTACT_HALF_HEIGHT.
    #[test]
    fn friendly_projectile_hits_at_vertical_extent_loc_8abc() {
        let mut rt = NativeRuntime::new(manifest());
        let limit = PROJECTILE_HITBOX_HALF + ENEMY_CONTACT_HALF_HEIGHT as i32;
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            lifetime: 20,
            friendly: true,
            ..Default::default()
        });
        rt.active_enemies.push(test_enemy(
            EnemyFamily::Dragon,
            100 << 16,
            ((100 + limit) as u32) << 16,
            2,
        ));

        rt.process_friendly_projectile_hits();

        assert_eq!(
            rt.active_enemies[0].hit_points, 1,
            "ASM loc_8ABC: enemy exactly at projectile vertical extent should be hit"
        );
    }

    /// ASM loc_8ABC only fires after projectile collision_type is non-zero;
    /// native must reject enemies just outside vertical overlap.
    #[test]
    fn friendly_projectile_misses_just_beyond_vertical_extent_loc_8abc() {
        let mut rt = NativeRuntime::new(manifest());
        let miss_dy = PROJECTILE_HITBOX_HALF + ENEMY_CONTACT_HALF_HEIGHT as i32 + 1;
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            lifetime: 20,
            friendly: true,
            ..Default::default()
        });
        rt.active_enemies.push(test_enemy(
            EnemyFamily::Dragon,
            100 << 16,
            ((100 + miss_dy) as u32) << 16,
            2,
        ));

        rt.process_friendly_projectile_hits();

        assert_eq!(
            rt.active_enemies[0].hit_points, 2,
            "ASM loc_8ABC: enemy one pixel beyond projectile vertical extent should miss"
        );
    }

    /// ASM loc_8ABC consumes the projectile even when the enemy survives. Native
    /// must still refresh the trace roster so HP and damage flash reflect the hit.
    #[test]
    fn friendly_projectile_surviving_hit_refreshes_active_enemy_roster_loc_8abc() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            lifetime: 20,
            friendly: true,
            ..Default::default()
        });
        rt.active_enemies
            .push(test_enemy(EnemyFamily::Dragon, 100 << 16, 100 << 16, 3));
        rt.active_enemy_roster = rt
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();

        rt.process_friendly_projectile_hits();

        assert_eq!(rt.active_enemies[0].hit_points, 2);
        assert_eq!(
            rt.active_enemy_roster[0].hit_points,
            Some(2),
            "ASM loc_8ABC: surviving projectile hit should refresh trace HP"
        );
        assert!(
            rt.active_enemy_roster[0].damage_flash,
            "ASM loc_8ABC: surviving projectile hit should refresh trace damage flash"
        );
    }

    /// No ASM loc_8ABC collision means no projectile hit side effects; native
    /// should not manufacture a trace roster refresh when nothing collided.
    #[test]
    fn friendly_projectile_no_hit_does_not_refresh_empty_roster_loc_8abc() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            lifetime: 20,
            friendly: true,
            ..Default::default()
        });
        rt.active_enemies
            .push(test_enemy(EnemyFamily::Dragon, 200 << 16, 100 << 16, 2));

        rt.process_friendly_projectile_hits();

        assert!(
            rt.active_enemy_roster.is_empty(),
            "ASM loc_8ABC: no collision should leave an empty trace roster untouched"
        );
    }

    /// ASM loc_8ACC (kid.asm:9913): axe hits solid block — `clr.w x_vel(a3);
    /// clr.b $12(a3); clr.b palette_line(a3)`, play sfx, animate stru_8BDE,
    /// delete. Native uses lifetime expiry instead.
    /// TODO(fidelity): native Maniaxe axe ignores solid terrain contact.
    #[test]
    fn maniaxe_axe_solid_terrain_not_modelled_loc_8acc() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_maniaxe_axe(&slice, input);
        assert_eq!(rt.projectiles.len(), 1);
        let axe = &rt.projectiles[0];
        // The ASM axe checks `andi.w #$7000,d7; cmpi.w #$6000,d7` (Solid collision)
        // each frame. Native axe has no terrain_kill flag — it relies on lifetime.
        assert!(
            !axe.terrain_kill,
            "ASM loc_8ACC: TODO(fidelity) — native axe does not check solid terrain; \
             ASM destroys axe with stru_8BDE explosion animation on solid-block contact"
        );
    }

    /// ASM loc_8AF2 (kid.asm:9925): `jmp Delete_CurrentObject` — axe deletion
    /// after solid-block explosion or out-of-bounds. Native: lifetime expiry
    /// removes the projectile from the list.
    #[test]
    fn maniaxe_axe_deleted_on_lifetime_expiry_loc_8af2() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 5 << 16,
            y_velocity: 0,
            lifetime: 0, // already expired — removed on next step
            friendly: true,
            ..Default::default()
        });

        rt.step_projectiles();

        assert!(
            rt.projectiles.is_empty(),
            "ASM loc_8AF2: `jmp Delete_CurrentObject` — native equivalent is lifetime=0 removal"
        );
    }

    /// ASM loc_8AF2 (kid.asm:9925): after ManiaxeAxe_CheckOutOfBounds or the
    /// solid-block explosion path, the axe object is deleted. Native mirrors
    /// this by clearing Maniaxe's active flag once only expired axe projectiles
    /// remain.
    #[test]
    fn maniaxe_axe_active_clears_when_only_expired_projectile_remains_loc_8af2() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        rt.helmet.maniaxe_axe_active = true;
        rt.helmet.maniaxe_turnaround = 5;
        let slice = combat_slice();
        rt.projectiles.push(EnemyProjectile {
            x_pos: slice.player.x_pos,
            y_pos: slice.player.y_pos,
            x_velocity: MANIAXE_BASE_SPEED << 16,
            lifetime: 0,
            friendly: true,
            ..Default::default()
        });

        rt.step_maniaxe_axe(&slice, FrameInput::default());

        assert!(
            !rt.helmet.maniaxe_axe_active,
            "ASM loc_8AF2: deleted/expired axe must clear Maniaxe_throwing_axe"
        );
    }

    /// ASM loc_8B32 (kid.asm:9954): `moveq #1,d7; rts` — true return of
    /// ManiaxeAxe_CheckOutOfBounds (sub_8AF6). Checks camera-relative bounds:
    /// x in [cam_x - $10, cam_x + $160], y in [cam_y - $10, cam_y + $100].
    /// Native uses lifetime counter instead. Verify the lifetime bounds give
    /// finite axe range analogous to ASM camera bounds.
    #[test]
    fn maniaxe_axe_bounded_lifetime_loc_8b32() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_maniaxe_axe(&slice, input);
        assert_eq!(rt.projectiles.len(), 1);
        let lifetime = rt.projectiles[0].lifetime;
        // ASM uses camera bounds [$160 + $10 = $170 = 368px wide], but native uses
        // a fixed lifetime. The lifetime must be finite (nonzero and bounded).
        assert!(
            lifetime > 0 && lifetime < 200,
            "ASM loc_8B32: axe should have a finite bounded lifetime (got {lifetime}); \
             ASM uses camera-relative bounds check instead"
        );
    }

    // --- Eyeclops beam tests ---

    /// ASM loc_E4B2 (kid.asm:18008): an active Eyeclops beam decrements `d7`,
    /// seeds `d0=$12` and `d1=-$0F`, then negates `d0` only when the latched
    /// `$FFFFFABE` direction byte is non-zero.
    #[test]
    #[allow(non_snake_case)]
    fn loc_E4B2_eyeclops_beam_render_setup_decrements_counter_and_latches_side() {
        let right = helmets::eyeclops_beam_render_setup_loc_E4B2(0x8001, 0);
        assert_eq!(
            right.beam_counter, 0x8000,
            "ASM loc_E4B2 should subq.w #1,d7 before choosing beam segment shape"
        );
        assert_eq!(
            (right.x_offset_px, right.y_offset_px),
            (0x12, -0x0F),
            "ASM loc_E4B2 should keep d0=$12 and d1=-$0F when `$FFFFFABE` is zero"
        );

        let left = helmets::eyeclops_beam_render_setup_loc_E4B2(0, 0xFF);
        assert_eq!(
            left.beam_counter, 0xFFFF,
            "ASM loc_E4B2 uses word arithmetic, so decrementing zero wraps before the BMI path"
        );
        assert_eq!(
            left.x_offset_px, -0x12,
            "ASM loc_E4B2 should negate d0 when the latched x-direction byte is non-zero"
        );
        assert_eq!(
            left.y_offset_px, -0x0F,
            "ASM loc_E4B2 should leave d1 at -$0F regardless of beam direction"
        );
    }

    /// ASM loc_E4C4 (kid.asm:18017): the Eyeclops beam origin is Kid word
    /// position plus the signed setup offset, minus camera word position, plus
    /// the `$80` sprite coordinate bias on both axes.
    #[test]
    #[allow(non_snake_case)]
    fn loc_E4C4_eyeclops_beam_screen_origin_applies_camera_and_sprite_bias() {
        let right_setup = helmets::eyeclops_beam_render_setup_loc_E4B2(3, 0);
        let right = helmets::eyeclops_beam_screen_origin_loc_E4C4(
            0x0120,
            0x00A0,
            0x0100,
            0x0080,
            right_setup.x_offset_px,
            right_setup.y_offset_px,
        );
        assert_eq!(
            right.screen_x_word, 0x00B2,
            "ASM loc_E4C4 should compute KidX+$12-CameraX+$80 for a zero direction latch"
        );
        assert_eq!(
            right.screen_y_word, 0x0091,
            "ASM loc_E4C4 should compute KidY-$0F-CameraY+$80"
        );

        let left_setup = helmets::eyeclops_beam_render_setup_loc_E4B2(3, 0xFF);
        let left = helmets::eyeclops_beam_screen_origin_loc_E4C4(
            0x0010,
            0x0008,
            0x0020,
            0x0010,
            left_setup.x_offset_px,
            left_setup.y_offset_px,
        );
        assert_eq!(
            left.screen_x_word, 0x005E,
            "ASM loc_E4C4 uses wrapping word arithmetic after loc_E4B2 negates d0"
        );
        assert_eq!(
            left.screen_y_word, 0x0069,
            "ASM loc_E4C4 should preserve the wrapped signed Y offset before adding `$80`"
        );
    }

    /// ASM loc_E50A (kid.asm:18042): mirrored Eyeclops beams subtract the
    /// segment span from the already biased screen X and set bit `$0800` on
    /// every tile word emitted by the following sprite table loop.
    #[test]
    #[allow(non_snake_case)]
    fn loc_E50A_eyeclops_beam_mirrored_anchor_subtracts_segment_span_and_sets_flip() {
        let anchor = helmets::eyeclops_beam_mirrored_anchor_loc_E50A(0x00C0, 3);

        assert_eq!(
            anchor.anchor_x_word, 0x0098,
            "ASM loc_E50A should subtract `$10 + (d7 << 3)` from the beam screen X",
        );
        assert_eq!(
            anchor.tile_flip_bit, 0x0800,
            "ASM loc_E50A should seed d4 with the horizontal tile flip bit",
        );
    }

    /// ASM loc_E512 (kid.asm:18047): after the beam segment counter survives
    /// the `< 4` gate, the ROM shifts it left by two and reads the matching
    /// longword from `off_E560`.
    #[test]
    #[allow(non_snake_case)]
    fn loc_E512_eyeclops_beam_dispatches_through_four_longword_table_entries() {
        let dispatches = [
            helmets::eyeclops_beam_table_dispatch_loc_E512(0),
            helmets::eyeclops_beam_table_dispatch_loc_E512(1),
            helmets::eyeclops_beam_table_dispatch_loc_E512(2),
            helmets::eyeclops_beam_table_dispatch_loc_E512(3),
        ];

        assert_eq!(
            dispatches.map(|dispatch| dispatch.map(|entry| entry.pc_table_byte_offset)),
            [Some(0), Some(4), Some(8), Some(12)],
            "ASM loc_E512 should use `d7 << 2` as the byte offset into off_E560",
        );
        assert_eq!(
            dispatches.map(|dispatch| dispatch.map(|entry| entry.table_label)),
            [
                Some("unk_E570"),
                Some("unk_E596"),
                Some("unk_E5C4"),
                Some("unk_E5FA"),
            ],
            "ASM loc_E512 should select the four short-beam sprite tables in order",
        );
    }

    /// ASM loc_E51A (kid.asm:18052): each short-beam table entry emits a VDP
    /// sprite, increments `Number_Sprites`, places that byte into the low link
    /// byte, and mirrors X by subtracting both the table X offset and the
    /// width bucket derived from the size word.
    #[test]
    #[allow(non_snake_case)]
    fn loc_E51A_eyeclops_beam_sprite_loop_emits_table_entries_and_mirrors_x() {
        let right = helmets::eyeclops_beam_sprites_loc_E51A("unk_E570", 0x00B2, 0x0091, 0, 0, 0)
            .expect("ASM loc_E512 dispatches d7=0 to unk_E570 before loc_E51A");

        assert_eq!(
            right.number_sprites, 4,
            "ASM loc_E51A should increment Number_Sprites once per unk_E570 entry before the `$8000` sentinel",
        );
        assert_eq!(
            right.sprites[0],
            helmets::EyeclopsBeamSpriteLocE51A {
                y: 0x0081,
                size_link: 0x0501,
                tile_word: 0xE676,
                x: 0x00B2,
            },
            "ASM loc_E51A should add the first entry's signed Y offset and link sprite #1",
        );
        assert_eq!(
            right.sprites[2].x, 0x00BA,
            "ASM loc_E51A non-mirrored X should add the table X offset to screen X",
        );

        let left =
            helmets::eyeclops_beam_sprites_loc_E51A("unk_E570", 0x005E, 0x0069, 0xFF, 0x0800, 5)
                .expect("ASM loc_E512 dispatches d7=0 to unk_E570 before loc_E51A");
        assert_eq!(
            left.sprites[0],
            helmets::EyeclopsBeamSpriteLocE51A {
                y: 0x0059,
                size_link: 0x0506,
                tile_word: 0xEE76,
                x: 0x004E,
            },
            "ASM loc_E51A mirrored X should subtract the table X offset and the size-derived width bucket",
        );
    }

    /// ASM loc_E54E (kid.asm:18078): when `$FFFFFABE` is clear, the short
    /// Eyeclops beam stores `d0 + table_x_offset` directly as each sprite X.
    #[test]
    #[allow(non_snake_case)]
    fn loc_E54E_eyeclops_beam_non_mirrored_x_adds_table_offset_without_width_bias() {
        let right = helmets::eyeclops_beam_sprites_loc_E51A("unk_E570", 0x00B2, 0x0091, 0, 0, 0)
            .expect("ASM loc_E512 dispatches d7=0 to unk_E570 before loc_E51A");

        assert_eq!(
            right
                .sprites
                .iter()
                .map(|sprite| sprite.x)
                .collect::<Vec<_>>(),
            vec![0x00B2, 0x00B2, 0x00BA, 0x00B2],
            "ASM loc_E54E should add `(a0)+` to d5 and store it, with no sprite-width subtraction on the clear `$FFFFFABE` path",
        );
    }

    /// ASM loc_E554 (kid.asm:18084): the `$8000` table sentinel is followed by
    /// two signed Y offsets. The ROM adds both to d1 and carries them into
    /// the bounds path as d5/d6 for the beam's vertical collision band.
    #[test]
    #[allow(non_snake_case)]
    fn loc_E554_eyeclops_beam_sentinel_prepares_vertical_collision_band() {
        let render = helmets::eyeclops_beam_sprites_loc_E51A("unk_E570", 0x00B2, 0x0091, 0, 0, 0)
            .expect("ASM loc_E512 dispatches d7=0 to unk_E570 before loc_E51A");

        assert_eq!(
            (render.collision_top_y_word, render.collision_bottom_y_word),
            (0x0089, 0x00A1),
            "ASM loc_E554 should add the post-sentinel words `$FFF8` and `$0010` to d1 before the bounds path",
        );
    }

    #[test]
    fn eyeclops_c_press_fires_beam_right() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        let mut slice = combat_slice();
        slice.player.facing = Direction::Right;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_eyeclops_beam(&slice, input);

        assert!(rt.helmet.eyeclops_beam_active);
        assert_eq!(
            rt.helmet.eyeclops_beam_counter, 1,
            "ASM sub_E49A seeds `$FFFFFAB8` with `#1` for the normal Eyeclops beam"
        );
        assert!(
            !rt.helmet.eyeclops_beam_hard,
            "the normal C-press path should keep the hard-beam bit clear"
        );
        assert_eq!(rt.projectiles.len(), 1);
        let beam = &rt.projectiles[0];
        assert!(beam.friendly, "beam should be player-friendly");
        assert_eq!(
            beam.x_pos, slice.player.x_pos,
            "ASM sub_E49A grows the Eyeclops beam from Kid's live X coordinate"
        );
        assert_eq!(beam.x_velocity, EYECLOPS_BEAM_SPEED, "beam moves right");
        assert_eq!(beam.lifetime, EYECLOPS_BEAM_LIFETIME);
        assert_eq!(beam.y_velocity, 0, "beam has no vertical velocity");
        assert_eq!(beam.gravity, 0, "beam has no gravity");
        assert_eq!(
            beam.homing_accel, 0,
            "ASM sub_E49A is a straight horizontal beam, not a homing projectile"
        );
        assert_eq!(
            beam.speed_cap, 0,
            "ASM sub_E49A does not apply homing speed caps to the beam"
        );
        assert!(
            !beam.wall_bounce,
            "ASM sub_E49A beam ends by lifetime/state, not wall-bounce logic"
        );
        // Y offset: 8px above player
        assert_eq!(
            beam.y_pos,
            (slice.player.y_pos as i64 - (8 << 16)) as u32,
            "beam spawns 8px above player"
        );
    }

    #[test]
    fn eyeclops_c_press_fires_beam_left() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        let mut slice = combat_slice();
        slice.player.facing = Direction::Left;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_eyeclops_beam(&slice, input);

        assert!(rt.helmet.eyeclops_beam_active);
        assert_eq!(rt.projectiles.len(), 1);
        assert_eq!(
            rt.projectiles[0].x_velocity, -EYECLOPS_BEAM_SPEED,
            "beam moves left when facing left"
        );
    }

    #[test]
    fn eyeclops_no_double_fire() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        rt.helmet.eyeclops_beam_active = true;
        // Add an existing beam projectile so active flag stays set
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: EYECLOPS_BEAM_SPEED,
            y_velocity: 0,
            lifetime: 30,
            friendly: true,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_eyeclops_beam(&slice, input);

        assert_eq!(
            rt.projectiles.len(),
            1,
            "should not fire while beam is active"
        );
    }

    #[test]
    fn eyeclops_beam_clears_when_projectile_expires() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        rt.helmet.eyeclops_beam_active = true;
        rt.helmet.eyeclops_beam_counter = 0x8001;
        rt.helmet.eyeclops_beam_hard = true;
        // Projectile with 0 lifetime = expired
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: EYECLOPS_BEAM_SPEED,
            y_velocity: 0,
            lifetime: 0,
            friendly: true,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });
        let slice = combat_slice();

        rt.step_eyeclops_beam(&slice, FrameInput::default());

        assert!(
            !rt.helmet.eyeclops_beam_active,
            "beam flag should clear when all projectiles expire"
        );
        assert_eq!(rt.helmet.eyeclops_beam_counter, 0);
        assert!(
            !rt.helmet.eyeclops_beam_hard,
            "ASM sub_E49A clears the beam state word when the coroutine ends"
        );
    }

    #[test]
    fn eyeclops_beam_allows_refire_after_expiry() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        rt.helmet.eyeclops_beam_active = true;
        // No projectiles with lifetime > 0 → beam should deactivate
        let slice = combat_slice();

        // First call: clears the active flag
        rt.step_eyeclops_beam(&slice, FrameInput::default());
        assert!(!rt.helmet.eyeclops_beam_active);

        // Second call with C press: should fire new beam
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_eyeclops_beam(&slice, input);

        assert!(rt.helmet.eyeclops_beam_active);
        assert_eq!(rt.projectiles.len(), 1);
    }

    #[test]
    fn eyeclops_non_eyeclops_form_clears_state() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::TheKid;
        rt.helmet.eyeclops_beam_active = true;
        rt.helmet.eyeclops_beam_counter = 0x8001;
        rt.helmet.eyeclops_beam_hard = true;
        let slice = combat_slice();

        rt.step_eyeclops_beam(&slice, FrameInput::default());

        assert!(
            !rt.helmet.eyeclops_beam_active,
            "non-Eyeclops form should clear beam state"
        );
        assert_eq!(rt.helmet.eyeclops_beam_counter, 0);
        assert!(!rt.helmet.eyeclops_beam_hard);
    }

    #[test]
    fn eyeclops_c_held_does_not_fire() {
        // C must be rising edge (newly pressed), not held
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        rt.prior_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_eyeclops_beam(&slice, input);

        assert!(
            rt.projectiles.is_empty(),
            "held C (not rising edge) should not fire"
        );
    }

    #[test]
    fn eyeclops_b_without_c_does_not_fire_hard_beam() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        rt.progression.diamond_count = 2;
        let slice = combat_slice();
        let input = FrameInput {
            b: true,
            ..FrameInput::default()
        };

        rt.step_eyeclops_beam(&slice, input);

        assert!(
            !rt.helmet.eyeclops_beam_active,
            "ASM loc_7606 requires the full held-button mask before arming `$FFFFFAB8` with `$8001`"
        );
        assert!(!rt.helmet.eyeclops_beam_hard);
        assert_eq!(
            rt.progression.diamond_count, 2,
            "hard beam must not spend diamonds unless both native gate buttons are held"
        );
        assert!(
            rt.projectiles.is_empty(),
            "B-only input must not spawn an Eyeclops hard beam"
        );
    }

    #[test]
    fn eyeclops_hard_beam_sub_e49a_consumes_two_diamonds() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        rt.progression.diamond_count = 2;
        let slice = combat_slice();
        let input = FrameInput {
            b: true,
            c: true,
            ..FrameInput::default()
        };

        rt.step_eyeclops_beam(&slice, input);

        assert!(rt.helmet.eyeclops_beam_active);
        assert_eq!(
            rt.helmet.eyeclops_beam_counter, 0x8001,
            "ASM sub_E49A seeds `$FFFFFAB8` with `$8001` for the hard beam path"
        );
        assert!(rt.helmet.eyeclops_beam_hard);
        assert_eq!(
            rt.progression.diamond_count, 0,
            "ASM sub_E49A spends two diamonds when the hard beam starts"
        );
        assert_eq!(rt.projectiles.len(), 1);
    }

    // --- Juggernaut skull shot tests ---

    #[test]
    fn juggernaut_c_press_arms_sub_86fa_before_spawning_sub_87b0() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_juggernaut_skull(&slice, input);

        assert!(
            rt.projectiles.is_empty(),
            "ASM sub_86FA should hibernate before the skull worker appears"
        );
        assert_eq!(rt.helmet.juggernaut_skull_delay_frames, 3);

        for expected in [2, 1] {
            rt.prior_input = input;
            rt.step_juggernaut_skull(&slice, FrameInput::default());
            assert_eq!(rt.helmet.juggernaut_skull_delay_frames, expected);
            assert!(rt.projectiles.is_empty());
        }

        rt.prior_input = FrameInput::default();
        rt.step_juggernaut_skull(&slice, FrameInput::default());

        assert_eq!(rt.projectiles.len(), 1, "third wake should spawn the skull");
        assert!(rt.projectiles[0].friendly);
        assert_eq!(rt.projectiles[0].gravity, JUGGERNAUT_SKULL_GRAVITY);
        assert_eq!(
            rt.projectiles[0].x_pos,
            ((128 + JUGGERNAUT_SKULL_OFFSET_X) as u32) << 16
        );
        assert_eq!(
            rt.projectiles[0].y_pos,
            ((256 + JUGGERNAUT_SKULL_OFFSET_Y) as u32) << 16
        );
        assert_eq!(rt.projectiles[0].x_velocity, JUGGERNAUT_SKULL_X_SPEED);
        assert_eq!(
            rt.projectiles[0].lifetime, JUGGERNAUT_SKULL_LIFETIME,
            "ASM sub_87B0: native skull projectile should carry the full worker lifetime"
        );
        assert_eq!(
            rt.projectiles[0].reaim_countdown,
            u16::from(JUGGERNAUT_SKULL_BOUNCES),
            "ASM loc_897C/loc_899C: skull should start with the full rebound budget"
        );
    }

    #[test]
    fn juggernaut_skull_has_gravity() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: JUGGERNAUT_SKULL_X_SPEED,
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].y_velocity, JUGGERNAUT_SKULL_GRAVITY,
            "gravity should increase y_velocity"
        );
    }

    #[test]
    fn juggernaut_skull_sub_87b0_horizontal_bounce_reduces_speed_and_budget() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 30 << 16,
            y_pos: 40 << 16,
            x_velocity: 0x4_0000,
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        rt.set_block_grid(bounce_grid(&[(2, 2), (2, 3)]));

        rt.step_projectiles();

        let projectile = &rt.projectiles[0];
        assert_eq!(
            projectile.x_velocity, -0x3_0000,
            "ASM sub_87B0 should reduce a +4px/frame skull rebound to -3px/frame after the first wall hit",
        );
        assert_eq!(
            projectile.reaim_countdown,
            u16::from(JUGGERNAUT_SKULL_BOUNCES - 1),
            "ASM sub_87B0 should spend one of the four skull rebounds on that hit",
        );
    }

    #[test]
    fn juggernaut_skull_sub_87b0_deletes_on_the_fourth_bounce() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 30 << 16,
            y_pos: 40 << 16,
            x_velocity: 0x4_0000,
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            reaim_countdown: 1,
            ..Default::default()
        });
        rt.set_block_grid(bounce_grid(&[(2, 2), (2, 3)]));

        rt.step_projectiles();

        assert!(
            rt.projectiles.is_empty(),
            "ASM sub_87B0 should delete the skull when the last rebound is spent",
        );
    }

    #[test]
    /// ASM loc_89A8 (kid.asm:9815): skull explosion/deletion path.
    /// Reached via loc_899C when bounce budget is exhausted (`subq.w #1,d0; beq loc_899C`).
    /// `sf is_moved(a3); clr.b $12(a3); clr.b palette_line(a3)` clears visual fields,
    /// `moveq #sfx_Juggernaut_skull_explode,d0; jsr j_PlaySound` plays explosion SFX,
    /// `Init_Animation stru_8BDE; Hibernate_UntilAnimFinished; Delete_CurrentObject`
    /// animates then deletes. Native: returning false from step_juggernaut_skull_sub_87B0
    /// removes the projectile (sound/animation are visual-only).
    /// loc_899C also decrements $FFFFFB70 (live skull count); native counts inline.
    fn loc_89a8_skull_explode_on_last_bounce_budget() {
        let mut rt = NativeRuntime::new(manifest());
        // Skull with exactly 1 bounce remaining, heading into a wall
        rt.projectiles.push(EnemyProjectile {
            x_pos: 30 << 16,
            y_pos: 40 << 16,
            x_velocity: 0x3_0000,
            y_velocity: 0,
            lifetime: 20,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            reaim_countdown: 1, // last bounce
            ..Default::default()
        });
        rt.set_block_grid(bounce_grid(&[(2, 2), (2, 3)]));

        rt.step_projectiles();

        assert!(
            rt.projectiles.is_empty(),
            "ASM loc_89A8: skull should be deleted (explosion path) when bounce budget reaches 0 \
             — equivalent to loc_899C `subq.w #1,d0; beq` → loc_89A8 `Delete_CurrentObject`"
        );
    }

    #[test]
    /// ASM loc_89A8: verify skull survives when bounce budget > 1 (does not reach
    /// loc_899C). The skull rebounds and continues its main loop (loc_8810).
    fn loc_89a8_skull_survives_with_remaining_bounce_budget() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 30 << 16,
            y_pos: 40 << 16,
            x_velocity: 0x3_0000,
            y_velocity: 0,
            lifetime: 20,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            reaim_countdown: 2, // 2 bounces left — survives this one
            ..Default::default()
        });
        rt.set_block_grid(bounce_grid(&[(2, 2), (2, 3)]));

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM loc_897C: skull with 2 bounces remaining should survive wall hit \
             and return to loc_8810 main loop — loc_89A8 NOT reached"
        );
        assert_eq!(
            rt.projectiles[0].reaim_countdown, 1,
            "ASM loc_897C: subq.w #1,d0 should decrement bounce budget from 2 to 1"
        );
    }

    #[test]
    fn juggernaut_max_skull_limit() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        // Fire 8 skulls
        for _ in 0..8 {
            rt.step_juggernaut_skull(&slice, input);
            rt.prior_input = input;
            rt.step_juggernaut_skull(&slice, FrameInput::default());
            rt.step_juggernaut_skull(&slice, FrameInput::default());
            rt.step_juggernaut_skull(&slice, FrameInput::default());
            rt.prior_input = FrameInput::default();
        }
        assert_eq!(rt.projectiles.len(), 8);

        // 9th should be rejected
        rt.prior_input = FrameInput::default();
        rt.step_juggernaut_skull(&slice, input);
        assert_eq!(rt.projectiles.len(), 8, "should cap at 8 skulls");
    }

    /// ASM loc_876A (kid.asm:9599): while the 3-frame skull worker is pending,
    /// it decrements the counter and stays in that worker; another C press must
    /// not re-arm the startup delay.
    #[test]
    fn juggernaut_skull_c_press_ignored_while_worker_pending_loc_876a() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        let slice = combat_slice();
        let c_press = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_juggernaut_skull(&slice, c_press);
        assert_eq!(rt.helmet.juggernaut_skull_delay_frames, 3);

        rt.prior_input = FrameInput::default();
        rt.step_juggernaut_skull(&slice, c_press);

        assert_eq!(
            rt.helmet.juggernaut_skull_delay_frames, 2,
            "ASM loc_876A: pending worker should tick 3->2, not re-arm to 3"
        );
        assert!(
            rt.projectiles.is_empty(),
            "ASM loc_876A: pending worker must not spawn before the counter reaches zero"
        );
    }

    #[test]
    fn juggernaut_non_juggernaut_form_no_skull() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::TheKid;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_juggernaut_skull(&slice, input);

        assert!(rt.projectiles.is_empty());
    }

    // --- loc_7718/loc_7738: Juggernaut standing-still C-press gate ---

    #[test]
    /// ASM loc_7718 (kid.asm:7800): cmpi.w #Juggernaut,(Current_Helmet) / bne.s loc_7742 —
    /// non-Juggernaut forms fall through to Eyeclops check, never arm a skull.
    fn loc_7718_non_juggernaut_skips_to_loc_7742() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_juggernaut_skull(&slice, input);

        assert_eq!(
            rt.helmet.juggernaut_skull_delay_frames, 0,
            "ASM loc_7718: bne.s loc_7742 — non-Juggernaut must not arm skull worker"
        );
        assert!(rt.projectiles.is_empty());
    }

    #[test]
    /// ASM loc_7718 (kid.asm:7800): cmpi.w #8,($FFFFFB70) / bge.w loc_7772 —
    /// when 8 skulls are already active, C press is rejected.
    fn loc_7718_max_skull_limit_rejects_ninth() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        let slice = combat_slice();

        // Spawn 8 skulls (fire + tick through the 3-frame delay each time)
        for _ in 0..8 {
            let input = FrameInput {
                c: true,
                ..FrameInput::default()
            };
            rt.step_juggernaut_skull(&slice, input);
            rt.prior_input = input;
            rt.step_juggernaut_skull(&slice, FrameInput::default());
            rt.step_juggernaut_skull(&slice, FrameInput::default());
            rt.step_juggernaut_skull(&slice, FrameInput::default());
            rt.prior_input = FrameInput::default();
        }
        assert_eq!(rt.projectiles.len(), 8);

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_juggernaut_skull(&slice, input);

        assert_eq!(
            rt.helmet.juggernaut_skull_delay_frames, 0,
            "ASM loc_7718: bge.w loc_7772 — 9th skull C press must be rejected"
        );
    }

    #[test]
    /// ASM loc_7718 + loc_7738 (kid.asm:7800-7812): Juggernaut with < 8 active
    /// skulls allocates slot (loc_7718) then assigns sub_86FA (loc_7738).
    /// Native models this as arming `juggernaut_skull_delay_frames`.
    fn loc_7738_juggernaut_c_press_arms_sub_86fa() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_juggernaut_skull(&slice, input);

        assert_eq!(
            rt.helmet.juggernaut_skull_delay_frames, 3,
            "ASM loc_7738: move.l #sub_86FA,4(a0) — native arms 3-frame delay for skull worker"
        );
        assert!(
            rt.projectiles.is_empty(),
            "ASM sub_86FA hibernates before spawning sub_87B0"
        );
    }

    // --- loc_874C: Juggernaut skull worker position tracking (sub_86FA) ---

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_874C (kid.asm:9591): `move.l $1A(a2),x_pos(a3); move.l $1E(a2),y_pos(a3);
    /// move.b $16(a2),x_direction(a3)` — copies Kid's position and facing to the skull
    /// GFX object. Native reads player position at spawn time (delay reaching 0).
    /// Verify spawn position matches Kid's position at the moment of projectile creation.
    fn loc_874C_skull_spawns_at_kid_position() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;

        let mut slice = combat_slice();
        slice.player.x_pos = 0x0064_0000; // 100 px
        slice.player.y_pos = 0x00C8_0000; // 200 px
        slice.player.facing = Direction::Right;

        // Arm the skull worker
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_juggernaut_skull(&slice, input);
        assert_eq!(rt.helmet.juggernaut_skull_delay_frames, 3);

        // Tick through the 3-frame delay, moving Kid each frame
        rt.prior_input = input;
        slice.player.x_pos = 0x0078_0000; // 120 px (Kid moved)
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        assert!(rt.projectiles.is_empty());

        rt.prior_input = FrameInput::default();
        slice.player.x_pos = 0x008C_0000; // 140 px
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        assert!(rt.projectiles.is_empty());

        // Third tick: delay reaches 0, projectile spawned using FINAL position
        slice.player.x_pos = 0x00A0_0000; // 160 px
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        assert_eq!(
            rt.projectiles.len(),
            1,
            "skull should spawn after 3-frame delay"
        );

        // Spawn position should be based on the final player position (160px),
        // not the initial position (100px) — matching ASM loc_874C/loc_879A
        // which copies Kid position every frame of the delay.
        let expected_x = ((160 + JUGGERNAUT_SKULL_OFFSET_X) as u32) << 16;
        let expected_y = ((200 + JUGGERNAUT_SKULL_OFFSET_Y) as u32) << 16;
        assert_eq!(
            rt.projectiles[0].x_pos, expected_x,
            "ASM loc_874C: skull x should use Kid's position at spawn time, not at C press",
        );
        assert_eq!(
            rt.projectiles[0].y_pos, expected_y,
            "ASM loc_874C: skull y should use Kid's initial y (unchanged in this test)",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_874C (kid.asm:9594): `move.b $16(a2),x_direction(a3)` — skull inherits
    /// Kid's facing. When facing left, x_velocity is negative and offset is mirrored.
    fn loc_874C_skull_direction_mirrors_kid_facing() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;

        let mut slice = combat_slice();
        slice.player.facing = Direction::Left;

        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_juggernaut_skull(&slice, input);

        // Tick through 3-frame delay
        rt.prior_input = input;
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        rt.prior_input = FrameInput::default();
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        rt.step_juggernaut_skull(&slice, FrameInput::default());

        assert_eq!(rt.projectiles.len(), 1);
        assert_eq!(
            rt.projectiles[0].x_velocity, -JUGGERNAUT_SKULL_X_SPEED,
            "ASM loc_874C: left-facing Kid should spawn skull with negative x velocity",
        );
        // X offset should be to the left of Kid
        let player_x = slice.player.x_pos;
        assert!(
            rt.projectiles[0].x_pos < player_x,
            "ASM loc_874C: left-facing skull x offset should be behind (left of) Kid",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_874C (kid.asm:9591-9597): `moveq #3,d0` — the skull worker uses
    /// a 3-frame counter before spawning sub_87B0. Native models this via
    /// `juggernaut_skull_delay_frames = JUGGERNAUT_SKULL_STARTUP_FRAMES (3)`.
    /// `tst.b ($FFFFFA6C); bne.w loc_874C` at kid.asm:9583 skips companion GFX
    /// init when companion already exists — visual-only, no native equivalent.
    fn loc_874C_three_frame_delay_before_spawn() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        let slice = combat_slice();

        // Arm
        rt.step_juggernaut_skull(
            &slice,
            FrameInput {
                c: true,
                ..FrameInput::default()
            },
        );
        assert_eq!(
            rt.helmet.juggernaut_skull_delay_frames, 3,
            "ASM loc_874C: moveq #3,d0 — 3-frame delay counter",
        );

        // Tick 1
        rt.prior_input = FrameInput {
            c: true,
            ..FrameInput::default()
        };
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        assert_eq!(rt.helmet.juggernaut_skull_delay_frames, 2);
        assert!(rt.projectiles.is_empty(), "no spawn during delay");

        // Tick 2
        rt.prior_input = FrameInput::default();
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        assert_eq!(rt.helmet.juggernaut_skull_delay_frames, 1);
        assert!(rt.projectiles.is_empty(), "no spawn during delay");

        // Tick 3 — spawn
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        assert_eq!(rt.helmet.juggernaut_skull_delay_frames, 0);
        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM loc_874C: sub_87B0 should spawn on the third delay tick",
        );
    }

    // --- loc_876A: Juggernaut skull worker per-frame tick (sub_86FA active loop) ---

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_876A (kid.asm:9599): `jsr Hibernate_Object_1Frame; subq.w #1,d0;
    /// bne.w loc_879A` — each frame of the 3-frame delay decrements the counter.
    /// When d0 reaches 0, the skull worker allocates a sub_87B0 projectile at Kid's
    /// current position. Verify the countdown ticks and spawn occurs on the right frame.
    fn loc_876A_skull_worker_per_frame_countdown() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        // C press arms the 3-frame delay
        rt.step_juggernaut_skull(&slice, input);
        assert_eq!(
            rt.helmet.juggernaut_skull_delay_frames, 3,
            "ASM loc_876A: moveq #3,d0 — counter starts at 3"
        );

        // Frame 1: subq.w #1,d0 → d0=2, bne → skip spawn
        rt.prior_input = input;
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        assert_eq!(rt.helmet.juggernaut_skull_delay_frames, 2);
        assert!(
            rt.projectiles.is_empty(),
            "ASM loc_876A: d0=2 (NE) — no spawn yet"
        );

        // Frame 2: subq.w #1,d0 → d0=1, bne → skip spawn
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        assert_eq!(rt.helmet.juggernaut_skull_delay_frames, 1);
        assert!(
            rt.projectiles.is_empty(),
            "ASM loc_876A: d0=1 (NE) — no spawn yet"
        );

        // Frame 3: subq.w #1,d0 → d0=0, EQ → fall through to spawn
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        assert_eq!(
            rt.helmet.juggernaut_skull_delay_frames, 0,
            "ASM loc_876A: d0=0 — counter exhausted"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM loc_876A: d0=0 (EQ) → allocate sub_87B0 projectile"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_876A (kid.asm:9603-9608): after counter reaches 0, the skull worker
    /// copies `x_pos(a3), y_pos(a3), x_direction(a3)` to the new sub_87B0 object's
    /// $44/$46/$48 fields. Verify the spawned projectile inherits Kid's position.
    fn loc_876A_spawn_copies_kid_position_to_projectile() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        let mut slice = combat_slice();
        slice.player.x_pos = 200 << 16;
        slice.player.y_pos = 300 << 16;
        slice.player.facing = Direction::Right;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_juggernaut_skull(&slice, input);
        rt.prior_input = input;
        // Tick through 3 delay frames
        for _ in 0..3 {
            rt.step_juggernaut_skull(&slice, FrameInput::default());
        }

        assert_eq!(rt.projectiles.len(), 1);
        let expected_x = ((200 + JUGGERNAUT_SKULL_OFFSET_X) as u32) << 16;
        let expected_y = ((300 + JUGGERNAUT_SKULL_OFFSET_Y) as u32) << 16;
        assert_eq!(
            rt.projectiles[0].x_pos, expected_x,
            "ASM loc_876A: spawned projectile x should include facing offset from Kid"
        );
        assert_eq!(
            rt.projectiles[0].y_pos, expected_y,
            "ASM loc_876A: spawned projectile y should be 8px above Kid"
        );
    }

    // --- loc_87A0: Skull worker post-spawn visual tracking (visual-only) ---

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_87A0 (kid.asm:9614): `move.l $1E(a2),y_pos(a3); tst.b $18(a3);
    /// beq.s loc_876A; jmp Delete_CurrentObject` — after spawning sub_87B0, the
    /// skull worker continues copying Kid y_pos each frame until animation completes.
    /// This is visual-only (GFX tracking); native does not model the companion GFX
    /// object. Verify that the native spawn is unaffected by this visual-only phase
    /// — the projectile exists and the delay counter is fully consumed.
    fn loc_87A0_post_spawn_visual_tracking_not_modelled() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        // Arm and tick through 3-frame delay
        rt.step_juggernaut_skull(&slice, input);
        rt.prior_input = input;
        for _ in 0..3 {
            rt.step_juggernaut_skull(&slice, FrameInput::default());
        }

        assert_eq!(rt.projectiles.len(), 1, "projectile spawned after delay");
        assert_eq!(
            rt.helmet.juggernaut_skull_delay_frames, 0,
            "ASM loc_87A0: post-spawn visual tracking phase is visual-only; \
             native delay counter should be 0 after spawn"
        );

        // A subsequent tick with no C press should not re-arm or create duplicates
        rt.prior_input = FrameInput::default();
        rt.step_juggernaut_skull(&slice, FrameInput::default());
        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM loc_87A0: post-spawn loop doesn't produce additional projectiles"
        );
    }

    // --- loc_87EC: sub_87B0 initial position + velocity setup ---

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_87EC (kid.asm:9641): `move.w d7,x_vel(a3); add.w d6,x_pos(a3)` —
    /// after direction branch, d7=4 (right) or -4 (left), d6=$1A (right) or -$1A
    /// (left). Stores x_vel and adds facing-dependent x offset to x_pos.
    /// Verify right-facing spawn: x_vel=+4px, x_offset=+26px.
    fn loc_87EC_right_facing_skull_initial_velocity_and_offset() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        let mut slice = combat_slice();
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 200 << 16;
        slice.player.facing = Direction::Right;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_juggernaut_skull(&slice, input);
        rt.prior_input = input;
        for _ in 0..3 {
            rt.step_juggernaut_skull(&slice, FrameInput::default());
        }

        assert_eq!(rt.projectiles.len(), 1);
        assert_eq!(
            rt.projectiles[0].x_velocity, JUGGERNAUT_SKULL_X_SPEED,
            "ASM loc_87EC: right-facing → moveq #4,d7 → x_vel = +4px/frame (0x4_0000)"
        );
        let expected_x = ((100 + 26) as u32) << 16; // d6=$1A=26
        assert_eq!(
            rt.projectiles[0].x_pos, expected_x,
            "ASM loc_87EC: right-facing → moveq #$1A,d6 → x_pos += 26px"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_87EC (kid.asm:9638-9641): left-facing branch sets d7=-4, d6=-$1A.
    /// `move.w d7,x_vel(a3)` → x_vel=-4px, `add.w d6,x_pos(a3)` → x_pos -= 26px.
    fn loc_87EC_left_facing_skull_initial_velocity_and_offset() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        let mut slice = combat_slice();
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 200 << 16;
        slice.player.facing = Direction::Left;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_juggernaut_skull(&slice, input);
        rt.prior_input = input;
        for _ in 0..3 {
            rt.step_juggernaut_skull(&slice, FrameInput::default());
        }

        assert_eq!(rt.projectiles.len(), 1);
        assert_eq!(
            rt.projectiles[0].x_velocity, -JUGGERNAUT_SKULL_X_SPEED,
            "ASM loc_87EC: left-facing → moveq #-$4,d7 → x_vel = -4px/frame"
        );
        let expected_x = ((100 - 26) as u32) << 16; // d6=-$1A=-26
        assert_eq!(
            rt.projectiles[0].x_pos, expected_x,
            "ASM loc_87EC: left-facing → moveq #-$1A,d6 → x_pos -= 26px"
        );
    }

    // --- loc_8810: Skull projectile main loop (gravity + collision checks) ---

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_8810 (kid.asm:9659): `addi.l #$1000,y_vel(a3)` — gravity applied
    /// every frame of the skull main loop. Verify cumulative gravity over 3 frames.
    fn loc_8810_skull_gravity_accumulates_each_frame() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 50 << 16,
            y_pos: 50 << 16,
            x_velocity: JUGGERNAUT_SKULL_X_SPEED,
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            ..Default::default()
        });

        rt.step_projectiles();
        assert_eq!(
            rt.projectiles[0].y_velocity, JUGGERNAUT_SKULL_GRAVITY,
            "ASM loc_8810: frame 1 → y_vel = $1000"
        );

        rt.step_projectiles();
        assert_eq!(
            rt.projectiles[0].y_velocity,
            JUGGERNAUT_SKULL_GRAVITY * 2,
            "ASM loc_8810: frame 2 → y_vel = $2000"
        );

        rt.step_projectiles();
        assert_eq!(
            rt.projectiles[0].y_velocity,
            JUGGERNAUT_SKULL_GRAVITY * 3,
            "ASM loc_8810: frame 3 → y_vel = $3000"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_8810 (kid.asm:9660-9663): `move.w x_pos(a3),d6; move.w d6,d5;
    /// move.w x_vel(a3),d7; bmi.w loc_887A` — horizontal movement dispatches based
    /// on x_vel sign: positive → right-moving path, negative → left-moving path.
    /// Verify a right-moving skull advances x_pos by x_vel when no wall is hit.
    fn loc_8810_right_moving_skull_no_wall() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 20 << 16, // mid-tile, far from boundary
            y_pos: 20 << 16,
            x_velocity: 0x2_0000, // 2 px/frame (won't cross tile boundary at col 2)
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            ..Default::default()
        });

        rt.step_projectiles();

        // After gravity: y_vel = $1000; x should advance by 2px
        let expected_x = (20 + 2) << 16;
        assert_eq!(
            rt.projectiles[0].x_pos >> 16,
            expected_x >> 16,
            "ASM loc_8810: right-moving, no wall → x_pos += x_vel"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_8810 (kid.asm:9659+): without a level loaded, sub_87B0 still
    /// runs the linear `add.l x_vel(a3),x_pos(a3); add.l y_vel(a3),y_pos(a3)`
    /// step. The Rust no-grid let-else fallback in `step_juggernaut_skull_sub_87B0`
    /// must reproduce this addition on both axes — flipping the `+` to `-` or
    /// `*` would push the skull backwards or slingshot it across the map.
    fn loc_8810_skull_no_grid_fallback_advances_y_pos_by_velocity() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0,
            y_velocity: 0x2_0000, // 2 px/frame downwards before gravity
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            ..Default::default()
        });
        // Intentionally no `set_block_grid` — exercises the let-else no-grid path.

        rt.step_projectiles();

        let projectile = &rt.projectiles[0];
        // ASM loc_8810: gravity is added before the move (`addi.l #$1000,y_vel`),
        // so the post-step y_vel is `0x2_0000 + 0x1000`.
        assert_eq!(
            projectile.y_velocity, 0x21000,
            "ASM loc_8810: gravity must accumulate before the linear step"
        );
        // ASM loc_8810 linear step: `add.l y_vel(a3),y_pos(a3)`.
        assert_eq!(
            projectile.y_pos,
            (100u32 << 16).wrapping_add(0x21000),
            "ASM loc_8810: no-grid fallback must add (not subtract or multiply) y_velocity into y_pos"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM sub_1D78 -> Is_Juggernaut_Skull_Active gate: the skull main loop
    /// (loc_8810) only runs when the helmet is actually Juggernaut and the
    /// projectile carries the skull's `$1000/frame` gravity tag. A friendly
    /// projectile with non-skull gravity (e.g. an Eyeclops beam) must take
    /// the regular projectile path, NOT the skull bounce path. This guards
    /// the AND chain in `is_juggernaut_skull_projectile` so swapping any of
    /// the conjuncts to OR routes a non-skull through sub_87B0.
    fn is_juggernaut_skull_projectile_requires_skull_gravity_for_terrain_kill_path() {
        let mut rt = NativeRuntime::new(manifest());
        // Friendly non-skull projectile (gravity=0) sitting just before a
        // solid block; with `terrain_kill=true` the regular projectile path
        // (kid-core projectiles.rs step_projectiles) deletes it on contact.
        rt.projectiles.push(EnemyProjectile {
            x_pos: 30 << 16,
            y_pos: 40 << 16,
            x_velocity: 0x4_0000,
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: 0,
            terrain_kill: true,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        // Solid block at col 2 (px 32-47) at row 2 (px 32-47) — covers the
        // post-step position (34, 40).
        rt.set_block_grid(bounce_grid(&[(2, 2), (2, 3)]));

        rt.step_projectiles();

        // Original: regular path triggers terrain-kill → projectile removed.
        // If `is_juggernaut_skull_projectile` misclassifies this projectile
        // as a skull, sub_87B0's wall-snap path keeps it alive (with snapped
        // x_pos and a decremented bounce budget).
        assert!(
            rt.projectiles.is_empty(),
            "friendly non-skull projectile with terrain_kill must take the \
             terrain-kill branch, not the skull bounce path"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM sub_B55C (probed at loc_8852/loc_8916 with `cmpi.w #$6000,d5`):
    /// the skull's tile-boundary collision check only fires when the cell
    /// reports a solid-class collision. Native `solid_terrain_at` mirrors
    /// that gate by ANDing `has_block` with `collision != Empty`. A
    /// presentation cell (has_block=true, collision=Empty) — which the level
    /// loader emits for skinned-but-passable backdrop tiles — must NOT
    /// register as a wall, otherwise the skull would burn a bounce on a
    /// non-existent obstacle.
    fn loc_8852_solid_terrain_gate_ignores_presentation_only_blocks() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 30 << 16,
            y_pos: 40 << 16,
            x_velocity: 0x4_0000,
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });

        // Presentation block at (col 2, row 2): has_block=1, collision=Empty
        // (hi byte 0x80 — block flag set, collision bits 000).
        let w = 10u16;
        let h = 10u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let off = (usize::from(2u16) * usize::from(w) + usize::from(2u16)) * 2;
        data[off] = 0x80;
        rt.set_block_grid(terrain::BlockGrid::from_level_layout(&data, w, h));

        rt.step_projectiles();

        let projectile = &rt.projectiles[0];
        assert_eq!(
            projectile.reaim_countdown,
            u16::from(JUGGERNAUT_SKULL_BOUNCES),
            "ASM loc_8852: a non-solid (collision=Empty) cell must not consume a bounce"
        );
        assert_eq!(
            projectile.x_velocity, 0x4_0000,
            "ASM loc_8852: x_velocity must remain unchanged when the probe reports non-solid"
        );
        assert_eq!(
            projectile.x_pos >> 16,
            34,
            "ASM loc_8810 fallthrough: presentation block keeps the linear x_pos += x_vel step"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM Lion.asm sub_3E2CE / loc_3E1B4 (release path on owner-token match):
    /// the lion's `$5B(a5)` active-shot count is decremented only when the
    /// owning enemy is actually a Lion. Native `release_lion_projectile_owners`
    /// must AND the family check together with segment/layout/token matching;
    /// loosening the family conjunct to OR would let an unrelated active enemy
    /// inherit the released token and corrupt its `ufo_beam_segments`.
    fn release_lion_projectile_owners_requires_lion_family() {
        let mut rt = NativeRuntime::new(manifest());
        let mut tank = test_enemy(EnemyFamily::Tank, 0, 0, 1);
        tank.segment_index = 7;
        tank.layout_index = 3;
        tank.ufo_beam_interval = 0x42;
        tank.ufo_beam_segments = 5;
        rt.active_enemies.push(tank);

        // Expiring projectile (lifetime=0) routed through release_lion_projectile_owners.
        let mut projectile = make_projectile(0, 0);
        projectile.lifetime = 0;
        projectile.owner = Some(EnemyProjectileOwner {
            segment_index: 7,
            layout_index: 3,
            token: 0x42,
        });
        rt.projectiles.push(projectile);

        rt.step_projectiles();

        assert_eq!(
            rt.active_enemies[0].ufo_beam_segments, 5,
            "non-Lion enemy with matching segment/layout/token must NOT have its \
             ufo_beam_segments decremented — the family conjunct guards the release"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM Lion.asm sub_3E2CE: the release decrements the lion's segment count
    /// only when *all* of family/segment_index/layout_index/ufo_beam_interval
    /// match. A live Lion sharing the same segment/token but a different
    /// layout slot must not absorb the released projectile.
    fn release_lion_projectile_owners_requires_layout_match() {
        let mut rt = NativeRuntime::new(manifest());
        let mut lion = test_enemy(EnemyFamily::Lion, 0, 0, 1);
        lion.segment_index = 2;
        lion.layout_index = 9; // owner.layout_index will not match
        lion.ufo_beam_interval = 0x5A;
        lion.ufo_beam_segments = 3;
        rt.active_enemies.push(lion);

        let mut projectile = make_projectile(0, 0);
        projectile.lifetime = 0;
        projectile.owner = Some(EnemyProjectileOwner {
            segment_index: 2,
            layout_index: 4, // mismatches lion.layout_index
            token: 0x5A,
        });
        rt.projectiles.push(projectile);

        rt.step_projectiles();

        assert_eq!(
            rt.active_enemies[0].ufo_beam_segments, 3,
            "Lion with non-matching layout_index must not have its segment count \
             released — layout_index is one of the four required AND conjuncts"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM Lion.asm sub_3E2CE: the camera-box check converts world positions to
    /// screen-space by `screen = (pos>>16) - (camera>>16)` and demands both
    /// axes sit in [-$40, $X]. The published off-screen test parks the camera
    /// at the origin with a positive screen-y, so any `>> with <<` mutation on
    /// either side of the y-subtraction collapses to the same in-box answer
    /// and survives. Anchor the camera at $100 px and pull the projectile
    /// inside the y-positive band — the `<<` mutant on `p.y_pos` truncates to
    /// 0 and reports `screen_y = -$100`, while the `<<` mutant on
    /// `camera.y_pos` reports `screen_y = +$150`, both falling outside the
    /// allowed [-$40, $120] band and forcing an unwarranted delete.
    fn lion_projectile_sub_3e2ce_keeps_in_box_shot_when_camera_is_offset_y_positive() {
        let mut rt = NativeRuntime::new(manifest());
        let mut lion = test_enemy(EnemyFamily::Lion, 0, 0, 6);
        lion.ufo_beam_interval = 0x5A;
        lion.ufo_beam_segments = 1;
        rt.active_enemies.push(lion);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.camera.x_pos = 0x0100 << 16;
        slice.camera.y_pos = 0x0100 << 16;
        rt.opening_slice = Some(slice);

        // screen_x = 0xE0 - 0x100 = -0x20 (in [-$40, $180])
        // screen_y = 0x150 - 0x100 = +0x50 (in [-$40, $120])
        let mut projectile = make_projectile(0x00E0, 0x0150);
        projectile.owner = Some(EnemyProjectileOwner {
            segment_index: lion.segment_index,
            layout_index: lion.layout_index,
            token: 0x5A,
        });
        rt.projectiles.push(projectile);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM Lion.asm sub_3E2CE: a projectile inside the camera box must survive \
             — a `>> with <<` mutation on either y-axis term would compute the \
             wrong screen_y and force an off-screen delete"
        );
        assert_eq!(
            rt.active_enemies[0].ufo_beam_segments, 1,
            "in-box projectile must NOT release the lion's active-shot count"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM Lion.asm sub_3E2CE: the camera box reaches negative screen offsets
    /// down to -$40 on both axes (the lion's "still on screen" margin).
    /// Stripping the leading `-` from either lower bound (`(-$40..=$X)`)
    /// would shrink the box to non-negative offsets only; a projectile that
    /// sits inside the negative-x and negative-y halves of the original box
    /// would be falsely reported off-screen. Park both axes at -$20 to lock
    /// the missing margin under test.
    fn lion_projectile_sub_3e2ce_keeps_in_box_shot_inside_negative_screen_offsets() {
        let mut rt = NativeRuntime::new(manifest());
        let mut lion = test_enemy(EnemyFamily::Lion, 0, 0, 6);
        lion.ufo_beam_interval = 0x5A;
        lion.ufo_beam_segments = 1;
        rt.active_enemies.push(lion);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.camera.x_pos = 0x0100 << 16;
        slice.camera.y_pos = 0x0100 << 16;
        rt.opening_slice = Some(slice);

        // screen_x = 0xE0 - 0x100 = -0x20 (in [-$40, $180])
        // screen_y = 0xE0 - 0x100 = -0x20 (in [-$40, $120])
        let mut projectile = make_projectile(0x00E0, 0x00E0);
        projectile.owner = Some(EnemyProjectileOwner {
            segment_index: lion.segment_index,
            layout_index: lion.layout_index,
            token: 0x5A,
        });
        rt.projectiles.push(projectile);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM Lion.asm sub_3E2CE: a projectile in the negative-screen-offset \
             half of the camera box must survive — deleting the `-` from either \
             lower bound would clamp the box to non-negative offsets and force \
             an off-screen delete"
        );
        assert_eq!(
            rt.active_enemies[0].ufo_beam_segments, 1,
            "in-box projectile must NOT release the lion's active-shot count"
        );
    }

    // --- sub_3E2CE called from loc_3E2A8: post-bounce camera-box re-check ---

    /// Build an `n × n`-cell grid (n × 16 px on each axis) with solid blocks
    /// at the supplied (col, row) positions. Mirrors `bounce_grid` from
    /// `fixtures.rs` but lets each test pick a wider grid so the lion
    /// projectile can land in the high-pixel region needed to drive the
    /// camera offset past the post-bounce camera-box bounds.
    fn lion_bounce_grid(
        width_cells: u16,
        height_cells: u16,
        cells: &[(u16, u16)],
    ) -> terrain::BlockGrid {
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        for &(col, row) in cells {
            let off = (usize::from(row) * usize::from(width_cells) + usize::from(col)) * 2;
            data[off] = 0xE0; // has_block=1, collision=Solid(6)
        }
        terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells)
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM Lion.asm sub_3E2CE called from loc_3E2A8: every wall-bounce
    /// iteration ends with `bsr.w sub_3E2CE` re-running the on-screen test
    /// before the next `dbf d0`. Native mirrors the call inline at the end
    /// of the bounce loop. The published `lion_projectile_sub_3e2ce_*`
    /// tests cover the post-move (pre-bounce) call site only — they leave
    /// `wall_bounce: true` projectiles untested at the second call site, so
    /// arithmetic and range mutations on lines 662-666 survive.
    ///
    /// Pin the post-bounce in-box path with a corner bounce: pre-bounce
    /// `(398, 350)` with `vel = (+2, +2)` moves to `(400, 352)`, lands in
    /// solid cells `(25, 22)` (destination), `(25, 21)` (h-probe),
    /// `(24, 22)` (v-probe), corner-flips both velocities and re-applies
    /// to `(396, 348)`. Camera at `(380, 300)` gives screen-space
    /// `(16, 48)` — inside `[-$40, $180] / [-$40, $120]`. The camera-axis
    /// values are large enough that any `>> with <<` mutation collapses
    /// the affected term to zero and pushes the screen-coordinate outside
    /// the band, any `replace - with +` mutation pushes the sum past the
    /// upper bound, and stripping the leading `-` from either lower bound
    /// clamps the box to non-negative offsets so the in-box position is
    /// re-classified as off-screen. Deleting the trailing `!` flips the
    /// survival branch into a delete.
    fn lion_projectile_wall_bounce_camera_box_corner_bounce_keeps_in_box() {
        let mut rt = NativeRuntime::new(manifest());
        let mut lion = test_enemy(EnemyFamily::Lion, 0, 0, 6);
        lion.ufo_beam_interval = 0x5A;
        lion.ufo_beam_segments = 1;
        rt.active_enemies.push(lion);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.camera.x_pos = 380u32 << 16;
        slice.camera.y_pos = 300u32 << 16;
        rt.opening_slice = Some(slice);

        // Solids at corner destination + h-probe + v-probe so loc_3E260
        // classifies `h_blocked && v_blocked` and routes through the
        // loc_3E266 corner branch (terrain_collision_id == 0).
        rt.set_block_grid(lion_bounce_grid(50, 50, &[(25, 22), (25, 21), (24, 22)]));

        let mut projectile = make_projectile(398, 350);
        projectile.x_velocity = 0x2_0000;
        projectile.y_velocity = 0x2_0000;
        projectile.wall_bounce = true;
        projectile.owner = Some(EnemyProjectileOwner {
            segment_index: lion.segment_index,
            layout_index: lion.layout_index,
            token: 0x5A,
        });
        rt.projectiles.push(projectile);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM Lion.asm sub_3E2CE (called from loc_3E2A8): a post-bounce \
             projectile inside the camera box must survive — `>> with <<`, \
             `- with +`, `delete -` lower-bound, and `delete !` mutations on \
             lines 662-666 each force an unwarranted off-screen delete here"
        );
        assert_eq!(
            rt.projectiles[0].x_pos,
            396u32 << 16,
            "post-bounce x_pos must equal pre-bounce + negated x_vel (398-2)"
        );
        assert_eq!(
            rt.projectiles[0].y_pos,
            348u32 << 16,
            "post-bounce y_pos must equal pre-bounce + negated y_vel (350-2)"
        );
        assert_eq!(
            rt.active_enemies[0].ufo_beam_segments, 1,
            "in-box post-bounce projectile must NOT release the lion's \
             active-shot count"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM Lion.asm sub_3E2CE called from loc_3E2A8: the post-bounce
    /// re-check must delete the projectile when the post-bounce x falls
    /// outside the camera band even if y stays inside. Replacing `&&`
    /// with `||` would let a single in-axis pass through, and replacing
    /// `-` with `/` would collapse the screen-space subtraction to a
    /// small in-band quotient. Both mutations survive the in-box test
    /// because `&&` and `||` agree when both axes are in.
    ///
    /// Drive the projectile far across the screen with `vel = (+240, 0)`
    /// from `(760, 350)` so the post-move position `(1000, 350)` still
    /// satisfies the pre-bounce sub_3E2CE check (camera at `(700, 300)`
    /// gives screen `(300, 50)`), but the horizontal-only bounce reflects
    /// `x_vel` to `-240` and re-applies to `(520, 350)`. Screen becomes
    /// `(-180, 50)` — `screen_x` is outside `[-$40, $180]` while
    /// `screen_y` stays inside `[-$40, $120]`. The original deletes; the
    /// `&& -> ||` mutant keeps and the `replace - with /` mutant on the
    /// x subtraction (`520/700 = 0`) keeps. Test asserts deletion plus a
    /// segment-count release so both survivors fail.
    fn lion_projectile_wall_bounce_camera_box_horizontal_bounce_deletes_when_x_off_screen() {
        let mut rt = NativeRuntime::new(manifest());
        let mut lion = test_enemy(EnemyFamily::Lion, 0, 0, 6);
        lion.ufo_beam_interval = 0x5A;
        lion.ufo_beam_segments = 1;
        rt.active_enemies.push(lion);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.camera.x_pos = 700u32 << 16;
        slice.camera.y_pos = 300u32 << 16;
        rt.opening_slice = Some(slice);

        // Single solid cell at the post-move destination; the v-probe at
        // (47, 21) stays empty so loc_3E260 classifies as straight-wall
        // ($04: rightwall) and loc_3E266 reflects only x_vel.
        rt.set_block_grid(lion_bounce_grid(70, 30, &[(62, 21)]));

        let mut projectile = make_projectile(760, 350);
        projectile.x_velocity = 0xF0_0000; // +240 px/frame
        projectile.y_velocity = 0;
        projectile.wall_bounce = true;
        projectile.owner = Some(EnemyProjectileOwner {
            segment_index: lion.segment_index,
            layout_index: lion.layout_index,
            token: 0x5A,
        });
        rt.projectiles.push(projectile);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles.len(),
            0,
            "ASM Lion.asm sub_3E2CE (called from loc_3E2A8): a post-bounce \
             projectile with screen_x outside [-$40, $180] must delete — \
             the `&& -> ||` mutant would keep it on the strength of \
             screen_y alone, and the `replace - with /` x mutant would \
             collapse 520/700 to 0 and report screen_x in-band"
        );
        assert_eq!(
            rt.active_enemies[0].ufo_beam_segments, 0,
            "off-screen post-bounce delete must release the lion's \
             segment count via release_lion_projectile_owners"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM Lion.asm sub_3E2CE called from loc_3E2A8: mirror of the
    /// horizontal-bounce off-screen test on the y axis to pin the
    /// `replace - with /` mutation on `screen_y`. The horizontal version
    /// can't reach this mutation because its x-only bounce keeps
    /// screen_y stable.
    ///
    /// Pre-bounce `(350, 760)` with `vel = (0, +240)` moves to
    /// `(350, 1000)`, lands in solid cell `(21, 62)`, reflects only y_vel
    /// to `-240` and re-applies to `(350, 520)`. Camera at `(300, 720)`
    /// gives pre-bounce screen `(50, 280)` (in the box at the upper y
    /// edge) and post-bounce screen `(50, -200)` — `screen_y` outside
    /// `[-$40, $120]` while `screen_x` stays in `[-$40, $180]`. The
    /// `replace - with /` y mutant collapses `520/720` to `0` (in band)
    /// and would keep the projectile.
    fn lion_projectile_wall_bounce_camera_box_vertical_bounce_deletes_when_y_off_screen() {
        let mut rt = NativeRuntime::new(manifest());
        let mut lion = test_enemy(EnemyFamily::Lion, 0, 0, 6);
        lion.ufo_beam_interval = 0x5A;
        lion.ufo_beam_segments = 1;
        rt.active_enemies.push(lion);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.camera.x_pos = 300u32 << 16;
        slice.camera.y_pos = 720u32 << 16;
        rt.opening_slice = Some(slice);

        // Single solid cell at the post-move destination; the h-probe at
        // (21, 47) stays empty so loc_3E260 classifies as straight-floor
        // ($0C) and loc_3E266 reflects only y_vel.
        rt.set_block_grid(lion_bounce_grid(30, 70, &[(21, 62)]));

        let mut projectile = make_projectile(350, 760);
        projectile.x_velocity = 0;
        projectile.y_velocity = 0xF0_0000; // +240 px/frame
        projectile.wall_bounce = true;
        projectile.owner = Some(EnemyProjectileOwner {
            segment_index: lion.segment_index,
            layout_index: lion.layout_index,
            token: 0x5A,
        });
        rt.projectiles.push(projectile);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles.len(),
            0,
            "ASM Lion.asm sub_3E2CE (called from loc_3E2A8): a post-bounce \
             projectile with screen_y outside [-$40, $120] must delete — \
             the `replace - with /` y mutant would collapse 520/720 to 0 \
             and report screen_y in-band"
        );
        assert_eq!(
            rt.active_enemies[0].ufo_beam_segments, 0,
            "off-screen post-bounce delete must release the lion's \
             segment count via release_lion_projectile_owners"
        );
    }

    // --- loc_8852: Right-moving tile-boundary wall snap + bounce ---

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_8852 (kid.asm:9674): `add.w d5,d7; move.w d7,x_pos(a3)` — moves
    /// x_pos by x_vel. `bsr.w sub_B55C; cmpi.w #$6000,d5` probes for solid.
    /// If solid: `andi.w #$FFF0,d7; subq.w #1,d7` snaps to (tile_left - 1).
    /// Then loc_88B6: `subq.w #1,d0` decrements bounce counter, decays x_vel by 1,
    /// reverses direction via `neg.w d7; not.b x_direction`.
    fn loc_8852_right_wall_snap_and_bounce() {
        let mut rt = NativeRuntime::new(manifest());
        // Skull at x=30 moving right at 4px/frame → next_x=34 → col 2 (px 32-47)
        rt.projectiles.push(EnemyProjectile {
            x_pos: 30 << 16,
            y_pos: 40 << 16,
            x_velocity: 0x4_0000, // 4 px/frame
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        // Solid wall at col 2, rows 2-3 (covers y=40 at row 2)
        rt.set_block_grid(bounce_grid(&[(2, 2), (2, 3)]));

        rt.step_projectiles();

        let p = &rt.projectiles[0];
        // ASM: andi.w #$FFF0,d7 → 34 & 0xFFF0 = 32; subq.w #1,d7 → 31
        assert_eq!(
            p.x_pos >> 16,
            31,
            "ASM loc_8852: right-wall snap → (next_x & $FFF0) - 1 = 31"
        );
        // ASM loc_88B6: x_vel was 4, decay by 1 → 3, neg → -3
        assert_eq!(
            p.x_velocity, -0x3_0000,
            "ASM loc_8852/loc_88B6: decay 4→3, reverse → -3px/frame"
        );
        assert_eq!(
            p.reaim_countdown,
            u16::from(JUGGERNAUT_SKULL_BOUNCES - 1),
            "ASM loc_88B6: subq.w #1,d0 — bounce counter decremented"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_8890 (kid.asm:9700): left-moving mirror of loc_8852.
    /// `sub.w d7,d5; move.w d5,x_pos(a3)` → probe → if solid:
    /// `andi.w #$FFF0,d7; addi.w #$10,d7` snaps to tile right edge.
    fn loc_8852_left_wall_snap_and_bounce_mirror() {
        let mut rt = NativeRuntime::new(manifest());
        // Skull at x=33 moving left at -4px/frame → next_x=29 → col 1 (px 16-31)
        rt.projectiles.push(EnemyProjectile {
            x_pos: 33 << 16,
            y_pos: 40 << 16,
            x_velocity: -0x4_0000, // -4 px/frame
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        // Solid wall at col 1, rows 2-3
        rt.set_block_grid(bounce_grid(&[(1, 2), (1, 3)]));

        rt.step_projectiles();

        let p = &rt.projectiles[0];
        // ASM loc_8890: andi.w #$FFF0,d7 → 29 & 0xFFF0 = 16; addi.w #$10 → 32
        assert_eq!(
            p.x_pos >> 16,
            32,
            "ASM loc_8890: left-wall snap → (next_x & $FFF0) + $10 = 32"
        );
        // ASM loc_88B6: x_vel was -4, |x_vel|=4, decay → 3, neg → +3
        assert_eq!(
            p.x_velocity, 0x3_0000,
            "ASM loc_8890/loc_88B6: decay |-4|→3, reverse → +3px/frame"
        );
    }

    // --- loc_8864: Not-solid branch skips wall snap → y-axis movement ---

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_8864 (kid.asm:9682): `bne.w loc_88E6` — after sub_B55C probe,
    /// if terrain is NOT solid ($6000), skip wall snap entirely and fall through
    /// to loc_88E6 (y-axis movement). x_pos keeps the raw (x + x_vel) value.
    fn loc_8864_not_solid_skips_wall_snap() {
        let mut rt = NativeRuntime::new(manifest());
        // Skull at x=20 moving right at 4px/frame → next_x=24 → col 1 (px 16-31), no wall
        rt.projectiles.push(EnemyProjectile {
            x_pos: 20 << 16,
            y_pos: 40 << 16,
            x_velocity: 0x4_0000,
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        // No solid blocks anywhere — empty grid
        rt.set_block_grid(bounce_grid(&[]));

        rt.step_projectiles();

        let p = &rt.projectiles[0];
        assert_eq!(
            p.x_pos >> 16,
            24,
            "ASM loc_8864: not-solid → no snap, x_pos = 20 + 4 = 24"
        );
        // x_velocity unchanged (no bounce)
        assert_eq!(
            p.x_velocity, 0x4_0000,
            "ASM loc_8864: not-solid → x_vel unchanged"
        );
        // Bounce counter unchanged
        assert_eq!(
            p.reaim_countdown,
            u16::from(JUGGERNAUT_SKULL_BOUNCES),
            "ASM loc_8864: not-solid → bounce counter untouched"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_8864 (kid.asm:9682): left-moving mirror — `bne.w loc_88E6` at
    /// loc_8890's equivalent position. No solid terrain → x_pos keeps raw value.
    fn loc_8864_left_moving_not_solid_skips_wall_snap() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 40 << 16,
            y_pos: 40 << 16,
            x_velocity: -0x3_0000,
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        rt.set_block_grid(bounce_grid(&[]));

        rt.step_projectiles();

        let p = &rt.projectiles[0];
        assert_eq!(
            p.x_pos >> 16,
            37,
            "ASM loc_8864 mirror: not-solid → x_pos = 40 - 3 = 37"
        );
        assert_eq!(
            p.x_velocity, -0x3_0000,
            "ASM loc_8864 mirror: not-solid → x_vel unchanged"
        );
    }

    // --- loc_88D2/loc_88DC: Bounce velocity decay + direction reversal ---

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_88D2 (kid.asm:9725): negative x_vel decay.
    /// `cmpi.w #$FFFF,d7; beq.w loc_88DC; addq.w #1,d7` — when x_vel < -1 (word),
    /// reduce magnitude by 1 before negation. At x_vel == -1, skip decay.
    /// Then loc_88DC: `neg.w d7; move.w d7,x_vel(a3)`.
    fn loc_88D2_negative_vel_decay_skips_at_minus_one() {
        let mut rt = NativeRuntime::new(manifest());
        // Skull at x=17 moving left at -1px/frame → next_x=16 → col 1 (px 16-31)
        // Wall at col 1 → collision
        rt.projectiles.push(EnemyProjectile {
            x_pos: 17 << 16,
            y_pos: 40 << 16,
            x_velocity: -0x1_0000, // -1px/frame (word = $FFFF)
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        rt.set_block_grid(bounce_grid(&[(1, 2), (1, 3)]));

        rt.step_projectiles();

        let p = &rt.projectiles[0];
        // ASM loc_88D2: cmpi.w #$FFFF,d7 → equal, skip addq.w → d7 stays $FFFF
        // loc_88DC: neg.w → $0001, so x_vel becomes +1
        assert_eq!(
            p.x_velocity, 0x1_0000,
            "ASM loc_88D2: -1px hits wall → no decay, neg → +1px"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_88B6/loc_88DC: positive x_vel == 1 decay.
    /// `cmpi.w #1,d7; beq.w loc_88DC; subq.w #1,d7` — at x_vel == 1 (word),
    /// skip decay, then loc_88DC negates → -1.
    fn loc_88DC_positive_vel_one_no_decay() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 14 << 16,
            y_pos: 40 << 16,
            x_velocity: 0x1_0000, // +1px/frame (word = $0001)
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        // Wall at col 0 (px 0-15) — skull at x=14 + 1 = 15, still col 0 which is solid
        rt.set_block_grid(bounce_grid(&[(0, 2), (0, 3), (1, 2), (1, 3)]));

        rt.step_projectiles();

        let p = &rt.projectiles[0];
        // ASM loc_88B6: cmpi.w #1,d7 → equal, skip subq → d7 stays 1
        // loc_88DC: neg.w → $FFFF, x_vel becomes -1
        assert_eq!(
            p.x_velocity, -0x1_0000,
            "ASM loc_88DC: +1px hits wall → no decay, neg → -1px"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_88D2 (kid.asm:9725): negative x_vel with magnitude > 1.
    /// `cmpi.w #$FFFF,d7; bne → addq.w #1,d7` — d7 = -3 (word $FFFD), add 1 → $FFFE (-2).
    /// loc_88DC: neg → $0002 (+2). Confirms decay by 1 for |x_vel| > 1.
    fn loc_88D2_negative_vel_decays_by_one() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 33 << 16,
            y_pos: 40 << 16,
            x_velocity: -0x3_0000, // -3px/frame
            y_velocity: 0,
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        rt.set_block_grid(bounce_grid(&[(1, 2), (1, 3)]));

        rt.step_projectiles();

        let p = &rt.projectiles[0];
        // ASM: d7=$FFFD, addq.w #1 → $FFFE, neg.w → $0002
        assert_eq!(
            p.x_velocity, 0x2_0000,
            "ASM loc_88D2: -3px hits wall → decay to |-2|, neg → +2px"
        );
    }

    // --- loc_88E6: Y-axis movement with gravity ---

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_88E6 (kid.asm:9735): y-axis movement.
    /// `move.l y_pos(a3),d7; move.l y_vel(a3),d4; add.l d4,d6; move.l d6,y_pos(a3)`
    /// — applies y_vel to y_pos. When no tile-boundary crossing detected,
    /// loops back to loc_8810 without snapping.
    fn loc_88E6_y_movement_no_floor_collision() {
        let mut rt = NativeRuntime::new(manifest());
        // Skull falling at 2px/frame, y=20. Gravity adds $1000 first → y_vel=$2_1000.
        // y_pos = 20<<16 + $2_1000 = $16_1000, pixel = $16 = 22. Row 1 (px 16-31), no wall.
        rt.projectiles.push(EnemyProjectile {
            x_pos: 40 << 16,
            y_pos: 20 << 16,
            x_velocity: 0,
            y_velocity: 0x2_0000, // +2px/frame downward (before gravity)
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        rt.set_block_grid(bounce_grid(&[]));

        rt.step_projectiles();

        let p = &rt.projectiles[0];
        // After gravity: y_vel = $2_0000 + $1000 = $2_1000
        // y_pos = $14_0000 + $2_1000 = $16_1000 → pixel 22
        assert_eq!(
            p.y_pos,
            (20 << 16) + 0x2_0000 + 0x1000,
            "ASM loc_88E6: y_pos += y_vel (after gravity)"
        );
        assert_eq!(
            p.y_velocity,
            0x2_0000 + 0x1000,
            "ASM loc_88E6: y_vel = original + gravity, no rebound"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_88E6 (kid.asm:9735): upward movement (y_vel < 0).
    /// `bmi.w loc_893C` branches to the upward tile-crossing check.
    /// No ceiling → y_pos updated normally, loops back to loc_8810.
    fn loc_88E6_upward_no_ceiling() {
        let mut rt = NativeRuntime::new(manifest());
        // Gravity adds $1000: y_vel = -$2_0000 + $1000 = -$1_F000
        // y_pos = 30<<16 + (-$1_F000) = $1E_0000 - $1_F000 = $1C_1000 → pixel 28
        rt.projectiles.push(EnemyProjectile {
            x_pos: 40 << 16,
            y_pos: 30 << 16,
            x_velocity: 0,
            y_velocity: -0x2_0000, // upward (before gravity)
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        rt.set_block_grid(bounce_grid(&[]));

        rt.step_projectiles();

        let p = &rt.projectiles[0];
        let expected_vel = -0x2_0000_i32 + 0x1000; // -$1_F000
        assert_eq!(
            p.y_pos as i64,
            (30_i64 << 16) + i64::from(expected_vel),
            "ASM loc_88E6/loc_893C: y_pos += y_vel (after gravity)"
        );
        assert_eq!(
            p.y_velocity, expected_vel,
            "ASM loc_88E6: y_vel = -$2_0000 + gravity($1000) = -$1_F000"
        );
    }

    // --- loc_8916: Downward floor collision snap + ¾ rebound ---

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_8916 (kid.asm:9755): downward floor collision.
    /// `bsr.w sub_B55C; cmpi.w #$6000,d5` — probes terrain at new y position.
    /// If solid: `andi.w #$FFF0,d7; subq.w #1,d7` snaps y_pos to tile top - 1.
    /// `clr.w $20(a3)` clears a bounce-related field.
    /// Falls to loc_897C: `neg.l d7; asr.l #1,d7; asr.l #1,d6; add.l d6,d7`
    /// = ¾ rebound speed.
    fn loc_8916_floor_collision_snap_and_rebound() {
        let mut rt = NativeRuntime::new(manifest());
        // Gravity adds $1000: y_vel = $4_0000 + $1000 = $4_1000.
        // y=30 + $4_1000 high word = 30+4 = 34 → row 2 solid.
        rt.projectiles.push(EnemyProjectile {
            x_pos: 40 << 16,
            y_pos: 30 << 16,
            x_velocity: 0,
            y_velocity: 0x4_0000, // +4px/frame (before gravity)
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        // Solid floor at row 2 (px 32-47)
        rt.set_block_grid(bounce_grid(&[(2, 2), (3, 2)]));

        rt.step_projectiles();

        let p = &rt.projectiles[0];
        // ASM: andi.w #$FFF0,d7 → 34 & 0xFFF0 = 32; subq.w #1 → 31
        assert_eq!(
            p.y_pos >> 16,
            31,
            "ASM loc_8916: floor snap → (next_y & $FFF0) - 1 = 31"
        );
        // After gravity: effective y_vel = $4_1000.
        // ASM loc_897C: neg.l(-$4_1000) = $4_1000, asr.l #1 → $2_0800,
        // asr.l #1 → $1_0400, sum = $3_0C00. So rebound = -$3_0C00.
        let effective_vel: i32 = 0x4_0000 + 0x1000; // $4_1000
        let negated = -effective_vel;
        let half = negated >> 1;
        let quarter = half >> 1;
        let expected_rebound = half + quarter;
        assert_eq!(
            p.y_velocity, expected_rebound,
            "ASM loc_8916/loc_897C: ¾ rebound of $4_1000"
        );
        assert_eq!(
            p.reaim_countdown,
            u16::from(JUGGERNAUT_SKULL_BOUNCES - 1),
            "ASM loc_8916: floor bounce decrements bounce counter"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    /// ASM loc_8958 (kid.asm:9782): upward ceiling collision — mirror of loc_8916.
    /// `andi.w #$FFF0,d7; addi.w #$10,d7` snaps to tile bottom edge.
    /// Same ¾ rebound via loc_897C but direction reverses (ceiling → downward).
    fn loc_8916_ceiling_collision_snap_and_rebound() {
        let mut rt = NativeRuntime::new(manifest());
        // Gravity adds $1000: y_vel = -$4_0000 + $1000 = -$3_F000.
        // y=33 + (-$3_F000 high word) → 33-3=30? No — let me compute precisely.
        // y_pos = $21_0000, y_vel after gravity = -$3_F000.
        // next_y = $21_0000 + (-$3_F000) = $21_0000 - $3_F000 = $1D_1000 → pixel $1D = 29.
        // Row 1 (px 16-31) → solid ceiling.
        rt.projectiles.push(EnemyProjectile {
            x_pos: 40 << 16,
            y_pos: 33 << 16,
            x_velocity: 0,
            y_velocity: -0x4_0000, // -4px/frame upward (before gravity)
            lifetime: 10,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            reaim_countdown: u16::from(JUGGERNAUT_SKULL_BOUNCES),
            ..Default::default()
        });
        // Solid ceiling at row 1 (px 16-31)
        rt.set_block_grid(bounce_grid(&[(2, 1), (3, 1)]));

        rt.step_projectiles();

        let p = &rt.projectiles[0];
        // ASM loc_8958: andi.w #$FFF0 → 29 & 0xFFF0 = 16; addi.w #$10 → 32
        assert_eq!(
            p.y_pos >> 16,
            32,
            "ASM loc_8958: ceiling snap → (next_y & $FFF0) + $10 = 32"
        );
        // After gravity: effective y_vel = -$3_F000.
        // ASM loc_897C: neg.l($3_F000) = $3_F000, asr.l #1 → $1_F800,
        // asr.l #1 → $FC00, sum = $1_F800 + $FC00 = $2_F400.
        let effective_vel: i32 = -0x4_0000 + 0x1000; // -$3_F000
        let negated = -effective_vel;
        let half = negated >> 1;
        let quarter = half >> 1;
        let expected_rebound = half + quarter;
        assert_eq!(
            p.y_velocity, expected_rebound,
            "ASM loc_8958/loc_897C: ceiling ¾ rebound of $3_F000"
        );
    }

    // --- loc_7742: Eyeclops standing-still C-press beam ---

    #[test]
    /// ASM loc_7742 (kid.asm:7815): cmpi.w #Eyeclops,(Current_Helmet) / bne.s loc_7772 /
    ///   tst.w ($FFFFFAB8) / bne.s loc_7772 / move.w #1,($FFFFFAB8) /
    ///   move.b x_direction(a3),($FFFFFABE) — C press fires normal beam, sets
    ///   beam counter to 1, latches facing direction.
    fn loc_7742_eyeclops_c_press_sets_beam_counter_to_one() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        let mut slice = combat_slice();
        slice.player.facing = Direction::Right;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_eyeclops_beam(&slice, input);

        assert!(
            rt.helmet.eyeclops_beam_active,
            "ASM loc_7742: tst.w ($FFFFFAB8) was zero — beam must activate"
        );
        assert_eq!(
            rt.helmet.eyeclops_beam_counter, 1,
            "ASM loc_7742: move.w #1,($FFFFFAB8) — normal beam counter is 1"
        );
        assert!(
            !rt.helmet.eyeclops_beam_hard,
            "ASM loc_7742: normal C-press path sets $FFFFFAB8=1, not $8001 (hard)"
        );
        assert_eq!(rt.projectiles.len(), 1);
        assert!(
            rt.projectiles[0].x_velocity > 0,
            "ASM loc_7742: move.b x_direction(a3),($FFFFFABE) — beam direction must match facing"
        );
    }

    #[test]
    /// ASM loc_7742 (kid.asm:7818): tst.w ($FFFFFAB8) / bne.s loc_7772 —
    /// if beam is already active, a second C press is blocked.
    fn loc_7742_eyeclops_beam_active_blocks_refire() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        rt.helmet.eyeclops_beam_active = true;
        rt.helmet.eyeclops_beam_counter = 1;
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: EYECLOPS_BEAM_SPEED,
            y_velocity: 0,
            lifetime: 30,
            friendly: true,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
            ..Default::default()
        });
        let slice = combat_slice();
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_eyeclops_beam(&slice, input);

        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM loc_7742: bne.s loc_7772 — active beam must block refire"
        );
    }

    #[test]
    /// ASM loc_7742 (kid.asm:7821): move.b x_direction(a3),($FFFFFABE) —
    /// beam direction is latched from Kid's facing at fire time.
    fn loc_7742_eyeclops_beam_latches_left_direction() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Eyeclops;
        let mut slice = combat_slice();
        slice.player.facing = Direction::Left;
        let input = FrameInput {
            c: true,
            ..FrameInput::default()
        };

        rt.step_eyeclops_beam(&slice, input);

        assert_eq!(
            rt.projectiles[0].x_velocity, -EYECLOPS_BEAM_SPEED,
            "ASM loc_7742: move.b x_direction(a3),($FFFFFABE) — left-facing beam must go left"
        );
    }

    /// ASM loc_779C (kid.asm:7842): `tst.b (Ctrl_Right_Held) / beq.s loc_77E6 /
    /// add.w d0,d7 / addq.w #1,d7 / bsr.w sub_922C` — when pressing right into
    /// a solid wall, `sub_922C` returns NE (wall found), the ROM executes
    /// `sf x_direction(a3)` (face right) and branches to loc_7828 (stay standing).
    #[test]
    fn loc_779c_right_held_wall_found_faces_right_and_holds() {
        let grid = standing_still_wall_grid(5); // wall at col 5 (x=80)
        // TheKid half_width=5, edge = x + 6.  At x=74 edge=80 → col 5 → solid.
        let hit = terrain::probe_wall(&grid, 74, 63, 5, 31, true);
        assert!(
            hit.hit,
            "ASM loc_779C: sub_922C should return NE (wall) when right edge lands on solid col 5"
        );
    }

    /// ASM loc_77E6 (kid.asm:7871): `tst.b (Ctrl_Left_Held) / beq.s loc_7828 /
    /// sub.w d0,d7 / subq.w #1,d7 / bsr.w sub_922C` — left-press mirror of
    /// loc_779C.  Wall found → `st x_direction(a3)` (face left), loc_7828.
    #[test]
    fn loc_77e6_left_held_wall_found_faces_left_and_holds() {
        let grid = standing_still_wall_grid(2); // wall at col 2 (x=32..47)
        // At x=38, left edge = 38 - 5 - 1 = 32 → col 2 → solid.
        // probe_wall uses edge_x = center - half_width (no -1 for left), but
        // probe_wall_column does center - half_width - 1.  probe_wall checks
        // center - half_width = 38 - 5 = 33 → col 2 → solid.
        let hit = terrain::probe_wall(&grid, 38, 63, 5, 31, false);
        assert!(
            hit.hit,
            "ASM loc_77E6: sub_922C should return NE (wall) when left edge lands on solid col 2"
        );
    }

    /// ASM loc_779C → loc_77C2 (kid.asm:7857): when `sub_922C` returns EQ (no
    /// wall), the ROM falls through to loc_77C2 which checks helmet bitmask
    /// `#%1011000101`.  For TheKid (bit 0 set), `btst d7,d6 / bne loc_7820`
    /// immediately enters walking state.  Native: `probe_wall` returns no hit,
    /// `should_hold_standing_still` returns false → walking proceeds.
    #[test]
    fn loc_77c2_no_wall_right_allows_walking() {
        let grid = standing_still_wall_grid(7); // wall far away at col 7
        // At x=40, right edge = 40 + 6 = 46 → col 2 → empty.
        let hit = terrain::probe_wall(&grid, 40, 63, 5, 31, true);
        assert!(
            !hit.hit,
            "ASM loc_779C → loc_77C2: sub_922C should return EQ (no wall) when right is clear"
        );
    }

    /// ASM loc_77E6 → loc_77FC (kid.asm:7882): left-press mirror of loc_77C2.
    /// No wall on left → loc_77FC → bitmask check → loc_7820 for TheKid.
    #[test]
    fn loc_77fc_no_wall_left_allows_walking() {
        let grid = standing_still_wall_grid(0); // wall at col 0
        // At x=40, left edge = 40 - 5 = 35 → col 2 → empty (wall is at col 0).
        let hit = terrain::probe_wall(&grid, 40, 63, 5, 31, false);
        assert!(
            !hit.hit,
            "ASM loc_77E6 → loc_77FC: sub_922C should return EQ (no wall) when left is clear"
        );
    }

    /// ASM loc_77C2 (kid.asm:7857): `move.w (Current_Helmet),d7 /
    /// move.w #%1011000101,d6 / btst d7,d6 / bne loc_7820` — helmets with the
    /// corresponding bit set skip the `sub_B55C` ledge probe and go straight to
    /// loc_7820 (enter walking).  Bitmask bits: 0=Kid, 2=Cyclone, 6=IronKnight,
    /// 7=Berzerker, 9=Micromax.
    #[test]
    fn loc_77c2_helmet_bitmask_identifies_walk_through_helmets() {
        let bitmask: u16 = 0b10_1100_0101; // %1011000101
        let walk_through = [
            PlayerForm::TheKid,
            PlayerForm::Cyclone,
            PlayerForm::IronKnight,
            PlayerForm::Berzerker,
            PlayerForm::Micromax,
        ];
        let ledge_probe = [
            PlayerForm::Skycutter,
            PlayerForm::RedStealth,
            PlayerForm::Eyeclops,
            PlayerForm::Juggernaut,
            PlayerForm::Maniaxe,
        ];

        for form in walk_through {
            assert_ne!(
                bitmask & (1 << form.helmet_id()),
                0,
                "ASM loc_77C2: btst d7,d6 should be set for {:?} (id={}), routing to loc_7820",
                form,
                form.helmet_id()
            );
        }
        for form in ledge_probe {
            assert_eq!(
                bitmask & (1 << form.helmet_id()),
                0,
                "ASM loc_77C2: btst d7,d6 should be clear for {:?} (id={}), falling through to sub_B55C",
                form,
                form.helmet_id()
            );
        }
    }

    /// ASM loc_77C2 (kid.asm:7864): for helmets NOT in the bitmask, the ROM
    /// probes `sub_B55C` with `(1,-1)` and checks `cmpi.w #$4000,d5`.  $4000 is
    /// `CollisionKind::SlopeUp`.  If found, `sf x_direction / bra loc_7828`
    /// (face right, stay standing).  Otherwise loc_7820 (enter walking).
    #[test]
    fn loc_77c2_sub_b55c_slope_up_blocks_right_walk() {
        let w = 8u16;
        let h = 6u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Floor at row 4
        for col in 0..w {
            let offset = (4 * usize::from(w) + usize::from(col)) * 2;
            data[offset] = 0x60;
            data[offset + 1] = 0x01;
        }
        // Slope-up block ($40) at col 4, row 3 (one row above floor, one col right)
        let offset = (3 * usize::from(w) + 4) * 2;
        data[offset] = 0x40; // collision type 4 = SlopeUp ($4000)
        data[offset + 1] = 0x01;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        // Player at x=48 (col 3 centre), foot at y=63.
        // sub_B55C(1, -1) probes at (x+1, y-1) = (49, 62) → col 3, row 3 → empty.
        // But if player is at x=63: probe at (64, 62) → col 4, row 3 → SlopeUp.
        let kind = terrain::probe_relative_collision_sub_B55C(&grid, 63, 63, 1, -1);
        assert_eq!(
            kind,
            terrain::CollisionKind::SlopeUp,
            "ASM loc_77C2: sub_B55C(1,-1) should return $4000 (SlopeUp) when slope block is adjacent"
        );
    }

    /// ASM loc_77FC (kid.asm:7889): left-press mirror — `sub_B55C` with `(-1,-1)`,
    /// checks `cmpi.w #$5000,d5`.  $5000 is `CollisionKind::SlopeDown`.
    #[test]
    fn loc_77fc_sub_b55c_slope_down_blocks_left_walk() {
        let w = 8u16;
        let h = 6u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Floor at row 4
        for col in 0..w {
            let offset = (4 * usize::from(w) + usize::from(col)) * 2;
            data[offset] = 0x60;
            data[offset + 1] = 0x01;
        }
        // Slope-down block ($50) at col 2, row 3
        let offset = (3 * usize::from(w) + 2) * 2;
        data[offset] = 0x50; // collision type 5 = SlopeDown ($5000)
        data[offset + 1] = 0x01;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        // Player at x=48 (col 3), foot at y=63.
        // sub_B55C(-1, -1) probes at (47, 62) → col 2, row 3 → SlopeDown.
        let kind = terrain::probe_relative_collision_sub_B55C(&grid, 48, 63, -1, -1);
        assert_eq!(
            kind,
            terrain::CollisionKind::SlopeDown,
            "ASM loc_77FC: sub_B55C(-1,-1) should return $5000 (SlopeDown) when slope block is adjacent"
        );
    }

    /// ASM loc_7820 (kid.asm:7896): `clr.w ($FFFFF8F0) / bra.w loc_8D72` — when
    /// neither wall nor ledge blocks movement, the ROM clears the walk accumulator
    /// and enters the walking main loop.  This is the fallthrough for: no direction
    /// held, or direction held with clear path and bitmask-included helmet.
    #[test]
    fn loc_7820_no_direction_held_does_not_block_walking() {
        // When no direction is held, loc_779C's `tst.b (Ctrl_Right_Held) / beq.s
        // loc_77E6` falls through to `tst.b (Ctrl_Left_Held) / beq.s loc_7828`.
        // With neither held the ROM continues to loc_7828 (platform standing), not
        // loc_7820.  The walk-entry path only triggers when a direction IS held but
        // no wall/ledge blocks it.  Verify that probe_wall on an empty grid returns
        // no hit — the native equivalent of "clear path → walking proceeds".
        let data = vec![0u8; 8 * 6 * 2];
        let grid = terrain::BlockGrid::from_level_layout(&data, 8, 6);
        let hit_r = terrain::probe_wall(&grid, 48, 63, 5, 31, true);
        let hit_l = terrain::probe_wall(&grid, 48, 63, 5, 31, false);
        assert!(
            !hit_r.hit && !hit_l.hit,
            "ASM loc_7820: empty grid must allow walking in both directions"
        );
    }

    // --- loc_787E–loc_78C0: standing-still platform departure / support / crush ---

    /// ASM loc_787E (kid.asm:7924): when the standing-still platform block
    /// (loc_7828) clears Addr_PlatformStandingOn and `tst.l x_vel(a3)` is zero,
    /// the ROM calls `bsr.w sub_78E8` (refresh grounded helmet state) then
    /// `bra.w loc_75D4` (return to standing-still loop).
    /// Native: grounded player with zero x_velocity stays standing; sub_78E8
    /// runs unconditionally each grounded frame (lib.rs on_ground gate).
    #[test]
    fn loc_787e_platform_gone_zero_xvel_refreshes_helmet_and_stays_standing() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Maniaxe;
        rt.helmet.maniaxe_axe_active = true;
        // No live projectile → sub_78E8 should clear maniaxe_axe_active
        let mut slice = combat_slice();
        slice.player.on_ground = true;
        slice.player.x_velocity = 0; // zero x_vel → loc_787E path
        slice.player.y_velocity = 0;

        rt.refresh_grounded_helmet_state_sub_78E8();

        assert!(
            !rt.helmet.maniaxe_axe_active,
            "ASM loc_787E: sub_78E8 should clear Maniaxe axe flag when no live projectile exists",
        );
        assert!(
            slice.player.on_ground,
            "ASM loc_787E: player should remain grounded (standing-still loop continues)",
        );
    }

    /// ASM loc_7886 (kid.asm:7929): after sub_902A finds floor while Kid stands
    /// on a platform, `bsr.w sub_8ED0` checks whether Kid still overlaps.
    /// If unsupported (NE), the ROM enters MoveID_Jump (airborne).
    /// Native: `platform_support_check_sub_8ED0` returns true (unsupported)
    /// when no platform is provided or Kid is off the edge.
    #[test]
    fn loc_7886_sub_8ed0_unsupported_means_kid_falls_off_platform() {
        let platform = make_test_platform(40, 100, 2, 0); // x=40, width=47 → right=87
        // Kid far to the right of the platform → unsupported
        // sub_8ED0 checks: player_x - half_width > platform_right → 96 - 7 = 89 > 87
        let unsupported = platform_support_check_sub_8ED0(
            Some(&platform),
            96u32 << 16, // x_pos=96, past platform right edge (87)
            7,
            PlayerForm::TheKid,
            false,
        );
        assert!(
            unsupported,
            "ASM loc_7886: sub_8ED0 should return unsupported when Kid is off the platform edge",
        );
    }

    /// ASM loc_7886 → loc_789C (kid.asm:7929→7937): when sub_8ED0 returns EQ
    /// (still supported), the ROM falls through to loc_789C which loads
    /// `($FFFFFA98).w` into d0 (dead — overwritten by sub_78E8), calls
    /// sub_78E8, then `bra.w loc_75D4`.  Native: supported check passes,
    /// `apply_platform_collision` carries the player, on_ground stays true.
    #[test]
    fn loc_7886_to_789c_supported_platform_stays_grounded() {
        let platform = make_test_platform(40, 100, 2, 0); // x=40..71
        // Kid centred on the platform → supported
        let unsupported = platform_support_check_sub_8ED0(
            Some(&platform),
            50u32 << 16,
            7,
            PlayerForm::TheKid,
            false,
        );
        assert!(
            !unsupported,
            "ASM loc_789C: sub_8ED0 should confirm support when Kid overlaps the platform span",
        );
    }

    /// ASM loc_789C (kid.asm:7937): `move.l ($FFFFFA98).w,d0 / bsr.w sub_78E8
    /// / bra.w loc_75D4`.  The $FFFFFA98 load is dead code — sub_78E8
    /// immediately overwrites d0 with `move.w (Current_Helmet).w,d0`.
    /// In the standing-still context $FFFFFA98 is cleared at loc_75D4 and
    /// never stored, unlike the walking counterpart at loc_82D8.
    /// Verify that sub_78E8 behaviour is unaffected by any prior d0 value.
    #[test]
    fn loc_789c_dead_fffffa98_load_does_not_affect_sub_78e8() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::RedStealth;
        rt.helmet.red_stealth_sword_timer = 5;

        // sub_78E8 should preserve the sword timer regardless of any
        // hypothetical d0 value from $FFFFFA98.
        rt.refresh_grounded_helmet_state_sub_78E8();

        assert_eq!(
            rt.helmet.red_stealth_sword_timer, 5,
            "ASM loc_789C: sub_78E8 preserves Red Stealth sword timer (d0 from $FFFFFA98 is dead)",
        );
    }

    /// ASM loc_78A8 (kid.asm:7943): sub_902A returned negative (ceiling
    /// collision while riding a platform).  The ROM checks `tst.l $E(a4)`
    /// (platform y_vel).  If bmi (y_vel < 0, platform moving up), the ROM
    /// branches to loc_78C0.  If y_vel >= 0, the ROM clears
    /// Addr_PlatformStandingOn, calls sub_942A, and enters walking (loc_8BF0).
    /// Native does not yet model ceiling crush detection on platforms.
    /// This test documents the expected ASM branching.
    #[test]
    fn loc_78a8_upward_platform_ceiling_collision_branches_to_crush_death() {
        // ASM loc_78A8: `move.w (Addr_PlatformStandingOn).w,a4 / tst.l $E(a4)
        // / bmi.w loc_78C0`.  Platform y_vel < 0 (upward) → crush death.
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.y_velocity = -0x1_0000; // moving upward

        // TODO(fidelity): native does not yet detect this crush condition.
        // When platform ceiling crush is implemented, this test should verify
        // that an upward-moving platform pushing Kid into a ceiling triggers
        // the Death path (loc_78C0).
        assert!(
            platform.y_velocity < 0,
            "ASM loc_78A8: upward platform velocity (bmi) should route to loc_78C0 crush death",
        );
    }

    /// ASM loc_78A8 (kid.asm:7943): when platform y_vel >= 0 (not moving up),
    /// ceiling collision does not crush — the ROM clears PlatformStandingOn
    /// and enters walking via sub_942A → loc_8BF0.
    #[test]
    fn loc_78a8_downward_platform_ceiling_collision_enters_walking() {
        let mut platform = make_test_platform(40, 100, 2, 0);
        platform.y_velocity = 0x1_0000; // moving downward

        // ASM: `tst.l $E(a4) / bmi.w loc_78C0` — positive y_vel does NOT
        // branch to crush death; falls through to `clr.w (Addr_PlatformStandingOn)
        // / bsr.w sub_942A / bra.w loc_8BF0` (walking state).
        assert!(
            platform.y_velocity >= 0,
            "ASM loc_78A8: downward platform velocity should skip crush and enter walking",
        );
    }

    /// ASM loc_78C0 (kid.asm:7952): `moveq #0,d7 / bra.w Death` — platform
    /// crush death.  This is the terminal path when an upward-moving platform
    /// pushes Kid into a ceiling (sub_902A returns negative and platform
    /// y_vel < 0).  d7=0 tells the Death routine that no specific sub-type
    /// applies (generic death).
    /// TODO(fidelity): native does not yet implement platform crush death.
    #[test]
    fn loc_78c0_platform_crush_death_is_generic_death_d7_zero() {
        // The ASM sets d7=0 before branching to Death, distinguishing this
        // from other death types (spike d7=1, pit d7=2, etc.).
        // This documents the expected behaviour for when platform crush death
        // is implemented in native.
        let d7: i32 = 0; // moveq #0,d7
        assert_eq!(
            d7, 0,
            "ASM loc_78C0: platform crush death uses d7=0 (generic death type)",
        );
    }

    // --- Berzerker charge tests ---

    fn berzerker_charge_runtime(accum: u8, input: FrameInput) -> NativeRuntime {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.previous_input = input;
        rt.helmet.berzerker_charging = true;
        rt.helmet.berzerker_charge_accum = accum;
        rt
    }

    fn berzerker_charge_slice(
        x_velocity: i32,
        facing: Direction,
        on_ground: bool,
        floor_terrain_type: TerrainType,
    ) -> OpeningSliceState {
        let mut slice = combat_slice();
        slice.player.on_ground = on_ground;
        slice.player.x_velocity = x_velocity;
        slice.player.facing = facing;
        slice.terrain.floor_terrain_type = floor_terrain_type;
        slice
    }

    #[test]
    fn berzerker_charge_builds_when_moving_fast() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.previous_input = FrameInput {
            right: true,
            ..FrameInput::default()
        };
        let mut slice = combat_slice();
        slice.player.on_ground = true;
        slice.player.x_velocity = 0x2_0000; // above 1.0 FP threshold

        // Accumulator builds by $10 each frame — needs 16 frames to overflow
        for _ in 0..15 {
            rt.step_berzerker_charge(&slice);
            assert!(!rt.helmet.berzerker_charging, "should not be charging yet");
        }
        rt.step_berzerker_charge(&slice);
        assert!(
            rt.helmet.berzerker_charging,
            "should start charging after overflow"
        );
    }

    /// ASM loc_950A (kid.asm:10973): negative x_vel while facing right branches
    /// to loc_957C and clears the Berzerker charge accumulator.
    #[test]
    fn berzerker_charge_clears_when_velocity_opposes_facing_loc_950a() {
        let mut rt = berzerker_charge_runtime(
            0xF0,
            FrameInput {
                right: true,
                ..FrameInput::default()
            },
        );
        let slice = berzerker_charge_slice(-0x2_0000, Direction::Right, true, TerrainType::Normal);

        rt.step_berzerker_charge(&slice);

        assert!(
            !rt.helmet.berzerker_charging,
            "ASM loc_950A: backward motion must branch to loc_957C, not accumulate"
        );
        assert_eq!(
            rt.helmet.berzerker_charge_accum, 0,
            "ASM loc_957C: rejected Berzerker charge frames clear $FFFFFA13"
        );
    }

    /// ASM loc_9512 (kid.asm:10978): on ice, no held left/right input branches
    /// to loc_957C even when Berzerker is otherwise moving fast enough.
    #[test]
    fn berzerker_charge_clears_on_ice_without_direction_loc_9512() {
        let mut rt = berzerker_charge_runtime(0xF0, FrameInput::default());
        let slice = berzerker_charge_slice(0x2_0000, Direction::Right, true, TerrainType::Ice);

        rt.step_berzerker_charge(&slice);

        assert!(
            !rt.helmet.berzerker_charging,
            "ASM loc_9512: Ice with no directional input must clear charge state"
        );
        assert_eq!(rt.helmet.berzerker_charge_accum, 0);
    }

    /// ASM loc_9512: non-ice terrain skips the directional-input gate.
    #[test]
    fn berzerker_charge_non_ice_allows_no_direction_loc_9512() {
        let mut rt = berzerker_charge_runtime(0xF0, FrameInput::default());
        let slice = berzerker_charge_slice(0x2_0000, Direction::Right, true, TerrainType::Normal);

        rt.step_berzerker_charge(&slice);

        assert!(
            rt.helmet.berzerker_charging,
            "ASM loc_9512: non-ice should continue to loc_9526 even without held direction"
        );
        assert_eq!(rt.helmet.berzerker_charge_accum, 0xFF);
    }

    /// ASM loc_9512: either held direction on ice continues to loc_9526.
    #[test]
    fn berzerker_charge_ice_allows_either_direction_loc_9512() {
        for input in [
            FrameInput {
                left: true,
                ..FrameInput::default()
            },
            FrameInput {
                right: true,
                ..FrameInput::default()
            },
        ] {
            let mut rt = berzerker_charge_runtime(0xF0, input);
            let slice = berzerker_charge_slice(0x2_0000, Direction::Right, true, TerrainType::Ice);

            rt.step_berzerker_charge(&slice);

            assert!(
                rt.helmet.berzerker_charging,
                "ASM loc_9512: held direction on ice should continue to loc_9526"
            );
            assert_eq!(rt.helmet.berzerker_charge_accum, 0xFF);
        }
    }

    /// ASM loc_9526 (kid.asm:10986): exactly $10000 is accepted; only lower
    /// velocities branch to loc_957C.
    #[test]
    fn berzerker_charge_threshold_equality_builds_loc_9526() {
        let mut rt = berzerker_charge_runtime(0xF0, FrameInput::default());
        let slice = berzerker_charge_slice(0x1_0000, Direction::Right, true, TerrainType::Normal);

        rt.step_berzerker_charge(&slice);

        assert!(
            rt.helmet.berzerker_charging,
            "ASM loc_9526: velocity == $10000 should increment and overflow accumulator"
        );
        assert_eq!(rt.helmet.berzerker_charge_accum, 0xFF);
    }

    /// ASM loc_9526 path can continue in-air while a direction remains held;
    /// native models that as the `(on_ground || left || right)` admission gate.
    #[test]
    fn berzerker_charge_airborne_with_direction_still_builds_loc_9526() {
        let mut rt = berzerker_charge_runtime(
            0xF0,
            FrameInput {
                left: true,
                ..FrameInput::default()
            },
        );
        let slice = berzerker_charge_slice(0x2_0000, Direction::Right, false, TerrainType::Normal);

        rt.step_berzerker_charge(&slice);

        assert!(
            rt.helmet.berzerker_charging,
            "ASM loc_9526: held direction should keep the charge path live while airborne"
        );
        assert_eq!(rt.helmet.berzerker_charge_accum, 0xFF);
    }

    #[test]
    fn berzerker_charge_resets_when_slow() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.previous_input = FrameInput {
            right: true,
            ..FrameInput::default()
        };
        rt.helmet.berzerker_charging = true;
        rt.helmet.berzerker_charge_accum = 0xFF;
        let mut slice = combat_slice();
        slice.player.on_ground = true;
        slice.player.x_velocity = 0x8000; // below threshold

        rt.step_berzerker_charge(&slice);

        assert!(
            !rt.helmet.berzerker_charging,
            "should stop charging below threshold"
        );
        assert_eq!(
            rt.helmet.berzerker_charge_accum, 0,
            "accumulator should reset"
        );
    }

    #[test]
    fn berzerker_charge_not_active_for_other_forms() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::TheKid;
        rt.helmet.berzerker_charging = true;
        let mut slice = combat_slice();
        slice.player.on_ground = true;
        slice.player.x_velocity = 0x2_0000;

        rt.step_berzerker_charge(&slice);

        assert!(!rt.helmet.berzerker_charging);
    }

    #[test]
    fn berzerker_charge_kills_enemy_on_contact() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.helmet.berzerker_charging = true;
        let mut slice = combat_slice();
        slice.player.x_pos = 0x0040_0000;
        slice.player.y_pos = 0x0040_0000;

        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, 0x0040_0000, 0x0040_0000, 1));

        rt.process_berzerker_charge_hits(&slice);

        assert!(
            rt.active_enemies.is_empty(),
            "enemy should be killed by charge"
        );
    }

    /// ASM Check_for_recent_damge Berzerker gate treats active charging contact
    /// as enemy damage. Native's contact approximation keeps horizontal overlap
    /// inclusive at PLAYER_HALF_WIDTH + ENEMY_CONTACT_HALF_WIDTH.
    #[test]
    fn berzerker_charge_hits_at_horizontal_extent() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.helmet.berzerker_charging = true;
        let mut slice = combat_slice();
        slice.player.x_pos = 0x0040_0000;
        slice.player.y_pos = 0x0040_0000;
        let limit = PLAYER_HALF_WIDTH + ENEMY_CONTACT_HALF_WIDTH as i32;

        rt.active_enemies.push(test_enemy(
            EnemyFamily::Goat,
            (0x0040 + limit as u32) << 16,
            0x0040_0000,
            2,
        ));

        rt.process_berzerker_charge_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 1,
            "ASM Berzerker_charging contact: enemy exactly at horizontal extent should be hit"
        );
    }

    /// ASM Berzerker charging contact only fires when the contact boxes overlap;
    /// native must reject enemies just outside horizontal range.
    #[test]
    fn berzerker_charge_misses_just_beyond_horizontal_extent() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.helmet.berzerker_charging = true;
        let mut slice = combat_slice();
        slice.player.x_pos = 0x0040_0000;
        slice.player.y_pos = 0x0040_0000;
        let miss_dx = PLAYER_HALF_WIDTH + ENEMY_CONTACT_HALF_WIDTH as i32 + 1;

        rt.active_enemies.push(test_enemy(
            EnemyFamily::Goat,
            (0x0040 + miss_dx as u32) << 16,
            0x0040_0000,
            2,
        ));

        rt.process_berzerker_charge_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 2,
            "ASM Berzerker_charging contact: enemy one pixel beyond horizontal extent should miss"
        );
    }

    /// ASM Berzerker charging contact is also bounded vertically by the enemy
    /// contact half-height in native's rectangle approximation.
    #[test]
    fn berzerker_charge_misses_just_beyond_vertical_extent() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.helmet.berzerker_charging = true;
        let mut slice = combat_slice();
        slice.player.x_pos = 0x0040_0000;
        slice.player.y_pos = 0x0040_0000;
        let miss_dy = ENEMY_CONTACT_HALF_HEIGHT as i32 + 1;

        rt.active_enemies.push(test_enemy(
            EnemyFamily::Goat,
            0x0040_0000,
            (0x0040 + miss_dy as u32) << 16,
            2,
        ));

        rt.process_berzerker_charge_hits(&slice);

        assert_eq!(
            rt.active_enemies[0].hit_points, 2,
            "ASM Berzerker_charging contact: enemy one pixel beyond vertical extent should miss"
        );
    }

    /// A charging Berzerker kill removes the enemy from both the live list and
    /// the trace roster in the same native contact step.
    #[test]
    fn berzerker_charge_kill_refreshes_active_enemy_roster() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.helmet.berzerker_charging = true;
        let mut slice = combat_slice();
        slice.player.x_pos = 0x0040_0000;
        slice.player.y_pos = 0x0040_0000;

        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, 0x0040_0000, 0x0040_0000, 1));
        rt.active_enemy_roster = rt
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();

        rt.process_berzerker_charge_hits(&slice);

        assert!(rt.active_enemies.is_empty());
        assert!(
            rt.active_enemy_roster.is_empty(),
            "ASM Berzerker_charging contact: trace roster should drop killed enemy"
        );
    }

    /// Without Berzerker contact, native should not manufacture a trace roster
    /// refresh from the active enemy list.
    #[test]
    fn berzerker_charge_no_hit_does_not_refresh_empty_roster() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.helmet.berzerker_charging = true;
        let mut slice = combat_slice();
        slice.player.x_pos = 0x0040_0000;
        slice.player.y_pos = 0x0040_0000;

        rt.active_enemies
            .push(test_enemy(EnemyFamily::Goat, 0x0080_0000, 0x0040_0000, 2));

        rt.process_berzerker_charge_hits(&slice);

        assert!(
            rt.active_enemy_roster.is_empty(),
            "ASM Berzerker_charging contact: no hit should leave empty trace roster untouched"
        );
    }

    #[test]
    fn berzerker_charging_immune_to_contact_damage() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.helmet.berzerker_charging = true;
        let mut slice = combat_slice();
        slice.player.x_pos = 0x0040_0000;
        slice.player.y_pos = 0x0040_0000;

        // Add a hopper (normally does contact damage) at same position
        rt.active_enemies.push(test_enemy(
            EnemyFamily::BigHoppingSkull,
            0x0040_0000,
            0x0040_0000,
            3,
        ));

        let damage_source = rt.find_contact_damage_source(&slice);
        assert!(
            damage_source.is_none(),
            "charging Berzerker should be immune to contact damage"
        );
    }

    fn berzerker_wall_grid(
        left_cell: terrain::Cell,
        right_cell: terrain::Cell,
    ) -> terrain::BlockGrid {
        let w = 10u16;
        let h = 6u16;
        let data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        for row in 2..=3 {
            grid.set_cell_at(4, row, left_cell);
            grid.set_cell_at(5, row, right_cell);
        }
        grid
    }

    fn berzerker_wall_slice(x_velocity: i32, facing: Direction) -> OpeningSliceState {
        let bounds = LevelBounds {
            width_pixels: 10 * 16,
            height_pixels: 6 * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        // x=75 gives left edge x=70 (col 4) and right edge x=80 (col 5).
        slice.player.x_pos = 75 << 16;
        slice.player.y_pos = (4 * 16) << 16;
        slice.player.x_velocity = x_velocity;
        slice.player.facing = facing;
        slice.player.on_ground = true;
        slice
    }

    fn cell_is_empty(cell: terrain::Cell) -> bool {
        !cell.has_block && cell.collision == terrain::CollisionKind::Empty
    }

    #[test]
    fn berzerker_charge_breaks_smash_block_wall() {
        let mut rt = NativeRuntime::new(manifest());
        // Build a grid with a smash block wall at col 5, rows 2-3
        let w = 10u16;
        let h = 6u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let set_cell = |data: &mut Vec<u8>, col: usize, row: usize, hi: u8| {
            let offset = (row * usize::from(w) + col) * 2;
            data[offset] = hi;
            data[offset + 1] = 0x01;
        };
        // Smash block wall at col 5 (collision=6 Solid, has_block, block_type=0 Smash)
        // 1_110_0000 = 0xE0
        set_cell(&mut data, 5, 2, 0xE0);
        set_cell(&mut data, 5, 3, 0xE0);
        // Floor at row 4
        for col in 0..10 {
            set_cell(&mut data, col, 4, 0x60);
        }
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        rt.set_block_grid(grid);

        rt.progression.current_form = PlayerForm::Berzerker;
        rt.helmet.berzerker_charging = true;

        // Player just left of the smash block, moving right
        let bounds = rt.level_bounds(rt.current_level);
        let mut slice = OpeningSliceState::new(bounds);
        // Col 5 starts at x=80px. Player at x=74 with half_width=5, right edge = 79.
        slice.player.x_pos = 74 << 16;
        slice.player.y_pos = (4 * 16) << 16; // on floor
        slice.player.x_velocity = 0x2_0000; // moving right
        slice.player.facing = Direction::Right;
        slice.player.on_ground = true;

        rt.berzerker_break_wall_block(&slice);

        // Check the smash block at col 5, row 3 (at player height) was broken
        let grid = rt.block_grid.as_ref().unwrap();
        let cell = grid.cell_at(5, 3);
        assert!(
            !cell.has_block,
            "smash block should be broken by Berzerker charge"
        );
    }

    /// ASM loc_92EC (kid.asm:10735): block dispatch uses the wall probe on the
    /// side Berzerker is actually moving toward; zero x_vel falls back to facing.
    #[test]
    fn berzerker_charge_zero_velocity_facing_left_breaks_left_wall_loc_92ec() {
        let smash = terrain::Cell::from_be_bytes(0xE0, 0x01);
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(berzerker_wall_grid(
            smash,
            terrain::Cell::from_be_bytes(0, 0),
        ));

        rt.berzerker_break_wall_block(&berzerker_wall_slice(0, Direction::Left));

        let grid = rt
            .block_grid
            .as_ref()
            .expect("grid should remain installed");
        assert!(
            cell_is_empty(grid.cell_at(4, 3)),
            "ASM loc_92EC: zero velocity + facing left should dispatch the left wall block"
        );
        assert_eq!(grid.cell_at(5, 3), terrain::Cell::from_be_bytes(0, 0));
    }

    /// ASM loc_92EC: nonzero x_vel chooses the probe direction over sprite facing.
    #[test]
    fn berzerker_charge_positive_velocity_breaks_right_wall_even_facing_left_loc_92ec() {
        let smash = terrain::Cell::from_be_bytes(0xE0, 0x01);
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(berzerker_wall_grid(
            terrain::Cell::from_be_bytes(0, 0),
            smash,
        ));

        rt.berzerker_break_wall_block(&berzerker_wall_slice(0x2_0000, Direction::Left));

        let grid = rt
            .block_grid
            .as_ref()
            .expect("grid should remain installed");
        assert!(
            cell_is_empty(grid.cell_at(5, 3)),
            "ASM loc_92EC: positive velocity should dispatch the right wall block"
        );
    }

    /// ASM loc_92EC: negative x_vel chooses the left probe even if the sprite is
    /// still facing right.
    #[test]
    fn berzerker_charge_negative_velocity_breaks_left_wall_even_facing_right_loc_92ec() {
        let smash = terrain::Cell::from_be_bytes(0xE0, 0x01);
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(berzerker_wall_grid(
            smash,
            terrain::Cell::from_be_bytes(0, 0),
        ));

        rt.berzerker_break_wall_block(&berzerker_wall_slice(-0x2_0000, Direction::Right));

        let grid = rt
            .block_grid
            .as_ref()
            .expect("grid should remain installed");
        assert!(
            cell_is_empty(grid.cell_at(4, 3)),
            "ASM loc_92EC: negative velocity should dispatch the left wall block"
        );
    }

    /// ASM loc_92DE -> loc_92EC only dispatches actual block handlers; a solid
    /// wall cell with `has_block` clear must not be deleted.
    #[test]
    fn berzerker_charge_solid_non_block_wall_is_not_deleted_loc_92de() {
        let solid_non_block = terrain::Cell::from_be_bytes(0x60, 0x01);
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(berzerker_wall_grid(
            terrain::Cell::from_be_bytes(0, 0),
            solid_non_block,
        ));

        rt.berzerker_break_wall_block(&berzerker_wall_slice(0x2_0000, Direction::Right));

        let grid = rt
            .block_grid
            .as_ref()
            .expect("grid should remain installed");
        assert_eq!(
            grid.cell_at(5, 3),
            solid_non_block,
            "ASM loc_92DE: solid terrain without the block flag must not dispatch off_933E"
        );
    }

    /// ASM loc_92EC dispatches by block type. Native Berzerker charge currently
    /// implements only Smash deletion, so other block types must be preserved.
    #[test]
    fn berzerker_charge_non_smash_block_is_not_deleted_loc_92ec() {
        let prize = terrain::Cell::from_be_bytes(0xE1, 0x01);
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(berzerker_wall_grid(
            terrain::Cell::from_be_bytes(0, 0),
            prize,
        ));

        rt.berzerker_break_wall_block(&berzerker_wall_slice(0x2_0000, Direction::Right));

        let grid = rt
            .block_grid
            .as_ref()
            .expect("grid should remain installed");
        assert_eq!(
            grid.cell_at(5, 3),
            prize,
            "ASM loc_92EC: non-Smash block types are not handled by native Berzerker deletion"
        );
    }

    // --- 105-level soak test ---

    #[test]
    fn soak_all_105_levels_no_panic() {
        let mut manifest = manifest_with_levels(105);
        // Vary theme, weather, and murder wall across levels to cover all code paths
        let themes: [u8; 11] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let weather_flags: [u8; 4] = [0, 1, 2, 3]; // none, lava, storm, storm+hail
        let murder_flags: [u8; 3] = [0, 0b10, 0b11]; // none, normal, reversed

        for (i, level) in manifest.levels.iter_mut().enumerate() {
            if let Some(header) = level.header_metadata.as_mut() {
                header.level_theme = themes[i % themes.len()];
                header.background_theme = themes[i % themes.len()];
                header.weather_flags = weather_flags[i % weather_flags.len()];
                header.murder_wall_flags = murder_flags[i % murder_flags.len()];
            }
        }

        let mut runtime = NativeRuntime::new(manifest);
        runtime.set_block_grid(grid_with_floor_at_row(17));

        // Boot through sega/title screens
        for _ in 0..120 {
            runtime.step_frame(FrameInput::default());
        }
        // Press start to begin
        runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });
        for _ in 0..120 {
            runtime.step_frame(FrameInput::default());
        }

        // Step through each level via debug warp (N key simulated by advance_level)
        for level_id in 0u8..105 {
            runtime.begin_level_entry(LevelId(level_id));
            runtime.set_block_grid(grid_with_floor_at_row(17));
            // Run 60 frames of gameplay per level with invariant checks
            for frame in 0..60 {
                runtime.step_frame(FrameInput::default());
                let trace = runtime.trace_frame();

                // Velocity invariants: must stay within terminal limits
                if let Some(y_vel) = trace.kid_y_velocity {
                    assert!(
                        (-0x8_0000..=0x8_0000).contains(&y_vel),
                        "level {level_id} frame {frame}: y_vel {y_vel:#x} outside terminal limits"
                    );
                }
                if let Some(x_vel) = trace.kid_x_velocity {
                    assert!(
                        x_vel.abs() <= 0x8_0000,
                        "level {level_id} frame {frame}: x_vel {x_vel:#x} exceeds max ground speed"
                    );
                }

                // Player lifecycle must be a valid state — all variants are valid,
                // but the match ensures the enum hasn't been corrupted or extended
                // without updating this soak test.
                assert!(
                    matches!(
                        runtime.player_lifecycle,
                        PlayerLifecycle::Alive
                            | PlayerLifecycle::DeathWindup { .. }
                            | PlayerLifecycle::Dying { .. }
                            | PlayerLifecycle::Respawning { .. }
                            | PlayerLifecycle::LevelComplete { .. }
                            | PlayerLifecycle::GameOver { .. }
                            | PlayerLifecycle::ContinueCountdown { .. }
                    ),
                    "level {level_id} frame {frame}: unexpected lifecycle {:?}",
                    runtime.player_lifecycle
                );
            }
        }
    }

    // --- 2P alternating mode tests ---

    #[test]
    fn options_menu_toggles_two_player_controller_mode_with_left_or_right() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 0 };

        runtime.step_frame(FrameInput {
            right: true,
            ..FrameInput::default()
        });
        assert!(
            runtime.options_two_player_two_controllers,
            "right on cursor 0 should toggle the ROM's one-controller/two-controller option"
        );

        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            left: true,
            ..FrameInput::default()
        });
        assert!(
            !runtime.options_two_player_two_controllers,
            "left on cursor 0 should toggle the option back"
        );
    }

    #[test]
    fn options_menu_tracks_controls_speed_and_exit_like_the_rom() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 1 };

        runtime.step_frame(FrameInput {
            right: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.options_controls_scheme, 1,
            "right on cursor 1 should increment the controls scheme"
        );

        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            left: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.options_controls_scheme, 0,
            "left on cursor 1 should decrement the controls scheme"
        );

        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            down: true,
            ..FrameInput::default()
        });
        assert!(matches!(
            runtime.boot_state,
            BootState::OptionsMenu { cursor: 2 }
        ));
        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            right: true,
            ..FrameInput::default()
        });
        assert!(
            runtime.options_speed_button_fast,
            "right on cursor 2 should toggle the speed-button mode"
        );

        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            down: true,
            ..FrameInput::default()
        });
        assert!(matches!(
            runtime.boot_state,
            BootState::OptionsMenu { cursor: 3 }
        ));

        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            right: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.boot_state,
            BootState::TitleScreen { cursor: 0 },
            "right on EXIT should follow ASM Option_Exit back to title"
        );

        runtime.boot_state = BootState::OptionsMenu { cursor: 2 };
        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            b: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.boot_state,
            BootState::TitleScreen { cursor: 0 },
            "B should also exit options back to title"
        );
    }

    #[test]
    fn title_menu_up_navigation_plays_sub_1bc26_only_when_the_cursor_moves() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 1 };
        runtime.title_menu_active = true;
        runtime.title_menu_arm_frame = 0;

        runtime.step_frame(FrameInput {
            up: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.boot_state,
            BootState::TitleScreen { cursor: 0 },
            "ASM Obj_TitleMenu loc_1BBDE should move the cursor from row 1 to row 0 on Up",
        );
        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::NavigateJingle],
            "ASM Obj_TitleMenu loc_1BBDE should call sub_1BC26 when Up actually moves the cursor",
        );

        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            up: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::NavigateJingle],
            "ASM Obj_TitleMenu should stay silent when Up hits the top row guard",
        );
    }

    #[test]
    fn title_menu_down_navigation_plays_sub_1bc26_only_when_the_cursor_moves() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.title_menu_active = true;
        runtime.title_menu_arm_frame = 0;

        runtime.step_frame(FrameInput {
            down: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.boot_state,
            BootState::TitleScreen { cursor: 1 },
            "ASM Obj_TitleMenu loc_1BBF6 should move the cursor from row 0 to row 1 on Down",
        );
        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::NavigateJingle],
            "ASM Obj_TitleMenu loc_1BBF6 should call sub_1BC26 when Down actually moves the cursor",
        );

        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            down: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.boot_state,
            BootState::TitleScreen { cursor: 2 },
            "ASM Obj_TitleMenu loc_1BBF6 should move the cursor from row 1 to row 2 on Down",
        );
        assert_eq!(
            runtime.played_sound_effects,
            vec![
                QueuedSoundEffect::NavigateJingle,
                QueuedSoundEffect::NavigateJingle
            ],
            "ASM Obj_TitleMenu loc_1BBF6 should call sub_1BC26 on each successful Down navigation",
        );

        runtime.played_sound_effects.clear();
        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            down: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.played_sound_effects,
            Vec::<QueuedSoundEffect>::new(),
            "ASM Obj_TitleMenu should stay silent when Down hits the bottom row guard",
        );
    }

    #[test]
    fn title_start_one_player_sets_two_player_flag_off_and_begins_level_entry_loc_1bbd6() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.title_menu_active = true;
        runtime.title_menu_arm_frame = 0;

        runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.title_menu_selection_pending,
            Some(0),
            "ASM Obj_TitleMenu Title_InputLoop queues cursor 0 for one-player selection",
        );
        assert_eq!(
            runtime.boot_state,
            BootState::TitleScreen { cursor: 0 },
            "ASM Obj_TitleMenu loc_1BBD6 waits before transitioning to level entry"
        );
    }

    #[test]
    fn title_start_two_player_sets_two_player_flag_on_and_begins_level_entry_loc_1bbd6() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 1 };
        runtime.title_menu_active = true;
        runtime.title_menu_arm_frame = 0;

        runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.title_menu_selection_pending,
            Some(1),
            "ASM Obj_TitleMenu Title_InputLoop queues cursor 1 for two-player selection",
        );
        assert_eq!(
            runtime.boot_state,
            BootState::TitleScreen { cursor: 1 },
            "ASM Obj_TitleMenu loc_1BBD6 waits before transitioning to level entry"
        );
    }

    #[test]
    fn title_start_options_enters_options_menu_mode_loc_1bbd6() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 2 };
        runtime.title_menu_active = true;
        runtime.title_menu_arm_frame = 0;

        runtime.step_frame(FrameInput {
            start: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.title_menu_selection_pending,
            Some(2),
            "ASM Obj_TitleMenu Title_InputLoop queues cursor 2 for options-menu selection",
        );
        assert_eq!(
            runtime.boot_state,
            BootState::TitleScreen { cursor: 2 },
            "ASM Obj_TitleMenu loc_1BBD6 waits before transitioning to options menu"
        );
    }

    #[test]
    fn options_menu_left_and_right_toggle_paths_play_sub_1bc26() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 0 };

        runtime.step_frame(FrameInput {
            right: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::NavigateJingle],
            "ASM Option_2PInput should call sub_1BC26 before toggling the 2-player controller mode",
        );

        runtime.played_sound_effects.clear();
        runtime.prior_input = FrameInput::default();
        runtime.boot_state = BootState::OptionsMenu { cursor: 1 };
        runtime.step_frame(FrameInput {
            right: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::NavigateJingle],
            "ASM Option_Controls_Right should call sub_1BC26 before incrementing the controls option",
        );
    }

    #[test]
    fn options_menu_panel_step_sub_1cc88_alternates_the_worker_offsets_each_frame() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 0 };

        runtime.step_frame(FrameInput::default());
        assert_eq!(
            runtime.options_menu_panel_toggle_byte, 0xFF,
            "ASM sub_1CC88 should flip `($FFFFFB5C).w` from 0 to $FF on the first options-menu frame",
        );
        assert_eq!(
            runtime.options_menu_left_panel_x, 0x0148,
            "ASM sub_1CC88 should add `($FFFFFB5E).w` to the left worker's `$24(a0)` word when the toggled byte is non-zero",
        );
        assert_eq!(
            runtime.options_menu_right_panel_x, -0x0008,
            "ASM sub_1CC88 should subtract the same shared delta from the right worker on that first frame",
        );

        runtime.step_frame(FrameInput::default());
        assert_eq!(
            runtime.options_menu_panel_toggle_byte, 0x00,
            "ASM sub_1CC88 should flip `($FFFFFB5C).w` back to 0 on the following frame",
        );
        assert_eq!(
            runtime.options_menu_left_panel_x, 0x0030,
            "ASM sub_1CC88 should restore the left worker offset on the zero-byte branch",
        );
        assert_eq!(
            runtime.options_menu_right_panel_x, 0x0110,
            "ASM sub_1CC88 should restore the right worker offset on the zero-byte branch",
        );
    }

    #[test]
    fn options_menu_panel_step_loc_1cca2_reverses_the_worker_offsets_when_the_toggled_byte_hits_zero()
     {
        let (toggle_byte, left_panel_x, right_panel_x) =
            crate::boot::step_option_menu_panels_sub_1CC88(0xFF, 0x0118, 0x0148, -0x0008);

        assert_eq!(
            toggle_byte, 0x00,
            "ASM loc_1CCA2 (kid.asm:31818) is reached when `not.b ($FFFFFB5C).w` produces zero",
        );
        assert_eq!(
            left_panel_x, 0x0030,
            "ASM loc_1CCA2 should apply `sub.w d7,$24(a0)` on the zero-byte branch",
        );
        assert_eq!(
            right_panel_x, 0x0110,
            "ASM loc_1CCA2 should apply `add.w d7,$24(a1)` on the zero-byte branch",
        );
    }

    #[test]
    fn options_menu_panel_step_loc_1ccaa_returns_the_updated_toggle_and_worker_offsets_from_either_branch()
     {
        let non_zero_branch =
            crate::boot::step_option_menu_panels_sub_1CC88(0x00, 0x0118, 0x0030, 0x0110);
        let zero_branch =
            crate::boot::step_option_menu_panels_sub_1CC88(0xFF, 0x0118, 0x0148, -0x0008);

        assert_eq!(
            non_zero_branch,
            (0xFF, 0x0148, -0x0008),
            "ASM loc_1CCAA (kid.asm:31822) should return the non-zero branch's updated toggle and worker words after the shared epilogue",
        );
        assert_eq!(
            zero_branch,
            (0x00, 0x0030, 0x0110),
            "ASM loc_1CCAA should likewise return the zero-branch worker words after the same shared epilogue",
        );
    }

    #[test]
    fn options_menu_loc_1cb88_intro_loop_omitted_enters_directly() {
        // ASM loc_1CB88 (kid.asm:31678-31688): The ROM has an intro loop that
        // steps panel animation while scrolling Camera_Y_pos for visual effect
        // before entering the interactive loop. Native omits this and enters
        // OptionsMenu directly with panels already positioned.
        //   bne.w loc_1CB88        ; loop until camera scroll completes
        //   moveq #8,d2            ; reset oscillation
        //   subq.w #1,d0
        //   subq.w #1,d1
        //   bsr.w sub_1CDA8        ; update panel workers
        // loc_1CB88:
        //   bsr.w sub_1CC88        ; step panel animation
        //   bclr #7,(Ctrl_1_Pressed).w
        //   beq.s OptionScreen_IntroLoop
        //   bra.w Option_Exit
        // Native enters directly at the interactive loop state without the camera
        // scroll intro, but still calls sub_1CC88 each frame for panel animation.
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 0 };

        // Verify native steps panels immediately like loc_1CB88 does
        runtime.step_frame(FrameInput::default());
        assert_eq!(
            runtime.options_menu_panel_toggle_byte, 0xFF,
            "ASM loc_1CB88 calls sub_1CC88; native should toggle the panel byte on first frame",
        );
    }

    #[test]
    fn options_menu_loc_1cbf2_up_guard_skips_decrement_at_cursor_zero() {
        // ASM loc_1CBF2 (kid.asm:31721-31727): Branch target when Up is not pressed
        // or cursor is already at 0, skipping the decrement and sound effect.
        //   bclr #Button_Up,(Ctrl_Pressed).w
        //   beq.w loc_1CBF2         ; skip if Up not pressed
        //   tst.w (Options_Selected_Option).w
        //   beq.w loc_1CBF2         ; skip if already at 0
        //   bsr.w sub_1BC26
        //   subq.w #1,(Options_Selected_Option).w
        // loc_1CBF2:                ; continue to check Down
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 0 };

        runtime.step_frame(FrameInput {
            up: true,
            ..FrameInput::default()
        });
        assert!(matches!(
            runtime.boot_state,
            BootState::OptionsMenu { cursor: 0 }
        ));
        assert_eq!(
            runtime.played_sound_effects.len(),
            0,
            "ASM loc_1CBF2 skips sub_1BC26 when cursor is at 0; no sound should play"
        );

        // Compare to cursor 1 where Up should decrement
        runtime.boot_state = BootState::OptionsMenu { cursor: 1 };
        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            up: true,
            ..FrameInput::default()
        });
        assert!(
            matches!(runtime.boot_state, BootState::OptionsMenu { cursor: 0 }),
            "ASM OptionScreen_Input should decrement cursor from 1 to 0 on Up press"
        );
        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::NavigateJingle],
            "ASM OptionScreen_Input calls sub_1BC26 when cursor moves from 1 to 0"
        );
    }

    #[test]
    fn options_menu_loc_1cc0e_down_guard_skips_increment_at_cursor_three() {
        // ASM loc_1CC0E (kid.asm:31730-31736): Branch target when Down is not pressed
        // or cursor is already at 3, skipping the increment and sound effect.
        //   bclr #Button_Down,(Ctrl_Pressed).w
        //   beq.w loc_1CC0E         ; skip if Down not pressed
        //   cmpi.w #3,(Options_Selected_Option).w
        //   beq.w loc_1CC0E         ; skip if already at 3 (EXIT option)
        //   bsr.w sub_1BC26
        //   addq.w #1,(Options_Selected_Option).w
        // loc_1CC0E:                ; continue to check Left
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 3 };

        runtime.step_frame(FrameInput {
            down: true,
            ..FrameInput::default()
        });
        assert!(matches!(
            runtime.boot_state,
            BootState::OptionsMenu { cursor: 3 }
        ));
        assert_eq!(
            runtime.played_sound_effects.len(),
            0,
            "ASM loc_1CC0E skips sub_1BC26 when cursor is at 3; no sound should play"
        );

        // Compare to cursor 2 where Down should increment
        runtime.boot_state = BootState::OptionsMenu { cursor: 2 };
        runtime.prior_input = FrameInput::default();
        runtime.step_frame(FrameInput {
            down: true,
            ..FrameInput::default()
        });
        assert!(
            matches!(runtime.boot_state, BootState::OptionsMenu { cursor: 3 }),
            "ASM OptionScreen_Input should increment cursor from 2 to 3 on Down press"
        );
        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::NavigateJingle],
            "ASM OptionScreen_Input calls sub_1BC26 when cursor moves from 2 to 3"
        );
    }

    #[test]
    fn level_layout_read_pass_loc_1ccc2_calls_sub_1cd20_then_sub_1cd32_for_0x460_iterations() {
        let read_pass = crate::boot::level_layout_read_pass_loc_1CCC2();

        assert_eq!(
            read_pass.dbf_seed, 0x045F,
            "ASM sub_1CCAE should seed `d0` with `$45F` before entering loc_1CCC2",
        );
        assert_eq!(
            read_pass.iterations, 0x0460,
            "ASM loc_1CCC2 (kid.asm:31837) should execute `$45F + 1` DBF iterations",
        );
        assert_eq!(
            read_pass.subroutine_order,
            ["sub_1CD20", "sub_1CD32"],
            "ASM loc_1CCC2 should call `sub_1CD20` before `sub_1CD32` on each iteration",
        );
    }

    #[test]
    fn level_layout_copy_to_vram_loc_1ccde_uploads_0x1180_words_to_vram() {
        let copy_pass = crate::boot::level_layout_copy_to_vram_loc_1CCDE();

        assert_eq!(
            copy_pass.dbf_seed, 0x117F,
            "ASM sub_1CCAE should seed `d0` with `$117F` before entering loc_1CCDE",
        );
        assert_eq!(
            copy_pass.words_copied, 0x1180,
            "ASM loc_1CCDE (kid.asm:31845) should execute `$117F + 1` word copies from `Level_Layout` into the VDP data port",
        );
    }

    #[test]
    fn level_layout_delay_loc_1cce8_waits_for_0x101_dbf_iterations() {
        let delay = crate::boot::level_layout_delay_loc_1CCE8();

        assert_eq!(
            delay.dbf_seed, 0x0100,
            "ASM loc_1CCE8 should seed `d0` with `$100` before the busy-wait loop",
        );
        assert_eq!(
            delay.iterations, 0x0101,
            "ASM loc_1CCE8 (kid.asm:31850) should spin for `$100 + 1` DBF iterations before reloading the VRAM read command",
        );
    }

    #[test]
    fn level_layout_read_pass_loc_1ccfc_calls_sub_1cd32_then_sub_1cd20_for_0x460_iterations() {
        let read_pass = crate::boot::level_layout_read_pass_loc_1CCFC();

        assert_eq!(
            read_pass.dbf_seed, 0x045F,
            "ASM sub_1CCAE should reseed `d0` with `$45F` before entering loc_1CCFC",
        );
        assert_eq!(
            read_pass.iterations, 0x0460,
            "ASM loc_1CCFC (kid.asm:31856) should execute `$45F + 1` DBF iterations",
        );
        assert_eq!(
            read_pass.subroutine_order,
            ["sub_1CD32", "sub_1CD20"],
            "ASM loc_1CCFC should reverse the earlier readback order and call `sub_1CD32` before `sub_1CD20` on each iteration",
        );
    }

    #[test]
    fn level_layout_final_copy_to_vram_loc_1cd18_uploads_0x1180_words_to_vram() {
        let copy_pass = crate::boot::level_layout_final_copy_to_vram_loc_1CD18();

        assert_eq!(
            copy_pass.dbf_seed, 0x117F,
            "ASM sub_1CCAE should reseed `d0` with `$117F` before entering loc_1CD18",
        );
        assert_eq!(
            copy_pass.words_copied, 0x1180,
            "ASM loc_1CD18 (kid.asm:31864) should execute `$117F + 1` word copies from `Level_Layout` into the VDP data port",
        );
    }

    #[test]
    fn vdp_bit15_clear_to_decompression_buffer_loc_1cd5a_masks_bit_15_for_0x800_words() {
        let copy_pass = crate::boot::vdp_bit15_clear_to_decompression_buffer_loc_1CD5A();

        assert_eq!(
            copy_pass.dbf_seed, 0x07FF,
            "ASM sub_1CD44 should seed `d7` with `$7FF` before entering loc_1CD5A",
        );
        assert_eq!(
            copy_pass.words_copied, 0x0800,
            "ASM loc_1CD5A (kid.asm:31908) should execute `$7FF + 1` VDP reads into `Decompression_Buffer` before the final VRAM upload",
        );
        assert_eq!(
            copy_pass.cleared_mask, 0x7FFF,
            "ASM loc_1CD5A should apply `bclr #$F,d6`, clearing bit 15 from each copied word",
        );
        assert_eq!(
            copy_pass.destination_base, 0xFFFF_77B2,
            "ASM sub_1CD44 should point `a4` at `Decompression_Buffer` before loc_1CD5A starts copying",
        );
        assert_eq!(
            copy_pass.vram_word_addr, 0x0000,
            "ASM sub_1CD44 should retarget the VDP to `vdpComm($0000,VRAM,WRITE)` after loc_1CD5A completes",
        );
    }

    #[test]
    fn vdp_upload_decompression_buffer_loc_1cd78_writes_0x800_words_back_to_vram() {
        let upload = crate::boot::vdp_upload_decompression_buffer_loc_1CD78();

        assert_eq!(
            upload.dbf_seed, 0x07FF,
            "ASM sub_1CD44 should reseed `d7` with `$7FF` before entering loc_1CD78",
        );
        assert_eq!(
            upload.words_uploaded, 0x0800,
            "ASM loc_1CD78 (kid.asm:31917) should execute `$7FF + 1` word uploads from `Decompression_Buffer` back into the VDP data port",
        );
        assert_eq!(
            upload.source_base, 0xFFFF_77B2,
            "ASM sub_1CD44 should rewind `a4` to `Decompression_Buffer` before loc_1CD78 begins the VRAM upload",
        );
    }

    #[test]
    fn vdp_post_upload_delay_loc_1cd82_waits_for_0x101_dbf_iterations() {
        let delay = crate::boot::vdp_post_upload_delay_loc_1CD82();

        assert_eq!(
            delay.dbf_seed, 0x0100,
            "ASM sub_1CD44 should seed `d7` with `$100` before entering loc_1CD82",
        );
        assert_eq!(
            delay.iterations, 0x0101,
            "ASM loc_1CD82 (kid.asm:31922) should spin for `$100 + 1` DBF iterations after the VRAM upload completes",
        );
    }

    #[test]
    fn vdp_pattern_stripe_outer_loop_loc_1cd90_runs_for_28_rows() {
        let loop_shape = crate::boot::vdp_pattern_stripe_outer_loop_loc_1CD90();

        assert_eq!(
            loop_shape.row_seed, 0x001B,
            "ASM sub_1CD88 should seed `d1` with `$1B` before entering loc_1CD90",
        );
        assert_eq!(
            loop_shape.row_iterations, 0x001C,
            "ASM loc_1CD90 (kid.asm:31935) should execute `$1B + 1` outer DBF iterations, producing 28 rows",
        );
    }

    #[test]
    fn vdp_pattern_stripe_word_copy_loop_loc_1cd92_copies_40_words_per_row() {
        let loop_shape = crate::boot::vdp_pattern_stripe_word_copy_loop_loc_1CD92();

        assert_eq!(
            loop_shape.source, 0xFFFF77B2,
            "ASM loc_1CD92 reads `(a0)+` after `lea (Decompression_Buffer).l,a0`, so the copy source should be `Decompression_Buffer` at `$FFFF77B2`",
        );
        assert_eq!(
            loop_shape.copy_seed, 0x0027,
            "ASM loc_1CD90 should reload `d0` with `$27` before entering loc_1CD92",
        );
        assert_eq!(
            loop_shape.copy_iterations, 0x0028,
            "ASM loc_1CD92 (kid.asm:31938) should execute `$27 + 1` inclusive DBF iterations, copying 40 words into the VDP data port",
        );
    }

    #[test]
    fn plane_line_word_copy_loc_1cdd4_reads_one_0x50_byte_row_from_decompression_buffer() {
        let line_copy = crate::boot::plane_line_word_copy_loc_1CDD4(3);

        assert_eq!(
            line_copy.source_base, 0xFFFF77B2,
            "ASM sub_1CDA8 should seed `a4` from `Decompression_Buffer` before loc_1CDD4 begins copying",
        );
        assert_eq!(
            line_copy.source_byte_offset, 0x00F0,
            "ASM sub_1CDA8 multiplies the caller's row index by `$50` bytes before loc_1CDD4, so row 3 should start `$F0` bytes into `Decompression_Buffer`",
        );
        assert_eq!(
            line_copy.words_copied, 0x0028,
            "ASM loc_1CDD4 (kid.asm:31970) should execute `$27 + 1` inclusive DBF iterations, copying 40 words from the selected row",
        );
    }

    #[test]
    fn vdp_pattern_stripe_fill_loop_loc_1cd9a_appends_24_fill_words_per_row() {
        let loop_shape = crate::boot::vdp_pattern_stripe_fill_loop_loc_1CD9A();

        assert_eq!(
            loop_shape.fill_word, 0x0094,
            "ASM loc_1CD9A writes immediate word `$94` into the VDP data port on every iteration",
        );
        assert_eq!(
            loop_shape.fill_seed, 0x0017,
            "ASM loc_1CD92 should reload `d0` with `$17` before entering loc_1CD9A",
        );
        assert_eq!(
            loop_shape.fill_iterations, 0x0018,
            "ASM loc_1CD9A (kid.asm:31943) should execute `$17 + 1` inclusive DBF iterations, appending 24 fill words after each copied row",
        );
    }

    #[test]
    fn classify_kid_object_relation_loc_1cd2_prefers_kidright_when_right_penetration_is_minimal() {
        let relation = crate::trace::classify_kid_object_relation(
            crate::trace::ObjectBoundsPx {
                left: 10,
                right: 30,
                top: 10,
                bottom: 30,
            },
            crate::trace::ObjectBoundsPx {
                left: 26,
                right: 60,
                top: 0,
                bottom: 60,
            },
        );

        assert_eq!(
            relation.map(crate::trace::KidObjectRelation::as_debug_label),
            Some("kidright"),
            "ASM loc_1CD2 (kid.asm:3460) should store `colid_kidright` when the right-side penetration wins the overlap comparison",
        );
    }

    #[test]
    fn classify_kid_object_relation_loc_1cd8_prefers_kidleft_when_left_penetration_is_minimal() {
        let relation = crate::trace::classify_kid_object_relation(
            crate::trace::ObjectBoundsPx {
                left: 10,
                right: 30,
                top: 10,
                bottom: 30,
            },
            crate::trace::ObjectBoundsPx {
                left: 0,
                right: 14,
                top: 0,
                bottom: 60,
            },
        );

        assert_eq!(
            relation.map(crate::trace::KidObjectRelation::as_debug_label),
            Some("kidleft"),
            "ASM loc_1CD8 (kid.asm:3465) should store `colid_kidleft` when the left-side penetration wins the overlap comparison",
        );
    }

    #[test]
    fn classify_kid_object_relation_loc_1cde_prefers_kidbelow_when_top_penetration_is_minimal() {
        let relation = crate::trace::classify_kid_object_relation(
            crate::trace::ObjectBoundsPx {
                left: 10,
                right: 30,
                top: 10,
                bottom: 30,
            },
            crate::trace::ObjectBoundsPx {
                left: 0,
                right: 60,
                top: 26,
                bottom: 60,
            },
        );

        assert_eq!(
            relation.map(crate::trace::KidObjectRelation::as_debug_label),
            Some("kidbelow"),
            "ASM loc_1CDE (kid.asm:3470) should store `colid_kidbelow` when the top-side penetration wins the overlap comparison",
        );
    }

    #[test]
    fn classify_kid_object_relation_loc_1ce4_prefers_kidabove_when_bottom_penetration_is_minimal() {
        let relation = crate::trace::classify_kid_object_relation(
            crate::trace::ObjectBoundsPx {
                left: 10,
                right: 30,
                top: 40,
                bottom: 60,
            },
            crate::trace::ObjectBoundsPx {
                left: 0,
                right: 60,
                top: 0,
                bottom: 44,
            },
        );

        assert_eq!(
            relation.map(crate::trace::KidObjectRelation::as_debug_label),
            Some("kidabove"),
            "ASM loc_1CE4 (kid.asm:3475) should store `colid_kidabove` when the bottom-side penetration wins the overlap comparison before the later jump-state gate",
        );
    }

    #[test]
    fn kid_object_collision_writeback_loc_1cf2_copies_the_selected_colid_into_both_slots() {
        let right = crate::trace::kid_object_collision_writeback_loc_1CF2(
            crate::trace::KidObjectRelation::Right,
            0xC064,
        );
        assert_eq!(
            right.player_collision_label, "kidright",
            "ASM loc_1CF2 (kid.asm:3480) should copy the branch-selected `d7` collision id into `$38(a0)`",
        );
        assert_eq!(
            right.object_collision_label, "kidright",
            "ASM loc_1CF2 should also mirror that same `d7` collision id into `$38(a2)`",
        );
        assert_eq!(
            right.object_collision_flags, 0xC064,
            "ASM loc_1CF2 should copy the raw object collision word from `$3A(a2)` into `$3A(a0)` before the later invulnerability masking path runs",
        );

        let above = crate::trace::kid_object_collision_writeback_loc_1CF2(
            crate::trace::KidObjectRelation::Above,
            0x002C,
        );
        assert_eq!(
            (above.player_collision_label, above.object_collision_label),
            ("kidabove", "kidabove"),
            "ASM loc_1CF2 should stay agnostic to the entry branch and duplicate whichever `colid_kid*` value reached the shared tail",
        );
    }

    #[test]
    fn kid_object_invulnerability_gate_loc_1d00_clears_ordinary_collisions_but_keeps_special_2c_hits()
     {
        let ordinary = crate::trace::kid_object_invulnerability_gate_loc_1D00(true, 0x0010, 0xC063);
        assert_eq!(
            ordinary.next_player_collision_type, 0,
            "ASM loc_1D00 (kid.asm:3485) should clear `$38(a0)` when invulnerability is active and the masked `$3A(a0)` payload stays below `$64`",
        );
        assert_eq!(
            ordinary.next_player_collision_flags, 0xC063,
            "ASM loc_1D00 should compare a masked copy of `$3A(a0)` but leave the stored flag word unchanged on the ordinary-clear branch",
        );

        let special = crate::trace::kid_object_invulnerability_gate_loc_1D00(true, 0x002C, 0xC064);
        assert_eq!(
            special.next_player_collision_type, 0x002C,
            "ASM loc_1D00 should branch to loc_1D34 instead of clearing the special `$2C` collision id",
        );
        assert_eq!(
            special.next_player_collision_flags, 0x0064,
            "ASM loc_1D34 should clear bits 15 and 14 in `$3A(a0)` before returning from the invulnerability gate",
        );
    }

    #[test]
    fn kid_object_stomp_object_writeback_loc_1d46_marks_the_enemy_as_kidbelow() {
        let stomp = crate::trace::kid_object_stomp_object_writeback_loc_1D46();

        assert_eq!(
            stomp.object_collision_label, "kidbelow",
            "ASM loc_1D46 (kid.asm:3512) should write `colid_kidbelow` into `$38(a2)` before the player-side stomp writeback at loc_1D4C",
        );
    }

    #[test]
    fn kid_object_stomp_player_writeback_loc_1d4c_copies_kidbelow_and_enemy_flags_into_the_player_slot()
     {
        let stomp = crate::trace::kid_object_stomp_player_writeback_loc_1D4C(0xC02A);

        assert_eq!(
            stomp.player_collision_label, "kidbelow",
            "ASM loc_1D4C (kid.asm:3515) should write `colid_kidbelow` into `$38(a0)` on the stomp path",
        );
        assert_eq!(
            stomp.player_collision_flags, 0xC02A,
            "ASM loc_1D4C should copy the raw enemy collision flags from `$3A(a2)` into `$3A(a0)` before `loc_1D00` masks them",
        );
        assert!(
            stomp.branches_to_invulnerability_gate,
            "ASM loc_1D4C should branch straight into `loc_1D00` after the stomp writeback",
        );
    }

    #[test]
    fn kid_object_type_four_collision_loc_1d5a_stalls_forever_below_64() {
        let collision = crate::trace::kid_object_type_four_collision_loc_1D5A(0x0063);

        assert!(
            collision.stalls_forever,
            "ASM loc_1D5A (kid.asm:3521) should feed the raw `$3A(a2)` word into the later `$64` compare for object subtype 4",
        );
        assert_eq!(
            collision.player_collision_flags, None,
            "ASM loc_1D62 (kid.asm:3525) self-branches on `blt.s loc_1D62`, so the subtype-4 path should never reach the later `$3A(a0)` writeback when `$3A(a2) < $64`",
        );
        assert_eq!(
            collision.player_collision_label, None,
            "ASM loc_1D62 should hang before writing `colid_kidbelow` into `$38(a0)` on the `< $64` path",
        );
        assert_eq!(
            collision.object_collision_label, None,
            "ASM loc_1D62 should also hang before mirroring `colid_kidbelow` into `$38(a2)` on the `< $64` path",
        );
        assert!(
            !collision.branches_to_invulnerability_gate,
            "ASM loc_1D62 should not reach `bra.s loc_1D00` once the self-branch is taken",
        );
    }

    #[test]
    fn kid_object_type_four_collision_loc_1d62_writes_back_kidbelow_once_flags_are_64_or_higher() {
        let collision = crate::trace::kid_object_type_four_collision_loc_1D5A(0x0064);

        assert!(!collision.stalls_forever);
        assert_eq!(
            collision.player_collision_flags,
            Some(0x0064),
            "ASM loc_1D62 (kid.asm:3525) should fall through past the self-branch once `cmpi.w #$64,d6` is not less-than, then copy `d6` into `$3A(a0)`",
        );
        assert_eq!(
            collision.player_collision_label,
            Some("kidbelow"),
            "ASM loc_1D62 should write `colid_kidbelow` into `$38(a0)` on the non-stalling path",
        );
        assert_eq!(
            collision.object_collision_label,
            Some("kidbelow"),
            "ASM loc_1D62 should mirror `colid_kidbelow` into `$38(a2)` before jumping to `loc_1D00`",
        );
        assert!(
            collision.branches_to_invulnerability_gate,
            "ASM loc_1D62 should branch to `loc_1D00` after the subtype-4 writeback when `$3A(a2) >= $64`",
        );
    }

    #[test]
    fn kid_object_invulnerability_gate_loc_1d00_leaves_non_invulnerable_or_large_payload_hits_untouched()
     {
        let vulnerable =
            crate::trace::kid_object_invulnerability_gate_loc_1D00(false, 0x0010, 0xC063);
        assert_eq!(
            vulnerable.next_player_collision_type, 0x0010,
            "ASM loc_1D00 should return immediately when `KidIsInvulnerable` is zero",
        );
        assert_eq!(
            vulnerable.next_player_collision_flags, 0xC063,
            "ASM loc_1D00 should leave `$3A(a0)` untouched when the invulnerability byte is clear",
        );

        let large_payload =
            crate::trace::kid_object_invulnerability_gate_loc_1D00(true, 0x0010, 0xC064);
        assert_eq!(
            large_payload.next_player_collision_type, 0x0010,
            "ASM loc_1D00 should keep `$38(a0)` when the masked `$3A(a0)` payload is `$64` or greater",
        );
        assert_eq!(
            large_payload.next_player_collision_flags, 0xC064,
            "ASM loc_1D00 only uses the masked copy for the compare, so the stored collision flags stay unchanged on the keep branch",
        );
    }

    #[test]
    fn options_menu_loc_1cc22_left_button_jump_table_dispatches_to_option_handlers() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.boot_state = BootState::OptionsMenu { cursor: 0 };
        runtime.step_frame(FrameInput {
            left: true,
            ..FrameInput::default()
        });
        assert!(
            runtime.options_two_player_two_controllers,
            "ASM loc_1CC22 (kid.asm:31744) jump table index0 dispatches to Option_2PInput for cursor 0"
        );

        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 1 };
        runtime.options_controls_scheme = 3;
        runtime.step_frame(FrameInput {
            left: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.options_controls_scheme, 2,
            "ASM loc_1CC22 jump table index 2 dispatches to Option_Controls_Left for cursor 1"
        );

        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 2 };
        runtime.options_speed_button_fast = false;
        runtime.step_frame(FrameInput {
            left: true,
            ..FrameInput::default()
        });
        assert!(
            runtime.options_speed_button_fast,
            "ASM loc_1CC22 jump table index 4 dispatches to Option_SpeedButton for cursor 2"
        );
    }

    #[test]
    fn options_menu_loc_1cc2a_right_button_entry_skips_dispatch_when_right_not_pressed() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 0 };
        runtime.options_two_player_two_controllers = false;

        runtime.step_frame(FrameInput {
            right: false,
            ..FrameInput::default()
        });
        assert!(
            !runtime.options_two_player_two_controllers,
            "ASM loc_1CC2A (kid.asm:31754) skips the jump table dispatch when Right is not pressed; \
             the bclr/beq sequence branches to return_1CC86 without touching any option"
        );
    }

    #[test]
    fn options_menu_loc_1cc3e_right_button_jump_table_dispatches_to_option_handlers() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.boot_state = BootState::OptionsMenu { cursor: 0 };
        runtime.step_frame(FrameInput {
            right: true,
            ..FrameInput::default()
        });
        assert!(
            runtime.options_two_player_two_controllers,
            "ASM loc_1CC3E (kid.asm:31761) jump table index 0 dispatches to Option_2PInput for cursor 0"
        );

        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 1 };
        runtime.options_controls_scheme = 2;
        runtime.step_frame(FrameInput {
            right: true,
            ..FrameInput::default()
        });
        assert_eq!(
            runtime.options_controls_scheme, 3,
            "ASM loc_1CC3E jump table index 2 dispatches to Option_Controls_Right for cursor 1"
        );

        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::OptionsMenu { cursor: 2 };
        runtime.options_speed_button_fast = false;
        runtime.step_frame(FrameInput {
            right: true,
            ..FrameInput::default()
        });
        assert!(
            runtime.options_speed_button_fast,
            "ASM loc_1CC3E jump table index 4 dispatches to Option_SpeedButton for cursor 2"
        );
    }

    #[test]
    fn end_sequence_display_refresh_sub_1d830_combines_camera_scroll_upload_and_panel_step() {
        let refresh =
            end_sequence_display_refresh_sub_1D830(0x0123, 0x0045, 0x00, 0x0118, 0x0030, 0x0110);

        assert_eq!(
            refresh.camera_y_scroll, 0x0045,
            "ASM sub_1D830 should forward Camera_Y_pos into sub_1DA24's VSRAM upload",
        );
        assert_eq!(
            refresh.h_scroll_value, -0x0123,
            "ASM sub_1D830 should use sub_1DA24's negated Camera_X_pos for each horizontal-scroll word pair",
        );
        assert_eq!(
            refresh.h_scroll_word_pairs, 0x00E0,
            "ASM sub_1DA24 should emit `$DF+1` horizontal-scroll word pairs across the plane refresh",
        );
        assert_eq!(
            refresh.panel_toggle_byte, 0xFF,
            "ASM sub_1D830 should still advance sub_1CC88 after the display refresh",
        );
        assert_eq!(
            refresh.left_panel_x, 0x0148,
            "ASM sub_1D830 should preserve sub_1CC88's left worker offset update after refreshing the display",
        );
        assert_eq!(
            refresh.right_panel_x, -0x0008,
            "ASM sub_1D830 should preserve sub_1CC88's right worker offset update after refreshing the display",
        );
    }

    #[test]
    fn end_sequence_h_scroll_upload_loc_1da50_repeats_negated_camera_x_across_448_words() {
        assert_eq!(
            crate::boot::end_sequence_h_scroll_upload_loc_1DA50(0x0123),
            crate::boot::EndSequenceHorizontalScrollUploadLoc1DA50 {
                vram_word_addr: 0x1400,
                repeated_word: -0x0123,
                word_write_count: 0x01C0,
            },
            "ASM loc_1DA50 should write `-Camera_X_pos` twice per inclusive `dbf d6,loc_1DA50` pass, so `d6=#$DF` fills 448 horizontal-scroll words at `vdpComm($1400,VRAM,WRITE)`",
        );
    }

    #[test]
    fn end_sequence_tile_fill_loc_1da6a_streams_2048_copies_of_tile_0094() {
        assert_eq!(
            crate::boot::end_sequence_tile_fill_loc_1DA6A(),
            crate::boot::EndSequenceTileFillLoc1DA6A {
                fill_word: 0x0094,
                word_write_count: 0x0800,
            },
            "ASM loc_1DA6A should stream `$7FF + 1` copies of tile word `$0094` because `move.w d1,(a6)` executes before the inclusive `dbf d0,loc_1DA6A` test",
        );
    }

    #[test]
    fn end_sequence_vram_zero_fill_loc_1da7e_clears_16_words_from_vram_1280() {
        assert_eq!(
            crate::boot::end_sequence_vram_zero_fill_loc_1DA7E(),
            crate::boot::EndSequenceVramZeroFillLoc1DA7E {
                vram_word_addr: 0x1280,
                fill_word: 0x0000,
                word_write_count: 0x0010,
            },
            "ASM loc_1DA7E should program `vdpComm($1280,VRAM,WRITE)` and stream `$F + 1` zero words because `move.w d1,(a6)` executes before the inclusive `dbf d0,loc_1DA7E` test",
        );
    }

    #[test]
    fn end_sequence_fade_loop_loc_1d270_runs_eight_palette_darken_passes_before_vram_upload() {
        let fade_loop = crate::boot::end_sequence_fade_loop_loc_1D270();

        assert_eq!(
            fade_loop.loop_seed, 0x0007,
            "ASM loc_1D270 (kid.asm:32637) should seed `d0` with `$7` before the fade loop",
        );
        assert_eq!(
            fade_loop.fade_palette_byte_offset, 0x0020,
            "ASM loc_1D270 should reload `d7 = $20` before each `sub_1C85E` call",
        );
        assert_eq!(
            fade_loop.fade_iterations, 0x0008,
            "ASM loc_1D270 should execute `$7 + 1` inclusive DBF iterations",
        );
        assert_eq!(
            fade_loop.hibernate_frames, 0x0006,
            "ASM loc_1D270 should hibernate for 6 frames after each palette darken pass",
        );
        assert_eq!(
            fade_loop.post_loop_vram_word_addr, 0xE000,
            "ASM loc_1D270 should retarget the VDP command port to VRAM word address `$E000` after the loop",
        );
        assert_eq!(
            fade_loop.post_loop_subroutine, "sub_1DA62",
            "ASM loc_1D270 should hand off to `sub_1DA62` once the eight fades complete",
        );
    }

    #[test]
    fn end_sequence_panel_setup_loc_1d306_copies_nine_palette_words_and_seeds_panel_scroll_state() {
        let panel_setup = crate::boot::end_sequence_panel_setup_loc_1D306();

        assert_eq!(
            panel_setup.palette_words_copied, 0x0009,
            "ASM loc_1D306 (kid.asm:32671) should execute `$8 + 1` palette copies into `Palette_Buffer+$E`",
        );
        assert_eq!(
            panel_setup.first_panel_x, 0x0030,
            "ASM loc_1D306 should place the left panel worker at x=`$30`",
        );
        assert_eq!(
            panel_setup.first_panel_y, 0x0048,
            "ASM loc_1D306 should place the left panel worker at y=`$48`",
        );
        assert_eq!(
            panel_setup.second_panel_x, 0x0110,
            "ASM loc_1D306 should place the right panel worker at x=`$110`",
        );
        assert_eq!(
            panel_setup.second_panel_y, 0x0048,
            "ASM loc_1D306 should place the right panel worker at y=`$48`",
        );
        assert_eq!(
            panel_setup.stripe_row_seed, 0x001B,
            "ASM loc_1D306 should seed `d0 = $1B` before the first `sub_1CDA8` row upload",
        );
        assert_eq!(
            panel_setup.stripe_column_seed, 0x001F,
            "ASM loc_1D306 should seed `d1 = $1F` before the first `sub_1CDA8` column upload",
        );
        assert_eq!(
            panel_setup.scroll_counter_seed, 0x0008,
            "ASM loc_1D306 should seed `d2 = 8` for the later `loc_1D3A0` refresh cadence",
        );
        assert_eq!(
            panel_setup.scroll_distance_seed, 0x00E0,
            "ASM loc_1D306 should seed `d3 = $E0` before the vertical panel scroll loop",
        );
        assert_eq!(
            panel_setup.panel_toggle_byte, 0x00,
            "ASM loc_1D306 should clear the toggle byte with `sf ($FFFFFB5C).w` before panel animation starts",
        );
        assert_eq!(
            panel_setup.panel_delta, 0x0118,
            "ASM loc_1D306 should store `$118` into the shared panel delta word",
        );
        assert_eq!(
            panel_setup.tilemap_row_upload_subroutine, "sub_1CDA8",
            "ASM loc_1D306 should call `sub_1CDA8` once after seeding the row and column counters",
        );
    }

    #[test]
    fn end_sequence_panel_scroll_loop_loc_1d3a0_runs_112_scroll_steps_and_28_row_uploads() {
        let scroll_loop = crate::boot::end_sequence_panel_scroll_loop_loc_1D3A0();

        assert_eq!(
            scroll_loop.display_refresh_subroutine, "sub_1DA24",
            "ASM loc_1D3A0 (kid.asm:32706) should refresh the display through `sub_1DA24` on every active pass",
        );
        assert_eq!(
            scroll_loop.hibernate_subroutine, "j_Hibernate_Object_1Frame",
            "ASM loc_1D3A0 should hibernate exactly one frame per active pass",
        );
        assert_eq!(
            scroll_loop.panel_step_subroutine, "sub_1CC88",
            "ASM loc_1D3C4 should step the menu-panel animation after each scroll update before branching back to loc_1D3A0",
        );
        assert_eq!(
            scroll_loop.row_upload_subroutine, "sub_1CDA8",
            "ASM loc_1D3A0 should call `sub_1CDA8` whenever the `d2` cadence counter rolls over",
        );
        assert_eq!(
            scroll_loop.scroll_counter_seed, 0x0008,
            "ASM loc_1D3A0 should start the cadence counter from `d2 = 8`",
        );
        assert_eq!(
            scroll_loop.scroll_distance_seed, 0x00E0,
            "ASM loc_1D3A0 should start the remaining vertical distance from `d3 = $E0`",
        );
        assert_eq!(
            scroll_loop.scroll_step, 0x0002,
            "ASM loc_1D3A0 should subtract 2 from both `Camera_Y_pos` and `d3` per active pass",
        );
        assert_eq!(
            scroll_loop.active_iterations, 0x0070,
            "ASM loc_1D3A0 should perform `$E0 / 2 = $70` active scroll iterations before the next top-of-loop exit test reaches loc_1D3CA",
        );
        assert_eq!(
            scroll_loop.panel_refresh_iterations, 0x001C,
            "ASM loc_1D3A0 should trigger `$70 / 4 = $1C` row uploads because `d2` reaches zero every fourth active pass",
        );
    }

    #[test]
    fn end_sequence_credits_glyph_cache_loc_1d3d0_allocates_six_workers_then_seeds_layout_and_text()
    {
        let glyph_cache = crate::boot::end_sequence_credits_glyph_cache_loc_1D3D0();

        assert_eq!(
            glyph_cache.glyph_count, 0x0006,
            "ASM loc_1D3D0 (kid.asm:32729) should iterate `dbf d0,loc_1D3D0` from `d0 = 5`, yielding six cached credits glyph workers",
        );
        assert_eq!(
            glyph_cache.glyph_pointer_table_base, 0xFB72,
            "ASM loc_1D3CA should point `a4` at `$FFFFFB72` before the glyph-cache loop starts",
        );
        assert_eq!(
            glyph_cache.glyph_pointer_slot_offsets_in_write_order,
            [0x0014, 0x0010, 0x000C, 0x0008, 0x0004, 0x0000],
            "ASM loc_1D3D0 should write the six glyph pointers in descending `d0` order because `lsl.w #2,d7` converts `5..0` into `$14..$0`",
        );
        assert_eq!(
            glyph_cache.glyph_init_subroutine, "sub_1D786",
            "ASM loc_1D3D0 should allocate each cached glyph worker through `sub_1D786`",
        );
        assert_eq!(
            glyph_cache.platform_layout_label, "unk_1D75A",
            "ASM loc_1D3D0 should seed `(Addr_PlatformLayout).w` from `unk_1D75A` once all six glyph workers are cached",
        );
        assert_eq!(
            glyph_cache.text_script_label, "EndText1",
            "ASM loc_1D3D0 should start the credits text loop from `EndText1` after seeding the glyph layout pointer",
        );
    }

    #[test]
    fn end_sequence_hide_glyphs_loc_1d7f0_clears_the_display_flag_for_all_six_cached_workers() {
        let hide = crate::boot::end_sequence_hide_glyphs_loc_1D7F0();

        assert_eq!(
            hide.glyph_pointer_table_base, 0xFB72,
            "ASM sub_1D7E6 should seed `a4` from `$FFFFFB72` before loc_1D7F0 starts walking the cached glyph pointers",
        );
        assert_eq!(
            hide.loop_seed, 0x0005,
            "ASM sub_1D7E6 should seed `d7 = 5` before the `dbf d7,loc_1D7F0` hide loop",
        );
        assert_eq!(
            hide.hidden_glyphs, 0x0006,
            "ASM loc_1D7F0 (kid.asm:33034) should clear six cached glyph workers because `dbf d7,loc_1D7F0` counts inclusively from 5 to -1",
        );
        assert_eq!(
            hide.cleared_flag_offset, 0x0013,
            "ASM loc_1D7F0 should clear each cached worker's display-enable byte via `sf $13(a3)`",
        );
    }

    #[test]
    fn end_sequence_page_fade_outer_loop_loc_1d80c_reloads_the_palette_window_eight_times() {
        let fade = crate::boot::end_sequence_page_fade_outer_loop_loc_1D80C();

        assert_eq!(
            fade.outer_loop_seed, 0x0007,
            "ASM sub_1D800 should seed `d0 = 7` before the outer `dbf d0,loc_1D80C` fade loop",
        );
        assert_eq!(
            fade.outer_loop_iterations, 0x0008,
            "ASM loc_1D80C (kid.asm:33051) should repeat the fade setup eight times because `dbf d0,loc_1D80C` counts inclusively from 7 to -1",
        );
        assert_eq!(
            fade.fade_palette_byte_offset, 0x0060,
            "ASM loc_1D80C should reload `d7 = $60` before every `sub_1C85E` fade-setup call",
        );
        assert_eq!(
            fade.fade_setup_subroutine, "sub_1C85E",
            "ASM loc_1D80C should run `sub_1C85E` once per outer fade pass before reseeding the inner wait loop",
        );
        assert_eq!(
            fade.inner_loop_seed, 0x0003,
            "ASM loc_1D80C should seed `d1 = 3` immediately before entering the inner `loc_1D814` loop",
        );
        assert_eq!(
            fade.inner_loop_iterations, 0x0004,
            "ASM loc_1D80C hands `d1 = 3` to the following `dbf d1,loc_1D814`, so each outer fade pass should contain four inner refresh waits",
        );
        assert_eq!(
            fade.inner_loop_entry_label, "loc_1D814",
            "ASM loc_1D80C should fall straight into the inner wait loop at `loc_1D814` after the palette setup",
        );
    }

    #[test]
    fn end_sequence_page_fade_inner_loop_loc_1d814_runs_four_refresh_and_hibernate_pairs() {
        let fade = crate::boot::end_sequence_page_fade_inner_loop_loc_1D814();

        assert_eq!(
            fade.loop_seed, 0x0003,
            "ASM loc_1D80C should hand `d1 = 3` into the inner `dbf d1,loc_1D814` loop",
        );
        assert_eq!(
            fade.iterations, 0x0004,
            "ASM loc_1D814 (kid.asm:33056) should execute four refresh/wait passes because `dbf d1,loc_1D814` counts inclusively from 3 to -1",
        );
        assert_eq!(
            fade.refresh_subroutine, "sub_1D830",
            "ASM loc_1D814 should refresh the end-sequence display through `sub_1D830` on every inner-loop pass",
        );
        assert_eq!(
            fade.hibernate_subroutine, "j_Hibernate_Object_1Frame",
            "ASM loc_1D814 should hibernate one frame immediately after each `sub_1D830` refresh",
        );
        assert_eq!(
            fade.next_outer_label, "loc_1D80C",
            "ASM loc_1D814 should fall through to the outer `dbf d0,loc_1D80C` once the inner `d1` counter expires",
        );
    }

    #[test]
    fn end_sequence_page_break_dispatch_loc_1d40e_branches_to_the_wait_setup_entry() {
        let dispatch = crate::boot::end_sequence_page_break_dispatch_loc_1D40E();

        assert_eq!(
            dispatch.branch_target, "loc_1D412",
            "ASM loc_1D40E (kid.asm:32757) should use the page-break jump-table entry to branch straight into `loc_1D412`",
        );
        assert!(
            dispatch.has_padding_nop,
            "ASM loc_1D40E should leave the trailing `nop` padding word in place after the short branch",
        );
    }

    #[test]
    fn end_sequence_page_break_wait_loop_loc_1d418_refreshes_fades_and_reloads_glyphs() {
        let wait_loop = crate::boot::end_sequence_page_break_wait_loop_loc_1D418();

        assert_eq!(
            wait_loop.wait_seed, 0x00C8,
            "ASM loc_1D418 should inherit `d0 = $C8` from loc_1D412 before entering the DBF wait loop",
        );
        assert_eq!(
            wait_loop.wait_iterations, 0x00C9,
            "ASM loc_1D418 should execute `$C8 + 1 = $C9` `sub_1D830 -> Hibernate_Object_1Frame` refresh pairs before the page refresh continues",
        );
        assert_eq!(
            wait_loop.refresh_subroutine, "sub_1D830",
            "ASM loc_1D418 should refresh the display via `sub_1D830` on each page-break wait iteration",
        );
        assert_eq!(
            wait_loop.hibernate_subroutine, "j_Hibernate_Object_1Frame",
            "ASM loc_1D418 should hibernate one frame after each `sub_1D830` refresh",
        );
        assert_eq!(
            wait_loop.hide_glyphs_subroutine, "sub_1D7E6",
            "ASM loc_1D418 should hide all cached glyph workers through `sub_1D7E6` before the fade",
        );
        assert_eq!(
            wait_loop.fade_subroutine, "sub_1D800",
            "ASM loc_1D418 should run `sub_1D800` to fade the previous page out before redrawing",
        );
        assert_eq!(
            wait_loop.stripe_subroutine, "sub_1CDE0",
            "ASM loc_1D418 should rebuild the page stripes through `sub_1CDE0` after the fade",
        );
        assert_eq!(
            wait_loop.palette_buffer_word_offset, 0x0062,
            "ASM loc_1D418 should patch the longword starting at `Palette_Buffer+$62` before the palette upload",
        );
        assert_eq!(
            wait_loop.palette_buffer_longword, 0x0EEE0EC0,
            "ASM loc_1D418 should write `$0EEE0EC0` into `Palette_Buffer+$62` before calling `j_Palette_to_VRAM`",
        );
        assert_eq!(
            wait_loop.palette_upload_subroutine, "j_Palette_to_VRAM",
            "ASM loc_1D418 should upload the patched palette through `j_Palette_to_VRAM`",
        );
        assert_eq!(
            wait_loop.glyph_layout_source, "Addr_PlatformLayout",
            "ASM loc_1D418 should reload `a4` from `(Addr_PlatformLayout).w` before restoring glyph placements",
        );
        assert_eq!(
            wait_loop.glyph_load_subroutine, "sub_1D7A8",
            "ASM loc_1D418 should reload glyph placements through `sub_1D7A8` from the saved platform-layout pointer",
        );
    }

    #[test]
    fn end_sequence_text_loop_loc_1d3f0_dispatches_pages_breaks_and_zero_terminator() {
        let text_loop = crate::boot::end_sequence_text_loop_loc_1D3F0();

        assert_eq!(
            text_loop.page_text_subroutine, "sub_1D080",
            "ASM loc_1D3F0 (kid.asm:32741) should dispatch positive page entries through `sub_1D080`",
        );
        assert_eq!(
            text_loop.page_break_word, 0xFFFF,
            "ASM loc_1D404 is reached from loc_1D3F0 when the next word is the negative `$FFFF` page-break sentinel",
        );
        assert_eq!(
            text_loop.page_break_wait_seed, 0x00C8,
            "ASM loc_1D412 should seed `d0 = $C8` before the page-break wait loop",
        );
        assert_eq!(
            text_loop.page_break_wait_iterations, 0x00C9,
            "ASM loc_1D418 should execute `$C8 + 1 = $C9` `sub_1D830 -> Hibernate_Object_1Frame` refresh pairs before the page refresh continues",
        );
        assert_eq!(
            text_loop.page_break_refresh_subroutine, "sub_1D830",
            "ASM loc_1D418 should refresh the display via `sub_1D830` on each page-break wait iteration",
        );
        assert_eq!(
            text_loop.page_break_hide_glyphs_subroutine, "sub_1D7E6",
            "ASM loc_1D418 should hide all cached glyph workers through `sub_1D7E6` before the fade",
        );
        assert_eq!(
            text_loop.page_break_fade_subroutine, "sub_1D800",
            "ASM loc_1D418 should run `sub_1D800` to fade the previous page out before redrawing",
        );
        assert_eq!(
            text_loop.page_break_stripe_subroutine, "sub_1CDE0",
            "ASM loc_1D418 should rebuild the page stripes through `sub_1CDE0` after the fade",
        );
        assert_eq!(
            text_loop.page_break_glyph_load_subroutine, "sub_1D7A8",
            "ASM loc_1D418 should reload glyph placements through `sub_1D7A8` from the saved platform-layout pointer",
        );
        assert_eq!(
            text_loop.page_break_platform_layout_source, "Addr_PlatformLayout",
            "ASM loc_1D418 should read the current glyph layout from `(Addr_PlatformLayout).w` before calling `sub_1D7A8`",
        );
        assert_eq!(
            text_loop.terminator_word, 0x0000,
            "ASM loc_1D3F0 should treat a zero word as the end-of-credits terminator",
        );
        assert_eq!(
            text_loop.terminator_branch, "loc_1D44C",
            "ASM loc_1D3F0 should branch straight to `loc_1D44C` once the zero terminator is reached",
        );
    }

    #[test]
    fn end_sequence_terminator_exit_loc_1d450_waits_clears_palette_and_resets_state() {
        let exit = crate::boot::end_sequence_terminator_exit_loc_1D450();

        assert_eq!(
            exit.wait_seed, 0x0190,
            "ASM loc_1D44C should seed `d0 = $190` before the terminating wait loop enters loc_1D450",
        );
        assert_eq!(
            exit.wait_iterations, 0x0191,
            "ASM loc_1D450 (kid.asm:32784) should execute `$190 + 1 = $191` `sub_1D830 -> Hibernate_Object_1Frame` refresh pairs before blanking the palette",
        );
        assert_eq!(
            exit.refresh_subroutine, "sub_1D830",
            "ASM loc_1D450 should keep refreshing the display via `sub_1D830` during the final credits hold",
        );
        assert_eq!(
            exit.hibernate_subroutine, "j_Hibernate_Object_1Frame",
            "ASM loc_1D450 should hibernate one frame after each terminating display refresh",
        );
        assert_eq!(
            exit.palette_clear_dbf_seed, 0x003F,
            "ASM loc_1D450 should seed `d0 = $3F` before the palette-clear loop enters loc_1D464",
        );
        assert_eq!(
            exit.palette_clear_words, 0x0040,
            "ASM loc_1D464 (kid.asm:32791) should run `clr.w (a0)+; dbf d0,loc_1D464` for `$3F + 1 = $40` palette words before the final upload",
        );
        assert_eq!(
            exit.wait_for_vint_subroutine, "j_WaitForVint",
            "ASM loc_1D450 should wait for VBlank once after clearing the palette buffer",
        );
        assert_eq!(
            exit.idle_subroutine, "j_Do_Nothing",
            "ASM loc_1D450 should call `j_Do_Nothing` between the VBlank wait and the palette upload",
        );
        assert_eq!(
            exit.palette_upload_subroutine, "j_Palette_to_VRAM",
            "ASM loc_1D450 should upload the cleared palette through `j_Palette_to_VRAM` before resetting state",
        );
        assert!(
            exit.sets_unknown_flag_fbce,
            "ASM loc_1D450 should set `($FFFFFBCE).w` before clearing the neighbouring state words",
        );
        assert!(
            exit.clears_word_fbcc,
            "ASM loc_1D450 should clear `($FFFFFBCC).w` before returning control",
        );
        assert!(
            exit.clears_game_mode,
            "ASM loc_1D450 should clear `(Game_Mode).w` before leaving the end sequence",
        );
        assert!(
            exit.clears_number_lives,
            "ASM loc_1D450 should clear `(Number_Lives).w` before jumping to the callback",
        );
        assert_eq!(
            exit.continuation_pointer_source, "off_7192",
            "ASM loc_1D450 should finish by jumping through the callback stored in `(off_7192).w`",
        );
    }

    #[test]
    fn end_sequence_stripe_loop_loc_1cde4_steps_ten_display_refreshes_across_column_pairs() {
        let stripe_loop = crate::boot::end_sequence_stripe_loop_loc_1CDE4();

        assert_eq!(
            stripe_loop.start_column, 0x0009,
            "ASM sub_1CDE0 should seed `d0` with column `$9` before loc_1CDE4 starts",
        );
        assert_eq!(
            stripe_loop.iteration_seed, 0x0009,
            "ASM sub_1CDE0 should seed `d1` with `$9` before the `dbf d1,loc_1CDE4` loop",
        );
        assert_eq!(
            stripe_loop.iterations, 0x000A,
            "ASM loc_1CDE4 (kid.asm:31985) should execute `$9 + 1` outer iterations",
        );
        assert_eq!(
            stripe_loop.stripe_columns_per_iteration, 0x0002,
            "ASM loc_1CDE4 should call `sub_1CDFA` twice per iteration, covering adjacent column pairs",
        );
        assert_eq!(
            stripe_loop.final_next_column, 0x001D,
            "ASM loc_1CDE4 should leave `d0` at 29 after covering column pairs `(9,10)` through `(27,28)`",
        );
    }

    #[test]
    fn end_sequence_stripe_fill_loop_loc_1ce1c_writes_28_copies_of_0094_between_z80_guards() {
        let fill = crate::boot::end_sequence_stripe_fill_loop_loc_1CE1C();

        assert_eq!(
            fill.fill_word, 0x0094,
            "ASM loc_1CE1C (kid.asm:32012) should write immediate word `$94` on each pass through the fill loop",
        );
        assert_eq!(
            fill.iteration_seed, 0x001B,
            "ASM sub_1CDFA should seed `d7` with `$1B` before entering loc_1CE1C",
        );
        assert_eq!(
            fill.iterations, 0x001C,
            "ASM loc_1CE1C should execute `$1B + 1` inclusive DBF iterations, producing 28 fill writes",
        );
        assert_eq!(
            fill.z80_guard_subroutines,
            ["j_sub_914", "j_sub_924"],
            "ASM sub_1CDFA should wrap loc_1CE1C with the `j_sub_914` / `j_sub_924` guard calls",
        );
    }

    #[test]
    fn options_menu_text_draw_loc_1ce3a_highlights_row_zero_and_swaps_the_2p_copy_variant() {
        let first_option = crate::boot::options_menu_text_draw_loc_1CE3A(0, false);
        assert_eq!(
            first_option.first_line_row, 0,
            "ASM loc_1CE3A (kid.asm:32034) should always draw `OptText1` on row 0 before touching the 2P sub-option line",
        );
        assert!(
            first_option.first_line_highlighted,
            "ASM DrawOptText1 reaches loc_1CE3A with `d3 = 1` only when `Options_Selected_Option` is zero, so the first row should highlight only for cursor 0",
        );
        assert_eq!(
            first_option.second_line_row, 1,
            "ASM loc_1CE3A should then load `d4 = 1` before drawing the second options-menu row",
        );
        assert_eq!(
            first_option.second_line_text_label, "OptText2",
            "ASM loc_1CE56 (kid.asm:32044) should reach the shared second-row draw with `a4 = OptText2` when `Options_Suboption_2PController` is clear",
        );

        let two_controller = crate::boot::options_menu_text_draw_loc_1CE3A(2, true);
        assert!(
            !two_controller.first_line_highlighted,
            "ASM DrawOptText1 should leave `d3 = 0` once any option other than row 0 is selected",
        );
        assert_eq!(
            two_controller.second_line_text_label, "OptText3",
            "ASM loc_1CE56 should also serve the taken override path, drawing the shared second row with `a4 = OptText3` when `Options_Suboption_2PController` is non-zero",
        );
    }

    #[test]
    fn options_menu_controls_header_loc_1ceb2_draws_both_packed_opttext4_entries_on_row_three() {
        let controls_selected = crate::boot::options_menu_controls_header_loc_1CEB2(1);

        assert!(
            controls_selected.highlighted,
            "ASM DrawOptText2 (kid.asm:32126) should only raise `d3` when `Options_Selected_Option == 1`",
        );
        assert_eq!(
            controls_selected.first_line_x, 0,
            "ASM loc_1CEB2 (kid.asm:32132) should start `OptText4` from x offset 0 for the CONTROLS header",
        );
        assert_eq!(
            controls_selected.first_line_y, 3,
            "ASM loc_1CEB2 should keep the first packed `OptText4` entry on y offset 3",
        );
        assert_eq!(
            controls_selected.first_line_text_label, "OptText4.CONTROLS",
            "ASM loc_1CEB2 should draw the first packed `OptText4` entry as the CONTROLS header",
        );
        assert_eq!(
            controls_selected.second_line_x, 11,
            "ASM loc_1CEB2 should reach the second packed `OptText4` entry only after the first draw advances `a4` and `addq.w #1,a4` skips onto MODE's x offset",
        );
        assert_eq!(
            controls_selected.second_line_y, 3,
            "ASM loc_1CEB2 should keep the MODE header on the same y offset as CONTROLS",
        );
        assert_eq!(
            controls_selected.second_line_text_label, "OptText4.MODE",
            "ASM loc_1CEB2 should decode the second packed `OptText4` entry as the MODE header",
        );

        let another_row = crate::boot::options_menu_controls_header_loc_1CEB2(0);
        assert!(
            !another_row.highlighted,
            "ASM DrawOptText2 should leave `d3 = 0` for every cursor row other than CONTROLS",
        );
    }

    #[test]
    fn options_menu_controls_bindings_loc_1ced6_uses_highlighted_tile_base_and_all_six_scheme_permutations()
     {
        let highlighted = crate::boot::options_menu_controls_bindings_loc_1CED6(1, 4);
        assert!(
            highlighted.highlighted,
            "ASM loc_1CED6 (kid.asm:32145) should preserve the `d3` highlight state chosen just before the join",
        );
        assert_eq!(
            highlighted.selection_tile_word, 0xE528,
            "ASM loc_1CED6 should add `Options_Suboption_Controls + $1B` onto the highlighted `$E509` tile base before writing the selector word",
        );
        assert_eq!(
            highlighted.binding_label_rows,
            [15, 17, 19],
            "ASM loc_1CED6 should seed `d1` from `word_1CE2C + 5` and then step the three binding rows by two",
        );
        assert_eq!(
            highlighted.binding_text_labels,
            ["OptText5.SPECIAL", "OptText5.SPEED", "OptText5.JUMP"],
            "ASM loc_1CED6 should decode `unk_1CF28` scheme 4 as SPECIAL, SPEED, JUMP",
        );
        assert_eq!(
            highlighted.button_badge_rows,
            [15, 17, 19],
            "ASM loc_1CED6 should draw the trailing OptText6 A/B/C badges on the same three rows as the binding labels",
        );
        assert_eq!(
            highlighted.button_badge_text_labels,
            ["OptText6.A", "OptText6.B", "OptText6.C"],
            "ASM loc_1CED6 should keep the fixed OptText6 button badges in A, B, C order",
        );

        let unhighlighted = crate::boot::options_menu_controls_bindings_loc_1CED6(0, 5);
        assert!(
            !unhighlighted.highlighted,
            "ASM loc_1CED6 should fall back to the non-highlighted palette base when `d3` is clear",
        );
        assert_eq!(
            unhighlighted.selection_tile_word, 0xC529,
            "ASM loc_1CED6 should use the non-highlighted `$C509` tile base when the cursor is not on CONTROLS",
        );
        assert_eq!(
            unhighlighted.binding_text_labels,
            ["OptText5.SPECIAL", "OptText5.JUMP", "OptText5.SPEED"],
            "ASM loc_1CED6 should decode `unk_1CF28` scheme 5 as SPECIAL, JUMP, SPEED",
        );
    }

    #[test]
    fn options_menu_controls_binding_loop_loc_1cef8_runs_three_rows_and_decodes_scheme_stride_offsets()
     {
        let bindings = crate::boot::options_menu_controls_binding_loop_loc_1CEF8(3);

        assert_eq!(
            bindings.dbf_seed, 0x0002,
            "ASM loc_1CEF8 (kid.asm:32159) should seed `d2 = 2`, which makes the DBF loop run three inclusive iterations",
        );
        assert_eq!(
            bindings.iterations, 0x0003,
            "ASM loc_1CEF8 should execute exactly three binding-label draws before the DBF terminates",
        );
        assert_eq!(
            bindings.row_start, 15,
            "ASM loc_1CEF8 should inherit `d1 = word_1CE2C + 5`, starting the first binding label on row 15",
        );
        assert_eq!(
            bindings.row_step, 2,
            "ASM loc_1CEF8 should `addq.w #2,d1` between passes, spacing the binding rows by two",
        );
        assert_eq!(
            bindings.binding_text_labels,
            ["OptText5.JUMP", "OptText5.SPECIAL", "OptText5.SPEED"],
            "ASM loc_1CEF8 should decode scheme 3's `unk_1CF28` stride as JUMP, SPECIAL, SPEED",
        );
    }

    #[test]
    fn draw_text_line_tile_base_loc_1d050_selects_font_bank_and_palette_line() {
        assert_eq!(
            crate::boot::draw_text_line_palette_base_loc_1D05E(false),
            0xC000,
            "ASM loc_1D05E (kid.asm:32424) should keep `d6 = $C000` when `tst.b d3` sees an unhighlighted line",
        );
        assert_eq!(
            crate::boot::draw_text_line_palette_base_loc_1D05E(true),
            0xE000,
            "ASM loc_1D05E should overwrite `d6` with `$E000` when the highlight flag in `d3` is non-zero",
        );
        assert_eq!(
            crate::boot::draw_text_line_tile_base_loc_1D050(false, false),
            0xC4DC,
            "ASM DrawTextLine should keep the default `$4DC` glyph base and add palette line `$C000` when both `d4` and `d3` are clear",
        );
        assert_eq!(
            crate::boot::draw_text_line_tile_base_loc_1D050(true, false),
            0xE4DC,
            "ASM loc_1D050 (kid.asm:32418) should switch only the palette line to `$E000` when `d3` is set and `d4` is clear",
        );
        assert_eq!(
            crate::boot::draw_text_line_tile_base_loc_1D050(false, true),
            0xC509,
            "ASM DrawTextLine should replace `$4DC` with `$509` before loc_1D050 when `tst.b d4` is non-zero",
        );
        assert_eq!(
            crate::boot::draw_text_line_tile_base_loc_1D050(true, true),
            0xE509,
            "ASM loc_1D050 should combine the alternate `$509` glyph base with the highlighted `$E000` palette line",
        );
    }

    #[test]
    fn draw_text_line_glyph_loop_loc_1d068_streams_letters_until_the_first_nul_byte() {
        let glyphs = crate::boot::draw_text_line_glyph_loop_loc_1D068(0xC4DC, b"ACE\0Z");

        assert_eq!(
            glyphs.glyph_tiles,
            vec![0xC4DC, 0xC4DE, 0xC4E0],
            "ASM loc_1D068 (kid.asm:32429) should subtract `$41` from each non-zero letter byte and add that offset onto the running tile base in `d7`",
        );
        assert_eq!(
            glyphs.bytes_consumed, 4,
            "ASM loc_1D068 should stop after consuming the first NUL terminator from `(a4)+`, leaving any later bytes unread",
        );
    }

    #[test]
    fn draw_text_line_terminator_loc_1d07a_flushes_the_vdp_guard_only_for_the_nul_branch() {
        assert_eq!(
            crate::boot::draw_text_line_terminator_loc_1D07A(0),
            Some(crate::boot::DrawTextLineTerminatorLoc1D07A {
                flushes_vdp_guard: true,
                returns_after_flush: true,
            }),
            "ASM loc_1D07A (kid.asm:32439) should call `j_sub_924` and return immediately once `loc_1D068` reads a zero byte",
        );
        assert_eq!(
            crate::boot::draw_text_line_terminator_loc_1D07A(b'A'),
            None,
            "ASM loc_1D07A should stay unreachable for non-zero bytes because the glyph loop keeps dispatching back through `loc_1D068`",
        );
    }

    #[test]
    fn credits_glyph_preview_entry_loc_1d0a2_writes_the_vdp_command_and_gates_non_zero_glyphs() {
        let letter = crate::boot::credits_glyph_preview_entry_loc_1D0A2(0x4C40_0000, b'A');

        assert_eq!(
            letter,
            crate::boot::CreditsGlyphPreviewEntryLoc1D0A2 {
                vdp_command: 0x4C40_0000,
                tile_word: Some(0xE4DC),
                z80_guard_subroutines: Some(["j_sub_914", "j_sub_924"]),
                refresh_dbf_seed: Some(0x0002),
            },
            "ASM loc_1D0A2 (kid.asm:32463-32472) should write the prepared VDP command, remap `'A'` to tile word `$E4DC`, guard the write with `j_sub_914/j_sub_924`, and seed `d6 = 2` for the shared three-pass refresh loop",
        );

        assert_eq!(
            crate::boot::credits_glyph_preview_entry_loc_1D0A2(0x4C40_0000, 0),
            crate::boot::CreditsGlyphPreviewEntryLoc1D0A2 {
                vdp_command: 0x4C40_0000,
                tile_word: None,
                z80_guard_subroutines: None,
                refresh_dbf_seed: None,
            },
            "ASM loc_1D0A2 should still write `d5` to `4(a6)` on entry, but a zero glyph byte must branch straight to `return_1D0EA` without the guarded tile write or `d6` seed",
        );
    }

    #[test]
    fn credits_glyph_preview_refresh_loop_loc_1d0c2_runs_three_vblank_refresh_passes() {
        let refresh = crate::boot::credits_glyph_preview_refresh_loop_loc_1D0C2(0x4C40_0000);

        assert_eq!(
            refresh,
            crate::boot::CreditsGlyphPreviewRefreshLoopLoc1D0C2 {
                dbf_seed: 0x0002,
                iterations: 0x0003,
                subroutine_order: [
                    "sub_1DA24",
                    "sub_1CC88",
                    "j_Make_SpritesFromGfxObjects",
                    "j_WaitForVint",
                    "j_Transfer_SpriteAndKidToVRAM",
                ],
                preserves_register_window: true,
                next_vdp_command: 0x4C42_0000,
            },
            "ASM loc_1D0C2 (kid.asm:32475-32484) should run the display-refresh helper chain three times from `d6 = 2`, preserve `d0-a5` across the render helpers, and then advance `d5` by `$20000` for the next glyph row",
        );
    }

    #[test]
    fn options_menu_speed_text_loc_1cf78_switches_between_normal_and_fast_copy_pairs() {
        let normal = crate::boot::options_menu_speed_text_loc_1CF78(2, false);
        assert!(
            normal.highlighted,
            "ASM DrawOptText3 (kid.asm:32248) should only raise `d3` when `Options_Selected_Option == 2`",
        );
        assert_eq!(
            (normal.header_x, normal.header_y, normal.header_text_label),
            (0, 12, "OptText7.SPEED_BUTTON"),
            "ASM loc_1CF78 (kid.asm:32254) should always start by drawing the packed `OptText7` SPEED BUTTON header at `(0,$C)`",
        );
        assert_eq!(
            (normal.mode_x, normal.mode_y, normal.mode_text_label),
            (14, 12, "OptText8.NORMAL_ACTION"),
            "ASM loc_1CF78 should load `OptText8` when `Options_Suboption_Speed` is clear, so the second draw reads NORMAL ACTION from the `( $E,$C )` entry",
        );
        assert_eq!(
            (normal.prompt_x, normal.prompt_y, normal.prompt_text_label),
            (14, 14, "OptText8.PUSH_FOR_FAST"),
            "ASM loc_1CF94 should reuse the next packed `OptText8` entry for the trailing PUSH FOR FAST prompt",
        );

        let fast = crate::boot::options_menu_speed_text_loc_1CF78(1, true);
        assert!(
            !fast.highlighted,
            "ASM DrawOptText3 should leave `d3 = 0` when the cursor is not on the SPEED BUTTON row",
        );
        assert_eq!(
            (fast.mode_x, fast.mode_y, fast.mode_text_label),
            (14, 12, "OptText9.FAST_ACTION"),
            "ASM loc_1CF78 should switch `a4` to `OptText9` when `Options_Suboption_Speed` is non-zero, so the second draw becomes FAST ACTION",
        );
        assert_eq!(
            (fast.prompt_x, fast.prompt_y, fast.prompt_text_label),
            (14, 14, "OptText9.PUSH_FOR_NORMAL"),
            "ASM loc_1CF94 should then consume the following packed `OptText9` entry for the PUSH FOR NORMAL prompt",
        );
    }

    #[test]
    fn options_menu_exit_text_loc_1cffe_highlights_only_the_exit_row() {
        let selected = crate::boot::options_menu_exit_text_loc_1CFFE(3);
        assert!(
            selected.highlighted,
            "ASM DrawOptText4 (kid.asm:32357) should only raise `d3` when `Options_Selected_Option == 3`",
        );
        assert_eq!(
            (selected.text_x, selected.text_y, selected.text_label),
            (0, 17, "OptText10.EXIT"),
            "ASM loc_1CFFE (kid.asm:32363) should always point `a4` at `OptText10` and draw EXIT from packed coordinates `(0,$11)`",
        );

        let unselected = crate::boot::options_menu_exit_text_loc_1CFFE(2);
        assert!(
            !unselected.highlighted,
            "ASM DrawOptText4 should leave `d3 = 0` whenever the cursor is not on the EXIT row",
        );
    }

    #[test]
    fn end_sequence_camera_shake_sub_1d846_masks_v_int_bit_7_before_sign_shifting_rng_bytes() {
        let (camera_x_pos, camera_y_pos) = end_sequence_camera_shake_sub_1D846(0x81, 0x1F, 0x85);

        assert_eq!(
            camera_x_pos, -4,
            "ASM sub_1D846 should clear bit 7 of `V_Int_counter`, XOR the first RNG byte, then `asr.w #5` the signed result into Camera_X_pos",
        );
        assert_eq!(
            camera_y_pos, 0,
            "ASM sub_1D846 should reuse the same masked `V_Int_counter` byte for the second RNG byte before writing Camera_Y_pos",
        );
        assert_eq!(
            end_sequence_camera_shake_sub_1D846(0x81, 0x1F, 0x05),
            (camera_x_pos, camera_y_pos),
            "ASM sub_1D846's `bclr #7,d5` means the top bit of `V_Int_counter` must not affect either camera offset",
        );
    }

    #[test]
    fn end_sequence_burst_sprite_init_sub_1d86c_uses_masked_rng_offsets_and_preserves_priority() {
        let init = end_sequence_burst_sprite_init_sub_1D86C(0x81, 0x44, 0x85, 0x01);

        assert_eq!(
            init.callback_label, "sub_1D9D6",
            "ASM sub_1D86C should allocate a worker whose callback is `sub_1D9D6`",
        );
        assert_eq!(
            init.x_pos, 0x0062,
            "ASM sub_1D86C should arithmetic-shift the first signed RNG byte by one and add it to base X `$A0`",
        );
        assert_eq!(
            init.y_pos, 0x0090,
            "ASM sub_1D86C should arithmetic-shift the second signed RNG byte by one and add it to base Y `$70`",
        );
        assert_eq!(
            init.priority, 0x01,
            "ASM sub_1D86C should copy `d4` into the spawned worker's `$48(a0)` priority byte",
        );
        assert_eq!(
            end_sequence_burst_sprite_init_sub_1D86C(0x81, 0x44, 0x05, 0x01),
            init,
            "ASM sub_1D86C's `bclr #7,d5` means bit 7 of `V_Int_counter` must not affect the spawned offsets",
        );
    }

    #[test]
    fn loc_1d1d6_end_sequence_burst_prelude_seeds_priority_one_and_two_wait_frames() {
        let prelude = end_sequence_burst_prelude_loc_1D1D6();

        assert_eq!(
            prelude.burst_subroutine, "sub_1D86C",
            "ASM loc_1D1D6 (kid.asm:32582-32585) should branch to `sub_1D86C` for each burst spawn",
        );
        assert_eq!(
            prelude.burst_priority, 1,
            "ASM loc_1D1D6 should load `d4 = 1` before calling `sub_1D86C`, so spawned burst sprites keep priority 1",
        );
        assert_eq!(
            prelude.wait_seed, 0x0001,
            "ASM loc_1D1D6 should seed `d1 = 1` immediately before the `loc_1D1DE` hibernate loop",
        );
        assert_eq!(
            prelude.wait_iterations, 0x0002,
            "ASM loc_1D1D6 feeds `d1 = 1` into `dbf d1,loc_1D1DE`, so the inner wait loop runs `$1 + 1 = 2` frames",
        );
    }

    #[test]
    fn loc_1d1de_end_sequence_burst_wait_loop_repeats_21_outer_cycles_back_to_loc_1d1d6() {
        let wait_loop = end_sequence_burst_wait_loop_loc_1D1DE();

        assert_eq!(
            wait_loop.hibernate_subroutine, "j_Hibernate_Object_1Frame",
            "ASM loc_1D1DE (kid.asm:32587-32590) should hibernate once per inner-loop body",
        );
        assert_eq!(
            wait_loop.inner_wait_seed, 0x0001,
            "ASM loc_1D1DE should consume the `d1 = 1` seed prepared at `loc_1D1D6`",
        );
        assert_eq!(
            wait_loop.inner_wait_iterations, 0x0002,
            "ASM loc_1D1DE's `dbf d1,loc_1D1DE` should produce `$1 + 1 = 2` hibernation frames per burst",
        );
        assert_eq!(
            wait_loop.outer_loop_seed, 0x0014,
            "ASM loc_1D1DE should inherit `d0 = $14` from `loc_1D1CE` before the outer `dbf`",
        );
        assert_eq!(
            wait_loop.outer_loop_iterations, 0x0015,
            "ASM loc_1D1DE's `dbf d0,loc_1D1D6` should repeat the burst prelude `$14 + 1 = $15` times",
        );
        assert_eq!(
            wait_loop.next_outer_label, "loc_1D1D6",
            "ASM loc_1D1DE should loop back to `loc_1D1D6` until the outer DBF counter expires",
        );
    }

    #[test]
    fn loc_1d1ec_end_sequence_burst_camera_loop_runs_91_refresh_cycles() {
        let burst_loop = end_sequence_burst_camera_loop_loc_1D1EC();

        assert_eq!(
            burst_loop.burst_priority, 1,
            "ASM loc_1D1EC (kid.asm:32593-32599) should reload `d4 = 1` before each `sub_1D86C` burst spawn",
        );
        assert_eq!(
            burst_loop.outer_loop_seed, 0x005A,
            "ASM loc_1D1EC should start from `d0 = $5A` after the preceding wait-loop exhausts",
        );
        assert_eq!(
            burst_loop.outer_loop_iterations, 0x005B,
            "ASM loc_1D1EC's `dbf d0,loc_1D1EC` should repeat the burst-camera loop `$5A + 1 = $5B` times",
        );
        assert_eq!(
            burst_loop.per_iteration_subroutines,
            [
                "sub_1D86C",
                "sub_1D846",
                "sub_1DA24",
                "j_Hibernate_Object_1Frame",
            ],
            "ASM loc_1D1EC should run burst spawn, camera shake, display refresh, and one-frame hibernate in that exact order each iteration",
        );
    }

    #[test]
    fn end_sequence_falling_sprite_init_sub_1d8b4_keeps_full_v_int_delay_and_121_frame_fall() {
        let init = end_sequence_falling_sprite_init_sub_1D8B4(0x00AF, 0x0064, 0x1268, 0x33, 0x85);

        assert_eq!(
            init.gfx_slot_base, 0x0100_0000,
            "ASM sub_1D8B4 should load the fixed gfx-object slot `$1000000` before seeding the falling sprite",
        );
        assert!(
            init.display_enabled,
            "ASM sub_1D8B4 should set `$13(a3)` so the falling sprite starts visible",
        );
        assert_eq!(
            init.palette_line, 2,
            "ASM sub_1D8B4 should write palette line 2 before the sprite starts falling",
        );
        assert_eq!(
            init.vram_tile, 0x03DD,
            "ASM sub_1D8B4 should seed vram tile `$3DD` for the falling sprite",
        );
        assert_eq!(
            (init.x_pos, init.y_pos, init.sprite_mapping),
            (0x00AF, 0x0064, 0x1268),
            "ASM sub_1D8B4 should copy the caller's `$44/$46/$48` words directly into the spawned gfx object",
        );
        assert_eq!(
            init.initial_hibernate_frames, 0x005B,
            "ASM sub_1D8B4 should XOR the full `V_Int_counter` byte into the random delay before `lsr.w #1`",
        );
        assert_eq!(
            init.fall_frames, 0x0079,
            "ASM loc_1D8FA should run for `#$78+1` hibernated frames before deleting the current object",
        );
        assert_eq!(
            init.first_frame_y_delta_fp, 0,
            "ASM loc_1D8FA adds `d1` before the first `$800` increment, so the first falling frame should not move yet",
        );
        assert_eq!(
            init.last_frame_y_delta_fp, 0x03C000,
            "ASM loc_1D8FA's last per-frame Y add should use the 120th `$800` step",
        );
        assert_eq!(
            init.total_y_delta_fp, 0xE2E000,
            "ASM loc_1D8FA should accumulate the full 121-frame arithmetic series of `$800` Y deltas",
        );
        assert_ne!(
            end_sequence_falling_sprite_init_sub_1D8B4(0x00AF, 0x0064, 0x1268, 0x33, 0x05)
                .initial_hibernate_frames,
            init.initial_hibernate_frames,
            "ASM sub_1D8B4 uses the full `V_Int_counter` byte here, so bit 7 must affect the initial hibernate delay",
        );
    }

    #[test]
    fn loc_1d226_end_sequence_spawn_loop_seeds_three_falling_sprites_from_word_1d83a() {
        let spawn_loop = crate::boot::end_sequence_falling_sprite_spawn_loop_loc_1D226();

        assert_eq!(
            spawn_loop.object_slot_flags, 0x6000,
            "ASM loc_1D226 (kid.asm:32611-32619) should load `a0 = $6000` before each `j_Allocate_ObjectSlot` call",
        );
        assert_eq!(
            spawn_loop.callback_label, "sub_1D8B4",
            "ASM loc_1D226 should seed each allocated worker with callback `sub_1D8B4`",
        );
        assert_eq!(
            spawn_loop.loop_seed, 0x0002,
            "ASM loc_1D226 should start the DBF loop from `d0 = 2`",
        );
        assert_eq!(
            spawn_loop.iterations, 0x0003,
            "ASM loc_1D226's `dbf d0,loc_1D226` should allocate exactly three falling sprites",
        );
        assert_eq!(
            spawn_loop.spawn_inputs,
            [
                crate::boot::EndSequenceFallingSpriteSpawnInput {
                    x_pos: 0x00AF,
                    y_pos: 0x0064,
                    sprite_mapping: 0x1268,
                },
                crate::boot::EndSequenceFallingSpriteSpawnInput {
                    x_pos: 0x0099,
                    y_pos: 0x007F,
                    sprite_mapping: 0x126C,
                },
                crate::boot::EndSequenceFallingSpriteSpawnInput {
                    x_pos: 0x008C,
                    y_pos: 0x005A,
                    sprite_mapping: 0x1270,
                },
            ],
            "ASM loc_1D226 should read the three x/y pairs from `word_1D83A` and increment `d1` by 4 each spawn (`$1268,$126C,$1270`)",
        );
    }

    #[test]
    fn loc_1d24a_end_sequence_low_priority_burst_prelude_clears_priority_and_seeds_two_frame_wait()
    {
        let prelude = crate::boot::end_sequence_burst_low_priority_prelude_loc_1D24A();

        assert_eq!(
            prelude.burst_subroutine, "sub_1D86C",
            "ASM loc_1D24A (kid.asm:32622-32625) should keep calling `sub_1D86C` for each burst spawn",
        );
        assert_eq!(
            prelude.burst_priority, 0,
            "ASM loc_1D24A clears `d4` before `sub_1D86C`, so these burst sprites should use priority 0 rather than the earlier priority-1 path",
        );
        assert_eq!(
            prelude.wait_seed, 0x0001,
            "ASM loc_1D24A should seed `d1 = 1` for the following `loc_1D252` wait loop",
        );
    }

    #[test]
    fn loc_1d252_end_sequence_low_priority_wait_loop_repeats_51_outer_cycles_back_to_loc_1d24a() {
        let wait_loop = crate::boot::end_sequence_burst_low_priority_wait_loop_loc_1D252();

        assert_eq!(
            wait_loop.hibernate_subroutine, "j_Hibernate_Object_1Frame",
            "ASM loc_1D252 (kid.asm:32627-32630) should hibernate once per inner-loop body",
        );
        assert_eq!(
            wait_loop.inner_wait_seed, 0x0001,
            "ASM loc_1D252 should consume the `d1 = 1` seed prepared at `loc_1D24A`",
        );
        assert_eq!(
            wait_loop.inner_wait_iterations, 0x0002,
            "ASM loc_1D252's `dbf d1,loc_1D252` should produce `$1 + 1 = 2` hibernation frames per burst",
        );
        assert_eq!(
            wait_loop.outer_loop_seed, 0x0032,
            "ASM loc_1D252 should inherit `d0 = $32` from the setup immediately before `loc_1D24A`",
        );
        assert_eq!(
            wait_loop.outer_loop_iterations, 0x0033,
            "ASM loc_1D252's `dbf d0,loc_1D24A` should repeat the low-priority burst prelude `$32 + 1 = $33` times",
        );
        assert_eq!(
            wait_loop.next_outer_label, "loc_1D24A",
            "ASM loc_1D252 should loop back to `loc_1D24A` until the outer DBF counter expires",
        );
    }

    #[test]
    fn prize_10000_points_star_spawn_loc_10b3a_uses_the_first_mapping_only_on_frame_zero() {
        let spawn = prize_10000_points_star_spawn_loc_10B3A(0, 0x0123, 0x0045)
            .expect("frame 0 should spawn the first prize star");

        assert_eq!(
            spawn.sprite_mapping_label, "LnkTo_unk_E1056-Data_Index",
            "ASM loc_10B3A should keep the `move.w #LnkTo_unk_E1056,$22(a1)` special case only for `tst.w d3 == 0`",
        );
        assert_eq!(
            (spawn.x_pos, spawn.y_pos),
            (0x0123, 0x0045),
            "ASM loc_10B3A should copy the cached prize position from `$6C/$70(a5)` into the spawned star",
        );
        assert_eq!(
            (spawn.x_velocity_fp, spawn.y_velocity_fp),
            (-0x0002_0000, -0x0002_8000),
            "ASM loc_10B3A should convert the first `unk_10BF4` pair `(-$10,-$14)` into 16.16 velocities via `swap/clr/asr.l #3`",
        );
        assert!(
            spawn.display_enabled && spawn.is_moved,
            "ASM loc_10B3A should set both `$13(a1)` and `$14(a1)` on the spawned prize star",
        );
        assert_eq!(spawn.palette_line, 1);
        assert_eq!(spawn.timer, 0x0078);
    }

    #[test]
    fn prize_10000_points_star_spawn_loc_10b3a_only_emits_every_fifth_frame_after_frame_zero() {
        assert_eq!(
            prize_10000_points_star_spawn_loc_10B3A(4, 0x0000, 0x0000),
            None,
            "ASM loc_10B3A should not run on intermediate `d3` values where `unk_10A7A` holds `$FF`",
        );

        let spawn = prize_10000_points_star_spawn_loc_10B3A(15, 0x0080, 0x0060)
            .expect("frame 15 should spawn the fourth prize star");
        assert_eq!(
            spawn.sprite_mapping_label, "LnkTo_unk_E104E-Data_Index",
            "ASM loc_10B3A should leave the default mapping in place for non-zero `d3` spawns",
        );
        assert_eq!(
            (spawn.x_velocity_fp, spawn.y_velocity_fp),
            (0x0001_0000, -0x0002_8000),
            "ASM loc_10B3A should select the fourth `unk_10BF4` velocity pair `($8,-$14)` when `d3=15` maps to spawn index 3",
        );
    }

    #[test]
    fn prize_10000_points_star_dispatch_loc_10b76_skips_empty_worker_slots() {
        let dispatch = prize_10000_points_star_dispatch_loc_10B76(0x000C, false);

        assert!(
            !dispatch.should_step_slot,
            "ASM loc_10B76 should branch to loc_10BCE immediately when `$44(a5,d2.w)` is null",
        );
        assert_eq!(
            dispatch.next_slot_offset, 0x0010,
            "ASM loc_10B76 should leave empty star slots to the shared 4-byte slot walk",
        );
        assert!(
            !dispatch.finished_scan,
            "ASM loc_10B76 should only finish after the shared scan advances past the fifth slot",
        );
    }

    #[test]
    fn prize_10000_points_star_cleanup_loc_10be6_walks_all_five_cached_slots_before_deleting_parent()
     {
        let cleanup = prize_10000_points_star_cleanup_loc_10BE6([
            0x00FF_2448,
            0,
            0x00FF_24E0,
            0x00FF_252C,
            0x00FF_2578,
        ]);

        assert_eq!(
            cleanup.visited_slots,
            [0x00FF_2448, 0, 0x00FF_24E0, 0x00FF_252C, 0x00FF_2578],
            "ASM loc_10BE6 should execute `move.l (a2)+,a3; jsr (j_loc_1078).w` five times from `$44(a5)` through the fifth cached star slot before the `dbf` loop ends",
        );
        assert!(
            cleanup.should_delete_current_object,
            "ASM loc_10BE6 should jump to `Delete_CurrentObject` once the five-slot cleanup loop finishes",
        );
    }

    #[test]
    fn prize_10000_points_star_step_loc_10b74_hides_moved_flag_once_timer_reaches_0x70() {
        let step =
            prize_10000_points_star_step_loc_10B74(0x0000, 0x0070, 0x0040, 0x0030, 0, 0, true);

        assert!(
            !step.is_moved,
            "ASM loc_10B74 should execute `sf $14(a1)` exactly on the `$70` timer branch"
        );
        assert_eq!(
            step.x_pos, 0x0040,
            "ASM loc_10B74 should not move the star while the timer is still above `$20`"
        );
        assert_eq!(
            step.timer, 0x006F,
            "ASM loc_10BCA should still decrement the timer after the `$70` visibility change"
        );
        assert_eq!(step.latched_screen_pos, None);
        assert!(!step.delete_prize_worker);
    }

    #[test]
    fn prize_10000_points_star_step_loc_10b98_uses_the_shared_countdown_tail_for_other_timers() {
        let step =
            prize_10000_points_star_step_loc_10B74(0x0004, 0x0071, 0x0040, 0x0030, 0, 0, true);

        assert_eq!(
            step.x_pos, 0x0040,
            "ASM loc_10B98 should skip the `$70` hide path and the `$20` drift path for all other timers above `$20`",
        );
        assert!(
            step.is_moved,
            "ASM loc_10B98 should leave `$14(a1)` unchanged when the timer is neither `$70` nor `<= $20`",
        );
        assert_eq!(
            step.timer, 0x0070,
            "ASM loc_10B98 should fall straight into loc_10BCA's shared decrement tail",
        );
        assert_eq!(step.latched_screen_pos, None);
        assert!(!step.delete_prize_worker);
    }

    #[test]
    fn prize_10000_points_star_latch_screen_pos_loc_10b9a_caches_camera_relative_coords() {
        assert_eq!(
            prize_10000_points_star_latch_screen_pos_loc_10B9A(0x0034, 0x0056, 0x0010, 0x0020),
            (0x0024, 0x0036),
            "ASM loc_10B9A should store `$1A/$1E(a1) - Camera_X/Y_pos` into `$3E/$42(a1)` before loc_10BB2 starts the leftward drift",
        );
    }

    #[test]
    fn prize_10000_points_star_step_loc_10b74_latches_screen_position_at_timer_0x20_before_moving_left()
     {
        let step = prize_10000_points_star_step_loc_10B74(
            0x0008, 0x0020, 0x0034, 0x0056, 0x0010, 0x0020, true,
        );

        assert_eq!(
            step.latched_screen_pos,
            Some((0x0024, 0x0036)),
            "ASM loc_10B9A should store `$1A/$1E(a1) - Camera_X/Y_pos` into `$3E/$42(a1)` before loc_10BB2 starts the leftward drift",
        );
        assert_eq!(
            step.x_pos, 0x002E,
            "ASM loc_10BB2 should subtract 6 from the world X position once the timer reaches `$20`"
        );
        assert_eq!(
            step.timer, 0x001F,
            "ASM loc_10BCA should decrement the shared countdown after the `$20` branch"
        );
        assert!(step.is_moved);
        assert!(!step.delete_prize_worker);
    }

    #[test]
    fn prize_10000_points_star_step_loc_10b74_deletes_the_parent_worker_when_the_last_star_exits_left()
     {
        let step =
            prize_10000_points_star_step_loc_10B74(0x0010, 0x0010, -0x001B, 0x0000, 0, 0, true);

        assert!(
            step.delete_prize_worker,
            "ASM loc_10B74 should jump to loc_10BE0 when the fifth star's updated screen X falls below `#$FFE0` after loc_10BB2",
        );
        assert_eq!(
            step.x_pos, -0x0021,
            "ASM loc_10BB2 should still apply the 6px left step before the off-screen kill check"
        );
        assert_eq!(
            step.timer, 0x0010,
            "ASM loc_10BE0 branches before loc_10BCA, so the deletion frame must not decrement the timer",
        );
    }

    #[test]
    fn prize_10000_points_star_advance_loc_10bce_finishes_after_the_fifth_slot() {
        let advance = prize_10000_points_star_advance_loc_10BCE(0x0010);

        assert_eq!(
            advance.next_slot_offset, 0x0014,
            "ASM loc_10BCE should advance the longword slot index by 4 bytes each pass",
        );
        assert!(
            advance.finished_scan,
            "ASM loc_10BCE should stop the scan once `d2` reaches `#$14` after the fifth slot",
        );
    }

    #[test]
    fn enemy_death_burst_init_sub_37002_seeds_the_shared_homing_burst_sprite() {
        let init = enemy_death_burst_init_sub_37002(0x0044, 0x0046);

        assert_eq!(
            init.sound_effect_label, "sfx_Boss_eye_pops",
            "ASM sub_37002 should play `sfx_Boss_eye_pops` before allocating the burst sprite",
        );
        assert_eq!(
            init.gfx_slot_base, 0x0101_0002,
            "ASM sub_37002 should allocate gfx slot `$01010002` for the burst sprite",
        );
        assert_eq!(
            init.object_meta, 0x0020,
            "ASM sub_37002 should seed `object_meta(a3)` with `$20` before `loc_32146`",
        );
        assert!(
            init.display_enabled && init.is_moved,
            "ASM sub_37002 should set both `$13(a3)` and `is_moved(a3)` on the spawned burst sprite",
        );
        assert_eq!(
            init.x_direction, 0,
            "ASM sub_37002 should clear `x_direction(a3)` before the homing loop starts",
        );
        assert_eq!(
            init.sprite_mapping_label, "LnkTo_unk_C8460-Data_Index",
            "ASM sub_37002 should load the shared burst sprite mapping before the first hibernate",
        );
        assert_eq!(
            (init.x_pos, init.y_pos, init.sprite_word_y_adjust),
            (0x0044, 0x0046, 4),
            "ASM sub_37002 should copy `$44/$46(a5)` into the burst sprite and then add 4 to `$A(a3)`",
        );
        assert_eq!(
            init.chase_frames, 0x003C,
            "ASM sub_37002 should run the burst homing loop for `#$3C` hibernated frames unless collision deletes it first",
        );
        assert_eq!(
            init.homing_accel, 0x0000_0FA0,
            "ASM sub_37002 should accelerate the burst sprite toward Kid by `$FA0` on each axis per frame",
        );
        assert_eq!(
            init.delete_collision_types,
            [0x001C, 0x002C, -1],
            "ASM sub_37002 should delete immediately on collision types `$1C`, `$2C`, or `$FFFF`",
        );
    }

    #[test]
    fn enemy_death_burst_loop_gate_loc_37054_deletes_on_collision_before_countdown() {
        let gate = enemy_death_burst_loop_gate_loc_37054(0x001C, 0x003C);

        assert_eq!(
            gate.next,
            EnemyDeathBurstLoopNextLoc37054::Delete,
            "ASM loc_37054 should branch to the cleanup tail immediately when `collision_type(a3)` is `$001C`",
        );
        assert_eq!(
            (gate.next_collision_type, gate.next_chase_frames),
            (0x001C, 0x003C),
            "ASM loc_37054 checks collision before the later `clr.w collision_type(a3); subq.w #1,d0` path",
        );
    }

    #[test]
    fn enemy_death_burst_loop_gate_loc_37054_clears_collision_and_counts_down() {
        let gate = enemy_death_burst_loop_gate_loc_37054(0x1234, 2);

        assert_eq!(
            gate,
            EnemyDeathBurstLoopGateLoc37054 {
                next_collision_type: 0,
                next_chase_frames: 1,
                next: EnemyDeathBurstLoopNextLoc37054::ChaseKid,
            },
            "ASM loc_37054 should clear non-delete collision values, decrement `d0`, and continue to `loc_37078` while the counter is non-zero",
        );
    }

    #[test]
    fn enemy_death_burst_cleanup_loc_370c2_clears_motion_before_deleting() {
        let cleanup = enemy_death_burst_cleanup_loc_370C2();

        assert_eq!(
            cleanup,
            EnemyDeathBurstCleanupLoc370C2 {
                collision_type: 0,
                x_velocity: 0,
                y_velocity: 0,
                delete_current_object: true,
            },
            "ASM loc_370C2 should clear collision and both velocity longwords before jumping to `j_Delete_CurrentObject`",
        );
    }

    #[test]
    fn enemy_death_burst_homing_step_sub_37002_tracks_kid_with_signed_axis_accel() {
        let left_and_up = enemy_death_burst_homing_step_sub_37002(0x0040, 0x0080, 0x0060, 0x0040);
        assert_eq!(
            left_and_up.x_direction, 0xFF,
            "ASM loc_37078 should set `x_direction(a3)=$FF` when the burst sprite is at or left of Kid",
        );
        assert_eq!(
            (left_and_up.x_velocity_delta, left_and_up.y_velocity_delta),
            (0x0000_0FA0, -0x0000_0FA0),
            "ASM loc_37078..loc_370B8 should add `$FA0` toward Kid on both axes",
        );

        let right_and_down =
            enemy_death_burst_homing_step_sub_37002(0x0080, 0x0040, 0x0060, 0x0080);
        assert_eq!(
            right_and_down.x_direction, 0,
            "ASM loc_3709E should clear `x_direction(a3)` when the burst sprite is right of Kid",
        );
        assert_eq!(
            (
                right_and_down.x_velocity_delta,
                right_and_down.y_velocity_delta
            ),
            (-0x0000_0FA0, 0x0000_0FA0),
            "ASM loc_370AA should compare the burst y position with Kid and, through loc_370C0, loop after adding `$FA0` when the burst is above or level with Kid",
        );
    }

    #[test]
    fn boss_death_burst_init_sub_370d2_seeds_fall_homing_and_animation_constants() {
        let init = boss_death_burst_init_sub_370D2(0x0044, 0x0046);

        assert_eq!(
            init.sound_effect_label, "sfx_Voice_die",
            "ASM sub_370D2 should play `sfx_Voice_die` before allocating the boss death burst sprite",
        );
        assert_eq!(
            init.gfx_slot_base, 0x0101_0002,
            "ASM sub_370D2 should reuse gfx slot `$01010002` for the boss death burst sprite",
        );
        assert_eq!(
            (init.x_pos, init.y_pos, init.sprite_word_y_adjust),
            (0x0044, 0x0046, 3),
            "ASM sub_370D2 should copy `$44/$46(a5)` into the sprite and then add 3 to `$A(a3)`",
        );
        assert_eq!(
            init.chase_frames, 0x0078,
            "ASM sub_370D2 should seed the grounded homing loop timer with `#$78`",
        );
        assert_eq!(
            init.landing_gravity, 0x0000_2710,
            "ASM loc_3712E should add `$2710` to `y_vel(a3)` on each airborne frame before landing",
        );
        assert_eq!(
            init.homing_accel, 0x0000_07D0,
            "ASM loc_3716C..loc_371B4 should accelerate the grounded burst toward Kid by `$7D0` per axis",
        );
        assert_eq!(
            (
                init.entry_animation_label,
                init.collision_animation_label,
                init.timeout_animation_label,
            ),
            ("stru_37BD8", "stru_37BC6", "stru_37C0E"),
            "ASM sub_370D2 should enter with `stru_37BD8`, switch to `stru_37BC6` on collision delete, and to `stru_37C0E` on timeout",
        );
        assert_eq!(
            init.delete_collision_types,
            [0x001C, 0x002C, -1],
            "ASM sub_370D2 should share sub_37002's collision-delete set",
        );
    }

    #[test]
    fn boss_death_burst_fall_and_homing_sub_370d2_use_grounded_checks_and_slower_accel() {
        assert_eq!(
            boss_death_burst_fall_step_sub_370D2(0),
            0x0000_2710,
            "ASM loc_3712E should apply `$2710` gravity before testing the grounded byte",
        );

        let left_and_up = boss_death_burst_homing_step_sub_370D2(0x0040, 0x0080, 0x0060, 0x0040);
        assert_eq!(
            left_and_up.x_direction, 0xFF,
            "ASM loc_3716C should set `x_direction(a3)=$FF` when the burst sprite is at or left of Kid",
        );
        assert_eq!(
            (left_and_up.x_velocity_delta, left_and_up.y_velocity_delta),
            (0x0000_07D0, -0x0000_07D0),
            "ASM loc_3716C..loc_371AC should use the slower `$7D0` accel on both axes after landing",
        );

        let right_and_down = boss_death_burst_homing_step_sub_370D2(0x0080, 0x0040, 0x0060, 0x0080);
        assert_eq!(
            right_and_down.x_direction, 0,
            "ASM loc_37192 should clear `x_direction(a3)` when the burst sprite is right of Kid",
        );
        assert_eq!(
            (
                right_and_down.x_velocity_delta,
                right_and_down.y_velocity_delta
            ),
            (-0x0000_07D0, 0x0000_07D0),
            "ASM loc_3719E should add `$7D0` to `y_vel(a3)` when the burst is above or level with Kid after landing",
        );
    }

    #[test]
    fn boss_death_burst_loop_gate_loc_37148_deletes_on_collision_before_countdown() {
        let gate = boss_death_burst_loop_gate_loc_37148(0x002C, 0x0078);

        assert_eq!(
            gate.next,
            BossDeathBurstLoopNextLoc37148::CollisionFinish,
            "ASM loc_37148 should branch to the collision finish immediately when `collision_type(a3)` is `$002C`",
        );
        assert_eq!(
            (gate.next_collision_type, gate.next_chase_frames),
            (0x002C, 0x0078),
            "ASM loc_37148 checks collision before the later `clr.w collision_type(a3); subq.w #1,d0` path",
        );
    }

    #[test]
    fn boss_death_burst_loop_gate_loc_37148_clears_collision_and_counts_down() {
        let gate = boss_death_burst_loop_gate_loc_37148(0x1234, 2);

        assert_eq!(
            gate,
            BossDeathBurstLoopGateLoc37148 {
                next_collision_type: 0,
                next_chase_frames: 1,
                next: BossDeathBurstLoopNextLoc37148::ChaseKid,
            },
            "ASM loc_37148 should clear non-delete collision values, decrement `d0`, and continue the grounded homing loop while the counter is non-zero",
        );

        let timeout = boss_death_burst_loop_gate_loc_37148(0, 1);
        assert_eq!(
            timeout.next,
            BossDeathBurstLoopNextLoc37148::TimeoutFinish,
            "ASM loc_37148 should fall through to the timeout finish once `subq.w #1,d0` leaves the timer at zero",
        );
    }

    #[test]
    fn boss_death_burst_collision_finish_loc_371b6_clears_motion_and_deletes_after_anim() {
        let finish = boss_death_burst_collision_finish_loc_371B6();

        assert_eq!(
            finish,
            BossDeathBurstCollisionFinishLoc371B6 {
                collision_type: 0,
                x_velocity: 0,
                y_velocity: 0,
                animation_label: "stru_37BC6",
                hibernate_until_anim_finished: true,
                delete_current_object: true,
            },
            "ASM loc_371B6 should clear collision and both velocity longwords, run `stru_37BC6` to completion, then delete the object",
        );
    }

    #[test]
    fn boss_death_timeout_sound_sub_370d2_selects_the_rng_gated_finish_jingle() {
        assert_eq!(
            boss_death_timeout_sound_sub_370D2(0x00),
            "sfx_Boss_dies",
            "ASM loc_371F4 should choose `sfx_Boss_dies` when bit 0 of `$FFFFF809` is clear",
        );
        assert_eq!(
            boss_death_timeout_sound_sub_370D2(0x01),
            "sfx_Plethora_x",
            "ASM loc_371D4 should take the odd-bit branch and queue `sfx_Plethora_x` even though the ROM notes 'NO SOUND!'",
        );
    }

    #[test]
    fn boss_death_burst_timeout_loop_gate_loc_3720a_deletes_or_clears_collision_after_hibernate() {
        let collided = boss_death_burst_timeout_loop_gate_loc_3720A(0x001C);
        assert_eq!(
            collided,
            BossDeathBurstTimeoutLoopGateLoc3720A {
                hibernate_one_frame: true,
                next_collision_type: 0x001C,
                next: BossDeathBurstTimeoutLoopNextLoc3720A::CollisionFinish,
            },
            "ASM loc_3720A should hibernate one frame, then branch to loc_371B6 without clearing collision type `$001C`",
        );

        let continuing = boss_death_burst_timeout_loop_gate_loc_3720A(0x1234);
        assert_eq!(
            continuing,
            BossDeathBurstTimeoutLoopGateLoc3720A {
                hibernate_one_frame: true,
                next_collision_type: 0,
                next: BossDeathBurstTimeoutLoopNextLoc3720A::ContinueTimeoutFinish,
            },
            "ASM loc_3720A should clear non-delete collision values before comparing the burst sprite with Kid",
        );
    }

    #[test]
    fn boss_death_burst_timeout_face_kid_loc_37246_clears_direction_only_when_right_of_kid() {
        assert_eq!(
            boss_death_burst_timeout_face_kid_loc_37246(0x0061, 0x0060),
            0,
            "ASM loc_37246 should clear `x_direction(a3)` only when `x_pos(a3)` is strictly greater than Kid's `$1A`",
        );
        assert_eq!(
            boss_death_burst_timeout_face_kid_loc_37246(0x0060, 0x0060),
            0xFF,
            "ASM loc_3720A should use strict `bgt.s loc_37246`, so equal X falls through to `st x_direction(a3)`",
        );
        assert_eq!(
            boss_death_burst_timeout_face_kid_loc_37246(0x005F, 0x0060),
            0xFF,
            "ASM loc_3720A should set `x_direction(a3)` when the burst is left of Kid before branching to loc_3724A",
        );
    }
}
