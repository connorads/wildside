//! Determinism proof for the `AssetSource` port (#17).
//!
//! Every generated-bundle byte the runtime reads at boot and level-load time
//! now flows through [`AssetSource`](wildside_data::AssetSource). This test
//! drives two runtimes off the **same** manifest — one through the default
//! native `FsAssetSource`, one through an [`InMemoryAssetSource`] populated by
//! walking `output_root` — and asserts both trace channels of
//! [`ScriptRunner::run_with_vdp`] match frame-for-frame.

use super::*;
use std::path::{Path, PathBuf};
use wildside_data::InMemoryAssetSource;

/// Walk `root` recursively, returning each file's bytes keyed by its path
/// **relative to** `root` — the same bundle-relative keys the runtime builds.
fn in_memory_source_from_bundle(root: &Path) -> InMemoryAssetSource {
    let mut source = InMemoryAssetSource::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let (Ok(bytes), Ok(rel)) = (std::fs::read(&path), path.strip_prefix(root)) {
                source.insert(rel.to_path_buf(), bytes);
            }
        }
    }
    source
}

/// Assert two per-frame channels are equal frame-for-frame without dumping the
/// (multi-kilobyte) `VdpState` on mismatch — just the diverging frame index.
fn assert_channels_match<T: PartialEq>(label: &str, fs: &[T], mem: &[T]) {
    assert_eq!(
        fs.len(),
        mem.len(),
        "{label} channel length differs: fs={}, in-memory={}",
        fs.len(),
        mem.len()
    );
    for (frame, (fs_frame, mem_frame)) in fs.iter().zip(mem).enumerate() {
        assert!(
            fs_frame == mem_frame,
            "{label} channel diverges at frame {frame}: the in-memory AssetSource \
             produced different bytes than fs, so a runtime read is not byte-faithful \
             through the port"
        );
    }
}

/// The injected `InMemoryAssetSource` reproduces the fs trace exactly across
/// both `run_with_vdp` channels, proving every runtime read goes through the
/// port and is byte-identical.
///
/// Comparing `Vec<NativeTraceFrame>` alone would be vacuous: the four VDP
/// loaders run with `.ok()` (lib.rs `new_with_source`) and feed only
/// `vdp_state`, which surfaces on the *second* `run_with_vdp` channel, not in
/// `NativeTraceFrame`. So the test compares BOTH channels, drives a gameplay
/// path (warp) so level/block reads flow into `NativeTraceFrame`, and uses the
/// in-memory source as the SOLE backend — any read that escapes the port hits a
/// missing key and visibly diverges the VDP channel rather than passing as
/// `None == None`.
#[test]
fn in_memory_asset_source_matches_fs_trace() {
    let Some(manifest) = cached_generated_manifest() else {
        return;
    };

    let in_memory = in_memory_source_from_bundle(&manifest.output_root);

    // Non-vacuity guard: the four VDP-loader assets plus the platform-script
    // table must be present, or the parity comparison below would be vacuous.
    for rel in [
        "scenes/artcomp/Sega_Logo.bin",      // load_sega_screen_data
        "scenes/mapeni/intro1_wildside.bin", // load_intro_sub_1b850_data
        "scenes/mapeni/2F7D0.bin",           // load_title_screen_data
        "level/platform_scripts.asm",        // load_platform_script_table
    ] {
        assert!(
            in_memory.exists(Path::new(rel)),
            "in-memory source must contain {rel}; otherwise the parity check is vacuous"
        );
    }
    assert!(
        in_memory.len() > 100,
        "bundle walk found only {} assets — expected a full generated bundle",
        in_memory.len()
    );

    let source: Arc<dyn AssetSource> = Arc::new(in_memory);

    // Phase 1 — boot/VDP. Empty input runs Sega -> intro sequences -> title;
    // the VDP loaders' decoded output surfaces only on the VDP channel, so a
    // missing/mis-keyed asset diverges here (Some vs None) instead of passing.
    let boot_script = [0u8; 240];
    let (fs_boot_trace, fs_boot_vdp) =
        ScriptRunner::new(manifest.clone()).run_with_vdp(&boot_script, boot_script.len());
    let (mem_boot_trace, mem_boot_vdp) = ScriptRunner::new(manifest.clone())
        .with_asset_source(source.clone())
        .run_with_vdp(&boot_script, boot_script.len());
    assert!(
        fs_boot_vdp.iter().any(Option::is_some),
        "boot VDP channel should carry decoded loader output (Sega/intro/title)"
    );
    assert_channels_match("boot trace", &fs_boot_trace, &mem_boot_trace);
    assert_channels_match("boot VDP", &fs_boot_vdp, &mem_boot_vdp);

    // Phase 2 — gameplay. Warping into a level routes the level/block/theme
    // reads into NativeTraceFrame fields (floor snap, platforms), so the trace
    // channel proves those reads are byte-faithful through the port.
    let play_script = [0u8; 90];
    let (fs_play_trace, fs_play_vdp) = ScriptRunner::new(manifest.clone())
        .warp_to_level(0x01)
        .run_with_vdp(&play_script, play_script.len());
    let (mem_play_trace, mem_play_vdp) = ScriptRunner::new(manifest.clone())
        .warp_to_level(0x01)
        .with_asset_source(source.clone())
        .run_with_vdp(&play_script, play_script.len());
    assert_channels_match("gameplay trace", &fs_play_trace, &mem_play_trace);
    assert_channels_match("gameplay VDP", &fs_play_vdp, &mem_play_vdp);
}
