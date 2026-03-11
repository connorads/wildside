use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundVisuals {
    pub placements: Vec<BackgroundPlacement>,
    pub chunks: Vec<BackgroundChunk>,
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
