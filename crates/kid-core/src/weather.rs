use super::*;

impl NativeRuntime {
    pub(crate) fn step_murder_wall(&mut self) {
        let Some(wall) = self.murder_wall.as_mut() else {
            return;
        };
        wall.speed = (wall.speed + MURDER_WALL_ACCELERATION).min(wall.max_speed);
        if wall.reversed {
            wall.x_pos -= i64::from(wall.speed);
        } else {
            wall.x_pos += i64::from(wall.speed);
        }
    }

    /// Check if the murder wall overlaps the player, triggering instant death.
    pub(crate) fn murder_wall_kills_player(&self, slice: &OpeningSliceState) -> bool {
        let Some(wall) = &self.murder_wall else {
            return false;
        };
        let player_x = slice.player.x_pos as i64;
        let wall_x = wall.x_pos;
        if wall.reversed {
            // Wall moves right-to-left: kill if player's right side touches wall
            let wall_left = wall_x;
            player_x + (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16) >= wall_left
        } else {
            // Wall moves left-to-right: kill if wall passes player's left side
            let wall_right = wall_x + (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16);
            wall_right >= player_x
        }
    }

    pub(crate) fn step_weather(&mut self, slice: &OpeningSliceState) {
        // Hail only for Storm+Hail (weather 3), active after 35s of level time
        if self.weather_type != 3 {
            return;
        }
        // Level timer counts down from 180s. 35s elapsed = 145s remaining.
        if self.level_timer.seconds > 145 {
            return;
        }

        // Toggle hail active/inactive every 600 frames ($258)
        if self.hail_period_timer > 0 {
            self.hail_period_timer -= 1;
        } else {
            self.hail_active = !self.hail_active;
            self.hail_period_timer = 600;
        }

        if !self.hail_active {
            return;
        }

        // Spawn hail every 30 frames during active period
        if self.hail_spawn_timer > 0 {
            self.hail_spawn_timer -= 1;
            return;
        }
        self.hail_spawn_timer = 30;

        // Spawn a hail projectile above the camera, at a random-ish X
        let camera_x = slice.camera.x_pos >> 16;
        // Simple pseudo-random X offset based on frame index
        let x_offset = ((self.frame_index * 7 + 13) % 320) as u32;
        let spawn_x = camera_x.wrapping_add(x_offset);
        let spawn_y = (slice.camera.y_pos >> 16).saturating_sub(16);

        self.projectiles.push(EnemyProjectile {
            x_pos: spawn_x << 16,
            y_pos: spawn_y << 16,
            x_velocity: 0,
            y_velocity: 0x4_0000, // 4 px/frame downward
            lifetime: 120,
            friendly: false,
            gravity: 0,
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
        });
    }

    pub(crate) fn init_weather(&mut self, level_id: LevelId) {
        let header = self.level_header(level_id);
        self.weather_type = header.map(|h| h.weather_flags).unwrap_or(0);
        self.hail_active = false;
        self.hail_spawn_timer = 0;
        self.hail_period_timer = 0;
        self.lava_geysers.clear();
        if self.weather_type == 1 {
            self.init_lava_geysers(level_id);
        }
    }

    /// Initialise lava geyser positions for lava weather levels.
    /// Uses hardcoded per-level position tables matching the original ASM data.
    pub(crate) fn init_lava_geysers(&mut self, level_id: LevelId) {
        // Per-level geyser positions: (x_px, y_px, height_px)
        let positions: &[(u16, u16, u16)] = match level_id.0 {
            // Under Skull Mountain 2 (USM2)
            0x06 => &[
                (0x180, 0x312, 0x60),
                (0x240, 0x312, 0x63),
                (0x346, 0x312, 0x40),
            ],
            // Black Pit
            0x45 => &[
                (0x290, 0x170, 0x20),
                (0x3D0, 0x170, 0x20),
                (0x510, 0x170, 0x40),
                (0x650, 0x170, 0x60),
                (0x790, 0x170, 0x80),
                (0x8D0, 0x170, 0xA0),
                (0xA10, 0x170, 0xC0),
                (0xB50, 0x170, 0xE0),
                (0xC90, 0x170, 0x100),
                (0xDD0, 0x170, 0x100),
                (0xF10, 0x170, 0x100),
                (0x1050, 0x170, 0x100),
                (0x1190, 0x170, 0x100),
            ],
            // Elsewhere 29
            0x3E => &[
                (0x090, 0x330, 0x260),
                (0x010, 0x1D0, 0x090),
                (0x110, 0x1D0, 0x090),
            ],
            _ => &[],
        };

        for &(x, y, h) in positions {
            let initial_delay = u16::from(h & 0xF) * 10;
            self.lava_geysers.push(LavaGeyser {
                x_px: x,
                y_px: y,
                height_px: h >> 4,
                phase: 0,
                timer: initial_delay,
                current_height: 0,
            });
        }
    }

