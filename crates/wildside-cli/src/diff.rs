/// Side-by-side diff image generation for native vs oracle frame comparison.
///
/// Produces a 3-panel composite: [native | oracle | amplified_diff]
/// with text annotations showing MAE, level ID, and checkpoint label.
use font8x8::UnicodeFonts;

/// Mean absolute error between two RGB byte buffers.
pub(crate) fn frame_mae(native: &[u8], oracle: &[u8]) -> f32 {
    if native.len() != oracle.len() || native.is_empty() {
        return f32::MAX;
    }
    let sum: u64 = native
        .iter()
        .zip(oracle.iter())
        .map(|(n, o)| u64::from(n.abs_diff(*o)))
        .sum();
    sum as f32 / native.len() as f32
}

/// Count the number of pixels that differ between two RGB buffers.
pub(crate) fn pixel_mismatch_count(a: &[u8], b: &[u8]) -> usize {
    a.chunks_exact(3)
        .zip(b.chunks_exact(3))
        .filter(|(pa, pb)| pa != pb)
        .count()
}

/// Generate a 3-panel composite RGB image: [native | oracle | amplified_diff].
///
/// `w` and `h` are the dimensions of each individual frame.
/// Returns the composite as (width, height, rgb_bytes).
pub(crate) fn generate_diff_composite(
    native: &[u8],
    oracle: &[u8],
    w: u32,
    h: u32,
) -> (u32, u32, Vec<u8>) {
    let pw = w as usize;
    let ph = h as usize;
    let frame_stride = pw * 3;
    let composite_w = pw * 3;
    let composite_h = ph;
    let mut rgb = vec![0u8; composite_w * composite_h * 3];

    for y in 0..ph {
        let src_row_start = y * frame_stride;
        let dst_row_start = y * composite_w * 3;

        // Panel 1: native
        let native_row = &native[src_row_start..src_row_start + frame_stride];
        rgb[dst_row_start..dst_row_start + frame_stride].copy_from_slice(native_row);

        // Panel 2: oracle
        let oracle_row = &oracle[src_row_start..src_row_start + frame_stride];
        let panel2_start = dst_row_start + frame_stride;
        rgb[panel2_start..panel2_start + frame_stride].copy_from_slice(oracle_row);

        // Panel 3: amplified diff (abs delta * 4, clamped)
        let panel3_start = dst_row_start + frame_stride * 2;
        for x in 0..frame_stride {
            let delta = native_row[x].abs_diff(oracle_row[x]);
            rgb[panel3_start + x] = (u16::from(delta) * 4).min(255) as u8;
        }
    }

    (composite_w as u32, composite_h as u32, rgb)
}

/// Render an 8x8 font character at (px, py) into an RGB buffer.
/// Uses font8x8 basic glyphs. Out-of-bounds pixels are silently skipped.
pub(crate) fn draw_text(rgb: &mut [u8], stride_w: usize, px: usize, py: usize, text: &str) {
    for (ci, ch) in text.chars().enumerate() {
        let glyph = font8x8::BASIC_FONTS.get(ch).unwrap_or([0u8; 8]);
        for (row, &bits) in glyph.iter().enumerate() {
            for col in 0..8 {
                if bits & (1 << col) != 0 {
                    let x = px + ci * 8 + col;
                    let y = py + row;
                    let idx = (y * stride_w + x) * 3;
                    if x < stride_w && idx + 2 < rgb.len() {
                        rgb[idx] = 255;
                        rgb[idx + 1] = 255;
                        rgb[idx + 2] = 255;
                    }
                }
            }
        }
    }
}

