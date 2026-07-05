//! Platform-free web play session: bundle blob in, RGBA frames out.
//!
//! This is the functional core of the browser shell (#19). It owns the
//! runtime + level visuals and turns one input mask into one 320x224 frame per
//! [`step`](WebSession::step). No wasm-bindgen types here — the same code runs
//! natively, which is how the capture-frames parity test drives it.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use wildside_core::{FrameInput, LevelId, NativeRuntime, NativeTraceFrame};
use wildside_data::{AssetSource, BundleAssetSource, parse_manifest_with_source};
use wildside_frame::{
    FRAME_HEIGHT, FRAME_WIDTH, build_scene, render_scene, render_trace_with_vdp_latched_state,
};
use wildside_render::{NativeLevelVisuals, load_level_visuals_with_source};

/// RGB24 framebuffer length — the pure raster layer's native stride.
pub const FRAME_RGB_LEN: usize = (FRAME_WIDTH * FRAME_HEIGHT * 3) as usize;
/// RGBA framebuffer length — what the canvas consumes (alpha always 255).
pub const FRAME_RGBA_LEN: usize = (FRAME_WIDTH * FRAME_HEIGHT * 4) as usize;

/// One interactive play session over a fetched asset bundle.
pub struct WebSession {
    manifest: wildside_data::AssetManifest,
    source: Arc<dyn AssetSource>,
    runtime: NativeRuntime,
    visuals: NativeLevelVisuals,
    visuals_level: u8,
    trace: NativeTraceFrame,
    rgb: Vec<u8>,
    rgba: Vec<u8>,
}

impl WebSession {
    /// Boot a session from a packed bundle blob (`pack-web-bundle` output),
    /// warped straight into `level`. The Phase-1 shell skips the VDP-rendered
    /// attract screens entirely (see [`step`](Self::step)), so gameplay is the
    /// entry point.
    pub fn from_bundle_blob(blob: &[u8], level: u8) -> Result<Self> {
        let source = BundleAssetSource::from_blob(blob).context("failed to parse bundle blob")?;
        Self::from_source(Arc::new(source), level)
    }

    /// Boot from any [`AssetSource`] that holds a generated bundle (including
    /// `asset-manifest.json`). The parity test injects a native `FsAssetSource`
    /// here to prove blob-backed and fs-backed sessions render identically.
    pub fn from_source(source: Arc<dyn AssetSource>, level: u8) -> Result<Self> {
        let manifest_bytes = source
            .read(Path::new("asset-manifest.json"))
            .context("bundle has no asset-manifest.json")?;
        let manifest = parse_manifest_with_source(&manifest_bytes, &*source)?;

        // Mirrors ScriptRunner::run_inner (wildside-core scenario.rs) exactly:
        // the web session must trace frame-for-frame with run_script_with_warp
        // so native capture-frames output is the pixel oracle for this shell.
        let mut runtime = NativeRuntime::with_generated_level_layout_with_source(
            manifest.clone(),
            source.clone(),
        );
        runtime.set_recorded_title_card_handoff_delay(true);
        runtime.boot_to_level(LevelId(level));

        let visuals = load_level_visuals_with_source(&manifest, level, &*source)?;
        let trace = runtime.trace_frame();
        let mut session = Self {
            manifest,
            source,
            runtime,
            visuals,
            visuals_level: level,
            trace,
            rgb: vec![0; FRAME_RGB_LEN],
            rgba: vec![0; FRAME_RGBA_LEN],
        };
        session.render_current_frame();
        Ok(session)
    }

    /// Advance one frame with the raw controller byte (`FrameInput::from_mask`
    /// layout: Up/Down/Left/Right/B/C/A/Start = bits 0..7) and rasterise it.
    pub fn step(&mut self, mask: u8) {
        let previous_trace = self.trace.clone();
        self.trace = self.runtime.step_frame(FrameInput::from_mask(mask));

        if self.trace.current_level.0 != self.visuals_level {
            // Keep the previous level's visuals if the new level's are missing
            // from the bundle rather than killing the session mid-play.
            if let Ok(visuals) = load_level_visuals_with_source(
                &self.manifest,
                self.trace.current_level.0,
                &*self.source,
            ) {
                self.visuals = visuals;
                self.visuals_level = self.trace.current_level.0;
            }
        }

        self.render_frame_after_step(Some(&previous_trace));
    }

    fn render_current_frame(&mut self) {
        self.render_frame_after_step(None);
    }

    /// Whether the runtime is on a VDP-rendered shell screen (Sega/title/level
    /// card) rather than gameplay. Phase 1 renders these black (see
    /// [`step`](Self::step)); shells can surface a "press any key" hint —
    /// the ROM's title-card handoff waits for a button edge.
    pub fn on_shell_screen(&self) -> bool {
        self.runtime.vdp_state().is_some()
    }

    fn render_frame_after_step(&mut self, previous_trace: Option<&NativeTraceFrame>) {
        self.rgb.fill(0);
        if self.on_shell_screen() {
            // TODO(fidelity): VDP-rendered shell screens (Sega/title/level
            // card) need wildside-md's renderer, which drags in ymfm-sys (C++)
            // and is deferred to the web-audio phase (#19 wrinkle, #20). Hold
            // a black frame; gameplay frames never take this path.
        } else {
            let render_trace = render_trace_with_vdp_latched_state(&self.trace, previous_trace);
            let scene = build_scene(
                &render_trace,
                false,
                Some(&self.visuals),
                self.runtime.block_grid(),
            );
            render_scene(&scene, &mut self.rgb);
        }
        expand_rgb_to_rgba(&self.rgb, &mut self.rgba);
    }

    /// Latest frame as RGB24 — the raster layer's native output, byte-comparable
    /// against native `capture-frames` PNG pixel data.
    pub fn rgb_frame(&self) -> &[u8] {
        &self.rgb
    }

    /// Latest frame as RGBA (alpha 255) — what the canvas blits.
    pub fn rgba_frame(&self) -> &[u8] {
        &self.rgba
    }

    /// Latest engine trace (for shells that surface state, e.g. debug HUDs).
    pub fn trace(&self) -> &NativeTraceFrame {
        &self.trace
    }

    /// One-line state summary for shell diagnostics.
    pub fn debug_state(&self) -> String {
        format!(
            "mode={:?} level={:#04x} vdp={} grid={} rgb_nonzero={}",
            self.trace.game_mode,
            self.trace.current_level.0,
            self.runtime.vdp_state().is_some(),
            self.runtime.block_grid().is_some(),
            self.rgb.iter().any(|byte| *byte != 0),
        )
    }
}

/// Expand the pure layer's RGB24 into canvas RGBA with opaque alpha. The RGB24
/// stride stays the shared rasteriser's contract (see wildside-frame lib.rs);
/// the web shell widens at this blit boundary per the decision on #18.
fn expand_rgb_to_rgba(rgb: &[u8], rgba: &mut [u8]) {
    debug_assert_eq!(rgb.len() * 4, rgba.len() * 3);
    for (src, dst) in rgb.chunks_exact(3).zip(rgba.chunks_exact_mut(4)) {
        dst[..3].copy_from_slice(src);
        dst[3] = 255;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_rgb24_to_opaque_rgba() {
        let rgb = [1u8, 2, 3, 4, 5, 6];
        let mut rgba = [0u8; 8];
        expand_rgb_to_rgba(&rgb, &mut rgba);
        assert_eq!(rgba, [1, 2, 3, 255, 4, 5, 6, 255]);
    }
}
