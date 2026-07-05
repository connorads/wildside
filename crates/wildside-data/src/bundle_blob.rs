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

use crate::{AssetSource, InMemoryAssetSource};
use anyhow::{Context, Result, bail, ensure};
use std::path::{Path, PathBuf};

const BLOB_MAGIC: &[u8; 8] = b"WSBNDL01";

/// Serialise every file under `root` (a bundle `output_root`) into a single
/// blob. Native-only in practice (walks the filesystem); the inverse of
/// [`BundleAssetSource::from_blob`].
pub fn pack_bundle_blob(root: &Path) -> Result<Vec<u8>> {
    let mut paths = Vec::new();
    collect_files(root, root, &mut paths)?;
    ensure!(
        !paths.is_empty(),
        "bundle root {} contains no files to pack",
        root.display()
    );
    // Sorted by relative path for deterministic output.
    paths.sort();

    let mut blob = Vec::new();
    blob.extend_from_slice(BLOB_MAGIC);
    blob.extend_from_slice(&(paths.len() as u32).to_le_bytes());
    for rel in &paths {
        let key = blob_key(rel)?;
        let bytes = std::fs::read(root.join(rel))
            .with_context(|| format!("failed to read {}", root.join(rel).display()))?;
        blob.extend_from_slice(&(key.len() as u32).to_le_bytes());
        blob.extend_from_slice(key.as_bytes());
        blob.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        blob.extend_from_slice(&bytes);
    }
    Ok(blob)
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
