use super::*;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActivePlatformHandleSub2728 {
    pub(crate) layout_offset_words: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DuplicatePlatformResolutionLoc286A {
    pub(crate) deallocate_existing_slot: bool,
    pub(crate) mark_special_platform_loaded: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformCollisionEntryLoc257E {
    pub(crate) collision_response_word: u32,
    pub(crate) first_platform_slot_pointer_addr: u16,
    pub(crate) kid_gfx_object_addr: u32,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlayerHitboxLoc2590 {
    pub(crate) left: i16,
    pub(crate) right: i16,
    pub(crate) top: i16,
    pub(crate) bottom: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformCollisionReturnLoc2720 {
    pub(crate) collision_type_word: u16,
    pub(crate) scratch_bytes_released: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HorizontalOverlapResolutionLoc25EE {
    pub(crate) left_exit_px: i16,
    pub(crate) right_exit_px: i16,
    pub(crate) push_x_px: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HorizontalOverlapResolutionLoc2616 {
    pub(crate) push_x_px: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VerticalOverlapResolutionLoc261E {
    pub(crate) push_y_px: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VerticalOverlapResolutionLoc2632 {
    pub(crate) top_exit_px: i16,
    pub(crate) bottom_exit_px: i16,
    pub(crate) push_y_px: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VerticalOverlapResolutionLoc265A {
    pub(crate) push_y_px: i16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VerticalPushMagnitudeLoc266A {
    pub(crate) magnitude_word: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn platform_collision_return_loc_2720(
    collision_type_word: u16,
) -> PlatformCollisionReturnLoc2720 {
    // ASM loc_2720 (kid.asm:4639): after `loc_271A` writes `#4` into `$38(a0)`,
    // `Platforms_CheckCollision` releases the 8-byte scratch frame, restores
    // registers with `movem.l`, and returns without changing the collision type.
    PlatformCollisionReturnLoc2720 {
        collision_type_word,
        scratch_bytes_released: 8,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn horizontal_overlap_resolution_loc_25EE(
    platform_left_px: i16,
    platform_right_px: i16,
    player_left_px: i16,
    player_right_px: i16,
) -> HorizontalOverlapResolutionLoc25EE {
    // ASM loc_25EE (kid.asm:4513): once `loc_25A0` has established overlap and
    // `player_left_px > platform_left_px`, this branch handles the case where
    // `player_right_px` still lies inside the platform span. It computes both
    // horizontal escape distances, compares their magnitudes, and keeps the
    // smaller-magnitude push in `d7` before falling through to `loc_261E`.
    let left_exit_px = platform_left_px
        .wrapping_sub(player_right_px)
        .wrapping_sub(1);
    let right_exit_px = platform_right_px
        .wrapping_sub(player_left_px)
        .wrapping_add(1);
    let push_x_px = if left_exit_px.wrapping_neg() < right_exit_px {
        left_exit_px
    } else {
        right_exit_px
    };
    HorizontalOverlapResolutionLoc25EE {
        left_exit_px,
        right_exit_px,
        push_x_px,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn horizontal_overlap_right_exit_loc_2616(
    platform_right_px: i16,
    player_left_px: i16,
) -> HorizontalOverlapResolutionLoc2616 {
    // ASM loc_2616 (kid.asm:4530): when `player_left_px > platform_left_px`
    // and `player_right_px >= platform_right_px`, the scan skips the
    // mixed-overlap compare and uses only the right-hand horizontal exit.
    HorizontalOverlapResolutionLoc2616 {
        push_x_px: platform_right_px
            .wrapping_sub(player_left_px)
            .wrapping_add(1),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn vertical_overlap_top_exit_loc_261E(
    platform_top_px: i16,
    player_bottom_px: i16,
) -> VerticalOverlapResolutionLoc261E {
    // ASM loc_261E (kid.asm:4535): after `d7` is fixed, the top-edge path runs
    // when `player_top_px <= platform_top_px` and uses only the upward escape
    // distance before the absolute-value compare at `loc_2662`.
    VerticalOverlapResolutionLoc261E {
        push_y_px: platform_top_px
            .wrapping_sub(player_bottom_px)
            .wrapping_sub(1),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn vertical_overlap_resolution_loc_2632(
    platform_top_px: i16,
    platform_bottom_px: i16,
    player_top_px: i16,
    player_bottom_px: i16,
) -> VerticalOverlapResolutionLoc2632 {
    // ASM loc_2632 (kid.asm:4544): when `player_top_px > platform_top_px`
    // but `player_bottom_px <= platform_bottom_px`, the vertical branch keeps
    // both escape distances, compares magnitudes via `neg.w d4`, and stores
    // the smaller-magnitude push in `d6` before the absolute-value stage.
    let top_exit_px = platform_top_px
        .wrapping_sub(player_bottom_px)
        .wrapping_sub(1);
    let bottom_exit_px = platform_bottom_px
        .wrapping_sub(player_top_px)
        .wrapping_add(1);
    let push_y_px = if top_exit_px.wrapping_neg() < bottom_exit_px {
        top_exit_px
    } else {
        bottom_exit_px
    };
    VerticalOverlapResolutionLoc2632 {
        top_exit_px,
        bottom_exit_px,
        push_y_px,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn vertical_overlap_bottom_exit_loc_265A(
    platform_bottom_px: i16,
    player_top_px: i16,
) -> VerticalOverlapResolutionLoc265A {
    // ASM loc_265A (kid.asm:4561): when the player's feet already extend past
    // the platform bottom, the routine skips the mixed compare and uses only
    // the downward escape `d6 = platform_bottom - player_top + 1`.
    VerticalOverlapResolutionLoc265A {
        push_y_px: platform_bottom_px
            .wrapping_sub(player_top_px)
            .wrapping_add(1),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn vertical_push_magnitude_loc_266A(push_y_px: i16) -> VerticalPushMagnitudeLoc266A {
    // ASM loc_266A (kid.asm:4571): copy `d6` into `d4`, and when the signed
    // push is negative run `neg.w d4`. Returning the raw 16-bit word keeps the
    // original wraparound behaviour for the `$8000` overflow case.
    VerticalPushMagnitudeLoc266A {
        magnitude_word: if push_y_px >= 0 {
            push_y_px as u16
        } else {
            push_y_px.wrapping_neg() as u16
        },
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HorizontalPlatformPushLoc2686 {
    pub(crate) kid_hitbox_left_px: i16,
    pub(crate) kid_hitbox_right_px: i16,
    pub(crate) kid_gfx_x_px: i16,
    pub(crate) takes_negative_branch: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn apply_horizontal_platform_push_loc_2686(
    kid_hitbox_left_px: i16,
    kid_hitbox_right_px: i16,
    kid_gfx_x_px: i16,
    push_x_px: i16,
) -> HorizontalPlatformPushLoc2686 {
    // ASM loc_2686 (kid.asm:4583): once `d7` wins the axis compare, add that
    // signed push into `Kid_hitbox_left/right` and `$1A(a0)` before `tst.w d7`
    // picks the negative path at `loc_26B6` or the non-negative path below.
    HorizontalPlatformPushLoc2686 {
        kid_hitbox_left_px: kid_hitbox_left_px.wrapping_add(push_x_px),
        kid_hitbox_right_px: kid_hitbox_right_px.wrapping_add(push_x_px),
        kid_gfx_x_px: kid_gfx_x_px.wrapping_add(push_x_px),
        takes_negative_branch: push_x_px < 0,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PositivePlatformCarryLoc26A2 {
    pub(crate) carried_x_velocity_fp: i32,
    pub(crate) touched_right_wall: bool,
    pub(crate) escalates_to_loc_271_a: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn positive_platform_carry_loc_26A2(
    platform_x_velocity_fp: i32,
    left_wall_already_touched: bool,
) -> PositivePlatformCarryLoc26A2 {
    // ASM loc_26A2 (kid.asm:4593): after the non-negative `d7` branch, keep
    // `$A(a2)` only when it is non-negative, otherwise clamp the carried
    // velocity to zero, set `$FFFFFAA3`, and jump to `loc_271A` only if
    // `$FFFFFAA2` was already set by the opposite-side branch.
    PositivePlatformCarryLoc26A2 {
        carried_x_velocity_fp: platform_x_velocity_fp.max(0),
        touched_right_wall: true,
        escalates_to_loc_271_a: left_wall_already_touched,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NegativePlatformCarryLoc26B6 {
    pub(crate) carried_x_velocity_fp: i32,
    pub(crate) touched_left_wall: bool,
    pub(crate) escalates_to_loc_271_a: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NegativePlatformCarryCommitLoc26C0 {
    pub(crate) carried_x_velocity_fp: i32,
    pub(crate) touched_left_wall: bool,
    pub(crate) escalates_to_loc_271_a: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformLandingLoc26D4 {
    pub(crate) kid_hitbox_top_px: i16,
    pub(crate) kid_hitbox_bottom_px: i16,
    pub(crate) kid_gfx_y_px: i16,
    pub(crate) kid_y_velocity_fp: i32,
    pub(crate) standing_platform_addr: u16,
    pub(crate) touched_top_face: bool,
    pub(crate) escalates_to_loc_271_a: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BottomPlatformCarryLoc26FE {
    pub(crate) carried_y_velocity_fp: i32,
    pub(crate) touched_bottom_face: bool,
    pub(crate) escalates_to_loc_271_a: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn negative_platform_carry_loc_26B6(
    platform_x_velocity_fp: i32,
    right_wall_already_touched: bool,
) -> NegativePlatformCarryLoc26B6 {
    // ASM loc_26B6 (kid.asm:4601): after the negative `d7` branch, keep
    // `$A(a2)` only when it is negative, otherwise clamp the carried velocity
    // to zero, set `$FFFFFAA2`, and jump to `loc_271A` only if `$FFFFFAA3`
    // was already set by the non-negative branch.
    NegativePlatformCarryLoc26B6 {
        carried_x_velocity_fp: platform_x_velocity_fp.min(0),
        touched_left_wall: true,
        escalates_to_loc_271_a: right_wall_already_touched,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn negative_platform_carry_commit_loc_26C0(
    carried_x_velocity_fp: i32,
    right_wall_already_touched: bool,
) -> NegativePlatformCarryCommitLoc26C0 {
    // ASM loc_26C0 (kid.asm:4606): write the already-chosen horizontal carry
    // back to `$26(a0)`, set `$FFFFFAA2`, and jump to `loc_271A` only if the
    // non-negative branch had already set `$FFFFFAA3`.
    NegativePlatformCarryCommitLoc26C0 {
        carried_x_velocity_fp,
        touched_left_wall: true,
        escalates_to_loc_271_a: right_wall_already_touched,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn platform_landing_loc_26D4(
    kid_hitbox_top_px: i16,
    kid_hitbox_bottom_px: i16,
    kid_gfx_y_px: i16,
    push_y_px: i16,
    standing_platform_addr: u16,
    bottom_face_already_touched: bool,
) -> PlatformLandingLoc26D4 {
    // ASM loc_26D4 (kid.asm:4614): add the chosen vertical push into
    // `Kid_hitbox_top/bottom` and `$1E(a0)`. When that push is negative, clear
    // `y_vel(a3)`, remember `a2` in `(Addr_PlatformStandingOn).w`, set
    // `$FFFFFAA4`, and escalate to `loc_271A` only if `$FFFFFAA5` was already
    // set by the opposite-face branch.
    PlatformLandingLoc26D4 {
        kid_hitbox_top_px: kid_hitbox_top_px.wrapping_add(push_y_px),
        kid_hitbox_bottom_px: kid_hitbox_bottom_px.wrapping_add(push_y_px),
        kid_gfx_y_px: kid_gfx_y_px.wrapping_add(push_y_px),
        kid_y_velocity_fp: 0,
        standing_platform_addr,
        touched_top_face: true,
        escalates_to_loc_271_a: bottom_face_already_touched,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn bottom_platform_carry_loc_26FE(
    platform_y_velocity_fp: i32,
    character_movement_is_jump: bool,
) -> BottomPlatformCarryLoc26FE {
    // ASM loc_26FE (kid.asm:4628): use the platform's `$E(a2)` velocity plus
    // the extra `$4000` downward bias, set `$FFFFFAA5`, and skip `loc_271A`
    // only while `(Character_Movement).w == MoveID_Jump`.
    BottomPlatformCarryLoc26FE {
        carried_y_velocity_fp: platform_y_velocity_fp.wrapping_add(0x4000),
        touched_bottom_face: true,
        escalates_to_loc_271_a: !character_movement_is_jump,
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AllocatePlatformSlotSub238E {
    pub(crate) allocated_slot_addr: u16,
    pub(crate) next_free_platform_slot_addr: u16,
    pub(crate) first_platform_slot_addr: u16,
    pub(crate) number_platforms: u16,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AllocatePlatformSlotResultSub238E {
    Allocated(AllocatePlatformSlotSub238E),
    NoFreeSlot { failure_label: &'static str },
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn allocate_platform_slot_sub_238E(
    next_free_platform_slot_addr: u16,
    allocated_slot_next_addr: u16,
    _first_platform_slot_addr: u16,
    number_platforms: u16,
) -> AllocatePlatformSlotResultSub238E {
    // ASM sub_238E / loc_23AE (kid.asm:4214-4222): if the free-list head at
    // `(Addr_NextFreePlatformSlot).w` is zero, jump straight to `loc_6E2`.
    // Otherwise pop that slot, link it onto the front of the active list, and
    // increment `(Number_Platforms).w`.
    if next_free_platform_slot_addr == 0 {
        AllocatePlatformSlotResultSub238E::NoFreeSlot {
            failure_label: "loc_6E2",
        }
    } else {
        AllocatePlatformSlotResultSub238E::Allocated(AllocatePlatformSlotSub238E {
            allocated_slot_addr: next_free_platform_slot_addr,
            next_free_platform_slot_addr: allocated_slot_next_addr,
            first_platform_slot_addr: next_free_platform_slot_addr,
            number_platforms: number_platforms.wrapping_add(1),
        })
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformSlotLinkSub23B4 {
    pub(crate) slot_addr: u16,
    pub(crate) next_slot_addr: u16,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeallocatePlatformSlotResultSub23B4 {
    Deallocated,
    MissingSlot { failure_label: &'static str },
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn deallocate_platform_slot_sub_23B4(
    active_links: &mut [PlatformSlotLinkSub23B4],
    first_platform_slot_addr: &mut u16,
    next_free_platform_slot_addr: &mut u16,
    number_platforms: &mut u16,
    target_slot_addr: u16,
) -> DeallocatePlatformSlotResultSub23B4 {
    // ASM loc_23BA (kid.asm:4236-4241): start from the sentinel that stores
    // `(Addr_FirstPlatformSlot).w`, follow each slot's word link until the
    // target `a3` appears, and keep walking on mismatches. loc_23CC then
    // unlinks the matched slot, pushes it onto `(Addr_NextFreePlatformSlot).w`,
    // and decrements `(Number_Platforms).w`; loc_23E4 jumps to `loc_6E2` when
    // the walk reaches a zero link first.
    let mut previous_index: Option<usize> = None;
    let mut cursor = *first_platform_slot_addr;

    while cursor != 0 {
        let Some(current_index) = active_links
            .iter()
            .position(|link| link.slot_addr == cursor)
        else {
            return DeallocatePlatformSlotResultSub23B4::MissingSlot {
                failure_label: "loc_6E2",
            };
        };
        if cursor == target_slot_addr {
            let next_active_addr = active_links[current_index].next_slot_addr;
            if let Some(previous_slot_index) = previous_index {
                active_links[previous_slot_index].next_slot_addr = next_active_addr;
            } else {
                *first_platform_slot_addr = next_active_addr;
            }
            active_links[current_index].next_slot_addr = *next_free_platform_slot_addr;
            *next_free_platform_slot_addr = target_slot_addr;
            *number_platforms = number_platforms.wrapping_sub(1);
            return DeallocatePlatformSlotResultSub23B4::Deallocated;
        }
        previous_index = Some(current_index);
        cursor = active_links[current_index].next_slot_addr;
    }

    DeallocatePlatformSlotResultSub23B4::MissingSlot {
        failure_label: "loc_6E2",
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn find_active_platform_miss_loc_2740() -> Option<usize> {
    // ASM loc_2740 (kid.asm:4662): once the active platform-slot walk reaches
    // a zero link, `moveq #0,d7; rts` reports "no matching slot".
    None
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn resolve_duplicate_platform_load_loc_286A(
    existing_platform_kind: u8,
) -> DuplicatePlatformResolutionLoc286A {
    // ASM loc_2850 / loc_286A (kid.asm:4771): after `sub_2728` finds an
    // already-active platform for the current loader offset, `tst.b $1F(a3)`
    // checks its kind byte. Non-negative kinds branch to `loc_286A` and call
    // `Deallocate_PlatformSlot`, while negative kinds stay active and receive
    // `st $12(a3)` instead.
    let existing_is_special = (existing_platform_kind as i8).is_negative();
    DuplicatePlatformResolutionLoc286A {
        deallocate_existing_slot: !existing_is_special,
        mark_special_platform_loaded: existing_is_special,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn advance_platform_loader_offset_loc_286E(platform_loader_offset: u16) -> u16 {
    // ASM loc_286E (kid.asm:4785): every exit path from
    // `Manage_PlatformLoading` advances `(PlatformLoader_Offset).w` by one
    // 12-byte platform-layout record before returning.
    platform_loader_offset.wrapping_add(0x000C)
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn find_active_platform_by_layout_offset_sub_2728(
    layout_offset_words: u16,
    active_platforms: &[ActivePlatformHandleSub2728],
) -> Option<usize> {
    // ASM sub_2728 (kid.asm:4649): `lea (Addr_FirstPlatformSlot).w,a2`
    // seeds the active-list cursor, then `loc_272C` walks each live slot in
    // list order until `$20(a2)` matches the current PlatformLoader_Offset.
    let mut slot_index = 0;
    while slot_index < active_platforms.len() {
        // ASM loc_272C (kid.asm:4652): load the current slot link, skip empty
        // entries, and loop until `cmp.w $20(a2),d5` finds the first match.
        if active_platforms[slot_index].layout_offset_words == layout_offset_words {
            return Some(slot_index);
        }
        slot_index += 1;
    }
    find_active_platform_miss_loc_2740()
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn begin_platform_collision_loc_257E(
    kid_gfx_object_addr: u32,
) -> PlatformCollisionEntryLoc257E {
    // ASM loc_257E (kid.asm:4471): after the transform guard passes,
    // `Platforms_CheckCollision` reserves 8 stack bytes for platform bounds,
    // clears the long scratch word at `$FFFFFAA2`, points `a2` at the
    // `(Addr_FirstPlatformSlot).w` sentinel, and reloads Kid's gfx object
    // address into `a0` before continuing to `loc_2590`.
    PlatformCollisionEntryLoc257E {
        collision_response_word: 0,
        first_platform_slot_pointer_addr: 0xFA8E,
        kid_gfx_object_addr,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_player_hitbox_loc_2590(
    left: i16,
    right: i16,
    top: i16,
    bottom: i16,
) -> PlayerHitboxLoc2590 {
    // ASM loc_2590 (kid.asm:4478): seed the platform collision scan from the
    // current Kid hitbox globals by loading left/right/top/bottom into
    // `d0`/`d1`/`d2`/`d3` before the linked-list walk at `loc_25A0`.
    PlayerHitboxLoc2590 {
        left,
        right,
        top,
        bottom,
    }
}

impl PlatformLoaderState {
    pub(crate) fn advance_to_window(&mut self, layout: &PlatformLayout, window: ActivationWindow) {
        while self.next_index < layout.placements.len() {
            let placement = &layout.placements[self.next_index];
            let preload_limit = window
                .x_max
                .saturating_add(PLATFORM_PRELOAD_X_PADDING)
                .saturating_add(u32::from(placement.buffer_right));
            if u32::from(placement.x) > preload_limit {
                break;
            }
            self.next_index += 1;
        }
    }

    pub(crate) fn active_platforms(
        &self,
        layout: &PlatformLayout,
        window: ActivationWindow,
    ) -> Vec<NativeObjectSnapshot> {
        layout
            .placements
            .iter()
            .take(self.next_index)
            .filter(|placement| platform_is_active(placement, window))
            .map(|placement| NativeObjectSnapshot {
                kind: NativeObjectKind::Platform {
                    kind: placement.kind,
                    subtype: placement.subtype,
                },
                x_pos: u32::from(placement.x) << 16,
                y_pos: u32::from(placement.y) << 16,
                layout_index: None,
                x_velocity: None,
                y_velocity: None,
                facing: None,
                on_ground: None,
                hit_points: None,
                damage_flash: false,
                palette_variant: 0,
                stationary: false,
                shared_walker_state: None,
                shared_walker_patrol_timer: None,
                shared_walker_patrol_short_next: None,
                walk_phase: None,
                wait_timer: None,
                debug_fields: std::collections::BTreeMap::new(),
            })
            .collect()
    }
}

/// Check if a platform is within the activation window.
pub(crate) fn platform_is_active(platform: &PlatformPlacement, window: ActivationWindow) -> bool {
    // ASM loc_2762 (kid.asm:4684): after loading Kid's X/Y into `d6`, the ROM
    // rejects only strict out-of-range placements via `bgt`/`blt` against the
    // buffered left/right/top/bottom edges. Equality stays in range, so the
    // native port mirrors that with inclusive `RangeInclusive` checks.
    let object_x = u32::from(platform.x);
    let object_y = u32::from(platform.y);
    let x_min = window.x_min.saturating_sub(u32::from(platform.buffer_left));
    let x_max = window
        .x_max
        .saturating_add(u32::from(platform.buffer_right));
    let y_min = window.y_min.saturating_sub(u32::from(platform.buffer_top));
    let y_max = window
        .y_max
        .saturating_add(u32::from(platform.buffer_bottom));
    (x_min..=x_max).contains(&object_x) && (y_min..=y_max).contains(&object_y)
}

/// Check if the player should land on top of any active platform.
///
/// Runs after terrain collision so that platforms override terrain when the
/// player is above a platform surface. Only applies when the player is not
/// already grounded by terrain (falling or airborne).
pub(crate) fn apply_platform_collision(
    slice: &mut OpeningSliceState,
    platforms: &[ActivePlatform],
) {
    let player_x_px = (slice.player.x_pos >> 16) as i32;
    let player_foot_y_px = (slice.player.y_pos >> 16) as i32;
    let player_left = player_x_px - PLAYER_HALF_WIDTH;
    let player_right = player_x_px + PLAYER_HALF_WIDTH;

    // ASM loc_7886 (kid.asm:7929): after sub_902A finds floor while Kid
    // stands on a platform, the ROM calls sub_8ED0 to check whether Kid's
    // hitbox still overlaps the platform span.  If unsupported (NE), the ROM
    // sets MoveID_Jump / sub_B270 / loc_A6F8 — Kid becomes airborne.  If
    // supported (EQ), the ROM continues to loc_789C (stay standing).
    // Native mirrors this via the `platform_support_check_sub_8ED0` call
    // below: unsupported platforms are skipped (Kid gets no carry velocity,
    // eventually falls off via normal gravity/floor logic).
    if slice.player.on_ground {
        for platform in platforms {
            if platform.x_velocity == 0 && platform.y_velocity == 0 {
                continue;
            }
            let plat_y_px = (platform.y_pos >> 16) as i32;
            if platform_support_check_sub_8ED0(
                Some(platform),
                slice.player.x_pos,
                PLAYER_HALF_WIDTH,
                PlayerForm::TheKid,
                false,
            ) {
                continue;
            }
            // Player feet within snap distance of platform top → riding it
            let within_snap =
                slice.player.y_pos.abs_diff(platform.y_pos) <= GROUND_SNAP_TOLERANCE_FP;
            if !within_snap && player_foot_y_px.abs_diff(plat_y_px) > 2 {
                continue;
            }

            // ASM loc_78A8 (kid.asm:7943): when sub_902A returns negative
            // (ceiling collision while being carried by a platform), the ROM
            // checks the platform's y_vel ($E(a4)).  If y_vel < 0 (platform
            // moving upward), Kid is crushed → loc_78C0 (kid.asm:7952):
            // `moveq #0,d7 / bra.w Death`.  If y_vel >= 0 (platform moving
            // down or stationary), the ROM clears Addr_PlatformStandingOn and
            // enters walking (sub_942A → loc_8BF0).
            //
            // ASM loc_8DA0 (kid.asm:10164): walking-on-platform post-wall-
            // collision y tracking — `move.w (Addr_PlatformStandingOn),a4;
            // move.l $E(a4),y_vel(a3); move.l 6(a4),y_pos(a3); subq.w #1,
            // y_pos(a3)` — snaps y_pos to platform_y - 1, copies y_vel.
            // Then `bsr.w sub_902A`: if EQ → loc_8DD4 (support check), if
            // d7==2 → loc_8DF6 (kid.asm:10193): `moveq #0,d7; bra.w Death`
            // (platform crush death), if NE → detach (clr Addr_PlatformStandingOn,
            // clr y_vel, sub_942A, bra loc_8BF0).
            //
            // ASM loc_8DD4 (kid.asm:10179): walking-on-platform support test
            // — `bsr.w sub_8ED0`: if NE (unsupported) → `move.w #MoveID_Jump;
            // bsr.w sub_B270; bra.w loc_A6F8` (become airborne).  If EQ
            // (supported) → loc_8DEA: `move.l ($FFFFFA98),d0; bsr.w sub_942A;
            // bra.w loc_8BF0` (continue walking on platform).
            // Native: `platform_support_check_sub_8ED0` decides skip/carry;
            // unsupported platforms are skipped so Kid falls via gravity.
            // TODO(fidelity): native does not yet detect platform-vs-ceiling
            // crush death.  Upward-moving platforms carry the player without
            // checking for ceiling collision.
            //
            // ASM loc_8306 (kid.asm:9077): crawling-on-platform y tracking —
            // `move.w (Addr_PlatformStandingOn).w,a4; move.l $E(a4),y_vel(a3);
            // move.l 6(a4),y_pos(a3); subq.w #1,y_pos(a3)` — snaps Kid's
            // y_pos to platform top minus 1, copies platform y_vel. Then
            // `bsr.w sub_902A`: if d7==2 → Death (loc_837C crush); if NE
            // (ceiling hit) → detach + re-enter crawl (loc_8218); if EQ →
            // loc_833E floor/edge check.
            // Native: platform carry applies velocity additively rather than
            // snapping to platform_y - 1; the sub_902A ceiling/crush branch
            // is a TODO(fidelity) above.
            //
            // ASM loc_833E (kid.asm:9093): crawl-on-platform floor/edge
            // check — `bsr.w sub_8ED0` (platform support test): if EQ
            // (supported) → loc_8370 (continue crawling with $FFFFFA98
            // accumulated velocity). If NE (unsupported), `bsr.w sub_B43A`
            // floor probe: if floor → loc_818A (walking→airborne exit); else
            // detach + re-enter crawl (loc_8218).
            // Native: `platform_support_check_sub_8ED0` (below) mirrors
            // sub_8ED0; unsupported platforms are skipped (no carry), so Kid
            // falls off via normal gravity.
            //
            // Carry player with platform velocity
            slice.player.x_pos = (i64::from(slice.player.x_pos) + i64::from(platform.x_velocity))
                .clamp(0, i64::from(u32::MAX)) as u32;
            slice.player.y_pos = (i64::from(slice.player.y_pos) + i64::from(platform.y_velocity))
                .clamp(0, i64::from(u32::MAX)) as u32;
            let floor_y_px = (slice.player.y_pos >> 16) as i32;
            slice.terrain.floor_y_pos = (floor_y_px.max(0) as u32) << 16;
            return;
        }
        return;
    }

    if slice.player.y_velocity < 0 {
        // Moving upward — don't land on platforms from below
        return;
    }

    for platform in platforms {
        let plat_x_px = (platform.x_pos >> 16) as i32;
        let plat_y_px = (platform.y_pos >> 16) as i32;
        let plat_right = plat_x_px + i32::from(platform.width_px);

        // Horizontal overlap check
        if player_right < plat_x_px || player_left > plat_right {
            continue;
        }

        // Vertical check: player feet should be at or just below the platform top.
        // Use the same snap tolerance as terrain floor collision.
        let plat_top_fp = platform.y_pos;
        let within_snap = slice.player.y_pos.abs_diff(plat_top_fp) <= GROUND_SNAP_TOLERANCE_FP;
        let feet_at_or_below = player_foot_y_px >= plat_y_px;

        if feet_at_or_below || within_snap {
            // Don't land if player feet are well past the platform (inside it)
            let plat_bottom = plat_y_px + i32::from(platform.height_px);
            if player_foot_y_px > plat_bottom {
                continue;
            }

            // Land on this platform
            let floor_y_px = plat_y_px.saturating_sub(1);
            let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
            // ASM floor landing: loc_AAF8 (y_vel >= 0) preserves sub-pixel,
            // loc_AB22 (y_vel < 0) zeroes it. Use pre_gravity_y_vel since
            // native applies gravity before collisions (GravityFirst).
            if slice.pre_gravity_y_vel >= 0 {
                slice.player.y_pos =
                    (floor_y_fp & 0xFFFF_0000) | (slice.player.y_pos & 0x0000_FFFF);
            } else {
                slice.player.y_pos = floor_y_fp;
            }
            slice.player.on_ground = true;
            slice.player.y_velocity = 0;
            slice.terrain.floor_y_pos = floor_y_fp;
            return;
        }
    }
}

#[allow(non_snake_case)]
pub(crate) fn platform_support_check_sub_8ED0(
    platform: Option<&ActivePlatform>,
    player_x_pos: u32,
    collision_half_width_px: i32,
    current_form: PlayerForm,
    crawling: bool,
) -> bool {
    // ASM sub_8ED0 (kid.asm:10282): if no active platform is recorded, set
    // `collision_type=4` and return unsupported. Otherwise, check whether
    // Kid's centre ± active half-width still overlaps the platform span.
    // Called from loc_833E (kid.asm:9093) during crawl-on-platform and from
    // loc_7886 (kid.asm:7929) during standing-on-platform.
    let Some(platform) = platform else {
        return true;
    };

    let support_half_width_px = if crawling {
        if current_form == PlayerForm::TheKid {
            5
        } else {
            7
        }
    } else {
        collision_half_width_px
    };
    let player_x_px = (player_x_pos >> 16) as i32;
    let platform_left_px = (platform.x_pos >> 16) as i32;
    let platform_right_px = platform_left_px + i32::from(platform.width_px);

    player_x_px + support_half_width_px < platform_left_px
        || player_x_px - support_half_width_px > platform_right_px
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformSpriteDispatchInputLoc244C {
    pub(crate) world_x_px: i16,
    pub(crate) world_y_px: i16,
    pub(crate) camera_x_px: i16,
    pub(crate) camera_y_px: i16,
    pub(crate) width_px: i16,
    pub(crate) height_px: i16,
    pub(crate) mapping_addr_word: u16,
    pub(crate) render_flags: u8,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlatformSpriteVariantLoc244C {
    Loc24CC,
    Loc24F0,
    Loc251C,
    Loc24D4,
    Loc24C0,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformSpriteDispatchLoc244C {
    pub(crate) screen_x_word: i16,
    pub(crate) screen_y_word: i16,
    pub(crate) width_tiles_minus_one: u16,
    pub(crate) height_tiles_minus_one: u16,
    pub(crate) base_tile_word: u16,
    pub(crate) mapping_addr_word: u16,
    pub(crate) variant: PlatformSpriteVariantLoc244C,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn platform_sprite_dispatch_loc_244C(
    input: PlatformSpriteDispatchInputLoc244C,
) -> Option<PlatformSpriteDispatchLoc244C> {
    // ASM loc_244C (kid.asm:4329): walk the active platform list, cull against
    // the `320x224` camera window using the platform width/height words at
    // `$1A(a2)`/`$1C(a2)`, bias visible positions by `+$80`, set `d2=#$500`,
    // and dispatch through `off_24AC` using the low nibble of `$1F(a2)`.
    let relative_x = input.world_x_px.wrapping_sub(input.camera_x_px);
    if relative_x >= 0x0140 {
        return None;
    }
    if relative_x.wrapping_add(input.width_px) < 0 {
        return None;
    }

    let relative_y = input.world_y_px.wrapping_sub(input.camera_y_px);
    if relative_y >= 0x00E0 {
        return None;
    }
    if relative_y.wrapping_add(input.height_px) < 0 {
        return None;
    }

    let variant = match input.render_flags & 0x0F {
        0 => PlatformSpriteVariantLoc244C::Loc24CC,
        1 => PlatformSpriteVariantLoc244C::Loc24F0,
        2 => PlatformSpriteVariantLoc244C::Loc251C,
        3 => PlatformSpriteVariantLoc244C::Loc24D4,
        4 => PlatformSpriteVariantLoc244C::Loc24C0,
        _ => return None,
    };

    Some(PlatformSpriteDispatchLoc244C {
        screen_x_word: relative_x.wrapping_add(0x0080),
        screen_y_word: relative_y.wrapping_add(0x0080),
        width_tiles_minus_one: (input.width_px >> 4) as u16,
        height_tiles_minus_one: (input.height_px >> 4) as u16,
        base_tile_word: 0x0500,
        mapping_addr_word: input.mapping_addr_word,
        variant,
    })
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformSpriteEntryLoc2548 {
    pub(crate) y_word: u16,
    pub(crate) tile_word: u16,
    pub(crate) mapping_word: u16,
    pub(crate) x_word: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlatformSpriteEmitLoc2548 {
    pub(crate) sprite: PlatformSpriteEntryLoc2548,
    pub(crate) next_number_sprites: u8,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn emit_platform_sprite_loc_2548(
    screen_x_word: i16,
    screen_y_word: i16,
    base_tile_word: u16,
    mapping_addr_word: u16,
    number_sprites: u8,
) -> PlatformSpriteEmitLoc2548 {
    // ASM loc_2548 (kid.asm:4442): write `(y,tile,mapping,x)` into the next
    // sprite slot, increment `(Number_Sprites).w`, add that low byte into `d2`,
    // and add the mapping word in `a4` onto `$8000`.
    let next_number_sprites = number_sprites.wrapping_add(1);
    let tile_low_byte = (base_tile_word as u8).wrapping_add(next_number_sprites);
    let tile_word = (base_tile_word & 0xFF00) | u16::from(tile_low_byte);

    PlatformSpriteEmitLoc2548 {
        sprite: PlatformSpriteEntryLoc2548 {
            y_word: screen_y_word as u16,
            tile_word,
            mapping_word: 0x8000u16.wrapping_add(mapping_addr_word),
            x_word: screen_x_word as u16,
        },
        next_number_sprites,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn finish_platform_sprite_list_loc_256E(next_sprite_slot_addr: u32) -> u32 {
    // ASM loc_256E (kid.asm:4456): `move.l a0,(Addr_NextSpriteSlot).w; rts`.
    // `a0` already points at the first free 8-byte sprite slot after the
    // final `loc_2548` write, so the epilogue only publishes that pointer.
    next_sprite_slot_addr
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn platform_single_sprite_loc_24C0(
    dispatch: PlatformSpriteDispatchLoc244C,
    number_sprites: u8,
) -> PlatformSpriteEmitLoc2548 {
    // ASM loc_24C0 (kid.asm:4374): replace `d2` with `#$400`, copy `d7` back
    // into `d4`, emit one sprite through `loc_2548`, then loop to `loc_244C`.
    emit_platform_sprite_loc_2548(
        dispatch.screen_x_word,
        dispatch.screen_y_word,
        0x0400,
        dispatch.mapping_addr_word,
        number_sprites,
    )
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlatformSpriteStripLoc24CC {
    pub(crate) sprites: Vec<PlatformSpriteEntryLoc2548>,
    pub(crate) next_number_sprites: u8,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn platform_strip_sprites_loc_24CC(
    dispatch: PlatformSpriteDispatchLoc244C,
    number_sprites: u8,
) -> PlatformSpriteStripLoc24CC {
    // ASM loc_24CC (kid.asm:4381): set `d2=#$700`, clear `d1`, subtract 2
    // from `d6`, then fall into loc_24D4/loc_24D8. Because `d1` starts at 0,
    // the outer DBF loop runs exactly one row while the inner DBF loop emits
    // `d0 + 1` consecutive 16px-wide cells across that row.
    let mut sprites = Vec::new();
    let mut next_number_sprites = number_sprites;
    let mut screen_x_word = dispatch.screen_x_word;
    let screen_y_word = dispatch.screen_y_word.wrapping_sub(2);

    for _ in 0..=dispatch.width_tiles_minus_one {
        let emitted = emit_platform_sprite_loc_2548(
            screen_x_word,
            screen_y_word,
            0x0700,
            dispatch.mapping_addr_word,
            next_number_sprites,
        );
        sprites.push(emitted.sprite);
        next_number_sprites = emitted.next_number_sprites;
        screen_x_word = screen_x_word.wrapping_add(0x0010);
    }

    PlatformSpriteStripLoc24CC {
        sprites,
        next_number_sprites,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlatformSpriteStripLoc24F0 {
    pub(crate) sprites: Vec<PlatformSpriteEntryLoc2548>,
    pub(crate) next_number_sprites: u8,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn platform_strip_sprites_loc_24F0(
    dispatch: PlatformSpriteDispatchLoc244C,
    number_sprites: u8,
) -> PlatformSpriteStripLoc24F0 {
    // ASM loc_24F0 (kid.asm:4400): emit the first platform sprite at the
    // current mapping word, advance `a4`/`d4`, then branch into the remaining
    // interior/final sprite handling based on the decremented width counter.
    let mut sprites = Vec::new();
    let mut next_number_sprites = number_sprites;

    let left = emit_platform_sprite_loc_2548(
        dispatch.screen_x_word,
        dispatch.screen_y_word,
        dispatch.base_tile_word,
        dispatch.mapping_addr_word,
        next_number_sprites,
    );
    sprites.push(left.sprite);
    next_number_sprites = left.next_number_sprites;

    let middle_mapping_word = dispatch.mapping_addr_word.wrapping_add(0x0004);
    let right_mapping_word = middle_mapping_word.wrapping_add(0x0004);
    let mut screen_x_word = dispatch.screen_x_word.wrapping_add(0x0010);

    if dispatch.width_tiles_minus_one > 1 {
        // ASM loc_2506 (kid.asm:4410): emit the interior span with the same
        // middle mapping word each time, advance `d4` by 16px after every
        // sprite, and let DBF repeat exactly `width_tiles_minus_one - 1` times.
        for _ in 0..(dispatch.width_tiles_minus_one - 1) {
            let emitted = emit_platform_sprite_loc_2548(
                screen_x_word,
                dispatch.screen_y_word,
                dispatch.base_tile_word,
                middle_mapping_word,
                next_number_sprites,
            );
            sprites.push(emitted.sprite);
            next_number_sprites = emitted.next_number_sprites;
            screen_x_word = screen_x_word.wrapping_add(0x0010);
        }
    }

    // ASM loc_2512 (kid.asm:4415): skip the interior loop when only two caps
    // are needed, advance `a4` once more to the right-cap mapping word, and
    // emit the final sprite at the already-advanced X position.
    let right = emit_platform_sprite_loc_2548(
        screen_x_word,
        dispatch.screen_y_word,
        dispatch.base_tile_word,
        right_mapping_word,
        next_number_sprites,
    );
    sprites.push(right.sprite);
    next_number_sprites = right.next_number_sprites;

    PlatformSpriteStripLoc24F0 {
        sprites,
        next_number_sprites,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn platform_strip_sprites_loc_251C(
    dispatch: PlatformSpriteDispatchLoc244C,
    number_sprites: u8,
) -> PlatformSpriteStripLoc24CC {
    // ASM loc_251C (kid.asm:4421): emit the first platform sprite at the
    // current mapping word, advance `a4`/`d6`, then branch into the remaining
    // vertical-strip handling based on the decremented height counter.
    let mut sprites = Vec::new();
    let mut next_number_sprites = number_sprites;

    let top = emit_platform_sprite_loc_2548(
        dispatch.screen_x_word,
        dispatch.screen_y_word,
        dispatch.base_tile_word,
        dispatch.mapping_addr_word,
        next_number_sprites,
    );
    sprites.push(top.sprite);
    next_number_sprites = top.next_number_sprites;

    let middle_mapping_word = dispatch.mapping_addr_word.wrapping_add(0x0004);
    let bottom_mapping_word = middle_mapping_word.wrapping_add(0x0004);
    let mut screen_y_word = dispatch.screen_y_word.wrapping_add(0x0010);

    if dispatch.height_tiles_minus_one > 1 {
        // ASM loc_2532 (kid.asm:4431): reuse the middle mapping word while
        // `dbf d0,loc_2532` walks the interior span one 16px row at a time.
        for _ in 0..(dispatch.height_tiles_minus_one - 1) {
            let emitted = emit_platform_sprite_loc_2548(
                dispatch.screen_x_word,
                screen_y_word,
                dispatch.base_tile_word,
                middle_mapping_word,
                next_number_sprites,
            );
            sprites.push(emitted.sprite);
            next_number_sprites = emitted.next_number_sprites;
            screen_y_word = screen_y_word.wrapping_add(0x0010);
        }
    }

    // ASM loc_253E (kid.asm:4436): skip the interior loop when only two caps
    // are needed, advance `a4` once more to the bottom-cap mapping word, and
    // emit the final sprite at the already-advanced Y position.
    let bottom = emit_platform_sprite_loc_2548(
        dispatch.screen_x_word,
        screen_y_word,
        dispatch.base_tile_word,
        bottom_mapping_word,
        next_number_sprites,
    );
    sprites.push(bottom.sprite);
    next_number_sprites = bottom.next_number_sprites;

    PlatformSpriteStripLoc24CC {
        sprites,
        next_number_sprites,
    }
}

// ---------------------------------------------------------------------------
// SoftPlatform / TrapPlatformDown / TrapPlatformUp script routines
// ---------------------------------------------------------------------------

/// Per-frame action for the SoftPlatform settled wait state.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SoftPlatformSettledActionLoc4160 {
    /// Kid is still standing on this platform — stay in settled wait.
    StaySettled,
    /// Kid left or switched platforms — return to `SoftPlatform_WaitLoop`.
    ReturnToWaitLoop,
}

/// SoftPlatform settled wait: after the bounce loop restores the platform to
/// its original y position and clears velocity, this state hibernates one
/// frame per iteration and checks whether Kid is still standing on this
/// specific platform.
#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn soft_platform_settled_step_loc_4160(
    platform_standing_on: u16,
    this_platform_addr: u16,
) -> SoftPlatformSettledActionLoc4160 {
    // ASM loc_4160 (kid.asm:5242): jsr Hibernate, then:
    //   move.w (Addr_PlatformStandingOn).w,d7
    //   beq.s  SoftPlatform_WaitLoop        ; zero → no platform
    //   cmp.w  a3,d7
    //   bne.s  SoftPlatform_WaitLoop        ; different platform
    //   bra.s  loc_4160                     ; same → keep settled
    if platform_standing_on == 0 || platform_standing_on != this_platform_addr {
        SoftPlatformSettledActionLoc4160::ReturnToWaitLoop
    } else {
        SoftPlatformSettledActionLoc4160::StaySettled
    }
}

/// Per-frame action for the TrapPlatformDown post-fall wait state.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrapPlatformDownPostFallActionLoc41EE {
    /// Platform not yet deactivated — continue waiting.
    ContinueWaiting,
    /// Deactivated flag set — deallocate slot and delete object.
    Delete,
}

/// TrapPlatformDown post-fall wait: after the platform has fallen off-screen
/// (platform Y - Kid screen Y >= $E0), velocity is cleared and the sub-pixel
/// field set to $800. This loop hibernates each frame and polls the
/// deactivation flag at `$12(a3)`.
#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn trap_platform_down_post_fall_step_loc_41EE(
    deactivated_flag: u8,
) -> TrapPlatformDownPostFallActionLoc41EE {
    // ASM loc_41EE (kid.asm:5304): jsr Hibernate, then:
    //   tst.b  $12(a3)
    //   beq.s  loc_41EE            ; zero → keep waiting
    //   ; nonzero → fall through to TrapPlatformDown_Delete
    if deactivated_flag != 0 {
        TrapPlatformDownPostFallActionLoc41EE::Delete
    } else {
        TrapPlatformDownPostFallActionLoc41EE::ContinueWaiting
    }
}

/// Per-frame result of the TrapPlatformUp move loop.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TrapPlatformUpMoveStepLoc4230 {
    /// Updated y velocity after deceleration (clamped to -$40000).
    pub(crate) new_y_velocity: i32,
    /// `true` while the platform is still rising (platform Y - Kid Y > -$E0).
    pub(crate) continue_moving: bool,
}

/// TrapPlatformUp move loop step: after hibernating and applying the current
/// velocity to the platform position (`TrapPlatformUp_Move`), this routine
/// decelerates upward velocity by $4000 per frame (clamped to -$40000) and
/// checks whether the platform has risen far enough above Kid to exit.
#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn trap_platform_up_move_step_loc_4230(
    y_velocity: i32,
    platform_y_px: i16,
    kid_y_px: i16,
) -> TrapPlatformUpMoveStepLoc4230 {
    // ASM loc_4230 (kid.asm:5341): after Hibernate + TrapPlatformUp_Move:
    //   move.l $E(a3),d7
    //   subi.l #$4000,d7
    //   cmpi.l #-$40000,d7
    //   bgt.w  +
    //   move.l #-$40000,d7
    // +
    //   move.l d7,$E(a3)          ; store clamped velocity
    //   move.w 6(a3),d7           ; platform y_pos word
    //   sub.w  $1E(a2),d7         ; subtract Kid y_pos word
    //   cmpi.w #-$E0,d7
    //   bgt.s  loc_4230           ; still close → keep moving
    let decelerated = y_velocity.wrapping_sub(0x4000);
    let clamped = if decelerated > -0x40000_i32 {
        decelerated
    } else {
        -0x40000_i32
    };
    let relative_y = platform_y_px.wrapping_sub(kid_y_px);
    TrapPlatformUpMoveStepLoc4230 {
        new_y_velocity: clamped,
        continue_moving: relative_y > -0xE0,
    }
}

/// Per-frame action for the TrapPlatformUp post-rise wait state.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrapPlatformUpPostRiseActionLoc4270 {
    /// Deactivation flag still zero — continue waiting.
    ContinueWaiting,
    /// Deactivated flag set — deallocate slot and delete object.
    Delete,
}

/// TrapPlatformUp post-rise wait: after the platform has risen far enough
/// above Kid (platform Y - Kid Y <= -$E0), velocity is cleared and the
/// sub-pixel field set to $800. This loop hibernates each frame and polls
/// the deactivation flag at `$12(a3)`.
#[cfg_attr(not(test), allow(dead_code))]
#[allow(non_snake_case)]
pub(crate) fn trap_platform_up_post_rise_step_loc_4270(
    deactivated_flag: u8,
) -> TrapPlatformUpPostRiseActionLoc4270 {
    // ASM loc_4270 (kid.asm:5358): jsr Hibernate, then:
    //   tst.b  $12(a3)
    //   beq.s  loc_4270            ; zero → keep waiting
    //   ; nonzero → fall through to TrapPlatformUp_Delete
    //   bsr.w  Deallocate_PlatformSlot
    //   jmp    Delete_CurrentObject(pc)
    if deactivated_flag != 0 {
        TrapPlatformUpPostRiseActionLoc4270::Delete
    } else {
        TrapPlatformUpPostRiseActionLoc4270::ContinueWaiting
    }
}
