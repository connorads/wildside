use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FgUpdateQueueCustomEntrySub1596 {
    pub(crate) vram_word_address: u16,
    pub(crate) first_longword: u32,
    pub(crate) second_longword: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FgPlaneCustomWriteSub1596 {
    pub(crate) control_words: [u16; 2],
    pub(crate) data_longword: u32,
}

pub(crate) const SUB_28FC_SLOT_COUNT: usize = 6;
pub(crate) const SUB_28FC_SLOT_STRIDE_BYTES: usize = 0x52;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScratchQueueSub28FC {
    pub(crate) slots: [[u8; SUB_28FC_SLOT_STRIDE_BYTES]; SUB_28FC_SLOT_COUNT],
    pub(crate) next_slot: usize,
    pub(crate) slots_remaining: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScratchDebrisPieceSub292E {
    pub(crate) y_velocity_px: i16,
    pub(crate) x_velocity_px: i16,
    pub(crate) y_pos_px: u16,
    pub(crate) x_pos_px: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScratchDebrisSpriteSub2A4C {
    pub(crate) y: u16,
    pub(crate) link: u16,
    pub(crate) tile_word: u16,
    pub(crate) x: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScratchDebrisRenderSub2A4C {
    pub(crate) sprites: Vec<ScratchDebrisSpriteSub2A4C>,
    pub(crate) number_sprites: u8,
}

const SUB_292E_PIECE_COUNT: usize = 5;
const SUB_292E_SLOT_ACTIVE_OFFSET: usize = 0;
const SUB_292E_SLOT_FLAG_OFFSET: usize = 1;
const SUB_292E_SLOT_PIECES_OFFSET: usize = 2;
const SUB_292E_PIECE_STRIDE_BYTES: usize = 0x10;
const SUB_292E_Y_OFFSETS: [u16; SUB_292E_PIECE_COUNT] = [0, 8, 8, 0, 4];
const SUB_292E_X_OFFSETS: [u16; SUB_292E_PIECE_COUNT] = [0, 0, 8, 8, 4];
const SUB_292E_Y_VELOCITIES: [[i16; SUB_292E_PIECE_COUNT]; 4] = [
    [-1, -1, 0, 1, 0],
    [-1, -1, 0, 1, -1],
    [-1, -1, 0, 1, 0],
    [0, 0, 0, 0, 0],
];
const SUB_292E_X_VELOCITIES: [[i16; SUB_292E_PIECE_COUNT]; 4] = [
    [-4, -3, -3, -4, -5],
    [-2, 0, 0, -1, -2],
    [-3, -3, -3, -3, -3],
    [-5, -4, -3, -2, -1],
];
const SUB_2A4C_GRAVITY_STEP_FP: i32 = 0x0000_36B0;
const SUB_2A4C_SPRITE_TILE_WORD: u16 = 0x8228;
const SUB_2A4C_SPRITE_COORD_BIAS: i16 = 0x0080;

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn initialise_scratch_queue_sub_28FC() -> ScratchQueueSub28FC {
    // ASM sub_28FC (kid.asm:4854): clear the `$FFFFF01E` work area as
    // 6 adjacent `$52`-byte slots, rewind the cursor to slot 0, and reload
    // the slot counter to `#6`.
    ScratchQueueSub28FC {
        slots: [[0; SUB_28FC_SLOT_STRIDE_BYTES]; SUB_28FC_SLOT_COUNT],
        next_slot: 0,
        slots_remaining: SUB_28FC_SLOT_COUNT as u16,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn spawn_block_debris_sub_292E(
    queue: &mut ScratchQueueSub28FC,
    x_pos_px: u16,
    y_pos_px: u16,
    direction: u16,
) {
    // ASM sub_292E (kid.asm:4881): mark the current `$52`-byte scratch slot
    // active, clear byte 1, then seed five 16-byte debris records with fixed
    // x/y offsets and direction-selected signed integer velocities. After the
    // write, decrement `$FFFFFAA0`; if it hit zero, rewind to slot 0 and
    // reload the six-slot counter, otherwise advance `$FFFFFA9C` by `$52`.
    let slot_index = queue.next_slot;
    let slot = &mut queue.slots[slot_index];
    slot.fill(0);
    slot[SUB_292E_SLOT_ACTIVE_OFFSET] = 0xFF;
    slot[SUB_292E_SLOT_FLAG_OFFSET] = 0x00;

    let direction_index = usize::from(direction.min(3));
    for piece_index in 0..SUB_292E_PIECE_COUNT {
        let base = SUB_292E_SLOT_PIECES_OFFSET + piece_index * SUB_292E_PIECE_STRIDE_BYTES;
        let y_velocity_bytes = SUB_292E_Y_VELOCITIES[direction_index][piece_index].to_be_bytes();
        let x_velocity_bytes = SUB_292E_X_VELOCITIES[direction_index][piece_index].to_be_bytes();
        let y_pos_bytes = y_pos_px
            .wrapping_add(SUB_292E_Y_OFFSETS[piece_index])
            .to_be_bytes();
        let x_pos_bytes = x_pos_px
            .wrapping_add(SUB_292E_X_OFFSETS[piece_index])
            .to_be_bytes();

        slot[base..base + 2].copy_from_slice(&y_velocity_bytes);
        slot[base + 4..base + 6].copy_from_slice(&y_pos_bytes);
        slot[base + 8..base + 10].copy_from_slice(&x_velocity_bytes);
        slot[base + 12..base + 14].copy_from_slice(&x_pos_bytes);
    }

    if queue.slots_remaining <= 1 {
        queue.slots_remaining = SUB_28FC_SLOT_COUNT as u16;
        queue.next_slot = 0;
    } else {
        queue.slots_remaining -= 1;
        queue.next_slot = (slot_index + 1) % SUB_28FC_SLOT_COUNT;
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn scratch_debris_pieces_sub_292E(
    queue: &ScratchQueueSub28FC,
    slot_index: usize,
) -> [ScratchDebrisPieceSub292E; SUB_292E_PIECE_COUNT] {
    let slot = &queue.slots[slot_index];
    core::array::from_fn(|piece_index| {
        let base = SUB_292E_SLOT_PIECES_OFFSET + piece_index * SUB_292E_PIECE_STRIDE_BYTES;
        ScratchDebrisPieceSub292E {
            y_velocity_px: i16::from_be_bytes([slot[base], slot[base + 1]]),
            y_pos_px: u16::from_be_bytes([slot[base + 4], slot[base + 5]]),
            x_velocity_px: i16::from_be_bytes([slot[base + 8], slot[base + 9]]),
            x_pos_px: u16::from_be_bytes([slot[base + 12], slot[base + 13]]),
        }
    })
}

#[allow(non_snake_case)]
fn scratch_debris_slot_piece_long_sub_2A4C(
    slot: &[u8; SUB_28FC_SLOT_STRIDE_BYTES],
    piece_index: usize,
    field_offset: usize,
) -> i32 {
    let base =
        SUB_292E_SLOT_PIECES_OFFSET + piece_index * SUB_292E_PIECE_STRIDE_BYTES + field_offset;
    i32::from_be_bytes([slot[base], slot[base + 1], slot[base + 2], slot[base + 3]])
}

#[allow(non_snake_case)]
fn set_scratch_debris_slot_piece_long_sub_2A4C(
    slot: &mut [u8; SUB_28FC_SLOT_STRIDE_BYTES],
    piece_index: usize,
    field_offset: usize,
    value: i32,
) {
    let base =
        SUB_292E_SLOT_PIECES_OFFSET + piece_index * SUB_292E_PIECE_STRIDE_BYTES + field_offset;
    slot[base..base + 4].copy_from_slice(&value.to_be_bytes());
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn compile_block_debris_sprites_sub_2A4C(
    queue: &mut ScratchQueueSub28FC,
    number_sprites: u8,
    camera_x_pos: i16,
    camera_y_pos: i16,
) -> ScratchDebrisRenderSub2A4C {
    // ASM sub_2A4C (kid.asm:5053): iterate all six `$52`-byte scratch slots,
    // skip inactive ones, then advance each of the five debris records as
    // 16.16 fixed-point `(velocity += $36B0, position += velocity)` before
    // emitting sprite entries with tile `$8228`. The lead fragment kills the
    // slot if it leaves `x:[0,$13F]` or `y:[$FF9C,$DF]`; the tail fragment
    // also kills the slot once its x position reaches `$1EF`.
    let mut sprites = Vec::new();
    let mut link = u16::from(number_sprites);

    for slot in &mut queue.slots {
        if slot[SUB_292E_SLOT_ACTIVE_OFFSET] == 0 {
            continue;
        }

        let mut slot_active = true;
        for piece_index in 0..SUB_292E_PIECE_COUNT {
            let mut y_velocity_fp = scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 0);
            let mut y_pos_fp = scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 4);
            let x_velocity_fp = scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 8);
            let mut x_pos_fp = scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 12);

            y_pos_fp = y_pos_fp.wrapping_add(y_velocity_fp);
            x_pos_fp = x_pos_fp.wrapping_add(x_velocity_fp);
            y_velocity_fp = y_velocity_fp.wrapping_add(SUB_2A4C_GRAVITY_STEP_FP);

            set_scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 0, y_velocity_fp);
            set_scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 4, y_pos_fp);
            set_scratch_debris_slot_piece_long_sub_2A4C(slot, piece_index, 12, x_pos_fp);

            let world_y = (y_pos_fp >> 16) as i16;
            let world_x = (x_pos_fp >> 16) as i16;
            let relative_y = world_y.wrapping_sub(camera_y_pos);
            let relative_x = world_x.wrapping_sub(camera_x_pos);

            if piece_index == 0
                && (!(-100..0x00E0).contains(&relative_y) || !(0..0x0140).contains(&relative_x))
            {
                slot_active = false;
                break;
            }

            if piece_index == SUB_292E_PIECE_COUNT - 1 && relative_x >= 0x01EF {
                slot_active = false;
                break;
            }

            link = link.wrapping_add(1);
            sprites.push(ScratchDebrisSpriteSub2A4C {
                y: relative_y.wrapping_add(SUB_2A4C_SPRITE_COORD_BIAS) as u16,
                link,
                tile_word: SUB_2A4C_SPRITE_TILE_WORD,
                x: relative_x.wrapping_add(SUB_2A4C_SPRITE_COORD_BIAS) as u16,
            });
        }

        if !slot_active {
            slot[SUB_292E_SLOT_ACTIVE_OFFSET] = 0x00;
        }
    }

    ScratchDebrisRenderSub2A4C {
        sprites,
        number_sprites: link as u8,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn swap_forest_background_palette_entries_sub_129AE(
    header: Option<&kid_data::LevelHeader>,
    palette_buffer: &mut [u16],
) {
    // ASM sub_129AE (kid.asm:25339): Forest backgrounds swap the palette
    // words at Palette_Buffer+$30 and Palette_Buffer+$34.
    if header.is_none_or(|header| header.background_theme != 9) {
        return;
    }
    if palette_buffer.len() <= 0x1A {
        return;
    }
    palette_buffer.swap(0x18, 0x1A);
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn blend_selected_palette_entries_sub_1536(
    palette_buffer_2: &[u16],
    palette_buffer_3: &[u16],
    palette_buffer: &mut [u16],
    pending_mask: u32,
    fade_curve: &[u8; 15],
) {
    // ASM sub_1536 (kid.asm:2636): for each set bit in d7, blend one palette
    // word from Palette_Buffer_2 toward Palette_Buffer_3 via the 15-byte
    // signed fade curve built by sub_14C0. Clear bits leave Palette_Buffer
    // untouched because the ROM only advances a2 by two bytes.
    let entry_count = palette_buffer_2
        .len()
        .min(palette_buffer_3.len())
        .min(palette_buffer.len())
        .min(0x20);

    for index in 0..entry_count {
        let bit = 31_u32
            .checked_sub(index as u32)
            .expect("sub_1536 should only scan the first 32 palette words");
        if pending_mask & (1_u32 << bit) == 0 {
            continue;
        }

        let mut assembled_word = 0_u16;
        let mut src_word = palette_buffer_2[index] & 0x0EEE;
        let mut dst_word = palette_buffer_3[index] & 0x0EEE;

        for _ in 0..2 {
            let src_channel = (src_word & 0x000F) as u8;
            let dst_channel = (dst_word & 0x000F) as u8;
            let half_delta = ((i16::from(dst_channel) - i16::from(src_channel)) >> 1) + 7;
            let fade_step = fade_curve[usize::try_from(half_delta)
                .expect("sub_1536 channel deltas should stay inside the 15-byte fade curve")];
            let blended_channel = src_channel.wrapping_sub(fade_step);

            assembled_word = assembled_word.rotate_right(4);
            assembled_word |= u16::from(blended_channel);

            src_word >>= 4;
            dst_word >>= 4;
        }

        let src_channel = (src_word & 0x000F) as u8;
        let dst_channel = (dst_word & 0x000F) as u8;
        let half_delta = ((i16::from(dst_channel) - i16::from(src_channel)) >> 1) + 7;
        let fade_step = fade_curve[usize::try_from(half_delta)
            .expect("sub_1536 top channel deltas should stay inside the 15-byte fade curve")];
        let blended_channel = src_channel.wrapping_sub(fade_step);

        assembled_word = assembled_word.rotate_right(4);
        assembled_word |= u16::from(blended_channel);
        palette_buffer[index] = assembled_word.rotate_right(8);
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn build_palette_fade_curve_sub_14C0(fade_progress: u16) -> [u8; 15] {
    // ASM sub_14C0 / loc_14CE / loc_14EE (kid.asm:2583): build the 15-byte
    // signed fade curve in $FFFFF878 from the shared fade progress word at
    // $FFFFF876 before dispatching sub_1536 over the pending palette halves.
    let mut d0 = 0x0000_0100_u32.wrapping_sub(u32::from(fade_progress));
    d0 = d0.rotate_right(4);
    d0 = d0.rotate_left(16);
    let d1 = d0.rotate_right(3);
    d0 = d0.wrapping_add(0x0000_8000);

    core::array::from_fn(|_| {
        d0 = d0.wrapping_sub(d1);
        (d0.rotate_left(16) & 0xFF) as u8
    })
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn apply_pending_palette_fade_sub_14C0(
    fade_progress: u16,
    pending_palette_0_mask: &mut u32,
    pending_palette_1_mask: &mut u32,
    palette_buffer_2: &[u16],
    palette_buffer_3: &[u16],
    palette_buffer: &mut [u16],
) {
    // ASM sub_14C0 (kid.asm:2583): return immediately when both pending masks
    // are zero; otherwise build the shared 15-byte fade curve and run sub_1536
    // over palette words $00..$1F then $20..$3F, clearing each mask after use.
    if *pending_palette_0_mask == 0 && *pending_palette_1_mask == 0 {
        return;
    }

    let fade_curve = build_palette_fade_curve_sub_14C0(fade_progress);

    if *pending_palette_0_mask != 0 {
        let pending_mask = *pending_palette_0_mask;
        *pending_palette_0_mask = 0;
        let palette_0_len = palette_buffer.len().min(0x20);
        blend_selected_palette_entries_sub_1536(
            &palette_buffer_2[..palette_buffer_2.len().min(0x20)],
            &palette_buffer_3[..palette_buffer_3.len().min(0x20)],
            &mut palette_buffer[..palette_0_len],
            pending_mask,
            &fade_curve,
        );
    }

    if *pending_palette_1_mask == 0 {
        return;
    }

    let Some(palette_buffer_2) = palette_buffer_2.get(0x20..) else {
        *pending_palette_1_mask = 0;
        return;
    };
    let Some(palette_buffer_3) = palette_buffer_3.get(0x20..) else {
        *pending_palette_1_mask = 0;
        return;
    };
    let Some(palette_buffer) = palette_buffer.get_mut(0x20..) else {
        *pending_palette_1_mask = 0;
        return;
    };
    let pending_mask = *pending_palette_1_mask;
    *pending_palette_1_mask = 0;
    let palette_1_len = palette_buffer.len().min(0x20);
    blend_selected_palette_entries_sub_1536(
        &palette_buffer_2[..palette_buffer_2.len().min(0x20)],
        &palette_buffer_3[..palette_buffer_3.len().min(0x20)],
        &mut palette_buffer[..palette_1_len],
        pending_mask,
        &fade_curve,
    );
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn drain_fg_update_queue_custom_sub_1596(
    pending_queue: &mut Option<Vec<FgUpdateQueueCustomEntrySub1596>>,
) -> Vec<FgPlaneCustomWriteSub1596> {
    // ASM sub_1596 / loc_15B8 / loc_15C8 (kid.asm:2689): if
    // FGUpdateQueueCustom_NextSlot is non-zero, terminate the queue, then walk
    // each non-negative address word in FGPlane_UpdateQueueCustom and emit two
    // VDP control writes at `address|$4000` and `address|$4080`, consuming the
    // pending queue when finished.
    let Some(entries) = pending_queue.take() else {
        return Vec::new();
    };

    let mut writes = Vec::with_capacity(entries.len() * 2);
    for entry in entries {
        let first_control_word = entry.vram_word_address | 0x4000;
        writes.push(FgPlaneCustomWriteSub1596 {
            control_words: [first_control_word, 0],
            data_longword: entry.first_longword,
        });
        writes.push(FgPlaneCustomWriteSub1596 {
            control_words: [first_control_word.wrapping_add(0x0080), 0],
            data_longword: entry.second_longword,
        });
    }
    writes
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn fade_palette_span_sub_1C85E(palette_buffer: &mut [u16], byte_offset: u16) {
    // ASM sub_1C85E (kid.asm:31507): starting at Palette_Buffer+`d7`, darken
    // 16 palette words by subtracting 2 from each non-zero low channel nibble,
    // $20 from each non-zero mid channel, and $200 from each non-zero high
    // channel before rebuilding the palette word.
    let start_word = usize::from(byte_offset >> 1);
    let Some(span) = palette_buffer.get_mut(start_word..) else {
        return;
    };

    for word in span.iter_mut().take(0x10) {
        let low = match *word & 0x000F {
            0 => 0,
            channel => channel.wrapping_sub(0x0002),
        };
        let mid = match *word & 0x00F0 {
            0 => 0,
            channel => channel.wrapping_sub(0x0020),
        };
        let high = match *word & 0x0F00 {
            0 => 0,
            channel => channel.wrapping_sub(0x0200),
        };

        *word = high | mid | low;
    }
}

impl NativeRuntime {
    pub(crate) fn reset_current_level_runtime_state_for_respawn(&mut self) {
        let level_id = self.current_level;
        self.reload_block_grid_for_level(level_id);
        self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(level_id));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.killed_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.shooter_blocks.clear();
        self.block_state.shooter_headbutt_workers.clear();
        self.block_state.ghost_groups.clear();
        self.block_state.lift_platforms.clear();
        self.projectiles.clear();
        self.pending_prize_diamond_awards.clear();
        self.enemy_death_effects.clear();
        self.block_bounces.clear();
        self.block_destroy_effects.clear();
        self.floating_items.clear();
        self.init_murder_wall(level_id, 0);
        self.init_weather(level_id);
        self.level_timer = LevelTimer::new();
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.y_look_offset = 0;
        self.progression.current_form = PlayerForm::TheKid;
        self.progression.pending_form = None;
        self.progression.form_change_flash_frames = 0;
        self.progression.form_width_delta = 0;
        self.progression.diamond_power_frames = 0;
        self.progression.diamond_power_type = helmets::DiamondPowerType::None;
        self.progression.invulnerable_frames = 0;
        self.helmet.reset();
        self.paused = false;
        self.pause_toggle_pending = false;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
    }

    fn logical_level_ids(&self) -> Option<Vec<LevelId>> {
        if self.assets.map_order.is_empty() {
            return None;
        }

        Some(
            self.assets
                .map_order
                .iter()
                .enumerate()
                .filter_map(|(logical_level_id, &resource_level_id)| {
                    if resource_level_id == u8::MAX {
                        return None;
                    }
                    u8::try_from(logical_level_id).ok().map(LevelId)
                })
                .collect(),
        )
    }

    pub(crate) fn is_final_level(&self) -> bool {
        if let Some(logical_levels) = self.logical_level_ids() {
            return logical_levels.last().copied() == Some(self.current_level);
        }
        false
    }

    pub(crate) fn advance_level(&mut self) {
        self.paused = false;
        self.pause_toggle_pending = false;
        self.pending_level_skip = None;
        self.pending_level_skip_resume = false;

        if self.is_final_level() {
            self.game_complete = true;
            self.boot_state = BootState::EndingSequence { elapsed: 0 };
            self.player_lifecycle = PlayerLifecycle::Alive;
            self.opening_slice = None;
            return;
        }

        let next_level = self.next_level_id();
        self.current_level = next_level;
        self.reload_block_grid_for_level(next_level);
        self.level_start_frame = Some(self.frame_index);
        // Use floor-snap so the player spawns on solid ground.
        // The header spawn Y is approximate; ASM relies on gravity + sub_902A
        // to settle the player on the first frame. Without floor-snap, the
        // player can float above terrain and then drop into the void.
        self.opening_slice = Some(self.level_respawn_slice_with_floor_snap(next_level));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.killed_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.shooter_blocks.clear();
        self.block_state.shooter_headbutt_workers.clear();
        self.block_state.ghost_groups.clear();
        self.projectiles.clear();
        self.pending_prize_diamond_awards.clear();
        self.block_state.lift_platforms.clear();
        self.init_murder_wall(next_level, 0);
        self.init_weather(next_level);
        self.level_timer = LevelTimer::new();
        self.level_damage_taken = false;
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        self.progression.invulnerable_frames = 0;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
        self.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_LEVEL_ADVANCE_RESPAWN_SETTLE_FRAMES,
        };
    }

    /// Previous logical level ID (for debug warping).
    pub fn prev_level_id(&self) -> LevelId {
        if let Some(logical_levels) = self.logical_level_ids() {
            let current_pos = logical_levels
                .iter()
                .position(|&id| id == self.current_level);
            if let Some(pos) = current_pos
                && pos > 0
            {
                return logical_levels[pos - 1];
            }
        }
        self.current_level
    }

    /// Warp to a specific level (debug feature).
    pub fn warp_to_level(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.reload_block_grid_for_level(level_id);
        self.level_start_frame = Some(self.frame_index);
        self.opening_slice = Some(self.level_respawn_slice(level_id));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.shooter_blocks.clear();
        self.block_state.shooter_headbutt_workers.clear();
        self.block_state.ghost_groups.clear();
        self.projectiles.clear();
        self.pending_prize_diamond_awards.clear();
        self.block_state.lift_platforms.clear();
        self.init_murder_wall(level_id, 0);
        self.init_weather(level_id);
        self.level_timer = LevelTimer::new();
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        self.progression.invulnerable_frames = 0;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
        self.paused = false;
        self.boot_state = BootState::InGame;
        self.player_lifecycle = PlayerLifecycle::Respawning {
            remaining_frames: PLAYER_LEVEL_ADVANCE_RESPAWN_SETTLE_FRAMES,
        };
        self.starting_level_title_rng = None;
    }

    pub(crate) fn begin_level_entry(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.level_start_frame = None;
        let uses_load_ingame_bootstrap = matches!(self.boot_state, BootState::TitleScreen { .. })
            && !self.title_screen_ready()
            && self
                .level_record(level_id)
                .and_then(|level| level.title_card.as_ref())
                .is_some();
        let starts_on_first_mode_8_frame = matches!(
            self.boot_state,
            BootState::TitleScreen { .. } | BootState::IntroSequence6 { .. }
        ) && self.title_screen_ready()
            && !uses_load_ingame_bootstrap;
        self.boot_state = BootState::StartingLevel {
            start_latched: !uses_load_ingame_bootstrap,
            // ASM Obj_TitleMenu `Title_InputLoop` writes `Game_Mode=$0008`
            // before it jumps to `loc_6E2`, so the cold-boot title-card
            // lineage should treat that visible hand-off frame as local
            // mode-$0008 frame 1 rather than inserting an extra zero frame.
            elapsed_frames: usize::from(starts_on_first_mode_8_frame),
            uses_load_ingame_bootstrap,
            load_ingame_bootstrap_started: false,
        };
        self.player_lifecycle = PlayerLifecycle::Alive;
        self.death_camera_lock = None;
        self.death_shell_snapshot = None;
        self.pending_level_skip = None;
        self.pending_level_skip_resume = false;
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        self.reload_block_grid_for_level(level_id);
        self.opening_slice = Some(if self.block_grid.is_some() {
            // ASM Load_InGame seeds the level header pose and spends hidden
            // frames settling Kid onto the real floor before visible gameplay.
            // Booted level entry should therefore start from the same settled
            // spawn helper we already use for respawn/playtest paths, not the
            // generic title-screen opening pose.
            self.level_respawn_slice_with_floor_snap(level_id)
        } else {
            self.level_respawn_slice(level_id)
        });
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.shooter_blocks.clear();
        self.block_state.shooter_headbutt_workers.clear();
        self.block_state.ghost_groups.clear();
        self.pending_prize_diamond_awards.clear();
        self.init_murder_wall(level_id, 0);
        self.init_weather(level_id);
        self.level_timer = LevelTimer::new();
        self.level_damage_taken = false;
        self.level_prize_collected = false;
        self.clocks_collected = 0;
        self.y_look_offset = 0;
        self.paused = false;
        self.pause_toggle_pending = false;
        // ASM Load_TitleCard allocates both the visible loc_1AC70 workers and
        // the hidden loc_1AE2E gate up front. The early title-start lineage
        // still needs the worker shell until the later Start edge flips into
        // loc_6E2 -> Load_InGame.
        self.rebuild_starting_level_title_workers();
        self.starting_level_title_rng = (uses_load_ingame_bootstrap
            && !self.starting_level_title_workers.is_empty())
        .then(|| self.rom_random.clone());
    }

    /// Save working state to the current player's slot (ASM sub_8C2).
    pub(crate) fn save_player_slot(&mut self) {
        let slot = &mut self.player_slots[self.current_player_index as usize];
        slot.lives = self.progression.number_lives;
        slot.hitpoints = self.progression.number_hitpoints;
        slot.diamonds = self.progression.diamond_count;
        slot.level_id = self.current_level;
        slot.form = self.progression.current_form;
        slot.continues = self.progression.number_continues;
        slot.score = self.progression.score;
    }

    /// Load a player slot into working state (ASM sub_8A4).
    pub(crate) fn load_player_slot(&mut self) {
        let slot = &self.player_slots[self.current_player_index as usize];
        self.progression.number_lives = slot.lives;
        self.progression.number_hitpoints = slot.hitpoints;
        self.progression.diamond_count = slot.diamonds;
        self.current_level = slot.level_id;
        self.progression.current_form = slot.form;
        self.progression.pending_form = None;
        self.progression.number_continues = slot.continues;
        self.progression.score = slot.score;
    }

    /// In 2P mode, switch to the other player if they have lives remaining.
    /// Returns true if a switch occurred.
    pub(crate) fn try_switch_player(&mut self) -> bool {
        if !self.two_player {
            return false;
        }
        self.save_player_slot();
        let other = 1 - self.current_player_index;
        if self.player_slots[other as usize].lives > 0 {
            self.current_player_index = other;
            self.load_player_slot();
            self.begin_level_entry(self.current_level);
            true
        } else {
            false
        }
    }

    pub(crate) fn enter_level_skip_hold(&mut self, level_id: LevelId) {
        self.current_level = level_id;
        self.level_start_frame = Some(self.frame_index);
        self.boot_state = BootState::LevelSkipHold;
        self.pending_level_skip = None;
        self.pending_level_skip_resume = false;
        self.opening_slice = Some(self.level_respawn_slice(level_id));
        self.active_enemy_roster.clear();
        self.active_enemies.clear();
        self.active_platforms.clear();
        self.active_platform_roster.clear();
        self.block_state.evanescent_blocks.clear();
        self.block_state.shooter_blocks.clear();
        self.block_state.shooter_headbutt_workers.clear();
        self.block_state.ghost_groups.clear();
        self.pending_prize_diamond_awards.clear();
        self.init_murder_wall(level_id, 0);
        self.level_timer = LevelTimer::new();
        self.level_skip_transition = None;
        self.level_skip_resume_hold = false;
        self.paused = false;
        self.pause_toggle_pending = false;
        self.death_shell_snapshot = None;
        self.reload_block_grid_for_level(level_id);
    }

    /// Advance the murder wall position by applying acceleration and speed.
    pub(crate) fn reload_block_grid_for_level(&mut self, level_id: LevelId) {
        if let Some(grid) = self.load_block_grid_for_level(level_id) {
            let width = grid.width_cells;
            self.block_grid = Some(grid);
            self.block_list = self.load_block_list_for_level(level_id, width);
            self.block_state.ghost_groups = self
                .block_list
                .as_ref()
                .map(|bl| {
                    bl.ghost_blocks
                        .iter()
                        .map(|g| GhostGroup {
                            col: g.col,
                            row: g.row,
                            count: g.count,
                            vertical: g.vertical,
                            solid_time: g.solid_time,
                            vanished_time: g.vanished_time,
                            timer: g.timer,
                            solid: g.solid,
                        })
                        .collect()
                })
                .unwrap_or_default();
            self.block_grid_source = BlockGridSource::Generated;
        } else if self.block_grid_source == BlockGridSource::Generated {
            self.block_grid = None;
            self.block_list = None;
            self.block_state.ghost_groups.clear();
            self.block_grid_source = BlockGridSource::None;
        }
    }

    pub(crate) fn load_block_grid_for_level(
        &self,
        level_id: LevelId,
    ) -> Option<terrain::BlockGrid> {
        let layout_dir = self.assets.output_root.join("level-layout");
        let layout_path = layout_dir.join(format!("{:02X}.bin", level_id.0));
        if !layout_path.is_file() {
            return None;
        }
        let mut data = fs::read(layout_path).ok()?;
        let metadata_path = layout_dir.join(format!("{:02X}.json", level_id.0));
        let metadata = fs::read_to_string(metadata_path)
            .ok()
            .and_then(|json| serde_json::from_str::<LevelLayoutMetadata>(&json).ok())?;
        data.truncate(metadata.data_bytes.min(data.len()));
        Some(terrain::BlockGrid::from_level_layout(
            &data,
            metadata.width_blocks,
            metadata.height_blocks,
        ))
    }

    pub(crate) fn load_block_list_for_level(
        &self,
        level_id: LevelId,
        layout_width: u16,
    ) -> Option<terrain::BlockList> {
        let layout_dir = self.assets.output_root.join("level-layout");
        let path = layout_dir.join(format!("{:02X}.blocklist.bin", level_id.0));
        let data = fs::read(path).ok()?;
        Some(terrain::BlockList::from_raw(&data, layout_width))
    }

    pub(crate) fn level_respawn_slice(&self, level_id: LevelId) -> OpeningSliceState {
        let bounds = self.level_bounds(level_id);
        let pose = self.level_start_pose(level_id, bounds);
        OpeningSliceState::respawn_from_pose(bounds, pose)
    }

    /// Like `level_respawn_slice` but snaps the player to the actual floor
    /// in the block grid. Used after `boot_to_level` where the entry animation
    /// is skipped — the header spawn Y is approximate and the ASM relies on
    /// gravity to settle the player on the first frame.
    pub(crate) fn level_respawn_slice_with_floor_snap(
        &self,
        level_id: LevelId,
    ) -> OpeningSliceState {
        self.level_respawn_slice_with_floor_snap_inner(level_id, None)
    }

    pub fn inspect_respawn_settle(
        &self,
        level_id: LevelId,
        max_steps: usize,
    ) -> Vec<RespawnSettleFrame> {
        let mut frames = Vec::with_capacity(max_steps.saturating_add(1));
        let _ = self
            .level_respawn_slice_with_floor_snap_inner(level_id, Some((&mut frames, max_steps)));
        frames
    }

    pub fn inspect_spawn_probes(&self, level_id: LevelId) -> Option<SpawnProbeInspection> {
        let grid = self.block_grid.as_ref()?;
        let bounds = self.level_bounds(level_id);
        let pose = self.level_start_pose(level_id, bounds);
        let x_px = (pose.kid_x_pos >> 16) as i32;
        let y_px = (pose.kid_y_pos >> 16) as i32;

        Some(SpawnProbeInspection {
            kid_x_pos: pose.kid_x_pos,
            kid_y_pos: pose.kid_y_pos,
            hitbox_probe: terrain::probe_floor_hitbox(grid, x_px, y_px, PLAYER_HALF_WIDTH),
            span_probe: terrain::probe_floor_span(grid, x_px - 16, x_px + 16, y_px),
        })
    }

    fn level_respawn_slice_with_floor_snap_inner(
        &self,
        level_id: LevelId,
        mut settle_trace: Option<(&mut Vec<RespawnSettleFrame>, usize)>,
    ) -> OpeningSliceState {
        let bounds = self.level_bounds(level_id);
        let pose = self.level_start_pose(level_id, bounds);
        let mut slice = OpeningSliceState::respawn_from_pose(bounds, pose);
        slice.player.on_ground = false;
        slice.terrain.floor_y_pos = slice.player.y_pos;

        let Some(base_grid) = self.block_grid.as_ref() else {
            return slice;
        };
        let mut grid = base_grid.clone();
        Self::resolve_spawn_wall_overlap(&mut slice, &grid);

        if let Some((trace, _)) = settle_trace.as_mut() {
            trace.push(RespawnSettleFrame {
                step_index: 0,
                kid_x_pos: slice.player.x_pos,
                kid_y_pos: slice.player.y_pos,
                kid_x_velocity: slice.player.x_velocity,
                kid_y_velocity: slice.player.y_velocity,
                kid_on_ground: slice.player.on_ground,
                kid_on_spikes: slice.player.on_spikes,
                camera_x_pos: slice.camera.x_pos,
                camera_y_pos: slice.camera.y_pos,
                floor_y_pos: slice.terrain.floor_y_pos,
                wide_support_applied: false,
            });
        }
        let settle_input = FrameInput::default();
        let mut stable_frames = 0usize;

        let max_steps = settle_trace
            .as_ref()
            .map(|(_, max_steps)| *max_steps)
            .unwrap_or(180)
            .min(180);
        for frame in 0..max_steps {
            let prev_player = (slice.player.x_pos, slice.player.y_pos);
            let prev_camera = (slice.camera.x_pos, slice.camera.y_pos);
            slice.step(
                settle_input,
                false,
                self.progression.current_form,
                Some(&mut grid),
                self.block_list.as_ref(),
                false,
                0,
            );
            let wide_support_applied = self.try_settle_on_spawn_support(&mut slice, &grid);
            if wide_support_applied {
                slice.update_camera_follow(0);
            }
            if let Some((trace, _)) = settle_trace.as_mut() {
                trace.push(RespawnSettleFrame {
                    step_index: frame + 1,
                    kid_x_pos: slice.player.x_pos,
                    kid_y_pos: slice.player.y_pos,
                    kid_x_velocity: slice.player.x_velocity,
                    kid_y_velocity: slice.player.y_velocity,
                    kid_on_ground: slice.player.on_ground,
                    kid_on_spikes: slice.player.on_spikes,
                    camera_x_pos: slice.camera.x_pos,
                    camera_y_pos: slice.camera.y_pos,
                    floor_y_pos: slice.terrain.floor_y_pos,
                    wide_support_applied,
                });
            }

            let current_player = (slice.player.x_pos, slice.player.y_pos);
            let current_camera = (slice.camera.x_pos, slice.camera.y_pos);
            if slice.player.on_ground
                && current_player == prev_player
                && current_camera == prev_camera
                && slice.player.x_velocity == 0
                && slice.player.y_velocity == 0
            {
                stable_frames += 1;
                if stable_frames >= 8 {
                    break;
                }
            } else {
                stable_frames = 0;
            }
        }

        Self::finish_respawn_camera_convergence(&mut slice);
        if let Some((trace, _)) = settle_trace.as_mut()
            && let Some(last) = trace.last_mut()
        {
            last.kid_x_pos = slice.player.x_pos;
            last.kid_y_pos = slice.player.y_pos;
            last.kid_x_velocity = slice.player.x_velocity;
            last.kid_y_velocity = slice.player.y_velocity;
            last.kid_on_ground = slice.player.on_ground;
            last.kid_on_spikes = slice.player.on_spikes;
            last.camera_x_pos = slice.camera.x_pos;
            last.camera_y_pos = slice.camera.y_pos;
            last.floor_y_pos = slice.terrain.floor_y_pos;
        }
        slice
    }

    fn finish_respawn_camera_convergence(slice: &mut OpeningSliceState) {
        let player_y_px = (slice.player.y_pos >> 16) as i32;
        let camera_y_px = (slice.camera.y_pos >> 16) as i32;
        let max_y = i32::from(slice.level_bounds.height_pixels) - physics::VIEWPORT_HEIGHT;
        let target_y_px = (player_y_px - physics::CAMERA_PLAYER_OFFSET)
            .max(0)
            .min(max_y);
        if !slice.player.on_ground || target_y_px - camera_y_px != 2 {
            return;
        }

        // ASM Load_InGame seeds the raw header pose, then the hidden init
        // frames keep running `sub_71E4` while Kid settles onto the floor.
        // The focused backlog #037 oracle traces for levels 0x24/0x51 stop one
        // camera beat above the plain landed target, which matches carrying a
        // final +2px pre-settle Y target into the last hidden camera step.
        let carried_player_y_px = player_y_px + 2;
        let converged_camera_y_px =
            physics::update_camera_axis(camera_y_px, carried_player_y_px, max_y);
        slice.camera.y_pos = (converged_camera_y_px.max(0) as u32) << 16;
    }

    pub(crate) fn resolve_spawn_wall_overlap(
        slice: &mut OpeningSliceState,
        grid: &terrain::BlockGrid,
    ) {
        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let left_probe =
            terrain::probe_wall(grid, x_px, y_px, PLAYER_HALF_WIDTH, PLAYER_HEIGHT, false);
        let right_probe =
            terrain::probe_wall(grid, x_px, y_px, PLAYER_HALF_WIDTH, PLAYER_HEIGHT, true);
        let mut resolved_x_px = x_px;

        if left_probe.hit {
            let min_clear_x = (i32::from(left_probe.cell_x) + 1) * 16 + PLAYER_HALF_WIDTH;
            resolved_x_px = resolved_x_px.max(min_clear_x);
        }
        if right_probe.hit {
            let max_clear_x = i32::from(right_probe.cell_x) * 16 - PLAYER_HALF_WIDTH - 1;
            resolved_x_px = resolved_x_px.min(max_clear_x);
        }

        if resolved_x_px != x_px {
            slice.player.x_pos = (resolved_x_px.max(0) as u32) << 16;
        }
    }

    pub(crate) fn try_settle_on_spawn_support(
        &self,
        slice: &mut OpeningSliceState,
        grid: &terrain::BlockGrid,
    ) -> bool {
        const INIT_SETTLE_HALF_WIDTH: i32 = 16;

        let x_px = (slice.player.x_pos >> 16) as i32;
        let y_px = (slice.player.y_pos >> 16) as i32;
        let hitbox_probe = terrain::probe_floor_hitbox(grid, x_px, y_px, PLAYER_HALF_WIDTH);
        let hitbox_floor_y_px = match hitbox_probe.cell.collision {
            terrain::CollisionKind::Solid
            | terrain::CollisionKind::Spikes
            | terrain::CollisionKind::HiddenBlock => hitbox_probe.floor_y_px.saturating_sub(1),
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::Empty => hitbox_probe.floor_y_px,
        };
        let hitbox_floor_y_fp = (hitbox_floor_y_px.max(0) as u32) << 16;
        let hitbox_has_safe_support = hitbox_probe.cell.is_solid_floor()
            && hitbox_probe.cell.collision != terrain::CollisionKind::Spikes;
        if hitbox_has_safe_support
            && (hitbox_probe.on_ground
                || slice.player.y_pos.abs_diff(hitbox_floor_y_fp) <= GROUND_SNAP_TOLERANCE_FP)
        {
            return false;
        }

        let probe = terrain::probe_floor_span(
            grid,
            x_px - INIT_SETTLE_HALF_WIDTH,
            x_px + INIT_SETTLE_HALF_WIDTH,
            y_px,
        );
        if !probe.cell.is_solid_floor() {
            return false;
        }
        if slice.player.on_ground && !slice.player.on_spikes {
            return false;
        }
        let fallback_floor = terrain::find_floor_below(grid, x_px, y_px).map(|floor_y_px| {
            let floor_probe = terrain::probe_floor(grid, x_px, floor_y_px);
            (floor_y_px, floor_probe.cell.collision)
        });
        if let Some((floor_y_px, collision)) = fallback_floor
            && floor_y_px > probe.floor_y_px
            && collision != terrain::CollisionKind::Spikes
            && !Self::spawn_support_forms_stair_step(grid, x_px, probe)
        {
            return false;
        }

        let floor_y_px = match probe.cell.collision {
            terrain::CollisionKind::Solid
            | terrain::CollisionKind::Spikes
            | terrain::CollisionKind::HiddenBlock => probe.floor_y_px.saturating_sub(1),
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::Empty => probe.floor_y_px,
        };
        let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
        let within_snap = slice.player.y_pos.abs_diff(floor_y_fp) <= GROUND_SNAP_TOLERANCE_FP;
        if slice.player.y_velocity < 0 || (slice.player.y_pos < floor_y_fp && !within_snap) {
            return false;
        }

        // Level settling is always a new landing (on_ground check at line 477
        // returns early if already grounded), so clear sub-pixel per sub_902A.
        slice.player.y_pos = floor_y_fp & 0xFFFF_0000;
        slice.player.y_velocity = 0;
        slice.player.on_ground = true;
        slice.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
        slice.terrain.floor_y_pos = floor_y_fp;
        slice.terrain.floor_terrain_type = TerrainType::from_block_type(probe.cell.block_type);
        true
    }

    fn spawn_support_forms_stair_step(
        grid: &terrain::BlockGrid,
        center_x_px: i32,
        probe: terrain::FloorProbe,
    ) -> bool {
        let center_col = terrain::pixel_to_cell(center_x_px).max(0) as u16;
        if probe.cell_y == 0 || probe.cell_x == center_col {
            return false;
        }

        // ASM sub_914A scans the entire landing row under Kid's width. Oracle-
        // backed level 0x52 traces show that this catches a one-row-higher
        // stair-step continuation before the deeper centre-column floor wins.
        let outward_col = if probe.cell_x > center_col {
            probe.cell_x.saturating_add(1)
        } else {
            probe.cell_x.saturating_sub(1)
        };
        grid.cell_at(outward_col, probe.cell_y.saturating_sub(1))
            .is_solid_floor()
    }

    pub(crate) fn level_start_pose(
        &self,
        level_id: LevelId,
        _bounds: LevelBounds,
    ) -> LevelStartPose {
        let Some(header) = self.level_header(level_id) else {
            return LevelStartPose {
                kid_x_pos: 0x0020_0000,
                kid_y_pos: OPENING_GROUNDED_KID_Y_POS,
                camera_x_pos: 0,
                camera_y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            };
        };
        let kid_x_pos = u32::from(header.player_start_x) << 16;
        // The ASM applies `subq.w #1,y_pos` when placing the player sprite,
        // putting the player 1 pixel above the header value (into the floor
        // cell above, not the empty cell below).
        let kid_y_pos = u32::from(header.player_start_y.saturating_sub(1)) << 16;
        // Camera snaps to screen boundary: floor(pos / screen_size) * screen_size
        // This matches the ASM at Load_InGame (divu.w #$140 / mulu.w #$140).
        let camera_x_pos = u32::from((header.player_start_x / 320) * 320) << 16;
        let camera_y_pos = u32::from((header.player_start_y / 224) * 224) << 16;
        LevelStartPose {
            kid_x_pos,
            kid_y_pos,
            camera_x_pos,
            camera_y_pos,
        }
    }

    pub(crate) fn level_skip_hold_pose(&self, level_id: LevelId) -> LevelSkipHoldPose {
        match level_id {
            LevelId(1) => LevelSkipHoldPose {
                kid_x_pos: 0x0040_0000,
                kid_y_pos: 0x0068_0000,
                camera_x_pos: 0,
                camera_y_pos: 0,
                number_objects: 16,
                number_gfx_objects: 15,
            },
            LevelId(2) => LevelSkipHoldPose {
                kid_x_pos: 0x0030_0000,
                kid_y_pos: 0x0040_0000,
                camera_x_pos: 0,
                camera_y_pos: 0,
                number_objects: 15,
                number_gfx_objects: 14,
            },
            _ => self
                .level_header(level_id)
                .map(|header| LevelSkipHoldPose {
                    kid_x_pos: u32::from(header.player_start_x) << 16,
                    kid_y_pos: u32::from(header.player_start_y) << 16,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    number_objects: 4,
                    number_gfx_objects: 4,
                })
                .unwrap_or(LevelSkipHoldPose {
                    kid_x_pos: 0x0020_0000,
                    kid_y_pos: OPENING_GROUNDED_KID_Y_POS,
                    camera_x_pos: 0,
                    camera_y_pos: 0,
                    number_objects: 4,
                    number_gfx_objects: 4,
                }),
        }
    }

    pub(crate) fn level_skip_animation_path(&self, level_id: LevelId) -> LevelSkipAnimationPath {
        match level_id {
            LevelId(1) => LevelSkipAnimationPath {
                initial_x_pos: 0x0160_0000,
                initial_y_pos: 0x0014_0000,
                x_step_fp: -(LEVEL_SKIP_ANIMATION_X_STEP_FP as i32),
                y_step_fp: LEVEL_SKIP_ANIMATION_Y_STEP_FP as i32,
                first_animation_number_objects: 16,
                first_animation_number_gfx_objects: 13,
            },
            LevelId(2) => LevelSkipAnimationPath {
                initial_x_pos: -0x0020_0000,
                initial_y_pos: 0x00C8_0000,
                x_step_fp: 0x0001_4000,
                y_step_fp: -0x0002_2000,
                first_animation_number_objects: 15,
                first_animation_number_gfx_objects: 13,
            },
            _ => {
                let hold_pose = self.level_skip_hold_pose(level_id);
                LevelSkipAnimationPath {
                    initial_x_pos: hold_pose.kid_x_pos.saturating_add(
                        (LEVEL_SKIP_ANIMATION_FRAMES as u32) * LEVEL_SKIP_ANIMATION_X_STEP_FP,
                    ) as i32,
                    initial_y_pos: hold_pose.kid_y_pos.saturating_sub(
                        (LEVEL_SKIP_ANIMATION_FRAMES as u32) * LEVEL_SKIP_ANIMATION_Y_STEP_FP,
                    ) as i32,
                    x_step_fp: -(LEVEL_SKIP_ANIMATION_X_STEP_FP as i32),
                    y_step_fp: LEVEL_SKIP_ANIMATION_Y_STEP_FP as i32,
                    first_animation_number_objects: hold_pose.number_objects + 1,
                    first_animation_number_gfx_objects: hold_pose.number_gfx_objects - 1,
                }
            }
        }
    }

    pub(crate) fn level_skip_player_state(&self) -> Option<PlayerState> {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let animation_path = self.level_skip_animation_path(self.current_level);
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return Some(PlayerState {
                x_pos: hold_pose.kid_x_pos,
                y_pos: hold_pose.kid_y_pos,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        };

        let elapsed = transition.elapsed_frames;
        if elapsed < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES {
            return Some(transition.frozen_player);
        }

        let animation_start = LEVEL_SKIP_FREEZE_VISIBLE_FRAMES
            + LEVEL_SKIP_HIDE_PLAYER_FRAMES
            + LEVEL_SKIP_BLANK_FRAMES
            + LEVEL_SKIP_SMALL_PRELOAD_FRAMES
            + LEVEL_SKIP_LARGE_PRELOAD_FRAMES;
        if elapsed < animation_start {
            return None;
        }
        let animation_frame = elapsed.saturating_sub(animation_start);
        if animation_frame >= LEVEL_SKIP_ANIMATION_FRAMES {
            return Some(PlayerState {
                x_pos: hold_pose.kid_x_pos,
                y_pos: hold_pose.kid_y_pos,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }

        let animation_frame_i64 = i64::try_from(animation_frame).unwrap_or(i64::MAX);
        let x_pos = i64::from(animation_path.initial_x_pos)
            + i64::from(animation_path.x_step_fp) * animation_frame_i64;
        let y_pos = i64::from(animation_path.initial_y_pos)
            + i64::from(animation_path.y_step_fp) * animation_frame_i64;
        Some(PlayerState {
            x_pos: (x_pos as i32) as u32,
            y_pos: (y_pos as i32) as u32,
            x_velocity: animation_path.x_step_fp,
            y_velocity: animation_path.y_step_fp,
            facing: Direction::Right,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        })
    }

    pub(crate) fn level_skip_camera_state(&self) -> CameraState {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return CameraState {
                x_pos: hold_pose.camera_x_pos,
                y_pos: hold_pose.camera_y_pos,
            };
        };
        if transition.elapsed_frames
            < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES
        {
            return transition.frozen_camera;
        }
        CameraState {
            x_pos: hold_pose.camera_x_pos,
            y_pos: hold_pose.camera_y_pos,
        }
    }

    pub(crate) fn level_skip_active_objects(&self) -> Vec<NativeObjectSnapshot> {
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return Vec::new();
        };
        if transition.elapsed_frames
            < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES
        {
            return transition.frozen_active_objects.clone();
        }
        Vec::new()
    }

    pub(crate) fn level_skip_object_counts(&self) -> (u16, u16) {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let animation_path = self.level_skip_animation_path(self.current_level);
        let Some(transition) = self.level_skip_transition.as_ref() else {
            return (hold_pose.number_objects, hold_pose.number_gfx_objects);
        };
        let elapsed = transition.elapsed_frames;
        if elapsed < LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES {
            return (
                transition.frozen_number_objects,
                transition.frozen_number_gfx_objects,
            );
        }
        let blank_start = LEVEL_SKIP_FREEZE_VISIBLE_FRAMES + LEVEL_SKIP_HIDE_PLAYER_FRAMES;
        let small_preload_start = blank_start + LEVEL_SKIP_BLANK_FRAMES;
        let large_preload_start = small_preload_start + LEVEL_SKIP_SMALL_PRELOAD_FRAMES;
        let animation_start = large_preload_start + LEVEL_SKIP_LARGE_PRELOAD_FRAMES;
        if elapsed < small_preload_start {
            (0, 0)
        } else if elapsed < large_preload_start {
            (2, 0)
        } else if elapsed < animation_start {
            (15, 0)
        } else if elapsed == animation_start {
            (
                animation_path.first_animation_number_objects,
                animation_path.first_animation_number_gfx_objects,
            )
        } else {
            (hold_pose.number_objects, hold_pose.number_gfx_objects)
        }
    }

    pub(crate) fn level_skip_resume_elapsed_frames(&self) -> Option<usize> {
        if !self.level_skip_resume_hold {
            return None;
        }
        self.level_start_frame
            .map(|frame| self.frame_index.saturating_sub(frame))
    }

    pub(crate) fn level_skip_resume_player_state(&self) -> Option<PlayerState> {
        self.level_skip_resume_player_state_at(self.level_skip_resume_elapsed_frames().unwrap_or(0))
    }

    pub(crate) fn level_skip_resume_player_state_at(&self, elapsed: usize) -> Option<PlayerState> {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        if elapsed < LEVEL_SKIP_RESUME_VISIBLE_FRAMES {
            return Some(PlayerState {
                x_pos: hold_pose.kid_x_pos,
                y_pos: hold_pose.kid_y_pos,
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: true,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }
        let resume_entry_elapsed = elapsed.saturating_sub(TITLE_TO_LEVEL_FRAMES);
        if elapsed >= TITLE_TO_LEVEL_FRAMES
            && self.current_level == LevelId(1)
            && resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES
        {
            return Some(PlayerState {
                x_pos: 0x0020_0000,
                y_pos: LEVEL_ONE_SKIP_RESUME_ENTRY_PLAYER_Y_FP[resume_entry_elapsed],
                x_velocity: 0,
                y_velocity: 0,
                facing: Direction::Right,
                on_ground: resume_entry_elapsed >= LEVEL_SKIP_RESUME_ENTRY_FRAMES - 3,
                on_spikes: false,
                just_jumped: false,
                jump_buffer_frames: 0,
                right_wall_sticky: false,
                left_wall_sticky: false,
                ledge_snap_cooldown: 0,
                wall_cling_direction: None,
            });
        }
        None
    }

    pub(crate) fn level_skip_resume_camera_state(&self) -> CameraState {
        self.level_skip_resume_camera_state_at(self.level_skip_resume_elapsed_frames().unwrap_or(0))
    }

    pub(crate) fn level_skip_resume_camera_state_at(&self, elapsed: usize) -> CameraState {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        let resume_entry_elapsed = elapsed.saturating_sub(TITLE_TO_LEVEL_FRAMES);
        if elapsed >= TITLE_TO_LEVEL_FRAMES
            && self.current_level == LevelId(1)
            && resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES
        {
            return CameraState {
                x_pos: 0,
                y_pos: LEVEL_ONE_SKIP_RESUME_ENTRY_CAMERA_Y_FP[resume_entry_elapsed],
            };
        }
        if self.current_level == LevelId(1) && elapsed == 52 {
            return CameraState {
                x_pos: 0,
                y_pos: 0xFFA0_0000,
            };
        }
        CameraState {
            x_pos: hold_pose.camera_x_pos,
            y_pos: hold_pose.camera_y_pos,
        }
    }

    pub(crate) fn level_skip_resume_active_objects(&self) -> Vec<NativeObjectSnapshot> {
        self.level_skip_resume_active_objects_at(
            self.level_skip_resume_elapsed_frames().unwrap_or(0),
        )
    }

    pub(crate) fn level_skip_resume_active_objects_at(
        &self,
        elapsed: usize,
    ) -> Vec<NativeObjectSnapshot> {
        let Some(player_state) = self.level_skip_resume_player_state_at(elapsed) else {
            return Vec::new();
        };
        vec![NativeObjectSnapshot {
            kind: NativeObjectKind::Player,
            x_pos: player_state.x_pos,
            y_pos: player_state.y_pos,
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
        }]
    }

    pub(crate) fn level_skip_resume_object_counts(&self) -> (u16, u16) {
        self.level_skip_resume_object_counts_at(
            self.level_skip_resume_elapsed_frames().unwrap_or(0),
        )
    }

    pub(crate) fn level_skip_resume_object_counts_at(&self, elapsed: usize) -> (u16, u16) {
        let hold_pose = self.level_skip_hold_pose(self.current_level);
        if elapsed < LEVEL_SKIP_RESUME_VISIBLE_FRAMES + LEVEL_SKIP_RESUME_HIDE_PLAYER_FRAMES {
            return (hold_pose.number_objects, hold_pose.number_gfx_objects);
        }
        if elapsed
            < LEVEL_SKIP_RESUME_VISIBLE_FRAMES
                + LEVEL_SKIP_RESUME_HIDE_PLAYER_FRAMES
                + LEVEL_SKIP_RESUME_BLANK_FRAMES
        {
            return (0, 0);
        }
        if elapsed == TITLE_TO_LEVEL_FRAMES - 1 {
            return (4, 2);
        }
        let resume_entry_elapsed = elapsed.saturating_sub(TITLE_TO_LEVEL_FRAMES);
        if elapsed >= TITLE_TO_LEVEL_FRAMES
            && self.current_level == LevelId(1)
            && resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES
        {
            if resume_entry_elapsed == 0 {
                return (5, 4);
            }
            if resume_entry_elapsed < LEVEL_SKIP_RESUME_ENTRY_FRAMES - 3 {
                return (5, 6);
            }
            return (4, 6);
        }
        (hold_pose.number_objects, hold_pose.number_gfx_objects)
    }

    pub(crate) fn level_skip_resume_final_slice(&self, level_id: LevelId) -> OpeningSliceState {
        if level_id == LevelId(1) {
            let bounds = self.level_bounds(level_id);
            return OpeningSliceState::respawn_from_pose(
                bounds,
                LevelStartPose {
                    kid_x_pos: 0x0020_0000,
                    kid_y_pos: 0x00BF_0000,
                    camera_x_pos: 0,
                    camera_y_pos: 0x0014_0000,
                },
            );
        }
        self.level_respawn_slice_with_floor_snap(level_id)
    }

    pub fn next_level_id(&self) -> LevelId {
        if let Some(logical_levels) = self.logical_level_ids() {
            let current_pos = logical_levels
                .iter()
                .position(|&id| id == self.current_level);
            if let Some(pos) = current_pos {
                let next_pos = pos + 1;
                if next_pos < logical_levels.len() {
                    return logical_levels[next_pos];
                }
            }
        }
        // Fallback: next numerically higher level ID
        let mut level_ids = self
            .assets
            .levels
            .iter()
            .map(|level| level.level_id)
            .collect::<Vec<_>>();
        level_ids.sort_unstable();
        level_ids
            .into_iter()
            .find(|level_id| *level_id > self.current_level.0)
            .map(LevelId)
            .or_else(|| {
                self.assets
                    .levels
                    .first()
                    .map(|level| LevelId(level.level_id))
            })
            .unwrap_or(self.current_level)
    }

    pub fn level_name(&self, level_id: LevelId) -> Option<&str> {
        self.level_record(level_id)
            .and_then(|level| level.display_name.as_deref())
    }

    pub(crate) fn level_header(&self, level_id: LevelId) -> Option<&kid_data::LevelHeader> {
        // The game resolves headers via MapOrder_Index[level_id] → resource level.
        // Prefer the resource level's header to match the ASM lookup path.
        self.resource_level_record(level_id)
            .and_then(|level| level.header_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.header_metadata.as_ref())
            })
    }

    pub(crate) fn enemy_layout(&self, level_id: LevelId) -> Option<&kid_data::EnemyLayout> {
        self.resource_level_record(level_id)
            .and_then(|level| level.enemy_layout_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.enemy_layout_metadata.as_ref())
            })
    }

    pub(crate) fn platform_layout(&self, level_id: LevelId) -> Option<&kid_data::PlatformLayout> {
        self.resource_level_record(level_id)
            .and_then(|level| level.platform_metadata.as_ref())
            .or_else(|| {
                self.level_record(level_id)
                    .and_then(|level| level.platform_metadata.as_ref())
            })
    }

    pub(crate) fn level_record(&self, level_id: LevelId) -> Option<&kid_data::LevelRecord> {
        self.assets
            .levels
            .iter()
            .find(|level| level.level_id == level_id.0)
    }

    pub(crate) fn resource_level_record(
        &self,
        level_id: LevelId,
    ) -> Option<&kid_data::LevelRecord> {
        resource_level_record(&self.assets, level_id.0)
    }

    pub(crate) fn level_bounds(&self, level_id: LevelId) -> LevelBounds {
        self.level_header(level_id)
            .map(|header| LevelBounds {
                width_pixels: u16::from(header.width_screens) * 320,
                height_pixels: u16::from(header.height_screens) * 224,
            })
            .unwrap_or(LevelBounds {
                width_pixels: 320,
                height_pixels: 224,
            })
    }
}
