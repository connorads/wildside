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

mod dto;

pub use dto::*;
