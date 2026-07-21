use super::*;

/// Cache key for oracle runs: `(oracle_name, rom_sha1, script_sha1)`.
/// Oracle output is deterministic for a given backend + ROM + input script
/// **and helper binary**, so the cache key folds the backend's cache version
/// into `script_sha1` to invalidate stale entries when the helper's boot,
/// warp, or capture logic changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OracleCacheKey {
    pub(crate) rom_sha1: String,
    pub(crate) script_sha1: String,
    pub(crate) oracle_name: String,
}

impl OracleCacheKey {
    #[cfg(test)]
    pub(crate) fn new(rom_bytes: &[u8], script: &[u8]) -> Self {
        Self {
            rom_sha1: sha1_hex(rom_bytes),
            script_sha1: sha1_hex(script),
            oracle_name: "jgenesis".to_string(),
        }
    }

    /// Build a cache key from a DynamicScenario. Includes `warp_to_level`
    /// in the hash so that different target levels produce distinct keys,
    /// even when the post-warp script is identical. `cache_version` comes
    /// from the backend and changes whenever the helper's behaviour (e.g.
    /// boot flow, warp timing) changes — mixing it into `script_sha1`
    /// naturally unreaches stale entries without any explicit deletion.
    pub(crate) fn from_scenario(
        rom_bytes: &[u8],
        scenario: &scenarios::DynamicScenario,
        oracle_name: &str,
        cache_version: &str,
    ) -> Self {
        let scenario_json = serde_json::to_vec(scenario).unwrap_or_default();
        let mut script_bytes = scenario_json;
        script_bytes.push(0);
        script_bytes.extend_from_slice(cache_version.as_bytes());
        Self {
            rom_sha1: sha1_hex(rom_bytes),
            script_sha1: sha1_hex(&script_bytes),
            oracle_name: oracle_name.to_string(),
        }
    }

    pub(crate) fn cache_dir(&self, repo_root: &Path) -> PathBuf {
        repo_root
            .join(".cache/oracle-cache")
            .join(&self.oracle_name)
            .join(&self.rom_sha1)
            .join(&self.script_sha1)
    }
}

/// Check if a cached oracle manifest exists for this key.
pub(crate) fn load_cached_oracle(key: &OracleCacheKey, repo_root: &Path) -> Option<OracleManifest> {
    let dir = key.cache_dir(repo_root);
    let manifest_path = dir.join("manifest.json");
    let json = fs::read_to_string(&manifest_path).ok()?;
    let manifest: OracleManifest = serde_json::from_str(&json).ok()?;
    // Verify all referenced frame PNGs still exist.
    for frame in &manifest.frames {
        if !frame.png_path.exists() {
            return None;
        }
    }
    Some(manifest)
}

