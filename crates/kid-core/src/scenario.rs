use super::*;

impl NativeRuntime {
    /// Boot through menus to level 0, then advance to `target` using the
    /// level skip mechanism. Used by sweep infrastructure to reach any level
    /// without encoding warp buttons in the input script.
    ///
    /// Boot timing matches the oracle helper exactly: 720 frames with
    /// Start presses at frames 120, 200, 280, 360, 440, 460.
    pub fn boot_to_level(&mut self, target: LevelId) {
        let start = FrameInput {
            start: true,
            ..FrameInput::default()
        };
        let idle = FrameInput::default();
        let start_frames: &[usize] = &[120, 200, 280, 360, 440, 460];

        for frame in 0..720 {
            if start_frames.contains(&frame) {
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

        // The original ASM has no entry animation — the player appears
        // instantly at the spawn point. Always skip the native entry
        // animation so parity timing matches the oracle.
        self.skip_entry_animation = true;
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

pub fn run_script(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
) -> Vec<NativeTraceFrame> {
    run_script_with_warp(assets, script, frame_count, None)
}

/// Run a scripted scenario, optionally warping to a target level first.
/// When `warp_to_level` is `Some`, the runtime boots to level 0 and then
/// advances to the target level before running the script.
pub fn run_script_with_warp(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
    warp_to_level: Option<u8>,
) -> Vec<NativeTraceFrame> {
    let mut runtime = NativeRuntime::with_generated_level_layout(assets);
    if let Some(level_id) = warp_to_level {
        runtime.boot_to_level(LevelId(level_id));
    }
    run_script_with_runtime(&mut runtime, script, frame_count)
}

fn run_script_with_runtime(
    runtime: &mut NativeRuntime,
    script: &[u8],
    frame_count: usize,
) -> Vec<NativeTraceFrame> {
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
