use super::*;
// Full-frame oracle warps do more than the explicit 60-frame stable-level poll:
// the ROM level-skip path has a live title-card tail before the helper starts
// counting stable target frames. The compatibility `boot_to_level` path skips
// that tail, so settled full-frame traces need the oracle-observed live age.
const ORACLE_SETTLED_WARP_LIVE_FRAMES: usize = 108;
const RECORDING_SETTLED_WARP_LIVE_FRAMES: usize = 60;
const RECORDING_SETTLED_WARP_TIMER_SECONDS_LOC_BDC0: u16 = 179;
const RECORDING_SETTLED_WARP_TIMER_SUBFRAMES_LOC_BDC0: u16 = 0x08;
const LEVEL_ZERO_RECORDING_TIMER_SECONDS_BEFORE_FRAME_0_LOC_BE82: u16 = 180;
const LEVEL_ZERO_RECORDING_TIMER_SUBFRAMES_BEFORE_FRAME_0_LOC_BE82: u16 = 1;
const LEVEL_ZERO_RECORDING_TIME_FRAMES_BEFORE_FRAME_0_LOC_3038A: u16 = 0x003C;
const SETTLED_WARP_MAX_ATTEMPT_FRAMES: usize = 828;
const RECORDING_DEBUG_TITLE_WAIT_FRAMES_LOC_1AE2E: usize = 120;

/// ASM loc_74E0 (kid.asm): measured per-level load completion of the
/// debug-warp title window (#442). The ROM's loader blocks the frame loop, so
/// controller RNG freezes until this title-wait step; the level then runs
/// live Mode_Level under the remaining window. Measured by
/// `oracle-settle-census` derivation v2 (run 1786640409) and checked in as
/// `crates/wildside-cli/data/oracle-settle-shell.yaml`, with direct title-wait
/// traces for 0x1F and 0x4B. An exhaustive core test pins the table. Level 0
/// takes the title path and has no debug-warp load freeze.
pub(crate) fn debug_level_entry_load_complete_step_loc_74e0(level: LevelId) -> Option<usize> {
    match level {
        LevelId(0x49) => Some(56),
        LevelId(0x4d | 0x52) => Some(57),
        LevelId(0x4a | 0x4f | 0x50 | 0x5d | 0x5e | 0x67 | 0x68) => Some(58),
        LevelId(0x59) => Some(59),
        LevelId(0x4b | 0x53 | 0x58 | 0x5f) => Some(60),
        LevelId(0x55 | 0x5a | 0x66) => Some(61),
        LevelId(0x57 | 0x60) => Some(62),
        LevelId(0x33 | 0x4c | 0x65) => Some(63),
        LevelId(0x4e | 0x54 | 0x56 | 0x5c | 0x61) => Some(64),
        LevelId(0x51 | 0x5b) => Some(65),
        LevelId(0x2a) => Some(66),
        LevelId(0x1b) => Some(67),
        LevelId(0x07 | 0x10 | 0x26) => Some(68),
        LevelId(0x16 | 0x1e | 0x27) => Some(69),
        LevelId(0x01 | 0x0b | 0x1d | 0x64) => Some(70),
        LevelId(0x02 | 0x05 | 0x0c | 0x1a | 0x1c | 0x25 | 0x39 | 0x62) => Some(71),
        LevelId(0x48 | 0x63) => Some(72),
        LevelId(0x03 | 0x2b) => Some(73),
        LevelId(0x0a | 0x0e | 0x11 | 0x15 | 0x18 | 0x24 | 0x3a | 0x3e) => Some(74),
        // #509: 0x1F allocates its RockTanks on oracle title-wait row 75;
        // RNG resumes on row 76, after the first object scheduler pass.
        LevelId(0x09 | 0x1f | 0x23 | 0x28 | 0x2f | 0x30 | 0x38 | 0x3c | 0x41 | 0x45 | 0x47) => {
            Some(75)
        }
        LevelId(0x06 | 0x0d | 0x17 | 0x2c | 0x2e | 0x31 | 0x37 | 0x44 | 0x46) => Some(76),
        LevelId(0x0f | 0x19 | 0x3d | 0x40 | 0x42) => Some(77),
        LevelId(0x22 | 0x2d | 0x36) => Some(78),
        LevelId(0x14 | 0x20 | 0x21 | 0x34 | 0x35 | 0x43) => Some(79),
        LevelId(0x08 | 0x13 | 0x29 | 0x32) => Some(80),
        LevelId(0x04) => Some(81),
        LevelId(0x3b) => Some(82),
        LevelId(0x12) => Some(84),
        LevelId(0x3f) => Some(85),
        _ => None,
    }
}

/// The legacy fixed live boundary the measured table replaces. It equals
/// `load_complete_step - 1` for level 0x48 (72), the one level whose shell the
/// fixed model reproduced exactly — the anchor that fixed the mapping.
pub(crate) const DEBUG_LEVEL_ENTRY_LEGACY_LIVE_BOUNDARY_LOC_74E0: usize = 71;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettleWarpMode {
    FullFrameOracle,
    RecordingOracle,
}

fn is_live_unpaused_target_level(runtime: &NativeRuntime, level_id: LevelId) -> bool {
    let trace = runtime.trace_frame();
    trace.game_mode == GameMode::InGame
        && runtime.current_level == level_id
        && !runtime.paused
        && trace.kid_x_pos.is_some_and(|x| x != 0)
        && trace.number_objects != 0
}

fn settled_warp_complete(stable_target_frames: usize, target_live_frames: usize) -> bool {
    stable_target_frames >= target_live_frames && target_live_frames > 0
}

fn oracle_warp_primes_respawn_settle_enemy_loading(level_id: LevelId) -> bool {
    level_id == LevelId(1)
}

fn oracle_warp_target_live_frames(level_id: LevelId) -> usize {
    if level_id == LevelId(1) {
        ORACLE_SETTLED_WARP_LIVE_FRAMES + 1
    } else if level_id.0 <= 1 {
        ORACLE_SETTLED_WARP_LIVE_FRAMES
    } else {
        1
    }
}

fn recording_oracle_warp_target_live_frames(
    level_id: LevelId,
    debug_lifecycle_completed: bool,
) -> usize {
    if !debug_lifecycle_completed {
        return match level_id {
            LevelId(1) => ORACLE_SETTLED_WARP_LIVE_FRAMES - 1,
            LevelId(0x06) => RECORDING_SETTLED_WARP_LIVE_FRAMES + 0x18,
            LevelId(0x08) => 101,
            LevelId(0x1B) => 112,
            LevelId(0x3E) => 107,
            _ => RECORDING_SETTLED_WARP_LIVE_FRAMES,
        };
    }
    // #442: the debug title lifecycle now ages every measured level from its
    // per-level load completion (`debug_level_entry_load_complete_step_loc_74e0`),
    // so the residual stable window is the helper minimum everywhere — the
    // census (derivation v2, run 1786640409) measured exactly 60 on all 68
    // warped levels. The old per-level tails (0x01/0x06/0x08/0x1B/0x3E/0x46/
    // 0x64) and family adjustments modelled the load duration the fixed
    // boundary lacked; they are deleted, not zeroed, so a reappearing need
    // must re-justify itself against the shell table.
    match level_id {
        // #430 - ASM loc_74E0 -> loc_6E2: Elsewhere 9's Rubber-floor bounce
        // exposes the recording helper's post-step boundary. Oracle settle
        // run 1786728817 pins the pre-scenario state at target-live step 59.
        // Native's settle loop counts its already-live entry state as frame
        // one, so this route requires 59 rather than the generic 60.
        LevelId(0x52) => RECORDING_SETTLED_WARP_LIVE_FRAMES - 1,
        _ => RECORDING_SETTLED_WARP_LIVE_FRAMES,
    }
}

fn compatibility_boot_waits_for_ingame(use_recording_oracle_lifecycle: bool) -> bool {
    !use_recording_oracle_lifecycle
}

fn boot_boundary_rebuilds_pristine_level(
    used_debug_warp: bool,
    use_recording_oracle_lifecycle: bool,
) -> bool {
    !used_debug_warp || !use_recording_oracle_lifecycle
}

fn recording_settle_primes_respawn(level_id: LevelId, debug_lifecycle_completed: bool) -> bool {
    level_id == LevelId(0)
        || (!debug_lifecycle_completed && oracle_warp_primes_respawn_settle_enemy_loading(level_id))
}

fn recording_oracle_warp_hidden_object_passes_loc_e12(level_id: LevelId) -> usize {
    debug_level_entry_load_complete_step_loc_74e0(level_id).map_or(0, |load_complete| {
        RECORDING_DEBUG_TITLE_WAIT_FRAMES_LOC_1AE2E.saturating_sub(load_complete)
    })
}

fn recording_oracle_lava_water_handoff_passes_loc_1223c(
    level_id: LevelId,
    reload_delay: u16,
) -> usize {
    // Oracle PC traces at loc_12240 (immediately after the loc_1223C
    // Hibernate_Object_1Frame return) expose the coroutine's d0 at scenario
    // frame 0. The recording helper's collapsed Load_InGame hand-off omits
    // three Mountain returns (d2=5) and ordinarily six Cave returns (d2=8).
    // Level $65's step-63 loader boundary omits fourteen Cave returns; its
    // resident worker is already at d0=2, so those returns cross two rotation
    // boundaries and land on the oracle's d0=7 render latch. Keep this at the
    // recording-oracle boundary: ordinary gameplay already advances the same
    // typed coroutine once per Execute_Objects pass.
    match (level_id, reload_delay) {
        (LevelId(0x65), 8) => 14,
        (_, 5) => 3,
        (_, 8) => 6,
        _ => 0,
    }
}

