use super::*;

impl NativeRuntime {
    pub(crate) fn should_apply_level_one_opening_hit(
        &self,
        slice: &OpeningSliceState,
        input: FrameInput,
    ) -> bool {
        let hit_elapsed_frames = if input.a {
            LEVEL_ONE_OPENING_HIT_LEVEL_ELAPSED_FRAMES_RUN
        } else {
            LEVEL_ONE_OPENING_HIT_LEVEL_ELAPSED_FRAMES_WALK
        };
        self.current_level == LevelId(0)
            && self.invulnerable_frames == 0
            && input.right
            && self.number_hitpoints == self.current_form.max_hitpoints()
            && self
                .level_elapsed_frames()
                .is_some_and(|elapsed| elapsed >= hit_elapsed_frames && slice.camera.x_pos == 0)
    }

    /// Check for stomp-kills: player landing on enemies from above.
    /// Decrements HP on stomp; removes enemy at 0 HP. Always bounces player.
    pub(crate) fn process_stomps(
        &mut self,
        slice: &mut OpeningSliceState,
        pre_physics_y_velocity: i32,
        pre_physics_y_pos: u32,
    ) -> bool {
        if !matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
            return false;
        }

        let mut stomped_indices = Vec::new();
        for (i, enemy) in self.active_enemy_roster.iter().enumerate() {
            let is_diamond = matches!(enemy.kind, NativeObjectKind::Enemy { object_type: 0x01 });
            if is_diamond {
                continue;
            }
            // Skip invulnerable enemies (e.g. submerged TarMonster)
            if i < self.active_enemies.len() && !self.active_enemies[i].is_vulnerable() {
                continue;
            }
            if classify_enemy_contact(
                slice.player,
                enemy,
                pre_physics_y_velocity,
                pre_physics_y_pos,
            ) == EnemyContactResult::Stomp
            {
                stomped_indices.push(i);
            }
        }

        if stomped_indices.is_empty() {
            return false;
        }

        // Decrement HP; collect indices of enemies that should be removed
        let mut kill_indices = Vec::new();
        for &roster_idx in &stomped_indices {
            if roster_idx < self.active_enemies.len() {
                let enemy = &mut self.active_enemies[roster_idx];
                enemy.hit_points = enemy.hit_points.saturating_sub(1);
                if enemy.hit_points == 0 {
                    kill_indices.push(roster_idx);
                } else {
                    enemy.damage_flash_frames = ENEMY_DAMAGE_FLASH_FRAMES;
                    // Bosses retaliate with projectiles when damaged
                    if matches!(
                        enemy.family,
                        EnemyFamily::HeadyMetal
                            | EnemyFamily::Shishkaboss
                            | EnemyFamily::BoomerangBosses
                            | EnemyFamily::BagelBrothers
                    ) {
                        let boss_x = enemy.x_pos;
                        let boss_y = enemy.y_pos;
                        let is_heady = matches!(enemy.family, EnemyFamily::HeadyMetal);
                        self.projectiles.push(EnemyProjectile {
                            x_pos: boss_x,
                            y_pos: boss_y,
                            x_velocity: 0x2_0000,
                            y_velocity: -0x1_0000,
                            lifetime: if is_heady { 120 } else { 90 },
                            gravity: if is_heady { 0x2710 } else { 0x800 },
                            friendly: false,
                            homing: is_heady,
                        });
                        self.projectiles.push(EnemyProjectile {
                            x_pos: boss_x,
                            y_pos: boss_y,
                            x_velocity: -0x2_0000,
                            y_velocity: -0x1_0000,
                            lifetime: if is_heady { 120 } else { 90 },
                            gravity: if is_heady { 0x2710 } else { 0x800 },
                            friendly: false,
                            homing: is_heady,
                        });
                    }
                    // TarMonster fires 5 tar blobs in a spread pattern when damaged
                    if matches!(enemy.family, EnemyFamily::TarMonster) {
                        let tx = enemy.x_pos;
                        let ty = enemy.y_pos;
                        spawn_tar_monster_retaliation(&mut self.projectiles, tx, ty);
                    }
                }
            }
        }

