mod blocks;
mod boot;
mod combat;
pub(crate) mod constants;
mod demo;
pub(crate) mod enemies;
pub mod enigma_timing;
mod gfx_animation;
mod helmets;
pub mod intro_timing;
mod level;
mod mode_level_timing;
mod movement;
pub mod physics;
mod platforms;
mod projectiles;
pub mod saxman_timing;
mod scenario;
pub mod terrain;
mod trace;
mod transform_presentation;
pub(crate) mod types;
pub mod vdp;
mod weather;

use combat::{
    HeadyMetalBreakupPieceSub37708, HeadyMetalCachedWorkerSub37708, HeadyMetalScatterVelocities,
    HeadyMetalSpriteVelocitySub376EC, PendingSoundEffect, QueuedSoundCommand, QueuedSoundEffect,
    heady_metal_apply_piece_velocities_sub_376EC, heady_metal_init_enemy20_loc_39247,
    heady_metal_sprite_init_sub_376B6, heady_metal_sprite_init_sub_3764A,
    heady_metal_sprite_init_sub_37680,
};
use constants::*;
use enemies::*;
use physics::{Direction, HorizontalInput, LevelBounds};
use platforms::*;
pub use scenario::{
    ScriptRunner, run_script, run_script_with_direct_initial_title_card_handoff,
    run_script_with_direct_title_card_handoff, run_script_with_physics_order,
    run_script_with_settled_warp, run_script_with_trace, run_script_with_warp,
};
use wildside_data::{
    AssetManifest, AssetSource, EnemyLayout, EnemyLayoutObject, FsAssetSource, PlatformLayout,
    PlatformPlacement, load_level_asset_bundle_with_source, resource_level_id,
    resource_level_record,
};
pub const BOOT_TO_LEVEL_START_FRAMES: [usize; 7] = constants::BOOT_TO_LEVEL_START_FRAMES;
pub use blocks::{
    FloatingPrizeGraphicsSpec, Prize10DiamondsInitLoc108E8, Prize10DiamondsSpawnLoc108EE,
    Prize10DiamondsWorkerCountdownGateLoc10984, Prize10DiamondsWorkerScanStartLoc10960,
    PrizeBlockSpawnFlashLoc102B8, floating_prize_graphics_spec,
};
pub use boot::{
    HudTimerSpriteLayoutSubBD0A, HudTimerSpritePositionSubBD0A, PAUSE_MENU_Y_OFFSET_LOC_6948,
    PauseMenuTextSelectionRestartText, PauseMenuTextWriteRow, Prize10000PointsStarSpawnLoc10B3A,
    Prize10000PointsStarStepLoc10B74, hud_timer_low_time_check_end_decrease_time_left,
    hud_timer_sprite_layout_sub_BD0A, pause_darken_palette_loc_6C70,
    pause_menu_text_selection_restart_text, pause_menu_text_write_plan_loc_69D2,
    pause_menu_text_write_row_vdp_write_cmd_loc_6A1E,
    pause_menu_tile_save_wrap_column_reset_loc_69AE,
    prize_10000_points_star_latch_screen_pos_loc_10B9A, prize_10000_points_star_spawn_loc_10B3A,
    prize_10000_points_star_step_loc_10B74,
};
pub use combat::{
    HEADY_METAL_SUB_37ABC_BODY_PIECE_OFFSETS_LOC_39266, heady_metal_piece_sprite_index_sub_37A22,
    speed_bonus_threshold_loc_d660, time_bonus_loc_d56e,
};
pub use enemies::layout_object_family_name;
pub use level::{
    BackgroundArtPermutationLoc11E8A, BackgroundChunkCopiedTileLoc12884,
    BackgroundChunkCopyLoc12884, BackgroundFollowupArtPermutationLoc11EC4,
    BackgroundFollowupArtSourceLoc11EC4, BackgroundScrollForestWaterRippleSetupLoc305E6,
    BackgroundScrollForestWaterRippleVisibleBandLoc305D4,
    BackgroundScrollShiftLayersLeadingPartialFillLoc30562,
    BackgroundScrollShiftLayersTrailingPartialFillLoc30592, BackgroundScrollThemeDispatchLoc30598,
    EOL_COUNTER_START_FIXED_FRAMES_LOC_D770_ORACLE, EOL_FIXED_PRE_COUNTER_FRAMES_LOC_D770,
    EOL_FOREGROUND_REVEAL_VISIBLE_DELAY_FRAMES_LOC_C21E_ORACLE,
    EOL_HOLOGRAM_VISIBLE_FRAME_LOC_C1D0_ORACLE, EOL_POST_ROWS_PRE_COUNTER_FRAMES_LOC_D770,
    EOL_PRE_SCOREBOARD_FADE_FRAMES_LOC_C0CC, EOL_SCORE_COUNTER_START_LEAD_FRAMES_LOC_D754_ORACLE,
    EOL_SCORE_ROWS_START_FIXED_FRAMES_LOC_D468,
    EOL_SCORE_ROWS_VISIBLE_DELAY_FRAMES_LOC_D468_ORACLE, EolScoreCounterSnapshotLocD770,
    FlagpoleScoreboardRevealTileLocC21E, ForegroundArtPermutationLoc11E48,
    JuggernautSkullArtDmaLoc129DC, SharedVramArtSourceLoc11EFA,
    TOUCH_FLAGPOLE_GFX_CLEANUP_VISIBLE_FRAME_LOC_C0B6_ORACLE,
    TOUCH_FLAGPOLE_HIBERNATE_FRAMES_LOC_C070, TransferScrollDataStormLoc30110,
    background_art_decompress_loc_11E8A_11EA2, background_chunk_row_loop_loc_12884,
    background_desert_tileset_loc_12988, background_dimensions_loc_11B40,
    background_final_pattern_loc_12952, background_followup_art_decompress_loc_11EA2_11EF2,
    background_hail_overlay_loc_1294A, background_hills_tileset_loc_1295E,
    background_mappings_read_chunk_loc_12830, background_mappings_terminator_loc_12842,
    background_mountain_tileset_loc_12910, background_row_pointers_loc_11B7A,
    background_scroll_desert_heat_ripple_loop_loc_306F4,
    background_scroll_desert_heat_ripple_setup_loc_306D4,
    background_scroll_desert_heat_ripple_visible_band_loc_306C2,
    background_scroll_forest_water_ripple_loop_loc_3060A,
    background_scroll_forest_water_ripple_setup_loc_305E6,
    background_scroll_forest_water_ripple_visible_band_loc_305D4,
    background_scroll_shift_layers_leading_partial_fill_loc_30562,
    background_scroll_shift_layers_trailing_partial_fill_loc_30592,
    background_scroll_theme_dispatch_loc_30598, blend_palette_word_sub_14C0,
    block_mapping_or_mask_loc_11D42, copy_block_mappings_loc_11D46,
    eol_score_counter_snapshot_loc_D770, eol_score_counter_transfer_frames_from_trace_loc_D770,
    eol_title_return_faded_palette_word_loc_D734, flag_bottom_art_dma_loop_loc_12B9C,
    flagpole_scoreboard_reveal_tiles_loc_C21E, foreground_art_decompress_loc_11E48_11E60,
    hitpoint_display_art_dma_loop_loc_12C34, juggernaut_skull_art_dma_loop_loc_129DC,
    juggernaut_skull_art_dma_loop_loc_129E8, juggernaut_star_art_dma_loop_loc_129FC,
    shared_vram_art_decompresses_loc_11EFA,
};
pub use mode_level_timing::{
    CyclonePickupTransformEntrySubE12, GemsQueueCommandAdvance, GemsQueueCommandControl,
    GemsQueueCommandPath, GemsStopMusicArbitration, GemsStopMusicBusAckPolls,
    GemsStopMusicBusWindow, GemsStopMusicElapsedBusWindow, GemsStopMusicElapsedTiming,
    GemsStopMusicTiming, GenesisMclkCount, LoadInGamePlatformArtSourceWorkload,
    LoadInGamePlatformInitialisationSourceWorkload, LoadInGameTimerBonusPath, M68kCycleCount,
    M68kExternalWaitEvent, M68kRefreshAdvance, M68kRefreshPhase,
    ModeLevelForegroundCellQueuePathSub1596, ModeLevelForegroundCellQueueWorkSub1596,
    ModeLevelHudBuildPath, ModeLevelPendingHardwareOwnershipAfterLoc7d6,
    ModeLevelPostHudPhaseAdvance, ModeLevelScrollTransferPath, ModeLevelTailAdvanceLoc892,
    ModeLevelTailBackgroundWorkLoc3038A, ModeLevelTailCameraCellMappingSub30194,
    ModeLevelTailCameraStripeSub30194, ModeLevelTailInputsLoc892, ModeLevelTailPaletteWorkSub14C0,
    ModeLevelTailPieceRefreshSub307D8, ModeLevelTailPieceRowsSub30744,
    ModeLevelTailShiftDataAdvanceLoc30452, ModeLevelTailShiftDataSlotLoc30452,
    ModeLevelTailShiftDataStateLoc30452, ModeLevelTailShiftLayersWork, ModeLevelTailStateLoc892,
    ModeLevelTailWorkLoc892, ModeLevelVblankArbitrationBeforeSubE12,
    ModeLevelVblankRefreshSuffixAfterFinalVdpOwnership, ModeLevelVblankWorkBeforeSubE12,
    ReadJoypadControl, TransformRasterWorkSub8E0, WaitForVintArbitration,
    WaitForVintInterruptSample, WaitForVintResumePath,
    block_debris_sprites_advance_with_scheduler_sub_2a4c, block_debris_sprites_raw_cycles_sub_2a4c,
    cyclone_pickup_transform_entry_sub_e12, damage_flash_wait_cycles_loc_bc44,
    demo_exit_wait_cycles_loc_119e0,
    diamond_power_compile_sprites_advance_with_scheduler_sub_3f798,
    diamond_power_compile_sprites_raw_cycles_sub_3f798, execute_objects_elapsed_cycles_sub_e12,
    execute_objects_external_waits_sub_e12, execute_objects_raw_cycles_sub_e12,
    execute_objects_setup_cycles_sub_e12,
    execute_scripted_platforms_advance_with_scheduler_sub_23ea,
    execute_scripted_platforms_raw_cycles_sub_23ea, gems_queue_command_advance_with_scheduler,
    gems_stop_music_elapsed_cycles, gems_stop_music_elapsed_timing,
    gems_stop_music_elapsed_timing_from_absolute_entry, gems_stop_music_timing,
    gfx_objects_collision_advance_with_scheduler_sub_16f0,
    gfx_objects_collision_kid_advance_with_scheduler_sub_1b76,
    gfx_objects_collision_kid_f86e_advance_with_scheduler_sub_1b76,
    gfx_objects_collision_kid_f86e_raw_cycles_sub_1b76,
    gfx_objects_collision_kid_raw_cycles_sub_1b76, gfx_objects_collision_raw_cycles_sub_16f0,
    gfx_objects_move_and_animate_advance_with_scheduler_sub_1444,
    gfx_objects_move_and_animate_raw_cycles_sub_1444, hud_diamond_only_raw_cycles_sub_44dc,
    hud_flag_only_raw_cycles_sub_44dc, hud_no_upload_raw_cycles_sub_44dc,
    input_worker_cycles_loc_7452, kid_projectile_collision_advance_with_scheduler_sub_1d76,
    kid_projectile_collision_raw_cycles_sub_1d76,
    load_in_game_platform_initialisation_source_workload, load_in_game_stop_music_tail_advance,
    load_in_game_stop_music_tail_advance_with_scheduler, load_in_game_stop_music_tail_raw_cycles,
    make_sprites_from_gfx_objects_advance_with_scheduler_sub_1492,
    make_sprites_from_gfx_objects_raw_cycles_sub_1492,
    make_sprites_from_platforms_advance_with_scheduler_sub_2444,
    make_sprites_from_platforms_raw_cycles_sub_2444,
    manage_enemy_loading_advance_with_scheduler_loc_36814, manage_enemy_loading_raw_cycles,
    manage_platform_loading_advance_with_scheduler_loc_2744,
    manage_platform_loading_raw_cycles_loc_2744, mode_level_hud_build_path_sub_bdc0,
    mode_level_latch_damage_state_advance_loc_83e,
    mode_level_post_hud_advance_to_loc_892_with_phase_observer,
    mode_level_post_hud_advance_to_loc_892_with_scheduler,
    mode_level_post_hud_phase_raw_cycles_to_loc_892, mode_level_post_hud_raw_cycles_to_loc_892,
    mode_level_tail_advance_to_wait_for_vint_loc_892,
    mode_level_tail_camera_stripe_raw_cycles_sub_30194,
    mode_level_tail_camera_stripe_work_sub_30194, mode_level_tail_palette_raw_cycles_sub_14c0,
    mode_level_tail_piece_refresh_raw_cycles_sub_307d8,
    mode_level_tail_raw_cycles_to_wait_for_vint_loc_892, mode_level_vblank_advance_before_sub_e12,
    mode_level_vblank_elapsed_cycles_before_sub_e12,
    mode_level_vblank_source_cycles_before_sub_e12,
    platforms_check_collision_advance_with_scheduler_sub_2574,
    platforms_check_collision_raw_cycles_sub_2574, player_callback_cycles_loc_a73a,
    player_callback_cycles_sub_e12, post_execute_elapsed_cycles_before_sub_44dc,
    post_execute_raw_cycles_before_sub_44dc, prize_object_cycles_sub_e12,
    prize_tail_finish_cycles_sub_1007a, read_joypad_advance_with_scheduler, read_joypad_raw_cycles,
    shooter_kid_collision_advance_with_scheduler_sub_219c,
    shooter_kid_collision_raw_cycles_sub_219c,
    shooter_projectile_collision_advance_with_scheduler_sub_226a,
    shooter_projectile_collision_raw_cycles_sub_226a,
    shooter_support_collision_advance_with_scheduler_sub_1f52,
    shooter_support_collision_raw_cycles_sub_1f52, terrain_effects_advance_with_scheduler_sub_5e02,
    terrain_effects_raw_cycles_sub_5e02, terrain_special_scan_advance_with_scheduler_sub_1fa2,
    terrain_special_scan_raw_cycles_sub_1fa2,
    transform_sub_8e0_advance_to_wait_for_vint_poll_with_scheduler,
    wait_for_vint_elapsed_cycles_to_mode_level,
    wait_for_vint_mclk_from_vint_assertion_to_mode_level,
    wait_for_vint_source_cycles_to_mode_level,
};
pub use platforms::{
    PlatformSpriteDispatchInputLoc244C, PlatformSpriteDispatchLoc244C, PlatformSpriteEmitLoc2548,
    PlatformSpriteEntryLoc2548, PlatformSpriteStripLoc24CC, PlatformSpriteStripLoc24F0,
    PlatformSpriteVariantLoc244C, emit_platform_sprite_loc_2548,
    finish_platform_sprite_list_loc_256E, platform_single_sprite_loc_24C0,
    platform_sprite_dispatch_loc_244C, platform_strip_sprites_loc_24CC,
    platform_strip_sprites_loc_24F0, platform_strip_sprites_loc_251C,
};
use std::collections::HashSet;
use std::sync::Arc;
#[cfg(test)]
pub(crate) use trace::opening_level_one_object_counts;
pub use transform_presentation::{
    IRON_KNIGHT_TRANSFORM_TABLE_STRU_7FF8, PresentedTransformPhase, TransformDescriptor,
    TransformHandler, TransformRasterResume, TransformTableSource, ported_forward_transform_passes,
    presented_transform_phase, transform_raster_resume_after_sub_8e0,
};
pub use types::*;
pub use weather::{
    LavaGeyserCameraWindowInputLoc147D6, LavaGeyserFullHeightRowLoc14874,
    LavaGeyserSpriteChunkLoc14BE8, LavaGeyserSpriteTemplateLoc14BE8, MurderWallWarningInputSub5E02,
    MurderWallWarningRenderSub5E02, MurderWallWarningSpriteSub5E02,
    SpecialEffectUnknownPlaneFillRuntimeLoc14E42, StormPatternRuntimeUploadLoc14D32,
    StormSharedArtRuntimeUploadLoc14D8A, compile_murder_wall_warning_sprites_sub_5E02,
    lava_geyser_full_height_rows_for_sprite_row_loc_14874, lava_geyser_full_height_rows_loc_14874,
    lava_geyser_is_inside_camera_window_loc_147D6, lava_geyser_sprite_chunks_loc_14BE8,
};

#[allow(non_snake_case)]
fn heady_metal_breakup_slots_sub_37708_from_anchor_loc_39247(
    anchor_x: u16,
    anchor_y: u16,
) -> [Option<HeadyMetalBreakupPieceSub37708>; 13] {
    // ASM Enemy20_HeadyMetal_Init loc_39247 seeds `$FFFFFB7A` at y+$1F,
    // then `sub_37680` installs the twelve body-piece workers through loc_39266.
    let mut slots = [None; 13];
    let anchor_worker = heady_metal_sprite_init_sub_3764A(anchor_x, anchor_y.wrapping_add(0x001F));
    debug_assert_eq!(anchor_worker.object_meta, 0x0020);
    slots[0] = Some(HeadyMetalBreakupPieceSub37708 {
        sprite_word_y: 0,
        x_pos_fp: u32::from(anchor_worker.x_pos) << 16,
        y_pos_fp: u32::from(anchor_worker.y_pos) << 16,
        x_velocity: 0,
        y_velocity: 0,
    });
    for (index, (x_offset, y_offset)) in HEADY_METAL_SUB_37ABC_BODY_PIECE_OFFSETS_LOC_39266
        .iter()
        .copied()
        .enumerate()
    {
        let worker = heady_metal_sprite_init_sub_37680(
            anchor_x.wrapping_add_signed(x_offset),
            anchor_y.wrapping_add_signed(y_offset),
        );
        debug_assert_eq!(worker.object_meta, 0x0021);
        slots[index + 1] = Some(HeadyMetalBreakupPieceSub37708 {
            sprite_word_y: 0,
            x_pos_fp: u32::from(worker.x_pos) << 16,
            y_pos_fp: u32::from(worker.y_pos) << 16,
            x_velocity: 0,
            y_velocity: 0,
        });
    }
    slots
}

#[allow(non_snake_case)]
fn heady_metal_cached_workers_sub_37708_from_anchor_loc_39247(
    anchor_x: u16,
    anchor_y: u16,
) -> [HeadyMetalCachedWorkerSub37708; 5] {
    // ASM Enemy20_HeadyMetal_Init loc_39247 stores the five final-scatter
    // worker pointers at `$FFFFFB72/$FB76/$FB7A/$FBAE/$FBB2`. The second head
    // slot starts at the boss anchor, `$FB7A` is shifted down `$1F`, and the
    // two eye workers come from the `sub_376B6` offsets near loc_39326.
    let head_1 = heady_metal_sprite_init_sub_3764A(anchor_x, anchor_y);
    let head_2 = heady_metal_sprite_init_sub_3764A(anchor_x, anchor_y);
    let head_3 = heady_metal_sprite_init_sub_3764A(anchor_x, anchor_y.wrapping_add(0x001F));
    let eye_left = heady_metal_sprite_init_sub_376B6(
        anchor_x.wrapping_add_signed(-0x000B),
        anchor_y.wrapping_add_signed(-0x0003),
    );
    let eye_right = heady_metal_sprite_init_sub_376B6(
        anchor_x.wrapping_add(0x000C),
        anchor_y.wrapping_add_signed(-0x0003),
    );
    debug_assert_eq!(head_1.object_meta, 0x0020);
    debug_assert_eq!(head_2.object_meta, 0x0020);
    debug_assert_eq!(head_3.object_meta, 0x0020);
    debug_assert_eq!(eye_left.object_meta, 0x0020);
    debug_assert_eq!(eye_right.object_meta, 0x0020);
    [
        HeadyMetalCachedWorkerSub37708 {
            slot_offset: 0xFB72,
            x_pos_fp: u32::from(head_1.x_pos) << 16,
            y_pos_fp: u32::from(head_1.y_pos) << 16,
            x_velocity: 0,
            y_velocity: 0,
        },
        HeadyMetalCachedWorkerSub37708 {
            slot_offset: 0xFB76,
            x_pos_fp: u32::from(head_2.x_pos) << 16,
            y_pos_fp: u32::from(head_2.y_pos) << 16,
            x_velocity: 0,
            y_velocity: 0,
        },
        HeadyMetalCachedWorkerSub37708 {
            slot_offset: 0xFB7A,
            x_pos_fp: u32::from(head_3.x_pos) << 16,
            y_pos_fp: u32::from(head_3.y_pos) << 16,
            x_velocity: 0,
            y_velocity: 0,
        },
        HeadyMetalCachedWorkerSub37708 {
            slot_offset: 0xFBAE,
            x_pos_fp: u32::from(eye_left.x_pos) << 16,
            y_pos_fp: u32::from(eye_left.y_pos) << 16,
            x_velocity: 0,
            y_velocity: 0,
        },
        HeadyMetalCachedWorkerSub37708 {
            slot_offset: 0xFBB2,
            x_pos_fp: u32::from(eye_right.x_pos) << 16,
            y_pos_fp: u32::from(eye_right.y_pos) << 16,
            x_velocity: 0,
            y_velocity: 0,
        },
    ]
}

#[derive(Debug, Clone)]
struct ResidentGameplayRenderSnapshot {
    trace: NativeTraceFrame,
    block_grid: Option<terrain::BlockGrid>,
}

#[derive(Debug, Clone)]
pub struct NativeRuntime {
    assets: AssetManifest,
    /// Injected byte source for every generated-bundle read at level-load and
    /// boot time (VDP loaders, level/block decoders). `Arc` so the derived
    /// `Clone` shares one immutable source across sweep/recording threads;
    /// `new` default-wires a native [`FsAssetSource`] over `assets.output_root`.
    assets_source: Arc<dyn AssetSource>,
    frame_index: usize,
    time_frames_loc_3038a: u16,
    mode_level_tail_state_loc_892: ModeLevelTailStateLoc892,
    mode_level_tail_work_loc_892: Option<ModeLevelTailWorkLoc892>,
    boot_state: BootState,
    trace_input: FrameInput,
    prior_input: FrameInput,
    previous_input: FrameInput,
    respawn_start_latched_frames: u8,
    respawn_live_handoff_frames: u8,
    title_card_live_handoff_frames: u8,
    recorded_title_card_handoff_delay: bool,
    direct_title_card_handoff_capture: bool,
    /// ASM loc_12E6C/loc_12E96: an early logo-loop Start enters
    /// IntroSequence1 soon enough for the next scripted Start to reach
    /// sub_1C246's first polling window.
    cold_intro_sequence_1_early_start_poll: bool,
    debug_level_entry_suppresses_title_card_live_handoff: bool,
    /// ASM loc_74E0: the ROM's debug level-skip shell runs the level LIVE
    /// behind the (never-shown) title card — measured shell trace run
    /// 1786640409 (#442). The recording bootstrap sets this so frame-loop
    /// subsystems (murder wall) age through the hidden shell.
    /// DEVIATION: play-native's interactive debug title card holds the level
    /// frozen until Start so the picker cannot kill Kid behind the card;
    /// that path leaves this false.
    debug_level_entry_runs_live_shell_loc_74e0: bool,
    recording_oracle_debug_lifecycle_completed: bool,
    recording_oracle_settle_completed: bool,
    start_press_count: usize,
    current_level: LevelId,
    level_start_frame: Option<usize>,
    progression: PlayerProgression,
    helmet: HelmetState,
    paused: bool,
    pause_menu_option: u8,
    pause_menu_arrow_cursor_cmds_loc_6a84: Option<boot::PauseMenuArrowCursorCmdsLoc6A84>,
    pause_toggle_pending: bool,
    pending_pause_restart_round: Option<PauseRestartRoundState>,
    pending_pause_last_life_give_up: bool,
    allow_pausing: bool,
    allow_pausing_fade_frames: u8,
    pending_level_skip: Option<LevelId>,
    pending_level_skip_resume: bool,
    /// ASM `$FFFFFBCC`: palette fade target colour. Teleporter `loc_BCF6`
    /// seeds `$00EE` before branching through `Teleport`; the level-entry
    /// fade shell copies this word into `Palette_Buffer_3`.
    teleport_warp_colour_fbcc: u16,
    /// ASM `sub_73D0` fade progress word `$FFFFF876` for the live level-entry
    /// fullscreen fade worker.
    level_entry_fade_progress_sub_73d0: u16,
    /// ASM `$FFFFF888`: pending palette mask for `Palette_Buffer+$00..+$3E`
    /// during the live `sub_73D0` level-entry fullscreen fade.
    level_entry_pending_palette_0_mask_sub_73d0: u32,
    /// ASM `$FFFFF88C`: pending palette mask for `Palette_Buffer+$40..+$7E`
    /// during the live `sub_73D0` level-entry fullscreen fade.
    level_entry_pending_palette_1_mask_sub_73d0: u32,
    /// ASM `PaletteToDMA_Flag` `$FFFFF890`: `sub_7196` sets this before the
    /// first gameplay pass and `sub_73D0` clears it during that object pass.
    palette_dma_suppressed_f890: bool,
    /// Frame-start `Palette_to_VRAM $970` branch consumed before this frame's
    /// `Execute_Objects` pass can mutate `$FFFFF890`.
    mode_level_palette_dma_sub_970: Option<bool>,
    /// ASM loc_DA4A writes PlayerStart_X/Y and sets `$FFFFFC29` before the
    /// Teleport tail. Load_InGame's loc_11BD2 then preserves those start words
    /// instead of reading the destination level header pair.
    pending_player_start_override_loc_11bd2: Option<(u16, u16)>,
    player_lifecycle: PlayerLifecycle,
    opening_slice: Option<OpeningSliceState>,
    /// ASM `sub_21F8` Kid hitbox globals retained when `loc_224C` sees a
    /// zero first collision-size word and returns without rewriting them.
    retained_kid_hitbox_loc_224c: Option<combat::CollisionRectSub1F52>,
    /// Camera words transferred by ASM Mode_Level ->
    /// Transfer_ScrollDataToVRAM loc_300BA before the current gameplay object
    /// pass can advance the authoritative camera state.
    gameplay_vdp_camera_latch_loc_300ba: Option<CameraState>,
    probe_start_slope_settle_frames: u8,
    starting_level_title_workers: Vec<LevelEntryTitleWorker>,
    starting_level_title_rng: Option<RomRandomState>,
    death_camera_lock: Option<CameraState>,
    death_windup_path: DeathWindupPath,
    death_preserved_object_velocity: Option<(i32, i32)>,
    death_respawn_trace_profile: Option<DeathRespawnTraceProfile>,
    level_skip_transition: Option<LevelSkipTransitionState>,
    death_shell_snapshot: Option<LevelSkipTransitionState>,
    level_complete_gfx_snapshot: Option<LevelCompleteGfxSnapshot>,
    level_entry_camera_handoff: Option<(usize, CameraState)>,
    level_entry_player_handoff: Option<(usize, PlayerState)>,
    level_skip_resume_hold: bool,
    active_enemy_roster: Vec<NativeObjectSnapshot>,
    active_enemies: Vec<ActiveEnemy>,
    /// Enemies killed during this level — (segment_index, layout_index).
    /// Prevents respawn when their spawn point re-enters the activation window.
    /// ASM: per-segment killed bitfield at $FFFFF66A.
    killed_enemies: HashSet<(usize, usize)>,
    /// Native mirror of EnemyStatus_Table. ASM loc_31F9A clears 30 words, but
    /// loc_3200A and Manage_EnemyLoading iterate the unchecked layout count.
    /// ORIGINAL_QUIRK: retain the full six-bit status-index space without
    /// corrupting adjacent native state for buckets containing over 30 entries.
    enemy_status_words: [u16; 0x0040],
    /// ASM `$FFFFFB51`: set after `Manage_EnemyLoading` has completed one pass.
    /// Native currently mirrors it for settled recording/oracle-warps, where
    /// probe frames start after the hidden first pass has already happened.
    enemy_loading_initial_pass_complete: bool,
    /// Enemy Objects linked by the trailing `Manage_EnemyLoading` pass have
    /// not yet run their family initialiser to allocate a GfxObject.
    /// ASM Mode_Level loc_80C -> loc_3691E, then Execute_Objects loc_E1E.
    pending_enemy_gfx_initialisations_loc_3691e: u16,
    /// ASM Manage_EnemyLoading loc_3691E (#419): enemies allocated by the
    /// post-Execute loaders run their family Init on the NEXT object pass
    /// (the init ends in the walker hibernate), so their first behaviour
    /// step lands one pass after the roster appearance. Identities
    /// `(segment_index, layout_index)` allocated this frame map to their
    /// remaining no-step passes; the AI loop decrements one per pass.
    /// Per-family counts: Goat 1 (init ends in the walker hibernate),
    /// Driller 3 (the loc_3691E defer, Enemy1A_Driller_Init's opening
    /// `j_Hibernate_Object_1Frame`, and the loc_3C0D2/loc_3C026 walker-entry
    /// hibernate).
    freshly_loaded_enemies_loc_3691e: std::collections::HashMap<enemies::ActiveEnemyKey, u8>,
    /// Enemy18_Tornado_Init consumes its loc_35326 RNG byte on the first
    /// Execute_Objects pass after loc_3691E allocation, not in the loader tail.
    pending_tornado_initialisations_loc_3691e: HashSet<(usize, usize)>,
    /// Blocks drilled by Driller enemies this frame.
    /// Driller.asm loc_3BE78: checks sprite timer, destroys Smash/Ice blocks ahead.
    drilled_blocks: Vec<DrilledBlockLoc3BE78>,
    active_platforms: Vec<ActivePlatform>,
    active_platform_roster: Vec<NativeObjectSnapshot>,
    active_platform_standing_on: Option<u16>,
    active_lift_platform_standing_on: Option<u16>,
    platform_script_table: Option<wildside_data::PlatformScriptTable>,
    block_grid: Option<terrain::BlockGrid>,
    block_list: Option<terrain::BlockList>,
    block_grid_source: BlockGridSource,
    /// Player has beaten the final level — show victory/ending.
    game_complete: bool,
    block_state: BlockInteractionState,
    projectiles: Vec<EnemyProjectile>,
    /// ASM Enemy20_HeadyMetal_Init installs the `$FFFFFB7A..$FFFFFBAA`
    /// breakup workers once; sub_37708 then mutates and clears those slots.
    pub(crate) heady_metal_breakup_slots_sub_37708:
        Option<[Option<HeadyMetalBreakupPieceSub37708>; 13]>,
    /// ASM `$FFFFFB72/$FB76/$FB7A/$FBAE/$FBB2`: fixed HeadyMetal cached
    /// workers used by the final `sub_3787C/sub_378A2` scatter phase.
    pub(crate) heady_metal_cached_workers_sub_37708: Option<[HeadyMetalCachedWorkerSub37708; 5]>,
    /// ASM sub_3787C/sub_378A2 final-scatter velocities for the five cached
    /// HeadyMetal workers at `$FFFFFB72/$FB76/$FB7A/$FBAE/$FBB2`.
    pub(crate) heady_metal_cached_worker_velocities_sub_3787c: HeadyMetalScatterVelocities,
    /// ASM `$FFFFFB4E`: multi-head boss death counter. Shared death entry
    /// `loc_39EBE` increments it; Shishkaboss skewer pieces launch once it is 3.
    pub(crate) multi_head_boss_death_count_fb4e: u8,
    /// ASM `$FFFFFB72/$FFFFFB74/$FFFFFB76`: per-head boss-eye breakup trigger
    /// words. Boss head routines mirror nonzero `d7` here before sibling
    /// eye-piece loops branch from orbit to detach.
    pub(crate) boss_eye_breakup_trigger_words_fb72_fb74_fb76: [u16; 3],
    diamond_power_rain_slots: Vec<DiamondPowerRainSlot>,
    diamond_power_circle_slots: Vec<helmets::CircleOfDoomSlot>,
    diamond_power_wall_slots: Vec<helmets::WallOfDeathSlot>,
    diamond_power_five_way_slots: Vec<helmets::FiveWayShotSlot>,
    diamond_power_snake_slots: Vec<helmets::SnakeSlot>,
    diamond_power_extra_hp_life_slots: Vec<helmets::ExtraHPLifeSlot>,
    diamond_power_invuln_haze_slots: Vec<helmets::InvulnHazeSlot>,
    diamond_power_object_lists: helmets::DiamondPowerObjectListsSub3F5E8,
    diamond_power_object_ram: helmets::DiamondPowerObjectRamSub3F596,
    diamond_power_invuln_haze_timer: i16,
    /// ROM `FiveWayShotReady` byte, armed by `sub_7428` from Juggernaut
    /// player-state handlers and consumed later by `DiamondPower_Run`.
    five_way_shot_ready: bool,
    /// ASM sub_975C (kid.asm:11286): visual-only Juggernaut companion state
    /// for the `$FFFFF862` GFX object. Native records the step until that
    /// companion exists as a first-class render object.
    juggernaut_companion_state_sub_975c: helmets::JuggernautCompanionStateSub975C,
    last_juggernaut_companion_step_sub_975c: Option<helmets::JuggernautCompanionStepSub975C>,
    /// ROM `KidIsInvulnerable` byte (`$FFFFFA2B`), set only by
    /// `DiamondPower_Init_Invulnerability loc_3FC78` and cleared by the shared
    /// Diamond Power reset tails. Damage flash uses `progression.invulnerable_frames`.
    kid_is_invulnerable: bool,
    diamond_power_snake_head_angle: u16,
    diamond_power_snake_turn_rate: u16,
    diamond_power_snake_movement_speed: u16,
    diamond_power_sprites: Vec<DiamondPowerSpriteSnapshot>,
    diamond_power_sprite_buffer_sub_3f798: Vec<[u16; 4]>,
    diamond_power_buffered_sprite_count_before_compile_sub_3f798: u8,
    diamond_power_number_sprites_sub_3f798: u8,
    eyeclops_hard_beam_object: Option<EyeclopsHardBeamObjectSnapshot>,
    fire_demon_trails: Vec<FireDemonTrailChild>,
    weather: WeatherState,
    /// Native mirror of the ROM's 64-word `Palette_Buffer`.
    /// Most live render paths still use extracted assets directly, but
    /// environment workers mutate this buffer so palette side effects are no
    /// longer trapped in test-only helpers.
    palette_buffer: [u16; 0x40],
    /// Native mirror of the ROM's `Palette_Buffer_2`, used by storm and global
    /// palette fade workers as the source side of `sub_14C0`.
    palette_buffer_2: [u16; 0x40],
    /// Native mirror of the ROM's `Palette_Buffer_3`, used by storm and global
    /// palette fade workers as the target side of `sub_14C0`.
    palette_buffer_3: [u16; 0x40],
    /// ASM Animation_Geyser loc_14B2A: five one-frame hibernations before
    /// rotating `Palette_Buffer+$28/$2A/$2C/$2E`.
    lava_geyser_palette_delay_loc_14b2a: u16,
    rotate_palette_lava_water_loc_1223c: Option<RotatePaletteLavaWaterStateLoc1223C>,
    island_shore_cycle_loc_1202a: Option<IslandShoreCycleStateLoc1202A>,
    transform_resident_island_shore_vram_frames_loc_1202a: Option<(u16, u16, u16)>,
    island_shore_dma_uploads_loc_1202a: Vec<IslandShoreDmaUploadLoc1202A>,
    lava_geyser_art_uploads_loc_14c6a: Vec<LavaGeyserArtUploadLoc14C6A>,
    murder_wall: Option<MurderWallState>,
    /// ASM `$FFFFFAC0`: five-tick murder-wall pattern countdown.
    murder_wall_warning_animation_timer_fac0: u8,
    /// ASM `$FFFFFABF`: two-bit murder-wall pattern index.
    murder_wall_warning_animation_frame_fabf: u8,
    /// Metal block bounce animations in progress.
    block_bounces: Vec<BlockBounceAnim>,
    /// ASM sub_1007A count-only prize-block worker tail after the visible
    /// wobble table has finished but before the coroutine's spawn hand-off
    /// deletes its object/gfx allocation.
    prize_block_worker_tails: Vec<PrizeBlockWorkerTail>,
    /// ASM loc_102B8 flash workers spawned by sub_1007A. Their position comes
    /// from `cell_x * 16 + 8`, `cell_y * 16 - 8`, matching the ROM object.
    prize_block_spawn_flashes: Vec<PrizeBlockSpawnFlash>,
    /// ASM `sub_1D84 loc_1E5E` allocates a `loc_1F08` hit-spark helper for
    /// pass-0 Kid projectile/sword hits, copying the worker's `$1A/$1E`.
    kid_projectile_hit_sparks: Vec<KidProjectileHitSparkLoc1F08>,
    /// ASM sub_E446 hidden-block reveal workers queued by EyeClops beam strip
    /// scans, one `$12C`-frame worker plus foreground-cell upload per reveal.
    pub(crate) hidden_block_fg_update_workers: Vec<terrain::HiddenBlockColumnUpdateSubE446>,
    /// One-shot `FGPlane_UpdateQueueCell` records emitted with sub_E446
    /// reveals and consumed by the following Mode_Level `sub_1596` pass.
    pending_fg_update_cell_queue_sub_1596: Vec<(u16, u16, terrain::Cell)>,
    /// Block destruction fragment effects.
    block_destroy_effects: Vec<BlockDestroyEffect>,
    /// ASM sub_2A4C falling block debris scratch workers.
    block_debris_swarms: Vec<BlockDebrisSwarmSub2A4C>,
    /// Sprite positions emitted by the most recent ASM sub_2A4C live step.
    block_debris_sprite_pieces_sub_2a4c: Vec<(i16, i16)>,
    /// ASM sub_292E cursor `$FFFFFA9C` expressed as a six-slot index.
    block_debris_scratch_cursor_sub_292e: usize,
    /// Floating prize items rising from blocks.
    floating_items: Vec<FloatingItem>,
    /// ASM Prize10Diamonds_Init / Prize10000Points_Init parent coroutines.
    /// These award immediately and own gfx workers instead of a collectible
    /// falling object.
    prize_worker_effects: Vec<PrizeWorkerEffect>,
    /// ASM loc_104AE/loc_105D6: the PrizeHelmet worker sleeps ONE frame
    /// (`j_Hibernate_Object_1Frame`) between GfxObjects_CollisionKid latching
    /// its `$38` word and the wake that consumes it — so the award lands on
    /// the frame AFTER the latch, once the Kid's movement for that frame has
    /// run (the Kid object heads Execute_Objects). Oracle 0x55 (WRAM
    /// f34/f35): the Kid rises to his exact bounce apex 143.0 and headbutts
    /// the (0,6) prize block on the wake frame before the transform freezes
    /// him (#420).
    latched_prize_wake_armed_loc_104ae: bool,
    /// ASM PrizeClock_Init loc_10796 swaps the active clock prize animation
    /// when its countdown reaches `$012C`.
    clock_prize_animation_swaps_loc_10796: Vec<(u16, u16)>,
    /// ASM PrizeContinue_Init loc_1080A swaps the active continue prize
    /// animation when its countdown reaches `$012C`.
    continue_prize_animation_swaps_loc_1080a: Vec<(u16, u16)>,
    /// ASM PrizeHelmet* loc_105D6 deletes after object-side collision, then
    /// Kid remains visible at the loc_A6F8 hibernate point without changing
    /// Current_Helmet.
    prize_helmet_collision_hibernate_delay_frames: u8,
    /// Helmet whose sub_7EB2 hitpoint seed is exposed on the held loc_A6F8
    /// frames after a PrizeHelmet* object-side collision.
    prize_helmet_collision_hibernate_hitpoints_form: Option<PlayerForm>,
    /// Helmet written to Current_Helmet after the old-helmet reverse table
    /// finishes during the latched prize collision hibernate.
    prize_helmet_collision_hibernate_switch_form: Option<PlayerForm>,
    /// ASM `Iron_Knight_block_breaker` ($FFFFFA6B): set by sub_AF7A after a
    /// handled Iron Knight block impact, cleared by sub_DB22 on the normal
    /// grounded clean-up path.
    iron_knight_block_breaker: bool,
    /// Native guard for a same-cell metal rebound: ASM loc_110D0 creates one
    /// sub_11120 worker for the contacted metal block, then a later normal
    /// landing path reaches sub_DB22 and clears the Iron Knight latch.
    iron_knight_last_metal_block_hit: Option<(u16, u16)>,
    /// Prize diamonds award their count later via PrizeDiamondCollected_Init,
    /// not on the touch frame.
    pending_prize_diamond_awards: Vec<u8>,
    /// Visual mirror of ASM PrizeDiamondCollected_Init `$3E/$42` HUD-flight
    /// objects. The award delay above remains the authoritative count timing.
    pending_prize_diamond_hud_flights: Vec<PrizeDiamondHudFlight>,
    /// Visual mirror of ASM PrizeClockCollected_Init loc_FF92. The timer bonus
    /// and clock counter increment happen after this 32-frame HUD flight.
    pending_prize_clock_hud_flights: Vec<PrizeClockHudFlight>,
    /// ASM Mini_Hopping_Skull.asm sub_3F3F0 spawns a transient worker that
    /// hibernates 15 frames, then optionally plays the scream SFX before
    /// deleting itself.
    pending_sound_effects: Vec<PendingSoundEffect>,
    /// Internal log used by focused sound-worker tests.
    played_sound_effects: Vec<QueuedSoundEffect>,
    /// Internal log preserving ASM `j_PlaySound`/`j_PlaySound2` command bytes.
    played_sound_commands: Vec<QueuedSoundCommand>,
    /// Internal log of ASM `j_PlaySound` BGM command bytes.
    played_bgm_ids: Vec<u8>,
    /// Internal log of ASM sub_E1330/sub_E1338 music-speed command bytes.
    played_music_speed_updates: Vec<u8>,
    /// Enemy death particle effects.
    enemy_death_effects: Vec<EnemyDeathEffect>,
    /// ASM Dragon_Flying.asm loc_3492C hibernates before allocating the
    /// walking-dragon child; keep that delayed child outside the active roster.
    pending_dragon_death_children: Vec<PendingDragonDeathChild>,
    /// Visual-only dying flying-dragon GfxObject kept alive until `stru_345E4`
    /// completes. It is appended to the trace roster but is not interactive.
    pending_dragon_death_animations: Vec<PendingDragonDeathAnimation>,
    /// Set each frame if a stomp occurred.
    stomped_this_frame: bool,
    /// ASM `$FFFFFA74` (kid.asm:936): "Kid touched an enemy LAST frame". Each
    /// Mode_Level frame copies `$FFFFFA75` here before the
    /// GfxObjects_CollisionKid passes. While set, GfxObjects_CollisionKid_Collide
    /// (kid.asm:3384) skips the loc_1C42 side classifier entirely and routes to
    /// loc_1D4C: `colid_kidbelow` written to the Kid slot only — sustained
    /// contact can never register a stomp and never writes the enemy colid.
    kid_touched_enemy_last_frame_loc_fa74: bool,
    /// ASM `$FFFFFA75` (kid.asm:3383): `st` on any Kid/enemy hitbox overlap in
    /// GfxObjects_CollisionKid_Collide, cleared each frame after the FA74 copy.
    kid_touched_enemy_this_frame_loc_fa75: bool,
    /// `kidabove` collisions resolve on the next frame, not immediately on the
    /// overlap frame.
    pending_stomp_bounce: Option<PendingStompBounce>,
    /// ASM Mode_Level writes Kid/object collision words near the end of the
    /// frame; Character_CheckCollision consumes them on the next object pass.
    pending_player_contact_damage: Option<PendingPlayerContactDamage>,
    /// Native's decomposed hidden-shell GFX pass runs before its gameplay tail.
    /// Pin the producing frame so that tail cannot consume the collision early.
    hidden_shell_contact_latched_frame_loc_1cf2: Option<usize>,
    /// Collision words emitted from object execution after this frame's
    /// Character_CheckCollision point. Promoted to
    /// `pending_player_contact_damage` once that consume point has passed.
    delayed_player_contact_damage: Option<PendingPlayerContactDamage>,
    /// ASM sub_1F52 writes `colid_kidbelow` when Kid overlaps a live shooter
    /// barrel rectangle. Native detects the rectangle before movement, then
    /// consumes the collision after terrain has resolved for this frame.
    shooter_barrel_damage_latched_loc_1f52: bool,
    /// Landing DragonWalking grazes already spent one frame in the fresh
    /// falling-to-ground hand-off, so the following grounded frame consumes
    /// them without adding the normal pending-contact delay.
    pending_grounded_dragon_landing_graze: Option<PendingPlayerContactDamage>,
    pending_enemy_collisions: Vec<PendingEnemyCollision>,
    /// ASM Init_RNG / Get_RandomNumber_byte global gameplay RNG state.
    rom_random: RomRandomState,
    /// Count of blocks hit by headbutt this frame.
    blocks_hit_this_frame: u8,
    /// Sound test play pending (consumed each frame).
    sound_test_play_pending: bool,
    /// Cached Sega screen art and palette data (loaded once, reused each frame).
    sega_screen_data: Option<vdp::SegaScreenData>,
    /// Cached intro sequence art/tilemap/palette for sub_1B850 segments.
    intro_sub_1b850_data: Option<vdp::IntroSub1B850Data>,
    /// Cached title screen art/tilemap/palette for sub_1B2A4.
    title_screen_data: Option<vdp::TitleScreenData>,
    /// Cached Load_TitleCard art/tilemap/palette by low theme nibble.
    title_card_data: Option<vdp::TitleCardData>,
    /// Software VDP state for non-gameplay screens (Sega logo, intro, title).
    /// Populated each frame during boot modes so the VDP renderer can produce
    /// accurate frames; `None` during gameplay where the pre-extracted asset
    /// pipeline handles rendering.
    vdp_state: Option<vdp::VdpState>,
    /// ASM sub_914 / sub_924 nest counter at `$FFFFF805`: active Z80 stop
    /// requests. Zero means the Z80 bus may run; non-zero keeps it halted.
    #[allow(dead_code)]
    z80_stop_request_depth: u8,
    /// ASM sub_914 / sub_924 writes SR directly before stopping/starting the
    /// Z80 so later gameplay code can observe the current interrupt mask.
    #[allow(dead_code)]
    z80_guard_sr: u16,
    /// Native mirror of the Z80 run state toggled by ASM `j_Stop_z80` /
    /// `j_Start_z80`.
    #[allow(dead_code)]
    z80_running: bool,
    /// ASM OptionScreen_Input `Option_2PInput`: false = one controller, true = two.
    ///
    /// ASM ReadJoypad (kid.asm:1332): routes Ctrl_Held/Ctrl_Pressed to
    /// per-player variables depending on this flag and Current_player:
    ///   - loc_B8C (kid.asm:1342): default path (1P mode or Current_player==0)
    ///     stores `Ctrl_Held → Ctrl_1_Held`, `Ctrl_Pressed → Ctrl_1_Pressed`.
    ///   - 2P + Current_player!=0: stores to Ctrl_2_Held/Ctrl_2_Pressed.
    ///   - loc_BB6 (kid.asm:1358): common post-cheat path wraps the hardware
    ///     poll with sub_914/sub_924 and reads ports $A10003/$A10005 into
    ///     Ctrl_1/Ctrl_2.
    ///
    /// Native: single FrameInput consumed directly — no per-player routing.
    options_two_player_two_controllers: bool,
    /// ASM OptionScreen_Input `Options_Suboption_Controls`: 0..=5.
    options_controls_scheme: u8,
    /// ASM OptionScreen_Input `Option_SpeedButton`: false = normal action, true = fast.
    options_speed_button_fast: bool,
    /// ASM sub_1CC88 (kid.asm:31808): option-menu worker byte toggled by
    /// `not.b ($FFFFFB5C).w` before applying the symmetric `$FFFFFB5E` offset.
    options_menu_panel_toggle_byte: u8,
    /// ASM loc_1CB2C / sub_1CC88: left option-menu worker word at `$24(a0)`.
    options_menu_left_panel_x: i16,
    /// ASM loc_1CB2C / sub_1CC88: right option-menu worker word at `$24(a1)`.
    options_menu_right_panel_x: i16,
    /// ASM loc_1CB2C / sub_1CC88: shared signed offset seeded from
    /// `($FFFFFB5E).w`.
    options_menu_panel_delta: i16,
    level_timer: LevelTimer,
    /// Whole seconds encoded into the timer digit entries most recently built
    /// in `Sprite_Table` by ASM Make_SpriteAttr_HUD loc_BEE8.
    hud_timer_sprite_seconds_loc_bee8: u16,
    /// ASM Make_SpriteAttr_HUD loc_BDC0 snapshots Number_Diamonds before
    /// Execute_Objects can complete a collected-diamond worker.
    hud_diamond_count_loc_bdc0: u16,
    /// Demo bootstrap's presented `Make_SpriteAttr_HUD loc_BE82` clock.
    /// `loc_3038A` remains suspended during this interval, so it cannot reuse
    /// the authoritative level timer without perturbing unrelated presentation
    /// state such as the `$DD40` HUD DMA phase.
    demo_hud_timer_presentation_loc_be82: Option<LevelTimer>,
    /// Result of the demo's leading Mode_Level HUD tick, consumed by the
    /// later native hazard pass when the Kid object is live.
    pending_demo_timer_tick_result_loc_be82: Option<bool>,
    hud_animation_state: boot::HudAnimationStateSub44DC,
    /// ASM sub_44DC loc_4546/loc_4626 emits these HUD uploads in execution
    /// order. This records the frame-local commands separately from their
    /// resulting resident VRAM pages, so a later raster scheduler can decide
    /// their visible scanout position from Mode_Level workload.
    hud_dma_events_sub_44dc: Vec<HudDmaEventSub44DC>,
    /// Raster placements derived from an explicit CPU/VDP boundary seed.
    hud_raster_upload_timings_sub_44dc: Vec<HudRasterUploadTimingSub44DC>,
    /// One-shot hardware boundary for the next live `Execute_Objects sub_E12`.
    mode_level_raster_seed_sub_e12: Option<vdp::ModeLevelRasterSeedSubE12>,
    /// Source workload consumed by the current Mode_Level VBlank prefix and
    /// suffix, retained after one-shot producer queues are drained.
    mode_level_vblank_work_before_sub_e12: Option<ModeLevelVblankWorkBeforeSubE12>,
    /// Classified CPU/beam boundary at ASM `sub_44DC` entry.
    mode_level_cpu_raster_cursor_sub_44dc: Option<vdp::ModeLevelCpuRasterCursor>,
    /// Source-derived raw CPU work for the classified live `sub_44DC` path.
    mode_level_hud_raw_cycles_sub_44dc: Option<M68kCycleCount>,
    /// Source-ordered workload captured immediately before the live
    /// `Execute_Objects sub_E12` pass mutates any linked worker.
    mode_level_work_before_hud_sub_44dc: Option<ModeLevelPreHudWorkLedger>,
    /// Source-ordered workload after the live `sub_44DC` HUD pass.
    mode_level_work_after_hud_sub_44dc: Option<ModeLevelPostHudWorkLedger>,
    /// Frame-local ASM `Manage_PlatformLoading loc_2744` branch, captured
    /// before native advances the platform-layout cursor.
    mode_level_platform_loader_work_loc_2744: Option<ModeLevelPlatformLoaderWorkLoc2744>,
    /// Frame-local ASM `ReadJoypad loc_C22` demo-stream branch, captured before
    /// native advances the persistent Demo counter.
    demo_read_joypad_work_loc_c22: Option<ModeLevelDemoReadJoypadWorkLocC22>,
    /// The selected `off_7B0AC` page last transferred by ASM sub_5D4A to
    /// HUD-diamond VRAM `$DD40`; presentation timing is modelled separately.
    gameplay_hud_vram_state: vdp::GameplayHudVramState,
    /// `$DD40` page resident when `Kid_Transform` hibernates Mode_Level before
    /// its next `sub_44DC` pass.
    transform_resident_hud_diamond_vram_page_loc_5d4a: Option<u8>,
    /// ASM sub_44DC loc_4586's currently resident life-icon art page at VRAM
    /// `$4620`. This differs from `$FFFFFA5E` after the animation table wraps.
    hud_life_icon_vram_page_loc_4586: u8,
    /// Camera Y look-ahead accumulator ($FFFFFB58): +1 per frame while down held
    /// and grounded, -8 per frame on release, clamped to [0, 30].
    y_look_offset: i16,
    /// ASM $FFFFFA2E: Kid's y_pos from the previous camera-follow invocation.
    /// Used by loc_726C to detect upward motion near the level ceiling.
    camera_last_kid_y: i16,
    /// Set to true when player takes damage during current level (for no-hit bonus).
    level_damage_taken: bool,
    /// Exact per-frame damage latch, mirroring ROM `Just_received_damage`.
    just_received_damage: bool,
    /// Frame-entry snapshot of `Just_received_damage`. Mode_Level collision
    /// observes this before the independent loc_BC6E flash worker can clear
    /// the live latch later in the same frame.
    recent_damage_gate_held_this_frame_loc_bc6e: bool,
    /// Debug-only attribution for the hostile projectile that triggered Kid
    /// damage during the current frame.
    last_projectile_damage_probe: Option<String>,
    /// Debug-only Drips splash reseed trace for ASM loc_33488 -> loc_3340E.
    last_drips_splash_reseed_probe: Option<String>,
    /// ASM Lion.asm loc_3E192/sub_3E2CE child worker remains count-visible in
    /// the hit frame even when native has consumed the projectile collision.
    lion_projectile_object_only_tail_count: u16,
    /// ASM Drips.asm loc_33488/loc_3392C mirror of `$FFFFFB42`: incremented
    /// when loc_335D8 allocates a surface crawler and decremented only after
    /// the crawler's finish animation reaches loc_3392C. Native cannot derive
    /// this from visible helper projectiles because loc_338F8 also spawns a
    /// separate Tar-style visual worker.
    drips_surface_crawler_count_loc_fffffb42: usize,
    /// Set to true when player collects any prize from a block (disqualifies no-prize bonus).
    level_prize_collected: bool,
    /// Set to true when `apply_level_completion_bonuses` awarded the
    /// ASM loc_D660 speed bonus (10,000 points) for the current
    /// level-complete sequence. Cleared on level restart.
    level_speed_bonus_awarded: bool,
    /// Amount added by `apply_level_completion_bonuses` for the
    /// ASM loc_D60E path bonus (PathBonus_Values[level] × 1000) on the
    /// current level-complete sequence. Cleared on level restart.
    level_path_bonus_amount: u32,
    /// ASM loc_D17E copies live `(Score).w` into the EOL score object before
    /// clearing the live score word. Native keeps the copied value here for
    /// render-facing traces while progression retains the awarded total.
    level_complete_initial_score: u32,
    /// Number of clock pickups collected during current level (for completion time calc).
    clocks_collected: u16,
    /// ASM Prize10000Points_Init loc_10AC2 / main-loop loc_7452 share the
    /// `$FFFFFB4C` countdown that starts the Hundred K Trip event in level 4.
    hundred_k_trip_timer: u16,
    /// Title screen phase timer: counts down to demo playback in both the
    /// pre-menu attract phase and the interactive title menu. The interactive
    /// loop resets this on Up/Down navigation, matching the ASM.
    ///
    // NOTE (was TODO, resolved 2026-06-04 — timing subsumed, not a gap):
    // ASM loc_1BAB4 (kid.asm) is the pre-menu intro bridge — a
    // `jsr Hibernate_Object_1Frame; bsr sub_1C246; dbf d0,loc_1BAB4` loop for
    // d0=$B8 (184) frames (set at loc_1BAB0) before Obj_TitleMenu's setup, with
    // each frame polling Start via sub_1C246.
    //
    // Native folds intro→title timing into oracle-calibrated duration constants
    // rather than simulating the per-object hibernate loop, and the resulting
    // transition timing is frame-exact vs jgenesis. The $B8 wait is captured by
    // `INTRO_SEQUENCE_6_TITLE_ATTRACT_LOCK_FRAMES` (constants.rs): oracle frame
    // probes for the cold IntroSequence6 route show the Start latch is not
    // consumed by sub_1C246 until cold frame 2634 — 237 local frames after
    // Game_Mode first becomes $28 at frame 2396 (the $B8 wait plus the
    // surrounding loc_1B37C/sub_1B43A worker overhead). The full
    // `Load_IntroSequence6 -> loc_1BAB0 -> loc_1BAEA -> loc_1BB02` cold path is
    // locked to enter DemoPlay on global frame 3270 by
    // `cold_boot_intro_sequence_6_enters_demo_play_on_frame_3270`, with the
    // sub_1C246 Start-poll modelled and verified by
    // `cold_boot_intro_sequence_6_start_keeps_loc_1b37c_shell_until_loc_1b5ce`.
    // So the loop body itself is intentionally not simulated frame-by-frame; its
    // observable timing is oracle-verified through these constants and tests.
    //
    // NOTE (was TODO, resolved 2026-06-04 — NOT a gap): ASM loc_1C5A4 (kid.asm)
    // is the shared init tail for the two slide-in GfxObjects (sub_1C572 enters
    // from x=$3A moving +1, loc_1C58C from x=$DE moving -1). loc_1C5A4 sets
    // `st $13(a3)`, y_pos=$C8, vram_tile=$140, then `Hibernate_Object $3C`
    // before a per-frame `add.w d0,x_pos` slide loop (loc_1C5BE).
    //
    // These objects are allocated ONLY by Load_IntroSequence3 (kid.asm:29085-
    // 29088), not by Obj_TitleMenu — they are the Wild Side intro's sliding door
    // halves, not a title-screen decoration (the earlier wording on this field
    // was wrong). Native ALREADY simulates them per-frame: `boot.rs`
    // `title_menu_sliding_sprites_sub_1C572` seeds both halves with the exact
    // loc_1C5A4 values (y_pos=$C8, vram_tile=$140, hibernate=$3C, moving=$42 =
    // $41+1, x=$3A/+1 and x=$DE/-1) and `vdp.rs`
    // `push_intro_sequence_3_sliding_sprites_sub_1C572` advances their x_pos one
    // pixel per frame into the IntroSequence3 VDP sprite table. So this is NOT
    // rendered from a static tilemap snapshot and is NOT a gap. Covered by
    // `boot.rs` sub_1c572_* tests and `vdp.rs`
    // intro_sequence_3_sub_1C572_draws_and_slides_door_halves.
    title_idle_frames: u16,
    /// Title screen local elapsed frames, used for diagnostics and demo timing.
    title_screen_elapsed_frames: usize,
    /// Cold-boot intro-return title rebuild uses Obj_TitleMenu's hidden shell
    /// staging before the later title pose becomes ready.
    title_screen_uses_intro_return_hidden_shell: bool,
    /// ASM sub_1C246 return path that owns the title RNG scheduler. This is
    /// stored explicitly because global ready-frame equality is not a lineage.
    title_screen_intro_return_lineage: Option<TitleScreenIntroReturnLineage>,
    /// The cold IntroSequence6 Start-skip route keeps Obj_TitleMenu's cursor
    /// as the trace-facing gfx object after the hidden title rebuild. Earlier
    /// title-entry routes keep reporting the standard title pose.
    intro_sequence_6_start_skip_cursor_trace_priority: bool,
    /// IntroSequence6 Start-skip enters `$2C` through the `$28` sub_1C246
    /// branch while stale loc_1BCD0 slot-2 WRAM survives longer than the
    /// direct `$24` intro-sequence-5 title return.
    intro_sequence_6_start_skip_intro_sequence_5_pose_carry: bool,
    /// Local IntroSequence6 VDP lineage carried across a `$28 -> $2C`
    /// Start-skip while the sibling hidden-shell workers remain visible.
    intro_sequence_6_start_skip_natural_vdp_carry: Option<IntroSequence6StartSkipNaturalVdpCarry>,
    /// Direct `$24` IntroSequence5 title returns keep the current VDP image for
    /// a few frames before the hidden title rebuild blacks it out.
    intro_sequence_5_title_return_vdp_snapshot: Option<vdp::VdpState>,
    /// ASM sub_1B8F4 loc_1B924 title return keeps the final IntroSequence1
    /// worker image resident while loc_6E2 runs Pal_FadeOut.
    intro_sequence_1_title_return_vdp_snapshot: Option<vdp::VdpState>,
    /// ASM sub_1B988 -> sub_1C246 -> loc_1C264 title return keeps the final
    /// IntroSequence4 image resident while loc_6E2 runs Pal_FadeOut.
    intro_sequence_4_title_return_vdp_snapshot: Option<vdp::VdpState>,
    /// ASM `$FFFFFC82`: title/intro region selector. Load_IntroSequence5 calls
    /// sub_1C278 before selecting its helmet art and palette overlay.
    title_screen_region_byte: u8,
    /// First global boot frame where the title pose becomes visible in trace output.
    title_screen_ready_frame: usize,
    /// Last global boot frame of the initial title attract lock.
    title_menu_arm_frame: usize,
    /// False during the initial title attract wait where only Start matters.
    /// True once the Start press arms the real interactive title menu.
    title_menu_active: bool,
    /// True while `Obj_TitleMenu` keeps its 15-frame input lock after Start.
    /// VDP map/cursor linking has the independent two-pass clock below.
    title_menu_cursor_pending: bool,
    /// First frame whose final VDP state contains `Obj_TitleMenu loc_1BB52`'s
    /// menu map and cursor. Input remains locked independently until
    /// `title_menu_arm_frame`.
    title_menu_vdp_upload_frame_loc_1bb52: Option<usize>,
    /// Hidden title Start consumed before trace-facing readiness. The ROM's
    /// title-loop countdown is live, but the next Start should expose the
    /// interactive menu rather than selecting the current row.
    title_menu_hidden_prearm_pending: bool,
    /// Pending title-menu selection (0=1P, 1=2P, 2=options) applied on the
    /// following frame to match Obj_TitleMenu's deferred hand-off.
    title_menu_selection_pending: Option<u8>,
    /// ASM Obj_TitleMenu `Title_InputLoop` hibernates once before the Start
    /// edge flips into `Game_Mode=$0008`, so visible level entry lands one
    /// traced frame after the pending selection wakes.
    title_menu_selection_delay_frames: usize,
    /// ASM Obj_TitleMenu `loc_1BBD6` jumps into `loc_6E2`, but Load_TitleCard
    /// does not immediately replace the resident title/menu VDP image.
    title_menu_level_entry_vdp_carry: Option<(vdp::VdpState, usize)>,
    /// ASM Obj_TitleMenu `loc_1BB02` jumps into `loc_6E2`, retaining the
    /// completed title image while Pal_FadeOut runs before Load_DemoPlay.
    demo_entry_title_vdp_snapshot: Option<vdp::VdpState>,
    /// Demo playback reads inputs after game logic (ASM ReadJoypad path), so
    /// the byte fetched this frame becomes the controller state for the next
    /// frame.
    demo_buffered_input: Option<FrameInput>,
    /// Demo Start abort uses a hibernating object coroutine (`loc_119D8`):
    /// the live Start press arms the return, then title only becomes visible
    /// two frames later when that object wakes and flips Game_Mode.
    pending_demo_title_return: u8,
    /// Once the wake-up lands, preserve the current frame's updated gameplay
    /// slice under title mode instead of freezing the previous frame.
    demo_title_return_after_frame: bool,
    /// Demo auto-timeout runs in ReadJoypad after the current gameplay frame,
    /// so the hand-off should not become visible until the next native step.
    demo_auto_timeout_return_pending: bool,
    /// Demo 2's timeout frame retains the previous trace-facing gameplay
    /// snapshot even though Execute_Objects still performs its leading RNG
    /// draw before ReadJoypad reaches loc_C0A.
    demo_auto_timeout_freeze_gameplay_frame: bool,
    /// The ROM auto-timeout path clears Game_Mode to 0 instead of taking the
    /// paused title helper used by manual Start aborts.
    demo_auto_timeout_handoff_active: bool,
    /// The demo auto-timeout hand-off preserves the final jump movement word
    /// into the first mode-0 timeout frames.
    demo_auto_timeout_handoff_preserves_jump_movement: bool,
    /// Whether Load_SegaScreen enters before or after its first outer pass.
    demo_auto_timeout_mode_0_entry: DemoAutoTimeoutMode0Entry,
    /// Timeout restart hands the generic intro state machine a different
    /// sequence-1 duration than cold boot.
    demo_auto_timeout_intro_sequence_1_frames_override: Option<usize>,
    /// Timeout restart also keeps intro sequence 2 alive much longer than the
    /// shared post-sequence-1 attract timer.
    demo_auto_timeout_intro_sequence_2_frames_override: Option<usize>,
    /// Timeout restart keeps intro sequence 3 alive far longer than cold boot.
    /// Its direct title return is still Start-driven; the natural timeout
    /// path continues through the later intro sequences.
    demo_auto_timeout_intro_sequence_3_frames_override: Option<usize>,
    /// ASM sub_1B988 / loc_1B9B6 hibernate intro sequence 4 for `#$21B+1`
    /// frames before exposing intro sequence 5 on the timeout-restart path.
    demo_auto_timeout_intro_sequence_4_frames_override: Option<usize>,
    /// ASM loc_1BA28 / sub_1C204 / loc_1BA5C keep timeout-restart intro
    /// sequence 5 alive far longer than the shared 80-frame cold-boot timer.
    demo_auto_timeout_intro_sequence_5_frames_override: Option<usize>,
    /// A Start-driven timeout-restart intro-sequence-3 title jump can still
    /// pass through the shared deferred Start→title skip countdown before
    /// title mode becomes visible, so the later handoff must remember to keep
    /// the intro pose.
    timeout_restart_title_intro_sequence_3_return_pending: bool,
    /// ASM sub_1C246 can also jump straight from the carried intro-sequence-4
    /// camera hand-off into title after the shared 2-frame deferral.
    timeout_restart_title_intro_sequence_4_return_pending: bool,
    /// ASM sub_1C246 can also jump straight from timeout-restart intro
    /// sequence 5 into title mode after the shared 2-frame Start deferral.
    timeout_restart_title_intro_sequence_5_return_pending: bool,
    /// ASM loc_1BCD0 has replaced the five-pose GfxObject roster with the
    /// late loc_1BE3C/loc_1BE88 particle roster before entering title mode.
    timeout_restart_title_intro_sequence_5_particle_roster: bool,
    /// Demo abort returns to title without rebuilding Kid/camera RAM
    /// immediately; oracle traces keep exposing the live gameplay snapshot
    /// under title mode until later title objects take over.
    preserve_demo_title_snapshot: bool,
    /// Level whose gameplay VDP image remains resident while manual demo
    /// return runs Pal_FadeOut before loc_6E2 clears VRAM.
    demo_title_visual_level: Option<LevelId>,
    /// Terminal gameplay raster state presented while loc_119D8 / ReadJoypad
    /// loc_C0A has entered title mode but loc_6E2 is still fading the planes.
    demo_title_render_snapshot: Option<Box<ResidentGameplayRenderSnapshot>>,
    demo_title_snapshot_object_counts: Option<(u16, u16)>,
    /// The timeout-restart intro-sequence-3 return reaches title mode before
    /// the standard title pose appears; oracle traces keep the last intro pose
    /// alive for the first title frames, then hide Kid again.
    timeout_restart_title_intro_sequence_3_pose: bool,
    /// ASM sub_1C246 enters title mode before the title objects rebuild, so
    /// the timeout-restart intro-sequence-5 slot-2 pose survives briefly into
    /// title mode before Kid blanks again.
    timeout_restart_title_intro_sequence_5_pose: bool,
    /// The later intro-sequence-4 Start return reaches title mode while the
    /// carried `sub_1B988` camera hand-off is still latched.
    timeout_restart_title_intro_sequence_4_camera_carry: bool,
    /// Resident IntroSequence3 VDP copied across `sub_1C246 -> loc_1C264` so
    /// `loc_6E2` can present Pal_FadeOut before clearing VRAM.
    timeout_restart_intro_sequence_3_title_return_vdp_snapshot: Option<vdp::VdpState>,
    /// Resident IntroSequence2 VDP copied across `loc_1BA02 -> sub_1C246 ->
    /// loc_1C264` so `loc_6E2` can present Pal_FadeOut before clearing VRAM.
    intro_sequence_2_title_return_vdp_snapshot: Option<vdp::VdpState>,
    /// Resident IntroSequence2 VDP presented by `Load_IntroSequence3 ->
    /// sub_1B93E -> sub_1C204` before the rebuilt Wild Side scene fades in.
    intro_sequence_3_handoff_vdp_snapshot: Option<vdp::VdpState>,
    /// Resident IntroSequence3 VDP copied across `loc_1B96A -> Game_Mode=$20`
    /// so `loc_6E2` can present Pal_FadeOut before Load_IntroSequence4.
    intro_sequence_4_handoff_vdp_snapshot: Option<vdp::VdpState>,
    /// Resident IntroSequence4 VDP copied across `sub_1B988 loc_1B9B6 ->
    /// Game_Mode=$24` so `loc_6E2` can present Pal_FadeOut before
    /// Load_IntroSequence5 clears VRAM.
    intro_sequence_5_handoff_vdp_snapshot: Option<vdp::VdpState>,
    /// The later Start-skip intro-return lineage reaches the same
    /// `sub_1C246 -> Game_Mode=$2C` hand-off, but oracle keeps the rebuilt
    /// title pose hidden for one extra frame before the standard title pose
    /// returns.
    timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay: bool,
    /// ASM loc_1B96A switches to `Game_Mode=$20` before Load_IntroSequence4 /
    /// sub_1B988 clear the old intro-sequence-3 roster, so timeout-restart
    /// intro sequence 4 briefly keeps the previous pose visible. The same
    /// latch also carries sub_1B988's `Camera_Y_pos=4` hand-off through the
    /// later intro-sequence-5 window until title setup takes over.
    timeout_restart_intro_sequence_4_pose_carry: bool,
    /// After the live snapshot is fully cleared, oracle traces still keep Kid
    /// hidden for a longer title-object rebuild window on the demo-return path.
    hide_demo_return_title_player: bool,
    /// Timeout-restart demo playback ignores queued manual Start aborts until
    /// the restarted attract demo reaches its live gameplay hand-off.
    ignore_demo_start_abort_until_gameplay: bool,
    /// The active demo started from the timeout-restart title shell. It keeps
    /// the late demo-1 velocity sample, but oracle does not expose the older
    /// cold-boot controller pause/count sample on that lineage.
    demo_started_from_timeout_restart_shell: bool,
    /// The current mode-$28 title shell came from a timeout-restart intro
    /// replay rather than the original cold boot.
    intro_sequence_6_entered_from_timeout_restart: bool,
    /// The next demo start comes from a Start-skip intro return rather than
    /// the cold-boot attract path.
    next_demo_uses_start_skip_intro_return_camera_wrap: bool,
    /// Active demo bootstrap should expose the Start-skip intro-return hidden
    /// camera wrap instead of the cold-boot row.
    demo_uses_start_skip_intro_return_camera_wrap: bool,
    /// The next demo start comes from the generic `loc_119D8 -> loc_6E2`
    /// demo-title return rather than the cold-boot attract path.
    next_demo_uses_demo_title_return_camera_wrap: bool,
    /// Active demo bootstrap should expose the demo-title-return hidden camera
    /// wrap instead of the cold-boot row.
    demo_uses_demo_title_return_camera_wrap: bool,
    /// Load_IntroSequence6 can also flow into Obj_TitleMenu `loc_1BB02 ->
    /// Load_DemoPlay -> Load_InGame`; oracle keeps that timeout-restart
    /// lineage's demo-1 hidden wrap distinct from cold boot and the earlier
    /// Start-skip intro return.
    demo_uses_intro_sequence_6_hidden_camera_wrap: bool,
    /// A later attract demo can also start from a title shell that already had
    /// loc_1B5CE's standard pose restored, either on the timeout-restart
    /// IntroSequence6 lineage or the later intro-sequence-4 title return.
    /// When that demo returns to title, oracle re-exposes the pose two frames
    /// earlier than the generic demo-title rebuild.
    demo_uses_intro_sequence_6_early_title_restore: bool,
    /// A later demo auto-timeout can re-enter intro sequence 1 after the
    /// timeout-restart shell already restored loc_1B5CE's standard title pose.
    /// If Start skips that intro-1 revisit back to title, oracle exposes the
    /// rebuilt title pose two local frames earlier than the generic
    /// intro-return settle window.
    timeout_restart_intro_sequence_1_uses_early_title_restore: bool,
    /// ASM Obj_TitleMenu `loc_1BB02 -> Load_DemoPlay -> Load_InGame` keeps a
    /// distinct demo-2 preload row only on the later IntroSequence6 revisit
    /// that still carries the old `loc_1BCD0` slot-2 pose but no longer uses
    /// the earlier hidden-camera wrap.
    demo_uses_intro_sequence_6_later_demo_2_preload_row: bool,
    /// Active demo-2 bootstrap came from the original cold-boot
    /// IntroSequence6 shell rather than a later timeout-restart revisit.
    demo_uses_cold_intro_sequence_6_demo_2_preload_rows: bool,
    /// Active demo-1 bootstrap came from the cold demo-timeout
    /// IntroSequence6 shell and still exposes the controller-side `$60000`
    /// y-velocity sample at the `loc_1BB02 -> Load_DemoPlay` hand-off.
    demo_uses_intro_sequence_6_timeout_return_velocity_sample: bool,
    /// Active demo-2 bootstrap came from the direct timeout-restart
    /// intro-sequence-5 title return. That `sub_1C246 -> loc_1C264` lineage
    /// exposes a lower one-frame camera preload row than the generic demo-2
    /// attract path.
    demo_uses_intro_sequence_5_title_return_preload_row: bool,
    /// ASM loc_1B5CE has already restored the standard title pose before a
    /// later timeout-restart demo starts. When that lineage returns to title,
    /// oracle exposes the pose two frames earlier than the generic demo-title
    /// rebuild window before the usual hide/rebuild choreography finishes.
    demo_title_return_uses_intro_sequence_6_early_title_restore: bool,
    /// ASM sub_1B8F4 / loc_1B920 can re-enter the title shell via
    /// sub_1C246's deferred Start skip. Oracle later exposes that demo's
    /// timeout-restart `loc_1BCD0` slot-2 pose one frame earlier than the
    /// plain timeout path, so keep that lineage alive through the demo's
    /// later auto-timeout return.
    demo_timeout_restart_uses_intro_return_slot_2_early_visibility: bool,
    /// Only the earlier Start-skip intro-return lineage keeps intro sequence 5
    /// alive for one extra shared fade window before `loc_1BA5C` reaches its
    /// `move.w #$28,(Game_Mode).w` hand-off. The later intro-sequence-6 title
    /// restore still inherits the one-frame-earlier slot-2 visibility, but it
    /// must not reuse this extra intro-sequence-5 timeout delay.
    demo_timeout_restart_uses_start_skip_intro_sequence_5_delay: bool,
    /// A later attract demo that starts after IntroSequence6 has already
    /// restored the standard title pose reaches the next timeout-restart
    /// `loc_1BA5C -> Game_Mode=$28` edge one frame earlier than the earlier
    /// Start-skip revisit, even though both lineages share the same early
    /// `loc_1BCD0` slot-2 visibility.
    demo_timeout_restart_uses_intro_sequence_6_early_intro_sequence_5_handoff: bool,
    /// Next Load_IntroSequence3 roster index from ROM byte `$FFFFFC83`.
    next_intro_sequence_3_roster_index: usize,
    /// Active/last Load_IntroSequence3 roster used by trace-visible intro pose.
    active_intro_sequence_3_roster_index: usize,
    /// Alternates between demo 1 and demo 2 (ASM $FFFFFBC8 toggle).
    demo_toggle: bool,
    /// Tracks whether demo playback is currently inside Kid_Transform so the
    /// post-transform demo counter parity can be restored on the exit frame.
    demo_transform_active: bool,
    /// Trace-only velocity export hold after demo Kid_Transform exits.
    demo_post_transform_trace_velocity_hold_frames: u8,
    /// Trace-only stale GFX-slot y velocity after a wall-cling boundary settle.
    wall_cling_boundary_trace_y_velocity: Option<(i32, u8)>,
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
    /// Debug label for the gameplay branch that last controlled Kid this frame.
    last_kid_frame_motion_branch: KidFrameMotionBranch,
    /// Debug label for the first post-motion system that moved or consumed Kid.
    last_kid_frame_post_motion_mutator: KidFramePostMotionMutator,
    /// Debug label for the prior frame's first post-motion system that touched Kid.
    previous_kid_frame_post_motion_mutator: KidFramePostMotionMutator,
    /// Native mirror of `Platforms_CheckCollision` scratch cells
    /// `$FFFFFAA2..$FFFFFAA5` plus `$FFFFFA96` from ASM `loc_257E`.
    last_platform_collision_scratch_loc_257e: PlatformCollisionScratchLoc257E,
    /// Native mirror of Kid object `collision_type(a4)` for frame-local handoffs.
    last_kid_collision_type: u16,
    /// Debug snapshot of Kid before any pre-physics mutator runs this frame.
    last_kid_frame_pre_physics_start: Option<PlayerState>,
    /// Debug snapshot of Kid immediately before `step_with_order(...)`.
    last_kid_frame_pre_physics_before_physics: Option<PlayerState>,
    /// Ordered list of pre-physics handlers that changed Kid state this frame.
    last_kid_frame_pre_physics_mutators: Vec<KidFramePrePhysicsMutator>,
    /// Debug snapshot of native carry-style state before pre-physics consumption,
    /// including the closest native equivalent to ROM's pre-sub_B168
    /// Character_CheckCollision / Jump_On_Enemy handoff.
    last_kid_frame_collision_handoff_probe: Option<KidCollisionHandoffProbe>,
    /// Debug snapshot of stomp eligibility immediately after physics and before
    /// any post-motion consumer can move Kid again.
    last_kid_frame_same_frame_stomp_probe: Option<KidSameFrameStompProbe>,
    /// Debug snapshot of the Iron Knight wall-attach gate at the pre-physics
    /// consumer boundary.
    last_kid_frame_iron_knight_attach_probe: Option<KidIronKnightAttachProbe>,
    /// Debug snapshot of the shared kid_flip branch-selection fan-out after the
    /// post-sub_AF10 wall scan.
    last_kid_frame_kid_flip_branch_probe: Option<KidFlipBranchProbe>,
    /// Debug snapshot of the shared post-kid_flip follow-on stages inside the
    /// native loc_AA22 mirror.
    last_kid_frame_kid_flip_follow_on_probe: Option<KidFlipFollowOnProbe>,
    /// Debug snapshot of the gate inputs and branch taken by the native
    /// `apply_jump_and_gravity_vel_only(...)` call.
    last_kid_frame_jump_gravity_probe: Option<KidJumpGravityProbe>,
    /// Debug snapshot of the AsmFaithful grounded-release scan before normal physics.
    last_kid_frame_grounded_release_probe: Option<KidGroundedReleaseProbe>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PauseRestartRoundState {
    remaining_lives: u16,
    elapsed_frames: u8,
    level_state_cleared: bool,
}

const PAUSE_RESTART_APPLY_LIFE_FRAME_LOC_6B46: u8 = 1;
const PAUSE_RESTART_ZERO_KID_FRAME_LOC_6E2: u8 = 29;
const PAUSE_RESTART_CLEAR_LEVEL_STATE_FRAME_LOC_6E2: u8 = 33;
const PAUSE_RESTART_REBUILD_LEVEL_STATE_FRAME_LOC_6E2: u8 = 71;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlaneClearPassSetupLoc1698 {
    pub(crate) target_addr: u32,
    pub(crate) dbf_seed: u16,
    pub(crate) next_source_longword: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct PlaneClearWordFillLoc16B6 {
    pub(crate) target_addr: u32,
    pub(crate) next_addr: u32,
    pub(crate) words: Vec<u16>,
}

#[allow(non_snake_case)]
#[cfg(test)]
pub(crate) fn plane_clear_word_fill_loc_16B6(
    target_addr: u32,
    source_word: u16,
    dbf_seed: u16,
) -> PlaneClearWordFillLoc16B6 {
    // ASM loc_16B6 (kid.asm:2811): `move.w (a6),(a0)+` followed by `dbf d0`
    // copies the same source word `d0+1` times and advances `a0` by two bytes
    // per iteration.
    let repeat_count = usize::from(dbf_seed).wrapping_add(1);
    PlaneClearWordFillLoc16B6 {
        target_addr,
        next_addr: target_addr.wrapping_add((repeat_count as u32) * 2),
        words: vec![source_word; repeat_count],
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
pub(crate) fn plane_clear_word_fill_loc_16C8(
    target_addr: u32,
    source_word: u16,
    dbf_seed: u16,
) -> PlaneClearWordFillLoc16B6 {
    // ASM loc_16C8 (kid.asm:2817): the second clear pass uses the same
    // `move.w (a6),(a0)+` / `dbf d0` loop body as loc_16B6, so it emits
    // `d0+1` copies of the current source word and advances `a0` by two bytes
    // per iteration.
    plane_clear_word_fill_loc_16B6(target_addr, source_word, dbf_seed)
}

#[allow(non_snake_case)]
#[cfg(test)]
pub(crate) fn plane_clear_word_fill_loc_16E0(
    target_addr: u32,
    source_word: u16,
    dbf_seed: u16,
) -> PlaneClearWordFillLoc16B6 {
    // ASM loc_16E0 (kid.asm:2824): the final clear pass reuses the same
    // word-fill loop body, then falls through to `j_sub_924` after the
    // inclusive `dbf` count reaches `-1`.
    plane_clear_word_fill_loc_16B6(target_addr, source_word, dbf_seed)
}

#[derive(Debug, Clone, Copy)]
struct PendingStompBounce {
    anchor_x_pos: u32,
    anchor_y_pos: u32,
    y_velocity: i32,
    causes_contact_damage: bool,
}

#[derive(Debug, Clone, Copy)]
struct PendingPlayerContactDamage {
    enemy_x: u32,
    enemy_family: EnemyFamily,
}

#[derive(Debug, Clone, Copy)]
struct PendingEnemyCollision {
    segment_index: usize,
    layout_index: usize,
    collision_type: u16,
    target: PendingEnemyCollisionTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StompContactTarget {
    ActiveEnemy(usize),
    DripsDropProjectile(usize),
    DripsSurfaceCrawlerProjectile(usize),
    DripsSurfaceCrawlerBurstProjectile(usize),
}

#[derive(Debug, Clone)]
struct StompContactCandidate {
    target: StompContactTarget,
    snapshot: NativeObjectSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingEnemyCollisionTarget {
    Parent,
    ChildAttachment,
}

#[derive(Debug, Clone, Copy)]
struct IntroSequence6StartSkipNaturalVdpCarry {
    elapsed_at_mode_switch: usize,
    carry_timeout_restart_hidden_camera_wrap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntroSequence6Loc1b45aVdpPhase {
    BlackoutBeforeTitleRows,
    BlackoutAfterTitleRows,
    RestoredBeforeTitleArt,
}

impl NativeRuntime {
    pub(crate) fn ensure_heady_metal_breakup_slots_loc_39247(&mut self) {
        if self.heady_metal_breakup_slots_sub_37708.is_some()
            && self.heady_metal_cached_workers_sub_37708.is_some()
        {
            return;
        }
        let Some(heady_metal) = self
            .active_enemies
            .iter()
            .find(|enemy| matches!(enemy.family, EnemyFamily::HeadyMetal))
        else {
            return;
        };

        // ASM Enemy20_HeadyMetal_Init loc_39247 seeds `$FFFFFB7A..$FFFFFBAA`
        // once; loc_3763A/sub_37ABC refreshes the live table instead of
        // re-projecting body pieces from the tracking anchor.
        let anchor_x = (heady_metal.x_pos >> 16) as u16;
        let anchor_y = (heady_metal.y_pos >> 16) as u16;
        let init = heady_metal_init_enemy20_loc_39247(anchor_x, anchor_y);
        debug_assert_eq!(init.total_worker_slot_count, 17);
        debug_assert_eq!(usize::from(init.body_piece_sprite_count), 12);
        debug_assert_eq!(init.tracking_loop_entry_label, "loc_37582");
        if self.heady_metal_breakup_slots_sub_37708.is_none() {
            self.heady_metal_breakup_slots_sub_37708 =
                Some(heady_metal_breakup_slots_sub_37708_from_anchor_loc_39247(
                    init.anchor_x_pos,
                    init.anchor_y_pos,
                ));
        }
        if self.heady_metal_cached_workers_sub_37708.is_none() {
            self.heady_metal_cached_workers_sub_37708 =
                Some(heady_metal_cached_workers_sub_37708_from_anchor_loc_39247(
                    init.anchor_x_pos,
                    init.anchor_y_pos,
                ));
        }
    }

    /// ASM Mode_Level's GfxObjects_MoveAndAnimate + GfxObjects_Collision pass
    /// for the Driller drill children: integrate their `$26` x velocity and
    /// latch `$38` wall events against the post-smash grid, ready for the
    /// next body pass's Driller.asm loc_3BE78 consume. Runs even while the
    /// body hibernates in the retract/death animation — those hibernate
    /// loops only skip sub_3BF72's re-pin, not the GfxObjects passes.
    pub(crate) fn move_driller_drill_children_gfx_objects_pass(&mut self) {
        let Some(grid) = self.block_grid.as_ref() else {
            return;
        };
        for enemy in &mut self.active_enemies {
            if !matches!(enemy.family, EnemyFamily::Driller) {
                continue;
            }
            let Some(children) = enemy.driller_children.as_mut() else {
                continue;
            };
            driller_child_move_and_collide_gfx_objects_collision(
                &mut children.left,
                DrillerDrillChildSide::Left,
                grid,
            );
            driller_child_move_and_collide_gfx_objects_collision(
                &mut children.right,
                DrillerDrillChildSide::Right,
                grid,
            );
        }
    }

    pub(crate) fn move_heady_metal_body_piece_workers_sub_376ec(&mut self) {
        let Some(heady_metal) = self
            .active_enemies
            .iter()
            .find(|enemy| matches!(enemy.family, EnemyFamily::HeadyMetal))
        else {
            return;
        };
        let Some(piece_slots) = self.heady_metal_breakup_slots_sub_37708.as_mut() else {
            return;
        };

        // ASM sub_376EC (kid.asm:39483) copies the current d6/d7 tracker
        // velocities into every live `$FFFFFB72..$FFFFFBB2` worker, then the
        // scheduler advances each GfxObject independently in FP16.16. Its
        // fractional phase is not necessarily the root enemy's phase.

        let mut velocity_slots: [Option<HeadyMetalSpriteVelocitySub376EC>; 17] = [None; 17];
        if let Some(cached_workers) = self.heady_metal_cached_workers_sub_37708.as_ref() {
            for &(slot_index, worker_index) in &[(0, 0), (1, 1), (2, 2), (15, 3), (16, 4)] {
                let worker = cached_workers[worker_index];
                velocity_slots[slot_index] = Some(HeadyMetalSpriteVelocitySub376EC {
                    x_velocity: worker.x_velocity,
                    y_velocity: worker.y_velocity,
                });
            }
        }
        for (piece_index, piece) in piece_slots.iter().enumerate() {
            if let Some(piece) = piece {
                velocity_slots[piece_index + 2] = Some(HeadyMetalSpriteVelocitySub376EC {
                    x_velocity: piece.x_velocity,
                    y_velocity: piece.y_velocity,
                });
            }
        }

        // ASM sub_376EC loc_376F2 walks the 17 `$FFFFFB72..$FFFFFBB2` worker
        // pointers, skipping null longwords. Native stores `$FFFFFB7A` both as
        // cached worker 2 and body-piece slot 0, so copy index 2 back to both.
        let velocity_slots = heady_metal_apply_piece_velocities_sub_376EC(
            velocity_slots,
            heady_metal.x_velocity,
            heady_metal.y_velocity,
        );

        for (piece_index, slot) in piece_slots.iter_mut().enumerate() {
            let Some(slot) = slot else {
                continue;
            };
            if let Some(velocity) = velocity_slots[piece_index + 2] {
                slot.x_velocity = velocity.x_velocity;
                slot.y_velocity = velocity.y_velocity;
            }
            slot.x_pos_fp = slot.x_pos_fp.wrapping_add_signed(slot.x_velocity);
            slot.y_pos_fp = slot.y_pos_fp.wrapping_add_signed(slot.y_velocity);
        }
        if let Some(cached_workers) = self.heady_metal_cached_workers_sub_37708.as_mut() {
            for &(slot_index, worker_index) in &[(0, 0), (1, 1), (2, 2), (15, 3), (16, 4)] {
                let worker = &mut cached_workers[worker_index];
                if let Some(velocity) = velocity_slots[slot_index] {
                    worker.x_velocity = velocity.x_velocity;
                    worker.y_velocity = velocity.y_velocity;
                }
                worker.x_pos_fp = worker.x_pos_fp.wrapping_add_signed(worker.x_velocity);
                worker.y_pos_fp = worker.y_pos_fp.wrapping_add_signed(worker.y_velocity);
            }
        }
    }

    pub(crate) fn rewind_heady_metal_recording_handoff_gfx_passes_loc_142e(&mut self, passes: u16) {
        // ASM GfxObjects_MoveAndAnimate loc_142E adds each linked GfxObject's
        // FP16.16 velocity once per pass. The recording oracle adapter may
        // collapse a different number of presentation passes than Object
        // coroutine passes, so reverse only the measured excess here.
        if let Some(piece_slots) = self.heady_metal_breakup_slots_sub_37708.as_mut() {
            for piece in piece_slots.iter_mut().flatten() {
                piece.x_pos_fp = piece.x_pos_fp.wrapping_add_signed(
                    piece
                        .x_velocity
                        .wrapping_mul(i32::from(passes))
                        .wrapping_neg(),
                );
                piece.y_pos_fp = piece.y_pos_fp.wrapping_add_signed(
                    piece
                        .y_velocity
                        .wrapping_mul(i32::from(passes))
                        .wrapping_neg(),
                );
            }
        }
        if let Some(cached_workers) = self.heady_metal_cached_workers_sub_37708.as_mut() {
            for worker in cached_workers {
                worker.x_pos_fp = worker.x_pos_fp.wrapping_add_signed(
                    worker
                        .x_velocity
                        .wrapping_mul(i32::from(passes))
                        .wrapping_neg(),
                );
                worker.y_pos_fp = worker.y_pos_fp.wrapping_add_signed(
                    worker
                        .y_velocity
                        .wrapping_mul(i32::from(passes))
                        .wrapping_neg(),
                );
            }
        }
    }

    pub(crate) fn queue_start_sequence(&mut self, effect: QueuedSoundEffect) {
        self.queue_sound_command(QueuedSoundCommand::StartSequence(effect));
    }

    pub(crate) fn queue_stop_sequence(&mut self, effect: QueuedSoundEffect) {
        self.queue_sound_command(QueuedSoundCommand::StopSequence(effect));
    }

    pub(crate) fn queue_sound_command(&mut self, command: QueuedSoundCommand) {
        self.played_sound_effects.push(command.effect());
        self.played_sound_commands.push(command);
    }

    pub fn new(assets: AssetManifest) -> Self {
        let assets_source: Arc<dyn AssetSource> =
            Arc::new(FsAssetSource::new(assets.output_root.clone()));
        Self::new_with_source(assets, assets_source)
    }

    /// Construct a runtime reading generated-bundle bytes through an injected
    /// [`AssetSource`]. The four VDP loaders run here (so the source must exist
    /// at construction); [`new`](Self::new) default-wires a native
    /// [`FsAssetSource`] for the ~1500 legacy call sites.
    pub fn new_with_source(assets: AssetManifest, assets_source: Arc<dyn AssetSource>) -> Self {
        let sega_screen_data = vdp::load_sega_screen_data(assets_source.as_ref()).ok();
        let intro_sub_1b850_data = vdp::load_intro_sub_1b850_data(assets_source.as_ref()).ok();
        let title_screen_data = vdp::load_title_screen_data(assets_source.as_ref()).ok();
        let title_card_data = vdp::load_title_card_data(assets_source.as_ref()).ok();
        let mut runtime = Self {
            assets,
            assets_source,
            frame_index: 0,
            time_frames_loc_3038a: 0,
            mode_level_tail_state_loc_892: ModeLevelTailStateLoc892::power_on_zero(),
            mode_level_tail_work_loc_892: None,
            boot_state: BootState::SegaScreen { elapsed: 0 },
            trace_input: FrameInput::default(),
            prior_input: FrameInput::default(),
            previous_input: FrameInput::default(),
            respawn_start_latched_frames: 0,
            respawn_live_handoff_frames: 0,
            title_card_live_handoff_frames: 0,
            recorded_title_card_handoff_delay: false,
            direct_title_card_handoff_capture: false,
            cold_intro_sequence_1_early_start_poll: false,
            debug_level_entry_suppresses_title_card_live_handoff: false,
            debug_level_entry_runs_live_shell_loc_74e0: false,
            recording_oracle_debug_lifecycle_completed: false,
            recording_oracle_settle_completed: false,
            start_press_count: 0,
            current_level: LevelId(0),
            level_start_frame: None,
            progression: PlayerProgression::default(),
            helmet: HelmetState::default(),
            paused: false,
            pause_menu_option: 0,
            pause_menu_arrow_cursor_cmds_loc_6a84: None,
            pause_toggle_pending: false,
            pending_pause_restart_round: None,
            pending_pause_last_life_give_up: false,
            allow_pausing: false,
            allow_pausing_fade_frames: 0,
            pending_level_skip: None,
            pending_level_skip_resume: false,
            teleport_warp_colour_fbcc: 0,
            level_entry_fade_progress_sub_73d0: 0,
            level_entry_pending_palette_0_mask_sub_73d0: 0,
            level_entry_pending_palette_1_mask_sub_73d0: 0,
            palette_dma_suppressed_f890: false,
            mode_level_palette_dma_sub_970: None,
            pending_player_start_override_loc_11bd2: None,
            player_lifecycle: PlayerLifecycle::Alive,
            opening_slice: None,
            retained_kid_hitbox_loc_224c: None,
            gameplay_vdp_camera_latch_loc_300ba: None,
            probe_start_slope_settle_frames: 0,
            starting_level_title_workers: Vec::new(),
            starting_level_title_rng: None,
            death_camera_lock: None,
            death_windup_path: DeathWindupPath::default(),
            death_preserved_object_velocity: None,
            death_respawn_trace_profile: None,
            level_skip_transition: None,
            death_shell_snapshot: None,
            level_complete_gfx_snapshot: None,
            level_entry_camera_handoff: None,
            level_entry_player_handoff: None,
            level_skip_resume_hold: false,
            active_enemy_roster: Vec::new(),
            active_enemies: Vec::new(),
            killed_enemies: HashSet::new(),
            enemy_status_words: [0; 0x0040],
            enemy_loading_initial_pass_complete: false,
            pending_enemy_gfx_initialisations_loc_3691e: 0,
            freshly_loaded_enemies_loc_3691e: std::collections::HashMap::new(),
            pending_tornado_initialisations_loc_3691e: HashSet::new(),
            drilled_blocks: Vec::new(),
            active_platforms: Vec::new(),
            active_platform_roster: Vec::new(),
            active_platform_standing_on: None,
            active_lift_platform_standing_on: None,
            platform_script_table: None,
            block_grid: None,
            block_list: None,
            block_grid_source: BlockGridSource::None,
            game_complete: false,
            block_state: BlockInteractionState::default(),
            projectiles: Vec::new(),
            heady_metal_breakup_slots_sub_37708: None,
            heady_metal_cached_workers_sub_37708: None,
            heady_metal_cached_worker_velocities_sub_3787c: HeadyMetalScatterVelocities::default(),
            multi_head_boss_death_count_fb4e: 0,
            boss_eye_breakup_trigger_words_fb72_fb74_fb76: [0; 3],
            diamond_power_rain_slots: Vec::new(),
            diamond_power_circle_slots: Vec::new(),
            diamond_power_wall_slots: Vec::new(),
            diamond_power_five_way_slots: Vec::new(),
            diamond_power_snake_slots: Vec::new(),
            diamond_power_extra_hp_life_slots: Vec::new(),
            diamond_power_invuln_haze_slots: Vec::new(),
            diamond_power_object_lists: helmets::DiamondPowerObjectListsSub3F5E8::default(),
            diamond_power_object_ram: helmets::DiamondPowerObjectRamSub3F596::default(),
            diamond_power_invuln_haze_timer: 0,
            five_way_shot_ready: false,
            juggernaut_companion_state_sub_975c: helmets::JuggernautCompanionStateSub975C {
                byte_15_active: false,
                byte_18_active: false,
                blocked_latch: false,
            },
            last_juggernaut_companion_step_sub_975c: None,
            kid_is_invulnerable: false,
            diamond_power_snake_head_angle: 0,
            diamond_power_snake_turn_rate: 0,
            diamond_power_snake_movement_speed: 0,
            diamond_power_sprites: Vec::new(),
            diamond_power_sprite_buffer_sub_3f798: Vec::new(),
            diamond_power_buffered_sprite_count_before_compile_sub_3f798: 0,
            diamond_power_number_sprites_sub_3f798: 0,
            eyeclops_hard_beam_object: None,
            fire_demon_trails: Vec::new(),
            weather: WeatherState::default(),
            palette_buffer: [0; 0x40],
            palette_buffer_2: [0; 0x40],
            palette_buffer_3: [0; 0x40],
            lava_geyser_palette_delay_loc_14b2a: 5,
            rotate_palette_lava_water_loc_1223c: None,
            island_shore_cycle_loc_1202a: None,
            transform_resident_island_shore_vram_frames_loc_1202a: None,
            island_shore_dma_uploads_loc_1202a: Vec::new(),
            lava_geyser_art_uploads_loc_14c6a: Vec::new(),
            murder_wall: None,
            murder_wall_warning_animation_timer_fac0: 0,
            murder_wall_warning_animation_frame_fabf: 0,
            block_bounces: Vec::new(),
            prize_block_worker_tails: Vec::new(),
            prize_block_spawn_flashes: Vec::new(),
            kid_projectile_hit_sparks: Vec::new(),
            hidden_block_fg_update_workers: Vec::new(),
            pending_fg_update_cell_queue_sub_1596: Vec::new(),
            block_destroy_effects: Vec::new(),
            block_debris_swarms: Vec::new(),
            block_debris_sprite_pieces_sub_2a4c: Vec::new(),
            block_debris_scratch_cursor_sub_292e: 0,
            floating_items: Vec::new(),
            prize_worker_effects: Vec::new(),
            latched_prize_wake_armed_loc_104ae: false,
            clock_prize_animation_swaps_loc_10796: Vec::new(),
            continue_prize_animation_swaps_loc_1080a: Vec::new(),
            prize_helmet_collision_hibernate_delay_frames: 0,
            prize_helmet_collision_hibernate_hitpoints_form: None,
            prize_helmet_collision_hibernate_switch_form: None,
            iron_knight_block_breaker: false,
            iron_knight_last_metal_block_hit: None,
            pending_prize_diamond_awards: Vec::new(),
            pending_prize_diamond_hud_flights: Vec::new(),
            pending_prize_clock_hud_flights: Vec::new(),
            pending_sound_effects: Vec::new(),
            played_sound_effects: Vec::new(),
            played_sound_commands: Vec::new(),
            played_bgm_ids: Vec::new(),
            played_music_speed_updates: Vec::new(),
            enemy_death_effects: Vec::new(),
            pending_dragon_death_children: Vec::new(),
            pending_dragon_death_animations: Vec::new(),
            stomped_this_frame: false,
            kid_touched_enemy_last_frame_loc_fa74: false,
            kid_touched_enemy_this_frame_loc_fa75: false,
            pending_stomp_bounce: None,
            pending_player_contact_damage: None,
            hidden_shell_contact_latched_frame_loc_1cf2: None,
            delayed_player_contact_damage: None,
            shooter_barrel_damage_latched_loc_1f52: false,
            pending_grounded_dragon_landing_graze: None,
            pending_enemy_collisions: Vec::new(),
            rom_random: RomRandomState::default(),
            blocks_hit_this_frame: 0,
            sound_test_play_pending: false,
            sega_screen_data,
            intro_sub_1b850_data,
            title_screen_data,
            title_card_data,
            vdp_state: None,
            z80_stop_request_depth: 0,
            z80_guard_sr: 0x2500,
            z80_running: true,
            options_two_player_two_controllers: false,
            options_controls_scheme: 0,
            options_speed_button_fast: false,
            options_menu_panel_toggle_byte: 0,
            options_menu_left_panel_x: 0x0030,
            options_menu_right_panel_x: 0x0110,
            options_menu_panel_delta: 0x0118,
            level_timer: LevelTimer::new(),
            hud_timer_sprite_seconds_loc_bee8: LEVEL_TIMER_START_SECONDS,
            hud_diamond_count_loc_bdc0: 0,
            demo_hud_timer_presentation_loc_be82: None,
            pending_demo_timer_tick_result_loc_be82: None,
            hud_animation_state: boot::HudAnimationStateSub44DC::initialised_from_sub_44_b0(),
            hud_dma_events_sub_44dc: Vec::new(),
            hud_raster_upload_timings_sub_44dc: Vec::new(),
            mode_level_raster_seed_sub_e12: None,
            mode_level_vblank_work_before_sub_e12: None,
            mode_level_cpu_raster_cursor_sub_44dc: None,
            mode_level_hud_raw_cycles_sub_44dc: None,
            mode_level_work_before_hud_sub_44dc: None,
            mode_level_work_after_hud_sub_44dc: None,
            mode_level_platform_loader_work_loc_2744: None,
            demo_read_joypad_work_loc_c22: None,
            gameplay_hud_vram_state: vdp::GameplayHudVramState::initialised_from_sub_44b0(),
            transform_resident_hud_diamond_vram_page_loc_5d4a: None,
            // Oracle Demo 1 bootstrap frames expose ArtUnc_4692 page 2 before
            // the first loc_4586 life-icon upload occurs.
            hud_life_icon_vram_page_loc_4586: 8,
            y_look_offset: 0,
            camera_last_kid_y: 0,
            level_damage_taken: false,
            just_received_damage: false,
            recent_damage_gate_held_this_frame_loc_bc6e: false,
            last_projectile_damage_probe: None,
            last_drips_splash_reseed_probe: None,
            lion_projectile_object_only_tail_count: 0,
            drips_surface_crawler_count_loc_fffffb42: 0,
            level_prize_collected: false,
            level_speed_bonus_awarded: false,
            level_path_bonus_amount: 0,
            level_complete_initial_score: 0,
            clocks_collected: 0,
            hundred_k_trip_timer: 0,
            title_idle_frames: 0,
            title_screen_elapsed_frames: 0,
            title_screen_uses_intro_return_hidden_shell: false,
            title_screen_intro_return_lineage: None,
            intro_sequence_6_start_skip_cursor_trace_priority: false,
            intro_sequence_6_start_skip_intro_sequence_5_pose_carry: false,
            intro_sequence_6_start_skip_natural_vdp_carry: None,
            intro_sequence_5_title_return_vdp_snapshot: None,
            intro_sequence_1_title_return_vdp_snapshot: None,
            intro_sequence_4_title_return_vdp_snapshot: None,
            // ASM loc_61C seeds `$FFFFFC82` to 2 before the cold-boot intro;
            // Load_IntroSequence5's first sub_1C278 call then wraps it to 0.
            title_screen_region_byte: 2,
            title_screen_ready_frame: TITLE_SCREEN_READY_GLOBAL_FRAME,
            title_menu_arm_frame: 0,
            title_menu_active: false,
            title_menu_cursor_pending: false,
            title_menu_vdp_upload_frame_loc_1bb52: None,
            title_menu_hidden_prearm_pending: false,
            title_menu_selection_pending: None,
            title_menu_selection_delay_frames: 0,
            title_menu_level_entry_vdp_carry: None,
            demo_entry_title_vdp_snapshot: None,
            demo_buffered_input: None,
            pending_demo_title_return: 0,
            demo_title_return_after_frame: false,
            demo_auto_timeout_return_pending: false,
            demo_auto_timeout_freeze_gameplay_frame: false,
            demo_auto_timeout_handoff_active: false,
            demo_auto_timeout_handoff_preserves_jump_movement: false,
            demo_auto_timeout_mode_0_entry: DemoAutoTimeoutMode0Entry::Fresh,
            demo_auto_timeout_intro_sequence_1_frames_override: None,
            demo_auto_timeout_intro_sequence_2_frames_override: None,
            demo_auto_timeout_intro_sequence_3_frames_override: None,
            demo_auto_timeout_intro_sequence_4_frames_override: None,
            demo_auto_timeout_intro_sequence_5_frames_override: None,
            timeout_restart_title_intro_sequence_3_return_pending: false,
            timeout_restart_title_intro_sequence_4_return_pending: false,
            timeout_restart_title_intro_sequence_5_return_pending: false,
            timeout_restart_title_intro_sequence_5_particle_roster: false,
            preserve_demo_title_snapshot: false,
            demo_title_visual_level: None,
            demo_title_render_snapshot: None,
            demo_title_snapshot_object_counts: None,
            timeout_restart_title_intro_sequence_3_pose: false,
            timeout_restart_title_intro_sequence_4_camera_carry: false,
            timeout_restart_intro_sequence_3_title_return_vdp_snapshot: None,
            intro_sequence_2_title_return_vdp_snapshot: None,
            intro_sequence_3_handoff_vdp_snapshot: None,
            intro_sequence_4_handoff_vdp_snapshot: None,
            intro_sequence_5_handoff_vdp_snapshot: None,
            timeout_restart_title_intro_sequence_5_pose: false,
            timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay: false,
            timeout_restart_intro_sequence_4_pose_carry: false,
            hide_demo_return_title_player: false,
            ignore_demo_start_abort_until_gameplay: false,
            demo_started_from_timeout_restart_shell: false,
            intro_sequence_6_entered_from_timeout_restart: false,
            next_demo_uses_start_skip_intro_return_camera_wrap: false,
            demo_uses_start_skip_intro_return_camera_wrap: false,
            next_demo_uses_demo_title_return_camera_wrap: false,
            demo_uses_demo_title_return_camera_wrap: false,
            demo_uses_intro_sequence_6_hidden_camera_wrap: false,
            demo_uses_intro_sequence_6_early_title_restore: false,
            timeout_restart_intro_sequence_1_uses_early_title_restore: false,
            demo_uses_intro_sequence_6_later_demo_2_preload_row: false,
            demo_uses_cold_intro_sequence_6_demo_2_preload_rows: false,
            demo_uses_intro_sequence_6_timeout_return_velocity_sample: false,
            demo_uses_intro_sequence_5_title_return_preload_row: false,
            demo_title_return_uses_intro_sequence_6_early_title_restore: false,
            demo_timeout_restart_uses_intro_return_slot_2_early_visibility: false,
            demo_timeout_restart_uses_start_skip_intro_sequence_5_delay: false,
            demo_timeout_restart_uses_intro_sequence_6_early_intro_sequence_5_handoff: false,
            next_intro_sequence_3_roster_index: 0,
            active_intro_sequence_3_roster_index: 0,
            demo_toggle: false,
            demo_transform_active: false,
            demo_post_transform_trace_velocity_hold_frames: 0,
            wall_cling_boundary_trace_y_velocity: None,
            two_player: false,
            current_player_index: 0,
            player_slots: [PlayerSlot::default(), PlayerSlot::default()],
            physics_order: PhysicsOrder::default(),
            trace_physics: false,
            last_kid_frame_motion_branch: KidFrameMotionBranch::default(),
            last_kid_frame_post_motion_mutator: KidFramePostMotionMutator::default(),
            previous_kid_frame_post_motion_mutator: KidFramePostMotionMutator::default(),
            last_platform_collision_scratch_loc_257e: PlatformCollisionScratchLoc257E::default(),
            last_kid_collision_type: 0,
            last_kid_frame_pre_physics_start: None,
            last_kid_frame_pre_physics_before_physics: None,
            last_kid_frame_pre_physics_mutators: Vec::new(),
            last_kid_frame_collision_handoff_probe: None,
            last_kid_frame_same_frame_stomp_probe: None,
            last_kid_frame_iron_knight_attach_probe: None,
            last_kid_frame_kid_flip_branch_probe: None,
            last_kid_frame_kid_flip_follow_on_probe: None,
            last_kid_frame_jump_gravity_probe: None,
            last_kid_frame_grounded_release_probe: None,
        };
        runtime.reload_block_grid_for_level(LevelId(0));
        runtime.load_platform_script_table();
        runtime.update_vdp_state();
        runtime
    }

    pub fn set_player_probe_position(&mut self, x_px: u16, y_px: u16) {
        // ASM loc_C4E maintains held/pressed joypad latches outside the live
        // GfxObject. Direct probe starts are WRAM seeds, so clear native's
        // cached latches with the position seed instead of carrying whatever
        // the settle route last held into frame 0.
        self.prior_input = FrameInput::default();
        self.previous_input = FrameInput::default();
        // ASM loc_9F18 / direct WRAM probe bootstrap: oracle frame 0 exposes
        // the patched slope snap before neutral loc_A13A slope continuation
        // advances. Keep that as a probe-only synthetic first-frame guard.
        self.probe_start_slope_settle_frames = 2;
        if self.opening_slice.is_none() {
            self.opening_slice = Some(if self.block_grid().is_some() {
                self.level_respawn_slice_with_floor_snap(self.current_level)
            } else {
                self.level_respawn_slice(self.current_level)
            });
        }
        if let Some(slice) = self.opening_slice.as_mut() {
            slice.player.x_pos = u32::from(x_px) << 16;
            slice.player.y_pos = u32::from(y_px) << 16;
            slice.player.x_velocity = 0;
            slice.player.y_velocity = 0;
            slice.player.facing = Direction::Right;
            slice.player.on_ground = true;
            slice.player.just_jumped = false;
            slice.player.jump_buffer_frames = 0;
            slice.player.right_wall_sticky = false;
            slice.player.left_wall_sticky = false;
            slice.player.wall_cling_direction = None;
            slice.terrain.floor_y_pos = u32::from(y_px) << 16;
            slice.terrain.floor_terrain_type = TerrainType::Normal;
            slice.terrain.slope_walk_kind = None;
            slice.terrain.slope_sliding = false;
        }
    }

    pub fn set_player_probe_velocity(&mut self, x_velocity: i32, y_velocity: i32) {
        if let Some(slice) = self.opening_slice.as_mut() {
            slice.player.x_velocity = x_velocity;
            slice.player.y_velocity = y_velocity;
            slice.player.on_ground = false;
        }
    }

    pub fn set_camera_probe_position(&mut self, x_px: u16, y_px: u16) {
        if let Some(slice) = self.opening_slice.as_mut() {
            slice.camera.x_pos = u32::from(x_px) << 16;
            slice.camera.y_pos = u32::from(y_px) << 16;
        }
        self.rebase_mode_level_tail_after_external_override_loc_2ffe8();
        self.prime_probe_camera_enemy_loading_loc_31f8e_368a8();
    }

    pub(crate) fn reset_and_preload_mode_level_tail_loc_720_2ffe8(&mut self) {
        self.mode_level_tail_state_loc_892.clear_loc_720();
        self.rebase_mode_level_tail_after_external_override_loc_2ffe8();
        // ASM sub_73D0/loc_7414: every Load_InGame path seeds both full
        // palette masks before the first live Mode_Level loc_892 tail.
        self.mode_level_tail_state_loc_892
            .set_pending_palette_masks_f888_f88c(u32::MAX, u32::MAX);
    }

    pub(crate) fn rebase_mode_level_tail_after_external_override_loc_2ffe8(&mut self) {
        // Load_InGame preloads from the physical Camera_X/Y words. Trace
        // presentation may temporarily substitute cleared respawn/title-card
        // camera state, so prefer the resident gameplay slice when present.
        let camera = self
            .opening_slice
            .map_or_else(|| self.current_camera_state(), |slice| slice.camera);
        let level_height_pixels = self
            .level_header(self.current_level)
            .map_or(u16::MAX, |header| {
                u16::from(header.height_screens).saturating_mul(14 * 16)
            });
        self.mode_level_tail_state_loc_892.preload_loc_2ffe8(
            (camera.x_pos >> 16) as u16,
            (camera.y_pos >> 16) as u16,
            level_height_pixels,
            self.time_frames_loc_3038a,
        );
    }

    pub(crate) fn advance_mode_level_tail_loc_892(&mut self) {
        let Some((width_screens, background_theme)) = self
            .level_header(self.current_level)
            .map(|header| (header.width_screens, u16::from(header.background_theme)))
        else {
            self.mode_level_tail_work_loc_892 = None;
            return;
        };
        let Some(bgscroll) = self
            .resource_level_record(self.current_level)
            .and_then(|record| record.bgscroll_metadata.clone())
        else {
            self.mode_level_tail_work_loc_892 = None;
            return;
        };
        let camera = self
            .opening_slice
            .map_or_else(|| self.current_camera_state(), |slice| slice.camera);
        let camera_x_pos = (camera.x_pos >> 16) as u16;
        let camera_y_pos = (camera.y_pos >> 16) as u16;
        let (previous_camera_x_pos, previous_camera_y_pos) = self
            .mode_level_tail_state_loc_892
            .camera_latches_f82c_f82e();
        let previous_x_tile = previous_camera_x_pos >> 3;
        let previous_y_tile = (previous_camera_y_pos as i16) >> 3;
        let x_tile = camera_x_pos >> 3;
        let y_tile = (camera_y_pos as i16) >> 3;
        // ASM sub_30194 loc_301CE: gather the live 21-cell row for either
        // unsplit vertical path. Downward motion uses d6+$1C before the
        // shared row decoder; the pure timing core classifies its mappings.
        let camera_stripe_cells_sub_30194 = self.block_grid.as_ref().and_then(|grid| {
            let source_y_tile = if y_tile > previous_y_tile {
                y_tile.checked_add(0x001C)?
            } else {
                y_tile
            };
            let row = u16::try_from(source_y_tile >> 1).ok()?;
            let first_column = x_tile >> 1;
            (y_tile != previous_y_tile
                && first_column.checked_add(20)? < grid.width_cells
                && row < grid.height_cells)
                .then(|| {
                    (0..21_u16)
                        .map(|offset| grid.raw_word_at(first_column + offset, row))
                        .collect::<Vec<_>>()
                })
        });
        // ASM sub_30194 loc_3023C/loc_30262: horizontal motion selects the
        // newly exposed tile column. Rightward uses d7+$28; leftward uses d7.
        // The unsplit plane window uses 15 consecutive block rows through
        // sub_302F6.
        let camera_column_cells_sub_30194 = self.block_grid.as_ref().and_then(|grid| {
            let source_x_tile = if previous_x_tile < x_tile {
                x_tile.checked_add(0x0028)?
            } else {
                x_tile
            };
            let column = source_x_tile >> 1;
            let first_row = u16::try_from((y_tile & !1) >> 1).ok()?;
            (previous_x_tile != x_tile
                && column < grid.width_cells
                && first_row.checked_add(14)? < grid.height_cells)
                .then(|| {
                    (0..15_u16)
                        .map(|offset| grid.raw_word_at(column, first_row + offset))
                        .collect::<Vec<_>>()
                })
        });
        let level_width_tiles = u16::from(width_screens).wrapping_mul(40);
        let (mut background_width, _) = background_dimensions_loc_11B40(level_width_tiles, 0);
        let background_format_nonzero = matches!(background_theme, 3 | 5 | 7 | 9);
        if background_format_nonzero {
            background_width = 0x0040;
        }

        let advance =
            self.mode_level_tail_state_loc_892
                .advance_loc_892(ModeLevelTailInputsLoc892 {
                    camera_x_pos,
                    camera_y_pos,
                    camera_stripe_cells_sub_30194: camera_stripe_cells_sub_30194.as_deref(),
                    camera_column_cells_sub_30194: camera_column_cells_sub_30194.as_deref(),
                    time_frames_before_loc_3038a: self.time_frames_loc_3038a,
                    game_mode_zero: self.game_mode().as_disassembly_value() == 0,
                    special_effect_4: self.weather.weather_type == 4,
                    background_width,
                    background_format_nonzero,
                    background_no_scroll: false,
                    background_theme,
                    bgscroll_profile: &bgscroll.layer_speeds,
                    bgscroll_has_terminator: bgscroll.has_terminator,
                });
        self.mode_level_tail_state_loc_892 = advance.next_state;
        // ASM sub_14C0 clears each non-zero shared mask before returning.
        self.level_entry_pending_palette_0_mask_sub_73d0 = 0;
        self.level_entry_pending_palette_1_mask_sub_73d0 = 0;
        self.time_frames_loc_3038a = advance.next_time_frames;
        self.mode_level_tail_work_loc_892 = advance.work;
    }

    pub fn mode_level_tail_work_loc_892(&self) -> Option<ModeLevelTailWorkLoc892> {
        self.mode_level_tail_work_loc_892
    }

    pub fn mode_level_cpu_raster_cursor_sub_44dc(&self) -> Option<vdp::ModeLevelCpuRasterCursor> {
        self.mode_level_cpu_raster_cursor_sub_44dc
    }

    pub fn mode_level_hud_raw_cycles_sub_44dc(&self) -> Option<M68kCycleCount> {
        self.mode_level_hud_raw_cycles_sub_44dc
    }

    pub fn mode_level_work_after_hud_sub_44dc(&self) -> Option<&ModeLevelPostHudWorkLedger> {
        self.mode_level_work_after_hud_sub_44dc.as_ref()
    }

    /// Seed the hardware beam and refresh state at the next live ASM
    /// `Execute_Objects sub_E12` boundary.
    ///
    /// The seed is consumed once the corresponding Mode_Level pass reaches
    /// `sub_44DC`. It exists for hardware checkpoints and a future persistent
    /// scheduler; normal gameplay does not invent a value.
    pub fn seed_mode_level_raster_sub_e12(&mut self, seed: vdp::ModeLevelRasterSeedSubE12) {
        self.mode_level_raster_seed_sub_e12 = Some(seed);
    }

    /// Seed the hardware beam at ASM `Mode_Level $7C8` and advance the typed
    /// VBlank source/arbitration path to the next `Execute_Objects sub_E12`.
    pub fn seed_mode_level_raster_mode_entry(
        &mut self,
        seed: vdp::ModeLevelRasterSeedModeEntry,
        work: ModeLevelVblankWorkBeforeSubE12,
        arbitration: ModeLevelVblankArbitrationBeforeSubE12,
    ) -> bool {
        let Some(seed) =
            vdp::mode_level_raster_seed_sub_e12_from_mode_entry(seed, work, arbitration)
        else {
            return false;
        };
        self.mode_level_raster_seed_sub_e12 = Some(seed);
        true
    }

    /// Seed the VDP VINT assertion boundary and advance the sampled
    /// `WaitForVint -> V_Int -> MainGameLoop -> Mode_Level` path.
    pub fn seed_mode_level_raster_vint_assertion(
        &mut self,
        seed: vdp::ModeLevelRasterSeedVintAssertion,
        sample: WaitForVintInterruptSample,
        work: ModeLevelVblankWorkBeforeSubE12,
        arbitration: ModeLevelVblankArbitrationBeforeSubE12,
    ) -> bool {
        let Some(mode_entry) =
            vdp::mode_level_raster_seed_mode_entry_from_vint_assertion(seed, sample)
        else {
            return false;
        };
        self.seed_mode_level_raster_mode_entry(mode_entry, work, arbitration)
    }

    /// Seed the first ASM `WaitForVint $93E` poll on a path with no
    /// asynchronous Z80 access to the 68k bus.
    pub fn seed_mode_level_raster_wait_for_vint_poll_no_z80(
        &mut self,
        poll: vdp::WaitForVintPollSeedNoZ80,
        assertion: vdp::ModeLevelRasterSeedVintAssertion,
        work: ModeLevelVblankWorkBeforeSubE12,
        arbitration: ModeLevelVblankArbitrationBeforeSubE12,
    ) -> bool {
        let Some(sample) = vdp::wait_for_vint_sample_without_z80_bus_access(poll, assertion) else {
            return false;
        };
        self.seed_mode_level_raster_vint_assertion(assertion, sample, work, arbitration)
    }

    fn mode_level_vblank_work_before_sub_e12(&mut self) -> ModeLevelVblankWorkBeforeSubE12 {
        let (camera_x_pixels, camera_y_pixels) =
            self.opening_slice.as_ref().map_or((0, 0), |slice| {
                (
                    (slice.camera.x_pos >> 16) as u16,
                    (slice.camera.y_pos >> 16) as u16,
                )
            });
        let cell_paths = self
            .pending_fg_update_cell_queue_sub_1596
            .iter()
            .map(|&(col, row, cell)| {
                if crate::level::fg_update_queue_cell_vram_word_loc_1614(
                    col,
                    row,
                    camera_x_pixels,
                    camera_y_pixels,
                )
                .is_none()
                {
                    ModeLevelForegroundCellQueuePathSub1596::Culled
                } else if (cell.to_word() as i16).is_negative() {
                    ModeLevelForegroundCellQueuePathSub1596::VisibleBlock
                } else {
                    ModeLevelForegroundCellQueuePathSub1596::VisibleTheme
                }
            })
            .collect::<Vec<_>>();
        self.pending_fg_update_cell_queue_sub_1596.clear();
        let foreground_cell_queue_sub_1596 =
            ModeLevelForegroundCellQueueWorkSub1596::from_paths(&cell_paths)
                .expect("ROM foreground cell queue holds fewer than 128 records");
        ModeLevelVblankWorkBeforeSubE12 {
            // ASM Mode_Level $7C8..sub_E12: dynamic-art dimensions cross the
            // render asset boundary and are completed by the execution shell.
            dynamic_art_tiles_loc_ad4: None,
            palette_dma_sub_970: self
                .mode_level_palette_dma_sub_970
                .unwrap_or(!self.palette_dma_suppressed_f890),
            scroll_path: ModeLevelScrollTransferPath::NormalLoc300ba,
            foreground_custom_queue_entries_sub_1596: 0,
            foreground_cell_queue_sub_1596,
            paused_sub_6874: false,
            hud_path_sub_bdc0: ModeLevelHudBuildPath::LocBe20DirectInitialCounterUpdates,
        }
    }

    pub fn set_playtest_form(&mut self, form: PlayerForm) {
        self.progression.current_form = form;
        self.progression.number_hitpoints = form.max_hitpoints();
    }

    pub fn set_probe_diamond_count(&mut self, count: u16) {
        self.progression.diamond_count = count;
    }

    pub fn set_probe_lives_count(&mut self, count: u16) {
        // ASM variables.asm: `Number_Lives` is `$FFFFFC3E`; pause menu
        // `Game_Paused_ChkStart` (kid.asm:6865) compares/decrements this word.
        self.progression.number_lives = count;
    }

    fn apply_pause_menu_frame_loc_6ae0(&mut self, input: FrameInput, start_pressed: bool) -> bool {
        // ASM restart_text (kid.asm:6741): live pause menu text uses
        // Number_Lives to choose "Give Up" vs "Restart Round" before the loop
        // handles cursor movement or Start.
        let restart_text =
            boot::pause_menu_text_selection_restart_text(self.progression.number_lives);
        debug_assert_eq!(
            matches!(
                restart_text,
                boot::PauseMenuTextSelectionRestartText::RestartRound
            ),
            self.progression.number_lives > 1
        );

        // ASM Game_Paused_Loop (kid.asm:6829-6857): Up/Down only move the
        // two-entry cursor when they are fresh Ctrl_Pressed edges.
        let arrow_cursor_cmds = self.opening_slice.as_ref().map(|slice| {
            boot::pause_menu_arrow_cursor_cmds_loc_6A84(
                (slice.camera.x_pos >> 16) as u16,
                (slice.camera.y_pos >> 16) as u16,
            )
        });
        if input.up && !self.previous_input.up {
            if let Some(cursor) =
                boot::pause_loop_up_handler_game_paused_loop(true, self.pause_menu_option)
            {
                self.pause_menu_option = cursor.new_pause_option;
                self.pause_menu_arrow_cursor_cmds_loc_6a84 = arrow_cursor_cmds;
            }
        } else if input.down
            && !self.previous_input.down
            && let Some(cursor) =
                boot::pause_loop_down_handler_loc_6B0E(true, self.pause_menu_option)
        {
            self.pause_menu_option = cursor.new_pause_option;
            self.pause_menu_arrow_cursor_cmds_loc_6a84 = arrow_cursor_cmds;
        }

        match boot::pause_loop_start_handler_chk_start(
            start_pressed,
            self.pause_menu_option,
            self.progression.number_lives,
        ) {
            boot::PauseMenuStartResult::NotPressed => {}
            boot::PauseMenuStartResult::ContinueGame => {
                if let Some((camera_x_pos, camera_y_pos)) = self.pause_menu_camera_words() {
                    // ASM loc_6B68 (kid.asm:6880): the Continue path restores
                    // the saved nametable rows before clearing Game_Paused.
                    let _ = boot::pause_menu_tile_restore_plan_loc_6B68(camera_x_pos, camera_y_pos);
                }
                self.paused = false;
                self.pause_menu_option = 0;
                self.pause_menu_arrow_cursor_cmds_loc_6a84 = None;
                self.pause_toggle_pending = false;
                self.pending_pause_restart_round = None;
                self.pending_pause_last_life_give_up = false;
                return true;
            }
            boot::PauseMenuStartResult::RestartLevel { remaining_lives } => {
                // ASM Game_Paused_ChkStart (kid.asm:6865-6876): after
                // `subi.w #1,(Number_Lives).w`, the restart branch clears
                // Extra_hitpoint_slots and Current_Helmet, restores Kid HP,
                // sets the restart flag, then jumps through loc_6E2.
                self.pause_toggle_pending = false;
                self.pause_menu_arrow_cursor_cmds_loc_6a84 = None;
                self.pending_pause_last_life_give_up = false;
                self.pending_pause_restart_round = Some(PauseRestartRoundState {
                    remaining_lives,
                    elapsed_frames: 0,
                    level_state_cleared: false,
                });
            }
            boot::PauseMenuStartResult::LastLifeGameOver => {
                // ASM Game_Paused_ChkStart loc_6B62: the Start frame only
                // decrements `Number_Lives` to zero and sets Game_Paused=3.
                // The later loc_6C0A pause-exit tail runs on the next frame.
                self.progression.number_lives = 0;
                self.pause_toggle_pending = false;
                self.pause_menu_arrow_cursor_cmds_loc_6a84 = None;
                self.pending_pause_restart_round = None;
                self.pending_pause_last_life_give_up = true;
            }
        }
        false
    }

    fn pause_menu_camera_words(&self) -> Option<(u16, u16)> {
        self.opening_slice.as_ref().map(|slice| {
            (
                (slice.camera.x_pos >> 16) as u16,
                (slice.camera.y_pos >> 16) as u16,
            )
        })
    }

    fn step_pause_last_life_give_up_loc_6c0a(&mut self) -> bool {
        if !self.pending_pause_last_life_give_up {
            return false;
        }
        self.pending_pause_last_life_give_up = false;
        if let boot::PauseMenuUnpauseExitLoc6C0A::LastLifeGameOver { number_lives, .. } =
            boot::pause_menu_unpause_exit_loc_6C0A(3)
        {
            // ASM loc_6C0A (kid.asm:6964-6973): last-life Give Up plays the
            // bummer voice, sets Kid object action fields `$38/$3A`, writes
            // Number_Lives=1, and clears Game_Paused. It does not write
            // Number_Hitpoints; the subsequent death action advances later.
            self.progression.number_lives = number_lives;
            self.player_lifecycle = PlayerLifecycle::DeathWindup {
                remaining_frames: PLAYER_DEATH_WINDUP_FRAMES,
            };
            self.paused = false;
            self.pause_menu_option = 0;
            self.pause_menu_arrow_cursor_cmds_loc_6a84 = None;
            self.pause_toggle_pending = false;
            self.pending_pause_restart_round = None;
            self.allow_pausing = false;
            self.clear_level_entry_fade_sub_73d0();
            true
        } else {
            false
        }
    }

    fn apply_pause_restart_life_spend_loc_6b46(&mut self, remaining_lives: u16) {
        // ASM Game_Paused_ChkStart loc_6B46-loc_6B5E: the restart path spends
        // one life, clears Extra_hitpoint_slots/Current_Helmet, restores Kid's
        // base hitpoints, sets the restart flag, then jumps to loc_6E2.
        self.progression.number_lives = remaining_lives;
        self.progression.extra_hitpoint_slots = 0;
        self.progression.current_form = PlayerForm::TheKid;
        self.progression.pending_form = None;
        self.progression.number_hitpoints = PlayerForm::TheKid.max_hitpoints();
        self.progression.form_change_flash_frames = 0;
        self.progression.form_width_delta = 0;
        self.helmet.reset();
    }

    fn clear_pause_restart_level_state_loc_6e2(&mut self) {
        // ASM loc_6E2 enters the Load_InGame reload shell before loc_6C3E
        // finally clears Game_Paused. Oracle WRAM exposes a short blank object
        // window here rather than live gameplay continuing under the menu.
        self.opening_slice = None;
        self.pause_menu_arrow_cursor_cmds_loc_6a84 = None;
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.pending_dragon_death_children.clear();
        self.pending_dragon_death_animations.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.active_platform_standing_on = None;
        self.active_lift_platform_standing_on = None;
        self.projectiles.clear();
        self.heady_metal_breakup_slots_sub_37708 = None;
        self.floating_items.clear();
        self.prize_worker_effects.clear();
        self.enemy_death_effects.clear();
        self.pending_prize_diamond_awards.clear();
        self.pending_prize_diamond_hud_flights.clear();
        self.pending_prize_clock_hud_flights.clear();
        self.clear_block_object_workers_loc_6e2();
    }

    fn finish_pause_restart_round_loc_6c3e(&mut self) {
        self.reset_current_level_runtime_state_for_respawn();
        // ASM loc_6C3E clears Game_Paused after the loc_6E2 restart shell.
        self.paused = false;
        self.pause_menu_option = 0;
        self.pause_menu_arrow_cursor_cmds_loc_6a84 = None;
        self.pause_toggle_pending = false;
        self.pending_pause_restart_round = None;
        self.initialise_level_entry_fade_sub_73d0();
    }

    fn step_pause_restart_round_shell_loc_6b46(&mut self) {
        let Some(mut state) = self.pending_pause_restart_round else {
            return;
        };

        state.elapsed_frames = state.elapsed_frames.saturating_add(1);
        if state.elapsed_frames == PAUSE_RESTART_APPLY_LIFE_FRAME_LOC_6B46 {
            self.apply_pause_restart_life_spend_loc_6b46(state.remaining_lives);
        }
        if state.elapsed_frames >= PAUSE_RESTART_CLEAR_LEVEL_STATE_FRAME_LOC_6E2
            && !state.level_state_cleared
        {
            self.clear_pause_restart_level_state_loc_6e2();
            state.level_state_cleared = true;
        }
        if state.elapsed_frames == PAUSE_RESTART_CLEAR_LEVEL_STATE_FRAME_LOC_6E2 {
            // ASM loc_6C3E clears Game_Paused while the loc_6E2 reload shell
            // is still blank; the live level state is rebuilt later.
            self.paused = false;
            self.pause_menu_option = 0;
            self.pause_menu_arrow_cursor_cmds_loc_6a84 = None;
            self.pause_toggle_pending = false;
        }
        if state.elapsed_frames >= PAUSE_RESTART_REBUILD_LEVEL_STATE_FRAME_LOC_6E2 {
            self.finish_pause_restart_round_loc_6c3e();
        } else {
            self.pending_pause_restart_round = Some(state);
        }
    }

    #[allow(non_snake_case)]
    #[allow(dead_code)]
    fn stop_z80_guard_sub_914(&mut self) {
        // ASM sub_914 (kid.asm:1027): increment `$FFFFF805`, raise SR to
        // `$2700`, then call `j_Stop_z80` unconditionally. Nested callers keep
        // stacking the byte counter even though the bus is already halted.
        self.z80_stop_request_depth = self.z80_stop_request_depth.wrapping_add(1);
        self.z80_guard_sr = 0x2700;
        self.z80_running = false;
    }

    #[allow(non_snake_case)]
    #[allow(dead_code)]
    fn prepare_plane_clear_pass_loc_1698(&mut self) -> PlaneClearPassSetupLoc1698 {
        self.stop_z80_guard_sub_914();
        // ASM loc_1698 (kid.asm:2803): after `j_sub_914`, the ROM targets
        // `$FE0000`, seeds `d0` with `#$3F` for the first 64-word `dbf` fill
        // pass, and stores `#$20` into `4(a6)` before entering `loc_16B6`.
        PlaneClearPassSetupLoc1698 {
            target_addr: 0x00FE_0000,
            dbf_seed: 0x003F,
            next_source_longword: 0x0000_0020,
        }
    }

    #[allow(dead_code)]
    fn start_z80_guard_sub_924(&mut self) {
        // ASM sub_924 (kid.asm:1038): decrement the shared byte, branch on a
        // signed `bgt` if nested stops remain, otherwise clear the byte,
        // restore SR=`$2500`, and call `j_Start_z80`.
        let remaining_depth = self.z80_stop_request_depth.wrapping_sub(1);
        self.z80_stop_request_depth = remaining_depth;
        if (remaining_depth as i8) > 0 {
            return;
        }
        self.z80_stop_request_depth = 0;
        self.z80_guard_sr = 0x2500;
        self.z80_running = true;
    }

    fn load_platform_script_table(&mut self) {
        // Redirect from the legacy `source_root` read to the bundle: `level` is
        // a COPIED_DIRECTORY, so `level/platform_scripts.asm` is byte-identical
        // under `output_root`. Best-effort: a miss leaves the table unset.
        if let Ok(bytes) = self
            .assets_source
            .read(std::path::Path::new("level/platform_scripts.asm"))
            && let Ok(contents) = String::from_utf8(bytes)
        {
            self.platform_script_table = Some(wildside_data::parse_platform_scripts(&contents));
        }
    }

    pub fn with_generated_level_layout(assets: AssetManifest) -> Self {
        Self::new(assets)
    }

    /// Like [`with_generated_level_layout`](Self::with_generated_level_layout)
    /// but reads generated-bundle bytes through an injected [`AssetSource`].
    pub fn with_generated_level_layout_with_source(
        assets: AssetManifest,
        assets_source: Arc<dyn AssetSource>,
    ) -> Self {
        Self::new_with_source(assets, assets_source)
    }

    /// Override the physics sub-step ordering (for sweep comparison experiments).
    pub fn set_physics_order(&mut self, order: PhysicsOrder) {
        self.physics_order = order;
    }

    /// Enable sub-frame physics tracing for TAS debugging.
    pub fn set_trace_physics(&mut self, enabled: bool) {
        self.trace_physics = enabled;
    }

    /// Select oracle-recorded title-card hand-off timing for scripted replay tools.
    pub fn set_recorded_title_card_handoff_delay(&mut self, enabled: bool) {
        self.recorded_title_card_handoff_delay = enabled;
        self.direct_title_card_handoff_capture = !enabled;
    }

    /// Set the block collision grid for the current level.
    /// Call after constructing the runtime, before stepping frames.
    pub fn set_block_grid(&mut self, grid: terrain::BlockGrid) {
        self.block_grid = Some(grid);
        self.block_grid_source = BlockGridSource::Manual;
    }

    /// Set the block collision grid during a level-entry shell and rebuild the
    /// hidden opening slice against that exact terrain.
    pub fn set_level_entry_block_grid(&mut self, grid: terrain::BlockGrid) {
        self.set_block_grid(grid);
        if matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::LevelSkipHold
        ) {
            // ASM loc_74E0/loc_1AE2E re-enters Load_InGame with the target
            // level terrain already loaded; play-native swaps generated assets
            // at the shell boundary, so the hidden spawn settle must be rebuilt
            // from the freshly loaded grid before gameplay is released.
            self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(self.current_level));
        }
    }

    pub fn level_entry_opening_player_position(&self) -> Option<(u32, u32)> {
        self.opening_slice
            .map(|slice| (slice.player.x_pos, slice.player.y_pos))
    }

    /// Level assets backing the source-port renderer's currently presented
    /// image. ASM loc_119D8 changes `Current_LevelID` to zero before
    /// Pal_FadeOut has finished presenting the demo gameplay planes.
    pub fn source_port_render_level(&self) -> LevelId {
        if self.demo_title_palette_fade_progress_loc_28c6().is_some() {
            self.demo_title_visual_level.unwrap_or(self.current_level)
        } else {
            self.current_level
        }
    }

    /// Terminal gameplay trace retained through the synchronous demo-return
    /// fade after Current_LevelID has already been reset.
    pub fn source_port_render_trace_snapshot(&self) -> Option<&NativeTraceFrame> {
        self.demo_title_palette_fade_progress_loc_28c6()
            .and(self.demo_title_render_snapshot.as_deref())
            .map(|snapshot| &snapshot.trace)
    }

    /// Outgoing mutated layout retained alongside the demo-return trace.
    pub fn source_port_render_block_grid(&self) -> Option<&terrain::BlockGrid> {
        if self.demo_title_palette_fade_progress_loc_28c6().is_some() {
            self.demo_title_render_snapshot
                .as_deref()
                .and_then(|snapshot| snapshot.block_grid.as_ref())
                .or(self.block_grid.as_ref())
        } else {
            self.block_grid.as_ref()
        }
    }

    /// ASM `loc_124A6`/`sub_F730` state for the SolidGhost group owning a
    /// layout cell. Kept as scalar diagnostics so render tooling need not
    /// depend on the runtime's private block-worker representation.
    pub fn source_port_ghost_group_at(&self, col: u16, row: u16) -> Option<(u16, u16, bool)> {
        self.block_state.ghost_groups.iter().find_map(|group| {
            let contains = if group.vertical {
                col == group.col
                    && row >= group.row
                    && row < group.row.saturating_add(u16::from(group.count))
            } else {
                row == group.row
                    && col >= group.col
                    && col < group.col.saturating_add(u16::from(group.count))
            };
            contains.then_some((group.timer, group.state, group.solid))
        })
    }

    fn intro_sequence_2_fifo_lineage_loc_1c194(&self) -> vdp::IntroSequence2FifoLineageLoc1C194 {
        use vdp::IntroSequence2FifoLineageLoc1C194 as Lineage;

        // ASM loc_1BA02 -> sub_1C180/loc_1C194 inherits the Standard-mode
        // CPU/FIFO phase established by the preceding attract/intro lineage.
        // These states are durable mode-history markers, unlike frame_index;
        // oracle FIFO probes pin each reachable published-route combination.
        match (
            self.title_screen_region_byte,
            self.demo_toggle,
            self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility,
            self.timeout_restart_intro_sequence_1_uses_early_title_restore,
        ) {
            (0, false, true, false) => Lineage::Region0EarlySlot,
            (1, true, false, false) => Lineage::Region1Demo,
            (2, false, true, true) => Lineage::Region2EarlyRestore,
            (0, true, true, true) => Lineage::Region0DemoEarlyRestore,
            (2, true, true, true) => Lineage::Region2DemoEarlyRestore,
            (0, true, false, false) => Lineage::Region0Demo,
            (1, false, false, false) => Lineage::Region1,
            (0, _, _, _) => Lineage::Region0Demo,
            (1, _, _, _) => Lineage::Region1,
            _ => Lineage::Region2EarlyRestore,
        }
    }

    /// Rebuild VDP state based on the current boot phase.
    /// Called once per frame after `step_boot_state`.
    fn update_vdp_state(&mut self) {
        let previous_vdp_state = self.vdp_state.clone();
        if matches!(
            self.boot_state,
            BootState::IntroSequence {
                segment: 2,
                elapsed: 0,
                ..
            }
        ) {
            // ASM Load_IntroSequence3 -> sub_1B93E -> sub_1C204 presents the
            // resident IntroSequence2 image before fading its CRAM to black.
            self.intro_sequence_3_handoff_vdp_snapshot
                .clone_from(&previous_vdp_state);
        }
        if matches!(
            self.boot_state,
            BootState::IntroSequence {
                segment: 3,
                elapsed: 0,
                ..
            }
        ) {
            // ASM sub_1B93E -> loc_1B96A changes Game_Mode before loc_6E2's
            // Pal_FadeOut has consumed the resident IntroSequence3 image.
            self.intro_sequence_4_handoff_vdp_snapshot
                .clone_from(&previous_vdp_state);
        }
        if matches!(
            self.boot_state,
            BootState::IntroSequence {
                segment: 4,
                elapsed: 0,
                ..
            }
        ) {
            // ASM sub_1B988 loc_1B9B6 changes Game_Mode before loc_6E2's
            // Pal_FadeOut has consumed the resident IntroSequence4 image.
            self.intro_sequence_5_handoff_vdp_snapshot
                .clone_from(&previous_vdp_state);
        }
        self.vdp_state = match &self.boot_state {
            BootState::SegaScreen { elapsed } => self
                .sega_screen_data
                .as_ref()
                .map(|data| vdp::build_sega_screen_vdp(data, *elapsed)),
            BootState::TitleScreen { .. }
                if self.demo_title_palette_fade_progress_loc_28c6().is_some() =>
            {
                // ASM loc_119D8 -> Pal_FadeOut keeps the gameplay planes and
                // sprites resident while CRAM fades towards `$082A`.
                None
            }
            BootState::TitleScreen { .. } if self.demo_title_solid_fill_active_loc_6e2() => {
                Some(vdp::solid_colour_vdp_state(0x082A))
            }
            BootState::TitleScreen { .. } if self.demo_auto_timeout_handoff_active => {
                self.sega_screen_data.as_ref().map(|data| {
                    // ASM ReadJoypad loc_C0A writes Game_Mode=$00 for the demo
                    // auto-timeout restart. Once Pal_FadeOut and loc_6E2's
                    // `$082A` clear hold finish, the visible VDP follows the
                    // mode-0 Sega loader rather than the title shell.
                    vdp::build_sega_screen_vdp(data, self.title_screen_elapsed_frames)
                })
            }
            BootState::IntroSequence {
                segment, elapsed, ..
            } => self.intro_sub_1b850_data.as_ref().map(|data| {
                if *segment == 0
                    && let Some(sega_data) = self.sega_screen_data.as_ref()
                {
                    if self
                        .demo_auto_timeout_intro_sequence_1_frames_override
                        .is_some()
                    {
                        return vdp::build_intro_sequence_1_demo_restart_vdp_at_elapsed(
                            data,
                            sega_data,
                            self.title_screen_region_byte,
                            *elapsed,
                        );
                    }
                    return vdp::build_intro_sequence_1_cold_boot_vdp_at_elapsed(
                        data,
                        sega_data,
                        self.title_screen_region_byte,
                        *elapsed,
                    );
                }
                if *segment == 3 {
                    return vdp::build_intro_sequence_4_handoff_vdp_at_elapsed(
                        data,
                        self.title_screen_region_byte,
                        *elapsed,
                        self.intro_sequence_4_handoff_vdp_snapshot.as_ref(),
                    );
                }
                if *segment == 1 {
                    return vdp::build_intro_sequence_2_vdp_at_elapsed_with_fifo_phase_loc_1c194(
                        data,
                        self.title_screen_region_byte,
                        *elapsed,
                        self.intro_sequence_2_fifo_lineage_loc_1c194(),
                    );
                }
                if *segment == 2 {
                    return vdp::build_intro_sequence_3_handoff_vdp_at_elapsed(
                        data,
                        self.title_screen_region_byte,
                        *elapsed,
                        self.active_intro_sequence_3_roster_index,
                        self.intro_sequence_3_handoff_vdp_snapshot.as_ref(),
                    );
                }
                if *segment == 4
                    && let Some(vdp) = vdp::build_intro_sequence_5_handoff_vdp_at_elapsed(
                        *elapsed,
                        self.intro_sequence_5_handoff_vdp_snapshot.as_ref(),
                    )
                {
                    return vdp;
                }
                if *segment == 4
                    && self
                        .demo_auto_timeout_intro_sequence_5_frames_override
                        .is_some()
                {
                    return vdp::build_timeout_restart_intro_sequence_5_vdp_at_elapsed_loc_1ba28(
                        data,
                        self.title_screen_region_byte,
                        *elapsed,
                        self.active_intro_sequence_3_roster_index,
                        self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility,
                    );
                }
                vdp::build_intro_sequence_vdp_with_title_variant_at_elapsed(
                    data,
                    *segment,
                    self.title_screen_region_byte,
                    *elapsed,
                    self.active_intro_sequence_3_roster_index,
                )
            }),
            BootState::IntroSequence6 { .. }
                if !self.title_screen_vdp_ready()
                    && self
                        .intro_sequence_6_hidden_shell_frame_index_loc_1b43a()
                        .is_none() =>
            {
                self.intro_sub_1b850_data.as_ref().map(|data| {
                    // ASM loc_1BA5C writes `Game_Mode=$28`, then loc_1BA90 keeps
                    // the intro-5 worker shell alive through the fade counter.
                    // Load_IntroSequence6 then clears/fills VRAM, but
                    // sub_1B43A/loc_1B446 still hibernates for `$2A` frames
                    // before the first sub_1BF24 transformation frame is
                    // visible. Keep the fully faded `$AAA` shell during that
                    // blank interval; rebuilding the title-character tilemap
                    // earlier leaves the stale white ninja silhouette visible.
                    vdp::build_intro_sequence_5_fade_hold_vdp_at_elapsed(
                        data,
                        self.title_screen_region_byte,
                        self.title_screen_elapsed_frames,
                        self.active_intro_sequence_3_roster_index,
                    )
                })
            }
            BootState::IntroSequence6 { cursor, .. }
                if self.intro_sequence_6_natural_title_workers_active() =>
            {
                self.title_screen_data.as_ref().map(|data| {
                    let title_worker_elapsed =
                        self.intro_sequence_6_title_worker_presentation_elapsed_loc_1b45a();
                    let menu_cursor = self
                        .title_menu_vdp_active_loc_1bb52()
                        .then_some(*cursor);
                    vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                        data,
                        7,
                        self.title_screen_region_byte,
                        menu_cursor,
                        true,
                        title_worker_elapsed,
                        vdp::TitleScreenVdpLineage {
                            natural_intro_sequence_6_pre_ready_shell: true,
                            natural_intro_sequence_6_worker_elapsed: Some(title_worker_elapsed),
                            title_menu_raster_upload: self
                                .title_menu_vdp_raster_upload_active_loc_1bb52(),
                            ..vdp::TitleScreenVdpLineage::default()
                        },
                    )
                })
            }
            BootState::IntroSequence6 { .. }
                if self.intro_sequence_6_natural_title_vdp_rebuilt() =>
            {
                self.title_screen_data.as_ref().map(|data| {
                    vdp::build_intro_sequence_6_vdp_after_loc_1b45a_title_art(
                        data,
                        self.intro_sequence_6_lightning_flash_frame_loc_1b37c(),
                        self.title_screen_region_byte,
                        self.intro_sequence_6_level_layout_x(),
                        self.intro_sequence_6_presented_hidden_camera_y(),
                    )
                })
            }
            BootState::IntroSequence6 { .. }
                if self.intro_sequence_6_loc_1b45a_vdp_phase().is_some() =>
            {
                let phase = self
                    .intro_sequence_6_loc_1b45a_vdp_phase()
                    .expect("guard ensures loc_1B45A VDP phase is present");
                self.title_screen_data.as_ref().map(|data| match phase {
                    IntroSequence6Loc1b45aVdpPhase::BlackoutBeforeTitleRows => {
                        vdp::build_intro_sequence_6_vdp_at_loc_1b470_palette_blackout(
                            data,
                            self.intro_sequence_6_lightning_flash_frame_loc_1b37c(),
                            self.title_screen_region_byte,
                            self.intro_sequence_6_level_layout_x(),
                            self.intro_sequence_6_presented_hidden_camera_y(),
                            false,
                        )
                    }
                    IntroSequence6Loc1b45aVdpPhase::BlackoutAfterTitleRows => {
                        vdp::build_intro_sequence_6_vdp_at_loc_1b470_palette_blackout(
                            data,
                            self.intro_sequence_6_lightning_flash_frame_loc_1b37c(),
                            self.title_screen_region_byte,
                            self.intro_sequence_6_level_layout_x(),
                            self.intro_sequence_6_presented_hidden_camera_y(),
                            true,
                        )
                    }
                    IntroSequence6Loc1b45aVdpPhase::RestoredBeforeTitleArt => {
                        vdp::build_intro_sequence_6_vdp_after_loc_1b494_palette_restore(
                            data,
                            self.intro_sequence_6_lightning_flash_frame_loc_1b37c(),
                            self.title_screen_region_byte,
                            self.intro_sequence_6_level_layout_x(),
                            self.intro_sequence_6_presented_hidden_camera_y(),
                        )
                    }
                })
            }
            BootState::IntroSequence6 { .. } if !self.title_screen_vdp_ready() => {
                self.title_screen_data.as_ref().map(|data| {
                    vdp::build_intro_sequence_6_vdp_with_character_frame(
                        data,
                        self.intro_sequence_6_hidden_shell_frame_index_loc_1b43a(),
                        self.intro_sequence_6_lightning_flash_frame_loc_1b37c(),
                        vdp::IntroSequence6RasterUploads {
                            lightning: self.intro_sequence_6_lightning_raster_upload(),
                            title: self.intro_sequence_6_title_raster_upload(),
                        },
                        self.title_screen_region_byte,
                        self.intro_sequence_6_level_layout_x(),
                        self.intro_sequence_6_presented_hidden_camera_y(),
                    )
                })
            }
            BootState::TitleScreen { .. }
                if self
                    .intro_sequence_6_start_skip_natural_vdp_carry
                    .and_then(|carry| {
                        let elapsed = self
                            .title_screen_elapsed_frames
                            .saturating_add(carry.elapsed_at_mode_switch);
                        (elapsed < TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_TITLE_POSE_VISIBLE_ELAPSED_FRAMES)
                            .then_some(elapsed)
                    })
                    .is_some() =>
            {
                let carry = self
                    .intro_sequence_6_start_skip_natural_vdp_carry
                    .expect("guard ensures natural IntroSequence6 VDP elapsed is present");
                let elapsed = self
                    .title_screen_elapsed_frames
                    .saturating_add(carry.elapsed_at_mode_switch);
                let camera_y_pos =
                    Self::intro_sequence_6_presented_hidden_camera_y_for_elapsed(
                    elapsed,
                    carry.carry_timeout_restart_hidden_camera_wrap,
                    false,
                )
                    .unwrap_or_else(|| self.current_camera_state().y_pos);
                let level_layout_x_pos = Self::intro_sequence_6_level_layout_x_for_elapsed(
                    elapsed,
                    carry.carry_timeout_restart_hidden_camera_wrap,
                    false,
                );
                self.title_screen_data.as_ref().map(|data| {
                    if elapsed >= INTRO_SEQUENCE_6_NATURAL_TITLE_VDP_READY_FRAMES {
                        vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                            data,
                            7,
                            self.title_screen_region_byte,
                            None,
                            true,
                            elapsed,
                            vdp::TitleScreenVdpLineage {
                                intro_return_hidden_shell: false,
                                direct_intro_sequence_5_title_return: false,
                                natural_intro_sequence_6_pre_ready_shell: true,
                                natural_intro_sequence_6_worker_elapsed: None,
                                fresh_title_worker_lineage: false,
                                ..vdp::TitleScreenVdpLineage::default()
                            },
                        )
                    } else if elapsed >= INTRO_SEQUENCE_6_NATURAL_TITLE_VDP_REBUILD_FRAMES {
                        vdp::build_intro_sequence_6_vdp_after_loc_1b45a_title_art(
                            data,
                            Self::intro_sequence_6_lightning_flash_frame_loc_1b37c_at_elapsed(
                                elapsed,
                            ),
                            self.title_screen_region_byte,
                            level_layout_x_pos,
                            camera_y_pos,
                        )
                    } else {
                        vdp::build_intro_sequence_6_vdp_with_character_frame(
                            data,
                            Self::intro_sequence_6_hidden_shell_frame_index_loc_1b43a_at_elapsed(
                                elapsed,
                            ),
                            Self::intro_sequence_6_lightning_flash_frame_loc_1b37c_at_elapsed(
                                elapsed,
                            ),
                            vdp::IntroSequence6RasterUploads::default(),
                            self.title_screen_region_byte,
                            level_layout_x_pos,
                            camera_y_pos,
                        )
                    }
                })
            }
            BootState::TitleScreen { .. }
                if self.intro_sequence_6_start_skip_intro_sequence_5_pose_carry
                    && !self.intro_sequence_6_start_skip_cursor_trace_priority
                    && !self.title_screen_vdp_ready() =>
            {
                // ASM loc_1BAB0 (kid.asm:30370) can switch Game_Mode via
                // sub_1C246 while sibling intro workers still own the current
                // VDP image. Oracle Start-at-2400/2450 probes carry that
                // resident image only briefly after the `$28 -> $2C` jump,
                // then the `Load_TitleScreen` restore is black until the
                // title VDP rebuild reaches the short 58-frame reveal.
                self.intro_sequence_5_title_return_vdp_snapshot
                    .clone()
                    .and_then(|mut vdp| {
                        vdp::apply_intro_title_return_fade_to_black_loc_28c6(
                            &mut vdp,
                            self.title_screen_elapsed_frames,
                        )
                        .then_some(vdp)
                    })
                    .or_else(|| Some(vdp::blank_vdp_state()))
            }
            BootState::TitleScreen { .. }
                if matches!(
                    self.title_screen_intro_return_lineage,
                    Some(
                        TitleScreenIntroReturnLineage::ColdIntroSequence1
                            | TitleScreenIntroReturnLineage::TimeoutRestartIntroSequence1EarlyRestore
                    )
                ) && self
                    .intro_return_load_title_screen_reveal_elapsed()
                    .is_some_and(|reveal| self.title_screen_elapsed_frames < reveal) =>
            {
                self.intro_sequence_1_title_return_vdp_snapshot
                    .clone()
                    .and_then(|mut vdp| {
                        vdp::apply_intro_title_return_fade_to_black_loc_28c6(
                            &mut vdp,
                            self.title_screen_elapsed_frames,
                        )
                        .then_some(vdp)
                    })
                    .or_else(|| Some(vdp::blank_vdp_state()))
            }
            BootState::TitleScreen { .. }
                if matches!(
                    self.title_screen_intro_return_lineage,
                    Some(TitleScreenIntroReturnLineage::TimeoutRestartIntroSequence5)
                ) && self.title_screen_elapsed_frames
                    < if self.timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay
                    {
                        START_SKIP_TIMEOUT_RESTART_INTRO_SEQUENCE_5_TITLE_RNG_FIRST_LIVE_ELAPSED
                    } else {
                        TIMEOUT_RESTART_INTRO_SEQUENCE_5_TITLE_RNG_FIRST_LIVE_ELAPSED
                    } =>
            {
                self.intro_sequence_5_title_return_vdp_snapshot
                    .clone()
                    .and_then(|mut vdp| {
                        vdp::apply_intro_title_return_fade_to_black_loc_28c6(
                            &mut vdp,
                            self.title_screen_elapsed_frames,
                        )
                        .then_some(vdp)
                    })
                    .or_else(|| Some(vdp::blank_vdp_state()))
            }
            BootState::TitleScreen { .. }
                if self.title_screen_uses_intro_return_hidden_shell
                    && self.timeout_restart_title_intro_sequence_5_pose
                    && !self.intro_sequence_6_start_skip_intro_sequence_5_pose_carry
                    && !self.title_screen_vdp_ready() =>
            {
                self.intro_sequence_5_title_return_vdp_snapshot
                    .clone()
                    .and_then(|mut vdp| {
                        vdp::apply_intro_title_return_fade_to_black_loc_28c6(
                            &mut vdp,
                            self.title_screen_elapsed_frames,
                        )
                        .then_some(vdp)
                    })
                    .or_else(|| Some(vdp::blank_vdp_state()))
            }
            BootState::TitleScreen { .. }
                if matches!(
                    self.title_screen_intro_return_lineage,
                    Some(TitleScreenIntroReturnLineage::TimeoutRestartIntroSequence3)
                ) && self.title_screen_elapsed_frames
                    < TIMEOUT_RESTART_INTRO_SEQUENCE_3_TITLE_RNG_FIRST_LIVE_ELAPSED =>
            {
                self.timeout_restart_intro_sequence_3_title_return_vdp_snapshot
                    .clone()
                    .and_then(|mut vdp| {
                        vdp::apply_intro_title_return_fade_to_black_loc_28c6(
                            &mut vdp,
                            self.title_screen_elapsed_frames,
                        )
                        .then_some(vdp)
                    })
                    .or_else(|| Some(vdp::blank_vdp_state()))
            }
            BootState::TitleScreen { .. }
                if matches!(
                    self.title_screen_intro_return_lineage,
                    Some(TitleScreenIntroReturnLineage::LaterIntroSequence2)
                ) && self.title_screen_elapsed_frames
                    < INTRO_SEQUENCE_2_TITLE_RNG_FIRST_LIVE_ELAPSED =>
            {
                self.intro_sequence_2_title_return_vdp_snapshot
                    .clone()
                    .and_then(|mut vdp| {
                        vdp::apply_intro_title_return_fade_to_black_loc_28c6(
                            &mut vdp,
                            self.title_screen_elapsed_frames,
                        )
                        .then_some(vdp)
                    })
                    .or_else(|| Some(vdp::blank_vdp_state()))
            }
            BootState::TitleScreen { .. }
                if matches!(
                    self.title_screen_intro_return_lineage,
                    Some(TitleScreenIntroReturnLineage::LaterIntroSequence)
                ) && self.title_screen_elapsed_frames
                    < TIMEOUT_RESTART_INTRO_SEQUENCE_4_TITLE_RNG_FIRST_LIVE_ELAPSED =>
            {
                self.intro_sequence_4_title_return_vdp_snapshot
                    .clone()
                    .and_then(|mut vdp| {
                        vdp::apply_intro_title_return_fade_to_black_loc_28c6(
                            &mut vdp,
                            self.title_screen_elapsed_frames,
                        )
                        .then_some(vdp)
                    })
                    .or_else(|| Some(vdp::blank_vdp_state()))
            }
            BootState::TitleScreen { .. }
                if self.intro_return_load_title_screen_reveal_elapsed()
                    == Some(self.title_screen_elapsed_frames) =>
            {
                self.title_screen_data.as_ref().map(|data| {
                    // ASM sub_1C246 -> loc_1C264 -> loc_6E2 blocks through
                    // Pal_FadeOut, clears VRAM, then runs Load_TitleScreen on
                    // the cold IntroSequence1, loc_1B9D0 -> loc_1BA02
                    // IntroSequence2, and sub_1B988 IntroSequence4 returns.
                    // Their first presented frame contains the base uploads
                    // but precedes the fresh title workers' first loc_E12 pass.
                    vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                        data,
                        7,
                        self.title_screen_region_byte,
                        None,
                        false,
                        57,
                        vdp::TitleScreenVdpLineage {
                            intro_return_hidden_shell: true,
                            direct_intro_sequence_5_title_return: false,
                            natural_intro_sequence_6_pre_ready_shell: false,
                            natural_intro_sequence_6_worker_elapsed: None,
                            fresh_title_worker_lineage: true,
                            ..vdp::TitleScreenVdpLineage::default()
                        },
                    )
                })
            }
            BootState::TitleScreen { .. } if self.title_screen_vdp_blackout() => {
                Some(vdp::blank_vdp_state())
            }
            BootState::IntroSequence6 { .. } | BootState::TitleScreen { .. } => {
                self.title_screen_data.as_ref().map(|data| {
                    let menu_cursor = if self.title_menu_vdp_active_loc_1bb52()
                        && !self.cold_intro_sequence_1_pre_vdp_menu_activation()
                    {
                        match self.boot_state {
                            BootState::TitleScreen { cursor }
                            | BootState::IntroSequence6 { cursor, .. } => Some(cursor),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let manual_demo_return = matches!(
                        self.title_screen_intro_return_lineage,
                        Some(TitleScreenIntroReturnLineage::ManualDemoReturn)
                    );
                    let timeout_restart_intro_sequence_3_return = matches!(
                        self.title_screen_intro_return_lineage,
                        Some(TitleScreenIntroReturnLineage::TimeoutRestartIntroSequence3)
                    );
                    let timeout_restart_intro_sequence_5_return = matches!(
                        self.title_screen_intro_return_lineage,
                        Some(TitleScreenIntroReturnLineage::TimeoutRestartIntroSequence5)
                    );
                    let timeout_restart_intro_sequence_1_early_return = matches!(
                        self.title_screen_intro_return_lineage,
                        Some(
                            TitleScreenIntroReturnLineage::TimeoutRestartIntroSequence1EarlyRestore
                        )
                    );
                    let later_intro_sequence_4_return = matches!(
                        self.title_screen_intro_return_lineage,
                        Some(TitleScreenIntroReturnLineage::LaterIntroSequence)
                    );
                    let title_elapsed_frames = if manual_demo_return
                        || timeout_restart_intro_sequence_1_early_return
                    {
                        self.effective_title_screen_rebuild_elapsed()
                    } else if timeout_restart_intro_sequence_5_return {
                        // ASM loc_1BA60 -> sub_1C246 -> loc_1C264 returns from
                        // Load_TitleScreen before the first resumed loc_E12
                        // presentation. The ordinary return's fresh
                        // Obj_TitleTextChameleon pass is two renderer ticks
                        // ahead of the raw title-mode clock; the later
                        // Start-skip lineage carries one additional hidden
                        // loader frame. The independently verified RNG cadence
                        // still resumes only once on either frame.
                        self.title_screen_elapsed_frames.saturating_add(
                            if self
                                .timeout_restart_title_intro_sequence_5_uses_intro_return_hidden_delay
                            {
                                1
                            } else {
                                2
                            },
                        )
                    } else if timeout_restart_intro_sequence_3_return
                        || later_intro_sequence_4_return
                    {
                        // ASM sub_1B93E and sub_1B988's title loaders rejoin
                        // loc_E12 one frame ahead of the generic intro return.
                        self.title_screen_elapsed_frames.saturating_add(1)
                    } else {
                        self.title_screen_elapsed_frames
                    };
                    // ASM loc_119D8 -> loc_6E2 -> Load_TitleScreen presents
                    // sub_1BF24/sub_1B532 once before the newly allocated title
                    // objects have run. The following Mode_Standard pass adds
                    // the Kid/Chameleon title workers.
                    let fresh_title_worker_lineage =
                        manual_demo_return
                            || timeout_restart_intro_sequence_3_return
                            || timeout_restart_intro_sequence_5_return
                            || self.intro_return_load_title_screen_reveal_elapsed().is_some();
                    let fresh_title_base_upload =
                        fresh_title_worker_lineage && title_elapsed_frames == 57;
                    let title_vdp_ready =
                        self.title_screen_vdp_ready() && !fresh_title_base_upload;
                    let frame_index = if title_vdp_ready {
                        7
                    } else {
                        self.title_screen_character_frame_index_sub_1bf24()
                    };
                    let natural_intro_sequence_6_pre_ready_shell =
                        matches!(self.boot_state, BootState::IntroSequence6 { .. })
                            && !self.intro_sequence_6_start_skip_cursor_trace_priority
                            && title_vdp_ready
                            && !self.title_screen_ready();
                    vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                        data,
                        frame_index,
                        self.title_screen_region_byte,
                        menu_cursor,
                        title_vdp_ready,
                        title_elapsed_frames,
                        vdp::TitleScreenVdpLineage {
                            intro_return_hidden_shell: self
                                .title_screen_uses_intro_return_hidden_shell,
                            direct_intro_sequence_5_title_return: self
                                .title_screen_uses_intro_return_hidden_shell
                                && self.timeout_restart_title_intro_sequence_5_pose
                                && !self.intro_sequence_6_start_skip_intro_sequence_5_pose_carry,
                            natural_intro_sequence_6_pre_ready_shell,
                            natural_intro_sequence_6_worker_elapsed:
                                natural_intro_sequence_6_pre_ready_shell
                                    .then_some(title_elapsed_frames),
                            fresh_title_worker_lineage,
                            title_menu_raster_upload: self
                                .title_menu_vdp_raster_upload_active_loc_1bb52(),
                        },
                    )
                })
            }
            BootState::StartingLevel { .. } => {
                if let Some((vdp, carry_until_frame)) = &self.title_menu_level_entry_vdp_carry
                    && self.frame_index < *carry_until_frame
                {
                    let mut vdp = vdp.clone();
                    let carry_start_frame =
                        carry_until_frame.saturating_sub(TITLE_MENU_TO_TITLE_CARD_VDP_CARRY_FRAMES);
                    let fade_progress = Self::title_menu_level_entry_carry_fade_progress_loc_1bbd6(
                        self.frame_index,
                        carry_start_frame,
                    );
                    vdp::apply_pal_fade_out_to_black_loc_28c6(&mut vdp, fade_progress);
                    Some(vdp)
                } else {
                    let theme_index = self.current_level_title_card_theme_index();
                    self.title_card_data
                        .as_ref()
                        .and_then(|data| vdp::build_title_card_vdp(data, theme_index))
                }
            }
            BootState::DemoPlay {
                frame_counter,
                use_demo_2,
                ..
            } if *frame_counter < self.demo_gameplay_start_frame(*use_demo_2) => {
                // ASM Obj_TitleMenu loc_1BB02 -> loc_6E2 keeps the completed
                // title VDP resident through Pal_FadeOut, then Load_DemoPlay
                // -> Load_InGame keeps the remaining bootstrap black.
                self.demo_entry_title_vdp_snapshot
                    .clone()
                    .and_then(|mut vdp| {
                        vdp::apply_title_to_demo_fade_to_black_loc_28c6(
                            &mut vdp,
                            usize::from(*frame_counter),
                        )
                        .then_some(vdp)
                    })
                    .or_else(|| Some(vdp::blank_vdp_state()))
            }
            _ if matches!(self.player_lifecycle, PlayerLifecycle::Respawning { .. }) => {
                // ASM lose_life -> Teleport loc_B8DE writes Game_Mode=$0008
                // and re-enters loc_6E2, which dispatches Load_TitleCard
                // before Load_InGame. The hidden respawn shell therefore
                // renders the current level's title-card VDP background, not
                // a plain black transition frame.
                let theme_index = self.current_level_title_card_theme_index();
                self.title_card_data
                    .as_ref()
                    .and_then(|data| vdp::build_title_card_vdp(data, theme_index))
            }
            _ => None,
        };
    }

    fn intro_sequence_6_hidden_shell_frame_index_loc_1b43a(&self) -> Option<u8> {
        Self::intro_sequence_6_hidden_shell_frame_index_loc_1b43a_at_elapsed(
            self.intro_sequence_6_hidden_shell_presentation_elapsed_loc_1b43a(),
        )
    }

    fn intro_sequence_6_hidden_shell_presentation_elapsed_loc_1b43a(&self) -> usize {
        let elapsed = self.title_screen_elapsed_frames;
        let BootState::IntroSequence6 {
            carry_timeout_restart_hidden_camera_wrap,
            ..
        } = self.boot_state
        else {
            return elapsed;
        };
        Self::intro_sequence_6_hidden_shell_presentation_elapsed_loc_1b43a_for_lineage(
            elapsed,
            self.intro_sequence_6_entered_from_timeout_restart,
            carry_timeout_restart_hidden_camera_wrap,
            self.demo_toggle,
        )
    }

    fn intro_sequence_6_hidden_shell_presentation_elapsed_loc_1b43a_for_lineage(
        elapsed: usize,
        entered_from_timeout_restart: bool,
        carry_timeout_restart_hidden_camera_wrap: bool,
        demo_toggle: bool,
    ) -> usize {
        if !entered_from_timeout_restart {
            return elapsed;
        }

        // ASM loc_1BA5C -> Load_IntroSequence6 allocates loc_1B37C and
        // sub_1B43A after the shared fade. The first timeout-restart shell is
        // two presentation frames ahead of cold boot; the later cleared-camera
        // revisit retains the existing three-frame worker delay relative to
        // that first shell.
        let early_elapsed = elapsed.saturating_add(
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_EARLY_WORKER_PRESENTATION_ADVANCE_FRAMES,
        );
        if carry_timeout_restart_hidden_camera_wrap {
            early_elapsed
        } else if !demo_toggle {
            early_elapsed.saturating_sub(TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_WORKER_DELAY_FRAMES)
        } else {
            elapsed
        }
    }

    fn title_menu_level_entry_carry_fade_progress_loc_1bbd6(
        frame_index: usize,
        carry_start_frame: usize,
    ) -> u16 {
        let elapsed = frame_index.saturating_sub(carry_start_frame);
        if elapsed <= 4 {
            // ASM Obj_TitleMenu loc_1BBD6 sets `$FFFFFBCE` before jumping to
            // loc_6E2, so Pal_FadeOut loc_28C6 has already consumed most of
            // its fade by the recorded cold-route VDP carry. Oracle #210
            // probes expose `$FFFFF876=$0040,$0030,...,$0000` across the
            // carried title/menu frames.
            0x0040u16.saturating_sub((elapsed as u16).saturating_mul(0x10))
        } else {
            0
        }
    }

    fn intro_sequence_6_start_skip_preserves_natural_vdp_loc_1bab0_at_elapsed(
        title_screen_elapsed_frames: usize,
    ) -> bool {
        // ASM loc_1BAB0 (kid.asm:30368) polls sub_1C246 while sibling
        // loc_1B37C/sub_1B43A workers are still running. If Start lands after
        // loc_1BA90's fade hold but before loc_1B446 presents its first hidden
        // shell frame, the `$28 -> $2C` jump keeps following that natural VDP
        // lineage. Once loc_1B446 has presented a frame, Load_TitleScreen owns
        // the visible rebuild instead.
        let presented_elapsed = title_screen_elapsed_frames.saturating_add(1);
        title_screen_elapsed_frames > INTRO_SEQUENCE_5_FADE_COUNTER_ITERATIONS
            && Self::intro_sequence_6_hidden_shell_frame_index_loc_1b43a_at_elapsed(
                presented_elapsed,
            )
            .is_none()
    }

    fn intro_sequence_6_hidden_shell_frame_index_loc_1b43a_at_elapsed(
        title_screen_elapsed_frames: usize,
    ) -> Option<u8> {
        let script = crate::boot::title_screen_hidden_shell_script_loc_1B446();
        const LOC_1B446_NATIVE_PRESENTATION_ADVANCE_FRAMES: usize = 16;

        // ASM loc_1BA90 runs the intro-5 fade hold before j_loc_6E2 enters
        // Load_IntroSequence6. Native's title_screen_elapsed_frames includes
        // that handoff window, but sub_1B43A's `$2A` wait starts afterwards.
        // Oracle full-frame captures show the first loc_1B446/sub_1BF24
        // hidden-shell upload already visible at cold-boot frame 2432 and
        // still blank at 2428, 16 native-local frames earlier than the raw
        // object-loop count because the VDP upload is presented before the
        // next traced object snapshot.
        let elapsed = title_screen_elapsed_frames
            .saturating_sub(INTRO_SEQUENCE_5_FADE_COUNTER_ITERATIONS)
            .saturating_add(LOC_1B446_NATIVE_PRESENTATION_ADVANCE_FRAMES);
        let initial = usize::from(script.initial_hibernate_frames);
        if elapsed < initial {
            return None;
        }
        let local = elapsed - initial;
        // ASM loc_1B446 hibernates before each `unk_1B4D6` byte. Oracle frame
        // 2460 still lands in the early `0,1,0` flicker, so native's VDP
        // reconstruction keeps that first repeated-zero window before applying
        // the ROM's 3-frame byte cadence.
        let step = usize::from(script.loop_hibernate_frames).max(1);
        // ASM loc_1B446 presents the initial sub_1BF24(d4=0) title mapping
        // through the first long hidden-shell hold. Oracle VRAM frame 2460
        // still has the `$20C1` frame-0 mapping and only starts the
        // `unk_1B4D6` 0/1 flicker at frame 2481, so the native reconstruction
        // keeps the retained zero frame before advancing through the byte
        // stream at the ROM's 3-frame cadence.
        let frame_index = local.saturating_sub((step * 15) + 1) / step;
        script.title_frames.get(frame_index).copied().or_else(|| {
            // ASM sub_1B43A/loc_1B446 stops dispatching sub_1BF24 after the
            // `$FF` sentinel, but it does not clear the VRAM rectangle; the
            // final `unk_1B4D6` frame remains visible until loc_1B45A/loc_1B5CE
            // overwrite it. Native rebuilds VDP state from scratch each frame,
            // so replay the retained frame explicitly.
            script.title_frames.last().copied()
        })
    }

    fn intro_sequence_6_lightning_flash_frame_loc_1b37c(&self) -> u8 {
        const OPENING_WORKER_FRAMES: usize = 5 * 3;

        let timeout_opening_worker_start = match self.boot_state {
            BootState::IntroSequence6 {
                carry_timeout_restart_hidden_camera_wrap,
                ..
            } if self.intro_sequence_6_entered_from_timeout_restart => Some(
                INTRO_SEQUENCE_6_POST_LOADER_RNG_FIRST_LIVE_FRAME
                    + usize::from(!carry_timeout_restart_hidden_camera_wrap)
                        * TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_WORKER_DELAY_FRAMES,
            ),
            _ => None,
        };
        let timeout_opening_worker_elapsed = timeout_opening_worker_start.and_then(|start| {
            self.title_screen_elapsed_frames
                .checked_sub(start)
                .filter(|&worker_elapsed| worker_elapsed < OPENING_WORKER_FRAMES)
        });
        if let Some(worker_elapsed) = timeout_opening_worker_elapsed {
            return Self::intro_sequence_6_timeout_opening_lightning_flash_frame_loc_1b37c(
                worker_elapsed,
            );
        }

        Self::intro_sequence_6_lightning_flash_frame_loc_1b37c_at_elapsed(
            self.intro_sequence_6_lightning_presentation_elapsed_loc_1b37c(),
        )
    }

    fn intro_sequence_6_timeout_opening_lightning_flash_frame_loc_1b37c(
        worker_elapsed: usize,
    ) -> u8 {
        const HIBERNATE_FRAMES: usize = 3;

        // ASM loc_1B37C (kid.asm:29587): retain frame 0 through the initial
        // three-frame hibernate, then present the opening `1,0,2,0` uploads at
        // three-frame intervals. The wrapped-timeout worker becomes live on
        // local frame 33 and the later revisit on frame 36, independently of
        // their active-display FIFO clocks.
        let completed_uploads = worker_elapsed / HIBERNATE_FRAMES;
        if completed_uploads == 0 {
            return 0;
        }
        crate::boot::title_screen_lightning_sequence_loc_1B37C().steps
            [completed_uploads.saturating_sub(1).min(3)]
        .flash_frame
    }

    /// ASM `loc_1B37C -> sub_1B41C` (kid.asm): active-display upload timing.
    fn intro_sequence_6_lightning_raster_upload(
        &self,
    ) -> Option<vdp::IntroSequence6LightningRasterUpload> {
        let BootState::IntroSequence6 {
            carry_timeout_restart_hidden_camera_wrap,
            ..
        } = self.boot_state
        else {
            return None;
        };
        let presentation_elapsed = self.intro_sequence_6_lightning_presentation_elapsed_loc_1b37c();
        let previous_flash_frame =
            Self::intro_sequence_6_lightning_flash_frame_loc_1b37c_at_elapsed(
                presentation_elapsed.saturating_sub(1),
            );
        let lineage = if !self.intro_sequence_6_entered_from_timeout_restart {
            vdp::IntroSequence6RasterLineage::ColdBoot
        } else if carry_timeout_restart_hidden_camera_wrap {
            vdp::IntroSequence6RasterLineage::WrappedTimeout
        } else {
            vdp::IntroSequence6RasterLineage::LaterTimeout
        };
        vdp::intro_sequence_6_lightning_raster_upload(
            presentation_elapsed,
            previous_flash_frame,
            lineage,
        )
    }

    /// ASM `sub_1B43A -> loc_1B446 -> sub_1BF24` (kid.asm): active-display
    /// hidden-character nametable upload timing.
    fn intro_sequence_6_title_raster_upload(&self) -> Option<vdp::IntroSequence6TitleRasterUpload> {
        let BootState::IntroSequence6 {
            carry_timeout_restart_hidden_camera_wrap,
            ..
        } = self.boot_state
        else {
            return None;
        };
        let presentation_elapsed =
            self.intro_sequence_6_hidden_shell_presentation_elapsed_loc_1b43a();
        // Oracle sub_1C5D0 traces show both timeout lineages entering the
        // repeating loc_1B446 byte stream one object pass ahead of the
        // trace-facing worker clock. Keep that correction local to frames
        // which actually contain the active-display upload; loc_1B45A's later
        // blackout/restore phase retains its separately verified timing.
        let raster_frame_elapsed = presentation_elapsed.saturating_add(usize::from(
            self.intro_sequence_6_entered_from_timeout_restart,
        ));
        let current_frame_index =
            Self::intro_sequence_6_hidden_shell_frame_index_loc_1b43a_at_elapsed(
                raster_frame_elapsed,
            );
        let previous_frame_index =
            Self::intro_sequence_6_hidden_shell_frame_index_loc_1b43a_at_elapsed(
                raster_frame_elapsed.saturating_sub(1),
            );
        let lineage = if !self.intro_sequence_6_entered_from_timeout_restart {
            vdp::IntroSequence6RasterLineage::ColdBoot
        } else if carry_timeout_restart_hidden_camera_wrap {
            vdp::IntroSequence6RasterLineage::WrappedTimeout
        } else {
            vdp::IntroSequence6RasterLineage::LaterTimeout
        };
        vdp::intro_sequence_6_title_raster_upload(
            presentation_elapsed,
            current_frame_index,
            previous_frame_index,
            lineage,
        )
    }

    fn intro_sequence_6_lightning_presentation_elapsed_loc_1b37c(&self) -> usize {
        let elapsed = self.title_screen_elapsed_frames;
        let BootState::IntroSequence6 {
            carry_timeout_restart_hidden_camera_wrap,
            ..
        } = self.boot_state
        else {
            return elapsed;
        };
        Self::intro_sequence_6_lightning_presentation_elapsed_loc_1b37c_for_lineage(
            elapsed,
            self.intro_sequence_6_entered_from_timeout_restart,
            carry_timeout_restart_hidden_camera_wrap,
        )
    }

    fn intro_sequence_6_lightning_presentation_elapsed_loc_1b37c_for_lineage(
        elapsed: usize,
        entered_from_timeout_restart: bool,
        carry_timeout_restart_hidden_camera_wrap: bool,
    ) -> usize {
        // ASM loc_1B37C's opening three-frame hibernations have an independent
        // presentation clock from sibling sub_1B43A. The first wrapped timeout
        // shell is four passes ahead; cold boot and the later timeout revisit
        // are one pass ahead. Oracle FIFO traces pin the same local upload
        // elapsed while their scanline entry points remain lineage-specific.
        if entered_from_timeout_restart && carry_timeout_restart_hidden_camera_wrap {
            elapsed.saturating_add(
                TIMEOUT_RESTART_INTRO_SEQUENCE_6_EARLY_LIGHTNING_PRESENTATION_ADVANCE_FRAMES,
            )
        } else {
            elapsed.saturating_add(INTRO_SEQUENCE_6_LIGHTNING_PRESENTATION_ADVANCE_FRAMES)
        }
    }

    fn intro_sequence_6_lightning_flash_frame_loc_1b37c_at_elapsed(
        title_screen_elapsed_frames: usize,
    ) -> u8 {
        const LOC_1B37C_FIRST_PRESENTED_UPLOAD_DELAY_FRAMES: usize = 3;
        const LOC_1B3BC_LOOP_START_TITLE_ELAPSED_FRAMES: usize = 49;
        const LOC_1B3BC_FIRST_VISIBLE_LOOP_FLASH_TITLE_ELAPSED_FRAMES: usize =
            LOC_1B3BC_LOOP_START_TITLE_ELAPSED_FRAMES + 3;
        const LOC_1B3BC_LOOP_FLASH_COUNT: usize = 0x002C + 1;
        const LOC_1B3BC_CLOSING_FLASH_START_TITLE_ELAPSED_FRAMES: usize =
            LOC_1B3BC_FIRST_VISIBLE_LOOP_FLASH_TITLE_ELAPSED_FRAMES
                + LOC_1B3BC_LOOP_FLASH_COUNT * 3;

        let sequence = crate::boot::title_screen_lightning_sequence_loc_1B37C();
        if title_screen_elapsed_frames >= LOC_1B3BC_FIRST_VISIBLE_LOOP_FLASH_TITLE_ELAPSED_FRAMES {
            if title_screen_elapsed_frames < LOC_1B3BC_CLOSING_FLASH_START_TITLE_ELAPSED_FRAMES {
                // ASM loc_1B3BC (kid.asm:29608): after the opening flashes,
                // the DBF loop uploads `(d1 & 3) + 1` for d1=$2C..0. Oracle
                // WRAM at cold-boot frames 2460/2520/2580 shows this loop
                // starts on native title elapsed 49 and keeps d1=0's flash
                // visible while hibernating before the post-loop zero upload.
                let loop_uploads =
                    (title_screen_elapsed_frames - LOC_1B3BC_LOOP_START_TITLE_ELAPSED_FRAMES) / 3;
                let d1 = 0x002C_u16
                    .saturating_add(1)
                    .saturating_sub(loop_uploads.try_into().unwrap_or(u16::MAX));
                return ((d1 & 3) + 1) as u8;
            }

            // ASM loc_1B3BC -> loc_1B37C (kid.asm:29613-29635): after DBF
            // expires, present the five closing uploads `0,1,0,2,0`, each
            // after its own three-frame hibernate, then retain frame 0 when
            // the worker deletes itself. Use the canonical script tail so the
            // reconstructed VDP cannot silently omit this closing sequence.
            let closing_index = (title_screen_elapsed_frames
                - LOC_1B3BC_CLOSING_FLASH_START_TITLE_ELAPSED_FRAMES)
                / 3;
            return sequence.steps[4 + LOC_1B3BC_LOOP_FLASH_COUNT + closing_index.min(4)]
                .flash_frame;
        }
        // ASM loc_1BA90's intro-5 fade hold precedes Load_IntroSequence6, and
        // loc_1B37C then begins with `j_Hibernate_Object #3` before its first
        // `sub_1B41C` upload is visible. Native rebuilds the whole VDP image
        // from counters, so model that local presentation delay explicitly.
        let worker_elapsed = title_screen_elapsed_frames
            .saturating_sub(INTRO_SEQUENCE_5_FADE_COUNTER_ITERATIONS)
            .saturating_sub(LOC_1B37C_FIRST_PRESENTED_UPLOAD_DELAY_FRAMES);
        let mut elapsed_at_step = 0usize;
        let mut flash_frame = 0u8;
        for step in sequence.steps {
            elapsed_at_step += usize::from(step.hibernate_frames);
            if worker_elapsed < elapsed_at_step {
                break;
            }
            flash_frame = step.flash_frame;
        }
        flash_frame
    }

    fn title_screen_vdp_ready(&self) -> bool {
        self.title_screen_ready()
            || self
                .intro_return_load_title_screen_reveal_elapsed()
                .is_some_and(|elapsed| self.title_screen_elapsed_frames > elapsed)
            || (self.title_screen_uses_intro_return_hidden_shell
                && self.timeout_restart_title_intro_sequence_5_pose
                && !self.intro_sequence_6_start_skip_intro_sequence_5_pose_carry
                && self.title_screen_elapsed_frames >= TITLE_ATTRACT_LOCK_FRAMES)
            || (self.intro_sequence_6_start_skip_cursor_trace_priority
                && self.title_screen_elapsed_frames
                    >= INTRO_SEQUENCE_6_START_SKIP_TITLE_VDP_READY_FRAMES)
            || (self.intro_sequence_6_start_skip_intro_sequence_5_pose_carry
                && !self.intro_sequence_6_start_skip_cursor_trace_priority
                && self.title_screen_elapsed_frames
                    >= INTRO_SEQUENCE_6_EARLY_START_SKIP_TITLE_VDP_READY_FRAMES)
            || (matches!(self.boot_state, BootState::IntroSequence6 { .. })
                && !self.intro_sequence_6_start_skip_cursor_trace_priority
                && self.title_screen_elapsed_frames
                    >= INTRO_SEQUENCE_6_NATURAL_TITLE_VDP_READY_FRAMES)
    }

    fn title_menu_vdp_active_loc_1bb52(&self) -> bool {
        self.title_menu_active
            && self
                .title_menu_vdp_upload_frame_loc_1bb52
                .map_or(!self.title_menu_cursor_pending, |upload_frame| {
                    self.frame_index >= upload_frame
                })
    }

    fn title_menu_vdp_raster_upload_active_loc_1bb52(&self) -> bool {
        self.title_menu_active
            && self.title_menu_vdp_upload_frame_loc_1bb52 == Some(self.frame_index)
    }

    fn cold_intro_sequence_1_title_return(&self) -> bool {
        self.title_screen_uses_intro_return_hidden_shell
            && matches!(
                self.title_screen_intro_return_lineage,
                Some(TitleScreenIntroReturnLineage::ColdIntroSequence1)
            )
    }

    fn intro_return_load_title_screen_reveal_elapsed(&self) -> Option<usize> {
        if !self.title_screen_uses_intro_return_hidden_shell {
            return None;
        }
        match self.title_screen_intro_return_lineage {
            // ASM sub_1B8F4 -> sub_1C246 rejoins the blocking title loader two
            // outer passes ahead of its generic cold IntroSequence1 return.
            Some(TitleScreenIntroReturnLineage::TimeoutRestartIntroSequence1EarlyRestore) => {
                Some(TIMEOUT_RESTART_INTRO_SEQUENCE_1_EARLY_TITLE_RNG_FIRST_LIVE_ELAPSED)
            }
            Some(
                TitleScreenIntroReturnLineage::ColdIntroSequence1
                | TitleScreenIntroReturnLineage::LaterIntroSequence2,
            ) => Some(57),
            // ASM sub_1B988 -> sub_1C246 rejoins the blocking title loader
            // one outer pass earlier than the IntroSequence1/2 returns.
            Some(TitleScreenIntroReturnLineage::LaterIntroSequence) => Some(56),
            _ => None,
        }
    }

    fn cold_intro_sequence_1_pre_vdp_menu_activation(&self) -> bool {
        if !self.cold_intro_sequence_1_title_return() || !self.title_menu_active {
            return false;
        }
        let title_entry_frame = self
            .frame_index
            .saturating_sub(self.title_screen_elapsed_frames);
        let first_visible_vdp_frame = title_entry_frame.saturating_add(57);
        self.title_menu_arm_frame <= first_visible_vdp_frame
    }

    fn intro_sequence_6_natural_title_vdp_rebuilt(&self) -> bool {
        matches!(self.boot_state, BootState::IntroSequence6 { .. })
            && !self.intro_sequence_6_start_skip_cursor_trace_priority
            && !self.title_screen_ready()
            && self.title_screen_elapsed_frames >= INTRO_SEQUENCE_6_NATURAL_TITLE_VDP_REBUILD_FRAMES
            && self.title_screen_elapsed_frames < INTRO_SEQUENCE_6_NATURAL_TITLE_VDP_READY_FRAMES
    }

    fn intro_sequence_6_natural_title_workers_active(&self) -> bool {
        // ASM loc_1B45A allocates loc_1B6AC/loc_1B5CE after Load_TitleArt.
        // Their `$1E` waits end well before loc_1B5CE's trace-facing title
        // pose, so preserve the live reveal throughout mode `$28`. The prior
        // presentation already has GfxObjects_Collision_BottomBoundary4's
        // final zero-scroll row, although the two title workers wake on the
        // following pass.
        matches!(self.boot_state, BootState::IntroSequence6 { .. })
            && !self.intro_sequence_6_start_skip_cursor_trace_priority
            && self.intro_sequence_6_title_worker_presentation_elapsed_loc_1b45a()
                >= INTRO_SEQUENCE_6_NATURAL_TITLE_WORKER_START_FRAMES.saturating_sub(1)
    }

    fn intro_sequence_6_title_worker_presentation_elapsed_loc_1b45a(&self) -> usize {
        let elapsed = self.title_screen_elapsed_frames;
        let BootState::IntroSequence6 {
            carry_timeout_restart_hidden_camera_wrap,
            ..
        } = self.boot_state
        else {
            return elapsed;
        };

        // ASM sub_1B43A -> loc_1B45A allocates loc_1B6AC and loc_1B5CE on
        // the same delayed object schedule as the parent worker. The later
        // timeout-restart revisit resumes those workers three presentations
        // behind the raw mode-$28 clock; their 8px loc_1BB10 slide, 6px
        // loc_1B610 drift, and sub_1B71A map cadence all retain that delay.
        if self.intro_sequence_6_entered_from_timeout_restart
            && !carry_timeout_restart_hidden_camera_wrap
            && !self.demo_toggle
        {
            elapsed.saturating_sub(TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_WORKER_DELAY_FRAMES)
        } else {
            elapsed
        }
    }

    fn intro_sequence_6_loc_1b45a_vdp_phase(&self) -> Option<IntroSequence6Loc1b45aVdpPhase> {
        let BootState::IntroSequence6 {
            carry_timeout_restart_hidden_camera_wrap,
            ..
        } = self.boot_state
        else {
            return None;
        };
        if self.intro_sequence_6_start_skip_cursor_trace_priority
            || self.title_screen_ready()
            || self.title_screen_elapsed_frames >= INTRO_SEQUENCE_6_NATURAL_TITLE_VDP_REBUILD_FRAMES
        {
            return None;
        }

        // ASM sub_1B43A -> loc_1B45A runs on the same worker-local clock as
        // the loc_E12 stall schedule. The later timeout-restart revisit delays
        // that worker by three displayed frames; the wrapped first revisit and
        // cold boot do not. loc_1B470 presents frame 7 while clearing CRAM,
        // then sub_1B532 rebuilds the title-text plane one VBlank later. The
        // saved palette returns before the long Load_TitleArt upload completes.
        let displayed_local_frame = self.title_screen_elapsed_frames.saturating_add(1);
        let later_worker_delay = if self.intro_sequence_6_entered_from_timeout_restart
            && !carry_timeout_restart_hidden_camera_wrap
            && !self.demo_toggle
        {
            TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_WORKER_DELAY_FRAMES
        } else {
            0
        };
        let worker_local_frame = displayed_local_frame.saturating_sub(later_worker_delay);
        match worker_local_frame {
            INTRO_SEQUENCE_6_PALETTE_BLACKOUT_STALL_FRAME_LOC_1B470 => {
                Some(IntroSequence6Loc1b45aVdpPhase::BlackoutBeforeTitleRows)
            }
            frame if frame == INTRO_SEQUENCE_6_PALETTE_BLACKOUT_STALL_FRAME_LOC_1B470 + 1 => {
                Some(IntroSequence6Loc1b45aVdpPhase::BlackoutAfterTitleRows)
            }
            frame if frame >= INTRO_SEQUENCE_6_TITLE_ART_STALL_FIRST_FRAME_LOC_1B494 => {
                Some(IntroSequence6Loc1b45aVdpPhase::RestoredBeforeTitleArt)
            }
            _ => None,
        }
    }

    fn title_screen_vdp_blackout(&self) -> bool {
        // The second branch's `&& !skip` is redundant: the outer `skip ||` already
        // short-circuits when skip is set, so it is dropped here.
        (self.intro_sequence_6_start_skip_cursor_trace_priority
            || (self.title_screen_uses_intro_return_hidden_shell
                && !self.intro_sequence_6_start_skip_intro_sequence_5_pose_carry))
            && !self.title_screen_vdp_ready()
    }

    fn title_screen_character_frame_index_sub_1bf24(&self) -> u8 {
        if self.title_screen_ready() {
            // ASM Load_TitleScreen (kid.asm:29456): `moveq #7,d4; bsr.w
            // sub_1BF24` draws the visible title character once the hidden
            // intro-return rebuild has completed.
            return 7;
        }
        if !self.title_screen_uses_intro_return_hidden_shell {
            return 0;
        }

        let script = crate::boot::title_screen_hidden_shell_script_loc_1B446();
        let elapsed = self.title_screen_elapsed_frames;
        let initial = usize::from(script.initial_hibernate_frames);
        if elapsed < initial {
            return 0;
        }
        let local = elapsed - initial;
        let step = usize::from(script.loop_hibernate_frames).max(1);
        let frame_index = local / step;
        script
            .title_frames
            .get(frame_index)
            .copied()
            .unwrap_or_else(|| {
                crate::boot::title_screen_hidden_shell_palette_restore_loc_1B45A().blackout_frame
            })
    }

    fn current_level_title_card_theme_index(&self) -> u8 {
        load_level_asset_bundle_with_source(
            &self.assets,
            self.current_level.0,
            self.assets_source.as_ref(),
        )
        .ok()
        .and_then(|bundle| bundle.parsed_header.map(|header| header.level_theme & 0x0F))
        .unwrap_or(0)
    }

    pub fn block_grid(&self) -> Option<&terrain::BlockGrid> {
        self.block_grid.as_ref()
    }

    /// Returns the current VDP state for non-gameplay screen rendering.
    /// `Some` during Sega screen, intro sequences, and title screen;
    /// `None` during gameplay where the pre-extracted asset pipeline renders.
    pub fn vdp_state(&self) -> Option<&vdp::VdpState> {
        self.vdp_state.as_ref()
    }

    pub fn step_frame(&mut self, input: FrameInput) -> NativeTraceFrame {
        self.step_frame_with_mode_level_raster_scheduler(input, &mut |_| None)
    }

    /// Advance one native frame while allowing the execution shell to advance
    /// the live gameplay raster owner.
    ///
    /// Core classifies the live ROM source path first. The callback completes
    /// asset-bound inputs and advances its persistent CPU/VDP/GEMS timeline;
    /// During ordinary `Mode_Level`, the returned seed is consumed by the
    /// existing `sub_44DC` schedulers. Synchronous `Kid_Transform sub_8E0`
    /// VBlanks invoke the callback as a heartbeat and ignore that seed.
    pub fn step_frame_with_mode_level_raster_scheduler(
        &mut self,
        input: FrameInput,
        scheduler: &mut dyn FnMut(
            ModeLevelVblankWorkBeforeSubE12,
        ) -> Option<vdp::ModeLevelRasterSeedSubE12>,
    ) -> NativeTraceFrame {
        let transform_hibernating_at_frame_start =
            self.object_pass_hibernating_for_transform_loc_7e90();
        self.pending_demo_timer_tick_result_loc_be82 = None;
        self.hud_dma_events_sub_44dc.clear();
        self.hud_raster_upload_timings_sub_44dc.clear();
        self.mode_level_cpu_raster_cursor_sub_44dc = None;
        self.mode_level_vblank_work_before_sub_e12 = None;
        self.mode_level_hud_raw_cycles_sub_44dc = None;
        self.mode_level_work_before_hud_sub_44dc = None;
        self.mode_level_work_after_hud_sub_44dc = None;
        self.mode_level_platform_loader_work_loc_2744 = None;
        self.demo_read_joypad_work_loc_c22 = None;
        self.mode_level_tail_work_loc_892 = None;
        self.mode_level_palette_dma_sub_970 = None;
        // ASM loc_12098/loc_12110/loc_12188 DMA requests are emitted by one
        // Execute_Objects pass. Do not retain them across frames whose object
        // pass skips the shore worker, including transformation hibernation.
        self.island_shore_dma_uploads_loc_1202a.clear();
        if self.opening_slice.is_some()
            && matches!(
                self.boot_state,
                BootState::InGame | BootState::DemoPlay { .. }
            )
        {
            // ASM Mode_Level transfers scroll before Execute_Objects and
            // ReadJoypad, so a later same-frame mode switch must retain this
            // camera rather than the post-update WRAM camera.
            self.gameplay_vdp_camera_latch_loc_300ba = Some(self.current_camera_state());
            // ASM Mode_Level $7CC runs before Execute_Objects can wake
            // sub_73D0 and clear PaletteToDMA_Flag.
            self.mode_level_palette_dma_sub_970 = Some(!self.palette_dma_suppressed_f890);
        }
        // ASM Mode_Level loc_80C: this trace-only count represents Objects
        // linked by this frame's trailing loc_3691E pass before their next
        // Execute_Objects loc_E1E initialiser. It must not leak into title or
        // other non-gameplay frames that skip `refresh_active_roster`.
        self.pending_enemy_gfx_initialisations_loc_3691e = 0;
        let title_card_live_handoff_active_at_frame_start = self.title_card_live_handoff_frames > 0;
        let title_card_load_ingame_hibernated_at_frame_start =
            title_card_live_handoff_active_at_frame_start
                && self.cold_intro_sequence_1_early_start_poll
                && self.direct_title_card_handoff_capture
                && TITLE_CARD_LIVE_HANDOFF_FRAMES_LOC_1AE2E
                    .saturating_sub(self.title_card_live_handoff_frames)
                    < TITLE_CARD_LOAD_INGAME_HIBERNATE_FRAMES_LOC_1AE2E;
        if title_card_live_handoff_active_at_frame_start {
            self.title_card_live_handoff_frames =
                self.title_card_live_handoff_frames.saturating_sub(1);
        }

        let raw_input = input;
        self.trace_input = input;
        self.previous_kid_frame_post_motion_mutator = self.last_kid_frame_post_motion_mutator;
        self.last_kid_frame_motion_branch = KidFrameMotionBranch::None;
        self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::None;
        self.last_platform_collision_scratch_loc_257e = PlatformCollisionScratchLoc257E::default();
        self.shooter_barrel_damage_latched_loc_1f52 = false;
        self.last_kid_collision_type = 0;
        self.last_kid_frame_pre_physics_start = None;
        self.last_kid_frame_pre_physics_before_physics = None;
        self.last_kid_frame_pre_physics_mutators.clear();
        self.last_kid_frame_collision_handoff_probe = None;
        self.last_kid_frame_same_frame_stomp_probe = None;
        self.last_kid_frame_iron_knight_attach_probe = None;
        self.last_kid_frame_kid_flip_branch_probe = None;
        self.last_kid_frame_kid_flip_follow_on_probe = None;
        self.last_kid_frame_jump_gravity_probe = None;
        self.last_kid_frame_grounded_release_probe = None;
        self.last_drips_splash_reseed_probe = None;
        self.last_juggernaut_companion_step_sub_975c = None;
        self.weather.storm_scroll_transfer_loc_30110 = None;
        self.wall_cling_boundary_trace_y_velocity = self
            .wall_cling_boundary_trace_y_velocity
            .and_then(|(y_velocity, frames)| {
                frames
                    .checked_sub(1)
                    .filter(|remaining| *remaining > 0)
                    .map(|remaining| (y_velocity, remaining))
            });
        let replay_demo_1_timeout_handoff_frame = matches!(
            self.boot_state,
            BootState::DemoPlay {
                use_demo_2: false,
                ..
            }
        ) && self.demo_auto_timeout_return_pending
            && self.demo_buffered_input.is_some();
        if replay_demo_1_timeout_handoff_frame {
            self.demo_auto_timeout_return_pending = false;
        } else if self.demo_auto_timeout_return_pending {
            self.demo_auto_timeout_return_pending = false;
            self.finish_demo_auto_timeout_return(DemoAutoTimeoutMode0Entry::Fresh, None);
        }
        let demo_mode_active = matches!(self.boot_state, BootState::DemoPlay { .. });
        let Some(input) = self.step_demo_input(input, replay_demo_1_timeout_handoff_frame) else {
            self.trace_input = raw_input;
            return self.trace_frame();
        };
        let demo_return_render_snapshot_at_frame_start = (replay_demo_1_timeout_handoff_frame
            || self.demo_title_return_after_frame)
            .then(|| ResidentGameplayRenderSnapshot {
                trace: self.trace_frame(),
                block_grid: self.block_grid.clone(),
            });
        let freeze_demo_auto_timeout_snapshot = self.demo_auto_timeout_freeze_gameplay_frame;
        let input = self.readjoypad_input_with_options_loc_bb6(input, demo_mode_active);
        self.trace_input = input;
        self.maybe_begin_pending_level_skip();
        self.maybe_begin_pending_level_skip_resume();
        self.maybe_clear_time_frames_for_level_skip_resume_loc_6551();
        self.maybe_finish_level_skip_resume();
        // ASM loc_C4E (kid.asm:1404): ReadJoypad edge detection —
        // `eor.b d1,d0; and.b (Ctrl_1).w,d0; or.b d0,(Ctrl_1_Pressed).w`
        // computes newly-pressed bits, then copies held state:
        // `move.b (Ctrl_1).w,(Ctrl_1_Held).w`. Native equivalent: compare
        // current `input` fields against `self.previous_input` for rising
        // edges, then store `self.previous_input = input` at frame end.
        let start_pressed = input.start && !self.previous_input.start;
        let apply_pending_pause_toggle = self.pause_toggle_pending;
        self.pause_toggle_pending = false;
        self.step_pause_restart_round_shell_loc_6b46();
        let pause_last_life_exit_frame = self.step_pause_last_life_give_up_loc_6c0a();
        // Two distinct ASM pause routines (loc_6b46 restart-round, loc_6c0a
        // last-life give-up) both suppress the toggle; kept as separate arms to
        // document that each ROM mechanism independently gates it.
        #[allow(clippy::if_same_then_else)]
        let normal_pause_continue_frame = if self.pending_pause_restart_round.is_some() {
            self.pause_toggle_pending = false;
            false
        } else if self.pending_pause_last_life_give_up {
            self.pause_toggle_pending = false;
            false
        } else if self.paused {
            self.apply_pause_menu_frame_loc_6ae0(input, start_pressed)
        } else {
            self.handle_start_press(input, start_pressed);
            false
        };
        let sega_screen_first_object_pass = matches!(
            self.boot_state,
            BootState::SegaScreen { elapsed }
                if elapsed == SEGA_SCREEN_FIRST_OBJECT_PASS_ELAPSED
        );
        let intro_sequence_rng_long_burn_active = match self.boot_state {
            BootState::IntroSequence {
                segment: 0,
                elapsed,
                ..
            } => {
                // ASM loc_C0A -> loc_6E2 hands the timeout restart into
                // IntroSequence1 with its first Mode_Standard object pass on
                // the VBlank that advances local elapsed 43 -> 44. Cold boot
                // enters the same loc_E12 schedule from pre-step elapsed 44.
                let first_live_elapsed = if self
                    .demo_auto_timeout_intro_sequence_1_frames_override
                    .is_some()
                {
                    43
                } else if self.cold_intro_sequence_1_early_start_poll
                    && self.direct_title_card_handoff_capture
                {
                    // ASM Load_IntroSequence1 -> sub_1B8F4 runs its first
                    // post-loader Mode_Standard object pass on cold frame 69.
                    // With the frame-34 native mode entry this is pre-step
                    // elapsed 34.
                    34
                } else {
                    44
                };
                elapsed >= first_live_elapsed
            }
            BootState::IntroSequence {
                segment: 1,
                elapsed,
                ..
            } if self
                .demo_auto_timeout_intro_sequence_2_frames_override
                .is_some() =>
            {
                // ASM loc_1B9D0 -> sub_1C204 completes the hidden loader/fade
                // before loc_1BA02 owns a live outer object pass on the
                // timeout-restart lineage.
                elapsed >= DEMO_AUTO_TIMEOUT_INTRO_SECOND_RNG_FIRST_LIVE_ELAPSED
            }
            BootState::IntroSequence {
                segment: 2,
                elapsed,
                ..
            } if self
                .demo_auto_timeout_intro_sequence_3_frames_override
                .is_some()
                || self.timeout_restart_title_intro_sequence_3_return_pending =>
            {
                // ASM sub_1B93E -> sub_1C204 completes the hidden loader/fade
                // before loc_1B96E owns the live outer object-pass window.
                // sub_1C246 then consumes Start without ending that pass, so
                // keep both deferred loc_1C264 hand-off frames live after the
                // timeout duration override has been cleared.
                elapsed >= DEMO_AUTO_TIMEOUT_INTRO_THIRD_RNG_FIRST_LIVE_ELAPSED
            }
            BootState::IntroSequence {
                segment: 3,
                elapsed,
                ..
            } if self
                .demo_auto_timeout_intro_sequence_4_frames_override
                .is_some() =>
            {
                // ASM Load_IntroSequence4 leaves sub_1B988 hibernating until
                // its worker wakes inside a fresh Mode_Standard outer pass.
                // Execute_Objects loc_E12 burns before sub_1B988 writes
                // Camera_Y_pos=4, then remains live through loc_1B9B6.
                elapsed >= DEMO_AUTO_TIMEOUT_INTRO_FOURTH_RNG_FIRST_LIVE_ELAPSED
            }
            BootState::IntroSequence {
                segment: 4,
                elapsed,
                ..
            } if self
                .demo_auto_timeout_intro_sequence_5_frames_override
                .is_some() =>
            {
                // ASM loc_1BA28's hibernating workers wake inside a fresh
                // Mode_Standard outer pass. Execute_Objects loc_E12 burns
                // before loc_1BA60 polls Start, including both deferred
                // sub_1C246 -> loc_1C264 title hand-off frames.
                // ASM Load_IntroSequence5 allocates loc_1BCD0 in the same
                // outer pass that wakes loc_1BA28's workers.
                let workers_wake_one_frame_early =
                    self.demo_timeout_restart_uses_intro_return_slot_2_early_visibility;
                let first_live_elapsed = if workers_wake_one_frame_early {
                    DEMO_AUTO_TIMEOUT_INTRO_FIFTH_START_SKIP_RNG_FIRST_LIVE_ELAPSED
                } else {
                    DEMO_AUTO_TIMEOUT_INTRO_FIFTH_RNG_FIRST_LIVE_ELAPSED
                };
                let panel_upload_stall_elapsed = if workers_wake_one_frame_early {
                    INTRO_SEQUENCE_5_START_SKIP_PANEL_UPLOAD_STALL_PRE_STEP_ELAPSED
                } else {
                    INTRO_SEQUENCE_5_PANEL_UPLOAD_STALL_PRE_STEP_ELAPSED
                };
                elapsed >= first_live_elapsed && elapsed != panel_upload_stall_elapsed
            }
            BootState::IntroSequence6 {
                carry_timeout_restart_hidden_camera_wrap,
                ..
            } => {
                let displayed_local_frame = self.title_screen_elapsed_frames.saturating_add(1);
                // ASM Load_DemoPlay toggles `$FFFFFBC8`; on the later demo-2
                // revisit the cleared-camera lineage and false toggle identify
                // the three-frame-delayed Load_IntroSequence6 worker shell.
                let later_worker_delay = if self.intro_sequence_6_entered_from_timeout_restart
                    && !carry_timeout_restart_hidden_camera_wrap
                    && !self.demo_toggle
                {
                    TIMEOUT_RESTART_INTRO_SEQUENCE_6_LATER_WORKER_DELAY_FRAMES
                } else {
                    0
                };
                let worker_local_frame = displayed_local_frame.saturating_sub(later_worker_delay);
                // ASM loc_1BA5C writes Game_Mode=$28 before loc_1BA90's
                // `$100,$E0,...,$00` fade resumes have finished. Each resume
                // is still inside the old Mode_Standard -> Execute_Objects
                // loc_E12 pass; loc_1BAAC only enters Load_IntroSequence6
                // after the ninth draw. Once the loader allocates loc_1BAB0,
                // loc_1B37C, and sub_1B43A, the outer passes resume except
                // while loc_1B45A's palette/title uploads cross VBlanks. The
                // later early-title-restore lineage exposes those three new
                // workers three frames later, so shift their whole schedule
                // without shifting loc_1BA90's already-running fade passes.
                displayed_local_frame <= INTRO_SEQUENCE_6_ENTRY_FADE_OUTER_PASSES_LOC_1BA90
                    || (worker_local_frame >= INTRO_SEQUENCE_6_POST_LOADER_RNG_FIRST_LIVE_FRAME
                        && worker_local_frame
                            != INTRO_SEQUENCE_6_PALETTE_BLACKOUT_STALL_FRAME_LOC_1B470
                        && !(INTRO_SEQUENCE_6_TITLE_ART_STALL_FIRST_FRAME_LOC_1B494
                            ..=INTRO_SEQUENCE_6_TITLE_ART_STALL_LAST_FRAME_LOAD_TITLE_ART)
                            .contains(&worker_local_frame))
            }
            _ => false,
        };
        let frame_started_in_title_screen =
            matches!(self.boot_state, BootState::TitleScreen { .. });
        let demo_auto_timeout_mode_0_local_frame =
            self.title_screen_elapsed_frames.saturating_add(1);
        let demo_auto_timeout_mode_0_rng_long_burn_active = frame_started_in_title_screen
            && self.demo_auto_timeout_handoff_active
            && DEMO_AUTO_TIMEOUT_MODE_0_RNG_LONG_FRAMES
                .contains(&demo_auto_timeout_mode_0_local_frame)
            && !(demo_auto_timeout_mode_0_local_frame == 1
                && self.demo_auto_timeout_mode_0_entry
                    == DemoAutoTimeoutMode0Entry::ReadJoypadReplay);
        let frame_started_in_demo_auto_timeout_handoff =
            frame_started_in_title_screen && self.demo_auto_timeout_handoff_active;
        let frame_started_in_level_entry =
            matches!(self.boot_state, BootState::StartingLevel { .. });
        // ASM loc_74E0 -> loc_6E2/Load_TitleCard: the debug level-skip shell
        // runs two early Mode_Standard object passes, hibernates through the
        // loader freeze (the ROM loader blocks the frame loop, so controller
        // RNG stops), then runs hidden Mode_Level from the measured per-level
        // load completion (#442, oracle-settle-shell.yaml). The first live
        // pass only burns Execute_Objects' RNG longword; Manage_EnemyLoading
        // exposes enemies on the following tick.
        let debug_level_entry_elapsed = match self.boot_state {
            BootState::StartingLevel { elapsed_frames, .. }
                if self.debug_level_entry_suppresses_title_card_live_handoff =>
            {
                Some(elapsed_frames)
            }
            _ => None,
        };
        let debug_level_entry_live_boundary = self.debug_level_entry_live_boundary_loc_74e0();
        let debug_level_entry_object_pass = debug_level_entry_elapsed
            .is_some_and(|elapsed| !(2..debug_level_entry_live_boundary).contains(&elapsed));
        let debug_level_entry_hibernated = debug_level_entry_elapsed
            .is_some_and(|elapsed| elapsed < debug_level_entry_live_boundary);
        let debug_level_entry_live_object_pass = debug_level_entry_elapsed
            .is_some_and(|elapsed| elapsed >= debug_level_entry_live_boundary);
        let level_entry_hibernated_gameplay_execution = self
            .starting_level_title_card_bootstrap_active_loc_1ae2e()
            || debug_level_entry_hibernated;
        // ASM loc_1AE2E owns Ctrl_Pressed/Ctrl_Held for every title-card
        // entry, even on routes whose hidden Mode_Level execution continues
        // underneath the shell.
        let level_entry_hibernated_gameplay_input = frame_started_in_level_entry;
        self.step_boot_state(input, start_pressed);
        self.update_vdp_state();
        if sega_screen_first_object_pass {
            // ASM Mode_Standard -> Execute_Objects loc_E12 (kid.asm:854,
            // 1636) burns one `Get_RandomNumber_long` before dispatching the
            // loc_12E64 logo controller. Its loc_12E6C/loc_12E96 VBlank loops
            // remain inside that same object pass, so a later Start edge or
            // natural timeout must not defer or repeat the draw.
            let _ = self.rom_random.next_long();
        }
        if intro_sequence_rng_long_burn_active {
            // ASM Mode_Standard -> Execute_Objects loc_E12 (kid.asm:854,
            // 1636): the live IntroSequence1 object pass begins on local
            // elapsed frame 44 and continues through the deferred Start
            // hand-off into `Game_Mode=$002C`.
            let _ = self.rom_random.next_long();
        }
        if frame_started_in_title_screen
            && if frame_started_in_demo_auto_timeout_handoff {
                demo_auto_timeout_mode_0_rng_long_burn_active
            } else {
                self.title_screen_rng_long_burn_active()
            }
        {
            // ASM TitleScreen still runs through Mode_Standard
            // (kid.asm:854-894), but oracle keeps `RNG_Offset` frozen through
            // the hidden title rebuild window after the early `$002C`
            // hand-off. Only burn `Get_RandomNumber_long` outside that
            // oracle-backed freeze range.
            let _ = self.rom_random.next_long();
        }
        self.step_allow_pausing_fade_sub_73d0();
        if debug_level_entry_object_pass {
            // ASM Execute_Objects loc_E12: both early title-card object passes
            // and every hidden Mode_Level pass begin with one RNG longword.
            let _ = self.rom_random.next_long();
        } else if debug_level_entry_elapsed.is_none()
            && frame_started_in_level_entry
            && let BootState::StartingLevel {
                elapsed_frames,
                uses_load_ingame_bootstrap,
                ..
            } = self.boot_state
        {
            let title_first_visible_frame = if matches!(
                self.title_screen_intro_return_lineage,
                Some(TitleScreenIntroReturnLineage::ColdIntroSequence1)
            ) && self.cold_intro_sequence_1_early_start_poll
                && self.direct_title_card_handoff_capture
            {
                crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME.saturating_sub(8)
            } else {
                crate::movement::LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME
            };
            let cold_boot_titlecard_long_burn =
                    // Oracle WRAM watches keep the cold-boot mode-$0008 shell
                    // RNG-idle through the hidden carry window, then expose a
                    // single pre-visible `Execute_Objects` burn on the frame
                    // immediately before the loc_1AB26 / loc_1AD48 seed
                    // batches wake.
                    elapsed_frames == title_first_visible_frame.saturating_sub(3)
                        // The burn resumes on the first visible frame when
                        // Execute_Objects first runs the loc_1AB26/loc_1AD48
                        // workers, and continues every frame after.
                        || elapsed_frames >= title_first_visible_frame;
            if uses_load_ingame_bootstrap || cold_boot_titlecard_long_burn {
                // ASM loc_6E2 -> Load_TitleCard keeps the cold-boot
                // mode-$0008 title-card shell dormant through an early
                // hidden pre-roll. The burn resumes once the
                // loc_1AB26/loc_1AD48 workers are live within
                // Execute_Objects — frame 43 (letter batch) and 44
                // (deferred number) both burn, matching oracle delta of
                // 38 bytes (4 long + 34 worker bytes).
                let _ = self.rom_random.next_long();
            }
        }
        if let BootState::StartingLevel {
            elapsed_frames,
            uses_load_ingame_bootstrap,
            load_ingame_bootstrap_started,
            ..
        } = self.boot_state
        {
            if uses_load_ingame_bootstrap
                && !load_ingame_bootstrap_started
                // ASM Obj_TitleMenu Title_InputLoop/loc_1BB52 still owns the
                // carried early-start shell for this window. Keep a separate
                // loc_1AC70 seed stream that only begins burning RNG once the
                // later Load_TitleCard worker lineage is active.
                && elapsed_frames >= EARLY_START_TITLE_MENU_SHELL_CARRY_FRAMES
                && let Some(title_rng) = self.starting_level_title_rng.as_mut()
            {
                let _ = title_rng.next_long();
            }
            if !self.debug_level_entry_suppresses_title_card_live_handoff {
                self.maybe_seed_starting_level_title_workers(elapsed_frames);
            }
        }
        if let BootState::DemoPlay { use_demo_2, .. } = self.boot_state
            && self.demo_gameplay_active()
            && self.opening_slice.is_none()
        {
            self.begin_demo_gameplay(NativeRuntime::demo_level_id(use_demo_2));
        }

        let gameplay_slice_active = (matches!(
            self.boot_state,
            BootState::StartingLevel { .. } | BootState::InGame
        ) && !level_entry_hibernated_gameplay_execution
            && !title_card_load_ingame_hibernated_at_frame_start)
            || (self.demo_gameplay_active() && self.opening_slice.is_some());
        let demo_hud_timer_tail_active = match self.boot_state {
            BootState::DemoPlay {
                frame_counter,
                use_demo_2,
                ..
            } => {
                let start_frame = if use_demo_2 {
                    demo::hud_timer_tail_start_frame(true)
                } else {
                    self.demo_gameplay_start_frame(false).saturating_add(1)
                };
                frame_counter >= start_frame
            }
            _ => false,
        };
        if demo_hud_timer_tail_active
            && gameplay_slice_active
            && !transform_hibernating_at_frame_start
        {
            // ASM Mode_Level calls Make_SpriteAttr_HUD before Execute_Objects.
            // Demo bootstrap and other Kid hibernations can keep native
            // movement out of resolve_frame_hazards even though this outer HUD
            // pass still decrements Time_SubSeconds at loc_BE82.
            self.pending_demo_timer_tick_result_loc_be82 =
                Some(self.tick_level_timer_hud_loc_be82());
        }
        let demo_hud_presentation_active = match self.boot_state {
            BootState::DemoPlay { use_demo_2, .. } => use_demo_2 || demo_hud_timer_tail_active,
            _ => false,
        };
        if demo_hud_presentation_active
            && self.demo_gameplay_active()
            && gameplay_slice_active
            && self.level_start_frame != Some(self.frame_index)
            && !transform_hibernating_at_frame_start
        {
            self.tick_demo_hud_timer_presentation_loc_be82();
        }
        if gameplay_slice_active {
            // ASM Mode_Level loc_7E2/Transfer_ScrollDataToVRAM loc_30110 runs
            // before Execute_Objects and the later `sub_6034` weather worker.
            self.transfer_scroll_data_storm_loc_30110();
        }
        let preserve_demo_title_snapshot_frame =
            matches!(self.boot_state, BootState::TitleScreen { .. })
                && self.preserve_demo_title_snapshot;
        let hundred_k_trip_teleport_started = !freeze_demo_auto_timeout_snapshot
            && !preserve_demo_title_snapshot_frame
            && self.step_hundred_k_trip_timer_loc_7452();
        let hundred_k_trip_transition_active =
            matches!(self.boot_state, BootState::HundredKTripTransition { .. });
        if freeze_demo_auto_timeout_snapshot && gameplay_slice_active {
            // ASM Mode_Level -> Execute_Objects loc_E12 draws before ReadJoypad
            // loc_C0A observes the terminal demo counter. Native keeps the
            // prior trace-facing physics snapshot on this scheduling boundary,
            // but the RNG side effect from that final object pass still lands.
            let _ = self.rom_random.next_long();
        }
        if !freeze_demo_auto_timeout_snapshot
            && !preserve_demo_title_snapshot_frame
            && !hundred_k_trip_teleport_started
            && !hundred_k_trip_transition_active
        {
            if gameplay_slice_active
                && !self.paused
                && self.pending_pause_restart_round.is_none()
                && !pause_last_life_exit_frame
                && (matches!(self.boot_state, BootState::InGame)
                    || (self.demo_gameplay_active() && self.opening_slice.is_some()))
                && !debug_level_entry_object_pass
            {
                let vblank_work = self.mode_level_vblank_work_before_sub_e12();
                self.mode_level_vblank_work_before_sub_e12 = Some(vblank_work);
                if let Some(seed) = scheduler(vblank_work) {
                    self.mode_level_raster_seed_sub_e12 = Some(seed);
                }
                if self.progression.form_change_flash_frames <= 1 {
                    self.mode_level_work_before_hud_sub_44dc =
                        Some(self.observe_mode_level_work_before_hud_sub_44dc());
                    // ASM Execute_Objects loc_E12 (kid.asm:1636) calls
                    // Get_RandomNumber_long before walking any object coroutine.
                    // Kid_Transform's sub_8E0 loop owns intervening VBlanks;
                    // the final cleanup count resumes this suspended object
                    // pass when the count reaches one.
                    let _ = self.rom_random.next_long();
                }
            }
            // ASM Mode_Level loc_7F4 builds the HUD before Execute_Objects.
            // Preserve the count encoded into this frame's Sprite_Table;
            // Increase_Diamonds may increment it later in the object pass.
            self.hud_diamond_count_loc_bdc0 = self.progression.diamond_count;
            if debug_level_entry_live_object_pass {
                // ASM Mode_Level Execute_Objects: the Kid collision consumer
                // precedes enemy workers; loc_BC5C's damage-flash worker also
                // remains live beneath the debug-warp title window (#427/#442).
                self.recent_damage_gate_held_this_frame_loc_bc6e = self.just_received_damage;
                self.step_damage_flash_thread_loc_bc5c();
                self.consume_hidden_shell_player_contact_damage_loc_b700();
            }
            self.refresh_active_roster();
            if debug_level_entry_live_object_pass {
                // ASM GfxObjects_CollisionKid runs after Execute_Objects and
                // leaves contact for the following hidden pass to consume.
                self.latch_hidden_shell_player_contact_damage_loc_1cf2();
            }
            // ASM Mode_Level loc_7396/loc_73B8 advances the murder wall every
            // frame, including the hidden debug-warp shell frames behind the
            // title window (#442). The visible Kid frame owns the wall while
            // InGame; the hidden pass runs no Kid frame, so step it here —
            // 0x08's wall reaches frame 0 aged by the whole live shell, not
            // just the visible window. The wall worker (sub_7196 allocates
            // loc_7452's camera/wall coroutine during Load_InGame) runs from
            // the same live boundary as the object passes. Measured: the
            // wall's 128·N(N+1) acceleration ramp fits the oracle's
            // level_settle checkpoint exactly at N=162 ticks on the sweep's
            // sampling (sweep run 1786646093 pinned the off-by-one both ways).
            if self.debug_level_entry_runs_live_shell_loc_74e0
                && debug_level_entry_elapsed
                    .is_some_and(|elapsed| elapsed >= debug_level_entry_live_boundary)
                && let Some(mut slice) = self.opening_slice
            {
                let mut camera_x = slice.camera.x_pos;
                self.step_murder_wall(&mut camera_x);
                slice.camera.x_pos = camera_x;
                self.opening_slice = Some(slice);
            }
            self.step_pending_prize_diamond_awards();
            self.step_pending_sound_effects();
            self.step_pending_dragon_death_children();
            // ASM loc_892 (kid.asm:956): Mode_Level rendering tail join point.
            // When $FFFFFB49 (paused), skip j_DiamondPower_CompileSprites and
            // Make_SpritesFromPlatforms but still run j_sub_30194 (HUD sprites),
            // j_loc_3038A (scroll data transfer / Time_Frames), sub_14C0
            // (palette fade). All three tail calls are visual/VDP — not modelled.
            // Diamond Power uses raw input (Start+A held) since gameplay_input delays A by 1 frame
            if matches!(self.player_lifecycle, PlayerLifecycle::Alive)
                && gameplay_slice_active
                && !self.paused
                && self.pending_pause_restart_round.is_none()
                && !pause_last_life_exit_frame
            {
                // ASM Mode_Level calls DiamondPower_Run before ReadJoypad.
                // In native's post-step trace indexing, the ROM `Ctrl_Held`
                // value visible to DiamondPower_Run is the two-frame
                // `prior_input` latch; `previous_input` is already the value
                // movement sees for this traced frame.
                self.step_diamond_power(self.prior_input);
            }
            if gameplay_slice_active
                && !self.paused
                && self.pending_pause_restart_round.is_none()
                && !pause_last_life_exit_frame
            {
                // ASM loc_7606 / loc_8C26 call sub_7428 after DiamondPower_Run
                // but before the next frame consumes FiveWayShotReady. Use raw
                // Ctrl_Held here; `gameplay_input` delays A for movement.
                self.step_juggernaut_five_way_shot_ready_sub_7428(input);
                // ASM loc_1AE2E owns Game_Mode=$0008 until Start writes
                // Game_Mode=$000C and jumps through loc_6E2. Gameplay input and
                // Mode_Level execution both stay hibernated during that shell.
                // Preserve the frame-start title-card ownership across the
                // loc_1AE2E mode write: the Start edge may expose Mode_Level
                // now, but the same frame's gameplay pass still receives no
                // B/direction input.
                let title_card_hibernates_gameplay_input = level_entry_hibernated_gameplay_input;
                let demo_live_input = matches!(self.boot_state, BootState::DemoPlay { .. })
                    && self
                        .demo_startup_frame()
                        .is_some_and(|frame| frame > demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME);
                let gameplay_input = if title_card_hibernates_gameplay_input {
                    FrameInput::default()
                } else if demo_live_input {
                    input
                } else if normal_pause_continue_frame {
                    // ASM Game_Paused_Loop (kid.asm:6829) calls ReadJoypad
                    // before loc_6B68 clears Game_Paused, so the resumed
                    // Mode_Level pass consumes the Start-only Ctrl_Held latch.
                    self.gameplay_input_from_latch(input, input)
                } else {
                    self.gameplay_input(input)
                };
                let jump_pressed = if title_card_hibernates_gameplay_input {
                    false
                } else if demo_live_input {
                    input.b && !self.previous_input.b
                } else {
                    self.gameplay_jump_pressed()
                };
                let demo_use_demo_2 = self.demo_uses_second_recording();
                let demo_gameplay_start = demo_use_demo_2
                    .map_or(demo::DEMO_GAMEPLAY_START_FRAME, |use_demo_2| {
                        self.demo_gameplay_start_frame(use_demo_2)
                    });
                let demo_special_kid_path = match self.demo_startup_frame() {
                    Some(frame)
                        if demo_use_demo_2 == Some(false)
                            && (demo_gameplay_start..=demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME)
                                .contains(&frame) =>
                    {
                        self.last_kid_frame_motion_branch =
                            KidFrameMotionBranch::DemoBootstrapPoseHold;
                        self.apply_demo_1_bootstrap_pose_hold();
                        true
                    }
                    Some(frame) if frame == demo_gameplay_start => {
                        self.last_kid_frame_motion_branch =
                            KidFrameMotionBranch::DemoBootstrapCameraHandoff;
                        self.apply_demo_bootstrap_camera_handoff();
                        true
                    }
                    Some(frame) if frame == demo::DEMO_BOOTSTRAP_INIT_FRAME => {
                        self.last_kid_frame_motion_branch =
                            KidFrameMotionBranch::DemoBootstrapInitialSettle;
                        self.apply_demo_bootstrap_initial_settle();
                        true
                    }
                    Some(frame)
                        if frame > demo::DEMO_BOOTSTRAP_INIT_FRAME
                            && frame < demo::DEMO_BOOTSTRAP_GROUNDED_FRAME =>
                    {
                        self.last_kid_frame_motion_branch =
                            KidFrameMotionBranch::DemoBootstrapFreefall;
                        self.apply_demo_bootstrap_freefall(frame);
                        true
                    }
                    Some(frame)
                        if (demo::DEMO_BOOTSTRAP_GROUNDED_FRAME
                            ..=demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME)
                            .contains(&frame) =>
                    {
                        self.last_kid_frame_motion_branch =
                            KidFrameMotionBranch::DemoBootstrapGroundedHold;
                        self.apply_demo_bootstrap_grounded_hold();
                        true
                    }
                    _ => {
                        if self.try_apply_demo_bootstrap_grounded_run(gameplay_input, jump_pressed)
                        {
                            self.last_kid_frame_motion_branch =
                                KidFrameMotionBranch::DemoBootstrapGroundedRun;
                            true
                        } else {
                            self.seed_demo_1_late_jump_reentry();
                            self.restore_demo_1_late_fall_velocity();
                            self.refresh_demo_grounded_run_surface_type();
                            self.apply_evanescent_sprite_only_overlays_before_kid_loc_EDB0();
                            // ASM Execute_Objects order lets the Skull Mountain 3
                            // ghost-block worker (`sub_F730`) update Level_Layout
                            // before Kid's `sub_902A` ceiling probes consume it.
                            // Kid_Transform loc_7E90 waits in sub_8E0 instead
                            // of returning to Mode_Level loc_7FC. The entry
                            // frame has already run sub_F730; later host frames
                            // must retain its timer until the outer pass resumes.
                            if !transform_hibernating_at_frame_start {
                                self.step_ghost_blocks();
                            }
                            let transform_starts_this_frame =
                                self.progression.form_change_flash_frames == 0;
                            self.refresh_opening_slice(gameplay_input, jump_pressed);
                            if transform_starts_this_frame
                                && self.progression.form_change_flash_frames > 0
                            {
                                // ASM Execute_Objects loc_E1E suspends the object
                                // pass inside Kid_Transform loc_7E90. Native has
                                // already advanced independent Gfx animations,
                                // so restore the walking Dragon timer observed at
                                // that scheduler boundary while retaining movement.
                                self.rewind_walking_dragon_animation_on_transform_entry_loc_e1e();
                                self.transform_resident_hud_diamond_vram_page_loc_5d4a = Some(
                                    self.gameplay_hud_vram_state
                                        .diamond_frame_table_byte_offset_loc_5d4a(),
                                );
                                // ASM Mode_Level loc_80C dispatches loc_12098
                                // before Kid_Transform loc_7E90 hibernates the
                                // Kid path. A transformation begun during the
                                // refresh must retain that final object tick.
                                self.transform_resident_island_shore_vram_frames_loc_1202a =
                                    self.island_shore_cycle_loc_1202a.map(|state| {
                                        (
                                            state.primary_vram_frame,
                                            state.mid_vram_frame,
                                            state.lower_vram_frame,
                                        )
                                    });
                                self.step_island_shore_cycle_loc_1202a();
                            }
                            self.repair_timeout_restart_demo_1_late_jump_reentry();
                            false
                        }
                    }
                };
                if demo_special_kid_path && self.time_frames_loc_3038a != 0 {
                    // Mode_Level loc_7FC calls sub_F730 before Execute_Objects
                    // reaches the hibernating demo-bootstrap Kid coroutine.
                    // The bootstrap substitutes Kid motion only; independent
                    // SolidGhost workers still age on every outer level pass.
                    self.step_ghost_blocks();
                    // ASM Execute_Objects loc_E12 schedules the independent
                    // loc_1202A Island object while the demo bootstrap's Kid
                    // path is hibernating outside refresh_opening_slice.
                    self.step_island_shore_cycle_loc_1202a();
                }
                // ASM Execute_Objects sub_E12 has now resumed the Kid
                // coroutine. Complete the entry-time Mode_Level ledger with
                // the source branches actually taken this frame, before
                // loc_80C continues to enemy loading and sub_44DC.
                self.complete_mode_level_player_execution_path_before_hud_sub_44dc();
                self.manage_enemy_loading_after_execute_objects_loc_80c();
                self.load_debug_level_entry_enemies_after_player_loc_80c();
            }
        } else {
            self.last_kid_frame_motion_branch = KidFrameMotionBranch::DemoAutoTimeoutFrozen;
        }
        if apply_pending_pause_toggle {
            self.paused = !self.paused;
            if self.paused
                && let Some((camera_x_pos, camera_y_pos)) = self.pause_menu_camera_words()
            {
                // ASM loc_690C (kid.asm:6617): pause entry uploads ArtUnc_PauseMenu
                // before the text tilemap overwrites the nametable rows.
                let _ = boot::pause_menu_art_total_bytes_loc_690C(
                    boot::PAUSE_MENU_ART_HEADER_WORD_LOC_690C,
                );
                // ASM loc_695C (kid.asm:6664): pause entry saves the five
                // nametable rows that the menu text will overwrite.
                let _ = boot::pause_menu_tile_save_plan_loc_695C(camera_x_pos, camera_y_pos);
            }
        }
        if self.demo_title_return_after_frame {
            self.demo_title_return_after_frame = false;
            self.finish_demo_title_return(raw_input, demo_return_render_snapshot_at_frame_start);
        } else if replay_demo_1_timeout_handoff_frame {
            self.finish_demo_auto_timeout_return(
                DemoAutoTimeoutMode0Entry::ReadJoypadReplay,
                demo_return_render_snapshot_at_frame_start,
            );
        }

        let transform_cleanup_resumed_mode_level = transform_hibernating_at_frame_start
            && !self.object_pass_hibernating_for_transform_loc_7e90();
        if transform_cleanup_resumed_mode_level {
            self.manage_platform_loading_after_transform_cleanup_loc_7e90();
            if let BootState::DemoPlay {
                frame_counter,
                use_demo_2,
                ..
            } = self.boot_state
            {
                self.demo_read_joypad_work_loc_c22 = Some(Self::demo_read_joypad_work_loc_c22(
                    frame_counter,
                    use_demo_2,
                ));
            }
        }
        if Self::hud_animation_runs_in_mode_level_loc_80c(
            gameplay_slice_active,
            self.object_pass_hibernating_before_mode_level_tail_loc_892(),
        ) {
            // ASM Mode_Level loc_80C (kid.asm:923): `bsr.w sub_44DC` advances
            // animated HUD VRAM pages once per level frame before the rendered
            // frame is sampled. Kid_Transform loc_7CBE/loc_7D22 waits in
            // sub_8E0 instead of returning to this outer pass, preserving the
            // resident `$DD40` page through the hibernating VBlanks.
            let previous_life_page = self.hud_life_icon_vram_page_loc_4586;
            let previous_life_frame_offset = self.hud_animation_state.life_frame_offset_fb5e;
            let previous_clock_frame_offset = self.hud_animation_state.clock_frame_offset_fb5f;
            let previous_diamond_page = self
                .gameplay_hud_vram_state
                .diamond_frame_table_byte_offset_loc_5d4a();
            let step = boot::step_hud_animation_sub_44DC(self.hud_animation_state);
            self.hud_animation_state = step.next_state;
            if let Some(coin_upload) = step.coin_upload {
                // ASM sub_44DC loc_4534 (kid.asm): the coin longword writes
                // precede every later HUD-art command in this pass.
                self.hud_dma_events_sub_44dc.push(HudDmaEventSub44DC::Coin {
                    frame_table_byte_offset: coin_upload.frame_table_byte_offset,
                });
            }
            if let Some(life_upload) = step.life_upload {
                // ASM loc_4586 selects `off_457A[frame]`, then sub_44D0 copies
                // its 128-byte page to VRAM `$4620`. Keep the resident page,
                // rather than inferring it later from the looping `$FFFA5E`.
                self.hud_life_icon_vram_page_loc_4586 = life_upload.frame_table_byte_offset;
                self.hud_dma_events_sub_44dc
                    .push(HudDmaEventSub44DC::LifeIcon {
                        frame_table_byte_offset: life_upload.frame_table_byte_offset,
                        // ASM loc_455E (kid.asm): only the script cursor at 6
                        // advances to the `$FF` sentinel at 8 and loops back
                        // through loc_455E with d7=0.  The resident VRAM page
                        // cannot identify this branch because bootstrap starts
                        // with page 8 while the script cursor starts at 0.
                        wrapped_to_start: previous_life_frame_offset == 6
                            && self.hud_animation_state.life_frame_offset_fb5e == 0,
                    });
            }
            if let Some(clock_upload) = step.clock_upload {
                // ASM sub_44DC loc_45DA (kid.asm): clock art follows life art
                // and precedes the two DMA-backed HUD animations.
                self.hud_dma_events_sub_44dc
                    .push(HudDmaEventSub44DC::Clock {
                        frame_table_byte_offset: clock_upload.frame_table_byte_offset,
                        // ASM loc_45B2 (kid.asm): cursor 6 advances to the
                        // `$FF` sentinel at 8 and repeats with d7=0.
                        wrapped_to_start: previous_clock_frame_offset == 6
                            && self.hud_animation_state.clock_frame_offset_fb5f == 0,
                    });
            }
            if let Some(diamond_upload) = step.diamond_upload {
                self.gameplay_hud_vram_state
                    .record_diamond_dma_sub_5d4a(diamond_upload.frame_table_byte_offset);
                self.hud_dma_events_sub_44dc
                    .push(HudDmaEventSub44DC::Diamond {
                        frame_table_byte_offset: u8::try_from(
                            diamond_upload.frame_table_byte_offset,
                        )
                        .expect("sub_5D4A selector fits in u8"),
                    });
            }
            if let Some(flag_upload) = step.flag_upload {
                // ASM sub_44DC loc_4680 -> sub_5DA6 (kid.asm): the flag DMA is
                // the final HUD-art command before the routine returns.
                self.hud_dma_events_sub_44dc.push(HudDmaEventSub44DC::Flag {
                    frame_table_byte_offset: flag_upload.frame_table_byte_offset,
                });
            }
            // ASM loc_BF2A (kid.asm:15526): the lives HUD path clamps
            // Number_Lives to 99 during display, not when prizes/score award
            // extra lives.
            self.progression.number_lives =
                boot::hud_lives_clamp_loc_BF2A(self.progression.number_lives);
            self.mode_level_hud_raw_cycles_sub_44dc =
                mode_level_timing::hud_life_icon_only_raw_cycles_sub_44dc(
                    &self.hud_dma_events_sub_44dc,
                )
                .or_else(|| {
                    mode_level_timing::hud_diamond_only_raw_cycles_sub_44dc(
                        &self.hud_dma_events_sub_44dc,
                    )
                });

            if let (Some(seed), Some(ledger)) = (
                self.mode_level_raster_seed_sub_e12.take(),
                self.mode_level_work_before_hud_sub_44dc.as_ref(),
            ) {
                self.mode_level_cpu_raster_cursor_sub_44dc =
                    vdp::mode_level_cpu_raster_cursor_sub_44dc(seed, ledger);
                self.hud_raster_upload_timings_sub_44dc.extend(
                    self.hud_dma_events_sub_44dc
                        .iter()
                        .copied()
                        .filter_map(|command| {
                            vdp::life_icon_raster_timing_from_sub_e12(
                                seed,
                                ledger,
                                command,
                                previous_life_page,
                            )
                            .or_else(|| {
                                vdp::diamond_raster_timing_from_sub_e12(
                                    seed,
                                    ledger,
                                    command,
                                    previous_diamond_page,
                                )
                            })
                        }),
                );
            }
            self.mode_level_work_after_hud_sub_44dc =
                Some(self.observe_mode_level_work_after_hud_sub_44dc());
        }

        let demo_mode_level_tail_hibernating = matches!(
            self.boot_state,
            BootState::DemoPlay {
                frame_counter,
                use_demo_2,
                ..
            } if frame_counter < demo::mode_level_tail_start_frame(use_demo_2)
        );
        if self.game_mode().as_disassembly_value() != 0
            && !demo_mode_level_tail_hibernating
            && !self.object_pass_hibernating_before_mode_level_tail_loc_892()
        {
            // ASM Mode_Level loc_892 reaches loc_3038A only after the current
            // Execute_Objects pass returns. Kid_Transform loc_7CBE/loc_7D22
            // instead waits inside sub_8E0; Death loc_B80C likewise waits in
            // the Kid callback. Those VBlanks transfer the existing
            // scroll/HUD buffers without incrementing Time_Frames.
            // The two Load_DemoPlay paths reach this tail on their independently
            // measured startup frames. The loc_7E90 cleanup frame returns to
            // the suspended outer pass and reaches it normally.
            self.advance_mode_level_tail_loc_892();
        }
        if matches!(
            self.game_mode(),
            GameMode::LevelTransition | GameMode::InGame | GameMode::DemoPlay
        ) {
            self.step_murder_wall_warning_animation_sub_5e02();
        }
        let trace = self.trace_frame();
        if self.demo_post_transform_trace_velocity_hold_frames > 0 {
            self.demo_post_transform_trace_velocity_hold_frames -= 1;
        }
        self.demo_auto_timeout_freeze_gameplay_frame = false;
        self.prior_input = self.previous_input;
        // ASM loc_C4E (kid.asm:1410): `move.b (Ctrl_1).w,(Ctrl_1_Held).w` —
        // latch current held state for next-frame edge detection. Also
        // loc_C9C: route Ctrl_1_Held/Pressed → Ctrl_Held/Pressed (1P default).
        self.previous_input = if level_entry_hibernated_gameplay_input {
            // ASM loc_1AE2E consumes only Ctrl_Pressed Start while the
            // title-card shell owns mode $0008. Gameplay bits held on that
            // handoff frame must not become the first live Mode_Level latch.
            FrameInput {
                start: input.start,
                ..FrameInput::default()
            }
        } else {
            input
        };
        if matches!(self.boot_state, BootState::LevelSkipHold)
            && let Some(transition) = self.level_skip_transition.as_mut()
        {
            transition.elapsed_frames += 1;
        }
        self.frame_index += 1;
        trace
    }

    fn begin_kid_pre_physics_probe(&mut self, player: PlayerState) {
        self.last_kid_frame_pre_physics_start = Some(player);
        self.last_kid_frame_pre_physics_before_physics = Some(player);
        self.last_kid_frame_pre_physics_mutators.clear();
    }

    fn begin_kid_collision_handoff_probe(&mut self, slice: &OpeningSliceState) {
        let player = slice.player;
        // ROM loc_AA22 runs Character_CheckCollision immediately before sub_B168,
        // and Jump_On_Enemy can already restore the saved anchor and inject the
        // bounce velocity there. Native does not have the same collision payload,
        // so record the closest pre-physics stomp/collision carry state here.
        let stomp_candidate =
            self.first_stomp_contact_candidate(slice, player.y_velocity, player.y_pos);
        self.last_kid_frame_collision_handoff_probe = Some(KidCollisionHandoffProbe {
            player_lifecycle: self.player_lifecycle,
            previous_post_motion_mutator: self.previous_kid_frame_post_motion_mutator,
            pending_stomp_bounce: self.pending_stomp_bounce.is_some(),
            pending_contact_damage: self.pending_player_contact_damage.is_some(),
            pending_form: self.progression.pending_form,
            pending_level_skip: self.pending_level_skip,
            form_change_flash_frames: self.progression.form_change_flash_frames,
            on_ground_before: player.on_ground,
            jump_buffer_frames: player.jump_buffer_frames,
            wall_cling_direction: player.wall_cling_direction,
            stomp_candidate: stomp_candidate.is_some(),
            stomp_enemy_object_type: stomp_candidate.as_ref().and_then(|enemy| match enemy.kind {
                NativeObjectKind::Enemy { object_type } => Some(object_type),
                _ => None,
            }),
            stomp_enemy_x_pos: stomp_candidate.as_ref().map(|enemy| enemy.x_pos),
            stomp_enemy_y_pos: stomp_candidate.as_ref().map(|enemy| enemy.y_pos),
            anchor_x_pos: player.x_pos,
            anchor_y_pos: player.y_pos,
        });
    }

    fn note_kid_same_frame_stomp_probe(
        &mut self,
        slice: &OpeningSliceState,
        pre_physics_y_velocity: i32,
        pre_physics_y_pos: u32,
        frame_start_x_pos: u32,
        frame_start_y_pos: u32,
    ) {
        let candidate =
            self.first_stomp_contact_candidate(slice, pre_physics_y_velocity, pre_physics_y_pos);
        self.last_kid_frame_same_frame_stomp_probe = Some(KidSameFrameStompProbe {
            stomp_candidate: candidate.is_some(),
            enemy_object_type: candidate.as_ref().and_then(|enemy| match enemy.kind {
                NativeObjectKind::Enemy { object_type } => Some(object_type),
                _ => None,
            }),
            enemy_x_pos: candidate.as_ref().map(|enemy| enemy.x_pos),
            enemy_y_pos: candidate.as_ref().map(|enemy| enemy.y_pos),
            pre_physics_y_velocity,
            post_physics_y_velocity: slice.player.y_velocity,
            anchor_x_pos: frame_start_x_pos,
            anchor_y_pos: frame_start_y_pos,
        });
    }

    fn note_kid_iron_knight_attach_probe(
        &mut self,
        input: FrameInput,
        player_before: PlayerState,
        climbing_before: bool,
        player_after: PlayerState,
    ) {
        if self.progression.current_form != PlayerForm::IronKnight {
            self.last_kid_frame_iron_knight_attach_probe = None;
            return;
        }

        let c_pressed = input.c && !self.previous_input.c;
        let wall_probe = self.block_grid.as_ref().and_then(|grid| {
            let x_px = (player_before.x_pos >> 16) as i32;
            let y_px = (player_before.y_pos >> 16) as i32;
            let facing_right = player_before.facing == Direction::Right;

            if climbing_before {
                Some(terrain::probe_iron_knight_climb_resume_wall(
                    grid,
                    x_px,
                    y_px,
                    self.progression.current_form.half_width(),
                    facing_right,
                ))
            } else if player_before.on_ground {
                Some(terrain::probe_wall_column(
                    grid,
                    x_px,
                    y_px,
                    self.progression.current_form.half_width(),
                    PLAYER_HEIGHT,
                    facing_right,
                ))
            } else {
                None
            }
        });
        let candidate = c_pressed
            && player_before.on_ground
            && !climbing_before
            && wall_probe.is_some_and(|probe| probe.hit);
        let climbing_after = self.helmet.iron_knight_climbing;
        let attach_triggered = !climbing_before
            && climbing_after
            && player_after.y_velocity == IRON_KNIGHT_ATTACH_Y_VEL;

        self.last_kid_frame_iron_knight_attach_probe = Some(KidIronKnightAttachProbe {
            c_pressed,
            on_ground_before: player_before.on_ground,
            facing: Some(player_before.facing),
            climbing_before,
            climbing_after,
            y_velocity_before: player_before.y_velocity,
            y_velocity_after: player_after.y_velocity,
            wall_probe,
            candidate,
            attach_triggered,
        });
    }

    fn note_kid_flip_branch_probe(
        &mut self,
        input: FrameInput,
        player_before: PlayerState,
        player_after: PlayerState,
        slice: &OpeningSliceState,
    ) {
        // sub_AF10 (kid.asm:13750): kid_flip wall scan — called at
        // kid_flip entry (`bsr.w sub_AF10; beq.w loc_AD34`). Scans the
        // player's leading-edge column for wall tiles across the full
        // body height.
        //   loc_AF2E (kid.asm:13760): helmet-size merge point. d5=$F
        //   for Micromax/Juggernaut (16px body), $1F for others (32px).
        //   `sub.w d5,d6` → d6 = top of body. `asr.w #4` both → row
        //   indices. `sub.w d6,d7` → row count. Load row pointer from
        //   $4A04, add FA78+1 column offset (negated if FA72≠0).
        //   loc_AF52 (kid.asm:13774): column resolve — `add.w x_pos,d6;
        //   asr.w #4,d6; add.w d6,d6; add.w d6,a4` → tile pointer.
        //   loc_AF5C (kid.asm:13780): scan loop — `move.w (a4),d6;
        //   andi.w #$7000,d6; cmpi.w #$6000,d6; bge loc_AF76` — if
        //   wall ($6000+) → hit. Else `add.w Level_width_tiles,a4;
        //   dbf d7,loc_AF5C` — step down one row, repeat.
        //   loc_AF76 (kid.asm:13791): `moveq #1,d7; rts` — wall found.
        //   Fall-through: `moveq #0,d7; rts` — no wall in column.
        // Native: `probe_wall` (3-point body scan) covers the same
        // column. The 3-point probe at y-8, y-h/2, y-h+8 samples every
        // tile row for 32px bodies; for Micromax (16px) the points
        // collapse to fewer distinct rows but still cover the column.
        let wall_probe = self.block_grid.as_ref().map(|grid| {
            terrain::probe_wall(
                grid,
                (player_before.x_pos >> 16) as i32,
                (player_before.y_pos >> 16) as i32,
                self.progression.current_form.half_width(),
                PLAYER_HEIGHT,
                player_before.facing == Direction::Right,
            )
        });
        let wall_scan_hit = wall_probe.is_some_and(|probe| probe.hit);
        let b_pressed = input.b && !self.previous_input.b;
        let c_pressed = input.c && !self.previous_input.c;
        let branch = if !wall_scan_hit {
            KidFlipBranchSelection::NoWallPath
        } else if b_pressed {
            match self.progression.current_form {
                PlayerForm::TheKid => KidFlipBranchSelection::BPressedTheKid,
                // ASM loc_AC42 (kid.asm:13474): non-Kid B-press in kid_flip.
                // Micromax: y_vel=$FFFC0000, x_vel=±$FFFE0000, flip direction,
                // init stru_8B74, bra loc_A6F8. Same velocities as loc_A6AE
                // (platform-grab detach).
                PlayerForm::Micromax => KidFlipBranchSelection::BPressedMicromax,
                // ASM loc_AC88 (kid.asm:13495): non-Kid/non-Micromax B-press
                // in kid_flip. `tst.b ($FFFFFA6A); beq loc_AD2C` — if on
                // Skycutter board: floor probe (sub_B43A) or y near top →
                // loc_ACA4 (`move.l #$30000,y_vel` — board dive). Otherwise
                // fall through to loc_AD2C (no action). In practice only
                // Skycutter can have $FA6A set, so other helmets always skip.
                // Native: step_skycutter_flipboard B-press handles the dive.
                _ if player_before.on_ground => KidFlipBranchSelection::BPressedGroundedLaunch,
                _ => KidFlipBranchSelection::BPressedFallthrough,
            }
        } else if c_pressed {
            // ASM loc_ACB0 (kid.asm:13510): kid_flip C-press dispatch.
            // Cyclone → st (Cyclone_flying). Maniaxe → loc_ACCC: axe throw.
            // Skycutter → loc_AD00/loc_AD26: flipboard deploy. Others → skip.
            match self.progression.current_form {
                PlayerForm::Cyclone => KidFlipBranchSelection::CPressedCyclone,
                PlayerForm::Maniaxe => KidFlipBranchSelection::CPressedManiaxe,
                PlayerForm::Skycutter => KidFlipBranchSelection::CPressedSkycutter,
                _ => KidFlipBranchSelection::CPressedFallthrough,
            }
        } else {
            KidFlipBranchSelection::NoButtonBranch
        };

        self.last_kid_frame_kid_flip_branch_probe = Some(KidFlipBranchProbe {
            wall_scan_hit,
            b_pressed,
            c_pressed,
            helmet_id: self.progression.current_form.helmet_id(),
            facing: Some(player_before.facing),
            on_ground_before: player_before.on_ground,
            on_ground_after: player_after.on_ground,
            branch,
            wall_probe,
        });
        self.last_kid_frame_kid_flip_follow_on_probe =
            slice
                .kid_flip_follow_on_stages
                .map(|stages| KidFlipFollowOnProbe {
                    branch,
                    motion_branch: self.last_kid_frame_motion_branch,
                    post_motion_mutator: self.last_kid_frame_post_motion_mutator,
                    wall_hit_after_floor: slice.physics_trace.is_some_and(|trace| trace.wall_hit),
                    stages,
                });
    }

    fn maybe_hold_wall_cling_boundary_trace_y_velocity(&mut self, player: PlayerState) {
        let Some(probe) = self.last_kid_frame_kid_flip_follow_on_probe else {
            return;
        };
        let sampled = probe.stages.after_floor.y_velocity;
        if sampled == 0
            || probe.stages.after_floor.on_ground
            || probe.stages.after_floor.wall_cling_direction.is_none()
            || !probe.stages.after_wall.on_ground
            || probe.stages.after_wall.y_velocity != 0
            || !player.on_ground
            || player.y_velocity != 0
        {
            return;
        }

        // ASM loc_A762 -> loc_A8DE/sub_902A: after a wall-cling jump-loop frame
        // settles on the wall/floor boundary, the live player state is grounded
        // with cleared y_vel, but oracle frames 957..977 of any-percent-published
        // still expose the post-gravity y_vel sample through the Kid GFX slot.
        self.wall_cling_boundary_trace_y_velocity = Some((sampled, 21));
    }

    fn kid_pre_physics_probe_state_changed(before: PlayerState, after: PlayerState) -> bool {
        before.x_pos != after.x_pos
            || before.y_pos != after.y_pos
            || before.x_velocity != after.x_velocity
            || before.y_velocity != after.y_velocity
            || before.on_ground != after.on_ground
    }

    fn note_kid_pre_physics_mutator(
        &mut self,
        mutator: KidFramePrePhysicsMutator,
        before: PlayerState,
        after: PlayerState,
    ) {
        self.last_kid_frame_pre_physics_before_physics = Some(after);
        if Self::kid_pre_physics_probe_state_changed(before, after) {
            self.last_kid_frame_pre_physics_mutators.push(mutator);
        }
    }

    fn begin_kid_grounded_release_probe(
        &mut self,
        player: PlayerState,
        input: FrameInput,
        jump_pressed: bool,
        custom_gravity: bool,
    ) {
        if self.physics_order != PhysicsOrder::AsmFaithful {
            self.last_kid_frame_grounded_release_probe = None;
            return;
        }

        let held_direction = match (input.left, input.right) {
            (true, false) => Some(Direction::Left),
            (false, true) => Some(Direction::Right),
            _ => None,
        };
        let mut probe = KidGroundedReleaseProbe {
            on_ground_before: player.on_ground,
            held_direction,
            down_held: input.down,
            jump_pressed,
            jump_buffer_frames: player.jump_buffer_frames,
            custom_gravity,
            wall_cling_direction: player.wall_cling_direction,
            velocity_branch: if player.on_ground && jump_pressed {
                KidGroundedReleaseVelocityBranch::JumpPressed
            } else if player.on_ground && player.jump_buffer_frames > 0 {
                KidGroundedReleaseVelocityBranch::JumpBuffered
            } else if !player.on_ground && custom_gravity {
                KidGroundedReleaseVelocityBranch::AirborneCustomGravity
            } else if !player.on_ground {
                KidGroundedReleaseVelocityBranch::AirborneGravity
            } else {
                KidGroundedReleaseVelocityBranch::GroundedHold
            },
            ..KidGroundedReleaseProbe::default()
        };

        if !player.on_ground {
            self.last_kid_frame_grounded_release_probe = Some(probe);
            return;
        }

        let Some(grid) = self.block_grid.as_ref() else {
            self.last_kid_frame_grounded_release_probe = Some(probe);
            return;
        };

        let collision_hw = self.progression.current_form.half_width();
        let x_px = (player.x_pos >> 16) as i16 as i32;
        let y_px = (player.y_pos >> 16) as i16 as i32;
        let probe_row = ((y_px + 1) >> 4).max(0) as u16;
        let left_col = ((x_px - collision_hw) >> 4).max(0) as u16;
        let right_col = ((x_px + collision_hw) >> 4)
            .max(0)
            .min(grid.width_cells as i32 - 1) as u16;

        probe.probe_row = Some(probe_row);
        probe.left_col = Some(left_col);
        probe.right_col = Some(right_col);

        for col in left_col..=right_col {
            let cell = grid.cell_at(col, probe_row);
            if matches!(
                cell.collision,
                terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
            ) {
                probe.support_col = Some(col);
                probe.support_collision = Some(cell.collision);
                probe.support_block_type = Some(cell.block_type);
                break;
            }
        }

        probe.release_triggered = probe.support_col.is_none();
        self.last_kid_frame_grounded_release_probe = Some(probe);
    }

    fn gameplay_input(&self, input: FrameInput) -> FrameInput {
        self.gameplay_input_from_latch(input, self.previous_input)
    }

    fn gameplay_input_from_latch(&self, input: FrameInput, held_latch: FrameInput) -> FrameInput {
        let held = held_latch.with_left_right_conflict_resolved();
        let mut a_held = if self.progression.current_form == PlayerForm::Skycutter {
            self.prior_input.a
        } else {
            held_latch.a
        };
        if self.options_speed_button_fast {
            // ASM loc_749C (kid.asm:7616): after loc_7464 unpacks `Ctrl_A_Held`
            // and resolves Left+Right, Fast Action executes `not.b Ctrl_A_Held`.
            // Demo play skips `gameplay_input`, matching the `Demo_Mode_flag` gate.
            a_held = !a_held;
        }
        FrameInput {
            up: held.up,
            down: held.down,
            // ASM loc_7464 clears Ctrl_Right_Held when Left+Right are both
            // latched, before gameplay movement reads the direction bytes.
            left: held.left,
            right: held.right,
            // ASM loc_7452 hibernates before loc_7464 refreshes `Ctrl_A_Held`.
            // The Skycutter ground path at sub_9A20 exposes that older A/run
            // latch in the Invulnerability audit; non-Skycutter movement stays
            // on the existing oracle-locked held latch.
            a: a_held,
            // Gameplay motion consumes the previous controller state, including jump hold.
            b: held_latch.b,
            c: input.c,
            start: input.start,
        }
    }

    fn readjoypad_input_with_options_loc_bb6(
        &self,
        input: FrameInput,
        demo_mode_active: bool,
    ) -> FrameInput {
        if demo_mode_active {
            return input;
        }
        // ASM loc_BB6 (kid.asm:1361-1368) calls Permute_ABCButtons before
        // loc_C4E computes pressed edges and latches Ctrl_1_Held.
        input.with_abc_buttons_permuted(self.options_controls_scheme)
    }

    fn step_allow_pausing_fade_sub_73d0(&mut self) {
        if self.allow_pausing || !matches!(self.boot_state, BootState::InGame) {
            return;
        }
        // ASM sub_73D0 $73D0: this is the first instruction in the fade
        // worker after the preceding Mode_Level palette call.
        self.palette_dma_suppressed_f890 = false;
        level::apply_pending_palette_fade_sub_14C0(
            self.level_entry_fade_progress_sub_73d0,
            &mut self.level_entry_pending_palette_0_mask_sub_73d0,
            &mut self.level_entry_pending_palette_1_mask_sub_73d0,
            &self.palette_buffer_2,
            &self.palette_buffer_3,
            &mut self.palette_buffer,
        );
        let mut fade = level::FullscreenPaletteFadeSub73D0 {
            palette_to_dma_enabled: false,
            palette_buffer_2: self.palette_buffer_2,
            palette_buffer_3: self.palette_buffer_3,
            fade_progress: self.level_entry_fade_progress_sub_73d0,
            pending_palette_0_mask: self.level_entry_pending_palette_0_mask_sub_73d0,
            pending_palette_1_mask: self.level_entry_pending_palette_1_mask_sub_73d0,
            allow_pausing: self.allow_pausing,
        };
        let fade_completed = level::step_fullscreen_palette_fade_sub_73D0(&mut fade);
        self.level_entry_pending_palette_0_mask_sub_73d0 = fade.pending_palette_0_mask;
        self.level_entry_pending_palette_1_mask_sub_73d0 = fade.pending_palette_1_mask;
        self.mode_level_tail_state_loc_892
            .set_pending_palette_masks_f888_f88c(
                fade.pending_palette_0_mask,
                fade.pending_palette_1_mask,
            );
        self.level_entry_fade_progress_sub_73d0 = fade.fade_progress;
        self.allow_pausing_fade_frames = self.allow_pausing_fade_frames.saturating_add(1);
        if fade_completed {
            self.allow_pausing = true;
            // ASM Load_InGame leaves FBCC as scratch reset by the next load.
            // The title-card / level-skip paths cleared the warp colour at
            // their StartingLevel/Hold -> InGame hand-off (boot.rs:7449/7571);
            // the no-card Teleport path lands directly in InGame, so clear it
            // here once the entry fade finishes — otherwise a later normal
            // level entry (e.g. death respawn) re-tints its black fade yellow.
            self.teleport_warp_colour_fbcc = 0;
        }
    }

    pub(crate) fn initialise_level_entry_fade_sub_73d0(&mut self) {
        let fade = level::initialise_fullscreen_palette_fade_sub_73D0(
            self.palette_buffer,
            self.teleport_warp_colour_fbcc,
        );
        // ASM sub_7196 sets PaletteToDMA_Flag before allocating sub_73D0.
        self.palette_dma_suppressed_f890 = true;
        self.palette_buffer_2 = fade.palette_buffer_2;
        self.palette_buffer_3 = fade.palette_buffer_3;
        self.level_entry_fade_progress_sub_73d0 = fade.fade_progress;
        // ASM sub_73D0 / loc_7414 (kid.asm:7549): the worker seeds both
        // pending masks to `-1` before its first one-frame hibernate, so the
        // first live InGame refresh has a pending full-palette blend.
        self.level_entry_pending_palette_0_mask_sub_73d0 = u32::MAX;
        self.level_entry_pending_palette_1_mask_sub_73d0 = u32::MAX;
        self.mode_level_tail_state_loc_892
            .set_pending_palette_masks_f888_f88c(u32::MAX, u32::MAX);
        self.allow_pausing = fade.allow_pausing;
        self.allow_pausing_fade_frames = 0;
    }

    pub(crate) fn clear_level_entry_fade_sub_73d0(&mut self) {
        self.palette_dma_suppressed_f890 = false;
        self.level_entry_fade_progress_sub_73d0 = 0;
        self.level_entry_pending_palette_0_mask_sub_73d0 = 0;
        self.level_entry_pending_palette_1_mask_sub_73d0 = 0;
        self.mode_level_tail_state_loc_892
            .set_pending_palette_masks_f888_f88c(0, 0);
        self.allow_pausing_fade_frames = 0;
    }

    fn gameplay_jump_pressed(&self) -> bool {
        self.previous_input.b && !self.prior_input.b
    }

    fn starting_level_title_card_bootstrap_active_loc_1ae2e(&self) -> bool {
        matches!(
            self.boot_state,
            BootState::StartingLevel {
                uses_load_ingame_bootstrap: false,
                ..
            }
        ) && self
            .level_header(self.current_level)
            .is_some_and(|header| header.murder_wall_flags != 0)
    }

    /// ASM loc_74E0 (#442): the hidden Mode_Level pass of the debug-warp
    /// title window runs from the measured per-level load completion. The
    /// mapping is `boundary = load_complete_step - 1`: native's elapsed-B
    /// tick is the tick after oracle title-wait row B, and the oracle's first
    /// drawing tick follows row `load_complete - 1`. Anchor: level 0x48
    /// (load_complete 72) is the one level the legacy fixed boundary 71
    /// reproduced exactly, and 71 = 72 - 1.
    pub(crate) fn debug_level_entry_live_boundary_loc_74e0(&self) -> usize {
        crate::scenario::debug_level_entry_load_complete_step_loc_74e0(self.current_level).map_or(
            crate::scenario::DEBUG_LEVEL_ENTRY_LEGACY_LIVE_BOUNDARY_LOC_74E0,
            |load_complete| load_complete.saturating_sub(1),
        )
    }

    pub(crate) fn debug_level_entry_mode_level_active_loc_74e0(&self) -> bool {
        self.debug_level_entry_suppresses_title_card_live_handoff
            && match self.boot_state {
                BootState::StartingLevel { elapsed_frames, .. } => {
                    elapsed_frames > self.debug_level_entry_live_boundary_loc_74e0()
                }
                _ => false,
            }
    }

    #[allow(non_snake_case)]
    fn queue_jump_sound_from_probe_loc_A480(&mut self) {
        let Some(probe) = self.last_kid_frame_jump_gravity_probe else {
            return;
        };
        if matches!(
            probe.branch,
            KidJumpGravityBranch::JumpPressed | KidJumpGravityBranch::JumpBuffered
        ) {
            // ASM sub_A432 loc_A466 stores the launch y velocity in
            // `$FFFFFAAA`; sub_B270 compares against that dynamic threshold
            // when selecting later airborne sprite-table states.
            self.helmet.airborne_mapping_jump_threshold_y_velocity = Some(probe.y_velocity_after);
            self.helmet.airborne_mapping_jump_sequence_active = true;
            // ASM sub_A432 loc_A480 plays `sfx_Jump` after the launch velocity
            // is stored and x_vel is scaled by 3/4.
            self.queue_start_sequence(QueuedSoundEffect::Jump);
        }
    }

    fn queue_kid_pullup_sound_if_started_loc_abf0(&mut self, previous_cooldown: u8, cooldown: u8) {
        if previous_cooldown == 0 && cooldown == LEDGE_SNAP_PULLUP_FRAMES {
            // ASM kid_flip loc_ABF0 (kid.asm:13458-13467) initialises
            // `stru_8BB4`, then plays sfx_The_Kid_pullup before hibernating
            // until the pull-up animation finishes.
            self.queue_start_sequence(QueuedSoundEffect::KidPullup);
        }
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
                let byte = demo::demo_byte(*use_demo_2, *data_index);
                self.demo_buffered_input = Some(demo::decode_demo_byte(byte));
                self.demo_post_transform_trace_velocity_hold_frames =
                    demo::DEMO_POST_TRANSFORM_TRACE_VELOCITY_HOLD_FRAMES;
            }
            self.demo_transform_active = transform_active;
        } else {
            self.demo_transform_active = false;
            self.demo_post_transform_trace_velocity_hold_frames = 0;
        }
    }

    pub(crate) fn gameplay_hibernating_for_transform(&self) -> bool {
        self.progression.form_change_flash_frames > 0
    }

    const fn hud_animation_runs_in_mode_level_loc_80c(
        gameplay_slice_active: bool,
        transform_still_hibernating_before_loc_80c: bool,
    ) -> bool {
        gameplay_slice_active && !transform_still_hibernating_before_loc_80c
    }

    pub(crate) fn object_pass_hibernating_for_transform_loc_7e90(&self) -> bool {
        // ASM Kid_Transform loc_7E90 clears Currently_transforming and returns
        // from the final cleanup count. The outer Execute_Objects loc_E12 pass
        // therefore resumes while native's pre-lifecycle countdown is one.
        self.progression.form_change_flash_frames > 1
    }

    fn object_pass_hibernating_before_mode_level_tail_loc_892(&self) -> bool {
        // ASM Kid_Transform loc_7CBE/loc_7D22 and Death loc_B80C both wait
        // inside the Kid object's Execute_Objects callback. Until it returns,
        // the suspended Mode_Level pass cannot reach loc_80C or loc_892.
        self.object_pass_hibernating_for_transform_loc_7e90()
            || (self.opening_slice.is_some()
                && matches!(
                    self.player_lifecycle,
                    PlayerLifecycle::DeathWindup { .. } | PlayerLifecycle::Dying { .. }
                ))
    }

    fn apply_form_width_overlap_fixup(&mut self, slice: &mut OpeningSliceState) {
        if self.progression.form_width_delta == 0 {
            return;
        }
        if self.block_grid.is_none() {
            self.progression.form_width_delta = 0;
            return;
        }

        if let Some(aligned_x) =
            self.resolve_transform_overlap_sub_7b30(slice, self.progression.current_form)
        {
            slice.player.x_pos = (aligned_x as u32) << 16;
        }

        self.progression.form_width_delta = 0;
    }

    fn resolve_transform_overlap_sub_7b30(
        &self,
        slice: &OpeningSliceState,
        target_form: PlayerForm,
    ) -> Option<i32> {
        let grid = self.block_grid.as_ref()?;
        let x_px = (slice.player.x_pos >> 16) as i32;
        let foot_y_px = (slice.player.y_pos >> 16) as i32;
        let half_width = target_form.half_width();
        // ASM loc_757E (kid.asm:7691–7698): `moveq #$10,d6` then adds $10
        // unless Current_Helmet_Available is Micromax (9) or Juggernaut (5).
        // d6=$10 → short form (1 cell tall), d6=$20 → tall form (2 cells).
        // sub_7B30 converts: d6==$10 → d5=0 rows, else d5=1 row.
        let probe_height = target_form.ceiling_probe_height() + 1;
        let mut rows_up = if probe_height == 0x10 { 0 } else { 1 };
        // ASM loc_7B48 (kid.asm:8241): `move.w y_pos(a3),d4; andi.w #$F,d4;
        // cmpi.w #$F,d4; beq.w loc_7B5A; addq.w #1,d5` — if foot pixel is
        // NOT tile-aligned (y_pos & $F != $F), add one extra row to scan.
        if foot_y_px & 0xF != 0xF {
            rows_up += 1;
        }

        // ASM loc_7B5A (kid.asm:8248): `move.w x_pos(a3),d6; sub.w d7,d6;
        // bpl.w loc_7B74` — compute left_edge = x_pos - half_width. If
        // negative (off-screen left), snap x to half_width and check right.
        let left_edge = x_px - half_width;
        if left_edge < 0 {
            let cleared_x = half_width;
            let right_edge = cleared_x + half_width;
            // ASM loc_7BB6 (kid.asm:8287): `move.w 2(sp),d7; bsr.w sub_7BEA;
            // beq.s loc_7B9E` — shared right-column probe entry. Reached from
            // both the negative-left-edge snap (loc_7B5A) and the left-blocked
            // snap-right (loc_7BA4). Probes right_edge column upward; if clear
            // → loc_7B9E (success). If blocked → loc_7BC0 (restore & fail).
            return if terrain::transform_overlap_column_blocked_sub_7bea(
                grid, right_edge, foot_y_px, rows_up,
            ) {
                // ASM loc_7BC0 (kid.asm:8292): `moveq #1,d7; addq.w #4,sp;
                // move.l d0,x_pos(a3); move.l d1,y_pos(a3); rts` — both
                // snaps failed, restore original position and return blocked.
                None
            } else {
                Some(cleared_x)
            };
        }

        // ASM loc_7B74 (kid.asm:8259): `move.w d6,d4; move.w 2(sp),d7;
        // bsr.w sub_7BEA; bne.w loc_7BA4` — left edge non-negative; probe
        // left column for walls. If blocked, jump to loc_7BA4 (snap right).
        if terrain::transform_overlap_column_blocked_sub_7bea(grid, left_edge, foot_y_px, rows_up) {
            // ASM loc_7BA4 (kid.asm:8279): `andi.w #$FFF0,d4; addi.w #$10,d4;
            // add.w (sp),d4; move.w d4,x_pos(a3)` — snap x_pos right to the
            // next tile boundary + half_width, then check the right column.
            let cleared_x = (left_edge & !0xF) + 16 + half_width;
            let right_edge = cleared_x + half_width;
            // Falls through to loc_7BB6 (kid.asm:8287) — right-column probe
            // after snap. loc_7BC0 if still blocked.
            return if terrain::transform_overlap_column_blocked_sub_7bea(
                grid, right_edge, foot_y_px, rows_up,
            ) {
                None
            } else {
                Some(cleared_x)
            };
        }

        // ASM loc_7B74 continued (kid.asm:8264–8271): left clear — probe right
        // column. `cmp.w (Level_width_pixels).w,d6; bge.w loc_7BCE` — if right
        // edge >= level width, snap left. Otherwise sub_7BEA on right column.
        let right_edge = x_px + half_width;
        if right_edge < grid.width_pixels()
            && !terrain::transform_overlap_column_blocked_sub_7bea(
                grid, right_edge, foot_y_px, rows_up,
            )
        {
            // ASM loc_7B9E (kid.asm:8273): `moveq #0,d7; addq.w #4,sp; rts`
            // — both columns clear, return success with position unchanged.
            return Some(x_px);
        }

        // ASM loc_7BCE (kid.asm:8300): right column blocked or off-level —
        // snap x_pos left: `(right_edge & $FFF0) - 1 - half_width`.
        let cleared_x = (right_edge & !0xF) - 1 - half_width;
        if cleared_x < 0 {
            return None;
        }
        let left_edge = cleared_x - half_width;
        // ASM loc_7BCE cont. (kid.asm:8307–8310): `bsr.w sub_7BEA; beq.s loc_7B9E;
        // bra.s loc_7BC0` — if left probe after snap-left also blocked → loc_7BC0.
        if terrain::transform_overlap_column_blocked_sub_7bea(grid, left_edge, foot_y_px, rows_up) {
            None
        } else {
            Some(cleared_x)
        }
    }

    fn handle_pending_form_change_sub_7acc(&mut self, slice: &mut OpeningSliceState) -> bool {
        let Some(target_form) = self.progression.pending_form else {
            return false;
        };

        // ASM sub_7ACC (kid.asm:8178): when `Check_Helmet_Change` is set, try
        // sub_7B30's space-clear using the target helmet hitbox. If it fails,
        // Kid drops the queued helmet, while any already-helmeted form dies.
        // ASM loc_8252 (kid.asm:9013): in crawling state (loc_8218), after
        // sub_7ACC returns NE, Micromax(9) and Juggernaut(5) exit crawling
        // via `bclr #Button_B; bra.w loc_8E16`; other forms continue
        // crawling. Native: no separate crawling state, so transforms
        // resolve uniformly here.
        //
        // ASM loc_7AD6 (kid.asm:8185): `cmpi.w #6,($FFFFFA56).w; beq.w loc_7B12`
        // — if Character_Movement == 6 (uphill-downhill/slope), skip the
        // sub_7B30 overlap check entirely and jump straight to Kid_Transform.
        if self.slope_transform_bypass_loc_7ad6(slice) {
            self.resolve_pending_form_change();
            self.apply_form_width_overlap_fixup(slice);
            return false;
        }
        //
        // ASM loc_7B0A (kid.asm:8202): short-form helmets (Micromax=9,
        // Juggernaut=5) keep d6=$10; others get d6=$20 via `addi.w #$10,d6`.
        // Native: `ceiling_probe_height()` returns 15 vs 31, producing the
        // same rows_up values in `resolve_transform_overlap_sub_7b30`.
        if let Some(aligned_x) = self
            .block_grid
            .as_ref()
            .and_then(|_| self.resolve_transform_overlap_sub_7b30(slice, target_form))
        {
            // ASM loc_7B12 (kid.asm:8206): `bra.w Kid_Transform` — sub_7B30
            // returned 0 (space clear), proceed with the helmet transform.
            slice.player.x_pos = (aligned_x as u32) << 16;
            self.resolve_pending_form_change();
            self.apply_form_width_overlap_fixup(slice);
            return false;
        }

        if self.block_grid.is_none() {
            self.resolve_pending_form_change();
            self.apply_form_width_overlap_fixup(slice);
            return false;
        }

        // ASM loc_7B1A (kid.asm:8213): sub_7B30 returned non-zero (blocked).
        // `tst.w (Current_Helmet).w` — if helmeted, `bra.w Death` (crush).
        // ASM loc_7B28 (kid.asm:8220): if unhelmeted Kid, `sf (Check_Helmet_Change).w`
        // — cancel the pending transform silently and return d7=0.
        self.progression.pending_form = None;
        if self.progression.current_form == PlayerForm::TheKid {
            return false;
        }

        self.progression.number_hitpoints = 0;
        self.opening_slice = Some(*slice);
        self.begin_death_windup();
        true
    }

    fn slope_transform_bypass_loc_7ad6(&self, slice: &OpeningSliceState) -> bool {
        if !slice.player.on_ground || slice.player.x_velocity == 0 {
            return false;
        }
        let Some(grid) = self.block_grid.as_ref() else {
            return false;
        };
        let half_width = self.progression.current_form.half_width();
        let x_px = (slice.player.x_pos >> 16) as i32;
        let foot_y_px = (slice.player.y_pos >> 16) as i32;
        [x_px - half_width, x_px, x_px + half_width]
            .into_iter()
            .map(|probe_x| terrain::probe_floor(grid, probe_x, foot_y_px))
            .any(|probe| {
                probe.on_ground
                    && matches!(
                        probe.cell.collision,
                        terrain::CollisionKind::SlopeUp | terrain::CollisionKind::SlopeDown
                    )
            })
    }

    fn try_apply_warp_cheat_sub_b43a(
        &mut self,
        slice: &OpeningSliceState,
        movement_input: FrameInput,
    ) -> bool {
        let x_px = (slice.player.x_pos >> 16) as u16;
        let y_px = (slice.player.y_pos >> 16) as u16;
        if self.current_level != LevelId(1)
            || !slice.player.on_ground
            || !movement_input.down
            || !movement_input.b
            || x_px != 0x09D3
            || y_px != 0x005F
        {
            return false;
        }

        // ASM sub_B43A (kid.asm:14352): MoveID_Crawling on level 1 at
        // x=$09D3/y=$005F with Button_B held writes Current_LevelID=$32,
        // then fabricates `colid_kidbelow` + `object_meta=$A0`. The latter
        // routes Character_CheckCollision to CharacterCollision_TouchFlagpole.
        self.current_level = LevelId(0x32);
        self.last_kid_collision_type = 0x0028;
        self.begin_flagpole_level_complete_loc_c056(
            slice,
            KidFramePostMotionMutator::WarpCheatFlagTouch,
        );
        true
    }

    fn refresh_opening_slice(&mut self, input: FrameInput, jump_pressed: bool) {
        if !self.step_lifecycle(input) {
            self.demo_transform_active = false;
            return;
        }
        self.update_demo_transform_parity();

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
        self.stomped_this_frame = false;
        self.begin_kid_collision_handoff_probe(&slice);
        self.begin_kid_pre_physics_probe(slice.player);
        let player_before_form_width_overlap_fixup = slice.player;
        self.apply_form_width_overlap_fixup(&mut slice);
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::FormWidthOverlapFixup,
            player_before_form_width_overlap_fixup,
            slice.player,
        );
        let player_before_pending_stomp_bounce = slice.player;
        self.apply_pending_stomp_bounce(&mut slice);
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::PendingStompBounce,
            player_before_pending_stomp_bounce,
            slice.player,
        );
        slice.prize_block_hit = None;
        slice.lift_block_hit = None;
        slice.shifting_block_hit = None;
        slice.metal_block_hit = None;
        slice.rubber_block_hit = None;
        slice.smash_block_hit = None;
        slice.ice_block_hit = None;
        slice.shooter_block_hit = None;
        slice.teleporter_contact = None;
        slice.ceiling_prize_clear = None;
        self.blocks_hit_this_frame = 0;
        self.sound_test_play_pending = false;
        // ASM Kid_Transform replaces the normal frame loop with a coroutine
        // that hibernates gameplay objects. Keep the previous slice/roster
        // stable until the transform flash countdown finishes.
        if self.gameplay_hibernating_for_transform() {
            self.last_kid_frame_motion_branch = KidFrameMotionBranch::TransformHibernating;
            self.opening_slice = Some(slice);
            return;
        }
        // Tick visual effects
        self.step_visual_effects();
        // ASM loc_75D4 calls sub_7ACC from the Kid dispatch, while a
        // PrizeHelmet* worker at loc_104AE/loc_105D6 can set its pending form
        // before that dispatch. Run the same space-clear immediately, before
        // movement, so Kid_Transform owns this wake frame.
        if self.handle_pending_form_change_sub_7acc(&mut slice) {
            return;
        }
        // ASM PrizeHelmet* loc_104AE/loc_105D6 can start Kid_Transform from
        // an object wake before this frame's movement state dispatch. Once it
        // has done so, the transform coroutine owns the frame; letting the
        // ordinary loop continue moves Kid into the adjacent wall first.
        if self.gameplay_hibernating_for_transform() {
            self.last_kid_frame_motion_branch = KidFrameMotionBranch::TransformHibernating;
            self.opening_slice = Some(slice);
            return;
        }
        // Prize blocks headbutted from below update their queued sub_1007A
        // lifecycle before movement, so the clear frame is collision-visible.
        self.step_deferred_ceiling_clears();
        // ASM sub_114C0 / sub_114F4 are the same style of coroutine: shooter
        // headbutt tiles update before physics on their wake frames.
        self.step_shooter_headbutt_workers();
        if self.prize_helmet_collision_hibernate_delay_frames > 0 {
            self.prize_helmet_collision_hibernate_delay_frames -= 1;
            if self.prize_helmet_collision_hibernate_delay_frames == 0 {
                // ASM loc_7E90 (kid.asm:8597): transform cleanup clears both
                // velocity longwords before returning to the regular movement
                // loop. The #258 latched-prize path reaches the same cleanup
                // timing even though native models the visual transform as a
                // hibernate-only special case.
                slice.player.x_velocity = 0;
                slice.player.y_velocity = 0;
                // ASM loc_7CD0 must have written the new helmet before the
                // cleanup returns. The to-Kid form break (loc_B786) has
                // transform_anim_frames(TheKid)=0, so its switch point never
                // matches inside the hibernate window — consume it here.
                if let Some(form) = self.prize_helmet_collision_hibernate_switch_form.take() {
                    self.progression.current_form = form;
                }
            } else if self.prize_helmet_collision_hibernate_delay_frames < 115 {
                // ASM loc_A6F8 (kid.asm:13038): the airborne path has already
                // run movement and Hibernate_Object_1Frame. PrizeHelmet*
                // loc_105D6 can expose that held Kid pose after the object
                // sees collision_type(a3) and deletes itself.
                self.last_kid_frame_motion_branch =
                    KidFrameMotionBranch::PrizeHelmetCollisionHibernate;
                if let Some(form) = self.prize_helmet_collision_hibernate_hitpoints_form.take() {
                    // ASM sub_7EB2 (kid.asm:8614): the latched collision
                    // exposes the touched helmet's hitpoint seed even though
                    // oracle keeps Current_Helmet unchanged until the old
                    // helmet reverse table reaches loc_7CD0.
                    self.progression.number_hitpoints =
                        form.max_hitpoints() + self.progression.extra_hitpoint_slots;
                }
                if let Some(form) = self.prize_helmet_collision_hibernate_switch_form
                    && self.prize_helmet_collision_hibernate_delay_frames
                        == crate::combat::transform_anim_frames(form) as u8
                {
                    // ASM loc_7CD0 (kid.asm:8426): after the old helmet's
                    // reverse animation entries have hibernated, write
                    // Current_Helmet_Available into Current_Helmet before
                    // walking the new helmet's forward table.
                    self.progression.current_form = form;
                    self.prize_helmet_collision_hibernate_switch_form = None;
                }
                if self.trace_physics {
                    slice.physics_trace = Some(PhysicsTrace {
                        pre_horizontal: (slice.player.x_pos, slice.player.x_velocity),
                        post_horizontal: (slice.player.x_pos, slice.player.x_velocity),
                        horizontal_step_applied: false,
                        pre_vertical: (slice.player.y_pos, slice.player.y_velocity),
                        post_vertical: (slice.player.y_pos, slice.player.y_velocity),
                        pre_gravity_y_vel: slice.player.y_velocity,
                        post_gravity_y_vel: slice.player.y_velocity,
                        gravity_applied: false,
                        on_ground_before: slice.player.on_ground,
                        on_ground_after: slice.player.on_ground,
                        final_pos: (slice.player.x_pos, slice.player.y_pos),
                        final_vel: (slice.player.x_velocity, slice.player.y_velocity),
                        ..PhysicsTrace::default()
                    });
                }
                self.opening_slice = Some(slice);
                return;
            }
        }
        let frame_start_on_ground = slice.player.on_ground;
        let frame_start_walk_visual_accumulator = slice.walk_visual_accumulator;

        // Step helmet abilities BEFORE physics
        let player_before_skycutter_flipboard = slice.player;
        self.step_skycutter_flipboard(&mut slice, input);
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::SkycutterFlipboard,
            player_before_skycutter_flipboard,
            slice.player,
        );
        let player_before_cyclone_flight = slice.player;
        let cyclone_acceleration_before_sub_b168 =
            (self.progression.current_form == PlayerForm::Cyclone && self.helmet.cyclone_flying)
                .then_some(self.helmet.cyclone_y_accel);
        self.step_cyclone_flight(&mut slice, input);
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::CycloneFlight,
            player_before_cyclone_flight,
            slice.player,
        );
        let player_before_iron_knight_climb = slice.player;
        let iron_knight_climbing_before = self.helmet.iron_knight_climbing;
        let iron_knight_climb_step = self.step_iron_knight_climb(&mut slice, input);
        self.note_kid_iron_knight_attach_probe(
            input,
            player_before_iron_knight_climb,
            iron_knight_climbing_before,
            slice.player,
        );
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::IronKnightClimb,
            player_before_iron_knight_climb,
            slice.player,
        );
        // ASM loc_8CB6 (kid.asm:10100): walking-state C-press → Maniaxe axe throw.
        // ASM loc_8CF8 (kid.asm:10117): walking-state C-press → Red Stealth sword.
        // ASM loc_8C26 (kid.asm:10058): walking-state sub_7428 + Eyeclops hard beam.
        // Native runs these helpers every frame, so Red Stealth keeps the
        // C-press allocation gated to the grounded walking path while the
        // already-allocated sword worker timer can keep ticking.
        self.step_red_stealth_sword_with_start_gate(input, slice.player.on_ground);
        // ASM sub_B270 loc_B350 (kid.asm:14220-14248) drives the Red Stealth
        // airborne attack-down slash counter independently of the ground swing
        // timer. Wired here so the counter advances even though the renderer
        // hasn't yet consumed the returned sprite mapping (#153 first layer).
        self.step_red_stealth_airborne_slash(&slice, input);
        self.step_eyeclops_beam(&slice, input);
        self.step_maniaxe_axe(&slice, input);
        self.step_berzerker_charge(&mut slice);
        self.step_juggernaut_skull(&slice, input);
        let player_before_micromax_stick = slice.player;
        self.step_micromax_stick(&mut slice, input);
        self.note_kid_pre_physics_mutator(
            KidFramePrePhysicsMutator::MicromaxStick,
            player_before_micromax_stick,
            slice.player,
        );
        let frame_start_x_pos = slice.player.x_pos;
        let frame_start_y_pos = slice.player.y_pos;

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
        let mut platform_support_crawling = false;
        if self.progression.form_change_flash_frames == 0 {
            let deferred_prize_ceiling_cells: Vec<(u16, u16, DeferredPrizeBlockPhase)> = self
                .block_state
                .deferred_ceiling_clears
                .iter()
                .filter(|entry| entry.phase != DeferredPrizeBlockPhase::Deleted)
                .map(|entry| (entry.col, entry.row, entry.phase))
                .collect();
            let hidden_block_cells_before_physics = self.hidden_block_reveal_cells_for_diff();
            // Helmet abilities that override normal gravity
            let custom_gravity = self.helmet.skycutter_on_board
                || self.helmet.cyclone_flying
                || self.helmet.iron_knight_climbing
                || self.helmet.micromax_stuck;
            let crawl_standup_platform_blocked = !input.down
                && self.previous_input.down
                && slice.player.on_ground
                && crawl_standup_platform_overhead_sub_DAA6(
                    &self.active_platforms,
                    slice.player.x_pos,
                    slice.player.y_pos,
                    self.progression.current_form,
                );
            let mut movement_input = if crawl_standup_platform_blocked {
                // ASM loc_8270 -> sub_DAA6: NE returns to loc_82D8, so the
                // frame remains on the crawling movement path even after Down
                // is released. Native has no explicit movement ID, so keep the
                // physics input in crawl mode for this frame only.
                FrameInput {
                    down: true,
                    ..input
                }
            } else {
                input
            };

            if !input.down
                && self.previous_input.down
                && slice.player.on_ground
                && !crawl_standup_platform_blocked
                && let Some(grid) = self.block_grid.as_ref()
                && movement::crawl_standup_head_blocked_loc_8270(
                    grid,
                    slice.player.x_pos,
                    slice.player.y_pos,
                    self.progression.current_form.half_width(),
                )
            {
                if self.progression.current_form == PlayerForm::Skycutter {
                    // ASM loc_8270: a blocked Skycutter stand-up branches
                    // straight back to loc_82D8 instead of calling sub_81B4.
                    movement_input.down = true;
                } else {
                    let standup_escape = movement::crawl_wall_escape_sub_81B4(
                        grid,
                        slice.player.x_pos,
                        slice.player.y_pos,
                        self.progression.current_form.half_width(),
                    );
                    slice.player.x_pos = standup_escape.x_pos;
                    if standup_escape.blocked {
                        // ASM sub_81B4 returns NE only when both crawl side
                        // probes are solid, so the release frame stays crawling.
                        movement_input.down = true;
                    }
                }
            }

            // ASM loc_726C/loc_727E (kid.asm:7377/7385): Y-axis look-offset
            // pull-back adjustments run once per frame inside sub_71E4.
            // loc_728C's bottom-bound adjustment happens after the shifted
            // target is clamped, so it affects the accumulator for future
            // frames while the current camera step still uses the pre-decrement
            // target.
            {
                let kid_y = (slice.player.y_pos >> 16) as i16;
                let cam_y = (slice.camera.y_pos >> 16) as i16;
                let max_y = i32::from(slice.level_bounds.height_pixels) - physics::VIEWPORT_HEIGHT;
                let target_y = (i32::from(kid_y) - physics::CAMERA_PLAYER_OFFSET)
                    .max(0)
                    .min(max_y);
                self.y_look_offset += physics::camera_y_ceiling_look_adjust_loc_726c(
                    target_y,
                    kid_y,
                    self.camera_last_kid_y,
                );
                self.y_look_offset +=
                    physics::camera_y_viewport_top_look_adjust_loc_727e(kid_y, cam_y);
                self.y_look_offset += physics::camera_y_bottom_look_adjust_loc_728c(
                    target_y,
                    max_y,
                    self.y_look_offset,
                );
            }

            // ASM loc_8C70 (kid.asm:10079): walking-state crawl transition —
            // Micromax/Juggernaut and sub_7A10-blocked frames branch to
            // loc_8CB6 instead of entering MoveID_Crawling. Native has no
            // explicit crawl movement ID, so suppress Down only for gameplay
            // movement while keeping the original input for look offset.
            if input.down && slice.player.on_ground {
                if matches!(
                    self.progression.current_form,
                    PlayerForm::Micromax | PlayerForm::Juggernaut
                ) {
                    movement_input.down = false;
                } else if let Some(grid) = self.block_grid.as_ref() {
                    let down_hold_alignment = movement::align_player_for_down_hold_sub_7A10(
                        grid,
                        slice.player.x_pos,
                        slice.player.y_pos,
                        slice.level_bounds.width_pixels,
                    );
                    slice.player.x_pos = down_hold_alignment.x_pos;
                    if down_hold_alignment.blocked {
                        movement_input.down = false;
                    }
                }
            }

            if self.try_apply_warp_cheat_sub_b43a(&slice, movement_input) {
                return;
            }

            slice.wall_cling_release_probe = None;
            slice.kid_flip_follow_on_stages = None;
            slice.kid_jump_gravity_probe = None;
            let shooter_barrel_support_floor_y = self
                .apply_shooter_barrel_support_sub_1f52(&mut slice, self.progression.current_form);
            self.shooter_barrel_damage_latched_loc_1f52 = shooter_barrel_support_floor_y.is_some();
            let shooter_damage_landing_handoff = shooter_barrel_support_floor_y.filter(|_| {
                self.previous_kid_frame_post_motion_mutator
                    == KidFramePostMotionMutator::ProjectileDamage
                    && self.block_grid.as_ref().is_none_or(|grid| {
                        let x_px = (slice.player.x_pos >> 16) as i32;
                        let y_px = (pre_physics_y_pos >> 16) as i32;
                        let half_width = self.progression.current_form.half_width();
                        let row = ((y_px + 1) >> 4).max(0) as u16;
                        let left_col = ((x_px - half_width) >> 4).max(0) as u16;
                        let right_col = ((x_px + half_width) >> 4)
                            .max(0)
                            .min(grid.width_cells as i32 - 1)
                            as u16;
                        !(left_col..=right_col).any(|col| {
                            matches!(
                                grid.cell_at(col, row).collision,
                                terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
                            )
                        })
                    })
            });

            if iron_knight_climb_step == IronKnightClimbStep::AttachHibernate {
                // ORIGINAL_QUIRK: `loc_A276` seeds wall-climb state, then
                // `Hibernate_Object_1Frame` stops Kid immediately. The attach
                // frame preserves position while the new upward velocity becomes
                // visible in object state. `loc_A28A` still runs `sub_71E4`, so
                // camera follow advances once on the hibernated attach frame.
                self.last_kid_frame_motion_branch = KidFrameMotionBranch::IronKnightAttachHibernate;
                if self.trace_physics {
                    slice.physics_trace = Some(PhysicsTrace {
                        pre_horizontal: (slice.player.x_pos, slice.player.x_velocity),
                        post_horizontal: (slice.player.x_pos, slice.player.x_velocity),
                        horizontal_step_applied: false,
                        pre_vertical: (slice.player.y_pos, slice.player.y_velocity),
                        post_vertical: (slice.player.y_pos, slice.player.y_velocity),
                        pre_gravity_y_vel: slice.player.y_velocity,
                        post_gravity_y_vel: slice.player.y_velocity,
                        gravity_applied: false,
                        on_ground_before: slice.player.on_ground,
                        on_ground_after: slice.player.on_ground,
                        final_pos: (slice.player.x_pos, slice.player.y_pos),
                        final_vel: (slice.player.x_velocity, slice.player.y_velocity),
                        ..PhysicsTrace::default()
                    });
                }
                slice.update_camera_follow(self.y_look_offset);
            } else if iron_knight_climb_step == IronKnightClimbStep::ExitHibernate {
                // ORIGINAL_QUIRK: `loc_A2E2`/`loc_A2F6` flip facing and jump
                // into `loc_A6F8`, which hibernates the release frame before
                // normal jump physics consumes the preserved climb velocity on
                // the next frame. `loc_A6F8` still runs `sub_71E4`, so camera
                // follow advances once on the hibernated release frame.
                self.last_kid_frame_motion_branch =
                    KidFrameMotionBranch::IronKnightClimbExitHibernate;
                if self.trace_physics {
                    slice.physics_trace = Some(PhysicsTrace {
                        pre_horizontal: (slice.player.x_pos, slice.player.x_velocity),
                        post_horizontal: (slice.player.x_pos, slice.player.x_velocity),
                        horizontal_step_applied: false,
                        pre_vertical: (slice.player.y_pos, slice.player.y_velocity),
                        post_vertical: (slice.player.y_pos, slice.player.y_velocity),
                        pre_gravity_y_vel: slice.player.y_velocity,
                        post_gravity_y_vel: slice.player.y_velocity,
                        gravity_applied: false,
                        on_ground_before: slice.player.on_ground,
                        on_ground_after: slice.player.on_ground,
                        final_pos: (slice.player.x_pos, slice.player.y_pos),
                        final_vel: (slice.player.x_velocity, slice.player.y_velocity),
                        ..PhysicsTrace::default()
                    });
                }
                slice.update_camera_follow(self.y_look_offset);
            } else if let Some(support_floor_y) = shooter_damage_landing_handoff {
                self.last_kid_frame_motion_branch = KidFrameMotionBranch::NormalPhysics;
                self.begin_kid_grounded_release_probe(
                    slice.player,
                    movement_input,
                    jump_pressed,
                    custom_gravity,
                );
                let jump_profile = physics::JUMP_AIR_PROFILES
                    [usize::from(self.progression.current_form.helmet_id())];
                let pre_handoff_x_velocity = slice.player.x_velocity;
                let (x_velocity, facing) = physics::update_jump_horizontal_velocity(
                    slice.player.x_velocity,
                    slice.player.facing,
                    physics::HorizontalInput {
                        left: movement_input.left,
                        right: movement_input.right,
                        run: movement_input.a,
                    },
                    jump_profile,
                );
                // ASM Mode_Level calls sub_1F52 after the Kid/projectile
                // collision passes. If ShooterBlock damage consumed the prior
                // frame, the late `colid_kidbelow` latch is visible as the
                // next frame's landing handoff, not as loc_B5F6's one-pixel
                // walking release from the old airborne Y.
                slice.player.x_velocity = x_velocity;
                slice.player.facing = facing;
                slice.player.y_pos = support_floor_y;
                slice.player.y_velocity = 0;
                slice.player.on_ground = true;
                slice.player.on_spikes = false;
                slice.player.wall_cling_direction = None;
                slice.terrain.floor_y_pos = support_floor_y;
                slice.terrain.floor_terrain_type = TerrainType::Normal;
                slice.terrain.slope_walk_kind = None;
                slice.terrain.slope_sliding = false;
                slice.player.jump_buffer_frames = 0;
                if let Some(ref mut t) = slice.physics_trace {
                    t.pre_horizontal = (frame_start_x_pos, pre_handoff_x_velocity);
                    t.post_horizontal = (frame_start_x_pos, x_velocity);
                    t.horizontal_step_applied = false;
                    t.pre_vertical = (pre_physics_y_pos, pre_physics_y_velocity);
                    t.post_vertical = (support_floor_y, 0);
                    t.pre_gravity_y_vel = pre_physics_y_velocity;
                    t.post_gravity_y_vel = 0;
                    t.gravity_applied = false;
                    t.on_ground_before = false;
                    t.on_ground_after = true;
                    t.final_pos = (slice.player.x_pos, slice.player.y_pos);
                    t.final_vel = (slice.player.x_velocity, slice.player.y_velocity);
                }
                slice.update_camera_follow(self.y_look_offset);
            } else if iron_knight_climb_step == IronKnightClimbStep::ResumeClimb {
                self.last_kid_frame_motion_branch = KidFrameMotionBranch::IronKnightClimbResume;
                let allow_asm_faithful_solid_touch_snap = !self.demo_gameplay_active();
                self.begin_kid_grounded_release_probe(
                    slice.player,
                    input,
                    jump_pressed,
                    custom_gravity,
                );
                slice.step_iron_knight_climb_resume(
                    movement_input,
                    jump_pressed,
                    self.progression.current_form,
                    self.block_grid.as_mut(),
                    self.block_list.as_ref(),
                    self.y_look_offset,
                    allow_asm_faithful_solid_touch_snap,
                    &deferred_prize_ceiling_cells,
                );
                self.last_kid_frame_jump_gravity_probe = slice.kid_jump_gravity_probe;
                self.queue_jump_sound_from_probe_loc_A480();
            } else {
                self.last_kid_frame_motion_branch = KidFrameMotionBranch::NormalPhysics;
                let ledge_snap_cooldown_before = slice.player.ledge_snap_cooldown;
                let player_before_step = slice.player;
                // ASM sub_902A lands only after the contact row is reached. The
                // legacy touch-snap shortcut is useful for non-AsmFaithful
                // orders, but AsmFaithful must preserve the remaining
                // sub-half-pixel gap until the real contact row or gameplay and
                // demo arcs land one frame early.
                let allow_asm_faithful_solid_touch_snap =
                    self.physics_order != PhysicsOrder::AsmFaithful;
                let suppress_probe_slope_velocity = self.probe_start_slope_settle_frames == 2;
                self.probe_start_slope_settle_frames =
                    self.probe_start_slope_settle_frames.saturating_sub(1);
                self.begin_kid_grounded_release_probe(
                    slice.player,
                    movement_input,
                    jump_pressed,
                    custom_gravity,
                );
                slice.step_with_order_and_maniaxe_axe(
                    movement_input,
                    jump_pressed,
                    self.progression.current_form,
                    self.block_grid.as_mut(),
                    self.block_list.as_ref(),
                    custom_gravity,
                    self.y_look_offset,
                    self.helmet.maniaxe_axe_active,
                    self.physics_order,
                    allow_asm_faithful_solid_touch_snap,
                    self.demo_post_transform_trace_velocity_hold_frames > 0,
                    suppress_probe_slope_velocity,
                    &deferred_prize_ceiling_cells,
                );
                if ledge_snap_cooldown_before == 1 && slice.player.ledge_snap_cooldown == 0 {
                    // ASM loc_1062 -> sub_78E8 -> loc_75D4: the final pull-up
                    // animation tick returns to the caller, installs the
                    // standing mapping, and hibernates without normal physics.
                    self.last_kid_frame_motion_branch =
                        KidFrameMotionBranch::PullUpAnimationFinished;
                }
                self.last_kid_frame_jump_gravity_probe = slice.kid_jump_gravity_probe;
                if suppress_probe_slope_velocity
                    && slice.player.on_ground
                    && slice.player.x_velocity == 0
                    && slice.player.y_velocity == 0
                {
                    // ASM loc_9F18 direct probe WRAM seed: oracle frame 0
                    // exposes the patched slope contact row after physics
                    // finishes, but before neutral loc_A13A slope continuation.
                    slice.player.y_pos = slice.player.y_pos.wrapping_add(0x0001_0000);
                    slice.terrain.floor_y_pos = slice.player.y_pos;
                }
                self.queue_jump_sound_from_probe_loc_A480();
                self.queue_kid_pullup_sound_if_started_loc_abf0(
                    ledge_snap_cooldown_before,
                    slice.player.ledge_snap_cooldown,
                );
                if ledge_snap_cooldown_before == 0
                    && slice.player.ledge_snap_cooldown == LEDGE_SNAP_PULLUP_FRAMES
                {
                    let x_px = (player_before_step.x_pos >> 16) as i16;
                    let foot_y_px = (player_before_step.y_pos >> 16) as i16;
                    let half_width = self.progression.current_form.half_width() as i16;
                    // ASM sub_21F8 loc_2232 writes the pre-snap Kid bounds.
                    // Pull-up art then uses the loc_224C zero-size sentinel,
                    // which keeps these globals live after loc_ABF0 snaps the
                    // sprite anchor to the ledge.
                    self.retained_kid_hitbox_loc_224c = Some(combat::CollisionRectSub1F52 {
                        left_px: x_px.saturating_sub(half_width),
                        right_px: x_px.saturating_add(half_width),
                        top_px: foot_y_px.saturating_sub(PLAYER_HEIGHT as i16),
                        bottom_px: foot_y_px,
                    });
                } else if slice.player.ledge_snap_cooldown == 0 {
                    self.retained_kid_hitbox_loc_224c = None;
                }
            }
            if !matches!(
                iron_knight_climb_step,
                IronKnightClimbStep::AttachHibernate | IronKnightClimbStep::ExitHibernate
            ) {
                // ASM loc_75D4/loc_8218/loc_8BF0: after sub_71E4 and
                // Character_CheckCollision, store y_pos(a3) into $FFFFFA2E.
                // Native only models the Y component; $FFFFFA2C is not consumed
                // by camera logic.
                self.camera_last_kid_y = (slice.player.y_pos >> 16) as i16;
            }
            self.trigger_skycutter_top_boundary_bounce_loc_9114(&slice, pre_physics_y_velocity);
            // ASM sub_AF7A: IronKnight breaks floor blocks on landing.
            // Called from loc_A94E (kid.asm:13239) in the jump-loop landing
            // sequence: `bsr.w sub_AF7A; bne.w loc_A762`. Runs after
            // sub_B000 (Red Stealth floor impact) and before sub_DB22.
            // Checks after physics so we know the player just touched a floor cell.
            // Not-IK or velocity < $48000 (without prior flag) → loc_AFD0
            // (kid.asm:13830): `moveq #0,d7; rts` — no break, return 0.
            if slice.player.on_ground && pre_physics_y_velocity > 0 {
                let px = (slice.player.x_pos >> 16) as i32;
                let py = (slice.player.y_pos >> 16) as i32;
                let hw = self.progression.current_form.half_width();
                let block_list = self.block_list.as_ref();
                if let Some(ref mut grid) = self.block_grid {
                    let red_stealth_impact = helmets::red_stealth_floor_impact_sub_B000(
                        self.progression.current_form,
                        // ASM sub_B270 and sub_B000 both read the same
                        // Red_Stealth_sword_swing byte; native stores the
                        // airborne down-slash phase separately so the renderer
                        // can distinguish it from the ground slash timer.
                        self.helmet.red_stealth_sword_timer > 0
                            || self.helmet.red_stealth_airborne_slash_counter > 0,
                        grid,
                        px,
                        py,
                    );
                    if red_stealth_impact.kind != helmets::RedStealthFloorImpactKindSubB000::None {
                        let block_x = red_stealth_impact.block_col * 16;
                        let block_y = red_stealth_impact.block_row * 16;
                        match red_stealth_impact.kind {
                            // ASM loc_B070 (kid.asm:13904): off_B058 block
                            // type 0 (Smash) handler — `moveq #2,d6; jmp
                            // (j_Block_LoadSmashBits).l`.
                            helmets::RedStealthFloorImpactKindSubB000::Smash => {
                                grid.erase_block_from_level_layout_loc_11542(
                                    red_stealth_impact.block_col,
                                    red_stealth_impact.block_row,
                                );
                                self.block_destroy_effects.push(BlockDestroyEffect {
                                    // ASM loc_10F1A/loc_10F2A: d6=2 nudges
                                    // smash debris 4 px upward, then the
                                    // worker hibernates for 2 frames.
                                    x_px: block_x,
                                    y_px: block_y.saturating_sub(4),
                                    remaining: 2,
                                    direction: Self::BLOCK_DESTROY_DIRECTION_ABOVE,
                                    pending_ice_origin_nudge: false,
                                });
                            }
                            // ASM off_B058 block type 1 (Prize) jumps through
                            // j_loc_1002E before the common rebound tail.
                            helmets::RedStealthFloorImpactKindSubB000::Prize => {
                                Self::start_prize_block_wobble_sub_1007A(
                                    grid,
                                    block_list,
                                    &mut self.block_state.deferred_ceiling_clears,
                                    &mut self.block_bounces,
                                    &mut self.pending_fg_update_cell_queue_sub_1596,
                                    red_stealth_impact.block_col,
                                    red_stealth_impact.block_row,
                                );
                            }
                            // ASM off_B058 block type 2 (Ice) jumps through
                            // IceBlock_SmashAndLoadBullets before rebounding.
                            helmets::RedStealthFloorImpactKindSubB000::Ice => {
                                // ASM off_B058[2] -> IceBlock_SmashAndLoadBullets
                                // (kid.asm:13898, 22849): sub_B000 sets d6=0
                                // before the ice jump-table entry, so sub_11006
                                // remaps it to downward bullets.
                                let target_open = Self::ice_block_bullet_target_cell_sub_11006(
                                    red_stealth_impact.block_col,
                                    red_stealth_impact.block_row,
                                    Self::BLOCK_DESTROY_DIRECTION_BELOW,
                                    grid.width_cells,
                                    grid.height_cells,
                                )
                                .is_some_and(
                                    |(target_col, target_row)| {
                                        Self::ice_block_bullet_target_cell_open_loc_11052(
                                            grid.cell_at(target_col, target_row),
                                        )
                                    },
                                );
                                if target_open {
                                    self.projectiles.extend(Self::ice_block_bullets_loc_11052(
                                        red_stealth_impact.block_col,
                                        red_stealth_impact.block_row,
                                        Self::BLOCK_DESTROY_DIRECTION_BELOW,
                                    ));
                                }
                                grid.erase_block_from_level_layout_loc_11542(
                                    red_stealth_impact.block_col,
                                    red_stealth_impact.block_row,
                                );
                                self.block_destroy_effects.push(BlockDestroyEffect {
                                    // ASM sub_10F9C (kid.asm:22881): d6=0
                                    // falls through to the downward ice-shard
                                    // nudge after the first visible frame.
                                    x_px: block_x,
                                    y_px: block_y,
                                    remaining: 4,
                                    direction: Self::BLOCK_DESTROY_DIRECTION_BELOW,
                                    pending_ice_origin_nudge: true,
                                });
                            }
                            // ASM off_B058 block types 3/4 return directly to
                            // the common `y_vel=-$38000` rebound tail.
                            helmets::RedStealthFloorImpactKindSubB000::BounceOnly => {}
                            // ASM off_B058 block type 5 jumps to loc_110D0:
                            // metal hit effect plus block restoration.
                            helmets::RedStealthFloorImpactKindSubB000::MetalBounce => {
                                slice.metal_block_hit = Some((
                                    red_stealth_impact.block_col,
                                    red_stealth_impact.block_row,
                                    Self::BLOCK_DESTROY_DIRECTION_BELOW,
                                ));
                            }
                            helmets::RedStealthFloorImpactKindSubB000::None => {}
                        }
                        slice.player.y_velocity = red_stealth_impact.bounce_y_velocity;
                        slice.player.on_ground = false;
                    }
                    let probe = terrain::probe_floor_hitbox(grid, px, py, hw);
                    // ASM loc_AF96 (kid.asm:13808): active break path — `st
                    // (Iron_Knight_block_breaker).w`, save regs, `bsr.w
                    // sub_BBE2` to probe for a block, then dispatch through
                    // off_AFD8 jump table.  On hit: `move.l #$FFFF8000,
                    // y_vel(a3); moveq #1,d7; rts`.  No block or block ID
                    // > 5 → loc_AFCA (kid.asm:13826): restore regs, fall
                    // through to loc_AFD0 (`moveq #0,d7; rts`).
                    let probed_cell = (probe.cell_x, probe.cell_y);
                    let same_metal_rebound_cell = self.iron_knight_last_metal_block_hit
                        == Some(probed_cell)
                        && self.iron_knight_block_breaker
                        && probe.on_ground
                        && probe.cell.has_block
                        && probe.cell.block_type == terrain::BlockType::Metal;
                    let collision = if same_metal_rebound_cell {
                        // ASM loc_A94E/sub_DB22: after loc_110D0's tiny
                        // rebound, re-touching the same restored metal cell
                        // settles through the normal grounded clean-up instead
                        // of spawning another sub_11120 worker.
                        blocks::IronKnightBlockBreakerCollisionSubAF7A {
                            next_block_breaker: false,
                            handled_collision: false,
                            next_y_velocity: None,
                            effect: None,
                        }
                    } else {
                        Self::iron_knight_block_breaker_collision_sub_AF7A(
                            self.progression.current_form == PlayerForm::IronKnight,
                            self.iron_knight_block_breaker,
                            pre_physics_y_velocity,
                            probe.on_ground && probe.cell.has_block,
                            probe.cell.block_type,
                        )
                    };
                    self.iron_knight_block_breaker = if collision.handled_collision {
                        collision.next_block_breaker
                    } else {
                        // ASM loc_A94E falls through to sub_DB22 when sub_AF7A
                        // returns zero; sub_DB22 clears Iron_Knight_block_breaker.
                        false
                    };
                    if !self.iron_knight_block_breaker {
                        self.iron_knight_last_metal_block_hit = None;
                    }

                    if let Some(effect) = collision.effect {
                        if effect != blocks::IronKnightBlockEffectSubAF7A::Metal {
                            self.iron_knight_last_metal_block_hit = None;
                        }
                        match effect {
                            // ASM loc_AFF0 (kid.asm:13854): off_AFD8 block
                            // type 0 (Smash) handler — `moveq #2,d6; jmp
                            // (j_Block_LoadSmashBits).l`.
                            blocks::IronKnightBlockEffectSubAF7A::Smash => {
                                grid.erase_block_from_level_layout_loc_11542(
                                    probe.cell_x,
                                    probe.cell_y,
                                );
                                self.spawn_smash_block_effect_sub_10eda(
                                    probe.cell_x * 16,
                                    probe.cell_y * 16,
                                    Self::BLOCK_DESTROY_DIRECTION_ABOVE,
                                );
                                slice.player.y_velocity =
                                    collision.next_y_velocity.unwrap_or(-0x8000);
                                slice.player.on_ground = false;
                            }
                            blocks::IronKnightBlockEffectSubAF7A::Prize => {
                                Self::start_prize_block_wobble_sub_1007A(
                                    grid,
                                    block_list,
                                    &mut self.block_state.deferred_ceiling_clears,
                                    &mut self.block_bounces,
                                    &mut self.pending_fg_update_cell_queue_sub_1596,
                                    probe.cell_x,
                                    probe.cell_y,
                                );
                                slice.player.y_velocity =
                                    collision.next_y_velocity.unwrap_or(-0x8000);
                                slice.player.on_ground = false;
                            }
                            blocks::IronKnightBlockEffectSubAF7A::Ice => {
                                // ASM off_AFD8[2] -> IceBlock_SmashAndLoadBullets
                                // (kid.asm:13845, 22849): Iron Knight ice
                                // breaks keep sub_AF7A's d6=0, then load
                                // bullets through sub_11006 before the
                                // sub_10F9C cosmetic worker is spawned.
                                let target_open = Self::ice_block_bullet_target_cell_sub_11006(
                                    probe.cell_x,
                                    probe.cell_y,
                                    Self::BLOCK_DESTROY_DIRECTION_BELOW,
                                    grid.width_cells,
                                    grid.height_cells,
                                )
                                .is_some_and(
                                    |(target_col, target_row)| {
                                        Self::ice_block_bullet_target_cell_open_loc_11052(
                                            grid.cell_at(target_col, target_row),
                                        )
                                    },
                                );
                                if target_open {
                                    self.projectiles.extend(Self::ice_block_bullets_loc_11052(
                                        probe.cell_x,
                                        probe.cell_y,
                                        Self::BLOCK_DESTROY_DIRECTION_BELOW,
                                    ));
                                }
                                grid.erase_block_from_level_layout_loc_11542(
                                    probe.cell_x,
                                    probe.cell_y,
                                );
                                self.spawn_ice_block_effect_sub_10f9c(
                                    probe.cell_x * 16,
                                    probe.cell_y * 16,
                                    Self::BLOCK_DESTROY_DIRECTION_BELOW,
                                );
                                slice.player.y_velocity =
                                    collision.next_y_velocity.unwrap_or(-0x8000);
                                slice.player.on_ground = false;
                            }
                            // ASM off_AFD8 -> loc_110D0/sub_11120: metal
                            // blocks bounce, play the steel hit effect, then
                            // restore $E50B instead of being deleted.
                            blocks::IronKnightBlockEffectSubAF7A::Metal => {
                                slice.metal_block_hit = Some((
                                    probe.cell_x,
                                    probe.cell_y,
                                    Self::BLOCK_DESTROY_DIRECTION_BELOW,
                                ));
                                self.iron_knight_last_metal_block_hit =
                                    Some((probe.cell_x, probe.cell_y));
                                slice.player.y_velocity =
                                    collision.next_y_velocity.unwrap_or(-0x8000);
                                slice.player.on_ground = false;
                            }
                        }
                    }
                }
            }
            if slice.player.on_ground
                && pre_physics_y_velocity > 0
                && self.progression.current_form == PlayerForm::IronKnight
            {
                // ASM loc_A94E (kid.asm:13236-13250): once sub_B000 and
                // sub_AF7A both return zero, ordinary Iron Knight floor
                // landing queues `sfx_Jump_on_enemy = $4C` before settling.
                self.queue_start_sequence(QueuedSoundEffect::JumpOnEnemy);
            }

            self.note_kid_same_frame_stomp_probe(
                &slice,
                pre_physics_y_velocity,
                pre_physics_y_pos,
                frame_start_x_pos,
                frame_start_y_pos,
            );
            // ASM kid_flip (loc_ACB0): Cyclone flight activation from C press
            // runs AFTER physics so flight takes effect next frame.
            self.step_cyclone_activation(&slice, input);
            if let Some(stages) = slice.kid_flip_follow_on_stages.as_mut() {
                stages.cyclone_acceleration_before_sub_b168 = cyclone_acceleration_before_sub_b168;
            }
            self.note_kid_flip_branch_probe(
                input,
                player_before_iron_knight_climb,
                slice.player,
                &slice,
            );
            self.maybe_hold_wall_cling_boundary_trace_y_velocity(slice.player);
            // ASM loc_833E -> sub_8ED0 checks `Character_Movement` for
            // MoveID_Crawling before choosing the platform-support width.
            // Native has no persistent movement ID, so preserve the post
            // stand-up-check crawl-path decision for the later platform pass.
            platform_support_crawling = movement_input.down && slice.player.on_ground;
            self.queue_hidden_block_reveal_bounces_from_diff(
                hidden_block_cells_before_physics.as_deref(),
            );
        }
        self.finish_deferred_ceiling_clears();
        // Wide spawn-support settling is a bootstrap/respawn helper, not
        // general demo gameplay logic. Keeping it live after the hidden demo
        // settle finishes yanks Kid down onto nearby support during later arcs,
        // which in turn makes the frame-1406 diamond collectible too early.
        let demo_bootstrap_support_active = self
            .demo_startup_frame()
            .is_some_and(|frame| frame <= demo::DEMO_OBJECT_BOOTSTRAP_END_FRAME);
        let spawn_support_snapped = demo_bootstrap_support_active
            && self
                .block_grid
                .as_ref()
                .is_some_and(|grid| self.try_settle_on_spawn_support(&mut slice, grid));
        let demo_bootstrap_floor_snapped = self.try_snap_demo_bootstrap_floor(&mut slice);
        if spawn_support_snapped || demo_bootstrap_floor_snapped {
            slice.update_camera_follow(self.y_look_offset);
        }

        // ASM loc_8218 (kid.asm:8997): crawling state entry — sets
        // `Character_Movement = MoveID_Crawling`, calls sub_71E4 (camera
        // follow), hibernates 1 frame, sets half_width = $E (14px), runs
        // Character_CheckCollision, saves x/y to $FFFFFA2C/$FFFFFA2E, then
        // sub_7ACC (helmet change check). Native: the crawling state is not
        // a separate movement ID — down-held+grounded is handled inline.
        //
        // ASM loc_825C (kid.asm:9018): crawl continuation — if down still
        // held (`tst.b (Ctrl_Down_Held)`), branch to loc_82D8 (crawl loop).
        // If released, `subi.w #8,($FFFFFB58)` decrements the look offset,
        // clamped to 0, then falls through to loc_8270 (stand-up check).
        //
        // ASM loc_8C12 (kid.asm:10051): walking-state version —
        // `tst.b (Ctrl_Down_Held); bne.s loc_8C26` skips decrement when down
        // held; `subi.w #8,($FFFFFB58); bge.s loc_8C26` decrements, clamp to 0.
        //
        // ASM loc_7772/loc_82D8/loc_8C70: grounded Down-held paths run
        // `addq.w #1,($FFFFFB58)` with no cap only while `KidGrabbedByHand`
        // is clear; sub_71E4 loc_728C treats `$1E` as the camera-shift
        // threshold, not a maximum accumulator.
        // ASM loc_7650 (kid.asm:7741-7746): same offset decay in other
        // movement states. Native merges the increment/decay paths here.
        if input.down {
            if slice.player.on_ground && !slice.kid_grabbed_by_hand {
                self.y_look_offset += 1;
            }
        } else if self.y_look_offset > 0 {
            self.y_look_offset = (self.y_look_offset - 8).max(0);
        }

        // ASM loc_8270 (kid.asm:9025): stand-up from crawl — `bsr.w
        // sub_DAA6` checks for platform overhead (returns NE → can't stand),
        // `moveq #-$11,d7; bsr.w sub_B43A` floor probe, Skycutter gate,
        // then `bsr.w sub_81B4` wall escape. Native: physics handles floor
        // probe implicitly; sub_81B4 is crawl_wall_escape_sub_81B4;
        // sub_DAA6 is crawl_standup_platform_overhead_sub_DAA6, which keeps
        // the physics input on the crawl path for the blocked stand-up frame.
        //
        // ASM loc_8298 (kid.asm:9038): when sub_B43A finds floor (EQ),
        // `bsr.w sub_B41C` restores the standing half_width from unk_B408
        // (e.g. TheKid=5, Skycutter=10). Falls through to loc_829C which
        // clears Button_B and dispatches: Skycutter → walking (sub_942A →
        // loc_8BF0), others → loc_82BA.
        // Native: collision half_width always comes from
        // `PlayerForm::half_width()`, so the sub_B41C restore is implicit.
        //
        // ASM loc_82BA (kid.asm:9050): non-Skycutter path after stand-up —
        // `tst.l x_vel(a3)`: if non-zero, `bsr.w sub_78E8` then `bra.w
        // loc_75D4` (standing loop); if zero → loc_82CA.
        // Native: `refresh_grounded_helmet_state_sub_78E8` runs every
        // grounded frame (below), covering both x_vel paths.
        //
        // ASM loc_82CA (kid.asm:9057): zero-velocity idle entry from crawl
        // stand-up — `clr.w ($FFFFF8F0)` (walk accumulator), `bclr
        // #Button_B,(Ctrl_Pressed)`, `bra.w loc_8E3E`. Visual-only: the
        // walk accumulator and sprite dispatch at loc_8E3E are not modelled.
        //
        // Berzerker charge: break Smash blocks on wall contact
        if self.helmet.berzerker_charging {
            self.berzerker_break_wall_block(&slice);
        }

        // Step platform movement scripts
        if let Some(ref script_table) = self.platform_script_table {
            platforms::execute_scripted_platforms_sub_23EA(
                &mut self.active_platforms,
                script_table,
                matches!(self.player_lifecycle, PlayerLifecycle::DeathWindup { .. }),
            );
        }
        let platform_standing_before_index = standing_platform_index_loc_8ED0(
            &self.active_platforms,
            self.active_platform_standing_on,
            slice.player,
            self.progression.current_form,
            platform_support_crawling,
        );
        let platform_standing_before = platform_standing_before_index
            .and_then(|platform_index| self.active_platforms.get(platform_index))
            .and_then(|platform| platform.slot_addr);
        let kid_y_px = (slice.player.y_pos >> 16) as i16;
        let active_platforms_before_special_step = self.active_platforms.clone();
        let mut stepped_platforms = Vec::with_capacity(self.active_platforms.len());
        for mut platform in self.active_platforms.iter().copied() {
            let action = platform.step_special_platform_loc_40D2(
                platform
                    .slot_addr
                    .is_some_and(|slot_addr| self.active_platform_standing_on == Some(slot_addr)),
                kid_y_px,
                platform.deactivated_flag,
            );
            if action == SpecialPlatformStepActionLoc40D2::Delete {
                if let Some(layout_index) = platform.layout_index {
                    // ASM TrapPlatformDown_Delete / TrapPlatformUp_Delete
                    // tail-calls Deallocate_PlatformSlot before
                    // Delete_CurrentObject. Mirror that through the live
                    // PlatformLoaderState slot model so the freed slot becomes
                    // the next Allocate_PlatformSlot head.
                    slice
                        .objects
                        .platform_loader
                        .deallocate_layout_offset_sub_23B4(
                            platform_loader_offset_for_index_loc_286E(layout_index),
                        );
                }
            } else {
                stepped_platforms.push(platform);
            }
        }
        self.active_platforms = stepped_platforms;
        // ASM Addr_PlatformStandingOn is a slot address; after native vector
        // compaction from loc_2850/delete actions, preserve that identity.
        let platform_standing_before = remap_standing_platform_addr_by_layout_index_loc_2850(
            platform_standing_before,
            &active_platforms_before_special_step,
            &self.active_platforms,
        );
        self.active_platform_standing_on = platform_standing_before;

        // Platform collision: player can stand on top of platforms
        let player_before_platform_collision = slice.player;
        let character_movement_is_jump = self.current_character_movement(slice.player) == 3;
        let platform_collision_result = apply_platform_collision_with_ceiling_probe_trace_loc_257E(
            &mut slice,
            &self.active_platforms,
            self.block_grid.as_ref(),
            self.progression.current_form,
            platform_support_crawling,
            platform_standing_before,
            character_movement_is_jump,
        );
        self.last_platform_collision_scratch_loc_257e = platform_collision_result.scratch;
        let platform_collision_outcome = platform_collision_result.outcome;
        if self.last_kid_frame_post_motion_mutator == KidFramePostMotionMutator::None
            && slice.player != player_before_platform_collision
        {
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::PlatformCollision;
        }
        match platform_collision_outcome {
            PlatformCollisionOutcome::CrushDeath => {
                self.opening_slice = Some(slice);
                self.begin_death_windup();
                return;
            }
            PlatformCollisionOutcome::StandingOn(platform_addr) => {
                self.active_platform_standing_on = Some(platform_addr);
            }
            PlatformCollisionOutcome::None => {
                self.active_platform_standing_on = standing_platform_index_loc_8ED0(
                    &self.active_platforms,
                    self.active_platform_standing_on,
                    slice.player,
                    self.progression.current_form,
                    platform_support_crawling,
                )
                .and_then(|platform_index| self.active_platforms.get(platform_index))
                .and_then(|platform| platform.slot_addr);
            }
        }
        self.exit_iron_knight_climb_on_platform_landing_loc_A9AA(
            player_before_platform_collision,
            &slice,
        );
        self.trigger_standing_special_platform_loc_40D2(&slice, platform_support_crawling);
        self.active_platform_roster = self
            .active_platforms
            .iter()
            .map(ActivePlatform::snapshot)
            .collect();

        // Evanescent blocks: trigger when player stands on one, tick timers,
        // and clear expired blocks from the grid.
        self.step_evanescent_blocks(&slice);

        // Lift platforms: step movement and collision
        let player_before_lift_platforms = slice.player;
        let lift_collision_outcome = self.step_lift_platforms(&mut slice);
        if self.last_kid_frame_post_motion_mutator == KidFramePostMotionMutator::None
            && slice.player != player_before_lift_platforms
        {
            self.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::LiftPlatform;
        }
        match lift_collision_outcome {
            PlatformCollisionOutcome::CrushDeath => {
                self.opening_slice = Some(slice);
                self.begin_death_windup();
                return;
            }
            PlatformCollisionOutcome::StandingOn(_) | PlatformCollisionOutcome::None => {}
        }

        // ASM loc_787E (kid.asm:7924): when the standing-still platform block
        // (loc_7828) clears Addr_PlatformStandingOn and x_vel is zero, the ROM
        // calls sub_78E8 then returns to loc_75D4.
        // ASM loc_78C6 (kid.asm:7957): when Addr_PlatformStandingOn is null at
        // loc_7828, the ROM zeroes x_vel/y_vel, runs sub_B43A floor check:
        //   bmi → loc_BC80 (no floor, death/fall state)
        //   beq → loc_B5F6 (no solid/spikes cell, drop 1px)
        //   positive → sub_78E8 + loop back to loc_75D4 (standing-still).
        // Native: grounded flag already set; velocity is zero from the movement
        // system; sub_78E8 runs unconditionally below.
        // ASM loc_789C (kid.asm:7937): when sub_902A finds floor and sub_8ED0
        // confirms platform support, the ROM loads $FFFFFA98 into d0 (dead —
        // immediately overwritten by sub_78E8's `move.w (Current_Helmet),d0`),
        // calls sub_78E8, and returns to loc_75D4.
        // Native runs sub_78E8 unconditionally each grounded frame, covering
        // the "platform gone" (loc_787E/loc_78C6) and "platform supported"
        // (loc_789C) paths.
        let mounted_skycutter_board_landing = self.progression.current_form
            == PlayerForm::Skycutter
            && self.helmet.skycutter_on_board;
        if !frame_start_on_ground && slice.player.on_ground && !mounted_skycutter_board_landing {
            // ASM loc_A94E (kid.asm:13236), loc_A9BE (kid.asm:13274), and
            // loc_AB0A/loc_AB4A (kid.asm:13372/13394) clear transient helmet
            // motion state through sub_DB22 before grounded movement resumes.
            // Mounted Skycutter board landings take the `$FA6A` loop/bounce
            // path instead, so keep that state for the board handler.
            if self.progression.current_form == PlayerForm::Cyclone && self.helmet.cyclone_flying {
                // ASM loc_AB0A/loc_AB4A falls through to loc_8BF0 without
                // calling sub_942A, so the prior `$FFFFF8F0 >> 8` walk entry
                // remains the Sprite_Table source. Oracle `$FF2502/$FF0502`
                // captures identify walk 3 for phase $029B and walk 1 for
                // phase $005A on the two published-route landing families.
                self.helmet.cyclone_grounded_sprite_source =
                    Some(CycloneGroundedSpriteSource::from_walk_visual_accumulator(
                        frame_start_walk_visual_accumulator,
                    ));
            }
            self.clear_special_movement_state_sub_DB22();
        }
        if slice.player.on_ground {
            self.refresh_grounded_helmet_state_sub_78E8();
        } else {
            // ASM sub_942A loc_9454: airborne frames run the moving/airborne
            // dispatch, which clears Maniaxe_throwing_axe on the throw-anim-done
            // flag just like the grounded sub_78E8 loc_794C path. Mutually
            // exclusive with the grounded branch above (post-physics on_ground),
            // so the throw latch decrements exactly once per frame (#337).
            self.tick_maniaxe_airborne_throw_latch_loc_9454();
        }

        // ASM loc_7FC (kid.asm:915): `jsr (j_sub_F7E0).l; bsr.w sub_6034`
        // Mode_Level mid-loop calls shooter slot tick (sub_F730 via the
        // j_sub_F7E0 trampoline) then dispatches level special effects
        // (storm/hail weather via sub_6034). Both are skipped when
        // $FFFFFB49 is set (paused); native gates on gameplay_slice_active.
        //
        // ASM loc_80C (kid.asm:922): pause-skip resumes the gameplay tail at
        // collision setup, enemy loading, object motion, input, platform
        // scripts/collision, then Kid/object collision. Native covers that resumed
        // Mode_Level tail inside refresh_opening_slice before these shooter
        // and weather follow-on workers.
        //
        // ASM loc_806 (kid.asm:919): `jsr (j_sub_F096).l` — shooter block
        // scan. Detects new shooter blocks entering Kid's ±3-col / -5..+2-row
        // window when Kid crosses a block boundary, registers them for the
        // 22-frame fire-and-expire cycle driven by sub_F730. Native merges
        // both scan (sub_F096) and tick (sub_F730) into step_shooter_blocks.
        self.step_shooter_blocks(&slice);

        // Queue the full prize-block lifecycle from a headbutt. The tile stays
        // Solid (0x6F) until the coroutine completes, then the same state
        // record clears the tile and spawns the prize object.
        if let Some(prize_clear) = slice.ceiling_prize_clear
            && let Some((content, _, _)) = slice.prize_block_hit
        {
            if let Some(grid) = &self.block_grid {
                let queued_cell = grid.erase_cell_at_loc_11542(prize_clear.col, prize_clear.row);
                // ASM sub_1007A (kid.asm:21369-21375) queues the restored
                // terrain word before replacing the live cell with `$6Fxx`.
                self.pending_fg_update_cell_queue_sub_1596.push((
                    prize_clear.col,
                    prize_clear.row,
                    queued_cell,
                ));
            }
            self.block_state
                .deferred_ceiling_clears
                .push(DeferredCeilingClear {
                    col: prize_clear.col,
                    row: prize_clear.row,
                    content,
                    restore_cell: prize_clear.restore_cell,
                    reveal_only: prize_clear.reveal_only,
                    deleted_frames_remaining: Self::PRIZE_BLOCK_DELETED_FRAMES,
                    phase: DeferredPrizeBlockPhase::Deleted,
                });
            // ASM sub_1007A (kid.asm:21345-21599) drives the prize-block
            // headbutt wobble for 31 frames with the delta pattern in
            // PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A. The engine computes the
            // cumulative y_offset each frame and the renderer applies it to
            // the block draw, replacing the earlier 8-frame placeholder arc.
            let layout_word_addr = self.block_grid.as_ref().map_or(0, |grid| {
                u32::from(terrain::BlockList::level_layout_low_word_addr(
                    prize_clear.col,
                    prize_clear.row,
                    grid.width_cells,
                ))
            });
            let headbutt_worker = Self::prize_block_headbutt_worker_loc_1002E(
                prize_clear.col,
                prize_clear.row,
                layout_word_addr,
            );
            self.block_bounces.push(BlockBounceAnim {
                cell_x: headbutt_worker.gfx_x_pos >> 4,
                cell_y: headbutt_worker.gfx_y_pos >> 4,
                x_offset_px: 0,
                remaining: 31,
                initial_duration: 31,
                kind: BlockBounceKind::Prize,
                prize_content: Some(content),
                prize_deleted_tile: prize_clear.reveal_only,
                metal_direction: 0,
                y_offset_px: 0,
            });
        }

        self.process_block_effects(&mut slice);
        self.collect_floating_items(&mut slice);
        // ASM loc_104AE/loc_105D6: the PrizeHelmet worker wakes AFTER the Kid
        // in the same Execute_Objects pass, one hibernate frame after its
        // latch, so a latched helmet converts only after the Kid's movement
        // for the wake frame has run (#420).
        self.award_collision_latched_prize_loc_104ae();
        // ASM loc_75D4 (kid.asm:7715): `bsr.w sub_7ACC` — check pending
        // helmet transform after collision processing.  In native, this runs
        // for all movement states (the ROM calls it from each state handler).
        if self.handle_pending_form_change_sub_7acc(&mut slice) {
            return;
        }
        if self.gameplay_hibernating_for_transform() {
            // ASM Kid_Transform: the worker-wake takeover suspends the rest
            // of the Mode_Level frame — the weather, lava, and palette
            // rotation ticks below do not run on the wake frame (published
            // TAS pixel baseline pins the rotation phase across transforms,
            // #420).
            self.last_kid_frame_motion_branch = KidFrameMotionBranch::TransformHibernating;
            self.opening_slice = Some(slice);
            return;
        }

        // Weather: hail spawns falling projectiles, lava geysers erupt
        self.island_shore_dma_uploads_loc_1202a.clear();
        self.lava_geyser_art_uploads_loc_14c6a.clear();
        self.step_weather(&slice);
        self.step_lava_geysers(&slice);
        self.step_rotate_palette_lava_water_loc_1223c();
        self.step_island_shore_cycle_loc_1202a();

        if self.resolve_frame_hazards(
            &mut slice,
            input,
            pre_physics_y_velocity,
            pre_physics_y_pos,
            frame_start_x_pos,
            frame_start_y_pos,
        ) {
            return;
        }
        if slice.player.on_ground {
            // ASM landing paths clear `$FFFFFA67` through sub_DB22 before
            // grounded animation resumes. The launch threshold at `$FAAA`
            // remains stale, so it must not classify a later ledge walk-off.
            self.helmet.airborne_mapping_jump_sequence_active = false;
        }
        self.opening_slice = Some(slice);
    }

    #[allow(non_snake_case)]
    fn exit_iron_knight_climb_on_platform_landing_loc_A9AA(
        &mut self,
        player_before_platform_collision: PlayerState,
        slice: &OpeningSliceState,
    ) {
        if self.progression.current_form != PlayerForm::IronKnight
            || !self.helmet.iron_knight_climbing
            || player_before_platform_collision.on_ground
            || !slice.player.on_ground
            || standing_platform_index_loc_8ED0(
                &self.active_platforms,
                self.active_platform_standing_on,
                slice.player,
                self.progression.current_form,
                false,
            )
            .is_none()
        {
            return;
        }

        // ASM loc_A9AA: a platform object wrote Addr_PlatformStandingOn during
        // the wall-climb hibernate frame; with Skycutter's board flag clear,
        // loc_A9BE leaves wall climb for the platform-walking path.
        self.helmet.iron_knight_climbing = false;
        self.helmet.iron_knight_climb_accel = 0;
        self.helmet.iron_knight_climb_pose_accum = 0;
    }

    #[allow(non_snake_case)]
    fn trigger_standing_special_platform_loc_40D2(
        &mut self,
        slice: &OpeningSliceState,
        platform_support_crawling: bool,
    ) {
        if let Some(platform_index) = standing_platform_index_loc_8ED0(
            &self.active_platforms,
            self.active_platform_standing_on,
            slice.player,
            self.progression.current_form,
            platform_support_crawling,
        ) && let Some(platform) = self.active_platforms.get_mut(platform_index)
        {
            // ASM SoftPlatform_WaitLoop loc_40E2 (kid.asm:5206): `move.l
            // $2A(a2),d1` samples Kid's y velocity before the platform
            // hibernates for this frame, so use the native pre-gravity sample
            // rather than the post-platform-collision velocity.
            platform.trigger_special_platform_standing_loc_40D2(
                slice.pre_gravity_y_vel,
                self.progression.current_form,
            );
        }
    }

    #[cfg(test)]
    #[allow(non_snake_case)]
    fn trigger_standing_soft_platform_bounce_loc_40E2(
        &mut self,
        slice: &OpeningSliceState,
        platform_support_crawling: bool,
    ) {
        self.trigger_standing_special_platform_loc_40D2(slice, platform_support_crawling);
    }

    pub(crate) fn trigger_skycutter_top_boundary_bounce_loc_9114(
        &mut self,
        slice: &OpeningSliceState,
        pre_physics_y_velocity: i32,
    ) {
        if self.progression.current_form != PlayerForm::Skycutter
            || !self.helmet.skycutter_on_board
            || pre_physics_y_velocity >= 0
            || slice.player.y_velocity != 0x0003_0000
        {
            return;
        }

        let y_px = (slice.player.y_pos >> 16) as i32;
        if y_px != PlayerForm::Skycutter.ceiling_probe_height() {
            return;
        }

        // ASM loc_9114 (kid.asm:10524): mounted Skycutter above-level ceiling
        // contact stores `move.b #1,($FFFFFA69)` before the `$30000` downward
        // velocity. Native movement owns the velocity; the runtime shell owns
        // the helmet latch that drives the flipboard coroutine.
        self.helmet.skycutter_flipboard = 1;
    }
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod tests;
