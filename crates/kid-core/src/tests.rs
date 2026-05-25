use super::*;
use crate::blocks::{
    FloatingPrizeFallStepLoc10C50, LiftBlockFallStepLoc1141E, LiftBlockRiseStepLoc113F4,
    OneTimePrizeEntryCheckLoc10876, OneTimePrizeInitLoc10856, OneTimePrizeScanStartLoc1086A,
    OneTimePrizeScanStepLoc10886, OneTimePrizeStoreLoc1089A, Prize10DiamondsInitLoc108E8,
    Prize10DiamondsSpawnLoc108EE, Prize10DiamondsWorkerCountdownGateLoc10984,
    Prize10DiamondsWorkerInitLoc10988, Prize10DiamondsWorkerScanStartLoc10960,
    Prize10DiamondsWorkerStepLoc109C0, PrizeClockCollectedWorldStepLocFF92,
    PrizeClockCountdownLoc10796, PrizeClockFallGateLoc107B0, PrizeClockSoundCleanupLoc107E0,
    PrizeClockSoundRefcountLoc10792, PrizeCollectedArcHibernateStepLocFEEE,
    PrizeCollectedWorldStepLocFEFE, PrizeContinueCountdownLoc1080A, PrizeContinueDeleteLoc10838,
    ShooterCenterWorkerGateLoc1146E, ShooterLeftSupportProbeLoc11448,
};
use crate::boot::{
    FlagHudDmaUploadSub44DC, HudAnimationStateSub44DC, HudDisplayNumberTilesCalcDisplayNumber,
    HudFrameUploadSub44DC, HudHitpointSpriteSubBDC0, HudTimerDigitTilesLocBEE8,
    HudTimerSpriteLayoutSubBD0A, HudTimerSpritePositionSubBD0A, IntroSequence6LongwordFillLoc1B170,
    IntroSequence6WordFillLoc1B182, IoPortFlagsSub6E24,
    PAUSE_MENU_ART_COPY_BYTES_PER_ITERATION_LOC_690C, PAUSE_MENU_TILE_SAVE_ROW_COUNT_LOC_695C,
    PAUSE_MENU_Y_OFFSET_LOC_6948, PauseMenuGateLoc687C, PauseMenuPlaneColumnLayout,
    PauseMenuSpriteEntrySub6874, PauseMenuTextSelectionRestartText,
    RAM_CLEAR_LONGWORD_COUNT_LOC_5FE, RAM_CLEAR_START_ADDR_LOC_5FE, calc_display_number,
    copy_sega_screen_palette_loc_12DFE, end_sequence_burst_camera_loop_loc_1D1EC,
    end_sequence_burst_prelude_loc_1D1D6, end_sequence_burst_sprite_init_sub_1D86C,
    end_sequence_burst_wait_loop_loc_1D1DE, end_sequence_camera_shake_sub_1D846,
    end_sequence_display_refresh_sub_1D830, end_sequence_falling_sprite_init_sub_1D8B4,
    flag_hud_dma_upload_sub_5DA6, hide_pause_menu_overwritten_sprites_sub_6874,
    hud_lives_clamp_loc_BF2A, hud_timer_digit_tiles_loc_BEE8,
    hud_timer_low_time_check_end_decrease_time_left, hud_timer_sprite_layout_sub_BD0A,
    init_hud_animation_frames_sub_44B0, intro_sequence_6_longword_fill_loc_1B170,
    intro_sequence_6_striped_tile_fill_loc_1B194, intro_sequence_6_tilemap_clear_loc_1B182,
    is_boss_level_loc_C03A, make_hud_hitpoint_layout_sub_BDC0, pause_menu_art_total_bytes_loc_690C,
    pause_menu_gate_loc_687C, pause_menu_plane_column_layout_loc_693A,
    pause_menu_text_selection_restart_text, pause_menu_text_write_plan_loc_69D2,
    pause_menu_text_write_row_vdp_write_cmd_loc_6A1E, pause_menu_tile_restore_plan_loc_6B68,
    pause_menu_tile_save_plan_loc_695C, pause_menu_tile_save_row_vdp_read_cmd_loc_695C,
    pause_menu_tile_save_wrap_column_reset_loc_69AE, prize_10000_points_star_advance_loc_10BCE,
    prize_10000_points_star_cleanup_loc_10BE6, prize_10000_points_star_dispatch_loc_10B76,
    prize_10000_points_star_latch_screen_pos_loc_10B9A, prize_10000_points_star_spawn_loc_10B3A,
    prize_10000_points_star_step_loc_10B74, read_io_port_a10001_flags_sub_6E24,
    sega_screen_object_bootstrap_loc_12E48, sega_screen_tile_strip_sub_12E3C, sin_cos_sub_40236,
    step_hud_animation_sub_44DC, title_shell_art_decompress_loc_1BF54,
    title_shell_art_index_loc_1BF3E, title_shell_vram_fill_loc_1BF70,
    validate_options_after_ram_clear_loc_5FE, vdp_game_init_register_commands_loc_5E6,
    vdp_upload_32_longwords_sub_44D0,
};
use crate::combat::*;
use crate::helmets::{
    IronKnightClimbVisualSubA3BC, JuggernautCompanionStateSub975C, JuggernautCompanionStepSub975C,
    JuggernautCompanionVisualSub975C, KidAirborneVisualInputSubB270, KidAirborneVisualKindSubB270,
    RedStealthFloorImpactKindSubB000, SkycutterVisualStateSub98F2,
    iron_knight_climb_visual_sub_A3BC, juggernaut_companion_step_sub_975C,
    kid_airborne_visual_sub_B270, red_stealth_floor_impact_sub_B000,
    red_stealth_swing_anchor_sub_868E, skycutter_visual_step_sub_98F2,
};
use crate::level::{
    BONUS_DIGIT_CLEAR_BASE_LOC_C328, BONUS_DIGIT_CLEAR_ROW_COUNT_LOC_C328,
    BackgroundArtDecompressLoc11EA2, BackgroundArtPermutationLoc11E8A,
    BackgroundFollowupArtDecompressLoc11EC4, BackgroundFollowupArtDecompressLoc11EF2,
    BackgroundFollowupArtPermutationLoc11EC4, BackgroundFollowupArtSourceLoc11EC4,
    BackgroundPaletteFollowupLoc11DF8, BackgroundPaletteOverrideLoc11DF4,
    BackgroundScrollDesertHeatRippleSetupLoc306D4,
    BackgroundScrollDesertHeatRippleVisibleBandLoc306C2,
    BackgroundScrollEnigmaPrimaryCopySetupLoc30426, BackgroundScrollEnigmaRowSourceLoc303D4,
    BackgroundScrollEnigmaRowTableLookupLoc303EA, BackgroundScrollEnigmaRowTransferLoc303D4,
    BackgroundScrollForestWaterRippleSetupLoc305E6,
    BackgroundScrollForestWaterRippleVisibleBandLoc305D4, BackgroundScrollLowerPassLoopLoc30066,
    BackgroundScrollLowerPassRendererLoc30066, BackgroundScrollLowerPassResumeLoc30054,
    BackgroundScrollLowerPassSetupLoc3001E, BackgroundScrollPieceRefreshDualWrapBranchLoc30904,
    BackgroundScrollPieceRefreshLinearCopyLoc308A6,
    BackgroundScrollPieceRefreshRowWrapFinalCopyLoc30968,
    BackgroundScrollPieceRefreshRowWrapFirstCopyLoc30942,
    BackgroundScrollPieceRefreshRowWrapSecondCopyLoc30954,
    BackgroundScrollPieceRefreshSeedLoc3081A,
    BackgroundScrollPieceRefreshTileWrapCounterSetupLoc308DE,
    BackgroundScrollPieceRefreshTileWrapFirstCopyLoc30988,
    BackgroundScrollPieceRefreshTileWrapWrappedCopyLoc3099E,
    BackgroundScrollPieceRefreshTileWrappedCopyLoc308E8,
    BackgroundScrollPieceRefreshVdpSeedLoc30884, BackgroundScrollPieceRefreshWindowSplitLoc3086A,
    BackgroundScrollPieceRefreshWrapCounterSetupLoc308B6, BackgroundScrollPieceRefreshWrapLoc30836,
    BackgroundScrollPieceRefreshWrappedCopyLoc308C4, BackgroundScrollPieceRowDispatchLoc3079A,
    BackgroundScrollPieceRowDispatchLoc3079ABranch, BackgroundScrollPieceRowUploadLoc307B2,
    BackgroundScrollShiftDataSecondPassAdvanceLoc304A8,
    BackgroundScrollShiftDataSecondPassSlotLoc3048C, BackgroundScrollShiftDataSlotLoc30452,
    BackgroundScrollShiftDataWrapLoopBodyLoc304E0, BackgroundScrollShiftDataWrapLoopLoc304C2,
    BackgroundScrollShiftDataWrapLoopLoc304D8, BackgroundScrollShiftDataWrapStoreLoc304CA,
    BackgroundScrollShiftLayersLeadingPartialFillLoc30562,
    BackgroundScrollShiftLayersNoScrollFillLoc30502, BackgroundScrollShiftLayersSetupLoc3050A,
    BackgroundScrollShiftLayersTrailingPartialFillLoc30592,
    BackgroundScrollStripeHorizontalGateLoc3023C,
    BackgroundScrollStripeHorizontalLayoutSeedLoc3026C,
    BackgroundScrollStripeHorizontalLowerWindowLoc3028E,
    BackgroundScrollStripeHorizontalPassLoc30296,
    BackgroundScrollStripeHorizontalRowTransferLoc302FE,
    BackgroundScrollStripeHorizontalRowTransferLoc3031A,
    BackgroundScrollStripeHorizontalRowTransferSource, BackgroundScrollStripeLayoutSeedLoc301D8,
    BackgroundScrollStripeRightWindowLoc301F6, BackgroundScrollStripeRowSelectionLoc301CE,
    BackgroundScrollStripeVerticalColumnTransferLoc3037C,
    BackgroundScrollStripeVerticalColumnTransferLoc30354,
    BackgroundScrollStripeVerticalPassLoc301FE, BackgroundScrollThemeDispatchLoc30598,
    BackgroundScrollUpperPassLoc2FFE8, BackgroundScrollUpperPassResumeLoc30014,
    BlockLayoutDecodeWordsLoc1354E, BlockLayoutDispatchLoc132D6, BlockLayoutDispatchLoc13504,
    BlockLayoutEntryWordsLoc133B2, BlockLayoutEntryWordsLoc133F0, BlockLayoutEntryWordsLoc1332E,
    BlockLayoutEntryWordsLoc1335E, BlockLayoutEntryWordsLoc13470, BlockLayoutLoopLoc133A0,
    BlockLayoutLoopLoc133DE, BlockLayoutLoopLoc1331E, BlockLayoutLoopLoc1342E,
    BlockLayoutLoopLoc1345E, BlockLayoutLoopLoc1347C, BlockLayoutPropertyWordLoc1337C,
    BlockLayoutRepeatWordsLoc1351E, BlockLayoutRepeatWordsLoc13514, BlockLayoutTerminateLoc134BA,
    DecompressToRamChunkSetupLoc143EA, DecompressToRamPaletteRowStartLoc143CC,
    DecompressToVramSpecialReturnLoc143B2, EOL_FIXED_PRE_COUNTER_FRAMES_LOC_D770,
    EOL_NUMBER_DIGIT_COUNT_LOC_C48C, EOL_NUMBER_DIGIT_COUNT_LOC_C466,
    EOL_NUMBER_DIGIT_SPACING_LOC_C4E2, EOL_NUMBER_DISPLAY_FLAG_LOC_C466,
    EOL_NUMBER_GFX_SPEC_LOC_C466, EOL_NUMBER_INITIAL_PATTERN_LOC_C466,
    EOL_NUMBER_MAX_VALUE_LOC_C48C, EOL_NUMBER_PALETTE_LOC_C466, EOL_NUMBER_PRIORITY_LOC_C466,
    EOL_NUMBER_SLOT_ARRAY_OFFSET_LOC_C466, EOL_POST_ROWS_PRE_COUNTER_FRAMES_LOC_D770,
    EOL_PRE_SCOREBOARD_FADE_FRAMES_LOC_C0CC, EOL_SCORE_BOARD_BGM_WAIT_FRAMES_LOC_C21E,
    EOL_SCORE_ROWS_START_FIXED_FRAMES_LOC_D468, EnemyLayoutResetAndSeekInputLoc31F8E,
    EnemyLayoutResetAndSeekLoc31F8E, FLAGPOLE_FADE_PALETTE_0_MASK_LOC_C1DC,
    FLAGPOLE_FADE_PALETTE_1_MASK_LOC_C1DC, FLAGPOLE_FADE_STEP_LOC_C1D0,
    FLAGPOLE_HOLO_PALETTE_DEST_OFFSET_LOC_C1BC, FLAGPOLE_HOLO_PALETTE_WORD_COUNT_LOC_C1BC,
    FLAGPOLE_HOLO_TILEMAP_COLS_LOC_C188, FLAGPOLE_HOLO_TILEMAP_PAD_LOC_C190,
    FLAGPOLE_HOLO_TILEMAP_ROWS_LOC_C186, FLAGPOLE_PALETTE_BACKUP_COUNT_LOC_C1AA,
    FLAGPOLE_SCOREBOARD_BUDGET_PER_FRAME_LOC_C21E, FLAGPOLE_SCREEN_HEIGHT_OFFSET_LOC_C1EE,
    FLAGPOLE_SCREEN_WIDTH_OFFSET_LOC_C1EE, FgPlaneCustomWriteSub1596,
    FgUpdateQueueCullAdvanceLoc167A, FgUpdateQueueCustomEntrySub1596, FgUpdateQueueResetLoc1680,
    FlagpoleScoreboardCursorSubC254, FlagpoleScoreboardStepSubC254,
    ForegroundArtDecompressLoc11E60, ForegroundArtPermutationLoc11E48,
    ForegroundPaletteOverrideLoc11D8E, FullscreenPaletteFadeSub73D0,
    IslandPaletteCycleLowerBandLoc12188, IslandPaletteCycleMidBandLoc12110,
    IslandPaletteCyclePrimaryBandLoc12098, LateBackgroundPaletteOverrideLoc11E06,
    LevelCameraBootstrapLoc11F48, LinkedObjectSlotInit, ManageEnemyLoadingCountDownActiveLoc3685E,
    ManageEnemyLoadingSpawnObjectLoc3691E, ManageEnemyLoadingStatusBranchLoc36840,
    POWERS_OF_TEN_LOC_C48C, PaletteFadeOutInitLoc28A6, PaletteFadeOutStepLoc28C6,
    PostLoadObjectRoutineLoc11F96, RotatePaletteLavaWaterDelayLoc1223A,
    SCOREBOARD_DIGIT_CLEAR_BASES_LOC_C2F4, SCOREBOARD_DIGIT_CLEAR_HIBERNATE_FRAMES_LOC_C2F4,
    SCOREBOARD_DIGIT_CLEAR_ROW_COUNT_LOC_C2F4, SCOREBOARD_DIGIT_CLEAR_STEP_COUNT_LOC_C2F2,
    SPRITE_SCATTER_CENTRE_LOC_C436, SPRITE_SCATTER_ENTRY_COUNT_LOC_C428,
    SPRITE_SCATTER_ENTRY_STRIDE_LOC_C428, SPRITE_SCATTER_TABLE_X_OFFSET_LOC_C428,
    SPRITE_SCATTER_VISIBLE_MAX_LOC_C436, SPRITE_SCATTER_VISIBLE_MIN_LOC_C436, SUB_28FC_SLOT_COUNT,
    SUB_292E_SLOT_ACTIVE_OFFSET, ScratchDebrisPieceSub292E, ScratchDebrisRenderSub2A4C,
    ScratchDebrisSpriteSub2A4C, SharedVramArtDecompressLoc11EFA, SharedVramArtSourceLoc11EFA,
    ShooterObjectSlotsSubF06A, SpriteFrameStateSubB52, TILE_FADE_BUFFER_LONGWORDS_LOC_C38A,
    TILE_FADE_COLUMN_TABLE_UNK_C418, TILE_FADE_HIBERNATE_FRAMES_LOC_C392,
    TILE_FADE_MASK_COLUMNS_SUB_C3DA, TILE_FADE_STEP_COUNT_LOC_C392,
    TILE_FADE_VRAM_ADDRESS_SUB_C378, TILE_FADE_WRITE_ITERATIONS_LOC_C3BA,
    TOUCH_FLAGPOLE_CLEAR_OFFSETS_LOC_C056, TOUCH_FLAGPOLE_GFX_CLEANUP_ENTRY_LOC_C0B6,
    TOUCH_FLAGPOLE_HIBERNATE_FRAMES_LOC_C070, TOUCH_FLAGPOLE_VDP_CMDS_LOC_C070,
    TransferScrollDataDispatchLoc3039C, TransferScrollDataDmaTailLoc300D2,
    TransferScrollDataPreludeLoc3038A, TransferScrollDataRendererLoc3039C,
    TransferScrollDataStormLoc30110, TransferScrollDataStormLoopLoc30130,
    TransferScrollDataStormLoopLoc30172, TransferScrollDataStormSetupLoc30158,
    TransferScrollDataVsramSetupLoc300BA, VdpVramWriteCmd,
    apply_background_palette_override_loc_11E04, apply_background_palette_override_loc_11E12,
    apply_foreground_palette_override_loc_11D9E, apply_foreground_palette_override_loc_11DAC,
    apply_mountain_weather_override_loc_11EEA, apply_pending_palette_fade_sub_14C0,
    background_dimensions_loc_11B40, background_row_pointers_loc_11B7A,
    background_scroll_desert_heat_ripple_loop_loc_306F4,
    background_scroll_desert_heat_ripple_setup_loc_306D4,
    background_scroll_desert_heat_ripple_visible_band_loc_306C2,
    background_scroll_enigma_primary_copy_setup_loc_30426,
    background_scroll_enigma_primary_longword_copy_loc_30428,
    background_scroll_enigma_row_table_lookup_loc_303EA,
    background_scroll_enigma_row_transfer_loc_303D4,
    background_scroll_enigma_secondary_longword_copy_loc_30432,
    background_scroll_forest_water_ripple_loop_loc_3060A,
    background_scroll_forest_water_ripple_setup_loc_305E6,
    background_scroll_forest_water_ripple_visible_band_loc_305D4,
    background_scroll_lower_pass_loop_loc_30066, background_scroll_lower_pass_resume_loc_30054,
    background_scroll_lower_pass_setup_loc_3001E,
    background_scroll_piece_refresh_dual_wrap_branch_loc_30904,
    background_scroll_piece_refresh_linear_copy_loc_308A6,
    background_scroll_piece_refresh_row_wrap_final_copy_loc_30968,
    background_scroll_piece_refresh_row_wrap_first_copy_loc_30942,
    background_scroll_piece_refresh_row_wrap_second_copy_loc_30954,
    background_scroll_piece_refresh_seed_loc_3081A,
    background_scroll_piece_refresh_tile_wrap_counter_setup_loc_308DE,
    background_scroll_piece_refresh_tile_wrap_first_copy_loc_30988,
    background_scroll_piece_refresh_tile_wrap_wrapped_copy_loc_3099E,
    background_scroll_piece_refresh_tile_wrapped_copy_loc_308E8,
    background_scroll_piece_refresh_vdp_seed_loc_30884,
    background_scroll_piece_refresh_window_split_loc_3086A,
    background_scroll_piece_refresh_wrap_counter_setup_loc_308B6,
    background_scroll_piece_refresh_wrap_loc_30836,
    background_scroll_piece_refresh_wrapped_copy_loc_308C4,
    background_scroll_piece_row_dispatch_loc_3079A, background_scroll_piece_row_upload_loc_307B2,
    background_scroll_shift_data_second_pass_advance_loc_304A8,
    background_scroll_shift_data_second_pass_slot_loc_3048C,
    background_scroll_shift_data_second_pass_unchanged_loc_304A4,
    background_scroll_shift_data_slot_loc_30452, background_scroll_shift_data_unchanged_loc_30476,
    background_scroll_shift_data_wrap_loop_body_loc_304E0,
    background_scroll_shift_data_wrap_loop_loc_304C2,
    background_scroll_shift_data_wrap_loop_loc_304D8,
    background_scroll_shift_data_wrap_store_loc_304CA,
    background_scroll_shift_layers_leading_partial_fill_loc_30562,
    background_scroll_shift_layers_no_scroll_fill_loc_30502,
    background_scroll_shift_layers_setup_loc_3050A,
    background_scroll_shift_layers_trailing_partial_fill_loc_30592,
    background_scroll_stripe_horizontal_gate_loc_3023C,
    background_scroll_stripe_horizontal_layout_seed_loc_3026C,
    background_scroll_stripe_horizontal_lower_window_loc_3028E,
    background_scroll_stripe_horizontal_pass_loc_30296,
    background_scroll_stripe_horizontal_row_transfer_loc_302FE,
    background_scroll_stripe_horizontal_row_transfer_loc_3031A,
    background_scroll_stripe_layout_seed_loc_301D8,
    background_scroll_stripe_restore_camera_column_loc_3023A,
    background_scroll_stripe_right_window_loc_301F6,
    background_scroll_stripe_row_selection_loc_301CE,
    background_scroll_stripe_vertical_column_transfer_loc_3036A,
    background_scroll_stripe_vertical_column_transfer_loc_3037C,
    background_scroll_stripe_vertical_column_transfer_loc_30354,
    background_scroll_stripe_vertical_pass_loc_301FE, background_scroll_theme_dispatch_loc_30598,
    background_scroll_upper_pass_loc_2FFE8, background_scroll_upper_pass_resume_loc_30014,
    blend_palette_entry_loc_1538, blend_selected_palette_entries_sub_1536,
    block_layout_decode_word_run_loc_1354E, block_layout_decrement_decoded_word_loc_13540,
    block_layout_decrement_words_loc_13544, block_layout_dispatch_loc_132D6,
    block_layout_dispatch_loc_13504, block_layout_entry_words_loc_133B2,
    block_layout_entry_words_loc_133F0, block_layout_entry_words_loc_1332E,
    block_layout_entry_words_loc_1335E, block_layout_entry_words_loc_13470,
    block_layout_fixed_pattern_repeat_loop_loc_133DE,
    block_layout_increment_decoded_word_loc_13532, block_layout_long_run_repeat_loop_loc_1342E,
    block_layout_property_words_loc_1337C, block_layout_repeat_current_word_loc_1352A,
    block_layout_repeat_decoded_word_loc_13526, block_layout_repeat_loop_loc_1331E,
    block_layout_repeat_words_loc_1351E, block_layout_repeat_words_loc_13514,
    block_layout_short_count_property_words_loc_134AC,
    block_layout_short_count_repeat_loop_loc_1347C, block_layout_short_pair_repeat_loop_loc_1345E,
    block_layout_store_property_word_loc_13398, block_layout_terminate_loc_134BA,
    block_layout_variable_repeat_loop_loc_133A0, block_layout_variable_repeat_loop_loc_1334C,
    build_palette_fade_curve_sub_14C0, classify_background_palette_followup_loc_11DF8,
    classify_background_palette_override_loc_11DF4, classify_foreground_palette_override_loc_11D8E,
    classify_late_background_palette_override_loc_11E06,
    classify_post_load_object_routine_loc_11F96, compile_block_debris_sprites_sub_2A4C,
    copy_background_palette_loc_11E16, copy_foreground_palette_loc_11DB0,
    decode_block_layout_loc_132A6, decompress_to_ram_palette_row_bytes_loc_143CE,
    decompress_to_ram_palette_row_start_loc_143CC, decompress_to_vram_special_return_loc_143B2,
    drain_fg_update_queue_custom_sub_1596, empty_collision_row_loc_11B28,
    enemy_layout_finish_loc_3204E, enemy_layout_reset_and_seek_loc_31F8E,
    enemy_layout_status_word_loc_3200A, enemy_layout_status_word_with_negative_sentinel_loc_32036,
    eol_extra_life_bounce_step_loc_C60C, eol_extract_digit_loc_C4CE, eol_number_digit_loc_C4AE,
    eol_number_digits_loc_C48C, eol_object_position_step_loc_C5A2, fade_palette_span_sub_1C85E,
    fg_update_queue_cell_mapping_longwords_loc_165E, fg_update_queue_cell_vram_word_loc_1614,
    fg_update_queue_culled_entry_advance_loc_167A, flagpole_fade_step_loc_C1D0,
    flagpole_scoreboard_cursor_loc_C1EE, flagpole_scoreboard_reveal_duration_loc_C21E,
    flagpole_scoreboard_step_sub_C254, initialise_camera_bootstrap_loc_11F48,
    initialise_evanesc_object_slots_sub_EABC, initialise_fullscreen_palette_fade_sub_73D0,
    initialise_palette_fade_out_loc_28A6, initialise_scratch_queue_sub_28FC,
    initialise_shooter_object_slots_sub_F06A, initialise_sprite_frame_state_sub_B52,
    level_skip_completion_time_digit_sprites_loc_D468,
    manage_enemy_loading_allocate_slot_hint_loc_3690A,
    manage_enemy_loading_count_down_active_loc_3685E, manage_enemy_loading_setup_loc_3682C,
    manage_enemy_loading_spawn_object_loc_3691E, manage_enemy_loading_status_branch_loc_36840,
    manage_enemy_loading_status_word_loc_36900, object_check_in_range_active_count_gate_loc_320E2,
    object_check_in_range_normal_x_loc_320C0, object_check_in_range_normal_y_loc_320D8,
    object_check_in_range_wide_load_x_loc_32122, object_check_in_range_wide_load_y_loc_32140,
    object_out_of_range_delete_tail_loc_320B4, object_out_of_range_positive_slot_loc_320A6,
    prepare_background_art_decompress_loc_11EA2,
    prepare_followup_background_art_decompress_loc_11EF2,
    prepare_foreground_art_decompress_loc_11E60, prepare_mountain_background_art_loc_11EC4,
    reset_fg_update_queues_loc_1680, scoreboard_digit_vdp_clear_sub_C346,
    scratch_debris_pieces_sub_292E, select_background_art_permutation_loc_11E8A,
    select_background_art_permutation_loc_11E96, select_foreground_art_permutation_loc_11E48,
    select_foreground_art_permutation_loc_11E54, shared_vram_art_decompresses_loc_11EFA,
    should_process_fg_update_queue_cell_loc_15D2, spawn_block_debris_loc_2928,
    spawn_block_debris_sub_292E, sprite_scatter_step_loc_C436,
    step_fullscreen_palette_fade_sub_73D0, step_island_palette_cycle_lower_band_loc_12188,
    step_island_palette_cycle_mid_band_loc_12110, step_island_palette_cycle_primary_band_loc_12098,
    step_palette_fade_out_loc_28C6, swap_forest_background_palette_entries_sub_129AE,
    terrain_layout_row_pointers_loc_11B40, tile_fade_mask_column_sub_C3F6,
    tile_fade_mask_step_sub_C3DA, transfer_scroll_data_dispatch_loc_3039C,
    transfer_scroll_data_dma_tail_loc_300D2, transfer_scroll_data_prelude_loc_3038A,
    transfer_scroll_data_storm_loc_30110, transfer_scroll_data_storm_loop_loc_30130,
    transfer_scroll_data_storm_loop_loc_30172, transfer_scroll_data_storm_setup_loc_30158,
    transfer_scroll_data_vsram_setup_loc_300BA,
};
use crate::movement::*;
use crate::projectiles::{
    KidProjectileCollisionCursorLoc1DCC, KidProjectileCollisionObjectLoc1DCC,
    KidProjectileCollisionPassSub1D76, first_live_kid_projectile_collision_cursor_loc_1DCC,
    first_mirrored_kid_projectile_overlap_loc_1E1E,
    first_non_mirrored_kid_projectile_overlap_loc_1DF2, kid_projectile_collision_passes_sub_1D76,
};
use crate::weather::{
    FallingWeatherSlotSub61CA, LevelSpecialEffectsRoutineSub6034, MurderWallWarningInputSub5E02,
    MurderWallWarningRenderSub5E02, MurderWallWarningSpriteSub5E02, WeatherCollisionRowsSub61CA,
    WeatherImpactSlotSub61CA, WeatherPaletteOpSub6048, WeatherSpecialEffectsStepSub6048,
    WeatherSpriteSub61CA, WeatherWorkerInputSub61CA, compile_murder_wall_warning_sprites_sub_5E02,
    copy_murder_wall_palette_loc_140EE, dispatch_level_special_effects_sub_6034,
    step_level_special_effects_sub_6048, step_weather_worker_sub_61CA,
};
use kid_data::{
    AssetManifest, BgscrollProfile, DecodedAssetFile, DecodedLevelAssets, EnemyLayout, LevelHeader,
    LevelRecord, LevelResources, PlatformLayout, PlatformPlacement, PlatformScriptRef,
};
use std::path::PathBuf;
use std::{
    collections::{BTreeMap, HashSet},
    env, fs,
    ops::RangeInclusive,
    process,
    time::{SystemTime, UNIX_EPOCH},
};

