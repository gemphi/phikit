//! Finished stills through phiray — for the examples' side-by-side checks
//! against Blender and for the Render job — drawn the way the viewport draws
//! a shading mode, off the UI's worker.

use super::super::mesh::BMeshPalette;
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
/// image. Returning `false` from it stops the render early, which is
/// how a job is cancelled.
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
    let mut tracer = phiray::Tracer::from_scene(&gpu, scene::build(objects, &palette))?;
    tracer.set_samples(samples);
    tracer.set_mode(match shading {
        BShading::Wireframe => phiray::Mode::Wire,
        BShading::Solid => phiray::Mode::Solid,
        BShading::Material => phiray::Mode::Preview,
        BShading::Rendered => phiray::Mode::Rendered,
    });
    loop {
        let (taken, done) = tracer.sample(&gpu, cam, w, h);
        if done {
            if denoise {
                tracer.denoise(&gpu);
            }
            let still = BStill {
                rgba: tracer.pixels(&gpu),
                display: tracer.display(&gpu),
                linear: tracer.radiance(&gpu),
                albedo: tracer.albedo(&gpu),
                normal: tracer.normal(&gpu),
                depth: tracer.depth(&gpu),
            };
            step(tracer.samples(), Some(still));
            return Some(tracer.samples());
        }
        // Reading the image back is the slow part, so progress carries a
        // count and the picture only at the end.
        if !step(taken + 1, None) {
            return Some(taken + 1);
        }
    }
}
