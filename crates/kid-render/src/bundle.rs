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

const HUD_DIGIT_BASE_TILE_INDEX: u16 = 0x06BA;
const HUD_DIAMOND_BASE_TILE_INDEX: u16 = 0x06EA;
const BLOCKS_ART_PATH: &str = "artcomp/Blocks.bin";
const HUD_NUMBERS_ART_PATH: &str = "artcomp/HUD_numbers.bin";
const HUD_DIAMOND_ART_PATH: &str = "artunc/diamond_1.bin";
// Enemy art/palette paths removed — now defined inline in ENEMY_FAMILIES table.
const HORIZONTAL_PLATFORM_ART_PATH: &str = "artcomp/Horizontal_platform.bin";
const VERTICAL_PLATFORM_ART_PATH: &str = "artcomp/Vertical_platform.bin";
const KID_PALETTE_PATH: &str = "palette_kid/Kid.bin";
const BLOCK_MAPPING_PATH: &str = "block_mappings.bin";
const THEME_NAMES: [&str; 11] = [
    "theme0", "sky", "ice", "hill", "island", "desert", "swamp", "mountain", "cave", "forest",
    "city",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLevelVisuals {
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

pub fn load_level_visuals(manifest: &AssetManifest, level_id: u8) -> Result<NativeLevelVisuals> {
    ensure_generated_bundle_contract(manifest)?;
    let level_bundle = load_level_asset_bundle(manifest, level_id)?;
    let header = level_bundle
        .parsed_header
        .as_ref()
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} is missing header metadata"))?;
    let decoded = manifest.decoded_level1.as_ref();

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
        None => match decoded {
            Some(d) => read_decoded_file(manifest, &d.foreground, "foreground layout")?,
            None => bail!("level 0x{level_id:02X} has no foreground layout data"),
        },
    };
    let block_layout = match level_bundle.block.as_ref() {
        Some(asset) => asset.bytes.clone(),
        None => match decoded {
            Some(d) => read_decoded_file(manifest, &d.block, "block layout")?,
            None => bail!("level 0x{level_id:02X} has no block layout data"),
        },
    };
    let background_layout = match level_bundle.background.as_ref() {
        Some(asset) => asset.bytes.clone(),
        None => match decoded {
            Some(d) => read_decoded_file(manifest, &d.background, "background layout")?,
            None => bail!("level 0x{level_id:02X} has no background layout data"),
        },
    };
    let parsed_foreground_layout = ForegroundLayout::parse(
        &foreground_layout,
        header.width_screens,
        header.height_screens,
    )
    .context(format!(
        "failed to parse level 0x{level_id:02X} foreground layout"
    ))?;

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
        .context(format!(
            "failed to load level 0x{level_id:02X} background visuals"
        ))?;

    Ok(NativeLevelVisuals {
        foreground_tiles,
        background_tiles,
        block_tiles,
        palettes,
        background,
        level: LevelVisuals {
            level_id,
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

/// Enemy family specification: object_type, art path, palette path, frame duration, frames.
/// Frame data extracted from kid-chameleon-disasm sprite_frame_vram definitions.
struct EnemyFamilySpec {
    object_type: u8,
    art_path: &'static str,
    palette_path: &'static str,
    /// Extra palette variant paths (variants 1 and 2).
    extra_palette_paths: &'static [&'static str],
    frame_duration: usize,
    frames: &'static [SpriteFrameDescriptor],
}

const fn fd(
    tile_base_index: u16,
    origin_x: i16,
    origin_y: i16,
    width_px: u16,
    height_px: u16,
) -> SpriteFrameDescriptor {
    SpriteFrameDescriptor {
        origin_x,
        origin_y,
        width_px,
        height_px,
        tile_base_index,
    }
}

static ENEMY_FAMILIES: &[EnemyFamilySpec] = &[
    EnemyFamilySpec {
        object_type: 0x00, // Fire Demon
        art_path: "artcomp/Enemy00_Fire_demon.bin",
        palette_path: "palette_enemy/Fire_Demon_1_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Fire_Demon_1_2.bin",
            "palette_enemy/Fire_Demon_1_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x000, 0x0E, 0x00, 0x1A, 0x20),
            fd(0x010, 0x0D, 0x00, 0x1A, 0x20),
            fd(0x020, 0x0C, 0x00, 0x1A, 0x20),
            fd(0x030, 0x0C, 0x00, 0x1E, 0x20),
            fd(0x020, 0x0C, 0x00, 0x1A, 0x20),
            fd(0x010, 0x0D, 0x00, 0x1A, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x01, // Diamond (collectible)
        art_path: "artcomp/Enemy01_Diamond.bin",
        palette_path: "palette_enemy/Crystal_1.bin",
        extra_palette_paths: &["palette_enemy/Crystal_2.bin", "palette_enemy/Crystal_3.bin"],
        frame_duration: 6,
        frames: &[
            fd(0x02F, 0x0A, 0x00, 0x14, 0x20),
            fd(0x03B, 0x0A, 0x00, 0x14, 0x20),
            fd(0x047, 0x0A, 0x00, 0x14, 0x20),
            fd(0x053, 0x0A, 0x00, 0x14, 0x20),
            fd(0x05F, 0x0A, 0x00, 0x14, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x03, // Robot
        art_path: "artcomp/Enemy03_Robot.bin",
        palette_path: "palette_enemy/Plethora.bin",
        extra_palette_paths: &[],
        frame_duration: 9,
        frames: &[
            fd(0x039, 0x12, 0x00, 0x1D, 0x14),
            fd(0x045, 0x0A, 0x00, 0x12, 0x15),
            fd(0x04E, 0x10, 0x00, 0x1C, 0x15),
            fd(0x05A, 0x12, 0x00, 0x1D, 0x14),
            fd(0x066, 0x0A, 0x00, 0x15, 0x15),
            fd(0x06F, 0x10, 0x00, 0x1C, 0x15),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x04, // Armadillo
        art_path: "artcomp/Enemy04_Armadillo.bin",
        palette_path: "palette_enemy/Armadillo_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Armadillo_2.bin",
            "palette_enemy/Armadillo_3.bin",
        ],
        frame_duration: 11,
        frames: &[
            fd(0x0A5, 0x15, 0x00, 0x27, 0x17),
            fd(0x0B4, 0x15, 0x00, 0x28, 0x18),
            fd(0x0C3, 0x15, 0x00, 0x28, 0x18),
            fd(0x0D2, 0x15, 0x00, 0x28, 0x17),
            fd(0x0E1, 0x15, 0x00, 0x28, 0x18),
            fd(0x0F0, 0x15, 0x00, 0x28, 0x18),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x05, // Tar Monster
        art_path: "artcomp/Enemy05_Tar_monster.bin",
        palette_path: "palette_enemy/Tar_Monster_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Tar_Monster_2.bin",
            "palette_enemy/Tar_Monster_3.bin",
        ],
        frame_duration: 3,
        frames: &[
            fd(0x0AC, 0x11, 0x00, 0x1B, 0x20),
            fd(0x044, 0x10, 0x00, 0x1B, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x06, // Sphere
        art_path: "artcomp/Enemy06_Sphere.bin",
        palette_path: "palette_enemy/Sphere_1.bin",
        extra_palette_paths: &["palette_enemy/Sphere_2.bin", "palette_enemy/Sphere_3.bin"],
        frame_duration: 9,
        frames: &[
            fd(0x012, 0x08, 0x09, 0x10, 0x10),
            fd(0x016, 0x08, 0x09, 0x10, 0x10),
            fd(0x01A, 0x08, 0x09, 0x10, 0x10),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x07, // Archer (skeleton)
        art_path: "artcomp/Enemy07_Archer.bin",
        palette_path: "palette_enemy/Archer_1.bin",
        extra_palette_paths: &["palette_enemy/Archer_2.bin", "palette_enemy/Archer_3.bin"],
        frame_duration: 8,
        frames: &[
            fd(0x030, 0x0F, 0x00, 0x1D, 0x1F),
            fd(0x040, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x050, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x060, 0x0E, 0x00, 0x1B, 0x1F),
            fd(0x070, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x080, 0x0E, 0x00, 0x1B, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x08, // Orca
        art_path: "artcomp/Enemy08_Orca.bin",
        palette_path: "palette_enemy/Orca_1.bin",
        extra_palette_paths: &[],
        frame_duration: 9,
        frames: &[
            fd(0x05C, 0x10, 0x00, 0x1C, 0x25),
            fd(0x070, 0x10, 0x00, 0x1C, 0x26),
            fd(0x084, 0x10, 0x00, 0x1D, 0x26),
            fd(0x098, 0x10, 0x00, 0x1E, 0x25),
            fd(0x0AC, 0x10, 0x00, 0x1C, 0x26),
            fd(0x0C0, 0x10, 0x00, 0x1C, 0x26),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x09, // Crab
        art_path: "artcomp/Enemy09_Crab.bin",
        palette_path: "palette_enemy/Crab_1.bin",
        extra_palette_paths: &["palette_enemy/Crab_2.bin", "palette_enemy/Crab_3.bin"],
        frame_duration: 8,
        frames: &[
            fd(0x030, 0x0F, 0x00, 0x1D, 0x1F),
            fd(0x040, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x050, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x060, 0x0E, 0x00, 0x1B, 0x1F),
            fd(0x070, 0x0E, 0x00, 0x1B, 0x20),
            fd(0x080, 0x0E, 0x00, 0x1B, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0A, // Rock Tank
        art_path: "artcomp/Enemy0A_Rock_tank.bin",
        palette_path: "palette_enemy/Rock_tank_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Rock_tank_2.bin",
            "palette_enemy/Rock_tank_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x02C, 0x0F, 0x00, 0x1E, 0x08),
            fd(0x030, 0x0F, 0x00, 0x1E, 0x08),
            fd(0x034, 0x0F, 0x00, 0x1E, 0x08),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0B, // Rock Tank (shooting)
        art_path: "artcomp/Enemy0A_Rock_tank.bin",
        palette_path: "palette_enemy/Rock_tank_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Rock_tank_2.bin",
            "palette_enemy/Rock_tank_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x02C, 0x0F, 0x00, 0x1E, 0x08),
            fd(0x030, 0x0F, 0x00, 0x1E, 0x08),
            fd(0x034, 0x0F, 0x00, 0x1E, 0x08),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0C, // Dragon (flying)
        art_path: "artcomp/Enemy0C_Dragon.bin",
        palette_path: "palette_enemy/Dragon_1.bin",
        extra_palette_paths: &["palette_enemy/Dragon_2.bin", "palette_enemy/Dragon_3.bin"],
        frame_duration: 7,
        frames: &[
            fd(0x11D, 0x0D, 0x10, 0x2E, 0x1E),
            fd(0x135, 0x0D, 0x10, 0x30, 0x19),
            fd(0x14D, 0x0E, 0x10, 0x2F, 0x1A),
            fd(0x165, 0x0D, 0x10, 0x2E, 0x19),
            fd(0x17D, 0x0D, 0x10, 0x2E, 0x1E),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0D, // Dragon (walking)
        art_path: "artcomp/Enemy0C_Dragon.bin",
        palette_path: "palette_enemy/Dragon_1.bin",
        extra_palette_paths: &["palette_enemy/Dragon_2.bin", "palette_enemy/Dragon_3.bin"],
        frame_duration: 9,
        frames: &[
            fd(0x060, 0x10, 0x00, 0x1F, 0x26),
            fd(0x074, 0x10, 0x00, 0x1F, 0x27),
            fd(0x088, 0x10, 0x00, 0x20, 0x28),
            fd(0x09C, 0x10, 0x00, 0x20, 0x27),
            fd(0x0B0, 0x10, 0x00, 0x20, 0x28),
            fd(0x0C4, 0x10, 0x00, 0x1F, 0x27),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0E, // Cloud
        art_path: "artcomp/Enemy0E_Cloud.bin",
        palette_path: "palette_enemy/Cloud_1.bin",
        extra_palette_paths: &["palette_enemy/Cloud_2.bin", "palette_enemy/Cloud_3.bin"],
        frame_duration: 11,
        frames: &[
            fd(0x095, 0x10, 0x00, 0x1F, 0x1F),
            fd(0x0A5, 0x10, -1, 0x20, 0x1E),
            fd(0x0B5, 0x10, -1, 0x20, 0x1E),
            fd(0x0C5, 0x10, -2, 0x20, 0x1D),
            fd(0x0B5, 0x10, -1, 0x20, 0x1E),
            fd(0x0A5, 0x10, -1, 0x20, 0x1E),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x0F, // UFO
        art_path: "artcomp/Enemy0F_UFO.bin",
        palette_path: "palette_enemy/UFO_1.bin",
        extra_palette_paths: &["palette_enemy/UFO_2.bin", "palette_enemy/UFO_3.bin"],
        frame_duration: 5,
        frames: &[
            fd(0x15A, 0x10, 0x00, 0x20, 0x1D),
            fd(0x16A, 0x10, 0x01, 0x20, 0x1E),
            fd(0x17A, 0x10, 0x02, 0x20, 0x1F),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x10, // Goat
        art_path: "artcomp/Enemy10_Goat.bin",
        palette_path: "palette_enemy/Goat_1.bin",
        extra_palette_paths: &["palette_enemy/Goat_2.bin", "palette_enemy/Goat_3.bin"],
        frame_duration: 8,
        frames: &[
            fd(0x0A8, 0x0F, 0x00, 0x1E, 0x1F),
            fd(0x0B8, 0x0F, 0x00, 0x1E, 0x20),
            fd(0x0C8, 0x10, 0x00, 0x1F, 0x1F),
            fd(0x0D8, 0x11, 0x00, 0x20, 0x1F),
            fd(0x0E8, 0x10, 0x00, 0x1F, 0x20),
            fd(0x0F8, 0x0F, 0x00, 0x1E, 0x1F),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x11, // Ninja
        art_path: "artcomp/Enemy11_Ninja.bin",
        palette_path: "palette_enemy/Ninja_1.bin",
        extra_palette_paths: &["palette_enemy/Ninja_2.bin", "palette_enemy/Ninja_3.bin"],
        frame_duration: 8,
        frames: &[
            fd(0x078, 0x0C, 0x00, 0x1A, 0x1F),
            fd(0x088, 0x0D, 0x00, 0x1A, 0x20),
            fd(0x098, 0x09, 0x00, 0x16, 0x20),
            fd(0x0A4, 0x0A, 0x00, 0x17, 0x1F),
            fd(0x0B0, 0x0A, 0x00, 0x18, 0x20),
            fd(0x0BC, 0x0D, 0x00, 0x1C, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x12, // Lion
        art_path: "artcomp/Enemy12_Lion.bin",
        palette_path: "palette_enemy/Lion_1.bin",
        extra_palette_paths: &["palette_enemy/Lion_2.bin", "palette_enemy/Lion_3.bin"],
        frame_duration: 7,
        frames: &[
            fd(0x16C, 0x15, 0x00, 0x2C, 0x2D),
            fd(0x190, 0x16, 0x00, 0x2C, 0x2E),
            fd(0x1B4, 0x15, 0x00, 0x2B, 0x2E),
            fd(0x1D8, 0x14, 0x00, 0x2D, 0x2D),
            fd(0x1FC, 0x15, 0x00, 0x28, 0x2E),
            fd(0x21A, 0x15, 0x00, 0x2B, 0x2E),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x13, // Scorpion
        art_path: "artcomp/Enemy13_Scorpion.bin",
        palette_path: "palette_enemy/Scorpion_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Scorpion_2.bin",
            "palette_enemy/Scorpion_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x04C, 0x18, 0x00, 0x30, 0x1F),
            fd(0x064, 0x16, 0x00, 0x2E, 0x20),
            fd(0x07C, 0x16, 0x00, 0x30, 0x20),
            fd(0x094, 0x16, 0x00, 0x30, 0x1F),
            fd(0x0AC, 0x16, 0x00, 0x2F, 0x20),
            fd(0x0C4, 0x16, 0x00, 0x2F, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x14, // Spinning Twins
        art_path: "artcomp/Enemy14_Spinning_twins.bin",
        palette_path: "palette_enemy/Spinning_Twins_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Spinning_Twins_2.bin",
            "palette_enemy/Spinning_Twins_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x0A2, 0x20, 0x00, 0x40, 0x1F),
            fd(0x0C2, 0x19, 0x00, 0x34, 0x1F),
            fd(0x0DE, 0x0F, 0x00, 0x1E, 0x1F),
            fd(0x0EE, 0x1E, 0x00, 0x3D, 0x1F),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x16, // Drip
        art_path: "artcomp/Enemy16_Drip.bin",
        palette_path: "palette_enemy/Drips_1.bin",
        extra_palette_paths: &["palette_enemy/Drips_2.bin", "palette_enemy/Drips_3.bin"],
        frame_duration: 8,
        frames: &[
            fd(0x000, 0x0C, 0x00, 0x0F, 0x0E),
            fd(0x004, 0x0A, 0x00, 0x0E, 0x0F),
            fd(0x008, 0x08, 0x00, 0x0F, 0x0F),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x17, // Hand
        art_path: "artcomp/Enemy17_Hand.bin",
        palette_path: "palette_enemy/Hand_1.bin",
        extra_palette_paths: &["palette_enemy/Hand_2.bin", "palette_enemy/Hand_3.bin"],
        frame_duration: 1,
        frames: &[fd(0x03F, 0x10, 0x01, 0x20, 0x12)],
    },
    EnemyFamilySpec {
        object_type: 0x18, // Tornado
        art_path: "artcomp/Enemy18_Tornado.bin",
        palette_path: "palette_enemy/Tornado_1.bin",
        extra_palette_paths: &["palette_enemy/Tornado_2.bin", "palette_enemy/Tornado_3.bin"],
        frame_duration: 5,
        frames: &[
            fd(0x028, 0x0E, 0x00, 0x1D, 0x28),
            fd(0x03C, 0x0D, 0x00, 0x1C, 0x27),
            fd(0x050, 0x0D, 0x00, 0x1C, 0x28),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x19, // Fireball
        art_path: "artcomp/Enemy19_Fireball.bin",
        palette_path: "palette_enemy/Fireball_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Fireball_2.bin",
            "palette_enemy/Fireball_3.bin",
        ],
        frame_duration: 6,
        frames: &[
            fd(0x064, 0x07, 0x00, 0x0E, 0x08),
            fd(0x066, 0x07, 0x00, 0x0D, 0x07),
            fd(0x068, 0x06, 0x00, 0x0C, 0x06),
            fd(0x06A, 0x07, 0x00, 0x0E, 0x08),
            fd(0x06C, 0x07, 0x00, 0x0D, 0x08),
            fd(0x06E, 0x06, 0x00, 0x0C, 0x06),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x1A, // Driller
        art_path: "artcomp/Enemy1A_Driller.bin",
        palette_path: "palette_enemy/Drill_1.bin",
        extra_palette_paths: &["palette_enemy/Drill_2.bin", "palette_enemy/Drill_3.bin"],
        frame_duration: 9,
        frames: &[
            fd(0x028, 0x08, 0x00, 0x10, 0x0F),
            fd(0x02C, 0x08, 0x00, 0x10, 0x0E),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x1B, // Emo Rock
        art_path: "artcomp/Enemy1B_Emo_rock.bin",
        palette_path: "palette_enemy/Emo_Rock_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Emo_Rock_2.bin",
            "palette_enemy/Emo_Rock_3.bin",
        ],
        frame_duration: 20,
        frames: &[
            fd(0x010, 0x10, 0x00, 0x1D, 0x20),
            fd(0x040, 0x10, 0x00, 0x1F, 0x1C),
            fd(0x050, 0x10, 0x00, 0x1F, 0x20),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x1C, // Mini Hopping Skull
        art_path: "artcomp/Enemy1C_Mini_hopping_skull.bin",
        palette_path: "palette_enemy/Mini_Hopping_Skull_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Mini_Hopping_Skull_2.bin",
            "palette_enemy/Mini_Hopping_Skull_3.bin",
        ],
        frame_duration: 9,
        frames: &[
            fd(0x02C, 0x07, 0x00, 0x0F, 0x11),
            fd(0x032, 0x08, 0x00, 0x0F, 0x12),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x1D, // Big Hopping Skull
        art_path: "artcomp/Enemy1D_Big_hopping_skull.bin",
        palette_path: "palette_enemy/Big_Hopping_Skull_1.bin",
        extra_palette_paths: &[
            "palette_enemy/Big_Hopping_Skull_2.bin",
            "palette_enemy/Big_Hopping_Skull_3.bin",
        ],
        frame_duration: 80,
        frames: &[fd(0x040, 0x0D, 0x00, 0x19, 0x20)],
    },
    EnemyFamilySpec {
        object_type: 0x20, // Heady Metal (boss)
        art_path: "artcomp/Enemy20_Heady_Metal.bin",
        palette_path: "palette_enemy/Boss.bin",
        extra_palette_paths: &[],
        frame_duration: 10,
        frames: &[fd(0x040, 0x0D, 0x00, 0x19, 0x20)],
    },
    EnemyFamilySpec {
        object_type: 0x21, // Boss Eyes / Attacks
        art_path: "artcomp/Enemy21_Boss_eyes_or_attacks.bin",
        palette_path: "palette_enemy/Boss_Eyes.bin",
        extra_palette_paths: &[],
        frame_duration: 6,
        frames: &[
            fd(0x000, 0x08, 0x08, 0x10, 0x10),
            fd(0x004, 0x08, 0x08, 0x10, 0x10),
            fd(0x008, 0x08, 0x08, 0x10, 0x10),
            fd(0x00C, 0x08, 0x08, 0x10, 0x10),
            fd(0x010, 0x08, 0x08, 0x10, 0x10),
            fd(0x014, 0x08, 0x08, 0x10, 0x10),
            fd(0x018, 0x08, 0x08, 0x10, 0x10),
            fd(0x01C, 0x08, 0x08, 0x10, 0x10),
        ],
    },
    EnemyFamilySpec {
        object_type: 0x22, // Shiskaboss (head)
        art_path: "artcomp/Enemy22_Boss_head_stick_boomerangs_arrows.bin",
        palette_path: "palette_enemy/Boss.bin",
        extra_palette_paths: &[],
        frame_duration: 12,
        frames: &[
            fd(0x0A4, 0x20, 0x00, 0x40, 0x40),
            fd(0x0E4, 0x20, 0x00, 0x3B, 0x40),
            fd(0x124, 0x20, 0x00, 0x3C, 0x40),
            fd(0x164, 0x23, 0x00, 0x3F, 0x40),
            fd(0x1A4, 0x20, 0x0F, 0x3C, 0x4F),
        ],
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
    let mut families = std::collections::HashMap::new();
    for spec in ENEMY_FAMILIES {
        let palette = match load_ingame_palette_asset(manifest, spec.palette_path, spec.art_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("skipping enemy 0x{:02X} sprite: {e}", spec.object_type);
                continue;
            }
        };
        let frames = match load_compressed_sprite_frames(
            manifest,
            spec.art_path,
            &format!("enemy 0x{:02X}", spec.object_type),
            spec.frames,
        ) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("skipping enemy 0x{:02X} art: {e}", spec.object_type);
                continue;
            }
        };
        let extra_palettes = spec
            .extra_palette_paths
            .iter()
            .filter_map(|path| load_ingame_palette_asset(manifest, path, spec.art_path).ok())
            .collect();
        families.insert(
            spec.object_type,
            AnimatedSpriteVisual {
                palette,
                extra_palettes,
                frames,
                frame_duration: spec.frame_duration,
            },
        );
    }
    Ok(EnemyVisuals { families })
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
