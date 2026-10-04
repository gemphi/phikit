//! Material Preview and Rendered shading through phiray: the scene is
//! ray traced on a render thread ([`worker`]) and painted as one image that
//! refines while the view is still. Material Preview uses the studio rig
//! (like Blender's default HDRI preview); Rendered uses the file's lights and
//! world. Falls back to the mesh painter when
//! there is no GPU adapter.

mod atlas;
mod bands;
mod build;
mod clock;
mod job;
mod coarse;
mod key;
mod pace;
mod place;
mod posted;
mod reframe;
mod scene;
mod settle;
mod still;
pub mod studio;
mod vert;
mod view;
mod wake;
mod worker;
mod world;
pub use self::still::{BStill, BStillLook, still, still_stepwise};
pub use self::world::{BRayLook, ENV_CDF_SLOT, ENV_IRRADIANCE_SLOT, ENV_PREFILTERED_SLOT, ENV_SLOT, environment, light, lit, studio_lighting, view_words};
#[cfg(test)]
mod paint_tests;
#[cfg(test)]
mod tests;

use super::mesh::BMeshPalette;
use crate::v2::scene::BObject3D;
use gpui::{Bounds, Corners, Pixels, RenderImage, Window};
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use self::worker::{BRayJob, BRayShot, BRayWorker};
pub(in crate::v2::view::draw::canvas::view3d) use self::view::BRayView;
pub use self::wake::{drawing, landed, refining, set_ticking, take_fresh, ticking};
pub(super) use self::wake::REFINING;

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
    /// What each view posted lately was framed for (`reframe.rs`).
    shots: self::reframe::BShotLog,
    /// How long the posted view takes to arrive (`clock.rs`).
    clock: self::clock::BViewClock,
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
pub(super) fn prepare(scene: &std::rc::Rc<Vec<BObject3D>>, palette: &BMeshPalette) -> Result<(), BRayDeclined> {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        // Another render device was picked: the old worker's thread ends with
        // its channel, and the new one builds the scene again.
        if s.worker.as_ref().is_some_and(|w| w.as_ref().is_some_and(|w| w.generation != phiray::Gpu::generation())) {
            (s.worker, s.scene) = (None, 0);
        }
        s.worker.get_or_insert_with(BRayWorker::start);
        let key = self::key::BRayKey::of_list(scene, palette);
        let fresh = key != s.scene;
        let Some(Some(w)) = s.worker.as_ref() else { return Err(BRayDeclined::NoGpu) };
        if fresh {
            w.send(BRayJob::Scene(key, self::key::BRayKey::shape(scene, palette), scene.to_vec(), *palette));
        }
        let refused = w.refused(key);
        if fresh {
            s.scene = key;
            s.view = 0;
        }
        if refused { Err(BRayDeclined::TooLarge) } else { Ok(()) }
    })
}

