mod blocks;
mod boot;
mod combat;
pub(crate) mod constants;
mod demo;
pub(crate) mod enemies;
mod helmets;
mod level;
mod movement;
pub mod physics;
mod platforms;
mod projectiles;
mod scenario;
pub mod terrain;
mod trace;
pub(crate) mod types;
mod weather;

use constants::*;
use enemies::*;
use kid_data::{
    AssetManifest, EnemyLayout, EnemyLayoutObject, PlatformLayout, PlatformPlacement,
    resource_level_record,
};
use physics::{Direction, HorizontalInput, LevelBounds};
use platforms::*;
pub use scenario::{
    run_script, run_script_with_physics_order, run_script_with_trace, run_script_with_warp,
};
pub const BOOT_TO_LEVEL_START_FRAMES: [usize; 7] = constants::BOOT_TO_LEVEL_START_FRAMES;
use std::fs;
#[cfg(test)]
pub(crate) use trace::opening_level_one_object_counts;
pub use types::*;

#[derive(Debug, Clone)]
pub struct NativeRuntime {
    assets: AssetManifest,
    frame_index: usize,
    boot_state: BootState,
    prior_input: FrameInput,
    previous_input: FrameInput,
    start_press_count: usize,
    current_level: LevelId,
    level_start_frame: Option<usize>,
    progression: PlayerProgression,
    helmet: HelmetState,
    paused: bool,
    pause_toggle_pending: bool,
    pending_level_skip: Option<LevelId>,
    pending_level_skip_resume: bool,
    player_lifecycle: PlayerLifecycle,
    opening_slice: Option<OpeningSliceState>,
    death_camera_lock: Option<CameraState>,
    level_skip_transition: Option<LevelSkipTransitionState>,
    level_skip_resume_hold: bool,
    /// When true, skip the entry animation on the current level (used after
    /// level-skip warp — the original game has no entry animation).
    skip_entry_animation: bool,
    active_enemy_roster: Vec<NativeObjectSnapshot>,
    active_enemies: Vec<ActiveEnemy>,
    active_platforms: Vec<ActivePlatform>,
    active_platform_roster: Vec<NativeObjectSnapshot>,
    platform_script_table: Option<kid_data::PlatformScriptTable>,
    block_grid: Option<terrain::BlockGrid>,
    block_list: Option<terrain::BlockList>,
    block_grid_source: BlockGridSource,
    /// Player has beaten the final level — show victory/ending.
    game_complete: bool,
    block_state: BlockInteractionState,
    projectiles: Vec<EnemyProjectile>,
    weather: WeatherState,
    murder_wall: Option<MurderWallState>,
    /// Metal block bounce animations in progress.
    block_bounces: Vec<BlockBounceAnim>,
    /// Block destruction fragment effects.
    block_destroy_effects: Vec<BlockDestroyEffect>,
    /// Floating prize items rising from blocks.
    floating_items: Vec<FloatingItem>,
    /// Prize diamonds award their count later via PrizeDiamondCollected_Init,
    /// not on the touch frame.
    pending_prize_diamond_awards: Vec<u8>,
    /// Enemy death particle effects.
    enemy_death_effects: Vec<EnemyDeathEffect>,
    /// Set each frame if a stomp occurred.
    stomped_this_frame: bool,
    /// Count of blocks hit by headbutt this frame.
    blocks_hit_this_frame: u8,
    /// Sound test play pending (consumed each frame).
    sound_test_play_pending: bool,
    level_timer: LevelTimer,
    /// Camera Y look-ahead accumulator ($FFFFFB58): +1 per frame while down held
    /// and grounded, -8 per frame on release, clamped to [0, 30].
    y_look_offset: i16,
    /// Set to true when player takes damage during current level (for no-hit bonus).
    level_damage_taken: bool,
    /// Set to true when player collects any prize from a block (disqualifies no-prize bonus).
    level_prize_collected: bool,
    /// Number of clock pickups collected during current level (for completion time calc).
    clocks_collected: u16,
    /// Title screen phase timer: counts down to demo playback in both the
    /// pre-menu attract phase and the interactive title menu. The interactive
    /// loop resets this on Up/Down navigation, matching the ASM.
    title_idle_frames: u16,
    /// Title screen local elapsed frames, used for diagnostics and demo timing.
    title_screen_elapsed_frames: usize,
    /// First global boot frame where the title pose becomes visible in trace output.
    title_screen_ready_frame: usize,
    /// Last global boot frame of the initial title attract lock.
    title_menu_arm_frame: usize,
    /// False during the initial title attract wait where only Start matters.
    /// True once the Start press arms the real interactive title menu.
    title_menu_active: bool,
    /// Demo playback reads inputs after game logic (ASM ReadJoypad path), so
    /// the byte fetched this frame becomes the controller state for the next
    /// frame.
    demo_buffered_input: Option<FrameInput>,
    /// Demo Start abort uses a hibernating object coroutine (`loc_119D8`):
    /// the live Start press arms the return, then title only becomes visible
    /// two frames later when that object wakes and flips Game_Mode.
    pending_demo_title_return: u8,
    /// Alternates between demo 1 and demo 2 (ASM $FFFFFBC8 toggle).
    demo_toggle: bool,
    /// Tracks whether demo playback is currently inside Kid_Transform so the
    /// post-transform demo counter parity can be restored on the exit frame.
    demo_transform_active: bool,
    /// Two-player alternating mode enabled.
    two_player: bool,
    /// Current player index (0=P1, 1=P2). Mirrors ASM $FFFFFC39.
    current_player_index: u8,
    /// Per-player persistent state slots.
    player_slots: [PlayerSlot; 2],
    /// Physics sub-step ordering (default: GravityFirst = current native order).
    physics_order: PhysicsOrder,
    /// When true, populate `PhysicsTrace` on each frame for sub-step debugging.
    trace_physics: bool,
}

