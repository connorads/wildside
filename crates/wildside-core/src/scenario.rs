use super::*;
use crate::level::{
    island_shore_art_source_label_loc_1220e, step_island_palette_cycle_lower_band_loc_12188,
};

// Full-frame oracle warps do more than the explicit 60-frame stable-level poll:
// the ROM level-skip path has a live title-card tail before the helper starts
// counting stable target frames. The compatibility `boot_to_level` path skips
// that tail, so settled full-frame traces need the oracle-observed live age.
const ORACLE_SETTLED_WARP_LIVE_FRAMES: usize = 108;
const RECORDING_SETTLED_WARP_LIVE_FRAMES: usize = 60;
const MULTI_HEAD_BOSS_LEVEL_SKIP_LIVE_TAIL_FRAMES: usize = 60;
const DRILLER_LEVEL_SKIP_LIVE_TAIL_FRAMES_LOC_3BE78: usize = 63;
// ASM loc_74E0: residual stable gameplay windows after the recording runner
// has already executed the typed debug title-card lifecycle and its hidden
// Mode_Level object passes. The direct compatibility path retains its older
// full-window values in `recording_oracle_warp_target_live_frames`.
const LEVEL_ONE_RECORDING_SETTLED_WARP_LIVE_FRAMES: usize = 65;
const LEVEL_SIX_RECORDING_SETTLED_WARP_LIVE_FRAMES: usize = 61;
const LEVEL_EIGHT_RECORDING_SETTLED_WARP_LIVE_FRAMES: usize = 102;
const LEVEL_1B_RECORDING_SETTLED_WARP_LIVE_FRAMES: usize = 70;
const BLOODY_SWAMP_RECORDING_SETTLED_WARP_LIVE_FRAMES: usize = 108;
const RECORDING_SETTLED_WARP_TIMER_SECONDS_LOC_BDC0: u16 = 179;
const RECORDING_SETTLED_WARP_TIMER_SUBFRAMES_LOC_BDC0: u16 = 0x08;
const LEVEL_ZERO_RECORDING_TIMER_SECONDS_BEFORE_FRAME_0_LOC_BE82: u16 = 180;
const LEVEL_ZERO_RECORDING_TIMER_SUBFRAMES_BEFORE_FRAME_0_LOC_BE82: u16 = 1;
const LEVEL_ZERO_RECORDING_TIME_FRAMES_BEFORE_FRAME_0_LOC_3038A: u16 = 0x003C;
const LEVEL_SIX_RECORDING_GHOST_BLOCK_EXTRA_TICKS: usize = 0x08;
// ASM loc_124A6/sub_F730: #230 Crypt oracle recordings keep the col $1D
// group vanished at frame 298 while the col $22 wall remains solid until
// frame 365. The recording shell therefore advances SolidGhost state by $2F
// ticks beyond the direct boot settle for the 0x38/0x47 block-list aliases.
const CRYPT_RECORDING_GHOST_BLOCK_EXTRA_TICKS: usize = 0x2F;
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
            LevelId(0) => RECORDING_SETTLED_WARP_LIVE_FRAMES,
            LevelId(1) => ORACLE_SETTLED_WARP_LIVE_FRAMES - 1,
            LevelId(0x06) => RECORDING_SETTLED_WARP_LIVE_FRAMES + 0x18,
            LevelId(0x08) => 101,
            LevelId(0x1B) => 112,
            LevelId(0x3E) => 107,
            _ => RECORDING_SETTLED_WARP_LIVE_FRAMES,
        };
    }
    if level_id == LevelId(0) {
        RECORDING_SETTLED_WARP_LIVE_FRAMES
    } else if level_id == LevelId(1) {
        LEVEL_ONE_RECORDING_SETTLED_WARP_LIVE_FRAMES
    } else if level_id == LevelId(0x08) {
        // ASM loc_74E0's level-skip path has a live target-level tail before
        // the helper's 60 stable-frame recording window. Level 0x08 exposes
        // that hidden age through Murderwall loc_140CE plus loc_7316/loc_7334:
        // oracle issue266 frame 0 has advanced 102 wall ticks from the -$30 px
        // seed, so native must enter the scenario one pre-frame earlier and let
        // the traced frame supply tick 102.
        LEVEL_EIGHT_RECORDING_SETTLED_WARP_LIVE_FRAMES
    } else if level_id == LevelId(0x06) {
        // Under Skull Mountain 3 exposes the ROM level-skip tail through
        // loc_124A6/sub_F730 ghost-block phase. Once oracle enemy capture can
        // see Dragon_Flying.asm loc_3480A via loc_32146's collision source, the
        // same issue #289 recording pins the live tail at 0x18 frames for
        // object scheduling; the final eight ticks belong only to the
        // ghost-block timer below.
        LEVEL_SIX_RECORDING_SETTLED_WARP_LIVE_FRAMES
    } else if level_id == LevelId(0x1B) {
        LEVEL_1B_RECORDING_SETTLED_WARP_LIVE_FRAMES
    } else if level_id == LevelId(0x3E) {
        // Bloody Swamp exposes the ROM level-skip live tail through Murderwall
        // loc_140CE. Oracle issue374 settle tracing reaches frame-0 wall age at
        // target_live 47, so the recording frame-60 checkpoint needs the same
        // hidden 47-frame prefix before the normal 60 visible frames.
        BLOODY_SWAMP_RECORDING_SETTLED_WARP_LIVE_FRAMES
    } else {
        RECORDING_SETTLED_WARP_LIVE_FRAMES
    }
}

