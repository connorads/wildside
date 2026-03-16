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
        let glyph_index = ch as usize;
        let glyph = if glyph_index < 128 {
            font8x8::BASIC_FONTS.get(ch).unwrap_or([0u8; 8])
        } else {
            [0u8; 8]
        };
        for (row, &bits) in glyph.iter().enumerate() {
            for col in 0..8 {
                if bits & (1 << col) != 0 {
                    let x = px + ci * 8 + col;
                    let y = py + row;
                    let idx = (y * stride_w + x) * 3;
                    if idx + 2 < rgb.len() {
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
    draw_text(
        rgb,
        w,
        panel_w * 2 + 2,
        2,
        &format!("DIFF x4 MAE={mae:.1}"),
    );
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
        assert!(rgb.iter().any(|&b| b == 255));
    }
}
