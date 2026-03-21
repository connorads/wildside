use super::*;

impl NativeRuntime {
    pub(crate) fn enter_title_screen(&mut self) {
        self.boot_state = BootState::TitleScreen { cursor: 0 };
        self.title_idle_frames = 0;
        self.title_screen_elapsed_frames = 0;
        self.title_screen_ready_frame = self.frame_index.max(TITLE_SCREEN_READY_GLOBAL_FRAME);
        self.title_menu_arm_frame = self.frame_index.saturating_add(TITLE_ATTRACT_LOCK_FRAMES);
        self.title_menu_active = false;
    }

    pub(crate) fn title_screen_ready(&self) -> bool {
        self.frame_index >= self.title_screen_ready_frame
    }

    pub(crate) fn demo_startup_frame(&self) -> Option<u16> {
        match self.boot_state {
            BootState::DemoPlay { frame_counter, .. } => Some(frame_counter),
            _ => None,
        }
    }

    pub(crate) fn demo_gameplay_active(&self) -> bool {
        self.demo_startup_frame()
            .is_some_and(|frame| frame >= demo::DEMO_GAMEPLAY_START_FRAME)
    }

    pub(crate) fn demo_level_id(use_demo_2: bool) -> LevelId {
        LevelId(if use_demo_2 {
            demo::DEMO_2_LEVEL
        } else {
            demo::DEMO_1_LEVEL
        })
    }

    fn begin_demo_play(&mut self, use_demo_2: bool) {
        self.boot_state = BootState::DemoPlay {
            frame_counter: 0,
            data_index: 0,
            use_demo_2,
        };
        self.current_level = LevelId(0);
        self.level_start_frame = None;
        self.opening_slice = None;
        self.active_enemies.clear();
        self.active_enemy_roster.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.projectiles.clear();
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.progression.current_form = PlayerForm::TheKid;
        self.progression.pending_form = None;
        self.progression.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
        self.reload_block_grid_for_level(self.current_level);
    }

    pub(crate) fn begin_demo_gameplay(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.level_start_frame = Some(self.frame_index);
        self.reload_block_grid_for_level(level_id);
        let mut opening_slice = self.level_respawn_slice(level_id);
        if self.block_grid.is_some() {
            let settled_slice = self.level_respawn_slice_with_floor_snap(level_id);
            opening_slice.terrain.floor_y_pos = settled_slice.player.y_pos;
        }
        self.opening_slice = Some(opening_slice);
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.ghost_groups.clear();
        self.projectiles.clear();
        self.block_state.lift_platforms.clear();
        self.init_murder_wall(level_id, 0);
        self.init_weather(level_id);
        self.level_timer = LevelTimer::new();
        self.level_damage_taken = false;
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.y_look_offset = 0;
        self.paused = false;
        self.pause_toggle_pending = false;
    }

    pub(crate) fn apply_demo_bootstrap_initial_settle(&mut self) {
        let Some(slice) = self.opening_slice.as_mut() else {
            return;
        };
        // Load_InGame leaves the demo player one pixel above the later
        // grounded hand-off pose, then the hidden bootstrap drops that pixel.
        // on_ground=false starts the freefall phase (frames 67-69) where
        // gravity-trajectory position updates happen without touching y_vel.
        slice.player.y_pos = slice.player.y_pos.saturating_add(0x0001_0000);
        slice.player.on_ground = false;
        slice.player.y_velocity = 0;
        slice.update_camera_follow(0);
    }

