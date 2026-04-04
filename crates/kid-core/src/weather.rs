use super::*;

#[derive(Clone, Copy)]
struct LavaFireballSegment {
    frames: u8,
    delta_y: i16,
    sprite_kind: LavaFireballSpriteKind,
}

#[derive(Clone, Copy)]
enum LavaFireballSpriteKind {
    TallRise,
    MidRise,
    MidBody,
    MidFall,
    TallFall,
}

const LAVA_FIREBALL_SCRIPT: [LavaFireballSegment; 11] = [
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -4,
        sprite_kind: LavaFireballSpriteKind::TallRise,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -3,
        sprite_kind: LavaFireballSpriteKind::TallRise,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -3,
        sprite_kind: LavaFireballSpriteKind::MidRise,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: -2,
        sprite_kind: LavaFireballSpriteKind::MidRise,
    },
    LavaFireballSegment {
        frames: 0x09,
        delta_y: -1,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x05,
        delta_y: 0,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x05,
        delta_y: 0,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x09,
        delta_y: 1,
        sprite_kind: LavaFireballSpriteKind::MidBody,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: 2,
        sprite_kind: LavaFireballSpriteKind::MidFall,
    },
    LavaFireballSegment {
        frames: 0x0B,
        delta_y: 2,
        sprite_kind: LavaFireballSpriteKind::TallFall,
    },
    LavaFireballSegment {
        frames: 0x15,
        delta_y: 4,
        sprite_kind: LavaFireballSpriteKind::TallFall,
    },
];

fn kid_hitbox_bounds_px(slice: &OpeningSliceState) -> (i32, i32, i32, i32) {
    let x_px = (slice.player.x_pos >> 16) as i32;
    let foot_y_px = (slice.player.y_pos >> 16) as i32;
    (
        x_px - PLAYER_HALF_WIDTH,
        x_px + PLAYER_HALF_WIDTH,
        foot_y_px - PLAYER_HEIGHT,
        foot_y_px,
    )
}

#[allow(non_snake_case)]
fn lava_geyser_hits_kid_sub_14A58(geyser: &LavaGeyser, slice: &OpeningSliceState) -> bool {
    if geyser.current_height == 0 {
        return false;
    }

    let (kid_left, kid_right, kid_top, kid_bottom) = kid_hitbox_bounds_px(slice);
    let geyser_left = i32::from(geyser.x_px) + 3;
    let geyser_right = geyser_left + 0x1A;
    let geyser_bottom = i32::from(geyser.y_px);
    // ASM sub_14A58 compares Kid against `$54(a5)+$10`, where `$54(a5)` tracks
    // the current top sprite row built by the geyser worker.
    let geyser_top = i32::from(geyser.y_px.saturating_sub(geyser.current_height)) + 0x10;

    kid_right >= geyser_left
        && kid_left <= geyser_right
        && kid_top <= geyser_bottom
        && kid_bottom >= geyser_top
}

#[allow(non_snake_case)]
fn lava_fireball_hits_kid_sub_14AB0(
    fireball: &LavaFireball,
    slice: &OpeningSliceState,
    sprite_kind: LavaFireballSpriteKind,
) -> bool {
    let (kid_left, kid_right, kid_top, kid_bottom) = kid_hitbox_bounds_px(slice);
    let fireball_left = i32::from(fireball.x_px) + 1;
    let fireball_right = fireball_left + 6;
    let baseline_y = i32::from(fireball.current_y_px);
    // ASM sub_14AB0 uses d1 as the segment baseline and `$54(a5)` as the top
    // sprite row written by sub_14BD8. The table-specific adjustments below
    // mirror the five sprite-layout buckets (`unk_14664` .. `unk_14696`).
    let (fireball_top, fireball_bottom) = match sprite_kind {
        LavaFireballSpriteKind::TallRise => (baseline_y - 23, baseline_y - 8),
        LavaFireballSpriteKind::MidRise => (baseline_y - 18, baseline_y - 1),
        LavaFireballSpriteKind::MidBody => (baseline_y - 7, baseline_y - 1),
        LavaFireballSpriteKind::MidFall => (baseline_y - 12, baseline_y - 1),
        LavaFireballSpriteKind::TallFall => (baseline_y - 23, baseline_y + 6),
    };

    kid_right >= fireball_left
        && kid_left <= fireball_right
        && kid_top <= fireball_bottom
        && kid_bottom >= fireball_top
}

