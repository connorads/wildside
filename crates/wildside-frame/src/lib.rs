//! SDL-free CPU rasteriser shared by `wildside-cli`'s native player and the
//! future browser (WASM) build.
//!
//! This crate owns the pure layer that turns a gameplay trace
//! (`wildside-core`) plus decoded visuals (`wildside-render`) into
//! [`DrawCommand`]s and then CPU-rasterises them into a fixed 320×224
//! framebuffer. It holds **zero** platform/SDL references: SDL (native) and a
//! canvas blit (web) only upload, scale, and present the buffer this crate
//! produces.
//!
//! ## Pixel format: RGB24
//!
//! The rasteriser writes packed **RGB24** (3 bytes per pixel, no alpha),
//! `len = FRAME_WIDTH * FRAME_HEIGHT * 3`. Native SDL consumes this directly
//! via `PixelFormatEnum::RGB24`; the web build expands to RGBA (alpha = 255)
//! at its blit boundary rather than widening the stride here. See issue #18.

// Faithful 68k translations carry wide signatures and complex tuple/closure
// types; same crate-level posture the native player used before the move.
#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::doc_lazy_continuation
)]

mod dto;
// The render modules are `pub` (not just glob-re-exported) so the native
// player's qualified `scene::`/`sprites::` call sites keep resolving after the
// move; their flat API is also re-exported below for bare-name callers.
pub mod hud;
pub mod render;
pub mod scene;
pub mod sprites;

pub use dto::*;
pub use hud::*;
pub use render::*;
pub use scene::*;
pub use sprites::*;
