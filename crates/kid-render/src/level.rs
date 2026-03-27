use crate::tiles::TileQuad;
use anyhow::{Result, bail};
use kid_data::BgscrollProfile;

const BLOCK_VARIANT_SKIN_COUNT: usize = 16;
const BLOCK_VARIANT_GROUP_COUNT: usize = 5;
const BLOCK_MAPPING_QUAD_COUNT: usize = 1 + BLOCK_VARIANT_SKIN_COUNT * BLOCK_VARIANT_GROUP_COUNT;
const FOREGROUND_CELLS_PER_SCREEN_X: u16 = 20;
const FOREGROUND_CELLS_PER_SCREEN_Y: u16 = 14;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForegroundLayout {
    pub width_cells: u16,
    pub height_cells: u16,
    cells: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelVisuals {
    pub level_id: u8,
    pub foreground_theme_id: u8,
    pub background_theme_id: u8,
    pub background_scroll: Option<BgscrollProfile>,
    pub foreground_layout: ForegroundLayout,
    pub foreground_layout_bytes: Vec<u8>,
    pub block_layout_bytes: Vec<u8>,
    pub background_layout_bytes: Vec<u8>,
    pub block_mapping: BlockVisualMapping,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockVisualMapping {
    terrain_by_skin: Vec<TileQuad>,
    /// Block_Mappings: flat array indexed by cell skin byte.
    /// ASM uses skin byte × 8 as offset into Block_Mappings for ALL block
    /// types — the skin byte is a direct index, not per-block-type.
    block_quads: Vec<TileQuad>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockVisualKind {
    Terrain,
    GenericBlock,
    Smash,
    Prize,
    Ice,
    Metal,
    Rubber,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedBlockVisual<'a> {
    pub kind: BlockVisualKind,
    pub quad: &'a TileQuad,
}

impl ForegroundLayout {
    pub fn parse(bytes: &[u8], width_screens: u8, height_screens: u8) -> Result<Self> {
        let width_cells = u16::from(width_screens) * FOREGROUND_CELLS_PER_SCREEN_X;
        let height_cells = u16::from(height_screens) * FOREGROUND_CELLS_PER_SCREEN_Y;
        let expected_len = usize::from(width_cells) * usize::from(height_cells);
        if bytes.len() == expected_len {
            return Ok(Self {
                width_cells,
                height_cells,
                cells: bytes.to_vec(),
            });
        }
        // Tolerate slight size differences (±5 bytes) — some levels have
        // trailing padding or are minimally truncated in the extracted data.
        if bytes.len().abs_diff(expected_len) <= 5 {
            let mut cells = vec![0u8; expected_len];
            let copy_len = bytes.len().min(expected_len);
            cells[..copy_len].copy_from_slice(&bytes[..copy_len]);
            return Ok(Self {
                width_cells,
                height_cells,
                cells,
            });
        }
        // For large mismatches, try to infer dimensions from the data length.
        // The width must be a multiple of FOREGROUND_CELLS_PER_SCREEN_X (20).
        let cells_per_x = usize::from(FOREGROUND_CELLS_PER_SCREEN_X);
        let cells_per_y = usize::from(FOREGROUND_CELLS_PER_SCREEN_Y);
        if bytes.len().is_multiple_of(cells_per_x) {
            let total_rows = bytes.len() / cells_per_x;
            if total_rows.is_multiple_of(cells_per_y) || total_rows > 0 {
                let inferred_width = width_cells;
                let inferred_height = (bytes.len() / usize::from(inferred_width)).max(1) as u16;
                if usize::from(inferred_width) * usize::from(inferred_height) == bytes.len() {
                    return Ok(Self {
                        width_cells: inferred_width,
                        height_cells: inferred_height,
                        cells: bytes.to_vec(),
                    });
                }
            }
        }
        bail!(
            "foreground layout length {} does not match expected {} ({}x{} cells)",
            bytes.len(),
            expected_len,
            width_cells,
            height_cells
        );
    }

    pub fn skin_at(&self, col: u16, row: u16) -> u8 {
        if col >= self.width_cells || row >= self.height_cells {
            return 0;
        }
        self.cells[usize::from(row) * usize::from(self.width_cells) + usize::from(col)]
    }
}

impl BlockVisualMapping {
    pub fn new(terrain_by_skin: Vec<TileQuad>, block_quads: Vec<TileQuad>) -> Result<Self> {
        if block_quads.len() != BLOCK_MAPPING_QUAD_COUNT {
            bail!(
                "expected {BLOCK_MAPPING_QUAD_COUNT} block mapping quads, got {}",
                block_quads.len()
            );
        }

        Ok(Self {
            terrain_by_skin,
            block_quads,
        })
    }

    pub fn resolve(&self, skin: u8, block_type: Option<u8>) -> Option<ResolvedBlockVisual<'_>> {
        let index = usize::from(skin);
        match block_type {
            None => self
                .terrain_by_skin
                .get(index)
                .map(|quad| ResolvedBlockVisual {
                    kind: BlockVisualKind::Terrain,
                    quad,
                }),
            Some(block_type) => {
                let (kind, group_index) = block_visual_kind(block_type)?;
                let block_index = 1 + group_index * BLOCK_VARIANT_SKIN_COUNT + index;
                self.block_quads
                    .get(block_index)
                    .or(self.block_quads.first())
                    .map(|quad| ResolvedBlockVisual { kind, quad })
            }
        }
    }
}

fn block_visual_kind(block_type: u8) -> Option<(BlockVisualKind, usize)> {
    match block_type {
        0 => Some((BlockVisualKind::Smash, 0)),
        1 => Some((BlockVisualKind::Prize, 1)),
        2 => Some((BlockVisualKind::Ice, 2)),
        5 => Some((BlockVisualKind::Metal, 3)),
        6 => Some((BlockVisualKind::Rubber, 4)),
        // Other block types (Mushroom, SolidGhost, Teleporter, Shifting, etc.)
        // keep their terrain skin — no override needed.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ForegroundLayout;

    #[test]
    fn parse_foreground_layout_uses_level_screen_dimensions() {
        let bytes = vec![0x2A; 20 * 14 * 2];

        let layout = ForegroundLayout::parse(&bytes, 2, 1).expect("foreground layout");

        assert_eq!(layout.width_cells, 40);
        assert_eq!(layout.height_cells, 14);
        assert_eq!(layout.skin_at(0, 0), 0x2A);
        assert_eq!(layout.skin_at(39, 13), 0x2A);
        assert_eq!(layout.skin_at(40, 0), 0);
    }

    #[test]
    fn parse_foreground_layout_rejects_truncated_bytes() {
        let error = ForegroundLayout::parse(&[0x00; 10], 1, 1).expect_err("truncated layout");

        assert!(
            error
                .to_string()
                .contains("foreground layout length 10 does not match expected 280")
        );
    }
}
