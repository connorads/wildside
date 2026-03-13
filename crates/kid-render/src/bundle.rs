use crate::background::{
    BackgroundChunk, BackgroundVisuals, ChunkedBackgroundVisuals, compose_layered_background,
    parse_background_chunk, parse_chunked_background_layout, parse_layered_background_layout,
    parse_layered_background_plane,
};
use crate::hud::{HudIcon, HudVisuals};
use crate::level::{BlockVisualMapping, ForegroundLayout, LevelVisuals};
use crate::palette::{Palette, PaletteSet};
use crate::player::{
    AnimatedSpriteVisual, EnemyVisuals, PlatformVisuals, PlayerVisuals, SpriteFrame,
};
use crate::tiles::{TileRef, TileSheet, parse_tile_quads};
use anyhow::{Context, Result, anyhow, bail};
use kid_data::{
    AssetFile, AssetManifest, DecodedAssetFile, decode_enigma, decode_saxman,
    ensure_generated_bundle_contract, load_level_asset_bundle,
};
use sha1::{Digest, Sha1};
use std::fs;
use std::path::Path;

const LEVEL1_ID: u8 = 0x00;
const HUD_DIGIT_BASE_TILE_INDEX: u16 = 0x06BA;
const HUD_DIAMOND_BASE_TILE_INDEX: u16 = 0x06EA;
const BLOCKS_ART_PATH: &str = "artcomp/Blocks.bin";
const HUD_NUMBERS_ART_PATH: &str = "artcomp/HUD_numbers.bin";
const HUD_DIAMOND_ART_PATH: &str = "artunc/diamond_1.bin";
const DIAMOND_ENEMY_ART_PATH: &str = "artcomp/Enemy01_Diamond.bin";
const DIAMOND_ENEMY_PALETTE_PATH: &str = "palette_enemy/Crystal_1.bin";
const DRAGON_ENEMY_ART_PATH: &str = "artcomp/Enemy0C_Dragon.bin";
const DRAGON_ENEMY_PALETTE_PATH: &str = "palette_enemy/Dragon_1.bin";
const MINI_HOPPING_SKULL_ART_PATH: &str = "artcomp/Enemy1C_Mini_hopping_skull.bin";
const MINI_HOPPING_SKULL_PALETTE_PATH: &str = "palette_enemy/Mini_Hopping_Skull_1.bin";
const HORIZONTAL_PLATFORM_ART_PATH: &str = "artcomp/Horizontal_platform.bin";
const VERTICAL_PLATFORM_ART_PATH: &str = "artcomp/Vertical_platform.bin";
const KID_PALETTE_PATH: &str = "palette_kid/Kid.bin";
const BLOCK_MAPPING_PATH: &str = "block_mappings.bin";
const THEME_NAMES: [&str; 11] = [
    "theme0", "sky", "ice", "hill", "island", "desert", "swamp", "mountain", "cave", "forest",
    "city",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level1Visuals {
    pub foreground_tiles: TileSheet,
    pub background_tiles: TileSheet,
    pub block_tiles: TileSheet,
    pub palettes: PaletteSet,
    pub background: BackgroundVisuals,
    pub level: LevelVisuals,
    pub hud: HudVisuals,
    pub player: PlayerVisuals,
    pub enemies: EnemyVisuals,
    pub platforms: PlatformVisuals,
}

pub fn load_level1_visuals(manifest: &AssetManifest) -> Result<Level1Visuals> {
    ensure_generated_bundle_contract(manifest)?;
    let level_bundle = load_level_asset_bundle(manifest, LEVEL1_ID)?;
    let header = level_bundle
        .parsed_header
        .as_ref()
        .ok_or_else(|| anyhow!("generated bundle is missing Level 1 header metadata"))?;
    let decoded = manifest
        .decoded_level1
        .as_ref()
        .ok_or_else(|| anyhow!("generated bundle is missing decoded Level 1 assets"))?;

    let foreground_theme = resolve_theme_name(header.level_theme)
        .ok_or_else(|| anyhow!("unsupported foreground theme id {}", header.level_theme))?;
    let background_theme = resolve_theme_name(header.background_theme).ok_or_else(|| {
        anyhow!(
            "unsupported background theme id {}",
            header.background_theme
        )
    })?;

    let foreground_layout = match level_bundle.foreground.as_ref() {
        Some(asset) => decode_level_saxman_asset(asset, "foreground layout")?,
        None => read_decoded_file(manifest, &decoded.foreground, "foreground layout")?,
    };
    let block_layout = read_decoded_file(manifest, &decoded.block, "block layout")?;
    let background_layout = level_bundle
        .background
        .as_ref()
        .map(|asset| asset.bytes.clone())
        .unwrap_or(read_decoded_file(
            manifest,
            &decoded.background,
            "background layout",
        )?);
    let parsed_foreground_layout = ForegroundLayout::parse(
        &foreground_layout,
        header.width_screens,
        header.height_screens,
    )
    .context("failed to parse Level 1 foreground layout")?;

    let foreground_tiles = load_tile_sheet(
        manifest,
        "theme",
        find_theme_asset(manifest, &format!("artcomp_fg/{foreground_theme}.bin")).ok_or_else(
            || anyhow!("missing foreground art asset theme/artcomp_fg/{foreground_theme}.bin"),
        )?,
        "foreground tile sheet",
    )?;
    let background_tiles = load_tile_sheet(
        manifest,
        "theme",
        find_theme_asset(manifest, &format!("artcomp_bg/{background_theme}.bin")).ok_or_else(
            || anyhow!("missing background art asset theme/artcomp_bg/{background_theme}.bin"),
        )?,
        "background tile sheet",
    )?;
    let block_tiles = load_tile_sheet(
        manifest,
        "ingame",
        find_ingame_asset(manifest, BLOCKS_ART_PATH)
            .ok_or_else(|| anyhow!("missing block art asset ingame/{BLOCKS_ART_PATH}"))?,
        "block tile sheet",
    )?;
    let hud_digits = load_tile_sheet(
        manifest,
        "ingame",
        find_ingame_asset(manifest, HUD_NUMBERS_ART_PATH)
            .ok_or_else(|| anyhow!("missing HUD digit art asset ingame/{HUD_NUMBERS_ART_PATH}"))?,
        "HUD digit tile sheet",
    )?;
    let hud_diamond_tiles = load_uncompressed_tile_sheet(
        manifest,
        "ingame",
        find_ingame_asset(manifest, HUD_DIAMOND_ART_PATH).ok_or_else(|| {
            anyhow!("missing HUD diamond art asset ingame/{HUD_DIAMOND_ART_PATH}")
        })?,
        "HUD diamond tile sheet",
    )?;
    let player = load_player_visuals(manifest)?;
    let enemies = load_enemy_visuals(manifest)?;
    let platforms = load_platform_visuals(manifest, foreground_theme)?;

    let palettes = PaletteSet {
        foreground: Palette::from_bytes(&read_root_asset(
            manifest,
            "theme",
            find_theme_asset(manifest, &format!("palette_fg/{foreground_theme}.bin")).ok_or_else(
                || {
                    anyhow!(
                        "missing foreground palette asset theme/palette_fg/{foreground_theme}.bin"
                    )
                },
            )?,
            "foreground palette",
        )?)?,
        background: Palette::from_bytes(&read_root_asset(
            manifest,
            "theme",
            find_theme_asset(manifest, &format!("palette_bg/{background_theme}.bin")).ok_or_else(
                || {
                    anyhow!(
                        "missing background palette asset theme/palette_bg/{background_theme}.bin"
                    )
                },
            )?,
            "background palette",
        )?)?,
    };

    let terrain_quads = parse_tile_quads(&read_root_asset(
        manifest,
        "theme",
        find_theme_asset(manifest, &format!("mappings/{foreground_theme}.bin")).ok_or_else(
            || anyhow!("missing terrain mapping asset theme/mappings/{foreground_theme}.bin"),
        )?,
        "terrain mapping",
    )?)?;
    let block_quads = parse_tile_quads(&read_root_asset(
        manifest,
        "theme",
        find_theme_asset(manifest, BLOCK_MAPPING_PATH)
            .ok_or_else(|| anyhow!("missing block mapping asset theme/{BLOCK_MAPPING_PATH}"))?,
        "block mapping",
    )?)?;
    let background = load_background_visuals(manifest, background_theme, &background_layout)
        .context("failed to load Level 1 background visuals")?;

    Ok(Level1Visuals {
        foreground_tiles,
        background_tiles,
        block_tiles,
        palettes,
        background,
        level: LevelVisuals {
            level_id: LEVEL1_ID,
            foreground_theme_id: header.level_theme,
            background_theme_id: header.background_theme,
            background_scroll: level_bundle.parsed_bgscroll.clone(),
            foreground_layout: parsed_foreground_layout,
            foreground_layout_bytes: foreground_layout,
            block_layout_bytes: block_layout,
            background_layout_bytes: background_layout,
            block_mapping: BlockVisualMapping::new(terrain_quads, block_quads)
                .context("failed to build Level 1 block visual mapping")?,
        },
        hud: HudVisuals {
            digits: hud_digits,
            digit_base_tile_index: HUD_DIGIT_BASE_TILE_INDEX,
            diamond_tiles: hud_diamond_tiles,
            diamond_base_tile_index: HUD_DIAMOND_BASE_TILE_INDEX,
            icons: vec![
                HudIcon::new("ankh", (0x01A1, 0x008D), TileRef::from_attr_word(0x8231)),
                HudIcon::new("colon", (0x0090, 0x0090), TileRef::from_attr_word(0x86C4)),
                HudIcon::new("diamond", (0x01A0, 0x00A2), TileRef::from_attr_word(0x86F2)),
            ],
            hitpoint_tiles: [
                TileRef::from_attr_word(0x822D),
                TileRef::from_attr_word(0x822E),
                TileRef::from_attr_word(0x822F),
            ],
        },
        player,
        enemies,
        platforms,
    })
}

fn resolve_theme_name(theme_id: u8) -> Option<&'static str> {
    THEME_NAMES.get(usize::from(theme_id)).copied()
}

