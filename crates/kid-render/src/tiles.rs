use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileSheet {
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub pixels: [u8; 64],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileRef {
    pub tile_index: u16,
    pub palette_line: u8,
    pub priority: bool,
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileQuad {
    pub tiles: [TileRef; 4],
}

impl TileSheet {
    pub fn from_4bpp_bytes(bytes: &[u8]) -> Result<Self> {
        if !bytes.len().is_multiple_of(32) {
            bail!(
                "4bpp tile sheet length {} is not a multiple of 32 bytes",
                bytes.len()
            );
        }
        let tiles = bytes
            .chunks_exact(32)
            .map(Tile::from_4bpp_bytes)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { tiles })
    }

    pub fn apply_pixel_permutation(&mut self, table: &[u8; 16]) {
        for tile in &mut self.tiles {
            for pixel in &mut tile.pixels {
                *pixel = table[usize::from(*pixel)];
            }
        }
    }

    pub fn tile_size(&self) -> (u8, u8) {
        (8, 8)
    }
}

impl Tile {
    fn from_4bpp_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            bail!("expected 32 bytes per tile, got {}", bytes.len());
        }
        let mut pixels = [0_u8; 64];
        for (row, row_bytes) in bytes.chunks_exact(4).enumerate() {
            for (column_group, byte) in row_bytes.iter().enumerate() {
                let pixel_index = row * 8 + column_group * 2;
                pixels[pixel_index] = byte >> 4;
                pixels[pixel_index + 1] = byte & 0x0F;
            }
        }
        Ok(Self { pixels })
    }
}

impl TileRef {
    pub fn from_attr_word(word: u16) -> Self {
        Self {
            tile_index: word & 0x07FF,
            palette_line: ((word >> 13) & 0x03) as u8,
            priority: word & 0x8000 != 0,
            flip_y: word & 0x1000 != 0,
            flip_x: word & 0x0800 != 0,
        }
    }
}

pub fn parse_tile_quads(bytes: &[u8]) -> Result<Vec<TileQuad>> {
    if !bytes.len().is_multiple_of(8) {
        bail!(
            "tile quad mapping length {} is not a multiple of 8 bytes",
            bytes.len()
        );
    }
    Ok(bytes
        .chunks_exact(8)
        .map(|chunk| TileQuad {
            tiles: [
                TileRef::from_attr_word(u16::from_be_bytes([chunk[0], chunk[1]])),
                TileRef::from_attr_word(u16::from_be_bytes([chunk[2], chunk[3]])),
                TileRef::from_attr_word(u16::from_be_bytes([chunk[4], chunk[5]])),
                TileRef::from_attr_word(u16::from_be_bytes([chunk[6], chunk[7]])),
            ],
        })
        .collect())
}
