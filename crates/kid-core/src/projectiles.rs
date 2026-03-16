use super::*;

impl NativeRuntime {
    /// Advance all active projectiles: move by velocity, decrement lifetime, remove expired.
    /// Homing projectiles accelerate toward `player_pos` each frame.
    pub(crate) fn step_projectiles(&mut self) {
        let player_x = self.opening_slice.as_ref().map(|s| s.player.x_pos);
        let player_y = self.opening_slice.as_ref().map(|s| s.player.y_pos);
        self.projectiles.retain_mut(|p| {
            if p.lifetime == 0 {
                return false;
            }
            p.lifetime -= 1;
            p.y_velocity += p.gravity;
            // Homing: accelerate toward player ($7D0 per axis per frame from ASM)
            if p.homing {
                if let (Some(px), Some(py)) = (player_x, player_y) {
                    let dx = px as i64 - p.x_pos as i64;
                    let dy = py as i64 - p.y_pos as i64;
                    p.x_velocity += if dx > 0 { HOMING_ACCEL } else { -HOMING_ACCEL };
                    p.y_velocity += if dy > 0 { HOMING_ACCEL } else { -HOMING_ACCEL };
                }
            }
            p.x_pos = (p.x_pos as i64 + i64::from(p.x_velocity)) as u32;
            p.y_pos = (p.y_pos as i64 + i64::from(p.y_velocity)) as u32;
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
