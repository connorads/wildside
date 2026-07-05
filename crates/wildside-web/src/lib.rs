//! Browser shell for the source port (#19): a thin wasm-bindgen adapter over
//! [`WebSession`]. All rendering/input logic lives in `session.rs` (natively
//! testable); this layer only moves bytes across the JS boundary.
//!
//! The JS side reads the RGBA framebuffer zero-copy via
//! [`frame_ptr`](WildsideGame::frame_ptr)/[`frame_len`](WildsideGame::frame_len)
//! over wasm linear memory, rebuilding its view if memory grows.

mod session;

pub use session::{FRAME_RGB_LEN, FRAME_RGBA_LEN, WebSession};

use sha1::{Digest, Sha1};
use wasm_bindgen::prelude::*;
use wildside_frame::{FRAME_HEIGHT, FRAME_WIDTH};

/// One playable game instance bound to a fetched asset bundle.
#[wasm_bindgen]
pub struct WildsideGame {
    session: WebSession,
}

#[wasm_bindgen]
impl WildsideGame {
    /// Boot from a packed bundle blob (`wildside pack-web-bundle` output).
    /// `undefined` boots the real attract flow (Sega logo → intro → title
    /// menu); a number warps straight into that level (0 = first level).
    #[wasm_bindgen(constructor)]
    pub fn new(bundle: &[u8], level: Option<u8>) -> Result<WildsideGame, JsError> {
        let session = WebSession::from_bundle_blob(bundle, level)
            .map_err(|error| JsError::new(&format!("{error:#}")))?;
        Ok(Self { session })
    }

    /// Advance one frame with the raw controller byte
    /// (Up/Down/Left/Right/B/C/A/Start = bits 0..7).
    pub fn step(&mut self, mask: u8) {
        self.session.step(mask);
    }

    /// Pointer to the RGBA framebuffer in wasm linear memory.
    pub fn frame_ptr(&self) -> *const u8 {
        self.session.rgba_frame().as_ptr()
    }

    /// Byte length of the RGBA framebuffer.
    pub fn frame_len(&self) -> usize {
        self.session.rgba_frame().len()
    }

    /// SHA-1 of the current RGB24 frame — matches a hash of the raw pixel data
    /// in a native `capture-frames` PNG for the same scenario frame, so parity
    /// is checkable from the browser console without downloading the buffer.
    pub fn frame_rgb_sha1(&self) -> String {
        let mut hasher = Sha1::new();
        hasher.update(self.session.rgb_frame());
        format!("{:x}", hasher.finalize())
    }

    /// Whether the engine is on a VDP shell screen (Sega/intro/title/level
    /// card). Level cards hold until a button edge, so the JS shell can hint.
    pub fn on_shell_screen(&self) -> bool {
        self.session.on_shell_screen()
    }

    /// Diagnostic snapshot for the browser console (game mode, level, render
    /// branch); throwaway aid while the shell is young.
    pub fn debug_state(&self) -> String {
        self.session.debug_state()
    }

    pub fn width() -> u32 {
        FRAME_WIDTH
    }

    pub fn height() -> u32 {
        FRAME_HEIGHT
    }
}
