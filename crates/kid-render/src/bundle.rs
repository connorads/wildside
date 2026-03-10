use crate::hud::{HudIcon, HudVisuals};
use crate::level::{BlockVisualMapping, LevelVisuals};
use crate::palette::{Palette, PaletteSet};
use crate::tiles::{TileRef, TileSheet, parse_tile_quads};
use anyhow::{Context, Result, anyhow, bail};
use kid_data::{
    AssetFile, AssetManifest, DecodedAssetFile, decode_saxman, ensure_generated_bundle_contract,
};
use sha1::{Digest, Sha1};
use std::fs;
use std::path::Path;

const LEVEL1_ID: u8 = 0x00;
const HUD_DIGIT_BASE_TILE_INDEX: u16 = 0x06BA;
const BLOCKS_ART_PATH: &str = "artcomp/Blocks.bin";
const HUD_NUMBERS_ART_PATH: &str = "artcomp/HUD_numbers.bin";
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
    pub level: LevelVisuals,
    pub hud: HudVisuals,
}

pub fn load_level1_visuals(manifest: &AssetManifest) -> Result<Level1Visuals> {
    ensure_generated_bundle_contract(manifest)?;
    let level = manifest
        .levels
        .iter()
        .find(|level| level.level_id == LEVEL1_ID)
        .ok_or_else(|| anyhow!("generated bundle does not contain Level 1 metadata"))?;
    let header = level
        .header_metadata
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

    let foreground_layout = read_decoded_file(manifest, &decoded.foreground, "foreground layout")?;
    let block_layout = read_decoded_file(manifest, &decoded.block, "block layout")?;
    let background_layout = read_decoded_file(manifest, &decoded.background, "background layout")?;

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

    Ok(Level1Visuals {
        foreground_tiles,
        background_tiles,
        block_tiles,
        palettes,
        level: LevelVisuals {
            level_id: LEVEL1_ID,
            foreground_theme_id: header.level_theme,
            background_theme_id: header.background_theme,
            foreground_layout_bytes: foreground_layout,
            block_layout_bytes: block_layout,
            background_layout_bytes: background_layout,
            block_mapping: BlockVisualMapping::new(terrain_quads, block_quads)
                .context("failed to build Level 1 block visual mapping")?,
        },
        hud: HudVisuals {
            digits: hud_digits,
            digit_base_tile_index: HUD_DIGIT_BASE_TILE_INDEX,
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
