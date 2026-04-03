use super::*;

impl NativeRuntime {
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
        if let Some(grid) = &mut self.block_grid {
            let mut spawned_items = Vec::new();
            self.block_state
                .deferred_ceiling_clears
                .retain_mut(|entry| match entry.phase {
                    DeferredPrizeBlockPhase::Deleted | DeferredPrizeBlockPhase::Restored => true,
                    DeferredPrizeBlockPhase::PendingClear => {
                        // ASM sub_1007A line 21411: after prize spawns, restore
                        // cell to $E001 (Smash block, skin=1). The player can
                        // then smash this block on a second headbutt.
                        grid.set_cell_at(
                            entry.col,
                            entry.row,
                            terrain::Cell::from_be_bytes(0xE0, 0x01),
                        );
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
                        true
                    }
                    DeferredPrizeBlockPhase::FreshSmash => false,
                });
            self.floating_items.extend(spawned_items);
        }
    }

    pub(crate) fn step_shooter_blocks(&mut self, slice: &OpeningSliceState) {
        let Some(grid) = &self.block_grid else {
            return;
        };

        // ASM loc_EB72: scan the visible layout window and register each
        // visible shooter cell at most once while its slot is alive.
        let cam_x = slice.camera.x_pos >> 16;
        let cam_y = slice.camera.y_pos >> 16;
        let start_col = (cam_x / 16).saturating_sub(1) as u16;
        let end_col = ((cam_x + 320) / 16 + 2).min(u32::from(grid.width_cells)) as u16;
        let start_row = (cam_y / 16).saturating_sub(1) as u16;
        let end_row = ((cam_y + 224) / 16 + 2).min(u32::from(grid.height_cells)) as u16;

        self.block_state.shooter_blocks.retain(|entry| {
            entry.col >= start_col
                && entry.col < end_col
                && entry.row >= start_row
                && entry.row < end_row
        });

        for row in start_row..end_row {
            for col in start_col..end_col {
                let cell = grid.cell_at(col, row);
                if !cell.has_block || cell.block_type != terrain::BlockType::Shooter {
                    continue;
                }
                let already_registered = self
                    .block_state
                    .shooter_blocks
                    .iter()
                    .any(|entry| entry.col == col && entry.row == row);
                if !already_registered {
                    self.block_state.shooter_blocks.push(ShooterBlockEntry {
                        col,
                        row,
                        directions: cell.skin,
                        age: 0,
                    });
                }
            }
        }

        self.block_state.shooter_blocks.retain_mut(|entry| {
            let still_visible = entry.col >= start_col
                && entry.col < end_col
                && entry.row >= start_row
                && entry.row < end_row;
            let still_shooter = still_visible
                && matches!(
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

    /// Step lift platforms using ASM coroutine timing (sub_113C0).
    /// Phase 1 (rise): 2px/frame up × 8 frames.
    /// Phase 2 (erase): block erased from grid at frame 8.
    /// Phase 3 (fall): 1px/frame down × 16 frames, then delete.
    pub(crate) fn step_lift_platforms(&mut self, slice: &mut OpeningSliceState) {
        self.block_state.lift_platforms.retain_mut(|lift| {
            lift.age += 1;

            if lift.age <= LIFT_RISE_FRAMES {
                // Phase 1: rise at 2px/frame
                let delta = -(LIFT_RISE_SPEED_PX << 16);
                lift.y_velocity = delta;
                lift.y_pos =
                    (i64::from(lift.y_pos) + i64::from(delta)).clamp(0, i64::from(u32::MAX)) as u32;
            } else if lift.age <= LIFT_RISE_FRAMES + LIFT_FALL_FRAMES {
                // Phase 2: erase block from grid (once, at start of fall phase)
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

                // Phase 3: fall at 1px/frame
                let delta = LIFT_FALL_SPEED_PX << 16;
                lift.y_velocity = delta;
                lift.y_pos =
                    (i64::from(lift.y_pos) + i64::from(delta)).clamp(0, i64::from(u32::MAX)) as u32;
            } else {
                // Phase 4: delete
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
            self.block_bounces.push(BlockBounceAnim {
                cell_x: cx,
                cell_y: cy,
                remaining: 8,
            });
        }

        // Smash block destruction effect
        if let Some((x_px, y_px)) = slice.smash_block_hit {
            self.blocks_hit_this_frame += 1;
            self.block_destroy_effects.push(BlockDestroyEffect {
                x_px,
                y_px,
                remaining: 8,
            });
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
        let floating_item_can_fall = |grid: &terrain::BlockGrid, item: &FloatingItem| {
            let current_cell = grid.cell_at(item.level_col, item.level_row);
            let current_row_open = matches!(
                current_cell.collision,
                terrain::CollisionKind::Empty | terrain::CollisionKind::HiddenBlock
            );
            if current_row_open {
                return true;
            }
            if !item.first_row_grace {
                return false;
            }
            if item.content != terrain::PrizeContent::Diamond {
                return true;
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
                    f.first_row_grace = false;
                    f.level_row = f.level_row.saturating_add(1);
                }
            } else {
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
