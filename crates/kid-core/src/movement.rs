use super::*;

impl PlayerState {
    pub(crate) fn intro_sequence_3_pose() -> Self {
        Self {
            x_pos: INTRO_SEQUENCE_3_KID_X_POS,
            y_pos: INTRO_SEQUENCE_3_KID_Y_POS,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Left,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        }
    }

    pub(crate) fn title_pose() -> Self {
        Self {
            x_pos: TITLE_KID_X_POS,
            y_pos: TITLE_KID_Y_POS,
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
        }
    }
}

impl OpeningSliceState {
    pub(crate) fn new(bounds: LevelBounds) -> Self {
        Self {
            player: PlayerState {
                x_pos: OPENING_ENTRY_KID_X_POS,
                y_pos: OPENING_ENTRY_KID_Y_POS,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: false,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            },
            camera: CameraState {
                x_pos: 0,
                y_pos: OPENING_ENTRY_CAMERA_Y_FP,
            },
            terrain: TerrainState {
                floor_y_pos: OPENING_GROUNDED_KID_Y_POS,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: bounds,
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn respawn(bounds: LevelBounds) -> Self {
        Self::respawn_from_pose(
            bounds,
            LevelStartPose {
                kid_x_pos: OPENING_ENTRY_KID_X_POS,
                kid_y_pos: OPENING_GROUNDED_KID_Y_POS,
                camera_x_pos: 0,
                camera_y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            },
        )
    }

    pub(crate) fn respawn_from_pose(bounds: LevelBounds, pose: LevelStartPose) -> Self {
        Self {
            player: PlayerState {
                x_pos: pose.kid_x_pos,
                y_pos: pose.kid_y_pos,
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
            },
            camera: CameraState {
                x_pos: pose.camera_x_pos,
                y_pos: pose.camera_y_pos,
            },
            terrain: TerrainState {
                floor_y_pos: pose.kid_y_pos,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: bounds,
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            smash_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step(
        &mut self,
        level_elapsed: usize,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        custom_gravity: bool,
        y_look_offset: i16,
    ) {
        self.step_with_order(
            level_elapsed,
            input,
            jump_pressed,
            form,
            block_grid,
            block_list,
            custom_gravity,
            y_look_offset,
            PhysicsOrder::GravityFirst,
            true,
            &[],
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step_with_order(
        &mut self,
        level_elapsed: usize,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        custom_gravity: bool,
        y_look_offset: i16,
        physics_order: PhysicsOrder,
        allow_asm_faithful_solid_touch_snap: bool,
        deferred_prize_ceiling_cells: &[(u16, u16)],
    ) {
        if level_elapsed < OPENING_SCROLL_START_FRAME {
            self.run_entry_animation(level_elapsed);
            return;
        }
        // ASM pull-up animation (stru_8BB4): Hibernate_UntilAnimFinished
        // loops for 21 frames after ledge snap, suppressing all physics.
        // Camera follow resumes on the final frame (ASM exits animation,
        // transitions to Standingstill, then main loop runs camera follow).
        if self.player.ledge_snap_cooldown > 0 {
            self.player.ledge_snap_cooldown -= 1;
            if self.player.ledge_snap_cooldown > 0 {
                return;
            }
            // Last cooldown frame: physics still suppressed but camera catches up.
            self.update_camera_follow(y_look_offset);
            return;
        }
        let mut block_grid = block_grid;
        let on_ground_before = self.player.on_ground;
        self.wall_cling_release_probe = None;
        self.kid_flip_follow_on_stages = None;
        self.kid_jump_gravity_probe = None;
        // ASM sub_914A uses FA78 (per-form half-width from unk_B408) for all
        // collision probes. Thread the form's half-width through to collision
        // functions instead of the constant PLAYER_HALF_WIDTH.
        let collision_hw = form.half_width();

        match physics_order {
            PhysicsOrder::GravityFirst => {
                if let Some(ref mut t) = self.physics_trace {
                    t.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.on_ground_before = on_ground_before;
                }

                // Current native order: horizontal, vertical (gravity+position), collisions
                let horizontal_step_applied =
                    self.update_horizontal_motion(input, jump_pressed, form, block_grid.as_deref());

                if let Some(ref mut t) = self.physics_trace {
                    t.post_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.horizontal_step_applied = horizontal_step_applied;
                    t.pre_vertical = (self.player.y_pos, self.player.y_velocity);
                    t.pre_gravity_y_vel = self.player.y_velocity;
                }

                // Save pre-gravity y_vel for floor landing sub-pixel logic (ASM tests
                // y_vel in Character_CheckCollision, before sub_B168 applies gravity).
                let pre_gravity_y_vel = self.player.y_velocity;
                self.update_vertical_motion(input, jump_pressed, form, custom_gravity);
                self.pre_gravity_y_vel = pre_gravity_y_vel;

                if let Some(ref mut t) = self.physics_trace {
                    t.post_gravity_y_vel = self.player.y_velocity;
                    t.gravity_applied =
                        self.player.y_velocity != pre_gravity_y_vel || !self.player.on_ground;
                    t.ascending_and_b_held = pre_gravity_y_vel < 0 && input.b;
                    t.post_vertical = (self.player.y_pos, self.player.y_velocity);
                    // Detect jump initiation: y_vel went negative from non-negative
                    if pre_gravity_y_vel >= 0 && self.player.y_velocity < 0 {
                        t.jump_initiated = true;
                        t.jump_y_vel = Some(self.player.y_velocity);
                    }
                }

                let pre_collision_x = self.player.x_pos;
                let airborne_slope_recontact = if let Some(grid) = block_grid.as_deref_mut() {
                    self.apply_wall_collision(Some(grid), collision_hw);
                    if self.player.x_pos != pre_collision_x
                        && let Some(ref mut t) = self.physics_trace
                    {
                        t.wall_hit = true;
                        t.wall_snap_x = Some(self.player.x_pos);
                    }
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid, collision_hw) {
                        self.finalise_trace();
                        // ASM camera follow runs after all physics sub-steps. On the
                        // ledge-snap frame, the camera follow sees the pre-snap
                        // position — skip the follow here and let the next frame
                        // catch up, matching the oracle's one-frame camera lag.
                        return;
                    }
                    self.apply_ceiling_collision(
                        Some(grid),
                        block_list,
                        form,
                        collision_hw,
                        deferred_prize_ceiling_cells,
                    );
                    self.apply_floor_collision(
                        Some(grid),
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                } else {
                    self.apply_floor_collision(
                        None,
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                };
                if let (Some(grid), Some(collision)) =
                    (block_grid.as_deref(), airborne_slope_recontact)
                {
                    self.apply_immediate_slope_recontact(
                        input,
                        form,
                        collision,
                        grid,
                        collision_hw,
                    );
                }
            }
            PhysicsOrder::PositionFirst => {
                if let Some(ref mut t) = self.physics_trace {
                    t.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.on_ground_before = self.player.on_ground;
                }
                // ASM order: horizontal motion + position, apply y_pos from y_velocity
                // (without gravity first), collisions, then gravity
                let horizontal_step_applied =
                    self.update_horizontal_motion(input, jump_pressed, form, block_grid.as_deref());
                if let Some(ref mut t) = self.physics_trace {
                    t.post_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.horizontal_step_applied = horizontal_step_applied;
                    t.pre_vertical = (self.player.y_pos, self.player.y_velocity);
                    t.pre_gravity_y_vel = self.player.y_velocity;
                }
                // Apply vertical position from current y_velocity (before gravity)
                self.apply_vertical_position_only(input, jump_pressed, form);
                // In PositionFirst, y_vel at collision time is pre-gravity (matches ASM).
                self.pre_gravity_y_vel = self.player.y_velocity;
                if let Some(ref mut t) = self.physics_trace {
                    t.post_vertical = (self.player.y_pos, self.player.y_velocity);
                }
                let airborne_slope_recontact = if let Some(grid) = block_grid.as_deref_mut() {
                    self.apply_wall_collision(Some(grid), collision_hw);
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid, collision_hw) {
                        self.finalise_trace();
                        // ASM camera follow runs after all physics sub-steps. On the
                        // ledge-snap frame, the camera follow sees the pre-snap
                        // position — skip the follow here and let the next frame
                        // catch up, matching the oracle's one-frame camera lag.
                        return;
                    }
                    self.apply_ceiling_collision(
                        Some(grid),
                        block_list,
                        form,
                        collision_hw,
                        deferred_prize_ceiling_cells,
                    );
                    self.apply_floor_collision(
                        Some(grid),
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                } else {
                    self.apply_floor_collision(
                        None,
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                };
                if let (Some(grid), Some(collision)) =
                    (block_grid.as_deref(), airborne_slope_recontact)
                {
                    self.apply_immediate_slope_recontact(
                        input,
                        form,
                        collision,
                        grid,
                        collision_hw,
                    );
                }
                // Apply gravity after collisions (ASM sub_B168 runs after collision checks)
                let pre_post_grav_vel = self.player.y_velocity;
                if !self.player.on_ground && !custom_gravity {
                    let ascending_and_b_held = self.player.y_velocity < 0 && input.b;
                    self.player.y_velocity =
                        physics::apply_gravity(self.player.y_velocity, ascending_and_b_held);
                    if let Some(ref mut t) = self.physics_trace {
                        t.post_gravity_y_vel = self.player.y_velocity;
                        t.gravity_applied = true;
                        t.ascending_and_b_held = ascending_and_b_held;
                    }
                } else if let Some(ref mut t) = self.physics_trace {
                    t.post_gravity_y_vel = pre_post_grav_vel;
                }
            }
            PhysicsOrder::AsmFaithful => {
                if let Some(ref mut t) = self.physics_trace {
                    t.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.on_ground_before = on_ground_before;
                }

                // ASM walking coroutine (loc_8BF0): sub_B43A checks floor at
                // (y_pos + 1) / 16 across player width BEFORE any velocity/position
                // updates. If no Solid/Spikes cell found → loc_B5F6: y_pos += 1px,
                // transition to Jump, skip sub_9A0A/sub_8F26 entirely.
                if on_ground_before && let Some(grid) = block_grid.as_deref() {
                    let x_int = (self.player.x_pos >> 16) as i16 as i32;
                    let y_int = (self.player.y_pos >> 16) as i16 as i32;
                    let probe_row = ((y_int + 1) >> 4).max(0) as u16;
                    let left_col = ((x_int - collision_hw) >> 4).max(0) as u16;
                    let right_col = ((x_int + collision_hw) >> 4)
                        .max(0)
                        .min(grid.width_cells as i32 - 1)
                        as u16;
                    let mut has_floor = false;
                    for col in left_col..=right_col {
                        let cell = grid.cell_at(col, probe_row);
                        if matches!(
                            cell.collision,
                            terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
                        ) {
                            has_floor = true;
                            break;
                        }
                    }
                    if !has_floor {
                        // loc_B5F6: addq.w #1,y_pos(a3)
                        self.player.y_pos = self.player.y_pos.wrapping_add(0x0001_0000);
                        self.player.on_ground = false;
                        self.player.on_spikes = false;
                        self.terrain.floor_terrain_type = TerrainType::Normal;
                        self.finalise_trace();
                        self.update_camera_follow(y_look_offset);
                        return;
                    }
                }

                let entry_stage = self.kid_flip_follow_on_stage_state();
                let mut kid_flip_follow_on_stages = KidFlipFollowOnStages {
                    entry: entry_stage,
                    after_press_away: entry_stage,
                    after_jump_gravity: entry_stage,
                    after_floor: entry_stage,
                    after_wall: entry_stage,
                };

                // ASM loc_AA9C: pressing away from wall exits wall-cling.
                if let Some(dir) = self.player.wall_cling_direction {
                    let pressing_away = match dir {
                        Direction::Right => input.left,
                        Direction::Left => input.right,
                    };
                    if pressing_away {
                        self.player.wall_cling_direction = None;
                    }
                }
                kid_flip_follow_on_stages.after_press_away = self.kid_flip_follow_on_stage_state();

                // Step 1: sub_B084 — horizontal velocity only (no x_pos update).
                // Suppressed during wall-cling (ASM loc_AA22 never calls sub_B084).
                let horizontal_step_applied = if self.player.wall_cling_direction.is_some() {
                    false
                } else {
                    self.update_horizontal_velocity_only(
                        input,
                        jump_pressed,
                        form,
                        block_grid.as_deref(),
                    )
                };

                if let Some(ref mut t) = self.physics_trace {
                    // Record post-horizontal with unchanged x_pos (vel-only update)
                    t.post_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.horizontal_step_applied = horizontal_step_applied;
                    t.pre_vertical = (self.player.y_pos, self.player.y_velocity);
                    t.pre_gravity_y_vel = self.player.y_velocity;
                }

                // Step 2: sub_B168 — jump initiation + gravity (velocity only)
                let pre_gravity_y_vel = self.player.y_velocity;
                self.apply_jump_and_gravity_vel_only(input, jump_pressed, form, custom_gravity);
                // In AsmFaithful, gravity runs before position, so pre_gravity_y_vel
                // is the velocity before gravity was applied (same as GravityFirst).
                self.pre_gravity_y_vel = pre_gravity_y_vel;
                kid_flip_follow_on_stages.after_jump_gravity =
                    self.kid_flip_follow_on_stage_state();

                if let Some(ref mut t) = self.physics_trace {
                    t.post_gravity_y_vel = self.player.y_velocity;
                    t.gravity_applied =
                        self.player.y_velocity != pre_gravity_y_vel || !self.player.on_ground;
                    t.ascending_and_b_held = pre_gravity_y_vel < 0 && input.b;
                    if pre_gravity_y_vel >= 0 && self.player.y_velocity < 0 {
                        t.jump_initiated = true;
                        t.jump_y_vel = Some(self.player.y_velocity);
                    }
                }

                // Step 3: sub_902A — apply y_pos += y_vel, then floor/ceiling collision.
                // ASM runs sub_902A (vertical) before sub_8F26 (horizontal), so floor
                // probes see old x_pos. This is critical at ledge edges: with old x_pos
                // the probe still finds ground, preventing a 1-frame-early fall.
                let pre_vertical_y_px = (self.player.y_pos >> 16) as i32;
                self.apply_y_velocity_to_position();

                if let Some(ref mut t) = self.physics_trace {
                    t.post_vertical = (self.player.y_pos, self.player.y_velocity);
                }

                let airborne_slope_recontact = if let Some(grid) = block_grid.as_deref_mut() {
                    self.apply_ceiling_collision(
                        Some(grid),
                        block_list,
                        form,
                        collision_hw,
                        deferred_prize_ceiling_cells,
                    );
                    self.apply_floor_collision(
                        Some(grid),
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        allow_asm_faithful_solid_touch_snap,
                    )
                } else {
                    self.apply_floor_collision(
                        None,
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        allow_asm_faithful_solid_touch_snap,
                    )
                };
                kid_flip_follow_on_stages.after_floor = self.kid_flip_follow_on_stage_state();
                if let (Some(grid), Some(collision)) =
                    (block_grid.as_deref(), airborne_slope_recontact)
                {
                    self.apply_immediate_slope_recontact(
                        input,
                        form,
                        collision,
                        grid,
                        collision_hw,
                    );
                }
                if let Some(grid) = block_grid.as_deref() {
                    self.record_wall_cling_release_probe(input, form, grid, pre_vertical_y_px);
                }
                if let Some(grid) = block_grid.as_deref()
                    && self.try_release_wall_cling_overhang(form, grid, pre_vertical_y_px)
                {
                    kid_flip_follow_on_stages.after_wall = self.kid_flip_follow_on_stage_state();
                    self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
                    self.finalise_trace();
                    self.update_camera_follow(y_look_offset);
                    return;
                }
                if let Some(grid) = block_grid.as_deref()
                    && self.try_release_descending_wall_cling_into_run(input, form, grid)
                {
                    kid_flip_follow_on_stages.after_wall = self.kid_flip_follow_on_stage_state();
                    self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
                    self.finalise_trace();
                    self.update_camera_follow(y_look_offset);
                    return;
                }

                // Step 4: sub_8F26 — apply x_pos += x_vel, then wall collision.
                self.apply_x_velocity_to_position();

                let pre_collision_x = self.player.x_pos;
                if let Some(grid) = block_grid {
                    self.apply_wall_collision(Some(grid), collision_hw);
                    if self.player.x_pos != pre_collision_x
                        && let Some(ref mut t) = self.physics_trace
                    {
                        t.wall_hit = true;
                        t.wall_snap_x = Some(self.player.x_pos);
                    }
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid, collision_hw) {
                        kid_flip_follow_on_stages.after_wall =
                            self.kid_flip_follow_on_stage_state();
                        self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
                        self.finalise_trace();
                        return;
                    }
                }
                kid_flip_follow_on_stages.after_wall = self.kid_flip_follow_on_stage_state();
                self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
            }
        }
        self.finalise_trace();
        self.update_camera_follow(y_look_offset);
    }

    fn held_horizontal_direction(input: FrameInput) -> Option<Direction> {
        match (input.left, input.right) {
            (true, false) => Some(Direction::Left),
            (false, true) => Some(Direction::Right),
            _ => None,
        }
    }

    fn kid_flip_follow_on_stage_state(&self) -> KidFlipFollowOnStageState {
        KidFlipFollowOnStageState {
            x_velocity: self.player.x_velocity,
            y_velocity: self.player.y_velocity,
            on_ground: self.player.on_ground,
            wall_cling_direction: self.player.wall_cling_direction,
        }
    }

    fn wall_cling_probe_direction(
        &self,
        held_direction: Option<Direction>,
    ) -> (Option<Direction>, KidWallClingProbeDirectionSource) {
        if let Some(direction) = self.player.wall_cling_direction {
            return (Some(direction), KidWallClingProbeDirectionSource::WallCling);
        }
        if let Some(direction) = held_direction {
            return (Some(direction), KidWallClingProbeDirectionSource::Held);
        }
        (
            Some(self.player.facing),
            KidWallClingProbeDirectionSource::Facing,
        )
    }

    fn wall_cling_direction_is_sticky(&self, direction: Direction) -> bool {
        match direction {
            Direction::Right => self.player.right_wall_sticky,
            Direction::Left => self.player.left_wall_sticky,
        }
    }

    fn record_wall_cling_release_probe(
        &mut self,
        input: FrameInput,
        form: PlayerForm,
        grid: &terrain::BlockGrid,
        pre_vertical_y_px: i32,
    ) {
        let held_direction = Self::held_horizontal_direction(input);
        let (probe_direction, probe_direction_source) =
            self.wall_cling_probe_direction(held_direction);
        let (forward_wall_probe, top_edge_probe) = match probe_direction {
            Some(direction) => {
                let moving_right = direction == Direction::Right;
                (
                    Some(terrain::probe_wall(
                        grid,
                        (self.player.x_pos >> 16) as i32,
                        (self.player.y_pos >> 16) as i32,
                        form.half_width(),
                        PLAYER_HEIGHT,
                        moving_right,
                    )),
                    Some(terrain::probe_wall_top_edge(
                        grid,
                        (self.player.x_pos >> 16) as i32,
                        (self.player.y_pos >> 16) as i32,
                        form.half_width(),
                        PLAYER_HEIGHT,
                        moving_right,
                    )),
                )
            }
            None => (None, None),
        };

        let branch = match self.player.wall_cling_direction {
            Some(direction) => {
                let pressing_away = match direction {
                    Direction::Right => input.left,
                    Direction::Left => input.right,
                };
                if pressing_away {
                    KidWallClingReleaseBranch::PressAwayExit
                } else if self.player.y_velocity < 0 {
                    KidWallClingReleaseBranch::SubAe6e
                } else {
                    KidWallClingReleaseBranch::SubAdb8
                }
            }
            None => KidWallClingReleaseBranch::NoWallClingState,
        };

        let sub_adb8_gate = match self.player.wall_cling_direction {
            None => KidWallClingReleaseGate::WallClingMissing,
            Some(direction) => {
                if !self.wall_cling_direction_is_sticky(direction) {
                    KidWallClingReleaseGate::StickyInactive
                } else if self.player.on_ground {
                    KidWallClingReleaseGate::OnGround
                } else if self.player.y_velocity < 0 {
                    KidWallClingReleaseGate::Rising
                } else if held_direction != Some(direction) {
                    KidWallClingReleaseGate::HeldDirectionMismatch
                } else if forward_wall_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::ForwardWallBlocked
                } else if top_edge_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::TopEdgeBlocked
                } else {
                    KidWallClingReleaseGate::Ready
                }
            }
        };

        let post_vertical_y_px = (self.player.y_pos >> 16) as i32;
        let upward_displacement = pre_vertical_y_px - post_vertical_y_px;
        let distance_to_tile = (-(post_vertical_y_px + 1)) & 0xF;
        let sub_ae6e_gate = match self.player.wall_cling_direction {
            None => KidWallClingReleaseGate::WallClingMissing,
            Some(direction) => {
                if !self.wall_cling_direction_is_sticky(direction) {
                    KidWallClingReleaseGate::StickyInactive
                } else if self.player.y_velocity >= 0 {
                    KidWallClingReleaseGate::NotAscending
                } else if upward_displacement <= distance_to_tile {
                    KidWallClingReleaseGate::NeedsTileCrossing
                } else if forward_wall_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::ForwardWallBlocked
                } else if top_edge_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::TopEdgeBlocked
                } else {
                    KidWallClingReleaseGate::Ready
                }
            }
        };

        self.wall_cling_release_probe = Some(KidWallClingReleaseProbe {
            held_direction,
            probe_direction,
            probe_direction_source,
            wall_cling_direction: self.player.wall_cling_direction,
            right_wall_sticky: self.player.right_wall_sticky,
            left_wall_sticky: self.player.left_wall_sticky,
            on_ground: self.player.on_ground,
            y_velocity: self.player.y_velocity,
            pre_vertical_y_px,
            post_vertical_y_px,
            branch,
            sub_adb8_gate,
            sub_ae6e_gate,
            forward_wall_probe,
            top_edge_probe,
        });
    }

    fn should_hold_standing_still(
        &mut self,
        input: FrameInput,
        block_grid: Option<&terrain::BlockGrid>,
        collision_hw: i32,
    ) -> bool {
        if !self.player.on_ground || self.player.x_velocity != 0 {
            return false;
        }
        let Some(direction) = Self::held_horizontal_direction(input) else {
            return false;
        };
        let Some(grid) = block_grid else {
            return false;
        };

        self.player.facing = direction;
        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        terrain::probe_wall(
            grid,
            x_px,
            y_px,
            collision_hw,
            PLAYER_HEIGHT,
            direction == Direction::Right,
        )
        .hit
    }

    fn try_release_wall_cling_overhang(
        &mut self,
        form: PlayerForm,
        grid: &terrain::BlockGrid,
        pre_vertical_y_px: i32,
    ) -> bool {
        let Some(direction) = self.player.wall_cling_direction else {
            return false;
        };
        let sticky_active = match direction {
            Direction::Right => self.player.right_wall_sticky,
            Direction::Left => self.player.left_wall_sticky,
        };
        if !sticky_active || self.player.y_velocity >= 0 {
            return false;
        }

        let y_px = (self.player.y_pos >> 16) as i32;
        let upward_displacement = pre_vertical_y_px - y_px;
        let distance_to_tile = (-(y_px + 1)) & 0xF;
        if upward_displacement <= distance_to_tile {
            return false;
        }

        // ASM jump wall-cling state keeps suppressing h_accel until the first
        // upward tile-row crossing where the overhang has actually cleared.
        // On that release frame, the sticky wall hold contributes a 1px x-pos
        // shimmy and clears the cling state, which is exactly the demo-1
        // frame-2723 frontier behaviour.
        let x_px = (self.player.x_pos >> 16) as i32;
        let moving_right = direction == Direction::Right;
        if terrain::probe_wall(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }
        if terrain::probe_wall_top_edge(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }

        match direction {
            Direction::Right => self.player.x_pos = self.player.x_pos.wrapping_add(0x0001_0000),
            Direction::Left => self.player.x_pos = self.player.x_pos.wrapping_sub(0x0001_0000),
        }
        self.player.x_velocity = 0;
        self.player.wall_cling_direction = None;
        self.player.right_wall_sticky = false;
        self.player.left_wall_sticky = false;
        true
    }

    fn try_release_descending_wall_cling_into_run(
        &mut self,
        input: FrameInput,
        form: PlayerForm,
        grid: &terrain::BlockGrid,
    ) -> bool {
        let Some(direction) = self.player.wall_cling_direction else {
            return false;
        };
        let sticky_active = match direction {
            Direction::Right => self.player.right_wall_sticky,
            Direction::Left => self.player.left_wall_sticky,
        };
        if !sticky_active || self.player.on_ground || self.player.y_velocity < 0 {
            return false;
        }
        if Self::held_horizontal_direction(input) != Some(direction) {
            return false;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let moving_right = direction == Direction::Right;
        // ASM loc_AA9C/loc_AD34: while descending in wall-cling, holding back
        // into the wall exits cling with a 1px shimmy once the wall/top-edge
        // probes both clear, so the next frame can re-enter jump air-accel.
        if terrain::probe_wall(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }
        if terrain::probe_wall_top_edge(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }

        match direction {
            Direction::Right => self.player.x_pos = self.player.x_pos.wrapping_add(0x0001_0000),
            Direction::Left => self.player.x_pos = self.player.x_pos.wrapping_sub(0x0001_0000),
        }
        self.player.x_velocity = 0;
        self.player.wall_cling_direction = None;
        self.player.right_wall_sticky = false;
        self.player.left_wall_sticky = false;
        true
    }

    /// Record final state into the physics trace and detect floor landing.
    fn finalise_trace(&mut self) {
        if let Some(ref mut t) = self.physics_trace {
            t.on_ground_after = self.player.on_ground;
            t.final_pos = (self.player.x_pos, self.player.y_pos);
            t.final_vel = (self.player.x_velocity, self.player.y_velocity);
            // Detect floor landing: transitioned from airborne to grounded
            if !t.on_ground_before && t.on_ground_after {
                t.floor_hit = true;
                t.floor_y_px = Some((self.terrain.floor_y_pos >> 16) as i32);
                // sub_902A clears sub-pixel on new landings (clr.w $20(a3))
                t.landing_path = Some(LandingPath::ZeroSubPixel);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step_iron_knight_climb_resume(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        y_look_offset: i16,
        allow_asm_faithful_solid_touch_snap: bool,
        deferred_prize_ceiling_cells: &[(u16, u16)],
    ) {
        let stage = self.kid_flip_follow_on_stage_state();
        self.kid_flip_follow_on_stages = Some(KidFlipFollowOnStages {
            entry: stage,
            after_press_away: stage,
            after_jump_gravity: stage,
            after_floor: stage,
            after_wall: stage,
        });
        self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
            branch: KidJumpGravityBranch::AirborneCustomGravity,
            jump_pressed,
            input_b_held: input.b,
            custom_gravity: true,
            on_ground_before: self.player.on_ground,
            on_ground_after: self.player.on_ground,
            just_jumped_before: self.player.just_jumped,
            just_jumped_after: self.player.just_jumped,
            jump_buffer_frames_before: self.player.jump_buffer_frames,
            jump_buffer_frames_after: self.player.jump_buffer_frames,
            y_velocity_before: self.player.y_velocity,
            y_velocity_after: self.player.y_velocity,
        });

        if let Some(ref mut trace) = self.physics_trace {
            trace.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
            trace.post_horizontal = (self.player.x_pos, self.player.x_velocity);
            trace.horizontal_step_applied = false;
            trace.pre_vertical = (self.player.y_pos, self.player.y_velocity);
            trace.pre_gravity_y_vel = self.player.y_velocity;
            trace.post_gravity_y_vel = self.player.y_velocity;
            trace.gravity_applied = false;
            trace.ascending_and_b_held = false;
        }
        self.pre_gravity_y_vel = self.player.y_velocity;

        let pre_vertical_y_px = (self.player.y_pos >> 16) as i32;
        self.apply_y_velocity_to_position();
        if let Some(ref mut trace) = self.physics_trace {
            trace.post_vertical = (self.player.y_pos, self.player.y_velocity);
        }

        if let Some(grid) = block_grid {
            self.apply_ceiling_collision(
                Some(grid),
                block_list,
                form,
                form.half_width(),
                deferred_prize_ceiling_cells,
            );
            self.apply_floor_collision(
                Some(grid),
                false,
                !jump_pressed,
                form.half_width(),
                allow_asm_faithful_solid_touch_snap,
            );
            self.record_wall_cling_release_probe(input, form, grid, pre_vertical_y_px);
        } else {
            self.apply_floor_collision(
                None,
                false,
                !jump_pressed,
                form.half_width(),
                allow_asm_faithful_solid_touch_snap,
            );
        }

        self.finalise_trace();
        self.update_camera_follow(y_look_offset);
    }

    pub(crate) fn try_snap_wall_sticky_ledge(
        &mut self,
        jump_pressed: bool,
        grid: &terrain::BlockGrid,
        collision_hw: i32,
    ) -> bool {
        let jump_requested = jump_pressed || self.player.jump_buffer_frames > 0;
        if !jump_requested || self.player.on_ground {
            return false;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let (probe_x_px, direct_probe, snap_x_px) = if self.player.right_wall_sticky {
            let probe_x_px = x_px + collision_hw + 1;
            let probe = terrain::probe_floor(grid, probe_x_px, y_px);
            let support_left_px = i32::from(probe.cell_x) * 16;
            (probe_x_px, probe, support_left_px + collision_hw - 1)
        } else if self.player.left_wall_sticky {
            let probe_x_px = x_px - collision_hw - 1;
            let probe = terrain::probe_floor(grid, probe_x_px, y_px);
            let support_right_px = (i32::from(probe.cell_x) + 1) * 16 - 1;
            (probe_x_px, probe, support_right_px - collision_hw + 1)
        } else {
            return false;
        };

        let probe = if direct_probe.cell.is_solid_floor() {
            direct_probe
        } else if self.player.y_velocity >= 0 && self.player.jump_buffer_frames > 0 {
            // ASM pull-up path (loc_ABA4) searches downward from roughly 20px
            // above the current feet position and snaps onto the first solid
            // ledge it finds while the wall-cling jump request is still live.
            let search_start_y_px = y_px.saturating_sub(0x14) & !0xF;
            let Some(surface_y_px) = terrain::find_floor_below(grid, probe_x_px, search_start_y_px)
            else {
                return false;
            };
            let scanned_probe = terrain::probe_floor(grid, probe_x_px, surface_y_px);
            if !scanned_probe.cell.is_solid_floor() {
                return false;
            }
            scanned_probe
        } else {
            return false;
        };

        let (floor_y_px, contact_y_px) = match probe.cell.collision {
            terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes => {
                (probe.floor_y_px.saturating_sub(1), probe.floor_y_px)
            }
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::HiddenBlock
            | terrain::CollisionKind::Empty => (probe.floor_y_px, probe.floor_y_px),
        };
        let contact_y_fp = (contact_y_px.max(0) as u32) << 16;
        let valid_contact_window = if self.player.y_velocity < 0 {
            self.player.y_pos >= contact_y_fp
                && self.player.y_pos <= contact_y_fp.saturating_add(15u32 << 16)
        } else {
            self.player.y_pos >= contact_y_fp
                && self.player.y_pos <= contact_y_fp.saturating_add(18u32 << 16)
        };
        if !valid_contact_window {
            return false;
        }

        let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
        self.player.x_pos = (snap_x_px.max(0) as u32) << 16;
        // Preserve sub-pixel from current position: ASM's ledge snap uses
        // move.w to set the pixel part, leaving the low word (sub-pixel) intact.
        self.player.y_pos = (floor_y_fp & 0xFFFF_0000) | (self.player.y_pos & 0x0000_FFFF);
        self.player.x_velocity = 0;
        self.player.y_velocity = 0;
        self.player.on_ground = true;
        self.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
        self.player.right_wall_sticky = false;
        self.player.left_wall_sticky = false;
        self.player.wall_cling_direction = None;
        // ASM has no jump buffer — the VBlank interrupt's Ctrl_1_Press is a
        // single-frame edge. Clear the buffer on ledge snap to prevent a
        // spurious jump on the next grounded frame.
        self.player.jump_buffer_frames = 0;
        self.player.ledge_snap_cooldown = LEDGE_SNAP_PULLUP_FRAMES;
        self.terrain.floor_y_pos = floor_y_fp;
        self.terrain.floor_terrain_type = TerrainType::from_block_type(probe.cell.block_type);
        true
    }

    /// Pre-physics entry animation: player descends and settles to ground.
    pub(crate) fn run_entry_animation(&mut self, level_elapsed: usize) {
        let previous_y_pos = self.player.y_pos;
        if level_elapsed <= OPENING_ENTRY_HOLD_FRAMES {
            self.camera.y_pos = OPENING_ENTRY_CAMERA_Y_FP;
            self.player.y_pos = OPENING_ENTRY_KID_Y_POS;
            self.player.y_velocity = 0;
            self.player.x_velocity = 0;
            self.player.on_ground = false;
            return;
        }
        let settle_progress = (level_elapsed - OPENING_ENTRY_HOLD_FRAMES)
            .min(OPENING_SCROLL_START_FRAME - OPENING_ENTRY_HOLD_FRAMES);
        let settle_span = OPENING_SCROLL_START_FRAME - OPENING_ENTRY_HOLD_FRAMES;
        self.camera.y_pos = interpolate_u32(
            OPENING_ENTRY_CAMERA_Y_FP,
            OPENING_GROUNDED_CAMERA_Y_FP,
            settle_progress,
            settle_span,
        );
        self.player.y_pos = interpolate_u32(
            OPENING_ENTRY_KID_Y_POS,
            OPENING_GROUNDED_KID_Y_POS,
            settle_progress,
            settle_span,
        );
        self.player.y_velocity =
            i32::try_from(self.player.y_pos.saturating_sub(previous_y_pos)).unwrap_or(i32::MAX);
        self.player.x_velocity = 0;
        self.player.on_ground = self.player.y_pos >= self.terrain.floor_y_pos;
        self.camera.x_pos = 0;
        self.player.x_pos = OPENING_ENTRY_KID_X_POS;
    }

    /// Horizontal velocity update only (sub_B084 equivalent).
    /// Updates x_vel and facing but does NOT apply x_pos += x_vel.
    /// Returns (horizontal_step_applied, jump_suppressed_by_buffer).
    pub(crate) fn update_horizontal_velocity_only(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&terrain::BlockGrid>,
    ) -> bool {
        if self.should_hold_standing_still(input, block_grid, form.half_width()) {
            return false;
        }
        if self.player.on_ground && self.player.jump_buffer_frames > 0 && !input.b {
            self.player.jump_buffer_frames = 0;
        }
        let grounded_slope = self
            .grounded_slope_collision(block_grid)
            .filter(|_| self.player.on_ground);
        if self.player.on_ground
            && self.player.y_velocity >= 0
            && self.player.jump_buffer_frames > 0
        {
            return false;
        }
        if self.player.on_ground && self.player.y_velocity >= 0 && jump_pressed {
            if let Some(collision) = grounded_slope {
                let h_input = HorizontalInput {
                    left: input.left,
                    right: input.right,
                    run: input.a,
                };
                let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                );
                self.player.x_velocity = new_vel;
                self.player.facing = new_facing;
            }
            return false;
        }

        let near_floor =
            self.player.y_pos.abs_diff(self.terrain.floor_y_pos) <= GROUND_SNAP_TOLERANCE_FP;
        if !self.player.on_ground && jump_pressed && self.player.y_velocity >= 0 && near_floor {
            return false;
        }

        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = if self.player.on_ground {
            if let Some(collision) = grounded_slope {
                update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                )
            } else {
                physics::update_horizontal_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    form.ground_speed_table(self.terrain.floor_terrain_type),
                )
            }
        } else if self.player.just_jumped {
            (self.player.x_velocity, self.player.facing)
        } else {
            let jump_profile = physics::JUMP_AIR_PROFILES[usize::from(form.helmet_id())];
            physics::update_jump_horizontal_velocity(
                self.player.x_velocity,
                self.player.facing,
                h_input,
                jump_profile,
            )
        };
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;
        true
    }

    /// Jump initiation + gravity on y_vel only (sub_B168 equivalent).
    /// Does NOT update x_pos or y_pos — only velocity fields.
    pub(crate) fn apply_jump_and_gravity_vel_only(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        custom_gravity: bool,
    ) {
        let before = self.player;
        // Jump initiation
        if self.player.on_ground && self.player.y_velocity >= 0 && jump_pressed {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
                branch: KidJumpGravityBranch::JumpPressed,
                jump_pressed,
                input_b_held: input.b,
                custom_gravity,
                on_ground_before: before.on_ground,
                on_ground_after: self.player.on_ground,
                just_jumped_before: before.just_jumped,
                just_jumped_after: self.player.just_jumped,
                jump_buffer_frames_before: before.jump_buffer_frames,
                jump_buffer_frames_after: self.player.jump_buffer_frames,
                y_velocity_before: before.y_velocity,
                y_velocity_after: self.player.y_velocity,
            });
            return;
        }

        // Jump buffer initiation
        if self.player.on_ground
            && self.player.y_velocity >= 0
            && self.player.jump_buffer_frames > 0
        {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
                branch: KidJumpGravityBranch::JumpBuffered,
                jump_pressed,
                input_b_held: input.b,
                custom_gravity,
                on_ground_before: before.on_ground,
                on_ground_after: self.player.on_ground,
                just_jumped_before: before.just_jumped,
                just_jumped_after: self.player.just_jumped,
                jump_buffer_frames_before: before.jump_buffer_frames,
                jump_buffer_frames_after: self.player.jump_buffer_frames,
                y_velocity_before: before.y_velocity,
                y_velocity_after: self.player.y_velocity,
            });
            return;
        }

        let branch = if !self.player.on_ground {
            if self.player.just_jumped {
                KidJumpGravityBranch::AirborneJustJumpedHold
            } else if custom_gravity {
                KidJumpGravityBranch::AirborneCustomGravity
            } else if self.player.y_velocity < 0 && input.b {
                KidJumpGravityBranch::AirborneHeldJumpGravity
            } else {
                KidJumpGravityBranch::AirborneDefaultGravity
            }
        } else {
            KidJumpGravityBranch::GroundedHold
        };

        // Airborne: apply gravity to y_vel only
        if !self.player.on_ground {
            if jump_pressed {
                self.player.jump_buffer_frames = JUMP_BUFFER_FRAMES;
            } else if self.player.jump_buffer_frames > 0 {
                self.player.jump_buffer_frames -= 1;
            }
            if self.player.just_jumped {
                self.player.just_jumped = false;
            } else if !custom_gravity {
                let ascending_and_b_held = self.player.y_velocity < 0 && input.b;
                self.player.y_velocity =
                    physics::apply_gravity(self.player.y_velocity, ascending_and_b_held);
            }
        } else {
            self.player.y_velocity = 0;
            self.player.just_jumped = false;
        }

        self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
            branch,
            jump_pressed,
            input_b_held: input.b,
            custom_gravity,
            on_ground_before: before.on_ground,
            on_ground_after: self.player.on_ground,
            just_jumped_before: before.just_jumped,
            just_jumped_after: self.player.just_jumped,
            jump_buffer_frames_before: before.jump_buffer_frames,
            jump_buffer_frames_after: self.player.jump_buffer_frames,
            y_velocity_before: before.y_velocity,
            y_velocity_after: self.player.y_velocity,
        });
    }

    /// Apply y_vel to y_pos only. ASM sub_902A applies vertical position
    /// before sub_8F26 applies horizontal, so floor probes see old x_pos.
    fn apply_y_velocity_to_position(&mut self) {
        let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
        self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
    }

    /// Apply x_vel to x_pos only. ASM sub_8F26 runs after sub_902A.
    fn apply_x_velocity_to_position(&mut self) {
        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
    }

    /// Physics-driven horizontal movement using speed tables.
    /// Uses ground tables when on_ground, air tables when airborne.
    pub(crate) fn update_horizontal_motion(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&terrain::BlockGrid>,
    ) -> bool {
        if self.should_hold_standing_still(input, block_grid, form.half_width()) {
            return false;
        }
        if self.player.on_ground && self.player.jump_buffer_frames > 0 && !input.b {
            self.player.jump_buffer_frames = 0;
        }
        let grounded_slope = self
            .grounded_slope_collision(block_grid)
            .filter(|_| self.player.on_ground);
        if self.player.on_ground
            && self.player.y_velocity >= 0
            && self.player.jump_buffer_frames > 0
        {
            return false;
        }
        if self.player.on_ground && self.player.y_velocity >= 0 && jump_pressed {
            if let Some(collision) = grounded_slope {
                let h_input = HorizontalInput {
                    left: input.left,
                    right: input.right,
                    run: input.a,
                };
                let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                );
                self.player.x_velocity = new_vel;
                self.player.facing = new_facing;
            }
            return false;
        }

        let near_floor =
            self.player.y_pos.abs_diff(self.terrain.floor_y_pos) <= GROUND_SNAP_TOLERANCE_FP;
        if !self.player.on_ground && jump_pressed && self.player.y_velocity >= 0 && near_floor {
            return false;
        }

        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = if self.player.on_ground {
            if let Some(collision) = grounded_slope {
                update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                )
            } else {
                physics::update_horizontal_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    form.ground_speed_table(self.terrain.floor_terrain_type),
                )
            }
        } else if self.player.just_jumped {
            (self.player.x_velocity, self.player.facing)
        } else {
            let jump_profile = physics::JUMP_AIR_PROFILES[usize::from(form.helmet_id())];
            physics::update_jump_horizontal_velocity(
                self.player.x_velocity,
                self.player.facing,
                h_input,
                jump_profile,
            )
        };
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;

        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
        true
    }

    pub(crate) fn grounded_slope_collision(
        &self,
        block_grid: Option<&terrain::BlockGrid>,
    ) -> Option<terrain::CollisionKind> {
        let grid = block_grid?;
        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        match terrain::probe_floor(grid, x_px, y_px).cell.collision {
            terrain::CollisionKind::SlopeDown => Some(terrain::CollisionKind::SlopeDown),
            terrain::CollisionKind::SlopeUp => Some(terrain::CollisionKind::SlopeUp),
            _ => None,
        }
    }

    /// Physics-driven vertical movement: jump initiation and gravity.
    pub(crate) fn update_vertical_motion(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        custom_gravity: bool,
    ) {
        if self.player.on_ground && self.player.y_velocity >= 0 && jump_pressed {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        if self.player.on_ground
            && self.player.y_velocity >= 0
            && self.player.jump_buffer_frames > 0
        {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        if !self.player.on_ground {
            if jump_pressed {
                self.player.jump_buffer_frames = JUMP_BUFFER_FRAMES;
            } else if self.player.jump_buffer_frames > 0 {
                self.player.jump_buffer_frames -= 1;
            }
            if self.player.just_jumped {
                self.player.just_jumped = false;
            } else if !custom_gravity {
                // Normal gravity; helmet flight abilities (Skycutter/Cyclone)
                // apply their own gravity before physics runs.
                let ascending_and_b_held = self.player.y_velocity < 0 && input.b;
                self.player.y_velocity =
                    physics::apply_gravity(self.player.y_velocity, ascending_and_b_held);
            }
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
        } else {
            self.player.y_velocity = 0;
            self.player.just_jumped = false;
        }
    }

    /// PositionFirst mode: apply y_pos from y_velocity (and handle jump initiation)
    /// but defer gravity to after collisions.
    pub(crate) fn apply_vertical_position_only(
        &mut self,
        _input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
    ) {
        // Jump initiation (same as update_vertical_motion)
        if self.player.on_ground && self.player.y_velocity >= 0 && jump_pressed {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        // Jump buffer initiation
        if self.player.on_ground
            && self.player.y_velocity >= 0
            && self.player.jump_buffer_frames > 0
        {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        // Airborne: apply position from current velocity (no gravity yet)
        if !self.player.on_ground {
            if jump_pressed {
                self.player.jump_buffer_frames = JUMP_BUFFER_FRAMES;
            } else if self.player.jump_buffer_frames > 0 {
                self.player.jump_buffer_frames -= 1;
            }
            if self.player.just_jumped {
                self.player.just_jumped = false;
            }
            // Position update only — gravity is deferred
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
        } else {
            self.player.y_velocity = 0;
            self.player.just_jumped = false;
        }
    }

    pub(crate) fn apply_wall_collision(
        &mut self,
        block_grid: Option<&terrain::BlockGrid>,
        collision_hw: i32,
    ) {
        let Some(grid) = block_grid else {
            self.player.right_wall_sticky = false;
            self.player.left_wall_sticky = false;
            return;
        };
        if self.player.x_velocity == 0 {
            if self.player.on_ground {
                self.player.right_wall_sticky = false;
                self.player.left_wall_sticky = false;
            }
            return;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let moving_right = self.player.x_velocity > 0;
        let moving_left = self.player.x_velocity < 0;
        if !moving_right || self.player.on_ground {
            self.player.right_wall_sticky = false;
        }
        if !moving_left || self.player.on_ground {
            self.player.left_wall_sticky = false;
        }

        // ASM sub_8F26 tile-boundary guard (kid.asm:10335-10341): skip wall probe
        // if pixel displacement doesn't cross a 16px tile boundary. Without this,
        // a false probe hit zeroes sub-pixel via x_pos = clamped_center << 16.
        // Suppressed when wall_sticky is set (first frame after wall hit needs
        // re-probe even though x_vel=0; subsequent wall-cling frames with
        // x_vel=0 are handled earlier in the jump loop).
        let wall_stuck = (moving_right && self.player.right_wall_sticky)
            || (moving_left && self.player.left_wall_sticky);
        if !wall_stuck {
            let old_x_pos = self
                .player
                .x_pos
                .wrapping_sub(self.player.x_velocity as u32);
            let old_px = (old_x_pos >> 16) as i32;
            let new_px = (self.player.x_pos >> 16) as i32;
            if moving_right {
                let displacement = new_px - old_px;
                let right_edge = old_px + collision_hw;
                let distance_to_tile = (-(right_edge + 1)) & 0xF;
                // ASM: cmp.w d0,d4 / ble (skip when displacement <= distance).
                // Includes distance=0 case (player at tile boundary, no crossing).
                if displacement <= distance_to_tile {
                    return;
                }
            } else {
                let displacement = old_px - new_px;
                let left_edge = old_px - collision_hw;
                let distance_to_tile = left_edge & 0xF;
                if displacement <= distance_to_tile {
                    return;
                }
            }
        }

        let mut probe =
            terrain::probe_wall(grid, x_px, y_px, collision_hw, PLAYER_HEIGHT, moving_right);

        if !probe.hit && !self.player.on_ground {
            let top_edge_probe = terrain::probe_wall_top_edge(
                grid,
                x_px,
                y_px,
                collision_hw,
                PLAYER_HEIGHT,
                moving_right,
            );
            if top_edge_probe.hit {
                probe = top_edge_probe;
            }
        }

        // Right wall top-edge retention (overhang sticky)
        if !probe.hit && moving_right && self.player.right_wall_sticky {
            let top_edge_probe =
                terrain::probe_wall_top_edge(grid, x_px, y_px, collision_hw, PLAYER_HEIGHT, true);
            if top_edge_probe.hit {
                probe = top_edge_probe;
            } else {
                self.player.x_pos = ((self.player.x_pos >> 16) + 1) << 16;
                self.player.x_velocity = 0;
                self.player.right_wall_sticky = false;
                return;
            }
        }

        // Left wall top-edge retention (overhang sticky)
        if !probe.hit && moving_left && self.player.left_wall_sticky {
            let top_edge_probe =
                terrain::probe_wall_top_edge(grid, x_px, y_px, collision_hw, PLAYER_HEIGHT, false);
            if top_edge_probe.hit {
                probe = top_edge_probe;
            } else {
                self.player.x_pos = ((self.player.x_pos >> 16) - 1) << 16;
                self.player.x_velocity = 0;
                self.player.left_wall_sticky = false;
                return;
            }
        }

        if probe.hit {
            let clamped_center = if moving_right {
                probe.wall_x_px - (collision_hw + 1)
            } else {
                // Leftward wall probes test the current left edge pixel
                // (`center_x - half_width`), so the flush non-colliding centre
                // sits exactly `half_width` pixels to the right of the wall
                // surface. The extra `+1` belongs only to the moving-right path,
                // which probes one pixel beyond the right edge.
                probe.wall_x_px + collision_hw
            };
            if moving_right && x_px < clamped_center {
                self.player.right_wall_sticky = false;
                return;
            }
            if moving_left && x_px > clamped_center {
                self.player.left_wall_sticky = false;
                return;
            }
            self.player.x_pos = (clamped_center.max(0) as u32) << 16;
            self.player.x_velocity = 0;
            self.player.right_wall_sticky = moving_right && !self.player.on_ground;
            self.player.left_wall_sticky = moving_left && !self.player.on_ground;
            // ASM loc_A8DE: enter wall-cling on airborne wall hit. Suppresses
            // sub_B084 (h_accel) in subsequent frames until landing or press-away.
            if !self.player.on_ground {
                self.player.wall_cling_direction = Some(if moving_right {
                    Direction::Right
                } else {
                    Direction::Left
                });
            }
        } else {
            self.player.right_wall_sticky = false;
            self.player.left_wall_sticky = false;
        }
    }

    pub(crate) fn apply_ceiling_collision(
        &mut self,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        form: PlayerForm,
        collision_hw: i32,
        deferred_prize_ceiling_cells: &[(u16, u16)],
    ) {
        let Some(grid) = block_grid else { return };
        if self.player.y_velocity >= 0 {
            return;
        }

        // ASM sub_902A upward path only probes the ceiling if the current
        // movement actually crosses the next tile boundary above the head.
        // Without this guard, we keep re-hitting the same ceiling tile every
        // frame while still near it, producing a false multi-frame latch.
        let collision_height = form.ceiling_probe_height();
        let old_y_pos = self
            .player
            .y_pos
            .wrapping_sub(self.player.y_velocity as u32);
        let old_y_px = (old_y_pos >> 16) as i32;
        let new_y_px = (self.player.y_pos >> 16) as i32;
        let upward_displacement = old_y_px - new_y_px;
        let top_edge_old = old_y_px - collision_height;
        let distance_to_tile = top_edge_old & 0xF;
        if upward_displacement <= distance_to_tile {
            return;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;

        let probe = terrain::probe_ceiling(grid, x_px, y_px, collision_hw, collision_height);

        if probe.hit {
            let deferred_prize_ceiling_cell = deferred_prize_ceiling_cells
                .iter()
                .any(|&(col, row)| col == probe.cell_x && row == probe.cell_y);
            // Hidden block reveal: headbutting a hidden block from below
            // changes it to solid, making the block visible and functional.
            if !deferred_prize_ceiling_cell
                && probe.cell.collision == terrain::CollisionKind::HiddenBlock
            {
                // Change collision from HiddenBlock(2) to Solid(6),
                // preserving the block type and skin.
                let block_flag: u8 = if probe.cell.has_block { 0x80 } else { 0 };
                let block_type_bits = probe.cell.block_type as u8 & 0x0F;
                let new_hi = block_flag | 0x60 | block_type_bits; // Solid(6) collision
                grid.set_cell_at(
                    probe.cell_x,
                    probe.cell_y,
                    terrain::Cell::from_be_bytes(new_hi, probe.cell.skin),
                );

                // If the hidden block contains a prize, dispatch it
                if probe.cell.has_block && probe.cell.block_type == terrain::BlockType::Prize {
                    let content = block_list
                        .and_then(|bl| bl.prize_content_at(probe.cell_x, probe.cell_y))
                        .unwrap_or(terrain::PrizeContent::Diamond);
                    let block_x_px = probe.cell_x * 16;
                    let block_y_px = probe.cell_y * 16;
                    self.prize_block_hit = Some((content, block_x_px, block_y_px));
                    // ASM sub_1007A: keep cell Solid, clear has_block
                    grid.set_cell_at(
                        probe.cell_x,
                        probe.cell_y,
                        terrain::Cell::from_be_bytes(0x6F, probe.cell.skin),
                    );
                    self.ceiling_prize_clear = Some((probe.cell_x, probe.cell_y, probe.cell));
                }
            }

            // Prize/smash/lift block headbutt: hitting from below.
            if !deferred_prize_ceiling_cell
                && probe.cell.collision == terrain::CollisionKind::Solid
                && probe.cell.has_block
            {
                match probe.cell.block_type {
                    terrain::BlockType::Prize => {
                        let content = block_list
                            .and_then(|bl| bl.prize_content_at(probe.cell_x, probe.cell_y))
                            .unwrap_or(terrain::PrizeContent::Diamond);
                        let block_x_px = probe.cell_x * 16;
                        let block_y_px = probe.cell_y * 16;
                        self.prize_block_hit = Some((content, block_x_px, block_y_px));
                        // ASM sub_1007A writes $6F to the cell: Solid collision,
                        // has_block=false, block_type=Deleted. The tile stays solid
                        // as structural ceiling for ~32 frames (bounce animation),
                        // then clears. Without this, Cyclone flight near a Prize
                        // ceiling clears the tile to Empty immediately, letting
                        // the player fly through.
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0x6F, probe.cell.skin),
                        );
                        self.ceiling_prize_clear = Some((probe.cell_x, probe.cell_y, probe.cell));
                        // ASM sub_914A scans ALL tiles across player width, so
                        // adjacent Smash blocks are cleared on the same frame.
                        // probe_ceiling returns on the first hit, missing the
                        // other edge. Clear adjacent Smash blocks now.
                        let top_y = y_px - collision_height;
                        let tile_row = (top_y / 16).max(0) as u16;
                        for &edge_x in &[x_px - collision_hw, x_px + collision_hw] {
                            let col = (edge_x / 16).max(0) as u16;
                            if col == probe.cell_x && tile_row == probe.cell_y {
                                continue;
                            }
                            let cell = grid.cell_at(col, tile_row);
                            if cell.collision == terrain::CollisionKind::Solid
                                && cell.has_block
                                && cell.block_type == terrain::BlockType::Smash
                            {
                                grid.set_cell_at(col, tile_row, terrain::Cell::from_be_bytes(0, 0));
                            }
                        }
                    }
                    terrain::BlockType::Smash => {
                        let block_x_px = probe.cell_x * 16;
                        let block_y_px = probe.cell_y * 16;
                        self.smash_block_hit = Some((block_x_px, block_y_px));
                        // ASM j_Block_LoadSmashBits clears Smash blocks immediately
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0, 0),
                        );
                    }
                    terrain::BlockType::Lift => {
                        let block_x_px = probe.cell_x * 16;
                        let block_y_px = probe.cell_y * 16;
                        self.lift_block_hit = Some((block_x_px, block_y_px));
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0, 0),
                        );
                    }
                    terrain::BlockType::Drill => {
                        let drill_x_fp = (u32::from(probe.cell_x) * 16 + 8) << 16;
                        let drill_y_fp = (u32::from(probe.cell_y) * 16 + 16) << 16;
                        self.drill_projectile_hit = Some((drill_x_fp, drill_y_fp));
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0, 0),
                        );
                    }
                    terrain::BlockType::Metal => {
                        self.metal_block_hit = Some((probe.cell_x, probe.cell_y));
                    }
                    _ => {}
                }
            }

            // Push feet down so head clears the ceiling: feet = ceiling_bottom + height
            let new_y = probe.ceiling_y_px + collision_height;
            self.player.y_pos = (new_y.max(0) as u32) << 16;
            // ASM sub_902A ceiling path (loc_908C): clr.l y_vel(a3) then
            // clr.w $20(a3) (zero sub-pixel). y_pos is already pixel-aligned
            // from the << 16 above (sub-pixel zeroed). sub_BB54 also re-zeros
            // y_vel at line 15092.
            self.player.y_velocity = 0;
        }
    }

    pub(crate) fn apply_floor_collision(
        &mut self,
        block_grid: Option<&terrain::BlockGrid>,
        horizontal_step_applied: bool,
        allow_upward_slope_snap: bool,
        collision_hw: i32,
        allow_solid_touch_snap: bool,
    ) -> Option<terrain::CollisionKind> {
        if let Some(grid) = block_grid {
            let was_on_ground = self.player.on_ground;
            let x_px = (self.player.x_pos >> 16) as i32;
            let y_px = (self.player.y_pos >> 16) as i32;
            let left_edge_probe = terrain::probe_floor(grid, x_px - collision_hw, y_px);
            let right_edge_probe = terrain::probe_floor(grid, x_px + collision_hw, y_px);
            let mut probe = terrain::probe_floor_hitbox(grid, x_px, y_px, collision_hw);
            if was_on_ground
                && matches!(
                    probe.cell.collision,
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                )
            {
                let edge_probe = match (left_edge_probe.on_ground, right_edge_probe.on_ground) {
                    (true, true) => {
                        if left_edge_probe.floor_y_px >= right_edge_probe.floor_y_px {
                            Some(left_edge_probe)
                        } else {
                            Some(right_edge_probe)
                        }
                    }
                    (true, false) => Some(left_edge_probe),
                    (false, true) => Some(right_edge_probe),
                    (false, false) => None,
                };
                if let Some(edge_probe) = edge_probe
                    && edge_probe.floor_y_px > probe.floor_y_px
                {
                    probe = edge_probe;
                }
            }
            let mut allow_hitbox_snap = true;
            let mut center_slope_snap_candidate = false;
            if !was_on_ground {
                let center_probe = terrain::probe_floor(grid, x_px, y_px);
                if matches!(
                    center_probe.cell.collision,
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                ) {
                    let center_floor_fp = (center_probe.floor_y_px.max(0) as u32) << 16;
                    let near_center_slope = self.player.y_pos >= center_floor_fp
                        || self.player.y_pos.abs_diff(center_floor_fp)
                            <= SLOPE_AIR_SNAP_TOLERANCE_FP;
                    if near_center_slope {
                        probe = center_probe;
                        center_slope_snap_candidate = true;
                    } else {
                        allow_hitbox_snap = matches!(
                            left_edge_probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        ) || matches!(
                            right_edge_probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        );
                    }
                }
            }
            let preserving_ground_contact = self.player.on_ground && probe.cell.is_solid_floor();
            if center_slope_snap_candidate
                || (allow_hitbox_snap && (probe.on_ground || preserving_ground_contact))
            {
                let (floor_y_px, contact_y_px) = match probe.cell.collision {
                    terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes => {
                        (probe.floor_y_px.saturating_sub(1), probe.floor_y_px)
                    }
                    terrain::CollisionKind::SlopeDown
                    | terrain::CollisionKind::SlopeUp
                    | terrain::CollisionKind::HiddenBlock
                    | terrain::CollisionKind::Empty => (probe.floor_y_px, probe.floor_y_px),
                };
                let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
                let contact_y_fp = (contact_y_px.max(0) as u32) << 16;
                self.terrain.floor_y_pos = floor_y_fp;
                let within_snap =
                    self.player.y_pos.abs_diff(floor_y_fp) <= GROUND_SNAP_TOLERANCE_FP;
                let can_land = match probe.cell.collision {
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp => {
                        let within_slope_touch_snap =
                            self.player.y_pos.abs_diff(floor_y_fp) <= 0x0001_0000;
                        (self.player.y_velocity >= 0
                            && (self.player.y_pos >= floor_y_fp || within_slope_touch_snap))
                            || (allow_upward_slope_snap
                                && center_slope_snap_candidate
                                && self.player.y_velocity < 0
                                && self.player.y_velocity >= -(SLOPE_AIR_SNAP_TOLERANCE_FP as i32)
                                && (self.player.y_pos >= floor_y_fp || within_snap))
                    }
                    terrain::CollisionKind::Solid
                    | terrain::CollisionKind::Spikes
                    | terrain::CollisionKind::HiddenBlock
                    | terrain::CollisionKind::Empty => {
                        // ORIGINAL_QUIRK: ASM sub_902A only lands after the
                        // falling step reaches the solid contact row. The
                        // AsmFaithful path must not collapse a remaining
                        // sub-half-pixel gap into an early landing.
                        let within_solid_touch_snap = allow_solid_touch_snap
                            && !was_on_ground
                            && self.player.y_pos.abs_diff(floor_y_fp) < 0x0000_8000;
                        self.player.y_velocity >= 0
                            && (self.player.y_pos >= contact_y_fp
                                || within_solid_touch_snap
                                || (was_on_ground && within_snap))
                    }
                };
                if can_land {
                    let should_apply_slope_recontact = !was_on_ground
                        && self.player.y_velocity < 0
                        && matches!(
                            probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        );
                    if !was_on_ground
                        && horizontal_step_applied
                        && !matches!(
                            probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        )
                    {
                        let landing_x =
                            i64::from(self.player.x_pos) - i64::from(self.player.x_velocity);
                        self.player.x_pos = landing_x.clamp(0, i64::from(u32::MAX)) as u32;
                    }
                    // ASM sub_902A floor collision (kid.asm:10461-10462):
                    // When falling through a floor tile, sub_902A snaps y_pos
                    // and does `clr.w $20(a3)` (clears sub-pixel). But when
                    // already on the ground (y_vel=0, no tile crossing), sub_902A
                    // exits early via loc_913A without touching y_pos. So we
                    // only clear sub-pixel on actual landings (airborne→grounded).
                    if !was_on_ground {
                        // New landing: clear sub-pixel (sub_902A clr.w $20(a3))
                        self.player.y_pos = floor_y_fp & 0xFFFF_0000;
                    } else {
                        // Already grounded: preserve sub-pixel (sub_902A exits early)
                        self.player.y_pos =
                            (floor_y_fp & 0xFFFF_0000) | (self.player.y_pos & 0x0000_FFFF);
                    }
                    // Mushroom blocks bounce the player back up
                    if probe.cell.block_type == terrain::BlockType::Mushroom {
                        self.player.y_velocity =
                            -(self.player.y_velocity.abs() * 3 / 4).max(MUSHROOM_MIN_BOUNCE);
                        self.player.on_ground = false;
                    } else {
                        self.player.on_ground = true;
                        self.player.y_velocity = 0;
                        self.player.wall_cling_direction = None;
                    }
                    // Teleporter blocks warp player on contact
                    if probe.cell.has_block
                        && probe.cell.block_type == terrain::BlockType::Teleporter
                    {
                        let tp_col = (x_px / 16).max(0) as u16;
                        let tp_row = (probe.floor_y_px / 16).max(0) as u16;
                        self.teleporter_contact = Some((tp_col, tp_row));
                    }
                    self.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
                    self.terrain.floor_terrain_type =
                        TerrainType::from_block_type(probe.cell.block_type);
                    if should_apply_slope_recontact {
                        return match probe.cell.collision {
                            terrain::CollisionKind::SlopeDown => {
                                Some(terrain::CollisionKind::SlopeDown)
                            }
                            terrain::CollisionKind::SlopeUp => {
                                Some(terrain::CollisionKind::SlopeUp)
                            }
                            _ => None,
                        };
                    }
                }
            } else if self.player.on_ground {
                // Was on ground but no longer — start falling
                self.player.on_ground = false;
                self.player.on_spikes = false;
                self.terrain.floor_terrain_type = TerrainType::Normal;
            }
        } else if self.player.on_ground {
            self.player.on_ground = false;
            self.player.on_spikes = false;
            self.terrain.floor_terrain_type = TerrainType::Normal;
        }
        None
    }

    pub(crate) fn apply_immediate_slope_recontact(
        &mut self,
        input: FrameInput,
        form: PlayerForm,
        collision: terrain::CollisionKind,
        grid: &terrain::BlockGrid,
        collision_hw: i32,
    ) {
        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
            self.player.x_velocity,
            self.player.facing,
            h_input,
            collision,
            form,
            self.terrain.floor_terrain_type,
        );
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;
        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let probe = terrain::probe_floor_hitbox(grid, x_px, y_px, collision_hw);
        let floor_y_px = match probe.cell.collision {
            terrain::CollisionKind::Solid
            | terrain::CollisionKind::HiddenBlock
            | terrain::CollisionKind::Spikes => probe.floor_y_px.saturating_sub(1),
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::Empty => probe.floor_y_px,
        };
        let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
        self.terrain.floor_y_pos = floor_y_fp;
        self.player.y_pos = floor_y_fp;
        self.player.on_ground = true;
        self.player.y_velocity = 0;
        self.player.wall_cling_direction = None;
        self.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
    }

    /// Camera follows player using the real sub_71E4 follow algorithm.
    /// `y_look_offset` is the down-look accumulator ($FFFFFB58): when >= 30,
    /// shifts the camera target downward by that many pixels.
    pub(crate) fn update_camera_follow(&mut self, y_look_offset: i16) {
        let player_x_px = (self.player.x_pos >> 16) as i32;
        let mut player_y_px = (self.player.y_pos >> 16) as i32;
        // ASM sub_71E4 loc_728C: when offset >= $1E, add it to camera Y target
        // and subtract the reference $1E. This effectively shifts the view down.
        if y_look_offset >= 0x1E {
            player_y_px += i32::from(y_look_offset);
        }
        let camera_x_px = (self.camera.x_pos >> 16) as i32;
        let camera_y_px = (self.camera.y_pos >> 16) as i32;

        let (new_cx, new_cy) = physics::update_camera(
            camera_x_px,
            camera_y_px,
            player_x_px,
            player_y_px,
            &self.level_bounds,
        );

        self.camera.x_pos = (new_cx.max(0) as u32) << 16;
        self.camera.y_pos = (new_cy.max(0) as u32) << 16;
    }
}

