use super::*;

pub(super) fn render_scene(scene: &Scene, rgb: &mut [u8]) {
    clear_frame(rgb, scene.background);
    let mut high_priority_pixels = vec![false; (FRAME_WIDTH * FRAME_HEIGHT) as usize];
    for command in &scene.commands {
        match command {
            DrawCommand::Rect {
                x,
                y,
                width,
                height,
                colour,
            } => fill_rect(
                rgb,
                &mut high_priority_pixels,
                *x,
                *y,
                *width,
                *height,
                *colour,
            ),
            DrawCommand::Text {
                x,
                y,
                text,
                colour,
                scale,
            } => draw_text(
                rgb,
                &mut high_priority_pixels,
                *x,
                *y,
                text,
                *colour,
                *scale,
            ),
            DrawCommand::IndexedTile {
                x,
                y,
                tile_sheet,
                tile,
                palette,
            } => draw_indexed_tile(
                rgb,
                &mut high_priority_pixels,
                *x,
                *y,
                tile_sheet,
                *tile,
                palette,
            ),
            DrawCommand::Sprite {
                x,
                y,
                tile_sheet,
                palette,
                tiles,
            } => draw_sprite(
                rgb,
                &mut high_priority_pixels,
                *x,
                *y,
                tile_sheet,
                palette,
                tiles,
            ),
        }
    }
}

pub(super) fn clear_frame(rgb: &mut [u8], colour: [u8; 3]) {
    for chunk in rgb.chunks_exact_mut(3) {
        chunk.copy_from_slice(&colour);
    }
}

pub(super) fn push_panel(
    commands: &mut Vec<DrawCommand>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    fill: [u8; 3],
    border: [u8; 3],
) {
    commands.push(DrawCommand::Rect {
        x,
        y,
        width,
        height,
        colour: fill,
    });
    commands.push(DrawCommand::Rect {
        x,
        y,
        width,
        height: 2,
        colour: border,
    });
    commands.push(DrawCommand::Rect {
        x,
        y: y + height - 2,
        width,
        height: 2,
        colour: border,
    });
    commands.push(DrawCommand::Rect {
        x,
        y,
        width: 2,
        height,
        colour: border,
    });
    commands.push(DrawCommand::Rect {
        x: x + width - 2,
        y,
        width: 2,
        height,
        colour: border,
    });
}

pub(super) fn text(x: i32, y: i32, value: &str, colour: [u8; 3], scale: i32) -> DrawCommand {
    DrawCommand::Text {
        x,
        y,
        text: value.to_string(),
        colour,
        scale,
    }
}

pub(super) fn draw_text(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    text: &str,
    colour: [u8; 3],
    scale: i32,
) {
    let mut cursor_x = x;
    let mut cursor_y = y;
    for ch in text.chars() {
        if ch == '\n' {
            cursor_x = x;
            cursor_y += 10 * scale;
            continue;
        }
        draw_glyph(
            rgb,
            high_priority_pixels,
            cursor_x,
            cursor_y,
            ch,
            colour,
            scale,
        );
        cursor_x += 8 * scale;
    }
}

pub(super) fn draw_glyph(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    ch: char,
    colour: [u8; 3],
    scale: i32,
) {
    let Some(bitmap) = BASIC_FONTS.get(ch) else {
        return;
    };
    for (row, bits) in bitmap.iter().enumerate() {
        for column in 0..8 {
            if bits & (1 << column) == 0 {
                continue;
            }
            fill_rect(
                rgb,
                high_priority_pixels,
                x + (column * scale),
                y + ((row as i32) * scale),
                scale,
                scale,
                colour,
            );
        }
    }
}

pub(super) fn fill_rect(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    colour: [u8; 3],
) {
    for py in y.max(0)..(y + height).min(FRAME_HEIGHT as i32) {
        for px in x.max(0)..(x + width).min(FRAME_WIDTH as i32) {
            set_pixel(rgb, high_priority_pixels, px, py, colour, None);
        }
    }
}

pub(super) fn draw_sprite(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    tile_sheet: &TileSheet,
    palette: &Palette,
    tiles: &[SpriteTile],
) {
    for tile in tiles {
        draw_indexed_tile(
            rgb,
            high_priority_pixels,
            x + tile.offset_x,
            y + tile.offset_y,
            tile_sheet,
            tile.tile,
            palette,
        );
    }
}

pub(super) fn draw_indexed_tile(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    tile_sheet: &TileSheet,
    tile_ref: TileRef,
    palette: &Palette,
) {
    let Some(tile) = tile_sheet.tiles.get(usize::from(tile_ref.tile_index)) else {
        return;
    };

    for dest_y in 0..8 {
        for dest_x in 0..8 {
            let source_x = if tile_ref.flip_x { 7 - dest_x } else { dest_x };
            let source_y = if tile_ref.flip_y { 7 - dest_y } else { dest_y };
            let palette_index = tile.pixels[source_y * 8 + source_x];
            let Some(colour) =
                resolve_palette_colour(palette, tile_ref.palette_line, palette_index)
            else {
                continue;
            };
            set_pixel(
                rgb,
                high_priority_pixels,
                x + dest_x as i32,
                y + dest_y as i32,
                colour,
                Some(tile_ref.priority),
            );
        }
    }
}

pub(super) fn resolve_palette_colour(
    palette: &Palette,
    selected_palette_line: u8,
    palette_index: u8,
) -> Option<[u8; 3]> {
    if palette_index == 0 {
        return None;
    }

    let palette_offset =
        usize::from(selected_palette_line).saturating_mul(PALETTE_LINE_COLOUR_COUNT);
    let colour = palette
        .colours
        .get(palette_offset + usize::from(palette_index - 1))?;

    Some([colour.red, colour.green, colour.blue])
}

pub(super) fn set_pixel(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    colour: [u8; 3],
    priority: Option<bool>,
) {
    if !(0..FRAME_WIDTH as i32).contains(&x) || !(0..FRAME_HEIGHT as i32).contains(&y) {
        return;
    }
    let pixel_index = (y as u32 * FRAME_WIDTH + x as u32) as usize;
    if matches!(priority, Some(false)) && high_priority_pixels[pixel_index] {
        return;
    }
    let rgb_index = pixel_index * 3;
    rgb[rgb_index..rgb_index + 3].copy_from_slice(&colour);
    high_priority_pixels[pixel_index] = matches!(priority, Some(true));
}
