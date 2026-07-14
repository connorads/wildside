//! Mega Drive support: the machine emulation/probe runtime plus a pure-Rust
//! software VDP renderer. The renderer lives in [`vdp`] with no CPU/audio
//! dependencies; everything else (68k + Z80 + ymfm + GEMS) is in `runtime`,
//! behind the default-on `audio` feature — ymfm-sys is C++ and cannot build
//! for bare wasm, so the web shell depends on this crate with
//! `default-features = false` for shell-screen rendering (#19).

#[cfg(feature = "audio")]
mod runtime;
mod vdp;

#[cfg(feature = "audio")]
pub use runtime::*;
pub use vdp::{
    FrameBuffer, VdpPixelLayerSample, VdpPixelSample, VdpRasterFrame, VdpRasterVramWordWrite,
    VdpSnapshot, render_vdp_into, sample_vdp_pixel,
};