pub(crate) fn update_level_one_slope_ground_velocity(
    x_vel: i32,
    facing: Direction,
    input: HorizontalInput,
    collision: terrain::CollisionKind,
    form: PlayerForm,
    terrain_type: TerrainType,
) -> (i32, Direction) {
    if form != PlayerForm::TheKid {
        return physics::update_horizontal_velocity(
            x_vel,
            facing,
            input,
            form.ground_speed_table(terrain_type),
        );
    }

    let slope_descends_right = collision == terrain::CollisionKind::SlopeDown;
    let mut canonical_vel = if slope_descends_right { x_vel } else { -x_vel };
    let uphill_pressed = if slope_descends_right {
        input.right
    } else {
        input.left
    };
    let downhill_pressed = if slope_descends_right {
        input.left
    } else {
        input.right
    };
    let (uphill_cap, uphill_accel, downhill_cap, downhill_accel) = if input.run {
        (
            KID_SLOPE_RUN_UPHILL_CAP_FP,
            KID_SLOPE_RUN_UPHILL_ACCEL_FP,
            KID_SLOPE_RUN_DOWNHILL_CAP_FP,
            KID_SLOPE_RUN_DOWNHILL_ACCEL_FP,
        )
    } else {
        (
            KID_SLOPE_WALK_UPHILL_CAP_FP,
            KID_SLOPE_WALK_UPHILL_ACCEL_FP,
            KID_SLOPE_WALK_DOWNHILL_CAP_FP,
            KID_SLOPE_WALK_DOWNHILL_ACCEL_FP,
        )
    };

    if uphill_pressed {
        if canonical_vel < 0 {
            canonical_vel += uphill_accel * 2;
        } else if canonical_vel <= uphill_cap {
            canonical_vel += uphill_accel;
            if canonical_vel > uphill_cap {
                canonical_vel = uphill_cap;
            }
        } else {
            canonical_vel -= KID_SLOPE_BRAKE_FP;
            if canonical_vel > KID_SLOPE_MAX_SPEED_FP {
                canonical_vel = KID_SLOPE_MAX_SPEED_FP;
            }
        }
    } else if downhill_pressed {
        if canonical_vel >= 0 {
            canonical_vel -= downhill_accel * 2;
        } else if canonical_vel >= downhill_cap {
            canonical_vel -= downhill_accel;
            if canonical_vel < downhill_cap {
                canonical_vel = downhill_cap;
            }
        } else {
            canonical_vel += KID_SLOPE_BRAKE_FP;
            if canonical_vel < -KID_SLOPE_MAX_SPEED_FP {
                canonical_vel = -KID_SLOPE_MAX_SPEED_FP;
            }
        }
    } else if canonical_vel < KID_SLOPE_NEUTRAL_TARGET_FP {
        canonical_vel += KID_SLOPE_NEUTRAL_DECEL_FP;
        if canonical_vel > KID_SLOPE_NEUTRAL_TARGET_FP {
            canonical_vel = KID_SLOPE_NEUTRAL_TARGET_FP;
        }
    } else {
        canonical_vel -= KID_SLOPE_NEUTRAL_DECEL_FP;
        if canonical_vel < KID_SLOPE_NEUTRAL_TARGET_FP {
            canonical_vel = KID_SLOPE_NEUTRAL_TARGET_FP;
        }
    }

    let next_facing = if uphill_pressed {
        if slope_descends_right {
            Direction::Right
        } else {
            Direction::Left
        }
    } else if downhill_pressed {
        if slope_descends_right {
            Direction::Left
        } else {
            Direction::Right
        }
    } else if slope_descends_right == (canonical_vel >= 0) {
        Direction::Right
    } else {
        Direction::Left
    };

    let next_x_vel = if slope_descends_right {
        canonical_vel
    } else {
        -canonical_vel
    };
    (next_x_vel, next_facing)
}