fn recording_oracle_time_frames_loc_3038a(level_id: LevelId, time_frames: u16) -> u16 {
    debug_level_entry_load_complete_step_loc_74e0(level_id).map_or(time_frames, |load_complete| {
        // ASM loc_74E0 -> Mode_Level loc_892 -> loc_3038A: level loading
        // blocks Mode_Level, so Time_Frames advances only after the loader
        // completes. The recording shim has already counted every one of the
        // 120 title-wait steps as live; retain only the first live tick after
        // the measured load-completion boundary.
        time_frames
            .wrapping_add(1)
            .wrapping_sub(load_complete as u16)
    })
}

fn should_restore_recorded_title_card_after_first_live(
    restore_after_first_live: bool,
    game_mode: GameMode,
) -> bool {
    restore_after_first_live && game_mode == GameMode::InGame
}

/// How [`NativeRuntime::boot`] starts a session.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BootOptions {
    /// Use the oracle-recorded title-card hand-off timing, as the scenario
    /// runner does, so a shell traces frame-for-frame with it.
    pub recorded_title_card_handoff: bool,
    /// Boot through the menus and warp to this level; `None` leaves the
    /// runtime at power-on for the attract flow.
    pub warp: Option<LevelId>,
}

impl NativeRuntime {
    /// Production entry point for a shell: apply `options` to a freshly
    /// constructed runtime, then drive it with [`step_frame`](Self::step_frame).
    pub fn boot(&mut self, options: BootOptions) {
        self.set_recorded_title_card_handoff_delay(options.recorded_title_card_handoff);
        if let Some(level) = options.warp {
            self.boot_to_level(level);
        }
    }

    /// Boot through menus to level 0, then advance to `target` using the
    /// level skip mechanism. Used by sweep infrastructure to reach any level
    /// without encoding warp buttons in the input script.
    ///
    /// Boot timing matches the oracle helper exactly: 720 frames with
    /// Start presses at frames 120, 200, 280, 360, 460.
    pub fn boot_to_level(&mut self, target: LevelId) {
        self.boot_to_level_with_warp_lifecycle(target, false);
    }

    /// The settled-recording-warp bootstrap the scenario runner and the state
    /// sweep use. Public so the #442 witness gate can drive the exact same
    /// path when comparing native frame-0 state against the measured
    /// oracle-settle-shell witnesses.
    pub fn boot_to_level_for_recording_oracle(&mut self, target: LevelId) {
        self.set_recorded_title_card_handoff_delay(true);
        self.boot_to_level_with_warp_lifecycle(target, true);
    }

