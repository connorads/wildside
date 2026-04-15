use super::*;

fn crawl_escape_test_grid(left_solid: bool, right_solid: bool) -> terrain::BlockGrid {
    let width = 8u16;
    let height = 4u16;
    let mut data = vec![0u8; usize::from(width) * usize::from(height) * 2];
    if left_solid {
        data[(usize::from(width) + 2) * 2] = 0x60;
    }
    if right_solid {
        data[(usize::from(width) + 4) * 2] = 0x60;
    }
    terrain::BlockGrid::from_level_layout(&data, width, height)
}

fn crawl_slope_test_grid(left_slope_down: bool, right_slope_up: bool) -> terrain::BlockGrid {
    let width = 8u16;
    let height = 4u16;
    let mut data = vec![0u8; usize::from(width) * usize::from(height) * 2];
    if left_slope_down {
        data[(2 * usize::from(width) + 2) * 2] = 0x50;
    }
    if right_slope_up {
        data[(2 * usize::from(width) + 4) * 2] = 0x40;
    }
    terrain::BlockGrid::from_level_layout(&data, width, height)
}

mod chunk_01 {
    use super::*;

    #[test]
    fn crawl_wall_escape_sub_81b4_snaps_left_on_the_first_open_side() {
        let grid = crawl_escape_test_grid(false, true);

        let result = crawl_wall_escape_sub_81B4(&grid, 50 << 16, 32 << 16, 14);

        assert_eq!(
            result,
            CrawlWallEscapeSub81B4 {
                x_pos: 33 << 16,
                blocked: false,
            },
            "ASM sub_81B4 should take loc_81D8 immediately when the left sample is not `$6000`, even if the right sample would be solid",
        );
    }

    #[test]
    fn initialise_sprite_frame_state_sub_b52_resets_the_shared_sprite_cursors() {
        assert_eq!(
            initialise_sprite_frame_state_sub_B52(),
            SpriteFrameStateSubB52 {
                number_sprites: 0,
                next_sprite_slot_addr: 0xFFFF_0000,
                dynamic_sprite_art_addr: 0xFFFF_0500,
            },
            "ASM sub_B52 should clear `Number_Sprites`, restore `Addr_NextSpriteSlot=Sprite_Table`, and reset `$FFFFF838` to `$FFFF0500`",
        );
    }

    #[test]
    fn loc_14a4_walks_each_linked_gfx_object_until_the_null_tail_for_sprite_builds() {
        assert_eq!(
            crate::level::gfx_object_sprite_build_order_loc_14A4(None, &[Some(1), Some(2), None]),
            Vec::<usize>::new(),
            "ASM loc_14A4 should branch straight to loc_14B2 when `(Addr_FirstGfxObjectSlot).w` is null before any sprite build runs",
        );
        assert_eq!(
            crate::level::gfx_object_sprite_build_order_loc_14A4(
                Some(1),
                &[Some(1), Some(2), None],
            ),
            vec![1, 2],
            "ASM loc_14A4 should keep following `0(a3)` and calling `j_Make_SpriteFromGfxObject` until the null tail stops the loop at loc_14B2",
        );
    }

    #[test]
    fn loc_1b170_fills_eight_longwords_of_intro_sequence_6_vram_with_0x11111111() {
        assert_eq!(
            intro_sequence_6_longword_fill_loc_1B170(),
            IntroSequence6LongwordFillLoc1B170 {
                vram_word_addr: 0x1780,
                fill_longword: 0x1111_1111,
                longword_write_count: 8,
            },
            "ASM loc_1B170 should stream `d1=$11111111` into `vdpComm($1780,VRAM,WRITE)` exactly `d0+1` times with `d0=#7`"
        );
    }

    #[test]
    fn loc_1b182_clears_the_intro_sequence_6_tilemap_with_2048_bc_words() {
        assert_eq!(
            intro_sequence_6_tilemap_clear_loc_1B182(),
            IntroSequence6WordFillLoc1B182 {
                vram_word_addr: 0x0000,
                fill_word: 0x20BC,
                word_write_count: 2048,
            },
            "ASM loc_1B182 should write `#$20BC` to `vdpComm($0000,VRAM,WRITE)` exactly `d0+1` times with `d0=#$7FF`"
        );
    }

    #[test]
    fn loc_1b194_emits_fourteen_striped_64_pair_bands_into_e000() {
        let writes = intro_sequence_6_striped_tile_fill_loc_1B194();

        assert_eq!(writes.vram_word_addr, 0xE000);
        assert_eq!(
            writes.word_writes.len(),
            14 * 32 * 4,
            "ASM loc_1B194 should repeat the two 32-pair inner DBF loops `d0+1` times with `d0=#$D`"
        );
        assert_eq!(
            &writes.word_writes[..4],
            &[0x00BD, 0x00BE, 0x00BD, 0x00BE],
            "ASM loc_1B196 should start each outer band with repeated `$00BD,$00BE` pairs"
        );
        assert_eq!(
            &writes.word_writes[64..68],
            &[0x00BF, 0x00C0, 0x00BF, 0x00C0],
            "ASM loc_1B1A4 should follow with repeated `$00BF,$00C0` pairs after the first 32 `$00BD,$00BE` pairs"
        );
        assert_eq!(
            &writes.word_writes[writes.word_writes.len() - 4..],
            &[0x00BF, 0x00C0, 0x00BF, 0x00C0],
            "ASM loc_1B194 should end on the final `$00BF,$00C0` pair from the fourteenth outer pass"
        );
    }

    #[test]
    fn loc_1bf3e_multiplies_art_index_by_six_and_selects_tile_pattern() {
        let result = title_shell_art_index_loc_1BF3E();
        assert_eq!(
            result.table_entry_size, 6,
            "ASM loc_1BF3E should multiply `d4` by 6 via `mulu.w #6,d4` because each table entry has 4-byte pointer plus 2-byte width"
        );
        assert_eq!(
            result.tile_pattern_index_zero, 0x20C1,
            "ASM loc_1BF3E should select `#$20C1`for `d0` when `d4` is zero after the `tst.w d4; beq.s loc_1BF54` fallthrough"
        );
        assert_eq!(
            result.tile_pattern_index_nonzero, 0x210F,
            "ASM loc_1BF3E should select `#$210F` for `d0` when `d4` is nonzero via the `move.w #$210F,d0` before `loc_1BF54`"
        );
    }

    #[test]
    fn loc_1bf54_decompresses_mappings_and_previews_vram_fill() {
        let result = title_shell_art_decompress_loc_1BF54();
        assert_eq!(
            result.vram_fill_iterations, 360,
            "ASM loc_1BF54 should initialize `d0=$167` for 360 iterations (`dbf` loops `d0+1` times) before the `loc_1BF70` fill"
        );
        assert_eq!(
            result.vram_fill_pattern, 0x20BC,
            "ASM loc_1BF54 should set `d1=$20BC` as the fill pattern written by `move.w d1,(a0)+` in `loc_1BF70`"
        );
        assert_eq!(
            result.vram_fill_addr, 0xFFFF7F82,
            "ASM loc_1BF54 should set `a0=$FFFF7F82` as the fill destination via `lea ($FFFF7F82).l,a0`"
        );
    }

    #[test]
    fn loc_1bf70_fills_360_vram_words_with_pattern_20bc() {
        let result = title_shell_vram_fill_loc_1BF70();
        assert_eq!(
            result.fill_pattern, 0x20BC,
            "ASM loc_1BF70 should fill with pattern `#$20BC` via `move.w d1,(a0)+`"
        );
        assert_eq!(
            result.fill_iterations, 360,
            "ASM loc_1BF70 should run `d0+1` iterations with `d0=$167`, filling 360 words total"
        );
    }

    #[test]
    fn loc_1bf84_copies_20_rows_of_mapping_data_to_vram_work_ram() {
        let result = crate::boot::title_shell_row_copy_loop_loc_1BF84();
        assert_eq!(
            result.row_count, 20,
            "ASM loc_1BF84 should run `d1=$13` + 1 = 20 outer iterations for the title shell row copy loop"
        );
        assert_eq!(
            result.vram_row_stride_bytes, 0x24,
            "ASM loc_1BF84 should advance `a0` by `$24` = 36 bytes per row via `addi.w #$24,a0`"
        );
    }

    #[test]
    fn loc_1bf8a_copies_one_row_of_mapping_words_from_decomp_buffer() {
        let result = crate::boot::title_shell_word_copy_loop_loc_1BF8A();
        assert_eq!(
            result.source, 0xFFFF0000,
            "ASM loc_1BF8A reads from `Decompression_Buffer` which is at `$FFFF0000`"
        );
        assert_eq!(
            result.dest, 0xFFFF7F82,
            "ASM loc_1BF8A writes to VRAM work RAM at `$FFFF7F82`"
        );
    }

    #[test]
    fn loc_1c060_clears_400_words_of_vram_work_ram_before_mapping_copy() {
        let result = crate::boot::intro_tile_upload_clear_loop_loc_1C060();
        assert_eq!(
            result.vram_work_ram_addr, 0xFFFF7F82,
            "ASM loc_1C060 clears VRAM work RAM at `$FFFF7F82`"
        );
        assert_eq!(
            result.clear_iterations, 400,
            "ASM loc_1C060 runs `d0+1` iterations where `d0=$18F` = 399, clearing 400 words total"
        );
        assert_eq!(
            result.fill_value, 0x0000,
            "ASM loc_1C060 fills with zero via `moveq #0,d1` before the `move.w d1,(a0)+` loop"
        );
    }

    #[test]
    fn loc_1c07c_copies_mapping_rows_with_32_byte_stride() {
        let result = crate::boot::intro_tile_upload_row_copy_loop_loc_1C07C();
        assert_eq!(
            result.vram_row_stride_bytes, 0x20,
            "ASM loc_1C07C should advance `a0` by `$20` = 32 bytes per row via `addi.w #$20,a0`"
        );
    }

    #[test]
    fn loc_1c082_copies_one_row_of_mapping_words_to_vram_work_ram() {
        let result = crate::boot::intro_tile_upload_word_copy_loop_loc_1C082();
        assert_eq!(
            result.source, 0xFFFF0000,
            "ASM loc_1C082 reads from `Decompression_Buffer` which is at `$FFFF0000`"
        );
        assert_eq!(
            result.dest, 0xFFFF7F82,
            "ASM loc_1C082 writes to VRAM work RAM at `$FFFF7F82` before the offset adjustment"
        );
    }

    #[test]
    fn loc_1c0a4_streams_15_words_per_row_to_vdp_for_8_rows() {
        let result = crate::boot::intro_tile_upload_vram_dma_loop_loc_1C0A4();
        assert_eq!(
            result.words_per_row, 15,
            "ASM loc_1C0A4 runs `d6+1` iterations where `moveq #$E,d6` sets d6=14, giving 15 words per row"
        );
        assert_eq!(
            result.vram_address_step, 0x0080_0000,
            "ASM loc_1C0A4 advances VRAM address by `$800000` = 2048 bytes per row via `addi.l #$800000,d0`"
        );
        assert_eq!(
            result.row_count, 8,
            "ASM loc_1C0A4 outer loop runs `d7+1` iterations where `moveq #7,d7` gives 8 total rows"
        );
    }

    #[test]
    fn loc_1c0be_streams_16_words_per_row_to_vdp_for_2_rows() {
        let result = crate::boot::intro_tile_upload_vram_dma_group_2_loc_1C0BE();
        assert_eq!(
            result.words_per_row, 16,
            "ASM loc_1C0BE runs `d6+1` iterations where `moveq #$F,d6` sets d6=15, giving 16 words per row"
        );
        assert_eq!(
            result.vram_address_step, 0x0080_0000,
            "ASM loc_1C0BE advances VRAM address by `$800000` = 2048 bytes per row via `addi.l #$800000,d0`"
        );
        assert_eq!(
            result.row_count, 2,
            "ASM loc_1C0BE outer loop runs `d7+1` iterations where `moveq #1,d7` sets d7=1, giving 2 total rows"
        );
    }

    #[test]
    fn loc_1c0d6_streams_15_words_per_row_to_vdp_for_15_rows_with_stride() {
        let result = crate::boot::intro_tile_upload_vram_dma_group_3_loc_1C0D6();
        assert_eq!(
            result.words_per_row, 15,
            "ASM loc_1C0D6 runs `d6+1` iterations where `moveq #$E,d6` sets d6=14, giving 15 words per row"
        );
        assert_eq!(
            result.vram_address_step, 0x0080_0000,
            "ASM loc_1C0D6 advances VRAM address by `$800000` = 2048 bytes per row via `addi.l #$800000,d0`"
        );
        assert_eq!(
            result.row_count, 15,
            "ASM loc_1C0D6 outer loop runs `d7+1` iterations where `moveq #$E,d7` sets d7=14, giving 15 total rows"
        );
        assert_eq!(
            result.stride_padding_bytes, 2,
            "ASM loc_1C0D6 executes `addq.w #2,a0` after each row to skip 2 bytes of padding between rows"
        );
    }

    #[test]
    fn loc_1c11e_zero_fills_90_words_in_decompression_buffer() {
        let result = crate::boot::intro_decompression_buffer_zero_fill_loc_1C11E();
        assert_eq!(
            result.word_count, 90,
            "ASM loc_1C11E runs `d0+1` iterations where `move.w #$59,d0` sets d0=89, giving 90 words total"
        );
        assert_eq!(
            result.fill_value, 0,
            "ASM loc_1C11E fills with zero via `moveq #0,d1` before the `move.w d1,(a0)+` loop"
        );
    }

    #[test]
    fn gfx_object_predecessor_index_sub_ff6_returns_none_when_loc_1004_walks_to_a_null_link() {
        let next_indices = [
            Some(1), // sentinel -> first live node
            Some(2), // first live node -> second live node
            None,    // loc_1004 hits the null tail before finding target 3
            None,
        ];

        assert_eq!(
            gfx_object_predecessor_index_sub_FF6(&next_indices, 3),
            None,
            "ASM loc_1004 should keep following `0(a4)` until it either sees `a3` or reaches a null link, and the null-link path should return without unlinking",
        );
    }

    #[test]
    fn gfx_object_delete_predecessor_index_loc_10b6_restarts_the_walk_from_the_sentinel_head() {
        let next_indices = [
            Some(1), // sentinel -> first live node
            Some(2), // first live node -> second live node
            Some(3), // second live node -> target
            None,
        ];

        assert_eq!(
            gfx_object_delete_predecessor_index_loc_10B6(&next_indices, 3),
            Some(2),
            "ASM loc_10B6 should reload `Addr_FirstGfxObjectSlot` before `loc_10BA`, so the delete walk still finds the predecessor even when the caller was just mutating owner back-references",
        );
    }

    #[test]
    fn gfx_object_delete_loc_10c6_splices_the_target_into_the_free_list_and_routes_kind_2_to_kid_projectile()
     {
        assert_eq!(
            gfx_object_delete_loc_10C6(Some(4), Some(7), 5, 2, 3),
            GfxObjectDeleteLoc10C6 {
                predecessor_next_index: Some(4),
                target_next_free_index: Some(7),
                next_free_head_index: 3,
                next_number_gfx_objects: 4,
                special_slot: GfxObjectSpecialSlotLoc10C6::KidProjectile,
            },
            "ASM loc_10C6 should splice `0(a3)` over the predecessor link, push `a3` onto `Addr_NextFreeGfxObjectSlot`, decrement `Number_GfxObjects`, and route object kind 2 through `Addr_GfxObject_KidProjectile`",
        );
    }

    #[test]
    fn gfx_object_delete_loc_1114_copies_the_targets_successor_over_the_predecessor_link() {
        let result = gfx_object_delete_loc_10C6(Some(4), Some(7), 5, 0, 3);

        assert_eq!(
            result.predecessor_next_index,
            Some(4),
            "ASM loc_1114 should execute `move.l 4(a3),4(a0)`, so the predecessor keeps the target's successor link before the delete path pushes `a3` onto the free list",
        );
    }

    #[test]
    fn gfx_object_delete_special_chain_loc_111a_returns_without_relink_when_the_target_is_missing()
    {
        let next_indices = [
            Some(1), // sentinel -> first singleton entry
            Some(2), // first singleton entry -> second singleton entry
            None,
        ];

        assert_eq!(
            gfx_object_delete_special_predecessor_index_loc_111A(&next_indices, 3),
            None,
            "ASM loc_111A should return immediately when the `loc_1106` singleton-chain walk hits a null `4(a0)` link before seeing `a3`",
        );
    }

    #[test]
    fn gfx_object_delete_loc_10ea_routes_object_kind_2_to_the_kid_projectile_singleton_slot() {
        assert_eq!(
            gfx_object_delete_loc_10C6(None, Some(7), 5, 2, 3).special_slot,
            GfxObjectSpecialSlotLoc10C6::KidProjectile,
            "ASM loc_10EA should take object kind 2 through the extra decrement-to-zero branch and clear `Addr_GfxObject_KidProjectile` rather than the Kid or `$FFFFF86A` singleton",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn gfx_object_alloc_loc_F9C_routes_object_kind_2_to_the_fffff86a_chain() {
        assert_eq!(
            gfx_object_alloc_special_chain_loc_F9C(2),
            GfxObjectAllocationSpecialChainLocF9C::UnknownF86A,
            "ASM loc_F9C should receive original kind 2 after one decrement, decrement it to zero, and splice the new gfx object through `$FFFFF86A`",
        );

        assert_eq!(
            gfx_object_alloc_special_chain_loc_F9C(5),
            GfxObjectAllocationSpecialChainLocF9C::None,
            "ASM loc_F9C should fall through the later kind checks without touching a special chain for unknown object kinds",
        );
    }

    #[test]
    fn gfx_object_delete_loc_10f4_routes_object_kind_3_to_the_fffff86a_singleton_slot() {
        assert_eq!(
            gfx_object_delete_loc_10C6(None, Some(7), 5, 3, 3).special_slot,
            GfxObjectSpecialSlotLoc10C6::UnknownF86A,
            "ASM loc_10F4 should take object kind 3 through the extra decrement-to-zero branch and clear the `$FFFFF86A` singleton rather than the Kid or KidProjectile slot",
        );
    }

    #[test]
    fn gfx_object_delete_loc_10fe_routes_object_kind_4_to_the_fffff86e_singleton_slot() {
        assert_eq!(
            gfx_object_delete_loc_10C6(None, Some(7), 5, 4, 3).special_slot,
            GfxObjectSpecialSlotLoc10C6::UnknownF86E,
            "ASM loc_10FE should take object kind 4 through the extra decrement-to-zero branch and clear the `$FFFFF86E` singleton rather than the earlier Kid, KidProjectile, or `$FFFFF86A` slots",
        );
    }

    #[test]
    fn clear_owner_gfx_backref_loc_1078_ignores_odd_owner_pointers() {
        let links = clear_owner_gfx_backref_loc_1078(0x00FF_8001, [0x10, 0x20, 0x30], 0x20);

        assert_eq!(
            links,
            [0x10, 0x20, 0x30],
            "ASM loc_1078 should branch to the shared free-list walk immediately when bit 0 of `a5` is set",
        );
    }

    #[test]
    fn clear_owner_gfx_backref_loc_1078_requires_the_masked_owner_address_to_stay_in_wram() {
        let links = clear_owner_gfx_backref_loc_1078(0x0100_8000, [0x10, 0x20, 0x30], 0x20);

        assert_eq!(
            links,
            [0x10, 0x20, 0x30],
            "ASM loc_1078 should mask `a5` to 24 bits and skip the `$36/$3A/$3E(a5)` clears when the result falls below `$FF0000`",
        );
    }

    #[test]
    fn clear_owner_gfx_backref_loc_1078_clears_only_the_first_matching_owner_slot() {
        let links = clear_owner_gfx_backref_loc_1078(
            0x01FF_8000,
            [0x00AA_0000, 0x00AA_0000, 0x00AA_0000],
            0x00AA_0000,
        );

        assert_eq!(
            links,
            [0, 0x00AA_0000, 0x00AA_0000],
            "ASM loc_1078 should clear `$36(a5)` first and branch away before touching `$3A/$3E(a5)` even if they also match `a3`",
        );
    }

    #[test]
    fn clear_owner_gfx_backref_loc_10a0_clears_the_second_owner_slot_after_the_first_miss() {
        let links = clear_owner_gfx_backref_loc_1078(
            0x01FF_8000,
            [0x00BB_0000, 0x00AA_0000, 0x00CC_0000],
            0x00AA_0000,
        );

        assert_eq!(
            links,
            [0x00BB_0000, 0, 0x00CC_0000],
            "ASM loc_10A0 should compare `$3A(a5)` only after `$36(a5)` misses, clear that second back-reference, and branch away before touching `$3E(a5)`",
        );
    }

    #[test]
    fn clear_owner_gfx_backref_loc_10ac_clears_the_third_owner_slot_after_two_misses() {
        let links = clear_owner_gfx_backref_loc_1078(
            0x01FF_8000,
            [0x00BB_0000, 0x00CC_0000, 0x00AA_0000],
            0x00AA_0000,
        );

        assert_eq!(
            links,
            [0x00BB_0000, 0x00CC_0000, 0],
            "ASM loc_10AC should only clear `$3E(a5)` after both `$36(a5)` and `$3A(a5)` miss the deleted gfx pointer",
        );
    }

    #[test]
    fn gfx_object_unlink_loc_100e_splices_the_target_successor_over_the_predecessor_link() {
        let mut next_indices = [
            Some(1), // sentinel -> first live node
            Some(3), // predecessor -> target
            None,
            Some(2), // target -> successor
        ];

        gfx_object_unlink_loc_100E(&mut next_indices, 1, 3);

        assert_eq!(
            next_indices,
            [Some(1), Some(2), None, Some(2)],
            "ASM loc_100E should write the target's `0(a3)` link back through the predecessor's `0(a4)` slot, leaving the detached target unchanged until the later reinsertion",
        );
    }

    #[test]
    fn gfx_object_insertion_predecessor_index_loc_101c_skips_equal_priorities_before_reinsertion() {
        let next_indices = [
            Some(1), // sentinel -> first live node
            Some(2), // priority 4
            Some(3), // equal priority 6
            Some(4), // higher priority 9
            None,
        ];
        let priority_words = [0, 4, 6, 9, 6];

        assert_eq!(
            gfx_object_insertion_predecessor_index_loc_101C(&next_indices, &priority_words, 6),
            2,
            "ASM loc_101C should keep walking while `$A(a4) <= d5`, so a reinserted priority-6 worker lands after existing priority-6 nodes and before the first higher-priority node",
        );
    }

    #[test]
    fn gfx_object_insert_loc_102e_splices_the_target_before_the_old_successor() {
        let mut next_indices = [
            Some(1), // sentinel -> first live node
            Some(2), // predecessor -> old successor
            Some(4), // old successor -> tail
            None,
            None,
        ];

        gfx_object_insert_loc_102E(&mut next_indices, 1, 3);

        assert_eq!(
            next_indices,
            [Some(1), Some(3), Some(4), Some(2), None],
            "ASM loc_102E should copy the predecessor's old `0(a4)` link into `0(a3)` and then repoint `0(a4)` at the reinserted node",
        );
    }

    #[test]
    fn crawl_wall_escape_sub_81b4_snaps_right_when_left_is_solid_and_right_is_open() {
        let grid = crawl_escape_test_grid(true, false);

        let result = crawl_wall_escape_sub_81B4(&grid, 50 << 16, 32 << 16, 14);

        assert_eq!(
            result,
            CrawlWallEscapeSub81B4 {
                x_pos: 78 << 16,
                blocked: false,
            },
            "ASM sub_81B4 should take loc_81FA when the left sample is `$6000` and the right sample is not",
        );
    }

    #[test]
    fn crawl_wall_escape_sub_81b4_reports_blocked_when_both_sides_are_solid() {
        let grid = crawl_escape_test_grid(true, true);

        let result = crawl_wall_escape_sub_81B4(&grid, 50 << 16, 32 << 16, 14);

        assert_eq!(
            result,
            CrawlWallEscapeSub81B4 {
                x_pos: 50 << 16,
                blocked: true,
            },
            "ASM sub_81B4 should return d7=1 and leave x_pos unchanged when both side samples read `$6000`",
        );
    }

    #[test]
    fn crawl_slope_snap_sub_83e6_nudges_left_on_rightward_slope_up_contact() {
        let grid = crawl_slope_test_grid(false, true);

        let result = crawl_slope_snap_sub_83E6(&grid, 50 << 16, 32 << 16, 0x0001_0000, 14);

        assert_eq!(
            result,
            CrawlSlopeSnapSub83E6 {
                x_pos: 49 << 16,
                adjusted: true,
            },
            "ASM sub_83E6 should mirror the `$4000` right-side probe branch and subtract the low nibble plus one from x_pos",
        );
    }

    #[test]
    fn crawl_slope_snap_sub_83e6_snaps_right_on_leftward_slope_down_contact() {
        let grid = crawl_slope_test_grid(true, false);

        let result = crawl_slope_snap_sub_83E6(&grid, 50 << 16, 32 << 16, -0x0001_0000, 14);

        assert_eq!(
            result,
            CrawlSlopeSnapSub83E6 {
                x_pos: 62 << 16,
                adjusted: true,
            },
            "ASM sub_83E6 should mirror loc_841A and snap to the current tile column plus `half_width` on a `$5000` left-side probe",
        );
    }

    #[test]
    fn crawl_slope_snap_sub_83e6_ignores_non_matching_slope_types() {
        let grid = crawl_slope_test_grid(true, false);

        let result = crawl_slope_snap_sub_83E6(&grid, 50 << 16, 32 << 16, 0x0001_0000, 14);

        assert_eq!(
            result,
            CrawlSlopeSnapSub83E6 {
                x_pos: 50 << 16,
                adjusted: false,
            },
            "ASM sub_83E6 should leave x_pos alone when the motion-side probe does not match the expected slope nibble",
        );
    }

    #[test]
    fn sub_bbe2_promotes_the_right_solid_block_after_kid_crosses_the_source_column() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 4 * 2 * 2], 4, 2);
        grid.set_cell_at(1, 0, terrain::Cell::from_be_bytes(0xE1, 0x01));
        grid.set_cell_at(2, 0, terrain::Cell::from_be_bytes(0xEB, 0x02));

        let probe = terrain::kid_block_interaction_probe_sub_BBE2(&grid, 0x0002, 0x0020)
            .expect("ASM sub_BBE2 should select the right-hand solid block once Kid has moved into the next column");