fn find_theme_asset<'a>(manifest: &'a AssetManifest, relative_path: &str) -> Option<&'a AssetFile> {
    manifest
        .theme_assets
        .iter()
        .find(|asset| asset.relative_path == relative_path)
}

fn find_ingame_asset<'a>(
    manifest: &'a AssetManifest,
    relative_path: &str,
) -> Option<&'a AssetFile> {
    manifest
        .ingame_assets
        .iter()
        .find(|asset| asset.relative_path == relative_path)
}

fn load_background_visuals(
    manifest: &AssetManifest,
    theme_name: &str,
    layout_bytes: &[u8],
) -> Result<BackgroundVisuals> {
    if uses_layered_backgrounds(theme_name) {
        return load_layered_background_visuals(manifest, theme_name, layout_bytes);
    }

    let placements = parse_chunked_background_layout(layout_bytes)
        .context("failed to parse chunked background layout")?;
    let prefix = format!("bg_chunks/{theme_name}_");
    let mut chunk_assets = manifest
        .theme_assets
        .iter()
        .filter_map(|asset| {
            parse_background_chunk_id(&asset.relative_path, &prefix)
                .map(|chunk_id| (chunk_id, asset))
        })
        .collect::<Vec<_>>();
    chunk_assets.sort_by_key(|(chunk_id, _)| *chunk_id);

    let max_chunk_id = chunk_assets
        .iter()
        .map(|(chunk_id, _)| *chunk_id)
        .max()
        .unwrap_or(0);
    let mut chunks = vec![
        BackgroundChunk {
            width_tiles: 0,
            height_tiles: 0,
            tiles: Vec::new(),
        };
        usize::from(max_chunk_id) + 1
    ];
    for (chunk_id, asset) in chunk_assets {
        let bytes =
            read_root_asset(manifest, "theme", asset, "background chunk").with_context(|| {
                format!(
                    "failed to read background chunk {} for theme {theme_name}",
                    asset.relative_path
                )
            })?;
        chunks[usize::from(chunk_id)] = parse_background_chunk(&bytes).with_context(|| {
            format!(
                "failed to parse background chunk {} for theme {theme_name}",
                asset.relative_path
            )
        })?;
    }

    for placement in &placements {
        let chunk = chunks.get(usize::from(placement.chunk_id)).ok_or_else(|| {
            anyhow!(
                "background layout references missing chunk id 0x{:04X} for theme {theme_name}",
                placement.chunk_id
            )
        })?;
        if chunk.tiles.is_empty() {
            bail!(
                "background layout references unloaded chunk id 0x{:04X} for theme {theme_name}",
                placement.chunk_id
            );
        }
    }

    Ok(BackgroundVisuals::Chunked(ChunkedBackgroundVisuals {
        placements,
        chunks,
    }))
}

