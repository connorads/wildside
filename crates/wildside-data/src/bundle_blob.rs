//! Single-file serialisation of a generated asset bundle.
//!
//! The web build cannot walk a directory tree: the browser fetches exactly one
//! artefact and the engine reads assets from it through [`AssetSource`]. This
//! module defines that artefact — a length-prefixed archive of every file under
//! a bundle `output_root`, keyed by bundle-relative path — plus the two ends of
//! the pipe: [`pack_bundle_blob`] (native, packs the extracted bundle) and
//! [`BundleAssetSource`] (any target, parses the blob back into a source).
//!
//! Ships decoded/derived assets only, never raw ROM bytes; per the decision on
//! #16 a packed blob is for the private (bundled) build.
//!
//! Format (all integers little-endian):
//!
//! ```text
//! magic   b"WSBNDL01"                      (8 bytes)
//! count   u32                              number of entries
//! entry*  path_len u32, path (UTF-8, `/`-separated bundle-relative),
//!         data_len u32, data
//! ```
//!
//! Entries are sorted by path so packing the same bundle twice is
//! byte-identical (the deploy story diffs blobs by hash).

use crate::{AssetManifest, AssetSource, InMemoryAssetSource, runtime_asset_catalogue};
use anyhow::{Context, Result, bail, ensure};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use thiserror::Error;
use wildside_rom::CanonicalKidRom;

const BLOB_MAGIC: &[u8; 8] = b"WSBNDL01";

#[derive(Debug, Error)]
pub enum RuntimeAssetExtractionError {
    #[error("invalid embedded runtime extraction recipe: {0}")]
    InvalidRecipe(String),
    #[error("runtime extraction recipe is missing source asset {0}")]
    MissingSource(String),
    #[error(transparent)]
    Bundle(#[from] anyhow::Error),
}

/// Extract the deterministic browser runtime bundle from a verified canonical
/// ROM entirely in memory.
pub fn extract_runtime_bundle(
    rom: &CanonicalKidRom,
) -> std::result::Result<Vec<u8>, RuntimeAssetExtractionError> {
    let mut entries = extract_rom_slice_entries(rom.as_bytes())?;
    let manifest_bytes = include_bytes!("../runtime-manifest.json").to_vec();
    let manifest: AssetManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| RuntimeAssetExtractionError::InvalidRecipe(error.to_string()))?;

    for decoded in &manifest.decoded_levels {
        let level = manifest
            .levels
            .iter()
            .find(|level| level.level_id == decoded.level_id)
            .ok_or_else(|| {
                RuntimeAssetExtractionError::InvalidRecipe(format!(
                    "decoded level 0x{:02X} has no level record",
                    decoded.level_id
                ))
            })?;
        let foreground_path =
            level_source_path(level.resources.foreground.as_deref(), "foreground")?;
        let block_path = level_source_path(level.resources.block.as_deref(), "block")?;
        let background_path =
            level_source_path(level.resources.background.as_deref(), "background")?;
        let foreground_encoded = entries.get(&foreground_path).ok_or_else(|| {
            RuntimeAssetExtractionError::MissingSource(foreground_path.display().to_string())
        })?;
        let foreground = crate::decode_saxman(foreground_encoded)?;
        let block = entries.get(&block_path).cloned().ok_or_else(|| {
            RuntimeAssetExtractionError::MissingSource(block_path.display().to_string())
        })?;
        let background = entries.get(&background_path).cloned().ok_or_else(|| {
            RuntimeAssetExtractionError::MissingSource(background_path.display().to_string())
        })?;
        entries.insert(PathBuf::from(&decoded.foreground.relative_path), foreground);
        entries.insert(PathBuf::from(&decoded.block.relative_path), block);
        entries.insert(PathBuf::from(&decoded.background.relative_path), background);
    }
    entries.insert(PathBuf::from("asset-manifest.json"), manifest_bytes);
    pack_entries(entries.into_iter().collect()).map_err(Into::into)
}

fn level_source_path(
    relative: Option<&str>,
    family: &str,
) -> std::result::Result<PathBuf, RuntimeAssetExtractionError> {
    relative
        .map(|path| Path::new("level").join(path))
        .ok_or_else(|| {
            RuntimeAssetExtractionError::InvalidRecipe(format!(
                "decoded level is missing its {family} source path"
            ))
        })
}

