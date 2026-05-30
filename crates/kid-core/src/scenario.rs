use super::*;

// Full-frame oracle warps do more than the explicit 60-frame stable-level poll:
// the ROM level-skip path has a live title-card tail before the helper starts
// counting stable target frames. Native's direct `boot_to_level` shortcut skips
// that tail, so settled full-frame traces need the oracle-observed live age.
const ORACLE_SETTLED_WARP_LIVE_FRAMES: usize = 108;
const RECORDING_SETTLED_WARP_LIVE_FRAMES: usize = 60;
const SETTLED_WARP_MAX_ATTEMPT_FRAMES: usize = 828;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettleWarpMode {
    FullFrameOracle,
    RecordingOracle,
}

fn is_live_unpaused_target_level(runtime: &NativeRuntime, level_id: LevelId) -> bool {
    runtime.trace_frame().game_mode == GameMode::InGame
        && runtime.current_level == level_id
        && !runtime.paused
}

impl NativeRuntime {
    /// Boot through menus to level 0, then advance to `target` using the
    /// level skip mechanism. Used by sweep infrastructure to reach any level
    /// without encoding warp buttons in the input script.
    ///
    /// Boot timing matches the oracle helper exactly: 720 frames with
    /// Start presses at frames 120, 200, 280, 360, 460.
    pub fn boot_to_level(&mut self, target: LevelId) {
        let start = FrameInput {
            start: true,
            ..FrameInput::default()
        };
        let idle = FrameInput::default();

        for frame in 0..720 {
            if BOOT_TO_LEVEL_START_FRAMES.contains(&frame) {
                self.step_frame(start);
            } else {
                self.step_frame(idle);
            }
        }

        // The boot-title oracle now keeps the Level 1 title-card shell hidden
        // through frame 719, so a fixed 720-frame bootstrap no longer implies
        // that live gameplay has started. Continue idling until the hidden
        // hand-off actually reaches `InGame`, keeping the helper independent
        // of the exact title-card settle length.
        for _ in 0..720 {
            if matches!(self.boot_state, BootState::InGame) {
                break;
            }
            self.step_frame(idle);
        }

        // Jump directly to target level using internal skip mechanism.
        if self.current_level != target {
            self.pending_level_skip = Some(target);
            self.step_frame(idle);
            self.pending_level_skip_resume = true;
            self.step_frame(idle);
            for _ in 0..LEVEL_SKIP_RESUME_TOTAL_FRAMES {
                self.step_frame(idle);
            }
        }

        // Reset the opening slice to the correct spawn position. Floor-snap
        // places the player on the actual floor (header spawn Y is approximate).
        // After floor-snap, simulate the camera's gravity-following path to
        // match how the oracle's camera converges during real gravity settling.
        self.boot_state = BootState::InGame;
        self.level_skip_resume_hold = false;
        self.pending_level_skip = None;
        self.pending_level_skip_resume = false;
        self.level_skip_transition = None;
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.progression.number_lives = 3;
        self.progression.number_hitpoints = self.progression.current_form.max_hitpoints();
        self.just_received_damage = false;
        self.death_shell_snapshot = None;
        self.level_complete_gfx_snapshot = None;
        self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(self.current_level));
        self.level_start_frame = Some(self.frame_index);
    }

    pub fn enter_level_playtest(&mut self, level_id: LevelId) -> NativeTraceFrame {
        self.warp_to_level(level_id);
        self.opening_slice = Some(if self.block_grid().is_some() {
            self.level_respawn_slice_with_floor_snap(level_id)
        } else {
            self.level_respawn_slice(level_id)
        });
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.level_complete_gfx_snapshot = None;
        self.progression.invulnerable_frames = 0;
        self.refresh_active_roster();
        self.trace_frame()
    }

    fn settle_target_level_live_window(&mut self, level_id: LevelId, target_live_frames: usize) {
        let idle = FrameInput::default();
        let mut stable_target_frames = 0usize;

        for _ in 0..SETTLED_WARP_MAX_ATTEMPT_FRAMES {
            if stable_target_frames >= target_live_frames {
                break;
            }
            if is_live_unpaused_target_level(self, level_id) {
                stable_target_frames += 1;
                if stable_target_frames >= target_live_frames {
                    break;
                }
            } else {
                stable_target_frames = 0;
            }
            self.step_frame(idle);
        }
    }

    pub fn settle_oracle_warp_live_window(&mut self, level_id: LevelId) {
        let target_live_frames = if level_id.0 <= 1 {
            ORACLE_SETTLED_WARP_LIVE_FRAMES
        } else {
            1
        };
        self.settle_target_level_live_window(level_id, target_live_frames);
    }

    pub fn settle_recording_oracle_warp_live_window(&mut self, level_id: LevelId) {
        // ASM loc_1AE2E/loc_74E0 feed recording oracles through the helper's
        // `_settled_recording_oracle` branch. Level 0 comes straight through
        // the title-card gate and waits 60 stable live frames; Level 1 still
        // reaches the ROM's level-skip resume tail that native `boot_to_level`
        // shortcuts, so it shares the full-frame 108-frame compatibility age.
        self.prime_respawn_settle_enemy_loading_loc_368a8(level_id);
        let target_live_frames = if level_id == LevelId(1) {
            ORACLE_SETTLED_WARP_LIVE_FRAMES
        } else {
            RECORDING_SETTLED_WARP_LIVE_FRAMES
        };
        self.settle_target_level_live_window(level_id, target_live_frames);
    }
}