fn load_layered_background_visuals(
    manifest: &AssetManifest,
    theme_name: &str,
    layout_bytes: &[u8],
) -> Result<BackgroundVisuals> {
    let layout = parse_layered_background_layout(layout_bytes, theme_name).with_context(|| {
        format!("failed to parse layered background layout for theme {theme_name}")
    })?;
    let prefix = format!("bg_chunks/{theme_name}_eni_");
    let mut plane_assets = manifest
        .theme_assets
        .iter()
        .filter_map(|asset| {
            parse_layered_background_chunk_id(&asset.relative_path, &prefix)
                .map(|plane_id| (plane_id, asset))
        })
        .collect::<Vec<_>>();
    plane_assets.sort_by_key(|(plane_id, _)| *plane_id);

    let mut planes = Vec::with_capacity(plane_assets.len());
    for (plane_id, asset) in plane_assets {
        let encoded = read_root_asset(manifest, "theme", asset, "layered background plane")
            .with_context(|| {
                format!(
                    "failed to read layered background plane {} for theme {theme_name}",
                    asset.relative_path
                )
            })?;
        let decoded = decode_enigma(&encoded).with_context(|| {
            format!(
                "failed to decode layered background plane {} for theme {theme_name}",
                asset.relative_path
            )
        })?;
        planes.push(parse_layered_background_plane(&decoded).with_context(|| {
            format!(
                "failed to parse layered background plane {plane_id:02X} for theme {theme_name}"
            )
        })?);
    }

    Ok(BackgroundVisuals::Layered(
        compose_layered_background(&layout, &planes).with_context(|| {
            format!("failed to compose layered background rows for theme {theme_name}")
        })?,
    ))
}

