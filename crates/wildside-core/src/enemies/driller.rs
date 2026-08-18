use super::*;

pub(crate) const DRILLER_COLID_KIDABOVE: u16 = colid::KIDABOVE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrillerInitTailRouteLoc3BE60 {
    ClearDirectionLoc3C0D2,
    SetDirectionLoc3C026,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrillerInitTailLoc3BE60 {
    pub(crate) next_drill_refcount: u8,
    pub(crate) callback: &'static str,
    pub(crate) next_route: DrillerInitTailRouteLoc3BE60,
}

pub(crate) fn driller_init_tail_loc_3be60(
    x_direction_set: bool,
    drill_refcount: u8,
) -> DrillerInitTailLoc3BE60 {
    // ASM Driller.asm loc_3BE60: after the optional initial DrillMoving sound,
    // increment the shared refcount at $FFFFFA34+object_meta, set a0=loc_3BE78,
    // then dispatch by x_direction into the shared walker loops.
    DrillerInitTailLoc3BE60 {
        next_drill_refcount: drill_refcount.wrapping_add(1),
        callback: "loc_3BE78",
        next_route: if x_direction_set {
            DrillerInitTailRouteLoc3BE60::SetDirectionLoc3C026
        } else {
            DrillerInitTailRouteLoc3BE60::ClearDirectionLoc3C0D2
        },
    }
}

/// Driller linked drill child X offset for block drilling.
/// ASM Driller.asm loc_3BD74 initialises the two tread/drill children at
/// parent X -$18 and +$18; loc_3BE78 probes the selected child object, not
/// the parent's `$48(a5)` collision half-width.
pub(crate) const DRILLER_DRILL_CHILD_X_OFFSET: i32 = 0x18;

/// ASM sub_3BF72 loc_3BFD4 (kid.asm): the per-frame re-pin puts the LEADING
/// child a full `$18` from the body and the TRAILING child at `$17`
/// (`subi.w #$18`/`addi.w #$17` when x_direction is clear, mirrored when set).
pub(crate) const DRILLER_DRILL_CHILD_TRAILING_X_OFFSET: i32 = 0x17;

/// ASM CollisionSize row `unk_31A7A` (kid.asm): `dc.w $1,$F,-$B,$9` — the
/// drill-child hitbox the level-collision engine reads through `$22(a2)`
/// for anim `stru_3BCBC` frames `LnkTo_unk_C80A8..C80C0`.
pub(crate) const DRILLER_DRILL_CHILD_BOX_X_OFFSET_UNK_31A7A: i32 = 0x1;
pub(crate) const DRILLER_DRILL_CHILD_BOX_WIDTH_UNK_31A7A: i32 = 0xF;
pub(crate) const DRILLER_DRILL_CHILD_BOX_Y_OFFSET_UNK_31A7A: i32 = -0xB;
pub(crate) const DRILLER_DRILL_CHILD_BOX_HEIGHT_UNK_31A7A: i32 = 0x9;

/// ORIGINAL_QUIRK: Enemy1A_Driller_Init arms `$26(a1)` with ∓$8000 x
/// velocity, but the children never set `is_moved` (`$14`), so GfxObject_Move
/// skips them and the velocity is NEVER integrated into position — it exists
/// solely to steer GfxObjects_Collision_Solid's moving-left/right resolution.
/// Child position is purely the loc_3BFD4 re-pin plus collision push-outs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrillerDrillChildSide {
    /// `$3A(a5)`: spawned at body x-$18, x_vel -$8000, sprite unflipped.
    Left,
    /// `$3E(a5)`: spawned at body x+$18, x_vel +$8000, `st $16(a1)` flipped.
    Right,
}

/// One drill child GfxObject. ASM Enemy1A_Driller_Init allocates both with
/// `st $3C(a1)` (level collision), so GfxObjects_Collision latches `$38`
/// wall events and pushes them out of solid tiles each frame; without
/// `is_moved` they never self-propel (see DRILLER_DRILL_CHILD_X_VEL).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrillerDrillChildSub3BF72 {
    /// Integer x pixel (`$1A(a2)`). Can go negative: GfxObjects_Collision's
    /// LeftBoundary arm shifts the object so the hitbox's left edge lands on
    /// x=0, leaving x = -1 for the unk_31A7A box (`sub.w d0,d7`).
    pub(crate) x_pos: i32,
    /// Integer y pixel; loc_3BFD4 re-pins it to the body y word each walker
    /// frame (`move.w d5,y_pos(a4)`), and the children never gain y velocity.
    pub(crate) y_pos: i32,
    /// Latched `$38(a2)` collision word (colid_leftwall=8 / colid_rightwall=4).
    /// GfxObjects_Collision overwrites it on each new contact; Driller.asm
    /// loc_3BE8E clears it only when the facing-side consume matches.
    pub(crate) wall_event: u16,
}

/// Both drill children of one Driller body, keyed by init side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrillerDrillChildrenSub3BF72 {
    pub(crate) left: DrillerDrillChildSub3BF72,
    pub(crate) right: DrillerDrillChildSub3BF72,
}

/// ASM Enemy1A_Driller_Init: both children spawn at body x -/+ $18 with the
/// body y (`move.w x_pos(a3),$1A(a1)` then `subi.w #$18`/`addi.w #$18`).
pub(crate) fn driller_spawn_children_enemy1a_init(
    body_x_px: i32,
    body_y_px: i32,
) -> DrillerDrillChildrenSub3BF72 {
    DrillerDrillChildrenSub3BF72 {
        left: DrillerDrillChildSub3BF72 {
            x_pos: body_x_px - DRILLER_DRILL_CHILD_X_OFFSET,
            y_pos: body_y_px,
            wall_event: 0,
        },
        right: DrillerDrillChildSub3BF72 {
            x_pos: body_x_px + DRILLER_DRILL_CHILD_X_OFFSET,
            y_pos: body_y_px,
            wall_event: 0,
        },
    }
}

