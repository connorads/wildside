/// Block collision grid parsed from the Level_Layout RAM format.
///
/// The Level_Layout lives at $FFA652 in 68k RAM, with 2 bytes per 16x16 cell:
///   bit 15:    block flag (cell contains a block)
///   bits 14-12: collision type (3 bits)
///   bits 11-8:  block type (4 bits)
///   bits 7-0:   skin (visible terrain piece or block graphic)
///
/// Collision types from constants.asm:
///   colid_empty     = 0x00
///   colid_rightwall = 0x04
///   colid_leftwall  = 0x08
///   colid_floor     = 0x0C
///   colid_ceiling   = 0x10
///   colid_slopeup   = 0x14
///   colid_slopedown = 0x18
///   colid_hurt      = 0x1C

// ---------------------------------------------------------------------------
// Cell collision types (3-bit field from ram_layout.txt)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CollisionKind {
    Empty = 0,
    // 1 unused
    HiddenBlock = 2,
    // 3 unused
    SlopeDown = 4,
    SlopeUp = 5,
    Solid = 6,
    Spikes = 7,
}

impl CollisionKind {
    fn from_bits(bits: u8) -> Self {
        match bits {
            0 => Self::Empty,
            2 => Self::HiddenBlock,
            4 => Self::SlopeDown,
            5 => Self::SlopeUp,
            6 => Self::Solid,
            7 => Self::Spikes,
            _ => Self::Empty,
        }
    }
}

// ---------------------------------------------------------------------------
// Block types (4-bit field)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockType {
    Smash = 0,
    Prize = 1,
    Ice = 2,
    SolidGhost = 3,
    Teleporter = 4,
    Metal = 5,
    Rubber = 6,
    Shifting = 7,
    Mushroom = 9,
    Evanescent = 0xA,
    Lift = 0xB,
    Drill = 0xC,
    Deleted = 0xF,
    None = 0xFF,
}

impl BlockType {
    fn from_bits(bits: u8) -> Self {
        match bits {
            0 => Self::Smash,
            1 => Self::Prize,
            2 => Self::Ice,
            3 => Self::SolidGhost,
            4 => Self::Teleporter,
            5 => Self::Metal,
            6 => Self::Rubber,
            7 => Self::Shifting,
            9 => Self::Mushroom,
            0xA => Self::Evanescent,
            0xB => Self::Lift,
            0xC => Self::Drill,
            0xF => Self::Deleted,
            _ => Self::None,
        }
    }
}

// ---------------------------------------------------------------------------
// Individual cell
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub has_block: bool,
    pub collision: CollisionKind,
    pub block_type: BlockType,
    pub skin: u8,
}

impl Cell {
    /// Parse from the 2-byte big-endian Level_Layout entry.
    /// Byte layout: [block_flag(1) | collision(3) | block_type(4)] [skin(8)]
    pub fn from_be_bytes(hi: u8, lo: u8) -> Self {
        let has_block = hi & 0x80 != 0;
        let collision_bits = (hi >> 4) & 0x07;
        let block_type_bits = hi & 0x0F;
        Self {
            has_block,
            collision: CollisionKind::from_bits(collision_bits),
            block_type: if has_block {
                BlockType::from_bits(block_type_bits)
            } else {
                BlockType::None
            },
            skin: lo,
        }
    }

    pub fn is_solid_floor(&self) -> bool {
        matches!(
            self.collision,
            CollisionKind::Solid | CollisionKind::SlopeUp | CollisionKind::SlopeDown
        )
    }
}

// ---------------------------------------------------------------------------
// Block grid
// ---------------------------------------------------------------------------

/// The full level collision grid. Each cell is 16x16 pixels.
#[derive(Debug, Clone)]
pub struct BlockGrid {
    pub width_cells: u16,
    pub height_cells: u16,
    cells: Vec<Cell>,
}

/// Result of probing terrain at a point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainProbe {
    pub cell: Cell,
    /// Column in cell grid (0-based).
    pub cell_x: u16,
    /// Row in cell grid (0-based).
    pub cell_y: u16,
}

/// Result of checking for floor beneath a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloorProbe {
    /// Whether there is solid ground at or immediately below the probe point.
    pub on_ground: bool,
    /// The Y pixel position of the floor surface (top of the solid cell).
    /// Only meaningful when `on_ground` is true.
    pub floor_y_px: i32,
    /// The cell that provides the floor.
    pub cell: Cell,
}

