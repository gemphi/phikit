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

    fn palette(&self) -> BMeshPalette {
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
pub fn still(objects: &[BObject3D], look: BStillLook, cam: phiray::Camera, w: u32, h: u32, samples: u32, denoise: bool) -> Option<(Vec<u8>, u32)> {
    use crate::v2::scene::BShading;
    let gpu = phiray::Ray::gpu()?;
    let (shading, palette) = (look.shading, look.palette());
    let cam = with_studio(cam, shading);
    let mut tracer = phiray::Tracer::from_scene(&gpu, scene::build(objects, &palette))?;
    // A Workbench render's `render_aa` is its anti-aliasing count; the
    // path tracer's samples are its own.
    tracer.set_samples(samples);
    tracer.set_aa_samples(samples);
    tracer.set_mode(match shading {
        BShading::Wireframe => phiray::Mode::Wire,
        BShading::Solid => phiray::Mode::Solid,
        BShading::Material => phiray::Mode::Preview,
        BShading::Rendered => phiray::Mode::Rendered,
    });
    while !tracer.sample(&gpu, cam, w, h).1 {}
    if denoise {
        tracer.denoise(&gpu);
    }
    composite(&gpu, &mut tracer);
    Some((tracer.pixels(&gpu), tracer.samples()))
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

/// The same render, one sample at a time: `step` is called after each with
/// the samples taken so far and, once the frame has settled, the finished
/// image. Returning `false` from it stops the render early, which is how a
/// job is cancelled: the image as far as it got — the bands finished and
/// the one under way at its samples so far — is handed to `step` as the
/// finished one, as Blender keeps a cancelled render's tiles.
///
/// A frame with more pixels than the device binds as one image
/// (`Tracer::max_pixels`) is rendered in horizontal bands, each a tile of
/// the frame (`Camera::tile`), and the bands are joined — so a 4K frame
/// renders whole on a device that binds 128 MiB. The samples reported step
/// over the bands together, so progress still ends at `samples`.
///
/// `still` is this without the reporting; a final render uses this one so the
/// Rendering space can show progress rather than freeze (`scene/render_job.rs`).
pub fn still_stepwise(
    objects: &[BObject3D],
    look: BStillLook,
    cam: phiray::Camera,
    w: u32,
    h: u32,
    samples: u32,
    denoise: bool,
    region: super::region::BStillRegion,
    mut step: impl FnMut(u32, Option<BStill>) -> bool,
) -> Option<u32> {
    use crate::v2::scene::BShading;
    let gpu = phiray::Ray::gpu()?;
    let (shading, palette) = (look.shading, look.palette());
    let cam = with_studio(cam, shading);
    let mut tracer = phiray::Tracer::from_scene(&gpu, scene::build(objects, &palette))?;
    // A Workbench render's `render_aa` is its anti-aliasing count; the
    // path tracer's samples are its own.
    tracer.set_samples(samples);
    tracer.set_aa_samples(samples);
    tracer.set_mode(match shading {
        BShading::Wireframe => phiray::Mode::Wire,
        BShading::Solid => phiray::Mode::Solid,
        BShading::Material => phiray::Mode::Preview,
        BShading::Rendered => phiray::Mode::Rendered,
    });
    // The pixels drawn: the region of the frame — two pixels wider each
    // way, for the same reason as the bands' overlap, and cut back after —
    // or the frame itself.
    // A band is drawn some rows over into its neighbours and laid in
    // without them, and the region with that margin around it: the effects
    // and the anti-aliasing gather each read a pixel's neighbours and clamp
    // at a tile's edge, and the overlap is what makes the seam invisible —
    // two rows, or SMAA's reach when the single pass leaves it on the
    // finished frame (`Tracer::seam_rows`).
    let reach = tracer.seam_rows();
    let (rx, ry, rw, rh) = region.drawn(w, h, reach);
    let bands = BBandList::of(rw, rh, BBandList::most(&gpu), reach);
    let mut drawn = BStill::empty(rw, rh);
    for (k, (y0, rows)) in bands.list.iter().enumerate() {
        let (over, under) = (reach.min(*y0), reach.min(rh - y0 - rows));
        let tiled = bands.list.len() > 1 || region.pixels.is_some();
        let band = phiray::Camera { tile: if tiled { [rx, ry + y0 - over, w, h] } else { [0; 4] }, ..cam };
        loop {
            let (taken, done) = tracer.sample(&gpu, band, rw, rows + over + under);
            // Reading the image back is the slow part, so progress carries a
            // count and the picture only at the end.
            let go_on = done || step(bands.progress(k, taken + 1, tracer.samples()), None);
            if done || !go_on {
                if denoise && done {
                    tracer.denoise(&gpu);
                }
                if done {
                    composite(&gpu, &mut tracer);
                }
                drawn.lay(&BStill::read(&tracer, &gpu), *y0 as usize, *rows as usize, over as usize, rw as usize);
                if !go_on {
                    step(bands.progress(k, taken + 1, tracer.samples()), Some(region.framed(drawn, w, h, reach)));
                    return Some(taken + 1);
                }
                break;
            }
        }
    }
    step(tracer.samples(), Some(region.framed(drawn, w, h, reach)));
    Some(tracer.samples())
}

/// The scene's compositor over the finished render, as Blender's render
/// pipeline runs it: on the scene-linear pixels, before the view transform
/// (`scene/composite.rs`). Nothing happens when the file has no tree, it is
/// switched off, or nothing reaches its output. A band renders as its own
/// image, so each band is composited on its own — which is sound only
/// because every node that runs is per-pixel; a node that read a
/// neighbourhood would see a seam, and those are exactly the ones
/// `BCompTree::declined` refuses.
fn composite(gpu: &phiray::Gpu, tracer: &mut phiray::Tracer) {
    use crate::v2::scene::composite::BComposite;
    if let Some(tree) = crate::v2::scene::composite::tree() {
        let run = tracer.composite(gpu, &tree);
        if run.any() {
            crate::v2::app::journal::BJournal::note(run.line());
        }
    }
}

/// Material Preview is lit by the studio image `scene::build` puts in the
/// atlas: a camera that was not told so (built by hand rather than by
/// `lit`) gets the lighting words here, or every surface comes out a dark
/// mirror of the raw image.
pub(super) fn with_studio(mut cam: phiray::Camera, shading: crate::v2::scene::BShading) -> phiray::Camera {
    if shading == crate::v2::scene::BShading::Material && cam.preview == [0; 4] && super::studio::environment().is_some() {
        super::world::studio_lighting(&mut cam);
    }
    cam
}

impl BStill {
    /// The tracer's image as it stands, in every form.
    fn read(tracer: &phiray::Tracer, gpu: &phiray::Gpu) -> Self {
        Self { rgba: tracer.pixels(gpu), display: tracer.display(gpu), linear: tracer.radiance(gpu), albedo: tracer.albedo(gpu), normal: tracer.normal(gpu), depth: tracer.depth(gpu) }
    }

    pub(super) fn empty(w: u32, h: u32) -> Self {
        let n = (w * h) as usize;
        Self { rgba: vec![0; n * 4], display: vec![[0.0; 3]; n], linear: vec![[0.0; 3]; n], albedo: vec![[0.0; 3]; n], normal: vec![[0.0; 3]; n], depth: vec![0.0; n] }
    }

    /// `rows` of `piece`, `w` pixels wide, from its row `skip`, laid in from
    /// row `y0`.
    fn lay(&mut self, piece: &Self, y0: usize, rows: usize, skip: usize, w: usize) {
        let (at, from, n) = (y0 * w, skip * w, rows * w);
        self.rgba[at * 4..(at + n) * 4].copy_from_slice(&piece.rgba[from * 4..(from + n) * 4]);
        self.display[at..at + n].copy_from_slice(&piece.display[from..from + n]);
        self.linear[at..at + n].copy_from_slice(&piece.linear[from..from + n]);
        self.albedo[at..at + n].copy_from_slice(&piece.albedo[from..from + n]);
        self.normal[at..at + n].copy_from_slice(&piece.normal[from..from + n]);
        self.depth[at..at + n].copy_from_slice(&piece.depth[from..from + n]);
    }
}
