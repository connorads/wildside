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

#[cfg(test)]
mod tests {
    use super::{TileRef, TileSheet, parse_tile_quads};

    #[test]
    fn decodes_mega_drive_4bpp_tile_nibbles_in_row_order() {
        let mut bytes = [0_u8; 32];
        bytes[0..4].copy_from_slice(&[0x01, 0x23, 0x45, 0x67]);
        bytes[28..32].copy_from_slice(&[0x89, 0xAB, 0xCD, 0xEF]);

        let sheet = TileSheet::from_4bpp_bytes(&bytes).expect("tile sheet");

        assert_eq!(sheet.tiles.len(), 1);
        assert_eq!(
            &sheet.tiles[0].pixels[0..8],
            &[0, 1, 2, 3, 4, 5, 6, 7],
            "Mega Drive 4bpp art stores two pixels per byte, high nibble first"
        );
        assert_eq!(
            &sheet.tiles[0].pixels[56..64],
            &[8, 9, 10, 11, 12, 13, 14, 15],
            "tile rows remain row-major across the full 8x8 tile"
        );
    }

    #[test]
    fn applies_pixel_permutation_to_every_tile_pixel() {
        let mut sheet = TileSheet::from_4bpp_bytes(&[0x01; 32]).expect("tile sheet");
        let table = [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0];

        sheet.apply_pixel_permutation(&table);

        assert_eq!(sheet.tiles[0].pixels[0], 15);
        assert_eq!(sheet.tiles[0].pixels[1], 14);
        assert_eq!(sheet.tiles[0].pixels[62], 15);
        assert_eq!(sheet.tiles[0].pixels[63], 14);
    }

    #[test]
    fn decodes_vdp_tile_attribute_word_fields() {
        let tile = TileRef::from_attr_word(0xFABC);

        assert_eq!(tile.tile_index, 0x02BC);
        assert_eq!(tile.palette_line, 3);
        assert!(tile.priority);
        assert!(tile.flip_y);
        assert!(tile.flip_x);
    }

    #[test]
    fn masks_vdp_tile_attribute_bits_independently() {
        let tile = TileRef::from_attr_word(0x6801);

        assert_eq!(tile.tile_index, 0x0001);
        assert_eq!(tile.palette_line, 3);
        assert!(!tile.priority);
        assert!(!tile.flip_y);
        assert!(tile.flip_x);
    }

    #[test]
    fn vdp_tile_attribute_flip_x_uses_only_bit_eleven() {
        let tile = TileRef::from_attr_word(0xF7FF);

        assert_eq!(tile.tile_index, 0x07FF);
        assert_eq!(tile.palette_line, 3);
        assert!(tile.priority);
        assert!(tile.flip_y);
        assert!(!tile.flip_x);
    }

    #[test]
    fn parses_tile_quads_as_four_big_endian_vdp_attribute_words() {
        let quads =
            parse_tile_quads(&[0x80, 0x01, 0x20, 0x02, 0x10, 0x03, 0x08, 0x04]).expect("tile quad");

        assert_eq!(quads.len(), 1);
        assert!(quads[0].tiles[0].priority);
        assert_eq!(quads[0].tiles[1].palette_line, 1);
        assert!(quads[0].tiles[2].flip_y);
        assert!(quads[0].tiles[3].flip_x);
    }
}