impl NativeRuntime {
    pub fn new(assets: AssetManifest) -> Self {
        let mut runtime = Self {
            assets,
            frame_index: 0,
            boot_state: BootState::SegaScreen { elapsed: 0 },
            prior_input: FrameInput::default(),
            previous_input: FrameInput::default(),
            start_press_count: 0,
            current_level: LevelId(0),
            level_start_frame: None,
            progression: PlayerProgression::default(),
            helmet: HelmetState::default(),
            paused: false,
            pause_toggle_pending: false,
            pending_level_skip: None,
            pending_level_skip_resume: false,
            player_lifecycle: PlayerLifecycle::Alive,
            opening_slice: None,
            death_camera_lock: None,
            level_skip_transition: None,
            level_skip_resume_hold: false,
            skip_entry_animation: false,
            active_enemy_roster: Vec::new(),
            active_enemies: Vec::new(),
            active_platforms: Vec::new(),
            active_platform_roster: Vec::new(),
            platform_script_table: None,
            block_grid: None,
            block_list: None,
            block_grid_source: BlockGridSource::None,
            game_complete: false,
            block_state: BlockInteractionState::default(),
            projectiles: Vec::new(),
            weather: WeatherState::default(),
            murder_wall: None,
            block_bounces: Vec::new(),
            block_destroy_effects: Vec::new(),
            floating_items: Vec::new(),
            pending_prize_diamond_awards: Vec::new(),
            enemy_death_effects: Vec::new(),
            stomped_this_frame: false,
            blocks_hit_this_frame: 0,
            sound_test_play_pending: false,
            level_timer: LevelTimer::new(),
            y_look_offset: 0,
            level_damage_taken: false,
            level_prize_collected: false,
            clocks_collected: 0,
            title_idle_frames: 0,
            title_screen_elapsed_frames: 0,
            title_screen_ready_frame: TITLE_SCREEN_READY_GLOBAL_FRAME,
            title_menu_arm_frame: 0,
            title_menu_active: false,
            demo_buffered_input: None,
            pending_demo_title_return: 0,
            demo_toggle: false,
            demo_transform_active: false,
            two_player: false,
            current_player_index: 0,
            player_slots: [PlayerSlot::default(), PlayerSlot::default()],
            physics_order: PhysicsOrder::default(),
            trace_physics: false,
        };
        runtime.reload_block_grid_for_level(LevelId(0));
        runtime.load_platform_script_table();
        runtime
    }