impl BlockGrid {
    /// Parse from raw Level_Layout RAM dump (big-endian, 2 bytes per cell,
    /// row-major order). `width_cells` and `height_cells` come from
    /// Level_width_blocks and Level_height_blocks respectively.
    pub fn from_level_layout(data: &[u8], width_cells: u16, height_cells: u16) -> Self {
        let expected = usize::from(width_cells) * usize::from(height_cells) * 2;
        let cell_count = usize::from(width_cells) * usize::from(height_cells);
        let mut cells = Vec::with_capacity(cell_count);
        for i in 0..cell_count {
            let byte_offset = i * 2;
            if byte_offset + 1 < data.len() {
                cells.push(Cell::from_be_bytes(data[byte_offset], data[byte_offset + 1]));
            } else {
                cells.push(Cell::from_be_bytes(0, 0));
            }
        }
        // Pad if data was shorter than expected
        cells.resize(cell_count, Cell::from_be_bytes(0, 0));
        let _ = expected; // suppress unused warning
        Self {
            width_cells,
            height_cells,
            cells,
        }
    }

    /// Look up the cell at grid coordinates. Returns empty cell if out of bounds.
    pub fn cell_at(&self, col: u16, row: u16) -> Cell {
        if col >= self.width_cells || row >= self.height_cells {
            return Cell::from_be_bytes(0, 0);
        }
        self.cells[usize::from(row) * usize::from(self.width_cells) + usize::from(col)]
    }

    /// Level width in pixels.
    pub fn width_pixels(&self) -> i32 {
        i32::from(self.width_cells) * 16
    }

    /// Level height in pixels.
    pub fn height_pixels(&self) -> i32 {
        i32::from(self.height_cells) * 16
    }
}

// ---------------------------------------------------------------------------
// Terrain probing — pure functions
// ---------------------------------------------------------------------------

/// Convert a pixel coordinate to the cell grid index.
/// Matches the 68k `asr.w #4` (divide by 16, arithmetic shift).
pub fn pixel_to_cell(px: i32) -> i32 {
    px >> 4
}

/// Probe the terrain cell at a given pixel position.
pub fn probe_at(grid: &BlockGrid, x_px: i32, y_px: i32) -> TerrainProbe {
    let cx = pixel_to_cell(x_px);
    let cy = pixel_to_cell(y_px);
    let cell_x = cx.max(0) as u16;
    let cell_y = cy.max(0) as u16;
    TerrainProbe {
        cell: grid.cell_at(cell_x, cell_y),
        cell_x,
        cell_y,
    }
}

/// Check for a floor surface at or below the given pixel position.
///
/// Scans downward from the player's feet position. If the cell at the
/// player's feet is solid, the floor is the top of that cell. If not,
/// checks one cell below (to handle the common case of standing just
/// above a surface).
///
/// The `foot_y_px` should be the bottom of the player's collision box
/// (typically player_y_pos >> 16, since y_pos points to the feet in
/// the original game).
pub fn probe_floor(grid: &BlockGrid, x_px: i32, foot_y_px: i32) -> FloorProbe {
    let col = pixel_to_cell(x_px).max(0) as u16;
    let row = pixel_to_cell(foot_y_px).max(0) as u16;

    // Check the cell at foot level
    let cell = grid.cell_at(col, row);
    if cell.is_solid_floor() {
        return FloorProbe {
            on_ground: true,
            floor_y_px: i32::from(row) * 16,
            cell,
        };
    }

    // Check one cell below (player might be at the boundary between cells)
    let below_row = row.saturating_add(1);
    let below_cell = grid.cell_at(col, below_row);
    if below_cell.is_solid_floor() {
        let floor_y = i32::from(below_row) * 16;
        // Only count as "on ground" if the player's feet are within 1 pixel
        // of the surface (handles sub-pixel positioning)
        let on_ground = foot_y_px >= floor_y - 1 && foot_y_px <= floor_y;
        return FloorProbe {
            on_ground,
            floor_y_px: floor_y,
            cell: below_cell,
        };
    }

    FloorProbe {
        on_ground: false,
        floor_y_px: grid.height_pixels(),
        cell,
    }
}

