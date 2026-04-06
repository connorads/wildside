use super::*;
use crate::combat::CollisionRectSub1F52;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeBlockHeadbuttWorkerLoc1002E {
    pub(crate) sound_effect_label: &'static str,
    pub(crate) allocation_spec: u16,
    pub(crate) object_handler_addr: u32,
    pub(crate) block_col: u16,
    pub(crate) block_row: u16,
    pub(crate) layout_word_addr: u32,
    pub(crate) gfx_x_pos: u16,
    pub(crate) gfx_y_pos: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeBlockDeletedTileLoc101D8 {
    pub(crate) written_cell: terrain::Cell,
    pub(crate) fg_queue_entry: (u16, u16, u16),
    pub(crate) child_display_enabled: bool,
    pub(crate) sound_effect_label: &'static str,
    pub(crate) animation_script_addr: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeBlockSpawnFlashLoc102B8 {
    pub(crate) display_enabled: bool,
    pub(crate) palette_line: u8,
    pub(crate) addroffset_sprite: u16,
    pub(crate) animation_script_addr: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeClockSoundRefcountLoc10792 {
    pub(crate) should_play_sound: bool,
    pub(crate) next_refcount: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeClockCountdownLoc10796 {
    pub(crate) next_remaining: u16,
    pub(crate) should_delete: bool,
    pub(crate) should_swap_animation: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeClockFallGateLoc107B0 {
    pub(crate) stays_active: bool,
    pub(crate) hit_level_bottom: bool,
    pub(crate) collided_with_player: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeContinueCountdownLoc1080A {
    pub(crate) next_remaining: u16,
    pub(crate) should_delete: bool,
    pub(crate) should_swap_animation: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeContinueDeleteLoc10838 {
    pub(crate) sound_effect_id: u8,
    pub(crate) preserves_d0_across_sound_call: bool,
    pub(crate) should_delete_current_object: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FloatingPrizeFallStepLoc10C50 {
    pub(crate) next_y_velocity: i32,
    pub(crate) next_y_pos: i64,
    pub(crate) next_aligned_y: i32,
    pub(crate) next_first_row_grace: bool,
    pub(crate) next_level_row: u16,
    pub(crate) next_level_layout_offset: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OneTimePrizeInitLoc10856 {
    pub(crate) was_first_prize_initialisation: bool,
    pub(crate) next_write_index: u16,
    pub(crate) entries: [u32; 0x28],
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OneTimePrizeScanStartLoc1086A {
    pub(crate) remaining_entry_checks: u16,
    pub(crate) prize_layout_addr: u16,
    pub(crate) entry_byte_offset_from_table_base: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OneTimePrizeEntryCheckLoc10876 {
    pub(crate) entry_layout_addr: u16,
    pub(crate) entry_level_id: u16,
    pub(crate) layout_addr_matches: bool,
    pub(crate) level_id_matches: bool,
    pub(crate) should_init_prize_diamond: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OneTimePrizeScanStepLoc10886 {
    pub(crate) next_remaining_entry_checks: u16,
    pub(crate) next_entry_byte_offset_from_table_base: u16,
    pub(crate) should_continue_scan: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OneTimePrizeStoreLoc1089A {
    pub(crate) next_write_index: u16,
    pub(crate) entries: [u32; 0x28],
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10DiamondsInitLoc108E8 {
    pub(crate) cleared_worker_slots: [u32; 10],
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10DiamondsSpawnLoc108EE {
    pub(crate) should_hibernate_one_frame: bool,
    pub(crate) should_scan_workers: bool,
    pub(crate) should_spawn_worker: bool,
    pub(crate) burst_table_index: Option<u8>,
    pub(crate) allocation_spec: Option<u32>,
    pub(crate) sound_effect_label: Option<&'static str>,
    pub(crate) priority: Option<u8>,
    pub(crate) display_enabled: Option<bool>,
    pub(crate) animation_script_label: Option<&'static str>,
    pub(crate) x_pos: Option<u16>,
    pub(crate) y_pos: Option<u16>,
    pub(crate) x_velocity: Option<i32>,
    pub(crate) y_velocity: Option<i32>,
    pub(crate) is_moved: Option<bool>,
    pub(crate) lifetime_frames: Option<u16>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10DiamondsWorkerScanStartLoc10960 {
    pub(crate) first_worker_slot_byte_offset: u16,
    pub(crate) worker_slot_stride_bytes: u16,
    pub(crate) worker_table_len_bytes: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10DiamondsWorkerCountdownGateLoc10984 {
    pub(crate) should_clear_display_enabled: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10DiamondsWorkerInitLoc10988 {
    pub(crate) camera_relative_x_pos: i32,
    pub(crate) camera_relative_y_pos: i32,
    pub(crate) x_velocity: i32,
    pub(crate) y_velocity: i32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Prize10DiamondsWorkerStepLoc109C0 {
    pub(crate) next_camera_relative_x_pos: i32,
    pub(crate) next_camera_relative_y_pos: i32,
    pub(crate) next_x_pos: u16,
    pub(crate) next_y_pos: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeClockSoundCleanupLoc107E0 {
    pub(crate) next_refcount: u8,
    pub(crate) should_play_stop_sound: bool,
    pub(crate) should_delete_current_object: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A: [i16; 31] = [
    -2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, 2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, -2, -2, -2, -2, 2,
    2, 2, 2, 2,
];

impl NativeRuntime {
    #[allow(non_snake_case)]
    pub(crate) fn lift_platform_vertical_step_sub_FD00(y_pos: u32, y_velocity: i32) -> u32 {
        // ASM sub_FD00 (kid.asm:21064): `move.l $E(a3),d7; add.l d7,6(a3)`.
        y_pos.wrapping_add_signed(y_velocity)
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_clock_sound_refcount_loc_10792(
        active_clock_prize_sounds: u8,
    ) -> PrizeClockSoundRefcountLoc10792 {
        // ASM PrizeClock_Init -> loc_10792 (kid.asm:21997-22005): only the
        // first active clock prize plays `sfx_Clock_prize`; every spawned
        // clock increments the shared `$FFFFFB55` refcount afterwards.
        PrizeClockSoundRefcountLoc10792 {
            should_play_sound: active_clock_prize_sounds == 0,
            next_refcount: active_clock_prize_sounds.wrapping_add(1),
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_clock_countdown_loc_10796(remaining: u16) -> PrizeClockCountdownLoc10796 {
        // ASM loc_10796 (kid.asm:22006-22014): hibernate one frame, subtract 1
        // from `d3`, delete immediately when it reaches zero, and swap to
        // `stru_10C1E` exactly on the `$012C` countdown edge.
        let next_remaining = remaining.saturating_sub(1);
        PrizeClockCountdownLoc10796 {
            next_remaining,
            should_delete: next_remaining == 0,
            should_swap_animation: next_remaining == 0x012C,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_clock_fall_gate_loc_107B0(
        integer_y: i64,
        level_height_pixels: i64,
        collected: bool,
    ) -> PrizeClockFallGateLoc107B0 {
        // ASM loc_107B0 (kid.asm:22015-22028): after `sub_10C38`, the clock
        // keeps looping only while `y_pos(a3)` stays strictly above
        // `Level_height_pixels` and `collision_type(a3)` remains zero.
        let hit_level_bottom = integer_y >= level_height_pixels;
        let collided_with_player = collected;
        PrizeClockFallGateLoc107B0 {
            stays_active: !hit_level_bottom && !collided_with_player,
            hit_level_bottom,
            collided_with_player,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_clock_sound_cleanup_loc_107E0(
        active_clock_prize_sounds: u8,
    ) -> PrizeClockSoundCleanupLoc107E0 {
        // ASM loc_107E0 (kid.asm:22028-22033): once the clock exits its fall
        // loop, decrement the shared `$FFFFFB55` refcount and play
        // `sfx_Clock_prize` through `PlaySound2` only when that decremented
        // count reaches zero.
        // ASM loc_107EE (kid.asm:22034): the routine then jumps straight to
        // `Delete_CurrentObject` regardless of whether the stop sound ran.
        let next_refcount = active_clock_prize_sounds.wrapping_sub(1);
        PrizeClockSoundCleanupLoc107E0 {
            next_refcount,
            should_play_stop_sound: next_refcount == 0,
            should_delete_current_object: true,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_continue_countdown_loc_1080A(
        remaining: u16,
    ) -> PrizeContinueCountdownLoc1080A {
        // ASM loc_1080A (kid.asm:22045-22053): the continue prize hibernates,
        // decrements `d3`, deletes once the result reaches zero, and swaps to
        // `stru_10C2E` exactly on the shared `$012C` countdown edge before
        // the later fall/collision gate resumes at loc_10824.
        let next_remaining = remaining.saturating_sub(1);
        PrizeContinueCountdownLoc1080A {
            next_remaining,
            should_delete: next_remaining == 0,
            should_swap_animation: next_remaining == 0x012C,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_continue_delete_loc_10838() -> PrizeContinueDeleteLoc10838 {
        // ASM loc_10838 (kid.asm:22062): preserve `d0`, call `j_PlaySound`
        // with `#$6A`, restore `d0`, then jump straight to
        // `Delete_CurrentObject`.
        PrizeContinueDeleteLoc10838 {
            sound_effect_id: 0x6A,
            preserves_d0_across_sound_call: true,
            should_delete_current_object: true,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn floating_prize_fall_step_loc_10C50(
        aligned_y: i32,
        y_pos: i64,
        y_velocity: i32,
        first_row_grace: bool,
        level_row: u16,
        level_layout_offset: u32,
        level_width_tiles: u16,
    ) -> FloatingPrizeFallStepLoc10C50 {
        // ASM loc_10C50 (kid.asm:22553): once the prechecks allow the fall,
        // add `#$1388` to `d1`, advance `y_pos(a3)`, snap the integer row with
        // `andi.w #$FFF0,d2`, and only when that snapped row differs from `d0`
        // clear bit `$10` in `d3` and advance `a2` by `Level_width_tiles`
        // before the shared tail at loc_10C74 stores the new aligned row back
        // into `d0`.
        let next_y_velocity = y_velocity.saturating_add(0x1388);
        let next_y_pos = y_pos.saturating_add(i64::from(next_y_velocity));
        let integer_y = (next_y_pos >> 16) as i32;
        let next_aligned_y = integer_y & !0xF;
        let crossed_row_boundary = next_aligned_y != aligned_y;

        FloatingPrizeFallStepLoc10C50 {
            next_y_velocity,
            next_y_pos,
            next_aligned_y,
            next_first_row_grace: if crossed_row_boundary {
                false
            } else {
                first_row_grace
            },
            next_level_row: if crossed_row_boundary {
                level_row.saturating_add(1)
            } else {
                level_row
            },
            next_level_layout_offset: if crossed_row_boundary {
                level_layout_offset.wrapping_add(u32::from(level_width_tiles))
            } else {
                level_layout_offset
            },
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn one_time_prize_init_loc_10856(
        header_word: i16,
        entries: [u32; 0x28],
    ) -> OneTimePrizeInitLoc10856 {
        // ASM loc_10856 (kid.asm:22080): if `(a0)` is negative, this is the
        // first one-time prize collected for the current player. Clear the
        // header and all forty packed `(layout_addr, level_id)` longwords
        // before the later scan restarts at loc_1086A.
        if header_word < 0 {
            OneTimePrizeInitLoc10856 {
                was_first_prize_initialisation: true,
                next_write_index: 0,
                entries: [0; 0x28],
            }
        } else {
            OneTimePrizeInitLoc10856 {
                was_first_prize_initialisation: false,
                next_write_index: header_word as u16,
                entries,
            }
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn one_time_prize_scan_start_loc_1086A(
        prize_layout_addr: u16,
    ) -> OneTimePrizeScanStartLoc1086A {
        // ASM loc_1086A (kid.asm:22092): reset `d0` to `#$27`, copy the
        // current block's layout address from `$4A(a5)` into `d1`, and start
        // scanning at `2(a0)` so the header word is skipped.
        OneTimePrizeScanStartLoc1086A {
            remaining_entry_checks: 0x27,
            prize_layout_addr,
            entry_byte_offset_from_table_base: 2,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn one_time_prize_entry_check_loc_10876(
        packed_entry: u32,
        prize_layout_addr: u16,
        current_level_id: u16,
    ) -> OneTimePrizeEntryCheckLoc10876 {
        // ASM loc_10876 (kid.asm:22098): compare `(a1)` against the current
        // prize layout address in `d1`; only on a layout match does the code
        // load `Current_LevelID` into `d2`, compare `2(a1)`, and branch to
        // `PrizeDiamond_Init` when both words match. Native packs each
        // `(layout_addr, level_id)` pair as `0xLLLL_VVVV`.
        let entry_layout_addr = (packed_entry >> 16) as u16;
        let entry_level_id = packed_entry as u16;
        let layout_addr_matches = entry_layout_addr == prize_layout_addr;
        let level_id_matches = layout_addr_matches && entry_level_id == current_level_id;
        OneTimePrizeEntryCheckLoc10876 {
            entry_layout_addr,
            entry_level_id,
            layout_addr_matches,
            level_id_matches,
            should_init_prize_diamond: level_id_matches,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn one_time_prize_scan_step_loc_10886(
        remaining_entry_checks: u16,
        entry_byte_offset_from_table_base: u16,
    ) -> OneTimePrizeScanStepLoc10886 {
        // ASM loc_10886 (kid.asm:22107): advance `a1` by four bytes to the
        // next packed entry, then `dbf d0,loc_10876` repeats until the
        // post-decrement counter underflows from zero to `$FFFF`.
        let next_remaining_entry_checks = remaining_entry_checks.wrapping_sub(1);
        OneTimePrizeScanStepLoc10886 {
            next_remaining_entry_checks,
            next_entry_byte_offset_from_table_base: entry_byte_offset_from_table_base
                .wrapping_add(4),
            should_continue_scan: next_remaining_entry_checks != u16::MAX,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn one_time_prize_store_loc_1089A(
        current_write_index: u16,
        entries: [u32; 0x28],
        prize_layout_addr: u16,
        current_level_id: u16,
    ) -> OneTimePrizeStoreLoc1089A {
        // ASM loc_1089A (kid.asm:22119): keep `d0` as the current header slot,
        // compute the next header value in `d1` with wrap at `$28`, store that
        // next value back into `(a0)`, then write the new `(layout_addr,
        // level_id)` pair into slot `d0`. The header therefore points to the
        // next insertion slot, not the one just written.
        let mut next_entries = entries;
        next_entries[usize::from(current_write_index)] =
            (u32::from(prize_layout_addr) << 16) | u32::from(current_level_id);
        let mut next_write_index = current_write_index.wrapping_add(1);
        if next_write_index >= 0x28 {
            next_write_index = 0;
        }
        OneTimePrizeStoreLoc1089A {
            next_write_index,
            entries: next_entries,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_10_diamonds_init_loc_108E8() -> Prize10DiamondsInitLoc108E8 {
        // ASM Prize10Diamonds_Init -> loc_108E8 (kid.asm:22170-22173): set
        // `a0 = $44(a5)`, seed `d0 = #9`, then `clr.l (a0)+` under DBF to
        // clear exactly ten longword worker slots before the per-frame loop
        // resumes at `loc_108EE`.
        Prize10DiamondsInitLoc108E8 {
            cleared_worker_slots: [0; 10],
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_10_diamonds_spawn_loc_108EE(
        frame_index: u16,
        spawn_x_pos: u16,
        spawn_y_pos: u16,
    ) -> Prize10DiamondsSpawnLoc108EE {
        // ASM loc_108EE (kid.asm:22174-22207): hibernate one frame, and for
        // the first `$1E` frames use `byte_108AC` to decide whether this
        // frame allocates a burst worker. Non-negative entries become an
        // index into `unk_10A52`, the worker copies the cached owner
        // position, plays `sfx_Diamond_prize`, starts with timer `$78`, and
        // seeds FP16.16 velocity as `(word << 16) >> 3`.
        const BURST_FRAME_TABLE: [i8; 0x1E] = [
            0, 1, 2, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, 4, 5, 6, 7, 8, 9,
        ];
        const BURST_VELOCITIES: [(i16, i16); 10] = [
            (-20, -36),
            (-20, -27),
            (-20, -18),
            (-20, -9),
            (6, -36),
            (-4, -28),
            (16, -28),
            (-4, -16),
            (16, -16),
            (6, -8),
        ];

        let Some(&burst_table_index) = BURST_FRAME_TABLE.get(usize::from(frame_index)) else {
            return Prize10DiamondsSpawnLoc108EE {
                should_hibernate_one_frame: true,
                should_scan_workers: true,
                should_spawn_worker: false,
                burst_table_index: None,
                allocation_spec: None,
                sound_effect_label: None,
                priority: None,
                display_enabled: None,
                animation_script_label: None,
                x_pos: None,
                y_pos: None,
                x_velocity: None,
                y_velocity: None,
                is_moved: None,
                lifetime_frames: None,
            };
        };

        if burst_table_index < 0 {
            return Prize10DiamondsSpawnLoc108EE {
                should_hibernate_one_frame: true,
                should_scan_workers: true,
                should_spawn_worker: false,
                burst_table_index: None,
                allocation_spec: None,
                sound_effect_label: None,
                priority: None,
                display_enabled: None,
                animation_script_label: None,
                x_pos: None,
                y_pos: None,
                x_velocity: None,
                y_velocity: None,
                is_moved: None,
                lifetime_frames: None,
            };
        }

        let burst_table_index = burst_table_index as u8;
        let (x_velocity_word, y_velocity_word) = BURST_VELOCITIES[usize::from(burst_table_index)];
        Prize10DiamondsSpawnLoc108EE {
            should_hibernate_one_frame: true,
            should_scan_workers: true,
            should_spawn_worker: true,
            burst_table_index: Some(burst_table_index),
            allocation_spec: Some(0x00FE_0000),
            sound_effect_label: Some("sfx_Diamond_prize"),
            priority: Some(1),
            display_enabled: Some(true),
            animation_script_label: Some("LnkTo_unk_E1046-Data_Index"),
            x_pos: Some(spawn_x_pos),
            y_pos: Some(spawn_y_pos),
            x_velocity: Some(i32::from(x_velocity_word) << 13),
            y_velocity: Some(i32::from(y_velocity_word) << 13),
            is_moved: Some(true),
            lifetime_frames: Some(0x0078),
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_10_diamonds_worker_scan_start_loc_10960()
    -> Prize10DiamondsWorkerScanStartLoc10960 {
        // ASM loc_10960 (kid.asm:22208) seeds `d2 = 0` before `loc_10962`
        // walks the ten longword worker slots at `$44(a5)` and `loc_10A3A`
        // advances the offset by four bytes until it reaches `$28`.
        Prize10DiamondsWorkerScanStartLoc10960 {
            first_worker_slot_byte_offset: 0,
            worker_slot_stride_bytes: 4,
            worker_table_len_bytes: 0x28,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_10_diamonds_worker_countdown_gate_loc_10984(
        lifetime_frames: u16,
    ) -> Prize10DiamondsWorkerCountdownGateLoc10984 {
        // ASM loc_10984 (kid.asm:22223-22227): the `$70` frame clears
        // `$14(a1)` with `sf` before branching into the shared countdown at
        // `loc_109E8`; every other timer value just falls straight through.
        Prize10DiamondsWorkerCountdownGateLoc10984 {
            should_clear_display_enabled: lifetime_frames == 0x0070,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_10_diamonds_worker_init_loc_10988(
        x_pos: i32,
        y_pos: i32,
        camera_x_pos: i32,
        camera_y_pos: i32,
    ) -> Prize10DiamondsWorkerInitLoc10988 {
        // ASM loc_10988 (kid.asm:22227-22242): cache camera-relative FP16.16
        // position in `$3E/$42`, then seed `$26/$2A` from
        // `(#$1260000 - rel_x) >> 5` and `(#$300000 - rel_y) >> 5`.
        let camera_relative_x_pos = x_pos.wrapping_sub(camera_x_pos);
        let camera_relative_y_pos = y_pos.wrapping_sub(camera_y_pos);
        Prize10DiamondsWorkerInitLoc10988 {
            camera_relative_x_pos,
            camera_relative_y_pos,
            x_velocity: (0x0126_0000_i32.wrapping_sub(camera_relative_x_pos)) >> 5,
            y_velocity: (0x0030_0000_i32.wrapping_sub(camera_relative_y_pos)) >> 5,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_10_diamonds_worker_step_loc_109C0(
        camera_relative_x_pos: i32,
        camera_relative_y_pos: i32,
        x_velocity: i32,
        y_velocity: i32,
        camera_x_integer: u16,
        camera_y_integer: u16,
    ) -> Prize10DiamondsWorkerStepLoc109C0 {
        // ASM loc_109C0 (kid.asm:22243-22256): advance `$3E/$42` by `$26/$2A`,
        // then rewrite `$1A/$1E` from the updated integer words plus the
        // camera's integer position. The low 16-bit sub-pixel fraction stays
        // only in the cached camera-relative words.
        let next_camera_relative_x_pos = camera_relative_x_pos.wrapping_add(x_velocity);
        let next_camera_relative_y_pos = camera_relative_y_pos.wrapping_add(y_velocity);
        Prize10DiamondsWorkerStepLoc109C0 {
            next_camera_relative_x_pos,
            next_camera_relative_y_pos,
            next_x_pos: (((next_camera_relative_x_pos >> 16) as i16)
                .wrapping_add(camera_x_integer as i16)) as u16,
            next_y_pos: (((next_camera_relative_y_pos >> 16) as i16)
                .wrapping_add(camera_y_integer as i16)) as u16,
        }
    }

    #[allow(non_snake_case)]
    pub(crate) fn step_lift_platform_sub_FCA6(
        lift: &mut LiftPlatform,
        level_height_pixels: u16,
    ) -> bool {
        // ASM sub_FCA6 (kid.asm:21021): hibernate one frame, call sub_FD00,
        // increment `d1` only when the integer `6(a3)` word changes, apply the
        // pre-/post-`$32` acceleration, then delete once `6(a3)` reaches the
        // signed level height.
        lift.age = lift.age.saturating_add(1);
        lift.y_pos = Self::lift_platform_vertical_step_sub_FD00(lift.y_pos, lift.y_velocity);

        let current_integer_y = (lift.y_pos >> 16) as u16;
        if current_integer_y != lift.last_integer_y {
            lift.moved_y_steps = lift.moved_y_steps.wrapping_add(1);
            lift.last_integer_y = current_integer_y;
        }

        if lift.moved_y_steps > 0x32 {
            lift.y_velocity = lift.y_velocity.wrapping_add(0x07D0);
        } else {
            lift.y_velocity = lift.y_velocity.wrapping_sub(0x0708);
        }

        let signed_integer_y = i32::from(i16::from_ne_bytes(current_integer_y.to_ne_bytes()));
        signed_integer_y < i32::from(level_height_pixels)
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_collision_rects_sub_F568(
        kid_hitbox: CollisionRectSub1F52,
        shooter_blocks: &[(u16, u16, u8)],
    ) -> Vec<CollisionRectSub1F52> {
        // ASM sub_F568 (kid.asm:20196): convert Kid's pixel hitbox to a
        // slightly-expanded block window, then emit up to two collision
        // rectangles per visible shooter edge inside that window.
        let left_block = (((i32::from(kid_hitbox.left_px)) >> 4) - 1).max(0) as i16;
        let top_block = (((i32::from(kid_hitbox.top_px)) >> 4) - 1).max(0) as i16;
        let right_block = ((i32::from(kid_hitbox.right_px) + 0x0F) >> 4) as i16 + 1;
        let bottom_block = ((i32::from(kid_hitbox.bottom_px) + 0x0F) >> 4) as i16 + 1;
        let mut rects = Vec::new();

        for &(col, row, directions) in shooter_blocks {
            let col = col as i16;
            let row = row as i16;
            if col < left_block || col > right_block || row < top_block || row > bottom_block {
                continue;
            }

            let x = col << 4;
            let y = row << 4;
            if directions & 0x01 != 0
                && row - top_block >= 2
                && col != left_block
                && col != right_block
            {
                rects.push(CollisionRectSub1F52 {
                    left_px: x + 2,
                    right_px: x + 14,
                    top_px: y - 13,
                    bottom_px: y - 1,
                });
                rects.push(CollisionRectSub1F52 {
                    left_px: x + 5,
                    right_px: x + 11,
                    top_px: y - 9,
                    bottom_px: y - 1,
                });
            }
            if directions & 0x02 != 0
                && col < right_block - 1
                && row != top_block
                && row != bottom_block
            {
                rects.push(CollisionRectSub1F52 {
                    left_px: (col + 1) << 4,
                    right_px: ((col + 1) << 4) + 8,
                    top_px: y + 1,
                    bottom_px: y + 13,
                });
                rects.push(CollisionRectSub1F52 {
                    left_px: (col + 1) << 4,
                    right_px: ((col + 1) << 4) + 12,
                    top_px: y + 4,
                    bottom_px: y + 10,
                });
            }
            if directions & 0x04 != 0
                && row < bottom_block - 1
                && col != left_block
                && col != right_block
            {
                rects.push(CollisionRectSub1F52 {
                    left_px: x + 2,
                    right_px: x + 14,
                    top_px: y + 15,
                    bottom_px: y + 23,
                });
                rects.push(CollisionRectSub1F52 {
                    left_px: x + 5,
                    right_px: x + 11,
                    top_px: y + 15,
                    bottom_px: y + 27,
                });
            }
            if directions & 0x08 != 0
                && col - left_block >= 2
                && row != top_block
                && row != bottom_block
            {
                rects.push(CollisionRectSub1F52 {
                    left_px: x - 8,
                    right_px: x,
                    top_px: y + 1,
                    bottom_px: y + 13,
                });
                rects.push(CollisionRectSub1F52 {
                    left_px: x - 12,
                    right_px: x,
                    top_px: y + 4,
                    bottom_px: y + 10,
                });
            }
        }

        rects
    }

    // Oracle-backed sub_1007A timing for demo 0x1B:
    // hit frame N:      write 0x6F
    // frames N+1..N+32: keep 0x6F during the bounce coroutine
    // frame N+33:       restore original prize cell (E1xx)
    // frame N+34:       clear cell and spawn prize object
    pub(crate) const PRIZE_BLOCK_DELETED_FRAMES: u16 = 32;
    pub(crate) const PRIZE_DIAMOND_LIFETIME_FRAMES: u16 = 0x01F4;
    pub(crate) const PRIZE_GENERIC_LIFETIME_FRAMES: u16 = 0x0320;
    // PrizeDiamond_Init starts moving on the frame after object creation, so a
    // newly spawned native floating item should be active immediately.
    pub(crate) const PRIZE_ITEM_INITIAL_DORMANT_FRAMES: u16 = 0;

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_headbutt_worker_loc_1002E(
        block_col: u16,
        block_row: u16,
        layout_word_addr: u32,
    ) -> PrizeBlockHeadbuttWorkerLoc1002E {
        // ASM loc_1002E (kid.asm:21317): preserve `d0`, play `sfx_Block_hit`,
        // allocate object/gfx slots via `sub_E02`, install `sub_1007A`, copy
        // `d1/d2/d3` into `$44/$46/$48(a0)`, and seed the gfx object's pixel
        // position by shifting the caller's block coordinates left by four.
        PrizeBlockHeadbuttWorkerLoc1002E {
            sound_effect_label: "sfx_Block_hit",
            allocation_spec: 0x2001,
            object_handler_addr: 0x0001_007A,
            block_col,
            block_row,
            layout_word_addr,
            gfx_x_pos: block_col << 4,
            gfx_y_pos: block_row << 4,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_deleted_tile_loc_101D8(
        block_col: u16,
        block_row: u16,
        deleted_skin: u8,
    ) -> PrizeBlockDeletedTileLoc101D8 {
        // ASM loc_101D8 (kid.asm:21459): write the deleted-solid tile by
        // storing `d4` into `(a0)` and then overriding only the high byte with
        // `$6F`, queue the same `(d0,d1,d4)` triple into
        // `FGUpdateQueueCell_NextSlot`. ASM loc_10200 (kid.asm:21474) then
        // publishes that advanced queue pointer, clears the child display byte
        // with `st $13(a3)`, preserves `d0` across the `sfx_Block_hit` call,
        // and seeds `d7` with `stru_10296` before the bounce coroutine.
        PrizeBlockDeletedTileLoc101D8 {
            written_cell: terrain::Cell::from_be_bytes(0x6F, deleted_skin),
            fg_queue_entry: (block_col, block_row, u16::from(deleted_skin)),
            child_display_enabled: false,
            sound_effect_label: "sfx_Block_hit",
            animation_script_addr: 0x0001_0296,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_wobble_y_deltas_sub_1007A() -> &'static [i16; 31] {
        // ASM loc_100DA (kid.asm:21373) starts the prize-block wobble by
        // repeating `subq.w #2,y_pos(a3)` for the inclusive `dbf d0` count
        // seeded from `moveq #5,d0`, then the later `loc_100E8/100F6/10104/
        // 10112/10120` loops mirror the remaining 6/5/5/4/5-frame bounces.
        &PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn deleted_tile_wobble_y_deltas_loc_10200() -> &'static [i16; 31] {
        // ASM loc_10236 (kid.asm:21493): after the opening six-frame
        // `loc_10228` rise, `moveq #5,d1` drives an inclusive `dbf` loop of
        // six `addq.w #2,y_pos(a3)` hibernate steps for the deleted tile.
        // ASM loc_10244 (kid.asm:21499) then reloads `d0` with `#4`, so the
        // next `subq.w #2,y_pos(a3)` phase lasts five frames before the later
        // rebound loops continue. ASM loc_10252 (kid.asm:21505) reloads `d1`
        // with `#4`, so the next rebound is five `addq.w #2,y_pos(a3)` steps.
        // ASM loc_10260 (kid.asm:21511) then reloads `d0` with `#3`, which
        // shortens the next `subq.w #2,y_pos(a3)` dip to four frames. ASM
        // loc_1026E (kid.asm:21517) reloads `d1` with `#4` for a final
        // five-frame `addq.w #2,y_pos(a3)` rebound before the animation waits
        // to finish and the worker deletes itself.
        &PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_spawn_flash_loc_102B8() -> PrizeBlockSpawnFlashLoc102B8 {
        // ASM loc_102B8 (kid.asm:21542): follow the worker's `$36(a5)` gfx
        // link, hide it via `st $13(a3)`, force palette line 0, seed
        // `addroffset_sprite(a3)` with `LnkTo_unk_E0F2E`, then run `stru_102DE`
        // until completion before the worker deletes itself.
        PrizeBlockSpawnFlashLoc102B8 {
            display_enabled: false,
            palette_line: 0,
            addroffset_sprite: 0x0F2E,
            animation_script_addr: 0x0001_02DE,
        }
    }

    fn floating_item_lifetime(content: terrain::PrizeContent) -> u16 {
        match content {
            terrain::PrizeContent::Diamond => Self::PRIZE_DIAMOND_LIFETIME_FRAMES,
            // ASM sub_10C78 seeds d3 with $0001_0320 for the shared
            // Ankh/Clock/Continue falling-prize initialiser, while sub_10CB0
            // reuses the same lifetime for helmets before its extra cleanup.
            // ASM loc_10464 (kid.asm:21700) is Skycutter's per-frame loop, and
            // ASM loc_104AE (kid.asm:21726) is Cyclone's matching loop, but
            // both still run on that same shared d3 countdown until the later
            // delete exit fires.
            terrain::PrizeContent::Points10000
            | terrain::PrizeContent::Skycutter
            | terrain::PrizeContent::Cyclone
            | terrain::PrizeContent::RedStealth
            | terrain::PrizeContent::Eyeclops
            | terrain::PrizeContent::Juggernaut
            | terrain::PrizeContent::IronKnight
            | terrain::PrizeContent::Berzerker
            | terrain::PrizeContent::Maniaxe
            | terrain::PrizeContent::Micromax
            | terrain::PrizeContent::Ankh
            | terrain::PrizeContent::Clock
            | terrain::PrizeContent::Continue
            | terrain::PrizeContent::TenDiamonds => Self::PRIZE_GENERIC_LIFETIME_FRAMES,
        }
    }

    fn is_helmet_prize(content: terrain::PrizeContent) -> bool {
        matches!(
            content,
            terrain::PrizeContent::Skycutter
                | terrain::PrizeContent::Cyclone
                | terrain::PrizeContent::RedStealth
                | terrain::PrizeContent::Eyeclops
                | terrain::PrizeContent::Juggernaut
                | terrain::PrizeContent::IronKnight
                | terrain::PrizeContent::Berzerker
                | terrain::PrizeContent::Maniaxe
                | terrain::PrizeContent::Micromax
        )
    }

    fn clear_existing_helmet_prizes_for_sub_10cec(&mut self) {
        let mut removed_effects = Vec::new();
        self.floating_items.retain(|item| {
            if !Self::is_helmet_prize(item.content) {
                return true;
            }

            let x_px = (item.x_pos >> 16) as u16;
            let y_px = ((item.y_pos >> 16) as u16).saturating_sub(8);
            removed_effects.push(BlockDestroyEffect {
                // ASM loc_10CF4 (kid.asm:22624) loops through the linked
                // object list until it finds an existing helmet prize in the
                // `$6C..$8C` object-meta range, then sub_10CEC deletes that
                // older owner before sub_10D3E spawns a transient animation
                // at the deleted helmet's x_pos and y_pos-8.
                x_px,
                y_px,
                remaining: 8,
            });
            false
        });
        // ASM loc_10D38 (kid.asm:22644) is the shared epilogue after the
        // loc_10CF4 scan, so when no older helmet matched the walk simply
        // falls through here and returns without seeding any cleanup effect.
        self.block_destroy_effects.extend(removed_effects);
    }

    const RUBBER_BLOCK_SUB_10DE4_SEQUENCE: [(u8, u8); 9] = [
        (1, 2),
        (0, 1),
        (1, 1),
        (0, 1),
        (1, 2),
        (0, 2),
        (1, 3),
        (0, 3),
        (0, 0),
    ];
    pub(crate) const BLOCK_DESTROY_DIRECTION_BELOW: u16 = 0;
    pub(crate) const BLOCK_DESTROY_DIRECTION_LEFT: u16 = 1;
    pub(crate) const BLOCK_DESTROY_DIRECTION_ABOVE: u16 = 2;
    pub(crate) const BLOCK_DESTROY_DIRECTION_RIGHT: u16 = 3;

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn rubber_block_base_skin_for_sub_10de4(cell: terrain::Cell) -> Option<u8> {
        if cell.block_type != terrain::BlockType::Rubber {
            return None;
        }

        // ASM loc_10E06 (kid.asm:22730): rubber skins `$0C/$0D` share the
        // low animation pair, so the worker seeds `d0` with `$E60C` before the
        // shared `loc_10E0A` toggles between `$E60C` and `$E60D`.
        Some(match cell.skin {
            0x0C | 0x0D => 0x0C,
            _ => 0x0E,
        })
    }

    fn write_rubber_block_skin_for_sub_10de4(
        &mut self,
        col: u16,
        row: u16,
        base_skin: u8,
        skin_offset: u8,
    ) {
        let Some(grid) = self.block_grid.as_mut() else {
            return;
        };
        let cell = grid.cell_at(col, row);
        if cell.block_type != terrain::BlockType::Rubber {
            return;
        }

        // ASM sub_10DE4 writes back into the same Level_Layout cell via
        // sub_11530, preserving the collision/block bits and only toggling the
        // rubber animation skin between the two adjacent frames.
        grid.set_cell_at(
            col,
            row,
            terrain::Cell::from_be_bytes(0xE6, base_skin.saturating_add(skin_offset)),
        );
    }

    fn start_rubber_block_animation_sub_10de4(&mut self, col: u16, row: u16) {
        let Some(grid) = self.block_grid.as_ref() else {
            return;
        };
        let Some(base_skin) = Self::rubber_block_base_skin_for_sub_10de4(grid.cell_at(col, row))
        else {
            return;
        };

        if let Some(existing) = self
            .block_state
            .rubber_block_animations
            .iter_mut()
            .find(|entry| entry.col == col && entry.row == row)
        {
            existing.base_skin = base_skin;
            existing.phase = 0;
            existing.frames_until_next_write = Self::RUBBER_BLOCK_SUB_10DE4_SEQUENCE[0].1;
        } else {
            self.block_state
                .rubber_block_animations
                .push(RubberBlockAnimation {
                    col,
                    row,
                    base_skin,
                    phase: 0,
                    frames_until_next_write: Self::RUBBER_BLOCK_SUB_10DE4_SEQUENCE[0].1,
                });
        }

        // ASM loc_10E0A (kid.asm:22733): `addq.w #1,d0; bsr.w sub_11530`
        // writes the `base+1` rubber frame immediately before the first
        // hibernate, so the contact frame already shows the jiggled rubber.
        self.write_rubber_block_skin_for_sub_10de4(col, row, base_skin, 1);
    }

    fn step_rubber_block_animations_sub_10de4(&mut self) {
        let mut pending_writes = Vec::new();
        self.block_state
            .rubber_block_animations
            .retain_mut(|entry| {
                if entry.frames_until_next_write > 0 {
                    entry.frames_until_next_write -= 1;
                }
                if entry.frames_until_next_write > 0 {
                    return true;
                }

                entry.phase = entry.phase.saturating_add(1);
                let Some(&(skin_offset, next_delay)) =
                    Self::RUBBER_BLOCK_SUB_10DE4_SEQUENCE.get(usize::from(entry.phase))
                else {
                    return false;
                };
                pending_writes.push((entry.col, entry.row, entry.base_skin, skin_offset));
                entry.frames_until_next_write = next_delay;
                entry.phase + 1 < Self::RUBBER_BLOCK_SUB_10DE4_SEQUENCE.len() as u8
            });

        for (col, row, base_skin, skin_offset) in pending_writes {
            self.write_rubber_block_skin_for_sub_10de4(col, row, base_skin, skin_offset);
        }
    }

    fn offset_block_destroy_effect_origin(direction: u16, x_px: u16, y_px: u16) -> (u16, u16) {
        match direction {
            Self::BLOCK_DESTROY_DIRECTION_RIGHT => (x_px.saturating_add(4), y_px),
            // ASM loc_10F1A (kid.asm:22825): `subq.w #4,y_pos(a3)` nudges the
            // smash effect 4 px upward when `d6 == 2`.
            Self::BLOCK_DESTROY_DIRECTION_ABOVE => (x_px, y_px.saturating_sub(4)),
            // ASM loc_10F20 (kid.asm:22829): `subq.w #4,x_pos(a3)` nudges the
            // smash effect 4 px left when `d6 == 1`.
            Self::BLOCK_DESTROY_DIRECTION_LEFT => (x_px.saturating_sub(4), y_px),
            _ => (x_px, y_px.saturating_add(4)),
        }
    }

    pub(crate) fn spawn_smash_block_effect_sub_10eda(
        &mut self,
        x_px: u16,
        y_px: u16,
        direction: u16,
    ) {
        let (x_px, y_px) = Self::offset_block_destroy_effect_origin(direction, x_px, y_px);
        self.block_destroy_effects.push(BlockDestroyEffect {
            // ASM sub_10EDA swaps the smash-block sprite art immediately,
            // nudges it 4px in the impact direction, then hibernates for 2
            // frames before dispatching the shared debris worker.
            x_px,
            y_px,
            remaining: 2,
        });
    }

    pub(crate) fn spawn_ice_block_effect_sub_10f9c(
        &mut self,
        x_px: u16,
        y_px: u16,
        direction: u16,
    ) {
        let (x_px, y_px) = Self::offset_block_destroy_effect_origin(direction, x_px, y_px);
        self.block_destroy_effects.push(BlockDestroyEffect {
            // ASM sub_10F9C mirrors sub_10EDA's directional 4px nudge but uses
            // the ice-block sprite set and a 3-frame hibernate before the
            // shared debris worker runs.
            x_px,
            y_px,
            remaining: 3,
        });
    }

    pub(crate) fn step_evanescent_blocks(&mut self, slice: &OpeningSliceState) {
        // Trigger: if the player is on ground, check the floor cell
        if slice.player.on_ground
            && let Some(grid) = &self.block_grid
        {
            let x_px = (slice.player.x_pos >> 16) as i32;
            let y_px = (slice.player.y_pos >> 16) as i32;
            let probe = terrain::probe_floor(grid, x_px, y_px);
            if probe.on_ground && probe.cell.block_type == terrain::BlockType::Evanescent {
                let col = terrain::pixel_to_cell(x_px).max(0) as u16;
                // The floor cell is the one the player is standing ON (below feet)
                let row = terrain::pixel_to_cell(probe.floor_y_px).max(0) as u16;
                let already_tracked = self
                    .block_state
                    .evanescent_blocks
                    .iter()
                    .any(|e| e.col == col && e.row == row);
                if !already_tracked
                    && self.block_state.evanescent_blocks.len() < EVANESCENT_MAX_ENTRIES
                {
                    let initial_timer = i16::from(probe.cell.skin) - EVANESCENT_SKIN_OFFSET;
                    self.block_state.evanescent_blocks.push(EvanescentEntry {
                        col,
                        row,
                        timer: initial_timer,
                    });
                }
            }
        }

        // Tick timers and remove expired blocks
        if let Some(grid) = &mut self.block_grid {
            self.block_state.evanescent_blocks.retain_mut(|entry| {
                entry.timer += 1;
                if entry.timer >= EVANESCENT_REMOVAL_THRESHOLD {
                    grid.set_cell_at(entry.col, entry.row, terrain::Cell::from_be_bytes(0, 0));
                    false
                } else {
                    true
                }
            });
        }
    }

    /// Ghost blocks: tick timers, toggle solid/vanished, update block grid.
    pub(crate) fn step_ghost_blocks(&mut self) {
        let Some(grid) = &mut self.block_grid else {
            return;
        };
        for group in &mut self.block_state.ghost_groups {
            if group.timer > 0 {
                group.timer -= 1;
                continue;
            }
            // Transition
            group.solid = !group.solid;
            group.timer = if group.solid {
                group.solid_time
            } else {
                group.vanished_time
            };

            // Update grid cells for each block in the group
            for i in 0..group.count {
                let (col, row) = if group.vertical {
                    (group.col, group.row + u16::from(i))
                } else {
                    (group.col + u16::from(i), group.row)
                };
                if group.solid {
                    // Restore solid ghost block (collision type 6 = Solid, block type 3 = SolidGhost)
                    let hi = 0x80 | (0x06 << 4) | 0x03;
                    grid.set_cell_at(col, row, terrain::Cell::from_be_bytes(hi, 0));
                } else {
                    // Clear the cell (vanished)
                    grid.set_cell_at(col, row, terrain::Cell::from_be_bytes(0, 0));
                }
            }
        }
    }

    /// When the player is standing on a Shifting block and the cell above is empty,
    /// move the block up one cell.
    /// Shooter blocks (type 0x8): fire projectiles in directions encoded in the
    /// skin byte (bit 0=up, 1=right, 2=down, 3=left). Speed: 4px/frame.
    /// ASM loc_EB72 -> LoadBulletsFromEvanescentBlock registers each visible
    /// shooter once, fires on age 0, then frees the slot again at age $16.
    const SHOOTER_FIRE_INTERVAL: u16 = 22;
    const SHOOTER_PROJECTILE_SPEED: i32 = 4 << 16; // 4px/frame in FP16.16
    const SHOOTER_HEADBUTT_HOLD_FRAMES: u8 = 8;

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_slot_tick_sub_F730(
        timer: u16,
        state: u16,
        active_delay: u16,
        reload_delay: u16,
    ) -> (u16, u16) {
        // ASM sub_F730 (kid.asm:20400): decrement the slot timer first, then
        // advance the visible-state word through the `0 -> $8001 -> $8002 ->
        // 3` and `n -> $8004 -> $8005 -> 0` countdown ladders.
        let decremented_timer = timer.wrapping_sub(1);
        if decremented_timer != 0 {
            return (decremented_timer, state);
        }

        if state & 0x8000 != 0 {
            return match state {
                0x8001 | 0x8004 => (decremented_timer.wrapping_add(2), state.wrapping_add(1)),
                0x8002 => (reload_delay, 3),
                _ => (active_delay, 0),
            };
        }

        if state == 0 {
            (decremented_timer.wrapping_add(2), 0x8001)
        } else {
            (decremented_timer.wrapping_add(2), 0x8004)
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_player_in_range_sub_F536(
        player_x_pos: u32,
        player_y_pos: u32,
        shooter_col: u16,
        shooter_row: u16,
    ) -> bool {
        // ASM sub_F536 (kid.asm:20167): convert Kid to block-space, then accept
        // the shooter when `kid_x_block-4 .. kid_x_block+4` contains the
        // source col and `kid_y_block-6 .. kid_y_block+3` contains the source
        // row. The comparisons are inclusive because the ROM uses `bgt`/`bmi`
        // after subtracting the shooter coordinates.
        let mut delta_x = ((player_x_pos >> 16) as i32 >> 4) - 4 - i32::from(shooter_col);
        if delta_x > 0 {
            return false;
        }
        delta_x += 8;
        if delta_x < 0 {
            return false;
        }

        let mut delta_y = ((player_y_pos >> 16) as i32 >> 4) - 6 - i32::from(shooter_row);
        if delta_y > 0 {
            return false;
        }
        delta_y += 9;
        delta_y >= 0
    }

    fn shooter_deleted_tile_for_sub_114c0(skin: u8) -> terrain::Cell {
        terrain::Cell::from_be_bytes(0xEF, skin)
    }

    fn shooter_support_tile_for_sub_114f4() -> terrain::Cell {
        terrain::Cell::from_be_bytes(0xE9, 0x19)
    }

    /// Tick prize-block bounce lifecycles. ASM sub_1007A holds 0x6F, briefly
    /// restores the original prize cell, then writes `$E001` and creates the
    /// prize object.
    pub(crate) fn step_deferred_ceiling_clears(&mut self) {
        if let Some(grid) = &mut self.block_grid {
            self.block_state
                .deferred_ceiling_clears
                .retain_mut(|entry| match entry.phase {
                    DeferredPrizeBlockPhase::Deleted => {
                        if entry.deleted_frames_remaining == 0 {
                            grid.set_cell_at(entry.col, entry.row, entry.restore_cell);
                            entry.phase = DeferredPrizeBlockPhase::Restored;
                        } else {
                            entry.deleted_frames_remaining -= 1;
                        }
                        true
                    }
                    DeferredPrizeBlockPhase::Restored => {
                        entry.phase = DeferredPrizeBlockPhase::PendingClear;
                        true
                    }
                    DeferredPrizeBlockPhase::PendingClear | DeferredPrizeBlockPhase::FreshSmash => {
                        true
                    }
                });
        }
    }

    pub(crate) fn finish_deferred_ceiling_clears(&mut self) {
        if self.block_grid.is_none() {
            return;
        }

        let mut spawned_items = Vec::new();
        let mut retained_entries = Vec::new();
        for mut entry in std::mem::take(&mut self.block_state.deferred_ceiling_clears) {
            match entry.phase {
                DeferredPrizeBlockPhase::Deleted | DeferredPrizeBlockPhase::Restored => {
                    retained_entries.push(entry);
                }
                DeferredPrizeBlockPhase::PendingClear => {
                    // ASM sub_1007A line 21411: after prize spawns, restore
                    // cell to $E001 (Smash block, skin=1). The player can
                    // then smash this block on a second headbutt.
                    if let Some(grid) = self.block_grid.as_mut() {
                        grid.set_cell_at(
                            entry.col,
                            entry.row,
                            terrain::Cell::from_be_bytes(0xE0, 0x01),
                        );
                    }
                    if Self::is_helmet_prize(entry.content) {
                        self.clear_existing_helmet_prizes_for_sub_10cec();
                    }
                    spawned_items.push(FloatingItem {
                        x_pos: (u32::from(entry.col) * 16 + 8) << 16,
                        y_pos: i64::from(u32::from(entry.row) * 16) << 16,
                        y_velocity: 0,
                        remaining: Self::floating_item_lifetime(entry.content),
                        spawn_delay: Self::PRIZE_ITEM_INITIAL_DORMANT_FRAMES,
                        level_col: entry.col,
                        level_row: entry.row,
                        first_row_grace: true,
                        content: entry.content,
                        collected: false,
                    });
                    // ASM sub_7196 creates Kid with priority #1, while
                    // sub_1007A runs as object #$2001 and spawns Prize_Init
                    // at #$1FFF. Kid therefore finishes this frame before the
                    // prize-block coroutine writes $E001, so the freshly
                    // restored Smash cell must stay non-blocking for Kid
                    // until the next frame ends.
                    entry.phase = DeferredPrizeBlockPhase::FreshSmash;
                    retained_entries.push(entry);
                }
                DeferredPrizeBlockPhase::FreshSmash => {}
            }
        }
        self.block_state.deferred_ceiling_clears = retained_entries;
        self.floating_items.extend(spawned_items);
    }

    pub(crate) fn step_shooter_blocks(&mut self, slice: &OpeningSliceState) {
        let Some(grid) = &self.block_grid else {
            return;
        };

        let kid_block_col = ((slice.player.x_pos >> 16) >> 4) as i16;
        let kid_block_row = ((slice.player.y_pos >> 16) >> 4) as i16;
        let max_col = i16::try_from(grid.width_cells.saturating_sub(1))
            .expect("block grids should fit in i16");
        let max_row = i16::try_from(grid.height_cells.saturating_sub(1))
            .expect("block grids should fit in i16");
        let previous_col = self.block_state.shooter_last_scanned_col;
        let previous_row = self.block_state.shooter_last_scanned_row;
        let mut pending_entries = Vec::new();
        let mut queue_scan = |col: u16, row: u16, skin: u8| {
            let already_registered = self
                .block_state
                .shooter_blocks
                .iter()
                .any(|entry| entry.col == col && entry.row == row);
            let already_pending = pending_entries
                .iter()
                .any(|(pending_col, pending_row, _)| *pending_col == col && *pending_row == row);
            if !already_registered && !already_pending {
                pending_entries.push((col, row, skin));
            }
        };

        if previous_col != kid_block_col {
            let entering_col = if previous_col < kid_block_col {
                (kid_block_col + 3).min(max_col)
            } else {
                (kid_block_col - 3).max(0)
            } as u16;
            let start_row = (kid_block_row - 5).max(0) as u16;
            let end_row = (kid_block_row + 2).min(max_row) as u16;
            for row in start_row..=end_row {
                let cell = grid.cell_at(entering_col, row);
                if cell.has_block && cell.block_type == terrain::BlockType::Shooter {
                    queue_scan(entering_col, row, cell.skin);
                }
            }
        }

        if previous_row != kid_block_row {
            let entering_row = if previous_row < kid_block_row {
                (kid_block_row + 2).min(max_row)
            } else {
                (kid_block_row - 5).max(0)
            } as u16;
            let start_col = (kid_block_col - 3).max(0) as u16;
            let end_col = (kid_block_col + 3).min(max_col) as u16;
            for col in start_col..=end_col {
                let cell = grid.cell_at(col, entering_row);
                if cell.has_block && cell.block_type == terrain::BlockType::Shooter {
                    queue_scan(col, entering_row, cell.skin);
                }
            }
        }

        self.block_state.shooter_last_scanned_col = kid_block_col;
        self.block_state.shooter_last_scanned_row = kid_block_row;
        self.block_state
            .shooter_blocks
            .extend(
                pending_entries
                    .into_iter()
                    .map(|(col, row, directions)| ShooterBlockEntry {
                        col,
                        row,
                        directions,
                        age: 0,
                    }),
            );

        self.block_state.shooter_blocks.retain_mut(|entry| {
            let still_shooter = matches!(
                grid.cell_at(entry.col, entry.row),
                terrain::Cell {
                    has_block: true,
                    block_type: terrain::BlockType::Shooter,
                    ..
                }
            );
            if !still_shooter {
                return false;
            }

            if entry.age == 0 {
                let block_x = u32::from(entry.col) * 16 + 8;
                let block_y = u32::from(entry.row) * 16 + 8;
                let directions: [(u8, i32, i32); 4] = [
                    (1, 0, -Self::SHOOTER_PROJECTILE_SPEED),
                    (2, Self::SHOOTER_PROJECTILE_SPEED, 0),
                    (4, 0, Self::SHOOTER_PROJECTILE_SPEED),
                    (8, -Self::SHOOTER_PROJECTILE_SPEED, 0),
                ];
                for (mask, xv, yv) in directions {
                    if entry.directions & mask != 0 {
                        self.projectiles.push(EnemyProjectile {
                            x_pos: block_x << 16,
                            y_pos: block_y << 16,
                            x_velocity: xv,
                            y_velocity: yv,
                            lifetime: 120,
                            gravity: 0,
                            friendly: false,
                            homing_accel: 0,
                            speed_cap: 0,
                            wall_bounce: false,
                            ..Default::default()
                        });
                    }
                }
            }

            entry.age += 1;
            entry.age < Self::SHOOTER_FIRE_INTERVAL
        });
    }

    pub(crate) fn step_shooter_headbutt_workers(&mut self) {
        let Some(grid) = &mut self.block_grid else {
            self.block_state.shooter_headbutt_workers.clear();
            return;
        };

        self.block_state
            .shooter_headbutt_workers
            .retain_mut(|worker| {
                match worker.kind {
                    ShooterHeadbuttWorkerKind::Center => match worker.age {
                        0 => {
                            // ASM sub_114C0: first execution writes `$EF1B`.
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                Self::shooter_deleted_tile_for_sub_114c0(0x1B),
                            );
                        }
                        Self::SHOOTER_HEADBUTT_HOLD_FRAMES => {
                            // ASM sub_114C0: after one 8-frame hibernate, write `$EF1A`.
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                Self::shooter_deleted_tile_for_sub_114c0(0x1A),
                            );
                        }
                        age if age == Self::SHOOTER_HEADBUTT_HOLD_FRAMES * 2 => {
                            // ASM sub_114C0: after the second 8-frame hibernate, erase the cell.
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                terrain::Cell::from_be_bytes(0, 0),
                            );
                            return false;
                        }
                        _ => {}
                    },
                    ShooterHeadbuttWorkerKind::Support => match worker.age {
                        0 => {
                            // ASM sub_114F4 bails if the target cell is no longer empty when
                            // the worker first wakes.
                            if grid.cell_at(worker.col, worker.row)
                                != terrain::Cell::from_be_bytes(0, 0)
                            {
                                return false;
                            }
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                Self::shooter_deleted_tile_for_sub_114c0(0x1A),
                            );
                        }
                        Self::SHOOTER_HEADBUTT_HOLD_FRAMES => {
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                Self::shooter_deleted_tile_for_sub_114c0(0x1B),
                            );
                        }
                        age if age == Self::SHOOTER_HEADBUTT_HOLD_FRAMES * 2 => {
                            grid.set_cell_at(
                                worker.col,
                                worker.row,
                                Self::shooter_support_tile_for_sub_114f4(),
                            );
                            return false;
                        }
                        _ => {}
                    },
                }

                worker.age = worker.age.saturating_add(1);
                true
            });
    }

    pub(crate) fn apply_shifting_blocks(&mut self, slice: &OpeningSliceState) {
        if !slice.player.on_ground {
            return;
        }
        let Some(grid) = &mut self.block_grid else {
            return;
        };

        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        // The floor cell is just below the player's feet
        let col = terrain::pixel_to_cell(x_px).max(0) as u16;
        let foot_row = terrain::pixel_to_cell(y_px + 1).max(0) as u16;

        let cell = grid.cell_at(col, foot_row);
        if cell.block_type != terrain::BlockType::Shifting {
            return;
        }
        if foot_row == 0 {
            return; // no cell above to shift into
        }

        let above = grid.cell_at(col, foot_row - 1);
        // ASM only checks `btst #6` (has_block bit) — no collision check.
        if above.has_block {
            return; // cell above is occupied
        }

        // Move block up: copy cell to row above, clear current cell
        grid.set_cell_at(col, foot_row - 1, cell);
        grid.set_cell_at(col, foot_row, terrain::Cell::from_be_bytes(0, 0));
    }

    pub(crate) fn step_lift_platforms(&mut self, slice: &mut OpeningSliceState) {
        self.block_state.lift_platforms.retain_mut(|lift| {
            if !lift.block_erased {
                if let Some(grid) = &mut self.block_grid {
                    grid.set_cell_at(
                        lift.source_col,
                        lift.source_row,
                        terrain::Cell::from_be_bytes(0, 0),
                    );
                }
                lift.block_erased = true;
            }

            if !Self::step_lift_platform_sub_FCA6(lift, slice.level_bounds.height_pixels) {
                return false;
            }

            // Platform collision: player can stand on lift
            let plat_x_px = (lift.x_pos >> 16) as i32;
            let plat_y_px = (lift.y_pos >> 16) as i32;
            let player_x_px = (slice.player.x_pos >> 16) as i32;
            let player_foot_y_px = (slice.player.y_pos >> 16) as i32;
            let player_left = player_x_px - PLAYER_HALF_WIDTH;
            let player_right = player_x_px + PLAYER_HALF_WIDTH;
            let plat_right = plat_x_px + i32::from(lift.width_px);

            if player_right >= plat_x_px
                && player_left <= plat_right
                && slice.player.y_velocity >= 0
                && !slice.player.on_ground
            {
                let foot_dist = player_foot_y_px - plat_y_px;
                if (-2..=4).contains(&foot_dist) {
                    slice.player.y_pos = lift.y_pos;
                    slice.player.y_velocity = lift.y_velocity;
                    slice.player.on_ground = true;
                    slice.terrain.floor_y_pos = lift.y_pos;
                }
            }

            // If player is on ground and riding this lift, carry along
            if slice.player.on_ground {
                let within_snap =
                    slice.player.y_pos.abs_diff(lift.y_pos) <= GROUND_SNAP_TOLERANCE_FP;
                if within_snap && player_right >= plat_x_px && player_left <= plat_right {
                    slice.player.y_pos = (i64::from(slice.player.y_pos)
                        + i64::from(lift.y_velocity))
                    .clamp(0, i64::from(u32::MAX)) as u32;
                    slice.terrain.floor_y_pos =
                        (((slice.player.y_pos >> 16) as i32).max(0) as u32) << 16;
                }
            }

            true
        });
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn lift_block_bob_deltas_sub_FB3E() -> &'static [i16] {
        // ASM sub_FB3E (kid.asm:20905): before the lift platform spawns, the
        // block bobs in three hibernating up/down passes: 6, 5, then 4/5
        // frames at 2px per frame.
        &[
            -2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, 2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, -2, -2,
            -2, -2, 2, 2, 2, 2, 2,
        ]
    }

    /// Process block hit effects from the current frame's slice (prizes, bounces,
    /// lifts, drills, teleporter warps).
    pub(crate) fn process_block_effects(&mut self, slice: &mut OpeningSliceState) {
        // Prize block hit: spawn the immediate debris effect. The actual prize
        // object is created later by the queued sub_1007A lifecycle.
        if let Some((_content, x_px, y_px)) = slice.prize_block_hit {
            self.blocks_hit_this_frame += 1;
            self.block_destroy_effects.push(BlockDestroyEffect {
                x_px,
                y_px,
                remaining: 8,
            });
        }

        // Metal block bounce animation
        if let Some((cx, cy)) = slice.metal_block_hit {
            self.blocks_hit_this_frame += 1;
            self.block_bounces.push(BlockBounceAnim {
                cell_x: cx,
                cell_y: cy,
                remaining: 8,
            });
        }

        // ASM loc_10DA4: rubber contact spawns a temporary jiggle object and
        // plays sfx_Rubber_block without changing the collision response.
        if let Some((cx, cy)) = slice.rubber_block_hit {
            self.blocks_hit_this_frame += 1;
            self.start_rubber_block_animation_sub_10de4(cx, cy);
            self.block_bounces.push(BlockBounceAnim {
                cell_x: cx,
                cell_y: cy,
                remaining: 8,
            });
        }

        // Smash block destruction effect
        if let Some((x_px, y_px)) = slice.smash_block_hit {
            self.blocks_hit_this_frame += 1;
            self.spawn_smash_block_effect_sub_10eda(
                x_px,
                y_px,
                Self::BLOCK_DESTROY_DIRECTION_BELOW,
            );
        }

        if let Some((cell_x, cell_y)) = slice.shooter_block_hit
            && let Some(grid) = self.block_grid.as_mut()
        {
            self.blocks_hit_this_frame += 1;
            let shooter_cell = grid.cell_at(cell_x, cell_y);
            // ASM loc_11430 marks the struck shooter as deleted-solid
            // immediately via `or.b #$F,(a1)` before the timed workers run.
            grid.set_cell_at(
                cell_x,
                cell_y,
                Self::shooter_deleted_tile_for_sub_114c0(shooter_cell.skin),
            );

            self.block_state
                .shooter_headbutt_workers
                .push(ShooterHeadbuttWorker {
                    col: cell_x,
                    row: cell_y,
                    age: 0,
                    kind: ShooterHeadbuttWorkerKind::Center,
                });

            if cell_y > 0 {
                let support_row = cell_y - 1;
                let mut try_queue_support = |support_col: u16| {
                    if grid.cell_at(support_col, support_row) == terrain::Cell::from_be_bytes(0, 0)
                    {
                        // ASM sub_1149C: queue one support worker per empty cell above the
                        // struck shooter, including the diagonals when in bounds.
                        self.block_state
                            .shooter_headbutt_workers
                            .push(ShooterHeadbuttWorker {
                                col: support_col,
                                row: support_row,
                                age: 0,
                                kind: ShooterHeadbuttWorkerKind::Support,
                            });
                    }
                };

                try_queue_support(cell_x);
                if cell_x > 0 {
                    try_queue_support(cell_x - 1);
                }
                if cell_x + 1 < grid.width_cells {
                    try_queue_support(cell_x + 1);
                }
            }
        }

        // Lift block: headbutt spawns a rising platform (ASM sub_113C0)
        if let Some((x_px, y_px)) = slice.lift_block_hit {
            let col = x_px / 16;
            let row = y_px / 16;
            self.block_state.lift_platforms.push(LiftPlatform {
                x_pos: u32::from(x_px) << 16,
                y_pos: u32::from(y_px) << 16,
                y_velocity: 0,
                age: 0,
                last_integer_y: y_px,
                moved_y_steps: 0,
                width_px: 16,
                source_col: col,
                source_row: row,
                block_erased: false,
            });
        }

        // Drill block: headbutt spawns a horizontal projectile (ASM sub_1129C: ±4px/frame)
        if let Some((x_fp, y_fp)) = slice.drill_projectile_hit {
            // Direction based on player position relative to drill block
            let dir = if slice.player.x_pos >= x_fp { 1i32 } else { -1 };
            self.projectiles.push(EnemyProjectile {
                x_pos: x_fp,
                y_pos: y_fp,
                x_velocity: dir * 0x4_0000, // 4 px/frame horizontal
                y_velocity: 0,
                lifetime: 120,
                gravity: 0,
                friendly: false,
                homing_accel: 0,
                speed_cap: 0,
                wall_bounce: false,
                ..Default::default()
            });
        }

        // Teleporter block: warp player to destination
        if let Some((cell_x, cell_y)) = slice.teleporter_contact
            && let Some(block_list) = self.block_list.as_ref()
            && let Some(tp) = block_list.teleporter_at(cell_x, cell_y)
        {
            // Warp to destination position
            slice.player.x_pos = (u32::from(tp.dest_x) * 16) << 16;
            slice.player.y_pos = (u32::from(tp.dest_y) * 16) << 16;
            slice.player.y_velocity = 0;
            slice.player.on_ground = true;
        }
    }

    /// Check player overlap with floating prize items and collect on contact.
    /// ASM spawns prize items as separate objects — the player must physically
    /// touch them. The native previously applied prizes immediately at block-hit
    /// time, causing spurious form changes.
    /// Check player overlap with floating prize items and collect on contact.
    /// ASM spawns prize items as separate objects — the player must physically
    /// touch them. The native previously applied prizes immediately at block-hit
    /// time, causing spurious form changes.
    /// Check player overlap with floating prize items and collect on contact.
    /// ASM uses GfxObjects_Collision (collision_type field) to detect player
    /// contact each frame. The collision check uses the standard sprite bounding
    /// box overlap from the object system.
    pub(crate) fn collect_floating_items(&mut self, slice: &mut OpeningSliceState) {
        use crate::constants::{PLAYER_HALF_WIDTH, PLAYER_HEIGHT};
        const PLAYER_PRIZE_PICKUP_HEIGHT: i32 = (PLAYER_HEIGHT + 1) / 2;

        let player_x_px = (slice.player.x_pos >> 16) as i32;
        let player_y_px = (slice.player.y_pos >> 16) as i32;
        let player_left = player_x_px - PLAYER_HALF_WIDTH;
        let player_right = player_x_px + PLAYER_HALF_WIDTH;

        let mut collected_contents = Vec::new();
        let mut collected_prize_diamonds = 0usize;
        for item in &mut self.floating_items {
            if item.collected || item.spawn_delay > 0 {
                continue;
            }
            let item_x_px = (item.x_pos >> 16) as i32;
            let item_y_px = (item.y_pos >> 16) as i32;
            let (player_top, item_left, item_right, item_top, item_bottom, inclusive_overlap) =
                if item.content == terrain::PrizeContent::Diamond {
                    // PrizeDiamond_Init uses the standard gfx-object collision path,
                    // with `y_pos` anchored at the object's bottom edge rather than
                    // the top of a tiny 16px sprite box.
                    (
                        player_y_px - PLAYER_HEIGHT,
                        item_x_px - 7,
                        item_x_px + 7,
                        item_y_px - PLAYER_HEIGHT,
                        item_y_px,
                        true,
                    )
                } else {
                    // Oracle demo 0x1B keeps falling helmets non-collectible until
                    // they overlap Kid's lower body, not the full 31px sprite box.
                    (
                        player_y_px - PLAYER_PRIZE_PICKUP_HEIGHT,
                        item_x_px - 8,
                        item_x_px + 8,
                        item_y_px,
                        item_y_px + 16,
                        false,
                    )
                };

            let overlap_x = if inclusive_overlap {
                player_left <= item_right && player_right >= item_left
            } else {
                player_left < item_right && player_right > item_left
            };
            let overlap_y = if inclusive_overlap {
                player_top <= item_bottom && player_y_px >= item_top
            } else {
                player_top < item_bottom && player_y_px > item_top
            };

            if overlap_x && overlap_y {
                // ASM loc_1000A / loc_1001E (kid.asm:21302-21312) hibernate
                // the collected clock object and keep recomputing `x_pos/y_pos`
                // from the cached camera-relative words at `$3E/$42` until the
                // final delete. Native does not model that lingering sprite, so
                // collection only flips the removal flag and leaves the cached
                // coordinates untouched until `step_visual_effects` removes it.
                item.collected = true;
                if item.content == terrain::PrizeContent::Diamond {
                    collected_prize_diamonds += 1;
                } else {
                    collected_contents.push(item.content);
                }
            }
        }
        for _ in 0..collected_prize_diamonds {
            self.queue_prize_diamond_award();
        }
        for content in collected_contents {
            self.apply_prize_content(content);
        }
    }

    pub(crate) fn step_visual_effects(&mut self) {
        self.step_rubber_block_animations_sub_10de4();
        let level_height_pixels = i64::from(self.level_bounds(self.current_level).height_pixels);

        let floating_item_can_fall = |grid: &terrain::BlockGrid, item: &FloatingItem| {
            let current_cell = grid.cell_at(item.level_col, item.level_row);
            let current_row_open = matches!(
                current_cell.collision,
                terrain::CollisionKind::Empty | terrain::CollisionKind::HiddenBlock
            );
            if current_row_open {
                return true;
            }
            // ASM loc_103B2 (kid.asm:21630): sample the current row first and
            // stop immediately on solid rows unless the bit-16 grace flag is
            // still set. When that grace bit is present, the routine checks
            // one row below and keeps falling only if that second sample is
            // passable too.
            if !item.first_row_grace {
                return false;
            }
            let below_cell = grid.cell_at(item.level_col, item.level_row.saturating_add(1));
            matches!(
                below_cell.collision,
                terrain::CollisionKind::Empty | terrain::CollisionKind::HiddenBlock
            )
        };

        for b in &mut self.block_bounces {
            b.remaining = b.remaining.saturating_sub(1);
        }
        self.block_bounces.retain(|b| b.remaining > 0);

        for e in &mut self.block_destroy_effects {
            e.remaining = e.remaining.saturating_sub(1);
        }
        self.block_destroy_effects.retain(|e| e.remaining > 0);

        for f in &mut self.floating_items {
            if f.spawn_delay > 0 {
                f.spawn_delay -= 1;
                continue;
            }
            // ASM loc_10396 (kid.asm:21621) and loc_1071E (kid.asm:21965):
            // each hibernated frame decrements `d3`, deletes the object when
            // the countdown reaches zero, and swaps animation exactly at the
            // routine-specific threshold (`$00C8` for diamonds, `$012C` for
            // the ankh). Native floating items do not carry animation-script
            // state, so this loop models the shared countdown/delete timing
            // and leaves the visual script swap implicit.
            f.remaining = f.remaining.saturating_sub(1);
            let can_fall = self
                .block_grid
                .as_ref()
                .is_none_or(|grid| floating_item_can_fall(grid, f));
            if can_fall {
                let aligned_y = ((f.y_pos >> 16) as i32) & !0xF;
                let fall = Self::floating_prize_fall_step_loc_10C50(
                    aligned_y,
                    f.y_pos,
                    f.y_velocity,
                    f.first_row_grace,
                    f.level_row,
                    0,
                    0,
                );
                f.y_velocity = fall.next_y_velocity;
                f.y_pos = fall.next_y_pos;
                f.first_row_grace = fall.next_first_row_grace;
                f.level_row = fall.next_level_row;
            } else {
                // ASM loc_103DA (kid.asm:21645): `moveq #0,d1` feeds the
                // shared tail at `loc_10404`, so blocked floating prizes
                // cancel the pending vertical delta for this frame.
                f.y_velocity = 0;
            }
        }
        self.floating_items.retain(|f| {
            let integer_y = f.y_pos >> 16;
            // ASM loc_10430 (kid.asm:21678): after the shared tail at
            // `loc_10404`, delete the prize object once `y_pos(a3)` reaches
            // or exceeds `Level_height_pixels`.
            // ASM loc_10738 (kid.asm:21974) is PrizeAnkh's matching per-frame
            // loop body: once `sub_10C38` updates the fall state, the object
            // stays alive only while its integer `y_pos(a3)` remains strictly
            // below `Level_height_pixels` and `collision_type(a3)` stays zero.
            // ASM loc_1047C (kid.asm:21709), loc_104C6 (kid.asm:21735),
            // loc_10510 (kid.asm:21761), loc_1055A (kid.asm:21787),
            // loc_105A4 (kid.asm:21813), loc_10638 (kid.asm:21865),
            // loc_10682 (kid.asm:21891), and loc_10768 (kid.asm:21987) are
            // the matching Skycutter/Cyclone/Red Stealth/Eyeclops/
            // Juggernaut/Berzerker/Maniaxe/PrizeAnkh delete exits: once the
            // per-frame loop at loc_10464/loc_104AE/loc_104F8/loc_10542/
            // loc_1058C/loc_10620/loc_1066A/loc_10738 sees either a non-zero
            // `collision_type(a3)` or the bottom-of-level compare, control
            // jumps straight to `Delete_CurrentObject`.
            let gate =
                Self::prize_clock_fall_gate_loc_107B0(integer_y, level_height_pixels, f.collected);
            (f.spawn_delay > 0 || f.remaining > 0) && gate.stays_active
        });

        for e in &mut self.enemy_death_effects {
            e.remaining = e.remaining.saturating_sub(1);
        }
        self.enemy_death_effects.retain(|e| e.remaining > 0);

        // ASM loc_10434 (kid.asm:21683) is the `bra.s loc_10434` padding trap
        // immediately after the `OneTimePrize_CheckCollected` function chunk.
        // Native falls back to ordinary Rust control flow here and deliberately
        // does not model the unreachable self-loop.
    }
}
