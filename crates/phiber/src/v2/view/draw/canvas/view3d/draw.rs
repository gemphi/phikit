//! Who draws the viewport: the GPU (phiray) or the CPU painter.
//!
//! Blender draws every shading mode on the GPU — Solid and Wireframe through
//! Workbench, Material and Rendered through EEVEE or Cycles. Phiber does the
//! same: Solid is `phiray::Mode::Solid`, a single Workbench dispatch over the
//! same BVH the tracer uses, and Material/Rendered are the path tracer.
//!
//! The CPU painter is no longer a performance tier — it is the fallback for a
//! machine with no GPU adapter, and the Wireframe overlay. Solid used to run
//! on it below 200k triangles, which meant the common case re-shaded every
//! pixel on one thread inside the paint pass. That was the crawl.

use crate::v2::scene::BShading;
use std::cell::Cell;

/// The most the CPU painter will draw. It exists for a machine with no GPU
/// and for a scene the GPU declined; above this it re-shades every pixel of
/// millions of triangles inside the paint pass and the window stops
/// answering the pointer. Refusing and saying so is the honest alternative.
pub const CPU_MAX_TRIS: usize = 400_000;

/// Can the CPU painter take this scene at all?
pub fn cpu_can_draw(tris: usize) -> bool {
    tris <= CPU_MAX_TRIS
}

thread_local! {
    /// Did the last painted frame use the GPU? (`app::trace`)
    static ON_GPU: Cell<bool> = const { Cell::new(false) };
}

/// True when the viewport's last frame was drawn by phiray.
pub fn on_gpu() -> bool {
    ON_GPU.with(Cell::get)
}

pub(super) fn record(gpu: bool) {
    ON_GPU.with(|f| f.set(gpu));
}

/// Does this shading mode light the scene with the studio rig?
pub(super) fn studio(shading: BShading) -> bool {
    matches!(shading, BShading::Material | BShading::Solid)
}

/// Which phiray mode draws this shading. Every mode has a kernel now; the CPU
/// painter is reached only when there is no GPU adapter at all. Material
/// Preview is its own deterministic kernel (`Mode::Preview`), as EEVEE is
/// to Cycles: it used to be path traced like Rendered, so every orbit frame
/// was one noisy sample of a scene of plain cubes.
pub(super) fn gpu_mode(shading: BShading) -> Option<phiray::Mode> {
    match shading {
        BShading::Solid => Some(phiray::Mode::Solid),
        BShading::Material => Some(phiray::Mode::Preview),
        BShading::Rendered => Some(phiray::Mode::Rendered),
        BShading::Wireframe => Some(phiray::Mode::Wire),
    }
}

/// Should this frame go to the GPU?
#[cfg(test)]
fn wants_gpu(shading: BShading) -> bool {
    gpu_mode(shading).is_some()
}

/// Who draws this frame.
pub(super) struct BPlan {
    /// phiray has the scene and will paint it.
    pub ray: bool,
    /// The CPU painter draws — only with no GPU, and only under its ceiling.
    pub cpu_draws: bool,
    /// The kernel phiray uses for this shading.
    pub mode: phiray::Mode,
}

/// Decide the renderer for this frame and tell the overlay when neither can
/// draw. The CPU painter is bounded: past its ceiling it draws nothing and
/// the overlay says why, rather than freezing the window on a scene it cannot
/// shade in a frame.
pub(super) fn plan(shading: BShading, scene: &std::rc::Rc<Vec<crate::v2::scene::BObject3D>>, palette: &super::mesh::BMeshPalette) -> BPlan {
    use super::rendered::{self, BRayDeclined};
    let mode = gpu_mode(shading);
    let gpu = match mode {
        Some(_) => rendered::prepare(scene, palette),
        None => Err(BRayDeclined::NoGpu),
    };
    let ray = gpu.is_ok();
    record(ray);
    let tris = crate::v2::scene::scene_metrics(scene).2;
    let cpu_draws = !ray && cpu_can_draw(tris);
    super::super::overlay::note_gpu(gpu.err().filter(|_| !cpu_draws), tris);
    BPlan { ray, cpu_draws, mode: mode.unwrap_or(phiray::Mode::Rendered) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shaded_mode_draws_on_the_gpu() {
        // Solid is the one that regressed: it used to fall to the CPU painter
        // below a triangle count, which is where the viewport lost its frame
        // rate on ordinary scenes.
        for shading in BShading::ALL {
            assert!(wants_gpu(shading), "{shading:?} must not draw on the CPU");
        }
    }

    #[test]
    fn the_cpu_painter_refuses_what_it_cannot_draw_interactively() {
        // A 9M-triangle character declined by the GPU used to go to the CPU
        // painter whole. One frame took seconds and the window stopped
        // answering clicks. The painter now has a ceiling, and the viewport
        // says so instead of freezing.
        assert!(cpu_can_draw(CPU_MAX_TRIS));
        assert!(!cpu_can_draw(8_970_212));
    }

    #[test]
    fn solid_uses_workbench_not_the_path_tracer() {
        assert_eq!(gpu_mode(BShading::Solid), Some(phiray::Mode::Solid));
        assert_eq!(gpu_mode(BShading::Rendered), Some(phiray::Mode::Rendered));
        // Wireframe is a ray cast too, so an edge behind a face is hidden —
        // the CPU stroke path had no depth test and drew every edge over
        // every other.
        assert_eq!(gpu_mode(BShading::Wireframe), Some(phiray::Mode::Wire));
    }
}
