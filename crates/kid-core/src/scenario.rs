use super::*;

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
        self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(self.current_level));
        self.level_start_frame = Some(self.frame_index);
    }

    pub fn enter_level_playtest(&mut self, level_id: LevelId) -> NativeTraceFrame {
        self.begin_level_entry(level_id);
        let max_bootstrap_frames = TITLE_TO_LEVEL_FRAMES + PLAYTEST_LEVEL_SETTLE_FRAMES + 2;
        for _ in 0..max_bootstrap_frames {
            let trace = self.trace_frame();
            if trace.game_mode == GameMode::InGame
                && trace.kid_on_ground == Some(true)
                && trace.kid_y_pos == Some(OPENING_GROUNDED_KID_Y_POS)
            {
                return trace;
            }
            self.step_frame(FrameInput::default());
        }
        self.trace_frame()
    }
}

/// Builder for running scripted scenarios with configurable options.
pub struct ScriptRunner {
    assets: AssetManifest,
    warp_to_level: Option<u8>,
    physics_order: Option<PhysicsOrder>,
    trace_physics: bool,
}

impl ScriptRunner {
    pub fn new(assets: AssetManifest) -> Self {
        Self {
            assets,
            warp_to_level: None,
            physics_order: None,
            trace_physics: false,
        }
    }

    pub fn warp_to_level(mut self, level: u8) -> Self {
        self.warp_to_level = Some(level);
        self
    }

    pub fn physics_order(mut self, order: PhysicsOrder) -> Self {
        self.physics_order = Some(order);
        self
    }

    pub fn trace_physics(mut self) -> Self {
        self.trace_physics = true;
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
            runtime.boot_to_level(LevelId(level_id));
        }
        let mut trace = Vec::with_capacity(frame_count);
        for frame_index in 0..frame_count {
            let input = script
                .get(frame_index)
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
