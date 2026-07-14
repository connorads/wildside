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
    AudioSampleClock, FRAME_HEIGHT, FRAME_WIDTH, NativeAudioDirector, build_scene,
    load_gems_audio_engine, render_scene, render_trace_with_vdp_latched_state_and_resident,
    render_vdp_trace_overlay,
};
use wildside_md::GemsAudioEngine;
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
    audio: Option<SessionAudio>,
}

/// GEMS audio state once [`WebSession::enable_audio`] has run: the engine, the
/// shared per-frame director, and the device-rate share clock. `samples` holds
/// the interleaved-stereo f32 output of the most recent `step`.
struct SessionAudio {
    gems: GemsAudioEngine,
    director: NativeAudioDirector,
    clock: AudioSampleClock,
    output_rate: u32,
    samples: Vec<f32>,
}

impl WebSession {
    /// Boot a session from a packed bundle blob (`pack-web-bundle` output).
    /// `None` boots the real attract flow (Sega logo → intro → title menu);
    /// `Some(level)` warps straight into that level.
    pub fn from_bundle_blob(blob: &[u8], level: Option<u8>) -> Result<Self> {
        let source = BundleAssetSource::from_blob(blob).context("failed to parse bundle blob")?;
        Self::from_source(Arc::new(source), level)
    }

    /// Boot from any [`AssetSource`] that holds a generated bundle (including
    /// `asset-manifest.json`). The parity test injects a native `FsAssetSource`
    /// here to prove blob-backed and fs-backed sessions render identically.
    pub fn from_source(source: Arc<dyn AssetSource>, level: Option<u8>) -> Result<Self> {
        let manifest_bytes = source
            .read(Path::new("asset-manifest.json"))
            .context("bundle has no asset-manifest.json")?;
        let manifest = parse_manifest_with_source(&manifest_bytes, &*source)?;

        // Mirrors ScriptRunner::run_inner (wildside-core scenario.rs) exactly:
        // the web session must trace frame-for-frame with run_script_with_warp
        // (warped or not) so native capture-frames output is the pixel oracle
        // for this shell.
        let mut runtime = NativeRuntime::with_generated_level_layout_with_source(
            manifest.clone(),
            source.clone(),
        );
        runtime.set_recorded_title_card_handoff_delay(true);
        if let Some(level) = level {
            runtime.boot_to_level(LevelId(level));
        }

        // Level 0 is the boot level when the attract flow runs; `step` reloads
        // visuals when the traced level changes (same default as
        // capture_scenario_frames in wildside-cli sweep/frames.rs).
        let visuals_level = level.unwrap_or(0);
        let visuals = load_level_visuals_with_source(&manifest, visuals_level, &*source)?;
        let trace = runtime.trace_frame();
        let mut session = Self {
            manifest,
            source,
            runtime,
            visuals,
            visuals_level,
            trace,
            rgb: vec![0; FRAME_RGB_LEN],
            rgba: vec![0; FRAME_RGBA_LEN],
            audio: None,
        };
        session.render_current_frame();
        Ok(session)
    }

    /// Enable GEMS audio, generating interleaved-stereo f32 at `output_rate`
    /// Hz (the AudioContext sample rate — browsers refuse audio before a user
    /// gesture, so this runs later than boot). The director starts cold and
    /// derives BGM/SFX from the next stepped trace, exactly like the native
    /// batch renderer over the same frames.
    pub fn enable_audio(&mut self, output_rate: u32) -> Result<()> {
        let gems =
            load_gems_audio_engine(&*self.source).context("bundle has no GEMS sound banks")?;
        self.audio = Some(SessionAudio {
            gems,
            director: NativeAudioDirector::new(),
            clock: AudioSampleClock::new(output_rate),
            output_rate,
            samples: Vec::new(),
        });
        Ok(())
    }

    /// Advance one frame with the raw controller byte (`FrameInput::from_mask`
    /// layout: Up/Down/Left/Right/B/C/A/Start = bits 0..7) and rasterise it.
    pub fn step(&mut self, mask: u8) {
        let previous_trace = self.trace.clone();
        self.trace = self.runtime.step_frame(FrameInput::from_mask(mask));

        if let Some(audio) = self.audio.as_mut() {
            audio
                .director
                .step(&self.manifest, &self.trace, &mut audio.gems);
            let sample_count = audio.clock.next_frame_samples();
            audio.samples = audio
                .gems
                .generate_samples_at_rate(audio.output_rate, sample_count);
        }

        let render_level = self.runtime.source_port_render_level().0;
        if render_level != self.visuals_level {
            // Keep the previous level's visuals if the new level's are missing
            // from the bundle rather than killing the session mid-play.
            if let Ok(visuals) =
                load_level_visuals_with_source(&self.manifest, render_level, &*self.source)
            {
                self.visuals = visuals;
                self.visuals_level = render_level;
            }
        }

        self.render_frame_after_step(Some(&previous_trace));
    }

    fn render_current_frame(&mut self) {
        self.render_frame_after_step(None);
    }

    /// Whether the runtime is on a VDP-rendered shell screen (Sega/intro/
    /// title/level card) rather than gameplay. Shells can use this for
    /// context-sensitive hints — the ROM holds level cards until a button
    /// edge, which reads as a hang without one.
    pub fn on_shell_screen(&self) -> bool {
        self.runtime.vdp_state().is_some()
    }

