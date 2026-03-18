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

pub use types::*;
pub use scenario::{run_script, run_script_with_warp};
use constants::*;
#[cfg(test)]
pub(crate) use trace::opening_level_one_object_counts;
use enemies::*;
use kid_data::{
    AssetManifest, EnemyLayout, EnemyLayoutObject, PlatformLayout, PlatformPlacement,
    resource_level_record,
};
use physics::{Direction, HorizontalInput, LevelBounds};
use platforms::*;
use std::fs;

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
    /// Title screen idle frame counter: triggers demo after 640 frames.
    title_idle_frames: u16,
    /// Alternates between demo 1 and demo 2 (ASM $FFFFFBC8 toggle).
    demo_toggle: bool,
    /// Two-player alternating mode enabled.
    two_player: bool,
    /// Current player index (0=P1, 1=P2). Mirrors ASM $FFFFFC39.
    current_player_index: u8,
    /// Per-player persistent state slots.
    player_slots: [PlayerSlot; 2],
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
            demo_toggle: false,
            two_player: false,
            current_player_index: 0,
            player_slots: [PlayerSlot::default(), PlayerSlot::default()],
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

        self.refresh_active_roster();
        // Diamond Power uses raw input (Start+A held) since gameplay_input delays A by 1 frame
        if matches!(self.player_lifecycle, PlayerLifecycle::Alive)
            && matches!(self.boot_state, BootState::InGame | BootState::DemoPlay { .. })
            && !self.paused
        {
            self.step_diamond_power(input);
        }
        self.refresh_opening_slice(self.gameplay_input(input), self.gameplay_jump_pressed());
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

    fn refresh_opening_slice(&mut self, input: FrameInput, jump_pressed: bool) {
        if !self.step_lifecycle(input) {
            return;
        }

        let level_elapsed = self.level_elapsed_frames().unwrap_or(0);
        // When skip_entry_animation is set (after level-skip warp), pretend
        // the level has been running long enough to bypass the entry animation.
        // The original game has no entry animation — the player appears
        // instantly at the spawn point.
        let effective_elapsed = if self.skip_entry_animation {
            level_elapsed + OPENING_SCROLL_START_FRAME
        } else {
            level_elapsed
        };
        let bounds = self.level_bounds(self.current_level);
        let mut slice = self
            .opening_slice
            .unwrap_or_else(|| OpeningSliceState::new(bounds));
        slice.prize_block_hit = None;
        slice.lift_block_hit = None;
        slice.drill_projectile_hit = None;
        slice.metal_block_hit = None;
        slice.smash_block_hit = None;
        slice.teleporter_contact = None;
        self.stomped_this_frame = false;
        self.blocks_hit_this_frame = 0;
        self.sound_test_play_pending = false;
        // Tick visual effects
        self.step_visual_effects();
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

        // Helmet abilities that override normal gravity
        let custom_gravity = self.helmet.skycutter_on_board
            || self.helmet.cyclone_flying
            || self.helmet.iron_knight_climbing
            || self.helmet.micromax_stuck;
        slice.step(
            effective_elapsed,
            input,
            jump_pressed,
            self.progression.current_form,
            self.block_grid.as_mut(),
            self.block_list.as_ref(),
            custom_gravity,
            self.y_look_offset,
        );
        if self.skip_entry_animation
            && let Some(grid) = self.block_grid.as_ref()
            && self.try_settle_on_spawn_support(&mut slice, grid)
        {
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

        self.process_block_effects(&mut slice);
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
