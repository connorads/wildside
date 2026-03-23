use super::*;

/// Diamond Power type active for the current activation.
/// ASM: DiamondPower_Init_Index (kid.asm:44375) maps form+tier to power type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum DiamondPowerType {
    #[default]
    None,
    /// Kid/IronKnight(T1)/Maniaxe(T1): orbiting diamonds damage enemies on contact.
    CircleOfDoom,
    /// Skycutter(T1)/Berzerker(T1): pure invulnerability, 480f.
    Invulnerability,
    /// RedStealth(T1): invulnerability variant, 480f.
    SamuraiHaze,
    /// Cyclone(T1): projectiles rain from top of screen, 720f.
    SlashingRain,
    /// Cyclone(T2): longer rain with tracking, 1800f.
    TrackingRain,
    /// Juggernaut: 5 projectiles in spread, costs 5 diamonds, 80f.
    FiveWayShot,
    /// Micromax(T1): 6 forward-firing diamond chain, 480f.
    MiniSnake,
    /// Micromax(T2)/Kid(T2)/Skycutter(T2)/RedStealth(T2): faster snake, 1200f.
    DeathSnake,
    /// IronKnight(T2): instant +1 HP.
    ExtraHitPoint,
    /// Maniaxe(T2): instant +1 life.
    ExtraLife,
    /// Berzerker(T2): vertical diamond wall slides across screen, 1200f.
    WallOfDeath,
}

/// Map form + tier to Diamond Power type. ASM: DiamondPower_Init_Index.
pub(crate) fn diamond_power_for_form(form: PlayerForm, tier2: bool) -> DiamondPowerType {
    match (form, tier2) {
        (PlayerForm::TheKid, false) => DiamondPowerType::CircleOfDoom,
        (PlayerForm::TheKid, true) => DiamondPowerType::DeathSnake,
        (PlayerForm::Skycutter, false) => DiamondPowerType::Invulnerability,
        (PlayerForm::Skycutter, true) => DiamondPowerType::DeathSnake,
        (PlayerForm::Cyclone, false) => DiamondPowerType::SlashingRain,
        (PlayerForm::Cyclone, true) => DiamondPowerType::TrackingRain,
        (PlayerForm::RedStealth, false) => DiamondPowerType::SamuraiHaze,
        (PlayerForm::RedStealth, true) => DiamondPowerType::DeathSnake,
        (PlayerForm::Eyeclops, _) => DiamondPowerType::None, // blocked
        (PlayerForm::Juggernaut, _) => DiamondPowerType::FiveWayShot,
        (PlayerForm::IronKnight, false) => DiamondPowerType::CircleOfDoom,
        (PlayerForm::IronKnight, true) => DiamondPowerType::ExtraHitPoint,
        (PlayerForm::Berzerker, false) => DiamondPowerType::Invulnerability,
        (PlayerForm::Berzerker, true) => DiamondPowerType::WallOfDeath,
        (PlayerForm::Maniaxe, false) => DiamondPowerType::CircleOfDoom,
        (PlayerForm::Maniaxe, true) => DiamondPowerType::ExtraLife,
        (PlayerForm::Micromax, false) => DiamondPowerType::MiniSnake,
        (PlayerForm::Micromax, true) => DiamondPowerType::DeathSnake,
    }
}

/// Duration in frames for each Diamond Power type.
pub(crate) fn diamond_power_duration(power: DiamondPowerType) -> u16 {
    match power {
        DiamondPowerType::None => 0,
        DiamondPowerType::CircleOfDoom => 480, // indefinite in ASM, use 480 as practical limit
        DiamondPowerType::Invulnerability => 480, // $1E0
        DiamondPowerType::SamuraiHaze => 480,  // $1E0
        DiamondPowerType::SlashingRain => 720, // $2D0
        DiamondPowerType::TrackingRain => 1800, // $708
        DiamondPowerType::FiveWayShot => 80,   // $50
        DiamondPowerType::MiniSnake => 480,    // $1E0
        DiamondPowerType::DeathSnake => 1200,  // $4B0
        DiamondPowerType::ExtraHitPoint => 0,  // instant
        DiamondPowerType::ExtraLife => 0,      // instant
        DiamondPowerType::WallOfDeath => 1200, // $4B0
    }
}

