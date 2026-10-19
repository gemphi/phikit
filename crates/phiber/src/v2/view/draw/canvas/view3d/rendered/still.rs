//! Finished stills through phiray — for the examples' side-by-side checks
//! against Blender and for the Render job — drawn the way the viewport draws
//! a shading mode, off the UI's worker, in bands when the frame outgrows
//! one binding (`bands.rs`).

use super::super::mesh::BMeshPalette;
use super::bands::BBandList;
use super::scene;
use crate::v2::scene::BObject3D;
use gpui::Hsla;

/// How a still is drawn: the shading mode, the colour of anything without a
/// material, and — for Solid — the matcap and whether the materials' images
/// go up (`BMeshPalette`).
#[derive(Clone, Copy, Debug)]
pub struct BStillLook {
    pub shading: crate::v2::scene::BShading,
    pub plain: Hsla,
    pub matcap: Option<&'static str>,
    pub texture: bool,
}

impl BStillLook {
    /// `shading` with no images of its own.
    pub fn new(shading: crate::v2::scene::BShading, plain: Hsla) -> Self {
        Self { shading, plain, matcap: None, texture: false }
    }

    /// A look for MEASURING a scene (`scene_fit`), where the question is how
    /// many bytes the built arrays come to. The colour reaches only the
    /// palette's entries, never a count, so there is no shade to take from
    /// the theme here — `Hsla::default()` says that, where a grey written out
    /// in numbers looked like a choice nobody could trace.
    pub fn measuring(shading: crate::v2::scene::BShading) -> Self {
        Self::new(shading, Hsla::default())
    }

    pub(super) fn palette(&self) -> BMeshPalette {
        let plain = self.plain;
        BMeshPalette { shading: self.shading, cube: plain, sphere: plain, gem: plain, loaded: plain, accent: plain, accent_active: plain, matcap: self.matcap, texture: self.texture }
    }
}

/// One finished still of `objects` through phiray, drawn the way the viewport
/// draws `shading` — for side-by-side checks against Blender
/// (`examples/color_render.rs`). Returns RGBA8 rows (top first) and the
/// samples averaged, or `None` without a GPU adapter or when the scene does
/// not fit. `plain` paints anything without a material, `samples` is how far
/// the still refines (Blender's Max Samples) and `denoise` runs the a-trous
/// filter over the finished frame (Blender's Denoise).
/// A render, settled, as one image.
///
/// Banded through [`still_stepwise`] rather than traced in one pass: a
/// frame with more pixels than the device draws at once comes back short,
/// and the caller then has a buffer that does not fill `w × h`. This used
/// to trace `w × h` directly, so `phiber -b … -f 1` on a 1440 × 1440 frame
/// failed with *"the pixels do not fill the frame"* while the app's own F12,
/// which already banded, was fine (`bands.rs`, `Tracer::max_pixels`).
/// One implementation now, so the command line and F12 cannot diverge again.
pub fn still(
    objects: &[BObject3D],
    look: BStillLook,
    cam: phiray::Camera,
    w: u32,
    h: u32,
    samples: u32,
    denoise: bool,
    // Adaptive Sampling and the Time Limit as the plan resolved them
    // (`BRenderPlan::adapt`, `time_limit`). This used to pass `[0.0; 4]`
    // — adaptive off, always — so a headless render never had the
    // settings the panel showed, and `--samples 4096` meant 4096 for
    // every pixel.
    adapt: [f32; 4],
    time_limit: f32,
) -> Option<(Vec<u8>, u32)> {
    let mut image: Option<Vec<u8>> = None;
    let taken = still_stepwise(objects, look, cam, w, h, samples, denoise, adapt, time_limit, super::region::BStillRegion::default(), |_, at| {
        if let Some(settled) = at.image() {
            image = Some(settled.rgba);
        }
        true
    })?;
    Some((image?, taken))
}

/// A settled render, in the three forms a save might want: the bytes the
/// screen shows, the display-space floats behind them (a 16-bit PNG), and the
/// scene-linear radiance (an EXR or HDR).
///
/// The floats are read back from the same buffer as the bytes, so they cost
/// one more copy rather than a second render.
pub struct BStill {
    /// RGBA8, row 0 at the top.
    pub rgba: Vec<u8>,
    /// The same image as floats, the view transform already applied.
    pub display: Vec<[f32; 3]>,
    /// Radiance before the view transform — what a deep format stores.
    pub linear: Vec<[f32; 3]>,
    /// Blender's "Diffuse Color" pass, scene-linear.
    pub albedo: Vec<[f32; 3]>,
    /// Blender's "Normal" pass, world space.
    pub normal: Vec<[f32; 3]>,
    /// Blender's "Depth" pass; a ray that met nothing reads 1e10.
    pub depth: Vec<f32>,
}

