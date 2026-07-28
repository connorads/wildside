use std::collections::BTreeSet;

use font8x8::{BASIC_FONTS, UnicodeFonts};
use wildside_core::{pause_darken_palette_loc_6C70, physics::Direction};
use wildside_render::{MdColour, Palette, TileRef, TileSheet};

use crate::dto::*;
use crate::sprites;

/// Render a single sprite frame onto a blank 320x224 canvas at (`at_x`,`at_y`)
/// using the real `build_sprite_tiles` assembly and `draw_sprite` blit, with a
/// magenta backdrop so transparent pixels (where foreground would show through)
/// are obvious. Returns the full-frame packed RGB24 buffer
/// (`len == FRAME_WIDTH * FRAME_HEIGHT * 3`, no alpha); callers crop as needed.
/// Debug helper for inspecting sub-sprite geometry (e.g. RockTank turret #145).
pub fn render_frame_on_blank_canvas(
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

/// Rasterise `scene` into `rgb`, a packed RGB24 framebuffer
/// (`len == FRAME_WIDTH * FRAME_HEIGHT * 3`, no alpha). The web build expands
/// this to RGBA at its blit boundary; native SDL uploads it as `RGB24`.
pub fn render_scene(scene: &Scene, rgb: &mut [u8]) {
    clear_frame(rgb, scene.background);
    render_scene_overlay(scene, rgb);
    if scene.pause_palette_darken {
        apply_pause_palette_darken_loc_6c70(rgb);
    }
}

/// Composite `scene`'s draw commands onto `rgb` (packed RGB24, no clear) —
/// `render_scene` without the backdrop fill or pause darken.
pub fn render_scene_overlay(scene: &Scene, rgb: &mut [u8]) {
    let mut high_priority_pixels = vec![false; (FRAME_WIDTH * FRAME_HEIGHT) as usize];
    let mut sat_opaque_pixels = vec![false; (FRAME_WIDTH * FRAME_HEIGHT) as usize];
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
            DrawCommand::PaletteFadeSub73D0 {
                target_word,
                fade_progress,
            } => palette_fade_frame_sub_73d0(rgb, *target_word, *fade_progress),
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
            DrawCommand::SatSprite {
                x,
                y,
                tile_sheet,
                palette,
                tiles,
            } => draw_sat_sprite_first_opaque(
                rgb,
                &mut high_priority_pixels,
                &mut sat_opaque_pixels,
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

fn palette_fade_frame_sub_73d0(rgb: &mut [u8], target_word: u16, fade_progress: u16) {
    // ASM sub_73D0 / sub_14C0 (kid.asm:7527,2583): the level-entry worker
    // fades the complete CRAM palette. The scene renderer has already expanded
    // MD palette entries to RGB, so quantise each channel back to its 3-bit MD
    // value, run the ROM word blend, then expand again.
    for pixel in rgb.chunks_exact_mut(3) {
        let source_word = md_word_from_rgb(pixel[0], pixel[1], pixel[2]);
        let faded_word =
            wildside_core::blend_palette_word_sub_14C0(fade_progress, source_word, target_word);
        let colour = MdColour::from_word(faded_word);
        pixel.copy_from_slice(&[colour.red, colour.green, colour.blue]);
    }
}

fn md_word_from_rgb(red: u8, green: u8, blue: u8) -> u16 {
    let red = md_channel_from_expanded_rgb(red);
    let green = md_channel_from_expanded_rgb(green);
    let blue = md_channel_from_expanded_rgb(blue);
    (u16::from(blue) << 9) + (u16::from(green) << 5) + (u16::from(red) << 1)
}

fn md_channel_from_expanded_rgb(value: u8) -> u8 {
    ((u16::from(value) * 7 + 127) / 255) as u8
}

pub fn sample_scene_pixel_provenance(
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
            palette_colour_count: None,
            resolved_palette_slot: None,
            raw_palette_word: None,
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
            palette_colour_count: None,
            resolved_palette_slot: None,
            raw_palette_word: None,
            source_x: None,
            source_y: None,
            rgb: Some(*colour),
            priority: None,
            prior_high_priority: false,
            accepted: false,
        }),
        DrawCommand::PaletteFadeSub73D0 { .. } => ((0..FRAME_WIDTH as i32).contains(&x)
            && (0..FRAME_HEIGHT as i32).contains(&y))
        .then(|| FinalScenePixelWrite {
            command_index,
            kind: "palette_fade_sub_73d0".to_owned(),
            origin_x: Some(0),
            origin_y: Some(0),
            width: Some(FRAME_WIDTH as i32),
            height: Some(FRAME_HEIGHT as i32),
            tile_index: None,
            palette_line: None,
            palette_index: None,
            palette_colour_count: None,
            resolved_palette_slot: None,
            raw_palette_word: None,
            source_x: None,
            source_y: None,
            rgb: None,
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
            palette_colour_count: None,
            resolved_palette_slot: None,
            raw_palette_word: None,
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
                palette_colour_count: None,
                resolved_palette_slot: None,
                raw_palette_word: None,
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
        }
        | DrawCommand::SatSprite {
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
    let (resolved_palette_slot, colour) =
        resolve_palette_entry(palette, tile_ref.palette_line, palette_index)?;
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
        palette_colour_count: Some(palette.colours.len()),
        resolved_palette_slot: Some(resolved_palette_slot),
        raw_palette_word: palette
            .colours
            .get(resolved_palette_slot)
            .map(|colour| colour.raw_word),
        source_x: Some(u8::try_from(source_x).expect("source x fits u8")),
        source_y: Some(u8::try_from(source_y).expect("source y fits u8")),
        rgb: Some(colour),
        priority: Some(tile_ref.priority),
        prior_high_priority: false,
        accepted: false,
    })
}

pub fn clear_frame(rgb: &mut [u8], colour: [u8; 3]) {
    for chunk in rgb.chunks_exact_mut(3) {
        chunk.copy_from_slice(&colour);
    }
}

pub fn apply_pause_palette_darken_loc_6c70(rgb: &mut [u8]) {
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
    let raw_word = md_channel << 1;
    let mut palette_buffer = [0u16; 0x40];
    palette_buffer[0] = raw_word;
    pause_darken_palette_loc_6C70(&mut palette_buffer);
    let darkened_md_channel = (palette_buffer[0] >> 1) & 0x7;
    (darkened_md_channel * 255 / 7) as u8
}

pub fn push_panel(
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

pub fn text(x: i32, y: i32, value: &str, colour: [u8; 3], scale: i32) -> DrawCommand {
    DrawCommand::Text {
        x,
        y,
        text: value.to_string(),
        colour,
        scale,
    }
}

pub fn draw_text(
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

pub fn draw_glyph(
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

pub fn fill_rect(
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

pub fn clear_priority_rect(
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

pub fn draw_sprite(
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

/// Rasterise source-ordered Sprite_Table entries.
///
/// ASM Make_SpritesFromGfxObjects (kid.asm:2558-2576) writes a linked SAT in
/// GfxObject priority order. The VDP retains the first non-transparent sprite
/// pixel it sees, independently from whether a plane later hides that pixel.
#[allow(clippy::too_many_arguments)]
fn draw_sat_sprite_first_opaque(
    rgb: &mut [u8],
    high_priority_pixels: &mut [bool],
    sat_opaque_pixels: &mut [bool],
    x: i32,
    y: i32,
    tile_sheet: &TileSheet,
    palette: &Palette,
    tiles: &[SpriteTile],
) {
    for sprite_tile in tiles {
        let Some(tile) = tile_sheet
            .tiles
            .get(usize::from(sprite_tile.tile.tile_index))
        else {
            continue;
        };
        for dest_y in 0..8 {
            for dest_x in 0..8 {
                let source_x = if sprite_tile.tile.flip_x {
                    7 - dest_x
                } else {
                    dest_x
                };
                let source_y = if sprite_tile.tile.flip_y {
                    7 - dest_y
                } else {
                    dest_y
                };
                let palette_index = tile.pixels[source_y * 8 + source_x];
                let Some(colour) =
                    resolve_palette_colour(palette, sprite_tile.tile.palette_line, palette_index)
                else {
                    continue;
                };
                let pixel_x = x + sprite_tile.offset_x + dest_x as i32;
                let pixel_y = y + sprite_tile.offset_y + dest_y as i32;
                if !(0..FRAME_WIDTH as i32).contains(&pixel_x)
                    || !(0..FRAME_HEIGHT as i32).contains(&pixel_y)
                {
                    continue;
                }
                let pixel_index = (pixel_y as u32 * FRAME_WIDTH + pixel_x as u32) as usize;
                if sat_opaque_pixels[pixel_index] {
                    continue;
                }
                sat_opaque_pixels[pixel_index] = true;
                set_pixel(
                    rgb,
                    high_priority_pixels,
                    pixel_x,
                    pixel_y,
                    colour,
                    Some(sprite_tile.tile.priority),
                );
            }
        }
    }
}

pub fn draw_indexed_tile_row(
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
pub fn draw_indexed_tile_slice(
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

pub fn draw_indexed_tile(
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

pub fn resolve_palette_colour(
    palette: &Palette,
    selected_palette_line: u8,
    palette_index: u8,
) -> Option<[u8; 3]> {
    resolve_palette_entry(palette, selected_palette_line, palette_index).map(|(_, colour)| colour)
}

fn resolve_palette_entry(
    palette: &Palette,
    selected_palette_line: u8,
    palette_index: u8,
) -> Option<(usize, [u8; 3])> {
    if palette_index == 0 {
        return None;
    }

    let resolved_slot = if palette.colours.len() == 16 {
        // ASM loc_1AB40 copies Pal_19C48 as a complete 16-word title-card
        // palette line; Load_EnemyArtPaletteToVRAM loc_36D16 similarly
        // selects an enemy CRAM line before native stores it as one local
        // 16-slot Palette. Unlike 15-colour visible-only level palettes,
        // slot 0 is present in the data, so non-zero tile indices address
        // their matching MD palette slot directly without reapplying the
        // sprite's hardware palette-line bits.
        usize::from(palette_index)
    } else if palette.colours.len() == 8 {
        match selected_palette_line {
            0 => Some(usize::from(palette_index)),
            // ASM `sub_307D8` streams chunked cave background bytes through
            // `unk_309B8`, whose tile words are `$2780 + tile_id`: palette
            // line 1 with normal 4bpp pixel nibbles 1..7. Native stores the
            // compact 8-word background palette alone, so those nibbles still
            // address the local palette directly.
            1 | 2 => match palette_index {
                1..=7 => Some(usize::from(palette_index)),
                8..=15 => Some(usize::from(palette_index - 8)),
                _ => None,
            },
            _ => None,
        }?
    } else {
        let palette_offset =
            usize::from(selected_palette_line).saturating_mul(PALETTE_LINE_COLOUR_COUNT);
        palette_offset + usize::from(palette_index - 1)
    };
    let colour = palette.colours.get(resolved_slot)?;

    Some((resolved_slot, [colour.red, colour.green, colour.blue]))
}

pub fn set_pixel(
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
    use std::sync::Arc;
    use wildside_render::{Palette, Tile, TileRef, TileSheet};

    fn test_colour(slot: u16) -> MdColour {
        MdColour {
            red: slot as u8,
            green: slot.saturating_mul(2) as u8,
            blue: slot.saturating_mul(3) as u8,
            raw_word: slot,
        }
    }

    fn test_palette(count: u16) -> Arc<Palette> {
        Arc::new(Palette {
            colours: (0..count).map(test_colour).collect(),
        })
    }

    fn test_tile_sheet() -> Arc<TileSheet> {
        let mut pixels = [0; 64];
        for y in 0..8 {
            for x in 0..8 {
                pixels[y * 8 + x] = ((x + y) % 15 + 1) as u8;
            }
        }
        Arc::new(TileSheet {
            tiles: vec![Tile { pixels }],
        })
    }

    fn test_tile_ref(priority: bool) -> TileRef {
        TileRef {
            tile_index: 0,
            palette_line: 0,
            priority,
            flip_x: false,
            flip_y: false,
        }
    }

    fn test_sprite_frame() -> wildside_render::SpriteFrame {
        wildside_render::SpriteFrame {
            origin_x: 0,
            origin_y: 0,
            width_px: 8,
            height_px: 8,
            tile_base_index: 0,
            tile_sheet: (*test_tile_sheet()).clone(),
        }
    }

    fn pixel_rgb(rgb: &[u8], x: i32, y: i32) -> [u8; 3] {
        let index = ((y as u32 * FRAME_WIDTH + x as u32) as usize) * 3;
        [rgb[index], rgb[index + 1], rgb[index + 2]]
    }

    fn first_visible_glyph_pixel(ch: char) -> (i32, i32) {
        let bitmap = BASIC_FONTS.get(ch).expect("font has glyph");
        for (source_y, bits) in bitmap.iter().enumerate() {
            for source_x in 0..8 {
                if bits & (1 << source_x) != 0 {
                    return (source_x, source_y as i32);
                }
            }
        }
        panic!("glyph should have a visible pixel");
    }

    fn visible_glyph_pixel_at_or_below(ch: char, min_source_y: usize) -> (i32, i32) {
        let bitmap = BASIC_FONTS.get(ch).expect("font has glyph");
        for (source_y, bits) in bitmap.iter().enumerate().skip(min_source_y) {
            for source_x in 0..8 {
                if bits & (1 << source_x) != 0 {
                    return (source_x, source_y as i32);
                }
            }
        }
        panic!("glyph should have a visible lower-row pixel");
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
    fn palette_resolution_covers_compact_and_full_palette_offsets() {
        let compact = Palette {
            colours: (0..8).map(test_colour).collect(),
        };
        assert_eq!(resolve_palette_colour(&compact, 0, 0), None);
        assert_eq!(resolve_palette_colour(&compact, 0, 7), Some([7, 14, 21]));
        assert_eq!(resolve_palette_colour(&compact, 1, 0), None);
        assert_eq!(resolve_palette_colour(&compact, 2, 15), Some([7, 14, 21]));
        assert_eq!(resolve_palette_colour(&compact, 3, 1), None);

        let full = Palette {
            colours: (0..30).map(test_colour).collect(),
        };
        assert_eq!(resolve_palette_colour(&full, 0, 1), Some([0, 0, 0]));
        assert_eq!(resolve_palette_colour(&full, 1, 1), Some([15, 30, 45]));
        assert_eq!(resolve_palette_colour(&full, 1, 15), Some([29, 58, 87]));
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

    #[test]
    fn palette_fade_command_blends_frame_through_sub_14c0() {
        let source = MdColour::from_word(0x0EEE);
        let mut rgb = vec![source.red, source.green, source.blue];

        palette_fade_frame_sub_73d0(&mut rgb, 0x0000, 0x0080);

        let expected = MdColour::from_word(wildside_core::blend_palette_word_sub_14C0(
            0x0080, 0x0EEE, 0x0000,
        ));
        assert_eq!(
            rgb,
            vec![expected.red, expected.green, expected.blue],
            "ASM sub_73D0 calls sub_14C0 for the full CRAM fade; the renderer should post-process already-expanded MD pixels through the same palette-word blend"
        );
    }

    #[test]
    fn palette_fade_command_starts_black_and_ends_bright_sub_73d0() {
        let source = MdColour::from_word(0x0ACE);
        let mut first = vec![source.red, source.green, source.blue];
        palette_fade_frame_sub_73d0(&mut first, 0x0000, 0x0000);

        let mut final_frame = vec![source.red, source.green, source.blue];
        palette_fade_frame_sub_73d0(&mut final_frame, 0x0000, 0x0100);

        assert_eq!(
            first,
            vec![0, 0, 0],
            "ASM sub_73D0 clears `$FFFFF876` before the first sub_14C0 fade step"
        );
        assert_eq!(
            final_frame,
            vec![source.red, source.green, source.blue],
            "ASM loc_741E stops the fade only after progress passes `$0100`, so `$0100` is the full-bright frame"
        );
    }

    #[test]
    fn render_frame_on_blank_canvas_returns_full_rgb_frame_with_magenta_backdrop() {
        let palette = test_palette(16);
        let frame = test_sprite_frame();

        let rgb = render_frame_on_blank_canvas(&frame, &palette, 13, 17);

        assert_eq!(rgb.len(), (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize);
        assert_eq!(pixel_rgb(&rgb, 0, 0), [255, 0, 255]);
        assert_eq!(pixel_rgb(&rgb, 12, 17), [255, 0, 255]);
        assert_eq!(pixel_rgb(&rgb, 13, 17), [1, 2, 3]);
        assert_eq!(pixel_rgb(&rgb, 20, 24), [15, 30, 45]);
    }

    #[test]
    fn render_scene_clears_background_and_overlay_only_composites_commands() {
        let scene = Scene {
            background: [3, 4, 5],
            commands: vec![DrawCommand::Rect {
                x: 4,
                y: 6,
                width: 2,
                height: 3,
                colour: [9, 8, 7],
            }],
            pause_palette_darken: false,
        };
        let mut rgb = vec![1; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];

        render_scene_overlay(&scene, &mut rgb);
        assert_eq!(pixel_rgb(&rgb, 0, 0), [1, 1, 1]);
        assert_eq!(pixel_rgb(&rgb, 4, 6), [9, 8, 7]);
        assert_eq!(pixel_rgb(&rgb, 5, 8), [9, 8, 7]);

        render_scene(&scene, &mut rgb);
        assert_eq!(pixel_rgb(&rgb, 0, 0), [3, 4, 5]);
        assert_eq!(pixel_rgb(&rgb, 4, 6), [9, 8, 7]);
        assert_eq!(pixel_rgb(&rgb, 6, 8), [3, 4, 5]);
    }

    #[test]
    fn push_panel_emits_fill_and_two_pixel_border_rects() {
        let mut commands = Vec::new();

        push_panel(&mut commands, 10, 20, 30, 40, [1, 2, 3], [4, 5, 6]);

        assert_eq!(
            commands,
            vec![
                DrawCommand::Rect {
                    x: 10,
                    y: 20,
                    width: 30,
                    height: 40,
                    colour: [1, 2, 3],
                },
                DrawCommand::Rect {
                    x: 10,
                    y: 20,
                    width: 30,
                    height: 2,
                    colour: [4, 5, 6],
                },
                DrawCommand::Rect {
                    x: 10,
                    y: 58,
                    width: 30,
                    height: 2,
                    colour: [4, 5, 6],
                },
                DrawCommand::Rect {
                    x: 10,
                    y: 20,
                    width: 2,
                    height: 40,
                    colour: [4, 5, 6],
                },
                DrawCommand::Rect {
                    x: 38,
                    y: 20,
                    width: 2,
                    height: 40,
                    colour: [4, 5, 6],
                },
            ]
        );
    }

    #[test]
    fn draw_text_rasterises_scaled_glyphs_and_newlines() {
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
        let mut high_priority_pixels = vec![false; (FRAME_WIDTH * FRAME_HEIGHT) as usize];
        let colour = [7, 8, 9];
        let (a_x, a_y) = first_visible_glyph_pixel('A');
        let (b_x, b_y) = first_visible_glyph_pixel('B');
        let (lower_8_x, lower_8_y) = visible_glyph_pixel_at_or_below('8', 5);
        let blank_a_row = BASIC_FONTS
            .get('A')
            .expect("font has A")
            .iter()
            .position(|bits| *bits == 0)
            .expect("A has a blank source row");

        draw_text(
            &mut rgb,
            &mut high_priority_pixels,
            30,
            40,
            "A\nB",
            colour,
            2,
        );

        assert_eq!(pixel_rgb(&rgb, 30 + a_x * 2, 40 + a_y * 2), colour);
        assert_eq!(
            pixel_rgb(&rgb, 30, 40 + blank_a_row as i32 * 2),
            [0, 0, 0],
            "transparent glyph rows stay transparent"
        );
        assert_eq!(
            pixel_rgb(&rgb, 30 + 16 + b_x * 2, 40 + b_y * 2),
            [0, 0, 0],
            "newline resets the cursor instead of drawing B on A's row"
        );
        assert_eq!(pixel_rgb(&rgb, 30 + b_x * 2, 60 + b_y * 2), colour);

        clear_frame(&mut rgb, [0, 0, 0]);
        draw_text(&mut rgb, &mut high_priority_pixels, 30, 90, "AB", colour, 2);
        assert_eq!(
            pixel_rgb(&rgb, 30 + 16 + b_x * 2, 90 + b_y * 2),
            colour,
            "same-line glyphs advance by 8 scaled source columns"
        );
        assert_eq!(
            pixel_rgb(&rgb, 30 + 10 + b_x * 2, 90 + b_y * 2),
            [0, 0, 0],
            "same-line glyphs do not use an unscaled or under-scaled advance"
        );

        clear_frame(&mut rgb, [0, 0, 0]);
        draw_text(&mut rgb, &mut high_priority_pixels, 30, 120, "8", colour, 2);
        assert_eq!(
            pixel_rgb(&rgb, 30 + lower_8_x * 2, 120 + lower_8_y * 2),
            colour,
            "scaled glyph rows are offset from the glyph origin"
        );
    }

    #[test]
    fn draw_sprite_applies_tile_offsets_before_blitting() {
        let tile_sheet = test_tile_sheet();
        let palette = test_palette(16);
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
        let mut high_priority_pixels = vec![false; (FRAME_WIDTH * FRAME_HEIGHT) as usize];

        draw_sprite(
            &mut rgb,
            &mut high_priority_pixels,
            7,
            9,
            &tile_sheet,
            &palette,
            &[SpriteTile {
                offset_x: 3,
                offset_y: 4,
                tile: test_tile_ref(false),
            }],
        );

        assert_eq!(pixel_rgb(&rgb, 7, 9), [0, 0, 0]);
        assert_eq!(pixel_rgb(&rgb, 10, 13), [1, 2, 3]);
        assert_eq!(pixel_rgb(&rgb, 17, 20), [15, 30, 45]);
    }

    #[test]
    fn sat_sprites_keep_the_first_opaque_pixel() {
        // ASM Make_SpritesFromGfxObjects (kid.asm:2558-2576) emits the
        // source-ordered Sprite_Table. The VDP retains its first opaque pixel.
        let tile_sheet = test_tile_sheet();
        let first_palette = test_palette(16);
        let mut later_palette = (*test_palette(16)).clone();
        later_palette.colours[1] = MdColour {
            red: 90,
            green: 91,
            blue: 92,
            raw_word: 0,
        };
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
        render_scene(
            &Scene {
                background: [0, 0, 0],
                commands: vec![
                    DrawCommand::SatSprite {
                        x: 10,
                        y: 10,
                        tile_sheet: Arc::clone(&tile_sheet),
                        palette: first_palette,
                        tiles: vec![SpriteTile {
                            offset_x: 0,
                            offset_y: 0,
                            tile: test_tile_ref(false),
                        }],
                    },
                    DrawCommand::SatSprite {
                        x: 10,
                        y: 10,
                        tile_sheet,
                        palette: Arc::new(later_palette),
                        tiles: vec![SpriteTile {
                            offset_x: 0,
                            offset_y: 0,
                            tile: test_tile_ref(false),
                        }],
                    },
                ],
                pause_palette_darken: false,
            },
            &mut rgb,
        );

        assert_eq!(pixel_rgb(&rgb, 10, 10), [1, 2, 3]);
    }

    #[test]
    fn command_pixel_write_reports_global_bounds_text_dimensions_and_sprite_offsets() {
        assert!(
            command_pixel_write(
                0,
                &DrawCommand::Tint {
                    colour: [1, 2, 3],
                    alpha: 4,
                },
                0,
                0,
            )
            .is_some()
        );
        assert!(
            command_pixel_write(
                0,
                &DrawCommand::Tint {
                    colour: [1, 2, 3],
                    alpha: 4,
                },
                0,
                FRAME_HEIGHT as i32,
            )
            .is_none()
        );
        assert!(
            command_pixel_write(
                1,
                &DrawCommand::PaletteFadeSub73D0 {
                    target_word: 0,
                    fade_progress: 0,
                },
                FRAME_WIDTH as i32,
                0,
            )
            .is_none()
        );

        let text = command_pixel_write(
            2,
            &DrawCommand::Text {
                x: 10,
                y: 20,
                text: "AB".to_owned(),
                colour: [9, 8, 7],
                scale: 2,
            },
            11,
            24,
        )
        .expect("text command should report visible pixel provenance");
        assert_eq!(text.width, Some(32));
        assert_eq!(text.height, Some(16));

        let tile_sheet = test_tile_sheet();
        let palette = test_palette(16);
        let sprite = command_pixel_write(
            3,
            &DrawCommand::Sprite {
                x: 10,
                y: 20,
                tile_sheet,
                palette,
                tiles: vec![SpriteTile {
                    offset_x: 2,
                    offset_y: 3,
                    tile: test_tile_ref(false),
                }],
            },
            12,
            23,
        )
        .expect("sprite command should apply per-tile offsets");
        assert_eq!(sprite.origin_x, Some(12));
        assert_eq!(sprite.origin_y, Some(23));
        assert_eq!(sprite.source_x, Some(0));
        assert_eq!(sprite.source_y, Some(0));
    }

    #[test]
    fn rect_contains_uses_inclusive_start_exclusive_end_edges() {
        assert!(rect_contains(10, 20, 3, 4, 10, 20));
        assert!(rect_contains(10, 20, 3, 4, 12, 23));
        assert!(!rect_contains(10, 20, 3, 4, 9, 20));
        assert!(!rect_contains(10, 20, 3, 4, 13, 20));
        assert!(!rect_contains(10, 20, 3, 4, 10, 19));
        assert!(!rect_contains(10, 20, 3, 4, 10, 24));
    }

    #[test]
    fn set_pixel_fill_and_clear_priority_apply_bounds_and_priority() {
        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
        let mut high_priority_pixels = vec![false; (FRAME_WIDTH * FRAME_HEIGHT) as usize];

        fill_rect(&mut rgb, &mut high_priority_pixels, -1, -1, 3, 3, [1, 2, 3]);
        assert_eq!(pixel_rgb(&rgb, 0, 0), [1, 2, 3]);
        assert_eq!(pixel_rgb(&rgb, 1, 1), [1, 2, 3]);
        assert_eq!(pixel_rgb(&rgb, 2, 2), [0, 0, 0]);

        set_pixel(
            &mut rgb,
            &mut high_priority_pixels,
            4,
            5,
            [9, 0, 0],
            Some(true),
        );
        set_pixel(
            &mut rgb,
            &mut high_priority_pixels,
            4,
            5,
            [0, 9, 0],
            Some(false),
        );
        assert_eq!(pixel_rgb(&rgb, 4, 5), [9, 0, 0]);

        clear_priority_rect(&mut high_priority_pixels, 4, 5, 1, 1);
        set_pixel(
            &mut rgb,
            &mut high_priority_pixels,
            4,
            5,
            [0, 9, 0],
            Some(false),
        );
        assert_eq!(pixel_rgb(&rgb, 4, 5), [0, 9, 0]);

        set_pixel(&mut rgb, &mut high_priority_pixels, -1, 5, [7, 7, 7], None);
        set_pixel(
            &mut rgb,
            &mut high_priority_pixels,
            FRAME_WIDTH as i32,
            5,
            [7, 7, 7],
            None,
        );
        assert_eq!(pixel_rgb(&rgb, 4, 5), [0, 9, 0]);
    }

    #[test]
    fn indexed_tile_primitives_report_flips_rows_slices_and_draw_pixels() {
        let tile_sheet = test_tile_sheet();
        let palette = test_palette(16);
        let flipped = TileRef {
            flip_x: true,
            flip_y: true,
            ..test_tile_ref(false)
        };

        let pixel = indexed_tile_pixel_write(
            3,
            "indexed_tile",
            10,
            20,
            &tile_sheet,
            flipped,
            &palette,
            11,
            22,
        )
        .expect("flipped indexed tile pixel should be visible");
        assert_eq!(pixel.source_x, Some(6));
        assert_eq!(pixel.source_y, Some(5));
        assert_eq!(pixel.palette_index, Some(12));
        assert_eq!(pixel.rgb, Some([12, 24, 36]));
        assert!(
            indexed_tile_pixel_write(
                3,
                "indexed_tile",
                10,
                20,
                &tile_sheet,
                flipped,
                &palette,
                18,
                20
            )
            .is_none()
        );

        let row =
            indexed_tile_row_pixel_write(4, 10, 30, &tile_sheet, flipped, &palette, 2, 12, 30)
                .expect("row pixel should use row as source y");
        assert_eq!(row.kind, "indexed_tile_row");
        assert_eq!(row.source_x, Some(5));
        assert_eq!(row.source_y, Some(5));
        assert!(
            indexed_tile_row_pixel_write(4, 10, 30, &tile_sheet, flipped, &palette, 2, 11, 31)
                .is_none()
        );
        assert!(
            indexed_tile_row_pixel_write(4, 10, 30, &tile_sheet, flipped, &palette, 2, 72, 30)
                .is_none()
        );

        let slice = indexed_tile_slice_pixel_write(
            5,
            10,
            40,
            &tile_sheet,
            flipped,
            &palette,
            2,
            1,
            3,
            12,
            42,
        )
        .expect("slice pixel should use clipped column window");
        assert_eq!(slice.kind, "indexed_tile_slice");
        assert_eq!(slice.source_x, Some(5));
        assert_eq!(slice.source_y, Some(5));
        assert!(
            indexed_tile_slice_pixel_write(
                5,
                10,
                40,
                &tile_sheet,
                flipped,
                &palette,
                2,
                1,
                3,
                10,
                42
            )
            .is_none()
        );
        assert!(
            indexed_tile_slice_pixel_write(
                5,
                10,
                40,
                &tile_sheet,
                flipped,
                &palette,
                2,
                1,
                3,
                14,
                42
            )
            .is_none()
        );

        let mut rgb = vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize];
        let mut high_priority_pixels = vec![false; (FRAME_WIDTH * FRAME_HEIGHT) as usize];
        draw_indexed_tile(
            &mut rgb,
            &mut high_priority_pixels,
            20,
            50,
            &tile_sheet,
            flipped,
            &palette,
        );
        assert_eq!(pixel_rgb(&rgb, 21, 52), [12, 24, 36]);

        clear_frame(&mut rgb, [0, 0, 0]);
        assert_eq!(pixel_rgb(&rgb, 21, 52), [0, 0, 0]);
        draw_indexed_tile_row(
            &mut rgb,
            &mut high_priority_pixels,
            20,
            60,
            &tile_sheet,
            flipped,
            &palette,
            2,
        );
        assert_eq!(pixel_rgb(&rgb, 21, 60), [12, 24, 36]);
        assert_eq!(pixel_rgb(&rgb, 21, 62), [0, 0, 0]);

        draw_indexed_tile_slice(
            &mut rgb,
            &mut high_priority_pixels,
            20,
            70,
            &tile_sheet,
            flipped,
            &palette,
            2,
            1,
            3,
        );
        assert_eq!(pixel_rgb(&rgb, 22, 72), [11, 22, 33]);
        assert_eq!(pixel_rgb(&rgb, 24, 72), [0, 0, 0]);
    }

    #[test]
    fn text_pixel_visible_reports_scaled_source_and_newline_rows() {
        let bitmap = BASIC_FONTS.get('A').expect("font has A");
        let (source_y, row_bits) = bitmap
            .iter()
            .enumerate()
            .find(|(_, bits)| **bits != 0)
            .expect("A has visible pixels");
        let source_x = (0..8)
            .find(|column| row_bits & (1 << column) != 0)
            .expect("A row has visible pixel");

        assert_eq!(
            text_pixel_visible(
                10,
                20,
                "A\nB",
                2,
                10 + source_x * 2,
                20 + source_y as i32 * 2
            ),
            Some((source_x as u8, source_y as u8))
        );

        let wide_pixel = ['M', 'W', 'H', 'X', '8']
            .into_iter()
            .find_map(|ch| {
                let bitmap = BASIC_FONTS.get(ch)?;
                for (source_y, bits) in bitmap.iter().enumerate().skip(6) {
                    for source_x in 6..8 {
                        if bits & (1 << source_x) != 0 {
                            return Some((ch, source_x, source_y));
                        }
                    }
                }
                None
            })
            .expect("fixture font has a visible lower-right glyph pixel");
        assert_eq!(
            text_pixel_visible(
                10,
                20,
                &wide_pixel.0.to_string(),
                2,
                10 + wide_pixel.1 * 2,
                20 + wide_pixel.2 as i32 * 2
            ),
            Some((wide_pixel.1 as u8, wide_pixel.2 as u8)),
            "scaled glyph bounds must cover the full 8x8 source, not only 8 + scale pixels"
        );

        let blank_a_row = bitmap
            .iter()
            .position(|bits| *bits == 0)
            .expect("A has at least one blank source row");
        assert_eq!(
            text_pixel_visible(10, 20, "A", 2, 10, 20 + blank_a_row as i32 * 2),
            None,
            "transparent glyph pixels must not be reported as visible"
        );
        assert_eq!(
            text_pixel_visible(
                10,
                20,
                "AB",
                2,
                10 + 16 + source_x * 2,
                20 + source_y as i32 * 2
            ),
            Some((source_x as u8, source_y as u8)),
            "second glyph starts after 8 scaled source columns"
        );
        assert_eq!(
            text_pixel_visible(10, 20, "A", 2, 10 + 16, 20),
            None,
            "the right edge after one scaled glyph is exclusive"
        );

        let bitmap_b = BASIC_FONTS.get('B').expect("font has B");
        let (source_y_b, row_bits_b) = bitmap_b
            .iter()
            .enumerate()
            .find(|(_, bits)| **bits != 0)
            .expect("B has visible pixels");
        let source_x_b = (0..8)
            .find(|column| row_bits_b & (1 << column) != 0)
            .expect("B row has visible pixel");
        assert_eq!(
            text_pixel_visible(
                10,
                20,
                "A\nB",
                2,
                10 + source_x_b * 2,
                20 + 10 * 2 + source_y_b as i32 * 2
            ),
            Some((source_x_b as u8, source_y_b as u8))
        );
        assert_eq!(text_pixel_visible(10, 20, "A", 0, 10, 20), None);
    }

    #[test]
    fn scene_pixel_provenance_tracks_priority_acceptance_and_clear() {
        let tile_sheet = test_tile_sheet();
        let palette = test_palette(16);
        let mut pixels = BTreeSet::new();
        pixels.insert((5, 5));
        pixels.insert((0, 0));
        let scene = Scene {
            background: [1, 1, 1],
            commands: vec![
                DrawCommand::Sprite {
                    x: 5,
                    y: 5,
                    tile_sheet: Arc::clone(&tile_sheet),
                    palette: Arc::clone(&palette),
                    tiles: vec![SpriteTile {
                        offset_x: 0,
                        offset_y: 0,
                        tile: test_tile_ref(true),
                    }],
                },
                DrawCommand::Sprite {
                    x: 5,
                    y: 5,
                    tile_sheet: Arc::clone(&tile_sheet),
                    palette: Arc::clone(&palette),
                    tiles: vec![SpriteTile {
                        offset_x: 0,
                        offset_y: 0,
                        tile: test_tile_ref(false),
                    }],
                },
                DrawCommand::ClearPriorityRect {
                    x: 5,
                    y: 5,
                    width: 1,
                    height: 1,
                },
                DrawCommand::Sprite {
                    x: 5,
                    y: 5,
                    tile_sheet,
                    palette,
                    tiles: vec![SpriteTile {
                        offset_x: 0,
                        offset_y: 0,
                        tile: test_tile_ref(false),
                    }],
                },
            ],
            pause_palette_darken: false,
        };

        let diagnostics = sample_scene_pixel_provenance(&scene, &pixels);
        let target = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.x == 5 && diagnostic.y == 5)
            .expect("target diagnostic exists");
        assert_eq!(target.final_rgb, [1, 2, 3]);
        assert!(!target.final_high_priority);
        assert_eq!(target.writes.len(), 4);
        assert!(target.writes[0].accepted);
        assert!(target.writes[0].priority == Some(true));
        assert_eq!(target.writes[0].palette_colour_count, Some(16));
        assert_eq!(target.writes[0].resolved_palette_slot, Some(1));
        assert_eq!(target.writes[0].raw_palette_word, Some(1));
        assert!(!target.writes[1].accepted);
        assert!(target.writes[1].prior_high_priority);
        assert!(target.writes[2].accepted);
        assert_eq!(target.writes[2].rgb, None);
        assert!(target.writes[3].accepted);

        let background = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.x == 0 && diagnostic.y == 0)
            .expect("background diagnostic exists");
        assert_eq!(background.final_rgb, [1, 1, 1]);
        assert!(background.writes.is_empty());
    }
}
