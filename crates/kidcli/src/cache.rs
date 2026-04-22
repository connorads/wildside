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
            warp_to_level: None,
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
}
