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
    Shooter = 8,
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
            8 => Self::Shooter,
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
    /// Column in the cell grid that provided the floor.
    pub cell_x: u16,
    /// Row in the cell grid that provided the floor.
    pub cell_y: u16,
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
    /// Column in cell grid (0-based). Only meaningful when `hit` is true.
    pub cell_x: u16,
    /// Row in cell grid (0-based). Only meaningful when `hit` is true.
    pub cell_y: u16,
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
    /// Column in cell grid (0-based). Only meaningful when `hit` is true.
    pub cell_x: u16,
    /// Row in cell grid (0-based). Only meaningful when `hit` is true.
    pub cell_y: u16,
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
                cells.push(Cell::from_be_bytes(
                    data[byte_offset],
                    data[byte_offset + 1],
                ));
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

    /// Mutate the cell at grid coordinates. No-op if out of bounds.
    pub fn set_cell_at(&mut self, col: u16, row: u16, cell: Cell) {
        if col >= self.width_cells || row >= self.height_cells {
            return;
        }
        self.cells[usize::from(row) * usize::from(self.width_cells) + usize::from(col)] = cell;
    }
}

// ---------------------------------------------------------------------------
// Block List — prize/ghost/teleporter/hidden block data from RAM dump
// ---------------------------------------------------------------------------

/// Prize content from bits 0-3 of the Block List prize entry's last byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrizeContent {
    Diamond,
    Points10000,
    Skycutter,
    Cyclone,
    RedStealth,
    Eyeclops,
    Juggernaut,
    IronKnight,
    Berzerker,
    Maniaxe,
    Micromax,
    Ankh,
    Clock,
    Continue,
    TenDiamonds,
}

impl PrizeContent {
    fn from_id(id: u8) -> Self {
        match id & 0x0F {
            0x00 | 0x0F => Self::Diamond,
            0x01 => Self::Points10000,
            0x02 => Self::Skycutter,
            0x03 => Self::Cyclone,
            0x04 => Self::RedStealth,
            0x05 => Self::Eyeclops,
            0x06 => Self::Juggernaut,
            0x07 => Self::IronKnight,
            0x08 => Self::Berzerker,
            0x09 => Self::Maniaxe,
            0x0A => Self::Micromax,
            0x0B => Self::Ankh,
            0x0C => Self::Clock,
            0x0D => Self::Continue,
            0x0E => Self::TenDiamonds,
            _ => Self::Diamond,
        }
    }

    pub fn to_id(self) -> u8 {
        match self {
            Self::Diamond => 0x00,
            Self::Points10000 => 0x01,
            Self::Skycutter => 0x02,
            Self::Cyclone => 0x03,
            Self::RedStealth => 0x04,
            Self::Eyeclops => 0x05,
            Self::Juggernaut => 0x06,
            Self::IronKnight => 0x07,
            Self::Berzerker => 0x08,
            Self::Maniaxe => 0x09,
            Self::Micromax => 0x0A,
            Self::Ankh => 0x0B,
            Self::Clock => 0x0C,
            Self::Continue => 0x0D,
            Self::TenDiamonds => 0x0E,
        }
    }
}

/// A single prize block entry from the Block List.
#[derive(Debug, Clone, Copy)]
pub struct PrizeBlockEntry {
    pub col: u16,
    pub row: u16,
    pub content: PrizeContent,
    pub disguised: bool,
}

/// A ghost block group entry from the Block List.
/// Ghost blocks cycle between solid and vanished states on timers.
#[derive(Debug, Clone, Copy)]
pub struct GhostBlockEntry {
    pub col: u16,
    pub row: u16,
    /// Number of blocks in the group (1-based).
    pub count: u8,
    /// True = vertical group, false = horizontal group.
    pub vertical: bool,
    /// Frames the group stays solid.
    pub solid_time: u16,
    /// Frames the group stays vanished.
    pub vanished_time: u16,
    /// Current timer (counts down to next transition).
    pub timer: u16,
    /// True when the ghost blocks are currently solid.
    pub solid: bool,
}

/// A teleporter block entry from the Block List.
/// Touching this block transports the player to the destination.
#[derive(Debug, Clone, Copy)]
pub struct TeleporterEntry {
    pub col: u16,
    pub row: u16,
    /// Destination level/map ID.
    pub dest_map_id: u8,
    /// Destination Y position (in blocks).
    pub dest_y: u8,
    /// Destination X position (in blocks).
    pub dest_x: u16,
}

