use super::*;
use crate::enemies::{archer_adjust_angle, archer_angle_to_player, arrow_velocity_from_angle};

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidProjectileCollisionPassSub1D76 {
    pub(crate) object_list_head_address: u32,
    pub(crate) collision_mode: u8,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn kid_projectile_collision_passes_sub_1D76() -> [KidProjectileCollisionPassSub1D76; 2] {
    // ASM sub_1D76 (kid.asm:3537): dispatch Kid projectile collision through
    // sub_1D84 twice, first with list head `$FFFFF862` and `d7=0`, then with
    // list head `$FFFFF5A0` and `d7=1` before falling through into sub_1D84.
    [
        KidProjectileCollisionPassSub1D76 {
            object_list_head_address: 0x00FF_F862,
            collision_mode: 0,
        },
        KidProjectileCollisionPassSub1D76 {
            object_list_head_address: 0x00FF_F5A0,
            collision_mode: 1,
        },
    ]
}

impl NativeRuntime {
    fn is_juggernaut_skull_projectile(projectile: &EnemyProjectile) -> bool {
        projectile.friendly
            && projectile.gravity == JUGGERNAUT_SKULL_GRAVITY
            && projectile.homing_accel == 0
            && projectile.speed_cap == 0
            && !projectile.terrain_kill
    }

    fn solid_terrain_at(grid: &terrain::BlockGrid, x_px: i32, y_px: i32) -> bool {
        let col = terrain::pixel_to_cell(x_px).max(0) as u16;
        let row = terrain::pixel_to_cell(y_px).max(0) as u16;
        let cell = grid.cell_at(col, row);
        cell.has_block && cell.collision != terrain::CollisionKind::Empty
    }

    #[allow(non_snake_case)]
    fn step_juggernaut_skull_sub_87B0(
        projectile: &mut EnemyProjectile,
        grid: Option<&terrain::BlockGrid>,
    ) -> bool {
        let Some(grid) = grid else {
            projectile.x_pos = (projectile.x_pos as i64 + i64::from(projectile.x_velocity)) as u32;
            projectile.y_pos = (projectile.y_pos as i64 + i64::from(projectile.y_velocity)) as u32;
            return true;
        };

        let original_y = projectile.y_pos;
        let x_velocity = projectile.x_velocity;
        let next_x = (i64::from(projectile.x_pos) + i64::from(x_velocity)) as u32;
        projectile.x_pos = next_x;
        let next_x_px = (next_x >> 16) as i32;
        let current_y_px = (original_y >> 16) as i32;

        if Self::solid_terrain_at(grid, next_x_px, current_y_px) {
            let snapped_x_px = if x_velocity >= 0 {
                (next_x_px & !0xF) - 1
            } else {
                (next_x_px & !0xF) + 16
            };
            projectile.x_pos = (snapped_x_px.max(0) as u32) << 16;
            projectile.reaim_countdown = projectile.reaim_countdown.saturating_sub(1);
            if projectile.reaim_countdown == 0 {
                return false;
            }

            let decayed_x_velocity = if x_velocity > 0x1_0000 {
                x_velocity - 0x1_0000
            } else if x_velocity < -0x1_0000 {
                x_velocity + 0x1_0000
            } else {
                x_velocity
            };
            projectile.x_velocity = -decayed_x_velocity;
        }

        let y_velocity = projectile.y_velocity;
        let next_y = (i64::from(projectile.y_pos) + i64::from(y_velocity)) as u32;
        projectile.y_pos = next_y;
        let current_x_px = (projectile.x_pos >> 16) as i32;
        let next_y_px = (next_y >> 16) as i32;
        if Self::solid_terrain_at(grid, current_x_px, next_y_px) {
            let snapped_y_px = if y_velocity >= 0 {
                (next_y_px & !0xF) - 1
            } else {
                (next_y_px & !0xF) + 16
            };
            projectile.y_pos = (snapped_y_px.max(0) as u32) << 16;
            projectile.reaim_countdown = projectile.reaim_countdown.saturating_sub(1);
            if projectile.reaim_countdown == 0 {
                return false;
            }

            // ASM sub_87B0 (kid.asm:9625): on a floor/ceiling hit, negate the
            // current y-velocity then keep 3/4 of that rebound speed.
            let negated = -y_velocity;
            let half = negated >> 1;
            let quarter = half >> 1;
            projectile.y_velocity = half + quarter;
        }

        true
    }

    fn release_lion_projectile_owners(&mut self, owners: Vec<EnemyProjectileOwner>) {
        for owner in owners {
            if let Some(enemy) = self.active_enemies.iter_mut().find(|enemy| {
                enemy.family == EnemyFamily::Lion
                    && enemy.segment_index == owner.segment_index
                    && enemy.layout_index == owner.layout_index
                    && enemy.ufo_beam_interval == owner.token
            }) {
                enemy.ufo_beam_segments = enemy.ufo_beam_segments.saturating_sub(1);
            }
        }
    }

    /// Advance all active projectiles: move by velocity, decrement lifetime, remove expired.
    /// Homing projectiles accelerate toward `player_pos` each frame.
    /// Re-aiming projectiles (Archer arrows) adjust angle ±1 every `reaim_interval` frames.
    /// Wall-bounce projectiles (Lion) reverse velocity on terrain collision (Lion.asm loc_3E260).
    pub(crate) fn step_projectiles(&mut self) {
        let player_x = self.opening_slice.as_ref().map(|s| s.player.x_pos);
        let player_y = self.opening_slice.as_ref().map(|s| s.player.y_pos);
        let camera = self.opening_slice.as_ref().map(|s| s.camera);
        let grid = self.block_grid.as_ref();
        let mut released_owners = Vec::new();
        self.projectiles.retain_mut(|p| {
            if p.lifetime == 0 {
                if let Some(owner) = p.owner {
                    released_owners.push(owner);
                }
                return false;
            }
            p.lifetime -= 1;
            p.y_velocity += p.gravity;

            // Arrow re-aiming: ASM loc_342D6 loop. Every `reaim_interval` frames,
            // compute target angle toward player, adjust current_angle by ±1,
            // then update velocity from the new angle.
            if p.reaim_interval > 0 {
                if p.reaim_countdown == 0 {
                    p.reaim_countdown = p.reaim_interval;
                    if let (Some(px), Some(py)) = (player_x, player_y) {
                        let target = archer_angle_to_player(p.x_pos, p.y_pos, px, py) as u8;
                        p.current_angle = archer_adjust_angle(p.current_angle, target);
                        let (xv, yv) = arrow_velocity_from_angle(p.current_angle as usize);
                        p.x_velocity = xv;
                        p.y_velocity = yv;
                    }
                } else {
                    p.reaim_countdown -= 1;
                }
            }
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
            if Self::is_juggernaut_skull_projectile(p) {
                return Self::step_juggernaut_skull_sub_87B0(p, grid);
            }

            p.x_pos = (p.x_pos as i64 + i64::from(p.x_velocity)) as u32;
            p.y_pos = (p.y_pos as i64 + i64::from(p.y_velocity)) as u32;

            // ASM Lion.asm sub_3E2CE: lion projectiles only survive while the
            // parent token matches and the object stays inside the camera box
            // x=[-0x40,0x180], y=[-0x40,0x120].
            if let (Some(owner), Some(camera)) = (p.owner, camera) {
                let screen_x = ((p.x_pos >> 16) as i32) - ((camera.x_pos >> 16) as i32);
                let screen_y = ((p.y_pos >> 16) as i32) - ((camera.y_pos >> 16) as i32);
                let in_camera_box =
                    (-0x40..=0x180).contains(&screen_x) && (-0x40..=0x120).contains(&screen_y);
                if !in_camera_box {
                    released_owners.push(owner);
                    return false;
                }
            }

            // Terrain-kill: Archer arrows die on level/object collision.
            // ASM loc_342D6: `tst.b $19(a3)` (has_level_collision) / `tst.w collision_type(a3)`.
            if p.terrain_kill
                && let Some(g) = grid
            {
                let px_i = (p.x_pos >> 16) as i32;
                let py_i = (p.y_pos >> 16) as i32;
                let col = terrain::pixel_to_cell(px_i).max(0) as u16;
                let row = terrain::pixel_to_cell(py_i).max(0) as u16;
                let cell = g.cell_at(col, row);
                if cell.has_block && cell.collision != terrain::CollisionKind::Empty {
                    if let Some(owner) = p.owner {
                        released_owners.push(owner);
                    }
                    return false; // destroyed on terrain contact
                }
            }

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
        self.release_lion_projectile_owners(released_owners);
    }

    /// Check if any projectile overlaps the player hitbox, returning true if hit.
    pub(crate) fn projectile_hits_player(&mut self, slice: &OpeningSliceState) -> bool {
        let px = (slice.player.x_pos >> 16) as i32;
        let py = (slice.player.y_pos >> 16) as i32;
        let half = PROJECTILE_HITBOX_HALF;
        let mut hit = false;
        let mut released_owners = Vec::new();
        self.projectiles.retain(|p| {
            if p.friendly {
                return true; // player-fired projectiles don't damage player
            }
            let prx = (p.x_pos >> 16) as i32;
            let pry = (p.y_pos >> 16) as i32;
            let overlaps = (prx - px).abs() < half + 8 && (pry - py).abs() < half + 16;
            if overlaps {
                hit = true;
                if let Some(owner) = p.owner {
                    released_owners.push(owner);
                }
                false // remove projectile on hit
            } else {
                true
            }
        });
        self.release_lion_projectile_owners(released_owners);
        hit
    }
}
