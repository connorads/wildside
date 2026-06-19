use super::*;

/// Render a single sprite frame onto a blank 320x224 canvas at (`at_x`,`at_y`)
/// using the real `build_sprite_tiles` assembly and `draw_sprite` blit, with a
/// magenta backdrop so transparent pixels (where foreground would show through)
/// are obvious. Returns the full-frame RGB buffer; callers crop as needed.
/// Debug helper for inspecting sub-sprite geometry (e.g. RockTank turret #145).
pub(crate) fn render_frame_on_blank_canvas(
    frame: &wildside_render::SpriteFrame,
    palette: &Palette,
    at_x: i32,
    at_y: i32,
) -> Vec<u8> {
    let pixels = (FRAME_WIDTH * FRAME_HEIGHT) as usize;
    let mut rgb = vec![0u8; pixels * 3];
    for pixel in 0..pixels {
        rgb[pixel * 3] = 255; // magenta backdrop marks transparency
        rgb[pixel * 3 + 2] = 255;
    }
    let mut high_priority_pixels = vec![false; pixels];
    let tiles = sprites::build_sprite_tiles(frame, Direction::Right, 0);
    draw_sprite(
        &mut rgb,
        &mut high_priority_pixels,
        at_x,
        at_y,
        &frame.tile_sheet,
        palette,
        &tiles,
    );
    rgb
}

pub(crate) fn render_scene(scene: &Scene, rgb: &mut [u8]) {
    clear_frame(rgb, scene.background);
    render_scene_overlay(scene, rgb);
    if scene.pause_palette_darken {
        apply_pause_palette_darken_loc_6c70(rgb);
    }
}