/// Builder for running scripted scenarios with configurable options.
pub struct ScriptRunner {
    assets: AssetManifest,
    warp_to_level: Option<u8>,
    settle_warp: Option<SettleWarpMode>,
    physics_order: Option<PhysicsOrder>,
    trace_physics: bool,
    probe_start_position: Option<(u16, u16)>,
    probe_camera_position: Option<(u16, u16)>,
    playtest_form: Option<PlayerForm>,
}

impl ScriptRunner {
    pub fn new(assets: AssetManifest) -> Self {
        Self {
            assets,
            warp_to_level: None,
            settle_warp: None,
            physics_order: None,
            trace_physics: false,
            probe_start_position: None,
            probe_camera_position: None,
            playtest_form: None,
        }
    }

    #[must_use]
    pub fn warp_to_level(mut self, level: u8) -> Self {
        self.warp_to_level = Some(level);
        self
    }

    #[must_use]
    pub fn settle_warp(mut self) -> Self {
        self.settle_warp = Some(SettleWarpMode::FullFrameOracle);
        self
    }

    #[must_use]
    pub fn settle_recording_warp(mut self) -> Self {
        self.settle_warp = Some(SettleWarpMode::RecordingOracle);
        self
    }

    #[must_use]
    pub fn physics_order(mut self, order: PhysicsOrder) -> Self {
        self.physics_order = Some(order);
        self
    }

    #[must_use]
    pub fn trace_physics(mut self) -> Self {
        self.trace_physics = true;
        self
    }

    #[must_use]
    pub fn probe_start_position(mut self, x_px: u16, y_px: u16) -> Self {
        self.probe_start_position = Some((x_px, y_px));
        self
    }

    #[must_use]
    pub fn probe_camera_position(mut self, x_px: u16, y_px: u16) -> Self {
        self.probe_camera_position = Some((x_px, y_px));
        self
    }

    #[must_use]
    pub fn playtest_form(mut self, form: PlayerForm) -> Self {
        self.playtest_form = Some(form);
        self
    }