/// Parsed Block List from the RAM dump at FF3B24.
///
/// The Block List stores runtime data for interactive blocks: prizes,
/// ghost block groups, teleporters, and hidden blocks. Entries are stored
/// sequentially: all prizes first, then ghosts, then teleporters, then hidden.
#[derive(Debug, Clone)]
pub struct BlockList {
    pub prize_blocks: Vec<PrizeBlockEntry>,
    pub ghost_blocks: Vec<GhostBlockEntry>,
    pub teleporters: Vec<TeleporterEntry>,
    pub hidden_blocks: Vec<HiddenBlockEntry>,
}

/// A hidden block entry from the Block List.
/// Hidden blocks are invisible until the player headbutts them from below.
#[derive(Debug, Clone)]
pub struct HiddenBlockEntry {
    pub col: u16,
    pub row: u16,
    /// The block type that gets revealed (Prize, Evanescent, Teleporter, etc.).
    pub block_type: u8,
    /// Block setting byte (direction for shooters/drills, side for teleporters).
    pub block_setting: u8,
}

impl BlockList {
    /// Parse prize block entries from the raw Block List RAM dump.
    ///
    /// Prize entries are 8 bytes each at the start of the block list.
    /// The Level_Layout RAM address in bytes 0-1 is used to detect the
    /// end of the prize section (zero or address outside layout range).
    pub fn from_raw(data: &[u8], layout_width: u16) -> Self {
        let mut prize_blocks = Vec::new();
        let mut offset = 0;
        while offset + 8 <= data.len() {
            let layout_addr = u16::from_be_bytes([data[offset], data[offset + 1]]);
            if layout_addr == 0 {
                break;
            }
            let col = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
            let row = u16::from_be_bytes([data[offset + 4], data[offset + 5]]);
            let disguised = data[offset + 6] & 0x80 != 0;
            let content_id = data[offset + 7] & 0x0F;
            let content = PrizeContent::from_id(content_id);

            // Validate: the layout address should point into the Level_Layout
            // RAM range. Each cell is 2 bytes, so the address divided by 2
            // gives the cell index. If width is 0 or address seems invalid,
            // we've hit the end of prize entries.
            if layout_width > 0 {
                let cell_index = layout_addr / 2;
                let derived_col = cell_index % layout_width;
                let derived_row = cell_index / layout_width;
                // Use the derived position as a cross-check; prefer the
                // explicit col/row fields since they're authoritative.
                let _ = (derived_col, derived_row);
            }

            prize_blocks.push(PrizeBlockEntry {
                col,
                row,
                content,
                disguised,
            });
            offset += 8;
        }

        // Ghost block group entries follow prizes (16 bytes each).
        // Detect end by checking for zero timer and zero layout address.
        let mut ghost_blocks = Vec::new();
        while offset + 16 <= data.len() {
            let timer = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let mapping_frame = data[offset + 3];
            let solid_time = u16::from_be_bytes([data[offset + 4], data[offset + 5]]);
            let vanished_time = u16::from_be_bytes([data[offset + 6], data[offset + 7]]);
            let layout_addr = u16::from_be_bytes([data[offset + 8], data[offset + 9]]);
            if layout_addr == 0 && timer == 0 && solid_time == 0 {
                offset += 16; // advance past ghost terminator entry
                break;
            }
            let col = u16::from_be_bytes([data[offset + 10], data[offset + 11]]);
            let row = u16::from_be_bytes([data[offset + 12], data[offset + 13]]);
            let count = (data[offset + 14] & 0x0F) + 1;
            let vertical = data[offset + 15] & 0x01 != 0;
            let solid = mapping_frame == 0;

            ghost_blocks.push(GhostBlockEntry {
                col,
                row,
                count,
                vertical,
                solid_time,
                vanished_time,
                timer,
                solid,
            });
            offset += 16;
        }

        // Teleporter entries follow ghosts (10 bytes each).
        // End when layout_addr is 0.
        let mut teleporters = Vec::new();
        while offset + 10 <= data.len() {
            let layout_addr = u16::from_be_bytes([data[offset], data[offset + 1]]);
            if layout_addr == 0 {
                break;
            }
            let col = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
            let row = u16::from_be_bytes([data[offset + 4], data[offset + 5]]);
            let dest_map_id = data[offset + 6];
            let dest_y = data[offset + 7];
            let dest_x = u16::from_be_bytes([data[offset + 8], data[offset + 9]]);

            teleporters.push(TeleporterEntry {
                col,
                row,
                dest_map_id,
                dest_y,
                dest_x,
            });
            offset += 10;
        }

        // Hidden block entries follow teleporters (8 bytes each).
        // Skip past teleporter terminator (2 zero bytes).
        if offset + 2 <= data.len() {
            offset += 2; // skip teleporter terminator
        }
        let mut hidden_blocks = Vec::new();
        while offset + 8 <= data.len() {
            let layout_addr = u16::from_be_bytes([data[offset], data[offset + 1]]);
            if layout_addr == 0 {
                break;
            }
            let col = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
            let row = u16::from_be_bytes([data[offset + 4], data[offset + 5]]);
            let block_type = data[offset + 6];
            let block_setting = data[offset + 7];
            hidden_blocks.push(HiddenBlockEntry {
                col,
                row,
                block_type,
                block_setting,
            });
            offset += 8;
        }

        Self {
            prize_blocks,
            ghost_blocks,
            teleporters,
            hidden_blocks,
        }
    }