fn uses_layered_backgrounds(theme_name: &str) -> bool {
    matches!(theme_name, "hill" | "desert" | "mountain" | "forest")
}

#[derive(Clone, Copy)]
struct PlayerFrameDescriptor {
    relative_path: &'static str,
    origin_x: i16,
    origin_y: i16,
    width_px: u16,
    height_px: u16,
}

#[derive(Clone, Copy)]
struct SpriteFrameDescriptor {
    origin_x: i16,
    origin_y: i16,
    width_px: u16,
    height_px: u16,
    tile_base_index: u16,
}

const DIAMOND_FRAMES: [SpriteFrameDescriptor; 3] = [
    SpriteFrameDescriptor {
        origin_x: 0x0A,
        origin_y: 0x00,
        width_px: 0x14,
        height_px: 0x14,
        tile_base_index: 0x002F,
    },
    SpriteFrameDescriptor {
        origin_x: 0x0A,
        origin_y: 0x00,
        width_px: 0x14,
        height_px: 0x14,
        tile_base_index: 0x003B,
    },
    SpriteFrameDescriptor {
        origin_x: 0x0A,
        origin_y: 0x00,
        width_px: 0x14,
        height_px: 0x14,
        tile_base_index: 0x0047,
    },
];

const MINI_HOPPING_SKULL_FRAMES: [SpriteFrameDescriptor; 2] = [
    SpriteFrameDescriptor {
        origin_x: 0x07,
        origin_y: 0x00,
        width_px: 0x0F,
        height_px: 0x11,
        tile_base_index: 0x002C,
    },
    SpriteFrameDescriptor {
        origin_x: 0x08,
        origin_y: 0x00,
        width_px: 0x0F,
        height_px: 0x12,
        tile_base_index: 0x0032,
    },
];

const DRAGON_FLYING_FRAMES: [SpriteFrameDescriptor; 5] = [
    SpriteFrameDescriptor {
        origin_x: 0x0D,
        origin_y: 0x10,
        width_px: 0x2E,
        height_px: 0x1E,
        tile_base_index: 0x011D,
    },
    SpriteFrameDescriptor {
        origin_x: 0x0D,
        origin_y: 0x10,
        width_px: 0x30,
        height_px: 0x19,
        tile_base_index: 0x0135,
    },
    SpriteFrameDescriptor {
        origin_x: 0x0E,
        origin_y: 0x10,
        width_px: 0x2F,
        height_px: 0x1A,
        tile_base_index: 0x014D,
    },
    SpriteFrameDescriptor {
        origin_x: 0x0D,
        origin_y: 0x10,
        width_px: 0x2E,
        height_px: 0x19,
        tile_base_index: 0x0165,
    },
    SpriteFrameDescriptor {
        origin_x: 0x0D,
        origin_y: 0x10,
        width_px: 0x2E,
        height_px: 0x1E,
        tile_base_index: 0x017D,
    },
];

