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
            CollisionKind::Solid
                | CollisionKind::SlopeUp
                | CollisionKind::SlopeDown
                | CollisionKind::Spikes
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

/// Result of checking for a wall blocking horizontal movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallProbe {
    /// Whether a solid wall was found at the leading edge.
    pub hit: bool,
    /// The X pixel of the wall surface (left edge of cell when moving right,
    /// right edge when moving left). Only meaningful when `hit` is true.
    pub wall_x_px: i32,
    pub cell: Cell,
}

/// Result of checking for a ceiling above the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CeilingProbe {
    /// Whether a solid ceiling was found above the hitbox.
    pub hit: bool,
    /// The Y pixel of the bottom of the ceiling cell.
    /// Only meaningful when `hit` is true.
    pub ceiling_y_px: i32,
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

/// Probe floor checking both horizontal edges of the player's hitbox.
///
/// Returns the highest floor found (smallest Y value = highest position)
/// so the player lands on top of the tallest surface under either edge.
pub fn probe_floor_hitbox(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    half_width: i32,
) -> FloorProbe {
    let left = probe_floor(grid, center_x_px - half_width, foot_y_px);
    let right = probe_floor(grid, center_x_px + half_width, foot_y_px);

    match (left.on_ground, right.on_ground) {
        (true, true) => {
            if left.floor_y_px <= right.floor_y_px {
                left
            } else {
                right
            }
        }
        (true, false) => left,
        (false, true) => right,
        (false, false) => left,
    }
}

/// Check for a wall blocking horizontal movement.
///
/// Probes the leading edge of the hitbox at two Y positions: just above
/// foot level (to avoid false-positives on floor cells) and mid-body.
/// Only `Solid` cells count as walls — slopes allow pass-through.
pub fn probe_wall(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    half_width: i32,
    height: i32,
    moving_right: bool,
) -> WallProbe {
    let edge_x = if moving_right {
        center_x_px + half_width
    } else {
        center_x_px - half_width
    };

    // Probe just above feet (avoids floor) and at mid-body
    let probe_ys = [foot_y_px - 1, foot_y_px - height / 2];

    for &py in &probe_ys {
        let probe = probe_at(grid, edge_x, py);
        if probe.cell.collision == CollisionKind::Solid {
            let wall_x = if moving_right {
                // Wall surface is left edge of the solid cell
                i32::from(probe.cell_x) * 16
            } else {
                // Wall surface is right edge of the solid cell
                (i32::from(probe.cell_x) + 1) * 16
            };
            return WallProbe {
                hit: true,
                wall_x_px: wall_x,
                cell: probe.cell,
            };
        }
    }

    WallProbe {
        hit: false,
        wall_x_px: 0,
        cell: Cell::from_be_bytes(0, 0),
    }
}

