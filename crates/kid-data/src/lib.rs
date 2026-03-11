use anyhow::{Context, Result, anyhow, bail};
use image::{GrayImage, Luma};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const COPIED_DIRECTORIES: &[&str] = &[
    "code/enemy",
    "doc",
    "ingame/anim",
    "ingame/artcomp",
    "ingame/artunc",
    "ingame/artunc_kid",
    "ingame/palette_enemy",
    "ingame/palette_kid",
    "level",
    "scenes",
    "sound",
    "theme",
];
const COPIED_FILES: &[&str] = &["constants.asm", "variables.asm"];
pub const GENERATED_BUNDLE_VERSION: u32 = 5;
pub const LEVEL1_DECODED_ASSET_VERSION: u32 = 2;
const LEVEL1_ID: u8 = 0x00;
const LEVEL1_DECODED_OUTPUT_DIR: &str = "decoded/level/00";
const BACKGROUND_PREVIEW_WIDTH_TILES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManifest {
    #[serde(default)]
    pub generated_bundle_version: u32,
    pub rom_sha1: String,
    pub upstream_commit: String,
    pub source_root: PathBuf,
    pub output_root: PathBuf,
    pub levels: Vec<LevelRecord>,
    pub constants: Vec<NamedValue>,
    pub ram_variables: Vec<RamVariable>,
    pub enemy_sources: Vec<AssetFile>,
    pub theme_assets: Vec<AssetFile>,
    pub scene_assets: Vec<AssetFile>,
    pub animation_assets: Vec<AssetFile>,
    pub sound_assets: Vec<AssetFile>,
    #[serde(default)]
    pub ingame_assets: Vec<AssetFile>,
    #[serde(default)]
    pub decoded_level1: Option<DecodedLevelAssets>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedLevelAssets {
    pub contract_version: u32,
    pub level_id: u8,
    pub foreground: DecodedAssetFile,
    pub block: DecodedAssetFile,
    pub background: DecodedAssetFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedAssetFile {
    pub relative_path: String,
    pub preview_png_path: String,
    pub size_bytes: u64,
    pub sha1: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelRecord {
    pub level_id: u8,
    pub title_text_id: Option<u8>,
    pub display_name: Option<String>,
    pub resource_paths: Vec<String>,
    #[serde(default)]
    pub resources: LevelResources,
    #[serde(default)]
    pub header_metadata: Option<LevelHeader>,
    #[serde(default)]
    pub enemy_layout_metadata: Option<EnemyLayout>,
    #[serde(default)]
    pub bgscroll_metadata: Option<BgscrollProfile>,
    #[serde(default)]
    pub platform_metadata: Option<PlatformLayout>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelResources {
    pub platform_script: Option<String>,
    pub bgscroll: Option<String>,
    pub header: Option<String>,
    pub enemy: Option<String>,
    pub foreground: Option<String>,
    pub block: Option<String>,
    pub background: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelAssetBundle {
    pub level_id: u8,
    pub display_name: Option<String>,
    pub platform_script: Option<LoadedAsset>,
    pub parsed_platform_layout: Option<PlatformLayout>,
    pub bgscroll: Option<LoadedAsset>,
    pub parsed_bgscroll: Option<BgscrollProfile>,
    pub header: Option<LoadedAsset>,
    pub parsed_header: Option<LevelHeader>,
    pub enemy: Option<LoadedAsset>,
    pub parsed_enemy_layout: Option<EnemyLayout>,
    pub foreground: Option<LoadedAsset>,
    pub block: Option<LoadedAsset>,
    pub background: Option<LoadedAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedAsset {
    pub relative_path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyDefinition {
    pub id: u8,
    pub constant_name: String,
    pub symbolic_name: String,
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelHeader {
    pub width_screens: u8,
    pub height_screens: u8,
    pub height_flag_bits: u8,
    pub level_theme: u8,
    pub murder_wall_flags: u8,
    pub background_theme: u8,
    pub weather_flags: u8,
    pub player_start_x: u16,
    pub player_start_y: u16,
    pub flag_x: u16,
    pub flag_y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyLayout {
    pub unknown_prefix: u32,
    pub enemy_type_slots: Vec<u16>,
    pub unknown_word: u16,
    pub header_byte: u8,
    pub object_count: u8,
    pub objects: Vec<EnemyLayoutObject>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyLayoutObject {
    pub object_type: u8,
    pub flags: u8,
    pub hit_points_plus_one: u16,
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BgscrollProfile {
    pub layer_speeds: Vec<u8>,
    pub has_terminator: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformLayout {
    pub placements: Vec<PlatformPlacement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformPlacement {
    pub x: u16,
    pub y: u16,
    pub buffer_left: u8,
    pub buffer_right: u8,
    pub buffer_top: u8,
    pub buffer_bottom: u8,
    pub kind: u8,
    pub subtype: u8,
    pub h_variant: u8,
    pub v_variant: u8,
    pub script: PlatformScriptRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformScriptRef {
    pub raw: String,
    pub relative_offset: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedValue {
    pub name: String,
    pub value: i64,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RamVariable {
    pub name: String,
    pub address: u32,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetFile {
    pub relative_path: String,
    pub size_bytes: u64,
    pub sha1: String,
}

pub fn generated_bundle_dir(repo_root: &Path, rom_sha1: &str) -> PathBuf {
    repo_root.join(".cache/generated").join(rom_sha1)
}

pub fn manifest_path(bundle_dir: &Path) -> PathBuf {
    bundle_dir.join("asset-manifest.json")
}

pub fn export_split_bundle(
    source_root: &Path,
    output_root: &Path,
    rom_sha1: &str,
    upstream_commit: &str,
) -> Result<AssetManifest> {
    if !source_root.is_dir() {
        bail!(
            "split source root does not exist: {}",
            source_root.display()
        );
    }

    let staging_root = create_bundle_staging_dir(output_root)?;
    let mut manifest =
        export_split_bundle_into(source_root, &staging_root, rom_sha1, upstream_commit)?;
    replace_bundle_from_staging(&staging_root, output_root)?;
    manifest.output_root = output_root.to_path_buf();
    fs::write(
        manifest_path(output_root),
        serde_json::to_vec_pretty(&manifest)?,
    )
    .with_context(|| format!("failed to write {}", manifest_path(output_root).display()))?;
    Ok(manifest)
}

fn export_split_bundle_into(
    source_root: &Path,
    output_root: &Path,
    rom_sha1: &str,
    upstream_commit: &str,
) -> Result<AssetManifest> {
    for relative in COPIED_DIRECTORIES {
        copy_directory_recursive(&source_root.join(relative), &output_root.join(relative))?;
    }
    for relative in COPIED_FILES {
        copy_file(&source_root.join(relative), &output_root.join(relative))?;
    }

    let level_names = parse_level_names(
        &fs::read_to_string(output_root.join("doc/level_list.txt")).with_context(|| {
            format!(
                "failed to read {}",
                output_root.join("doc/level_list.txt").display()
            )
        })?,
    )?;
    let level_files = parse_level_files(
        &fs::read_to_string(output_root.join("level/level_files.txt")).with_context(|| {
            format!(
                "failed to read {}",
                output_root.join("level/level_files.txt").display()
            )
        })?,
    )?;
    let constants = parse_named_values(
        &fs::read_to_string(output_root.join("constants.asm")).with_context(|| {
            format!(
                "failed to read {}",
                output_root.join("constants.asm").display()
            )
        })?,
    )?;
    let ram_variables = parse_ram_variables(
        &fs::read_to_string(output_root.join("variables.asm")).with_context(|| {
            format!(
                "failed to read {}",
                output_root.join("variables.asm").display()
            )
        })?,
    )?;

    let manifest = AssetManifest {
        generated_bundle_version: GENERATED_BUNDLE_VERSION,
        rom_sha1: rom_sha1.to_owned(),
        upstream_commit: upstream_commit.to_owned(),
        source_root: source_root.to_path_buf(),
        output_root: output_root.to_path_buf(),
        levels: merge_level_catalogues(level_names, level_files),
        constants,
        ram_variables,
        enemy_sources: collect_asset_files(&output_root.join("code/enemy"))?,
        theme_assets: collect_asset_files(&output_root.join("theme"))?,
        scene_assets: collect_asset_files(&output_root.join("scenes"))?,
        animation_assets: collect_asset_files(&output_root.join("ingame/anim"))?,
        sound_assets: collect_asset_files(&output_root.join("sound"))?,
        ingame_assets: collect_asset_files(&output_root.join("ingame"))?,
        decoded_level1: None,
    };
    let mut manifest = manifest;
    normalise_manifest(&mut manifest)?;
    manifest.decoded_level1 = Some(export_decoded_level1_assets(&manifest)?);
    ensure_generated_bundle_contract(&manifest)?;

    fs::write(
        manifest_path(output_root),
        serde_json::to_vec_pretty(&manifest)?,
    )
    .with_context(|| format!("failed to write {}", manifest_path(output_root).display()))?;

    Ok(manifest)
}

fn create_bundle_staging_dir(output_root: &Path) -> Result<PathBuf> {
    let parent = output_root.parent().ok_or_else(|| {
        anyhow!(
            "generated bundle path has no parent directory: {}",
            output_root.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let staging_root = unique_bundle_temp_path(output_root, "staging");
    fs::create_dir_all(&staging_root)
        .with_context(|| format!("failed to create {}", staging_root.display()))?;
    Ok(staging_root)
}

fn replace_bundle_from_staging(staging_root: &Path, output_root: &Path) -> Result<()> {
    let backup_root = unique_bundle_temp_path(output_root, "backup");
    let had_existing_bundle = output_root.exists();
    if had_existing_bundle {
        fs::rename(output_root, &backup_root).with_context(|| {
            format!(
                "failed to move existing generated bundle {} to {}",
                output_root.display(),
                backup_root.display()
            )
        })?;
    }

    match fs::rename(staging_root, output_root) {
        Ok(()) => {
            if had_existing_bundle {
                fs::remove_dir_all(&backup_root).with_context(|| {
                    format!(
                        "failed to remove old generated bundle {}",
                        backup_root.display()
                    )
                })?;
            }
            Ok(())
        }
        Err(error) => {
            if had_existing_bundle {
                let restore_result = fs::rename(&backup_root, output_root);
                if let Err(restore_error) = restore_result {
                    return Err(restore_error).with_context(|| {
                        format!(
                            "failed to restore generated bundle {} after swap failure: {}",
                            output_root.display(),
                            error
                        )
                    });
                }
            }
            Err(error).with_context(|| {
                format!(
                    "failed to move generated bundle staging dir {} into place at {}",
                    staging_root.display(),
                    output_root.display()
                )
            })
        }
    }
}

fn unique_bundle_temp_path(output_root: &Path, suffix: &str) -> PathBuf {
    let parent = output_root
        .parent()
        .expect("generated bundle path should have a parent");
    let base_name = output_root
        .file_name()
        .expect("generated bundle path should have a filename")
        .to_string_lossy();
    for attempt in 0..10_000u32 {
        let candidate = parent.join(format!(
            ".{base_name}.{suffix}.{}.{}",
            process::id(),
            attempt
        ));
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!(".{base_name}.{suffix}.{}", process::id()))
}

pub fn load_manifest(path: &Path) -> Result<AssetManifest> {
    let bytes =
        fs::read(path).with_context(|| format!("failed to read manifest {}", path.display()))?;
    let mut manifest: AssetManifest =
        serde_json::from_slice(&bytes).context("failed to parse asset manifest")?;
    normalise_manifest(&mut manifest)?;
    Ok(manifest)
}

pub fn ensure_generated_bundle_contract(manifest: &AssetManifest) -> Result<()> {
    if manifest.generated_bundle_version != GENERATED_BUNDLE_VERSION {
        bail!(
            "generated bundle version mismatch: expected {}, got {}",
            GENERATED_BUNDLE_VERSION,
            manifest.generated_bundle_version
        );
    }

    let decoded = manifest
        .decoded_level1
        .as_ref()
        .ok_or_else(|| anyhow!("generated bundle is missing decoded Level 1 assets"))?;
    if decoded.contract_version != LEVEL1_DECODED_ASSET_VERSION {
        bail!(
            "decoded Level 1 asset version mismatch: expected {}, got {}",
            LEVEL1_DECODED_ASSET_VERSION,
            decoded.contract_version
        );
    }
    if decoded.level_id != LEVEL1_ID {
        bail!(
            "decoded Level 1 contract points at level 0x{:02X}, expected 0x{:02X}",
            decoded.level_id,
            LEVEL1_ID
        );
    }

    validate_decoded_asset_file(manifest, &decoded.foreground, "foreground")?;
    validate_decoded_asset_file(manifest, &decoded.block, "block")?;
    validate_decoded_asset_file(manifest, &decoded.background, "background")?;
    Ok(())
}

pub fn load_level_asset_bundle(manifest: &AssetManifest, level_id: u8) -> Result<LevelAssetBundle> {
    let level = manifest
        .levels
        .iter()
        .find(|level| level.level_id == level_id)
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} not found in asset manifest"))?
        .clone();
    let header = load_asset(manifest, level.resources.header.as_deref())?;
    let parsed_header = header
        .as_ref()
        .map(|asset| parse_level_header(&asset.bytes))
        .transpose()?
        .or(level.header_metadata.clone());
    let enemy = load_asset(manifest, level.resources.enemy.as_deref())?;
    let parsed_enemy_layout = enemy
        .as_ref()
        .map(|asset| parse_enemy_layout(&asset.bytes))
        .transpose()?
        .or(level.enemy_layout_metadata.clone());
    let bgscroll = load_asset(manifest, level.resources.bgscroll.as_deref())?;
    let parsed_bgscroll = bgscroll
        .as_ref()
        .map(|asset| parse_bgscroll_profile(&asset.bytes))
        .transpose()?
        .or(level.bgscroll_metadata.clone());
    let platform_script = load_asset(manifest, level.resources.platform_script.as_deref())?;
    let parsed_platform_layout = platform_script
        .as_ref()
        .map(|asset| parse_platform_layout(std::str::from_utf8(&asset.bytes)?))
        .transpose()?
        .or(level.platform_metadata.clone());
    Ok(LevelAssetBundle {
        level_id,
        display_name: level.display_name.clone(),
        platform_script,
        parsed_platform_layout,
        bgscroll,
        parsed_bgscroll,
        header,
        parsed_header,
        enemy,
        parsed_enemy_layout,
        foreground: load_asset(manifest, level.resources.foreground.as_deref())?,
        block: load_asset(manifest, level.resources.block.as_deref())?,
        background: load_asset(manifest, level.resources.background.as_deref())?,
    })
}

pub fn enemy_definitions(manifest: &AssetManifest) -> Vec<EnemyDefinition> {
    let mut definitions = manifest
        .constants
        .iter()
        .filter_map(|value| {
            let symbolic_name = value.name.strip_prefix("objid_")?;
            let id = u8::try_from(value.value).ok()?;
            Some(EnemyDefinition {
                id,
                constant_name: value.name.clone(),
                symbolic_name: symbolic_name.to_owned(),
                source_path: find_enemy_source_path(manifest, symbolic_name),
            })
        })
        .collect::<Vec<_>>();
    definitions.sort_by_key(|definition| definition.id);
    definitions
}

pub fn find_enemy_definition(manifest: &AssetManifest, selector: &str) -> Option<EnemyDefinition> {
    let selector = selector.trim();
    let by_id = selector
        .strip_prefix("0x")
        .or_else(|| selector.strip_prefix("0X"))
        .or(Some(selector))
        .and_then(|hex| u8::from_str_radix(hex, 16).ok());
    let normalised_selector = normalise_enemy_name(selector);
    enemy_definitions(manifest).into_iter().find(|definition| {
        Some(definition.id) == by_id
            || normalise_enemy_name(&definition.constant_name) == normalised_selector
            || normalise_enemy_name(&definition.symbolic_name) == normalised_selector
    })
}

pub fn parse_level_header(bytes: &[u8]) -> Result<LevelHeader> {
    if bytes.len() < 0x0C {
        bail!(
            "level header was {} bytes, expected at least 12 bytes",
            bytes.len()
        );
    }

    let width_screens = bytes[0];
    let raw_height = bytes[1];
    let raw_level_theme = bytes[2];
    let raw_background_theme = bytes[3];

    Ok(LevelHeader {
        width_screens,
        height_screens: raw_height & 0x3F,
        height_flag_bits: raw_height >> 6,
        level_theme: raw_level_theme & 0x3F,
        murder_wall_flags: raw_level_theme >> 6,
        background_theme: raw_background_theme & 0x0F,
        weather_flags: raw_background_theme >> 4,
        player_start_x: read_u16_be(bytes, 4),
        player_start_y: read_u16_be(bytes, 6),
        flag_x: read_u16_be(bytes, 8),
        flag_y: read_u16_be(bytes, 10),
    })
}

pub fn parse_enemy_layout(bytes: &[u8]) -> Result<EnemyLayout> {
    const HEADER_LEN: usize = 14;
    const OBJECT_LEN: usize = 8;

    if bytes.len() < HEADER_LEN {
        bail!(
            "enemy layout was {} bytes, expected at least {HEADER_LEN} bytes",
            bytes.len()
        );
    }

    let object_count = bytes[13];
    let expected_len = HEADER_LEN + usize::from(object_count) * OBJECT_LEN;
    if bytes.len() < expected_len {
        bail!(
            "enemy layout was {} bytes, expected at least {expected_len} bytes for {object_count} objects",
            bytes.len()
        );
    }

    let mut objects = Vec::with_capacity(usize::from(object_count));
    for chunk in bytes[HEADER_LEN..expected_len].chunks_exact(OBJECT_LEN) {
        objects.push(EnemyLayoutObject {
            object_type: chunk[0],
            flags: chunk[1],
            hit_points_plus_one: read_u16_be(chunk, 2),
            x: read_u16_be(chunk, 4),
            y: read_u16_be(chunk, 6),
        });
    }

    Ok(EnemyLayout {
        unknown_prefix: read_u32_be(bytes, 0),
        enemy_type_slots: vec![
            read_u16_be(bytes, 4),
            read_u16_be(bytes, 6),
            read_u16_be(bytes, 8),
        ],
        unknown_word: read_u16_be(bytes, 10),
        header_byte: bytes[12],
        object_count,
        objects,
    })
}

pub fn parse_bgscroll_profile(bytes: &[u8]) -> Result<BgscrollProfile> {
    match bytes.iter().position(|byte| *byte == 0xFF) {
        Some(terminator_index) => Ok(BgscrollProfile {
            layer_speeds: bytes[..terminator_index].to_vec(),
            has_terminator: true,
        }),
        None => Ok(BgscrollProfile {
            layer_speeds: bytes.to_vec(),
            has_terminator: false,
        }),
    }
}

pub fn parse_platform_layout(contents: &str) -> Result<PlatformLayout> {
    let mut placements = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with("dc.w") {
            continue;
        }
        let Some(arguments) = trimmed.strip_prefix("ptfm") else {
            continue;
        };
        let fields = arguments
            .trim()
            .split(',')
            .map(str::trim)
            .collect::<Vec<_>>();
        if fields.len() != 11 {
            bail!(
                "platform layout line {} had {} ptfm fields, expected 11",
                index + 1,
                fields.len()
            );
        }
        placements.push(PlatformPlacement {
            x: fields[0]
                .parse()
                .with_context(|| format!("invalid platform x on line {}", index + 1))?,
            y: fields[1]
                .parse()
                .with_context(|| format!("invalid platform y on line {}", index + 1))?,
            buffer_left: fields[2]
                .parse()
                .with_context(|| format!("invalid platform left buffer on line {}", index + 1))?,
            buffer_right: fields[3]
                .parse()
                .with_context(|| format!("invalid platform right buffer on line {}", index + 1))?,
            buffer_top: fields[4]
                .parse()
                .with_context(|| format!("invalid platform top buffer on line {}", index + 1))?,
            buffer_bottom: fields[5]
                .parse()
                .with_context(|| format!("invalid platform bottom buffer on line {}", index + 1))?,
            kind: fields[6]
                .parse()
                .with_context(|| format!("invalid platform kind on line {}", index + 1))?,
            subtype: fields[7]
                .parse()
                .with_context(|| format!("invalid platform subtype on line {}", index + 1))?,
            h_variant: fields[8]
                .parse()
                .with_context(|| format!("invalid platform h variant on line {}", index + 1))?,
            v_variant: fields[9]
                .parse()
                .with_context(|| format!("invalid platform v variant on line {}", index + 1))?,
            script: parse_platform_script_ref(fields[10]),
        });
    }
    Ok(PlatformLayout { placements })
}

fn parse_platform_script_ref(value: &str) -> PlatformScriptRef {
    let relative_offset = value
        .strip_prefix("PlatformScript_")
        .and_then(|rest| rest.strip_suffix("-PlatformScript_BaseAddress"))
        .and_then(|hex| u16::from_str_radix(hex, 16).ok());
    PlatformScriptRef {
        raw: value.to_owned(),
        relative_offset,
    }
}

fn merge_level_catalogues(
    names: BTreeMap<u8, (u8, String)>,
    files: BTreeMap<u8, Vec<String>>,
) -> Vec<LevelRecord> {
    let mut by_level = BTreeMap::new();
    for (level_id, (title_text_id, display_name)) in names {
        by_level.insert(
            level_id,
            LevelRecord {
                level_id,
                title_text_id: Some(title_text_id),
                display_name: Some(display_name),
                resource_paths: files.get(&level_id).cloned().unwrap_or_default(),
                resources: level_resources_from_paths(
                    files.get(&level_id).cloned().unwrap_or_default().as_slice(),
                ),
                header_metadata: None,
                enemy_layout_metadata: None,
                bgscroll_metadata: None,
                platform_metadata: None,
            },
        );
    }
    for (level_id, resource_paths) in files {
        by_level.entry(level_id).or_insert(LevelRecord {
            level_id,
            title_text_id: None,
            display_name: None,
            resources: level_resources_from_paths(&resource_paths),
            resource_paths,
            header_metadata: None,
            enemy_layout_metadata: None,
            bgscroll_metadata: None,
            platform_metadata: None,
        });
    }
    by_level.into_values().collect()
}

fn normalise_manifest(manifest: &mut AssetManifest) -> Result<()> {
    for level in &mut manifest.levels {
        if level.resources == LevelResources::default() {
            level.resources = level_resources_from_paths(&level.resource_paths);
        }
        if level.header_metadata.is_none()
            && let Some(relative_path) = level.resources.header.as_deref()
        {
            let full_path = manifest.output_root.join("level").join(relative_path);
            if full_path.is_file() {
                let bytes = fs::read(&full_path).with_context(|| {
                    format!("failed to read level header {}", full_path.display())
                })?;
                level.header_metadata = Some(parse_level_header(&bytes)?);
            }
        }
        if level.enemy_layout_metadata.is_none()
            && let Some(relative_path) = level.resources.enemy.as_deref()
        {
            let full_path = manifest.output_root.join("level").join(relative_path);
            if full_path.is_file() {
                let bytes = fs::read(&full_path).with_context(|| {
                    format!("failed to read enemy layout {}", full_path.display())
                })?;
                level.enemy_layout_metadata = Some(parse_enemy_layout(&bytes)?);
            }
        }
        if level.bgscroll_metadata.is_none()
            && let Some(relative_path) = level.resources.bgscroll.as_deref()
        {
            let full_path = manifest.output_root.join("level").join(relative_path);
            if full_path.is_file() {
                let bytes = fs::read(&full_path)
                    .with_context(|| format!("failed to read bgscroll {}", full_path.display()))?;
                level.bgscroll_metadata = Some(parse_bgscroll_profile(&bytes)?);
            }
        }
        if level.platform_metadata.is_none()
            && let Some(relative_path) = level.resources.platform_script.as_deref()
        {
            let full_path = manifest.output_root.join("level").join(relative_path);
            if full_path.is_file() {
                let contents = fs::read_to_string(&full_path).with_context(|| {
                    format!("failed to read platform layout {}", full_path.display())
                })?;
                level.platform_metadata = Some(parse_platform_layout(&contents)?);
            }
        }
    }
    Ok(())
}

fn level_resources_from_paths(paths: &[String]) -> LevelResources {
    let mut resources = LevelResources::default();
    for path in paths {
        if path.starts_with("platform/") {
            resources.platform_script = Some(path.clone());
        } else if path.starts_with("bgscroll/") {
            resources.bgscroll = Some(path.clone());
        } else if path.starts_with("header/") {
            resources.header = Some(path.clone());
        } else if path.starts_with("enemy/") {
            resources.enemy = Some(path.clone());
        } else if path.starts_with("foreground/") {
            resources.foreground = Some(path.clone());
        } else if path.starts_with("block/") {
            resources.block = Some(path.clone());
        } else if path.starts_with("background/") {
            resources.background = Some(path.clone());
        }
    }
    resources
}

fn load_asset(
    manifest: &AssetManifest,
    relative_path: Option<&str>,
) -> Result<Option<LoadedAsset>> {
    let Some(relative_path) = relative_path else {
        return Ok(None);
    };
    let full_path = manifest.output_root.join("level").join(relative_path);
    let bytes = fs::read(&full_path)
        .with_context(|| format!("failed to read level asset {}", full_path.display()))?;
    Ok(Some(LoadedAsset {
        relative_path: relative_path.to_owned(),
        bytes,
    }))
}

fn export_decoded_level1_assets(manifest: &AssetManifest) -> Result<DecodedLevelAssets> {
    let level = manifest
        .levels
        .iter()
        .find(|level| level.level_id == LEVEL1_ID)
        .ok_or_else(|| anyhow!("level 0x{LEVEL1_ID:02X} not found in asset manifest"))?;
    let header = level
        .header_metadata
        .as_ref()
        .ok_or_else(|| anyhow!("level 0x{LEVEL1_ID:02X} is missing header metadata"))?;
    let preview_width = usize::from(header.width_screens) * 40;
    let output_dir = manifest.output_root.join(LEVEL1_DECODED_OUTPUT_DIR);
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    let foreground = export_saxman_decoded_asset(
        manifest,
        level.resources.foreground.as_deref(),
        "foreground",
        preview_width,
    )?;
    let block = export_raw_level_asset(
        manifest,
        level.resources.block.as_deref(),
        "block",
        preview_width,
    )?;
    let background = export_raw_level_asset(
        manifest,
        level.resources.background.as_deref(),
        "background",
        BACKGROUND_PREVIEW_WIDTH_TILES,
    )?;

    Ok(DecodedLevelAssets {
        contract_version: LEVEL1_DECODED_ASSET_VERSION,
        level_id: LEVEL1_ID,
        foreground,
        block,
        background,
    })
}

fn export_saxman_decoded_asset(
    manifest: &AssetManifest,
    relative_path: Option<&str>,
    stem: &str,
    preview_width: usize,
) -> Result<DecodedAssetFile> {
    let relative_path = relative_path
        .ok_or_else(|| anyhow!("level 0x{LEVEL1_ID:02X} is missing {stem} asset path"))?;
    let encoded_path = manifest.output_root.join("level").join(relative_path);
    let decoded_bytes = decode_saxman(
        &fs::read(&encoded_path)
            .with_context(|| format!("failed to read {}", encoded_path.display()))?,
    )?;
    export_decoded_asset_file(manifest, stem, &decoded_bytes, preview_width, false)
}

fn export_raw_level_asset(
    manifest: &AssetManifest,
    relative_path: Option<&str>,
    stem: &str,
    preview_width: usize,
) -> Result<DecodedAssetFile> {
    let relative_path = relative_path
        .ok_or_else(|| anyhow!("level 0x{LEVEL1_ID:02X} is missing {stem} asset path"))?;
    let encoded_path = manifest.output_root.join("level").join(relative_path);
    let raw_bytes = fs::read(&encoded_path)
        .with_context(|| format!("failed to read {}", encoded_path.display()))?;
    export_decoded_asset_file(manifest, stem, &raw_bytes, preview_width, false)
}

fn export_decoded_asset_file(
    manifest: &AssetManifest,
    stem: &str,
    decoded_bytes: &[u8],
    preview_width: usize,
    word_preview: bool,
) -> Result<DecodedAssetFile> {
    let raw_path = manifest
        .output_root
        .join(LEVEL1_DECODED_OUTPUT_DIR)
        .join(format!("{stem}.bin"));
    let preview_path = manifest
        .output_root
        .join(LEVEL1_DECODED_OUTPUT_DIR)
        .join(format!("{stem}.png"));
    fs::write(&raw_path, decoded_bytes)
        .with_context(|| format!("failed to write {}", raw_path.display()))?;
    if word_preview {
        write_word_preview_png(&preview_path, decoded_bytes, preview_width)?;
    } else {
        write_byte_preview_png(&preview_path, decoded_bytes, preview_width)?;
    }

    Ok(DecodedAssetFile {
        relative_path: normalise_relative_path(
            raw_path
                .strip_prefix(&manifest.output_root)
                .expect("decoded asset should stay inside output root"),
        ),
        preview_png_path: normalise_relative_path(
            preview_path
                .strip_prefix(&manifest.output_root)
                .expect("decoded preview should stay inside output root"),
        ),
        size_bytes: decoded_bytes.len() as u64,
        sha1: sha1_hex(decoded_bytes),
    })
}

fn validate_decoded_asset_file(
    manifest: &AssetManifest,
    asset: &DecodedAssetFile,
    label: &str,
) -> Result<()> {
    let asset_path = manifest.output_root.join(&asset.relative_path);
    let preview_path = manifest.output_root.join(&asset.preview_png_path);
    if !asset_path.is_file() {
        bail!(
            "decoded Level 1 {label} asset is missing: {}",
            asset_path.display()
        );
    }
    if !preview_path.is_file() {
        bail!(
            "decoded Level 1 {label} preview is missing: {}",
            preview_path.display()
        );
    }
    let bytes = fs::read(&asset_path)
        .with_context(|| format!("failed to read decoded asset {}", asset_path.display()))?;
    if bytes.len() as u64 != asset.size_bytes {
        bail!(
            "decoded Level 1 {label} size mismatch: expected {}, got {}",
            asset.size_bytes,
            bytes.len()
        );
    }
    let actual_sha1 = sha1_hex(&bytes);
    if actual_sha1 != asset.sha1 {
        bail!(
            "decoded Level 1 {label} sha1 mismatch: expected {}, got {}",
            asset.sha1,
            actual_sha1
        );
    }
    Ok(())
}

fn write_byte_preview_png(path: &Path, bytes: &[u8], width: usize) -> Result<()> {
    let width = width.max(1);
    let height = bytes.len().max(1).div_ceil(width);
    let mut image = GrayImage::new(width as u32, height as u32);
    for (index, byte) in bytes.iter().copied().enumerate() {
        let x = (index % width) as u32;
        let y = (index / width) as u32;
        image.put_pixel(x, y, Luma([byte]));
    }
    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_word_preview_png(path: &Path, bytes: &[u8], width_words: usize) -> Result<()> {
    let words = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    let width_words = width_words.max(1);
    let height = words.len().max(1).div_ceil(width_words);
    let mut image = GrayImage::new(width_words as u32, height as u32);
    for (index, value) in words.iter().copied().enumerate() {
        let x = (index % width_words) as u32;
        let y = (index / width_words) as u32;
        image.put_pixel(x, y, Luma([(value & 0x00FF) as u8]));
    }
    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn decode_saxman(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 4 {
        bail!("Saxman payload is too short");
    }
    let input_offset = usize::from(read_u16_be(data, 0)) + 2;
    if input_offset > data.len() {
        bail!(
            "Saxman input offset {} is past payload length {}",
            input_offset,
            data.len()
        );
    }

    let mut bits = BitReader::new(&data[2..input_offset]);
    let mut input_position = input_offset;
    let mut output = Vec::new();

    loop {
        if bits.read_bit()? == 1 {
            output.push(read_input_byte(data, &mut input_position)?);
            continue;
        }

        if bits.read_bit()? == 0 {
            let count = usize::from(bits.read_bit()?) + 2;
            let distance = usize::from(read_input_byte(data, &mut input_position)?);
            if distance == 0 {
                write_zero_fill(&mut output, count.saturating_sub(1));
            } else {
                copy_from_output(&mut output, distance, count)?;
            }
            continue;
        }

        let high_distance = usize::from(bits.read_bits(3)?);
        let count_code = bits.read_bits(2)?;
        let low_distance = usize::from(read_input_byte(data, &mut input_position)?);
        let count = if count_code == 0b11 {
            usize::from(read_input_byte(data, &mut input_position)?)
        } else {
            usize::from(count_code) + 3
        };
        if count_code == 0b11 && count == 0 {
            break;
        }
        if count_code == 0b11 && count < 6 {
            continue;
        }

        let distance = low_distance + (high_distance << 8);
        if distance == 0 {
            write_zero_fill(&mut output, count.saturating_sub(1));
        } else {
            copy_from_output(&mut output, distance, count)?;
        }
    }

    Ok(output)
}

pub fn decode_enigma(data: &[u8]) -> Result<Vec<u16>> {
    if data.len() < 6 {
        bail!("Enigma payload is too short");
    }
    let packet_length = usize::from(data[0]);
    if packet_length == 0 || packet_length > 16 {
        bail!("Enigma packet length must be between 1 and 16 bits");
    }
    let flag_mask = data[1];
    let mut incrementing_value = read_u16_be(data, 2);
    let common_value = read_u16_be(data, 4);
    let mut bits = BitReader::new(&data[6..]);
    let mut output = Vec::new();

    loop {
        if bits.read_bit()? == 1 {
            let mode = bits.read_bits(2)?;
            match mode {
                0..=2 => {
                    let count = usize::from(bits.read_bits(4)?) + 1;
                    let flags = read_enigma_flags(&mut bits, flag_mask)?;
                    let mut value = (bits.read_bits(packet_length)? as u16) | flags;
                    for _ in 0..count {
                        output.push(value);
                        value = match mode {
                            0 => value,
                            1 => value.wrapping_add(1),
                            2 => value.wrapping_sub(1),
                            _ => unreachable!(),
                        };
                    }
                }
                3 => {
                    let count = usize::from(bits.read_bits(4)?);
                    if count == 0x0F {
                        break;
                    }
                    for _ in 0..=count {
                        let flags = read_enigma_flags(&mut bits, flag_mask)?;
                        let value = (bits.read_bits(packet_length)? as u16) | flags;
                        output.push(value);
                    }
                }
                _ => unreachable!(),
            }
        } else if bits.read_bit()? == 0 {
            let count = usize::from(bits.read_bits(4)?) + 1;
            for _ in 0..count {
                output.push(incrementing_value);
                incrementing_value = incrementing_value.wrapping_add(1);
            }
        } else {
            let count = usize::from(bits.read_bits(4)?) + 1;
            output.extend(std::iter::repeat_n(common_value, count));
        }
    }

    Ok(output)
}

fn read_enigma_flags(bits: &mut BitReader<'_>, flag_mask: u8) -> Result<u16> {
    let mut flags = 0_u16;
    for (mask_bit, flag) in [
        (4_u8, 1_u16 << 15),
        (3_u8, 1_u16 << 14),
        (2_u8, 1_u16 << 13),
        (1_u8, 1_u16 << 12),
        (0_u8, 1_u16 << 11),
    ] {
        if flag_mask & (1 << mask_bit) != 0 && bits.read_bit()? == 1 {
            flags |= flag;
        }
    }
    Ok(flags)
}

fn read_input_byte(data: &[u8], input_position: &mut usize) -> Result<u8> {
    let byte = *data
        .get(*input_position)
        .ok_or_else(|| anyhow!("compressed payload ended unexpectedly"))?;
    *input_position += 1;
    Ok(byte)
}

fn copy_from_output(output: &mut Vec<u8>, distance: usize, count: usize) -> Result<()> {
    if distance == 0 {
        bail!("back-reference distance cannot be zero");
    }
    for _ in 0..count {
        let value = output
            .len()
            .checked_sub(distance)
            .and_then(|source_index| output.get(source_index).copied())
            .unwrap_or(0);
        output.push(value);
    }
    Ok(())
}

fn write_zero_fill(output: &mut Vec<u8>, count: usize) {
    output.extend(std::iter::repeat_n(0_u8, count));
}

struct BitReader<'a> {
    bytes: &'a [u8],
    bit_index: usize,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_index: 0,
        }
    }

    fn read_bit(&mut self) -> Result<u8> {
        let byte = *self
            .bytes
            .get(self.bit_index / 8)
            .ok_or_else(|| anyhow!("compressed bitstream ended unexpectedly"))?;
        let shift = 7 - (self.bit_index % 8);
        self.bit_index += 1;
        Ok((byte >> shift) & 1)
    }

    fn read_bits(&mut self, count: usize) -> Result<u16> {
        let mut value = 0_u16;
        for _ in 0..count {
            value = (value << 1) | u16::from(self.read_bit()?);
        }
        Ok(value)
    }
}

fn read_u16_be(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32_be(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn find_enemy_source_path(manifest: &AssetManifest, symbolic_name: &str) -> Option<String> {
    let wanted = normalise_enemy_name(symbolic_name);
    manifest.enemy_sources.iter().find_map(|asset| {
        let stem = Path::new(&asset.relative_path)
            .file_stem()?
            .to_string_lossy();
        if normalise_enemy_name(&stem) == wanted {
            Some(asset.relative_path.clone())
        } else {
            None
        }
    })
}

fn normalise_enemy_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

fn copy_directory_recursive(source: &Path, dest: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }
    fs::create_dir_all(dest).with_context(|| format!("failed to create {}", dest.display()))?;
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let source_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if entry_type.is_dir() {
            copy_directory_recursive(&source_path, &dest_path)?;
        } else if entry_type.is_file() {
            copy_file(&source_path, &dest_path)?;
        }
    }
    Ok(())
}

fn copy_file(source: &Path, dest: &Path) -> Result<()> {
    if !source.is_file() {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::copy(source, dest).with_context(|| {
        format!(
            "failed to copy asset from {} to {}",
            source.display(),
            dest.display()
        )
    })?;
    Ok(())
}

fn collect_asset_files(root: &Path) -> Result<Vec<AssetFile>> {
    let mut files = Vec::new();
    if !root.exists() {
        return Ok(files);
    }
    collect_asset_files_recursive(root, root, &mut files)?;
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

fn collect_asset_files_recursive(
    root: &Path,
    current: &Path,
    files: &mut Vec<AssetFile>,
) -> Result<()> {
    for entry in
        fs::read_dir(current).with_context(|| format!("failed to read {}", current.display()))?
    {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let path = entry.path();
        if entry_type.is_dir() {
            collect_asset_files_recursive(root, &path, files)?;
            continue;
        }
        if !entry_type.is_file() {
            continue;
        }

        let bytes =
            fs::read(&path).with_context(|| format!("failed to read asset {}", path.display()))?;
        let relative = path
            .strip_prefix(root)
            .map_err(|_| anyhow!("{} is not under {}", path.display(), root.display()))?;
        files.push(AssetFile {
            relative_path: normalise_relative_path(relative),
            size_bytes: bytes.len() as u64,
            sha1: sha1_hex(&bytes),
        });
    }
    Ok(())
}

fn parse_level_names(contents: &str) -> Result<BTreeMap<u8, (u8, String)>> {
    let mut levels = BTreeMap::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with("0x") {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let level_id = parse_hex_byte(
            parts
                .next()
                .ok_or_else(|| anyhow!("missing level id on line {}", index + 1))?,
        )?;
        let title_text_id = parse_hex_byte(
            parts
                .next()
                .ok_or_else(|| anyhow!("missing title id on line {}", index + 1))?,
        )?;
        let display_name = parts.collect::<Vec<_>>().join(" ");
        if display_name.is_empty() {
            bail!("missing level name on line {}", index + 1);
        }
        levels.insert(level_id, (title_text_id, display_name));
    }
    Ok(levels)
}

fn parse_level_files(contents: &str) -> Result<BTreeMap<u8, Vec<String>>> {
    let mut levels = BTreeMap::new();
    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let level_id = parse_hex_byte(
            parts
                .next()
                .ok_or_else(|| anyhow!("missing level id on line {}", index + 1))?,
        )?;
        let resource_paths = parts.map(str::to_owned).collect::<Vec<_>>();
        levels.insert(level_id, resource_paths);
    }
    Ok(levels)
}

fn parse_named_values(contents: &str) -> Result<Vec<NamedValue>> {
    let mut values = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        let (assignment, comment) = split_comment(trimmed);
        let Some((name, value)) = assignment.split_once('=') else {
            continue;
        };
        let value = match parse_numeric_value(value.trim()) {
            Ok(value) => value,
            Err(_) => continue,
        };
        values.push(NamedValue {
            name: name.trim().to_owned(),
            value,
            comment: comment.map(str::to_owned),
        });
    }
    Ok(values)
}

fn parse_ram_variables(contents: &str) -> Result<Vec<RamVariable>> {
    let mut values = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        let (assignment, comment) = split_comment(trimmed);
        let Some((name, value)) = assignment.split_once('=') else {
            continue;
        };
        let value = value.trim();
        if !value.starts_with('$') {
            continue;
        }
        values.push(RamVariable {
            name: name.trim().to_owned(),
            address: parse_hex_u32(value)?,
            detail: comment.map(str::to_owned),
        });
    }
    Ok(values)
}

fn split_comment(line: &str) -> (&str, Option<&str>) {
    if let Some((assignment, comment)) = line.split_once(';') {
        (assignment.trim_end(), Some(comment.trim()))
    } else {
        (line, None)
    }
}

fn parse_numeric_value(value: &str) -> Result<i64> {
    if let Some(hex) = value.strip_prefix('$') {
        return i64::from_str_radix(hex, 16).with_context(|| format!("invalid hex value {value}"));
    }
    value
        .parse::<i64>()
        .with_context(|| format!("invalid integer value {value}"))
}

fn parse_hex_byte(value: &str) -> Result<u8> {
    let hex = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value);
    u8::from_str_radix(hex, 16).with_context(|| format!("invalid hex byte {value}"))
}

fn parse_hex_u32(value: &str) -> Result<u32> {
    let hex = value
        .strip_prefix('$')
        .ok_or_else(|| anyhow!("expected $-prefixed address, got {value}"))?;
    u32::from_str_radix(hex, 16).with_context(|| format!("invalid hex address {value}"))
}

fn sha1_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn normalise_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const SAXMAN_FIXTURE: &[u8] = &[0x00, 0x02, 0xF4, 0x60, 0x41, 0x42, 0x43, 0x44, 0x00, 0x00];
    const ENIGMA_FIXTURE: &[u8] = &[0x05, 0x00, 0x00, 0x01, 0x00, 0x0A, 0x09, 0x18, 0x10, 0xFE];
    const REAL_STYLE_BLOCK_FIXTURE: &[u8] = &[
        0x40, 0x76, 0x02, 0x08, 0xA0, 0x80, 0x38, 0x1F, 0x92, 0x08, 0x64, 0x82, 0x81, 0x40, 0x9E,
        0x58, 0x28, 0x96, 0x08, 0x86, 0x02, 0x99, 0x80, 0xA8, 0x68, 0x0B, 0x1C, 0x03, 0x07, 0x00,
        0xD1, 0xC0,
    ];
    const REAL_STYLE_BACKGROUND_FIXTURE: &[u8] = &[
        0x00, 0x0D, 0x00, 0x00, 0x00, 0x2D, 0x00, 0x0D, 0x00, 0x1E, 0x00, 0x2D, 0x00, 0x0D, 0x00,
        0x3C, 0x00, 0x2D, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x29, 0x00, 0x0F, 0x00, 0x24, 0x00, 0x29,
        0x00, 0x0F, 0x00, 0x3C, 0x00, 0x29, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x02, 0x00,
        0x06, 0x00, 0x20, 0x00, 0x02, 0x00, 0x24, 0x00, 0x1F, 0x00, 0x02, 0x00, 0x26, 0x00, 0x0A,
        0x00, 0x06, 0x00, 0x28,
    ];

    fn write_test_bundle_source(
        source_root: &Path,
        foreground_bytes: &[u8],
        block_bytes: &[u8],
        background_bytes: &[u8],
    ) {
        fs::create_dir_all(source_root.join("code/enemy")).expect("enemy dir");
        fs::create_dir_all(source_root.join("doc")).expect("doc dir");
        fs::create_dir_all(source_root.join("ingame/anim")).expect("anim dir");
        fs::create_dir_all(source_root.join("ingame/artcomp")).expect("artcomp dir");
        fs::create_dir_all(source_root.join("ingame/artunc")).expect("artunc dir");
        fs::create_dir_all(source_root.join("ingame/artunc_kid")).expect("kid art dir");
        fs::create_dir_all(source_root.join("ingame/palette_enemy")).expect("enemy palette dir");
        fs::create_dir_all(source_root.join("ingame/palette_kid")).expect("kid palette dir");
        fs::create_dir_all(source_root.join("level")).expect("level dir");
        fs::create_dir_all(source_root.join("level/platform")).expect("platform assets dir");
        fs::create_dir_all(source_root.join("level/bgscroll")).expect("bgscroll assets dir");
        fs::create_dir_all(source_root.join("level/header")).expect("header assets dir");
        fs::create_dir_all(source_root.join("level/enemy")).expect("enemy assets dir");
        fs::create_dir_all(source_root.join("level/foreground")).expect("foreground assets dir");
        fs::create_dir_all(source_root.join("level/block")).expect("block assets dir");
        fs::create_dir_all(source_root.join("level/background")).expect("background assets dir");
        fs::create_dir_all(source_root.join("scenes")).expect("scene dir");
        fs::create_dir_all(source_root.join("sound")).expect("sound dir");
        fs::create_dir_all(source_root.join("theme")).expect("theme dir");

        fs::write(source_root.join("code/enemy/Armadillo.asm"), b"enemy code").expect("enemy");
        fs::write(source_root.join("theme/sample.bin"), b"theme").expect("theme");
        fs::write(source_root.join("scenes/title.bin"), b"scene").expect("scene");
        fs::write(source_root.join("ingame/anim/idle.bin"), b"anim").expect("anim");
        fs::write(
            source_root.join("ingame/artcomp/Blocks.bin"),
            SAXMAN_FIXTURE,
        )
        .expect("blocks");
        fs::write(
            source_root.join("ingame/artcomp/HUD_numbers.bin"),
            SAXMAN_FIXTURE,
        )
        .expect("hud numbers");
        fs::write(source_root.join("ingame/artunc/diamond_1.bin"), &[0; 32]).expect("diamond");
        fs::write(
            source_root.join("ingame/palette_enemy/Crystal_1.bin"),
            &[0; 30],
        )
        .expect("diamond palette");
        fs::write(
            source_root.join("ingame/palette_enemy/Mini_Hopping_Skull_1.bin"),
            &[0; 30],
        )
        .expect("skull palette");
        fs::write(source_root.join("ingame/palette_kid/Kid.bin"), &[0; 30]).expect("kid palette");
        fs::write(source_root.join("ingame/artunc_kid/kid.bin"), &[0; 32 * 12]).expect("kid idle");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_walk_1.bin"),
            &[1; 32 * 12],
        )
        .expect("kid walk");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_jumping_1.bin"),
            &[2; 32 * 12],
        )
        .expect("kid jump rise");
        fs::write(
            source_root.join("ingame/artunc_kid/kid_jumping_2.bin"),
            &[3; 32 * 12],
        )
        .expect("kid jump fall");
        fs::write(source_root.join("sound/music.bin"), b"sound").expect("sound");
        fs::write(
            source_root.join("level/platform/00.asm"),
            b"\tptfm\t0,0,0,0,0,0,0,0,0,0,PlatformScript_Nothing-PlatformScript_BaseAddress\n\tdc.w\t$FFFF\n",
        )
        .expect("platform");
        fs::write(
            source_root.join("level/bgscroll/00.bin"),
            [0x10, 0x10, 0x40, 0x60, 0xFF],
        )
        .expect("bgscroll");
        fs::write(
            source_root.join("level/header/00.bin"),
            [
                0x06, 0x04, 0x08, 0x18, 0x07, 0x50, 0x00, 0xE0, 0x07, 0x50, 0x03, 0x00,
            ],
        )
        .expect("header");
        fs::write(
            source_root.join("level/enemy/00.bin"),
            [
                0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x01, 0x00, 0x1C, 0x7D, 0x00, 0x80, 0x01,
                0x80, 0x16, 0x00, 0x01, 0x01, 0x20, 0x00, 0xE0,
            ],
        )
        .expect("enemy");
        fs::write(
            source_root.join("level/foreground/00.bin"),
            foreground_bytes,
        )
        .expect("foreground");
        fs::write(source_root.join("level/block/00.bin"), block_bytes).expect("block");
        fs::write(
            source_root.join("level/background/00.bin"),
            background_bytes,
        )
        .expect("background");
        fs::write(
            source_root.join("doc/level_list.txt"),
            "0x00 0x4A Blue Lake Woods 1\n",
        )
        .expect("level names");
        fs::write(
            source_root.join("level/level_files.txt"),
            "00 platform/00.asm bgscroll/00.bin header/00.bin enemy/00.bin foreground/00.bin block/00.bin background/00.bin\n",
        )
        .expect("level files");
        fs::write(source_root.join("constants.asm"), "Theme_Sky = 1 ; sky\n").expect("constants");
        fs::write(
            source_root.join("variables.asm"),
            "Camera_X_pos = $FFFFF81C ; long (16.16 fixed point)\n",
        )
        .expect("variables");
    }

    fn write_default_test_bundle_source(source_root: &Path) {
        write_test_bundle_source(
            source_root,
            SAXMAN_FIXTURE,
            REAL_STYLE_BLOCK_FIXTURE,
            REAL_STYLE_BACKGROUND_FIXTURE,
        );
    }

    #[test]
    fn parses_level_names_and_titles() {
        let parsed =
            parse_level_names("0x00 0x4A Blue Lake Woods 1\n0x01 0x4B Blue Lake Woods 2\n")
                .expect("level names should parse");
        assert_eq!(
            parsed.get(&0x00),
            Some(&(0x4A, String::from("Blue Lake Woods 1")))
        );
        assert_eq!(
            parsed.get(&0x01),
            Some(&(0x4B, String::from("Blue Lake Woods 2")))
        );
    }

    #[test]
    fn parses_level_files() {
        let parsed = parse_level_files("00 platform/00.asm bgscroll/00.bin header/00.bin\n")
            .expect("level files should parse");
        assert_eq!(
            parsed.get(&0x00),
            Some(&vec![
                String::from("platform/00.asm"),
                String::from("bgscroll/00.bin"),
                String::from("header/00.bin"),
            ])
        );
    }

    #[test]
    fn classifies_level_resources() {
        let resources = level_resources_from_paths(&[
            String::from("platform/00.asm"),
            String::from("bgscroll/00.bin"),
            String::from("header/00.bin"),
            String::from("enemy/00.bin"),
            String::from("foreground/00.bin"),
            String::from("block/00.bin"),
            String::from("background/00.bin"),
        ]);
        assert_eq!(
            resources.platform_script.as_deref(),
            Some("platform/00.asm")
        );
        assert_eq!(resources.bgscroll.as_deref(), Some("bgscroll/00.bin"));
        assert_eq!(resources.header.as_deref(), Some("header/00.bin"));
        assert_eq!(resources.enemy.as_deref(), Some("enemy/00.bin"));
        assert_eq!(resources.foreground.as_deref(), Some("foreground/00.bin"));
        assert_eq!(resources.block.as_deref(), Some("block/00.bin"));
        assert_eq!(resources.background.as_deref(), Some("background/00.bin"));
    }

    #[test]
    fn resolves_enemy_definitions_to_sources() {
        let manifest = AssetManifest {
            generated_bundle_version: GENERATED_BUNDLE_VERSION,
            rom_sha1: String::from("abc123"),
            upstream_commit: String::from("deadbeef"),
            source_root: PathBuf::from("/tmp/source"),
            output_root: PathBuf::from("/tmp/output"),
            levels: Vec::new(),
            constants: vec![
                NamedValue {
                    name: String::from("objid_Armadillo"),
                    value: 0x04,
                    comment: None,
                },
                NamedValue {
                    name: String::from("objid_TarMonster"),
                    value: 0x05,
                    comment: None,
                },
            ],
            ram_variables: Vec::new(),
            enemy_sources: vec![
                AssetFile {
                    relative_path: String::from("Armadillo.asm"),
                    size_bytes: 123,
                    sha1: String::from("x"),
                },
                AssetFile {
                    relative_path: String::from("Tar_Monster.asm"),
                    size_bytes: 456,
                    sha1: String::from("y"),
                },
            ],
            theme_assets: Vec::new(),
            scene_assets: Vec::new(),
            animation_assets: Vec::new(),
            sound_assets: Vec::new(),
            ingame_assets: Vec::new(),
            decoded_level1: None,
        };

        let definitions = enemy_definitions(&manifest);
        assert_eq!(definitions.len(), 2);
        assert_eq!(definitions[0].source_path.as_deref(), Some("Armadillo.asm"));
        assert_eq!(
            definitions[1].source_path.as_deref(),
            Some("Tar_Monster.asm")
        );
        assert_eq!(
            find_enemy_definition(&manifest, "04")
                .as_ref()
                .map(|definition| definition.symbolic_name.as_str()),
            Some("Armadillo")
        );
        assert_eq!(
            find_enemy_definition(&manifest, "tarmonster")
                .as_ref()
                .map(|definition| definition.id),
            Some(0x05)
        );
    }

    #[test]
    fn parses_level_header_fields() {
        let header = parse_level_header(&[
            0x06, 0x84, 0x88, 0x18, 0x07, 0x50, 0x00, 0xE0, 0x07, 0x50, 0x03, 0x00,
        ])
        .expect("header should parse");
        assert_eq!(header.width_screens, 0x06);
        assert_eq!(header.height_screens, 0x04);
        assert_eq!(header.height_flag_bits, 0x02);
        assert_eq!(header.level_theme, 0x08);
        assert_eq!(header.murder_wall_flags, 0x02);
        assert_eq!(header.background_theme, 0x08);
        assert_eq!(header.weather_flags, 0x01);
        assert_eq!(header.player_start_x, 0x0750);
        assert_eq!(header.player_start_y, 0x00E0);
        assert_eq!(header.flag_x, 0x0750);
        assert_eq!(header.flag_y, 0x0300);
    }

    #[test]
    fn parses_enemy_layout_header_and_objects() {
        let layout = parse_enemy_layout(&[
            0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x01, 0x40, 0x1C, 0x7D, 0x00, 0x80, 0x02,
            0x80, 0x16, 0x00, 0x01, 0x01, 0x20, 0x00, 0xE0, 0x81, 0xFF, 0x00, 0x02, 0x03, 0x40,
            0x02, 0x00,
        ])
        .expect("enemy layout should parse");

        assert_eq!(layout.unknown_prefix, 0x0000_0000);
        assert_eq!(layout.enemy_type_slots, vec![0xFFFF, 0x0001, 0x401C]);
        assert_eq!(layout.unknown_word, 0x7D00);
        assert_eq!(layout.header_byte, 0x80);
        assert_eq!(layout.object_count, 0x02);
        assert_eq!(layout.objects.len(), 2);
        assert_eq!(layout.objects[0].object_type, 0x80);
        assert_eq!(layout.objects[0].flags, 0x16);
        assert_eq!(layout.objects[0].hit_points_plus_one, 0x0001);
        assert_eq!(layout.objects[0].x, 0x0120);
        assert_eq!(layout.objects[0].y, 0x00E0);
        assert_eq!(layout.objects[1].object_type, 0x81);
        assert_eq!(layout.objects[1].flags, 0xFF);
        assert_eq!(layout.objects[1].hit_points_plus_one, 0x0002);
        assert_eq!(layout.objects[1].x, 0x0340);
        assert_eq!(layout.objects[1].y, 0x0200);
    }

    #[test]
    fn parses_bgscroll_profile_layers() {
        let profile =
            parse_bgscroll_profile(&[0x10, 0x10, 0x40, 0x60, 0xFF]).expect("bgscroll should parse");
        assert_eq!(profile.layer_speeds, vec![0x10, 0x10, 0x40, 0x60]);
        assert!(profile.has_terminator);
    }

    #[test]
    fn parses_unterminated_bgscroll_profile_layers() {
        let profile =
            parse_bgscroll_profile(&[0x10, 0x10, 0x40, 0x60]).expect("bgscroll should parse");
        assert_eq!(profile.layer_speeds, vec![0x10, 0x10, 0x40, 0x60]);
        assert!(!profile.has_terminator);
    }

    #[test]
    fn parses_platform_layout_entries() {
        let layout = parse_platform_layout(
            "\tptfm\t24,25,0,40,0,19,0,2,3,0,PlatformScript_3C3A-PlatformScript_BaseAddress\n\tdc.w\t$FFFF\n",
        )
        .expect("platform layout should parse");
        assert_eq!(layout.placements.len(), 1);
        let placement = &layout.placements[0];
        assert_eq!(placement.x, 24);
        assert_eq!(placement.y, 25);
        assert_eq!(placement.buffer_right, 40);
        assert_eq!(placement.kind, 0);
        assert_eq!(placement.subtype, 2);
        assert_eq!(placement.h_variant, 3);
        assert_eq!(placement.v_variant, 0);
        assert_eq!(
            placement.script.raw,
            "PlatformScript_3C3A-PlatformScript_BaseAddress"
        );
        assert_eq!(placement.script.relative_offset, Some(0x3C3A));
    }

    #[test]
    fn saxman_fixture_decodes_to_expected_bytes() {
        let decoded = decode_saxman(SAXMAN_FIXTURE).expect("fixture should decode");
        assert_eq!(decoded, b"ABCD");
    }

    #[test]
    fn enigma_fixture_decodes_to_expected_tiles() {
        let decoded = decode_enigma(ENIGMA_FIXTURE).expect("fixture should decode");
        assert_eq!(
            decoded,
            vec![0x0001, 0x0002, 0x0003, 0x000A, 0x000A, 0x0010]
        );
    }

    #[test]
    fn export_split_bundle_writes_decoded_level1_entries() {
        let source = TempDir::new().expect("temp dir");
        let output = TempDir::new().expect("temp dir");
        let source_root = source.path();
        write_default_test_bundle_source(source_root);

        let manifest = export_split_bundle(source_root, output.path(), "abc123", "deadbeef")
            .expect("bundle export should succeed");

        assert_eq!(manifest.generated_bundle_version, GENERATED_BUNDLE_VERSION);
        let decoded_level1 = manifest
            .decoded_level1
            .as_ref()
            .expect("decoded Level 1 contract should be present");
        assert_eq!(
            decoded_level1.contract_version,
            LEVEL1_DECODED_ASSET_VERSION
        );
        assert_eq!(decoded_level1.level_id, 0x00);
        assert_eq!(
            fs::read(output.path().join(&decoded_level1.foreground.relative_path))
                .expect("decoded foreground"),
            b"ABCD"
        );
        assert_eq!(
            fs::read(output.path().join(&decoded_level1.block.relative_path))
                .expect("decoded block"),
            REAL_STYLE_BLOCK_FIXTURE
        );
        assert_eq!(
            fs::read(output.path().join(&decoded_level1.background.relative_path))
                .expect("decoded background"),
            REAL_STYLE_BACKGROUND_FIXTURE
        );
        assert!(
            output
                .path()
                .join(&decoded_level1.foreground.preview_png_path)
                .is_file()
        );
        assert!(
            output
                .path()
                .join(&decoded_level1.block.preview_png_path)
                .is_file()
        );
        assert!(
            output
                .path()
                .join(&decoded_level1.background.preview_png_path)
                .is_file()
        );
        assert_eq!(manifest.rom_sha1, "abc123");
        assert_eq!(manifest.levels.len(), 1);
        assert_eq!(
            manifest.levels[0].resources.platform_script.as_deref(),
            Some("platform/00.asm")
        );
        assert_eq!(
            manifest.levels[0]
                .header_metadata
                .as_ref()
                .map(|header| header.width_screens),
            Some(0x06)
        );
        assert_eq!(
            manifest.levels[0]
                .enemy_layout_metadata
                .as_ref()
                .map(|layout| layout.object_count),
            Some(0x01)
        );
        assert_eq!(
            manifest.levels[0]
                .bgscroll_metadata
                .as_ref()
                .map(|profile| (profile.layer_speeds.clone(), profile.has_terminator)),
            Some((vec![0x10, 0x10, 0x40, 0x60], true))
        );
        assert_eq!(
            manifest.levels[0]
                .platform_metadata
                .as_ref()
                .map(|layout| layout.placements.len()),
            Some(1)
        );
        assert_eq!(manifest.enemy_sources.len(), 1);
        assert_eq!(manifest.theme_assets.len(), 1);
        assert_eq!(manifest.ingame_assets.len(), 9);
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "anim/idle.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artcomp/Blocks.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artcomp/HUD_numbers.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artunc/diamond_1.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "palette_enemy/Crystal_1.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "palette_enemy/Mini_Hopping_Skull_1.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "palette_kid/Kid.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artunc_kid/kid.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artunc_kid/kid_walk_1.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artunc_kid/kid_jumping_1.bin")
        );
        assert!(
            manifest
                .ingame_assets
                .iter()
                .any(|asset| asset.relative_path == "artunc_kid/kid_jumping_2.bin")
        );
        assert!(manifest_path(output.path()).is_file());

        let level_bundle =
            load_level_asset_bundle(&manifest, 0x00).expect("level bundle should load");
        assert_eq!(level_bundle.level_id, 0x00);
        assert_eq!(
            level_bundle.display_name.as_deref(),
            Some("Blue Lake Woods 1")
        );
        assert_eq!(
            level_bundle
                .platform_script
                .as_ref()
                .map(|asset| asset.bytes.as_slice()),
            Some(
                &b"\tptfm\t0,0,0,0,0,0,0,0,0,0,PlatformScript_Nothing-PlatformScript_BaseAddress\n\tdc.w\t$FFFF\n"
                    [..]
            )
        );
        assert_eq!(
            level_bundle
                .parsed_header
                .as_ref()
                .map(|header| header.width_screens),
            Some(0x06)
        );
        assert_eq!(
            level_bundle
                .parsed_header
                .as_ref()
                .map(|header| header.flag_y),
            Some(0x0300)
        );
        assert_eq!(
            level_bundle
                .parsed_bgscroll
                .as_ref()
                .map(|profile| (profile.layer_speeds.clone(), profile.has_terminator)),
            Some((vec![0x10, 0x10, 0x40, 0x60], true))
        );
        assert_eq!(
            level_bundle
                .parsed_platform_layout
                .as_ref()
                .map(|layout| layout.placements.len()),
            Some(1)
        );
        assert_eq!(
            level_bundle
                .parsed_enemy_layout
                .as_ref()
                .map(|layout| layout.object_count),
            Some(0x01)
        );
        assert_eq!(
            level_bundle
                .parsed_enemy_layout
                .as_ref()
                .and_then(|layout| layout.objects.first())
                .map(|object| object.object_type),
            Some(0x80)
        );
    }

    #[test]
    fn export_split_bundle_keeps_existing_bundle_when_regeneration_fails() {
        let initial_source = TempDir::new().expect("temp dir");
        write_default_test_bundle_source(initial_source.path());
        let output_root = TempDir::new().expect("temp dir");
        let output_path = output_root.path().join("bundle");

        let original_manifest =
            export_split_bundle(initial_source.path(), &output_path, "abc123", "deadbeef")
                .expect("initial export should succeed");
        let original_manifest_path = manifest_path(&output_path);
        let original_manifest_bytes =
            fs::read(&original_manifest_path).expect("original manifest should exist");

        let failing_source = TempDir::new().expect("temp dir");
        write_test_bundle_source(
            failing_source.path(),
            &[0x00, 0x00],
            REAL_STYLE_BLOCK_FIXTURE,
            ENIGMA_FIXTURE,
        );

        let error = export_split_bundle(failing_source.path(), &output_path, "abc123", "deadbeef")
            .expect_err("regeneration should fail");
        assert!(error.to_string().contains("Saxman"));

        let retained_manifest_bytes =
            fs::read(&original_manifest_path).expect("original manifest should still exist");
        assert_eq!(retained_manifest_bytes, original_manifest_bytes);
        let retained_manifest = load_manifest(&original_manifest_path).expect("manifest reload");
        assert_eq!(retained_manifest, original_manifest);
        assert!(
            output_path
                .join(
                    retained_manifest
                        .decoded_level1
                        .as_ref()
                        .expect("decoded metadata")
                        .foreground
                        .relative_path
                        .as_str()
                )
                .is_file()
        );
    }
}