impl NativeRuntime {
    pub(crate) fn step_skycutter_flipboard(
        &mut self,
        slice: &mut OpeningSliceState,
        input: FrameInput,
    ) {
        if self.progression.current_form != PlayerForm::Skycutter {
            self.helmet.skycutter_flipboard = 0;
            self.helmet.skycutter_on_board = false;
            return;
        }

        // C button rising edge → deploy/retract flipboard
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && self.helmet.skycutter_flipboard == 0 {
            self.helmet.skycutter_flipboard = 1;
        }

        // Step flipboard counter
        if self.helmet.skycutter_flipboard > 0 {
            let next = self
                .helmet
                .skycutter_flipboard
                .saturating_add(SKYCUTTER_COUNTER_STEP);
            if next >= SKYCUTTER_FLIPBOARD_TIMEOUT {
                // Animation complete: retract
                self.helmet.skycutter_flipboard = 0;
            } else {
                // Mount toggle at midpoint
                if self.helmet.skycutter_flipboard < SKYCUTTER_MOUNT_FRAME
                    && next >= SKYCUTTER_MOUNT_FRAME
                {
                    self.helmet.skycutter_on_board = !self.helmet.skycutter_on_board;
                }
                self.helmet.skycutter_flipboard = next;
            }
        }

        // While on board: override gravity with Skycutter flight physics
        if self.helmet.skycutter_on_board && !slice.player.on_ground {
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
        if self.helmet.skycutter_on_board && slice.player.on_ground {
            slice.player.y_velocity = SKYCUTTER_LANDING_BOUNCE;
            slice.player.on_ground = false;
            self.helmet.skycutter_flipboard = 1; // restart deploy animation
        }
    }

    /// Step Cyclone vertical flight: acceleration-based with C-button thrust.
    ///
    /// When flying, an acceleration accumulator drifts downward by $400/frame
    /// (gravity). Holding C subtracts $3000 (thrust). The accumulator is added
    /// to y_velocity each frame, with velocity clamped to [-$30000, $28000].
    /// Landing clears the flying flag.
    /// ASM sub_B168 (loc_B214): when Cyclone_flying is already set, replace
    /// normal gravity with the flight accumulator. The thrust check uses
    /// `bclr #Button_C,(Ctrl_Pressed)` — a rising-edge test that consumes the
    /// C press bit so kid_flip won't see it. Native equivalent: check
    /// `previous_input` for 1-frame edge detection (not `prior_input` which
    /// has a 2-frame gap).
    pub(crate) fn step_cyclone_flight(&mut self, slice: &mut OpeningSliceState, input: FrameInput) {
        if self.progression.current_form != PlayerForm::Cyclone {
            self.helmet.cyclone_flying = false;
            self.helmet.cyclone_y_accel = 0;
            return;
        }

        // Landing clears flight
        if self.helmet.cyclone_flying && slice.player.on_ground {
            self.helmet.cyclone_flying = false;
            return;
        }

        if !self.helmet.cyclone_flying {
            return;
        }

        // ASM uses `addi.w` which wraps on M68K 16-bit word arithmetic.
        // Must use wrapping_add, not saturating_add — the wrap behaviour is
        // load-bearing when the accumulator gets very negative.
        self.helmet.cyclone_y_accel = self
            .helmet
            .cyclone_y_accel
            .wrapping_add(CYCLONE_ACCEL_GRAVITY);

        // ASM: `bclr #Button_C,(Ctrl_Pressed)` — thrust on C rising edge only,
        // not while held. Uses previous_input for proper 1-frame edge detection
        // (prior_input is 2 frames back due to gameplay_input asymmetry).
        let c_newly_pressed = input.c && !self.previous_input.c;
        if c_newly_pressed {
            // ASM `subi.w` wraps: e.g. 0xAA00 - 0x3000 = 0x7A00 (negative →
            // positive wrap). This is intentional — the wrap produces a sudden
            // downward push that's part of the flight feel.
            self.helmet.cyclone_y_accel = self
                .helmet
                .cyclone_y_accel
                .wrapping_sub(CYCLONE_ACCEL_THRUST);
            if self.helmet.cyclone_y_accel < CYCLONE_ACCEL_MIN {
                self.helmet.cyclone_y_accel = CYCLONE_ACCEL_MIN;
            }
        } else {
            // Not thrusting: cap descent acceleration
            if self.helmet.cyclone_y_accel > CYCLONE_ACCEL_MAX_DESCENT {
                self.helmet.cyclone_y_accel = CYCLONE_ACCEL_MAX_DESCENT;
            }
        }

        // Apply acceleration to velocity
        slice.player.y_velocity += i32::from(self.helmet.cyclone_y_accel);

        // Clamp velocity to rise/fall limits.
        slice.player.y_velocity = slice
            .player
            .y_velocity
            .clamp(CYCLONE_MAX_RISE_VEL, CYCLONE_MAX_FALL_VEL);
    }

    /// ASM kid_flip (loc_ACB0): C rising edge activates flight when airborne.
    /// Runs AFTER physics, so flight takes effect on the next frame. Uses
    /// `bclr #Button_C,(Ctrl_Pressed)` which is consumed by sub_B168 when
    /// already flying — so only activates when cyclone_flying is false.
    pub(crate) fn step_cyclone_activation(&mut self, slice: &OpeningSliceState, input: FrameInput) {
        if self.progression.current_form != PlayerForm::Cyclone {
            return;
        }
        // Only activate when not already flying (ASM: sub_B168 consumes C
        // pressed bit via bclr when flying, so kid_flip never sees it).
        if self.helmet.cyclone_flying {
            return;
        }
        let c_pressed = input.c && !self.previous_input.c;
        if c_pressed && !slice.player.on_ground {
            self.helmet.cyclone_flying = true;
            self.helmet.cyclone_y_accel = CYCLONE_ACCEL_INITIAL;
        }
    }

    /// IronKnight wall climb: the ROM accepts the attach from the grounded
    /// walking path as well as from airborne wall contact. C presses while
    /// climbing boost upward, natural drift slides down.
    pub(crate) fn step_iron_knight_climb(
        &mut self,
        slice: &mut OpeningSliceState,
        input: FrameInput,
    ) {
        if self.progression.current_form != PlayerForm::IronKnight {
            self.helmet.iron_knight_climbing = false;
            self.helmet.iron_knight_climb_accel = 0;
            return;
        }

        let c_pressed = input.c && !self.prior_input.c;

        // ROM walking state `loc_7664` and jump state `kid_flip` both allow a
        // raw C edge to enter wall climbing when Iron Knight is pressed against
        // a wall in the facing direction.
        if c_pressed
            && !self.helmet.iron_knight_climbing
            && let Some(ref grid) = self.block_grid
        {
            let facing_right = slice.player.facing == Direction::Right;
            let px = (slice.player.x_pos >> 16) as i32;
            let py = (slice.player.y_pos >> 16) as i32;
            let probe = if slice.player.on_ground {
                // ROM walking-state `loc_7664` uses `sub_922C`: a single edge
                // column at half_width + 1 through the form's full height.
                terrain::probe_wall_column(
                    grid,
                    px,
                    py,
                    self.progression.current_form.half_width(),
                    PLAYER_HEIGHT,
                    facing_right,
                )
            } else {
                terrain::probe_wall(grid, px, py, 5, 31, facing_right)
            };
            if probe.hit {
                self.helmet.iron_knight_climbing = true;
                self.helmet.iron_knight_climb_accel = 0;
                slice.player.y_velocity = IRON_KNIGHT_ATTACH_Y_VEL;
                slice.player.x_velocity = 0;
                slice.player.on_ground = false;
                return;
            }
        }

        if !self.helmet.iron_knight_climbing {
            return;
        }

        // Landing stops climb
        if slice.player.on_ground {
            self.helmet.iron_knight_climbing = false;
            self.helmet.iron_knight_climb_accel = 0;
            return;
        }

        // B press = jump off wall
        let b_pressed = input.b && !self.prior_input.b;
        if b_pressed {
            self.helmet.iron_knight_climbing = false;
            self.helmet.iron_knight_climb_accel = 0;
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
                self.helmet.iron_knight_climbing = false;
                self.helmet.iron_knight_climb_accel = 0;
                return;
            }
        }

        // Accumulate drift (sliding down)
        self.helmet.iron_knight_climb_accel = self
            .helmet
            .iron_knight_climb_accel
            .saturating_add(IRON_KNIGHT_SLIDE_DRIFT);

        // C press: boost upward
        if c_pressed {
            self.helmet.iron_knight_climb_accel = self
                .helmet
                .iron_knight_climb_accel
                .saturating_sub(IRON_KNIGHT_CLIMB_BOOST);
            if self.helmet.iron_knight_climb_accel < IRON_KNIGHT_MIN_ACCEL {
                self.helmet.iron_knight_climb_accel = IRON_KNIGHT_MIN_ACCEL;
            }
        } else {
            // Cap slide-down acceleration
            if self.helmet.iron_knight_climb_accel > IRON_KNIGHT_MAX_SLIDE_ACCEL {
                self.helmet.iron_knight_climb_accel = IRON_KNIGHT_MAX_SLIDE_ACCEL;
            }
        }

        // Apply acceleration to velocity
        slice.player.y_velocity += i32::from(self.helmet.iron_knight_climb_accel);

        // Clamp velocity to climb/fall limits.
        slice.player.y_velocity = slice
            .player
            .y_velocity
            .clamp(IRON_KNIGHT_MAX_CLIMB_VEL, IRON_KNIGHT_MAX_FALL_VEL);

        // Zero horizontal velocity while climbing
        slice.player.x_velocity = 0;
    }