    /// During frames between initial_settle and grounded_hold, the ASM's demo
    /// bootstrap applies gravity-trajectory position updates without touching
    /// the kid object's y_vel field. The kid object is hibernating — sub_B168
    /// (gravity) and sub_A4EE (position) don't run. Instead, the boot code
    /// directly adjusts y_pos with accumulated gravity: Δy = frame * 0x4000.
    pub(crate) fn apply_demo_bootstrap_freefall(&mut self, frame_counter: u16) {
        let Some(slice) = self.opening_slice.as_mut() else {
            return;
        };
        let freefall_frame = u32::from(frame_counter - demo::DEMO_BOOTSTRAP_INIT_FRAME);
        let shadow_vel = freefall_frame * 0x4000;
        slice.player.y_pos = slice.player.y_pos.wrapping_add(shadow_vel);
        // y_vel stays 0 — the kid object's velocity field is not used
        slice.player.y_velocity = 0;
        slice.update_camera_follow(0);
    }

    pub(crate) fn apply_demo_bootstrap_grounded_hold(&mut self) {
        let Some(slice) = self.opening_slice.as_mut() else {
            return;
        };
        // Oracle-backed demo traces keep the player on a short grounded
        // bootstrap hold after a 3-frame free-fall hand-off from the hidden
        // settle. Camera follow keeps converging during the hold.
        slice.player.y_pos = slice.player.y_pos.saturating_sub(slice.player.y_pos & 0x0000_FFFF);
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.player.on_spikes = false;
        slice.terrain.floor_y_pos = slice.player.y_pos;
        slice.update_camera_follow(0);
    }

    pub(crate) fn try_apply_demo_bootstrap_grounded_run(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
    ) -> bool {
        let Some(frame_counter) = self.demo_startup_frame() else {
            return false;
        };
        if frame_counter <= demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME || jump_pressed || input.b {
            return false;
        }

        let Some(slice) = self.opening_slice.as_mut() else {
            return false;
        };
        if !slice.player.on_ground || slice.player.y_velocity != 0 {
            return false;
        }

        let Some(grid) = self.block_grid.as_ref() else {
            return false;
        };
        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let hw = self.progression.current_form.half_width();
        let support_probe = terrain::probe_floor_hitbox(grid, x_px, y_px, hw);
        if support_probe.on_ground || support_probe.cell.is_solid_floor() {
            return false;
        }

        // Demo playback keeps the actor grounded on an invisible hand-off path
        // after the bootstrap hold, then regular physics resumes once the
        // recorded jump lifts off from that pinned floor.
        slice.update_horizontal_motion(input, false, self.progression.current_form, Some(grid));
        slice.apply_wall_collision(Some(grid), hw);
        slice.player.y_pos = slice.terrain.floor_y_pos;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.player.on_spikes = false;
        slice.update_camera_follow(self.y_look_offset);
        true
    }

    pub(crate) fn apply_demo_bootstrap_camera_handoff(&mut self) {
        let Some(slice) = self.opening_slice.as_mut() else {
            return;
        };
        // The first visible demo gameplay frame keeps the held bootstrap
        // player pose but lets camera follow advance once from the respawn
        // preload position before the trace is sampled.
        slice.update_camera_follow(0);
    }

    pub(crate) fn try_snap_demo_bootstrap_floor(
        &self,
        slice: &mut OpeningSliceState,
    ) -> bool {
        let Some(frame_counter) = self.demo_startup_frame() else {
            return false;
        };
        if !(demo::DEMO_BOOTSTRAP_GROUNDED_FRAME..=demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME)
            .contains(&frame_counter)
            || slice.player.y_velocity < 0
            || self.block_grid.is_none()
        {
            return false;
        }

        let settled_slice = self.level_respawn_slice_with_floor_snap(self.current_level);
        let settled_floor_y = settled_slice.player.y_pos;
        if slice.player.y_pos.abs_diff(settled_floor_y) > GROUND_SNAP_TOLERANCE_FP {
            return false;
        }

        slice.player.y_pos = settled_floor_y;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.player.on_spikes = false;
        slice.terrain.floor_y_pos = settled_floor_y;
        slice.terrain.floor_terrain_type = settled_slice.terrain.floor_terrain_type;
        true
    }