    fn boot_to_level_with_warp_lifecycle(
        &mut self,
        target: LevelId,
        use_recording_oracle_lifecycle: bool,
    ) {
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

        if compatibility_boot_waits_for_ingame(use_recording_oracle_lifecycle) {
            // The boot-title oracle now keeps the Level 1 title-card shell
            // hidden through frame 719, so a fixed 720-frame bootstrap no
            // longer implies that live gameplay has started. Compatibility
            // callers wait for that hand-off; the recording oracle instead
            // writes Current_LevelID immediately and keeps the hidden shell.
            for _ in 0..720 {
                if matches!(self.boot_state, BootState::InGame) {
                    break;
                }
                self.step_frame(idle);
            }
        }
        let used_debug_warp = self.current_level != target;
        if !used_debug_warp && use_recording_oracle_lifecycle {
            // ASM loc_1AE2E -> loc_6E2: the direct title route waits before
            // consuming Start, then retains Load_InGame's live-mode shell.
            // The recorded input latch needs release after the pressed edge.
            for _ in 0..RECORDING_DEBUG_TITLE_WAIT_FRAMES_LOC_1AE2E {
                self.step_frame(idle);
            }
            self.step_frame(start);
            for _ in 0..RECORDING_DEBUG_TITLE_WAIT_FRAMES_LOC_1AE2E {
                if self.trace_frame().game_mode == GameMode::InGame {
                    break;
                }
                self.step_frame(idle);
            }
            self.prior_input = idle;
            self.previous_input = idle;
            return;
        }
        if used_debug_warp {
            if use_recording_oracle_lifecycle {
                // ASM loc_74E0 re-enters loc_6E2/Load_TitleCard after the
                // debug level-skip input. The recording oracle waits 60 live
                // frames, ticks the skip, then gives the ordinary mode-$0008
                // title-card lifecycle 120 frames before pressing Start.
                // Its helper first writes Current_LevelID to target - 1 so
                // LevelSkip_Cheat's addq.w #1 selects the requested level.
                self.current_level = LevelId(target.0.wrapping_sub(1));
                for _ in 0..60 {
                    self.step_frame(idle);
                }
                self.step_frame(idle);
                self.begin_debug_level_entry_loc_74e0(target);
                // ASM loc_74E0 (#442): the recording bootstrap replays the
                // ROM's live shell — the level ages behind the never-shown
                // title card, so frame-loop subsystems (murder wall) run.
                self.debug_level_entry_runs_live_shell_loc_74e0 = true;
                for _ in 0..RECORDING_DEBUG_TITLE_WAIT_FRAMES_LOC_1AE2E {
                    self.step_frame(idle);
                }
                self.step_frame(start);
                self.prior_input = FrameInput::default();
                self.previous_input = FrameInput::default();
                self.recording_oracle_debug_lifecycle_completed = true;
            } else {
                self.pending_level_skip = Some(target);
                self.step_frame(idle);
                self.pending_level_skip_resume = true;
                self.step_frame(idle);
                for _ in 0..LEVEL_SKIP_RESUME_TOTAL_FRAMES {
                    self.step_frame(idle);
                }
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
        if boot_boundary_rebuilds_pristine_level(used_debug_warp, use_recording_oracle_lifecycle) {
            self.progression.number_hitpoints = self.progression.current_form.max_hitpoints();
            self.just_received_damage = false;
        } else {
            // ASM loc_74E0 (#421): the recording debug lifecycle seeds
            // Number_Hitpoints in Load_InGame BEFORE the hidden title window,
            // and the shell's live Mode_Level passes can spend it — 0x61's
            // li=0 Scorpion walks through Kid mid-shell and the ROM keeps
            // that hit (oracle frame 0 reads Number_Hitpoints=1 with
            // Just_received_damage clear after the loc_BC54 flash). The
            // boundary must not restore HP or clear the flag here; the flash
            // countdown already ticks through the shell's live frames.
        }
        self.death_shell_snapshot = None;
        self.level_complete_gfx_snapshot = None;
        if boot_boundary_rebuilds_pristine_level(used_debug_warp, use_recording_oracle_lifecycle) {
            self.reload_block_grid_for_level(self.current_level);
            self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(self.current_level));
        } else if self.block_grid.is_none() {
            self.reload_block_grid_for_level(self.current_level);
        } else {
            // ASM loc_74E0 (#420/#442): the recording debug lifecycle already
            // ran Load_InGame BEFORE the hidden title window, and the target
            // level then aged live behind it — including Driller.asm
            // loc_3BE78 block smashes eating Level_Layout cells. The oracle
            // never rebuilds Level_Layout at the title->live boundary
            // (0x55's frame-0 RAM keeps the drilled `00f9/00fa` words), so
            // reloading a lived-in grid here would resurrect the drilled
            // blocks and terrain-kill the walker the ROM keeps alive.
        }
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
        self.rebase_mode_level_tail_after_external_override_loc_2ffe8();
        self.refresh_active_roster();
        self.trace_frame()
    }

    fn settle_target_level_live_window(
        &mut self,
        level_id: LevelId,
        target_live_frames: usize,
        platform_rewind_passes: usize,
    ) {
        let idle = FrameInput::default();
        let mut stable_target_frames = 0usize;
        let mut platform_history = std::collections::VecDeque::new();

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
            if platform_rewind_passes > 0 {
                platform_history.push_back((
                    self.active_platforms.clone(),
                    self.active_platform_standing_on,
                    self.active_lift_platform_standing_on,
                    self.mode_level_platform_loader_work_loc_2744,
                ));
                if platform_history.len() > platform_rewind_passes {
                    platform_history.pop_front();
                }
            }
            self.step_frame(idle);
        }
        if platform_history.len() == platform_rewind_passes
            && let Some((platforms, standing_on, lift_standing_on, loader_work)) =
                platform_history.pop_front()
        {
            // ASM Mode_Level loc_80C runs Execute_ScriptedPlatforms/sub_23EA
            // once per live pass. Native's collapsed recording helper runs
            // two extra platform passes before the oracle's first input, so
            // retain that earlier Platform_RAM boundary without rewinding
            // unrelated object, HUD, palette, or RNG state.
            self.active_platforms = platforms;
            self.active_platform_standing_on = standing_on;
            self.active_lift_platform_standing_on = lift_standing_on;
            self.mode_level_platform_loader_work_loc_2744 = loader_work;
            self.active_platform_roster = self
                .active_platforms
                .iter()
                .map(ActivePlatform::snapshot)
                .collect();
        }
        if settled_warp_complete(stable_target_frames, target_live_frames) {
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
        if oracle_warp_primes_respawn_settle_enemy_loading(level_id) {
            self.prime_respawn_settle_enemy_loading_loc_368a8(level_id);
        }
        let target_live_frames = oracle_warp_target_live_frames(level_id);
        self.settle_target_level_live_window(level_id, target_live_frames, 0);
    }

    pub fn settle_recording_oracle_warp_live_window(&mut self, level_id: LevelId) {
        self.settle_recording_oracle_warp_live_window_with_target_frames(level_id, None);
    }

    /// #442: the hidden live window every settled-warp caller runs. All five
    /// harness callers route through here; `target_live_frames_override`
    /// remains as the census experiment entry point. The measured model is
    /// live: the debug lifecycle ages the level from its per-level load
    /// completion (`debug_level_entry_load_complete_step_loc_74e0`) and the
    /// residual window is the helper minimum; the frame-0 RNG witnesses
    /// (`gameplay_regression_settled_warp_frame0_witnesses_probe_levels`)
    /// gate the shell against `oracle-settle-shell.yaml`.
    fn settle_recording_oracle_warp_live_window_with_target_frames(
        &mut self,
        level_id: LevelId,
        target_live_frames_override: Option<usize>,
    ) {
        // ASM loc_1AE2E/loc_74E0 feed recording oracles through the helper's
        // `_settled_recording_oracle` branch. Level 0 comes straight through
        // the title-card gate and waits 60 stable live frames; Level 1 still
        // reaches the ROM's level-skip resume tail that native `boot_to_level`
        // shortcuts. Oracle settle tracing ends Level 1 at RNG offset $FFCB;
        // the first scenario frame then advances to $FFCF via Drips loc_3340E.
        let debug_lifecycle_completed = self.recording_oracle_debug_lifecycle_completed;
        let title_lifecycle_active = self.title_card_live_handoff_frames > 0;
        if debug_lifecycle_completed {
            // ASM loc_1AE2E -> loc_6E2 -> Load_InGame rebuilds the enemy
            // status table immediately before the target's first Mode_Level
            // pass. The typed debug title lifecycle has already created and
            // aged target objects through that same shell, so preserve those
            // live objects while rebuilding status/loading state.
            self.killed_enemies.clear();
            self.enemy_status_words.fill(0);
            self.enemy_loading_initial_pass_complete = false;
        }
        if !title_lifecycle_active
            && recording_settle_primes_respawn(level_id, debug_lifecycle_completed)
        {
            self.prime_respawn_settle_enemy_loading_loc_368a8(level_id);
        }
        if !title_lifecycle_active {
            self.prime_recording_warp_sprite_animation_clocks(debug_lifecycle_completed);
        }
        // #442: the measured window model — the debug lifecycle ages the
        // level from its per-level load completion, so the residual window is
        // the per-route table with no family adjustments.
        let target_live_frames = target_live_frames_override.unwrap_or_else(|| {
            recording_oracle_warp_target_live_frames(level_id, debug_lifecycle_completed)
        });
        let platform_rewind_passes = usize::from(level_id == LevelId(0x33)) * 2;
        self.settle_target_level_live_window(level_id, target_live_frames, platform_rewind_passes);
        if debug_lifecycle_completed {
            self.reconcile_recording_handoff_lion_fire_wait_loc_3e09e();
            // ASM Tank.asm loc_3C044/loc_3C0F0: the recording helper's
            // target-live row 0 is a capture-only duplicate of
            // post_title_start. Native's collapsed title handoff has already
            // consumed the Tank pass that the first scenario row presents.
            // Retain one Object hibernate at that adapter boundary while the
            // independent Gfx animations continue. Raw oracle settle row 59
            // and scenario frame 0 pin 0x1F slots 15/16/21 at
            // X196/388/324 then X197/389/325 (#509).
            for enemy in &self.active_enemies {
                if matches!(enemy.family, EnemyFamily::Tank | EnemyFamily::TankShooting) {
                    self.freshly_loaded_enemies_loc_3691e
                        .insert(ActiveEnemyKey::from_enemy(enemy), 1);
                }
            }
        }
        // ASM Execute_Objects loc_E1E: recording handoffs resume their saved
        // floating-prize worker after the Kid callback. Preserve that adapter
        // boundary order; ordinary `$1FFF` allocations retain before-Kid
        // order from Allocate_ObjectSlot loc_DE0.
        self.floating_prize_order_sub_e12 = crate::FloatingPrizeOrderSubE12::AfterPlayer;
        self.reconcile_recording_handoff_driller_gfx_clocks_loc_142e(3);
        self.reconcile_recording_handoff_driller_boundary_turn_loc_3c026();
        if level_id == LevelId(0x33) {
            // ASM Mode_Level -> GfxObjects_MoveAndAnimate loc_142E: the
            // recording helper's collapsed Level 0x33 shell over-ages the
            // Heady composite by two Gfx passes while its Object coroutine
            // already matches the oracle allocation frames. Reconcile only
            // that foreign-helper presentation boundary.
            self.rewind_heady_metal_recording_handoff_gfx_passes_loc_142e(2);
            self.seed_heady_recording_handoff_hud_diamond_loc_45ee();
        }
        // ASM loc_1AE2E -> Mode_Level -> Execute_Objects loc_E12: preserve
        // the first visible sample of any boss arrow which crossed the
        // recording helper's hidden-to-visible handoff. Later gameplay
        // allocations use the ordinary loc_3A0D2 coroutine path unchanged.
        self.defer_recording_handoff_boss_arrows_loc_e12();
        if debug_lifecycle_completed {
            self.time_frames_loc_3038a =
                recording_oracle_time_frames_loc_3038a(level_id, self.time_frames_loc_3038a);
        }
        // ASM Init_SpriteAttr_HUD loc_BDC0 seeds Time_SubSeconds to 1. A typed
        // debug lifecycle has already run every live loc_BE82 timer pass,
        // including transformation pauses, so retain its per-level result.
        // Legacy callers still need the #289 Level 0x07 oracle seed because
        // they collapse that lifecycle. The direct Level 0 route has its own
        // measured title-card boundary.
        if level_id == LevelId(0) {
            // ASM loc_3038A/Make_SpriteAttr_HUD loc_BE82: direct Level 0
            // oracle watches reach recording frame 0 at Time_Frames=$003D,
            // Time=2:59 and Time_SubSeconds=$003C. Seed the immediately prior
            // state here; the first scenario step performs both decrements.
            self.time_frames_loc_3038a = LEVEL_ZERO_RECORDING_TIME_FRAMES_BEFORE_FRAME_0_LOC_3038A;
            self.level_timer.seconds = LEVEL_ZERO_RECORDING_TIMER_SECONDS_BEFORE_FRAME_0_LOC_BE82;
            self.level_timer.sub_frames =
                LEVEL_ZERO_RECORDING_TIMER_SUBFRAMES_BEFORE_FRAME_0_LOC_BE82;
        } else if !debug_lifecycle_completed {
            self.level_timer.seconds = RECORDING_SETTLED_WARP_TIMER_SECONDS_LOC_BDC0;
            self.level_timer.sub_frames = RECORDING_SETTLED_WARP_TIMER_SUBFRAMES_LOC_BDC0;
        }
        // ASM loc_1AE2E's recording hand-off presents the current timer table
        // before the first scenario step. Keep loc_BEE8 aligned with either
        // the retained typed lifecycle or the legacy seed above.
        self.hud_timer_sprite_seconds_loc_bee8 = self.level_timer.seconds;
        self.rebase_mode_level_tail_after_external_override_loc_2ffe8();
        let hidden_object_passes = recording_oracle_warp_hidden_object_passes_loc_e12(level_id);
        // ASM Mode_Level -> Execute_Objects loc_E12: Load_InGame loc_11FB0
        // allocates the Island shore coroutine, whose loc_12098/loc_12110/
        // loc_12188 countdowns continue through every remaining hidden
        // title-wait pass. A zero pass count naturally performs no work.
        if !debug_lifecycle_completed {
            for _ in 0..hidden_object_passes {
                self.step_island_shore_cycle_loc_1202a();
            }
        }
        debug_assert_eq!(level_id, self.current_level);
        let target_reload_delay = self.level_header(level_id).map(|header| {
            crate::level::compute_rotate_palette_delay_loc_1223A(u16::from(header.level_theme))
                .delay_frames
        });
        let lava_water_handoff_passes = target_reload_delay.map_or(0, |reload_delay| {
            recording_oracle_lava_water_handoff_passes_loc_1223c(level_id, reload_delay)
        });
        // ASM RotatePalette_Lava_Water loc_1223A/loc_1223C: preserve the
        // object-local d0/d2 cadence across the collapsed recording hand-off,
        // including any live Palette_Buffer rotation it performs.
        if self
            .rotate_palette_lava_water_loc_1223c
            .is_some_and(|state| {
                state.owner_level == level_id && Some(state.reload_delay_d2) == target_reload_delay
            })
        {
            for _ in 0..lava_water_handoff_passes {
                self.step_rotate_palette_lava_water_loc_1223c();
            }
        }
        self.recording_oracle_settle_completed = true;
    }
}

/// Builder for running scripted scenarios with configurable options.
pub struct ScriptRunner {
    assets: AssetManifest,
    warp_to_level: Option<u8>,
    settle_warp: Option<SettleWarpMode>,
    physics_order: Option<PhysicsOrder>,
    trace_physics: bool,
    recorded_title_card_handoff_delay: bool,
    probe_start_position: Option<(u16, u16)>,
    probe_start_velocity: Option<(i32, i32)>,
    probe_camera_position: Option<(u16, u16)>,
    playtest_form: Option<PlayerForm>,
    initial_diamond_count: Option<u16>,
    initial_lives: Option<u16>,
    asset_source: Option<Arc<dyn AssetSource>>,
    restore_recorded_title_card_after_first_live: bool,
}

impl ScriptRunner {
    pub fn new(assets: AssetManifest) -> Self {
        Self {
            assets,
            warp_to_level: None,
            settle_warp: None,
            physics_order: None,
            trace_physics: false,
            recorded_title_card_handoff_delay: true,
            probe_start_position: None,
            probe_start_velocity: None,
            probe_camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
            asset_source: None,
            restore_recorded_title_card_after_first_live: false,
        }
    }

