use crate::tiles::TileQuad;
use anyhow::{Result, bail};

const BLOCK_VARIANT_SKIN_COUNT: usize = 16;
const BLOCK_VARIANT_GROUP_COUNT: usize = 5;
const BLOCK_MAPPING_QUAD_COUNT: usize = 1 + BLOCK_VARIANT_SKIN_COUNT * BLOCK_VARIANT_GROUP_COUNT;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelVisuals {
    pub level_id: u8,
    pub foreground_theme_id: u8,
    pub background_theme_id: u8,
    pub foreground_layout_bytes: Vec<u8>,
    pub block_layout_bytes: Vec<u8>,
    pub background_layout_bytes: Vec<u8>,
    pub block_mapping: BlockVisualMapping,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockVisualMapping {
    terrain_by_skin: Vec<TileQuad>,
    generic_block: TileQuad,
    smash_by_skin: Vec<TileQuad>,
    prize_by_skin: Vec<TileQuad>,
    ice_by_skin: Vec<TileQuad>,
    metal_by_skin: Vec<TileQuad>,
    rubber_by_skin: Vec<TileQuad>,
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

impl BlockVisualMapping {
    pub fn new(terrain_by_skin: Vec<TileQuad>, block_quads: Vec<TileQuad>) -> Result<Self> {
        if block_quads.len() != BLOCK_MAPPING_QUAD_COUNT {
            bail!(
                "expected {BLOCK_MAPPING_QUAD_COUNT} block mapping quads, got {}",
                block_quads.len()
            );
        }

        let mut block_quads = block_quads.into_iter();
        let generic_block = block_quads
            .next()
            .expect("validated block mapping quad count includes the generic block");

        Ok(Self {
            terrain_by_skin,
            generic_block,
            smash_by_skin: take_block_skin_group(&mut block_quads),
            prize_by_skin: take_block_skin_group(&mut block_quads),
            ice_by_skin: take_block_skin_group(&mut block_quads),
            metal_by_skin: take_block_skin_group(&mut block_quads),
            rubber_by_skin: take_block_skin_group(&mut block_quads),
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
                let kind = block_visual_kind(block_type);
                match kind {
                    BlockVisualKind::Terrain => unreachable!(),
                    BlockVisualKind::GenericBlock => Some(ResolvedBlockVisual {
                        kind,
                        quad: &self.generic_block,
                    }),
                    BlockVisualKind::Smash => Some(ResolvedBlockVisual {
                        kind,
                        quad: self.smash_by_skin.get(index).unwrap_or(&self.generic_block),
                    }),
                    BlockVisualKind::Prize => Some(ResolvedBlockVisual {
                        kind,
                        quad: self.prize_by_skin.get(index).unwrap_or(&self.generic_block),
                    }),
                    BlockVisualKind::Ice => Some(ResolvedBlockVisual {
                        kind,
                        quad: self.ice_by_skin.get(index).unwrap_or(&self.generic_block),
                    }),
                    BlockVisualKind::Metal => Some(ResolvedBlockVisual {
                        kind,
                        quad: self.metal_by_skin.get(index).unwrap_or(&self.generic_block),
                    }),
                    BlockVisualKind::Rubber => Some(ResolvedBlockVisual {
                        kind,
                        quad: self
                            .rubber_by_skin
                            .get(index)
                            .unwrap_or(&self.generic_block),
                    }),
                }
            }
        }
    }
}

fn take_block_skin_group(block_quads: &mut impl Iterator<Item = TileQuad>) -> Vec<TileQuad> {
    block_quads.take(BLOCK_VARIANT_SKIN_COUNT).collect()
}

fn block_visual_kind(block_type: u8) -> BlockVisualKind {
    match block_type {
        0 => BlockVisualKind::Smash,
        1 => BlockVisualKind::Prize,
        2 => BlockVisualKind::Ice,
        5 => BlockVisualKind::Metal,
        6 => BlockVisualKind::Rubber,
        _ => BlockVisualKind::GenericBlock,
    }
}