/// ASM sub_3BF72 loc_3BFD4: every shared-walker hibernate site with `$3A(a5)`
/// set re-pins both children to the body. Facing-left (x_direction clear):
/// left child at x-$18, right at x+$17; facing-right (x_direction set): left
/// at x-$17, right at x+$18.
pub(crate) fn driller_repin_children_loc_3bfd4(
    children: &mut DrillerDrillChildrenSub3BF72,
    body_x_px: i32,
    body_y_px: i32,
    facing: Direction,
) {
    let (left_offset, right_offset) = match facing {
        Direction::Left => (
            DRILLER_DRILL_CHILD_X_OFFSET,
            DRILLER_DRILL_CHILD_TRAILING_X_OFFSET,
        ),
        Direction::Right => (
            DRILLER_DRILL_CHILD_TRAILING_X_OFFSET,
            DRILLER_DRILL_CHILD_X_OFFSET,
        ),
    };
    children.left.x_pos = body_x_px - left_offset;
    children.right.x_pos = body_x_px + right_offset;
    children.left.y_pos = body_y_px;
    children.right.y_pos = body_y_px;
}

/// ASM GfxObjects_Collision for one drill child at its pinned position:
/// latch `$38` wall events against level boundaries and `$6000`-class solid
/// tiles, pushing the hitbox out.
///
/// The solid resolution follows GfxObjects_Collision_Solid_MovingLeft/Right:
/// `lsr.l #8` of the SIGNED `$26` velocity leaves d0.w negative for the left
/// child, so the `muls`/`cmp.l` wall-vs-floor disambiguation always picks
/// the wall arm for a horizontal mover whose hitbox bottom sits below the
/// tile top (loc_19BA -> colid_leftwall / loc_1966 -> colid_rightwall).
pub(crate) fn driller_child_move_and_collide_gfx_objects_collision(
    child: &mut DrillerDrillChildSub3BF72,
    side: DrillerDrillChildSide,
    grid: &terrain::BlockGrid,
) {
    let x_int = child.x_pos;

    // ASM GfxObjects_Collision_SubBoxLoop: unflipped boxes span
    // [x+off, x+off+width]; `$16(a2)`-flipped boxes mirror to
    // [x-off-width, x-off]. The left child is unflipped, the right child is
    // flipped (`st $16(a1)` in Enemy1A_Driller_Init).
    let (box_left, box_right) = match side {
        DrillerDrillChildSide::Left => (
            x_int + DRILLER_DRILL_CHILD_BOX_X_OFFSET_UNK_31A7A,
            x_int
                + DRILLER_DRILL_CHILD_BOX_X_OFFSET_UNK_31A7A
                + DRILLER_DRILL_CHILD_BOX_WIDTH_UNK_31A7A,
        ),
        DrillerDrillChildSide::Right => (
            x_int
                - DRILLER_DRILL_CHILD_BOX_X_OFFSET_UNK_31A7A
                - DRILLER_DRILL_CHILD_BOX_WIDTH_UNK_31A7A,
            x_int - DRILLER_DRILL_CHILD_BOX_X_OFFSET_UNK_31A7A,
        ),
    };
    let level_width_px = i32::from(grid.width_cells) * 16;

    // ASM GfxObjects_Collision_ChkBoundaries: the level edges are checked
    // before any tile scan and latch the same wall codes.
    if box_left < 0 {
        // GfxObjects_Collision_LeftBoundary: `sub.w d0,d7` shifts the object
        // so the hitbox's left edge lands on x=0 (left child ends at x=-1).
        child.x_pos = x_int - box_left;
        child.wall_event = DRILLER_COLID_LEFTWALL;
        return;
    }
    if box_right >= level_width_px {
        // GfxObjects_Collision_RightBoundary: width - (right - x) - 1.
        child.x_pos = level_width_px - (box_right - x_int) - 1;
        child.wall_event = DRILLER_COLID_RIGHTWALL;
        return;
    }

    let box_top = child.y_pos + DRILLER_DRILL_CHILD_BOX_Y_OFFSET_UNK_31A7A;
    let box_bottom = box_top + DRILLER_DRILL_CHILD_BOX_HEIGHT_UNK_31A7A;
    if box_top < 0 {
        return;
    }
    // ASM GfxObjects_Collision_Tiles_Loop: rows outer, columns inner, first
    // colliding tile wins.
    for row in terrain::pixel_to_cell(box_top)..=terrain::pixel_to_cell(box_bottom) {
        for col in (box_left >> 4)..=(box_right >> 4) {
            let cell = grid.cell_at(col as u16, row as u16);
            // ASM loc_17C6: `$7000` spikes are ignored; `$4000/$5000` slopes
            // route to the slope arms whose horizontal-mover outcomes never
            // produce the `$38` wall codes Driller.asm loc_3BE8E consumes, so
            // they are skipped here; `$6000` is the solid arm.
            if cell.collision != terrain::CollisionKind::Solid {
                continue;
            }
            let tile_x = col << 4;
            let tile_y = row << 4;
            match side {
                DrillerDrillChildSide::Left => {
                    // GfxObjects_Collision_Solid_MovingLeft(Down): d0.w after
                    // `lsr.l #8` of $FFFF8000 is $FF80 (negative), so
                    // `(bottom-tile_y)*d0 < (left-tile_right)*y_vel(=0)`
                    // holds whenever the box bottom is below the tile top and
                    // loc_19BA -> loc_19D2 latches colid_leftwall.
                    if box_bottom > tile_y {
                        // loc_1A00 (unflipped): x = tile_x - $1 + $10.
                        child.x_pos = tile_x + 16 - DRILLER_DRILL_CHILD_BOX_X_OFFSET_UNK_31A7A;
                        child.wall_event = DRILLER_COLID_LEFTWALL;
                    }
                }
                DrillerDrillChildSide::Right => {
                    // Solid_MovingRight(Down): d0.w stays positive, so
                    // `(bottom-tile_y)*d0 > 0` picks loc_1966 -> loc_197E
                    // colid_rightwall for a box below the tile top.
                    if box_bottom > tile_y {
                        // loc_197E (flipped): x = tile_x + $1 - 1.
                        child.x_pos = tile_x + DRILLER_DRILL_CHILD_BOX_X_OFFSET_UNK_31A7A - 1;
                        child.wall_event = DRILLER_COLID_RIGHTWALL;
                    }
                }
            }
            return;
        }
    }
}

