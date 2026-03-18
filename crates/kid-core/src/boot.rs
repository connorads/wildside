use super::*;

impl NativeRuntime {
    /// Handle demo playback input override and exit conditions.
    /// Returns `Some(input)` with the (possibly overridden) input to continue,
    /// or `None` if the frame was fully handled (early return with trace).
    pub(crate) fn step_demo_input(
        &mut self,
        input: FrameInput,
    ) -> Option<FrameInput> {
        let BootState::DemoPlay {
            frame_counter,
            data_index,
            use_demo_2,
        } = &mut self.boot_state
        else {
            return Some(input);
        };

        // Start button aborts demo → return to title
        if input.start && !self.previous_input.start {
            self.boot_state = BootState::TitleScreen;
            self.title_idle_frames = 0;
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
        if *frame_counter >= demo::DEMO_MAX_FRAMES {
            self.boot_state = BootState::TitleScreen;
            self.title_idle_frames = 0;
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
        // Read demo byte and decode into input
        let data = if *use_demo_2 {
            &demo::DEMO_INPUT_DATA_2[..]
        } else {
            &demo::DEMO_INPUT_DATA_1[..]
        };
        let byte = data.get(*data_index).copied().unwrap_or(0);
        let demo_input = demo::decode_demo_byte(byte);
        // Advance data pointer every 2 frames (ASM: `andi.w #1,d7; bne`)
        *frame_counter += 1;
        if *frame_counter % 2 == 0 {
            *data_index += 1;
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
                        && self.current_form != PlayerForm::Eyeclops
                        && self.diamond_count >= 20
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
                if *elapsed >= SEGA_SCREEN_FRAMES || start_pressed {
                    self.boot_state = BootState::IntroSequence {
                        segment: 0,
                        elapsed: 0,
                    };
                }
            }
            // ASM has 5 intro sequences (Load_IntroSequence1-5) plus
            // Load_IntroSequence6 (title screen). Each loads unique VDP art.
            BootState::IntroSequence { segment, elapsed } => {
                *elapsed += 1;
                if start_pressed || *elapsed >= INTRO_SEGMENT_FRAMES {
                    if *segment >= 4 {
                        self.title_idle_frames = 0;
                        self.boot_state = BootState::TitleScreen;
                    } else {
                        *segment += 1;
                        *elapsed = 0;
                    }
                }
            }
            BootState::TitleScreen => {
                if start_pressed && self.start_press_count >= BOOTSTRAP_LEVEL_START_PRESS_COUNT {
                    self.title_idle_frames = 0;
                    self.begin_level_entry(self.current_level);
                }
                // B button opens options menu
                else if input.b && !self.prior_input.b {
                    self.title_idle_frames = 0;
                    self.boot_state = BootState::OptionsMenu { cursor: 0 };
                } else if input == FrameInput::default() {
                    // No buttons pressed — count idle frames for demo trigger
                    self.title_idle_frames += 1;
                    if self.title_idle_frames >= demo::TITLE_IDLE_FRAMES {
                        self.title_idle_frames = 0;
                        self.demo_toggle = !self.demo_toggle;
                        let use_demo_2 = self.demo_toggle;
                        let level = if use_demo_2 {
                            demo::DEMO_2_LEVEL
                        } else {
                            demo::DEMO_1_LEVEL
                        };
                        self.begin_level_entry(LevelId(level));
                        self.boot_state = BootState::DemoPlay {
                            frame_counter: 0,
                            data_index: 0,
                            use_demo_2,
                        };
                    }
                } else {
                    self.title_idle_frames = 0;
                }
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
                    self.title_idle_frames = 0;
                    self.boot_state = BootState::TitleScreen;
                }
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
                    self.boot_state = BootState::TitleScreen;
                    self.title_idle_frames = 0;
                    self.player_lifecycle = PlayerLifecycle::Alive;
                    self.number_lives = 3;
                    self.number_continues = 3;
                    self.diamond_count = 0;
                    self.score = 0;
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
        self.evanescent_blocks.clear();
        self.ghost_groups.clear();
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
}