const HORIZONTAL_PLATFORM_FRAME: SpriteFrameDescriptor = SpriteFrameDescriptor {
    origin_x: 0,
    origin_y: 0,
    width_px: 0x48,
    height_px: 0x10,
    tile_base_index: 0,
};

const VERTICAL_PLATFORM_FRAME: SpriteFrameDescriptor = SpriteFrameDescriptor {
    origin_x: 0,
    origin_y: 0,
    width_px: 0x20,
    height_px: 0x20,
    tile_base_index: 0,
};

const KID_IDLE_FRAME: PlayerFrameDescriptor = PlayerFrameDescriptor {
    relative_path: "artunc_kid/kid.bin",
    origin_x: 0x0B,
    origin_y: 0x01,
    width_px: 0x14,
    height_px: 0x20,
};

const KID_WALK_FRAMES: [PlayerFrameDescriptor; 6] = [
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_1.bin",
        origin_x: 0x0A,
        origin_y: 0x01,
        width_px: 0x13,
        height_px: 0x1F,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_2.bin",
        origin_x: 0x08,
        origin_y: 0x01,
        width_px: 0x11,
        height_px: 0x20,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_3.bin",
        origin_x: 0x08,
        origin_y: 0x01,
        width_px: 0x11,
        height_px: 0x20,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_4.bin",
        origin_x: 0x0A,
        origin_y: 0x01,
        width_px: 0x14,
        height_px: 0x1F,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_5.bin",
        origin_x: 0x08,
        origin_y: 0x01,
        width_px: 0x11,
        height_px: 0x20,
    },
    PlayerFrameDescriptor {
        relative_path: "artunc_kid/kid_walk_6.bin",
        origin_x: 0x08,
        origin_y: 0x01,
        width_px: 0x11,
        height_px: 0x20,
    },
];

const KID_JUMP_RISE_FRAME: PlayerFrameDescriptor = PlayerFrameDescriptor {
    relative_path: "artunc_kid/kid_jumping_1.bin",
    origin_x: 0x0A,
    origin_y: 0x00,
    width_px: 0x16,
    height_px: 0x20,
};

const KID_JUMP_FALL_FRAME: PlayerFrameDescriptor = PlayerFrameDescriptor {
    relative_path: "artunc_kid/kid_jumping_2.bin",
    origin_x: 0x09,
    origin_y: 0x00,
    width_px: 0x14,
    height_px: 0x20,
};

fn load_enemy_visuals(manifest: &AssetManifest) -> Result<EnemyVisuals> {
    Ok(EnemyVisuals {
        diamond: AnimatedSpriteVisual {
            palette: load_ingame_palette_asset(
                manifest,
                DIAMOND_ENEMY_PALETTE_PATH,
                "diamond enemy palette",
            )?,
            frames: load_compressed_sprite_frames(
                manifest,
                DIAMOND_ENEMY_ART_PATH,
                "diamond enemy art",
                &DIAMOND_FRAMES,
            )?,
            frame_duration: 6,
        },
        dragon: AnimatedSpriteVisual {
            palette: load_ingame_palette_asset(
                manifest,
                DRAGON_ENEMY_PALETTE_PATH,
                "dragon enemy palette",
            )?,
            frames: load_compressed_sprite_frames(
                manifest,
                DRAGON_ENEMY_ART_PATH,
                "dragon enemy art",
                &DRAGON_FLYING_FRAMES,
            )?,
            frame_duration: 7,
        },
        mini_hopping_skull: AnimatedSpriteVisual {
            palette: load_ingame_palette_asset(
                manifest,
                MINI_HOPPING_SKULL_PALETTE_PATH,
                "mini hopping skull palette",
            )?,
            frames: load_compressed_sprite_frames(
                manifest,
                MINI_HOPPING_SKULL_ART_PATH,
                "mini hopping skull art",
                &MINI_HOPPING_SKULL_FRAMES,
            )?,
            frame_duration: 9,
        },
    })
}