    /// Inject the [`AssetSource`] used for generated-bundle reads. Defaults to a
    /// native [`FsAssetSource`](wildside_data::FsAssetSource) over the manifest's
    /// `output_root`; the determinism test uses this to drive an in-memory source.
    #[must_use]
    pub fn with_asset_source(mut self, source: Arc<dyn AssetSource>) -> Self {
        self.asset_source = Some(source);
        self
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
    pub fn direct_title_card_handoff(mut self) -> Self {
        self.recorded_title_card_handoff_delay = false;
        self
    }

    #[must_use]
    pub fn direct_initial_title_card_handoff(mut self) -> Self {
        self.recorded_title_card_handoff_delay = false;
        self.restore_recorded_title_card_after_first_live = true;
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
        self.run_inner(script, frame_count, false).1
    }

    /// Runs the scenario and returns the visible VDP state immediately before
    /// scenario frame zero alongside the post-update trace timeline.
    pub fn run_with_initial_render_latch(
        self,
        script: &[u8],
        frame_count: usize,
    ) -> (NativeTraceFrame, Vec<NativeTraceFrame>) {
        let (initial_render_latch, trace, _) = self.run_inner(script, frame_count, false);
        (initial_render_latch, trace)
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
        let (_, trace, vdp_snapshots) = self.run_inner(script, frame_count, true);
        (trace, vdp_snapshots)
    }

    /// Replay a scenario while visiting each frame without retaining the
    /// complete trace or cloning each frame's VDP memory.
    ///
    /// The visitor receives the scenario-relative index, the post-step
    /// runtime, the pre-step trace used by the renderer's VDP latch, and the
    /// post-step trace. Returning an error stops the replay immediately.
    pub fn try_for_each_frame<E, F>(
        self,
        script: &[u8],
        frame_count: usize,
        mut visitor: F,
    ) -> Result<(), E>
    where
        F: FnMut(usize, &NativeRuntime, &NativeTraceFrame, &NativeTraceFrame) -> Result<(), E>,
    {
        let (mut runtime, delay_recording_input, mut restore_title_card_handoff) =
            self.into_runtime();
        for frame_index in 0..frame_count {
            let previous_trace = runtime.trace_frame();
            let input = script_frame_input(script, frame_index, delay_recording_input);
            let trace = runtime.step_frame(input);
            restore_recorded_title_card_handoff_if_needed(
                &mut runtime,
                &mut restore_title_card_handoff,
                trace.game_mode,
            );
            visitor(frame_index, &runtime, &previous_trace, &trace)?;
        }
        Ok(())
    }

    /// Replay a scenario while allowing an execution shell to advance the
    /// persistent hardware timeline at each live gameplay VBlank.
    ///
    /// The scheduler runs before the active raster owner: `sub_E12` during
    /// ordinary `Mode_Level`, or the next synchronous `sub_8E0` pass during
    /// `Kid_Transform`. The visitor sees the resulting post-step trace and
    /// the same pre-step render latch as [`Self::try_for_each_frame`].
    pub fn try_for_each_frame_with_raster_scheduler<E, S, F>(
        self,
        script: &[u8],
        frame_count: usize,
        mut scheduler: S,
        mut visitor: F,
    ) -> Result<(), E>
    where
        S: FnMut(usize, ModeLevelVblankWorkBeforeSubE12) -> Option<vdp::ModeLevelRasterSeedSubE12>,
        F: FnMut(usize, &NativeRuntime, &NativeTraceFrame, &NativeTraceFrame) -> Result<(), E>,
    {
        let (mut runtime, delay_recording_input, mut restore_title_card_handoff) =
            self.into_runtime();
        for frame_index in 0..frame_count {
            let previous_trace = runtime.trace_frame();
            let input = script_frame_input(script, frame_index, delay_recording_input);
            let trace = runtime.step_frame_with_mode_level_raster_scheduler(input, &mut |work| {
                scheduler(frame_index, work)
            });
            restore_recorded_title_card_handoff_if_needed(
                &mut runtime,
                &mut restore_title_card_handoff,
                trace.game_mode,
            );
            visitor(frame_index, &runtime, &previous_trace, &trace)?;
        }
        Ok(())
    }

    fn run_inner(
        self,
        script: &[u8],
        frame_count: usize,
        capture_vdp: bool,
    ) -> (
        NativeTraceFrame,
        Vec<NativeTraceFrame>,
        Vec<Option<vdp::VdpState>>,
    ) {
        let (mut runtime, delay_recording_input, mut restore_title_card_handoff) =
            self.into_runtime();
        // ASM Mode_Level loc_901 presents this already-built sprite/scroll/HUD
        // state before the first scenario step produces its post-update trace.
        let initial_render_latch = runtime.trace_frame();
        let mut trace = Vec::with_capacity(frame_count);
        let mut vdp_snapshots = if capture_vdp {
            Vec::with_capacity(frame_count)
        } else {
            Vec::new()
        };
        for frame_index in 0..frame_count {
            let input = script_frame_input(script, frame_index, delay_recording_input);
            let snapshot = runtime.step_frame(input);
            restore_recorded_title_card_handoff_if_needed(
                &mut runtime,
                &mut restore_title_card_handoff,
                snapshot.game_mode,
            );
            trace.push(snapshot);
            if capture_vdp {
                // step_frame can early-return before update_vdp_state (lib.rs:2546,
                // the DemoPlay no-input path), so the captured snapshot then
                // reflects the prior frame - identical to what play-native
                // observes, so viewer/player parity holds.
                vdp_snapshots.push(runtime.vdp_state().cloned());
            }
        }
        (initial_render_latch, trace, vdp_snapshots)
    }

    fn into_runtime(self) -> (NativeRuntime, bool, bool) {
        let mut runtime = match self.asset_source {
            Some(source) => {
                NativeRuntime::with_generated_level_layout_with_source(self.assets, source)
            }
            None => NativeRuntime::with_generated_level_layout(self.assets),
        };
        runtime.set_recorded_title_card_handoff_delay(self.recorded_title_card_handoff_delay);
        if let Some(order) = self.physics_order {
            runtime.set_physics_order(order);
        }
        if self.trace_physics {
            runtime.set_trace_physics(true);
        }
        if let Some(level_id) = self.warp_to_level {
            let level_id = LevelId(level_id);
            if self.settle_warp == Some(SettleWarpMode::RecordingOracle) {
                runtime.boot_to_level_for_recording_oracle(level_id);
            } else {
                runtime.boot_to_level(level_id);
            }
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
        (
            runtime,
            delay_recording_input,
            self.restore_recorded_title_card_after_first_live,
        )
    }
}

fn script_frame_input(
    script: &[u8],
    frame_index: usize,
    delay_recording_input: bool,
) -> FrameInput {
    // ASM Mode_Level (kid.asm:877-917) executes objects/player logic before
    // ReadJoypad, so the settled recording oracle's script byte for frame N is
    // only visible to gameplay on traced frame N+2.
    let input_index = if delay_recording_input {
        frame_index.checked_sub(1)
    } else {
        Some(frame_index)
    };
    input_index
        .and_then(|index| script.get(index))
        .copied()
        .map(FrameInput::from_mask)
        .unwrap_or_default()
}

fn restore_recorded_title_card_handoff_if_needed(
    runtime: &mut NativeRuntime,
    restore_title_card_handoff: &mut bool,
    game_mode: GameMode,
) {
    if should_restore_recorded_title_card_after_first_live(*restore_title_card_handoff, game_mode) {
        runtime.set_recorded_title_card_handoff_delay(true);
        *restore_title_card_handoff = false;
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

pub fn run_script_with_direct_title_card_handoff(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
) -> Vec<NativeTraceFrame> {
    ScriptRunner::new(assets)
        .direct_title_card_handoff()
        .run(script, frame_count)
}

pub fn run_script_with_direct_initial_title_card_handoff(
    assets: AssetManifest,
    script: &[u8],
    frame_count: usize,
) -> Vec<NativeTraceFrame> {
    ScriptRunner::new(assets)
        .direct_initial_title_card_handoff()
        .run(script, frame_count)
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
#[expect(
    clippy::disallowed_methods,
    reason = "tests read opt-in environment switches"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn settled_warp_guard_requires_live_kid_matching_level_and_unpaused() {
        let mut runtime = NativeRuntime::new(minimal_manifest());

        runtime.boot_state = BootState::InGame;
        runtime.current_level = LevelId(3);
        runtime.paused = false;
        runtime.opening_slice = Some(runtime.level_respawn_slice(LevelId(3)));
        assert!(is_live_unpaused_target_level(&runtime, LevelId(3)));

        let slice = runtime.opening_slice.take();
        assert!(!is_live_unpaused_target_level(&runtime, LevelId(3)));
        runtime.opening_slice = slice;

        runtime.paused = true;
        assert!(!is_live_unpaused_target_level(&runtime, LevelId(3)));

        runtime.paused = false;
        assert!(!is_live_unpaused_target_level(&runtime, LevelId(4)));

        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        assert!(!is_live_unpaused_target_level(&runtime, LevelId(3)));
    }

    #[test]
    fn recording_title_shell_does_not_load_enemies_loc_80c() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.set_recorded_title_card_handoff_delay(true);
        for frame in 0..720 {
            runtime.step_frame(FrameInput {
                start: BOOT_TO_LEVEL_START_FRAMES.contains(&frame),
                ..FrameInput::default()
            });
        }
        // ASM GameModesArray selects Mode_Standard for the title card. It
        // burns one Execute_Objects longword but never calls loc_80C's enemy
        // loader. The recording helper writes Current_LevelID before waiting
        // 60 frames; that register change must not allocate the new layout.
        // Oracle #506 pre_skip_wait rows 1..60 retain only title workers.
        runtime.current_level = LevelId(0x54);
        let mut expected_rng = runtime.rom_random.clone();
        for step in 1..=60 {
            expected_rng.next_long();
            let frame = runtime.step_frame(FrameInput::default());
            assert_eq!(frame.game_mode, GameMode::LevelTransition);
            assert_eq!(runtime.rom_random, expected_rng, "title wait {step}");
            assert!(runtime.active_enemies.is_empty(), "title wait {step}");
        }
    }

    #[test]
    fn recording_title_bootstrap_preserves_loader_rng_freeze_loc_1ae2e() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level_for_recording_oracle(LevelId(0));
        // ASM loc_1AE2E -> Load_InGame / Execute_Objects: oracle settle
        // rows 0..68 retain this complete ring; row 69 draws one longword.
        // ROM 28b904000b2863b6760531807760b571f1a5fc1d, settled level 0.
        let ring = [
            0xbf, 0x7b, 0x08, 0xc0, 0x56, 0x34, 0x36, 0x44, 0x0e, 0x22, 0xfb, 0xc1, 0x5b, 0x91,
            0x50, 0x33, 0xd2, 0xbe, 0x57, 0xb6, 0x27, 0x9c, 0x99, 0x04, 0x31, 0x36, 0xb8, 0xd7,
            0xf3, 0x85, 0xb0, 0x30, 0x46, 0xe7, 0xf3, 0xc1, 0x61, 0x0a, 0x81, 0x99, 0x53, 0xbb,
            0x28, 0xf1, 0x1a, 0x7d, 0x81, 0xca, 0xe8, 0xc1, 0x82, 0xd9, 0x69, 0xa2, 0x0f,
        ];
        assert_eq!(runtime.rom_random.offset_word(), 0xffdb);
        assert_eq!(runtime.rom_random.ring_bytes(), ring);
        for elapsed in 1..=68 {
            let frame = runtime.step_frame(FrameInput::default());
            assert_eq!(runtime.rom_random.offset_word(), 0xffdb, "step {elapsed}");
            assert_eq!(runtime.rom_random.ring_bytes(), ring, "step {elapsed}");
            assert_eq!(
                is_live_unpaused_target_level(&runtime, LevelId(0)),
                elapsed < 27,
                "ASM Load_InGame clears Kid before reloading the live roster"
            );
            if elapsed == 68 {
                assert_eq!(frame.kid_x_pos, Some(0));
                assert_eq!(frame.number_objects, 4);
            }
        }
        let frame = runtime.step_frame(FrameInput::default());
        let mut expected = RomRandomState::from_raw_parts(ring, 0xffdb);
        expected.next_long();
        assert_eq!(runtime.rom_random, expected);
        assert_eq!(frame.kid_x_pos, Some(32 << 16));
        assert_eq!(frame.number_objects, 5);
        assert!(is_live_unpaused_target_level(&runtime, LevelId(0)));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0));
        let first_recorded = runtime.step_frame(FrameInput::default());
        assert_eq!(first_recorded.rng_offset, Some(0xfff3));
        assert_eq!(first_recorded.hud_diamond_frame_offset, Some(6));
        assert_eq!(first_recorded.hud_life_frame_offset, Some(0));
        // ASM loc_720/sub_44B0/sub_44DC: oracle frame-zero RAM FA58..FA61.
        let hud = runtime.hud_animation_state;
        assert_eq!(
            [
                hud.diamond_countdown_fb58,
                hud.life_countdown_fb59,
                hud.clock_countdown_fb5a,
                hud.coin_countdown_fb5b,
                hud.flag_countdown_fb5c,
                hud.diamond_frame_offset_fb5d,
                hud.life_frame_offset_fb5e,
                hud.clock_frame_offset_fb5f,
                hud.coin_frame_offset_fb60,
                hud.flag_frame_offset_fb61
            ],
            [0x06, 0x21, 0x22, 0x02, 0x01, 0x06, 0x00, 0x00, 0x00, 0x08]
        );
    }

