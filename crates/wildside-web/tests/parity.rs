//! Web-session parity over the real generated bundle (#19/#21).
//!
//! Skips (returns early) when `.cache/generated` is absent, matching the
//! cache-backed tests elsewhere in the workspace.

use std::path::PathBuf;
use std::sync::Arc;

use wildside_data::{FsAssetSource, load_manifest, manifest_path, pack_bundle_blob};
use wildside_web::WebSession;

/// Locate a cached generated bundle root (any ROM sha1), honouring the
/// `KID_REPO_ROOT` override the other cache-backed tests use.
fn cached_generated_root() -> Option<PathBuf> {
    let repo_root = match std::env::var("KID_REPO_ROOT") {
        Ok(root) if !root.is_empty() => PathBuf::from(root),
        _ => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("wildside-web crate should live under <repo>/crates/wildside-web")
            .to_path_buf(),
    };
    let generated = repo_root.join(".cache/generated");
    for entry in std::fs::read_dir(generated).ok()?.flatten() {
        let path = entry.path();
        if manifest_path(&path).is_file() {
            return Some(path);
        }
    }
    eprintln!("skipping cache-backed web parity test: no generated bundle in .cache/generated");
    None
}

/// Walk-right-and-jump input, exercising scrolling, sprites, and block
/// interactions rather than a static idle frame.
fn scripted_mask(frame: usize) -> u8 {
    let jump = if frame % 60 >= 45 { 0x10 } else { 0 };
    0x08 | jump
}

/// A blob-backed session renders byte-identically to an fs-backed session for
/// every frame of a scripted run — the delivery format drops no bytes
/// (`pack_bundle_blob` / `BundleAssetSource` round-trip over the real bundle).
#[test]
fn blob_session_renders_identically_to_fs_session() {
    let Some(root) = cached_generated_root() else {
        return;
    };
    let blob = pack_bundle_blob(&root).expect("pack bundle blob");
    let mut blob_session = WebSession::from_bundle_blob(&blob, 0).expect("blob session");
    let mut fs_session =
        WebSession::from_source(Arc::new(FsAssetSource::new(root)), 0).expect("fs session");

    assert_eq!(
        blob_session.rgb_frame(),
        fs_session.rgb_frame(),
        "boot frame diverged"
    );
    for frame in 0..180 {
        let mask = scripted_mask(frame);
        blob_session.step(mask);
        fs_session.step(mask);
        assert!(
            blob_session.rgb_frame() == fs_session.rgb_frame(),
            "rendered frame {frame} diverged between blob-backed and fs-backed sessions"
        );
    }
}

/// The session's stepping is trace-for-trace identical to
/// `run_script_with_warp` — the engine path native `capture-frames` renders
/// from — so capture-frames PNGs are a valid pixel oracle for the web shell.
#[test]
fn session_traces_match_script_runner() {
    let Some(root) = cached_generated_root() else {
        return;
    };
    let manifest = load_manifest(&manifest_path(&root)).expect("load manifest");
    let script: Vec<u8> = (0..120).map(scripted_mask).collect();
    let expected = wildside_core::run_script_with_warp(manifest, &script, script.len(), Some(0));

    let mut session =
        WebSession::from_source(Arc::new(FsAssetSource::new(root)), 0).expect("session");
    for (frame, want) in expected.iter().enumerate() {
        session.step(script[frame]);
        assert!(
            session.trace() == want,
            "session trace diverged from ScriptRunner at frame {frame}"
        );
    }
}