    pub fn run(self, script: &[u8], frame_count: usize) -> Vec<NativeTraceFrame> {
        let mut runtime = NativeRuntime::with_generated_level_layout(self.assets);
        if let Some(order) = self.physics_order {
            runtime.set_physics_order(order);
        }
        if self.trace_physics {
            runtime.set_trace_physics(true);
        }
        if let Some(level_id) = self.warp_to_level {
            let level_id = LevelId(level_id);
            runtime.boot_to_level(level_id);
            match self.settle_warp {
                Some(SettleWarpMode::FullFrameOracle) => {
                    runtime.settle_oracle_warp_live_window(level_id);
                }
                Some(SettleWarpMode::RecordingOracle) => {
                    runtime.settle_recording_oracle_warp_live_window(level_id);
                }
                None => {}
            }
        }
        if let Some((x_px, y_px)) = self.probe_start_position {
            runtime.set_player_probe_position(x_px, y_px);
        }
        if let Some((x_px, y_px)) = self.probe_camera_position {
            runtime.set_camera_probe_position(x_px, y_px);
        }
        if let Some(form) = self.playtest_form {
            runtime.set_playtest_form(form);
        }
        let delay_recording_input = self.settle_warp == Some(SettleWarpMode::RecordingOracle);
        let mut trace = Vec::with_capacity(frame_count);
        for frame_index in 0..frame_count {
            // ASM Mode_Level (kid.asm:877-917) executes objects/player logic
            // before ReadJoypad, so the settled recording oracle's script byte
            // for frame N is only visible to gameplay on traced frame N+2.
            let input_index = if delay_recording_input {
                frame_index.checked_sub(1)
            } else {
                Some(frame_index)
            };
            let input = input_index
                .and_then(|index| script.get(index))
                .copied()
                .map(FrameInput::from_mask)
                .unwrap_or_default();
            trace.push(runtime.step_frame(input));
        }
        trace
    }
}

// Thin wrappers delegating to ScriptRunner to minimise caller churn.

pub fn run_script(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
) -> Vec<NativeTraceFrame> {
    ScriptRunner::new(assets).run(script, frame_count)
}

pub fn run_script_with_warp(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
    warp_to_level: Option<u8>,
) -> Vec<NativeTraceFrame> {
    let mut runner = ScriptRunner::new(assets);
    if let Some(level) = warp_to_level {
        runner = runner.warp_to_level(level);
    }
    runner.run(script, frame_count)
}

pub fn run_script_with_settled_warp(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
    warp_to_level: Option<u8>,
) -> Vec<NativeTraceFrame> {
    let mut runner = ScriptRunner::new(assets).settle_warp();
    if let Some(level) = warp_to_level {
        runner = runner.warp_to_level(level);
    }
    runner.run(script, frame_count)
}

pub fn run_script_with_physics_order(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
    warp_to_level: Option<u8>,
    physics_order: PhysicsOrder,
) -> Vec<NativeTraceFrame> {
    let mut runner = ScriptRunner::new(assets).physics_order(physics_order);
    if let Some(level) = warp_to_level {
        runner = runner.warp_to_level(level);
    }
    runner.run(script, frame_count)
}

pub fn run_script_with_trace(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
    warp_to_level: Option<u8>,
) -> Vec<NativeTraceFrame> {
    let mut runner = ScriptRunner::new(assets).trace_physics();
    if let Some(level) = warp_to_level {
        runner = runner.warp_to_level(level);
    }
    runner.run(script, frame_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn settled_warp_guard_requires_ingame_matching_level_and_unpaused() {
        let mut runtime = NativeRuntime::new(minimal_manifest());

        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(3);
        runtime.paused = false;
        assert!(is_live_unpaused_target_level(&runtime, LevelId(3)));

        runtime.paused = true;
        assert!(!is_live_unpaused_target_level(&runtime, LevelId(3)));

        runtime.paused = false;
        assert!(!is_live_unpaused_target_level(&runtime, LevelId(4)));

        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        assert!(!is_live_unpaused_target_level(&runtime, LevelId(3)));
    }

    #[test]
    fn settled_warp_attempt_budget_covers_live_window_plus_hidden_bootstrap_tail() {
        assert_eq!(SETTLED_WARP_MAX_ATTEMPT_FRAMES, 828);
    }

    fn minimal_manifest() -> AssetManifest {
        AssetManifest {
            generated_bundle_version: kid_data::GENERATED_BUNDLE_VERSION,
            rom_sha1: String::from("test"),
            upstream_commit: String::from("test"),
            source_root: PathBuf::new(),
            output_root: PathBuf::new(),
            map_order: Vec::new(),
            levels: Vec::new(),
            constants: Vec::new(),
            ram_variables: Vec::new(),
            enemy_sources: Vec::new(),
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_levels: Vec::new(),
        }
    }
}
