//! Mega Drive support: the machine emulation/probe runtime plus a pure-Rust
//! software VDP renderer. The renderer lives in [`vdp`] with no CPU/audio
//! dependencies; everything else (68k + Z80 + ymfm + GEMS) is in `runtime`.

mod runtime;
mod vdp;

pub use runtime::*;
pub use vdp::{
    FrameBuffer, VdpPixelLayerSample, VdpPixelSample, VdpSnapshot, render_vdp_into,
    sample_vdp_pixel,
};
