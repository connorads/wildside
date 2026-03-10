use crate::tiles::{TileRef, TileSheet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudVisuals {
    pub digits: TileSheet,
    pub digit_base_tile_index: u16,
    pub icons: Vec<HudIcon>,
    pub hitpoint_tiles: [TileRef; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudIcon {
    pub name: String,
    pub position: (u16, u16),
    pub tile: TileRef,
}

impl HudIcon {
    pub fn new(name: &str, position: (u16, u16), tile: TileRef) -> Self {
        Self {
            name: name.to_owned(),
            position,
            tile,
        }
    }
}
