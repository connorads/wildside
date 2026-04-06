use super::*;
use crate::level::fade_palette_span_sub_1C85E;
use std::collections::BTreeMap;

const INTRO_SEQUENCE_3_ROSTER_DESCRIPTOR_POSITIONS: [(u16, u16); 14] = [
    (0x00F8, 0x00D0),
    (0x00E3, 0x00CC),
    (0x0040, 0x00CC),
    (0x0060, 0x00D0),
    (0x00D0, 0x00C8),
    (0x00C4, 0x00CC),
    (0x0070, 0x00CC),
    (0x0098, 0x00C4),
    (0x00E4, 0x00C8),
    (0x0050, 0x00C8),
    (0x0024, 0x00CC),
    (0x003C, 0x00C8),
    (0x0114, 0x00CA),
    (0x002C, 0x00C8),
];
const INTRO_SEQUENCE_3_ROSTERS: [[u8; 8]; 4] = [
    [0, 2, 3, 4, 6, 7, 9, 10],
    [0, 5, 6, 7, 8, 12, 0xFF, 0xFF],
    [1, 7, 8, 10, 11, 13, 0xFF, 0xFF],
    [1, 2, 3, 5, 7, 8, 9, 0xFF],
];
const INTRO_SEQUENCE_3_TRACE_DESCRIPTOR_SLOT: usize = 2;
const LEVEL_ENTRY_TITLE_SLOT_2_INDEX: usize = 2;
const LEVEL_ENTRY_TITLE_ANIMATION_FRAMES: usize = 64;
pub(crate) const LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME: usize = 43;
pub(crate) const LEVEL_ENTRY_TITLE_DEFERRED_NUMBER_VISIBLE_FRAME: usize =
    LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME + 1;
