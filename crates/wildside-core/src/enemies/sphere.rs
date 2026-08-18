#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct SphereWallBounceTrapLoc35FC4 {
    pub(crate) reachable_from_collision_dispatch: bool,
    pub(crate) traps_forever: bool,
}

#[allow(non_snake_case)]
#[cfg(test)]
pub(crate) fn sphere_wall_bounce_trap_loc_35FC4() -> SphereWallBounceTrapLoc35FC4 {
    // ASM Sphere.asm loc_35FC4: `jmp loc_35FC4(pc)` is a self-loop trap after
    // Enemy06_Sphere_BounceWall's `bra.s Enemy06_Sphere_Loop`, so normal sphere
    // collision dispatch must never continue into the orphaned code that follows.
    SphereWallBounceTrapLoc35FC4 {
        reachable_from_collision_dispatch: false,
        traps_forever: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SphereRespawnMarkerLoc36088 {
    pub(crate) index: usize,
    pub(crate) word_value: u16,
}

pub(crate) fn sphere_respawn_marker_loc_36088(
    object_status_slot: u8,
) -> SphereRespawnMarkerLoc36088 {
    // ASM Sphere.asm loc_36088: positive `$42(a5)` is doubled as a word offset
    // into EnemyStatus_Table, then `$2168` is written before the delete tail.
    SphereRespawnMarkerLoc36088 {
        index: usize::from(object_status_slot),
        word_value: 0x2168,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SphereDeleteTailLoc36096 {
    pub(crate) deletes_current_object: bool,
}

pub(crate) fn sphere_delete_tail_loc_36096() -> SphereDeleteTailLoc36096 {
    // ASM Sphere.asm loc_36096: every death-status route converges on the direct
    // `jmp (j_Delete_CurrentObject).w` tail after the explosion animation.
    SphereDeleteTailLoc36096 {
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) enum SphereChildLoopStepLoc360F4 {
    Delete,
    CheckCollision,
    Continue { frames_until_collision_check: u16 },
}

#[allow(non_snake_case)]
#[cfg(test)]
pub(crate) fn sphere_child_loop_step_loc_360F4(
    delete_flag_set: bool,
    frames_until_collision_check: u16,
) -> SphereChildLoopStepLoc360F4 {
    // ASM Sphere.asm loc_360F4: after `j_Hibernate_Object_1Frame`, `$19(a3)`
    // wins first; otherwise d0 either reaches the collision check or decrements
    // and loops for another frame.
    if delete_flag_set {
        SphereChildLoopStepLoc360F4::Delete
    } else if frames_until_collision_check == 0 {
        SphereChildLoopStepLoc360F4::CheckCollision
    } else {
        SphereChildLoopStepLoc360F4::Continue {
            frames_until_collision_check: frames_until_collision_check - 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) enum SphereCollisionGateLoc36106 {
    Delete,
    ContinueLoop,
}

#[cfg(test)]
pub(crate) fn sphere_collision_gate_loc_36106(collision_type: u16) -> SphereCollisionGateLoc36106 {
    // ASM Sphere.asm loc_36106: after the d0 grace counter expires, a non-zero
    // `collision_type(a3)` exits the child object; zero resumes the wait loop.
    if collision_type == 0 {
        SphereCollisionGateLoc36106::ContinueLoop
    } else {
        SphereCollisionGateLoc36106::Delete
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct SphereChildDeleteTailLoc3610E {
    pub(crate) deletes_current_object: bool,
}

#[allow(non_snake_case)]
#[cfg(test)]
pub(crate) fn sphere_child_delete_tail_loc_3610E() -> SphereChildDeleteTailLoc3610E {
    // ASM Sphere.asm loc_3610E: both the `$19(a3)` flag route and non-zero
    // collision route end at the direct `j_Delete_CurrentObject` tail.
    SphereChildDeleteTailLoc3610E {
        deletes_current_object: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct SphereCollisionTableGuardLoc36128 {
    pub(crate) reachable_from_valid_table_index: bool,
    pub(crate) traps_forever: bool,
}

#[cfg(test)]
pub(crate) fn sphere_collision_table_guard_loc_36128() -> SphereCollisionTableGuardLoc36128 {
    // ASM Sphere.asm loc_36128: `bra.s loc_36128` sits immediately before
    // `off_3612A`; valid collision IDs start at `$04` and dispatch into the
    // table, so this byte guard must never be a normal route.
    SphereCollisionTableGuardLoc36128 {
        reachable_from_valid_table_index: false,
        traps_forever: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SphereChildVerticalBounceLoc36142 {
    pub(crate) y_velocity: i32,
    pub(crate) resumes_child_loop: bool,
}

pub(crate) fn sphere_child_vertical_bounce_loc_36142(
    y_velocity: i32,
) -> SphereChildVerticalBounceLoc36142 {
    // ASM Sphere.asm loc_36142: `neg.l y_vel(a3)` then `bra.s loc_360F4`.
    SphereChildVerticalBounceLoc36142 {
        y_velocity: y_velocity.wrapping_neg(),
        resumes_child_loop: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    #[test]
    fn sphere_wall_bounce_trap_loc_35fc4_is_unreachable_self_loop() {
        assert_eq!(
            sphere_wall_bounce_trap_loc_35FC4(),
            SphereWallBounceTrapLoc35FC4 {
                reachable_from_collision_dispatch: false,
                traps_forever: true,
            },
            "ASM Sphere.asm loc_35FC4 is `jmp loc_35FC4(pc)` after Enemy06_Sphere_BounceWall already branched back to the loop"
        );
    }

    #[test]
    fn sphere_respawn_marker_loc_36088_writes_positive_status_slot() {
        assert_eq!(
            sphere_respawn_marker_loc_36088(0x05),
            SphereRespawnMarkerLoc36088 {
                index: 5,
                word_value: 0x2168,
            },
            "ASM Sphere.asm loc_36088 doubles positive `$42(a5)` into an EnemyStatus_Table word offset and writes `$2168`"
        );
    }

    #[test]
    fn sphere_delete_tail_loc_36096_jumps_to_delete_current_object() {
        assert_eq!(
            sphere_delete_tail_loc_36096(),
            SphereDeleteTailLoc36096 {
                deletes_current_object: true,
            },
            "ASM Sphere.asm loc_36096 should jump directly to `j_Delete_CurrentObject` after death-status handling"
        );
    }

    #[test]
    fn sphere_child_loop_loc_360f4_hibernates_then_prioritises_delete_flag() {
        assert_eq!(
            sphere_child_loop_step_loc_360F4(true, 6),
            SphereChildLoopStepLoc360F4::Delete,
            "ASM Sphere.asm loc_360F4 tests `$19(a3)` before consuming the d0 grace counter"
        );
        assert_eq!(
            sphere_child_loop_step_loc_360F4(false, 6),
            SphereChildLoopStepLoc360F4::Continue {
                frames_until_collision_check: 5,
            },
            "ASM Sphere.asm loc_360F4 decrements non-zero d0 and resumes its one-frame hibernate loop"
        );
        assert_eq!(
            sphere_child_loop_step_loc_360F4(false, 0),
            SphereChildLoopStepLoc360F4::CheckCollision,
            "ASM Sphere.asm loc_360F4 falls through to the collision check only when d0 is already zero"
        );
    }

    #[test]
    fn sphere_collision_gate_loc_36106_deletes_only_after_collision() {
        assert_eq!(
            sphere_collision_gate_loc_36106(0),
            SphereCollisionGateLoc36106::ContinueLoop,
            "ASM Sphere.asm loc_36106 resumes the child loop while `collision_type(a3)` is zero"
        );
        assert_eq!(
            sphere_collision_gate_loc_36106(0x0004),
            SphereCollisionGateLoc36106::Delete,
            "ASM Sphere.asm loc_36106 branches to deletion for any non-zero collision event"
        );
    }

    #[test]
    fn sphere_child_delete_tail_loc_3610e_jumps_to_delete_current_object() {
        assert_eq!(
            sphere_child_delete_tail_loc_3610E(),
            SphereChildDeleteTailLoc3610E {
                deletes_current_object: true,
            },
            "ASM Sphere.asm loc_3610E is the spawned child object's direct `j_Delete_CurrentObject` tail"
        );
    }

    #[test]
    fn sphere_collision_table_guard_loc_36128_is_self_loop_trap() {
        assert_eq!(
            sphere_collision_table_guard_loc_36128(),
            SphereCollisionTableGuardLoc36128 {
                reachable_from_valid_table_index: false,
                traps_forever: true,
            },
            "ASM Sphere.asm loc_36128 is `bra.s loc_36128`, guarding the bytes before `off_3612A` rather than a valid collision table entry"
        );
    }

    #[test]
    fn sphere_child_vertical_bounce_loc_36142_negates_y_velocity_and_resumes() {
        assert_eq!(
            sphere_child_vertical_bounce_loc_36142(0x0002_8000),
            SphereChildVerticalBounceLoc36142 {
                y_velocity: -0x0002_8000,
                resumes_child_loop: true,
            },
            "ASM Sphere.asm loc_36142 should `neg.l y_vel(a3)` before branching back to loc_360F4"
        );

        assert_eq!(
            sphere_child_vertical_bounce_loc_36142(i32::MIN),
            SphereChildVerticalBounceLoc36142 {
                y_velocity: i32::MIN,
                resumes_child_loop: true,
            },
            "ASM Sphere.asm loc_36142 uses 68k long-word negation, so the two's-complement minimum wraps to itself"
        );
    }

    #[test]
    fn sphere_wall_bounce_keeps_facing_per_loc_36148() {
        let mut grid = mini_hopping_skull_test_grid();
        grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.on_ground = false;
        enemy.facing = Direction::Right;
        enemy.x_pos = 24 << 16;
        enemy.y_pos = 24 << 16;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM GfxObjects_Collision loc_171C records the wall contact after motion without rewriting x_direction"
        );
        assert_eq!(enemy.x_velocity, 0x0001_0000);
        assert_eq!(
            sphere_pending_collision_type_loc_171c(&enemy),
            colid::RIGHTWALL
        );

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.facing,
            Direction::Right,
            "ASM Sphere.asm loc_36148 only negates `x_vel(a3)`, so wall hits must not rewrite x_direction"
        );
        assert_eq!(enemy.x_velocity, -0x0001_0000);
    }

    #[test]
    fn sphere_live_collision_row_reports_level_collision_loc_171c() {
        let data = vec![0u8; 10 * 12 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 10, 12);
        for col in 0..10 {
            grid.set_cell_at(col, 10, terrain::Cell::from_be_bytes(0x60, 0x00));
        }

        let mut enemy = sphere_test_enemy();
        enemy.x_pos = 80 << 16;
        enemy.y_pos = 152 << 16;
        enemy.x_velocity = 0;
        enemy.y_velocity = 0x0001_0000;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.y_velocity,
            0x0001_0000 + SPHERE_GRAVITY,
            "ASM Mode_Level writes Sphere collision_type after Enemy06_Sphere already ran, so the first live frame only records the floor hit"
        );
        assert_eq!(
            enemy.y_pos,
            152 << 16,
            "ASM GfxObjects_Collision loc_1A44 still snaps the live `stru_36174` active row on the collision pass"
        );
        assert_eq!(
            sphere_pending_collision_type_loc_171c(&enemy),
            colid::FLOOR,
            "ASM Sphere.asm `stru_36174` LnkTo_unk_C8120 maps to active CollisionSize_Index row `unk_31AB0`, not the preceding death zero row"
        );

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.y_velocity,
            -(0x0001_0000 + SPHERE_GRAVITY + SPHERE_GRAVITY),
            "ASM Sphere.asm loc_36142 consumes the previous GfxObjects_Collision floor type on the next object tick after applying gravity"
        );
        assert!(
            enemy.y_pos < 152 << 16,
            "the consumed floor response should send the Sphere upward on the next motion pass"
        );
    }

    #[test]
    fn sphere_trace_export_exposes_unwritten_current_hp_loc_35ef2() {
        let mut enemy = sphere_test_enemy();
        enemy.hit_points = 2;

        assert_eq!(
            enemy.snapshot().hit_points,
            Some(0),
            "ASM Sphere.asm Enemy06_Sphere_Init never writes GfxObject current_hp `$44`, so ROM-visible snapshots should expose zero"
        );
    }

    #[test]
    fn sphere_gravity_wraps_as_long_word_addition_loc_35f70() {
        let mut enemy = sphere_test_enemy();
        enemy.y_velocity = i32::MAX - (SPHERE_GRAVITY - 1);

        step_bouncer(&mut enemy, None);

        assert_eq!(
            enemy.y_velocity,
            i32::MIN,
            "ASM Sphere.asm loc_35F70 uses `addi.l #$800,y_vel(a3)`, whose 32-bit result wraps instead of saturating"
        );
    }

    #[test]
    fn sphere_right_wall_bounce_snaps_active_collision_row_loc_19a8() {
        let data = vec![0u8; 8 * 4 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        grid.set_cell_at(4, 1, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.x_pos = 57 << 16;
        enemy.y_pos = 24 << 16;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, 0x0001_0000,
            "ASM GfxObjects_Collision loc_171C records collision_type after Enemy06_Sphere's current-frame logic has already run"
        );
        assert_eq!(
            enemy.x_pos,
            56 << 16,
            "ASM GfxObjects_Collision loc_19A8 snaps row (-7,14,-7,14) to wall_left - (-7) - 14 - 1"
        );
        assert_eq!(
            sphere_pending_collision_type_loc_171c(&enemy),
            colid::RIGHTWALL
        );

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, -0x0001_0000,
            "ASM Sphere.asm loc_36148 negates x_vel when the next object tick consumes the right-wall collision"
        );
    }

    #[test]
    fn sphere_right_wall_probe_hits_active_snap_edge_loc_173e() {
        let data = vec![0u8; 8 * 4 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        grid.set_cell_at(4, 1, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.x_pos = 55 << 16;
        enemy.y_pos = 24 << 16;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, 0x0001_0000,
            "ASM GfxObjects_Collision loc_173E/1796 records the right-wall event after Enemy06_Sphere has already run"
        );
        assert_eq!(enemy.x_pos, 56 << 16);
        assert_eq!(
            sphere_pending_collision_type_loc_171c(&enemy),
            colid::RIGHTWALL,
            "ASM loc_19A8 snaps the Sphere centre to tile_left - 8 when the active row reaches the right-wall boundary"
        );
    }

    #[test]
    fn sphere_wall_collision_consumes_floor_response_loc_197e() {
        let data = vec![0u8; 8 * 4 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 8, 4);
        grid.set_cell_at(4, 1, terrain::Cell::from_be_bytes(0x60, 0x00));
        grid.set_cell_at(4, 2, terrain::Cell::from_be_bytes(0x60, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.x_pos = 56 << 16;
        enemy.y_pos = 24 << 16;
        enemy.x_velocity = 0x0001_0000;
        enemy.y_velocity = 0x0001_0000;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, 0x0001_0000,
            "ASM GfxObjects_Collision loc_197E/19A8 reports the right-wall collision after Enemy06_Sphere has already run"
        );
        assert_eq!(
            enemy.y_velocity,
            0x0001_0000 + SPHERE_GRAVITY,
            "ASM GfxObjects_Collision jumps back to the object loop after loc_19A8, so loc_1A44 floor handling must not also run in the same frame"
        );
        assert_eq!(enemy.x_pos, 56 << 16);
        assert_eq!(
            enemy.y_pos,
            (24 << 16) + 0x0001_0000 + SPHERE_GRAVITY as u32
        );
        assert_eq!(
            sphere_pending_collision_type_loc_171c(&enemy),
            colid::RIGHTWALL
        );

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, -0x0001_0000,
            "ASM Sphere.asm loc_36148 consumes only the wall event recorded by the prior collision pass"
        );
        assert!(
            enemy.y_velocity > 0,
            "the prior wall collision must not be double-consumed as a floor bounce"
        );
    }

    #[test]
    fn sphere_left_boundary_snaps_active_collision_row_loc_1aea() {
        let grid = terrain::BlockGrid::from_level_layout(&[0; 8 * 4 * 2], 8, 4);

        let mut enemy = sphere_test_enemy();
        enemy.x_pos = 0;
        enemy.y_pos = 24 << 16;
        enemy.x_velocity = -0x0001_0000;
        enemy.y_velocity = 0;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, -0x0001_0000,
            "ASM GfxObjects_Collision_LeftBoundary loc_1AEA only records the collision during the post-object collision pass"
        );
        assert_eq!(
            enemy.x_pos,
            7 << 16,
            "ASM GfxObjects_Collision_LeftBoundary loc_1AEA subtracts the negative left edge, snapping Sphere's x=-7 row back to centre x=7"
        );
        assert_eq!(
            sphere_pending_collision_type_loc_171c(&enemy),
            colid::LEFTWALL
        );

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_velocity, 0x0001_0000,
            "ASM Sphere.asm loc_36148 negates x_vel when the next object tick consumes the left-boundary collision"
        );
    }

    #[test]
    fn sphere_floor_bounce_uses_active_collision_row_loc_1a44() {
        let data = vec![0u8; 10 * 12 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 10, 12);
        for col in 0..10 {
            grid.set_cell_at(col, 10, terrain::Cell::from_be_bytes(0x60, 0x00));
        }

        let mut enemy = sphere_test_enemy();
        enemy.x_pos = 80 << 16;
        enemy.y_pos = 152 << 16;
        enemy.x_velocity = 0;
        enemy.y_velocity = 0x0001_0000;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.y_velocity,
            0x0001_0000 + SPHERE_GRAVITY,
            "ASM GfxObjects_Collision reports colid_floor after Enemy06_Sphere's current-frame dispatch"
        );
        assert_eq!(
            enemy.y_pos,
            152 << 16,
            "ASM GfxObjects_Collision loc_1A44 snaps row (-7,14,-7,14) to tile_top - (-7) - 14 - 1"
        );
        assert_eq!(sphere_pending_collision_type_loc_171c(&enemy), colid::FLOOR);

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.y_velocity,
            -(0x0001_0000 + SPHERE_GRAVITY + SPHERE_GRAVITY),
            "ASM Sphere.asm loc_36142 negates the post-gravity y_vel on the next object tick"
        );
    }

    #[test]
    fn sphere_slope_up_backs_out_motion_per_loc_3614e() {
        let data = vec![0u8; 10 * 10 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 10, 10);
        grid.set_cell_at(5, 5, terrain::Cell::from_be_bytes(0x40, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.facing = Direction::Right;
        enemy.x_pos = (5 * 16 + 8) << 16;
        enemy.y_pos = 85 << 16;
        enemy.x_velocity = 0x0002_0000;
        enemy.y_velocity = 0x0002_0000;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            sphere_pending_collision_type_loc_171c(&enemy),
            colid::SLOPEUP
        );
        assert_eq!(enemy.x_pos, (5 * 16 + 10) << 16);
        assert_eq!(enemy.y_velocity, 0x0002_0000 + SPHERE_GRAVITY);

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_pos,
            ((5 * 16 + 8) << 16) - (0x0002_0000 + SPHERE_GRAVITY + SPHERE_GRAVITY) as u32,
            "ASM Sphere.asm loc_3614E subtracts the previous post-gravity vector before the next global motion pass applies the swapped vector"
        );
        assert_eq!(
            enemy.y_velocity, -0x0002_0000,
            "ASM Sphere.asm loc_3614E sets y_vel to -old_x_vel"
        );
        assert_eq!(
            enemy.x_velocity,
            -(0x0002_0000 + SPHERE_GRAVITY + SPHERE_GRAVITY)
        );
        assert_eq!(enemy.facing, Direction::Right);
    }

    #[test]
    fn sphere_slope_down_backs_out_motion_per_loc_36164() {
        let data = vec![0u8; 10 * 10 * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, 10, 10);
        grid.set_cell_at(5, 5, terrain::Cell::from_be_bytes(0x50, 0x00));

        let mut enemy = sphere_test_enemy();
        enemy.facing = Direction::Right;
        enemy.x_pos = (5 * 16 + 8) << 16;
        enemy.y_pos = 89 << 16;
        enemy.x_velocity = 0x0002_0000;
        enemy.y_velocity = 0x0001_0000;

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            sphere_pending_collision_type_loc_171c(&enemy),
            colid::SLOPEDOWN
        );
        assert_eq!(enemy.x_pos, (5 * 16 + 10) << 16);
        assert_eq!(enemy.y_velocity, 0x0001_0000 + SPHERE_GRAVITY);

        step_bouncer(&mut enemy, Some(&grid));

        assert_eq!(
            enemy.x_pos,
            ((5 * 16 + 8) << 16) + (0x0001_0000 + SPHERE_GRAVITY + SPHERE_GRAVITY) as u32,
            "ASM Sphere.asm loc_36164 backs out by the previous vector before the next global motion pass applies the swapped vector"
        );
        assert_eq!(
            enemy.x_velocity,
            0x0001_0000 + SPHERE_GRAVITY + SPHERE_GRAVITY
        );
        assert_eq!(enemy.y_velocity, 0x0002_0000);
        assert_eq!(enemy.facing, Direction::Right);
    }
}