/// Check for a ceiling above the player's hitbox.
///
/// Probes both horizontal edges at the top of the hitbox.
/// Only `Solid` cells count as ceilings.
pub fn probe_ceiling(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    half_width: i32,
    height: i32,
) -> CeilingProbe {
    let top_y = foot_y_px - height;

    for &px in &[center_x_px - half_width, center_x_px + half_width] {
        let probe = probe_at(grid, px, top_y);
        if probe.cell.collision == CollisionKind::Solid {
            return CeilingProbe {
                hit: true,
                // Bottom of ceiling cell = next row boundary
                ceiling_y_px: (i32::from(probe.cell_y) + 1) * 16,
                cell: probe.cell,
            };
        }
    }

    CeilingProbe {
        hit: false,
        ceiling_y_px: 0,
        cell: Cell::from_be_bytes(0, 0),
    }
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

    // -----------------------------------------------------------------------
    // Hitbox-aware probes
    // -----------------------------------------------------------------------

    /// Build a grid with a floor gap and a wall:
    ///
    ///   col: 0  1  2  3  4  5
    /// row 0: .  .  .  .  .  .
    /// row 1: .  .  .  .  .  .
    /// row 2: .  .  .  .  .  .
    /// row 3: S  S  .  .  S  S    (floor with gap in cols 2-3)
    /// row 4: S  S  S  S  S  S    (solid below)
    ///
    /// Plus a wall: col 5 rows 1-2 are solid (wall on right side).
    fn make_wall_grid() -> BlockGrid {
        let w = 6u16;
        let h = 5u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let set_solid = |data: &mut Vec<u8>, col: usize, row: usize| {
            let offset = (row * usize::from(w) + col) * 2;
            data[offset] = 0x60; // collision=6 (Solid)
            data[offset + 1] = 0x01;
        };
        // Row 3: floor with gap
        set_solid(&mut data, 0, 3);
        set_solid(&mut data, 1, 3);
        set_solid(&mut data, 4, 3);
        set_solid(&mut data, 5, 3);
        // Row 4: solid everywhere
        for col in 0..6 {
            set_solid(&mut data, col, 4);
        }
        // Wall: col 5, rows 1-2
        set_solid(&mut data, 5, 1);
        set_solid(&mut data, 5, 2);
        BlockGrid::from_level_layout(&data, w, h)
    }

    #[test]
    fn probe_floor_hitbox_lands_on_both_edges() {
        let grid = make_wall_grid();
        // Player at x=8 (col 0), half_width=5. Both edges in cols 0-1, floor at row 3.
        let probe = probe_floor_hitbox(&grid, 8, 48, 5);
        assert!(probe.on_ground);
        assert_eq!(probe.floor_y_px, 48);
    }

    #[test]
    fn probe_floor_hitbox_one_edge_over_gap() {
        let grid = make_wall_grid();
        // Player at x=28 (near boundary of col 1/2). Left edge at 23 (col 1, solid),
        // right edge at 33 (col 2, gap). Should still find floor from left edge.
        let probe = probe_floor_hitbox(&grid, 28, 48, 5);
        assert!(probe.on_ground);
        assert_eq!(probe.floor_y_px, 48);
    }

    #[test]
    fn probe_floor_hitbox_both_edges_over_gap() {
        let grid = make_wall_grid();
        // Player at x=40 (col 2), half_width=5. Both edges in cols 2-3 (gap).
        let probe = probe_floor_hitbox(&grid, 40, 48, 5);
        assert!(!probe.on_ground);
    }

    #[test]
    fn probe_wall_moving_right_hits_wall() {
        let grid = make_wall_grid();
        // Player at x=72 (col 4), half_width=5. Right edge at 77 (col 4).
        // Move a bit right so edge enters col 5 (wall).
        // x=76, right edge = 81, pixel_to_cell(81) = 5 → wall col
        let probe = probe_wall(&grid, 76, 47, 5, 31, true);
        assert!(probe.hit);
        // Wall surface is left edge of col 5 = 80px
        assert_eq!(probe.wall_x_px, 80);
    }

    #[test]
    fn probe_wall_moving_right_no_wall() {
        let grid = make_wall_grid();
        // Player at x=24, no wall to the right
        let probe = probe_wall(&grid, 24, 47, 5, 31, true);
        assert!(!probe.hit);
    }

    #[test]
    fn probe_wall_moving_left_hits_wall() {
        let grid = make_wall_grid();
        // Wall at col 5 rows 1-2. If player is to the right of col 5,
        // moving left... Actually let's test against the floor acting as
        // a wall by building a scenario.
        // Better: use the existing wall at col 5. Player just right of wall,
        // moving left. But col 5 is the last column. Let me test differently.
        //
        // Player at x=81 (past col 5 wall), feet at y=31 (row 1 area).
        // Left edge = 81-5 = 76, pixel_to_cell(76) = 4 → no wall. Need to
        // be closer: x=85, left edge = 80, pixel_to_cell(80) = 5 → wall!
        let probe = probe_wall(&grid, 85, 31, 5, 31, false);
        assert!(probe.hit);
        // Wall surface is right edge of col 5 = 96px
        assert_eq!(probe.wall_x_px, 96);
    }

    #[test]
    fn probe_wall_ignores_slopes() {
        // Slopes should not block horizontal movement
        let mut data = vec![0u8; 4 * 3 * 2];
        // Put a slope-up (collision=5) at col 2, row 1
        let offset = (1 * 4 + 2) * 2;
        data[offset] = 0x50; // collision=5 = SlopeUp
        let grid = BlockGrid::from_level_layout(&data, 4, 3);

        // Player at x=30, right edge=35, pixel_to_cell(35)=2 → slope cell
        let probe = probe_wall(&grid, 30, 20, 5, 31, true);
        assert!(!probe.hit);
    }

    #[test]
    fn probe_ceiling_hits_solid_above() {
        let grid = make_wall_grid();
        // Ceiling test: wall at col 5 rows 1-2. Put player below with head
        // reaching into a solid cell.
        // Actually, let's build a ceiling scenario with make_test_grid:
        // Row 0 empty, row 1 empty, row 2 solid. Player feet at y=31 (row 1),
        // height=31, top = 31-31=0, pixel_to_cell(0)=0 → empty. No ceiling.
        //
        // Use wall_grid: player at col 0-1 area. Solid is at row 3-4.
        // For ceiling, we need solid above. Wall at col 5 rows 1-2.
        // Player at x=82, feet at y=48, height=31, top=17, pixel_to_cell(17)=1.
        // Left edge=77 (col 4, row 1: empty). Right edge=87 (col 5, row 1: solid!)
        let probe = probe_ceiling(&grid, 82, 48, 5, 31);
        assert!(probe.hit);
        // Bottom of ceiling cell (col 5, row 1) = (1+1)*16 = 32
        assert_eq!(probe.ceiling_y_px, 32);
    }

    #[test]
    fn probe_ceiling_clear_above() {
        let grid = make_wall_grid();
        // Player at x=24, feet at y=48, height=31, top=17 (row 1).
        // Left edge=19 (col 1, row 1: empty). Right edge=29 (col 1, row 1: empty).
        let probe = probe_ceiling(&grid, 24, 48, 5, 31);
        assert!(!probe.hit);
    }
}