    fn load_platform_script_table(&mut self) {
        let scripts_path = self.assets.source_root.join("level/platform_scripts.asm");
        if let Ok(contents) = std::fs::read_to_string(&scripts_path) {
            self.platform_script_table = Some(kid_data::parse_platform_scripts(&contents));
        }
    }

    pub fn with_generated_level_layout(assets: AssetManifest) -> Self {
        Self::new(assets)
    }

    /// Override the physics sub-step ordering (for sweep comparison experiments).
    pub fn set_physics_order(&mut self, order: PhysicsOrder) {
        self.physics_order = order;
    }

    /// Enable sub-frame physics tracing for TAS debugging.
    pub fn set_trace_physics(&mut self, enabled: bool) {
        self.trace_physics = enabled;
    }

    /// Set the block collision grid for the current level.
    /// Call after constructing the runtime, before stepping frames.
    pub fn set_block_grid(&mut self, grid: terrain::BlockGrid) {
        self.block_grid = Some(grid);
        self.block_grid_source = BlockGridSource::Manual;
    }

    pub fn block_grid(&self) -> Option<&terrain::BlockGrid> {
        self.block_grid.as_ref()
    }

    pub fn step_frame(&mut self, input: FrameInput) -> NativeTraceFrame {
        let Some(input) = self.step_demo_input(input) else {
            return self.trace_frame();
        };
        self.maybe_begin_pending_level_skip();
        self.maybe_begin_pending_level_skip_resume();
        self.maybe_finish_level_skip_resume();
        let start_pressed = input.start && !self.previous_input.start;
        let apply_pending_pause_toggle = self.pause_toggle_pending;
        self.pause_toggle_pending = false;
        self.handle_start_press(input, start_pressed);
        self.step_boot_state(input, start_pressed);
        if let BootState::DemoPlay { use_demo_2, .. } = self.boot_state
            && self.demo_gameplay_active()
            && self.opening_slice.is_none()
        {
            self.begin_demo_gameplay(NativeRuntime::demo_level_id(use_demo_2));
        }

        self.refresh_active_roster();
        self.step_pending_prize_diamond_awards();
        let gameplay_slice_active = matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame
        ) || (self.demo_gameplay_active()
            && self.opening_slice.is_some());
        // Diamond Power uses raw input (Start+A held) since gameplay_input delays A by 1 frame
        if matches!(self.player_lifecycle, PlayerLifecycle::Alive)
            && gameplay_slice_active
            && !self.paused
        {
            self.step_diamond_power(input);
        }
        if gameplay_slice_active {
            let demo_live_input = matches!(self.boot_state, BootState::DemoPlay { .. })
                && self
                    .demo_startup_frame()
                    .is_some_and(|frame| frame > demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME);
            let gameplay_input = if demo_live_input {
                input
            } else {
                self.gameplay_input(input)
            };
            let jump_pressed = if demo_live_input {
                input.b && !self.previous_input.b
            } else {
                self.gameplay_jump_pressed()
            };
            match self.demo_startup_frame() {
                Some(frame) if frame == demo::DEMO_GAMEPLAY_START_FRAME => {
                    self.apply_demo_bootstrap_camera_handoff();
                }
                Some(frame) if frame == demo::DEMO_BOOTSTRAP_INIT_FRAME => {
                    self.apply_demo_bootstrap_initial_settle();
                }
                Some(frame)
                    if frame > demo::DEMO_BOOTSTRAP_INIT_FRAME
                        && frame < demo::DEMO_BOOTSTRAP_GROUNDED_FRAME =>
                {
                    self.apply_demo_bootstrap_freefall(frame);
                }
                Some(frame)
                    if (demo::DEMO_BOOTSTRAP_GROUNDED_FRAME
                        ..=demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME)
                        .contains(&frame) =>
                {
                    self.apply_demo_bootstrap_grounded_hold();
                }
                _ => {
                    if !self.try_apply_demo_bootstrap_grounded_run(gameplay_input, jump_pressed) {
                        self.refresh_opening_slice(gameplay_input, jump_pressed);
                    }
                }
            }
        }
        if apply_pending_pause_toggle {
            self.paused = !self.paused;
        }

