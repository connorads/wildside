use super::*;

fn kid_projectile_collision_object_loc_1dcc(
    deleted: bool,
    mirrored: bool,
    collision_index_word: u16,
    x_anchor_px: i16,
    y_anchor_px: i16,
    collision_sizes: Vec<CollisionSizeEntrySub21F8>,
) -> KidProjectileCollisionObjectLoc1DCC {
    KidProjectileCollisionObjectLoc1DCC {
        deleted,
        mirrored,
        collision_index_word,
        x_anchor_px,
        y_anchor_px,
        collision_sizes,
    }
}

fn cached_any_percent_demo_runtime_and_script() -> Option<(NativeRuntime, Vec<u8>)> {
    cached_tas_runtime_and_script("kc_any_percent_2024")
}

fn published_natural_vblank_work() -> ModeLevelVblankWorkBeforeSubE12 {
    ModeLevelVblankWorkBeforeSubE12 {
        dynamic_art_tiles_loc_ad4: Some([16, 4, 0, 0, 0, 0]),
        palette_dma_sub_970: true,
        scroll_path: ModeLevelScrollTransferPath::NormalLoc300ba,
        foreground_custom_queue_entries_sub_1596: 0,
        foreground_cell_queue_entries_sub_1596: 0,
        paused_sub_6874: false,
        hud_path_sub_bdc0: ModeLevelHudBuildPath::LocBe20DirectInitialCounterUpdates,
    }
}

fn published_vblank_arbitration(
    vdp_bus_halt_cycles: u32,
    refresh_wait_cycles: u32,
) -> ModeLevelVblankArbitrationBeforeSubE12 {
    ModeLevelVblankArbitrationBeforeSubE12 {
        vdp_bus_halt_cycles,
        vdp_pending_wait_cycles: 2_002,
        refresh_wait_cycles,
        refresh_suffix_after_final_vdp_ownership:
            ModeLevelVblankRefreshSuffixAfterFinalVdpOwnership {
                raw_cycles_before_pending_wait: 16,
                pending_wait_cycles: 60,
                raw_cycles_after_pending_wait: 1_146,
            },
    }
}

#[test]
fn published_tas_first_gameplay_pass_clears_palette_dma_suppression_sub_73d0() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut observed = Vec::new();

    for (frame_index, mask) in script.iter().copied().enumerate().take(904) {
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        if matches!(frame_index, 902 | 903) {
            observed.push((
                frame_index,
                trace
                    .mode_level_vblank_work_before_sub_e12
                    .expect("published gameplay frame exposes its VBlank work")
                    .palette_dma_sub_970,
            ));
        }
    }

    assert_eq!(
        observed,
        vec![(902, false), (903, true)],
        "ASM sub_7196 sets PaletteToDMA_Flag before frame 902; sub_73D0 clears it during that first object pass",
    );
}

#[test]
fn published_tas_frame_1697_vint_sample_drives_life_icon_raster_timing_sub_e12() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut frame_1697 = None;

    for (frame_index, mask) in script.iter().copied().enumerate().take(1698) {
        if frame_index == 1697 {
            // Bounded jgenesis trace: helper frame 1696 samples VINT 58 mclk
            // after scanline 224 h_mclk 16, resumes WaitForVint at stale
            // `$942`, then reaches Mode_Level `$7C8` at h_mclk 1684.
            assert!(runtime.seed_mode_level_raster_vint_assertion(
                vdp::ModeLevelRasterSeedVintAssertion {
                    beam: vdp::H40BeamPosition::new(224, 16).expect("H40 VINT beam"),
                },
                WaitForVintInterruptSample {
                    mclk_until_interrupt_sample: 58,
                    interrupt_entry_refresh_wait_events: 0,
                    resume: WaitForVintResumePath::StaleBranchLoc942,
                    arbitration_after_handler_entry: WaitForVintArbitration {
                        refresh_wait_events: 2,
                        z80_68k_bus_steals: 0,
                    },
                },
                published_natural_vblank_work(),
                published_vblank_arbitration(3_383, 46),
            ));
        }
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        if frame_index == 1697 {
            let entry = runtime.mode_level_cpu_raster_cursor_sub_44dc();
            let exit = entry.and_then(|entry| {
                vdp::life_icon_cpu_raster_cursor_after_sub_44dc(
                    entry,
                    &trace.hud_dma_events_sub_44dc,
                    &[],
                )
            });
            frame_1697 = Some((trace.hud_raster_upload_timings_sub_44dc, entry, exit));
        }
    }

    assert_eq!(
        frame_1697,
        Some((
            vec![HudRasterUploadTimingSub44DC {
                command: HudDmaEventSub44DC::LifeIcon {
                    frame_table_byte_offset: 4,
                    wrapped_to_start: false,
                },
                previous_frame_table_byte_offset: 0,
                first_commit_scanline: 42,
                first_commit_pixel: 380,
            }],
            Some(vdp::ModeLevelCpuRasterCursor {
                beam: vdp::H40BeamPosition::new(42, 1_294).expect("typed sub_44DC beam"),
                refresh_phase: M68kRefreshPhase::new(64).expect("oracle refresh phase"),
            }),
            Some(vdp::ModeLevelCpuRasterCursor {
                beam: vdp::H40BeamPosition::new(49, 3_072).expect("typed loc_80C beam"),
                refresh_phase: M68kRefreshPhase::new(58).expect("oracle refresh phase"),
            }),
        )),
        "ASM sub_44DC loc_4586 commits, then loc_44D4 exits at the loc_80C call"
    );
}

#[test]
fn published_tas_no_z80_polls_drive_frames_1703_and_1709_life_icon_timing() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut timings = Vec::new();

    for (frame_index, mask) in script.iter().copied().enumerate().take(1710) {
        let poll = match frame_index {
            1_703 => Some((107, 442, 78, 3_380, 46)),
            1_709 => Some((105, 1_008, 104, 3_384, 48)),
            _ => None,
        };
        if let Some((scanline, scanline_mclk, refresh_phase, halt_cycles, refresh_cycles)) = poll {
            assert!(
                runtime.seed_mode_level_raster_wait_for_vint_poll_no_z80(
                    vdp::WaitForVintPollSeedNoZ80 {
                        first_poll_beam: vdp::H40BeamPosition::new(scanline, scanline_mclk)
                            .expect("bounded first WaitForVint $93E beam"),
                        refresh_phase: M68kRefreshPhase::new(refresh_phase)
                            .expect("bounded jgenesis refresh phase"),
                    },
                    vdp::ModeLevelRasterSeedVintAssertion {
                        beam: vdp::H40BeamPosition::new(224, 16).expect("H40 VINT assertion"),
                    },
                    published_natural_vblank_work(),
                    published_vblank_arbitration(halt_cycles, refresh_cycles),
                )
            );
        }
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        if matches!(frame_index, 1_703 | 1_709) {
            // Bounded jgenesis trace: frame 1709 admits four GEMS bus steals
            // after the final pending FIFO enqueue; frame 1703 admits none.
            let z80_offsets = if frame_index == 1_709 {
                [43, 96, 151, 192].map(M68kCycleCount::from_raw).to_vec()
            } else {
                Vec::new()
            };
            let entry = runtime.mode_level_cpu_raster_cursor_sub_44dc();
            let exit = entry.and_then(|entry| {
                vdp::life_icon_cpu_raster_cursor_after_sub_44dc(
                    entry,
                    &trace.hud_dma_events_sub_44dc,
                    &z80_offsets,
                )
            });
            timings.push((
                frame_index,
                trace.hud_raster_upload_timings_sub_44dc,
                entry,
                exit,
            ));
        }
    }

    assert_eq!(
        timings,
        vec![
            (
                1_703,
                vec![HudRasterUploadTimingSub44DC {
                    command: HudDmaEventSub44DC::LifeIcon {
                        frame_table_byte_offset: 8,
                        wrapped_to_start: false,
                    },
                    previous_frame_table_byte_offset: 4,
                    first_commit_scanline: 32,
                    first_commit_pixel: 330,
                }],
                Some(vdp::ModeLevelCpuRasterCursor {
                    beam: vdp::H40BeamPosition::new(32, 765).expect("typed sub_44DC beam"),
                    refresh_phase: M68kRefreshPhase::new(116).expect("oracle refresh phase"),
                }),
                Some(vdp::ModeLevelCpuRasterCursor {
                    beam: vdp::H40BeamPosition::new(39, 2_690).expect("typed loc_80C beam"),
                    refresh_phase: M68kRefreshPhase::new(58).expect("oracle refresh phase"),
                }),
            ),
            (
                1_709,
                vec![HudRasterUploadTimingSub44DC {
                    command: HudDmaEventSub44DC::LifeIcon {
                        frame_table_byte_offset: 4,
                        wrapped_to_start: false,
                    },
                    previous_frame_table_byte_offset: 8,
                    first_commit_scanline: 15,
                    first_commit_pixel: 92,
                }],
                Some(vdp::ModeLevelCpuRasterCursor {
                    beam: vdp::H40BeamPosition::new(14, 2_431).expect("typed sub_44DC beam"),
                    refresh_phase: M68kRefreshPhase::new(110).expect("oracle refresh phase"),
                }),
                Some(vdp::ModeLevelCpuRasterCursor {
                    beam: vdp::H40BeamPosition::new(22, 838).expect("typed loc_80C beam"),
                    refresh_phase: M68kRefreshPhase::new(102).expect("oracle refresh phase"),
                }),
            ),
        ],
        "ASM WaitForVint -> Mode_Level -> sub_44DC loc_4586 -> loc_44D4"
    );
}

#[test]
fn published_tas_frame_1706_sub_e12_seed_drives_diamond_dma_raster_timing() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut observed = None;

    for (frame_index, mask) in script.iter().copied().enumerate().take(1_707) {
        if frame_index == 1_706 {
            // Oracle selected PC at ASM Execute_Objects sub_E12: absolute
            // cycle 218,368,469, H40 243:23, refresh phase 50.
            runtime.seed_mode_level_raster_sub_e12(vdp::ModeLevelRasterSeedSubE12 {
                beam: vdp::H40BeamPosition::new(243, 23).expect("bounded sub_E12 beam"),
                refresh_phase: M68kRefreshPhase::new(50).expect("bounded sub_E12 refresh phase"),
            });
        }
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        if frame_index == 1_706 {
            observed = Some(trace.hud_raster_upload_timings_sub_44dc);
        }
    }

    assert_eq!(
        observed,
        Some(vec![HudRasterUploadTimingSub44DC {
            command: HudDmaEventSub44DC::Diamond {
                frame_table_byte_offset: 12,
            },
            previous_frame_table_byte_offset: 8,
            first_commit_scanline: 18,
            first_commit_pixel: 172,
        }]),
        "ASM sub_E12 -> sub_44DC loc_4626 -> sub_5D4A commits the frame-1706 Diamond DMA",
    );
}

#[test]
fn published_tas_life_icon_upload_lower_bound_separates_late_and_visible_commits() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut observed = Vec::new();

    for (frame_index, mask) in script.iter().copied().enumerate().take(1710) {
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        if matches!(frame_index, 1_697 | 1_703 | 1_709) {
            let mut vblank = published_natural_vblank_work();
            vblank.hud_path_sub_bdc0 = ModeLevelHudBuildPath::LocBe20ThenLocBe4cStableCounters;
            observed.push((
                frame_index,
                vdp::life_icon_earliest_commit_scanline_from_vint_assertion(
                    vblank,
                    trace
                        .mode_level_work_before_hud_sub_44dc
                        .as_ref()
                        .expect("published target frame has typed pre-HUD work"),
                ),
            ));
        }
    }

    assert_eq!(
        observed,
        vec![(1_697, Some(41)), (1_703, Some(31)), (1_709, Some(14))],
        "ASM V_Int -> Mode_Level -> sub_E12 -> sub_44DC loc_4586 conservative lower bound",
    );
}

#[test]
fn published_tas_frame_1705_exposes_loc_102b8_flash_clock() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut observed = Vec::new();
    for (frame_index, mask) in script.iter().copied().enumerate().take(1707) {
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        if (1_704..=1_705).contains(&frame_index) {
            observed.push((
                frame_index,
                trace.prize_block_spawn_flashes,
                trace
                    .floating_items
                    .into_iter()
                    .filter(|&(x_px, _, _, content_id)| x_px == 536 && content_id == 0)
                    .collect::<Vec<_>>(),
            ));
        }
    }
    assert_eq!(
        observed,
        vec![
            (
                1_704,
                vec![(536, 56, 1), (520, 56, 11), (504, 56, 18), (488, 56, 24)],
                vec![(536, 85, 477, 0)],
            ),
            (
                1_705,
                vec![(520, 56, 10), (504, 56, 17), (488, 56, 23)],
                vec![(536, 86, 476, 0)],
            ),
        ],
        "ASM loc_102B8 terminates while PrizeDiamond_Init loc_10396 remains live",
    );
}

#[test]
fn published_tas_helpers_expose_source_ordered_post_hud_work_loc_80c() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let expected_phases = vec![
        ModeLevelPostHudPhase::ManageEnemyLoading,
        ModeLevelPostHudPhase::GfxObjectsMoveAndAnimate,
        ModeLevelPostHudPhase::ReadJoypad,
        ModeLevelPostHudPhase::TerrainEffectsSub5e02,
        ModeLevelPostHudPhase::ManagePlatformLoading,
        ModeLevelPostHudPhase::ExecuteScriptedPlatforms,
        ModeLevelPostHudPhase::PlatformsCheckCollision,
        ModeLevelPostHudPhase::GfxObjectsCollisionF86a,
        ModeLevelPostHudPhase::GfxObjectsCollisionKidProjectile,
        ModeLevelPostHudPhase::LatchDamageState,
        ModeLevelPostHudPhase::GfxObjectsCollisionKidProjectileAgainstKid,
        ModeLevelPostHudPhase::CollisionSub1f52,
        ModeLevelPostHudPhase::CollisionSub219c,
        ModeLevelPostHudPhase::CollisionSub226a,
        ModeLevelPostHudPhase::GfxObjectsCollisionKidF86a,
        ModeLevelPostHudPhase::GfxObjectsCollisionKidF86e,
        ModeLevelPostHudPhase::CollisionSub1d76,
        ModeLevelPostHudPhase::CollisionSub1fa2,
        ModeLevelPostHudPhase::PostCollisionSub2a4c,
        ModeLevelPostHudPhase::MakeSpritesFromGfxObjects,
        ModeLevelPostHudPhase::DiamondPowerCompileSprites,
        ModeLevelPostHudPhase::MakeSpritesFromPlatforms,
    ];
    let mut observed = Vec::new();
    let mut observed_shooter_work = Vec::new();
    let mut observed_block_debris_work = Vec::new();
    let mut observed_diamond_power_compile_work = Vec::new();
    let mut observed_kid_projectile_collision_work = Vec::new();
    let mut observed_gfx_move_animate_work = Vec::new();
    let mut observed_gfx_sprite_work = Vec::new();
    let mut observed_gfx_collision_work = Vec::new();
    let mut observed_terrain_special_scan = Vec::new();
    let mut observed_raw_cycles_to_loc_892 = Vec::new();

    for (frame_index, mask) in script.iter().copied().enumerate().take(1_710) {
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        if frame_index == 902 {
            let ledger = runtime
                .mode_level_work_after_hud_sub_44dc()
                .expect("first gameplay frame should expose the loc_80C workload");
            assert_eq!(
                ledger.terrain_special_scan_sub_1fa2,
                Some(ModeLevelTerrainSpecialScanSub1FA2 {
                    bounds: ModeLevelKidGfxTerrainScanBounds {
                        left_col: 2,
                        right_col: 2,
                        top_row: 45,
                        bottom_row: 45,
                    },
                    tile_iterations: 1,
                    first_special_collision: None,
                }),
                "ASM sub_21F8 loc_2252 collapses the activation-frame hitbox before sub_1FA2",
            );
            assert_eq!(
                mode_level_post_hud_phase_raw_cycles_to_loc_892(ledger)
                    .expect("first gameplay post-HUD path is classified")[17],
                M68kCycleCount::from_raw(400),
                "ASM loc_876 -> sub_1FA2 -> loc_87A scans one point cell",
            );
            assert_eq!(
                ledger.gfx_sprite_work_sub_1492,
                ModeLevelGfxSpriteWorkSub1492::Classified {
                    nodes: vec![
                        ModeLevelGfxSpriteNodeWorkSub1492::HiddenFlagClear,
                        ModeLevelGfxSpriteNodeWorkSub1492::HiddenFlagClear,
                        ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulled,
                        ModeLevelGfxSpriteNodeWorkSub1492::VerticallyCulled,
                    ],
                },
                "ASM frame-902 sub_1492 keeps the second persistent node hidden during loc_7508 activation",
            );
            assert_eq!(
                mode_level_post_hud_phase_raw_cycles_to_loc_892(ledger)
                    .expect("first gameplay post-HUD path is classified")[19],
                M68kCycleCount::from_raw(936),
                "ASM loc_87E -> sub_1492 -> loc_882 activation-frame paths",
            );
        }
        if matches!(frame_index, 1_697 | 1_703 | 1_709) {
            let ledger = runtime
                .mode_level_work_after_hud_sub_44dc()
                .expect("published helper should take the natural loc_80C path");
            assert_eq!(
                trace.mode_level_work_after_hud_sub_44dc.as_ref(),
                Some(ledger),
                "ASM Mode_Level loc_80C workload must cross the trace boundary",
            );
            assert_eq!(
                trace.mode_level_vblank_work_before_sub_e12,
                Some(ModeLevelVblankWorkBeforeSubE12 {
                    dynamic_art_tiles_loc_ad4: None,
                    ..published_natural_vblank_work()
                }),
                "core exposes the VBlank branches and leaves render-owned loc_AD4 art unclassified",
            );
            assert_eq!(
                trace.mode_level_tail_work_loc_892,
                runtime.mode_level_tail_work_loc_892(),
                "ASM Mode_Level loc_892 workload must cross the trace boundary",
            );
            if frame_index == 1_697 {
                let mut reordered = ledger.clone();
                reordered.phases.swap(0, 1);
                assert_eq!(
                    mode_level_post_hud_raw_cycles_to_loc_892(&reordered),
                    None,
                    "a post-HUD ledger outside ASM source order must fail closed",
                );
            }
            let read_joypad_raw_cycles = read_joypad_raw_cycles(ledger.read_joypad_work)
                .expect("published helper should classify Demo ReadJoypad");
            let enemy_loading_raw_cycles =
                manage_enemy_loading_raw_cycles(ledger.enemy_loading_path)
                    .expect("published helper should classify Demo enemy loading");
            observed_shooter_work.push((frame_index, ledger.shooter_collision_work));
            observed_block_debris_work.push((frame_index, ledger.block_debris_work_sub_2a4c));
            observed_diamond_power_compile_work
                .push((frame_index, ledger.diamond_power_compile_work_sub_3f798));
            observed_kid_projectile_collision_work
                .push((frame_index, ledger.kid_projectile_collision_work_sub_1d76));
            observed_gfx_move_animate_work
                .push((frame_index, ledger.gfx_move_animate_work_sub_1444.clone()));
            observed_gfx_sprite_work.push((frame_index, ledger.gfx_sprite_work_sub_1492.clone()));
            observed_gfx_collision_work.push((frame_index, ledger.gfx_collision_work.clone()));
            observed_terrain_special_scan.push((frame_index, ledger.terrain_special_scan_sub_1fa2));
            observed_raw_cycles_to_loc_892.push((
                frame_index,
                mode_level_post_hud_raw_cycles_to_loc_892(ledger)
                    .expect("published helper should classify the complete post-HUD path")
                    .get(),
            ));
            if frame_index == 1_697 {
                assert_eq!(
                    mode_level_post_hud_phase_raw_cycles_to_loc_892(ledger)
                        .expect("published helper should classify every post-HUD phase")
                        .map(M68kCycleCount::get),
                    [
                        114, 4_518, 1_294, 56, 446, 88, 370, 356, 356, 56, 192, 68, 308, 64, 192,
                        2_600, 144, 576, 426, 11_646, 106, 96,
                    ],
                    "ASM Mode_Level loc_810..loc_892 frame-1697 phase costs must stay source-ordered",
                );
                let advance = mode_level_post_hud_advance_to_loc_892_with_scheduler(
                    M68kRefreshPhase::new(58).expect("oracle frame-1697 loc_810 phase"),
                    ledger,
                    |_, _| Some(M68kCycleCount::from_raw(0)),
                    |_| {},
                )
                .expect("published post-HUD path is fully source-scheduled");
                assert_eq!(advance.elapsed, M68kCycleCount::from_raw(24_454));
                assert_eq!(advance.refresh_events, 191);
                assert_eq!(
                    advance.phase_after,
                    M68kRefreshPhase::new(64).expect("oracle frame-1697 loc_892 phase"),
                );
                let mut first_hardware_advance = true;
                let mut phase_advances = Vec::new();
                let traced_advance = mode_level_post_hud_advance_to_loc_892_with_phase_observer(
                    M68kRefreshPhase::new(58).expect("oracle frame-1697 loc_810 phase"),
                    ledger,
                    |_, _| {
                        let wait = u32::from(first_hardware_advance) * 11;
                        first_hardware_advance = false;
                        Some(M68kCycleCount::from_raw(wait))
                    },
                    |_| {},
                    |phase| phase_advances.push(phase),
                )
                .expect("published post-HUD path supports typed phase observation");
                assert_eq!(phase_advances.len(), ledger.phases.len());
                assert_eq!(
                    phase_advances
                        .iter()
                        .map(|advance| advance.phase)
                        .collect::<Vec<_>>(),
                    ledger.phases,
                    "ASM Mode_Level loc_810..loc_892 observer preserves source order",
                );
                assert_eq!(
                    phase_advances[0].generated_hardware_wait_cycles,
                    M68kCycleCount::from_raw(11),
                    "ASM loc_810 phase owns a hardware wait generated by its instruction",
                );
                assert!(phase_advances[1..].iter().all(|advance| {
                    advance.generated_hardware_wait_cycles == M68kCycleCount::from_raw(0)
                }));
                for phase_advance in &phase_advances {
                    assert_eq!(
                        phase_advance.elapsed.get(),
                        phase_advance.raw_cycles.get()
                            + 2 * phase_advance.refresh_events
                            + phase_advance.generated_hardware_wait_cycles.get(),
                        "ASM {:?} phase timing accounts for raw, refresh, and generated waits",
                        phase_advance.phase,
                    );
                }
                assert_eq!(
                    traced_advance.elapsed.get(),
                    phase_advances
                        .iter()
                        .map(|advance| advance.elapsed.get())
                        .sum::<u32>(),
                );
                assert_eq!(
                    vdp::mode_level_post_hud_cursor_to_loc_892_with_scheduler(
                        vdp::ModeLevelCpuRasterCursor {
                            beam: vdp::H40BeamPosition::new(49, 3_072)
                                .expect("frame-1697 loc_810 beam"),
                            refresh_phase: M68kRefreshPhase::new(58)
                                .expect("frame-1697 loc_810 phase"),
                        },
                        ledger,
                        |_, _| Some(M68kCycleCount::from_raw(0)),
                        |_| {},
                    ),
                    Some(vdp::ModeLevelCpuRasterCursor {
                        beam: vdp::H40BeamPosition::new(99, 3_250)
                            .expect("oracle frame-1697 loc_892 beam"),
                        refresh_phase: M68kRefreshPhase::new(64)
                            .expect("oracle frame-1697 loc_892 phase"),
                    }),
                );
            }
            observed.push((
                frame_index,
                ledger.phases.clone(),
                ledger.enemy_loading_path,
                enemy_loading_raw_cycles.get(),
                ledger.read_joypad_work,
                read_joypad_raw_cycles.get(),
                ledger.enemy_loading_initial_pass_complete,
                ledger.linked_gfx_object_count,
                ledger.murder_wall_active_sub_5e02,
                ledger.platform_loader_work,
                ledger.platform_work,
            ));
        }
    }

    assert_eq!(
        observed_raw_cycles_to_loc_892,
        vec![(1_697, 24_072), (1_703, 25_598), (1_709, 26_052)],
        "ASM Mode_Level loc_810 -> loc_892 selected-PC source spans",
    );
    assert_eq!(
        observed
            .iter()
            .map(
                |(_, phases, enemy_path, _, joypad_path, _, loader_complete, _, _, _, _)| (
                    phases,
                    *enemy_path,
                    *joypad_path,
                    *loader_complete,
                )
            )
            .collect::<Vec<_>>(),
        vec![
            (
                &expected_phases,
                ModeLevelEnemyLoadingPath::DemoNonKnightsReturnLoc3682a,
                ModeLevelReadJoypadWork::DemoPlaybackLocC22(Some(
                    ModeLevelDemoReadJoypadWorkLocC22 {
                        counter_before_increment: 723,
                        advances_keypress_pointer: true,
                    },
                )),
                false,
            ),
            (
                &expected_phases,
                ModeLevelEnemyLoadingPath::DemoNonKnightsReturnLoc3682a,
                ModeLevelReadJoypadWork::DemoPlaybackLocC22(Some(
                    ModeLevelDemoReadJoypadWorkLocC22 {
                        counter_before_increment: 729,
                        advances_keypress_pointer: true,
                    },
                )),
                false,
            ),
            (
                &expected_phases,
                ModeLevelEnemyLoadingPath::DemoNonKnightsReturnLoc3682a,
                ModeLevelReadJoypadWork::DemoPlaybackLocC22(Some(
                    ModeLevelDemoReadJoypadWorkLocC22 {
                        counter_before_increment: 735,
                        advances_keypress_pointer: true,
                    },
                )),
                false,
            ),
        ],
        "ASM Mode_Level loc_80C..loc_892 source order and ReadJoypad loc_C22 work",
    );
    assert_eq!(
        observed
            .iter()
            .map(
                |(
                    frame,
                    _,
                    _,
                    enemy_raw,
                    _,
                    joypad_raw,
                    _,
                    gfx_count,
                    murder_wall_active,
                    platform_loader_work,
                    platform_work,
                )| {
                    (
                        *frame,
                        *enemy_raw,
                        *joypad_raw,
                        *gfx_count,
                        *murder_wall_active,
                        terrain_effects_raw_cycles_sub_5e02(*murder_wall_active)
                            .map(M68kCycleCount::get),
                        *platform_loader_work,
                        platform_loader_work
                            .and_then(manage_platform_loading_raw_cycles_loc_2744)
                            .map(M68kCycleCount::get),
                        platform_work.active_object_count,
                        execute_scripted_platforms_raw_cycles_sub_23ea(*platform_work)
                            .map(M68kCycleCount::get),
                        platforms_check_collision_raw_cycles_sub_2574(*platform_work)
                            .map(M68kCycleCount::get),
                        make_sprites_from_platforms_raw_cycles_sub_2444(*platform_work)
                            .map(M68kCycleCount::get),
                    )
                },
            )
            .collect::<Vec<_>>(),
        vec![
            (
                1_697,
                84,
                1_266,
                16,
                false,
                Some(38),
                Some(ModeLevelPlatformLoaderWorkLoc2744::Placement {
                    wrapped_to_start: true,
                    range: ModeLevelPlatformLoaderRangeLoc2762::OutOfRangeRight,
                    existing_slot: ModeLevelPlatformLoaderExistingSlotSub2728::Missing,
                }),
                Some(428),
                0,
                Some(70),
                Some(352),
                Some(78),
            ),
            (
                1_703,
                84,
                1_266,
                17,
                false,
                Some(38),
                Some(ModeLevelPlatformLoaderWorkLoc2744::Placement {
                    wrapped_to_start: true,
                    range: ModeLevelPlatformLoaderRangeLoc2762::OutOfRangeRight,
                    existing_slot: ModeLevelPlatformLoaderExistingSlotSub2728::Missing,
                }),
                Some(428),
                0,
                Some(70),
                Some(352),
                Some(78),
            ),
            (
                1_709,
                84,
                1_266,
                16,
                false,
                Some(38),
                Some(ModeLevelPlatformLoaderWorkLoc2744::Placement {
                    wrapped_to_start: true,
                    range: ModeLevelPlatformLoaderRangeLoc2762::OutOfRangeRight,
                    existing_slot: ModeLevelPlatformLoaderExistingSlotSub2728::Missing,
                }),
                Some(428),
                0,
                Some(70),
                Some(352),
                Some(78),
            ),
        ],
        "ASM enemy loading, ReadJoypad, and linked GfxObject/platform work",
    );
    assert_eq!(
        observed_shooter_work
            .iter()
            .map(|(frame, work)| (
                *frame,
                *work,
                shooter_support_collision_raw_cycles_sub_1f52(*work).map(M68kCycleCount::get),
                shooter_kid_collision_raw_cycles_sub_219c(*work).map(M68kCycleCount::get),
                shooter_projectile_collision_raw_cycles_sub_226a(*work).map(M68kCycleCount::get),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                1_697,
                ModeLevelShooterCollisionWork {
                    active_support_slots: 0,
                    active_moving_objects: 0,
                },
                Some(50),
                Some(290),
                Some(46),
            ),
            (
                1_703,
                ModeLevelShooterCollisionWork {
                    active_support_slots: 0,
                    active_moving_objects: 0,
                },
                Some(50),
                Some(290),
                Some(46),
            ),
            (
                1_709,
                ModeLevelShooterCollisionWork {
                    active_support_slots: 0,
                    active_moving_objects: 0,
                },
                Some(50),
                Some(290),
                Some(46),
            ),
        ],
        "ASM sub_1F52/sub_219C/sub_226A null-sentinel collision paths",
    );
    assert_eq!(
        observed_block_debris_work
            .iter()
            .map(|(frame, work)| (
                *frame,
                *work,
                block_debris_sprites_raw_cycles_sub_2a4c(*work).map(M68kCycleCount::get),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                1_697,
                ModeLevelBlockDebrisWorkSub2A4C {
                    active_slot_count: 0,
                },
                Some(408),
            ),
            (
                1_703,
                ModeLevelBlockDebrisWorkSub2A4C {
                    active_slot_count: 0,
                },
                Some(408),
            ),
            (
                1_709,
                ModeLevelBlockDebrisWorkSub2A4C {
                    active_slot_count: 0,
                },
                Some(408),
            ),
        ],
        "ASM sub_2A4C six inactive scratch-slot paths",
    );
    assert_eq!(
        observed_diamond_power_compile_work
            .iter()
            .map(|(frame, work)| (
                *frame,
                *work,
                diamond_power_compile_sprites_raw_cycles_sub_3f798(*work).map(M68kCycleCount::get),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                1_697,
                ModeLevelDiamondPowerCompileWorkSub3F798 {
                    buffered_entry_count: 0,
                },
                Some(56),
            ),
            (
                1_703,
                ModeLevelDiamondPowerCompileWorkSub3F798 {
                    buffered_entry_count: 0,
                },
                Some(56),
            ),
            (
                1_709,
                ModeLevelDiamondPowerCompileWorkSub3F798 {
                    buffered_entry_count: 0,
                },
                Some(56),
            ),
        ],
        "ASM DiamondPower_CompileSprites empty-buffer path",
    );
    assert_eq!(
        observed_kid_projectile_collision_work
            .iter()
            .map(|(frame, work)| (
                *frame,
                *work,
                kid_projectile_collision_raw_cycles_sub_1d76(*work).map(M68kCycleCount::get),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                1_697,
                ModeLevelKidProjectileCollisionWorkSub1D76 {
                    kid_projectile_object_count: 0,
                    diamond_power_object_count: 0,
                },
                Some(126),
            ),
            (
                1_703,
                ModeLevelKidProjectileCollisionWorkSub1D76 {
                    kid_projectile_object_count: 0,
                    diamond_power_object_count: 0,
                },
                Some(126),
            ),
            (
                1_709,
                ModeLevelKidProjectileCollisionWorkSub1D76 {
                    kid_projectile_object_count: 0,
                    diamond_power_object_count: 0,
                },
                Some(126),
            ),
        ],
        "ASM sub_1D76 two empty sub_1D84 object-list paths",
    );
    assert_eq!(
        observed_gfx_move_animate_work
            .iter()
            .map(|(frame, work)| (
                *frame,
                gfx_objects_move_and_animate_raw_cycles_sub_1444(work).map(M68kCycleCount::get),
            ))
            .collect::<Vec<_>>(),
        vec![
            (1_697, Some(4_500)),
            (1_703, Some(4_894)),
            (1_709, Some(4_594)),
        ],
        "ASM sub_1444 derives every stationary prize-worker animation script state",
    );
    assert_eq!(
        observed_gfx_sprite_work
            .iter()
            .map(|(frame, work)| (
                *frame,
                make_sprites_from_gfx_objects_raw_cycles_sub_1492(work).map(M68kCycleCount::get),
            ))
            .collect::<Vec<_>>(),
        vec![
            (1_697, Some(11_628)),
            (1_703, Some(12_554)),
            (1_709, Some(13_240)),
        ],
        "ASM sub_1492 derives each visible, culled, and hidden linked GfxObject path",
    );
    assert_eq!(
        observed_gfx_collision_work
            .iter()
            .map(|(frame, work)| {
                (
                    *frame,
                    work.kid_projectile_list_f866,
                    work.object_kind_two_list_f86a,
                    work.kid_is_invulnerable,
                    gfx_objects_collision_raw_cycles_sub_16f0(work.object_kind_two_list_f86a)
                        .map(M68kCycleCount::get),
                    gfx_objects_collision_raw_cycles_sub_16f0(work.kid_projectile_list_f866)
                        .map(M68kCycleCount::get),
                    gfx_objects_collision_kid_raw_cycles_sub_1b76(
                        work.kid_projectile_list_f866,
                        work,
                    )
                    .map(M68kCycleCount::get),
                    gfx_objects_collision_kid_raw_cycles_sub_1b76(
                        work.object_kind_two_list_f86a,
                        work,
                    )
                    .map(M68kCycleCount::get),
                    gfx_objects_collision_kid_f86e_raw_cycles_sub_1b76(work)
                        .map(M68kCycleCount::get),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (
                1_697,
                ModeLevelGfxCollisionListWork::Empty,
                ModeLevelGfxCollisionListWork::Empty,
                false,
                Some(330),
                Some(330),
                Some(166),
                Some(166),
                Some(2_574),
            ),
            (
                1_703,
                ModeLevelGfxCollisionListWork::Empty,
                ModeLevelGfxCollisionListWork::Empty,
                false,
                Some(330),
                Some(330),
                Some(166),
                Some(166),
                Some(2_780),
            ),
            (
                1_709,
                ModeLevelGfxCollisionListWork::Empty,
                ModeLevelGfxCollisionListWork::Empty,
                false,
                Some(330),
                Some(330),
                Some(166),
                Some(166),
                Some(2_718),
            ),
        ],
        "ASM sub_16F0/sub_1B76 selected-PC paths, including live `$F86E` nodes",
    );
    assert_eq!(
        observed_terrain_special_scan
            .iter()
            .map(|(frame, scan)| {
                let scan = scan.expect("published helper should expose sub_1FA2 terrain scan");
                (
                    *frame,
                    scan.bounds.right_col - scan.bounds.left_col + 1,
                    scan.bounds.bottom_row - scan.bounds.top_row + 1,
                    scan.tile_iterations,
                    scan.first_special_collision,
                    terrain_special_scan_raw_cycles_sub_1fa2(scan).map(M68kCycleCount::get),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (1_697, 2, 2, 4, None, Some(558)),
            (1_703, 2, 2, 4, None, Some(558)),
            (1_709, 2, 3, 6, None, Some(688)),
        ],
        "ASM sub_1FA2 inclusive clear terrain-cell walks",
    );
}

#[test]
fn published_tas_helpers_1696_1702_1708_derive_live_mode_tail_work() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut observed = Vec::new();

    for (frame_index, mask) in script.iter().copied().enumerate().take(1_709) {
        runtime.step_frame(FrameInput::from_mask(mask));
        if matches!(frame_index, 1_696 | 1_702 | 1_708) {
            let work = runtime
                .mode_level_tail_work_loc_892()
                .expect("published natural Mode_Level tail should classify");
            observed.push((
                frame_index,
                work.background_loc_3038a
                    .piece_rows_sub_30744
                    .rendered_nonnegative_entries,
                mode_level_tail_raw_cycles_to_wait_for_vint_loc_892(&work)
                    .expect("published helper tail has no external waits")
                    .get(),
            ));
        }
    }

    assert_eq!(
        observed,
        vec![
            (1_696, 29, 12_852),
            (1_702, 29, 13_080),
            (1_708, 22, 12_310),
        ],
        "ASM loc_892 live state should reproduce the oracle helper ledgers",
    );
}

#[test]
fn published_tas_frame_902_derives_first_gameplay_mode_tail_work() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut frame_902 = None;

    for (frame_index, mask) in script.iter().copied().enumerate().take(903) {
        runtime.step_frame(FrameInput::from_mask(mask));
        if frame_index == 902 {
            frame_902 = runtime.mode_level_tail_work_loc_892();
        }
    }

    let work = frame_902.expect("published frame 902 reaches Mode_Level loc_892");
    assert_eq!(
        work.camera_stripe_sub_30194,
        ModeLevelTailCameraStripeSub30194::UpwardHorizontalNoWrap {
            ordinary_theme_cells: 20,
            priority_theme_cells: 0,
            block_cells: 1,
        },
        "ASM sub_30194 loc_30354/loc_3037C: oracle frame 902 uploads 20 ordinary cells and one block cell",
    );
    assert_eq!(
        work.background_loc_3038a.piece_refresh_sub_307d8,
        ModeLevelTailPieceRefreshSub307D8::CachedRowUnchanged,
        "ASM sub_307D8 loc_3080E -> return_308B4: the first gameplay row is already cached",
    );
    assert_eq!(
        work.palette_sub_14c0,
        ModeLevelTailPaletteWorkSub14C0::BothMasksFull,
        "ASM sub_73D0/loc_7414 -> sub_14C0: both first-gameplay palette masks are full",
    );
    assert_eq!(
        mode_level_tail_raw_cycles_to_wait_for_vint_loc_892(&work),
        Some(M68kCycleCount::from_raw(31_176)),
        "ASM loc_892 -> WaitForVint $93E: oracle frame 902 has 31,176 raw source cycles",
    );
}

#[test]
fn published_tas_demo_bootstrap_presents_hud_timer_before_authoritative_timer_tail() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut frame_962 = None;

    for (frame_index, mask) in script.iter().copied().enumerate().take(963) {
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        if frame_index == 962 {
            frame_962 = Some((
                trace.game_mode,
                trace.current_level,
                trace.level_timer_seconds,
                trace.level_timer_sub_frames,
                trace.hud_timer_sprite_seconds_loc_bee8,
            ));
        }
    }

    // ASM Load_DemoPlay -> Load_InGame sub_8E0 reaches Make_SpriteAttr_HUD
    // loc_BE82 on every bootstrap pass, while loc_3038A remains suspended.
    // Oracle frame 962 therefore displays 2:59 even though native's
    // authoritative timer has not reached its delayed gameplay-tail tick.
    assert_eq!(
        frame_962,
        Some((GameMode::DemoPlay, LevelId(0x1B), 180, 6, Some(179))),
    );
}

#[test]
fn published_tas_sega_start_handoff_presents_sub_1c204_fade_one_frame_later() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut samples = Vec::new();

    for (frame_index, mask) in script.iter().copied().enumerate().take(85) {
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        if matches!(frame_index, 82..=84) {
            samples.push((
                frame_index,
                trace.game_mode,
                trace.boot_phase_elapsed,
                runtime.vdp_state().cloned(),
            ));
        }
    }

    // ASM Load_SegaScreen loc_12EDE changes Game_Mode after the frame-79
    // Start edge. Load_IntroSequence1's sub_1B8F4 enters sub_1C204, whose
    // first Pal_1C28A palette-buffer update is not presented until the
    // following VBlank. Oracle therefore keeps black through published
    // frame 82, presents the first fade step on frame 83, and advances it
    // again on frame 84.
    assert_eq!(samples[0].1, GameMode::IntroSequence1);
    assert_eq!(samples[1].1, GameMode::IntroSequence1);
    assert_eq!(samples[2].1, GameMode::IntroSequence1);
    assert_eq!(samples[0].2, Some(46));
    assert_eq!(samples[1].2, Some(47));
    assert_eq!(samples[2].2, Some(48));
    assert!(
        samples[0]
            .3
            .as_ref()
            .is_some_and(|vdp| vdp.cram.iter().all(|byte| *byte == 0)),
    );
    assert!(
        samples[1]
            .3
            .as_ref()
            .is_some_and(|vdp| vdp.cram.iter().any(|byte| *byte != 0)),
    );
    assert_ne!(
        samples[2].3.as_ref().map(|vdp| &vdp.cram),
        samples[1].3.as_ref().map(|vdp| &vdp.cram),
    );
}

#[test]
fn published_tas_cold_intro_1_title_restore_stages_load_title_screen_loc_1b258() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut expected_resident_vdp = None;
    let mut observed_fade_vdp = Vec::new();
    let mut frame_195 = None;
    let mut frame_196 = None;
    let mut frame_197 = None;

    for (frame_index, mask) in script.iter().copied().enumerate().take(198) {
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        match frame_index {
            138 => {
                let elapsed = trace
                    .boot_phase_elapsed
                    .expect("frame 138 should expose IntroSequence1 elapsed");
                expected_resident_vdp = runtime.intro_sub_1b850_data.as_ref().and_then(|data| {
                    runtime.sega_screen_data.as_ref().map(|sega_data| {
                        // ASM sub_1B8F4 loc_1B924 hibernates before polling
                        // sub_1C246. The final deferred loc_1C7A0 object pass
                        // adds the next text glyph before loc_1C264 enters
                        // loc_6E2's Pal_FadeOut.
                        vdp::build_intro_sequence_1_cold_boot_vdp_at_elapsed(
                            data,
                            sega_data,
                            runtime.title_screen_region_byte,
                            elapsed + 1,
                        )
                    })
                });
            }
            139 | 142 | 143 | 156 | 157 => {
                observed_fade_vdp.push((frame_index, runtime.vdp_state().cloned()));
            }
            195 => frame_195 = runtime.vdp_state().cloned(),
            196 => {
                frame_196 = Some((
                    trace.boot_phase,
                    trace.boot_phase_elapsed,
                    runtime.vdp_state().cloned(),
                ));
            }
            197 => {
                frame_197 = Some((
                    trace.boot_phase,
                    trace.boot_phase_elapsed,
                    runtime.vdp_state().cloned(),
                ));
            }
            _ => {}
        }
    }

    let resident_vdp = expected_resident_vdp
        .expect("cached generated manifest should contain cold IntroSequence1 data");
    let faded_resident_vdp = |elapsed| {
        let mut vdp = resident_vdp.clone();
        assert!(vdp::apply_intro_title_return_fade_to_black_loc_28c6(
            &mut vdp, elapsed,
        ));
        vdp
    };
    let expected_fade_vdp = [
        (139, resident_vdp.clone()),
        (142, resident_vdp.clone()),
        (143, faded_resident_vdp(4)),
        (156, faded_resident_vdp(17)),
        (157, vdp::blank_vdp_state()),
    ];

    for ((observed_frame, observed), (expected_frame, expected)) in
        observed_fade_vdp.iter().zip(expected_fade_vdp.iter())
    {
        assert_eq!(observed_frame, expected_frame);
        assert!(
            observed.as_ref() == Some(expected),
            "ASM loc_1B924 -> loc_1C264 -> loc_6E2 cold title-return VDP diverged at published frame {observed_frame}",
        );
    }

    let title_data = runtime
        .title_screen_data
        .as_ref()
        .expect("cached generated manifest should contain title-screen data");
    let lineage = vdp::TitleScreenVdpLineage {
        intro_return_hidden_shell: true,
        fresh_title_worker_lineage: true,
        ..vdp::TitleScreenVdpLineage::default()
    };
    let expected_loader_frame =
        vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
            title_data,
            7,
            runtime.title_screen_region_byte,
            None,
            false,
            57,
            lineage,
        );
    let expected_first_object_pass =
        vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
            title_data,
            7,
            runtime.title_screen_region_byte,
            None,
            true,
            58,
            lineage,
        );

    // ASM sub_1C246 -> loc_1C264 -> loc_6E2 retains and fades the final intro
    // VDP, then runs the blocking title loader. Oracle frame 195 is the final
    // black hold, frame 196 presents Load_TitleScreen's base upload, and frame
    // 197 exposes the first fresh Obj_TitleTextKid/Obj_TitleTextChameleon pass.
    // The Start tap on frame 181 occurred inside the blocking loader, so no
    // menu is drawn.
    assert_eq!(frame_195, Some(vdp::blank_vdp_state()));
    assert_eq!(
        frame_196,
        Some((
            Some(String::from("title_screen")),
            Some(57),
            Some(expected_loader_frame),
        )),
    );
    assert_eq!(
        frame_197,
        Some((
            Some(String::from("title_screen")),
            Some(58),
            Some(expected_first_object_pass),
        )),
    );
}

fn cached_issue260_floor_probe_route_script() -> Option<Vec<u8>> {
    if !cache_backed_tests_enabled() {
        eprintln!(
            "skipping cache-backed source-port test: set KID_ENABLE_CACHE_BACKED_TESTS=1 to use generated .cache assets"
        );
        return None;
    }
    let scenario_path = repo_root()
        .join(".cache/debug/issue260-route-probes/floor-explicit-left-drips-close-617.json");
    if !scenario_path.is_file() {
        eprintln!(
            "skipping cache-backed source-port test: issue #260 route scenario missing at {}",
            scenario_path.display()
        );
        return None;
    }
    let scenario_raw = fs::read_to_string(&scenario_path).expect("read issue #260 route scenario");
    let scenario_json: serde_json::Value =
        serde_json::from_str(&scenario_raw).expect("parse issue #260 route scenario");
    Some(
        scenario_json["script"]
            .as_array()
            .expect("issue #260 route script array")
            .iter()
            .map(|value| value.as_u64().expect("issue #260 route script byte") as u8)
            .collect(),
    )
}

fn cached_issue289_level06_walk_right_script() -> Option<(LevelId, Vec<u8>)> {
    if !cache_backed_tests_enabled() {
        eprintln!(
            "skipping cache-backed source-port test: set KID_ENABLE_CACHE_BACKED_TESTS=1 to use generated .cache assets"
        );
        return None;
    }
    let scenario_path = repo_root()
        .join(".cache/recordings/issue-289-level06-walk-right-20260618-current/scenario.json");
    if !scenario_path.is_file() {
        eprintln!(
            "skipping cache-backed source-port test: issue #289 route scenario missing at {}",
            scenario_path.display()
        );
        return None;
    }
    let scenario_raw = fs::read_to_string(&scenario_path).expect("read issue #289 route scenario");
    let scenario_json: serde_json::Value =
        serde_json::from_str(&scenario_raw).expect("parse issue #289 route scenario");
    let level = scenario_json["warp_to_level"]
        .as_u64()
        .map(|level| LevelId(level as u8))
        .expect("issue #289 route should declare warp_to_level");
    Some((
        level,
        scenario_json["script"]
            .as_array()
            .expect("issue #289 route script array")
            .iter()
            .map(|value| value.as_u64().expect("issue #289 route script byte") as u8)
            .collect(),
    ))
}

fn cached_issue230_crypt38_walk_right_script() -> Option<(LevelId, Vec<u8>)> {
    if !cache_backed_tests_enabled() {
        eprintln!(
            "skipping cache-backed source-port test: set KID_ENABLE_CACHE_BACKED_TESTS=1 to use generated .cache assets"
        );
        return None;
    }
    let scenario_path =
        repo_root().join(".cache/recordings/issue230-crypt38-walk-right/scenario.json");
    if !scenario_path.is_file() {
        eprintln!(
            "skipping cache-backed source-port test: issue #230 route scenario missing at {}",
            scenario_path.display()
        );
        return None;
    }
    let scenario_raw = fs::read_to_string(&scenario_path).expect("read issue #230 route scenario");
    let scenario_json: serde_json::Value =
        serde_json::from_str(&scenario_raw).expect("parse issue #230 route scenario");
    let level = scenario_json["warp_to_level"]
        .as_u64()
        .map(|level| LevelId(level as u8))
        .expect("issue #230 route should declare warp_to_level");
    Some((
        level,
        scenario_json["script"]
            .as_array()
            .expect("issue #230 route script array")
            .iter()
            .map(|value| value.as_u64().expect("issue #230 route script byte") as u8)
            .collect(),
    ))
}

#[test]
fn tas_title_cursor_moves_do_not_delay_attract_demo_handoff() {
    let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
        return;
    };
    let mut frame_834 = None;
    let mut frame_835 = None;

    for (frame_index, mask) in script.iter().copied().enumerate().take(836) {
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        match frame_index {
            834 => {
                frame_834 = Some((trace.game_mode, trace.current_level, trace.options_cursor));
            }
            835 => {
                frame_835 = Some((trace.game_mode, trace.current_level, trace.options_cursor));
                break;
            }
            _ => {}
        }
    }

    assert_eq!(
        frame_834,
        Some((GameMode::TitleScreen, LevelId(0), Some(2))),
        "ASM Obj_TitleMenu Title_InputLoop should keep the moved cursor visible until the attract countdown expires",
    );
    assert_eq!(
        frame_835,
        Some((GameMode::DemoPlay, LevelId(0), None)),
        "ASM loc_1BB02 should enter demo play on the oracle-backed TAS frame even after cursor moves",
    );
}

#[test]
fn published_tas_title_to_demo_replays_loc_28c6_resident_fade() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut resident_vdp = None;
    let mut observed = Vec::new();

    for (frame_index, mask) in script.iter().copied().enumerate().take(856) {
        runtime.step_frame(FrameInput::from_mask(mask));
        match frame_index {
            836 => resident_vdp = runtime.vdp_state().cloned(),
            837 | 840 | 841 | 854 | 855 => {
                observed.push((frame_index, runtime.vdp_state().cloned()));
            }
            _ => {}
        }
    }

    let resident_vdp = resident_vdp.expect("frame 836 should expose the completed title VDP");
    let faded_resident_vdp = |elapsed| {
        let mut vdp = resident_vdp.clone();
        assert!(vdp::apply_title_to_demo_fade_to_black_loc_28c6(
            &mut vdp, elapsed,
        ));
        vdp
    };
    let expected = [
        (837, resident_vdp.clone()),
        (840, faded_resident_vdp(3)),
        (841, faded_resident_vdp(4)),
        (854, faded_resident_vdp(17)),
        (855, vdp::blank_vdp_state()),
    ];

    for ((observed_frame, observed), (expected_frame, expected)) in
        observed.iter().zip(expected.iter())
    {
        assert_eq!(observed_frame, expected_frame);
        assert!(
            observed.as_ref() == Some(expected),
            "ASM Obj_TitleMenu loc_1BB02 -> loc_6E2 -> Pal_FadeOut diverged at published frame {observed_frame}",
        );
    }
}

#[test]
fn published_tas_demo_timeout_return_keeps_outgoing_render_level_through_loc_28c6() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut samples = Vec::new();

    for (frame_index, mask) in script.iter().copied().enumerate().take(12_153) {
        runtime.step_frame(FrameInput::from_mask(mask));
        if matches!(frame_index, 12_133 | 12_142 | 12_151 | 12_152) {
            samples.push((
                frame_index,
                runtime.source_port_render_level(),
                runtime.demo_title_palette_fade_progress_loc_28c6(),
            ));
        }
    }

    // ASM demo timeout ReadJoypad loc_C0A -> loc_6E2 retains the outgoing
    // Knights Isle planes through Pal_FadeOut even though Current_LevelID has
    // already been reset to zero for the following title/intro shell.
    assert_eq!(
        samples,
        vec![
            (12_133, LevelId(0x1B), Some(0x0100)),
            (12_142, LevelId(0x1B), Some(0x00A0)),
            (12_151, LevelId(0x1B), Some(0x0010)),
            (12_152, LevelId(0x00), None),
        ]
    );
}

#[test]
fn published_tas_knights_isle_demo_runs_island_shore_object_loc_11fb0_1202a() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut samples = Vec::new();
    for (frame_index, mask) in script.iter().copied().enumerate().take(3_127) {
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        if matches!(frame_index, 3_120 | 3_126) {
            samples.push((
                frame_index,
                trace.game_mode,
                trace.current_level,
                runtime.source_port_render_level(),
                runtime.demo_title_palette_fade_progress_loc_28c6(),
                trace.island_shore_cycle_active,
                trace.island_shore_vram_frames_loc_1202a,
                trace.resident_gameplay_vdp_camera_loc_300ba,
            ));
        }
    }

    // ASM Load_DemoPlay -> Load_InGame loc_11FB0 allocates loc_1202A for the
    // Island background. The synchronous Pal_FadeOut shell freezes and keeps
    // that outgoing object state while Current_LevelID is already zero.
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].0, 3_120);
    assert_eq!(samples[0].1, GameMode::DemoPlay);
    assert_eq!(samples[0].2, LevelId(0x15));
    assert_eq!(samples[0].3, LevelId(0x15));
    assert_eq!(samples[0].4, None);
    assert!(samples[0].5);
    assert!(samples[0].6.is_some());
    assert_eq!(samples[0].7, None);

    assert_eq!(samples[1].0, 3_126);
    assert_eq!(samples[1].1, GameMode::TitleScreen);
    assert_eq!(samples[1].2, LevelId(0x00));
    assert_eq!(samples[1].3, LevelId(0x15));
    assert_eq!(samples[1].4, Some(0x00F0));
    assert!(samples[1].5);
    assert_eq!(samples[1].6, samples[0].6);
    assert_eq!(samples[1].7, Some((0x01E8_0000, 0x011C_0000)));
}

#[test]
fn published_tas_island_shore_dma_clock_tracks_demo_lineages_loc_1202a() {
    let Some((mut runtime, script)) = cached_tas_runtime_and_script("any-percent-published") else {
        return;
    };
    let mut samples = Vec::new();
    for (frame_index, mask) in script.iter().copied().enumerate().take(22_153) {
        let trace = runtime.step_frame(FrameInput::from_mask(mask));
        if matches!(
            frame_index,
            2_921 | 2_922 | 2_923 | 7_604 | 7_605 | 22_151 | 22_152
        ) {
            samples.push((
                frame_index,
                trace
                    .island_shore_dma_uploads_loc_1202a
                    .iter()
                    .map(|upload| upload.vram_word_address)
                    .collect::<Vec<_>>(),
                trace.island_shore_post_visible_dma_latch_loc_1202a,
            ));
        }
    }

    assert_eq!(
        samples,
        vec![
            (2_921, vec![], false),
            (2_922, vec![0xF600, 0xF760], false),
            (2_923, vec![], false),
            (7_604, vec![], false),
            (7_605, vec![], false),
            (22_151, vec![], false),
            (22_152, vec![0xF600, 0xF760], true),
        ],
        "ASM loc_12098/loc_12110/loc_12188 ordinary post-update events are already one frame behind the FIFO commit, while the IntroSequence6 hidden-camera-wrap event is frame-aligned and needs a post-visible render latch"
    );
}

#[test]
fn cold_boot_intro_sequence_6_enters_demo_play_on_frame_3270() {
    let mut runtime = NativeRuntime::new(manifest());
    let mut frame_3269 = None;
    let mut frame_3270 = None;
    let mut frame_3327 = None;
    let mut frame_3328 = None;
    let mut frame_3329 = None;

    for frame_index in 0..=3_329 {
        let trace = runtime.step_frame(FrameInput::default());
        match frame_index {
            3_269 => {
                frame_3269 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                ));
            }
            3_270 => {
                frame_3270 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                ));
            }
            3_327 => {
                frame_3327 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                ));
            }
            3_328 => {
                frame_3328 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                ));
            }
            3_329 => {
                frame_3329 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                ));
            }
            _ => {}
        }
    }

    // ASM loc_1BA5C enters Load_IntroSequence6, loc_1BAB0 burns the `$B8`
    // pre-roll, then Obj_TitleMenu `loc_1BAEA/loc_1BAEE` owns the `$280`
    // idle loop before `loc_1BB02` writes `Game_Mode=$10`.
    assert_eq!(
        frame_3269,
        Some((GameMode::IntroSequence6, LevelId(0x00), Some(873))),
        "frame 3269 should still be the final cold-boot mode-$28 title frame",
    );
    assert_eq!(
        frame_3270,
        Some((GameMode::DemoPlay, LevelId(0x00), Some(0))),
        "frame 3270 should be the first cold-boot idle demo-play frame",
    );
    assert_eq!(
        frame_3327,
        Some((GameMode::DemoPlay, LevelId(0x1B), Some(57), 0x01E8_0000)),
        "frame 3327 should expose the cold-boot Skydragon preload camera row",
    );
    assert_eq!(
        frame_3328,
        Some((GameMode::DemoPlay, LevelId(0x1B), Some(58), 0x01C0_0000)),
        "frame 3328 should keep the second cold-boot Skydragon preload camera row",
    );
    assert_eq!(
        frame_3329,
        Some((GameMode::DemoPlay, LevelId(0x1B), Some(59), 0x02A0_0000)),
        "frame 3329 should settle to Load_InGame's respawn camera snap",
    );
}

#[test]
fn erase_block_from_level_layout_preserves_forest_skin_loc_11542() {
    let Some(mut runtime) = cached_generated_runtime() else {
        return;
    };
    runtime.warp_to_level(LevelId(0x00));
    let grid = runtime.block_grid.as_mut().expect("generated level 0 grid");
    let (block_col, block_row, erase_cell) = (0..grid.height_cells)
        .flat_map(|row| (0..grid.width_cells).map(move |col| (col, row)))
        .find_map(|(col, row)| {
            let cell = grid.cell_at(col, row);
            let erase_cell = grid.erase_cell_at_loc_11542(col, row);
            (cell.has_block && erase_cell != terrain::Cell::from_be_bytes(0, 0))
                .then_some((col, row, erase_cell))
        })
        .expect("generated forest level should contain a block over visible foreground");

    assert_ne!(
        erase_cell,
        terrain::Cell::from_be_bytes(0, 0),
        "ASM EraseBlockFromLevelLayout loc_11542 should restore the decoded forest foreground skin, not blank the cell"
    );
    assert!(
        !erase_cell.has_block,
        "ASM loc_11542 rebuilds a plain terrain cell before writing it back"
    );

    grid.set_cell_at(
        block_col,
        block_row,
        terrain::Cell::from_be_bytes(0xE0, erase_cell.skin),
    );
    grid.erase_block_from_level_layout_loc_11542(block_col, block_row);

    assert_eq!(
        grid.cell_at(block_col, block_row),
        erase_cell,
        "ASM loc_11542 keeps the forest skin byte so smashing blocks does not punch visible gaps through leaves"
    );
}

#[test]
fn cold_boot_demo_timeout_intro_sequence_6_exposes_hidden_shell_on_frame_6850() {
    let Some(mut runtime) = cached_generated_runtime() else {
        return;
    };
    let mut frame_6849 = None;
    let mut frame_6850 = None;
    let mut frame_6851 = None;
    let mut frame_7012 = None;
    let mut frame_7013 = None;
    let mut frame_7053 = None;
    let mut frame_7054 = None;
    let mut frame_7066 = None;
    let mut frame_7081 = None;
    let mut frame_7085 = None;
    let mut frame_7147 = None;
    let mut frame_7148 = None;
    let mut frame_7688 = None;
    let mut frame_7689 = None;
    let mut frame_7748 = None;
    let mut frame_7749 = None;
    let mut frame_8851 = None;
    let mut frame_8852 = None;
    let mut frame_8853 = None;

    for frame_index in 0..=8_853 {
        let trace = runtime.step_frame(FrameInput::default());
        match frame_index {
            6_849 => {
                frame_6849 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                    trace.number_objects,
                ));
            }
            6_850 => {
                frame_6850 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                    trace.number_objects,
                ));
            }
            6_851 => {
                frame_6851 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                    trace.number_objects,
                ));
            }
            7_012 => {
                frame_7012 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                    trace.number_objects,
                ));
            }
            7_013 => {
                frame_7013 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                    trace.number_objects,
                ));
            }
            7_053 => {
                frame_7053 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                    trace.number_objects,
                ));
            }
            7_054 => {
                frame_7054 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                    trace.number_objects,
                ));
            }
            7_066 => {
                frame_7066 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                    trace.number_objects,
                ));
            }
            7_081 => {
                frame_7081 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                    trace.number_objects,
                ));
            }
            7_085 => {
                frame_7085 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.camera_y_pos,
                    trace.number_objects,
                ));
            }
            7_147 => {
                frame_7147 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                ));
            }
            7_148 => {
                frame_7148 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                ));
            }
            7_688 => {
                frame_7688 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.kid_y_velocity,
                ));
            }
            7_689 => {
                frame_7689 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.kid_y_velocity,
                ));
            }
            7_748 => {
                frame_7748 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.number_objects,
                    trace.camera_y_pos,
                ));
            }
            7_749 => {
                frame_7749 = Some((
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.number_objects,
                    trace.camera_y_pos,
                ));
            }
            8_851 => {
                frame_8851 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.kid_y_velocity,
                ));
            }
            8_852 => {
                frame_8852 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.kid_y_velocity,
                ));
            }
            8_853 => {
                frame_8853 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.boot_phase_elapsed,
                    trace.number_objects,
                    trace.number_gfx_objects,
                    trace.character_movement,
                ));
            }
            _ => {}
        }
    }
    // ASM ReadJoypad loc_C0A restarts the intro loop after the demo timeout.
    // When loc_1BA5C reaches Load_IntroSequence6 again, loc_1B1E8 has already
    // called sub_1B222 with `$FFFFA656=$FFD00000` and allocated the three
    // hidden title-shell workers (`loc_1BAB0`, `loc_1B37C`, `sub_1B43A`).
    assert_eq!(
        frame_6849,
        Some((GameMode::IntroSequence6, LevelId(0x00), Some(33), 0, 0)),
        "frame 6849 should still keep the cold timeout-restart mode-$28 hidden shell cleared",
    );
    assert_eq!(
        frame_6850,
        Some((
            GameMode::IntroSequence6,
            LevelId(0x00),
            Some(34),
            0xFFD0_0000,
            3,
        )),
        "frame 6850 should expose Load_IntroSequence6's seeded hidden camera and three workers",
    );
    assert_eq!(
        frame_6851,
        Some((
            GameMode::IntroSequence6,
            LevelId(0x00),
            Some(35),
            0xFFD0_0000,
            3,
        )),
        "frame 6851 should keep the hidden workers and seeded camera row",
    );
    assert_eq!(
        frame_7012,
        Some((GameMode::IntroSequence6, Some(196), 0xFFD0_0000, 3)),
        "frame 7012 should still be the final three-worker hidden shell frame",
    );
    assert_eq!(
        frame_7013,
        Some((GameMode::IntroSequence6, Some(197), 0xFFD0_0000, 2)),
        "frame 7013 should enter sub_1B43A's two-worker palette/art phase",
    );
    assert_eq!(
        frame_7053,
        Some((GameMode::IntroSequence6, Some(237), 0xFFD0_0000, 2)),
        "frame 7053 should be the final two-worker hidden shell frame",
    );
    assert_eq!(
        frame_7054,
        Some((GameMode::IntroSequence6, Some(238), 0xFFD0_0000, 4)),
        "frame 7054 should expose the GfxObjects_Collision_BottomBoundary4 allocation",
    );
    assert_eq!(
        frame_7066,
        Some((GameMode::IntroSequence6, Some(250), 0xFFD3_0000, 4)),
        "frame 7066 should start the cold timeout-return +3px hidden camera ramp",
    );
    assert_eq!(
        frame_7081,
        Some((GameMode::IntroSequence6, Some(265), 0, 3)),
        "frame 7081 should drop back to three workers after GfxObjects_Collision_BottomBoundary4 deletes",
    );
    assert_eq!(
        frame_7085,
        Some((GameMode::IntroSequence6, Some(269), 0, 4)),
        "frame 7085 should restore the four-object hidden title shell",
    );
    assert_eq!(
        frame_7147,
        Some((GameMode::IntroSequence6, Some(331), None, None)),
        "frame 7147 should still hide the rebuilt loc_1B5CE title pose",
    );
    assert_eq!(
        frame_7148,
        Some((
            GameMode::IntroSequence6,
            Some(332),
            Some(0x002E_0000),
            Some(0x008C_0000),
        )),
        "frame 7148 should expose loc_1B5CE's rebuilt title pose on the cold timeout-return shell",
    );
    assert_eq!(
        frame_7688,
        Some((
            GameMode::IntroSequence6,
            LevelId(0x00),
            Some(872),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
            Some(0),
        )),
        "frame 7688 should still be the final cold timeout-return mode-$28 title frame",
    );
    assert_eq!(
        frame_7689,
        Some((
            GameMode::DemoPlay,
            LevelId(0x00),
            Some(0),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
            Some(0x0006_0000),
        )),
        "frame 7689 should enter demo play through loc_1BB02 with the controller-side y velocity sample",
    );
    assert_eq!(
        frame_7748,
        Some((GameMode::DemoPlay, Some(59), 2, 0x00E0_0000)),
        "frame 7748 should expose the cold timeout-return controller object count before the camera wrap",
    );
    assert_eq!(
        frame_7749,
        Some((GameMode::DemoPlay, Some(60), 2, 0xFD40_0000)),
        "frame 7749 should expose the cold timeout-return Knights Isle hidden camera row",
    );
    assert_eq!(
        frame_8851,
        Some((
            GameMode::DemoPlay,
            LevelId(0x15),
            Some(1079),
            Some(0x039B_7C80),
            Some(0x0185_7680),
            Some(0),
        )),
        "frame 8851 should still expose the final cold timeout-return Knights Isle demo step",
    );
    assert_eq!(
        frame_8852,
        Some((
            GameMode::DemoPlay,
            LevelId(0x15),
            Some(1079),
            Some(0x0399_D280),
            Some(0x0186_B680),
            Some(0),
        )),
        "frame 8852 should replay the final buffered demo step before ReadJoypad's deferred timeout return",
    );
    assert_eq!(
        frame_8853,
        Some((GameMode::SegaScreen, LevelId(0x00), None, 9, 4, Some(3))),
        "frame 8853 should apply ReadJoypad loc_C0A's mode-0 timeout return while preserving the final demo jump snapshot",
    );
}

fn runtime_with_projectile_and_player(
    proj_x: u16,
    proj_y: u16,
    player_x: u16,
    player_y: u16,
) -> NativeRuntime {
    let mut rt = NativeRuntime::new(manifest());
    rt.projectiles.push(make_projectile(proj_x, proj_y));
    let mut slice = OpeningSliceState::new(default_bounds());
    slice.player.x_pos = u32::from(player_x) << 16;
    slice.player.y_pos = u32::from(player_y) << 16;
    rt.opening_slice = Some(slice);
    rt
}

mod chunk_09 {
    use super::*;

    #[test]
    fn demo_1_bootstrap_keeps_camera_hidden_through_frame_2537() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_538) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (2_529..=2_537).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_2529 = frames.get(&2_529).expect("frame 2529 should be captured");
        assert_eq!(frame_2529.current_level, LevelId(0x15));
        assert_eq!(frame_2529.camera_y_pos, 0);
        assert_eq!(frame_2529.kid_x_pos, None);
        assert_eq!(frame_2529.kid_y_pos, None);

        let frame_2530 = frames.get(&2_530).expect("frame 2530 should be captured");
        assert_eq!(frame_2530.camera_y_pos, 0);

        let frame_2531 = frames.get(&2_531).expect("frame 2531 should be captured");
        assert_eq!(frame_2531.camera_y_pos, 0x00E0_0000);
        assert_eq!(frame_2531.kid_x_pos, None);
        assert_eq!(frame_2531.kid_y_pos, None);

        let frame_2532 = frames.get(&2_532).expect("frame 2532 should be captured");
        assert_eq!(frame_2532.camera_y_pos, 0xFF20_0000);

        let frame_2537 = frames.get(&2_537).expect("frame 2537 should be captured");
        assert_eq!(frame_2537.camera_y_pos, 0x00E0_0000);
        assert_eq!(frame_2537.kid_x_pos, None);
        assert_eq!(frame_2537.kid_y_pos, None);
    }

    #[test]
    fn demo_1_bootstrap_holds_pose_and_input_until_frame_2571() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_578) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if [2_544usize, 2_553, 2_571, 2_577].contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_2544 = frames.get(&2_544).expect("frame 2544 should be captured");
        assert_eq!(frame_2544.camera_y_pos, 0x00D8_0000);
        assert_eq!(frame_2544.kid_x_pos, Some(0x0010_0000));
        assert_eq!(frame_2544.kid_y_pos, Some(0x00FF_0000));

        let frame_2553 = frames.get(&2_553).expect("frame 2553 should be captured");
        assert_eq!(frame_2553.camera_y_pos, 0x0091_0000);
        assert_eq!(frame_2553.kid_x_pos, Some(0x0010_0000));
        assert_eq!(frame_2553.kid_y_pos, Some(0x00FF_0000));

        let frame_2571 = frames.get(&2_571).expect("frame 2571 should be captured");
        assert_eq!(frame_2571.camera_y_pos, 0x0064_0000);
        assert_eq!(frame_2571.kid_x_pos, Some(0x0010_0000));

        let frame_2577 = frames.get(&2_577).expect("frame 2577 should be captured");
        assert_eq!(frame_2577.kid_x_pos, Some(0x0010_1000));
    }

    #[test]
    fn demo_1_handoff_stays_airborne_until_frame_2625() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_626) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (2_623..=2_625).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_2623 = frames.get(&2_623).expect("frame 2623 should be captured");
        assert_eq!(frame_2623.kid_x_pos, Some(0x0059_E500));
        assert_eq!(frame_2623.kid_y_pos, Some(0x0139_C000));
        assert_eq!(frame_2623.kid_on_ground, Some(false));

        let frame_2624 = frames.get(&2_624).expect("frame 2624 should be captured");
        assert_eq!(frame_2624.kid_x_pos, Some(0x005C_7700));
        assert_eq!(frame_2624.kid_y_pos, Some(0x013F_4000));
        assert_eq!(
            frame_2624.kid_on_ground,
            Some(false),
            "demo playback should preserve the remaining sub-half-pixel gap on frame 2624",
        );

        let frame_2625 = frames.get(&2_625).expect("frame 2625 should be captured");
        assert_eq!(frame_2625.kid_x_pos, Some(0x005C_7700));
        assert_eq!(frame_2625.kid_y_pos, Some(0x013F_0000));
        assert_eq!(frame_2625.kid_on_ground, Some(true));
    }

    #[test]
    fn demo_1_post_landing_hold_preserves_standingstill_through_frame_2714() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_715) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (2_697..=2_714).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        for frame_index in 2_697..=2_714 {
            let frame = frames
                .get(&frame_index)
                .unwrap_or_else(|| panic!("frame {frame_index} should be captured"));
            assert_eq!(
                frame.kid_x_pos,
                Some(0x011A_0000),
                "demo playback should hold Kid at the landing tile through frame {frame_index}",
            );
            assert_eq!(
                frame.kid_x_velocity,
                Some(0),
                "standingstill should keep horizontal speed at zero on frame {frame_index}",
            );
            assert_eq!(
                frame.kid_on_ground,
                Some(true),
                "Kid should remain grounded during the post-landing standingstill hold on frame {frame_index}",
            );
        }
    }

    #[test]
    fn demo_1_wall_cling_snap_releases_into_run_on_frame_2723() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_726) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (2_723..=2_725).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_2723 = frames
            .get(&2_723)
            .unwrap_or_else(|| panic!("frame 2723 should be captured"));
        assert_eq!(
            frame_2723.kid_x_pos,
            Some(0x011B_0000),
            "demo 1 should shimmy 1px off the wall-cling on frame 2723",
        );

        let frame_2724 = frames
            .get(&2_724)
            .unwrap_or_else(|| panic!("frame 2724 should be captured"));
        assert_eq!(
            frame_2724.kid_x_pos,
            Some(0x011B_4080),
            "the grounded release should hand off into the oracle-backed run arc on frame 2724",
        );

        let frame_2725 = frames
            .get(&2_725)
            .unwrap_or_else(|| panic!("frame 2725 should be captured"));
        assert_eq!(frame_2725.kid_x_pos, Some(0x011B_C180));
    }

    #[test]
    fn demo_1_left_wall_snap_lands_flush_on_frame_2773() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_776) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (2_772..=2_775).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_2772 = frames
            .get(&2_772)
            .unwrap_or_else(|| panic!("frame 2772 should be captured"));
        assert_eq!(frame_2772.kid_x_pos, Some(0x0135_5000));

        let frame_2773 = frames
            .get(&2_773)
            .unwrap_or_else(|| panic!("frame 2773 should be captured"));
        assert_eq!(
            frame_2773.kid_x_pos,
            Some(0x0135_0000),
            "demo 1 should snap flush to the left wall on frame 2773",
        );

        let frame_2774 = frames
            .get(&2_774)
            .unwrap_or_else(|| panic!("frame 2774 should be captured"));
        assert_eq!(
            frame_2774.kid_x_pos,
            Some(0x0135_0000),
            "after the flush snap, the next held-input frame should stay pinned at the wall",
        );
    }

    #[test]
    fn demo_1_buffered_wall_sticky_pullup_lands_on_frame_2831() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_2831 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_832) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 2_831 {
                frame_2831 = Some(trace);
            }
        }

        let frame_2831 = frame_2831.unwrap_or_else(|| panic!("frame 2831 should be captured"));
        assert_eq!(
            frame_2831.kid_x_pos,
            Some(0x0144_0000),
            "demo 1 should pull up onto the prize-block top on frame 2831",
        );
        assert_eq!(
            frame_2831.kid_y_pos,
            Some(0x013F_8000),
            "demo 1 should preserve the oracle-backed pull-up sub-pixel on frame 2831",
        );
        assert_eq!(
            frame_2831.kid_y_velocity,
            Some(0),
            "the pull-up landing should clear vertical speed on frame 2831",
        );
        assert_eq!(
            frame_2831.kid_on_ground,
            Some(true),
            "the buffered pull-up should leave Kid grounded on frame 2831",
        );
    }

    #[test]
    fn demo_1_lands_on_the_walking_dragon_on_frame_2938() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_2938 = None;
        let mut dragon_positions = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_939) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 2_938 {
                frame_2938 = Some(trace);
                dragon_positions = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| (enemy.x_pos, enemy.y_pos))
                    .collect();
                break;
            }
        }

        let frame_2938 = frame_2938.unwrap_or_else(|| panic!("frame 2938 should be captured"));
        assert_eq!(
            frame_2938.kid_x_pos,
            Some(0x01B4_1700),
            "demo 1 should have the oracle-backed X position when landing on the dragon",
        );
        assert_eq!(
            frame_2938.kid_y_pos,
            Some(0x0188_3300),
            "demo 1 should have the oracle-backed Y position when landing on the dragon",
        );
        assert!(
            dragon_positions.contains(&(0x01C2_8000, 0x01B0_0000)),
            "expected the landing dragon at (0x01C28000, 0x01B00000), got {dragon_positions:#x?}",
        );
    }

    #[test]
    fn demo_1_keeps_the_oracle_backed_stomp_drift_on_frame_2994() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_2994 = None;
        let mut frame_3001 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_002) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 2_994 {
                frame_2994 = Some(trace.clone());
            }
            if frame_index == 3_001 {
                frame_3001 = Some(trace);
                break;
            }
        }

        let frame_2994 = frame_2994.unwrap_or_else(|| panic!("frame 2994 should be captured"));
        assert_eq!(
            frame_2994.kid_x_pos,
            Some(0x026B_0000),
            "demo 1 should preserve the post-dragon-stomp horizontal carry on frame 2994",
        );
        assert_eq!(
            frame_2994.kid_y_pos,
            Some(0x019F_5300),
            "frame 2994 should stay on the oracle-backed vertical path after the stomp",
        );

        let frame_3001 = frame_3001.unwrap_or_else(|| panic!("frame 3001 should be captured"));
        assert_eq!(
            frame_3001.kid_x_pos,
            Some(0x026E_8C00),
            "the carried drift should still line up with the oracle by frame 3001",
        );
        assert_eq!(
            frame_3001.camera_x_pos, 0x01C8_0000,
            "camera follow should recover once the carried drift lines the player back up",
        );
    }

    #[test]
    fn demo_1_lower_dragon_holds_the_post_stomp_tile_through_frame_2962() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragon_2962 = None;
        let mut dragon_2963 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_964) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 2_962 {
                dragon_2962 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.y_pos, enemy.facing, enemy.on_ground));
            }
            if frame_index == 2_963 {
                dragon_2963 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.y_pos, enemy.facing, enemy.on_ground));
                break;
            }
        }

        assert_eq!(
            dragon_2962,
            Some((0x01C2_8000, 0x01B0_0000, Direction::Right, true)),
            "the surviving lower dragon should still be parked on the oracle-backed stomp tile through frame 2962 instead of resuming patrol immediately",
        );
        assert_eq!(
            dragon_2963,
            Some((0x01C3_0000, 0x01B0_0000, Direction::Right, true)),
            "once the shared hurt hold clears, the lower dragon should resume the oracle-backed right-facing patrol step on frame 2963",
        );
    }

    #[test]
    fn demo_1_reenters_walking_from_standingstill_on_frame_3252() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frames = BTreeMap::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_259) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (3_251..=3_258).contains(&frame_index) {
                frames.insert(frame_index, trace);
            }
        }

        let frame_3251 = frames
            .get(&3_251)
            .unwrap_or_else(|| panic!("frame 3251 should be captured"));
        assert_eq!(
            frame_3251.kid_x_pos,
            Some(0x027D_0000),
            "demo 1 should still be standing still on frame 3251",
        );
        assert_eq!(frame_3251.kid_x_velocity, Some(0));

        let frame_3252 = frames
            .get(&3_252)
            .unwrap_or_else(|| panic!("frame 3252 should be captured"));
        assert_eq!(
            frame_3252.kid_x_pos,
            Some(0x027D_1000),
            "demo 1 should restart the oracle-backed walk acceleration on frame 3252",
        );
        assert_eq!(
            frame_3252.kid_x_velocity,
            Some(0x0000_1000),
            "frame 3252 should take the first +0x1000 walking step",
        );
        assert_eq!(frame_3252.kid_on_ground, Some(true));

        let frame_3258 = frames
            .get(&3_258)
            .unwrap_or_else(|| panic!("frame 3258 should be captured"));
        assert_eq!(
            frame_3258.kid_x_pos,
            Some(0x027E_C000),
            "the resumed walk should stay aligned with the oracle through frame 3258",
        );
        assert_eq!(frame_3258.kid_x_velocity, Some(0x0000_7000));
    }

    #[test]
    fn demo_1_loads_the_second_knights_isle_dragon_by_frame_3400() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragon_layout_indices = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_401) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_400 {
                let mut layout_indices: Vec<_> = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| enemy.layout_index)
                    .collect();
                layout_indices.sort_unstable();
                dragon_layout_indices = Some(layout_indices);
                break;
            }
        }

        let dragon_layout_indices =
            dragon_layout_indices.unwrap_or_else(|| panic!("frame 3400 should be captured"));
        assert_eq!(
            dragon_layout_indices,
            vec![0, 1, 2, 4],
            "frame 3400 should keep all four Knights Isle walking dragons active inside the ASM-backed loader window",
        );
    }

    #[test]
    fn demo_1_reaches_the_last_oracle_backed_pre_stomp_arc_on_frame_3399() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_3399 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_400) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_399 {
                frame_3399 = Some(trace);
                break;
            }
        }

        let frame_3399 = frame_3399.unwrap_or_else(|| panic!("frame 3399 should be captured"));
        assert_eq!(
            frame_3399.kid_x_pos,
            Some(0x037E_F100),
            "demo 1 should keep the oracle-backed pre-stomp X position through frame 3399",
        );
        assert_eq!(
            frame_3399.kid_y_pos,
            Some(0x01A1_C000),
            "demo 1 should keep the oracle-backed pre-stomp Y position through frame 3399",
        );
    }

    #[test]
    fn demo_1_walking_dragons_keep_grounded_zero_velocity_on_frame_3399() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragons_3399 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_400) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_399 {
                let mut dragons: Vec<_> = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| {
                        (
                            enemy.layout_index,
                            enemy.x_velocity,
                            enemy.y_velocity,
                            enemy.on_ground,
                        )
                    })
                    .collect();
                dragons.sort_unstable_by_key(|dragon| dragon.0);
                dragons_3399 = Some(dragons);
                break;
            }
        }

        let dragons_3399 = dragons_3399.unwrap_or_else(|| panic!("frame 3399 should be captured"));
        assert_eq!(
            dragons_3399,
            vec![
                (0, 0, 0, true),
                (1, 0, 0, true),
                (2, 0, 0, true),
                (4, 0, 0, true),
            ],
            "frame 3399 should keep the grounded walking dragons on the oracle-backed zero-velocity path even though the position/facing frontier remains open",
        );
    }

    #[test]
    fn demo_1_frame_3400_reports_kid_object_support_candidates() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_3400 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_401) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_400 {
                frame_3400 = Some(trace);
                break;
            }
        }

        let frame_3400 = frame_3400.unwrap_or_else(|| panic!("frame 3400 should be captured"));
        let support_debug = frame_3400
            .gameplay_debug_fields
            .get("kid_object_support_candidates")
            .unwrap_or_else(|| panic!("frame 3400 should emit Kid/object support diagnostics"));

        assert!(
            support_debug.contains("enemy_0D#00"),
            "the frame-3400 diagnostics should include the nearby right-side walking dragon candidate",
        );
        assert!(
            support_debug.contains("support="),
            "the frame-3400 diagnostics should report support classification details for the nearby candidates",
        );
    }

    #[test]
    fn published_tas_dragon_breath_children_follow_rom_object_gfx_lifecycle_loc_3d02a() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let expected = [
            (7_937usize, (9u16, 8u16)),
            (7_938, (10, 9)),
            (7_955, (11, 10)),
            (8_005, (11, 10)),
            (8_022, (10, 9)),
            (8_023, (9, 8)),
        ];
        let mut captured = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_024) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if expected
                .iter()
                .any(|(expected_frame, _)| *expected_frame == frame_index)
            {
                captured.push((
                    frame_index,
                    (trace.number_objects, trace.number_gfx_objects),
                ));
            }
        }

        assert_eq!(
            captured, expected,
            "ASM Dragon.asm loc_3CF90 allocates each breath child, loc_3D02A loads its linked GfxObject, loc_3D082 keeps both live, and loc_3D09E deletes the lifecycle",
        );
    }

    #[test]
    fn published_tas_transform_cleanup_resumes_dragons_loc_7e90_e12() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut captured = Vec::new();
        let mut wall_boundary = Vec::new();
        let mut frame_7797_counts = None;
        let mut breath_contact_resume = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_860) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(frame_index, 7_685 | 7_686) {
                let mut dragons: Vec<_> = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| (enemy.layout_index, enemy.x_pos))
                    .collect();
                dragons.sort_unstable_by_key(|dragon| dragon.0);
                captured.push((frame_index, dragons));
            }
            if matches!(frame_index, 7_746..=7_748) {
                let dragon = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .unwrap_or_else(|| {
                        panic!("walking Dragon #02 should be live on frame {frame_index}")
                    });
                wall_boundary.push((frame_index, dragon.x_pos, dragon.facing));
            }
            if frame_index == 7_797 {
                frame_7797_counts = Some((trace.number_objects, trace.number_gfx_objects));
            }
            if matches!(frame_index, 7_858 | 7_859) {
                let dragon = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 0
                    })
                    .unwrap_or_else(|| {
                        panic!("walking Dragon #00 should be live on frame {frame_index}")
                    });
                breath_contact_resume.push((frame_index, dragon.x_pos, dragon.facing));
            }
        }

        assert_eq!(
            captured,
            vec![
                (
                    7_685,
                    vec![
                        (0, 0x032F_0000),
                        (1, 0x03A8_8000),
                        (2, 0x01F8_8000),
                        (4, 0x02E5_8000),
                    ],
                ),
                (
                    7_686,
                    vec![
                        (0, 0x032F_8000),
                        (1, 0x03A8_0000),
                        (2, 0x01F9_0000),
                        (4, 0x02E6_0000),
                    ],
                ),
            ],
            "ASM Kid_Transform loc_7E90 clears Currently_transforming before the outer Execute_Objects loc_E12 pass resumes every walking Dragon on frame 7686",
        );
        assert_eq!(
            wall_boundary,
            vec![
                (7_746, 0x0217_0000, Direction::Right),
                (7_747, 0x0217_8000, Direction::Right),
                (7_748, 0x0217_8000, Direction::Left),
            ],
            "ASM Lion.asm loc_3E3B2/loc_3E406 should preserve the resumed Dragon's fractional wall-boundary phase",
        );
        assert_eq!(
            frame_7797_counts,
            Some((9, 8)),
            "ASM Dragon.asm loc_3CF90/loc_3D02A breath allocation must retain its oracle object-count phase after transform cleanup",
        );
        assert_eq!(
            breath_contact_resume,
            vec![
                (7_858, 0x0367_0000, Direction::Right),
                (7_859, 0x0367_8000, Direction::Right),
            ],
            "ASM Dragon.asm loc_3CFDA -> loc_3CEBE -> loc_3CF4C/loc_3CF62 resumes walking after a nonlethal Kid-above hit ends the breath child",
        );
    }

    #[test]
    fn published_tas_dragon_breath_counts_follow_raw_lists_loc_3cfda_3d09e() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let expected = [
            (7_817usize, (11u16, 10u16)),
            (7_834, (10, 9)),
            (7_835, (10, 9)),
            (7_850usize, (10u16, 9u16)),
            (7_954, (10, 9)),
            (8_022, (10, 9)),
            (8_023, (9, 8)),
            (8_024, (9, 8)),
        ];
        let mut captured = Vec::new();
        let mut final_execute_dragon = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_025) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if expected
                .iter()
                .any(|(expected_frame, _)| *expected_frame == frame_index)
            {
                captured.push((
                    frame_index,
                    (trace.number_objects, trace.number_gfx_objects),
                ));
            }
            if matches!(frame_index, 7_953..=7_955) {
                let dragon = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .unwrap_or_else(|| {
                        panic!("walking Dragon #02 should be live on frame {frame_index}")
                    });
                final_execute_dragon.push((
                    frame_index,
                    dragon.x_pos,
                    dragon.wait_timer,
                    dragon.dragon_script_cmd,
                    dragon.dragon_script_variant & DRAGON_WALKING_TRANSFORM_TIMER_ALIGNED_FLAG != 0,
                ));
            }
        }

        assert_eq!(
            captured, expected,
            "ASM Dragon.asm loc_3CF90 allocates each breath pair, loc_3CFDA ends a contacted parent's child, and loc_3D09E removes completed children from both linked-list counts",
        );
        assert_eq!(
            final_execute_dragon,
            vec![
                (7_953, 0x01E3_0000, 2, DRAGON_WALKING_PHASE_WALK, true),
                (7_954, 0x01E2_8000, 1, DRAGON_WALKING_PHASE_WALK, true),
                (7_955, 0x01E2_8000, 500, DRAGON_WALKING_PHASE_BREATH, true,),
            ],
            "ASM Kid_Transform loc_7E90 resumes the suspended Execute_Objects loc_E1E pass one callback behind native's atomic transform model, preserving the final Dragon's `$66=1` walk before loc_3CE90/loc_3CF6E",
        );
    }

    #[test]
    fn published_tas_trailing_loader_initialises_dragon_on_next_object_pass_loc_3691e_3cd46() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut captured = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_841) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(frame_index, 21_837..=21_840) {
                captured.push((
                    frame_index,
                    trace
                        .active_objects
                        .iter()
                        .find(|object| {
                            object.kind == NativeObjectKind::Enemy { object_type: 0x0D }
                                && object.layout_index == Some(2)
                        })
                        .map(|object| (object.x_pos, object.y_pos)),
                ));
            }
        }

        assert_eq!(
            captured,
            vec![
                (21_837, None),
                (21_838, None),
                (21_839, Some((0x0150_0000, 0x0180_0000))),
                (21_840, Some((0x014F_8000, 0x0181_0000))),
            ],
            "ASM Mode_Level loc_80C runs Manage_EnemyLoading loc_3691E after Execute_Objects: the new Dragon Object waits at Enemy0D_Dragon_Init loc_3CD46 until the next pass, which links its spawn-position GfxObject before AI can move it",
        );
    }

    #[test]
    fn published_tas_dragon_breath_completion_edges_match_loc_3d09e() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let expected = [(7_885usize, (9u16, 8u16)), (8_006, (10, 9))];
        let mut captured = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_007) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if expected
                .iter()
                .any(|(expected_frame, _)| *expected_frame == frame_index)
            {
                captured.push((
                    frame_index,
                    (trace.number_objects, trace.number_gfx_objects),
                ));
            }
        }

        assert_eq!(
            captured, expected,
            "ASM Dragon.asm loc_3D082 -> loc_3D09E must delete each completed breath pair on the oracle object pass",
        );
    }

    #[test]
    fn published_tas_frame_7969_reports_kid_ground_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));
        let ground_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_ground_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid-ground probe diagnostics"));

        assert!(
            ground_debug.contains("latched_ground=0"),
            "the frame-7969 diagnostics should show the grounded latch clears after the frame-7968 attach hibernate",
        );
        assert!(
            ground_debug.contains("y_vel=0xffffc080"),
            "the frame-7969 diagnostics should show the resume-frame ROM wall lookup now preserves upward climb velocity",
        );
        assert!(
            ground_debug.contains("hitbox=ground=0"),
            "the frame-7969 diagnostics should show the grounded probes no longer relatch despite the stale support cell under Kid",
        );
    }

    #[test]
    fn published_tas_frame_7968_reports_kid_special_motion_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_969) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let motion_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_special_motion_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid special-motion diagnostics"));

        assert!(
            motion_debug.contains(&format!("helmet={:#06x}", frame_7968.current_helmet_id)),
            "the frame-7968 diagnostics should echo the live helmet id",
        );
        assert!(
            motion_debug.contains("pending_stomp_bounce="),
            "the frame-7968 diagnostics should include the pending stomp-bounce latch state",
        );
    }

    #[test]
    fn published_tas_frame_7969_reports_kid_goal_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));
        let goal_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_goal_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid-goal probe diagnostics"));

        assert!(
            goal_debug.contains("lifecycle=alive"),
            "the frame-7969 diagnostics should show Kid is still in the normal gameplay lifecycle",
        );
        assert!(
            goal_debug.contains("touch=0"),
            "the frame-7969 diagnostics should prove native is not already touching the goal flag",
        );
        assert!(
            !frame_7969.level_complete,
            "the frame-7969 trace should not already be in LevelComplete",
        );
    }

    #[test]
    fn published_tas_frame_7968_reports_kid_gameplay_path_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_969) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let path_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid gameplay-path diagnostics"));

        assert!(
            path_debug.contains("lifecycle=alive"),
            "the frame-7968 diagnostics should show Kid is still on the live gameplay lifecycle path",
        );
        assert!(
            path_debug.contains("motion=iron_knight_attach_hibernate"),
            "the frame-7968 diagnostics should prove native now mirrors loc_A276's one-frame attach hibernate",
        );
        assert!(
            path_debug.contains("post=none"),
            "the frame-7968 diagnostics should prove no post-physics handoff moved Kid on the native side",
        );
        assert_eq!(
            frame_7968.kid_x_pos,
            Some(0x03A8_0000),
            "frame 7968 should preserve Kid's x position while the grounded Iron Knight attach hibernates",
        );
        assert_eq!(
            frame_7968.kid_y_pos,
            Some(0x01BF_0000),
            "frame 7968 should preserve Kid's y position while the grounded Iron Knight attach hibernates",
        );
        assert_eq!(
            frame_7968.kid_y_velocity,
            Some(0xFFFF_C000u32 as i32),
            "frame 7968 should still expose loc_A276's seeded upward climb velocity in object state",
        );
    }

    #[test]
    fn published_tas_demo_bootstrap_freefall_exports_shadow_y_velocity() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut sampled = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(907) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (904..=906).contains(&frame_index) {
                sampled.push((frame_index, trace));
            }
        }

        assert_eq!(sampled.len(), 3);
        for (frame_index, trace) in sampled {
            let expected = ((frame_index - 903) as i32) * 0x4000;
            let path_debug = trace
                .gameplay_debug_fields
                .get("kid_gameplay_path_probe")
                .unwrap_or_else(|| panic!("frame {frame_index} should emit path diagnostics"));
            assert!(
                path_debug.contains("motion=demo_bootstrap_freefall"),
                "frame {frame_index} should still be in the ASM-backed demo bootstrap freefall path: {path_debug}",
            );
            assert_eq!(
                trace.kid_y_velocity,
                Some(expected),
                "ASM/oracle any-percent-published frame {frame_index} exports the bootstrap shadow velocity, not zero",
            );
        }
    }

    #[test]
    fn published_tas_wall_floor_boundary_exports_sampled_y_velocity() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_957 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(958) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 957 {
                frame_957 = Some(trace);
                break;
            }
        }

        let trace = frame_957.unwrap_or_else(|| panic!("frame 957 should be captured"));
        let probe = trace
            .gameplay_debug_fields
            .get("kid_flip_follow_on_probe")
            .unwrap_or_else(|| panic!("frame 957 should emit Kid follow-on diagnostics"));
        assert!(
            probe.contains(
                "after_floor[x_vel=0x00000000/y_vel=0xffff59e0/ground=0/wall_cling=right]"
            ),
            "ASM loc_AAC8/sub_902A should keep the post-gravity sample after the vertical floor probe: {probe}",
        );
        assert!(
            probe
                .contains("after_wall[x_vel=0x00000000/y_vel=0x00000000/ground=1/wall_cling=none]"),
            "ASM loc_AD7C wall/floor boundary settlement should clear simulation y velocity: {probe}",
        );
        assert_eq!(
            trace.kid_y_velocity,
            Some(0xFFFF_59E0u32 as i32),
            "oracle any-percent-published frame 957 exports the sampled post-gravity y velocity through the stale GFX slot",
        );
    }

    #[test]
    fn published_tas_frame_7969_matches_oracle_iron_knight_climb_resume_pose() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));
        let path_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid gameplay-path diagnostics"));

        assert!(
            path_debug.contains("motion=iron_knight_climb_resume"),
            "frame 7969 should stay on the dedicated Iron Knight climb resume path",
        );
        assert_eq!(
            frame_7969.kid_x_pos,
            Some(0x03A8_0000),
            "frame 7969 should keep Kid pinned to the wall column during the resume step",
        );
        assert_eq!(
            frame_7969.kid_y_pos,
            Some(0x01BE_C080),
            "frame 7969 should climb to the oracle-backed resume position",
        );
    }

    #[test]
    fn published_tas_frame_8000_stays_on_live_climb_gameplay() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_8000 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_001) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_000 {
                frame_8000 = Some(trace);
                break;
            }
        }

        let frame_8000 = frame_8000.unwrap_or_else(|| panic!("frame 8000 should be captured"));
        let path_debug = frame_8000
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8000 should emit kid gameplay-path diagnostics"));

        assert_eq!(
            frame_8000.game_mode,
            GameMode::DemoPlay,
            "frame 8000 should still trace as live demo gameplay before the timeout handoff becomes visible",
        );
        assert!(
            !path_debug.contains("motion=demo_auto_timeout_frozen"),
            "frame 8000 should not freeze the timeout snapshot one gameplay frame early",
        );
        assert_eq!(
            frame_8000.kid_y_pos,
            Some(0x01BA_6D00),
            "frame 8000 should keep climbing to the oracle-backed live gameplay pose",
        );
    }

    #[test]
    fn published_tas_frame_8001_delays_demo_1_timeout_handoff_by_one_more_climb_step() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_8001 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_002) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_001 {
                frame_8001 = Some(trace);
                break;
            }
        }

        let frame_8001 = frame_8001.unwrap_or_else(|| panic!("frame 8001 should be captured"));
        let path_debug = frame_8001
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8001 should emit kid gameplay-path diagnostics"));

        assert_eq!(
            frame_8001.game_mode,
            GameMode::DemoPlay,
            "frame 8001 should still be live demo gameplay before the timeout handoff becomes visible",
        );
        assert!(
            path_debug.contains("motion=iron_knight_climb_resume"),
            "frame 8001 should stay on the climb-resume motion instead of dropping into the timeout handoff",
        );
        assert_eq!(
            frame_8001.kid_y_pos,
            Some(0x01BA_1780),
            "frame 8001 should take one more oracle-backed climb step before the title return",
        );
    }

    #[test]
    fn published_tas_frame_8002_keeps_demo_1_in_live_climb_gameplay() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_8002 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_003) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_002 {
                frame_8002 = Some(trace);
                break;
            }
        }

        let frame_8002 = frame_8002.unwrap_or_else(|| panic!("frame 8002 should be captured"));
        let path_debug = frame_8002
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8002 should emit kid gameplay-path diagnostics"));

        assert_eq!(
            frame_8002.game_mode,
            GameMode::DemoPlay,
            "frame 8002 should still be live demo gameplay before the mode-0 handoff becomes visible",
        );
        assert_eq!(
            frame_8002.current_level,
            LevelId(0x15),
            "frame 8002 should stay in Knights Isle until the ROM timeout gate actually trips",
        );
        assert!(
            path_debug.contains("motion=iron_knight_climb_resume"),
            "frame 8002 should stay on the climb-resume motion instead of flipping into the title return",
        );
        assert_eq!(
            frame_8002.kid_y_pos,
            Some(0x01B9_BD00),
            "frame 8002 should keep the oracle-backed live climb pose instead of freezing the prior step",
        );
    }

    #[test]
    fn published_tas_frame_8003_keeps_demo_1_in_live_climb_gameplay() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_8003 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_004) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_003 {
                frame_8003 = Some(trace);
                break;
            }
        }

        let frame_8003 = frame_8003.unwrap_or_else(|| panic!("frame 8003 should be captured"));
        let path_debug = frame_8003
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8003 should emit kid gameplay-path diagnostics"));

        assert_eq!(
            frame_8003.game_mode,
            GameMode::DemoPlay,
            "frame 8003 should still trace as live demo gameplay before the mode-0 handoff becomes visible",
        );
        assert_eq!(
            frame_8003.current_level,
            LevelId(0x15),
            "frame 8003 should stay in Knights Isle until the stored timeout counter is compared on the next ReadJoypad pass",
        );
        assert!(
            path_debug.contains("motion=iron_knight_climb_resume"),
            "frame 8003 should keep climbing instead of entering the timeout title return one frame early",
        );
        assert_eq!(
            frame_8003.kid_y_pos,
            Some(0x01B9_5E00),
            "frame 8003 should keep the oracle-backed live climb pose while the ROM still compares the prior timeout counter",
        );
    }

    #[test]
    fn published_tas_frame_8004_keeps_demo_1_in_live_climb_gameplay() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_8004 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_005) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_004 {
                frame_8004 = Some(trace);
                break;
            }
        }

        let frame_8004 = frame_8004.unwrap_or_else(|| panic!("frame 8004 should be captured"));

        assert_eq!(
            frame_8004.game_mode,
            GameMode::DemoPlay,
            "frame 8004 should still trace as live demo gameplay before the ROM leaves demo mode",
        );
        assert_eq!(
            frame_8004.current_level,
            LevelId(0x15),
            "frame 8004 should stay in Knights Isle while the demo timeout counter is still below $3F2 in oracle RAM",
        );
        assert_eq!(
            frame_8004.kid_y_pos,
            Some(0x01B8_FB00),
            "frame 8004 should keep the oracle-backed live climb pose instead of freezing the prior step",
        );
    }

    #[test]
    fn published_tas_demo_1_allocates_kid_then_dragon_gfx_objects_on_frames_2607_and_2608() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut counts = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(2_609) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (2_605..=2_608).contains(&frame_index) {
                counts.push((
                    frame_index,
                    trace.boot_phase_elapsed,
                    trace.number_objects,
                    trace.number_gfx_objects,
                ));
            }
        }

        // ASM Load_DemoPlay -> Load_InGame exposes its second non-gfx worker
        // on frame 2605. Frame 2606 has two system GfxObjects; frame 2607 adds
        // Kid's two slots. Manage_EnemyLoading loc_3691E allocates the Knights
        // Isle Dragon Object there, then Enemy0D_Dragon_Init calls
        // Load_GfxObjectSlot on frame 2608. Raw oracle WRAM reads 2/0, 6/2,
        // 7/4, and 7/5 across these four frames.
        assert_eq!(
            counts,
            vec![
                (2_605, Some(70), 2, 0),
                (2_606, Some(71), 6, 2),
                (2_607, Some(72), 7, 4),
                (2_608, Some(73), 7, 5),
            ]
        );
    }

    #[test]
    fn published_tas_demo_1_third_dragon_gfx_object_survives_the_title_snapshot() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut counts = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_155) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if (3_063..=3_065).contains(&frame_index)
                || matches!(frame_index, 3_121 | 3_122 | 3_153 | 3_154)
            {
                counts.push((
                    frame_index,
                    trace.boot_phase_elapsed,
                    trace.number_objects,
                    trace.number_gfx_objects,
                ));
            }
        }

        // ASM Manage_EnemyLoading loc_3691E links the third Knights Isle
        // Dragon Object on frame 3064. Enemy0D_Dragon_Init calls
        // Load_GfxObjectSlot on the next Execute_Objects pass, so raw oracle
        // WRAM advances 7/6 -> 8/6 -> 8/7 across these frames.
        assert_eq!(
            counts,
            vec![
                (3_063, Some(528), 7, 6),
                (3_064, Some(529), 8, 6),
                (3_065, Some(530), 8, 7),
                (3_121, Some(586), 8, 7),
                (3_122, Some(0), 8, 7),
                (3_153, Some(31), 8, 7),
                (3_154, Some(32), 0, 0),
            ]
        );
    }

    #[test]
    fn published_tas_intro_sequence_2_carries_intro_1_gfx_until_rebuild() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut counts = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(5_547) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(frame_index, 5_525 | 5_534 | 5_535 | 5_545 | 5_546) {
                counts.push((
                    frame_index,
                    trace.boot_phase_elapsed,
                    trace.number_objects,
                    trace.number_gfx_objects,
                ));
            }
        }

        // ASM sub_1B8F4 -> loc_1B920 leaves IntroSequence1's sub_1C54A
        // GfxObject linked while Game_Mode already exposes IntroSequence2.
        // Raw oracle WRAM keeps that 1/1 roster through frame 5534, clears it
        // in loc_6E2's rebuild on frame 5535, then Load_IntroSequence2 ->
        // loc_1B9D0 exposes its controller and loc_1C7A0 text worker without
        // allocating another GfxObject.
        assert_eq!(
            counts,
            vec![
                (5_525, Some(21), 1, 1),
                (5_534, Some(30), 1, 1),
                (5_535, Some(31), 0, 0),
                (5_545, Some(41), 1, 0),
                (5_546, Some(42), 2, 0),
            ]
        );
    }

    #[test]
    fn published_tas_timeout_restart_intro_3_advances_title_text_kid_gfx_passes() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let expected = [
            (6_264, 3, 0),
            (6_265, 5, 1),
            (6_266, 5, 2),
            (6_268, 4, 2),
            (6_269, 4, 2),
            (6_357, 4, 2),
            (6_358, 4, 3),
            (6_359, 4, 3),
            (6_367, 4, 3),
            (6_368, 4, 4),
            (6_369, 4, 4),
        ];
        let mut actual = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(6_370) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if expected
                .iter()
                .any(|(expected_frame, _, _)| *expected_frame == frame_index)
            {
                actual.push((frame_index, trace.number_objects, trace.number_gfx_objects));
            }
        }

        // ASM Load_TitleScreen -> Obj_TitleTextKid loc_1B5E4 -> sub_1B652:
        // after the timeout-restart IntroSequence3 return rejoins loc_E12,
        // each initial and popup GfxObject allocation is visible on that pass.
        assert_eq!(actual, expected);
    }

    #[test]
    fn published_tas_demo_1_keeps_walking_dragon_gfx_across_layout_segment_boundary() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let expected = [(7_485, 8, 7), (7_486, 8, 7), (7_492, 8, 7), (7_493, 8, 7)];
        let mut actual = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_494) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if expected
                .iter()
                .any(|(expected_frame, _, _)| *expected_frame == frame_index)
            {
                actual.push((frame_index, trace.number_objects, trace.number_gfx_objects));
            }
        }

        // ASM Dragon.asm shared walker -> sub_3E8BA culls each already-linked
        // object from its live position; loc_368A8 changing layout segments
        // does not deallocate or reinitialise the three existing GfxObjects.
        assert_eq!(actual, expected);
    }

    #[test]
    fn published_tas_timeout_restart_intro_5_advances_title_text_kid_popup_passes() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let expected = [
            (10_437, 4, 2),
            (10_438, 4, 3),
            (10_447, 4, 3),
            (10_448, 4, 4),
            (16_798, 4, 2),
            (16_799, 4, 3),
            (16_808, 4, 3),
            (16_809, 4, 4),
        ];
        let mut actual = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(16_810) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if expected
                .iter()
                .any(|(expected_frame, _, _)| *expected_frame == frame_index)
            {
                actual.push((frame_index, trace.number_objects, trace.number_gfx_objects));
            }
        }

        // ASM Obj_TitleTextKid loc_1B5E4 -> sub_1B652 links each popup on
        // the Execute_Objects pass that resumes the hibernating title worker.
        // Raw oracle WRAM pins the plain timeout return two passes ahead and
        // the delayed Start-skip return one pass ahead of the generic shell.
        assert_eq!(actual, expected);
    }

    #[test]
    fn published_tas_later_demo_1_title_popup_passes_remain_sequential() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let expected = [
            (12_579, 4, 2),
            (12_580, 4, 3),
            (12_589, 4, 3),
            (12_590, 4, 4),
        ];
        let mut actual = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(12_591) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if expected
                .iter()
                .any(|(expected_frame, _, _)| *expected_frame == frame_index)
            {
                actual.push((frame_index, trace.number_objects, trace.number_gfx_objects));
            }
        }

        // ASM Obj_TitleTextKid loc_1B5E4 -> sub_1B652 allocates one popup,
        // hibernates for `$A`, then allocates the second. Raw oracle WRAM
        // therefore exposes a real ten-frame 4/3 interval on this return.
        assert_eq!(actual, expected);
    }

    #[test]
    fn published_tas_demo_1_times_out_on_frame_8069() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_8068 = None;
        let mut frame_8069 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_070) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                8_068 => frame_8068 = Some((trace.game_mode, trace.current_level)),
                8_069 => {
                    frame_8069 = Some((trace.game_mode, trace.current_level));
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_8068,
            Some((GameMode::DemoPlay, LevelId(0x15))),
            "frame 8068 should still be live demo gameplay while the oracle timeout counter first reaches $3F2",
        );
        assert_eq!(
            frame_8069,
            Some((GameMode::SegaScreen, LevelId(0x00))),
            "frame 8069 should expose the ROM's mode-0 timeout handoff on the first post-threshold frame",
        );
    }

    #[test]
    fn published_tas_demo_1_preserves_the_oracle_timeout_snapshot_on_frames_8069_and_8070() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_8069 = None;
        let mut frame_8070 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_071) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                8_069 => {
                    frame_8069 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.camera_x_pos,
                        trace.camera_y_pos,
                        trace.number_objects,
                        trace.number_gfx_objects,
                        trace.character_movement,
                    ));
                }
                8_070 => {
                    frame_8070 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.camera_x_pos,
                        trace.camera_y_pos,
                        trace.number_objects,
                        trace.number_gfx_objects,
                        trace.character_movement,
                    ));
                    break;
                }
                _ => {}
            }
        }

        let expected = Some((
            GameMode::SegaScreen,
            LevelId(0x00),
            Some(0x0398_2C80),
            Some(0x0188_3680),
            0x02FF_0000,
            0x00E7_0000,
            9,
            8,
            Some(3),
        ));
        assert_eq!(
            frame_8069, expected,
            "frame 8069 should keep the oracle-backed Kid/camera/object snapshot while the timeout handoff first exposes mode 0",
        );
        assert_eq!(
            frame_8070, expected,
            "frame 8070 should keep the same preserved timeout snapshot until the title rebuild starts clearing it",
        );
    }

    #[test]
    fn published_tas_frame_8056_hibernates_the_iron_knight_wall_release() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_8056 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_057) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_056 {
                frame_8056 = Some(trace);
                break;
            }
        }

        let frame_8056 = frame_8056.unwrap_or_else(|| panic!("frame 8056 should be captured"));
        let path_debug = frame_8056
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8056 should emit kid gameplay-path diagnostics"));

        assert!(
            path_debug.contains("motion=iron_knight_climb_exit_hibernate"),
            "frame 8056 should leave the climb-resume loop and hibernate the wall-release frame",
        );
        assert_eq!(
            frame_8056.kid_y_pos,
            Some(0x0186_D680),
            "frame 8056 should preserve the oracle-backed wall-cling pose while the release frame hibernates",
        );
    }

    #[test]
    fn published_tas_frame_8057_matches_the_oracle_wall_release_carry() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_8057 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_058) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 8_057 {
                frame_8057 = Some(trace);
                break;
            }
        }

        let frame_8057 = frame_8057.unwrap_or_else(|| panic!("frame 8057 should be captured"));
        let path_debug = frame_8057
            .gameplay_debug_fields
            .get("kid_gameplay_path_probe")
            .unwrap_or_else(|| panic!("frame 8057 should emit kid gameplay-path diagnostics"));

        assert!(
            !path_debug.contains("motion=iron_knight_climb_resume"),
            "frame 8057 should stay off the dedicated climb-resume motion after the wall release",
        );
        assert_eq!(
            frame_8057.kid_x_pos,
            Some(0x03A7_D500),
            "frame 8057 should pick up the oracle-backed leftward carry on the first post-release frame",
        );
        assert_eq!(
            frame_8057.kid_y_pos,
            Some(0x0185_9680),
            "frame 8057 should match the oracle-backed post-release jump pose",
        );
    }

    #[test]
    fn published_tas_frames_8056_to_8058_match_the_oracle_wall_release_camera_carry() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_8056 = None;
        let mut frame_8057 = None;
        let mut frame_8058 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(8_059) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                8_056 => frame_8056 = Some(trace.camera_y_pos),
                8_057 => frame_8057 = Some(trace.camera_y_pos),
                8_058 => {
                    frame_8058 = Some(trace.camera_y_pos);
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_8056,
            Some(0x00F0_0000),
            "frame 8056 should still advance the release-frame camera follow before the Iron Knight wall-release hibernates",
        );
        assert_eq!(
            frame_8057,
            Some(0x00EF_0000),
            "frame 8057 should continue from the oracle-backed release-frame camera state",
        );
        assert_eq!(
            frame_8058,
            Some(0x00EE_0000),
            "frame 8058 should keep the one-pixel-per-frame camera falloff after the wall release",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_pre_physics_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_pre_physics_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid pre-physics diagnostics"));
        assert!(
            frame_7968_debug.contains("first=iron_knight_climb"),
            "frame 7968 should record Iron Knight climb as the first pre-physics mutator on the attach frame",
        );
        assert!(
            frame_7968_debug.contains("mutators=iron_knight_climb"),
            "frame 7968 should keep the attach visible in the mutator list",
        );
        assert!(
            frame_7968_debug.contains(
                "before_physics[x=0x03a80000/y=0x01bf0000/x_vel=0x00000000/y_vel=0xffffc000/ground=0]"
            ),
            "frame 7968 should expose the seeded climb velocity and airborne state before the hibernated frame ends",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_pre_physics_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid pre-physics diagnostics"));
        assert!(
            frame_7969_debug.contains("first=iron_knight_climb"),
            "frame 7969 should now resume through the Iron Knight climb mutator before normal physics",
        );
        assert!(
            frame_7969_debug.contains("mutators=iron_knight_climb"),
            "frame 7969 should keep the climb mutator visible while the resume-frame bug remains open",
        );
        assert!(
            frame_7969_debug.contains(
                "before_physics[x=0x03a80000/y=0x01bf0000/x_vel=0x00000000/y_vel=0xffffc080/ground=0]"
            ),
            "frame 7969 should apply the ROM climb drift before the dedicated resume step runs",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_grounded_release_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        assert!(
            !frame_7968
                .gameplay_debug_fields
                .contains_key("kid_grounded_release_probe"),
            "frame 7968 should skip the grounded-release probe because loc_A276 hibernates before native normal physics",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_grounded_release_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid grounded-release diagnostics"));
        assert!(
            frame_7969_debug.contains(
                "ground=0/held=right/jump=0/jump_buffer=0/velocity=airborne_custom_gravity/custom_gravity=1/wall_cling=none"
            ),
            "frame 7969 should show the resume frame stays on Iron Knight custom gravity even though the later motion path still diverges",
        );
        assert!(
            frame_7969_debug.contains("scan=row=none/cols=none/support=none"),
            "frame 7969 should prove the old grounded support scan no longer runs once the attach frame hibernates",
        );
        assert!(
            frame_7969_debug.contains("release=0/post=none"),
            "frame 7969 should show no later native release handoff recovers the lost climb state",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_collision_handoff_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_collision_handoff_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid collision-handoff diagnostics"));
        assert!(
            frame_7968_debug.contains(
                "pending_stomp=0/pending_form=none/pending_level_skip=none/form_flash=0/ground=1/jump_buffer=0/wall_cling=none"
            ),
            "frame 7968 should still enter the attach frame with no pre-existing collision carry state",
        );
        assert!(
            frame_7968_debug.contains("anchor=0x03a80000,0x01bf0000"),
            "frame 7968 should preserve the saved anchor before the attach hibernate",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_collision_handoff_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid collision-handoff diagnostics"));
        assert!(
            frame_7969_debug.contains(
                "pending_stomp=0/pending_form=none/pending_level_skip=none/form_flash=0/ground=0/jump_buffer=0/wall_cling=none"
            ),
            "frame 7969 should show the next frame starts airborne from the hibernated attach state",
        );
        assert!(
            frame_7969_debug
                .contains("stomp_candidate=0/stomp_enemy=none/anchor=0x03a80000,0x01bf0000"),
            "frame 7969 should keep the anchor stable even though the climb follow-on is still wrong",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_same_frame_stomp_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_same_frame_stomp_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit same-frame stomp diagnostics"));
        assert!(
            frame_7968_debug.contains("pre_y_vel=0xffffc000/post_y_vel=0xffffc000"),
            "frame 7968 should record the hibernated attach frame's preserved seeded climb velocity",
        );
        assert!(
            frame_7968_debug.contains("candidate=0/enemy=none"),
            "frame 7968 should keep the stomp path cold while preserving the attach anchor",
        );
        assert!(
            frame_7968_debug.contains("anchor=0x03a80000,0x01bf0000"),
            "frame 7968 should keep the stomp path cold while preserving the attach anchor",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_same_frame_stomp_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit same-frame stomp diagnostics"));
        assert!(
            frame_7969_debug.contains("pre_y_vel=0xffffc080/post_y_vel=0xffffc080"),
            "frame 7969 should show the resume frame now preserves the climb velocity through physics",
        );
        assert!(
            frame_7969_debug.contains("candidate=0/enemy=none"),
            "frame 7969 should keep the stomp path cold while the new frontier stays in player motion",
        );
        assert!(
            frame_7969_debug.contains("anchor=0x03a80000,0x01bf0000"),
            "frame 7969 should keep the stomp path cold while the new frontier stays in player motion",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_iron_knight_attach_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_969) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let probe_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_iron_knight_attach_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit Iron Knight attach diagnostics"));

        assert!(
            probe_debug.contains("c_press=1/ground=1/climbing=0=>1/facing=right"),
            "frame 7968 should consume the raw C edge and enter Iron Knight climb from the grounded walking path",
        );
        assert!(
            probe_debug.contains("y_vel=0x00000000->0xffffc000"),
            "frame 7968 should mirror loc_A276's -0x4000 wall-climb lift-off seed",
        );
        assert!(
            probe_debug
                .contains("wall=hit=1/x=944/cell=59,27 collision=solid block=None has_block=0"),
            "frame 7968 should preserve the ROM edge-column wall hit at the attach boundary",
        );
        assert!(
            probe_debug.contains("candidate=1/attach=1"),
            "frame 7968 should prove the ported sub_922C gate now fires the live grounded attach window",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_flip_branch_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_flip_branch_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid_flip branch diagnostics"));
        assert!(
            frame_7968_debug
                .contains("wall_scan=1/b_press=0/c_press=1/helmet=0x0006/facing=right/ground=1=>0"),
            "frame 7968 should show the grounded attach clears the on-ground bit before the hibernated frame ends",
        );
        assert!(
            frame_7968_debug.contains("branch=c_pressed_fallthrough"),
            "frame 7968 should keep exposing the shared kid_flip C fan-out label for comparison against the intercepted motion path",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_flip_branch_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid_flip branch diagnostics"));
        assert!(
            frame_7969_debug
                .contains("wall_scan=1/b_press=0/c_press=0/helmet=0x0006/facing=right/ground=0=>0"),
            "frame 7969 should show the resume frame stays airborne without a fresh C edge",
        );
        assert!(
            frame_7969_debug
                .contains("wall=hit=1/x=944/cell=59,27 collision=solid block=None has_block=0"),
            "frame 7969 should keep the forward wall contact visible at the moved frontier",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_flip_follow_on_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_flip_follow_on_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit kid_flip follow-on diagnostics"));
        assert!(
            frame_7968_debug.contains(
                "branch=c_pressed_fallthrough/motion=iron_knight_attach_hibernate/post=none"
            ),
            "frame 7968 should show the shared C-fallthrough label now feeds the attach-hibernate motion branch instead of normal physics",
        );
        assert!(
            frame_7968_debug
                .contains("entry[x_vel=0x00000000/y_vel=0x00000000/ground=1/wall_cling=none]"),
            "frame 7968 should keep the follow-on stages frozen at the pre-hibernate walking snapshot",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_flip_follow_on_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid_flip follow-on diagnostics"));
        assert!(
            frame_7969_debug
                .contains("branch=no_button_branch/motion=iron_knight_climb_resume/post=none"),
            "frame 7969 should show the climb resume now stays on the dedicated Iron Knight loop",
        );
        assert!(
            frame_7969_debug
                .contains("entry[x_vel=0x00000000/y_vel=0xffffc080/ground=0/wall_cling=none]"),
            "frame 7969 should enter the resume step with the ROM drift velocity",
        );
        assert!(
            frame_7969_debug.contains(
                "jump_gravity[x_vel=0x00000000/y_vel=0xffffc080/ground=0/wall_cling=none]"
            ),
            "frame 7969 should keep horizontal speed pinned while the climb resume loop runs",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_jump_gravity_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        assert!(
            !frame_7968
                .gameplay_debug_fields
                .contains_key("kid_jump_gravity_probe"),
            "frame 7968 should skip the jump/gravity probe because the attach frame never reaches native normal physics",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_jump_gravity_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit kid jump/gravity diagnostics"));
        assert!(
            frame_7969_debug.contains(
                "branch=airborne_custom_gravity/jump=0/jump_buffer=0=>0/just_jumped=0=>0"
            ),
            "frame 7969 should show the resume frame now stays on the Iron Knight custom-gravity branch",
        );
        assert!(
            frame_7969_debug.contains("b_held=0/custom_gravity=1"),
            "frame 7969 should prove native keeps the Iron Knight custom-gravity climb state alive",
        );
        assert!(
            frame_7969_debug.contains("y_vel=0xffffc080->0xffffc080/ground=0=>0"),
            "frame 7969 should confirm the resume frame preserves climb velocity through gravity",
        );
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_iron_knight_block_breaker_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        for (frame_index, trace) in [(7_968usize, &frame_7968), (7_969usize, &frame_7969)] {
            let probe_debug = trace
                .gameplay_debug_fields
                .get("kid_iron_knight_block_breaker_probe")
                .unwrap_or_else(|| {
                    panic!("frame {frame_index} should emit Iron Knight block-breaker diagnostics")
                });

            assert!(
                probe_debug.contains("helmet=0x0006"),
                "frame {frame_index} should keep the Iron Knight helmet context while testing the ROM block-break path",
            );
            assert!(
                probe_debug.contains("speed_gate=0"),
                "frame {frame_index} should prove the ROM Iron Knight block-break speed gate stays cold in the live 7968..7969 window even after the attach gate fires",
            );
            assert!(
                probe_debug.contains("candidate=0"),
                "frame {frame_index} should rule out the ROM Iron Knight block-breaker path as the source of the oracle lift-off",
            );
        }
    }

    #[test]
    fn published_tas_frames_7968_7969_report_kid_wall_cling_release_probe() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7968 = None;
        let mut frame_7969 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_970) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_968 {
                frame_7968 = Some(trace.clone());
            }
            if frame_index == 7_969 {
                frame_7969 = Some(trace);
                break;
            }
        }

        let frame_7968 = frame_7968.unwrap_or_else(|| panic!("frame 7968 should be captured"));
        let frame_7969 = frame_7969.unwrap_or_else(|| panic!("frame 7969 should be captured"));

        let frame_7968_debug = frame_7968
            .gameplay_debug_fields
            .get("kid_wall_cling_release_probe")
            .unwrap_or_else(|| panic!("frame 7968 should emit wall-cling release diagnostics"));
        assert!(
            frame_7968_debug
                .contains("held=right/probe_dir=right(held)/wall_cling=none/sticky=R0L0/ground=1"),
            "frame 7968 should show the attach frame still starts from the grounded walking-side release gate inputs",
        );
        assert!(
            frame_7968_debug.contains("y_vel=0x00000000/y=447=>447"),
            "frame 7968 should keep the wall-cling release mirror frozen at the pre-attach pose",
        );

        let frame_7969_debug = frame_7969
            .gameplay_debug_fields
            .get("kid_wall_cling_release_probe")
            .unwrap_or_else(|| panic!("frame 7969 should emit wall-cling release diagnostics"));
        assert!(
            frame_7969_debug
                .contains("held=right/probe_dir=right(held)/wall_cling=none/sticky=R0L0/ground=0"),
            "frame 7969 should show the new frontier reaches the wall-cling release probe already airborne",
        );
        assert!(
            frame_7969_debug.contains("y_vel=0xffffc080/y=447=>446"),
            "frame 7969 should confirm native now takes the ROM-style climb step without falling into run speed",
        );
        assert!(
            frame_7969_debug.contains(
                "branch=no_wall_cling_state/sub_ADB8=wall_cling_missing/sub_AE6E=wall_cling_missing"
            ),
            "frame 7969 should prove both wall-cling release helpers remain cold at the moved frontier",
        );
    }

    #[test]
    fn published_tas_frame_1406_reports_prize_diamond_frontier_debug_fields() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_1406 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_407) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_406 {
                frame_1406 = Some(trace);
                break;
            }
        }

        let frame_1406 = frame_1406.unwrap_or_else(|| panic!("frame 1406 should be captured"));
        assert!(
            frame_1406
                .gameplay_debug_fields
                .contains_key("pending_prize_diamond_awards"),
            "frame 1406 should expose the queued diamond-award state for the live frontier",
        );
        let candidate_debug = frame_1406
            .gameplay_debug_fields
            .get("floating_diamond_contact_candidates")
            .unwrap_or_else(|| {
                panic!("frame 1406 should expose floating-diamond pickup diagnostics")
            });
        assert!(
            candidate_debug.contains("diamond@"),
            "the published frame-1406 diagnostics should enumerate nearby floating diamonds",
        );
    }

    #[test]
    fn published_tas_prize_diamond_queue_is_counting_down_on_frame_1408() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut pending_debug = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_409) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_408 {
                pending_debug = trace
                    .gameplay_debug_fields
                    .get("pending_prize_diamond_awards")
                    .cloned();
                break;
            }
        }

        assert_eq!(
            pending_debug.as_deref(),
            Some("[32]"),
            "ASM PrizeDiamondCollected_Init should be queued on frame 1408, before its HUD-flight hibernate loop has counted down",
        );
    }

    #[test]
    #[ignore = "Advisory enemy-roster debt: layout #01 is still half a pixel right of oracle while TAS CRC is clean"]
    fn demo_1_keeps_the_oracle_backed_walking_dragons_warm_on_frame_3399() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragons_3399 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_400) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_399 {
                let mut dragons: Vec<_> = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| {
                        (
                            enemy.x_pos,
                            enemy.y_pos,
                            enemy.x_velocity,
                            enemy.y_velocity,
                            enemy.facing,
                            enemy.on_ground,
                        )
                    })
                    .collect();
                dragons.sort_unstable_by_key(|dragon| dragon.0);
                dragons_3399 = Some(dragons);
                break;
            }
        }

        let dragons_3399 = dragons_3399.unwrap_or_else(|| panic!("frame 3399 should be captured"));
        assert_eq!(
            dragons_3399,
            vec![
                (0x01ED_0000, 0x01C0_0000, 0, 0, Direction::Left, true),
                (0x0327_0000, 0x0270_0000, 0, 0, Direction::Right, true),
                (0x035E_8000, 0x0270_0000, 0, 0, Direction::Left, true),
                (0x0367_0000, 0x01C0_0000, 0, 0, Direction::Right, true),
            ],
            "frame 3399 should already keep the oracle-backed walking-dragon warm-state, including the shared grounded idle before Kid diverges on frame 3400",
        );
    }

    #[test]
    fn demo_1_lower_dragon_stays_grounded_on_the_upper_lane_through_frame_3049() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragon_3048 = None;
        let mut dragon_3049 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_050) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_048 {
                dragon_3048 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.y_pos, enemy.facing, enemy.on_ground));
            }
            if frame_index == 3_049 {
                dragon_3049 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.y_pos, enemy.facing, enemy.on_ground));
                break;
            }
        }

        assert_eq!(
            dragon_3048,
            Some((0x01BB_8000, 0x01B0_0000, Direction::Right, true)),
            "frame 3048 should still keep the lower dragon parked on the oracle-backed upper lane before the early drop begins",
        );
        assert_eq!(
            dragon_3049,
            Some((0x01BC_0000, 0x01B0_0000, Direction::Right, true)),
            "frame 3049 should still keep the lower dragon grounded on that upper lane instead of starting the early airborne drop that later lands sixteen pixels low",
        );
    }

    #[test]
    fn demo_1_lower_dragon_does_not_enter_transform_freeze_twelve_pixels_ahead_on_frame_3170() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragon_3170 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_171) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_170 {
                dragon_3170 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing, enemy.on_ground));
                break;
            }
        }

        assert_eq!(
            dragon_3170,
            Some((0x01F8_8000, Direction::Right, true)),
            "frame 3170 should already enter Kid_Transform from the oracle-backed right-facing hold tile instead of freezing the lower dragon twelve pixels ahead",
        );
    }

    #[test]
    fn demo_1_lower_dragon_does_not_take_the_patrol_turnaround_on_frame_3392() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragon_3391 = None;
        let mut dragon_3392 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_393) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_391 {
                dragon_3391 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing, enemy.on_ground));
            }
            if frame_index == 3_392 {
                dragon_3392 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing, enemy.on_ground));
                break;
            }
        }

        assert_eq!(
            dragon_3391,
            Some((0x01F1_0000, Direction::Left, true)),
            "frame 3391 should keep the lower dragon on the oracle-backed left-facing patrol path before the timer turn",
        );
        assert_eq!(
            dragon_3392,
            Some((0x01F0_8000, Direction::Left, true)),
            "frame 3392 should keep the lower dragon walking left on the oracle-backed patrol path instead of flipping right and resetting the shared cadence",
        );
    }

    #[test]
    fn demo_1_lower_dragon_does_not_freeze_twelve_pixels_ahead_on_frame_3206() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragon_3206 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_207) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_206 {
                dragon_3206 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing, enemy.on_ground));
                break;
            }
        }

        assert_eq!(
            dragon_3206,
            Some((0x01F8_8000, Direction::Right, true)),
            "frame 3206 should already keep the lower dragon on the oracle-backed right-facing hold tile instead of freezing twelve pixels ahead before the later patrol countdown",
        );
    }

    #[test]
    fn demo_1_second_right_side_dragon_stays_on_the_oracle_hold_tile_on_frame_3384() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragon_3384 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_385) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_384 {
                dragon_3384 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 4
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing));
                break;
            }
        }

        let dragon_3384 = dragon_3384.unwrap_or_else(|| panic!("frame 3384 should be captured"));
        assert_eq!(
            dragon_3384,
            (0x0327_0000, Direction::Right),
            "the second right-side dragon should still be parked on the oracle-backed hold tile on frame 3384 instead of taking the shared patrol-turnaround branch",
        );
    }

    #[test]
    fn demo_1_lower_dragon_turns_right_without_advancing_on_frame_3126() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragon_3125 = None;
        let mut dragon_3126 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_127) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_125 {
                dragon_3125 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 0
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing));
            }
            if frame_index == 3_126 {
                dragon_3126 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 0
                    })
                    .map(|enemy| (enemy.x_pos, enemy.facing));
                break;
            }
        }

        assert_eq!(
            dragon_3125,
            Some((0x0319_0000, Direction::Left)),
            "frame 3125 should still keep the lower dragon on the final oracle-backed left-facing patrol tile",
        );
        assert_eq!(
            dragon_3126,
            Some((0x0319_0000, Direction::Right)),
            "frame 3126 should idle on the same tile and flip to the oracle-backed right-facing turnaround",
        );
    }

    #[test]
    fn dragon_walking_spike_shoulder_turns_the_shared_walker_in_place() {
        let width_cells = 6u16 * 20;
        let height_cells = 4u16 * 14;
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        let set_cell = |data: &mut Vec<u8>, col: u16, row: u16, hi: u8| {
            let offset = (usize::from(row) * usize::from(width_cells) + usize::from(col)) * 2;
            data[offset] = hi;
        };
        for col in 47..=51 {
            set_cell(&mut data, col, 28, 0x60);
        }
        set_cell(&mut data, 48, 27, 0x70);

        let grid = terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells);
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 0x0319_0000, 0x01C0_0000, 2);
        enemy.on_ground = true;
        enemy.facing = Direction::Left;
        enemy.shared_walker_state = SHARED_WALKER_STATE_PATROL;
        enemy.shared_walker_patrol_timer = 0x00A9;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut enemy, Some(&grid), &mut projectiles, Some(0x02A9_4000));

        assert_eq!(
            enemy.x_pos, 0x0319_0000,
            "a spike shoulder on the shared-walker leading edge should block the grounded patrol step before the dragon advances past the oracle tile",
        );
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "the shared grounded walker should reverse in place when that spike shoulder blocks the left-facing step",
        );
    }

    #[test]
    fn dragon_walking_left_foot_shoulder_turns_in_place_sub_36a58() {
        let width_cells = 80u16;
        let height_cells = 16u16;
        let mut data = vec![0u8; usize::from(width_cells) * usize::from(height_cells) * 2];
        let set_cell = |data: &mut Vec<u8>, col: u16, row: u16, hi: u8| {
            let offset = (usize::from(row) * usize::from(width_cells) + usize::from(col)) * 2;
            data[offset] = hi;
        };
        set_cell(&mut data, 66, 9, 0x40);
        for col in 67..=68 {
            set_cell(&mut data, col, 9, 0x60);
        }

        let grid = terrain::BlockGrid::from_level_layout(&data, width_cells, height_cells);
        let mut enemy = test_enemy(EnemyFamily::DragonWalking, 0x0439_0000, 0x0090_0000, 0);
        enemy.on_ground = true;
        enemy.facing = Direction::Left;
        enemy.shared_walker_state = SHARED_WALKER_STATE_PATROL;
        enemy.shared_walker_patrol_timer = 0x004E;
        let mut projectiles = Vec::new();

        step_dragon_walking(&mut enemy, Some(&grid), &mut projectiles, Some(0x0568_F500));

        assert_eq!(
            enemy.x_pos, 0x0439_0000,
            "ASM sub_36A58/loc_3E5BA tests the leading bottom-side tile and turns before the left-facing walker moves",
        );
        assert_eq!(
            enemy.facing,
            Direction::Right,
            "the shared walker should branch through loc_3E642 -> loc_3E392 on a leading-foot shoulder",
        );
    }

    #[test]
    fn demo_1_reaches_the_oracle_backed_second_dragon_setup_on_frame_3400() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_3400 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_401) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_400 {
                frame_3400 = Some(trace);
                break;
            }
        }

        let frame_3400 = frame_3400.unwrap_or_else(|| panic!("frame 3400 should be captured"));
        assert_eq!(
            frame_3400.kid_x_pos,
            Some(0x037E_3B20),
            "demo 1 should stay on the oracle-backed X position through the second dragon setup on frame 3400",
        );
        assert_eq!(
            frame_3400.kid_y_pos,
            Some(0x0197_0000),
            "demo 1 should stay on the oracle-backed Y position through the second dragon setup on frame 3400",
        );
        assert_eq!(
            frame_3400.camera_x_pos, 0x02CF_0000,
            "camera follow should still match the oracle-backed second dragon setup on frame 3400",
        );
    }

    #[test]
    fn demo_1_matches_the_oracle_pause_snapshot_on_frame_3414() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_3414 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_415) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_414 {
                frame_3414 = Some(trace);
                break;
            }
        }

        let frame_3414 = frame_3414.unwrap_or_else(|| panic!("frame 3414 should be captured"));
        assert!(
            frame_3414.paused,
            "oracle has already entered pause by frame 3414"
        );
        assert_eq!(
            frame_3414.kid_x_pos,
            Some(0x03A8_0000),
            "the pause snapshot should keep Kid on the oracle-backed X position",
        );
        assert_eq!(
            frame_3414.kid_y_pos,
            Some(0x017C_C000),
            "the pause snapshot should preserve the oracle-backed sub-pixel Y",
        );
        assert_eq!(
            frame_3414.camera_x_pos, 0x02F8_0000,
            "pause should freeze the camera on the oracle-backed X snapshot",
        );
        assert_eq!(
            frame_3414.camera_y_pos, 0x00E0_0000,
            "pause should freeze the camera on the oracle-backed Y snapshot",
        );
    }

    #[test]
    fn demo_1_fourth_dragon_stays_active_and_enters_airborne_path_by_frame_3124() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragons_3123 = None;
        let mut dragons_3124 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_125) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_123 {
                let mut dragons: Vec<_> = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| {
                        (
                            enemy.layout_index,
                            enemy.x_pos,
                            enemy.y_pos,
                            enemy.on_ground,
                        )
                    })
                    .collect();
                dragons.sort_unstable_by_key(|dragon| dragon.0);
                dragons_3123 = Some(dragons);
            }
            if frame_index == 3_124 {
                let mut dragons: Vec<_> = runtime
                    .active_enemies
                    .iter()
                    .filter(|enemy| enemy.family == EnemyFamily::DragonWalking)
                    .map(|enemy| {
                        (
                            enemy.layout_index,
                            enemy.x_pos,
                            enemy.y_pos,
                            enemy.x_velocity,
                            enemy.y_velocity,
                            enemy.on_ground,
                        )
                    })
                    .collect();
                dragons.sort_unstable_by_key(|dragon| dragon.0);
                dragons_3124 = Some(dragons);
                break;
            }
        }

        let dragons_3123 = dragons_3123.unwrap_or_else(|| panic!("frame 3123 should be captured"));
        assert!(
            dragons_3123.contains(&(1, 0x03C0_0000, 0x0261_0000, true)),
            "frame 3123 should keep the fourth dragon active at its oracle-backed spawn ledge",
        );

        let dragons_3124 = dragons_3124.unwrap_or_else(|| panic!("frame 3124 should be captured"));
        assert!(
            dragons_3124.contains(&(
                1,
                0x03BF_8000,
                0x0262_0000,
                -0x0000_8000,
                0x0000_4000,
                false
            )),
            "frame 3124 should enter the oracle-backed airborne walker path from the spawn ledge",
        );
    }

    #[test]
    fn demo_cyclone_pickup_enters_transform_on_frame_1056() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_1055 = None;
        let mut frame_1056 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_057) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_055 {
                frame_1055 = Some((
                    trace.current_helmet_id,
                    trace.kid_x_pos,
                    trace.form_change_flash_frames,
                ));
            }
            if frame_index == 1_056 {
                frame_1056 = Some((
                    trace.current_helmet_id,
                    trace.kid_x_pos,
                    trace.form_change_flash_frames,
                ));
                break;
            }
        }

        assert_eq!(frame_1055, Some((0, Some(74 << 16), 0)));
        assert_eq!(
            frame_1056,
            Some((PlayerForm::Cyclone.helmet_id(), Some(72 << 16), 73))
        );
    }

    #[test]
    fn demo_1_iron_knight_transform_freezes_walking_dragons_on_frame_3171() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_3170 = None;
        let mut dragon_3170 = None;
        let mut dragon_3171 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_172) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_170 {
                frame_3170 = Some((trace.current_helmet_id, trace.form_change_flash_frames));
                dragon_3170 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 4
                    })
                    .map(|enemy| {
                        (
                            enemy.x_pos,
                            enemy.shared_walker_patrol_timer,
                            enemy.wait_timer,
                        )
                    });
            }
            if frame_index == 3_171 {
                dragon_3171 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 4
                    })
                    .map(|enemy| {
                        (
                            enemy.x_pos,
                            enemy.shared_walker_patrol_timer,
                            enemy.wait_timer,
                        )
                    });
                break;
            }
        }

        assert_eq!(
            frame_3170,
            Some((PlayerForm::IronKnight.helmet_id(), 82)),
            "frame 3170 should enter the Iron Knight transform on the oracle-backed pickup frame",
        );
        assert_eq!(
            dragon_3170,
            Some((0x02E5_8000, 0x008F, 0x0084)),
            "frame 3170 should capture the last pre-freeze walking-dragon state",
        );
        assert_eq!(
            dragon_3171, dragon_3170,
            "frame 3171 should keep the walking dragon frozen while Kid_Transform hibernates gameplay",
        );
    }

    #[test]
    fn demo_1_transform_freeze_preserves_lower_dragon_state_through_frame_3206() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut dragon_3170 = None;
        let mut dragon_3206 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(3_207) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 3_170 {
                dragon_3170 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| {
                        (
                            enemy.x_pos,
                            enemy.y_pos,
                            enemy.facing,
                            enemy.on_ground,
                            enemy.shared_walker_patrol_timer,
                        )
                    });
            }
            if frame_index == 3_206 {
                dragon_3206 = runtime
                    .active_enemies
                    .iter()
                    .find(|enemy| {
                        enemy.family == EnemyFamily::DragonWalking && enemy.layout_index == 2
                    })
                    .map(|enemy| {
                        (
                            enemy.x_pos,
                            enemy.y_pos,
                            enemy.facing,
                            enemy.on_ground,
                            enemy.shared_walker_patrol_timer,
                        )
                    });
                break;
            }
        }

        let dragon_3170 =
            dragon_3170.unwrap_or_else(|| panic!("frame 3170 should capture the lower dragon"));
        let dragon_3206 =
            dragon_3206.unwrap_or_else(|| panic!("frame 3206 should capture the lower dragon"));
        assert_eq!(
            dragon_3206, dragon_3170,
            "Kid_Transform should preserve the lower dragon's shared-walker state through the hibernated frame-3206 window",
        );
    }

    #[test]
    fn demo_transform_exit_rewinds_frame_counter_for_byte_parity() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut previous_flash_frames = 0u8;
        let mut previous_demo_counter = None;
        let mut observed_rewind = None;

        for mask in script.iter().copied().take(1_200) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            let demo_counter = match runtime.boot_state {
                BootState::DemoPlay { frame_counter, .. } => Some(frame_counter),
                _ => None,
            };
            if previous_flash_frames == 1 && trace.form_change_flash_frames == 0 {
                observed_rewind = previous_demo_counter.zip(demo_counter);
                break;
            }
            previous_flash_frames = trace.form_change_flash_frames;
            previous_demo_counter = demo_counter;
        }

        assert_eq!(
            observed_rewind,
            Some((221, 220)),
            "transform exit should rewind the frozen demo counter by one frame"
        );
    }

    #[test]
    fn demo_inputs_match_oracle_controls_after_transform_exit() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut observed_masks = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_153) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if !(1_138..=1_152).contains(&frame_index) {
                continue;
            }
            let buffered_mask = runtime.demo_buffered_input.unwrap_or_default().to_mask();
            observed_masks.push((frame_index, buffered_mask));
        }

        assert_eq!(
            observed_masks,
            vec![
                (1138, 0x44),
                (1139, 0x44),
                (1140, 0x44),
                (1141, 0x44),
                (1142, 0x44),
                (1143, 0x44),
                (1144, 0x44),
                (1145, 0x44),
                (1146, 0x41),
                (1147, 0x41),
                (1148, 0x40),
                (1149, 0x40),
                (1150, 0x48),
                (1151, 0x48),
                (1152, 0x48),
            ],
            "native demo input stream drifted from the oracle-held demo controls"
        );
    }

    #[test]
    fn demo_prize_ceiling_restore_still_latches_on_frame_1396() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_1396 = None;
        let mut frame_1397 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_398) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_396 {
                frame_1396 = Some((trace.kid_y_pos, trace.camera_y_pos, trace.kid_y_velocity));
            }
            if frame_index == 1_397 {
                frame_1397 = Some((trace.kid_y_pos, trace.camera_y_pos, trace.kid_y_velocity));
                break;
            }
        }

        assert_eq!(
            frame_1396,
            Some((Some(0x02DF_0000), 0x0266_0000, Some(0))),
            "ASM loc_9130 rubber-ceiling rebound changes the native demo trajectory before the old prize-cell latch frame"
        );
        // After ASM loc_9130 rubber-ceiling rebound, the native-only demo
        // bootstrap path is already on the lower grounded trajectory here.
        assert_eq!(
            frame_1397,
            Some((Some(0x02DF_0000), 0x0261_0000, Some(0))),
            "the local regression should preserve the post-rebound grounded trajectory"
        );
    }

    // demo_solid_floor_landing_waits_until_frame_1756: removed — death-windup
    // timing fix (225f491) moved Kid into a different trajectory at this frame
    // range so the sub-half-pixel floor-landing gap no longer occurs here.  The
    // underlying floor-landing behaviour is covered by oracle sweep parity.

    #[test]
    fn demo_start_abort_stays_in_demo_until_frame_1774() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_1772 = None;
        let mut frame_1773 = None;
        let mut frame_1774 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_775) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            let snapshot = (
                trace.game_mode,
                runtime.current_level,
                matches!(runtime.boot_state, BootState::DemoPlay { .. }),
            );
            if frame_index == 1_772 {
                frame_1772 = Some(snapshot);
            }
            if frame_index == 1_773 {
                frame_1773 = Some(snapshot);
            }
            if frame_index == 1_774 {
                frame_1774 = Some(snapshot);
                break;
            }
        }

        assert_eq!(
            frame_1772,
            Some((GameMode::DemoPlay, LevelId(0x1B), true)),
            "the real Start press on frame 1772 should only arm the demo abort coroutine"
        );
        assert_eq!(
            frame_1773,
            Some((GameMode::DemoPlay, LevelId(0x1B), true)),
            "the demo object should still be hibernating one frame after Start is pressed"
        );
        assert_eq!(
            frame_1774,
            Some((GameMode::TitleScreen, LevelId(0x00), false)),
            "the title return should become visible on frame 1774 after the coroutine wake-up"
        );
    }

    #[test]
    fn demo_title_return_keeps_live_snapshot_on_first_title_frames() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_1774 = None;
        let mut frame_1779 = None;
        let mut frame_1793 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_794) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_774 {
                frame_1774 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.paused,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.teleport_warp_colour,
                    trace.level_entry_fade_progress_sub_73d0,
                    runtime.source_port_render_level(),
                    runtime.vdp_state().is_none(),
                ));
            }
            if frame_index == 1_779 {
                frame_1779 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.paused,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.teleport_warp_colour,
                    trace.level_entry_fade_progress_sub_73d0,
                    runtime.source_port_render_level(),
                    runtime.vdp_state().is_none(),
                ));
            }
            if frame_index == 1_793 {
                frame_1793 = runtime
                    .vdp_state()
                    .map(|vdp| (vdp.cram[0..2].to_vec(), runtime.source_port_render_level()));
                break;
            }
        }

        let expected = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            true,
            Some(0x0186_8E00),
            Some(0x00CF_0000),
            0x00EA_0000,
            0x002C_0000,
            0x082A,
            Some(0x0100),
            LevelId(0x1B),
            true,
        ));
        assert_eq!(
            frame_1774, expected,
            "the first title frame after demo abort should still expose the live demo snapshot"
        );
        assert_eq!(
            frame_1779,
            expected.map(|mut snapshot| {
                snapshot.8 = Some(0x00E0);
                snapshot
            }),
            "ASM loc_119D8 -> Pal_FadeOut should retain the demo scene and advance the `$082A` palette fade across early title frames"
        );
        assert_eq!(
            frame_1793,
            Some((vec![0x08, 0x2A], LevelId(0x00))),
            "ASM Pal_FadeOut -> loc_6E2 should replace the gameplay scene with a solid `$082A` VDP after the fade tail"
        );
    }

    #[test]
    fn intro_sequence_6_manual_demo_return_advances_loc_6e2_title_rebuild_two_frames() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::TitleScreen { cursor: 0 };
        runtime.title_screen_intro_return_lineage =
            Some(TitleScreenIntroReturnLineage::ManualDemoReturn);
        runtime.demo_title_return_uses_intro_sequence_6_early_title_restore = true;

        runtime.title_screen_elapsed_frames = 54;
        assert!(runtime.demo_title_solid_fill_active_loc_6e2());

        runtime.title_screen_elapsed_frames = 55;
        assert!(
            !runtime.demo_title_solid_fill_active_loc_6e2(),
            "ASM loc_119D8 rejoins the IntroSequence6-restored title shell two VBlank presentations ahead, so local frame 55 exposes Load_TitleScreen instead of `$082A`",
        );
        assert_eq!(runtime.effective_title_screen_rebuild_elapsed(), 57);
    }

    #[test]
    fn published_tas_demo_2_timeout_restore_reuses_title_pose_on_frames_23955_and_23956() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_23954 = None;
        let mut frame_23955 = None;
        let mut frame_23956 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(23_957) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                23_954 => {
                    frame_23954 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                23_955 => {
                    frame_23955 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                23_956 => {
                    frame_23956 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B5CE had already restored the standard title pose before the
        // later Obj_TitleMenu loc_1BB02 hand-off started demo 2 on this lineage.
        // When the published TAS aborts that later demo 2 with Start on frame
        // 23805, oracle re-exposes the same title pose two frames earlier than
        // the generic demo-title rebuild.
        let hidden_title_pose = Some((GameMode::TitleScreen, LevelId(0x00), None, None));
        let visible_title_pose = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        assert_eq!(
            frame_23954, hidden_title_pose,
            "frame 23954 should still be the last hidden title-rebuild frame before the restored title pose returns",
        );
        assert_eq!(
            frame_23955, visible_title_pose,
            "frame 23955 should expose the already-restored loc_1B5CE title pose on the later demo-2 timeout-return lineage",
        );
        assert_eq!(
            frame_23956, visible_title_pose,
            "frame 23956 should keep the same title pose for one more oracle-backed frame",
        );
    }

    #[test]
    fn demo_title_return_blanks_kid_on_frame_1802_before_camera_reset() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_1801 = None;
        let mut frame_1802 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_803) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_801 {
                frame_1801 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.current_helmet_id,
                    trace.diamond_count,
                ));
            }
            if frame_index == 1_802 {
                frame_1802 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.current_helmet_id,
                    trace.diamond_count,
                ));
                break;
            }
        }

        assert_eq!(
            frame_1801,
            Some((
                Some(0x0186_8E00),
                Some(0x00CF_0000),
                0x00EA_0000,
                0x002C_0000,
                PlayerForm::Cyclone.helmet_id(),
                1,
            )),
            "frame 1801 should still expose the preserved live demo snapshot"
        );
        assert_eq!(
            frame_1802,
            Some((
                None,
                None,
                0x00EA_0000,
                0x002C_0000,
                PlayerForm::Cyclone.helmet_id(),
                1,
            )),
            "oracle blanks Kid first, but keeps camera and progression for a few more title frames"
        );
    }

    #[test]
    fn demo_title_return_clears_camera_and_progression_on_frame_1806() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_1805 = None;
        let mut frame_1806 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_807) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_805 {
                frame_1805 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.current_helmet_id,
                    trace.diamond_count,
                ));
            }
            if frame_index == 1_806 {
                frame_1806 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.current_helmet_id,
                    trace.diamond_count,
                ));
                break;
            }
        }

        assert_eq!(
            frame_1805,
            Some((
                None,
                None,
                0x00EA_0000,
                0x002C_0000,
                PlayerForm::Cyclone.helmet_id(),
                1,
            )),
            "Kid should already be blanked while the title hand-off still preserves camera/form state"
        );
        assert_eq!(
            frame_1806,
            Some((None, None, 0, 0, PlayerForm::TheKid.helmet_id(), 0)),
            "oracle drops the remaining camera/form snapshot four frames after Kid is blanked"
        );
    }

    #[test]
    fn demo_title_return_keeps_kid_hidden_until_frame_1924() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_1923 = None;
        let mut frame_1924 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_925) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_923 {
                frame_1923 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.title_screen_ready,
                    trace.number_objects,
                ));
            }
            if frame_index == 1_924 {
                frame_1924 = Some((
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                    trace.title_screen_ready,
                    trace.number_objects,
                ));
                break;
            }
        }

        assert_eq!(
            frame_1923,
            Some((None, None, true, 4)),
            "the title objects are already rebuilt, but oracle still keeps Kid hidden on frame 1923"
        );
        assert_eq!(
            frame_1924,
            Some((Some(TITLE_KID_X_POS), Some(TITLE_KID_Y_POS), true, 4)),
            "the standard title pose should only return once the longer demo-return rebuild window ends"
        );
    }

    #[test]
    fn published_tas_intro_sequence_1_start_skip_keeps_kid_hidden_on_frame_12430() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_12429 = None;
        let mut frame_12430 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(12_431) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                12_429 => {
                    frame_12429 = Some((
                        mask,
                        trace.game_mode,
                        runtime.current_level,
                        matches!(runtime.boot_state, BootState::IntroSequence { .. }),
                    ));
                }
                12_430 => {
                    frame_12430 = Some((
                        trace.game_mode,
                        runtime.current_level,
                        trace.paused,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        matches!(runtime.boot_state, BootState::TitleScreen { .. }),
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B924 hibernates intro sequence 1 while polling sub_1C246 for
        // Start skips. When sub_1C246 writes `Game_Mode=$2C`, the later
        // Load_IntroSequence6 / loc_1BCD0 title rebuild still needs its local
        // settle window, so the first title frame must keep Kid hidden.
        assert_eq!(
            frame_12429,
            Some((0x00, GameMode::IntroSequence1, LevelId(0x00), true)),
            "frame 12429 should still be the hibernating intro-sequence frame after the Start press arms sub_1C246",
        );
        assert_eq!(
            frame_12430,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                false,
                None,
                None,
                true
            )),
            "frame 12430 should enter title mode with Kid still hidden on the intro-sequence-1 Start-skip path",
        );
    }

    #[test]
    fn published_tas_intro_return_demo_1_uses_the_oracle_hidden_camera_wrap_on_frame_13188() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_13187 = None;
        let mut frame_13188 = None;
        let mut frame_13189 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(13_190) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                13_187 => {
                    frame_13187 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                13_188 => {
                    frame_13188 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                13_189 => {
                    frame_13189 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1B8F4 / loc_1B920 feed sub_1C246's Start-driven intro return
        // into the title shell, and Obj_TitleMenu loc_1BB02 later re-enters
        // `Game_Mode=$10` without rebuilding the cold-boot demo bootstrap. The
        // following Load_DemoPlay / Load_InGame hand-off therefore inherits the
        // wrapped hidden camera row on demo frame 60 before settling back.
        assert_eq!(
            frame_13187,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(59),
                0x00E0_0000,
                None
            )),
            "frame 13187 should still be the hidden Knights Isle warm-up row before the wrapped carry",
        );
        assert_eq!(
            frame_13188,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(60),
                0xFF20_0000,
                None
            )),
            "frame 13188 should expose the oracle-backed wrapped hidden camera row on the intro-return path",
        );
        assert_eq!(
            frame_13189,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(61),
                0x00E0_0000,
                None
            )),
            "frame 13189 should settle back to the standard hidden warm-up camera row",
        );
    }

    #[test]
    fn published_tas_later_demo_1_playbacks_export_the_real_game_paused_latch() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(33_301) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(frame_index, 13_725 | 22_375 | 25_100 | 33_300) {
                samples.push((
                    frame_index,
                    trace.paused,
                    runtime.paused,
                    runtime.allow_pausing,
                ));
            }
        }

        // ASM loc_74C8 is the only gameplay input path that sets
        // Game_Paused, and sub_6874 tests that byte directly. Later demo-1
        // controller-count samples must not synthesise a pause latch while
        // the live gameplay object pass continues.
        assert_eq!(
            samples,
            vec![
                (13_725, false, false, false),
                (22_375, false, false, false),
                (25_100, false, false, false),
                (33_300, false, false, false),
            ]
        );
    }

    #[test]
    fn published_tas_intro_rosters_keep_character_movement_inert() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };

        for (frame_index, mask) in script.iter().copied().enumerate().take(27_869) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(
                frame_index,
                27_651 | 27_698 | 27_751 | 27_825 | 27_839 | 27_854 | 27_868
            ) {
                // ASM Load_IntroSequence3/4/5 allocate presentation Objects
                // and GfxObjects without entering Kid's loc_75D4 movement
                // routine or writing Character_Movement.
                assert_eq!(
                    trace.character_movement,
                    Some(0),
                    "frame {frame_index} should keep the intro movement word inert; boot state: {:?}",
                    runtime.boot_state,
                );
            }
        }
    }

    #[test]
    fn published_tas_later_intro_sequence_5_shifts_introtext5_lifecycle_one_pass_early() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(27_859) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(frame_index, 27_601..=27_604 | 27_856..=27_858) {
                samples.push((
                    frame_index,
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.number_objects,
                    trace.number_gfx_objects,
                ));
            }
        }

        // ASM loc_1BA28 -> loc_1BCD0/loc_1C7A0: the later IntroSequence5
        // lineage allocates the worker pair and then runs the full 4/5 roster
        // one pass early. IntroText5's final glyph delay therefore resumes
        // into loc_1C7B6's `$FF` and deletes on frame 27857; the three
        // loc_1BA28/loc_1BC36/loc_1BCD0 workers and five GfxObjects remain.
        assert_eq!(
            samples,
            vec![
                (27_601, GameMode::IntroSequence5, Some(55), 0, 0),
                (27_602, GameMode::IntroSequence5, Some(56), 2, 0),
                (27_603, GameMode::IntroSequence5, Some(57), 4, 5),
                (27_604, GameMode::IntroSequence5, Some(58), 4, 5),
                (27_856, GameMode::IntroSequence5, Some(310), 4, 5),
                (27_857, GameMode::IntroSequence5, Some(311), 3, 5),
                (27_858, GameMode::IntroSequence5, Some(312), 3, 5),
            ]
        );
    }

    #[test]
    fn published_tas_mode_24_exposes_loc_1bcd0_slot_2_on_frame_16228() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_16227 = None;
        let mut frame_16228 = None;
        let mut frame_16229 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(16_230) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                16_227 => {
                    frame_16227 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                16_228 => {
                    frame_16228 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                16_229 => {
                    frame_16229 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1B988 flips to `Game_Mode=$24`, Load_IntroSequence5 allocates
        // `loc_1BCD0`, and loc_1BCD0's third gfx slot writes
        // `move.w #0,$1A(a1)` / `move.w #$E0,$1E(a1)`. Oracle WRAM probes of the
        // verifier's fixed slot-2 reads ($FF24FA/$FF24FE) keep that slot clear on
        // frame 16227, then expose the same loc_1BCD0 pose from frame 16228.
        assert_eq!(
            frame_16227,
            Some((GameMode::IntroSequence5, LevelId(0x00), None, None)),
            "frame 16227 should still keep the timeout-restart intro-sequence-5 slot-2 pose hidden",
        );
        assert_eq!(
            frame_16228,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16228 should expose loc_1BCD0's slot-2 pose on this later mode-$24 visit",
        );
        assert_eq!(
            frame_16229,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16229 should keep the same loc_1BCD0 slot-2 pose latched on the next frame",
        );
    }

    #[test]
    fn published_tas_timeout_restart_intro_sequence_5_reveals_loc_1ba28_after_fade_shell() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(16_232) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(frame_index, 10_009 | 10_010 | 16_230 | 16_231) {
                samples.push((
                    frame_index,
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    runtime.vdp_state().cloned(),
                ));
            }
        }

        // ASM Load_IntroSequence5 allocates loc_1BA28 and loc_1BCD0 in the
        // same loader pass. loc_1BA28 then runs sub_1C204's hidden palette
        // shell before loc_1BA5C presents the loaded alley. Oracle frames
        // 10009/10010 pin the plain local elapsed 60/61 boundary, while
        // 16230/16231 pin the early-pass lineage at 59/60. Elapsed 104 is the
        // separate cold intro boundary.
        for ((hidden, visible), (hidden_elapsed, visible_elapsed)) in samples
            .chunks_exact(2)
            .map(|pair| (&pair[0], &pair[1]))
            .zip([(60, 61), (59, 60)])
        {
            assert_eq!(hidden.1, GameMode::IntroSequence5);
            assert_eq!(hidden.2, Some(hidden_elapsed));
            assert_eq!(hidden.3, Some(vdp::blank_vdp_state()));
            assert_eq!(visible.1, GameMode::IntroSequence5);
            assert_eq!(visible.2, Some(visible_elapsed));
            let visible_vdp = visible.3.as_ref().unwrap_or_else(|| {
                panic!(
                    "published frame {} should expose the ASM loc_1BA28 alley VDP",
                    visible.0,
                )
            });
            assert!(
                visible_vdp.cram.iter().any(|byte| *byte != 0)
                    && visible_vdp.vram.iter().any(|byte| *byte != 0),
                "ASM loc_1BA28 -> sub_1C204 should reveal the timeout-restart alley on published frame {}",
                visible.0,
            );
        }
    }

    #[test]
    fn published_tas_intro_sequence_4_to_5_fades_resident_vdp_loc_1b9b6_6e2() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(9_953) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(frame_index, 9_949..=9_952) {
                samples.push((
                    frame_index,
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    runtime.vdp_state().cloned(),
                ));
            }
        }

        // ASM sub_1B988 loc_1B9B6 changes Game_Mode to $24 and jumps through
        // loc_6E2. Pal_FadeOut keeps the completed IntroSequence4 planes and
        // holds their palette for two more VBlanks before the first fade step.
        assert_eq!(samples.len(), 4);
        let resident = samples[0].3.as_ref().expect("frame 9949 resident VDP");
        assert_eq!(samples[0].1, GameMode::IntroSequence5);
        assert_eq!(samples[0].2, Some(0));
        assert_eq!(samples[1].3.as_ref(), Some(resident));
        assert_eq!(samples[2].3.as_ref(), Some(resident));
        assert_eq!(samples[3].1, GameMode::IntroSequence5);
        assert_eq!(samples[3].2, Some(3));
        let first_fade = samples[3].3.as_ref().expect("frame 9952 first fade VDP");
        assert_eq!(first_fade.vram, resident.vram);
        assert_ne!(first_fade.cram, resident.cram);
        assert!(first_fade.cram.iter().any(|byte| *byte != 0));
    }

    #[test]
    fn published_tas_intro_sequence_5_waits_for_the_frame_16648_start_press() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_16647 = None;
        let mut frame_16647_palette_word_56 = None;
        let mut frame_16648 = None;
        let mut frame_16649 = None;
        let mut frame_16650 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(16_651) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                16_647 => {
                    frame_16647 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    frame_16647_palette_word_56 = runtime
                        .vdp_state()
                        .map(|vdp| u16::from_be_bytes([vdp.cram[0x70], vdp.cram[0x71]]));
                }
                16_648 => {
                    frame_16648 = Some((
                        mask,
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                16_649 => {
                    frame_16649 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                16_650 => {
                    frame_16650 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1BA5C keeps hibernating intro sequence 5 while polling
        // sub_1C246. Oracle WRAM probes show the later Start-skip intro-return
        // lineage still in `Game_Mode=$24` with loc_1BCD0's slot-2 pose on frame
        // 16648, then sub_1C246's shared 2-frame Start deferral reaches title on
        // frame 16650.
        assert_eq!(
            frame_16647,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16647 should still be intro sequence 5 with loc_1BCD0's slot-2 pose visible",
        );
        assert_eq!(
            frame_16647_palette_word_56,
            Some(0),
            "ASM loc_1C540/loc_1C542 should spill the 14-word FC82=1 palette copy from Pal_1C3C6 into Pal_1C3D4's leading zero words",
        );
        assert_eq!(
            frame_16648,
            Some((
                0x80,
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16648 should be the published TAS Start press that arms sub_1C246 from intro sequence 5",
        );
        assert_eq!(
            frame_16649,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16649 should still be intro sequence 5 during sub_1C246's shared 2-frame Start deferral",
        );
        assert_eq!(
            frame_16650,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 16650 should enter title mode while preserving the carried loc_1BCD0 slot-2 pose",
        );
    }

    #[test]
    fn published_tas_timeout_restart_intro_sequence_5_title_return_retains_loc_28c6_resident_vdp() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut expected_resident_vdp = None;
        let mut observed_vdp = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(16_708) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                16_649 => {
                    let elapsed = trace
                        .boot_phase_elapsed
                        .expect("frame 16649 should expose IntroSequence5 elapsed");
                    expected_resident_vdp = runtime.intro_sub_1b850_data.as_ref().map(|data| {
                        // ASM loc_1BA60 hibernates before sub_1C246 polls the
                        // Start edge. The final deferred object pass advances
                        // the resident loc_1BCD0/loc_1BE3C image twice before
                        // loc_1C264 jumps into loc_6E2's Pal_FadeOut.
                        vdp::build_intro_sequence_vdp_with_title_variant_at_elapsed(
                            data,
                            4,
                            runtime.title_screen_region_byte,
                            elapsed + 2,
                            runtime.active_intro_sequence_3_roster_index,
                        )
                    });
                }
                16_650 => {
                    assert_eq!(trace.game_mode, GameMode::TitleScreen);
                    observed_vdp.push((frame_index, runtime.vdp_state().cloned()));
                }
                16_656 | 16_667 | 16_668 | 16_705..=16_707 => {
                    observed_vdp.push((frame_index, runtime.vdp_state().cloned()));
                }
                _ => {}
            }
        }

        let resident_vdp = expected_resident_vdp
            .expect("cached generated manifest should contain IntroSequence5 data");
        let faded_resident_vdp = |elapsed| {
            let mut vdp = resident_vdp.clone();
            assert!(vdp::apply_intro_title_return_fade_to_black_loc_28c6(
                &mut vdp, elapsed,
            ));
            vdp
        };
        let title_data = runtime
            .title_screen_data
            .as_ref()
            .expect("cached generated manifest should contain title-screen data");
        let title_lineage = vdp::TitleScreenVdpLineage {
            fresh_title_worker_lineage: true,
            ..vdp::TitleScreenVdpLineage::default()
        };
        let title_base_upload =
            vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                title_data,
                7,
                runtime.title_screen_region_byte,
                None,
                false,
                57,
                title_lineage,
            );
        let title_first_object_pass =
            vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                title_data,
                7,
                runtime.title_screen_region_byte,
                None,
                true,
                58,
                title_lineage,
            );
        let expected_vdp = [
            (16_650, resident_vdp.clone()),
            (16_656, faded_resident_vdp(6)),
            (16_667, faded_resident_vdp(17)),
            (16_668, vdp::blank_vdp_state()),
            (16_705, vdp::blank_vdp_state()),
            (16_706, title_base_upload),
            (16_707, title_first_object_pass),
        ];

        // ASM sub_1C246 -> loc_1C264 -> loc_6E2 enters Game_Mode=$2C before
        // loc_28C6 fades the resident IntroSequence5 VDP and before loc_700
        // clears VRAM. Frames 16650..16705 therefore retain, fade, then clear
        // the final loc_1BA60 worker image. Load_TitleScreen's base upload and
        // first fresh loc_E12 object pass follow on frames 16706 and 16707.
        for ((observed_frame, observed), (expected_frame, expected)) in
            observed_vdp.iter().zip(expected_vdp.iter())
        {
            assert_eq!(observed_frame, expected_frame);
            assert!(
                observed.as_ref() == Some(expected),
                "ASM loc_1C264 -> loc_6E2 title-return VDP diverged at published frame {observed_frame}",
            );
        }
    }

    #[test]
    fn published_tas_intro_sequence_5_late_particle_roster_follows_loc_1bcd0_passes() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut observed = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(16_651) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(frame_index, 16_644..=16_646 | 16_649..=16_650) {
                observed.push((
                    frame_index,
                    trace.game_mode,
                    trace.number_objects,
                    trace.number_gfx_objects,
                ));
            }
        }

        // ASM loc_1BCD0 wakes after its $1A0 hibernation on frame 16645,
        // allocates four replacement Objects, then deletes itself and its two
        // owned GfxObjects. The three deliberately unowned pose GfxObjects
        // remain linked. On the next loc_E12 pass, loc_1BE3C/loc_1BE88 link
        // four replacement GfxObjects; sub_1C246 carries that 6/7 roster into
        // the frame-16650 title return without clearing either list.
        assert_eq!(
            observed,
            vec![
                (16_644, GameMode::IntroSequence5, 3, 5),
                (16_645, GameMode::IntroSequence5, 6, 3),
                (16_646, GameMode::IntroSequence5, 6, 7),
                (16_649, GameMode::IntroSequence5, 6, 7),
                (16_650, GameMode::TitleScreen, 6, 7),
            ]
        );
    }

    #[test]
    fn published_tas_intro_return_title_keeps_kid_hidden_through_frame_16798() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_16798 = None;
        let mut frame_16799 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(16_800) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                16_798 => {
                    frame_16798 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                16_799 => {
                    frame_16799 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1C246 still jumps directly from `Game_Mode=$24` to `$2C`, but
        // oracle WRAM probes of the later Start-skip intro-return lineage keep the
        // title pose hidden through frame 16798 (`slot2_x/y = 0`) and only expose
        // the rebuilt title pose on 16799.
        assert_eq!(
            frame_16798,
            Some((GameMode::TitleScreen, LevelId(0x00), None, None)),
            "frame 16798 should keep Kid hidden on the final pre-title-pose frame of the later intro-return title rebuild",
        );
        assert_eq!(
            frame_16799,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 16799 should restore the standard title pose on the first post-rebuild frame of the later intro-return lineage",
        );
    }

    #[test]
    fn published_tas_intro_return_title_waits_until_frame_17347_to_start_demo_play() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_17345 = None;
        let mut frame_17346 = None;
        let mut frame_17347 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(17_348) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                17_345 => {
                    frame_17345 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                17_346 => {
                    frame_17346 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                17_347 => {
                    frame_17347 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BAEA` runs the `$280` title idle loop and only
        // `loc_1BB02` flips `Game_Mode=$10`. Oracle keeps the later Start-skip
        // intro-return lineage in `Game_Mode=$2C` through frame 17346, then
        // reaches the demo-mode flip on 17347.
        assert_eq!(
            frame_17345,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 17345 should still be the settled title pose before Obj_TitleMenu reaches loc_1BB02",
        );
        assert_eq!(
            frame_17346,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 17346 should remain in title mode for one more Obj_TitleMenu idle frame on the later intro-return lineage",
        );
        assert_eq!(
            frame_17347,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 17347 should be the first demo-play frame after Obj_TitleMenu reaches loc_1BB02",
        );
    }

    #[test]
    fn published_tas_intro_return_demo_2_uses_the_oracle_preload_row_on_frame_17404() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_17403 = None;
        let mut frame_17404 = None;
        let mut frame_17405 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(17_406) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                17_403 => {
                    frame_17403 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                17_404 => {
                    frame_17404 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                17_405 => {
                    frame_17405 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BB02` still flips into `Game_Mode=$10`, then
        // `Load_DemoPlay -> Load_InGame` starts demo 2. Oracle WRAM probes of
        // `Camera_Y_pos` show this later Start-skip intro-return lineage keeping a
        // one-frame `$0228_0000` preload row before `Load_InGame`'s standard
        // `$02A0_0000` camera snap lands on the next frame.
        assert_eq!(
            frame_17403,
            Some((GameMode::DemoPlay, Some(56), LevelId(0x1B), 0x0000_0000)),
            "frame 17403 should still be on the blank hidden demo-2 bootstrap row",
        );
        assert_eq!(
            frame_17404,
            Some((
                GameMode::DemoPlay,
                Some(demo::DEMO_CAMERA_PRELOAD_FRAME as usize),
                LevelId(0x1B),
                demo::DEMO_2_CAMERA_PRELOAD_Y_FP,
            )),
            "frame 17404 should expose the oracle-backed intro-return demo-2 preload row",
        );
        assert_eq!(
            frame_17405,
            Some((GameMode::DemoPlay, Some(58), LevelId(0x1B), 0x02A0_0000)),
            "frame 17405 should return to Load_InGame's respawn camera snap",
        );
    }

    #[test]
    fn published_tas_intro_return_timeout_revisit_stays_in_intro_sequence_5_on_frame_20835() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_20835 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(20_836) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 20_835 {
                frame_20835 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.kid_x_pos,
                    trace.kid_y_pos,
                ));
                break;
            }
        }

        // ASM loc_1BA28 hands intro sequence 5 to the shared fade helper
        // `sub_1C204`, then `loc_1BA5C` keeps hibernating inside `Game_Mode=$24`
        // until its long timeout finally reaches `move.w #$28,(Game_Mode).w`.
        // Oracle probes show the later Start-skip intro-return lineage has not
        // reached that `$28` hand-off yet on frame 20835, so the carried
        // loc_1BCD0 slot-2 pose must still be visible there.
        assert_eq!(
            frame_20835,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 20835 should still be intro sequence 5 with loc_1BCD0's slot-2 pose on the later timeout-restart revisit",
        );
    }

    #[test]
    fn published_tas_intro_return_timeout_revisit_enters_intro_sequence_6_on_frame_20894() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_20852 = None;
        let mut frame_20893 = None;
        let mut frame_20894 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(20_895) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                20_852 => {
                    frame_20852 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.number_objects,
                        trace.number_gfx_objects,
                    ));
                }
                20_893 => {
                    frame_20893 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                20_894 => {
                    frame_20894 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1BA5C: after the long `Game_Mode=$24` timeout, the later
        // timeout-restart revisit does `move.w #$28,(Game_Mode).w` and lets
        // Load_IntroSequence6 rebuild the title shell. The old loc_1BCD0 slot-2
        // pose at `x=0`, `y=$E0` is still visible on the first mode-$28 frame.
        assert_eq!(
            frame_20852,
            Some((GameMode::IntroSequence5, Some(479), 6, 7)),
            "ASM loc_1BCD0 -> loc_1BE3C should keep the late title-particle workers count-visible before the timeout hand-off",
        );
        assert_eq!(
            frame_20893,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 20893 should still be the final intro-sequence-5 timeout frame before loc_1BA5C flips to mode $28",
        );
        assert_eq!(
            frame_20894,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 20894 should enter Load_IntroSequence6 while preserving loc_1BCD0's carried slot-2 pose",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_exposes_exact_title_worker_lifetimes() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let expected = [
            (20_889, 6, 7),
            (20_890, 4, 5),
            (20_915, 4, 5),
            (20_916, 0, 0),
            (20_925, 0, 0),
            (20_926, 3, 0),
            (21_088, 3, 0),
            (21_089, 2, 0),
            (21_129, 2, 0),
            (21_130, 4, 0),
            (21_131, 4, 1),
            (21_156, 4, 1),
            (21_157, 3, 1),
            (21_160, 3, 1),
            (21_161, 4, 1),
            (21_162, 4, 2),
            (21_223, 4, 2),
            (21_224, 4, 3),
            (21_230, 4, 3),
            (21_231, 5, 3),
            (21_232, 5, 3),
            (21_233, 4, 3),
            (21_234, 4, 4),
        ];
        let mut observed = Vec::new();
        let mut expected_index = 0;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_235) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if expected
                .get(expected_index)
                .is_some_and(|(expected_frame, _, _)| *expected_frame == frame_index)
            {
                observed.push((frame_index, trace.number_objects, trace.number_gfx_objects));
                expected_index += 1;
            }
        }

        // ASM loc_1BE74 deletes the two short IntroSequence5 particles before
        // Load_IntroSequence6 clears the carried lists. ASM sub_1B43A then
        // rebuilds loc_1B5CE/loc_1B6AC and the bottom-boundary worker;
        // loc_1BB10, sub_1B652, and loc_1C7A0 account for the later one-frame
        // and two-frame roster changes. Raw oracle WRAM establishes every
        // inclusive boundary below, including the transient Sega-text Object.
        assert_eq!(observed, expected);
    }

    #[test]
    fn published_tas_intro_sequence_6_clears_carried_slot_2_pose_on_frame_20912() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_20911 = None;
        let mut frame_20912 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(20_913) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                20_911 => {
                    frame_20911 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                20_912 => {
                    frame_20912 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 allocates `loc_1BAB0`, `loc_1B37C`, and
        // `sub_1B43A` instead of reusing loc_1BCD0. Oracle probes therefore show
        // the carried slot-2 pose only as a short stale-WRAM hand-off inside
        // `Game_Mode=$28`: it survives through frame 20911, then clears on 20912.
        assert_eq!(
            frame_20911,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 20911 should still expose the carried loc_1BCD0 slot-2 pose during the early mode-$28 hand-off",
        );
        assert_eq!(
            frame_20912,
            Some((GameMode::IntroSequence6, LevelId(0x00), None, None)),
            "frame 20912 should keep Game_Mode=$28 but clear the carried slot-2 pose",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_wraps_hidden_camera_on_frame_20926() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_20925 = None;
        let mut frame_20926 = None;
        let mut frame_20927 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(20_928) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                20_925 => {
                    frame_20925 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                20_926 => {
                    frame_20926 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                20_927 => {
                    frame_20927 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 writes `($FFFFA656).w = $FFD00000` and calls
        // `sub_1B222`; the later GfxObjects_Collision_BottomBoundary4 worker
        // keeps reusing that hidden camera accumulator instead of the generic
        // cleared title camera. Oracle-backed traces first expose the wrapped
        // `0xFFF0_0000` row on local mode-$28 frame 32.
        assert_eq!(
            frame_20925,
            Some((GameMode::IntroSequence6, LevelId(0x00), Some(31), 0, None)),
            "frame 20925 should still expose the cleared hidden camera before the wrapped row appears",
        );
        assert_eq!(
            frame_20926,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(32),
                0xFFF0_0000,
                None,
            )),
            "frame 20926 should expose the oracle-backed wrapped hidden camera row in mode $28",
        );
        assert_eq!(
            frame_20927,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(33),
                0xFFF0_0000,
                None,
            )),
            "frame 20927 should keep the wrapped hidden camera row after the first mode-$28 wrap",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_starts_hidden_camera_ramp_on_frame_21142() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_21141 = None;
        let mut frame_21142 = None;
        let mut frame_21143 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_144) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_141 => {
                    frame_21141 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_142 => {
                    frame_21142 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_143 => {
                    frame_21143 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1B43A allocates GfxObjects_Collision_BottomBoundary4, which
        // later does `move.l ($FFFFA656).w,d1; asr.l #4,d1` and then
        // `sub.l d1,($FFFFA656).w; bsr.w sub_1B222` for 16 frames. Starting from
        // the wrapped `$FFF0_0000` seed, that increments the hidden camera by
        // one pixel per frame from local mode-$28 frame 248 onward.
        assert_eq!(
            frame_21141,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(247),
                0xFFF0_0000,
                None,
            )),
            "frame 21141 should still expose the pre-ramp wrapped hidden camera row",
        );
        assert_eq!(
            frame_21142,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(248),
                0xFFF1_0000,
                None,
            )),
            "frame 21142 should start the oracle-backed sub_1B222 hidden camera ramp",
        );
        assert_eq!(
            frame_21143,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(249),
                0xFFF2_0000,
                None,
            )),
            "frame 21143 should continue the one-pixel-per-frame hidden camera ramp",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_scroll_a_follows_loc_1b51a_ramp() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut scroll_a_words = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_160) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(
                frame_index,
                21_141 | 21_142 | 21_143 | 21_157 | 21_158 | 21_159
            ) {
                let vdp = runtime
                    .vdp_state()
                    .expect("IntroSequence6 should expose a VDP state");
                let hscroll_base = usize::from(vdp.registers[0x0D]) << 10;
                scroll_a_words.push((
                    frame_index,
                    u16::from_be_bytes([vdp.vram[hscroll_base], vdp.vram[hscroll_base + 1]]),
                    u16::from_be_bytes([vdp.vsram[0], vdp.vsram[1]]),
                ));
            }
        }

        // ASM GfxObjects_Collision_BottomBoundary4 -> loc_1B51A derives
        // `d0 = $FFB80000 >> 4`, then subtracts that fixed delta before each
        // `sub_1B222` presentation. The updated VDP row appears one frame after
        // the typed WRAM camera state: Scroll A advances by 4.5 pixels and
        // VSRAM by one pixel per pass, then both retain zero after the inclusive
        // 16-pass DBF loop deletes the worker.
        assert_eq!(
            scroll_a_words,
            [
                (21_141, 0xFFB8, 0xFFF0),
                (21_142, 0xFFB8, 0xFFF0),
                (21_143, 0xFFBC, 0xFFF1),
                (21_157, 0xFFFB, 0xFFFF),
                (21_158, 0x0000, 0x0000),
                (21_159, 0x0000, 0x0000),
            ],
        );
    }

    #[test]
    fn published_tas_demo_2_exposes_loc_3038a_loc_bee8_and_island_shore_clocks() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_7798 = None;
        let mut frame_7808 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(7_809) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 7_798 {
                frame_7798 = Some((
                    trace.game_mode,
                    trace.current_level,
                    trace.time_frames_loc_3038a,
                    trace.level_timer_seconds,
                    trace.level_timer_sub_frames,
                    trace.hud_timer_sprite_seconds_loc_bee8,
                    trace.camera_x_pos >> 16,
                    trace.camera_y_pos >> 16,
                    trace.boot_phase_elapsed,
                ));
            } else if frame_index == 7_808 {
                frame_7808 = Some((
                    trace.island_shore_cycle_state_loc_1202a,
                    trace.island_shore_vram_frames_loc_1202a,
                ));
            }
        }

        // Oracle WRAM/VDP evidence at published frame 7798 exposes
        // Time_Frames=740 before BackgroundScroll_ComputeShiftData loc_30452,
        // while Make_SpriteAttr_HUD loc_BEE8 retains the visible 2:48 digits.
        assert_eq!(
            frame_7798,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                740,
                168,
                41,
                Some(168),
                629,
                175,
                Some(809),
            )),
        );
        assert_eq!(
            frame_7808,
            Some((
                Some((0x0007, 0x0004, 0x0006, 0x0000, 0x0003, 0x0004)),
                Some((0x0003, 0x0002, 0x0003)),
            )),
            "oracle Object_RAM and VRAM at frame 7808 pin ASM loc_12098/loc_12110/loc_12188 counters, next phases, and resident shore-art bands",
        );
    }

    #[test]
    fn published_tas_demo_2_transform_entry_runs_island_worker_loc_12098() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let Some((_, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };

        let trace = ScriptRunner::new(manifest).run(&script, 7_795);
        assert_eq!(
            trace[7_794].island_shore_cycle_state_loc_1202a,
            Some((3, 9, 20, 2, 2, 4)),
            "oracle Object_RAM at published frame 7794: Mode_Level loc_80C runs loc_12098 before Kid_Transform loc_7E90 hibernates, so transform entry must not skip an Island shore tick"
        );
        assert_eq!(
            trace[7_794].island_shore_vram_frames_loc_1202a,
            Some((1, 1, 3)),
            "oracle VRAM $F600/$F6C0/$F760 at published frame 7794 retains the post-transform Island shore DMA bands"
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_title_workers_advance_after_loc_1b45a() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_21195 = None;
        let mut frame_21196 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_197) {
            runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_195 => frame_21195 = runtime.vdp_state().cloned(),
                21_196 => {
                    frame_21196 = runtime.vdp_state().cloned();
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1B43A loc_1B45A restores the saved palette, reloads title
        // art, then allocates loc_1B6AC and loc_1B5CE. Their one-frame
        // hibernation loops advance the Chameleon plane reveal and Kid/title
        // sprites on every presented frame. Oracle frames 21195 and 21196
        // are consecutive distinct images inside that live worker interval.
        assert_ne!(
            frame_21195, frame_21196,
            "loc_1B6AC/loc_1B5CE must not freeze the rebuilt title VDP before the trace-facing title pose becomes ready",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_exposes_title_pose_on_frame_21224() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_21223 = None;
        let mut frame_21224 = None;
        let mut frame_21225 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_226) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_223 => {
                    frame_21223 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_224 => {
                    frame_21224 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_225 => {
                    frame_21225 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B5CE is the worker that finally writes Kid's visible title
        // pose during the mode-$28 rebuild: `move.w #$2E,x_pos(a3)` /
        // `move.w #$8C,y_pos(a3)`. Oracle keeps Kid hidden through local
        // mode-$28 frame 329 and only exposes that pose on frame 330.
        assert_eq!(
            frame_21223,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(329),
                None,
                None
            )),
            "frame 21223 should still keep Kid hidden one frame before loc_1B5CE exposes the rebuilt title pose",
        );
        assert_eq!(
            frame_21224,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(330),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21224 should be the first mode-$28 frame where loc_1B5CE's rebuilt title pose becomes visible",
        );
        assert_eq!(
            frame_21225,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(331),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21225 should keep the rebuilt mode-$28 title pose visible on the following frame",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_stays_out_of_demo_play_on_frame_21718() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_21717 = None;
        let mut frame_21718 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_719) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_717 => {
                    frame_21717 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_718 => {
                    frame_21718 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 `loc_1BAB0` first burns a `$B8` DBF-backed
        // pre-roll before Obj_TitleMenu `loc_1BAEA` even seeds the shared `$280`
        // title idle loop. Oracle stays in mode `$28` through local frame 824 and
        // only diverges after that clean edge.
        assert_eq!(
            frame_21717,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(823),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21717 should still keep the rebuilt mode-$28 title pose visible one frame before the new clean frontier",
        );
        assert_eq!(
            frame_21718,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(824),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21718 should still stay in mode $28 after Load_IntroSequence6's `loc_1BAB0` pre-roll and Obj_TitleMenu's shared title idle countdown",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_stays_out_of_demo_play_on_frame_21719() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_21718 = None;
        let mut frame_21719 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_720) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_718 => {
                    frame_21718 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_719 => {
                    frame_21719 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 `loc_1BAB0` burns the separate `$B8` pre-roll,
        // then `loc_1BAEA: move.w #$280,d0` feeds a DBF-backed idle loop. Because
        // DBF counts the zero slot too, oracle still stays in mode `$28` on local
        // frame 825 and only reaches `loc_1BB02` on the following frame.
        assert_eq!(
            frame_21718,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(824),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21718 should still be the final already-clean mode-$28 title frame",
        );
        assert_eq!(
            frame_21719,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(825),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21719 should still stay in mode $28 because Obj_TitleMenu loc_1BAEA's DBF loop has one more hibernated frame before loc_1BB02",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_stays_out_of_demo_play_on_frame_21764() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_21763 = None;
        let mut frame_21764 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_765) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_763 => {
                    frame_21763 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_764 => {
                    frame_21764 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BAEA -> loc_1BAEE` supplies the raw `$280+1`
        // DBF idle, and the timeout-restart IntroSequence6 rebuild carries the
        // central `loc_1B37C -> loc_1B3BC` lightning loop's extra 45 inclusive
        // passes before `loc_1BB02` flips back into demo play.
        assert_eq!(
            frame_21763,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(869),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21763 should still keep the rebuilt mode-$28 title pose visible one frame before the loc_1B3BC-extended hand-off",
        );
        assert_eq!(
            frame_21764,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(870),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21764 should still stay in mode $28 on the last loc_1B3BC-extended title-pose frame before the later demo flip",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_enters_demo_play_on_frame_21765() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_21764 = None;
        let mut frame_21765 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_766) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_764 => {
                    frame_21764 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_765 => {
                    frame_21765 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BAEA -> loc_1BAEE` plus the IntroSequence6
        // lightning `loc_1B37C -> loc_1B3BC` loop keep mode `$28` through the
        // previous frame; `loc_1BB02` is the first demo-play frame.
        assert_eq!(
            frame_21764,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(870),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21764 should still be the final mode-$28 title-pose frame before the later demo flip",
        );
        assert_eq!(
            frame_21765,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(0),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 21765 should be the first demo-play frame after the oracle-backed timeout-restart title hold ends",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_uses_demo_1_hidden_camera_wrap_on_frame_21825() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_21824 = None;
        let mut frame_21825 = None;
        let mut frame_21826 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_827) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_824 => {
                    frame_21824 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_825 => {
                    frame_21825 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_826 => {
                    frame_21826 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1BA5C enters Load_IntroSequence6, and the later Obj_TitleMenu
        // `loc_1BB02 -> Load_DemoPlay -> Load_InGame` hand-off preserves a
        // timeout-restart-specific hidden demo-1 wrap before the warm-up settles
        // back to the usual `$00E0_0000` row.
        assert_eq!(
            frame_21824,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(59),
                0x00E0_0000,
                None
            )),
            "frame 21824 should still be the hidden Knights Isle warm-up row before the repaired wrap",
        );
        assert_eq!(
            frame_21825,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(60),
                0xFD20_0000,
                None
            )),
            "frame 21825 should expose the oracle-backed IntroSequence6 hidden camera wrap",
        );
        assert_eq!(
            frame_21826,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(61),
                0x00E0_0000,
                None
            )),
            "frame 21826 should settle back to the usual hidden Knights Isle warm-up row",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_demo_1_counts_follow_loc_119d8_load_ingame_passes() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let expected = [
            (21_823, 1, 0),
            (21_824, 2, 0),
            (21_825, 2, 0),
            (21_853, 7, 5),
            (21_854, 7, 5),
            (21_855, 6, 5),
            (22_017, 6, 5),
            (22_018, 6, 5),
            (22_019, 6, 5),
        ];
        let mut actual = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(22_020) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if expected
                .iter()
                .any(|(expected_frame, _, _)| *expected_frame == frame_index)
            {
                actual.push((frame_index, trace.number_objects, trace.number_gfx_objects));
            }
        }

        // ASM Obj_TitleMenu loc_1BB02 -> Load_DemoPlay -> loc_119D8 ->
        // Load_InGame retains the hidden controller and bootstrap workers in
        // Object-pass order. Raw oracle WRAM pins the three shifted edges.
        assert_eq!(actual, expected);
    }

    #[test]
    fn published_tas_intro_sequence_6_keeps_demo_1_bootstrap_hidden_on_frame_21837() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_21837 = None;
        let mut frame_21838 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(21_839) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                21_837 => {
                    frame_21837 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                21_838 => {
                    frame_21838 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM `loc_1BB02 -> Load_DemoPlay -> Load_InGame` has already started the
        // hidden Knights Isle bootstrap here, but the timeout-restart
        // `IntroSequence6` lineage keeps frame 21837 on the `$00E0_0000` hidden
        // row before exposing the already-advanced bootstrap pose on frame 21838.
        assert_eq!(
            frame_21837,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(72),
                0x00E0_0000,
                None,
                None
            )),
            "frame 21837 should keep the IntroSequence6 demo-1 bootstrap hidden for one extra frame",
        );
        assert_eq!(
            frame_21838,
            Some((
                GameMode::DemoPlay,
                LevelId(0x15),
                Some(73),
                0x00D8_0000,
                Some(0x0010_0000),
                Some(0x00FF_0000),
            )),
            "frame 21838 should expose the already-advanced demo bootstrap pose after the hidden hold",
        );
    }

    #[test]
    fn published_tas_intro_6_demo_1_shifts_later_dragon_count_edges_loc_368a8_3691e() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(22_297) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(
                frame_index,
                22_274 | 22_275 | 22_276 | 22_277 | 22_293 | 22_294 | 22_295 | 22_296
            ) {
                samples.push((
                    frame_index,
                    trace.boot_phase_elapsed,
                    trace.camera_x_pos,
                    trace.camera_y_pos,
                    trace.number_objects,
                    trace.number_gfx_objects,
                ));
            }
        }

        // ASM Mode_Level loc_80C calls Manage_EnemyLoading after Execute_Objects.
        // loc_368A8 admits each later Knights Isle Dragon at the camera-window
        // boundary; loc_3691E links its Object on that pass, then
        // Enemy0D_Dragon_Init loc_3CD46 allocates its GfxObject on the next.
        assert_eq!(
            samples,
            vec![
                (22_274, Some(509), 0x019A_0000, 0x00B6_0000, 6, 5),
                (22_275, Some(510), 0x019E_0000, 0x00B7_0000, 6, 5),
                (22_276, Some(511), 0x01A1_0000, 0x00B9_0000, 7, 5),
                (22_277, Some(512), 0x01A5_0000, 0x00BB_0000, 7, 6),
                (22_293, Some(528), 0x01C6_0000, 0x00F9_0000, 7, 6),
                (22_294, Some(529), 0x01C7_0000, 0x00FE_0000, 7, 6),
                (22_295, Some(530), 0x01C8_0000, 0x0102_0000, 8, 6),
                (22_296, Some(531), 0x01C9_0000, 0x0106_0000, 8, 7),
            ]
        );
    }

    #[test]
    fn published_tas_intro_6_demo_1_start_return_preserves_loc_119d8_counts() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(22_585) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(frame_index, 22_551 | 22_552 | 22_583 | 22_584) {
                samples.push((
                    frame_index,
                    trace.game_mode,
                    trace.boot_phase_elapsed,
                    trace.number_objects,
                    trace.number_gfx_objects,
                ));
            }
        }

        // ASM loc_119D8 writes Game_Mode=$2C and jumps to loc_6E2 without
        // deleting its Object/Gfx pair. The terminal 9/8 Mode_Level sample is
        // therefore retained for the 32-frame manual title snapshot, then the
        // hidden title rebuild clears both linked lists.
        assert_eq!(
            samples,
            vec![
                (22_551, GameMode::DemoPlay, Some(703), 9, 8),
                (22_552, GameMode::TitleScreen, Some(0), 9, 8),
                (22_583, GameMode::TitleScreen, Some(31), 9, 8),
                (22_584, GameMode::TitleScreen, Some(32), 0, 0),
            ]
        );
    }

    #[test]
    fn published_tas_boot_title_exposes_obj_title_text_kid_bursts_on_frames_289_and_299() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_287 = None;
        let mut frame_288 = None;
        let mut frame_289 = None;
        let mut frame_298_gfx_objects = None;
        let mut frame_299_gfx_objects = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(300) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                287 => {
                    frame_287 = Some((
                        trace.game_mode,
                        trace.title_screen_ready,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                288 => {
                    frame_288 = Some((
                        trace.game_mode,
                        trace.title_screen_ready,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                289 => {
                    frame_289 = Some((
                        trace.game_mode,
                        trace.title_screen_ready,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.number_gfx_objects,
                    ));
                }
                298 => frame_298_gfx_objects = Some(trace.number_gfx_objects),
                299 => {
                    frame_299_gfx_objects = Some(trace.number_gfx_objects);
                    break;
                }
                _ => {}
            }
        }

        assert_eq!(
            frame_287,
            Some((GameMode::TitleScreen, false, None, None)),
            "the published TAS still keeps Kid hidden on frame 287 even though title mode is active"
        );
        assert_eq!(
            frame_288,
            Some((GameMode::TitleScreen, false, None, None)),
            "the published TAS keeps the title pose hidden for one more frame after 287"
        );
        assert_eq!(
            frame_289,
            Some((
                GameMode::TitleScreen,
                true,
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
                3,
            )),
            "ASM Obj_TitleTextKid -> sub_1B652 restores the title pose and allocates its first popup on frame 289"
        );
        assert_eq!(
            frame_298_gfx_objects,
            Some(3),
            "ASM loc_1B63C keeps only the first sub_1B652 popup through frame 298"
        );
        assert_eq!(
            frame_299_gfx_objects,
            Some(4),
            "ASM loc_1B63C -> sub_1B652 allocates the second title-text popup on frame 299"
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_timeout_restore_reuses_title_pose_on_frames_22700_and_22701()
    {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_22699 = None;
        let mut frame_22700 = None;
        let mut frame_22701 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(22_702) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                22_699 => {
                    frame_22699 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                }
                22_700 => {
                    frame_22700 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                }
                22_701 => {
                    frame_22701 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_facing,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B5CE has already restored the standard title pose before the
        // later IntroSequence6 demo enters gameplay. When the published TAS sends
        // Start back to title on that lineage, oracle briefly re-exposes the
        // pre-existing title pose before the later title helper resumes.
        let visible_title_pose = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
            Some(Direction::Right),
        ));
        assert_eq!(
            frame_22699,
            Some((GameMode::TitleScreen, LevelId(0x00), None, None, None)),
            "frame 22699 should still be the last hidden title-rebuild frame before the restored title pose appears",
        );
        assert_eq!(
            frame_22700, visible_title_pose,
            "frame 22700 should still expose loc_1B5CE's standard title pose on the first timeout-return title frame",
        );
        assert_eq!(
            frame_22701, visible_title_pose,
            "frame 22701 should keep the same title pose for one more oracle-backed frame",
        );
    }

    #[test]
    fn published_tas_intro_sequence_6_timeout_restore_reenters_demo_play_on_frame_23248() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_23247 = None;
        let mut frame_23248 = None;
        let mut frame_23249 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(23_250) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                23_247 => {
                    frame_23247 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                23_248 => {
                    frame_23248 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                23_249 => {
                    frame_23249 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BAEA` seeds the shared `$280` DBF-backed title
        // idle loop, and `loc_1BB02` is the only site here that writes
        // `Game_Mode=$10`. Because loc_1B5CE had already restored the standard
        // title pose before this later IntroSequence6 demo ever started, oracle
        // reaches that demo-play flip two title frames earlier than the generic
        // demo-timeout return.
        let title_pose = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        let demo_pose = Some((
            GameMode::DemoPlay,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        assert_eq!(
            frame_23247, title_pose,
            "frame 23247 should still be the final title-mode frame before Obj_TitleMenu reaches loc_1BB02",
        );
        assert_eq!(
            frame_23248, demo_pose,
            "frame 23248 should be the first demo-play frame on the later IntroSequence6 timeout-return lineage",
        );
        assert_eq!(
            frame_23249, demo_pose,
            "frame 23249 should remain in demo playback after the earlier loc_1BB02 hand-off",
        );
    }

    #[test]
    fn published_tas_timeout_restore_demo_2_uses_timeout_preload_row_on_frame_23305() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_23279 = None;
        let mut frame_23304 = None;
        let mut frame_23305 = None;
        let mut frame_23306 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(23_307) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                23_279 => {
                    frame_23279 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                    ));
                }
                23_304 => {
                    frame_23304 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                23_305 => {
                    frame_23305 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                23_306 => {
                    frame_23306 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BB02 -> Load_DemoPlay -> Load_InGame` still
        // toggles `Demo_Num` on the earlier later-demo that starts immediately
        // after the IntroSequence6 timeout title restore. Oracle shows the
        // Skydragon level ID as soon as the demo startup reaches
        // `DEMO_LEVEL_SWITCH_FRAME`, and this earlier timeout-restart path uses
        // the same lower `$0220_0000` preload row as the direct
        // intro-sequence-5 title return. The later IntroSequence6 revisit
        // without the hidden-camera carry uses the plain `$0228_0000` row.
        assert_eq!(
            frame_23279,
            Some((GameMode::DemoPlay, Some(31), LevelId(0x1B))),
            "frame 23279 should switch to the oracle-backed Skydragon demo level",
        );
        assert_eq!(
            frame_23304,
            Some((GameMode::DemoPlay, Some(56), LevelId(0x1B), 0x0000_0000)),
            "frame 23304 should still be on the blank hidden timeout-restore demo-2 bootstrap row",
        );
        assert_eq!(
            frame_23305,
            Some((
                GameMode::DemoPlay,
                Some(demo::DEMO_CAMERA_PRELOAD_FRAME as usize),
                LevelId(0x1B),
                demo::DEMO_2_INTRO_SEQUENCE_5_TITLE_RETURN_CAMERA_PRELOAD_Y_FP,
            )),
            "frame 23305 should keep the timeout-restart demo-2 preload row on the earlier timeout-restore lineage",
        );
        assert_eq!(
            frame_23306,
            Some((GameMode::DemoPlay, Some(58), LevelId(0x1B), 0x02A0_0000)),
            "frame 23306 should return to Load_InGame's respawn camera snap",
        );
    }

    #[test]
    fn published_tas_timeout_restore_demo_1_uses_cold_boot_wrap_on_frame_24563() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_24562 = None;
        let mut frame_24563 = None;
        let mut frame_24564 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(24_565) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                24_562 => {
                    frame_24562 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                24_563 => {
                    frame_24563 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                24_564 => {
                    frame_24564 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM `loc_119D8 -> loc_6E2 -> Obj_TitleMenu loc_1BB02 ->
        // Load_DemoPlay -> Load_InGame` returns from the later Skydragon demo
        // after loc_1B5CE has already restored the title pose. That lineage
        // still toggles back to Knights Isle, but oracle uses the cold-boot
        // demo-1 wrap (`$FF00_0000`) rather than the generic demo-title return
        // wrap (`$FF20_0000`) on startup frame 60.
        assert_eq!(
            frame_24562,
            Some((GameMode::DemoPlay, Some(59), LevelId(0x15), 0x00E0_0000)),
            "frame 24562 should still be the hidden Knights Isle warm-up row",
        );
        assert_eq!(
            frame_24563,
            Some((GameMode::DemoPlay, Some(60), LevelId(0x15), 0xFF00_0000)),
            "frame 24563 should use the oracle-backed cold-boot demo-1 camera wrap",
        );
        assert_eq!(
            frame_24564,
            Some((GameMode::DemoPlay, Some(61), LevelId(0x15), 0x00E0_0000)),
            "frame 24564 should settle back to the hidden warm-up row",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_5_revisit_exposes_loc_1bcd0_slot_2_on_frame_27603() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_27602 = None;
        let mut frame_27603 = None;
        let mut frame_27604 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(27_605) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                27_602 => {
                    frame_27602 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                27_603 => {
                    frame_27603 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                27_604 => {
                    frame_27604 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence5 allocates `loc_1BCD0`, whose third gfx slot
        // writes `move.w #0,$1A(a1)` / `move.w #$E0,$1E(a1)`. After the later
        // IntroSequence6 timeout-return demo hand-off, the next timeout-restart
        // revisit still follows the same start-skip lineage, so oracle exposes
        // that slot-2 pose one frame earlier than the plain timeout-restart copy.
        assert_eq!(
            frame_27602,
            Some((GameMode::IntroSequence5, LevelId(0x00), None, None)),
            "frame 27602 should still keep the later timeout-restart intro-sequence-5 slot-2 pose hidden",
        );
        assert_eq!(
            frame_27603,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 27603 should expose loc_1BCD0's slot-2 pose on the later timeout-restart revisit",
        );
        assert_eq!(
            frame_27604,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 27604 should keep the same loc_1BCD0 slot-2 pose latched on the following frame",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_5_revisit_enters_intro_sequence_6_on_frame_28066() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_28065 = None;
        let mut frame_28066 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_067) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                28_065 => {
                    frame_28065 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                28_066 => {
                    frame_28066 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1BA28 hands intro sequence 5 to the shared fade helper, then
        // loc_1BA5C owns the later `move.w #$28,(Game_Mode).w` timeout edge.
        // The later intro-sequence-6 title-restore lineage still inherits the
        // one-frame-earlier loc_1BCD0 slot-2 visibility, but it must not also
        // reuse the earlier Start-skip lineage's extra intro-sequence-5 delay.
        assert_eq!(
            frame_28065,
            Some((
                GameMode::IntroSequence5,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 28065 should still be the final intro-sequence-5 timeout frame before loc_1BA5C flips to mode $28 on the later revisit",
        );
        assert_eq!(
            frame_28066,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(INTRO_SEQUENCE_5_SLOT_2_X_POS),
                Some(INTRO_SEQUENCE_5_SLOT_2_Y_POS),
            )),
            "frame 28066 should enter Load_IntroSequence6 while preserving loc_1BCD0's carried slot-2 pose on the later revisit",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_tracks_load_intro_sequence_6_counts() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let sample_frames = [
            28_065, 28_066, 28_087, 28_088, 28_100, 28_101, 28_263, 28_264, 28_304, 28_305, 28_306,
            28_331, 28_332, 28_335, 28_336, 28_337, 28_398, 28_399, 28_405, 28_406, 28_407, 28_408,
            28_409,
        ];
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_410) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if sample_frames.contains(&frame_index) {
                samples.push((frame_index, trace.number_objects, trace.number_gfx_objects));
            }
        }

        // ASM Load_IntroSequence6 clears the inherited loc_1BCD0 lists, then
        // allocates loc_1BAB0, loc_1B37C, and sub_1B43A. Later boundaries are
        // owned by loc_1B5CE, loc_1B6AC, loc_1BB10, sub_1B652, and loc_1C7A0.
        // Raw oracle WRAM pins both sides of every Object/Gfx list transition.
        assert_eq!(
            samples,
            vec![
                (28_065, 4, 5),
                (28_066, 4, 5),
                (28_087, 4, 5),
                (28_088, 0, 0),
                (28_100, 0, 0),
                (28_101, 3, 0),
                (28_263, 3, 0),
                (28_264, 2, 0),
                (28_304, 2, 0),
                (28_305, 4, 0),
                (28_306, 4, 1),
                (28_331, 4, 1),
                (28_332, 3, 1),
                (28_335, 3, 1),
                (28_336, 4, 1),
                (28_337, 4, 2),
                (28_398, 4, 2),
                (28_399, 4, 3),
                (28_405, 4, 3),
                (28_406, 5, 3),
                (28_407, 5, 3),
                (28_408, 4, 3),
                (28_409, 4, 4),
            ]
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_menu_keeps_loc_1bb52_cursor_gfx() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(29_437) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(frame_index, 28_763 | 28_764 | 29_435 | 29_436) {
                samples.push((
                    frame_index,
                    trace.game_mode,
                    trace.number_objects,
                    trace.number_gfx_objects,
                ));
            }
        }

        // ASM Obj_TitleMenu loc_1BB52 allocates the `$FFFF2578` cursor
        // GfxObject owned by `$FFFF0DA0`. Raw oracle linked lists show it
        // joining the settled 4/4 title shell on frame 28764 and remaining
        // live until loc_1BB02 hands the next demo to loc_6E2.
        assert_eq!(
            samples,
            vec![
                (28_763, GameMode::IntroSequence6, 4, 4),
                (28_764, GameMode::IntroSequence6, 4, 5),
                (29_435, GameMode::DemoPlay, 4, 5),
                (29_436, GameMode::DemoPlay, 1, 0),
            ]
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_menu_streams_loc_1bb52_on_frame_28764() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_766) {
            runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(frame_index, 28_763..=28_765) {
                let vdp = runtime
                    .vdp_state()
                    .expect("later IntroSequence6 menu frames should expose a VDP shell");
                let menu_word = u16::from_be_bytes([vdp.vram[0x0208], vdp.vram[0x0209]]);
                let raster = vdp.raster_frame.as_ref().map(|raster| {
                    let initial_menu_word = u16::from_be_bytes([
                        raster.initial_vram[0x0208],
                        raster.initial_vram[0x0209],
                    ]);
                    let first_write = raster
                        .vram_word_writes
                        .first()
                        .map(|write| (write.scanline, write.pixel, write.byte_addr, write.value));
                    (initial_menu_word, first_write)
                });
                samples.push((frame_index, menu_word, raster));
            }
        }

        // ASM Obj_TitleMenu loc_1BB52 -> sub_1C5D0 writes the 13x5 menu map
        // after the frame-28762 Start edge. Oracle FIFO tracing places its
        // first duplicated `$451B` commit on frame 28764 at line 81 pixel 220.
        // That is below the menu's final scanline, so frame 28764 renders from
        // the old `$20BC` map while retaining the completed menu in final VRAM.
        assert_eq!(
            samples,
            vec![
                (28_763, 0x20BC, None),
                (
                    28_764,
                    0x451B,
                    Some((0x20BC, Some((81, 220, 0x0208, 0x451B))))
                ),
                (28_765, 0x451B, None),
            ],
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_keeps_camera_cleared_on_frame_28098() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_28097 = None;
        let mut frame_28098 = None;
        let mut frame_28099 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_100) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                28_097 => {
                    frame_28097 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_098 => {
                    frame_28098 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_099 => {
                    frame_28099 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 still seeds `($FFFFA656).w` and calls
        // `sub_1B222`, but on this later title-restore revisit oracle does not
        // reuse the earlier timeout-restart hidden camera wrap. The stale
        // loc_1BCD0 slot-2 pose survives, while `Camera_Y_pos` stays cleared
        // through local mode-$28 frame 33 instead of jumping to `$FFF0_0000`.
        assert_eq!(
            frame_28097,
            Some((GameMode::IntroSequence6, LevelId(0x00), Some(31), 0)),
            "frame 28097 should still expose the cleared camera on the later IntroSequence6 revisit",
        );
        assert_eq!(
            frame_28098,
            Some((GameMode::IntroSequence6, LevelId(0x00), Some(32), 0)),
            "frame 28098 should keep Camera_Y_pos cleared instead of reusing the earlier hidden wrap",
        );
        assert_eq!(
            frame_28099,
            Some((GameMode::IntroSequence6, LevelId(0x00), Some(33), 0)),
            "frame 28099 should still keep the later IntroSequence6 camera cleared on the following frame",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_uses_delayed_seeded_camera_row_on_frame_28101()
    {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_28100 = None;
        let mut frame_28101 = None;
        let mut frame_28102 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_103) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                28_100 => {
                    frame_28100 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_101 => {
                    frame_28101 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_102 => {
                    frame_28102 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Load_IntroSequence6 still seeds `($FFFFA656).w` and `sub_1B222` on
        // the later title-restore revisit, but unlike the earlier timeout path it
        // never exposes the wrapped `$FFF0_0000` row here. Oracle RAM probes keep
        // `Camera_Y_pos` cleared through local mode-$28 frame 34, then switch to
        // the delayed seeded `$FFD00000` row on frame 35.
        assert_eq!(
            frame_28100,
            Some((GameMode::IntroSequence6, LevelId(0x00), Some(34), 0)),
            "frame 28100 should still keep the later IntroSequence6 camera cleared before the delayed seed appears",
        );
        assert_eq!(
            frame_28101,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(35),
                0xFFD0_0000
            )),
            "frame 28101 should expose the delayed `sub_1B222` seeded camera row on the later revisit",
        );
        assert_eq!(
            frame_28102,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(36),
                0xFFD0_0000
            )),
            "frame 28102 should keep the delayed seeded camera row latched on the next frame",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_starts_delayed_camera_ramp_on_frame_28317() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_28316 = None;
        let mut frame_28317 = None;
        let mut frame_28318 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_319) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                28_316 => {
                    frame_28316 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_317 => {
                    frame_28317 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                }
                28_318 => {
                    frame_28318 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM GfxObjects_Collision_BottomBoundary4 snapshots
        // `d1 = ($FFFFA656).w >> 4` once, then repeats
        // `sub.l d1,($FFFFA656).w; bsr.w sub_1B222` for 16 frames. On the later
        // revisit `($FFFFA656).w` is still `$FFD00000`, so `d1=$FFFD0000` and the
        // hidden camera rises by 3 pixels per frame from local mode-$28 frame 251.
        assert_eq!(
            frame_28316,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(250),
                0xFFD0_0000
            )),
            "frame 28316 should still keep the delayed seeded camera flat before the later ramp starts",
        );
        assert_eq!(
            frame_28317,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(251),
                0xFFD3_0000
            )),
            "frame 28317 should start the later IntroSequence6 +3px hidden camera ramp",
        );
        assert_eq!(
            frame_28318,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(252),
                0xFFD6_0000
            )),
            "frame 28318 should continue the later IntroSequence6 +3px hidden camera ramp",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_keeps_title_pose_hidden_through_frame_28398() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_28396 = None;
        let mut frame_28398 = None;
        let mut frame_28399 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(28_400) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                28_396 => {
                    frame_28396 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                28_398 => {
                    frame_28398 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                28_399 => {
                    frame_28399 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B5CE still owns the eventual `move.w #$2E,x_pos(a3)` /
        // `move.w #$8C,y_pos(a3)` write on the later mode-$28 revisit, but oracle
        // keeps Kid hidden through local frame 332 after the delayed `$FFD00000`
        // camera-ramp lineage and only exposes the standard title pose on 333.
        assert_eq!(
            frame_28396,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(330),
                None,
                None
            )),
            "frame 28396 should still keep Kid hidden on the later mode-$28 revisit even though the earlier timeout-restart lineage already shows the title pose",
        );
        assert_eq!(
            frame_28398,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(332),
                None,
                None
            )),
            "frame 28398 should still be hidden on the final pre-title-pose frame of the later revisit",
        );
        assert_eq!(
            frame_28399,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(333),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 28399 should be the first later-revisit frame where loc_1B5CE's standard title pose becomes visible",
        );
    }

    #[test]
    fn demo_diamond_prize_has_not_queued_collection_on_frame_1404() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_1404 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_405) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_404 {
                frame_1404 = Some((
                    trace.score,
                    trace.diamond_count,
                    trace
                        .gameplay_debug_fields
                        .get("pending_prize_diamond_awards")
                        .cloned(),
                ));
                break;
            }
        }

        assert_eq!(
            frame_1404,
            Some((0, 0, Some("[]".into()))),
            "ASM PrizeDiamondCollected_Init should not be queued before the diamond collision frame",
        );
    }

    #[test]
    fn demo_diamond_prize_queues_collection_on_frame_1406_without_awarding_score() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_1406 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_407) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_406 {
                frame_1406 = Some((
                    trace.score,
                    trace.diamond_count,
                    trace
                        .gameplay_debug_fields
                        .get("pending_prize_diamond_awards")
                        .cloned(),
                ));
                break;
            }
        }

        assert_eq!(
            frame_1406,
            Some((0, 0, Some("[32]".into()))),
            "ASM PrizeDiamondCollected_Init should be queued on the diamond collision frame before its HUD-flight hibernate loop has counted down",
        );
    }

    #[test]
    fn demo_prize_diamond_awards_on_frame_1438_without_score() {
        let Some((mut runtime, script)) = cached_any_percent_demo_runtime_and_script() else {
            return;
        };
        let mut frame_1438 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(1_439) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if frame_index == 1_438 {
                frame_1438 = Some((trace.score, trace.diamond_count));
                break;
            }
        }

        assert_eq!(
            frame_1438,
            Some((0, 1)),
            "oracle frame 1438 increments the prize diamond count without adding score"
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_keeps_mode_28_through_frame_29404() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_29401 = None;
        let mut frame_29402 = None;
        let mut frame_29404 = None;
        let mut frame_29405 = None;
        let mut frame_29431 = None;
        let mut frame_29432 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(29_433) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                29_401 => {
                    frame_29401 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                29_402 => {
                    frame_29402 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                29_404 => {
                    frame_29404 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                29_405 => {
                    frame_29405 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                        trace.kid_y_velocity,
                    ));
                }
                29_431 => {
                    frame_29431 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.kid_y_velocity,
                    ));
                }
                29_432 => {
                    frame_29432 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.kid_y_velocity,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BAEA: move.w #$280,d0` feeds the shared
        // `loc_1BAEE` DBF-backed idle loop, and only `loc_1BB02` writes
        // `Game_Mode=$10`. On the later title-restore revisit oracle keeps the
        // already-visible title pose in mode `$28` for three extra local frames
        // before that hand-off lands on frame 29405.
        let intro_sequence_6_pose = Some((
            GameMode::IntroSequence6,
            LevelId(0x00),
            Some(1335),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        assert_eq!(
            frame_29401, intro_sequence_6_pose,
            "frame 29401 should still be the settled later IntroSequence6 title pose before the delayed loc_1BB02 hand-off",
        );
        assert_eq!(
            frame_29402,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(1336),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 29402 should remain in mode $28 on the first extra later-lineage idle frame",
        );
        assert_eq!(
            frame_29404,
            Some((
                GameMode::IntroSequence6,
                LevelId(0x00),
                Some(1338),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 29404 should still be the final later-lineage mode-$28 frame before loc_1BB02 flips to demo play",
        );
        assert_eq!(
            frame_29405,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(0),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
                Some(0),
            )),
            "ASM Obj_TitleMenu loc_1BB02 -> Load_DemoPlay should make frame 29405 the first later-lineage demo-play frame without synthesising a Kid velocity write",
        );
        assert_eq!(
            frame_29431,
            Some((
                GameMode::DemoPlay,
                Some(demo::DEMO_2_INTRO_SEQUENCE_6_LATER_CONTROLLER_SAMPLE_END_FRAME as usize),
                Some(0),
            )),
            "frame 29431 should keep the oracle Kid y velocity inert throughout the later IntroSequence6 demo-2 controller window",
        );
        assert_eq!(
            frame_29432,
            Some((
                GameMode::DemoPlay,
                Some(demo::DEMO_2_INTRO_SEQUENCE_6_LATER_CONTROLLER_SAMPLE_END_FRAME as usize + 1),
                None,
            )),
            "frame 29432 should release back to the raw hidden Load_InGame player sample",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_6_revisit_uses_intro_return_demo_2_preload_row_on_frame_29462()
     {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_29461 = None;
        let mut frame_29462 = None;
        let mut frame_29463 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(29_464) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                29_461 => {
                    frame_29461 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                29_462 => {
                    frame_29462 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                }
                29_463 => {
                    frame_29463 = Some((
                        trace.game_mode,
                        trace.boot_phase_elapsed,
                        trace.current_level,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BB02` still reaches `Load_DemoPlay ->
        // Load_InGame`, but on the later IntroSequence6 title-restore revisit
        // oracle keeps the same `$0228_0000` hidden demo-2 preload row as the
        // earlier Start-skip intro-return lineage before `Load_InGame` snaps to
        // `$02A0_0000` on the following frame.
        assert_eq!(
            frame_29461,
            Some((GameMode::DemoPlay, Some(56), LevelId(0x1B), 0x0000_0000)),
            "frame 29461 should still be on the blank hidden later demo-2 bootstrap row",
        );
        assert_eq!(
            frame_29462,
            Some((
                GameMode::DemoPlay,
                Some(demo::DEMO_CAMERA_PRELOAD_FRAME as usize),
                LevelId(0x1B),
                demo::DEMO_2_CAMERA_PRELOAD_Y_FP,
            )),
            "frame 29462 should expose the oracle-backed later IntroSequence6 demo-2 preload row",
        );
        assert_eq!(
            frame_29463,
            Some((GameMode::DemoPlay, Some(58), LevelId(0x1B), 0x02A0_0000)),
            "frame 29463 should return to Load_InGame's respawn camera snap",
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_intro_sequence_5_keeps_camera_handoff_on_frame_32005() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_32004 = None;
        let mut frame_32005 = None;
        let mut frame_32006 = None;
        let mut frame_32007 = None;
        let mut frame_32011 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(32_012) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                32_004 => {
                    frame_32004 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                }
                32_005 => {
                    frame_32005 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                }
                32_006 => {
                    frame_32006 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                }
                32_007 => {
                    frame_32007 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                }
                32_011 => {
                    frame_32011 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1B988 writes `move.w #4,(Camera_Y_pos).w`, and
        // Load_IntroSequence5 only allocates `loc_1BA28` / `loc_1BCD0`, so the
        // later timeout-restart revisit after the repaired demo-2 preload row
        // must still carry that intro-sequence-4 camera into the opening
        // intro-sequence-5 frames. Oracle keeps `Camera_Y_pos=$0004_0000`
        // through frame 32011, even though the scene changes around it.
        assert_eq!(
            frame_32004,
            Some((GameMode::IntroSequence4, LevelId(0x00), 0x0004_0000)),
            "frame 32004 should still be the final intro-sequence-4 frame with sub_1B988's camera hand-off latched",
        );
        assert_eq!(
            frame_32005,
            Some((GameMode::IntroSequence4, LevelId(0x00), 0x0004_0000)),
            "frame 32005 should keep the carried sub_1B988 camera hand-off instead of clearing Camera_Y_pos on the Start-tapped revisit",
        );
        assert_eq!(
            frame_32006,
            Some((GameMode::IntroSequence4, LevelId(0x00), 0x0004_0000)),
            "frame 32006 should still be the deferred sub_1C246 Start-return frame before loc_1C264 flips to title",
        );
        assert_eq!(
            frame_32007,
            Some((GameMode::TitleScreen, LevelId(0x00), 0x0004_0000)),
            "frame 32007 should expose the deferred `sub_1C246 -> Game_Mode=$2C` title return while preserving the carried camera hand-off",
        );
        assert_eq!(
            frame_32011,
            Some((GameMode::TitleScreen, LevelId(0x00), 0x0004_0000)),
            "frame 32011 should still keep the carried intro-sequence-4 camera inside the early title-return window",
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_title_clears_intro_sequence_4_camera_on_frame_32039() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_32038 = None;
        let mut frame_32039 = None;
        let mut frame_32040 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(32_041) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                32_038 => {
                    frame_32038 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                }
                32_039 => {
                    frame_32039 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                }
                32_040 => {
                    frame_32040 = Some((trace.game_mode, trace.current_level, trace.camera_y_pos));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1C246 has already returned this lineage to `Game_Mode=$2C`;
        // the carried `sub_1B988` camera only survives for the brief title
        // rebuild window before oracle restores the cleared title camera on frame
        // 32039.
        assert_eq!(
            frame_32038,
            Some((GameMode::TitleScreen, LevelId(0x00), 0x0004_0000)),
            "frame 32038 should still keep the carried intro-sequence-4 camera on the last oracle-backed title-settle frame",
        );
        assert_eq!(
            frame_32039,
            Some((GameMode::TitleScreen, LevelId(0x00), 0x0000_0000)),
            "frame 32039 should clear the carried intro-sequence-4 camera once the title rebuild settles",
        );
        assert_eq!(
            frame_32040,
            Some((GameMode::TitleScreen, LevelId(0x00), 0x0000_0000)),
            "frame 32040 should stay on the cleared title camera after the carry window ends",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_4_title_restore_stages_load_title_screen_loc_1b258() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_32006 = None;
        let mut frame_32007 = None;
        let mut frame_32010 = None;
        let mut frame_32011 = None;
        let mut frame_32024 = None;
        let mut frame_32025 = None;
        let mut frame_32062 = None;
        let mut frame_32063 = None;
        let mut frame_32064 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(32_065) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                32_006 => frame_32006 = runtime.vdp_state().cloned(),
                32_007 => frame_32007 = runtime.vdp_state().cloned(),
                32_010 => frame_32010 = runtime.vdp_state().cloned(),
                32_011 => frame_32011 = runtime.vdp_state().cloned(),
                32_024 => frame_32024 = runtime.vdp_state().cloned(),
                32_025 => frame_32025 = runtime.vdp_state().cloned(),
                32_062 => frame_32062 = runtime.vdp_state().cloned(),
                32_063 => {
                    frame_32063 = Some((
                        trace.boot_phase,
                        trace.boot_phase_elapsed,
                        runtime.vdp_state().cloned(),
                    ));
                }
                32_064 => {
                    frame_32064 = Some((
                        trace.boot_phase,
                        trace.boot_phase_elapsed,
                        runtime.vdp_state().cloned(),
                    ));
                }
                _ => {}
            }
        }

        let title_data = runtime
            .title_screen_data
            .as_ref()
            .expect("cached generated manifest should contain title-screen data");
        let lineage = vdp::TitleScreenVdpLineage {
            intro_return_hidden_shell: true,
            fresh_title_worker_lineage: true,
            ..vdp::TitleScreenVdpLineage::default()
        };
        let expected_loader_frame =
            vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                title_data,
                7,
                runtime.title_screen_region_byte,
                None,
                false,
                57,
                lineage,
            );
        let expected_first_object_pass =
            vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                title_data,
                7,
                runtime.title_screen_region_byte,
                None,
                true,
                58,
                lineage,
            );

        // ASM sub_1B988 -> sub_1C246 -> loc_1C264 -> loc_6E2 first keeps the
        // completed IntroSequence4 VDP resident while Pal_FadeOut runs. Oracle
        // holds frames 32007..32010, fades frames 32011..32024, then clears
        // VRAM on frame 32025 before the blocking title loader. Frame 32063
        // presents Load_TitleScreen's base upload, and frame 32064 exposes the
        // first fresh title-object pass.
        let resident = frame_32006.expect("frame 32006 resident IntroSequence4 VDP");
        assert_eq!(frame_32007.as_ref(), Some(&resident));
        assert_eq!(frame_32010.as_ref(), Some(&resident));
        let first_fade = frame_32011.expect("frame 32011 first Pal_FadeOut step");
        assert_eq!(first_fade.vram, resident.vram);
        assert_ne!(first_fade.cram, resident.cram);
        assert!(
            frame_32024
                .as_ref()
                .is_some_and(|vdp| vdp.cram.iter().any(|byte| *byte != 0))
        );
        assert_eq!(frame_32025, Some(vdp::blank_vdp_state()));
        assert_eq!(frame_32062, Some(vdp::blank_vdp_state()));
        assert_eq!(
            frame_32063,
            Some((
                Some(String::from("title_screen")),
                Some(56),
                Some(expected_loader_frame),
            )),
        );
        assert_eq!(
            frame_32064,
            Some((
                Some(String::from("title_screen")),
                Some(57),
                Some(expected_first_object_pass),
            )),
        );
    }

    #[test]
    fn published_tas_timeout_restart_intro_sequence_3_replays_sub_1b93e_palette_shell() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_5952 = None;
        let mut frame_5953 = None;
        let mut frame_5993 = None;
        let mut frame_5994 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(5_995) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                5_952 => frame_5952 = runtime.vdp_state().cloned(),
                5_953 => {
                    frame_5953 = Some((
                        trace.boot_phase,
                        trace.boot_phase_elapsed,
                        runtime.vdp_state().cloned(),
                    ));
                }
                5_993 => frame_5993 = runtime.vdp_state().cloned(),
                5_994 => {
                    frame_5994 = Some((
                        trace.boot_phase,
                        trace.boot_phase_elapsed,
                        runtime.vdp_state().cloned(),
                    ));
                }
                _ => {}
            }
        }

        let resident = frame_5952.expect("frame 5952 should retain IntroSequence2 VDP");
        // ASM Load_IntroSequence3 -> sub_1B93E -> sub_1C204 presents the
        // resident IntroSequence2 image once at the mode handoff, fades it to
        // black, then keeps the rebuilt scene black past loc_1AEEC's first
        // trace-visible roster frame. Oracle first starts fading the new scene
        // in on frame 5998.
        assert_eq!(
            frame_5953,
            Some((
                Some(String::from("intro_sequence_3")),
                Some(0),
                Some(resident),
            )),
        );
        assert!(
            frame_5993
                .as_ref()
                .is_some_and(|vdp| vdp.cram.iter().all(|byte| *byte == 0)),
            "the rebuilt scene should remain hidden under black CRAM on frame 5993",
        );
        let (phase, elapsed, frame_5994_vdp) =
            frame_5994.expect("frame 5994 should be IntroSequence3 elapsed 41");
        assert_eq!(phase.as_deref(), Some("intro_sequence_3"));
        assert_eq!(elapsed, Some(41));
        assert!(
            frame_5994_vdp
                .as_ref()
                .is_some_and(|vdp| vdp.cram.iter().all(|byte| *byte == 0)),
            "loc_1AEEC's first roster pass should still render under black CRAM",
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_4_title_return_rebuild_counts_match_loc_1b258() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(32_167) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(
                frame_index,
                32_007
                    | 32_038
                    | 32_039
                    | 32_061
                    | 32_062
                    | 32_063
                    | 32_064
                    | 32_065
                    | 32_066
                    | 32_155
                    | 32_156
                    | 32_165
                    | 32_166
            ) {
                samples.push((
                    frame_index,
                    trace.game_mode,
                    trace.number_objects,
                    trace.number_gfx_objects,
                ));
            }
        }

        // ASM sub_1B988 -> sub_1C246 -> loc_1C264 enters Load_TitleScreen.
        // Raw oracle WRAM shows the two carried Object workers have no linked
        // GfxObject, loc_6E2 clears both lists on frame 32039, then
        // Load_TitleScreen loc_1B258 allocates its three workers before the
        // following Obj_TitleMenu passes add loc_1BB10 and loc_1C7A0.
        assert_eq!(
            samples,
            vec![
                (32_007, GameMode::TitleScreen, 2, 0),
                (32_038, GameMode::TitleScreen, 2, 0),
                (32_039, GameMode::TitleScreen, 0, 0),
                (32_061, GameMode::TitleScreen, 0, 0),
                (32_062, GameMode::TitleScreen, 3, 0),
                (32_063, GameMode::TitleScreen, 5, 1),
                (32_064, GameMode::TitleScreen, 5, 2),
                (32_065, GameMode::TitleScreen, 4, 2),
                (32_066, GameMode::TitleScreen, 4, 2),
                (32_155, GameMode::TitleScreen, 4, 2),
                (32_156, GameMode::TitleScreen, 4, 3),
                (32_165, GameMode::TitleScreen, 4, 3),
                (32_166, GameMode::TitleScreen, 4, 4),
            ]
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_title_restores_pose_on_frame_32156() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_32155 = None;
        let mut frame_32156 = None;
        let mut frame_32157 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(32_158) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                32_155 => {
                    frame_32155 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                32_156 => {
                    frame_32156 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                32_157 => {
                    frame_32157 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1C246 has already returned the later intro-sequence-4 revisit to
        // `Game_Mode=$2C`, and oracle-backed TAS traces show the rebuilt title
        // shell exposes the standard `loc_1B5CE` pose on frame 32156 rather than
        // waiting for another generic intro-return rebuild frame.
        assert_eq!(
            frame_32155,
            Some((GameMode::TitleScreen, LevelId(0x00), None, None)),
            "frame 32155 should still be the final hidden title-rebuild frame on the later intro-sequence-4 revisit",
        );
        assert_eq!(
            frame_32156,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 32156 should be the first title frame that re-exposes the standard title pose on the later intro-sequence-4 revisit",
        );
        assert_eq!(
            frame_32157,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 32157 should keep the restored title pose visible on the following frame",
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_title_resumes_demo_play_on_frame_32704() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_32703 = None;
        let mut frame_32704 = None;
        let mut frame_32705 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(32_706) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                32_703 => {
                    frame_32703 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                32_704 => {
                    frame_32704 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                32_705 => {
                    frame_32705 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM Obj_TitleMenu `loc_1BAEA` / `loc_1BB02` own the final `$2C -> $10`
        // attract hand-off after `sub_1C246` has already restored the later
        // intro-sequence-4 title pose. Oracle reaches `loc_1BB02` on frame 32704
        // without moving Kid off the standard title pose first.
        assert_eq!(
            frame_32703,
            Some((
                GameMode::TitleScreen,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 32703 should still be the final settled later intro-sequence-4 title frame",
        );
        assert_eq!(
            frame_32704,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 32704 should be the first demo-play frame after the later intro-sequence-4 title return reaches loc_1BB02",
        );
        assert_eq!(
            frame_32705,
            Some((
                GameMode::DemoPlay,
                LevelId(0x00),
                Some(TITLE_KID_X_POS),
                Some(TITLE_KID_Y_POS),
            )),
            "frame 32705 should stay in demo playback while the hidden bootstrap continues",
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_demo_return_reuses_title_pose_on_frames_34368_and_34369()
    {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_34367 = None;
        let mut frame_34368 = None;
        let mut frame_34369 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(34_370) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                34_367 => {
                    frame_34367 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                34_368 => {
                    frame_34368 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                34_369 => {
                    frame_34369 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1B5CE had already restored the standard title pose before the
        // later intro-sequence-4 title helper reached `loc_1BB02` on frame 32704.
        // When that later demo returns to title, oracle re-exposes the same pose
        // two frames earlier than the generic demo-title rebuild window.
        let hidden_title_pose = Some((GameMode::TitleScreen, LevelId(0x00), None, None));
        let visible_title_pose = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        assert_eq!(
            frame_34367, hidden_title_pose,
            "frame 34367 should still be the last hidden title-rebuild frame before the restored title pose returns",
        );
        assert_eq!(
            frame_34368, visible_title_pose,
            "frame 34368 should expose the already-restored loc_1B5CE title pose on the later intro-sequence-4 demo-return lineage",
        );
        assert_eq!(
            frame_34369, visible_title_pose,
            "frame 34369 should keep the same title pose for one more oracle-backed frame",
        );
    }

    #[test]
    fn published_tas_early_restore_intro_sequence_1_title_counts_follow_raw_lists() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(34_379) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(
                frame_index,
                34_220
                    | 34_249
                    | 34_250
                    | 34_251
                    | 34_252
                    | 34_273
                    | 34_274
                    | 34_275
                    | 34_276
                    | 34_277
                    | 34_367
                    | 34_368
                    | 34_377
                    | 34_378
            ) {
                samples.push((
                    frame_index,
                    trace.game_mode,
                    trace.number_objects,
                    trace.number_gfx_objects,
                ));
            }
        }

        // ASM sub_1B8F4 -> sub_1C246 -> loc_1C264 carries the final
        // IntroSequence1 controller/text Object pair and its GfxObject for 32
        // title traces. Load_TitleScreen then clears and rebuilds both lists;
        // Obj_TitleTextKid loc_1B5E4 -> sub_1B652 links its two popups ten
        // traces apart after the standard title pose returns.
        assert_eq!(
            samples,
            vec![
                (34_220, GameMode::TitleScreen, 2, 1),
                (34_249, GameMode::TitleScreen, 2, 1),
                (34_250, GameMode::TitleScreen, 2, 1),
                (34_251, GameMode::TitleScreen, 2, 1),
                (34_252, GameMode::TitleScreen, 0, 0),
                (34_273, GameMode::TitleScreen, 0, 0),
                (34_274, GameMode::TitleScreen, 3, 0),
                (34_275, GameMode::TitleScreen, 5, 1),
                (34_276, GameMode::TitleScreen, 5, 2),
                (34_277, GameMode::TitleScreen, 4, 2),
                (34_367, GameMode::TitleScreen, 4, 2),
                (34_368, GameMode::TitleScreen, 4, 3),
                (34_377, GameMode::TitleScreen, 4, 3),
                (34_378, GameMode::TitleScreen, 4, 4),
            ]
        );
    }

    #[test]
    fn published_tas_early_restore_intro_sequence_1_stages_load_title_screen_loc_1b258() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut expected_resident_vdp = None;
        let mut observed_fade_vdp = Vec::new();
        let mut frame_34274 = None;
        let mut frame_34275 = None;
        let mut frame_34276 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(34_277) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                34_219 => {
                    let elapsed = trace
                        .boot_phase_elapsed
                        .expect("frame 34219 should expose IntroSequence1 elapsed");
                    expected_resident_vdp =
                        runtime.intro_sub_1b850_data.as_ref().and_then(|data| {
                            runtime.sega_screen_data.as_ref().map(|sega_data| {
                                // ASM sub_1B8F4 loc_1B924 wakes the loc_1C7A0
                                // text worker once more before loc_1C264 enters
                                // loc_6E2's Pal_FadeOut.
                                vdp::build_intro_sequence_1_cold_boot_vdp_at_elapsed(
                                    data,
                                    sega_data,
                                    runtime.title_screen_region_byte,
                                    elapsed + 1,
                                )
                            })
                        });
                }
                34_220 | 34_223 | 34_224 | 34_237 | 34_238 => {
                    observed_fade_vdp.push((frame_index, runtime.vdp_state().cloned()));
                }
                34_274 => frame_34274 = runtime.vdp_state().cloned(),
                34_275 => {
                    frame_34275 = Some((
                        trace.boot_phase,
                        trace.boot_phase_elapsed,
                        runtime.vdp_state().cloned(),
                    ));
                }
                34_276 => {
                    frame_34276 = Some((
                        trace.boot_phase,
                        trace.boot_phase_elapsed,
                        runtime.vdp_state().cloned(),
                    ));
                }
                _ => {}
            }
        }

        let resident_vdp = expected_resident_vdp
            .expect("cached generated manifest should contain cold IntroSequence1 data");
        let faded_resident_vdp = |elapsed| {
            let mut vdp = resident_vdp.clone();
            assert!(vdp::apply_intro_title_return_fade_to_black_loc_28c6(
                &mut vdp, elapsed,
            ));
            vdp
        };
        let expected_fade_vdp = [
            (34_220, resident_vdp.clone()),
            (34_223, resident_vdp.clone()),
            (34_224, faded_resident_vdp(4)),
            (34_237, faded_resident_vdp(17)),
            (34_238, vdp::blank_vdp_state()),
        ];

        for ((observed_frame, observed), (expected_frame, expected)) in
            observed_fade_vdp.iter().zip(expected_fade_vdp.iter())
        {
            assert_eq!(observed_frame, expected_frame);
            assert!(
                observed.as_ref() == Some(expected),
                "ASM loc_1B924 -> loc_1C264 -> loc_6E2 early-restore title VDP diverged at published frame {observed_frame}",
            );
        }

        let title_data = runtime
            .title_screen_data
            .as_ref()
            .expect("cached generated manifest should contain title-screen data");
        let lineage = vdp::TitleScreenVdpLineage {
            intro_return_hidden_shell: true,
            fresh_title_worker_lineage: true,
            ..vdp::TitleScreenVdpLineage::default()
        };
        let expected_loader_frame =
            vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                title_data,
                7,
                runtime.title_screen_region_byte,
                None,
                false,
                57,
                lineage,
            );
        let expected_first_object_pass =
            vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                title_data,
                7,
                runtime.title_screen_region_byte,
                None,
                true,
                58,
                lineage,
            );

        // ASM sub_1B8F4 -> sub_1C246 -> loc_1C264 -> loc_6E2 runs the
        // blocking title loader two outer passes ahead of the generic return.
        // Oracle frame 34274 is its final black hold, frame 34275 presents
        // Load_TitleScreen's base upload, and frame 34276 exposes the first
        // fresh title-object pass.
        assert_eq!(frame_34274, Some(vdp::blank_vdp_state()));
        assert_eq!(
            frame_34275,
            Some((
                Some(String::from("title_screen")),
                Some(55),
                Some(expected_loader_frame),
            )),
        );
        assert_eq!(
            frame_34276,
            Some((
                Some(String::from("title_screen")),
                Some(56),
                Some(expected_first_object_pass),
            )),
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_2_title_restore_stages_load_title_screen_loc_1b258() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_36877 = None;
        let mut frame_36878 = None;
        let mut frame_36879 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(36_880) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                36_877 => frame_36877 = runtime.vdp_state().cloned(),
                36_878 => {
                    frame_36878 = Some((
                        trace.boot_phase,
                        trace.boot_phase_elapsed,
                        runtime.vdp_state().cloned(),
                    ));
                }
                36_879 => {
                    frame_36879 = Some((
                        trace.boot_phase,
                        trace.boot_phase_elapsed,
                        runtime.vdp_state().cloned(),
                    ));
                }
                _ => {}
            }
        }

        let title_data = runtime
            .title_screen_data
            .as_ref()
            .expect("cached generated manifest should contain title-screen data");
        let lineage = vdp::TitleScreenVdpLineage {
            intro_return_hidden_shell: true,
            fresh_title_worker_lineage: true,
            ..vdp::TitleScreenVdpLineage::default()
        };
        let expected_loader_frame =
            vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                title_data,
                7,
                runtime.title_screen_region_byte,
                None,
                false,
                57,
                lineage,
            );
        let expected_first_object_pass =
            vdp::build_title_screen_vdp_with_character_frame_menu_title_objects_and_lineage(
                title_data,
                7,
                runtime.title_screen_region_byte,
                None,
                true,
                58,
                lineage,
            );

        // ASM loc_1B9D0 -> loc_1BA02 -> sub_1C246 -> loc_1C264 -> loc_6E2
        // runs the blocking title loader. Oracle frame 36877 is its final
        // black hold, frame 36878 presents Load_TitleScreen's base upload,
        // and frame 36879 exposes the first fresh title-object pass.
        assert_eq!(frame_36877, Some(vdp::blank_vdp_state()));
        assert_eq!(
            frame_36878,
            Some((
                Some(String::from("title_screen")),
                Some(57),
                Some(expected_loader_frame),
            )),
        );
        assert_eq!(
            frame_36879,
            Some((
                Some(String::from("title_screen")),
                Some(58),
                Some(expected_first_object_pass),
            )),
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_2_title_return_fades_resident_vdp_loc_28c6() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut resident_vdp = None;
        let mut title_frames = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(36_840) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                36_820 => resident_vdp = runtime.vdp_state().cloned(),
                36_821 | 36_825 | 36_838 | 36_839 => title_frames.push((
                    frame_index,
                    trace.boot_phase,
                    trace.boot_phase_elapsed,
                    runtime.vdp_state().cloned(),
                )),
                _ => {}
            }
        }

        let resident_vdp = resident_vdp.expect("frame 36820 has resident IntroSequence2 VDP");
        let expected_title_frames = [0usize, 4, 17]
            .into_iter()
            .map(|elapsed| {
                let mut vdp = resident_vdp.clone();
                assert!(vdp::apply_intro_title_return_fade_to_black_loc_28c6(
                    &mut vdp, elapsed
                ));
                Some(vdp)
            })
            .chain(std::iter::once(Some(vdp::blank_vdp_state())))
            .collect::<Vec<_>>();

        // ASM loc_1B9D0 -> loc_1BA02 -> sub_1C246 -> loc_1C264 -> loc_6E2
        // retains the final IntroSequence2 VDP while Pal_FadeOut loc_28C6
        // steps its CRAM, then clears VRAM after the final fade trace.
        assert_eq!(
            title_frames,
            vec![
                (
                    36_821,
                    Some(String::from("title_screen")),
                    Some(0),
                    expected_title_frames[0].clone(),
                ),
                (
                    36_825,
                    Some(String::from("title_screen")),
                    Some(4),
                    expected_title_frames[1].clone(),
                ),
                (
                    36_838,
                    Some(String::from("title_screen")),
                    Some(17),
                    expected_title_frames[2].clone(),
                ),
                (
                    36_839,
                    Some(String::from("title_screen")),
                    Some(18),
                    expected_title_frames[3].clone(),
                ),
            ],
        );
    }

    #[test]
    fn published_tas_later_intro_sequence_2_title_counts_follow_raw_lists() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut samples = Vec::new();

        for (frame_index, mask) in script.iter().copied().enumerate().take(36_982) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            if matches!(
                frame_index,
                36_821
                    | 36_852
                    | 36_853
                    | 36_876
                    | 36_877
                    | 36_878
                    | 36_879
                    | 36_880
                    | 36_970
                    | 36_971
                    | 36_980
                    | 36_981
            ) {
                samples.push((
                    frame_index,
                    trace.game_mode,
                    trace.number_objects,
                    trace.number_gfx_objects,
                ));
            }
        }

        // ASM loc_1B9D0 -> loc_1BA02 -> sub_1C246 -> loc_1C264 carries two
        // IntroSequence2 Objects without a GfxObject through 32 title traces.
        // Load_TitleScreen then clears and rebuilds both lists, while
        // Obj_TitleTextKid loc_1B5E4 -> sub_1B652 contributes its two later
        // popup GfxObjects ten traces apart.
        assert_eq!(
            samples,
            vec![
                (36_821, GameMode::TitleScreen, 2, 0),
                (36_852, GameMode::TitleScreen, 2, 0),
                (36_853, GameMode::TitleScreen, 0, 0),
                (36_876, GameMode::TitleScreen, 0, 0),
                (36_877, GameMode::TitleScreen, 3, 0),
                (36_878, GameMode::TitleScreen, 5, 1),
                (36_879, GameMode::TitleScreen, 5, 2),
                (36_880, GameMode::TitleScreen, 4, 2),
                (36_970, GameMode::TitleScreen, 4, 2),
                (36_971, GameMode::TitleScreen, 4, 3),
                (36_980, GameMode::TitleScreen, 4, 3),
                (36_981, GameMode::TitleScreen, 4, 4),
            ]
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_intro_sequence_1_restarts_demo_play_on_frame_34916() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_34915 = None;
        let mut frame_34916 = None;
        let mut frame_34917 = None;
        let mut frame_34973 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(34_974) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                34_915 => {
                    frame_34915 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                34_916 => {
                    frame_34916 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                34_917 => {
                    frame_34917 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.kid_x_pos,
                        trace.kid_y_pos,
                    ));
                }
                34_973 => {
                    frame_34973 = Some((
                        trace.game_mode,
                        trace.current_level,
                        trace.boot_phase_elapsed,
                        trace.camera_y_pos,
                    ));
                    break;
                }
                _ => {}
            }
        }

        // ASM sub_1C246 has already returned this later intro-sequence-1
        // Start-skip lineage to `Game_Mode=$2C`, and oracle-backed traces show the
        // early-restored loc_1B5CE title pose means Obj_TitleMenu `loc_1BAEA`
        // reaches `loc_1BB02` two title frames earlier here as well.
        let title_pose = Some((
            GameMode::TitleScreen,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        let demo_pose = Some((
            GameMode::DemoPlay,
            LevelId(0x00),
            Some(TITLE_KID_X_POS),
            Some(TITLE_KID_Y_POS),
        ));
        assert_eq!(
            frame_34915, title_pose,
            "frame 34915 should still be the final title-mode frame before the later intro-sequence-1 title shell reaches loc_1BB02",
        );
        assert_eq!(
            frame_34916, demo_pose,
            "frame 34916 should be the first demo-play frame after the later intro-sequence-1 title return reaches loc_1BB02",
        );
        assert_eq!(
            frame_34917, demo_pose,
            "frame 34917 should remain in demo playback on the following hidden bootstrap frame",
        );
        assert_eq!(
            frame_34973,
            Some((
                GameMode::DemoPlay,
                LevelId(0x1B),
                Some(demo::DEMO_CAMERA_PRELOAD_FRAME as usize),
                demo::DEMO_2_INTRO_SEQUENCE_5_TITLE_RETURN_CAMERA_PRELOAD_Y_FP,
            )),
            "frame 34973 should keep the timeout-return demo-2 preload camera row after the later intro-sequence-1 title return",
        );
    }

    #[test]
    fn published_tas_later_demo_2_transform_ignores_start_abort_on_frame_35148() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_35146 = None;
        let mut frame_35147 = None;
        let mut frame_35148 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(35_149) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            let snapshot = (
                trace.game_mode,
                trace.current_level,
                trace.paused,
                matches!(runtime.boot_state, BootState::DemoPlay { .. }),
            );
            match frame_index {
                35_146 => frame_35146 = Some(snapshot),
                35_147 => frame_35147 = Some(snapshot),
                35_148 => {
                    frame_35148 = Some(snapshot);
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_119D8 polls Start for demo aborts, but Kid_Transform hibernates
        // the regular gameplay/object loop while the later published-TAS demo-2
        // transform is still in progress. Oracle therefore ignores the frame-35146
        // Start press and keeps demo play active through frame 35148.
        let demo_transform_pose = Some((GameMode::DemoPlay, LevelId(0x1B), false, true));
        assert_eq!(
            frame_35146, demo_transform_pose,
            "frame 35146 should still be demo play when the later demo-2 Start press lands during Kid_Transform",
        );
        assert_eq!(
            frame_35147, demo_transform_pose,
            "frame 35147 should remain in demo play while the transform coroutine still hibernates the frame loop",
        );
        assert_eq!(
            frame_35148, demo_transform_pose,
            "frame 35148 should still ignore the queued Start abort until the later demo-2 transform hibernation ends",
        );
    }

    #[test]
    fn published_tas_later_timeout_restart_intro_sequence_2_start_title_jump_keeps_mode_18() {
        let Some((mut runtime, script)) = cached_any_percent_published_runtime_and_script() else {
            return;
        };
        let mut frame_36819 = None;
        let mut frame_36820 = None;
        let mut frame_36821 = None;

        for (frame_index, mask) in script.iter().copied().enumerate().take(36_822) {
            let trace = runtime.step_frame(FrameInput::from_mask(mask));
            match frame_index {
                36_819 => frame_36819 = Some(trace.game_mode),
                36_820 => frame_36820 = Some(trace.game_mode),
                36_821 => {
                    frame_36821 = Some(trace.game_mode);
                    break;
                }
                _ => {}
            }
        }

        // ASM loc_1BA02 tests Start inside the intro-sequence-2 DBF loop. On
        // this timeout-restart lineage, oracle keeps `$18` visible for one more
        // traced frame before sub_1C246 -> loc_1C264 writes title mode `$2C`.
        assert_eq!(
            frame_36819,
            Some(GameMode::IntroSequence2),
            "frame 36819 should keep loc_1BA02 in `Game_Mode=$18` while the Start edge lands",
        );
        assert_eq!(
            frame_36820,
            Some(GameMode::IntroSequence2),
            "frame 36820 should keep loc_1BA02 in `Game_Mode=$18` for one more traced frame",
        );
        assert_eq!(
            frame_36821,
            Some(GameMode::TitleScreen),
            "frame 36821 should expose loc_1C264's Start-driven title jump after one more traced `$18` frame",
        );
    }

    /// Determinism: running the same scenario twice must produce identical traces.
    /// Non-determinism would invalidate all oracle comparisons.
    #[test]
    fn determinism_same_scenario_produces_identical_traces() {
        let manifest = manifest_with_levels(3);
        let script = vec![0u8; 121]; // 120 frames of idle

        let trace1 = run_script_with_warp(manifest.clone(), &script, 120, Some(0));
        let trace2 = run_script_with_warp(manifest, &script, 120, Some(0));

        assert_eq!(trace1.len(), trace2.len(), "trace lengths differ");

        let crcs1: Vec<u32> = trace1
            .iter()
            .map(crate::types::NativeTraceFrame::authoritative_crc32)
            .collect();
        let crcs2: Vec<u32> = trace2
            .iter()
            .map(crate::types::NativeTraceFrame::authoritative_crc32)
            .collect();
        assert_eq!(crcs1, crcs2, "determinism violation: CRC arrays differ");

        for (i, (a, b)) in trace1.iter().zip(trace2.iter()).enumerate() {
            assert_eq!(a.kid_x_pos, b.kid_x_pos, "frame {i}: kid_x_pos differs");
            assert_eq!(a.kid_y_pos, b.kid_y_pos, "frame {i}: kid_y_pos differs");
            assert_eq!(
                a.camera_x_pos, b.camera_x_pos,
                "frame {i}: camera_x_pos differs"
            );
            assert_eq!(
                a.camera_y_pos, b.camera_y_pos,
                "frame {i}: camera_y_pos differs"
            );
            assert_eq!(a.score, b.score, "frame {i}: score differs");
            assert_eq!(
                a.number_hitpoints, b.number_hitpoints,
                "frame {i}: HP differs"
            );
            assert_eq!(
                a.active_enemy_object_count, b.active_enemy_object_count,
                "frame {i}: enemy count differs"
            );
        }
    }

    #[test]
    fn script_runner_run_with_vdp_returns_frame_aligned_trace_and_vdp_states_without_cache() {
        let manifest = manifest_with_levels(1);
        let (trace, snapshots) = ScriptRunner::new(manifest).run_with_vdp(&[0, 0, 0], 3);

        assert_eq!(
            trace.len(),
            3,
            "ScriptRunner::run_with_vdp must preserve the requested trace frame count"
        );
        assert_eq!(
            snapshots.len(),
            3,
            "ScriptRunner::run_with_vdp must keep VDP snapshots frame-aligned with the trace"
        );
        assert!(
            trace.iter().any(|frame| frame.frame_index != 0),
            "the returned trace must come from live runtime stepping, not a default-frame replacement"
        );
    }

    #[test]
    fn script_runner_returns_pre_step_render_latch_mode_level_loc_901() {
        let manifest = manifest_with_levels(1);
        let (zero_frame_latch, zero_trace) = ScriptRunner::new(manifest.clone())
            .warp_to_level(0)
            .run_with_initial_render_latch(&[], 0);
        let (initial_render_latch, trace) = ScriptRunner::new(manifest)
            .warp_to_level(0)
            .run_with_initial_render_latch(&[0, 0, 0], 3);

        assert!(zero_trace.is_empty());
        assert_eq!(trace.len(), 3);
        assert_eq!(
            initial_render_latch.authoritative_crc32(),
            zero_frame_latch.authoritative_crc32(),
            "ASM Mode_Level loc_901 render evidence must expose the configured runtime boundary before any scenario step"
        );
    }

    #[test]
    fn settled_warp_runner_starts_after_stable_ingame_window() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let script = vec![0u8; 2];

        let fresh = run_script_with_warp(manifest.clone(), &script, 1, Some(1));
        let settled = run_script_with_settled_warp(manifest, &script, 1, Some(1));

        assert_eq!(
            settled[0].frame_index,
            fresh[0].frame_index + 108,
            "full-frame oracle helper includes the Level 1 hidden resume-tail frame that lets Tank.asm loc_3C044 expose the ROM body/turret phase",
        );
        assert_eq!(settled[0].game_mode, GameMode::InGame);
        assert_eq!(settled[0].current_level, LevelId(1));
        assert!(!settled[0].paused);
        let first_tank_body = settled[0]
            .active_objects
            .iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x0A }
                    && object.layout_index == Some(1)
                    && object.y_pos == 0x00E0_0000
                    && object.hit_points == Some(2)
            })
            .expect("Level 0x01 first RockTank body should be active after settled warp");
        let first_tank_turret = settled[0]
            .active_objects
            .iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x0A }
                    && object.layout_index == Some(1)
                    && object.y_pos == 0x00D8_0000
                    && matches!(object.hit_points, None | Some(0))
            })
            .expect("Level 0x01 first RockTank turret child should be active after settled warp");
        assert_eq!(
            (first_tank_body.x_pos, first_tank_turret.x_pos),
            (0x00A4_8000, 0x00A4_0000),
            "ASM Tank.asm loc_3B2E4 into shared walker loc_3C0FA/loc_3C026: oracle settled frame 0 has the body one half-pixel ahead of the synced turret child",
        );
        assert_eq!(
            first_tank_body.facing,
            Some(Direction::Right),
            "ASM loc_3C152 -> loc_3C026 turns the leftward RockTank back to the set x_direction/rightward path before settled frame 0",
        );
    }

    #[test]
    fn settled_recording_warp_level_zero_uses_title_card_live_window() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.set_recorded_title_card_handoff_delay(true);

        runtime.boot_to_level(LevelId(0));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0));

        assert_eq!(
            runtime.trace_frame().hud_timer_sprite_seconds_loc_bee8,
            Some(179),
            "ASM Mode_Level loc_901 transfers the timer digits previously built by Make_SpriteAttr_HUD loc_BEE8; reseeding live timer RAM must not rewrite that sprite table",
        );
        let first_recorded_frame = runtime.step_frame(FrameInput::default());
        assert_eq!(
            (
                first_recorded_frame.time_frames_loc_3038a,
                first_recorded_frame.level_timer_seconds,
                first_recorded_frame.level_timer_sub_frames,
            ),
            (0x003D, 179, 0x003C),
            "ASM loc_3038A/Make_SpriteAttr_HUD loc_BE82: Level 0 oracle frame 0 exposes Time_Frames=$003D and timer 2:59/$003C",
        );
        assert_eq!(
            first_recorded_frame.hud_timer_sprite_seconds_loc_bee8,
            Some(179),
            "ASM Make_SpriteAttr_HUD loc_BEE8 rebuilds the next timer sprite table from the decremented 2:59 live digits",
        );
        let first_dragon = first_recorded_frame
            .active_objects
            .into_iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x0D }
                    && object.layout_index == Some(2)
            })
            .expect("Level 0x00 settled recording should expose the opening dragon");
        assert_eq!(
            first_dragon.x_pos, 0x00AA_8000,
            "ASM loc_1AE2E recording oracle bootstrap waits 60 stable live frames for Level 0; using the 108-frame full-frame sweep age over-advances the opening dragon",
        );
    }

    #[test]
    fn issue384_hoverboard_beach_archer_does_not_fire_by_recording_frame_60_loc_33fda() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let level = LevelId(0x17);
        let trace = ScriptRunner::new(manifest)
            .warp_to_level(level.0)
            .settle_recording_warp()
            .run(&[0; 360], 360);
        let frame_60 = &trace[60];
        assert_eq!(
            frame_60.number_objects, 5,
            "ASM Archer.asm loc_33FDA/loc_3408C/loc_34266: the Level 0x17 oracle remains in the initial hibernate and held-arrow pose path through recording frame 60, so Number_of_Arrows stays zero and no arrow contributes to Number_Objects"
        );
        assert_eq!(
            (trace[132].number_objects, trace[133].number_objects),
            (6, 5),
            "ASM Make_SpriteFromGfxObject loc_11B6 and Archer.asm loc_342D6/loc_34336: the first arrow exists on oracle frame 132, then its off-screen `$19` flag decrements Number_of_Arrows and deletes it on frame 133"
        );
        assert_eq!(
            trace[359].number_objects, 5,
            "ASM Archer.asm loc_34336: no fired arrow remains at the Level 0x17 idle-tail checkpoint"
        );
    }

    #[test]
    fn issue386_elsewhere12_mapped_fixture_uses_loc_15a0_dimensions() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let grid = NativeRuntime::with_generated_level_layout(manifest.clone())
            .block_grid_for_level(LevelId(0x54))
            .expect("Level 0x54 should decode its mapped resource terrain");
        assert_eq!(
            (grid.width_cells, grid.height_cells),
            (20, 56),
            "ASM Load_LevelData loc_15A0: logical Level 0x54 maps to resource 0x68's 1x4 header, so a stale 2x2 live-dump fixture must not override the ROM dimensions",
        );
        let trace = ScriptRunner::new(manifest)
            .warp_to_level(0x54)
            .settle_recording_warp()
            .run(&[0; 61], 61);
        let frame_60 = &trace[60];

        assert_eq!(
            (
                frame_60.game_mode_value,
                frame_60.current_level,
                frame_60.number_lives,
                frame_60.kid_y_pos,
                frame_60.kid_y_velocity,
                frame_60.camera_y_pos,
            ),
            (
                GameMode::InGame.as_disassembly_value(),
                LevelId(0x54),
                3,
                Some(0x032F_0000),
                Some(0),
                0x0293_0000,
            ),
            "ASM loc_7526/sub_7B30: the debug title-card entry starts Kid one pixel above PlayerStart_Y and resolves the spawn before gravity; the Level 0x54 oracle remains alive at recording frame 60",
        );
    }

    #[test]
    fn recording_settle_loads_in_view_enemies_loc_368a8_3691e() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        for (level, expected_counts) in [(0x01, (4, 6)), (0x36, (5, 6))] {
            let trace = ScriptRunner::new(manifest.clone())
                .warp_to_level(level)
                .settle_recording_warp()
                .run(&[0; 360], 360);
            let frame_60 = &trace[60];

            assert_eq!(
                (frame_60.number_objects, frame_60.number_gfx_objects),
                expected_counts,
                "ASM Mode_Level loc_80C -> Manage_EnemyLoading loc_368A8/loc_3691E must allocate every in-view enemy during the settled recording window for Level {level:#04x}",
            );
            assert_eq!(
                (trace[359].number_objects, trace[359].number_gfx_objects),
                expected_counts,
                "ASM Lion.asm sub_3E8BA keeps the Level {level:#04x} in-view roster alive at the idle tail: {:?}",
                trace[359].active_enemy_details,
            );
        }
    }

    #[test]
    fn issue388_pyramids_cloud_wait_resumes_at_next_loc_35bb2_command() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let trace = ScriptRunner::new(manifest)
            .warp_to_level(0x18)
            .settle_recording_warp()
            .run(&[0; 360], 360);
        let idle_tail = &trace[359];

        assert_eq!(
            (
                idle_tail.game_mode_value,
                idle_tail.number_lives,
                idle_tail.number_hitpoints,
                idle_tail.just_received_damage,
                idle_tail.number_objects,
                idle_tail.kid_x_pos,
                idle_tail.kid_y_pos,
                idle_tail.camera_y_pos,
            ),
            (
                GameMode::InGame.as_disassembly_value(),
                3,
                1,
                false,
                6,
                Some(0x0060_0000),
                Some(0x00CF_0000),
                0x002C_0000,
            ),
            "ASM Cloud.asm loc_35BB2/loc_35C54: a0 remains after the consumed wait word, so the Level 0x18 Cloud clears `$44(a5)` after its second wait instead of repeatedly firing until Kid dies",
        );
    }

    #[test]
    fn script_runner_recording_settle_uses_level_zero_recording_age() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let first_recorded_frame = ScriptRunner::new(manifest)
            .warp_to_level(0)
            .settle_recording_warp()
            .run(&[0], 1)
            .into_iter()
            .next()
            .expect("one frame");
        let first_dragon = first_recorded_frame
            .active_objects
            .into_iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x0D }
                    && object.layout_index == Some(2)
            })
            .expect("Level 0x00 settled recording should expose the opening dragon");

        assert_eq!(
            first_dragon.x_pos, 0x00AA_8000,
            "ASM loc_1AE2E recording scenarios use the 60-frame title-card live window; full-frame sweep settle would over-advance this dragon by 48 frames",
        );
    }

    #[test]
    fn settled_recording_warp_level_zero_delays_first_gameplay_input_until_next_frame() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let mut script = vec![0; 64];
        script[60..].fill(0x08);
        let trace = ScriptRunner::new(manifest)
            .warp_to_level(0)
            .settle_recording_warp()
            .run(&script, script.len());

        assert_eq!(
            trace[60].kid_x_pos,
            Some(0x0020_0000),
            "ASM Mode_Level (kid.asm:877-917) runs Execute_Objects before ReadJoypad; the first Right sample should not move Kid on the same traced frame",
        );
        assert_eq!(
            trace[60].character_movement,
            Some(0),
            "ASM loc_C4E/loc_C9C only exposes the new Ctrl_Held value after the movement slice has already run",
        );
        assert_eq!(
            trace[61].kid_x_pos,
            Some(0x0020_0000),
            "oracle-probed Ctrl_Held changes on this frame, but Mode_Level movement consumed the previous Ctrl_Held before ReadJoypad",
        );
        assert_eq!(trace[61].character_movement, Some(0));
        assert_eq!(
            trace[62].kid_x_pos,
            Some(0x0020_1000),
            "the first sampled Right input is consumed by Kid movement on the following frame",
        );
        assert_eq!(trace[62].character_movement, Some(2));
    }

    #[test]
    fn settled_recording_warp_level_one_keeps_level_skip_resume_tail_age() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);

        runtime.boot_to_level(LevelId(1));
        runtime.settle_recording_oracle_warp_live_window(LevelId(1));

        let first_recorded_frame = runtime.step_frame(FrameInput::default());
        let first_tank_body = first_recorded_frame
            .active_objects
            .into_iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x0A }
                    && object.layout_index == Some(1)
                    && object.y_pos == 0x00E0_0000
            })
            .expect("Level 0x01 settled recording should expose the first RockTank body");
        assert_eq!(
            first_tank_body.x_pos, 0x00A3_8000,
            "ASM loc_74E0 level-skip resume has a live title-card tail that native `boot_to_level` shortcuts; oracle settle tracing ends with RNG offset $FFCB, so frame 0 reaches Tank.asm loc_3B40A at x=$00A38000 and RNG $FFCF",
        );
        assert_eq!(first_tank_body.facing, Some(Direction::Right));
        assert_eq!(
            first_recorded_frame.rng_offset,
            Some(0xFFCF),
            "Drips.asm loc_3340E consumes the two Level 1 controller RNG words on scenario frame 0; over-settling by two frames starts #260 probes at $FFD7 instead",
        );
        assert_eq!(
            first_recorded_frame.rng_last_byte,
            Some(0x09),
            "oracle-probe --capture-settle-trace for issue260_p11 ends Level 1 target_live step 59 at RNG $FFCB/$32, then Drips.asm loc_3340E advances frame 0 to $FFCF/$09",
        );
    }

    #[test]
    fn level48_recording_settle_matches_ufo_drips_rng_frontier_loc_74e0() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let frame = ScriptRunner::new(manifest)
            .warp_to_level(0x48)
            .settle_recording_warp()
            .run(&[0], 1)
            .into_iter()
            .next()
            .expect("one settled Level 0x48 frame");
        let mut drips_waits = frame
            .active_enemy_details
            .iter()
            .filter(|object| object.kind == NativeObjectKind::Enemy { object_type: 0x16 })
            .map(|object| (object.x_pos >> 16, object.wait_timer))
            .collect::<Vec<_>>();
        drips_waits.sort_unstable_by_key(|(x, _)| *x);

        assert_eq!(
            (frame.rng_offset, frame.number_objects, drips_waits),
            (
                Some(0xFFD2),
                13,
                vec![
                    (488, Some(0x0040)),
                    (504, Some(0x005F)),
                    (648, Some(0x0023)),
                    (664, Some(0x0044)),
                ],
            ),
            "ASM loc_74E0 -> Execute_Objects loc_E12 and UFO.asm Enemy0F_UFO_Init: the debug title shell begins the target object schedule at oracle title-wait step 72, leaving frame 0 at RNG $FFD2, 13 counted workers, and the four loc_3340E Drips waits",
        );
        assert_eq!(
            frame.rng_ring,
            Some(RomRandomRing::from_bytes([
                0xF8, 0x5C, 0x32, 0x71, 0xB9, 0x61, 0x09, 0xFA, 0x54, 0xC0, 0x05, 0xB3, 0xE5, 0x5E,
                0x22, 0x59, 0x9B, 0x3F, 0xFA, 0xE6, 0x27, 0x5E, 0xA7, 0x64, 0x9C, 0xB7, 0x36, 0x64,
                0x5B, 0xBC, 0xDA, 0x11, 0xB2, 0xE6, 0xAC, 0xED, 0x40, 0xD1, 0xE6, 0x9F, 0x61, 0x05,
                0xC3, 0x78, 0x00, 0xA3, 0xC5, 0x89, 0x96, 0x12, 0xE1, 0x8E, 0x29, 0x15, 0x4A,
            ])),
            "ASM Init_RNG/Get_RandomNumber_byte (kid.asm:1106-1204): oracle-probe --capture-settle-trace plus frame 0 proves the complete Level 48 ring matches even though UFO draw assignment differs",
        );
    }

    #[test]
    fn settled_recording_warp_level_eight_keeps_murder_wall_level_skip_tail_age() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);

        runtime.boot_to_level(LevelId(0x08));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x08));

        let first_recorded_frame = runtime.step_frame(FrameInput::default());
        assert_eq!(
            first_recorded_frame.murder_wall_x,
            Some(-0x001B_7B00),
            "ASM Murderwall loc_140CE starts Level 0x08 at -$30 px, and loc_7316/loc_7334 accelerates one $100 step per frame; oracle issue266 settled warp frame 0 has advanced 102 wall ticks through the level-skip live tail",
        );
    }

    #[test]
    fn settled_recording_warp_bloody_swamp_keeps_murder_wall_level_skip_tail_age() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);

        runtime.boot_to_level(LevelId(0x3E));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x3E));

        let first_recorded_frame = runtime.step_frame(FrameInput::default());
        assert_eq!(
            first_recorded_frame.murder_wall_x,
            Some(-0x0019_0200),
            "ASM Murderwall loc_140CE plus the level-skip loc_74E0 live tail starts Bloody Swamp recording frame 0 at the oracle issue374 wall age",
        );

        let trace = std::iter::once(first_recorded_frame)
            .chain((0..60).map(|_| runtime.step_frame(FrameInput::default())))
            .collect::<Vec<_>>();
        assert_eq!(
            trace[60].camera_x_pos, 0x0007_0000,
            "ASM loc_74E0 hidden level-skip tail plus 60 recording frames reaches oracle issue374 Bloody Swamp camera_x at frame 60",
        );
        assert_eq!(
            trace[60].murder_wall_x,
            Some(0x0007_7400),
            "ASM Murderwall loc_140CE frame-60 wall position should match oracle issue374 after the hidden level-skip tail",
        );
    }

    #[test]
    fn settled_recording_warp_level_six_ages_flying_dragon_into_descent_fire() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let trace = ScriptRunner::new(manifest)
            .warp_to_level(0x06)
            .settle_recording_warp()
            .run(&[0; 75], 75);
        let frame_36 = trace
            .get(36)
            .expect("Level 0x06 issue #289 probe should include frame 36");
        let frame_68 = trace
            .get(68)
            .expect("Level 0x06 issue #289 probe should include frame 68");
        let frame_73 = trace
            .get(73)
            .expect("Level 0x06 issue #289 probe should include frame 73");
        let frame_74 = trace
            .get(74)
            .expect("Level 0x06 issue #289 probe should include frame 74");

        let dragon_at = |frame: usize| {
            trace[frame]
                .active_enemy_details
                .iter()
                .find(|object| {
                    object.kind == NativeObjectKind::Enemy { object_type: 0x0C }
                        && object.layout_index == Some(0)
                })
                .expect("Level 0x06 opening Flying Dragon")
        };
        assert_eq!(
            (dragon_at(3).x_pos, dragon_at(3).y_pos),
            (0x0088_0000, 0x001F_0000),
            "ASM Dragon_Flying.asm loc_3480A DBF hibernation and the interval-1 oracle keep the opening Dragon stationary through recording frame 3",
        );
        assert_eq!(
            (dragon_at(8).x_pos, dragon_at(8).y_pos),
            (0x0088_0000, 0x001F_0000),
            "ASM Dragon_Flying.asm loc_3480A keeps the opening Dragon stationary through oracle recording frame 8",
        );
        assert_eq!(
            (dragon_at(9).x_pos, dragon_at(9).y_pos),
            (0x0087_0000, 0x0020_0000),
            "ASM Dragon_Flying.asm loc_348FC first commits the descent vector on oracle recording frame 9",
        );

        assert_eq!(
            frame_36.number_objects, 6,
            "ASM Dragon_Flying.asm loc_348A0/loc_34BAC spawns the descent-fire child during the settled recording live window; oracle issue289 frame 36 has the Dragon parent plus fire child active",
        );
        assert!(
            frame_36
                .active_projectile_details
                .iter()
                .any(|projectile| projectile.kind == ActiveProjectileKind::DragonFlyingDescentFire),
            "ASM Dragon_Flying.asm loc_34BAC child must be present at oracle issue289 frame 36, not a load-only parent still waiting in loc_34710 hover",
        );
        assert_eq!(
            trace[60].number_gfx_objects, 6,
            "ASM Dragon_Flying.asm loc_34BAC calls Load_GfxObjectSlot before loc_32146, so the descent-fire child contributes one GfxObject at oracle recording frame 60",
        );
        assert_eq!(
            frame_68.number_objects, 6,
            "ASM Dragon_Flying.asm loc_34C4A cleanup hibernates the descent-fire child through oracle issue289 frame 68",
        );
        assert_eq!(
            frame_73.number_objects, 6,
            "ASM Dragon_Flying.asm loc_34C4A cleanup keeps the descent-fire child allocated through oracle issue289 frame 73",
        );
        assert_eq!(
            frame_74.number_objects, 5,
            "ASM Dragon_Flying.asm loc_34C3A/loc_34C4A has cleaned up the descent-fire child by oracle issue289 frame 74 while the Dragon parent continues",
        );
    }

    #[test]
    fn settled_recording_warp_ages_multi_head_boss_level_skip_tail_enemy22_init() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level(LevelId(0x10));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x10));
        let trace = (0..=60)
            .map(|_| runtime.step_frame(FrameInput::default()))
            .collect::<Vec<_>>();

        assert_eq!(
            trace[60].number_objects, 24,
            "ASM Enemy22_Shiskaboss_Init keeps nine skewer workers live while loc_39566 has two boss-arrow workers allocated at the oracle Level 0x10 settle checkpoint",
        );
        assert_eq!(
            trace[60]
                .active_projectile_details
                .iter()
                .filter(|projectile| projectile.kind == ActiveProjectileKind::BossHeadArrow)
                .count(),
            2,
            "ASM loc_74E0's hidden live tail must age both loc_39566 projectile gates before Level 0x10 recording frame 60",
        );
    }

    #[test]
    fn settled_recording_warp_level_1b_keeps_opening_sphere_short_age_loc_36148() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);

        runtime.boot_to_level(LevelId(0x1B));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x1B));

        let first_recorded_frame = runtime.step_frame(FrameInput::default());
        let sphere = first_recorded_frame
            .active_objects
            .iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x06 }
                    && object.layout_index == Some(0)
            })
            .expect("Level 0x1B settled recording should expose the opening Sphere");

        assert_eq!(
            (
                first_recorded_frame.number_hitpoints,
                first_recorded_frame.just_received_damage,
                sphere.x_pos,
                sphere.y_pos,
                sphere.x_velocity,
                sphere.y_velocity,
                sphere.hit_points,
            ),
            (
                2,
                false,
                0x0010_0000,
                0x025E_5800,
                Some(-0x0001_0000),
                Some(0x0000_4800),
                Some(0),
            ),
            "ASM Sphere.asm Enemy06_Sphere_Init/loc_36148 plus GfxObjects_Collision loc_171C: oracle issue #359 frame 0 has only the short settled-recording live age, not a pre-damaged Kid from over-aged hidden enemy scheduling",
        );
    }

    #[test]
    fn settled_recording_warp_level_one_matches_first_rock_tank_body_turret_phase() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let trace = ScriptRunner::new(manifest)
            .warp_to_level(1)
            .settle_recording_warp()
            .run(&[0, 0], 2);

        let tank_pair = |frame: &NativeTraceFrame| {
            let body = frame
                .active_objects
                .iter()
                .find(|object| {
                    object.kind == NativeObjectKind::Enemy { object_type: 0x0A }
                        && object.layout_index == Some(1)
                        && object.y_pos == 0x00E0_0000
                        && object.hit_points == Some(2)
                })
                .expect("Level 0x01 first RockTank body should be active");
            let turret = frame
                .active_objects
                .iter()
                .find(|object| {
                    object.kind == NativeObjectKind::Enemy { object_type: 0x0A }
                        && object.layout_index == Some(1)
                        && object.y_pos == 0x00D8_0000
                        && matches!(object.hit_points, None | Some(0))
                })
                .expect("Level 0x01 first RockTank turret child should be active");
            (body.x_pos, turret.x_pos)
        };

        assert_eq!(
            tank_pair(&trace[0]),
            (0x00A3_8000, 0x00A3_0000),
            "ASM Tank.asm loc_3B40A enters shared walker loc_3C026, and loc_3C03A/loc_3C044 syncs the child on the hibernate boundary; oracle frame 0 sees the body one half-pixel ahead of the turret",
        );
        assert_eq!(
            tank_pair(&trace[1]),
            (0x00A4_0000, 0x00A4_0000),
            "ASM shared walker loc_3C03A syncs the turret before the next hibernate, so oracle frame 1 has both Tank objects at the same X",
        );
    }

    #[test]
    fn issue260_route_turns_first_rock_tank_left_at_oracle_frame_99() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let Some(script) = cached_issue260_floor_probe_route_script() else {
            return;
        };

        let trace = ScriptRunner::new(manifest)
            .warp_to_level(1)
            .settle_recording_warp()
            .probe_start_position(344, 575)
            .probe_camera_position(192, 352)
            .run(&script, script.len());
        let frame_99 = trace
            .get(99)
            .expect("issue #260 route should include frame 99");
        let first_tank_body = frame_99
            .active_objects
            .iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x0A }
                    && object.layout_index == Some(1)
                    && object.y_pos == 0x00E0_0000
                    && object.hit_points == Some(2)
            })
            .expect("Level 0x01 first RockTank body should still be active at route frame 99");
        let first_tank_turret = frame_99
            .active_objects
            .iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x0A }
                    && object.layout_index == Some(1)
                    && object.y_pos == 0x00D8_0000
                    && matches!(object.hit_points, None | Some(0))
            })
            .expect(
                "Level 0x01 first RockTank turret child should still be active at route frame 99",
            );

        assert_eq!(
            (
                first_tank_body.x_pos,
                first_tank_turret.x_pos,
                first_tank_body.facing,
                first_tank_turret.facing,
            ),
            (
                0x00D5_8000,
                0x00D5_0000,
                Some(Direction::Left),
                Some(Direction::Left),
            ),
            "ASM Tank.asm loc_3B40A enters shared walker loc_3C026/loc_3C04E; loc_3C0A6 must turn at the route's right-edge boundary before storing the predicted X",
        );
    }

    #[test]
    fn issue260_route_turns_first_rock_tank_right_at_oracle_frame_333() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let Some(script) = cached_issue260_floor_probe_route_script() else {
            return;
        };

        let trace = ScriptRunner::new(manifest)
            .warp_to_level(1)
            .settle_recording_warp()
            .probe_start_position(344, 575)
            .probe_camera_position(192, 352)
            .run(&script, script.len());
        let frame_333 = trace
            .get(333)
            .expect("issue #260 route should include frame 333");
        let first_tank_body = frame_333
            .active_objects
            .iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x0A }
                    && object.layout_index == Some(1)
                    && object.y_pos == 0x00E0_0000
                    && object.hit_points == Some(2)
            })
            .expect("Level 0x01 first RockTank body should still be active at route frame 333");
        let first_tank_turret = frame_333
            .active_objects
            .iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x0A }
                    && object.layout_index == Some(1)
                    && object.y_pos == 0x00D8_0000
                    && matches!(object.hit_points, None | Some(0))
            })
            .expect(
                "Level 0x01 first RockTank turret child should still be active at route frame 333",
            );

        assert_eq!(
            (
                first_tank_body.x_pos,
                first_tank_turret.x_pos,
                first_tank_body.facing,
                first_tank_turret.facing,
            ),
            (
                0x00D5_0000,
                0x00D5_0000,
                Some(Direction::Right),
                Some(Direction::Right),
            ),
            "ASM Tank.asm loc_3B40A enters shared walker loc_3C04E; oracle keeps the route's later right-edge boundary on the pre-turn hibernate frame",
        );
    }

    #[test]
    fn settled_recording_probe_level_one_suppresses_inner_visible_drips_spawn() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level(LevelId(1));
        runtime.settle_recording_oracle_warp_live_window(LevelId(1));
        assert!(
            runtime
                .active_enemies
                .iter()
                .all(|enemy| { enemy.family != crate::enemies::EnemyFamily::Drips }),
            "settled Level 1 recording baseline should not already contain the lower Drips controllers"
        );
        runtime.set_player_probe_position(0x0158, 0x023F);
        runtime.set_camera_probe_position(0x00C0, 0x0160);
        assert!(runtime.enemy_loading_initial_pass_complete);

        let frame = runtime.step_frame(FrameInput::default());

        assert!(
            frame
                .active_objects
                .iter()
                .all(|object| { object.kind != (NativeObjectKind::Enemy { object_type: 0x16 }) }),
            "ASM Drips.asm Enemy16_Drip_Init/loc_33488 keeps the parent/controller gfx object hidden; settled probes must not expose Level 1 Drips layout indices 7 or 8 as active enemy sprites",
        );
    }

    #[test]
    fn settled_recording_probe_start_clears_stale_player_motion_state_loc_9f18() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level(LevelId(0));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0));
        runtime.set_player_probe_position(0x02F8, 0x00E7);
        runtime.set_camera_probe_position(0x0260, 0x0020);

        let frame = runtime.step_frame(FrameInput::default());

        assert_eq!(
            (
                frame.kid_x_pos,
                frame.kid_y_pos,
                frame.kid_x_velocity,
                frame.kid_y_velocity,
                frame.kid_facing,
                frame.character_movement,
            ),
            (
                Some(0x02F8_0000),
                Some(0x00E8_0000),
                Some(0),
                Some(0),
                Some(Direction::Right),
                Some(3),
            ),
            "ASM loc_9F18 consumes the first neutral frame from a stationary right-facing probe; direct #258 starts must not inherit stale slope-release velocity/facing from the settled route",
        );
    }

    #[test]
    fn mode_level_loads_rock_tank_after_object_step_loc_80c_368a8() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level(LevelId(1));
        runtime.settle_recording_oracle_warp_live_window(LevelId(1));
        runtime
            .active_enemies
            .retain(|enemy| enemy.layout_index != 0);
        runtime.rebuild_active_enemy_roster();
        runtime.set_player_probe_position(0x00EB, 0x009F);
        runtime.set_camera_probe_position(0x00C8, 0x0000);
        assert!(runtime.enemy_loading_initial_pass_complete);

        let frame = runtime.step_frame(FrameInput::default());
        let spawned_tank_pair = frame
            .active_objects
            .iter()
            .filter(|object| {
                object.kind == (NativeObjectKind::Enemy { object_type: 0x0A })
                    && object.layout_index == Some(0)
            })
            .collect::<Vec<_>>();

        assert_eq!(
            spawned_tank_pair.len(),
            2,
            "ASM Mode_Level loc_80C calls Manage_EnemyLoading after Execute_Objects, so the Level 01 `$0288,$008F` Tank pair should be trace-visible before its first object step"
        );
        assert!(
            spawned_tank_pair
                .iter()
                .any(|object| object.x_pos == 0x0288_0000
                    && object.y_pos == 0x008F_0000
                    && object.hit_points == Some(2)),
            "ASM Tank.asm loc_3B31E body should still be at the layout spawn position on the loader frame"
        );
        assert!(
            spawned_tank_pair.iter().any(|object| {
                object.x_pos == 0x0288_0000
                    && object.y_pos == 0x0087_0000
                    && object.hit_points.is_none()
            }),
            "ASM Tank.asm loc_3B31E child gfx slot should be exposed at the unstepped turret offset on the loader frame"
        );
    }

    #[test]
    fn probe_camera_primes_highwater_shooting_rock_tank_loc_31f8e_368a8() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level(LevelId(2));
        runtime.settle_recording_oracle_warp_live_window(LevelId(2));
        runtime.set_player_probe_position(0x02F8, 0x008F);
        runtime.set_camera_probe_position(0x0260, 0x0000);

        let frame = runtime.trace_frame();
        let spawned_tank_pair = frame
            .active_objects
            .iter()
            .filter(|object| {
                object.kind == (NativeObjectKind::Enemy { object_type: 0x0B })
                    && object.layout_index == Some(2)
            })
            .collect::<Vec<_>>();

        assert_eq!(
            spawned_tank_pair.len(),
            2,
            "ASM loc_31F8E/Manage_EnemyLoading loc_368A8 must reseek the Level 02 enemy layout for a direct probe camera before #271 contact scoring"
        );
        assert!(
            spawned_tank_pair.iter().any(|object| {
                object.x_pos == 0x0328_0000
                    && object.y_pos == 0x009F_0000
                    && object.hit_points == Some(2)
            }),
            "ASM Tank_Shooting.asm loc_3B5A6 body should be exposed at Highwater layout object 02"
        );
        assert!(
            spawned_tank_pair.iter().any(|object| {
                object.x_pos == 0x0328_0000
                    && object.y_pos == 0x0097_0000
                    && object.hit_points == Some(0)
            }),
            "ASM Tank_Shooting.asm loc_3B5FA child gfx slot should be exposed 8px above the body"
        );
    }

    #[test]
    fn level01_layout0_rock_tank_scan_enters_loc_3c266_fall() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level(LevelId(1));
        let grid = runtime.block_grid().expect("Level 01 block grid");
        let scan = terrain::grounded_walker_scan_left_sub_36A58(grid, 0x0288, 0x008F, 0x0B, 0x20);

        assert_eq!(
            (scan.flags_d7, scan.floor_class_d6, scan.column_span_d3),
            (0, -1, 2),
            "ASM sub_36A58 -> loc_36AF0 classifies Level 01 layout `$0288,$008F` as a clear following row; loc_3C0FA must branch to loc_3C266 rather than pinning the Tank to ground"
        );
    }

    #[test]
    fn settled_level_one_drips_crawlers_count_as_objects_loc_335d8() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };

        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level(LevelId(1));
        runtime.settle_recording_oracle_warp_live_window(LevelId(1));
        runtime.set_player_probe_position(0x0158, 0x023F);
        runtime.set_camera_probe_position(0x00C0, 0x0160);

        let mut samples = Vec::new();
        for frame_index in 0..=629 {
            let frame = runtime.step_frame(FrameInput::default());
            if matches!(frame_index, 536 | 537 | 597 | 598 | 629) {
                samples.push((
                    frame_index,
                    frame.number_objects,
                    frame.number_gfx_objects,
                    frame
                        .active_projectile_details
                        .iter()
                        .any(|projectile| projectile.kind == ActiveProjectileKind::DripsDrop),
                    frame
                        .active_projectile_details
                        .iter()
                        .filter(|projectile| {
                            projectile.kind == ActiveProjectileKind::DripsSurfaceCrawler
                        })
                        .count(),
                ));
            }
        }

        assert_eq!(
            samples,
            vec![
                (536, 6, 4, false, 2),
                (537, 6, 4, false, 2),
                (597, 6, 4, true, 2),
                (598, 6, 4, true, 2),
                (629, 6, 4, false, 2),
            ],
            "ASM Drips.asm loc_335D8 allocates each surface crawler with j_Allocate_ObjectSlot while loc_33424/loc_3345C reuses the preallocated gfx-only falling child",
        );
    }

    #[test]
    fn settled_recording_level_12_flag_touch_preserves_oracle_stale_enemy_gfx_loc_c056() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);

        runtime.boot_to_level(LevelId(0x12));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x12));
        runtime.set_player_probe_position(0x0620, 0x0200);
        let flag_touch = runtime.step_frame(FrameInput::default());

        let active_enemy = |object_type, layout_index| {
            flag_touch.active_objects.iter().any(|object| {
                object.kind == NativeObjectKind::Enemy { object_type }
                    && object.layout_index == Some(layout_index)
            })
        };

        assert!(
            active_enemy(0x04, 4),
            "ASM CharacterCollision_TouchFlagpole loc_C056 preserves the stale Armadillo gfx slot observed in issue-175 oracle flag_touch before loc_C070 deletes object threads",
        );
        assert!(
            active_enemy(0x0D, 8),
            "ASM CharacterCollision_TouchFlagpole loc_C056 preserves the stale DragonWalking gfx slot observed in issue-175 oracle flag_touch before loc_C070 deletes object threads",
        );
        assert_eq!(
            flag_touch.active_platform_object_count, 4,
            "ASM Manage_PlatformLoading loc_2762 keeps the four visible Level 0x12 platform slots live through the flag-touch frame",
        );
        assert_eq!(
            (flag_touch.number_objects, flag_touch.number_gfx_objects),
            (9, 10),
            "issue-175 oracle flag_touch reports Kid + 2 EOL workers + 2 stale enemy gfx slots plus the four live platform slots"
        );
    }

    #[test]
    fn settled_level_one_first_rock_tank_reaches_oracle_patrol_x_by_frame_300() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let script = vec![0u8; 301];

        let trace = run_script_with_settled_warp(manifest, &script, 301, Some(1));
        let frame_300 = trace
            .get(300)
            .expect("settled Level 0x01 frame 300 should be captured");
        let first_tank_body = frame_300
            .active_objects
            .iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x0A }
                    && object.layout_index == Some(1)
                    && object.y_pos == 0x00E0_0000
                    && object.hit_points == Some(2)
            })
            .expect("Level 0x01 first RockTank body should still be active at frame 300");
        let first_tank_turret = frame_300
            .active_objects
            .iter()
            .find(|object| {
                object.kind == NativeObjectKind::Enemy { object_type: 0x0A }
                    && object.layout_index == Some(1)
                    && object.y_pos == 0x00D8_0000
                    && matches!(object.hit_points, None | Some(0))
            })
            .expect("Level 0x01 first RockTank turret child should still be active at frame 300");

        assert_eq!(
            (first_tank_body.x_pos, first_tank_turret.x_pos),
            (0x00C4_8000, 0x00C4_0000),
            "ASM Tank.asm loc_3B40A dispatches into shared walker loc_3C026/loc_3C04E; current full-frame oracle frame 300 matches the first RockTank body one half-pixel ahead of its turret child before the next tile-boundary turn",
        );
    }

    /// Determinism with active gameplay inputs (walk + jump).
    #[test]
    fn determinism_active_gameplay_identical_traces() {
        let manifest = manifest_with_levels(3);
        let mut script = vec![0u8; 181]; // 180 frames
        // Walk right from frame 60
        for s in script.iter_mut().skip(60).take(90) {
            *s = 1 << 3; // right
        }
        // Jump at frame 90
        for s in script.iter_mut().skip(90).take(4) {
            *s |= 1 << 4; // B button (jump)
        }

        let trace1 = run_script_with_warp(manifest.clone(), &script, 180, Some(0));
        let trace2 = run_script_with_warp(manifest, &script, 180, Some(0));

        assert_eq!(trace1.len(), trace2.len());

        let crcs1: Vec<u32> = trace1
            .iter()
            .map(crate::types::NativeTraceFrame::authoritative_crc32)
            .collect();
        let crcs2: Vec<u32> = trace2
            .iter()
            .map(crate::types::NativeTraceFrame::authoritative_crc32)
            .collect();
        assert_eq!(
            crcs1, crcs2,
            "determinism violation: CRC arrays differ on active gameplay"
        );

        for (i, (a, b)) in trace1.iter().zip(trace2.iter()).enumerate() {
            assert_eq!(a.kid_x_pos, b.kid_x_pos, "frame {i}: kid_x_pos differs");
            assert_eq!(a.kid_y_pos, b.kid_y_pos, "frame {i}: kid_y_pos differs");
            assert_eq!(
                a.kid_x_velocity, b.kid_x_velocity,
                "frame {i}: kid_x_velocity differs"
            );
            assert_eq!(
                a.kid_y_velocity, b.kid_y_velocity,
                "frame {i}: kid_y_velocity differs"
            );
        }
    }

    // --- Ghost blocks: step_ghost_blocks through NativeRuntime ---

    fn assert_under_skull_mountain3_lower_ghost_group_ticks_loc_124a6_f730(rt: &mut NativeRuntime) {
        let lower_group = rt
            .block_state
            .ghost_groups
            .iter()
            .find(|group| group.col == 0x17 && group.row == 0x12)
            .copied()
            .expect("Level 0x06 should keep the lower Skull Mountain 3 ghost group after load");
        assert_eq!(lower_group.count, 2);
        assert!(!lower_group.vertical);
        assert_eq!(lower_group.timer, 0x28);
        assert_eq!(lower_group.state, 0);

        for _ in 0..0x28 {
            rt.step_ghost_blocks();
        }

        let lower_group = rt
            .block_state
            .ghost_groups
            .iter()
            .find(|group| group.col == 0x17 && group.row == 0x12)
            .expect("Level 0x06 lower ghost group should still be tracked after ticking");
        assert_eq!(
            lower_group.state, 0x8001,
            "ASM sub_F730 loc_F8A6 starts the two-frame vanished lead-in when the initial solid hold expires"
        );
        let grid = rt
            .block_grid
            .as_ref()
            .expect("Level 0x06 grid should exist");
        for col in 0x17..=0x18 {
            let cell = grid.cell_at(col, 0x12);
            assert_eq!(
                cell.collision,
                terrain::CollisionKind::Empty,
                "ASM loc_F8A6/loc_F90A should clear collision for Skull Mountain 3 ghost cell ({col},18)"
            );
        }
    }

    #[test]
    fn level_entry_preserves_under_skull_mountain3_ghost_groups_loc_124a6_f730() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut rt = NativeRuntime::with_generated_level_layout(manifest);

        rt.begin_level_entry(LevelId(0x06));

        assert_under_skull_mountain3_lower_ghost_group_ticks_loc_124a6_f730(&mut rt);
    }

    #[test]
    fn warp_to_level_preserves_under_skull_mountain3_ghost_groups_loc_124a6_f730() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut rt = NativeRuntime::with_generated_level_layout(manifest);

        rt.warp_to_level(LevelId(0x06));

        assert_under_skull_mountain3_lower_ghost_group_ticks_loc_124a6_f730(&mut rt);
    }

    #[test]
    fn issue289_level06_lower_ghost_group_vanishes_before_kid_jump_collision_loc_f730_902a() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let Some((level, script)) = cached_issue289_level06_walk_right_script() else {
            return;
        };
        let mut rt = NativeRuntime::with_generated_level_layout(manifest);
        rt.boot_to_level(level);
        rt.settle_recording_oracle_warp_live_window(level);

        let mut frame_512 = None;
        for frame_index in 0usize..513 {
            let input = frame_index
                .checked_sub(1)
                .and_then(|index| script.get(index))
                .copied()
                .map(FrameInput::from_mask)
                .unwrap_or_default();
            let trace = rt.step_frame(input);
            if frame_index == 512 {
                frame_512 = Some(trace);
                break;
            }
        }

        let lower_group = rt
            .block_state
            .ghost_groups
            .iter()
            .find(|group| group.col == 0x17 && group.row == 0x12)
            .copied()
            .expect("Level 0x06 lower ghost group should still be tracked");
        assert!(
            !lower_group.solid,
            "ASM loc_124A6 should preserve the lower ghost group so sub_F730 can clear it during the recorded level-06 route; got timer={:#06x} state={:#06x}",
            lower_group.timer, lower_group.state
        );

        let frame_512 = frame_512.expect("issue #289 route should replay through frame 512");
        assert_eq!(frame_512.current_level, LevelId(0x06));
        assert_eq!(frame_512.kid_x_pos, Some(0x018A_0000));
        assert!(
            frame_512.kid_y_pos < Some(0x014F_0000),
            "ASM sub_F730 must vanish the lower Skull Mountain 3 ghost group before Kid's sub_902A ceiling probes; otherwise the jump at frame 512 is cancelled by stale solid cell (24,18)"
        );
        assert!(
            frame_512
                .kid_y_velocity
                .is_some_and(|velocity| velocity < 0),
            "ASM sub_902A should preserve the jump launch once sub_F730 has cleared the ghost block collision"
        );
    }

    #[test]
    fn issue230_crypt_recording_warp_vanishes_four_cell_vertical_ghost_loc_124a6_f730() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let Some((level, script)) = cached_issue230_crypt38_walk_right_script() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);

        runtime.boot_to_level(level);
        runtime.settle_recording_oracle_warp_live_window(level);
        for frame_index in 0usize..=298 {
            let input = frame_index
                .checked_sub(1)
                .and_then(|index| script.get(index))
                .copied()
                .map(FrameInput::from_mask)
                .unwrap_or_default();
            runtime.step_frame(input);
        }

        let ghost_group = runtime
            .block_state
            .ghost_groups
            .iter()
            .find(|group| group.col == 0x1D && group.row == 0x44)
            .copied()
            .expect("The Crypt should keep the vertical SolidGhost group at col $1D row $44");
        assert_eq!(
            ghost_group.count, 4,
            "ASM loc_124BE stores the raw count byte, and sub_F730/loc_F90A clear exactly four vertical cells"
        );
        assert!(ghost_group.vertical);
        assert!(
            !ghost_group.solid,
            "ASM sub_F730 should have vanished this Crypt group before issue #230 frame 298"
        );

        let grid = runtime
            .block_grid
            .as_ref()
            .expect("The Crypt should have a generated block grid");
        for row in 0x44..=0x47 {
            let cell = grid.cell_at(0x1D, row);
            assert!(
                !cell.has_block,
                "ASM loc_F90A should clear the SolidGhost block bit for Crypt cell (29,{row})"
            );
            assert_eq!(
                cell.collision,
                terrain::CollisionKind::Empty,
                "ASM loc_F90A should clear collision for Crypt cell (29,{row})"
            );
        }
        let support = grid.cell_at(0x1D, 0x48);
        assert_eq!(
            support.collision,
            terrain::CollisionKind::Solid,
            "ASM DBF count should leave row $48 as solid floor under the vanished Crypt ghost group"
        );
        assert!(
            !support.has_block,
            "row $48 is ordinary terrain support, not part of the SolidGhost group"
        );
    }

    #[test]
    fn issue230_crypt_walk_right_crosses_vanished_ghost_column_loc_124a6_f730() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let Some((level, script)) = cached_issue230_crypt38_walk_right_script() else {
            return;
        };

        let trace = ScriptRunner::new(manifest)
            .warp_to_level(level.0)
            .settle_recording_warp()
            .run(&script, script.len());

        let frame_298 = trace
            .get(298)
            .expect("issue #230 route should include frame 298");
        assert_eq!(frame_298.current_level, LevelId(0x38));
        assert_eq!(
            frame_298.kid_x_pos,
            Some(0x01D1_FB00),
            "oracle tas-diff frame 298: ASM sub_F730 has already cleared the SolidGhost wall at col $1D"
        );
        assert_eq!(frame_298.kid_y_pos, Some(0x047F_0000));
        assert_eq!(frame_298.kid_x_velocity, Some(0x0002_5B00));
        assert_eq!(frame_298.character_movement, Some(2));

        let frame_305 = trace
            .get(305)
            .expect("issue #230 route should include frame 305");
        assert_eq!(
            frame_305.kid_x_pos,
            Some(0x01E0_0F00),
            "ASM loc_124A6/sub_F730 count parity must let Kid cross the vanished ghost column instead of stopping or falling early"
        );
        assert_eq!(frame_305.kid_y_pos, Some(0x047F_0000));
        assert_eq!(frame_305.kid_y_velocity, Some(0));
        assert_eq!(frame_305.character_movement, Some(2));
    }

    #[test]
    fn issue289_level07_settled_recording_primes_island_shore_vram_loc_1202a() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let mut runtime = NativeRuntime::with_generated_level_layout(manifest);
        runtime.boot_to_level(LevelId(0x07));
        runtime.settle_recording_oracle_warp_live_window(LevelId(0x07));

        let mut frame_60 = None;
        let mut frame_71 = None;
        for frame_index in 0usize..=71 {
            let trace = runtime.step_frame(FrameInput::default());
            match frame_index {
                60 => frame_60 = Some(trace),
                71 => frame_71 = Some(trace),
                _ => {}
            }
        }

        let frame_60 = frame_60.expect("frame 60 should be captured");
        assert_eq!(
            frame_60.time_frames_loc_3038a, 174,
            "ASM loc_3038A Time_Frames already matches the recording oracle; loc_1202A shore priming must not advance the global timer"
        );
        assert_eq!(
            (
                frame_60.level_timer_seconds,
                frame_60.level_timer_sub_frames
            ),
            (178, 0x07),
            "ASM Make_SpriteAttr_HUD loc_BE82 decrements Time_SubSeconds before loc_BEE8 writes the visible timer; #289 oracle watches frame 60 at 2:58/$07"
        );
        assert_eq!(
            frame_60.island_shore_vram_frames_loc_1202a,
            Some((3, 2, 1)),
            "ASM loc_12098/loc_12110/loc_12188: oracle VDP samples for #289 frame 60 show the primary/mid bands at 3/2 and the lower shore band at frame 1"
        );

        let frame_71 = frame_71.expect("frame 71 should be captured");
        assert_eq!(
            frame_71.time_frames_loc_3038a, 185,
            "ASM loc_3038A remains on the oracle global timer at the second #289 checkpoint"
        );
        assert_eq!(
            (
                frame_71.level_timer_seconds,
                frame_71.level_timer_sub_frames
            ),
            (177, 0x38),
            "ASM Make_SpriteAttr_HUD loc_BE82/loc_BEE8: #289 oracle watches frame 71 after the next visible timer rollover at 2:57/$38"
        );
        assert_eq!(
            frame_71.island_shore_vram_frames_loc_1202a,
            Some((0, 3, 1)),
            "ASM loc_1202A: #289 frame 71 crosses the next primary/mid shore phases while the lower band remains on the oracle VDP frame"
        );
    }

    #[test]
    fn ghost_block_step_sub_f730_zero_state_starts_vanish_lead_in_and_restores_terrain_skin() {
        // ASM sub_F730/loc_F8A6: state 0 is the solid `$E304` hold. On timer
        // expiry it advances to `$8001` and restores the original
        // Level_terrain_layout byte as a no-collision Level_Layout word.
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 8 * 8 * 2];
        let ghost_offset = (5 * 8 + 3) * 2;
        data[ghost_offset] = 0xE3;
        data[ghost_offset + 1] = 0x04;
        let mut erase_words = vec![0u16; 8 * 8];
        erase_words[5 * 8 + 3] = 0x6005;
        let grid = terrain::BlockGrid::from_level_layout_with_erase_words_loc_11542(
            &data,
            erase_words,
            8,
            8,
        );
        rt.block_grid = Some(grid);
        rt.block_state.ghost_groups.push(GhostGroup {
            col: 3,
            row: 5,
            count: 1,
            vertical: false,
            solid_time: 10,
            vanished_time: 5,
            timer: 1,
            state: 0,
            solid: true,
        });

        rt.step_ghost_blocks();

        assert!(!rt.block_state.ghost_groups[0].solid);
        assert_eq!(rt.block_state.ghost_groups[0].state, 0x8001);
        assert_eq!(rt.block_state.ghost_groups[0].timer, 2);
        let cell = rt.block_grid.as_ref().unwrap().cell_at(3, 5);
        assert!(
            !cell.has_block,
            "vanished ghost block should clear grid cell"
        );
        assert_eq!(
            cell.collision,
            terrain::CollisionKind::Empty,
            "ASM loc_F8A6 restores terrain skin without collision"
        );
        assert_eq!(cell.skin, 0x05);
    }

    #[test]
    fn ghost_block_step_sub_f730_final_flash_restores_solid_grid() {
        // ASM sub_F730/loc_F7AA: state `$8005` expires to state 0 and writes
        // `$E304` back for the solid hold.
        let mut rt = NativeRuntime::new(manifest());
        let grid = grid_with_cells(8, 8, &[]);
        rt.block_grid = Some(grid);
        rt.block_state.ghost_groups.push(GhostGroup {
            col: 3,
            row: 5,
            count: 1,
            vertical: false,
            solid_time: 10,
            vanished_time: 5,
            timer: 1,
            state: 0x8005,
            solid: false,
        });

        rt.step_ghost_blocks();

        assert!(rt.block_state.ghost_groups[0].solid);
        assert_eq!(rt.block_state.ghost_groups[0].state, 0);
        assert_eq!(rt.block_state.ghost_groups[0].timer, 10);
        let cell = rt.block_grid.as_ref().unwrap().cell_at(3, 5);
        assert!(cell.has_block, "restored ghost block should have has_block");
        assert_eq!(cell.block_type, terrain::BlockType::SolidGhost);
        assert!(matches!(cell.collision, terrain::CollisionKind::Solid));
    }

    #[test]
    fn ghost_block_step_decrements_timer_without_transition() {
        // When timer > 0, should just decrement — no transition, no grid change.
        let mut rt = NativeRuntime::new(manifest());
        let grid = grid_with_cells(8, 8, &[(3, 5, 0xE3, 0)]);
        rt.block_grid = Some(grid);
        rt.block_state.ghost_groups.push(GhostGroup {
            col: 3,
            row: 5,
            count: 1,
            vertical: false,
            solid_time: 10,
            vanished_time: 5,
            timer: 3,
            state: 0,
            solid: true,
        });

        rt.step_ghost_blocks();

        assert!(
            rt.block_state.ghost_groups[0].solid,
            "should still be solid"
        );
        assert_eq!(rt.block_state.ghost_groups[0].timer, 2);
        // Grid should be unchanged
        let cell = rt.block_grid.as_ref().unwrap().cell_at(3, 5);
        assert!(cell.has_block, "grid should be unchanged during countdown");
    }

    #[test]
    fn ghost_block_vertical_group_sub_f730_vanish_clears_correct_cells() {
        // Vertical group of 3 blocks starting at (2, 4)
        let mut rt = NativeRuntime::new(manifest());
        let grid = grid_with_cells(8, 8, &[(2, 4, 0xE3, 0), (2, 5, 0xE3, 0), (2, 6, 0xE3, 0)]);
        rt.block_grid = Some(grid);
        rt.block_state.ghost_groups.push(GhostGroup {
            col: 2,
            row: 4,
            count: 3,
            vertical: true,
            solid_time: 10,
            vanished_time: 5,
            timer: 1,
            state: 0,
            solid: true,
        });

        rt.step_ghost_blocks();

        let grid = rt.block_grid.as_ref().unwrap();
        for row in 4..=6 {
            let cell = grid.cell_at(2, row);
            assert!(
                !cell.has_block,
                "vertical ghost cell at row {row} should be cleared"
            );
        }
    }

    // --- Shifting blocks ---

    #[test]
    fn shifting_block_no_op_when_airborne() {
        let mut rt = NativeRuntime::new(manifest());
        // Shifting block at (3, 5)
        let grid = grid_with_cells(8, 8, &[(3, 5, 0xE7, 0)]);
        rt.block_grid = Some(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.on_ground = false;
        slice.player.x_pos = 56 << 16;
        slice.player.y_pos = 79 << 16;
        rt.boot_state = BootState::InGame;
        rt.opening_slice = Some(slice);

        rt.step_frame(FrameInput::default());

        // Block should not have moved
        let grid = rt.block_grid.as_ref().unwrap();
        assert!(
            grid.cell_at(3, 5).has_block,
            "block should remain when player airborne"
        );
    }

    #[test]
    fn shifting_block_no_op_at_row_zero() {
        // A shifting block at row 0 has nowhere to shift up to.
        let mut rt = NativeRuntime::new(manifest());
        // Shifting block at (3, 0), solid floor at row 1
        let grid = grid_with_cells(8, 4, &[(3, 0, 0xE7, 0), (3, 1, 0x60, 0)]);
        rt.block_grid = Some(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.on_ground = true;
        // Standing on row 0: feet at y = pixel_to_cell boundary (row 0 top = 0px, +1 = 1)
        slice.player.x_pos = 56 << 16;
        slice.player.y_pos = 0 << 16; // at pixel 0 — foot_row = pixel_to_cell(1) = 0
        slice.terrain.floor_y_pos = 0;
        rt.boot_state = BootState::InGame;
        rt.opening_slice = Some(slice);

        rt.step_frame(FrameInput::default());

        // Block should still be there
        assert!(
            rt.block_grid.as_ref().unwrap().cell_at(3, 0).has_block,
            "shifting block at row 0 should not move"
        );
    }

    // --- Deferred ceiling clears: full lifecycle ---

    #[test]
    fn deferred_ceiling_clear_full_lifecycle() {
        // ASM sub_1007A: Deleted(32 frames) → Restored(1 frame) → PendingClear → FreshSmash → removed
        let mut rt = NativeRuntime::new(manifest());
        // Set up a grid with a prize block that has been hit (written as 0x6F = Solid+Deleted)
        let grid = grid_with_cells(8, 8, &[(3, 2, 0x6F, 0)]);
        rt.block_grid = Some(grid);

        let restore_cell = terrain::Cell::from_be_bytes(0xE1, 0x05); // original prize cell
        rt.block_state
            .deferred_ceiling_clears
            .push(DeferredCeilingClear {
                col: 3,
                row: 2,
                content: terrain::PrizeContent::Cyclone,
                restore_cell,
                reveal_only: false,
                deleted_frames_remaining: 2, // shortened for test
                phase: DeferredPrizeBlockPhase::Deleted,
            });

        // Tick 1: still Deleted, frames_remaining → 1
        rt.step_deferred_ceiling_clears();
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::Deleted
        );
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].deleted_frames_remaining,
            1
        );

        // Tick 2: still Deleted, frames_remaining → 0
        rt.step_deferred_ceiling_clears();
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::Deleted
        );
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].deleted_frames_remaining,
            0
        );

        // Tick 3: frames_remaining == 0, transition to Restored, grid gets restore_cell
        rt.step_deferred_ceiling_clears();
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::Restored
        );
        let cell = rt.block_grid.as_ref().unwrap().cell_at(3, 2);
        assert_eq!(cell, restore_cell, "grid should have the restore cell");

        // ASM sub_1007A allocates both loc_102B8 and Prize_Init after the
        // restored-cell wake. PrizeHelmetCyclone_Init then hibernates before
        // its first sub_10C38 update, so spawn at the frame's finisher.
        assert!(rt.floating_items.is_empty());
        rt.finish_deferred_ceiling_clears();
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::FreshSmash
        );
        assert_eq!(rt.floating_items.len(), 1, "should spawn floating item");
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES
        );
        // Grid should now have Smash block ($E001)
        let cell = rt.block_grid.as_ref().unwrap().cell_at(3, 2);
        assert!(cell.has_block);
        assert_eq!(cell.block_type, terrain::BlockType::Smash);

        // Next finish_deferred_ceiling_clears: FreshSmash → removed
        rt.finish_deferred_ceiling_clears();
        assert!(
            rt.block_state.deferred_ceiling_clears.is_empty(),
            "FreshSmash should be cleaned up"
        );
    }

    #[test]
    fn deferred_diamond_ceiling_clear_spawns_on_restored_wake_prizediamond_init() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_blocks());
        rt.block_state
            .deferred_ceiling_clears
            .push(DeferredCeilingClear {
                col: 3,
                row: 2,
                content: terrain::PrizeContent::Diamond,
                restore_cell: terrain::Cell::from_be_bytes(0xE0, 0x00),
                reveal_only: false,
                deleted_frames_remaining: 0,
                phase: DeferredPrizeBlockPhase::Deleted,
            });

        rt.step_deferred_ceiling_clears();

        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::PendingClearPrizeSpawned,
            "ASM PrizeDiamond_Init uses `sf is_moved(a3)`, so the diamond enters the live object roster on the restored-cell wake"
        );
        assert_eq!(rt.floating_items.len(), 1);
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_DIAMOND_LIFETIME_FRAMES
        );

        rt.finish_deferred_ceiling_clears();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "finish_deferred_ceiling_clears should write the Smash cell without spawning a duplicate diamond"
        );
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::FreshSmash
        );
    }

    #[test]
    fn deferred_disguised_prize_reveals_visible_block_without_spawning_loc_101d8() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_blocks());
        rt.block_state
            .deferred_ceiling_clears
            .push(DeferredCeilingClear {
                col: 2,
                row: 2,
                content: terrain::PrizeContent::Eyeclops,
                restore_cell: terrain::Cell::from_be_bytes(0xE1, 0x01),
                reveal_only: true,
                deleted_frames_remaining: 0,
                phase: DeferredPrizeBlockPhase::Deleted,
            });

        rt.step_deferred_ceiling_clears();
        assert_eq!(
            rt.block_state.deferred_ceiling_clears[0].phase,
            DeferredPrizeBlockPhase::PendingClear,
            "ASM loc_101D8 should skip Prize_Init and move straight to the visible-block write"
        );

        rt.finish_deferred_ceiling_clears();

        assert!(
            rt.block_state.deferred_ceiling_clears.is_empty(),
            "loc_101D8 deletes the worker after writing the visible prize block"
        );
        assert!(
            rt.floating_items.is_empty(),
            "loc_101D8 should reveal `$E102` without spawning the prize object"
        );
        assert_eq!(
            rt.block_grid.as_ref().unwrap().cell_at(2, 2),
            terrain::Cell::from_be_bytes(0xE1, 0x02),
            "ASM loc_101D8 tail writes visible `$E102`, requiring a second hit to pay"
        );
    }

    #[test]
    fn deferred_ankh_ceiling_clear_uses_sub_10c78_lifetime() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_blocks());
        rt.block_state
            .deferred_ceiling_clears
            .push(DeferredCeilingClear {
                col: 3,
                row: 2,
                content: terrain::PrizeContent::Ankh,
                restore_cell: terrain::Cell::from_be_bytes(0xE0, 0x0B),
                reveal_only: false,
                deleted_frames_remaining: 0,
                phase: DeferredPrizeBlockPhase::PendingClear,
            });

        rt.finish_deferred_ceiling_clears();

        assert_eq!(rt.floating_items.len(), 1);
        // ASM sub_10C78 initialises d3 as `$0001_0320`, so non-diamond fallers
        // keep the bit-16 first-row grace flag and an 0x320-frame lifetime.
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES
        );
        assert!(rt.floating_items[0].first_row_grace);
    }

    #[test]
    fn deferred_helmet_ceiling_clear_replaces_existing_helmet_via_loc_10cf4_scan() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_blocks());
        rt.floating_items.push(FloatingItem {
            x_pos: 0x0070_0000,
            y_pos: 0x0080_0000,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 7,
            level_row: 8,
            first_row_grace: true,
            content: terrain::PrizeContent::Skycutter,
            collected: false,
            collision_latched: false,
        });
        rt.floating_items.push(FloatingItem {
            x_pos: 0x0030_0000,
            y_pos: 0x0040_0000,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 3,
            level_row: 4,
            first_row_grace: true,
            content: terrain::PrizeContent::Clock,
            collected: false,
            collision_latched: false,
        });
        rt.block_state
            .deferred_ceiling_clears
            .push(DeferredCeilingClear {
                col: 5,
                row: 2,
                content: terrain::PrizeContent::Cyclone,
                restore_cell: terrain::Cell::from_be_bytes(0xE0, 0x03),
                reveal_only: false,
                deleted_frames_remaining: 0,
                phase: DeferredPrizeBlockPhase::PendingClear,
            });

        rt.finish_deferred_ceiling_clears();

        assert_eq!(
            rt.floating_items.len(),
            2,
            "ASM loc_10CF4 should keep scanning until it finds the older helmet owner, deleting only that helmet before the new one spawns"
        );
        assert!(
            rt.floating_items
                .iter()
                .all(|item| item.content != terrain::PrizeContent::Skycutter),
            "the older floating helmet should be removed"
        );
        assert!(
            rt.floating_items
                .iter()
                .any(|item| item.content == terrain::PrizeContent::Clock),
            "non-helmet prize items should survive the helmet cleanup"
        );
        assert!(
            rt.floating_items
                .iter()
                .any(|item| item.content == terrain::PrizeContent::Cyclone),
            "the newly spawned helmet should still appear"
        );
        assert!(
            rt.block_destroy_effects
                .iter()
                .any(|effect| effect.x_px == 0x0070 && effect.y_px == 0x0078),
            "ASM loc_10CF4 should hand sub_10CEC the matched helmet slot so the follow-on effect spawns at the deleted helmet's x, y-8"
        );
    }

    #[test]
    fn deferred_helmet_ceiling_clear_without_existing_helmet_falls_through_loc_10d38() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_blocks());
        rt.floating_items.push(FloatingItem {
            x_pos: 0x0030_0000,
            y_pos: 0x0040_0000,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 3,
            level_row: 4,
            first_row_grace: true,
            content: terrain::PrizeContent::Clock,
            collected: false,
            collision_latched: false,
        });
        rt.block_state
            .deferred_ceiling_clears
            .push(DeferredCeilingClear {
                col: 5,
                row: 2,
                content: terrain::PrizeContent::Cyclone,
                restore_cell: terrain::Cell::from_be_bytes(0xE0, 0x03),
                reveal_only: false,
                deleted_frames_remaining: 0,
                phase: DeferredPrizeBlockPhase::PendingClear,
            });

        rt.finish_deferred_ceiling_clears();

        assert!(
            rt.block_destroy_effects.is_empty(),
            "ASM loc_10D38 should be the no-match return path from sub_10CEC, so spawning a helmet with no older helmet owner must not create a cleanup effect",
        );
        assert!(
            rt.floating_items
                .iter()
                .any(|item| item.content == terrain::PrizeContent::Clock),
            "non-helmet prizes should survive the no-match scan"
        );
        assert!(
            rt.floating_items
                .iter()
                .any(|item| item.content == terrain::PrizeContent::Cyclone),
            "the requested helmet should still spawn after the loc_10D38 return path"
        );
    }

    #[test]
    fn ledge_snap_cooldown_start_queues_pullup_sfx_loc_abf0() {
        let mut rt = NativeRuntime::new(manifest());

        rt.queue_kid_pullup_sound_if_started_loc_abf0(0, LEDGE_SNAP_PULLUP_FRAMES);

        assert_eq!(
            rt.played_sound_effects,
            vec![QueuedSoundEffect::KidPullup],
            "ASM kid_flip loc_ABF0 (kid.asm:13464) plays sfx_The_Kid_pullup=$64 immediately after Init_Animation(stru_8BB4)"
        );
    }

    #[test]
    fn ledge_snap_cooldown_continue_does_not_repeat_pullup_sfx_loc_abf0() {
        let mut rt = NativeRuntime::new(manifest());

        rt.queue_kid_pullup_sound_if_started_loc_abf0(
            LEDGE_SNAP_PULLUP_FRAMES,
            LEDGE_SNAP_PULLUP_FRAMES - 1,
        );

        assert!(
            rt.played_sound_effects.is_empty(),
            "ASM loc_ABF0 plays the pull-up cue only when the stru_8BB4 animation starts, not on hibernated cooldown frames"
        );
    }

    // --- Process block effects ---

    /// ASM `loc_110D0` (kid.asm:23019) drives the metal-block ±8 y_pos arc via
    /// `BlockBounceAnim`; ASM `sub_10DE4` (kid.asm:22717) drives the rubber
    /// block through a skin-swap coroutine without ever touching `y_pos(a3)`,
    /// so native must NOT also push a BlockBounceAnim entry for rubber.
    #[test]
    fn process_block_effects_metal_pushes_bounce_rubber_does_not_push_bounce() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.metal_block_hit = Some((5, 3, NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE));
        slice.rubber_block_hit = Some((7, 4));

        rt.process_block_effects(&mut slice);

        assert_eq!(
            rt.block_bounces.len(),
            1,
            "only the metal block contributes a y_pos bounce; rubber must not duplicate it"
        );
        assert_eq!(rt.block_bounces[0].cell_x, 5);
        assert_eq!(rt.block_bounces[0].cell_y, 3);
        assert_eq!(
            rt.blocks_hit_this_frame, 0,
            "#242: both metal (loc_110D0 -> sfx_Metal_block) and rubber (loc_10DA4 -> sfx_Rubber_block) play their own SFX directly; neither may feed the generic sfx_Block_hit headbutt counter (which played the brick-break crunch on bounces)"
        );
        assert_eq!(
            rt.played_sound_effects,
            vec![
                QueuedSoundEffect::MetalBlock,
                QueuedSoundEffect::RubberBlock
            ],
            "ASM loc_110D0 (kid.asm:23021) plays sfx_Metal_block ($12) for the springy bounce and loc_10DA4 (kid.asm:22692) plays sfx_Rubber_block ($56); both are queued from core in block-effect order"
        );
        assert_eq!(
            rt.played_sound_commands,
            vec![
                QueuedSoundCommand::StartSequence(QueuedSoundEffect::MetalBlock),
                QueuedSoundCommand::StartSequence(QueuedSoundEffect::RubberBlock)
            ],
            "native live playback consumes the ASM-backed StartSequence command trace, not only the legacy sound-id list"
        );
    }

    // --- RockTank drive SFX stop (#339) ---

    #[test]
    fn loc_3c46e_last_rock_tank_exit_emits_stopsequence_tank_driving_stop() {
        // ASM loc_3C46E -> loc_3C4A8 (kid.asm:43965-43997): the shared
        // deactivation routine `subi.b #1`s the $FFFFFA34 drive-sound refcount
        // as a Rock Tank leaves, then replays byte_3C46C[1] = sfx_Tank_driving
        // through j_PlaySound2 only when that decrement reaches zero. Both
        // exits route here — scroll-off via loc_3C436 and death via Tank.asm
        // loc_3B480 after Hibernate_UntilAnimFinished. #339: the stop was
        // unported, so the drive drone looped forever once the last tank left.
        let mut rt = NativeRuntime::new(manifest());
        let mut tank = make_pacer(EnemyFamily::Tank, 3, 0x100, 0x80);
        // The drive SFX is currently in the pooled audible refcount.
        tank.shared_walker_patrol_short_next = true;
        rt.active_enemies.push(tank);
        let driving_before = rt.driving_rock_tank_count();
        assert_eq!(
            driving_before, 1,
            "a latched Rock Tank counts toward the pooled $FFFFFA34 drive refcount"
        );

        // This frame's activations/death-removals drop it from the active list.
        rt.active_enemies.clear();
        rt.stop_rock_tank_drive_sound_on_last_exit_loc_3c46e(driving_before);

        assert_eq!(
            rt.played_sound_commands,
            vec![QueuedSoundCommand::StopSequence(
                QueuedSoundEffect::TankDriving
            )],
            "ASM loc_3C4A8: refcount 1->0 must stop sfx_Tank_driving via StopSequence ($12 cmdstopseq), not StartSequence ($10)"
        );
    }

    #[test]
    fn loc_3c4a8_drive_stop_holds_while_another_rock_tank_remains() {
        // ASM loc_3C4A8: `subi.b #1,(a4,d7.w); bne.s loc_3C4C0` — when the
        // decrement leaves the refcount nonzero, the j_PlaySound2 stop is
        // skipped. Native pools both RockTank variants into one count
        // (byte_3C46C[1] is shared), so a surviving tank holds the drone.
        let mut rt = NativeRuntime::new(manifest());
        let mut tank = make_pacer(EnemyFamily::Tank, 3, 0x100, 0x80);
        tank.shared_walker_patrol_short_next = true;
        let mut shooting_tank = make_pacer(EnemyFamily::TankShooting, 3, 0x200, 0x80);
        shooting_tank.shared_walker_patrol_short_next = true;
        rt.active_enemies.push(tank);
        rt.active_enemies.push(shooting_tank);
        let driving_before = rt.driving_rock_tank_count();
        assert_eq!(
            driving_before, 2,
            "RockTank and RockTank_shooting pool into one drive refcount (byte_3C46C[1])"
        );

        rt.active_enemies.remove(0); // one tank leaves; one remains driving
        rt.stop_rock_tank_drive_sound_on_last_exit_loc_3c46e(driving_before);

        assert!(
            rt.played_sound_commands.is_empty(),
            "ASM loc_3C4A8: refcount 2->1 stays nonzero, so no StopSequence stop fires"
        );
        assert_eq!(
            rt.driving_rock_tank_count(),
            1,
            "the surviving Rock Tank keeps the pooled drive refcount at one"
        );
    }

    #[test]
    fn loc_3c4a8_drive_stop_fires_once_when_two_rock_tanks_leave_together() {
        let mut rt = NativeRuntime::new(manifest());
        let mut tank = make_pacer(EnemyFamily::Tank, 3, 0x100, 0x80);
        tank.shared_walker_patrol_short_next = true;
        let mut shooting_tank = make_pacer(EnemyFamily::TankShooting, 3, 0x200, 0x80);
        shooting_tank.shared_walker_patrol_short_next = true;
        rt.active_enemies.push(tank);
        rt.active_enemies.push(shooting_tank);
        let driving_before = rt.driving_rock_tank_count();

        rt.active_enemies.clear();
        rt.stop_rock_tank_drive_sound_on_last_exit_loc_3c46e(driving_before);

        assert_eq!(
            rt.played_sound_commands,
            vec![QueuedSoundCommand::StopSequence(
                QueuedSoundEffect::TankDriving
            )],
            "ASM loc_3C4A8 should decrement the shared RockTank refcount once per exiting object and stop the pooled drive loop only on the final 1->0 decrement"
        );
    }

    #[test]
    fn loc_3c46e_drive_stop_silent_when_no_rock_tank_was_driving() {
        // No driving Rock Tank last frame => nothing to decrement, so the
        // deactivation routine never reaches the j_PlaySound2 stop.
        let mut rt = NativeRuntime::new(manifest());
        rt.stop_rock_tank_drive_sound_on_last_exit_loc_3c46e(0);
        assert!(
            rt.played_sound_commands.is_empty(),
            "loc_3C46E only plays the stop when a previously-driving tank reaches refcount zero"
        );
    }

    #[test]
    fn driving_rock_tank_count_excludes_unlatched_tanks_and_other_families() {
        // The pooled drive refcount mirrors the live sub_3C352 membership:
        // only Rock Tanks whose drive SFX is currently audible count, and
        // other pacer families never touch byte_3C46C[1].
        let mut rt = NativeRuntime::new(manifest());
        // Spawn-default latch is false for Tank (only DragonWalking defaults
        // true), so this tank has not yet started its drive SFX.
        rt.active_enemies
            .push(make_pacer(EnemyFamily::Tank, 3, 0x100, 0x80));
        rt.active_enemies
            .push(make_pacer(EnemyFamily::Crab, 3, 0x140, 0x80));

        assert_eq!(
            rt.driving_rock_tank_count(),
            0,
            "only latched Rock Tanks count as driving; un-started tanks and non-tank families do not"
        );
    }

    #[test]
    fn load_ingame_stops_live_rock_tank_drive_drone() {
        // ASM Load_InGame (kid.asm:23623) j_StopMusic resets the GEMS
        // driver on every level (re)entry, silencing the still-driving Rock Tank
        // drone. #339 REOPEN: the per-exit edge stop is bypassed by the
        // death-respawn / level-skip bulk clears, so this mirror stops the drone
        // for any tank still live at the teardown.
        let mut rt = NativeRuntime::new(manifest());
        let mut tank = make_pacer(EnemyFamily::Tank, 3, 0x100, 0x80);
        tank.shared_walker_patrol_short_next = true;
        rt.active_enemies.push(tank);

        rt.stop_level_looping_drones_on_load_ingame();

        assert_eq!(
            rt.played_sound_commands,
            vec![QueuedSoundCommand::StopSequence(
                QueuedSoundEffect::TankDriving
            )],
            "Load_InGame j_StopMusic must stop the live Rock Tank drive drone via StopSequence ($12 cmdstopseq)"
        );
    }

    #[test]
    fn load_ingame_drone_stop_silent_when_no_drone_active() {
        // With no looping drone live, the j_StopMusic mirror must stay silent so
        // a level (re)entry never queues a spurious stop.
        let mut rt = NativeRuntime::new(manifest());
        rt.active_enemies
            .push(make_pacer(EnemyFamily::Tank, 3, 0x100, 0x80));

        rt.stop_level_looping_drones_on_load_ingame();

        assert!(
            rt.played_sound_commands.is_empty(),
            "an un-latched tank (no drive SFX started) and no active clock must leave Load_InGame silent"
        );
    }

    #[test]
    fn load_ingame_level_skip_stops_live_rock_tank_drive_drone() {
        // ASM Load_InGame j_StopMusic runs as the skip target loads;
        // the skip teardown branch of refresh_active_roster must stop the prior
        // level's live drone so it does not re-accumulate across skips. #339.
        let mut rt = NativeRuntime::new(manifest());
        let mut tank = make_pacer(EnemyFamily::Tank, 3, 0x100, 0x80);
        tank.shared_walker_patrol_short_next = true;
        rt.active_enemies.push(tank);
        rt.level_skip_resume_hold = true;
        rt.played_sound_commands.clear();

        rt.refresh_active_roster();

        assert!(
            rt.played_sound_commands
                .contains(&QueuedSoundCommand::StopSequence(
                    QueuedSoundEffect::TankDriving
                )),
            "the level-skip teardown must mirror Load_InGame j_StopMusic and stop the live Rock Tank drone"
        );
    }

    #[test]
    fn iron_knight_metal_block_bounce_uses_loc_1117a_down_then_up_arc() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.metal_block_hit = Some((5, 3, NativeRuntime::BLOCK_DESTROY_DIRECTION_BELOW));

        rt.process_block_effects(&mut slice);
        assert_eq!(
            rt.block_bounces[0].metal_direction,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_BELOW,
            "ASM sub_AF7A stores d6=0 into loc_110D0's `$4C(a5)` field for Iron Knight metal hits"
        );

        rt.step_visual_effects();
        assert_eq!(
            rt.block_bounces[0].y_offset_px, 2,
            "ASM sub_11120 loc_1117A: `$4C(a5)==0` takes the first addq.w #2 y_pos frame"
        );
    }

    #[test]
    fn metal_block_bounce_live_runtime_uses_both_sub_11120_phase_tables() {
        for (direction, expected_deltas) in [
            (
                NativeRuntime::BLOCK_DESTROY_DIRECTION_BELOW,
                blocks::METAL_BLOCK_BOUNCE_Y_DELTAS_LOC_1117A,
            ),
            (
                NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE,
                blocks::METAL_BLOCK_BOUNCE_Y_DELTAS_LOC_11196,
            ),
        ] {
            let mut rt = NativeRuntime::new(manifest());
            let mut slice = OpeningSliceState::new(default_bounds());
            slice.metal_block_hit = Some((5, 3, direction));

            rt.process_block_effects(&mut slice);

            let mut expected_offset = 0i16;
            for (frame_index, delta) in expected_deltas.into_iter().enumerate() {
                rt.step_visual_effects();
                expected_offset += delta;
                if frame_index < 7 {
                    assert_eq!(
                        rt.block_bounces[0].y_offset_px, expected_offset,
                        "ASM sub_11120 live metal bounce frame {frame_index} should follow the `$4C(a5)`-selected loc_1117A/loc_11196 phase table",
                    );
                } else {
                    assert_eq!(
                        expected_offset, 0,
                        "ASM sub_11120 final hibernated phase step should settle the metal block back at its origin",
                    );
                    assert!(
                        rt.block_bounces.is_empty(),
                        "ASM sub_11120 should finish the metal-block bounce after the eighth hibernated phase step",
                    );
                }
            }
        }
    }

    #[test]
    fn metal_block_effect_cell_words_loc_110d0_keeps_tile_coordinates_unshifted() {
        assert_eq!(
            NativeRuntime::metal_block_effect_cell_words_loc_110D0(5, 3),
            (5, 3),
            "ASM loc_110D0 should cache the struck block's tile coordinates in `$44/$46` before any sprite-space scaling happens",
        );
    }

    #[test]
    fn metal_block_push_entry_loc_111f4_starts_the_right_scan_one_cell_ahead() {
        assert_eq!(
            NativeRuntime::metal_block_push_entry_loc_111F4(5, 3, 12),
            Some(6),
            "ASM loc_111F4 should pre-advance the layout pointer and column counter to the cell immediately right of the struck metal block when `d6 == 3`",
        );
    }

    #[test]
    fn metal_block_push_entry_loc_111f4_falls_back_to_the_left_scan_for_other_directions() {
        assert_eq!(
            NativeRuntime::metal_block_push_entry_loc_111F4(5, 1, 12),
            Some(4),
            "ASM loc_111F4 should treat non-`3` directions as the left-walking branch and pre-decrement the source column before `loc_11230` starts scanning",
        );
    }

    #[test]
    fn metal_block_push_entry_loc_111f4_aborts_the_right_scan_at_the_level_edge() {
        assert_eq!(
            NativeRuntime::metal_block_push_entry_loc_111F4(11, 3, 12),
            None,
            "ASM loc_111F4 should jump to `loc_11296` once the source block already sits on `Level_width_blocks-1`, because the right scan has no in-bounds cell to inspect",
        );
    }

    #[test]
    fn metal_block_push_scan_right_loc_11208_stops_on_the_last_contiguous_metal_block() {
        let row = [
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0x00, 0x00),
        ];

        assert_eq!(
            NativeRuntime::metal_block_push_scan_right_loc_11208(&row, 0),
            Some(2),
            "ASM loc_11208 should keep advancing across contiguous solid metal blocks and, via `loc_11256`, select the last matching block before the first non-solid cell",
        );
    }

    #[test]
    fn metal_block_push_scan_right_loc_11208_aborts_on_in_bounds_solid_non_metal() {
        let row = [
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE0, 0x01),
            terrain::Cell::from_be_bytes(0x00, 0x00),
        ];

        assert_eq!(
            NativeRuntime::metal_block_push_scan_right_loc_11208(&row, 0),
            None,
            "ASM loc_11208 should jump to `loc_11296` when the scan hits an in-bounds solid cell whose bit-15 / block-type test is not `$E5xx` metal",
        );
    }

    #[test]
    fn metal_block_push_scan_right_loc_11208_aborts_when_the_metal_run_reaches_the_right_edge() {
        // ASM loc_11208 `cmp.w d4,d1; bge.w loc_11296` (d4 = Level_width_blocks-1)
        // stops sampling once the index walks past the last column, so a metal
        // run that reaches the right edge without a non-solid terminator selects
        // no block (no `j_sub_E02` worker). The native bound is `current_col <
        // width`; a `<=` slip would index one cell past the row and panic.
        let row = [
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
        ];

        assert_eq!(
            NativeRuntime::metal_block_push_scan_right_loc_11208(&row, 0),
            None,
            "ASM loc_11208 should abort at `loc_11296` (selecting no block) when the contiguous metal run extends to the level's right edge",
        );
    }

    #[test]
    fn metal_block_push_scan_left_loc_11230_stops_on_the_last_contiguous_metal_block() {
        let row = [
            terrain::Cell::from_be_bytes(0x00, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
        ];

        assert_eq!(
            NativeRuntime::metal_block_push_scan_left_loc_11230(&row, 3),
            Some(1),
            "ASM loc_11230 should keep retreating across contiguous solid metal blocks and, via `loc_1125A`, select the last matching block before the first non-solid cell",
        );
    }

    #[test]
    fn metal_block_push_scan_left_loc_11230_aborts_when_the_chain_runs_into_column_zero() {
        let row = [
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
        ];

        assert_eq!(
            NativeRuntime::metal_block_push_scan_left_loc_11230(&row, 3),
            None,
            "ASM loc_11230 should jump to `loc_11296` once the contiguous metal run reaches column zero, because the left scan refuses to sample past the level edge",
        );
    }

    #[test]
    fn metal_block_push_scan_left_loc_11230_aborts_on_in_bounds_solid_non_metal() {
        // ASM loc_11230 confirms `$6000` collision, then `bclr #$F,d7;
        // beq.s loc_11296` and `andi.w #$F00,d7; cmpi.w #$500,d7; bne.s
        // loc_11296`: a solid cell that is a block but NOT `$500` metal aborts
        // the chain. That is `!has_block || block_type != Metal => None`; an
        // `&&` slip would walk past the obstruction onto the non-solid cell at
        // column zero and wrongly select it. cell 0 is left non-solid so the
        // faithful `None` and the mutated `Some(1)` are distinguishable.
        let row = [
            terrain::Cell::from_be_bytes(0x00, 0x00),
            terrain::Cell::from_be_bytes(0xE0, 0x01),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
            terrain::Cell::from_be_bytes(0xE5, 0x00),
        ];

        assert_eq!(
            NativeRuntime::metal_block_push_scan_left_loc_11230(&row, 3),
            None,
            "ASM loc_11230 should jump to `loc_11296` when the left scan hits an in-bounds solid block whose block-type test is not `$E5xx` metal",
        );
    }

    #[test]
    fn metal_block_push_mark_deleted_loc_1125c_preserves_collision_and_skin_bits() {
        assert_eq!(
            NativeRuntime::metal_block_push_mark_deleted_loc_1125C(terrain::Cell::from_be_bytes(
                0xE5, 0x0B
            )),
            terrain::Cell::from_be_bytes(0xEF, 0x0B),
            "ASM loc_1125C should apply `or.b #$F,(a2)`, turning the selected `$E50B` metal block into `$EF0B` without changing its collision or skin bits",
        );
    }

    #[test]
    fn metal_block_push_step_right_loc_112e2_reaches_the_neighbour_check_when_d3_hits_four() {
        let step = NativeRuntime::metal_block_push_step_right_loc_112E2(0x0050, 3);

        assert_eq!(
            step.x_pos, 0x0054,
            "ASM loc_112E2 should move the worker sprite 4 px right on each wake",
        );
        assert_eq!(step.substep_counter, 4);
        assert!(
            step.reached_neighbour_check,
            "ASM loc_112E2 should branch to `loc_11334` once the incremented `d3` reaches 4",
        );
    }

    #[test]
    fn metal_block_push_step_left_loc_112fe_reaches_the_neighbour_check_when_d3_hits_four() {
        let step = NativeRuntime::metal_block_push_step_left_loc_112FE(0x0050, 3);

        assert_eq!(
            step.x_pos, 0x004C,
            "ASM loc_112FE should move the worker sprite 4 px left on each wake",
        );
        assert_eq!(step.substep_counter, 4);
        assert!(
            step.reached_neighbour_check,
            "ASM loc_112FE should branch to `loc_1130C` once the incremented `d3` reaches 4",
        );
    }

    #[test]
    fn step_metal_block_push_workers_sub_1129c_moves_each_worker_in_its_own_direction() {
        // ASM loc_112E2 `cmpi.w #3,d6; bne.s loc_112FE`: direction 3 ($4C(a5)==3)
        // adds 4 px to x_pos(a3), every other direction subtracts 4. The
        // direction select gates the SIGN of the per-wake x_pos step; the
        // existing Berzerker integration test only asserts the column hand-off,
        // not x_pos, so a `== -> !=` flip there survives. substep_counter starts
        // at 0 so this single wake stays short of the `d3 == 4` neighbour check
        // and both workers survive with their x_pos updated.
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(terrain::BlockGrid::from_level_layout(&[0; 9 * 6 * 2], 9, 6));
        rt.block_state.metal_block_push_workers = vec![
            crate::types::MetalBlockPushWorker {
                col: 2,
                row: 2,
                direction: NativeRuntime::BLOCK_DESTROY_DIRECTION_RIGHT,
                substep_counter: 0,
                x_pos_px: 0x0050,
            },
            crate::types::MetalBlockPushWorker {
                col: 5,
                row: 4,
                direction: NativeRuntime::BLOCK_DESTROY_DIRECTION_LEFT,
                substep_counter: 0,
                x_pos_px: 0x0050,
            },
        ];

        rt.step_metal_block_push_workers_sub_1129C();

        assert_eq!(
            rt.block_state.metal_block_push_workers[0].x_pos_px, 0x0054,
            "ASM loc_112E2 should add 4 px to a direction-3 (right) worker's x_pos on each wake",
        );
        assert_eq!(
            rt.block_state.metal_block_push_workers[1].x_pos_px, 0x004C,
            "ASM loc_112FE should subtract 4 px from a non-3 (left) worker's x_pos on each wake",
        );
    }

    #[test]
    fn metal_block_push_left_delete_gate_loc_11328_deletes_at_column_zero_before_sampling_left() {
        assert!(
            NativeRuntime::metal_block_push_left_delete_gate_loc_11328(
                0,
                terrain::Cell::from_be_bytes(0x60, 0x05),
            ),
            "ASM loc_11328 should delete immediately when `tst.w d1; ble.s loc_11328` sees the worker already at column zero, without relying on the sampled left neighbour",
        );
    }

    #[test]
    fn metal_block_push_left_delete_gate_loc_11328_continues_only_while_the_left_neighbour_is_open()
    {
        assert!(
            NativeRuntime::metal_block_push_left_delete_gate_loc_11328(
                3,
                terrain::Cell::from_be_bytes(0x60, 0x00),
            ),
            "ASM loc_1130C should restore once the sampled left neighbour masks to `$6000`",
        );
        assert!(
            !NativeRuntime::metal_block_push_left_delete_gate_loc_11328(
                3,
                terrain::Cell::from_be_bytes(0x00, 0x00),
            ),
            "ASM loc_1130C should loop back to `loc_112E2` while the sampled left neighbour is not solid",
        );
    }

    #[test]
    fn metal_block_effect_sprite_pos_loc_1110e_scales_tile_words_into_pixels() {
        assert_eq!(
            NativeRuntime::metal_block_effect_sprite_pos_loc_1110E(5, 3),
            (80, 48),
            "ASM loc_1110E should mirror the cached `$44/$46` tile words into sprite-space with `lsl.w #4` for both axes",
        );
    }

    #[test]
    fn rubber_block_contact_runs_sub_10de4_skin_sequence() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_floor_at_row(10);
        grid.set_cell_at(7, 4, terrain::Cell::from_be_bytes(0xE6, 0x0C));
        rt.set_block_grid(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.rubber_block_hit = Some((7, 4));

        rt.process_block_effects(&mut slice);

        let skin =
            |runtime: &NativeRuntime| runtime.block_grid().expect("block grid").cell_at(7, 4).skin;

        assert_eq!(
            skin(&rt),
            0x0D,
            "ASM sub_10DE4 writes the `base+1` rubber frame on the contact frame"
        );

        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0D);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0C);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0D);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0C);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0D);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0D);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0C);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0C);
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0D);
        rt.step_visual_effects();
        rt.step_visual_effects();
        rt.step_visual_effects();
        assert_eq!(skin(&rt), 0x0C);
        rt.step_visual_effects();
        rt.step_visual_effects();
        rt.step_visual_effects();
        assert_eq!(
            skin(&rt),
            0x0C,
            "ASM sub_10DE4 finishes on the base rubber frame after the final 3-frame hibernate"
        );
    }

    #[test]
    fn rubber_block_contact_spanning_two_cells_allocates_two_sub_10de4_workers() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_floor_at_row(18);
        grid.set_cell_at(85, 16, terrain::Cell::from_be_bytes(0xE6, 0x0C));
        grid.set_cell_at(86, 16, terrain::Cell::from_be_bytes(0xE6, 0x0C));
        rt.set_block_grid(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 0x055c_0000;
        slice.rubber_block_hit = Some((85, 16));

        rt.process_block_effects(&mut slice);

        assert_eq!(
            rt.block_state.rubber_block_animations.len(),
            2,
            "ASM loc_10DA4/sub_10DE4 allocates one object-only rubber skin worker for each contacted rubber cell; natural Level 00 frame 778 spans cells 85 and 86"
        );
        assert!(
            rt.block_state
                .rubber_block_animations
                .iter()
                .any(|entry| entry.col == 85 && entry.row == 16)
        );
        assert!(
            rt.block_state
                .rubber_block_animations
                .iter()
                .any(|entry| entry.col == 86 && entry.row == 16)
        );
    }

    #[test]
    fn rubber_block_base_skin_for_sub_10de4_loc_10e06_keeps_the_low_animation_pair_for_0c_and_0d() {
        assert_eq!(
            NativeRuntime::rubber_block_base_skin_for_sub_10de4(terrain::Cell::from_be_bytes(
                0xE6, 0x0D,
            )),
            Some(0x0C),
            "ASM loc_10E06 should seed rubber skins `$0C/$0D` from base frame `$E60C` so the shared writer only toggles the low pair",
        );
    }

    #[test]
    fn rubber_block_contact_loc_10e0a_writes_the_high_pair_base_plus_one_before_the_first_hibernate()
     {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_floor_at_row(10);
        grid.set_cell_at(7, 4, terrain::Cell::from_be_bytes(0xE6, 0x0E));
        rt.set_block_grid(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.rubber_block_hit = Some((7, 4));

        rt.process_block_effects(&mut slice);

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(7, 4).skin,
            0x0F,
            "ASM loc_10E0A should increment the selected rubber base frame before the first `sub_11530` write, so the contact frame for the high pair is `$E60F` rather than `$E60E`",
        );
    }

    #[test]
    fn metal_block_push_right_delete_gate_loc_11358_deletes_at_the_right_boundary_before_sampling_right()
     {
        assert!(
            NativeRuntime::metal_block_push_right_delete_gate_loc_11358(
                7,
                8,
                terrain::Cell::from_be_bytes(0x60, 0x05),
            ),
            "ASM loc_11358 should delete immediately when `cmp.w Level_width_blocks-1,d1` reaches the last column, without relying on the sampled right neighbour",
        );
    }

    #[test]
    fn metal_block_push_right_delete_gate_loc_11358_continues_only_while_the_right_neighbour_is_open()
     {
        assert!(
            NativeRuntime::metal_block_push_right_delete_gate_loc_11358(
                3,
                8,
                terrain::Cell::from_be_bytes(0x60, 0x00),
            ),
            "ASM loc_11334 should restore once the sampled right neighbour masks to `$6000`",
        );
        assert!(
            !NativeRuntime::metal_block_push_right_delete_gate_loc_11358(
                3,
                8,
                terrain::Cell::from_be_bytes(0x00, 0x00),
            ),
            "ASM loc_11334 should loop back to `loc_112E2` while the sampled right neighbour is not solid",
        );
    }

    #[test]
    fn berzerker_charge_side_hit_starts_metal_block_push_loc_111f4() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.helmet.berzerker_charging = true;
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 9 * 6 * 2], 9, 6);
        grid.set_cell_at(5, 3, terrain::Cell::from_be_bytes(0xE5, 0x0B));
        grid.set_cell_at(6, 3, terrain::Cell::from_be_bytes(0xE5, 0x0B));
        grid.set_cell_at(8, 3, terrain::Cell::from_be_bytes(0x60, 0x00));
        rt.set_block_grid(grid);

        let mut slice = combat_slice();
        let half_width = PlayerForm::Berzerker.half_width();
        slice.player.x_pos = ((5 * 16 - half_width - 1) as u32) << 16;
        slice.player.y_pos = (3 * 16 + 8) << 16;
        slice.player.x_velocity = 0x0002_0000;
        slice.player.facing = Direction::Right;
        slice.player.on_ground = true;

        rt.berzerker_break_wall_block(&slice);

        assert!(
            !rt.helmet.berzerker_charging,
            "ASM loc_9326 should clear Berzerker_charging after the non-rubber wall scan"
        );
        assert_eq!(
            rt.block_state.metal_block_push_workers.len(),
            1,
            "ASM off_933E[5] should allocate sub_1129C for side-hit Metal blocks instead of treating them as inert"
        );
        assert_eq!(rt.block_state.metal_block_push_workers[0].col, 6);
        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(6, 3),
            terrain::Cell::from_be_bytes(0x60, 0x0B),
            "ASM loc_1125C/sub_1129C should erase the selected edge metal block before the moving sprite starts"
        );
        assert!(
            rt.played_sound_effects
                .contains(&QueuedSoundEffect::BerzerkerMovingBlock),
            "ASM sub_1129C should queue sfx_Berzerker_moving_block=$10"
        );

        rt.step_visual_effects();
        assert_eq!(
            rt.block_state.metal_block_push_workers[0].col, 7,
            "ASM loc_11334 should keep moving through a non-solid right neighbour"
        );
        for _ in 0..4 {
            rt.step_visual_effects();
        }
        assert!(
            rt.block_state.metal_block_push_workers.is_empty(),
            "ASM loc_11358 should restore and delete once the moving block reaches a solid right neighbour"
        );
        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(7, 3),
            terrain::Cell::from_be_bytes(0xE5, 0x0B),
            "ASM loc_11358 should restore `$E50B` at the final moving-block column"
        );
    }

    #[test]
    fn process_block_effects_smash_creates_destroy_effect() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.smash_block_hit = Some((80, 48, NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE));

        rt.process_block_effects(&mut slice);

        assert_eq!(rt.block_destroy_effects.len(), 1);
        assert_eq!(rt.block_destroy_effects[0].x_px, 80);
        assert_eq!(
            rt.block_destroy_effects[0].y_px, 44,
            "ASM sub_BB54 sets d6=2 for ordinary smash-block headbutts, so sub_10EDA should nudge the effect upward through loc_10F1A",
        );
        assert_eq!(rt.block_destroy_effects[0].remaining, 2);
        assert_eq!(
            rt.block_destroy_effects[0].direction,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE
        );
    }

    #[test]
    fn loc_bb6e_ceiling_ice_block_loads_bullets_loc_11052() {
        let mut runtime = NativeRuntime::new(manifest());

        let w: u16 = 10;
        let h: u16 = 10;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let idx = (2 * w + 3) as usize * 2;
        data[idx] = 0xE2;
        data[idx + 1] = 0x01;
        let mut grid = terrain::BlockGrid::from_level_layout(&data, w, h);

        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        slice.player.x_pos = 48 << 16;
        slice.player.y_pos = 65 << 16;
        slice.player.y_velocity = -0x000E_0000;

        slice.apply_ceiling_collision(
            Some(&mut grid),
            None,
            PlayerForm::TheKid,
            PLAYER_HALF_WIDTH,
            &[],
        );

        assert!(
            !grid.cell_at(3, 2).has_block,
            "ASM off_BB80[2] routes ceiling Ice through IceBlock_SmashAndLoadBullets, so the cell must be deleted"
        );
        assert_eq!(
            slice.ice_block_hit,
            Some((3, 2, NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE)),
            "ASM sub_BB54 sets d6=2 for normal ceiling hits before dispatching off_BB80[2]",
        );

        runtime.set_block_grid(grid);
        runtime.process_block_effects(&mut slice);

        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::PrizeBlock],
            "ASM IceBlock_SmashAndLoadBullets (kid.asm:22849) plays sfx_Prize_block before loading bullets and sub_10F9C"
        );
        assert_eq!(
            runtime
                .projectiles
                .iter()
                .map(|p| (
                    p.kind,
                    p.x_pos >> 16,
                    p.y_pos >> 16,
                    p.x_velocity,
                    p.y_velocity,
                ))
                .collect::<Vec<_>>(),
            vec![
                (EnemyProjectileKind::ShooterBlock, 56, 40, 0, -(4 << 16)),
                (EnemyProjectileKind::ShooterBlock, 52, 44, 0, -(4 << 16)),
                (EnemyProjectileKind::ShooterBlock, 60, 44, 0, -(4 << 16)),
            ],
            "ASM sub_BB54 sets d6=2 for normal ceiling hits, so IceBlock_LoadBullets remaps to upward bullets",
        );
    }

    #[test]
    fn ice_block_bullets_loc_11052_track_source_layout_cell_like_shooter_object_bullets() {
        // IceBlock_LoadBullets spawns ShooterObject-style bullets that scan and
        // track their source Level_Layout cell: native models them with the same
        // tracking fields as the canonical shooter-block bullet
        // `EnemyProjectile::shooter_block_loc_EC76` — `shooter_tracks_layout`
        // true, the layout cell stored, and the freshly-seeded
        // `shooter_state_word` `$FFFE` (the shooter slot state machine increments
        // it to `$FFFF` on its first live step at projectiles.rs:2034). These
        // fields are non-`Default` (the derived `Default` zeroes them), so pin
        // them so a dropped assignment cannot silently de-track the ice bullets.
        let source_col = 3;
        let source_row = 2;
        let bullets =
            NativeRuntime::ice_block_bullets_loc_11052(source_col, source_row, /* d6 */ 2);

        assert_eq!(bullets.len(), 3, "IceBlock_LoadBullets fires three bullets");
        for bullet in &bullets {
            assert_eq!(bullet.kind, EnemyProjectileKind::ShooterBlock);
            assert_eq!(bullet.lifetime, 120);
            assert!(
                bullet.shooter_tracks_layout,
                "ice bullets track their source Level_Layout cell like shooter-object bullets"
            );
            assert_eq!(bullet.shooter_layout_col, source_col);
            assert_eq!(bullet.shooter_layout_row, source_row);
            assert_eq!(
                bullet.shooter_state_word, 0xFFFE,
                "freshly-seeded shooter state word; first live step increments it to $FFFF"
            );
        }
    }

    #[test]
    fn process_block_effects_sub_1149c_queues_empty_support_workers() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_shooter_blocks(&[(5, 4, 0x02)]);
        grid.set_cell_at(6, 3, terrain::Cell::from_be_bytes(0x60, 0));
        rt.set_block_grid(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((5, 4));

        rt.process_block_effects(&mut slice);

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 4),
            terrain::Cell::from_be_bytes(0xE8, 0x02),
            "ASM loc_1146E branches to `loc_11496` once any `sub_1149C` probe sets `d4`, so the struck shooter cell should stay untouched on the support-worker path"
        );
        assert_eq!(rt.block_state.shooter_headbutt_workers.len(), 2);
        assert_eq!(
            rt.block_state
                .shooter_headbutt_workers
                .iter()
                .filter(|worker| worker.kind == ShooterHeadbuttWorkerKind::Center)
                .count(),
            0,
            "ASM loc_11496 is the immediate epilogue after the `bne.s` at loc_1146E, so the support-worker path should return without queuing `sub_114C0`"
        );
        assert_eq!(
            rt.block_state
                .shooter_headbutt_workers
                .iter()
                .filter(|worker| worker.kind == ShooterHeadbuttWorkerKind::Support)
                .count(),
            2,
            "ASM sub_1149C should queue only the empty support cells above the shooter"
        );
    }

    #[test]
    fn shooter_support_spawn_loc_11496_returns_without_marking_the_struck_cell() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_shooter_blocks(&[(5, 4, 0x02)]);
        grid.set_cell_at(6, 3, terrain::Cell::from_be_bytes(0x60, 0));
        rt.set_block_grid(grid);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((5, 4));

        rt.process_block_effects(&mut slice);

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 4),
            terrain::Cell::from_be_bytes(0xE8, 0x02),
            "ASM loc_11496 is the immediate epilogue after the `bne.s` at loc_1146E, so the support-worker path must return before the struck shooter gets `or.b #$F,(a1)`"
        );
        assert!(
            rt.block_state
                .shooter_headbutt_workers
                .iter()
                .all(|worker| worker.kind == ShooterHeadbuttWorkerKind::Support),
            "ASM loc_11496 should restore registers and return without leaving any struck-cell `sub_114C0` worker behind"
        );
    }

    #[test]
    fn iron_knight_smash_block_uses_sub_10eda_upward_offset() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::IronKnight;
        runtime.progression.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();

        let mut grid = grid_with_floor_at_row(14);
        grid.set_cell_at(5, 14, terrain::Cell::from_be_bytes(0xE0, 0x01));
        runtime.block_grid = Some(grid);

        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = (5 * 16 + 8) << 16;
        slice.player.y_pos = 223 << 16;
        // ASM sub_AF7A requires the first Iron Knight block-break hit to meet
        // `$48000` before off_AFD8 dispatches the smash effect.
        slice.player.y_velocity = 0x0005_0000;
        slice.player.on_ground = false;
        runtime.opening_slice = Some(slice);

        runtime.step_frame(FrameInput::default());

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(runtime.block_destroy_effects[0].x_px, 80);
        assert_eq!(
            runtime.block_destroy_effects[0].y_px, 220,
            "ASM loc_10F1A subtracts 4 from `y_pos(a3)` when `d6 == 2`, so sub_10EDA nudges the smash effect 4px upward before the 2-frame hibernate"
        );
        assert_eq!(runtime.block_destroy_effects[0].remaining, 2);
    }

    #[test]
    fn iron_knight_ice_block_uses_sub_10f9c_d6_zero_downward_offset() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.boot_state = BootState::InGame;
        runtime.progression.current_form = PlayerForm::IronKnight;
        runtime.progression.number_hitpoints = PlayerForm::IronKnight.max_hitpoints();

        let mut grid = grid_with_floor_at_row(14);
        grid.set_cell_at(5, 14, terrain::Cell::from_be_bytes(0xE2, 0x01));
        runtime.block_grid = Some(grid);

        let bounds = runtime.level_bounds(LevelId(0));
        let mut slice = OpeningSliceState::respawn(bounds);
        slice.player.x_pos = (5 * 16 + 8) << 16;
        slice.player.y_pos = 223 << 16;
        // ASM sub_AF7A requires the first Iron Knight block-break hit to meet
        // `$48000` before off_AFD8 dispatches the ice effect.
        slice.player.y_velocity = 0x0005_0000;
        slice.player.on_ground = false;
        runtime.opening_slice = Some(slice);

        runtime.step_frame(FrameInput::default());

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(runtime.block_destroy_effects[0].x_px, 80);
        assert_eq!(
            runtime.block_destroy_effects[0].y_px, 224,
            "ASM sub_10F9C first exposes the ice worker at the source cell before loc_10FDC applies the hibernate-frame nudge"
        );
        assert_eq!(runtime.block_destroy_effects[0].remaining, 4);

        runtime.step_visual_effects();

        assert_eq!(
            runtime.block_destroy_effects[0].y_px, 228,
            "ASM sub_AF7A leaves d6=0 for off_AFD8[2], so sub_10F9C falls through to `addq.w #4,y_pos(a3)` before the shared loc_10FEC hibernate tail"
        );
        assert_eq!(runtime.block_destroy_effects[0].remaining, 3);
    }

    #[test]
    fn ice_block_effect_sub_10f9c_uses_loc_10fe2_leftward_offset() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_ice_block_effect_sub_10f9c(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_LEFT,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(runtime.block_destroy_effects[0].x_px, 80);
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 4);

        runtime.step_visual_effects();

        assert_eq!(
            runtime.block_destroy_effects[0].x_px, 76,
            "ASM loc_10FE2 subtracts 4 from `x_pos(a3)` when `d6 == 1`, after sub_10F9C's source-cell setup frame"
        );
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 3);
        assert_eq!(
            runtime.block_destroy_effects[0].direction,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_LEFT
        );
    }

    #[test]
    fn ice_block_effect_sub_10f9c_uses_loc_10fe8_rightward_offset() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_ice_block_effect_sub_10f9c(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_RIGHT,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(runtime.block_destroy_effects[0].x_px, 80);
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 4);

        runtime.step_visual_effects();

        assert_eq!(
            runtime.block_destroy_effects[0].x_px, 84,
            "ASM loc_10FE8 adds 4 to `x_pos(a3)` when `d6 == 3`, after sub_10F9C's source-cell setup frame"
        );
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 3);
        assert_eq!(
            runtime.block_destroy_effects[0].direction,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_RIGHT
        );
    }

    #[test]
    fn ice_block_effect_sub_10f9c_uses_the_default_downward_offset_when_d6_is_zero() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_ice_block_effect_sub_10f9c(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_BELOW,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(
            runtime.block_destroy_effects[0].x_px, 80,
            "ASM sub_10F9C exposes the source-cell setup frame before the default downward loc_10FEC tail",
        );
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 4);

        runtime.step_visual_effects();

        assert_eq!(
            runtime.block_destroy_effects[0].y_px, 52,
            "ASM sub_10F9C takes the default `addq.w #4,y_pos(a3)` path when `d6` is neither 1, 2, nor 3",
        );
        assert_eq!(runtime.block_destroy_effects[0].remaining, 3);
        assert_eq!(
            runtime.block_destroy_effects[0].direction,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_BELOW
        );
    }

    #[test]
    fn ice_block_effect_sub_10f9c_uses_loc_10fdc_upward_offset() {
        // ASM sub_10F9C loc_10FDC (kid.asm:22900): `cmpi.w #2,d6; beq.s
        // loc_10FDC` nudges the ice shard worker 4px UP (`subq.w #4,y_pos(a3)`)
        // for the ordinary ceiling headbutt that sub_BB54 seeds with d6=2. The
        // three other arms (LEFT/RIGHT/default-down) are pinned above, but the
        // ABOVE arm had no exact-value test: deleting it falls through to the
        // default `addq.w #4,y_pos(a3)` (down) and would survive undetected.
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_ice_block_effect_sub_10f9c(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(
            runtime.block_destroy_effects[0].x_px, 80,
            "ASM sub_10F9C exposes the source-cell setup frame before the loc_10FDC upward nudge",
        );
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 4);

        runtime.step_visual_effects();

        assert_eq!(
            runtime.block_destroy_effects[0].y_px, 44,
            "ASM loc_10FDC subtracts 4 from `y_pos(a3)` when `d6 == 2`, after sub_10F9C's source-cell setup frame",
        );
        assert_eq!(runtime.block_destroy_effects[0].x_px, 80);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 3);
        assert_eq!(
            runtime.block_destroy_effects[0].direction,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE
        );
    }

    #[test]
    fn smash_block_effect_sub_10eda_uses_loc_10f20_leftward_offset() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_smash_block_effect_sub_10eda(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_LEFT,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(
            runtime.block_destroy_effects[0].x_px, 76,
            "ASM loc_10F20 subtracts 4 from `x_pos(a3)` when `d6 == 1`, so sub_10EDA nudges the smash effect 4px left before the 2-frame hibernate"
        );
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 2);
        assert_eq!(
            runtime.block_destroy_effects[0].direction,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_LEFT
        );
    }

    #[test]
    fn smash_block_effect_sub_10eda_uses_loc_10f2a_default_downward_offset() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_smash_block_effect_sub_10eda(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_BELOW,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(runtime.block_destroy_effects[0].x_px, 80);
        assert_eq!(
            runtime.block_destroy_effects[0].y_px, 52,
            "ASM sub_10EDA falls through to the default `addq.w #4,y_pos(a3)` path before loc_10F2A when `d6 == 0`, so the smash effect should nudge 4px downward before the 2-frame hibernate"
        );
        assert_eq!(runtime.block_destroy_effects[0].remaining, 2);
        assert_eq!(
            runtime.block_destroy_effects[0].direction,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_BELOW
        );
    }

    #[test]
    fn smash_block_effect_sub_10eda_uses_loc_10f26_rightward_offset() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.spawn_smash_block_effect_sub_10eda(
            80,
            48,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_RIGHT,
        );

        assert_eq!(runtime.block_destroy_effects.len(), 1);
        assert_eq!(
            runtime.block_destroy_effects[0].x_px, 84,
            "ASM loc_10F26 adds 4 to `x_pos(a3)` when `d6 == 3`, so sub_10EDA nudges the smash effect 4px right before the 2-frame hibernate"
        );
        assert_eq!(runtime.block_destroy_effects[0].y_px, 48);
        assert_eq!(runtime.block_destroy_effects[0].remaining, 2);
        assert_eq!(
            runtime.block_destroy_effects[0].direction,
            NativeRuntime::BLOCK_DESTROY_DIRECTION_RIGHT
        );
    }

    #[test]
    fn hidden_block_reveal_dispatch_requires_changed_solid_cell_loc_e096() {
        for (after_hi, after_lo, case) in [
            (0xA5, 0x01, "unchanged hidden block"),
            (0x00, 0x00, "cleared non-block cell"),
        ] {
            let mut rt = NativeRuntime::new(manifest());
            rt.set_block_grid(grid_with_cells(8, 8, &[(2, 2, after_hi, after_lo)]));

            rt.queue_hidden_block_reveal_bounces_from_diff(Some(&[(2, 2)]));

            assert!(
                rt.block_bounces.is_empty(),
                "ASM sub_DFB0 loc_E096 should only queue a reveal worker after a hidden cell becomes a real block: {case}",
            );
            assert!(
                rt.played_sound_effects.is_empty(),
                "unchanged/non-block cells must not queue sfx_Reveal_Hidden_block: {case}",
            );
        }
    }

    #[test]
    fn hidden_block_reveal_dispatch_places_fallback_worker_at_source_cell_loc_e096() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_cells(8, 8, &[(2, 3, 0xE0, 0x01)]));

        rt.queue_hidden_block_reveal_bounces_from_diff(Some(&[(2, 3)]));

        assert_eq!(rt.block_bounces.len(), 1);
        assert_eq!(
            (
                rt.block_bounces[0].cell_x,
                rt.block_bounces[0].cell_y,
                rt.block_bounces[0].remaining,
                rt.block_bounces[0].kind,
            ),
            (2, 3, 8, BlockBounceKind::HiddenReveal),
            "ASM loc_E096/loc_E2D2 should convert the reveal object's pixel position back to the source tile coordinates",
        );
        assert_eq!(
            rt.played_sound_effects,
            vec![QueuedSoundEffect::HiddenBlockReveal],
        );
    }

    #[test]
    fn hidden_block_reveal_dispatch_only_suppresses_active_same_cell_workers_loc_e096() {
        let mut active_same_cell = NativeRuntime::new(manifest());
        active_same_cell.set_block_grid(grid_with_cells(8, 8, &[(2, 3, 0xE0, 0x01)]));
        active_same_cell.block_bounces.push(BlockBounceAnim {
            cell_x: 2,
            cell_y: 3,
            x_offset_px: 0,
            remaining: 1,
            initial_duration: 8,
            kind: BlockBounceKind::HiddenReveal,
            prize_content: None,
            prize_deleted_tile: false,
            metal_direction: 0,
            y_offset_px: 0,
        });

        active_same_cell.queue_hidden_block_reveal_bounces_from_diff(Some(&[(2, 3)]));

        assert_eq!(
            active_same_cell.block_bounces.len(),
            1,
            "ASM loc_E096 native dispatch should not allocate a duplicate worker while the same hidden reveal is still active",
        );
        assert!(
            active_same_cell.played_sound_effects.is_empty(),
            "duplicate suppression must happen before queuing sfx_Reveal_Hidden_block",
        );

        let mut expired_same_cell = NativeRuntime::new(manifest());
        expired_same_cell.set_block_grid(grid_with_cells(8, 8, &[(2, 3, 0xE0, 0x01)]));
        expired_same_cell.block_bounces.push(BlockBounceAnim {
            cell_x: 2,
            cell_y: 3,
            x_offset_px: 0,
            remaining: 0,
            initial_duration: 8,
            kind: BlockBounceKind::HiddenReveal,
            prize_content: None,
            prize_deleted_tile: false,
            metal_direction: 0,
            y_offset_px: 0,
        });

        expired_same_cell.queue_hidden_block_reveal_bounces_from_diff(Some(&[(2, 3)]));

        assert_eq!(
            expired_same_cell.block_bounces.len(),
            2,
            "an expired same-cell worker must not suppress the new loc_E096 reveal worker",
        );
        assert_eq!(
            expired_same_cell.block_bounces[1].remaining, 8,
            "the newly queued worker should carry loc_E304's eight-frame reveal duration",
        );

        let mut other_cell_active = NativeRuntime::new(manifest());
        other_cell_active.set_block_grid(grid_with_cells(8, 8, &[(2, 3, 0xE0, 0x01)]));
        other_cell_active.block_bounces.push(BlockBounceAnim {
            cell_x: 4,
            cell_y: 3,
            x_offset_px: 0,
            remaining: 1,
            initial_duration: 8,
            kind: BlockBounceKind::HiddenReveal,
            prize_content: None,
            prize_deleted_tile: false,
            metal_direction: 0,
            y_offset_px: 0,
        });

        other_cell_active.queue_hidden_block_reveal_bounces_from_diff(Some(&[(2, 3)]));

        assert_eq!(
            other_cell_active.block_bounces.len(),
            2,
            "an active reveal worker at a different cell must not suppress loc_E096 for the newly revealed source cell",
        );
        assert_eq!(
            (
                other_cell_active.block_bounces[1].cell_x,
                other_cell_active.block_bounces[1].cell_y
            ),
            (2, 3),
        );
    }

    #[test]
    fn process_block_effects_teleporter_leaves_transition_to_hazard_path_loc_bc80() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_list = Some(terrain::BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![terrain::TeleporterEntry {
                layout_addr: 0x1234,
                col: 5,
                row: 3,
                dest_map_id: 0,
                dest_y: 10,
                dest_x: 20,
            }],
            hidden_blocks: vec![],
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 200 << 16;
        slice.player.y_velocity = -0x2_0000;
        slice.player.on_ground = false;
        slice.teleporter_contact = Some(0x1234);

        rt.process_block_effects(&mut slice);

        assert_eq!(
            slice.player.x_pos,
            100 << 16,
            "ASM loc_BC80/loc_BCF6 routes teleporter activation through Teleport; block effects must not inline-snap x"
        );
        assert_eq!(
            slice.player.y_pos,
            200 << 16,
            "ASM Teleport reloads the level shell rather than mutating the live slice y"
        );
        assert_eq!(
            slice.player.y_velocity, -0x2_0000,
            "inline teleporter snap must not invent a velocity clear"
        );
        assert!(
            !slice.player.on_ground,
            "inline teleporter snap must not invent grounded state"
        );
    }

    #[test]
    fn process_block_effects_teleporter_no_op_without_block_list() {
        // If block_list is None, teleporter contact should be a no-op.
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 200 << 16;
        slice.teleporter_contact = Some(0x1234);

        rt.process_block_effects(&mut slice);

        assert_eq!(
            slice.player.x_pos,
            100 << 16,
            "player should not move without block_list"
        );
    }

    /// ASM sub_B43A loc_B522 decrements `Telepad_timer` from `$5A` while Kid is
    /// grounded on the telepad. The live frame loop must not warp on first
    /// contact; loc_BC80 only runs once loc_B522 stores the activated cell.
    #[test]
    fn live_telepad_warps_only_after_90_grounded_frames_loc_b522() {
        let mut rt = NativeRuntime::new(manifest());
        rt.boot_state = BootState::InGame;
        rt.current_level = LevelId(0);
        rt.set_block_grid(grid_with_cells(8, 8, &[(2, 4, 0xE4, 0)]));
        rt.block_list = Some(terrain::BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![terrain::TeleporterEntry {
                layout_addr: terrain::BlockList::level_layout_low_word_addr(2, 4, 8),
                col: 2,
                row: 4,
                dest_map_id: 0,
                dest_y: 6,
                dest_x: 7,
            }],
            hidden_blocks: vec![],
        });

        let mut slice = OpeningSliceState::new(LevelBounds {
            width_pixels: 8 * 16,
            height_pixels: 8 * 16,
        });
        slice.player.x_pos = 39 << 16;
        slice.player.y_pos = 63 << 16;
        slice.player.x_velocity = 0;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.terrain.floor_y_pos = 64;
        rt.opening_slice = Some(slice);

        for frame in 1..90 {
            rt.step_frame(FrameInput::default());
            let slice = rt
                .opening_slice
                .as_ref()
                .expect("telepad countdown frame should keep gameplay slice");
            assert_eq!(
                slice.player.x_pos,
                39 << 16,
                "loc_B522 frame {frame}: nonzero countdown still returns floor support without warping",
            );
        }

        rt.step_frame(FrameInput::default());
        // ASM loc_BCF6 sets $FFFFFC29 then branches to Teleport, which keeps
        // Game_Mode=$000C -> Load_InGame ($C), NOT Load_TitleCard ($8): the
        // telepad warp reloads the level with no title card, so native lands in
        // BootState::InGame rather than the title-card shell. #327.
        assert!(
            matches!(rt.boot_state, BootState::InGame),
            "ASM loc_BCF6 keeps Game_Mode=$C -> Load_InGame after the loc_B522 countdown reaches zero (no title card)"
        );
        let slice = rt
            .opening_slice
            .as_ref()
            .expect("telepad activation should seed the same-level reload slice");
        assert_eq!(
            slice.player.x_pos,
            (7 * 16) << 16,
            "loc_B522 frame 90 should activate loc_BC80 destination x through PlayerStart_X",
        );
        assert_eq!(
            slice.player.y_pos, 0x0583_0000,
            "loc_B522 frame 90 activates loc_BC80 destination y; loc_7526 starts at PlayerStart_Y-1 and loc_11BD2/loc_11BEC runs the hidden Load_InGame floor settle before gameplay",
        );
    }

    #[test]
    fn blue_lake_woods_2_teleporter_uses_generated_block_list_loc_b522_loc_bc80() {
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.boot_state = BootState::InGame;
        rt.current_level = LevelId(1);
        rt.reload_block_grid_for_level(LevelId(1));

        let grid = rt
            .block_grid()
            .expect("Blue Lake Woods 2 generated block grid");
        assert_eq!(
            (
                grid.cell_at(156, 36).has_block,
                grid.cell_at(156, 36).block_type
            ),
            (true, terrain::BlockType::Teleporter),
            "ASM sub_B43A loc_B4FA only starts loc_B522 for block type `$400` with bit 15 set",
        );
        assert!(
            rt.block_list
                .as_ref()
                .expect("Blue Lake Woods 2 block list")
                .teleporter_at(156, 36)
                .is_some(),
            "ASM loc_BC96 must find the generated block-list teleporter entry for the activated tile",
        );

        let mut slice = OpeningSliceState::new(LevelBounds {
            width_pixels: grid.width_cells * 16,
            height_pixels: grid.height_cells * 16,
        });
        slice.player.x_pos = ((156 * 16) + 7) << 16;
        slice.player.y_pos = ((36 * 16) - 1) << 16;
        slice.player.x_velocity = 0;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.terrain.floor_y_pos = 36 * 16;
        rt.opening_slice = Some(slice);

        for _ in 1..90 {
            rt.step_frame(FrameInput::default());
        }
        rt.step_frame(FrameInput::default());

        // ASM loc_BCF6 keeps Game_Mode=$000C (FC29 set) -> Load_InGame ($C),
        // not Load_TitleCard ($8): same-level telepads reload with no title
        // card, so native lands in BootState::InGame. #327.
        assert!(
            matches!(rt.boot_state, BootState::InGame),
            "ASM loc_BCF6 keeps Game_Mode=$C -> Load_InGame for same-level telepads (no title card)"
        );
        let slice = rt
            .opening_slice
            .as_ref()
            .expect("same-level teleporter should seed the reloaded level slice");
        assert_eq!(
            (slice.player.x_pos, slice.player.y_pos),
            ((28 * 16) << 16, ((9 * 16) - 1) << 16),
            "ASM loc_BCB2 writes PlayerStart tile 28,9, then loc_7526 seeds Kid at PlayerStart_Y-1 during the Teleport level reload",
        );
        assert!(
            rt.played_sound_effects
                .contains(&QueuedSoundEffect::Teleport),
            "ASM loc_BCF6 passes d6=-5 into Teleport for sfx_Teleport",
        );
    }

    #[test]
    fn loc_7a60_left_probe_blocked_snaps_right_to_tile_boundary() {
        // ASM loc_7A60 (kid.asm:8126): when x_pos - $E < 0 or left probe finds
        // $4000, snap x to `((x-14) & $FFF0) + $10 + 14`. Grid col 1 at y=4
        // is solid, blocking the left probe from x=32.
        let grid = grid_with_cells(8, 8, &[(1, 4, 0x60, 0)]);

        let alignment = align_player_for_down_hold_sub_7A10(&grid, 32 << 16, 64 << 16, 128);

        assert_eq!(
            alignment,
            DownHoldAlignmentSub7A10 {
                x_pos: 46 << 16,
                blocked: false,
            },
            "ASM loc_7A60: left blocked → snap right to next tile boundary + half_width",
        );
    }

    #[test]
    fn loc_7a60_left_out_of_bounds_snaps_right() {
        // ASM loc_7A60 (kid.asm:8126): when x_pos - $E < 0 (bmi.w loc_7A60),
        // the left edge is past the level boundary → snap right.
        let grid = grid_with_cells(8, 8, &[]);

        let alignment = align_player_for_down_hold_sub_7A10(&grid, 10 << 16, 64 << 16, 128);

        assert_eq!(
            alignment,
            DownHoldAlignmentSub7A10 {
                x_pos: 14 << 16,
                blocked: false,
            },
            "ASM loc_7A60: x - 14 < 0 → snap right to ((10-14) & !0xF) + 16 + 14 = 14",
        );
    }

    #[test]
    fn loc_7a98_right_probe_blocked_snaps_left() {
        // ASM loc_7A98 (kid.asm:8149): right probe finds $4000 or x + $E >=
        // Level_width_pixels → snap left by `(x+14) & $F + 1`.
        let grid = grid_with_cells(8, 8, &[(2, 4, 0x60, 0)]);

        let alignment = align_player_for_down_hold_sub_7A10(&grid, 32 << 16, 64 << 16, 128);

        assert_eq!(
            alignment,
            DownHoldAlignmentSub7A10 {
                x_pos: 17 << 16,
                blocked: false,
            },
            "ASM loc_7A98: right blocked → snap left by low nibble + 1",
        );
    }

    #[test]
    fn loc_7a90_both_sides_blocked_restores_original_x() {
        // ASM loc_7A90 (kid.asm:8143): `move.w (sp)+,x_pos(a3); moveq #1,d7;
        // rts` — neither left-snap nor right-snap found clearance, so restore
        // the original integer x_pos (subpixel discarded) and return blocked.
        let grid = grid_with_cells(8, 8, &[(1, 4, 0x60, 0), (3, 4, 0x60, 0)]);

        let alignment =
            align_player_for_down_hold_sub_7A10(&grid, (32 << 16) | 0x8000, 64 << 16, 128);

        assert_eq!(
            alignment,
            DownHoldAlignmentSub7A10 {
                x_pos: 32 << 16,
                blocked: true,
            },
            "ASM loc_7A90: both sides blocked → restore integer x, blocked=true",
        );
    }

    #[test]
    fn loc_7a90_right_snap_fails_left_recheck() {
        // ASM loc_7A90 via loc_7A98 (kid.asm:8149→8143): right probe blocked,
        // snap left, but the left re-probe at the snapped position also finds
        // a wall → fall through to loc_7A90 (bra.s) and restore original x.
        let grid = grid_with_cells(8, 8, &[(1, 4, 0x60, 0), (3, 4, 0x60, 0)]);

        let alignment = align_player_for_down_hold_sub_7A10(&grid, 46 << 16, 64 << 16, 128);

        assert!(
            alignment.blocked,
            "ASM loc_7A90: right blocked, left re-probe also blocked → restore and return blocked",
        );
        assert_eq!(
            alignment.x_pos,
            46 << 16,
            "ASM loc_7A90: restored x_pos should be the original integer value",
        );
    }

    #[test]
    fn loc_331d2_masks_only_the_sampled_relative_cell_collision_nibble() {
        let grid = grid_with_cells(8, 8, &[(3, 4, 0x70, 0x09), (2, 4, 0x60, 0x05)]);

        assert_eq!(
            crate::terrain::relative_collision_mask_loc_331D2(&grid, 40, 64, 8, 0),
            0x7000,
            "ASM loc_331D2 (kid.asm:37627) should add the relative offsets onto `x_pos/y_pos(a3)`, look up exactly that Level_Layout cell through `$FFFF4A04`, and keep only the `$7000` collision nibble",
        );
    }

    #[test]
    fn loc_33248_zeroes_disguised_solid_ghost_blocks_before_masking_collision() {
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_33248(0xB355),
            0,
            "ASM loc_33248 (kid.asm:37670) should treat a bit-15 tile with block-type nibble `$300` as non-colliding after the `bclr #$F` / `andi.w #$F00` gate",
        );
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_33248(0x6355),
            0x6000,
            "ASM loc_33248 should leave ordinary non-blocking words on the `beq` path and keep only their collision nibble",
        );
    }

    #[test]
    fn loc_33264_zeroes_the_middle_row_disguised_block_before_masking_collision() {
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_33264(0xB344),
            0,
            "ASM loc_33264 (kid.asm:37680) should apply the same `bclr #$F` / `andi.w #$F00` disguised-block gate to `d6` before falling through to the shared collision mask",
        );
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_33264(0x6444),
            0x6000,
            "ASM loc_33264 should keep ordinary middle-row collision words on the fall-through path and preserve only their `$7000` collision nibble",
        );
    }

    #[test]
    fn loc_33280_keeps_only_the_top_row_collision_nibble() {
        assert_eq!(
            crate::terrain::relative_collision_nibble_loc_33280(0xF6A5),
            0x7000,
            "ASM loc_33280 (kid.asm:37690) should end the third sampled-word path with `andi.w #$7000,d7`, dropping both the block flag and the lower tile bits",
        );
        assert_eq!(
            crate::terrain::relative_collision_nibble_loc_33280(0x0333),
            0,
            "ASM loc_33280 should leave a zero collision nibble when the incoming top-row word has no `$1000/$2000/$4000/$5000/$6000/$7000` terrain bits set",
        );
    }

    #[test]
    fn loc_332d0_zeroes_the_horizontal_row_disguised_block_before_masking_collision() {
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_332D0(0xB377),
            0,
            "ASM loc_332D0 (kid.asm:37723) should apply the same `bclr #$F` / `andi.w #$F00` disguised-block gate to horizontal-scan word `d5` before masking the collision nibble",
        );
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_332D0(0x4777),
            0x4000,
            "ASM loc_332D0 should keep ordinary horizontal-scan words on the fall-through path and preserve only their `$7000` collision nibble",
        );
    }

    #[test]
    fn loc_332ec_zeroes_the_second_horizontal_row_disguised_block_before_masking_collision() {
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_332EC(0xB355),
            0,
            "ASM loc_332EC (kid.asm:37733) should apply the same `bclr #$F` / `andi.w #$F00` disguised-block gate to horizontal-scan word `d6` before masking the collision nibble",
        );
        assert_eq!(
            crate::terrain::relative_collision_word_mask_loc_332EC(0x6555),
            0x6000,
            "ASM loc_332EC should keep ordinary second-row horizontal words on the fall-through path and preserve only their `$7000` collision nibble",
        );
    }

    #[test]
    fn loc_33308_keeps_only_the_final_horizontal_row_collision_nibble() {
        assert_eq!(
            crate::terrain::relative_collision_nibble_loc_33308(0xF6A5),
            0x7000,
            "ASM loc_33308 (kid.asm:37743) should end the horizontal three-word scan with `andi.w #$7000,d7`, dropping the block flag and lower tile bits",
        );
        assert_eq!(
            crate::terrain::relative_collision_nibble_loc_33308(0x0333),
            0,
            "ASM loc_33308 should leave a zero collision nibble when the final horizontal word carries no terrain bits in `$7000`",
        );
    }

    #[test]
    fn loc_3330e_reads_one_relative_level_layout_cell_and_masks_its_collision_bits() {
        let grid = grid_with_cells(8, 8, &[(3, 5, 0x67, 0x89), (4, 5, 0x45, 0x67)]);

        assert_eq!(
            crate::terrain::relative_collision_mask_loc_3330E(&grid, 56, 80, 8, 0),
            0x4000,
            "ASM loc_3330E (kid.asm:37750) should add `d6/d7` onto `x_pos/y_pos(a3)`, resolve exactly that Level_Layout cell through `$FFFF4A04`, and keep only the `$7000` collision nibble",
        );
        assert_eq!(
            crate::terrain::relative_collision_mask_loc_3330E(&grid, 56, 80, -8, 0),
            0x6000,
            "ASM loc_3330E should honour negative relative X offsets before the `lsr.w #4` cell conversion so the sampled collision nibble follows the addressed neighbour cell",
        );
    }

    #[test]
    fn loc_3330e_uses_unsigned_y_cell_conversion_after_word_add() {
        let grid = grid_with_cells(1, 4096, &[(0, 4095, 0x60, 0x00)]);

        assert_eq!(
            crate::terrain::relative_collision_mask_loc_3330E(&grid, 0, 0, 0, -1),
            0x6000,
            "ASM loc_3330E (kid.asm:37750) uses `add.w y_pos(a3),d7` followed by `lsr.w #4,d7`, so y=-1 resolves to row `$0FFF` instead of taking the signed negative-coordinate guard used by loc_331D2",
        );
    }

    #[test]
    fn loc_331f6_reads_the_foot_row_plus_14_and_the_two_rows_above_it() {
        let grid = grid_with_cells(
            8,
            8,
            &[(2, 1, 0x45, 0x01), (2, 2, 0xA3, 0x02), (2, 3, 0x70, 0x03)],
        );

        let filtered = crate::terrain::relative_collision_column_loc_331F6(&grid, 32, 34, 0, 0);
        assert_eq!(
            filtered,
            crate::terrain::RelativeCollisionColumnLoc331F6 {
                current_row: 0x7000,
                one_row_up: 0,
                two_rows_up: 0x4000,
            },
            "ASM loc_331F6 (kid.asm:37642) should sample `y_pos(a3)+$E`, then walk up two `Level_width_tiles` rows, while routing each word through loc_33248 before keeping only the collision nibble",
        );

        let raw = crate::terrain::relative_collision_column_loc_331F6(&grid, 32, 34, 0, 0x03E7);
        assert_eq!(
            raw,
            crate::terrain::RelativeCollisionColumnLoc331F6 {
                current_row: 0x7003,
                one_row_up: 0xA302,
                two_rows_up: 0x4001,
            },
            "ASM loc_331F6 should branch straight to `return_33284` when `d4 == $3E7`, preserving the raw tile words instead of running the loc_33248 filter",
        );
    }

    #[test]
    fn ice_block_load_bullets_restore_direction_loc_11080_restores_the_original_direction_bits() {
        assert_eq!(
            NativeRuntime::ice_block_load_bullets_restore_direction_loc_11080(0),
            2,
            "ASM loc_11080 should restore the original down-facing `d6` after the entry `eori.w #2,d6` remap",
        );
        assert_eq!(
            NativeRuntime::ice_block_load_bullets_restore_direction_loc_11080(1),
            3,
            "ASM loc_11080 should restore the original right-facing `d6` before returning",
        );
        assert_eq!(
            NativeRuntime::ice_block_load_bullets_restore_direction_loc_11080(2),
            0,
            "ASM loc_11080 should restore the original up-facing `d6` before returning",
        );
        assert_eq!(
            NativeRuntime::ice_block_load_bullets_restore_direction_loc_11080(3),
            1,
            "ASM loc_11080 should restore the original left-facing `d6` before returning",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_11020_steps_one_cell_right_inside_the_level_width()
     {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(5, 3, 3, 8, 6),
            Some((6, 3)),
            "ASM loc_11020 should remap direction 3 through `eori.w #2,d6`, then probe one block to the right when `source_col + 1 < Level_width_blocks`",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_11020_rejects_the_right_boundary() {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(7, 3, 3, 8, 6),
            None,
            "ASM loc_11020 should branch to `loc_11080` when `source_col + 1 >= Level_width_blocks`",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_11036_steps_one_cell_down_inside_the_level_height()
     {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(5, 3, 0, 8, 6),
            Some((5, 4)),
            "ASM loc_11036 should remap direction 0 through `eori.w #2,d6`, then probe one block downward when `source_row + 1 < Level_height_blocks`",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_11036_rejects_the_bottom_boundary() {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(5, 5, 0, 8, 6),
            None,
            "ASM loc_11036 should branch to `loc_11080` when `source_row + 1 >= Level_height_blocks`",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_1104a_steps_one_cell_left_inside_the_level_width()
    {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(5, 3, 1, 8, 6),
            Some((4, 3)),
            "ASM loc_1104A should remap direction 1 through `eori.w #2,d6`, then probe one block to the left when `source_col != 0`",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_sub_11006_loc_1104a_rejects_the_left_boundary() {
        assert_eq!(
            NativeRuntime::ice_block_bullet_target_cell_sub_11006(0, 3, 1, 8, 6),
            None,
            "ASM loc_1104A should branch to `loc_11080` when the source column is already zero",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_open_loc_11052_accepts_empty_and_hidden_block_cells() {
        assert!(
            NativeRuntime::ice_block_bullet_target_cell_open_loc_11052(
                terrain::Cell::from_be_bytes(0x00, 0x00)
            ),
            "ASM loc_11052 should keep firing through Empty cells because `btst #6,(a0)` sees a clear collision high bit",
        );
        assert!(
            NativeRuntime::ice_block_bullet_target_cell_open_loc_11052(
                terrain::Cell::from_be_bytes(0x20, 0x00)
            ),
            "ASM loc_11052 should also allow HiddenBlock terrain because collision kind 2 leaves bit 6 clear",
        );
    }

    #[test]
    fn ice_block_bullet_target_cell_open_loc_11052_rejects_cells_with_the_collision_high_bit_set() {
        for (hi, label) in [
            (0x40, "SlopeUp"),
            (0x50, "SlopeDown"),
            (0x60, "Solid"),
            (0x70, "Spikes"),
        ] {
            assert!(
                !NativeRuntime::ice_block_bullet_target_cell_open_loc_11052(
                    terrain::Cell::from_be_bytes(hi, 0x00)
                ),
                "ASM loc_11052 should reject {label} terrain because `btst #6,(a0)` branches to `loc_11080` whenever the collision high bit is set",
            );
        }
    }

    #[test]
    fn loc_3bede_driller_smash_block_spawns_effect_at_tile_origin() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_cells(8, 8, &[(5, 3, 0xE0, 0x01)]));

        rt.apply_drilled_block_loc_3bede_3bef2(DrilledBlockLoc3BE78 {
            col: 5,
            row: 3,
            action: DrillerBlockDrillActionLoc3BEDE::SmashBits,
            direction: NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE,
        });

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 3),
            terrain::Cell::from_be_bytes(0, 0),
            "ASM Driller.asm loc_3BEDE reaches Block_LoadSmashBits, which erases the source layout cell",
        );
        assert_eq!(rt.block_destroy_effects.len(), 1);
        assert_eq!(
            (
                rt.block_destroy_effects[0].x_px,
                rt.block_destroy_effects[0].y_px,
                rt.block_destroy_effects[0].remaining,
            ),
            (80, 44, 3),
            "ASM loc_3BEDE passes tile coordinates to sub_10EDA; direction 2 keeps x=5*16, applies loc_10F1A's 4px upward nudge, and includes the allocating Execute_Objects pass before loc_10F2A's two visible hibernate frames",
        );
    }

    #[test]
    fn loc_3bef2_driller_ice_block_restores_original_direction_for_effect() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_cells(8, 8, &[(5, 3, 0xE2, 0x01)]));

        rt.apply_drilled_block_loc_3bede_3bef2(DrilledBlockLoc3BE78 {
            col: 5,
            row: 3,
            action: DrillerBlockDrillActionLoc3BEDE::IceBullets,
            direction: NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE,
        });

        assert_eq!(rt.block_destroy_effects.len(), 1);
        assert_eq!(
            (
                rt.block_destroy_effects[0].x_px,
                rt.block_destroy_effects[0].y_px,
                rt.block_destroy_effects[0].direction,
                rt.block_destroy_effects[0].remaining,
            ),
            (80, 48, NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE, 4),
            "ASM loc_3BEF2 lets IceBlock_LoadBullets temporarily eor d6 with 2, then loc_11080 restores the original direction before sub_10F9C records the source-cell effect",
        );
    }

    #[test]
    fn loc_3bef2_driller_ice_block_spawns_three_shooter_bullets_when_target_open() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_cells(8, 8, &[(5, 3, 0xE2, 0x01)]));

        rt.apply_drilled_block_loc_3bede_3bef2(DrilledBlockLoc3BE78 {
            col: 5,
            row: 3,
            action: DrillerBlockDrillActionLoc3BEDE::IceBullets,
            direction: 3,
        });

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 3),
            terrain::Cell::from_be_bytes(0, 0),
            "ASM sub_10F9C should erase the drilled ice source cell after loc_3BEF2 queues the worker",
        );
        assert_eq!(rt.block_destroy_effects.len(), 1);
        assert_eq!(
            rt.block_destroy_effects[0].remaining, 4,
            "ASM sub_10F9C should keep the drilled ice worker alive for its source-cell setup frame before loc_10FEC's three-frame hibernate"
        );
        assert_eq!(
            rt.projectiles
                .iter()
                .map(|p| (
                    p.kind,
                    p.x_pos >> 16,
                    p.y_pos >> 16,
                    p.x_velocity,
                    p.y_velocity,
                ))
                .collect::<Vec<_>>(),
            vec![
                (EnemyProjectileKind::ShooterBlock, 88, 56, 4 << 16, 0),
                (EnemyProjectileKind::ShooterBlock, 84, 52, 4 << 16, 0),
                (EnemyProjectileKind::ShooterBlock, 84, 60, 4 << 16, 0),
            ],
            "ASM IceBlock_LoadBullets remaps Driller direction 3 to rightward d6=1 and loads centre, upper, then lower shooter objects",
        );
    }

    // --- Shooter blocks: multi-direction and cell-change pruning ---

    #[test]
    fn shooter_player_in_range_sub_f536_accepts_both_inclusive_edges() {
        assert!(
            NativeRuntime::shooter_player_in_range_sub_F536((6 * 16) << 16, (9 * 16) << 16, 2, 3),
            "ASM sub_F536 should accept the left/top inclusive edge of Kid's `x_block-4 .. x_block+4` and `y_block-6 .. y_block+3` window",
        );
        assert!(
            NativeRuntime::shooter_player_in_range_sub_F536((6 * 16) << 16, (9 * 16) << 16, 10, 12),
            "ASM sub_F536 should also accept the right/bottom inclusive edge of that same block-space window",
        );
    }

    #[test]
    fn shooter_player_in_range_sub_f536_rejects_just_outside_window() {
        assert!(
            !NativeRuntime::shooter_player_in_range_sub_F536((6 * 16) << 16, (9 * 16) << 16, 1, 3),
            "ASM loc_F564 should return zero for source columns left of `kid_x_block-4`",
        );
        assert!(
            !NativeRuntime::shooter_player_in_range_sub_F536(
                (6 * 16) << 16,
                (9 * 16) << 16,
                11,
                12
            ),
            "ASM loc_F564 should return zero for source columns right of `kid_x_block+4`",
        );
        assert!(
            !NativeRuntime::shooter_player_in_range_sub_F536((6 * 16) << 16, (9 * 16) << 16, 2, 2),
            "ASM loc_F564 should return zero for source rows above `kid_y_block-6`",
        );
        assert!(
            !NativeRuntime::shooter_player_in_range_sub_F536(
                (6 * 16) << 16,
                (9 * 16) << 16,
                10,
                13
            ),
            "ASM loc_F564 should return zero for source rows below `kid_y_block+3`",
        );
    }

    #[test]
    fn shooter_collision_rects_sub_f568_emits_two_upward_rectangles_inside_the_kid_window() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(6, 7, 0x01)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 98,
                    right_px: 110,
                    top_px: 99,
                    bottom_px: 111,
                },
                CollisionRectSub1F52 {
                    left_px: 101,
                    right_px: 107,
                    top_px: 103,
                    bottom_px: 111,
                },
            ],
            "ASM sub_F568 should emit the two upward collision rectangles from loc_F5B0 when the shooter sits inside Kid's expanded block window",
        );
    }

    #[test]
    fn shooter_collision_rects_sub_f568_rejects_edges_that_touch_the_window_border() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(4, 7, 0x08), (8, 7, 0x02)],
        );

        assert!(
            rects.is_empty(),
            "ASM sub_F568 skips the left/right-facing rectangles when the source row or column lands exactly on the expanded Kid window edge",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f5a4_empty_active_list_terminates() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[],
        );

        assert!(
            rects.is_empty(),
            "ASM loc_F5A4 tests the initial active-shooter pointer before slot reads; a zero head jumps to the shared terminator path",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f5a6_omits_the_rom_zero_sentinel() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(4, 7, 0x08), (8, 7, 0x02)],
        );

        assert!(
            rects.is_empty(),
            "ASM loc_F5A6 branches to the shared tail, which writes a zero word sentinel after exhausting or rejecting shooter slots; native returns no sentinel rectangle",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f72a_terminates_without_a_native_sentinel() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(20, 7, 0x01)],
        );

        assert!(
            rects.is_empty(),
            "ASM loc_F72A writes a zero word terminator to the ROM output buffer; native omits that sentinel from the returned rectangle list",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f5aa_advances_after_rejected_slot() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(20, 7, 0x01), (6, 7, 0x01)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 98,
                    right_px: 110,
                    top_px: 99,
                    bottom_px: 111,
                },
                CollisionRectSub1F52 {
                    left_px: 101,
                    right_px: 107,
                    top_px: 103,
                    bottom_px: 111,
                },
            ],
            "ASM loc_F5AA follows the current slot's next pointer after rejecting an out-of-window shooter, so later active shooters can still emit rectangles",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f62c_emits_right_facing_rectangles_inside_window() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(5, 6, 0x02), (6, 6, 0x02)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 96,
                    right_px: 104,
                    top_px: 97,
                    bottom_px: 109,
                },
                CollisionRectSub1F52 {
                    left_px: 96,
                    right_px: 108,
                    top_px: 100,
                    bottom_px: 106,
                },
            ],
            "ASM loc_F62C should emit the right-facing pair for col `< right_block-1` and skip the shooter exactly on that edge",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f67e_emits_down_facing_rectangles_inside_window() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(6, 6, 0x04), (6, 7, 0x04)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 98,
                    right_px: 110,
                    top_px: 111,
                    bottom_px: 119,
                },
                CollisionRectSub1F52 {
                    left_px: 101,
                    right_px: 107,
                    top_px: 111,
                    bottom_px: 123,
                },
            ],
            "ASM loc_F67E should emit the down-facing pair for row `< bottom_block-1` and skip the shooter exactly on that edge",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f6d2_emits_left_facing_rectangles_inside_window() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(5, 6, 0x08), (6, 6, 0x08)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 88,
                    right_px: 96,
                    top_px: 97,
                    bottom_px: 109,
                },
                CollisionRectSub1F52 {
                    left_px: 84,
                    right_px: 96,
                    top_px: 100,
                    bottom_px: 106,
                },
            ],
            "ASM loc_F6D2 should skip a left-facing shooter with col-left_block `< 2` and emit the pair for the next inside column",
        );
    }

    #[test]
    fn shooter_collision_rects_loc_f722_left_skip_advances_to_next_slot() {
        let rects = NativeRuntime::shooter_collision_rects_sub_F568(
            CollisionRectSub1F52 {
                left_px: 80,
                right_px: 96,
                top_px: 96,
                bottom_px: 112,
            },
            &[(5, 6, 0x08), (6, 6, 0x08)],
        );

        assert_eq!(
            rects,
            vec![
                CollisionRectSub1F52 {
                    left_px: 88,
                    right_px: 96,
                    top_px: 97,
                    bottom_px: 109,
                },
                CollisionRectSub1F52 {
                    left_px: 84,
                    right_px: 96,
                    top_px: 100,
                    bottom_px: 106,
                },
            ],
            "ASM loc_F722 should advance from a skipped left-facing slot to `$C(a0)` and process the next active shooter",
        );
    }

    #[test]
    fn shooter_collision_top_block_loc_f574_clamps_the_expanded_top_window() {
        assert_eq!(
            NativeRuntime::shooter_collision_top_block_loc_F574(0x000F),
            0,
            "ASM loc_F574 should clamp a negative `(Kid_hitbox_top >> 4) - 1` result to zero",
        );
        assert_eq!(
            NativeRuntime::shooter_collision_top_block_loc_F574(0x0010),
            0,
            "ASM loc_F574 should keep an exact zero expanded top block",
        );
        assert_eq!(
            NativeRuntime::shooter_collision_top_block_loc_F574(0x0050),
            4,
            "ASM loc_F574 should keep positive expanded top-block results",
        );
    }

    #[test]
    fn shooter_collision_right_bottom_blocks_loc_f580_expands_after_logical_word_shift() {
        assert_eq!(
            NativeRuntime::shooter_collision_right_bottom_blocks_loc_F580(0x0060, 0x0070),
            crate::blocks::ShooterCollisionRightBottomBlocksLocF580 {
                right_block: 7,
                bottom_block: 8,
            },
            "ASM loc_F580 should compute `(hitbox + $F) >> 4` and then add one block to both right and bottom bounds",
        );
        assert_eq!(
            NativeRuntime::shooter_collision_right_bottom_blocks_loc_F580(-0x0010, -0x0010),
            crate::blocks::ShooterCollisionRightBottomBlocksLocF580 {
                right_block: 0x1000,
                bottom_block: 0x1000,
            },
            "ASM loc_F580 should use logical `lsr.w`, not arithmetic shift, after the wrapping 16-bit add",
        );
    }

    #[test]
    fn visible_shooter_block_registers_all_four_directions_without_loading_bullets() {
        // ASM sub_F096 loc_F106 registers visible shooter slots. Even when all
        // support-marker checks pass, the negative loc_F410 path compiles
        // sprites instead of allocating moving ShooterObject_RAM bullets.
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_shooter_blocks(&[(5, 3, 0x0F)]);
        grid.set_cell_at(5, 2, terrain::Cell::from_be_bytes(0, 0x2D));
        grid.set_cell_at(6, 3, terrain::Cell::from_be_bytes(0, 0x2D));
        grid.set_cell_at(5, 4, terrain::Cell::from_be_bytes(0, 0x2D));
        grid.set_cell_at(4, 3, terrain::Cell::from_be_bytes(0, 0x2D));
        rt.set_block_grid(grid);
        rt.block_state.shooter_last_scanned_col = 1;
        rt.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        rt.step_shooter_blocks(&slice);

        assert_eq!(rt.projectiles.len(), 0);
        rt.step_shooter_blocks(&slice);
        assert_eq!(
            rt.projectiles.len(),
            0,
            "ASM loc_F410 should not allocate loc_EC76 moving bullets for visible shooter slots"
        );
        assert_eq!(rt.block_state.shooter_blocks.len(), 1);
        assert_eq!(rt.block_state.shooter_blocks[0].directions, 0x0F);
        assert!(rt.block_state.shooter_blocks[0].supported_by_neighbour);
    }

    #[test]
    fn visible_shooter_block_single_direction_up_only_registers_without_loading_bullet() {
        // ASM loc_F106: skin byte 0x01 = up only, and loc_F14C requires
        // the upper neighbour skin byte to be non-zero before the loc_F410
        // visible-sprite path is marked negative.
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_shooter_blocks(&[(5, 3, 0x01)]);
        grid.set_cell_at(5, 2, terrain::Cell::from_be_bytes(0, 0x2D));
        rt.set_block_grid(grid);
        rt.block_state.shooter_last_scanned_col = 1;
        rt.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        rt.step_shooter_blocks(&slice);

        assert_eq!(rt.projectiles.len(), 0);
        rt.step_shooter_blocks(&slice);
        assert_eq!(
            rt.projectiles.len(),
            0,
            "ASM loc_F410 should not allocate loc_EC76 moving bullets for visible shooter slots"
        );
        assert_eq!(rt.block_state.shooter_blocks.len(), 1);
        assert_eq!(rt.block_state.shooter_blocks[0].directions, 0x01);
        assert!(rt.block_state.shooter_blocks[0].supported_by_neighbour);
    }

    #[test]
    fn baseline_level3_e919_mushroom_block_is_not_scanned_as_shooter_sub_f096() {
        // ASM sub_F096 loc_F106/loc_F1DE scans visible shooter/drill layout
        // words, while ASM off_BB80[9] -> loc_BBC2 -> loc_11430 reaches type-9
        // `$E919` cells only through the touch-block worker path. Baseline
        // Highwater Pass 2 `$E919` terrain must not fire before `sub_114F4`
        // restores a support cell and explicitly queues a native shooter slot.
        let mut rt = NativeRuntime::new(manifest());
        let w = 40u16;
        let h = 8u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let offset = (3usize * usize::from(w) + 5usize) * 2;
        data[offset] = 0xE9;
        data[offset + 1] = 0x19;
        rt.set_block_grid(terrain::BlockGrid::from_level_layout(&data, w, h));
        rt.block_state.shooter_last_scanned_col = 1;
        rt.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        rt.step_shooter_blocks(&slice);

        assert_eq!(
            rt.projectiles.len(),
            0,
            "ASM sub_F096 should not treat untouched `$E919` Mushroom terrain as a visible shooter candidate"
        );
        assert!(
            rt.block_state.shooter_blocks.is_empty(),
            "only visible `$ECxx` shooter terrain should be native shooter-live through sub_F096"
        );
    }

    #[test]
    fn highwater_pass_2_first_e919_touch_block_restores_support_without_projectiles() {
        // ASM off_BB80[9] -> loc_BBC2 -> loc_11430: Highwater Pass 2's first
        // `$E919` touch blocks queue sub_1149C support workers; sub_114F4
        // restores the empty cells above as `$E919`, but does not allocate an
        // active Process_ShooterObjects slot. Baseline and restored `$E919`
        // terrain are both non-shooter candidates for sub_F096.
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 40 * 16 * 2];
        let offset = (9usize * 40 + 20usize) * 2;
        data[offset] = 0xE9;
        data[offset + 1] = 0x19;
        rt.set_block_grid(terrain::BlockGrid::from_level_layout(&data, 40, 16));
        rt.progression.current_form = PlayerForm::Juggernaut;
        rt.progression.number_hitpoints = PlayerForm::Juggernaut.max_hitpoints();
        rt.projectiles.clear();

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((20, 9));
        rt.process_block_effects(&mut slice);

        for _ in 0..=16 {
            rt.step_shooter_headbutt_workers();
        }
        rt.step_shooter_blocks(&slice);

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(20, 8),
            terrain::Cell::from_be_bytes(0xE9, 0x19),
            "ASM sub_114F4 restores the empty support cell above the hit `$E919` block"
        );
        assert_eq!(
            rt.projectiles.len(),
            0,
            "ASM sub_114F4 does not allocate a Process_ShooterObjects slot for restored `$E919` support cells"
        );
    }

    #[test]
    fn highwater_pass_2_ec2f_shooter_registers_as_right_facing_loc_f106() {
        // ASM sub_F096 loc_F106/loc_F1DE accepts layout words with block
        // nibble `$C`, then stores `(word & $00FF) - $2D` as direction bits.
        // Highwater Pass 2's exposed shooter cells include `$EC2F`, so they
        // must register right (`$2F - $2D = $02`) when Kid enters the scan
        // column. The visible loc_F410 path draws sprites; it does not allocate
        // moving loc_EC76 shooter bullets.
        let mut rt = NativeRuntime::new(manifest());
        let w = 40u16;
        let h = 8u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let offset = (3usize * usize::from(w) + 5usize) * 2;
        data[offset] = 0xEC;
        data[offset + 1] = 0x2F;
        let support_offset = (3usize * usize::from(w) + 6usize) * 2;
        data[support_offset + 1] = 0x2D;
        rt.set_block_grid(terrain::BlockGrid::from_level_layout(&data, w, h));
        rt.block_state.shooter_last_scanned_col = 1;
        rt.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        rt.step_shooter_blocks(&slice);

        assert_eq!(rt.projectiles.len(), 0);
        rt.step_shooter_blocks(&slice);
        assert_eq!(
            rt.projectiles.len(),
            0,
            "ASM loc_F106 should not allocate a loc_EC76 shooter object for the `$EC2F` visible slot"
        );
        assert_eq!(rt.block_state.shooter_blocks.len(), 1);
        assert_eq!(rt.block_state.shooter_blocks[0].directions, 0x02);
    }

    #[test]
    fn highwater_pass_1_juggernaut_touch_reaches_ea1e_evanescent_shooter_loc_eae4() {
        // ASM loc_EAE4 scans the live `Kid_hitbox_left/right` words, which
        // already include sub_B41C's per-form half-width. Highwater Pass 1's
        // first touch-then-shoot column includes `$EA1E`; Juggernaut's `$17`
        // half-width reaches it while The Kid's `$05` half-width would not.
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        rt.set_block_grid(grid_with_cells(8, 8, &[(5, 3, 0xEA, 0x1E)]));

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = 56 << 16;
        slice.player.y_pos = 64 << 16;
        rt.step_evanescent_blocks(&slice);

        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM loc_EAE4 should use Juggernaut's live hitbox width and trigger the adjacent `$EA1E` shooter"
        );
        assert_eq!(rt.projectiles[0].x_velocity, 4 << 16);
        assert_eq!(rt.projectiles[0].y_velocity, 0);
        assert!(
            rt.trace_frame()
                .active_projectile_details
                .iter()
                .any(|projectile| projectile.kind == ActiveProjectileKind::ShooterBlock),
            "ASM loc_EAE4 -> loc_EC76 should keep touch-triggered Highwater shooter bullets typed as shooter-block projectiles"
        );
        assert_eq!(
            rt.block_grid.as_ref().unwrap().cell_at(5, 3),
            terrain::Cell::from_be_bytes(0xE3, 0x1E),
            "ASM EvanescentBlock_Trigger rewrites the triggered shooter high byte to `$E3` while preserving the direction skin"
        );
        assert_eq!(
            rt.played_sound_effects,
            vec![QueuedSoundEffect::EvanescentBlock],
            "ASM EvanescentBlock_Trigger (kid.asm:17283) plays sfx_Evanescent_block before loading shooter bullets"
        );
    }

    #[test]
    fn highwater_pass_1_actual_ea24_column_fires_left_loc_eae4() {
        // ASM loc_EAE4 -> loc_EB72 scans body contact against the live level
        // layout, then EvanescentBlock_Trigger stores `skin - $1C` as direction
        // bits for loc_EC76. Highwater Pass 1's first post-rhino shooter column
        // is `$EA24` at (47,44), which must fire left (`$24 - $1C = $08`).
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.enter_level_playtest(LevelId(0x02));
        rt.progression.current_form = PlayerForm::Juggernaut;
        rt.projectiles.clear();

        let mut slice = shooter_test_slice(0, 0);
        slice.level_bounds = default_bounds();
        slice.level_bounds.width_pixels = 60 * 16;
        slice.level_bounds.height_pixels = 112 * 16;
        slice.player.x_pos = ((47 * 16 - 24) as u32) << 16;
        slice.player.y_pos = ((44 * 16 + 8) as u32) << 16;
        rt.step_evanescent_blocks(&slice);

        assert!(
            rt.projectiles
                .iter()
            .any(|projectile| projectile.x_velocity == -(4 << 16) && projectile.y_velocity == 0),
            "ASM loc_EC76 should fire a leftward shot from Highwater Pass 1's `$EA24` column"
        );
        assert!(
            rt.trace_frame()
                .active_projectile_details
                .iter()
                .any(|projectile| projectile.kind == ActiveProjectileKind::ShooterBlock),
            "ASM loc_EAE4 -> loc_EC76 should keep Highwater Pass 1 touch-triggered bullets typed as shooter-block projectiles"
        );
        assert_eq!(
            rt.block_grid.as_ref().unwrap().cell_at(47, 44),
            terrain::Cell::from_be_bytes(0xE3, 0x24),
            "ASM EvanescentBlock_Trigger should rewrite the actual `$EA24` cell to `$E324`"
        );
    }

    #[test]
    fn highwater_pass_1_actual_ea24_column_fires_through_frame_loop() {
        // ASM loc_806 -> sub_F096 runs after the main player/object loop, while
        // loc_EAE4 -> loc_EC76 is driven by the level block worker pass. The
        // normal native frame loop must still fire the first post-rhino
        // Highwater Pass 1 `$EA24` touch shooter; a direct worker call can hide
        // stale slice/order bugs.
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.enter_level_playtest(LevelId(0x02));
        rt.progression.current_form = PlayerForm::Juggernaut;
        rt.projectiles.clear();

        let mut slice = rt
            .opening_slice
            .expect("level should have an opening slice");
        slice.player.x_pos = ((47 * 16 - 24) as u32) << 16;
        slice.player.y_pos = ((44 * 16 + 8) as u32) << 16;
        slice.player.x_velocity = 0;
        slice.player.y_velocity = 0;
        slice.player.on_ground = false;
        rt.opening_slice = Some(slice);

        rt.step_frame(FrameInput::default());

        assert!(
            rt.projectiles
                .iter()
                .any(|projectile| projectile.x_velocity == -(4 << 16) && projectile.y_velocity == 0),
            "ASM loc_EC76 should fire a leftward shot from Highwater Pass 1's `$EA24` column through the normal frame loop"
        );
    }

    #[test]
    fn highwater_pass_1_actual_ea1e_column_fires_through_frame_loop() {
        // ASM loc_EAE4 -> loc_EB72 scans Kid's live hitbox over the level's
        // first Highwater Pass 1 touch-shooter column. The `$EA1E` skin stores
        // `skin - $1C == $02`, so loc_EC76 must fire right when Juggernaut
        // brushes the column through the normal frame loop.
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.enter_level_playtest(LevelId(0x02));
        rt.progression.current_form = PlayerForm::Juggernaut;
        rt.projectiles.clear();

        let mut slice = rt
            .opening_slice
            .expect("level should have an opening slice");
        slice.player.x_pos = ((11 * 16 - 24) as u32) << 16;
        slice.player.y_pos = ((97 * 16 + 8) as u32) << 16;
        slice.player.x_velocity = 0;
        slice.player.y_velocity = 0;
        slice.player.on_ground = false;
        rt.opening_slice = Some(slice);

        rt.step_frame(FrameInput::default());

        assert!(
            rt.projectiles
                .iter()
                .any(|projectile| projectile.x_velocity == 4 << 16 && projectile.y_velocity == 0),
            "ASM loc_EC76 should fire a rightward shot from Highwater Pass 1's `$EA1E` column through the normal frame loop"
        );
    }

    #[test]
    fn highwater_pass_1_touch_shooter_frame_loop_advances_once_loc_ec76_edfa() {
        // ASM loc_EB72 -> loc_EC76 immediately falls through to
        // Process_ShooterObjects at loc_EC22/loc_EDFA. The same just-fired
        // shooter object should not then receive a second loc_EDFA step later
        // in the native frame shell.
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.enter_level_playtest(LevelId(0x02));
        rt.progression.current_form = PlayerForm::Juggernaut;
        rt.projectiles.clear();

        let mut slice = rt
            .opening_slice
            .expect("level should have an opening slice");
        slice.player.x_pos = ((11 * 16 - 24) as u32) << 16;
        slice.player.y_pos = ((97 * 16 + 8) as u32) << 16;
        slice.player.x_velocity = 0;
        slice.player.y_velocity = 0;
        slice.player.on_ground = false;
        rt.opening_slice = Some(slice);

        rt.step_frame(FrameInput::default());

        let projectile = rt
            .projectiles
            .iter()
            .find(|projectile| {
                projectile.kind == EnemyProjectileKind::ShooterBlock
                    && projectile.x_velocity == 4 << 16
                    && projectile.y_velocity == 0
            })
            .expect("rightward Highwater shooter should exist after trigger frame");
        assert_eq!(
            projectile.x_pos, 0x00bc_0000,
            "ASM loc_EC76 -> loc_EDFA should advance the just-triggered shooter exactly once in its trigger frame"
        );
        assert_eq!(projectile.shooter_state_word, 0xFFFF);
    }

    #[test]
    fn highwater_pass_1_berzerker_charge_pose_fires_ea1e_through_frame_loop() {
        // ASM loc_9564 writes Berzerker's head-forward sprite, but loc_EAE4
        // still scans the sub_B41C `$07` Berzerker hitbox. Once that hitbox
        // reaches the first Highwater `$EA1E` touch-shooter column, it must
        // spawn the rightward shooter object.
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.enter_level_playtest(LevelId(0x02));
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.helmet.berzerker_charge_accum = 0xF0;
        rt.projectiles.clear();

        let mut slice = rt
            .opening_slice
            .expect("level should have an opening slice");
        slice.player.x_pos = ((11 * 16 - 8) as u32) << 16;
        slice.player.y_pos = ((97 * 16 + 8) as u32) << 16;
        slice.player.x_velocity = 0x0002_0000;
        slice.player.y_velocity = 0;
        slice.player.facing = Direction::Right;
        slice.player.on_ground = false;
        rt.opening_slice = Some(slice);

        rt.step_frame(FrameInput::default());

        assert!(
            rt.projectiles
                .iter()
                .any(|projectile| projectile.x_velocity == 4 << 16 && projectile.y_velocity == 0),
            "ASM loc_9564 -> loc_EAE4 -> loc_EC76 should fire a rightward shot from Highwater Pass 1's `$EA1E` column through the normal frame loop"
        );

        let barrier = rt.block_grid.as_ref().unwrap().cell_at(15, 97);
        assert!(
            barrier.has_block
                && barrier.collision == terrain::CollisionKind::Solid
                && barrier.block_type == terrain::BlockType::Ice
                && barrier.skin == 0x03,
            "Highwater Pass 1's first `$EA1E` rightward shot should initially face the ice barrier in the next layout column"
        );

        for _ in 0..13 {
            rt.step_frame(FrameInput::default());
        }

        assert_eq!(
            rt.block_grid.as_ref().unwrap().cell_at(15, 97),
            terrain::Cell::from_be_bytes(0, 0),
            "ASM loc_EE2A -> loc_F00A should let the first `$EA1E` shot clear Highwater Pass 1's ice barrier through the normal frame loop"
        );
    }

    #[test]
    fn highwater_pass_1_berzerker_live_charge_from_spawn_clears_first_barrier_loc_eae4_f02a() {
        // ASM loc_9564 -> loc_EAE4 -> loc_EC76 -> loc_F02A: the real route
        // enters Highwater Pass 1 at x=48/y=1632, builds Berzerker's head-forward
        // charge pose while moving right, touches the stacked `$EA1F` shooter
        // cells, and the fired shooter objects erase the `$E203` ice barrier.
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.enter_level_playtest(LevelId(0x02));
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.progression.number_hitpoints = PlayerForm::Berzerker.max_hitpoints();
        rt.projectiles.clear();

        for _ in 0..100 {
            rt.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }

        for row in 97..=103 {
            let cell = rt.block_grid.as_ref().unwrap().cell_at(15, row);
            assert!(
                cell.has_block && cell.block_type == terrain::BlockType::Ice,
                "ASM loc_EE2A should leave Highwater Pass 1 ice barrier row {row} intact until the shooter object reaches it on the oracle-timed live route"
            );
        }

        for _ in 100..110 {
            rt.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }

        let source_cell = rt.block_grid.as_ref().unwrap().cell_at(11, 103);
        assert!(
            !source_cell.has_block,
            "ASM loc_ED04 -> loc_ED52 should restore Highwater's lower source shooter terrain by the oracle frame-110 unblock window"
        );

        for _ in 110..120 {
            rt.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }

        for row in 97..=103 {
            let cell = rt.block_grid.as_ref().unwrap().cell_at(15, row);
            assert!(
                !cell.has_block,
                "ASM loc_EAE4 -> loc_EC76 -> loc_F02A should clear Highwater Pass 1 ice barrier row {row} on the live spawn-to-barrier route"
            );
        }
    }

    #[test]
    fn highwater_pass_1_touch_shooter_draws_on_trigger_frame_loc_ec76_edfa() {
        // ASM loc_EB72 -> loc_EC76 immediately falls through to
        // Process_ShooterObjects at loc_EC22/loc_EDFA, so the shooter object
        // takes its first 4px movement before the current frame is drawn.
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.enter_level_playtest(LevelId(0x02));
        rt.progression.current_form = PlayerForm::Juggernaut;
        rt.projectiles.clear();

        let mut slice = shooter_test_slice(0, 0);
        slice.level_bounds = default_bounds();
        slice.level_bounds.width_pixels = 60 * 16;
        slice.level_bounds.height_pixels = 112 * 16;
        slice.player.x_pos = ((11 * 16 - 24) as u32) << 16;
        slice.player.y_pos = ((97 * 16 + 8) as u32) << 16;
        rt.step_evanescent_blocks(&slice);

        assert_eq!(
            rt.projectiles[0].x_pos, 0x00bc_0000,
            "ASM loc_EC76 -> loc_EDFA should move the rightward Highwater shooter spike from cell centre `$00b8` to `$00bc` on the trigger frame"
        );
        assert_eq!(rt.projectiles[0].y_pos, 0x0618_0000);
        assert_eq!(rt.projectiles[0].shooter_state_word, 0xFFFF);
    }

    #[test]
    fn highwater_shooter_barrier_probe_formats_cell_words_loc_eae4_f02a() {
        // ASM loc_9564 -> loc_EAE4 -> loc_EC76 -> loc_F02A is the #214 live
        // playtest route. Keep this probe cache-independent so mutation tests
        // pin the cell-word encoding and level/game-mode gate directly.
        let mut rt = NativeRuntime::new(manifest());
        rt.boot_state = BootState::InGame;
        rt.current_level = LevelId(0x02);
        rt.set_block_grid(grid_with_cells(
            16,
            104,
            &[
                (11, 97, 0xE8, 0x1E),
                (11, 99, 0xEA, 0x1F),
                (11, 103, 0xE9, 0x19),
                (15, 97, 0xE2, 0x21),
                (15, 100, 0xEF, 0x1A),
                (15, 103, 0xE0, 0x44),
            ],
        ));
        rt.block_state.shooter_last_scanned_col = 10;
        rt.block_state.shooter_last_scanned_row = 96;
        rt.block_state.shooter_blocks = vec![ShooterBlockEntry {
            col: 11,
            row: 97,
            directions: 0x05,
            supported_by_neighbour: true,
            restored_touch_shooter: false,
            age: 7,
        }];
        rt.block_state.shooter_headbutt_workers = vec![ShooterHeadbuttWorker {
            col: 15,
            row: 98,
            age: 3,
            kind: ShooterHeadbuttWorkerKind::Center,
        }];
        rt.projectiles
            .push(EnemyProjectile::shooter_block_from_centre_loc_EC76(
                11,
                97,
                184,
                1560,
                4 << 16,
                -2 << 16,
            ));

        let probe = rt
            .trace_frame()
            .gameplay_debug_fields
            .get("highwater_shooter_barrier_probe")
            .cloned()
            .expect("level 02 in-game trace should expose the #214 probe");

        assert_eq!(
            probe,
            concat!(
                "scan=10,96 ",
                "slots=[11:97 dirs=0x05 age=7 supported=1] ",
                "headbutt=[15:98 Center age=3] ",
                "projectiles=[kind=ShooterBlock x=0x00b80000 y=0x06180000 ",
                "vel=0x00040000,0xfffe0000 life=120] ",
                "shooters=[11:97=E81E,11:98=0000,11:99=EA1F,11:100=0000,",
                "11:101=0000,11:102=0000,11:103=E919] ",
                "barrier=[15:97=E221,15:98=0000,15:99=0000,15:100=EF1A,",
                "15:101=0000,15:102=0000,15:103=E044]"
            ),
            "ASM loc_EAE4/loc_F02A trace probe should expose exact shooter/barrier layout words"
        );

        rt.current_level = LevelId(0x01);
        assert!(
            !rt.trace_frame()
                .gameplay_debug_fields
                .contains_key("highwater_shooter_barrier_probe"),
            "ASM loc_EAE4/loc_F02A probe should stay scoped to Highwater level 02"
        );

        rt.current_level = LevelId(0x02);
        rt.boot_state = BootState::TitleScreen { cursor: 0 };
        assert!(
            !rt.trace_frame()
                .gameplay_debug_fields
                .contains_key("highwater_shooter_barrier_probe"),
            "ASM loc_EAE4/loc_F02A probe should stay scoped to live gameplay"
        );
    }

    #[test]
    fn highwater_pass_1_trace_exposes_shooter_barrier_probe_loc_eae4_f02a() {
        // ASM loc_9564 -> loc_EAE4 -> loc_EC76 -> loc_F02A is the #214 live
        // playtest route. Trace needs the shooter slots, projectile, and
        // barrier cell words visible before patching the worker lifecycle.
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.enter_level_playtest(LevelId(0x02));
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.progression.number_hitpoints = PlayerForm::Berzerker.max_hitpoints();
        rt.projectiles.clear();

        for _ in 0..100 {
            rt.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }

        let trace = rt.trace_frame();
        let probe = trace
            .gameplay_debug_fields
            .get("highwater_shooter_barrier_probe")
            .expect("Highwater trace should expose #214 shooter/barrier probe");
        assert!(
            probe.contains("projectiles=[kind=ShooterBlock"),
            "probe should expose typed shooter-block projectile: {probe}"
        );
        assert!(
            probe.contains("shooters=[11:97="),
            "probe should expose the first Highwater shooter column: {probe}"
        );
        assert!(
            probe.contains("barrier=[15:97="),
            "probe should expose the first Highwater ice barrier column: {probe}"
        );
    }

    #[test]
    fn highwater_pass_1_projectile_triggered_evanescent_cells_expire_loc_f04a_eb72() {
        // ASM loc_F04A routes shooter-object hits on evanescent source cells
        // back through EvanescentBlock_Trigger. The #214 oracle recording shows
        // the upward shots cascading through Highwater's upper `$EA1E/$EA1F`
        // source cells, then those cells restore before the route tail; leaving
        // them as `$E3xx` is the stuck-top-block playtest bug.
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.enter_level_playtest(LevelId(0x02));
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.progression.number_hitpoints = PlayerForm::Berzerker.max_hitpoints();
        rt.projectiles.clear();

        for _ in 0..150 {
            rt.step_frame(FrameInput {
                right: true,
                ..FrameInput::default()
            });
        }

        for row in 97..=100 {
            let cell = rt.block_grid.as_ref().unwrap().cell_at(11, row);
            assert!(
                !cell.has_block,
                "ASM loc_F04A -> loc_EB72 -> loc_ED04 should expire projectile-triggered Highwater source row {row}, got {cell:?}"
            );
        }
    }

    #[test]
    fn loc_ed52_collision_overlay_opens_highwater_source_before_frame_109() {
        // ASM loc_EDB0 skips layout writes for `$80` actions. Native's folded
        // timer still has to expose the non-solid terrain before the #214
        // Highwater oracle route's f_109 side scan; waiting until post-f_109
        // leaves Kid one acceleration tick behind.
        assert!(
            !NativeRuntime::evanescent_sprite_only_overlay_enabled_loc_EDB0(15, 0x80),
            "timer 15 is still before the Highwater unblock window"
        );
        assert!(
            NativeRuntime::evanescent_sprite_only_overlay_enabled_loc_EDB0(16, 0x80),
            "timer 16 should expose the source cell before the f_109 side scan"
        );
        assert!(
            !NativeRuntime::evanescent_sprite_only_overlay_enabled_loc_EDB0(16, 0x81),
            "only `$80` sprite-only actions use this native folded-cadence bridge"
        );
    }

    #[test]
    fn highwater_pass_1_berzerker_charge_pose_reaches_ea1e_touch_shooter_loc_eae4() {
        // ASM loc_9564 selects Berzerker's head-forward `off_9688` mappings,
        // but loc_EAE4 still scans the live sub_B41C hitbox words. The first
        // Highwater `$EA1E` touch shooter fires only once Berzerker's `$07`
        // half-width reaches the column.
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.enter_level_playtest(LevelId(0x02));
        rt.progression.current_form = PlayerForm::Berzerker;
        rt.helmet.berzerker_charge_pose = true;
        rt.projectiles.clear();

        let mut slice = shooter_test_slice(0, 0);
        slice.level_bounds.width_pixels = 60 * 16;
        slice.level_bounds.height_pixels = 112 * 16;
        slice.player.x_pos = ((11 * 16 - 8) as u32) << 16;
        slice.player.y_pos = ((97 * 16 + 8) as u32) << 16;

        rt.step_evanescent_blocks(&slice);

        assert!(
            rt.projectiles
                .iter()
                .any(|projectile| projectile.x_velocity == 4 << 16 && projectile.y_velocity == 0),
            "ASM loc_EAE4 should let Berzerker's charge-pose hitbox touch the `$EA1E` shooter"
        );
        assert_eq!(
            rt.block_grid.as_ref().unwrap().cell_at(11, 97),
            terrain::Cell::from_be_bytes(0xE3, 0x1E),
            "ASM EvanescentBlock_Trigger should rewrite the touched `$EA1E` cell to `$E31E`"
        );
    }

    #[test]
    fn blue_lake_woods_2_live_rock_tank_shooting_spawns_tank_bullet_loc_3b730() {
        // ASM Tank_Shooting.asm Enemy0B_RockTank_shooting_Init and loc_3B5FA:
        // Blue Lake Woods 2 resource level `$4B` has a literal `$0B` object at
        // x=1544/y=383. Once its animation gate reaches loc_3C50E, the live
        // frame loop must allocate loc_3B730 and expose a typed TankBullet.
        let Some(mut rt) = cached_generated_runtime() else {
            return;
        };
        rt.enter_level_playtest(LevelId(0x01));
        rt.active_enemies.clear();
        rt.projectiles.clear();

        let mut slice = rt
            .opening_slice
            .expect("level should have an opening slice");
        slice.player.x_pos = 1544 << 16;
        slice.player.y_pos = 383 << 16;
        slice.player.x_velocity = 0;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.camera.x_pos = 1400 << 16;
        slice.camera.y_pos = 240 << 16;
        rt.opening_slice = Some(slice);

        let mut saw_shooting_tank = false;
        let mut saw_tank_bullet = false;
        for _ in 0..90 {
            let trace = rt.step_frame(FrameInput::default());
            saw_shooting_tank |= trace.active_objects.iter().any(|object| {
                matches!(object.kind, NativeObjectKind::Enemy { object_type: 0x0B })
                    && object.hit_points.is_some()
            });
            saw_tank_bullet |= trace.active_projectile_details.iter().any(|projectile| {
                projectile.kind == ActiveProjectileKind::TankBullet
                    && projectile.x_velocity != 0
                    && !projectile.friendly
            });
        }

        assert!(
            saw_shooting_tank,
            "Level 01's live `$0B` RockTank object should remain object_type 0x0B rather than collapsing to the plain 0x0A Tank path"
        );
        assert!(
            saw_tank_bullet,
            "ASM Tank_Shooting.asm loc_3C50E -> loc_3B730 should produce a hostile typed TankBullet in the live frame loop"
        );
    }

    #[test]
    fn shooter_block_pruned_when_cell_no_longer_shooter() {
        // If the grid cell changes (e.g. block destroyed), the shooter entry should be pruned.
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 3, 0x02)]));
        rt.block_state.shooter_last_scanned_col = 1;
        rt.block_state.shooter_last_scanned_row = 3;

        let mut slice = shooter_test_slice(0, 0);
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (3 * 16) << 16;
        rt.step_shooter_blocks(&slice);
        assert_eq!(rt.block_state.shooter_blocks.len(), 1);

        // Now overwrite the grid cell to empty
        rt.block_grid
            .as_mut()
            .unwrap()
            .set_cell_at(5, 3, terrain::Cell::from_be_bytes(0, 0));

        rt.step_shooter_blocks(&slice);
        assert!(
            rt.block_state.shooter_blocks.is_empty(),
            "shooter entry should be pruned when cell is no longer a shooter"
        );
    }

    #[test]
    fn shooter_headbutt_support_worker_runs_sub_114f4_timeline() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 4, 0x02)]));

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((5, 4));
        rt.process_block_effects(&mut slice);

        rt.step_shooter_headbutt_workers();
        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 3),
            terrain::Cell::from_be_bytes(0xEF, 0x1A),
            "ASM sub_114F4 writes `$EF1A` when the support cell is still empty"
        );

        for _ in 0..8 {
            rt.step_shooter_headbutt_workers();
        }
        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 3),
            terrain::Cell::from_be_bytes(0xEF, 0x1B),
            "ASM sub_114F4 writes `$EF1B` after the first 8-frame hibernate"
        );

        for _ in 0..8 {
            rt.step_shooter_headbutt_workers();
        }
        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 3),
            terrain::Cell::from_be_bytes(0xE9, 0x19),
            "ASM sub_114F4 finishes by restoring `$E919` to the support cell"
        );
    }

    #[test]
    fn shooter_headbutt_restored_support_cell_stays_non_shooter_without_scan_slot() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 4, 0x02)]));
        rt.block_state.shooter_last_scanned_col = 2;
        rt.block_state.shooter_last_scanned_row = 4;

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = (2 * 16) << 16;
        slice.player.y_pos = (4 * 16) << 16;
        slice.shooter_block_hit = Some((5, 4));
        rt.process_block_effects(&mut slice);

        for _ in 0..=16 {
            rt.step_shooter_headbutt_workers();
        }

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 3),
            terrain::Cell::from_be_bytes(0xE9, 0x19),
            "ASM sub_114F4 restores `$E919` to the empty support cell"
        );

        rt.step_shooter_blocks(&slice);

        assert_eq!(
            rt.projectiles.len(),
            0,
            "ASM sub_114F4 only restores the support tile; it does not register restored `$E919` as a shooter slot"
        );
    }

    #[test]
    fn shooter_headbutt_support_worker_loc_1152c_deletes_when_target_refills_before_wake() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_shooter_blocks(&[(5, 4, 0x02)]));

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shooter_block_hit = Some((5, 4));
        rt.process_block_effects(&mut slice);
        rt.block_grid.as_mut().expect("block grid").set_cell_at(
            5,
            3,
            terrain::Cell::from_be_bytes(0x60, 0),
        );

        rt.step_shooter_headbutt_workers();

        assert_eq!(
            rt.block_grid().expect("block grid").cell_at(5, 3),
            terrain::Cell::from_be_bytes(0x60, 0),
            "ASM loc_1152C deletes the support worker immediately if the support cell is no longer empty on wake"
        );
    }

    #[test]
    fn shooter_center_worker_gate_loc_1146e_restores_the_original_column_and_skips_centre_worker_after_support_spawn()
     {
        let no_support = NativeRuntime::shooter_center_worker_gate_loc_1146E(6, false);
        assert_eq!(
            no_support,
            ShooterCenterWorkerGateLoc1146E {
                restored_block_col: 5,
                should_queue_center_worker: true,
            },
            "ASM loc_1146E should undo loc_11458's extra `addq.w #1,d1` before falling through to loc_11474 when `d4` is still zero",
        );

        let support_spawned = NativeRuntime::shooter_center_worker_gate_loc_1146E(6, true);
        assert_eq!(
            support_spawned,
            ShooterCenterWorkerGateLoc1146E {
                restored_block_col: 5,
                should_queue_center_worker: false,
            },
            "ASM loc_1146E should branch straight to loc_11496 once any earlier `sub_1149C` call leaves `d4` non-zero",
        );
    }

    // --- Floating items: spawn delay, lifetime, collision boxes ---

    #[test]
    fn floating_item_with_spawn_delay_not_collectible() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 5,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;

        rt.collect_floating_items(&mut slice);

        assert!(
            !rt.floating_items[0].collected,
            "item with spawn_delay > 0 should not be collectible"
        );
    }

    #[test]
    fn floating_item_spawn_delay_counts_down_in_visual_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 3,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();
        assert_eq!(rt.floating_items[0].spawn_delay, 2);
        rt.step_visual_effects();
        assert_eq!(rt.floating_items[0].spawn_delay, 1);
        rt.step_visual_effects();
        assert_eq!(rt.floating_items[0].spawn_delay, 0);
        // Now the item should start falling
        rt.step_visual_effects();
        assert!(
            rt.floating_items[0].y_velocity > 0,
            "item should start accumulating velocity after delay expires"
        );
    }

    #[test]
    fn floating_item_removed_when_lifetime_expires() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 1,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();
        // remaining was 1, saturating_sub(1) = 0
        assert!(
            rt.floating_items.is_empty(),
            "item with remaining=0 should be removed"
        );
    }

    #[test]
    fn floating_item_loc_10396_keeps_the_frame_0xc8_countdown_alive() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 0x00C9,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10396 only branches to delete once `subq.w #1,d3` reaches zero; the `$00C8` frame stays alive for the late animation swap",
        );
        assert_eq!(
            rt.floating_items[0].remaining, 0x00C8,
            "ASM loc_10396 reaches the `cmpi.w #$C8,d3` animation threshold one frame before deletion becomes possible",
        );
    }

    #[test]
    fn floating_ankh_loc_1071e_keeps_the_frame_0x12c_countdown_alive() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 0x012D,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Ankh,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_1071E only branches to delete once `subq.w #1,d3` reaches zero; the `$012C` frame stays alive for the later animation swap",
        );
        assert_eq!(
            rt.floating_items[0].remaining, 0x012C,
            "ASM loc_1071E reaches the `cmpi.w #$12C,d3` animation threshold one frame before deletion becomes possible",
        );
    }

    #[test]
    fn floating_ankh_loc_1071e_collects_before_timeout_and_grants_the_life_bonus() {
        let mut rt = NativeRuntime::new(manifest());
        start_level(&mut rt);
        rt.floating_items.push(FloatingItem {
            x_pos: 72 << 16,
            y_pos: i64::from(100 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 4,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Ankh,
            collected: false,
            collision_latched: false,
        });
        let lives_before = rt.progression.number_lives;
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 72 << 16;
        slice.player.y_pos = 110 << 16;

        rt.collect_floating_items(&mut slice);

        assert!(rt.floating_items[0].collected);
        assert_eq!(
            rt.progression.number_lives,
            lives_before + 1,
            "ASM loc_1071E should take the collision path before timeout expiry so the collected ankh still grants its life bonus",
        );

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_1071E jumps to `Delete_CurrentObject` after the collection path allocates the follow-up worker",
        );
    }

    #[test]
    fn floating_ankh_loc_10738_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (4 * 16 + 8) << 16,
            y_pos: i64::from(160 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 4,
            level_row: 10,
            first_row_grace: true,
            content: terrain::PrizeContent::Ankh,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10738 should keep looping while the ankh stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_10738 still burns one shared falling-prize countdown step before the later delete exit",
        );
    }

    #[test]
    fn floating_ankh_loc_10768_deletes_the_collected_ankh_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Ankh,
            collected: true,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_10768 should jump straight to `Delete_CurrentObject` once PrizeAnkh's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_skycutter_loc_10464_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (2 * 16 + 8) << 16,
            y_pos: i64::from(128 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 2,
            level_row: 8,
            first_row_grace: true,
            content: terrain::PrizeContent::Skycutter,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10464 should keep looping while Skycutter stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_10464 still runs the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_item_collected_flag_causes_removal() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: true,
            collision_latched: false,
        });

        rt.step_visual_effects();
        assert!(
            rt.floating_items.is_empty(),
            "collected item should be removed in visual step"
        );
    }

    #[test]
    fn floating_skycutter_loc_1047c_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Skycutter,
            collected: true,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_1047C should jump straight to `Delete_CurrentObject` once Skycutter's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_cyclone_loc_104ae_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (3 * 16 + 8) << 16,
            y_pos: i64::from(144 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 3,
            level_row: 9,
            first_row_grace: true,
            content: terrain::PrizeContent::Cyclone,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_104AE should keep looping while Cyclone stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_104AE still advances the shared helmet countdown on each hibernated frame before the shared delete exit",
        );
    }

    #[test]
    fn floating_cyclone_loc_104c6_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Cyclone,
            collected: true,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_104C6 should jump straight to `Delete_CurrentObject` once Cyclone's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_red_stealth_loc_104f8_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (4 * 16 + 8) << 16,
            y_pos: i64::from(160 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 4,
            level_row: 10,
            first_row_grace: true,
            content: terrain::PrizeContent::RedStealth,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_104F8 should keep looping while Red Stealth stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_104F8 still advances the shared helmet countdown on each hibernated frame before the shared delete exit",
        );
    }

    #[test]
    fn floating_red_stealth_loc_10510_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::RedStealth,
            collected: true,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_10510 should jump straight to `Delete_CurrentObject` once Red Stealth's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_eyeclops_loc_10542_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (5 * 16 + 8) << 16,
            y_pos: i64::from(176 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 5,
            level_row: 11,
            first_row_grace: true,
            content: terrain::PrizeContent::Eyeclops,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10542 should keep looping while Eyeclops stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_10542 still advances the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_eyeclops_loc_1055a_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Eyeclops,
            collected: true,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_1055A should jump straight to `Delete_CurrentObject` once Eyeclops' `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_juggernaut_loc_1058c_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (6 * 16 + 8) << 16,
            y_pos: i64::from(192 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 12,
            first_row_grace: true,
            content: terrain::PrizeContent::Juggernaut,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_1058C should keep looping while Juggernaut stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_1058C still advances the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_iron_knight_loc_105d6_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (7 * 16 + 8) << 16,
            y_pos: i64::from(208 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 7,
            level_row: 13,
            first_row_grace: true,
            content: terrain::PrizeContent::IronKnight,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_105D6 should keep looping while Iron Knight stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_105D6 still advances the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_iron_knight_loc_105ee_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::IronKnight,
            collected: true,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_105EE should jump straight to `Delete_CurrentObject` once Iron Knight's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_berzerker_loc_10620_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (8 * 16 + 8) << 16,
            y_pos: i64::from(224 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 8,
            level_row: 14,
            first_row_grace: true,
            content: terrain::PrizeContent::Berzerker,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10620 should keep looping while Berzerker stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_10620 still advances the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_berzerker_loc_10638_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 116 << 16,
            y_pos: 132 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 7,
            level_row: 8,
            first_row_grace: true,
            content: terrain::PrizeContent::Berzerker,
            collected: true,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_10638 should jump straight to `Delete_CurrentObject` once Berzerker's `collision_type(a3)` becomes non-zero",
        );
    }

    #[test]
    fn floating_maniaxe_loc_1066a_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (9 * 16 + 8) << 16,
            y_pos: i64::from(240 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 9,
            level_row: 15,
            first_row_grace: true,
            content: terrain::PrizeContent::Maniaxe,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_1066A should keep looping while Maniaxe stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_1066A still advances the shared helmet countdown on each hibernated frame before the later delete exit",
        );
    }

    #[test]
    fn floating_micromax_loc_106b4_stays_alive_while_above_level_bottom_and_uncollected() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (10 * 16 + 8) << 16,
            y_pos: i64::from(256 << 16),
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 10,
            level_row: 16,
            first_row_grace: true,
            content: terrain::PrizeContent::Micromax,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_106B4 should keep looping while Micromax stays above `Level_height_pixels` and `collision_type(a3)` remains zero",
        );
        assert_eq!(
            rt.floating_items[0].remaining,
            NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES - 1,
            "ASM loc_106B4 still advances the shared helmet countdown on each hibernated frame before the shared delete exit",
        );
    }

    #[test]
    fn floating_micromax_delete_exit_loc_106cc_jumps_straight_to_delete_current_object() {
        assert!(
            NativeRuntime::floating_micromax_delete_exit_loc_106CC(),
            "ASM loc_106CC should jump straight to `Delete_CurrentObject` once Micromax leaves loc_106B4",
        );
    }

    #[test]
    fn floating_micromax_loc_106cc_deletes_the_collected_helmet_on_the_next_step() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 132 << 16,
            y_pos: 148 << 16,
            y_velocity: 0,
            remaining: NativeRuntime::PRIZE_GENERIC_LIFETIME_FRAMES,
            spawn_delay: 0,
            level_col: 8,
            level_row: 9,
            first_row_grace: true,
            content: terrain::PrizeContent::Micromax,
            collected: true,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_106CC should jump straight to `Delete_CurrentObject` once Micromax's `collision_type(a3)` becomes non-zero",
        );
    }

    // --- Visual effects: countdown and retention ---

    #[test]
    fn block_bounce_anim_counts_down_and_is_removed() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_bounces.push(BlockBounceAnim {
            cell_x: 3,
            cell_y: 5,
            x_offset_px: 0,
            remaining: 2,
            initial_duration: 8,
            kind: BlockBounceKind::Metal,
            prize_content: None,
            prize_deleted_tile: false,
            metal_direction: NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE,
            y_offset_px: 0,
        });

        rt.step_visual_effects();
        assert_eq!(rt.block_bounces.len(), 1);
        assert_eq!(rt.block_bounces[0].remaining, 1);

        rt.step_visual_effects();
        assert!(
            rt.block_bounces.is_empty(),
            "bounce anim should be removed when remaining reaches 0"
        );
    }

    #[test]
    fn block_destroy_effect_counts_down_and_is_removed() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_destroy_effects.push(BlockDestroyEffect {
            x_px: 80,
            y_px: 48,
            remaining: 1,
            direction: NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE,
            pending_ice_origin_nudge: false,
        });

        rt.step_visual_effects();
        assert!(
            rt.block_destroy_effects.is_empty(),
            "destroy effect should be removed when remaining reaches 0"
        );
    }

    #[test]
    fn block_destroy_effect_hands_off_to_falling_sub_2a4c_debris() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_destroy_effects.push(BlockDestroyEffect {
            x_px: 80,
            y_px: 48,
            remaining: 1,
            direction: NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE,
            pending_ice_origin_nudge: false,
        });

        rt.step_visual_effects();

        assert!(rt.block_destroy_effects.is_empty());
        assert_eq!(
            rt.trace_frame().block_debris_pieces,
            vec![(85, 47), (89, 44), (89, 53), (79, 53), (79, 44)],
            "ASM loc_2994 writes scratch records with d7 descending from 4 to 0, then sub_2A4C advances them"
        );

        rt.step_visual_effects();
        assert_eq!(
            rt.trace_frame().block_debris_pieces[0],
            (86, 42),
            "ASM sub_2A4C should keep advancing the shard instead of deleting it after the smash worker hibernate"
        );
    }

    #[test]
    fn block_debris_velocity_tables_match_asm_sub_292e_all_directions() {
        // ASM sub_292E (kid.asm:4881) dispatches `d6` (the block-destroy
        // direction) to one of four velocity tables, paired with the shared
        // base-position offsets unk_29E8/unk_29F2 (x = [0,0,8,8,4], y =
        // [0,8,8,0,4]). loc_2994's d7 DBF loop writes indices 4 through 0;
        // sub_2A4C then advances each shard once per frame.
        //
        // The native tables (blocks.rs:1772-1779) are byte-for-byte the ROM
        // word tables, verified against kid.bin:
        //   ABOVE  (d6=2, loc_2988): x=unk_29FC [-1,-1, 1, 1, 1]
        //                            y=unk_2A06 [-4,-3,-3,-4,-5]
        //   BELOW  (d6=0, loc_297A): x=unk_2A10 [-1,-2, 1, 1,-1]
        //                            y=unk_2A1A [-2, 0, 0,-1,-2]
        //   LEFT   (d6=1, loc_296C): x=unk_2A38 [-5,-4,-3,-2,-1]
        //                            y=unk_2A42 [-3,-3,-3,-3,-3]
        //   RIGHT  (else,  loc_2994): x=unk_2A24 [ 5, 4, 3, 2, 1]
        //                             y=unk_2A2E [-3,-3,-3,-3,-3]
        //
        // ABOVE is already pinned by
        // `block_destroy_effect_hands_off_to_falling_sub_2a4c_debris`; the other
        // three match arms (block_debris_velocity_sub_292e) had no exact-value
        // test, so deleting the BELOW/LEFT arms (they fall through to the RIGHT
        // default) or swapping a table survived mutation. This pins all three.
        //
        // Each expected piece is the seed (x_px+base_x, y_px+base_y) advanced by
        // one velocity step; gravity (0x36B0) is sub-pixel on the first frame, so
        // the integer pixel only reflects the table velocity. x_px=80,y_px=48
        // keeps every shard on-camera (same anchor as the ABOVE test).
        let pieces_for = |direction: u16| {
            let mut rt = NativeRuntime::new(manifest());
            rt.block_destroy_effects.push(BlockDestroyEffect {
                x_px: 80,
                y_px: 48,
                remaining: 1,
                direction,
                pending_ice_origin_nudge: false,
            });
            rt.step_visual_effects();
            rt.trace_frame().block_debris_pieces
        };

        assert_eq!(
            pieces_for(NativeRuntime::BLOCK_DESTROY_DIRECTION_BELOW),
            vec![(83, 50), (89, 47), (89, 56), (78, 56), (79, 46)],
            "ASM loc_297A: d6=0 selects unk_2A10/unk_2A1A (BELOW) velocity table"
        );
        assert_eq!(
            pieces_for(NativeRuntime::BLOCK_DESTROY_DIRECTION_LEFT),
            vec![(83, 49), (86, 45), (85, 53), (76, 53), (75, 45)],
            "ASM loc_296C: d6=1 selects unk_2A38/unk_2A42 (LEFT) velocity table"
        );
        assert_eq!(
            pieces_for(NativeRuntime::BLOCK_DESTROY_DIRECTION_RIGHT),
            vec![(85, 49), (90, 45), (91, 53), (84, 53), (85, 45)],
            "ASM loc_2994 default: d6=3 selects unk_2A24/unk_2A2E (RIGHT) velocity table"
        );
    }

    #[test]
    fn live_block_debris_uses_six_slot_scratch_ring_sub_292e() {
        let mut rt = NativeRuntime::new(manifest());
        for index in 0..7 {
            rt.block_destroy_effects.push(BlockDestroyEffect {
                x_px: 16 + index * 16,
                y_px: 48,
                remaining: 1,
                direction: NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE,
                pending_ice_origin_nudge: false,
            });
        }

        rt.step_visual_effects();

        assert_eq!(
            rt.block_debris_swarms.len(),
            6,
            "ASM sub_292E decrements `$FFFFFAA0` from six and rewinds `$FFFFFA9C`; a seventh same-frame debris spawn overwrites slot 0 instead of appending a seventh worker"
        );
        assert_eq!(
            rt.block_debris_swarms[0].pieces[0].x_pos_fp >> 16,
            16 + 6 * 16 + 4 + 1,
            "ASM loc_29DE/sub_292E wraps the cursor and replaces the first scratch slot with the seventh debris swarm; loc_2994's descending d7 makes piece 0 the source-index-4 entry (base_x +4, D6_2 x-velocity +1)"
        );
    }

    #[test]
    fn highwater_block_debris_culls_against_camera_not_world_origin_sub_2a4c() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(LevelBounds {
            width_pixels: 320,
            height_pixels: 2048,
        });
        slice.camera.y_pos = 1500 << 16;
        rt.opening_slice = Some(slice);
        rt.block_destroy_effects.clear();
        rt.block_debris_swarms.clear();
        rt.block_destroy_effects.push(BlockDestroyEffect {
            x_px: 244,
            y_px: 1552,
            remaining: 1,
            direction: NativeRuntime::BLOCK_DESTROY_DIRECTION_RIGHT,
            pending_ice_origin_nudge: false,
        });

        rt.step_visual_effects();

        assert!(
            !rt.trace_frame().block_debris_pieces.is_empty(),
            "ASM sub_2A4C culls debris after subtracting Camera_Y_pos; Highwater shards around world y=1552 should remain visible when the camera is on that band"
        );
    }

    #[test]
    fn live_block_debris_tail_fragment_culls_slot_sub_2a4c() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_debris_swarms.clear();
        rt.block_debris_swarms.push(BlockDebrisSwarmSub2A4C {
            pieces: core::array::from_fn(|index| BlockDebrisPieceSub2A4C {
                x_pos_fp: if index == 4 {
                    0x01EF << 16
                } else {
                    0x0020 << 16
                },
                y_pos_fp: 0x0040 << 16,
                x_velocity_fp: 0,
                y_velocity_fp: 0,
            }),
        });

        rt.step_visual_effects();

        assert!(
            rt.block_debris_swarms.is_empty(),
            "ASM sub_2A4C clears the whole scratch slot once the tail fragment reaches relative x `$1EF`, even if the lead fragment is still visible"
        );
        assert_eq!(
            rt.trace_frame().block_debris_pieces,
            vec![(0x0020, 0x0040); 4],
            "ASM sub_2A4C emits the first four debris sprites before the tail fragment `$1EF` check clears the scratch slot"
        );
    }

    #[test]
    fn prize_block_spawn_flash_counts_down_and_is_traced_with_loc_102b8_position() {
        let mut rt = NativeRuntime::new(manifest());
        rt.prize_block_spawn_flashes.push(PrizeBlockSpawnFlash {
            x_px: 56,
            y_px: 24,
            remaining: 2,
        });

        let frame = rt.trace_frame();
        assert_eq!(
            frame.prize_block_spawn_flashes,
            vec![(56, 24, 2)],
            "ASM sub_1007A loc_102B8 flash position must be available to the renderer"
        );

        rt.step_visual_effects();
        assert_eq!(rt.prize_block_spawn_flashes[0].remaining, 1);

        rt.step_visual_effects();
        assert!(
            rt.prize_block_spawn_flashes.is_empty(),
            "loc_102B8 flash worker should be removed after stru_102DE finishes"
        );
    }

    #[test]
    fn enemy_death_effect_counts_down_and_is_removed() {
        let mut rt = NativeRuntime::new(manifest());
        rt.enemy_death_effects.push(EnemyDeathEffect {
            x_pos: 100,
            y_pos: 200,
            remaining: 1,
            heady_metal_split: None,
        });

        rt.step_visual_effects();
        assert!(
            rt.enemy_death_effects.is_empty(),
            "enemy death effect should be removed when remaining reaches 0"
        );
    }

    #[test]
    fn heady_metal_death_effect_uses_loc_37922_cleanup_when_countdown_reaches_zero() {
        let mut rt = NativeRuntime::new(manifest());
        let pieces = [
            HeadyMetalDeathPieceState {
                slot_offset: 0xFB72,
                x_pos: 100,
                y_pos: 200,
                x_velocity: -0x0002_6D18,
                y_velocity: 0,
            },
            HeadyMetalDeathPieceState {
                slot_offset: 0xFB76,
                x_pos: 100,
                y_pos: 200,
                x_velocity: 0x0002_6D18,
                y_velocity: 0,
            },
            HeadyMetalDeathPieceState {
                slot_offset: 0xFB7A,
                x_pos: 100,
                y_pos: 200,
                x_velocity: 0,
                y_velocity: 0x0001_3880,
            },
            HeadyMetalDeathPieceState {
                slot_offset: 0xFBAE,
                x_pos: 100,
                y_pos: 200,
                x_velocity: -0x0002_6D18,
                y_velocity: 0,
            },
            HeadyMetalDeathPieceState {
                slot_offset: 0xFBB2,
                x_pos: 100,
                y_pos: 200,
                x_velocity: 0x0002_6D18,
                y_velocity: 0,
            },
        ];
        rt.enemy_death_effects.push(EnemyDeathEffect {
            x_pos: 100,
            y_pos: 200,
            remaining: 1,
            heady_metal_split: Some(HeadyMetalDeathEffectSplit {
                current_x_velocity_delta: -0x0002_6D18,
                left_worker_x_velocity_delta: 0x0002_6D18,
                centre_worker_y_velocity_delta: 0x0001_3880,
                centre_worker_sprite_label: Some("LnkTo_unk_C8458-Data_Index"),
                right_worker_x_velocity_delta: -0x0002_6D18,
                far_right_worker_x_velocity_delta: 0x0002_6D18,
                cleanup_worker_slot_offsets: [0xFB72, 0xFB76, 0xFB7A, 0xFBAE, 0xFBB2],
                pieces,
            }),
        });

        rt.step_visual_effects();

        assert!(
            rt.enemy_death_effects.is_empty(),
            "ASM sub_378C8 loc_37922 deletes and clears the five HeadyMetal cached workers once the latched `$42(a3)` countdown reaches zero"
        );
    }

    #[test]
    fn heady_metal_death_effect_applies_loc_378d6_piece_impulse_while_countdown_survives() {
        let mut rt = NativeRuntime::new(manifest());
        rt.spawn_enemy_death_effect(EnemyFamily::HeadyMetal, 100, 200);

        rt.step_visual_effects();

        let split = rt.enemy_death_effects[0]
            .heady_metal_split
            .expect("HeadyMetal death shell should retain split piece state");
        assert_eq!(
            rt.enemy_death_effects[0].remaining,
            HEADYMETAL_DEATH_EFFECT_FRAMES - 1,
            "ASM sub_378C8 decrements `$42(a3)` once while the final HeadyMetal breakup shell survives",
        );
        assert_eq!(
            split.pieces[0].x_velocity, -0x0004_DA30,
            "ASM loc_378D6 should add `#-$26D18` to `$FFFFFB72`'s x velocity on each surviving cleanup tick",
        );
        assert_eq!(
            split.pieces[0].x_pos,
            100u32.wrapping_add((-0x0004_DA30i32) as u32),
            "native shell should advance the first HeadyMetal death piece with the updated fixed-point velocity",
        );
        assert_eq!(
            split.pieces[2].y_velocity, 0x0002_7100,
            "ASM loc_378D6 should add `#$13880` to `$FFFFFB7A`'s y velocity on each surviving cleanup tick",
        );
        assert_eq!(
            split.pieces[2].y_pos,
            200u32.wrapping_add(0x0002_7100),
            "native shell should advance the centre HeadyMetal death piece with the updated fixed-point velocity",
        );
    }

    // --- Evanescent blocks: max entries cap ---

    #[test]
    fn evanescent_block_respects_max_entries_cap() {
        // ASM limits evanescent tracking to 19 ($13) entries.
        let grid = grid_with_evanescent(2, 3, 28);
        let mut runtime = runtime_with_evanescent_grid(grid);

        // Pre-fill to capacity
        for i in 0..EVANESCENT_MAX_ENTRIES {
            let slot_addr = runtime
                .block_state
                .allocate_evanescent_slot_loc_DF68()
                .expect("test prefill should consume the loc_DF30 evanescent free list");
            runtime.block_state.evanescent_blocks.push(EvanescentEntry {
                slot_addr,
                col: 10 + i as u16,
                row: 10,
                directions: 0,
                timer: 0,
                bullets_loaded: true,
            });
        }
        assert_eq!(
            runtime.block_state.evanescent_blocks.len(),
            EVANESCENT_MAX_ENTRIES
        );

        // Player standing on the evanescent block at (2, 3)
        let slice = {
            let mut s = shooter_test_slice(0, 0);
            s.player.on_ground = true;
            s.player.x_pos = (2 * 16 + 8) << 16;
            s.player.y_pos = (3 * 16 - 1) << 16;
            s
        };

        runtime.step_evanescent_blocks(&slice);

        assert_eq!(
            runtime.block_state.evanescent_blocks.len(),
            EVANESCENT_MAX_ENTRIES,
            "should not exceed max entries cap"
        );
        assert!(
            runtime.played_sound_effects.is_empty(),
            "the native loc_EAE4 scan skips entering EvanescentBlock_Trigger when the slot list is already full"
        );

        runtime.trigger_evanescent_block_from_shooter_loc_F04A(2, 3, 28);
        assert_eq!(
            runtime.block_state.evanescent_blocks.len(),
            EVANESCENT_MAX_ENTRIES,
            "the trigger routine itself still must not allocate past the cap"
        );
        assert_eq!(
            runtime.played_sound_effects,
            vec![QueuedSoundEffect::EvanescentBlock],
            "ASM EvanescentBlock_Trigger (kid.asm:17283) plays sfx_Evanescent_block before the free-slot bailout"
        );
    }

    // --- Lift platform: player carry-along ---

    #[test]
    fn lift_platform_carries_grounded_player() {
        // When player is on_ground and within snap tolerance of the lift, the lift
        // should carry the player's y_pos along.
        let mut rt = NativeRuntime::new(manifest());
        let lift_y: u32 = 100 << 16;
        rt.block_state.lift_platforms.push(LiftPlatform {
            x_pos: 32 << 16,
            y_pos: lift_y,
            y_velocity: 0,
            age: 0,
            last_integer_y: 100,
            moved_y_steps: 0,
            width_px: 16,
            source_col: 2,
            source_row: 6,
            block_erased: false,
        });

        let mut slice = OpeningSliceState::new(default_bounds());
        // Player standing on the lift
        slice.player.x_pos = 40 << 16; // within platform x range
        slice.player.y_pos = lift_y;
        slice.player.on_ground = true;
        slice.terrain.floor_y_pos = lift_y;

        let outcome = rt.step_lift_platforms(&mut slice);
        assert_eq!(
            outcome,
            PlatformCollisionOutcome::StandingOn(
                crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C
            ),
            "ASM sub_FB3E allocates a Platform_RAM slot before sub_FCA6, so lift landing should expose Addr_PlatformStandingOn"
        );
        assert_eq!(
            rt.block_state.lift_platforms[0].y_pos, lift_y,
            "ASM sub_FCA6 should leave `6(a3)` unchanged on the first frame because `$E(a3)` starts at zero",
        );
        assert_eq!(
            slice.player.y_pos,
            99 << 16,
            "ASM loc_8DA0 reseats grounded riders to platform_y - 1 even before the lift's integer `6(a3)` word changes",
        );

        let outcome = rt.step_lift_platforms(&mut slice);
        assert_eq!(
            outcome,
            PlatformCollisionOutcome::StandingOn(
                crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C
            ),
            "ASM loc_7828 keeps carrying through the stored lift Platform_RAM slot"
        );
        let expected_lift_y = 0x0063_F8F8u32;
        assert_eq!(
            rt.block_state.lift_platforms[0].y_pos, expected_lift_y,
            "ASM sub_FCA6 should move the lift by the fractional `sub_FD00` step on the second frame",
        );
        assert_eq!(
            slice.player.y_pos,
            expected_lift_y.wrapping_sub(0x0001_0000),
            "ASM loc_8DA0 copies the lift's current 16.16 y_pos then subtracts one pixel",
        );
    }

    #[test]
    fn lift_platform_loc_78a8_crushes_rider_into_ceiling() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::TheKid;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&[0; 8 * 8 * 2], 8, 8));
        rt.block_state.lift_platforms.push(LiftPlatform {
            x_pos: 32 << 16,
            y_pos: 30 << 16,
            y_velocity: -0x0001_0000,
            age: 0,
            last_integer_y: 30,
            moved_y_steps: 0,
            width_px: 16,
            source_col: 2,
            source_row: 6,
            block_erased: true,
        });
        rt.active_lift_platform_standing_on =
            Some(crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C);

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 40 << 16;
        slice.player.y_pos = 30 << 16;
        slice.player.on_ground = true;
        slice.terrain.floor_y_pos = 30 << 16;

        let outcome = rt.step_lift_platforms(&mut slice);

        assert_eq!(
            outcome,
            PlatformCollisionOutcome::CrushDeath,
            "ASM loc_78A8/loc_78C0: upward lift platform y_vel after ceiling contact should route to Death",
        );
    }

    #[test]
    fn lift_platform_support_uses_current_form_width_sub_8ed0() {
        let mut rt = NativeRuntime::new(manifest());
        rt.progression.current_form = PlayerForm::Juggernaut;
        rt.block_state.lift_platforms.push(LiftPlatform {
            x_pos: 32 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            age: 0,
            last_integer_y: 100,
            moved_y_steps: 0,
            width_px: 16,
            source_col: 2,
            source_row: 6,
            block_erased: true,
        });

        let mut slice = OpeningSliceState::new(default_bounds());
        // The Kid's hardcoded half-width 5 would miss this platform
        // horizontally, but Juggernaut's ASM sub_8ED0 width overlaps it.
        slice.player.x_pos = 60 << 16;
        slice.player.y_pos = 100 << 16;
        slice.player.y_velocity = 0;
        slice.player.on_ground = false;

        let outcome = rt.step_lift_platforms(&mut slice);

        assert_eq!(
            outcome,
            PlatformCollisionOutcome::StandingOn(
                crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C
            ),
            "ASM sub_FB3E/sub_FCA6 should surface a platform slot for sub_8ED0 support"
        );
        assert!(
            slice.player.on_ground,
            "ASM sub_8ED0 uses Current_Helmet width, so Juggernaut should still be supported by this lift",
        );
    }

    #[test]
    fn lift_platform_mount_snaps_to_platform_top_minus_one_loc_8da0() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_state.lift_platforms.push(LiftPlatform {
            x_pos: 32 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            age: 0,
            last_integer_y: 100,
            moved_y_steps: 0,
            width_px: 16,
            source_col: 2,
            source_row: 6,
            block_erased: true,
        });

        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 40 << 16;
        slice.player.y_pos = 100 << 16;
        slice.player.y_velocity = 0;
        slice.player.on_ground = false;

        let outcome = rt.step_lift_platforms(&mut slice);

        assert_eq!(
            outcome,
            PlatformCollisionOutcome::StandingOn(
                crate::platforms::PLATFORM_RAM_FIRST_SLOT_ADDR_LOC_231C
            ),
            "ASM sub_FB3E allocates a Platform_RAM slot before the lift can support Kid",
        );
        assert_eq!(
            slice.player.y_pos,
            99 << 16,
            "ASM loc_8DA0 copies platform_y then subtracts one pixel, instead of additively carrying on the mount frame",
        );
    }

    #[test]
    fn lift_block_bob_deltas_sub_fb3e_match_the_three_hibernating_passes() {
        let deltas = NativeRuntime::lift_block_bob_deltas_sub_FB3E();

        assert_eq!(
            deltas,
            &[
                -2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, 2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, -2,
                -2, -2, -2, 2, 2, 2, 2, 2,
            ],
            "ASM sub_FB3E should bob the struck lift block through the `6/6 -> 5/5 -> 4/5` up/down hibernate passes before the platform hand-off",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            2,
            "ASM sub_FB3E should finish the bob cycle 2px below the starting point before the later animation and platform setup correct the anchor",
        );
    }

    #[test]
    fn lift_block_bob_delta_sub_fb3e_indexes_pass_boundaries_and_tail() {
        // ASM sub_FB3E loc_FB94/loc_FBA2/loc_FBB0/loc_FBBE/loc_FBCC/loc_FBDA:
        // each DBF-controlled pass owns a contiguous slice of the bob table;
        // elapsed == 31 is the first post-table frame.
        let cases = [
            (0, Some(-2)),
            (5, Some(-2)),
            (6, Some(2)),
            (11, Some(2)),
            (12, Some(-2)),
            (16, Some(-2)),
            (17, Some(2)),
            (21, Some(2)),
            (22, Some(-2)),
            (25, Some(-2)),
            (26, Some(2)),
            (30, Some(2)),
            (31, None),
        ];

        for (elapsed, expected) in cases {
            assert_eq!(
                NativeRuntime::lift_block_bob_delta_sub_FB3E(elapsed),
                expected,
                "ASM sub_FB3E bob delta at elapsed frame {elapsed}"
            );
        }
    }

    #[test]
    fn lift_block_bob_up_pass_loc_fb94_matches_the_first_hibernating_loop() {
        let deltas = NativeRuntime::lift_block_bob_up_pass_loc_FB94();

        assert_eq!(
            deltas,
            &[-2, -2, -2, -2, -2, -2],
            "ASM loc_FB94 should execute six `subq.w #2,y_pos(a3)` steps because `moveq #5,d0` feeds an inclusive DBF loop",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            -12,
            "ASM loc_FB94 should move the struck lift block 12px upward before the first down pass starts",
        );
    }

    #[test]
    fn lift_block_bob_down_pass_loc_fba2_matches_the_first_return_loop() {
        let deltas = NativeRuntime::lift_block_bob_down_pass_loc_FBA2();

        assert_eq!(
            deltas,
            &[2, 2, 2, 2, 2, 2],
            "ASM loc_FBA2 should execute six `addq.w #2,y_pos(a3)` steps because `moveq #5,d1` feeds an inclusive DBF loop",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            12,
            "ASM loc_FBA2 should return the struck lift block 12px downward before the five-frame up pass starts",
        );
    }

    #[test]
    fn lift_block_bob_up_pass_loc_fbb0_matches_the_second_hibernating_loop() {
        let deltas = NativeRuntime::lift_block_bob_up_pass_loc_FBB0();

        assert_eq!(
            deltas,
            &[-2, -2, -2, -2, -2],
            "ASM loc_FBB0 should execute five `subq.w #2,y_pos(a3)` steps because `moveq #4,d0` feeds an inclusive DBF loop",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            -10,
            "ASM loc_FBB0 should move the struck lift block 10px upward before the second return pass starts",
        );
    }

    #[test]
    fn lift_block_bob_down_pass_loc_fbbe_matches_the_second_return_loop() {
        let deltas = NativeRuntime::lift_block_bob_down_pass_loc_FBBE();

        assert_eq!(
            deltas,
            &[2, 2, 2, 2, 2],
            "ASM loc_FBBE should execute five `addq.w #2,y_pos(a3)` steps because `moveq #4,d1` feeds an inclusive DBF loop",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            10,
            "ASM loc_FBBE should return the struck lift block 10px downward before the four-frame up pass starts",
        );
    }

    #[test]
    fn lift_block_bob_up_pass_loc_fbcc_matches_the_final_hibernating_loop() {
        let deltas = NativeRuntime::lift_block_bob_up_pass_loc_FBCC();

        assert_eq!(
            deltas,
            &[-2, -2, -2, -2],
            "ASM loc_FBCC should execute four `subq.w #2,y_pos(a3)` steps because `moveq #3,d0` feeds an inclusive DBF loop",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            -8,
            "ASM loc_FBCC should move the struck lift block 8px upward before the final five-frame return pass starts",
        );
    }

    #[test]
    fn lift_block_bob_down_pass_loc_fbda_matches_the_final_return_loop() {
        let deltas = NativeRuntime::lift_block_bob_down_pass_loc_FBDA();

        assert_eq!(
            deltas,
            &[2, 2, 2, 2, 2],
            "ASM loc_FBDA should execute five `addq.w #2,y_pos(a3)` steps because `moveq #4,d1` feeds an inclusive DBF loop",
        );
        assert_eq!(
            deltas.iter().copied().sum::<i16>(),
            10,
            "ASM loc_FBDA should move the struck lift block 10px downward before `Hibernate_UntilAnimFinished` starts",
        );
    }

    // --- Process block effects: lift block spawns platform ---

    #[test]
    fn process_block_effects_lift_spawns_platform() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.lift_block_hit = Some((48, 80));

        rt.process_block_effects(&mut slice);

        assert!(
            rt.played_sound_effects.is_empty(),
            "ASM sub_FB3E bobs/animates the struck block before sub_FCA6 queues the elevator sound",
        );
        assert!(
            rt.block_state.lift_platforms.is_empty(),
            "ASM sub_FB3E should not allocate the sub_FCA6 platform worker on the headbutt frame",
        );
        assert_eq!(
            rt.block_bounces.len(),
            1,
            "ASM sub_FB3E should bob the struck lift block before allocating the sub_FCA6 platform worker",
        );
        assert_eq!(rt.block_bounces[0].cell_x, 48 / 16);
        assert_eq!(rt.block_bounces[0].cell_y, 80 / 16);
        assert_eq!(
            rt.block_bounces[0].remaining,
            NativeRuntime::lift_block_bob_deltas_sub_FB3E().len() as u8,
        );

        for _ in 0..NativeRuntime::lift_block_bob_deltas_sub_FB3E().len() {
            rt.step_visual_effects();
        }

        assert_eq!(
            rt.played_sound_effects,
            vec![QueuedSoundEffect::ElevatorBlock],
            "ASM sub_FCA6 (kid.asm:21021) queues sfx_Elevator_block=$27 only after sub_FB3E hands off to the platform worker",
        );
        assert_eq!(rt.block_state.lift_platforms.len(), 1);
        assert_eq!(rt.block_state.lift_platforms[0].x_pos, 48 << 16);
        assert_eq!(rt.block_state.lift_platforms[0].y_pos, 80 << 16);
        assert_eq!(
            rt.block_state.lift_platforms[0].y_velocity, -0x0001_0000,
            "ASM sub_FB3E (kid.asm:20999): lift platform worker seeds y_vel with `$FFFF0000` before sub_FCA6 steps it",
        );
        assert_eq!(rt.block_state.lift_platforms[0].source_col, 48 / 16);
        assert_eq!(rt.block_state.lift_platforms[0].source_row, 80 / 16);
        assert_eq!(rt.block_state.lift_platforms[0].age, 0);
        assert_eq!(rt.block_state.lift_platforms[0].last_integer_y, 80);
        assert_eq!(rt.block_state.lift_platforms[0].moved_y_steps, 0);
        assert!(!rt.block_state.lift_platforms[0].block_erased);
    }

    #[test]
    fn process_block_effects_shifting_headbutt_reserves_then_commits_loc_11364() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_grid = Some(grid_with_cells(8, 8, &[(3, 5, 0xE7, 0x00)]));
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.shifting_block_hit = Some((3, 5));

        rt.process_block_effects(&mut slice);

        let grid = rt
            .block_grid
            .as_ref()
            .expect("grid should remain installed");
        assert_eq!(
            rt.played_sound_effects,
            vec![QueuedSoundEffect::ShiftingBlock],
            "ASM loc_11364 should play sfx_Shifting_block before the row/above-cell worker gate",
        );
        assert_eq!(
            grid.cell_at(3, 4),
            terrain::Cell::from_be_bytes(0x6F, 0x00),
            "ASM loc_11364 should reserve the above cell by writing `$6F` before sub_113C0 rises",
        );
        assert_eq!(
            grid.cell_at(3, 5),
            terrain::Cell::from_be_bytes(0xE7, 0x00),
            "ASM sub_113C0 should leave the source shifting block solid during the rise worker",
        );
        assert_eq!(rt.block_bounces.len(), 1);

        for _ in 0..8 {
            rt.step_visual_effects();
        }

        let grid = rt
            .block_grid
            .as_ref()
            .expect("grid should remain installed");
        assert_eq!(
            grid.cell_at(3, 4),
            terrain::Cell::from_be_bytes(0xE7, 0x10),
            "ASM sub_113C0 should commit `$E710` to the reserved upper cell after the eight loc_113F4 hibernates",
        );
        assert!(
            !grid.cell_at(3, 5).has_block,
            "ASM sub_113C0 should erase the original lower source block after committing the upper cell",
        );
        assert_eq!(
            rt.block_bounces.len(),
            1,
            "ASM loc_1141E keeps the shifting-block sprite object alive after the layout commit",
        );
        assert_eq!(rt.block_bounces[0].kind, BlockBounceKind::ShiftingFall);
        assert_eq!(rt.block_bounces[0].remaining, 16);
        assert_eq!(
            rt.block_bounces[0].y_offset_px, 16,
            "ASM sub_113C0 should add `$10` to y_pos before the loc_1141E fall tail",
        );

        rt.step_visual_effects();

        assert_eq!(rt.block_bounces[0].remaining, 15);
        assert_eq!(
            rt.block_bounces[0].y_offset_px, 15,
            "ASM loc_1141E should subtract one y pixel on each hibernating fall-tail frame",
        );

        for _ in 0..15 {
            rt.step_visual_effects();
        }

        assert!(
            rt.block_bounces.is_empty(),
            "ASM loc_1141E should jump to Delete_CurrentObject after sixteen 1px fall-tail frames",
        );
    }

    #[test]
    fn shifting_block_activation_loc_11364_skips_the_worker_on_the_top_row() {
        let activation = NativeRuntime::shifting_block_activation_loc_11364(
            0,
            terrain::Cell::from_be_bytes(0x00, 0x07),
        );

        assert!(
            activation.should_play_sound,
            "ASM loc_11364 should preserve `d0` and play `sfx_Shifting_block` even when `tst.w d2` aborts the worker path on row zero",
        );
        assert!(
            !activation.should_spawn_worker,
            "ASM loc_11364 should branch straight to `loc_113BA` when `d2 == 0`, so a top-row shifting block never allocates the `sub_113C0` worker",
        );
        assert_eq!(activation.reserved_above_cell, None);
    }

    #[test]
    fn loc_113ba_only_allows_the_worker_when_the_above_cell_is_clear_and_in_bounds() {
        assert!(
            !NativeRuntime::shifting_block_worker_gate_loc_113BA(0, false),
            "ASM loc_113BA should return immediately when `tst.w d2` sees the top row",
        );
        assert!(
            !NativeRuntime::shifting_block_worker_gate_loc_113BA(5, true),
            "ASM loc_113BA should return immediately when `btst #6,(a2)` reports the above cell occupied",
        );
        assert!(
            NativeRuntime::shifting_block_worker_gate_loc_113BA(5, false),
            "ASM loc_113BA should let the caller continue into `sub_113C0` only for non-zero rows with bit 6 clear above",
        );
    }

    #[test]
    fn shifting_block_activation_loc_11364_only_reserves_clear_cells_above_the_source_block() {
        let blocked = NativeRuntime::shifting_block_activation_loc_11364(
            5,
            terrain::Cell::from_be_bytes(0x60, 0x09),
        );
        assert!(
            !blocked.should_spawn_worker,
            "ASM loc_11364 should branch to `loc_113BA` when `btst #6,(a2)` finds the cell above occupied",
        );
        assert_eq!(blocked.reserved_above_cell, None);

        let clear = NativeRuntime::shifting_block_activation_loc_11364(
            5,
            terrain::Cell::from_be_bytes(0x00, 0x09),
        );
        assert!(
            clear.should_spawn_worker,
            "ASM loc_11364 should continue into the `sub_113C0` setup once the source row is non-zero and the cell above has bit 6 clear",
        );
        assert_eq!(
            clear.reserved_above_cell,
            Some(terrain::Cell::from_be_bytes(0x6F, 0x09)),
            "ASM loc_11364 should reserve the above cell by writing only high byte `$6F`, preserving that cell's original skin byte",
        );
    }

    /// ASM loc_BBD2 (kid.asm:15130): off_BB80 index 7 (Shifting) gate inside
    /// sub_BB54.  `tst.b ($FFFFFA6A); bne return_BBE0` — when $FA6A (on_board)
    /// is set, the shifting block handler is a no-op; when $FA6A is clear, it
    /// falls through to `jmp (j_loc_11364).l` which plays the shifting sound and
    /// runs the worker.  This test verifies the $FA6A-clear path: loc_BBD2
    /// reaches loc_11364 which produces `should_play_sound=true`.
    #[test]
    fn loc_bbd2_shifting_block_gate_fa6a_clear_reaches_loc_11364() {
        // When $FA6A is clear (the loc_BBD2 gate passes), sub_BB54 calls
        // j_loc_11364.  Native: shifting_block_activation_loc_11364 is the
        // loc_11364 entry point.  A non-zero source row with a clear cell
        // above should produce sound + worker.
        let activation = NativeRuntime::shifting_block_activation_loc_11364(
            3,
            terrain::Cell::from_be_bytes(0x00, 0x00),
        );

        assert!(
            activation.should_play_sound,
            "ASM loc_BBD2: $FA6A clear → j_loc_11364 plays sfx_Shifting_block"
        );
        assert!(
            activation.should_spawn_worker,
            "ASM loc_BBD2: $FA6A clear + non-zero row + clear cell above → sub_113C0 worker spawns"
        );
    }

    #[test]
    fn lift_platform_vertical_step_sub_fd00_adds_signed_velocity_to_fp16_y() {
        let next_y = NativeRuntime::lift_platform_vertical_step_sub_FD00(0x0080_0000, -0x0000_0708);

        assert_eq!(
            next_y, 0x007F_F8F8,
            "ASM sub_FD00 should add `$E(a3)` directly into `6(a3)`, preserving the fractional carry/borrow",
        );
    }

    #[test]
    fn lift_block_rise_step_loc_113f4_moves_up_2px_for_eight_hibernating_iterations() {
        let mut y_pos = 80u16;
        let mut loop_counter = 7u16;
        let mut steps = Vec::new();

        loop {
            let step = NativeRuntime::lift_block_rise_step_loc_113F4(y_pos, loop_counter);
            steps.push(step);
            y_pos = step.next_y_pos;
            loop_counter = step.next_loop_counter;
            if !step.should_repeat_loop {
                break;
            }
        }

        assert_eq!(steps.len(), 8);
        assert!(
            steps.iter().all(|step| step.should_hibernate_one_frame),
            "ASM loc_113F4 should call `j_Hibernate_Object_1Frame` on every rise iteration before the `dbf d0,loc_113F4` back-edge",
        );
        assert_eq!(
            steps[0],
            LiftBlockRiseStepLoc113F4 {
                next_y_pos: 78,
                next_loop_counter: 6,
                should_hibernate_one_frame: true,
                should_repeat_loop: true,
            },
        );
        assert_eq!(
            steps.last().copied(),
            Some(LiftBlockRiseStepLoc113F4 {
                next_y_pos: 64,
                next_loop_counter: u16::MAX,
                should_hibernate_one_frame: true,
                should_repeat_loop: false,
            }),
            "ASM loc_113F4 starts from `move.w #7,d0`, so the final `dbf` decrements to `-1` only after eight 2px upward moves",
        );
    }

    #[test]
    fn lift_block_fall_step_loc_1141e_descends_1px_for_sixteen_frames_then_deletes() {
        let mut y_pos = 80u16;
        let mut loop_counter = 0x0Fu16;
        let mut steps = Vec::new();

        loop {
            let step = NativeRuntime::lift_block_fall_step_loc_1141E(y_pos, loop_counter);
            steps.push(step);
            y_pos = step.next_y_pos;
            loop_counter = step.next_loop_counter;
            if !step.should_repeat_loop {
                break;
            }
        }

        assert_eq!(steps.len(), 16);
        assert!(
            steps.iter().all(|step| step.should_hibernate_one_frame),
            "ASM loc_1141E should call `j_Hibernate_Object_1Frame` before each `dbf d0,loc_1141E` check during the descent",
        );
        assert_eq!(
            steps[0],
            LiftBlockFallStepLoc1141E {
                next_y_pos: 79,
                next_loop_counter: 0x0E,
                should_hibernate_one_frame: true,
                should_delete_current_object: false,
                should_repeat_loop: true,
            },
        );
        assert_eq!(
            steps.last().copied(),
            Some(LiftBlockFallStepLoc1141E {
                next_y_pos: 64,
                next_loop_counter: u16::MAX,
                should_hibernate_one_frame: true,
                should_delete_current_object: true,
                should_repeat_loop: false,
            }),
            "ASM loc_1141E starts from `move.w #$F,d0`, so the worker only reaches `Delete_CurrentObject` after sixteen 1px descent steps",
        );
    }

    #[test]
    fn shooter_left_support_probe_loc_11448_skips_column_zero_and_only_targets_empty_left_cells() {
        let left_edge = NativeRuntime::shooter_left_support_probe_loc_11448(0, true);
        assert_eq!(
            left_edge,
            ShooterLeftSupportProbeLoc11448 {
                should_probe_support: false,
                support_col: None,
                next_block_col_for_right_probe: 1,
            },
            "ASM loc_11448 does `subq.w #1,d1; bmi.s loc_11458`, so the left neighbour probe is skipped entirely for the leftmost shooter column",
        );

        let occupied_left = NativeRuntime::shooter_left_support_probe_loc_11448(5, false);
        assert_eq!(
            occupied_left,
            ShooterLeftSupportProbeLoc11448 {
                should_probe_support: false,
                support_col: None,
                next_block_col_for_right_probe: 6,
            },
            "ASM loc_11448 should skip `sub_1149C` when `tst.w -2(a1)` sees a non-empty left support cell",
        );

        let empty_left = NativeRuntime::shooter_left_support_probe_loc_11448(5, true);
        assert_eq!(
            empty_left,
            ShooterLeftSupportProbeLoc11448 {
                should_probe_support: true,
                support_col: Some(4),
                next_block_col_for_right_probe: 6,
            },
            "ASM loc_11448 should call `sub_1149C` for the left support cell, then restore the shared `d1` state so `loc_11458` can continue with the right-hand probe",
        );
    }

    /// ASM loc_1002E (kid.asm:21317) plays `sfx_Block_hit` and spawns the
    /// sub_1007A wobble worker but never calls `Block_LoadSmashBits`; only
    /// type-0 smash blocks emit destroy fragments. The prize-block path
    /// must therefore only bump `blocks_hit_this_frame` — pushing a
    /// BlockDestroyEffect here drew debris over an intact wobbling block.
    #[test]
    fn process_block_effects_prize_does_not_spawn_destroy_fragments() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.prize_block_hit = Some((terrain::PrizeContent::Diamond, 48, 32));

        rt.process_block_effects(&mut slice);

        assert!(
            rt.block_destroy_effects.is_empty(),
            "ASM loc_1002E does not call Block_LoadSmashBits; prize-block hits must not emit debris"
        );
        assert_eq!(rt.blocks_hit_this_frame, 1);
    }

    // --- Floating item: diamond uses inclusive overlap, helmet uses exclusive ---

    #[test]
    fn floating_diamond_inclusive_overlap_collects_at_boundary() {
        // Diamond uses inclusive overlap (<=/>= for edges).
        // Place item exactly at edge of player bounding box.
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });

        let mut slice = OpeningSliceState::new(default_bounds());
        // ASM PrizeDiamond_Init loc_10396 uses CollisionSize_Index unk_31D5E:
        // item_left = 100-5 = 95, item_right = 100+5 = 105.
        // Player half_width = 5, so player_left = x-5, player_right = x+5
        // For inclusive overlap: player_left <= item_right -> x-5 <= 105 -> x <= 110
        // AND player_right >= item_left -> x+5 >= 95 -> x >= 90.
        // Test exactly at boundary: player_right == item_left (x+5 = 95 -> x = 90).
        slice.player.x_pos = 90 << 16;
        // Diamond item_top = 100 - 10 = 90, item_bottom = 100.
        // Inclusive: player_top <= item_bottom and player_y >= item_top.
        slice.player.y_pos = 100 << 16;

        rt.collect_floating_items(&mut slice);

        assert!(
            rt.floating_items[0].collected,
            "diamond should collect with inclusive boundary overlap"
        );
    }

    #[test]
    fn floating_helmet_exclusive_overlap_misses_at_exact_boundary() {
        // Helmet uses exclusive overlap (< / > for edges).
        // At exact boundary, it should NOT collect.
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Skycutter,
            collected: false,
            collision_latched: false,
        });

        let mut slice = OpeningSliceState::new(default_bounds());
        // Helmet box: item_left = 100-8 = 92, item_right = 100+8 = 108
        // For exclusive: player_right > item_left → x+5 > 92 → x > 87
        // At x=87: player_right = 92 = item_left → NOT > → miss
        slice.player.x_pos = 87 << 16;
        // Set y to definitely overlap vertically
        slice.player.y_pos = 110 << 16;

        rt.collect_floating_items(&mut slice);

        assert!(
            !rt.floating_items[0].collected,
            "helmet should NOT collect at exact exclusive boundary"
        );
    }

    // --- Floating item: velocity accumulation and row tracking ---

    #[test]
    fn floating_item_accumulates_velocity_each_frame() {
        // ASM: +$1388/frame downward velocity
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: 100 << 16,
            y_pos: (6 * 16) << 16,
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 6,
            level_row: 6,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();
        assert_eq!(
            rt.floating_items[0].y_velocity, 0x1388,
            "first frame: velocity = $1388"
        );
        let y_after_1 = rt.floating_items[0].y_pos;

        rt.step_visual_effects();
        assert_eq!(
            rt.floating_items[0].y_velocity,
            0x1388 * 2,
            "second frame: velocity = $2710"
        );
        assert!(
            rt.floating_items[0].y_pos > y_after_1,
            "item should fall further on second frame"
        );
    }

    #[test]
    fn floating_item_loc_103b2_uses_the_grace_probe_row_before_stopping() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_blocks();
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0xE0, 0x01));
        grid.set_cell_at(2, 3, terrain::Cell::from_be_bytes(0x00, 0x00));
        grid.set_cell_at(3, 2, terrain::Cell::from_be_bytes(0xE0, 0x01));
        grid.set_cell_at(3, 3, terrain::Cell::from_be_bytes(0xE0, 0x01));
        rt.set_block_grid(grid);
        rt.floating_items.push(FloatingItem {
            x_pos: (2 * 16 + 8) << 16,
            y_pos: i64::from(32 << 16),
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 2,
            level_row: 2,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });
        rt.floating_items.push(FloatingItem {
            x_pos: (3 * 16 + 8) << 16,
            y_pos: i64::from(32 << 16),
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 3,
            level_row: 2,
            first_row_grace: true,
            content: terrain::PrizeContent::Clock,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items[0].y_velocity, 0x1388,
            "ASM loc_103B2 keeps the item falling when the current row is blocked but the one-row grace probe below is still open",
        );
        assert_eq!(
            rt.floating_items[1].y_velocity, 0,
            "ASM loc_103B2 stops immediately once both the current row and the grace-probed row below are solid",
        );
    }

    #[test]
    fn floating_item_loc_103da_zeroes_velocity_when_the_fall_branch_is_blocked() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_blocks();
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0xE0, 0x01));
        grid.set_cell_at(2, 3, terrain::Cell::from_be_bytes(0xE0, 0x01));
        rt.set_block_grid(grid);
        let blocked_y = i64::from(32 << 16);
        rt.floating_items.push(FloatingItem {
            x_pos: (2 * 16 + 8) << 16,
            y_pos: blocked_y,
            y_velocity: 0x2710,
            remaining: 500,
            spawn_delay: 0,
            level_col: 2,
            level_row: 2,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items[0].y_velocity, 0,
            "ASM loc_103DA should replace the blocked fall delta with zero before the shared tail continues",
        );
        assert_eq!(
            rt.floating_items[0].y_pos, blocked_y,
            "ASM loc_103DA should leave the floating item on the same pixel row when the blocked branch skips `loc_103DE`",
        );
    }

    #[test]
    fn floating_item_loc_10402_updates_the_row_baseline_once_per_16px_boundary_crossing() {
        let mut rt = NativeRuntime::new(manifest());
        let mut grid = grid_with_blocks();
        grid.set_cell_at(2, 2, terrain::Cell::from_be_bytes(0, 0));
        grid.set_cell_at(2, 3, terrain::Cell::from_be_bytes(0, 0));
        rt.set_block_grid(grid);
        rt.floating_items.push(FloatingItem {
            x_pos: (2 * 16 + 8) << 16,
            y_pos: i64::from(47 << 16),
            y_velocity: 0xF000,
            remaining: 500,
            spawn_delay: 0,
            level_col: 2,
            level_row: 2,
            first_row_grace: true,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items[0].level_row, 3,
            "ASM loc_10402 should advance the tracked level row once the falling item crosses into the next snapped 16px band",
        );
        assert!(
            !rt.floating_items[0].first_row_grace,
            "ASM loc_10402 should clear the one-row grace bit once the new aligned row is copied into the running baseline",
        );

        let y_after_crossing = rt.floating_items[0].y_pos;
        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items[0].level_row, 3,
            "ASM loc_10402 should keep the updated row baseline in `d0`, so later frames inside the same 16px band do not advance `level_row` again",
        );
        assert!(
            rt.floating_items[0].y_pos > y_after_crossing,
            "the follow-up frame should still fall inside the same open row without spuriously re-advancing the tracked row",
        );
    }

    #[test]
    fn floating_item_loc_10430_deletes_items_once_they_reach_level_height() {
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (2 * 16 + 8) << 16,
            y_pos: i64::from(895 << 16),
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 2,
            level_row: 55,
            first_row_grace: false,
            content: terrain::PrizeContent::Diamond,
            collected: false,
            collision_latched: false,
        });
        rt.floating_items.push(FloatingItem {
            x_pos: (3 * 16 + 8) << 16,
            y_pos: i64::from(896 << 16),
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 3,
            level_row: 56,
            first_row_grace: false,
            content: terrain::PrizeContent::Clock,
            collected: false,
            collision_latched: false,
        });

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items.len(),
            1,
            "ASM loc_10430 should jump straight to `Delete_CurrentObject` once `y_pos(a3)` reaches `Level_height_pixels`",
        );
        assert_eq!(
            rt.floating_items[0].level_row, 55,
            "ASM loc_10430 should keep the still-in-bounds item alive while its integer Y word remains strictly below `Level_height_pixels`",
        );
    }

    // --- Clock-prize loop start/stop (#342) ---

    fn clock_floating_item() -> FloatingItem {
        FloatingItem {
            x_pos: (3 * 16 + 8) << 16,
            y_pos: i64::from(100 << 16),
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 3,
            level_row: 6,
            first_row_grace: false,
            content: terrain::PrizeContent::Clock,
            collected: false,
            collision_latched: false,
        }
    }

    #[test]
    fn loc_10796_clock_prize_countdown_records_animation_swap_event() {
        // ASM loc_10796 (kid.asm:22006-22014): after the hibernated countdown
        // reaches `$012C`, the clock prize swaps to `stru_10C1E`.
        let mut rt = NativeRuntime::new(manifest());
        let mut clock = clock_floating_item();
        clock.remaining = 0x012D;
        rt.floating_items.push(clock);

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items[0].remaining, 0x012C,
            "ASM loc_10796 decrements d3 before testing the animation-swap threshold",
        );
        assert_eq!(
            rt.trace_frame().clock_prize_animation_swaps_loc_10796,
            vec![(3 * 16 + 8, 100)],
            "ASM loc_10796 exposes the live clock-prize animation swap edge to the trace",
        );
    }

    #[test]
    fn loc_1080a_continue_prize_countdown_records_animation_swap_event() {
        // ASM loc_1080A (kid.asm:22045-22053): after the hibernated countdown
        // reaches `$012C`, the continue prize swaps to `stru_10C2E`.
        let mut rt = NativeRuntime::new(manifest());
        let mut item = clock_floating_item();
        item.content = terrain::PrizeContent::Continue;
        item.remaining = 0x012D;
        rt.floating_items.push(item);

        rt.step_visual_effects();

        assert_eq!(
            rt.floating_items[0].remaining, 0x012C,
            "ASM loc_1080A decrements d3 before testing the animation-swap threshold",
        );
        assert_eq!(
            rt.trace_frame().continue_prize_animation_swaps_loc_1080a,
            vec![(3 * 16 + 8, 100)],
            "ASM loc_1080A exposes the live continue-prize animation swap edge to the trace",
        );
    }

    #[test]
    fn loc_10838_continue_prize_delete_emits_raw_sequence_6a() {
        // ASM loc_10838 (kid.asm:22062-22067): every Continue prize delete
        // exit preserves `d0`, plays raw sequence `$6A`, then deletes the
        // current object. This exercises the live retain/removal path.
        let mut rt = NativeRuntime::new(manifest());
        let mut item = clock_floating_item();
        item.content = terrain::PrizeContent::Continue;
        item.remaining = 1;
        rt.floating_items.push(item);

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_1080A -> loc_10838: d3 reaching zero deletes the Continue prize object",
        );
        assert_eq!(
            rt.played_sound_commands,
            vec![QueuedSoundCommand::StartSequence(
                QueuedSoundEffect::ContinuePrizeDelete
            )],
            "ASM loc_10838: Continue cleanup must issue j_PlaySound with raw id $6A",
        );
        assert_eq!(QueuedSoundEffect::ContinuePrizeDelete.sound_id(), 0x6A);
    }

    #[test]
    fn loc_10792_first_clock_prize_spawn_emits_startsequence_clock_prize() {
        // ASM PrizeClock_Init loc_10792 (kid.asm:21996-22005): the first active
        // clock plays the looping sfx_Clock_prize; the start fires on the
        // `0 -> first-clock` spawn edge. #342: native used to start it on
        // collection (apply_prize_content) with no matching stop.
        let mut rt = NativeRuntime::new(manifest());
        let before = rt.active_clock_prize_count();
        assert_eq!(before, 0, "no clock is falling yet");
        rt.floating_items.push(clock_floating_item());
        rt.start_clock_prize_sound_on_first_spawn_loc_10792(before);
        assert_eq!(
            rt.played_sound_commands,
            vec![QueuedSoundCommand::StartSequence(
                QueuedSoundEffect::ClockPrize
            )],
            "ASM loc_10792: the first falling clock starts sfx_Clock_prize via StartSequence ($10 cmdstartseq)",
        );
    }

    #[test]
    fn loc_10792_clock_prize_start_holds_while_another_clock_is_active() {
        // ASM loc_10792 `tst.b ($FFFFFB55)`: later clocks only `addq.b #1` the
        // refcount and skip the j_PlaySound start because it was already nonzero.
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(clock_floating_item());
        let before = rt.active_clock_prize_count();
        assert_eq!(before, 1, "one clock is already falling");
        rt.floating_items.push(clock_floating_item());
        rt.start_clock_prize_sound_on_first_spawn_loc_10792(before);
        assert!(
            rt.played_sound_commands.is_empty(),
            "ASM loc_10792: a second concurrent clock does not restart sfx_Clock_prize",
        );
    }

    #[test]
    fn loc_107e0_last_clock_prize_exit_emits_stopsequence_clock_prize() {
        // ASM loc_107E0 (kid.asm:22028-22033): once the last falling clock
        // object is deleted (here it reached Level_height_pixels), the
        // decremented $FFFFFB55 hits zero and sfx_Clock_prize is stopped via
        // j_PlaySound2 ($12 cmdstopseq). #342: this stop was unported, so the
        // looping clock sequence ran forever. Exercises the live wiring in
        // step_visual_effects, not just the helper.
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(FloatingItem {
            x_pos: (3 * 16 + 8) << 16,
            y_pos: i64::from(896 << 16),
            y_velocity: 0,
            remaining: 500,
            spawn_delay: 0,
            level_col: 3,
            level_row: 56,
            first_row_grace: false,
            content: terrain::PrizeContent::Clock,
            collected: false,
            collision_latched: false,
        });
        assert_eq!(rt.active_clock_prize_count(), 1, "one clock is falling");

        rt.step_visual_effects();

        assert!(
            rt.floating_items.is_empty(),
            "ASM loc_10430/loc_107B0: the clock object is deleted once y_pos reaches Level_height_pixels",
        );
        assert_eq!(
            rt.played_sound_commands,
            vec![QueuedSoundCommand::StopSequence(
                QueuedSoundEffect::ClockPrize
            )],
            "ASM loc_107E0: deleting the last falling clock must stop sfx_Clock_prize via StopSequence ($12 cmdstopseq)",
        );
    }

    #[test]
    fn loc_107e0_clock_prize_stop_holds_while_another_clock_remains() {
        // ASM loc_107E0 `subq.b #1,$FFFFFB55; bne.s loc_107EE`: a surviving
        // clock keeps the refcount nonzero, so the j_PlaySound2 stop is skipped.
        let mut rt = NativeRuntime::new(manifest());
        rt.floating_items.push(clock_floating_item());
        rt.floating_items.push(clock_floating_item());
        let before = rt.active_clock_prize_count();
        assert_eq!(before, 2, "two clocks are falling");
        rt.floating_items.pop();
        rt.stop_clock_prize_sound_on_last_exit_loc_107e0(before);
        assert!(
            rt.played_sound_commands.is_empty(),
            "ASM loc_107E0: refcount 2->1 stays nonzero, so no StopSequence fires",
        );
        assert_eq!(
            rt.active_clock_prize_count(),
            1,
            "the surviving clock keeps the refcount at one",
        );
    }

    #[test]
    fn loc_107e0_clock_prize_stop_silent_when_no_clock_was_active() {
        // No falling clock last tick => nothing to decrement, so loc_107E0
        // never reaches the j_PlaySound2 stop.
        let mut rt = NativeRuntime::new(manifest());
        rt.stop_clock_prize_sound_on_last_exit_loc_107e0(0);
        assert!(
            rt.played_sound_commands.is_empty(),
            "loc_107E0 only stops the loop when a previously-active clock reaches refcount zero",
        );
    }

    #[test]
    fn floating_item_step_returns_past_the_unreachable_loc_10434_padding_loop() {
        let mut rt = NativeRuntime::new(manifest());
        rt.step_visual_effects();

        assert!(rt.floating_items.is_empty());
        assert!(
            rt.block_bounces.is_empty() && rt.block_destroy_effects.is_empty(),
            "ASM loc_10434 is only the unreachable self-loop padding after `OneTimePrize_CheckCollected`, so native `step_visual_effects` should return to the caller instead of emulating that trap",
        );
    }

    #[test]
    fn projectile_hits_player_exact_overlap() {
        // Projectile at same position as player — must hit.
        // Kills: return false replacement, all >> with << mutations (lines 136-137, 144-145).
        let mut rt = runtime_with_projectile_and_player(100, 100, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            rt.projectile_hits_player(&slice),
            "projectile at player position must hit"
        );
        assert!(
            rt.projectiles.is_empty(),
            "hitting projectile should be removed"
        );
    }

    #[test]
    fn projectile_top_contact_bounces_without_damage_loc_b640_jump_on_enemy() {
        let mut rt = runtime_with_projectile_and_player(100, 116, 100, 100);
        let hitpoints_before = rt.progression.number_hitpoints;
        let mut slice = rt.opening_slice.unwrap();
        slice.player.on_ground = false;
        slice.player.y_velocity = 0x0001_0000;
        rt.opening_slice = Some(slice);

        let mut slice = rt.opening_slice.unwrap();
        let consumed = rt.resolve_frame_hazards(
            &mut slice,
            FrameInput::default(),
            0x0001_0000,
            100 << 16,
            100 << 16,
            100 << 16,
        );

        assert!(
            consumed,
            "ASM off_B640 routes hostile projectile colid `$2C` through Jump_On_Enemy"
        );
        assert_eq!(
            rt.progression.number_hitpoints, hitpoints_before,
            "ASM Jump_On_Enemy only falls through to Check_for_recent_damge when object bit 14 is set"
        );
        assert!(
            rt.projectiles.is_empty(),
            "ASM sub_1D84 still consumes the projectile collision object before dispatching colid `$2C`"
        );
        assert!(
            rt.pending_stomp_bounce.is_some(),
            "ASM Jump_On_Enemy queues the loc_B6DA bounce mechanics"
        );
        assert_eq!(
            rt.last_kid_frame_post_motion_mutator,
            KidFramePostMotionMutator::StompBounce,
        );
    }

    #[test]
    fn shooter_object_overlap_does_not_enter_kid_collision_loc_edfa_issue_277() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::ShooterBlock,
            x_pos: 99 << 16,
            y_pos: 114 << 16,
            y_velocity: -(4 << 16),
            lifetime: 117,
            ..EnemyProjectile::default()
        });
        let hitpoints_before = rt.progression.number_hitpoints;
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 101 << 16;
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        slice.player.x_velocity = -0x0002_9900;
        slice.player.facing = Direction::Left;
        let post_snap_x_pos = slice.player.x_pos;
        rt.opening_slice = Some(slice);

        let consumed = rt.resolve_frame_hazards(
            &mut slice,
            FrameInput {
                left: true,
                ..FrameInput::default()
            },
            0x0007_0000,
            100 << 16,
            post_snap_x_pos,
            100 << 16,
        );

        assert!(
            !consumed,
            "ASM Process_ShooterObjects loc_EDFA..loc_EF62 moves ShooterBlock slots against level layout and compiles sprites; it does not call Character_CheckCollision/off_B640"
        );
        assert_eq!(
            rt.progression.number_hitpoints, hitpoints_before,
            "the issue-277 left-stair shard contact must not be recast as generic projectile damage"
        );
        assert!(
            rt.pending_stomp_bounce.is_none(),
            "the same contact must not synthesize off_B640 `$2C`/Jump_On_Enemy"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "ROM shooter objects survive Kid overlap until their layout/lifetime path deletes them"
        );
        assert!(
            !rt.kid_touched_enemy_this_frame_loc_fa75,
            "loc_1C18's projectile-list contact latch should not see shooter-object sprite slots"
        );
    }

    #[test]
    fn shooter_object_body_overlap_damages_without_stomp_loc_edfa_issue_277() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::ShooterBlock,
            x_pos: 101 << 16,
            y_pos: 98 << 16,
            y_velocity: -(4 << 16),
            lifetime: 115,
            ..EnemyProjectile::default()
        });
        let hitpoints_before = rt.progression.number_hitpoints;
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 101 << 16;
        slice.player.on_ground = true;
        slice.player.y_velocity = 0;
        slice.player.x_velocity = -0x0002_9900;
        slice.player.facing = Direction::Left;
        rt.opening_slice = Some(slice);

        let consumed = rt.resolve_frame_hazards(
            &mut slice,
            FrameInput {
                left: true,
                ..FrameInput::default()
            },
            0x0007_0000,
            100 << 16,
            100 << 16,
            100 << 16,
        );

        assert!(
            consumed,
            "issue-277 oracle frame 90 takes damage once the upward ShooterBlock shard reaches Kid's body"
        );
        assert_eq!(
            rt.progression.number_hitpoints,
            hitpoints_before - 1,
            "ShooterBlock body contact should use Check_for_recent_damge semantics, not Jump_On_Enemy"
        );
        assert!(
            rt.pending_stomp_bounce.is_none(),
            "ASM Process_ShooterObjects loc_EDFA..loc_EF62 does not synthesize off_B640 `$2C`/Jump_On_Enemy"
        );
        assert!(
            !rt.projectiles.is_empty(),
            "ASM Process_ShooterObjects keeps the shooter-object slot alive after Kid damage"
        );
        let slice_after = rt.opening_slice.expect("damage should store slice");
        assert_eq!(
            slice_after.player.y_pos,
            0x0007_4000 + (100 << 16),
            "issue-277 ShooterBlock damage should preserve the airborne post-gravity Y instead of the native same-frame floor snap"
        );
        assert_eq!(slice_after.player.y_velocity, 0x0007_4000);
        assert_eq!(
            slice_after.player.x_pos,
            (100 << 16) - 0x0002_9800,
            "issue-277 ShooterBlock damage should also preserve the cancelled jump-air X step"
        );
        assert_eq!(slice_after.player.x_velocity, -0x0002_9800);
        assert!(
            !slice_after.player.on_ground,
            "ShooterBlock damage before terrain snap should leave Kid airborne for this frame"
        );
    }

    #[test]
    fn shooter_object_point_collision_uses_strict_kid_hitbox_edges_sub_219c() {
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 101 << 16;

        let mut edge_rt = NativeRuntime::new(manifest());
        edge_rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::ShooterBlock,
            x_pos: 105 << 16,
            y_pos: 98 << 16,
            ..EnemyProjectile::default()
        });
        assert!(
            !edge_rt.projectile_hits_player(&slice),
            "ASM sub_219C rejects shooter-slot points on Kid_hitbox_right with `bge`"
        );

        let mut inside_rt = NativeRuntime::new(manifest());
        inside_rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::ShooterBlock,
            x_pos: 104 << 16,
            y_pos: 98 << 16,
            ..EnemyProjectile::default()
        });
        assert!(
            inside_rt.projectile_hits_player(&slice),
            "ASM sub_219C accepts the same shooter-slot point one pixel inside Kid_hitbox_right"
        );
    }

    #[test]
    fn shooter_damage_late_sub_1f52_lands_next_frame_issue_277() {
        let mut rt = NativeRuntime::new(manifest());
        rt.set_block_grid(grid_with_cells(120, 24, &[(97, 19, 0xE8, 0x01)]));
        rt.block_state.shooter_blocks = vec![ShooterBlockEntry {
            col: 97,
            row: 19,
            directions: 0x01,
            supported_by_neighbour: false,
            restored_touch_shooter: false,
            age: 1,
        }];
        start_level(&mut rt);
        rt.progression.invulnerable_frames = 8;
        rt.last_kid_frame_post_motion_mutator = KidFramePostMotionMutator::ProjectileDamage;
        rt.previous_input = FrameInput {
            left: true,
            ..FrameInput::default()
        };

        let mut slice = OpeningSliceState::new(LevelBounds {
            width_pixels: 2048,
            height_pixels: 384,
        });
        slice.player.x_pos = 0x0614_b800;
        slice.player.y_pos = 0x012c_bd00;
        slice.player.x_velocity = -0x0002_9800;
        slice.player.y_velocity = 0x0007_7d00;
        slice.player.facing = Direction::Left;
        slice.player.on_ground = false;
        rt.opening_slice = Some(slice);

        let trace = rt.step_frame(FrameInput {
            left: true,
            ..FrameInput::default()
        });
        let landed = rt
            .opening_slice
            .expect("frame should keep live gameplay slice")
            .player;

        assert_eq!(
            landed.y_pos, 0x012f_0000,
            "ASM Mode_Level calls sub_1F52 after projectile collision; issue-277 oracle frame 91 consumes the late shooter support as a landing handoff at the shooter rectangle bottom"
        );
        assert_eq!(
            landed.x_pos, 0x0614_b800,
            "loc_A94E -> loc_A99E updates the walking handoff without spending an extra horizontal position step on this frame"
        );
        assert_eq!(
            landed.x_velocity, -0x0002_9700,
            "the handoff should still apply the next jump-air leftward velocity sample before entering walking"
        );
        assert_eq!(landed.y_velocity, 0);
        assert!(landed.on_ground);
        assert_eq!(
            trace.character_movement,
            Some(2),
            "the frame should export MoveID_Walking rather than the dropped MoveID_Jump state"
        );
    }

    #[test]
    fn projectile_overlap_latches_fa75_and_blocks_next_frame_fresh_stomp_loc_1c18() {
        let mut rt = runtime_with_projectile_and_player(100, 100, 100, 100);
        rt.progression.invulnerable_frames = 1;
        let hitpoints_before = rt.progression.number_hitpoints;
        let mut slice = rt.opening_slice.unwrap();
        slice.player.on_ground = false;
        slice.player.y_velocity = 0x0001_0000;

        assert!(
            !rt.resolve_frame_hazards(
                &mut slice,
                FrameInput::default(),
                0x0001_0000,
                100 << 16,
                100 << 16,
                100 << 16,
            ),
            "ASM loc_1D00 suppresses invulnerable damage after the projectile-side contact is consumed"
        );
        assert_eq!(rt.progression.number_hitpoints, hitpoints_before);
        assert!(
            rt.projectiles.is_empty(),
            "ASM loc_1E94 still consumes the overlapping projectile during the invulnerability frame"
        );
        assert!(
            rt.kid_touched_enemy_this_frame_loc_fa75,
            "ASM GfxObjects_CollisionKid_Collide loc_1C18 latches FA75 for projectile-list overlaps too"
        );

        rt.progression.invulnerable_frames = 0;
        rt.projectiles.push(make_projectile(100, 116));
        let mut next_slice = slice;

        assert!(
            rt.resolve_frame_hazards(
                &mut next_slice,
                FrameInput::default(),
                0x0001_0000,
                100 << 16,
                100 << 16,
                100 << 16,
            ),
            "next-frame projectile contact still consumes the hazard frame"
        );
        assert_eq!(
            rt.progression.number_hitpoints,
            hitpoints_before - 1,
            "ASM loc_1CE4 sees FA74 from the prior projectile overlap and falls through loc_1D4C to damage instead of a fresh `$2C` stomp"
        );
        assert!(
            rt.pending_stomp_bounce.is_none(),
            "sustained contact must not queue Jump_On_Enemy"
        );
    }

    #[test]
    fn drips_falling_drop_overlap_is_not_player_damage_loc_3345c_issue_260() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            y_velocity: DRIPS_DROP_GRAVITY,
            lifetime: DRIPS_DROP_LIFETIME,
            terrain_kill: true,
            owner: Some(EnemyProjectileOwner {
                segment_index: 0,
                layout_index: 7,
                token: 0,
            }),
            ..Default::default()
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);
        let slice = rt.opening_slice.unwrap();

        assert!(
            !rt.projectile_hits_player(&slice),
            "ASM Drips.asm loc_3345C only advances the falling child until floor collision `$0C`; oracle issue-260 frame 478 keeps Kid HP unchanged while the visible drop overlaps"
        );
        assert_eq!(rt.projectiles.len(), 1);
        assert_eq!(rt.projectiles[0].kind, EnemyProjectileKind::DripsDrop);
        assert!(
            rt.last_projectile_damage_probe.is_none(),
            "falling Drips drops are visual until loc_33488/loc_335D8 turns them into splash/crawler state"
        );
    }

    #[test]
    fn drips_falling_drop_side_contact_damages_before_fall_step_loc_3345c_issue_260() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 100 << 16,
            y_pos: 72 << 16,
            y_velocity: 0x0000_8000,
            lifetime: DRIPS_DROP_LIFETIME,
            terrain_kill: true,
            owner: Some(EnemyProjectileOwner {
                segment_index: 0,
                layout_index: 7,
                token: 0,
            }),
            ..Default::default()
        });
        rt.progression.invulnerable_frames = 0;
        rt.kid_is_invulnerable = false;
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        slice.player.on_ground = false;
        rt.opening_slice = Some(slice);
        let hitpoints_before = rt.progression.number_hitpoints;

        assert!(
            rt.resolve_frame_hazards(
                &mut slice,
                FrameInput::default(),
                0,
                100 << 16,
                100 << 16,
                100 << 16,
            ),
            "ASM Drips.asm loc_3345C ignores Kid collision for the child worker, but GfxObjects_CollisionKid still writes Kid `$38=$28`; oracle issue-260 frame 372 damages Kid before the drop advances"
        );
        assert_eq!(rt.progression.number_hitpoints, hitpoints_before - 1);
        assert_eq!(
            rt.projectiles.len(),
            1,
            "loc_3345C only consumes terrain `$0C`; Kid `$28` damage must not delete the falling child"
        );
        assert_eq!(rt.projectiles[0].kind, EnemyProjectileKind::DripsDrop);
        assert!(
            rt.last_projectile_damage_probe
                .as_deref()
                .is_some_and(|probe| probe.starts_with("hit:DripsDropObject@")),
            "the regression should identify the object-collision path, not generic projectile overlap"
        );
    }

    #[test]
    fn fire_demon_death_trail_hits_player_without_consuming_child_loc_3a6ca() {
        let mut rt = NativeRuntime::new(manifest());
        rt.fire_demon_trails.push(FireDemonTrailChild {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            remaining_frames: 0x96,
            scheduler_delay_frames: 0,
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);
        let slice = rt.opening_slice.unwrap();

        assert!(
            rt.projectile_hits_player(&slice),
            "ASM Fire_Demon loc_3A63C trail still has hostile collision"
        );
        assert_eq!(
            rt.fire_demon_trails.len(),
            1,
            "ASM Fire_Demon loc_3A6CA loops until `$19(a3)` or d3 expiry; Kid collision must not delete the trail child"
        );
    }

    #[test]
    fn fire_demon_death_trail_misses_player_on_separate_axis_loc_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        rt.fire_demon_trails.push(FireDemonTrailChild {
            x_pos: 100 << 16,
            y_pos: 122 << 16,
            remaining_frames: 0x96,
            scheduler_delay_frames: 0,
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);
        let slice = rt.opening_slice.unwrap();

        assert!(
            !rt.projectile_hits_player(&slice),
            "ASM Fire_Demon loc_3A63C trail collision still requires both X and Y overlap; Y at the exclusive boundary misses"
        );
        assert!(rt.last_projectile_damage_probe.is_none());
    }

    #[test]
    fn fire_demon_death_trail_misses_player_at_x_boundary_loc_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        rt.fire_demon_trails.push(FireDemonTrailChild {
            x_pos: 114 << 16,
            y_pos: 100 << 16,
            remaining_frames: 0x96,
            scheduler_delay_frames: 0,
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);
        let slice = rt.opening_slice.unwrap();

        assert!(
            !rt.projectile_hits_player(&slice),
            "ASM Fire_Demon loc_3A63C trail collision uses the generic projectile x threshold; the 14px exclusive boundary misses"
        );
        assert!(rt.last_projectile_damage_probe.is_none());
    }

    #[test]
    fn fire_demon_death_trail_misses_player_far_left_loc_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        rt.fire_demon_trails.push(FireDemonTrailChild {
            x_pos: 53 << 16,
            y_pos: 100 << 16,
            remaining_frames: 0x96,
            scheduler_delay_frames: 0,
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);
        let slice = rt.opening_slice.unwrap();

        assert!(
            !rt.projectile_hits_player(&slice),
            "ASM Fire_Demon loc_3A63C trail collision compares projectile-minus-Kid distance before abs; a trail 47px left is outside the hostile hitbox"
        );
        assert!(rt.last_projectile_damage_probe.is_none());
    }

    #[test]
    fn fire_demon_death_trail_damage_probe_reports_signed_delta_loc_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        rt.fire_demon_trails.push(FireDemonTrailChild {
            x_pos: 87 << 16,
            y_pos: 79 << 16,
            remaining_frames: 0x96,
            scheduler_delay_frames: 0,
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);
        let slice = rt.opening_slice.unwrap();

        assert!(
            rt.projectile_hits_player(&slice),
            "ASM Fire_Demon loc_3A63C trail collision uses the same centred overlap as generic hostile projectile damage"
        );
        assert!(
            rt.last_projectile_damage_probe
                .as_deref()
                .is_some_and(|probe| probe.contains("dx=-13 dy=-21")),
            "fire-demon trail diagnostics must preserve signed projectile-minus-Kid deltas"
        );
    }

    fn supported_fire_demon_trail_grid_loc_3a63c() -> terrain::BlockGrid {
        let mut data = vec![0u8; 5 * 5 * 2];
        for col in 1..=3 {
            data[(2 * 5 + col) * 2] = 0x60;
        }
        terrain::BlockGrid::from_level_layout(&data, 5, 5)
    }

    #[test]
    fn dragon_walking_breath_hits_player_without_consuming_child_loc_3d082() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            lifetime: DRAGON_WALKING_BREATH_LIFETIME,
            friendly: false,
            kind: EnemyProjectileKind::DragonWalkingBreath,
            ..Default::default()
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);
        let slice = rt.opening_slice.unwrap();

        assert!(
            rt.projectile_hits_player(&slice),
            "ASM Dragon.asm loc_3D02A breath is still a hostile hitbox"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM Dragon.asm loc_3D082 keeps polling until parent `$5B` or child `$18`; Kid collision must not delete the breath child"
        );
        assert_eq!(
            rt.projectiles[0].kind,
            EnemyProjectileKind::DragonWalkingBreath
        );
    }

    #[test]
    fn fireball_trail_hits_player_without_consuming_child_loc_33110() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            lifetime: 0x96,
            friendly: false,
            kind: EnemyProjectileKind::FireballTrail,
            ..Default::default()
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);
        let slice = rt.opening_slice.unwrap();

        assert!(
            rt.projectile_hits_player(&slice),
            "ASM Fireball.asm loc_330D0 trail remains a hostile hitbox"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM Fireball.asm loc_33110 deletes on `$19` or d3 expiry; Kid collision must not consume the trail child"
        );
    }

    #[test]
    fn fire_demon_death_spawns_typed_child_not_projectile_carrier_loc_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        rt.active_enemies.push(test_enemy(
            EnemyFamily::FireDemon,
            0x0064_0000,
            0x00C8_0000,
            1,
        ));

        rt.remove_dead_enemy(0);

        assert_eq!(
            rt.active_enemies.len(),
            1,
            "ASM Fire_Demon loc_3A5A6 hibernates through the death animation before the loc_3A63C child allocation"
        );
        assert_eq!(
            rt.active_enemies[0].wait_timer,
            FIRE_DEMON_DEATH_ANIMATION_FRAMES
        );
        assert!(
            rt.fire_demon_trails.is_empty(),
            "ASM Fire_Demon loc_3A63C is reached only after the loc_3A5A6 animation hibernate"
        );

        rt.active_enemies[0].wait_timer = 0;
        rt.remove_dead_enemy(0);

        assert!(
            rt.projectiles.is_empty(),
            "ASM Fire_Demon loc_3A63C allocates a child object, not a generic projectile"
        );
        assert_eq!(rt.fire_demon_trails.len(), 1);
        assert_eq!(rt.fire_demon_trails[0].x_pos, 0x0064_0000);
        assert_eq!(
            rt.fire_demon_trails[0].y_pos, 0x00D0_0000,
            "ASM loc_3A63C: y_pos += $F, then align down with $FFF0"
        );
        assert_eq!(rt.fire_demon_trails[0].remaining_frames, 0x96);
    }

    #[test]
    fn fire_demon_walking_trail_spawns_typed_child_loc_3a542_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_grid = Some(supported_fire_demon_trail_grid_loc_3a63c());
        let mut demon = test_enemy(EnemyFamily::FireDemon, 0x0020_0000, 0x0020_0000, 3);
        demon.on_ground = true;
        rt.active_enemies.push(demon);

        rt.spawn_fire_demon_walking_trails_loc_3a542();

        assert_eq!(rt.fire_demon_trails.len(), 1);
        assert_eq!(rt.fire_demon_trails[0].x_pos, 0x0020_0000);
        assert_eq!(
            rt.fire_demon_trails[0].y_pos, 0x0020_0000,
            "ASM Fire_Demon.asm loc_3A63C applies y += $F then aligns down with $FFF0 for walking fire trails too"
        );
        assert_eq!(rt.fire_demon_trails[0].remaining_frames, 0x96);
        assert_eq!(
            rt.active_enemies[0].grounded_frames, 1,
            "ASM Fire_Demon.asm loc_3A542 advances through the saved walking animation after the allocation gate"
        );
    }

    #[test]
    fn fire_demon_walking_trail_skips_clipped_child_loc_3a542_3a6ca() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_grid = Some(supported_fire_demon_trail_grid_loc_3a63c());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.camera = CameraState { x_pos: 0, y_pos: 0 };
        rt.opening_slice = Some(slice);
        let mut demon = test_enemy(EnemyFamily::FireDemon, 0x0180_0000, 0x0020_0000, 3);
        demon.on_ground = true;
        rt.active_enemies.push(demon);

        rt.spawn_fire_demon_walking_trails_loc_3a542();

        assert!(
            rt.fire_demon_trails.is_empty(),
            "ASM Make_SpriteFromGfxObject loc_11EE leaves `$19(a3)` set beyond x=$160, so Fire_Demon.asm loc_3A6CA deletes the clipped walking-trail child",
        );
    }

    #[test]
    fn fire_demon_refresh_active_roster_spawns_walking_trail_loc_3a542_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        rt.boot_state = BootState::InGame;
        rt.block_grid = Some(supported_fire_demon_trail_grid_loc_3a63c());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 0x0100_0000;
        slice.player.y_pos = 0x0100_0000;
        rt.opening_slice = Some(slice);
        let mut demon = test_enemy(EnemyFamily::FireDemon, 0x0020_0000, 0x0020_0000, 3);
        demon.on_ground = true;
        rt.active_enemies.push(demon);

        rt.refresh_active_roster();

        assert_eq!(
            rt.fire_demon_trails.len(),
            1,
            "ASM Fire_Demon.asm loc_3A542 is part of the live object refresh path, not only a direct helper call"
        );
        assert_eq!(rt.fire_demon_trails[0].remaining_frames, 0x96);
    }

    #[test]
    fn fire_demon_walking_trail_respects_seven_child_cap_loc_3a564() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_grid = Some(supported_fire_demon_trail_grid_loc_3a63c());
        rt.fire_demon_trails = vec![
            FireDemonTrailChild {
                x_pos: 0x0020_0000,
                y_pos: 0x0020_0000,
                remaining_frames: 0x96,
                scheduler_delay_frames: 0,
            };
            7
        ];
        let mut demon = test_enemy(EnemyFamily::FireDemon, 0x0020_0000, 0x0020_0000, 3);
        demon.on_ground = true;
        rt.active_enemies.push(demon);

        rt.spawn_fire_demon_walking_trails_loc_3a542();

        assert_eq!(
            rt.fire_demon_trails.len(),
            7,
            "ASM Fire_Demon.asm loc_3A564 does not allocate when Number_of_Fire_Trails is already 7"
        );
        assert_eq!(
            rt.active_enemies[0].grounded_frames, 1,
            "ASM Fire_Demon.asm loc_3A542 still advances the parent animation when allocation is capped"
        );
    }

    #[test]
    fn fire_demon_walking_trail_skips_unsupported_floor_loc_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(
            &[0u8; 5 * 5 * 2],
            5,
            5,
        ));
        let mut demon = test_enemy(EnemyFamily::FireDemon, 0x0020_0000, 0x0020_0000, 3);
        demon.on_ground = true;
        rt.active_enemies.push(demon);

        rt.spawn_fire_demon_walking_trails_loc_3a542();

        assert!(
            rt.fire_demon_trails.is_empty(),
            "ASM Fire_Demon.asm loc_3A63C falls through to loc_3A6DA before the child is observable when the floor probes do not all mask to `$6000`"
        );
        assert_eq!(rt.active_enemies[0].grounded_frames, 1);
    }

    #[test]
    fn fire_demon_walking_trail_uses_raw_level_two_saved_walk_cadence_loc_3a542() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_grid = Some(supported_fire_demon_trail_grid_loc_3a63c());
        let mut demon = test_enemy(EnemyFamily::FireDemon, 0x0020_0000, 0x0020_0000, 5);
        demon.enemy_level = 2;
        demon.on_ground = true;
        demon.grounded_frames = 18;
        rt.active_enemies.push(demon);

        rt.spawn_fire_demon_walking_trails_loc_3a542();

        assert_eq!(
            rt.fire_demon_trails.len(),
            1,
            "ASM Fire_Demon.asm loc_3A542 uses raw enemy_level 2's saved stru_3A334 cadence even if current HP later differs"
        );
        assert_eq!(rt.active_enemies[0].grounded_frames, 1);
    }

    #[test]
    fn fire_demon_typed_child_projects_to_trace_fire_trail_loc_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        rt.fire_demon_trails.push(FireDemonTrailChild {
            x_pos: 0x0064_0000,
            y_pos: 0x00D0_0000,
            remaining_frames: 0x95,
            scheduler_delay_frames: 0,
        });

        let trace = rt.trace_frame();

        assert_eq!(
            trace.active_projectiles,
            vec![(0x0064_0000, 0x00D0_0000, false)]
        );
        assert_eq!(trace.active_projectile_details.len(), 1);
        assert_eq!(
            trace.active_projectile_details[0].kind,
            ActiveProjectileKind::FireDemonDeathTrail
        );
        assert_eq!(trace.active_projectile_details[0].remaining_frames, 0x95);
    }

    #[test]
    fn level_40_idle_tail_keeps_one_fire_demon_trail_loc_3a542_3a6ca() {
        let Some(manifest) = cached_generated_manifest() else {
            return;
        };
        let trace = ScriptRunner::new(manifest)
            .warp_to_level(0x40)
            .settle_recording_warp()
            .run(&vec![0; 360], 360);
        let mut fire_demon_phases = trace[0]
            .active_enemy_details
            .iter()
            .filter(|enemy| matches!(enemy.kind, NativeObjectKind::Enemy { object_type: 0x00 }))
            .map(|enemy| (enemy.layout_index, enemy.walk_phase))
            .collect::<Vec<_>>();
        fire_demon_phases.sort_unstable();
        assert_eq!(
            fire_demon_phases,
            vec![
                (Some(34), Some(34)),
                (Some(36), Some(31)),
                (Some(37), Some(31))
            ],
            "ASM Init_Animation sub_1040/GfxObject_Animate sub_113E and oracle Object_RAM preserve the high Fire Demon's distinct loader-cohort phase at recording frame 0",
        );
        let settle_frame = &trace[59];
        let settle_trails = settle_frame
            .active_projectile_details
            .iter()
            .filter(|projectile| projectile.kind == ActiveProjectileKind::FireDemonDeathTrail)
            .count();
        assert_eq!(
            settle_trails, 5,
            "ASM Fire_Demon.asm loc_3A542 and oracle Object_RAM keep five trail children at Level 0x40 frame 59",
        );
        assert_eq!(
            (settle_frame.number_objects, settle_frame.number_gfx_objects),
            (12, 12),
            "oracle Level 0x40 frame 59 counts three parents, five trails, the singleton palette helper, Kid, HUD, and the level worker",
        );
        let frame = trace.last().expect("Level 0x40 frame 359");
        let trail_count = frame
            .active_projectile_details
            .iter()
            .filter(|projectile| projectile.kind == ActiveProjectileKind::FireDemonDeathTrail)
            .count();

        assert_eq!(
            trail_count,
            1,
            "ASM Fire_Demon.asm loc_3A542/loc_3A6CA and oracle Level 0x40 frame 359 keep one live trail; the loc_11EE `$0160` boundary must cull the two right-side children: {:?}",
            (
                frame.number_objects,
                frame.number_gfx_objects,
                &frame.active_projectile_details
            ),
        );
        let mut fire_demon_x_positions = frame
            .active_enemy_details
            .iter()
            .filter(|enemy| matches!(enemy.kind, NativeObjectKind::Enemy { object_type: 0x00 }))
            .map(|enemy| enemy.x_pos)
            .collect::<Vec<_>>();
        fire_demon_x_positions.sort_unstable();
        assert!(
            fire_demon_x_positions
                .iter()
                .zip([0x0194_0000_u32, 0x01F4_0000, 0x0203_0000])
                .all(|(native, oracle)| native.abs_diff(oracle) <= 2 << 16),
            "ASM Fire_Demon.asm loc_3A542 -> loc_3C04E must retain the recording oracle's object-scheduler age within one 2px patrol step at frame 359: {:?}",
            (&fire_demon_x_positions, &frame.active_projectile_details),
        );
        let trail = frame
            .active_projectile_details
            .iter()
            .find(|projectile| projectile.kind == ActiveProjectileKind::FireDemonDeathTrail)
            .expect("one Fire Demon trail at Level 0x40 frame 359");
        assert!(
            trail.x_pos.abs_diff(0x0150_0000) <= 1 << 16,
            "ASM Fire_Demon.asm loc_3A63C copies the parent X within the state oracle's 1px scheduler tolerance: {:#010x}",
            trail.x_pos,
        );
        assert_eq!(
            trail.remaining_frames, 14,
            "ASM Fire_Demon.asm loc_3A6CA decrements d3 once per child-object pass after the oracle-aligned spawn",
        );
        assert_eq!(
            (frame.number_objects, frame.number_gfx_objects),
            (8, 8),
            "ASM Execute_Objects loc_E1E and Make_SpriteFromGfxObject loc_14A4 expose eight linked Object/Gfx workers at oracle Level 0x40 frame 359",
        );
    }

    #[test]
    fn dragon_walking_breath_projectile_projects_to_trace_loc_3d02a() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 0x0064_0000,
            y_pos: 0x00D0_0000,
            lifetime: 68,
            friendly: false,
            kind: EnemyProjectileKind::DragonWalkingBreath,
            ..Default::default()
        });

        let trace = rt.trace_frame();

        assert_eq!(
            trace.active_projectiles,
            vec![(0x0064_0000, 0x00D0_0000, false)]
        );
        assert_eq!(trace.active_projectile_details.len(), 1);
        assert_eq!(
            trace.active_projectile_details[0].kind,
            ActiveProjectileKind::DragonWalkingBreath,
            "ASM Dragon.asm loc_3D02A breath child should remain typed through the trace so the player renders `stru_3CCC8` instead of a generic projectile dot"
        );
        assert_eq!(trace.active_projectile_details[0].remaining_frames, 68);
    }

    #[test]
    fn maniaxe_axe_projectile_projects_to_trace_stru_8b86() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 0x0064_0000,
            y_pos: 0x00D0_0000,
            x_velocity: 0x0005_0000,
            friendly: true,
            kind: EnemyProjectileKind::ManiaxeAxe,
            ..Default::default()
        });

        let trace = rt.trace_frame();

        assert_eq!(
            trace.active_projectile_details[0].kind,
            ActiveProjectileKind::ManiaxeAxe,
            "ASM ManiaxeAxe_Init installs `stru_8B86`; trace must keep the axe typed so native play renders the four `maniaxe_axe_1..4` frames instead of a generic green dot"
        );
    }

    #[test]
    fn fire_demon_legacy_projectile_trace_signature_requires_all_gates_loc_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            x_pos: 0x0064_0000,
            y_pos: 0x00D0_0000,
            x_velocity: 0,
            lifetime: 45,
            gravity: 0x1800,
            ..Default::default()
        });
        rt.projectiles.push(EnemyProjectile {
            x_pos: 0x0068_0000,
            y_pos: 0x00D0_0000,
            x_velocity: 0,
            lifetime: 46,
            gravity: 0x1800,
            ..Default::default()
        });
        rt.projectiles.push(EnemyProjectile {
            x_pos: 0x006C_0000,
            y_pos: 0x00D0_0000,
            x_velocity: 0,
            lifetime: 100,
            friendly: true,
            gravity: 0,
            ..Default::default()
        });

        let trace = rt.trace_frame();
        let kinds = trace
            .active_projectile_details
            .iter()
            .map(|projectile| projectile.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                ActiveProjectileKind::FireDemonDeathTrail,
                ActiveProjectileKind::Generic,
                ActiveProjectileKind::Generic,
            ],
            "ASM Fire_Demon loc_3A63C legacy trace fallback should require hostile gravity-$1800 short-lived trail signatures, not just a matching velocity",
        );
    }

    #[test]
    fn fire_demon_death_trail_lifetime_uses_loc_3a6ca_counter() {
        let mut rt = NativeRuntime::new(manifest());
        rt.fire_demon_trails.push(FireDemonTrailChild {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            remaining_frames: 1,
            scheduler_delay_frames: 0,
        });

        rt.step_projectiles();

        assert!(
            rt.fire_demon_trails.is_empty(),
            "ASM Fire_Demon.asm loc_3A6CA hibernates, decrements d3 to zero, then falls through to loc_3A6DA without a generic zero-lifetime frame"
        );

        rt.fire_demon_trails.push(FireDemonTrailChild {
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            remaining_frames: 0,
            scheduler_delay_frames: 0,
        });

        rt.step_projectiles();

        assert_eq!(
            rt.fire_demon_trails[0].remaining_frames, 0xFFFF,
            "ASM Fire_Demon.asm loc_3A6CA uses word subtraction, so an unexpected d3=0 wraps and keeps the trail alive"
        );
    }

    #[test]
    fn fire_demon_death_trail_deletes_when_centre_support_has_block_loc_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        for col in 1..=3 {
            let offset = (2 * 5 + col) * 2;
            data[offset] = if col == 2 { 0xE0 } else { 0x60 };
        }
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.fire_demon_trails.push(FireDemonTrailChild {
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            remaining_frames: 0x96,
            scheduler_delay_frames: 0,
        });

        rt.step_projectiles();

        assert!(
            rt.fire_demon_trails.is_empty(),
            "ASM Fire_Demon loc_3A63C: `bclr #$F,d6; bne loc_3A6DA` deletes when the centre support word has bit 15 set"
        );
    }

    #[test]
    fn fire_demon_death_trail_survives_on_three_plain_solid_cells_loc_3a63c() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        for col in 1..=3 {
            data[(2 * 5 + col) * 2] = 0x60;
        }
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.fire_demon_trails.push(FireDemonTrailChild {
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            remaining_frames: 0x96,
            scheduler_delay_frames: 0,
        });

        rt.step_projectiles();

        assert_eq!(
            rt.fire_demon_trails.len(),
            1,
            "ASM Fire_Demon loc_3A63C keeps the trail only when d5/d6/d7 all mask to `$6000` and centre d6 has no block flag"
        );
    }

    #[test]
    fn drips_pool_growth_splash_does_not_spawn_surface_crawler_loc_33488() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(3 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 0x0020_0000,
            y_pos: 0x0026_0000,
            x_velocity: 0,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            gravity: 0,
            terrain_kill: true,
            drips_spawn_crawler_on_splash: false,
            ..Default::default()
        });

        rt.step_projectiles();

        assert!(
            rt.projectiles
                .iter()
                .all(|projectile| projectile.kind != EnemyProjectileKind::DripsDrop),
            "ASM Drips.asm loc_3345C ends the falling child once terrain collision reaches loc_33488",
        );
        assert!(
            rt.projectiles
                .iter()
                .all(|projectile| projectile.kind != EnemyProjectileKind::DripsSurfaceCrawler),
            "ASM Drips.asm loc_33488 first writes off_33520 pool-growth sprites before loc_335D8 can allocate a crawler",
        );
    }

    #[test]
    fn drips_sentinel_splash_spawns_surface_crawler_loc_335d8() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(3 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 0x0020_0000,
            y_pos: 0x0026_0000,
            x_velocity: 0,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            gravity: 0,
            terrain_kill: true,
            drips_spawn_crawler_on_splash: true,
            ..Default::default()
        });

        rt.step_projectiles();

        assert!(
            rt.projectiles
                .iter()
                .all(|projectile| projectile.kind != EnemyProjectileKind::DripsDrop),
            "ASM Drips.asm loc_3345C ends the falling child once terrain collision reaches the splash/reset path",
        );
        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_335D8 should allocate a hostile surface crawler after the drop splash");
        assert_eq!(
            crawler.x_pos, 0x0021_0000,
            "ASM loc_335D8 queues `$44(a0)=drop_x+1` for the surface crawler"
        );
        assert_eq!(
            crawler.y_pos, 0x0030_0000,
            "ASM Drips.asm loc_33488 adds `$A` to the falling child before loc_335D8 queues `$46(a0)=(drop_y+$A)&$FFF0` for the surface crawler"
        );
        assert_eq!(
            crawler.x_velocity.abs(),
            0x0000_3000,
            "ASM loc_335D8 seeds surface crawler x_vel to +/-$3000"
        );
        assert!(
            !crawler.friendly,
            "ASM loc_335D8 creates a hostile child object, not a friendly projectile"
        );
    }

    #[test]
    fn drips_owned_drop_advances_pool_and_reseeds_parent_only_on_splash_loc_33488() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(3 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        let mut drips = test_enemy(EnemyFamily::Drips, 0x0020_0000, 0x0020_0000, 1);
        drips.drips_attached = true;
        drips.wait_timer = DRIPS_DROP_IN_FLIGHT_TIMER;
        rt.active_enemies.push(drips);
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 0x0020_0000,
            y_pos: 0x0026_0000,
            x_velocity: 0,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            gravity: 0,
            terrain_kill: true,
            owner: Some(EnemyProjectileOwner {
                segment_index: 0,
                layout_index: 0,
                token: 0,
            }),
            ..Default::default()
        });

        rt.step_projectiles();

        let drips = rt
            .active_enemies
            .iter()
            .find(|enemy| enemy.family == EnemyFamily::Drips)
            .expect("Drips parent remains active after child splash");
        assert_eq!(
            drips.drips_surface_table_index, 0,
            "ASM Drips.asm loc_33488 enters stru_33332 before consuming off_33520"
        );
        assert_eq!(
            drips_splash_animation_remaining_loc_33488(drips.wait_timer),
            Some(DRIPS_SPLASH_ANIMATION_FRAMES_LOC_33488),
            "ASM Drips.asm loc_33488 hibernates through the splash animation before loc_3350E reseeds"
        );
        assert_eq!(drips.drips_pending_splash_x_pos, 0x0020_0000);
        assert_eq!(drips.drips_pending_splash_y_pos, 0x0030_0000);

        // ASM Drips.asm loc_33488 / stru_33332: the one duration-1 splash
        // frame finishes before the next off_33520 entry resumes.
        let mut drilled_blocks = Vec::new();
        for _ in 0..DRIPS_SPLASH_ANIMATION_FRAMES_LOC_33488 {
            step_active_enemies(
                &mut rt.active_enemies,
                rt.block_grid.as_ref(),
                None,
                None,
                None,
                None,
                PlayerForm::TheKid,
                Some(&mut rt.projectiles),
                &mut drilled_blocks,
            );
            rt.step_projectiles();
        }

        let drips = rt
            .active_enemies
            .iter()
            .find(|enemy| enemy.family == EnemyFamily::Drips)
            .expect("Drips parent remains active after splash animation");
        assert_eq!(
            drips.drips_surface_table_index, 1,
            "ASM Drips.asm loc_33488 should consume off_33520 entry 0 only after the splash animation hibernates"
        );
        assert_ne!(
            drips.wait_timer, DRIPS_DROP_IN_FLIGHT_TIMER,
            "ASM Drips.asm loc_3350E should reseed the dormant timer after loc_33488 finishes"
        );
        assert_eq!(drips.drips_pending_splash_x_pos, 0);
        assert_eq!(drips.drips_pending_splash_y_pos, 0);
        assert!(
            rt.projectiles
                .iter()
                .all(|projectile| projectile.kind != EnemyProjectileKind::DripsSurfaceCrawler),
            "ASM off_33520 entry 0 is pool growth, not loc_335D8 crawler allocation"
        );
    }

    #[test]
    fn drips_burst_helper_still_counts_against_surface_cap_loc_3392c() {
        let mut rt = NativeRuntime::new(manifest());
        let mut drips = test_enemy(EnemyFamily::Drips, 0x0020_0000, 0x0020_0000, 1);
        drips.drips_attached = true;
        drips.wait_timer = drips_splash_animation_timer_loc_33488(0);
        drips.drips_surface_table_index = 2;
        drips.drips_pending_splash_x_pos = 0x0020_0000;
        drips.drips_pending_splash_y_pos = 0x002A_0000;
        rt.active_enemies.push(drips);
        for index in 0..crate::projectiles::DRIPS_SURFACE_CRAWLER_LIMIT_LOC_33488 {
            rt.projectiles.push(EnemyProjectile {
                kind: EnemyProjectileKind::DripsSurfaceCrawlerBurst,
                x_pos: (0x0040 + index as u32) << 16,
                y_pos: 0x0040_0000,
                lifetime: 1,
                ..Default::default()
            });
        }

        rt.step_projectiles();

        assert!(
            rt.projectiles
                .iter()
                .all(|projectile| projectile.kind != EnemyProjectileKind::DripsSurfaceCrawler),
            "ASM Drips.asm loc_33488 checks `$FFFFFB42 < 5`; loc_3392C decrements it only after the loc_338F8 burst-helper animation finishes"
        );
        let drips = rt
            .active_enemies
            .iter()
            .find(|enemy| enemy.family == EnemyFamily::Drips)
            .expect("Drips parent remains active after capped splash");
        assert_eq!(
            drips.drips_surface_table_index, 0,
            "ASM Drips.asm loc_33506 still rewinds the off_33520 cursor when the allocation cap blocks loc_335D8"
        );
    }

    #[test]
    fn drips_ready_splash_sentinel_allocates_and_counts_crawler_loc_33488() {
        let mut rt = NativeRuntime::new(manifest());
        let mut drips = test_enemy(EnemyFamily::Drips, 0x0020_0000, 0x0020_0000, 1);
        drips.drips_attached = true;
        drips.wait_timer = drips_splash_animation_timer_loc_33488(0);
        drips.drips_surface_table_index = 2;
        drips.drips_pending_splash_x_pos = 0x0020_0000;
        drips.drips_pending_splash_y_pos = 0x002A_0000;
        rt.active_enemies.push(drips);

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33488 negative off_33520 sentinel allocates loc_335D8 while `$FFFFFB42 < 5`");
        assert_eq!(
            crawler.x_pos, 0x0021_0000,
            "ASM Drips.asm loc_33488 queues `$44(a0)=drop_x+1` for loc_335D8"
        );
        assert_eq!(
            crawler.y_pos, 0x0030_0000,
            "ASM Drips.asm loc_33488 queues `$46(a0)=(drop_y+$A)&$FFF0` for loc_335D8"
        );
        assert_eq!(
            rt.drips_surface_crawler_count_loc_fffffb42, 1,
            "ASM Drips.asm loc_33488 increments `$FFFFFB42` immediately after allocating loc_335D8"
        );
        let drips = rt
            .active_enemies
            .iter()
            .find(|enemy| enemy.family == EnemyFamily::Drips)
            .expect("Drips parent remains active after sentinel splash");
        assert_eq!(
            drips.drips_surface_table_index, 0,
            "ASM Drips.asm loc_33506 rewinds the off_33520 cursor after the negative sentinel"
        );
    }

    #[test]
    fn drips_sentinel_splash_runs_crawler_rng_before_parent_reseed_loc_33488() {
        let mut rt = NativeRuntime::new(manifest());
        let mut drips = test_enemy(EnemyFamily::Drips, 0x0020_0000, 0x0020_0000, 1);
        drips.drips_attached = true;
        drips.wait_timer = drips_splash_animation_timer_loc_33488(0);
        drips.drips_surface_table_index = 2;
        drips.drips_pending_splash_x_pos = 0x0020_0000;
        drips.drips_pending_splash_y_pos = 0x002A_0000;
        rt.active_enemies.push(drips);

        rt.step_projectiles();

        let drips = rt
            .active_enemies
            .iter()
            .find(|enemy| enemy.family == EnemyFamily::Drips)
            .expect("Drips parent remains active after sentinel splash");
        assert_eq!(
            drips.wait_timer, 0x0024,
            "ASM Drips.asm loc_33488 queues loc_335D8 before the parent returns to loc_3340E; \
            oracle issue-260 frame 639 already has the child's random direction branch applied, \
            so the parent reseed consumes the following RNG byte"
        );
        assert!(
            rt.projectiles
                .iter()
                .any(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler),
            "ASM Drips.asm loc_33488 still allocates the loc_335D8 crawler on the negative off_33520 sentinel"
        );
    }

    #[test]
    fn drips_ready_splash_reseeds_from_kid_gfx_x_loc_3340e_issue_260() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 0x0040_0000;
        rt.opening_slice = Some(slice);
        let mut drips = test_enemy(EnemyFamily::Drips, 0x0020_0000, 0x0020_0000, 1);
        drips.drips_attached = true;
        drips.wait_timer = drips_splash_animation_timer_loc_33488(0);
        drips.drips_surface_table_index = 2;
        drips.drips_pending_splash_x_pos = 0x0020_0000;
        drips.drips_pending_splash_y_pos = 0x002A_0000;
        rt.active_enemies.push(drips);

        rt.step_projectiles();

        let drips = rt
            .active_enemies
            .iter()
            .find(|enemy| enemy.family == EnemyFamily::Drips)
            .expect("Drips parent remains active after sentinel splash");
        assert_eq!(
            drips.wait_timer, 0x0064,
            "ASM Drips.asm loc_3350E returns to loc_3340E after the splash; loc_3340E reads Addr_GfxObject_Kid x_pos directly, so issue-260 post-splash reseeds must not add a Drips-local 16px offset"
        );
    }

    #[test]
    fn drips_same_frame_ready_splashes_share_surface_cap_loc_33488() {
        let mut rt = NativeRuntime::new(manifest());
        for index in 0..6 {
            let mut drips = test_enemy(
                EnemyFamily::Drips,
                (0x0020 + index) << 16,
                0x0020_0000,
                index as u8,
            );
            drips.drips_attached = true;
            drips.wait_timer = drips_splash_animation_timer_loc_33488(0);
            drips.drips_surface_table_index = 2;
            drips.drips_pending_splash_x_pos = (0x0020 + index) << 16;
            drips.drips_pending_splash_y_pos = 0x002A_0000;
            rt.active_enemies.push(drips);
        }

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles
                .iter()
                .filter(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
                .count(),
            crate::projectiles::DRIPS_SURFACE_CRAWLER_LIMIT_LOC_33488,
            "ASM Drips.asm loc_33488 checks the shared `$FFFFFB42 < 5` counter for every same-frame sentinel splash"
        );
        assert_eq!(
            rt.drips_surface_crawler_count_loc_fffffb42,
            crate::projectiles::DRIPS_SURFACE_CRAWLER_LIMIT_LOC_33488,
            "ASM Drips.asm loc_33488 increments `$FFFFFB42` once per allocated loc_335D8 crawler"
        );
    }

    #[test]
    fn drips_typed_children_project_to_trace_kinds_loc_33424_loc_335d8() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            lifetime: DRIPS_DROP_LIFETIME,
            ..Default::default()
        });
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0021_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x0000_3000,
            lifetime: DRIPS_DROP_LIFETIME,
            ..Default::default()
        });

        let trace = rt.trace_frame();

        assert_eq!(trace.active_projectile_details.len(), 2);
        assert_eq!(
            trace.active_projectile_details[0].kind,
            ActiveProjectileKind::DripsDrop,
            "ASM Drips.asm loc_33424/loc_3345C falling children should remain distinguishable from generic projectiles in trace output",
        );
        assert_eq!(
            trace.active_projectile_details[1].kind,
            ActiveProjectileKind::DripsSurfaceCrawler,
            "ASM Drips.asm loc_335D8 crawler children should remain distinguishable from generic projectiles in trace output",
        );
    }

    #[test]
    fn drips_surface_crawler_spawn_honours_five_child_cap_loc_33488() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0xE0;
        for col in 1..=2 {
            data[(2 * 5 + col) * 2] = 0x60;
        }
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        for index in 0..5u32 {
            rt.projectiles.push(EnemyProjectile {
                kind: EnemyProjectileKind::DripsSurfaceCrawler,
                x_pos: (0x0010 + index) << 16,
                y_pos: 0x0020_0000,
                x_velocity: 0x3000,
                lifetime: DRIPS_DROP_LIFETIME,
                ..Default::default()
            });
        }
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            lifetime: DRIPS_DROP_LIFETIME,
            terrain_kill: true,
            drips_spawn_crawler_on_splash: true,
            ..Default::default()
        });

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles
                .iter()
                .filter(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
                .count(),
            5,
            "ASM Drips.asm loc_33488 only allocates loc_335D8 crawlers while `$FFFFFB42 < 5`"
        );
    }

    #[test]
    fn drips_surface_crawler_enters_finish_when_right_probe_reaches_level_width_loc_33588() {
        let mut rt = NativeRuntime::new(manifest());
        let level_width = rt.level_bounds(rt.current_level).width_pixels;
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: u32::from(level_width.saturating_sub(8)) << 16,
            y_pos: 0x0020_0000,
            x_velocity: 0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let finish = rt.projectiles.first().expect(
            "ASM Drips.asm loc_33588 routes contact to loc_338EA/loc_3392C before deletion",
        );
        assert_eq!(finish.kind, EnemyProjectileKind::DripsSurfaceCrawlerFinish);
        assert_eq!(finish.x_velocity, 0);
        assert_eq!(finish.y_velocity, 0);
    }

    #[test]
    fn drips_surface_crawler_advances_while_inside_level_loc_33650() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33650 keeps an in-bounds crawler alive after its one-frame hibernate");
        assert_eq!(
            crawler.x_pos, 0x0020_3000,
            "ASM Drips.asm loc_33650 reaches the normal movement path after loc_33588 reports no level-edge contact"
        );
        assert_eq!(
            crawler.y_pos, 0x0020_0000,
            "ASM Drips.asm loc_33650 leaves y unchanged on the initial horizontal crawler path"
        );
    }

    #[test]
    fn drips_surface_crawler_deletes_without_solid_support_loc_33650() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(
            &[0u8; 5 * 5 * 2],
            5,
            5,
        ));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        assert!(
            rt.projectiles
                .iter()
                .all(|projectile| projectile.kind != EnemyProjectileKind::DripsSurfaceCrawler),
            "ASM Drips.asm loc_33650 routes non-$6000 support from loc_331D2 out of the crawler loop"
        );
        let helper = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawlerBurst)
            .expect(
                "ASM Drips.asm loc_33650 branches to loc_338F8, which allocates Tar_Monster.asm loc_325B6 before loc_3392C deletes the crawler",
            );
        assert_eq!(
            helper.x_pos, 0x0020_0000,
            "ASM Drips.asm loc_338F8 adds d6=0 before copying x_pos to the loc_325B6 helper"
        );
        assert_eq!(
            helper.y_pos, 0x0018_0000,
            "ASM Drips.asm loc_338F8 preserves d7=-8 from loc_33650/loc_33550 before copying y_pos"
        );
    }

    #[test]
    fn drips_surface_crawler_solid_support_preserves_loc_33650_movement() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect(
                "ASM Drips.asm loc_33650 should keep crawling while loc_331D2 sees `$6000` support",
            );
        assert_eq!(
            crawler.x_pos, 0x0020_3000,
            "ASM loc_33650 support probe should allow the normal horizontal crawler movement"
        );
    }

    #[test]
    fn drips_surface_crawler_leftward_current_solid_loops_loc_33650() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        data[(2 * 5 + 1) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0025_0000,
            y_pos: 0x0020_0000,
            x_velocity: -0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33650 loops when loc_331F6 d5 is `$6000`");
        assert_eq!(
            crawler.x_pos, 0x0024_D000,
            "ASM loc_33650 current-row `$6000` branch should keep normal leftward crawler movement"
        );
        assert_eq!(crawler.y_pos, 0x0020_0000);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::HorizontalLoc33650,
            "ASM loc_33650 `cmpi.w #$6000,d5; beq.s loc_33650` stays in horizontal movement"
        );
    }

    #[test]
    fn drips_surface_crawler_leftward_upper_solid_runs_loc_3393c_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        data[(5 + 1) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0025_0000,
            y_pos: 0x0020_0000,
            x_velocity: -0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33650 routes loc_331F6 d6 `$6000` through loc_3393C, not deletion");
        assert_eq!(
            crawler.x_pos, 0x0021_0000,
            "ASM loc_3393C/loc_33AF6 snaps X to `$FFF0`, applies the +1 first animation delta, then no second X delta"
        );
        assert_eq!(
            crawler.y_pos, 0x001B_0000,
            "ASM loc_3393C/loc_33B5E applies the -5 second Y delta before jumping to loc_33730"
        );
        assert_eq!(
            crawler.x_velocity, 0,
            "ASM loc_33AF6 clears x_vel before loc_3393C applies no X velocity delta"
        );
        assert_eq!(
            crawler.y_velocity, -0x3000,
            "ASM loc_3393C/loc_33B5E seeds y_vel with -$3000"
        );
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::VerticalLoc33730,
            "ASM loc_3393C dispatches through off_33ADA to loc_33730 after the turn script"
        );
    }

    #[test]
    fn drips_surface_crawler_leftward_upper_slope_runs_loc_33a40_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        data[(5 + 1) * 2] = 0x50;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0025_0000,
            y_pos: 0x0020_0000,
            x_velocity: -0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33650 routes loc_331F6 d6 `$5000` through loc_33A40, not deletion");
        assert_eq!(
            crawler.x_pos, 0x001D_0000,
            "ASM loc_33A40/loc_33AF6 snaps X, applies +1, then applies the -4 second X delta"
        );
        assert_eq!(
            crawler.y_pos, 0x001D_0000,
            "ASM loc_33A40/loc_33B5E applies the -3 second Y delta before jumping to loc_33892"
        );
        assert_eq!(
            crawler.x_velocity, -0x2000,
            "ASM loc_33A40/loc_33B5E seeds x_vel with -$2000"
        );
        assert_eq!(
            crawler.y_velocity, -0x2000,
            "ASM loc_33A40/loc_33B5E seeds y_vel with -$2000"
        );
    }

    #[test]
    fn drips_surface_crawler_leftward_current_corner_runs_loc_33a2c_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        data[(2 * 5 + 1) * 2] = 0x40;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0025_0000,
            y_pos: 0x0020_0000,
            x_velocity: -0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33650 routes loc_331F6 d5 `$4000` through loc_33A2C, not deletion");
        assert_eq!(crawler.x_pos, 0x001C_0000);
        assert_eq!(crawler.y_pos, 0x0024_0000);
        assert_eq!(crawler.x_velocity, -0x2000);
        assert_eq!(crawler.y_velocity, 0x2000);
    }

    #[test]
    fn drips_surface_crawler_leftward_empty_side_runs_loc_33982_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0025_0000,
            y_pos: 0x0020_0000,
            x_velocity: -0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33650 falls through to loc_33982 when d6/d5 do not match earlier left-facing turn cases");
        assert_eq!(crawler.x_pos, 0x001F_0000);
        assert_eq!(crawler.y_pos, 0x0025_0000);
        assert_eq!(crawler.x_velocity, 0);
        assert_eq!(crawler.y_velocity, 0x3000);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::VerticalLoc337A0
        );
    }

    #[test]
    fn drips_surface_crawler_vertical_loc_33730_loops_on_left_solid_row() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x002F_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: -0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::VerticalLoc33730,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33730 loops when loc_33286 d5 reports `$6000`");
        assert_eq!(crawler.x_pos, 0x002F_0000);
        assert_eq!(crawler.y_pos, 0x001F_D000);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::VerticalLoc33730
        );
    }

    #[test]
    fn drips_surface_crawler_vertical_loc_33730_enters_finish_without_upper_support() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(5 + 1) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: -0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::VerticalLoc33730,
            ..Default::default()
        });

        rt.step_projectiles();

        let finish = rt
            .projectiles
            .first()
            .expect("ASM Drips.asm loc_33730 routes non-$6000 loc_331D2 upper support to loc_338F8, then loc_3392C");
        assert_eq!(finish.kind, EnemyProjectileKind::DripsSurfaceCrawlerBurst);
        assert_eq!(finish.x_velocity, 0);
    }

    #[test]
    fn drips_surface_crawler_vertical_loc_33730_centre_solid_runs_loc_33964_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: -0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::VerticalLoc33730,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33730 routes loc_33286 d6 `$6000` through loc_33964");
        assert_eq!(crawler.x_pos, 0x0025_0000);
        assert_eq!(crawler.y_pos, 0x0021_0000);
        assert_eq!(crawler.x_velocity, 0x3000);
        assert_eq!(crawler.y_velocity, 0);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::VerticalLoc336D0
        );
        assert!(crawler.drips_surface_crawler_x_direction_flipped);
    }

    #[test]
    fn drips_surface_crawler_vertical_loc_33730_falling_centre_solid_runs_loc_33946_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(5 + 2) * 2] = 0x60;
        data[(2 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::VerticalLoc33730,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33782 routes loc_33286 d6 `$6000` through loc_33946");
        assert_eq!(crawler.x_pos, 0x0025_0000);
        assert_eq!(crawler.y_pos, 0x0020_0000);
        assert_eq!(crawler.x_velocity, 0x3000);
        assert_eq!(crawler.y_velocity, 0);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::HorizontalLoc33650
        );
        assert!(!crawler.drips_surface_crawler_x_direction_flipped);
    }

    #[test]
    fn drips_surface_crawler_vertical_loc_33730_falling_centre_slope_runs_loc_33a72_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(5 + 2) * 2] = 0x60;
        data[(2 * 5 + 2) * 2] = 0x50;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::VerticalLoc33730,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33782 routes loc_33286 d6 `$5000` through loc_33A72");
        assert_eq!(crawler.x_pos, 0x0022_0000);
        assert_eq!(crawler.y_pos, 0x0023_0000);
        assert_eq!(crawler.x_velocity, 0x2000);
        assert_eq!(crawler.y_velocity, 0x2000);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::DiagonalLoc33892
        );
        assert!(crawler.drips_surface_crawler_x_direction_flipped);
    }

    #[test]
    fn drips_surface_crawler_vertical_loc_33730_falling_fallback_runs_loc_339a0_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::VerticalLoc33730,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33782 falls through to loc_339A0 when loc_33286 d6/d5 do not match");
        assert_eq!(crawler.x_pos, 0x001A_0000);
        assert_eq!(crawler.y_pos, 0x0020_0000);
        assert_eq!(crawler.x_velocity, -0x3000);
        assert_eq!(crawler.y_velocity, 0);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::VerticalLoc336D0
        );
    }

    #[test]
    fn drips_surface_crawler_vertical_loc_337a0_negative_left_solid_runs_loc_339be_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(5 + 1) * 2] = 0x60;
        data[(2 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: -0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::VerticalLoc337A0,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_337A0 routes loc_33286 d5 `$6000` through loc_339BE");
        assert_eq!(crawler.x_pos, 0x001B_0000);
        assert_eq!(crawler.y_pos, 0x0021_0000);
        assert_eq!(crawler.x_velocity, -0x3000);
        assert_eq!(crawler.y_velocity, 0);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::VerticalLoc336D0
        );
        assert!(crawler.drips_surface_crawler_x_direction_flipped);
    }

    #[test]
    fn drips_surface_crawler_vertical_loc_336d0_nonflipped_upper_solid_runs_loc_3396e_turn_script()
    {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 1) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0022_0000,
            x_velocity: 0,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::VerticalLoc336D0,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect(
                "ASM Drips.asm loc_336D0 routes non-flipped loc_331F6 d6 `$6000` through loc_3396E",
            );
        assert_eq!(crawler.x_pos, 0x0021_0000);
        assert_eq!(crawler.y_pos, 0x0027_0000);
        assert_eq!(crawler.x_velocity, 0);
        assert_eq!(crawler.y_velocity, 0x3000);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::VerticalLoc33730
        );
        assert!(!crawler.drips_surface_crawler_x_direction_flipped);
    }

    #[test]
    fn drips_surface_crawler_vertical_loc_336d0_flipped_current_solid_runs_loc_339b4_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 1) * 2] = 0x60;
        data[(2 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::VerticalLoc336D0,
            drips_surface_crawler_x_direction_flipped: true,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect(
                "ASM Drips.asm loc_3371A routes flipped loc_331F6 d5 `$6000` through loc_339B4",
            );
        assert_eq!(crawler.x_pos, 0x001F_0000);
        assert_eq!(crawler.y_pos, 0x0025_0000);
        assert_eq!(crawler.x_velocity, 0);
        assert_eq!(crawler.y_velocity, 0x3000);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::VerticalLoc337A0
        );
        assert!(crawler.drips_surface_crawler_x_direction_flipped);
    }

    #[test]
    fn drips_surface_crawler_vertical_loc_337a0_negative_centre_corner_runs_loc_33a04_turn_script()
    {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(5 + 2) * 2] = 0x40;
        data[(2 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: -0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::VerticalLoc337A0,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_337A0 routes loc_33286 d6 `$4000` through loc_33A04");
        assert_eq!(crawler.x_pos, 0x0020_0000);
        assert_eq!(crawler.y_pos, 0x0020_0000);
        assert_eq!(crawler.x_velocity, 0x2000);
        assert_eq!(crawler.y_velocity, -0x2000);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::DiagonalLoc33810
        );
    }

    #[test]
    fn drips_surface_crawler_diagonal_loc_33810_waits_for_alignment_sum() {
        let mut rt = NativeRuntime::new(manifest());
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(
            &[0u8; 5 * 5 * 2],
            5,
            5,
        ));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: -0x2000,
            y_velocity: 0x2000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::DiagonalLoc33810,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33810 loops until low-nibble X/Y alignment sums to `$F`");
        assert_eq!(crawler.x_pos, 0x001F_E000);
        assert_eq!(crawler.y_pos, 0x0020_2000);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::DiagonalLoc33810
        );
    }

    #[test]
    fn drips_surface_crawler_diagonal_loc_33810_negative_solid_runs_loc_33a54_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0027_0000,
            y_pos: 0x0028_0000,
            x_velocity: -0x2000,
            y_velocity: -0x2000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::DiagonalLoc33810,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect(
                "ASM Drips.asm loc_33810 routes negative-y loc_3330E d7 `$6000` through loc_33A54",
            );
        assert_eq!(crawler.x_pos, 0x001F_0000);
        assert_eq!(crawler.y_pos, 0x0022_0000);
        assert_eq!(crawler.x_velocity, 0);
        assert_eq!(crawler.y_velocity, -0x3000);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::VerticalLoc337A0
        );
    }

    #[test]
    fn drips_surface_crawler_diagonal_loc_33892_positive_solid_runs_loc_33a22_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x2000,
            y_velocity: 0x2000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::DiagonalLoc33892,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_33892 routes non-negative-y loc_3330E d7 `$6000` through loc_33A22");
        assert_eq!(crawler.x_pos, 0x002A_0000);
        assert_eq!(crawler.y_pos, 0x0020_0000);
        assert_eq!(crawler.x_velocity, 0x3000);
        assert_eq!(crawler.y_velocity, 0);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::HorizontalLoc33650
        );
    }

    #[test]
    fn drips_surface_crawler_vertical_loc_337a0_positive_left_solid_runs_loc_3398c_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x002F_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::VerticalLoc337A0,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_337F2 routes loc_33286 d5 `$6000` through loc_3398C");
        assert_eq!(crawler.x_pos, 0x002A_0000);
        assert_eq!(crawler.y_pos, 0x001F_0000);
        assert_eq!(crawler.x_velocity, -0x3000);
        assert_eq!(crawler.y_velocity, 0);
        assert_eq!(
            crawler.drips_surface_crawler_state,
            DripsSurfaceCrawlerState::HorizontalLoc33650
        );
    }

    #[test]
    fn drips_surface_crawler_rightward_upper_corner_runs_loc_339dc_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        data[(5 + 2) * 2] = 0x40;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_336AA routes loc_331F6 d6 `$4000` through loc_339DC, not deletion");
        assert_eq!(crawler.x_pos, 0x0024_0000);
        assert_eq!(crawler.y_pos, 0x001D_0000);
        assert_eq!(crawler.x_velocity, 0x2000);
        assert_eq!(crawler.y_velocity, -0x2000);
    }

    #[test]
    fn drips_surface_crawler_rightward_upper_solid_runs_loc_33996_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        data[(5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_336AA routes loc_331F6 d6 `$6000` through loc_33996, not deletion");
        assert_eq!(crawler.x_pos, 0x0020_0000);
        assert_eq!(crawler.y_pos, 0x001B_0000);
        assert_eq!(crawler.x_velocity, 0);
        assert_eq!(crawler.y_velocity, -0x3000);
    }

    #[test]
    fn drips_surface_crawler_rightward_current_slope_runs_loc_33a18_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        data[(2 * 5 + 3) * 2] = 0x50;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x002A_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_336AA routes loc_331F6 d5 `$5000` through loc_33A18, not deletion");
        assert_eq!(crawler.x_pos, 0x0035_0000);
        assert_eq!(crawler.y_pos, 0x0025_0000);
        assert_eq!(crawler.x_velocity, 0x2000);
        assert_eq!(crawler.y_velocity, 0x2000);
    }

    #[test]
    fn drips_surface_crawler_rightward_empty_side_runs_loc_339c8_turn_script() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x002A_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0x3000,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let crawler = rt
            .projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::DripsSurfaceCrawler)
            .expect("ASM Drips.asm loc_336AA falls through to loc_339C8 when d6/d5 do not match the earlier turn cases");
        assert_eq!(crawler.x_pos, 0x0031_0000);
        assert_eq!(crawler.y_pos, 0x0025_0000);
        assert_eq!(crawler.x_velocity, 0);
        assert_eq!(crawler.y_velocity, 0x3000);
    }

    #[test]
    fn drips_surface_crawler_enters_finish_when_left_probe_crosses_zero_loc_335aa() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0007_0000,
            y_pos: 0x0020_0000,
            x_velocity: -0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        rt.step_projectiles();

        let finish = rt.projectiles.first().expect(
            "ASM Drips.asm loc_335AA routes `x_pos(a3)-8` contact through loc_338EA/loc_3392C",
        );
        assert_eq!(finish.kind, EnemyProjectileKind::DripsSurfaceCrawlerFinish);
        assert_eq!(finish.x_velocity, 0);
        assert_eq!(finish.y_velocity, 0);
    }

    #[test]
    fn drips_surface_crawler_deletes_when_vertical_probe_leaves_level_loc_335b0() {
        let mut upward = NativeRuntime::new(manifest());
        upward.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: 0x0007_0000,
            x_velocity: 0,
            y_velocity: -0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        upward.step_projectiles();

        let upward_finish = upward.projectiles.first().expect(
            "ASM Drips.asm loc_335B0/loc_335CA routes upper contact through loc_338EA/loc_3392C",
        );
        assert_eq!(
            upward_finish.kind,
            EnemyProjectileKind::DripsSurfaceCrawlerFinish
        );
        assert_eq!(upward_finish.x_velocity, 0);
        assert_eq!(upward_finish.y_velocity, 0);

        let mut downward = NativeRuntime::new(manifest());
        let level_height = downward.level_bounds(downward.current_level).height_pixels;
        downward.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 0x0020_0000,
            y_pos: u32::from(level_height.saturating_sub(8)) << 16,
            x_velocity: 0,
            y_velocity: 0x3000,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            ..Default::default()
        });

        downward.step_projectiles();

        let downward_finish = downward
            .projectiles
            .first()
            .expect("ASM Drips.asm loc_335B0 routes lower contact through loc_338EA/loc_3392C");
        assert_eq!(
            downward_finish.kind,
            EnemyProjectileKind::DripsSurfaceCrawlerFinish
        );
        assert_eq!(downward_finish.x_velocity, 0);
        assert_eq!(downward_finish.y_velocity, 0);
    }

    #[test]
    fn drips_drop_floor_collision_uses_object_bottom_loc_3345c() {
        // Solid floor cell at (col 2, row 3) => floor top at y=48. The falling
        // child's GfxObjects_Collision box bottom is `y+$A`, so contact fires
        // at y=38 (bottom edge 48 enters row 3) and not at y=37 (bottom 47 is
        // still row 2). Oracle issue-260 floor probe: fall ends at y=582 with
        // the splash resting at the 592 floor top — bottom `+$A`, not `+$1F`.
        let mut above = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(3 * 5 + 2) * 2] = 0x60;
        above.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        above.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 0x0020_0000,
            y_pos: 0x0025_0000,
            x_velocity: 0,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            terrain_kill: true,
            ..Default::default()
        });

        above.step_projectiles();

        assert!(
            above
                .projectiles
                .iter()
                .any(|projectile| projectile.kind == EnemyProjectileKind::DripsDrop),
            "ASM Drips.asm loc_3345C: at y=37 the `y+$A` bottom edge (47) is still above the row-3 floor, so no `$0C` contact yet"
        );

        let mut contact = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(3 * 5 + 2) * 2] = 0x60;
        contact.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        contact.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 0x0020_0000,
            y_pos: 0x0026_0000,
            x_velocity: 0,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            terrain_kill: true,
            owner: Some(EnemyProjectileOwner {
                segment_index: 0,
                layout_index: 0,
                token: 0,
            }),
            ..Default::default()
        });
        contact
            .active_enemies
            .push(test_enemy(EnemyFamily::Drips, 0x0020_0000, 0x0020_0000, 1));

        contact.step_projectiles();

        assert!(
            contact.projectiles.is_empty(),
            "ASM Drips.asm loc_3345C reports floor `$0C` on the frame the `y+$A` bottom edge enters the solid row"
        );
        assert_eq!(
            contact.played_sound_effects,
            vec![QueuedSoundEffect::DripsDripping],
            "ASM Drips.asm loc_3345C plays sfx_Drips_dripping before loc_33488"
        );
    }

    #[test]
    fn drips_drop_origin_overlap_does_not_splash_before_floor_collision_loc_3345c() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(2 * 5 + 2) * 2] = 0xE0;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 0x0020_0000,
            y_pos: 0x0020_0000,
            x_velocity: 0,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            terrain_kill: true,
            ..Default::default()
        });

        rt.step_projectiles();

        assert!(
            rt.projectiles
                .iter()
                .any(|projectile| projectile.kind == EnemyProjectileKind::DripsDrop),
            "ASM Drips.asm loc_3345C compares collision_type(a1) with `$0C`; a solid cell at the falling child's origin is not enough to splash before the +$1F floor probe reaches terrain",
        );
    }

    #[test]
    fn drips_drop_nearby_floor_row_does_not_splash_before_contact_loc_3345c() {
        let mut rt = NativeRuntime::new(manifest());
        let mut data = vec![0u8; 5 * 5 * 2];
        data[(3 * 5 + 2) * 2] = 0x60;
        rt.block_grid = Some(terrain::BlockGrid::from_level_layout(&data, 5, 5));
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsDrop,
            x_pos: 0x0020_0000,
            y_pos: 0x000f_0000,
            x_velocity: 0,
            y_velocity: 0,
            lifetime: DRIPS_DROP_LIFETIME,
            friendly: false,
            terrain_kill: true,
            ..Default::default()
        });

        rt.step_projectiles();

        assert!(
            rt.projectiles
                .iter()
                .any(|projectile| projectile.kind == EnemyProjectileKind::DripsDrop),
            "ASM Drips.asm loc_3345C waits for collision_type `$0C`; the one-row lookahead cell selected by native floor probing is not itself a splash contact",
        );
    }

    #[test]
    fn projectile_misses_player_outside_x_range() {
        // Projectile 14px right of player — exactly at boundary (< 14 fails).
        // half=6, half+8=14. |prx - px| = 14, NOT < 14 → miss.
        // Kills: < with <= on x-axis (line 146 first <).
        let mut rt = runtime_with_projectile_and_player(114, 100, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            !rt.projectile_hits_player(&slice),
            "projectile 14px away in x should miss (boundary of < 14)"
        );
        assert_eq!(rt.projectiles.len(), 1, "missed projectile should survive");
    }

    #[test]
    fn projectile_hits_player_at_x_boundary() {
        // Projectile 13px right of player — just inside x range.
        // |prx - px| = 13 < 14 → hit (if y also overlaps).
        let mut rt = runtime_with_projectile_and_player(113, 100, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            rt.projectile_hits_player(&slice),
            "projectile 13px away in x should hit (just inside < 14)"
        );
    }

    #[test]
    fn lion_projectile_collision_uses_sprite_row_unk_31944_and_misses_issue_289_gap() {
        let mut rt = NativeRuntime::new(manifest());
        let mut projectile = make_projectile(459, 687);
        projectile.kind = EnemyProjectileKind::LionProjectile;
        rt.projectiles.push(projectile);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 448 << 16;
        slice.player.y_pos = 703 << 16;
        rt.opening_slice = Some(slice);

        assert!(
            !rt.projectile_hits_player(&slice),
            "ASM Lion.asm loc_3E12A -> stru_3DF34 maps C7E20/C7E28/C7E30 to CollisionSize_Index row unk_31944 (-5,9,-5,9); at issue-289 frame 28 the projectile box starts one pixel right of Kid_hitbox_right"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM Lion.asm loc_3E192 only deletes the child after GfxObjects_CollisionKid writes a collision type"
        );
        assert_eq!(
            rt.projectiles[0].lifetime, 100,
            "the non-contact Lion projectile should remain live for the next loc_3E192 frame"
        );
    }

    #[test]
    fn lion_projectile_collision_uses_inclusive_gfxobject_edges_loc_1bce() {
        let mut rt = NativeRuntime::new(manifest());
        let mut projectile = make_projectile(458, 687);
        projectile.kind = EnemyProjectileKind::LionProjectile;
        rt.projectiles.push(projectile);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 448 << 16;
        slice.player.y_pos = 703 << 16;
        rt.opening_slice = Some(slice);

        assert!(
            rt.projectile_hits_player(&slice),
            "ASM GfxObjects_CollisionKid loc_1BCE rejects with strict bgt/blt misses, so Lion projectile left edge == Kid_hitbox_right is still contact"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM Lion.asm loc_3E192 only deletes on `colid_kidabove`/`colid_hurt`; side/below damage contacts leave the shot alive"
        );
        assert_eq!(
            rt.projectiles[0].lifetime, 100,
            "this side/below Lion contact should not mark the child for the loc_3E192 delete tail"
        );
    }

    #[test]
    fn projectile_misses_player_outside_y_range() {
        // Projectile 22px below player — exactly at boundary (< 22 fails).
        // half=6, half+16=22. |pry - py| = 22, NOT < 22 → miss.
        // Kills: < with <= on y-axis (line 146 second <).
        let mut rt = runtime_with_projectile_and_player(100, 122, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            !rt.projectile_hits_player(&slice),
            "projectile 22px away in y should miss (boundary of < 22)"
        );
    }

    #[test]
    fn projectile_hits_player_at_y_boundary() {
        // Projectile 21px below player — just inside y range.
        let mut rt = runtime_with_projectile_and_player(100, 121, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            rt.projectile_hits_player(&slice),
            "projectile 21px away in y should hit (just inside < 22)"
        );
    }

    #[test]
    fn projectile_hits_player_negative_offset() {
        // Projectile 13px LEFT of player — tests abs() and subtraction direction.
        // Kills: - with + on (prx - px) and (pry - py).
        let mut rt = runtime_with_projectile_and_player(87, 79, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            rt.projectile_hits_player(&slice),
            "projectile 13px left and 21px above should hit"
        );
        assert!(
            rt.last_projectile_damage_probe
                .as_deref()
                .is_some_and(|probe| probe.contains("dx=-13 dy=-21")),
            "native projectile damage diagnostics must preserve signed projectile-minus-Kid deltas"
        );
    }

    #[test]
    fn projectile_misses_player_far_left_asymmetric_offset() {
        let mut rt = runtime_with_projectile_and_player(0, 100, 100, 100);
        let slice = rt.opening_slice.unwrap();

        assert!(
            !rt.projectile_hits_player(&slice),
            "generic projectile overlap uses signed subtraction before abs; a projectile 100px left of Kid is outside the ASM-sized hitbox"
        );
    }

    #[test]
    fn projectile_misses_player_far_above_asymmetric_offset() {
        let mut rt = runtime_with_projectile_and_player(100, 0, 100, 100);
        let slice = rt.opening_slice.unwrap();

        assert!(
            !rt.projectile_hits_player(&slice),
            "generic projectile overlap uses signed subtraction before abs on Y as well as X"
        );
    }

    #[test]
    fn projectile_misses_when_x_overlap_but_y_outside() {
        // x overlap (same x) but y is 22px away → miss.
        // Kills: && with || replacement.
        let mut rt = runtime_with_projectile_and_player(100, 122, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            !rt.projectile_hits_player(&slice),
            "x overlap but y outside should miss (both conditions required)"
        );
    }

    #[test]
    fn projectile_friendly_does_not_hit_player() {
        // Friendly projectile at player position — must not hit.
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(100, 100);
        proj.friendly = true;
        rt.projectiles.push(proj);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);
        assert!(
            !rt.projectile_hits_player(&slice),
            "friendly projectile must not damage player"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "friendly projectile should survive"
        );
    }

    #[test]
    fn drips_surface_crawler_startup_does_not_hit_player_loc_335d8() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0x0000_3000,
            y_velocity: 0,
            lifetime: u16::MAX,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::StartupLoc335D8,
            ..Default::default()
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);

        assert!(
            !rt.projectile_hits_player(&slice),
            "ASM Drips.asm loc_335D8 runs `stru_33FA6` before `st is_moved(a3)` and loc_33650 collision, so startup crawlers must not damage Kid"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "the loc_335D8 startup child is visible and should survive until its startup animation completes"
        );
        assert!(
            rt.last_projectile_damage_probe.is_none(),
            "startup crawlers should not report a generic projectile damage candidate"
        );
    }

    #[test]
    fn drips_surface_crawler_post_startup_hibernate_does_not_hit_same_frame_loc_33650() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0,
            y_velocity: 0,
            lifetime: u16::MAX,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::StartupLoc335D8,
            animation_elapsed_frames: 1,
            ..Default::default()
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].drips_surface_crawler_state,
            DripsSurfaceCrawlerState::HorizontalLoc33650,
            "ASM Drips.asm loc_335D8 tails through loc_33630/loc_33526 before entering loc_33650"
        );
        assert!(
            !rt.projectile_hits_player(&slice),
            "ASM Drips.asm loc_33650 starts with Hibernate_Object_1Frame before loc_33550, so the frame that finishes loc_335D8 startup must not damage Kid"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "the post-startup loc_33650 hibernate frame should keep the visible crawler alive"
        );
        assert!(
            rt.last_projectile_damage_probe.is_none(),
            "the hibernate boundary should not report a stale crawler damage candidate"
        );

        rt.step_projectiles();

        assert!(
            rt.projectile_hits_player(&slice),
            "after loc_33650's first Hibernate_Object_1Frame completes, the crawler reaches loc_33550 and can damage Kid"
        );
    }

    #[test]
    fn drips_surface_crawler_live_contact_hits_at_issue_260_boundary_loc_33550() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 585 << 16,
            y_pos: 101 << 16,
            x_velocity: 0x0000_3000,
            y_velocity: 0,
            lifetime: u16::MAX,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::HorizontalLoc33650,
            ..Default::default()
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 600 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);

        assert!(
            rt.projectile_hits_player(&slice),
            "ASM Drips.asm loc_33650 calls loc_33550 after GfxObjects_CollisionKid; issue-260 oracle frame 550 damages Kid with the live crawler at dx=-15, one pixel outside the generic hostile projectile window"
        );
        assert!(
            rt.projectiles.is_empty(),
            "ASM loc_33550 contact routes the crawler to loc_338F8, so native should consume the live crawler damage source"
        );
        assert!(
            rt.last_projectile_damage_probe
                .as_deref()
                .is_some_and(|probe| probe.contains("dx=-15 dy=1")),
            "the issue-260 regression should preserve the boundary geometry in diagnostics"
        );
    }

    #[test]
    fn drips_surface_crawler_live_contact_rejects_outside_loc_33550_boundary() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawler,
            x_pos: 584 << 16,
            y_pos: 101 << 16,
            x_velocity: 0x0000_3000,
            y_velocity: 0,
            lifetime: u16::MAX,
            friendly: false,
            drips_surface_crawler_state: DripsSurfaceCrawlerState::HorizontalLoc33650,
            ..Default::default()
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 600 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);

        assert!(
            !rt.projectile_hits_player(&slice),
            "ASM Drips.asm loc_33650/loc_33550 keeps the issue-260 crawler contact at |dx|=15 inclusive, but |dx|=16 is outside the GfxObjects_CollisionKid contact"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "a non-contacting loc_33550 crawler should stay live for the terrain/support checks"
        );
    }

    #[test]
    fn drips_surface_crawler_burst_helper_does_not_hit_player_loc_338f8() {
        let mut rt = NativeRuntime::new(manifest());
        rt.projectiles.push(EnemyProjectile {
            kind: EnemyProjectileKind::DripsSurfaceCrawlerBurst,
            x_pos: 100 << 16,
            y_pos: 100 << 16,
            x_velocity: 0,
            y_velocity: 0,
            lifetime: 60,
            friendly: false,
            ..Default::default()
        });
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 100 << 16;
        slice.player.y_pos = 100 << 16;
        rt.opening_slice = Some(slice);

        assert!(
            !rt.projectile_hits_player(&slice),
            "ASM Drips.asm loc_338F8 allocates Tar_Monster.asm loc_325B6 after crawler contact/terrain exit; the helper animation must not be a second generic Kid-hit source"
        );
        assert_eq!(
            rt.projectiles.len(),
            1,
            "ASM loc_3392C keeps the helper animation alive until its own lifecycle deletes it"
        );
    }

    #[test]
    fn kid_projectile_collision_passes_sub_1d76_dispatches_the_two_rom_list_heads() {
        let passes = kid_projectile_collision_passes_sub_1D76();

        assert_eq!(
            passes,
            [
                KidProjectileCollisionPassSub1D76 {
                    object_list_head_address: 0x00FF_F862,
                    collision_mode: 0,
                },
                KidProjectileCollisionPassSub1D76 {
                    object_list_head_address: 0x00FF_F5A0,
                    collision_mode: 1,
                },
            ],
            "ASM sub_1D76 should call sub_1D84 twice: once for `$FFFFF862` with `d7=0`, then for `$FFFFF5A0` with `d7=1`",
        );
    }

    #[test]
    fn kid_projectile_target_x_span_loc_1db4_mirrors_the_leading_offset_when_facing_flipped() {
        assert_eq!(
            crate::projectiles::kid_projectile_target_x_span_loc_1DB4(100, 0, 6, 20),
            crate::projectiles::KidProjectileTargetXSpanLoc1DB4 {
                left_px: 106,
                right_px: 126,
            },
            "ASM sub_1D84 should use `[x_pos + leading, x_pos + leading + span]` when `x_direction(a0)` is clear before reaching `loc_1DB4`",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_target_x_span_loc_1DB4(100, 0xFF, 6, 20),
            crate::projectiles::KidProjectileTargetXSpanLoc1DB4 {
                left_px: 74,
                right_px: 94,
            },
            "ASM loc_1DB4 should negate the leading offset, anchor the right edge at `x_pos - leading`, and subtract the span for flipped targets",
        );
    }

    #[test]
    fn kid_projectile_target_rect_loc_1dbe_adds_the_shared_y_span_after_the_x_branch() {
        assert_eq!(
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0, 6, 20, -12, 30),
            crate::projectiles::KidProjectileTargetRectLoc1DBE {
                left_px: 106,
                right_px: 126,
                top_px: 28,
                bottom_px: 58,
            },
            "ASM loc_1DBE should add the leading Y offset to `y_pos(a0)`, copy that value, and add the final Y span after the non-flipped X branch",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0xFF, 6, 20, -12, 30),
            crate::projectiles::KidProjectileTargetRectLoc1DBE {
                left_px: 74,
                right_px: 94,
                top_px: 28,
                bottom_px: 58,
            },
            "ASM loc_1DBE should keep the same `[y_pos + offset, y_pos + offset + span]` range after the flipped `loc_1DB4` X branch",
        );
    }

    #[test]
    fn first_live_kid_projectile_collision_cursor_loc_1dcc_skips_deleted_entries_and_halves_the_collision_word()
     {
        assert_eq!(
            first_live_kid_projectile_collision_cursor_loc_1DCC(&[
                kid_projectile_collision_object_loc_1dcc(
                    true,
                    false,
                    0x0002,
                    32,
                    48,
                    vec![collision_size_entry_sub_21f8(-2, 4, -2, 4)],
                ),
                kid_projectile_collision_object_loc_1dcc(
                    false,
                    true,
                    0x0006,
                    120,
                    80,
                    vec![
                        collision_size_entry_sub_21f8(-5, 10, -31, 31),
                        collision_size_entry_sub_21f8(0, 0, 0, 0),
                    ],
                ),
            ]),
            Some(KidProjectileCollisionCursorLoc1DCC {
                object_index: 1,
                collision_size_index: 3,
                mirrored: true,
                x_anchor_px: 120,
                y_anchor_px: 80,
                collision_sizes: vec![
                    collision_size_entry_sub_21f8(-5, 10, -31, 31),
                    collision_size_entry_sub_21f8(0, 0, 0, 0),
                ],
            }),
            "ASM loc_1DCC should skip deleted gfx objects via `$3D(a2)`, then seed the CollisionSize cursor from `$22(a2)>>1` before the orientation-specific scan starts",
        );
    }

    #[test]
    fn first_non_mirrored_kid_projectile_overlap_loc_1df2_skips_misses_until_the_first_inclusive_overlap()
     {
        let target =
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0, 6, 20, -12, 30);
        let cursor = KidProjectileCollisionCursorLoc1DCC {
            object_index: 0,
            collision_size_index: 0,
            mirrored: false,
            x_anchor_px: 100,
            y_anchor_px: 40,
            collision_sizes: vec![
                collision_size_entry_sub_21f8(40, 10, -12, 30),
                collision_size_entry_sub_21f8(-20, 5, -12, 30),
                collision_size_entry_sub_21f8(6, 20, -12, 30),
            ],
        };

        assert_eq!(
            first_non_mirrored_kid_projectile_overlap_loc_1DF2(target, &cursor),
            Some(2),
            "ASM loc_1DF2 should keep advancing 8-byte CollisionSize entries after X/Y misses, then stop on the first inclusive overlap for clear `x_direction(a2)`",
        );
    }

    #[test]
    fn first_non_mirrored_kid_projectile_overlap_loc_1df2_stops_this_object_on_the_zero_word_sentinel()
     {
        let target =
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0, 6, 20, -12, 30);
        let cursor = KidProjectileCollisionCursorLoc1DCC {
            object_index: 0,
            collision_size_index: 0,
            mirrored: false,
            x_anchor_px: 100,
            y_anchor_px: 40,
            collision_sizes: vec![
                collision_size_entry_sub_21f8(40, 10, -12, 30),
                collision_size_entry_sub_21f8(0, 0, 0, 0),
                collision_size_entry_sub_21f8(6, 20, -12, 30),
            ],
        };

        assert_eq!(
            first_non_mirrored_kid_projectile_overlap_loc_1DF2(target, &cursor),
            None,
            "ASM loc_1DF2 should branch back to `loc_1DCC` as soon as `(a3)` is zero, even if a later CollisionSize entry in RAM would overlap",
        );
    }

    #[test]
    fn first_mirrored_kid_projectile_overlap_loc_1e1e_negates_the_left_offset_then_subtracts_width()
    {
        let target =
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0xFF, 6, 20, -12, 30);
        let cursor = KidProjectileCollisionCursorLoc1DCC {
            object_index: 0,
            collision_size_index: 0,
            mirrored: true,
            x_anchor_px: 100,
            y_anchor_px: 40,
            collision_sizes: vec![
                collision_size_entry_sub_21f8(40, 10, -12, 30),
                collision_size_entry_sub_21f8(-6, 20, -12, 30),
            ],
        };

        assert_eq!(
            first_mirrored_kid_projectile_overlap_loc_1E1E(target, &cursor),
            Some(1),
            "ASM loc_1E1E should mirror the horizontal bounds by negating the stored left offset, then subtract the width back toward the facing side before the inclusive overlap checks",
        );
    }

    #[test]
    fn first_mirrored_kid_projectile_overlap_loc_1e1e_stops_this_object_on_the_zero_word_sentinel()
    {
        let target =
            crate::projectiles::kid_projectile_target_rect_loc_1DBE(100, 40, 0xFF, 6, 20, -12, 30);
        let cursor = KidProjectileCollisionCursorLoc1DCC {
            object_index: 0,
            collision_size_index: 0,
            mirrored: true,
            x_anchor_px: 100,
            y_anchor_px: 40,
            collision_sizes: vec![
                collision_size_entry_sub_21f8(40, 10, -12, 30),
                collision_size_entry_sub_21f8(0, 0, 0, 0),
                collision_size_entry_sub_21f8(-6, 20, -12, 30),
            ],
        };

        assert_eq!(
            first_mirrored_kid_projectile_overlap_loc_1E1E(target, &cursor),
            None,
            "ASM loc_1E1E should branch back to `loc_1DCC` on the zero-word CollisionSize sentinel before any later mirrored entry is considered",
        );
    }

    #[test]
    fn kid_projectile_overlap_branch_loc_1e4a_routes_nonzero_d7_straight_to_wall_response() {
        assert_eq!(
            crate::projectiles::kid_projectile_overlap_branch_loc_1E4A(1, 0x1234, 0x2000, 0x2010),
            crate::projectiles::KidProjectileOverlapBranchLoc1E4A::WallResponseLoc1EA0,
            "ASM loc_1E4A (kid.asm:3634-3635) should jump to `loc_1EA0` as soon as `tst.w d7` sees a non-zero collision pass",
        );
    }

    #[test]
    fn kid_projectile_overlap_branch_loc_1e4a_only_diverts_the_inclusive_null_window() {
        const LNK_TO3_NULL_MINUS_DATA_INDEX: u16 = 0x2000;
        const LNK_TO4_NULL_MINUS_DATA_INDEX: u16 = 0x2010;

        assert_eq!(
            crate::projectiles::kid_projectile_overlap_branch_loc_1E4A(
                0,
                LNK_TO3_NULL_MINUS_DATA_INDEX.wrapping_sub(2),
                LNK_TO3_NULL_MINUS_DATA_INDEX,
                LNK_TO4_NULL_MINUS_DATA_INDEX,
            ),
            crate::projectiles::KidProjectileOverlapBranchLoc1E4A::SpawnHelperLoc1E5E,
            "ASM loc_1E4A should fall through to `loc_1E5E` when `$22(a0)` stays below `LnkTo3_NULL-Data_Index`",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_overlap_branch_loc_1E4A(
                0,
                LNK_TO3_NULL_MINUS_DATA_INDEX,
                LNK_TO3_NULL_MINUS_DATA_INDEX,
                LNK_TO4_NULL_MINUS_DATA_INDEX,
            ),
            crate::projectiles::KidProjectileOverlapBranchLoc1E4A::WallResponseLoc1EA0,
            "ASM loc_1E4A's `ble.s loc_1EA0` means the lower bound is inclusive",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_overlap_branch_loc_1E4A(
                0,
                LNK_TO4_NULL_MINUS_DATA_INDEX,
                LNK_TO3_NULL_MINUS_DATA_INDEX,
                LNK_TO4_NULL_MINUS_DATA_INDEX,
            ),
            crate::projectiles::KidProjectileOverlapBranchLoc1E4A::WallResponseLoc1EA0,
            "ASM loc_1E4A should keep the upper bound inclusive before leaving for `loc_1EA0`",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_overlap_branch_loc_1E4A(
                0,
                LNK_TO4_NULL_MINUS_DATA_INDEX.wrapping_add(2),
                LNK_TO3_NULL_MINUS_DATA_INDEX,
                LNK_TO4_NULL_MINUS_DATA_INDEX,
            ),
            crate::projectiles::KidProjectileOverlapBranchLoc1E4A::SpawnHelperLoc1E5E,
            "ASM loc_1E4A should resume the `loc_1E5E` helper-allocation path once `$22(a0)` rises above `LnkTo4_NULL-Data_Index`",
        );
    }

    #[test]
    fn kid_projectile_dp_collision_word_loc_1ea0_uses_object_meta_low_byte_table() {
        assert_eq!(
            crate::projectiles::kid_projectile_dp_collision_word_loc_1EA0(0x00),
            0x001C,
            "ASM loc_1EA0 should map `unk_1EBE[0]` to `colid_hurt` for Diamond Power list collisions",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_dp_collision_word_loc_1EA0(0x02),
            0x0000,
            "ASM loc_1EA0 should preserve the explicit zero collision word at `unk_1EBE[2]`",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_dp_collision_word_loc_1EA0(0x0D),
            0x002C,
            "ASM loc_1EA0 should map DragonWalking's `$3A & $FF == $0D` through `unk_1EBE` to `colid_kidabove`",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_dp_collision_word_loc_1EA0(0x20),
            0xFFFF,
            "ASM loc_1EA0 should expose the terminal `$FFFF` table entries used by high object metadata values",
        );
        assert_eq!(
            crate::projectiles::kid_projectile_dp_collision_word_loc_1EA0(0xFF),
            0xFFFF,
            "Native code treats out-of-table values defensively as `$FFFF`; the ROM table is only defined for known `$3A & $FF` values",
        );
    }

    #[test]
    fn wall_of_death_slot_collision_queues_dragon_kidabove_loc_1d84_1ea0() {
        let mut rt = NativeRuntime::new(manifest());
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.camera.y_pos = 0x0073_0000;
        rt.opening_slice = Some(slice);
        rt.progression.diamond_power_type = crate::helmets::DiamondPowerType::WallOfDeath;
        rt.progression.diamond_power_frames = 100;
        rt.diamond_power_wall_slots = vec![crate::helmets::WallOfDeathSlot {
            x_offset: 0x0091,
            y_offset: 0x0091,
            reached_right_edge: false,
            size_index: 6,
        }];
        rt.active_enemies.push(test_enemy(
            EnemyFamily::DragonWalking,
            0x009A_0000,
            0x0110_0000,
            1,
        ));

        rt.step_diamond_power(FrameInput::default());
        assert_eq!(
            rt.active_enemies[0].hit_points, 1,
            "ASM loc_1EA0 writes the collision word during the DP-object scan; the enemy family consumes it on the next object pass",
        );

        rt.apply_pending_enemy_collisions();

        assert_eq!(
            rt.active_enemies.len(),
            1,
            "ASM Dragon.asm loc_3CEFC enters the death animation instead of deleting the live object immediately",
        );
        assert_eq!(
            rt.active_enemies[0].hit_points, 0,
            "ASM loc_1EA0 should map DragonWalking `$3A & $FF == $0D` to `$002C`, then Dragon.asm loc_3CE9A/loc_3CEBE should consume it as damage",
        );
        assert!(rt.active_enemies[0].death_animation_active);
    }

    #[test]
    fn kid_projectile_helper_spawn_loc_1e5e_allocates_loc_1f08_and_copies_projectile_coordinates() {
        assert_eq!(
            crate::projectiles::kid_projectile_helper_spawn_loc_1E5E(0x0123, 0x0456, 0x0000),
            crate::projectiles::KidProjectileHelperSpawnLoc1E5E {
                allocate_object_slot_arg: 0xFFFF,
                callback_label: "loc_1F08",
                helper_x_pos: 0x0123,
                helper_y_pos: 0x0456,
                target_collision_id: 0x001C,
            },
            "ASM loc_1E5E (kid.asm:3642-3650) should allocate with `a0 = $FFFF`, install `loc_1F08`, copy `$1A/$1E(a0)` into the helper payload, and default the collided object to `colid_hurt`",
        );
    }

    #[test]
    fn kid_projectile_helper_spawn_loc_1e5e_overrides_hurt_with_ffff_for_3c_projectiles() {
        assert_eq!(
            crate::projectiles::kid_projectile_helper_spawn_loc_1E5E(0x0123, 0x0456, 0x003C)
                .target_collision_id,
            0xFFFF,
            "ASM loc_1E5E should replace the provisional `colid_hurt` write with `$FFFF` when `$3A(a0) == $3C` before falling into `loc_1E94`",
        );
    }

    #[test]
    fn kid_projectile_rightwall_writeback_loc_1e94_marks_the_projectile_and_restarts_the_scan() {
        assert_eq!(
            crate::projectiles::kid_projectile_rightwall_writeback_loc_1E94(),
            crate::projectiles::KidProjectileRightwallWritebackLoc1E94 {
                projectile_collision_id: 0x0004,
                next_label: "sub_1D84",
            },
            "ASM loc_1E94 (kid.asm:3655-3656) should store `#4` into `$38(a0)` before branching back to `sub_1D84`",
        );
    }

    #[test]
    fn projectile_hitbox_half_plus_8_not_times() {
        // Projectile 12px away in x — inside `half + 8 = 14` but outside `half * 8 = 48`.
        // If + mutated to *, 12 < 48 would still pass, so use boundary at 13.
        // Also check that 15px misses — kills + with * (6*8=48 would wrongly pass).
        let mut rt = runtime_with_projectile_and_player(115, 100, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            !rt.projectile_hits_player(&slice),
            "15px x gap should miss with half + 8 = 14"
        );
    }

    #[test]
    fn projectile_hitbox_half_plus_16_not_times() {
        // Projectile 23px away in y — outside `half + 16 = 22`.
        // If + mutated to *, 23 < 96 would wrongly pass.
        let mut rt = runtime_with_projectile_and_player(100, 123, 100, 100);
        let slice = rt.opening_slice.unwrap();
        assert!(
            !rt.projectile_hits_player(&slice),
            "23px y gap should miss with half + 16 = 22"
        );
    }

    #[test]
    fn step_projectiles_applies_velocity() {
        // Basic movement test calling actual step_projectiles.
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(100, 100);
        proj.x_velocity = 0x2_0000; // +2px/frame
        proj.y_velocity = -0x1_0000; // -1px/frame
        rt.projectiles.push(proj);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].x_pos,
            102 << 16,
            "x should advance by x_velocity"
        );
        assert_eq!(
            rt.projectiles[0].y_pos,
            99 << 16,
            "y should advance by y_velocity"
        );
    }

    #[test]
    fn step_projectiles_wall_bounce_reapplies_velocity() {
        // After a wall bounce, position = pre-bounce + bounced velocity.
        // Kills: line 126 + with *, line 127 + with - and + with *.
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(50, 50);
        proj.x_velocity = 0x2_0000; // 2px/frame rightward
        proj.y_velocity = 0x1_0000; // 1px/frame downward
        proj.wall_bounce = true;

        // Build a grid with a solid block at the destination cell.
        // Projectile at (50,50), moving right 2px → lands at (52,51).
        // Place solid block at cell containing (52,51).
        let w = 10u16;
        let h = 10u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Solid block at col=3 (pixels 48-63), row=3 (pixels 48-63)
        let dest_col = terrain::pixel_to_cell(52).max(0) as u16;
        let dest_row = terrain::pixel_to_cell(51).max(0) as u16;
        let off = (usize::from(dest_row) * usize::from(w) + usize::from(dest_col)) * 2;
        data[off] = 0x60; // has_block=0, collision=Solid(6), block_type=0
        // Actually need has_block=true: hi=0xE0 (has_block=1, collision=6(Solid), block_type=0)
        data[off] = 0xE0;
        let grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        rt.set_block_grid(grid);

        let x_before = proj.x_pos;
        let y_before = proj.y_pos;
        rt.projectiles.push(proj);
        rt.step_projectiles();

        // After bounce: velocity should be negated on the blocked axis, then re-applied.
        // The exact outcome depends on which axis is blocked, but position should differ
        // from a simple velocity addition (which would be inside the wall).
        let p = &rt.projectiles[0];
        // The projectile should have moved (not stayed in place)
        assert!(
            p.x_pos != x_before || p.y_pos != y_before,
            "wall-bounce projectile should move after bouncing"
        );
    }

    #[test]
    fn step_projectiles_gravity_accumulates() {
        // Gravity adds to y_velocity each frame.
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(100, 100);
        proj.y_velocity = 0;
        proj.gravity = 0x1000;
        rt.projectiles.push(proj);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].y_velocity, 0x1000,
            "gravity should be added to y_velocity"
        );

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].y_velocity, 0x2000,
            "gravity should accumulate each frame"
        );
    }

    #[test]
    fn loc_3263c_burst_helpers_apply_their_asm_gravity_order() {
        for (kind, expected_y_pos) in [
            (
                EnemyProjectileKind::TarMonsterBurstChild,
                (102 << 16) + 0x1000,
            ),
            (EnemyProjectileKind::DripsSurfaceCrawlerBurst, 102 << 16),
        ] {
            let mut rt = NativeRuntime::new(manifest());
            let mut proj = make_projectile(100, 100);
            proj.kind = kind;
            proj.y_velocity = 0x0002_0000;
            proj.gravity = 0x1000;
            rt.projectiles.push(proj);

            rt.step_projectiles();

            let moved = &rt.projectiles[0];
            assert_eq!(
                moved.y_pos, expected_y_pos,
                "ASM Tar_Monster.asm loc_3263C resumes with `$1000` feeding Tar-child motion; Drips keeps the shared post-move gravity order for {kind:?}"
            );
            assert_eq!(
                moved.y_velocity, 0x0002_1000,
                "ASM Tar_Monster.asm loc_3263C adds gravity after the movement step for {kind:?}"
            );
        }
    }

    #[test]
    fn step_projectiles_lifetime_decrements() {
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(100, 100);
        proj.lifetime = 5;
        rt.projectiles.push(proj);

        rt.step_projectiles();
        assert_eq!(rt.projectiles[0].lifetime, 4);

        // Step to expiry: 4→3→2→1→0, then next call sees 0 and removes
        for _ in 0..4 {
            rt.step_projectiles();
        }
        assert_eq!(rt.projectiles[0].lifetime, 0, "lifetime should reach 0");
        rt.step_projectiles();
        assert!(
            rt.projectiles.is_empty(),
            "projectile should be removed when lifetime is 0 at frame start"
        );
    }

    #[test]
    fn fireball_trail_lifetime_uses_loc_33110_counter() {
        let mut rt = NativeRuntime::new(manifest());
        let mut expiring = make_projectile(100, 100);
        expiring.kind = EnemyProjectileKind::FireballTrail;
        expiring.lifetime = 1;
        expiring.x_velocity = 0x0001_0000;
        rt.projectiles.push(expiring);

        rt.step_projectiles();

        assert!(
            rt.projectiles.is_empty(),
            "ASM Fireball.asm loc_33110 hibernates/moves, decrements d3 to zero, then falls through to loc_33120 without a generic zero-lifetime frame"
        );

        let mut wrapped = make_projectile(100, 100);
        wrapped.kind = EnemyProjectileKind::FireballTrail;
        wrapped.lifetime = 0;
        rt.projectiles.push(wrapped);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].lifetime, 0xFFFF,
            "ASM Fireball.asm loc_33110 uses word subtraction, so an unexpected d3=0 wraps and keeps the trail alive"
        );
    }

    #[test]
    fn fireball_obstacle_trail_lifetime_uses_loc_331ba_counter() {
        let mut rt = NativeRuntime::new(manifest());
        let mut expiring = make_projectile(100, 100);
        expiring.kind = EnemyProjectileKind::FireballObstacleTrail;
        expiring.lifetime = 1;
        expiring.x_velocity = 0x0001_0000;
        rt.projectiles.push(expiring);

        rt.step_projectiles();

        assert!(
            rt.projectiles.is_empty(),
            "ASM Fireball.asm loc_331BA hibernates/moves, decrements d3 to zero, then falls through to loc_331CA without a generic zero-lifetime frame"
        );

        let mut wrapped = make_projectile(100, 100);
        wrapped.kind = EnemyProjectileKind::FireballObstacleTrail;
        wrapped.lifetime = 0;
        rt.projectiles.push(wrapped);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].lifetime, 0xFFFF,
            "ASM Fireball.asm loc_331BA uses word subtraction, so an unexpected d3=0 wraps and keeps the trail alive"
        );
    }

    #[test]
    fn step_projectiles_homing_adjusts_velocity() {
        // Homing projectile accelerates toward player.
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(50, 50);
        proj.homing_accel = 0x800;
        proj.speed_cap = 0x6_0000;
        rt.projectiles.push(proj);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 200 << 16; // player far right
        slice.player.y_pos = 200 << 16; // player far below
        rt.opening_slice = Some(slice);

        rt.step_projectiles();

        assert!(
            rt.projectiles[0].x_velocity > 0,
            "homing should accelerate rightward toward player"
        );
        assert!(
            rt.projectiles[0].y_velocity > 0,
            "homing should accelerate downward toward player"
        );
    }

    #[test]
    fn step_projectiles_speed_cap_clamps_homing() {
        let mut rt = NativeRuntime::new(manifest());
        let mut proj = make_projectile(50, 50);
        proj.homing_accel = 0x7_0000; // very high accel
        proj.speed_cap = 0x2_0000; // low cap
        rt.projectiles.push(proj);
        let mut slice = OpeningSliceState::new(default_bounds());
        slice.player.x_pos = 200 << 16;
        slice.player.y_pos = 200 << 16;
        rt.opening_slice = Some(slice);

        rt.step_projectiles();

        assert_eq!(
            rt.projectiles[0].x_velocity, 0x2_0000,
            "x velocity should be clamped to speed_cap"
        );
        assert_eq!(
            rt.projectiles[0].y_velocity, 0x2_0000,
            "y velocity should be clamped to speed_cap"
        );
    }
}