    /// Micromax wall/ceiling stick: auto-attach on contact while airborne,
    /// reduced gravity while stuck, B press detaches with kick velocity.
    pub(crate) fn step_micromax_stick(&mut self, slice: &mut OpeningSliceState, input: FrameInput) {
        // Only active for Micromax form
        if self.progression.current_form != PlayerForm::Micromax {
            self.helmet.micromax_stuck = false;
            return;
        }

        // Already stuck: handle slide and detach
        if self.helmet.micromax_stuck {
            let b_pressed = input.b && !self.prior_input.b;
            if b_pressed {
                // Detach: kick upward and away
                self.helmet.micromax_stuck = false;
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
                self.helmet.micromax_stuck = false;
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
                    self.helmet.micromax_stuck = false;
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
                self.helmet.micromax_stuck = true;
                slice.player.y_velocity = 0;
                slice.player.x_velocity = 0;
                return;
            }

            // Check wall contact in facing direction
            let facing_right = slice.player.facing == Direction::Right;
            let wall = terrain::probe_wall(grid, px, py, MICROMAX_HALF_W, MICROMAX_H, facing_right);
            if wall.hit {
                self.helmet.micromax_stuck = true;
                slice.player.y_velocity = 0;
                slice.player.x_velocity = 0;
            }
        }
    }