        let trace = self.trace_frame();
        self.prior_input = self.previous_input;
        self.previous_input = input;
        if matches!(self.boot_state, BootState::LevelSkipHold)
            && let Some(transition) = self.level_skip_transition.as_mut()
        {
            transition.elapsed_frames += 1;
        }
        self.frame_index += 1;
        trace
    }

    fn level_elapsed_frames(&self) -> Option<usize> {
        self.level_start_frame
            .map(|frame| self.frame_index.saturating_sub(frame))
    }

    fn gameplay_input(&self, input: FrameInput) -> FrameInput {
        FrameInput {
            up: self.previous_input.up,
            down: self.previous_input.down,
            left: self.previous_input.left,
            right: self.previous_input.right,
            a: self.previous_input.a,
            // Gameplay motion consumes the previous controller state, including jump hold.
            b: self.previous_input.b,
            c: input.c,
            start: input.start,
        }
    }

    fn gameplay_jump_pressed(&self) -> bool {
        self.previous_input.b && !self.prior_input.b
    }

    fn update_demo_transform_parity(&mut self) {
        let transform_active = self.progression.form_change_flash_frames > 0;
        if let BootState::DemoPlay {
            frame_counter,
            data_index,
            use_demo_2,
            ..
        } = &mut self.boot_state
        {
            if self.demo_transform_active && !transform_active {
                // During Kid_Transform, step_demo_input freezes frame_counter.
                // When the coroutine exits, native would otherwise resume one
                // frame too far ahead, flipping the 2-frame byte parity for the
                // rest of the demo playback. The first transform frame still
                // consumes one ReadJoypad byte before the coroutine freeze, so
                // restore that skipped byte advance when the transform exits.
                *frame_counter = frame_counter.saturating_sub(1);
                *data_index += 1;
                let data = if *use_demo_2 {
                    &demo::DEMO_INPUT_DATA_2[..]
                } else {
                    &demo::DEMO_INPUT_DATA_1[..]
                };
                let byte = data.get(*data_index).copied().unwrap_or(0);
                self.demo_buffered_input = Some(demo::decode_demo_byte(byte));
            }
            self.demo_transform_active = transform_active;
        } else {
            self.demo_transform_active = false;
        }
    }

    fn apply_form_width_overlap_fixup(&mut self, slice: &mut OpeningSliceState) {
        if self.progression.form_width_delta == 0 {
            return;
        }
        let Some(grid) = self.block_grid.as_ref() else {
            self.progression.form_width_delta = 0;
            return;
        };

        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let new_half = self.progression.current_form.half_width();

        // Probe left edge (x - half_width)
        let left_edge = x_px - new_half;
        let left_probe = terrain::probe_wall(grid, x_px, y_px, new_half, PLAYER_HEIGHT, false);
        if left_probe.hit {
            // Push right: align to tile boundary + half_width
            let aligned = (left_edge & !0xF) + 16 + new_half;
            slice.player.x_pos = (aligned as u32) << 16;
        } else {
            // Probe right edge (x + half_width)
            let right_edge = x_px + new_half;
            let right_probe = terrain::probe_wall(grid, x_px, y_px, new_half, PLAYER_HEIGHT, true);
            if right_probe.hit {
                // Push left: ASM formula (right_edge & 0xFFF0) - 1 - half_width
                let aligned = (right_edge & !0xF) - 1 - new_half;
                slice.player.x_pos = (aligned as u32) << 16;
            }
        }

        self.progression.form_width_delta = 0;
    }

    fn refresh_opening_slice(&mut self, input: FrameInput, jump_pressed: bool) {
        if !self.step_lifecycle(input) {
            self.demo_transform_active = false;
            return;
        }
        self.update_demo_transform_parity();

        let level_elapsed = self.level_elapsed_frames().unwrap_or(0);
        // When skip_entry_animation is set (after level-skip warp), pretend
        // the level has been running long enough to bypass the entry animation.
        // The original game has no entry animation — the player appears
        // instantly at the spawn point.
        let effective_elapsed = if self.skip_entry_animation || self.demo_gameplay_active() {
            level_elapsed + OPENING_SCROLL_START_FRAME
        } else {
            level_elapsed
        };
        let bounds = self.level_bounds(self.current_level);
        let mut slice = self.opening_slice.unwrap_or_else(|| {
            if matches!(self.boot_state, BootState::DemoPlay { .. })
                && self.current_level != LevelId(0)
            {
                self.level_respawn_slice(self.current_level)
            } else {
                OpeningSliceState::new(bounds)
            }
        });
        // ASM sub_7B30 (kid.asm:8230): before Kid_Transform, checks if the
        // new helmet's wider hitbox overlaps walls and immediately pushes the
        // player clear. This can come from a transform resolved last frame or
        // from a floating helmet prize collected later in this frame.
        self.apply_form_width_overlap_fixup(&mut slice);
        slice.prize_block_hit = None;
        slice.lift_block_hit = None;
        slice.drill_projectile_hit = None;
        slice.metal_block_hit = None;
        slice.smash_block_hit = None;
        slice.teleporter_contact = None;
        slice.ceiling_prize_clear = None;
        self.stomped_this_frame = false;
        self.blocks_hit_this_frame = 0;
        self.sound_test_play_pending = false;
        // Tick visual effects
        self.step_visual_effects();
        // Prize blocks headbutted from below update their queued sub_1007A
        // lifecycle before movement, so the clear frame is collision-visible.
        self.step_deferred_ceiling_clears();
        // Step helmet abilities BEFORE physics
        self.step_skycutter_flipboard(&mut slice, input);
        self.step_cyclone_flight(&mut slice, input);
        self.step_iron_knight_climb(&mut slice, input);
        self.step_red_stealth_sword(input);
        self.step_eyeclops_beam(&slice, input);
        self.step_maniaxe_axe(&slice, input);
        self.step_berzerker_charge(&slice);
        self.step_juggernaut_skull(&slice, input);
        self.step_micromax_stick(&mut slice, input);

        // Capture pre-physics velocity for stomp detection: after physics,
        // floor collision zeroes y_velocity so the stomp check would always fail.
        let pre_physics_y_velocity = slice.player.y_velocity;
        let pre_physics_y_pos = slice.player.y_pos;

        // Initialise physics trace for this frame if tracing is enabled.
        slice.physics_trace = if self.trace_physics {
            Some(PhysicsTrace::default())
        } else {
            None
        };

        // ASM Kid_Transform runs as a coroutine: during the transform animation,
        // the player object is in the transform state and normal movement/physics
        // don't run. Suppress physics while form_change_flash_frames > 0.
        if self.progression.form_change_flash_frames == 0 {
            let deferred_prize_ceiling_cells: Vec<(u16, u16)> = self
                .block_state
                .deferred_ceiling_clears
                .iter()
                .filter(|entry| entry.phase != DeferredPrizeBlockPhase::Deleted)
                .map(|entry| (entry.col, entry.row))
                .collect();
            // Helmet abilities that override normal gravity
            let custom_gravity = self.helmet.skycutter_on_board
                || self.helmet.cyclone_flying
                || self.helmet.iron_knight_climbing
                || self.helmet.micromax_stuck;
            let allow_asm_faithful_solid_touch_snap =
                !(self.demo_gameplay_active() && self.current_level == LevelId(demo::DEMO_2_LEVEL));
            slice.step_with_order(
                effective_elapsed,
                input,
                jump_pressed,
                self.progression.current_form,
                self.block_grid.as_mut(),
                self.block_list.as_ref(),
                custom_gravity,
                self.y_look_offset,
                self.physics_order,
                allow_asm_faithful_solid_touch_snap,
                &deferred_prize_ceiling_cells,
            );
            // ASM kid_flip (loc_ACB0): Cyclone flight activation from C press
            // runs AFTER physics so flight takes effect next frame.
            self.step_cyclone_activation(&slice, input);
        }
        self.finish_deferred_ceiling_clears();
        // Wide spawn-support settling is a bootstrap/respawn helper, not
        // general demo gameplay logic. Keeping it live after the hidden demo
        // settle finishes yanks Kid down onto nearby support during later arcs,
        // which in turn makes the frame-1406 diamond collectible too early.
        let demo_bootstrap_support_active = self
            .demo_startup_frame()
            .is_some_and(|frame| frame <= demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME);
        let spawn_support_snapped = (self.skip_entry_animation || demo_bootstrap_support_active)
            && self
                .block_grid
                .as_ref()
                .is_some_and(|grid| self.try_settle_on_spawn_support(&mut slice, grid));
        let demo_bootstrap_floor_snapped = self.try_snap_demo_bootstrap_floor(&mut slice);
        if spawn_support_snapped || demo_bootstrap_floor_snapped {
            slice.update_camera_follow(self.y_look_offset);
        }

        // Y look-ahead accumulator ($FFFFFB58): +1/frame while grounded+down,
        // -8/frame on release, clamped to [0, 30].
        if input.down && slice.player.on_ground {
            self.y_look_offset = (self.y_look_offset + 1).min(30);
        } else if self.y_look_offset > 0 {
            self.y_look_offset = (self.y_look_offset - 8).max(0);
        }

        // Berzerker charge: break Smash blocks on wall contact
        if self.helmet.berzerker_charging {
            self.berzerker_break_wall_block(&slice);
        }

        // Step platform movement scripts
        if let Some(ref script_table) = self.platform_script_table {
            for platform in &mut self.active_platforms {
                platform.step_script(script_table);
            }
        }

        // Platform collision: player can stand on top of platforms
        apply_platform_collision(&mut slice, &self.active_platforms);

        // Evanescent blocks: trigger when player stands on one, tick timers,
        // and clear expired blocks from the grid.
        self.step_evanescent_blocks(&slice);

        // Ghost blocks: cycle between solid and vanished states
        self.step_ghost_blocks();

        // Shifting blocks: move block up one cell when player stands on it
        self.apply_shifting_blocks(&slice);

        // Lift platforms: step movement and collision
        self.step_lift_platforms(&mut slice);

        // Shooter blocks: fire projectiles periodically
        self.step_shooter_blocks(&slice);

        // Weather: hail spawns falling projectiles, lava geysers erupt
        self.step_weather(&slice);
        self.step_lava_geysers(&slice);

        // Queue the full prize-block lifecycle from a headbutt. The tile stays
        // Solid (0x6F) until the coroutine completes, then the same state
        // record clears the tile and spawns the prize object.
        if let Some((col, row, restore_cell)) = slice.ceiling_prize_clear
            && let Some((content, _, _)) = slice.prize_block_hit
        {
            self.block_state
                .deferred_ceiling_clears
                .push(DeferredCeilingClear {
                    col,
                    row,
                    content,
                    restore_cell,
                    deleted_frames_remaining: Self::PRIZE_BLOCK_DELETED_FRAMES,
                    phase: DeferredPrizeBlockPhase::Deleted,
                });
        }

        self.process_block_effects(&mut slice);
        self.collect_floating_items(&mut slice);
        if self.progression.pending_form.is_some() {
            // Oracle demo 0x1B enters Kid_Transform on the same frame that the
            // floating Cyclone prize is touched, so the traced helmet id and the
            // 2px wall-separation push are already visible at frame end.
            self.resolve_pending_form_change();
            self.apply_form_width_overlap_fixup(&mut slice);
        }
        if self.resolve_frame_hazards(&mut slice, input, pre_physics_y_velocity, pre_physics_y_pos)
        {
            return;
        }
        self.opening_slice = Some(slice);
    }
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod tests;
