use super::*;
#[cfg(test)]
use crate::combat::CollisionSizeEntrySub21F8;
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

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidProjectileTargetXSpanLoc1DB4 {
    pub(crate) left_px: i16,
    pub(crate) right_px: i16,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidProjectileTargetRectLoc1DBE {
    pub(crate) left_px: i16,
    pub(crate) right_px: i16,
    pub(crate) top_px: i16,
    pub(crate) bottom_px: i16,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KidProjectileCollisionObjectLoc1DCC {
    pub(crate) deleted: bool,
    pub(crate) mirrored: bool,
    pub(crate) collision_index_word: u16,
    pub(crate) x_anchor_px: i16,
    pub(crate) y_anchor_px: i16,
    pub(crate) collision_sizes: Vec<CollisionSizeEntrySub21F8>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KidProjectileCollisionCursorLoc1DCC {
    pub(crate) object_index: usize,
    pub(crate) collision_size_index: usize,
    pub(crate) mirrored: bool,
    pub(crate) x_anchor_px: i16,
    pub(crate) y_anchor_px: i16,
    pub(crate) collision_sizes: Vec<CollisionSizeEntrySub21F8>,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn kid_projectile_target_x_span_loc_1DB4(
    x_pos: i16,
    x_direction: u8,
    x_leading_offset: i16,
    x_span: i16,
) -> KidProjectileTargetXSpanLoc1DB4 {
    // ASM sub_1D84 (kid.asm:3560-3567): when `x_direction(a0)` is clear the
    // pre-`loc_1DB4` path does `d0 = x_pos + leading_offset`, copies that to
    // `d1`, then adds the span so the target runs `[d0, d0 + span]`.
    //
    // ASM loc_1DB4 (kid.asm:3568-3572): when `x_direction(a0)` is non-zero the
    // routine mirrors that X interval by negating the leading offset, adding it
    // to `x_pos`, copying the result to `d1`, then subtracting the span so the
    // target runs `[d1 - span, d1]`.
    if x_direction == 0 {
        let left_px = x_pos.wrapping_add(x_leading_offset);
        KidProjectileTargetXSpanLoc1DB4 {
            left_px,
            right_px: left_px.wrapping_add(x_span),
        }
    } else {
        let right_px = x_pos.wrapping_sub(x_leading_offset);
        KidProjectileTargetXSpanLoc1DB4 {
            left_px: right_px.wrapping_sub(x_span),
            right_px,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn kid_projectile_target_rect_loc_1DBE(
    x_pos: i16,
    y_pos: i16,
    x_direction: u8,
    x_leading_offset: i16,
    x_span: i16,
    y_offset: i16,
    y_span: i16,
) -> KidProjectileTargetRectLoc1DBE {
    let x_span =
        kid_projectile_target_x_span_loc_1DB4(x_pos, x_direction, x_leading_offset, x_span);
    // ASM loc_1DBE (kid.asm:3574-3578): after either X-span path, load the
    // leading Y offset into `d2`, add `y_pos(a0)`, copy that to `d3`, then add
    // the final span so the shared target rectangle runs `[d2, d2 + span]`.
    let top_px = y_pos.wrapping_add(y_offset);
    KidProjectileTargetRectLoc1DBE {
        left_px: x_span.left_px,
        right_px: x_span.right_px,
        top_px,
        bottom_px: top_px.wrapping_add(y_span),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn first_live_kid_projectile_collision_cursor_loc_1DCC(
    objects: &[KidProjectileCollisionObjectLoc1DCC],
) -> Option<KidProjectileCollisionCursorLoc1DCC> {
    for (object_index, object) in objects.iter().enumerate() {
        // ASM loc_1DCC (kid.asm:3581-3589): follow the gfx-object list, skip
        // deleted entries via `tst.b $3D(a2)`, then load `$22(a2)`, halve it,
        // index `CollisionSize_Index`, and back the cursor up by one 8-byte
        // slot so `loc_1DF2`/`loc_1E1E` can start with `addq.w #8,a3`.
        if object.deleted {
            continue;
        }

        return Some(KidProjectileCollisionCursorLoc1DCC {
            object_index,
            collision_size_index: usize::from(object.collision_index_word >> 1),
            mirrored: object.mirrored,
            x_anchor_px: object.x_anchor_px,
            y_anchor_px: object.y_anchor_px,
            collision_sizes: object.collision_sizes.clone(),
        });
    }

    None
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn first_non_mirrored_kid_projectile_overlap_loc_1DF2(
    target: KidProjectileTargetRectLoc1DBE,
    cursor: &KidProjectileCollisionCursorLoc1DCC,
) -> Option<usize> {
    // ASM loc_1DF2 (kid.asm:3595-3608): after `loc_1DCC` has selected a live
    // object with clear `x_direction(a2)`, advance one 8-byte CollisionSize
    // entry at a time. A zero first word aborts this object, otherwise the
    // scan uses inclusive overlap checks and returns on the first hit.
    if cursor.mirrored {
        return None;
    }

    for (collision_size_index, collision_size) in cursor.collision_sizes.iter().enumerate() {
        if collision_size.left_offset_px == 0 {
            return None;
        }

        let left_px = cursor
            .x_anchor_px
            .saturating_add(collision_size.left_offset_px);
        if left_px > target.right_px {
            continue;
        }

        let right_px = left_px.saturating_add(collision_size.width_px);
        if right_px < target.left_px {
            continue;
        }

        let top_px = cursor
            .y_anchor_px
            .saturating_add(collision_size.top_offset_px);
        if top_px > target.bottom_px {
            continue;
        }

        let bottom_px = top_px.saturating_add(collision_size.height_px);
        if bottom_px < target.top_px {
            continue;
        }

        return Some(collision_size_index);
    }

    None
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn first_mirrored_kid_projectile_overlap_loc_1E1E(
    target: KidProjectileTargetRectLoc1DBE,
    cursor: &KidProjectileCollisionCursorLoc1DCC,
) -> Option<usize> {
    // ASM loc_1E1E (kid.asm:3615-3629): for set `x_direction(a2)`, advance one
    // 8-byte CollisionSize entry at a time, mirror the horizontal interval by
    // negating the stored left offset, and subtract the width back from that
    // anchor before applying the same inclusive overlap tests.
    if !cursor.mirrored {
        return None;
    }

    for (collision_size_index, collision_size) in cursor.collision_sizes.iter().enumerate() {
        if collision_size.left_offset_px == 0 {
            return None;
        }

        let right_px = cursor
            .x_anchor_px
            .saturating_sub(collision_size.left_offset_px);
        if right_px < target.left_px {
            continue;
        }

        let left_px = right_px.saturating_sub(collision_size.width_px);
        if left_px > target.right_px {
            continue;
        }

        let top_px = cursor
            .y_anchor_px
            .saturating_add(collision_size.top_offset_px);
        if top_px > target.bottom_px {
            continue;
        }

        let bottom_px = top_px.saturating_add(collision_size.height_px);
        if bottom_px < target.top_px {
            continue;
        }

        return Some(collision_size_index);
    }

    None
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
