use crate::ModeLevelPrizeTailListWork;

/// Motorola 68000 execution time, excluding VDP bus stalls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M68kCycleCount(u32);

impl M68kCycleCount {
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Cycles from the finished-animation wake at ASM loc_1066 through the next
/// `Execute_Objects` loc_E1E dispatch.
pub fn prize_tail_finish_cycles_sub_1007a(work: ModeLevelPrizeTailListWork) -> M68kCycleCount {
    // ASM sub_1007A (kid.asm:21407-21435) creates the flash and prize through
    // sub_E02, then Delete_CurrentObject walks the Object and GfxObject lists.
    // Each visited node in loc_DE0, loc_FD4, loc_EA2, loc_10BA, and loc_1106
    // executes one 54-cycle loop body. The fixed path costs 5,268 cycles.
    const FIXED_PATH_CYCLES: u32 = 5_268;
    const LIST_NODE_CYCLES: u32 = 54;

    let visited_nodes = [
        work.flash_object_insert_rank_loc_de0,
        work.flash_gfx_insert_rank_loc_fd4,
        work.prize_object_insert_rank_loc_de0,
        work.prize_gfx_insert_rank_loc_fd4,
        work.tail_object_delete_rank_loc_ea2,
        work.tail_gfx_delete_rank_loc_10ba,
        work.tail_gfx_category4_delete_rank_loc_1106,
    ]
    .into_iter()
    .map(u32::from)
    .sum::<u32>();

    M68kCycleCount(FIXED_PATH_CYCLES + LIST_NODE_CYCLES * visited_nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prize_tail_finish_cycles_follow_all_five_asm_list_walks_sub_1007a() {
        let frame_1697 = ModeLevelPrizeTailListWork {
            flash_object_insert_rank_loc_de0: 5,
            flash_gfx_insert_rank_loc_fd4: 15,
            prize_object_insert_rank_loc_de0: 1,
            prize_gfx_insert_rank_loc_fd4: 16,
            tail_object_delete_rank_loc_ea2: 14,
            tail_gfx_delete_rank_loc_10ba: 5,
            tail_gfx_category4_delete_rank_loc_1106: 11,
        };
        let frame_1703 = ModeLevelPrizeTailListWork {
            flash_object_insert_rank_loc_de0: 6,
            flash_gfx_insert_rank_loc_fd4: 16,
            prize_object_insert_rank_loc_de0: 1,
            prize_gfx_insert_rank_loc_fd4: 17,
            tail_object_delete_rank_loc_ea2: 15,
            tail_gfx_delete_rank_loc_10ba: 5,
            tail_gfx_category4_delete_rank_loc_1106: 12,
        };

        assert_eq!(
            prize_tail_finish_cycles_sub_1007a(frame_1697).get(),
            8_886,
            "bounded oracle: frame 1697 loc_1066 -> next loc_E1E"
        );
        assert_eq!(
            prize_tail_finish_cycles_sub_1007a(frame_1703).get(),
            9_156,
            "bounded oracle: frame 1703 loc_1066 -> next loc_E1E"
        );
    }
}
