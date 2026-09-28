//! Material Preview and Rendered shading through phiray: the scene is
//! ray traced on a render thread ([`worker`]) and painted as one image that
//! refines while the view is still. Material Preview uses the studio rig
//! (like Blender's default HDRI preview); Rendered uses the file's lights and
//! world. Falls back to the mesh painter when
//! there is no GPU adapter.

mod key;
mod scene;
mod settle;
mod still;
pub mod studio;
mod vert;
mod worker;
mod world;
pub use self::still::{BStill, still, still_stepwise};
pub use self::world::{BRayLook, ENV_CDF_SLOT, ENV_SLOT, environment, light, lit, view_words};
#[cfg(test)]
mod tests;

use super::mesh::BMeshPalette;
use crate::v2::scene::BObject3D;
use crate::v2::scene::camera::BViewSetup;
use gpui::{Bounds, Corners, Hsla, Pixels, RenderImage, Window};
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use self::worker::{BRayJob, BRayShot, BRayWorker};

/// Longest rendered side in device pixels (keeps frames interactive).
const MAX_SIDE: f32 = 1600.0;

#[derive(Default)]
struct BRayState {
    worker: Option<Option<BRayWorker>>,
    scene: u64,
    view: u64,
    /// Shown image: (frame seq, view key, finished, image).
    image: Option<(u64, u64, bool, Arc<RenderImage>)>,
    /// Holds the traced size still while the viewport is being dragged.
    settle: self::settle::BSettle,
}

thread_local! {
    static STATE: RefCell<BRayState> = RefCell::new(BRayState::default());
    /// How many views have been posted to the tracer — each one restarts it.
    static POSTED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// How many times the tracer has been restarted with a new view. A window
/// drag should add at most a couple, however many frames it lasts.
pub fn views_posted() -> usize {
    POSTED.with(std::cell::Cell::get)
}

/// What the painter needs this frame.
#[derive(Clone, Copy)]
pub(super) struct BRayView {
    pub yaw: f32,
    pub pitch: f32,
    pub pan: (f32, f32),
    pub zoom: f32,
    pub setup: BViewSetup,
    pub sky: Hsla,
    pub grid: Hsla,
    /// Material Preview: studio lighting, file lights and world ignored.
    pub studio: bool,
    /// Wireframe: the wire's colour.
    pub wire: Hsla,
    /// Which phiray kernel draws it — Workbench for Solid, the tracer
    /// otherwise. Solid finishes in one dispatch, so it never re-polls.
    pub mode: phiray::Mode,
    /// What Properties → Render asks for: how far to refine, how many diffuse
    /// bounces, whether to denoise (`scene::render_set`).
    pub set: crate::v2::scene::BRenderSet,
}

/// Why the GPU is not drawing this frame, when it is not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super::super) enum BRayDeclined {
    /// No adapter at all: the CPU painter is the only renderer.
    NoGpu,
    /// The scene exceeds what this device can bind, even chunked.
    TooLarge,
}

/// Hand `scene` to the render thread when it changed. `Err` → the GPU is not
/// drawing, and says why, so the fallback can decide what it can honestly do.
pub(super) fn prepare(scene: &[BObject3D], palette: &BMeshPalette) -> Result<(), BRayDeclined> {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.worker.get_or_insert_with(BRayWorker::start);
        let key = self::key::key(scene, palette);
        let fresh = key != s.scene;
        let Some(Some(w)) = s.worker.as_ref() else { return Err(BRayDeclined::NoGpu) };
        if fresh {
            w.send(BRayJob::Scene(key, scene.to_vec(), *palette));
        }
        let refused = w.refused(key);
        if fresh {
            s.scene = key;
            s.view = 0;
        }
        if refused { Err(BRayDeclined::TooLarge) } else { Ok(()) }
    })
}

