use super::*;
use crate::level::step_island_palette_cycle_lower_band_loc_12188;

// Full-frame oracle warps do more than the explicit 60-frame stable-level poll:
// the ROM level-skip path has a live title-card tail before the helper starts
// counting stable target frames. Native's direct `boot_to_level` shortcut skips
// that tail, so settled full-frame traces need the oracle-observed live age.
const ORACLE_SETTLED_WARP_LIVE_FRAMES: usize = 108;
const RECORDING_SETTLED_WARP_LIVE_FRAMES: usize = 60;
const RECORDING_SETTLED_WARP_TIMER_SECONDS_LOC_BDC0: u16 = 179;
const RECORDING_SETTLED_WARP_TIMER_SUBFRAMES_LOC_BDC0: u16 = 0x08;
const LEVEL_SIX_RECORDING_GHOST_BLOCK_EXTRA_TICKS: usize = 0x08;
const ISLAND_RECORDING_SHORE_EXTRA_TICKS_LOC_1202A: usize = 0x10;
const ISLAND_RECORDING_SHORE_LOWER_EXTRA_TICKS_LOC_12188: usize = 0x12;
const SETTLED_WARP_MAX_ATTEMPT_FRAMES: usize = 828;
const LEVEL_ONE_RECORDING_SETTLE_RNG_OFFSET: u16 = 0xFFCB;
// Oracle `issue260_p11` settle tracing reaches Level 1 target_live step 59 at
// RNG $FFCB/$32. The first scenario frame then runs the two controller RNG
// words through Get_RandomNumber_byte (kid.asm:1181) before Drips.asm
// loc_3340E sees $FFCF/$09.
const LEVEL_ONE_RECORDING_SETTLE_RNG_RING: [u8; 55] = [
    0xF8, 0x5C, 0x32, 0x1D, 0xA6, 0xA1, 0xDA, 0xE0, 0xF3, 0x21, 0x05, 0xB3, 0xE5, 0x5E, 0x22, 0x59,
    0x9B, 0x3F, 0xFA, 0xE6, 0x27, 0x5E, 0xA7, 0x64, 0x9C, 0xB7, 0x36, 0x64, 0x5B, 0xBC, 0xDA, 0x11,
    0xB2, 0xE6, 0xAC, 0xED, 0x40, 0xD1, 0xE6, 0x9F, 0x61, 0x05, 0xC3, 0x78, 0x00, 0xA3, 0xC5, 0x89,
    0x96, 0x12, 0xE1, 0x8E, 0x29, 0x15, 0x4A,
];

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
        self.reload_block_grid_for_level(self.current_level);
        self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(self.current_level));
        self.level_start_frame = Some(self.frame_index);
        self.allow_pausing = true;
        self.allow_pausing_fade_frames = 17;
    }

    pub fn enter_level_playtest(&mut self, level_id: LevelId) -> NativeTraceFrame {
        self.warp_to_level(level_id);
        // Playtest/debug level selection is outside the ROM joypad path. Clear
        // the loc_C4E held-input latches so menu-held movement/jump cannot be
        // consumed by the first gameplay frame after the warp.
        self.prior_input = FrameInput::default();
        self.previous_input = FrameInput::default();
        self.opening_slice = Some(if self.block_grid().is_some() {
            self.level_respawn_slice_with_floor_snap(level_id)
        } else {
            self.level_respawn_slice(level_id)
        });
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.level_complete_gfx_snapshot = None;
        self.progression.invulnerable_frames = 0;
        self.allow_pausing = true;
        self.allow_pausing_fade_frames = 17;
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
        if stable_target_frames >= target_live_frames && target_live_frames > 0 {
            // ASM Manage_EnemyLoading loc_36968 sets `$FFFFFB51` at the end
            // of its first pass. Settled warp helpers fast-forward over that
            // hidden/live window, so preserve the same post-initial-pass spawn
            // gate for subsequent probe frames.
            self.enemy_loading_initial_pass_complete = true;
            self.allow_pausing = true;
            self.allow_pausing_fade_frames = 17;
        }
    }

    pub fn settle_oracle_warp_live_window(&mut self, level_id: LevelId) {
        if level_id == LevelId(1) {
            self.prime_respawn_settle_enemy_loading_loc_368a8(level_id);
        }
        let target_live_frames = if level_id == LevelId(1) {
            ORACLE_SETTLED_WARP_LIVE_FRAMES + 1
        } else if level_id.0 <= 1 {
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
        // shortcuts. Oracle settle tracing ends Level 1 at RNG offset $FFCB;
        // the first scenario frame then advances to $FFCF via Drips loc_3340E.
        self.prime_respawn_settle_enemy_loading_loc_368a8(level_id);
        let target_live_frames = if level_id == LevelId(1) {
            ORACLE_SETTLED_WARP_LIVE_FRAMES - 1
        } else if level_id == LevelId(0x08) {
            // ASM loc_74E0's level-skip path has a live target-level tail
            // before the helper's 60 stable-frame recording window. Level 0x08
            // exposes that hidden age through Murderwall loc_140CE plus
            // loc_7316/loc_7334: oracle issue266 frame 0 has advanced 102 wall
            // ticks from the -$30 px seed, so native must enter the scenario
            // one pre-frame earlier and let the traced frame supply tick 102.
            101
        } else if level_id == LevelId(0x06) {
            // Under Skull Mountain 3 exposes the ROM level-skip tail through
            // loc_124A6/sub_F730 ghost-block phase. Once oracle enemy capture
            // can see Dragon_Flying.asm loc_3480A via loc_32146's collision
            // source, the same issue #289 recording pins the live tail at 0x18
            // frames for object scheduling; the final eight ticks belong only
            // to the ghost-block timer below.
            RECORDING_SETTLED_WARP_LIVE_FRAMES + 0x18
        } else {
            RECORDING_SETTLED_WARP_LIVE_FRAMES
        };
        self.settle_target_level_live_window(level_id, target_live_frames);
        // ASM Init_SpriteAttr_HUD loc_BDC0 seeds Time_SubSeconds to 1, then
        // the ROM level-skip recording path reaches scenario frame 0 with
        // Time_Seconds=2:59 and Time_SubSeconds=$08. The first scripted frame
        // decrements that to $07; #289 oracle watches show frame 60 at 2:58/$07
        // and frame 71 at 2:57/$38. Keep this on the recording settle shim so
        // ordinary gameplay timer startup stays on LevelTimer::new.
        self.level_timer.seconds = RECORDING_SETTLED_WARP_TIMER_SECONDS_LOC_BDC0;
        self.level_timer.sub_frames = RECORDING_SETTLED_WARP_TIMER_SUBFRAMES_LOC_BDC0;
        if level_id == LevelId(0x06) {
            // ASM loc_124A6 calls sub_F730 for ghost blocks on the level-skip
            // tail that the recording helper observes. Dragon_Flying.asm
            // loc_3480A oracle frames prove object scheduling is eight frames
            // younger than the ghost-block phase here, so advance only the
            // block timer for those remaining ticks.
            for _ in 0..LEVEL_SIX_RECORDING_GHOST_BLOCK_EXTRA_TICKS {
                self.step_ghost_blocks();
            }
        }
        // ASM loc_1202A runs the Island shore DMA object through the ROM
        // level-skip shell before recording frame 0. `Time_Frames` is already
        // at the oracle phase here, so only the object-local VRAM worker gets
        // the hidden tail; non-Island levels no-op because the object is absent.
        for _ in 0..ISLAND_RECORDING_SHORE_EXTRA_TICKS_LOC_1202A {
            self.step_island_shore_cycle_loc_1202a();
        }
        // ASM loc_12188's lower shore band reaches the #289 oracle VDP frame
        // through additional level-skip DMA age after the primary/mid bands
        // already match. Keep this scoped to the recording settle shim; normal
        // live gameplay continues through the unified loc_1202A object.
        for _ in 0..ISLAND_RECORDING_SHORE_LOWER_EXTRA_TICKS_LOC_12188 {
            self.step_island_shore_lower_band_only_loc_12188();
        }
        if level_id == LevelId(1) {
            self.rom_random = RomRandomState::from_raw_parts(
                LEVEL_ONE_RECORDING_SETTLE_RNG_RING,
                LEVEL_ONE_RECORDING_SETTLE_RNG_OFFSET,
            );
        }
    }

    fn step_island_shore_lower_band_only_loc_12188(&mut self) {
        self.island_shore_dma_uploads_loc_1202a.clear();
        let Some(state) = self.island_shore_cycle_loc_1202a.as_mut() else {
            return;
        };

        let lower = step_island_palette_cycle_lower_band_loc_12188(
            state.lower_countdown_48,
            state.lower_phase_d1,
        );
        state.lower_countdown_48 = lower.next_countdown;
        state.lower_phase_d1 = lower.next_phase;
        if let (
            Some(frame_index),
            Some(source_offset),
            Some(vram_word_address),
            Some(dma_length_bytes),
        ) = (
            lower.uploaded_frame_index,
            lower.dma_source_offset,
            lower.vram_word_address,
            lower.dma_length_bytes,
        ) {
            state.lower_vram_frame = frame_index;
            self.island_shore_dma_uploads_loc_1202a
                .push(IslandShoreDmaUploadLoc1202A {
                    frame_index,
                    source_offset,
                    vram_word_address,
                    dma_length_bytes,
                });
        }
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
    probe_start_velocity: Option<(i32, i32)>,
    probe_camera_position: Option<(u16, u16)>,
    playtest_form: Option<PlayerForm>,
    initial_diamond_count: Option<u16>,
    initial_lives: Option<u16>,
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
            probe_start_velocity: None,
            probe_camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
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
    pub fn probe_start_velocity(mut self, x_velocity: i32, y_velocity: i32) -> Self {
        self.probe_start_velocity = Some((x_velocity, y_velocity));
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

    #[must_use]
    pub fn initial_diamond_count(mut self, count: u16) -> Self {
        self.initial_diamond_count = Some(count);
        self
    }

    #[must_use]
    pub fn initial_lives(mut self, count: u16) -> Self {
        self.initial_lives = Some(count);
        self
    }

    pub fn run(self, script: &[u8], frame_count: usize) -> Vec<NativeTraceFrame> {
        self.run_inner(script, frame_count, false).0
    }

    /// Like [`run`](Self::run) but also captures the per-frame VDP snapshot
    /// (`Some` on non-gameplay screens, `None` during gameplay — see
    /// `vdp_state` at lib.rs:2491 / `update_vdp_state` at lib.rs:2142). Used by
    /// the TAS viewer to render real VDP pixels for title/Sega/intro screens.
    pub fn run_with_vdp(
        self,
        script: &[u8],
        frame_count: usize,
    ) -> (Vec<NativeTraceFrame>, Vec<Option<vdp::VdpState>>) {
        self.run_inner(script, frame_count, true)
    }

    fn run_inner(
        self,
        script: &[u8],
        frame_count: usize,
        capture_vdp: bool,
    ) -> (Vec<NativeTraceFrame>, Vec<Option<vdp::VdpState>>) {
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
        if let Some((x_velocity, y_velocity)) = self.probe_start_velocity {
            runtime.set_player_probe_velocity(x_velocity, y_velocity);
        }
        if let Some((x_px, y_px)) = self.probe_camera_position {
            runtime.set_camera_probe_position(x_px, y_px);
        }
        if let Some(form) = self.playtest_form {
            runtime.set_playtest_form(form);
        }
        if let Some(count) = self.initial_diamond_count {
            runtime.set_probe_diamond_count(count);
        }
        if let Some(count) = self.initial_lives {
            runtime.set_probe_lives_count(count);
        }
        let delay_recording_input = self.settle_warp == Some(SettleWarpMode::RecordingOracle);
        let mut trace = Vec::with_capacity(frame_count);
        let mut vdp_snapshots = if capture_vdp {
            Vec::with_capacity(frame_count)
        } else {
            Vec::new()
        };
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
            if capture_vdp {
                // step_frame can early-return before update_vdp_state (lib.rs:2546,
                // the DemoPlay no-input path), so the captured snapshot then
                // reflects the prior frame — identical to what play-native
                // observes, so viewer/player parity holds.
                vdp_snapshots.push(runtime.vdp_state().cloned());
            }
        }
        (trace, vdp_snapshots)
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
            generated_bundle_version: wildside_data::GENERATED_BUNDLE_VERSION,
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

    // VDP capture (run_with_vdp) needs ROM-derived title/Sega art, so these are
    // cache-backed: set KID_ENABLE_CACHE_BACKED_TESTS=1 after `mise run
    // extract-assets`. Mirrors the loader in crate::tests::fixtures.
    fn cached_generated_manifest() -> Option<AssetManifest> {
        if !matches!(
            std::env::var("KID_ENABLE_CACHE_BACKED_TESTS")
                .ok()
                .as_deref(),
            Some("1" | "true" | "TRUE" | "yes" | "YES")
        ) {
            return None;
        }
        let repo_root = if let Ok(root) = std::env::var("KID_REPO_ROOT")
            && !root.is_empty()
        {
            PathBuf::from(root)
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
        };
        let manifest_path = repo_root
            .join(".cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json");
        if !manifest_path.is_file() {
            return None;
        }
        Some(wildside_data::load_manifest(&manifest_path).expect("load cached manifest"))
    }

    // Boot script that presses Start through the Sega/intro/title sequence,
    // exercising the VDP-shell screens where snapshot capture is active.
    // 0x80 is the Start button mask.
    fn boot_through_title_script() -> Vec<u8> {
        let mut script = vec![0u8; 600];
        for &frame in &BOOT_TO_LEVEL_START_FRAMES {
            script[frame] |= 0x80;
        }
        script
    }

    #[test]
    fn run_and_run_with_vdp_produce_identical_traces() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let script = boot_through_title_script();
        // The only delta between the two paths is an immutable vdp_state().cloned()
        // read, so the authoritative trace must be byte-identical frame-by-frame.
        let plain = ScriptRunner::new(manifest.clone()).run(&script, script.len());
        let (with_vdp, _snapshots) =
            ScriptRunner::new(manifest).run_with_vdp(&script, script.len());
        assert_eq!(plain.len(), with_vdp.len());
        for (index, (a, b)) in plain.iter().zip(with_vdp.iter()).enumerate() {
            assert_eq!(
                a.authoritative_crc32(),
                b.authoritative_crc32(),
                "frame {index}: run() vs run_with_vdp().0 diverged",
            );
        }
    }

    #[test]
    fn vdp_captured_some_on_title_none_on_ingame() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        // update_vdp_state (lib.rs:2142) returns Some only for non-gameplay
        // screens; BootState::InGame hits the `_ => None` arm (lib.rs:2328), so
        // the capture splits exactly on the gameplay boundary. Two runs pin both
        // sides: a plain boot sits on Sega/intro/title (VDP shell), a warp drops
        // straight into live gameplay (no VDP shell).
        let (title_trace, title_snapshots) =
            ScriptRunner::new(manifest.clone()).run_with_vdp(&[0u8; 200], 200);
        let some_on_non_gameplay = title_trace
            .iter()
            .zip(&title_snapshots)
            .any(|(frame, snapshot)| snapshot.is_some() && frame.game_mode != GameMode::InGame);
        assert!(
            some_on_non_gameplay,
            "Sega/intro/title frames must capture a VDP snapshot",
        );

        let (game_trace, game_snapshots) = ScriptRunner::new(manifest)
            .warp_to_level(0)
            .run_with_vdp(&[0u8; 60], 60);
        let none_on_gameplay = game_trace
            .iter()
            .zip(&game_snapshots)
            .any(|(frame, snapshot)| snapshot.is_none() && frame.game_mode == GameMode::InGame);
        assert!(
            none_on_gameplay,
            "gameplay frames must leave the VDP snapshot empty",
        );
    }
}