/// Persist an oracle manifest and its frame PNGs into the cache.
/// Frame PNGs are copied from the original out_dir into the cache dir.
pub(crate) fn store_oracle_cache(
    key: &OracleCacheKey,
    repo_root: &Path,
    manifest: &OracleManifest,
    _source_dir: &Path,
) -> Result<()> {
    let dir = key.cache_dir(repo_root);
    let frames_dir = dir.join("frames");
    fs::create_dir_all(&frames_dir)
        .with_context(|| format!("failed to create {}", frames_dir.display()))?;

    // Copy frame PNGs into cache and rewrite paths.
    let mut cached_manifest = manifest.clone();
    for frame in &mut cached_manifest.frames {
        let filename = frame
            .png_path
            .file_name()
            .ok_or_else(|| anyhow!("frame PNG has no filename"))?;
        let cached_path = frames_dir.join(filename);
        if frame.png_path.exists() && frame.png_path != cached_path {
            fs::copy(&frame.png_path, &cached_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    frame.png_path.display(),
                    cached_path.display()
                )
            })?;
        }
        frame.png_path = cached_path;
    }

    // Copy audio into cache.
    if manifest.audio_path.exists() {
        let cached_audio = dir.join("audio.wav");
        if manifest.audio_path != cached_audio {
            fs::copy(&manifest.audio_path, &cached_audio).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    manifest.audio_path.display(),
                    cached_audio.display()
                )
            })?;
        }
        cached_manifest.audio_path = cached_audio;
    }

    let manifest_json = serde_json::to_vec_pretty(&cached_manifest)?;
    fs::write(dir.join("manifest.json"), manifest_json)
        .with_context(|| format!("failed to write manifest to {}", dir.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::OracleFrame;
    use tempfile::TempDir;

    fn test_manifest(frame_path: PathBuf, audio_path: PathBuf) -> OracleManifest {
        OracleManifest {
            sample_rate: 44_100,
            audio_path,
            frames: vec![OracleFrame {
                label: "frame_0".to_string(),
                frame_index: 0,
                png_path: frame_path,
                sha1: "fake-frame-sha1".to_string(),
            }],
            states: Vec::new(),
            frame_crcs: vec![0x1234_5678],
            frame_rng_states: Vec::new(),
            gems_frame_signatures: Vec::new(),
            gems_frame_audio_ram: Vec::new(),
            memory_watches: Vec::new(),
            vdp_pixel_samples: Vec::new(),
            vdp_fifo_commits: Vec::new(),
            vdp_fifo_enqueues: Vec::new(),
            m68k_pc_trace: Vec::new(),
            settle_trace: Vec::new(),
        }
    }

    #[test]
    fn cache_key_deterministic_for_same_inputs() {
        let rom = b"fake rom bytes";
        let script = b"fake script";
        let key1 = OracleCacheKey::new(rom, script);
        let key2 = OracleCacheKey::new(rom, script);
        assert_eq!(key1, key2);
        assert_eq!(key1.rom_sha1, key2.rom_sha1);
        assert_eq!(key1.script_sha1, key2.script_sha1);
    }

    #[test]
    fn cache_key_differs_for_different_script() {
        let rom = b"fake rom bytes";
        let key1 = OracleCacheKey::new(rom, b"script_a");
        let key2 = OracleCacheKey::new(rom, b"script_b");
        assert_ne!(key1.script_sha1, key2.script_sha1);
    }

    #[test]
    fn cache_key_differs_when_cache_version_changes() {
        let rom = b"fake rom bytes";
        let scenario = scenarios::DynamicScenario {
            slug: "t".into(),
            script: vec![0; 1],
            frame_count: 1,
            checkpoints: Vec::new(),
            start_position: None,
            start_velocity: None,
            camera_position: None,
            playtest_form: None,
            initial_diamond_count: None,
            initial_lives: None,
            warp_to_level: None,
            settle_oracle_warp: false,
        };
        let key_v1 = OracleCacheKey::from_scenario(rom, &scenario, "jgenesis", "v1");
        let key_v2 = OracleCacheKey::from_scenario(rom, &scenario, "jgenesis", "v2");
        assert_ne!(
            key_v1.script_sha1, key_v2.script_sha1,
            "cache_version must contribute to script_sha1 so helper changes invalidate stale entries"
        );
    }

    #[test]
    fn cache_dir_uses_sha1_components() {
        let key = OracleCacheKey::new(b"rom", b"script");
        let dir = key.cache_dir(Path::new("/repo"));
        assert!(dir.to_string_lossy().contains(".cache/oracle-cache"));
        assert!(dir.to_string_lossy().contains(&key.rom_sha1));
        assert!(dir.to_string_lossy().contains(&key.script_sha1));
    }

    #[test]
    fn load_cached_oracle_returns_none_for_missing_dir() {
        let key = OracleCacheKey::new(b"nonexistent", b"nonexistent");
        let result = load_cached_oracle(&key, Path::new("/tmp/not-a-real-dir"));
        assert!(result.is_none());
    }

    #[test]
    fn store_oracle_cache_copies_frames_audio_and_rewrites_manifest_paths() {
        let dir = TempDir::new().expect("temp");
        let source_dir = dir.path().join("source");
        fs::create_dir_all(&source_dir).expect("source dir");
        let source_frame = source_dir.join("frame_0000.png");
        let source_audio = source_dir.join("audio-source.wav");
        fs::write(&source_frame, b"fake png bytes").expect("frame");
        fs::write(&source_audio, b"fake wav bytes").expect("audio");

        let key = OracleCacheKey::new(b"rom", b"script");
        let manifest = test_manifest(source_frame.clone(), source_audio.clone());
        store_oracle_cache(&key, dir.path(), &manifest, &source_dir).expect("store");

        let cached = load_cached_oracle(&key, dir.path()).expect("cached manifest");
        let expected_dir = key.cache_dir(dir.path());
        let expected_frame = expected_dir.join("frames/frame_0000.png");
        let expected_audio = expected_dir.join("audio.wav");
        assert_eq!(cached.frames[0].png_path, expected_frame);
        assert_eq!(cached.audio_path, expected_audio);
        assert_eq!(
            fs::read(&cached.frames[0].png_path).expect("cached frame"),
            b"fake png bytes"
        );
        assert_eq!(
            fs::read(&cached.audio_path).expect("cached audio"),
            b"fake wav bytes"
        );
        assert_eq!(cached.sample_rate, 44_100);
        assert_eq!(cached.frame_crcs, vec![0x1234_5678]);
    }

    #[test]
    fn store_oracle_cache_accepts_manifest_that_already_points_at_cached_frame() {
        let dir = TempDir::new().expect("temp");
        let source_dir = dir.path().join("source");
        fs::create_dir_all(&source_dir).expect("source dir");
        let source_frame = source_dir.join("frame_0000.png");
        let source_audio = source_dir.join("audio-source.wav");
        fs::write(&source_frame, b"fake png bytes").expect("frame");
        fs::write(&source_audio, b"fake wav bytes").expect("audio");

        let key = OracleCacheKey::new(b"rom", b"script");
        let manifest = test_manifest(source_frame, source_audio);
        store_oracle_cache(&key, dir.path(), &manifest, &source_dir).expect("store");
        let cached = load_cached_oracle(&key, dir.path()).expect("cached manifest");

        store_oracle_cache(&key, dir.path(), &cached, &source_dir).expect("store cached manifest");
        assert_eq!(
            fs::read(key.cache_dir(dir.path()).join("frames/frame_0000.png"))
                .expect("cached frame"),
            b"fake png bytes"
        );
    }

    #[test]
    fn load_cached_oracle_rejects_manifest_when_cached_frame_is_missing() {
        let dir = TempDir::new().expect("temp");
        let source_dir = dir.path().join("source");
        fs::create_dir_all(&source_dir).expect("source dir");
        let source_frame = source_dir.join("frame_0000.png");
        let source_audio = source_dir.join("audio-source.wav");
        fs::write(&source_frame, b"fake png bytes").expect("frame");
        fs::write(&source_audio, b"fake wav bytes").expect("audio");

        let key = OracleCacheKey::new(b"rom", b"script");
        let manifest = test_manifest(source_frame, source_audio);
        store_oracle_cache(&key, dir.path(), &manifest, &source_dir).expect("store");

        let cached_frame = key.cache_dir(dir.path()).join("frames/frame_0000.png");
        fs::remove_file(cached_frame).expect("remove cached frame");

        assert!(load_cached_oracle(&key, dir.path()).is_none());
    }
}