    #[test]
    fn settled_warp_attempt_budget_covers_live_window_plus_hidden_bootstrap_tail() {
        assert_eq!(SETTLED_WARP_MAX_ATTEMPT_FRAMES, 828);
    }

    #[test]
    fn settled_warp_complete_requires_nonzero_target_and_full_stable_window() {
        assert!(
            !settled_warp_complete(0, 0),
            "settled helpers must not mark a zero-frame target complete; otherwise the spawn gate can be set without observing a live target level"
        );
        assert!(
            !settled_warp_complete(59, 60),
            "ASM recording settle waits for the full stable live window, not one frame short"
        );
        assert!(
            settled_warp_complete(60, 60),
            "ASM loc_1AE2E recording settle should complete exactly on the required stable live frame count"
        );
        assert!(
            settled_warp_complete(61, 60),
            "additional stable live frames should remain complete once the window has been met"
        );
    }

    #[test]
    fn oracle_warp_respawn_prime_is_level_one_only() {
        assert!(!oracle_warp_primes_respawn_settle_enemy_loading(LevelId(0)));
        assert!(oracle_warp_primes_respawn_settle_enemy_loading(LevelId(1)));
        assert!(!oracle_warp_primes_respawn_settle_enemy_loading(LevelId(2)));
    }

    #[test]
    fn boot_and_recording_settle_lifecycle_predicates_pin_all_state_combinations() {
        assert!(compatibility_boot_waits_for_ingame(false));
        assert!(!compatibility_boot_waits_for_ingame(true));

        assert!(boot_boundary_rebuilds_pristine_level(false, false));
        assert!(boot_boundary_rebuilds_pristine_level(false, true));
        assert!(boot_boundary_rebuilds_pristine_level(true, false));
        assert!(!boot_boundary_rebuilds_pristine_level(true, true));

        assert!(recording_settle_primes_respawn(LevelId(0), false));
        assert!(recording_settle_primes_respawn(LevelId(0), true));
        assert!(recording_settle_primes_respawn(LevelId(1), false));
        assert!(!recording_settle_primes_respawn(LevelId(1), true));
        assert!(!recording_settle_primes_respawn(LevelId(2), false));
        assert!(!recording_settle_primes_respawn(LevelId(2), true));
    }

    #[test]
    fn oracle_warp_target_live_frames_pin_level_specific_windows() {
        assert_eq!(
            oracle_warp_target_live_frames(LevelId(0)),
            ORACLE_SETTLED_WARP_LIVE_FRAMES,
            "Level 0 full-frame settle uses the ordinary oracle stable live window"
        );
        assert_eq!(
            oracle_warp_target_live_frames(LevelId(1)),
            ORACLE_SETTLED_WARP_LIVE_FRAMES + 1,
            "Level 1 full-frame settle includes the extra resume-tail frame before probes"
        );
        assert_eq!(
            oracle_warp_target_live_frames(LevelId(2)),
            1,
            "non-opening full-frame oracle warps only need one live target frame after direct boot"
        );
    }

    #[test]
    fn recording_oracle_warp_target_live_frames_is_the_measured_helper_minimum() {
        // #442: the debug title lifecycle ages every measured level from its
        // per-level load completion, so the residual stable window is the
        // helper minimum (census v2 measures exactly 60 on all 104 warped
        // level IDs). The old 0x01/0x06/0x08/0x1B/0x3E/0x46/0x64 tails
        // and Driller/boss/Drips family adjustments modelled the missing load
        // duration and are deleted.
        for level in [
            0x00u8, 0x01, 0x06, 0x08, 0x1B, 0x3E, 0x3F, 0x46, 0x55, 0x64, 0x12,
        ] {
            assert_eq!(
                recording_oracle_warp_target_live_frames(LevelId(level), true),
                RECORDING_SETTLED_WARP_LIVE_FRAMES,
                "level {level:#04x}: the debug-lifecycle residual window is the measured helper minimum"
            );
        }
    }

    #[test]
    fn recording_oracle_warp_legacy_windows_pin_every_special_route_loc_1ae2e() {
        // ASM loc_1AE2E/loc_74E0: before the typed debug lifecycle completes,
        // these are the oracle-measured stable-live tails retained for legacy
        // callers. Every other level uses the 60-frame helper minimum.
        let special = [
            (0x00, 60),
            (0x01, 107),
            (0x06, 84),
            (0x08, 101),
            (0x1B, 112),
            (0x3E, 107),
        ];
        for (level, expected) in special {
            assert_eq!(
                recording_oracle_warp_target_live_frames(LevelId(level), false),
                expected,
                "level {level:#04x}: preserve its pre-loc_74E0 oracle tail"
            );
        }
        assert_eq!(
            recording_oracle_warp_target_live_frames(LevelId(0x52), false),
            RECORDING_SETTLED_WARP_LIVE_FRAMES,
            "the 0x52 59-frame correction applies only after the debug lifecycle"
        );
        assert_eq!(
            recording_oracle_warp_target_live_frames(LevelId(0x52), true),
            RECORDING_SETTLED_WARP_LIVE_FRAMES - 1,
            "oracle settle run 1786728817 pins 0x52 at target-live step 59"
        );
    }