        self.stomped_this_frame = true;

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

        // Rebuild roster after changes
        self.active_enemy_roster = self
            .active_enemies
            .iter()
            .map(ActiveEnemy::snapshot)
            .collect();

        // Bounce the player upward
        slice.player.y_velocity = STOMP_BOUNCE_VELOCITY;
        slice.player.on_ground = false;
        true
    }

    /// Collect Diamond enemies on touch: remove from active list, increment count.
    pub(crate) fn collect_diamonds(&mut self, slice: &OpeningSliceState) {
        if !matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
            return;
        }

        let mut collected_indices = Vec::new();
        for (i, enemy) in self.active_enemies.iter().enumerate() {
            if enemy.family != EnemyFamily::Diamond {
                continue;
            }
            let snapshot = NativeObjectSnapshot {
                kind: NativeObjectKind::Enemy {
                    object_type: enemy.family.object_type(),
                },
                x_pos: enemy.x_pos,
                y_pos: enemy.y_pos,
                layout_index: Some(enemy.layout_index),
                x_velocity: Some(enemy.x_velocity),
                y_velocity: Some(enemy.y_velocity),
                facing: Some(enemy.facing),
                on_ground: Some(enemy.on_ground),
                hit_points: Some(enemy.hit_points),
                damage_flash: false,
            };
            if player_touches_enemy(slice.player, &snapshot) {
                collected_indices.push(i);
            }
        }

        for &idx in collected_indices.iter().rev() {
            self.active_enemies.remove(idx);
            self.collect_diamond();
        }

        if !collected_indices.is_empty() {
            // Rebuild roster after removal
            self.active_enemy_roster = self
                .active_enemies
                .iter()
                .map(ActiveEnemy::snapshot)
                .collect();
        }
    }

    /// Add one life, capped at 9 per ASM.
    pub(crate) fn add_life(&mut self) {
        if self.number_lives < 9 {
            self.number_lives += 1;
        }
    }

    /// Increment diamond count (capped at 99 per ASM). Awards extra life at 50/100.
    pub(crate) fn collect_diamond(&mut self) {
        if self.diamond_count >= 99 {
            return;
        }
        self.diamond_count += 1;
        self.score += 500;
        if self.diamond_count % 50 == 0 {
            self.add_life();
        }
    }

    pub(crate) fn apply_prize_content(&mut self, content: terrain::PrizeContent) {
        match content {
            terrain::PrizeContent::Diamond => {
                self.collect_diamond();
            }
            terrain::PrizeContent::TenDiamonds => {
                for _ in 0..10 {
                    self.collect_diamond();
                }
            }
            terrain::PrizeContent::Clock => {
                self.level_timer.add_clock_bonus();
            }
            terrain::PrizeContent::Continue => {
                self.number_continues = self.number_continues.saturating_add(1);
            }
            terrain::PrizeContent::Ankh => {
                self.add_life();
            }
            terrain::PrizeContent::Points10000 => {
                self.score += 10_000;
            }
            terrain::PrizeContent::Skycutter => self.queue_form_change(PlayerForm::Skycutter),
            terrain::PrizeContent::Cyclone => self.queue_form_change(PlayerForm::Cyclone),
            terrain::PrizeContent::RedStealth => self.queue_form_change(PlayerForm::RedStealth),
            terrain::PrizeContent::Eyeclops => self.queue_form_change(PlayerForm::Eyeclops),
            terrain::PrizeContent::Juggernaut => self.queue_form_change(PlayerForm::Juggernaut),
            terrain::PrizeContent::IronKnight => self.queue_form_change(PlayerForm::IronKnight),
            terrain::PrizeContent::Berzerker => self.queue_form_change(PlayerForm::Berzerker),
            terrain::PrizeContent::Maniaxe => self.queue_form_change(PlayerForm::Maniaxe),
            terrain::PrizeContent::Micromax => self.queue_form_change(PlayerForm::Micromax),
        }
    }

    /// Returns the x_pos of the damaging enemy if contact damage should apply, or None.
    pub(crate) fn find_contact_damage_source(&self, slice: &OpeningSliceState) -> Option<u32> {
        if self.invulnerable_frames != 0 || !matches!(self.player_lifecycle, PlayerLifecycle::Alive)
        {
            return None;
        }
        // Berzerker charging is immune to contact damage (they damage enemies instead)
        if self.berzerker_charging {
            return None;
        }

        self.active_enemies.iter().find_map(|enemy| {
            // All enemies damage on contact except collectible diamonds
            if matches!(enemy.family, EnemyFamily::Diamond) {
                return None;
            }
            // Skip invulnerable enemies (e.g. submerged TarMonster)
            if !enemy.is_vulnerable() {
                return None;
            }
            let snapshot = enemy.snapshot();
            if player_touches_enemy(slice.player, &snapshot) {
                Some(enemy.x_pos)
            } else {
                None
            }
        })
    }

    pub(crate) fn fell_below_level(&self, slice: &OpeningSliceState) -> bool {
        let player_y_px = slice.player.y_pos >> 16;
        player_y_px > u32::from(slice.level_bounds.height_pixels)
    }

    pub(crate) fn player_touches_flag(&self, slice: &OpeningSliceState) -> bool {
        let Some(header) = self.level_header(self.current_level) else {
            return false;
        };
        let flag_x_fp = u32::from(header.flag_x) << 16;
        let flag_y_fp = u32::from(header.flag_y) << 16;
        let half_fp = FLAG_HIT_HALF_SIZE << 16;
        let dx = slice.player.x_pos.abs_diff(flag_x_fp);
        let dy = slice.player.y_pos.abs_diff(flag_y_fp);
        dx <= half_fp && dy <= half_fp
    }

    /// Check if the player is standing on a teleporter block and return the destination.
    pub(crate) fn check_teleporter_contact(
        &self,
        slice: &OpeningSliceState,
    ) -> Option<(u8, u16, u8)> {
        let block_list = self.block_list.as_ref()?;
        if block_list.teleporters.is_empty() {
            return None;
        }
        // Player's feet position in cell coordinates
        let px = (slice.player.x_pos >> 16) as i32;
        let py = (slice.player.y_pos >> 16) as i32;
        let col = terrain::pixel_to_cell(px) as u16;
        let row = terrain::pixel_to_cell(py) as u16;

        block_list
            .teleporter_at(col, row)
            .map(|t| (t.dest_map_id, t.dest_x, t.dest_y))
    }

    pub(crate) fn should_apply_level_one_late_hazard(&self, slice: &OpeningSliceState) -> bool {
        self.current_level == LevelId(0)
            && self.number_hitpoints == 1
            && self.invulnerable_frames == 0
            && slice.camera.x_pos > LEVEL_ONE_LATE_HAZARD_CAMERA_X_FP
            && self
                .level_elapsed_frames()
                .is_some_and(|elapsed| elapsed >= LEVEL_ONE_LATE_HAZARD_LEVEL_ELAPSED_FRAMES)
    }

    pub(crate) fn queue_form_change(&mut self, form: PlayerForm) {
        self.pending_form = Some(form);
    }

    pub(crate) fn resolve_pending_form_change(&mut self) {
        let Some(form) = self.pending_form.take() else {
            return;
        };
        self.current_form = form;
        self.number_hitpoints = form.max_hitpoints();
        // Clear ability state on form change
        self.skycutter_flipboard = 0;
        self.skycutter_on_board = false;
        self.cyclone_flying = false;
        self.cyclone_y_accel = 0;
        self.red_stealth_sword_timer = 0;
        self.eyeclops_beam_active = false;
        self.maniaxe_axe_active = false;
        self.maniaxe_turnaround = 0;
        self.berzerker_charging = false;
        self.berzerker_charge_accum = 0;
        self.iron_knight_climbing = false;
        self.iron_knight_climb_accel = 0;
        self.micromax_stuck = false;
    }

    pub(crate) fn apply_damage_with_knockback(
        &mut self,
        slice: &mut OpeningSliceState,
        enemy_x: u32,
    ) {
        let dies = self.current_form == PlayerForm::TheKid && self.number_hitpoints <= 1;
        self.apply_damage();

        // No knockback if the hit kills (death windup takes over)
        if !dies {
            let push_left = enemy_x > slice.player.x_pos;
            slice.player.x_velocity = if push_left {
                -KNOCKBACK_X_VELOCITY
            } else {
                KNOCKBACK_X_VELOCITY
            };
            slice.player.y_velocity = KNOCKBACK_Y_VELOCITY;
            slice.player.on_ground = false;
        }
    }

    pub(crate) fn apply_damage(&mut self) {
        self.level_damage_taken = true;
        if self.current_form != PlayerForm::TheKid {
            self.queue_form_change(PlayerForm::TheKid);
            self.resolve_pending_form_change();
            self.invulnerable_frames = PLAYER_DAMAGE_INVULNERABLE_FRAMES;
            return;
        }

        if self.number_hitpoints > 1 {
            self.number_hitpoints -= 1;
            self.invulnerable_frames = PLAYER_DAMAGE_INVULNERABLE_FRAMES;
            return;
        }

        self.number_hitpoints = 0;
        self.begin_death_windup();
    }

    pub(crate) fn begin_death_windup(&mut self) {
        self.paused = false;
        self.pause_toggle_pending = false;
        self.invulnerable_frames = 0;
        self.death_camera_lock = self.opening_slice.map(|slice| slice.camera);
        self.player_lifecycle = PlayerLifecycle::DeathWindup {
            remaining_frames: PLAYER_DEATH_WINDUP_FRAMES,
        };
    }

    pub(crate) fn advance_death_windup_motion(&mut self) {
        if matches!(self.boot_state, BootState::LevelSkipHold) || self.paused {
            return;
        }
        if self.current_level == LevelId(0)
            && self
                .death_camera_lock
                .is_some_and(|camera| camera.x_pos == LEVEL_ONE_LATE_HAZARD_CAMERA_X_FP)
        {
            let bounds = self.level_bounds(self.current_level);
            let mut slice = self
                .opening_slice
                .unwrap_or_else(|| OpeningSliceState::new(bounds));
            let first_windup_frame = matches!(
                self.player_lifecycle,
                PlayerLifecycle::DeathWindup { remaining_frames }
                    if remaining_frames == PLAYER_DEATH_WINDUP_FRAMES
            );
            if first_windup_frame {
                slice.player.on_ground = false;
                slice.player.x_velocity = LEVEL_ONE_LATE_HAZARD_DEATH_X_VELOCITY_FP;
                slice.player.y_velocity = LEVEL_ONE_LATE_HAZARD_DEATH_Y_VELOCITY_FP;
            } else {
                let next_x = i64::from(slice.player.x_pos) + i64::from(slice.player.x_velocity);
                let next_y = i64::from(slice.player.y_pos) + i64::from(slice.player.y_velocity);
                slice.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
                slice.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
                slice.player.y_velocity = slice
                    .player
                    .y_velocity
                    .saturating_add(LEVEL_ONE_LATE_HAZARD_DEATH_GRAVITY_FP);
                slice.player.on_ground = false;
            }
            if let Some(camera_lock) = self.death_camera_lock {
                slice.camera = camera_lock;
            }
            self.opening_slice = Some(slice);
            return;
        }
        let level_elapsed = self.level_elapsed_frames().unwrap_or(0);
        let bounds = self.level_bounds(self.current_level);
        let mut slice = self
            .opening_slice
            .unwrap_or_else(|| OpeningSliceState::new(bounds));
        slice.step(
            level_elapsed,
            FrameInput::default(),
            false,
            self.current_form,
            self.block_grid.as_mut(),
            self.block_list.as_ref(),
            false,
        );
        if let Some(camera_lock) = self.death_camera_lock {
            slice.camera = camera_lock;
        }
        self.opening_slice = Some(slice);
    }

    pub(crate) fn enter_death_hidden_state(&mut self) {
        self.current_form = PlayerForm::TheKid;
        self.pending_form = None;
        self.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
        self.number_lives = self.number_lives.saturating_sub(1);
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.evanescent_blocks.clear();
        self.ghost_groups.clear();
        self.death_camera_lock = None;
        self.player_lifecycle = PlayerLifecycle::Dying {
            remaining_frames: PLAYER_DEATH_HIDE_FRAMES,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyContactResult {
    None,
    Stomp,
    Damage,
}

/// Classify contact between player and enemy as stomp, damage, or no contact.
///
/// Stomp: player is falling (y_velocity > 0) and the player's feet are at or
/// above the enemy's top edge. Contacts lower on the sprite deal damage
/// instead of granting a bounce.
pub(crate) fn classify_enemy_contact(
    player: PlayerState,
    enemy: &NativeObjectSnapshot,
    pre_physics_y_velocity: i32,
    pre_physics_y_pos: u32,
) -> EnemyContactResult {
    let px = player.x_pos as i64;
    let py = player.y_pos as i64;
    let p_left = px - (PLAYER_CONTACT_HALF_WIDTH << 16);
    let p_right = px + (PLAYER_CONTACT_HALF_WIDTH << 16);
    let p_top = py - (i64::from(PLAYER_HEIGHT) << 16);
    let p_bottom = py;

    let ex = enemy.x_pos as i64;
    let ey = enemy.y_pos as i64;
    let stomp_enemy_half_width = ENEMY_CONTACT_HALF_WIDTH;
    let damage_enemy_half_width = match enemy.kind {
        NativeObjectKind::Enemy { object_type: 0x1C } => MINI_HOPPING_SKULL_CONTACT_HALF_WIDTH,
        _ => ENEMY_CONTACT_HALF_WIDTH,
    };
    let stomp_e_left = ex - (stomp_enemy_half_width << 16);
    let stomp_e_right = ex + (stomp_enemy_half_width << 16);
    let damage_e_left = ex - (damage_enemy_half_width << 16);
    let damage_e_right = ex + (damage_enemy_half_width << 16);
    let stomp_e_top = ey - (ENEMY_CONTACT_HALF_HEIGHT << 16);
    let damage_e_top = ey - (ENEMY_CONTACT_HALF_HEIGHT << 16);
    let damage_e_bottom = ey + (ENEMY_CONTACT_HALF_HEIGHT << 16);

    // Stomp: player must be falling and their feet must still be at or above
    // the enemy's top edge on the previous frame, so this frame crosses or
    // touches the stomp line instead of already interpenetrating from the side.
    let stomp_horizontal_overlap = p_right.min(stomp_e_right) - p_left.max(stomp_e_left);
    let previous_bottom = pre_physics_y_pos as i64;
    if pre_physics_y_velocity > 0
        && previous_bottom <= stomp_e_top
        && stomp_horizontal_overlap >= ENEMY_CONTACT_MIN_OVERLAP_FP
    {
        return EnemyContactResult::Stomp;
    }

    if matches!(enemy.kind, NativeObjectKind::Enemy { object_type: 0x1C })
        && enemy.on_ground == Some(false)
        && enemy.y_velocity.is_some_and(|y_velocity| y_velocity < 0)
    {
        return EnemyContactResult::None;
    }

    if !(p_left < damage_e_right
        && p_right > damage_e_left
        && p_top < damage_e_bottom
        && p_bottom > damage_e_top)
    {
        return EnemyContactResult::None;
    }
    let damage_horizontal_overlap = p_right.min(damage_e_right) - p_left.max(damage_e_left);
    if damage_horizontal_overlap < ENEMY_CONTACT_MIN_OVERLAP_FP {
        return EnemyContactResult::None;
    }

    EnemyContactResult::Damage
}

pub(crate) fn player_touches_enemy(player: PlayerState, enemy: &NativeObjectSnapshot) -> bool {
    // For contact detection (damage source), use current velocity — stomp
    // direction doesn't matter here, only overlap.
    !matches!(
        classify_enemy_contact(player, enemy, player.y_velocity, player.y_pos),
        EnemyContactResult::None
    )
}
