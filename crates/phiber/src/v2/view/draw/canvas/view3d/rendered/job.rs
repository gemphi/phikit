//! What the UI hands the render thread (`worker.rs`) and what it gets back:
//! a scene or a view to trace, and a published frame.

use super::super::mesh::BMeshPalette;
use crate::v2::scene::BObject3D;
use phiray::{Camera, Mode};

pub(super) enum BRayJob {
    /// The scene's identity and its shape (`BRayKey`), and the scene.
    Scene(u64, u64, Vec<BObject3D>, BMeshPalette),
    View(u64, BRayShot),
}

/// One frame to trace: the camera and size, which kernel draws it, and what
/// the Render panel asked for.
#[derive(Clone, Copy, Debug)]
pub(super) struct BRayShot {
    pub cam: Camera,
    pub w: u32,
    pub h: u32,
    pub mode: Mode,
    /// How far to refine (Blender's Max Samples).
    pub samples: u32,
    /// Viewport Denoising: the sample the shown frame is filtered from
    /// (`Tracer::denoise_shown`), or none.
    pub denoise_from: Option<u32>,
}

/// A published image (BGRA8, row 0 at the top).
pub(super) struct BRayFrame {
    pub seq: u64,
    pub view: u64,
    pub w: u32,
    pub h: u32,
    pub bgra: Vec<u8>,
    pub done: bool,
}
