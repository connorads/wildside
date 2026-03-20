use super::*;

impl PlayerState {
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
            pre_gravity_y_vel: 0,
            physics_trace: None,
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
            pre_gravity_y_vel: 0,
            physics_trace: None,
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

        match physics_order {
            PhysicsOrder::GravityFirst => {
                let on_ground_before = self.player.on_ground;
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
                    self.apply_wall_collision(Some(grid));
                    if self.player.x_pos != pre_collision_x
                        && let Some(ref mut t) = self.physics_trace
                    {
                        t.wall_hit = true;
                        t.wall_snap_x = Some(self.player.x_pos);
                    }
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid) {
                        self.finalise_trace();
                        // ASM camera follow runs after all physics sub-steps. On the
                        // ledge-snap frame, the camera follow sees the pre-snap
                        // position — skip the follow here and let the next frame
                        // catch up, matching the oracle's one-frame camera lag.
                        return;
                    }
                    self.apply_ceiling_collision(Some(grid), block_list);
                    self.apply_floor_collision(Some(grid), horizontal_step_applied, !jump_pressed)
                } else {
                    self.apply_floor_collision(None, horizontal_step_applied, !jump_pressed)
                };
                if let (Some(grid), Some(collision)) = (block_grid.as_deref(), airborne_slope_recontact) {
                    self.apply_immediate_slope_recontact(input, form, collision, grid);
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
                    self.apply_wall_collision(Some(grid));
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid) {
                        self.finalise_trace();
                        // ASM camera follow runs after all physics sub-steps. On the
                        // ledge-snap frame, the camera follow sees the pre-snap
                        // position — skip the follow here and let the next frame
                        // catch up, matching the oracle's one-frame camera lag.
                        return;
                    }
                    self.apply_ceiling_collision(Some(grid), block_list);
                    self.apply_floor_collision(Some(grid), horizontal_step_applied, !jump_pressed)
                } else {
                    self.apply_floor_collision(None, horizontal_step_applied, !jump_pressed)
                };
                if let (Some(grid), Some(collision)) = (block_grid.as_deref(), airborne_slope_recontact) {
                    self.apply_immediate_slope_recontact(input, form, collision, grid);
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
                let on_ground_before = self.player.on_ground;
                if let Some(ref mut t) = self.physics_trace {
                    t.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.on_ground_before = on_ground_before;
                }

                // Step 1: sub_B084 — horizontal velocity only (no x_pos update)
                let horizontal_step_applied =
                    self.update_horizontal_velocity_only(input, jump_pressed, form, block_grid.as_deref());

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

                // Step 3: sub_A4EE — apply velocity to position
                self.apply_velocity_to_position();

                if let Some(ref mut t) = self.physics_trace {
                    t.post_vertical = (self.player.y_pos, self.player.y_velocity);
                }

                // Step 4: sub_902A + sub_8F26 — collisions
                let pre_collision_x = self.player.x_pos;
                let airborne_slope_recontact = if let Some(grid) = block_grid.as_deref_mut() {
                    self.apply_wall_collision(Some(grid));
                    if self.player.x_pos != pre_collision_x
                        && let Some(ref mut t) = self.physics_trace
                    {
                        t.wall_hit = true;
                        t.wall_snap_x = Some(self.player.x_pos);
                    }
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid) {
                        self.finalise_trace();
                        // ASM camera follow runs after all physics sub-steps. On the
                        // ledge-snap frame, the camera follow sees the pre-snap
                        // position — skip the follow here and let the next frame
                        // catch up, matching the oracle's one-frame camera lag.
                        return;
                    }
                    self.apply_ceiling_collision(Some(grid), block_list);
                    self.apply_floor_collision(Some(grid), horizontal_step_applied, !jump_pressed)
                } else {
                    self.apply_floor_collision(None, horizontal_step_applied, !jump_pressed)
                };
                if let (Some(grid), Some(collision)) = (block_grid.as_deref(), airborne_slope_recontact) {
                    self.apply_immediate_slope_recontact(input, form, collision, grid);
                }
            }
        }
        self.finalise_trace();
        self.update_camera_follow(y_look_offset);
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
                // Determine which ASM landing path was taken
                t.landing_path = Some(if self.pre_gravity_y_vel >= 0 {
                    LandingPath::PreserveSubPixel
                } else {
                    LandingPath::ZeroSubPixel
                });
            }
        }
    }

    pub(crate) fn try_snap_wall_sticky_ledge(
        &mut self,
        jump_pressed: bool,
        grid: &terrain::BlockGrid,
    ) -> bool {
        if !jump_pressed || self.player.on_ground || self.player.y_velocity >= 0 {
            return false;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let (probe, snap_x_px) = if self.player.right_wall_sticky {
            let probe = terrain::probe_floor(grid, x_px + PLAYER_HALF_WIDTH + 1, y_px);
            let support_left_px = i32::from(probe.cell_x) * 16;
            (probe, support_left_px + PLAYER_HALF_WIDTH - 1)
        } else if self.player.left_wall_sticky {
            let probe = terrain::probe_floor(grid, x_px - PLAYER_HALF_WIDTH - 1, y_px);
            let support_right_px = (i32::from(probe.cell_x) + 1) * 16 - 1;
            (probe, support_right_px - PLAYER_HALF_WIDTH + 1)
        } else {
            return false;
        };

        if !probe.cell.is_solid_floor() {
            return false;
        }

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
        if self.player.y_pos < contact_y_fp
            || self.player.y_pos > contact_y_fp.saturating_add(15u32 << 16)
        {
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
        // Jump initiation
        if self.player.on_ground && self.player.y_velocity >= 0 && jump_pressed {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
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
            return;
        }

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
    }

    /// Apply velocity to position (sub_A4EE equivalent).
    /// x_pos += x_vel, y_pos += y_vel.
    pub(crate) fn apply_velocity_to_position(&mut self) {
        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
        let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
        self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
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

    pub(crate) fn apply_wall_collision(&mut self, block_grid: Option<&terrain::BlockGrid>) {
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
        // Suppressed when wall_sticky is set (player just hit a wall — ASM's
        // loc_AA22 wall-cling prevents acceleration in this case, but native
        // doesn't implement the cling yet so we must re-probe).
        let wall_stuck = (moving_right && self.player.right_wall_sticky)
            || (moving_left && self.player.left_wall_sticky);
        if !wall_stuck {
            let old_x_pos = self.player.x_pos.wrapping_sub(self.player.x_velocity as u32);
            let old_px = (old_x_pos >> 16) as i32;
            let new_px = (self.player.x_pos >> 16) as i32;
            if moving_right {
                let displacement = new_px - old_px;
                let right_edge = old_px + PLAYER_HALF_WIDTH;
                let distance_to_tile = (-(right_edge + 1)) & 0xF;
                if distance_to_tile > 0 && displacement <= distance_to_tile {
                    return;
                }
            } else {
                let displacement = old_px - new_px;
                let left_edge = old_px - PLAYER_HALF_WIDTH;
                let distance_to_tile = left_edge & 0xF;
                if distance_to_tile > 0 && displacement <= distance_to_tile {
                    return;
                }
            }
        }

        let mut probe = terrain::probe_wall(
            grid,
            x_px,
            y_px,
            PLAYER_HALF_WIDTH,
            PLAYER_HEIGHT,
            moving_right,
        );

        if !probe.hit && !self.player.on_ground {
            let top_edge_probe = terrain::probe_wall_top_edge(
                grid,
                x_px,
                y_px,
                PLAYER_HALF_WIDTH,
                PLAYER_HEIGHT,
                moving_right,
            );
            if top_edge_probe.hit {
                probe = top_edge_probe;
            }
        }

        // Right wall top-edge retention (overhang sticky)
        if !probe.hit && moving_right && self.player.right_wall_sticky {
            let top_edge_probe = terrain::probe_wall_top_edge(
                grid,
                x_px,
                y_px,
                PLAYER_HALF_WIDTH,
                PLAYER_HEIGHT,
                true,
            );
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
            let top_edge_probe = terrain::probe_wall_top_edge(
                grid,
                x_px,
                y_px,
                PLAYER_HALF_WIDTH,
                PLAYER_HEIGHT,
                false,
            );
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
                probe.wall_x_px - PLAYER_RIGHT_WALL_EXTENT
            } else {
                probe.wall_x_px + PLAYER_LEFT_WALL_EXTENT
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
        } else {
            self.player.right_wall_sticky = false;
            self.player.left_wall_sticky = false;
        }
    }

    pub(crate) fn apply_ceiling_collision(
        &mut self,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
    ) {
        let Some(grid) = block_grid else { return };
        if self.player.y_velocity >= 0 {
            return;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;

        let probe = terrain::probe_ceiling(grid, x_px, y_px, PLAYER_HALF_WIDTH, PLAYER_HEIGHT);

        if probe.hit {
            // Hidden block reveal: headbutting a hidden block from below
            // changes it to solid, making the block visible and functional.
            if probe.cell.collision == terrain::CollisionKind::HiddenBlock {
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
                    // Clear the now-revealed prize block
                    grid.set_cell_at(
                        probe.cell_x,
                        probe.cell_y,
                        terrain::Cell::from_be_bytes(0, 0),
                    );
                }
            }

            // Prize/smash/lift block headbutt: hitting from below
            if probe.cell.collision == terrain::CollisionKind::Solid && probe.cell.has_block {
                match probe.cell.block_type {
                    terrain::BlockType::Prize => {
                        let content = block_list
                            .and_then(|bl| bl.prize_content_at(probe.cell_x, probe.cell_y))
                            .unwrap_or(terrain::PrizeContent::Diamond);
                        let block_x_px = probe.cell_x * 16;
                        let block_y_px = probe.cell_y * 16;
                        self.prize_block_hit = Some((content, block_x_px, block_y_px));
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0, 0),
                        );
                    }
                    terrain::BlockType::Smash => {
                        let block_x_px = probe.cell_x * 16;
                        let block_y_px = probe.cell_y * 16;
                        self.smash_block_hit = Some((block_x_px, block_y_px));
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
                        // Drill block fires a downward projectile when headbutted
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
            let new_y = probe.ceiling_y_px + PLAYER_HEIGHT;
            self.player.y_pos = (new_y.max(0) as u32) << 16;
            // Small downward nudge on ceiling hit (original game pushes player down)
            self.player.y_velocity = 0x0001_0000;
        }
    }

    pub(crate) fn apply_floor_collision(
        &mut self,
        block_grid: Option<&terrain::BlockGrid>,
        horizontal_step_applied: bool,
        allow_upward_slope_snap: bool,
    ) -> Option<terrain::CollisionKind> {
        if let Some(grid) = block_grid {
            let was_on_ground = self.player.on_ground;
            let x_px = (self.player.x_pos >> 16) as i32;
            let y_px = (self.player.y_pos >> 16) as i32;
            let left_edge_probe = terrain::probe_floor(grid, x_px - PLAYER_HALF_WIDTH, y_px);
            let right_edge_probe = terrain::probe_floor(grid, x_px + PLAYER_HALF_WIDTH, y_px);
            let mut probe = terrain::probe_floor_hitbox(grid, x_px, y_px, PLAYER_HALF_WIDTH);
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
                        let within_solid_touch_snap =
                            self.player.y_pos.abs_diff(floor_y_fp) < 0x0000_8000;
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
                    // ASM floor landing: `tst.l y_vel(a3)` in Character_CheckCollision
                    // (before sub_B168 gravity). loc_AAF8 (y_vel >= 0) preserves sub-pixel
                    // via `move.w d7,y_pos(a3)`. loc_AB22 (y_vel < 0) zeroes it via
                    // `clr.w $20(a3)`. Use pre_gravity_y_vel since native applies gravity
                    // before collisions (GravityFirst), but ASM tests before gravity.
                    if self.pre_gravity_y_vel >= 0 {
                        self.player.y_pos =
                            (floor_y_fp & 0xFFFF_0000) | (self.player.y_pos & 0x0000_FFFF);
                    } else {
                        self.player.y_pos = floor_y_fp;
                    }
                    // Mushroom blocks bounce the player back up
                    if probe.cell.block_type == terrain::BlockType::Mushroom {
                        self.player.y_velocity =
                            -(self.player.y_velocity.abs() * 3 / 4).max(MUSHROOM_MIN_BOUNCE);
                        self.player.on_ground = false;
                    } else {
                        self.player.on_ground = true;
                        self.player.y_velocity = 0;
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
        let probe = terrain::probe_floor_hitbox(grid, x_px, y_px, PLAYER_HALF_WIDTH);
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