/// Paint the traced frame. Answers whether an image actually landed: the
/// tracer is asynchronous, so the first frames after a change — and every
/// frame on a machine whose adapter went away — have nothing to show, and
/// the caller has to draw something rather than leave the canvas empty.
pub(super) fn paint(bounds: Bounds<Pixels>, view: BRayView, corners: Corners<Pixels>, window: &mut Window) -> bool {
    let (bw, bh) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    if bw < 1.0 || bh < 1.0 {
        return false;
    }
    let dpr = window.scale_factor();
    let fit = (MAX_SIDE / (bw.max(bh) * dpr)).min(1.0);
    // And no more pixels than the device can bind as one image
    // (`Tracer::max_pixels`): a larger view is traced smaller and stretched.
    let most = phiray::Gpu::opened().map_or(f32::MAX, |gpu| phiray::Tracer::max_pixels(&gpu) as f32);
    let fit = fit * (most / (bw * bh * dpr * dpr * fit * fit)).sqrt().min(1.0);
    let live = ((bw * dpr * fit) as u32, (bh * dpr * fit) as u32);
    // While the viewport is being dragged, trace at the size already in hand
    // and stretch it; take the new size once the drag stops (`settle`).
    let ((w, h), waiting) =
        STATE.with(|s| s.borrow_mut().settle.size(live, std::time::Instant::now()));
    let look = BRayLook {
        sky: view.sky,
        grid: view.grid,
        grid_hide: view.grid_hide,
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
        view: view.view,
        studio: view.studio,
    };
    // The camera frames the size being traced, not the live bounds: the image
    // is stretched to fit, so a drag shows the old frame widened rather than
    // a new one squeezed into the old pixels.
    let cam = phiray::Camera {
        see_through: view.see_through,
        fx: view.fx,
        cavity: view.cavity,
        ssao: view.ssao,
        shadow: view.shadow,
        shadow_fade: view.shadow_fade,
        matcap: view.matcap,
        outline: view.outline,
        shade: view.shade,
        single: view.single,
        ..self::scene::camera(view.yaw, view.pitch, view.pan, view.zoom, view.setup, w as f32 / h.max(1) as f32, &look)
    };
    let (image, settled) = STATE.with(|s| {
        let (mut held, mut hasher) = (s.borrow_mut(), std::collections::hash_map::DefaultHasher::new());
        let s = &mut *held;
        // The sample target and the denoise flag are not part of `Camera`,
        // so they join the key by hand — changing either has to restart the
        // refinement, not carry on with the old one.
        (s.scene, w, h, view.mode as u32, bytemuck::bytes_of(&cam), view.set.preview_samples, view.set.preview_denoise, view.set.preview_denoise_from)
            .hash(&mut hasher);
        let key = hasher.finish();
        let Some(Some(worker)) = s.worker.as_ref() else { return (None, true) };
        let shot = |aspect| self::reframe::BShotView { zoom: view.zoom, pan: view.pan, aspect, orbit: (view.yaw, view.pitch, view.setup.pivot, view.setup.perspective) };
        if key != s.view {
            s.shots.note(key, shot(w as f32 / h.max(1) as f32));
            s.clock.posted(key);
            POSTED.with(|n| n.set(n.get() + 1));
            worker.send(BRayJob::View(
                key,
                BRayShot { cam, w, h, mode: view.mode, samples: view.set.preview_samples, denoise_from: view.set.preview_denoise.then_some(view.set.preview_denoise_from) },
            ));
        }
        let seq = s.image.as_ref().map_or(0, |i| i.0);
        let frame = worker.take(seq);
        s.view = key;
        // An empty frame is one the kernel never wrote (`worker`): nothing to
        // show, but the view has settled — the theme canvas stays.
        let mut blank = false;
        if let Some(f) = frame {
            if let Some(line) = s.clock.landed(f.view, f.done) {
                crate::v2::app::journal::BJournal::note(line);
            }
            if f.bgra.is_empty() {
                blank = f.done && f.view == key;
            } else if let Some(buf) = image::RgbaImage::from_raw(f.w, f.h, f.bgra) {
                let img = Arc::new(RenderImage::new(smallvec::smallvec![image::Frame::new(buf)]));
                if let Some((.., old)) = s.image.replace((f.seq, f.view, f.done, img)) {
                    let _ = window.drop_image(old);
                }
            }
        }
        // The image where its own view puts it in this one (`reframe.rs`).
        let place = |v: u64| s.shots.get(v).map_or(bounds, |old| old.place(&shot(bw / bh), bounds));
        match &s.image {
            Some((_, v, done, img)) => (Some((img.clone(), place(*v))), *done && *v == key),
            None => (None, blank),
        }
    });
    let painted = match image {
        Some((img, at)) => {
            let _ = window.paint_image(bounds, at, corners, img, 0, false);
            true
        }
        None => false,
    };
    // While a new size is waiting, draw again so the end of the drag is
    // seen; while the view refines, the app draws when a frame lands (the
    // flag is cleared as each window draw begins, `wake::drawing`, so any
    // one unsettled viewport keeps it).
    if !settled {
        REFINING.store(true, std::sync::atomic::Ordering::Release);
    }
    // No ticker yet to carry the frames the tracer is about to land: one
    // more frame, in which `render` sees the flag and starts it (`wake`).
    if waiting || (!settled && !self::wake::ticking()) {
        window.request_animation_frame();
    }
    painted
}