/// Annotate a diff composite with level info and MAE value.
pub(crate) fn annotate_diff(
    rgb: &mut [u8],
    composite_w: u32,
    level_id: u8,
    checkpoint_label: &str,
    mae: f32,
) {
    let w = composite_w as usize;
    let panel_w = w / 3;
    // Labels at top of each panel (y=2)
    draw_text(rgb, w, 2, 2, "NATIVE");
    draw_text(rgb, w, panel_w + 2, 2, "ORACLE");
    draw_text(rgb, w, panel_w * 2 + 2, 2, &format!("DIFF x4 MAE={mae:.1}"));
    // Level + checkpoint at bottom-left
    draw_text(
        rgb,
        w,
        2,
        12,
        &format!("L{level_id:02X} {checkpoint_label}"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit_pixels(rgb: &[u8], stride_w: usize) -> Vec<(usize, usize)> {
        rgb.chunks_exact(3)
            .enumerate()
            .filter_map(|(i, pixel)| {
                (pixel == [255, 255, 255]).then_some((i % stride_w, i / stride_w))
            })
            .collect()
    }

    fn has_lit_pixel_in(
        rgb: &[u8],
        stride_w: usize,
        x_range: std::ops::Range<usize>,
        y_range: std::ops::Range<usize>,
    ) -> bool {
        lit_pixels(rgb, stride_w)
            .iter()
            .any(|(x, y)| x_range.contains(x) && y_range.contains(y))
    }

    #[test]
    fn pixel_mismatch_identical_is_zero() {
        let frame = vec![128u8; 320 * 224 * 3];
        assert_eq!(pixel_mismatch_count(&frame, &frame), 0);
    }

    #[test]
    fn pixel_mismatch_all_different() {
        let black = vec![0u8; 12]; // 4 pixels
        let white = vec![255u8; 12];
        assert_eq!(pixel_mismatch_count(&black, &white), 4);
    }

    #[test]
    fn pixel_mismatch_partial() {
        // 3 pixels: first matches, second differs in R, third differs in all
        let a = vec![10, 20, 30, 40, 50, 60, 70, 80, 90];
        let b = vec![10, 20, 30, 41, 50, 60, 0, 0, 0];
        assert_eq!(pixel_mismatch_count(&a, &b), 2);
    }

    #[test]
    fn mae_identical_frames_is_zero() {
        let frame = vec![128u8; 320 * 224 * 3];
        assert_eq!(frame_mae(&frame, &frame), 0.0);
    }

    #[test]
    fn mae_completely_different_frames() {
        let black = vec![0u8; 12];
        let white = vec![255u8; 12];
        assert_eq!(frame_mae(&black, &white), 255.0);
    }

    #[test]
    fn mae_empty_frames_returns_max() {
        assert_eq!(frame_mae(&[], &[]), f32::MAX);
    }

    #[test]
    fn mae_mismatched_lengths_returns_max() {
        assert_eq!(frame_mae(&[0, 1], &[0]), f32::MAX);
    }

    #[test]
    fn diff_composite_dimensions() {
        let w = 4;
        let h = 2;
        let native = vec![100u8; (w * h * 3) as usize];
        let oracle = vec![120u8; (w * h * 3) as usize];
        let (cw, ch, rgb) = generate_diff_composite(&native, &oracle, w, h);
        assert_eq!(cw, w * 3);
        assert_eq!(ch, h);
        assert_eq!(rgb.len(), (cw * ch * 3) as usize);
    }

    #[test]
    fn diff_composite_amplifies_delta() {
        let w = 1;
        let h = 1;
        // Single pixel: native=100, oracle=110, delta=10, amplified=40
        let native = vec![100u8; 3];
        let oracle = vec![110u8; 3];
        let (_, _, rgb) = generate_diff_composite(&native, &oracle, w, h);
        // Third panel starts at pixel offset 2 (panel_w=1, so offset = 2*3 = 6)
        assert_eq!(rgb[6], 40); // 10 * 4 = 40
    }

    #[test]
    fn diff_composite_clamps_amplified_delta() {
        let w = 1;
        let h = 1;
        // delta=100, amplified=400 → clamped to 255
        let native = vec![0u8; 3];
        let oracle = vec![100u8; 3];
        let (_, _, rgb) = generate_diff_composite(&native, &oracle, w, h);
        assert_eq!(rgb[6], 255);
    }

    #[test]
    fn diff_composite_preserves_panel_rows() {
        let native = vec![
            10, 11, 12, 13, 14, 15, //
            20, 21, 22, 23, 24, 25,
        ];
        let oracle = vec![
            20, 21, 22, 23, 24, 25, //
            40, 41, 42, 43, 44, 45,
        ];
        let (_, _, rgb) = generate_diff_composite(&native, &oracle, 2, 2);

        assert_eq!(
            &rgb[0..18],
            &[
                10, 11, 12, 13, 14, 15, 20, 21, 22, 23, 24, 25, 40, 40, 40, 40, 40, 40
            ]
        );
        assert_eq!(
            &rgb[18..36],
            &[
                20, 21, 22, 23, 24, 25, 40, 41, 42, 43, 44, 45, 80, 80, 80, 80, 80, 80
            ]
        );
    }

    #[test]
    fn draw_text_does_not_panic_on_empty_string() {
        let mut rgb = vec![0u8; 320 * 3];
        draw_text(&mut rgb, 320, 0, 0, "");
    }

    #[test]
    fn draw_text_renders_pixel_within_bounds() {
        let w = 320;
        let h = 16;
        let mut rgb = vec![0u8; w * h * 3];
        draw_text(&mut rgb, w, 0, 0, "A");
        // At least one pixel should be lit (white)
        assert!(rgb.contains(&255));
    }

    #[test]
    fn draw_text_renders_expected_a_glyph_coordinates() {
        let w = 12;
        let h = 10;
        let mut rgb = vec![0u8; w * h * 3];
        draw_text(&mut rgb, w, 2, 1, "A");

        assert_eq!(
            lit_pixels(&rgb, w),
            vec![
                (4, 1),
                (5, 1),
                (3, 2),
                (4, 2),
                (5, 2),
                (6, 2),
                (2, 3),
                (3, 3),
                (6, 3),
                (7, 3),
                (2, 4),
                (3, 4),
                (6, 4),
                (7, 4),
                (2, 5),
                (3, 5),
                (4, 5),
                (5, 5),
                (6, 5),
                (7, 5),
                (2, 6),
                (3, 6),
                (6, 6),
                (7, 6),
                (2, 7),
                (3, 7),
                (6, 7),
                (7, 7),
            ]
        );
    }

    #[test]
    fn draw_text_places_successive_characters_eight_pixels_apart() {
        let w = 20;
        let h = 8;
        let mut single = vec![0u8; w * h * 3];
        let mut double = vec![0u8; w * h * 3];
        draw_text(&mut single, w, 8, 0, "A");
        draw_text(&mut double, w, 0, 0, " A");

        assert_eq!(double, single);
    }

    #[test]
    fn draw_text_ignores_non_ascii_glyphs() {
        let mut rgb = vec![0u8; 12 * 8 * 3];
        draw_text(&mut rgb, 12, 0, 0, "\u{80}");
        assert!(!rgb.contains(&255));
    }

    #[test]
    fn draw_text_clips_without_wrapping_to_next_row() {
        let w = 4;
        let h = 8;
        let mut rgb = vec![0u8; w * h * 3];
        draw_text(&mut rgb, w, 2, 0, "A");

        assert_eq!(
            lit_pixels(&rgb, w),
            vec![
                (3, 1),
                (2, 2),
                (3, 2),
                (2, 3),
                (3, 3),
                (2, 4),
                (3, 4),
                (2, 5),
                (3, 5),
                (2, 6),
                (3, 6),
            ]
        );
    }

    #[test]
    fn draw_text_at_right_edge_draws_nothing() {
        let w = 4;
        let h = 8;
        let mut rgb = vec![0u8; w * h * 3];
        draw_text(&mut rgb, w, w, 0, "A");
        assert!(!rgb.contains(&255));
    }

    #[test]
    fn draw_text_skips_pixels_without_room_for_full_rgb_triplet() {
        let w = 8;
        let mut rgb = vec![0u8; w * 3 - 1];
        draw_text(&mut rgb, w, 4, 0, "A");

        assert_eq!(lit_pixels(&rgb, w), vec![(6, 0)]);
    }

    #[test]
    fn annotate_diff_writes_labels_into_each_panel() {
        let w = 300;
        let h = 24;
        let mut rgb = vec![0u8; w * h * 3];
        annotate_diff(&mut rgb, w as u32, 0x0A, "CP", 12.5);

        assert!(has_lit_pixel_in(&rgb, w, 2..24, 2..10));
        assert!(has_lit_pixel_in(&rgb, w, 104..150, 2..10));
        assert!(has_lit_pixel_in(&rgb, w, 202..270, 2..10));
        assert!(!has_lit_pixel_in(&rgb, w, 98..102, 2..10));
        assert!(!has_lit_pixel_in(&rgb, w, 198..202, 2..10));
        assert!(has_lit_pixel_in(&rgb, w, 2..48, 12..20));
    }
}