/// What a step of [`still_stepwise`] carries besides its sample count.
pub enum BStillAt {
    /// The count alone: nothing was read back this sample.
    Counting,
    /// The frame as it stands mid-render — its bytes only, the float passes
    /// empty (`BStill::peek`) — so the Rendering space can show the render
    /// as it happens rather than a bar over nothing. Blender's
    /// `RE_display_update_cb` hands the viewer its tiles the same way.
    Peek(BStill),
    /// The settled frame, every pass read.
    Settled(BStill),
}

impl BStillAt {
    /// The image, if this step carries one.
    pub fn image(self) -> Option<BStill> {
        match self {
            Self::Counting => None,
            Self::Peek(s) | Self::Settled(s) => Some(s),
        }
    }

    pub fn settled(&self) -> bool {
        matches!(self, Self::Settled(_))
    }
}

/// How often a running render's bytes are read back for the viewer: a peek
/// a few times a second is what the eye needs, and reading the picture back
/// is the slow part of a sample, so it is not done every one.
pub const PEEK_EVERY: std::time::Duration = std::time::Duration::from_millis(350);

#[path = "stepwise.rs"]
mod stepwise;
pub use stepwise::{still_stepwise, with_studio};


impl BStill {
    /// The tracer's image as it stands, in every form.
    fn read(tracer: &phiray::Tracer, gpu: &phiray::Gpu) -> Self {
        Self { rgba: tracer.pixels(gpu), display: tracer.display(gpu), linear: tracer.radiance(gpu), albedo: tracer.albedo(gpu), normal: tracer.normal(gpu), depth: tracer.depth(gpu) }
    }

    /// The bytes alone, for a look at a render under way.
    pub(super) fn peek(rgba: Vec<u8>) -> Self {
        Self { rgba, display: Vec::new(), linear: Vec::new(), albedo: Vec::new(), normal: Vec::new(), depth: Vec::new() }
    }

    /// `lay` for the bytes alone.
    /// A short `piece` is laid as far as it goes rather than panicking: a
    /// readback that timed out comes back empty, and a peek is not worth a
    /// crash (`gpu::PatienceFor` is what stops it happening).
    pub(super) fn lay_rgba(into: &mut [u8], piece: &[u8], y0: usize, rows: usize, skip: usize, w: usize) {
        let (at, from, n) = (y0 * w, skip * w, rows * w);
        let (dst, src) = ((at * 4, (at + n) * 4), (from * 4, (from + n) * 4));
        if dst.1 > into.len() || src.1 > piece.len() {
            return;
        }
        into[dst.0..dst.1].copy_from_slice(&piece[src.0..src.1]);
    }

    pub(super) fn empty(w: u32, h: u32) -> Self {
        let n = (w * h) as usize;
        Self { rgba: vec![0; n * 4], display: vec![[0.0; 3]; n], linear: vec![[0.0; 3]; n], albedo: vec![[0.0; 3]; n], normal: vec![[0.0; 3]; n], depth: vec![0.0; n] }
    }

    /// `rows` of `piece`, `w` pixels wide, from its row `skip`, laid in from
    /// row `y0`.
    fn lay(&mut self, piece: &Self, y0: usize, rows: usize, skip: usize, w: usize) {
        let (at, from, n) = (y0 * w, skip * w, rows * w);
        // A band whose readback came back short leaves the frame as it was
        // rather than bringing the render down (`lay_rgba`).
        if (at + n) * 4 > self.rgba.len() || (from + n) * 4 > piece.rgba.len() {
            return;
        }
        self.rgba[at * 4..(at + n) * 4].copy_from_slice(&piece.rgba[from * 4..(from + n) * 4]);
        self.display[at..at + n].copy_from_slice(&piece.display[from..from + n]);
        self.linear[at..at + n].copy_from_slice(&piece.linear[from..from + n]);
        self.albedo[at..at + n].copy_from_slice(&piece.albedo[from..from + n]);
        self.normal[at..at + n].copy_from_slice(&piece.normal[from..from + n]);
        self.depth[at..at + n].copy_from_slice(&piece.depth[from..from + n]);
    }
}