fn advance_lava_fireball(fireball: &mut LavaFireball) {
    if fireball.cooldown_timer > 0 {
        fireball.cooldown_timer -= 1;
        if fireball.cooldown_timer == 0 {
            fireball.segment_index = 0;
            fireball.segment_frame = 0;
            fireball.current_y_px = fireball.base_y_px;
        }
        return;
    }

    let segment = LAVA_FIREBALL_SCRIPT[usize::from(fireball.segment_index)];
    fireball.current_y_px = fireball.current_y_px.saturating_add_signed(segment.delta_y);
    fireball.segment_frame = fireball.segment_frame.saturating_add(1);

    if fireball.segment_frame >= segment.frames {
        fireball.segment_frame = 0;
        if usize::from(fireball.segment_index) + 1 == LAVA_FIREBALL_SCRIPT.len() {
            fireball.segment_index = 0;
            fireball.cooldown_timer = 0x3C;
            fireball.current_y_px = fireball.base_y_px;
        } else {
            fireball.segment_index += 1;
        }
    }
}

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
        if self.weather.weather_type != 3 {
            return;
        }
        // Level timer counts down from 180s. 35s elapsed = 145s remaining.
        if self.level_timer.seconds > 145 {
            return;
        }

        // Toggle hail active/inactive every 600 frames ($258)
        if self.weather.hail_period_timer > 0 {
            self.weather.hail_period_timer -= 1;
        } else {
            self.weather.hail_active = !self.weather.hail_active;
            self.weather.hail_period_timer = 600;
        }

        if !self.weather.hail_active {
            return;
        }

        // Spawn hail every 30 frames during active period
        if self.weather.hail_spawn_timer > 0 {
            self.weather.hail_spawn_timer -= 1;
            return;
        }
        self.weather.hail_spawn_timer = 30;

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
            ..Default::default()
        });
    }

    pub(crate) fn init_weather(&mut self, level_id: LevelId) {
        let header = self.level_header(level_id);
        self.weather.weather_type = header.map(|h| h.weather_flags).unwrap_or(0);
        self.weather.hail_active = false;
        self.weather.hail_spawn_timer = 0;
        self.weather.hail_period_timer = 0;
        self.weather.lava_geysers.clear();
        self.weather.lava_fireballs.clear();
        if self.weather.weather_type == 1 {
            self.init_lava_geysers(level_id);
        }
    }

    /// Initialise lava geyser positions for lava weather levels.
    /// Uses hardcoded per-level position tables matching the original ASM data.
    pub(crate) fn init_lava_geysers(&mut self, level_id: LevelId) {
        // Per-level geyser positions: (x_px, y_px, height_px)
        let positions: &[(u16, u16, u16)] = match level_id.0 {
            // Under Skull Mountain 2 (USM2)
            0x06 => &[(0x180, 0x312, 0x60), (0x240, 0x312, 0x63)],
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
            let initial_delay = (h & 0xF) * 10;
            self.weather.lava_geysers.push(LavaGeyser {
                x_px: x,
                y_px: y,
                height_px: h >> 4,
                phase: 0,
                timer: initial_delay,
                current_height: 0,
            });
        }

        if level_id.0 == 0x06 {
            self.weather.lava_fireballs.push(LavaFireball {
                x_px: 0x346,
                base_y_px: 0x312,
                current_y_px: 0x312,
                segment_index: 0,
                segment_frame: 0,
                cooldown_timer: 0,
            });
        }
    }

    /// Step all active lava geysers through their eruption cycle.
    pub(crate) fn step_lava_geysers(&mut self, slice: &OpeningSliceState) {
        if self.weather.weather_type != 1
            || (self.weather.lava_geysers.is_empty() && self.weather.lava_fireballs.is_empty())
        {
            return;
        }

        let mut should_damage = false;

        for geyser in &mut self.weather.lava_geysers {
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

            if lava_geyser_hits_kid_sub_14A58(geyser, slice) {
                should_damage = true;
            }
        }

        for fireball in &mut self.weather.lava_fireballs {
            let segment = LAVA_FIREBALL_SCRIPT[usize::from(fireball.segment_index)];
            if fireball.cooldown_timer == 0
                && lava_fireball_hits_kid_sub_14AB0(fireball, slice, segment.sprite_kind)
            {
                should_damage = true;
            }
            advance_lava_fireball(fireball);
        }

        if should_damage && self.progression.invulnerable_frames == 0 {
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