/// ASM ingame/anim/enemy/Driller.asm `stru_3BCCE`: one `$1E`-tick retract
/// frame the surviving stomped Driller hibernates through (loc_3BF4E).
pub(crate) const DRILLER_RETRACT_FRAMES_STRU_3BCCE: u8 = 0x1E;

pub(crate) const DRILLER_COLID_LEFTWALL: u16 = colid::LEFTWALL;
pub(crate) const DRILLER_COLID_RIGHTWALL: u16 = colid::RIGHTWALL;

/// ASM Driller.asm loc_3BE78/loc_3BE8E: the body callback selects the
/// facing-side child (`$3A(a5)`/d1=8 when x_direction is clear, `$3E(a5)`/
/// d1=4 when set), and only an exactly-matching `$38` word is consumed
/// (cleared) and turned into a block probe at the child's (x, y-$10).
pub(crate) fn driller_consume_child_wall_event_loc_3be8e(
    children: &mut DrillerDrillChildrenSub3BF72,
    facing: Direction,
) -> Option<(i32, i32)> {
    let (child, expected) = match facing {
        Direction::Left => (&mut children.left, DRILLER_COLID_LEFTWALL),
        Direction::Right => (&mut children.right, DRILLER_COLID_RIGHTWALL),
    };
    if child.wall_event == 0 || child.wall_event != expected {
        return None;
    }
    child.wall_event = 0;
    Some((child.x_pos, child.y_pos - 0x10))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrillerBlockDrillActionLoc3BEDE {
    None,
    SmashBits,
    IceBullets,
}

pub(crate) fn driller_block_drill_action_loc_3bede(
    cell: terrain::Cell,
) -> DrillerBlockDrillActionLoc3BEDE {
    if !(cell.has_block && cell.collision == terrain::CollisionKind::Solid) {
        return DrillerBlockDrillActionLoc3BEDE::None;
    }

    match cell.block_type {
        // ASM Driller.asm loc_3BEDE: block type mask `$000` or `$100`
        // calls Block_LoadSmashBits. In native block terms those are Smash
        // and Prize; Prize later restores through the normal deferred block path.
        terrain::BlockType::Smash | terrain::BlockType::Prize => {
            DrillerBlockDrillActionLoc3BEDE::SmashBits
        }
        // ASM Driller.asm loc_3BEF2: block type mask `$200` calls
        // IceBlock_SmashAndLoadBullets.
        terrain::BlockType::Ice => DrillerBlockDrillActionLoc3BEDE::IceBullets,
        _ => DrillerBlockDrillActionLoc3BEDE::None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrillerCollisionRouteLoc3BF02 {
    Continue,
    HurtAnimation,
    DeathAnimation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrillerCollisionDispatchLoc3BF02 {
    pub(crate) next_collision_type: u16,
    pub(crate) next_current_hp: u16,
    pub(crate) route: DrillerCollisionRouteLoc3BF02,
}

pub(crate) fn driller_collision_dispatch_loc_3bf02(
    collision_type: i16,
    current_hp: u16,
) -> DrillerCollisionDispatchLoc3BF02 {
    // ASM Driller.asm loc_3BF02: zero collision continues immediately;
    // negative words branch to loc_3BF1C before `clr.w collision_type(a3)`.
    // ASM Driller.asm loc_3BF66: every continue route rejoins the
    // `tst.b x_direction(a3)` dispatch into the shared walker loops.
    if collision_type == 0 {
        return DrillerCollisionDispatchLoc3BF02 {
            next_collision_type: 0,
            next_current_hp: current_hp,
            route: DrillerCollisionRouteLoc3BF02::Continue,
        };
    }

    if collision_type.is_negative() {
        return DrillerCollisionDispatchLoc3BF02 {
            next_collision_type: u16::from_ne_bytes(collision_type.to_ne_bytes()),
            next_current_hp: current_hp,
            route: DrillerCollisionRouteLoc3BF02::DeathAnimation,
        };
    }

    let collision_word = u16::from_ne_bytes(collision_type.to_ne_bytes());
    if collision_word != DRILLER_COLID_KIDABOVE {
        return DrillerCollisionDispatchLoc3BF02 {
            next_collision_type: 0,
            next_current_hp: current_hp,
            route: DrillerCollisionRouteLoc3BF02::Continue,
        };
    }

    let next_current_hp = current_hp.wrapping_sub(1);
    DrillerCollisionDispatchLoc3BF02 {
        next_collision_type: 0,
        next_current_hp,
        route: if next_current_hp == 0 {
            DrillerCollisionRouteLoc3BF02::DeathAnimation
        } else {
            // ASM Driller.asm loc_3BF4E: surviving stomp plays `stru_3BCCE`,
            // waits for it to finish, then restores `stru_3BCB2`.
            DrillerCollisionRouteLoc3BF02::HurtAnimation
        },
    }
}

/// Step a Driller enemy: consume the facing child's wall event into a block
/// smash (loc_3BE78), run the shared walker, then re-pin both children
/// (sub_3BF72 loc_3BFD4). The children's own move + level collision runs in
/// the runtime's post-object GfxObjects pass.
/// Returns the drilled cells and their ASM block-dispatch action this frame.
pub(crate) fn step_driller(
    enemy: &mut ActiveEnemy,
    block_grid: Option<&terrain::BlockGrid>,
) -> Vec<DrilledBlockLoc3BE78> {
    let Some(grid) = block_grid else {
        return Vec::new();
    };
    // ASM Enemy1A_Driller_Init spawns both children before entering the
    // walker. A native child created on this body pass cannot carry a prior
    // `$38` event, and loc_3BFD4 re-pins it before publication, so its
    // pre-consume placeholder coordinates are deliberately inert.
    let mut children = enemy
        .driller_children
        .unwrap_or_else(|| driller_spawn_children_enemy1a_init(0, 0));

    // ASM Mode_Level runs Execute_Objects before GfxObjects_MoveAndAnimate/
    // GfxObjects_Collision, so loc_3BE78 consumes the wall event latched by
    // the PREVIOUS frame's collision pass.
    let drilled = driller_consume_drilled_block_loc_3be78(&mut children, enemy.facing, grid);

    if drilled.is_empty() {
        step_pacer(enemy, Some(grid), None);
    } else {
        // ASM Driller.asm loc_3BEDE erases the block inside the body's own
        // pass, so the walker scan below already sees the drilled hole.
        let mut grid_after_drill = grid.clone();
        for drilled_block in &drilled {
            grid_after_drill.set_cell_at(
                drilled_block.col,
                drilled_block.row,
                terrain::Cell::from_be_bytes(0, 0),
            );
        }
        step_pacer(enemy, Some(&grid_after_drill), None);
    }

    // ASM sub_3BF72 loc_3BFD4: the shared-walker hibernate sites re-pin the
    // children to the post-walk body position each frame.
    driller_repin_children_loc_3bfd4(
        &mut children,
        (enemy.x_pos >> 16) as i32,
        (enemy.y_pos >> 16) as i32,
        enemy.facing,
    );
    enemy.driller_children = Some(children);

    drilled
}

/// ASM Driller.asm loc_3BE78..loc_3BEF2: consume the facing child's matching
/// `$38` wall word, probe the level word at the child's (x, y-$10) cell, and
/// dispatch negative words by their `$F00` nibble.
fn driller_consume_drilled_block_loc_3be78(
    children: &mut DrillerDrillChildrenSub3BF72,
    facing: Direction,
    grid: &terrain::BlockGrid,
) -> Vec<DrilledBlockLoc3BE78> {
    let Some((probe_x, probe_y)) = driller_consume_child_wall_event_loc_3be8e(children, facing)
    else {
        return Vec::new();
    };
    let direction = match facing {
        // ASM Driller.asm loc_3BE78 selects the right child and seeds d0=3.
        Direction::Right => 3,
        // ASM Driller.asm loc_3BE78 uses the left child with d0=1.
        Direction::Left => 1,
    };
    // ORIGINAL_QUIRK: loc_3BE8E computes the column with `lsr.w #4`, so a
    // boundary-clamped child at x=-1 indexes far outside the layout row and
    // reads unrelated RAM (observed on 0x55: never a smashable word). Native
    // maps that wrapped index to BlockGrid's empty out-of-range cell instead.
    let col = terrain::pixel_to_cell(probe_x) as u16;
    let row = terrain::pixel_to_cell(probe_y) as u16;
    let cell = grid.cell_at(col, row);

    let action = driller_block_drill_action_loc_3bede(cell);
    if matches!(action, DrillerBlockDrillActionLoc3BEDE::None) {
        Vec::new()
    } else {
        vec![DrilledBlockLoc3BE78 {
            col,
            row,
            action,
            direction,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::test_fixtures::*;

    #[test]
    fn driller_live_step_latches_init_tail_once_per_visible_driller() {
        let mut enemies = [ActiveEnemy {
            family: EnemyFamily::Driller,
            facing: Direction::Right,
            hit_points: 3,
            ..mini_hopping_skull_test_enemy()
        }];
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();

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
            None,
            &mut std::collections::HashMap::new(),
        );

        assert!(
            enemies[0].shared_walker_patrol_short_next,
            "ASM Driller.asm loc_3BE60 increments the shared DrillMoving refcount after installing loc_3BE78",
        );

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
            1,
            false,
            None,
            &mut rng,
            &mut drilled_blocks,
            None,
            &mut std::collections::HashMap::new(),
        );

        assert!(
            enemies[0].shared_walker_patrol_short_next,
            "ASM Driller.asm loc_3BE60 is an init tail, so the native latch must stay set after the first visible step",
        );
    }

    /// ASM Manage_EnemyLoading loc_3691E defers a fresh allocation's first
    /// execution one pass; `Enemy1A_Driller_Init` (Driller.asm line 6) then
    /// opens with `j_Hibernate_Object_1Frame`, and the loc_3C0D2/loc_3C026
    /// walker entry parks one more frame — a freshly loaded shell Driller
    /// spends THREE object passes without walking (oracle run 1786662123:
    /// 0x55 li=3 shows init resume codes 3BCF0/3BCF4 on title rows 61/62,
    /// parks at x=120 on row 63, and first steps on row 64).
    #[test]
    fn freshly_loaded_driller_defers_three_object_passes_loc_3691e() {
        let grid = mini_hopping_skull_test_grid();
        let mut enemies = [ActiveEnemy {
            family: EnemyFamily::Driller,
            facing: Direction::Left,
            hit_points: 3,
            x_pos: 64 << 16,
            ..mini_hopping_skull_test_enemy()
        }];
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();
        let mut freshly_loaded = std::collections::HashMap::new();
        freshly_loaded.insert((enemies[0].segment_index, enemies[0].layout_index), 3u8);

        for (pass, expected_x, label) in [
            (0u16, 64u32, "first pass is the loc_3691E allocation defer"),
            (1, 64, "second pass is the init's opening hibernate"),
            (2, 64, "third pass is the walker-entry hibernate"),
            (3, 63, "fourth pass takes the first walk step"),
        ] {
            step_active_enemies_with_rng(
                &mut enemies,
                Some(&grid),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                PlayerForm::TheKid,
                pass,
                false,
                None,
                &mut rng,
                &mut drilled_blocks,
                None,
                &mut freshly_loaded,
            );
            assert_eq!(
                enemies[0].x_pos >> 16,
                expected_x,
                "ASM loc_3691E + Enemy1A_Driller_Init entry hibernates: {label}"
            );
        }
        assert!(
            freshly_loaded.is_empty(),
            "the defer mark must be fully consumed after its three passes"
        );
    }

    /// ASM Enemy1A_Driller_Init computes the walk speed `$50(a5)` ONCE from
    /// `enemy_level+2` (loc_3BD6C/loc_3BD74) and no damage path rewrites it —
    /// a stomped Driller keeps its init speed. Oracle 0x55 visible window:
    /// the body patrols at 1px/frame after the second stomp drops it to hp 1
    /// (f119+ x steps 0x15,0x16,... per frame at hp 1).
    #[test]
    fn driller_walk_speed_keys_on_init_level_not_current_hp_loc_3bd74() {
        let grid = mini_hopping_skull_test_grid();
        let mut enemy = ActiveEnemy {
            family: EnemyFamily::Driller,
            facing: Direction::Left,
            enemy_level: 1,
            hit_points: 1,
            x_pos: 64 << 16,
            ..mini_hopping_skull_test_enemy()
        };
        enemy.driller_children = Some(driller_spawn_children_enemy1a_init(64, 32));
        step_driller(&mut enemy, Some(&grid));
        assert_eq!(
            enemy.x_pos,
            63 << 16,
            "an enemy_level-1 Driller stomped down to hp 1 still walks its \
             init `$10000` speed (1px/frame), not the hp-1 half-pixel tier"
        );
    }

    #[test]
    fn driller_spawn_setup_loc_3bd2a_faces_toward_player_and_seeds_hp() {
        assert_eq!(
            spawn_facing(EnemyFamily::Driller, 100 << 16, 80 << 16),
            Direction::Left,
            "ASM Driller.asm loc_3BD2A is reached without `st x_direction` when Kid is left of the Driller",
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Driller, 100 << 16, 100 << 16),
            Direction::Right,
            "ASM Driller.asm loc_3BD2A is reached after `st x_direction` when Kid is horizontally aligned",
        );
        assert_eq!(
            spawn_facing(EnemyFamily::Driller, 100 << 16, 120 << 16),
            Direction::Right,
            "ASM Driller.asm loc_3BD2A is reached after `st x_direction` when Kid is right of the Driller",
        );
        assert_eq!(
            spawn_hit_points(EnemyFamily::Driller, 0),
            2,
            "ASM Driller.asm loc_3BD2A stores enemy_level + 2 in current_hp",
        );
        assert_eq!(
            spawn_hit_points(EnemyFamily::Driller, 2),
            4,
            "ASM Driller.asm loc_3BD2A keeps raw enemy_level as the speed/HP tier before adding 2",
        );
    }

    #[test]
    fn driller_walk_speed_matches_loc_3bd6c_and_loc_3bd74() {
        assert_eq!(
            driller_walk_speed_loc_3bd74(2),
            0x0000_8000,
            "ASM Driller.asm loc_3BD74 gives low HP only the final `$8000` add",
        );
        assert_eq!(
            driller_walk_speed_loc_3bd74(3),
            0x0001_0000,
            "ASM Driller.asm loc_3BD6C lets HP 3 reach the middle `$8000` add before loc_3BD74",
        );
        assert_eq!(
            driller_walk_speed_loc_3bd74(4),
            0x0001_8000,
            "ASM Driller.asm loc_3BD64 high-HP preload plus loc_3BD6C/loc_3BD74 caps Driller speed at three `$8000` adds",
        );
        assert_eq!(
            driller_walk_speed_loc_3bd74(5),
            0x0001_8000,
            "ASM Driller.asm loc_3BD64 keeps HP above 4 on the same capped high-speed tier",
        );
    }

    #[test]
    fn driller_turnaround_counter_overflow_marks_death_loc_3c464() {
        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.family = EnemyFamily::Driller;
        enemy.facing = Direction::Right;
        enemy.hit_points = 1;
        enemy.shared_walker_turnaround_counter = 0xFC;
        let mut grid = mini_hopping_skull_test_grid();
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0, 0));

        step_pacer(&mut enemy, Some(&grid), None);

        assert_eq!(
            enemy.hit_points, 0,
            "ASM loc_3C464 -> Driller.asm loc_3BF02 routes $FFFF collision_type to the death path"
        );
        assert_eq!(
            enemy.wait_timer, DRILLER_DEATH_ANIMATION_FRAMES,
            "ASM Driller.asm loc_3BF1C should play the parent death animation instead of disappearing on the loc_3C464 route"
        );
        assert_eq!(enemy.shared_walker_turnaround_counter, 0);
    }

    #[test]
    fn driller_hurt_animation_hibernates_shared_walker_loc_3bf4e() {
        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.family = EnemyFamily::Driller;
        enemy.facing = Direction::Right;
        enemy.hit_points = 2;
        enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
        enemy.shared_walker_patrol_short_next = true;
        let start_x = enemy.x_pos;
        let mut enemies = [enemy];
        let grid = mini_hopping_skull_test_grid();
        let mut rng = RomRandomState::default();
        let mut drilled_blocks = Vec::new();

        step_active_enemies_with_rng(
            &mut enemies,
            Some(&grid),
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
            None,
            &mut std::collections::HashMap::new(),
        );

        assert_eq!(
            enemies[0].x_pos, start_x,
            "ASM Driller.asm loc_3BF4E hibernates through `stru_3BCCE` before loc_3BF66 can resume the shared walker"
        );
        assert_eq!(enemies[0].x_velocity, 0);
        assert_eq!(enemies[0].y_velocity, 0);
        assert!(drilled_blocks.is_empty());
    }

    #[test]
    fn driller_repin_children_loc_3bfd4_uses_asymmetric_facing_offsets() {
        let mut children = driller_spawn_children_enemy1a_init(120, 175);
        assert_eq!(
            (children.left.x_pos, children.right.x_pos),
            (120 - 0x18, 120 + 0x18),
            "ASM Enemy1A_Driller_Init spawns both children at body x -/+ $18",
        );

        driller_repin_children_loc_3bfd4(&mut children, 100, 176, Direction::Left);
        assert_eq!(
            (children.left.x_pos, children.right.x_pos),
            (100 - 0x18, 100 + 0x17),
            "ASM sub_3BF72 loc_3BFD4 x_direction-clear arm: subi #$18 left, addi #$17 right",
        );
        assert_eq!((children.left.y_pos, children.right.y_pos), (176, 176));

        driller_repin_children_loc_3bfd4(&mut children, 100, 176, Direction::Right);
        assert_eq!(
            (children.left.x_pos, children.right.x_pos),
            (100 - 0x17, 100 + 0x18),
            "ASM sub_3BF72 loc_3C000 x_direction-set arm: subi #$17 left, addi #$18 right",
        );
    }

    #[test]
    fn driller_child_latches_wall_events_and_pushes_out_gfx_objects_collision() {
        // 8x4 grid with a solid block at col 3, row 1 (unk_31A7A rows for
        // y=32 are 21..30 -> row 1).
        let mut grid = mini_hopping_skull_test_grid();
        grid.set_cell_at(3, 1, terrain::Cell::from_be_bytes(0xE0, 0x01));

        // Left child whose box [x+1, x+16] overlaps col 3 (span 48..63).
        let mut left = DrillerDrillChildSub3BF72 {
            x_pos: 62,
            y_pos: 32,
            wall_event: 0,
        };
        driller_child_move_and_collide_gfx_objects_collision(
            &mut left,
            DrillerDrillChildSide::Left,
            &grid,
        );
        assert_eq!(
            (left.x_pos, left.wall_event),
            (3 * 16 + 15, DRILLER_COLID_LEFTWALL),
            "ASM loc_19D2/loc_1A00: leftward solid contact latches colid_leftwall and pushes x to tile_x - $1 + $10",
        );

        // Right child (flipped box [x-16, x-1]) overlapping the same block.
        let mut right = DrillerDrillChildSub3BF72 {
            x_pos: 49,
            y_pos: 32,
            wall_event: 0,
        };
        driller_child_move_and_collide_gfx_objects_collision(
            &mut right,
            DrillerDrillChildSide::Right,
            &grid,
        );
        assert_eq!(
            (right.x_pos, right.wall_event),
            (3 * 16, DRILLER_COLID_RIGHTWALL),
            "ASM loc_197E flipped arm: rightward solid contact latches colid_rightwall and pushes x to tile_x + $1 - 1",
        );

        // One pixel short of overlap latches nothing.
        let mut shy = DrillerDrillChildSub3BF72 {
            x_pos: 63,
            y_pos: 32,
            wall_event: 0,
        };
        driller_child_move_and_collide_gfx_objects_collision(
            &mut shy,
            DrillerDrillChildSide::Left,
            &grid,
        );
        assert_eq!(
            (shy.x_pos, shy.wall_event),
            (63, 0),
            "ASM GfxObjects_Collision only fires once the unk_31A7A box (left edge x+1) reaches the tile span",
        );
    }

    #[test]
    fn driller_child_collision_pins_mirrored_unk_31a7a_box_endpoints() {
        let mut left_grid = mini_hopping_skull_test_grid();
        left_grid.set_cell_at(1, 1, terrain::Cell::from_be_bytes(0xE0, 0x01));
        left_grid.set_cell_at(4, 1, terrain::Cell::from_be_bytes(0xE0, 0x01));
        let mut left = DrillerDrillChildSub3BF72 {
            x_pos: 31,
            y_pos: 32,
            wall_event: 0,
        };
        driller_child_move_and_collide_gfx_objects_collision(
            &mut left,
            DrillerDrillChildSide::Left,
            &left_grid,
        );
        assert_eq!(
            (left.x_pos, left.wall_event),
            (31, 0),
            "ASM unk_31A7A unflipped box is exactly [x+1,x+16], so x=31 occupies only cols 2..2"
        );

        let mut right_endpoint_grid = mini_hopping_skull_test_grid();
        right_endpoint_grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0xE0, 0x01));
        let mut left_at_right_endpoint = DrillerDrillChildSub3BF72 {
            x_pos: 16,
            y_pos: 32,
            wall_event: 0,
        };
        driller_child_move_and_collide_gfx_objects_collision(
            &mut left_at_right_endpoint,
            DrillerDrillChildSide::Left,
            &right_endpoint_grid,
        );
        assert_eq!(
            (
                left_at_right_endpoint.x_pos,
                left_at_right_endpoint.wall_event
            ),
            (47, DRILLER_COLID_LEFTWALL),
            "ASM unk_31A7A includes the unflipped x+1+$F right endpoint at x+16"
        );

        let mut right_grid = mini_hopping_skull_test_grid();
        right_grid.set_cell_at(0, 1, terrain::Cell::from_be_bytes(0xE0, 0x01));
        right_grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0xE0, 0x01));
        right_grid.set_cell_at(3, 1, terrain::Cell::from_be_bytes(0xE0, 0x01));
        let mut right = DrillerDrillChildSub3BF72 {
            x_pos: 31,
            y_pos: 32,
            wall_event: 0,
        };
        driller_child_move_and_collide_gfx_objects_collision(
            &mut right,
            DrillerDrillChildSide::Right,
            &right_grid,
        );
        assert_eq!(
            (right.x_pos, right.wall_event),
            (0, DRILLER_COLID_RIGHTWALL),
            "ASM unk_31A7A flipped box is exactly [x-16,x-1], so its first solid at x=31 is col 0"
        );

        let mut edge = DrillerDrillChildSub3BF72 {
            x_pos: -1,
            y_pos: 32,
            wall_event: 0,
        };
        driller_child_move_and_collide_gfx_objects_collision(
            &mut edge,
            DrillerDrillChildSide::Left,
            &mini_hopping_skull_test_grid(),
        );
        assert_eq!(
            (edge.x_pos, edge.wall_event),
            (-1, 0),
            "ASM left-boundary branch is strict: a box whose left edge is exactly zero stays in bounds"
        );
    }

    #[test]
    fn driller_child_collision_pins_vertical_row_and_strict_wall_arm() {
        let mut top_grid = mini_hopping_skull_test_grid();
        top_grid.set_cell_at(2, 0, terrain::Cell::from_be_bytes(0xE0, 0x01));
        let mut top = DrillerDrillChildSub3BF72 {
            x_pos: 31,
            y_pos: 11,
            wall_event: 0,
        };
        driller_child_move_and_collide_gfx_objects_collision(
            &mut top,
            DrillerDrillChildSide::Left,
            &top_grid,
        );
        assert_eq!(
            (top.x_pos, top.wall_event),
            (47, DRILLER_COLID_LEFTWALL),
            "ASM GfxObjects_Collision accepts unk_31A7A box_top=0 and scans row 0"
        );

        let mut later_row_grid = mini_hopping_skull_test_grid();
        later_row_grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0xE0, 0x01));
        let mut row_one_only = DrillerDrillChildSub3BF72 {
            x_pos: 31,
            y_pos: 32,
            wall_event: 0,
        };
        driller_child_move_and_collide_gfx_objects_collision(
            &mut row_one_only,
            DrillerDrillChildSide::Left,
            &later_row_grid,
        );
        assert_eq!(
            (row_one_only.x_pos, row_one_only.wall_event),
            (31, 0),
            "ASM unk_31A7A y span 21..30 scans row 1 only, not later solid rows"
        );

        let mut exact_edge_grid = mini_hopping_skull_test_grid();
        exact_edge_grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0xE0, 0x01));
        for side in [DrillerDrillChildSide::Left, DrillerDrillChildSide::Right] {
            let mut exact_edge = DrillerDrillChildSub3BF72 {
                x_pos: if side == DrillerDrillChildSide::Left {
                    31
                } else {
                    48
                },
                y_pos: 18,
                wall_event: 0,
            };
            driller_child_move_and_collide_gfx_objects_collision(
                &mut exact_edge,
                side,
                &exact_edge_grid,
            );
            assert_eq!(
                exact_edge.wall_event, 0,
                "ASM loc_19BA/loc_1966 require box_bottom > tile_y; equality is not a wall hit"
            );
        }
    }

    #[test]
    fn driller_block_dispatch_requires_both_block_and_solid_flags_loc_3bede() {
        assert_eq!(
            driller_block_drill_action_loc_3bede(terrain::Cell {
                has_block: false,
                collision: terrain::CollisionKind::Solid,
                block_type: terrain::BlockType::Smash,
                ..terrain::Cell::from_be_bytes(0, 0)
            }),
            DrillerBlockDrillActionLoc3BEDE::None,
            "ASM loc_3BE8E requires a negative block word as well as the solid collision class"
        );
        assert_eq!(
            driller_block_drill_action_loc_3bede(terrain::Cell {
                has_block: true,
                collision: terrain::CollisionKind::Empty,
                block_type: terrain::BlockType::Smash,
                ..terrain::Cell::from_be_bytes(0, 0)
            }),
            DrillerBlockDrillActionLoc3BEDE::None,
            "ASM loc_3BE8E only receives the wall event produced by a solid drill-child contact"
        );
    }

    #[test]
    fn driller_child_level_boundaries_clamp_and_latch_loc_1aea() {
        let grid = mini_hopping_skull_test_grid();

        // 0x55 frame-0 witness: the wall-turned body at x=12 pins its left
        // child at -11; the LeftBoundary arm clamps the unk_31A7A box to the
        // level edge, leaving the child at exactly x=-1 with colid_leftwall.
        let mut left = DrillerDrillChildSub3BF72 {
            x_pos: -11,
            y_pos: 32,
            wall_event: 0,
        };
        driller_child_move_and_collide_gfx_objects_collision(
            &mut left,
            DrillerDrillChildSide::Left,
            &grid,
        );
        assert_eq!(
            (left.x_pos, left.wall_event),
            (-1, DRILLER_COLID_LEFTWALL),
            "ASM GfxObjects_Collision_LeftBoundary: `sub.w d0,d7` lands the box's left edge on x=0",
        );

        let mut right = DrillerDrillChildSub3BF72 {
            x_pos: 8 * 16 + 4,
            y_pos: 32,
            wall_event: 0,
        };
        driller_child_move_and_collide_gfx_objects_collision(
            &mut right,
            DrillerDrillChildSide::Right,
            &grid,
        );
        assert_eq!(
            (right.x_pos, right.wall_event),
            (8 * 16, DRILLER_COLID_RIGHTWALL),
            "ASM GfxObjects_Collision_RightBoundary: width - (right - x) - 1 keeps the flipped box inside the level",
        );
    }

    #[test]
    fn driller_consume_selects_facing_child_and_exact_code_loc_3be8e() {
        let mut children = driller_spawn_children_enemy1a_init(100, 176);
        children.left.wall_event = DRILLER_COLID_LEFTWALL;
        children.right.wall_event = DRILLER_COLID_RIGHTWALL;

        assert_eq!(
            driller_consume_child_wall_event_loc_3be8e(&mut children, Direction::Left),
            Some((100 - 0x18, 160)),
            "ASM loc_3BE78 x_direction-clear arm reads `$3A(a5)` with d1=8 and probes (x, y-$10)",
        );
        assert_eq!(
            children.left.wall_event, 0,
            "ASM loc_3BE8E clears the matched `$38` word",
        );
        assert_eq!(
            children.right.wall_event, DRILLER_COLID_RIGHTWALL,
            "the trailing child's latched word is untouched by the facing-side consume",
        );

        // A mismatched code on the facing child is left latched.
        children.left.wall_event = DRILLER_COLID_RIGHTWALL;
        assert_eq!(
            driller_consume_child_wall_event_loc_3be8e(&mut children, Direction::Left),
            None,
            "ASM loc_3BE8E `cmp.w d1,d4; bne` skips without clearing on a non-matching word",
        );
        assert_eq!(children.left.wall_event, DRILLER_COLID_RIGHTWALL);
    }

    #[test]
    fn driller_drill_probe_distinguishes_zero_and_out_of_range_axes_loc_3be8e() {
        let mut grid = mini_hopping_skull_test_grid();
        grid.set_cell_at(0, 0, terrain::Cell::from_be_bytes(0xE0, 0x01));

        for (probe_x, probe_y, expected) in [
            (
                0,
                0,
                vec![DrilledBlockLoc3BE78 {
                    col: 0,
                    row: 0,
                    action: DrillerBlockDrillActionLoc3BEDE::SmashBits,
                    direction: 1,
                }],
            ),
            (-1, 0, Vec::new()),
            (0, -1, Vec::new()),
            (i32::from(grid.width_cells) * 16, 0, Vec::new()),
            (0, i32::from(grid.height_cells) * 16, Vec::new()),
        ] {
            let mut children = driller_spawn_children_enemy1a_init(0, 0);
            children.left.x_pos = probe_x;
            children.left.y_pos = probe_y + 0x10;
            children.left.wall_event = DRILLER_COLID_LEFTWALL;

            assert_eq!(
                driller_consume_drilled_block_loc_3be78(&mut children, Direction::Left, &grid,),
                expected,
                "ASM Driller.asm loc_3BE8E probes exact cell zero, while native BlockGrid maps each out-of-range axis to the empty cell",
            );
        }
    }

    /// Oracle-probe run 2026-08-14 on level 0x68 (watches on row-4 cells
    /// `$FFA79E..$FFA7A2` + `Number_Objects`): the li=1 Driller walks right
    /// from x=72.0 at 0.5px/frame (hp 2) and its right drill child smashes
    /// row-4 cols 6/7/8 on scenario frames 2/34/66 exactly. This pins the
    /// whole chain: loc_3BFD4 re-pin (+$18, no GfxObject_Move integration),
    /// unk_31A7A box overlap latch, loc_197E push-out, and the loc_3BE78
    /// next-frame consume.
    #[test]
    fn driller_smash_cadence_matches_oracle_probe_level_0x68_row4() {
        let width: u16 = 40;
        let height: u16 = 8;
        let data = vec![0u8; usize::from(width) * usize::from(height) * 2];
        let mut grid = terrain::BlockGrid::from_level_layout(&data, width, height);
        // Level 0x68 rows: row 4 carries the `$E001` smash blocks (cols 4/5
        // already drilled during the shell), row 5 is the `$E710` floor.
        for col in 6..14 {
            grid.set_cell_at(col, 4, terrain::Cell::from_be_bytes(0xE0, 0x01));
        }
        for col in 0..width {
            grid.set_cell_at(col, 5, terrain::Cell::from_be_bytes(0xE7, 0x10));
        }

        let mut enemy = mini_hopping_skull_test_enemy();
        enemy.family = EnemyFamily::Driller;
        enemy.facing = Direction::Right;
        enemy.hit_points = 2;
        enemy.x_pos = 72 << 16;
        enemy.y_pos = 80 << 16;
        enemy.on_ground = true;
        // Frame-0 oracle state: children pinned by the previous walker frame,
        // no event pending (cols 4/5 are already open).
        enemy.driller_children = Some({
            let mut children = driller_spawn_children_enemy1a_init(72, 80);
            driller_repin_children_loc_3bfd4(&mut children, 72, 80, Direction::Right);
            children
        });

        let mut smashes = Vec::new();
        for frame in 0..70u32 {
            let drilled = step_driller(&mut enemy, Some(&grid));
            for block in &drilled {
                grid.set_cell_at(block.col, block.row, terrain::Cell::from_be_bytes(0, 0));
                smashes.push((frame, block.col, block.row, block.action, block.direction));
            }
            let children = enemy
                .driller_children
                .as_mut()
                .expect("step_driller seeds the children");
            driller_child_move_and_collide_gfx_objects_collision(
                &mut children.left,
                DrillerDrillChildSide::Left,
                &grid,
            );
            driller_child_move_and_collide_gfx_objects_collision(
                &mut children.right,
                DrillerDrillChildSide::Right,
                &grid,
            );
        }

        assert_eq!(
            smashes,
            vec![
                (2, 6, 4, DrillerBlockDrillActionLoc3BEDE::SmashBits, 3),
                (34, 7, 4, DrillerBlockDrillActionLoc3BEDE::SmashBits, 3),
                (66, 8, 4, DrillerBlockDrillActionLoc3BEDE::SmashBits, 3),
            ],
            "oracle-probe level 0x68: right-child smashes must land on frames 2/34/66 at row-4 cols 6/7/8",
        );
        assert_eq!(
            enemy.x_pos >> 16,
            72 + 35,
            "the hp-2 Driller walks 0.5px/frame (ASM loc_3BD74 single `$8000` add) through the drilled corridor without turning or dying",
        );
    }
}