fn extract_rom_slice_entries(
    rom: &[u8],
) -> std::result::Result<BTreeMap<PathBuf, Vec<u8>>, RuntimeAssetExtractionError> {
    let mut entries = BTreeMap::new();
    for (line_index, line) in include_str!("../runtime-rom-assets.txt")
        .lines()
        .enumerate()
    {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts = line.splitn(3, ' ').collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err(RuntimeAssetExtractionError::InvalidRecipe(format!(
                "line {} does not have offset, length and path",
                line_index + 1
            )));
        }
        let offset = usize::from_str_radix(parts[0], 16).map_err(|error| {
            RuntimeAssetExtractionError::InvalidRecipe(format!(
                "line {} has invalid offset: {error}",
                line_index + 1
            ))
        })?;
        let length = usize::from_str_radix(parts[1], 16).map_err(|error| {
            RuntimeAssetExtractionError::InvalidRecipe(format!(
                "line {} has invalid length: {error}",
                line_index + 1
            ))
        })?;
        let end = offset.checked_add(length).ok_or_else(|| {
            RuntimeAssetExtractionError::InvalidRecipe(format!(
                "line {} slice length overflows",
                line_index + 1
            ))
        })?;
        let bytes = rom.get(offset..end).ok_or_else(|| {
            RuntimeAssetExtractionError::InvalidRecipe(format!(
                "line {} slice {offset:#x}..{end:#x} exceeds ROM",
                line_index + 1
            ))
        })?;
        entries.insert(PathBuf::from(parts[2]), bytes.to_vec());
    }
    Ok(entries)
}

/// Derive checked extractor inputs from a canonical generated bundle.
///
/// This is a maintainer/update surface, not part of browser extraction.
pub fn build_runtime_extractor_inputs(
    root: &Path,
    rom: &CanonicalKidRom,
) -> Result<(String, Vec<u8>)> {
    let manifest = std::fs::read(root.join("asset-manifest.json")).with_context(|| {
        format!(
            "failed to read {}",
            root.join("asset-manifest.json").display()
        )
    })?;
    let manifest = runtime_manifest_bytes(&manifest)?;
    let mut recipe = String::from(
        "# Canonical ROM offset, byte length, and runtime bundle path.\n\
         # Generated from exact byte matches; keep sorted by path.\n",
    );
    for relative in runtime_asset_catalogue()
        .into_iter()
        .filter(|relative| runtime_pack_path(relative))
        .filter(|relative| relative != Path::new("asset-manifest.json"))
        .filter(|relative| !relative.starts_with("decoded"))
        .filter(|relative| root.join(relative).is_file())
    {
        let bytes = std::fs::read(root.join(&relative))
            .with_context(|| format!("failed to read {}", root.join(&relative).display()))?;
        let offset = rom
            .as_bytes()
            .windows(bytes.len())
            .position(|window| window == bytes)
            .with_context(|| {
                format!(
                    "runtime asset {} is not an exact canonical ROM slice",
                    relative.display()
                )
            })?;
        writeln!(
            recipe,
            "{offset:06x} {:06x} {}",
            bytes.len(),
            relative.display()
        )
        .expect("writing to String cannot fail");
    }
    Ok((recipe, manifest))
}

/// Serialise every file under `root` (a bundle `output_root`) into a single
/// blob. Native-only in practice (walks the filesystem); the inverse of
/// [`BundleAssetSource::from_blob`].
pub fn pack_bundle_blob(root: &Path) -> Result<Vec<u8>> {
    let mut paths = Vec::new();
    collect_files(root, root, &mut paths)?;
    pack_selected_paths(root, paths)
}

/// Serialise the checked browser runtime catalogue from a generated bundle.
///
/// Catalogue entries that are only negative [`AssetSource::exists`] queries
/// are absent from the source and remain absent from the blob.
pub fn pack_runtime_bundle_blob(root: &Path) -> Result<Vec<u8>> {
    let paths = runtime_asset_catalogue()
        .into_iter()
        .filter(|relative| runtime_pack_path(relative))
        .filter(|relative| root.join(relative).is_file())
        .collect::<Vec<_>>();
    let mut entries = read_selected_entries(root, paths)?;
    let manifest_entry = entries
        .iter_mut()
        .find(|(path, _)| path == Path::new("asset-manifest.json"))
        .context("runtime catalogue has no asset-manifest.json")?;
    manifest_entry.1 = runtime_manifest_bytes(&manifest_entry.1)?;
    pack_entries(entries)
}