// ASM Load_TitleCard allocates loc_1AB26/loc_1AE2E immediately, but oracle
// boot traces do not let the queued Start edge reach visible gameplay until
// global frame 520 after the frame-178 title-card hand-off. Keep the shared
// title-card lineage on the hidden bootstrap for 342 local frames so slot-2
// stays hidden at frame 200 and only the title shell appears on frame 504.
pub(crate) const LEVEL_ENTRY_TITLE_GAMEPLAY_HANDOFF_FRAMES: usize = 342;

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EndSequenceCreditsGlyphInitSub1D786 {
    pub(crate) raw_allocation_spec: u32,
    pub(crate) display_enabled: bool,
    pub(crate) tile_word: u16,
    pub(crate) priority: u8,
    pub(crate) palette_line: u8,
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) fn end_sequence_credits_glyph_init_sub_1D786() -> EndSequenceCreditsGlyphInitSub1D786 {
    // ASM sub_1D786 (kid.asm:32983): allocate the credits glyph worker with
    // raw gfx-object spec `$03000002`, clear display flag `$13(a1)`, then set
    // tile word `$0509`, priority 2, and palette line 1.
    EndSequenceCreditsGlyphInitSub1D786 {
        raw_allocation_spec: 0x0300_0002,
        display_enabled: false,
        tile_word: 0x0509,
        priority: 2,
        palette_line: 1,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectGfxLinksSubE02 {
    pub(crate) object_slot_address: u32,
    pub(crate) gfx_slot_address: u32,
    pub(crate) gfx_to_object_link: u32,
    pub(crate) object_to_gfx_link: u32,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_predecessor_index_sub_FF6(
    next_indices: &[Option<usize>],
    target_index: usize,
) -> Option<usize> {
    // ASM loc_1004 (kid.asm:1925): when the current `0(a4)` link is neither
    // null nor the target `a3`, load that next pointer into `a4` and branch
    // back to `loc_FFE`; if the walk reaches a null link first, `beq.s
    // loc_1038` returns without unlinking anything.
    let mut cursor_index = 0usize;
    loop {
        let next_index = next_indices[cursor_index]?;
        if next_index == target_index {
            return Some(cursor_index);
        }
        cursor_index = next_index;
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_unlink_loc_100E(
    next_indices: &mut [Option<usize>],
    predecessor_index: usize,
    target_index: usize,
) {
    // ASM loc_100E (kid.asm:1932): after loc_FFE finds the predecessor in
    // `a4`, copy the target's `0(a3)` next link over `0(a4)` to unlink `a3`
    // before the later sentinel restart and priority-ordered reinsertion.
    next_indices[predecessor_index] = next_indices[target_index];
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_insertion_predecessor_index_loc_101C(
    next_indices: &[Option<usize>],
    priority_words: &[u16],
    target_priority_word: u16,
) -> usize {
    // ASM loc_101C (kid.asm:1937): restart from the sentinel, keep following
    // `0(a4)` while each candidate's `$A(a4)` priority word is less than or
    // equal to `d5`, then insert after the last such node.
    let mut cursor_index = 0usize;
    loop {
        let Some(next_index) = next_indices[cursor_index] else {
            return cursor_index;
        };
        if priority_words[next_index] > target_priority_word {
            return cursor_index;
        }
        cursor_index = next_index;
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn gfx_object_insert_loc_102E(
    next_indices: &mut [Option<usize>],
    predecessor_index: usize,
    target_index: usize,
) {
    // ASM loc_102E (kid.asm:1946): copy the predecessor's current `0(a4)`
    // link into the target's `0(a3)` slot, then store `a3` back into `0(a4)`
    // so reinsertion splices the detached node ahead of the old successor.
    next_indices[target_index] = next_indices[predecessor_index];
    next_indices[predecessor_index] = Some(target_index);
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn allocate_object_and_gfx_links_sub_E02(
    object_slot_address: u32,
    gfx_slot_address: u32,
) -> ObjectGfxLinksSubE02 {
    // ASM sub_E02 (kid.asm:1622): `Allocate_ObjectSlot` leaves the new object
    // in `a0`, `Allocate_GfxObjectSlot_a1` leaves the gfx object in `a1`,
    // then the routine cross-links them through object `$36(a0)` and gfx
    // `$C(a1)` before returning.
    ObjectGfxLinksSubE02 {
        object_slot_address,
        gfx_slot_address,
        gfx_to_object_link: object_slot_address,
        object_to_gfx_link: gfx_slot_address,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockJumpWindowSubDFB0 {
    pub(crate) left_reach_px: u16,
    pub(crate) right_reach_px: u16,
    pub(crate) vertical_reach_px: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockRevealWorkerSubE0F8 {
    pub(crate) allocation_spec: u16,
    pub(crate) object_handler_addr: u32,
    pub(crate) block_col: u16,
    pub(crate) block_row: u16,
    pub(crate) layout_word_addr: u32,
    pub(crate) sprite_mapping: u16,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockRevealWorkerSubE1D4 {
    pub(crate) revealed_layout_byte: u8,
    pub(crate) allocation_spec: u16,
    pub(crate) object_handler_addr: u32,
    pub(crate) block_col: u16,
    pub(crate) block_row: u16,
    pub(crate) layout_word_addr: u32,
    pub(crate) sprite_mapping: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_jump_window_sub_DFB0(
    form: PlayerForm,
    facing: Direction,
) -> HiddenBlockJumpWindowSubDFB0 {
    // ASM sub_DFB0 (kid.asm:17317): when `Character_Movement == MoveID_Jump`,
    // index `unk_E246` by `Current_Helmet`, load two horizontal reach words
    // plus one vertical reach word, then swap the horizontal pair when
    // `x_direction(a3)` is non-zero before probing hidden blocks.
    let (mut left_reach_px, mut right_reach_px, vertical_reach_px) = match form {
        PlayerForm::Juggernaut => (0x0010, 0x0010, 0x0010),
        PlayerForm::Micromax => (0x0002, 0x0002, 0x0010),
        _ => (0x0004, 0x0004, 0x0020),
    };
    if facing == Direction::Left {
        std::mem::swap(&mut left_reach_px, &mut right_reach_px);
    }
    HiddenBlockJumpWindowSubDFB0 {
        left_reach_px,
        right_reach_px,
        vertical_reach_px,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_reveal_worker_sub_E0F8(
    block_col: u16,
    block_row: u16,
    layout_word_addr: u32,
    sprite_mapping: u16,
) -> HiddenBlockRevealWorkerSubE0F8 {
    // ASM sub_E0F8 (kid.asm:17444): allocate a `$2001` object slot, install
    // `loc_E304`, copy the caller's block column/row plus layout pointer, and
    // pull the next sprite mapping word from the `unk_E11E` table into `$4E(a0)`.
    HiddenBlockRevealWorkerSubE0F8 {
        allocation_spec: 0x2001,
        object_handler_addr: 0x0000_E304,
        block_col,
        block_row,
        layout_word_addr,
        sprite_mapping,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn hidden_block_reveal_worker_sub_E1D4(
    block_col: u16,
    block_row: u16,
    layout_word_addr: u32,
    sprite_mapping: u16,
) -> HiddenBlockRevealWorkerSubE1D4 {
    // ASM sub_E1D4 (kid.asm:17619): write `#$60` into `(a2)`, allocate a
    // `$2001` object slot, install `loc_E282`, then copy the caller's block
    // column/row plus layout pointer and sprite mapping into `$44/$46/$48/$4C/$4E`.
    HiddenBlockRevealWorkerSubE1D4 {
        revealed_layout_byte: 0x60,
        allocation_spec: 0x2001,
        object_handler_addr: 0x0000_E282,
        block_col,
        block_row,
        layout_word_addr,
        sprite_mapping,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn hide_end_sequence_credits_glyphs_sub_1D7E6(display_enabled: &mut [bool; 6]) {
    // ASM sub_1D7E6 (kid.asm:33029): load the six credits glyph worker slots
    // from `$FFFFFB72`, then clear each worker's display-enable byte `$13(a3)`.
    for visible in display_enabled.iter_mut() {
        *visible = false;
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn fade_end_sequence_palette_sub_1D800<F>(
    palette_buffer: &mut [u16],
    mut refresh_and_hibernate: F,
) where
    F: FnMut(),
{
    // ASM sub_1D800 (kid.asm:33046): spill the return address and
    // `d0-d1/d7/a4` into the ending scratch area, darken Palette_Buffer+$60
    // via sub_1C85E eight times, and after each fade pass run the
    // `sub_1D830 -> Hibernate_Object_1Frame` pair for four iterations.
    for _ in 0..8 {
        fade_palette_span_sub_1C85E(palette_buffer, 0x0060);
        for _ in 0..4 {
            refresh_and_hibernate();
        }
    }
}

impl PlayerState {
    pub(crate) fn intro_sequence_3_roster_count() -> usize {
        INTRO_SEQUENCE_3_ROSTERS.len()
    }

    pub(crate) fn intro_sequence_3_pose_for_roster(roster_index: usize) -> Self {
        // ROM Load_IntroSequence3 rotates through the four `unk_1AF78`
        // descriptor rosters via `$FFFFFC83`, and oracle traces expose the
        // third allocated roster descriptor in slot 2 as `kid_gfx`.
        let descriptor_index = INTRO_SEQUENCE_3_ROSTERS
            [roster_index % INTRO_SEQUENCE_3_ROSTERS.len()][INTRO_SEQUENCE_3_TRACE_DESCRIPTOR_SLOT]
            as usize;
        let (x_pos, y_pos) = INTRO_SEQUENCE_3_ROSTER_DESCRIPTOR_POSITIONS[descriptor_index];
        Self {
            x_pos: u32::from(x_pos) << 16,
            y_pos: u32::from(y_pos) << 16,
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Left,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        }
    }

    pub(crate) fn title_pose() -> Self {
        Self {
            x_pos: TITLE_KID_X_POS,
            y_pos: TITLE_KID_Y_POS,
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
        }
    }

    pub(crate) fn title_menu_cursor_pose(y_pos: u32) -> Self {
        Self {
            x_pos: 0x0028_0000,
            y_pos,
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
        }
    }

    pub(crate) fn intro_sequence_5_slot_2_pose() -> Self {
        Self {
            x_pos: INTRO_SEQUENCE_5_SLOT_2_X_POS,
            y_pos: INTRO_SEQUENCE_5_SLOT_2_Y_POS,
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
        }
    }

    pub(crate) fn from_level_entry_title_worker(
        worker: &LevelEntryTitleWorker,
        elapsed_frames: usize,
    ) -> Self {
        let motion = worker
            .motion
            .as_ref()
            .expect("seeded level-entry title worker");
        Self {
            x_pos: level_entry_title_worker_axis_pos(
                motion.start_x_pos,
                worker.target_x_pos,
                motion.x_step_fp,
                elapsed_frames,
            ),
            y_pos: level_entry_title_worker_axis_pos(
                motion.start_y_pos,
                worker.target_y_pos,
                motion.y_step_fp,
                elapsed_frames,
            ),
            x_velocity: 0,
            y_velocity: 0,
            facing: Direction::Left,
            on_ground: true,
            on_spikes: false,
            just_jumped: false,
            jump_buffer_frames: 0,
            right_wall_sticky: false,
            left_wall_sticky: false,
            ledge_snap_cooldown: 0,
            wall_cling_direction: None,
        }
    }
}

impl NativeRuntime {
    pub(crate) fn rebuild_starting_level_title_workers(&mut self) {
        self.starting_level_title_workers.clear();
        let Some(title_card) = self
            .level_record(self.current_level)
            .and_then(|level| level.title_card.as_ref())
            .cloned()
        else {
            return;
        };

        let visible_glyph_count = title_card
            .script_bytes
            .iter()
            .filter(|&&glyph| glyph != 0 && glyph != 0x84 && glyph != 0xFF)
            .count();
        let mut current_destination = title_card.line_destinations.first().cloned();
        let mut line_index = 0usize;
        let mut visible_glyph_index = 0usize;

        for &glyph in &title_card.script_bytes {
            match glyph {
                0 => {
                    line_index += 1;
                    current_destination = title_card.line_destinations.get(line_index).cloned();
                }
                0x84 => {
                    if let Some(destination) = current_destination.as_mut() {
                        destination.x = destination.x.saturating_add(0x10);
                    }
                }
                0xFF => break,
                _ => {
                    let Some(destination) = current_destination.clone() else {
                        break;
                    };
                    let gfx_slot_index = level_entry_title_letter_gfx_slot_index(
                        visible_glyph_index,
                        visible_glyph_count,
                    );
                    let target_x_pos = u32::from(destination.x) << 16;
                    let target_y_pos =
                        u32::from(level_entry_title_target_y(glyph, destination.y)) << 16;
                    self.starting_level_title_workers
                        .push(LevelEntryTitleWorker {
                            // ASM loc_1AB26 allocates title objects in script
                            // order, but `Allocate_ObjectSlot` reverse-inserts
                            // equal-priority workers before the next
                            // `Execute_Objects` pass. Oracle slot probes still
                            // read fixed gfx RAM slots directly, so keep the
                            // traced slot id separate from the later first-pass
                            // execution order used for RNG seeding.
                            gfx_slot_index,
                            activation_frame: LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME,
                            glyph,
                            target_x_pos,
                            target_y_pos,
                            motion: None,
                        });
                    if let Some(destination) = current_destination.as_mut() {
                        destination.x = destination
                            .x
                            .saturating_add(level_entry_title_advance_x(glyph));
                    }
                    visible_glyph_index += 1;
                }
            }
        }

        if let Some(destination) = title_card.level_number_destination {
            for number_index in 0..usize::from(title_card.level_number_glyph_count) {
                self.starting_level_title_workers
                    .push(LevelEntryTitleWorker {
                        // ASM loc_1AD48 allocates the deferred number worker
                        // after the visible title glyph shell, so the suffix
                        // occupies the next free gfx slots above the letters.
                        gfx_slot_index: visible_glyph_count + number_index,
                        activation_frame: LEVEL_ENTRY_TITLE_DEFERRED_NUMBER_VISIBLE_FRAME,
                        glyph: 0x7B,
                        target_x_pos: u32::from(
                            destination.x.saturating_add(number_index as u16 * 0x10),
                        ) << 16,
                        target_y_pos: u32::from(destination.y) << 16,
                        motion: None,
                    });
            }
        }
    }

    pub(crate) fn maybe_seed_starting_level_title_workers(&mut self, elapsed_frames: usize) {
        let uses_early_start_title_seed_rng = matches!(
            self.boot_state,
            BootState::StartingLevel {
                uses_load_ingame_bootstrap: true,
                load_ingame_bootstrap_started: false,
                ..
            }
        );
        let workers = &mut self.starting_level_title_workers;
        let rom_random = &mut self.rom_random;
        let title_rng = &mut self.starting_level_title_rng;
        let seed_indices = (0..workers.len())
            .filter(|&index| {
                let worker = &workers[index];
                worker.motion.is_none()
                    && level_entry_title_effective_activation_frame(
                        worker,
                        uses_early_start_title_seed_rng,
                    ) == elapsed_frames
            })
            .collect::<Vec<_>>();
        for index in seed_indices.into_iter().rev() {
            let worker = &mut workers[index];
            let (start_x_pos, start_y_pos) =
                if uses_early_start_title_seed_rng || title_rng.is_some() {
                    let rng = title_rng
                        .as_mut()
                        .expect("level-entry title seed RNG should be initialised");
                    let start_selector = rng.next_byte() & 3;
                    level_entry_title_start_pose(rng, start_selector)
                } else {
                    let start_selector = rom_random.next_byte() & 3;
                    level_entry_title_start_pose(rom_random, start_selector)
                };
            worker.motion = Some(LevelEntryTitleWorkerMotion {
                start_x_pos,
                start_y_pos,
                x_step_fp: level_entry_title_axis_step(start_x_pos, worker.target_x_pos),
                y_step_fp: level_entry_title_axis_step(start_y_pos, worker.target_y_pos),
            });
        }
    }

    pub(crate) fn starting_level_slot_2_pose(&self, elapsed_frames: usize) -> Option<PlayerState> {
        self.starting_level_title_workers
            .iter()
            .find(|worker| worker.gfx_slot_index == LEVEL_ENTRY_TITLE_SLOT_2_INDEX)
            .and_then(|worker| {
                let activation_frame = if matches!(
                    self.boot_state,
                    BootState::StartingLevel {
                        uses_load_ingame_bootstrap: true,
                        load_ingame_bootstrap_started: false,
                        ..
                    }
                ) {
                    level_entry_title_effective_activation_frame(worker, true)
                } else {
                    level_entry_title_effective_activation_frame(worker, false)
                };
                if elapsed_frames < activation_frame {
                    return None;
                }
                worker.motion.as_ref().map(|_| {
                    // ASM loc_1AC70 -> loc_1AD2A seeds the start pose on first
                    // execution, then hibernates for one frame before the
                    // first glide step. Oracle frame 504 is that first visible
                    // shell for slot 2, while the early title-start lineage
                    // exposes the same seeded pose on frame 172 after the
                    // shorter pre-Start mode-$0008 setup.
                    let visible_elapsed_frames = elapsed_frames.saturating_sub(activation_frame);
                    PlayerState::from_level_entry_title_worker(worker, visible_elapsed_frames)
                })
            })
    }

    pub(crate) fn starting_level_title_active_objects(
        &self,
        elapsed_frames: usize,
    ) -> Vec<NativeObjectSnapshot> {
        if matches!(
            self.boot_state,
            BootState::StartingLevel {
                uses_load_ingame_bootstrap: true,
                load_ingame_bootstrap_started: true,
                ..
            }
        ) {
            // ASM loc_1AE2E -> loc_6E2 stops exposing the carried loc_1AB26 /
            // loc_1AD48 title workers once the hidden Load_InGame bootstrap
            // begins; the later preload-object timeline owns the trace from
            // there.
            return Vec::new();
        }
        let mut objects = self
            .starting_level_title_workers
            .iter()
            .filter(|worker| {
                let activation_frame = if matches!(
                    self.boot_state,
                    BootState::StartingLevel {
                        uses_load_ingame_bootstrap: true,
                        load_ingame_bootstrap_started: false,
                        ..
                    }
                ) {
                    level_entry_title_effective_activation_frame(worker, true)
                } else {
                    level_entry_title_effective_activation_frame(worker, false)
                };
                worker.motion.is_some() && elapsed_frames >= activation_frame
            })
            .map(|worker| {
                let activation_frame = if matches!(
                    self.boot_state,
                    BootState::StartingLevel {
                        uses_load_ingame_bootstrap: true,
                        load_ingame_bootstrap_started: false,
                        ..
                    }
                ) {
                    level_entry_title_effective_activation_frame(worker, true)
                } else {
                    level_entry_title_effective_activation_frame(worker, false)
                };
                let visible_elapsed_frames = elapsed_frames.saturating_sub(activation_frame);
                let pose =
                    PlayerState::from_level_entry_title_worker(worker, visible_elapsed_frames);
                NativeObjectSnapshot {
                    kind: NativeObjectKind::Enemy { object_type: 0 },
                    x_pos: pose.x_pos,
                    y_pos: pose.y_pos,
                    layout_index: None,
                    x_velocity: None,
                    y_velocity: None,
                    facing: Some(Direction::Left),
                    on_ground: Some(true),
                    hit_points: None,
                    damage_flash: false,
                    palette_variant: 0,
                    stationary: true,
                    shared_walker_state: None,
                    shared_walker_patrol_timer: None,
                    shared_walker_patrol_short_next: None,
                    walk_phase: None,
                    wait_timer: None,
                    debug_fields: BTreeMap::from([(
                        String::from("gfx_slot_index"),
                        worker.gfx_slot_index.to_string(),
                    )]),
                }
            })
            .collect::<Vec<_>>();
        objects.sort_by_key(|object| {
            object
                .debug_fields
                .get("gfx_slot_index")
                .and_then(|slot| slot.parse::<usize>().ok())
                .unwrap_or(usize::MAX)
        });
        objects
    }
}

fn level_entry_title_target_y(glyph: u8, y: u16) -> u16 {
    if glyph == 0x83 {
        y.saturating_sub(8)
    } else {
        y
    }
}

fn level_entry_title_advance_x(glyph: u8) -> u16 {
    match glyph.wrapping_sub(b'a') {
        0x1B => 0x28,
        0x1C => 0x20,
        0x1D | 0x1E => 0x18,
        0x08 | 0x22 => 0x08,
        _ => 0x10,
    }
}

fn level_entry_title_letter_gfx_slot_index(
    visible_glyph_index: usize,
    visible_glyph_count: usize,
) -> usize {
    visible_glyph_count
        .saturating_sub(visible_glyph_index)
        .saturating_sub(1)
}

fn level_entry_title_effective_activation_frame(
    worker: &LevelEntryTitleWorker,
    uses_early_start_bootstrap: bool,
) -> usize {
    if uses_early_start_bootstrap {
        EARLY_START_TITLE_FIRST_VISIBLE_TRACE_FRAME
            + worker
                .activation_frame
                .saturating_sub(LEVEL_ENTRY_TITLE_FIRST_VISIBLE_TRACE_FRAME)
    } else {
        worker.activation_frame
    }
}

fn level_entry_title_axis_step(start_pos: u32, target_pos: u32) -> i32 {
    (target_pos as i32).wrapping_sub(start_pos as i32) >> 6
}

fn level_entry_title_worker_axis_pos(
    start_pos: u32,
    target_pos: u32,
    step_fp: i32,
    elapsed_frames: usize,
) -> u32 {
    let current = (start_pos as i32).wrapping_add(
        step_fp.wrapping_mul(elapsed_frames.min(LEVEL_ENTRY_TITLE_ANIMATION_FRAMES) as i32),
    );
    let target = target_pos as i32;
    if step_fp >= 0 {
        current.min(target) as u32
    } else {
        current.max(target) as u32
    }
}

fn level_entry_title_start_pose(rng: &mut RomRandomState, selector: u8) -> (u32, u32) {
    let (x, y) = match selector {
        0 => (-0x20, level_entry_title_random_y(rng)),
        1 => (level_entry_title_random_x(rng), -0x20),
        2 => (0x160, level_entry_title_random_y(rng)),
        _ => (level_entry_title_random_x(rng), 0x100),
    };
    ((x << 16) as u32, (y << 16) as u32)
}

fn level_entry_title_random_y(rng: &mut RomRandomState) -> i32 {
    let mut value = i32::from(rng.next_byte()) - 0x20;
    if value <= 0 {
        value += 0x28;
    }
    value & !3
}

fn level_entry_title_random_x(rng: &mut RomRandomState) -> i32 {
    (i32::from(rng.next_word() & 0x00FF) + 0x3E) & !3
}

impl OpeningSliceState {
    pub(crate) fn new(bounds: LevelBounds) -> Self {
        Self {
            player: PlayerState {
                x_pos: 0x0020_0000,
                y_pos: OPENING_GROUNDED_KID_Y_POS,
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
            },
            camera: CameraState {
                x_pos: 0,
                y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            },
            terrain: TerrainState {
                floor_y_pos: OPENING_GROUNDED_KID_Y_POS,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: bounds,
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn respawn(bounds: LevelBounds) -> Self {
        Self::respawn_from_pose(
            bounds,
            LevelStartPose {
                kid_x_pos: 0x0020_0000,
                kid_y_pos: OPENING_GROUNDED_KID_Y_POS,
                camera_x_pos: 0,
                camera_y_pos: OPENING_GROUNDED_CAMERA_Y_FP,
            },
        )
    }

    pub(crate) fn respawn_from_pose(bounds: LevelBounds, pose: LevelStartPose) -> Self {
        Self {
            player: PlayerState {
                x_pos: pose.kid_x_pos,
                y_pos: pose.kid_y_pos,
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
            },
            camera: CameraState {
                x_pos: pose.camera_x_pos,
                y_pos: pose.camera_y_pos,
            },
            terrain: TerrainState {
                floor_y_pos: pose.kid_y_pos,
                floor_terrain_type: TerrainType::Normal,
            },
            level_bounds: bounds,
            objects: ObjectSystemState {
                platform_loader: PlatformLoaderState::default(),
            },
            prize_block_hit: None,
            lift_block_hit: None,
            drill_projectile_hit: None,
            metal_block_hit: None,
            rubber_block_hit: None,
            smash_block_hit: None,
            shooter_block_hit: None,
            teleporter_contact: None,
            ceiling_prize_clear: None,
            pre_gravity_y_vel: 0,
            physics_trace: None,
            wall_cling_release_probe: None,
            kid_flip_follow_on_stages: None,
            kid_jump_gravity_probe: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        custom_gravity: bool,
        y_look_offset: i16,
    ) {
        self.step_with_order(
            input,
            jump_pressed,
            form,
            block_grid,
            block_list,
            custom_gravity,
            y_look_offset,
            PhysicsOrder::GravityFirst,
            true,
            &[],
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step_with_order(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        custom_gravity: bool,
        y_look_offset: i16,
        physics_order: PhysicsOrder,
        allow_asm_faithful_solid_touch_snap: bool,
        deferred_prize_ceiling_cells: &[(u16, u16, DeferredPrizeBlockPhase)],
    ) {
        // ASM loc_1062 / Hibernate_UntilAnimFinished (kid.asm:1985) loops for
        // 21 frames after ledge snap, suppressing all physics.
        // Camera follow resumes on the final frame (ASM exits animation,
        // transitions to Standingstill, then main loop runs camera follow).
        if self.player.ledge_snap_cooldown > 0 {
            self.player.ledge_snap_cooldown -= 1;
            if self.player.ledge_snap_cooldown > 0 {
                return;
            }
            // Last cooldown frame: physics still suppressed but camera catches up.
            self.update_camera_follow(y_look_offset);
            return;
        }
        let mut block_grid = block_grid;
        let on_ground_before = self.player.on_ground;
        self.wall_cling_release_probe = None;
        self.kid_flip_follow_on_stages = None;
        self.kid_jump_gravity_probe = None;
        // ASM sub_914A uses FA78 (per-form half-width from unk_B408) for all
        // collision probes. Thread the form's half-width through to collision
        // functions instead of the constant PLAYER_HALF_WIDTH.
        let collision_hw = form.half_width();

        match physics_order {
            PhysicsOrder::GravityFirst => {
                if let Some(ref mut t) = self.physics_trace {
                    t.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.on_ground_before = on_ground_before;
                }

                // Current native order: horizontal, vertical (gravity+position), collisions
                let horizontal_step_applied =
                    self.update_horizontal_motion(input, jump_pressed, form, block_grid.as_deref());

                if let Some(ref mut t) = self.physics_trace {
                    t.post_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.horizontal_step_applied = horizontal_step_applied;
                    t.pre_vertical = (self.player.y_pos, self.player.y_velocity);
                    t.pre_gravity_y_vel = self.player.y_velocity;
                }

                // Save pre-gravity y_vel for floor landing sub-pixel logic (ASM tests
                // y_vel in Character_CheckCollision, before sub_B168 applies gravity).
                let pre_gravity_y_vel = self.player.y_velocity;
                self.update_vertical_motion(input, jump_pressed, form, custom_gravity);
                self.pre_gravity_y_vel = pre_gravity_y_vel;

                if let Some(ref mut t) = self.physics_trace {
                    t.post_gravity_y_vel = self.player.y_velocity;
                    t.gravity_applied =
                        self.player.y_velocity != pre_gravity_y_vel || !self.player.on_ground;
                    t.ascending_and_b_held = pre_gravity_y_vel < 0 && input.b;
                    t.post_vertical = (self.player.y_pos, self.player.y_velocity);
                    // Detect jump initiation: y_vel went negative from non-negative
                    if pre_gravity_y_vel >= 0 && self.player.y_velocity < 0 {
                        t.jump_initiated = true;
                        t.jump_y_vel = Some(self.player.y_velocity);
                    }
                }

                let pre_collision_x = self.player.x_pos;
                let airborne_slope_recontact = if let Some(grid) = block_grid.as_deref_mut() {
                    self.apply_wall_collision(Some(grid), collision_hw);
                    if self.player.x_pos != pre_collision_x
                        && let Some(ref mut t) = self.physics_trace
                    {
                        t.wall_hit = true;
                        t.wall_snap_x = Some(self.player.x_pos);
                    }
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid, collision_hw) {
                        self.finalise_trace();
                        // ASM camera follow runs after all physics sub-steps. On the
                        // ledge-snap frame, the camera follow sees the pre-snap
                        // position — skip the follow here and let the next frame
                        // catch up, matching the oracle's one-frame camera lag.
                        return;
                    }
                    self.apply_ceiling_collision(
                        Some(grid),
                        block_list,
                        form,
                        collision_hw,
                        deferred_prize_ceiling_cells,
                    );
                    self.apply_floor_collision(
                        Some(grid),
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                } else {
                    self.apply_floor_collision(
                        None,
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                };
                if let (Some(grid), Some(collision)) =
                    (block_grid.as_deref(), airborne_slope_recontact)
                {
                    self.apply_immediate_slope_recontact(
                        input,
                        form,
                        collision,
                        grid,
                        collision_hw,
                    );
                }
            }
            PhysicsOrder::PositionFirst => {
                if let Some(ref mut t) = self.physics_trace {
                    t.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.on_ground_before = self.player.on_ground;
                }
                // ASM order: horizontal motion + position, apply y_pos from y_velocity
                // (without gravity first), collisions, then gravity
                let horizontal_step_applied =
                    self.update_horizontal_motion(input, jump_pressed, form, block_grid.as_deref());
                if let Some(ref mut t) = self.physics_trace {
                    t.post_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.horizontal_step_applied = horizontal_step_applied;
                    t.pre_vertical = (self.player.y_pos, self.player.y_velocity);
                    t.pre_gravity_y_vel = self.player.y_velocity;
                }
                // Apply vertical position from current y_velocity (before gravity)
                self.apply_vertical_position_only(input, jump_pressed, form);
                // In PositionFirst, y_vel at collision time is pre-gravity (matches ASM).
                self.pre_gravity_y_vel = self.player.y_velocity;
                if let Some(ref mut t) = self.physics_trace {
                    t.post_vertical = (self.player.y_pos, self.player.y_velocity);
                }
                let airborne_slope_recontact = if let Some(grid) = block_grid.as_deref_mut() {
                    self.apply_wall_collision(Some(grid), collision_hw);
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid, collision_hw) {
                        self.finalise_trace();
                        // ASM camera follow runs after all physics sub-steps. On the
                        // ledge-snap frame, the camera follow sees the pre-snap
                        // position — skip the follow here and let the next frame
                        // catch up, matching the oracle's one-frame camera lag.
                        return;
                    }
                    self.apply_ceiling_collision(
                        Some(grid),
                        block_list,
                        form,
                        collision_hw,
                        deferred_prize_ceiling_cells,
                    );
                    self.apply_floor_collision(
                        Some(grid),
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                } else {
                    self.apply_floor_collision(
                        None,
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        true,
                    )
                };
                if let (Some(grid), Some(collision)) =
                    (block_grid.as_deref(), airborne_slope_recontact)
                {
                    self.apply_immediate_slope_recontact(
                        input,
                        form,
                        collision,
                        grid,
                        collision_hw,
                    );
                }
                // Apply gravity after collisions (ASM sub_B168 runs after collision checks)
                let pre_post_grav_vel = self.player.y_velocity;
                if !self.player.on_ground && !custom_gravity {
                    let ascending_and_b_held = self.player.y_velocity < 0 && input.b;
                    self.player.y_velocity =
                        physics::apply_gravity(self.player.y_velocity, ascending_and_b_held);
                    if let Some(ref mut t) = self.physics_trace {
                        t.post_gravity_y_vel = self.player.y_velocity;
                        t.gravity_applied = true;
                        t.ascending_and_b_held = ascending_and_b_held;
                    }
                } else if let Some(ref mut t) = self.physics_trace {
                    t.post_gravity_y_vel = pre_post_grav_vel;
                }
            }
            PhysicsOrder::AsmFaithful => {
                if let Some(ref mut t) = self.physics_trace {
                    t.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.on_ground_before = on_ground_before;
                }

                // ASM walking coroutine (loc_8BF0): sub_B43A checks floor at
                // (y_pos + 1) / 16 across player width BEFORE any velocity/position
                // updates. If no Solid/Spikes cell found → loc_B5F6: y_pos += 1px,
                // transition to Jump, skip sub_9A0A/sub_8F26 entirely.
                let grounded_slope_before = self
                    .grounded_slope_collision(block_grid.as_deref())
                    .filter(|_| on_ground_before);
                if on_ground_before
                    && grounded_slope_before.is_none()
                    && let Some(grid) = block_grid.as_deref()
                    && !self.walking_floor_support_exists_at_current_position(grid, collision_hw)
                {
                    // loc_B5F6: addq.w #1,y_pos(a3)
                    self.player.y_pos = self.player.y_pos.wrapping_add(0x0001_0000);
                    self.player.on_ground = false;
                    self.player.on_spikes = false;
                    self.terrain.floor_terrain_type = TerrainType::Normal;
                    self.finalise_trace();
                    self.update_camera_follow(y_look_offset);
                    return;
                }

                let entry_stage = self.kid_flip_follow_on_stage_state();
                let mut kid_flip_follow_on_stages = KidFlipFollowOnStages {
                    entry: entry_stage,
                    after_press_away: entry_stage,
                    after_jump_gravity: entry_stage,
                    after_floor: entry_stage,
                    after_wall: entry_stage,
                };

                // ASM loc_AA9C: pressing away from wall exits wall-cling.
                if let Some(dir) = self.player.wall_cling_direction {
                    let pressing_away = match dir {
                        Direction::Right => input.left,
                        Direction::Left => input.right,
                    };
                    if pressing_away {
                        self.player.wall_cling_direction = None;
                    }
                }
                kid_flip_follow_on_stages.after_press_away = self.kid_flip_follow_on_stage_state();

                // Step 1: sub_B084 — horizontal velocity only (no x_pos update).
                // Suppressed during wall-cling (ASM loc_AA22 never calls sub_B084).
                let horizontal_step_applied = if self.player.wall_cling_direction.is_some() {
                    false
                } else {
                    self.update_horizontal_velocity_only(
                        input,
                        jump_pressed,
                        form,
                        block_grid.as_deref(),
                    )
                };

                // ASM loc_9D22: on slopes, y_vel mirrors x_vel so the player
                // moves diagonally along the 45° surface. SlopeUp (/): going
                // right means going up → y_vel = -x_vel. SlopeDown (\): going
                // right means going down → y_vel = +x_vel.
                if self.player.on_ground
                    && let Some(grid) = block_grid.as_deref()
                {
                    let slope = self.grounded_slope_collision(Some(grid));
                    match slope {
                        Some(terrain::CollisionKind::SlopeUp) => {
                            self.player.y_velocity = -self.player.x_velocity;
                        }
                        Some(terrain::CollisionKind::SlopeDown) => {
                            self.player.y_velocity = self.player.x_velocity;
                        }
                        _ => {}
                    }
                }

                if let Some(ref mut t) = self.physics_trace {
                    // Record post-horizontal with unchanged x_pos (vel-only update)
                    t.post_horizontal = (self.player.x_pos, self.player.x_velocity);
                    t.horizontal_step_applied = horizontal_step_applied;
                    t.pre_vertical = (self.player.y_pos, self.player.y_velocity);
                    t.pre_gravity_y_vel = self.player.y_velocity;
                }

                // Step 2: sub_B168 — jump initiation + gravity (velocity only)
                let pre_gravity_y_vel = self.player.y_velocity;
                self.apply_jump_and_gravity_vel_only(input, jump_pressed, form, custom_gravity);
                // In AsmFaithful, gravity runs before position, so pre_gravity_y_vel
                // is the velocity before gravity was applied (same as GravityFirst).
                self.pre_gravity_y_vel = pre_gravity_y_vel;
                kid_flip_follow_on_stages.after_jump_gravity =
                    self.kid_flip_follow_on_stage_state();

                if let Some(ref mut t) = self.physics_trace {
                    t.post_gravity_y_vel = self.player.y_velocity;
                    t.gravity_applied =
                        self.player.y_velocity != pre_gravity_y_vel || !self.player.on_ground;
                    t.ascending_and_b_held = pre_gravity_y_vel < 0 && input.b;
                    if pre_gravity_y_vel >= 0 && self.player.y_velocity < 0 {
                        t.jump_initiated = true;
                        t.jump_y_vel = Some(self.player.y_velocity);
                    }
                }

                // Step 3: sub_902A — apply y_pos += y_vel, then floor/ceiling collision.
                // ASM runs sub_902A (vertical) before sub_8F26 (horizontal), so floor
                // probes see old x_pos. This is critical at ledge edges: with old x_pos
                // the probe still finds ground, preventing a 1-frame-early fall.
                let pre_vertical_y_px = (self.player.y_pos >> 16) as i32;
                self.apply_y_velocity_to_position();

                if let Some(ref mut t) = self.physics_trace {
                    t.post_vertical = (self.player.y_pos, self.player.y_velocity);
                }

                let airborne_slope_recontact = if let Some(grid) = block_grid.as_deref_mut() {
                    self.apply_ceiling_collision(
                        Some(grid),
                        block_list,
                        form,
                        collision_hw,
                        deferred_prize_ceiling_cells,
                    );
                    self.apply_floor_collision(
                        Some(grid),
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        allow_asm_faithful_solid_touch_snap,
                    )
                } else {
                    self.apply_floor_collision(
                        None,
                        horizontal_step_applied,
                        !jump_pressed,
                        collision_hw,
                        allow_asm_faithful_solid_touch_snap,
                    )
                };
                kid_flip_follow_on_stages.after_floor = self.kid_flip_follow_on_stage_state();
                if let (Some(grid), Some(collision)) =
                    (block_grid.as_deref(), airborne_slope_recontact)
                {
                    self.apply_immediate_slope_recontact(
                        input,
                        form,
                        collision,
                        grid,
                        collision_hw,
                    );
                }
                if let Some(grid) = block_grid.as_deref() {
                    self.record_wall_cling_release_probe(input, form, grid, pre_vertical_y_px);
                }
                if let Some(grid) = block_grid.as_deref()
                    && self.try_release_wall_cling_overhang(form, grid, pre_vertical_y_px)
                {
                    kid_flip_follow_on_stages.after_wall = self.kid_flip_follow_on_stage_state();
                    self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
                    self.finalise_trace();
                    self.update_camera_follow(y_look_offset);
                    return;
                }
                if let Some(grid) = block_grid.as_deref()
                    && self.try_release_descending_wall_cling_into_run(input, form, grid)
                {
                    kid_flip_follow_on_stages.after_wall = self.kid_flip_follow_on_stage_state();
                    self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
                    self.finalise_trace();
                    self.update_camera_follow(y_look_offset);
                    return;
                }

                // Step 4: sub_8F26 — apply x_pos += x_vel, then wall collision.
                self.apply_x_velocity_to_position();

                let pre_collision_x = self.player.x_pos;
                if let Some(grid) = block_grid {
                    self.apply_wall_collision(Some(grid), collision_hw);
                    if self.player.x_pos != pre_collision_x
                        && let Some(ref mut t) = self.physics_trace
                    {
                        t.wall_hit = true;
                        t.wall_snap_x = Some(self.player.x_pos);
                    }
                    if self.try_snap_wall_sticky_ledge(jump_pressed, grid, collision_hw) {
                        kid_flip_follow_on_stages.after_wall =
                            self.kid_flip_follow_on_stage_state();
                        self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
                        self.finalise_trace();
                        return;
                    }
                    // ASM sub_9386 + sub_A254: after horizontal movement, snap Y
                    // to the slope surface. The ROM stores a linear constant
                    // ($FFFFFA24) and computes y = constant ± x each frame.
                    // Native equivalent: compute slope_floor_y from x_px directly,
                    // checking both the current cell row AND one row below/above
                    // to handle tile boundary crossings during horizontal movement.
                    if self.player.on_ground {
                        let x_px = (self.player.x_pos >> 16) as i32;
                        let y_px = (self.player.y_pos >> 16) as i32;
                        let col = (x_px >> 4).max(0) as u16;
                        // ASM sub_9386 + sub_A254: snap Y to slope surface using
                        // the current x_pos. Search current row ± 1 to handle
                        // tile boundary crossings during horizontal movement.
                        for row_offset in [0i32, -1, 1] {
                            let row = ((y_px >> 4) + row_offset).max(0) as u16;
                            let cell = grid.cell_at(col, row);
                            if matches!(
                                cell.collision,
                                terrain::CollisionKind::SlopeUp | terrain::CollisionKind::SlopeDown
                            ) {
                                let cell_top = i32::from(row) * 16;
                                let floor_y =
                                    terrain::slope_floor_y_pub(cell.collision, cell_top, x_px);
                                if (y_px - floor_y).abs() <= 16 {
                                    self.player.y_pos = (floor_y.max(0) as u32) << 16;
                                    break;
                                }
                            }
                        }
                    }
                    if self.player.on_ground
                        && self.grounded_slope_collision(Some(grid)).is_none()
                        && !self
                            .walking_floor_support_exists_at_current_position(grid, collision_hw)
                    {
                        // ASM sub_B43A/loc_B5F6: once the stepped walking
                        // hitbox no longer overlaps any solid floor, the next
                        // state is the airborne jump path rather than another
                        // synthetic grounded hold.
                        self.player.y_pos = self.player.y_pos.wrapping_add(0x0001_0000);
                        self.player.on_ground = false;
                        self.player.on_spikes = false;
                        self.terrain.floor_terrain_type = TerrainType::Normal;
                    }
                }
                kid_flip_follow_on_stages.after_wall = self.kid_flip_follow_on_stage_state();
                self.kid_flip_follow_on_stages = Some(kid_flip_follow_on_stages);
            }
        }
        self.finalise_trace();
        self.update_camera_follow(y_look_offset);
    }

    fn held_horizontal_direction(input: FrameInput) -> Option<Direction> {
        match (input.left, input.right) {
            (true, false) => Some(Direction::Left),
            (false, true) => Some(Direction::Right),
            _ => None,
        }
    }

    fn kid_flip_follow_on_stage_state(&self) -> KidFlipFollowOnStageState {
        KidFlipFollowOnStageState {
            x_velocity: self.player.x_velocity,
            y_velocity: self.player.y_velocity,
            on_ground: self.player.on_ground,
            wall_cling_direction: self.player.wall_cling_direction,
        }
    }

    fn wall_cling_probe_direction(
        &self,
        held_direction: Option<Direction>,
    ) -> (Option<Direction>, KidWallClingProbeDirectionSource) {
        if let Some(direction) = self.player.wall_cling_direction {
            return (Some(direction), KidWallClingProbeDirectionSource::WallCling);
        }
        if let Some(direction) = held_direction {
            return (Some(direction), KidWallClingProbeDirectionSource::Held);
        }
        (
            Some(self.player.facing),
            KidWallClingProbeDirectionSource::Facing,
        )
    }

    fn wall_cling_direction_is_sticky(&self, direction: Direction) -> bool {
        match direction {
            Direction::Right => self.player.right_wall_sticky,
            Direction::Left => self.player.left_wall_sticky,
        }
    }

    fn record_wall_cling_release_probe(
        &mut self,
        input: FrameInput,
        form: PlayerForm,
        grid: &terrain::BlockGrid,
        pre_vertical_y_px: i32,
    ) {
        let held_direction = Self::held_horizontal_direction(input);
        let (probe_direction, probe_direction_source) =
            self.wall_cling_probe_direction(held_direction);
        let (forward_wall_probe, top_edge_probe) = match probe_direction {
            Some(direction) => {
                let moving_right = direction == Direction::Right;
                (
                    Some(terrain::probe_wall(
                        grid,
                        (self.player.x_pos >> 16) as i32,
                        (self.player.y_pos >> 16) as i32,
                        form.half_width(),
                        PLAYER_HEIGHT,
                        moving_right,
                    )),
                    Some(terrain::probe_wall_top_edge(
                        grid,
                        (self.player.x_pos >> 16) as i32,
                        (self.player.y_pos >> 16) as i32,
                        form.half_width(),
                        PLAYER_HEIGHT,
                        moving_right,
                    )),
                )
            }
            None => (None, None),
        };

        let branch = match self.player.wall_cling_direction {
            Some(direction) => {
                let pressing_away = match direction {
                    Direction::Right => input.left,
                    Direction::Left => input.right,
                };
                if pressing_away {
                    KidWallClingReleaseBranch::PressAwayExit
                } else if self.player.y_velocity < 0 {
                    KidWallClingReleaseBranch::SubAe6e
                } else {
                    KidWallClingReleaseBranch::SubAdb8
                }
            }
            None => KidWallClingReleaseBranch::NoWallClingState,
        };

        let sub_adb8_gate = match self.player.wall_cling_direction {
            None => KidWallClingReleaseGate::WallClingMissing,
            Some(direction) => {
                if !self.wall_cling_direction_is_sticky(direction) {
                    KidWallClingReleaseGate::StickyInactive
                } else if self.player.on_ground {
                    KidWallClingReleaseGate::OnGround
                } else if self.player.y_velocity < 0 {
                    KidWallClingReleaseGate::Rising
                } else if held_direction != Some(direction) {
                    KidWallClingReleaseGate::HeldDirectionMismatch
                } else if forward_wall_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::ForwardWallBlocked
                } else if top_edge_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::TopEdgeBlocked
                } else {
                    KidWallClingReleaseGate::Ready
                }
            }
        };

        let post_vertical_y_px = (self.player.y_pos >> 16) as i32;
        let upward_displacement = pre_vertical_y_px - post_vertical_y_px;
        let distance_to_tile = (-(post_vertical_y_px + 1)) & 0xF;
        let sub_ae6e_gate = match self.player.wall_cling_direction {
            None => KidWallClingReleaseGate::WallClingMissing,
            Some(direction) => {
                if !self.wall_cling_direction_is_sticky(direction) {
                    KidWallClingReleaseGate::StickyInactive
                } else if self.player.y_velocity >= 0 {
                    KidWallClingReleaseGate::NotAscending
                } else if upward_displacement <= distance_to_tile {
                    KidWallClingReleaseGate::NeedsTileCrossing
                } else if forward_wall_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::ForwardWallBlocked
                } else if top_edge_probe.is_some_and(|probe| probe.hit) {
                    KidWallClingReleaseGate::TopEdgeBlocked
                } else {
                    KidWallClingReleaseGate::Ready
                }
            }
        };

        self.wall_cling_release_probe = Some(KidWallClingReleaseProbe {
            held_direction,
            probe_direction,
            probe_direction_source,
            wall_cling_direction: self.player.wall_cling_direction,
            right_wall_sticky: self.player.right_wall_sticky,
            left_wall_sticky: self.player.left_wall_sticky,
            on_ground: self.player.on_ground,
            y_velocity: self.player.y_velocity,
            pre_vertical_y_px,
            post_vertical_y_px,
            branch,
            sub_adb8_gate,
            sub_ae6e_gate,
            forward_wall_probe,
            top_edge_probe,
        });
    }

    fn should_hold_standing_still(
        &mut self,
        input: FrameInput,
        block_grid: Option<&terrain::BlockGrid>,
        collision_hw: i32,
    ) -> bool {
        if !self.player.on_ground || self.player.x_velocity != 0 {
            return false;
        }
        let Some(direction) = Self::held_horizontal_direction(input) else {
            return false;
        };
        let Some(grid) = block_grid else {
            return false;
        };

        self.player.facing = direction;
        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        terrain::probe_wall(
            grid,
            x_px,
            y_px,
            collision_hw,
            PLAYER_HEIGHT,
            direction == Direction::Right,
        )
        .hit
    }

    fn try_release_wall_cling_overhang(
        &mut self,
        form: PlayerForm,
        grid: &terrain::BlockGrid,
        pre_vertical_y_px: i32,
    ) -> bool {
        let Some(direction) = self.player.wall_cling_direction else {
            return false;
        };
        let sticky_active = match direction {
            Direction::Right => self.player.right_wall_sticky,
            Direction::Left => self.player.left_wall_sticky,
        };
        if !sticky_active || self.player.y_velocity >= 0 {
            return false;
        }

        let y_px = (self.player.y_pos >> 16) as i32;
        let upward_displacement = pre_vertical_y_px - y_px;
        let distance_to_tile = (-(y_px + 1)) & 0xF;
        if upward_displacement <= distance_to_tile {
            return false;
        }

        // ASM jump wall-cling state keeps suppressing h_accel until the first
        // upward tile-row crossing where the overhang has actually cleared.
        // On that release frame, the sticky wall hold contributes a 1px x-pos
        // shimmy and clears the cling state, which is exactly the demo-1
        // frame-2723 frontier behaviour.
        let x_px = (self.player.x_pos >> 16) as i32;
        let moving_right = direction == Direction::Right;
        if terrain::probe_wall(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }
        if terrain::probe_wall_top_edge(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }

        match direction {
            Direction::Right => self.player.x_pos = self.player.x_pos.wrapping_add(0x0001_0000),
            Direction::Left => self.player.x_pos = self.player.x_pos.wrapping_sub(0x0001_0000),
        }
        self.player.x_velocity = 0;
        self.player.wall_cling_direction = None;
        self.player.right_wall_sticky = false;
        self.player.left_wall_sticky = false;
        true
    }

    fn try_release_descending_wall_cling_into_run(
        &mut self,
        input: FrameInput,
        form: PlayerForm,
        grid: &terrain::BlockGrid,
    ) -> bool {
        let Some(direction) = self.player.wall_cling_direction else {
            return false;
        };
        let sticky_active = match direction {
            Direction::Right => self.player.right_wall_sticky,
            Direction::Left => self.player.left_wall_sticky,
        };
        if !sticky_active || self.player.on_ground || self.player.y_velocity < 0 {
            return false;
        }
        if Self::held_horizontal_direction(input) != Some(direction) {
            return false;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let moving_right = direction == Direction::Right;
        // ASM loc_AA9C/loc_AD34: while descending in wall-cling, holding back
        // into the wall exits cling with a 1px shimmy once the wall/top-edge
        // probes both clear, so the next frame can re-enter jump air-accel.
        if terrain::probe_wall(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }
        if terrain::probe_wall_top_edge(
            grid,
            x_px,
            y_px,
            form.half_width(),
            PLAYER_HEIGHT,
            moving_right,
        )
        .hit
        {
            return false;
        }

        match direction {
            Direction::Right => self.player.x_pos = self.player.x_pos.wrapping_add(0x0001_0000),
            Direction::Left => self.player.x_pos = self.player.x_pos.wrapping_sub(0x0001_0000),
        }
        self.player.x_velocity = 0;
        self.player.wall_cling_direction = None;
        self.player.right_wall_sticky = false;
        self.player.left_wall_sticky = false;
        true
    }

    /// Record final state into the physics trace and detect floor landing.
    fn finalise_trace(&mut self) {
        if let Some(ref mut t) = self.physics_trace {
            t.on_ground_after = self.player.on_ground;
            t.final_pos = (self.player.x_pos, self.player.y_pos);
            t.final_vel = (self.player.x_velocity, self.player.y_velocity);
            // Detect floor landing: transitioned from airborne to grounded
            if !t.on_ground_before && t.on_ground_after {
                t.floor_hit = true;
                t.floor_y_px = Some((self.terrain.floor_y_pos >> 16) as i32);
                // sub_902A clears sub-pixel on new landings (clr.w $20(a3))
                t.landing_path = Some(LandingPath::ZeroSubPixel);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step_iron_knight_climb_resume(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        y_look_offset: i16,
        allow_asm_faithful_solid_touch_snap: bool,
        deferred_prize_ceiling_cells: &[(u16, u16, DeferredPrizeBlockPhase)],
    ) {
        let stage = self.kid_flip_follow_on_stage_state();
        self.kid_flip_follow_on_stages = Some(KidFlipFollowOnStages {
            entry: stage,
            after_press_away: stage,
            after_jump_gravity: stage,
            after_floor: stage,
            after_wall: stage,
        });
        self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
            branch: KidJumpGravityBranch::AirborneCustomGravity,
            jump_pressed,
            input_b_held: input.b,
            custom_gravity: true,
            on_ground_before: self.player.on_ground,
            on_ground_after: self.player.on_ground,
            just_jumped_before: self.player.just_jumped,
            just_jumped_after: self.player.just_jumped,
            jump_buffer_frames_before: self.player.jump_buffer_frames,
            jump_buffer_frames_after: self.player.jump_buffer_frames,
            y_velocity_before: self.player.y_velocity,
            y_velocity_after: self.player.y_velocity,
        });

        if let Some(ref mut trace) = self.physics_trace {
            trace.pre_horizontal = (self.player.x_pos, self.player.x_velocity);
            trace.post_horizontal = (self.player.x_pos, self.player.x_velocity);
            trace.horizontal_step_applied = false;
            trace.pre_vertical = (self.player.y_pos, self.player.y_velocity);
            trace.pre_gravity_y_vel = self.player.y_velocity;
            trace.post_gravity_y_vel = self.player.y_velocity;
            trace.gravity_applied = false;
            trace.ascending_and_b_held = false;
        }
        self.pre_gravity_y_vel = self.player.y_velocity;

        let pre_vertical_y_px = (self.player.y_pos >> 16) as i32;
        self.apply_y_velocity_to_position();
        if let Some(ref mut trace) = self.physics_trace {
            trace.post_vertical = (self.player.y_pos, self.player.y_velocity);
        }

        if let Some(grid) = block_grid {
            self.apply_ceiling_collision(
                Some(grid),
                block_list,
                form,
                form.half_width(),
                deferred_prize_ceiling_cells,
            );
            self.apply_floor_collision(
                Some(grid),
                false,
                !jump_pressed,
                form.half_width(),
                allow_asm_faithful_solid_touch_snap,
            );
            self.record_wall_cling_release_probe(input, form, grid, pre_vertical_y_px);
        } else {
            self.apply_floor_collision(
                None,
                false,
                !jump_pressed,
                form.half_width(),
                allow_asm_faithful_solid_touch_snap,
            );
        }

        self.finalise_trace();
        self.update_camera_follow(y_look_offset);
    }

    pub(crate) fn try_snap_wall_sticky_ledge(
        &mut self,
        jump_pressed: bool,
        grid: &terrain::BlockGrid,
        collision_hw: i32,
    ) -> bool {
        let jump_requested = jump_pressed || self.player.jump_buffer_frames > 0;
        if !jump_requested || self.player.on_ground {
            return false;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let (probe_x_px, direct_probe, snap_x_px) = if self.player.right_wall_sticky {
            let probe_x_px = x_px + collision_hw + 1;
            let probe = terrain::probe_floor(grid, probe_x_px, y_px);
            let support_left_px = i32::from(probe.cell_x) * 16;
            (probe_x_px, probe, support_left_px + collision_hw - 1)
        } else if self.player.left_wall_sticky {
            let probe_x_px = x_px - collision_hw - 1;
            let probe = terrain::probe_floor(grid, probe_x_px, y_px);
            let support_right_px = (i32::from(probe.cell_x) + 1) * 16 - 1;
            (probe_x_px, probe, support_right_px - collision_hw + 1)
        } else {
            return false;
        };

        let probe = if direct_probe.cell.is_solid_floor() {
            direct_probe
        } else if self.player.y_velocity >= 0 && self.player.jump_buffer_frames > 0 {
            // ASM pull-up path (loc_ABA4) searches downward from roughly 20px
            // above the current feet position and snaps onto the first solid
            // ledge it finds while the wall-cling jump request is still live.
            let search_start_y_px = y_px.saturating_sub(0x14) & !0xF;
            let Some(surface_y_px) = terrain::find_floor_below(grid, probe_x_px, search_start_y_px)
            else {
                return false;
            };
            let scanned_probe = terrain::probe_floor(grid, probe_x_px, surface_y_px);
            if !scanned_probe.cell.is_solid_floor() {
                return false;
            }
            scanned_probe
        } else {
            return false;
        };

        let (floor_y_px, contact_y_px) = match probe.cell.collision {
            terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes => {
                (probe.floor_y_px.saturating_sub(1), probe.floor_y_px)
            }
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::HiddenBlock
            | terrain::CollisionKind::Empty => (probe.floor_y_px, probe.floor_y_px),
        };
        let contact_y_fp = (contact_y_px.max(0) as u32) << 16;
        let valid_contact_window = if self.player.y_velocity < 0 {
            self.player.y_pos >= contact_y_fp
                && self.player.y_pos <= contact_y_fp.saturating_add(15u32 << 16)
        } else {
            self.player.y_pos >= contact_y_fp
                && self.player.y_pos <= contact_y_fp.saturating_add(18u32 << 16)
        };
        if !valid_contact_window {
            return false;
        }

        let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
        self.player.x_pos = (snap_x_px.max(0) as u32) << 16;
        // Preserve sub-pixel from current position: ASM's ledge snap uses
        // move.w to set the pixel part, leaving the low word (sub-pixel) intact.
        self.player.y_pos = (floor_y_fp & 0xFFFF_0000) | (self.player.y_pos & 0x0000_FFFF);
        self.player.x_velocity = 0;
        self.player.y_velocity = 0;
        self.player.on_ground = true;
        self.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
        self.player.right_wall_sticky = false;
        self.player.left_wall_sticky = false;
        self.player.wall_cling_direction = None;
        // ASM has no jump buffer — the VBlank interrupt's Ctrl_1_Press is a
        // single-frame edge. Clear the buffer on ledge snap to prevent a
        // spurious jump on the next grounded frame.
        self.player.jump_buffer_frames = 0;
        self.player.ledge_snap_cooldown = LEDGE_SNAP_PULLUP_FRAMES;
        self.terrain.floor_y_pos = floor_y_fp;
        self.terrain.floor_terrain_type =
            grounded_surface_type_sub_8600(grid, x_px, y_px, collision_hw);
        true
    }

    /// Horizontal velocity update only (sub_B084 equivalent).
    /// Updates x_vel and facing but does NOT apply x_pos += x_vel.
    /// Returns (horizontal_step_applied, jump_suppressed_by_buffer).
    pub(crate) fn update_horizontal_velocity_only(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&terrain::BlockGrid>,
    ) -> bool {
        if self.should_hold_standing_still(input, block_grid, form.half_width()) {
            return false;
        }
        if self.player.on_ground && self.player.jump_buffer_frames > 0 && !input.b {
            self.player.jump_buffer_frames = 0;
        }
        let grounded_slope = self
            .grounded_slope_collision(block_grid)
            .filter(|_| self.player.on_ground);
        if self.player.on_ground && self.player.jump_buffer_frames > 0 {
            return false;
        }
        if self.player.on_ground && jump_pressed {
            if let Some(collision) = grounded_slope {
                let h_input = HorizontalInput {
                    left: input.left,
                    right: input.right,
                    run: input.a,
                };
                let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                );
                self.player.x_velocity = new_vel;
                self.player.facing = new_facing;
            }
            return false;
        }

        let near_floor =
            self.player.y_pos.abs_diff(self.terrain.floor_y_pos) <= GROUND_SNAP_TOLERANCE_FP;
        if !self.player.on_ground && jump_pressed && self.player.y_velocity >= 0 && near_floor {
            return false;
        }

        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = if self.player.on_ground {
            if let Some(collision) = grounded_slope {
                update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                )
            } else if form == PlayerForm::Skycutter {
                physics::update_skycutter_horizontal_velocity_sub_9A20(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                )
            } else {
                physics::update_horizontal_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    form.ground_speed_table(self.terrain.floor_terrain_type),
                )
            }
        } else if self.player.just_jumped {
            (self.player.x_velocity, self.player.facing)
        } else {
            let jump_profile = physics::JUMP_AIR_PROFILES[usize::from(form.helmet_id())];
            physics::update_jump_horizontal_velocity(
                self.player.x_velocity,
                self.player.facing,
                h_input,
                jump_profile,
            )
        };
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;
        true
    }

    /// Jump initiation + gravity on y_vel only (sub_B168 equivalent).
    /// Does NOT update x_pos or y_pos — only velocity fields.
    pub(crate) fn apply_jump_and_gravity_vel_only(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        custom_gravity: bool,
    ) {
        let before = self.player;
        // Jump initiation
        if self.player.on_ground && jump_pressed {
            // ASM loc_9F18 -> loc_B580 -> loc_A426: grounded slope jumps clear the
            // temporary slope-tracking y_vel before sub_A432 launches the jump.
            // Native mirrors x_vel into y_vel earlier in the frame, so the
            // grounded jump gate cannot require y_vel >= 0.
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
                branch: KidJumpGravityBranch::JumpPressed,
                jump_pressed,
                input_b_held: input.b,
                custom_gravity,
                on_ground_before: before.on_ground,
                on_ground_after: self.player.on_ground,
                just_jumped_before: before.just_jumped,
                just_jumped_after: self.player.just_jumped,
                jump_buffer_frames_before: before.jump_buffer_frames,
                jump_buffer_frames_after: self.player.jump_buffer_frames,
                y_velocity_before: before.y_velocity,
                y_velocity_after: self.player.y_velocity,
            });
            return;
        }

        // Jump buffer initiation
        if self.player.on_ground && self.player.jump_buffer_frames > 0 {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
                branch: KidJumpGravityBranch::JumpBuffered,
                jump_pressed,
                input_b_held: input.b,
                custom_gravity,
                on_ground_before: before.on_ground,
                on_ground_after: self.player.on_ground,
                just_jumped_before: before.just_jumped,
                just_jumped_after: self.player.just_jumped,
                jump_buffer_frames_before: before.jump_buffer_frames,
                jump_buffer_frames_after: self.player.jump_buffer_frames,
                y_velocity_before: before.y_velocity,
                y_velocity_after: self.player.y_velocity,
            });
            return;
        }

        let branch = if !self.player.on_ground {
            if self.player.just_jumped {
                KidJumpGravityBranch::AirborneJustJumpedHold
            } else if custom_gravity {
                KidJumpGravityBranch::AirborneCustomGravity
            } else if self.player.y_velocity < 0 && input.b {
                KidJumpGravityBranch::AirborneHeldJumpGravity
            } else {
                KidJumpGravityBranch::AirborneDefaultGravity
            }
        } else {
            KidJumpGravityBranch::GroundedHold
        };

        // Airborne: apply gravity to y_vel only
        if !self.player.on_ground {
            if jump_pressed {
                self.player.jump_buffer_frames = JUMP_BUFFER_FRAMES;
            } else if self.player.jump_buffer_frames > 0 {
                self.player.jump_buffer_frames -= 1;
            }
            if self.player.just_jumped {
                self.player.just_jumped = false;
            } else if !custom_gravity {
                let ascending_and_b_held = self.player.y_velocity < 0 && input.b;
                self.player.y_velocity =
                    physics::apply_gravity(self.player.y_velocity, ascending_and_b_held);
            }
        } else {
            self.player.y_velocity = 0;
            self.player.just_jumped = false;
        }

        self.kid_jump_gravity_probe = Some(KidJumpGravityProbe {
            branch,
            jump_pressed,
            input_b_held: input.b,
            custom_gravity,
            on_ground_before: before.on_ground,
            on_ground_after: self.player.on_ground,
            just_jumped_before: before.just_jumped,
            just_jumped_after: self.player.just_jumped,
            jump_buffer_frames_before: before.jump_buffer_frames,
            jump_buffer_frames_after: self.player.jump_buffer_frames,
            y_velocity_before: before.y_velocity,
            y_velocity_after: self.player.y_velocity,
        });
    }

    /// Apply y_vel to y_pos only. ASM sub_902A applies vertical position
    /// before sub_8F26 applies horizontal, so floor probes see old x_pos.
    fn apply_y_velocity_to_position(&mut self) {
        let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
        self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
    }

    /// Apply x_vel to x_pos only. ASM sub_8F26 runs after sub_902A.
    fn apply_x_velocity_to_position(&mut self) {
        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
    }

    /// Physics-driven horizontal movement using speed tables.
    /// Uses ground tables when on_ground, air tables when airborne.
    pub(crate) fn update_horizontal_motion(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        block_grid: Option<&terrain::BlockGrid>,
    ) -> bool {
        if self.should_hold_standing_still(input, block_grid, form.half_width()) {
            return false;
        }
        if self.player.on_ground && self.player.jump_buffer_frames > 0 && !input.b {
            self.player.jump_buffer_frames = 0;
        }
        let grounded_slope = self
            .grounded_slope_collision(block_grid)
            .filter(|_| self.player.on_ground);
        if self.player.on_ground && self.player.jump_buffer_frames > 0 {
            return false;
        }
        if self.player.on_ground && jump_pressed {
            if let Some(collision) = grounded_slope {
                let h_input = HorizontalInput {
                    left: input.left,
                    right: input.right,
                    run: input.a,
                };
                let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                );
                self.player.x_velocity = new_vel;
                self.player.facing = new_facing;
            }
            return false;
        }

        let near_floor =
            self.player.y_pos.abs_diff(self.terrain.floor_y_pos) <= GROUND_SNAP_TOLERANCE_FP;
        if !self.player.on_ground && jump_pressed && self.player.y_velocity >= 0 && near_floor {
            return false;
        }

        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = if self.player.on_ground {
            if let Some(collision) = grounded_slope {
                update_level_one_slope_ground_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    collision,
                    form,
                    self.terrain.floor_terrain_type,
                )
            } else if form == PlayerForm::Skycutter {
                physics::update_skycutter_horizontal_velocity_sub_9A20(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                )
            } else {
                physics::update_horizontal_velocity(
                    self.player.x_velocity,
                    self.player.facing,
                    h_input,
                    form.ground_speed_table(self.terrain.floor_terrain_type),
                )
            }
        } else if self.player.just_jumped {
            (self.player.x_velocity, self.player.facing)
        } else {
            let jump_profile = physics::JUMP_AIR_PROFILES[usize::from(form.helmet_id())];
            physics::update_jump_horizontal_velocity(
                self.player.x_velocity,
                self.player.facing,
                h_input,
                jump_profile,
            )
        };
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;

        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
        true
    }

    pub(crate) fn grounded_slope_collision(
        &self,
        block_grid: Option<&terrain::BlockGrid>,
    ) -> Option<terrain::CollisionKind> {
        let grid = block_grid?;
        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        match terrain::probe_floor(grid, x_px, y_px).cell.collision {
            terrain::CollisionKind::SlopeDown => Some(terrain::CollisionKind::SlopeDown),
            terrain::CollisionKind::SlopeUp => Some(terrain::CollisionKind::SlopeUp),
            _ => None,
        }
    }

    fn walking_floor_support_exists_at_current_position(
        &self,
        grid: &terrain::BlockGrid,
        collision_hw: i32,
    ) -> bool {
        let x_int = (self.player.x_pos >> 16) as i16 as i32;
        let y_int = (self.player.y_pos >> 16) as i16 as i32;
        let probe_row = ((y_int + 1) >> 4).max(0) as u16;
        let left_col = ((x_int - collision_hw) >> 4).max(0) as u16;
        let right_col = ((x_int + collision_hw) >> 4)
            .max(0)
            .min(grid.width_cells as i32 - 1) as u16;
        (left_col..=right_col).any(|col| {
            matches!(
                grid.cell_at(col, probe_row).collision,
                terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
            )
        })
    }

    /// Physics-driven vertical movement: jump initiation and gravity.
    pub(crate) fn update_vertical_motion(
        &mut self,
        input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
        custom_gravity: bool,
    ) {
        if self.player.on_ground && jump_pressed {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        if self.player.on_ground && self.player.jump_buffer_frames > 0 {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        if !self.player.on_ground {
            if jump_pressed {
                self.player.jump_buffer_frames = JUMP_BUFFER_FRAMES;
            } else if self.player.jump_buffer_frames > 0 {
                self.player.jump_buffer_frames -= 1;
            }
            if self.player.just_jumped {
                self.player.just_jumped = false;
            } else if !custom_gravity {
                // Normal gravity; helmet flight abilities (Skycutter/Cyclone)
                // apply their own gravity before physics runs.
                let ascending_and_b_held = self.player.y_velocity < 0 && input.b;
                self.player.y_velocity =
                    physics::apply_gravity(self.player.y_velocity, ascending_and_b_held);
            }
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
        } else {
            self.player.y_velocity = 0;
            self.player.just_jumped = false;
        }
    }

    /// PositionFirst mode: apply y_pos from y_velocity (and handle jump initiation)
    /// but defer gravity to after collisions.
    pub(crate) fn apply_vertical_position_only(
        &mut self,
        _input: FrameInput,
        jump_pressed: bool,
        form: PlayerForm,
    ) {
        // Jump initiation (same as update_vertical_motion)
        if self.player.on_ground && jump_pressed {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        // Jump buffer initiation
        if self.player.on_ground && self.player.jump_buffer_frames > 0 {
            let jump_height = physics::JUMP_HEIGHTS[usize::from(form.helmet_id())];
            let (new_x_vel, y_vel) = physics::initiate_jump(self.player.x_velocity, jump_height);
            self.player.x_velocity = new_x_vel;
            self.player.y_velocity = y_vel;
            self.player.on_ground = false;
            self.player.just_jumped = false;
            self.player.jump_buffer_frames = 0;
            let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
            self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
            return;
        }

        // Airborne: apply position from current velocity (no gravity yet)
        if !self.player.on_ground {
            if jump_pressed {
                self.player.jump_buffer_frames = JUMP_BUFFER_FRAMES;
            } else if self.player.jump_buffer_frames > 0 {
                self.player.jump_buffer_frames -= 1;
            }
            if self.player.just_jumped {
                self.player.just_jumped = false;
            }
            // Position update only — gravity is deferred
            let next_y = i64::from(self.player.y_pos) + i64::from(self.player.y_velocity);
            self.player.y_pos = next_y.clamp(0, i64::from(u32::MAX)) as u32;
        } else {
            self.player.y_velocity = 0;
            self.player.just_jumped = false;
        }
    }

    pub(crate) fn apply_wall_collision(
        &mut self,
        block_grid: Option<&terrain::BlockGrid>,
        collision_hw: i32,
    ) {
        let Some(grid) = block_grid else {
            self.player.right_wall_sticky = false;
            self.player.left_wall_sticky = false;
            return;
        };
        if self.player.x_velocity == 0 {
            if self.player.on_ground {
                self.player.right_wall_sticky = false;
                self.player.left_wall_sticky = false;
            }
            return;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let moving_right = self.player.x_velocity > 0;
        let moving_left = self.player.x_velocity < 0;
        if !moving_right || self.player.on_ground {
            self.player.right_wall_sticky = false;
        }
        if !moving_left || self.player.on_ground {
            self.player.left_wall_sticky = false;
        }

        // ASM sub_8F26 tile-boundary guard (kid.asm:10335-10341): skip wall probe
        // if pixel displacement doesn't cross a 16px tile boundary. Without this,
        // a false probe hit zeroes sub-pixel via x_pos = clamped_center << 16.
        // Suppressed when wall_sticky is set (first frame after wall hit needs
        // re-probe even though x_vel=0; subsequent wall-cling frames with
        // x_vel=0 are handled earlier in the jump loop).
        let wall_stuck = (moving_right && self.player.right_wall_sticky)
            || (moving_left && self.player.left_wall_sticky);
        if !wall_stuck {
            let old_x_pos = self
                .player
                .x_pos
                .wrapping_sub(self.player.x_velocity as u32);
            let old_px = (old_x_pos >> 16) as i32;
            let new_px = (self.player.x_pos >> 16) as i32;
            if moving_right {
                let displacement = new_px - old_px;
                let right_edge = old_px + collision_hw;
                let distance_to_tile = (-(right_edge + 1)) & 0xF;
                // ASM: cmp.w d0,d4 / ble (skip when displacement <= distance).
                // Includes distance=0 case (player at tile boundary, no crossing).
                if displacement <= distance_to_tile {
                    return;
                }
            } else {
                let displacement = old_px - new_px;
                let left_edge = old_px - collision_hw;
                let distance_to_tile = left_edge & 0xF;
                if displacement <= distance_to_tile {
                    return;
                }
            }
        }

        // ASM sub_8F26 only latches wall-cling from the direct side-wall probe
        // (`sub_922C`). A top-edge-only overhang contact is used later by
        // loc_AAC8/sub_ADB8/sub_AE6E to retain an existing sticky cling, not to
        // create a fresh wall hit. Promoting `probe_wall_top_edge` into a new
        // collision source falsely attached Kid on the level-0x15 frame-24921
        // frontier where the oracle keeps drifting right past the overhang.
        let mut probe =
            terrain::probe_wall(grid, x_px, y_px, collision_hw, PLAYER_HEIGHT, moving_right);

        // Right wall top-edge retention (overhang sticky)
        if !probe.hit && moving_right && self.player.right_wall_sticky {
            let top_edge_probe =
                terrain::probe_wall_top_edge(grid, x_px, y_px, collision_hw, PLAYER_HEIGHT, true);
            if top_edge_probe.hit {
                probe = top_edge_probe;
            } else {
                self.player.x_pos = ((self.player.x_pos >> 16) + 1) << 16;
                self.player.x_velocity = 0;
                self.player.right_wall_sticky = false;
                return;
            }
        }

        // Left wall top-edge retention (overhang sticky)
        if !probe.hit && moving_left && self.player.left_wall_sticky {
            let top_edge_probe =
                terrain::probe_wall_top_edge(grid, x_px, y_px, collision_hw, PLAYER_HEIGHT, false);
            if top_edge_probe.hit {
                probe = top_edge_probe;
            } else {
                self.player.x_pos = ((self.player.x_pos >> 16) - 1) << 16;
                self.player.x_velocity = 0;
                self.player.left_wall_sticky = false;
                return;
            }
        }

        if probe.hit {
            if probe.cell.has_block && probe.cell.block_type == terrain::BlockType::Rubber {
                // ASM sub_8F26 -> sub_922C -> off_933E index 6 -> loc_10DA4:
                // side contact with a rubber block takes sub_8F26's special
                // `d7=-1` return, so the just-applied horizontal step is
                // clamped, the bounce effect is queued, and sufficiently fast
                // airborne motion rebounds instead of latching wall-cling.
                self.rubber_block_hit = Some((probe.cell_x, probe.cell_y));
            }
            let clamped_center = if moving_right {
                probe.wall_x_px - (collision_hw + 1)
            } else {
                // Leftward wall probes test the current left edge pixel
                // (`center_x - half_width`), so the flush non-colliding centre
                // sits exactly `half_width` pixels to the right of the wall
                // surface. The extra `+1` belongs only to the moving-right path,
                // which probes one pixel beyond the right edge.
                probe.wall_x_px + collision_hw
            };
            if moving_right && x_px < clamped_center {
                self.player.right_wall_sticky = false;
                return;
            }
            if moving_left && x_px > clamped_center {
                self.player.left_wall_sticky = false;
                return;
            }
            self.player.x_pos = (clamped_center.max(0) as u32) << 16;
            let incoming_x_velocity = self.player.x_velocity;
            self.player.x_velocity = 0;
            if probe.cell.has_block
                && probe.cell.block_type == terrain::BlockType::Rubber
                && !self.player.on_ground
            {
                self.player.wall_cling_direction = None;
                self.player.right_wall_sticky = false;
                self.player.left_wall_sticky = false;
                if incoming_x_velocity.unsigned_abs() >= 0x0000_8000 {
                    self.player.x_velocity = -incoming_x_velocity;
                }
                return;
            }
            self.player.right_wall_sticky = moving_right && !self.player.on_ground;
            self.player.left_wall_sticky = moving_left && !self.player.on_ground;
            // ASM loc_A8DE: enter wall-cling on airborne wall hit. Suppresses
            // sub_B084 (h_accel) in subsequent frames until landing or press-away.
            if !self.player.on_ground {
                self.player.wall_cling_direction = Some(if moving_right {
                    Direction::Right
                } else {
                    Direction::Left
                });
            }
        } else {
            self.player.right_wall_sticky = false;
            self.player.left_wall_sticky = false;
        }
    }

    pub(crate) fn apply_ceiling_collision(
        &mut self,
        block_grid: Option<&mut terrain::BlockGrid>,
        block_list: Option<&terrain::BlockList>,
        form: PlayerForm,
        collision_hw: i32,
        deferred_prize_ceiling_cells: &[(u16, u16, DeferredPrizeBlockPhase)],
    ) {
        let Some(grid) = block_grid else { return };
        if self.player.y_velocity >= 0 {
            return;
        }

        // ASM sub_902A upward path only probes the ceiling if the current
        // movement actually crosses the next tile boundary above the head.
        // Without this guard, we keep re-hitting the same ceiling tile every
        // frame while still near it, producing a false multi-frame latch.
        let collision_height = form.ceiling_probe_height();
        let old_y_pos = self
            .player
            .y_pos
            .wrapping_sub(self.player.y_velocity as u32);
        let old_y_px = (old_y_pos >> 16) as i32;
        let new_y_px = (self.player.y_pos >> 16) as i32;
        let upward_displacement = old_y_px - new_y_px;
        let top_edge_old = old_y_px - collision_height;
        let distance_to_tile = top_edge_old & 0xF;
        if upward_displacement <= distance_to_tile {
            return;
        }

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;

        let probe = terrain::probe_ceiling(grid, x_px, y_px, collision_hw, collision_height);

        if probe.hit {
            let deferred_prize_ceiling_phase =
                deferred_prize_ceiling_cells
                    .iter()
                    .find_map(|&(col, row, phase)| {
                        (col == probe.cell_x && row == probe.cell_y).then_some(phase)
                    });
            if deferred_prize_ceiling_phase == Some(DeferredPrizeBlockPhase::FreshSmash) {
                return;
            }
            let deferred_prize_ceiling_cell = deferred_prize_ceiling_phase.is_some();
            // Hidden block reveal: headbutting a hidden block from below
            // changes it to solid, making the block visible and functional.
            if !deferred_prize_ceiling_cell
                && probe.cell.collision == terrain::CollisionKind::HiddenBlock
            {
                let layout_addr = ((u32::from(probe.cell_y) * u32::from(grid.width_cells))
                    + u32::from(probe.cell_x))
                    * 2;
                let revealed_cell = block_list
                    .and_then(|bl| bl.hidden_block_by_layout_addr_sub_E37A(layout_addr as u16))
                    .and_then(|entry| {
                        // ASM sub_E3C6 derives the revealed cell from the
                        // hidden-block metadata record rather than trusting the
                        // placeholder HiddenBlock cell still in Level_Layout.
                        block_list.and_then(|bl| bl.hidden_block_reveal_sub_E3C6(entry))
                    })
                    .map(|reveal| reveal.cell)
                    .unwrap_or_else(|| {
                        // Fallback for native-only fixtures that do not carry a
                        // full ROM Block List entry: preserve the existing block
                        // type/skin and only flip the collision to Solid.
                        let block_flag: u8 = if probe.cell.has_block { 0x80 } else { 0 };
                        let block_type_bits = probe.cell.block_type as u8 & 0x0F;
                        let new_hi = block_flag | 0x60 | block_type_bits;
                        terrain::Cell::from_be_bytes(new_hi, probe.cell.skin)
                    });
                grid.set_cell_at(probe.cell_x, probe.cell_y, revealed_cell);

                // If the hidden block contains a prize, dispatch it
                if probe.cell.has_block && probe.cell.block_type == terrain::BlockType::Prize {
                    let content = block_list
                        .and_then(|bl| bl.prize_content_at(probe.cell_x, probe.cell_y))
                        .unwrap_or(terrain::PrizeContent::Diamond);
                    let block_x_px = probe.cell_x * 16;
                    let block_y_px = probe.cell_y * 16;
                    self.prize_block_hit = Some((content, block_x_px, block_y_px));
                    // ASM sub_1007A: keep cell Solid, clear has_block
                    grid.set_cell_at(
                        probe.cell_x,
                        probe.cell_y,
                        terrain::Cell::from_be_bytes(0x6F, probe.cell.skin),
                    );
                    self.ceiling_prize_clear = Some((probe.cell_x, probe.cell_y, probe.cell));
                }
            }

            // Prize/smash/lift block headbutt: hitting from below.
            if !deferred_prize_ceiling_cell
                && probe.cell.collision == terrain::CollisionKind::Solid
                && probe.cell.has_block
            {
                match probe.cell.block_type {
                    terrain::BlockType::Prize => {
                        let content = block_list
                            .and_then(|bl| bl.prize_content_at(probe.cell_x, probe.cell_y))
                            .unwrap_or(terrain::PrizeContent::Diamond);
                        let block_x_px = probe.cell_x * 16;
                        let block_y_px = probe.cell_y * 16;
                        self.prize_block_hit = Some((content, block_x_px, block_y_px));
                        // ASM sub_1007A writes $6F to the cell: Solid collision,
                        // has_block=false, block_type=Deleted. The tile stays solid
                        // as structural ceiling for ~32 frames (bounce animation),
                        // then clears. Without this, Cyclone flight near a Prize
                        // ceiling clears the tile to Empty immediately, letting
                        // the player fly through.
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0x6F, probe.cell.skin),
                        );
                        self.ceiling_prize_clear = Some((probe.cell_x, probe.cell_y, probe.cell));
                        // ASM sub_914A scans ALL tiles across player width, so
                        // adjacent Smash blocks are cleared on the same frame.
                        // probe_ceiling returns on the first hit, missing the
                        // other edge. Clear adjacent Smash blocks now.
                        let top_y = y_px - collision_height;
                        let tile_row = (top_y / 16).max(0) as u16;
                        for &edge_x in &[x_px - collision_hw, x_px + collision_hw] {
                            let col = (edge_x / 16).max(0) as u16;
                            if col == probe.cell_x && tile_row == probe.cell_y {
                                continue;
                            }
                            let cell = grid.cell_at(col, tile_row);
                            if cell.collision == terrain::CollisionKind::Solid
                                && cell.has_block
                                && cell.block_type == terrain::BlockType::Smash
                            {
                                grid.set_cell_at(col, tile_row, terrain::Cell::from_be_bytes(0, 0));
                            }
                        }
                    }
                    terrain::BlockType::Smash => {
                        let block_x_px = probe.cell_x * 16;
                        let block_y_px = probe.cell_y * 16;
                        self.smash_block_hit = Some((block_x_px, block_y_px));
                        // ASM j_Block_LoadSmashBits clears Smash blocks immediately
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0, 0),
                        );
                    }
                    terrain::BlockType::Lift => {
                        let block_x_px = probe.cell_x * 16;
                        let block_y_px = probe.cell_y * 16;
                        self.lift_block_hit = Some((block_x_px, block_y_px));
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0, 0),
                        );
                    }
                    terrain::BlockType::Drill => {
                        let drill_x_fp = (u32::from(probe.cell_x) * 16 + 8) << 16;
                        let drill_y_fp = (u32::from(probe.cell_y) * 16 + 16) << 16;
                        self.drill_projectile_hit = Some((drill_x_fp, drill_y_fp));
                        grid.set_cell_at(
                            probe.cell_x,
                            probe.cell_y,
                            terrain::Cell::from_be_bytes(0, 0),
                        );
                    }
                    terrain::BlockType::Metal => {
                        self.metal_block_hit = Some((probe.cell_x, probe.cell_y));
                    }
                    terrain::BlockType::Shooter => {
                        // ASM loc_BBC2 -> loc_11430 routes shooter ceiling
                        // hits into the deleted-tile worker coroutines instead
                        // of clearing the block immediately.
                        self.shooter_block_hit = Some((probe.cell_x, probe.cell_y));
                    }
                    terrain::BlockType::Rubber => {
                        // ASM sub_902A -> sub_914A -> loc_91F6/loc_10DA4:
                        // ceiling contact with rubber also spawns the jiggle
                        // effect without deleting the block.
                        self.rubber_block_hit = Some((probe.cell_x, probe.cell_y));
                    }
                    _ => {}
                }
            }

            // Push feet down so head clears the ceiling: feet = ceiling_bottom + height
            let new_y = probe.ceiling_y_px + collision_height;
            self.player.y_pos = (new_y.max(0) as u32) << 16;
            // ASM sub_902A ceiling path (loc_908C): clr.l y_vel(a3) then
            // clr.w $20(a3) (zero sub-pixel). y_pos is already pixel-aligned
            // from the << 16 above (sub-pixel zeroed). sub_BB54 also re-zeros
            // y_vel at line 15092.
            self.player.y_velocity = 0;
        }
    }

    pub(crate) fn apply_floor_collision(
        &mut self,
        block_grid: Option<&terrain::BlockGrid>,
        horizontal_step_applied: bool,
        allow_upward_slope_snap: bool,
        collision_hw: i32,
        allow_solid_touch_snap: bool,
    ) -> Option<terrain::CollisionKind> {
        if let Some(grid) = block_grid {
            let was_on_ground = self.player.on_ground;
            let x_px = (self.player.x_pos >> 16) as i32;
            let y_px = (self.player.y_pos >> 16) as i32;
            let left_edge_probe = terrain::probe_floor(grid, x_px - collision_hw, y_px);
            let right_edge_probe = terrain::probe_floor(grid, x_px + collision_hw, y_px);
            let mut probe = terrain::probe_floor_hitbox(grid, x_px, y_px, collision_hw);
            if was_on_ground
                && matches!(
                    probe.cell.collision,
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                )
            {
                let left_edge_support =
                    left_edge_probe.on_ground || left_edge_probe.cell.is_solid_floor();
                let right_edge_support =
                    right_edge_probe.on_ground || right_edge_probe.cell.is_solid_floor();
                let edge_probe = match (left_edge_support, right_edge_support) {
                    (true, true) => {
                        if left_edge_probe.floor_y_px >= right_edge_probe.floor_y_px {
                            Some(left_edge_probe)
                        } else {
                            Some(right_edge_probe)
                        }
                    }
                    (true, false) => Some(left_edge_probe),
                    (false, true) => Some(right_edge_probe),
                    (false, false) => None,
                };
                if let Some(edge_probe) = edge_probe
                    && edge_probe.floor_y_px > probe.floor_y_px
                {
                    // ASM sub_914A keeps the previously grounded support under
                    // either hitbox edge while a flat-to-slope seam is still
                    // carrying Kid. The death-respawn Level 1 run-up stays on
                    // the deeper flat plateau through frame 1012 instead of
                    // dropping onto the adjacent slope early.
                    probe = edge_probe;
                }
            }
            let mut allow_hitbox_snap = true;
            let mut center_slope_snap_candidate = false;
            if !was_on_ground {
                let center_probe = terrain::probe_floor(grid, x_px, y_px);
                if matches!(
                    center_probe.cell.collision,
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                ) {
                    let center_floor_fp = (center_probe.floor_y_px.max(0) as u32) << 16;
                    let near_center_slope = self.player.y_pos >= center_floor_fp
                        || self.player.y_pos.abs_diff(center_floor_fp)
                            <= SLOPE_AIR_SNAP_TOLERANCE_FP;
                    if near_center_slope {
                        probe = center_probe;
                        center_slope_snap_candidate = true;
                    } else {
                        allow_hitbox_snap = matches!(
                            left_edge_probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        ) || matches!(
                            right_edge_probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        );
                    }
                }
            }
            let preserving_ground_contact = self.player.on_ground && probe.cell.is_solid_floor();
            if center_slope_snap_candidate
                || (allow_hitbox_snap && (probe.on_ground || preserving_ground_contact))
            {
                let (floor_y_px, contact_y_px) = match probe.cell.collision {
                    terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes => {
                        (probe.floor_y_px.saturating_sub(1), probe.floor_y_px)
                    }
                    terrain::CollisionKind::SlopeDown
                    | terrain::CollisionKind::SlopeUp
                    | terrain::CollisionKind::HiddenBlock
                    | terrain::CollisionKind::Empty => (probe.floor_y_px, probe.floor_y_px),
                };
                let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
                let contact_y_fp = (contact_y_px.max(0) as u32) << 16;
                self.terrain.floor_y_pos = floor_y_fp;
                let within_snap =
                    self.player.y_pos.abs_diff(floor_y_fp) <= GROUND_SNAP_TOLERANCE_FP;
                let can_land = match probe.cell.collision {
                    terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp => {
                        let within_slope_touch_snap =
                            self.player.y_pos.abs_diff(floor_y_fp) <= 0x0001_0000;
                        (self.player.y_velocity >= 0
                            && (self.player.y_pos >= floor_y_fp || within_slope_touch_snap))
                            || (allow_upward_slope_snap
                                && center_slope_snap_candidate
                                && self.player.y_velocity < 0
                                && self.player.y_velocity >= -(SLOPE_AIR_SNAP_TOLERANCE_FP as i32)
                                && (self.player.y_pos >= floor_y_fp || within_snap))
                    }
                    terrain::CollisionKind::Solid
                    | terrain::CollisionKind::Spikes
                    | terrain::CollisionKind::HiddenBlock
                    | terrain::CollisionKind::Empty => {
                        // ORIGINAL_QUIRK: ASM sub_902A only lands after the
                        // falling step reaches the solid contact row. The
                        // AsmFaithful path must not collapse a remaining
                        // sub-half-pixel gap into an early landing.
                        let within_solid_touch_snap = allow_solid_touch_snap
                            && !was_on_ground
                            && self.player.y_pos.abs_diff(floor_y_fp) < 0x0000_8000;
                        self.player.y_velocity >= 0
                            && (self.player.y_pos >= contact_y_fp
                                || within_solid_touch_snap
                                || (was_on_ground && within_snap))
                    }
                };
                if can_land {
                    let rubber_rebound = !was_on_ground
                        && probe.cell.has_block
                        && probe.cell.block_type == terrain::BlockType::Rubber;
                    let should_apply_slope_recontact = !was_on_ground
                        && self.player.y_velocity < 0
                        && matches!(
                            probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        );
                    if !was_on_ground
                        && horizontal_step_applied
                        && !rubber_rebound
                        && !matches!(
                            probe.cell.collision,
                            terrain::CollisionKind::SlopeDown | terrain::CollisionKind::SlopeUp
                        )
                    {
                        let landing_x =
                            i64::from(self.player.x_pos) - i64::from(self.player.x_velocity);
                        self.player.x_pos = landing_x.clamp(0, i64::from(u32::MAX)) as u32;
                    }
                    // ASM sub_902A floor collision (kid.asm:10461-10462):
                    // When falling through a floor tile, sub_902A snaps y_pos
                    // and does `clr.w $20(a3)` (clears sub-pixel). But when
                    // already on the ground (y_vel=0, no tile crossing), sub_902A
                    // exits early via loc_913A without touching y_pos. So we
                    // only clear sub-pixel on actual landings (airborne→grounded).
                    if !was_on_ground {
                        // New landing: clear sub-pixel (sub_902A clr.w $20(a3))
                        self.player.y_pos = floor_y_fp & 0xFFFF_0000;
                    } else {
                        // Already grounded: preserve sub-pixel (sub_902A exits early)
                        self.player.y_pos =
                            (floor_y_fp & 0xFFFF_0000) | (self.player.y_pos & 0x0000_FFFF);
                    }
                    let incoming_y_velocity = self.player.y_velocity;
                    if rubber_rebound {
                        // ASM sub_902A -> sub_914A -> loc_91F6 -> loc_9082:
                        // a fresh downward contact on rubber snaps to the
                        // contact row, clears the sub-pixel, then restores the
                        // negated incoming y_vel instead of settling grounded.
                        // The rebound branch keeps the frame's horizontal carry;
                        // unlike the normal landing path, loc_9082 does not
                        // rewind x_pos after the contact-row snap.
                        self.player.on_ground = false;
                        self.player.y_velocity = -incoming_y_velocity;
                        self.player.on_spikes = false;
                        self.terrain.floor_terrain_type =
                            grounded_surface_type_sub_8600(grid, x_px, y_px, collision_hw);
                        self.rubber_block_hit = Some((probe.cell_x, probe.cell_y));
                        self.player.wall_cling_direction = None;
                        return None;
                    }
                    // ASM off_933E: Mushroom (index 9) is a NOP — no bounce.
                    // Non-rubber block types land normally here.
                    self.player.on_ground = true;
                    self.player.y_velocity = 0;
                    self.player.wall_cling_direction = None;
                    if !was_on_ground {
                        // ASM loc_9F18 consumes only the single-frame
                        // `Ctrl_Pressed` edge. A generic buffered jump request
                        // must not survive a fresh floor landing into the next
                        // grounded frame, or native auto-jumps one frame early
                        // on the Level 1 death-respawn route.
                        self.player.jump_buffer_frames = 0;
                    }
                    // Teleporter blocks warp player on contact
                    if probe.cell.has_block
                        && probe.cell.block_type == terrain::BlockType::Teleporter
                    {
                        let tp_col = (x_px / 16).max(0) as u16;
                        let tp_row = (probe.floor_y_px / 16).max(0) as u16;
                        self.teleporter_contact = Some((tp_col, tp_row));
                    }
                    self.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
                    self.terrain.floor_terrain_type =
                        grounded_surface_type_sub_8600(grid, x_px, y_px, collision_hw);
                    if should_apply_slope_recontact {
                        return match probe.cell.collision {
                            terrain::CollisionKind::SlopeDown => {
                                Some(terrain::CollisionKind::SlopeDown)
                            }
                            terrain::CollisionKind::SlopeUp => {
                                Some(terrain::CollisionKind::SlopeUp)
                            }
                            _ => None,
                        };
                    }
                }
            } else if self.player.on_ground {
                // Was on ground but no longer — start falling
                self.player.on_ground = false;
                self.player.on_spikes = false;
                self.terrain.floor_terrain_type = TerrainType::Normal;
            }
        } else if self.player.on_ground {
            self.player.on_ground = false;
            self.player.on_spikes = false;
            self.terrain.floor_terrain_type = TerrainType::Normal;
        }
        None
    }

    pub(crate) fn apply_immediate_slope_recontact(
        &mut self,
        input: FrameInput,
        form: PlayerForm,
        collision: terrain::CollisionKind,
        grid: &terrain::BlockGrid,
        collision_hw: i32,
    ) {
        let h_input = HorizontalInput {
            left: input.left,
            right: input.right,
            run: input.a,
        };
        let (new_vel, new_facing) = update_level_one_slope_ground_velocity(
            self.player.x_velocity,
            self.player.facing,
            h_input,
            collision,
            form,
            self.terrain.floor_terrain_type,
        );
        self.player.x_velocity = new_vel;
        self.player.facing = new_facing;
        let next_x = i64::from(self.player.x_pos) + i64::from(self.player.x_velocity);
        self.player.x_pos = next_x.clamp(0, i64::from(u32::MAX)) as u32;

        let x_px = (self.player.x_pos >> 16) as i32;
        let y_px = (self.player.y_pos >> 16) as i32;
        let probe = terrain::probe_floor_hitbox(grid, x_px, y_px, collision_hw);
        let floor_y_px = match probe.cell.collision {
            terrain::CollisionKind::Solid
            | terrain::CollisionKind::HiddenBlock
            | terrain::CollisionKind::Spikes => probe.floor_y_px.saturating_sub(1),
            terrain::CollisionKind::SlopeDown
            | terrain::CollisionKind::SlopeUp
            | terrain::CollisionKind::Empty => probe.floor_y_px,
        };
        let floor_y_fp = (floor_y_px.max(0) as u32) << 16;
        self.terrain.floor_y_pos = floor_y_fp;
        self.player.y_pos = floor_y_fp;
        self.player.on_ground = true;
        self.player.y_velocity = 0;
        self.player.wall_cling_direction = None;
        self.player.on_spikes = probe.cell.collision == terrain::CollisionKind::Spikes;
    }

    /// Camera follows player using the real sub_71E4 follow algorithm.
    /// `y_look_offset` is the down-look accumulator ($FFFFFB58): when >= 30,
    /// shifts the camera target downward by that many pixels.
    pub(crate) fn update_camera_follow(&mut self, y_look_offset: i16) {
        let player_x_px = (self.player.x_pos >> 16) as i32;
        let mut player_y_px = (self.player.y_pos >> 16) as i32;
        // ASM sub_71E4 loc_728C: when offset >= $1E, add it to camera Y target
        // and subtract the reference $1E. This effectively shifts the view down.
        if y_look_offset >= 0x1E {
            player_y_px += i32::from(y_look_offset);
        }
        let camera_x_px = (self.camera.x_pos >> 16) as i32;
        let camera_y_px = (self.camera.y_pos >> 16) as i32;

        let (new_cx, new_cy) = physics::update_camera(
            camera_x_px,
            camera_y_px,
            player_x_px,
            player_y_px,
            &self.level_bounds,
        );

        self.camera.x_pos = (new_cx.max(0) as u32) << 16;
        self.camera.y_pos = (new_cy.max(0) as u32) << 16;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DownHoldAlignmentSub7A10 {
    pub(crate) x_pos: u32,
    pub(crate) blocked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct CrawlWallEscapeSub81B4 {
    pub(crate) x_pos: u32,
    pub(crate) blocked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct CrawlSlopeSnapSub83E6 {
    pub(crate) x_pos: u32,
    pub(crate) adjusted: bool,
}

pub(crate) fn update_level_one_slope_ground_velocity(
    x_vel: i32,
    facing: Direction,
    input: HorizontalInput,
    collision: terrain::CollisionKind,
    form: PlayerForm,
    terrain_type: TerrainType,
) -> (i32, Direction) {
    let _ = terrain_type;
    let _ = facing;
    let slope_descends_right = collision == terrain::CollisionKind::SlopeDown;
    // ASM sub_9386 stores `$FFFFFA26=0` for `$4000` SlopeUp and `$FF` for
    // `$5000` SlopeDown. loc_9D94 only negates/swallows inputs when `$FA26`
    // is set, so the canonical slope frame keeps live `x_vel` for SlopeUp and
    // flips it for SlopeDown.
    let mut canonical_vel = if slope_descends_right { -x_vel } else { x_vel };

    // ASM loc_9D22 uses three slope-velocity families: the shared `unk_9EB0`
    // table for Kid/Skycutter/Cyclone/Juggernaut/IronKnight/Berzerker, the
    // Micromax-specific `unk_9EE4` table, and the simplified loc_9E6C branch
    // for Eyeclops/Red Stealth/Maniaxe.
    if matches!(
        form,
        PlayerForm::RedStealth | PlayerForm::Eyeclops | PlayerForm::Maniaxe
    ) {
        let next_x_vel = if slope_descends_right {
            let advanced = x_vel + SPECIAL_FORM_SLOPE_STEP_FP;
            if advanced >= 0 {
                SPECIAL_FORM_SLOPE_STEP_FP
            } else {
                advanced
            }
        } else {
            let advanced = x_vel - SPECIAL_FORM_SLOPE_STEP_FP;
            if advanced < 0 {
                -SPECIAL_FORM_SLOPE_STEP_FP
            } else {
                advanced
            }
        };
        let next_facing = if next_x_vel >= 0 {
            Direction::Right
        } else {
            Direction::Left
        };
        return (next_x_vel, next_facing);
    }

    let (
        max_speed,
        brake,
        neutral_decel,
        neutral_target,
        uphill_cap,
        uphill_accel,
        downhill_cap,
        downhill_accel,
    ) = if input.run {
        match form {
            PlayerForm::Micromax => (
                MICROMAX_SLOPE_MAX_SPEED_FP,
                MICROMAX_SLOPE_BRAKE_FP,
                MICROMAX_SLOPE_NEUTRAL_DECEL_FP,
                MICROMAX_SLOPE_NEUTRAL_TARGET_FP,
                MICROMAX_SLOPE_RUN_UPHILL_CAP_FP,
                MICROMAX_SLOPE_RUN_UPHILL_ACCEL_FP,
                MICROMAX_SLOPE_RUN_DOWNHILL_CAP_FP,
                MICROMAX_SLOPE_RUN_DOWNHILL_ACCEL_FP,
            ),
            _ => (
                KID_SLOPE_MAX_SPEED_FP,
                KID_SLOPE_BRAKE_FP,
                KID_SLOPE_NEUTRAL_DECEL_FP,
                KID_SLOPE_NEUTRAL_TARGET_FP,
                KID_SLOPE_RUN_UPHILL_CAP_FP,
                KID_SLOPE_RUN_UPHILL_ACCEL_FP,
                KID_SLOPE_RUN_DOWNHILL_CAP_FP,
                KID_SLOPE_RUN_DOWNHILL_ACCEL_FP,
            ),
        }
    } else {
        match form {
            PlayerForm::Micromax => (
                MICROMAX_SLOPE_MAX_SPEED_FP,
                MICROMAX_SLOPE_BRAKE_FP,
                MICROMAX_SLOPE_NEUTRAL_DECEL_FP,
                MICROMAX_SLOPE_NEUTRAL_TARGET_FP,
                MICROMAX_SLOPE_WALK_UPHILL_CAP_FP,
                MICROMAX_SLOPE_WALK_UPHILL_ACCEL_FP,
                MICROMAX_SLOPE_WALK_DOWNHILL_CAP_FP,
                MICROMAX_SLOPE_WALK_DOWNHILL_ACCEL_FP,
            ),
            _ => (
                KID_SLOPE_MAX_SPEED_FP,
                KID_SLOPE_BRAKE_FP,
                KID_SLOPE_NEUTRAL_DECEL_FP,
                KID_SLOPE_NEUTRAL_TARGET_FP,
                KID_SLOPE_WALK_UPHILL_CAP_FP,
                KID_SLOPE_WALK_UPHILL_ACCEL_FP,
                KID_SLOPE_WALK_DOWNHILL_CAP_FP,
                KID_SLOPE_WALK_DOWNHILL_ACCEL_FP,
            ),
        }
    };

    let uphill_pressed = if slope_descends_right {
        input.left
    } else {
        input.right
    };
    let downhill_pressed = if slope_descends_right {
        input.right
    } else {
        input.left
    };

    if uphill_pressed {
        if canonical_vel < 0 {
            canonical_vel += uphill_accel * 2;
        } else if canonical_vel <= uphill_cap {
            canonical_vel += uphill_accel;
            if canonical_vel > uphill_cap {
                canonical_vel = uphill_cap;
            }
        } else {
            canonical_vel -= brake;
            if canonical_vel > max_speed {
                canonical_vel = max_speed;
            }
        }
    } else if downhill_pressed {
        if canonical_vel >= 0 {
            canonical_vel -= downhill_accel * 2;
        } else if canonical_vel >= downhill_cap {
            canonical_vel -= downhill_accel;
            if canonical_vel < downhill_cap {
                canonical_vel = downhill_cap;
            }
        } else {
            canonical_vel += brake;
            if canonical_vel < -max_speed {
                canonical_vel = -max_speed;
            }
        }
    } else if canonical_vel < neutral_target {
        canonical_vel += neutral_decel;
        if canonical_vel > neutral_target {
            canonical_vel = neutral_target;
        }
    } else {
        canonical_vel -= neutral_decel;
        if canonical_vel < neutral_target {
            canonical_vel = neutral_target;
        }
    }

    let next_facing = if uphill_pressed {
        if slope_descends_right {
            Direction::Right
        } else {
            Direction::Left
        }
    } else if downhill_pressed {
        if slope_descends_right {
            Direction::Left
        } else {
            Direction::Right
        }
    } else if slope_descends_right == (canonical_vel >= 0) {
        Direction::Right
    } else {
        Direction::Left
    };

    let next_x_vel = if slope_descends_right {
        -canonical_vel
    } else {
        canonical_vel
    };
    (next_x_vel, next_facing)
}

#[allow(non_snake_case)]
pub(crate) fn align_player_for_down_hold_sub_7A10(
    grid: &terrain::BlockGrid,
    x_pos: u32,
    y_pos: u32,
    level_width_pixels: u16,
) -> DownHoldAlignmentSub7A10 {
    let original_x_px = (x_pos >> 16) as i32;
    let foot_y_px = (y_pos >> 16) as i32;
    let mut aligned_x_px = original_x_px;
    let half_width = 14i32;
    let level_width_px = i32::from(level_width_pixels);
    let occupied = |center_x_px: i32, x_offset_px: i32| {
        !matches!(
            terrain::probe_relative_collision_sub_B55C(
                grid,
                center_x_px,
                foot_y_px,
                x_offset_px,
                0,
            ),
            terrain::CollisionKind::Empty
        )
    };

    if original_x_px - half_width < 0 || occupied(original_x_px, -half_width) {
        aligned_x_px = ((original_x_px - half_width) & !0xF) + 0x10 + half_width;
        if aligned_x_px + half_width >= level_width_px || occupied(aligned_x_px, half_width) {
            return DownHoldAlignmentSub7A10 {
                x_pos: (original_x_px.max(0) as u32) << 16,
                blocked: true,
            };
        }
    } else if original_x_px + half_width >= level_width_px || occupied(original_x_px, half_width) {
        let adjust = ((original_x_px + half_width) & 0xF) + 1;
        aligned_x_px -= adjust;
        if aligned_x_px - half_width < 0 || occupied(aligned_x_px, -half_width) {
            return DownHoldAlignmentSub7A10 {
                x_pos: (original_x_px.max(0) as u32) << 16,
                blocked: true,
            };
        }
    }

    DownHoldAlignmentSub7A10 {
        x_pos: (aligned_x_px.max(0) as u32) << 16,
        blocked: false,
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn grounded_surface_type_sub_8600(
    grid: &terrain::BlockGrid,
    player_x_px: i32,
    player_y_px: i32,
    half_width_px: i32,
) -> TerrainType {
    // ASM sub_8600 (kid.asm:9445): scan the foot row under Kid's full width,
    // count solid/spike support cells, then promote ice only on a strict
    // majority while rubber wins ties.
    let min_x_px = (player_x_px - half_width_px).max(0);
    let max_x_px = (player_x_px + half_width_px).clamp(0, grid.width_pixels().saturating_sub(1));
    let start_col = (min_x_px / 16).max(0) as u16;
    let end_col = (max_x_px / 16).max(0) as u16;
    let row = ((player_y_px + 1).max(0) / 16) as u16;

    let mut solid_count = 0u16;
    let mut ice_count = 0u16;
    let mut rubber_count = 0u16;

    for col in start_col..=end_col {
        let cell = grid.cell_at(col, row);
        if matches!(
            cell.collision,
            terrain::CollisionKind::Solid | terrain::CollisionKind::Spikes
        ) {
            solid_count = solid_count.saturating_add(1);
        }
        if cell.has_block {
            match cell.block_type {
                terrain::BlockType::Ice => ice_count = ice_count.saturating_add(1),
                terrain::BlockType::Rubber => rubber_count = rubber_count.saturating_add(1),
                _ => {}
            }
        }
    }

    let mut terrain_type = TerrainType::Normal;
    if ice_count.saturating_mul(2) > solid_count {
        terrain_type = TerrainType::Ice;
    }
    if solid_count != 0 && rubber_count.saturating_mul(2) >= solid_count {
        terrain_type = TerrainType::Rubber;
    }
    terrain_type
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn crawl_wall_escape_sub_81B4(
    grid: &terrain::BlockGrid,
    x_pos: u32,
    y_pos: u32,
    half_width: i32,
) -> CrawlWallEscapeSub81B4 {
    // ASM sub_81B4 (kid.asm:8952): probe one row 16px above the feet at
    // `x_pos ± half_width`, take the first non-solid side, and snap the player
    // flush to that tile boundary. Only return blocked when both side probes
    // are solid (`$6000`).
    let center_x_px = (x_pos >> 16) as i32;
    let foot_y_px = (y_pos >> 16) as i32;
    let sample_y_offset_px = -16;
    let original_x_pos = (center_x_px.max(0) as u32) << 16;

    if terrain::probe_relative_collision_sub_B55C(
        grid,
        center_x_px,
        foot_y_px,
        -half_width,
        sample_y_offset_px,
    ) != terrain::CollisionKind::Solid
    {
        let snapped_x_px = ((center_x_px - half_width) & !0xF) + 0xF - half_width;
        return CrawlWallEscapeSub81B4 {
            x_pos: (snapped_x_px.max(0) as u32) << 16,
            blocked: false,
        };
    }

    if terrain::probe_relative_collision_sub_B55C(
        grid,
        center_x_px,
        foot_y_px,
        half_width,
        sample_y_offset_px,
    ) != terrain::CollisionKind::Solid
    {
        let snapped_x_px = ((center_x_px + half_width) & !0xF) + half_width;
        return CrawlWallEscapeSub81B4 {
            x_pos: (snapped_x_px.max(0) as u32) << 16,
            blocked: false,
        };
    }

    CrawlWallEscapeSub81B4 {
        x_pos: original_x_pos,
        blocked: true,
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn crawl_slope_snap_sub_83E6(
    grid: &terrain::BlockGrid,
    x_pos: u32,
    y_pos: u32,
    x_velocity: i32,
    half_width: i32,
) -> CrawlSlopeSnapSub83E6 {
    // ASM sub_83E6 (kid.asm:9163): when crawling without a direct wall hit,
    // inspect the motion-side foot cell. Rightward or neutral motion accepts
    // only `$4000` slope-up and nudges left by the low nibble + 1; leftward
    // motion accepts only `$5000` slope-down and snaps to the current tile
    // column plus `half_width`. The ROM also clears `$1C(a3)` here, but that
    // y-size scratch word is not modelled in native state.
    let center_x_px = (x_pos >> 16) as i32;
    let foot_y_px = (y_pos >> 16) as i32;
    let original_x_pos = (center_x_px.max(0) as u32) << 16;

    if x_velocity >= 0 {
        if terrain::probe_relative_collision_sub_B55C(grid, center_x_px, foot_y_px, half_width, 0)
            == terrain::CollisionKind::SlopeUp
        {
            let adjust_px = ((center_x_px + half_width) & 0xF) + 1;
            let snapped_x_px = center_x_px - adjust_px;
            return CrawlSlopeSnapSub83E6 {
                x_pos: (snapped_x_px.max(0) as u32) << 16,
                adjusted: true,
            };
        }
    } else if terrain::probe_relative_collision_sub_B55C(
        grid,
        center_x_px,
        foot_y_px,
        -half_width,
        0,
    ) == terrain::CollisionKind::SlopeDown
    {
        let snapped_x_px = (center_x_px & !0xF) + half_width;
        return CrawlSlopeSnapSub83E6 {
            x_pos: (snapped_x_px.max(0) as u32) << 16,
            adjusted: true,
        };
    }

    CrawlSlopeSnapSub83E6 {
        x_pos: original_x_pos,
        adjusted: false,
    }
}
