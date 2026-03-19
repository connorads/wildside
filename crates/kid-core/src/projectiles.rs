use super::*;

impl NativeRuntime {
    /// Advance all active projectiles: move by velocity, decrement lifetime, remove expired.
    /// Homing projectiles accelerate toward `player_pos` each frame.
    /// Wall-bounce projectiles (Lion) reverse velocity on terrain collision (Lion.asm loc_3E260).
    pub(crate) fn step_projectiles(&mut self) {
        let player_x = self.opening_slice.as_ref().map(|s| s.player.x_pos);
        let player_y = self.opening_slice.as_ref().map(|s| s.player.y_pos);
        let grid = self.block_grid.as_ref();
        self.projectiles.retain_mut(|p| {
            if p.lifetime == 0 {
                return false;
            }
            p.lifetime -= 1;
            p.y_velocity += p.gravity;
            // Homing: accelerate toward player (per-projectile accel from ASM).
            if p.homing_accel != 0 {
                if let (Some(px), Some(py)) = (player_x, player_y) {
                    let dx = px as i64 - p.x_pos as i64;
                    // Lion.asm loc_3E20E: Y homing target is 16px ($10) above player
                    let dy = (py as i64 - (0x10 << 16)) - p.y_pos as i64;
                    p.x_velocity += if dx > 0 {
                        p.homing_accel
                    } else {
                        -p.homing_accel
                    };
                    p.y_velocity += if dy > 0 {
                        p.homing_accel
                    } else {
                        -p.homing_accel
                    };
                }
                // Speed cap (Lion.asm: ±$60000 on each axis)
                if p.speed_cap > 0 {
                    p.x_velocity = p.x_velocity.clamp(-p.speed_cap, p.speed_cap);
                    p.y_velocity = p.y_velocity.clamp(-p.speed_cap, p.speed_cap);
                }
            }
            p.x_pos = (p.x_pos as i64 + i64::from(p.x_velocity)) as u32;
            p.y_pos = (p.y_pos as i64 + i64::from(p.y_velocity)) as u32;

            // Wall-bounce: reverse velocity on terrain contact (Lion.asm loc_3E260).
            // ASM checks collision_type and bounces up to 15 times per frame.
            // Native: check if projectile centre is inside a solid cell after moving.
            if p.wall_bounce
                && let Some(g) = grid
            {
                let px_i = (p.x_pos >> 16) as i32;
                let py_i = (p.y_pos >> 16) as i32;
                let col = terrain::pixel_to_cell(px_i).max(0) as u16;
                let row = terrain::pixel_to_cell(py_i).max(0) as u16;
                let cell = g.cell_at(col, row);
                if cell.has_block && cell.collision != terrain::CollisionKind::Empty {
                    // Undo the move
                    p.x_pos = (p.x_pos as i64 - i64::from(p.x_velocity)) as u32;
                    p.y_pos = (p.y_pos as i64 - i64::from(p.y_velocity)) as u32;

                    // Determine bounce axis: probe horizontally and vertically
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

                    if h_blocked && v_blocked {
                        // Corner: reverse both (ASM swaps velocities)
                        p.x_velocity = -p.x_velocity;
                        p.y_velocity = -p.y_velocity;
                    } else if h_blocked {
                        // Horizontal wall: negate x (ASM loc_3E2A4)
                        p.x_velocity = -p.x_velocity;
                    } else {
                        // Vertical surface (floor/ceiling) or slope: negate y (ASM loc_3E2C8)
                        p.y_velocity = -p.y_velocity;
                    }

                    // Re-apply movement with bounced velocity
                    p.x_pos = (p.x_pos as i64 + i64::from(p.x_velocity)) as u32;
                    p.y_pos = (p.y_pos as i64 + i64::from(p.y_velocity)) as u32;
                }
            }
            true
        });
    }

    /// Check if any projectile overlaps the player hitbox, returning true if hit.
    pub(crate) fn projectile_hits_player(&mut self, slice: &OpeningSliceState) -> bool {
        let px = (slice.player.x_pos >> 16) as i32;
        let py = (slice.player.y_pos >> 16) as i32;
        let half = PROJECTILE_HITBOX_HALF;
        let mut hit = false;
        self.projectiles.retain(|p| {
            if p.friendly {
                return true; // player-fired projectiles don't damage player
            }
            let prx = (p.x_pos >> 16) as i32;
            let pry = (p.y_pos >> 16) as i32;
            let overlaps = (prx - px).abs() < half + 8 && (pry - py).abs() < half + 16;
            if overlaps {
                hit = true;
                false // remove projectile on hit
            } else {
                true
            }
        });
        hit
    }
}