fn recording_oracle_warp_extra_ghost_block_ticks(level_id: LevelId) -> usize {
    match level_id {
        LevelId(0x06) => LEVEL_SIX_RECORDING_GHOST_BLOCK_EXTRA_TICKS,
        LevelId(0x38 | 0x47) => CRYPT_RECORDING_GHOST_BLOCK_EXTRA_TICKS,
        _ => 0,
    }
}

fn recording_oracle_warp_resets_level_one_rng(level_id: LevelId) -> bool {
    level_id == LevelId(1)
}

fn recording_oracle_warp_enemy_live_tail_frames(
    families: impl IntoIterator<Item = EnemyFamily>,
    target_window_has_driller: bool,
) -> usize {
    let mut has_multi_head_boss = false;
    for family in families {
        has_multi_head_boss |= is_multi_head_boss(family);
    }
    usize::from(has_multi_head_boss) * MULTI_HEAD_BOSS_LEVEL_SKIP_LIVE_TAIL_FRAMES
        + usize::from(target_window_has_driller) * DRILLER_LEVEL_SKIP_LIVE_TAIL_FRAMES_LOC_3BE78
}

fn should_restore_recorded_title_card_after_first_live(
    restore_after_first_live: bool,
    game_mode: GameMode,
) -> bool {
    restore_after_first_live && game_mode == GameMode::InGame
}

impl NativeRuntime {
    /// Boot through menus to level 0, then advance to `target` using the
    /// level skip mechanism. Used by sweep infrastructure to reach any level
    /// without encoding warp buttons in the input script.
    ///
    /// Boot timing matches the oracle helper exactly: 720 frames with
    /// Start presses at frames 120, 200, 280, 360, 460.
    pub fn boot_to_level(&mut self, target: LevelId) {
        self.boot_to_level_with_warp_lifecycle(target, false);
    }