fn load_platform_visuals(
    manifest: &AssetManifest,
    foreground_theme: &str,
) -> Result<PlatformVisuals> {
    let palette = Palette::from_bytes(&read_root_asset(
        manifest,
        "theme",
        find_theme_asset(manifest, &format!("palette_fg/{foreground_theme}.bin")).ok_or_else(
            || anyhow!("missing platform palette asset theme/palette_fg/{foreground_theme}.bin"),
        )?,
        "platform palette",
    )?)?;

    Ok(PlatformVisuals {
        palette,
        horizontal: load_compressed_sprite_frame(
            manifest,
            HORIZONTAL_PLATFORM_ART_PATH,
            "horizontal platform art",
            HORIZONTAL_PLATFORM_FRAME,
        )?,
        vertical: load_compressed_sprite_frame(
            manifest,
            VERTICAL_PLATFORM_ART_PATH,
            "vertical platform art",
            VERTICAL_PLATFORM_FRAME,
        )?,
    })
}

fn load_player_visuals(manifest: &AssetManifest) -> Result<PlayerVisuals> {
    let palette = Palette::from_bytes(&read_root_asset(
        manifest,
        "ingame",
        find_ingame_asset(manifest, KID_PALETTE_PATH)
            .ok_or_else(|| anyhow!("missing Kid palette asset ingame/{KID_PALETTE_PATH}"))?,
        "Kid palette",
    )?)?;

    Ok(PlayerVisuals {
        palette,
        idle: load_player_frame(manifest, KID_IDLE_FRAME)?,
        walk: KID_WALK_FRAMES
            .into_iter()
            .map(|descriptor| load_player_frame(manifest, descriptor))
            .collect::<Result<Vec<_>>>()?,
        jump_rise: load_player_frame(manifest, KID_JUMP_RISE_FRAME)?,
        jump_fall: load_player_frame(manifest, KID_JUMP_FALL_FRAME)?,
    })
}

fn load_player_frame(
    manifest: &AssetManifest,
    descriptor: PlayerFrameDescriptor,
) -> Result<SpriteFrame> {
    let asset = find_ingame_asset(manifest, descriptor.relative_path).ok_or_else(|| {
        anyhow!(
            "missing Kid sprite asset ingame/{}",
            descriptor.relative_path
        )
    })?;
    let tile_sheet = load_uncompressed_tile_sheet(
        manifest,
        "ingame",
        asset,
        &format!("Kid sprite frame {}", descriptor.relative_path),
    )?;
    Ok(SpriteFrame {
        origin_x: descriptor.origin_x,
        origin_y: descriptor.origin_y,
        width_px: descriptor.width_px,
        height_px: descriptor.height_px,
        tile_base_index: 0,
        tile_sheet,
    })
}

fn load_ingame_palette_asset(
    manifest: &AssetManifest,
    relative_path: &str,
    family: &str,
) -> Result<Palette> {
    Palette::from_bytes(&read_root_asset(
        manifest,
        "ingame",
        find_ingame_asset(manifest, relative_path)
            .ok_or_else(|| anyhow!("missing {family} asset ingame/{relative_path}"))?,
        family,
    )?)
}

fn load_compressed_sprite_frames(
    manifest: &AssetManifest,
    relative_path: &str,
    family: &str,
    descriptors: &[SpriteFrameDescriptor],
) -> Result<Vec<SpriteFrame>> {
    let tile_sheet = load_compressed_sprite_sheet(manifest, relative_path, family)?;
    Ok(descriptors
        .iter()
        .copied()
        .map(|descriptor| SpriteFrame {
            origin_x: descriptor.origin_x,
            origin_y: descriptor.origin_y,
            width_px: descriptor.width_px,
            height_px: descriptor.height_px,
            tile_base_index: descriptor.tile_base_index,
            tile_sheet: tile_sheet.clone(),
        })
        .collect())
}

fn load_compressed_sprite_frame(
    manifest: &AssetManifest,
    relative_path: &str,
    family: &str,
    descriptor: SpriteFrameDescriptor,
) -> Result<SpriteFrame> {
    Ok(SpriteFrame {
        origin_x: descriptor.origin_x,
        origin_y: descriptor.origin_y,
        width_px: descriptor.width_px,
        height_px: descriptor.height_px,
        tile_base_index: descriptor.tile_base_index,
        tile_sheet: load_compressed_sprite_sheet(manifest, relative_path, family)?,
    })
}

