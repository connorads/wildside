use super::*;

impl NativeRuntime {
    pub(crate) fn step_skycutter_flipboard(
        &mut self,
        slice: &mut OpeningSliceState,
        input: FrameInput,
    ) {
        if self.current_form != PlayerForm::Skycutter {
            self.skycutter_flipboard = 0;
            self.skycutter_on_board = false;
            return;
        }

        // C button rising edge → deploy/retract flipboard
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && self.skycutter_flipboard == 0 {
            self.skycutter_flipboard = 1;
        }

        // Step flipboard counter
        if self.skycutter_flipboard > 0 {
            let next = self
                .skycutter_flipboard
                .saturating_add(SKYCUTTER_COUNTER_STEP);
            if next >= SKYCUTTER_FLIPBOARD_TIMEOUT {
                // Animation complete: retract
                self.skycutter_flipboard = 0;
            } else {
                // Mount toggle at midpoint
                if self.skycutter_flipboard < SKYCUTTER_MOUNT_FRAME && next >= SKYCUTTER_MOUNT_FRAME
                {
                    self.skycutter_on_board = !self.skycutter_on_board;
                }
                self.skycutter_flipboard = next;
            }
        }

        // While on board: override gravity with Skycutter flight physics
        if self.skycutter_on_board && !slice.player.on_ground {
            let gravity = if input.b {
                SKYCUTTER_GRAVITY_B_HELD
            } else {
                SKYCUTTER_GRAVITY_NO_B
            };
            // In the original, the gravity is subtracted (y_vel is positive=down,
            // and the board makes you fall more slowly). But looking at the ASM:
            // sub.l d6,d7 where d7=y_vel and d6=gravity. Since positive y_vel=down
            // in the original game and we use negative y_vel=up, this adds gravity
            // (descent) rather than subtracting.
            slice.player.y_velocity += gravity;
            // Clamp max upward speed
            if slice.player.y_velocity < SKYCUTTER_MAX_UPWARD_VEL {
                slice.player.y_velocity = SKYCUTTER_MAX_UPWARD_VEL;
            }

            // B button press while on board: upward kick
            let b_pressed = input.b && !self.prior_input.b;
            if b_pressed {
                let kick = -(slice.player.x_velocity.unsigned_abs() as i32 / 2 + 0x3_0000);
                slice.player.y_velocity = kick;
            }
        }

        // Landing while on board: bounce upward
        if self.skycutter_on_board && slice.player.on_ground {
            slice.player.y_velocity = SKYCUTTER_LANDING_BOUNCE;
            slice.player.on_ground = false;
            self.skycutter_flipboard = 1; // restart deploy animation
        }
    }

    /// Step Cyclone vertical flight: acceleration-based with C-button thrust.
    ///
    /// When flying, an acceleration accumulator drifts downward by $400/frame
    /// (gravity). Holding C subtracts $3000 (thrust). The accumulator is added
    /// to y_velocity each frame, with velocity clamped to [-$30000, $28000].
    /// Landing clears the flying flag.
    pub(crate) fn step_cyclone_flight(&mut self, slice: &mut OpeningSliceState, input: FrameInput) {
        if self.current_form != PlayerForm::Cyclone {
            self.cyclone_flying = false;
            self.cyclone_y_accel = 0;
            return;
        }

        // C rising edge activates flight when airborne
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && !slice.player.on_ground {
            self.cyclone_flying = true;
            self.cyclone_y_accel = CYCLONE_ACCEL_INITIAL;
        }

        // Landing clears flight
        if self.cyclone_flying && slice.player.on_ground {
            self.cyclone_flying = false;
            return;
        }

        if !self.cyclone_flying {
            return;
        }

        // Accumulate gravity
        self.cyclone_y_accel = self.cyclone_y_accel.saturating_add(CYCLONE_ACCEL_GRAVITY);

        // C held: apply upward thrust
        if input.c {
            self.cyclone_y_accel = self.cyclone_y_accel.saturating_sub(CYCLONE_ACCEL_THRUST);
            if self.cyclone_y_accel < CYCLONE_ACCEL_MIN {
                self.cyclone_y_accel = CYCLONE_ACCEL_MIN;
            }
        } else {
            // Not thrusting: cap descent acceleration
            if self.cyclone_y_accel > CYCLONE_ACCEL_MAX_DESCENT {
                self.cyclone_y_accel = CYCLONE_ACCEL_MAX_DESCENT;
            }
        }

        // Apply acceleration to velocity
        slice.player.y_velocity += i32::from(self.cyclone_y_accel);

        // Clamp velocity
        if slice.player.y_velocity > CYCLONE_MAX_FALL_VEL {
            slice.player.y_velocity = CYCLONE_MAX_FALL_VEL;
        }
        if slice.player.y_velocity < CYCLONE_MAX_RISE_VEL {
            slice.player.y_velocity = CYCLONE_MAX_RISE_VEL;
        }
    }