    fn boot_to_level_for_recording_oracle(&mut self, target: LevelId) {
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

        if !use_recording_oracle_lifecycle {
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
                for _ in 0..120 {
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
        self.progression.number_hitpoints = self.progression.current_form.max_hitpoints();
        self.just_received_damage = false;
        self.death_shell_snapshot = None;
        self.level_complete_gfx_snapshot = None;
        self.reload_block_grid_for_level(self.current_level);
        if !used_debug_warp || !use_recording_oracle_lifecycle {
            self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(self.current_level));
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
        self.settle_target_level_live_window(level_id, target_live_frames);
    }

    pub fn settle_recording_oracle_warp_live_window(&mut self, level_id: LevelId) {
        // ASM loc_1AE2E/loc_74E0 feed recording oracles through the helper's
        // `_settled_recording_oracle` branch. Level 0 comes straight through
        // the title-card gate and waits 60 stable live frames; Level 1 still
        // reaches the ROM's level-skip resume tail that native `boot_to_level`
        // shortcuts. Oracle settle tracing ends Level 1 at RNG offset $FFCB;
        // the first scenario frame then advances to $FFCF via Drips loc_3340E.
        let debug_lifecycle_completed = self.recording_oracle_debug_lifecycle_completed;
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
        if level_id == LevelId(0)
            || (!debug_lifecycle_completed
                && oracle_warp_primes_respawn_settle_enemy_loading(level_id))
        {
            self.prime_respawn_settle_enemy_loading_loc_368a8(level_id);
        }
        self.prime_recording_warp_sprite_animation_clocks();
        // ASM loc_74E0 enters the target Mode_Level object schedule while the
        // recording helper is still inside its fixed title wait. Nearby
        // Drillers spawn 63 object passes before target_live step zero, so
        // Driller.asm loc_3BE78 reaches the same drill child and block-worker
        // phase before the ordinary 60-frame stable window.
        let (camera_x_px, camera_y_px) = self.opening_slice.map_or((0, 0), |slice| {
            (slice.camera.x_pos >> 16, slice.camera.y_pos >> 16)
        });
        let target_window_has_driller = self.enemy_layout(level_id).is_some_and(|layout| {
            let view = enemy_layout_view(layout, camera_x_px, camera_y_px);
            view.objects.iter().any(|object| {
                EnemyFamily::from_layout_object(view.enemy_type_slots, object)
                    == EnemyFamily::Driller
            })
        });
        let enemy_level_skip_tail = recording_oracle_warp_enemy_live_tail_frames(
            self.active_enemies.iter().map(|enemy| enemy.family),
            target_window_has_driller,
        );
        // ASM loc_74E0 keeps the target level live behind the level-skip
        // shell. Enemy22_Shiskaboss_Init and the parallel multi-head boss
        // initialisers expose that full tail through their continuously
        // moving head, eye and projectile coroutines before recording frame
        // zero; the direct native boot shortcut otherwise starts them one
        // stable-window late.
        let target_live_frames =
            recording_oracle_warp_target_live_frames(level_id, debug_lifecycle_completed)
                .saturating_add(enemy_level_skip_tail);
        self.settle_target_level_live_window(level_id, target_live_frames);
        // ASM Init_SpriteAttr_HUD loc_BDC0 seeds Time_SubSeconds to 1. The
        // level-skip tail is level-dependent: #289 Level 0x07 watches reach
        // frame 0 at 2:59/$07, while the direct Level 0 release probe reaches
        // it at 2:59/$3C. Keep both oracle-derived seeds on this recording
        // settle shim so ordinary gameplay startup stays on LevelTimer::new.
        if level_id == LevelId(0) {
            // ASM loc_3038A/Make_SpriteAttr_HUD loc_BE82: direct Level 0
            // oracle watches reach recording frame 0 at Time_Frames=$003D,
            // Time=2:59 and Time_SubSeconds=$003C. Seed the immediately prior
            // state here; the first scenario step performs both decrements.
            self.time_frames_loc_3038a = LEVEL_ZERO_RECORDING_TIME_FRAMES_BEFORE_FRAME_0_LOC_3038A;
            self.level_timer.seconds = LEVEL_ZERO_RECORDING_TIMER_SECONDS_BEFORE_FRAME_0_LOC_BE82;
            self.level_timer.sub_frames =
                LEVEL_ZERO_RECORDING_TIMER_SUBFRAMES_BEFORE_FRAME_0_LOC_BE82;
        } else {
            self.level_timer.seconds = RECORDING_SETTLED_WARP_TIMER_SECONDS_LOC_BDC0;
            self.level_timer.sub_frames = RECORDING_SETTLED_WARP_TIMER_SUBFRAMES_LOC_BDC0;
        }
        // ASM loc_1AE2E's recording hand-off exposes the initial 3:00 table
        // before the first scenario step. The adapter reseeds timer RAM above
        // after collapsing the hidden settle window, so reseed the loc_BEE8
        // sprite-table value at the same boundary.
        self.hud_timer_sprite_seconds_loc_bee8 = self.level_timer.seconds;
        let extra_ghost_block_ticks = recording_oracle_warp_extra_ghost_block_ticks(level_id);
        if extra_ghost_block_ticks != 0 {
            // ASM loc_124A6 seeds ghost-block descriptors and sub_F730 ticks
            // them during the level-skip shell that recording helpers observe.
            // Dragon_Flying.asm loc_3480A pins Level 0x06 object scheduling
            // eight frames younger than its ghost phase. The Crypt 0x38/0x47
            // block-list aliases need their oracle-observed SolidGhost phase
            // without advancing Kid, object loading, or the global timer.
            for _ in 0..extra_ghost_block_ticks {
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
        if recording_oracle_warp_resets_level_one_rng(level_id) {
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
                    art_source_label: island_shore_art_source_label_loc_1220e(frame_index)
                        .to_owned(),
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
        // ASM Mode_Level loc_901 presents this already-built sprite/scroll/HUD
        // state before the first scenario step produces its post-update trace.
        let initial_render_latch = runtime.trace_frame();
        let delay_recording_input = self.settle_warp == Some(SettleWarpMode::RecordingOracle);
        let mut trace = Vec::with_capacity(frame_count);
        let mut vdp_snapshots = if capture_vdp {
            Vec::with_capacity(frame_count)
        } else {
            Vec::new()
        };
        let mut restore_recorded_title_card_after_first_live =
            self.restore_recorded_title_card_after_first_live;
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
            let snapshot = runtime.step_frame(input);
            if should_restore_recorded_title_card_after_first_live(
                restore_recorded_title_card_after_first_live,
                snapshot.game_mode,
            ) {
                runtime.set_recorded_title_card_handoff_delay(true);
                restore_recorded_title_card_after_first_live = false;
            }
            trace.push(snapshot);
            if capture_vdp {
                // step_frame can early-return before update_vdp_state (lib.rs:2546,
                // the DemoPlay no-input path), so the captured snapshot then
                // reflects the prior frame — identical to what play-native
                // observes, so viewer/player parity holds.
                vdp_snapshots.push(runtime.vdp_state().cloned());
            }
        }
        (initial_render_latch, trace, vdp_snapshots)
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
    fn recording_oracle_warp_target_live_frames_pin_oracle_routes() {
        assert_eq!(
            recording_oracle_warp_target_live_frames(LevelId(0), true),
            RECORDING_SETTLED_WARP_LIVE_FRAMES,
            "ASM loc_1AE2E Level 0 recording settle uses the full 60-frame live window"
        );
        assert_eq!(
            recording_oracle_warp_target_live_frames(LevelId(1), true),
            LEVEL_ONE_RECORDING_SETTLED_WARP_LIVE_FRAMES,
            "Level 1 recording settle stops at the RNG $FFCB route instead of the full-frame settle age"
        );
        assert_eq!(
            recording_oracle_warp_target_live_frames(LevelId(0x06), true),
            LEVEL_SIX_RECORDING_SETTLED_WARP_LIVE_FRAMES,
            "Under Skull Mountain 3 carries the Dragon_Flying.asm loc_3480A live-tail age before frame 0"
        );
        assert_eq!(
            recording_oracle_warp_target_live_frames(LevelId(0x08), true),
            LEVEL_EIGHT_RECORDING_SETTLED_WARP_LIVE_FRAMES,
            "Hills murder-wall recording settle enters one pre-frame before the traced 102nd wall tick"
        );
        assert_eq!(
            recording_oracle_warp_target_live_frames(LevelId(0x1B), true),
            LEVEL_1B_RECORDING_SETTLED_WARP_LIVE_FRAMES,
            "Skydragon Castle recording settle keeps its shorter opening Sphere loc_36148 age"
        );
        assert_eq!(
            recording_oracle_warp_target_live_frames(LevelId(0x3E), true),
            BLOODY_SWAMP_RECORDING_SETTLED_WARP_LIVE_FRAMES,
            "Bloody Swamp recording settle keeps Murderwall loc_140CE's live-tail age"
        );
        assert_eq!(
            recording_oracle_warp_target_live_frames(LevelId(0x3F), true),
            RECORDING_SETTLED_WARP_LIVE_FRAMES,
            "Scorpion Isle uses the default window; Archer.asm loc_33FDA timing is modelled in enemies"
        );
        assert_eq!(
            recording_oracle_warp_target_live_frames(LevelId(0x12), true),
            RECORDING_SETTLED_WARP_LIVE_FRAMES,
            "ordinary recording oracles use the default stable live window"
        );
    }

    #[test]
    fn recording_oracle_warp_enemy_tail_preserves_driller_loc_3be78_hidden_age() {
        assert_eq!(
            recording_oracle_warp_enemy_live_tail_frames([], true),
            DRILLER_LEVEL_SKIP_LIVE_TAIL_FRAMES_LOC_3BE78,
            "ASM loc_74E0 gives Driller.asm loc_3BE78 63 hidden object passes before the recording helper's stable target window",
        );
        assert_eq!(
            recording_oracle_warp_enemy_live_tail_frames([EnemyFamily::Archer], false),
            0,
            "families without an oracle-proven hidden tail retain the default recording settle window",
        );
        assert_eq!(
            recording_oracle_warp_enemy_live_tail_frames([EnemyFamily::Driller], false),
            0,
            "a transient pre-settle roster must not age Drillers unless the target Manage_EnemyLoading window contains them",
        );
    }

    #[test]
    fn recording_oracle_warp_extra_side_effects_are_level_specific() {
        assert_eq!(
            recording_oracle_warp_extra_ghost_block_ticks(LevelId(0x05)),
            0
        );
        assert_eq!(
            recording_oracle_warp_extra_ghost_block_ticks(LevelId(0x06)),
            LEVEL_SIX_RECORDING_GHOST_BLOCK_EXTRA_TICKS
        );
        assert_eq!(
            recording_oracle_warp_extra_ghost_block_ticks(LevelId(0x07)),
            0
        );
        assert_eq!(
            recording_oracle_warp_extra_ghost_block_ticks(LevelId(0x38)),
            CRYPT_RECORDING_GHOST_BLOCK_EXTRA_TICKS
        );
        assert_eq!(
            recording_oracle_warp_extra_ghost_block_ticks(LevelId(0x47)),
            CRYPT_RECORDING_GHOST_BLOCK_EXTRA_TICKS
        );

        assert!(!recording_oracle_warp_resets_level_one_rng(LevelId(0)));
        assert!(recording_oracle_warp_resets_level_one_rng(LevelId(1)));
        assert!(!recording_oracle_warp_resets_level_one_rng(LevelId(2)));
    }

    #[test]
    fn settle_recording_oracle_warp_ticks_level_specific_ghost_blocks_loc_124a6_f730() {
        let mut ordinary_level = runtime_with_single_solid_ghost_group();

        ordinary_level.settle_recording_oracle_warp_live_window(LevelId(0x05));

        let ordinary_group = ordinary_level.block_state.ghost_groups[0];
        assert_eq!(
            (
                ordinary_group.timer,
                ordinary_group.state,
                ordinary_group.solid
            ),
            (1, 0, true),
            "ordinary recording settles must not tick ASM sub_F730 SolidGhost state"
        );

        let mut skull_mountain = runtime_with_single_solid_ghost_group();

        skull_mountain.settle_recording_oracle_warp_live_window(LevelId(0x06));

        let skull_group = skull_mountain.block_state.ghost_groups[0];
        assert_eq!(
            (skull_group.timer, skull_group.state, skull_group.solid),
            (2, 3, false),
            "ASM loc_124A6/sub_F730 recording settle should advance Level 0x06 by the eight oracle-observed SolidGhost ticks"
        );
        let cell = skull_mountain
            .block_grid
            .as_ref()
            .expect("synthetic ghost grid should remain installed")
            .cell_at(1, 1);
        assert!(
            !cell.has_block,
            "ASM loc_F8A6 should clear the SolidGhost cell during the extra recording-settle ticks"
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

    #[test]
    fn step_island_shore_lower_band_only_updates_lower_dma_loc_12188() {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        runtime.island_shore_dma_uploads_loc_1202a.push(
            crate::types::IslandShoreDmaUploadLoc1202A {
                frame_index: 3,
                art_source_label: "stale".to_owned(),
                source_offset: 0,
                vram_word_address: 0,
                dma_length_bytes: 0,
            },
        );
        runtime.island_shore_cycle_loc_1202a = Some(IslandShoreCycleStateLoc1202A {
            primary_countdown_44: 9,
            mid_countdown_46: 0x0F,
            lower_countdown_48: 1,
            primary_phase_d3: 1,
            mid_phase_d2: 1,
            lower_phase_d1: 2,
            primary_vram_frame: 0,
            mid_vram_frame: 0,
            lower_vram_frame: 0,
        });

        runtime.step_island_shore_lower_band_only_loc_12188();

        let state = runtime
            .island_shore_cycle_loc_1202a
            .expect("lower shore state should remain active");
        assert_eq!(
            (
                state.lower_countdown_48,
                state.lower_phase_d1,
                state.lower_vram_frame
            ),
            (0x001E, 3, 2),
            "ASM loc_12188 should tick only the lower shore band state"
        );
        assert_eq!(
            runtime.island_shore_dma_uploads_loc_1202a,
            vec![crate::types::IslandShoreDmaUploadLoc1202A {
                frame_index: 2,
                art_source_label: "unk_97E0E".to_owned(),
                source_offset: 0x0162,
                vram_word_address: 0xF760,
                dma_length_bytes: 0x0180,
            }],
            "ASM loc_12188 should clear stale shore DMA uploads and enqueue only the lower-band transfer"
        );
    }

    fn runtime_with_single_solid_ghost_group() -> NativeRuntime {
        let mut runtime = NativeRuntime::new(minimal_manifest());
        let mut layout = vec![0u8; 4 * 4 * 2];
        let ghost_offset = (4 + 1) * 2;
        layout[ghost_offset] = 0xE3;
        layout[ghost_offset + 1] = 0x04;
        let mut erase_words = vec![0u16; 4 * 4];
        erase_words[4 + 1] = 0x6005;
        runtime.block_grid = Some(
            terrain::BlockGrid::from_level_layout_with_erase_words_loc_11542(
                &layout,
                erase_words,
                4,
                4,
            ),
        );
        runtime
            .block_state
            .ghost_groups
            .push(crate::types::GhostGroup {
                col: 1,
                row: 1,
                count: 1,
                vertical: false,
                solid_time: 10,
                vanished_time: 5,
                timer: 1,
                state: 0,
                solid: true,
            });
        runtime
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
}