/// Post the view, then paint the newest image into `bounds`.
pub(super) fn paint(bounds: Bounds<Pixels>, view: BRayView, window: &mut Window) {
    let (bw, bh) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    if bw < 1.0 || bh < 1.0 {
        return;
    }
    let dpr = window.scale_factor();
    let fit = (MAX_SIDE / (bw.max(bh) * dpr)).min(1.0);
    let live = ((bw * dpr * fit) as u32, (bh * dpr * fit) as u32);
    // While the viewport is being dragged, trace at the size already in hand
    // and stretch it; take the new size once the drag stops (`settle`).
    let ((w, h), waiting) =
        STATE.with(|s| s.borrow_mut().settle.size(live, std::time::Instant::now()));
    let look = BRayLook {
        sky: view.sky,
        grid: view.grid,
        set: view.set,
        // Workbench (Solid, Wireframe) draws the theme's viewport shade behind
        // the scene, whatever the file's world says (`View3DShading.
        // background_type` THEME); only Rendered looks through to the world.
        world: match (view.studio, view.mode) {
            (_, phiray::Mode::Wire | phiray::Mode::Solid) | (true, _) => None,
            (false, _) => crate::v2::scene::source::with_loaded(|m| m.map(|m| m.world.background.radiance())),
        },
        environment: match (view.studio, view.mode) {
            (_, phiray::Mode::Wire | phiray::Mode::Solid) => None,
            // Material Preview: Blender's studio light, whatever the file's world says.
            (true, _) => self::studio::environment().map(|_| self::studio::ROTATION),
            (false, _) => self::world::environment().map(|(_, rot)| rot),
        },
        lights: if view.studio { Vec::new() } else { crate::v2::scene::scene_lights() },
        wire: (view.mode == phiray::Mode::Wire).then_some(view.wire),
        view: self::world::view_words(),
        studio: view.studio,
    };
    // The camera frames the size being traced, not the live bounds: the image
    // is stretched to fit, so a drag shows the old frame widened rather than
    // a new one squeezed into the old pixels.
    let cam = self::scene::camera(view.yaw, view.pitch, view.pan, view.zoom, view.setup, w as f32 / h.max(1) as f32, &look);
    let (image, settled) = STATE.with(|s| {
        let mut s = s.borrow_mut();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        // The sample target and the denoise flag are not part of `Camera`,
        // so they join the key by hand — changing either has to restart the
        // refinement, not carry on with the old one.
        (s.scene, w, h, view.mode as u32, bytemuck::bytes_of(&cam), view.set.preview_samples, view.set.denoise)
            .hash(&mut hasher);
        let key = hasher.finish();
        let Some(Some(worker)) = s.worker.as_ref() else { return (None, true) };
        if key != s.view {
            POSTED.with(|n| n.set(n.get() + 1));
            worker.send(BRayJob::View(
                key,
                BRayShot { cam, w, h, mode: view.mode, samples: view.set.preview_samples, denoise: view.set.denoise },
            ));
        }
        let seq = s.image.as_ref().map_or(0, |i| i.0);
        let frame = worker.take(seq);
        s.view = key;
        // An empty frame is one the kernel never wrote (`worker`): nothing to
        // show, but the view has settled — the theme canvas stays.
        let mut blank = false;
        if let Some(f) = frame {
            if f.bgra.is_empty() {
                blank = f.done && f.view == key;
            } else if let Some(buf) = image::RgbaImage::from_raw(f.w, f.h, f.bgra) {
                let img = Arc::new(RenderImage::new(smallvec::smallvec![image::Frame::new(buf)]));
                if let Some((.., old)) = s.image.replace((f.seq, f.view, f.done, img)) {
                    let _ = window.drop_image(old);
                }
            }
        }
        match &s.image {
            Some((_, v, done, img)) => (Some(img.clone()), *done && *v == key),
            None => (None, blank),
        }
    });
    if let Some(img) = image {
        let _ = window.paint_image(bounds, bounds, Corners::default(), img, 0, false);
    }
    // Poll for new samples until the current view is fully refined — and
    // while a new size is waiting, so the end of the drag is seen.
    if !settled || waiting {
        window.request_animation_frame();
    }
}