    /// #442: the measured load-completion table drives the hidden Mode_Level
    /// boundary. Pin the witness anchors: 0x48 is the level the legacy fixed
    /// boundary (71 = 72 - 1) already modelled exactly; 0x30/0x60 are the
    /// dead-end levels whose loads complete at steps 75/62 (run 1786640409).
    #[test]
    fn debug_level_entry_load_complete_table_pins_witness_anchors() {
        assert_eq!(
            debug_level_entry_load_complete_step_loc_74e0(LevelId(0x48)),
            Some(72)
        );
        assert_eq!(
            debug_level_entry_load_complete_step_loc_74e0(LevelId(0x30)),
            Some(75)
        );
        assert_eq!(
            debug_level_entry_load_complete_step_loc_74e0(LevelId(0x60)),
            Some(62)
        );
        assert_eq!(
            debug_level_entry_load_complete_step_loc_74e0(LevelId(0)),
            None,
            "level 0 takes the title path and has no debug-warp load freeze"
        );
    }

    #[test]
    fn recording_warp_time_frames_tracks_debug_loader_completion_loc_3038a() {
        // ASM loc_74E0 -> Mode_Level loc_892 -> loc_3038A: loader frames do
        // not execute the level clock. Oracle frame-60 probes read 171 on
        // Level 0x02, 169 on 0x03, and 174 on the 0x07 anchor.
        assert_eq!(
            recording_oracle_time_frames_loc_3038a(LevelId(0x02), 241),
            171
        );
        assert_eq!(
            recording_oracle_time_frames_loc_3038a(LevelId(0x03), 241),
            169
        );
        assert_eq!(
            recording_oracle_time_frames_loc_3038a(LevelId(0x07), 241),
            174
        );
        assert_eq!(
            recording_oracle_time_frames_loc_3038a(LevelId(0), 174),
            174,
            "direct Level 0 startup does not use the debug loader table"
        );
    }

    #[test]
    fn recording_warp_primes_lava_water_handoff_from_oracle_loc_1223c() {
        assert_eq!(
            recording_oracle_lava_water_handoff_passes_loc_1223c(LevelId(0x03), 5),
            3,
            "Mountain loc_1223C d0 is three hibernate returns behind the oracle recording boundary"
        );
        assert_eq!(
            recording_oracle_lava_water_handoff_passes_loc_1223c(LevelId(0x04), 8),
            6,
            "Cave loc_1223C d0 is six hibernate returns behind the oracle recording boundary"
        );
        assert_eq!(
            recording_oracle_lava_water_handoff_passes_loc_1223c(LevelId(0x04), 7),
            0,
            "unrecognised coroutine delays must not gain an inferred recording shim"
        );
        assert_eq!(
            recording_oracle_lava_water_handoff_passes_loc_1223c(LevelId(0x65), 8),
            14,
            "Level $65 oracle CRAM frames 60/359 and d0=7 expose fourteen omitted Cave loc_1223C returns after its step-63 loader boundary"
        );
    }

    #[test]
    fn debug_level_entry_load_complete_table_pins_every_measured_level_loc_74e0() {
        // oracle-settle-census derivation v2, all 105 level IDs, plus the
        // #509 level 0x1F/0x4B title-wait traces. An exhaustive
        // domain assertion prevents an individual match arm silently falling
        // back to `None` while the three historical anchors remain green.
        let expected = [
            (0x49, 56),
            (0x4d, 57),
            (0x52, 57),
            (0x4a, 58),
            (0x4f, 58),
            (0x50, 58),
            (0x5d, 58),
            (0x5e, 58),
            (0x67, 58),
            (0x68, 58),
            (0x59, 59),
            (0x53, 60),
            (0x58, 60),
            (0x4b, 60),
            (0x5f, 60),
            (0x55, 61),
            (0x5a, 61),
            (0x66, 61),
            (0x57, 62),
            (0x60, 62),
            (0x33, 63),
            (0x4c, 63),
            (0x65, 63),
            (0x54, 64),
            (0x56, 64),
            (0x4e, 64),
            (0x5c, 64),
            (0x61, 64),
            (0x51, 65),
            (0x5b, 65),
            (0x2a, 66),
            (0x1b, 67),
            (0x07, 68),
            (0x10, 68),
            (0x26, 68),
            (0x16, 69),
            (0x1e, 69),
            (0x27, 69),
            (0x01, 70),
            (0x0b, 70),
            (0x1d, 70),
            (0x64, 70),
            (0x02, 71),
            (0x05, 71),
            (0x0c, 71),
            (0x1a, 71),
            (0x1c, 71),
            (0x25, 71),
            (0x39, 71),
            (0x62, 71),
            (0x48, 72),
            (0x63, 72),
            (0x03, 73),
            (0x2b, 73),
            (0x0a, 74),
            (0x0e, 74),
            (0x11, 74),
            (0x15, 74),
            (0x18, 74),
            (0x24, 74),
            (0x3a, 74),
            (0x3e, 74),
            (0x09, 75),
            (0x1f, 75),
            (0x23, 75),
            (0x28, 75),
            (0x2f, 75),
            (0x30, 75),
            (0x38, 75),
            (0x3c, 75),
            (0x41, 75),
            (0x45, 75),
            (0x47, 75),
            (0x06, 76),
            (0x0d, 76),
            (0x17, 76),
            (0x2c, 76),
            (0x2e, 76),
            (0x31, 76),
            (0x37, 76),
            (0x44, 76),
            (0x46, 76),
            (0x19, 77),
            (0x0f, 77),
            (0x3d, 77),
            (0x40, 77),
            (0x42, 77),
            (0x22, 78),
            (0x2d, 78),
            (0x36, 78),
            (0x14, 79),
            (0x20, 79),
            (0x21, 79),
            (0x34, 79),
            (0x35, 79),
            (0x43, 79),
            (0x08, 80),
            (0x13, 80),
            (0x29, 80),
            (0x32, 80),
            (0x04, 81),
            (0x3b, 82),
            (0x12, 84),
            (0x3f, 85),
        ];
        let expected = expected
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(expected.len(), 104, "all debug-warp level IDs are pinned");
        for level in 0u8..=0x68 {
            assert_eq!(
                debug_level_entry_load_complete_step_loc_74e0(LevelId(level)),
                expected.get(&level).copied(),
                "level {level:#04x}: loc_74E0 completion must match the checked-in census"
            );
        }
    }

    #[test]
    fn recording_oracle_warp_replays_measured_hidden_object_passes_loc_e12() {
        assert_eq!(
            recording_oracle_warp_hidden_object_passes_loc_e12(LevelId(0x07)),
            52,
            "oracle settle trace 1786744461 runs Mode_Level Execute_Objects from Level 0x07 load-complete step 68 through title-wait step 119"
        );
        assert_eq!(
            recording_oracle_warp_hidden_object_passes_loc_e12(LevelId(0)),
            0,
            "Level 0 takes the direct title path rather than the loc_74E0 debug-warp window"
        );
    }

    #[test]
    fn direct_initial_title_card_restore_waits_for_first_live_frame_loc_1ae2e() {
        assert!(
            !should_restore_recorded_title_card_after_first_live(false, GameMode::InGame),
            "ASM loc_1AE2E direct initial handoff only restores recorded delay when the one-shot restore flag is armed"
        );
        assert!(
            !should_restore_recorded_title_card_after_first_live(true, GameMode::LevelTransition),
            "ASM loc_1AE2E title-card warm-up must not restore recorded delay before gameplay starts"
        );
        assert!(
            !should_restore_recorded_title_card_after_first_live(true, GameMode::TitleScreen),
            "ASM Obj_TitleMenu loc_1BBD6 frames are still before the first live gameplay frame"
        );
        assert!(
            should_restore_recorded_title_card_after_first_live(true, GameMode::InGame),
            "ASM loc_1AE2E direct initial handoff restores recorded title-card delay after the first live frame"
        );
    }

    #[test]
    fn settle_recording_oracle_warp_resets_timer_without_cache() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.level_timer.seconds = 1;
        runtime.level_timer.sub_frames = 1;

        runtime.settle_recording_oracle_warp_live_window(LevelId(0));

        assert_eq!(
            (runtime.level_timer.seconds, runtime.level_timer.sub_frames),
            (180, 1),
            "ASM loc_3038A/Make_SpriteAttr_HUD loc_BE82 Level 0 recording settle seeds the state immediately before oracle frame 0 reaches 2:59/$3C"
        );
        assert_eq!(
            runtime.trace_frame().hud_timer_sprite_seconds_loc_bee8,
            Some(180),
            "ASM loc_1AE2E presents the initial 3:00 timer table built by Make_SpriteAttr_HUD loc_BEE8 at recording frame zero"
        );
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
    #[test]
    fn recording_debug_loader_resets_all_hud_clocks_before_live_pass() {
        // ASM loc_720 clears offsets; Load_InGame -> sub_44B0 seeds countdowns.
        // Oracle BLW2 settled recording RAM $FA58..$FA61 at frames 0 and 5.
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level_for_recording_oracle(LevelId(1));
        runtime.settle_recording_oracle_warp_live_window(LevelId(1));
        for frame in 0..=5 {
            runtime.step_frame(FrameInput::default());
            let hud = runtime.hud_animation_state;
            let bytes = [
                hud.diamond_countdown_fb58,
                hud.life_countdown_fb59,
                hud.clock_countdown_fb5a,
                hud.coin_countdown_fb5b,
                hud.flag_countdown_fb5c,
                hud.diamond_frame_offset_fb5d,
                hud.life_frame_offset_fb5e,
                hud.clock_frame_offset_fb5f,
                hud.coin_frame_offset_fb60,
                hud.flag_frame_offset_fb61,
            ];
            if frame == 0 {
                assert_eq!(bytes, [3, 72, 1, 5, 4, 6, 0, 6, 18, 2]);
            }
            if frame == 5 {
                assert_eq!(bytes, [4, 67, 68, 6, 5, 0, 0, 0, 0, 4]);
            }
        }
    }

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

