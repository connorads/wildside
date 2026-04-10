use super::*;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActivePlatformHandleSub2728 {
    pub(crate) layout_offset_words: u16,
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
pub(crate) fn find_active_platform_by_layout_offset_sub_2728(
    layout_offset_words: u16,
    active_platforms: &[ActivePlatformHandleSub2728],
) -> Option<usize> {
    // ASM sub_2728 (kid.asm:4649): walk the active platform-slot linked list
    // in allocation order and stop on the first slot whose `$20(a2)` matches
    // the current PlatformLoader_Offset word.
    active_platforms
        .iter()
        .position(|platform| platform.layout_offset_words == layout_offset_words)
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

    // If player is already on ground, check if they're riding a platform and
    // carry them along with the platform's velocity.
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
