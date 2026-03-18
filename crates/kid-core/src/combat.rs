use super::*;

/// Per-helmet transform animation frame counts from ASM Kid_Transform tables
/// (kid.asm:8678-8812, stru_7EFA..stru_808A). TheKid (helmet 0) has no table.
fn transform_anim_frames(form: PlayerForm) -> u16 {
    match form {
        PlayerForm::TheKid => 0,
        PlayerForm::Skycutter => 47,   // stru_7EFA
        PlayerForm::Cyclone => 72,     // stru_7F30
        PlayerForm::RedStealth => 54,  // stru_7F5A
        PlayerForm::Eyeclops => 93,    // stru_7F90
        PlayerForm::Juggernaut => 33,  // stru_7FCA
        PlayerForm::IronKnight => 96,  // stru_7FF8
        PlayerForm::Berzerker => 51,   // stru_803A
        PlayerForm::Maniaxe => 45,     // stru_8064
        PlayerForm::Micromax => 59,    // stru_808A
    }
}

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
                            homing_accel: if is_heady { 0x7D0 } else { 0 },
                            speed_cap: 0,
                            wall_bounce: false,
                        });
                        self.projectiles.push(EnemyProjectile {
                            x_pos: boss_x,
                            y_pos: boss_y,
                            x_velocity: -0x2_0000,
                            y_velocity: -0x1_0000,
                            lifetime: if is_heady { 120 } else { 90 },
                            gravity: if is_heady { 0x2710 } else { 0x800 },
                            friendly: false,
                            homing_accel: if is_heady { 0x7D0 } else { 0 },
                            speed_cap: 0,
                            wall_bounce: false,
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
            // Fire Demon: spawn fire trail on death
            if matches!(enemy.family, EnemyFamily::FireDemon) {
                spawn_fire_demon_death_trail(&mut self.projectiles, enemy.x_pos, enemy.y_pos);
            }
            self.active_enemies.remove(idx);
            self.add_score(1000);
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
                palette_variant: enemy.palette_variant,
                stationary: false,
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

    /// Add points with score cap (9,999,999) and extra-life-per-50K tracking.
    pub(crate) fn add_score(&mut self, points: u32) {
        let old_score = self.score;
        self.score = self.score.saturating_add(points).min(SCORE_CAP);
        // ASM: award extra life for every 50,000 point boundary crossed.
        let old_lives = old_score / SCORE_EXTRA_LIFE_INTERVAL;
        let new_lives = self.score / SCORE_EXTRA_LIFE_INTERVAL;
        for _ in old_lives..new_lives {
            self.add_life();
        }
    }

    /// Increment diamond count (capped at 99 per ASM). Awards extra life at 50/100.
    pub(crate) fn collect_diamond(&mut self) {
        if self.diamond_count >= 99 {
            return;
        }
        self.diamond_count += 1;
        self.add_score(500);
        if self.diamond_count.is_multiple_of(50) {
            self.add_life();
        }
    }

    pub(crate) fn apply_prize_content(&mut self, content: terrain::PrizeContent) {
        // ASM: NoPrize_Bonus_Flag is set whenever any prize block content is collected.
        self.level_prize_collected = true;
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
                self.clocks_collected += 1;
            }
            terrain::PrizeContent::Continue => {
                self.number_continues = self.number_continues.saturating_add(1);
            }
            terrain::PrizeContent::Ankh => {
                self.add_life();
            }
            terrain::PrizeContent::Points10000 => {
                self.add_score(10_000);
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
        if self.helmet.berzerker_charging {
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

    /// Apply all end-of-level bonuses per ASM (kid.asm:16650-16900).
    pub(crate) fn apply_level_completion_bonuses(&mut self) {
        // Time bonus: remaining seconds × 100
        self.add_score(u32::from(self.level_timer.seconds) * 100);

        // No-hit bonus: 5000 points if no damage taken during level
        if !self.level_damage_taken {
            self.add_score(5000);
        }

        // No-prize bonus: 5000 points if no prize block content collected during level
        // ASM: NoPrize_Bonus_Flag = 0 means bonus retained
        if !self.level_prize_collected {
            self.add_score(5000);
        }

        // Path and speed bonuses only apply to non-Elsewhere levels (< 0x49 in map order).
        let level_idx = self.current_level.0 as usize;
        if self.current_level.0 < FIRST_ELSEWHERE_LEVEL_ID {
            // Path bonus: per-level multiplier × 1000
            if level_idx < PATH_BONUS_TABLE.len() {
                let path_mult = PATH_BONUS_TABLE[level_idx] as u32;
                if path_mult > 0 {
                    self.add_score(path_mult * 1000);
                }
            }

            // Speed bonus: 10,000 points if completion time <= threshold.
            // ASM completion time = 180 - remaining + (clocks_collected * 180), capped at 255.
            if level_idx < SPEED_BONUS_TABLE.len() {
                let threshold = SPEED_BONUS_TABLE[level_idx];
                if threshold > 0 {
                    let elapsed_secs = LEVEL_TIMER_START_SECONDS
                        .saturating_sub(self.level_timer.seconds)
                        .saturating_add(self.clocks_collected * LEVEL_TIMER_START_SECONDS);
                    let completion_time = elapsed_secs.min(255) as u8;
                    if completion_time <= threshold {
                        self.add_score(10_000);
                    }
                }
            }
        }
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
        let old_form = self.current_form;
        self.current_form = form;
        self.number_hitpoints = form.max_hitpoints();
        // Per-helmet transform duration from ASM Kid_Transform tables
        // (kid.asm:8369-8607, stru_7EFA..stru_808A). Two phases:
        // Phase 1 (reverse old helmet) + Phase 2 (forward new helmet).
        // TheKid has no table, so only one phase runs.
        let old_frames = transform_anim_frames(old_form);
        let new_frames = transform_anim_frames(form);
        self.form_change_flash_frames = (old_frames + new_frames) as u8;
        // ASM zeroes velocity at end of transform (kid.asm:8599-8600)
        if let Some(ref mut slice) = self.opening_slice {
            slice.player.x_velocity = 0;
            slice.player.y_velocity = 0;
        }
        // TODO(fidelity): visual sprite art for transform animation.
        // ASM uses per-helmet sprite+palette cycling tables for a spinning
        // animation. Currently renders as a flash overlay. Needs ROM sprite
        // tile extraction for real visuals.
        // Clear ability state on form change
        self.helmet.reset();
    }

    pub(crate) fn apply_damage_with_knockback(
        &mut self,
        _slice: &mut OpeningSliceState,
        _enemy_x: u32,
    ) {
        // ASM does NOT apply knockback velocity on contact damage —
        // the player's velocity is unchanged. The perceived "push" in
        // the original comes from bounding-box separation, not an
        // impulse. Previous native code applied KNOCKBACK_X/Y_VELOCITY
        // which was an invented approximation.
        self.apply_damage();
    }

    pub(crate) fn apply_damage(&mut self) {
        self.level_damage_taken = true;
        // Clear wall-grab so the player isn't stuck to the wall during invulnerability
        if let Some(slice) = self.opening_slice.as_mut() {
            slice.player.right_wall_sticky = false;
            slice.player.left_wall_sticky = false;
        }
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
            0,
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

    /// Reset all state and return to title screen.
    pub(crate) fn return_to_title(&mut self) {
        self.boot_state = BootState::TitleScreen;
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.number_lives = 3;
        self.number_continues = 3;
        self.diamond_count = 0;
        self.score = 0;
        self.current_level = LevelId(0);
        self.opening_slice = None;
        self.two_player = false;
        self.current_player_index = 0;
        self.player_slots = [PlayerSlot::default(), PlayerSlot::default()];
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
    _pre_physics_y_pos: u32,
) -> EnemyContactResult {
    let px = player.x_pos as i64;
    let py = player.y_pos as i64;
    let p_left = px - (PLAYER_CONTACT_HALF_WIDTH << 16);
    let p_right = px + (PLAYER_CONTACT_HALF_WIDTH << 16);
    let p_top = py - (i64::from(PLAYER_HEIGHT) << 16);
    let p_bottom = py;

    let ex = enemy.x_pos as i64;
    let ey = enemy.y_pos as i64;
    let enemy_half_width = match enemy.kind {
        NativeObjectKind::Enemy { object_type: 0x1C } => MINI_HOPPING_SKULL_CONTACT_HALF_WIDTH,
        _ => ENEMY_CONTACT_HALF_WIDTH,
    };
    let e_left = ex - (enemy_half_width << 16);
    let e_right = ex + (enemy_half_width << 16);
    let e_top = ey - (ENEMY_CONTACT_HALF_HEIGHT << 16);
    let e_bottom = ey + (ENEMY_CONTACT_HALF_HEIGHT << 16);

    // Check AABB overlap first
    if !(p_left < e_right && p_right > e_left && p_top < e_bottom && p_bottom > e_top) {
        return EnemyContactResult::None;
    }
    let x_overlap = p_right.min(e_right) - p_left.max(e_left);
    if x_overlap < ENEMY_CONTACT_MIN_OVERLAP_FP {
        return EnemyContactResult::None;
    }

    // MiniHoppingSkull immunity while airborne and rising
    if matches!(enemy.kind, NativeObjectKind::Enemy { object_type: 0x1C })
        && enemy.on_ground == Some(false)
        && enemy.y_velocity.is_some_and(|vy| vy < 0)
    {
        return EnemyContactResult::None;
    }

    // ASM-style direction classification: compare relative velocity with overlap
    // to determine if the player is hitting from above (stomp) or from the side.
    // The ASM uses: y_overlap × |relative_x_vel| vs x_overlap × |relative_y_vel|
    // If vertical approach dominates, and player is falling → stomp.
    if pre_physics_y_velocity > 0 && !player.on_ground {
        let y_overlap = p_bottom.min(e_bottom) - p_top.max(e_top);
        let enemy_x_vel = enemy.x_velocity.unwrap_or(0) as i64;
        let enemy_y_vel = enemy.y_velocity.unwrap_or(0) as i64;
        let rel_x_vel = (enemy_x_vel - i64::from(player.x_velocity)).unsigned_abs();
        let rel_y_vel = (enemy_y_vel - i64::from(pre_physics_y_velocity)).unsigned_abs();

        // "Kid above" = small y_overlap relative to vertical approach speed.
        // When rel_y_vel is 0 (no vertical approach), can't be a stomp.
        if rel_y_vel > 0 {
            let from_above = (y_overlap as u64) * rel_x_vel < (x_overlap as u64) * rel_y_vel;
            if from_above {
                return EnemyContactResult::Stomp;
            }
        }
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