    /// IronKnight wall climb: C press while airborne attaches to adjacent wall,
    /// C presses while climbing boost upward, natural drift slides down.
    pub(crate) fn step_iron_knight_climb(
        &mut self,
        slice: &mut OpeningSliceState,
        input: FrameInput,
    ) {
        if self.current_form != PlayerForm::IronKnight {
            self.iron_knight_climbing = false;
            self.iron_knight_climb_accel = 0;
            return;
        }

        let c_pressed = input.c && !self.prior_input.c;

        // Try to attach to wall on C press while airborne and not already climbing
        if c_pressed && !slice.player.on_ground && !self.iron_knight_climbing {
            if let Some(ref grid) = self.block_grid {
                // Probe wall in player's facing direction
                let facing_right = slice.player.facing == Direction::Right;
                let px = (slice.player.x_pos >> 16) as i32;
                let py = (slice.player.y_pos >> 16) as i32;
                let probe = terrain::probe_wall(grid, px, py, 5, 31, facing_right);
                if probe.hit {
                    self.iron_knight_climbing = true;
                    self.iron_knight_climb_accel = 0;
                    slice.player.y_velocity = IRON_KNIGHT_ATTACH_Y_VEL;
                    slice.player.x_velocity = 0;
                    return;
                }
            }
        }

        if !self.iron_knight_climbing {
            return;
        }

        // Landing stops climb
        if slice.player.on_ground {
            self.iron_knight_climbing = false;
            self.iron_knight_climb_accel = 0;
            return;
        }

        // B press = jump off wall
        let b_pressed = input.b && !self.prior_input.b;
        if b_pressed {
            self.iron_knight_climbing = false;
            self.iron_knight_climb_accel = 0;
            // Jump away from wall
            let jump_height =
                physics::JUMP_HEIGHTS[usize::from(PlayerForm::IronKnight.helmet_id())];
            let (_, y_vel) = physics::initiate_jump(0, jump_height);
            slice.player.y_velocity = y_vel;
            return;
        }

        // Check wall still exists
        if let Some(ref grid) = self.block_grid {
            let facing_right = slice.player.facing == Direction::Right;
            let px = (slice.player.x_pos >> 16) as i32;
            let py = (slice.player.y_pos >> 16) as i32;
            let probe = terrain::probe_wall(grid, px, py, 5, 31, facing_right);
            if !probe.hit {
                // Wall gone, fall off
                self.iron_knight_climbing = false;
                self.iron_knight_climb_accel = 0;
                return;
            }
        }

        // Accumulate drift (sliding down)
        self.iron_knight_climb_accel = self
            .iron_knight_climb_accel
            .saturating_add(IRON_KNIGHT_SLIDE_DRIFT);

        // C press: boost upward
        if c_pressed {
            self.iron_knight_climb_accel = self
                .iron_knight_climb_accel
                .saturating_sub(IRON_KNIGHT_CLIMB_BOOST);
            if self.iron_knight_climb_accel < IRON_KNIGHT_MIN_ACCEL {
                self.iron_knight_climb_accel = IRON_KNIGHT_MIN_ACCEL;
            }
        } else {
            // Cap slide-down acceleration
            if self.iron_knight_climb_accel > IRON_KNIGHT_MAX_SLIDE_ACCEL {
                self.iron_knight_climb_accel = IRON_KNIGHT_MAX_SLIDE_ACCEL;
            }
        }

        // Apply acceleration to velocity
        slice.player.y_velocity += i32::from(self.iron_knight_climb_accel);

        // Clamp velocity
        if slice.player.y_velocity < IRON_KNIGHT_MAX_CLIMB_VEL {
            slice.player.y_velocity = IRON_KNIGHT_MAX_CLIMB_VEL;
        }
        if slice.player.y_velocity > IRON_KNIGHT_MAX_FALL_VEL {
            slice.player.y_velocity = IRON_KNIGHT_MAX_FALL_VEL;
        }

        // Zero horizontal velocity while climbing
        slice.player.x_velocity = 0;
    }

