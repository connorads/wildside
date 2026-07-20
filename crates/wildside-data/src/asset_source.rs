//! Read-only byte source for generated-bundle assets.
//!
//! The runtime read path (VDP loaders, level/block decoders, render visuals)
//! historically called `std::fs::read` directly against absolute paths under
//! `AssetManifest::output_root`. That couples pure runtime logic to the native
//! filesystem and breaks the web (WASM) build, which has no browser filesystem.
//!
//! [`AssetSource`] is the port that decouples those reads: callers resolve a
//! bundle-relative path and the adapter supplies the bytes. [`FsAssetSource`]
//! is the native adapter (byte-identical to the previous direct reads);
//! [`InMemoryAssetSource`] backs tests now and a fetched-bundle adapter later.

use anyhow::{Context, Result};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Read-only byte source for generated-bundle assets, resolved relative to a
/// bundle root (`AssetManifest::output_root`).
///
/// Lets the runtime read identical bytes from the filesystem (native), an
/// in-memory map (tests), or a fetched bundle (wasm). Paths passed to [`read`]
/// and [`exists`] are always bundle-relative (e.g. `scenes/artcomp/Sega_Logo.bin`),
/// never `output_root`-absolute.
///
/// [`read`]: AssetSource::read
/// [`exists`]: AssetSource::exists
///
/// `Debug` is a supertrait because `NativeRuntime` (`#[derive(Debug, Clone)]`)
/// stores an `Arc<dyn AssetSource>`; the derived `Debug` requires the trait
/// object to be `Debug`. `Send + Sync` mirror the `OracleBackend` port and are
/// required because the source rides on `NativeRuntime`, which crosses threads
/// in sweep/recording.
pub trait AssetSource: std::fmt::Debug + Send + Sync {
    /// Read the full bytes of the bundle-relative asset `rel`.
    fn read(&self, rel: &Path) -> Result<Vec<u8>>;

    /// Whether the bundle-relative asset `rel` exists as a readable file.
    ///
    /// Mirrors `Path::is_file` (not `Path::exists`) so a directory-named path
    /// is reported absent, exactly preserving the `.is_file()` guards that
    /// previously gated these reads.
    fn exists(&self, rel: &Path) -> bool;
}

/// Native [`AssetSource`] reading from the filesystem under `root`
/// (`AssetManifest::output_root`). Behaviour-preserving versus the previous
/// `output_root.join(rel)` + `std::fs::read` call sites.
#[derive(Debug, Clone)]
pub struct FsAssetSource {
    root: PathBuf,
}

impl FsAssetSource {
    /// Build a source rooted at `root` (a bundle `output_root`).
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl AssetSource for FsAssetSource {
    fn read(&self, rel: &Path) -> Result<Vec<u8>> {
        let path = self.root.join(rel);
        std::fs::read(&path).with_context(|| format!("failed to read asset {}", path.display()))
    }

    fn exists(&self, rel: &Path) -> bool {
        self.root.join(rel).is_file()
    }
}

/// In-memory [`AssetSource`] keyed by bundle-relative path. Backs the
/// determinism test now and a fetched-bundle (`BundleAssetSource`) adapter for
/// the web build later.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAssetSource {
    files: HashMap<PathBuf, Vec<u8>>,
}

impl InMemoryAssetSource {
    /// Build an empty source.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert the bytes for a bundle-relative path, replacing any prior entry.
    pub fn insert(&mut self, rel: impl Into<PathBuf>, bytes: Vec<u8>) {
        self.files.insert(rel.into(), bytes);
    }

    /// Number of stored assets.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the source holds no assets.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl AssetSource for InMemoryAssetSource {
    fn read(&self, rel: &Path) -> Result<Vec<u8>> {
        self.files
            .get(rel)
            .cloned()
            .with_context(|| format!("in-memory asset not found: {}", rel.display()))
    }

    fn exists(&self, rel: &Path) -> bool {
        self.files.contains_key(rel)
    }
}

