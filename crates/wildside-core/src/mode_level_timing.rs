use crate::{
    ModeLevelExecuteObjectsWork, ModeLevelFloatingPrizeFallPathLoc103B2,
    ModeLevelPrizeAnimationWake, ModeLevelPrizeBlockTailState, ModeLevelPrizeBlockWobbleWake,
    ModeLevelPrizeObjectExecutionSubE12, ModeLevelPrizeTailListWork,
};

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

/// Nominal 68000 cycles for one typed prize callback, measured from its
/// `Execute_Objects` loc_E1E list read through the following loc_E1E read.
///
/// `None` preserves branch uncertainty. VDP bus stalls are deliberately
/// excluded and belong to the raster scheduler.
pub fn prize_object_cycles_sub_e12(
    execute: &ModeLevelExecuteObjectsWork,
    callback: ModeLevelPrizeObjectExecutionSubE12,
) -> Option<M68kCycleCount> {
    // ASM loc_E1E -> loc_E3E -> callback costs 224 cycles on the runnable
    // path. A one-frame hibernate at sub_E76 costs another 160 cycles before
    // the next loc_E1E.
    const RUNNABLE_HIBERNATE_SHELL_CYCLES: u32 = 384;
    const RUNNABLE_DISPATCH_CYCLES: u32 = 224;

    let callback_cycles = match callback {
        ModeLevelPrizeObjectExecutionSubE12::FloatingPrize { index } => {
            let work = execute.floating_prize_paths.get(usize::from(index))?;
            match work.fall_path_loc_103b2? {
                // ASM loc_103B2 -> loc_103DE -> loc_10404 -> loc_10396.
                ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallSameRow => 234,
                // The lower-cell Level_Layout probe adds 78 cycles.
                ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallSameRow => 312,
                // ASM loc_103DA skips gravity and rejoins the bottom/collision
                // checks at loc_10404.
                ModeLevelFloatingPrizeFallPathLoc103B2::BlockedCurrentRow => 192,
                // Row-advance and blocked-grace costs need independent source
                // probes before they can enter the cycle scheduler.
                ModeLevelFloatingPrizeFallPathLoc103B2::DirectFallAdvanceRow
                | ModeLevelFloatingPrizeFallPathLoc103B2::GraceProbeFallAdvanceRow
                | ModeLevelFloatingPrizeFallPathLoc103B2::BlockedGraceProbe => return None,
            }
        }
        ModeLevelPrizeObjectExecutionSubE12::Player => return None,
        ModeLevelPrizeObjectExecutionSubE12::PlayerSpawnedInitialSub1007A => {
            let work = execute.player_spawned_prize_block_initial_sub_1007a?;
            match work.path {
                // ASM sub_1007A through the first loc_100DA hibernate.
                crate::ModeLevelPrizeBlockInitialPathSub1007A::Prize => 472,
                crate::ModeLevelPrizeBlockInitialPathSub1007A::RevealOnlyLoc101D8 => return None,
            }
        }
        ModeLevelPrizeObjectExecutionSubE12::FlashLoc102B8 { index } => {
            let work = execute
                .prize_block_flash_paths_sub_1066
                .get(usize::from(index))?;
            match work.wake {
                // ASM loc_1066 -> loc_1062 -> sub_E76.
                ModeLevelPrizeAnimationWake::Hibernate => 50,
                ModeLevelPrizeAnimationWake::FinishAndDelete => return None,
            }
        }
        ModeLevelPrizeObjectExecutionSubE12::WobbleSub1007A { index } => {
            let work = execute
                .prize_block_wobble_paths_sub_1007a
                .get(usize::from(index))?;
            match work.wake {
                ModeLevelPrizeBlockWobbleWake::ContinuePhase => 54,
                ModeLevelPrizeBlockWobbleWake::BeginNextPhase => 62,
                ModeLevelPrizeBlockWobbleWake::BeginAnimationWait => 96,
                ModeLevelPrizeBlockWobbleWake::Unclassified { .. } => return None,
            }
        }
        ModeLevelPrizeObjectExecutionSubE12::TailSub1007A { index } => {
            let work = execute
                .prize_block_tail_paths_sub_1066
                .get(usize::from(index))?;
            match work.state {
                ModeLevelPrizeBlockTailState::FinishSpawnPrizeAndDelete => {
                    return Some(M68kCycleCount(
                        RUNNABLE_DISPATCH_CYCLES
                            + prize_tail_finish_cycles_sub_1007a(work.list_work?).get(),
                    ));
                }
                ModeLevelPrizeBlockTailState::DeletedWorkerOverlap { .. }
                | ModeLevelPrizeBlockTailState::UnclassifiedAnimationWait { .. } => return None,
            }
        }
    };
    Some(M68kCycleCount(
        RUNNABLE_HIBERNATE_SHELL_CYCLES + callback_cycles,
    ))
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