fn pack_selected_paths(root: &Path, mut paths: Vec<PathBuf>) -> Result<Vec<u8>> {
    paths.sort();
    pack_entries(read_selected_entries(root, paths)?)
}

fn read_selected_entries(root: &Path, paths: Vec<PathBuf>) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    paths
        .into_iter()
        .map(|relative| {
            let bytes = std::fs::read(root.join(&relative))
                .with_context(|| format!("failed to read {}", root.join(&relative).display()))?;
            Ok((relative, bytes))
        })
        .collect()
}

fn pack_entries(mut entries: Vec<(PathBuf, Vec<u8>)>) -> Result<Vec<u8>> {
    ensure!(
        !entries.is_empty(),
        "cannot pack a bundle with no asset entries"
    );
    entries.sort_by(|left, right| left.0.cmp(&right.0));

    let mut blob = Vec::new();
    blob.extend_from_slice(BLOB_MAGIC);
    blob.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for (rel, bytes) in &entries {
        let key = blob_key(rel)?;
        blob.extend_from_slice(&(key.len() as u32).to_le_bytes());
        blob.extend_from_slice(key.as_bytes());
        blob.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        blob.extend_from_slice(bytes);
    }
    Ok(blob)
}

fn runtime_pack_path(relative: &Path) -> bool {
    let value = relative.to_string_lossy();
    !value.ends_with(".png")
        && !value.starts_with("level-layout/")
        && value != "level/background_includes.asm"
        && value != "level/mapheader_definitions.asm"
        && value != "level/platform_scripts.asm"
        && !(value.starts_with("level/platform/") && value.ends_with(".asm"))
}

fn runtime_manifest_bytes(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut manifest: AssetManifest =
        serde_json::from_slice(bytes).context("failed to parse generated asset manifest")?;
    manifest.source_root = PathBuf::new();
    manifest.output_root = PathBuf::new();
    for level in &mut manifest.levels {
        level.resources.platform_script = None;
        level
            .resource_paths
            .retain(|path| !path.starts_with("platform/"));
    }
    let catalogue = runtime_asset_catalogue();
    manifest.enemy_sources.clear();
    manifest.theme_assets.retain(|asset| {
        catalogue.contains(&PathBuf::from(format!("theme/{}", asset.relative_path)))
    });
    manifest.scene_assets.retain(|asset| {
        catalogue.contains(&PathBuf::from(format!("scenes/{}", asset.relative_path)))
    });
    manifest.animation_assets.retain(|asset| {
        catalogue.contains(&PathBuf::from(format!(
            "ingame/anim/{}",
            asset.relative_path
        )))
    });
    manifest.sound_assets.retain(|asset| {
        catalogue.contains(&PathBuf::from(format!("sound/{}", asset.relative_path)))
    });
    manifest.ingame_assets.retain(|asset| {
        catalogue.contains(&PathBuf::from(format!("ingame/{}", asset.relative_path)))
    });
    serde_json::to_vec(&manifest).context("failed to serialise runtime asset manifest")
}

/// Bundle-relative key with `/` separators regardless of host platform, so a
/// blob packed on Windows parses identically elsewhere.
fn blob_key(rel: &Path) -> Result<String> {
    let mut parts = Vec::new();
    for component in rel.components() {
        match component {
            std::path::Component::Normal(part) => parts.push(
                part.to_str()
                    .with_context(|| format!("non-UTF-8 asset path {}", rel.display()))?,
            ),
            other => bail!(
                "asset path {} contains unsupported component {other:?}",
                rel.display()
            ),
        }
    }
    Ok(parts.join("/"))
}

fn collect_files(root: &Path, dir: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read bundle directory {}", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("failed to list bundle directory {}", dir.display()))?
            .path();
        if path.is_dir() {
            collect_files(root, &path, paths)?;
        } else if path.is_file() {
            paths.push(
                path.strip_prefix(root)
                    .expect("walked paths stay under root")
                    .to_path_buf(),
            );
        }
    }
    Ok(())
}

/// [`AssetSource`] over a parsed bundle blob — the web build's asset backend.
/// Byte-faithful to the packed `output_root`: reading `rel` returns exactly the
/// bytes `FsAssetSource` would have read natively.
#[derive(Debug, Clone)]
pub struct BundleAssetSource {
    files: InMemoryAssetSource,
}

