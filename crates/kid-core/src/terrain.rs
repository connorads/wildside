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
    SlopeUp = 4,
    SlopeDown = 5,
    Solid = 6,
    Spikes = 7,
}

impl CollisionKind {
    fn from_bits(bits: u8) -> Self {
        match bits & 0x07 {
            0 | 1 | 3 => Self::Empty,
            2 => Self::HiddenBlock,
            4 => Self::SlopeUp,
            5 => Self::SlopeDown,
            6 => Self::Solid,
            7 => Self::Spikes,
            _ => unreachable!("collision bits are masked to 3 bits"),
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

    fn to_word(self) -> u16 {
        let block_flag = if self.has_block { 0x8000 } else { 0 };
        let collision_bits = (self.collision as u16) << 12;
        let block_type_bits = if self.has_block {
            (self.block_type as u16) << 8
        } else {
            0
        };
        block_flag | collision_bits | block_type_bits | u16::from(self.skin)
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

/// Result of ROM horizontal body/lead-column scans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorizontalCollisionScan {
    /// Any bit-$4000 collision cell inside the current hitbox rectangle.
    pub hitbox_blocked: bool,
    /// Any bit-$4000 collision cell in the lead column, or the level boundary.
    pub lead_column_blocked: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KidBlockInteractionProbeSubBBE2 {
    pub(crate) selected_layout_addr: u16,
    pub(crate) selected_col: u16,
    pub(crate) selected_row: u16,
    pub(crate) dispatch_table_offset: u16,
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn kid_block_interaction_probe_sub_BBE2(
    grid: &BlockGrid,
    layout_addr: u16,
    x_pos_px: u16,
) -> Option<KidBlockInteractionProbeSubBBE2> {
    // ASM sub_BBE2 (kid.asm:15142): derive `(col,row)` from the current
    // `Level_Layout` address, and when Kid has moved past that source column,
    // probe the tile immediately to the right. Only a right-hand tile whose
    // collision bits equal `$6000` may replace the base tile, and whichever
    // tile wins must still have bit 15 set before its block-type nibble is
    // converted into the `off_BB80` dispatch-table byte offset.
    if grid.width_cells == 0 {
        return None;
    }

    let cell_index = usize::from(layout_addr >> 1);
    let base_col = (cell_index % usize::from(grid.width_cells)) as u16;
    let base_row = (cell_index / usize::from(grid.width_cells)) as u16;
    let player_col = x_pos_px >> 4;

    let mut selected_layout_addr = layout_addr;
    let mut selected_col = base_col;
    let selected_row = base_row;
    let mut selected_cell = grid.cell_at(base_col, base_row);

    if player_col > base_col {
        let right_col = base_col.wrapping_add(1);
        let right_cell = grid.cell_at(right_col, base_row);
        if matches!(right_cell.collision, CollisionKind::Solid) {
            selected_layout_addr = layout_addr.wrapping_add(2);
            selected_col = right_col;
            selected_cell = right_cell;
        }
    }

    // ASM loc_BC1C (kid.asm:15162): fallback when player hasn't crossed the
    // source column (or right-hand tile wasn't Solid/$6000).  Reads the base
    // tile word: `move.w (a4),d7; bclr #$F,d7; beq.w loc_BC30` — bit 15
    // (has_block) clear means no block, zero d3 and return (None).
    if !selected_cell.has_block {
        return None;
    }

    // ASM loc_BC26 (kid.asm:15167): extract block-type nibble into the
    // off_BB80 dispatch-table byte offset.  `andi.w #$F00,d7; asr.w #6,d7`
    // isolates bits 11-8 then shifts right by 6, yielding block_type * 4.
    // `tst.w d3; rts` — sets Z flag for the caller (beq = "no block found"
    // when d3 was zeroed at loc_BC30; bne = valid block when d3 is nonzero).
    Some(KidBlockInteractionProbeSubBBE2 {
        selected_layout_addr,
        selected_col,
        selected_row,
        dispatch_table_offset: u16::from(selected_cell.block_type as u8) << 2,
    })
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn juggernaut_wall_scan_sub_96E8(
    grid: &BlockGrid,
    x_px: i32,
    y_px: i32,
    facing: crate::physics::Direction,
) -> bool {
    // ASM sub_96E8 (kid.asm:11231): scan the 16x16 tile rectangle bounded by
    // `y_pos-$1F .. y_pos-$10` and the facing-dependent X span
    // (`x_pos-$0B .. x_pos+$04` for clear x-direction, mirrored for set
    // x-direction). Return true on the first tile whose collision bits equal
    // `$6000`; slopes/spikes do not count.
    let row_start = (y_px - 0x1F) >> 4;
    let row_end = (y_px - 0x10) >> 4;
    // ASM loc_971A (kid.asm:11249): after the optional `exg d5,d7`,
    // convert the facing offsets into left/right pixel edges and tile columns.
    let (left_px, right_px) = match facing {
        crate::physics::Direction::Right => (x_px - 0x0B, x_px + 0x04),
        crate::physics::Direction::Left => (x_px - 0x04, x_px + 0x0B),
    };
    let col_start = left_px >> 4;
    let col_end = right_px >> 4;

    // ASM loc_972E (kid.asm:11259): reset the inner column count for each
    // row, scan across it, then advance to the next Level_width_tiles row.
    for row in row_start..=row_end {
        if row < 0 {
            continue;
        }
        for col in col_start..=col_end {
            if col < 0 {
                continue;
            }
            // ASM loc_9732 (kid.asm:11263): mask `(a1)+` with `$7000` and
            // only branch to the hit return when the result equals `$6000`.
            if grid.cell_at(col as u16, row as u16).collision == CollisionKind::Solid {
                // ASM loc_9754 (kid.asm:11276): restore registers, set d7=1,
                // and return NE to the caller on the first matching tile.
                return true;
            }
        }
    }

    false
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

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn blank_top_rows_from_height_flag_bits_sub_140F6(height_flag_bits: u8) -> u16 {
    // ASM sub_140F6 relies on Number_blank_top_rows, which Load_LevelData
    // seeds from the header's top two height bits as `height_flag_bits * 2`.
    u16::from(height_flag_bits & 0x03) * 2
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn insert_blank_top_rows_sub_140F6(
    level_layout: &[u8],
    width_blocks: u16,
    blank_top_rows: u16,
) -> Vec<u8> {
    // ASM sub_140F6 (kid.asm:26925) shifts the populated level-layout rows
    // downward to make room for Number_blank_top_rows empty rows at the top.
    // ASM loc_140FE seeds `a0` from the bottom populated row and `a1` from the
    // row that sits `Number_blank_top_rows` lower, so the copy starts from the
    // tail and leaves a blank strip above the original layout.
    // ASM loc_1413A performs the inner `move.w (a0)+,(a1)+` copy, with `d1`
    // seeded from `Level_width_blocks-1` so each shifted row copies exactly
    // one full row of layout words before the outer loop steps upward.
    // ASM loc_14138 reloads the per-row cell counter, copies one full row, then
    // steps both row bases upward and repeats until every populated row has
    // moved down by the requested blank-top-row count.
    // ASM loc_14156 then reloads the same per-row counter for each inserted
    // blank row, stepping `a1` upward after every row-sized zero-fill pass
    // until `Number_blank_top_rows` rows at the top are cleared.
    // ASM loc_14158 is the inner `move.w d4,(a1)+` loop inside that pass, with
    // `d4` kept at zero so each blank row stores exactly `Level_width_blocks`
    // zero words before the inclusive `dbf` exits.
    if width_blocks == 0 {
        return level_layout.to_vec();
    }

    let row_bytes = usize::from(width_blocks) * 2;
    if row_bytes == 0 || level_layout.is_empty() || !level_layout.len().is_multiple_of(row_bytes) {
        return level_layout.to_vec();
    }

    let blank_row_bytes = usize::from(blank_top_rows) * row_bytes;
    let mut shifted = vec![0_u8; blank_row_bytes + level_layout.len()];
    shifted[blank_row_bytes..].copy_from_slice(level_layout);
    shifted
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn insert_blank_top_rows_terrain_bytes_loc_141AA(
    terrain_layout: &[u8],
    width_blocks: u16,
    blank_top_rows: u16,
) -> Vec<u8> {
    // ASM loc_141AA (kid.asm:27003-27013): after the word-based level-layout
    // shift finishes, sub_140F6 repeats the same tail-first row move for the
    // byte-per-cell terrain layout rooted from the `$FFFF4BB8` row-pointer
    // table. `d1` reloads from `Level_width_blocks-1` before each row copy, so
    // the outer `dbf d0,loc_141AA` preserves the original row ordering while
    // shifting the populated terrain rows downward by `Number_blank_top_rows`.
    // ASM loc_141AC is the inner `move.b (a0)+,(a1)+` loop inside that pass,
    // with the inclusive `dbf` count again seeded from `Level_width_blocks-1`.
    // ASM loc_141C8 then reloads `d1` from the same cached row width before
    // each top-row clear, so the already shifted terrain bytes stay untouched
    // while the outer zero-fill loop walks exactly `Number_blank_top_rows`
    // inserted rows.
    // ASM loc_141CA is the inner `move.b d4,(a1)+` store inside that clear
    // pass, again using the inclusive `dbf` count so `Level_width_blocks=1`
    // still emits one zero byte per inserted row.
    if width_blocks == 0 {
        return terrain_layout.to_vec();
    }

    let row_bytes = usize::from(width_blocks);
    if row_bytes == 0
        || terrain_layout.is_empty()
        || !terrain_layout.len().is_multiple_of(row_bytes)
    {
        return terrain_layout.to_vec();
    }

    let blank_row_bytes = usize::from(blank_top_rows) * row_bytes;
    let mut shifted = vec![0_u8; blank_row_bytes + terrain_layout.len()];
    shifted[blank_row_bytes..].copy_from_slice(terrain_layout);
    shifted
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
    pub layout_addr: u16,
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
    pub layout_addr: u16,
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
    pub layout_addr: u16,
    pub col: u16,
    pub row: u16,
    /// The block type that gets revealed (Prize, Evanescent, Teleporter, etc.).
    pub block_type: u8,
    /// Block setting byte (direction for shooters/drills, side for teleporters).
    pub block_setting: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockRevealSubE3C6 {
    pub(crate) cell: Cell,
    pub(crate) sprite_mapping: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockPrizeRevealScanStartLocE3E4 {
    pub(crate) target_layout_addr: u16,
    pub(crate) preincrement_cursor_addr: u32,
    pub(crate) first_prize_record_addr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockPrizeRevealMatchLocE3EE {
    pub(crate) matched_record_addr: u32,
    pub(crate) scanned_records: usize,
    pub(crate) disguised: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HiddenBlockColumnUpdateSubE446 {
    pub(crate) layout_addr: u16,
    pub(crate) col: u16,
    pub(crate) row: u16,
    pub(crate) cell: Cell,
    pub(crate) sprite_mapping: u16,
    pub(crate) duration_frames: u16,
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
                layout_addr,
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
                layout_addr,
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
                layout_addr,
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

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn prize_block_by_layout_addr_loc_10342(
        &self,
        layout_addr: u16,
    ) -> Option<&PrizeBlockEntry> {
        // ASM loc_10342 (kid.asm:21588) advances `a1` by 8 bytes per prize
        // entry until `loc_10344` finds an `(a1)` layout-address match for the
        // worker's `$48(a5)` value, then consumes the matching entry's content.
        self.prize_blocks
            .iter()
            .find(|entry| entry.layout_addr == layout_addr)
    }

    /// Look up a teleporter at a given cell position.
    ///
    /// ASM loc_BC96 (kid.asm:15228): scan loop `move.w (a4),d4; cmp.w d4,d6;
    /// beq loc_BCB2; addi.w #$A,a4; dbf d7,loc_BC96`. The ROM matches by
    /// layout_addr (d6 = $FFFFFA86 telepad address, also d5 = d6-2 for the
    /// adjacent cell). Native matches by (col, row) which is equivalent since
    /// each block-list entry uniquely maps layout_addr ↔ (col, row).
    /// No match → loc_BCAC: `jmp (j_loc_6E2)` (lose life — unreachable in
    /// practice since the telepad entry was placed by the level designer).
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

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn hidden_block_by_layout_addr_sub_E37A(
        &self,
        layout_addr: u16,
    ) -> Option<&HiddenBlockEntry> {
        // ASM sub_E37A (kid.asm:17818) binary-searches the 8-byte hidden-block
        // list at `$FFFFF8EC` by the current Level_Layout address in `a2`.
        // ASM loc_E38E (kid.asm:17831): after the stride seed, `a0` is the
        // first entry, `a1` is one past `count*8`, and `a4` starts at `a0`.
        // ASM loc_E3A0 (kid.asm:17839): compare the target word from `a2`
        // with the current entry word and return only on an exact match.
        // ASM loc_E3B2 (kid.asm:17851): when the target is greater than the
        // current entry, advance by the current stride and clamp to `a1-8`.
        self.hidden_blocks
            .binary_search_by_key(&layout_addr, |entry| entry.layout_addr)
            .ok()
            .and_then(|index| self.hidden_blocks.get(index))
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn hidden_block_search_initial_stride_bytes_loc_E386(
        hidden_block_count: u16,
    ) -> u16 {
        // ASM loc_E386 (kid.asm:17824): starting from `d0=count-1` and
        // `d1=8`, halve d0 and double the byte stride until d0 shifts to zero.
        let mut remaining = hidden_block_count.wrapping_sub(1);
        let mut stride_bytes = 8u16;
        loop {
            remaining >>= 1;
            if remaining == 0 {
                return stride_bytes;
            }
            stride_bytes = stride_bytes.wrapping_mul(2);
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn hidden_block_reveal_sub_E3C6(
        &self,
        entry: &HiddenBlockEntry,
    ) -> Option<HiddenBlockRevealSubE3C6> {
        // ASM sub_E3C6 (kid.asm:17870) maps hidden-block metadata from `a4`
        // into the revealed Level_Layout cell word `d2` and sprite mapping
        // `d3`. Type 1 reuses the matching 8-byte prize entry from
        // `$FFFFF8D4`; the other reveal types come from `unk_E412/unk_E414`.
        if entry.block_type == 1 {
            let prize_match =
                self.hidden_block_prize_reveal_metadata_scan_loc_E3EE(0, entry.layout_addr)?;
            return Some(Self::hidden_block_prize_reveal_cell_loc_E40A(
                prize_match.disguised,
                entry.block_setting,
            ));
        }

        let (base_cell, sprite_mapping) = match entry.block_type {
            0x00 => (0xE001u16, 0x1058u16),
            0x02 => (0xE203u16, 0x1074u16),
            0x04 => (0xE407u16, 0x107Cu16),
            0x05 => (0xE50Bu16, 0x1084u16),
            0x06 => (0xE60Cu16, 0x1098u16),
            0x07 => (0xE710u16, 0x109Cu16),
            0x09 => (0xE919u16, 0x10A0u16),
            0x0A => (0xEA1Cu16, 0x10E4u16),
            0x0B => (0xEB2Cu16, 0x10A4u16),
            0x0C => (0xEC2Du16, 0x1084u16),
            _ => return None,
        };
        let cell_word = base_cell.wrapping_add(u16::from(entry.block_setting));
        Some(HiddenBlockRevealSubE3C6 {
            cell: Cell::from_be_bytes((cell_word >> 8) as u8, cell_word as u8),
            sprite_mapping,
        })
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn hidden_block_prize_reveal_scan_start_loc_E3E4(
        prize_block_list_base_addr: u32,
        entry: &HiddenBlockEntry,
    ) -> HiddenBlockPrizeRevealScanStartLocE3E4 {
        // ASM loc_E3E4 (kid.asm:17883): save `a0`, load the prize-block list
        // base from `$FFFFF8D4`, copy the current hidden-block layout word into
        // `d3`, then subtract 8 so the following loop starts on record 0.
        HiddenBlockPrizeRevealScanStartLocE3E4 {
            target_layout_addr: entry.layout_addr,
            preincrement_cursor_addr: prize_block_list_base_addr.wrapping_sub(8),
            first_prize_record_addr: prize_block_list_base_addr,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn hidden_block_prize_reveal_metadata_scan_loc_E3EE(
        &self,
        prize_block_list_base_addr: u32,
        layout_addr: u16,
    ) -> Option<HiddenBlockPrizeRevealMatchLocE3EE> {
        // ASM loc_E3EE (kid.asm:17889): advance through 8-byte prize records
        // until word 0 matches the hidden-block layout address already in `d3`.
        self.prize_blocks
            .iter()
            .position(|prize| prize.layout_addr == layout_addr)
            .map(|record_index| HiddenBlockPrizeRevealMatchLocE3EE {
                matched_record_addr: prize_block_list_base_addr + (record_index as u32 * 8),
                scanned_records: record_index + 1,
                disguised: self.prize_blocks[record_index].disguised,
            })
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn hidden_block_prize_reveal_cell_loc_E40A(
        disguised: bool,
        block_setting: u8,
    ) -> HiddenBlockRevealSubE3C6 {
        // ASM loc_E40A (kid.asm:17900): add the hidden-block setting byte to
        // `d2` with `add.b`, so only the low byte wraps; then restore `a0`.
        let (base_cell, sprite_mapping) = if disguised {
            (0xE101u16, 0x1058u16)
        } else {
            (0xE102u16, 0x1060u16)
        };
        let cell_word =
            (base_cell & 0xFF00) | u16::from((base_cell as u8).wrapping_add(block_setting));
        HiddenBlockRevealSubE3C6 {
            cell: Cell::from_be_bytes((cell_word >> 8) as u8, cell_word as u8),
            sprite_mapping,
        }
    }

    #[allow(non_snake_case)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn hidden_block_column_updates_sub_E446(
        &self,
        grid: &mut BlockGrid,
        col: u16,
        start_row: u16,
        row_count_inclusive: u16,
    ) -> Vec<HiddenBlockColumnUpdateSubE446> {
        // ASM sub_E446 (kid.asm:17963) walks one visible column in
        // Level_Layout, reveals every HiddenBlock cell it can resolve through
        // sub_E37A/sub_E3C6, writes the new cell back immediately, and queues
        // a `$12C`-frame object plus a foreground plane upload for the same
        // location.
        let mut updates = Vec::new();
        for offset in 0..=row_count_inclusive {
            let row = start_row.saturating_add(offset);
            if row >= grid.height_cells || col >= grid.width_cells {
                break;
            }

            let layout_addr =
                (((u32::from(row) * u32::from(grid.width_cells)) + u32::from(col)) * 2) as u16;
            if grid.cell_at(col, row).collision != CollisionKind::HiddenBlock {
                continue;
            }
            let Some(entry) = self.hidden_block_by_layout_addr_sub_E37A(layout_addr) else {
                continue;
            };
            let Some(reveal) = self.hidden_block_reveal_sub_E3C6(entry) else {
                continue;
            };

            grid.set_cell_at(col, row, reveal.cell);
            updates.push(HiddenBlockColumnUpdateSubE446 {
                layout_addr,
                col,
                row,
                cell: reveal.cell,
                sprite_mapping: reveal.sprite_mapping,
                duration_frames: 0x012C,
            });
        }
        updates
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
/// - `SlopeUp`: floor rises left-to-right (higher on the left, lower on the right). `/`
/// - `SlopeDown`: floor falls left-to-right (lower on the left, higher on the right). `\`
///   Public wrapper for slope_floor_y used by movement.rs sub_9386 Y-snap.
pub fn slope_floor_y_pub(collision: CollisionKind, cell_top_y: i32, x_px: i32) -> i32 {
    slope_floor_y(collision, cell_top_y, x_px)
}

fn slope_floor_y(collision: CollisionKind, cell_top_y: i32, x_px: i32) -> i32 {
    let x_within_cell = (x_px & 0x0F).clamp(0, 15);
    match collision {
        // ASM `sub_9386` / `sub_A254`: `$4000` stores `x + y`, so the
        // surface rises as X increases.
        // y = cell_top + 15 - x_within_cell
        CollisionKind::SlopeUp => cell_top_y + 15 - x_within_cell,
        // ASM `sub_9386` / `sub_A254` loc_A268 (kid.asm:12563): `$5000`
        // stores `y - x`, so the surface falls as X increases.
        // loc_A268: `move.w ($FA24).w,d0; add.w x_pos(a3),d0;
        // move.w d0,y_pos(a3)` — SlopeDown adds x_pos to the constant.
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
///
/// ASM loc_ABF0 (kid.asm:13449): floor scan loop in kid_flip pull-up —
/// `add.w (Level_width_tiles),a4; addi.w #$10,d6; move.w (a4),d7;
/// andi.w #$7000,d7; cmpi.w #$6000,d7; blt.s loc_ABF0`.  Walks down
/// tile rows until a solid cell is found.  Native: same row-walk with
/// `is_solid_floor()` instead of the $7000/$6000 bitmask test.
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

/// Mirror ROM `sub_922C`'s single-column wall scan.
///
/// Unlike `probe_wall`, this does not sample three body points. It probes the
/// exact edge column at `half_width + 1` and scans every 16px row from the
/// player's feet upward through the requested body height.
///
/// ASM loc_9288 (kid.asm:10693): height adjustment merge point. Crawling,
/// Micromax, and Juggernaut skip `subi.w #$10,d7` (extra 16px head clearance).
/// Then `asr.w #4,d7; sub.w d7,d5` converts head pixel row to tile offset and
/// computes the scan row count. Native: caller passes the adjusted `height`.
///
/// ASM loc_9294 (kid.asm:10699): scan loop body. `move.w (a4),d7; andi.w
/// #$7000,d7` extracts collision bits. `$6000` (Solid) → loc_92BC hit handler.
/// `$7000` (Spikes) → loc_92B6 sets `collision_type=$28` then falls through to
/// loc_92BC. Native: `matches!(cell.collision, Solid | Spikes)`.
///
/// ASM loc_92AA (kid.asm:10707): loop continuation. `suba.w Level_width_tiles,
/// a4` moves up one row. `dbf d5,loc_9294` continues; on exhaustion `bra.w
/// loc_9318` evaluates hit/Rubber/Berzerker flags. Native: `for row in
/// (top_row..=foot_row).rev()`.
///
/// ASM loc_92DE (kid.asm:10729): Berzerker gate for non-Rubber block
/// interaction. After loc_92BC sets the hit flag and detects a block that is
/// NOT Rubber ($18), `tst.b (Berzerker_charging); beq.s loc_92AA` skips if
/// not charging. `cmpi.w #MoveID_Jump,(Character_Movement); beq.s loc_92AA`
/// skips if jumping. Only a charging, non-jumping Berzerker reaches loc_92EC.
/// TODO(fidelity): native dispatches block interactions at the caller level
/// without this gate — any helmet can trigger Smash/Ice block effects on wall
/// contact.
///
/// ASM loc_92EC (kid.asm:10735): block interaction dispatch. Saves registers,
/// loads handler from `off_933E[block_type*4]` (`lea off_933E(pc),a2;
/// move.l (a2,d7.w),a2`), calls `j_Get_XY_From_LevelLayoutAddress`, then
/// checks `tst.l x_vel(a3); bpl.w loc_930E; moveq #1,d6` (d6=3 right, d6=1
/// left for the handler's direction arg). Native: caller reads
/// `probe.cell.block_type` after the scan.
///
/// ASM loc_930E (kid.asm:10747): `jsr (a2)` executes the block handler, then
/// `movem.w (sp)+,d0-d7; move.l (sp)+,a4; bra.s loc_92AA` restores state and
/// continues scanning. The ROM processes ALL blocks in the column; native
/// returns on the first Solid/Spikes hit.
///
/// ASM loc_9326 (kid.asm:10762): hit-with-no-rubber return. `tst.b 1(sp)`
/// confirms hit flag set, then `sf (Berzerker_charging)` clears the charge
/// before returning `moveq #1,d7` (NE = wall found). Native: probe returns
/// `hit: true`; berzerker_charging is managed separately.
///
/// ASM loc_9338 (kid.asm:10771): no-hit return. Both stack flags clear →
/// `moveq #0,d7; addq.w #2,sp; rts` (EQ = no wall). Native: probe returns
/// `hit: false`.
///
/// ASM loc_9380 (kid.asm:10799): sub_922C OOB column guard. Entry converts
/// the probe x coordinate to a column via `asr.w #4,d7`: if negative (`bmi.w
/// loc_9380`) or greater than `Level_width_tiles` (`bgt.w loc_9380`), jumps
/// to `moveq #1,d7; addq.w #2,sp; rts` — returning NE (hit=true, treat as
/// wall). TODO(fidelity): native returns `hit: false` for OOB via the
/// `edge_x < 0 || edge_x >= grid.width_pixels()` guard.
pub fn probe_wall_column(
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
        center_x_px - half_width - 1
    };

    if edge_x < 0 || edge_x >= grid.width_pixels() {
        return WallProbe {
            hit: false,
            wall_x_px: 0,
            cell: Cell::from_be_bytes(0, 0),
            cell_x: 0,
            cell_y: 0,
        };
    }

    let col = pixel_to_cell(edge_x).clamp(0, i32::from(grid.width_cells).saturating_sub(1)) as u16;
    let top_row = pixel_to_cell(foot_y_px - height).max(0) as u16;
    let foot_row =
        pixel_to_cell(foot_y_px).clamp(0, i32::from(grid.height_cells).saturating_sub(1)) as u16;

    for row in (top_row..=foot_row).rev() {
        let cell = grid.cell_at(col, row);
        if matches!(cell.collision, CollisionKind::Solid | CollisionKind::Spikes) {
            let wall_x = if moving_right {
                i32::from(col) * 16
            } else {
                (i32::from(col) + 1) * 16
            };
            return WallProbe {
                hit: true,
                wall_x_px: wall_x,
                cell,
                cell_x: col,
                cell_y: row,
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

/// Mirror Iron Knight wall-climb resume lookup `sub_B55C` at `loc_A30A`.
///
/// The ROM does not reuse the generic three-point wall scan once climb is
/// active. It samples exactly one cell at `half_width + 1` in the facing
/// direction and 24px above the feet, then requires the collision nibble to be
/// `0x6000` (solid) to keep climb state alive.
pub fn probe_iron_knight_climb_resume_wall(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    half_width: i32,
    moving_right: bool,
) -> WallProbe {
    let edge_x = if moving_right {
        center_x_px + half_width + 1
    } else {
        center_x_px - half_width - 1
    };
    let probe = probe_at(grid, edge_x, foot_y_px - 24);
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

/// Mirror the single-cell relative lookup in `sub_B55C`.
///
/// The ROM adds `d7` to `x_pos(a3)` and `d6` to `y_pos(a3)`, converts those
/// pixel coordinates into a block-row pointer plus cell offset, and `loc_331D2`
/// returns the raw collision nibble for exactly that cell.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn relative_collision_mask_loc_331D2(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    x_offset_px: i32,
    y_offset_px: i32,
) -> u16 {
    let sample_x = center_x_px + x_offset_px;
    let sample_y = foot_y_px + y_offset_px;
    if sample_x < 0 || sample_y < 0 {
        return 0;
    }
    probe_at(grid, sample_x, sample_y).cell.to_word() & 0x7000
}

/// Mirror the final collision-nibble mask in `loc_33280`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn relative_collision_nibble_loc_33280(tile_word: u16) -> u16 {
    // ASM loc_33280 (kid.asm:37690): the third sampled word skips any further
    // block-type tests here and keeps only `andi.w #$7000,d7`.
    tile_word & 0x7000
}

fn relative_collision_word_mask_shared_disguised_block(tile_word: u16) -> u16 {
    if (tile_word & 0x8000) != 0 && (tile_word & 0x0F00) == 0x0300 {
        0
    } else {
        relative_collision_nibble_loc_33280(tile_word)
    }
}

/// Mirror the disguised-block filter shared by `loc_331F6`, `loc_33248`, and
/// the later horizontal scan at `loc_33286`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn relative_collision_word_mask_loc_33248(tile_word: u16) -> u16 {
    // ASM loc_33248 (kid.asm:37670): `bclr #$F,dN` only enters the block-type
    // check when bit 15 was set. A surviving `$300` block-type nibble means a
    // disguised solid-ghost block, so the helper zeroes the word before
    // keeping only the collision nibble.
    relative_collision_word_mask_shared_disguised_block(tile_word)
}

/// Mirror the middle-word disguised-block filter in `loc_33264`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn relative_collision_word_mask_loc_33264(tile_word: u16) -> u16 {
    // ASM loc_33264 (kid.asm:37680): after `bclr #$F,d6`, the second sampled
    // word follows the same disguised-block gate as `loc_33248` before falling
    // through to the shared collision-nibble mask.
    relative_collision_word_mask_shared_disguised_block(tile_word)
}

/// Mirror the first horizontal-word disguised-block filter in `loc_332D0`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn relative_collision_word_mask_loc_332D0(tile_word: u16) -> u16 {
    // ASM loc_332D0 (kid.asm:37723): after `bclr #$F,d5`, the horizontal scan's
    // first sampled word uses the same disguised-block gate before falling
    // through to the collision-nibble mask.
    relative_collision_word_mask_shared_disguised_block(tile_word)
}

/// Mirror the second horizontal-word disguised-block filter in `loc_332EC`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn relative_collision_word_mask_loc_332EC(tile_word: u16) -> u16 {
    // ASM loc_332EC (kid.asm:37733): after `bclr #$F,d6`, the second
    // horizontal-scan word applies the same disguised-block gate before
    // falling through to the shared collision-nibble mask.
    relative_collision_word_mask_shared_disguised_block(tile_word)
}

/// Mirror the final horizontal-word collision mask in `loc_33308`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn relative_collision_nibble_loc_33308(tile_word: u16) -> u16 {
    // ASM loc_33308 (kid.asm:37743): after the `d7` disguised-block gate at
    // loc_332EC, the horizontal scan ends with `andi.w #$7000,d7`.
    relative_collision_nibble_loc_33280(tile_word)
}

/// Mirror the single-cell relative lookup in `loc_3330E`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn relative_collision_mask_loc_3330E(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    x_offset_px: i32,
    y_offset_px: i32,
) -> u16 {
    // ASM loc_3330E (kid.asm:37750): add `d6/d7` onto `x_pos/y_pos(a3)`,
    // derive the row pointer through `$FFFF4A04`, fetch one Level_Layout word,
    // and keep only `andi.w #$7000,d7`.
    relative_collision_mask_loc_331D2(grid, center_x_px, foot_y_px, x_offset_px, y_offset_px)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RelativeCollisionColumnLoc331F6 {
    pub(crate) current_row: u16,
    pub(crate) one_row_up: u16,
    pub(crate) two_rows_up: u16,
}

/// Mirror the three-row relative probe in `loc_331F6`.
#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn relative_collision_column_loc_331F6(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    x_offset_px: i32,
    sentinel_d4: u16,
) -> RelativeCollisionColumnLoc331F6 {
    let sample_x = center_x_px + x_offset_px;
    let sample_y = foot_y_px + 14;
    if sample_x < 0 || sample_y < 0 {
        return RelativeCollisionColumnLoc331F6 {
            current_row: 0,
            one_row_up: 0,
            two_rows_up: 0,
        };
    }

    let probe = probe_at(grid, sample_x, sample_y);
    let cell_x = probe.cell_x;
    let cell_y = probe.cell_y;
    let current_row = probe.cell.to_word();
    let one_row_up = if cell_y == 0 {
        0
    } else {
        grid.cell_at(cell_x, cell_y - 1).to_word()
    };
    let two_rows_up = if cell_y <= 1 {
        0
    } else {
        grid.cell_at(cell_x, cell_y - 2).to_word()
    };

    if sentinel_d4 == 0x03E7 {
        RelativeCollisionColumnLoc331F6 {
            current_row,
            one_row_up,
            two_rows_up,
        }
    } else {
        RelativeCollisionColumnLoc331F6 {
            current_row: relative_collision_word_mask_loc_33248(current_row),
            one_row_up: relative_collision_word_mask_loc_33264(one_row_up),
            two_rows_up: relative_collision_nibble_loc_33280(two_rows_up),
        }
    }
}

#[allow(non_snake_case)]
pub fn probe_relative_collision_sub_B55C(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    x_offset_px: i32,
    y_offset_px: i32,
) -> CollisionKind {
    match relative_collision_mask_loc_331D2(grid, center_x_px, foot_y_px, x_offset_px, y_offset_px)
    {
        0x2000 => CollisionKind::HiddenBlock,
        0x4000 => CollisionKind::SlopeUp,
        0x5000 => CollisionKind::SlopeDown,
        0x6000 => CollisionKind::Solid,
        0x7000 => CollisionKind::Spikes,
        _ => CollisionKind::Empty,
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

fn sets_collision_bit_14(cell: Cell) -> bool {
    matches!(
        cell.collision,
        CollisionKind::SlopeUp
            | CollisionKind::SlopeDown
            | CollisionKind::Solid
            | CollisionKind::Spikes
    )
}

#[allow(non_snake_case)]
fn horizontal_collision_scan_column_mask_loc_369CE(
    grid: &BlockGrid,
    column: i32,
    top_row: i32,
    bottom_row: i32,
) -> u16 {
    // ASM loc_369CE (kid.asm:38145): OR the lead-column words down the scanned
    // row span by stepping one `Level_width_tiles` row at a time, then test
    // bit `$4000` on the accumulated mask.
    let mut mask = 0u16;
    for row in top_row..=bottom_row {
        mask |= grid.cell_at(column as u16, row as u16).to_word();
    }
    mask
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_right_lead_column_mask_loc_36BB2(
    grid: &BlockGrid,
    column: i32,
    top_row: i32,
    bottom_row: i32,
) -> u16 {
    // ASM loc_36BB2 (kid.asm:38410): OR the in-bounds right lead-column words
    // into `d6`, advancing by `Level_width_tiles` until the inclusive `dbf d0`
    // loop has consumed the full vertical span.
    horizontal_collision_scan_column_mask_loc_369CE(grid, column, top_row, bottom_row)
}

#[allow(non_snake_case)]
fn horizontal_collision_scan_hitbox_mask_seed_loc_369F6() -> u16 {
    // ASM loc_369F6 (kid.asm:38167): `moveq #0,d5` clears the rectangle-scan
    // accumulator before the nested body sweep starts.
    0
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_rect_mask_loc_369F8(
    grid: &BlockGrid,
    left_col: i32,
    right_col: i32,
    top_row: i32,
    bottom_row: i32,
) -> u16 {
    // ASM loc_369F8 / loc_369FC (kid.asm:38170 / 38174): reset `a4` to each
    // row start, then `loc_369FC` walks `or.w (a4)+,d5` across that row before
    // advancing one `Level_width_tiles` row and repeating until the whole
    // rectangle has been reduced into `d5`.
    let mut mask = horizontal_collision_scan_hitbox_mask_seed_loc_369F6();
    for row in top_row..=bottom_row {
        for col in left_col..=right_col {
            mask |= grid.cell_at(col as u16, row as u16).to_word();
        }
    }
    mask
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_right_hitbox_mask_loc_36BBE(
    grid: &BlockGrid,
    left_col: i32,
    right_col: i32,
    top_row: i32,
    bottom_row: i32,
) -> u16 {
    // ASM loc_36BBE (kid.asm:38416): reset `a4` to the start of the current
    // hitbox row and reload `d0=d3` before each row-width scan.
    let mut mask = horizontal_collision_scan_hitbox_mask_seed_loc_369F6();
    for row in top_row..=bottom_row {
        mask |= horizontal_collision_scan_right_hitbox_row_mask_loc_36BC2(
            grid, left_col, right_col, row,
        );
    }
    mask
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_right_hitbox_row_mask_loc_36BC2(
    grid: &BlockGrid,
    left_col: i32,
    right_col: i32,
    row: i32,
) -> u16 {
    // ASM loc_36BC2 (kid.asm:38420): OR each word in the current hitbox row
    // into `d5` via post-increment `(a4)+`, stopping only after the inclusive
    // `dbf d0` width counter expires.
    let mut mask = 0u16;
    for col in left_col..=right_col {
        mask |= grid.cell_at(col as u16, row as u16).to_word();
    }
    mask
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_following_row_class_loc_36A10(words: &[u16]) -> i16 {
    // ASM loc_36A10 (kid.asm:38183): after the body rectangle scan, inspect the
    // following row from left to right. If the first cell already carries bit
    // `$4000`, stay on the “all blocked until proven otherwise” path and leave
    // `d6=0` only when every scanned word keeps bit `$4000` set.
    if words.is_empty() {
        return -1;
    }

    if words[0] & 0x4000 != 0 {
        for &word in words {
            // ASM loc_36A20 (kid.asm:38193): once the entry word is blocked,
            // keep advancing with `move.w (a1)+,d0`; the first later word that
            // clears bit `$4000` flips the mixed-row result to `d6=1`.
            if word & 0x4000 == 0 {
                // ASM loc_36A36 (kid.asm:38206): both mixed-row scans converge
                // here and tag the following row as mixed by returning `d6=1`.
                return 1;
            }
        }
        // ASM loc_36A2A (kid.asm:38199): when the inclusive `dbf d3,loc_36A20`
        // scan finds no clear cell, the routine preserves the blocked-row class
        // by returning with `d6=0`.
        0
    } else if words.iter().any(|word| {
        // ASM loc_36A40 (kid.asm:38213): when the entry word is clear, keep
        // advancing with `move.w (a1)+,d0` until a later cell sets bit
        // `$4000`, then branch to the shared mixed-row exit at `loc_36A36`.
        word & 0x4000 != 0
    }) {
        1
    } else {
        // ASM loc_36A4A (kid.asm:38219): if the clear-entry scan reaches the
        // inclusive `dbf d3,loc_36A40` tail without ever seeing bit `$4000`,
        // the routine marks the following row as fully clear with `d6=-1`.
        -1
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_left_lead_column_index_loc_36AA6(left_col: i32) -> Option<i32> {
    // ASM loc_36AA6 (kid.asm:38263): once `tst.w d0` proves the hitbox is not
    // already on the level's left edge, `lea -2(a1),a4` retargets the scan to
    // the lead column immediately left of the hitbox. A zero column offset
    // skips this path and falls back to the boundary result at `loc_36AD6`.
    if left_col == 0 {
        None
    } else {
        Some(left_col - 1)
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_left_lead_column_mask_loc_36AAE(
    grid: &BlockGrid,
    lead_col: i32,
    top_row: i32,
    bottom_row: i32,
) -> u16 {
    // ASM loc_36AAE (kid.asm:38268): with `a4` already retargeted by
    // `loc_36AA6`, OR the left lead-column words one row at a time via
    // `add.w (Level_width_tiles).w,a4` so any blocking bit from the scanned
    // prefix survives into the later `andi.w #$4000,d0` test.
    // ASM loc_36C46 (kid.asm:38486): Robot/Driller/Tank's `sub_36BD6`
    // reuses the same left lead-column loop for its object body, OR-ing
    // `(a4)` into `d6` before advancing by one `Level_width_tiles` row.
    let mut mask = 0u16;
    for row in top_row..=bottom_row {
        mask |= grid.cell_at(lead_col as u16, row as u16).to_word();
    }
    mask
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_left_following_row_class_loc_36AF0(words: &[u16]) -> i16 {
    // ASM loc_36AF0 (kid.asm:38306): after the leftward body scan, retarget
    // `a1` to the rightmost word of the following row and seed `d6=0`. The
    // first `move.w (a1),d0` decides whether the reverse scan starts on the
    // blocked-entry path or the clear-entry path.
    if words.is_empty() {
        return -1;
    }

    let rightmost_word = *words.last().expect("checked empty slice above");
    if rightmost_word & 0x4000 != 0 {
        for &word in words.iter().rev() {
            // ASM loc_36B04 (kid.asm:38318): the blocked-entry path then walks
            // right-to-left with `move.w -(a1),d0`, so the first earlier word
            // that clears bit `$4000` flips the row class to mixed.
            if word & 0x4000 == 0 {
                // ASM loc_36B1A (kid.asm:38331): the mixed-row exit loads
                // `d6=1` before returning.
                return 1;
            }
        }
        // ASM loc_36B0E (kid.asm:38324): if the inclusive reverse `dbf`
        // finishes without finding a clear word, the routine falls through the
        // shared epilogue and preserves the seeded `d6=0`.
        0
    } else {
        for &word in words.iter().rev() {
            // ASM loc_36B24 (kid.asm:38338): the clear-entry path pre-decrements
            // across the following row until an earlier word sets bit `$4000`,
            // then branches to the shared mixed-row exit at `loc_36B1A`.
            if word & 0x4000 != 0 {
                return 1;
            }
        }
        // ASM loc_36B2E (kid.asm:38344): once the clear-entry reverse scan
        // exhausts the inclusive `dbf d3,loc_36B24` loop without finding bit
        // `$4000`, it loads `d6=-1` for a fully clear following row.
        -1
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_left_following_lead_cell_blocked_loc_36ACE(word: u16) -> bool {
    let collision_nibble = word & 0x7000;

    // ASM loc_36ACE (kid.asm:38285): after the exact-`$4000` check fails, keep
    // only bit `$4000`; mixed kinds such as `$5000`/`$6000` still block, while
    // clear words fall through to the shared exit unchanged.
    collision_nibble & 0x4000 != 0
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_left_following_lead_cell_blocked_loc_36ABE(word: u16) -> bool {
    let collision_nibble = word & 0x7000;

    // ASM loc_36ABE (kid.asm:38276): after the inclusive lead-column scan
    // leaves `a4` on the following row, an exact `$4000` nibble takes the fast
    // path that increments `d7` before returning to the shared exit.
    collision_nibble == 0x4000
        || horizontal_collision_scan_left_following_lead_cell_blocked_loc_36ACE(collision_nibble)
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_left_hitbox_mask_loc_36AD8(
    grid: &BlockGrid,
    left_col: i32,
    right_col: i32,
    top_row: i32,
    bottom_row: i32,
) -> u16 {
    // ASM loc_36AD8 / loc_36ADC (kid.asm:38293 / 38296): reset `a4` to each
    // row start from `a1`, OR every word across the current row, then advance
    // `a1` by one `Level_width_tiles` row and repeat until the full leftward
    // hitbox rectangle has been reduced into `d5`.
    // ASM loc_36C52 (kid.asm:38492): `sub_36BD6` uses the same row restart,
    // copying `a1` into `a4` and reloading `d0=d3` before the per-row OR loop.
    let mut mask = horizontal_collision_scan_hitbox_mask_seed_loc_369F6();
    for row in top_row..=bottom_row {
        mask |= horizontal_collision_scan_left_hitbox_row_mask_loc_36C56(
            grid, left_col, right_col, row,
        );
    }
    mask
}

#[allow(non_snake_case)]
#[cfg_attr(not(test), allow(dead_code))]
fn horizontal_collision_scan_left_hitbox_row_mask_loc_36C56(
    grid: &BlockGrid,
    left_col: i32,
    right_col: i32,
    row: i32,
) -> u16 {
    // ASM loc_36C56 (kid.asm:38496): after `loc_36C52` resets `a4`, OR each
    // word in the current hitbox row into `d5` via post-increment `(a4)+`
    // until the inclusive `dbf d0` width counter expires.
    let mut mask = 0u16;
    for col in left_col..=right_col {
        mask |= grid.cell_at(col as u16, row as u16).to_word();
    }
    mask
}

pub(crate) fn transform_overlap_column_blocked_sub_7bea(
    grid: &BlockGrid,
    x_px: i32,
    foot_y_px: i32,
    rows_up: u16,
) -> bool {
    // ASM sub_7BEA (kid.asm:8317): start from the row containing `y_pos(a3)`,
    // walk upward one row per `suba.w Level_width_tiles,a4`, and stop as soon
    // as any scanned cell carries bit `$4000`.
    if x_px < 0 || foot_y_px < 0 {
        return false;
    }

    let column = pixel_to_cell(x_px);
    if column < 0 || column >= i32::from(grid.width_cells) {
        return false;
    }

    let mut row = pixel_to_cell(foot_y_px);
    if row < 0 {
        return false;
    }

    // ASM loc_7C02 (kid.asm:8327): `move.w (a4),d6; andi.w #$4000,d6;
    // bne.w loc_7C18; suba.w (Level_width_tiles).w,a4; dbf d7,loc_7C02`
    // — scan loop: read tile, check wall bit ($4000), move up one row, repeat.
    for _ in 0..=rows_up {
        if row >= 0
            && row < i32::from(grid.height_cells)
            && sets_collision_bit_14(grid.cell_at(column as u16, row as u16))
        {
            // ASM loc_7C18 (kid.asm:8337): `moveq #1,d7; rts` — wall found,
            // return blocked (non-zero d7 = column is obstructed).
            return true;
        }
        row -= 1;
    }

    // ASM sub_7BEA exit (kid.asm:8333): `moveq #0,d7; rts` — all rows clear.
    false
}

fn horizontal_collision_scan(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    half_width: i32,
    height: i32,
    leading_right: bool,
) -> HorizontalCollisionScan {
    // ASM loc_36B5A / loc_36BF6 (kid.asm:38369 / 38447): after clamping the
    // bottom row against `Level_height_blocks`, derive the inclusive row span
    // and base row pointer for the hitbox and lead-column scans.
    let top_row = pixel_to_cell(foot_y_px - height).max(0);
    let mut bottom_row = pixel_to_cell(foot_y_px - 1).max(0);
    if bottom_row >= i32::from(grid.height_cells) {
        bottom_row = i32::from(grid.height_cells).saturating_sub(1);
    }
    if bottom_row < top_row {
        let lead_column_blocked = if leading_right {
            pixel_to_cell(center_x_px + half_width - 1) + 1 >= i32::from(grid.width_cells)
        } else {
            pixel_to_cell(center_x_px - half_width) <= 0
        };
        return HorizontalCollisionScan {
            hitbox_blocked: false,
            lead_column_blocked,
        };
    }

    let left_col = pixel_to_cell(center_x_px - half_width).max(0);
    let mut right_col = pixel_to_cell(center_x_px + half_width - 1).max(0);
    if right_col >= i32::from(grid.width_cells) {
        right_col = i32::from(grid.width_cells).saturating_sub(1);
    }

    let hitbox_mask = if leading_right {
        horizontal_collision_scan_right_hitbox_mask_loc_36BBE(
            grid, left_col, right_col, top_row, bottom_row,
        )
    } else {
        horizontal_collision_scan_left_hitbox_mask_loc_36AD8(
            grid, left_col, right_col, top_row, bottom_row,
        )
    };
    let hitbox_blocked = hitbox_mask & 0x4000 != 0;

    let lead_column_blocked = if leading_right {
        let lead_col = right_col + 1;
        // ASM loc_36BA0 (kid.asm:38398): compare the right lead-column index
        // against `Level_width_blocks`; equality means the scan has reached the
        // level boundary and returns `d6=1` without reading a map word.
        if lead_col >= i32::from(grid.width_cells) {
            true
        } else {
            // ASM loc_36BAA / loc_36BB2 (kid.asm:38405 / 38410): retarget
            // `a4` to the in-bounds right lead column, OR one word per scanned
            // row via `add.w d4,a4`, then keep only bit `$4000` for the lead
            // result.
            // ASM loc_369C4 (kid.asm:38139): once the right lead column is still
            // in bounds, `loc_369CE` reduces that column to one OR mask before
            // the later `loc_369DE/loc_369EE` slope/solid checks decide whether
            // the side contact blocks.
            horizontal_collision_scan_right_lead_column_mask_loc_36BB2(
                grid, lead_col, top_row, bottom_row,
            ) & 0x4000
                != 0
        }
    } else if left_col == 0 {
        // ASM loc_36C38 (kid.asm:38475): when the doubled left-column offset
        // is zero, set `d6=1` and skip the `-2(a1)` lead-column read.
        true
    } else {
        let lead_col = horizontal_collision_scan_left_lead_column_index_loc_36AA6(left_col)
            .expect("leftward lead-column scan should only run when the column is in bounds");
        // ASM loc_36C40 (kid.asm:38482): retarget `a4` to `-2(a1)`, then OR
        // exactly the clamped `d2` row span by stepping one `Level_width_tiles`
        // row at a time. Unlike the earlier left scan, this path does not
        // inspect the following row below the hitbox.
        horizontal_collision_scan_left_lead_column_mask_loc_36AAE(
            grid, lead_col, top_row, bottom_row,
        ) & 0x4000
            != 0
    };

    HorizontalCollisionScan {
        hitbox_blocked,
        lead_column_blocked,
    }
}

/// Mirror Robot/Driller/Tank rightward rectangle scan `sub_36B3C`.
#[allow(non_snake_case)]
pub fn horizontal_collision_scan_right_sub_36B3C(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    half_width: i32,
    height: i32,
) -> HorizontalCollisionScan {
    // ASM sub_36B3C (kid.asm:38356): scan the current hitbox rectangle for any
    // bit-$4000 collision cells, then scan the lead column immediately to the
    // right over the same row span, treating the level's right boundary as
    // blocked.
    horizontal_collision_scan(grid, center_x_px, foot_y_px, half_width, height, true)
}

/// Mirror Robot/Driller/Tank leftward rectangle scan `sub_36BD6`.
#[allow(non_snake_case)]
pub fn horizontal_collision_scan_left_sub_36BD6(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    half_width: i32,
    height: i32,
) -> HorizontalCollisionScan {
    // ASM sub_36BD6 (kid.asm:38434): scan the current hitbox rectangle for any
    // bit-$4000 collision cells, then scan the lead column immediately to the
    // left over the same row span, treating column -1 / the level's left edge
    // as blocked.
    horizontal_collision_scan(grid, center_x_px, foot_y_px, half_width, height, false)
}

fn vertical_collision_scan_row(
    grid: &BlockGrid,
    center_x_px: i32,
    half_width: i32,
    row: i32,
) -> bool {
    if row < 0 {
        return false;
    }
    if row >= i32::from(grid.height_cells) {
        return false;
    }

    let mut left_col = pixel_to_cell(center_x_px - half_width);
    let mut right_col = pixel_to_cell(center_x_px + half_width - 1);
    if right_col < 0 || left_col >= i32::from(grid.width_cells) {
        return false;
    }
    left_col = left_col.max(0);
    right_col = right_col.min(i32::from(grid.width_cells).saturating_sub(1));
    if right_col < left_col {
        return false;
    }

    // ASM loc_36CAA (kid.asm:38539) / loc_36D00 (kid.asm:38586): OR `(a0)+`
    // for `d5 + 1` cells with DBF, so the first matching bit-$4000 word
    // anywhere in the inclusive span survives until the final
    // `andi.w #$4000,d6`.
    (left_col..=right_col).any(|col| sets_collision_bit_14(grid.cell_at(col as u16, row as u16)))
}

/// Mirror Robot/Driller/Tank downward row scan `sub_36C6A`.
#[allow(non_snake_case)]
pub fn vertical_collision_scan_down_sub_36C6A(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    half_width: i32,
) -> bool {
    // ASM sub_36C6A (kid.asm:38510): derive the row immediately below
    // `y_pos(a3)-1`, reject any row at or beyond `Level_height_blocks`, then
    // OR every cell under the current width span and keep only bit `$4000`.
    // ASM loc_36C82 (kid.asm:38523): after the row is accepted, load that
    // row's `$FFFF4A04` pointer and seed the inclusive left/right cell span
    // from `x_pos(a3) +/- $48(a5)`.
    let row = pixel_to_cell(foot_y_px - 1) + 1;
    vertical_collision_scan_row(grid, center_x_px, half_width, row)
}

/// Mirror Robot/Driller/Tank upward row scan `sub_36CB8`.
#[allow(non_snake_case)]
pub fn vertical_collision_scan_up_sub_36CB8(
    grid: &BlockGrid,
    center_x_px: i32,
    foot_y_px: i32,
    half_width: i32,
    height: i32,
) -> bool {
    // ASM sub_36CB8 (kid.asm:38551): derive the row immediately above the top
    // edge `y_pos(a3)-height`, exit early if it is negative, then OR the
    // current width span and keep only bit `$4000`.
    // ASM loc_36CCC (kid.asm:38562): after the negative-row `bpl` gate, rows
    // at or beyond `Level_height_blocks` clear `d6` and return before loading
    // a row pointer.
    // ASM loc_36CD8 (kid.asm:38570): after the row survives both gates, load
    // the `$FFFF4A04` row pointer and seed the inclusive span from
    // `(x_pos-half_width)>>4` through `(x_pos+half_width-1)>>4`.
    let row = pixel_to_cell(foot_y_px - height) - 1;
    vertical_collision_scan_row(grid, center_x_px, half_width, row)
}

/// Check for a ceiling above the player's hitbox.
///
/// ASM sub_914A (kid.asm:10559): scans ALL tiles across the player's width
/// at the ceiling row. The loop at loc_9188 reads each tile word, extracts
/// collision bits (`andi.w #$7000,d7`), and branches on:
///   - `$6000` (Solid) → loc_91AC hit handler
///   - `$7000` (Spikes) → loc_91A6 (`move.w #colid_kidbelow,collision_type`)
///     then falls through to loc_91AC
///
/// loc_919E (`dbf d5,loc_9188`) continues the loop; on exhaustion, falls
/// through to loc_9204 which evaluates the hit/Rubber flags:
///   - loc_9204: `_tst.b 0(sp)` — rubber bounce flag set → d7=-1 (bounce)
///   - loc_9212: `tst.b 1(sp)` — no rubber, check hit flag:
///     - 1(sp) set → d7=1 (normal ceiling hit)
///     - 1(sp) clear → loc_9220: d7=0 (no hit)
///
/// loc_9226: early exit when head row is negative (above level top):
/// `asr.w #4,d7; bmi.w loc_9226` → d7=1 (treat as ceiling hit).
/// TODO(fidelity): native returns no-hit for OOB rows instead.
///
/// loc_91AC sets the hit flag (`st 1(sp)`) and records the first-hit tile
/// address in `$FFFFFB6C`. It then checks for Rubber blocks at loc_91C0
/// (`andi.w #$F00; asr.w #6; cmpi.w #$18` = block_type 6) and dispatches
/// the rubber jiggle effect via `j_loc_10DA4`, setting the Rubber flag
/// (`_st 0(sp)`) so sub_914A returns d7=-1 (bounce) instead of d7=1 (stop).
///
/// Native simplification: probes only the two horizontal edges (not the
/// full tile span), checks `Solid | HiddenBlock` (not Spikes), and returns
/// on the first hit. The full-width scan and Spikes detection are
/// TODO(fidelity).
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

    fn make_empty_grid(width: u16, height: u16) -> BlockGrid {
        BlockGrid::from_level_layout(
            &vec![0u8; usize::from(width) * usize::from(height) * 2],
            width,
            height,
        )
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
    fn juggernaut_wall_scan_sub_96e8_uses_asm_right_facing_probe_bounds() {
        let mut grid = make_empty_grid(6, 4);
        grid.set_cell_at(2, 2, Cell::from_be_bytes(0x60, 0x01));
        grid.set_cell_at(3, 2, Cell::from_be_bytes(0x30, 0x01));

        assert!(
            juggernaut_wall_scan_sub_96E8(&grid, 43, 63, crate::physics::Direction::Right),
            "ASM sub_96E8/loc_971A scans row (y-$1F)>>4 through (y-$10)>>4 and right-facing columns (x-$0B)>>4 through (x+$04)>>4"
        );
    }

    #[test]
    fn juggernaut_wall_scan_sub_96e8_uses_asm_left_facing_probe_bounds() {
        let mut grid = make_empty_grid(6, 4);
        grid.set_cell_at(2, 2, Cell::from_be_bytes(0x60, 0x01));
        grid.set_cell_at(1, 2, Cell::from_be_bytes(0x30, 0x01));

        assert!(
            juggernaut_wall_scan_sub_96E8(&grid, 36, 63, crate::physics::Direction::Left),
            "ASM sub_96E8/loc_971A swaps the #$0B/#4 offsets for set x_direction before converting to tile columns"
        );
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
    fn vertical_collision_scan_down_sub_36c6a_reads_the_row_below_y_minus_one() {
        let mut grid = make_test_grid();
        grid.set_cell_at(1, 2, Cell::from_be_bytes(0x60, 0x01));

        assert!(
            vertical_collision_scan_down_sub_36C6A(&grid, 24, 32, 8),
            "ASM sub_36C6A should probe row ((y-1)>>4)+1, which lands on row 2 for foot y=32"
        );
        assert!(
            !vertical_collision_scan_down_sub_36C6A(&grid, 24, 16, 8),
            "ASM sub_36C6A should still probe row 1 while foot y remains in the 16..31 tile band"
        );
    }

    #[test]
    fn vertical_collision_scan_down_sub_36c6a_rejects_rows_at_or_beyond_level_height() {
        let grid = make_test_grid();

        assert!(
            !vertical_collision_scan_down_sub_36C6A(&grid, 24, 48, 8),
            "ASM sub_36C6A should clear d6 once the derived row reaches Level_height_blocks"
        );
    }

    #[test]
    fn vertical_collision_scan_down_sub_36c6a_ors_the_full_width_span() {
        let mut data = vec![0u8; 5 * 4 * 2];
        let solid = Cell::from_be_bytes(0x60, 0x01);
        data[(2 * 5 + 0) * 2] = 0;
        data[(2 * 5 + 0) * 2 + 1] = 0;
        let mut grid = BlockGrid::from_level_layout(&data, 5, 4);
        grid.set_cell_at(3, 2, solid);

        assert!(
            vertical_collision_scan_down_sub_36C6A(&grid, 40, 32, 16),
            "ASM loc_36CAA (kid.asm:38539) should OR every `(a0)+` word in the inclusive DBF span before masking bit `$4000`"
        );
    }

    #[test]
    #[ignore = "TODO(fidelity): loc_36C82 not yet implemented"]
    fn vertical_collision_scan_down_loc_36c82_seeds_the_row_and_inclusive_span() {
        let data = vec![0u8; 5 * 4 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 5, 4);
        grid.set_cell_at(1, 2, Cell::from_be_bytes(0x40, 0x01));

        assert!(
            vertical_collision_scan_down_sub_36C6A(&grid, 40, 32, 16),
            "ASM loc_36C82 (kid.asm:38523) should load row ((y-1)>>4)+1 and seed the inclusive span from `(x-half_width)>>4` through `(x+half_width-1)>>4`"
        );
        assert!(
            !vertical_collision_scan_down_sub_36C6A(&grid, 40, 31, 16),
            "ASM loc_36C82 should not use the row below until `subq #1; lsr #4; addq #1` advances the accepted row"
        );
    }

    #[test]
    fn vertical_collision_scan_up_sub_36cb8_reads_the_row_above_the_top_edge() {
        let data = vec![0u8; 4 * 5 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 4, 5);
        grid.set_cell_at(1, 1, Cell::from_be_bytes(0x60, 0x01));

        assert!(
            vertical_collision_scan_up_sub_36CB8(&grid, 24, 48, 8, 16),
            "ASM sub_36CB8 should probe row ((y-height)>>4)-1, which lands on row 1 here"
        );
        assert!(
            !vertical_collision_scan_up_sub_36CB8(&grid, 24, 32, 8, 16),
            "ASM sub_36CB8 should return immediately when the derived row is negative"
        );
    }

    #[test]
    fn vertical_collision_scan_up_sub_36cb8_rejects_rows_at_or_beyond_level_height() {
        let grid = make_test_grid();

        assert!(
            !vertical_collision_scan_up_sub_36CB8(&grid, 24, 80, 8, 16),
            "ASM loc_36CCC (kid.asm:38562) should clear d6 and return once the derived upward row reaches Level_height_blocks"
        );
    }

    #[test]
    fn vertical_collision_scan_up_sub_36cb8_ors_the_full_width_span() {
        let data = vec![0u8; 5 * 5 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 5, 5);
        grid.set_cell_at(3, 1, Cell::from_be_bytes(0x70, 0x01));

        assert!(
            vertical_collision_scan_up_sub_36CB8(&grid, 40, 48, 16, 16),
            "ASM sub_36CB8 should detect any bit-$4000 cell anywhere across the current horizontal span"
        );
    }

    #[test]
    fn vertical_collision_scan_up_loc_36cd8_seeds_the_row_and_inclusive_span() {
        let data = vec![0u8; 5 * 5 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 5, 5);
        grid.set_cell_at(1, 1, Cell::from_be_bytes(0x40, 0x01));

        assert!(
            vertical_collision_scan_up_sub_36CB8(&grid, 40, 48, 16, 16),
            "ASM loc_36CD8 (kid.asm:38570) should load row ((y-height)>>4)-1 and scan `(x-half_width)>>4` through `(x+half_width-1)>>4`"
        );
        assert!(
            !vertical_collision_scan_up_sub_36CB8(&grid, 40, 47, 16, 16),
            "ASM loc_36CD8 should not load the higher row until `subq #1,d4` moves the derived upward row there"
        );
    }

    #[test]
    fn vertical_collision_scan_up_loc_36d00_ors_each_cell_before_masking() {
        let data = vec![0u8; 6 * 5 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 6, 5);
        grid.set_cell_at(4, 1, Cell::from_be_bytes(0x40, 0x01));

        assert!(
            vertical_collision_scan_up_sub_36CB8(&grid, 48, 48, 24, 16),
            "ASM loc_36D00 (kid.asm:38586) should keep ORing `(a0)+` through the inclusive DBF span before masking bit `$4000`"
        );
        assert!(
            !vertical_collision_scan_up_sub_36CB8(&grid, 48, 48, 8, 16),
            "ASM loc_36D00 should only see cells inside the span derived by loc_36CD8"
        );
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
    fn cell_from_be_bytes_decodes_all_asm_collision_nibbles() {
        // ASM GfxObjects_Collision_Tiles_RowLoop (kid.asm:2916) first tests
        // bit `$4000`, and loc_17C6 (kid.asm:2942) masks `$7000` to dispatch
        // `$4000/$5000/$6000/$7000`. Thus `$0000`, `$1000`, and `$3000` are
        // empty terrain, while `$2000` remains hidden-block metadata.
        let cases = [
            (0x00, CollisionKind::Empty),
            (0x10, CollisionKind::Empty),
            (0x20, CollisionKind::HiddenBlock),
            (0x30, CollisionKind::Empty),
            (0x40, CollisionKind::SlopeUp),
            (0x50, CollisionKind::SlopeDown),
            (0x60, CollisionKind::Solid),
            (0x70, CollisionKind::Spikes),
        ];

        for (hi, expected_collision) in cases {
            let cell = Cell::from_be_bytes(hi, 0x5A);

            assert!(!cell.has_block);
            assert_eq!(cell.collision, expected_collision);
            assert_eq!(cell.block_type, BlockType::None);
            assert_eq!(cell.skin, 0x5A);
        }
    }

    #[test]
    fn cell_from_be_bytes_preserves_each_sub_bbe2_block_dispatch_nibble() {
        // ASM sub_BBE2 loc_BC26 (kid.asm:15167) masks `$F00` and shifts by 6,
        // producing a block-type dispatch offset into off_BB80.  Every ROM
        // block nibble, including `$F`, must survive decode and re-pack.
        let cases = [
            (0x00, BlockType::Smash),
            (0x01, BlockType::Prize),
            (0x02, BlockType::Ice),
            (0x03, BlockType::SolidGhost),
            (0x04, BlockType::Teleporter),
            (0x05, BlockType::Metal),
            (0x06, BlockType::Rubber),
            (0x07, BlockType::Shifting),
            (0x08, BlockType::Shooter),
            (0x09, BlockType::Mushroom),
            (0x0A, BlockType::Evanescent),
            (0x0B, BlockType::Lift),
            (0x0C, BlockType::Drill),
            (0x0F, BlockType::Deleted),
        ];

        for (nibble, expected_block_type) in cases {
            let hi = 0xE0 | nibble;
            let cell = Cell::from_be_bytes(hi, 0x34);

            assert!(cell.has_block);
            assert_eq!(cell.collision, CollisionKind::Solid);
            assert_eq!(cell.block_type, expected_block_type);
            assert_eq!(cell.to_word(), (u16::from(hi) << 8) | 0x0034);
        }
    }

    #[test]
    fn smash_block_nibble_zero_dispatches_to_off_bb80_entry_zero() {
        // ASM sub_BBE2 loc_BC26 (kid.asm:15167) maps block nibble `$0` to
        // dispatch offset `$00`, so sub_BB54 indexes off_BB80 entry 0:
        // `j_Block_LoadSmashBits`.
        let data = [0xE0, 0x34];
        let grid = BlockGrid::from_level_layout(&data, 1, 1);
        let probe = kid_block_interaction_probe_sub_BBE2(&grid, 0, 0)
            .expect("smash block should be an interactive block");

        assert_eq!(grid.cell_at(0, 0).block_type, BlockType::Smash);
        assert_eq!(probe.dispatch_table_offset, 0x00);
    }

    #[test]
    fn cell_from_be_bytes_parses_spikes() {
        // 0x70 = 0b0_111_0000 → collision=7 (spikes)
        let cell = Cell::from_be_bytes(0x70, 0x00);
        assert_eq!(cell.collision, CollisionKind::Spikes);
    }

    #[test]
    fn cell_from_be_bytes_parses_slopes() {
        // 0x40 = collision=4 (slope up)
        let cell = Cell::from_be_bytes(0x40, 0x00);
        assert_eq!(cell.collision, CollisionKind::SlopeUp);

        // 0x50 = collision=5 (slope down)
        let cell = Cell::from_be_bytes(0x50, 0x00);
        assert_eq!(cell.collision, CollisionKind::SlopeDown);
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
    fn probe_wall_column_uses_rom_half_width_plus_one_edge() {
        let grid = make_wall_grid();

        // ROM `sub_922C` scans the exact edge column at half_width + 1. With
        // x=72 and half_width=7, that lands on col 5; the narrower 5px probe
        // would still be one tile short.
        let probe = probe_wall_column(&grid, 72, 47, 7, 31, true);

        assert!(probe.hit);
        assert_eq!(probe.cell_x, 5);
        assert_eq!(probe.cell_y, 2);
        assert_eq!(probe.wall_x_px, 80);
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
        // Put a slope-up (collision=4) at col 2, row 1
        let offset = (1 * 4 + 2) * 2;
        data[offset] = 0x40; // collision=4 = SlopeUp
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
        // Row 1, col 0: SlopeUp (collision=4)
        let offset = (1 * 4 + 0) * 2;
        data[offset] = 0x40; // collision=4 = SlopeUp
        // Row 1, col 1: SlopeDown (collision=5)
        let offset = (1 * 4 + 1) * 2;
        data[offset] = 0x50; // collision=5 = SlopeDown
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
        // SlopeDown at (col 1, row 1): collision=5
        data[(usize::from(1u16) * usize::from(w) + usize::from(1u16)) * 2] = 0x50;
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

    // -----------------------------------------------------------------------
    // BlockList::from_raw — comprehensive binary parser test
    // -----------------------------------------------------------------------

    /// Build a raw Block List blob containing one entry of each type
    /// (prize, ghost, teleporter, hidden) with distinct field values so
    /// any byte-offset mutation produces a detectable wrong result.
    ///
    /// Binary format note: sections are sequential, each terminated by a
    /// sentinel. The prize section ends when layout_addr (first 2 bytes)
    /// is 0. The ghost section's first entry with timer=0 also acts as
    /// the prize terminator (timer bytes overlap layout_addr position).
    #[test]
    #[ignore = "TODO(fidelity): block_list raw parser not yet implemented"]
    fn block_list_from_raw_parses_all_sections() {
        let mut data = Vec::new();

        // --- Prize block entry (8 bytes) ---
        // layout_addr=0x0042, col=7, row=3, disguised=0x80|content=0x05
        data.extend_from_slice(&[
            0x00, 0x42, // layout_addr (non-zero → valid)
            0x00, 0x07, // col = 7
            0x00, 0x03, // row = 3
            0x80, // disguised bit set
            0x05, // content = 5 (Eyeclops)
        ]);

        // --- Ghost block entry (16 bytes) ---
        // The ghost's timer=0 acts as prize terminator (layout_addr=0).
        // timer=0, flags=0, mapping_frame=0 (solid), solid_time=25,
        // vanished_time=15, layout_addr=0x0020, col=4, row=6,
        // count_nibble=0x03→4, vertical=1
        data.extend_from_slice(&[
            0x00, 0x00, // timer = 0 (doubles as prize terminator)
            0x00, // flags
            0x00, // mapping_frame = 0 → solid
            0x00, 0x19, // solid_time = 25
            0x00, 0x0F, // vanished_time = 15
            0x00, 0x20, // layout_addr (non-zero → valid ghost)
            0x00, 0x04, // col = 4
            0x00, 0x06, // row = 6
            0x03, // count nibble = 3 → count = 4
            0x01, // vertical = true
        ]);
        // Ghost terminator (all zeros — timer=0, layout_addr=0, solid_time=0)
        data.extend_from_slice(&[0u8; 16]);

        // --- Teleporter entry (10 bytes) ---
        data.extend_from_slice(&[
            0x00, 0x10, // layout_addr
            0x00, 0x09, // col = 9
            0x00, 0x02, // row = 2
            0x05, // dest_map_id = 5
            0x0B, // dest_y = 11
            0x00, 0x18, // dest_x = 24
        ]);
        // Teleporter terminator (layout_addr=0)
        data.extend_from_slice(&[0x00, 0x00]);

        // --- Hidden block entry (8 bytes, after 2-byte teleporter terminator skip) ---
        data.extend_from_slice(&[
            0x00, 0x08, // layout_addr
            0x00, 0x01, // col = 1
            0x00, 0x0A, // row = 10
            0xAB, // block_type
            0xCD, // block_setting
        ]);
        // Hidden terminator
        data.extend_from_slice(&[0x00, 0x00]);

        let bl = BlockList::from_raw(&data, 20);

        // Prize assertions — every field must be exact
        assert_eq!(bl.prize_blocks.len(), 1);
        assert_eq!(bl.prize_blocks[0].layout_addr, 0x0012);
        assert_eq!(bl.prize_blocks[0].col, 7);
        assert_eq!(bl.prize_blocks[0].row, 3);
        assert!(bl.prize_blocks[0].disguised);
        assert_eq!(bl.prize_blocks[0].content, PrizeContent::Eyeclops);

        // Ghost assertions
        assert_eq!(bl.ghost_blocks.len(), 1);
        assert_eq!(bl.ghost_blocks[0].col, 4);
        assert_eq!(bl.ghost_blocks[0].row, 6);
        assert_eq!(bl.ghost_blocks[0].count, 4); // nibble 3 + 1
        assert!(bl.ghost_blocks[0].vertical);
        assert_eq!(bl.ghost_blocks[0].solid_time, 25);
        assert_eq!(bl.ghost_blocks[0].vanished_time, 15);
        assert_eq!(bl.ghost_blocks[0].timer, 0);
        assert!(bl.ghost_blocks[0].solid);

        // Teleporter assertions
        assert_eq!(bl.teleporters.len(), 1);
        assert_eq!(bl.teleporters[0].layout_addr, 0x0040);
        assert_eq!(bl.teleporters[0].col, 9);
        assert_eq!(bl.teleporters[0].row, 2);
        assert_eq!(bl.teleporters[0].dest_map_id, 5);
        assert_eq!(bl.teleporters[0].dest_y, 11);
        assert_eq!(bl.teleporters[0].dest_x, 24);

        // Hidden block assertions
        assert_eq!(bl.hidden_blocks.len(), 1);
        assert_eq!(bl.hidden_blocks[0].layout_addr, 0x0008);
        assert_eq!(bl.hidden_blocks[0].col, 1);
        assert_eq!(bl.hidden_blocks[0].row, 10);
        assert_eq!(bl.hidden_blocks[0].block_type, 0xAB);
        assert_eq!(bl.hidden_blocks[0].block_setting, 0xCD);
    }

    #[test]
    fn block_list_prize_content_lookup() {
        let mut data = Vec::new();
        // Two prize entries with different contents
        data.extend_from_slice(&[0x00, 0x10, 0x00, 0x02, 0x00, 0x05, 0x00, 0x03]); // col=2, row=5, Cyclone
        data.extend_from_slice(&[0x00, 0x20, 0x00, 0x08, 0x00, 0x01, 0x00, 0x0B]); // col=8, row=1, Ankh
        // Ghost with timer=0 as prize terminator + ghost terminator
        data.extend_from_slice(&[0u8; 16]); // timer=0, layout_addr=0, solid_time=0 → terminates both

        let bl = BlockList::from_raw(&data, 20);
        assert_eq!(bl.prize_content_at(2, 5), Some(PrizeContent::Cyclone));
        assert_eq!(bl.prize_content_at(8, 1), Some(PrizeContent::Ankh));
        assert_eq!(bl.prize_content_at(99, 99), None);
    }

    #[test]
    fn block_list_hidden_block_lookup() {
        let mut data = Vec::new();
        // No prizes: ghost timer=0 acts as prize terminator
        // Ghost terminator: timer=0, layout_addr=0, solid_time=0
        data.extend_from_slice(&[0u8; 16]);
        // No teleporters (layout_addr=0)
        data.extend_from_slice(&[0x00, 0x00]);
        // Teleporter terminator skip (2 bytes consumed by parser)
        // Hidden block: col=3, row=7, type=0x42, setting=0x01
        data.extend_from_slice(&[0x00, 0x10, 0x00, 0x03, 0x00, 0x07, 0x42, 0x01]);
        data.extend_from_slice(&[0x00, 0x00]); // hidden terminator

        let bl = BlockList::from_raw(&data, 20);
        let hb = bl
            .hidden_block_at(3, 7)
            .expect("hidden block at (3,7) should exist");
        assert_eq!(hb.layout_addr, 0x0010);
        assert_eq!(hb.block_type, 0x42);
        assert_eq!(hb.block_setting, 0x01);
        assert!(bl.hidden_block_at(99, 99).is_none());
    }

    #[test]
    fn hidden_block_by_layout_addr_sub_e37a_binary_searches_sorted_entries() {
        let bl = BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![
                HiddenBlockEntry {
                    layout_addr: 0x0008,
                    col: 1,
                    row: 0,
                    block_type: 0x01,
                    block_setting: 0x00,
                },
                HiddenBlockEntry {
                    layout_addr: 0x0020,
                    col: 4,
                    row: 1,
                    block_type: 0x04,
                    block_setting: 0x07,
                },
                HiddenBlockEntry {
                    layout_addr: 0x0048,
                    col: 9,
                    row: 3,
                    block_type: 0x0C,
                    block_setting: 0x02,
                },
            ],
        };

        let found = bl
            .hidden_block_by_layout_addr_sub_E37A(0x0020)
            .expect("ASM sub_E37A should find the matching hidden-block entry");

        assert_eq!(found.col, 4);
        assert_eq!(found.row, 1);
        assert_eq!(found.block_type, 0x04);
        assert_eq!(found.block_setting, 0x07);
    }

    #[test]
    fn hidden_block_search_loc_e38e_seeds_first_entry_and_one_past_end() {
        let bl = BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![
                HiddenBlockEntry {
                    layout_addr: 0x0010,
                    col: 2,
                    row: 0,
                    block_type: 0x01,
                    block_setting: 0x00,
                },
                HiddenBlockEntry {
                    layout_addr: 0x0020,
                    col: 4,
                    row: 1,
                    block_type: 0x04,
                    block_setting: 0x07,
                },
                HiddenBlockEntry {
                    layout_addr: 0x0030,
                    col: 6,
                    row: 2,
                    block_type: 0x0C,
                    block_setting: 0x02,
                },
            ],
        };

        let first = bl
            .hidden_block_by_layout_addr_sub_E37A(0x0010)
            .expect("ASM loc_E38E starts `a4` at the first hidden-block entry");
        let last = bl
            .hidden_block_by_layout_addr_sub_E37A(0x0030)
            .expect("ASM loc_E38E computes `a1` as one past `count*8`, so the final entry remains searchable");

        assert_eq!(first.col, 2);
        assert_eq!(last.col, 6);
        assert_eq!(
            BlockList::hidden_block_search_initial_stride_bytes_loc_E386(3),
            16,
            "ASM loc_E38E receives the stride doubled once for a three-entry hidden-block list",
        );
    }

    #[test]
    fn hidden_block_search_loc_e3a0_returns_only_exact_layout_match() {
        let bl = BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![
                HiddenBlockEntry {
                    layout_addr: 0x0010,
                    col: 2,
                    row: 0,
                    block_type: 0x01,
                    block_setting: 0x00,
                },
                HiddenBlockEntry {
                    layout_addr: 0x0028,
                    col: 5,
                    row: 1,
                    block_type: 0x04,
                    block_setting: 0x07,
                },
                HiddenBlockEntry {
                    layout_addr: 0x0040,
                    col: 8,
                    row: 2,
                    block_type: 0x0C,
                    block_setting: 0x02,
                },
            ],
        };

        let found = bl
            .hidden_block_by_layout_addr_sub_E37A(0x0028)
            .expect("ASM loc_E3A0 should return when `cmp.w (a4),d0` finds equality");

        assert_eq!(found.col, 5);
        assert!(
            bl.hidden_block_by_layout_addr_sub_E37A(0x0029).is_none(),
            "ASM loc_E3A0 only exits through loc_E3C0 on an exact layout-address match",
        );
    }

    #[test]
    fn hidden_block_search_loc_e3b2_searches_upper_half_and_clamps_to_last_entry() {
        let bl = BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![
                HiddenBlockEntry {
                    layout_addr: 0x0008,
                    col: 1,
                    row: 0,
                    block_type: 0x01,
                    block_setting: 0x00,
                },
                HiddenBlockEntry {
                    layout_addr: 0x0018,
                    col: 3,
                    row: 1,
                    block_type: 0x04,
                    block_setting: 0x07,
                },
                HiddenBlockEntry {
                    layout_addr: 0x0028,
                    col: 5,
                    row: 2,
                    block_type: 0x0C,
                    block_setting: 0x02,
                },
                HiddenBlockEntry {
                    layout_addr: 0x0038,
                    col: 7,
                    row: 3,
                    block_type: 0x0A,
                    block_setting: 0x01,
                },
            ],
        };

        let upper_half = bl
            .hidden_block_by_layout_addr_sub_E37A(0x0028)
            .expect("ASM loc_E3B2 should advance into the upper half when `d0 > (a4)`");
        let last = bl
            .hidden_block_by_layout_addr_sub_E37A(0x0038)
            .expect("ASM loc_E3B2 clamps probes at `a1-8`, keeping the last entry reachable");

        assert_eq!(upper_half.col, 5);
        assert_eq!(last.col, 7);
    }

    #[test]
    fn hidden_block_by_layout_addr_sub_e37a_returns_none_when_absent() {
        let bl = BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![
                HiddenBlockEntry {
                    layout_addr: 0x0010,
                    col: 2,
                    row: 2,
                    block_type: 0x05,
                    block_setting: 0x01,
                },
                HiddenBlockEntry {
                    layout_addr: 0x0030,
                    col: 6,
                    row: 4,
                    block_type: 0x08,
                    block_setting: 0x00,
                },
            ],
        };

        assert!(
            bl.hidden_block_by_layout_addr_sub_E37A(0x0020).is_none(),
            "ASM sub_E37A should leave `a4` unresolved when no hidden-block entry matches the layout address",
        );
    }

    #[test]
    fn hidden_block_reveal_sub_e3c6_uses_prize_disguise_flag_from_matching_layout_addr() {
        let bl = BlockList {
            prize_blocks: vec![
                PrizeBlockEntry {
                    layout_addr: 0x0012,
                    col: 3,
                    row: 1,
                    content: PrizeContent::Diamond,
                    disguised: false,
                },
                PrizeBlockEntry {
                    layout_addr: 0x0018,
                    col: 4,
                    row: 1,
                    content: PrizeContent::Cyclone,
                    disguised: true,
                },
            ],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![],
        };

        let visible_prize = bl
            .hidden_block_reveal_sub_E3C6(&HiddenBlockEntry {
                layout_addr: 0x0012,
                col: 3,
                row: 1,
                block_type: 0x01,
                block_setting: 0x03,
            })
            .expect("ASM sub_E3C6 should reveal visible prize tiles from the prize list");
        assert_eq!(visible_prize.cell, Cell::from_be_bytes(0xE1, 0x05));
        assert_eq!(visible_prize.sprite_mapping, 0x1060);

        let disguised_prize = bl
            .hidden_block_reveal_sub_E3C6(&HiddenBlockEntry {
                layout_addr: 0x0018,
                col: 4,
                row: 1,
                block_type: 0x01,
                block_setting: 0x04,
            })
            .expect("ASM sub_E3C6 should pick the disguised prize tile set when the prize entry is flagged negative");
        assert_eq!(disguised_prize.cell, Cell::from_be_bytes(0xE1, 0x05));
        assert_eq!(disguised_prize.sprite_mapping, 0x1058);
    }

    #[test]
    fn loc_e3e4_hidden_block_prize_reveal_seeds_preincrement_scan() {
        let entry = HiddenBlockEntry {
            layout_addr: 0x0018,
            col: 4,
            row: 1,
            block_type: 0x01,
            block_setting: 0x04,
        };

        let scan = BlockList::hidden_block_prize_reveal_scan_start_loc_E3E4(0xFFFF_3B24, &entry);

        assert_eq!(
            scan.target_layout_addr, 0x0018,
            "ASM loc_E3E4 should copy the hidden-block layout word from `(a4)` into `d3` before scanning prizes",
        );
        assert_eq!(
            scan.preincrement_cursor_addr, 0xFFFF_3B1C,
            "ASM loc_E3E4 should subtract 8 from the `$FFFFF8D4` prize-list base so the next pre-increment reaches record 0",
        );
        assert_eq!(
            scan.first_prize_record_addr, 0xFFFF_3B24,
            "ASM loc_E3E4 should preserve the actual first prize record after seeding the pre-increment cursor",
        );
    }

    #[test]
    fn loc_e3ee_hidden_block_prize_reveal_scans_eight_byte_records_until_match() {
        let bl = BlockList {
            prize_blocks: vec![
                PrizeBlockEntry {
                    layout_addr: 0x0008,
                    col: 1,
                    row: 0,
                    content: PrizeContent::Diamond,
                    disguised: false,
                },
                PrizeBlockEntry {
                    layout_addr: 0x0010,
                    col: 2,
                    row: 0,
                    content: PrizeContent::Clock,
                    disguised: false,
                },
                PrizeBlockEntry {
                    layout_addr: 0x0018,
                    col: 3,
                    row: 0,
                    content: PrizeContent::Micromax,
                    disguised: true,
                },
            ],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![],
        };

        let matched = bl
            .hidden_block_prize_reveal_metadata_scan_loc_E3EE(0xFFFF_3B24, 0x0018)
            .expect("ASM loc_E3EE should stop when the prize record layout word matches");

        assert_eq!(
            matched.matched_record_addr, 0xFFFF_3B34,
            "ASM loc_E3EE should advance the cursor by 8 bytes for each failed prize-record compare",
        );
        assert_eq!(
            matched.scanned_records, 3,
            "ASM loc_E3EE should include the matching record in the scan count",
        );
        assert!(
            matched.disguised,
            "ASM loc_E3EE should leave `a0` on the matching prize record so the flag word at offset 6 is read from that record",
        );
    }

    #[test]
    fn loc_e40a_hidden_block_prize_reveal_adds_setting_as_byte_and_restores_mapping() {
        let visible = BlockList::hidden_block_prize_reveal_cell_loc_E40A(false, 0x03);
        assert_eq!(
            visible.cell,
            Cell::from_be_bytes(0xE1, 0x05),
            "ASM loc_E40A should add the setting byte to visible prize base `$E102`",
        );
        assert_eq!(
            visible.sprite_mapping, 0x1060,
            "ASM loc_E40A should preserve the visible prize sprite mapping selected before the final add",
        );

        let wrapped = BlockList::hidden_block_prize_reveal_cell_loc_E40A(false, 0xFF);
        assert_eq!(
            wrapped.cell,
            Cell::from_be_bytes(0xE1, 0x01),
            "ASM loc_E40A uses `add.b`, so `$E102 + $FF` wraps the low byte without carrying to `$E201`",
        );

        let disguised = BlockList::hidden_block_prize_reveal_cell_loc_E40A(true, 0x04);
        assert_eq!(
            disguised.cell,
            Cell::from_be_bytes(0xE1, 0x05),
            "ASM loc_E40A should add the setting byte to disguised prize base `$E101`",
        );
        assert_eq!(
            disguised.sprite_mapping, 0x1058,
            "ASM loc_E40A should preserve the disguised prize sprite mapping selected from the matched flag word",
        );
    }

    #[test]
    fn prize_block_by_layout_addr_loc_10342_walks_the_eight_byte_prize_entries_until_match() {
        let bl = BlockList {
            prize_blocks: vec![
                PrizeBlockEntry {
                    layout_addr: 0x0008,
                    col: 1,
                    row: 0,
                    content: PrizeContent::Diamond,
                    disguised: false,
                },
                PrizeBlockEntry {
                    layout_addr: 0x0010,
                    col: 2,
                    row: 0,
                    content: PrizeContent::Clock,
                    disguised: false,
                },
                PrizeBlockEntry {
                    layout_addr: 0x0018,
                    col: 3,
                    row: 0,
                    content: PrizeContent::Micromax,
                    disguised: true,
                },
            ],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![],
        };

        let matched = bl
            .prize_block_by_layout_addr_loc_10342(0x0018)
            .expect("expected matching prize entry");

        assert_eq!(matched.layout_addr, 0x0018);
        assert_eq!(matched.content, PrizeContent::Micromax);
        assert!(
            matched.disguised,
            "ASM loc_10342 should keep stepping `a1` by 8 bytes until `loc_10344` reaches the requested layout address, then reuse that exact entry",
        );
    }

    #[test]
    fn hidden_block_reveal_sub_e3c6_maps_non_prize_types_from_rom_table() {
        let bl = BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![],
        };

        let reveal = bl
            .hidden_block_reveal_sub_E3C6(&HiddenBlockEntry {
                layout_addr: 0x0048,
                col: 9,
                row: 3,
                block_type: 0x0A,
                block_setting: 0x02,
            })
            .expect("ASM sub_E3C6 should map block type $0A through `unk_E412/unk_E414`");

        assert_eq!(reveal.cell, Cell::from_be_bytes(0xEA, 0x1E));
        assert_eq!(reveal.sprite_mapping, 0x10E4);
    }

    #[test]
    fn hidden_block_reveal_sub_e3c6_requires_matching_prize_entry_for_type_1() {
        let bl = BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![],
        };

        assert!(
            bl.hidden_block_reveal_sub_E3C6(&HiddenBlockEntry {
                layout_addr: 0x0020,
                col: 4,
                row: 1,
                block_type: 0x01,
                block_setting: 0x00,
            })
            .is_none(),
            "ASM sub_E3C6 walks the prize list for type 1, so a hidden prize without a matching layout entry should not resolve",
        );
    }

    #[test]
    fn hidden_block_column_updates_sub_e446_reveals_each_hidden_cell_inclusive_span() {
        let width = 4u16;
        let height = 5u16;
        let mut data = vec![0u8; usize::from(width) * usize::from(height) * 2];
        let set_cell = |data: &mut [u8], col: u16, row: u16, hi: u8, lo: u8| {
            let offset = (usize::from(row) * usize::from(width) + usize::from(col)) * 2;
            data[offset] = hi;
            data[offset + 1] = lo;
        };
        set_cell(&mut data, 1, 1, 0xA0, 0x00);
        set_cell(&mut data, 1, 2, 0xA0, 0x00);
        set_cell(&mut data, 1, 3, 0x60, 0x00);
        let mut grid = BlockGrid::from_level_layout(&data, width, height);

        let bl = BlockList {
            prize_blocks: vec![PrizeBlockEntry {
                layout_addr: 0x000A,
                col: 1,
                row: 1,
                content: PrizeContent::Diamond,
                disguised: false,
            }],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![
                HiddenBlockEntry {
                    layout_addr: 0x000A,
                    col: 1,
                    row: 1,
                    block_type: 0x01,
                    block_setting: 0x02,
                },
                HiddenBlockEntry {
                    layout_addr: 0x0012,
                    col: 1,
                    row: 2,
                    block_type: 0x0A,
                    block_setting: 0x01,
                },
            ],
        };

        let updates = bl.hidden_block_column_updates_sub_E446(&mut grid, 1, 1, 2);

        assert_eq!(
            updates,
            vec![
                HiddenBlockColumnUpdateSubE446 {
                    layout_addr: 0x000A,
                    col: 1,
                    row: 1,
                    cell: Cell::from_be_bytes(0xE1, 0x04),
                    sprite_mapping: 0x1060,
                    duration_frames: 0x012C,
                },
                HiddenBlockColumnUpdateSubE446 {
                    layout_addr: 0x0012,
                    col: 1,
                    row: 2,
                    cell: Cell::from_be_bytes(0xEA, 0x1D),
                    sprite_mapping: 0x10E4,
                    duration_frames: 0x012C,
                },
            ],
            "ASM sub_E446 should walk the requested rows inclusively, reveal each resolvable HiddenBlock cell, and attach the `$12C` worker lifetime",
        );
        assert_eq!(grid.cell_at(1, 1), Cell::from_be_bytes(0xE1, 0x04));
        assert_eq!(grid.cell_at(1, 2), Cell::from_be_bytes(0xEA, 0x1D));
        assert_eq!(
            grid.cell_at(1, 3),
            Cell::from_be_bytes(0x60, 0x00),
            "non-hidden cells in the walked column should be left alone",
        );
    }

    #[test]
    fn hidden_block_column_updates_sub_e446_skips_unresolved_hidden_cells() {
        let width = 3u16;
        let height = 4u16;
        let mut data = vec![0u8; usize::from(width) * usize::from(height) * 2];
        let offset = (usize::from(1u16) * usize::from(width) + usize::from(2u16)) * 2;
        data[offset] = 0xA0;
        let mut grid = BlockGrid::from_level_layout(&data, width, height);

        let bl = BlockList {
            prize_blocks: vec![],
            ghost_blocks: vec![],
            teleporters: vec![],
            hidden_blocks: vec![],
        };

        let updates = bl.hidden_block_column_updates_sub_E446(&mut grid, 2, 1, 0);

        assert!(
            updates.is_empty(),
            "ASM sub_E446 should leave HiddenBlock cells untouched when sub_E37A/sub_E3C6 cannot resolve them",
        );
        assert_eq!(grid.cell_at(2, 1), Cell::from_be_bytes(0xA0, 0x00));
    }

    #[test]
    fn prize_content_round_trip() {
        // Every content variant round-trips through to_id/from_id
        let variants = [
            PrizeContent::Diamond,
            PrizeContent::Points10000,
            PrizeContent::Skycutter,
            PrizeContent::Cyclone,
            PrizeContent::RedStealth,
            PrizeContent::Eyeclops,
            PrizeContent::Juggernaut,
            PrizeContent::IronKnight,
            PrizeContent::Berzerker,
            PrizeContent::Maniaxe,
            PrizeContent::Micromax,
            PrizeContent::Ankh,
            PrizeContent::Clock,
            PrizeContent::Continue,
            PrizeContent::TenDiamonds,
        ];
        for v in &variants {
            assert_eq!(
                PrizeContent::from_id(v.to_id()),
                *v,
                "round-trip failed for {v:?}"
            );
        }
    }

    #[test]
    fn prize_content_to_id_covers_all_non_zero() {
        // to_id returns distinct values for all non-Diamond variants
        let ids: Vec<u8> = [
            PrizeContent::Points10000,
            PrizeContent::Skycutter,
            PrizeContent::Cyclone,
            PrizeContent::RedStealth,
            PrizeContent::Eyeclops,
            PrizeContent::Juggernaut,
            PrizeContent::IronKnight,
            PrizeContent::Berzerker,
            PrizeContent::Maniaxe,
            PrizeContent::Micromax,
            PrizeContent::Ankh,
            PrizeContent::Clock,
            PrizeContent::Continue,
            PrizeContent::TenDiamonds,
        ]
        .iter()
        .map(|v| v.to_id())
        .collect();
        for (i, id) in ids.iter().enumerate() {
            assert!(*id > 0, "non-Diamond variant should have non-zero id");
            assert!(*id <= 0x0E, "id should be <= 0x0E");
            for (j, other) in ids.iter().enumerate() {
                if i != j {
                    assert_ne!(id, other, "ids must be unique");
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // BlockGrid::from_level_layout — grid construction
    // -----------------------------------------------------------------------

    #[test]
    fn block_grid_dimensions_match_input() {
        let w = 10u16;
        let h = 8u16;
        let data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        let grid = BlockGrid::from_level_layout(&data, w, h);
        assert_eq!(grid.width_pixels(), i32::from(w) * 16);
        assert_eq!(grid.height_pixels(), i32::from(h) * 16);
    }

    #[test]
    fn block_grid_set_cell_at_out_of_bounds_is_noop() {
        let data = vec![0u8; 4 * 4 * 2];
        let mut grid = BlockGrid::from_level_layout(&data, 4, 4);
        let solid = Cell::from_be_bytes(0x60, 0x00);
        // Out of bounds should not panic
        grid.set_cell_at(4, 0, solid);
        grid.set_cell_at(0, 4, solid);
        grid.set_cell_at(100, 100, solid);
        // In-bounds should work
        grid.set_cell_at(2, 2, solid);
        assert_eq!(grid.cell_at(2, 2).collision, CollisionKind::Solid);
    }

    // -----------------------------------------------------------------------
    // Probe functions — left-moving and edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn probe_wall_column_moving_left_hits_wall() {
        // Build grid with wall at col 0, rows 1-2
        let w = 4u16;
        let h = 4u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Wall at col 0, rows 1-2
        data[(1 * usize::from(w) + 0) * 2] = 0x60;
        data[(2 * usize::from(w) + 0) * 2] = 0x60;
        let grid = BlockGrid::from_level_layout(&data, w, h);

        // Player at x=16, moving left. half_width=7: edge_x = 16 - 7 - 1 = 8
        // pixel_to_cell(8) = 0 → wall col
        let probe = probe_wall_column(&grid, 16, 47, 7, 31, false);
        assert!(probe.hit);
        assert_eq!(probe.cell_x, 0);
        // Wall surface = right edge of col 0 = 16
        assert_eq!(probe.wall_x_px, 16);
    }

    #[test]
    fn probe_iron_knight_climb_resume_moving_left() {
        // Wall at col 0, row 1
        let w = 4u16;
        let h = 4u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        data[(1 * usize::from(w) + 0) * 2] = 0x60;
        let grid = BlockGrid::from_level_layout(&data, w, h);

        // Player at x=16, feet at y=48. Probe y = 48 - 24 = 24 (row 1).
        // Moving left: edge_x = 16 - 7 - 1 = 8 → col 0
        let probe = probe_iron_knight_climb_resume_wall(&grid, 16, 48, 7, false);
        assert!(probe.hit);
        assert_eq!(probe.wall_x_px, 16); // right edge of col 0
    }

    #[test]
    fn probe_iron_knight_climb_resume_moving_right() {
        let grid = make_wall_grid(); // wall at col 5, rows 1-2
        // Player at x=72, feet at y=48. Probe y = 48 - 24 = 24 (row 1).
        // Moving right: edge_x = 72 + 7 + 1 = 80 → col 5
        let probe = probe_iron_knight_climb_resume_wall(&grid, 72, 48, 7, true);
        assert!(probe.hit);
        assert_eq!(probe.wall_x_px, 80); // left edge of col 5
    }

    #[test]
    fn probe_wall_top_edge_moving_left() {
        // Wall at col 0, row 0
        let w = 4u16;
        let h = 4u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        data[0] = 0x60; // col 0, row 0
        let grid = BlockGrid::from_level_layout(&data, w, h);

        // Player feet at y=31, height=31 → top_y=0. Moving left: edge_x = 10 - 5 = 5 → col 0
        let probe = probe_wall_top_edge(&grid, 10, 31, 5, 31, false);
        assert!(probe.hit);
        assert_eq!(probe.wall_x_px, 16); // right edge of col 0
    }

    #[test]
    fn probe_wall_column_out_of_bounds_left() {
        let grid = make_wall_grid();
        // Player near left edge, moving left. edge_x would go negative.
        let probe = probe_wall_column(&grid, 2, 47, 7, 31, false);
        assert!(!probe.hit, "out-of-bounds probe should not hit");
    }

    #[test]
    fn probe_wall_column_moving_left_edge_calculation() {
        // Verify edge_x = center - half_width - 1 (not center / half_width)
        // Build grid where col 1 is solid but col 2 is not
        let w = 6u16;
        let h = 3u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        data[(1 * usize::from(w) + 1) * 2] = 0x60; // col 1, row 1 solid
        let grid = BlockGrid::from_level_layout(&data, w, h);

        // center=32, half_width=7 → edge_x = 32 - 7 - 1 = 24 → col 1 (solid)
        let probe = probe_wall_column(&grid, 32, 31, 7, 16, false);
        assert!(probe.hit);
        assert_eq!(probe.cell_x, 1);

        // center=33, half_width=7 → edge_x = 33 - 7 - 1 = 25 → col 1 (solid)
        let probe2 = probe_wall_column(&grid, 33, 31, 7, 16, false);
        assert!(probe2.hit);
        assert_eq!(probe2.cell_x, 1);

        // center=40, half_width=7 → edge_x = 40 - 7 - 1 = 32 → col 2 (empty)
        let probe3 = probe_wall_column(&grid, 40, 31, 7, 16, false);
        assert!(!probe3.hit);
    }

    #[test]
    fn probe_wall_three_point_height_calculation() {
        // Verify probe_wall samples lower, mid, and upper body points.
        // Build grid where only the middle row is solid.
        let w = 4u16;
        let h = 6u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Solid at col 3, row 2 only (middle of body)
        data[(2 * usize::from(w) + 3) * 2] = 0x60;
        let grid = BlockGrid::from_level_layout(&data, w, h);

        // Player at x=42, half_width=5: edge_x = 42 + 5 + 1 = 48 → col 3
        // height=32. Feet at y=63 (row 3).
        // lower_body = 63 - 8 = 55 (row 3) — not solid
        // mid_body = 63 - 16 = 47 (row 2) — SOLID
        let probe = probe_wall(&grid, 42, 63, 5, 32, true);
        assert!(probe.hit, "mid-body probe should hit solid at col 3 row 2");
        assert_eq!(probe.cell_x, 3);
    }

    #[test]
    fn probe_ceiling_uses_both_edges() {
        // Left edge hits ceiling, right edge doesn't
        let w = 4u16;
        let h = 4u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Ceiling at col 0, row 0
        data[0] = 0x60;
        let grid = BlockGrid::from_level_layout(&data, w, h);

        // center=8, half_width=5, height=31, feet=31 → top=0
        // left edge = 8 - 5 = 3 → col 0 (solid ceiling)
        let probe = probe_ceiling(&grid, 8, 31, 5, 31);
        assert!(probe.hit);

        // center=24, half_width=5 → left=19 (col 1), right=29 (col 1) → no ceiling
        let probe2 = probe_ceiling(&grid, 24, 31, 5, 31);
        assert!(!probe2.hit);
    }

    #[test]
    fn probe_ceiling_right_edge_hits() {
        // Right edge hits ceiling, left edge doesn't
        let w = 4u16;
        let h = 4u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Ceiling at col 2, row 0
        data[2 * 2] = 0x60;
        let grid = BlockGrid::from_level_layout(&data, w, h);

        // center=28, half_width=5, height=31, feet=31 → top=0
        // left edge = 28 - 5 = 23 → col 1 (empty)
        // right edge = 28 + 5 = 33 → col 2 (solid ceiling)
        let probe = probe_ceiling(&grid, 28, 31, 5, 31);
        assert!(probe.hit);
    }

    #[test]
    fn probe_floor_hitbox_asymmetric_edges() {
        // Left edge on floor, right edge over gap — assert left edge detected
        let grid = make_wall_grid(); // floor at row 3, gap at cols 2-3
        // center=24 (col 1), half_width=5: left=19 (col 1, solid), right=29 (col 1, solid)
        let probe = probe_floor_hitbox(&grid, 24, 48, 5);
        assert!(probe.on_ground);

        // center=28: left=23 (col 1, solid), right=33 (col 2, gap)
        let probe2 = probe_floor_hitbox(&grid, 28, 48, 5);
        assert!(probe2.on_ground);

        // center=56: left=51 (col 3, gap), right=61 (col 3, gap) — no floor
        let probe3 = probe_floor_hitbox(&grid, 56, 48, 5);
        assert!(!probe3.on_ground);
    }

    #[test]
    fn probe_floor_slope_on_ground_boundary() {
        let grid = make_slope_grid();
        // SlopeUp at col 0, row 1. x=8: floor_y = 16 + 15 - 8 = 23
        // At exactly the surface: on_ground
        let on = probe_floor(&grid, 8, 23);
        assert!(on.on_ground);
        // One pixel above: not on_ground
        let above = probe_floor(&grid, 8, 22);
        assert!(!above.on_ground);
        // One pixel below: on_ground
        let below = probe_floor(&grid, 8, 24);
        assert!(below.on_ground);
    }

    #[test]
    fn block_list_from_raw_multiple_prizes_distinct_fields() {
        // Two prizes with different col/row to test offset arithmetic
        let mut data = Vec::new();
        // Prize 1: col=2, row=3, content=Diamond (0), not disguised
        data.extend_from_slice(&[0x00, 0x10, 0x00, 0x02, 0x00, 0x03, 0x00, 0x00]);
        // Prize 2: col=5, row=8, content=IronKnight (7), disguised
        data.extend_from_slice(&[0x00, 0x20, 0x00, 0x05, 0x00, 0x08, 0x80, 0x07]);
        // Terminator via ghost timer=0
        data.extend_from_slice(&[0u8; 16]);

        let bl = BlockList::from_raw(&data, 20);
        assert_eq!(bl.prize_blocks.len(), 2);
        // First prize
        assert_eq!(bl.prize_blocks[0].col, 2);
        assert_eq!(bl.prize_blocks[0].row, 3);
        assert_eq!(bl.prize_blocks[0].content, PrizeContent::Diamond);
        assert!(!bl.prize_blocks[0].disguised);
        // Second prize — tests offset advancement (+8 per entry)
        assert_eq!(bl.prize_blocks[1].col, 5);
        assert_eq!(bl.prize_blocks[1].row, 8);
        assert_eq!(bl.prize_blocks[1].content, PrizeContent::IronKnight);
        assert!(bl.prize_blocks[1].disguised);
    }

    #[test]
    fn block_list_from_raw_ghost_non_solid() {
        // Ghost with mapping_frame != 0 should have solid=false
        let mut data = Vec::new();
        // Prize terminator + ghost entry with mapping_frame=1 (non-solid)
        data.extend_from_slice(&[
            0x00, 0x00, // timer=0 (prize terminator)
            0x00, 0x01, // flags=0, mapping_frame=1 → NOT solid
            0x00, 0x0A, // solid_time=10
            0x00, 0x05, // vanished_time=5
            0x00, 0x30, // layout_addr
            0x00, 0x02, // col=2
            0x00, 0x04, // row=4
            0x01, // count=2
            0x00, // horizontal
        ]);
        data.extend_from_slice(&[0u8; 16]); // ghost terminator

        let bl = BlockList::from_raw(&data, 20);
        assert_eq!(bl.ghost_blocks.len(), 1);
        assert!(!bl.ghost_blocks[0].solid, "mapping_frame=1 means not solid");
        assert!(!bl.ghost_blocks[0].vertical);
        assert_eq!(bl.ghost_blocks[0].count, 2);
    }

    #[test]
    fn block_list_from_raw_teleporter_dest_fields() {
        // Verify teleporter offset arithmetic for dest_x (bytes 8-9)
        let mut data = Vec::new();
        data.extend_from_slice(&[0u8; 16]); // no prizes, ghost terminator
        // Teleporter with large dest_x to test byte ordering
        data.extend_from_slice(&[
            0x00, 0x10, // layout_addr
            0x00, 0x01, // col=1
            0x00, 0x02, // row=2
            0x03, // dest_map_id=3
            0x04, // dest_y=4
            0x01, 0x00, // dest_x=256 (big-endian)
        ]);
        data.extend_from_slice(&[0x00, 0x00]); // teleporter terminator

        let bl = BlockList::from_raw(&data, 20);
        assert_eq!(bl.teleporters.len(), 1);
        assert_eq!(bl.teleporters[0].dest_x, 256);
        assert_eq!(bl.teleporters[0].dest_map_id, 3);
        assert_eq!(bl.teleporters[0].dest_y, 4);
    }

    #[test]
    fn block_grid_from_level_layout_cell_index() {
        // Verify row-major indexing: cell(col, row) = data[row * width + col]
        let w = 3u16;
        let h = 3u16;
        let mut data = vec![0u8; usize::from(w) * usize::from(h) * 2];
        // Put solid at (col=2, row=1) → byte offset = (1*3 + 2) * 2 = 10
        data[10] = 0x60;
        let grid = BlockGrid::from_level_layout(&data, w, h);
        assert_eq!(grid.cell_at(2, 1).collision, CollisionKind::Solid);
        assert_eq!(grid.cell_at(1, 2).collision, CollisionKind::Empty);
        assert_eq!(grid.cell_at(0, 0).collision, CollisionKind::Empty);
    }

    #[test]
    fn probe_floor_hitbox_with_exact_landing_y() {
        let grid = make_wall_grid();
        // Floor at row 3 (y=48). Player feet at exactly 48.
        let probe = probe_floor_hitbox(&grid, 8, 48, 5);
        assert!(probe.on_ground);
        assert_eq!(probe.floor_y_px, 48);
    }

    #[test]
    fn probe_floor_hitbox_below_floor_still_detects() {
        let grid = make_wall_grid();
        // Player feet 2px below floor surface — should still detect floor
        let probe = probe_floor_hitbox(&grid, 8, 50, 5);
        assert!(probe.on_ground);
    }

    #[test]
    fn horizontal_collision_scan_right_sub_36b3c_flags_a_slope_inside_the_hitbox() {
        let mut data = vec![0u8; 4 * 4 * 2];
        let slope_offset = (1 * 4 + 1) * 2;
        data[slope_offset] = 0x40;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let scan = horizontal_collision_scan_right_sub_36B3C(&grid, 24, 32, 8, 16);

        assert_eq!(
            scan,
            HorizontalCollisionScan {
                hitbox_blocked: true,
                lead_column_blocked: false,
            },
            "ASM sub_36B3C should treat any bit-$4000 collision cell inside the body rectangle as a hit, including slopes",
        );
    }

    #[test]
    fn horizontal_collision_scan_right_loc_36ba0_treats_the_right_boundary_as_blocked() {
        let data = [0u8; 4 * 4 * 2];
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let scan = horizontal_collision_scan_right_sub_36B3C(&grid, 56, 32, 8, 16);

        assert_eq!(
            scan,
            HorizontalCollisionScan {
                hitbox_blocked: false,
                lead_column_blocked: true,
            },
            "ASM loc_36BA0 should set `d6=1` and skip the lead-column scan when `d1 == Level_width_blocks` at the right edge",
        );
    }

    #[test]
    fn horizontal_collision_scan_right_loc_36b5a_clamps_the_bottom_row_before_scanning() {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(3 * 4 + 1) * 2] = 0x40;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let scan = horizontal_collision_scan_right_sub_36B3C(&grid, 24, 65, 8, 16);

        assert_eq!(
            scan,
            HorizontalCollisionScan {
                hitbox_blocked: true,
                lead_column_blocked: false,
            },
            "ASM loc_36B5A should clamp the bottom row to `Level_height_blocks - 1` before deriving `d2`, so a one-row span at the final level row still scans normally",
        );
    }

    #[test]
    fn horizontal_collision_scan_right_loc_36baa_flags_an_in_bounds_right_slope_column() {
        let mut data = vec![0u8; 4 * 4 * 2];
        let slope_offset = (1 * 4 + 2) * 2;
        data[slope_offset] = 0x50;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let scan = horizontal_collision_scan_right_sub_36B3C(&grid, 24, 32, 8, 16);

        assert_eq!(
            scan,
            HorizontalCollisionScan {
                hitbox_blocked: false,
                lead_column_blocked: true,
            },
            "ASM loc_36BAA should scan the in-bounds right lead column before the boundary fallback, so a `$5000` slope there must still block side contact",
        );
    }

    #[test]
    fn horizontal_collision_scan_column_mask_loc_369ce_ors_the_full_right_lead_column() {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(1 * 4 + 2) * 2] = 0x50;
        data[(2 * 4 + 2) * 2] = 0x60;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let mask = horizontal_collision_scan_column_mask_loc_369CE(&grid, 2, 1, 2);

        assert_eq!(
            mask & 0x7000,
            0x7000,
            "ASM loc_369CE should OR every scanned lead-column word together before the later bit tests, so stacked `$5000`/`$6000` rows keep both collision nibbles in the aggregate mask",
        );
    }

    #[test]
    fn horizontal_collision_scan_right_lead_column_mask_loc_36bb2_ors_the_inclusive_column_span() {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(1 * 4 + 2) * 2] = 0x50;
        data[(2 * 4 + 2) * 2] = 0x20;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let mask = horizontal_collision_scan_right_lead_column_mask_loc_36BB2(&grid, 2, 1, 2);

        assert_eq!(
            mask & 0x7000,
            0x7000,
            "ASM loc_36BB2 (kid.asm:38410) should keep OR-ing the right lead-column accumulator across the inclusive `dbf d0` row span, preserving bits from both scanned rows",
        );
    }

    #[test]
    fn horizontal_collision_scan_hitbox_mask_seed_loc_369f6_clears_the_rect_accumulator() {
        assert_eq!(
            horizontal_collision_scan_hitbox_mask_seed_loc_369F6(),
            0,
            "ASM loc_369F6 is just `moveq #0,d5`, so the rectangle scan must always start from a cleared accumulator",
        );
    }

    #[test]
    fn horizontal_collision_scan_rect_mask_loc_369f8_ors_every_word_in_the_hitbox() {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(1 * 4 + 1) * 2] = 0x40;
        data[(2 * 4 + 2) * 2] = 0x60;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let mask = horizontal_collision_scan_rect_mask_loc_369F8(&grid, 1, 2, 1, 2);

        assert_eq!(
            mask & 0x7000,
            0x6000,
            "ASM loc_369F8 should sweep every row of the hitbox rectangle, so a later-row `$6000` solid must survive in the final OR mask after the nested `dbf` loops finish",
        );
    }

    #[test]
    fn horizontal_collision_scan_rect_mask_loc_369fc_keeps_early_row_hits_after_later_empty_rows() {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(1 * 4 + 1) * 2] = 0x40;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let mask = horizontal_collision_scan_rect_mask_loc_369F8(&grid, 1, 2, 1, 2);

        assert_eq!(
            mask & 0x4000,
            0x4000,
            "ASM loc_369FC should keep OR-ing the same `d5` accumulator as it advances row-major through the rectangle, so a `$4000` hit on the first scanned row must survive even when the final row is empty",
        );
    }

    #[test]
    fn horizontal_collision_scan_right_hitbox_mask_loc_36bbe_resets_each_row_start() {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(1 * 4 + 1) * 2] = 0x40;
        data[(2 * 4 + 2) * 2] = 0x20;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let mask = horizontal_collision_scan_right_hitbox_mask_loc_36BBE(&grid, 1, 2, 1, 2);

        assert_eq!(
            mask & 0x7000,
            0x6000,
            "ASM loc_36BBE (kid.asm:38416) should reset `a4` to each row start before the row-width scan, so both columns are checked again on later rows",
        );
    }

    #[test]
    fn horizontal_collision_scan_right_hitbox_row_mask_loc_36bc2_ors_the_inclusive_row_width() {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(1 * 4 + 1) * 2] = 0x20;
        data[(1 * 4 + 2) * 2] = 0x40;
        data[(1 * 4 + 3) * 2] = 0x50;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let mask = horizontal_collision_scan_right_hitbox_row_mask_loc_36BC2(&grid, 1, 3, 1);

        assert_eq!(
            mask & 0x7000,
            0x7000,
            "ASM loc_36BC2 (kid.asm:38420) should OR every word in the inclusive hitbox row before `dbf d0` exits, preserving bits from the first and final scanned columns",
        );
    }

    #[test]
    fn horizontal_collision_scan_following_row_class_loc_36a10_keeps_d6_zero_for_all_blocked_rows()
    {
        assert_eq!(
            horizontal_collision_scan_following_row_class_loc_36A10(&[0x4000, 0x5000, 0x6000]),
            0,
            "ASM loc_36A10 should stay on the blocking path and leave `d6=0` when every scanned word in the following row keeps bit `$4000` set",
        );
    }

    #[test]
    fn horizontal_collision_scan_following_row_class_loc_36a2a_returns_zero_when_every_scanned_word_stays_blocked()
     {
        assert_eq!(
            horizontal_collision_scan_following_row_class_loc_36A10(&[0x4000]),
            0,
            "ASM loc_36A2A should return with `d6=0` when the first blocked word is also the last scanned word, so the inclusive `dbf` loop never finds a clear `$4000` bit to flip the row class",
        );
    }

    #[test]
    fn horizontal_collision_scan_following_row_class_loc_36a36_returns_one_when_a_blocked_entry_row_turns_mixed()
     {
        assert_eq!(
            horizontal_collision_scan_following_row_class_loc_36A10(&[0x4000, 0x0000]),
            1,
            "ASM loc_36A36 should return `d6=1` once the blocked-entry scan finds a later word with bit `$4000` clear",
        );
    }

    #[test]
    fn horizontal_collision_scan_following_row_class_loc_36a20_sets_d6_for_mixed_rows() {
        assert_eq!(
            horizontal_collision_scan_following_row_class_loc_36A10(&[0x4000, 0x0000, 0x6000]),
            1,
            "ASM loc_36A20 should keep scanning after the blocked entry word and return `d6=1` once a later word in the same row clears bit `$4000`",
        );
    }

    #[test]
    fn horizontal_collision_scan_following_row_class_loc_36a40_scans_past_a_clear_entry_until_a_later_blocked_word()
     {
        assert_eq!(
            horizontal_collision_scan_following_row_class_loc_36A10(&[0x0000, 0x0000, 0x4000]),
            1,
            "ASM loc_36A40 should keep scanning after a clear entry word and reach the shared mixed-row exit once a later word sets bit `$4000`",
        );
    }

    #[test]
    fn horizontal_collision_scan_following_row_class_loc_36a4a_returns_negative_one_when_the_row_stays_clear()
     {
        assert_eq!(
            horizontal_collision_scan_following_row_class_loc_36A10(&[0x0000, 0x0000, 0x0000]),
            -1,
            "ASM loc_36A4A should return `d6=-1` when the clear-entry path finishes its inclusive `dbf` scan without finding any later `$4000` word",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_lead_column_index_loc_36aa6_skips_the_boundary_and_targets_the_adjacent_column()
     {
        assert_eq!(
            horizontal_collision_scan_left_lead_column_index_loc_36AA6(0),
            None,
            "ASM loc_36AA6 should not run when `d0==0`, because the left-edge case falls straight to the boundary-class result at `loc_36AD6`",
        );
        assert_eq!(
            horizontal_collision_scan_left_lead_column_index_loc_36AA6(3),
            Some(2),
            "ASM loc_36AA6 should retarget `a4` to the column immediately left of the hitbox with `lea -2(a1),a4` once the left edge is in bounds",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_lead_column_mask_loc_36aae_keeps_blocking_bits_from_any_scanned_row()
     {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(2 * 4 + 0) * 2] = 0x50;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        assert_eq!(
            horizontal_collision_scan_left_lead_column_mask_loc_36AAE(&grid, 0, 1, 3) & 0x7000,
            0x5000,
            "ASM loc_36AAE should keep OR-ing the same lead-column accumulator while stepping one `Level_width_tiles` row at a time, so a later-row `$5000` slope must survive the whole inclusive scan",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_following_lead_cell_blocked_loc_36abe_treats_exact_4000_as_blocking()
     {
        assert!(
            horizontal_collision_scan_left_following_lead_cell_blocked_loc_36ABE(0x4000),
            "ASM loc_36ABE should take the exact `$4000` fast path for the following lead cell before the later mixed-kind branch runs",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_following_lead_cell_blocked_loc_36ace_keeps_mixed_collision_nibbles_blocking()
     {
        assert!(
            horizontal_collision_scan_left_following_lead_cell_blocked_loc_36ACE(0x5000),
            "ASM loc_36ACE should still treat mixed `$5000` slope words as blocking once the exact-`$4000` compare falls through",
        );
        assert!(
            !horizontal_collision_scan_left_following_lead_cell_blocked_loc_36ACE(0x3000),
            "ASM loc_36ACE should leave `d7` unchanged when the following lead cell has no bit `$4000` set",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_following_row_class_loc_36af0_starts_from_the_rightmost_word()
    {
        assert_eq!(
            horizontal_collision_scan_left_following_row_class_loc_36AF0(&[0x4000, 0x0000]),
            1,
            "ASM loc_36AF0 should inspect the rightmost following-row word first, so a clear right edge with a blocked word farther left must take the clear-entry path and return `d6=1`",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_following_row_class_loc_36b04_turns_a_blocked_entry_row_mixed()
     {
        assert_eq!(
            horizontal_collision_scan_left_following_row_class_loc_36AF0(&[0x0000, 0x5000]),
            1,
            "ASM loc_36B04 should keep pre-decrementing across the following row after a blocked right-edge entry, and the first earlier clear word must flip the result to `d6=1`",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_following_row_class_loc_36b1a_marks_mixed_rows() {
        assert_eq!(
            horizontal_collision_scan_left_following_row_class_loc_36AF0(&[0x0000, 0x4000]),
            1,
            "ASM loc_36B1A should return `d6=1` when either reverse scan sees both `$4000`-blocked and clear words in the following row",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_following_row_class_loc_36b24_scans_left_until_blocked() {
        assert_eq!(
            horizontal_collision_scan_left_following_row_class_loc_36AF0(
                &[0x4000, 0x0000, 0x0000,]
            ),
            1,
            "ASM loc_36B24 should keep pre-decrementing from a clear right edge until an earlier `$4000` word branches to the mixed-row `d6=1` exit",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_following_row_class_loc_36b2e_returns_negative_one_for_clear_rows()
     {
        assert_eq!(
            horizontal_collision_scan_left_following_row_class_loc_36AF0(&[0x0000, 0x0000]),
            -1,
            "ASM loc_36B2E should return `d6=-1` when the reverse clear-entry scan reaches the inclusive `dbf` tail without finding bit `$4000`",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_following_row_class_loc_36b0e_keeps_zero_for_all_blocked_words()
     {
        assert_eq!(
            horizontal_collision_scan_left_following_row_class_loc_36AF0(&[0x4000, 0x5000, 0x6000]),
            0,
            "ASM loc_36B0E should preserve the seeded `d6=0` when the blocked-entry reverse scan reaches the shared `dbf` epilogue without finding any earlier word that clears bit `$4000`",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_hitbox_mask_loc_36ad8_ors_every_scanned_word_in_the_leftward_body()
     {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(1 * 4 + 1) * 2] = 0x4000u16.to_be_bytes()[0];
        data[(2 * 4 + 2) * 2] = 0x6000u16.to_be_bytes()[0];
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        assert_eq!(
            horizontal_collision_scan_left_hitbox_mask_loc_36AD8(&grid, 1, 2, 1, 2) & 0x7000,
            0x6000,
            "ASM loc_36AD8 should preserve later-row blocking words while it replays the inclusive row-major body scan for the leftward rectangle",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_sub_36bd6_flags_a_slope_inside_the_hitbox() {
        let mut data = vec![0u8; 4 * 4 * 2];
        let slope_offset = (1 * 4 + 1) * 2;
        data[slope_offset] = 0x50;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let scan = horizontal_collision_scan_left_sub_36BD6(&grid, 24, 32, 8, 16);

        assert_eq!(
            scan,
            HorizontalCollisionScan {
                hitbox_blocked: true,
                lead_column_blocked: false,
            },
            "ASM sub_36BD6 should treat any bit-$4000 collision cell inside the body rectangle as a hit, including left-side slopes",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_sub_36bd6_treats_the_left_boundary_as_blocked() {
        let data = [0u8; 4 * 4 * 2];
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let scan = horizontal_collision_scan_left_sub_36BD6(&grid, 8, 32, 8, 16);

        assert_eq!(
            scan,
            HorizontalCollisionScan {
                hitbox_blocked: false,
                lead_column_blocked: true,
            },
            "ASM sub_36BD6 should return a blocked lead column once the next cell would step beyond the level's left edge",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_loc_36c38_sets_boundary_without_reading_lead_column() {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(1 * 4 + 1) * 2] = 0x40;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let scan = horizontal_collision_scan_left_sub_36BD6(&grid, 8, 32, 8, 16);

        assert_eq!(
            scan,
            HorizontalCollisionScan {
                hitbox_blocked: false,
                lead_column_blocked: true,
            },
            "ASM loc_36C38 should turn a zero left-column offset into `d6=1` before the lead-column scan, even when a blocking word exists inside the next in-bounds column",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_loc_36bf6_clamps_bottom_row_to_level_height() {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(3 * 4 + 1) * 2] = 0x40;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let scan = horizontal_collision_scan_left_sub_36BD6(&grid, 24, 80, 8, 32);

        assert_eq!(
            scan,
            HorizontalCollisionScan {
                hitbox_blocked: true,
                lead_column_blocked: false,
            },
            "ASM loc_36BF6 should clamp the bottom row to `Level_height_blocks - 1` before deriving `d2`, so the final in-bounds row still participates in the leftward body scan",
        );
    }

    #[test]
    fn horizontal_collision_scan_left_loc_36c40_ignores_the_following_row() {
        let mut data = vec![0u8; 4 * 4 * 2];
        data[(2 * 4 + 1) * 2] = 0x40;
        let grid = BlockGrid::from_level_layout(&data, 4, 4);

        let scan = horizontal_collision_scan_left_sub_36BD6(&grid, 40, 32, 8, 16);

        assert_eq!(
            scan,
            HorizontalCollisionScan {
                hitbox_blocked: false,
                lead_column_blocked: false,
            },
            "ASM loc_36C40 should scan only the in-span left lead column via `add.w d4,a4`; a blocking word on the following row below `d2` must not set `d6`",
        );
    }

    #[test]
    fn sub_140f6_height_flag_bits_expand_to_blank_top_rows() {
        assert_eq!(
            blank_top_rows_from_height_flag_bits_sub_140F6(0),
            0,
            "ASM Load_LevelData -> sub_140F6 should add no blank rows when the header flags are clear"
        );
        assert_eq!(
            blank_top_rows_from_height_flag_bits_sub_140F6(1),
            2,
            "ASM Load_LevelData rotates the top two height bits into Number_blank_top_rows, then doubles them"
        );
        assert_eq!(
            blank_top_rows_from_height_flag_bits_sub_140F6(3),
            6,
            "ASM Load_LevelData's maximum two-bit height flag value should produce six blank top rows"
        );
    }

    #[test]
    fn sub_140f6_inserts_zero_rows_above_existing_layout() {
        let shifted = insert_blank_top_rows_sub_140F6(
            &[
                0xE0, 0x01, 0x60, 0x02, // row 0
                0x20, 0x03, 0xA0, 0x04, // row 1
            ],
            2,
            2,
        );

        assert_eq!(
            shifted,
            vec![
                0x00, 0x00, 0x00, 0x00, // blank row 0
                0x00, 0x00, 0x00, 0x00, // blank row 1
                0xE0, 0x01, 0x60, 0x02, // original row 0
                0x20, 0x03, 0xA0, 0x04, // original row 1
            ],
            "ASM sub_140F6 should shift the populated rows downward and clear the inserted top rows"
        );
    }

    #[test]
    fn loc_140fe_starts_the_shift_from_the_bottom_row_with_blank_row_offset() {
        let shifted = insert_blank_top_rows_sub_140F6(
            &[
                0x10, 0x01, 0x10, 0x02, // row 0
                0x20, 0x03, 0x20, 0x04, // row 1
                0x30, 0x05, 0x30, 0x06, // row 2
            ],
            2,
            1,
        );

        assert_eq!(
            &shifted[8..12],
            &[0x20, 0x03, 0x20, 0x04],
            "ASM loc_140FE should place the middle source row one row lower after seeding `a1` by `Number_blank_top_rows * Level_width_blocks * 2` bytes"
        );
        assert_eq!(
            &shifted[12..16],
            &[0x30, 0x05, 0x30, 0x06],
            "ASM loc_140FE should start the downward copy from the bottom populated row instead of reordering the tail"
        );
    }

    #[test]
    fn loc_14138_repeats_the_row_copy_without_disturbing_source_order() {
        let shifted = insert_blank_top_rows_sub_140F6(
            &[
                0x11, 0x01, 0x11, 0x02, // row 0
                0x22, 0x03, 0x22, 0x04, // row 1
                0x33, 0x05, 0x33, 0x06, // row 2
            ],
            2,
            2,
        );

        assert_eq!(
            &shifted[..8],
            &[0x00; 8],
            "ASM loc_14138 should preserve the inserted blank rows while the outer DBF loop copies the populated rows below them"
        );
        assert_eq!(
            &shifted[8..],
            &[
                0x11, 0x01, 0x11, 0x02, // row 0
                0x22, 0x03, 0x22, 0x04, // row 1
                0x33, 0x05, 0x33, 0x06, // row 2
            ],
            "ASM loc_14138 should repeat the row copy for each source row without scrambling their original order"
        );
    }

    #[test]
    fn loc_1413a_copies_exactly_one_word_per_row_when_the_inner_dbf_counter_starts_at_zero() {
        let shifted = insert_blank_top_rows_sub_140F6(
            &[
                0x10, 0x01, // row 0
                0x20, 0x02, // row 1
            ],
            1,
            1,
        );

        assert_eq!(
            shifted,
            vec![
                0x00, 0x00, // blank row 0
                0x10, 0x01, // original row 0
                0x20, 0x02, // original row 1
            ],
            "ASM loc_1413A seeds `d1` from `Level_width_blocks-1`, so width=1 should still copy exactly one layout word before the inner `dbf d1,loc_1413A` falls through"
        );
    }

    #[test]
    fn loc_14156_repeats_the_zero_fill_for_every_requested_blank_top_row() {
        let shifted = insert_blank_top_rows_sub_140F6(
            &[
                0x10, 0x01, 0x10, 0x02, // row 0
                0x20, 0x03, 0x20, 0x04, // row 1
            ],
            2,
            3,
        );

        assert_eq!(
            &shifted[..12],
            &[0x00; 12],
            "ASM loc_14156 should step `a1` upward and repeat the zero-fill row loop until all `Number_blank_top_rows` rows at the top have been cleared"
        );
        assert_eq!(
            &shifted[12..],
            &[
                0x10, 0x01, 0x10, 0x02, // row 0
                0x20, 0x03, 0x20, 0x04, // row 1
            ],
            "ASM loc_14156 should only consume the requested blank-row count, then leave the shifted source rows intact below the cleared strip"
        );
    }

    #[test]
    fn loc_14158_still_writes_one_zero_word_when_the_blank_row_width_is_one_block() {
        let shifted = insert_blank_top_rows_sub_140F6(
            &[
                0x10, 0x01, // row 0
            ],
            1,
            2,
        );

        assert_eq!(
            &shifted[..4],
            &[0x00; 4],
            "ASM loc_14158 seeds `d1` from `Level_width_blocks-1`, so width=1 should still store one zero layout word into each blank row before the inner `dbf d1,loc_14158` falls through"
        );
        assert_eq!(
            &shifted[4..],
            &[0x10, 0x01],
            "ASM loc_14158 should only zero-fill the inserted rows and leave the shifted source row unchanged below them"
        );
    }

    #[test]
    fn loc_141aa_repeats_the_terrain_byte_row_copy_without_disturbing_source_order() {
        let shifted = insert_blank_top_rows_terrain_bytes_loc_141AA(
            &[
                0x11, 0x12, // row 0
                0x21, 0x22, // row 1
                0x31, 0x32, // row 2
            ],
            2,
            2,
        );

        assert_eq!(
            &shifted[..4],
            &[0x00; 4],
            "ASM loc_141AA should leave the inserted blank terrain rows at the top while the outer DBF loop copies the populated byte rows below them"
        );
        assert_eq!(
            &shifted[4..],
            &[
                0x11, 0x12, // row 0
                0x21, 0x22, // row 1
                0x31, 0x32, // row 2
            ],
            "ASM loc_141AA should reload `d1` from `Level_width_blocks-1` for each source row and preserve byte-row order during the downward shift"
        );
    }

    #[test]
    fn loc_141ac_still_copies_one_terrain_byte_when_the_inner_dbf_counter_starts_at_zero() {
        let shifted = insert_blank_top_rows_terrain_bytes_loc_141AA(
            &[
                0x10, // row 0
                0x20, // row 1
            ],
            1,
            1,
        );

        assert_eq!(
            shifted,
            vec![
                0x00, // blank row 0
                0x10, // original row 0
                0x20, // original row 1
            ],
            "ASM loc_141AC seeds `d1` from `Level_width_blocks-1`, so width=1 should still copy exactly one terrain byte before the inner `dbf d1,loc_141AC` falls through"
        );
    }

    #[test]
    fn loc_141c8_repeats_the_top_row_clear_once_per_inserted_blank_row() {
        let shifted = insert_blank_top_rows_terrain_bytes_loc_141AA(
            &[
                0x11, 0x12, // row 0
                0x21, 0x22, // row 1
            ],
            2,
            3,
        );

        assert_eq!(
            &shifted[..6],
            &[0x00; 6],
            "ASM loc_141C8 should rerun the row-sized zero-fill pass once per `Number_blank_top_rows`, leaving three inserted blank terrain rows before the shifted source bytes"
        );
        assert_eq!(
            &shifted[6..],
            &[
                0x11, 0x12, // row 0
                0x21, 0x22, // row 1
            ],
            "ASM loc_141C8 should only iterate the inserted blank rows, not consume any of the copied terrain payload below them"
        );
    }

    #[test]
    fn loc_141ca_still_writes_one_zero_byte_per_blank_row_when_width_is_one() {
        let shifted = insert_blank_top_rows_terrain_bytes_loc_141AA(
            &[
                0x33, // row 0
                0x44, // row 1
            ],
            1,
            2,
        );

        assert_eq!(
            shifted,
            vec![
                0x00, // blank row 0
                0x00, // blank row 1
                0x33, // shifted source row 0
                0x44, // shifted source row 1
            ],
            "ASM loc_141CA seeds `d1` from `Level_width_blocks-1`, so width=1 should still store exactly one zero byte into each inserted blank row before the inner `dbf d1,loc_141CA` falls through"
        );
    }
}