    #[test]
    fn recording_robot_init_hibernates_before_first_walk_loc_3a884() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        // ASM Robot.asm Enemy03_Robot_Init loc_3A884 enters the shared
        // walker at loc_3C0D2/loc_3C026, whose first pass hibernates before
        // loc_3C04E/loc_3C0FA can move the body. ROM 28b904000b2863b6760531807760b571f1a5fc1d,
        // settled Crypt layout 11, oracle recording frames 0..5.
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level_for_recording_oracle(LevelId(0x41));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x41));
        for (frame, (x, rng)) in [
            (0x0095_0000, 0xFFFA),
            (0x0094_8000, 0xFFFE),
            (0x0094_0000, 0xFFCB),
            (0x0093_8000, 0xFFCF),
            (0x0093_0000, 0xFFD3),
            (0x0092_8000, 0xFFD7),
        ]
        .into_iter()
        .enumerate()
        {
            runtime.step_frame(FrameInput::default());
            let robot = runtime
                .active_enemies
                .iter()
                .find(|enemy| matches!(enemy.family, EnemyFamily::Robot))
                .expect("Crypt Robot layout 11 remains active");
            assert_eq!(robot.x_pos, x, "Robot body X at frame {frame}");
            assert_eq!(
                runtime.rom_random.offset_word(),
                rng,
                "RNG at frame {frame}"
            );
        }
    }

    #[test]
    fn recording_handoff_enemy_pass_probe_loc_e1e() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let level = LevelId(0x1f);
        let family = EnemyFamily::Tank;
        let expected = vec![
            (15, 0x00c5_0000, 0x0410_0000),
            (16, 0x0185_0000, 0x0420_0000),
            (21, 0x0145_0000, 0x04c0_0000),
        ];
        {
            let mut runtime = NativeRuntime::with_generated_level_layout(manifest.clone());
            runtime.boot_to_level_for_recording_oracle(level);
            runtime.settle_recording_oracle_warp_live_window(level);

            let before = runtime
                .active_enemies
                .iter()
                .filter(|enemy| enemy.family == family)
                .map(|enemy| (enemy.layout_index, enemy.x_pos, enemy.y_pos))
                .collect::<Vec<_>>();
            let player_before = runtime.opening_slice.map(|slice| slice.player);
            let rng_before = runtime.rom_random.offset_word();

            let mut enemy_pass = runtime.clone();
            enemy_pass.refresh_active_roster();
            let after = enemy_pass
                .active_enemies
                .iter()
                .filter(|enemy| enemy.family == family)
                .map(|enemy| (enemy.layout_index, enemy.x_pos, enemy.y_pos))
                .collect::<Vec<_>>();

            let mut full_frame = runtime.clone();
            full_frame.step_frame(FrameInput::default());
            let after_full_frame = full_frame
                .active_enemies
                .iter()
                .filter(|enemy| enemy.family == family)
                .map(|enemy| (enemy.layout_index, enemy.x_pos, enemy.y_pos))
                .collect::<Vec<_>>();

            // ASM Execute_Objects loc_E1E: isolate the enemy scheduler from
            // Kid and the frame shell. Native's collapsed handoff already
            // presents oracle frame-0 X197/389/325, so the saved Tank
            // hibernate holds those positions for the first scenario pass.
            assert_eq!(before, expected, "pre-pass roster for {level:?}");
            assert_eq!(after, expected, "isolated pass for {level:?}");
            assert_eq!(after_full_frame, after, "full-frame roster for {level:?}");
            assert_eq!(
                enemy_pass.opening_slice.map(|slice| slice.player),
                player_before
            );
            assert_eq!(enemy_pass.rom_random.offset_word(), rng_before);
        }
    }

    #[test]
    fn issue186_level07_recording_handoff_retains_lion_roar_loc_3e09e() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level_for_recording_oracle(LevelId(0x07));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x07));

        let frame = runtime.step_frame(FrameInput::default());
        let lion = runtime
            .active_enemies
            .iter()
            .find(|enemy| enemy.family == EnemyFamily::Lion && enemy.layout_index == 1)
            .expect("Level 0x07 lower Lion remains active at frame 0");

        assert_eq!(
            (lion.lion_state, lion.wait_timer),
            (LionState::Roaring, 18),
            "ASM Lion.asm loc_3E09E: preserve the oracle's frame-19 loc_3E0DA projectile-allocation boundary"
        );
        assert!(
            frame.active_projectile_details.is_empty(),
            "ASM Lion.asm loc_3E0DA must not allocate the projectile before the loc_3E09E fire-sprite wait completes"
        );
        assert_eq!(
            (frame.number_hitpoints, frame.just_received_damage),
            (2, false),
            "oracle frame 0 has no Lion projectile contact"
        );
        let lion_trace = frame
            .active_enemy_details
            .iter()
            .find(|enemy| enemy.layout_index == Some(1))
            .expect("Level 0x07 lower Lion has a trace GfxObject");
        assert_eq!(
            lion_trace.x_pos, 0x0205_8000,
            "ASM Enemy12_Lion_Init loc_3DF56 exposes the linked GfxObject position, not native's collision carrier"
        );

        for _ in 1..19 {
            let frame = runtime.step_frame(FrameInput::default());
            assert!(
                frame.active_projectile_details.is_empty(),
                "ASM Lion.asm loc_3E09E must retain the fire-sprite wait through oracle frame 18"
            );
        }
        let frame_19 = runtime.step_frame(FrameInput::default());
        assert_eq!(
            frame_19.active_projectile_details.len(),
            1,
            "ASM Lion.asm loc_3E0DA allocates the first Lion projectile on oracle frame 19"
        );
        assert_eq!(
            frame_19.active_projectile_details[0].x_pos, 0x01F5_0000,
            "ASM Lion.asm loc_3E12A/loc_3E180 places the first moved left-facing projectile at the oracle GfxObject X"
        );

        for oracle_frame in 20..28 {
            let frame = runtime.step_frame(FrameInput::default());
            assert_eq!(
                (frame.number_hitpoints, frame.just_received_damage),
                (2, false),
                "ASM GfxObjects_CollisionKid loc_1BCE only latches Lion projectile contact before its next object wake; oracle frame {oracle_frame} remains undamaged"
            );
        }
        let frame_28 = runtime.step_frame(FrameInput::default());
        assert_eq!(
            (frame_28.number_hitpoints, frame_28.just_received_damage),
            (1, true),
            "ASM Lion.asm loc_3E192 consumes the collision latched by GfxObjects_CollisionKid on oracle frame 28"
        );

        for _ in 29..46 {
            runtime.step_frame(FrameInput::default());
        }
        let lion_before_charge = runtime
            .active_enemies
            .iter()
            .find(|enemy| enemy.family == EnemyFamily::Lion && enemy.layout_index == 1)
            .expect("Level 0x07 lower Lion remains active before charge resumes");
        assert_eq!(lion_before_charge.x_pos, 0x0210_0000);

        runtime.step_frame(FrameInput::default());
        let lion_after_charge = runtime
            .active_enemies
            .iter()
            .find(|enemy| enemy.family == EnemyFamily::Lion && enemy.layout_index == 1)
            .expect("Level 0x07 lower Lion remains active when charge resumes");
        assert_ne!(
            lion_after_charge.x_pos, 0x0210_0000,
            "ASM Lion.asm loc_3E11E resumes the charge on oracle frame 46"
        );
    }

    #[test]
    fn issue186_live_lion_loader_holds_spawn_then_applies_ledge_gravity_loc_3df56_3e682() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut script = vec![0x04; 35];
        script.extend([0x38; 6]);
        script.extend([0x18; 56]);
        let frames = ScriptRunner::new(manifest)
            .warp_to_level(0x07)
            .settle_recording_warp()
            .run(&script, script.len());
        let lion_62 = frames[62]
            .active_enemy_details
            .iter()
            .find(|enemy| enemy.layout_index == Some(0))
            .expect("Level 0x07 upper Lion enters the live loader window");
        assert_eq!(
            (lion_62.x_pos, lion_62.y_pos),
            (0x02B8_0000, 0x018F_0000),
            "ASM Enemy12_Lion_Init loc_3DF56 exposes the linked GfxObject at its spawn position before the first walker step"
        );
        let lion_63 = frames[63]
            .active_enemy_details
            .iter()
            .find(|enemy| enemy.layout_index == Some(0))
            .expect("Level 0x07 upper Lion remains active after ledge entry");
        assert_eq!(
            (lion_63.x_pos, lion_63.y_pos, lion_63.y_velocity),
            (0x02B6_8000, 0x0190_0000, Some(0x0000_4000)),
            "ASM Lion.asm loc_3E682 falls through loc_3E77E and applies gravity on the first ledge-entry pass"
        );
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
    fn run_with_vdp_returns_one_trace_and_vdp_state_per_requested_frame_without_cache() {
        let manifest = minimal_manifest();
        let (trace, snapshots) = ScriptRunner::new(manifest).run_with_vdp(&[0, 0, 0], 3);

        assert_eq!(
            trace.len(),
            3,
            "ScriptRunner::run_with_vdp must preserve the requested trace frame count"
        );
        assert_eq!(
            snapshots.len(),
            3,
            "ScriptRunner::run_with_vdp must keep the VDP snapshot channel frame-aligned with the trace"
        );
        assert!(
            trace.iter().any(|frame| frame.frame_index != 0),
            "the returned trace must come from live runtime stepping, not a default-frame replacement"
        );
    }

    #[test]
    fn try_for_each_frame_streams_aligned_runtime_frames_without_collecting() {
        let manifest = minimal_manifest();
        let script = [0, 0, 0];
        let expected = ScriptRunner::new(manifest.clone()).run(&script, script.len());
        let mut visited = Vec::new();

        ScriptRunner::new(manifest)
            .try_for_each_frame(&script, script.len(), |index, runtime, previous, trace| {
                visited.push((
                    index,
                    previous.frame_index,
                    trace.frame_index,
                    runtime.trace_frame().authoritative_crc32(),
                    trace.authoritative_crc32(),
                ));
                Ok::<_, std::convert::Infallible>(())
            })
            .expect("stream frames");

        assert_eq!(visited.len(), script.len());
        assert_eq!(
            visited
                .iter()
                .map(|(_, _, _, _, trace_crc)| *trace_crc)
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(NativeTraceFrame::authoritative_crc32)
                .collect::<Vec<_>>(),
            "streaming replay must preserve ScriptRunner::run frame semantics"
        );
        assert!(
            visited
                .iter()
                .enumerate()
                .all(|(expected, (actual, _, _, runtime_crc, trace_crc))| {
                    expected == *actual && runtime_crc == trace_crc
                }),
            "the visitor must receive every post-step runtime frame in script order"
        );
    }

    #[test]
    fn try_for_each_frame_stops_on_visitor_error() {
        let manifest = minimal_manifest();
        let mut visited = Vec::new();

        let error = ScriptRunner::new(manifest)
            .try_for_each_frame(&[0; 4], 4, |index, _, _, _| {
                visited.push(index);
                if index == 1 { Err("stop") } else { Ok(()) }
            })
            .expect_err("visitor failure must stop the replay");

        assert_eq!(error, "stop");
        assert_eq!(visited, [0, 1]);
    }

    #[test]
    fn raster_scheduler_runner_executes_visitor_for_every_requested_frame() {
        let mut scheduled = Vec::new();
        let mut visited = Vec::new();
        ScriptRunner::new(minimal_manifest())
            .try_for_each_frame_with_raster_scheduler(
                &[0; 3],
                3,
                |index, _| {
                    scheduled.push(index);
                    None
                },
                |index, _, _, trace| {
                    visited.push((index, trace.frame_index));
                    Ok::<_, std::convert::Infallible>(())
                },
            )
            .expect("scheduled replay");

        assert!(
            scheduled.is_empty(),
            "boot frames do not yet own a Mode_Level raster pass"
        );
        assert_eq!(
            visited.iter().map(|(index, _)| *index).collect::<Vec<_>>(),
            [0, 1, 2]
        );
    }

    #[test]
    fn recording_runner_reports_delayed_input_and_restores_title_handoff_once_loc_1ae2e() {
        let runner = ScriptRunner::new(minimal_manifest())
            .warp_to_level(1)
            .settle_recording_warp()
            .direct_initial_title_card_handoff();
        let (mut runtime, delay_input, mut restore_handoff) = runner.into_runtime();
        assert_eq!(runtime.current_level, LevelId(1));
        assert!(runtime.recording_oracle_debug_lifecycle_completed);
        assert!(
            delay_input,
            "recording-oracle replay delays script input by one frame"
        );
        assert!(
            restore_handoff,
            "direct initial handoff arms the one-shot restore"
        );

        runtime.set_recorded_title_card_handoff_delay(false);
        restore_recorded_title_card_handoff_if_needed(
            &mut runtime,
            &mut restore_handoff,
            GameMode::InGame,
        );
        assert!(
            !restore_handoff,
            "the first live frame consumes the restore latch"
        );
        assert!(runtime.recorded_title_card_handoff_delay);
    }

    #[test]
    fn run_script_with_warp_returns_requested_live_frames_without_cache() {
        let manifest = minimal_manifest();
        let trace = run_script_with_warp(manifest, &[0, 0], 2, Some(0));

        assert_eq!(
            trace.len(),
            2,
            "run_script_with_warp must return exactly the requested number of frames"
        );
        assert!(
            trace.iter().all(|frame| frame.current_level == LevelId(0)),
            "run_script_with_warp should step the warped runtime, not return a default frame"
        );
        assert!(
            trace
                .iter()
                .any(|frame| frame.game_mode == GameMode::InGame),
            "boot_to_level should put the wrapped runner in gameplay for a level warp"
        );
    }

    #[test]
    fn enter_level_playtest_returns_the_live_target_trace_without_cache() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        let trace = runtime.enter_level_playtest(LevelId(0x12));

        assert_eq!(trace.current_level, LevelId(0x12));
        assert_eq!(trace.game_mode, GameMode::InGame);
        assert_eq!(
            trace.authoritative_crc32(),
            runtime.trace_frame().authoritative_crc32(),
            "the returned playtest trace must describe the mutated runtime, not a default frame"
        );
    }

    #[test]
    fn direct_title_card_handoff_wrapper_returns_requested_frames_without_cache() {
        let manifest = minimal_manifest();
        let trace = run_script_with_direct_title_card_handoff(manifest, &[0, 0], 2);

        assert_eq!(
            trace.len(),
            2,
            "ASM Obj_TitleMenu loc_1BBD6 direct title-card wrapper must delegate to ScriptRunner::run for the requested frame count"
        );
    }

    #[test]
    fn direct_initial_title_card_handoff_wrapper_returns_requested_frames_without_cache() {
        let manifest = minimal_manifest();
        let trace = run_script_with_direct_initial_title_card_handoff(manifest, &[0, 0], 2);

        assert_eq!(
            trace.len(),
            2,
            "ASM loc_1AE2E direct initial title-card wrapper must delegate to ScriptRunner::run for the requested frame count"
        );
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

    #[test]
    fn level09_recording_settle_loads_resource_terrain_loc_11c7e() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);

        runtime.boot_to_level(LevelId(0x09));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x09));

        let frame = runtime.trace_frame();
        assert_eq!(frame.game_mode, GameMode::InGame);
        assert_eq!(frame.number_lives, 3);
        assert_eq!(frame.kid_y_pos, Some(0x019F_0000));
        assert_eq!(frame.camera_y_pos, 0x00E0_0000);
        assert!(
            runtime.block_grid().is_some(),
            "ASM loc_11C7E resource 0x56 terrain must remain loaded for logical level 0x09"
        );
        assert!(
            runtime
                .projectiles
                .iter()
                .all(|projectile| { projectile.kind != EnemyProjectileKind::DragonWalkingBreath }),
            "ASM Dragon.asm loc_3CF6E must not run only because a missing terrain sidecar killed Kid during settle"
        );
    }

    #[test]
    fn level55_recording_handoff_matches_oracle_hud_diamond_phase_loc_45ee() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level_for_recording_oracle(LevelId(0x55));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x55));

        assert_eq!(
            (
                runtime.hud_animation_state.diamond_countdown_fb58,
                runtime.hud_animation_state.diamond_frame_offset_fb5d,
                runtime
                    .gameplay_hud_vram_state
                    .diamond_frame_table_byte_offset_loc_5d4a(),
            ),
            (1, 0, 0),
            "ASM sub_44DC loc_45EE and oracle `$FA58/$FA5D/$DD40` watches pin the Level 0x55 recording hand-off phase"
        );

        let mut tail_phases = Vec::new();
        for frame_index in 0..360 {
            let frame = runtime.step_frame(FrameInput::default());
            if frame_index >= 350 {
                tail_phases.push((
                    frame.hud_diamond_frame_offset,
                    frame.hud_diamond_vram_page_loc_5d4a,
                ));
            }
        }
        assert_eq!(
            tail_phases,
            vec![
                (Some(2), Some(4)),
                (Some(2), Some(4)),
                (Some(2), Some(4)),
                (Some(4), Some(8)),
                (Some(4), Some(8)),
                (Some(4), Some(8)),
                (Some(4), Some(8)),
                (Some(4), Some(8)),
                (Some(4), Some(8)),
                (Some(6), Some(12)),
            ],
            "ASM sub_44DC loc_45EE and oracle frames 350..359 `$FA5D/$DD40` watches pin the post-transform diamond phase"
        );
    }

    #[test]
    fn level33_recording_handoff_retains_oracle_platform_phase_sub_23ea() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level_for_recording_oracle(LevelId(0x33));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x33));

        assert_eq!(
            (
                runtime.hud_animation_state.diamond_countdown_fb58,
                runtime.hud_animation_state.diamond_frame_offset_fb5d,
                runtime
                    .gameplay_hud_vram_state
                    .diamond_frame_table_byte_offset_loc_5d4a(),
            ),
            (4, 0, 0),
            "ASM sub_44DC loc_45EE and oracle `$FA58/$FA5D` watches pin the Level 0x33 recording hand-off seed"
        );

        for _ in 0..360 {
            runtime.step_frame(FrameInput::default());
        }

        let frame = runtime.trace_frame();
        assert_eq!(
            (
                frame.hud_diamond_frame_offset,
                frame.hud_diamond_vram_page_loc_5d4a,
            ),
            (Some(0), Some(0)),
            "oracle frame 359 has `$FA5D=0` and no `$DD40` write, so page 0 is fully resident"
        );

        let platform = runtime
            .active_platforms
            .iter()
            .find(|platform| platform.layout_index == Some(0))
            .expect("ASM sub_23EA should retain the lower Level 0x33 scripted platform");
        assert_eq!(platform.y_pos, 0x0094_0000);
        assert_eq!(
            platform
                .script
                .expect("scripted platform")
                .remaining_duration,
            0x0044
        );
    }
}