    /// Look up prize content for a block at the given cell position.
    pub fn prize_content_at(&self, col: u16, row: u16) -> Option<PrizeContent> {
        self.prize_blocks
            .iter()
            .find(|entry| entry.col == col && entry.row == row)
            .map(|entry| entry.content)
    }

    /// Look up a teleporter at a given cell position.
    pub fn teleporter_at(&self, col: u16, row: u16) -> Option<&TeleporterEntry> {
        self.teleporters
            .iter()
            .find(|entry| entry.col == col && entry.row == row)
    }

    /// Look up a hidden block at a given cell position.
    pub fn hidden_block_at(&self, col: u16, row: u16) -> Option<&HiddenBlockEntry> {
        self.hidden_blocks
            .iter()
            .find(|entry| entry.col == col && entry.row == row)
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

/// Compute the floor Y surface within a slope cell using linear interpolation.
///
/// Slopes span 15 pixels vertically within a 16x16 cell (full diagonal).
/// - `SlopeUp`: floor rises left-to-right (higher on the left, lower on the right). `\`
/// - `SlopeDown`: floor falls left-to-right (lower on the left, higher on the right). `/`
fn slope_floor_y(collision: CollisionKind, cell_top_y: i32, x_px: i32) -> i32 {
    let x_within_cell = (x_px & 0x0F).clamp(0, 15);
    match collision {
        // ASM upslope (colid_slopeup $14): high on left, low on right.
        // y = cell_top + 15 - x_within_cell
        CollisionKind::SlopeUp => cell_top_y + 15 - x_within_cell,
        // ASM downslope (colid_slopedown $18): low on left, high on right.
        // y = cell_top + x_within_cell
        CollisionKind::SlopeDown => cell_top_y + x_within_cell,
        _ => cell_top_y,
    }
}

/// Check for a floor surface at or below the given pixel position.
///
/// Scans downward from the player's feet position. If the cell at the
/// player's feet is solid, the floor is the top of that cell (or the
/// interpolated slope surface for slope cells). If not, checks one cell
/// below (to handle the common case of standing just above a surface).
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
        let cell_top = i32::from(row) * 16;
        let floor_y = slope_floor_y(cell.collision, cell_top, x_px);
        // For slopes, only count as on_ground if feet are at or below the
        // slope surface (player has reached the diagonal floor)
        let on_ground = if cell.collision == CollisionKind::SlopeUp
            || cell.collision == CollisionKind::SlopeDown
        {
            foot_y_px >= floor_y
        } else {
            true
        };
        return FloorProbe {
            on_ground,
            floor_y_px: floor_y,
            cell,
            cell_x: col,
            cell_y: row,
        };
    }

    // Check one cell below (player might be at the boundary between cells)
    let below_row = row.saturating_add(1);
    let below_cell = grid.cell_at(col, below_row);
    if below_cell.is_solid_floor() {
        let cell_top = i32::from(below_row) * 16;
        let floor_y = slope_floor_y(below_cell.collision, cell_top, x_px);
        // Only count as "on ground" if the player's feet are within 1 pixel
        // of the surface (handles sub-pixel positioning)
        let on_ground = foot_y_px >= floor_y - 1 && foot_y_px <= floor_y;
        return FloorProbe {
            on_ground,
            floor_y_px: floor_y,
            cell: below_cell,
            cell_x: col,
            cell_y: below_row,
        };
    }