pub(crate) fn render_scene_overlay(scene: &Scene, rgb: &mut [u8]) {
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
            DrawCommand::Tint { colour, alpha } => tint_frame(rgb, *colour, *alpha),
            DrawCommand::ClearPriorityRect {
                x,
                y,
                width,
                height,
            } => clear_priority_rect(&mut high_priority_pixels, *x, *y, *width, *height),
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
            DrawCommand::IndexedTileRow {
                x,
                y,
                tile_sheet,
                tile,
                palette,
                row,
            } => draw_indexed_tile_row(
                rgb,
                &mut high_priority_pixels,
                *x,
                *y,
                tile_sheet,
                *tile,
                palette,
                *row,
            ),
            DrawCommand::IndexedTileSlice {
                x,
                y,
                tile_sheet,
                tile,
                palette,
                row,
                start_column,
                width,
            } => draw_indexed_tile_slice(
                rgb,
                &mut high_priority_pixels,
                *x,
                *y,
                tile_sheet,
                *tile,
                palette,
                *row,
                *start_column,
                *width,
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

fn tint_frame(rgb: &mut [u8], colour: [u8; 3], alpha: u8) {
    let alpha = u16::from(alpha);
    let inverse = 255u16.saturating_sub(alpha);
    for pixel in rgb.chunks_exact_mut(3) {
        for channel in 0..3 {
            pixel[channel] = ((u16::from(pixel[channel]) * inverse
                + u16::from(colour[channel]) * alpha)
                / 255) as u8;
        }
    }
}

pub(crate) fn sample_scene_pixel_provenance(
    scene: &Scene,
    pixels: &BTreeSet<(usize, usize)>,
) -> Vec<FinalScenePixelDiagnostic> {
    let mut diagnostics = pixels
        .iter()
        .map(|(x, y)| FinalScenePixelDiagnostic {
            x: *x,
            y: *y,
            final_rgb: scene.background,
            final_high_priority: false,
            writes: Vec::new(),
        })
        .collect::<Vec<_>>();

    for (command_index, command) in scene.commands.iter().enumerate() {
        for diagnostic in &mut diagnostics {
            let x = i32::try_from(diagnostic.x).expect("screen x fits i32");
            let y = i32::try_from(diagnostic.y).expect("screen y fits i32");
            let Some(mut write) = command_pixel_write(command_index, command, x, y) else {
                continue;
            };
            write.prior_high_priority = diagnostic.final_high_priority;
            write.accepted =
                !matches!(write.priority, Some(false) if diagnostic.final_high_priority);
            if write.accepted {
                if let Some(rgb) = write.rgb {
                    diagnostic.final_rgb = rgb;
                    diagnostic.final_high_priority = matches!(write.priority, Some(true));
                } else {
                    diagnostic.final_high_priority = false;
                }
            }
            diagnostic.writes.push(write);
        }
    }

    diagnostics
}

fn command_pixel_write(
    command_index: usize,
    command: &DrawCommand,
    x: i32,
    y: i32,
) -> Option<FinalScenePixelWrite> {
    match command {
        DrawCommand::Rect {
            x: cx,
            y: cy,
            width,
            height,
            colour,
        } => rect_contains(*cx, *cy, *width, *height, x, y).then(|| FinalScenePixelWrite {
            command_index,
            kind: "rect".to_owned(),
            origin_x: Some(*cx),
            origin_y: Some(*cy),
            width: Some(*width),
            height: Some(*height),
            tile_index: None,
            palette_line: None,
            palette_index: None,
            source_x: None,
            source_y: None,
            rgb: Some(*colour),
            priority: None,
            prior_high_priority: false,
            accepted: false,
        }),
        DrawCommand::Tint { colour, .. } => ((0..FRAME_WIDTH as i32).contains(&x)
            && (0..FRAME_HEIGHT as i32).contains(&y))
        .then(|| FinalScenePixelWrite {
            command_index,
            kind: "tint".to_owned(),
            origin_x: Some(0),
            origin_y: Some(0),
            width: Some(FRAME_WIDTH as i32),
            height: Some(FRAME_HEIGHT as i32),
            tile_index: None,
            palette_line: None,
            palette_index: None,
            source_x: None,
            source_y: None,
            rgb: Some(*colour),
            priority: None,
            prior_high_priority: false,
            accepted: false,
        }),
        DrawCommand::ClearPriorityRect {
            x: cx,
            y: cy,
            width,
            height,
        } => rect_contains(*cx, *cy, *width, *height, x, y).then(|| FinalScenePixelWrite {
            command_index,
            kind: "clear_priority_rect".to_owned(),
            origin_x: Some(*cx),
            origin_y: Some(*cy),
            width: Some(*width),
            height: Some(*height),
            tile_index: None,
            palette_line: None,
            palette_index: None,
            source_x: None,
            source_y: None,
            rgb: None,
            priority: None,
            prior_high_priority: false,
            accepted: false,
        }),
        DrawCommand::Text {
            x: cx,
            y: cy,
            text,
            colour,
            scale,
        } => text_pixel_visible(*cx, *cy, text, *scale, x, y).map(|(source_x, source_y)| {
            FinalScenePixelWrite {
                command_index,
                kind: "text".to_owned(),
                origin_x: Some(*cx),
                origin_y: Some(*cy),
                width: Some(i32::try_from(text.len()).expect("text length fits i32") * 8 * *scale),
                height: Some(8 * *scale),
                tile_index: None,
                palette_line: None,
                palette_index: None,
                source_x: Some(source_x),
                source_y: Some(source_y),
                rgb: Some(*colour),
                priority: None,
                prior_high_priority: false,
                accepted: false,
            }
        }),
        DrawCommand::IndexedTile {
            x: cx,
            y: cy,
            tile_sheet,
            tile,
            palette,
        } => indexed_tile_pixel_write(
            command_index,
            "indexed_tile",
            *cx,
            *cy,
            tile_sheet,
            *tile,
            palette,
            x,
            y,
        ),
        DrawCommand::IndexedTileRow {
            x: cx,
            y: cy,
            tile_sheet,
            tile,
            palette,
            row,
        } => indexed_tile_row_pixel_write(
            command_index,
            *cx,
            *cy,
            tile_sheet,
            *tile,
            palette,
            *row,
            x,
            y,
        ),
        DrawCommand::IndexedTileSlice {
            x: cx,
            y: cy,
            tile_sheet,
            tile,
            palette,
            row,
            start_column,
            width,
        } => indexed_tile_slice_pixel_write(
            command_index,
            *cx,
            *cy,
            tile_sheet,
            *tile,
            palette,
            *row,
            *start_column,
            *width,
            x,
            y,
        ),
        DrawCommand::Sprite {
            x: cx,
            y: cy,
            tile_sheet,
            palette,
            tiles,
        } => tiles.iter().find_map(|sprite_tile| {
            indexed_tile_pixel_write(
                command_index,
                "sprite_tile",
                *cx + sprite_tile.offset_x,
                *cy + sprite_tile.offset_y,
                tile_sheet,
                sprite_tile.tile,
                palette,
                x,
                y,
            )
        }),
    }
}

fn rect_contains(cx: i32, cy: i32, width: i32, height: i32, x: i32, y: i32) -> bool {
    cx <= x && x < cx + width && cy <= y && y < cy + height
}

fn text_pixel_visible(
    x: i32,
    y: i32,
    text: &str,
    scale: i32,
    pixel_x: i32,
    pixel_y: i32,
) -> Option<(u8, u8)> {
    if scale <= 0 {
        return None;
    }
    let mut cursor_x = x;
    let mut cursor_y = y;
    for ch in text.chars() {
        if ch == '\n' {
            cursor_x = x;
            cursor_y += 10 * scale;
            continue;
        }
        if rect_contains(cursor_x, cursor_y, 8 * scale, 8 * scale, pixel_x, pixel_y) {
            let source_x = (pixel_x - cursor_x) / scale;
            let source_y = (pixel_y - cursor_y) / scale;
            let bitmap = BASIC_FONTS.get(ch)?;
            let bits = bitmap[usize::try_from(source_y).expect("source y fits usize")];
            if bits & (1 << source_x) != 0 {
                return Some((
                    u8::try_from(source_x).expect("source x fits u8"),
                    u8::try_from(source_y).expect("source y fits u8"),
                ));
            }
        }
        cursor_x += 8 * scale;
    }
    None
}

fn indexed_tile_pixel_write(
    command_index: usize,
    kind: &str,
    tile_x: i32,
    tile_y: i32,
    tile_sheet: &TileSheet,
    tile_ref: TileRef,
    palette: &Palette,
    pixel_x: i32,
    pixel_y: i32,
) -> Option<FinalScenePixelWrite> {
    if !rect_contains(tile_x, tile_y, 8, 8, pixel_x, pixel_y) {
        return None;
    }
    let dest_x = usize::try_from(pixel_x - tile_x).expect("tile local x fits usize");
    let dest_y = usize::try_from(pixel_y - tile_y).expect("tile local y fits usize");
    let source_x = if tile_ref.flip_x { 7 - dest_x } else { dest_x };
    let source_y = if tile_ref.flip_y { 7 - dest_y } else { dest_y };
    indexed_tile_source_pixel_write(
        command_index,
        kind,
        tile_x,
        tile_y,
        tile_sheet,
        tile_ref,
        palette,
        source_x,
        source_y,
    )
}

fn indexed_tile_row_pixel_write(
    command_index: usize,
    tile_x: i32,
    tile_y: i32,
    tile_sheet: &TileSheet,
    tile_ref: TileRef,
    palette: &Palette,
    row: u8,
    pixel_x: i32,
    pixel_y: i32,
) -> Option<FinalScenePixelWrite> {
    if tile_y != pixel_y || !(tile_x..tile_x + 8).contains(&pixel_x) {
        return None;
    }
    let dest_x = usize::try_from(pixel_x - tile_x).expect("tile local x fits usize");
    let dest_y = usize::from(row.min(7));
    let source_x = if tile_ref.flip_x { 7 - dest_x } else { dest_x };
    let source_y = if tile_ref.flip_y { 7 - dest_y } else { dest_y };
    indexed_tile_source_pixel_write(
        command_index,
        "indexed_tile_row",
        tile_x,
        tile_y,
        tile_sheet,
        tile_ref,
        palette,
        source_x,
        source_y,
    )
}

#[allow(clippy::too_many_arguments)]
fn indexed_tile_slice_pixel_write(
    command_index: usize,
    tile_x: i32,
    tile_y: i32,
    tile_sheet: &TileSheet,
    tile_ref: TileRef,
    palette: &Palette,
    row: u8,
    start_column: u8,
    width: u8,
    pixel_x: i32,
    pixel_y: i32,
) -> Option<FinalScenePixelWrite> {
    let start_column = start_column.min(7);
    let end_column = start_column.saturating_add(width).min(8);
    let start_x = tile_x + i32::from(start_column);
    let end_x = tile_x + i32::from(end_column);
    if tile_y + i32::from(row.min(7)) != pixel_y || !(start_x..end_x).contains(&pixel_x) {
        return None;
    }
    let dest_x = usize::try_from(pixel_x - tile_x).expect("tile local x fits usize");
    let dest_y = usize::from(row.min(7));
    let source_x = if tile_ref.flip_x { 7 - dest_x } else { dest_x };
    let source_y = if tile_ref.flip_y { 7 - dest_y } else { dest_y };
    indexed_tile_source_pixel_write(
        command_index,
        "indexed_tile_slice",
        tile_x,
        tile_y,
        tile_sheet,
        tile_ref,
        palette,
        source_x,
        source_y,
    )
}

fn indexed_tile_source_pixel_write(
    command_index: usize,
    kind: &str,
    tile_x: i32,
    tile_y: i32,
    tile_sheet: &TileSheet,
    tile_ref: TileRef,
    palette: &Palette,
    source_x: usize,
    source_y: usize,
) -> Option<FinalScenePixelWrite> {
    let tile = tile_sheet.tiles.get(usize::from(tile_ref.tile_index))?;
    let palette_index = tile.pixels[source_y * 8 + source_x];
    let colour = resolve_palette_colour(palette, tile_ref.palette_line, palette_index)?;
    Some(FinalScenePixelWrite {
        command_index,
        kind: kind.to_owned(),
        origin_x: Some(tile_x),
        origin_y: Some(tile_y),
        width: Some(8),
        height: Some(8),
        tile_index: Some(tile_ref.tile_index),
        palette_line: Some(tile_ref.palette_line),
        palette_index: Some(palette_index),
        source_x: Some(u8::try_from(source_x).expect("source x fits u8")),
        source_y: Some(u8::try_from(source_y).expect("source y fits u8")),
        rgb: Some(colour),
        priority: Some(tile_ref.priority),
        prior_high_priority: false,
        accepted: false,
    })
}

pub(super) fn clear_frame(rgb: &mut [u8], colour: [u8; 3]) {
    for chunk in rgb.chunks_exact_mut(3) {
        chunk.copy_from_slice(&colour);
    }
}

pub(super) fn apply_pause_palette_darken_loc_6c70(rgb: &mut [u8]) {
    // ASM Pause_DarkenPalette loc_6C70/loc_6C9E halves each MD CRAM channel
    // before Palette_to_VRAM uploads the paused palette. Native-player has
    // already resolved palette slots into expanded RGB by this point, so map
    // each RGB channel back to its nearest 3-bit MD value, halve it, then
    // expand it again.
    for channel in rgb {
        *channel = pause_darken_expanded_channel_loc_6c70(*channel);
    }
}

fn pause_darken_expanded_channel_loc_6c70(channel: u8) -> u8 {
    let md_channel = ((u16::from(channel) * 7 + 127) / 255).min(7);
    ((md_channel >> 1) * 255 / 7) as u8
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

pub(super) fn clear_priority_rect(
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    for py in y.max(0)..(y + height).min(FRAME_HEIGHT as i32) {
        for px in x.max(0)..(x + width).min(FRAME_WIDTH as i32) {
            let pixel_index = (py as u32 * FRAME_WIDTH + px as u32) as usize;
            high_priority_pixels[pixel_index] = false;
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

pub(super) fn draw_indexed_tile_row(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    tile_sheet: &TileSheet,
    tile_ref: TileRef,
    palette: &Palette,
    row: u8,
) {
    let Some(tile) = tile_sheet.tiles.get(usize::from(tile_ref.tile_index)) else {
        return;
    };
    let dest_y = usize::from(row.min(7));
    let source_y = if tile_ref.flip_y { 7 - dest_y } else { dest_y };

    for dest_x in 0..8 {
        let source_x = if tile_ref.flip_x { 7 - dest_x } else { dest_x };
        let palette_index = tile.pixels[source_y * 8 + source_x];
        let Some(colour) = resolve_palette_colour(palette, tile_ref.palette_line, palette_index)
        else {
            continue;
        };
        set_pixel(
            rgb,
            high_priority_pixels,
            x + dest_x as i32,
            y,
            colour,
            Some(tile_ref.priority),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_indexed_tile_slice(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    x: i32,
    y: i32,
    tile_sheet: &TileSheet,
    tile_ref: TileRef,
    palette: &Palette,
    row: u8,
    start_column: u8,
    width: u8,
) {
    let Some(tile) = tile_sheet.tiles.get(usize::from(tile_ref.tile_index)) else {
        return;
    };
    let dest_y = usize::from(row.min(7));
    let source_y = if tile_ref.flip_y { 7 - dest_y } else { dest_y };
    let start_column = start_column.min(7);
    let end_column = start_column.saturating_add(width).min(8);

    for dest_x in start_column..end_column {
        let dest_x = usize::from(dest_x);
        let source_x = if tile_ref.flip_x { 7 - dest_x } else { dest_x };
        let palette_index = tile.pixels[source_y * 8 + source_x];
        let Some(colour) = resolve_palette_colour(palette, tile_ref.palette_line, palette_index)
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

    let colour = if palette.colours.len() == 16 {
        // ASM loc_1AB40 copies Pal_19C48 as a complete 16-word title-card
        // palette line; Load_EnemyArtPaletteToVRAM loc_36D16 similarly
        // selects an enemy CRAM line before native stores it as one local
        // 16-slot Palette. Unlike 15-colour visible-only level palettes,
        // slot 0 is present in the data, so non-zero tile indices address
        // their matching MD palette slot directly without reapplying the
        // sprite's hardware palette-line bits.
        palette.colours.get(usize::from(palette_index))
    } else if palette.colours.len() == 8 {
        match selected_palette_line {
            0 => palette.colours.get(usize::from(palette_index)),
            // ASM `sub_307D8` streams chunked cave background bytes through
            // `unk_309B8`, whose tile words are `$2780 + tile_id`: palette
            // line 1 with normal 4bpp pixel nibbles 1..7. Native stores the
            // compact 8-word background palette alone, so those nibbles still
            // address the local palette directly.
            1 | 2 if (1..=7).contains(&palette_index) => {
                palette.colours.get(usize::from(palette_index))
            }
            1 | 2 if (8..=15).contains(&palette_index) => {
                palette.colours.get(usize::from(palette_index - 8))
            }
            _ => None,
        }
    } else {
        let palette_offset =
            usize::from(selected_palette_line).saturating_mul(PALETTE_LINE_COLOUR_COUNT);
        palette
            .colours
            .get(palette_offset + usize::from(palette_index - 1))
    }?;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_colour(slot: u16) -> MdColour {
        MdColour {
            red: slot as u8,
            green: slot.saturating_mul(2) as u8,
            blue: slot.saturating_mul(3) as u8,
            raw_word: slot,
        }
    }

    #[test]
    fn expanded_enemy_palette_slots_ignore_sprite_palette_line_loc_36d16() {
        // ASM Load_EnemyArtPaletteToVRAM loc_36D16 selects the enemy CRAM
        // line, while native stores the selected line as one 16-slot Palette.
        // Offsetting that local palette again by the sprite's palette_line
        // makes valid enemy pixels disappear.
        let palette = Palette {
            colours: (0..16).map(test_colour).collect(),
        };

        assert_eq!(resolve_palette_colour(&palette, 1, 8), Some([8, 16, 24]));
    }

    #[test]
    fn compact_background_palette_line_one_uses_local_slots_loc_307d8() {
        // ASM sub_307D8 indexes `unk_309B8`, which emits `$2780 + tile_id`
        // for chunked backgrounds: palette line 1, tile pixels still in
        // compact slots 1..7. Native stores those eight colours locally.
        let palette = Palette {
            colours: (0..8).map(test_colour).collect(),
        };

        assert_eq!(resolve_palette_colour(&palette, 1, 1), Some([1, 2, 3]));
    }

    #[test]
    fn pause_palette_darken_halves_expanded_md_channels_loc_6c70() {
        let mut rgb = vec![255, 218, 145, 109, 72, 36, 0];

        apply_pause_palette_darken_loc_6c70(&mut rgb);

        assert_eq!(
            rgb,
            vec![109, 109, 72, 36, 36, 0, 0],
            "ASM Pause_DarkenPalette loc_6C70 halves each 3-bit MD colour channel before re-expansion"
        );
    }

    #[test]
    fn tint_frame_blends_teleport_colour_loc_bcf6() {
        let mut rgb = vec![0, 0, 0, 255, 255, 255];

        tint_frame(&mut rgb, [255, 255, 0], 96);

        assert_eq!(
            rgb,
            vec![96, 96, 0, 255, 255, 159],
            "ASM loc_BCF6 seeds `$FFFFFBCC=$00EE`; native-player blends that fade colour over the transition frame",
        );
    }
}