    fn render_frame_after_step(&mut self, previous_trace: Option<&NativeTraceFrame>) {
        self.rgb.fill(0);
        let render_trace = render_trace_with_vdp_latched_state_and_resident(
            &self.trace,
            previous_trace,
            self.runtime.source_port_render_trace_snapshot(),
        );
        if let Some(vdp) = self.runtime.vdp_state() {
            // VDP shell screens (Sega/intro/title/level card) mirror the
            // capture_scenario_frames pipeline (wildside-cli sweep/frames.rs):
            // the pure renderer draws the frame, then the ROM-derived
            // title-card letter workers composite over it.
            let snapshot = wildside_md::VdpSnapshot {
                registers: vdp.registers,
                vram: vdp.vram.clone(),
                cram: vdp.cram.clone(),
                vsram: vdp.vsram.clone(),
                raster_frame: vdp.raster_frame.as_ref().map(|raster_frame| {
                    wildside_md::VdpRasterFrame {
                        initial_vram: raster_frame.initial_vram.clone(),
                        initial_cram: raster_frame.initial_cram.clone(),
                        vram_word_writes: raster_frame
                            .vram_word_writes
                            .iter()
                            .map(|write| wildside_md::VdpRasterVramWordWrite {
                                scanline: write.scanline,
                                pixel: write.pixel,
                                byte_addr: write.byte_addr,
                                value: write.value,
                            })
                            .collect(),
                    }
                }),
            };
            wildside_md::render_vdp_into(&snapshot, &mut self.rgb);
            render_vdp_trace_overlay(&render_trace, &self.visuals, &mut self.rgb);
        } else {
            let scene = build_scene(
                &render_trace,
                false,
                Some(&self.visuals),
                self.runtime.source_port_render_block_grid(),
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

    /// Interleaved-stereo f32 samples generated by the latest `step` (empty
    /// until [`enable_audio`](Self::enable_audio) runs). Length follows the
    /// 60Hz remainder division of the output rate, so consecutive frames can
    /// differ by one stereo pair (e.g. 800 vs 801 frames at 48kHz).
    pub fn audio_samples(&self) -> &[f32] {
        self.audio
            .as_ref()
            .map_or(&[], |audio| audio.samples.as_slice())
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
    use std::path::PathBuf;

    use wildside_data::{FsAssetSource, manifest_path};

    /// Locate a cached generated bundle root (any ROM sha1), honouring the
    /// `KID_REPO_ROOT` override the cache-backed integration tests use.
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
        eprintln!(
            "skipping cache-backed web session test: no generated bundle in .cache/generated"
        );
        None
    }

    fn cached_session(level: Option<u8>) -> Option<WebSession> {
        let root = cached_generated_root()?;
        Some(
            WebSession::from_source(Arc::new(FsAssetSource::new(root)), level)
                .expect("cached web session"),
        )
    }

    #[test]
    fn expands_rgb24_to_opaque_rgba() {
        let rgb = [1u8, 2, 3, 4, 5, 6];
        let mut rgba = [0u8; 8];
        expand_rgb_to_rgba(&rgb, &mut rgba);
        assert_eq!(rgba, [1, 2, 3, 255, 4, 5, 6, 255]);
    }

    #[test]
    fn cached_session_audio_enable_generates_latest_device_rate_samples() {
        let Some(mut session) = cached_session(Some(0)) else {
            return;
        };

        if let Err(err) = session.enable_audio(48_000) {
            eprintln!("skipping cache-backed web audio test: {err:#}");
            return;
        }
        assert!(
            session.audio_samples().is_empty(),
            "audio samples should stay empty until the first stepped frame"
        );

        session.step(0);

        assert_eq!(
            session.audio_samples().len(),
            800 * 2,
            "ASM Load_InGame (kid.asm:23623-23629) web audio should drive the shared GEMS director and 48kHz sample clock for one 60Hz frame"
        );
    }

    #[test]
    fn cached_session_shell_screen_and_debug_state_reflect_runtime() {
        let Some(attract) = cached_session(None) else {
            return;
        };
        let Some(mut gameplay) = cached_session(Some(0)) else {
            return;
        };
        for _ in 0..120 {
            if !gameplay.on_shell_screen() {
                break;
            }
            gameplay.step(0);
        }

        assert!(
            attract.on_shell_screen(),
            "ASM Load_SegaScreen starts as a VDP-rendered shell screen"
        );
        assert!(
            !gameplay.on_shell_screen(),
            "ASM Load_InGame should leave the browser session on the gameplay renderer"
        );

        let debug = attract.debug_state();
        assert!(
            debug.contains("mode=SegaScreen")
                && debug.contains("level=0x00")
                && debug.contains("vdp=true")
                && debug.contains("grid=true")
                && debug.contains("rgb_nonzero="),
            "debug_state should expose the runtime mode, level, shell renderer, grid state, and framebuffer summary"
        );
    }

    #[test]
    fn cached_session_step_reload_updates_stale_visuals_level() {
        let Some(mut session) = cached_session(Some(0)) else {
            return;
        };
        session.visuals_level = 0xFF;

        session.step(0);

        assert_eq!(
            session.trace.current_level.0, 0,
            "ASM Load_InGame keeps the warped level id stable for this one-frame probe"
        );
        assert_eq!(
            session.visuals_level, 0,
            "the web session should reload visuals when the trace level and cached visual level diverge"
        );
    }
}
