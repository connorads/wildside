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
pub(crate) struct LiftBlockHeadbuttWorkerLocFAFE {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloatingPrizeGraphicsSpec {
    pub object_meta: u16,
    pub palette_line: u8,
    pub animation_script_addr: u32,
}

pub fn floating_prize_graphics_spec(
    content: terrain::PrizeContent,
) -> Option<FloatingPrizeGraphicsSpec> {
    // ASM PrizeDiamond_Init (kid.asm:21604), PrizeAnkh_Init (kid.asm:21958),
    // PrizeClock_Init (kid.asm:21991), and PrizeContinue_Init (kid.asm:22038)
    // use palette line 0 with content-specific sprite scripts. ASM sub_10CB0
    // (kid.asm:22600) gives every helmet palette line 3 before each
    // PrizeHelmet* routine selects its own `stru_106D0..stru_10700` script.
    let spec = match content {
        terrain::PrizeContent::Diamond => FloatingPrizeGraphicsSpec {
            object_meta: 0x0064,
            palette_line: 0,
            animation_script_addr: 0x0001_0D6E,
        },
        terrain::PrizeContent::Skycutter => FloatingPrizeGraphicsSpec {
            object_meta: 0x006C,
            palette_line: 3,
            animation_script_addr: 0x0001_06DC,
        },
        terrain::PrizeContent::Cyclone => FloatingPrizeGraphicsSpec {
            object_meta: 0x0070,
            palette_line: 3,
            animation_script_addr: 0x0001_06E8,
        },
        terrain::PrizeContent::RedStealth => FloatingPrizeGraphicsSpec {
            object_meta: 0x0074,
            palette_line: 3,
            animation_script_addr: 0x0001_06F4,
        },
        terrain::PrizeContent::Eyeclops => FloatingPrizeGraphicsSpec {
            object_meta: 0x0078,
            palette_line: 3,
            animation_script_addr: 0x0001_06EE,
        },
        terrain::PrizeContent::Juggernaut => FloatingPrizeGraphicsSpec {
            object_meta: 0x007C,
            palette_line: 3,
            animation_script_addr: 0x0001_06D6,
        },
        terrain::PrizeContent::IronKnight => FloatingPrizeGraphicsSpec {
            object_meta: 0x0080,
            palette_line: 3,
            animation_script_addr: 0x0001_06FA,
        },
        terrain::PrizeContent::Berzerker => FloatingPrizeGraphicsSpec {
            object_meta: 0x0084,
            palette_line: 3,
            animation_script_addr: 0x0001_06E2,
        },
        terrain::PrizeContent::Maniaxe => FloatingPrizeGraphicsSpec {
            object_meta: 0x0088,
            palette_line: 3,
            animation_script_addr: 0x0001_06D0,
        },
        terrain::PrizeContent::Micromax => FloatingPrizeGraphicsSpec {
            object_meta: 0x008C,
            palette_line: 3,
            animation_script_addr: 0x0001_0700,
        },
        terrain::PrizeContent::Ankh => FloatingPrizeGraphicsSpec {
            object_meta: 0x0090,
            palette_line: 0,
            animation_script_addr: 0x0001_0C08,
        },
        terrain::PrizeContent::Clock => FloatingPrizeGraphicsSpec {
            object_meta: 0x0094,
            palette_line: 0,
            animation_script_addr: 0x0001_0C18,
        },
        terrain::PrizeContent::Continue => FloatingPrizeGraphicsSpec {
            object_meta: 0x0098,
            palette_line: 0,
            animation_script_addr: 0x0001_0C28,
        },
        terrain::PrizeContent::Points10000 | terrain::PrizeContent::TenDiamonds => return None,
    };
    Some(spec)
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HelmetCleanupAnimationSub10D3E {
    pub(crate) allocation_spec: u32,
    pub(crate) vram_tile: u16,
    pub(crate) display_enabled: bool,
    pub(crate) x_pos: u16,
    pub(crate) y_pos: u16,
    pub(crate) animation_script_addr: u32,
    pub(crate) deletes_after_animation: bool,
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
pub(crate) struct PrizeCollectedArcHibernateStepLocFEEE {
    pub(crate) next_screen_x_fp: i32,
    pub(crate) next_screen_y_fp: i32,
    pub(crate) screen_x_word: u16,
    pub(crate) screen_y_word: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeCollectedWorldStepLocFEFE {
    pub(crate) world_x_word: u16,
    pub(crate) world_y_word: u16,
    pub(crate) next_loop_counter: u16,
    pub(crate) should_loop: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrizeClockCollectedWorldStepLocFF92 {
    pub(crate) arc_step: PrizeCollectedArcHibernateStepLocFEEE,
    pub(crate) world_step: PrizeCollectedWorldStepLocFEFE,
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
pub(crate) struct ShooterHorizontalSpriteCellLocF808 {
    pub(crate) col: i16,
    pub(crate) row: i16,
    pub(crate) fg_queue_word: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShooterHorizontalSpanLocF808 {
    pub(crate) restored_cols: Vec<i16>,
    pub(crate) sprite_cells: Vec<ShooterHorizontalSpriteCellLocF808>,
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
/// ASM sub_11120 / loc_11196 (kid.asm:23090-23098): the Iron-Knight-hit
/// metal block runs four `subq.w #2,y_pos(a3)` hibernated frames then four
/// `addq.w #2,y_pos(a3)` hibernated frames, producing a 0 → -8 → 0 arc.
pub(crate) const METAL_BLOCK_BOUNCE_Y_DELTAS_LOC_11196: [i16; 8] = [-2, -2, -2, -2, 2, 2, 2, 2];
/// ASM sub_113C0 / loc_113F4 (kid.asm:23335-23339): shifting/lift block
/// workers rise by two pixels for eight hibernated frames before the layout
/// cell above is committed and the source cell is erased.
pub(crate) const SHIFTING_BLOCK_RISE_Y_DELTAS_LOC_113F4: [i16; 8] =
    [-2, -2, -2, -2, -2, -2, -2, -2];
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const EVANESCENT_BLOCK_REVEAL_DESCENT_Y_DELTAS_LOC_1117A: [i16; 4] = [2, 2, 2, 2];
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const EVANESCENT_BLOCK_REVEAL_ASCENT_Y_DELTAS_LOC_11188: [i16; 4] = [-2, -2, -2, -2];

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MetalBlockPushWakeStep {
    pub(crate) x_pos: u16,
    pub(crate) substep_counter: u16,
    pub(crate) reached_neighbour_check: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IronKnightBlockEffectSubAF7A {
    Smash,
    Prize,
    Ice,
    Metal,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IronKnightBlockBreakerCollisionSubAF7A {
    pub(crate) next_block_breaker: bool,
    pub(crate) handled_collision: bool,
    pub(crate) next_y_velocity: Option<i32>,
    pub(crate) effect: Option<IronKnightBlockEffectSubAF7A>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShiftingBlockActivationLoc11364 {
    pub(crate) should_play_sound: bool,
    pub(crate) should_spawn_worker: bool,
    pub(crate) reserved_above_cell: Option<terrain::Cell>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LiftBlockRiseStepLoc113F4 {
    pub(crate) next_y_pos: u16,
    pub(crate) next_loop_counter: u16,
    pub(crate) should_hibernate_one_frame: bool,
    pub(crate) should_repeat_loop: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LiftBlockFallStepLoc1141E {
    pub(crate) next_y_pos: u16,
    pub(crate) next_loop_counter: u16,
    pub(crate) should_hibernate_one_frame: bool,
    pub(crate) should_delete_current_object: bool,
    pub(crate) should_repeat_loop: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterLeftSupportProbeLoc11448 {
    pub(crate) should_probe_support: bool,
    pub(crate) support_col: Option<u16>,
    pub(crate) next_block_col_for_right_probe: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterCenterWorkerGateLoc1146E {
    pub(crate) restored_block_col: u16,
    pub(crate) should_queue_center_worker: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShooterCollisionRightBottomBlocksLocF580 {
    pub(crate) right_block: i16,
    pub(crate) bottom_block: i16,
}

impl NativeRuntime {
    fn shooter_has_support_loc_f106(
        grid: &terrain::BlockGrid,
        col: u16,
        row: u16,
        directions: u8,
    ) -> bool {
        let max_col = grid.width_cells.saturating_sub(1);
        let max_row = grid.height_cells.saturating_sub(1);
        let left = col
            .checked_sub(1)
            .is_some_and(|left_col| grid.cell_at(left_col, row).skin != 0);
        let right = col
            .checked_add(1)
            .filter(|right_col| *right_col <= max_col)
            .is_some_and(|right_col| grid.cell_at(right_col, row).skin != 0);
        let up = row
            .checked_sub(1)
            .is_some_and(|up_row| grid.cell_at(col, up_row).skin != 0);
        let down = row
            .checked_add(1)
            .filter(|down_row| *down_row <= max_row)
            .is_some_and(|down_row| grid.cell_at(col, down_row).skin != 0);

        (directions & 0x01 != 0 && up)
            || (directions & 0x02 != 0 && right)
            // ASM loc_F22E (kid.asm:19806): after shifting to the down
            // shooter bit, `add.w Level_width_tiles,a4; tst.b 1(a4)` marks
            // the slot negative when the lower neighbour has skin.
            || (directions & 0x04 != 0 && down)
            // ASM loc_F23E (kid.asm:19814): after shifting to the left
            // shooter bit, `tst.b -1(a1)` marks the slot negative when the
            // left neighbour's skin byte is non-zero.
            || (directions & 0x08 != 0 && left)
    }

    fn register_shooter_block_entry_loc_f106(
        &mut self,
        grid: &terrain::BlockGrid,
        col: u16,
        row: u16,
        directions: u8,
    ) {
        if self.block_state.shooter_blocks.len() >= Self::SHOOTER_SLOT_COUNT {
            // ASM loc_F254 (kid.asm:19827): when `$FFFFF8C4` has no free
            // shooter slot, the candidate allocation is skipped.
            return;
        }
        if self
            .block_state
            .shooter_blocks
            .iter()
            .any(|entry| entry.col == col && entry.row == row)
        {
            return;
        }

        self.block_state.shooter_blocks.push(ShooterBlockEntry {
            col,
            row,
            directions,
            supported_by_neighbour: Self::shooter_has_support_loc_f106(grid, col, row, directions),
            // ASM loc_F24E (kid.asm:19823): initialise the new shooter slot
            // with delay `$0100` and state word zero. Native folds the slot
            // tick into step_shooter_blocks, so age zero fires on that tick.
            age: 0,
        });
    }

    #[allow(non_snake_case)]
    fn shooter_direction_bits_loc_F106(cell: terrain::Cell) -> Option<u8> {
        // ASM sub_F096 loc_F106/loc_F1DE (kid.asm:19673/19789): visible
        // shooter scan accepts layout words whose block-type nibble is `$C`,
        // then stores `(word & $00FF) - $2D` as the direction bitfield. The
        // first Highwater Pass 2 right-facing shooters are `$EC2F`, so the
        // stored direction is `$02`.
        if cell.has_block && cell.block_type == terrain::BlockType::Drill && cell.skin >= 0x2D {
            return Some(cell.skin.wrapping_sub(0x2D));
        }

        // ASM off_BB80[9] -> loc_BBC2 -> loc_11430: Level 3 type-9 `$E919`
        // blocks are restored by the touch worker and then use the same
        // timed shooter-slot lifecycle as visible Shooter cells in native.
        (cell.has_block
            && matches!(
                cell.block_type,
                terrain::BlockType::Shooter | terrain::BlockType::Mushroom
            ))
        .then_some(cell.skin)
    }

    fn high_word_from_i32(value: i32) -> u16 {
        let bytes = value.to_be_bytes();
        u16::from_be_bytes([bytes[0], bytes[1]])
    }

    #[allow(non_snake_case)]
    pub(crate) fn lift_platform_vertical_step_sub_FD00(y_pos: u32, y_velocity: i32) -> u32 {
        // ASM sub_FD00 (kid.asm:21064): `move.l $E(a3),d7; add.l d7,6(a3)`.
        y_pos.wrapping_add_signed(y_velocity)
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_collected_arc_hibernate_step_loc_FEEE(
        screen_x_fp: i32,
        screen_y_fp: i32,
        x_delta_fp: i32,
        y_delta_fp: i32,
    ) -> PrizeCollectedArcHibernateStepLocFEEE {
        // ASM loc_FEEE (kid.asm:21215): after `Hibernate_Object_1Frame`,
        // add the precomputed ankh HUD deltas to cached screen-relative
        // longwords `$3E/$42`, then read the high word of `$3E` into `d4`.
        let next_screen_x_fp = screen_x_fp.wrapping_add(x_delta_fp);
        let next_screen_y_fp = screen_y_fp.wrapping_add(y_delta_fp);

        PrizeCollectedArcHibernateStepLocFEEE {
            next_screen_x_fp,
            next_screen_y_fp,
            screen_x_word: Self::high_word_from_i32(next_screen_x_fp),
            screen_y_word: Self::high_word_from_i32(next_screen_y_fp),
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_collected_world_step_loc_FEFE(
        arc_step: PrizeCollectedArcHibernateStepLocFEEE,
        camera_x_word: u16,
        camera_y_word: u16,
        loop_counter: u16,
    ) -> PrizeCollectedWorldStepLocFEFE {
        // ASM loc_FEFE (kid.asm:21221): add the camera words back to the
        // cached ankh screen-space words, write `x_pos/y_pos`, then let `dbf`
        // run the 32-frame arc until `d2` underflows to `$FFFF`.
        let next_loop_counter = loop_counter.wrapping_sub(1);

        PrizeCollectedWorldStepLocFEFE {
            world_x_word: arc_step.screen_x_word.wrapping_add(camera_x_word),
            world_y_word: arc_step.screen_y_word.wrapping_add(camera_y_word),
            next_loop_counter,
            should_loop: next_loop_counter != u16::MAX,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_clock_collected_world_step_loc_FF92(
        screen_x_fp: i32,
        screen_y_fp: i32,
        x_delta_fp: i32,
        y_delta_fp: i32,
        camera_x_word: u16,
        camera_y_word: u16,
        loop_counter: u16,
    ) -> PrizeClockCollectedWorldStepLocFF92 {
        // ASM loc_FF92 (kid.asm:21260): the clock collection loop hibernates,
        // advances cached screen-relative `$3E/$42`, writes world `x_pos/y_pos`
        // by adding camera words, and repeats through `dbf d2,loc_FF92`.
        let arc_step = Self::prize_collected_arc_hibernate_step_loc_FEEE(
            screen_x_fp,
            screen_y_fp,
            x_delta_fp,
            y_delta_fp,
        );
        let world_step = Self::prize_collected_world_step_loc_FEFE(
            arc_step,
            camera_x_word,
            camera_y_word,
            loop_counter,
        );

        PrizeClockCollectedWorldStepLocFF92 {
            arc_step,
            world_step,
        }
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
    pub(crate) fn floating_prize_fall_loc_103DE(
        aligned_y: i32,
        y_pos: i64,
        y_velocity: i32,
        first_row_grace: bool,
        level_row: u16,
        level_layout_offset: u32,
        level_width_tiles: u16,
    ) -> FloatingPrizeFallStepLoc10C50 {
        // ASM loc_103DE (kid.asm:21650): once the collision probes leave the
        // fall branch open, add `#$1388` to `d1`, advance `y_pos(a3)`, and
        // feed the snapped-row result into the shared `loc_10402/loc_10404`
        // tail.
        Self::floating_prize_fall_step_loc_10C50(
            aligned_y,
            y_pos,
            y_velocity,
            first_row_grace,
            level_row,
            level_layout_offset,
            level_width_tiles,
        )
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn floating_prize_row_sync_loc_10402(
        aligned_y: i32,
        y_pos: i64,
        y_velocity: i32,
        first_row_grace: bool,
        level_row: u16,
        level_layout_offset: u32,
        level_width_tiles: u16,
    ) -> FloatingPrizeFallStepLoc10C50 {
        // ASM loc_10402 (kid.asm:21663): copy the latest snapped row from
        // `d2` into `d0` after the `loc_103DE` fall step, with the preceding
        // boundary-crossing path already responsible for clearing bit `$10`
        // in `d3` and advancing `a2` by `Level_width_tiles`.
        Self::floating_prize_fall_step_loc_10C50(
            aligned_y,
            y_pos,
            y_velocity,
            first_row_grace,
            level_row,
            level_layout_offset,
            level_width_tiles,
        )
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
        let [layout_hi, layout_lo] = prize_layout_addr.to_be_bytes();
        let [level_hi, level_lo] = current_level_id.to_be_bytes();
        next_entries[usize::from(current_write_index)] =
            u32::from_be_bytes([layout_hi, layout_lo, level_hi, level_lo]);
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
        // ASM loc_FCBE (kid.asm:21030): hibernate one frame, call sub_FD00,
        // increment `d1` only when the integer `6(a3)` word changes, apply
        // the pre-/post-`$32` acceleration, then loop until signed `6(a3)`
        // reaches `Level_height_pixels`.
        lift.age = lift.age.saturating_add(1);
        lift.y_pos = Self::lift_platform_vertical_step_sub_FD00(lift.y_pos, lift.y_velocity);

        let current_integer_y = (lift.y_pos >> 16) as u16;
        if current_integer_y != lift.last_integer_y {
            lift.moved_y_steps = lift.moved_y_steps.wrapping_add(1);
            lift.last_integer_y = current_integer_y;
        }

        // ASM loc_FCD2 (kid.asm:21038-21042): once `d1` has counted more
        // than `$32` changed integer `6(a3)` rows, switch to falling.
        if lift.moved_y_steps > 0x32 {
            lift.y_velocity = lift.y_velocity.wrapping_add(0x07D0);
        } else {
            // ASM loc_FCE2 (kid.asm:21045): while `d1 <= $32`, keep applying
            // the upward acceleration by subtracting `$708` from `$E(a3)`.
            lift.y_velocity = lift.y_velocity.wrapping_sub(0x0708);
        }

        // ASM loc_FCEA (kid.asm:21048-21053): compare signed integer `6(a3)`
        // with `Level_height_pixels`; strictly below loops, otherwise delete.
        let signed_integer_y = i32::from(i16::from_ne_bytes(current_integer_y.to_ne_bytes()));
        // ASM loc_FCFE (kid.asm:21056): the strictly-below branch is only
        // `bra.s loc_FCBE`, so native returns true to keep this platform worker
        // retained for another loop iteration.
        signed_integer_y < i32::from(level_height_pixels)
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn lift_block_rise_step_loc_113F4(
        y_pos: u16,
        loop_counter: u16,
    ) -> LiftBlockRiseStepLoc113F4 {
        // ASM loc_113F4 (kid.asm:23335-23339): `subq.w #2,y_pos(a3)`, hibernate
        // one frame, then `dbf d0,loc_113F4`, so `d0 = 7` yields eight 2px
        // upward steps before the worker falls through to `sub_11530`.
        LiftBlockRiseStepLoc113F4 {
            next_y_pos: y_pos.wrapping_sub(2),
            next_loop_counter: loop_counter.wrapping_sub(1),
            should_hibernate_one_frame: true,
            should_repeat_loop: loop_counter != 0,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn lift_block_fall_step_loc_1141E(
        y_pos: u16,
        loop_counter: u16,
    ) -> LiftBlockFallStepLoc1141E {
        // ASM loc_1141E (kid.asm:23348-23352): after `addi.w #$10,y_pos(a3)`
        // restores the original source row, the worker hibernates through a
        // 16-step `subq.w #1,y_pos(a3)` descent and only then jumps to
        // `Delete_CurrentObject`.
        LiftBlockFallStepLoc1141E {
            next_y_pos: y_pos.wrapping_sub(1),
            next_loop_counter: loop_counter.wrapping_sub(1),
            should_hibernate_one_frame: true,
            should_delete_current_object: loop_counter == 0,
            should_repeat_loop: loop_counter != 0,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_left_support_probe_loc_11448(
        block_col: u16,
        left_support_is_empty: bool,
    ) -> ShooterLeftSupportProbeLoc11448 {
        // ASM loc_11448 (kid.asm:23374-23381): `subq.w #1,d1` first, so the
        // left probe is skipped entirely on the leftmost column. Otherwise
        // `tst.w -2(a1)` gates the `sub_1149C` call, and the shared `loc_11458`
        // tail leaves `d1` at `original + 1` for the later right-hand check.
        let should_probe_support = block_col > 0 && left_support_is_empty;
        ShooterLeftSupportProbeLoc11448 {
            should_probe_support,
            support_col: should_probe_support.then(|| block_col - 1),
            next_block_col_for_right_probe: block_col.wrapping_add(1),
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_center_worker_gate_loc_1146E(
        block_col_after_right_probe: u16,
        queued_support_worker: bool,
    ) -> ShooterCenterWorkerGateLoc1146E {
        // ASM loc_1146E (kid.asm:23394-23397): `subq.w #1,d1` restores the
        // original shooter column after loc_11458's shared `addq.w #1,d1`
        // tail, then `tst.w d4; bne.s loc_11496` suppresses the struck-cell
        // worker whenever any earlier `sub_1149C` call already queued a
        // support worker.
        ShooterCenterWorkerGateLoc1146E {
            restored_block_col: block_col_after_right_probe.wrapping_sub(1),
            should_queue_center_worker: !queued_support_worker,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_center_worker_loc_11474(
        block_col: u16,
        block_row: u16,
    ) -> ShooterHeadbuttWorker {
        // ASM loc_11474 (kid.asm:23399-23407): allocate a `$2001` worker,
        // install `sub_114C0`, then copy `d1/d2/d3` into `$44/$46/$48`. The
        // native equivalent keeps the struck cell address as `(col,row)`.
        ShooterHeadbuttWorker {
            col: block_col,
            row: block_row,
            age: 0,
            kind: ShooterHeadbuttWorkerKind::Center,
        }
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
        let top_block = Self::shooter_collision_top_block_loc_F574(kid_hitbox.top_px);
        let right_bottom = Self::shooter_collision_right_bottom_blocks_loc_F580(
            kid_hitbox.right_px,
            kid_hitbox.bottom_px,
        );
        let right_block = right_bottom.right_block;
        let bottom_block = right_bottom.bottom_block;
        let mut rects = Vec::new();

        // ASM loc_F5A4 (kid.asm:20223): the first active shooter pointer is
        // tested before reading any slot; a zero head skips directly to the
        // shared terminator path.
        for &(col, row, directions) in shooter_blocks {
            let col = col as i16;
            let row = row as i16;
            if col < left_block || col > right_block || row < top_block || row > bottom_block {
                // ASM loc_F5AA (kid.asm:20230): a rejected slot reloads the
                // next active shooter pointer from `$C(a0)`. Native stores
                // active slots densely, so `continue` advances to that next
                // candidate.
                continue;
            }

            let x = col << 4;
            let y = row << 4;
            if directions & 0x01 != 0
                && row - top_block >= 2
                && col != left_block
                && col != right_block
            {
                // ASM loc_F5B0 (kid.asm:20234): once the active shooter slot
                // overlaps Kid's expanded block window, bit 0 in direction
                // flags emits the two upward-facing collision rectangles.
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
                // ASM loc_F62C (kid.asm:20285): the right-facing shooter emits
                // two rectangles when its column is at least two blocks inside
                // the right edge and its row is off the top/bottom edges.
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
                // ASM loc_F67E (kid.asm:20319): the down-facing shooter emits
                // two rectangles when its row is at least two blocks inside the
                // bottom edge and its column is off the left/right edges.
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
                // ASM loc_F6D2 (kid.asm:20354): the left-facing shooter emits
                // two rectangles when its column is at least two blocks inside
                // the left edge and its row is off the top/bottom edges.
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
            // ASM loc_F722 (kid.asm:20387): every skipped left-facing edge
            // falls through to `$C(a0)` and resumes the emitter loop when
            // another active slot exists. Native's dense iteration models
            // that by adding no rectangles for the skipped slot and continuing.
        }

        // ASM loc_F5A6 -> loc_F72A (kid.asm:20226,20391): all termination
        // paths write a zero word at the rectangle output cursor and return.
        // Native returns only the populated rectangles, so an empty or
        // exhausted list has no sentinel element.
        rects
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_collision_top_block_loc_F574(kid_hitbox_top_px: i16) -> i16 {
        // ASM loc_F574 (kid.asm:20203): `asr.w #4; subq.w #1` derives the
        // expanded top block, and negative word results are clamped to zero
        // before the right and bottom block bounds are derived.
        (((i32::from(kid_hitbox_top_px)) >> 4) - 1).max(0) as i16
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_collision_right_bottom_blocks_loc_F580(
        kid_hitbox_right_px: i16,
        kid_hitbox_bottom_px: i16,
    ) -> ShooterCollisionRightBottomBlocksLocF580 {
        // ASM loc_F580 (kid.asm:20210): add `$F` to the right and bottom
        // hitbox words, logically shift each word right by four, then expand
        // the resulting block bounds by one.
        let right_block =
            ((kid_hitbox_right_px as u16).wrapping_add(0x000F) >> 4).wrapping_add(1) as i16;
        let bottom_block =
            ((kid_hitbox_bottom_px as u16).wrapping_add(0x000F) >> 4).wrapping_add(1) as i16;

        ShooterCollisionRightBottomBlocksLocF580 {
            right_block,
            bottom_block,
        }
    }

    // Oracle-backed sub_1007A timing for demo 0x1B:
    // hit frame N:      write 0x6F
    // frames N+1..N+32: keep 0x6F during the bounce coroutine
    // frame N+33:       restore original prize cell (E1xx)
    // frame N+34:       clear cell and spawn prize object
    pub(crate) const PRIZE_BLOCK_DELETED_FRAMES: u16 = 32;
    pub(crate) const PRIZE_BLOCK_WORKER_TAIL_FRAMES: u8 = 3;
    pub(crate) const PRIZE_BLOCK_SPAWN_FLASH_FRAMES: u8 = 25;
    pub(crate) const PRIZE_DIAMOND_LIFETIME_FRAMES: u16 = 0x01F4;
    pub(crate) const PRIZE_GENERIC_LIFETIME_FRAMES: u16 = 0x0320;
    // PrizeDiamond_Init starts moving on the frame after object creation, so a
    // newly spawned native floating item should be active immediately.
    pub(crate) const PRIZE_ITEM_INITIAL_DORMANT_FRAMES: u16 = 0;

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn lift_block_headbutt_worker_loc_FAFE(
        block_col: u16,
        block_row: u16,
        layout_word_addr: u32,
    ) -> LiftBlockHeadbuttWorkerLocFAFE {
        // ASM loc_FAFE (kid.asm:20881): allocate object/gfx slots through
        // `sub_E02`, install `sub_FB3E`, copy `d1/d2/d3` into `$44/$46/$48(a0)`,
        // and seed the gfx object's pixel position from the block coordinates.
        LiftBlockHeadbuttWorkerLocFAFE {
            allocation_spec: 0x2001,
            object_handler_addr: 0x0000_FB3E,
            block_col,
            block_row,
            layout_word_addr,
            gfx_x_pos: block_col << 4,
            gfx_y_pos: block_row << 4,
        }
    }

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
    pub(crate) fn prize_block_wobble_fall_loc_100F6() -> &'static [i16] {
        // ASM loc_100F6 (kid.asm:21385): after `moveq #4,d0`, the inclusive
        // `dbf d0,loc_100F6` loop applies five `subq.w #2,y_pos(a3)` steps.
        &PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A[12..17]
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_wobble_rise_loc_10104() -> &'static [i16] {
        // ASM loc_10104 (kid.asm:21391): after `moveq #4,d1`, the inclusive
        // `dbf d1,loc_10104` loop applies five `addq.w #2,y_pos(a3)` steps.
        &PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A[17..22]
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_wobble_fall_loc_10112() -> &'static [i16] {
        // ASM loc_10112 (kid.asm:21397): after `moveq #3,d0`, the inclusive
        // `dbf d0,loc_10112` loop applies four `subq.w #2,y_pos(a3)` steps.
        &PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A[22..26]
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_wobble_rise_loc_10120() -> &'static [i16] {
        // ASM loc_10120 (kid.asm:21403): after `moveq #4,d1`, the inclusive
        // `dbf d1,loc_10120` loop applies five `addq.w #2,y_pos(a3)` steps
        // before `j_Hibernate_UntilAnimFinished` takes over.
        &PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A[26..31]
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
    pub(crate) fn evanescent_block_reveal_alt_branch_iterations_loc_11196() -> u16 {
        // ASM loc_11196 (kid.asm:23090): when `$4C(a5)` is non-zero, the
        // alternate branch reloads `d0` with `#3` immediately before
        // `loc_11198`, so the inclusive `dbf` loop still runs four hibernated
        // frames before the later rebound.
        4
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn evanescent_block_reveal_alt_branch_y_deltas_loc_11198() -> &'static [i16; 4] {
        // ASM loc_11198 (kid.asm:23093): after `loc_11196` reloads `d0`, the
        // alternate branch spends four hibernated frames on
        // `subq.w #2,y_pos(a3)` before `loc_111A6` mirrors the motion.
        &EVANESCENT_BLOCK_REVEAL_ASCENT_Y_DELTAS_LOC_11188
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn evanescent_block_reveal_descent_y_deltas_loc_1117A() -> &'static [i16; 4] {
        // ASM loc_1117A (kid.asm:23077): when `$4C(a5)` is zero, `moveq #3,d0`
        // drives an inclusive `dbf` loop, so the worker spends four hibernated
        // frames on `addq.w #2,y_pos(a3)` before the mirrored rebound.
        &EVANESCENT_BLOCK_REVEAL_DESCENT_Y_DELTAS_LOC_1117A
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn evanescent_block_reveal_ascent_y_deltas_loc_11188() -> &'static [i16; 4] {
        // ASM loc_11188 (kid.asm:23083): after reloading `d0` with `#3`, the
        // worker spends four hibernated frames on `subq.w #2,y_pos(a3)` before
        // `loc_111B2` hands control to `Hibernate_UntilAnimFinished`.
        &EVANESCENT_BLOCK_REVEAL_ASCENT_Y_DELTAS_LOC_11188
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

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn helmet_cleanup_animation_sub_10D3E(
        x_pos: u16,
        y_pos: u16,
    ) -> HelmetCleanupAnimationSub10D3E {
        // ASM sub_10D3E (kid.asm:22653): the transient cleanup worker loads
        // gfx slot spec `$02000000`, clears `vram_tile(a3)`, sets `$13(a3)`,
        // copies its cached `$44/$46(a5)` coordinates, runs `stru_102DE`, then
        // deletes itself as soon as `Hibernate_UntilAnimFinished` returns.
        HelmetCleanupAnimationSub10D3E {
            allocation_spec: 0x0200_0000,
            vram_tile: 0,
            display_enabled: true,
            x_pos,
            y_pos,
            animation_script_addr: 0x0001_02DE,
            deletes_after_animation: true,
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

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn helmet_cleanup_effect_loc_10cec(
        content: terrain::PrizeContent,
        x_pos: u32,
        y_pos: i64,
    ) -> Option<BlockDestroyEffect> {
        if !Self::is_helmet_prize(content) {
            return None;
        }

        Some(BlockDestroyEffect {
            // ASM loc_10CF4 -> loc_10D38 (kid.asm:22608-22644): once the scan
            // finds an older helmet owner in the `$6C..$8C` range, sub_10CEC
            // copies its integer x, subtracts 8px from y, then seeds the
            // shared eight-frame cleanup animation at that adjusted point.
            x_px: (x_pos >> 16) as u16,
            y_px: ((y_pos >> 16) as u16).saturating_sub(8),
            remaining: 8,
            direction: Self::BLOCK_DESTROY_DIRECTION_ABOVE,
        })
    }

    fn clear_existing_helmet_prizes_for_sub_10cec(&mut self) {
        let mut removed_effects = Vec::new();
        self.floating_items.retain(|item| {
            let Some(effect) =
                Self::helmet_cleanup_effect_loc_10cec(item.content, item.x_pos, item.y_pos)
            else {
                return true;
            };
            removed_effects.push(effect);
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
    const BLOCK_DEBRIS_GRAVITY_STEP_FP_SUB_2A4C: i32 = 0x0000_36B0;
    const BLOCK_DEBRIS_BASE_X_SUB_292E: [i16; 5] = [0, 0, 8, 8, 4];
    const BLOCK_DEBRIS_BASE_Y_SUB_292E: [i16; 5] = [0, 8, 8, 0, 4];
    const BLOCK_DEBRIS_VEL_D6_2_SUB_292E: [(i16, i16); 5] =
        [(-1, -4), (-1, -3), (1, -3), (1, -4), (1, -5)];
    const BLOCK_DEBRIS_VEL_D6_0_SUB_292E: [(i16, i16); 5] =
        [(-1, -2), (-2, 0), (1, 0), (1, -1), (-1, -2)];
    const BLOCK_DEBRIS_VEL_D6_1_SUB_292E: [(i16, i16); 5] =
        [(-5, -3), (-4, -3), (-3, -3), (-2, -3), (-1, -3)];
    const BLOCK_DEBRIS_VEL_D6_DEFAULT_SUB_292E: [(i16, i16); 5] =
        [(5, -3), (4, -3), (3, -3), (2, -3), (1, -3)];

    fn block_debris_velocity_sub_292e(direction: u16) -> &'static [(i16, i16); 5] {
        match direction {
            Self::BLOCK_DESTROY_DIRECTION_ABOVE => &Self::BLOCK_DEBRIS_VEL_D6_2_SUB_292E,
            Self::BLOCK_DESTROY_DIRECTION_BELOW => &Self::BLOCK_DEBRIS_VEL_D6_0_SUB_292E,
            Self::BLOCK_DESTROY_DIRECTION_LEFT => &Self::BLOCK_DEBRIS_VEL_D6_1_SUB_292E,
            _ => &Self::BLOCK_DEBRIS_VEL_D6_DEFAULT_SUB_292E,
        }
    }

    fn block_debris_swarm_sub_2a4c(
        x_px: u16,
        y_px: u16,
        direction: u16,
    ) -> BlockDebrisSwarmSub2A4C {
        let velocities = Self::block_debris_velocity_sub_292e(direction);
        BlockDebrisSwarmSub2A4C {
            pieces: core::array::from_fn(|index| BlockDebrisPieceSub2A4C {
                // ASM sub_292E (kid.asm:4881) seeds word x/y positions and
                // x/y velocities; sub_2A4C then treats them as 16.16 longs.
                x_pos_fp: i32::from(x_px)
                    .saturating_add(i32::from(Self::BLOCK_DEBRIS_BASE_X_SUB_292E[index]))
                    << 16,
                y_pos_fp: i32::from(y_px)
                    .saturating_add(i32::from(Self::BLOCK_DEBRIS_BASE_Y_SUB_292E[index]))
                    << 16,
                x_velocity_fp: i32::from(velocities[index].0) << 16,
                y_velocity_fp: i32::from(velocities[index].1) << 16,
            }),
        }
    }
    const IRON_KNIGHT_BLOCK_BREAKER_MIN_Y_VELOCITY_SUB_AF7A: i32 = 0x0004_8000;
    const IRON_KNIGHT_BLOCK_BREAKER_REBOUND_Y_VELOCITY_SUB_AF7A: i32 = -0x0000_8000;

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn iron_knight_block_breaker_collision_sub_AF7A(
        is_iron_knight: bool,
        current_block_breaker: bool,
        y_velocity: i32,
        solid_collision: bool,
        block_type: terrain::BlockType,
    ) -> IronKnightBlockBreakerCollisionSubAF7A {
        // ASM sub_AF7A (kid.asm:13801): non-Iron-Knight forms and the first
        // slow hit return zero without touching `Iron_Knight_block_breaker`.
        if !is_iron_knight
            || (!current_block_breaker
                && y_velocity < Self::IRON_KNIGHT_BLOCK_BREAKER_MIN_Y_VELOCITY_SUB_AF7A)
        {
            return IronKnightBlockBreakerCollisionSubAF7A {
                next_block_breaker: current_block_breaker,
                handled_collision: false,
                next_y_velocity: None,
                effect: None,
            };
        }

        // ASM loc_AF96 sets `Iron_Knight_block_breaker` before probing the
        // contacted block with sub_BBE2, so even rejected dispatches keep the
        // latch for the follow-up collision scan.
        let next_block_breaker = true;
        if !solid_collision {
            return IronKnightBlockBreakerCollisionSubAF7A {
                next_block_breaker,
                handled_collision: false,
                next_y_velocity: None,
                effect: None,
            };
        }

        // ASM off_AFD8 dispatches block types 0..5 only. SolidGhost and
        // Teleporter jump through sub_AFD4, which unwinds to the unhandled
        // loc_AFCA return; Metal is handled by loc_110D0 without deletion.
        let effect = match block_type {
            terrain::BlockType::Smash => Some(IronKnightBlockEffectSubAF7A::Smash),
            terrain::BlockType::Prize => Some(IronKnightBlockEffectSubAF7A::Prize),
            terrain::BlockType::Ice => Some(IronKnightBlockEffectSubAF7A::Ice),
            terrain::BlockType::Metal => Some(IronKnightBlockEffectSubAF7A::Metal),
            terrain::BlockType::SolidGhost
            | terrain::BlockType::Teleporter
            | terrain::BlockType::Rubber
            | terrain::BlockType::Shifting
            | terrain::BlockType::Shooter
            | terrain::BlockType::Mushroom
            | terrain::BlockType::Evanescent
            | terrain::BlockType::Lift
            | terrain::BlockType::Drill
            | terrain::BlockType::Deleted
            | terrain::BlockType::None => None,
        };

        IronKnightBlockBreakerCollisionSubAF7A {
            next_block_breaker,
            handled_collision: effect.is_some(),
            next_y_velocity: effect
                .is_some()
                .then_some(Self::IRON_KNIGHT_BLOCK_BREAKER_REBOUND_Y_VELOCITY_SUB_AF7A),
            effect,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn rubber_block_base_skin_parts_for_sub_10de4(
        block_type: terrain::BlockType,
        skin: u8,
    ) -> Option<u8> {
        if block_type != terrain::BlockType::Rubber {
            return None;
        }

        // ASM loc_10E06 (kid.asm:22730): rubber skins `$0C/$0D` share the
        // low animation pair, so the worker seeds `d0` with `$E60C` before the
        // shared `loc_10E0A` toggles between `$E60C` and `$E60D`.
        Some(match skin {
            0x0C | 0x0D => 0x0C,
            _ => 0x0E,
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn rubber_block_base_skin_for_sub_10de4(cell: terrain::Cell) -> Option<u8> {
        Self::rubber_block_base_skin_parts_for_sub_10de4(cell.block_type, cell.skin)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn rubber_block_animation_slot_matches_sub_10de4(
        entry_col: u16,
        entry_row: u16,
        col: u16,
        row: u16,
    ) -> bool {
        // ASM sub_10DE4 (kid.asm:22717): the worker reuses the stored layout
        // cell only when both the column and row identify the same block.
        entry_col == col && entry_row == row
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn rubber_block_animation_wait_state_sub_10de4(
        frames_until_next_write: u8,
    ) -> (u8, bool) {
        // ASM loc_10E0A -> sub_10DE4 (kid.asm:22733-22762): each hibernate
        // count decrements toward zero, and the next skin write happens only
        // after the countdown reaches exactly zero.
        let next_frames_until_next_write = frames_until_next_write.saturating_sub(1);
        (
            next_frames_until_next_write,
            next_frames_until_next_write > 0,
        )
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn rubber_block_animation_next_step_sub_10de4(phase: u8) -> Option<(u8, u8, bool)> {
        let next_phase = phase.saturating_add(1);
        let &(skin_offset, next_delay) =
            Self::RUBBER_BLOCK_SUB_10DE4_SEQUENCE.get(usize::from(next_phase))?;
        // ASM sub_10DE4 (kid.asm:22733-22766): the ninth and final write uses
        // the last `(0, 0)` table entry and then jumps straight to
        // `Delete_CurrentObject`, so no worker remains after that step.
        let keeps_running =
            usize::from(next_phase.saturating_add(1)) < Self::RUBBER_BLOCK_SUB_10DE4_SEQUENCE.len();
        Some((skin_offset, next_delay, keeps_running))
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
            .find(|entry| {
                Self::rubber_block_animation_slot_matches_sub_10de4(entry.col, entry.row, col, row)
            })
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
                let (next_frames_until_next_write, should_keep_waiting) =
                    Self::rubber_block_animation_wait_state_sub_10de4(
                        entry.frames_until_next_write,
                    );
                entry.frames_until_next_write = next_frames_until_next_write;
                if should_keep_waiting {
                    return true;
                }

                let Some((skin_offset, next_delay, keeps_running)) =
                    Self::rubber_block_animation_next_step_sub_10de4(entry.phase)
                else {
                    return false;
                };
                entry.phase = entry.phase.saturating_add(1);
                pending_writes.push((entry.col, entry.row, entry.base_skin, skin_offset));
                entry.frames_until_next_write = next_delay;
                keeps_running
            });

        for (col, row, base_skin, skin_offset) in pending_writes {
            self.write_rubber_block_skin_for_sub_10de4(col, row, base_skin, skin_offset);
        }
    }

    #[allow(non_snake_case)]
    fn smash_block_destroy_effect_origin_sub_10EDA(
        direction: u16,
        x_px: u16,
        y_px: u16,
    ) -> (u16, u16) {
        match direction {
            // ASM loc_10F26 / loc_10FE8 (kid.asm:22833,22908): both smash and
            // ice helpers share the `addq.w #4,x_pos(a3)` branch, nudging the
            // effect 4 px right when `d6 == 3`.
            Self::BLOCK_DESTROY_DIRECTION_RIGHT => (x_px.saturating_add(4), y_px),
            // ASM loc_10F1A (kid.asm:22825): `subq.w #4,y_pos(a3)` nudges the
            // smash effect 4 px upward when `d6 == 2`.
            Self::BLOCK_DESTROY_DIRECTION_ABOVE => (x_px, y_px.saturating_sub(4)),
            // ASM loc_10F20 / loc_10FE2 (kid.asm:22829,22904): both smash and
            // ice helpers share the `subq.w #4,x_pos(a3)` branch, nudging the
            // effect 4 px left when `d6 == 1`.
            Self::BLOCK_DESTROY_DIRECTION_LEFT => (x_px.saturating_sub(4), y_px),
            // ASM sub_10EDA fallthrough before loc_10F2A (kid.asm:22831):
            // when `d6 == 0`, the routine takes the default `addq.w #4,
            // y_pos(a3)` path before the shared sprite-swap / hibernate tail
            // at `loc_10F2A`.
            _ => (x_px, y_px.saturating_add(4)),
        }
    }

    #[allow(non_snake_case)]
    fn ice_block_destroy_effect_origin_sub_10F9C(
        direction: u16,
        x_px: u16,
        y_px: u16,
    ) -> (u16, u16) {
        match direction {
            // ASM loc_10FE8 (kid.asm:22908): `d6 == 3` nudges the ice shard
            // worker 4 px right before the shared `loc_10FEC` sprite swap.
            Self::BLOCK_DESTROY_DIRECTION_RIGHT => (x_px.saturating_add(4), y_px),
            // ASM loc_10FDC (kid.asm:22900): `d6 == 2` nudges the ice shard
            // worker 4 px upward before the 3-frame hibernate.
            Self::BLOCK_DESTROY_DIRECTION_ABOVE => (x_px, y_px.saturating_sub(4)),
            // ASM loc_10FE2 (kid.asm:22904): `d6 == 1` nudges the ice shard
            // worker 4 px left before the shared `loc_10FEC` tail.
            Self::BLOCK_DESTROY_DIRECTION_LEFT => (x_px.saturating_sub(4), y_px),
            // ASM sub_10F9C (kid.asm:22881): when `d6` is neither 1, 2, nor
            // 3, the routine falls through to `addq.w #4,y_pos(a3)` before
            // the shared sprite swap and 3-frame hibernate at `loc_10FEC`.
            _ => (x_px, y_px.saturating_add(4)),
        }
    }

    pub(crate) fn spawn_smash_block_effect_sub_10eda(
        &mut self,
        x_px: u16,
        y_px: u16,
        direction: u16,
    ) {
        let (x_px, y_px) = Self::smash_block_destroy_effect_origin_sub_10EDA(direction, x_px, y_px);
        self.block_destroy_effects.push(BlockDestroyEffect {
            // ASM loc_10F2A (kid.asm:22836): after the direction-specific
            // offset branch, sub_10EDA swaps to `LnkTo_unk_E0E56-Data_Index`,
            // hibernates for 2 frames, then dispatches the shared debris
            // worker and deletes the coroutine.
            x_px,
            y_px,
            remaining: 2,
            direction,
        });
    }

    pub(crate) fn spawn_ice_block_effect_sub_10f9c(
        &mut self,
        x_px: u16,
        y_px: u16,
        direction: u16,
    ) {
        let (x_px, y_px) = Self::ice_block_destroy_effect_origin_sub_10F9C(direction, x_px, y_px);
        self.block_destroy_effects.push(BlockDestroyEffect {
            // ASM sub_10F9C / loc_10FEC (kid.asm:22881,22910): after the
            // direction-specific nudge, the worker marks the effect visible,
            // swaps to the ice-block art, hibernates for 3 frames, then
            // dispatches the shared debris helper and deletes itself.
            x_px,
            y_px,
            remaining: 3,
            direction,
        });
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn ice_block_bullet_target_cell_sub_11006(
        source_col: u16,
        source_row: u16,
        direction: u16,
        level_width_blocks: u16,
        level_height_blocks: u16,
    ) -> Option<(u16, u16)> {
        let remapped_direction = direction ^ 2;
        if remapped_direction == 0 {
            return source_row.checked_sub(1).map(|row| (source_col, row));
        }

        if remapped_direction < 2 {
            // ASM loc_11020 (kid.asm:22936): after `eori.w #2,d6`, the
            // `d6 == 1` path probes the cell one block to the right and bails
            // once `source_col + 1 >= Level_width_blocks`.
            let target_col = source_col.saturating_add(1);
            return (target_col < level_width_blocks).then_some((target_col, source_row));
        }

        if remapped_direction == 2 {
            // ASM loc_11036 (kid.asm:22947): the remapped `d6 == 2` branch
            // advances one block downward and aborts when
            // `source_row + 1 >= Level_height_blocks`.
            let target_row = source_row.saturating_add(1);
            return (target_row < level_height_blocks).then_some((source_col, target_row));
        }

        // ASM loc_1104A (kid.asm:22957): the final branch treats any other
        // remapped direction as left, aborting on column 0 before subtracting
        // one block from the source column.
        source_col.checked_sub(1).map(|col| (col, source_row))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn ice_block_bullet_target_cell_open_loc_11052(cell: terrain::Cell) -> bool {
        // ASM loc_11052 (kid.asm:22963): `btst #6,(a0)` rejects the target
        // cell whenever the high collision bit is set, which covers slope,
        // solid, and spike terrain but still allows Empty and HiddenBlock.
        !matches!(
            cell.collision,
            terrain::CollisionKind::SlopeUp
                | terrain::CollisionKind::SlopeDown
                | terrain::CollisionKind::Solid
                | terrain::CollisionKind::Spikes
        )
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn ice_block_load_bullets_restore_direction_loc_11080(
        remapped_direction: u16,
    ) -> u16 {
        // ASM loc_11080 (kid.asm:22983): the shared epilogue restores `d6`
        // with `eori.w #2,d6` before the saved registers are popped and the
        // routine returns.
        remapped_direction ^ 2
    }

    pub(crate) fn step_evanescent_blocks(&mut self, slice: &OpeningSliceState) {
        // ASM loc_EAE4 -> loc_EB72 scans Kid's expanded hitbox for the first
        // bit-15 type-$A block. The trigger is body contact, not just standing
        // on the cell; Highwater Pass' first evanescent shooters depend on this.
        let trigger = self
            .block_grid
            .as_ref()
            .and_then(|grid| {
                Self::evanescent_hitbox_trigger_loc_EAE4(
                    slice,
                    grid,
                    self.progression.current_form.half_width(),
                )
            })
            .filter(|(col, row, _)| {
                !self
                    .block_state
                    .evanescent_blocks
                    .iter()
                    .any(|entry| entry.col == *col && entry.row == *row)
                    && self.block_state.evanescent_blocks.len() < EVANESCENT_MAX_ENTRIES
            });

        if let Some((col, row, skin)) = trigger {
            if let Some(grid) = &mut self.block_grid {
                // ASM EvanescentBlock_Trigger writes `#$E3` to the high byte
                // immediately, preserving the low skin byte for later redraws.
                grid.set_cell_at(col, row, terrain::Cell::from_be_bytes(0xE3, skin));
            }
            let initial_timer = i16::from(skin) - EVANESCENT_SKIN_OFFSET;
            self.block_state.evanescent_blocks.push(EvanescentEntry {
                col,
                row,
                timer: initial_timer,
            });
            self.fire_evanescent_block_projectiles_loc_EC76(col, row, skin);
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

    #[allow(non_snake_case)]
    fn evanescent_hitbox_trigger_loc_EAE4(
        slice: &OpeningSliceState,
        grid: &terrain::BlockGrid,
        collision_half_width: i32,
    ) -> Option<(u16, u16, u8)> {
        let grid_width_pixels = u16::try_from(grid.width_pixels()).ok()?;
        let grid_height_pixels = u16::try_from(grid.height_pixels()).ok()?;
        let level_width_pixels = slice.level_bounds.width_pixels.min(grid_width_pixels);
        let level_height_pixels = slice.level_bounds.height_pixels.min(grid_height_pixels);
        if level_width_pixels == 0 || level_height_pixels == 0 {
            return None;
        }

        let x_px = (slice.player.x_pos >> 16) as i32;
        let foot_y_px = (slice.player.y_pos >> 16) as i32;
        let top_px = u16::try_from((foot_y_px - PLAYER_HEIGHT).saturating_sub(1).max(0)).ok()?;
        let bottom_px = crate::level::evanescent_bullet_bottom_clamp_loc_EAF8(
            u16::try_from(foot_y_px.max(0)).ok()?,
            level_height_pixels,
        )
        .bottom_px;
        if bottom_px < top_px {
            return None;
        }
        let vertical_rows =
            crate::level::evanescent_bullet_vertical_rows_loc_EB0C(top_px, bottom_px);
        // ASM loc_EAE4 scans `Kid_hitbox_left/right`, whose horizontal half-width
        // comes from sub_B41C/unk_B408 for the current helmet form.
        let left_px = u16::try_from((x_px - collision_half_width).saturating_sub(1).max(0)).ok()?;
        let right_px = crate::level::evanescent_bullet_right_clamp_loc_EB2A(
            u16::try_from((x_px + collision_half_width).max(0)).ok()?,
            level_width_pixels,
        )
        .right_px;
        if right_px < left_px {
            return None;
        }
        let horizontal_columns =
            crate::level::evanescent_bullet_horizontal_columns_loc_EB3E(left_px, right_px);
        let hit = crate::level::evanescent_bullet_scan_rows_loc_EB48(
            grid,
            vertical_rows.top_row,
            vertical_rows.row_dbf_counter,
            horizontal_columns.left_col,
            horizontal_columns.col_dbf_counter,
        )?;
        let skin = grid.cell_at(hit.col, hit.row).skin;
        Some((hit.col, hit.row, skin))
    }

    #[allow(non_snake_case)]
    fn fire_evanescent_block_projectiles_loc_EC76(&mut self, col: u16, row: u16, skin: u8) {
        // ASM EvanescentBlock_Trigger stores low-byte `skin - $1C` as the
        // direction bitfield, then loc_EC76 loads bullets for bits 3..0.
        let skin_offset =
            u8::try_from(EVANESCENT_SKIN_OFFSET).expect("evanescent skin offset fits in u8");
        let directions = skin.wrapping_sub(skin_offset);
        let block_x = u32::from(col) * 16 + 8;
        let block_y = u32::from(row) * 16 + 8;
        let speed = Self::SHOOTER_PROJECTILE_SPEED;
        for (mask, xv, yv) in [
            (0x08, -speed, 0),
            (0x04, 0, speed),
            (0x02, speed, 0),
            (0x01, 0, -speed),
        ] {
            if directions & mask != 0 {
                self.projectiles.push(EnemyProjectile {
                    x_pos: block_x << 16,
                    y_pos: block_y << 16,
                    x_velocity: xv,
                    y_velocity: yv,
                    lifetime: 120,
                    ..Default::default()
                });
            }
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
    const SHOOTER_SLOT_COUNT: usize = 20;
    const SHOOTER_HEADBUTT_HOLD_FRAMES: u8 = 8;

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_horizontal_span_loc_F808(
        camera_left_col: i16,
        camera_right_col: i16,
        camera_top_row: i16,
        camera_bottom_row: i16,
        start_col: i16,
        row: i16,
        tile_count: u8,
    ) -> ShooterHorizontalSpanLocF808 {
        let restored_cols = Self::shooter_horizontal_span_columns_loc_F828(start_col, tile_count);
        if Self::shooter_horizontal_span_rejected_loc_F830(
            camera_left_col,
            camera_right_col,
            camera_top_row,
            camera_bottom_row,
            start_col,
            row,
        ) {
            return ShooterHorizontalSpanLocF808 {
                restored_cols,
                sprite_cells: Vec::new(),
            };
        }

        let sprite_cells = restored_cols
            .iter()
            .copied()
            .filter(|col| *col >= camera_left_col && *col <= camera_right_col)
            .map(|col| {
                // ASM loc_F808 (kid.asm:20482): each span cell first writes
                // `$E304` back to the layout, then visible cells enqueue the
                // compact `(row << 8 & $F00) | (col << 2 & $7C)` FG word.
                let row_word = (i32::from(row) << 8) & 0x0F00;
                let col_word = (i32::from(col) << 2) & 0x007C;
                let fg_queue_word =
                    u16::try_from(row_word | col_word).expect("masked FG queue word fits u16");
                ShooterHorizontalSpriteCellLocF808 {
                    col,
                    row,
                    fg_queue_word,
                }
            })
            .collect::<Vec<_>>();

        ShooterHorizontalSpanLocF808 {
            restored_cols,
            sprite_cells,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_horizontal_span_columns_loc_F828(
        start_col: i16,
        tile_count: u8,
    ) -> Vec<i16> {
        // ASM loc_F828 (kid.asm:20498): increment `d0` and DBF back to
        // loc_F808, so the span is an inclusive counted run of `tile_count`
        // cells even when only a suffix is inside the camera window.
        (0..tile_count)
            .map(|offset| start_col + i16::from(offset))
            .collect::<Vec<_>>()
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_horizontal_span_rejected_loc_F830(
        camera_left_col: i16,
        camera_right_col: i16,
        camera_top_row: i16,
        camera_bottom_row: i16,
        start_col: i16,
        row: i16,
    ) -> bool {
        // ASM loc_F830 (kid.asm:20504): if the row or starting column fails
        // the coarse viewport tests, write the restore word across the whole
        // span and skip all custom FG update queue writes.
        let row_visible = row <= camera_bottom_row && row >= camera_top_row;
        let span_may_enter_view = start_col <= camera_right_col && start_col - camera_left_col > -8;
        !(row_visible && span_may_enter_view)
    }

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
        // ASM loc_F76C (kid.asm:20421): if the decremented timer remains
        // non-zero, branch to the loop tail without touching the state word.
        let decremented_timer = timer.wrapping_sub(1);
        if decremented_timer != 0 {
            return (decremented_timer, state);
        }

        if state & 0x8000 != 0 {
            return match state {
                0x8001 => (decremented_timer.wrapping_add(2), state.wrapping_add(1)),
                0x8004 => {
                    // ASM loc_F79A (kid.asm:20440): `$8004` is the only
                    // post-`bgt` negative state that advances one more step
                    // and reloads the timer with the two-frame delay.
                    (decremented_timer.wrapping_add(2), state.wrapping_add(1))
                }
                0x8002 => {
                    // ASM loc_F78C (kid.asm:20434): `$8002` exits the
                    // negative visibility ladder by storing state `3` and
                    // restoring the slot reload delay from `6(a0)`.
                    (reload_delay, 3)
                }
                _ => {
                    // ASM loc_F7AA (kid.asm:20448): `$8005` clears the state
                    // word and restores the active delay from `4(a0)` before
                    // redrawing the `$E304` shooter span.
                    (active_delay, 0)
                }
            };
        }

        if state == 0 {
            // ASM loc_F8A6 (kid.asm:20577): a zero state stores `$8001`
            // and adds two frames to the expired slot timer before the
            // horizontal redraw setup continues.
            (decremented_timer.wrapping_add(2), 0x8001)
        } else {
            // ASM loc_F898 (kid.asm:20570): a non-zero positive state starts
            // the flash pair by storing `$8004` and adding two frames.
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
        // ASM loc_F564 (kid.asm:20187): every failed `bgt`/`bmi` range test
        // returns zero in `d0`.
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

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_slot_sleep_frames_loc_F31C() -> u8 {
        // ASM loc_F31C (kid.asm:19909): the shooter slot's timer byte is
        // loaded with `$32` before returning to the shared redraw tail.
        0x32
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_slot_restart_phase_loc_F326(player_in_range: bool) -> Option<u16> {
        // ASM loc_F326 (kid.asm:19914): `sub_F536` returns non-zero while Kid
        // remains in range; that path writes phase `$A(a0)=1`. Zero falls
        // through to the free-slot unlink path.
        if player_in_range { Some(1) } else { None }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shooter_slot_freed_cell_loc_F336(skin: u8) -> terrain::Cell {
        // ASM loc_F336 (kid.asm:19921): before unlinking the active shooter
        // slot, `move.b #$EC,(a3)` restores the layout high byte and preserves
        // the skin byte already stored at the cell's low address.
        terrain::Cell::from_be_bytes(0xEC, skin)
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
        let mut early_spawned_prizes = Vec::new();
        if let Some(grid) = &mut self.block_grid {
            self.block_state
                .deferred_ceiling_clears
                .retain_mut(|entry| match entry.phase {
                    DeferredPrizeBlockPhase::Deleted => {
                        if entry.deleted_frames_remaining == 0 {
                            grid.set_cell_at(entry.col, entry.row, entry.restore_cell);
                            if entry.content == terrain::PrizeContent::Diamond {
                                // ASM PrizeDiamond_Init (kid.asm:21604) uses
                                // `sf is_moved(a3)`, unlike the helmet/shared
                                // prize initialisers that `st is_moved(a3)`.
                                // The diamond therefore enters the object
                                // roster on the same wake that restores the
                                // prize cell, before the later `$E001` write.
                                early_spawned_prizes
                                    .push(Self::floating_item_from_prize_clear(entry));
                                entry.phase = DeferredPrizeBlockPhase::PendingClearPrizeSpawned;
                            } else {
                                entry.phase = DeferredPrizeBlockPhase::Restored;
                            }
                        } else {
                            entry.deleted_frames_remaining -= 1;
                            if entry.deleted_frames_remaining == 0 {
                                // ASM sub_1007A reaches the post-wobble
                                // animation/spawn hand-off while the original
                                // #$2001 object/gfx allocation is still live.
                                self.prize_block_spawn_flashes.push(PrizeBlockSpawnFlash {
                                    // ASM sub_1007A (kid.asm:21512): after
                                    // spawning loc_102B8, d1/d2 are converted
                                    // to gfx coords with `lsl #4`, `+8` x and
                                    // `-8` y before `stru_102DE` starts.
                                    x_px: entry.col.saturating_mul(16).saturating_add(8),
                                    y_px: entry.row.saturating_mul(16).saturating_sub(8),
                                    remaining: Self::PRIZE_BLOCK_SPAWN_FLASH_FRAMES,
                                });
                            }
                        }
                        true
                    }
                    DeferredPrizeBlockPhase::Restored => {
                        if entry.content == terrain::PrizeContent::Diamond {
                            early_spawned_prizes.push(Self::floating_item_from_prize_clear(entry));
                            entry.phase = DeferredPrizeBlockPhase::PendingClearPrizeSpawned;
                        } else {
                            entry.phase = DeferredPrizeBlockPhase::PendingClear;
                        }
                        true
                    }
                    DeferredPrizeBlockPhase::PendingClear
                    | DeferredPrizeBlockPhase::PendingClearPrizeSpawned
                    | DeferredPrizeBlockPhase::FreshSmash => true,
                });
        }
        self.floating_items.extend(early_spawned_prizes);
    }

    fn floating_item_from_prize_clear(entry: &DeferredCeilingClear) -> FloatingItem {
        FloatingItem {
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
                DeferredPrizeBlockPhase::PendingClear
                | DeferredPrizeBlockPhase::PendingClearPrizeSpawned => {
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
                    if entry.phase == DeferredPrizeBlockPhase::PendingClear {
                        spawned_items.push(Self::floating_item_from_prize_clear(&entry));
                    }
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
            if self.block_state.shooter_blocks.len() + pending_entries.len()
                >= Self::SHOOTER_SLOT_COUNT
            {
                // ASM loc_F254 (kid.asm:19827): when `$FFFFF8C4` has no free
                // shooter slot, the candidate allocation is skipped and the
                // scan simply advances `a1/d3` to continue the DBF loop.
                return;
            }
            // ASM loc_F106 (kid.asm:19673): a raw shooter candidate pops one
            // free slot, pushes it to the active list, then stores the layout
            // address, column, row, and direction bits. Native works on decoded
            // grid cells and queues the same entry shape before ticking slots.
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

        // ASM loc_F09E (kid.asm:19623): cache Kid's block X in `$FFFFF8BE`;
        // unchanged columns branch to loc_F186 and leave the column scan alone.
        if previous_col != kid_block_col {
            let entering_col = if previous_col < kid_block_col {
                (kid_block_col + 3).min(max_col)
            } else {
                // ASM loc_F0D0 (kid.asm:19643): moving left scans `kid_x_block-3`,
                // clamped to column zero when the subtraction goes negative.
                (kid_block_col - 3).max(0)
            } as u16;
            // ASM loc_F0D8 (kid.asm:19649): scan rows from `kid_y_block-5`,
            // clamped to zero, through the later `kid_y_block+2` lower bound.
            let start_row = (kid_block_row - 5).max(0) as u16;
            // ASM loc_F0E0 (kid.asm:19655): build the inclusive bottom row as
            // `kid_y_block+2`, clamping to `Level_height_blocks-1`.
            let end_row = (kid_block_row + 2).min(max_row) as u16;
            // ASM loc_F0F0 (kid.asm:19663): subtract the top row from the
            // bottom row for DBF's inclusive count, load `$FFFF4A04[d4*2]`,
            // then add `d3*2` to address the entering column.
            for row in start_row..=end_row {
                let cell = grid.cell_at(entering_col, row);
                if let Some(directions) = Self::shooter_direction_bits_loc_F106(cell) {
                    queue_scan(entering_col, row, directions);
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
                if let Some(directions) = Self::shooter_direction_bits_loc_F106(cell) {
                    queue_scan(col, entering_row, directions);
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
                        supported_by_neighbour: Self::shooter_has_support_loc_f106(
                            grid, col, row, directions,
                        ),
                        // ASM loc_F24E (kid.asm:19823): initialise the new
                        // shooter slot with delay `$0100` and state word zero.
                        // Native folds the slot tick into this scan, so age
                        // starts at zero here and advances below.
                        age: 0,
                    }),
            );

        self.block_state.shooter_blocks.retain_mut(|entry| {
            let still_shooter =
                Self::shooter_direction_bits_loc_F106(grid.cell_at(entry.col, entry.row)).is_some();
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
                            kind: EnemyProjectileKind::ShooterBlock,
                            // ASM LoadBulletsFromEvanescentBlock/Process_ShooterObjects
                            // stores a plain shooter object: no gravity, homing,
                            // friendly damage, or wall bounce metadata.
                            ..Default::default()
                        });
                    }
                }
            }

            entry.age += 1;
            // ASM loc_F336 (kid.asm:19921): the ROM unlinks expired active
            // shooter slots back to `$FFFFF8C4`; native's retain false path is
            // the same active-list removal in Vec form.
            entry.age < Self::SHOOTER_FIRE_INTERVAL
        });
    }

    pub(crate) fn step_shooter_headbutt_workers(&mut self) {
        let Some(grid) = &mut self.block_grid else {
            self.block_state.shooter_headbutt_workers.clear();
            return;
        };

        let mut restored_shooter_cells = Vec::new();
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
                            // ASM sub_114F4 tests `(a1)` before queueing any tile writes; if
                            // the support cell refilled, `bne.s loc_1152C` jumps straight to
                            // `Delete_CurrentObject` on the first wake frame.
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
                            let restored_cell = Self::shooter_support_tile_for_sub_114f4();
                            grid.set_cell_at(worker.col, worker.row, restored_cell);
                            restored_shooter_cells.push((worker.col, worker.row, restored_cell));
                            return false;
                        }
                        _ => {}
                    },
                }

                worker.age = worker.age.saturating_add(1);
                true
            });

        let grid_snapshot = self.block_grid.clone();
        if let Some(grid) = grid_snapshot.as_ref() {
            for (col, row, cell) in restored_shooter_cells {
                if let Some(directions) = Self::shooter_direction_bits_loc_F106(cell) {
                    // ASM sub_114F4 (kid.asm:23459) restores support cells as
                    // `$E919`. Native represents those restored Highwater Pass
                    // touch cells in the same shooter-slot list used by
                    // sub_F096, so queue them as soon as the worker finishes
                    // instead of waiting for an unrelated movement scan edge.
                    self.register_shooter_block_entry_loc_f106(grid, col, row, directions);
                }
            }
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shifting_block_worker_gate_loc_113BA(
        source_row: u16,
        above_cell_sets_bit_6: bool,
    ) -> bool {
        // ASM loc_113BA (kid.asm:23314-23315): both the `tst.w d2` early exit
        // and the occupied-above `bne.s` path land on this shared `rts`, so
        // the worker setup only proceeds when the source row is non-zero and
        // bit 6 of the cell above is clear.
        source_row != 0 && !above_cell_sets_bit_6
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn shifting_block_activation_loc_11364(
        source_row: u16,
        above_cell: terrain::Cell,
    ) -> ShiftingBlockActivationLoc11364 {
        // ASM loc_11364 (kid.asm:23287-23315): preserve `d0`, play
        // `sfx_Shifting_block`, then only enter the `sub_113C0` worker setup
        // when `d2 != 0` and `btst #6,(a2)` sees the cell above as clear. Bit
        // 6 is the high collision bit, so slope, solid, and spike terrain all
        // block the worker even without bit 15 set. The setup reserves that
        // above cell by overwriting only its high byte with `$6F`; native
        // compresses the later worker into an immediate upward move once this
        // gate passes.
        let above_cell_sets_bit_6 = matches!(
            above_cell.collision,
            terrain::CollisionKind::SlopeUp
                | terrain::CollisionKind::SlopeDown
                | terrain::CollisionKind::Solid
                | terrain::CollisionKind::Spikes
        );
        let should_spawn_worker =
            Self::shifting_block_worker_gate_loc_113BA(source_row, above_cell_sets_bit_6);
        ShiftingBlockActivationLoc11364 {
            should_play_sound: true,
            should_spawn_worker,
            reserved_above_cell: should_spawn_worker
                .then_some(terrain::Cell::from_be_bytes(0x6F, above_cell.skin)),
        }
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

        let activation =
            Self::shifting_block_activation_loc_11364(foot_row, grid.cell_at(col, foot_row - 1));
        if !activation.should_spawn_worker {
            return;
        }

        // Move block up: copy cell to row above, clear current cell
        grid.set_cell_at(col, foot_row - 1, cell);
        grid.set_cell_at(col, foot_row, terrain::Cell::from_be_bytes(0, 0));
        // ASM loc_11364 -> sub_113C0 (kid.asm:23303-23339): native keeps the
        // collision-visible grid swap immediate, but the ROM still draws a
        // worker sprite rising two pixels for eight hibernated frames from the
        // source row into the reserved cell above.
        self.block_bounces.push(BlockBounceAnim {
            cell_x: col,
            cell_y: foot_row - 1,
            remaining: 8,
            initial_duration: 8,
            kind: BlockBounceKind::Shifting,
            prize_content: None,
            y_offset_px: 16,
        });
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

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn lift_block_bob_up_pass_loc_FB94() -> &'static [i16] {
        // ASM loc_FB94 (kid.asm:20931): `moveq #5,d0` makes the DBF loop run
        // six times, subtracting 2px from the linked gfx object's y position
        // before each one-frame hibernate.
        &[-2, -2, -2, -2, -2, -2]
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn lift_block_bob_down_pass_loc_FBA2() -> &'static [i16] {
        // ASM loc_FBA2 (kid.asm:20937): `moveq #5,d1` makes the DBF loop run
        // six times, adding 2px to the linked gfx object's y position before
        // each one-frame hibernate.
        &[2, 2, 2, 2, 2, 2]
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn lift_block_bob_up_pass_loc_FBB0() -> &'static [i16] {
        // ASM loc_FBB0 (kid.asm:20943): `moveq #4,d0` makes the DBF loop run
        // five times, subtracting 2px from the linked gfx object's y position
        // before each one-frame hibernate.
        &[-2, -2, -2, -2, -2]
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn lift_block_bob_down_pass_loc_FBBE() -> &'static [i16] {
        // ASM loc_FBBE (kid.asm:20949): `moveq #4,d1` makes the DBF loop run
        // five times, adding 2px to the linked gfx object's y position before
        // each one-frame hibernate.
        &[2, 2, 2, 2, 2]
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn lift_block_bob_up_pass_loc_FBCC() -> &'static [i16] {
        // ASM loc_FBCC (kid.asm:20955): `moveq #3,d0` makes the DBF loop run
        // four times, subtracting 2px from the linked gfx object's y position
        // before each one-frame hibernate.
        &[-2, -2, -2, -2]
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn lift_block_bob_down_pass_loc_FBDA() -> &'static [i16] {
        // ASM loc_FBDA (kid.asm:20961): `moveq #4,d1` makes the DBF loop run
        // five times, adding 2px to the linked gfx object's y position before
        // each one-frame hibernate.
        &[2, 2, 2, 2, 2]
    }

    /// Process block hit effects from the current frame's slice (prizes, bounces,
    /// lifts, drills, teleporter warps).
    pub(crate) fn process_block_effects(&mut self, slice: &mut OpeningSliceState) {
        // ASM loc_1002E (kid.asm:21317): the prize-block headbutt plays
        // `sfx_Block_hit` and spawns the sub_1007A wobble worker. It does
        // NOT spawn `Block_LoadSmashBits` debris — smash bits are reserved
        // for type-0 smash blocks via the `slice.smash_block_hit` branch
        // below. Spawning destroy fragments here was an invented placeholder
        // that produced visible debris over an intact (still wobbling)
        // prize block.
        if slice.prize_block_hit.is_some() {
            self.blocks_hit_this_frame += 1;
        }

        // Metal block bounce animation
        if let Some((cx, cy)) = slice.metal_block_hit {
            let (cell_x, cell_y) = Self::metal_block_effect_cell_words_loc_110D0(cx, cy);
            self.blocks_hit_this_frame += 1;
            self.block_bounces.push(BlockBounceAnim {
                cell_x,
                cell_y,
                remaining: 8,
                initial_duration: 8,
                kind: BlockBounceKind::Metal,
                prize_content: None,
                y_offset_px: 0,
            });
        }

        // ASM loc_10DA4 (kid.asm:22692): rubber contact plays sfx_Rubber_block
        // and spawns the `sub_10DE4` skin-swap coroutine. The ASM rubber
        // animation is a tile-skin cycle (`sub_11530` writes `$E60C` /
        // `$E60D` / `$E60E` over 9 hibernate-bracketed writes); the ROM never
        // touches `y_pos(a3)` for rubber, so native must NOT also push a
        // BlockBounceAnim — that was an invented vertical wobble.
        if let Some((cx, cy)) = slice.rubber_block_hit {
            self.blocks_hit_this_frame += 1;
            self.start_rubber_block_animation_sub_10de4(cx, cy);
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
            let mut should_queue_center_worker = true;

            if cell_y > 0 {
                let support_row = cell_y - 1;
                let mut queued_support_worker = false;
                let mut try_queue_support = |support_col: u16| {
                    if grid.cell_at(support_col, support_row) == terrain::Cell::from_be_bytes(0, 0)
                    {
                        // ASM sub_1149C: queue one support worker per empty cell above the
                        // struck shooter, including the diagonals when in bounds.
                        queued_support_worker = true;
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

                should_queue_center_worker = Self::shooter_center_worker_gate_loc_1146E(
                    cell_x.wrapping_add(1),
                    queued_support_worker,
                )
                .should_queue_center_worker;
            }

            if should_queue_center_worker {
                let shooter_cell = grid.cell_at(cell_x, cell_y);
                // ASM loc_11474 resets `a1` from `d3`, marks the struck
                // shooter deleted-solid via `or.b #$F,(a1)`, then installs the
                // `sub_114C0` centre worker.
                grid.set_cell_at(
                    cell_x,
                    cell_y,
                    Self::shooter_deleted_tile_for_sub_114c0(shooter_cell.skin),
                );
                self.block_state
                    .shooter_headbutt_workers
                    .push(Self::shooter_center_worker_loc_11474(cell_x, cell_y));
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
                lifetime: 120,
                // ASM sub_1129C/sub_10EDA creates the simple horizontal drill
                // effect; the remaining projectile fields stay at native defaults.
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

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn floating_micromax_delete_exit_loc_106CC() -> bool {
        // ASM loc_106CC (kid.asm:21917): Micromax's fall loop exits by
        // jumping straight to `Delete_CurrentObject`.
        true
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn metal_block_effect_cell_words_loc_110D0(cell_x: u16, cell_y: u16) -> (u16, u16) {
        // ASM loc_110D0 (kid.asm:23019): after `j_sub_E02`, the worker stores
        // the tile coordinates unmodified in `$44(a0)` and `$46(a0)` before
        // the sprite-position copy at loc_1110E.
        (cell_x, cell_y)
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn metal_block_push_entry_loc_111F4(
        source_col: u16,
        direction: u16,
        level_width_blocks: u16,
    ) -> Option<u16> {
        // ASM loc_111F4 (kid.asm:23130): seed `a2` from the source
        // `Level_Layout` word in `d3`, branch to the right-walking scan only
        // when `d6 == 3`, and otherwise enter the left-walking path. The
        // right branch pre-advances to the next column and aborts once the
        // source block already sits on `Level_width_blocks-1`.
        if direction == 3 {
            let next_col = source_col.saturating_add(1);
            return (next_col < level_width_blocks).then_some(next_col);
        }

        source_col.checked_sub(1)
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn metal_block_push_scan_right_loc_11208(
        row: &[terrain::Cell],
        source_col: u16,
    ) -> Option<u16> {
        // ASM loc_11208 (kid.asm:23139): starting from the cell immediately
        // right of the struck metal block, keep walking while each sampled
        // cell is both collision `$6000` and block type `$500`. The first
        // non-solid cell falls through `loc_11256` and selects the previous
        // metal block, while any in-bounds solid non-metal or empty-bit15
        // cell aborts at `loc_11296`.
        let width = u16::try_from(row.len()).expect("row slices should fit in u16");
        let mut current_col = source_col.saturating_add(1);
        while current_col < width {
            let cell = row[usize::from(current_col)];
            if cell.collision != terrain::CollisionKind::Solid {
                return current_col.checked_sub(1);
            }
            if !cell.has_block || cell.block_type != terrain::BlockType::Metal {
                return None;
            }
            current_col = current_col.saturating_add(1);
        }

        None
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn metal_block_push_scan_left_loc_11230(
        row: &[terrain::Cell],
        source_col: u16,
    ) -> Option<u16> {
        // ASM loc_11230 (kid.asm:23156): starting from the cell immediately
        // left of the struck metal block, keep walking while each sampled cell
        // is both collision `$6000` and block type `$500`. The first
        // non-solid cell falls through `loc_1125A` and selects the previous
        // metal block, while the left-edge test or any in-bounds solid
        // non-metal / bit-15-clear cell aborts at `loc_11296`.
        let mut current_col = source_col.checked_sub(1)?;
        loop {
            let cell = row[usize::from(current_col)];
            if cell.collision != terrain::CollisionKind::Solid {
                return Some(current_col.saturating_add(1));
            }
            if !cell.has_block || cell.block_type != terrain::BlockType::Metal {
                return None;
            }
            let next_col = current_col.checked_sub(1)?;
            current_col = next_col;
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn metal_block_push_mark_deleted_loc_1125C(cell: terrain::Cell) -> terrain::Cell {
        // ASM loc_1125C (kid.asm:23181): `or.b #$F,(a2)` keeps bit 15, the
        // collision nibble, and the skin byte intact while forcing the
        // selected metal block's type nibble to `$F` (deleted-solid).
        terrain::Cell::from_be_bytes(0x80 | ((cell.collision as u8) << 4) | 0x0F, cell.skin)
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn metal_block_push_step_right_loc_112E2(
        x_pos: u16,
        substep_counter: u16,
    ) -> MetalBlockPushWakeStep {
        let substep_counter = substep_counter.saturating_add(1);
        // ASM loc_112E2 (kid.asm:23224): after each 1-frame hibernate, the
        // right-moving branch adds 4 px to `x_pos(a3)`, increments `d3`, and
        // hands off to `loc_11334` once `d3 == 4`.
        MetalBlockPushWakeStep {
            x_pos: x_pos.saturating_add(4),
            substep_counter,
            reached_neighbour_check: substep_counter == 4,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn metal_block_push_step_left_loc_112FE(
        x_pos: u16,
        substep_counter: u16,
    ) -> MetalBlockPushWakeStep {
        let substep_counter = substep_counter.saturating_add(1);
        // ASM loc_112FE (kid.asm:23236): the left-moving wake mirrors the
        // right branch, subtracting 4 px from `x_pos(a3)`, incrementing `d3`,
        // and handing off to `loc_1130C` once `d3 == 4`.
        MetalBlockPushWakeStep {
            x_pos: x_pos.saturating_sub(4),
            substep_counter,
            reached_neighbour_check: substep_counter == 4,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn metal_block_push_left_delete_gate_loc_11328(
        source_col: u16,
        left_neighbour: terrain::Cell,
    ) -> bool {
        // ASM loc_1130C -> loc_11328 (kid.asm:23244-23274): after the
        // 4-substep leftward wake reaches its neighbour check, the worker
        // deletes immediately when `d1 <= 0`. Otherwise it probes the cell one
        // column further left and only loops back to `loc_112E2` when
        // `andi.w #$7000,(a1)` equals `$6000`, i.e. the neighbour collision
        // nibble is Solid. Any non-solid neighbour falls through loc_11328.
        source_col == 0 || left_neighbour.collision != terrain::CollisionKind::Solid
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn metal_block_effect_sprite_pos_loc_1110E(cell_x: u16, cell_y: u16) -> (u16, u16) {
        let (cell_x, cell_y) = Self::metal_block_effect_cell_words_loc_110D0(cell_x, cell_y);
        // ASM loc_1110E (kid.asm:23038): copy the cached tile coordinates into
        // sprite space with `lsl.w #4`, so the bounce worker renders at the
        // struck block's top-left 16px cell origin.
        (cell_x << 4, cell_y << 4)
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn metal_block_push_right_delete_gate_loc_11358(
        source_col: u16,
        level_width_blocks: u16,
        right_neighbour: terrain::Cell,
    ) -> bool {
        // ASM loc_11334 -> loc_11358 (kid.asm:23275-23284): after the
        // 4-substep rightward wake reaches its neighbour check, the worker
        // deletes immediately once `d1 >= Level_width_blocks-1`. Otherwise it
        // probes the cell one column further right and only loops back to
        // `loc_112E2` when the neighbour's masked collision nibble is `$6000`.
        source_col.saturating_add(1) >= level_width_blocks
            || right_neighbour.collision != terrain::CollisionKind::Solid
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

        // ASM sub_1007A (kid.asm:21345-21599), sub_11120 (kid.asm:23050),
        // and sub_113C0 (kid.asm:23322): each hibernate cycle applies one
        // entry from the relevant y-position script. Prize blocks use the
        // long wobble table, metal blocks use the loc_11196 up/down bounce,
        // and shifting blocks use loc_113F4's eight two-pixel rise steps.
        for b in &mut self.block_bounces {
            let elapsed = b.initial_duration.saturating_sub(b.remaining);
            let delta_table: &[i16] = match b.kind {
                BlockBounceKind::Prize => &PRIZE_BLOCK_WOBBLE_Y_DELTAS_SUB_1007A,
                BlockBounceKind::Metal => &METAL_BLOCK_BOUNCE_Y_DELTAS_LOC_11196,
                BlockBounceKind::Shifting => &SHIFTING_BLOCK_RISE_Y_DELTAS_LOC_113F4,
            };
            if let Some(delta) = delta_table.get(usize::from(elapsed)) {
                b.y_offset_px = b.y_offset_px.saturating_add(*delta);
            }
            b.remaining = b.remaining.saturating_sub(1);
        }
        for tail in &mut self.prize_block_worker_tails {
            tail.remaining = tail.remaining.saturating_sub(1);
        }
        self.prize_block_worker_tails
            .retain(|tail| tail.remaining > 0);

        let mut finished_prize_bounce_tail_frames = Vec::new();
        self.block_bounces.retain(|b| {
            if b.remaining > 0 {
                true
            } else {
                if b.kind == BlockBounceKind::Prize {
                    let tail_frames = if b.prize_content == Some(terrain::PrizeContent::Diamond) {
                        // ASM PrizeDiamond_Init's `sf is_moved(a3)` makes the
                        // original sub_1007A object/gfx allocation overlap the
                        // diamond's first live frame. Helmet/shared prize
                        // paths set `is_moved` and keep the shorter overlap.
                        Self::PRIZE_BLOCK_WORKER_TAIL_FRAMES.saturating_add(1)
                    } else {
                        Self::PRIZE_BLOCK_WORKER_TAIL_FRAMES
                    };
                    finished_prize_bounce_tail_frames.push(PrizeBlockWorkerTail {
                        cell_x: b.cell_x,
                        cell_y: b.cell_y,
                        content: b.prize_content,
                        remaining: tail_frames,
                    });
                }
                false
            }
        });
        self.prize_block_worker_tails
            .extend(finished_prize_bounce_tail_frames);

        for flash in &mut self.prize_block_spawn_flashes {
            flash.remaining = flash.remaining.saturating_sub(1);
        }
        self.prize_block_spawn_flashes
            .retain(|flash| flash.remaining > 0);

        for e in &mut self.block_destroy_effects {
            e.remaining = e.remaining.saturating_sub(1);
        }
        let mut new_debris_swarms = Vec::new();
        self.block_destroy_effects.retain(|e| {
            if e.remaining > 0 {
                return true;
            }
            new_debris_swarms.push(Self::block_debris_swarm_sub_2a4c(
                e.x_px,
                e.y_px,
                e.direction,
            ));
            false
        });
        self.block_debris_swarms.extend(new_debris_swarms);
        for swarm in &mut self.block_debris_swarms {
            for piece in &mut swarm.pieces {
                // ASM sub_2A4C (kid.asm:5053): per frame, add velocity to
                // 16.16 position, then add `$36B0` gravity to y velocity.
                piece.x_pos_fp = piece.x_pos_fp.wrapping_add(piece.x_velocity_fp);
                piece.y_pos_fp = piece.y_pos_fp.wrapping_add(piece.y_velocity_fp);
                piece.y_velocity_fp = piece
                    .y_velocity_fp
                    .wrapping_add(Self::BLOCK_DEBRIS_GRAVITY_STEP_FP_SUB_2A4C);
            }
        }
        self.block_debris_swarms.retain(|swarm| {
            swarm.pieces.iter().any(|piece| {
                let x = piece.x_pos_fp >> 16;
                let y = piece.y_pos_fp >> 16;
                (-0x0100..0x2000).contains(&x) && (-0x0100..0x0400).contains(&y)
            })
        });

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
            // loc_105A4 (kid.asm:21813), loc_105EE (kid.asm:21839),
            // loc_10638 (kid.asm:21865), loc_10682 (kid.asm:21891), and
            // loc_10768 (kid.asm:21987) are the matching Skycutter/Cyclone/
            // Red Stealth/Eyeclops/Juggernaut/Iron Knight/Berzerker/Maniaxe/
            // PrizeAnkh delete exits: once the per-frame loop at
            // loc_10464/loc_104AE/loc_104F8/loc_10542/loc_1058C/loc_105D6/
            // loc_10620/loc_1066A/loc_106B4/loc_10738 sees either a non-zero
            // `collision_type(a3)` or the bottom-of-level compare, control
            // jumps straight to `Delete_CurrentObject`. Micromax uses the
            // dedicated delete exit at ASM loc_106CC (kid.asm:21917).
            let gate =
                Self::prize_clock_fall_gate_loc_107B0(integer_y, level_height_pixels, f.collected);
            let stays_alive = (f.spawn_delay > 0 || f.remaining > 0) && gate.stays_active;
            if stays_alive {
                true
            } else if f.content == terrain::PrizeContent::Micromax {
                !Self::floating_micromax_delete_exit_loc_106CC()
            } else {
                false
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::CollisionRectSub1F52;
    use crate::types::{LiftPlatform, ShooterHeadbuttWorkerKind};
    use std::path::PathBuf;

    fn runtime_for_block_tests() -> NativeRuntime {
        NativeRuntime::new(kid_data::AssetManifest {
            generated_bundle_version: 0,
            rom_sha1: String::new(),
            upstream_commit: String::new(),
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
        })
    }

    #[test]
    fn lift_and_prize_headbutt_workers_shift_block_coordinates_into_pixels() {
        // ASM loc_FAFE / loc_1002E: `lsl.w #4,d4` scales the block column/row
        // into gfx pixel coordinates before storing them at `$1A/$1E(a1)`.
        let lift = NativeRuntime::lift_block_headbutt_worker_loc_FAFE(0x12, 0x0B, 0x00AB_CDEF);
        assert_eq!(lift.gfx_x_pos, 0x0120);
        assert_eq!(lift.gfx_y_pos, 0x00B0);

        let prize = NativeRuntime::prize_block_headbutt_worker_loc_1002E(0x09, 0x07, 0x0012_3456);
        assert_eq!(prize.gfx_x_pos, 0x0090);
        assert_eq!(prize.gfx_y_pos, 0x0070);
    }

    #[test]
    fn prize_block_wobble_tables_match_asm_loc_100da_through_loc_10120() {
        // ASM loc_100DA/100E8/100F6/10104/10112/10120: inclusive DBF loops
        // yield 6/6/5/5/4/5 hibernated steps with +/-2px deltas.
        assert_eq!(
            NativeRuntime::prize_block_wobble_y_deltas_sub_1007A(),
            &[
                -2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, 2, -2, -2, -2, -2, -2, 2, 2, 2, 2, 2, -2,
                -2, -2, -2, 2, 2, 2, 2, 2,
            ]
        );
        assert_eq!(
            NativeRuntime::prize_block_wobble_fall_loc_100F6(),
            &[-2, -2, -2, -2, -2]
        );
        assert_eq!(
            NativeRuntime::prize_block_wobble_rise_loc_10104(),
            &[2, 2, 2, 2, 2]
        );
        assert_eq!(
            NativeRuntime::prize_block_wobble_fall_loc_10112(),
            &[-2, -2, -2, -2]
        );
        assert_eq!(
            NativeRuntime::prize_block_wobble_rise_loc_10120(),
            &[2, 2, 2, 2, 2]
        );
        assert_eq!(
            NativeRuntime::deleted_tile_wobble_y_deltas_loc_10200(),
            NativeRuntime::prize_block_wobble_y_deltas_sub_1007A()
        );
    }

    #[test]
    fn evanescent_block_reveal_tables_match_asm_loc_1117a_through_loc_11198() {
        // ASM loc_1117A/11188/11196/11198: `moveq #3,d0` makes each DBF loop
        // run four hibernated steps, with the alternate branch reusing the
        // same four-step ascent delta sequence.
        assert_eq!(
            NativeRuntime::evanescent_block_reveal_alt_branch_iterations_loc_11196(),
            4
        );
        assert_eq!(
            NativeRuntime::evanescent_block_reveal_descent_y_deltas_loc_1117A(),
            &[2, 2, 2, 2]
        );
        assert_eq!(
            NativeRuntime::evanescent_block_reveal_ascent_y_deltas_loc_11188(),
            &[-2, -2, -2, -2]
        );
        assert_eq!(
            NativeRuntime::evanescent_block_reveal_alt_branch_y_deltas_loc_11198(),
            NativeRuntime::evanescent_block_reveal_ascent_y_deltas_loc_11188()
        );
    }

    #[test]
    fn prize_collection_helpers_match_asm_loc_feee_through_loc_ff92() {
        // ASM loc_FEEE/loc_FEFE/loc_FF92: the collection arc advances cached
        // screen-space FP words, then reprojects the high words back into
        // world coordinates while DBF keeps looping until the counter
        // underflows from zero to `$FFFF`.
        assert_eq!(NativeRuntime::high_word_from_i32(0x1234_5678), 0x1234);

        let arc = NativeRuntime::prize_collected_arc_hibernate_step_loc_FEEE(
            0x0010_0000,
            0x0008_0000,
            0x0001_8000,
            -0x0000_8000,
        );
        assert_eq!(arc.next_screen_x_fp, 0x0011_8000);
        assert_eq!(arc.next_screen_y_fp, 0x0007_8000);
        assert_eq!(arc.screen_x_word, 0x0011);
        assert_eq!(arc.screen_y_word, 0x0007);

        let world = NativeRuntime::prize_collected_world_step_loc_FEFE(arc, 0x0100, 0x0020, 1);
        assert_eq!(world.world_x_word, 0x0111);
        assert_eq!(world.world_y_word, 0x0027);
        assert_eq!(world.next_loop_counter, 0);
        assert!(world.should_loop);

        let underflow = NativeRuntime::prize_collected_world_step_loc_FEFE(arc, 0x0100, 0x0020, 0);
        assert_eq!(underflow.next_loop_counter, u16::MAX);
        assert!(!underflow.should_loop);

        let clock = NativeRuntime::prize_clock_collected_world_step_loc_FF92(
            0x0010_0000,
            0x0008_0000,
            0x0001_8000,
            -0x0000_8000,
            0x0100,
            0x0020,
            1,
        );
        assert_eq!(clock.arc_step, arc);
        assert_eq!(clock.world_step, world);
    }

    #[test]
    fn clock_and_continue_helpers_match_asm_loc_10792_through_loc_10838() {
        // ASM loc_10792..loc_10838: the first clock plays the looping sound,
        // countdowns swap animation on `$012C` and delete on zero, the fall
        // gate is strict on both bottom and collection, and the continue exit
        // always plays sound `#$6A` before deleting.
        assert_eq!(
            NativeRuntime::prize_clock_sound_refcount_loc_10792(0),
            PrizeClockSoundRefcountLoc10792 {
                should_play_sound: true,
                next_refcount: 1,
            }
        );
        assert_eq!(
            NativeRuntime::prize_clock_sound_refcount_loc_10792(3),
            PrizeClockSoundRefcountLoc10792 {
                should_play_sound: false,
                next_refcount: 4,
            }
        );

        assert_eq!(
            NativeRuntime::prize_clock_countdown_loc_10796(0x012D),
            PrizeClockCountdownLoc10796 {
                next_remaining: 0x012C,
                should_delete: false,
                should_swap_animation: true,
            }
        );
        assert_eq!(
            NativeRuntime::prize_clock_countdown_loc_10796(1),
            PrizeClockCountdownLoc10796 {
                next_remaining: 0,
                should_delete: true,
                should_swap_animation: false,
            }
        );

        assert_eq!(
            NativeRuntime::prize_clock_fall_gate_loc_107B0(0x00EF, 0x00F0, false),
            PrizeClockFallGateLoc107B0 {
                stays_active: true,
                hit_level_bottom: false,
                collided_with_player: false,
            }
        );
        assert_eq!(
            NativeRuntime::prize_clock_fall_gate_loc_107B0(0x00F0, 0x00F0, false),
            PrizeClockFallGateLoc107B0 {
                stays_active: false,
                hit_level_bottom: true,
                collided_with_player: false,
            }
        );
        assert_eq!(
            NativeRuntime::prize_clock_fall_gate_loc_107B0(0x00EF, 0x00F0, true),
            PrizeClockFallGateLoc107B0 {
                stays_active: false,
                hit_level_bottom: false,
                collided_with_player: true,
            }
        );

        assert_eq!(
            NativeRuntime::prize_clock_sound_cleanup_loc_107E0(1),
            PrizeClockSoundCleanupLoc107E0 {
                next_refcount: 0,
                should_play_stop_sound: true,
                should_delete_current_object: true,
            }
        );
        assert_eq!(
            NativeRuntime::prize_clock_sound_cleanup_loc_107E0(3),
            PrizeClockSoundCleanupLoc107E0 {
                next_refcount: 2,
                should_play_stop_sound: false,
                should_delete_current_object: true,
            }
        );

        assert_eq!(
            NativeRuntime::prize_continue_countdown_loc_1080A(0x012D),
            PrizeContinueCountdownLoc1080A {
                next_remaining: 0x012C,
                should_delete: false,
                should_swap_animation: true,
            }
        );
        assert_eq!(
            NativeRuntime::prize_continue_countdown_loc_1080A(1),
            PrizeContinueCountdownLoc1080A {
                next_remaining: 0,
                should_delete: true,
                should_swap_animation: false,
            }
        );
        assert_eq!(
            NativeRuntime::prize_continue_delete_loc_10838(),
            PrizeContinueDeleteLoc10838 {
                sound_effect_id: 0x6A,
                preserves_d0_across_sound_call: true,
                should_delete_current_object: true,
            }
        );
    }

    #[test]
    fn floating_prize_fall_helpers_match_asm_loc_103de_through_loc_10c50() {
        // ASM loc_103DE/loc_10402/loc_10C50: falling adds `#$1388` gravity,
        // snaps the integer row with `andi.w #$FFF0`, and only on a snapped-row
        // change clears the first-row grace and advances the layout pointer.
        let steady = NativeRuntime::floating_prize_fall_step_loc_10C50(
            0x0030,
            0x0030_F000,
            0x0000_1000,
            true,
            7,
            0x0080,
            0x0040,
        );
        assert_eq!(steady.next_y_velocity, 0x0000_2388);
        assert_eq!(steady.next_y_pos, 0x0031_1388);
        assert_eq!(steady.next_aligned_y, 0x0030);
        assert!(steady.next_first_row_grace);
        assert_eq!(steady.next_level_row, 7);
        assert_eq!(steady.next_level_layout_offset, 0x0080);

        let crossed = NativeRuntime::floating_prize_fall_step_loc_10C50(
            0x0030,
            0x003F_F000,
            0x0000_1000,
            true,
            7,
            0x0080,
            0x0040,
        );
        assert_eq!(crossed.next_y_velocity, 0x0000_2388);
        assert_eq!(crossed.next_y_pos, 0x0040_1388);
        assert_eq!(crossed.next_aligned_y, 0x0040);
        assert!(!crossed.next_first_row_grace);
        assert_eq!(crossed.next_level_row, 8);
        assert_eq!(crossed.next_level_layout_offset, 0x00C0);
        assert_eq!(
            NativeRuntime::floating_prize_fall_loc_103DE(
                0x0030,
                0x003F_F000,
                0x0000_1000,
                true,
                7,
                0x0080,
                0x0040,
            ),
            crossed
        );
        assert_eq!(
            NativeRuntime::floating_prize_row_sync_loc_10402(
                0x0030,
                0x003F_F000,
                0x0000_1000,
                true,
                7,
                0x0080,
                0x0040,
            ),
            crossed
        );
    }

    #[test]
    fn floating_prize_metadata_matches_asm_sub_10c78_through_sub_10cec() {
        // ASM sub_10C78/sub_10CB0/sub_10CEC: diamonds use the shorter `$01F4`
        // timer, all non-diamond falling prizes use `$0320`, only helmet
        // prizes take the cleanup path, and that cleanup effect reuses the
        // deleted helmet's integer x with `y-8`.
        assert_eq!(
            NativeRuntime::floating_item_lifetime(terrain::PrizeContent::Diamond),
            0x01F4
        );
        assert_eq!(
            NativeRuntime::floating_item_lifetime(terrain::PrizeContent::Ankh),
            0x0320
        );
        assert_eq!(
            NativeRuntime::floating_item_lifetime(terrain::PrizeContent::Skycutter),
            0x0320
        );

        assert!(NativeRuntime::is_helmet_prize(
            terrain::PrizeContent::Skycutter
        ));
        assert!(NativeRuntime::is_helmet_prize(
            terrain::PrizeContent::Micromax
        ));
        assert!(!NativeRuntime::is_helmet_prize(
            terrain::PrizeContent::Diamond
        ));
        assert!(!NativeRuntime::is_helmet_prize(terrain::PrizeContent::Ankh));
        assert!(!NativeRuntime::is_helmet_prize(
            terrain::PrizeContent::Continue
        ));

        let cleanup = NativeRuntime::helmet_cleanup_effect_loc_10cec(
            terrain::PrizeContent::Skycutter,
            0x0123_0000,
            0x0042_0000,
        )
        .expect("helmet prize should spawn cleanup effect");
        assert_eq!(cleanup.x_px, 0x0123);
        assert_eq!(cleanup.y_px, 0x003A);
        assert_eq!(cleanup.remaining, 8);
        assert!(
            NativeRuntime::helmet_cleanup_effect_loc_10cec(
                terrain::PrizeContent::Ankh,
                0x0123_0000,
                0x0042_0000,
            )
            .is_none()
        );
    }

    #[test]
    fn helmet_prize_cleanup_sub_10cec_removes_existing_helmets_and_spawns_effects() {
        // ASM sub_10CEC/loc_10CF4 (kid.asm:22620-22644): the new helmet
        // initialiser walks existing objects, deletes any prior helmet prize
        // in the `$6C..$8C` content range, and seeds sub_10D3E at x/y-8.
        let mut runtime = runtime_for_block_tests();
        runtime.floating_items = vec![
            FloatingItem {
                x_pos: 0x0020_0000,
                y_pos: 0x0030_0000,
                y_velocity: 0,
                remaining: 0x0320,
                spawn_delay: 0,
                level_col: 2,
                level_row: 3,
                first_row_grace: false,
                content: terrain::PrizeContent::Skycutter,
                collected: false,
            },
            FloatingItem {
                x_pos: 0x0040_0000,
                y_pos: 0x0050_0000,
                y_velocity: 0,
                remaining: 0x0320,
                spawn_delay: 0,
                level_col: 4,
                level_row: 5,
                first_row_grace: false,
                content: terrain::PrizeContent::Ankh,
                collected: false,
            },
            FloatingItem {
                x_pos: 0x0060_0000,
                y_pos: 0x0070_0000,
                y_velocity: 0,
                remaining: 0x0320,
                spawn_delay: 0,
                level_col: 6,
                level_row: 7,
                first_row_grace: false,
                content: terrain::PrizeContent::Micromax,
                collected: false,
            },
        ];

        runtime.clear_existing_helmet_prizes_for_sub_10cec();

        assert_eq!(runtime.floating_items.len(), 1);
        assert_eq!(
            runtime.floating_items[0].content,
            terrain::PrizeContent::Ankh
        );
        assert_eq!(
            runtime
                .block_destroy_effects
                .iter()
                .map(|effect| (effect.x_px, effect.y_px, effect.remaining))
                .collect::<Vec<_>>(),
            vec![(0x0020, 0x0028, 8), (0x0060, 0x0068, 8)]
        );
    }

    #[test]
    fn floating_prize_graphics_specs_match_asm_prize_initialisers() {
        // ASM PrizeDiamond_Init and PrizeHelmet* initialisers choose the
        // content-specific sprite scripts before the shared fall/collision
        // loops run; #136's renderer fix should consume this table instead of
        // placeholder colours.
        assert_eq!(
            floating_prize_graphics_spec(terrain::PrizeContent::Diamond),
            Some(FloatingPrizeGraphicsSpec {
                object_meta: 0x0064,
                palette_line: 0,
                animation_script_addr: 0x0001_0D6E,
            })
        );

        let helmet_specs = [
            (
                terrain::PrizeContent::Skycutter,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x006C,
                    palette_line: 3,
                    animation_script_addr: 0x0001_06DC,
                },
            ),
            (
                terrain::PrizeContent::Cyclone,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x0070,
                    palette_line: 3,
                    animation_script_addr: 0x0001_06E8,
                },
            ),
            (
                terrain::PrizeContent::RedStealth,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x0074,
                    palette_line: 3,
                    animation_script_addr: 0x0001_06F4,
                },
            ),
            (
                terrain::PrizeContent::Eyeclops,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x0078,
                    palette_line: 3,
                    animation_script_addr: 0x0001_06EE,
                },
            ),
            (
                terrain::PrizeContent::Juggernaut,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x007C,
                    palette_line: 3,
                    animation_script_addr: 0x0001_06D6,
                },
            ),
            (
                terrain::PrizeContent::IronKnight,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x0080,
                    palette_line: 3,
                    animation_script_addr: 0x0001_06FA,
                },
            ),
            (
                terrain::PrizeContent::Berzerker,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x0084,
                    palette_line: 3,
                    animation_script_addr: 0x0001_06E2,
                },
            ),
            (
                terrain::PrizeContent::Maniaxe,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x0088,
                    palette_line: 3,
                    animation_script_addr: 0x0001_06D0,
                },
            ),
            (
                terrain::PrizeContent::Micromax,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x008C,
                    palette_line: 3,
                    animation_script_addr: 0x0001_0700,
                },
            ),
        ];
        for (content, expected) in helmet_specs {
            assert_eq!(floating_prize_graphics_spec(content), Some(expected));
        }

        for (content, expected) in [
            (
                terrain::PrizeContent::Ankh,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x0090,
                    palette_line: 0,
                    animation_script_addr: 0x0001_0C08,
                },
            ),
            (
                terrain::PrizeContent::Clock,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x0094,
                    palette_line: 0,
                    animation_script_addr: 0x0001_0C18,
                },
            ),
            (
                terrain::PrizeContent::Continue,
                FloatingPrizeGraphicsSpec {
                    object_meta: 0x0098,
                    palette_line: 0,
                    animation_script_addr: 0x0001_0C28,
                },
            ),
        ] {
            assert_eq!(floating_prize_graphics_spec(content), Some(expected));
        }
        assert_eq!(
            floating_prize_graphics_spec(terrain::PrizeContent::TenDiamonds),
            None
        );
    }

    #[test]
    fn iron_knight_block_breaker_latch_matches_asm_sub_af7a() {
        // ASM sub_AF7A/off_AFD8: Iron Knight sets the block-breaker latch on
        // the first qualifying downward hit, dispatches prize blocks through
        // loc_1002E, then returns handled with y_vel `$FFFF8000`.
        assert_eq!(
            NativeRuntime::iron_knight_block_breaker_collision_sub_AF7A(
                false,
                false,
                0x0004_8000,
                true,
                terrain::BlockType::Prize,
            ),
            IronKnightBlockBreakerCollisionSubAF7A {
                next_block_breaker: false,
                handled_collision: false,
                next_y_velocity: None,
                effect: None,
            },
            "ASM sub_AF7A bne.w loc_AFD0 rejects non-Iron-Knight forms before velocity/block checks"
        );
        assert_eq!(
            NativeRuntime::iron_knight_block_breaker_collision_sub_AF7A(
                true,
                false,
                0x0004_7FFF,
                true,
                terrain::BlockType::Prize,
            ),
            IronKnightBlockBreakerCollisionSubAF7A {
                next_block_breaker: false,
                handled_collision: false,
                next_y_velocity: None,
                effect: None,
            },
            "ASM sub_AF7A blt.w loc_AFD0 rejects unlatched Iron Knight hits below `$48000`"
        );
        let first_hit = NativeRuntime::iron_knight_block_breaker_collision_sub_AF7A(
            true,
            false,
            0x0004_8000,
            true,
            terrain::BlockType::Prize,
        );
        assert_eq!(
            first_hit,
            IronKnightBlockBreakerCollisionSubAF7A {
                next_block_breaker: true,
                handled_collision: true,
                next_y_velocity: Some(-0x0000_8000),
                effect: Some(IronKnightBlockEffectSubAF7A::Prize),
            }
        );

        // ASM loc_AF96: once `Iron_Knight_block_breaker` is set, the next
        // dispatch skips the `$48000` velocity threshold instead of falling
        // back into the ordinary block-bounce path.
        let latched_hit = NativeRuntime::iron_knight_block_breaker_collision_sub_AF7A(
            true,
            true,
            0,
            true,
            terrain::BlockType::Smash,
        );
        assert_eq!(
            latched_hit,
            IronKnightBlockBreakerCollisionSubAF7A {
                next_block_breaker: true,
                handled_collision: true,
                next_y_velocity: Some(-0x0000_8000),
                effect: Some(IronKnightBlockEffectSubAF7A::Smash),
            }
        );
    }

    #[test]
    fn iron_knight_block_breaker_ignores_non_break_dispatches_like_off_afd8() {
        // ASM off_AFD8 routes SolidGhost and Teleporter to sub_AFD4, which
        // unwinds to loc_AFCA and returns unhandled; Metal is the only
        // non-breaking slot that still returns handled via loc_110D0.
        assert_eq!(
            NativeRuntime::iron_knight_block_breaker_collision_sub_AF7A(
                true,
                false,
                0x0004_8000,
                true,
                terrain::BlockType::SolidGhost,
            ),
            IronKnightBlockBreakerCollisionSubAF7A {
                next_block_breaker: true,
                handled_collision: false,
                next_y_velocity: None,
                effect: None,
            }
        );

        assert_eq!(
            NativeRuntime::iron_knight_block_breaker_collision_sub_AF7A(
                true,
                false,
                0x0004_8000,
                true,
                terrain::BlockType::Metal,
            ),
            IronKnightBlockBreakerCollisionSubAF7A {
                next_block_breaker: true,
                handled_collision: true,
                next_y_velocity: Some(-0x0000_8000),
                effect: Some(IronKnightBlockEffectSubAF7A::Metal),
            }
        );
    }

    #[test]
    fn helmet_cleanup_animation_matches_asm_sub_10d3e() {
        // ASM sub_10D3E (kid.asm:22653): the spawned cleanup worker always
        // uses gfx slot `$02000000`, clears VRAM tile zero, shows the sprite,
        // copies the cached word coordinates, runs `stru_102DE`, and deletes
        // itself immediately after the animation finishes.
        assert_eq!(
            NativeRuntime::helmet_cleanup_animation_sub_10D3E(0x0123, 0x003A),
            HelmetCleanupAnimationSub10D3E {
                allocation_spec: 0x0200_0000,
                vram_tile: 0,
                display_enabled: true,
                x_pos: 0x0123,
                y_pos: 0x003A,
                animation_script_addr: 0x0001_02DE,
                deletes_after_animation: true,
            }
        );
    }

    #[test]
    fn one_time_prize_helpers_match_asm_loc_10856_through_loc_1089a() {
        // ASM loc_10856..loc_1089A: negative headers clear the full forty-entry
        // table, scans start at `2(a0)` with `d0 = #$27`, matches require both
        // layout and level, DBF underflows to stop, and the header points at
        // the next insertion slot rather than the slot just written.
        let mut entries = [0_u32; 0x28];
        entries[5] = 0x1111_2222;

        let first = NativeRuntime::one_time_prize_init_loc_10856(-1, entries);
        assert!(first.was_first_prize_initialisation);
        assert_eq!(first.next_write_index, 0);
        assert_eq!(first.entries, [0; 0x28]);

        let resumed = NativeRuntime::one_time_prize_init_loc_10856(5, entries);
        assert!(!resumed.was_first_prize_initialisation);
        assert_eq!(resumed.next_write_index, 5);
        assert_eq!(resumed.entries[5], 0x1111_2222);

        // ASM loc_10856: `bpl.s loc_1086A` treats zero as an already
        // initialised header, so slot 0 resumes without clearing the table.
        let zero_header = NativeRuntime::one_time_prize_init_loc_10856(0, entries);
        assert!(!zero_header.was_first_prize_initialisation);
        assert_eq!(zero_header.next_write_index, 0);
        assert_eq!(zero_header.entries[5], 0x1111_2222);

        assert_eq!(
            NativeRuntime::one_time_prize_scan_start_loc_1086A(0x3456),
            OneTimePrizeScanStartLoc1086A {
                remaining_entry_checks: 0x27,
                prize_layout_addr: 0x3456,
                entry_byte_offset_from_table_base: 2,
            }
        );
        assert_eq!(
            NativeRuntime::one_time_prize_entry_check_loc_10876(0x1234_0005, 0x1234, 0x0005),
            OneTimePrizeEntryCheckLoc10876 {
                entry_layout_addr: 0x1234,
                entry_level_id: 0x0005,
                layout_addr_matches: true,
                level_id_matches: true,
                should_init_prize_diamond: true,
            }
        );
        assert_eq!(
            NativeRuntime::one_time_prize_entry_check_loc_10876(0x1234_0005, 0x1234, 0x0006),
            OneTimePrizeEntryCheckLoc10876 {
                entry_layout_addr: 0x1234,
                entry_level_id: 0x0005,
                layout_addr_matches: true,
                level_id_matches: false,
                should_init_prize_diamond: false,
            }
        );
        assert_eq!(
            NativeRuntime::one_time_prize_scan_step_loc_10886(0, 2),
            OneTimePrizeScanStepLoc10886 {
                next_remaining_entry_checks: u16::MAX,
                next_entry_byte_offset_from_table_base: 6,
                should_continue_scan: false,
            }
        );

        let stored = NativeRuntime::one_time_prize_store_loc_1089A(0x27, [0; 0x28], 0xBEEF, 0x0012);
        assert_eq!(stored.next_write_index, 0);
        assert_eq!(stored.entries[0x27], 0xBEEF_0012);
    }

    #[test]
    fn prize_10_diamonds_helpers_match_asm_loc_108ee_through_loc_109c0() {
        // ASM loc_108E8..loc_109C0: the burst table only spawns on the listed
        // frames, each spawned worker copies the owner position and seeds
        // `(word << 16) >> 3` velocities, and later worker steps reproject the
        // cached camera-relative FP words through the camera integers.
        assert_eq!(
            NativeRuntime::prize_10_diamonds_init_loc_108E8(),
            Prize10DiamondsInitLoc108E8 {
                cleared_worker_slots: [0; 10],
            }
        );

        // ASM loc_108EE: every negative `byte_108AC` entry branches to
        // `loc_10960`; every non-negative entry indexes `unk_10A52`.
        let expected_bursts = [
            (0, 0, -20, -36),
            (1, 1, -20, -27),
            (2, 2, -20, -18),
            (3, 3, -20, -9),
            (24, 4, 6, -36),
            (25, 5, -4, -28),
            (26, 6, 16, -28),
            (27, 7, -4, -16),
            (28, 8, 16, -16),
            (29, 9, 6, -8),
        ];
        for (frame, table_index, x_velocity_word, y_velocity_word) in expected_bursts {
            let spawned = NativeRuntime::prize_10_diamonds_spawn_loc_108EE(frame, 0x0120, 0x00B0);
            assert_eq!(
                spawned.burst_table_index,
                Some(table_index),
                "frame {frame}"
            );
            assert!(spawned.should_spawn_worker, "frame {frame}");
            assert_eq!(spawned.x_pos, Some(0x0120), "frame {frame}");
            assert_eq!(spawned.y_pos, Some(0x00B0), "frame {frame}");
            assert_eq!(
                spawned.x_velocity,
                Some(x_velocity_word << 13),
                "frame {frame}"
            );
            assert_eq!(
                spawned.y_velocity,
                Some(y_velocity_word << 13),
                "frame {frame}"
            );
            assert_eq!(spawned.lifetime_frames, Some(0x0078), "frame {frame}");
        }

        for frame in 4..24 {
            let skipped = NativeRuntime::prize_10_diamonds_spawn_loc_108EE(frame, 0x0120, 0x00B0);
            assert_eq!(skipped.burst_table_index, None, "frame {frame}");
            assert!(!skipped.should_spawn_worker, "frame {frame}");
            assert_eq!(skipped.x_velocity, None, "frame {frame}");
            assert_eq!(skipped.y_velocity, None, "frame {frame}");
        }

        let first = NativeRuntime::prize_10_diamonds_spawn_loc_108EE(0, 0x0120, 0x00B0);
        assert_eq!(first.burst_table_index, Some(0));
        assert!(first.should_hibernate_one_frame);
        assert!(first.should_scan_workers);
        assert!(first.should_spawn_worker);
        assert_eq!(first.allocation_spec, Some(0x00FE_0000));
        assert_eq!(first.sound_effect_label, Some("sfx_Diamond_prize"));
        assert_eq!(first.priority, Some(1));
        assert_eq!(first.display_enabled, Some(true));
        assert_eq!(
            first.animation_script_label,
            Some("LnkTo_unk_E1046-Data_Index")
        );
        assert_eq!(first.x_pos, Some(0x0120));
        assert_eq!(first.y_pos, Some(0x00B0));
        assert_eq!(first.x_velocity, Some(-20_i32 << 13));
        assert_eq!(first.y_velocity, Some(-36_i32 << 13));
        assert_eq!(first.is_moved, Some(true));
        assert_eq!(first.lifetime_frames, Some(0x0078));

        let past_end = NativeRuntime::prize_10_diamonds_spawn_loc_108EE(30, 0x0120, 0x00B0);
        assert!(!past_end.should_spawn_worker);
        assert_eq!(past_end.burst_table_index, None);

        assert_eq!(
            NativeRuntime::prize_10_diamonds_worker_scan_start_loc_10960(),
            Prize10DiamondsWorkerScanStartLoc10960 {
                first_worker_slot_byte_offset: 0,
                worker_slot_stride_bytes: 4,
                worker_table_len_bytes: 0x28,
            }
        );
        assert_eq!(
            NativeRuntime::prize_10_diamonds_worker_countdown_gate_loc_10984(0x0070),
            Prize10DiamondsWorkerCountdownGateLoc10984 {
                should_clear_display_enabled: true,
            }
        );
        assert_eq!(
            NativeRuntime::prize_10_diamonds_worker_countdown_gate_loc_10984(0x006F),
            Prize10DiamondsWorkerCountdownGateLoc10984 {
                should_clear_display_enabled: false,
            }
        );
        assert_eq!(
            NativeRuntime::prize_10_diamonds_worker_init_loc_10988(0x0100_0000, 0x0010_0000, 0, 0),
            Prize10DiamondsWorkerInitLoc10988 {
                camera_relative_x_pos: 0x0100_0000,
                camera_relative_y_pos: 0x0010_0000,
                x_velocity: 0x0001_3000,
                y_velocity: 0x0001_0000,
            }
        );
        assert_eq!(
            NativeRuntime::prize_10_diamonds_worker_step_loc_109C0(
                0x0001_0000,
                -0x0000_8000,
                0x0000_8000,
                0x0001_0000,
                0x0020,
                0x0010,
            ),
            Prize10DiamondsWorkerStepLoc109C0 {
                next_camera_relative_x_pos: 0x0001_8000,
                next_camera_relative_y_pos: 0x0000_8000,
                next_x_pos: 0x0021,
                next_y_pos: 0x0010,
            }
        );
    }

    #[test]
    fn lift_platform_and_block_helpers_match_asm_sub_fd00_and_loc_113f4_through_loc_1141e() {
        // ASM sub_FD00/loc_FCBE/loc_113F4/loc_1141E: the vertical step is a
        // wrapping FP add, lift platforms only count changed integer rows, the
        // velocity flips from subtracting `$708` to adding `$7D0` after more
        // than `$32` moved rows, and the rise/fall workers use DBF tails.
        assert_eq!(
            NativeRuntime::lift_platform_vertical_step_sub_FD00(0xFFFF_0000, 0x0002_0000),
            0x0001_0000
        );

        let mut rising = LiftPlatform {
            x_pos: 0,
            y_pos: 0x0010_0000,
            y_velocity: 0,
            age: 0,
            last_integer_y: 0x0010,
            moved_y_steps: 0,
            width_px: 16,
            source_col: 0,
            source_row: 0,
            block_erased: false,
        };
        assert!(NativeRuntime::step_lift_platform_sub_FCA6(
            &mut rising,
            0x0020
        ));
        assert_eq!(rising.age, 1);
        assert_eq!(rising.y_pos, 0x0010_0000);
        assert_eq!(rising.last_integer_y, 0x0010);
        assert_eq!(rising.moved_y_steps, 0);
        assert_eq!(rising.y_velocity, -0x0708);

        let mut falling = LiftPlatform {
            x_pos: 0,
            y_pos: 0x0020_0000,
            y_velocity: 0x0100,
            age: 2,
            last_integer_y: 0x001F,
            moved_y_steps: 0x33,
            width_px: 16,
            source_col: 0,
            source_row: 0,
            block_erased: false,
        };
        assert!(!NativeRuntime::step_lift_platform_sub_FCA6(
            &mut falling,
            0x0020
        ));
        assert_eq!(falling.age, 3);
        assert_eq!(falling.last_integer_y, 0x0020);
        assert_eq!(falling.moved_y_steps, 0x34);
        assert_eq!(falling.y_velocity, 0x08D0);

        assert_eq!(
            NativeRuntime::lift_block_rise_step_loc_113F4(0x0040, 7),
            LiftBlockRiseStepLoc113F4 {
                next_y_pos: 0x003E,
                next_loop_counter: 6,
                should_hibernate_one_frame: true,
                should_repeat_loop: true,
            }
        );
        assert_eq!(
            NativeRuntime::lift_block_rise_step_loc_113F4(0x0040, 0),
            LiftBlockRiseStepLoc113F4 {
                next_y_pos: 0x003E,
                next_loop_counter: u16::MAX,
                should_hibernate_one_frame: true,
                should_repeat_loop: false,
            }
        );
        assert_eq!(
            NativeRuntime::lift_block_fall_step_loc_1141E(0x0050, 1),
            LiftBlockFallStepLoc1141E {
                next_y_pos: 0x004F,
                next_loop_counter: 0,
                should_hibernate_one_frame: true,
                should_delete_current_object: false,
                should_repeat_loop: true,
            }
        );
        assert_eq!(
            NativeRuntime::lift_block_fall_step_loc_1141E(0x0050, 0),
            LiftBlockFallStepLoc1141E {
                next_y_pos: 0x004F,
                next_loop_counter: u16::MAX,
                should_hibernate_one_frame: true,
                should_delete_current_object: true,
                should_repeat_loop: false,
            }
        );
    }

    #[test]
    fn shooter_headbutt_helpers_match_asm_loc_11448_through_loc_11474() {
        // ASM loc_11448..loc_11474: the left support probe skips column zero,
        // the centre gate restores the original column and suppresses the
        // struck-cell worker after any support spawn, and the centre worker
        // carries the struck cell as `(col,row)` with age zero.
        assert_eq!(
            NativeRuntime::shooter_left_support_probe_loc_11448(0, true),
            ShooterLeftSupportProbeLoc11448 {
                should_probe_support: false,
                support_col: None,
                next_block_col_for_right_probe: 1,
            }
        );
        assert_eq!(
            NativeRuntime::shooter_left_support_probe_loc_11448(3, true),
            ShooterLeftSupportProbeLoc11448 {
                should_probe_support: true,
                support_col: Some(2),
                next_block_col_for_right_probe: 4,
            }
        );
        assert_eq!(
            NativeRuntime::shooter_left_support_probe_loc_11448(3, false),
            ShooterLeftSupportProbeLoc11448 {
                should_probe_support: false,
                support_col: None,
                next_block_col_for_right_probe: 4,
            }
        );

        assert_eq!(
            NativeRuntime::shooter_center_worker_gate_loc_1146E(5, false),
            ShooterCenterWorkerGateLoc1146E {
                restored_block_col: 4,
                should_queue_center_worker: true,
            }
        );
        assert_eq!(
            NativeRuntime::shooter_center_worker_gate_loc_1146E(5, true),
            ShooterCenterWorkerGateLoc1146E {
                restored_block_col: 4,
                should_queue_center_worker: false,
            }
        );

        let worker = NativeRuntime::shooter_center_worker_loc_11474(6, 9);
        assert_eq!(worker.col, 6);
        assert_eq!(worker.row, 9);
        assert_eq!(worker.age, 0);
        assert_eq!(worker.kind, ShooterHeadbuttWorkerKind::Center);
    }

    #[test]
    fn shooter_collision_rects_match_asm_sub_f568_direction_windows() {
        // ASM sub_F568/loc_F5B0/loc_F62C/loc_F67E/loc_F6D2: a shooter that is
        // two blocks inside every expanded window edge emits two rectangles per
        // direction, and each direction bit keeps its own edge/window gate.
        let hitbox = CollisionRectSub1F52 {
            left_px: 64,
            right_px: 128,
            top_px: 64,
            bottom_px: 128,
        };
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(5, 5, 0x0F), (2, 5, 0x0F)]),
            vec![
                CollisionRectSub1F52 {
                    left_px: 82,
                    right_px: 94,
                    top_px: 67,
                    bottom_px: 79,
                },
                CollisionRectSub1F52 {
                    left_px: 85,
                    right_px: 91,
                    top_px: 71,
                    bottom_px: 79,
                },
                CollisionRectSub1F52 {
                    left_px: 96,
                    right_px: 104,
                    top_px: 81,
                    bottom_px: 93,
                },
                CollisionRectSub1F52 {
                    left_px: 96,
                    right_px: 108,
                    top_px: 84,
                    bottom_px: 90,
                },
                CollisionRectSub1F52 {
                    left_px: 82,
                    right_px: 94,
                    top_px: 95,
                    bottom_px: 103,
                },
                CollisionRectSub1F52 {
                    left_px: 85,
                    right_px: 91,
                    top_px: 95,
                    bottom_px: 107,
                },
                CollisionRectSub1F52 {
                    left_px: 72,
                    right_px: 80,
                    top_px: 81,
                    bottom_px: 93,
                },
                CollisionRectSub1F52 {
                    left_px: 68,
                    right_px: 80,
                    top_px: 84,
                    bottom_px: 90,
                },
            ]
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(5, 5, 0x02)]),
            vec![
                CollisionRectSub1F52 {
                    left_px: 96,
                    right_px: 104,
                    top_px: 81,
                    bottom_px: 93,
                },
                CollisionRectSub1F52 {
                    left_px: 96,
                    right_px: 108,
                    top_px: 84,
                    bottom_px: 90,
                },
            ]
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(5, 5, 0x01)]),
            vec![
                CollisionRectSub1F52 {
                    left_px: 82,
                    right_px: 94,
                    top_px: 67,
                    bottom_px: 79,
                },
                CollisionRectSub1F52 {
                    left_px: 85,
                    right_px: 91,
                    top_px: 71,
                    bottom_px: 79,
                },
            ]
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(5, 5, 0x04)]),
            vec![
                CollisionRectSub1F52 {
                    left_px: 82,
                    right_px: 94,
                    top_px: 95,
                    bottom_px: 103,
                },
                CollisionRectSub1F52 {
                    left_px: 85,
                    right_px: 91,
                    top_px: 95,
                    bottom_px: 107,
                },
            ]
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(5, 5, 0x08)]),
            vec![
                CollisionRectSub1F52 {
                    left_px: 72,
                    right_px: 80,
                    top_px: 81,
                    bottom_px: 93,
                },
                CollisionRectSub1F52 {
                    left_px: 68,
                    right_px: 80,
                    top_px: 84,
                    bottom_px: 90,
                },
            ]
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(3, 5, 0x0F), (5, 3, 0x0F)]),
            vec![
                CollisionRectSub1F52 {
                    left_px: 64,
                    right_px: 72,
                    top_px: 81,
                    bottom_px: 93,
                },
                CollisionRectSub1F52 {
                    left_px: 64,
                    right_px: 76,
                    top_px: 84,
                    bottom_px: 90,
                },
                CollisionRectSub1F52 {
                    left_px: 82,
                    right_px: 94,
                    top_px: 63,
                    bottom_px: 71,
                },
                CollisionRectSub1F52 {
                    left_px: 85,
                    right_px: 91,
                    top_px: 63,
                    bottom_px: 75,
                },
            ]
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(8, 5, 0x02)]),
            Vec::<CollisionRectSub1F52>::new()
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(5, 8, 0x04)]),
            Vec::<CollisionRectSub1F52>::new()
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(4, 5, 0x08)]),
            Vec::<CollisionRectSub1F52>::new()
        );
    }

    #[test]
    fn shooter_collision_rects_sub_f568_keeps_inclusive_outer_window_edges() {
        // ASM sub_F568 loc_F5B0 (kid.asm:20234): the outer block-window
        // rejects only `col > right_block` / `row > bottom_block`; equality
        // remains in-window and is filtered later only by direction-specific
        // edge gates.
        let hitbox = CollisionRectSub1F52 {
            left_px: 64,
            right_px: 128,
            top_px: 64,
            bottom_px: 128,
        };

        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(9, 5, 0x08)]),
            vec![
                CollisionRectSub1F52 {
                    left_px: 136,
                    right_px: 144,
                    top_px: 81,
                    bottom_px: 93,
                },
                CollisionRectSub1F52 {
                    left_px: 132,
                    right_px: 144,
                    top_px: 84,
                    bottom_px: 90,
                },
            ],
            "ASM sub_F568 should keep col == right_block in the outer window"
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(5, 9, 0x01)]),
            vec![
                CollisionRectSub1F52 {
                    left_px: 82,
                    right_px: 94,
                    top_px: 131,
                    bottom_px: 143,
                },
                CollisionRectSub1F52 {
                    left_px: 85,
                    right_px: 91,
                    top_px: 135,
                    bottom_px: 143,
                },
            ],
            "ASM sub_F568 should keep row == bottom_block in the outer window"
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(10, 5, 0x08)]),
            Vec::<CollisionRectSub1F52>::new(),
            "ASM sub_F568 should reject col > right_block before direction gates"
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(5, 10, 0x01)]),
            Vec::<CollisionRectSub1F52>::new(),
            "ASM sub_F568 should reject row > bottom_block before direction gates"
        );
    }

    #[test]
    fn shooter_collision_rects_sub_f568_right_edge_gate_needs_direction_bit_and_two_block_margin() {
        // ASM loc_F62C (kid.asm:20285): bit 1 reaches the right-facing
        // rectangles only after `lsr.w #1,d6`, and `d4 - d1 + 1` must stay
        // negative, which requires at least two columns of right-edge margin.
        let hitbox = CollisionRectSub1F52 {
            left_px: 64,
            right_px: 128,
            top_px: 64,
            bottom_px: 128,
        };

        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(7, 5, 0x02)]),
            vec![
                CollisionRectSub1F52 {
                    left_px: 128,
                    right_px: 136,
                    top_px: 81,
                    bottom_px: 93,
                },
                CollisionRectSub1F52 {
                    left_px: 128,
                    right_px: 140,
                    top_px: 84,
                    bottom_px: 90,
                },
            ],
            "ASM loc_F62C should emit when bit 1 is set and col < right_block - 1"
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(7, 4, 0x01)]),
            Vec::<CollisionRectSub1F52>::new(),
            "ASM loc_F62C must not treat bit 0 as the right-facing shooter bit"
        );
        assert_eq!(
            NativeRuntime::shooter_collision_rects_sub_F568(hitbox, &[(8, 5, 0x02)]),
            Vec::<CollisionRectSub1F52>::new(),
            "ASM loc_F62C should reject col >= right_block - 1"
        );
    }

    #[test]
    fn rubber_block_worker_lookup_reuses_only_the_same_layout_cell() {
        // ASM sub_10DE4 (kid.asm:22717): the worker is keyed by the exact
        // `(col,row)` layout cell stored in `$44/$46(a5)`, so sharing either
        // axis alone must not reuse an existing animation slot.
        assert!(
            NativeRuntime::rubber_block_animation_slot_matches_sub_10de4(0x12, 0x07, 0x12, 0x07)
        );
        assert!(
            !NativeRuntime::rubber_block_animation_slot_matches_sub_10de4(0x12, 0x07, 0x13, 0x07)
        );
        assert!(
            !NativeRuntime::rubber_block_animation_slot_matches_sub_10de4(0x12, 0x07, 0x12, 0x08)
        );
        assert!(
            !NativeRuntime::rubber_block_animation_slot_matches_sub_10de4(0x12, 0x07, 0x13, 0x08)
        );
    }

    #[test]
    fn rubber_block_wait_gate_only_releases_when_the_hibernate_count_reaches_zero() {
        // ASM loc_10E0A (kid.asm:22733-22762): the worker hibernates for the
        // programmed count, then performs the next `sub_11530` write exactly
        // on the zero-crossing frame.
        assert_eq!(
            NativeRuntime::rubber_block_animation_wait_state_sub_10de4(2),
            (1, true)
        );
        assert_eq!(
            NativeRuntime::rubber_block_animation_wait_state_sub_10de4(1),
            (0, false)
        );
        assert_eq!(
            NativeRuntime::rubber_block_animation_wait_state_sub_10de4(0),
            (0, false)
        );
    }

    #[test]
    fn rubber_block_sequence_keeps_running_until_the_final_ninth_write() {
        // ASM sub_10DE4 (kid.asm:22733-22766): the `(1,3)` step is followed by
        // a final `(0,3)` write, but the last `(0,0)` table entry immediately
        // falls into `Delete_CurrentObject` instead of keeping the worker.
        assert_eq!(
            NativeRuntime::rubber_block_animation_next_step_sub_10de4(6),
            Some((0, 3, true))
        );
        assert_eq!(
            NativeRuntime::rubber_block_animation_next_step_sub_10de4(7),
            Some((0, 0, false))
        );
        assert_eq!(
            NativeRuntime::rubber_block_animation_next_step_sub_10de4(8),
            None
        );
    }

    #[test]
    fn rubber_block_base_skin_matches_asm_sub_10de4_skin_split() {
        // ASM sub_10DE4 / loc_10E06 (kid.asm:22717-22730): only rubber blocks
        // animate here, skins `$0C/$0D` collapse to base `$0C`, and the rest
        // of the rubber family seeds the `$0E/$0F` pair.
        assert_eq!(
            NativeRuntime::rubber_block_base_skin_parts_for_sub_10de4(
                terrain::BlockType::Rubber,
                0x0C,
            ),
            Some(0x0C)
        );
        assert_eq!(
            NativeRuntime::rubber_block_base_skin_parts_for_sub_10de4(
                terrain::BlockType::Rubber,
                0x0D,
            ),
            Some(0x0C)
        );
        assert_eq!(
            NativeRuntime::rubber_block_base_skin_parts_for_sub_10de4(
                terrain::BlockType::Rubber,
                0x0E,
            ),
            Some(0x0E)
        );
        assert_eq!(
            NativeRuntime::rubber_block_base_skin_parts_for_sub_10de4(
                terrain::BlockType::Evanescent,
                0x0C,
            ),
            None
        );
        assert_eq!(
            NativeRuntime::rubber_block_base_skin_for_sub_10de4(terrain::Cell::from_be_bytes(
                0xE6, 0x0D,
            )),
            Some(0x0C),
            "ASM sub_10DE4 reads the live cell skin byte before collapsing `$0D` to base `$0C`"
        );
        assert_eq!(
            NativeRuntime::rubber_block_base_skin_for_sub_10de4(terrain::Cell::from_be_bytes(
                0xE6, 0x0E,
            )),
            Some(0x0E),
            "ASM sub_10DE4 keeps non-$0C/$0D rubber skins on the `$0E/$0F` pair"
        );
        assert_eq!(
            NativeRuntime::rubber_block_base_skin_for_sub_10de4(terrain::Cell::from_be_bytes(
                0xE0, 0x0D,
            )),
            None,
            "ASM sub_10DE4 is only reached for rubber cells, not other block families"
        );
    }

    #[test]
    fn rubber_block_runtime_writes_asm_sub_10de4_immediate_and_delayed_skins() {
        // ASM sub_10DE4 / loc_10E0A (kid.asm:22717-22766): the worker reads
        // the live layout skin, immediately writes `base+1`, hibernates two
        // frames, then writes the base skin back into the same layout cell.
        let mut runtime = runtime_for_block_tests();
        let mut grid = terrain::BlockGrid::from_level_layout(&[0; 18], 3, 3);
        grid.set_cell_at(1, 1, terrain::Cell::from_be_bytes(0xE6, 0x0C));
        grid.set_cell_at(2, 1, terrain::Cell::from_be_bytes(0xE0, 0x0D));
        runtime.block_grid = Some(grid);

        runtime.start_rubber_block_animation_sub_10de4(1, 1);
        assert_eq!(
            runtime.block_grid.as_ref().unwrap().cell_at(1, 1),
            terrain::Cell::from_be_bytes(0xE6, 0x0D),
            "ASM loc_10E0A should write the jiggled rubber frame immediately"
        );
        assert_eq!(
            runtime.block_grid.as_ref().unwrap().cell_at(2, 1),
            terrain::Cell::from_be_bytes(0xE0, 0x0D),
            "ASM sub_10DE4 must leave non-rubber neighbour cells untouched"
        );
        assert_eq!(runtime.block_state.rubber_block_animations.len(), 1);

        runtime.step_rubber_block_animations_sub_10de4();
        assert_eq!(
            runtime.block_grid.as_ref().unwrap().cell_at(1, 1),
            terrain::Cell::from_be_bytes(0xE6, 0x0D),
            "ASM hibernate count 2 should keep waiting on the first native step"
        );

        runtime.step_rubber_block_animations_sub_10de4();
        assert_eq!(
            runtime.block_grid.as_ref().unwrap().cell_at(1, 1),
            terrain::Cell::from_be_bytes(0xE6, 0x0C),
            "ASM sub_10DE4 should write the base rubber frame after two hibernated frames"
        );
        assert_eq!(runtime.block_state.rubber_block_animations.len(), 1);
        assert_eq!(runtime.block_state.rubber_block_animations[0].phase, 1);
        assert_eq!(
            runtime.block_state.rubber_block_animations[0].frames_until_next_write,
            1
        );
    }

    #[test]
    fn block_destroy_effect_origins_match_asm_sub_10eda_and_sub_10f9c() {
        // ASM sub_10EDA/sub_10F9C (kid.asm:22817-22908): both block destroy
        // workers use the same direction branches for the 4px effect nudge.
        let cases = [
            (
                NativeRuntime::BLOCK_DESTROY_DIRECTION_BELOW,
                (0x0030, 0x0044),
            ),
            (
                NativeRuntime::BLOCK_DESTROY_DIRECTION_LEFT,
                (0x002C, 0x0040),
            ),
            (
                NativeRuntime::BLOCK_DESTROY_DIRECTION_ABOVE,
                (0x0030, 0x003C),
            ),
            (
                NativeRuntime::BLOCK_DESTROY_DIRECTION_RIGHT,
                (0x0034, 0x0040),
            ),
        ];
        for (direction, expected) in cases {
            assert_eq!(
                NativeRuntime::smash_block_destroy_effect_origin_sub_10EDA(
                    direction, 0x0030, 0x0040,
                ),
                expected
            );
            assert_eq!(
                NativeRuntime::ice_block_destroy_effect_origin_sub_10F9C(direction, 0x0030, 0x0040,),
                expected
            );
        }
    }
}
