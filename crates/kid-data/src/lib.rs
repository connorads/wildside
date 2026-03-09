use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const COPIED_DIRECTORIES: &[&str] = &[
    "code/enemy",
    "doc",
    "ingame/anim",
    "level",
    "scenes",
    "sound",
    "theme",
];
const COPIED_FILES: &[&str] = &["constants.asm", "variables.asm"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManifest {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelRecord {
    pub level_id: u8,
    pub title_text_id: Option<u8>,
    pub display_name: Option<String>,
    pub resource_paths: Vec<String>,
    #[serde(default)]
    pub resources: LevelResources,
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
    pub bgscroll: Option<LoadedAsset>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyLayout {
    pub unknown_prefix: u32,
    pub enemy_type_slots: Vec<u16>,
    pub unknown_word: u16,
    pub header_byte: u8,
    pub object_count: u8,
    pub objects: Vec<EnemyLayoutObject>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyLayoutObject {
    pub object_type: u8,
    pub flags: u8,
    pub hit_points_plus_one: u16,
    pub x: u16,
    pub y: u16,
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

    if output_root.exists() {
        fs::remove_dir_all(output_root)
            .with_context(|| format!("failed to remove {}", output_root.display()))?;
    }
    fs::create_dir_all(output_root)
        .with_context(|| format!("failed to create {}", output_root.display()))?;

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
    };

    fs::write(
        manifest_path(output_root),
        serde_json::to_vec_pretty(&manifest)?,
    )
    .with_context(|| format!("failed to write {}", manifest_path(output_root).display()))?;

    Ok(manifest)
}

pub fn load_manifest(path: &Path) -> Result<AssetManifest> {
    let bytes =
        fs::read(path).with_context(|| format!("failed to read manifest {}", path.display()))?;
    let mut manifest: AssetManifest =
        serde_json::from_slice(&bytes).context("failed to parse asset manifest")?;
    normalise_manifest(&mut manifest);
    Ok(manifest)
}

pub fn load_level_asset_bundle(manifest: &AssetManifest, level_id: u8) -> Result<LevelAssetBundle> {
    let level = manifest
        .levels
        .iter()
        .find(|level| level.level_id == level_id)
        .ok_or_else(|| anyhow!("level 0x{level_id:02X} not found in asset manifest"))?;
    let header = load_asset(manifest, level.resources.header.as_deref())?;
    let parsed_header = header
        .as_ref()
        .map(|asset| parse_level_header(&asset.bytes))
        .transpose()?;
    let enemy = load_asset(manifest, level.resources.enemy.as_deref())?;
    let parsed_enemy_layout = enemy
        .as_ref()
        .map(|asset| parse_enemy_layout(&asset.bytes))
        .transpose()?;
    Ok(LevelAssetBundle {
        level_id,
        display_name: level.display_name.clone(),
        platform_script: load_asset(manifest, level.resources.platform_script.as_deref())?,
        bgscroll: load_asset(manifest, level.resources.bgscroll.as_deref())?,
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
        });
    }
    by_level.into_values().collect()
}

fn normalise_manifest(manifest: &mut AssetManifest) {
    for level in &mut manifest.levels {
        if level.resources == LevelResources::default() {
            level.resources = level_resources_from_paths(&level.resource_paths);
        }
    }
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
    fn exports_bundle_and_writes_manifest() {
        let source = TempDir::new().expect("temp dir");
        let output = TempDir::new().expect("temp dir");
        let source_root = source.path();

        fs::create_dir_all(source_root.join("code/enemy")).expect("enemy dir");
        fs::create_dir_all(source_root.join("doc")).expect("doc dir");
        fs::create_dir_all(source_root.join("ingame/anim")).expect("anim dir");
        fs::create_dir_all(source_root.join("level")).expect("level dir");
        fs::create_dir_all(source_root.join("level/platform")).expect("platform assets dir");
        fs::create_dir_all(source_root.join("level/bgscroll")).expect("bgscroll assets dir");
        fs::create_dir_all(source_root.join("level/header")).expect("header assets dir");
        fs::create_dir_all(source_root.join("level/enemy")).expect("enemy assets dir");
        fs::create_dir_all(source_root.join("scenes")).expect("scene dir");
        fs::create_dir_all(source_root.join("sound")).expect("sound dir");
        fs::create_dir_all(source_root.join("theme")).expect("theme dir");

        fs::write(source_root.join("code/enemy/Armadillo.asm"), b"enemy code").expect("enemy");
        fs::write(source_root.join("theme/sample.bin"), b"theme").expect("theme");
        fs::write(source_root.join("scenes/title.bin"), b"scene").expect("scene");
        fs::write(source_root.join("ingame/anim/idle.bin"), b"anim").expect("anim");
        fs::write(source_root.join("sound/music.bin"), b"sound").expect("sound");
        fs::write(source_root.join("level/platform/00.asm"), b"platform").expect("platform");
        fs::write(source_root.join("level/bgscroll/00.bin"), b"bgscroll").expect("bgscroll");
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
            source_root.join("doc/level_list.txt"),
            "0x00 0x4A Blue Lake Woods 1\n",
        )
        .expect("level names");
        fs::write(
            source_root.join("level/level_files.txt"),
            "00 platform/00.asm bgscroll/00.bin header/00.bin enemy/00.bin\n",
        )
        .expect("level files");
        fs::write(source_root.join("constants.asm"), "Theme_Sky = 1 ; sky\n").expect("constants");
        fs::write(
            source_root.join("variables.asm"),
            "Camera_X_pos = $FFFFF81C ; long (16.16 fixed point)\n",
        )
        .expect("variables");

        let manifest = export_split_bundle(source_root, output.path(), "abc123", "deadbeef")
            .expect("bundle export should succeed");

        assert_eq!(manifest.rom_sha1, "abc123");
        assert_eq!(manifest.levels.len(), 1);
        assert_eq!(
            manifest.levels[0].resources.platform_script.as_deref(),
            Some("platform/00.asm")
        );
        assert_eq!(manifest.enemy_sources.len(), 1);
        assert_eq!(manifest.theme_assets.len(), 1);
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
            Some(&b"platform"[..])
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
}