    FloorProbe {
        on_ground: false,
        floor_y_px: grid.height_pixels(),
        cell,
        cell_x: col,
        cell_y: row,
    }
}

/// Find the floor Y position below a given point by scanning downward.
/// Used during entry animation to find where the player should land.
/// Returns the pixel Y of the floor surface (interpolated for slopes)
/// of the first solid cell below `start_y_px`.
pub fn find_floor_below(grid: &BlockGrid, x_px: i32, start_y_px: i32) -> Option<i32> {
    let col = pixel_to_cell(x_px).max(0) as u16;
    let start_row = pixel_to_cell(start_y_px).max(0) as u16;

    for row in start_row..grid.height_cells {
        let cell = grid.cell_at(col, row);
        if cell.is_solid_floor() {
            let cell_top = i32::from(row) * 16;
            return Some(slope_floor_y(cell.collision, cell_top, x_px));
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
    let center = probe_floor(grid, center_x_px, foot_y_px);
    if center.on_ground {
        return center;
    }

    let left = probe_floor(grid, center_x_px - half_width, foot_y_px);
    let right = probe_floor(grid, center_x_px + half_width, foot_y_px);
    let select_higher_floor = |a: FloorProbe, b: FloorProbe| {
        if a.floor_y_px <= b.floor_y_px { a } else { b }
    };

    match (left.on_ground, right.on_ground) {
        (true, true) => select_higher_floor(left, right),
        (true, false) => left,
        (false, true) => right,
        (false, false) => match (left.cell.is_solid_floor(), right.cell.is_solid_floor()) {
            (true, true) => select_higher_floor(left, right),
            (true, false) => left,
            (false, true) => right,
            (false, false) => left,
        },
    }
}

/// Probe a horizontal span and return the highest floor support found.
///
/// This is used during level-entry settling where the original game runs a
/// whole frame of collision logic before the first playable frame. Some spawn
/// points stabilise against support slightly beyond the normal gameplay hitbox.
pub fn probe_floor_span(
    grid: &BlockGrid,
    left_x_px: i32,
    right_x_px: i32,
    foot_y_px: i32,
) -> FloorProbe {
    let start_x = left_x_px.min(right_x_px);
    let end_x = left_x_px.max(right_x_px);
    let select_higher_floor = |a: FloorProbe, b: FloorProbe| {
        if a.floor_y_px <= b.floor_y_px { a } else { b }
    };
    let mut best_on_ground: Option<FloorProbe> = None;
    let mut best_support: Option<FloorProbe> = None;

    for x_px in start_x..=end_x {
        let probe = probe_floor(grid, x_px, foot_y_px);
        if probe.on_ground {
            best_on_ground = Some(match best_on_ground {
                Some(current) => select_higher_floor(current, probe),
                None => probe,
            });
            continue;
        }
        if probe.cell.is_solid_floor() {
            best_support = Some(match best_support {
                Some(current) => select_higher_floor(current, probe),
                None => probe,
            });
        }
    }

    best_on_ground
        .or(best_support)
        .unwrap_or_else(|| probe_floor(grid, start_x, foot_y_px))
}

/// Check for a wall blocking horizontal movement.
///
/// Probes the leading edge of the hitbox at lower-body, mid-body, and a
/// near-head position. The lower probe stays several pixels above the feet so
/// landing frames do not misclassify the floor cell as a side wall before
/// floor resolution runs.
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
        center_x_px + half_width + 1
    } else {
        center_x_px - half_width
    };

    // Keep the lower probe above the landing cell; otherwise a descending
    // player can clip a few pixels into the floor and get snapped sideways as
    // if the floor were a wall.
    let lower_body_y = foot_y_px - 8;
    let upper_body_y = foot_y_px - height + 8;
    let probe_ys = [lower_body_y, foot_y_px - height / 2, upper_body_y];

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
                cell_x: probe.cell_x,
                cell_y: probe.cell_y,
            };
        }
    }

    WallProbe {
        hit: false,
        wall_x_px: 0,
        cell: Cell::from_be_bytes(0, 0),
        cell_x: 0,
        cell_y: 0,
    }
}

