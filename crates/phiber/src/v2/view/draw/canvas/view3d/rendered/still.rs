//! Finished stills through phiray — for the examples' side-by-side checks
//! against Blender and for the Render job — drawn the way the viewport draws
//! a shading mode, off the UI's worker, in bands when the frame outgrows
//! one binding (`bands.rs`).

use super::super::mesh::BMeshPalette;
use super::bands::BBands;
use super::scene;
use crate::v2::scene::BObject3D;
use gpui::Hsla;

/// One finished still of `objects` through phiray, drawn the way the viewport
/// draws `shading` — for side-by-side checks against Blender
/// (`examples/color_render.rs`). Returns RGBA8 rows (top first) and the
/// samples averaged, or `None` without a GPU adapter or when the scene does
/// not fit. `plain` paints anything without a material, `samples` is how far
/// the still refines (Blender's Max Samples) and `denoise` runs the a-trous
/// filter over the finished frame (Blender's Denoise).
pub fn still(objects: &[BObject3D], shading: crate::v2::scene::BShading, plain: Hsla, cam: phiray::Camera, w: u32, h: u32, samples: u32, denoise: bool) -> Option<(Vec<u8>, u32)> {
    use crate::v2::scene::BShading;
    let gpu = phiray::Ray::gpu()?;
    let palette = BMeshPalette { shading, cube: plain, sphere: plain, gem: plain, loaded: plain, accent: plain, accent_active: plain };
    let cam = with_studio(cam, shading);
    let mut tracer = phiray::Tracer::from_scene(&gpu, scene::build(objects, &palette))?;
    tracer.set_samples(samples);
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
    shading: crate::v2::scene::BShading,
    plain: Hsla,
    cam: phiray::Camera,
    w: u32,
    h: u32,
    samples: u32,
    denoise: bool,
    mut step: impl FnMut(u32, Option<BStill>) -> bool,
) -> Option<u32> {
    use crate::v2::scene::BShading;
    let gpu = phiray::Ray::gpu()?;
    let palette = BMeshPalette { shading, cube: plain, sphere: plain, gem: plain, loaded: plain, accent: plain, accent_active: plain };
    let cam = with_studio(cam, shading);
    let mut tracer = phiray::Tracer::from_scene(&gpu, scene::build(objects, &palette))?;
    tracer.set_samples(samples);
    tracer.set_mode(match shading {
        BShading::Wireframe => phiray::Mode::Wire,
        BShading::Solid => phiray::Mode::Solid,
        BShading::Material => phiray::Mode::Preview,
        BShading::Rendered => phiray::Mode::Rendered,
    });
    let bands = BBands::of(w, h, BBands::most(&gpu));
    let mut whole = BStill::empty(w, h);
    for (k, (y0, rows)) in bands.list.iter().enumerate() {
        // A band is drawn two rows over into its neighbours and laid in
        // without them: the effects and the anti-aliasing gather each read
        // a pixel's neighbours and clamp at a tile's edge, and the overlap
        // is what makes the seam invisible.
        let (over, under) = (2 * u32::from(*y0 > 0), 2 * u32::from(y0 + rows < h));
        let band = phiray::Camera { tile: if bands.list.len() > 1 { [0, y0 - over, w, h] } else { [0; 4] }, ..cam };
        loop {
            let (taken, done) = tracer.sample(&gpu, band, w, rows + over + under);
            // Reading the image back is the slow part, so progress carries a
            // count and the picture only at the end.
            let go_on = done || step(bands.progress(k, taken + 1, tracer.samples()), None);
            if done || !go_on {
                if denoise && done {
                    tracer.denoise(&gpu);
                }
                whole.lay(&BStill::read(&tracer, &gpu), *y0 as usize, *rows as usize, over as usize, w as usize);
                if !go_on {
                    step(bands.progress(k, taken + 1, tracer.samples()), Some(whole));
                    return Some(taken + 1);
                }
                break;
            }
        }
    }
    step(tracer.samples(), Some(whole));
    Some(tracer.samples())
}

/// Material Preview is lit by the studio image `scene::build` puts in the
/// atlas: a camera that was not told so (built by hand rather than by
/// `lit`) gets the lighting words here, or every surface comes out a dark
/// mirror of the raw image.
fn with_studio(mut cam: phiray::Camera, shading: crate::v2::scene::BShading) -> phiray::Camera {
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

    fn empty(w: u32, h: u32) -> Self {
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

#[cfg(test)]
mod tests {
    use crate::v2::scene::{BObject3D, BShading, BShape};

    /// Four bands (12, 12, 12 and 4 rows, each drawn two rows over) of a
    /// 40 × 40 frame are the frame drawn whole, pixel for pixel —
    /// anti-aliased and outlined in Solid — so a Render larger than the
    /// device binds comes out as one image with no seam and no blank band.
    #[test]
    fn a_still_rendered_in_bands_is_the_still_rendered_whole() {
        if phiray::Ray::gpu().is_none() {
            return;
        }
        let objects = vec![BObject3D::Primitive { name: "Cube".into(), shape: BShape::Cube, x: 0.0, y: 0.0 }, BObject3D::Primitive { name: "Ball".into(), shape: BShape::Sphere, x: -2.0, y: 0.0 }];
        let look = super::super::world::BRayLook { sky: gpui::black(), grid: gpui::white(), grid_hide: 0, world: None, lights: Vec::new(), wire: None, view: [0.0, 0.0, 1.0, 0.0], environment: None, set: Default::default(), studio: false };
        let setup = crate::v2::scene::camera::setup_from_state(&crate::v2::state::BState::default());
        let cam = phiray::Camera { fx: [1.0, 0.5, 0.7, 1.0], ..super::super::scene::camera(20.0, 30.0, (0.0, 0.0), 1.0, setup, 1.0, &look) };
        let (whole, _) = super::still(&objects, BShading::Solid, gpui::white(), cam, 40, 40, 1, false).expect("a GPU");
        super::super::bands::force(Some(40 * 16));
        let mut banded = None;
        super::still_stepwise(&objects, BShading::Solid, gpui::white(), cam, 40, 40, 1, false, |_, done| {
            banded = banded.take().or(done);
            true
        });
        super::super::bands::force(None);
        let banded = banded.expect("the finished still");
        assert_eq!(banded.rgba.len(), whole.len());
        assert_eq!(banded.rgba, whole, "the bands differ from the frame drawn whole");
        assert!(banded.depth.iter().all(|d| *d > 0.0), "every band's depth was laid in");
    }

    /// A Material Preview still asked for with a camera built by hand
    /// (the examples) is lit by the studio image all the same — `lit`'s
    /// words, not a dark mirror — and a camera that has them, or another
    /// shading, is left alone.
    #[test]
    fn a_hand_built_material_preview_camera_is_lit_by_the_studio() {
        let cam = super::with_studio(phiray::Camera::default(), BShading::Material);
        let expected = super::super::studio::environment().is_some();
        assert_eq!(cam.preview != [0; 4], expected, "{:?}", cam.preview);
        assert_eq!((cam.env_hidden == 1, cam.envmap > 0), (expected, expected));
        let own = phiray::Camera { preview: [9, 9, 9, 9], ..Default::default() };
        assert_eq!(super::with_studio(own, BShading::Material).preview, [9, 9, 9, 9]);
        assert_eq!(super::with_studio(phiray::Camera::default(), BShading::Solid).preview, [0; 4]);
    }

}