    /// Step all active lava geysers through their eruption cycle.
    pub(crate) fn step_lava_geysers(&mut self, slice: &OpeningSliceState) {
        if self.weather_type != 1 || self.lava_geysers.is_empty() {
            return;
        }

        let player_x = (slice.player.x_pos >> 16) as i32;
        let player_y = (slice.player.y_pos >> 16) as i32;
        let mut should_damage = false;

        for geyser in &mut self.lava_geysers {
            match geyser.phase {
                0 => {
                    // Waiting/dormant
                    if geyser.timer > 0 {
                        geyser.timer -= 1;
                    } else {
                        geyser.phase = 1;
                        geyser.current_height = 0;
                    }
                }
                1 => {
                    // Erupting upward
                    geyser.current_height += LAVA_GEYSER_ERUPT_SPEED;
                    if geyser.current_height >= geyser.height_px {
                        geyser.current_height = geyser.height_px;
                        geyser.phase = 2;
                        geyser.timer = LAVA_GEYSER_HOLD_FRAMES;
                    }
                }
                2 => {
                    // Holding at full height
                    if geyser.timer > 0 {
                        geyser.timer -= 1;
                    } else {
                        geyser.phase = 3;
                    }
                }
                3 => {
                    // Collapsing
                    if geyser.current_height > LAVA_GEYSER_ERUPT_SPEED {
                        geyser.current_height -= LAVA_GEYSER_ERUPT_SPEED;
                    } else {
                        geyser.current_height = 0;
                        geyser.phase = 0;
                        geyser.timer = LAVA_GEYSER_DORMANT_FRAMES;
                    }
                }
                _ => {}
            }

            // Collision: player overlaps eruption column (16px wide, current_height tall)
            if geyser.current_height > 0 {
                let gx = i32::from(geyser.x_px);
                let gy_top = i32::from(geyser.y_px) - i32::from(geyser.current_height);
                let gy_bottom = i32::from(geyser.y_px);
                if player_x >= gx - 8
                    && player_x <= gx + 8
                    && player_y >= gy_top
                    && player_y <= gy_bottom
                {
                    should_damage = true;
                }
            }
        }

        if should_damage && self.invulnerable_frames == 0 {
            self.apply_damage();
        }
    }

    pub(crate) fn init_murder_wall(&mut self, level_id: LevelId, camera_x: u32) {
        let header = self.level_header(level_id);
        let flags = header.map(|h| h.murder_wall_flags).unwrap_or(0);
        if flags & 0b10 != 0 {
            let reversed = flags & 0b01 != 0;
            let max_speed = if level_id.0 == 0x08 {
                MURDER_WALL_SLOW_MAX_SPEED
            } else {
                MURDER_WALL_DEFAULT_MAX_SPEED
            };
            let initial_x = if camera_x == 0 {
                -(i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16)
            } else {
                (i64::from(camera_x) << 16) + (i64::from(MURDER_WALL_CAMERA_OFFSET_PX) << 16)
            };
            self.murder_wall = Some(MurderWallState {
                x_pos: initial_x,
                speed: 0,
                max_speed,
                reversed,
            });
        } else {
            self.murder_wall = None;
        }
    }
}