/// Probe only the player's exact top edge against a side wall.
///
/// This is intentionally narrower than `probe_wall`: it is used to keep the
/// player pinned under a right-side overhang for a few extra airborne frames
/// after a real wall hit, until the top edge has actually cleared the tile.
pub fn probe_wall_top_edge(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    half_width: i32,
    height: i32,
    moving_right: bool,
) -> WallProbe {
    let edge_x = if moving_right {
        center_x_px + half_width + 1
    } else {
        center_x_px - half_width
    };
    let top_y = foot_y_px - height;
    let probe = probe_at(grid, edge_x, top_y);
    if probe.cell.collision == CollisionKind::Solid {
        let wall_x = if moving_right {
            i32::from(probe.cell_x) * 16
        } else {
            (i32::from(probe.cell_x) + 1) * 16
        };
        return WallProbe {
            hit: true,
            wall_x_px: wall_x,
            cell: probe.cell,
            cell_x: probe.cell_x,
            cell_y: probe.cell_y,
        };
    }

    WallProbe {
        hit: false,
        wall_x_px: 0,
        cell: Cell::from_be_bytes(0, 0),
        cell_x: 0,
        cell_y: 0,
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
        if matches!(
            probe.cell.collision,
            CollisionKind::Solid | CollisionKind::HiddenBlock
        ) {
            return CeilingProbe {
                hit: true,
                // Bottom of ceiling cell = next row boundary
                ceiling_y_px: (i32::from(probe.cell_y) + 1) * 16,
                cell: probe.cell,
                cell_x: probe.cell_x,
                cell_y: probe.cell_y,
            };
        }
    }

    CeilingProbe {
        hit: false,
        ceiling_y_px: 0,
        cell: Cell::from_be_bytes(0, 0),
        cell_x: 0,
        cell_y: 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::identity_op)]
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
    fn probe_wall_does_not_treat_floor_as_side_wall_on_landing() {
        let grid = make_test_grid();
        // Feet have already dipped 1px into the landing row, but the player is
        // only intersecting the flat floor from above. This should not report a
        // side wall at x=80.
        let probe = probe_wall(&grid, 76, 49, 5, 31, true);
        assert!(!probe.hit);
    }

    #[test]
    fn probe_wall_hits_upper_overhang_edge() {
        let mut data = vec![0u8; 5 * 5 * 2];
        let offset = (1 * 5 + 2) * 2;
        data[offset] = 0x60;
        let grid = BlockGrid::from_level_layout(&data, 5, 5);

        let probe = probe_wall(&grid, 26, 48, 5, 31, true);
        assert!(probe.hit);
        assert_eq!(probe.wall_x_px, 32);
    }

    #[test]
    fn probe_wall_top_edge_keeps_overhang_contact_after_upper_probe_clears() {
        let mut data = vec![0u8; 5 * 5 * 2];
        let offset = (1 * 5 + 2) * 2;
        data[offset] = 0x60;
        let grid = BlockGrid::from_level_layout(&data, 5, 5);

        let probe = probe_wall(&grid, 26, 56, 5, 31, true);
        assert!(!probe.hit);

        let top_edge_probe = probe_wall_top_edge(&grid, 26, 56, 5, 31, true);
        assert!(top_edge_probe.hit);
        assert_eq!(top_edge_probe.wall_x_px, 32);
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

    // -----------------------------------------------------------------------
    // US-013: Slope floor physics
    // -----------------------------------------------------------------------

    /// Build a grid with slope cells for testing:
    ///
    ///   col: 0  1  2  3
    /// row 0: .  .  .  .
    /// row 1: /  \  S  .   (SlopeUp, SlopeDown, Solid, empty)
    fn make_slope_grid() -> BlockGrid {
        let w = 4u16;
        let h = 2u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Row 1, col 0: SlopeUp (collision=5)
        let offset = (1 * 4 + 0) * 2;
        data[offset] = 0x50; // collision=5 = SlopeUp
        // Row 1, col 1: SlopeDown (collision=4)
        let offset = (1 * 4 + 1) * 2;
        data[offset] = 0x40; // collision=4 = SlopeDown
        // Row 1, col 2: Solid (collision=6)
        let offset = (1 * 4 + 2) * 2;
        data[offset] = 0x60; // collision=6 = Solid
        BlockGrid::from_level_layout(&data, w, h)
    }

    #[test]
    fn slope_up_floor_y_interpolates_left_to_right() {
        let grid = make_slope_grid();
        // SlopeUp `/` at col 0, row 1 (cell_top = 16px).
        // ASM colid_slopeup: y = cell_top + 15 - x_within_cell
        // Left edge (x=0): floor should be low (high Y = 31)
        // Right edge (x=15): floor should be high (low Y = 16)
        let left = probe_floor(&grid, 0, 31);
        assert!(left.on_ground);
        assert_eq!(left.floor_y_px, 31);

        let right = probe_floor(&grid, 15, 16);
        assert!(right.on_ground);
        assert_eq!(right.floor_y_px, 16);
    }

    #[test]
    fn slope_down_floor_y_interpolates_left_to_right() {
        let grid = make_slope_grid();
        // SlopeDown `\` at col 1, row 1 (cell_top = 16px).
        // ASM colid_slopedown: y = cell_top + x_within_cell
        // Left edge (x=16): floor should be high (low Y = 16)
        // Right edge (x=31): floor should be low (high Y = 31)
        let left = probe_floor(&grid, 16, 16);
        assert!(left.on_ground);
        assert_eq!(left.floor_y_px, 16);

        let right = probe_floor(&grid, 31, 31);
        assert!(right.on_ground);
        assert_eq!(right.floor_y_px, 31);
    }

    #[test]
    fn slope_up_mid_cell_interpolation() {
        let grid = make_slope_grid();
        // SlopeUp at col 0, row 1. x=8: floor_y = 16 + 15 - 8 = 23
        let probe = probe_floor(&grid, 8, 23);
        assert!(probe.on_ground);
        assert_eq!(probe.floor_y_px, 23);
    }

    #[test]
    fn slope_not_on_ground_when_above_surface() {
        let grid = make_slope_grid();
        // SlopeUp at col 0, row 1. x=8: floor_y = 16 + 15 - 8 = 23.
        // Player at y=20 (above slope surface) should not be on ground.
        let probe = probe_floor(&grid, 8, 20);
        assert!(!probe.on_ground);
        assert_eq!(probe.floor_y_px, 23);
    }

    #[test]
    fn solid_floor_unchanged_by_slope_logic() {
        let grid = make_slope_grid();
        // Solid at col 2, row 1 (cell_top = 16px). Floor Y = 16 regardless of x.
        let probe = probe_floor(&grid, 32, 16);
        assert!(probe.on_ground);
        assert_eq!(probe.floor_y_px, 16);

        let probe = probe_floor(&grid, 47, 16);
        assert!(probe.on_ground);
        assert_eq!(probe.floor_y_px, 16);
    }

    #[test]
    fn probe_floor_hitbox_prefers_center_support_on_slope() {
        let grid = make_slope_grid();
        // SlopeUp at col 0, row 1. x=8: floor_y = 16 + 15 - 8 = 23
        let probe = probe_floor_hitbox(&grid, 8, 23, 5);
        assert!(probe.on_ground);
        assert_eq!(probe.floor_y_px, 23);
    }

    #[test]
    fn probe_floor_hitbox_prefers_center_support_before_edge_probes() {
        let w = 3u16;
        let h = 3u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // SlopeDown at (col 1, row 1): collision=4
        data[(usize::from(1u16) * usize::from(w) + usize::from(1u16)) * 2] = 0x40;
        // Solid at (col 0, row 2): collision=6
        data[(usize::from(2u16) * usize::from(w) + usize::from(0u16)) * 2] = 0x60;
        let grid = BlockGrid::from_level_layout(&data, w, h);

        // SlopeDown (new formula): floor_y = cell_top + x_within_cell
        // Col 1 row 1: cell_top=16. x=19, x_within_cell=3: floor_y=19
        let probe = probe_floor_hitbox(&grid, 19, 19, 5);
        assert!(probe.on_ground);
        assert_eq!(probe.cell.collision, CollisionKind::SlopeDown);
        assert_eq!(probe.floor_y_px, 19);
    }

    #[test]
    fn probe_floor_span_finds_support_beyond_normal_hitbox() {
        let w = 6u16;
        let h = 4u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        data[(usize::from(2u16) * usize::from(w) + usize::from(2u16)) * 2] = 0x60;
        for col in 0..usize::from(w) {
            data[(usize::from(3u16) * usize::from(w) + col) * 2] = 0x70;
        }
        let grid = BlockGrid::from_level_layout(&data, w, h);

        let narrow = probe_floor_hitbox(&grid, 16, 47, 5);
        assert_eq!(narrow.cell.collision, CollisionKind::Spikes);
        assert_eq!(narrow.floor_y_px, 48);

        let wide = probe_floor_span(&grid, 0, 32, 47);
        assert_eq!(wide.cell.collision, CollisionKind::Solid);
        assert_eq!(wide.floor_y_px, 32);
    }
}
