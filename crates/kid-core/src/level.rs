use super::*;

impl NativeRuntime {
    pub(crate) fn reset_current_level_runtime_state_for_respawn(&mut self) {
        let level_id = self.current_level;
        self.reload_block_grid_for_level(level_id);
        self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(level_id));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.killed_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.shooter_blocks.clear();
        self.block_state.ghost_groups.clear();
        self.block_state.lift_platforms.clear();
        self.projectiles.clear();
        self.pending_prize_diamond_awards.clear();
        self.enemy_death_effects.clear();
        self.block_bounces.clear();
        self.block_destroy_effects.clear();
        self.floating_items.clear();
        self.init_murder_wall(level_id, 0);
        self.init_weather(level_id);
        self.level_timer = LevelTimer::new();
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.y_look_offset = 0;
        self.progression.current_form = PlayerForm::TheKid;
        self.progression.pending_form = None;
        self.progression.form_change_flash_frames = 0;
        self.progression.form_width_delta = 0;
        self.progression.diamond_power_frames = 0;
        self.progression.diamond_power_type = helmets::DiamondPowerType::None;
        self.progression.invulnerable_frames = 0;
        self.helmet.reset();
        self.paused = false;
        self.pause_toggle_pending = false;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
    }

    fn logical_level_ids(&self) -> Option<Vec<LevelId>> {
        if self.assets.map_order.is_empty() {
            return None;
        }

        Some(
            self.assets
                .map_order
                .iter()
                .enumerate()
                .filter_map(|(logical_level_id, &resource_level_id)| {
                    if resource_level_id == u8::MAX {
                        return None;
                    }
                    u8::try_from(logical_level_id).ok().map(LevelId)
                })
                .collect(),
        )
    }

    pub(crate) fn is_final_level(&self) -> bool {
        if let Some(logical_levels) = self.logical_level_ids() {
            return logical_levels.last().copied() == Some(self.current_level);
        }
        false
    }

    pub(crate) fn advance_level(&mut self) {
        self.paused = false;
        self.pause_toggle_pending = false;
        self.pending_level_skip = None;
        self.pending_level_skip_resume = false;

        if self.is_final_level() {
            self.game_complete = true;
            self.boot_state = BootState::EndingSequence { elapsed: 0 };
            self.player_lifecycle = PlayerLifecycle::Alive;
            self.opening_slice = None;
            return;
        }

        let next_level = self.next_level_id();
        self.current_level = next_level;
        self.reload_block_grid_for_level(next_level);
        self.level_start_frame = Some(self.frame_index);
        // Use floor-snap so the player spawns on solid ground.
        // The header spawn Y is approximate; ASM relies on gravity + sub_902A
        // to settle the player on the first frame. Without floor-snap, the
        // player can float above terrain and then drop into the void.
        self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(next_level));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.killed_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.shooter_blocks.clear();
        self.block_state.ghost_groups.clear();
        self.projectiles.clear();
        self.pending_prize_diamond_awards.clear();
        self.block_state.lift_platforms.clear();
        self.init_murder_wall(next_level, 0);
        self.init_weather(next_level);
        self.level_timer = LevelTimer::new();
        self.level_damage_taken = false;
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        self.progression.invulnerable_frames = 0;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
        self.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_LEVEL_ADVANCE_RESPAWN_SETTLE_FRAMES,
        };
    }

    /// Previous logical level ID (for debug warping).
    pub fn prev_level_id(&self) -> LevelId {
        if let Some(logical_levels) = self.logical_level_ids() {
            let current_pos = logical_levels
                .iter()
                .position(|&id| id == self.current_level);
            if let Some(pos) = current_pos
                && pos > 0
            {
                return logical_levels[pos - 1];
            }
        }
        self.current_level
    }

    /// Warp to a specific level (debug feature).
    pub fn warp_to_level(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.reload_block_grid_for_level(level_id);
        self.level_start_frame = Some(self.frame_index);
        self.opening_slice = Some(self.level_respawn_slice(level_id));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.shooter_blocks.clear();
        self.block_state.ghost_groups.clear();
        self.projectiles.clear();
        self.pending_prize_diamond_awards.clear();
        self.block_state.lift_platforms.clear();
        self.init_murder_wall(level_id, 0);
        self.init_weather(level_id);
        self.level_timer = LevelTimer::new();
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        self.progression.invulnerable_frames = 0;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
        self.paused = false;
        self.boot_state = BootState::InGame;
        self.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_LEVEL_ADVANCE_RESPAWN_SETTLE_FRAMES,
        };
    }

    pub(crate) fn begin_level_entry(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.level_start_frame = None;
        self.boot_state = BootState::StartingLevel {
            input_armed: false,
            elapsed_frames: 0,
        };
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
        self.pending_level_skip = None;
        self.pending_level_skip_resume = false;
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        let bounds = self.level_bounds(level_id);
        self.opening_slice = Some(OpeningSliceState::new(bounds));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.shooter_blocks.clear();
        self.block_state.ghost_groups.clear();
        self.pending_prize_diamond_awards.clear();
        self.init_murder_wall(level_id, 0);
        self.init_weather(level_id);
        self.level_timer = LevelTimer::new();
        self.level_damage_taken = false;
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.y_look_offset = 0;
        self.paused = false;
        self.pause_toggle_pending = false;
        self.reload_block_grid_for_level(level_id);
        self.rebuild_starting_level_title_workers();
    }

    /// Save working state to the current player's slot (ASM sub_8C2).
    pub(crate) fn save_player_slot(&mut self) {
        let slot = &mut self.player_slots[self.current_player_index as usize];
        slot.lives = self.progression.number_lives;
        slot.hitpoints = self.progression.number_hitpoints;
        slot.diamonds = self.progression.diamond_count;
        slot.level_id = self.current_level;
        slot.form = self.progression.current_form;
        slot.continues = self.progression.number_continues;
        slot.score = self.progression.score;
    }

    /// Load a player slot into working state (ASM sub_8A4).
    pub(crate) fn load_player_slot(&mut self) {
        let slot = &self.player_slots[self.current_player_index as usize];
        self.progression.number_lives = slot.lives;
        self.progression.number_hitpoints = slot.hitpoints;
        self.progression.diamond_count = slot.diamonds;
        self.current_level = slot.level_id;
        self.progression.current_form = slot.form;
        self.progression.pending_form = None;
        self.progression.number_continues = slot.continues;
        self.progression.score = slot.score;
    }

    /// In 2P mode, switch to the other player if they have lives remaining.
    /// Returns true if a switch occurred.
    pub(crate) fn try_switch_player(&mut self) -> bool {
        if !self.two_player {
            return false;
        }
        self.save_player_slot();
        let other = 1 - self.current_player_index;
        if self.player_slots[other as usize].lives > 0 {
            self.current_player_index = other;
            self.load_player_slot();
            self.begin_level_entry(self.current_level);
            true
        } else {
            false
        }
    }

    pub(crate) fn enter_level_skip_hold(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.level_start_frame = Some(self.frame_index);
        self.boot_state = BootState::LevelSkipHold;
        self.pending_level_skip = None;
        self.pending_level_skip_resume = false;
        self.opening_slice = Some(self.level_respawn_slice(level_id));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.shooter_blocks.clear();
        self.block_state.ghost_groups.clear();
        self.pending_prize_diamond_awards.clear();
        self.init_murder_wall(level_id, 0);
        self.level_timer = LevelTimer::new();
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        self.paused = false;
        self.pause_toggle_pending = false;
        self.death_shell_snapshot = None;
        self.reload_block_grid_for_level(level_id);
    }

    /// Advance the murder wall position by applying acceleration and speed.
    pub(crate) fn reload_block_grid_for_level(&mut self, level_id: LevelId) {
        if let Some(grid) = self.load_block_grid_for_level(level_id) {
            let width = grid.width_cells;
            self.block_grid = Some(grid);
            self.block_list = self.load_block_list_for_level(level_id, width);
            self.block_state.ghost_groups = self
                .block_list
                .as_ref()
                .map(|bl| {
                    bl.ghost_blocks
                        .iter()
                        .map(|g| GhostGroup {
                            col: g.col,
                            row: g.row,
                            count: g.count,
                            vertical: g.vertical,
                            solid_time: g.solid_time,
                            vanished_time: g.vanished_time,
                            timer: g.timer,
                            solid: g.solid,
                        })
                        .collect()
                })
                .unwrap_or_default();
            self.block_grid_source = BlockGridSource::Generated;
        } else if self.block_grid_source == BlockGridSource::Generated {
            self.block_grid = None;
            self.block_list = None;
            self.block_state.ghost_groups.clear();
            self.block_grid_source = BlockGridSource::None;
        }
    }

    pub(crate) fn load_block_grid_for_level(
        &self,
        level_id: LevelId,
    ) -> Option<terrain::BlockGrid> {
        let layout_dir = self.assets.output_root.join("level-layout");
        let layout_path = layout_dir.join(format!("{:02X}.bin", level_id.0));
        if !layout_path.is_file() {
            return None;
        }
        let mut data = fs::read(layout_path).ok()?;
        let metadata_path = layout_dir.join(format!("{:02X}.json", level_id.0));
        let metadata = fs::read_to_string(metadata_path)
            .ok()
            .and_then(|json| serde_json::from_str::<LevelLayoutMetadata>(&json).ok())?;
        data.truncate(metadata.data_bytes.min(data.len()));
        Some(terrain::BlockGrid::from_level_layout(
            &data,
            metadata.width_blocks,
            metadata.height_blocks,
        ))
    }

    pub(crate) fn load_block_list_for_level(
        &self,
        level_id: LevelId,
        layout_width: u16,
    ) -> Option<terrain::BlockList> {
        let layout_dir = self.assets.output_root.join("level-layout");
        let path = layout_dir.join(format!("{:02X}.blocklist.bin", level_id.0));
        let data = fs::read(path).ok()?;
        Some(terrain::BlockList::from_raw(&data, layout_width))
    }

    pub(crate) fn level_respawn_slice(&self, level_id: LevelId) -> OpeningSliceState {
        let bounds = self.level_bounds(level_id);
        let pose = self.level_start_pose(level_id, bounds);
        OpeningSliceState::respawn_from_pose(bounds, pose)
    }

    /// Like `level_respawn_slice` but snaps the player to the actual floor
    /// in the block grid. Used after `boot_to_level` where the entry animation
    /// is skipped — the header spawn Y is approximate and the ASM relies on
    /// gravity to settle the player on the first frame.
    pub(crate) fn level_respawn_slice_with_floor_snap(
        &self,
        level_id: LevelId,
    ) -> OpeningSliceState {
        self.level_respawn_slice_with_floor_snap_inner(level_id, None)
    }

    pub fn inspect_respawn_settle(
        &self,
        level_id: LevelId,
        max_steps: usize,
    ) -> Vec<RespawnSettleFrame> {
        let mut frames = Vec::with_capacity(max_steps.saturating_add(1));
        let _ = self
            .level_respawn_slice_with_floor_snap_inner(level_id, Some((&mut frames, max_steps)));
        frames
    }

    pub fn inspect_spawn_probes(&self, level_id: LevelId) -> Option<SpawnProbeInspection> {
        let grid = self.block_grid.as_ref()?;
        let bounds = self.level_bounds(level_id);
        let pose = self.level_start_pose(level_id, bounds);
        let x_px = (pose.kid_x_pos >> 16) as i32;
        let y_px = (pose.kid_y_pos >> 16) as i32;

        Some(SpawnProbeInspection {
            kid_x_pos: pose.kid_x_pos,
            kid_y_pos: pose.kid_y_pos,
            hitbox_probe: terrain::probe_floor_hitbox(grid, x_px, y_px, PLAYER_HALF_WIDTH),
            span_probe: terrain::probe_floor_span(grid, x_px - 16, x_px + 16, y_px),
        })
    }

    fn level_respawn_slice_with_floor_snap_inner(
        &self,
        level_id: LevelId,
        mut settle_trace: Option<(&mut Vec<RespawnSettleFrame>, usize)>,
    ) -> OpeningSliceState {
        let bounds = self.level_bounds(level_id);
        let pose = self.level_start_pose(level_id, bounds);
        let mut slice = OpeningSliceState::respawn_from_pose(bounds, pose);
        slice.player.on_ground = false;
        slice.terrain.floor_y_pos = slice.player.y_pos;

        let Some(base_grid) = self.block_grid.as_ref() else {
            return slice;
        };
        let mut grid = base_grid.clone();
        Self::resolve_spawn_wall_overlap(&mut slice, &grid);

        if let Some((trace, _)) = settle_trace.as_mut() {
            trace.push(RespawnSettleFrame {
                step_index: 0,
                kid_x_pos: slice.player.x_pos,
                kid_y_pos: slice.player.y_pos,
                kid_x_velocity: slice.player.x_velocity,
                kid_y_velocity: slice.player.y_velocity,
                kid_on_ground: slice.player.on_ground,
                kid_on_spikes: slice.player.on_spikes,
                camera_x_pos: slice.camera.x_pos,
                camera_y_pos: slice.camera.y_pos,
                floor_y_pos: slice.terrain.floor_y_pos,
                wide_support_applied: false,
            });
        }
        let settle_input = FrameInput::default();
        let mut stable_frames = 0usize;

        let max_steps = settle_trace
            .as_ref()
            .map(|(_, max_steps)| *max_steps)
            .unwrap_or(180)
            .min(180);
        for frame in 0..max_steps {
            let prev_player = (slice.player.x_pos, slice.player.y_pos);
            let prev_camera = (slice.camera.x_pos, slice.camera.y_pos);
            slice.step(
                settle_input,
                false,
                self.progression.current_form,
                Some(&mut grid),
                self.block_list.as_ref(),
                false,
                0,
            );
            let wide_support_applied = self.try_settle_on_spawn_support(&mut slice, &grid);
            if wide_support_applied {
                slice.update_camera_follow(0);
            }
            if let Some((trace, _)) = settle_trace.as_mut() {
                trace.push(RespawnSettleFrame {
                    step_index: frame + 1,
                    kid_x_pos: slice.player.x_pos,
                    kid_y_pos: slice.player.y_pos,
                    kid_x_velocity: slice.player.x_velocity,
                    kid_y_velocity: slice.player.y_velocity,
                    kid_on_ground: slice.player.on_ground,
                    kid_on_spikes: slice.player.on_spikes,
                    camera_x_pos: slice.camera.x_pos,
                    camera_y_pos: slice.camera.y_pos,
                    floor_y_pos: slice.terrain.floor_y_pos,
                    wide_support_applied,
                });
            }

            let current_player = (slice.player.x_pos, slice.player.y_pos);
            let current_camera = (slice.camera.x_pos, slice.camera.y_pos);
            if slice.player.on_ground
                && current_player == prev_player
                && current_camera == prev_camera
                && slice.player.x_velocity == 0
                && slice.player.y_velocity == 0
            {
                stable_frames += 1;
                if stable_frames >= 8 {
                    break;
                }
            } else {
                stable_frames = 0;
            }
        }

        Self::finish_respawn_camera_convergence(&mut slice);
        if let Some((trace, _)) = settle_trace.as_mut()
            && let Some(last) = trace.last_mut()
        {
            last.kid_x_pos = slice.player.x_pos;
            last.kid_y_pos = slice.player.y_pos;
            last.kid_x_velocity = slice.player.x_velocity;
            last.kid_y_velocity = slice.player.y_velocity;
            last.kid_on_ground = slice.player.on_ground;
            last.kid_on_spikes = slice.player.on_spikes;
            last.camera_x_pos = slice.camera.x_pos;
            last.camera_y_pos = slice.camera.y_pos;
            last.floor_y_pos = slice.terrain.floor_y_pos;
        }
        slice
    }

    fn finish_respawn_camera_convergence(slice: &mut OpeningSliceState) {
        let player_y_px = (slice.player.y_pos >> 16) as i32;
        let camera_y_px = (slice.camera.y_pos >> 16) as i32;
        let max_y = i32::from(slice.level_bounds.height_pixels) - physics::VIEWPORT_HEIGHT;
        let target_y_px = (player_y_px - physics::CAMERA_PLAYER_OFFSET)
            .max(0)
            .min(max_y);
        if !slice.player.on_ground || target_y_px - camera_y_px != 2 {
            return;
        }

        // ASM Load_InGame seeds the raw header pose, then the hidden init
        // frames keep running `sub_71E4` while Kid settles onto the floor.
        // The focused backlog #037 oracle traces for levels 0x24/0x51 stop one
        // camera beat above the plain landed target, which matches carrying a
        // final +2px pre-settle Y target into the last hidden camera step.
        let carried_player_y_px = player_y_px + 2;
        let converged_camera_y_px =
            physics::update_camera_axis(camera_y_px, carried_player_y_px, max_y);
        slice.camera.y_pos = (converged_camera_y_px.max(0) as u32) << 16;
    }

    pub(crate) fn resolve_spawn_wall_overlap(
        slice: &mut OpeningSliceState,
        grid: &terrain::BlockGrid,
    ) {
        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let left_probe =
            terrain::probe_wall(grid, x_px, y_px, PLAYER_HALF_WIDTH, PLAYER_HEIGHT, false);
        let right_probe =
            terrain::probe_wall(grid, x_px, y_px, PLAYER_HALF_WIDTH, PLAYER_HEIGHT, true);
        let mut resolved_x_px = x_px;

        if left_probe.hit {
            let min_clear_x = (i32::from(left_probe.cell_x) + 1) * 16 + PLAYER_HALF_WIDTH;
            resolved_x_px = resolved_x_px.max(min_clear_x);
        }
        if right_probe.hit {
            let max_clear_x = i32::from(right_probe.cell_x) * 16 - PLAYER_HALF_WIDTH - 1;
            resolved_x_px = resolved_x_px.min(max_clear_x);
        }

        if resolved_x_px != x_px {
            slice.player.x_pos = (resolved_x_px.max(0) as u32) << 16;
        }
    }

    pub(crate) fn try_settle_on_spawn_support(
        &self,
        slice: &mut OpeningSliceState,
        grid: &terrain::BlockGrid,
    ) -> bool {
        const INIT_SETTLE_HALF_WIDTH: i32 = 16;

        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let hitbox_probe = terrain::probe_floor_hitbox(grid, x_px, y_px, PLAYER_HALF_WIDTH);
        let hitbox_floor_y_px = match hitbox_probe.cell.collision {
            terrain::CollisionKind::Solid
            | terrain::CollisionKind::Spikes
            | terrain::CollisionKind::HiddenBlock => hitbox_probe.floor_y_px.saturating_sub(1),
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::Empty => hitbox_probe.floor_y_px,
        };
        let hitbox_floor_y_fp = (hitbox_floor_y_px.max(0) as u32) << 16;
        let hitbox_has_safe_support = hitbox_probe.cell.is_solid_floor()
            && hitbox_probe.cell.collision != terrain::CollisionKind::Spikes;
        if hitbox_has_safe_support
            && (hitbox_probe.on_ground
                || slice.player.y_pos.abs_diff(hitbox_floor_y_fp) <= GROUND_SNAP_TOLERANCE_FP)
        {
            return false;
        }

        let probe = terrain::probe_floor_span(
            grid,
            x_px - INIT_SETTLE_HALF_WIDTH,
            x_px + INIT_SETTLE_HALF_WIDTH,
            y_px,
        );
        if !probe.cell.is_solid_floor() {
            return false;
        }
        if slice.player.on_ground && !slice.player.on_spikes {
            return false;
        }
        let fallback_floor = terrain::find_floor_below(grid, x_px, y_px).map(|floor_y_px| {
            let floor_probe = terrain::probe_floor(grid, x_px, floor_y_px);
            (floor_y_px, floor_probe.cell.collision)
        });
        if let Some((floor_y_px, collision)) = fallback_floor
            && floor_y_px > probe.floor_y_px
            && collision != terrain::CollisionKind::Spikes
            && !Self::spawn_support_forms_stair_step(grid, x_px, probe)
        {
            return false;
        }

        let floor_y_px = match probe.cell.collision {
            terrain::CollisionKind::Solid
            | terrain::CollisionKind::Spikes
            | terrain::CollisionKind::HiddenBlock => probe.floor_y_px.saturating_sub(1),
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::Empty => probe.floor_y_px,
        };
        let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
        let within_snap = slice.player.y_pos.abs_diff(floor_y_fp) <= GROUND_SNAP_TOLERANCE_FP;
        if slice.player.y_velocity < 0 || (slice.player.y_pos < floor_y_fp && !within_snap) {
            return false;
        }

        // Level settling is always a new landing (on_ground check at line 477
        // returns early if already grounded), so clear sub-pixel per sub_902A.
        slice.player.y_pos = floor_y_fp & 0xFFFF_0000;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
        slice.terrain.floor_y_pos = floor_y_fp;
        slice.terrain.floor_terrain_type = TerrainType::from_block_type(probe.cell.block_type);
        true
    }

    fn spawn_support_forms_stair_step(
        grid: &terrain::BlockGrid,
        center_x_px: i32,
        probe: terrain::FloorProbe,
    ) -> bool {
        let center_col = terrain::pixel_to_cell(center_x_px).max(0) as u16;
        if probe.cell_y == 0 || probe.cell_x == center_col {
            return false;
        }

        // ASM sub_914A scans the entire landing row under Kid's width. Oracle-
        // backed level 0x52 traces show that this catches a one-row-higher
        // stair-step continuation before the deeper centre-column floor wins.
        let outward_col = if probe.cell_x > center_col {
            probe.cell_x.saturating_add(1)
        } else {
            probe.cell_x.saturating_sub(1)
        };
        grid.cell_at(outward_col, probe.cell_y.saturating_sub(1))
            .is_solid_floor()
    }

    pub(crate) fn level_start_pose(
        &self,
        level_id: LevelId,
        _bounds: LevelBounds,
    ) -> LevelStartPose {
        let Some(header) = self.level_header(level_id) else {
            return LevelStartPose {
                kid_x_pos: 0x0020_0000,
                kid_y_pos: OPENING_GROUNDED_KID_Y_POS,
                camera_x_pos: 0,
                camera_y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            };
        };
        let kid_x_pos = u32::from(header.player_start_x) << 16;
        // The ASM applies `subq.w #1,y_pos` when placing the player sprite,
        // putting the player 1 pixel above the header value (into the floor
        // cell above, not the empty cell below).
        let kid_y_pos = u32::from(header.player_start_y.saturating_sub(1)) << 16;
        // Camera snaps to screen boundary: floor(pos / screen_size) * screen_size
        // This matches the ASM at Load_InGame (divu.w #$140 / mulu.w #$140).
        let camera_x_pos = u32::from((header.player_start_x / 320) * 320) << 16;
        let camera_y_pos = u32::from((header.player_start_y / 224) * 224) << 16;
        LevelStartPose {
            kid_x_pos,
            kid_y_pos,
            camera_x_pos,
            camera_y_pos,
        }
    }

    pub(crate) fn level_skip_hold_pose(&self, level_id: LevelId) -> LevelSkipHoldPose {
        match level_id {
            LevelId(1) => LevelSkipHoldPose {
                kid_x_pos: 0x0040_0000,
                kid_y_pos: 0x0068_0000,
                camera_x_pos: 0,
                camera_y_pos: 0,
                number_objects: 16,
                number_gfx_objects: 15,
            },
            LevelId(2) => LevelSkipHoldPose {
                kid_x_pos: 0x0030_0000,
                kid_y_pos: 0x0040_0000,
                camera_x_pos: 0,
                camera_y_pos: 0,
                number_objects: 15,
                number_gfx_objects: 14,
            },
            _ => self
                .level_header(level_id)
                .map(|header| LevelSkipHoldPose {
                    kid_x_pos: u32::from(header.player_start_x) << 16,
                    kid_y_pos: u32::from(header.player_start_y) << 16,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    number_objects: 4,
                    number_gfx_objects: 4,
                })
                .unwrap_or(LevelSkipHoldPose {
                    kid_x_pos: 0x0020_0000,
                    kid_y_pos: OPENING_GROUNDED_KID_Y_POS,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    number_objects: 4,
                    number_gfx_objects: 4,
                }),
        }
    }

    pub(crate) fn level_skip_animation_path(&self, level_id: LevelId) -> LevelSkipAnimationPath {
        match level_id {
            LevelId(1) => LevelSkipAnimationPath {
                initial_x_pos: 0x0160_0000,
                initial_y_pos: 0x0014_0000,
                x_step_fp: -(LEVEL_SKIP_ANIMATION_X_STEP_FP as i32),
                y_step_fp: LEVEL_SKIP_ANIMATION_Y_STEP_FP as i32,
                first_animation_number_objects: 16,
                first_animation_number_gfx_objects: 13,
            },
            LevelId(2) => LevelSkipAnimationPath {
                initial_x_pos: -0x0020_0000,
                initial_y_pos: 0x00C8_0000,
                x_step_fp: 0x0001_4000,
                y_step_fp: -0x0002_2000,
                first_animation_number_objects: 15,
                first_animation_number_gfx_objects: 13,
            },
            _ => {
                let hold_pose = self.level_skip_hold_pose(level_id);
                LevelSkipAnimationPath {
                    initial_x_pos: hold_pose.kid_x_pos.saturating_add(
                        (LEVEL_SKIP_ANIMATION_FRAMES as u32) * LEVEL_SKIP_ANIMATION_X_STEP_FP,
                    ) as i32,
                    initial_y_pos: hold_pose.kid_y_pos.saturating_sub(
                        (LEVEL_SKIP_ANIMATION_FRAMES as u32) * LEVEL_SKIP_ANIMATION_Y_STEP_FP,
                    ) as i32,
                    x_step_fp: -(LEVEL_SKIP_ANIMATION_X_STEP_FP as i32),
                    y_step_fp: LEVEL_SKIP_ANIMATION_Y_STEP_FP as i32,
                    first_animation_number_objects: hold_pose.number_objects + 1,
                    first_animation_number_gfx_objects: hold_pose.number_gfx_objects - 1,
                }
            }
        }
    }

    pub(crate) fn level_skip_player_state(&self) -> Option<PlayerState> {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let animation_path = self.level_skip_animation_path(self.current_level);
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return Some(PlayerState {
                x_pos: hold_pose.kid_x_pos,
                y_pos: hold_pose.kid_y_pos,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        };

        let elapsed = transition.elapsed_frames;
        if elapsed < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES {
            return Some(transition.frozen_player);
        }

        let animation_start = LEVEL_SKIP_FREEZE_VISIBLE_FRAMES
            + LEVEL_SKIP_HIDE_PLAYER_FRAMES
            + LEVEL_SKIP_BLANK_FRAMES
            + LEVEL_SKIP_SMALL_PRELOAD_FRAMES
            + LEVEL_SKIP_LARGE_PRELOAD_FRAMES;
        if elapsed < animation_start {
            return None;
        }
        let animation_frame = elapsed.saturating_sub(animation_start);
        if animation_frame >= LEVEL_SKIP_ANIMATION_FRAMES {
            return Some(PlayerState {
                x_pos: hold_pose.kid_x_pos,
                y_pos: hold_pose.kid_y_pos,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }

        let animation_frame_i64 = i64::try_from(animation_frame).unwrap_or(i64::MAX);
        let x_pos = i64::from(animation_path.initial_x_pos)
            + i64::from(animation_path.x_step_fp) * animation_frame_i64;
        let y_pos = i64::from(animation_path.initial_y_pos)
            + i64::from(animation_path.y_step_fp) * animation_frame_i64;
        Some(PlayerState {
            x_pos: (x_pos as i32) as u32,
            y_pos: (y_pos as i32) as u32,
            x_velocity: animation_path.x_step_fp,
            y_velocity: animation_path.y_step_fp,
            facing: Direction::Right,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        })
    }

    pub(crate) fn level_skip_camera_state(&self) -> CameraState {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return CameraState {
                x_pos: hold_pose.camera_x_pos,
                y_pos: hold_pose.camera_y_pos,
            };
        };
        if transition.elapsed_frames
            < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES
        {
            return transition.frozen_camera;
        }
        CameraState {
            x_pos: hold_pose.camera_x_pos,
            y_pos: hold_pose.camera_y_pos,
        }
    }

    pub(crate) fn level_skip_active_objects(&self) -> Vec<NativeObjectSnapshot> {
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return Vec::new();
        };
        if transition.elapsed_frames
            < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES
        {
            return transition.frozen_active_objects.clone();
        }
        Vec::new()
    }

    pub(crate) fn level_skip_object_counts(&self) -> (u16, u16) {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let animation_path = self.level_skip_animation_path(self.current_level);
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return (hold_pose.number_objects, hold_pose.number_gfx_objects);
        };
        let elapsed = transition.elapsed_frames;
        if elapsed < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES {
            return (
                transition.frozen_number_objects,
                transition.frozen_number_gfx_objects,
            );
        }
        let blank_start = LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES;
        let small_preload_start = blank_start + LEVEL_SKIP_BLANK_FRAMES;
        let large_preload_start = small_preload_start + LEVEL_SKIP_SMALL_PRELOAD_FRAMES;
        let animation_start = large_preload_start + LEVEL_SKIP_LARGE_PRELOAD_FRAMES;
        if elapsed < small_preload_start {
            (0, 0)
        } else if elapsed < large_preload_start {
            (2, 0)
        } else if elapsed < animation_start {
            (15, 0)
        } else if elapsed == animation_start {
            (
                animation_path.first_animation_number_objects,
                animation_path.first_animation_number_gfx_objects,
            )
        } else {
            (hold_pose.number_objects, hold_pose.number_gfx_objects)
        }
    }

    pub(crate) fn level_skip_resume_elapsed_frames(&self) -> Option<usize> {
        if !self.level_skip_resume_hold {
            return None;
        }
        self.level_start_frame
            .map(|frame| self.frame_index.saturating_sub(frame))
    }

    pub(crate) fn level_skip_resume_player_state(&self) -> Option<PlayerState> {
        self.level_skip_resume_player_state_at(self.level_skip_resume_elapsed_frames().unwrap_or(0))
    }

    pub(crate) fn level_skip_resume_player_state_at(&self, elapsed: usize) -> Option<PlayerState> {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        if elapsed < LEVEL_SKIP_RESUME_VISIBLE_FRAMES {
            return Some(PlayerState {
                x_pos: hold_pose.kid_x_pos,
                y_pos: hold_pose.kid_y_pos,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }
        let resume_entry_elapsed = elapsed.saturating_sub(TITLE_TO_LEVEL_FRAMES);
        if elapsed >= TITLE_TO_LEVEL_FRAMES
            && self.current_level == LevelId(1)
            && resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES
        {
            return Some(PlayerState {
                x_pos: 0x0020_0000,
                y_pos: LEVEL_ONE_SKIP_RESUME_ENTRY_PLAYER_Y_FP[resume_entry_elapsed],
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: resume_entry_elapsed >= LEVEL_SKIP_RESUME_ENTRY_FRAMES - 3,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }
        None
    }

    pub(crate) fn level_skip_resume_camera_state(&self) -> CameraState {
        self.level_skip_resume_camera_state_at(self.level_skip_resume_elapsed_frames().unwrap_or(0))
    }

    pub(crate) fn level_skip_resume_camera_state_at(&self, elapsed: usize) -> CameraState {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let resume_entry_elapsed = elapsed.saturating_sub(TITLE_TO_LEVEL_FRAMES);
        if elapsed >= TITLE_TO_LEVEL_FRAMES
            && self.current_level == LevelId(1)
            && resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES
        {
            return CameraState {
                x_pos: 0,
                y_pos: LEVEL_ONE_SKIP_RESUME_ENTRY_CAMERA_Y_FP[resume_entry_elapsed],
            };
        }
        if self.current_level == LevelId(1) && elapsed == 52 {
            return CameraState {
                x_pos: 0,
                y_pos: 0xFFA0_0000,
            };
        }
        CameraState {
            x_pos: hold_pose.camera_x_pos,
            y_pos: hold_pose.camera_y_pos,
        }
    }

    pub(crate) fn level_skip_resume_active_objects(&self) -> Vec<NativeObjectSnapshot> {
        self.level_skip_resume_active_objects_at(
            self.level_skip_resume_elapsed_frames().unwrap_or(0),
        )
    }

    pub(crate) fn level_skip_resume_active_objects_at(
        &self,
        elapsed: usize,
    ) -> Vec<NativeObjectSnapshot> {
        let Some(player_state) = self.level_skip_resume_player_state_at(elapsed) else {
            return Vec::new();
        };
        vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Player,
            x_pos: player_state.x_pos,
            y_pos: player_state.y_pos,
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
        }]
    }

    pub(crate) fn level_skip_resume_object_counts(&self) -> (u16, u16) {
        self.level_skip_resume_object_counts_at(
            self.level_skip_resume_elapsed_frames().unwrap_or(0),
        )
    }

    pub(crate) fn level_skip_resume_object_counts_at(&self, elapsed: usize) -> (u16, u16) {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        if elapsed < LEVEL_SKIP_RESUME_VISIBLE_FRAMES + LEVEL_SKIP_RESUME_HIDE_PLAYER_FRAMES {
            return (hold_pose.number_objects, hold_pose.number_gfx_objects);
        }
        if elapsed
            < LEVEL_SKIP_RESUME_VISIBLE_FRAMES
                + LEVEL_SKIP_RESUME_HIDE_PLAYER_FRAMES
                + LEVEL_SKIP_RESUME_BLANK_FRAMES
        {
            return (0, 0);
        }
        if elapsed == TITLE_TO_LEVEL_FRAMES - 1 {
            return (4, 2);
        }
        let resume_entry_elapsed = elapsed.saturating_sub(TITLE_TO_LEVEL_FRAMES);
        if elapsed >= TITLE_TO_LEVEL_FRAMES
            && self.current_level == LevelId(1)
            && resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES
        {
            if resume_entry_elapsed == 0 {
                return (5, 4);
            }
            if resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES - 3 {
                return (5, 6);
            }
            return (4, 6);
        }
        (hold_pose.number_objects, hold_pose.number_gfx_objects)
    }

    pub(crate) fn level_skip_resume_final_slice(&self, level_id: LevelId) -> OpeningSliceState {
        if level_id == LevelId(1) {
            let bounds = self.level_bounds(level_id);
            return OpeningSliceState::respawn_from_pose(
                bounds,
                LevelStartPose {
                    kid_x_pos: 0x0020_0000,
                    kid_y_pos: 0x00BF_0000,
                    camera_x_pos: 0,
                    camera_y_pos: 0x0014_0000,
                },
            );
        }
        self.level_respawn_slice_with_floor_snap(level_id)
    }

    pub fn next_level_id(&self) -> LevelId {
        if let Some(logical_levels) = self.logical_level_ids() {
            let current_pos = logical_levels
                .iter()
                .position(|&id| id == self.current_level);
            if let Some(pos) = current_pos {
                let next_pos = pos + 1;
                if next_pos < logical_levels.len() {
                    return logical_levels[next_pos];
                }
            }
        }
        // Fallback: next numerically higher level ID
        let mut level_ids = self
            .assets
            .levels
            .iter()
            .map(|level| level.level_id)
            .collect::<Vec<_>>();
        level_ids.sort_unstable();
        level_ids
            .into_iter()
            .find(|level_id| *level_id > self.current_level.0)
            .map(LevelId)
            .or_else(|| {
                self.assets
                    .levels
                    .first()
                    .map(|level| LevelId(level.level_id))
            })
            .unwrap_or(self.current_level)
    }

    pub fn level_name(&self, level_id: LevelId) -> Option<&str> {
        self.level_record(level_id)
            .and_then(|level| level.display_name.as_deref())
    }

    pub(crate) fn level_header(&self, level_id: LevelId) -> Option<&kid_data::LevelHeader> {
        // The game resolves headers via MapOrder_Index[level_id] → resource level.
        // Prefer the resource level's header to match the ASM lookup path.
        self.resource_level_record(level_id)
            .and_then(|level| level.header_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.header_metadata.as_ref())
            })
    }

    pub(crate) fn enemy_layout(&self, level_id: LevelId) -> Option<&kid_data::EnemyLayout> {
        self.resource_level_record(level_id)
            .and_then(|level| level.enemy_layout_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.enemy_layout_metadata.as_ref())
            })
    }

    pub(crate) fn platform_layout(&self, level_id: LevelId) -> Option<&kid_data::PlatformLayout> {
        self.resource_level_record(level_id)
            .and_then(|level| level.platform_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.platform_metadata.as_ref())
            })
    }

    pub(crate) fn level_record(&self, level_id: LevelId) -> Option<&kid_data::LevelRecord> {
        self.assets
            .levels
            .iter()
            .find(|level| level.level_id == level_id.0)
    }

    pub(crate) fn resource_level_record(
        &self,
        level_id: LevelId,
    ) -> Option<&kid_data::LevelRecord> {
        resource_level_record(&self.assets, level_id.0)
    }

    pub(crate) fn level_bounds(&self, level_id: LevelId) -> LevelBounds {
        self.level_header(level_id)
            .map(|header| LevelBounds {
                width_pixels: u16::from(header.width_screens) * 320,
                height_pixels: u16::from(header.height_screens) * 224,
            })
            .unwrap_or(LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            })
    }
}