mod fixtures;
use fixtures::*;

mod block_tests;
mod boot_tests;
mod combat_tests;
mod helmet_tests;
mod level_tests;
mod movement_tests;
mod projectile_tests;
mod runtime_tests;
mod weather_tests;

#[test]
fn scenario_boot_to_level_uses_start_path_and_reaches_ingame() {
    let mut runtime = NativeRuntime::new(manifest());

    runtime.boot_to_level(LevelId(0));

    assert!(
        matches!(runtime.boot_state, BootState::InGame),
        "boot_to_level must press through the title-card path before returning",
    );
    assert_eq!(runtime.current_level, LevelId(0));
    assert!(
        runtime.start_press_count >= BOOT_TO_LEVEL_START_FRAMES.len(),
        "the scripted boot path should issue the configured Start presses",
    );
    assert!(
        runtime.level_start_frame.is_some(),
        "boot_to_level should leave a traceable level-start frame",
    );
}

#[test]
fn scenario_script_wrappers_return_the_requested_trace_length() {
    let physics_order_trace = run_script_with_physics_order(
        manifest(),
        &[0x08, 0x40, 0x00],
        3,
        None,
        PhysicsOrder::GravityFirst,
    );
    let traced = run_script_with_trace(manifest(), &[0x00, 0x00], 2, None);

    assert_eq!(physics_order_trace.len(), 3);
    assert_eq!(physics_order_trace[2].frame_index, 2);
    assert_eq!(traced.len(), 2);
    assert_eq!(traced[1].frame_index, 1);
}