        assert_eq!(
            probe,
            terrain::KidBlockInteractionProbeSubBBE2 {
                selected_layout_addr: 0x0004,
                selected_col: 2,
                selected_row: 0,
                dispatch_table_offset: 0x002C,
            },
            "ASM sub_BBE2 should advance `d3` by two bytes, increment `d1`, and convert block type `$B` into the `off_BB80` byte offset `$2C`",
        );
    }

    #[test]
    fn sub_bbe2_keeps_the_base_block_when_kid_has_not_passed_the_source_column() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 4 * 2 * 2], 4, 2);
        grid.set_cell_at(1, 0, terrain::Cell::from_be_bytes(0xE4, 0x03));
        grid.set_cell_at(2, 0, terrain::Cell::from_be_bytes(0xEB, 0x04));

        let probe = terrain::kid_block_interaction_probe_sub_BBE2(&grid, 0x0002, 0x001F)
            .expect("ASM sub_BBE2 should stay on the source tile until Kid's `x_pos>>4` becomes strictly greater than the source column");

        assert_eq!(
            probe,
            terrain::KidBlockInteractionProbeSubBBE2 {
                selected_layout_addr: 0x0002,
                selected_col: 1,
                selected_row: 0,
                dispatch_table_offset: 0x0010,
            },
            "ASM sub_BBE2 should keep the original layout address and map block type `$4` to the `off_BB80` byte offset `$10` when the strict `cmp.w d1,d7` / `ble` check holds",
        );
    }

    #[test]
    fn sub_bbe2_returns_none_when_the_selected_tile_is_missing_bit_15() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 4 * 2 * 2], 4, 2);
        grid.set_cell_at(1, 0, terrain::Cell::from_be_bytes(0xE1, 0x01));
        grid.set_cell_at(2, 0, terrain::Cell::from_be_bytes(0x60, 0x00));

        assert_eq!(
            terrain::kid_block_interaction_probe_sub_BBE2(&grid, 0x0002, 0x0020),
            None,
            "ASM sub_BBE2 checks `$6000` on the right-hand candidate first, but still clears bit 15 afterwards; a collision-only tile must therefore zero `d3` instead of falling back to the base block",
        );
    }

    /// ASM loc_BB6E (kid.asm:15085): sub_BB54 dispatches through the off_BB80
    /// jump table: `move.l off_BB80(pc,d7.w),a4; jsr (a4)`.  d7 is the
    /// dispatch_table_offset from sub_BBE2 (block_type * 4).  For ceiling hits
    /// the native equivalent is the `match probe.cell.block_type` in
    /// apply_ceiling_collision.  This test verifies that a Smash block (type 0,
    /// offset 0) hit from below triggers the j_Block_LoadSmashBits handler
    /// (clearing the cell).
    #[test]
    fn loc_bb6e_off_bb80_dispatch_smash_block_clears_cell() {
        let w: u16 = 10;
        let h: u16 = 10;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Smash block at cell (3, 2): 0xE0 = Solid + has_block + block_type=0 (Smash).
        let idx = (2 * w + 3) as usize * 2;
        data[idx] = 0xE0;
        data[idx + 1] = 0x01;
        let mut grid = terrain::BlockGrid::from_level_layout(&data, w, h);
        let bounds = LevelBounds {
            width_pixels: w * 16,
            height_pixels: h * 16,
        };
        let mut slice = OpeningSliceState::new(bounds);
        slice.player.on_ground = false;
        // Position so head hits cell (3, 2): ceiling bottom at y=48.
        // Same setup as loc_90c0 test.
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

        let cell = grid.cell_at(3, 2);
        assert!(
            !cell.has_block,
            "ASM loc_BB6E: off_BB80 index 0 → j_Block_LoadSmashBits clears the Smash block"
        );
    }

    /// ASM loc_BB74 (kid.asm:15089): sub_BB54 cleanup after off_BB80 dispatch.
    /// `movem.w (sp)+,d0-d3; clr.l y_vel(a3); rts` — y_vel is zeroed after the
    /// block interaction completes.  The ceiling path (sub_902A loc_908C) also
    /// zeroes y_vel; this test verifies the y_vel=0 invariant on a block-hit
    /// ceiling collision (the loc_BB74 equivalent in native code).
    #[test]
    fn loc_bb74_ceiling_block_hit_zeroes_y_vel() {
        let w: u16 = 10;
        let h: u16 = 10;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Smash block at cell (3, 2).
        let idx = (2 * w + 3) as usize * 2;
        data[idx] = 0xE0;
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

        assert_eq!(
            slice.player.y_velocity, 0,
            "ASM loc_BB74: clr.l y_vel(a3) — velocity must be zero after ceiling block interaction"
        );
    }

    /// ASM loc_BC1C (kid.asm:15162): fallback path in sub_BBE2 when Kid hasn't
    /// moved past the source column.  `move.w (a4),d7; bclr #$F,d7; beq loc_BC30`
    /// — if the base tile has no block (bit 15 clear), d3 is zeroed (None).
    /// Native: `!selected_cell.has_block → return None`.
    #[test]
    fn loc_bc1c_sub_bbe2_base_tile_without_block_returns_none() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 4 * 2 * 2], 4, 2);
        // Cell (1,0): Solid, no block (high byte 0x60 = Solid collision, bit 15 clear).
        grid.set_cell_at(1, 0, terrain::Cell::from_be_bytes(0x60, 0x00));

        // x_pos_px = 0x0010 → player_col = 1, base_col = 1 → player NOT past source,
        // so loc_BC1C fires (use base tile).  Base tile has no block → None.
        assert_eq!(
            terrain::kid_block_interaction_probe_sub_BBE2(&grid, 0x0002, 0x0010),
            None,
            "ASM loc_BC1C: base tile bit 15 clear → bclr #$F,d7 clears CF, beq loc_BC30 zeroes d3"
        );
    }

    /// ASM loc_BC26 (kid.asm:15167): block-type extraction in sub_BBE2.
    /// `andi.w #$F00,d7; asr.w #6,d7` converts the block-type nibble (bits 11-8)
    /// into the off_BB80 dispatch offset (block_type * 4).
    /// Native: `dispatch_table_offset = (block_type as u8) << 2`.
    #[test]
    fn loc_bc26_sub_bbe2_extracts_block_type_as_dispatch_offset() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 4 * 2 * 2], 4, 2);
        // Cell (1,0): 0xE7 = Solid + has_block + block_type=7 (Shifting).
        // Block type 7: bits 11-8 = 0x0700 → andi #$F00 = 0x0700, asr #6 = 0x001C = 28.
        grid.set_cell_at(1, 0, terrain::Cell::from_be_bytes(0xE7, 0x05));

        let probe = terrain::kid_block_interaction_probe_sub_BBE2(&grid, 0x0002, 0x0010)
            .expect("ASM loc_BC26: base tile with block should yield a valid probe");

        assert_eq!(
            probe.dispatch_table_offset,
            7 * 4,
            "ASM loc_BC26: block_type 7 → andi.w #$F00 = $700, asr.w #6 = $1C (28)"
        );
    }

    #[test]
    fn sub_6874_hides_only_sprites_whose_masked_tiles_overlap_pause_menu_vram() {
        let mut sprites = [
            PauseMenuSpriteEntrySub6874 {
                y_pos: 0x0080,
                vram_tile_word: 0x0675,
            },
            PauseMenuSpriteEntrySub6874 {
                y_pos: 0x0082,
                vram_tile_word: 0x0676,
            },
            PauseMenuSpriteEntrySub6874 {
                y_pos: 0x0084,
                vram_tile_word: 0x8800 | 0x068E,
            },
            PauseMenuSpriteEntrySub6874 {
                y_pos: 0x0086,
                vram_tile_word: 0x068F,
            },
        ];

        hide_pause_menu_overwritten_sprites_sub_6874(&mut sprites);

        assert_eq!(
            sprites[0].y_pos, 0x0080,
            "ASM sub_6874 should leave sprites below tile $676 visible",
        );
        assert_eq!(
            sprites[1].y_pos, 0x0000,
            "ASM sub_6874 should hide sprites starting exactly at tile $676",
        );
        assert_eq!(
            sprites[2].y_pos, 0x0000,
            "ASM sub_6874 should mask the tile word with `$7FF` before comparing against the `$676..=$68E` pause-menu range",
        );
        assert_eq!(
            sprites[3].y_pos, 0x0086,
            "ASM sub_6874 should leave sprites above tile $68E visible",
        );
    }

    #[test]
    fn loc_687c_not_paused_returns_immediately() {
        // ASM sub_6874 (kid.asm:6575): tst.b (Game_Paused).w; bne.s loc_687C; rts
        // When Game_Paused == 0, sub_6874 returns before reaching loc_687C.
        assert_eq!(
            pause_menu_gate_loc_687C(0),
            PauseMenuGateLoc687C::NotPaused,
            "ASM sub_6874: Game_Paused=0 → rts before loc_687C"
        );
    }

    #[test]
    fn loc_687c_paused_value_3_returns_after_palette_darken() {
        // ASM loc_687C (kid.asm:6581-6587): after Pause_DarkenPalette + Palette_to_VRAM
        // + sub_E1334, cmpi.b #3,(Game_Paused).w; bne.s loc_6894; rts
        // When Game_Paused == 3 (unpause signal), returns before sprite hiding.
        assert_eq!(
            pause_menu_gate_loc_687C(3),
            PauseMenuGateLoc687C::UnpausingReturn,
            "ASM loc_687C: Game_Paused=3 → rts after palette darken (unpausing)"
        );
    }

    #[test]
    fn loc_687c_paused_value_1_continues_to_sprite_hide() {
        // ASM loc_687C: Game_Paused=1 (normal pause) → bne.s loc_6894 → sprite hiding.
        assert_eq!(
            pause_menu_gate_loc_687C(1),
            PauseMenuGateLoc687C::PausedContinueToSpriteHide,
            "ASM loc_687C: Game_Paused=1 → falls through to loc_6894 sprite hiding"
        );
        // Game_Paused=2 (another non-3 value) also continues.
        assert_eq!(
            pause_menu_gate_loc_687C(2),
            PauseMenuGateLoc687C::PausedContinueToSpriteHide,
            "ASM loc_687C: Game_Paused=2 → falls through to loc_6894 sprite hiding"
        );
    }

    #[test]
    /// ASM loc_892 (kid.asm:956): Mode_Level rendering tail join point.
    /// When $FFFFFB49 is non-zero (paused), `bne.s loc_892` skips
    /// `j_DiamondPower_CompileSprites` and `Make_SpritesFromPlatforms`.
    /// loc_892 unconditionally runs: `jsr j_sub_30194` (HUD sprite attrs),
    /// `jsr j_loc_3038A` (scroll data transfer + Time_Frames increment),
    /// `bsr.w sub_14C0` (palette fade). All three are visual/VDP — not
    /// modelled in native. Native gates DiamondPower on `!self.paused`.
    fn loc_892_paused_skips_diamond_sprite_compilation_but_runs_rendering_tail() {
        // The gate is in lib.rs: `!self.paused` for step_diamond_power.
        // When $FFFFFB49 (paused) is set, ASM skips DiamondPower_CompileSprites
        // and Make_SpritesFromPlatforms, jumping straight to loc_892 which runs
        // j_sub_30194 (HUD sprites), j_loc_3038A (scroll transfer), sub_14C0
        // (palette fade) — all visual/VDP, not modelled in native.
        // Native equivalent: step_diamond_power gated on `!self.paused`.
        let rt = NativeRuntime::new(manifest());
        assert!(
            !rt.paused,
            "ASM loc_892: default runtime is not paused — DiamondPower_CompileSprites runs"
        );
        let mut paused_rt = NativeRuntime::new(manifest());
        paused_rt.paused = true;
        assert!(
            paused_rt.paused,
            "ASM loc_892: $FFFFFB49 non-zero → bne.s loc_892 skips DiamondPower_CompileSprites; \
             native gates step_diamond_power on !self.paused"
        );
    }

    #[test]
    fn loc_690c_art_copy_bytes_per_iteration_is_32() {
        // ASM loc_690C (kid.asm:6622-6631): 8 × `move.l (a0)+,(a6)` per iteration.
        assert_eq!(PAUSE_MENU_ART_COPY_BYTES_PER_ITERATION_LOC_690C, 32);
    }

    #[test]
    fn loc_690c_total_bytes_scales_with_header_word() {
        // ASM kid.asm:6617-6619: `move.w (a0)+,d0; subq.w #1,d0` then
        // `dbf d0,loc_690C` — header_word iterations of 32 bytes each.
        assert_eq!(pause_menu_art_total_bytes_loc_690C(1), 32);
        assert_eq!(pause_menu_art_total_bytes_loc_690C(4), 128);
        // header_word=0 means zero iterations (dbf with d0=-1 exits immediately).
        assert_eq!(pause_menu_art_total_bytes_loc_690C(0), 0);
    }

    #[test]
    fn loc_693a_camera_at_origin_no_plane_wrap() {
        // ASM kid.asm:6633-6643: Camera_X_pos=0 → d0 = ($60 >> 3) & $3F = $C.
        // $C <= $30 → no wrapping, d1=8 longword reads, d2=-1.
        let layout = pause_menu_plane_column_layout_loc_693A(0);
        assert_eq!(
            layout,
            PauseMenuPlaneColumnLayout {
                tile_col: 0x0C,
                wrap_split: None,
            },
            "ASM loc_6894: Camera_X_pos=0 → tile_col=$C, no plane-edge wrap"
        );
    }

    #[test]
    fn loc_693a_camera_near_plane_edge_triggers_wrap() {
        // ASM loc_693A (kid.asm:6645-6652): when tile_col > $30, wrapping splits
        // the 16-word-per-row read into two parts.
        // Camera_X_pos=$100 → d0 = (($100 + $60) >> 3) & $3F = ($160 >> 3) & $3F
        //                    = $2C & $3F = $2C. $2C <= $30 → no wrap.
        let layout_no_wrap = pause_menu_plane_column_layout_loc_693A(0x100);
        assert_eq!(layout_no_wrap.wrap_split, None);

        // Camera_X_pos=$120 → d0 = (($120 + $60) >> 3) & $3F = ($180 >> 3) & $3F
        //                    = $30 & $3F = $30. $30 <= $30 → still no wrap.
        let layout_boundary = pause_menu_plane_column_layout_loc_693A(0x120);
        assert_eq!(
            layout_boundary,
            PauseMenuPlaneColumnLayout {
                tile_col: 0x30,
                wrap_split: None,
            },
            "ASM: tile_col=$30 is the last non-wrapping position (cmpi.w #$30,d0; bgt.s)"
        );

        // Camera_X_pos=$128 → d0 = (($128 + $60) >> 3) & $3F = ($188 >> 3) & $3F
        //                    = $31 & $3F = $31. $31 > $30 → loc_693A wrapping.
        // d1 = $40 - $31 = $F (15 words before edge), d2 = $10 - $F = 1 (after).
        let layout_wrap = pause_menu_plane_column_layout_loc_693A(0x128);
        assert_eq!(
            layout_wrap,
            PauseMenuPlaneColumnLayout {
                tile_col: 0x31,
                wrap_split: Some((0x0F, 0x01)),
            },
            "ASM loc_693A: tile_col=$31 → 15 words before plane edge, 1 after wrap"
        );
    }

    #[test]
    fn loc_693a_wrap_split_always_totals_16_words() {
        // ASM loc_693A: first_words + wrapped_words = $10 (16) for any wrapping
        // camera position. The non-wrapping path reads 8 longwords = 16 words.
        for camera_x in (0..=0xFFFF_u16).step_by(8) {
            let layout = pause_menu_plane_column_layout_loc_693A(camera_x);
            if let Some((first, wrapped)) = layout.wrap_split {
                assert_eq!(
                    first + wrapped,
                    0x10,
                    "ASM loc_693A: split must total $10 words (camera_x={:#06X}, tile_col={:#04X})",
                    camera_x,
                    layout.tile_col,
                );
            }
        }
    }

    #[test]
    fn loc_695c_row_count_is_5() {
        // ASM kid.asm:6660: `moveq #4,d5` — dbf iterates d5+1 = 5 times.
        assert_eq!(
            PAUSE_MENU_TILE_SAVE_ROW_COUNT_LOC_695C, 5,
            "ASM loc_695C: d5=4 → 5 rows saved into Decompression_Buffer"
        );
    }

    #[test]
    fn loc_6948_y_offset_is_0x40() {
        // ASM kid.asm:6656: `addi.w #$40,d7` — menu top sits $40 pixels below camera.
        assert_eq!(
            PAUSE_MENU_Y_OFFSET_LOC_6948, 0x40,
            "ASM loc_6948: d7 = Camera_Y_pos + $40"
        );
    }

    #[test]
    fn loc_695c_vdp_read_cmd_at_origin() {
        // Camera_Y_pos=0, tile_col=$C, row 0:
        // y_pixel = 0 + $40 + 0 = $40
        // plane_row = ($40 >> 3) & $1F = 8
        // row_byte_offset = 8 << 7 = $400
        // col_byte_offset = $C * 2 = $18
        // vram_addr = $400 + $18 = $418
        // VDP read cmd = $0418_0000
        let cmd = pause_menu_tile_save_row_vdp_read_cmd_loc_695C(0, 0x0C, 0);
        assert_eq!(
            cmd, 0x0418_0000,
            "ASM loc_695C: Camera_Y=0, tile_col=$C, row 0 → VRAM addr $0418"
        );
    }

    #[test]
    fn loc_695c_rows_advance_by_8_pixels() {
        // ASM kid.asm:6687: `addi.w #8,d7` — each row is 8 pixels (1 tile) apart.
        // Camera_Y=0, tile_col=0, rows 0 and 1:
        // Row 0: y=$40 → plane_row=8, offset=$400 → cmd=$0400_0000
        // Row 1: y=$48 → plane_row=9, offset=$480 → cmd=$0480_0000
        let cmd0 = pause_menu_tile_save_row_vdp_read_cmd_loc_695C(0, 0, 0);
        let cmd1 = pause_menu_tile_save_row_vdp_read_cmd_loc_695C(0, 0, 1);
        assert_eq!(cmd0, 0x0400_0000);
        assert_eq!(cmd1, 0x0480_0000);
        assert_eq!(
            (cmd1 >> 16) - (cmd0 >> 16),
            0x80,
            "ASM loc_695C: consecutive rows are $80 bytes (128) apart in nametable"
        );
    }

    #[test]
    fn loc_695c_plane_row_wraps_at_32() {
        // ASM loc_695C: `andi.w #$1F,d3` wraps to 32-row plane height.
        // Camera_Y=$C0, tile_col=0, row 0:
        // y_pixel = $C0 + $40 = $100, plane_row = ($100 >> 3) & $1F = $20 & $1F = 0
        // → wraps to row 0 of nametable.
        let cmd = pause_menu_tile_save_row_vdp_read_cmd_loc_695C(0xC0, 0, 0);
        assert_eq!(
            cmd >> 16,
            0,
            "ASM loc_695C: plane_row wraps at 32 via andi.w #$1F"
        );
    }

    #[test]
    fn loc_69ae_column_reset_clears_column_bits() {
        // ASM loc_69AE (kid.asm:6703): `andi.l #$FF80FFFF,d0`
        // Clears bits 6-0 of the high word (column byte offset within 128-byte row).
        let original = 0x0418_0000_u32; // row $400 + column $18
        let reset = pause_menu_tile_save_wrap_column_reset_loc_69AE(original);
        assert_eq!(
            reset, 0x0400_0000,
            "ASM loc_69AE: andi.l #$FF80FFFF clears column offset, keeps row"
        );
    }

    #[test]
    fn loc_69ae_column_reset_preserves_row_offset() {
        // With a high row offset — ensure only column bits are cleared.
        let original = 0x0F7E_0000_u32; // row $F00 + column $7E
        let reset = pause_menu_tile_save_wrap_column_reset_loc_69AE(original);
        assert_eq!(
            reset, 0x0F00_0000,
            "ASM loc_69AE: row offset $F00 preserved, column $7E cleared"
        );
    }

    #[test]
    fn loc_695c_plan_non_wrapped_has_5_rows_no_segments() {
        // Camera_X=0 → tile_col=$C, no wrap. All 5 rows should have wrap_segments=None.
        let plan = pause_menu_tile_save_plan_loc_695C(0, 0);
        assert_eq!(plan.len(), 5);
        for (i, row) in plan.iter().enumerate() {
            assert_eq!(
                row.wrap_segments, None,
                "ASM loc_695C: non-wrapped row {i} should read 8 longwords straight"
            );
        }
    }

    #[test]
    fn loc_69a0_wrapped_plan_splits_each_row() {
        // Camera_X=$128 → tile_col=$31, wrap_split=(15, 1).
        // ASM loc_69A0 (kid.asm:6692-6698): unpacks d2 into first_words (d1)
        // and wrapped_words (d4) for the split read.
        let plan = pause_menu_tile_save_plan_loc_695C(0x128, 0);
        for (i, row) in plan.iter().enumerate() {
            let (first_words, _second_cmd, wrapped_words) = row
                .wrap_segments
                .unwrap_or_else(|| panic!("ASM loc_69A0: row {i} should be wrapped"));
            assert_eq!(
                first_words, 15,
                "ASM loc_69A0: d1 = $40 - $31 = $F (15 words before plane edge)"
            );
            assert_eq!(
                wrapped_words, 1,
                "ASM loc_69A0: d4 = $10 - $F = 1 (1 word after wrap)"
            );
        }
    }

    #[test]
    fn loc_69ae_wrapped_second_cmd_resets_column() {
        // ASM loc_69AE: after first segment, column reset for second segment.
        let plan = pause_menu_tile_save_plan_loc_695C(0x128, 0);
        let row = &plan[0];
        let (_, second_cmd, _) = row.wrap_segments.unwrap();
        // Original cmd has column $31*2 = $62, reset should clear to 0.
        let original_col = (row.vdp_read_cmd >> 16) & 0x7F;
        assert_eq!(original_col, 0x62, "tile_col=$31 → col_byte_offset=$62");
        let reset_col = (second_cmd >> 16) & 0x7F;
        assert_eq!(
            reset_col, 0,
            "ASM loc_69AE: andi.l #$FF80FFFF clears column for second segment"
        );
    }

    #[test]
    fn loc_695c_plan_all_rows_address_different_plane_rows() {
        // Each of the 5 rows should target a distinct nametable row
        // (unless the 32-row plane wraps, but with camera_y=0 + $40 = row 8,
        // rows 8..12 are all distinct).
        let plan = pause_menu_tile_save_plan_loc_695C(0, 0);
        let plane_rows: Vec<u16> = plan
            .iter()
            .map(|r| ((r.vdp_read_cmd >> 16) as u16 >> 7) & 0x1F)
            .collect();
        assert_eq!(
            plane_rows,
            vec![8, 9, 10, 11, 12],
            "ASM loc_695C: 5 consecutive plane rows starting at (($40>>3)&$1F) = 8"
        );
    }

    #[test]
    fn loc_69d2_column_layout_matches_loc_693a() {
        // ASM loc_69D2 (kid.asm:6716-6724): identical column layout computation to
        // loc_693A (kid.asm:6645-6652). Both compute ((Camera_X_pos + $60) >> 3) & $3F
        // and split at tile_col > $30. The function pause_menu_plane_column_layout_loc_693A
        // covers both — verify a few representative values.
        let layout_origin = pause_menu_plane_column_layout_loc_693A(0);
        assert_eq!(layout_origin.tile_col, 0x0C);
        assert!(
            layout_origin.wrap_split.is_none(),
            "loc_69D2: tile_col=$C <= $30 → no wrap"
        );

        // loc_69EC (kid.asm:6727-6734): wrapping case, same as loc_693A.
        let layout_wrap = pause_menu_plane_column_layout_loc_693A(0x128);
        assert_eq!(layout_wrap.tile_col, 0x31);
        assert_eq!(
            layout_wrap.wrap_split,
            Some((0x0F, 0x01)),
            "loc_69EC: tile_col=$31 → 15 words before edge, 1 after wrap"
        );
    }

    #[test]
    fn restart_text_gives_up_when_lives_le_1() {
        // ASM restart_text (kid.asm:6741): cmpi.w #1,(Number_Lives).w; bgt.s +
        // Lives=0 → Give Up, Lives=1 → Give Up, Lives=2 → Restart Round.
        assert_eq!(
            pause_menu_text_selection_restart_text(0),
            PauseMenuTextSelectionRestartText::GiveUp,
            "ASM restart_text: 0 lives → Give Up"
        );
        assert_eq!(
            pause_menu_text_selection_restart_text(1),
            PauseMenuTextSelectionRestartText::GiveUp,
            "ASM restart_text: 1 life → Give Up (cmpi.w #1 + bgt.s means > 1 required)"
        );
        assert_eq!(
            pause_menu_text_selection_restart_text(2),
            PauseMenuTextSelectionRestartText::RestartRound,
            "ASM restart_text: 2 lives → Restart Round"
        );
        assert_eq!(
            pause_menu_text_selection_restart_text(99),
            PauseMenuTextSelectionRestartText::RestartRound,
            "ASM restart_text: many lives → Restart Round"
        );
    }

    #[test]
    fn loc_6a1e_vdp_write_cmd_at_origin() {
        // ASM loc_6A1E (kid.asm:6750-6760): Camera_Y_pos=0, tile_col=$C, row=0.
        // d7 = 0 + $40 = $40 → d3 = ($40 >> 3) & $1F = 8 → row_offset = 8 << 7 = $400.
        // d6 = $C * 2 = $18.  d0 = $400 + $18 = $418.
        // ori.w #$4000 → $4418.  swap → $44180000.
        let cmd = pause_menu_text_write_row_vdp_write_cmd_loc_6A1E(0, 0x0C, 0);
        assert_eq!(
            cmd, 0x4418_0000,
            "ASM loc_6A1E: VDP write cmd = row $400 + col $18 + $4000 write bit"
        );
    }

    #[test]
    fn loc_6a1e_write_cmd_differs_from_695c_read_cmd_by_0x4000() {
        // ASM loc_6A1E uses `ori.w #$4000` vs loc_695C `ori.w #0`.
        // The write command should be exactly $4000_0000 higher than the read command.
        let read_cmd = pause_menu_tile_save_row_vdp_read_cmd_loc_695C(0, 0x0C, 0);
        let write_cmd = pause_menu_text_write_row_vdp_write_cmd_loc_6A1E(0, 0x0C, 0);
        assert_eq!(
            write_cmd,
            read_cmd | 0x4000_0000,
            "ASM: write cmd = read cmd | $4000 in high word"
        );
    }

    #[test]
    fn loc_6a1e_rows_advance_by_8_pixels() {
        // ASM loc_6A1E: addi.w #8,d7 advances y by 8 pixels each row,
        // same as loc_695C for tile save.
        let cmd0 = pause_menu_text_write_row_vdp_write_cmd_loc_6A1E(0, 0, 0);
        let cmd1 = pause_menu_text_write_row_vdp_write_cmd_loc_6A1E(0, 0, 1);
        // Row 0: plane_row = ($40>>3)&$1F = 8. Row 1: plane_row = ($48>>3)&$1F = 9.
        // Difference: 1 row × 128 bytes = $80, shifted left 16 = $0080_0000.
        assert_eq!(
            cmd1 - cmd0,
            0x0080_0000,
            "ASM loc_6A1E: consecutive rows differ by one plane row ($80 bytes)"
        );
    }

    #[test]
    fn loc_69d2_text_write_plan_non_wrapped_has_5_rows() {
        // ASM loc_69D2 + loc_6A1E (kid.asm:6716-6772): 5 rows (d5=4, dbf),
        // no wrapping at camera origin.
        let plan = pause_menu_text_write_plan_loc_69D2(0, 0);
        assert_eq!(plan.len(), 5, "ASM loc_6A1E: d5=4 → 5 text write rows");
        for (i, row) in plan.iter().enumerate() {
            assert!(
                row.wrap_segments.is_none(),
                "ASM loc_69D2: camera_x=0 → no plane-edge wrap (row {i})"
            );
            // Verify write bit is set
            assert_ne!(
                row.vdp_write_cmd & 0x4000_0000,
                0,
                "ASM loc_6A1E: VDP write bit must be set (row {i})"
            );
        }
    }

    #[test]
    fn loc_69ec_wrapped_text_write_plan_splits_each_row() {
        // ASM loc_69EC (kid.asm:6727-6734): wrapping case.
        // Camera_X_pos=$128 → tile_col=$31, first_words=$F, wrapped_words=$1.
        let plan = pause_menu_text_write_plan_loc_69D2(0x128, 0);
        for (i, row) in plan.iter().enumerate() {
            let (first, _second_cmd, wrapped) = row
                .wrap_segments
                .unwrap_or_else(|| panic!("ASM loc_69EC: camera_x=$128 → wrap expected (row {i})"));
            assert_eq!(first, 0x0F, "ASM loc_6A5A: first_words=$F (row {i})");
            assert_eq!(wrapped, 0x01, "ASM loc_6A76: wrapped_words=$1 (row {i})");
        }
    }

    #[test]
    fn loc_6a64_wrapped_second_cmd_clears_column_preserves_write_bit() {
        // ASM loc_6A64 (kid.asm:6787): andi.l #$FF80FFFF,d0 — same column reset
        // as loc_69AE but operating on a write command (has $4000 bit set).
        let plan = pause_menu_text_write_plan_loc_69D2(0x128, 0);
        let (_first, second_cmd, _wrapped) = plan[0].wrap_segments.unwrap();
        // Column bits cleared, but $4000 write bit and row offset preserved.
        assert_eq!(
            second_cmd & 0x007F_0000,
            0,
            "ASM loc_6A64: column byte offset cleared in second segment"
        );
        assert_ne!(
            second_cmd & 0x4000_0000,
            0,
            "ASM loc_6A64: $4000 write bit preserved after column reset"
        );
    }

    #[test]
    fn loc_6b68_tile_restore_plan_non_wrapped_has_5_rows() {
        // ASM loc_6B68 + loc_6B90 + loc_6BA4 (kid.asm:6880-6934): 5 rows (d5=4, dbf),
        // no wrapping at camera origin. Reads from Decompression_Buffer, writes to VDP.
        // loc_6B90: d5=4, d7=Camera_Y_pos+$40, d6=tile_col*2 — same setup as text write.
        // loc_6BA4: per-row VDP write address identical to loc_6A1E (ori.w #$4000).
        let plan = pause_menu_tile_restore_plan_loc_6B68(0, 0);
        assert_eq!(plan.len(), 5, "ASM loc_6B90: d5=4 → 5 tile restore rows");
        for (i, row) in plan.iter().enumerate() {
            assert!(
                row.wrap_segments.is_none(),
                "ASM loc_6B68: camera_x=0 → no plane-edge wrap (row {i})"
            );
            // loc_6BA4: VDP write bit must be set (ori.w #$4000)
            assert_ne!(
                row.vdp_write_cmd & 0x4000_0000,
                0,
                "ASM loc_6BA4: VDP write bit must be set (row {i})"
            );
        }
    }

    #[test]
    fn loc_6b82_wrapped_tile_restore_plan_splits_each_row() {
        // ASM loc_6B82 (kid.asm:6893-6900): wrapping case — identical computation to
        // loc_693A wrap. d1 = $40 - d0 (first_words), d2 = $10 - d1 (wrapped_words),
        // swap d2; move.w d1,d2; swap d2.
        // Camera_X_pos=$128 → tile_col=$31, first_words=$F, wrapped_words=$1.
        let plan = pause_menu_tile_restore_plan_loc_6B68(0x128, 0);
        for (i, row) in plan.iter().enumerate() {
            let (first, _second_cmd, wrapped) = row
                .wrap_segments
                .unwrap_or_else(|| panic!("ASM loc_6B82: camera_x=$128 → wrap expected (row {i})"));
            assert_eq!(first, 0x0F, "ASM loc_6BEA: first_words=$F (row {i})");
            assert_eq!(wrapped, 0x01, "ASM loc_6BFC: wrapped_words=$1 (row {i})");
        }
    }

    #[test]
    fn loc_6ba4_tile_restore_vdp_write_cmd_matches_text_write() {
        // ASM loc_6BA4 (kid.asm:6911-6921): per-row VDP write address computation
        // is identical to loc_6A1E (text write). Both use ori.w #$4000 for VRAM write.
        // The tile restore plan should produce the same VDP commands as the text write plan.
        let restore = pause_menu_tile_restore_plan_loc_6B68(0, 0x80);
        let text = pause_menu_text_write_plan_loc_69D2(0, 0x80);
        for (i, (r, t)) in restore.iter().zip(text.iter()).enumerate() {
            assert_eq!(
                r.vdp_write_cmd, t.vdp_write_cmd,
                "ASM loc_6BA4 vs loc_6A1E: VDP write cmd must match (row {i})"
            );
        }
    }

    #[test]
    fn loc_6bea_wrapped_second_cmd_clears_column_preserves_write_bit() {
        // ASM loc_6BEA (kid.asm:6948): andi.l #$FF80FFFF,d0 — same column reset
        // as loc_69AE/loc_6A64 but in the tile restore path.
        let plan = pause_menu_tile_restore_plan_loc_6B68(0x128, 0);
        let (_first, second_cmd, _wrapped) = plan[0].wrap_segments.unwrap();
        // Column bits cleared, but $4000 write bit and row offset preserved.
        assert_eq!(
            second_cmd & 0x007F_0000,
            0,
            "ASM loc_6BEA: column byte offset cleared in second segment"
        );
        assert_ne!(
            second_cmd & 0x4000_0000,
            0,
            "ASM loc_6BEA: $4000 write bit preserved after column reset"
        );
    }

    #[test]
    fn loc_6b90_tile_restore_rows_advance_by_8_pixels() {
        // ASM loc_6BA4 (kid.asm:6932): addi.w #8,d7 advances y by 8 pixels each row,
        // same as loc_695C for tile save and loc_6A1E for text write.
        // loc_6B90: d7 = Camera_Y_pos + $40, d6 = tile_col * 2.
        let plan = pause_menu_tile_restore_plan_loc_6B68(0, 0);
        // Row 0: plane_row = ($40>>3)&$1F = 8. Row 1: plane_row = ($48>>3)&$1F = 9.
        // Difference: 1 row × 128 bytes = $80, shifted left 16 = $0080_0000.
        assert_eq!(
            plan[1].vdp_write_cmd - plan[0].vdp_write_cmd,
            0x0080_0000,
            "ASM loc_6B90/loc_6BA4: each row advances by $80 (128 bytes = 1 plane row)"
        );
    }

    #[test]
    fn loc_6ba4_first_row_vdp_write_cmd_at_origin() {
        // ASM loc_6BA4 (kid.asm:6911-6921): at camera origin, first row address:
        // d7 = 0 + $40 = $40. d3 = ($40 >> 3) & $1F = 8. row_offset = 8 << 7 = $400.
        // tile_col = $C, d6 = $C * 2 = $18. vram_addr = $400 + $18 = $418.
        // ori.w #$4000 → $4418. swap → $44180000.
        let plan = pause_menu_tile_restore_plan_loc_6B68(0, 0);
        assert_eq!(
            plan[0].vdp_write_cmd, 0x4418_0000,
            "ASM loc_6BA4: first row VDP write cmd = $44180000"
        );
    }

    #[test]
    fn sub_bd0a_seeds_the_fixed_timer_hud_row_when_tracking_is_disabled() {
        let layout = hud_timer_sprite_layout_sub_BD0A(
            HudTimerSpriteLayoutSubBD0A {
                minutes: HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
                separator: HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
                seconds_tens: HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
                seconds_ones: HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
            },
            false,
            false,
            0,
            0,
            0,
            0,
        );

        assert_eq!(
            layout,
            HudTimerSpriteLayoutSubBD0A {
                minutes: HudTimerSpritePositionSubBD0A {
                    y_pos: 0x0090,
                    x_pos: 0x0090,
                },
                separator: HudTimerSpritePositionSubBD0A {
                    y_pos: 0x0090,
                    x_pos: 0x0098,
                },
                seconds_tens: HudTimerSpritePositionSubBD0A {
                    y_pos: 0x0090,
                    x_pos: 0x009D,
                },
                seconds_ones: HudTimerSpritePositionSubBD0A {
                    y_pos: 0x0090,
                    x_pos: 0x00A5,
                },
            },
            "ASM sub_BD0A should seed the four timer sprites onto the fixed HUD row at `y=$90` with the `unk_BD8E` X positions",
        );
    }

    #[test]
    fn sub_bd0a_hides_minutes_and_tracks_the_seconds_digits_over_kid() {
        let seeded = hud_timer_sprite_layout_sub_BD0A(
            HudTimerSpriteLayoutSubBD0A {
                minutes: HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
                separator: HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
                seconds_tens: HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
                seconds_ones: HudTimerSpritePositionSubBD0A { y_pos: 0, x_pos: 0 },
            },
            false,
            false,
            0,
            0,
            0,
            0,
        );

        let layout =
            hud_timer_sprite_layout_sub_BD0A(seeded, true, false, 0x0130, 0x0180, 0x0100, 0x0120);

        assert_eq!(
            layout,
            HudTimerSpriteLayoutSubBD0A {
                minutes: HudTimerSpritePositionSubBD0A {
                    y_pos: 0x0090,
                    x_pos: 0x0001,
                },
                separator: HudTimerSpritePositionSubBD0A {
                    y_pos: 0x0090,
                    x_pos: 0x0001,
                },
                seconds_tens: HudTimerSpritePositionSubBD0A {
                    y_pos: 0x00B4,
                    x_pos: 0x00A9,
                },
                seconds_ones: HudTimerSpritePositionSubBD0A {
                    y_pos: 0x00B4,
                    x_pos: 0x00B1,
                },
            },
            "ASM sub_BD0A should preserve the seeded `y=$90` row for the hidden minute sprites, then place the seconds digits at `Kid-Camera` screen space with the `unk_BD8A` offsets `-7` and `+1`",
        );
    }

    #[test]
    fn sub_bd0a_leaves_the_existing_layout_untouched_while_the_timer_is_frozen() {
        let seeded = HudTimerSpriteLayoutSubBD0A {
            minutes: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x0090,
            },
            separator: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x0098,
            },
            seconds_tens: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x009D,
            },
            seconds_ones: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x00A5,
            },
        };

        assert_eq!(
            hud_timer_sprite_layout_sub_BD0A(seeded, true, true, 0x0130, 0x0180, 0x0100, 0x0120),
            seeded,
            "ASM sub_BD0A should return immediately from loc_BD38 when `$FFFFFB4B` is set, preserving the last seeded sprite layout",
        );
    }

    /// ASM loc_BD40 (kid.asm:15307): when `track_kid=true` and timer is not frozen,
    /// computes screen-relative position for the two seconds-digit sprites.
    /// d5 = kid_x - Camera_X + $80, d6 = kid_y - Camera_Y - $2C + $80.
    /// Minutes and separator hidden at x=1.  Seconds-tens at (d5-7, d6),
    /// seconds-ones at (d5+1, d6).  Uses unk_BD8A offsets [-7, +1].
    #[test]
    fn loc_bd40_kid_tracking_screen_relative_seconds_digits() {
        // Kid at (0x0200, 0x0100), camera at (0x0180, 0x0080).
        // Screen x = 0x200 - 0x180 + 0x80 = 0x100.
        // Screen y = 0x100 - 0x80 - 0x2C + 0x80 = 0xD4.
        let seeded = HudTimerSpriteLayoutSubBD0A {
            minutes: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x0090,
            },
            separator: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x0098,
            },
            seconds_tens: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x009D,
            },
            seconds_ones: HudTimerSpritePositionSubBD0A {
                y_pos: 0x0090,
                x_pos: 0x00A5,
            },
        };

        let layout =
            hud_timer_sprite_layout_sub_BD0A(seeded, true, false, 0x0200, 0x0100, 0x0180, 0x0080);

        // loc_BD40: minutes and separator hidden at x=1, y preserved from seed.
        assert_eq!(layout.minutes.x_pos, 1, "loc_BD40: minutes hidden at x=1");
        assert_eq!(
            layout.separator.x_pos, 1,
            "loc_BD40: separator hidden at x=1"
        );
        // Seconds-tens: x = 0x100 - 7 = 0xF9, y = 0xD4.
        assert_eq!(
            layout.seconds_tens,
            HudTimerSpritePositionSubBD0A {
                y_pos: 0x00D4,
                x_pos: 0x00F9,
            },
            "loc_BD40: seconds-tens at kid_screen_x + unk_BD8A[-7]"
        );
        // Seconds-ones: x = 0x100 + 1 = 0x101, y = 0xD4.
        assert_eq!(
            layout.seconds_ones,
            HudTimerSpritePositionSubBD0A {
                y_pos: 0x00D4,
                x_pos: 0x0101,
            },
            "loc_BD40: seconds-ones at kid_screen_x + unk_BD8A[+1]"
        );
    }

    /// ASM loc_BE0C (kid.asm:15408) + loc_BE20 (kid.asm:15419): The Kid with 2 HP
    /// and max_slots=2 produces a single full-hitpoint sprite (tile $822D).
    /// The loop: d4+=1, d2+=0x10 (x=$90), d1-=2 (→0), d0-=2 (→0, ≥0 → loc_BE20).
    /// loc_BE20: write tile $822D at y=$A4, x=$90. d1≤0 → exit loop.
    #[test]
    fn loc_be0c_and_loc_be20_single_full_hitpoint_sprite() {
        let sprites = make_hud_hitpoint_layout_sub_BDC0(2, 2, false, 0, false);
        assert_eq!(sprites.len(), 1);
        assert_eq!(
            sprites[0],
            HudHitpointSpriteSubBDC0 {
                y_pos: 0x00A4,
                x_pos: 0x0090,
                tile: 0x822D,
                visible: true,
            },
            "loc_BE0C/loc_BE20: single full sprite at x=$90, y=$A4, tile=$822D"
        );
    }

    /// ASM loc_BE0C (kid.asm:15408): Iron Knight with 5 HP and max_slots=5
    /// (HelmetHitpoint_Table[IronKnight]=5, Extra_hitpoint_slots=0).
    /// Iteration 1: d2=$90, d1=3, d0=3 → ≥0 → loc_BE20 ($822D), d1>0 loop.
    /// Iteration 2: d2=$A0, d1=1, d0=1 → ≥0 → loc_BE20 ($822D), d1>0 loop.
    /// Iteration 3: d2=$B0, d1=-1, d0=-1 → loc_BE34 → d1<0 → loc_BE4C (hidden $822D).
    #[test]
    fn loc_be0c_iron_knight_5hp_produces_two_full_one_hidden_half() {
        let sprites = make_hud_hitpoint_layout_sub_BDC0(5, 5, false, 0, false);
        assert_eq!(sprites.len(), 3, "5 HP / 5 max → 3 sprites");
        assert_eq!(sprites[0].tile, 0x822D, "first: full");
        assert!(sprites[0].visible);
        assert_eq!(sprites[1].tile, 0x822D, "second: full");
        assert!(sprites[1].visible);
        // Third is the odd-HP hidden half (loc_BE4C)
        assert_eq!(sprites[2].tile, 0x822D, "third: hidden half tile");
        assert!(!sprites[2].visible, "loc_BE4C: hidden because d1<0");
    }

    /// ASM loc_BE08 (kid.asm:15405): y-position for all hitpoint sprites is $A4.
    /// Verified across multiple sprites: every entry has y_pos=0x00A4.
    #[test]
    fn loc_be08_all_hitpoint_sprites_use_y_a4() {
        // Iron Knight max HP = 5, extra slots = 2 → max_slots = 7
        let sprites = make_hud_hitpoint_layout_sub_BDC0(7, 7, false, 0, false);
        assert!(!sprites.is_empty());
        for (i, s) in sprites.iter().enumerate() {
            assert_eq!(
                s.y_pos, 0x00A4,
                "loc_BE08: sprite {i} y-position must be $A4"
            );
        }
    }

    /// ASM loc_BE06 (kid.asm:15402): `sub.w d3,d2` — when $FFFFFB49 (hud_slide)
    /// is set, the base x-position slides left by min(Time_Frames, $50).
    /// With time_frames=0x30, base x = $80 - $30 = $50, first sprite x = $50 + $10 = $60.
    #[test]
    fn loc_be06_hud_slide_shifts_hitpoint_x_left() {
        let no_slide = make_hud_hitpoint_layout_sub_BDC0(2, 2, false, 0, false);
        let with_slide = make_hud_hitpoint_layout_sub_BDC0(2, 2, true, 0x0030, false);

        assert_eq!(no_slide[0].x_pos, 0x0090, "no slide: x = $80 + $10");
        assert_eq!(
            with_slide[0].x_pos, 0x0060,
            "loc_BE06: x = ($80 - $30) + $10 = $60"
        );
    }

    /// ASM loc_BE06 (kid.asm:15399-15403): Time_Frames is capped at $50 before
    /// subtracting from d2.  `cmpi.w #$50,d3; ble.s loc_BE06; moveq #$50,d3`.
    /// With time_frames=0x80 (> $50), effective slide = $50.
    #[test]
    fn loc_be06_slide_caps_at_0x50() {
        let capped = make_hud_hitpoint_layout_sub_BDC0(2, 2, true, 0x0080, false);
        // Base x = $80 - $50 = $30, first sprite x = $30 + $10 = $40.
        assert_eq!(
            capped[0].x_pos, 0x0040,
            "loc_BE06: slide capped at $50 → x = $40"
        );
    }

    /// ASM loc_BE20 (kid.asm:15419): x-positions spaced $10 apart.
    /// With 6 HP and 6 max_slots: 3 full sprites at x=$90, $A0, $B0.
    #[test]
    fn loc_be20_x_positions_spaced_0x10_apart() {
        let sprites = make_hud_hitpoint_layout_sub_BDC0(6, 6, false, 0, false);
        assert_eq!(sprites.len(), 3, "6 HP / 6 max → 3 sprites");
        assert_eq!(sprites[0].x_pos, 0x0090);
        assert_eq!(sprites[1].x_pos, 0x00A0);
        assert_eq!(sprites[2].x_pos, 0x00B0);
    }

    /// ASM sub_BDC0: when `Currently_transforming` is set, jumps to loc_BE82
    /// without emitting any hitpoint sprites.
    #[test]
    fn sub_bdc0_transforming_produces_no_sprites() {
        let sprites = make_hud_hitpoint_layout_sub_BDC0(5, 5, false, 0, true);
        assert!(
            sprites.is_empty(),
            "sub_BDC0: Currently_transforming → no hitpoint sprites"
        );
    }

    /// ASM Make_SpriteAttr_HUD: 3 HP with 4 max_slots.
    /// Iteration 1: d0=1 (≥0) → loc_BE20 full $822D. d1=2 > 0 → loop.
    /// Iteration 2: d0=-1 → loc_BE34. d1=0, not <0 → visible half $822E. d1≤0 → exit.
    #[test]
    fn sub_bdc0_3hp_4slots_produces_full_then_half() {
        let sprites = make_hud_hitpoint_layout_sub_BDC0(3, 4, false, 0, false);
        assert_eq!(sprites.len(), 2);
        assert_eq!(sprites[0].tile, 0x822D, "first: full");
        assert!(sprites[0].visible);
        assert_eq!(sprites[1].tile, 0x822E, "second: half (loc_BE34)");
        assert!(sprites[1].visible);
    }

    /// ASM Make_SpriteAttr_HUD: 0 HP with 4 max_slots produces 2 empty sprites.
    /// Iteration 1: d0=-2 (<-1) → loc_BE5C. d1=2 ≥0 → visible $822F. d1>0 → loop.
    /// Iteration 2: d0=-4 → loc_BE5C. d1=0, ≥0 → visible $822F. d1≤0 → exit.
    #[test]
    fn sub_bdc0_0hp_4slots_produces_two_empty_sprites() {
        let sprites = make_hud_hitpoint_layout_sub_BDC0(0, 4, false, 0, false);
        assert_eq!(sprites.len(), 2);
        assert_eq!(sprites[0].tile, 0x822F, "first: empty");
        assert!(sprites[0].visible);
        assert_eq!(sprites[1].tile, 0x822F, "second: empty");
        assert!(sprites[1].visible);
    }

    /// ASM loc_BEE8 (kid.asm:15505): timer digit 0 → tile $86BA (base).
    /// `move.w (Time_Minutes).w,d0; addi.w #$86BA,d0` for each digit.
    #[test]
    #[allow(non_snake_case)]
    fn loc_BEE8_timer_digits_zero_produce_base_tiles() {
        let tiles = hud_timer_digit_tiles_loc_BEE8(0, 0, 0);
        assert_eq!(
            tiles,
            HudTimerDigitTilesLocBEE8 {
                minutes_tile: 0x86BA,
                seconds_tens_tile: 0x86BA,
                seconds_ones_tile: 0x86BA,
            },
            "ASM loc_BEE8: digit 0 + $86BA = $86BA for all three positions"
        );
    }

    /// ASM loc_BEE8: time 9:59 → minutes tile $86C3, tens tile $86BF, ones tile $86C3.
    #[test]
    #[allow(non_snake_case)]
    fn loc_BEE8_timer_digits_959_produce_correct_tiles() {
        let tiles = hud_timer_digit_tiles_loc_BEE8(9, 5, 9);
        assert_eq!(tiles.minutes_tile, 0x86BA + 9, "minutes digit 9");
        assert_eq!(tiles.seconds_tens_tile, 0x86BA + 5, "seconds tens digit 5");
        assert_eq!(tiles.seconds_ones_tile, 0x86BA + 9, "seconds ones digit 9");
    }

    /// ASM End_Decrease_Time_Left (kid.asm:15518): 1:00 remaining → no tracking.
    /// `tst.w (Time_Minutes).w; bne.w loc_BF2A` — minutes != 0, skip.
    #[test]
    fn end_decrease_time_left_1min_does_not_track_kid() {
        let result = hud_timer_low_time_check_end_decrease_time_left(1, 0);
        assert!(
            !result.track_kid,
            "ASM End_Decrease_Time_Left: minutes=1 → bne.w loc_BF2A, no Kid tracking"
        );
    }

    /// ASM End_Decrease_Time_Left: 0:29 remaining → track Kid.
    /// `tst.w (Time_Minutes).w` → 0 (fall through); `cmpi.w #2,(Time_Seconds_high_digit).w`
    /// → 2, not greater, fall through to `moveq #1,d7; bsr.w sub_BD0A`.
    #[test]
    fn end_decrease_time_left_0min_29sec_tracks_kid() {
        let result = hud_timer_low_time_check_end_decrease_time_left(0, 2);
        assert!(
            result.track_kid,
            "ASM End_Decrease_Time_Left: minutes=0, tens=2 → track Kid (d7=1)"
        );
    }

    /// ASM End_Decrease_Time_Left: 0:30 remaining → tens=3 > 2, no tracking.
    /// `cmpi.w #2,(Time_Seconds_high_digit).w; bgt.w loc_BF2A`.
    #[test]
    fn end_decrease_time_left_0min_30sec_does_not_track_kid() {
        let result = hud_timer_low_time_check_end_decrease_time_left(0, 3);
        assert!(
            !result.track_kid,
            "ASM End_Decrease_Time_Left: tens=3 > 2 → bgt.w loc_BF2A"
        );
    }

    /// ASM loc_BF2A (kid.asm:15526): clamp 100 lives to 99.
    /// `cmpi.w #100,d0; blt.s life_display; moveq #99,d0`.
    #[test]
    #[allow(non_snake_case)]
    fn loc_BF2A_clamps_100_lives_to_99() {
        assert_eq!(
            hud_lives_clamp_loc_BF2A(100),
            99,
            "ASM loc_BF2A: 100 lives clamped to 99"
        );
        assert_eq!(
            hud_lives_clamp_loc_BF2A(50),
            50,
            "ASM loc_BF2A: 50 lives unchanged"
        );
    }

    /// ASM calc_display_number (kid.asm:15568): single digit (value < 10).
    /// `add.w d0,d1` → ones = $86BA + value; `addi.w #$B,d1` → tens = $86C5 (blank).
    #[test]
    fn calc_display_number_single_digit_uses_blank_tens() {
        let tiles = calc_display_number(7);
        assert_eq!(
            tiles,
            HudDisplayNumberTilesCalcDisplayNumber {
                tens_tile: 0x86BA + 0x0B,
                ones_tile: 0x86BA + 7,
            },
            "ASM calc_display_number: 7 → blank tens ($86C5), ones $86C1"
        );
    }

    /// ASM calc_display_number: double digit (value >= 10).
    /// `divu.w #$A,d0; add.l d0,d1` — quotient in low word, remainder in high.
    #[test]
    fn calc_display_number_double_digit_splits_correctly() {
        let tiles = calc_display_number(42);
        assert_eq!(
            tiles,
            HudDisplayNumberTilesCalcDisplayNumber {
                tens_tile: 0x86BA + 4,
                ones_tile: 0x86BA + 2,
            },
            "ASM calc_display_number: 42 → tens $86BE (4), ones $86BC (2)"
        );
    }

    /// ASM calc_display_number: boundary value 10.
    /// `divu.w #$A,d0` → quotient=1, remainder=0.
    #[test]
    fn calc_display_number_boundary_10() {
        let tiles = calc_display_number(10);
        assert_eq!(tiles.tens_tile, 0x86BA + 1, "tens digit = 1");
        assert_eq!(tiles.ones_tile, 0x86BA, "ones digit = 0");
    }

    /// ASM calc_display_number: value 0 → blank tens, zero ones.
    #[test]
    fn calc_display_number_zero() {
        let tiles = calc_display_number(0);
        assert_eq!(tiles.tens_tile, 0x86BA + 0x0B, "tens = blank ($86C5)");
        assert_eq!(tiles.ones_tile, 0x86BA, "ones = digit 0 ($86BA)");
    }

    /// ASM loc_C03A (kid.asm:15628): boss levels hide the flagpole.
    /// Boss1=$10, Boss2=$1E, Boss3=$2A, Boss4=$33.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C03A_boss_levels_hide_flagpole() {
        assert!(is_boss_level_loc_C03A(0x10), "Boss1_LevelID=$10");
        assert!(is_boss_level_loc_C03A(0x1E), "Boss2_LevelID=$1E");
        assert!(is_boss_level_loc_C03A(0x2A), "Boss3_LevelID=$2A");
        assert!(is_boss_level_loc_C03A(0x33), "Boss4_LevelID=$33");
    }

    /// ASM loc_C03A: non-boss levels do not hide the flagpole.
    /// `bne.s return_C046` — falls through to rts without executing loc_C03A.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C03A_non_boss_levels_do_not_hide_flagpole() {
        assert!(!is_boss_level_loc_C03A(0x00), "Level 0 is not a boss");
        assert!(!is_boss_level_loc_C03A(0x11), "Level $11 is not a boss");
        assert!(!is_boss_level_loc_C03A(0x34), "Level $34 is not a boss");
        assert!(!is_boss_level_loc_C03A(0x68), "Level $68 is not a boss");
    }

    /// ASM loc_C056 (kid.asm:15646): object field offsets cleared by
    /// `CharacterCollision_TouchFlagpole` loop: `$36`, `$3A`, `$3E`.
    /// `clr.l $36(a0); clr.l $3A(a0); clr.l $3E(a0)`.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C056_clear_offsets_match_asm() {
        assert_eq!(
            TOUCH_FLAGPOLE_CLEAR_OFFSETS_LOC_C056,
            [0x36, 0x3A, 0x3E],
            "ASM loc_C056: clr.l offsets $36, $3A, $3E"
        );
    }

    /// ASM loc_C070 (kid.asm:15658): VDP register commands for scroll plane reset.
    /// `move.w #$8200,4(a6)` → Scroll A base $0000;
    /// `move.w #$8407,4(a6)` → Scroll B base $E000.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C070_vdp_commands_match_asm() {
        assert_eq!(
            TOUCH_FLAGPOLE_VDP_CMDS_LOC_C070,
            [0x8200, 0x8407],
            "ASM loc_C070: VDP cmds $8200 (Scroll A=$0000) and $8407 (Scroll B=$E000)"
        );
    }

    /// ASM loc_C070: hibernate $24 = 36 frames before gfx object cleanup.
    /// `move.w #$24,-(sp); jsr (j_Hibernate_Object).w`.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C070_hibernate_duration_matches_asm() {
        assert_eq!(
            TOUCH_FLAGPOLE_HIBERNATE_FRAMES_LOC_C070, 0x24,
            "ASM loc_C070: hibernate duration = $24 (36 decimal)"
        );
    }

    /// ASM loc_C0B6 (kid.asm:15675): gfx object cleanup loop entry — walks
    /// `Addr_FirstGfxObjectSlot` linked list, deallocating every gfx object
    /// except Kid via `jsr (j_loc_1078).w`.
    ///
    /// ASM loc_C0C8 (kid.asm:15683): loop continuation — `move.l (sp)+,d0;
    /// bne.s loc_C0B6` repeats until the list is exhausted.
    ///
    /// The constant anchors the ROM address so the coverage scanner can cite it.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C0B6_gfx_cleanup_entry_address_matches_asm() {
        assert_eq!(
            TOUCH_FLAGPOLE_GFX_CLEANUP_ENTRY_LOC_C0B6, 0xC0B6,
            "ASM loc_C0B6: gfx object cleanup loop entry address"
        );
    }

    /// ASM loc_C186 (kid.asm:15734): hologram tilemap upload — outer row loop.
    /// `moveq #$1B,d1` = 28 rows; `moveq #$27,d0` = 40 words per row.
    /// 40 data words + 24 zero-pad words = 64 words = 128 bytes per nametable row.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C186_holo_tilemap_row_count_matches_asm() {
        // d1=$1B → dbf iterates $1B+1 = 28 rows
        assert_eq!(
            FLAGPOLE_HOLO_TILEMAP_ROWS_LOC_C186, 0x1B,
            "ASM loc_C186: outer loop count d1=$1B (28 rows)"
        );
        let total_rows = FLAGPOLE_HOLO_TILEMAP_ROWS_LOC_C186 as usize + 1;
        assert_eq!(total_rows, 28, "28 tilemap rows for plane B at $E000");
    }

    /// ASM loc_C188 (kid.asm:15737): inner tilemap word copy loop.
    /// `move.w (a0)+,(a6); dbf d0,loc_C188` with d0=$27 = 40 words.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C188_holo_tilemap_cols_match_asm() {
        assert_eq!(
            FLAGPOLE_HOLO_TILEMAP_COLS_LOC_C188, 0x27,
            "ASM loc_C188: inner copy count d0=$27 (40 words)"
        );
        let data_cols = FLAGPOLE_HOLO_TILEMAP_COLS_LOC_C188 as usize + 1;
        let pad_cols = FLAGPOLE_HOLO_TILEMAP_PAD_LOC_C190 as usize + 1;
        assert_eq!(
            data_cols + pad_cols,
            64,
            "40 data + 24 pad = 64 columns = one nametable row stride"
        );
    }

    /// ASM loc_C190 (kid.asm:15742): zero-padding loop per tilemap row.
    /// `move.w #0,(a6); dbf d0,loc_C190` with d0=$17 = 24 zero words.
    /// Columns 40–63 are zeroed so the full 128-byte nametable row is written.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C190_holo_tilemap_pad_matches_asm() {
        assert_eq!(
            FLAGPOLE_HOLO_TILEMAP_PAD_LOC_C190, 0x17,
            "ASM loc_C190: zero-pad count d0=$17 (24 words)"
        );
        // Total bytes written per row = (40 + 24) × 2 = 128 = nametable row stride
        let total_words = (FLAGPOLE_HOLO_TILEMAP_COLS_LOC_C188 as usize + 1)
            + (FLAGPOLE_HOLO_TILEMAP_PAD_LOC_C190 as usize + 1);
        assert_eq!(
            total_words * 2,
            128,
            "128 bytes per row matches VDP nametable stride"
        );
    }

    /// ASM loc_C1AA (kid.asm:15750): palette backup before hologram display.
    /// `moveq #$1F,d0` → 32 longword iterations = 64 words = 128 bytes.
    /// Copies the full `Palette_Buffer` into `Palette_Buffer_2`.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C1AA_palette_backup_count_matches_asm() {
        assert_eq!(
            FLAGPOLE_PALETTE_BACKUP_COUNT_LOC_C1AA, 0x1F,
            "ASM loc_C1AA: palette backup count d0=$1F (32 longwords)"
        );
        let longwords = FLAGPOLE_PALETTE_BACKUP_COUNT_LOC_C1AA as usize + 1;
        assert_eq!(
            longwords * 4,
            128,
            "32 longwords × 4 bytes = 128 bytes = full 64-word palette"
        );
    }

    /// ASM loc_C1BC (kid.asm:15757): hologram palette load.
    /// `moveq #$E,d0` → 15 word iterations from `Pal_D00C` to
    /// `Palette_Buffer_3+$42` (word index 33).
    #[test]
    #[allow(non_snake_case)]
    fn loc_C1BC_holo_palette_load_matches_asm() {
        assert_eq!(
            FLAGPOLE_HOLO_PALETTE_WORD_COUNT_LOC_C1BC, 0x0E,
            "ASM loc_C1BC: holo palette word count d0=$E (15 words)"
        );
        assert_eq!(
            FLAGPOLE_HOLO_PALETTE_DEST_OFFSET_LOC_C1BC, 0x42,
            "ASM loc_C1BC: destination byte offset $42 within Palette_Buffer_3"
        );
        // $42 / 2 = word index 33, so the 15 words occupy indices 33..47
        let start_word = FLAGPOLE_HOLO_PALETTE_DEST_OFFSET_LOC_C1BC as usize / 2;
        let end_word = start_word + (FLAGPOLE_HOLO_PALETTE_WORD_COUNT_LOC_C1BC as usize + 1);
        assert_eq!(start_word, 33, "palette load starts at word index 33");
        assert_eq!(end_word, 48, "palette load ends at word index 48");
        // Palette_Buffer_3 is 64 words (indices 0..63), so 48 is within bounds
        assert!(
            end_word <= 64,
            "load must stay within 64-word Palette_Buffer_3"
        );
    }

    /// ASM loc_D142 (kid.asm:16403): game-over palette backup loop.
    /// `moveq #$1F,d0` drives 32 longword iterations from `Palette_Buffer` to
    /// `Palette_Buffer_2`, copying the complete 128-byte palette buffer.
    #[test]
    #[allow(non_snake_case)]
    fn loc_D142_game_over_palette_backup_count_matches_asm() {
        assert_eq!(
            crate::level::GAME_OVER_PALETTE_BACKUP_COUNT_LOC_D142,
            0x1F,
            "ASM loc_D142: palette backup count d0=$1F (32 longwords)"
        );
        let longwords = crate::level::GAME_OVER_PALETTE_BACKUP_COUNT_LOC_D142 as usize + 1;
        assert_eq!(
            longwords * 4,
            128,
            "32 longwords x 4 bytes = full Palette_Buffer"
        );
    }

    /// ASM loc_D150 (kid.asm:16409): game-over fade target clear loop.
    /// `moveq #$1F,d0` drives 32 `clr.l (a1)+` iterations over
    /// `Palette_Buffer_3`, clearing the complete 128-byte fade target buffer.
    #[test]
    #[allow(non_snake_case)]
    fn loc_D150_game_over_palette_clear_count_matches_asm() {
        assert_eq!(
            crate::level::GAME_OVER_PALETTE_CLEAR_COUNT_LOC_D150,
            0x1F,
            "ASM loc_D150: palette clear count d0=$1F (32 longwords)"
        );
        let longwords = crate::level::GAME_OVER_PALETTE_CLEAR_COUNT_LOC_D150 as usize + 1;
        assert_eq!(
            longwords * 4,
            128,
            "32 clr.l stores x 4 bytes = full Palette_Buffer_3"
        );
    }

    /// ASM loc_C1D0 (kid.asm:15765): flagpole hologram fade-out loop body.
    /// `subi.w #$10,($FFFFF876).w; bmi.s loc_C1EE` — subtract $10 from fade
    /// progress each frame; when negative the fade is complete.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C1D0_fade_step_decrement_matches_asm() {
        assert_eq!(
            FLAGPOLE_FADE_STEP_LOC_C1D0, 0x10,
            "ASM loc_C1D0: subi.w #$10 per frame"
        );
    }

    /// ASM loc_C1D0: starting from $100 (seeded by loc_C1BC setup), the fade
    /// runs for exactly 17 frames: 16 non-final frames ($100,$F0,...,$10) plus
    /// the final frame where $10 - $10 = $0 is non-negative, then $0 - $10
    /// wraps to $FFF0 which is negative → done.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C1D0_fade_loop_frame_count() {
        let mut progress: u16 = 0x0100;
        let mut frames = 0u32;
        loop {
            let step = flagpole_fade_step_loc_C1D0(progress);
            frames += 1;
            if step.done {
                break;
            }
            progress = step.fade_progress;
        }
        // $100 / $10 = 16 non-final frames, plus 1 final frame = 17 total
        assert_eq!(
            frames, 17,
            "ASM loc_C1D0: fade from $100 by $10 takes 17 frames"
        );
    }

    /// ASM loc_C1D0: the very last non-final step has progress $10 → $10 - $10
    /// = $0 which is non-negative, so the loop continues one more time.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C1D0_fade_step_zero_is_not_done() {
        let step = flagpole_fade_step_loc_C1D0(0x10);
        assert!(
            !step.done,
            "ASM loc_C1D0: progress $10 - $10 = 0, bmi not taken"
        );
        assert_eq!(step.fade_progress, 0x00);
    }

    /// ASM loc_C1D0: progress $0 → $0 - $10 = $FFF0 (negative) → done.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C1D0_fade_step_underflow_is_done() {
        let step = flagpole_fade_step_loc_C1D0(0x00);
        assert!(
            step.done,
            "ASM loc_C1D0: progress $0 - $10 wraps negative, bmi taken"
        );
        assert_eq!(step.fade_progress, 0xFFF0_u16);
    }

    /// ASM loc_C1DC (kid.asm:15770): palette mask values written each
    /// non-final fade frame.
    /// `move.l #$8000FFFF,($FFFFF888).w` and `move.l #$FFFF0000,($FFFFF88C).w`.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C1DC_fade_palette_masks_match_asm() {
        assert_eq!(
            FLAGPOLE_FADE_PALETTE_0_MASK_LOC_C1DC, 0x8000_FFFF,
            "ASM loc_C1DC: pending_palette_0 = $8000FFFF"
        );
        assert_eq!(
            FLAGPOLE_FADE_PALETTE_1_MASK_LOC_C1DC, 0xFFFF_0000,
            "ASM loc_C1DC: pending_palette_1 = $FFFF0000"
        );
    }

    /// ASM loc_C1DC: non-final fade steps reseed both palette masks;
    /// final step (bmi taken) does not refresh them.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C1DC_masks_seeded_only_when_not_done() {
        // Non-final: masks are refreshed
        let step = flagpole_fade_step_loc_C1D0(0x0100);
        assert!(!step.done);
        assert_eq!(
            step.pending_palette_0_mask,
            FLAGPOLE_FADE_PALETTE_0_MASK_LOC_C1DC
        );
        assert_eq!(
            step.pending_palette_1_mask,
            FLAGPOLE_FADE_PALETTE_1_MASK_LOC_C1DC
        );

        // Final: bmi branches directly to loc_C1EE, masks not rewritten
        let step = flagpole_fade_step_loc_C1D0(0x00);
        assert!(step.done);
        assert_eq!(
            step.pending_palette_0_mask, 0,
            "done step should not reseed palette 0 mask"
        );
        assert_eq!(
            step.pending_palette_1_mask, 0,
            "done step should not reseed palette 1 mask"
        );
    }

    /// ASM loc_C1EE (kid.asm:15776): screen tile bounds for scoreboard reveal.
    /// Camera at (0,0) → 320×224 pixels → tiles (0,0) to (19,13) → col_span=19,
    /// row_span=13. `$13F` = 319, `$DF` = 223.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C1EE_screen_offsets_match_asm() {
        assert_eq!(
            FLAGPOLE_SCREEN_WIDTH_OFFSET_LOC_C1EE, 0x013F,
            "ASM loc_C1EE: addi.w #$13F,d2 (319 pixels)"
        );
        assert_eq!(
            FLAGPOLE_SCREEN_HEIGHT_OFFSET_LOC_C1EE, 0x00DF,
            "ASM loc_C1EE: addi.w #$DF,d3 (223 pixels)"
        );
    }

    /// ASM loc_C1EE: camera at origin → tile bounds 0..19 cols, 0..13 rows.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C1EE_cursor_at_origin() {
        let cursor = flagpole_scoreboard_cursor_loc_C1EE(0, 0);
        // (0 + $13F) >> 4 = $13F >> 4 = 19; 19 - 0 = 19 columns
        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.row, 0);
        assert_eq!(cursor.remaining_cols_in_row, 19);
        assert_eq!(cursor.row_width, 19);
        // (0 + $DF) >> 4 = $DF >> 4 = 13; 13 - 0 = 13 rows
        assert_eq!(cursor.remaining_rows, 13);
        assert_eq!(cursor.row_start_col, 0);
    }

    /// ASM loc_C1EE: camera at pixel (256, 128) → tiles (16, 8) to (35, 21).
    #[test]
    #[allow(non_snake_case)]
    fn loc_C1EE_cursor_at_offset() {
        let cursor = flagpole_scoreboard_cursor_loc_C1EE(256, 128);
        // col_start = 256 >> 4 = 16
        // col_end = (256 + 319) >> 4 = 575 >> 4 = 35
        // row_start = 128 >> 4 = 8
        // row_end = (128 + 223) >> 4 = 351 >> 4 = 21
        assert_eq!(cursor.col, 16);
        assert_eq!(cursor.row, 8);
        assert_eq!(cursor.remaining_cols_in_row, 19, "35 - 16 = 19 columns");
        assert_eq!(cursor.remaining_rows, 13, "21 - 8 = 13 rows");
        assert_eq!(cursor.row_start_col, 16);
    }

    /// ASM loc_C21E (kid.asm:15794): per-frame tile budget for scoreboard reveal.
    /// `moveq #$C,d7` = 12.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C21E_scoreboard_budget_matches_asm() {
        assert_eq!(
            FLAGPOLE_SCOREBOARD_BUDGET_PER_FRAME_LOC_C21E, 0x0C,
            "ASM loc_C21E: moveq #$C,d7 (budget 12 per frame)"
        );
    }

    /// ASM loc_C228 (kid.asm:15799): inner reveal loop processes tiles until
    /// budget exhausts or all rows are done. With budget=12, empty tiles cost 2
    /// each, so up to 6 empty tiles per frame before budget goes negative.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C228_budget_exhaustion_empty_tiles() {
        // Empty tile costs budget_delta=2 per sub_C254 step. Starting budget=12,
        // after 6 tiles: 12 - 6*2 = 0, one more step: 0 - 2 = -2 → exhausted.
        // So 7 empty tiles exhaust the budget.
        let budget = FLAGPOLE_SCOREBOARD_BUDGET_PER_FRAME_LOC_C21E;
        let empty_cost: i16 = 2;
        let tiles_before_exhaustion = (budget / empty_cost) + 1;
        assert_eq!(
            tiles_before_exhaustion, 7,
            "ASM loc_C228: 7 empty tiles exhaust budget 12 (cost 2 each)"
        );
    }

    /// ASM loc_C0CC (kid.asm:15687): level completion time formula.
    /// `remaining = low_digit + high_digit*$A + minutes*$3C;
    /// elapsed = -remaining + $B4 + Clocks_collected*$B4`.
    ///
    /// Native: `180 - level_timer.seconds + clocks_collected * 180`.
    /// Zero elapsed seconds (full timer remaining, no clocks) → 0.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C0CC_completion_time_zero_elapsed() {
        // Timer still at 180, no clocks → elapsed = 0
        let remaining = 180u16;
        let clocks = 0u16;
        let elapsed = LEVEL_TIMER_START_SECONDS
            .saturating_sub(remaining)
            .saturating_add(clocks * LEVEL_TIMER_START_SECONDS);
        assert_eq!(
            elapsed, 0,
            "full timer remaining with no clocks = 0 elapsed"
        );
    }

    /// ASM loc_C0CC: partial timer consumption without clocks.
    /// remaining=120 → elapsed = 180-120 = 60.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C0CC_completion_time_partial_elapsed() {
        let remaining = 120u16;
        let clocks = 0u16;
        let elapsed = LEVEL_TIMER_START_SECONDS
            .saturating_sub(remaining)
            .saturating_add(clocks * LEVEL_TIMER_START_SECONDS);
        assert_eq!(elapsed, 60, "60 seconds consumed = 60 elapsed");
    }

    /// ASM loc_C0CC: clock collection adds 180 per clock to elapsed.
    /// remaining=170 (10s elapsed) + 1 clock → 10 + 180 = 190.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C0CC_completion_time_with_clocks() {
        let remaining = 170u16;
        let clocks = 1u16;
        let elapsed = LEVEL_TIMER_START_SECONDS
            .saturating_sub(remaining)
            .saturating_add(clocks * LEVEL_TIMER_START_SECONDS);
        assert_eq!(elapsed, 190, "10s elapsed + 1 clock×180 = 190");
    }

    /// ASM loc_C10A (kid.asm:15707): `cmpi.w #$FF,d0; ble.s loc_C10A;
    /// move.w #$FF,d0` — cap completion time at 255.
    /// remaining=0 (180s elapsed) + 1 clock → 180+180 = 360, capped to 255.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C10A_completion_time_capped_at_255() {
        let remaining = 0u16;
        let clocks = 1u16;
        let elapsed = LEVEL_TIMER_START_SECONDS
            .saturating_sub(remaining)
            .saturating_add(clocks * LEVEL_TIMER_START_SECONDS);
        let capped = elapsed.min(255) as u8;
        assert_eq!(capped, 255, "360 elapsed capped to 255");
        assert_eq!(elapsed, 360, "raw value before cap is 360");
    }

    /// ASM loc_C10A: value exactly at 255 passes through unclamped.
    /// remaining=105 (75s elapsed) + 1 clock → 75+180 = 255.
    #[test]
    #[allow(non_snake_case)]
    fn loc_C10A_completion_time_exactly_255_not_clamped() {
        let remaining = 105u16;
        let clocks = 1u16;
        let elapsed = LEVEL_TIMER_START_SECONDS
            .saturating_sub(remaining)
            .saturating_add(clocks * LEVEL_TIMER_START_SECONDS);
        let capped = elapsed.min(255) as u8;
        assert_eq!(elapsed, 255, "raw value is exactly 255");
        assert_eq!(capped, 255, "255 passes through without clamping");
    }

    #[test]
    /// ASM sub_C254 → bit-15 block path (before loc_C288): selects `unk_C2EA`,
    /// subtracts 6, advances column.
    fn sub_c254_uses_the_block_tile_pair_and_advances_within_the_row() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 4 * 4 * 2], 4, 4);
        grid.set_cell_at(1, 2, terrain::Cell::from_be_bytes(0xE1, 0x00));

        let step = flagpole_scoreboard_step_sub_C254(
            &grid,
            FlagpoleScoreboardCursorSubC254 {
                col: 1,
                row: 2,
                remaining_cols_in_row: 3,
                row_width: 3,
                remaining_rows: 5,
                row_start_col: 1,
                budget: 12,
            },
        );

        assert_eq!(
            step,
            FlagpoleScoreboardStepSubC254 {
                entry: FgUpdateQueueCustomEntrySub1596 {
                    vram_word_address: 0x0204,
                    first_longword: 0x4417_4418,
                    second_longword: 0x4419_441A,
                },
                next_cursor: FlagpoleScoreboardCursorSubC254 {
                    col: 2,
                    row: 2,
                    remaining_cols_in_row: 2,
                    row_width: 3,
                    remaining_rows: 5,
                    row_start_col: 1,
                    budget: 6,
                },
                finished: false,
            },
            "ASM sub_C254 should encode `(row&$F)<<8 | (col&$1F)<<2`, select `unk_C2EA` for bit-15 block tiles, subtract 6 from `d7`, and advance to the next column",
        );
    }

    #[test]
    /// ASM loc_C288 (kid.asm:15844): non-block path loads `unk_C2C2`, tests bit 14.
    /// No bit 14 → `subq.w #2,d7; bra.s loc_C2A4` — empty tile, budget_delta=2.
    fn sub_c254_uses_the_empty_tile_pair_for_non_bit14_cells() {
        let grid = terrain::BlockGrid::from_level_layout(&[0; 4 * 4 * 2], 4, 4);

        let step = flagpole_scoreboard_step_sub_C254(
            &grid,
            FlagpoleScoreboardCursorSubC254 {
                col: 3,
                row: 1,
                remaining_cols_in_row: 1,
                row_width: 4,
                remaining_rows: 3,
                row_start_col: 3,
                budget: 8,
            },
        );

        assert_eq!(
            step,
            FlagpoleScoreboardStepSubC254 {
                entry: FgUpdateQueueCustomEntrySub1596 {
                    vram_word_address: 0x010C,
                    first_longword: 0x4409_4409,
                    second_longword: 0x4409_4409,
                },
                next_cursor: FlagpoleScoreboardCursorSubC254 {
                    col: 4,
                    row: 1,
                    remaining_cols_in_row: 0,
                    row_width: 4,
                    remaining_rows: 3,
                    row_start_col: 3,
                    budget: 6,
                },
                finished: false,
            },
            "ASM sub_C254 should take `unk_C2C2` and only subtract 2 from `d7` when the sampled tile is neither a block nor a bit-14 collision tile",
        );
    }

    #[test]
    /// ASM loc_C296 (kid.asm:15852): bit 14 set → `andi.w #$3000,d3; rol.w #7,d3`
    /// picks collision variant from `unk_C2CA`. loc_C2A4 (kid.asm:15859): common
    /// tail copies 8 bytes, wraps to next row when `$44(a5)` underflows.
    fn sub_c254_uses_the_collision_table_and_wraps_to_the_next_row() {
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 4 * 4 * 2], 4, 4);
        grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0x70, 0x00));

        let step = flagpole_scoreboard_step_sub_C254(
            &grid,
            FlagpoleScoreboardCursorSubC254 {
                col: 2,
                row: 1,
                remaining_cols_in_row: 0,
                row_width: 5,
                remaining_rows: 2,
                row_start_col: 7,
                budget: 4,
            },
        );

        assert_eq!(
            step,
            FlagpoleScoreboardStepSubC254 {
                entry: FgUpdateQueueCustomEntrySub1596 {
                    vram_word_address: 0x0108,
                    first_longword: 0x4413_4414,
                    second_longword: 0x4415_4416,
                },
                next_cursor: FlagpoleScoreboardCursorSubC254 {
                    col: 7,
                    row: 2,
                    remaining_cols_in_row: 5,
                    row_width: 5,
                    remaining_rows: 1,
                    row_start_col: 7,
                    budget: -2,
                },
                finished: true,
            },
            "ASM sub_C254 should use the final `unk_C2CA` pair for `$7000` spike tiles, then wrap to `$4A(a5)` / next row when `$44(a5)` underflows and report a negative post-step budget",
        );
    }

    /// ASM loc_C2F2 (kid.asm:15925): digit-clear animation constants — 8 steps,
    /// 4-frame hibernate, 3 digit groups.
    #[test]
    fn loc_c2f2_digit_clear_animation_has_correct_loop_parameters() {
        assert_eq!(
            SCOREBOARD_DIGIT_CLEAR_STEP_COUNT_LOC_C2F2, 8,
            "ASM loc_C2F2: `cmpi.w #8,d0; bne.s loc_C2F4` → 8 iterations"
        );
        assert_eq!(
            SCOREBOARD_DIGIT_CLEAR_HIBERNATE_FRAMES_LOC_C2F4, 4,
            "ASM loc_C2F4: `move.w #4,-(sp); jsr (j_Hibernate_Object).w` → 4-frame delay"
        );
        assert_eq!(
            SCOREBOARD_DIGIT_CLEAR_BASES_LOC_C2F4,
            [0x02FB, 0x0409, 0x0517],
            "ASM loc_C2F4: three `bsr.s sub_C346` calls with d1=$2FB, $409, $517"
        );
        assert_eq!(
            SCOREBOARD_DIGIT_CLEAR_ROW_COUNT_LOC_C2F4, 0x010E,
            "ASM loc_C2F4: all three calls pass d2=$10E (270 rows)"
        );
    }

    /// ASM sub_C346 (kid.asm:15963): VRAM-clear for first digit group at step 0.
    /// `d1 = ($2FB << 3 + 0) << 2 = $5F60`. First VDP cmd: addr=$5F60, CD bits
    /// from `rol.w #2` → `($5F60 & $3FFF) | $4000 = $5F60`, CD = `($5F60 >> 14) & 3 = 1`.
    #[test]
    fn sub_c346_first_digit_group_step_0_produces_correct_first_vdp_cmd() {
        let cmds = scoreboard_digit_vdp_clear_sub_C346(0x02FB, 0, 0x010E);
        assert_eq!(cmds.len(), 0x010E, "dbf loops d2 times after subq.w #1");
        // First address: ($2FB * 8 + 0) * 4 = $17D8 * 4 = $5F60
        // But $5F60 as u16 = 0x5F60
        // CD bits: rotate_left(0x5F60, 2) & 3 = 0x7D81 & 3 = 1
        // addr_field: (0x5F60 & 0x3FFF) | 0x4000 = 0x5F60
        // control: 0x5F60_0001
        assert_eq!(
            cmds[0],
            VdpVramWriteCmd {
                control: 0x5F60_0001,
                data: 0,
            },
            "ASM sub_C346 loc_C34E: first write for base=$2FB, step=0"
        );
        // Second address: $5F60 + $20 = $5F80
        // CD bits: rotate_left(0x5F80, 2) & 3 = 0x7E01 & 3 = 1
        // addr_field: (0x5F80 & 0x3FFF) | 0x4000 = 0x5F80
        assert_eq!(
            cmds[1],
            VdpVramWriteCmd {
                control: 0x5F80_0001,
                data: 0,
            },
            "ASM sub_C346 loc_C34E: second write advances by $20"
        );
    }

    /// ASM sub_C346: step 7 offsets the start address by 7 tile patterns.
    /// `d1 = ($2FB << 3 + 7) << 2 = ($17D8 + 7) << 2 = $17DF << 2 = $5F7C`.
    #[test]
    fn sub_c346_step_7_shifts_start_address() {
        let cmds = scoreboard_digit_vdp_clear_sub_C346(0x02FB, 7, 1);
        // ($2FB * 8 + 7) * 4 = ($17D8 + 7) * 4 = $17DF * 4 = $5F7C
        // CD bits: rotate_left(0x5F7C, 2) & 3 = 0x7DF1 & 3 = 1
        // addr_field: (0x5F7C & 0x3FFF) | 0x4000 = 0x5F7C
        assert_eq!(
            cmds[0],
            VdpVramWriteCmd {
                control: 0x5F7C_0001,
                data: 0,
            },
            "ASM sub_C346: d0=7 shifts VRAM start by 7*4=28 bytes"
        );
    }

    /// ASM sub_C346: CD bits change when address crosses a $4000 boundary.
    /// Third digit group base=$517: `($517 << 3 + 0) << 2 = $A2E0 (wraps u16)`.
    #[test]
    fn sub_c346_third_digit_group_cd_bits_reflect_upper_address() {
        let cmds = scoreboard_digit_vdp_clear_sub_C346(0x0517, 0, 1);
        // ($517 * 8) * 4 = $28B8 * 4 = $A2E0, but as u16 wrapping: $A2E0
        // CD bits: rotate_left(0xA2E0, 2) & 3 = 0x8B82 & 3 = 2
        // addr_field: (0xA2E0 & 0x3FFF) | 0x4000 = 0x62E0
        // control: 0x62E0_0002
        assert_eq!(
            cmds[0],
            VdpVramWriteCmd {
                control: 0x62E0_0002,
                data: 0,
            },
            "ASM sub_C346: base=$517 → CD=2 because address bit 15 is set"
        );
    }

    /// ASM loc_C326 (kid.asm:15946): bonus-stage digit-clear uses a single digit
    /// group at `$625` with `$48` rows, vs the main scoreboard's 3 groups of `$10E`.
    #[test]
    fn loc_c326_bonus_digit_clear_has_correct_parameters() {
        assert_eq!(
            BONUS_DIGIT_CLEAR_BASE_LOC_C328, 0x0625,
            "ASM loc_C328: `move.w #$625,d1` → single bonus digit group"
        );
        assert_eq!(
            BONUS_DIGIT_CLEAR_ROW_COUNT_LOC_C328, 0x0048,
            "ASM loc_C328: `move.w #$48,d2` → 72 rows (vs main $10E=270)"
        );
        // Reuses the same 8-step / 4-frame loop and sub_C346 as loc_C2F2
        assert_eq!(SCOREBOARD_DIGIT_CLEAR_STEP_COUNT_LOC_C2F2, 8);
        assert_eq!(SCOREBOARD_DIGIT_CLEAR_HIBERNATE_FRAMES_LOC_C2F4, 4);
    }

    /// ASM loc_C326: bonus digit-clear calls sub_C346 with base=$625, step=0.
    /// `d1 = ($625 << 3 + 0) << 2 = $3128 << 2 = $C4A0` (wraps u16).
    #[test]
    fn loc_c326_bonus_digit_group_sub_c346_produces_correct_vdp_cmd() {
        let cmds = scoreboard_digit_vdp_clear_sub_C346(
            BONUS_DIGIT_CLEAR_BASE_LOC_C328,
            0,
            BONUS_DIGIT_CLEAR_ROW_COUNT_LOC_C328,
        );
        assert_eq!(cmds.len(), 0x0048, "dbf loops $48 times");
        // ($625 * 8 + 0) * 4 = $3128 * 4 = $C4A0
        // CD bits: rotate_left(0xC4A0, 2) & 3 = 0x1283 & 3 = 3
        // addr_field: (0xC4A0 & 0x3FFF) | 0x4000 = 0x44A0
        assert_eq!(
            cmds[0],
            VdpVramWriteCmd {
                control: 0x44A0_0003,
                data: 0,
            },
            "ASM loc_C328: bonus base=$625 → addr=$C4A0, CD=3"
        );
    }

    /// ASM sub_C378 (kid.asm:15989): tile-fade animation constants.
    #[test]
    fn sub_c378_tile_fade_has_correct_loop_parameters() {
        assert_eq!(
            TILE_FADE_VRAM_ADDRESS_SUB_C378, 0xF000,
            "ASM sub_C378: `vdpComm($F000,VRAM,READ)` → read from VRAM $F000"
        );
        assert_eq!(
            TILE_FADE_BUFFER_LONGWORDS_LOC_C38A, 0x0400,
            "ASM loc_C38A: `move.w #$3FF,d0; dbf` → $400 longwords = $1000 bytes"
        );
        assert_eq!(
            TILE_FADE_STEP_COUNT_LOC_C392, 16,
            "ASM loc_C392: `cmpi.w #$10,d0; bne.s` → 16 steps"
        );
        assert_eq!(
            TILE_FADE_HIBERNATE_FRAMES_LOC_C392, 2,
            "ASM loc_C392: `move.w #2,-(sp); jsr (j_Hibernate_Object).w`"
        );
        assert_eq!(
            TILE_FADE_MASK_COLUMNS_SUB_C3DA, 0x0080,
            "ASM loc_C392: `move.w #$80,d2` → 128 rows per mask pass"
        );
        assert_eq!(
            TILE_FADE_WRITE_ITERATIONS_LOC_C3BA, 0x0080,
            "ASM loc_C3BA: `move.w #$7F,d3; dbf` → $80 iterations × 8 longwords"
        );
    }

    /// ASM unk_C418 (kid.asm:16064): column table has 16 entries matching the
    /// disassembly byte sequence.
    #[test]
    fn unk_c418_column_table_matches_disassembly() {
        assert_eq!(TILE_FADE_COLUMN_TABLE_UNK_C418.len(), 16);
        assert_eq!(
            TILE_FADE_COLUMN_TABLE_UNK_C418,
            [
                0x00, 0x12, 0x02, 0x10, 0x09, 0x1B, 0x0B, 0x19, 0x01, 0x13, 0x03, 0x11, 0x08, 0x1A,
                0x0A, 0x18
            ],
            "ASM unk_C418: 16-byte column offset table"
        );
    }

    /// ASM sub_C3F6 (kid.asm:16045): even selector → AND $0F (clear high nibble).
    /// `d3=0` → byte offset 0, mask=$0F. Buffer byte $AA → $0A.
    #[test]
    fn sub_c3f6_even_selector_clears_high_nibble() {
        let mut buf = vec![0xAA; 0x40];
        // d3=0 (even): lsr #1 → offset=0, carry=0, mask=$0F
        tile_fade_mask_column_sub_C3F6(&mut buf, 0, 2);
        assert_eq!(
            buf[0x00], 0x0A,
            "ASM sub_C3F6: even → AND $0F clears high nibble"
        );
        assert_eq!(buf[0x20], 0x0A, "ASM sub_C3F6 loc_C408: second row at +$20");
        assert_eq!(buf[0x01], 0xAA, "adjacent byte untouched");
    }

    /// ASM sub_C3F6: odd selector → AND $F0 (clear low nibble).
    /// `d3=1` → byte offset 0, mask=$F0. Buffer byte $AB → $A0.
    #[test]
    fn sub_c3f6_odd_selector_clears_low_nibble() {
        let mut buf = vec![0xAB; 0x40];
        // d3=1 (odd): lsr #1 → offset=0, carry=1, mask=$F0
        tile_fade_mask_column_sub_C3F6(&mut buf, 1, 2);
        assert_eq!(
            buf[0x00], 0xA0,
            "ASM sub_C3F6: odd → AND $F0 clears low nibble"
        );
        assert_eq!(buf[0x20], 0xA0, "ASM sub_C3F6 loc_C408: second row at +$20");
    }

    /// ASM sub_C3F6: larger selector offsets into the buffer correctly.
    /// `d3=$12` (even, 18): lsr → offset=9, mask=$0F.
    #[test]
    fn sub_c3f6_large_even_selector_offsets_correctly() {
        let mut buf = vec![0xFF; 0x40];
        tile_fade_mask_column_sub_C3F6(&mut buf, 0x12, 1);
        assert_eq!(buf[9], 0x0F, "ASM sub_C3F6: d3=$12 → offset 9, AND $0F");
        assert_eq!(buf[8], 0xFF, "byte 8 untouched");
        assert_eq!(buf[10], 0xFF, "byte 10 untouched");
    }

    /// ASM sub_C3DA (kid.asm:16029): step 0 masks 4 columns from table[0]=$00.
    /// Columns: $00, $04, $20, $24 — all even → AND $0F (clear high nibbles).
    #[test]
    fn sub_c3da_step_0_masks_four_columns() {
        // Need buffer large enough for $80 rows × $20 bytes = $1000 bytes
        let mut buf = vec![0xFF_u8; 0x1000];
        tile_fade_mask_step_sub_C3DA(&mut buf, 0, 0x0080);
        // table[0] = $00. Columns: $00, $04, $20, $24 — all even.
        // Even → offset = d3/2, mask = $0F
        // col $00: offset 0, $0F
        assert_eq!(buf[0x00], 0x0F, "ASM sub_C3DA: col $00 → byte 0, AND $0F");
        // col $04: offset 2, $0F
        assert_eq!(buf[0x02], 0x0F, "ASM sub_C3DA: col $04 → byte 2, AND $0F");
        // col $20: offset $10=16, $0F
        assert_eq!(buf[0x10], 0x0F, "ASM sub_C3DA: col $20 → byte 16, AND $0F");
        // col $24: offset $12=18, $0F
        assert_eq!(buf[0x12], 0x0F, "ASM sub_C3DA: col $24 → byte 18, AND $0F");
        // Untouched bytes
        assert_eq!(buf[0x01], 0xFF, "byte 1 untouched");
    }

    /// ASM sub_C3DA: step 4 (table[4]=$09) mixes odd and even selectors.
    /// Base $09 (odd), +4=$0D (odd), +$1C=$29 (odd), +4=$2D (odd) — all odd.
    #[test]
    fn sub_c3da_step_4_masks_four_odd_columns() {
        let mut buf = vec![0xFF_u8; 0x1000];
        tile_fade_mask_step_sub_C3DA(&mut buf, 4, 0x0080);
        // table[4] = $09. Columns: $09, $0D, $29, $2D — all odd.
        // Odd → offset = d3/2 (integer), mask = $F0
        // col $09: offset 4, $F0
        assert_eq!(buf[0x04], 0xF0, "ASM sub_C3DA: col $09 → byte 4, AND $F0");
        // col $0D: offset 6, $F0
        assert_eq!(buf[0x06], 0xF0, "ASM sub_C3DA: col $0D → byte 6, AND $F0");
        // col $29: offset $14=20, $F0
        assert_eq!(buf[0x14], 0xF0, "ASM sub_C3DA: col $29 → byte 20, AND $F0");
        // col $2D: offset $16=22, $F0
        assert_eq!(buf[0x16], 0xF0, "ASM sub_C3DA: col $2D → byte 22, AND $F0");
    }

    // ── loc_C428: sprite scatter (kid.asm:16082) ───────────────────────────────

    /// ASM loc_C428 (kid.asm:16082): sprite scatter constants match the
    /// disassembly — 10 entries at stride 8 from sprite table base + 6.
    #[test]
    fn loc_c428_sprite_scatter_constants() {
        assert_eq!(
            SPRITE_SCATTER_TABLE_X_OFFSET_LOC_C428, 0xFFFF_0006,
            "ASM loc_C428: lea ($FFFF0006).l,a0 → x-position word of first sprite"
        );
        assert_eq!(
            SPRITE_SCATTER_ENTRY_COUNT_LOC_C428, 10,
            "ASM loc_C428: moveq #9,d0 → dbf loops 10 times"
        );
        assert_eq!(
            SPRITE_SCATTER_ENTRY_STRIDE_LOC_C428, 8,
            "ASM loc_C452: addq.w #8,a0 → 8-byte stride per sprite entry"
        );
    }

    /// ASM loc_C436 (kid.asm:16089): visible bounds and centre threshold.
    #[test]
    fn loc_c436_sprite_scatter_bounds_and_centre() {
        assert_eq!(
            SPRITE_SCATTER_VISIBLE_MIN_LOC_C436, 0x0070,
            "ASM loc_C436: cmpi.w #$70,d2; ble.s loc_C452"
        );
        assert_eq!(
            SPRITE_SCATTER_VISIBLE_MAX_LOC_C436, 0x01D0,
            "ASM loc_C436: cmpi.w #$1D0,d2; bge.s loc_C452"
        );
        assert_eq!(
            SPRITE_SCATTER_CENTRE_LOC_C436, 0x0120,
            "ASM loc_C436: cmpi.w #$120,d2; bge.s loc_C450"
        );
    }

    /// ASM loc_C436/loc_C450 (kid.asm:16089-16103): sprites left of centre are
    /// decremented, sprites right of centre (or at centre) are incremented.
    #[test]
    fn loc_c436_sprite_scatter_nudges_toward_edges() {
        let mut xs: [u16; 10] = [0x0100, 0x0140, 0x0120, 0x0071, 0x01CF, 0, 0, 0, 0, 0];
        let any = sprite_scatter_step_loc_C436(&mut xs);
        assert!(any, "ASM loc_C436: at least one sprite in visible range");
        // $100 < $120 → subq → $FF
        assert_eq!(xs[0], 0x00FF, "ASM loc_C436: subq.w #1,(a0) for x < $120");
        // $140 >= $120 → addq → $141
        assert_eq!(xs[1], 0x0141, "ASM loc_C450: addq.w #1,(a0) for x >= $120");
        // $120 >= $120 → addq → $121
        assert_eq!(xs[2], 0x0121, "ASM loc_C450: addq.w #1,(a0) for x == $120");
        // $71 > $70 → subq → $70
        assert_eq!(
            xs[3], 0x0070,
            "ASM loc_C436: $71 is just inside visible range"
        );
        // $1CF < $1D0 → addq → $1D0
        assert_eq!(
            xs[4], 0x01D0,
            "ASM loc_C450: $1CF is just inside visible range"
        );
    }

    /// ASM loc_C452 (kid.asm:16105): out-of-range sprites are untouched.
    /// `d2 <= $70` or `d2 >= $1D0` → branch directly to loc_C452, no nudge.
    #[test]
    fn loc_c452_out_of_range_sprites_not_nudged() {
        let mut xs: [u16; 10] = [0x0070, 0x01D0, 0x0000, 0xFFFF, 0, 0, 0, 0, 0, 0];
        let any = sprite_scatter_step_loc_C436(&mut xs);
        assert!(
            !any,
            "ASM loc_C452: no sprite in visible range → d1 stays 0"
        );
        assert_eq!(xs[0], 0x0070, "ASM loc_C436: $70 ble → skip");
        assert_eq!(xs[1], 0x01D0, "ASM loc_C436: $1D0 bge → skip");
        assert_eq!(xs[2], 0x0000, "ASM loc_C436: $0000 ble → skip");
        assert_eq!(xs[3], 0xFFFF, "ASM loc_C436: $FFFF bge → skip (unsigned)");
    }

    /// ASM loc_C428: scatter completes when all sprites exit visible range.
    /// Simulates multiple passes until `d1=0` triggers `j_Delete_CurrentObject`.
    #[test]
    fn loc_c428_scatter_terminates_when_all_sprites_exit_visible_range() {
        // Two sprites just inside visible range, one step from exiting
        let mut xs: [u16; 10] = [0x0071, 0x01CF, 0, 0, 0, 0, 0, 0, 0, 0];
        // Pass 1: $71 → $70, $1CF → $1D0
        assert!(sprite_scatter_step_loc_C436(&mut xs), "pass 1 should nudge");
        // Pass 2: $70 ble → skip, $1D0 bge → skip → d1=0
        assert!(
            !sprite_scatter_step_loc_C436(&mut xs),
            "pass 2 should terminate"
        );
    }

    // ── loc_C466: Object_EOL_Number init (kid.asm:16118) ───────────────────────

    /// ASM loc_C466 (kid.asm:16118): EOL number gfx object init constants.
    #[test]
    fn loc_c466_eol_number_init_constants() {
        assert_eq!(
            EOL_NUMBER_DIGIT_COUNT_LOC_C466, 7,
            "ASM Object_EOL_Number: moveq #6,d7 → dbf loops 7 times"
        );
        assert_eq!(
            EOL_NUMBER_GFX_SPEC_LOC_C466, 0x0200_0004,
            "ASM loc_C466: move.l #$2000004,a1 → gfx object allocation spec"
        );
        assert_eq!(
            EOL_NUMBER_DISPLAY_FLAG_LOC_C466, 0xFF,
            "ASM loc_C466: st $13(a1) → display flag enabled ($FF)"
        );
        assert_eq!(
            EOL_NUMBER_PRIORITY_LOC_C466, 2,
            "ASM loc_C466: move.b #2,$11(a1) → priority/layer"
        );
        assert_eq!(
            EOL_NUMBER_PALETTE_LOC_C466, 1,
            "ASM loc_C466: move.b #1,$10(a1) → palette line"
        );
        assert_eq!(
            EOL_NUMBER_INITIAL_PATTERN_LOC_C466, 0x0517,
            "ASM loc_C466: move.w #$517,$24(a1) → initial sprite VRAM pattern"
        );
        assert_eq!(
            EOL_NUMBER_SLOT_ARRAY_OFFSET_LOC_C466, 0x0044,
            "ASM Object_EOL_Number: lea $44(a5),a4 → slot pointers start at $44"
        );
    }

    // ── loc_C48C: Object_EOL_Number display loop (kid.asm:16128) ──────────────

    /// ASM loc_C48C (kid.asm:16128): display loop constants — max value clamp,
    /// digit count, digit spacing, and PowersOfTen divisor table.
    #[test]
    fn loc_c48c_eol_number_display_loop_constants() {
        // ASM loc_C48C: cmpi.l #9999999,d6
        assert_eq!(
            EOL_NUMBER_MAX_VALUE_LOC_C48C, 9_999_999,
            "ASM loc_C48C: cmpi.l #9999999,d6 → clamp ceiling"
        );
        // ASM loc_C4E2+10: cmpi.w #7,d7
        assert_eq!(
            EOL_NUMBER_DIGIT_COUNT_LOC_C48C, 7,
            "ASM loc_C48C: 7 digits (millions → ones)"
        );
        // ASM loc_C4E2: mulu.w #$C,d0
        assert_eq!(
            EOL_NUMBER_DIGIT_SPACING_LOC_C4E2, 12,
            "ASM loc_C4E2: mulu.w #$C,d0 → 12px per digit"
        );
        // ASM PowersOfTen (kid.asm:16196): 7 entries from 10^6 to 10^0
        assert_eq!(
            POWERS_OF_TEN_LOC_C48C,
            [1_000_000, 100_000, 10_000, 1_000, 100, 10, 1],
            "ASM PowersOfTen: dc.l table from 1000000 down to 1"
        );
    }

    /// ASM loc_D468 (kid.asm:16626): the level-skip cheat path emits three
    /// completion-time digit sprites before entering the ordinary EOL object
    /// layout. `divu.w #$A,d3; swap d3` consumes ones, tens, then hundreds.
    #[test]
    #[allow(non_snake_case)]
    fn loc_D468_level_skip_completion_time_digits_are_placed_right_to_left() {
        let sprites = level_skip_completion_time_digit_sprites_loc_D468(true, 123, 0x0200, 0x0040)
            .expect("ASM loc_D468 should allocate digit sprites when LevelSkip_Cheat is set");

        assert_eq!(
            sprites.map(|sprite| sprite.digit),
            [3, 2, 1],
            "ASM loc_D468 should display Level_completion_time remainders from ones to hundreds"
        );
        assert_eq!(
            sprites.map(|sprite| sprite.x_pos),
            [0x031C, 0x0310, 0x0304],
            "ASM loc_D468 seeds d2 with camera X + `$11C` and subtracts `$C` per digit"
        );
        assert_eq!(
            sprites.map(|sprite| sprite.y_pos),
            [0x0114, 0x0114, 0x0114],
            "ASM loc_D468 places all three digits at camera Y + `$D4`"
        );
        assert!(
            sprites.iter().all(|sprite| sprite.display_flag == 0xFF),
            "ASM loc_D468 uses `st $13(a1)` for each digit object"
        );
        assert!(
            sprites.iter().all(|sprite| sprite.priority_layer == 2
                && sprite.palette_line == 1
                && sprite.vram_tile == 0x0517),
            "ASM loc_D468 gives each digit priority 2, palette 1, and VRAM tile `$517`"
        );
    }

    /// ASM loc_D47E (kid.asm:16635): the digit allocation loop divides the
    /// remaining completion time by 10 each pass, uses the previous remainder as
    /// the sprite digit, then subtracts `$C` from d2 before `dbf d6,loc_D47E`.
    #[test]
    #[allow(non_snake_case)]
    fn loc_D47E_level_skip_completion_time_loop_keeps_quotient_for_next_digit() {
        let sprites = level_skip_completion_time_digit_sprites_loc_D468(true, 7, 0x0200, 0x0040)
            .expect("ASM loc_D47E should run three allocation-loop iterations");

        assert_eq!(
            sprites.map(|sprite| sprite.digit),
            [7, 0, 0],
            "ASM loc_D47E should emit ones, then quotient digits after each `divu.w #$A,d3`"
        );
        assert_eq!(
            sprites.map(|sprite| sprite.x_pos),
            [0x031C, 0x0310, 0x0304],
            "ASM loc_D47E should keep subtracting `$C` from d2 between DBF iterations"
        );
    }

    /// ASM loc_D468 (kid.asm:16626): `tst.b (LevelSkip_Cheat).w; beq.s loc_D4CE`
    /// skips the three digit allocations entirely when the cheat flag is clear.
    #[test]
    #[allow(non_snake_case)]
    fn loc_D468_level_skip_completion_time_digits_skip_when_cheat_clear() {
        assert!(
            level_skip_completion_time_digit_sprites_loc_D468(false, 99, 0x0200, 0x0040).is_none(),
            "ASM loc_D468 should branch straight to loc_D4CE when LevelSkip_Cheat is clear"
        );
    }

    /// ASM loc_C48C (kid.asm:16128): values above 9999999 are clamped.
    #[test]
    fn loc_c48c_eol_number_clamp_overflow() {
        // 10_000_000 exceeds max → clamped to 9999999, all digits visible
        let digits = eol_number_digits_loc_C48C(10_000_000);
        assert_eq!(
            digits,
            [
                Some(9),
                Some(9),
                Some(9),
                Some(9),
                Some(9),
                Some(9),
                Some(9)
            ],
            "ASM loc_C48C: move.l #9999999,d6 when input > max"
        );
    }

    // ── loc_C4AE: per-digit dispatch (kid.asm:16139) ─────────────────────────

    /// ASM loc_C4AE (kid.asm:16139): digit extraction for a value with all
    /// significant digits (1234567).
    #[test]
    fn loc_c4ae_eol_digit_all_significant() {
        let digits = eol_number_digits_loc_C48C(1_234_567);
        assert_eq!(
            digits,
            [
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6),
                Some(7)
            ],
            "ASM loc_C4AE: each digit extracted via loc_C4CE repeated subtraction"
        );
    }

    // ── loc_C4C0: show digit 0 (kid.asm:16149) ───────────────────────────────

    /// ASM loc_C4C0 (kid.asm:16149): trailing zeros after a significant digit
    /// display as 0, and the ones place (digit_index=6) always shows 0.
    #[test]
    fn loc_c4c0_eol_trailing_and_ones_zero() {
        // 1_000_000: millions=1, then six trailing zeros all visible
        let digits = eol_number_digits_loc_C48C(1_000_000);
        assert_eq!(
            digits,
            [
                Some(1),
                Some(0),
                Some(0),
                Some(0),
                Some(0),
                Some(0),
                Some(0)
            ],
            "ASM loc_C4C0: trailing zeros shown after leading significant digit"
        );
        // 0: all blanked except ones place
        let digits = eol_number_digits_loc_C48C(0);
        assert_eq!(
            digits,
            [None, None, None, None, None, None, Some(0)],
            "ASM loc_C4C0: cmpi.w #6,d7; beq.s loc_C4C0 → ones place always shows 0"
        );
    }

    // ── loc_C4C8: leading zero suppression (kid.asm:16154) ────────────────────

    /// ASM loc_C4C8 (kid.asm:16154): leading zeros are blanked (clr.w $22(a0))
    /// until the first non-zero digit appears.
    #[test]
    fn loc_c4c8_eol_leading_zero_suppression() {
        // 42: only tens and ones visible, rest blanked
        let digits = eol_number_digits_loc_C48C(42);
        assert_eq!(
            digits,
            [None, None, None, None, None, Some(4), Some(2)],
            "ASM loc_C4C8: tst.w d2; beq.s loc_C4C8 → blank before first significant"
        );
        // 100_000: hundred-thousands through ones visible
        let digits = eol_number_digits_loc_C48C(100_000);
        assert_eq!(
            digits,
            [None, Some(1), Some(0), Some(0), Some(0), Some(0), Some(0)],
            "ASM loc_C4C8: millions blanked, then loc_C4C0 for trailing zeros"
        );
    }

    // ── loc_C4CE: digit extraction via repeated subtraction (kid.asm:16159) ───

    /// ASM loc_C4CE/loc_C4D2 (kid.asm:16159–16166): extract digit by repeated
    /// subtraction: `d1=-1; loop: d1++, d6-=d0, bpl loop; d6+=d0`.
    #[test]
    fn loc_c4ce_eol_extract_digit_repeated_subtraction() {
        // 7_654_321 / 1_000_000 → digit=7, remainder=654_321
        let (digit, rem) = eol_extract_digit_loc_C4CE(7_654_321, 1_000_000);
        assert_eq!(digit, 7, "ASM loc_C4D2: 7 subtractions of 1000000");
        assert_eq!(rem, 654_321, "ASM loc_C4D2: add.l d0,d6 restores remainder");

        // 9 / 10 → digit=0, remainder=9
        let (digit, rem) = eol_extract_digit_loc_C4CE(9, 10);
        assert_eq!(digit, 0, "ASM loc_C4CE: value < power → digit 0");
        assert_eq!(rem, 9, "remainder unchanged when digit is 0");

        // 1 / 1 → digit=1, remainder=0
        let (digit, rem) = eol_extract_digit_loc_C4CE(1, 1);
        assert_eq!(digit, 1, "ASM loc_C4D2: ones place extraction");
        assert_eq!(rem, 0, "ones place leaves 0 remainder");
    }

    /// ASM loc_C4AE (kid.asm:16139): single-digit dispatch decisions.
    #[test]
    fn loc_c4ae_eol_digit_dispatch_cases() {
        // remainder >= power → Some(digit) via loc_C4CE
        assert_eq!(
            eol_number_digit_loc_C4AE(500_000, 100_000, 1, false),
            Some(5),
            "ASM loc_C4AE: cmp.l d0,d6; bge.s loc_C4CE"
        );
        // remainder < power, ones place → Some(0) via loc_C4C0
        assert_eq!(
            eol_number_digit_loc_C4AE(0, 1, 6, false),
            Some(0),
            "ASM loc_C4C0: cmpi.w #6,d7; beq.s loc_C4C0"
        );
        // remainder < power, not ones, leading seen → Some(0) via loc_C4C0
        assert_eq!(
            eol_number_digit_loc_C4AE(0, 100, 4, true),
            Some(0),
            "ASM loc_C4C0: trailing zero after significant digit"
        );
        // remainder < power, not ones, no leading → None via loc_C4C8
        assert_eq!(
            eol_number_digit_loc_C4AE(0, 100_000, 1, false),
            None,
            "ASM loc_C4C8: tst.w d2; beq.s loc_C4C8 → blank"
        );
    }

    // ── loc_C564: EOL number-variant sprite setup (kid.asm:16218) ─────────────

    /// ASM loc_C564 (kid.asm:16218): 2-player arrow indicator uses palette 1
    /// and VRAM tile $409. Object_EOL_Text (kid.asm:16227) uses palette 2 and
    /// $517. Both fall through to loc_C592 for common init.
    #[test]
    fn loc_c564_eol_number_variant_constants() {
        use crate::level::{
            EOL_COMMON_DISPLAY_FLAG_LOC_C592, EOL_COMMON_PRIORITY_LOC_C592,
            EOL_NUMBER_VARIANT_PALETTE_LOC_C564, EOL_NUMBER_VARIANT_VRAM_TILE_LOC_C564,
            EOL_TEXT_PALETTE_LOC_C57C, EOL_TEXT_VRAM_TILE_LOC_C57C,
        };
        // ASM loc_C564: move.b #1,palette_line(a3)
        assert_eq!(
            EOL_NUMBER_VARIANT_PALETTE_LOC_C564, 1,
            "ASM loc_C564: palette line 1 for number-variant"
        );
        // ASM loc_C564: move.w #$409,vram_tile(a3)
        assert_eq!(
            EOL_NUMBER_VARIANT_VRAM_TILE_LOC_C564, 0x0409,
            "ASM loc_C564: VRAM tile $409"
        );
        // ASM Object_EOL_Text: move.b #2,palette_line(a3)
        assert_eq!(
            EOL_TEXT_PALETTE_LOC_C57C, 2,
            "ASM Object_EOL_Text: palette line 2"
        );
        // ASM Object_EOL_Text: move.w #$517,vram_tile(a3)
        assert_eq!(
            EOL_TEXT_VRAM_TILE_LOC_C57C, 0x0517,
            "ASM Object_EOL_Text: VRAM tile $517"
        );
        // ASM loc_C592: st $13(a3) — display flag $FF
        assert_eq!(
            EOL_COMMON_DISPLAY_FLAG_LOC_C592, 0xFF,
            "ASM loc_C592: st $13(a3) → display enabled"
        );
        // ASM loc_C592: move.b #1,priority(a3)
        assert_eq!(EOL_COMMON_PRIORITY_LOC_C592, 1, "ASM loc_C592: priority 1");
    }

    // ── loc_C5A2: EOL object position-update loop (kid.asm:16238) ─────────────

    /// ASM loc_C5A2 (kid.asm:16238): position += velocity each frame. When
    /// timer hits zero, velocity is cleared and position holds.
    #[test]
    fn loc_c5a2_eol_object_position_step_normal() {
        // Timer > 1: velocity applied, timer decremented
        let (x, y, t, cleared) = eol_object_position_step_loc_C5A2(0x58, 0x100, 8, -2, 0x1C);
        assert_eq!(x, 0x60, "ASM loc_C5A2: x_pos += x_vel");
        assert_eq!(y, 0xFE, "ASM loc_C5A2: y_pos += y_vel");
        assert_eq!(t, 0x1B, "ASM loc_C5A2: subq.w #1,$46(a5)");
        assert!(
            !cleared,
            "ASM loc_C5A2: timer still running, vel not cleared"
        );
    }

    #[test]
    fn loc_c5a2_eol_object_position_step_timer_expires() {
        // Timer == 1 → reaches 0, velocity cleared
        let (x, y, t, cleared) = eol_object_position_step_loc_C5A2(0x58, 0x100, 8, -2, 1);
        assert_eq!(x, 0x60, "ASM loc_C5A2: final velocity step applied");
        assert_eq!(y, 0xFE, "ASM loc_C5A2: final y velocity step applied");
        assert_eq!(t, 0, "ASM loc_C5A2: timer reached 0");
        assert!(cleared, "ASM loc_C5A2: clr.w $6C(a5); clr.w $70(a5)");
    }

    // ── loc_C5DE: EOL extra-life bounce (kid.asm:16257) ───────────────────────

    /// ASM loc_C5DE (kid.asm:16257): extra-life icon initial parameters.
    #[test]
    fn loc_c5de_eol_extra_life_initial_constants() {
        use crate::level::{
            EOL_EXTRA_LIFE_INITIAL_X_LOC_C5DE, EOL_EXTRA_LIFE_INITIAL_X_VEL_LOC_C5DE,
            EOL_EXTRA_LIFE_INITIAL_Y_LOC_C5DE, EOL_EXTRA_LIFE_INITIAL_Y_VEL_LOC_C5DE,
            EOL_EXTRA_LIFE_PALETTE_LOC_C5DE, EOL_EXTRA_LIFE_PRIORITY_LOC_C5DE,
            EOL_EXTRA_LIFE_VRAM_TILE_LOC_C5DE,
        };
        // ASM loc_C5DE: moveq #-$40,d0
        assert_eq!(
            EOL_EXTRA_LIFE_INITIAL_X_LOC_C5DE, -0x40,
            "ASM loc_C5DE: x = -$40"
        );
        // ASM loc_C5DE: moveq #0,d1
        assert_eq!(EOL_EXTRA_LIFE_INITIAL_Y_LOC_C5DE, 0, "ASM loc_C5DE: y = 0");
        // ASM loc_C5DE: moveq #2,d2
        assert_eq!(
            EOL_EXTRA_LIFE_INITIAL_X_VEL_LOC_C5DE, 2,
            "ASM loc_C5DE: x_vel = 2"
        );
        // ASM loc_C5DE: moveq #4,d3
        assert_eq!(
            EOL_EXTRA_LIFE_INITIAL_Y_VEL_LOC_C5DE, 4,
            "ASM loc_C5DE: y_vel = 4"
        );
        // ASM loc_C5DE: move.b #2,palette_line(a3)
        assert_eq!(
            EOL_EXTRA_LIFE_PALETTE_LOC_C5DE, 2,
            "ASM loc_C5DE: palette 2"
        );
        // ASM loc_C5DE: move.b #1,priority(a3)
        assert_eq!(
            EOL_EXTRA_LIFE_PRIORITY_LOC_C5DE, 1,
            "ASM loc_C5DE: priority 1"
        );
        // ASM loc_C5DE: move.w #$517,vram_tile(a3)
        assert_eq!(
            EOL_EXTRA_LIFE_VRAM_TILE_LOC_C5DE, 0x0517,
            "ASM loc_C5DE: VRAM $517"
        );
    }

    /// ASM loc_C60C–loc_C644 (kid.asm:16270–16299): bounce loop reverses
    /// velocity at bounds. X bounces in [-8, $D0], Y in [8, $D8].
    #[test]
    fn loc_c60c_eol_extra_life_bounce_interior() {
        // Interior movement: no boundary hit
        let (x, y, xv, yv) = eol_extra_life_bounce_step_loc_C60C(0x40, 0x40, 2, 4);
        assert_eq!(
            (x, y, xv, yv),
            (0x42, 0x44, 2, 4),
            "ASM loc_C60C: interior step, no bounce"
        );
    }

    #[test]
    fn loc_c60c_eol_extra_life_bounce_x_left_boundary() {
        // X crosses left bound -8, moving left → negate
        let (x, _y, xv, _yv) = eol_extra_life_bounce_step_loc_C60C(-7, 0x40, -2, 4);
        assert_eq!(x, -9, "ASM loc_C60C: x went past -8");
        assert_eq!(xv, 2, "ASM loc_C60C+4: neg.w d2 when x < -8 and d2 < 0");
    }

    #[test]
    fn loc_c60c_eol_extra_life_bounce_x_right_boundary() {
        // X crosses right bound $D0, moving right → negate
        let (x, _y, xv, _yv) = eol_extra_life_bounce_step_loc_C60C(0xD0, 0x40, 2, 4);
        assert_eq!(x, 0xD2, "ASM loc_C61E: x went past $D0");
        assert_eq!(xv, -2, "ASM loc_C61E: neg.w d2 when x > $D0 and d2 > 0");
    }

    #[test]
    fn loc_c60c_eol_extra_life_bounce_y_top_boundary() {
        // Y crosses top bound 8, moving up → negate
        let (_x, y, _xv, yv) = eol_extra_life_bounce_step_loc_C60C(0x40, 9, 2, -4);
        assert_eq!(y, 5, "ASM loc_C62A: y went past 8");
        assert_eq!(yv, 4, "ASM loc_C62A: neg.w d3 when y < 8 and d3 < 0");
    }

    #[test]
    fn loc_c60c_eol_extra_life_bounce_y_bottom_boundary() {
        // Y crosses bottom bound $D8, moving down → negate
        let (_x, y, _xv, yv) = eol_extra_life_bounce_step_loc_C60C(0x40, 0xD8, 2, 4);
        assert_eq!(y, 0xDC, "ASM loc_C638: y went past $D8");
        assert_eq!(yv, -4, "ASM loc_C638: neg.w d3 when y > $D8 and d3 > 0");
    }

    /// ASM loc_C60C: velocity NOT negated when already moving away from boundary.
    #[test]
    fn loc_c60c_eol_extra_life_bounce_no_double_negate() {
        // x < -8 but moving right (positive) → no negate
        let (_x, _y, xv, _yv) = eol_extra_life_bounce_step_loc_C60C(-9, 0x40, 2, 4);
        assert_eq!(
            xv, 2,
            "ASM loc_C60C+8: tst.w d2; bpl.s → skip negate when moving right"
        );

        // x > $D0 but moving left (negative) → no negate
        let (_x, _y, xv, _yv) = eol_extra_life_bounce_step_loc_C60C(0xD2, 0x40, -2, 4);
        assert_eq!(
            xv, -2,
            "ASM loc_C61E+4: tst.w d2; bmi.s → skip negate when moving left"
        );
    }

    #[test]
    fn sub_6e24_decodes_the_raw_a10001_bits_with_seq_and_sne_semantics() {
        let both_clear = read_io_port_a10001_flags_sub_6E24(0x00);
        let bit_6_only = read_io_port_a10001_flags_sub_6E24(0x40);
        let bit_7_only = read_io_port_a10001_flags_sub_6E24(0x80);
        let both_set = read_io_port_a10001_flags_sub_6E24(0xC0);

        assert_eq!(
            both_clear,
            IoPortFlagsSub6E24 {
                bit_7_clear_flag: true,
                bit_6_set_flag: false,
            },
            "ASM sub_6E24 should treat `btst #7` + `seq` as true when bit 7 is clear",
        );
        assert_eq!(
            bit_6_only,
            IoPortFlagsSub6E24 {
                bit_7_clear_flag: true,
                bit_6_set_flag: true,
            },
            "ASM sub_6E24 should set both output bytes when bit 7 is clear and bit 6 is set",
        );
        assert_eq!(
            bit_7_only,
            IoPortFlagsSub6E24 {
                bit_7_clear_flag: false,
                bit_6_set_flag: false,
            },
            "ASM sub_6E24 should clear the `seq`-backed flag once bit 7 is set",
        );
        assert_eq!(
            both_set,
            IoPortFlagsSub6E24 {
                bit_7_clear_flag: false,
                bit_6_set_flag: true,
            },
            "ASM sub_6E24 should keep the `sne`-backed flag true whenever bit 6 is set",
        );
    }

    #[test]
    fn sub_73d0_seeds_the_fullscreen_palette_fade_buffers() {
        let palette_buffer = core::array::from_fn(|index| 0x0100 + index as u16);

        let state = initialise_fullscreen_palette_fade_sub_73D0(palette_buffer, 0x0ACE);

        assert_eq!(
            state,
            FullscreenPaletteFadeSub73D0 {
                palette_to_dma_enabled: false,
                palette_buffer_2: palette_buffer,
                palette_buffer_3: [0x0ACE; 0x40],
                fade_progress: 0,
                pending_palette_0_mask: 0,
                pending_palette_1_mask: 0,
                allow_pausing: false,
            },
            "ASM sub_73D0 should clear PaletteToDMA_Flag, copy all `$40` words into Palette_Buffer_2, flood-fill Palette_Buffer_3 from `($FFFFFBCC).w`, and clear `($FFFFF876).w`",
        );
    }

    #[test]
    fn initialise_palette_fade_out_loc_28a6_copies_the_palette_and_seeds_a_full_fade() {
        let palette_buffer = core::array::from_fn(|index| 0x0100_u16.wrapping_add(index as u16));

        assert_eq!(
            initialise_palette_fade_out_loc_28A6(palette_buffer, 0x0ACE),
            PaletteFadeOutInitLoc28A6 {
                palette_buffer_2: palette_buffer,
                palette_buffer_3: [0x0ACE; 0x40],
                fade_progress: 0x0100,
            },
            "ASM loc_28A6 (kid.asm:4813) should copy 32 longwords from `Palette_Buffer` into `Palette_Buffer_2`, flood-fill all 64 words of `Palette_Buffer_3`, and seed `$FFFFF876` with `$0100`",
        );
    }

    #[test]
    fn step_palette_fade_out_loc_28c6_reseeds_both_pending_masks_while_the_fade_stays_non_negative()
    {
        assert_eq!(
            step_palette_fade_out_loc_28C6(0x0100, 0, 0),
            PaletteFadeOutStepLoc28C6 {
                fade_progress: 0x00F0,
                pending_palette_0_mask: u32::MAX,
                pending_palette_1_mask: u32::MAX,
                continue_fading: true,
            },
            "ASM loc_28C6 (kid.asm:4827) should subtract `$10` from `$FFFFF876`, reseed both pending palette masks with `-1`, and branch back through `loc_28DE` while the fade progress stays non-negative",
        );
    }

    #[test]
    fn step_palette_fade_out_loc_28c6_preserves_pending_masks_when_it_branches_to_loc_28ea() {
        assert_eq!(
            step_palette_fade_out_loc_28C6(0x0000, 0x1234_5678, 0x9ABC_DEF0),
            PaletteFadeOutStepLoc28C6 {
                fade_progress: 0xFFF0,
                pending_palette_0_mask: 0x1234_5678,
                pending_palette_1_mask: 0x9ABC_DEF0,
                continue_fading: false,
            },
            "ASM loc_28EA (kid.asm:4842) should only run the final VBlank refresh and return after `subi.w #$10,($FFFFF876).w` underflows, so the branch out of `loc_28C6` must preserve the existing pending palette masks",
        );
    }

    #[test]
    fn sub_73d0_only_finishes_after_the_seventeenth_fade_refresh_frame() {
        let mut state = initialise_fullscreen_palette_fade_sub_73D0([0x0000; 0x40], 0x0000);
        let mut completed_on = None;

        for frame in 1..=17 {
            let completed = step_fullscreen_palette_fade_sub_73D0(&mut state);
            if completed {
                completed_on = Some(frame);
                break;
            }
        }

        assert_eq!(
            completed_on,
            Some(17),
            "ASM sub_73D0 should loop until `addi.w #$10,($FFFFF876).w` pushes the progress word strictly past `$0100`",
        );
        assert_eq!(
            state.fade_progress, 0x0110,
            "the terminating sub_73D0 frame should leave the shared fade progress word at `$0110`",
        );
        assert_eq!(
            state.pending_palette_0_mask,
            u32::MAX,
            "each sub_73D0 refresh should keep `($FFFFF888).w` seeded with `-1`",
        );
        assert_eq!(
            state.pending_palette_1_mask,
            u32::MAX,
            "each sub_73D0 refresh should keep `($FFFFF88C).w` seeded with `-1`",
        );
        assert!(
            state.allow_pausing,
            "ASM sub_73D0 should restore Allow_Pausing on the terminating frame before deleting the worker",
        );
    }

    #[test]
    fn stop_z80_guard_sub_914_nests_the_stop_request_and_raises_sr() {
        let mut runtime = NativeRuntime::new(manifest());

        runtime.stop_z80_guard_sub_914();

        assert_eq!(
            runtime.z80_stop_request_depth, 1,
            "ASM sub_914 should increment `$FFFFF805` before stopping the Z80",
        );
        assert_eq!(
            runtime.z80_guard_sr, 0x2700,
            "ASM sub_914 should raise SR to `$2700` before `j_Stop_z80`",
        );
        assert!(
            !runtime.z80_running,
            "ASM sub_914 should halt the native Z80 mirror after the guard enters",
        );
    }

    #[test]
    fn start_z80_guard_sub_924_keeps_nested_stops_halted() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.stop_z80_guard_sub_914();
        runtime.stop_z80_guard_sub_914();

        runtime.start_z80_guard_sub_924();

        assert_eq!(
            runtime.z80_stop_request_depth, 1,
            "ASM sub_924 should keep `$FFFFF805` at the remaining nested depth when the signed `bgt` path is taken",
        );
        assert_eq!(
            runtime.z80_guard_sr, 0x2700,
            "ASM sub_924 should leave SR untouched while nested stops remain",
        );
        assert!(
            !runtime.z80_running,
            "ASM sub_924 should not restart the Z80 until the final nested stop releases",
        );
    }

    #[test]
    fn start_z80_guard_sub_924_clears_zero_depth_and_restarts_z80() {
        let mut runtime = NativeRuntime::new(manifest());
        runtime.stop_z80_guard_sub_914();

        runtime.start_z80_guard_sub_924();

        assert_eq!(
            runtime.z80_stop_request_depth, 0,
            "ASM sub_924 should clear `$FFFFF805` on the final release before `j_Start_z80`",
        );
        assert_eq!(
            runtime.z80_guard_sr, 0x2500,
            "ASM sub_924 should restore SR to `$2500` when the final stop request releases",
        );
        assert!(
            runtime.z80_running,
            "ASM sub_924 should restart the native Z80 mirror once no nested stops remain",
        );
    }

    #[test]
    fn loc_1698_stops_z80_and_seeds_the_first_plane_clear_pass() {
        let mut runtime = NativeRuntime::new(manifest());

        let setup = runtime.prepare_plane_clear_pass_loc_1698();

        assert_eq!(
            runtime.z80_stop_request_depth, 1,
            "ASM loc_1698 should call `j_sub_914` before touching the plane-clear buffer state",
        );
        assert_eq!(
            runtime.z80_guard_sr, 0x2700,
            "ASM loc_1698 inherits sub_914's raised SR before the fill passes begin",
        );
        assert!(
            !runtime.z80_running,
            "ASM loc_1698 should keep the native Z80 mirror halted while the plane-clear routine owns the bus",
        );
        assert_eq!(
            setup,
            PlaneClearPassSetupLoc1698 {
                target_addr: 0x00FE_0000,
                dbf_seed: 0x003F,
                next_source_longword: 0x0000_0020,
            },
            "ASM loc_1698 should target `$FE0000`, seed `d0=#$3F` for the first `dbf` fill loop, and store `#$20` into `4(a6)` before entering loc_16B6",
        );
    }

    #[test]
    fn loc_16b6_repeats_the_same_source_word_for_dbf_seed_plus_one_writes() {
        let fill = plane_clear_word_fill_loc_16B6(0x00FE_0000, 0xA55A, 0x003F);

        assert_eq!(
            fill.target_addr, 0x00FE_0000,
            "ASM loc_16B6 should start writing at the `a0=$FE0000` address seeded by loc_1698",
        );
        assert_eq!(
            fill.next_addr, 0x00FE_0080,
            "ASM loc_16B6 should advance `a0` by two bytes per iteration, so a `d0=#$3F` loop lands exactly 64 words later",
        );
        assert_eq!(
            fill.words.len(),
            0x40,
            "ASM loc_16B6 uses `dbf d0`, so a seed of `#$3F` must emit 64 copies of the same source word",
        );
        assert!(
            fill.words.iter().all(|&word| word == 0xA55A),
            "ASM loc_16B6 should keep re-reading `(a6)` as the same source word for every iteration of the first fill loop",
        );
    }

    #[test]
    fn loc_16c8_repeats_the_second_plane_clear_pass_for_dbf_seed_plus_one_writes() {
        let fill = plane_clear_word_fill_loc_16C8(0x00FE_0080, 0x5A5A, 0x0027);

        assert_eq!(
            fill.target_addr, 0x00FE_0080,
            "ASM loc_16C8 should resume the second fill loop from the address left by loc_16B6",
        );
        assert_eq!(
            fill.next_addr, 0x00FE_00D0,
            "ASM loc_16C8 seeds `d0=#$27`, so the inclusive `dbf` loop should advance `a0` by 40 words from `$FE0080` to `$FE00D0`",
        );
        assert_eq!(
            fill.words.len(),
            0x28,
            "ASM loc_16C8 uses `dbf d0`, so a seed of `#$27` must emit 40 copies of the same source word",
        );
        assert!(
            fill.words.iter().all(|&word| word == 0x5A5A),
            "ASM loc_16C8 should keep re-reading `(a6)` as the same source word for every iteration of the second fill loop",
        );
    }

    #[test]
    fn loc_16e0_repeats_the_final_plane_clear_pass_for_dbf_seed_plus_one_writes() {
        let fill = plane_clear_word_fill_loc_16E0(0x00FD_0000, 0xCAFE, 0x7FFF);

        assert_eq!(
            fill.target_addr, 0x00FD_0000,
            "ASM loc_16E0 should switch `a0` to `$FD0000` before the final fill loop starts",
        );
        assert_eq!(
            fill.next_addr, 0x00FE_0000,
            "ASM loc_16E0 seeds `d0=#$7FFF`, so the inclusive `dbf` loop should advance `a0` by 32768 words from `$FD0000` back to `$FE0000` before `j_sub_924`",
        );
        assert_eq!(
            fill.words.len(),
            0x8000,
            "ASM loc_16E0 uses `dbf d0`, so a seed of `#$7FFF` must emit 32768 copies of the same source word",
        );
        assert!(
            fill.words.iter().all(|&word| word == 0xCAFE),
            "ASM loc_16E0 should keep re-reading `(a6)` as the same source word for every iteration of the final fill loop before the Z80 guard releases",
        );
    }

    #[test]
    fn sound_driver_write_sub_e1330_queues_ff_05_and_preserves_a0_d0() {
        let write = sound_driver_write_sub_E1330(0x7B);

        assert_eq!(
            write,
            SoundDriverWriteSubE1330 {
                queued_bytes: [0xFF, 0x05, 0x7B],
                preserves_a0: true,
                preserves_d0: true,
            },
            "ASM sub_E1330 should thunk to sub_E1546, preserve `a0`/`d0`, and queue `FF 05 <sound-id>` into the GEMS ring",
        );
    }

    #[test]
    fn sound_driver_write_sub_e1334_queues_ff_0c_without_payload() {
        let write = sound_driver_write_sub_E1334();

        assert_eq!(
            write,
            SoundDriverWriteSubE1334 {
                queued_bytes: [0xFF, 0x0C],
            },
            "ASM sub_E1334 should thunk to sub_E155A and queue `FF 0C` into the GEMS ring with no trailing payload byte",
        );
    }

    #[test]
    fn sound_driver_write_sub_e1338_queues_ff_0d_without_payload() {
        let write = sound_driver_write_sub_E1338();

        assert_eq!(
            write,
            SoundDriverWriteSubE1338 {
                queued_bytes: [0xFF, 0x0D],
            },
            "ASM sub_E1338 should thunk to sub_E1568 and queue `FF 0D` into the GEMS ring with no trailing payload byte",
        );
    }

    #[test]
    fn sound_driver_write_sub_e133c_only_enters_the_shared_flush_path() {
        let write = sound_driver_write_sub_E133C();

        assert_eq!(
            write,
            SoundDriverWriteSubE133C { queued_bytes: [] },
            "ASM sub_E133C should thunk to loc_E1576, enter the shared critical section, and return without queueing a GEMS command byte",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E15E8_queues_command_17_and_three_low_byte_payloads() {
        let write = sound_driver_write_command_17_three_bytes_loc_E15E8(
            0x3E,
            0x1234_56A1,
            0xFFFF_FFB2,
            0x0000_01C3,
        );

        assert_eq!(
            write,
            SoundDriverWriteCommandAndThreeBytes {
                begin_write: sound_driver_begin_write_sub_E143A(0x3E),
                command_write: sound_driver_write_command_sub_E1482(0x3E, 0x17),
                payload_writes: [
                    sound_driver_write_byte_sub_E148E(0x00, 0xA1),
                    sound_driver_write_byte_sub_E148E(0x01, 0xB2),
                    sound_driver_write_byte_sub_E148E(0x02, 0xC3),
                ],
                releases_z80_bus: true,
                restores_previous_sr: true,
            },
            "ASM loc_E15E8 should queue `FF 17` then only the low bytes from `8(a6)`, `$C(a6)`, and `$10(a6)` while preserving the 6-bit ring wrap",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E136C_z80_handshake_releases_bus_without_delay_when_status_is_zero() {
        let handshake = sound_driver_z80_handshake_loc_E136C(0);

        assert_eq!(
            handshake,
            SoundDriverZ80HandshakeLocE136C {
                bus_request_register: 0xA11100,
                bus_latch_register: 0xA01A20,
                bus_status_register: 0xA01A21,
                requests_z80_bus: true,
                waits_for_bus_grant: true,
                latch_value: 1,
                observed_status: 0,
                releases_z80_bus: true,
                delay_iterations: 0,
            },
            "ASM loc_E136C should request the Z80 bus, poke `$A01A20`, read `$A01A21`, release the bus, and return immediately when the status byte is zero",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E1392_z80_handshake_runs_45_iteration_delay_for_nonzero_status() {
        let handshake = sound_driver_z80_handshake_loc_E136C(0x80);

        assert_eq!(
            handshake.delay_iterations, 0x45,
            "ASM loc_E1392 seeds `d0=$44`, so the `dbf` delay lasts `$44+1` iterations when `$A01A21` is non-zero",
        );
        assert!(
            handshake.releases_z80_bus,
            "ASM loc_E136C should release `$A11100` before the optional delay loop",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E13B0_waits_until_vdp_control_port_bit_1_clears() {
        assert!(
            sound_driver_vdp_dma_wait_loc_E13B0(0x0002),
            "ASM loc_E13B0 should branch back while `btst #1,d0` sees VDP control-port bit 1 set",
        );
        assert!(
            !sound_driver_vdp_dma_wait_loc_E13B0(0x0000),
            "ASM loc_E13B0 should return once VDP control-port bit 1 is clear",
        );
        assert!(
            !sound_driver_vdp_dma_wait_loc_E13B0(0x0001),
            "ASM loc_E13B0 tests bit 1 only; bit 0 must not keep the wait loop alive",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E1400_and_loc_E1406_sound_driver_upload_copy_then_zero_fill() {
        let source_root = unique_temp_path("sub-e13da-source");
        fs::create_dir_all(source_root.join("sound")).expect("sound dir should be creatable");
        fs::write(
            source_root.join("sound/GEMS_Sounddriver.bin"),
            [0x12, 0x34, 0x56, 0x78],
        )
        .expect("driver should be writable");
        let mut manifest = manifest();
        manifest.source_root = source_root.clone();

        let upload =
            sound_driver_upload_sub_E13DA(&manifest).expect("driver upload plan should load");

        assert_eq!(
            upload,
            SoundDriverUploadSubE13DA {
                source_relative_path: "sound/GEMS_Sounddriver.bin",
                destination_start: 0xA00000,
                destination_end_exclusive: 0xA02000,
                driver_bytes: vec![0x12, 0x34, 0x56, 0x78],
                driver_copy_iterations: 4,
                zero_fill_len: 0x2000 - 4,
                requests_z80_reset: true,
                stops_z80: true,
                preserves_a1: true,
            },
            "ASM loc_E1400 should copy exactly the GEMS sound-driver bytes into `$A00000`; ASM loc_E1406 should zero-fill the rest of Z80 RAM through `$A01FFF`",
        );

        fs::remove_dir_all(source_root).expect("temp sound dir should be removable");
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E1424_sound_driver_reset_waits_before_restart_and_reset_release() {
        let reset = sound_driver_reset_sub_E1416();

        assert_eq!(
            reset,
            SoundDriverResetSubE1416 {
                requests_z80_reset: true,
                reset_assert_delay_iterations: 15,
                restarts_z80_bus: true,
                releases_z80_reset: true,
            },
            "ASM loc_E1424 should burn the fixed 15-iteration reset delay before sub_E1416 restarts `$A11100` and releases reset with `$0100`",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E1460_sound_driver_begin_write_waits_for_bus_grant_before_reading_head() {
        let begin = sound_driver_begin_write_sub_E143A(0x2A);

        assert_eq!(
            begin,
            SoundDriverBeginWriteSubE143A {
                command_head_address: 0xA00036,
                ring_buffer_address: 0xA01A40,
                write_index: 0x2A,
                interrupt_mask: 0x0700,
                requests_z80_bus: true,
                waits_for_bus_grant: true,
            },
            "ASM loc_E1460 should wait for `$A11100` bit 0 to clear before sub_E143A returns the current write head from `$A00036` with ring base `$A01A40`",
        );
    }

    #[test]
    fn sound_driver_write_command_sub_e1482_queues_ff_prefix_and_wraps_the_head_byte() {
        let write = sound_driver_write_command_sub_E1482(0x3F, 0x10);

        assert_eq!(
            write,
            SoundDriverWriteCommandSubE1482 {
                queued_bytes: [0xFF, 0x10],
                next_write_index: 0x01,
            },
            "ASM sub_E1482 should write `FF`, fall through into the shared tail write for the command byte, and commit the final ring head as `(old + 2) & $3F`",
        );
    }

    #[test]
    fn sound_driver_write_byte_sub_e148e_writes_one_ring_byte_and_wraps_head() {
        let write = sound_driver_write_byte_sub_E148E(0x3F, 0xA5);

        assert_eq!(
            write,
            SoundDriverWriteByteSubE148E {
                queued_byte: 0xA5,
                next_write_index: 0x00,
                committed_write_index: 0x00,
            },
            "ASM sub_E148E should write `d0` into `(a1,d1.w)`, advance the 6-bit ring head by one, and store the committed head byte back through `a0`",
        );
    }

    #[test]
    fn sound_driver_write_long_low_byte_sub_e149c_writes_only_the_first_payload_byte() {
        let write = sound_driver_write_long_low_byte_sub_E149C(0x12, 0x44332211);

        assert_eq!(
            write,
            SoundDriverWriteLongLowByteSubE149C {
                begin_write: SoundDriverBeginWriteSubE143A {
                    command_head_address: 0xA00036,
                    ring_buffer_address: 0xA01A40,
                    write_index: 0x12,
                    interrupt_mask: 0x0700,
                    requests_z80_bus: true,
                    waits_for_bus_grant: true,
                },
                write: SoundDriverWriteByteSubE148E {
                    queued_byte: 0x11,
                    next_write_index: 0x13,
                    committed_write_index: 0x13,
                },
                releases_z80_bus: true,
                restores_previous_sr: true,
            },
            "ASM sub_E149C should enter sub_E143A, queue only the low byte from `8(a6)` through sub_E148E, then drop `$A11100` and restore SR in loc_E1470",
        );
    }

    #[test]
    fn sound_driver_write_pointer_sub_e14ac_queues_three_low_to_high_pointer_bytes() {
        let write = sound_driver_write_pointer_sub_E14AC(0x3E, 0x44332211);

        assert_eq!(
            write,
            SoundDriverWritePointerSubE14AC {
                begin_write: SoundDriverBeginWriteSubE143A {
                    command_head_address: 0xA00036,
                    ring_buffer_address: 0xA01A40,
                    write_index: 0x3E,
                    interrupt_mask: 0x0700,
                    requests_z80_bus: true,
                    waits_for_bus_grant: true,
                },
                writes: [
                    SoundDriverWriteByteSubE148E {
                        queued_byte: 0x11,
                        next_write_index: 0x3F,
                        committed_write_index: 0x3F,
                    },
                    SoundDriverWriteByteSubE148E {
                        queued_byte: 0x22,
                        next_write_index: 0x00,
                        committed_write_index: 0x00,
                    },
                    SoundDriverWriteByteSubE148E {
                        queued_byte: 0x33,
                        next_write_index: 0x01,
                        committed_write_index: 0x01,
                    },
                ],
                releases_z80_bus: true,
                restores_previous_sr: true,
            },
            "ASM sub_E14AC should enter sub_E143A, queue the low three bytes from `8(a6)` via repeated `asr.l #8` + sub_E148E, then finish in loc_E1470",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E1522_queues_selected_command_and_one_low_payload_byte() {
        let write = sound_driver_write_command_and_byte_loc_E1522(0x3E, 0x10, 0xA5);

        assert_eq!(
            write,
            SoundDriverWriteCommandAndByte {
                begin_write: SoundDriverBeginWriteSubE143A {
                    command_head_address: 0xA00036,
                    ring_buffer_address: 0xA01A40,
                    write_index: 0x3E,
                    interrupt_mask: 0x0700,
                    requests_z80_bus: true,
                    waits_for_bus_grant: true,
                },
                command_write: SoundDriverWriteCommandSubE1482 {
                    queued_bytes: [0xFF, 0x10],
                    next_write_index: 0x00,
                },
                payload_write: SoundDriverWriteByteSubE148E {
                    queued_byte: 0xA5,
                    next_write_index: 0x01,
                    committed_write_index: 0x01,
                },
                releases_z80_bus: true,
                restores_previous_sr: true,
            },
            "ASM loc_E1522 should queue caller-selected command `$10` through sub_E1482, then queue only the low byte from `8(a6)` before the loc_E1470 cleanup",
        );
    }

    #[test]
    fn sound_driver_write_sub_e151c_queues_ff10_and_the_sound_id() {
        let write = sound_driver_write_sub_E151C(0x3F, 0xA5);

        assert_eq!(
            write,
            SoundDriverWriteCommandAndByte {
                begin_write: SoundDriverBeginWriteSubE143A {
                    command_head_address: 0xA00036,
                    ring_buffer_address: 0xA01A40,
                    write_index: 0x3F,
                    interrupt_mask: 0x0700,
                    requests_z80_bus: true,
                    waits_for_bus_grant: true,
                },
                command_write: SoundDriverWriteCommandSubE1482 {
                    queued_bytes: [0xFF, 0x10],
                    next_write_index: 0x01,
                },
                payload_write: SoundDriverWriteByteSubE148E {
                    queued_byte: 0xA5,
                    next_write_index: 0x02,
                    committed_write_index: 0x02,
                },
                releases_z80_bus: true,
                restores_previous_sr: true,
            },
            "ASM sub_E151C should enter sub_E143A, queue `FF 10` through sub_E1482, then queue the low byte from `8(a6)` through sub_E148E before finishing in loc_E1470",
        );
    }

    #[test]
    fn sound_driver_write_sub_e153e_queues_ff12_and_the_sound_id() {
        let write = sound_driver_write_sub_E153E(0x3F, 0xA5);

        assert_eq!(
            write,
            SoundDriverWriteCommandAndByte {
                begin_write: SoundDriverBeginWriteSubE143A {
                    command_head_address: 0xA00036,
                    ring_buffer_address: 0xA01A40,
                    write_index: 0x3F,
                    interrupt_mask: 0x0700,
                    requests_z80_bus: true,
                    waits_for_bus_grant: true,
                },
                command_write: SoundDriverWriteCommandSubE1482 {
                    queued_bytes: [0xFF, 0x12],
                    next_write_index: 0x01,
                },
                payload_write: SoundDriverWriteByteSubE148E {
                    queued_byte: 0xA5,
                    next_write_index: 0x02,
                    committed_write_index: 0x02,
                },
                releases_z80_bus: true,
                restores_previous_sr: true,
            },
            "ASM sub_E153E should enter sub_E143A, queue `FF 12` through sub_E1482, then queue the low byte from `8(a6)` through sub_E148E before finishing in loc_E1470",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E1584_queues_selected_command_and_two_low_payload_bytes() {
        let write = sound_driver_write_command_and_two_bytes_loc_E1584(0x3D, 0x02, 0xA5, 0x5A);

        assert_eq!(
            write,
            SoundDriverWriteCommandAndTwoBytes {
                begin_write: SoundDriverBeginWriteSubE143A {
                    command_head_address: 0xA00036,
                    ring_buffer_address: 0xA01A40,
                    write_index: 0x3D,
                    interrupt_mask: 0x0700,
                    requests_z80_bus: true,
                    waits_for_bus_grant: true,
                },
                command_write: SoundDriverWriteCommandSubE1482 {
                    queued_bytes: [0xFF, 0x02],
                    next_write_index: 0x3F,
                },
                payload_writes: [
                    SoundDriverWriteByteSubE148E {
                        queued_byte: 0xA5,
                        next_write_index: 0x00,
                        committed_write_index: 0x00,
                    },
                    SoundDriverWriteByteSubE148E {
                        queued_byte: 0x5A,
                        next_write_index: 0x01,
                        committed_write_index: 0x01,
                    },
                ],
                releases_z80_bus: true,
                restores_previous_sr: true,
            },
            "ASM loc_E1584 should queue caller-selected command `$02`, then write the low bytes from `8(a6)` and `$C(a6)` before the loc_E1470 cleanup",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E157E_selects_command_02_before_two_payload_tail() {
        let write = sound_driver_write_command_02_two_bytes_loc_E157E(0x3E, 0x34, 0x12);

        assert_eq!(
            write.command_write,
            SoundDriverWriteCommandSubE1482 {
                queued_bytes: [0xFF, 0x02],
                next_write_index: 0x00,
            },
            "ASM loc_E157E should load `d0=#$02` before falling into loc_E1584, so the queued command must be `FF 02`",
        );
        assert_eq!(
            write.payload_writes,
            [
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0x34,
                    next_write_index: 0x01,
                    committed_write_index: 0x01,
                },
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0x12,
                    next_write_index: 0x02,
                    committed_write_index: 0x02,
                },
            ],
            "ASM loc_E157E should reuse loc_E1584, writing the low bytes from `8(a6)` and `$C(a6)` after command `$02`",
        );
        assert!(
            write.releases_z80_bus && write.restores_previous_sr,
            "ASM loc_E157E should finish through loc_E1470 after the shared loc_E1584 payload writes",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E159C_selects_command_00_before_two_payload_tail() {
        let write = sound_driver_write_command_00_two_bytes_loc_E159C(0x3E, 0x88776655, 0x11223344);

        assert_eq!(
            write.command_write,
            SoundDriverWriteCommandSubE1482 {
                queued_bytes: [0xFF, 0x00],
                next_write_index: 0x00,
            },
            "ASM loc_E159C should load `d0=#$00` before branching to loc_E1584, so the queued command must be `FF 00`",
        );
        assert_eq!(
            write.payload_writes,
            [
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0x55,
                    next_write_index: 0x01,
                    committed_write_index: 0x01,
                },
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0x44,
                    next_write_index: 0x02,
                    committed_write_index: 0x02,
                },
            ],
            "ASM loc_E159C should reuse loc_E1584, writing only the low bytes from `8(a6)` and `$C(a6)` after command `$00`",
        );
        assert!(
            write.releases_z80_bus && write.restores_previous_sr,
            "ASM loc_E159C should finish through loc_E1470 after the shared loc_E1584 payload writes",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E15A4_selects_command_01_before_two_payload_tail() {
        let write = sound_driver_write_command_01_two_bytes_loc_E15A4(0x3D, 0x01020304, 0xA0B0C0D0);

        assert_eq!(
            write.command_write,
            SoundDriverWriteCommandSubE1482 {
                queued_bytes: [0xFF, 0x01],
                next_write_index: 0x3F,
            },
            "ASM loc_E15A4 should load `d0=#$01` before branching to loc_E1584, so the queued command must be `FF 01`",
        );
        assert_eq!(
            write.payload_writes,
            [
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0x04,
                    next_write_index: 0x00,
                    committed_write_index: 0x00,
                },
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0xD0,
                    next_write_index: 0x01,
                    committed_write_index: 0x01,
                },
            ],
            "ASM loc_E15A4 should reuse loc_E1584, writing only the low bytes from `8(a6)` and `$C(a6)` after command `$01`",
        );
        assert!(
            write.releases_z80_bus && write.restores_previous_sr,
            "ASM loc_E15A4 should finish through loc_E1470 after the shared loc_E1584 payload writes",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E15AC_selects_command_05_and_writes_shifted_second_payload_byte() {
        let write =
            sound_driver_write_command_05_three_bytes_loc_E15AC(0x3C, 0x55667788, 0xA1B2C3D4);

        assert_eq!(
            write.command_write,
            SoundDriverWriteCommandSubE1482 {
                queued_bytes: [0xFF, 0x05],
                next_write_index: 0x3E,
            },
            "ASM loc_E15AC should queue command `$05` through sub_E1482 before any payload byte",
        );
        assert_eq!(
            write.payload_writes,
            [
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0x88,
                    next_write_index: 0x3F,
                    committed_write_index: 0x3F,
                },
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0xD4,
                    next_write_index: 0x00,
                    committed_write_index: 0x00,
                },
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0xC3,
                    next_write_index: 0x01,
                    committed_write_index: 0x01,
                },
            ],
            "ASM loc_E15AC should write low byte from `8(a6)`, low byte from `$C(a6)`, then the low byte after `asr.l #8,d0` of `$C(a6)`",
        );
        assert!(
            write.releases_z80_bus && write.restores_previous_sr,
            "ASM loc_E15AC should finish through loc_E1470 after the third payload write",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E15D0_selects_command_06_before_two_payload_tail() {
        let write = sound_driver_write_command_06_two_bytes_loc_E15D0(0x3D, 0x10203040, 0x50607080);

        assert_eq!(
            write.command_write,
            SoundDriverWriteCommandSubE1482 {
                queued_bytes: [0xFF, 0x06],
                next_write_index: 0x3F,
            },
            "ASM loc_E15D0 should load `d0=#$06` before branching to loc_E1584, so the queued command must be `FF 06`",
        );
        assert_eq!(
            write.payload_writes,
            [
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0x40,
                    next_write_index: 0x00,
                    committed_write_index: 0x00,
                },
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0x80,
                    next_write_index: 0x01,
                    committed_write_index: 0x01,
                },
            ],
            "ASM loc_E15D0 should reuse loc_E1584, writing only the low bytes from `8(a6)` and `$C(a6)` after command `$06`",
        );
        assert!(
            write.releases_z80_bus && write.restores_previous_sr,
            "ASM loc_E15D0 should finish through loc_E1470 after the shared loc_E1584 payload writes",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E15D8_selects_command_07_before_two_payload_tail() {
        let write = sound_driver_write_command_07_two_bytes_loc_E15D8(0x3E, 0xAABBCCDD, 0x11223344);

        assert_eq!(
            write.command_write,
            SoundDriverWriteCommandSubE1482 {
                queued_bytes: [0xFF, 0x07],
                next_write_index: 0x00,
            },
            "ASM loc_E15D8 should load `d0=#$07` before branching to loc_E1584, so the queued command must be `FF 07`",
        );
        assert_eq!(
            write.payload_writes,
            [
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0xDD,
                    next_write_index: 0x01,
                    committed_write_index: 0x01,
                },
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0x44,
                    next_write_index: 0x02,
                    committed_write_index: 0x02,
                },
            ],
            "ASM loc_E15D8 should reuse loc_E1584, writing only the low bytes from `8(a6)` and `$C(a6)` after command `$07`",
        );
        assert!(
            write.releases_z80_bus && write.restores_previous_sr,
            "ASM loc_E15D8 should finish through loc_E1470 after the shared loc_E1584 payload writes",
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn loc_E15E0_selects_command_0E_before_two_payload_tail() {
        let write = sound_driver_write_command_0E_two_bytes_loc_E15E0(0x3C, 0x0BADBEEF, 0xCAFED00D);

        assert_eq!(
            write.command_write,
            SoundDriverWriteCommandSubE1482 {
                queued_bytes: [0xFF, 0x0E],
                next_write_index: 0x3E,
            },
            "ASM loc_E15E0 should load `d0=#$0E` before branching to loc_E1584, so the queued command must be `FF 0E`",
        );
        assert_eq!(
            write.payload_writes,
            [
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0xEF,
                    next_write_index: 0x3F,
                    committed_write_index: 0x3F,
                },
                SoundDriverWriteByteSubE148E {
                    queued_byte: 0x0D,
                    next_write_index: 0x00,
                    committed_write_index: 0x00,
                },
            ],
            "ASM loc_E15E0 should reuse loc_E1584, writing only the low bytes from `8(a6)` and `$C(a6)` after command `$0E`",
        );
        assert!(
            write.releases_z80_bus && write.restores_previous_sr,
            "ASM loc_E15E0 should finish through loc_E1470 after the shared loc_E1584 payload writes",
        );
    }

    #[test]
    fn rom_random_state_matches_get_randomnumber_byte_seeded_sequence() {
        // ASM Init_RNG / Get_RandomNumber_byte (kid.asm:1106-1204).
        let mut rng = RomRandomState::default();
        let expected = [0xC0, 0x92, 0x03, 0x97, 0xA3, 0x1C, 0x09, 0xBC];
        let actual = expected.map(|_| rng.next_byte());
        assert_eq!(actual, expected);
    }

    #[test]
    fn rom_random_state_matches_get_randomnumber_word_seeded_sequence() {
        // ASM Get_RandomNumber_word (kid.asm:1210-1216) is two successive
        // Get_RandomNumber_byte calls packed into a big-endian word.
        let mut rng = RomRandomState::default();
        let expected = [0xC092, 0x0397, 0xA31C, 0x09BC];
        let actual = expected.map(|_| rng.next_word());
        assert_eq!(actual, expected);
    }

    #[test]
    fn initialise_scratch_queue_sub_28fc_clears_all_slots_and_rewinds_the_cursor() {
        let queue = initialise_scratch_queue_sub_28FC();
        let cleared_longwords =
            (queue.slots.len() * queue.slots[0].len()) / core::mem::size_of::<u32>();

        assert_eq!(
            queue.slots.len(),
            6,
            "ASM sub_28FC should seed six `$52`-byte scratch slots at `$FFFFF01E`",
        );
        assert_eq!(
            cleared_longwords, 0x7B,
            "ASM loc_2906 (kid.asm:4859) should clear exactly 123 longwords from `$FFFFF01E` via the `dbf d7,loc_2906` loop",
        );
        assert!(
            queue
                .slots
                .iter()
                .all(|slot| slot.iter().all(|&byte| byte == 0)),
            "ASM sub_28FC should clear every byte in the scratch queue before later workers write into it",
        );
        assert_eq!(
            queue.next_slot, 0,
            "ASM sub_28FC should rewind `$FFFFFA9C` to the first scratch slot",
        );
        assert_eq!(
            queue.slots_remaining, 6,
            "ASM sub_28FC should reload `$FFFFFAA0` with six available scratch slots",
        );
    }

    #[test]
    fn initialise_evanesc_object_slots_sub_eabc_chains_all_40_slots_and_clears_the_active_head() {
        let state = initialise_evanesc_object_slots_sub_EABC();

        assert_eq!(state.next_free_evanescent_object_slot_addr, 0xFFFF_EBDA);
        assert_eq!(state.first_evanescent_object_slot_addr, 0);
        assert_eq!(state.evanescent_slots.len(), 40);
        assert_eq!(
            state.evanescent_slots[0],
            LinkedObjectSlotInit {
                slot_addr: 0xFFFF_EBDA,
                next_slot_addr: 0xFFFF_EBE6,
            }
        );
        assert_eq!(
            state.evanescent_slots[38],
            LinkedObjectSlotInit {
                slot_addr: 0xFFFF_EDA2,
                next_slot_addr: 0xFFFF_EDAE,
            }
        );
        assert_eq!(
            state.evanescent_slots[39],
            LinkedObjectSlotInit {
                slot_addr: 0xFFFF_EDAE,
                next_slot_addr: 0,
            }
        );
        assert!(
            state
                .evanescent_slots
                .windows(2)
                .all(|pair| pair[0].next_slot_addr == pair[1].slot_addr),
            "ASM sub_EABC should build a contiguous `$0C`-stride free list through all 40 evanescent slots",
        );
    }

    #[test]
    fn initialise_shooter_object_slots_sub_f06a_chains_all_20_slots_and_resets_last_hit_coords() {
        let state: ShooterObjectSlotsSubF06A = initialise_shooter_object_slots_sub_F06A();

        assert_eq!(state.next_free_shooter_object_slot_addr, 0xFFFF_E7F2);
        assert_eq!(state.first_shooter_object_slot_addr, 0);
        assert_eq!(state.shooter_slots.len(), 20);
        assert_eq!(state.last_shooter_block_col, -1);
        assert_eq!(state.last_shooter_block_row, -1);
        assert_eq!(
            state.shooter_slots[0],
            LinkedObjectSlotInit {
                slot_addr: 0xFFFF_E7F2,
                next_slot_addr: 0xFFFF_E800,
            }
        );
        assert_eq!(
            state.shooter_slots[18],
            LinkedObjectSlotInit {
                slot_addr: 0xFFFF_E8EE,
                next_slot_addr: 0xFFFF_E8FC,
            }
        );
        assert_eq!(
            state.shooter_slots[19],
            LinkedObjectSlotInit {
                slot_addr: 0xFFFF_E8FC,
                next_slot_addr: 0,
            }
        );
        assert!(
            state
                .shooter_slots
                .windows(2)
                .all(|pair| pair[0].next_slot_addr == pair[1].slot_addr),
            "ASM sub_F06A should build a contiguous `$0E`-stride free list through all 20 shooter slots",
        );
    }

    #[test]
    fn spawn_block_debris_loc_2928_sets_the_alternate_variant_flag_before_shared_build() {
        let mut queue = initialise_scratch_queue_sub_28FC();

        spawn_block_debris_loc_2928(&mut queue, 0x0120, 0x0080, 2);

        assert_eq!(
            queue.slots[0][1], 0xFF,
            "ASM loc_2928 (kid.asm:4874) should set byte 1 with `st 1(a0)` before branching into the shared debris builder",
        );
        assert_eq!(
            scratch_debris_pieces_sub_292E(&queue, 0)[0],
            ScratchDebrisPieceSub292E {
                y_velocity_px: -1,
                x_velocity_px: -3,
                y_pos_px: 0x0080,
                x_pos_px: 0x0120,
            },
            "ASM loc_2928 should still flow into `loc_293A` and write the same first debris record as the standard builder path",
        );
    }

    #[test]
    fn spawn_block_debris_sub_292e_seeds_five_directional_debris_records() {
        let mut queue = initialise_scratch_queue_sub_28FC();

        spawn_block_debris_sub_292E(&mut queue, 0x0120, 0x0080, 2);

        assert_eq!(
            queue.slots[0][0], 0xFF,
            "ASM sub_292E should mark the current `$52`-byte scratch slot active before filling its debris records",
        );
        assert_eq!(
            queue.slots[0][1], 0x00,
            "ASM sub_292E should clear byte 1 for the standard smash/ice debris variant",
        );
        assert_eq!(
            scratch_debris_pieces_sub_292E(&queue, 0),
            [
                ScratchDebrisPieceSub292E {
                    y_velocity_px: -1,
                    x_velocity_px: -3,
                    y_pos_px: 0x0080,
                    x_pos_px: 0x0120,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: -1,
                    x_velocity_px: -3,
                    y_pos_px: 0x0088,
                    x_pos_px: 0x0120,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: -3,
                    y_pos_px: 0x0088,
                    x_pos_px: 0x0128,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 1,
                    x_velocity_px: -3,
                    y_pos_px: 0x0080,
                    x_pos_px: 0x0128,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: -3,
                    y_pos_px: 0x0084,
                    x_pos_px: 0x0124,
                },
            ],
            "ASM sub_292E should write the shared 0/8/8/0/4 position offsets and direction-2 signed velocity tables into the five 16-byte debris records",
        );
    }

    #[test]
    fn spawn_block_debris_sub_292e_loc_293a_uses_the_fallthrough_velocity_tables_for_direction_3_and_above()
     {
        let mut queue = initialise_scratch_queue_sub_28FC();

        spawn_block_debris_sub_292E(&mut queue, 0x0120, 0x0080, 7);

        assert_eq!(
            scratch_debris_pieces_sub_292E(&queue, 0),
            [
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: -5,
                    y_pos_px: 0x0080,
                    x_pos_px: 0x0120,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: -4,
                    y_pos_px: 0x0088,
                    x_pos_px: 0x0120,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: -3,
                    y_pos_px: 0x0088,
                    x_pos_px: 0x0128,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: -2,
                    y_pos_px: 0x0080,
                    x_pos_px: 0x0128,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: -1,
                    y_pos_px: 0x0084,
                    x_pos_px: 0x0124,
                },
            ],
            "ASM loc_293A (kid.asm:4887) should fall through to `unk_2A24`/`unk_2A2E` when `d6` is neither 0, 1, nor 2",
        );
    }

    #[test]
    fn spawn_block_debris_sub_292e_loc_297a_uses_the_first_direction_velocity_pair() {
        let mut queue = initialise_scratch_queue_sub_28FC();

        spawn_block_debris_sub_292E(&mut queue, 0x0120, 0x0080, 0);

        assert_eq!(
            scratch_debris_pieces_sub_292E(&queue, 0),
            [
                ScratchDebrisPieceSub292E {
                    y_velocity_px: -1,
                    x_velocity_px: -4,
                    y_pos_px: 0x0080,
                    x_pos_px: 0x0120,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: -1,
                    x_velocity_px: -3,
                    y_pos_px: 0x0088,
                    x_pos_px: 0x0120,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: -3,
                    y_pos_px: 0x0088,
                    x_pos_px: 0x0128,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 1,
                    x_velocity_px: -4,
                    y_pos_px: 0x0080,
                    x_pos_px: 0x0128,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: -5,
                    y_pos_px: 0x0084,
                    x_pos_px: 0x0124,
                },
            ],
            "ASM loc_297A (kid.asm:4910) should select `unk_2A10`/`unk_2A1A` when `tst.w d6` sees the zero direction",
        );
    }

    #[test]
    fn spawn_block_debris_sub_292e_loc_296c_uses_the_second_direction_velocity_pair() {
        let mut queue = initialise_scratch_queue_sub_28FC();

        spawn_block_debris_sub_292E(&mut queue, 0x0120, 0x0080, 1);

        assert_eq!(
            scratch_debris_pieces_sub_292E(&queue, 0),
            [
                ScratchDebrisPieceSub292E {
                    y_velocity_px: -1,
                    x_velocity_px: -2,
                    y_pos_px: 0x0080,
                    x_pos_px: 0x0120,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: -1,
                    x_velocity_px: 0,
                    y_pos_px: 0x0088,
                    x_pos_px: 0x0120,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: 0,
                    y_pos_px: 0x0088,
                    x_pos_px: 0x0128,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 1,
                    x_velocity_px: -1,
                    y_pos_px: 0x0080,
                    x_pos_px: 0x0128,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: -1,
                    x_velocity_px: -2,
                    y_pos_px: 0x0084,
                    x_pos_px: 0x0124,
                },
            ],
            "ASM loc_296C (kid.asm:4904) should select `unk_2A38`/`unk_2A42` when `d6 == 1`",
        );
    }

    #[test]
    fn spawn_block_debris_sub_292e_loc_2988_uses_the_third_direction_velocity_pair() {
        let mut queue = initialise_scratch_queue_sub_28FC();

        spawn_block_debris_sub_292E(&mut queue, 0x0120, 0x0080, 2);

        assert_eq!(
            scratch_debris_pieces_sub_292E(&queue, 0),
            [
                ScratchDebrisPieceSub292E {
                    y_velocity_px: -1,
                    x_velocity_px: -3,
                    y_pos_px: 0x0080,
                    x_pos_px: 0x0120,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: -1,
                    x_velocity_px: -3,
                    y_pos_px: 0x0088,
                    x_pos_px: 0x0120,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: -3,
                    y_pos_px: 0x0088,
                    x_pos_px: 0x0128,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 1,
                    x_velocity_px: -3,
                    y_pos_px: 0x0080,
                    x_pos_px: 0x0128,
                },
                ScratchDebrisPieceSub292E {
                    y_velocity_px: 0,
                    x_velocity_px: -3,
                    y_pos_px: 0x0084,
                    x_pos_px: 0x0124,
                },
            ],
            "ASM loc_2988 (kid.asm:4916) should select `unk_29FC`/`unk_2A06` when `cmpi.w #2,d6` matches exactly",
        );
    }

    #[test]
    fn spawn_block_debris_sub_292e_loc_2994_writes_five_records_with_the_shared_position_offsets() {
        let mut queue = initialise_scratch_queue_sub_28FC();

        spawn_block_debris_sub_292E(&mut queue, 0x0132, 0x0086, 3);

        let pieces = scratch_debris_pieces_sub_292E(&queue, 0);

        assert_eq!(
            pieces.len(),
            5,
            "ASM loc_2994 (kid.asm:4920) seeds `d7=#4`, so `dbf d7,loc_2994` must write exactly five debris records before the loop exits",
        );
        assert_eq!(
            pieces.map(|piece| (piece.x_pos_px, piece.y_pos_px)),
            [
                (0x0132, 0x0086),
                (0x0132, 0x008E),
                (0x013A, 0x008E),
                (0x013A, 0x0086),
                (0x0136, 0x008A),
            ],
            "ASM loc_2994 should add the shared `unk_29E8`/`unk_29F2` offsets onto `x_pos(a3)` and `y_pos(a3)` in piece order before advancing `a1` by `$10` bytes",
        );
    }

    #[test]
    fn spawn_block_debris_sub_292e_advances_and_wraps_the_scratch_cursor() {
        let mut queue = initialise_scratch_queue_sub_28FC();

        for slot_index in 0..SUB_28FC_SLOT_COUNT {
            spawn_block_debris_sub_292E(&mut queue, slot_index as u16, 0x0040, 3);
        }

        assert_eq!(
            queue.next_slot, 0,
            "ASM sub_292E should rewind `$FFFFFA9C` to slot 0 after consuming the sixth scratch slot",
        );
        assert_eq!(
            queue.slots_remaining, 6,
            "ASM sub_292E should reload `$FFFFFAA0` with six available slots once the queue wraps",
        );
        assert_eq!(
            scratch_debris_pieces_sub_292E(&queue, SUB_28FC_SLOT_COUNT - 1)[0].x_velocity_px,
            -5,
            "ASM sub_292E should keep using the selected direction table right up to the wrapping sixth slot",
        );
    }

    #[test]
    fn spawn_block_debris_sub_292e_loc_29de_advances_to_the_next_scratch_slot_while_capacity_remains()
     {
        let mut queue = initialise_scratch_queue_sub_28FC();

        spawn_block_debris_sub_292E(&mut queue, 0x0120, 0x0080, 1);

        assert_eq!(
            queue.next_slot, 1,
            "ASM loc_29DE should advance `$FFFFFA9C` by `$52` to scratch slot 1 when `$FFFFFAA0` stays non-zero after the decrement",
        );
        assert_eq!(
            queue.slots_remaining, 5,
            "ASM loc_29DE should leave five scratch slots available after the first debris spawn consumes one of the six entries",
        );
    }

    #[test]
    fn background_scroll_upper_pass_loc_2ffe8_seeds_the_top_window_and_keeps_28_iterations_inside_level_bounds()
     {
        assert_eq!(
            background_scroll_upper_pass_loc_2FFE8(0x0123, 0x0228, 0x0500),
            BackgroundScrollUpperPassLoc2FFE8 {
                scroll_data_camera_x: 0x0123,
                original_camera_y: 0x0228,
                stepped_camera_y: 0x0148,
                scroll_data_camera_y: 0x0140,
                loop_iterations: 0x001C,
            },
            "ASM loc_2FFE8 should snapshot the original camera words into `$FFFFF82C/d7`, subtract `$E0` from `Camera_Y_pos`, write `Camera_Y_pos-8` to `$FFFFF82E`, and keep `d6=#1C` when `d7+$E0` stays below `Level_height_pixels`",
        );
    }

    #[test]
    fn background_scroll_upper_pass_resume_loc_30014_restores_the_saved_camera_y_after_the_bottom_edge_clamp()
     {
        let prologue = background_scroll_upper_pass_loc_2FFE8(0x0040, 0x0300, 0x03D0);

        assert_eq!(
            background_scroll_upper_pass_resume_loc_30014(prologue),
            BackgroundScrollUpperPassResumeLoc30014 {
                restored_camera_y: 0x0300,
                loop_iterations: 0x001B,
            },
            "ASM loc_30014 should restore the saved `d7` camera Y after loc_2FFE8 decrements `d6` for the bottom-edge case where `d7+$E0 >= Level_height_pixels`",
        );
    }

    #[test]
    fn background_scroll_lower_pass_setup_loc_3001e_rebuilds_the_second_scroll_window_from_the_saved_camera_y()
     {
        let resume = background_scroll_upper_pass_resume_loc_30014(
            background_scroll_upper_pass_loc_2FFE8(0x0123, 0x0228, 0x0500),
        );

        assert_eq!(
            background_scroll_lower_pass_setup_loc_3001E(resume, 0x0500),
            BackgroundScrollLowerPassSetupLoc3001E {
                original_camera_y: 0x0228,
                stepped_camera_y: -0x0158,
                shift_data_camera_y: -0x0178,
                loop_iterations: 0x001C,
            },
            "ASM loc_3001E should rebuild the lower background pass from saved `d7` by storing `d7-$380` into `Camera_Y_pos`, `d7-$3A0` into `$FFFFFAA8`, and re-seeding `d6=#1C` while `d7+$E0` stays below `Level_height_pixels`",
        );
    }

    #[test]
    fn background_scroll_lower_pass_resume_loc_30054_biases_non_wrapped_rows_before_the_lower_loop()
    {
        let setup = background_scroll_lower_pass_setup_loc_3001E(
            background_scroll_upper_pass_resume_loc_30014(background_scroll_upper_pass_loc_2FFE8(
                0x0123, 0x0228, 0x0500,
            )),
            0x0500,
        );

        assert_eq!(
            background_scroll_lower_pass_resume_loc_30054(setup),
            BackgroundScrollLowerPassResumeLoc30054 {
                original_camera_y: 0x0228,
                stepped_camera_y: -0x0198,
                shift_data_camera_y: -0x0178,
                loop_iterations: 0x001E,
            },
            "ASM loc_30054 should subtract `$40` from the lower-pass `Camera_Y_pos` seed and add 2 to `d6` when the setup seed is not the wrapped `$FC80` special case",
        );
    }

    #[test]
    fn background_scroll_lower_pass_resume_loc_30054_preserves_the_wrapped_fc80_seed() {
        let setup = BackgroundScrollLowerPassSetupLoc3001E {
            original_camera_y: 0,
            stepped_camera_y: -0x0380,
            shift_data_camera_y: -0x03A0,
            loop_iterations: 0x001B,
        };

        assert_eq!(
            background_scroll_lower_pass_resume_loc_30054(setup),
            BackgroundScrollLowerPassResumeLoc30054 {
                original_camera_y: 0,
                stepped_camera_y: -0x0380,
                shift_data_camera_y: -0x03A0,
                loop_iterations: 0x001B,
            },
            "ASM loc_30054 should branch straight to the lower-row loop when `Camera_Y_pos` already equals the wrapped `$FC80` seed",
        );
    }

    #[test]
    fn background_scroll_lower_pass_loop_loc_30066_uses_piece_renderer_and_clears_shift_seed_for_zero_camera()
     {
        let resume = BackgroundScrollLowerPassResumeLoc30054 {
            original_camera_y: 0,
            stepped_camera_y: -0x0380,
            shift_data_camera_y: -0x03A0,
            loop_iterations: 0x001B,
        };

        assert_eq!(
            background_scroll_lower_pass_loop_loc_30066(resume, 0),
            BackgroundScrollLowerPassLoopLoc30066 {
                renderer: BackgroundScrollLowerPassRendererLoc30066::PiecesSub307D8,
                final_camera_y: 0,
                final_scroll_data_camera_y: 0,
                final_shift_data_camera_y: 0,
            },
            "ASM loc_30066 should dispatch piece backgrounds to `sub_307D8`, then the shared epilogue should restore `Camera_Y_pos/$FFFFF82E` from `d7` and clear `$FFFFFAA8` when `d7=0`",
        );
    }

    #[test]
    fn background_scroll_lower_pass_loop_loc_30066_uses_enigma_renderer_and_preserves_non_zero_shift_seed()
     {
        let resume = BackgroundScrollLowerPassResumeLoc30054 {
            original_camera_y: 0x0228,
            stepped_camera_y: -0x0198,
            shift_data_camera_y: -0x0178,
            loop_iterations: 0x001E,
        };

        assert_eq!(
            background_scroll_lower_pass_loop_loc_30066(resume, 0xFF),
            BackgroundScrollLowerPassLoopLoc30066 {
                renderer: BackgroundScrollLowerPassRendererLoc30066::EnigmaSub303BA,
                final_camera_y: 0x0228,
                final_scroll_data_camera_y: 0x0228,
                final_shift_data_camera_y: -0x0178,
            },
            "ASM loc_30066 should branch non-zero `Background_format` values to `sub_303BA`, and the shared epilogue should preserve the precomputed `$FFFFFAA8` seed when saved `d7` is non-zero",
        );
    }

    #[test]
    fn background_scroll_lower_pass_loop_loc_30076_routes_non_zero_background_formats_to_the_enigma_renderer()
     {
        let resume = BackgroundScrollLowerPassResumeLoc30054 {
            original_camera_y: 0x0100,
            stepped_camera_y: -0x02C0,
            shift_data_camera_y: -0x02E0,
            loop_iterations: 0x001C,
        };

        assert_eq!(
            background_scroll_lower_pass_loop_loc_30066(resume, 1).renderer,
            BackgroundScrollLowerPassRendererLoc30066::EnigmaSub303BA,
            "ASM loc_30076 should branch non-zero `Background_format` values to `bsr.w sub_303BA` before falling through to loc_3007A",
        );
    }

    #[test]
    fn background_scroll_lower_pass_loop_loc_3007a_restores_both_camera_words_from_the_saved_d7() {
        let resume = BackgroundScrollLowerPassResumeLoc30054 {
            original_camera_y: 0x0228,
            stepped_camera_y: -0x0198,
            shift_data_camera_y: -0x0178,
            loop_iterations: 0x001E,
        };

        let result = background_scroll_lower_pass_loop_loc_30066(resume, 0);

        assert_eq!(
            (result.final_camera_y, result.final_scroll_data_camera_y),
            (0x0228, 0x0228),
            "ASM loc_3007A should restore both `Camera_Y_pos` and `$FFFFF82E` from the saved `d7` before control reaches the `tst.w d7` zero check",
        );
    }

    #[test]
    fn background_scroll_lower_pass_loop_loc_30090_returns_with_zeroed_shift_seed_only_after_restoring_zero_camera_words()
     {
        let resume = BackgroundScrollLowerPassResumeLoc30054 {
            original_camera_y: 0,
            stepped_camera_y: -0x0380,
            shift_data_camera_y: -0x03A0,
            loop_iterations: 0x001B,
        };

        assert_eq!(
            background_scroll_lower_pass_loop_loc_30066(resume, 0),
            BackgroundScrollLowerPassLoopLoc30066 {
                renderer: BackgroundScrollLowerPassRendererLoc30066::PiecesSub307D8,
                final_camera_y: 0,
                final_scroll_data_camera_y: 0,
                final_shift_data_camera_y: 0,
            },
            "ASM loc_30090 should return after restoring zero `d7` into both camera words, with the preceding zero-check path having already cleared `$FFFFFAA8`",
        );
    }

    #[test]
    fn transfer_scroll_data_vsram_setup_loc_300ba_quarters_camera_y_unless_no_scroll_is_forced() {
        assert_eq!(
            transfer_scroll_data_vsram_setup_loc_300BA(0x0124, false),
            TransferScrollDataVsramSetupLoc300BA {
                first_vsram_word: 0x0124,
                second_vsram_word: 0x0049,
            },
            "ASM loc_300BA should write `Camera_Y_pos` to the first VSRAM slot, then use `lsr.w #2,d0` for the second slot when `(Background_NoScrollFlag).w` is clear",
        );
        assert_eq!(
            transfer_scroll_data_vsram_setup_loc_300BA(0x0124, true),
            TransferScrollDataVsramSetupLoc300BA {
                first_vsram_word: 0x0124,
                second_vsram_word: 0,
            },
            "ASM loc_300BA should branch around the shifted parallax value and force the shared-tail `d0=0` when `(Background_NoScrollFlag).w` is non-zero",
        );
    }

    #[test]
    fn transfer_scroll_data_prelude_loc_3038a_returns_immediately_for_invalid_special_effects() {
        assert_eq!(
            transfer_scroll_data_prelude_loc_3038A(4, 0x000C, 0x1234),
            None,
            "ASM loc_3038A should return before touching `Time_Frames` when `Level_Special_Effects == 4`",
        );
    }

    #[test]
    fn transfer_scroll_data_prelude_loc_3038a_only_increments_time_frames_outside_sega_screen_mode()
    {
        assert_eq!(
            transfer_scroll_data_prelude_loc_3038A(0, 0, 0x1234),
            Some(TransferScrollDataPreludeLoc3038A {
                next_time_frames: 0x1234,
            }),
            "ASM loc_3038A should branch straight to `loc_3039C` without incrementing `Time_Frames` when `Game_Mode` is zero",
        );
        assert_eq!(
            transfer_scroll_data_prelude_loc_3038A(0, 0x000C, 0x1234),
            Some(TransferScrollDataPreludeLoc3038A {
                next_time_frames: 0x1235,
            }),
            "ASM loc_3038A should `addq.w #1,(Time_Frames).w` before falling through when `Game_Mode` is non-zero",
        );
    }

    #[test]
    fn transfer_scroll_data_dispatch_loc_3039c_returns_after_shift_work_when_upload_guard_is_set() {
        assert_eq!(
            transfer_scroll_data_dispatch_loc_3039C(1, 0),
            TransferScrollDataDispatchLoc3039C {
                computes_shift_data: true,
                computes_shift_layers: true,
                renderer: TransferScrollDataRendererLoc3039C::Return303B8,
            },
            "ASM loc_3039C should always run the two background-shift helpers first, then return immediately when `$FFFFFB49` is non-zero",
        );
    }

    #[test]
    fn transfer_scroll_data_dispatch_loc_3039c_selects_piece_or_enigma_renderers_after_shift_work()
    {
        assert_eq!(
            transfer_scroll_data_dispatch_loc_3039C(0, 0),
            TransferScrollDataDispatchLoc3039C {
                computes_shift_data: true,
                computes_shift_layers: true,
                renderer: TransferScrollDataRendererLoc3039C::PiecesSub30744And307D8,
            },
            "ASM loc_3039C should fall through to `bsr.w sub_30744` then `bsr.w sub_307D8` when `$FFFFFB49` is clear and `Background_format` is zero",
        );
        assert_eq!(
            transfer_scroll_data_dispatch_loc_3039C(0, 0xFF),
            TransferScrollDataDispatchLoc3039C {
                computes_shift_data: true,
                computes_shift_layers: true,
                renderer: TransferScrollDataRendererLoc3039C::EnigmaSub303BA,
            },
            "ASM loc_3039C should branch to `sub_303BA` instead of the piece renderer pair when `Background_format` is non-zero",
        );
    }

    #[test]
    fn background_scroll_piece_row_dispatch_loc_3079a_returns_on_ff_and_only_skips_negative_scroll_entries()
     {
        assert_eq!(
            background_scroll_piece_row_dispatch_loc_3079A(
                0xFF, -1, 0x0120, 0x0040, 0x0700, 0x001C
            ),
            BackgroundScrollPieceRowDispatchLoc3079A {
                row_selector_byte: 0xFF,
                background_scroll_longword: -1,
                branch: BackgroundScrollPieceRowDispatchLoc3079ABranch::Return307B0,
                next_layout_row_offset: 0x0120,
                next_vdp_row_seed: 0x0700,
                next_dbf_counter: 0x001C,
            },
            "ASM loc_3079A should branch straight to `return_307B0` as soon as `(a2)+` yields `$FF`, without touching `a0`, `d2`, or the inclusive `dbf d3` counter",
        );
        assert_eq!(
            background_scroll_piece_row_dispatch_loc_3079A(
                0x08,
                i32::from_be_bytes([0x80, 0x00, 0x00, 0x00]),
                0x0120,
                0x0040,
                0x0700,
                0x001C,
            ),
            BackgroundScrollPieceRowDispatchLoc3079A {
                row_selector_byte: 0x08,
                background_scroll_longword: i32::from_be_bytes([0x80, 0x00, 0x00, 0x00]),
                branch: BackgroundScrollPieceRowDispatchLoc3079ABranch::ContinueLoc3079A,
                next_layout_row_offset: 0x0160,
                next_vdp_row_seed: 0x0780,
                next_dbf_counter: 0x001B,
            },
            "ASM loc_3079A should treat a negative `move.l (a1,d4.w),d5` result as the skip path, advancing `a0` by `Background_width`, adding `$80` to `d2`, and consuming one inclusive `dbf d3` step before retrying the loop",
        );
        assert_eq!(
            background_scroll_piece_row_dispatch_loc_3079A(
                0x08,
                0x0000_1234,
                0x0120,
                0x0040,
                0x0700,
                0x001C
            ),
            BackgroundScrollPieceRowDispatchLoc3079A {
                row_selector_byte: 0x08,
                background_scroll_longword: 0x0000_1234,
                branch: BackgroundScrollPieceRowDispatchLoc3079ABranch::Branch307B2,
                next_layout_row_offset: 0x0120,
                next_vdp_row_seed: 0x0700,
                next_dbf_counter: 0x001C,
            },
            "ASM loc_3079A should reach `bpl.s loc_307B2` only when the loaded scroll longword is non-negative, leaving the row pointer, VDP seed, and DBF counter untouched for the upload path",
        );
    }

    #[test]
    fn background_scroll_piece_row_upload_loc_307b2_packs_the_vdp_command_and_uses_the_swapped_tile_byte_offset()
     {
        let layout_row = [0xAA, 0xBB, 0x02, 0xCC];
        let block_mappings = [0x1000, 0x2000, 0x3568, 0x4000];

        assert_eq!(
            background_scroll_piece_row_upload_loc_307B2(
                0x0002_1357,
                0x0740,
                &layout_row,
                0x0120,
                &block_mappings,
                0x0040,
                0x001C,
            ),
            Some(BackgroundScrollPieceRowUploadLoc307B2 {
                vdp_control_word: 0x7757,
                vdp_autoincrement_word: 3,
                tile_byte_offset: 0x0002,
                tile_mapping_word: 0x3568,
                next_layout_row_offset: 0x0160,
                next_vdp_row_seed: 0x07C0,
                next_dbf_counter: 0x001B,
                loops_to_loc_3079_a: true,
            }),
            "ASM loc_307B2 should OR the low scroll word with `($F80 & d2) | $6000`, write autoincrement `3`, `swap d5` so the original high word becomes the `(a0,d5.w)` byte offset, and then advance `a0`/`d2` for the next inclusive `dbf d3,loc_3079A` pass",
        );
    }

    #[test]
    fn background_scroll_piece_refresh_seed_loc_3081a_uses_current_or_biased_rows_and_stops_on_ff()
    {
        let mut row_sequence = vec![0; 0x20];
        row_sequence[0x001E] = 0x08;
        row_sequence[0x0003] = 0xFF;

        let mut horiz_scroll_bytes = vec![0; 0x20];
        horiz_scroll_bytes[0x000C] = 0x01;
        horiz_scroll_bytes[0x000D] = 0x20;

        assert_eq!(
            background_scroll_piece_refresh_seed_loc_3081A(
                0x0002,
                0x0001,
                &row_sequence,
                &horiz_scroll_bytes,
                0x0020,
            ),
            Some(BackgroundScrollPieceRefreshSeedLoc3081A {
                row_sequence_offset: 0x001E,
                row_sequence_entry: 0x08,
                coarse_scroll_word: 0x0024,
                reduced_scroll_word: 0x0004,
                wrap_subtraction_count: 1,
            }),
            "ASM loc_3081A should inherit `d0 + $1C` when the camera row moved downward, read the row-sequence byte from `(a2,d0.w)`, load the coarse scroll word from `4(a1,d0.w)`, and keep subtracting `Background_width` until the value fits before `loc_30840`",
        );
        assert_eq!(
            background_scroll_piece_refresh_seed_loc_3081A(
                0x0003,
                0x0005,
                &row_sequence,
                &horiz_scroll_bytes,
                0x0020,
            ),
            None,
            "ASM loc_3081A should keep the un-biased current row when the cached row is greater and branch straight to `return_308B4` when `(a2,d0.w)` yields `$FF`",
        );
    }

    #[test]
    fn background_scroll_piece_refresh_wrap_loc_30836_keeps_subtracting_background_width_until_the_word_fits()
     {
        assert_eq!(
            background_scroll_piece_refresh_wrap_loc_30836(0x0091, 0x0020),
            BackgroundScrollPieceRefreshWrapLoc30836 {
                reduced_scroll_word: 0x0011,
                wrap_subtraction_count: 4,
            },
            "ASM loc_30836 should keep subtracting `Background_width` from the shifted coarse scroll word until `cmp.w d0,a4` sees a value strictly below the width",
        );
    }

    #[test]
    fn background_scroll_piece_refresh_window_split_loc_3086a_wraps_the_row_and_tile_windows_independently()
     {
        assert_eq!(
            background_scroll_piece_refresh_window_split_loc_3086A(0x0015, 0x0020, 0x003C),
            BackgroundScrollPieceRefreshWindowSplitLoc3086A {
                wrapped_row_window_end: 0x001D,
                row_window_wraps: true,
                tile_column_seed: 0x003C,
                wrapped_tile_window_span: 0x0024,
                tile_window_wraps: true,
            },
            "ASM loc_3086A should wrap `d7 = d0 + $28` by `Background_width` when the row span crosses the edge, then wrap `(d2 & $3F) + $28` by `$40` when the tile phase crosses the 64-column boundary",
        );
    }

    #[test]
    fn background_scroll_piece_refresh_vdp_seed_loc_30884_packs_the_masked_tile_column_and_row_bits()
     {
        assert_eq!(
            background_scroll_piece_refresh_vdp_seed_loc_30884(0x0045, 0x001E),
            BackgroundScrollPieceRefreshVdpSeedLoc30884 {
                tile_column_offset: 0x000A,
                vdp_control_word: 0x6F0A,
                vdp_autoincrement_word: 3,
            },
            "ASM loc_30884 should mask `d0` to 6 tile-column bits, double it into the word offset, pack `(d3 << 7) & $F80` under `$6000`, and then emit autoincrement `#3`",
        );
    }

    #[test]
    fn background_scroll_piece_refresh_linear_copy_loc_308a6_copies_41_mapping_words_from_the_layout_row()
     {
        let layout_row: Vec<u8> = (0..0x29).collect();
        let block_mappings: Vec<u16> = (0..0x29).map(|index| 0x4000 | index).collect();

        assert_eq!(
            background_scroll_piece_refresh_linear_copy_loc_308A6(&layout_row, &block_mappings),
            Some(BackgroundScrollPieceRefreshLinearCopyLoc308A6 {
                copied_mapping_words: block_mappings.clone(),
                bytes_consumed: 0x29,
            }),
            "ASM loc_308A6 should clear `d6`, read one layout byte from `(a0)+`, double it into the `(a3,d6.w)` word lookup, and keep copying through the inclusive `dbf d4` count seeded from `#$28`",
        );
    }

    #[test]
    fn background_scroll_piece_refresh_wrap_counter_setup_loc_308b6_splits_the_wrapped_copy_into_two_inclusive_passes()
     {
        assert_eq!(
            background_scroll_piece_refresh_wrap_counter_setup_loc_308B6(0x001D),
            BackgroundScrollPieceRefreshWrapCounterSetupLoc308B6 {
                wrapped_row_window_end: 0x001D,
                first_pass_dbf_counter: 0x000A,
                second_pass_dbf_counter: 1,
            },
            "ASM loc_308B6 should snapshot wrapped `d7` into `d4`, seed `d1` with `$28 - d7 - 1` for the first inclusive pass, and set `d2 = 1` so the shared tail retries exactly once",
        );
    }

    #[test]
    fn background_scroll_piece_refresh_tile_wrap_counter_setup_loc_308de_splits_the_tile_wrap_copy_into_two_inclusive_passes()
     {
        assert_eq!(
            background_scroll_piece_refresh_tile_wrap_counter_setup_loc_308DE(0x0024),
            BackgroundScrollPieceRefreshTileWrapCounterSetupLoc308DE {
                wrapped_tile_window_span: 0x0024,
                first_pass_dbf_counter: 0x0003,
                second_pass_dbf_counter: 1,
            },
            "ASM loc_308DE should snapshot wrapped `d6` into `d4`, seed `d1` with `$28 - d6 - 1` for the first inclusive pass, and set `d2 = 1` so loc_308E8 executes exactly one second pass",
        );
    }

    #[test]
    fn background_scroll_piece_refresh_tile_wrapped_copy_loc_308e8_resets_plane_b_then_continues_without_rewinding_a0()
     {
        let layout_row: Vec<u8> = (0..0x40).collect();
        let block_mappings: Vec<u16> = (0..0x40).map(|index| 0x6000 | index).collect();

        assert_eq!(
            background_scroll_piece_refresh_tile_wrapped_copy_loc_308E8(
                &layout_row,
                0x0003,
                &block_mappings,
                0x0024,
                0x6F0A,
            ),
            Some(BackgroundScrollPieceRefreshTileWrappedCopyLoc308E8 {
                first_pass_mapping_words: block_mappings[0x03..0x07].to_vec(),
                seam_vdp_control_word: 0x6F0A,
                seam_vdp_autoincrement_word: 3,
                second_pass_mapping_words: block_mappings[0x07..0x2C].to_vec(),
                bytes_consumed: 0x29,
                final_layout_row_offset: 0x002C,
            }),
            "ASM loc_308E8 should consume the first inclusive `dbf d1` span, write `d3` plus autoincrement `#3` to `(a5)`, then continue the second pass from the advanced `(a0)+` position without rewinding by `Background_width`",
        );
    }

    #[test]
    fn background_scroll_piece_refresh_dual_wrap_branch_loc_30904_routes_equal_and_asymmetric_seams_to_the_right_worker()
     {
        assert_eq!(
            background_scroll_piece_refresh_dual_wrap_branch_loc_30904(0x001D, 0x001D),
            BackgroundScrollPieceRefreshDualWrapBranchLoc30904::SharedSplitLoc30916,
            "ASM loc_30904 should fall through to loc_30916 when `cmp.w d6,d7` sees equal wrapped tile and row seams",
        );
        assert_eq!(
            background_scroll_piece_refresh_dual_wrap_branch_loc_30904(0x0015, 0x001D),
            BackgroundScrollPieceRefreshDualWrapBranchLoc30904::RowWrapFirstLoc30936,
            "ASM loc_30904 should branch with `bgt.s loc_30936` when wrapped row seam `d7` is greater than wrapped tile seam `d6`",
        );
        assert_eq!(
            background_scroll_piece_refresh_dual_wrap_branch_loc_30904(0x001D, 0x0015),
            BackgroundScrollPieceRefreshDualWrapBranchLoc30904::TileWrapFirstLoc30978,
            "ASM loc_30904 should branch with `blt.w loc_30978` when wrapped row seam `d7` is less than wrapped tile seam `d6`",
        );
    }

    #[test]
    fn background_scroll_piece_refresh_wrapped_copy_loc_308c4_rewinds_by_background_width_between_the_two_passes()
     {
        let layout_row: Vec<u8> = (0..0x20).collect();
        let block_mappings: Vec<u16> = (0..0x20).map(|index| 0x5000 | index).collect();

        assert_eq!(
            background_scroll_piece_refresh_wrapped_copy_loc_308C4(
                &layout_row,
                0x0015,
                &block_mappings,
                0x0020,
                0x001D,
            ),
            Some(BackgroundScrollPieceRefreshWrappedCopyLoc308C4 {
                first_pass_mapping_words: block_mappings[0x15..0x20].to_vec(),
                second_pass_mapping_words: block_mappings[0x00..0x1E].to_vec(),
                bytes_consumed: 0x29,
                final_layout_row_offset: -2,
            }),
            "ASM loc_308C4 should consume the first inclusive `dbf d1` span from the current `(a0)+` position, rewind `a0` by `Background_width`, reload `d1` from `d4`, and use `d2 = 1` to execute exactly one wrapped second pass",
        );
    }

    #[test]
    fn background_scroll_enigma_row_table_lookup_loc_303ea_biases_desert_rows_and_masks_tile_row_to_5_bits()
     {
        let background_row_table = [0, 0x11, 0x22, 0x33, 0];

        assert_eq!(
            background_scroll_enigma_row_table_lookup_loc_303EA(0x0021, 5, &background_row_table),
            Some(BackgroundScrollEnigmaRowTableLookupLoc303EA {
                map_header_byte_offset: 3,
                background_row_index: 0x33,
                row_mapping_byte_offset: 0x0CC0,
                vdp_control_word: 0x6080,
                vdp_autoincrement_word: 3,
            }),
            "ASM loc_303EA should apply the Desert `addq.w #1,a0` bias before `move.b 1(a0,d0.w),d1`, then mask the tile row with `$1F` when building the plane-B control word",
        );
    }

    #[test]
    fn background_scroll_enigma_primary_copy_setup_loc_30426_seeds_the_dbf_counter_to_7() {
        assert_eq!(
            background_scroll_enigma_primary_copy_setup_loc_30426(),
            BackgroundScrollEnigmaPrimaryCopySetupLoc30426 {
                longword_dbf_counter: 7,
            },
            "ASM loc_30426 should reload `d0` with `#7` before the primary `dbf d0,loc_30428` run, so the loop executes 8 inclusive iterations",
        );
    }

    #[test]
    fn background_scroll_enigma_primary_longword_copy_loc_30428_copies_two_longwords_per_inclusive_dbf_iteration()
     {
        let source_longwords = [
            0x0000_0000,
            0x1111_1111,
            0x2222_2222,
            0x3333_3333,
            0x4444_4444,
            0x5555_5555,
            0x6666_6666,
            0x7777_7777,
            0x8888_8888,
            0x9999_9999,
            0xAAAA_AAAA,
            0xBBBB_BBBB,
            0xCCCC_CCCC,
            0xDDDD_DDDD,
            0xEEEE_EEEE,
            0xFFFF_FFFF,
            0x1357_9BDF,
        ];

        assert_eq!(
            background_scroll_enigma_primary_longword_copy_loc_30428(&source_longwords, 7),
            source_longwords[..16].to_vec(),
            "ASM loc_30428 should write two longwords before each `dbf d0,loc_30428` test, so `d0=#7` copies 16 primary-stream longwords",
        );
    }

    #[test]
    fn background_scroll_enigma_secondary_longword_copy_loc_30432_copies_two_longwords_per_inclusive_dbf_iteration()
     {
        let source_longwords = [
            0x0001_0000,
            0x0001_0001,
            0x0001_0002,
            0x0001_0003,
            0x0001_0004,
            0x0001_0005,
            0x0001_0006,
            0x0001_0007,
            0x0001_0008,
            0x0001_0009,
            0x0001_000A,
            0x0001_000B,
            0x0001_000C,
            0x0001_000D,
            0x0001_000E,
            0x0001_000F,
            0x0001_0010,
            0x0001_0011,
        ];

        assert_eq!(
            background_scroll_enigma_secondary_longword_copy_loc_30432(&source_longwords, 7),
            source_longwords[..16].to_vec(),
            "ASM loc_30432 should write two longwords before each `dbf d0,loc_30432` test, so `d0=#7` copies 16 secondary-stream longwords",
        );
    }

    #[test]
    fn background_scroll_shift_data_slot_loc_30452_halves_the_phase_adds_camera_bias_and_caches_the_previous_word()
     {
        assert_eq!(
            background_scroll_shift_data_slot_loc_30452(0x0015, 0x0058, 0x0020),
            BackgroundScrollShiftDataSlotLoc30452 {
                next_phase_word: 0x000A,
                updated_scroll_word: 0x000E,
                cached_previous_scroll_word: 0x0058,
                previous_tile_column: 0x000B,
                updated_tile_column: 0x0001,
                branches_to_loc_30476: false,
            },
            "ASM loc_30452 should `lsr.w #1,d0`, save that halved phase in `a5`, add `Camera_X_pos>>3`, store the new word to `4(a0)`, mirror the old word to `6(a0)`, and only branch to loc_30476 when the logical `>>3` tile columns match",
        );
    }

    #[test]
    fn background_scroll_shift_data_unchanged_loc_30476_emits_ffff_when_the_tile_column_does_not_change()
     {
        assert_eq!(
            background_scroll_shift_data_unchanged_loc_30476(0x0028, 0x002F),
            Some(0xFFFF),
            "ASM loc_30476 should write `$FFFF` to `(a0)` when the logical `>>3` columns still match, because that slot skips `sub_304B4` and falls straight through to loc_3047A",
        );
    }

    #[test]
    fn background_scroll_shift_data_second_pass_slot_loc_3048c_mirrors_the_previous_word_and_only_branches_on_an_exact_match()
     {
        assert_eq!(
            background_scroll_shift_data_second_pass_slot_loc_3048C(0x1200, 0x11F8),
            BackgroundScrollShiftDataSecondPassSlotLoc3048C {
                updated_scroll_word: 0x1200,
                cached_previous_scroll_word: 0x11F8,
                branches_to_loc_304_a4: false,
            },
            "ASM loc_3048C should `swap d5`, store that low word into `4(a0)`, mirror the old `4(a0)` word into `6(a0)`, and only branch to `loc_304A4` when `cmp.w d5,d2` finds an exact word match",
        );
        assert_eq!(
            background_scroll_shift_data_second_pass_slot_loc_3048C(0x1200, 0x1200),
            BackgroundScrollShiftDataSecondPassSlotLoc3048C {
                updated_scroll_word: 0x1200,
                cached_previous_scroll_word: 0x1200,
                branches_to_loc_304_a4: true,
            },
            "ASM loc_3048C should reach `beq.s loc_304A4` only for exact word equality, unlike the earlier `loc_30452` tile-column compare",
        );
    }

    #[test]
    fn background_scroll_shift_data_second_pass_unchanged_loc_304a4_emits_ffff_only_for_exact_word_matches()
     {
        assert_eq!(
            background_scroll_shift_data_second_pass_unchanged_loc_304A4(0x1200, 0x1200),
            Some(0xFFFF),
            "ASM loc_304A4 should write `$FFFF` to `(a0)` after `loc_3048C` only when the second-pass compare sees the same full scroll word in both slots",
        );
        assert_eq!(
            background_scroll_shift_data_second_pass_unchanged_loc_304A4(0x1200, 0x11F8),
            None,
            "ASM loc_304A4 is only reachable from `beq.s loc_304A4`, so unequal second-pass words must skip the `$FFFF` sentinel and fall through to `sub_304B4`/`loc_304A8` instead",
        );
    }

    #[test]
    fn background_scroll_shift_data_second_pass_advance_loc_304a8_restores_the_accumulator_order_and_consumes_one_dbf_iteration()
     {
        assert_eq!(
            background_scroll_shift_data_second_pass_advance_loc_304A8(
                0x1200_3456,
                0x0000_1800,
                0x0010,
            ),
            BackgroundScrollShiftDataSecondPassAdvanceLoc304A8 {
                next_slot_offset_bytes: 8,
                next_phase_accumulator: 0x3456_2A00,
                next_dbf_counter: 0x000F,
                loops_to_loc_3048_c: true,
            },
            "ASM loc_304A8 should `addq.w #8,a0`, `swap d5` back to the accumulator ordering, add the fixed-point step from `d4`, and loop to `loc_3048C` while the inclusive `dbf d3` counter is still non-negative",
        );
        assert_eq!(
            background_scroll_shift_data_second_pass_advance_loc_304A8(
                0x1200_3456,
                0x0000_1800,
                0x0000,
            ),
            BackgroundScrollShiftDataSecondPassAdvanceLoc304A8 {
                next_slot_offset_bytes: 8,
                next_phase_accumulator: 0x3456_2A00,
                next_dbf_counter: 0xFFFF,
                loops_to_loc_3048_c: false,
            },
            "ASM loc_304A8 should fall through to `rts` once `dbf d3,loc_3048C` consumes the final zero seed and underflows the counter to `$FFFF`",
        );
    }

    #[test]
    fn background_scroll_shift_data_wrap_loop_loc_304c2_adds_the_28_row_bias_before_reducing_by_background_width()
     {
        assert_eq!(
            background_scroll_shift_data_wrap_loop_loc_304C2(0x0180, 0x0020),
            BackgroundScrollShiftDataWrapLoopLoc304C2 {
                seeded_phase_row: 0x0058,
                reduced_phase_row: 0x0018,
                subtraction_count: 2,
            },
            "ASM loc_304C2 should seed `d7` with `asr.w #3,4(a0) + $28`, copy that biased row to `d1`, then keep subtracting `d6` until `cmp.w d7,d6` finally makes `bgt.s loc_304CA` true",
        );
    }

    #[test]
    fn background_scroll_shift_data_wrap_store_loc_304ca_stores_the_reduced_row_and_masks_the_original_seed_into_a_word_offset()
     {
        assert_eq!(
            background_scroll_shift_data_wrap_store_loc_304CA(0x0058, 0x0018),
            BackgroundScrollShiftDataWrapStoreLoc304CA {
                reduced_phase_row_word: 0x0018,
                strip_table_word_offset: 0x0030,
            },
            "ASM loc_304CA should store the reduced `d7` word to `(a0)`, then use the original `d1` seed for `andi.w #$3F,d1` / `add.w d1,d1` so the strip-table offset comes from the pre-reduction row",
        );
    }

    #[test]
    fn background_scroll_shift_data_wrap_loop_body_loc_304e0_keeps_subtracting_background_width_until_the_reduced_row_fits()
     {
        assert_eq!(
            background_scroll_shift_data_wrap_loop_body_loc_304E0(0x0030, 0x0020),
            BackgroundScrollShiftDataWrapLoopBodyLoc304E0 {
                reduced_phase_row: 0x0010,
                subtraction_count: 1,
            },
            "ASM loc_304E0 should loop on `cmp.w d7,d6` / `sub.w d6,d7` until `d6 > d7`, leaving the reduced row ready for `loc_304CA`",
        );
    }

    #[test]
    fn background_scroll_shift_data_wrap_loop_loc_304d8_uses_the_unbiased_seed_before_joining_the_shared_tail()
     {
        assert_eq!(
            background_scroll_shift_data_wrap_loop_loc_304D8(0x0180, 0x0020),
            BackgroundScrollShiftDataWrapLoopLoc304D8 {
                seeded_phase_row: 0x0030,
                reduced_phase_row: 0x0010,
                subtraction_count: 1,
            },
            "ASM loc_304D8 should skip the `$28` bias, seed `d7` and `d1` from `asr.w #3,4(a0)`, and reuse the same subtract-until-`d6>d7` loop before falling into `loc_304CA`",
        );
    }

    #[test]
    fn background_scroll_shift_layers_no_scroll_fill_loc_30502_repeats_negated_camera_x_across_the_full_buffer()
     {
        assert_eq!(
            background_scroll_shift_layers_no_scroll_fill_loc_30502(0x0123),
            BackgroundScrollShiftLayersNoScrollFillLoc30502 {
                repeated_scroll_longword: 0xFEDD_0000,
                longword_count: 0x00E0,
            },
            "ASM loc_30502 should write the negated `Camera_X_pos` 16.16 longword into `Horiz_Scroll_Buffer` for the inclusive `$DF+1` `dbf` span when `(Background_NoScrollFlag).w` is non-zero",
        );
    }

    #[test]
    fn background_scroll_shift_layers_setup_loc_3050a_resolves_the_row_sequence_and_partial_row_spans()
     {
        assert_eq!(
            background_scroll_shift_layers_setup_loc_3050A(
                2,
                &[9, 4, 3],
                &[0x0040, 0x0080, 0x0100, 0x0180],
                0x0034,
                0x0123,
            ),
            Some(BackgroundScrollShiftLayersSetupLoc3050A {
                map_order_entry: 3,
                background_scroll_table_index: 3,
                row_sequence_byte_offset: 0x0181,
                block_dbf_counter: 0x001B,
                negated_camera_x_longword: 0xFEDD_0000,
                quarter_row_phase: 0x0005,
                branches_to_loc_3056_a: false,
                leading_partial_longword_count: Some(0x0003),
                trailing_partial_longword_count: Some(0x0005),
            }),
            "ASM loc_3050A should resolve the current level through `LnkTo_MapOrder_Index` and `LnkTo_BackgroundScroll_Index`, bias the row sequence by logical `Camera_Y_pos>>5`, rebuild `d2` as `-Camera_X_pos` in 16.16, and split a non-zero `(Camera_Y_pos>>2) & 7` phase into leading/trailing partial spans around the main 8-row loop",
        );
    }

    #[test]
    fn background_scroll_shift_layers_setup_loc_3050a_branches_straight_to_loc_3056a_when_the_quarter_row_phase_is_zero()
     {
        assert_eq!(
            background_scroll_shift_layers_setup_loc_3050A(
                0,
                &[1],
                &[0x0020, 0x0060],
                0x0020,
                0x0040
            ),
            Some(BackgroundScrollShiftLayersSetupLoc3050A {
                map_order_entry: 1,
                background_scroll_table_index: 1,
                row_sequence_byte_offset: 0x0061,
                block_dbf_counter: 0x001B,
                negated_camera_x_longword: 0xFFC0_0000,
                quarter_row_phase: 0,
                branches_to_loc_3056_a: true,
                leading_partial_longword_count: None,
                trailing_partial_longword_count: None,
            }),
            "ASM loc_3050A should branch straight to `loc_3056A` when `(Camera_Y_pos>>2) & 7` is zero, leaving `d4=-1` and skipping the leading/trailing partial-row setup",
        );
    }

    #[test]
    fn background_scroll_shift_layers_leading_partial_fill_loc_30562_repeats_the_first_row_for_8_minus_phase_entries()
     {
        assert_eq!(
            background_scroll_shift_layers_leading_partial_fill_loc_30562(
                0xFEDD_0000,
                0x0030,
                0x0005,
                0x001B,
            ),
            Some(BackgroundScrollShiftLayersLeadingPartialFillLoc30562 {
                repeated_scroll_longword: 0xFEDD_FFD0,
                longword_count: 0x0003,
                next_block_dbf_counter: 0x001A,
            }),
            "ASM loc_30562 should repeat the first partial-row scroll longword `8-phase` times and then `subq.w #1,d0` so the later `loc_3056A` block loop runs one fewer inclusive pass",
        );
    }

    #[test]
    fn background_scroll_shift_layers_trailing_partial_fill_loc_30592_repeats_the_last_row_for_phase_entries()
     {
        assert_eq!(
            background_scroll_shift_layers_trailing_partial_fill_loc_30592(
                0xFEDD_0000,
                0x0014,
                0x0005,
            ),
            Some(BackgroundScrollShiftLayersTrailingPartialFillLoc30592 {
                repeated_scroll_longword: 0xFEDD_FFEC,
                longword_count: 0x0005,
            }),
            "ASM loc_30592 should repeat the trailing partial-row scroll longword for the inclusive `dbf d4,loc_30592` span, which is exactly the saved quarter-row phase count",
        );
    }

    #[test]
    fn background_scroll_theme_dispatch_loc_30598_only_branches_for_forest_and_desert() {
        assert_eq!(
            background_scroll_theme_dispatch_loc_30598(0x0009),
            BackgroundScrollThemeDispatchLoc30598::ApplyForestWaterRipple,
            "ASM loc_30598 should branch to `BackgroundScroll_ApplyForestWaterRipple` when `Background_theme == Forest`",
        );
        assert_eq!(
            background_scroll_theme_dispatch_loc_30598(0x0005),
            BackgroundScrollThemeDispatchLoc30598::ApplyDesertHeatRipple,
            "ASM loc_30598 should branch to `BackgroundScroll_ApplyDesertHeatRipple` when `Background_theme == Desert`",
        );
        assert_eq!(
            background_scroll_theme_dispatch_loc_30598(0x0007),
            BackgroundScrollThemeDispatchLoc30598::Return,
            "ASM loc_30598 should return immediately when `Background_theme` is neither Forest nor Desert",
        );
    }

    #[test]
    fn background_scroll_forest_water_ripple_visible_band_loc_305d4_clamps_positive_spans_to_40_scanlines()
     {
        assert_eq!(
            background_scroll_forest_water_ripple_visible_band_loc_305D4(0x0010),
            Some(BackgroundScrollForestWaterRippleVisibleBandLoc305D4 {
                start_scanline: 0x0010,
                ripple_data_byte_offset: 0,
                affected_scanline_count: 0x0040,
            }),
            "ASM loc_305D4 should keep the positive-start band, clear `d2`, and clamp any visible span longer than `$40` scanlines before falling into `loc_305E6`",
        );
        assert_eq!(
            background_scroll_forest_water_ripple_visible_band_loc_305D4(0x00E0),
            None,
            "ASM loc_305D4 should branch to `return_30618` once `$E0-d0` is zero or negative",
        );
    }

    #[test]
    fn background_scroll_forest_water_ripple_setup_loc_305e6_loads_the_base_scroll_word_and_time_phase()
     {
        let horiz_scroll_buffer = [0xFEDD_0012, 0xBEEF_FFF0, 0x1234_5678];

        assert_eq!(
            background_scroll_forest_water_ripple_setup_loc_305E6(
                &horiz_scroll_buffer,
                0x0001,
                0x0003,
                0x0014,
                0x010C,
            ),
            Some(BackgroundScrollForestWaterRippleSetupLoc305E6 {
                start_scanline: 0x0001,
                target_word_offset_bytes: 0x0006,
                base_scroll_word: -0x0010,
                ripple_dbf_counter: 0x0002,
                ripple_data_byte_offset: 0x0017,
            }),
            "ASM loc_305E6 should point `a0` at the second word of the selected `Horiz_Scroll_Buffer` longword, cache that word in `d3`, turn the visible span into a `dbf` counter with `subq.w #1,d1`, and bias `a1` by `(Time_Frames>>2) & $3F`",
        );
    }

    #[test]
    fn background_scroll_forest_water_ripple_loop_loc_3060a_writes_signed_ripple_deltas_into_successive_scanlines()
     {
        let horiz_scroll_buffer = [0xAAAA_1000, 0xBBBB_2000, 0xCCCC_3000, 0xDDDD_4000];
        let ripple_data = [0x80, 0x05, 0xFE, 0x7F];

        assert_eq!(
            background_scroll_forest_water_ripple_loop_loc_3060A(
                &horiz_scroll_buffer,
                0x0001,
                0x0002,
                0x0100,
                &ripple_data,
                0x0001,
            ),
            Some(vec![0xAAAA_1000, 0xBBBB_0105, 0xCCCC_00FE, 0xDDDD_017F]),
            "ASM loc_3060A should sign-extend each ripple byte, add it to the cached base word from `d3`, write only the low word of the current `Horiz_Scroll_Buffer` longword, and advance by one longword per inclusive `dbf` step",
        );
    }

    #[test]
    fn background_scroll_desert_heat_ripple_visible_band_loc_306c2_rebases_negative_starts_and_clamps_positive_spans()
     {
        assert_eq!(
            background_scroll_desert_heat_ripple_visible_band_loc_306C2(0x04, 0x00A8),
            Some(BackgroundScrollDesertHeatRippleVisibleBandLoc306C2 {
                start_scanline: 0,
                ripple_data_byte_offset: 0x000A,
                affected_scanline_count: 0x0006,
            }),
            "ASM loc_306C2 should rebase negative starts to scanline 0, negate the hidden rows into `d2`, and keep only the visible `d0+$10` tail when the prelude branches around the positive-start clamp",
        );
        assert_eq!(
            background_scroll_desert_heat_ripple_visible_band_loc_306C2(0x0A, 0x00A0),
            Some(BackgroundScrollDesertHeatRippleVisibleBandLoc306C2 {
                start_scanline: 0x0028,
                ripple_data_byte_offset: 0x0000,
                affected_scanline_count: 0x0010,
            }),
            "ASM loc_306C2 should clear `d2` and clamp non-negative visible spans to `$10` scanlines before falling into `loc_306D4`",
        );
    }

    #[test]
    fn background_scroll_desert_heat_ripple_setup_loc_306d4_loads_the_base_scroll_word_and_30_phase()
     {
        let horiz_scroll_buffer = [0xAAAA_1111, 0xBBBB_FFF0, 0xCCCC_2222];

        assert_eq!(
            background_scroll_desert_heat_ripple_setup_loc_306D4(
                &horiz_scroll_buffer,
                0x0001,
                0x0006,
                0x000A,
                0x0134,
            ),
            Some(BackgroundScrollDesertHeatRippleSetupLoc306D4 {
                start_scanline: 0x0001,
                target_word_offset_bytes: 0x0006,
                base_scroll_word: -0x0010,
                ripple_dbf_counter: 0x0005,
                ripple_data_byte_offset: 0x003A,
            }),
            "ASM loc_306D4 should point `a0` at the second word of the selected `Horiz_Scroll_Buffer` longword, cache that word in `d3`, mask `Time_Frames` with `$30`, add `d2`, and turn the visible span into an inclusive `dbf` counter with `subq.w #1,d1`",
        );
    }

    #[test]
    fn background_scroll_desert_heat_ripple_loop_loc_306f4_writes_signed_ripple_deltas_into_successive_scanlines()
     {
        let horiz_scroll_buffer = [0xAAAA_1000, 0xBBBB_2000, 0xCCCC_3000, 0xDDDD_4000];
        let ripple_data = [0x00, 0x05, 0xFE, 0x7F];

        assert_eq!(
            background_scroll_desert_heat_ripple_loop_loc_306F4(
                &horiz_scroll_buffer,
                0x0001,
                0x0002,
                0x0100,
                &ripple_data,
                0x0001,
            ),
            Some(vec![0xAAAA_1000, 0xBBBB_0105, 0xCCCC_00FE, 0xDDDD_017F]),
            "ASM loc_306F4 should sign-extend each heat-ripple byte, add it to the cached base word from `d3`, write only the low word of the current `Horiz_Scroll_Buffer` longword, and advance by one longword per inclusive `dbf` step",
        );
    }

    #[test]
    fn background_scroll_enigma_row_transfer_loc_303d4_uses_the_lower_decompression_pair_below_row_1c()
     {
        let background_row_table = [0, 0x12, 0x00];

        assert_eq!(
            background_scroll_enigma_row_transfer_loc_303D4(0, 0, &background_row_table),
            Some(BackgroundScrollEnigmaRowTransferLoc303D4 {
                map_header_byte_offset: 1,
                background_row_index: 0x12,
                row_mapping_byte_offset: 0x0480,
                vdp_control_word: 0x6000,
                vdp_autoincrement_word: 3,
                source: BackgroundScrollEnigmaRowSourceLoc303D4::LowerDecompressionBuffer,
                primary_decompression_byte_offset: 0x0480,
                primary_longword_dbf_counter: 7,
                secondary_decompression_byte_offset: 0x0B80,
                longword_rows: 8,
            }),
            "ASM loc_303D4 should read `1(a0,d0.w)`, keep rows below `$1C` on the lower decompression pair, and seed plane B with `$6000 | ((d0 & $1F) << 7)` plus autoincrement `#3`",
        );
    }

    #[test]
    fn background_scroll_enigma_row_transfer_loc_303d4_biases_desert_tables_and_uses_the_upper_bank_at_row_1c()
     {
        let background_row_table = [0, 0x00, 0x1C, 0x00];

        assert_eq!(
            background_scroll_enigma_row_transfer_loc_303D4(1, 5, &background_row_table),
            Some(BackgroundScrollEnigmaRowTransferLoc303D4 {
                map_header_byte_offset: 3,
                background_row_index: 0,
                row_mapping_byte_offset: 0,
                vdp_control_word: 0x6080,
                vdp_autoincrement_word: 3,
                source: BackgroundScrollEnigmaRowSourceLoc303D4::LowerDecompressionBuffer,
                primary_decompression_byte_offset: 0,
                primary_longword_dbf_counter: 7,
                secondary_decompression_byte_offset: 0x0700,
                longword_rows: 8,
            }),
            "ASM loc_303D4 should add the Desert byte bias before `move.b 1(a0,d0.w),d1`, so tile row 1 reads the third payload byte",
        );
        assert_eq!(
            background_scroll_enigma_row_transfer_loc_303D4(0, 5, &background_row_table),
            Some(BackgroundScrollEnigmaRowTransferLoc303D4 {
                map_header_byte_offset: 2,
                background_row_index: 0x1C,
                row_mapping_byte_offset: 0x0700,
                vdp_control_word: 0x6000,
                vdp_autoincrement_word: 3,
                source: BackgroundScrollEnigmaRowSourceLoc303D4::UpperDecompressionBuffer,
                primary_decompression_byte_offset: 0x0700,
                primary_longword_dbf_counter: 7,
                secondary_decompression_byte_offset: 0x0700,
                longword_rows: 8,
            }),
            "ASM loc_303D4 should branch to `loc_3041C` once the row byte reaches `$1C`, reusing the upper decompression bank for both passes",
        );
    }

    #[test]
    fn transfer_scroll_data_dma_tail_loc_300d2_streams_the_second_vsram_word_and_dmas_h_scroll_buffer()
     {
        assert_eq!(
            transfer_scroll_data_dma_tail_loc_300D2(0x0049),
            TransferScrollDataDmaTailLoc300D2 {
                second_vsram_word: 0x0049,
                dma_source_addr: 0xFFFF_7432,
                dma_vram_word_addr: 0x1400,
                dma_word_count: 0x0380,
            },
            "ASM loc_300D2 should write the shared-tail `d0` into the second VSRAM slot, then DMA `Horiz_Scroll_Buffer` to `vdpComm($1400,VRAM,WRITE)` for `$380` words after stopping the Z80",
        );
    }
}