impl BundleAssetSource {
    /// Parse a blob produced by [`pack_bundle_blob`]. Rejects wrong magic and
    /// any truncated or overlong payload rather than mis-keying assets.
    pub fn from_blob(blob: &[u8]) -> Result<Self> {
        let payload = blob
            .strip_prefix(BLOB_MAGIC.as_slice())
            .context("not a wildside bundle blob (bad magic)")?;
        let (count, mut payload) = read_u32(payload).context("bundle blob missing entry count")?;
        let mut files = InMemoryAssetSource::new();
        for index in 0..count {
            let (path, data, rest) = read_entry(payload)
                .with_context(|| format!("bundle blob entry {index}/{count} is malformed"))?;
            files.insert(PathBuf::from(path), data.to_vec());
            payload = rest;
        }
        ensure!(
            payload.is_empty(),
            "bundle blob has {} trailing bytes after {count} entries",
            payload.len()
        );
        Ok(Self { files })
    }

    /// Number of assets in the bundle.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the bundle holds no assets. Unreachable via [`from_blob`] of a
    /// [`pack_bundle_blob`] artefact (packing an empty root fails), but keeps
    /// the `len`/`is_empty` pairing honest.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl AssetSource for BundleAssetSource {
    fn read(&self, rel: &Path) -> Result<Vec<u8>> {
        self.files.read(rel)
    }

    fn exists(&self, rel: &Path) -> bool {
        self.files.exists(rel)
    }
}

fn read_u32(bytes: &[u8]) -> Result<(u32, &[u8])> {
    ensure!(bytes.len() >= 4, "truncated length field");
    let (len, rest) = bytes.split_at(4);
    Ok((u32::from_le_bytes(len.try_into().expect("4 bytes")), rest))
}