/// Find the floor Y position below a given point by scanning downward.
/// Used during entry animation to find where the player should land.
/// Returns the pixel Y of the top of the first solid cell below `start_y_px`.
pub fn find_floor_below(grid: &BlockGrid, x_px: i32, start_y_px: i32) -> Option<i32> {
    let col = pixel_to_cell(x_px).max(0) as u16;
    let start_row = pixel_to_cell(start_y_px).max(0) as u16;

    for row in start_row..grid.height_cells {
        let cell = grid.cell_at(col, row);
        if cell.is_solid_floor() {
            return Some(i32::from(row) * 16);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_grid() -> BlockGrid {
        // 4x3 grid: top two rows empty, bottom row solid
        let mut data = vec![0u8; 4 * 3 * 2];
        // Row 2 (bottom): solid collision type = 6, shifted into position
        // Cell format: [block_flag(1) collision(3) block_type(4)] [skin(8)]
        // Solid collision (6) in bits 14-12 = 0x60 in the high byte
        for col in 0..4 {
            let offset = (2 * 4 + col) * 2;
            data[offset] = 0x60; // collision type 6 = solid
            data[offset + 1] = 0x01; // arbitrary skin
        }
        BlockGrid::from_level_layout(&data, 4, 3)
    }

    #[test]
    fn cell_at_returns_empty_for_air() {
        let grid = make_test_grid();
        let cell = grid.cell_at(0, 0);
        assert_eq!(cell.collision, CollisionKind::Empty);
        assert!(!cell.has_block);
    }

    #[test]
    fn cell_at_returns_solid_for_ground() {
        let grid = make_test_grid();
        let cell = grid.cell_at(0, 2);
        assert_eq!(cell.collision, CollisionKind::Solid);
    }

    #[test]
    fn cell_at_out_of_bounds_returns_empty() {
        let grid = make_test_grid();
        let cell = grid.cell_at(10, 10);
        assert_eq!(cell.collision, CollisionKind::Empty);
    }

    #[test]
    fn pixel_to_cell_divides_by_16() {
        assert_eq!(pixel_to_cell(0), 0);
        assert_eq!(pixel_to_cell(15), 0);
        assert_eq!(pixel_to_cell(16), 1);
        assert_eq!(pixel_to_cell(31), 1);
        assert_eq!(pixel_to_cell(32), 2);
    }

    #[test]
    fn probe_floor_finds_solid_ground() {
        let grid = make_test_grid();
        // Floor is at row 2 (y=32px). Probing at y=32 should find it.
        let result = probe_floor(&grid, 8, 32);
        assert!(result.on_ground);
        assert_eq!(result.floor_y_px, 32);
    }

    #[test]
    fn probe_floor_in_air_returns_not_grounded() {
        let grid = make_test_grid();
        // y=8 is in the first row (empty)
        let result = probe_floor(&grid, 8, 8);
        assert!(!result.on_ground);
    }

    #[test]
    fn find_floor_below_scans_down() {
        let grid = make_test_grid();
        // Starting from y=0, should find floor at row 2 = y=32
        let floor_y = find_floor_below(&grid, 8, 0);
        assert_eq!(floor_y, Some(32));
    }

    #[test]
    fn cell_from_be_bytes_parses_solid() {
        // 0x60 = 0b0_110_0000 → block=false, collision=6 (solid), block_type=0
        let cell = Cell::from_be_bytes(0x60, 0x42);
        assert!(!cell.has_block);
        assert_eq!(cell.collision, CollisionKind::Solid);
        assert_eq!(cell.block_type, BlockType::None);
        assert_eq!(cell.skin, 0x42);
    }

    #[test]
    fn cell_from_be_bytes_parses_block() {
        // 0xE1 = 0b1_110_0001 → block=true, collision=6 (solid), block_type=1 (prize)
        let cell = Cell::from_be_bytes(0xE1, 0x00);
        assert!(cell.has_block);
        assert_eq!(cell.collision, CollisionKind::Solid);
        assert_eq!(cell.block_type, BlockType::Prize);
    }

    #[test]
    fn cell_from_be_bytes_parses_spikes() {
        // 0x70 = 0b0_111_0000 → collision=7 (spikes)
        let cell = Cell::from_be_bytes(0x70, 0x00);
        assert_eq!(cell.collision, CollisionKind::Spikes);
    }

    #[test]
    fn cell_from_be_bytes_parses_slopes() {
        // 0x40 = collision=4 (slope down)
        let cell = Cell::from_be_bytes(0x40, 0x00);
        assert_eq!(cell.collision, CollisionKind::SlopeDown);

        // 0x50 = collision=5 (slope up)
        let cell = Cell::from_be_bytes(0x50, 0x00);
        assert_eq!(cell.collision, CollisionKind::SlopeUp);
    }
}