/// [`AssetSource`] decorator that records every read/existence query.
///
/// The web asset audit uses this around a real bundle source to derive and
/// ratchet the minimal runtime catalogue. Recording is observational only:
/// results and errors come directly from the wrapped source.
#[derive(Debug, Clone)]
pub struct RecordingAssetSource {
    inner: Arc<dyn AssetSource>,
    requested: Arc<Mutex<BTreeSet<PathBuf>>>,
}

impl RecordingAssetSource {
    pub fn new(inner: Arc<dyn AssetSource>) -> Self {
        Self {
            inner,
            requested: Arc::new(Mutex::new(BTreeSet::new())),
        }
    }

    pub fn requested_paths(&self) -> BTreeSet<PathBuf> {
        self.requested
            .lock()
            .expect("recording asset source mutex poisoned")
            .clone()
    }

    fn record(&self, rel: &Path) {
        self.requested
            .lock()
            .expect("recording asset source mutex poisoned")
            .insert(rel.to_path_buf());
    }
}

impl AssetSource for RecordingAssetSource {
    fn read(&self, rel: &Path) -> Result<Vec<u8>> {
        self.record(rel);
        self.inner.read(rel)
    }

    fn exists(&self, rel: &Path) -> bool {
        self.record(rel);
        self.inner.exists(rel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_source_reads_and_reports_files_relative_to_root() {
        let dir = tempfile::tempdir().expect("tempdir");
        let nested = dir.path().join("scenes/palette");
        std::fs::create_dir_all(&nested).expect("create nested");
        std::fs::write(nested.join("title.bin"), b"abcd").expect("write asset");

        let source = FsAssetSource::new(dir.path().to_path_buf());
        let rel = Path::new("scenes/palette/title.bin");
        assert!(source.exists(rel));
        assert_eq!(source.read(rel).expect("read asset"), b"abcd");

        // A missing file is absent and read fails.
        let missing = Path::new("scenes/palette/missing.bin");
        assert!(!source.exists(missing));
        assert!(source.read(missing).is_err());

        // `exists` mirrors `is_file`: a directory-named path is absent.
        assert!(!source.exists(Path::new("scenes/palette")));
    }

    #[test]
    fn in_memory_source_keys_by_relative_path() {
        let mut source = InMemoryAssetSource::new();
        assert!(source.is_empty());
        source.insert(
            PathBuf::from("level/platform_scripts.asm"),
            b"data".to_vec(),
        );
        assert_eq!(source.len(), 1);
        assert!(!source.is_empty());

        let rel = Path::new("level/platform_scripts.asm");
        assert!(source.exists(rel));
        assert_eq!(source.read(rel).expect("read asset"), b"data");
        assert!(!source.exists(Path::new("level/other.asm")));
        assert!(source.read(Path::new("level/other.asm")).is_err());

        source.insert(PathBuf::from("level/other.asm"), b"other".to_vec());
        assert_eq!(source.len(), 2);
        assert!(source.exists(Path::new("level/other.asm")));

        source.insert(
            PathBuf::from("level/platform_scripts.asm"),
            b"replacement".to_vec(),
        );
        assert_eq!(source.len(), 2);
        assert_eq!(
            source.read(rel).expect("read replaced asset"),
            b"replacement"
        );
    }

    #[test]
    fn recording_source_observes_reads_and_exists_without_changing_results() {
        let mut files = InMemoryAssetSource::new();
        files.insert("present.bin", b"bytes".to_vec());
        let source = RecordingAssetSource::new(Arc::new(files));

        assert!(source.exists(Path::new("present.bin")));
        assert!(!source.exists(Path::new("missing.bin")));
        assert_eq!(
            source.read(Path::new("present.bin")).expect("read"),
            b"bytes"
        );
        assert!(source.read(Path::new("missing.bin")).is_err());
        assert_eq!(
            source.requested_paths(),
            BTreeSet::from([PathBuf::from("missing.bin"), PathBuf::from("present.bin"),])
        );
    }
}