    /// Step RedStealth sword swing: C press starts swing, hitbox active after startup.
    pub(crate) fn step_red_stealth_sword(&mut self, input: FrameInput) {
        if self.progression.current_form != PlayerForm::RedStealth {
            self.helmet.red_stealth_sword_timer = 0;
            return;
        }

        // C rising edge starts a new swing (only if not already swinging)
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && self.helmet.red_stealth_sword_timer == 0 {
            self.helmet.red_stealth_sword_timer =
                RED_STEALTH_STARTUP_FRAMES + RED_STEALTH_ACTIVE_FRAMES;
        }

        // Tick down active timer
        if self.helmet.red_stealth_sword_timer > 0 {
            self.helmet.red_stealth_sword_timer -= 1;
        }
    }

    /// Returns true if the sword hitbox is active (past startup, within active frames).
    pub(crate) fn sword_is_active(&self) -> bool {
        self.helmet.red_stealth_sword_timer > 0
            && self.helmet.red_stealth_sword_timer <= RED_STEALTH_ACTIVE_FRAMES
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
            self.spawn_enemy_death_effect(enemy.family, enemy.x_pos, enemy.y_pos);
            self.active_enemies.remove(idx);
            self.add_score(1000);
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
            self.spawn_enemy_death_effect(enemy.family, enemy.x_pos, enemy.y_pos);
            self.active_enemies.remove(idx);
            self.add_score(1000);
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
        if !self.helmet.berzerker_charging {
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
            self.spawn_enemy_death_effect(enemy.family, enemy.x_pos, enemy.y_pos);
            self.active_enemies.remove(idx);
            self.add_score(1000);
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
        if self.progression.current_form != PlayerForm::Eyeclops {
            self.helmet.eyeclops_beam_active = false;
            return;
        }

        // Check if beam expired
        if self.helmet.eyeclops_beam_active && !self.projectiles.iter().any(|p| p.lifetime > 0) {
            self.helmet.eyeclops_beam_active = false;
        }

        // C rising edge fires beam (one at a time)
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && !self.helmet.eyeclops_beam_active {
            self.helmet.eyeclops_beam_active = true;
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
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
            });
        }
    }

    /// Step Maniaxe thrown axe: boomerang projectile that reverses after 12 frames.
    pub(crate) fn step_maniaxe_axe(&mut self, slice: &OpeningSliceState, input: FrameInput) {
        if self.progression.current_form != PlayerForm::Maniaxe {
            self.helmet.maniaxe_axe_active = false;
            return;
        }

        // C rising edge throws axe (only one at a time)
        let c_pressed = input.c && !self.prior_input.c;
        if c_pressed && !self.helmet.maniaxe_axe_active {
            self.helmet.maniaxe_axe_active = true;
            self.helmet.maniaxe_turnaround = MANIAXE_TRAVEL_FRAMES;
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
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
            });
        }

        // Turnaround: reverse axe projectile direction after travel frames
        if self.helmet.maniaxe_axe_active {
            if self.helmet.maniaxe_turnaround > 0 {
                self.helmet.maniaxe_turnaround -= 1;
            }
            if self.helmet.maniaxe_turnaround == 0 {
                // Reverse the axe projectile direction
                if let Some(p) = self.projectiles.last_mut() {
                    p.x_velocity = -p.x_velocity;
                    self.helmet.maniaxe_turnaround = MANIAXE_TRAVEL_FRAMES;
                }
            }
            // Check if axe expired
            if !self.projectiles.iter().any(|p| p.lifetime > 0) {
                self.helmet.maniaxe_axe_active = false;
            }
        }
    }

    /// Update Berzerker charge state: builds accumulator when moving fast, activates on overflow.
    pub(crate) fn step_berzerker_charge(&mut self, slice: &OpeningSliceState) {
        if self.progression.current_form != PlayerForm::Berzerker {
            self.helmet.berzerker_charging = false;
            self.helmet.berzerker_charge_accum = 0;
            return;
        }

        let vel_abs = slice.player.x_velocity.unsigned_abs() as i32;
        if vel_abs < BERZERKER_CHARGE_VEL_THRESHOLD
            || !(slice.player.on_ground || self.previous_input.left || self.previous_input.right)
        {
            self.helmet.berzerker_charging = false;
            self.helmet.berzerker_charge_accum = 0;
            return;
        }

        let (new_accum, overflow) = self
            .helmet
            .berzerker_charge_accum
            .overflowing_add(BERZERKER_CHARGE_ACCUM_STEP);
        if overflow {
            self.helmet.berzerker_charge_accum = 0xFF;
            self.helmet.berzerker_charging = true;
        } else {
            self.helmet.berzerker_charge_accum = new_accum;
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
        if self.progression.current_form != PlayerForm::Juggernaut {
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
            homing_accel: 0,
            speed_cap: 0,
            wall_bounce: false,
        });
    }

    /// Diamond Power: Start+A with 20+ diamonds activates a per-form special.
    /// ASM: DiamondPower_Run (kid.asm:44331). Eyeclops excluded.
    ///
    /// Two tiers: 20 diamonds = basic power, 50 diamonds (20+30) = enhanced power.
    /// Each form has a unique effect per tier (ASM: DiamondPower_Init_Index).
    pub(crate) fn step_diamond_power(&mut self, input: FrameInput) {
        // Count down active Diamond Power
        if self.progression.diamond_power_frames > 0 {
            self.progression.diamond_power_frames -= 1;

            match self.progression.diamond_power_type {
                DiamondPowerType::CircleOfDoom | DiamondPowerType::WallOfDeath => {
                    // AoE damage: kill enemies each frame (orbiting diamonds / wall)
                    self.progression.invulnerable_frames =
                        self.progression.invulnerable_frames.max(1);
                    self.diamond_power_kill_enemies();
                }
                DiamondPowerType::Invulnerability | DiamondPowerType::SamuraiHaze => {
                    // Pure invulnerability, no AoE damage
                    self.progression.invulnerable_frames =
                        self.progression.invulnerable_frames.max(1);
                }
                DiamondPowerType::SlashingRain | DiamondPowerType::TrackingRain => {
                    // Rain projectiles from top of screen every 4 frames
                    if self.progression.diamond_power_frames.is_multiple_of(4)
                        && self.projectiles.len() < MAX_PROJECTILES
                    {
                        let slice = self.opening_slice.as_ref();
                        let cam_x = slice.map(|s| s.camera.x_pos).unwrap_or(0);
                        // Random-ish X across screen width (320px)
                        let offset = ((self.progression.diamond_power_frames as u32)
                            .wrapping_mul(137))
                            % 320;
                        let rain_x = (cam_x as i64 + ((offset as i64) << 16)) as u32;
                        let rain_y = slice.map(|s| s.camera.y_pos).unwrap_or(0);
                        self.projectiles.push(EnemyProjectile {
                            x_pos: rain_x,
                            y_pos: rain_y,
                            x_velocity: 0,
                            y_velocity: 0x3_0000, // fast downward
                            lifetime: 60,
                            gravity: 0,
                            friendly: true, // damages enemies, not player
                            homing_accel: 0,
                            speed_cap: 0,
                            wall_bounce: false,
                        });
                    }
                }
                DiamondPowerType::FiveWayShot => {
                    // 5-way spread on first frame only
                    if self.progression.diamond_power_frames
                        == diamond_power_duration(DiamondPowerType::FiveWayShot) - 1
                    {
                        let slice = self.opening_slice.as_ref();
                        let (px, py) = slice
                            .map(|s| (s.player.x_pos, s.player.y_pos))
                            .unwrap_or((0, 0));
                        let spreads: [(i32, i32); 5] = [
                            (0x4_0000, -0x2_0000),  // far right, up
                            (0x2_0000, -0x3_0000),  // right, steep up
                            (0, -0x4_0000),         // straight up
                            (-0x2_0000, -0x3_0000), // left, steep up
                            (-0x4_0000, -0x2_0000), // far left, up
                        ];
                        for (xv, yv) in spreads {
                            if self.projectiles.len() < MAX_PROJECTILES {
                                self.projectiles.push(EnemyProjectile {
                                    x_pos: px,
                                    y_pos: py,
                                    x_velocity: xv,
                                    y_velocity: yv,
                                    lifetime: 60,
                                    gravity: 0x2000,
                                    friendly: true,
                                    homing_accel: 0,
                                    speed_cap: 0,
                                    wall_bounce: false,
                                });
                            }
                        }
                    }
                }
                DiamondPowerType::MiniSnake | DiamondPowerType::DeathSnake => {
                    // Forward-firing diamond chain: spawn projectile every 8 frames
                    self.progression.invulnerable_frames =
                        self.progression.invulnerable_frames.max(1);
                    if self.progression.diamond_power_frames.is_multiple_of(8)
                        && self.projectiles.len() < MAX_PROJECTILES
                    {
                        let slice = self.opening_slice.as_ref();
                        let (px, py, facing) = slice
                            .map(|s| (s.player.x_pos, s.player.y_pos, s.player.facing))
                            .unwrap_or((0, 0, Direction::Right));
                        let speed =
                            if self.progression.diamond_power_type == DiamondPowerType::MiniSnake {
                                0x3_0000 // 0x300 scaled
                            } else {
                                0x4_0000 // 0x400 scaled
                            };
                        let xv = if facing == Direction::Right {
                            speed
                        } else {
                            -speed
                        };
                        self.projectiles.push(EnemyProjectile {
                            x_pos: px,
                            y_pos: py,
                            x_velocity: xv,
                            y_velocity: 0,
                            lifetime: 90,
                            gravity: 0,
                            friendly: true,
                            homing_accel: 0,
                            speed_cap: 0,
                            wall_bounce: false,
                        });
                    }
                }
                DiamondPowerType::None
                | DiamondPowerType::ExtraHitPoint
                | DiamondPowerType::ExtraLife => {}
            }

            if self.progression.diamond_power_frames == 0 {
                self.progression.diamond_power_type = DiamondPowerType::None;
            }
            return;
        }

        // Activation: Start+A held, not Eyeclops, 20+ diamonds
        // ASM: DiamondPower_Check (kid.asm:44357)
        if !(input.start && input.a) {
            return;
        }
        if self.progression.current_form == PlayerForm::Eyeclops {
            return;
        }

        // Juggernaut 5-Way Shot only costs 5 diamonds (ASM: special case)
        let cost = if self.progression.current_form == PlayerForm::Juggernaut {
            5
        } else {
            20
        };
        if self.progression.diamond_count < cost {
            return;
        }

        // Two-tier diamond cost (ASM: DiamondPower_Check):
        // Always costs base amount. If remaining >= 30 after first deduction, costs 30 more
        // (50 total for non-Juggernaut) for enhanced tier-2 power.
        self.progression.diamond_count -= cost;
        let tier2 = if self.progression.current_form != PlayerForm::Juggernaut
            && self.progression.diamond_count >= 30
        {
            self.progression.diamond_count -= 30;
            true
        } else {
            false
        };

        let power = diamond_power_for_form(self.progression.current_form, tier2);
        self.progression.diamond_power_type = power;

        // Instant effects (no timer)
        match power {
            DiamondPowerType::ExtraHitPoint => {
                self.progression.number_hitpoints =
                    self.progression.number_hitpoints.saturating_add(1);
                self.progression.diamond_power_frames = 1; // single frame for visual feedback
                return;
            }
            DiamondPowerType::ExtraLife => {
                self.progression.number_lives = self.progression.number_lives.saturating_add(1);
                self.progression.diamond_power_frames = 1;
                return;
            }
            _ => {}
        }

        self.progression.diamond_power_frames = diamond_power_duration(power);
    }

    /// AoE damage helper: kill all non-diamond enemies (used by CircleOfDoom, WallOfDeath).
    fn diamond_power_kill_enemies(&mut self) {
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
            self.add_score(1000);
        }
        if !kill_indices.is_empty() {
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
    }
}