    /// Micromax wall/ceiling stick: auto-attach on contact while airborne,
    /// reduced gravity while stuck, B press detaches with kick velocity.
    pub(crate) fn step_micromax_stick(&mut self, slice: &mut OpeningSliceState, input: FrameInput) {
        // Only active for Micromax form
        if self.current_form != PlayerForm::Micromax {
            self.micromax_stuck = false;
            return;
        }

        // Already stuck: handle slide and detach
        if self.micromax_stuck {
            let b_pressed = input.b && !self.prior_input.b;
            if b_pressed {
                // Detach: kick upward and away
                self.micromax_stuck = false;
                slice.player.y_velocity = MICROMAX_DETACH_Y_VEL;
                // Reverse facing direction
                slice.player.facing = match slice.player.facing {
                    Direction::Right => Direction::Left,
                    Direction::Left => Direction::Right,
                };
                let kick_x = match slice.player.facing {
                    Direction::Right => MICROMAX_DETACH_X_SPEED,
                    Direction::Left => -MICROMAX_DETACH_X_SPEED,
                };
                slice.player.x_velocity = kick_x;
                slice.player.on_ground = false;
                return;
            }

            // While stuck: apply reduced gravity (slowly peels off)
            slice.player.y_velocity += MICROMAX_STUCK_GRAVITY;
            if slice.player.y_velocity > MICROMAX_STUCK_MAX_FALL {
                // Peeled off
                self.micromax_stuck = false;
                return;
            }

            // Zero horizontal velocity while stuck (player can't walk)
            slice.player.x_velocity = 0;

            // Check surface still exists
            if let Some(ref grid) = self.block_grid {
                let px = (slice.player.x_pos >> 16) as i32;
                let py = (slice.player.y_pos >> 16) as i32;
                let ceiling = terrain::probe_ceiling(grid, px, py, MICROMAX_HALF_W, MICROMAX_H);
                let wall_r = terrain::probe_wall(grid, px, py, MICROMAX_HALF_W, MICROMAX_H, true);
                let wall_l = terrain::probe_wall(grid, px, py, MICROMAX_HALF_W, MICROMAX_H, false);
                if !ceiling.hit && !wall_r.hit && !wall_l.hit {
                    self.micromax_stuck = false;
                }
            }
            return;
        }

        // Not stuck: try to attach while airborne
        if slice.player.on_ground {
            return;
        }
        // Only stick when falling or neutral (y_vel >= 0) per ASM
        if slice.player.y_velocity < 0 {
            return;
        }

        if let Some(ref grid) = self.block_grid {
            let px = (slice.player.x_pos >> 16) as i32;
            let py = (slice.player.y_pos >> 16) as i32;

            // Check ceiling contact
            let ceiling = terrain::probe_ceiling(grid, px, py, MICROMAX_HALF_W, MICROMAX_H);
            if ceiling.hit {
                self.micromax_stuck = true;
                slice.player.y_velocity = 0;
                slice.player.x_velocity = 0;
                return;
            }

            // Check wall contact in facing direction
            let facing_right = slice.player.facing == Direction::Right;
            let wall = terrain::probe_wall(grid, px, py, MICROMAX_HALF_W, MICROMAX_H, facing_right);
            if wall.hit {
                self.micromax_stuck = true;
                slice.player.y_velocity = 0;
                slice.player.x_velocity = 0;
            }
        }
    }

    /// Step RedStealth sword swing: C press starts swing, hitbox active after startup.
    pub(crate) fn step_red_stealth_sword(&mut self, input: FrameInput) {
        if self.current_form != PlayerForm::RedStealth {
            self.red_stealth_sword_timer = 0;
            return;
        }

        // C rising edge starts a new swing (only if not already swinging)
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && self.red_stealth_sword_timer == 0 {
            self.red_stealth_sword_timer = RED_STEALTH_STARTUP_FRAMES + RED_STEALTH_ACTIVE_FRAMES;
        }

        // Tick down active timer
        if self.red_stealth_sword_timer > 0 {
            self.red_stealth_sword_timer -= 1;
        }
    }

    /// Returns true if the sword hitbox is active (past startup, within active frames).
    pub(crate) fn sword_is_active(&self) -> bool {
        self.red_stealth_sword_timer > 0
            && self.red_stealth_sword_timer <= RED_STEALTH_ACTIVE_FRAMES
    }

