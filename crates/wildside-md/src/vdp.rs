//! Pure-Rust software VDP renderer: snapshot types, [`render_vdp_into`], and
//! [`sample_vdp_pixel`] plus their plane/window/sprite/tile helpers. Free of
//! the 68k/Z80/ymfm machine emulation in `runtime`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VdpSnapshot {
    pub registers: [u8; 0x20],
    pub vram: Vec<u8>,
    pub cram: Vec<u8>,
    pub vsram: Vec<u8>,
    pub raster_frame: Option<VdpRasterFrame>,
}

/// VRAM state and ordered writes for a frame whose CPU work overlaps display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VdpRasterFrame {
    pub initial_vram: Vec<u8>,
    pub vram_word_writes: Vec<VdpRasterVramWordWrite>,
}

/// One FIFO-committed VRAM word write at the VDP raster position it takes effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VdpRasterVramWordWrite {
    pub scanline: u16,
    pub pixel: u16,
    pub byte_addr: u16,
    pub value: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VdpPixelSample {
    pub x: usize,
    pub y: usize,
    pub layers: Vec<VdpPixelLayerSample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VdpPixelLayerSample {
    pub layer: &'static str,
    pub priority: bool,
    pub map_base: usize,
    pub cell_x: usize,
    pub cell_y: usize,
    pub cell_index: usize,
    pub entry: u16,
    pub tile_index: usize,
    pub palette_line: usize,
    pub colour_index: u8,
    pub source_x: usize,
    pub source_y: usize,
    pub rgb: Option<[u8; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WindowRegion {
    pub(crate) left: usize,
    pub(crate) top: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScrollPlane {
    pub(crate) base: usize,
    pub(crate) width_cells: usize,
    pub(crate) height_cells: usize,
    pub(crate) hscroll: i32,
    pub(crate) vscroll: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WindowPlane {
    pub(crate) base: usize,
    pub(crate) region: Option<WindowRegion>,
}

pub fn render_vdp_into(vdp: &VdpSnapshot, rgb: &mut [u8]) {
    let width = 320usize;
    let height = 224usize;
    let background = cram_rgb(&vdp.cram, usize::from(vdp.registers[7] & 0x3F));
    for pixel in rgb.chunks_exact_mut(3) {
        pixel.copy_from_slice(&background);
    }

    let plane_a_base = (usize::from(vdp.registers[2] & 0x38)) << 10;
    let plane_b_base = (usize::from(vdp.registers[4] & 0x07)) << 13;
    let window_base = window_plane_base(&vdp.registers);
    let sprite_table_base = sprite_table_base(&vdp.registers);
    let hscroll_base = usize::from(vdp.registers[0x0D]) << 10;
    let vscroll_a = i32::from(read_u16_wrapped(&vdp.vsram, 0) as i16);
    let vscroll_b = i32::from(read_u16_wrapped(&vdp.vsram, 2) as i16);
    let (plane_width_cells, plane_height_cells) = plane_dimensions(vdp.registers[0x10]);
    let window_region = window_region(&vdp.registers, width, height);
    let raster_frame = vdp.raster_frame.as_ref();
    let mut raster_vdp = raster_frame.map(|raster_frame| {
        let mut raster_vdp = vdp.clone();
        raster_vdp.vram.clone_from(&raster_frame.initial_vram);
        raster_vdp.raster_frame = None;
        raster_vdp
    });
    let initial_vram = raster_vdp
        .as_ref()
        .map_or(vdp.vram.as_slice(), |raster_vdp| raster_vdp.vram.as_slice());
    let sprite_scan = SpriteLineScan::build(initial_vram, sprite_table_base, height, width);
    let mut raster_write_index = 0usize;

    for y in 0..height {
        if let (Some(raster_frame), Some(raster_vdp)) = (raster_frame, raster_vdp.as_mut()) {
            apply_vram_raster_writes_before_scanline(
                raster_frame,
                raster_vdp,
                &mut raster_write_index,
                y,
            );
        }
        let plane_a = ScrollPlane {
            base: plane_a_base,
            width_cells: plane_width_cells,
            height_cells: plane_height_cells,
            hscroll: hscroll_for_line(
                raster_vdp
                    .as_ref()
                    .map_or(vdp.vram.as_slice(), |vdp| vdp.vram.as_slice()),
                hscroll_base,
                y,
                0,
                vdp.registers[0x0B],
            ),
            vscroll: vscroll_a,
        };
        let plane_b = ScrollPlane {
            base: plane_b_base,
            width_cells: plane_width_cells,
            height_cells: plane_height_cells,
            hscroll: hscroll_for_line(
                raster_vdp
                    .as_ref()
                    .map_or(vdp.vram.as_slice(), |vdp| vdp.vram.as_slice()),
                hscroll_base,
                y,
                2,
                vdp.registers[0x0B],
            ),
            vscroll: vscroll_b,
        };
        let window_plane = WindowPlane {
            base: window_base,
            region: window_region,
        };
        let line_sprites = sprite_scan.sprites_on_line(y);
        for x in 0..width {
            let presented_vdp = raster_vdp.as_ref().unwrap_or(vdp);
            for priority in [false, true] {
                if let Some(colour) = plane_pixel_rgb(presented_vdp, &plane_b, x, y, priority) {
                    write_rgb(rgb, width, x, y, colour);
                }
                if let Some(colour) = plane_a_or_window_pixel_rgb(
                    presented_vdp,
                    &plane_a,
                    &window_plane,
                    x,
                    y,
                    priority,
                ) {
                    write_rgb(rgb, width, x, y, colour);
                }
                if let Some(colour) = scanline_sprite_pixel_rgb(
                    presented_vdp,
                    &sprite_scan,
                    line_sprites,
                    x,
                    y,
                    priority,
                ) {
                    write_rgb(rgb, width, x, y, colour);
                }
            }
        }
    }
}

fn apply_vram_raster_writes_before_scanline(
    raster_frame: &VdpRasterFrame,
    vdp: &mut VdpSnapshot,
    write_index: &mut usize,
    scanline: usize,
) {
    while let Some(write) = raster_frame.vram_word_writes.get(*write_index)
        // The VDP background pipeline latches the next displayed row before
        // that scanline begins. A FIFO write during N can affect N+1, never a
        // partial span of background pixels within N.
        && usize::from(write.scanline) < scanline
    {
        let byte_addr = usize::from(write.byte_addr);
        if let Some(slot) = vdp.vram.get_mut(byte_addr..byte_addr + 2) {
            slot.copy_from_slice(&write.value.to_be_bytes());
        }
        *write_index += 1;
    }
}

pub fn sample_vdp_pixel(vdp: &VdpSnapshot, x: usize, y: usize) -> VdpPixelSample {
    let plane_a_base = (usize::from(vdp.registers[2] & 0x38)) << 10;
    let plane_b_base = (usize::from(vdp.registers[4] & 0x07)) << 13;
    let window_base = window_plane_base(&vdp.registers);
    let hscroll_base = usize::from(vdp.registers[0x0D]) << 10;
    let vscroll_a = i32::from(read_u16_wrapped(&vdp.vsram, 0) as i16);
    let vscroll_b = i32::from(read_u16_wrapped(&vdp.vsram, 2) as i16);
    let (plane_width_cells, plane_height_cells) = plane_dimensions(vdp.registers[0x10]);
    let window_region = window_region(&vdp.registers, 320, 224);

    let plane_a = ScrollPlane {
        base: plane_a_base,
        width_cells: plane_width_cells,
        height_cells: plane_height_cells,
        hscroll: hscroll_for_line(&vdp.vram, hscroll_base, y, 0, vdp.registers[0x0B]),
        vscroll: vscroll_a,
    };
    let plane_b = ScrollPlane {
        base: plane_b_base,
        width_cells: plane_width_cells,
        height_cells: plane_height_cells,
        hscroll: hscroll_for_line(&vdp.vram, hscroll_base, y, 2, vdp.registers[0x0B]),
        vscroll: vscroll_b,
    };
    let window_plane = WindowPlane {
        base: window_base,
        region: window_region,
    };

    let mut layers = Vec::new();
    for priority in [false, true] {
        if let Some(sample) = sample_plane_pixel(vdp, "plane_b", &plane_b, x, y, priority) {
            layers.push(sample);
        }
        if let Some(sample) =
            sample_plane_a_or_window_pixel(vdp, &plane_a, &window_plane, x, y, priority)
        {
            layers.push(sample);
        }
    }

    VdpPixelSample { x, y, layers }
}

pub(crate) fn plane_dimensions(size_register: u8) -> (usize, usize) {
    let width_cells = match size_register & 0x03 {
        0x01 => 64,
        0x03 => 128,
        _ => 32,
    };
    let height_cells = match (size_register >> 4) & 0x03 {
        0x01 => 64,
        0x03 => 128,
        _ => 32,
    };
    (width_cells, height_cells)
}

pub(crate) fn hscroll_for_line(
    vram: &[u8],
    base: usize,
    line: usize,
    plane_offset: usize,
    mode_register: u8,
) -> i32 {
    let offset = match mode_register & 0x03 {
        0x00 => base + plane_offset,
        0x01 => base + 4 * (line & 0x07) + plane_offset,
        0x02 => base + 32 * (line / 8) + plane_offset,
        0x03 => base + 4 * line + plane_offset,
        _ => unreachable!("mode register is masked to two bits"),
    };
    i32::from(read_u16_wrapped(vram, offset) & 0x03FF)
}

#[inline]
pub(crate) fn plane_pixel_rgb(
    vdp: &VdpSnapshot,
    plane: &ScrollPlane,
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<[u8; 3]> {
    let plane_width_pixels = (plane.width_cells * 8) as i32;
    let plane_height_pixels = (plane.height_cells * 8) as i32;
    let world_x = (screen_x as i32 - plane.hscroll).rem_euclid(plane_width_pixels);
    let world_y = (screen_y as i32 + plane.vscroll).rem_euclid(plane_height_pixels);
    let cell_x = (world_x as usize) / 8;
    let cell_y = (world_y as usize) / 8;
    let cell_index = cell_y * plane.width_cells + cell_x;
    let entry = read_u16_wrapped(&vdp.vram, plane.base + cell_index * 2);
    if (entry & 0x8000 != 0) != priority {
        return None;
    }
    tile_pixel_rgb(vdp, entry, (world_x as usize) & 7, (world_y as usize) & 7)
}

pub(crate) fn sample_plane_pixel(
    vdp: &VdpSnapshot,
    layer: &'static str,
    plane: &ScrollPlane,
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<VdpPixelLayerSample> {
    let plane_width_pixels = (plane.width_cells * 8) as i32;
    let plane_height_pixels = (plane.height_cells * 8) as i32;
    let world_x = (screen_x as i32 - plane.hscroll).rem_euclid(plane_width_pixels);
    let world_y = (screen_y as i32 + plane.vscroll).rem_euclid(plane_height_pixels);
    let cell_x = (world_x as usize) / 8;
    let cell_y = (world_y as usize) / 8;
    let cell_index = cell_y * plane.width_cells + cell_x;
    let entry = read_u16_wrapped(&vdp.vram, plane.base + cell_index * 2);
    if (entry & 0x8000 != 0) != priority {
        return None;
    }

    Some(tile_pixel_sample(
        vdp,
        layer,
        priority,
        plane.base,
        cell_x,
        cell_y,
        cell_index,
        entry,
        (world_x as usize) & 7,
        (world_y as usize) & 7,
    ))
}

pub(crate) fn plane_a_or_window_pixel_rgb(
    vdp: &VdpSnapshot,
    plane_a: &ScrollPlane,
    window: &WindowPlane,
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<[u8; 3]> {
    if let Some(region) = window.region
        && screen_x >= region.left
        && screen_x < region.left + region.width
        && screen_y >= region.top
        && screen_y < region.top + region.height
    {
        let window_x = screen_x - region.left;
        let window_y = screen_y - region.top;
        return window_pixel_rgb(vdp, window.base, window_x, window_y, priority);
    }

    plane_pixel_rgb(vdp, plane_a, screen_x, screen_y, priority)
}

pub(crate) fn sample_plane_a_or_window_pixel(
    vdp: &VdpSnapshot,
    plane_a: &ScrollPlane,
    window: &WindowPlane,
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<VdpPixelLayerSample> {
    if let Some(region) = window.region
        && screen_x >= region.left
        && screen_x < region.left + region.width
        && screen_y >= region.top
        && screen_y < region.top + region.height
    {
        let window_x = screen_x - region.left;
        let window_y = screen_y - region.top;
        return sample_window_pixel(vdp, window.base, window_x, window_y, priority);
    }

    sample_plane_pixel(vdp, "plane_a", plane_a, screen_x, screen_y, priority)
}

pub(crate) fn window_pixel_rgb(
    vdp: &VdpSnapshot,
    window_base: usize,
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<[u8; 3]> {
    let map_width_cells = if is_h40_mode(vdp.registers[0x0C]) {
        64
    } else {
        32
    };
    let cell_x = screen_x / 8;
    let cell_y = screen_y / 8;
    let cell_index = cell_y * map_width_cells + cell_x;
    let entry = read_u16_wrapped(&vdp.vram, window_base + cell_index * 2);
    if (entry & 0x8000 != 0) != priority {
        return None;
    }

    tile_pixel_rgb(vdp, entry, screen_x & 7, screen_y & 7)
}

pub(crate) fn sample_window_pixel(
    vdp: &VdpSnapshot,
    window_base: usize,
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<VdpPixelLayerSample> {
    let map_width_cells = if is_h40_mode(vdp.registers[0x0C]) {
        64
    } else {
        32
    };
    let cell_x = screen_x / 8;
    let cell_y = screen_y / 8;
    let cell_index = cell_y * map_width_cells + cell_x;
    let entry = read_u16_wrapped(&vdp.vram, window_base + cell_index * 2);
    if (entry & 0x8000 != 0) != priority {
        return None;
    }

    Some(tile_pixel_sample(
        vdp,
        "window",
        priority,
        window_base,
        cell_x,
        cell_y,
        cell_index,
        entry,
        screen_x & 7,
        screen_y & 7,
    ))
}

pub(crate) const MAX_SPRITES: usize = 80;
// H40 mode allows 20 sprites per scanline; H32 allows 16. Use 20 as the upper bound.
pub(crate) const MAX_SPRITES_PER_LINE: usize = 20;

pub(crate) struct SpriteEntry {
    pub(crate) xpos: i32,
    pub(crate) ypos: i32,
    pub(crate) width_pixels: i32,
    pub(crate) height_pixels: i32,
    pub(crate) width_tiles: usize,
    pub(crate) height_tiles: usize,
    pub(crate) pattern: u16,
}

pub(crate) struct SpriteLineScan {
    pub(crate) entries: Vec<SpriteEntry>,
    // Per-scanline: indices into `entries` for sprites visible on that line.
    pub(crate) line_indices: Vec<Vec<u8>>,
}

impl SpriteLineScan {
    pub(crate) fn build(
        vram: &[u8],
        sprite_table_base: usize,
        screen_height: usize,
        sprite_pixels_per_line: usize,
    ) -> Self {
        let mut entries = Vec::with_capacity(MAX_SPRITES);
        let mut line_indices = vec![Vec::new(); screen_height];
        let mut line_sprite_pixels = vec![0usize; screen_height];
        let mut sprite_index = 0usize;
        let mut visited = [false; MAX_SPRITES];

        for _ in 0..MAX_SPRITES {
            if sprite_index >= MAX_SPRITES || visited[sprite_index] {
                break;
            }
            visited[sprite_index] = true;

            let entry_base = sprite_table_base + sprite_index * 8;
            let ypos = i32::from(read_u16_wrapped(vram, entry_base)) - 128;
            let size_link = read_u16_wrapped(vram, entry_base + 2);
            let pattern = read_u16_wrapped(vram, entry_base + 4);
            let xpos = i32::from(read_u16_wrapped(vram, entry_base + 6)) - 128;
            let next_link = usize::from(size_link & 0x7F);
            let width_tiles = usize::from(((size_link >> 10) & 0x03) + 1);
            let height_tiles = usize::from(((size_link >> 8) & 0x03) + 1);
            let width_pixels = (width_tiles * 8) as i32;
            let height_pixels = (height_tiles * 8) as i32;

            let entry_idx = entries.len() as u8;
            entries.push(SpriteEntry {
                xpos,
                ypos,
                width_pixels,
                height_pixels,
                width_tiles,
                height_tiles,
                pattern,
            });

            let top = ypos.max(0) as usize;
            let bottom = ((ypos + height_pixels) as usize).min(screen_height);
            for (line, sprite_pixels) in line_indices[top.min(bottom)..bottom]
                .iter_mut()
                .zip(&mut line_sprite_pixels[top.min(bottom)..bottom])
            {
                if line.len() < MAX_SPRITES_PER_LINE
                    && sprite_pixels.saturating_add(width_pixels as usize) <= sprite_pixels_per_line
                {
                    line.push(entry_idx);
                    *sprite_pixels += width_pixels as usize;
                }
            }

            if next_link == 0 {
                break;
            }
            sprite_index = next_link;
        }

        Self {
            entries,
            line_indices,
        }
    }

    pub(crate) fn sprites_on_line(&self, y: usize) -> &[u8] {
        &self.line_indices[y]
    }
}

pub(crate) fn scanline_sprite_pixel_rgb(
    vdp: &VdpSnapshot,
    scan: &SpriteLineScan,
    line_sprites: &[u8],
    screen_x: usize,
    screen_y: usize,
    priority: bool,
) -> Option<[u8; 3]> {
    let screen_x_i = screen_x as i32;
    let screen_y_i = screen_y as i32;
    for &entry_idx in line_sprites {
        let entry = &scan.entries[entry_idx as usize];
        if ((entry.pattern & 0x8000) != 0) == priority
            && screen_x_i >= entry.xpos
            && screen_x_i < entry.xpos + entry.width_pixels
            && screen_y_i >= entry.ypos
            && screen_y_i < entry.ypos + entry.height_pixels
        {
            let local_x = (screen_x_i - entry.xpos) as usize;
            let local_y = (screen_y_i - entry.ypos) as usize;
            if let Some(colour) = sprite_tile_pixel_rgb(
                vdp,
                entry.pattern,
                entry.width_tiles,
                entry.height_tiles,
                local_x,
                local_y,
            ) {
                return Some(colour);
            }
        }
    }
    None
}

pub(crate) fn sprite_tile_pixel_rgb(
    vdp: &VdpSnapshot,
    pattern: u16,
    width_tiles: usize,
    height_tiles: usize,
    local_x: usize,
    local_y: usize,
) -> Option<[u8; 3]> {
    let hflip = pattern & 0x0800 != 0;
    let vflip = pattern & 0x1000 != 0;
    let palette = usize::from((pattern >> 13) & 0x03);
    let base_tile = usize::from(pattern & 0x07FF);
    let local_x = if hflip {
        width_tiles * 8 - 1 - local_x
    } else {
        local_x
    };
    let local_y = if vflip {
        height_tiles * 8 - 1 - local_y
    } else {
        local_y
    };
    let tile_col = local_x / 8;
    let tile_row = local_y / 8;
    let tile_index = base_tile + tile_col * height_tiles + tile_row;
    let pixel_x = local_x & 7;
    let pixel_y = local_y & 7;
    tile_pixel_rgb_from_parts(vdp, tile_index, palette, pixel_x, pixel_y)
}

#[inline]
pub(crate) fn tile_pixel_rgb(
    vdp: &VdpSnapshot,
    entry: u16,
    tile_x: usize,
    tile_y: usize,
) -> Option<[u8; 3]> {
    let tile_index = usize::from(entry & 0x07FF);
    let hflip = entry & 0x0800 != 0;
    let vflip = entry & 0x1000 != 0;
    let palette = usize::from((entry >> 13) & 0x03);
    let mut tile_x = tile_x & 7;
    let mut tile_y = tile_y & 7;
    if hflip {
        tile_x = 7 - tile_x;
    }
    if vflip {
        tile_y = 7 - tile_y;
    }
    tile_pixel_rgb_from_parts(vdp, tile_index, palette, tile_x, tile_y)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn tile_pixel_sample(
    vdp: &VdpSnapshot,
    layer: &'static str,
    priority: bool,
    map_base: usize,
    cell_x: usize,
    cell_y: usize,
    cell_index: usize,
    entry: u16,
    tile_x: usize,
    tile_y: usize,
) -> VdpPixelLayerSample {
    let tile_index = usize::from(entry & 0x07FF);
    let hflip = entry & 0x0800 != 0;
    let vflip = entry & 0x1000 != 0;
    let palette_line = usize::from((entry >> 13) & 0x03);
    let mut source_x = tile_x & 7;
    let mut source_y = tile_y & 7;
    if hflip {
        source_x = 7 - source_x;
    }
    if vflip {
        source_y = 7 - source_y;
    }
    let tile_byte = read_u8_wrapped(&vdp.vram, tile_index * 32 + source_y * 4 + source_x / 2);
    let colour_index = if source_x & 1 == 0 {
        tile_byte >> 4
    } else {
        tile_byte & 0x0F
    };
    let rgb = if colour_index == 0 {
        None
    } else {
        Some(cram_rgb(
            &vdp.cram,
            palette_line * 16 + usize::from(colour_index),
        ))
    };

    VdpPixelLayerSample {
        layer,
        priority,
        map_base,
        cell_x,
        cell_y,
        cell_index,
        entry,
        tile_index,
        palette_line,
        colour_index,
        source_x,
        source_y,
        rgb,
    }
}

#[inline]
pub(crate) fn tile_pixel_rgb_from_parts(
    vdp: &VdpSnapshot,
    tile_index: usize,
    palette: usize,
    tile_x: usize,
    tile_y: usize,
) -> Option<[u8; 3]> {
    let tile_byte = read_u8_wrapped(&vdp.vram, tile_index * 32 + tile_y * 4 + tile_x / 2);
    let colour_index = if tile_x & 1 == 0 {
        tile_byte >> 4
    } else {
        tile_byte & 0x0F
    };
    if colour_index == 0 {
        None
    } else {
        Some(cram_rgb(
            &vdp.cram,
            palette * 16 + usize::from(colour_index),
        ))
    }
}

pub(crate) fn window_plane_base(registers: &[u8; 0x20]) -> usize {
    let mask = if is_h40_mode(registers[0x0C]) {
        0x3E
    } else {
        0x3C
    };
    (usize::from(registers[3] & mask)) << 10
}

pub(crate) fn sprite_table_base(registers: &[u8; 0x20]) -> usize {
    let mask = if is_h40_mode(registers[0x0C]) {
        0x7E
    } else {
        0x7F
    };
    (usize::from(registers[5] & mask)) << 9
}

pub(crate) fn window_region(
    registers: &[u8; 0x20],
    width: usize,
    height: usize,
) -> Option<WindowRegion> {
    let horizontal_pos = usize::from(registers[0x11] & 0x1F) * 16;
    let vertical_pos = usize::from(registers[0x12] & 0x1F) * 8;
    let (left, window_width) = if registers[0x11] & 0x80 == 0 {
        (0, horizontal_pos.min(width))
    } else {
        let left = horizontal_pos.min(width);
        (left, width.saturating_sub(left))
    };
    let (top, window_height) = if registers[0x12] & 0x80 == 0 {
        (0, vertical_pos.min(height))
    } else {
        let top = vertical_pos.min(height);
        (top, height.saturating_sub(top))
    };
    if window_width == 0 || window_height == 0 {
        None
    } else {
        Some(WindowRegion {
            left,
            top,
            width: window_width,
            height: window_height,
        })
    }
}

pub(crate) fn is_h40_mode(mode_register: u8) -> bool {
    mode_register & 0x81 == 0x81
}

#[inline]
pub(crate) fn cram_rgb(cram: &[u8], palette_index: usize) -> [u8; 3] {
    let colour = read_u16_wrapped(cram, palette_index * 2);
    let red = ((colour >> 1) & 0x7) as u8;
    let green = ((colour >> 5) & 0x7) as u8;
    let blue = ((colour >> 9) & 0x7) as u8;
    [
        scale_channel(red),
        scale_channel(green),
        scale_channel(blue),
    ]
}

#[inline]
pub(crate) fn scale_channel(value: u8) -> u8 {
    ((u16::from(value) * 255 + 3) / 7) as u8
}

#[inline]
pub(crate) fn write_rgb(rgb: &mut [u8], width: usize, x: usize, y: usize, colour: [u8; 3]) {
    let index = (y * width + x) * 3;
    rgb[index..index + 3].copy_from_slice(&colour);
}

#[inline]
pub(crate) fn read_u8_wrapped(slice: &[u8], index: usize) -> u8 {
    if slice.is_empty() {
        0
    } else {
        slice[index % slice.len()]
    }
}

#[inline]
pub(crate) fn read_u16_wrapped(slice: &[u8], index: usize) -> u16 {
    u16::from_be_bytes([
        read_u8_wrapped(slice, index),
        read_u8_wrapped(slice, index + 1),
    ])
}
