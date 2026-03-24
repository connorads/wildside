use super::*;

impl PlatformLoaderState {
    pub(crate) fn advance_to_window(&mut self, layout: &PlatformLayout, window: ActivationWindow) {
        while self.next_index < layout.placements.len() {
            let placement = &layout.placements[self.next_index];
            let preload_limit = window
                .x_max
                .saturating_add(PLATFORM_PRELOAD_X_PADDING)
                .saturating_add(u32::from(placement.buffer_right));
            if u32::from(placement.x) > preload_limit {
                break;
            }
            self.next_index += 1;
        }
    }

    pub(crate) fn active_platforms(
        &self,
        layout: &PlatformLayout,
        window: ActivationWindow,
    ) -> Vec<NativeObjectSnapshot> {
        layout
            .placements
            .iter()
            .take(self.next_index)
            .filter(|placement| platform_is_active(placement, window))
            .map(|placement| NativeObjectSnapshot {
                kind: NativeObjectKind::Platform {
                    kind: placement.kind,
                    subtype: placement.subtype,
                },
                x_pos: u32::from(placement.x) << 16,
                y_pos: u32::from(placement.y) << 16,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            })
            .collect()
    }
}

/// Check if a platform is within the activation window.
pub(crate) fn platform_is_active(platform: &PlatformPlacement, window: ActivationWindow) -> bool {
    let object_x = u32::from(platform.x);
    let object_y = u32::from(platform.y);
    let x_min = window.x_min.saturating_sub(u32::from(platform.buffer_left));
    let x_max = window
        .x_max
        .saturating_add(u32::from(platform.buffer_right));
    let y_min = window.y_min.saturating_sub(u32::from(platform.buffer_top));
    let y_max = window
        .y_max
        .saturating_add(u32::from(platform.buffer_bottom));
    (x_min..=x_max).contains(&object_x) && (y_min..=y_max).contains(&object_y)
}

/// Check if the player should land on top of any active platform.
///
/// Runs after terrain collision so that platforms override terrain when the
/// player is above a platform surface. Only applies when the player is not
/// already grounded by terrain (falling or airborne).
pub(crate) fn apply_platform_collision(
    slice: &mut OpeningSliceState,
    platforms: &[ActivePlatform],
) {
    let player_x_px = (slice.player.x_pos >> 16) as i32;
    let player_foot_y_px = (slice.player.y_pos >> 16) as i32;
    let player_left = player_x_px - PLAYER_HALF_WIDTH;
    let player_right = player_x_px + PLAYER_HALF_WIDTH;

    // If player is already on ground, check if they're riding a platform and
    // carry them along with the platform's velocity.
    if slice.player.on_ground {
        for platform in platforms {
            if platform.x_velocity == 0 && platform.y_velocity == 0 {
                continue;
            }
            let plat_x_px = (platform.x_pos >> 16) as i32;
            let plat_y_px = (platform.y_pos >> 16) as i32;
            let plat_right = plat_x_px + i32::from(platform.width_px);

            if player_right < plat_x_px || player_left > plat_right {
                continue;
            }
            // Player feet within snap distance of platform top → riding it
            let within_snap =
                slice.player.y_pos.abs_diff(platform.y_pos) <= GROUND_SNAP_TOLERANCE_FP;
            if !within_snap && player_foot_y_px.abs_diff(plat_y_px) > 2 {
                continue;
            }

            // Carry player with platform velocity
            slice.player.x_pos = (i64::from(slice.player.x_pos) + i64::from(platform.x_velocity))
                .clamp(0, i64::from(u32::MAX)) as u32;
            slice.player.y_pos = (i64::from(slice.player.y_pos) + i64::from(platform.y_velocity))
                .clamp(0, i64::from(u32::MAX)) as u32;
            let floor_y_px = (slice.player.y_pos >> 16) as i32;
            slice.terrain.floor_y_pos = (floor_y_px.max(0) as u32) << 16;
            return;
        }
        return;
    }

    if slice.player.y_velocity < 0 {
        // Moving upward — don't land on platforms from below
        return;
    }

    for platform in platforms {
        let plat_x_px = (platform.x_pos >> 16) as i32;
        let plat_y_px = (platform.y_pos >> 16) as i32;
        let plat_right = plat_x_px + i32::from(platform.width_px);

        // Horizontal overlap check
        if player_right < plat_x_px || player_left > plat_right {
            continue;
        }

        // Vertical check: player feet should be at or just below the platform top.
        // Use the same snap tolerance as terrain floor collision.
        let plat_top_fp = platform.y_pos;
        let within_snap = slice.player.y_pos.abs_diff(plat_top_fp) <= GROUND_SNAP_TOLERANCE_FP;
        let feet_at_or_below = player_foot_y_px >= plat_y_px;

        if feet_at_or_below || within_snap {
            // Don't land if player feet are well past the platform (inside it)
            let plat_bottom = plat_y_px + i32::from(platform.height_px);
            if player_foot_y_px > plat_bottom {
                continue;
            }

            // Land on this platform
            let floor_y_px = plat_y_px.saturating_sub(1);
            let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
            // ASM floor landing: loc_AAF8 (y_vel >= 0) preserves sub-pixel,
            // loc_AB22 (y_vel < 0) zeroes it. Use pre_gravity_y_vel since
            // native applies gravity before collisions (GravityFirst).
            if slice.pre_gravity_y_vel >= 0 {
                slice.player.y_pos =
                    (floor_y_fp & 0xFFFF_0000) | (slice.player.y_pos & 0x0000_FFFF);
            } else {
                slice.player.y_pos = floor_y_fp;
            }
            slice.player.on_ground = true;
            slice.player.y_velocity = 0;
            slice.terrain.floor_y_pos = floor_y_fp;
            return;
        }
    }
}
