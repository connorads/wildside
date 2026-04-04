#[allow(dead_code)]
pub(crate) fn mark_background_corner_neighbours_sub_12752(
    level_layout: &mut [u8],
    width_blocks: u16,
    height_blocks: u16,
    col: u16,
    row: u16,
    tile_class: u8,
) {
    // ASM sub_12752 (kid.asm:25037): tile class $40 marks the cell to the
    // right plus the cell below; tile class $50 marks the cell to the left
    // plus the cell below.
    match tile_class {
        0x40 => {
            mark_background_neighbour_sub_12792(
                level_layout,
                width_blocks,
                height_blocks,
                col + 1,
                row,
            );
            mark_background_neighbour_sub_12792(
                level_layout,
                width_blocks,
                height_blocks,
                col,
                row + 1,
            );
        }
        0x50 => {
            if let Some(left_col) = col.checked_sub(1) {
                mark_background_neighbour_sub_12792(
                    level_layout,
                    width_blocks,
                    height_blocks,
                    left_col,
                    row,
                );
            }
            mark_background_neighbour_sub_12792(
                level_layout,
                width_blocks,
                height_blocks,
                col,
                row + 1,
            );
        }
        _ => {}
    }
}

fn mark_background_neighbour_sub_12792(
    level_layout: &mut [u8],
    width_blocks: u16,
    height_blocks: u16,
    col: u16,
    row: u16,
) {
    if col >= width_blocks || row >= height_blocks {
        return;
    }
    let cell_offset = (usize::from(row) * usize::from(width_blocks) + usize::from(col)) * 2;
    let Some(hi_byte) = level_layout.get_mut(cell_offset) else {
        return;
    };
    if *hi_byte & 0x80 != 0 {
        return;
    }
    *hi_byte |= 0x01;
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackgroundLayoutRecord {
    pub(crate) layout_word_offset: u16,
    pub(crate) col: u16,
    pub(crate) row: u16,
    pub(crate) tile_kind: u8,
    pub(crate) tile_variant: u8,
}

impl BackgroundLayoutRecord {
    #[allow(dead_code)]
    pub(crate) const fn new(
        layout_word_offset: u16,
        col: u16,
        row: u16,
        tile_kind: u8,
        tile_variant: u8,
    ) -> Self {
        Self {
            layout_word_offset,
            col,
            row,
            tile_kind,
            tile_variant,
        }
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn prepare_background_layout_cell_sub_127A8(
    level_layout: &mut [u8],
    terrain_layout: &[u8],
    layout_word_offset: u16,
) {
    // ASM sub_127A8 (kid.asm:25092): write $20 to the level-layout high byte,
    // then copy the terrain byte from the matching halfword index.
    let layout_word_offset = usize::from(layout_word_offset);
    let Some(cell) = level_layout.get_mut(layout_word_offset..layout_word_offset + 2) else {
        return;
    };
    let Some(&terrain_byte) = terrain_layout.get(layout_word_offset / 2) else {
        return;
    };
    cell[0] = 0x20;
    cell[1] = terrain_byte;
}

#[allow(non_snake_case)]
#[allow(dead_code)]
pub(crate) fn insert_background_layout_record_sub_127CE(
    records: &mut Vec<BackgroundLayoutRecord>,
    record: BackgroundLayoutRecord,
) {
    // ASM sub_127CE (kid.asm:25110): scan for the first existing 8-byte record
    // whose layout pointer is greater than the new one, shift the tail upward,
    // and insert the new record after all lower-or-equal offsets.
    let insertion_index = records
        .partition_point(|existing| existing.layout_word_offset <= record.layout_word_offset);
    records.insert(insertion_index, record);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell_hi(layout: &[u8], width_blocks: u16, col: u16, row: u16) -> u8 {
        let offset = (usize::from(row) * usize::from(width_blocks) + usize::from(col)) * 2;
        layout[offset]
    }

    #[test]
    fn sub_12752_tile_class_40_marks_right_and_below_cells() {
        let mut layout = vec![0u8; 4 * 4 * 2];

        mark_background_corner_neighbours_sub_12752(&mut layout, 4, 4, 1, 1, 0x40);

        assert_eq!(
            cell_hi(&layout, 4, 2, 1),
            0x01,
            "ASM sub_12752/sub_12792 should set bit 0 on the right neighbour for class $40"
        );
        assert_eq!(
            cell_hi(&layout, 4, 1, 2),
            0x01,
            "ASM sub_12752/sub_12792 should set bit 0 on the below neighbour for class $40"
        );
    }

    #[test]
    fn sub_12752_tile_class_50_marks_left_and_below_cells() {
        let mut layout = vec![0u8; 4 * 4 * 2];

        mark_background_corner_neighbours_sub_12752(&mut layout, 4, 4, 2, 1, 0x50);

        assert_eq!(
            cell_hi(&layout, 4, 1, 1),
            0x01,
            "ASM sub_12752/sub_12792 should set bit 0 on the left neighbour for class $50"
        );
        assert_eq!(
            cell_hi(&layout, 4, 2, 2),
            0x01,
            "ASM sub_12752/sub_12792 should set bit 0 on the below neighbour for class $50"
        );
    }

    #[test]
    fn sub_12752_respects_blocked_and_out_of_bounds_neighbours() {
        let mut layout = vec![0u8; 4 * 4 * 2];
        let blocked_offset = (usize::from(1u16) * usize::from(4u16) + usize::from(2u16)) * 2;
        layout[blocked_offset] = 0x80;

        mark_background_corner_neighbours_sub_12752(&mut layout, 4, 4, 1, 1, 0x40);
        mark_background_corner_neighbours_sub_12752(&mut layout, 4, 4, 0, 3, 0x50);

        assert_eq!(
            cell_hi(&layout, 4, 2, 1),
            0x80,
            "ASM sub_12792 must not touch neighbours whose high bit is already set"
        );
        assert_eq!(
            cell_hi(&layout, 4, 0, 3),
            0x00,
            "left/out-of-bounds cases should not wrap or mutate the source cell"
        );
        assert_eq!(
            cell_hi(&layout, 4, 0, 2),
            0x00,
            "class $50 at column 0 must skip the missing left neighbour"
        );
    }

    #[test]
    fn sub_127a8_marks_layout_cell_and_copies_terrain_byte() {
        let mut layout = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let terrain_layout = [0x90, 0x91, 0x92];

        prepare_background_layout_cell_sub_127A8(&mut layout, &terrain_layout, 2);

        assert_eq!(
            layout,
            vec![0x11, 0x22, 0x20, 0x91, 0x55, 0x66],
            "ASM sub_127A8 should write $20 to the high byte and copy the terrain byte from the matching cell index"
        );
    }

    #[test]
    fn sub_127ce_inserts_record_after_existing_lower_or_equal_offsets() {
        let mut records = vec![
            BackgroundLayoutRecord::new(0x0010, 1, 1, 0x80, 0x00),
            BackgroundLayoutRecord::new(0x0020, 2, 2, 0x81, 0x01),
            BackgroundLayoutRecord::new(0x0020, 3, 3, 0x82, 0x02),
            BackgroundLayoutRecord::new(0x0030, 4, 4, 0x83, 0x03),
        ];
        let inserted = BackgroundLayoutRecord::new(0x0020, 9, 9, 0xAA, 0xBB);

        insert_background_layout_record_sub_127CE(&mut records, inserted);

        assert_eq!(
            records,
            vec![
                BackgroundLayoutRecord::new(0x0010, 1, 1, 0x80, 0x00),
                BackgroundLayoutRecord::new(0x0020, 2, 2, 0x81, 0x01),
                BackgroundLayoutRecord::new(0x0020, 3, 3, 0x82, 0x02),
                BackgroundLayoutRecord::new(0x0020, 9, 9, 0xAA, 0xBB),
                BackgroundLayoutRecord::new(0x0030, 4, 4, 0x83, 0x03),
            ],
            "ASM sub_127CE should shift the first greater 8-byte record upward, leaving equal offsets ahead of the inserted slot"
        );
    }
}