    pub(crate) fn boot_phase_label(&self) -> Option<&'static str> {
        match self.boot_state {
            BootState::SegaScreen { .. } => Some("sega_screen"),
            BootState::IntroSequence { segment: 0, .. } => Some("intro_sequence_1"),
            BootState::IntroSequence { segment: 1, .. } => Some("intro_sequence_2"),
            BootState::IntroSequence { segment: 2, .. } => Some("intro_sequence_3"),
            BootState::IntroSequence { segment: 3, .. } => Some("intro_sequence_4"),
            BootState::IntroSequence { .. } => Some("intro_sequence_5"),
            BootState::TitleScreen { .. } => Some("title_screen"),
            BootState::OptionsMenu { .. } => Some("options_menu"),
            BootState::SoundTest { .. } => Some("sound_test"),
            BootState::StartingLevel { .. } => Some("starting_level"),
            BootState::LevelSkipHold => Some("level_skip_hold"),
            BootState::DemoPlay { .. } => Some("demo_play"),
            BootState::EndingSequence { .. } => Some("ending_sequence"),
            BootState::InGame => None,
        }
    }

    pub(crate) fn boot_phase_elapsed_frames(&self) -> Option<usize> {
        match self.boot_state {
            BootState::SegaScreen { elapsed } => Some(elapsed),
            BootState::IntroSequence { elapsed, .. } => Some(elapsed),
            BootState::TitleScreen { .. }
            | BootState::OptionsMenu { .. }
            | BootState::SoundTest { .. } => {
                Some(self.title_screen_elapsed_frames)
            }
            BootState::StartingLevel { remaining_frames } => {
                Some(TITLE_TO_LEVEL_FRAMES.saturating_sub(remaining_frames))
            }
            BootState::DemoPlay { frame_counter, .. } => Some(frame_counter as usize),
            BootState::EndingSequence { elapsed } => Some(elapsed),
            BootState::LevelSkipHold | BootState::InGame => None,
        }
    }

    /// Handle demo playback input override and exit conditions.
    /// Returns `Some(input)` with the (possibly overridden) input to continue,
    /// or `None` if the frame was fully handled (early return with trace).
    pub(crate) fn step_demo_input(&mut self, input: FrameInput) -> Option<FrameInput> {
        let Some((frame_counter, data_index, use_demo_2)) = (match self.boot_state {
            BootState::DemoPlay {
                frame_counter,
                data_index,
                use_demo_2,
            } => Some((frame_counter, data_index, use_demo_2)),
            _ => None,
        }) else {
            return Some(input);
        };

        // Start button aborts demo → return to title
        if input.start && !self.previous_input.start {
            self.enter_title_screen();
            self.opening_slice = None;
            self.active_enemies.clear();
            self.active_enemy_roster.clear();
            self.active_platforms.clear();
            self.active_platform_roster.clear();
            self.player_lifecycle = PlayerLifecycle::Alive;
            self.prior_input = input;
            self.previous_input = input;
            self.frame_index += 1;
            return None;
        }
        // Auto-end after DEMO_MAX_FRAMES
        if frame_counter >= demo::DEMO_MAX_FRAMES {
            self.enter_title_screen();
            self.opening_slice = None;
            self.active_enemies.clear();
            self.active_enemy_roster.clear();
            self.active_platforms.clear();
            self.active_platform_roster.clear();
            self.player_lifecycle = PlayerLifecycle::Alive;
            self.prior_input = FrameInput::default();
            self.previous_input = FrameInput::default();
            self.frame_index += 1;
            return None;
        }
        let level_id = Self::demo_level_id(use_demo_2);
        if frame_counter == demo::DEMO_LEVEL_SWITCH_FRAME {
            self.current_level = level_id;
            self.reload_block_grid_for_level(level_id);
        }
        if frame_counter >= demo::DEMO_OBJECT_WARMUP_FRAME && self.opening_slice.is_none() {
            self.begin_demo_gameplay(level_id);
        }
        if frame_counter < demo::DEMO_GAMEPLAY_START_FRAME {
            if let BootState::DemoPlay { frame_counter, .. } = &mut self.boot_state {
                *frame_counter += 1;
            }
            return Some(FrameInput::default());
        }
        // Read demo byte and decode into input
        let data = if use_demo_2 {
            &demo::DEMO_INPUT_DATA_2[..]
        } else {
            &demo::DEMO_INPUT_DATA_1[..]
        };
        let byte = data.get(data_index).copied().unwrap_or(0);
        let demo_input = demo::decode_demo_byte(byte);
        // ASM demo input (loc_C22) freezes during Kid_Transform: the coroutine's
        // Hibernate loop doesn't call the demo input handler, so the frame counter
        // and data pointer stay frozen until the transform completes. Match this by
        // skipping advancement when form_change_flash_frames > 0.
        if self.progression.form_change_flash_frames == 0 {
            // Advance data pointer every 2 frames (ASM: `andi.w #1,d7; bne`)
            if let BootState::DemoPlay {
                frame_counter,
                data_index,
                ..
            } = &mut self.boot_state
            {
                let playback_frame = *frame_counter - demo::DEMO_GAMEPLAY_START_FRAME;
                *frame_counter += 1;
                if playback_frame % 2 == 1 {
                    *data_index += 1;
                }
            }
        }
        Some(demo_input)
    }

    /// Process Start button presses: pause/unpause, level skip, demo trigger.
    pub(crate) fn handle_start_press(&mut self, input: FrameInput, start_pressed: bool) {
        if !start_pressed {
            return;
        }
        self.start_press_count += 1;
        if matches!(self.player_lifecycle, PlayerLifecycle::Alive) {
            match self.boot_state {
                BootState::InGame => {
                    if input.a
                        && input.c
                        && (!self.level_skip_resume_hold
                            || self.level_skip_resume_elapsed_frames()
                                == Some(LEVEL_SKIP_RESUME_TOTAL_FRAMES - 1))
                    {
                        self.pending_level_skip = Some(self.next_level_id());
                    } else if input.a
                        && !input.c
                        && self.progression.current_form != PlayerForm::Eyeclops
                        && self.progression.diamond_count >= 20
                    {
                        // Diamond Power activation suppresses pause toggle
                    } else if self.paused {
                        self.paused = false;
                        self.pause_toggle_pending = false;
                    } else {
                        self.pause_toggle_pending = true;
                    }
                }
                BootState::LevelSkipHold if input.a && input.c => {
                    self.pending_level_skip_resume = true;
                }
                _ => {}
            }
        }
    }

    /// Advance the boot state machine one frame (Sega screen, intros, title,
    /// options, sound test, level transition, ending).
    pub(crate) fn step_boot_state(&mut self, input: FrameInput, start_pressed: bool) {
        match &mut self.boot_state {
            BootState::SegaScreen { elapsed } => {
                *elapsed += 1;
                if *elapsed >= SEGA_SCREEN_FRAMES {
                    self.boot_state = BootState::IntroSequence {
                        segment: 0,
                        elapsed: 0,
                        pending_title_skip: 0,
                    };
                } else if start_pressed {
                    // Defer transition to next frame: the original game's sega
                    // screen detects Start AFTER VBlank (in its WaitForVint
                    // loop), so the mode change isn't visible until the next
                    // frame's state capture.
                    *elapsed = SEGA_SCREEN_FRAMES - 1;
                }
            }
            // ASM has 5 intro sequences (Load_IntroSequence1-5) plus
            // Load_IntroSequence6 (title screen). Each loads unique VDP art.
            BootState::IntroSequence {
                segment,
                elapsed,
                pending_title_skip,
            } => {
                // Deferred title skip: the original game's intro objects
                // read Ctrl_1_Pressed (set by VBlank interrupt), causing a
                // 2-frame delay between Start press and visible mode change.
                if *pending_title_skip > 0 {
                    *pending_title_skip -= 1;
                    if *pending_title_skip == 0 {
                        self.enter_title_screen();
                        return;
                    }
                    *elapsed += 1;
                    return;
                }
                *elapsed += 1;
                let segment_frames = if *segment == 0 {
                    INTRO_FIRST_SEGMENT_FRAMES
                } else {
                    INTRO_SEGMENT_FRAMES
                };
                let start_advances_segment = start_pressed
                    && (*segment != 0 || *elapsed >= INTRO_FIRST_SEGMENT_SKIP_ADVANCE_FRAMES);
                let start_jumps_to_title = start_pressed
                    && ((*segment == 0 && *elapsed >= INTRO_FIRST_SEGMENT_DIRECT_TITLE_FRAMES)
                        || *segment >= 3);
                if start_jumps_to_title {
                    *pending_title_skip = 2;
                } else if start_advances_segment {
                    if *segment >= 4 {
                        *pending_title_skip = 2;
                    } else {
                        *segment += 1;
                        *elapsed = 0;
                    }
                } else if *elapsed >= segment_frames {
                    // Natural timeout: advance immediately (no deferral)
                    if *segment >= 4 {
                        self.enter_title_screen();
                    } else {
                        *segment += 1;
                        *elapsed = 0;
                    }
                }
            }
            BootState::TitleScreen { cursor } => {
                let up_pressed = input.up && !self.prior_input.up;
                let down_pressed = input.down && !self.prior_input.down;

                if self.frame_index <= self.title_menu_arm_frame {
                    // Oracle-backed imported TAS traces keep title fully inert for
                    // one attract-lock window after entry, even if Start was
                    // already being tapped during the intro skip path.
                } else if !self.title_menu_active {
                    if start_pressed {
                        self.title_idle_frames = 0;
                        self.title_menu_active = true;
                    } else {
                        self.title_idle_frames += 1;
                        if self.title_idle_frames >= demo::TITLE_IDLE_FRAMES {
                            self.title_idle_frames = 0;
                            self.demo_toggle = !self.demo_toggle;
                            self.begin_demo_play(self.demo_toggle);
                        }
                    }
                } else if start_pressed {
                    self.title_idle_frames = 0;
                    match *cursor {
                        0 => {
                            self.two_player = false;
                            self.begin_level_entry(self.current_level);
                        }
                        1 => {
                            self.two_player = true;
                            self.begin_level_entry(self.current_level);
                        }
                        _ => {
                            self.boot_state = BootState::OptionsMenu { cursor: 0 };
                        }
                    }
                } else if up_pressed && *cursor > 0 {
                    self.title_idle_frames = 0;
                    *cursor -= 1;
                } else if down_pressed && *cursor < 2 {
                    self.title_idle_frames = 0;
                    *cursor += 1;
                } else {
                    self.title_idle_frames += 1;
                    if self.title_idle_frames >= demo::TITLE_IDLE_FRAMES {
                        self.title_idle_frames = 0;
                        self.demo_toggle = !self.demo_toggle;
                        self.begin_demo_play(self.demo_toggle);
                    }
                }
                self.title_screen_elapsed_frames += 1;
            }
            BootState::OptionsMenu { cursor } => {
                let b_pressed = input.b && !self.prior_input.b;
                let a_pressed = input.a && !self.prior_input.a;
                let c_pressed = input.c && !self.prior_input.c;
                // Up/Down navigate cursor (3 items: 2P, SOUND TEST, EXIT)
                if input.up && !self.prior_input.up && *cursor > 0 {
                    *cursor -= 1;
                }
                if input.down && !self.prior_input.down && *cursor < 2 {
                    *cursor += 1;
                }
                let cur = *cursor;
                // A/C on "2 PLAYERS" toggles the flag
                if cur == 0 && (a_pressed || c_pressed) {
                    self.two_player = !self.two_player;
                }
                // A/C on "SOUND TEST" enters sound test screen
                if cur == 1 && (a_pressed || c_pressed) {
                    self.boot_state = BootState::SoundTest { track: 0 };
                }
                // A/C on "EXIT" or Start/B exits back to title
                else if (cur == 2 && (a_pressed || c_pressed)) || start_pressed || b_pressed {
                    self.enter_title_screen();
                }
                self.title_screen_elapsed_frames += 1;
            }
            BootState::SoundTest { track } => {
                let b_pressed = input.b && !self.prior_input.b;
                let a_pressed = input.a && !self.prior_input.a;
                let c_pressed = input.c && !self.prior_input.c;
                if input.right && !self.prior_input.right {
                    *track = track.wrapping_add(1);
                }
                if input.left && !self.prior_input.left {
                    *track = track.wrapping_sub(1);
                }
                if input.up && !self.prior_input.up {
                    *track = track.wrapping_add(16);
                }
                if input.down && !self.prior_input.down {
                    *track = track.wrapping_sub(16);
                }
                // A/C plays the current track (handled by renderer via sound_test_track)
                // Start pressed flag is set on the trace for the audio layer to detect
                if a_pressed || c_pressed {
                    self.sound_test_play_pending = true;
                }
                if b_pressed || start_pressed {
                    self.boot_state = BootState::OptionsMenu { cursor: 1 };
                }
                self.title_screen_elapsed_frames += 1;
            }
            BootState::StartingLevel { remaining_frames } => {
                if *remaining_frames > 0 {
                    *remaining_frames -= 1;
                }
                if *remaining_frames == 0 {
                    self.boot_state = BootState::InGame;
                }
            }
            BootState::LevelSkipHold => {}
            BootState::InGame => {}
            BootState::DemoPlay { .. } => {
                // Demo input feeding and frame counting handled in step_demo_input()
            }
            BootState::EndingSequence { elapsed } => {
                *elapsed += 1;
                let total_ending_frames = ENDING_PAUSE_FRAMES
                    + ENDING_MUSIC_DELAY_FRAMES
                    + ENDING_FADE_OUT_FRAMES
                    + CREDITS_SCREEN_COUNT * CREDITS_SCREEN_TOTAL;
                if *elapsed >= total_ending_frames {
                    // Return to title screen after credits
                    self.enter_title_screen();
                    self.player_lifecycle = PlayerLifecycle::Alive;
                    self.progression.number_lives = 3;
                    self.progression.number_continues = 3;
                    self.progression.diamond_count = 0;
                    self.progression.score = 0;
                    self.current_level = LevelId(0);
                    self.two_player = false;
                    self.current_player_index = 0;
                    self.player_slots = [PlayerSlot::default(), PlayerSlot::default()];
                }
            }
        }
    }

    pub(crate) fn maybe_begin_pending_level_skip(&mut self) {
        let Some(level_id) = self.pending_level_skip.take() else {
            return;
        };
        let (
            frozen_player,
            frozen_camera,
            frozen_active_objects,
            mut frozen_number_objects,
            mut frozen_number_gfx_objects,
        ) = if self.level_skip_resume_hold {
            let elapsed = self
                .level_skip_resume_elapsed_frames()
                .unwrap_or(0)
                .saturating_sub(1);
            let Some(frozen_player) = self
                .level_skip_resume_player_state_at(elapsed)
                .or(self.opening_slice.map(|slice| slice.player))
            else {
                self.enter_level_skip_hold(level_id);
                return;
            };
            let frozen_camera = self.level_skip_resume_camera_state_at(elapsed);
            let frozen_active_objects = self.level_skip_resume_active_objects_at(elapsed);
            let (frozen_number_objects, frozen_number_gfx_objects) =
                self.level_skip_resume_object_counts_at(elapsed);
            (
                frozen_player,
                frozen_camera,
                frozen_active_objects,
                frozen_number_objects,
                frozen_number_gfx_objects,
            )
        } else {
            let Some(frozen_player) = self
                .current_player_state()
                .or(self.opening_slice.map(|slice| slice.player))
            else {
                self.enter_level_skip_hold(level_id);
                return;
            };
            let frozen_camera = self.current_camera_state();
            let frozen_active_objects = self.active_objects();
            let (frozen_number_objects, frozen_number_gfx_objects) = self
                .object_counts_from_roster(
                    &frozen_active_objects,
                    frozen_camera.x_pos,
                    frozen_camera.y_pos,
                );
            (
                frozen_player,
                frozen_camera,
                frozen_active_objects,
                frozen_number_objects,
                frozen_number_gfx_objects,
            )
        };
        if self.current_level == LevelId(0) && level_id == LevelId(1) {
            frozen_number_objects = 4;
            frozen_number_gfx_objects = 5;
        }
        self.enter_level_skip_hold(level_id);
        self.level_skip_transition = Some(LevelSkipTransitionState {
            elapsed_frames: 0,
            frozen_player,
            frozen_camera,
            frozen_active_objects,
            frozen_number_objects,
            frozen_number_gfx_objects,
        });
    }

    pub(crate) fn maybe_begin_pending_level_skip_resume(&mut self) {
        if !self.pending_level_skip_resume {
            return;
        }
        self.pending_level_skip_resume = false;
        self.boot_state = BootState::InGame;
        self.level_start_frame = Some(self.frame_index);
        self.level_skip_transition = None;
        self.level_skip_resume_hold = true;
        self.opening_slice = Some(self.level_respawn_slice(self.current_level));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.ghost_groups.clear();
        self.level_timer = LevelTimer::new();
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.paused = false;
        self.pause_toggle_pending = false;
    }

    pub(crate) fn maybe_finish_level_skip_resume(&mut self) {
        if self.level_skip_resume_elapsed_frames() != Some(LEVEL_SKIP_RESUME_TOTAL_FRAMES) {
            return;
        }
        self.level_skip_resume_hold = false;
        self.skip_entry_animation = true;
        self.level_start_frame = Some(self.frame_index);
        self.opening_slice = Some(self.level_skip_resume_final_slice(self.current_level));
    }

    /// Returns true if the frame should continue to physics/gameplay processing.
    /// Returns false for early-return cases (non-alive lifecycle, paused, etc.).
    pub(crate) fn step_lifecycle(&mut self, input: FrameInput) -> bool {
        if matches!(self.boot_state, BootState::EndingSequence { .. }) {
            self.opening_slice = None;
            return false;
        }
        if !matches!(
            self.boot_state,
            BootState::StartingLevel { .. }
                | BootState::LevelSkipHold
                | BootState::InGame
                | BootState::DemoPlay { .. }
        ) {
            self.opening_slice = None;
            self.progression.current_form = PlayerForm::TheKid;
            self.progression.pending_form = None;
            self.progression.number_lives = 3;
            self.progression.number_continues = 3;
            self.progression.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
            self.progression.invulnerable_frames = 0;
            self.paused = false;
            self.pause_toggle_pending = false;
            self.player_lifecycle = PlayerLifecycle::Alive;
            self.death_camera_lock = None;
            return false;
        }

        match self.player_lifecycle {
            PlayerLifecycle::Alive => {
                if self.progression.invulnerable_frames > 0 {
                    self.progression.invulnerable_frames -= 1;
                }
                if self.progression.form_change_flash_frames > 0 {
                    self.progression.form_change_flash_frames -= 1;
                }
                self.resolve_pending_form_change();
            }
            PlayerLifecycle::DeathWindup { remaining_frames } => {
                self.advance_death_windup_motion();
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::DeathWindup {
                        remaining_frames: remaining_frames - 1,
                    };
                } else {
                    self.enter_death_hidden_state();
                }
                return false;
            }
            PlayerLifecycle::Dying { remaining_frames } => {
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::Dying {
                        remaining_frames: remaining_frames - 1,
                    };
                } else if self.progression.number_lives == 0 {
                    self.player_lifecycle = PlayerLifecycle::GameOver {
                        remaining_frames: PLAYER_GAME_OVER_HOLD_FRAMES,
                    };
                } else if self.try_switch_player() {
                    // 2P: both have lives, switch turns
                    self.player_lifecycle = PlayerLifecycle::Alive;
                } else {
                    self.reload_block_grid_for_level(self.current_level);
                    self.opening_slice =
                        Some(self.level_respawn_slice_with_floor_snap(self.current_level));
                    self.level_timer = LevelTimer::new();
                    self.level_prize_collected = false;
                    self.clocks_collected = 0;
                    self.player_lifecycle = PlayerLifecycle::Respawning {
                        remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES,
                    };
                }
                return false;
            }
            PlayerLifecycle::Respawning { remaining_frames } => {
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::Respawning {
                        remaining_frames: remaining_frames - 1,
                    };
                } else {
                    self.player_lifecycle = PlayerLifecycle::Alive;
                    self.progression.invulnerable_frames = PLAYER_DAMAGE_INVULNERABLE_FRAMES;
                }
                return false;
            }
            PlayerLifecycle::LevelComplete { remaining_frames } => {
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::LevelComplete {
                        remaining_frames: remaining_frames - 1,
                    };
                } else {
                    self.advance_level();
                }
                return false;
            }
            PlayerLifecycle::GameOver { remaining_frames } => {
                // ASM: "GAME OVER" display, then transition to continue screen or title.
                if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::GameOver {
                        remaining_frames: remaining_frames - 1,
                    };
                } else if self.progression.number_continues > 0 {
                    // Transition to continue countdown screen
                    self.player_lifecycle = PlayerLifecycle::ContinueCountdown {
                        remaining_frames: CONTINUE_COUNTDOWN_FRAMES,
                    };
                } else if self.try_switch_player() {
                    self.player_lifecycle = PlayerLifecycle::Alive;
                } else {
                    self.return_to_title();
                }
                return false;
            }
            PlayerLifecycle::ContinueCountdown { remaining_frames } => {
                // ASM Continue_Screen: button press uses continue, timeout goes to title.
                let any_button = input.a || input.b || input.c || input.start;
                if any_button {
                    // Use a continue: restore lives and respawn at current level.
                    // ASM Use_Continue: clears helmet, diamonds, resets to 3 lives / 2 HP.
                    self.progression.number_continues -= 1;
                    self.progression.number_lives = 3;
                    self.progression.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
                    self.progression.current_form = PlayerForm::TheKid;
                    self.progression.pending_form = None;
                    self.progression.diamond_count = 0;
                    self.reload_block_grid_for_level(self.current_level);
                    self.opening_slice =
                        Some(self.level_respawn_slice_with_floor_snap(self.current_level));
                    self.level_timer = LevelTimer::new();
                    self.level_prize_collected = false;
                    self.clocks_collected = 0;
                    self.player_lifecycle = PlayerLifecycle::Respawning {
                        remaining_frames: PLAYER_DEATH_RESPAWN_SETTLE_FRAMES,
                    };
                } else if remaining_frames > 1 {
                    self.player_lifecycle = PlayerLifecycle::ContinueCountdown {
                        remaining_frames: remaining_frames - 1,
                    };
                } else {
                    // Countdown expired without input — return to title
                    self.return_to_title();
                }
                return false;
            }
        }

        if matches!(self.boot_state, BootState::LevelSkipHold) {
            return false;
        }

        if self.level_skip_resume_hold {
            return false;
        }

        if self.paused {
            return false;
        }

        true
    }
}