#[test]
fn scenario_settled_warp_adds_oracle_observed_live_settle_window() {
    let unsettled = run_script_with_warp(manifest(), &[0x00], 1, Some(0));
    let settled = run_script_with_settled_warp(manifest(), &[0x00], 1, Some(0));

    assert_eq!(settled[0].current_level, LevelId(0));
    assert!(!settled[0].paused);
    assert!(
        settled[0].frame_index >= unsettled[0].frame_index + 100,
        "settled full-frame traces should include the oracle-observed live target-level tail",
    );
}

#[test]
fn load_block_grid_resolves_decoded_resource_layout_for_mapped_level() {
    let output_root = unique_temp_path("decoded-resource-block-layout");
    let block_dir = output_root.join("decoded/level/56");
    fs::create_dir_all(&block_dir).expect("decoded block directory should be creatable");
    let block_path = block_dir.join("block.bin");
    fs::write(&block_path, [0x60, 0x2A]).expect("decoded block layout should be writable");

    let mut mapped = manifest();
    mapped.output_root = output_root.clone();
    mapped.map_order = vec![0; 10];
    mapped.map_order[9] = 0x56;
    mapped.levels[0].level_id = 0x56;
    if let Some(header) = mapped.levels[0].header_metadata.as_mut() {
        header.width_screens = 12;
        header.height_screens = 2;
    }
    mapped.decoded_levels = vec![DecodedLevelAssets {
        contract_version: 3,
        level_id: 0x56,
        foreground: DecodedAssetFile {
            relative_path: String::from("decoded/level/56/foreground.bin"),
            preview_png_path: String::from("decoded/level/56/foreground.png"),
            size_bytes: 0,
            sha1: String::new(),
        },
        block: DecodedAssetFile {
            relative_path: String::from("decoded/level/56/block.bin"),
            preview_png_path: String::from("decoded/level/56/block.png"),
            size_bytes: 2,
            sha1: String::new(),
        },
        background: DecodedAssetFile {
            relative_path: String::from("decoded/level/56/background.bin"),
            preview_png_path: String::from("decoded/level/56/background.png"),
            size_bytes: 0,
            sha1: String::new(),
        },
    }];

    let runtime = NativeRuntime::new(mapped);
    let grid = runtime
        .load_block_grid_for_level(LevelId(9))
        .expect("ASM MapOrder_Index lookup should resolve logical level 09 to decoded resource 56");

    assert_eq!(grid.width_cells, 240);
    assert_eq!(grid.height_cells, 28);
    assert_eq!(grid.cell_at(0, 0).collision, terrain::CollisionKind::Solid);

    fs::remove_dir_all(output_root).ok();
}

#[test]
fn scenario_settled_warp_starts_later_levels_in_live_gameplay() {
    let settled = run_script_with_settled_warp(manifest(), &[0x00], 1, Some(9));

    assert_eq!(settled[0].current_level, LevelId(9));
    assert_eq!(
        settled[0].game_mode,
        GameMode::InGame,
        "full-frame sweeps must not start while the native level-skip shell is still visible",
    );
    assert!(!settled[0].paused);
}

#[test]
fn scenario_settled_warp_level09_cached_assets_starts_in_live_gameplay() {
    let manifest_path = repo_root()
        .join(".cache/generated/28b904000b2863b6760531807760b571f1a5fc1d/asset-manifest.json");
    if !manifest_path.is_file() {
        eprintln!("skipping cached level09 settled-warp test: generated asset manifest missing");
        return;
    }
    let manifest = kid_data::load_manifest(&manifest_path).expect("load cached manifest");
    let settled = run_script_with_settled_warp(manifest, &[0x00], 1, Some(9));

    assert_eq!(settled[0].current_level, LevelId(9));
    assert_eq!(settled[0].game_mode, GameMode::InGame);
    assert!(!settled[0].paused);
}
