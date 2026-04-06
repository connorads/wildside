use super::*;
use crate::combat::CollisionRectSub1F52;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeBlockHeadbuttWorkerLoc1002E {
    pub(crate) sound_effect_label: &'static str,
    pub(crate) allocation_spec: u16,
    pub(crate) object_handler_addr: u32,
    pub(crate) block_col: u16,
    pub(crate) block_row: u16,
    pub(crate) layout_word_addr: u32,
    pub(crate) gfx_x_pos: u16,
    pub(crate) gfx_y_pos: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeBlockDeletedTileLoc101D8 {
    pub(crate) written_cell: terrain::Cell,
    pub(crate) fg_queue_entry: (u16, u16, u16),
    pub(crate) child_display_enabled: bool,
    pub(crate) sound_effect_label: &'static str,
    pub(crate) animation_script_addr: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeBlockSpawnFlashLoc102B8 {
    pub(crate) display_enabled: bool,
    pub(crate) palette_line: u8,
    pub(crate) addroffset_sprite: u16,
    pub(crate) animation_script_addr: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A: [i16; 31] = [
    -2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, 2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, -2, -2, -2, -2, 2,
    2, 2, 2, 2,
];

impl NativeRuntime {
    #[allow(non_snake_case)]
    pub(crate) fn lift_platform_vertical_step_sub_FD00(y_pos: u32, y_velocity: i32) -> u32 {
        // ASM sub_FD00 (kid.asm:21064): `move.l $E(a3),d7; add.l d7,6(a3)`.
        y_pos.wrapping_add_signed(y_velocity)
    }

    #[allow(non_snake_case)]
    pub(crate) fn step_lift_platform_sub_FCA6(
        lift: &mut LiftPlatform,
        level_height_pixels: u16,
    ) -> bool {
        // ASM sub_FCA6 (kid.asm:21021): hibernate one frame, call sub_FD00,
        // increment `d1` only when the integer `6(a3)` word changes, apply the
        // pre-/post-`$32` acceleration, then delete once `6(a3)` reaches the
        // signed level height.
        lift.age = lift.age.saturating_add(1);
        lift.y_pos = Self::lift_platform_vertical_step_sub_FD00(lift.y_pos, lift.y_velocity);

        let current_integer_y = (lift.y_pos >> 16) as u16;
        if current_integer_y != lift.last_integer_y {
            lift.moved_y_steps = lift.moved_y_steps.wrapping_add(1);
            lift.last_integer_y = current_integer_y;
        }

        if lift.moved_y_steps > 0x32 {
            lift.y_velocity = lift.y_velocity.wrapping_add(0x07D0);
        } else {
            lift.y_velocity = lift.y_velocity.wrapping_sub(0x0708);
        }

        let signed_integer_y = i32::from(i16::from_ne_bytes(current_integer_y.to_ne_bytes()));
        signed_integer_y < i32::from(level_height_pixels)
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_collision_rects_sub_F568(
        kid_hitbox: CollisionRectSub1F52,
        shooter_blocks: &[(u16, u16, u8)],
    ) -> Vec<CollisionRectSub1F52> {
        // ASM sub_F568 (kid.asm:20196): convert Kid's pixel hitbox to a
        // slightly-expanded block window, then emit up to two collision
        // rectangles per visible shooter edge inside that window.
        let left_block = (((i32::from(kid_hitbox.left_px)) >> 4) - 1).max(0) as i16;
        let top_block = (((i32::from(kid_hitbox.top_px)) >> 4) - 1).max(0) as i16;
        let right_block = ((i32::from(kid_hitbox.right_px) + 0x0F) >> 4) as i16 + 1;
        let bottom_block = ((i32::from(kid_hitbox.bottom_px) + 0x0F) >> 4) as i16 + 1;
        let mut rects = Vec::new();

        for &(col, row, directions) in shooter_blocks {
            let col = col as i16;
            let row = row as i16;
            if col < left_block || col > right_block || row < top_block || row > bottom_block {
                continue;
            }

            let x = col << 4;
            let y = row << 4;
            if directions & 0x01 != 0
                && row - top_block >= 2
                && col != left_block
                && col != right_block
            {
                rects.push(CollisionRectSub1F52 {
                    left_px: x + 2,
                    right_px: x + 14,
                    top_px: y - 13,
                    bottom_px: y - 1,
                });
                rects.push(CollisionRectSub1F52 {
                    left_px: x + 5,
                    right_px: x + 11,
                    top_px: y - 9,
                    bottom_px: y - 1,
                });
            }
            if directions & 0x02 != 0
                && col < right_block - 1
                && row != top_block
                && row != bottom_block
            {
                rects.push(CollisionRectSub1F52 {
                    left_px: (col + 1) << 4,
                    right_px: ((col + 1) << 4) + 8,
                    top_px: y + 1,
                    bottom_px: y + 13,
                });
                rects.push(CollisionRectSub1F52 {
                    left_px: (col + 1) << 4,
                    right_px: ((col + 1) << 4) + 12,
                    top_px: y + 4,
                    bottom_px: y + 10,
                });
            }
            if directions & 0x04 != 0
                && row < bottom_block - 1
                && col != left_block
                && col != right_block
            {
                rects.push(CollisionRectSub1F52 {
                    left_px: x + 2,
                    right_px: x + 14,
                    top_px: y + 15,
                    bottom_px: y + 23,
                });
                rects.push(CollisionRectSub1F52 {
                    left_px: x + 5,
                    right_px: x + 11,
                    top_px: y + 15,
                    bottom_px: y + 27,
                });
            }
            if directions & 0x08 != 0
                && col - left_block >= 2
                && row != top_block
                && row != bottom_block
            {
                rects.push(CollisionRectSub1F52 {
                    left_px: x - 8,
                    right_px: x,
                    top_px: y + 1,
                    bottom_px: y + 13,
                });
                rects.push(CollisionRectSub1F52 {
                    left_px: x - 12,
                    right_px: x,
                    top_px: y + 4,
                    bottom_px: y + 10,
                });
            }
        }

        rects
    }

    // Oracle-backed sub_1007A timing for demo 0x1B:
    // hit frame N:      write 0x6F
    // frames N+1..N+32: keep 0x6F during the bounce coroutine
    // frame N+33:       restore original prize cell (E1xx)
    // frame N+34:       clear cell and spawn prize object
    pub(crate) const PRIZE_BLOCK_DELETED_FRAMES: u16 = 32;
    pub(crate) const PRIZE_DIAMOND_LIFETIME_FRAMES: u16 = 0x01F4;
    pub(crate) const PRIZE_GENERIC_LIFETIME_FRAMES: u16 = 0x0320;
    // PrizeDiamond_Init starts moving on the frame after object creation, so a
    // newly spawned native floating item should be active immediately.
    pub(crate) const PRIZE_ITEM_INITIAL_DORMANT_FRAMES: u16 = 0;

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_headbutt_worker_loc_1002E(
        block_col: u16,
        block_row: u16,
        layout_word_addr: u32,
    ) -> PrizeBlockHeadbuttWorkerLoc1002E {
        // ASM loc_1002E (kid.asm:21317): preserve `d0`, play `sfx_Block_hit`,
        // allocate object/gfx slots via `sub_E02`, install `sub_1007A`, copy
        // `d1/d2/d3` into `$44/$46/$48(a0)`, and seed the gfx object's pixel
        // position by shifting the caller's block coordinates left by four.
        PrizeBlockHeadbuttWorkerLoc1002E {
            sound_effect_label: "sfx_Block_hit",
            allocation_spec: 0x2001,
            object_handler_addr: 0x0001_007A,
            block_col,
            block_row,
            layout_word_addr,
            gfx_x_pos: block_col << 4,
            gfx_y_pos: block_row << 4,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_deleted_tile_loc_101D8(
        block_col: u16,
        block_row: u16,
        deleted_skin: u8,
    ) -> PrizeBlockDeletedTileLoc101D8 {
        // ASM loc_101D8 (kid.asm:21459): write the deleted-solid tile by
        // storing `d4` into `(a0)` and then overriding only the high byte with
        // `$6F`, queue the same `(d0,d1,d4)` triple into
        // `FGUpdateQueueCell_NextSlot`. ASM loc_10200 (kid.asm:21474) then
        // publishes that advanced queue pointer, clears the child display byte
        // with `st $13(a3)`, preserves `d0` across the `sfx_Block_hit` call,
        // and seeds `d7` with `stru_10296` before the bounce coroutine.
        PrizeBlockDeletedTileLoc101D8 {
            written_cell: terrain::Cell::from_be_bytes(0x6F, deleted_skin),
            fg_queue_entry: (block_col, block_row, u16::from(deleted_skin)),
            child_display_enabled: false,
            sound_effect_label: "sfx_Block_hit",
            animation_script_addr: 0x0001_0296,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_wobble_y_deltas_sub_1007A() -> &'static [i16; 31] {
        // ASM loc_100DA (kid.asm:21373) starts the prize-block wobble by
        // repeating `subq.w #2,y_pos(a3)` for the inclusive `dbf d0` count
        // seeded from `moveq #5,d0`, then the later `loc_100E8/100F6/10104/
        // 10112/10120` loops mirror the remaining 6/5/5/4/5-frame bounces.
        &PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn deleted_tile_wobble_y_deltas_loc_10200() -> &'static [i16; 31] {
        // ASM loc_10236 (kid.asm:21493): after the opening six-frame
        // `loc_10228` rise, `moveq #5,d1` drives an inclusive `dbf` loop of
        // six `addq.w #2,y_pos(a3)` hibernate steps for the deleted tile.
        // ASM loc_10244 (kid.asm:21499) then reloads `d0` with `#4`, so the
        // next `subq.w #2,y_pos(a3)` phase lasts five frames before the later
        // rebound loops continue. ASM loc_10252 (kid.asm:21505) reloads `d1`
        // with `#4`, so the next rebound is five `addq.w #2,y_pos(a3)` steps.
        // ASM loc_10260 (kid.asm:21511) then reloads `d0` with `#3`, which
        // shortens the next `subq.w #2,y_pos(a3)` dip to four frames. ASM
        // loc_1026E (kid.asm:21517) reloads `d1` with `#4` for a final
        // five-frame `addq.w #2,y_pos(a3)` rebound before the animation waits
        // to finish and the worker deletes itself.
        &PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_spawn_flash_loc_102B8() -> PrizeBlockSpawnFlashLoc102B8 {
        // ASM loc_102B8 (kid.asm:21542): follow the worker's `$36(a5)` gfx
        // link, hide it via `st $13(a3)`, force palette line 0, seed
        // `addroffset_sprite(a3)` with `LnkTo_unk_E0F2E`, then run `stru_102DE`
        // until completion before the worker deletes itself.
        PrizeBlockSpawnFlashLoc102B8 {
            display_enabled: false,
            palette_line: 0,
            addroffset_sprite: 0x0F2E,
            animation_script_addr: 0x0001_02DE,
        }
    }

    fn floating_item_lifetime(content: terrain::PrizeContent) -> u16 {
        match content {
            terrain::PrizeContent::Diamond => Self::PRIZE_DIAMOND_LIFETIME_FRAMES,
            // ASM sub_10C78 seeds d3 with $0001_0320 for the shared
            // Ankh/Clock/Continue falling-prize initialiser, while sub_10CB0
            // reuses the same lifetime for helmets before its extra cleanup.
            terrain::PrizeContent::Points10000
            | terrain::PrizeContent::Skycutter
            | terrain::PrizeContent::Cyclone
            | terrain::PrizeContent::RedStealth
            | terrain::PrizeContent::Eyeclops
            | terrain::PrizeContent::Juggernaut
            | terrain::PrizeContent::IronKnight
            | terrain::PrizeContent::Berzerker
            | terrain::PrizeContent::Maniaxe
            | terrain::PrizeContent::Micromax
            | terrain::PrizeContent::Ankh
            | terrain::PrizeContent::Clock
            | terrain::PrizeContent::Continue
            | terrain::PrizeContent::TenDiamonds => Self::PRIZE_GENERIC_LIFETIME_FRAMES,
        }
    }

    fn is_helmet_prize(content: terrain::PrizeContent) -> bool {
        matches!(
            content,
            terrain::PrizeContent::Skycutter
                | terrain::PrizeContent::Cyclone
                | terrain::PrizeContent::RedStealth
                | terrain::PrizeContent::Eyeclops
                | terrain::PrizeContent::Juggernaut
                | terrain::PrizeContent::IronKnight
                | terrain::PrizeContent::Berzerker
                | terrain::PrizeContent::Maniaxe
                | terrain::PrizeContent::Micromax
        )
    }

    fn clear_existing_helmet_prizes_for_sub_10cec(&mut self) {
        let mut removed_effects = Vec::new();
        self.floating_items.retain(|item| {
            if !Self::is_helmet_prize(item.content) {
                return true;
            }

            let x_px = (item.x_pos >> 16) as u16;
            let y_px = ((item.y_pos >> 16) as u16).saturating_sub(8);
            removed_effects.push(BlockDestroyEffect {
                // ASM sub_10CEC deletes the older helmet owner, then sub_10D3E
                // spawns a transient animation at the deleted helmet's x_pos
                // and y_pos-8.
                x_px,
                y_px,
                remaining: 8,
            });
            false
        });
        self.block_destroy_effects.extend(removed_effects);
    }

    const RUBBER_BLOCK_SUB_10DE4_SEQUENCE: [(u8, u8); 9] = [
        (1, 2),
        (0, 1),
        (1, 1),
        (0, 1),
        (1, 2),
        (0, 2),
        (1, 3),
        (0, 3),
        (0, 0),
    ];
    pub(crate) const BLOCK_DESTROY_DIRECTION_BELOW: u16 = 0;
    pub(crate) const BLOCK_DESTROY_DIRECTION_LEFT: u16 = 1;
    pub(crate) const BLOCK_DESTROY_DIRECTION_ABOVE: u16 = 2;
    pub(crate) const BLOCK_DESTROY_DIRECTION_RIGHT: u16 = 3;

    fn rubber_block_base_skin_for_sub_10de4(cell: terrain::Cell) -> Option<u8> {
        if cell.block_type != terrain::BlockType::Rubber {
            return None;
        }

        Some(match cell.skin {
            0x0C | 0x0D => 0x0C,
            _ => 0x0E,
        })
    }

    fn write_rubber_block_skin_for_sub_10de4(
        &mut self,
        col: u16,
        row: u16,
        base_skin: u8,
        skin_offset: u8,
    ) {
        let Some(grid) = self.block_grid.as_mut() else {
            return;
        };
        let cell = grid.cell_at(col, row);
        if cell.block_type != terrain::BlockType::Rubber {
            return;
        }

        // ASM sub_10DE4 writes back into the same Level_Layout cell via
        // sub_11530, preserving the collision/block bits and only toggling the
        // rubber animation skin between the two adjacent frames.
        grid.set_cell_at(
            col,
            row,
            terrain::Cell::from_be_bytes(0xE6, base_skin.saturating_add(skin_offset)),
        );
    }

    fn start_rubber_block_animation_sub_10de4(&mut self, col: u16, row: u16) {
        let Some(grid) = self.block_grid.as_ref() else {
            return;
        };
        let Some(base_skin) = Self::rubber_block_base_skin_for_sub_10de4(grid.cell_at(col, row))
        else {
            return;
        };

        if let Some(existing) = self
            .block_state
            .rubber_block_animations
            .iter_mut()
            .find(|entry| entry.col == col && entry.row == row)
        {
            existing.base_skin = base_skin;
            existing.phase = 0;
            existing.frames_until_next_write = Self::RUBBER_BLOCK_SUB_10DE4_SEQUENCE[0].1;
        } else {
            self.block_state
                .rubber_block_animations
                .push(RubberBlockAnimation {
                    col,
                    row,
                    base_skin,
                    phase: 0,
                    frames_until_next_write: Self::RUBBER_BLOCK_SUB_10DE4_SEQUENCE[0].1,
                });
        }

        // ASM sub_10DE4 writes the `base+1` frame immediately before the first
        // hibernate, so the contact frame already shows the jiggled rubber.
        self.write_rubber_block_skin_for_sub_10de4(col, row, base_skin, 1);
    }

    fn step_rubber_block_animations_sub_10de4(&mut self) {
        let mut pending_writes = Vec::new();
        self.block_state
            .rubber_block_animations
            .retain_mut(|entry| {
                if entry.frames_until_next_write > 0 {
                    entry.frames_until_next_write -= 1;
                }
                if entry.frames_until_next_write > 0 {
                    return true;
                }

                entry.phase = entry.phase.saturating_add(1);
                let Some(&(skin_offset, next_delay)) =
                    Self::RUBBER_BLOCK_SUB_10DE4_SEQUENCE.get(usize::from(entry.phase))
                else {
                    return false;
                };
                pending_writes.push((entry.col, entry.row, entry.base_skin, skin_offset));
                entry.frames_until_next_write = next_delay;
                entry.phase + 1 < Self::RUBBER_BLOCK_SUB_10DE4_SEQUENCE.len() as u8
            });

        for (col, row, base_skin, skin_offset) in pending_writes {
            self.write_rubber_block_skin_for_sub_10de4(col, row, base_skin, skin_offset);
        }
    }

    fn offset_block_destroy_effect_origin(direction: u16, x_px: u16, y_px: u16) -> (u16, u16) {
        match direction {
            Self::BLOCK_DESTROY_DIRECTION_RIGHT => (x_px.saturating_add(4), y_px),
            Self::BLOCK_DESTROY_DIRECTION_ABOVE => (x_px, y_px.saturating_sub(4)),
            Self::BLOCK_DESTROY_DIRECTION_LEFT => (x_px.saturating_sub(4), y_px),
            _ => (x_px, y_px.saturating_add(4)),
        }
    }

    pub(crate) fn spawn_smash_block_effect_sub_10eda(
        &mut self,
        x_px: u16,
        y_px: u16,
        direction: u16,
    ) {
        let (x_px, y_px) = Self::offset_block_destroy_effect_origin(direction, x_px, y_px);
        self.block_destroy_effects.push(BlockDestroyEffect {
            // ASM sub_10EDA swaps the smash-block sprite art immediately,
            // nudges it 4px in the impact direction, then hibernates for 2
            // frames before dispatching the shared debris worker.
            x_px,
            y_px,
            remaining: 2,
        });
    }

    pub(crate) fn spawn_ice_block_effect_sub_10f9c(
        &mut self,
        x_px: u16,
        y_px: u16,
        direction: u16,
    ) {
        let (x_px, y_px) = Self::offset_block_destroy_effect_origin(direction, x_px, y_px);
        self.block_destroy_effects.push(BlockDestroyEffect {
            // ASM sub_10F9C mirrors sub_10EDA's directional 4px nudge but uses
            // the ice-block sprite set and a 3-frame hibernate before the
            // shared debris worker runs.
            x_px,
            y_px,
            remaining: 3,
        });
    }

    pub(crate) fn step_evanescent_blocks(&mut self, slice: &OpeningSliceState) {
        // Trigger: if the player is on ground, check the floor cell
        if slice.player.on_ground
            && let Some(grid) = &self.block_grid
        {
            let x_px = (slice.player.x_pos >> 16) as i32;
            let y_px = (slice.player.y_pos >> 16) as i32;
            let probe = terrain::probe_floor(grid, x_px, y_px);
            if probe.on_ground && probe.cell.block_type == terrain::BlockType::Evanescent {
                let col = terrain::pixel_to_cell(x_px).max(0) as u16;
                // The floor cell is the one the player is standing ON (below feet)
                let row = terrain::pixel_to_cell(probe.floor_y_px).max(0) as u16;
                let already_tracked = self
                    .block_state
                    .evanescent_blocks
                    .iter()
                    .any(|e| e.col == col && e.row == row);
                if !already_tracked
                    && self.block_state.evanescent_blocks.len() < EVANESCENT_MAX_ENTRIES
                {
                    let initial_timer = i16::from(probe.cell.skin) - EVANESCENT_SKIN_OFFSET;
                    self.block_state.evanescent_blocks.push(EvanescentEntry {
                        col,
                        row,
                        timer: initial_timer,
                    });
                }
            }
        }

        // Tick timers and remove expired blocks
        if let Some(grid) = &mut self.block_grid {
            self.block_state.evanescent_blocks.retain_mut(|entry| {
                entry.timer += 1;
                if entry.timer >= EVANESCENT_REMOVAL_THRESHOLD {
                    grid.set_cell_at(entry.col, entry.row, terrain::Cell::from_be_bytes(0, 0));
                    false
                } else {
                    true
                }
            });
        }
    }

    /// Ghost blocks: tick timers, toggle solid/vanished, update block grid.
    pub(crate) fn step_ghost_blocks(&mut self) {
        let Some(grid) = &mut self.block_grid else {
            return;
        };
        for group in &mut self.block_state.ghost_groups {
            if group.timer > 0 {
                group.timer -= 1;
                continue;
            }
            // Transition
            group.solid = !group.solid;
            group.timer = if group.solid {
                group.solid_time
            } else {
                group.vanished_time
            };

            // Update grid cells for each block in the group
            for i in 0..group.count {
                let (col, row) = if group.vertical {
                    (group.col, group.row + u16::from(i))
                } else {
                    (group.col + u16::from(i), group.row)
                };
                if group.solid {
                    // Restore solid ghost block (collision type 6 = Solid, block type 3 = SolidGhost)
                    let hi = 0x80 | (0x06 << 4) | 0x03;
                    grid.set_cell_at(col, row, terrain::Cell::from_be_bytes(hi, 0));
                } else {
                    // Clear the cell (vanished)
                    grid.set_cell_at(col, row, terrain::Cell::from_be_bytes(0, 0));
                }
            }
        }
    }

    /// When the player is standing on a Shifting block and the cell above is empty,
    /// move the block up one cell.
    /// Shooter blocks (type 0x8): fire projectiles in directions encoded in the
    /// skin byte (bit 0=up, 1=right, 2=down, 3=left). Speed: 4px/frame.
    /// ASM loc_EB72 -> LoadBulletsFromEvanescentBlock registers each visible
    /// shooter once, fires on age 0, then frees the slot again at age $16.
    const SHOOTER_FIRE_INTERVAL: u16 = 22;
    const SHOOTER_PROJECTILE_SPEED: i32 = 4 << 16; // 4px/frame in FP16.16
    const SHOOTER_HEADBUTT_HOLD_FRAMES: u8 = 8;

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_slot_tick_sub_F730(
        timer: u16,
        state: u16,
        active_delay: u16,
        reload_delay: u16,
    ) -> (u16, u16) {
        // ASM sub_F730 (kid.asm:20400): decrement the slot timer first, then
        // advance the visible-state word through the `0 -> $8001 -> $8002 ->
        // 3` and `n -> $8004 -> $8005 -> 0` countdown ladders.
        let decremented_timer = timer.wrapping_sub(1);
        if decremented_timer != 0 {
            return (decremented_timer, state);
        }

        if state & 0x8000 != 0 {
            return match state {
                0x8001 | 0x8004 => (decremented_timer.wrapping_add(2), state.wrapping_add(1)),
                0x8002 => (reload_delay, 3),
                _ => (active_delay, 0),
            };
        }

        if state == 0 {
            (decremented_timer.wrapping_add(2), 0x8001)
        } else {
            (decremented_timer.wrapping_add(2), 0x8004)
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_player_in_range_sub_F536(
        player_x_pos: u32,
        player_y_pos: u32,
        shooter_col: u16,
        shooter_row: u16,
    ) -> bool {
        // ASM sub_F536 (kid.asm:20167): convert Kid to block-space, then accept
        // the shooter when `kid_x_block-4 .. kid_x_block+4` contains the
        // source col and `kid_y_block-6 .. kid_y_block+3` contains the source
        // row. The comparisons are inclusive because the ROM uses `bgt`/`bmi`
        // after subtracting the shooter coordinates.
        let mut delta_x = ((player_x_pos >> 16) as i32 >> 4) - 4 - i32::from(shooter_col);
        if delta_x > 0 {
            return false;
        }
        delta_x += 8;
        if delta_x < 0 {
            return false;
        }

        let mut delta_y = ((player_y_pos >> 16) as i32 >> 4) - 6 - i32::from(shooter_row);
        if delta_y > 0 {
            return false;
        }
        delta_y += 9;
        delta_y >= 0
    }

    fn shooter_deleted_tile_for_sub_114c0(skin: u8) -> terrain::Cell {
        terrain::Cell::from_be_bytes(0xEF, skin)
    }

    fn shooter_support_tile_for_sub_114f4() -> terrain::Cell {
        terrain::Cell::from_be_bytes(0xE9, 0x19)
    }

    /// Tick prize-block bounce lifecycles. ASM sub_1007A holds 0x6F, briefly
    /// restores the original prize cell, then writes `$E001` and creates the
    /// prize object.
    pub(crate) fn step_deferred_ceiling_clears(&mut self) {
        if let Some(grid) = &mut self.block_grid {
            self.block_state
                .deferred_ceiling_clears
                .retain_mut(|entry| match entry.phase {
                    DeferredPrizeBlockPhase::Deleted => {
                        if entry.deleted_frames_remaining == 0 {
                            grid.set_cell_at(entry.col, entry.row, entry.restore_cell);
                            entry.phase = DeferredPrizeBlockPhase::Restored;
                        } else {
                            entry.deleted_frames_remaining -= 1;
                        }
                        true
                    }
                    DeferredPrizeBlockPhase::Restored => {
                        entry.phase = DeferredPrizeBlockPhase::PendingClear;
                        true
                    }
                    DeferredPrizeBlockPhase::PendingClear | DeferredPrizeBlockPhase::FreshSmash => {
                        true
                    }
                });
        }
    }

    pub(crate) fn finish_deferred_ceiling_clears(&mut self) {
        if self.block_grid.is_none() {
            return;
        }

        let mut spawned_items = Vec::new();
        let mut retained_entries = Vec::new();
        for mut entry in std::mem::take(&mut self.block_state.deferred_ceiling_clears) {
            match entry.phase {
                DeferredPrizeBlockPhase::Deleted | DeferredPrizeBlockPhase::Restored => {
                    retained_entries.push(entry);
                }
                DeferredPrizeBlockPhase::PendingClear => {
                    // ASM sub_1007A line 21411: after prize spawns, restore
                    // cell to $E001 (Smash block, skin=1). The player can
                    // then smash this block on a second headbutt.
                    if let Some(grid) = self.block_grid.as_mut() {
                        grid.set_cell_at(
                            entry.col,
                            entry.row,
                            terrain::Cell::from_be_bytes(0xE0, 0x01),
                        );
                    }
                    if Self::is_helmet_prize(entry.content) {
                        self.clear_existing_helmet_prizes_for_sub_10cec();
                    }
                    spawned_items.push(FloatingItem {
                        x_pos: (u32::from(entry.col) * 16 + 8) << 16,
                        y_pos: i64::from(u32::from(entry.row) * 16) << 16,
                        y_velocity: 0,
                        remaining: Self::floating_item_lifetime(entry.content),
                        spawn_delay: Self::PRIZE_ITEM_INITIAL_DORMANT_FRAMES,
                        level_col: entry.col,
                        level_row: entry.row,
                        first_row_grace: true,
                        content: entry.content,
                        collected: false,
                    });
                    // ASM sub_7196 creates Kid with priority #1, while
                    // sub_1007A runs as object #$2001 and spawns Prize_Init
                    // at #$1FFF. Kid therefore finishes this frame before the
                    // prize-block coroutine writes $E001, so the freshly
                    // restored Smash cell must stay non-blocking for Kid
                    // until the next frame ends.
                    entry.phase = DeferredPrizeBlockPhase::FreshSmash;
                    retained_entries.push(entry);
                }
                DeferredPrizeBlockPhase::FreshSmash => {}
            }
        }
        self.block_state.deferred_ceiling_clears = retained_entries;
        self.floating_items.extend(spawned_items);
    }

    pub(crate) fn step_shooter_blocks(&mut self, slice: &OpeningSliceState) {
        let Some(grid) = &self.block_grid else {
            return;
        };

        let kid_block_col = ((slice.player.x_pos >> 16) >> 4) as i16;
        let kid_block_row = ((slice.player.y_pos >> 16) >> 4) as i16;
        let max_col = i16::try_from(grid.width_cells.saturating_sub(1))
            .expect("block grids should fit in i16");
        let max_row = i16::try_from(grid.height_cells.saturating_sub(1))
            .expect("block grids should fit in i16");
        let previous_col = self.block_state.shooter_last_scanned_col;
        let previous_row = self.block_state.shooter_last_scanned_row;
        let mut pending_entries = Vec::new();
        let mut queue_scan = |col: u16, row: u16, skin: u8| {
            let already_registered = self
                .block_state
                .shooter_blocks
                .iter()
                .any(|entry| entry.col == col && entry.row == row);
            let already_pending = pending_entries
                .iter()
                .any(|(pending_col, pending_row, _)| *pending_col == col && *pending_row == row);
            if !already_registered && !already_pending {
                pending_entries.push((col, row, skin));
            }
        };

        if previous_col != kid_block_col {
            let entering_col = if previous_col < kid_block_col {
                (kid_block_col + 3).min(max_col)
            } else {
                (kid_block_col - 3).max(0)
            } as u16;
            let start_row = (kid_block_row - 5).max(0) as u16;
            let end_row = (kid_block_row + 2).min(max_row) as u16;
            for row in start_row..=end_row {
                let cell = grid.cell_at(entering_col, row);
                if cell.has_block && cell.block_type == terrain::BlockType::Shooter {
                    queue_scan(entering_col, row, cell.skin);
                }
            }
        }

        if previous_row != kid_block_row {
            let entering_row = if previous_row < kid_block_row {
                (kid_block_row + 2).min(max_row)
            } else {
                (kid_block_row - 5).max(0)
            } as u16;
            let start_col = (kid_block_col - 3).max(0) as u16;
            let end_col = (kid_block_col + 3).min(max_col) as u16;
            for col in start_col..=end_col {
                let cell = grid.cell_at(col, entering_row);
                if cell.has_block && cell.block_type == terrain::BlockType::Shooter {
                    queue_scan(col, entering_row, cell.skin);
                }
            }
        }

        self.block_state.shooter_last_scanned_col = kid_block_col;
        self.block_state.shooter_last_scanned_row = kid_block_row;
        self.block_state
            .shooter_blocks
            .extend(
                pending_entries
                    .into_iter()
                    .map(|(col, row, directions)| ShooterBlockEntry {
                        col,
                        row,
                        directions,
                        age: 0,
                    }),
            );

        self.block_state.shooter_blocks.retain_mut(|entry| {
            let still_shooter = matches!(
                grid.cell_at(entry.col, entry.row),
                terrain::Cell {
                    has_block: true,
                    block_type: terrain::BlockType::Shooter,
                    ..
                }
            );
            if !still_shooter {
                return false;
            }

            if entry.age == 0 {
                let block_x = u32::from(entry.col) * 16 + 8;
                let block_y = u32::from(entry.row) * 16 + 8;
                let directions: [(u8, i32, i32); 4] = [
                    (1, 0, -Self::SHOOTER_PROJECTILE_SPEED),
                    (2, Self::SHOOTER_PROJECTILE_SPEED, 0),
                    (4, 0, Self::SHOOTER_PROJECTILE_SPEED),
                    (8, -Self::SHOOTER_PROJECTILE_SPEED, 0),
                ];
                for (mask, xv, yv) in directions {
                    if entry.directions & mask != 0 {
                        self.projectiles.push(EnemyProjectile {
                            x_pos: block_x << 16,
                            y_pos: block_y << 16,
                            x_velocity: xv,
                            y_velocity: yv,
                            lifetime: 120,
                            gravity: 0,
                            friendly: false,
                            homing_accel: 0,
                            speed_cap: 0,
                            wall_bounce: false,
                            ..Default::default()
                        });
                    }
                }
            }

            entry.age += 1;
            entry.age < Self::SHOOTER_FIRE_INTERVAL
        });
    }

    pub(crate) fn step_shooter_headbutt_workers(&mut self) {
        let Some(grid) = &mut self.block_grid else {
            self.block_state.shooter_headbutt_workers.clear();
            return;
        };

        self.block_state
            .shooter_headbutt_workers
            .retain_mut(|worker| {
                match worker.kind {
                    ShooterHeadbuttWorkerKind::Center => match worker.age {
                        0 => {
                            // ASM sub_114C0: first execution writes `$EF1B`.
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                Self::shooter_deleted_tile_for_sub_114c0(0x1B),
                            );
                        }
                        Self::SHOOTER_HEADBUTT_HOLD_FRAMES => {
                            // ASM sub_114C0: after one 8-frame hibernate, write `$EF1A`.
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                Self::shooter_deleted_tile_for_sub_114c0(0x1A),
                            );
                        }
                        age if age == Self::SHOOTER_HEADBUTT_HOLD_FRAMES * 2 => {
                            // ASM sub_114C0: after the second 8-frame hibernate, erase the cell.
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                terrain::Cell::from_be_bytes(0, 0),
                            );
                            return false;
                        }
                        _ => {}
                    },
                    ShooterHeadbuttWorkerKind::Support => match worker.age {
                        0 => {
                            // ASM sub_114F4 bails if the target cell is no longer empty when
                            // the worker first wakes.
                            if grid.cell_at(worker.col, worker.row)
                                != terrain::Cell::from_be_bytes(0, 0)
                            {
                                return false;
                            }
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                Self::shooter_deleted_tile_for_sub_114c0(0x1A),
                            );
                        }
                        Self::SHOOTER_HEADBUTT_HOLD_FRAMES => {
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                Self::shooter_deleted_tile_for_sub_114c0(0x1B),
                            );
                        }
                        age if age == Self::SHOOTER_HEADBUTT_HOLD_FRAMES * 2 => {
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                Self::shooter_support_tile_for_sub_114f4(),
                            );
                            return false;
                        }
                        _ => {}
                    },
                }

                worker.age = worker.age.saturating_add(1);
                true
            });
    }

    pub(crate) fn apply_shifting_blocks(&mut self, slice: &OpeningSliceState) {
        if !slice.player.on_ground {
            return;
        }
        let Some(grid) = &mut self.block_grid else {
            return;
        };

        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        // The floor cell is just below the player's feet
        let col = terrain::pixel_to_cell(x_px).max(0) as u16;
        let foot_row = terrain::pixel_to_cell(y_px + 1).max(0) as u16;

        let cell = grid.cell_at(col, foot_row);
        if cell.block_type != terrain::BlockType::Shifting {
            return;
        }
        if foot_row == 0 {
            return; // no cell above to shift into
        }

        let above = grid.cell_at(col, foot_row - 1);
        // ASM only checks `btst #6` (has_block bit) — no collision check.
        if above.has_block {
            return; // cell above is occupied
        }

        // Move block up: copy cell to row above, clear current cell
        grid.set_cell_at(col, foot_row - 1, cell);
        grid.set_cell_at(col, foot_row, terrain::Cell::from_be_bytes(0, 0));
    }

    pub(crate) fn step_lift_platforms(&mut self, slice: &mut OpeningSliceState) {
        self.block_state.lift_platforms.retain_mut(|lift| {
            if !lift.block_erased {
                if let Some(grid) = &mut self.block_grid {
                    grid.set_cell_at(
                        lift.source_col,
                        lift.source_row,
                        terrain::Cell::from_be_bytes(0, 0),
                    );
                }
                lift.block_erased = true;
            }

            if !Self::step_lift_platform_sub_FCA6(lift, slice.level_bounds.height_pixels) {
                return false;
            }

            // Platform collision: player can stand on lift
            let plat_x_px = (lift.x_pos >> 16) as i32;
            let plat_y_px = (lift.y_pos >> 16) as i32;
            let player_x_px = (slice.player.x_pos >> 16) as i32;
            let player_foot_y_px = (slice.player.y_pos >> 16) as i32;
            let player_left = player_x_px - PLAYER_HALF_WIDTH;
            let player_right = player_x_px + PLAYER_HALF_WIDTH;
            let plat_right = plat_x_px + i32::from(lift.width_px);

            if player_right >= plat_x_px
                && player_left <= plat_right
                && slice.player.y_velocity >= 0
                && !slice.player.on_ground
            {
                let foot_dist = player_foot_y_px - plat_y_px;
                if (-2..=4).contains(&foot_dist) {
                    slice.player.y_pos = lift.y_pos;
                    slice.player.y_velocity = lift.y_velocity;
                    slice.player.on_ground = true;
                    slice.terrain.floor_y_pos = lift.y_pos;
                }
            }

            // If player is on ground and riding this lift, carry along
            if slice.player.on_ground {
                let within_snap =
                    slice.player.y_pos.abs_diff(lift.y_pos) <= GROUND_SNAP_TOLERANCE_FP;
                if within_snap && player_right >= plat_x_px && player_left <= plat_right {
                    slice.player.y_pos = (i64::from(slice.player.y_pos)
                        + i64::from(lift.y_velocity))
                    .clamp(0, i64::from(u32::MAX)) as u32;
                    slice.terrain.floor_y_pos =
                        (((slice.player.y_pos >> 16) as i32).max(0) as u32) << 16;
                }
            }

            true
        });
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn lift_block_bob_deltas_sub_FB3E() -> &'static [i16] {
        // ASM sub_FB3E (kid.asm:20905): before the lift platform spawns, the
        // block bobs in three hibernating up/down passes: 6, 5, then 4/5
        // frames at 2px per frame.
        &[
            -2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, 2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, -2, -2,
            -2, -2, 2, 2, 2, 2, 2,
        ]
    }

    /// Process block hit effects from the current frame's slice (prizes, bounces,
    /// lifts, drills, teleporter warps).
    pub(crate) fn process_block_effects(&mut self, slice: &mut OpeningSliceState) {
        // Prize block hit: spawn the immediate debris effect. The actual prize
        // object is created later by the queued sub_1007A lifecycle.
        if let Some((_content, x_px, y_px)) = slice.prize_block_hit {
            self.blocks_hit_this_frame += 1;
            self.block_destroy_effects.push(BlockDestroyEffect {
                x_px,
                y_px,
                remaining: 8,
            });
        }

        // Metal block bounce animation
        if let Some((cx, cy)) = slice.metal_block_hit {
            self.blocks_hit_this_frame += 1;
            self.block_bounces.push(BlockBounceAnim {
                cell_x: cx,
                cell_y: cy,
                remaining: 8,
            });
        }

        // ASM loc_10DA4: rubber contact spawns a temporary jiggle object and
        // plays sfx_Rubber_block without changing the collision response.
        if let Some((cx, cy)) = slice.rubber_block_hit {
            self.blocks_hit_this_frame += 1;
            self.start_rubber_block_animation_sub_10de4(cx, cy);
            self.block_bounces.push(BlockBounceAnim {
                cell_x: cx,
                cell_y: cy,
                remaining: 8,
            });
        }

        // Smash block destruction effect
        if let Some((x_px, y_px)) = slice.smash_block_hit {
            self.blocks_hit_this_frame += 1;
            self.spawn_smash_block_effect_sub_10eda(
                x_px,
                y_px,
                Self::BLOCK_DESTROY_DIRECTION_BELOW,
            );
        }

        if let Some((cell_x, cell_y)) = slice.shooter_block_hit
            && let Some(grid) = self.block_grid.as_mut()
        {
            self.blocks_hit_this_frame += 1;
            let shooter_cell = grid.cell_at(cell_x, cell_y);
            // ASM loc_11430 marks the struck shooter as deleted-solid
            // immediately via `or.b #$F,(a1)` before the timed workers run.
            grid.set_cell_at(
                cell_x,
                cell_y,
                Self::shooter_deleted_tile_for_sub_114c0(shooter_cell.skin),
            );

            self.block_state
                .shooter_headbutt_workers
                .push(ShooterHeadbuttWorker {
                    col: cell_x,
                    row: cell_y,
                    age: 0,
                    kind: ShooterHeadbuttWorkerKind::Center,
                });

            if cell_y > 0 {
                let support_row = cell_y - 1;
                let mut try_queue_support = |support_col: u16| {
                    if grid.cell_at(support_col, support_row) == terrain::Cell::from_be_bytes(0, 0)
                    {
                        // ASM sub_1149C: queue one support worker per empty cell above the
                        // struck shooter, including the diagonals when in bounds.
                        self.block_state
                            .shooter_headbutt_workers
                            .push(ShooterHeadbuttWorker {
                                col: support_col,
                                row: support_row,
                                age: 0,
                                kind: ShooterHeadbuttWorkerKind::Support,
                            });
                    }
                };

                try_queue_support(cell_x);
                if cell_x > 0 {
                    try_queue_support(cell_x - 1);
                }
                if cell_x + 1 < grid.width_cells {
                    try_queue_support(cell_x + 1);
                }
            }
        }

        // Lift block: headbutt spawns a rising platform (ASM sub_113C0)
        if let Some((x_px, y_px)) = slice.lift_block_hit {
            let col = x_px / 16;
            let row = y_px / 16;
            self.block_state.lift_platforms.push(LiftPlatform {
                x_pos: u32::from(x_px) << 16,
                y_pos: u32::from(y_px) << 16,
                y_velocity: 0,
                age: 0,
                last_integer_y: y_px,
                moved_y_steps: 0,
                width_px: 16,
                source_col: col,
                source_row: row,
                block_erased: false,
            });
        }

        // Drill block: headbutt spawns a horizontal projectile (ASM sub_1129C: ±4px/frame)
        if let Some((x_fp, y_fp)) = slice.drill_projectile_hit {
            // Direction based on player position relative to drill block
            let dir = if slice.player.x_pos >= x_fp { 1i32 } else { -1 };
            self.projectiles.push(EnemyProjectile {
                x_pos: x_fp,
                y_pos: y_fp,
                x_velocity: dir * 0x4_0000, // 4 px/frame horizontal
                y_velocity: 0,
                lifetime: 120,
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }

        // Teleporter block: warp player to destination
        if let Some((cell_x, cell_y)) = slice.teleporter_contact
            && let Some(block_list) = self.block_list.as_ref()
            && let Some(tp) = block_list.teleporter_at(cell_x, cell_y)
        {
            // Warp to destination position
            slice.player.x_pos = (u32::from(tp.dest_x) * 16) << 16;
            slice.player.y_pos = (u32::from(tp.dest_y) * 16) << 16;
            slice.player.y_velocity = 0;
            slice.player.on_ground = true;
        }
    }

    /// Check player overlap with floating prize items and collect on contact.
    /// ASM spawns prize items as separate objects — the player must physically
    /// touch them. The native previously applied prizes immediately at block-hit
    /// time, causing spurious form changes.
    /// Check player overlap with floating prize items and collect on contact.
    /// ASM spawns prize items as separate objects — the player must physically
    /// touch them. The native previously applied prizes immediately at block-hit
    /// time, causing spurious form changes.
    /// Check player overlap with floating prize items and collect on contact.
    /// ASM uses GfxObjects_Collision (collision_type field) to detect player
    /// contact each frame. The collision check uses the standard sprite bounding
    /// box overlap from the object system.
    pub(crate) fn collect_floating_items(&mut self, slice: &mut OpeningSliceState) {
        use crate::constants::{PLAYER_HALF_WIDTH, PLAYER_HEIGHT};
        const PLAYER_PRIZE_PICKUP_HEIGHT: i32 = (PLAYER_HEIGHT + 1) / 2;

        let player_x_px = (slice.player.x_pos >> 16) as i32;
        let player_y_px = (slice.player.y_pos >> 16) as i32;
        let player_left = player_x_px - PLAYER_HALF_WIDTH;
        let player_right = player_x_px + PLAYER_HALF_WIDTH;

        let mut collected_contents = Vec::new();
        let mut collected_prize_diamonds = 0usize;
        for item in &mut self.floating_items {
            if item.collected || item.spawn_delay > 0 {
                continue;
            }
            let item_x_px = (item.x_pos >> 16) as i32;
            let item_y_px = (item.y_pos >> 16) as i32;
            let (player_top, item_left, item_right, item_top, item_bottom, inclusive_overlap) =
                if item.content == terrain::PrizeContent::Diamond {
                    // PrizeDiamond_Init uses the standard gfx-object collision path,
                    // with `y_pos` anchored at the object's bottom edge rather than
                    // the top of a tiny 16px sprite box.
                    (
                        player_y_px - PLAYER_HEIGHT,
                        item_x_px - 7,
                        item_x_px + 7,
                        item_y_px - PLAYER_HEIGHT,
                        item_y_px,
                        true,
                    )
                } else {
                    // Oracle demo 0x1B keeps falling helmets non-collectible until
                    // they overlap Kid's lower body, not the full 31px sprite box.
                    (
                        player_y_px - PLAYER_PRIZE_PICKUP_HEIGHT,
                        item_x_px - 8,
                        item_x_px + 8,
                        item_y_px,
                        item_y_px + 16,
                        false,
                    )
                };

            let overlap_x = if inclusive_overlap {
                player_left <= item_right && player_right >= item_left
            } else {
                player_left < item_right && player_right > item_left
            };
            let overlap_y = if inclusive_overlap {
                player_top <= item_bottom && player_y_px >= item_top
            } else {
                player_top < item_bottom && player_y_px > item_top
            };

            if overlap_x && overlap_y {
                // ASM loc_1000A / loc_1001E (kid.asm:21302-21312) hibernate
                // the collected clock object and keep recomputing `x_pos/y_pos`
                // from the cached camera-relative words at `$3E/$42` until the
                // final delete. Native does not model that lingering sprite, so
                // collection only flips the removal flag and leaves the cached
                // coordinates untouched until `step_visual_effects` removes it.
                item.collected = true;
                if item.content == terrain::PrizeContent::Diamond {
                    collected_prize_diamonds += 1;
                } else {
                    collected_contents.push(item.content);
                }
            }
        }
        for _ in 0..collected_prize_diamonds {
            self.queue_prize_diamond_award();
        }
        for content in collected_contents {
            self.apply_prize_content(content);
        }
    }

    pub(crate) fn step_visual_effects(&mut self) {
        self.step_rubber_block_animations_sub_10de4();

        let floating_item_can_fall = |grid: &terrain::BlockGrid, item: &FloatingItem| {
            let current_cell = grid.cell_at(item.level_col, item.level_row);
            let current_row_open = matches!(
                current_cell.collision,
                terrain::CollisionKind::Empty | terrain::CollisionKind::HiddenBlock
            );
            if current_row_open {
                return true;
            }
            // ASM loc_103B2 (kid.asm:21630): sample the current row first and
            // stop immediately on solid rows unless the bit-16 grace flag is
            // still set. When that grace bit is present, the routine checks
            // one row below and keeps falling only if that second sample is
            // passable too.
            if !item.first_row_grace {
                return false;
            }
            let below_cell = grid.cell_at(item.level_col, item.level_row.saturating_add(1));
            matches!(
                below_cell.collision,
                terrain::CollisionKind::Empty | terrain::CollisionKind::HiddenBlock
            )
        };

        for b in &mut self.block_bounces {
            b.remaining = b.remaining.saturating_sub(1);
        }
        self.block_bounces.retain(|b| b.remaining > 0);

        for e in &mut self.block_destroy_effects {
            e.remaining = e.remaining.saturating_sub(1);
        }
        self.block_destroy_effects.retain(|e| e.remaining > 0);

        for f in &mut self.floating_items {
            if f.spawn_delay > 0 {
                f.spawn_delay -= 1;
                continue;
            }
            // ASM loc_10396 (kid.asm:21621): each hibernated frame decrements
            // `d3`, deletes the object when the countdown reaches zero, and
            // switches to `stru_10D80` exactly when `d3 == $00C8`. Native
            // floating items do not carry animation-script state, so this loop
            // models the shared countdown/delete timing and leaves the visual
            // script swap implicit.
            f.remaining = f.remaining.saturating_sub(1);
            let can_fall = self
                .block_grid
                .as_ref()
                .is_none_or(|grid| floating_item_can_fall(grid, f));
            if can_fall {
                let previous_aligned_y = ((f.y_pos >> 16) as i32) & !0xF;
                // ASM: velocity accumulates +$1388/frame (downward), applied to position.
                // PrizeDiamond_Init (kid.asm:21640) and sub_10C38.
                f.y_velocity = f.y_velocity.saturating_add(0x1388);
                f.y_pos += i64::from(f.y_velocity);
                let new_aligned_y = ((f.y_pos >> 16) as i32) & !0xF;
                if new_aligned_y != previous_aligned_y {
                    // ASM loc_10402 (kid.asm:21663): once the snapped 16px row
                    // changes, copy the new aligned Y into `d0` so later
                    // frames compare against the updated row baseline.
                    f.first_row_grace = false;
                    f.level_row = f.level_row.saturating_add(1);
                }
            } else {
                // ASM loc_103DA (kid.asm:21645): `moveq #0,d1` feeds the
                // shared tail at `loc_10404`, so blocked floating prizes
                // cancel the pending vertical delta for this frame.
                f.y_velocity = 0;
            }
        }
        self.floating_items
            .retain(|f| (f.spawn_delay > 0 || f.remaining > 0) && !f.collected);

        for e in &mut self.enemy_death_effects {
            e.remaining = e.remaining.saturating_sub(1);
        }
        self.enemy_death_effects.retain(|e| e.remaining > 0);
    }
}