    /// Check sword hitbox against active enemies during active frames.
    /// Damages hit enemies and removes dead ones.
    pub(crate) fn process_sword_hits(&mut self, slice: &OpeningSliceState) {
        if !self.sword_is_active() {
            return;
        }

        let px = (slice.player.x_pos >> 16) as i32;
        let py = (slice.player.y_pos >> 16) as i32;
        let sword_x = if slice.player.facing == Direction::Right {
            px + RED_STEALTH_REACH_X
        } else {
            px - RED_STEALTH_REACH_X
        };
        let sword_y = py + RED_STEALTH_OFFSET_Y;

        let mut kill_indices = Vec::new();
        for (i, enemy) in self.active_enemies.iter_mut().enumerate() {
            if matches!(enemy.family, EnemyFamily::Diamond) {
                continue;
            }
            let ex = (enemy.x_pos >> 16) as i32;
            let ey = (enemy.y_pos >> 16) as i32;
            let dx = (sword_x - ex).abs();
            let dy = (sword_y - ey).abs();
            if dx <= RED_STEALTH_HIT_HALF + ENEMY_CONTACT_HALF_WIDTH as i32
                && dy <= RED_STEALTH_HIT_HALF + ENEMY_CONTACT_HALF_HEIGHT as i32
            {
                enemy.hit_points = enemy.hit_points.saturating_sub(1);
                if enemy.hit_points == 0 {
                    kill_indices.push(i);
                } else {
                    enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                }
            }
        }

        for &idx in kill_indices.iter().rev() {
            let enemy = &self.active_enemies[idx];
            self.enemy_death_effects.push(EnemyDeathEffect {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                remaining: 10,
            });
            self.active_enemies.remove(idx);
            self.score += 1000;
        }
        if !kill_indices.is_empty() {
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
    }

    /// Check friendly projectiles (Eyeclops beam, Maniaxe axe) against enemies.
    pub(crate) fn process_friendly_projectile_hits(&mut self) {
        let mut kill_indices = Vec::new();
        let mut projectile_consumed = Vec::new();

        for (pi, proj) in self.projectiles.iter().enumerate() {
            if !proj.friendly || proj.lifetime == 0 {
                continue;
            }
            let prx = (proj.x_pos >> 16) as i32;
            let pry = (proj.y_pos >> 16) as i32;

            for (ei, enemy) in self.active_enemies.iter_mut().enumerate() {
                if matches!(enemy.family, EnemyFamily::Diamond) {
                    continue;
                }
                let ex = (enemy.x_pos >> 16) as i32;
                let ey = (enemy.y_pos >> 16) as i32;
                let dx = (prx - ex).abs();
                let dy = (pry - ey).abs();
                if dx <= PROJECTILE_HITBOX_HALF + ENEMY_CONTACT_HALF_WIDTH as i32
                    && dy <= PROJECTILE_HITBOX_HALF + ENEMY_CONTACT_HALF_HEIGHT as i32
                {
                    enemy.hit_points = enemy.hit_points.saturating_sub(1);
                    if enemy.hit_points == 0 {
                        kill_indices.push(ei);
                    } else {
                        enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                    }
                    projectile_consumed.push(pi);
                    break; // one enemy per projectile per frame
                }
            }
        }

        // Remove consumed projectiles
        for &pi in projectile_consumed.iter().rev() {
            self.projectiles.remove(pi);
        }

        // Remove dead enemies, spawn death effects, and award points
        for &idx in kill_indices.iter().rev() {
            let enemy = &self.active_enemies[idx];
            self.enemy_death_effects.push(EnemyDeathEffect {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                remaining: 10,
            });
            self.active_enemies.remove(idx);
            self.score += 1000;
        }
        if !kill_indices.is_empty() || !projectile_consumed.is_empty() {
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
    }

    /// When Berzerker is charging, side contact with enemies damages them.
    pub(crate) fn process_berzerker_charge_hits(&mut self, slice: &OpeningSliceState) {
        if !self.berzerker_charging {
            return;
        }

        let px = (slice.player.x_pos >> 16) as i32;
        let py = (slice.player.y_pos >> 16) as i32;

        let mut kill_indices = Vec::new();
        for (i, enemy) in self.active_enemies.iter_mut().enumerate() {
            if matches!(enemy.family, EnemyFamily::Diamond) {
                continue;
            }
            let ex = (enemy.x_pos >> 16) as i32;
            let ey = (enemy.y_pos >> 16) as i32;
            let dx = (px - ex).abs();
            let dy = (py - ey).abs();
            if dx <= PLAYER_HALF_WIDTH + ENEMY_CONTACT_HALF_WIDTH as i32
                && dy <= ENEMY_CONTACT_HALF_HEIGHT as i32
            {
                enemy.hit_points = enemy.hit_points.saturating_sub(1);
                if enemy.hit_points == 0 {
                    kill_indices.push(i);
                } else {
                    enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                }
            }
        }

        for &idx in kill_indices.iter().rev() {
            let enemy = &self.active_enemies[idx];
            self.enemy_death_effects.push(EnemyDeathEffect {
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                remaining: 10,
            });
            self.active_enemies.remove(idx);
            self.score += 1000;
        }
        if !kill_indices.is_empty() {
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
    }

    /// Step Eyeclops beam: C press fires a horizontal beam projectile.
    pub(crate) fn step_eyeclops_beam(&mut self, slice: &OpeningSliceState, input: FrameInput) {
        if self.current_form != PlayerForm::Eyeclops {
            self.eyeclops_beam_active = false;
            return;
        }

        // Check if beam expired
        if self.eyeclops_beam_active && !self.projectiles.iter().any(|p| p.lifetime > 0) {
            self.eyeclops_beam_active = false;
        }

        // C rising edge fires beam (one at a time)
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && !self.eyeclops_beam_active {
            self.eyeclops_beam_active = true;
            let dir = if slice.player.facing == Direction::Right {
                1
            } else {
                -1
            };
            self.projectiles.push(EnemyProjectile {
                x_pos: slice.player.x_pos,
                y_pos: (slice.player.y_pos as i64 - (8 << 16)) as u32,
                x_velocity: EYECLOPS_BEAM_SPEED * dir,
                y_velocity: 0,
                lifetime: EYECLOPS_BEAM_LIFETIME,
                friendly: true,
                gravity: 0,
                homing: false,
            });
        }
    }

    /// Step Maniaxe thrown axe: boomerang projectile that reverses after 12 frames.
    pub(crate) fn step_maniaxe_axe(&mut self, slice: &OpeningSliceState, input: FrameInput) {
        if self.current_form != PlayerForm::Maniaxe {
            self.maniaxe_axe_active = false;
            return;
        }

        // C rising edge throws axe (only one at a time)
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && !self.maniaxe_axe_active {
            self.maniaxe_axe_active = true;
            self.maniaxe_turnaround = MANIAXE_TRAVEL_FRAMES;
            let dir = if slice.player.facing == Direction::Right {
                1
            } else {
                -1
            };
            let speed = (MANIAXE_BASE_SPEED * dir) << 16;
            self.projectiles.push(EnemyProjectile {
                x_pos: slice.player.x_pos,
                y_pos: (slice.player.y_pos as i64 - (16 << 16)) as u32,
                x_velocity: speed,
                y_velocity: 0,
                lifetime: (MANIAXE_TRAVEL_FRAMES as u16) * 2 + MANIAXE_STARTUP_FRAMES as u16,
                friendly: true,
                gravity: 0,
                homing: false,
            });
        }

        // Turnaround: reverse axe projectile direction after travel frames
        if self.maniaxe_axe_active {
            if self.maniaxe_turnaround > 0 {
                self.maniaxe_turnaround -= 1;
            }
            if self.maniaxe_turnaround == 0 {
                // Reverse the axe projectile direction
                if let Some(p) = self.projectiles.last_mut() {
                    p.x_velocity = -p.x_velocity;
                    self.maniaxe_turnaround = MANIAXE_TRAVEL_FRAMES;
                }
            }
            // Check if axe expired
            if !self.projectiles.iter().any(|p| p.lifetime > 0) {
                self.maniaxe_axe_active = false;
            }
        }
    }

    /// Update Berzerker charge state: builds accumulator when moving fast, activates on overflow.
    pub(crate) fn step_berzerker_charge(&mut self, slice: &OpeningSliceState) {
        if self.current_form != PlayerForm::Berzerker {
            self.berzerker_charging = false;
            self.berzerker_charge_accum = 0;
            return;
        }

        let vel_abs = slice.player.x_velocity.unsigned_abs() as i32;
        if vel_abs < BERZERKER_CHARGE_VEL_THRESHOLD
            || (!slice.player.on_ground && !(self.previous_input.left || self.previous_input.right))
        {
            self.berzerker_charging = false;
            self.berzerker_charge_accum = 0;
            return;
        }

        let (new_accum, overflow) = self
            .berzerker_charge_accum
            .overflowing_add(BERZERKER_CHARGE_ACCUM_STEP);
        if overflow {
            self.berzerker_charge_accum = 0xFF;
            self.berzerker_charging = true;
        } else {
            self.berzerker_charge_accum = new_accum;
        }
    }

    /// Berzerker charge wall break: when charging into a Smash block wall, break it.
    pub(crate) fn berzerker_break_wall_block(&mut self, slice: &OpeningSliceState) {
        let Some(grid) = self.block_grid.as_mut() else {
            return;
        };
        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let moving_right = slice.player.x_velocity > 0
            || (slice.player.x_velocity == 0 && slice.player.facing == Direction::Right);
        let probe = terrain::probe_wall(
            grid,
            x_px,
            y_px,
            PLAYER_HALF_WIDTH,
            PLAYER_HEIGHT,
            moving_right,
        );
        if probe.hit && probe.cell.has_block && probe.cell.block_type == terrain::BlockType::Smash {
            grid.set_cell_at(
                probe.cell_x,
                probe.cell_y,
                terrain::Cell::from_be_bytes(0, 0),
            );
        }
    }

    /// Juggernaut skull shot: C fires a skull projectile with gravity arc.
    pub(crate) fn step_juggernaut_skull(&mut self, slice: &OpeningSliceState, input: FrameInput) {
        if self.current_form != PlayerForm::Juggernaut {
            return;
        }

        let c_pressed = input.c && !self.prior_input.c;
        if !c_pressed {
            return;
        }

        // Max 8 skulls active at once (ASM: cmpi.w #8,($FFFFFB70))
        let active_skulls = self
            .projectiles
            .iter()
            .filter(|p| p.friendly && p.gravity != 0)
            .count();
        if active_skulls >= JUGGERNAUT_MAX_SKULLS {
            return;
        }

        let dir = if slice.player.facing == Direction::Right {
            1
        } else {
            -1
        };
        self.projectiles.push(EnemyProjectile {
            x_pos: slice.player.x_pos,
            y_pos: (slice.player.y_pos as i64 - (8 << 16)) as u32,
            x_velocity: JUGGERNAUT_SKULL_X_SPEED * dir,
            y_velocity: 0,
            lifetime: JUGGERNAUT_SKULL_LIFETIME,
            friendly: true,
            gravity: JUGGERNAUT_SKULL_GRAVITY,
            homing: false,
        });
    }

    /// Diamond Power: Start+A with 20+ diamonds grants invulnerability and
    /// damages nearby enemies. Costs 20 diamonds per activation. Eyeclops excluded.
    pub(crate) fn step_diamond_power(&mut self, input: FrameInput) {
        // Count down active Diamond Power
        if self.diamond_power_frames > 0 {
            self.diamond_power_frames -= 1;
            self.invulnerable_frames = self.invulnerable_frames.max(1);

            // Damage all active non-diamond enemies each frame of Diamond Power
            let mut kill_indices = Vec::new();
            for (i, enemy) in self.active_enemies.iter_mut().enumerate() {
                if matches!(enemy.family, EnemyFamily::Diamond) {
                    continue;
                }
                enemy.hit_points = enemy.hit_points.saturating_sub(1);
                if enemy.hit_points == 0 {
                    kill_indices.push(i);
                } else {
                    enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                }
            }
            for &idx in kill_indices.iter().rev() {
                self.active_enemies.remove(idx);
                self.score += 1000;
            }
            if !kill_indices.is_empty() {
                self.active_enemy_roster = self
                    .active_enemies
                    .iter()
                    .map(ActiveEnemy::snapshot)
                    .collect();
            }
            return;
        }

        // Activation: Start+A held, not Eyeclops, 20+ diamonds
        if !(input.start && input.a) {
            return;
        }
        if self.current_form == PlayerForm::Eyeclops {
            return;
        }
        if self.diamond_count < 20 {
            return;
        }

        // Consume 20 diamonds, activate for 120 frames (2 seconds)
        self.diamond_count -= 20;
        self.diamond_power_frames = 120;
    }
}