fn load_compressed_sprite_sheet(
    manifest: &AssetManifest,
    relative_path: &str,
    family: &str,
) -> Result<TileSheet> {
    let asset = find_ingame_asset(manifest, relative_path)
        .ok_or_else(|| anyhow!("missing {family} asset ingame/{relative_path}"))?;
    load_tile_sheet(manifest, "ingame", asset, family)
}

fn parse_background_chunk_id(relative_path: &str, prefix: &str) -> Option<u16> {
    let suffix = relative_path.strip_prefix(prefix)?.strip_suffix(".bin")?;
    if suffix.contains("_eni_") {
        return None;
    }
    u16::from_str_radix(suffix, 16).ok()
}

fn parse_layered_background_chunk_id(relative_path: &str, prefix: &str) -> Option<u16> {
    let suffix = relative_path.strip_prefix(prefix)?.strip_suffix(".bin")?;
    u16::from_str_radix(suffix, 16).ok()
}

fn load_tile_sheet(
    manifest: &AssetManifest,
    root_name: &str,
    asset: &AssetFile,
    family: &str,
) -> Result<TileSheet> {
    let compressed = read_root_asset(manifest, root_name, asset, family)?;
    let decoded = decode_saxman(&compressed).with_context(|| {
        format!(
            "failed to decode {family} from {root_name}/{}",
            asset.relative_path
        )
    })?;
    TileSheet::from_4bpp_bytes(&decoded).with_context(|| {
        format!(
            "failed to parse {family} from {root_name}/{}",
            asset.relative_path
        )
    })
}

fn load_uncompressed_tile_sheet(
    manifest: &AssetManifest,
    root_name: &str,
    asset: &AssetFile,
    family: &str,
) -> Result<TileSheet> {
    let bytes = read_root_asset(manifest, root_name, asset, family)?;
    TileSheet::from_4bpp_bytes(&bytes).with_context(|| {
        format!(
            "failed to parse {family} from {root_name}/{}",
            asset.relative_path
        )
    })
}

fn decode_level_saxman_asset(asset: &kid_data::LoadedAsset, family: &str) -> Result<Vec<u8>> {
    decode_saxman(&asset.bytes)
        .with_context(|| format!("failed to decode {family} from {}", asset.relative_path))
}

fn read_decoded_file(
    manifest: &AssetManifest,
    asset: &DecodedAssetFile,
    family: &str,
) -> Result<Vec<u8>> {
    let path = manifest.output_root.join(&asset.relative_path);
    read_file_with_contract(
        &path,
        &asset.relative_path,
        asset.size_bytes,
        &asset.sha1,
        family,
    )
}

fn read_root_asset(
    manifest: &AssetManifest,
    root_name: &str,
    asset: &AssetFile,
    family: &str,
) -> Result<Vec<u8>> {
    let root = match root_name {
        "theme" => manifest.output_root.join("theme"),
        "ingame" => manifest.output_root.join("ingame"),
        other => bail!("unsupported generated bundle root {other}"),
    };
    let path = root.join(&asset.relative_path);
    read_file_with_contract(
        &path,
        &format!("{root_name}/{}", asset.relative_path),
        asset.size_bytes,
        &asset.sha1,
        family,
    )
}

fn read_file_with_contract(
    path: &Path,
    display_path: &str,
    expected_size: u64,
    expected_sha1: &str,
    family: &str,
) -> Result<Vec<u8>> {
    if !path.is_file() {
        bail!("missing {family} asset at {display_path}");
    }
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read {family} asset from {}", path.display()))?;
    if bytes.len() as u64 != expected_size {
        bail!(
            "{family} asset size mismatch for {display_path}: expected {expected_size} bytes, got {}",
            bytes.len()
        );
    }
    let actual_sha1 = sha1_hex(&bytes);
    if actual_sha1 != expected_sha1 {
        bail!(
            "{family} asset hash mismatch for {display_path}: expected {expected_sha1}, got {actual_sha1}"
        );
    }
    Ok(bytes)
}

fn sha1_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
