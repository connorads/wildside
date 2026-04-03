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
}
