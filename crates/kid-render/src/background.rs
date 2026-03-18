use crate::tiles::TileRef;
use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundVisuals {
    Chunked(ChunkedBackgroundVisuals),
    Layered(LayeredBackgroundVisuals),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkedBackgroundVisuals {
    pub placements: Vec<BackgroundPlacement>,
    pub chunks: Vec<BackgroundChunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayeredBackgroundVisuals {
    pub effect_start_row_selector: Option<u8>,
    pub rows: Vec<LayeredBackgroundRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayeredBackgroundRow {
    pub selector: u8,
    pub tiles: Vec<TileRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayeredBackgroundLayout {
    pub effect_start_row_selector: Option<u8>,
    pub row_selectors: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayeredBackgroundPlane {
    pub rows: Vec<Vec<TileRef>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundPlacement {
    pub chunk_id: u16,
    pub x_tiles: u16,
    pub y_tiles: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundChunk {
    pub width_tiles: u16,
    pub height_tiles: u16,
    pub tiles: Vec<BackgroundTile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundTile {
    pub tile_index: u16,
    pub flip_x: bool,
}

const LAYERED_BACKGROUND_PLANE_WIDTH_TILES: usize = 32;

pub fn parse_chunked_background_layout(bytes: &[u8]) -> Result<Vec<BackgroundPlacement>> {
    let mut placements = Vec::new();
    let mut cursor = 0usize;

    loop {
        if cursor + 2 > bytes.len() {
            bail!("background layout is missing the 0xFFFF terminator");
        }

        let chunk_id = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]);
        if chunk_id == 0xFFFF {
            return Ok(placements);
        }
        if chunk_id == 0x8000 {
            bail!("shared background layout pointers are not supported");
        }
        if cursor + 6 > bytes.len() {
            bail!("background layout entry for chunk 0x{chunk_id:04X} is truncated");
        }

        placements.push(BackgroundPlacement {
            chunk_id,
            x_tiles: u16::from_be_bytes([bytes[cursor + 2], bytes[cursor + 3]]),
            y_tiles: u16::from_be_bytes([bytes[cursor + 4], bytes[cursor + 5]]),
        });
        cursor += 6;
    }
}

pub fn parse_background_chunk(bytes: &[u8]) -> Result<BackgroundChunk> {
    if bytes.len() < 4 {
        bail!(
            "background chunk is too short: expected at least 4 bytes, got {}",
            bytes.len()
        );
    }

    let width_tiles = u16::from_be_bytes([bytes[0], bytes[1]]);
    let height_tiles = u16::from_be_bytes([bytes[2], bytes[3]]);
    let tile_count = usize::from(width_tiles) * usize::from(height_tiles);
    let expected_len = 4 + tile_count;
    if bytes.len() != expected_len {
        bail!(
            "background chunk payload length mismatch: expected {expected_len} bytes, got {}",
            bytes.len()
        );
    }

    Ok(BackgroundChunk {
        width_tiles,
        height_tiles,
        tiles: bytes[4..]
            .iter()
            .map(|byte| BackgroundTile {
                tile_index: u16::from(byte & 0x7F),
                flip_x: byte & 0x80 != 0,
            })
            .collect(),
    })
}

pub fn parse_layered_background_layout(
    bytes: &[u8],
    theme_name: &str,
) -> Result<LayeredBackgroundLayout> {
    let prefix_len = layered_background_prefix_len(theme_name)?;
    if bytes.len() < prefix_len {
        bail!(
            "layered background layout for theme {theme_name} is too short: expected at least {prefix_len} bytes, got {}",
            bytes.len()
        );
    }

    let effect_start_row_selector = match theme_name {
        "forest" | "desert" => bytes.first().copied(),
        _ => None,
    };

    Ok(LayeredBackgroundLayout {
        effect_start_row_selector,
        row_selectors: bytes[prefix_len..].to_vec(),
    })
}

pub fn parse_layered_background_plane(words: &[u16]) -> Result<LayeredBackgroundPlane> {
    if words.is_empty() {
        bail!("layered background plane is empty");
    }
    if !words.len().is_multiple_of(LAYERED_BACKGROUND_PLANE_WIDTH_TILES) {
        bail!(
            "layered background plane width mismatch: expected a multiple of {LAYERED_BACKGROUND_PLANE_WIDTH_TILES} tiles, got {}",
            words.len()
        );
    }

    Ok(LayeredBackgroundPlane {
        rows: words
            .chunks_exact(LAYERED_BACKGROUND_PLANE_WIDTH_TILES)
            .map(|row| row.iter().copied().map(TileRef::from_attr_word).collect())
            .collect(),
    })
}

pub fn compose_layered_background(
    layout: &LayeredBackgroundLayout,
    planes: &[LayeredBackgroundPlane],
) -> Result<LayeredBackgroundVisuals> {
    if planes.len() < 2 {
        bail!(
            "layered background needs at least two decoded planes, got {}",
            planes.len()
        );
    }

    let base_plane_row_count = planes[0].rows.len();
    if base_plane_row_count == 0 {
        bail!("layered background base plane is empty");
    }
    if planes[1].rows.len() != base_plane_row_count {
        bail!(
            "layered background base planes disagree on row count: {} vs {}",
            base_plane_row_count,
            planes[1].rows.len()
        );
    }
    // Extra planes (index ≥ 2) may have different row counts — that's fine,
    // the selector overflow logic uses each plane's own row count.

    let mut rows = Vec::with_capacity(layout.row_selectors.len());
    for &selector in &layout.row_selectors {
        let selector_index = usize::from(selector);
        let tiles = if selector_index < base_plane_row_count {
            let mut tiles = planes[0].rows[selector_index].clone();
            tiles.extend_from_slice(&planes[1].rows[selector_index]);
            tiles
        } else {
            // Overflow selectors index into extra planes sequentially.
            // Walk through planes 2.. using each plane's own row count.
            let mut remaining = selector_index - base_plane_row_count;
            let mut source_plane_index = 2;
            while source_plane_index < planes.len()
                && remaining >= planes[source_plane_index].rows.len()
            {
                remaining -= planes[source_plane_index].rows.len();
                source_plane_index += 1;
            }
            // If all planes are exhausted, clamp to the last row of the last
            // plane.  The original game reads garbage work-RAM here (desert
            // bgscroll files have selectors up to 128 that exceed the 3×28-row
            // plane space); clamping to the last valid row is the closest
            // faithful approximation.
            let (source_plane, remaining) = if source_plane_index >= planes.len()
                || remaining >= planes[source_plane_index].rows.len()
            {
                let last = planes.last().expect("at least one plane");
                (last, last.rows.len() - 1)
            } else {
                (&planes[source_plane_index], remaining)
            };
            let mut tiles = source_plane.rows[remaining].clone();
            // Duplicate for double-width scroll
            tiles.extend_from_slice(&source_plane.rows[remaining]);
            tiles
        };
        rows.push(LayeredBackgroundRow { selector, tiles });
    }

    Ok(LayeredBackgroundVisuals {
        effect_start_row_selector: layout.effect_start_row_selector,
        rows,
    })
}

fn layered_background_prefix_len(theme_name: &str) -> Result<usize> {
    match theme_name {
        "forest" | "hill" | "mountain" => Ok(1),
        "desert" => Ok(2),
        _ => bail!("theme {theme_name} does not use layered backgrounds"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LayeredBackgroundLayout, LayeredBackgroundPlane, compose_layered_background,
        parse_layered_background_layout, parse_layered_background_plane,
    };
    use crate::tiles::TileRef;

    #[test]
    fn parses_forest_layered_background_layout() {
        let layout = parse_layered_background_layout(&[0x11, 0x13, 0x14, 0x35], "forest")
            .expect("forest layout");

        assert_eq!(layout.effect_start_row_selector, Some(0x11));
        assert_eq!(layout.row_selectors, vec![0x13, 0x14, 0x35]);
    }

    #[test]
    fn parses_desert_layered_background_layout_with_unused_second_byte() {
        let layout = parse_layered_background_layout(&[0x09, 0x00, 0x1C, 0x1D], "desert")
            .expect("desert layout");

        assert_eq!(layout.effect_start_row_selector, Some(0x09));
        assert_eq!(layout.row_selectors, vec![0x1C, 0x1D]);
    }

    #[test]
    fn parses_layered_background_plane_rows_from_attr_words() {
        let plane = parse_layered_background_plane(&(0..64_u16).collect::<Vec<_>>())
            .expect("plane should parse");

        assert_eq!(plane.rows.len(), 2);
        assert_eq!(plane.rows[0].len(), 32);
        assert_eq!(plane.rows[1].len(), 32);
        assert_eq!(plane.rows[1][0].tile_index, 32);
    }

    #[test]
    fn composes_layered_background_rows_from_base_and_overflow_planes() {
        let layout = LayeredBackgroundLayout {
            effect_start_row_selector: Some(0x11),
            row_selectors: vec![0x00, 0x01, 0x02],
        };
        let planes = vec![
            LayeredBackgroundPlane {
                rows: vec![
                    vec![TileRef::from_attr_word(0x0001); 32],
                    vec![TileRef::from_attr_word(0x0002); 32],
                ],
            },
            LayeredBackgroundPlane {
                rows: vec![
                    vec![TileRef::from_attr_word(0x0011); 32],
                    vec![TileRef::from_attr_word(0x0012); 32],
                ],
            },
            LayeredBackgroundPlane {
                rows: vec![
                    vec![TileRef::from_attr_word(0x0021); 32],
                    vec![TileRef::from_attr_word(0x0022); 32],
                ],
            },
        ];

        let visuals = compose_layered_background(&layout, &planes).expect("layered background");

        assert_eq!(visuals.effect_start_row_selector, Some(0x11));
        assert_eq!(visuals.rows.len(), 3);
        assert_eq!(visuals.rows[0].tiles.len(), 64);
        assert_eq!(visuals.rows[0].tiles[0].tile_index, 1);
        assert_eq!(visuals.rows[0].tiles[32].tile_index, 17);
        assert_eq!(visuals.rows[2].tiles[0].tile_index, 33);
        assert_eq!(visuals.rows[2].tiles[32].tile_index, 33);
    }
}