fn read_entry(bytes: &[u8]) -> Result<(&str, &[u8], &[u8])> {
    let (path_len, rest) = read_u32(bytes)?;
    ensure!(
        rest.len() >= path_len as usize,
        "truncated path (want {path_len} bytes, have {})",
        rest.len()
    );
    let (path, rest) = rest.split_at(path_len as usize);
    let path = std::str::from_utf8(path).context("asset path is not UTF-8")?;
    let (data_len, rest) = read_u32(rest)?;
    ensure!(
        rest.len() >= data_len as usize,
        "truncated data for {path} (want {data_len} bytes, have {})",
        rest.len()
    );
    let (data, rest) = rest.split_at(data_len as usize);
    Ok((path, data, rest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wildside_rom::verify_canonical_rom_bytes;

    fn packed_fixture() -> (tempfile::TempDir, Vec<u8>) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("scenes/artcomp")).expect("mkdir");
        std::fs::create_dir_all(root.join("level")).expect("mkdir");
        std::fs::write(root.join("asset-manifest.json"), b"{}").expect("write");
        std::fs::write(root.join("scenes/artcomp/Sega_Logo.bin"), b"sega").expect("write");
        std::fs::write(root.join("level/platform_scripts.asm"), b"; asm").expect("write");
        let blob = pack_bundle_blob(root).expect("pack");
        (dir, blob)
    }

    fn canonical_generated_fixture() -> Option<(PathBuf, CanonicalKidRom)> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?.parent()?;
        let root = repo_root
            .join(".cache/generated")
            .join(wildside_rom::known_sha1());
        let rom_path = repo_root.join(".cache/upstream/kid-chameleon-disasm/kid.bin");
        if !root.join("asset-manifest.json").is_file() || !rom_path.is_file() {
            eprintln!("skipping runtime extractor parity: canonical generated cache is absent");
            return None;
        }
        let bytes = std::fs::read(rom_path).ok()?;
        let rom = verify_canonical_rom_bytes(&bytes).expect("cached ROM should be canonical");
        Some((root, rom))
    }

    #[test]
    fn round_trips_bundle_directory_byte_for_byte() {
        let (_dir, blob) = packed_fixture();
        let source = BundleAssetSource::from_blob(&blob).expect("parse");

        assert_eq!(source.len(), 3);
        assert!(!source.is_empty());
        for (rel, bytes) in [
            ("asset-manifest.json", b"{}".as_slice()),
            ("scenes/artcomp/Sega_Logo.bin", b"sega".as_slice()),
            ("level/platform_scripts.asm", b"; asm".as_slice()),
        ] {
            let rel = Path::new(rel);
            assert!(source.exists(rel), "missing {}", rel.display());
            assert_eq!(source.read(rel).expect("read"), bytes);
        }
        assert!(!source.exists(Path::new("scenes/artcomp/missing.bin")));
        assert!(
            source
                .read(Path::new("scenes/artcomp/missing.bin"))
                .is_err()
        );
    }

    #[test]
    fn packing_is_deterministic() {
        let (dir, blob) = packed_fixture();
        assert_eq!(blob, pack_bundle_blob(dir.path()).expect("repack"));
    }

    #[test]
    fn selected_packing_omits_unlisted_files_and_stays_deterministic() {
        let (dir, _blob) = packed_fixture();
        let paths = vec![
            PathBuf::from("scenes/artcomp/Sega_Logo.bin"),
            PathBuf::from("asset-manifest.json"),
        ];

        let first = pack_selected_paths(dir.path(), paths.clone()).expect("pack selected");
        let second = pack_selected_paths(dir.path(), paths).expect("repack selected");
        let source = BundleAssetSource::from_blob(&first).expect("parse");

        assert_eq!(first, second);
        assert_eq!(source.len(), 2);
        assert!(source.exists(Path::new("asset-manifest.json")));
        assert!(source.exists(Path::new("scenes/artcomp/Sega_Logo.bin")));
        assert!(!source.exists(Path::new("level/platform_scripts.asm")));
    }

    #[test]
    fn canonical_rom_extractor_matches_native_runtime_pack_byte_for_byte() {
        let Some((root, rom)) = canonical_generated_fixture() else {
            return;
        };
        let (recipe, manifest) =
            build_runtime_extractor_inputs(&root, &rom).expect("derive extractor inputs");
        if std::env::var_os("WILDSIDE_UPDATE_RUNTIME_EXTRACTOR").is_some() {
            std::fs::write(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime-rom-assets.txt"),
                recipe,
            )
            .expect("write ROM extraction recipe");
            std::fs::write(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime-manifest.json"),
                manifest,
            )
            .expect("write runtime manifest");
            return;
        }

        let extracted = extract_runtime_bundle(&rom).expect("extract from canonical ROM");
        let packed = pack_runtime_bundle_blob(&root).expect("pack generated runtime bundle");
        let first_difference = extracted
            .iter()
            .zip(&packed)
            .position(|(extracted, packed)| extracted != packed);
        assert!(
            extracted == packed,
            "runtime bundles differ: extracted={} bytes, packed={} bytes, first difference={first_difference:?}",
            extracted.len(),
            packed.len()
        );
    }

    #[test]
    fn packing_an_empty_root_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(pack_bundle_blob(dir.path()).is_err());
    }

    #[test]
    fn rejects_bad_magic_truncation_and_trailing_bytes() {
        let (_dir, blob) = packed_fixture();

        assert!(BundleAssetSource::from_blob(b"NOTBLOB1rest").is_err());
        assert!(BundleAssetSource::from_blob(&blob[..blob.len() - 1]).is_err());

        let mut trailing = blob.clone();
        trailing.push(0);
        assert!(BundleAssetSource::from_blob(&trailing).is_err());
    }

    #[test]
    fn entry_layout_is_length_prefixed_little_endian() {
        // One entry: path "a" (1 byte), data "xy" (2 bytes). Guards the wire
        // format itself so a byte-offset regression cannot round-trip cleanly.
        let mut blob = Vec::new();
        blob.extend_from_slice(b"WSBNDL01");
        blob.extend_from_slice(&1u32.to_le_bytes());
        blob.extend_from_slice(&1u32.to_le_bytes());
        blob.push(b'a');
        blob.extend_from_slice(&2u32.to_le_bytes());
        blob.extend_from_slice(b"xy");

        let source = BundleAssetSource::from_blob(&blob).expect("parse");
        assert_eq!(source.len(), 1);
        assert_eq!(source.read(Path::new("a")).expect("read"), b"xy");
    }

    #[test]
    fn zero_entry_blob_preserves_len_empty_contract() {
        let mut blob = Vec::new();
        blob.extend_from_slice(b"WSBNDL01");
        blob.extend_from_slice(&0u32.to_le_bytes());

        let source = BundleAssetSource::from_blob(&blob).expect("parse empty bundle");
        assert_eq!(source.len(), 0);
        assert!(source.is_empty());
        assert!(source.read(Path::new("asset-manifest.json")).is_err());
    }
}
